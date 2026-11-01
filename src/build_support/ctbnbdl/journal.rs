// SPDX-License-Identifier: AGPL-3.0-or-later
/*
This file is part of Collective Toolbox, a database and document workspace and utilities.
Copyright (C) 2026 Collective Toolbox Developers
Contact: info@collectivetoolbox.com

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU Affero General Public License as published by the Free
Software Foundation, either version 3 of the License, or (at your option) any
later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License along
with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

//! Unified durable append-only journal (`ledger-journal.jsonl`), single-inode reader/writer
//! `flock` guards on the repository's permanent ledger lock, standalone pending-transaction
//! recovery, and full `.ctbnbdl` bundle `(id, tokens)` verification.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::{
    CtbnbdlFramingDcs, LedgerNode, MAX_BYTES_PER_BUNDLE, MAX_NODES_PER_BUNDLE,
    MAX_NODES_PER_CHUNK, SYSTEM_REGION_START, decode_nodes_from_ctbnbdl_bytes_with,
    encode_nodes_to_ctbnbdl_bundles_with_limits, format_chunk_bundle_filename,
    format_chunk_filename, highwater_shadow_dirs, list_chunk_files,
    load_ledger_nodes_from_dir_raw, load_ledger_nodes_from_str,
    parse_chunk_filename_start_id, partition_active_and_inactive_nodes_with,
    verify_nodes_append_only_against_baseline, write_chunk_file_append_only,
    write_ctbnbdl_bundles_to_dir,
};
use crate::fs2::FileExt;
use crate::sha2::{Digest, Sha256};
use crate::shorthand::DcShorthand;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Decodes a UTF-8 string value node from its token slice if all tokens are Unicode codepoints.
#[must_use]
pub fn decode_unicode_string_tokens(tokens: &[DcShorthand]) -> Option<String> {
    if tokens.is_empty() {
        return None;
    }
    let mut out = String::with_capacity(tokens.len());
    for &tok in tokens {
        let DcShorthand::Unicode(cp) = tok else {
            return None;
        };
        let ch = char::from_u32(cp)?;
        out.push(ch);
    }
    Some(out)
}

/// Resolves the 16 `CtbnbdlFramingDcs` from an in-memory slice of `.ctbnodes` [`LedgerNode`]s
/// by bootstrapping from the `"ProgrammaticIdentifier"` string value node.
///
/// # Errors
/// Returns an error if `"ProgrammaticIdentifier"` or any required framing Dc identifier
/// cannot be resolved from `nodes`.
pub fn resolve_ctbnbdl_framing_dcs_from_nodes(
    nodes: &[LedgerNode],
) -> Result<CtbnbdlFramingDcs> {
    let mut strings_by_id: HashMap<u128, String> = HashMap::new();
    let mut tokens_by_id: HashMap<u128, &[DcShorthand]> =
        HashMap::with_capacity(nodes.len());

    for node in nodes {
        tokens_by_id.insert(node.id, node.tokens.as_slice());
        if let Some(s) = decode_unicode_string_tokens(&node.tokens) {
            strings_by_id.insert(node.id, s);
        }
    }

    let prog_ident_str_ids: HashSet<u128> = strings_by_id
        .iter()
        .filter_map(|(&id, s)| {
            if s == "ProgrammaticIdentifier" {
                Some(id)
            } else {
                None
            }
        })
        .collect();
    ensure!(
        !prog_ident_str_ids.is_empty(),
        "System graph is missing the \"ProgrammaticIdentifier\" string value node"
    );

    let mut bootstrap_triple: Option<(DcShorthand, DcShorthand, DcShorthand)> =
        None;
    for node in nodes {
        if let &[
            has_value_dc,
            DcShorthand::Long(claim_id),
            DcShorthand::Long(val_id),
        ] = node.tokens.as_slice()
        {
            if prog_ident_str_ids.contains(&val_id) {
                if let Some(&&[has_a_dc, subj, prop]) =
                    tokens_by_id.get(&claim_id)
                {
                    if subj == prop && matches!(subj, DcShorthand::Short(_)) {
                        bootstrap_triple =
                            Some((has_a_dc, has_value_dc, prop));
                        break;
                    }
                }
            }
        }
    }

    let (has_a_dc, has_value_dc, prog_ident_dc) = bootstrap_triple.context(
        "Failed to bootstrap HasA, HasValue, and ProgrammaticIdentifier from system graph",
    )?;

    let extract_prog_ident_subj = |claim_tokens: &[DcShorthand]| -> Option<DcShorthand> {
        match claim_tokens {
            &[ha, subj, prop]
            | &[ha, _, subj, prop]
                if ha == has_a_dc
                    && prop == prog_ident_dc
                    && matches!(subj, DcShorthand::Short(_)) =>
            {
                Some(subj)
            }
            _ => None,
        }
    };

    let mut raw_ident_to_short: HashMap<&str, DcShorthand> = HashMap::new();
    for node in nodes {
        if let &[
            pred,
            DcShorthand::Long(claim_id),
            DcShorthand::Long(str_id),
        ] = node.tokens.as_slice()
        {
            if pred == has_value_dc {
                if let Some(claim_tokens) = tokens_by_id.get(&claim_id) {
                    if let Some(subj) = extract_prog_ident_subj(claim_tokens) {
                        if let Some(ident_str) = strings_by_id.get(&str_id) {
                            raw_ident_to_short
                                .insert(ident_str.as_str(), subj);
                        }
                    }
                }
            }
        }
    }

    let sh_supersedes = *raw_ident_to_short
        .get("Supersedes")
        .context("Missing @ident(\"Supersedes\") in system graph")?;
    let sh_retracts = *raw_ident_to_short
        .get("Retracts")
        .context("Missing @ident(\"Retracts\") in system graph")?;
    let sh_enclosing_doc = *raw_ident_to_short
        .get("EnclosingDocument")
        .context("Missing @ident(\"EnclosingDocument\") in system graph")?;

    let mut inactive_ids: HashSet<u128> = HashSet::new();
    for node in nodes.iter().rev() {
        if inactive_ids.contains(&node.id) {
            continue;
        }
        if let &[verb, _subj, DcShorthand::Long(old_id)] =
            node.tokens.as_slice()
        {
            if verb == sh_supersedes || verb == sh_retracts {
                inactive_ids.insert(old_id);
            }
        }
    }

    let mut active_ident_to_short: HashMap<&str, DcShorthand> = HashMap::new();
    for node in nodes {
        if inactive_ids.contains(&node.id) {
            continue;
        }
        if let &[
            pred,
            DcShorthand::Long(claim_id),
            DcShorthand::Long(str_id),
        ] = node.tokens.as_slice()
        {
            if pred == has_value_dc && !inactive_ids.contains(&claim_id) {
                if let Some(claim_tokens) = tokens_by_id.get(&claim_id) {
                    if let Some(subj) = extract_prog_ident_subj(claim_tokens) {
                        if let Some(ident_str) = strings_by_id.get(&str_id) {
                            active_ident_to_short
                                .insert(ident_str.as_str(), subj);
                        }
                    }
                }
            }
        }
    }

    let get_req = |ident: &str| -> Result<DcShorthand> {
        active_ident_to_short.get(ident).copied().with_context(|| {
            format!(
                "Missing required @ident(\"{ident}\") in system graph for ctbnbdl framing"
            )
        })
    };

    Ok(CtbnbdlFramingDcs {
        begin_node: get_req("BeginGraphNodeDefinition")?,
        end_node: get_req("EndGraphNodeDefinition")?,
        escape: get_req("Escape")?,
        evaluate_as_atom: get_req("EvaluateValueAsAtom")?,
        has_a: get_req("HasA")?,
        has_value: get_req("HasValue")?,
        preferred: get_req("Preferred")?,
        supersedes: sh_supersedes,
        retracts: sh_retracts,
        enclosing_document: sh_enclosing_doc,
        is_instance_of: get_req("IsInstanceOf")?,
        is_subtype_of: get_req("IsSubtypeOf")?,
        is_based_on: get_req("IsBasedOn")?,
        is_deprecated: get_req("IsDeprecated")?,
        is_closing_delimiter_for: get_req("IsClosingDelimiterFor")?,
        is_associated_with: get_req("IsAssociatedWith")?,
    })
}

/// Relative path from the repository root to the unified durable ledger journal.
pub const LEDGER_JOURNAL_REL_PATH: &str =
    "src/formats/dcdata/data-src/nodes/allocated/ledger-journal.jsonl";

/// Relative path from the repository root to the local post-initialization marker.
pub const JOURNAL_INITIALIZED_REL_PATH: &str =
    "src/formats/dcdata/data-src/nodes/allocated/journal-initialized";

/// Relative path from the repository root to the shared permanent lock canary file.
pub const LOCK_CANARY_REL_PATH: &str =
    "src/formats/dcdata/data-src/nodes/ledger.lock";

/// Exclusive writer lock guard (`flock(LOCK_EX)`) on the permanent ledger lock.
#[derive(Debug)]
pub struct LedgerWriteGuard {
    repo_root: PathBuf,
    file: File,
}

/// Shared reader lock guard (`flock(LOCK_SH)`) on the permanent ledger lock.
#[derive(Debug)]
pub struct LedgerReadGuard {
    repo_root: PathBuf,
    file: File,
}

fn open_or_create_lock_canary(repo_root: &Path) -> Result<File> {
    let canary_path = repo_root.join(LOCK_CANARY_REL_PATH);
    if !canary_path.exists() {
        if let Some(parent) = canary_path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create {}", parent.display())
            })?;
        }
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .open(&canary_path)
            .with_context(|| {
                format!("Failed to create lock canary {}", canary_path.display())
            })?;
    }
    OpenOptions::new()
        .read(true)
        .open(&canary_path)
        .with_context(|| {
            format!(
                "Failed to open lock canary {} for flock",
                canary_path.display()
            )
        })
}

impl LedgerWriteGuard {
    /// Acquires an exclusive lock on the repository's permanent ledger lock.
    pub fn acquire(repo_root: &Path) -> Result<Self> {
        let file = open_or_create_lock_canary(repo_root)?;
        file.lock_exclusive().with_context(|| {
            format!(
                "Failed to acquire exclusive flock(LOCK_EX) on {}",
                repo_root.join(LOCK_CANARY_REL_PATH).display()
            )
        })?;
        Ok(Self {
            repo_root: repo_root.to_path_buf(),
            file,
        })
    }

    /// Returns the repository root protected by this write guard.
    #[must_use]
    pub fn repo_root(&self) -> &Path {
        &self.repo_root
    }
}

impl Drop for LedgerWriteGuard {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.file);
    }
}

impl LedgerReadGuard {
    /// Acquires a shared lock on the repository's permanent ledger lock.
    pub fn acquire(repo_root: &Path) -> Result<Self> {
        let file = open_or_create_lock_canary(repo_root)?;
        file.lock_shared().with_context(|| {
            format!(
                "Failed to acquire shared flock(LOCK_SH) on {}",
                repo_root.join(LOCK_CANARY_REL_PATH).display()
            )
        })?;
        Ok(Self {
            repo_root: repo_root.to_path_buf(),
            file,
        })
    }

    /// Returns the repository root protected by this read guard.
    #[must_use]
    pub fn repo_root(&self) -> &Path {
        &self.repo_root
    }
}

impl Drop for LedgerReadGuard {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.file);
    }
}

/// Fault-injection hook for deterministic crash and recovery unit tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultInjectionPoint {
    /// Write only `bytes_to_keep` bytes of the `BATCH_PENDING` line (without trailing `\n`) and abort.
    TornPendingRecord {
        /// Number of prefix bytes of the serialized JSON line to write before aborting.
        bytes_to_keep: usize,
    },
    /// Abort during Phase 1 after appending `bytes_to_keep` bytes of `item_index`'s `.ctbnodes` line (without `\n`).
    TornChunkLine {
        /// Zero-based index of the item in `items` whose `.ctbnodes` line should be torn.
        item_index: usize,
        /// Number of prefix bytes of `raw_line` to write before aborting.
        bytes_to_keep: usize,
    },
    /// Abort during Phase 1 after item `item_index` has been appended to `.ctbnodes` and moved to `nodes/allocated/`.
    AfterItemAppendedAndMoved {
        /// Zero-based index of the item after which to abort.
        item_index: usize,
    },
    /// Abort after all items are written and moved, before invoking `fsync` on files and directories.
    BeforeFsync,
    /// Write only `bytes_to_keep` bytes of the `BATCH_COMMITTED` line (without trailing `\n`) and abort.
    TornCommittedRecord {
        /// Number of prefix bytes of the serialized JSON line to write before aborting.
        bytes_to_keep: usize,
    },
}

/// A single planned node append item in a durable ledger transaction and/or manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannedLedgerItem {
    /// Allocated global node ID (`l<node_id>`).
    pub node_id: u128,
    /// Complete formatted `.ctbnodes` line (without trailing `\n`).
    pub raw_line: String,
    /// Target `.ctbnodes` chunk filename (e.g. `"85899_52392.ctbnodes"`).
    pub target_chunk: String,
    /// Optional parsed token strings (present in precomputed manifests).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tokens: Vec<String>,
    /// Optional comment string (present in precomputed manifests).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub comment: String,
    /// Provenance owner (`"csv"` or `"manual"`).
    #[serde(default)]
    pub provenance: String,
    /// Optional human-readable rationale from `# @rationale:`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
    /// Authorized active duplicate node IDs from `# @allow-duplicate-of:`.
    #[serde(default)]
    pub allow_duplicate_of: Vec<u128>,
    /// Source filename in `src/formats/dcdata/data-src/nodes/inbox/` (for `inbox_allocation`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inbox_name: Option<String>,
    /// Destination filename in `src/formats/dcdata/data-src/nodes/allocated/` (for `inbox_allocation`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allocated_name: Option<String>,
    /// Expected SHA-256 hex digest of the `.dcsh` source file (for `inbox_allocation`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inbox_sha256: Option<String>,
    /// Optional full `.dcsh` file contents (used in precomputed transaction manifests).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dcsh_content: Option<String>,
}

impl PlannedLedgerItem {
    /// Returns a normalized copy without manifest-only staging fields (`tokens`, `comment`, `dcsh_content`)
    /// so `BATCH_PENDING.items` and `manifest.planned_items` can be compared structurally.
    #[must_use]
    pub fn normalized_for_journal(&self) -> Self {
        Self {
            node_id: self.node_id,
            raw_line: self.raw_line.clone(),
            target_chunk: self.target_chunk.clone(),
            tokens: Vec::new(),
            comment: String::new(),
            provenance: self.provenance.clone(),
            rationale: self.rationale.clone(),
            allow_duplicate_of: self.allow_duplicate_of.clone(),
            inbox_name: self.inbox_name.clone(),
            allocated_name: self.allocated_name.clone(),
            inbox_sha256: self.inbox_sha256.clone(),
            dcsh_content: None,
        }
    }
}

/// Precomputed deterministic transaction manifest used with `--require-manifest`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionManifest {
    /// Human-readable manifest identifier.
    pub manifest_id: String,
    /// Transaction kind (`"inbox_allocation"` or `"csv_reconciliation"`).
    pub kind: String,
    /// Highest node ID in the baseline ledger prior to this transaction.
    pub baseline_last_id: u128,
    /// Total node count in the baseline ledger prior to this transaction.
    pub baseline_node_count: usize,
    /// Expected SHA-256 digest of the baseline `.ctbnodes` directory.
    pub baseline_ledger_sha256: String,
    /// Highest node ID in the ledger after this transaction completes.
    pub final_last_id: u128,
    /// Total node count in the ledger after this transaction completes.
    pub final_node_count: usize,
    /// Expected SHA-256 digest of the `.ctbnodes` directory after this transaction completes.
    pub final_ledger_sha256: String,
    /// Pinned SHA-256 digests of all input files (`nodes/inbox/*.dcsh` or CSV tables).
    pub pinned_input_sha256s: BTreeMap<String, String>,
    /// Ordered list of planned node allocations.
    pub planned_items: Vec<PlannedLedgerItem>,
}

/// A single JSONL record (`BATCH_PENDING` or `BATCH_COMMITTED`) in `ledger-journal.jsonl`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalRecord {
    /// Record state: `"BATCH_PENDING"` or `"BATCH_COMMITTED"`.
    pub state: String,
    /// Unique batch identifier matching `BATCH_PENDING` and `BATCH_COMMITTED`.
    pub batch_id: String,
    /// Transaction kind: `"bootstrap_checkpoint"`, `"inbox_allocation"`, or `"csv_reconciliation"`.
    pub kind: String,
    /// Highest node ID before this batch.
    pub baseline_last_id: u128,
    /// SHA-256 digest of `.ctbnodes` chunks before this batch.
    pub baseline_ledger_sha256: String,
    /// Highest node ID after this batch.
    pub final_last_id: u128,
    /// SHA-256 digest of `.ctbnodes` chunks after this batch.
    pub final_ledger_sha256: String,
    /// Optional SHA-256 digest of the required `TransactionManifest` bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manifest_sha256: Option<String>,
    /// Planned node items (recorded in `BATCH_PENDING`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<PlannedLedgerItem>,
    /// SHA-256 digests of active `.ctbnbdl` bundles in `assets/data/graph/` (recorded in `BATCH_COMMITTED`).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub active_bundles_sha256: BTreeMap<String, String>,
    /// SHA-256 digests of historical `.ctbnbdl` bundles in `assets/data/graph/system/` (recorded in `BATCH_COMMITTED`).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub history_bundles_sha256: BTreeMap<String, String>,
    /// Lowercase hex SHA-256 checksum over the canonical JSON serialization of this record with `record_sha256: ""`.
    pub record_sha256: String,
}

impl JournalRecord {
    /// Computes the expected `record_sha256` over this record with `record_sha256` cleared.
    pub fn compute_expected_sha256(&self) -> Result<String> {
        let mut clone = self.clone();
        clone.record_sha256.clear();
        let json = serde_json::to_string(&clone)
            .context("Failed to serialize JournalRecord for checksum")?;
        Ok(sha256_bytes_hex(json.as_bytes()))
    }

    /// Populates `self.record_sha256` with the computed digest and returns `self`.
    pub fn with_computed_sha256(mut self) -> Result<Self> {
        self.record_sha256 = self.compute_expected_sha256()?;
        Ok(self)
    }

    /// Verifies that `self.record_sha256` matches `compute_expected_sha256()`.
    pub fn verify_sha256(&self) -> Result<()> {
        ensure!(
            !self.record_sha256.is_empty(),
            "JournalRecord (batch_id={}, state={}) has empty record_sha256",
            self.batch_id,
            self.state
        );
        let expected = self.compute_expected_sha256()?;
        ensure!(
            self.record_sha256 == expected,
            "CRITICAL: JournalRecord checksum mismatch for batch_id={} state={}: expected {}, found {}",
            self.batch_id,
            self.state,
            expected,
            self.record_sha256
        );
        Ok(())
    }
}

/// Parsed state of `ledger-journal.jsonl`.
#[derive(Debug, Clone)]
pub struct ParsedJournal {
    /// Byte length of the validated `\n`-terminated prefix of `ledger-journal.jsonl`.
    pub valid_prefix_len: usize,
    /// Unterminated trailing line fragment (not ending in `\n`), if a crash interrupted the last write.
    pub torn_trailing_fragment: Option<Vec<u8>>,
    /// All complete `\n`-terminated records, each verified with `verify_sha256()`.
    pub records: Vec<JournalRecord>,
    /// All completed `(BATCH_PENDING, BATCH_COMMITTED)` pairs in chronological order.
    pub committed_pairs: Vec<(JournalRecord, JournalRecord)>,
    /// Currently open `BATCH_PENDING` record at the end of the journal, if any.
    pub open_pending: Option<JournalRecord>,
    /// Most recent `BATCH_COMMITTED` record, if any.
    pub last_committed: Option<JournalRecord>,
}

/// Computes the lowercase hex SHA-256 digest of `bytes`.
#[must_use]
pub fn sha256_bytes_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(64);
    for b in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{b:02x}");
    }
    out
}

/// Computes the lowercase hex SHA-256 digest of a file on disk.
pub fn sha256_file_hex(path: &Path) -> Result<String> {
    let bytes = fs::read(path)
        .with_context(|| format!("Failed to read {} for SHA-256", path.display()))?;
    Ok(sha256_bytes_hex(&bytes))
}

/// Invokes `sync_all()` on a file path.
pub fn fsync_file(path: &Path) -> Result<()> {
    let f = File::open(path).with_context(|| {
        format!("Failed to open {} for fsync", path.display())
    })?;
    f.sync_all()
        .with_context(|| format!("Failed to fsync {}", path.display()))
}

/// Invokes `sync_all()` on a directory path.
pub fn fsync_dir(dir: &Path) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    let f = File::open(dir).with_context(|| {
        format!("Failed to open directory {} for fsync", dir.display())
    })?;
    f.sync_all().with_context(|| {
        format!("Failed to fsync directory {}", dir.display())
    })
}

/// Returns the `.ctbnodes` chunk filename (`85899_34592.ctbnodes`, ...) that owns `node_id`.
#[must_use]
pub fn chunk_filename_for_node_id(node_id: u128) -> String {
    let offset = node_id.saturating_sub(SYSTEM_REGION_START);
    let max_u128 = u128::try_from(MAX_NODES_PER_CHUNK).unwrap_or(100);
    let chunk_start = SYSTEM_REGION_START.saturating_add(
        offset
            .checked_div(max_u128)
            .unwrap_or(0)
            .saturating_mul(max_u128),
    );
    format_chunk_filename(chunk_start)
}

/// Computes the canonical SHA-256 digest over all `.ctbnodes` chunk files in `graph_dir`,
/// sorted by ascending chunk start ID (`name\0content\0`).
pub fn compute_ledger_dir_sha256(graph_dir: &Path) -> Result<String> {
    let files = list_chunk_files(graph_dir)?;
    let mut hasher = Sha256::new();
    for path in files {
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        let bytes = fs::read(&path).with_context(|| {
            format!("Failed to read chunk {}", path.display())
        })?;
        hasher.update(name.as_bytes());
        hasher.update(b"\0");
        hasher.update(&bytes);
        hasher.update(b"\0");
    }
    let digest = hasher.finalize();
    let mut out = String::with_capacity(64);
    for b in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{b:02x}");
    }
    Ok(out)
}

/// Computes the canonical `.ctbnodes` directory SHA-256 digest from an in-memory slice of `LedgerNode`s.
#[must_use]
pub fn compute_nodes_slice_ledger_sha256(nodes: &[LedgerNode]) -> String {
    let mut hasher = Sha256::new();
    for chunk in nodes.chunks(MAX_NODES_PER_CHUNK) {
        let Some(first) = chunk.first() else {
            continue;
        };
        let name = format_chunk_filename(first.id);
        let mut content = String::new();
        for node in chunk {
            content.push_str(&node.format_line());
            content.push('\n');
        }
        hasher.update(name.as_bytes());
        hasher.update(b"\0");
        hasher.update(content.as_bytes());
        hasher.update(b"\0");
    }
    let digest = hasher.finalize();
    let mut out = String::with_capacity(64);
    for b in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{b:02x}");
    }
    out
}

/// Computes a sorted map of `<filename>.ctbnbdl -> sha256_hex` for all `.ctbnbdl` files directly in `dir`.
pub fn compute_bundle_dir_sha256s(
    dir: &Path,
) -> Result<BTreeMap<String, String>> {
    let mut map = BTreeMap::new();
    if !dir.is_dir() {
        return Ok(map);
    }
    for entry in fs::read_dir(dir)
        .with_context(|| format!("Failed to read {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("ctbnbdl") {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        map.insert(name.to_string(), sha256_file_hex(&path)?);
    }
    Ok(map)
}

fn decode_bundle_dir_nodes(
    dir: &Path,
    framing: CtbnbdlFramingDcs,
) -> Result<Vec<LedgerNode>> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut keyed_files: Vec<(u128, PathBuf)> = Vec::new();
    for entry in fs::read_dir(dir)
        .with_context(|| format!("Failed to read {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("ctbnbdl") {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let stem = name.strip_suffix(".ctbnbdl").unwrap_or(name);
        let digits: String = stem.chars().filter(|&c| c != '_').collect();
        let start_id: u128 = digits.parse().with_context(|| {
            format!("Invalid .ctbnbdl bundle filename {} in {}", name, dir.display())
        })?;
        keyed_files.push((start_id, path));
    }
    keyed_files.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

    let mut all_nodes = Vec::new();
    let mut prev_id: Option<u128> = None;
    for (_, path) in keyed_files {
        let bytes = fs::read(&path).with_context(|| {
            format!("Failed to read bundle {}", path.display())
        })?;
        let decoded = decode_nodes_from_ctbnbdl_bytes_with(&bytes, framing)
            .with_context(|| {
                format!("Failed to decode bundle {}", path.display())
            })?;
        for node in decoded {
            if let Some(prev) = prev_id {
                ensure!(
                    node.id > prev,
                    "Decoded .ctbnbdl node IDs in {} are not strictly ascending: l{} followed by l{}",
                    path.display(),
                    prev,
                    node.id
                );
            }
            prev_id = Some(node.id);
            all_nodes.push(node);
        }
    }
    Ok(all_nodes)
}

/// Verifies that the compiled active (`graph_dir/*.ctbnbdl`) and historical
/// (`graph_dir/system/*.ctbnbdl`) bundles match `committed_nodes` in:
/// 1. Exact bundle inventory and file SHA-256 digests (when `expected_active_sha256` / `expected_history_sha256` are provided),
/// 2. Disjoint strictly ascending node IDs (`active_ids.is_disjoint(&history_ids)`),
/// 3. Complete union equality in projected `(node.id, &node.tokens)` pairs (excluding comments), and
/// 4. Exact active vs. inactive partition equality against `partition_active_and_inactive_nodes_with`.
pub fn verify_compiled_bundles_match_ledger(
    graph_dir: &Path,
    committed_nodes: &[LedgerNode],
    framing: CtbnbdlFramingDcs,
    expected_active_sha256: Option<&BTreeMap<String, String>>,
    expected_history_sha256: Option<&BTreeMap<String, String>>,
) -> Result<()> {
    let history_dir = graph_dir.join("system");
    let actual_active_sha256 = compute_bundle_dir_sha256s(graph_dir)?;
    let actual_history_sha256 = compute_bundle_dir_sha256s(&history_dir)?;

    if let Some(expected_active) = expected_active_sha256 {
        ensure!(
            &actual_active_sha256 == expected_active,
            "Active .ctbnbdl bundle SHA-256 inventory in {} does not match BATCH_COMMITTED: expected {:?}, found {:?}",
            graph_dir.display(),
            expected_active,
            actual_active_sha256
        );
    }
    if let Some(expected_history) = expected_history_sha256 {
        ensure!(
            &actual_history_sha256 == expected_history,
            "Historical .ctbnbdl bundle SHA-256 inventory in {} does not match BATCH_COMMITTED: expected {:?}, found {:?}",
            history_dir.display(),
            expected_history,
            actual_history_sha256
        );
    }

    let decoded_active = decode_bundle_dir_nodes(graph_dir, framing)?;
    let decoded_history = decode_bundle_dir_nodes(&history_dir, framing)?;

    let active_ids: HashSet<u128> =
        decoded_active.iter().map(|n| n.id).collect();
    let history_ids: HashSet<u128> =
        decoded_history.iter().map(|n| n.id).collect();
    ensure!(
        active_ids.is_disjoint(&history_ids),
        "Active and historical .ctbnbdl bundles in {} are not disjoint!",
        graph_dir.display()
    );

    let (expected_active, expected_history) =
        partition_active_and_inactive_nodes_with(committed_nodes, framing);

fn project_node_id_tokens(
    slice: &[LedgerNode],
) -> Vec<(u128, &[crate::shorthand::DcShorthand])> {
    slice
        .iter()
        .map(|n| (n.id, n.tokens.as_slice()))
        .collect()
}

    let decoded_active_pairs = project_node_id_tokens(&decoded_active);
    let expected_active_pairs = project_node_id_tokens(&expected_active);
    ensure!(
        decoded_active_pairs == expected_active_pairs,
        "Decoded active .ctbnbdl nodes ({} nodes) in {} do not match expected active partition ({} nodes) by (id, tokens)",
        decoded_active_pairs.len(),
        graph_dir.display(),
        expected_active_pairs.len()
    );

    let decoded_history_pairs = project_node_id_tokens(&decoded_history);
    let expected_history_pairs = project_node_id_tokens(&expected_history);
    ensure!(
        decoded_history_pairs == expected_history_pairs,
        "Decoded historical .ctbnbdl nodes ({} nodes) in {} do not match expected inactive partition ({} nodes) by (id, tokens)",
        decoded_history_pairs.len(),
        history_dir.display(),
        expected_history_pairs.len()
    );

    let mut merged_decoded: Vec<(u128, &[crate::shorthand::DcShorthand])> =
        Vec::with_capacity(
            decoded_active_pairs
                .len()
                .saturating_add(decoded_history_pairs.len()),
        );
    merged_decoded.extend(decoded_active_pairs);
    merged_decoded.extend(decoded_history_pairs);
    merged_decoded.sort_by_key(|(id, _)| *id);

    let committed_pairs = project_node_id_tokens(committed_nodes);
    ensure!(
        merged_decoded == committed_pairs,
        "Decoded active + history .ctbnbdl union ({} nodes) does not equal committed .ctbnodes ledger ({} nodes) by (id, tokens)",
        merged_decoded.len(),
        committed_pairs.len()
    );

    Ok(())
}

/// Loads `.ctbnodes` directly under an already-held `LedgerWriteGuard` without opening a second lock fd
/// or running external reader-gate checks while `BATCH_PENDING` is open.
pub fn load_ledger_nodes_from_dir_with_write_guard(
    _write_guard: &LedgerWriteGuard,
    graph_dir: &Path,
) -> Result<Vec<LedgerNode>> {
    load_ledger_nodes_from_dir_raw(graph_dir)
}

/// Compiles `nodes` into active and historical `.ctbnbdl` bundles under an already-held `LedgerWriteGuard`
/// directly from the validated in-memory `nodes` slice, `fsync`s all bundle files and directories,
/// updates `built/` staging directories if present, verifies the compiled bundles via
/// `verify_compiled_bundles_match_ledger`, and returns `(active_bundles_sha256, history_bundles_sha256)`.
pub fn compile_nodes_slice_to_ctbnbdl_with_write_guard(
    write_guard: &LedgerWriteGuard,
    nodes: &[LedgerNode],
    graph_dir: &Path,
    framing: CtbnbdlFramingDcs,
) -> Result<(BTreeMap<String, String>, BTreeMap<String, String>)> {
    let (active_nodes, inactive_nodes) =
        partition_active_and_inactive_nodes_with(nodes, framing);

    let active_bundles = encode_nodes_to_ctbnbdl_bundles_with_limits(
        &active_nodes,
        framing,
        MAX_NODES_PER_BUNDLE,
        MAX_BYTES_PER_BUNDLE,
    )?;
    write_ctbnbdl_bundles_to_dir(graph_dir, &active_bundles, false)?;
    for (first_id, _) in &active_bundles {
        let filename = format_chunk_bundle_filename(*first_id);
        fsync_file(&graph_dir.join(filename))?;
    }
    fsync_dir(graph_dir)?;

    let inactive_bundles = encode_nodes_to_ctbnbdl_bundles_with_limits(
        &inactive_nodes,
        framing,
        MAX_NODES_PER_BUNDLE,
        MAX_BYTES_PER_BUNDLE,
    )?;
    let system_history_dir = graph_dir.join("system");
    write_ctbnbdl_bundles_to_dir(&system_history_dir, &inactive_bundles, true)?;
    for (first_id, _) in &inactive_bundles {
        let filename = format_chunk_bundle_filename(*first_id);
        fsync_file(&system_history_dir.join(filename))?;
    }
    fsync_dir(&system_history_dir)?;

    let runtime_graph_dir =
        write_guard.repo_root().join("built/assets/data/graph");
    if runtime_graph_dir.is_dir() {
        write_ctbnbdl_bundles_to_dir(&runtime_graph_dir, &active_bundles, true)?;
    }
    let staged_history_dir = write_guard
        .repo_root()
        .join("built/system_graph_history_assets/data/graph/system");
    if staged_history_dir.is_dir() {
        write_ctbnbdl_bundles_to_dir(
            &staged_history_dir,
            &inactive_bundles,
            true,
        )?;
    }

    let active_hashes = compute_bundle_dir_sha256s(graph_dir)?;
    let history_hashes = compute_bundle_dir_sha256s(&system_history_dir)?;
    verify_compiled_bundles_match_ledger(
        graph_dir,
        nodes,
        framing,
        Some(&active_hashes),
        Some(&history_hashes),
    )?;
    Ok((active_hashes, history_hashes))
}

/// Parses and validates `ledger-journal.jsonl`:
/// - Any complete (`\n`-terminated) line that fails JSON parsing or `verify_sha256()` is rejected as hard corruption.
/// - Only an unterminated final line (not ending in `\n`) is reported in `torn_trailing_fragment`.
pub fn inspect_ledger_journal(journal_path: &Path) -> Result<ParsedJournal> {
    let raw = fs::read(journal_path).with_context(|| {
        format!("Failed to read ledger journal {}", journal_path.display())
    })?;

    let mut valid_prefix_len = 0usize;
    let mut torn_trailing_fragment: Option<Vec<u8>> = None;
    let mut records: Vec<JournalRecord> = Vec::new();

    for line_bytes in raw.split_inclusive(|&byte| byte == b'\n') {
        let offset = valid_prefix_len;
        if line_bytes.last() == Some(&b'\n') {
            let line_str = std::str::from_utf8(line_bytes)
                .with_context(|| {
                    format!(
                        "CRITICAL: Invalid UTF-8 in complete journal line in {} at byte offset {offset}",
                        journal_path.display()
                    )
                })?
                .trim_end_matches(['\r', '\n']);
            if !line_str.trim().is_empty() {
                let rec: JournalRecord = serde_json::from_str(line_str)
                    .with_context(|| {
                        format!(
                            "CRITICAL: Corrupted complete journal line in {} at byte offset {offset}",
                            journal_path.display()
                        )
                    })?;
                rec.verify_sha256().with_context(|| {
                    format!(
                        "CRITICAL: Checksum failure in complete journal record in {} at byte offset {offset}",
                        journal_path.display()
                    )
                })?;
                records.push(rec);
            }
            valid_prefix_len = valid_prefix_len.saturating_add(line_bytes.len());
        } else {
            if !line_bytes.iter().all(u8::is_ascii_whitespace) {
                torn_trailing_fragment = Some(line_bytes.to_vec());
            }
        }
    }

    let mut committed_pairs: Vec<(JournalRecord, JournalRecord)> = Vec::new();
    let mut open_pending: Option<JournalRecord> = None;
    let mut last_committed: Option<JournalRecord> = None;

    for (idx, rec) in records.iter().enumerate() {
        match rec.state.as_str() {
            "BATCH_PENDING" => {
                ensure!(
                    open_pending.is_none(),
                    "CRITICAL: Journal {} has consecutive BATCH_PENDING records without an intervening BATCH_COMMITTED (record index {idx}, batch_id={})",
                    journal_path.display(),
                    rec.batch_id
                );
                open_pending = Some(rec.clone());
            }
            "BATCH_COMMITTED" => {
                let Some(pending) = open_pending.take() else {
                    bail!(
                        "CRITICAL: Journal {} has orphan BATCH_COMMITTED at record index {idx} (batch_id={}) without matching BATCH_PENDING",
                        journal_path.display(),
                        rec.batch_id
                    );
                };
                ensure!(
                    pending.batch_id == rec.batch_id
                        && pending.kind == rec.kind
                        && pending.baseline_last_id == rec.baseline_last_id
                        && pending.baseline_ledger_sha256
                            == rec.baseline_ledger_sha256
                        && pending.final_last_id == rec.final_last_id
                        && pending.final_ledger_sha256 == rec.final_ledger_sha256
                        && pending.manifest_sha256 == rec.manifest_sha256,
                    "CRITICAL: BATCH_COMMITTED (batch_id={}) does not match preceding BATCH_PENDING (batch_id={}) in {}",
                    rec.batch_id,
                    pending.batch_id,
                    journal_path.display()
                );
                last_committed = Some(rec.clone());
                committed_pairs.push((pending, rec.clone()));
            }
            other => {
                bail!(
                    "CRITICAL: Unknown journal record state {:?} at index {idx} in {}",
                    other,
                    journal_path.display()
                );
            }
        }
    }

    Ok(ParsedJournal {
        valid_prefix_len,
        torn_trailing_fragment,
        records,
        committed_pairs,
        open_pending,
        last_committed,
    })
}

/// Truncates an unterminated trailing line from `journal_path` at `valid_prefix_len` and `fsync`s both file and parent directory.
pub fn repair_torn_journal_suffix(
    journal_path: &Path,
    valid_prefix_len: usize,
) -> Result<()> {
    let raw = fs::read(journal_path).with_context(|| {
        format!("Failed to read {} for torn-suffix repair", journal_path.display())
    })?;
    let prefix = raw.get(..valid_prefix_len).context(
        "valid_prefix_len exceeds journal file length during torn-suffix repair",
    )?;
    let tmp_path = journal_path.with_extension("jsonl.repair.tmp");
    {
        let mut f = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&tmp_path)
            .with_context(|| {
                format!("Failed to create {}", tmp_path.display())
            })?;
        f.write_all(prefix)?;
        f.sync_all()?;
    }
    fs::rename(&tmp_path, journal_path).with_context(|| {
        format!(
            "Failed to replace {} with repaired journal {}",
            journal_path.display(),
            tmp_path.display()
        )
    })?;
    fsync_file(journal_path)?;
    if let Some(parent) = journal_path.parent() {
        fsync_dir(parent)?;
    }
    Ok(())
}

fn append_journal_record(
    journal_path: &Path,
    record: &JournalRecord,
    fault: Option<FaultInjectionPoint>,
) -> Result<()> {
    let record = record.clone().with_computed_sha256()?;
    let mut line = serde_json::to_string(&record)
        .context("Failed to serialize JournalRecord")?;
    line.push('\n');

    if let Some(parent) = journal_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!("Failed to create {}", parent.display())
        })?;
    }

    let torn_len = match (record.state.as_str(), fault) {
        (
            "BATCH_PENDING",
            Some(FaultInjectionPoint::TornPendingRecord { bytes_to_keep }),
        )
        | (
            "BATCH_COMMITTED",
            Some(FaultInjectionPoint::TornCommittedRecord { bytes_to_keep }),
        ) => Some(bytes_to_keep),
        _ => None,
    };

    if let Some(keep) = torn_len {
        let clamped = keep.min(line.len().saturating_sub(1));
        let slice = line.as_bytes().get(..clamped).unwrap_or_default();
        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(journal_path)
            .with_context(|| {
                format!("Failed to open {} for append", journal_path.display())
            })?;
        f.write_all(slice)?;
        f.sync_all()?;
        bail!(
            "Simulated fault injection: wrote torn {} record ({} bytes without newline)",
            record.state,
            slice.len()
        );
    }

    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(journal_path)
        .with_context(|| {
            format!("Failed to open {} for append", journal_path.display())
        })?;
    f.write_all(line.as_bytes())?;
    f.sync_all()?;
    if let Some(parent) = journal_path.parent() {
        fsync_dir(parent)?;
    }
    Ok(())
}

fn stamp_journal_initialized_marker(repo_root: &Path) -> Result<()> {
    let marker_path = repo_root.join(JOURNAL_INITIALIZED_REL_PATH);
    if let Some(parent) = marker_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!("Failed to create {}", parent.display())
        })?;
    }
    if !marker_path.is_file() {
        let mut f = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&marker_path)
            .with_context(|| {
                format!("Failed to write {}", marker_path.display())
            })?;
        f.write_all(b"initialized\n")?;
        f.sync_all()?;
        if let Some(parent) = marker_path.parent() {
            fsync_dir(parent)?;
        }
    }
    Ok(())
}

/// Loads all complete `.ctbnodes` nodes from `dir`, allowing at most a torn trailing line at the end
/// of the final chunk file when recovering an interrupted `BATCH_PENDING` transaction.
fn load_nodes_from_dir_allowing_torn_tail(
    dir: &Path,
) -> Result<(Vec<LedgerNode>, Option<(PathBuf, Vec<u8>)>)> {
    let files = list_chunk_files(dir)?;
    let mut all_nodes = Vec::new();
    let mut torn_tail: Option<(PathBuf, Vec<u8>)> = None;
    let total_files = files.len();

    for (idx, path) in files.into_iter().enumerate() {
        let is_last_file = idx.saturating_add(1) == total_files;
        let raw = fs::read(&path).with_context(|| {
            format!("Failed to read .ctbnodes chunk {}", path.display())
        })?;
        let complete_len = if is_last_file && !raw.is_empty() && raw.last() != Some(&b'\n') {
            let complete_len = raw.iter().rposition(|&byte| byte == b'\n')
                .map_or(0, |last_nl| last_nl.saturating_add(1));
            let tail = raw.get(complete_len..)
                .context("Invalid complete prefix length in .ctbnodes chunk")?;
            torn_tail = Some((path.clone(), tail.to_vec()));
            complete_len
        } else {
            ensure!(
                raw.is_empty() || raw.last() == Some(&b'\n'),
                "CRITICAL: Non-final .ctbnodes chunk {} has an unterminated line!",
                path.display()
            );
            raw.len()
        };
        let complete_bytes = raw.get(..complete_len)
            .context("Invalid complete prefix length in .ctbnodes chunk")?;
        let text_to_parse = std::str::from_utf8(complete_bytes)
            .with_context(|| {
                format!(
                    "CRITICAL: Invalid UTF-8 in complete .ctbnodes lines in {}",
                    path.display()
                )
            })?;

        let chunk_nodes =
            load_ledger_nodes_from_str(text_to_parse, &path.display().to_string())?;
        all_nodes.extend(chunk_nodes);
    }

    Ok((all_nodes, torn_tail))
}

fn verify_and_extract_baseline_for_pending(
    dir: &Path,
    label: &str,
    pending: &JournalRecord,
) -> Result<Vec<LedgerNode>> {
    let (disk_nodes, torn_tail) = load_nodes_from_dir_allowing_torn_tail(dir)?;
    let baseline_count = disk_nodes
        .iter()
        .take_while(|n| n.id <= pending.baseline_last_id)
        .count();
    let baseline_nodes: Vec<LedgerNode> = disk_nodes
        .get(..baseline_count)
        .unwrap_or_default()
        .to_vec();
    let appended_nodes: &[LedgerNode] =
        disk_nodes.get(baseline_count..).unwrap_or_default();

    let actual_baseline_sha = compute_nodes_slice_ledger_sha256(&baseline_nodes);
    ensure!(
        actual_baseline_sha == pending.baseline_ledger_sha256,
        "CRITICAL: {label} ({}) baseline prefix (<= l{}) SHA-256 {} does not match BATCH_PENDING.baseline_ledger_sha256 {}",
        dir.display(),
        pending.baseline_last_id,
        actual_baseline_sha,
        pending.baseline_ledger_sha256
    );

    ensure!(
        appended_nodes.len() <= pending.items.len(),
        "CRITICAL: {label} ({}) has {} nodes beyond baseline l{}, exceeding BATCH_PENDING.items ({} items)",
        dir.display(),
        appended_nodes.len(),
        pending.baseline_last_id,
        pending.items.len()
    );

    for (idx, node) in appended_nodes.iter().enumerate() {
        let Some(expected_item) = pending.items.get(idx) else {
            bail!("Unexpected appended node index {idx}");
        };
        let actual_line = node.format_line();
        ensure!(
            node.id == expected_item.node_id && actual_line == expected_item.raw_line,
            "CRITICAL: {label} ({}) appended node at offset {idx} (l{}: {:?}) does not match BATCH_PENDING item (l{}: {:?})",
            dir.display(),
            node.id,
            actual_line,
            expected_item.node_id,
            expected_item.raw_line
        );
    }

    if let Some((torn_path, frag)) = torn_tail {
        let next_idx = appended_nodes.len();
        let Some(next_item) = pending.items.get(next_idx) else {
            bail!(
                "CRITICAL: {label} ({}) has trailing fragment {:?} after all {} BATCH_PENDING items!",
                torn_path.display(),
                frag,
                pending.items.len()
            );
        };
        let expected_full_line = format!("{}\n", next_item.raw_line);
        ensure!(
            expected_full_line.as_bytes().starts_with(&frag),
            "CRITICAL: {label} ({}) trailing fragment {:?} is not a strict byte prefix of expected item line {:?}",
            torn_path.display(),
            frag,
            expected_full_line
        );
    }

    Ok(baseline_nodes)
}

fn parse_planned_items_to_nodes(
    items: &[PlannedLedgerItem],
) -> Result<Vec<LedgerNode>> {
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        let parsed = load_ledger_nodes_from_str(
            &format!("{}\n", item.raw_line),
            &item.target_chunk,
        )?;
        let Some(node) = parsed.into_iter().next() else {
            bail!(
                "Planned item for l{} produced no LedgerNode from raw_line {:?}",
                item.node_id,
                item.raw_line
            );
        };
        ensure!(
            node.id == item.node_id,
            "Planned item node_id l{} does not match parsed raw_line node l{}",
            item.node_id,
            node.id
        );
        out.push(node);
    }
    Ok(out)
}

fn append_single_node_to_ledger_dir(
    dir: &Path,
    all_nodes_through_current: &[LedgerNode],
    torn_bytes_to_keep: Option<usize>,
) -> Result<PathBuf> {
    let Some(last_node) = all_nodes_through_current.last() else {
        bail!("append_single_node_to_ledger_dir called with empty node slice");
    };
    let chunk_name = chunk_filename_for_node_id(last_node.id);
    let Some(chunk_start_id) = parse_chunk_filename_start_id(&chunk_name) else {
        bail!("Invalid chunk filename {chunk_name}");
    };
    let chunk_end_id = chunk_start_id
        .saturating_add(u128::try_from(MAX_NODES_PER_CHUNK).unwrap_or(100));

    let mut content = String::new();
    for node in all_nodes_through_current {
        if node.id >= chunk_start_id && node.id < chunk_end_id {
            if node.id == last_node.id
                && let Some(keep) = torn_bytes_to_keep
            {
                let line = node.format_line();
                let clamped = keep.min(line.len());
                content.push_str(line.get(..clamped).unwrap_or_default());
            } else {
                content.push_str(&node.format_line());
                content.push('\n');
            }
        }
    }
    let chunk_path = dir.join(&chunk_name);
    write_chunk_file_append_only(&chunk_path, &content)?;
    Ok(chunk_path)
}

fn unprotect_shadow_tail_for_append(shadow_dir: &Path, target_chunk: &str) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let chunk_path = shadow_dir.join(target_chunk);
        if chunk_path.exists() {
            let _ = fs::set_permissions(
                &chunk_path,
                fs::Permissions::from_mode(0o644),
            );
        }
    }
}

fn reprotect_shadow_chunks(shadow_dir: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(files) = list_chunk_files(shadow_dir) {
            for file in files {
                let _ =
                    fs::set_permissions(&file, fs::Permissions::from_mode(0o444));
            }
        }
    }
}

/// Executes standalone recovery of any open `BATCH_PENDING` record or torn journal line in
/// `<repo_root>/src/formats/dcdata/data-src/nodes/allocated/ledger-journal.jsonl`.
///
/// This function lives in `ctb_build_support` so it can be built and invoked via
/// `ctb-lint recover-ledger-journal` even when `ctb-formats-dcdata/build.rs` is blocked by an
/// uncommitted `BATCH_PENDING` transaction.
pub fn recover_pending_ledger_journal(
    repo_root: &Path,
    required_manifest_path: Option<&Path>,
    dry_run: bool,
) -> Result<bool> {
    let write_guard = LedgerWriteGuard::acquire(repo_root)?;
    let loaded_manifest = if let Some(mpath) = required_manifest_path {
        let bytes = fs::read(mpath).with_context(|| {
            format!("Failed to read manifest {}", mpath.display())
        })?;
        let sha = sha256_bytes_hex(&bytes);
        let manifest: TransactionManifest = serde_json::from_slice(&bytes)
            .with_context(|| {
                format!("Failed to parse manifest {}", mpath.display())
            })?;
        Some((manifest, sha))
    } else {
        None
    };
    let manifest_ref = loaded_manifest
        .as_ref()
        .map(|(m, s)| (m, s.as_str()));
    recover_pending_ledger_journal_with_guard(
        &write_guard,
        repo_root,
        manifest_ref,
        dry_run,
        None,
    )
}

/// Internal recovery implementation operating under an already-held [`LedgerWriteGuard`].
pub fn recover_pending_ledger_journal_with_guard(
    write_guard: &LedgerWriteGuard,
    repo_root: &Path,
    required_manifest: Option<(&TransactionManifest, &str)>,
    dry_run: bool,
    fault: Option<FaultInjectionPoint>,
) -> Result<bool> {
    let journal_path = repo_root.join(LEDGER_JOURNAL_REL_PATH);
    if !journal_path.exists() {
        return Ok(false);
    }

    let mut parsed = inspect_ledger_journal(&journal_path)?;
    if parsed.torn_trailing_fragment.is_some() {
        if dry_run {
            return Ok(true);
        }
        repair_torn_journal_suffix(&journal_path, parsed.valid_prefix_len)?;
        parsed = inspect_ledger_journal(&journal_path)?;
    }

    let Some(pending) = parsed.open_pending else {
        return Ok(false);
    };

    if let Some((manifest, manifest_sha)) = required_manifest {
        ensure!(
            pending.manifest_sha256.as_deref() == Some(manifest_sha),
            "CRITICAL: Open BATCH_PENDING manifest_sha256 {:?} does not match required manifest SHA-256 {}",
            pending.manifest_sha256,
            manifest_sha
        );
        let pending_norm: Vec<PlannedLedgerItem> = pending
            .items
            .iter()
            .map(PlannedLedgerItem::normalized_for_journal)
            .collect();
        let manifest_norm: Vec<PlannedLedgerItem> = manifest
            .planned_items
            .iter()
            .map(PlannedLedgerItem::normalized_for_journal)
            .collect();
        ensure!(
            pending.kind == manifest.kind
                && pending.baseline_last_id == manifest.baseline_last_id
                && pending.baseline_ledger_sha256
                    == manifest.baseline_ledger_sha256
                && pending.final_last_id == manifest.final_last_id
                && pending.final_ledger_sha256 == manifest.final_ledger_sha256
                && pending_norm == manifest_norm,
            "CRITICAL: Open BATCH_PENDING contents do not match required manifest {}",
            manifest.manifest_id
        );
    }

    let graph_dir = repo_root.join("assets/data/graph");
    let inbox_dir = repo_root.join("src/formats/dcdata/data-src/nodes/inbox");
    let allocated_dir =
        repo_root.join("src/formats/dcdata/data-src/nodes/allocated");

    let baseline_nodes = verify_and_extract_baseline_for_pending(
        &graph_dir,
        "primary ledger",
        &pending,
    )?;

    for (label, shadow_dir) in highwater_shadow_dirs(repo_root) {
        if shadow_dir.is_dir() && !list_chunk_files(&shadow_dir)?.is_empty() {
            verify_and_extract_baseline_for_pending(
                &shadow_dir,
                &label,
                &pending,
            )?;
        }
    }

    // Validate all inbox/allocated source files for inbox_allocation items.
    for item in &pending.items {
        if let (Some(inbox_name), Some(alloc_name), Some(expected_sha)) = (
            item.inbox_name.as_deref(),
            item.allocated_name.as_deref(),
            item.inbox_sha256.as_deref(),
        ) {
            let inbox_path = inbox_dir.join(inbox_name);
            let alloc_path = allocated_dir.join(alloc_name);
            let in_inbox = inbox_path.is_file();
            let in_alloc = alloc_path.is_file();
            ensure!(
                in_inbox || in_alloc,
                "CRITICAL: Source .dcsh file for pending node l{} is missing from both {} and {}",
                item.node_id,
                inbox_path.display(),
                alloc_path.display()
            );
            if in_inbox {
                let actual_sha = sha256_file_hex(&inbox_path)?;
                ensure!(
                    actual_sha == expected_sha,
                    "CRITICAL: Inbox file {} SHA-256 {} does not match BATCH_PENDING inbox_sha256 {}",
                    inbox_path.display(),
                    actual_sha,
                    expected_sha
                );
            }
            if in_alloc {
                let actual_sha = sha256_file_hex(&alloc_path)?;
                ensure!(
                    actual_sha == expected_sha,
                    "CRITICAL: Allocated file {} SHA-256 {} does not match BATCH_PENDING inbox_sha256 {}",
                    alloc_path.display(),
                    actual_sha,
                    expected_sha
                );
            }
        }
    }

    if dry_run {
        return Ok(true);
    }

    let planned_nodes = parse_planned_items_to_nodes(&pending.items)?;
    let mut progressive_nodes = baseline_nodes;
    let shadow_dirs = highwater_shadow_dirs(repo_root);
    let mut touched_primary_chunks: BTreeMap<String, PathBuf> = BTreeMap::new();
    let mut touched_shadow_chunks: Vec<PathBuf> = Vec::new();
    let mut touched_allocated_files: Vec<PathBuf> = Vec::new();

    for (idx, (item, node)) in
        pending.items.iter().zip(planned_nodes.into_iter()).enumerate()
    {
        progressive_nodes.push(node);

        let torn_keep = match fault {
            Some(FaultInjectionPoint::TornChunkLine {
                item_index,
                bytes_to_keep,
            }) if item_index == idx => Some(bytes_to_keep),
            _ => None,
        };

        let primary_chunk = append_single_node_to_ledger_dir(
            &graph_dir,
            &progressive_nodes,
            torn_keep,
        )?;
        if torn_keep.is_some() {
            fsync_file(&primary_chunk)?;
            bail!(
                "Simulated fault injection: wrote torn .ctbnodes line for item {idx} (l{})",
                item.node_id
            );
        }
        touched_primary_chunks
            .insert(item.target_chunk.clone(), primary_chunk.clone());

        for (label, shadow_dir) in &shadow_dirs {
            fs::create_dir_all(shadow_dir).with_context(|| {
                format!("Failed to create {label} at {}", shadow_dir.display())
            })?;
            unprotect_shadow_tail_for_append(shadow_dir, &item.target_chunk);
            let shadow_chunk = append_single_node_to_ledger_dir(
                shadow_dir,
                &progressive_nodes,
                None,
            )?;
            touched_shadow_chunks.push(shadow_chunk);
        }

        if let (Some(inbox_name), Some(alloc_name)) =
            (item.inbox_name.as_deref(), item.allocated_name.as_deref())
        {
            fs::create_dir_all(&allocated_dir)?;
            let inbox_path = inbox_dir.join(inbox_name);
            let alloc_path = allocated_dir.join(alloc_name);
            if inbox_path.is_file() {
                fs::rename(&inbox_path, &alloc_path).with_context(|| {
                    format!(
                        "Failed to move {} to {}",
                        inbox_path.display(),
                        alloc_path.display()
                    )
                })?;
            }
            touched_allocated_files.push(alloc_path);
        }

        if fault != Some(FaultInjectionPoint::BeforeFsync) {
            fsync_file(&primary_chunk)?;
            for (_, shadow_dir) in &shadow_dirs {
                let sc = shadow_dir.join(&item.target_chunk);
                if sc.is_file() {
                    fsync_file(&sc)?;
                }
            }
            if let Some(alloc_name) = item.allocated_name.as_deref() {
                let ap = allocated_dir.join(alloc_name);
                if ap.is_file() {
                    fsync_file(&ap)?;
                }
            }
        }

        if fault
            == Some(FaultInjectionPoint::AfterItemAppendedAndMoved {
                item_index: idx,
            })
        {
            bail!(
                "Simulated fault injection: crash after item {idx} (l{}) appended and moved",
                item.node_id
            );
        }
    }

    if fault == Some(FaultInjectionPoint::BeforeFsync) {
        bail!("Simulated fault injection: crash after all writes/renames before fsync");
    }

    // Unconditionally fsync all touched primary chunks, shadow chunks, allocated files, and directories.
    for path in touched_primary_chunks.values() {
        fsync_file(path)?;
    }
    fsync_dir(&graph_dir)?;

    for path in &touched_shadow_chunks {
        if path.is_file() {
            fsync_file(path)?;
        }
    }
    for (_, shadow_dir) in &shadow_dirs {
        reprotect_shadow_chunks(shadow_dir);
        fsync_dir(shadow_dir)?;
    }

    for path in &touched_allocated_files {
        if path.is_file() {
            fsync_file(path)?;
        }
    }
    fsync_dir(&inbox_dir)?;
    fsync_dir(&allocated_dir)?;

    let actual_final_sha = compute_ledger_dir_sha256(&graph_dir)?;
    ensure!(
        actual_final_sha == pending.final_ledger_sha256,
        "CRITICAL: Final ledger SHA-256 {} after completing batch {} does not match expected {}",
        actual_final_sha,
        pending.batch_id,
        pending.final_ledger_sha256
    );
    let actual_last_id = progressive_nodes.last().map(|n| n.id).unwrap_or(0);
    ensure!(
        actual_last_id == pending.final_last_id,
        "CRITICAL: Final node ID l{} after completing batch {} does not match expected l{}",
        actual_last_id,
        pending.batch_id,
        pending.final_last_id
    );

    let framing = resolve_ctbnbdl_framing_dcs_from_nodes(
        &progressive_nodes,
    )?;
    let (active_bundles_sha256, history_bundles_sha256) =
        compile_nodes_slice_to_ctbnbdl_with_write_guard(
            write_guard,
            &progressive_nodes,
            &graph_dir,
            framing,
        )?;

    let committed = JournalRecord {
        state: "BATCH_COMMITTED".to_string(),
        batch_id: pending.batch_id,
        kind: pending.kind,
        baseline_last_id: pending.baseline_last_id,
        baseline_ledger_sha256: pending.baseline_ledger_sha256,
        final_last_id: pending.final_last_id,
        final_ledger_sha256: pending.final_ledger_sha256,
        manifest_sha256: pending.manifest_sha256,
        items: Vec::new(),
        active_bundles_sha256,
        history_bundles_sha256,
        record_sha256: String::new(),
    };
    append_journal_record(&journal_path, &committed, fault)?;
    stamp_journal_initialized_marker(repo_root)?;
    Ok(true)
}

/// External reader/packager gate: verifies under `LedgerReadGuard` (`LOCK_SH`) that:
/// 1. `ledger-journal.jsonl` exists on disk (never trusting a missing journal),
/// 2. `ledger-journal.jsonl` has no torn trailing line and no open `BATCH_PENDING` transaction,
/// 3. `ledger-journal.jsonl` ends with a verified `BATCH_COMMITTED` record whose `final_ledger_sha256`
///    and `final_last_id` match `assets/data/graph/*.ctbnodes`, and
/// 4. `assets/data/graph/*.ctbnbdl` and `assets/data/graph/system/*.ctbnbdl` match `BATCH_COMMITTED`'s
///    recorded bundle SHA-256 inventory and pass full `(node.id, &node.tokens)` union and partition verification.
pub fn ensure_ledger_journal_clean(
    repo_root: &Path,
    _read_guard: &LedgerReadGuard,
) -> Result<()> {
    let journal_path = repo_root.join(LEDGER_JOURNAL_REL_PATH);
    let init_marker = repo_root.join(JOURNAL_INITIALIZED_REL_PATH);
    ensure!(
        journal_path.is_file(),
        "CRITICAL: Ledger journal {} is missing (initialized_marker={})! Gated readers refuse to trust an uninitialized or deleted journal.",
        journal_path.display(),
        init_marker.exists()
    );

    let parsed = inspect_ledger_journal(&journal_path)?;
    ensure!(
        parsed.torn_trailing_fragment.is_none(),
        "CRITICAL: Ledger journal {} has an unterminated trailing line; run `ctb-lint recover-ledger-journal` before reading or building.",
        journal_path.display()
    );
    ensure!(
        parsed.open_pending.is_none(),
        "CRITICAL: Ledger journal {} has an uncommitted BATCH_PENDING transaction ({:?}); run `ctb-lint recover-ledger-journal` before reading or building.",
        journal_path.display(),
        parsed.open_pending.as_ref().map(|p| p.batch_id.as_str())
    );
    let Some(last_committed) = parsed.last_committed else {
        bail!(
            "CRITICAL: Ledger journal {} contains no BATCH_COMMITTED records!",
            journal_path.display()
        );
    };

    let graph_dir = repo_root.join("assets/data/graph");
    let actual_ledger_sha = compute_ledger_dir_sha256(&graph_dir)?;
    ensure!(
        actual_ledger_sha == last_committed.final_ledger_sha256,
        "CRITICAL: On-disk .ctbnodes ledger SHA-256 {} in {} does not match latest BATCH_COMMITTED ({}) final_ledger_sha256 {}",
        actual_ledger_sha,
        graph_dir.display(),
        last_committed.batch_id,
        last_committed.final_ledger_sha256
    );

    let committed_nodes = load_ledger_nodes_from_dir_raw(&graph_dir)?;
    let actual_last_id = committed_nodes.last().map(|n| n.id).unwrap_or(0);
    ensure!(
        actual_last_id == last_committed.final_last_id,
        "CRITICAL: On-disk .ctbnodes highest node ID l{} does not match latest BATCH_COMMITTED ({}) final_last_id l{}",
        actual_last_id,
        last_committed.batch_id,
        last_committed.final_last_id
    );

    let framing = resolve_ctbnbdl_framing_dcs_from_nodes(
        &committed_nodes,
    )?;
    verify_compiled_bundles_match_ledger(
        &graph_dir,
        &committed_nodes,
        framing,
        Some(&last_committed.active_bundles_sha256),
        Some(&last_committed.history_bundles_sha256),
    )?;

    Ok(())
}

/// Output returned by [`execute_durable_ledger_transaction_with_planner`].
#[derive(Debug, Clone)]
pub struct DurableTransactionOutcome<T> {
    /// Whether an existing open `BATCH_PENDING` or already-committed manifest was resumed/reused.
    pub resumed_or_already_committed: bool,
    /// Number of `.ctbnodes` chunk files touched.
    pub files_written: usize,
    /// Number of nodes appended in the transaction.
    pub appended_count: usize,
    /// Final total node count in the ledger.
    pub total_nodes: usize,
    /// Custom planner payload (when the Fresh-Start planner ran).
    pub planner_payload: Option<T>,
}

/// Shared durable transactional writer used by both `process_inbox_nodes` and `update_system_graph_ledger`.
///
/// Follows the strict recovery-first, manifest-enforced protocol under `LedgerWriteGuard` (`LOCK_EX`):
/// 1. Repairs any torn journal suffix and resumes any open `BATCH_PENDING` record **before** calling
///    committed-only provenance validation (`planner_fn`).
/// 2. If `--require-manifest` is supplied and the newly recovered (or already committed) `BATCH_COMMITTED`
///    record matches `manifest_sha256`, verifies the final ledger and `.ctbnbdl` state and returns idempotently.
/// 3. Otherwise (Fresh-Start path), verifies baseline ledger SHA-256 and pinned input SHA-256s, runs
///    `planner_fn` on the committed baseline nodes and `committed_pairs`, verifies byte-for-byte
///    agreement with `--require-manifest` (if supplied), writes `BATCH_PENDING` (when `!dry_run` and
///    either `!planned_items.is_empty()` or `required_manifest.is_some()`), and executes Phase 1 + Phase 2 +
///    `BATCH_COMMITTED` via `recover_pending_ledger_journal_with_guard`.
pub fn execute_durable_ledger_transaction_with_planner<T, F>(
    repo_root: &Path,
    kind: &str,
    batch_id_prefix: &str,
    required_manifest_path: Option<&Path>,
    dry_run: bool,
    fault: Option<FaultInjectionPoint>,
    planner_fn: F,
) -> Result<DurableTransactionOutcome<T>>
where
    F: FnOnce(
        &LedgerWriteGuard,
        &[LedgerNode],
        &[(JournalRecord, JournalRecord)],
    ) -> Result<(
        Vec<PlannedLedgerItem>,
        BTreeMap<String, String>,
        T,
    )>,
{
    let write_guard = LedgerWriteGuard::acquire(repo_root)?;
    let journal_path = repo_root.join(LEDGER_JOURNAL_REL_PATH);
    ensure!(
        journal_path.is_file(),
        "CRITICAL: Ledger journal {} is missing!",
        journal_path.display()
    );

    let loaded_manifest = if let Some(mpath) = required_manifest_path {
        let bytes = fs::read(mpath).with_context(|| {
            format!("Failed to read required manifest {}", mpath.display())
        })?;
        let sha = sha256_bytes_hex(&bytes);
        let manifest: TransactionManifest = serde_json::from_slice(&bytes)
            .with_context(|| {
                format!("Failed to parse required manifest {}", mpath.display())
            })?;
        ensure!(
            manifest.kind == kind,
            "Required manifest {} has kind {:?}, expected {:?}",
            mpath.display(),
            manifest.kind,
            kind
        );
        Some((manifest, sha))
    } else {
        None
    };
    let manifest_ref = loaded_manifest
        .as_ref()
        .map(|(m, s)| (m, s.as_str()));

    // Step 1: Pending-first recovery BEFORE committed-only provenance validation!
    let recovered_pending = recover_pending_ledger_journal_with_guard(
        &write_guard,
        repo_root,
        manifest_ref,
        dry_run,
        fault,
    )?;

    let parsed = inspect_ledger_journal(&journal_path)?;
    if dry_run && recovered_pending {
        let pending_items = parsed
            .open_pending
            .as_ref()
            .map_or(0, |p| p.items.len());
        return Ok(DurableTransactionOutcome {
            resumed_or_already_committed: true,
            files_written: 0,
            appended_count: pending_items,
            total_nodes: 0,
            planner_payload: None,
        });
    }

    // Step 2: If a required manifest was supplied and the latest BATCH_COMMITTED already matches
    // `manifest_sha256` (either because step 1 just finished resuming it or because it was already
    // committed in an earlier invocation), verify final state and return idempotently.
    let graph_dir = repo_root.join("assets/data/graph");
    if let Some((manifest, manifest_sha)) = manifest_ref {
        if let Some(last_committed) = &parsed.last_committed {
            if last_committed.manifest_sha256.as_deref() == Some(manifest_sha) {
                let current_nodes = load_ledger_nodes_from_dir_with_write_guard(
                    &write_guard,
                    &graph_dir,
                )?;
                let actual_sha = compute_ledger_dir_sha256(&graph_dir)?;
                ensure!(
                    actual_sha == manifest.final_ledger_sha256
                        && current_nodes.len() == manifest.final_node_count
                        && current_nodes.last().map(|n| n.id)
                            == Some(manifest.final_last_id),
                    "CRITICAL: Committed manifest {} state on disk does not match manifest final state",
                    manifest.manifest_id
                );
                let framing =
                    resolve_ctbnbdl_framing_dcs_from_nodes(
                        &current_nodes,
                    )?;
                verify_compiled_bundles_match_ledger(
                    &graph_dir,
                    &current_nodes,
                    framing,
                    Some(&last_committed.active_bundles_sha256),
                    Some(&last_committed.history_bundles_sha256),
                )?;
                let touched_chunks: HashSet<&str> = manifest
                    .planned_items
                    .iter()
                    .map(|i| i.target_chunk.as_str())
                    .collect();
                return Ok(DurableTransactionOutcome {
                    resumed_or_already_committed: true,
                    files_written: touched_chunks.len(),
                    appended_count: manifest.planned_items.len(),
                    total_nodes: current_nodes.len(),
                    planner_payload: None,
                });
            }
        }
    }

    // Step 3: Fresh-Start Path. Verify clean committed journal state.
    ensure!(
        parsed.open_pending.is_none() && parsed.torn_trailing_fragment.is_none(),
        "CRITICAL: Journal {} still has pending/torn state after recovery step",
        journal_path.display()
    );
    let Some(last_committed) = &parsed.last_committed else {
        bail!(
            "CRITICAL: Journal {} has no BATCH_COMMITTED baseline",
            journal_path.display()
        );
    };

    let baseline_nodes =
        load_ledger_nodes_from_dir_with_write_guard(&write_guard, &graph_dir)?;
    for (label, shadow_dir) in highwater_shadow_dirs(repo_root) {
        if shadow_dir.is_dir() {
            let shadow_nodes = load_ledger_nodes_from_dir_raw(&shadow_dir)?;
            if !shadow_nodes.is_empty() {
                verify_nodes_append_only_against_baseline(
                    &label,
                    &shadow_nodes,
                    &baseline_nodes,
                )?;
            }
        }
    }

    let baseline_last_id = baseline_nodes.last().map(|n| n.id).unwrap_or(0);
    let baseline_ledger_sha256 = compute_ledger_dir_sha256(&graph_dir)?;
    ensure!(
        baseline_last_id == last_committed.final_last_id
            && baseline_ledger_sha256 == last_committed.final_ledger_sha256,
        "CRITICAL: Current ledger baseline (last_id=l{}, sha256={}) does not match latest BATCH_COMMITTED (last_id=l{}, sha256={})",
        baseline_last_id,
        baseline_ledger_sha256,
        last_committed.final_last_id,
        last_committed.final_ledger_sha256
    );

    if let Some((manifest, _)) = manifest_ref {
        ensure!(
            baseline_last_id == manifest.baseline_last_id
                && baseline_nodes.len() == manifest.baseline_node_count
                && baseline_ledger_sha256 == manifest.baseline_ledger_sha256,
            "CRITICAL: Required manifest {} baseline mismatch: expected (last_id=l{}, count={}, sha256={}), found (last_id=l{}, count={}, sha256={})",
            manifest.manifest_id,
            manifest.baseline_last_id,
            manifest.baseline_node_count,
            manifest.baseline_ledger_sha256,
            baseline_last_id,
            baseline_nodes.len(),
            baseline_ledger_sha256
        );
    }

    let (planned_items, actual_input_sha256s, payload) = planner_fn(
        &write_guard,
        &baseline_nodes,
        &parsed.committed_pairs,
    )?;

    let planned_nodes = parse_planned_items_to_nodes(&planned_items)?;
    let mut updated_nodes = baseline_nodes.clone();
    updated_nodes.extend(planned_nodes);
    verify_nodes_append_only_against_baseline(
        "existing working-tree ledger",
        &baseline_nodes,
        &updated_nodes,
    )?;

    let final_last_id = updated_nodes.last().map(|n| n.id).unwrap_or(0);
    let final_ledger_sha256 = compute_nodes_slice_ledger_sha256(&updated_nodes);

    if let Some((manifest, _)) = manifest_ref {
        ensure!(
            actual_input_sha256s == manifest.pinned_input_sha256s,
            "CRITICAL: Pinned input SHA-256 mismatch for manifest {}: expected {:?}, found {:?}",
            manifest.manifest_id,
            manifest.pinned_input_sha256s,
            actual_input_sha256s
        );
        let planned_norm: Vec<PlannedLedgerItem> = planned_items
            .iter()
            .map(PlannedLedgerItem::normalized_for_journal)
            .collect();
        let manifest_norm: Vec<PlannedLedgerItem> = manifest
            .planned_items
            .iter()
            .map(PlannedLedgerItem::normalized_for_journal)
            .collect();
        ensure!(
            final_last_id == manifest.final_last_id
                && updated_nodes.len() == manifest.final_node_count
                && final_ledger_sha256 == manifest.final_ledger_sha256
                && planned_norm == manifest_norm,
            "CRITICAL: Planned transaction output (final_last_id=l{}, count={}, sha256={}, items={}) does not match required manifest {} (final_last_id=l{}, count={}, sha256={}, items={})",
            final_last_id,
            updated_nodes.len(),
            final_ledger_sha256,
            planned_norm.len(),
            manifest.manifest_id,
            manifest.final_last_id,
            manifest.final_node_count,
            manifest.final_ledger_sha256,
            manifest_norm.len()
        );
    }

    let touched_chunks: HashSet<&str> = planned_items
        .iter()
        .map(|i| i.target_chunk.as_str())
        .collect();
    let files_written = touched_chunks.len();
    let appended_count = planned_items.len();

    if dry_run || (planned_items.is_empty() && manifest_ref.is_none()) {
        return Ok(DurableTransactionOutcome {
            resumed_or_already_committed: false,
            files_written: 0,
            appended_count,
            total_nodes: updated_nodes.len(),
            planner_payload: Some(payload),
        });
    }

    let batch_id = if let Some((manifest, _)) = manifest_ref {
        format!("{batch_id_prefix}-{}-{final_last_id}", manifest.manifest_id)
    } else {
        format!("{batch_id_prefix}-{baseline_last_id}-{final_last_id}")
    };

    let journal_items: Vec<PlannedLedgerItem> = planned_items
        .iter()
        .map(PlannedLedgerItem::normalized_for_journal)
        .collect();

    let pending = JournalRecord {
        state: "BATCH_PENDING".to_string(),
        batch_id,
        kind: kind.to_string(),
        baseline_last_id,
        baseline_ledger_sha256,
        final_last_id,
        final_ledger_sha256,
        manifest_sha256: manifest_ref.map(|(_, s)| s.to_string()),
        items: journal_items,
        active_bundles_sha256: BTreeMap::new(),
        history_bundles_sha256: BTreeMap::new(),
        record_sha256: String::new(),
    };
    append_journal_record(&journal_path, &pending, fault)?;

    recover_pending_ledger_journal_with_guard(
        &write_guard,
        repo_root,
        manifest_ref,
        false,
        fault,
    )?;

    Ok(DurableTransactionOutcome {
        resumed_or_already_committed: false,
        files_written,
        appended_count,
        total_nodes: updated_nodes.len(),
        planner_payload: Some(payload),
    })
}

#[cfg(test)]
#[allow(
    clippy::panic,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::unwrap_in_result,
    clippy::panic_in_result_fn,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    reason = "Standard repository test boilerplate"
)]
mod tests {
    use super::*;
    use crate::ctbnbdl::load_ledger_nodes_from_dir;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_TEST_DIR_ID: AtomicUsize = AtomicUsize::new(1);

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(prefix: &str) -> Self {
            let id = NEXT_TEST_DIR_ID.fetch_add(1, Ordering::Relaxed);
            let dir = std::env::temp_dir().join(format!(
                "ctb_journal_test_{}_{}_{}",
                prefix,
                std::process::id(),
                id
            ));
            if dir.exists() {
                let _ = fs::remove_dir_all(&dir);
            }
            fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let shadow_dir = highwater_shadow_dirs(&self.0)
                    .into_iter().next().unwrap().1;
                if shadow_dir.is_dir() {
                    if let Ok(entries) = fs::read_dir(&shadow_dir) {
                        for entry in entries.flatten() {
                            let _ = fs::set_permissions(
                                entry.path(),
                                fs::Permissions::from_mode(0o644),
                            );
                        }
                    }
                }
            }
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn setup_checkout_fixture() -> TempDir {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let real_root = crate::find_repository_root_from(manifest_dir).unwrap();
        let tmp = TempDir::new("checkout");
        let root = tmp.path();

        fs::create_dir_all(root.join(".git")).unwrap();
        fs::write(root.join(".ctoolbox_workspace_root"), b"").unwrap();
        fs::write(root.join("Cargo.toml"), b"[workspace]\n").unwrap();
        fs::create_dir_all(
            root.join("src/formats/dcdata/data-src/nodes/inbox"),
        )
        .unwrap();
        fs::create_dir_all(
            root.join("src/formats/dcdata/data-src/nodes/allocated"),
        )
        .unwrap();

        let all_nodes =
            load_ledger_nodes_from_dir_raw(&real_root.join("assets/data/graph"))
                .unwrap();
        let baseline_nodes: Vec<LedgerNode> =
            all_nodes.into_iter().take(17_833).collect();
        assert_eq!(baseline_nodes.len(), 17_833);
        assert_eq!(baseline_nodes.last().unwrap().id, 8_589_952_424);

        let graph_dst = root.join("assets/data/graph");
        fs::create_dir_all(graph_dst.join("system")).unwrap();
        for chunk in baseline_nodes.chunks(100) {
            let chunk_name = format_chunk_filename(chunk[0].id);
            let mut buf = String::new();
            for n in chunk {
                buf.push_str(&n.format_line());
                buf.push('\n');
            }
            fs::write(graph_dst.join(chunk_name), buf).unwrap();
        }
        {
            let write_guard = LedgerWriteGuard::acquire(root).unwrap();
            let framing =
                resolve_ctbnbdl_framing_dcs_from_nodes(&baseline_nodes)
                    .unwrap();
            compile_nodes_slice_to_ctbnbdl_with_write_guard(
                &write_guard,
                &baseline_nodes,
                &graph_dst,
                framing,
            )
            .unwrap();
        }
        tmp
    }

    fn write_initial_checkpoint_for_test(
        repo_root: &Path,
        commit_record: bool,
    ) -> Result<()> {
        let write_guard = LedgerWriteGuard::acquire(repo_root)?;
        let journal_path = repo_root.join(LEDGER_JOURNAL_REL_PATH);
        let init_marker = repo_root.join(JOURNAL_INITIALIZED_REL_PATH);
        ensure!(
            !init_marker.exists(),
            "CRITICAL: Ledger journal {} is missing after initialization marker {} was already created!",
            journal_path.display(),
            init_marker.display()
        );
        let graph_dir = repo_root.join("assets/data/graph");
        let nodes = load_ledger_nodes_from_dir_with_write_guard(
            &write_guard,
            &graph_dir,
        )?;
        let last_id = nodes.last().map(|n| n.id).unwrap_or(0);
        let actual_ledger_sha256 = compute_ledger_dir_sha256(&graph_dir)?;
        let framing = resolve_ctbnbdl_framing_dcs_from_nodes(&nodes)?;
        let active_bundles_sha256 = compute_bundle_dir_sha256s(&graph_dir)?;
        let history_bundles_sha256 =
            compute_bundle_dir_sha256s(&graph_dir.join("system"))?;
        verify_compiled_bundles_match_ledger(
            &graph_dir,
            &nodes,
            framing,
            Some(&active_bundles_sha256),
            Some(&history_bundles_sha256),
        )?;

        let pending = JournalRecord {
            state: "BATCH_PENDING".to_string(),
            batch_id: "bootstrap-a90464028".to_string(),
            kind: "bootstrap_checkpoint".to_string(),
            baseline_last_id: last_id,
            baseline_ledger_sha256: actual_ledger_sha256.clone(),
            final_last_id: last_id,
            final_ledger_sha256: actual_ledger_sha256.clone(),
            manifest_sha256: None,
            items: Vec::new(),
            active_bundles_sha256: BTreeMap::new(),
            history_bundles_sha256: BTreeMap::new(),
            record_sha256: String::new(),
        };
        append_journal_record(&journal_path, &pending, None)?;
        if !commit_record {
            bail!("Simulated fault injection: crash after BATCH_PENDING before BATCH_COMMITTED");
        }
        let committed = JournalRecord {
            state: "BATCH_COMMITTED".to_string(),
            batch_id: "bootstrap-a90464028".to_string(),
            kind: "bootstrap_checkpoint".to_string(),
            baseline_last_id: last_id,
            baseline_ledger_sha256: actual_ledger_sha256.clone(),
            final_last_id: last_id,
            final_ledger_sha256: actual_ledger_sha256,
            manifest_sha256: None,
            items: Vec::new(),
            active_bundles_sha256,
            history_bundles_sha256,
            record_sha256: String::new(),
        };
        append_journal_record(&journal_path, &committed, None)?;
        stamp_journal_initialized_marker(repo_root)?;
        Ok(())
    }

    #[test]
    fn test_journal_initialization_and_missing_journal_rejection() {
        let tmp = setup_checkout_fixture();
        let root = tmp.path();
        let graph_dir = root.join("assets/data/graph");
        let journal_path = root.join(LEDGER_JOURNAL_REL_PATH);
        let init_marker = root.join(JOURNAL_INITIALIZED_REL_PATH);

        // Before journal exists, gated readers must reject the uninitialized checkout (never trusting a missing journal).
        let err = load_ledger_nodes_from_dir(&graph_dir).unwrap_err();
        assert!(
            err.to_string().contains("is missing"),
            "Expected missing journal error before initialization, got: {err}"
        );

        // Write initial journal checkpoint under LOCK_EX.
        write_initial_checkpoint_for_test(root, true).unwrap();
        assert!(journal_path.is_file());
        assert!(init_marker.is_file());

        // Verify journal has BATCH_PENDING + BATCH_COMMITTED with items: [] and 17,833 nodes untouched.
        let parsed = inspect_ledger_journal(&journal_path).unwrap();
        assert!(parsed.torn_trailing_fragment.is_none());
        assert!(parsed.open_pending.is_none());
        assert_eq!(parsed.records.len(), 2);
        assert_eq!(parsed.committed_pairs.len(), 1);
        assert_eq!(
            parsed.last_committed.as_ref().unwrap().batch_id,
            "bootstrap-a90464028"
        );

        // Now gated readers and dc_codegen succeed!
        let nodes = load_ledger_nodes_from_dir(&graph_dir).unwrap();
        assert_eq!(nodes.len(), 17_833);
        let dc_code = crate::dc_codegen::generate_dc_code(&graph_dir).unwrap();
        assert!(dc_code.contains("pub const DC_RANGE"));

        // Delete ledger-journal.jsonl after initialization and verify gated readers reject!
        fs::remove_file(&journal_path).unwrap();
        let err_reader = load_ledger_nodes_from_dir(&graph_dir).unwrap_err();
        assert!(
            err_reader
                .to_string()
                .contains("initialized_marker=true"),
            "Expected post-initialization missing journal error, got: {err_reader}"
        );
    }

    #[test]
    fn test_pending_journal_crash_recovery_via_standalone_build_support() {
        let tmp = setup_checkout_fixture();
        let root = tmp.path();
        let graph_dir = root.join("assets/data/graph");

        // 1. Crash after writing BATCH_PENDING before BATCH_COMMITTED.
        let crash_err =
            write_initial_checkpoint_for_test(root, false).unwrap_err();
        assert!(crash_err.to_string().contains("Simulated fault injection"));

        // Verify dc_codegen / gated readers are blocked while BATCH_PENDING is open.
        let blocked_err =
            crate::dc_codegen::generate_dc_code(&graph_dir).unwrap_err();
        assert!(
            blocked_err
                .to_string()
                .contains("uncommitted BATCH_PENDING transaction"),
            "Expected dc_codegen to be blocked while BATCH_PENDING is open, got: {blocked_err}"
        );

        // Recover using standalone ctb_build_support::recover_pending_ledger_journal (no validate_data_tables needed).
        let recovered =
            recover_pending_ledger_journal(root, None, false).unwrap();
        assert!(recovered);

        // Now dc_codegen succeeds!
        let dc_code = crate::dc_codegen::generate_dc_code(&graph_dir).unwrap();
        assert!(dc_code.contains("pub const DC_RANGE"));
    }

    #[test]
    fn test_reject_tampered_ledger_against_journal() {
        let temp_dir = setup_checkout_fixture();
        let root = temp_dir.path();
        write_initial_checkpoint_for_test(root, true).unwrap();

        // Tamper with a .ctbnodes chunk after journal commit and verify gated reader rejects it.
        let tail_chunk = root.join("assets/data/graph/85899_52392.ctbnodes");
        let mut content = fs::read_to_string(&tail_chunk).unwrap();
        content.push_str("l8589952425: 65 # tampered\n");
        fs::write(&tail_chunk, content).unwrap();
        let read_guard = LedgerReadGuard::acquire(root).unwrap();
        let err_tampered =
            ensure_ledger_journal_clean(root, &read_guard).unwrap_err();
        assert!(
            err_tampered
                .to_string()
                .contains("does not match latest BATCH_COMMITTED")
        );
    }

    #[test]
    fn test_bundle_verification_catches_token_and_partition_corruption_at_same_max_id() {
        let tmp = setup_checkout_fixture();
        let root = tmp.path();
        let graph_dir = root.join("assets/data/graph");
        write_initial_checkpoint_for_test(root, true).unwrap();

        let nodes = load_ledger_nodes_from_dir_raw(&graph_dir).unwrap();
        let framing =
            resolve_ctbnbdl_framing_dcs_from_nodes(&nodes)
                .unwrap();

        // Corrupt a single token on an interior node (keeping the highest node ID l8589952424 unchanged!)
        // and re-encode the active bundle.
        let (mut active_nodes, _inactive_nodes) =
            partition_active_and_inactive_nodes_with(&nodes, framing);
        active_nodes[10].tokens = vec![crate::shorthand::DcShorthand::Unicode(999)];
        let corrupted_bundles = encode_nodes_to_ctbnbdl_bundles_with_limits(
            &active_nodes,
            framing,
            MAX_NODES_PER_BUNDLE,
            MAX_BYTES_PER_BUNDLE,
        )
        .unwrap();
        write_ctbnbdl_bundles_to_dir(&graph_dir, &corrupted_bundles, false)
            .unwrap();

        // Verify that both verify_compiled_bundles_match_ledger (even without expected SHA-256 map)
        // and ensure_ledger_journal_clean reject the corrupted bundle!
        let err_verify = verify_compiled_bundles_match_ledger(
            &graph_dir, &nodes, framing, None, None,
        )
        .unwrap_err();
        assert!(
            err_verify
                .to_string()
                .contains("do not match expected active partition")
        );

        let read_guard = LedgerReadGuard::acquire(root).unwrap();
        let err_gate =
            ensure_ledger_journal_clean(root, &read_guard).unwrap_err();
        assert!(
            err_gate.to_string().contains("bundle SHA-256 inventory")
                || err_gate
                    .to_string()
                    .contains("do not match expected active partition")
        );
    }
}
