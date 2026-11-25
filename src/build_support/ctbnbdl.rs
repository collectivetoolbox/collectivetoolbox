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

//! `.ctbnodes` (`CtbNodes`, `f639`) and `.ctbnbdl` (`CtbNodeBundle`, `f640`)
//! node ledger parsing and binary compilation.

#[path = "ctbnbdl/journal.rs"]
pub mod journal;
pub use journal::*;

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::path::{Path, PathBuf};

use crate::dctf::{decode_dctf_buf, decode_utf_8e_128_buf, encode_dctf_buf};
use crate::global_graph_layout::{SYSTEM_REGION_END, SYSTEM_REGION_START};
use crate::shorthand::DcShorthand;

/// Maximum number of node declarations stored in a single `.ctbnodes` chunk file.
pub const MAX_NODES_PER_CHUNK: usize = 100;

/// Maximum number of `.ctbnodes` files stored directly in the top-level graph directory
/// before sharding into 1000-file prefix subdirectories.
pub const ROOT_CHUNKS_BEFORE_SUBFOLDER: usize = 999;

/// Number of `.ctbnodes` files grouped into each prefix subdirectory starting at the 1000th file.
pub const CHUNKS_PER_SUBFOLDER: usize = 1000;

/// Relative repository path to the tracked unsplit `.ctbnbdl` bundle directory.
pub const GRAPH_BUNDLES_REL_PATH: &str = "assets/data/graph-bundles";

/// Subdirectory under `assets/data/graph-bundles/` holding gitignored live (active) `.ctbnbdl` bundles.
pub const GRAPH_BUNDLES_SPLIT_LIVE_SUBDIR: &str = "split/live";

/// Subdirectory under `assets/data/graph-bundles/` holding gitignored historical (inactive) `.ctbnbdl` bundles.
pub const GRAPH_BUNDLES_SPLIT_HISTORY_SUBDIR: &str = "split/history";

/// Maximum number of node definitions grouped into a single `.ctbnbdl` binary bundle file.
pub const MAX_NODES_PER_BUNDLE: usize = 1_000_000;

/// Maximum size in bytes of a single `.ctbnbdl` binary bundle file (50 MB).
pub const MAX_BYTES_PER_BUNDLE: usize = 50_000_000;

/// Leading 8-byte magic header for `.ctbnbdl` (`CtbNodeBundle2`, `f645`) files.
pub const CTBNBDL_MAGIC: [u8; 8] = *b"CTBNBDL\x02";

/// Legacy 8-byte magic header for v1 `.ctbnbdl` (`CtbNodeBundle1`, `f644`) files.
pub const CTBNBDL_MAGIC_V1: [u8; 8] = *b"CTBNBDL\x01";

/// Standard 21-byte `DcMixed2` (`f643`) header (`DcMM\0` + UUID `b0327a84-b745-492c-b39d-77c2827e3a6f`).
pub const DCMM_HEADER: [u8; 21] = [
    b'D', b'c', b'M', b'M', 0, 0xb0, 0x32, 0x7a, 0x84, 0xb7, 0x45, 0x49, 0x2c,
    0xb3, 0x9d, 0x77, 0xc2, 0x82, 0x7e, 0x3a, 0x6f,
];

/// Legacy 21-byte `DcMixed1` (`f642`) header (`DcMM\0` + UUID `d5ab819e-9b8e-4d61-adbc-933e8851e79a`).
pub const DCMM_HEADER_V1: [u8; 21] = [
    b'D', b'c', b'M', b'M', 0, 0xd5, 0xab, 0x81, 0x9e, 0x9b, 0x8e, 0x4d, 0x61,
    0xad, 0xbc, 0x93, 0x3e, 0x88, 0x51, 0xe7, 0x9a,
];

const DC_203_DCTF_BYTES: [u8; 4] = [0xF4, 0x90, 0x83, 0x8B];
const DC_203_UTF8E128_BYTES: [u8; 5] = [0xF8, 0x88, 0xA0, 0x83, 0x8B];
const DCMM_TYPE_BINARY: u8 = 0;
const DCMM_TYPE_LEGACY_TEXT: u8 = 1;
const DCMM_TYPE_BINARY_METADATA: u8 = 2;
const DCMM_TYPE_BINARY_SHA256: u8 = 3;
const DCMM_TYPE_LEGACY_SHA256: u8 = 4;
const BASE64_DIGIT_START: u32 = 127;
const BASE64_DIGIT_END: u32 = 190;
const BASE64_PADDING_DC: u32 = 195;

/// `Dc 641` (`Global graph system block most recent node ID at time of document creation`).
pub const DC_SYSTEM_BLOCK_MOST_RECENT_NODE_ID: DcShorthand =
    DcShorthand::Short(641);

/// Structured comment prefix recording a node's creation timestamp in signed nanoseconds
/// since the Unix epoch inside `.ctbnodes` lines (`# @created_ns:<i128> ...`).
pub const CREATED_NS_COMMENT_PREFIX: &str = "@created_ns:";

/// Framing and relationship Document Characters used when encoding, decoding, and partitioning
/// `.ctbnbdl` node definitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CtbnbdlFramingDcs {
    /// `BeginGraphNodeDefinition` (`Dc 617`).
    pub begin_node: DcShorthand,
    /// `EndGraphNodeDefinition` (`Dc 618`).
    pub end_node: DcShorthand,
    /// `BeginChecksummedSection` (`Dc 543`).
    pub begin_checksummed_section: DcShorthand,
    /// `EndChecksummedSection` (`Dc 544`).
    pub end_checksummed_section: DcShorthand,
    /// `SectionDigest` (`Dc 545`).
    pub section_digest: DcShorthand,
    /// `BeginTimeMoment` (`Dc 390`).
    pub begin_time_moment: DcShorthand,
    /// `EndTimeMoment` (`Dc 391`).
    pub end_time_moment: DcShorthand,
    /// `SectionTimestampNodeCreationTime` (`Dc 1941`), or `None` on historical prefixes prior to its introduction.
    pub section_node_creation_time: Option<DcShorthand>,
    /// `NsSinceEpoch` (`FormatId::NsSinceEpoch`, `f487`).
    pub ns_since_epoch_format: DcShorthand,
    /// `Sha256` (`FormatId::Sha256`, `f245`).
    pub sha256_format: DcShorthand,
    /// `Escape` (`Dc 255`, `Ignore following Dc`).
    pub escape: DcShorthand,
    /// `EvaluateValueAsAtom` (`Dc 616`).
    pub evaluate_as_atom: DcShorthand,
    /// `HasA` (`Dc 584`).
    pub has_a: DcShorthand,
    /// `HasValue` (`Dc 610`).
    pub has_value: DcShorthand,
    /// `Preferred` (`Dc 570`).
    pub preferred: DcShorthand,
    /// `Supersedes` (`Dc 591`).
    pub supersedes: DcShorthand,
    /// `Retracts` (`Dc 592`).
    pub retracts: DcShorthand,
    /// `EnclosingDocument` (`Dc 615`).
    pub enclosing_document: DcShorthand,
    /// `IsInstanceOf` (`Dc 586`).
    pub is_instance_of: DcShorthand,
    /// `IsSubtypeOf` (`Dc 587`).
    pub is_subtype_of: DcShorthand,
    /// `IsBasedOn` (`Dc 588`).
    pub is_based_on: DcShorthand,
    /// `IsDeprecated` (`Dc 589`).
    pub is_deprecated: DcShorthand,
    /// `IsClosingDelimiterFor` (`Dc 590`).
    pub is_closing_delimiter_for: DcShorthand,
    /// `IsAssociatedWith` (`Dc 598`).
    pub is_associated_with: DcShorthand,
}

/// Sanitizes a `.ctbnodes` comment string by escaping carriage returns and newlines.
#[must_use]
pub fn sanitize_comment(raw: &str) -> String {
    raw.replace('\r', "\\r").replace('\n', "\\n")
}

/// Returns the current system wall-clock timestamp in signed nanoseconds since the Unix epoch.
#[must_use]
pub fn current_unix_timestamp_ns() -> i128 {
    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(dur) => i128::try_from(dur.as_nanos()).unwrap_or(i128::MAX),
        Err(err) => {
            let before =
                i128::try_from(err.duration().as_nanos()).unwrap_or(i128::MAX);
            // Reason for fallback: saturating to i128::MIN if pre-epoch duration exceeds i128::MAX avoids overflow.
            before.checked_neg().unwrap_or(i128::MIN)
        }
    }
}

/// Single node entry in the System-block `.ctbnodes` ledger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerNode {
    /// Global node ID in `SYSTEM_REGION_START..=SYSTEM_REGION_END`.
    pub id: u128,
    /// Ordered token sequence defining this node.
    pub tokens: Vec<DcShorthand>,
    /// Human-readable comment rendered after `# `.
    pub comment: String,
    /// Optional creation timestamp in signed nanoseconds since the Unix epoch,
    /// persisted in `.ctbnodes` via `# @created_ns:<i128>` and in `.ctbnbdl`
    /// checksummed sections via `SectionTimestampNodeCreationTime`.
    pub created_ns: Option<i128>,
}

impl LedgerNode {
    /// Formats this node as a single `.ctbnodes` line (`l<id>: <tokens> # [@created_ns:<ts>] <comment>`).
    #[must_use]
    pub fn format_line(&self) -> String {
        let token_strs: Vec<String> =
            self.tokens.iter().map(ToString::to_string).collect();
        let body = token_strs.join(" ");
        let clean_comment = sanitize_comment(&self.comment);
        let full_comment = match (self.created_ns, clean_comment.is_empty()) {
            (Some(ts), true) => format!("{CREATED_NS_COMMENT_PREFIX}{ts}"),
            (Some(ts), false) => {
                format!("{CREATED_NS_COMMENT_PREFIX}{ts} {clean_comment}")
            }
            (None, _) => clean_comment,
        };
        match (body.is_empty(), full_comment.is_empty()) {
            (true, true) => format!("l{}: #", self.id),
            (true, false) => format!("l{}: # {full_comment}", self.id),
            (false, true) => format!("l{}: {body}", self.id),
            (false, false) => format!("l{}: {body} # {full_comment}", self.id),
        }
    }

    /// Parses a single non-empty `.ctbnodes` line.
    pub fn parse_line(
        line: &str,
        file_label: &str,
        line_no: usize,
    ) -> Result<Self> {
        let trimmed = line.trim();
        ensure!(
            !trimmed.is_empty(),
            "{file_label}:{line_no}: unexpected blank line in .ctbnodes file"
        );
        let (code_part, created_ns, comment) = match trimmed.split_once('#') {
            Some((before, after)) => {
                let raw_after = after.trim();
                if let Some(rest) =
                    raw_after.strip_prefix(CREATED_NS_COMMENT_PREFIX)
                {
                    let (ts_str, rem_comment) =
                        match rest.find(char::is_whitespace) {
                            Some(pos) => {
                                // Reason for fallback: pos comes from rest.find(...) so ..pos and pos.. are valid UTF-8 boundaries.
                                let head = rest.get(..pos).unwrap_or_default();
                                // Reason for fallback: pos comes from rest.find(...) so pos.. is a valid UTF-8 boundary.
                                let tail =
                                    rest.get(pos..).unwrap_or_default().trim();
                                (head, tail)
                            }
                            None => (rest, ""),
                        };
                    let ts = ts_str.parse::<i128>().with_context(|| {
                        format!(
                            "{file_label}:{line_no}: invalid @created_ns timestamp '{ts_str}'"
                        )
                    })?;
                    (before.trim(), Some(ts), rem_comment.to_string())
                } else {
                    (before.trim(), None, raw_after.to_string())
                }
            }
            None => (trimmed, None, String::new()),
        };
        let Some((lhs, rhs)) = code_part.split_once(':') else {
            bail!(
                "{file_label}:{line_no}: missing ':' separator in node declaration '{trimmed}'"
            );
        };
        let lhs_shorthand =
            DcShorthand::parse(lhs.trim()).with_context(|| {
                format!(
                    "{file_label}:{line_no}: invalid node ID shorthand '{lhs}'"
                )
            })?;
        let DcShorthand::Long(id) = lhs_shorthand else {
            bail!(
                "{file_label}:{line_no}: System-block node ID must use 'l<digits>' syntax, found '{lhs}'"
            );
        };
        ensure!(
            (SYSTEM_REGION_START..=SYSTEM_REGION_END).contains(&id),
            "{file_label}:{line_no}: node ID l{id} is outside System region ({SYSTEM_REGION_START}..={SYSTEM_REGION_END})"
        );
        let mut tokens = Vec::new();
        for raw_tok in rhs.split_whitespace() {
            let tok = DcShorthand::parse(raw_tok).with_context(|| {
                format!(
                    "{file_label}:{line_no}: invalid token shorthand '{raw_tok}'"
                )
            })?;
            tokens.push(tok);
        }
        Ok(Self {
            id,
            tokens,
            comment,
            created_ns,
        })
    }
}

fn format_grouped_id(first_id: u128) -> String {
    let digits = first_id.to_string();
    let len = digits.len();
    let mut grouped = String::with_capacity(
        len.saturating_add(len.checked_div(5).unwrap_or(0)),
    );
    for (idx, ch) in digits.chars().enumerate() {
        let remaining = len.saturating_sub(idx);
        if idx > 0 && remaining.is_multiple_of(5) {
            grouped.push('_');
        }
        grouped.push(ch);
    }
    grouped
}

/// Formats a global node ID into a `.ctbnodes` chunk filename with `_` grouping
/// every 5 digits from the right (e.g., `8589934592` -> `"85899_34592.ctbnodes"`).
#[must_use]
pub fn format_chunk_filename(first_id: u128) -> String {
    format!("{}.ctbnodes", format_grouped_id(first_id))
}

/// Formats a global node ID into a `.ctbnbdl` chunk bundle filename with `_` grouping
/// every 5 digits from the right (e.g., `8589934592` -> `"85899_34592.ctbnbdl"`).
#[must_use]
pub fn format_chunk_bundle_filename(first_id: u128) -> String {
    format!("{}.ctbnbdl", format_grouped_id(first_id))
}

fn parse_grouped_stem_id(stem: &str) -> Option<u128> {
    if stem.is_empty() || stem.starts_with('_') || stem.ends_with('_') {
        return None;
    }
    let mut clean = String::with_capacity(stem.len());
    for ch in stem.chars() {
        if ch.is_ascii_digit() {
            clean.push(ch);
        } else if ch == '_' {
            // separator
        } else {
            return None;
        }
    }
    if clean.is_empty() {
        return None;
    }
    clean.parse::<u128>().ok()
}

/// Parses a `.ctbnodes` chunk filename (e.g., `"85899_34592.ctbnodes"`) into its starting node ID.
#[must_use]
pub fn parse_chunk_filename_start_id(filename: &str) -> Option<u128> {
    let stem = filename.strip_suffix(".ctbnodes")?;
    parse_grouped_stem_id(stem)
}

/// Parses a `.ctbnbdl` chunk filename (e.g., `"85899_34592.ctbnbdl"`) into its starting node ID.
#[must_use]
pub fn parse_chunk_bundle_filename_start_id(filename: &str) -> Option<u128> {
    let stem = filename.strip_suffix(".ctbnbdl")?;
    parse_grouped_stem_id(stem)
}

/// Resolves the `.ctbnbdl` bundle directory associated with `graph_dir`:
/// - `<parent>/graph-bundles` when `graph_dir` ends with `"graph"` (e.g. `assets/data/graph` -> `assets/data/graph-bundles`),
/// - `<graph_dir>/graph-bundles` otherwise.
#[must_use]
pub fn bundles_dir_for_graph_dir(graph_dir: &Path) -> PathBuf {
    if graph_dir.file_name().and_then(|s| s.to_str()) == Some("graph") {
        graph_dir.with_file_name("graph-bundles")
    } else {
        graph_dir.join("graph-bundles")
    }
}

/// Formats a `.ctbnodes` chunk's relative path inside `graph_dir`:
/// - The first 999 `.ctbnodes` files (`chunk_idx` `0..999`, i.e. `85899_34592.ctbnodes` through
///   `85900_34392.ctbnodes`) live directly in `graph_dir`.
/// - Starting at the 1000th `.ctbnodes` file (`chunk_idx == 999`, `85900_34492.ctbnodes`), each
///   block of 1000 `.ctbnodes` files lives in a subdirectory named after the grouped prefix of
///   that block's starting chunk ID (`"85900/85900_34492.ctbnodes"`, ..., `"85901/85901_34492.ctbnodes"`).
#[must_use]
pub fn format_chunk_rel_path(first_id: u128) -> String {
    let filename = format_chunk_filename(first_id);
    let offset = first_id.saturating_sub(SYSTEM_REGION_START);
    // Reason for fallback: MAX_NODES_PER_CHUNK is the compile-time constant 100.
    let per_chunk = u128::try_from(MAX_NODES_PER_CHUNK).unwrap_or(100);
    // Reason for fallback: per_chunk is non-zero (100).
    let chunk_idx = offset.checked_div(per_chunk).unwrap_or(0);
    // Reason for fallback: ROOT_CHUNKS_BEFORE_SUBFOLDER is the compile-time constant 999.
    let root_limit = u128::try_from(ROOT_CHUNKS_BEFORE_SUBFOLDER).unwrap_or(999);
    if chunk_idx < root_limit {
        return filename;
    }
    // Reason for fallback: CHUNKS_PER_SUBFOLDER is the compile-time constant 1000.
    let per_sub = u128::try_from(CHUNKS_PER_SUBFOLDER).unwrap_or(1000);
    // Reason for fallback: per_sub is non-zero (1000).
    let bucket_idx = chunk_idx
        .saturating_sub(root_limit)
        .checked_div(per_sub)
        .unwrap_or(0);
    let bucket_start_chunk =
        root_limit.saturating_add(bucket_idx.saturating_mul(per_sub));
    let bucket_first_id = SYSTEM_REGION_START
        .saturating_add(bucket_start_chunk.saturating_mul(per_chunk));
    let bucket_grouped = format_grouped_id(bucket_first_id);
    // Reason for fallback: System-region IDs are >= 10 digits so format_grouped_id always contains '_'.
    let prefix = bucket_grouped
        .rsplit_once('_')
        .map_or(bucket_grouped.as_str(), |(p, _)| p);
    format!("{prefix}/{filename}")
}

/// Parses a `.ctbnodes` relative path (`"85899_34592.ctbnodes"` or `"85900/85900_34492.ctbnodes"`)
/// into its starting node ID, validating any parent directory component.
#[must_use]
pub fn parse_chunk_rel_path_start_id(rel_path: &str) -> Option<u128> {
    match rel_path.split_once('/') {
        None => parse_chunk_filename_start_id(rel_path),
        Some((subdir, fname)) => {
            if fname.contains('/') || parse_grouped_stem_id(subdir).is_none() {
                return None;
            }
            parse_chunk_filename_start_id(fname)
        }
    }
}

/// Returns `path` formatted relative to `graph_dir` using forward-slash separators.
#[must_use]
pub fn chunk_rel_path_from_graph_dir(
    graph_dir: &Path,
    path: &Path,
) -> Option<String> {
    let rel = path.strip_prefix(graph_dir).ok()?;
    let mut parts: Vec<&str> = Vec::new();
    for comp in rel.components() {
        parts.push(comp.as_os_str().to_str()?);
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("/"))
    }
}

/// Returns `true` if `dir_name` is a valid numeric-prefix `.ctbnodes` shard subdirectory name
/// (e.g. `"85900"`, `"85901"`, `"1_00000"`).
#[must_use]
pub fn is_chunk_prefix_subdir_name(dir_name: &str) -> bool {
    parse_grouped_stem_id(dir_name).is_some()
}

/// Returns the sorted list of `.ctbnodes` chunk files in `graph_dir` (including 1000-file
/// numeric-prefix subdirectories).
pub fn list_chunk_files(graph_dir: &Path) -> Result<Vec<PathBuf>> {
    if !graph_dir.exists() {
        return Ok(Vec::new());
    }
    let mut keyed_files: Vec<(u128, PathBuf)> = Vec::new();
    for entry in std::fs::read_dir(graph_dir).with_context(|| {
        format!("Failed to read graph directory {}", graph_dir.display())
    })? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if file_type.is_file() {
            if let Some(start_id) = parse_chunk_filename_start_id(name) {
                if (SYSTEM_REGION_START..=SYSTEM_REGION_END).contains(&start_id)
                {
                    keyed_files.push((start_id, path));
                }
            }
        } else if file_type.is_dir() && is_chunk_prefix_subdir_name(name) {
            for sub_entry in std::fs::read_dir(&path).with_context(|| {
                format!(
                    "Failed to read graph shard subdirectory {}",
                    path.display()
                )
            })? {
                let sub_entry = sub_entry?;
                if !sub_entry.file_type()?.is_file() {
                    continue;
                }
                let sub_path = sub_entry.path();
                let Some(sub_name) =
                    sub_path.file_name().and_then(|s| s.to_str())
                else {
                    continue;
                };
                if let Some(start_id) = parse_chunk_filename_start_id(sub_name)
                {
                    if (SYSTEM_REGION_START..=SYSTEM_REGION_END)
                        .contains(&start_id)
                    {
                        keyed_files.push((start_id, sub_path));
                    }
                }
            }
        }
    }
    keyed_files.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    Ok(keyed_files.into_iter().map(|(_, p)| p).collect())
}

/// Loads all `LedgerNode` entries from a `.ctbnodes` text string.
pub fn load_ledger_nodes_from_str(
    content: &str,
    file_label: &str,
) -> Result<Vec<LedgerNode>> {
    let mut nodes = Vec::new();
    for (idx, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let line_no = idx.saturating_add(1);
        let node = LedgerNode::parse_line(line, file_label, line_no)?;
        nodes.push(node);
    }
    Ok(nodes)
}

/// Loads all `LedgerNode` entries from a single `.ctbnodes` file.
pub fn load_ledger_nodes_from_file(path: &Path) -> Result<Vec<LedgerNode>> {
    // Reason for fallback: paths without a terminal filename component use the generic ".ctbnodes" diagnostic label
    let file_label = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(".ctbnodes");
    let content = std::fs::read_to_string(path).with_context(|| {
        format!("Failed to read .ctbnodes file {}", path.display())
    })?;
    load_ledger_nodes_from_str(&content, file_label)
}

fn load_ledger_nodes_from_dir_raw(graph_dir: &Path) -> Result<Vec<LedgerNode>> {
    let files = list_chunk_files(graph_dir)?;
    let mut nodes = Vec::new();
    for (chunk_idx, path) in files.iter().enumerate() {
        // Reason for fallback: chunk_idx * 100 easily fits in u128 on any 64-bit platform
        let chunk_start_id = SYSTEM_REGION_START.saturating_add(
            u128::try_from(chunk_idx.saturating_mul(MAX_NODES_PER_CHUNK))
                .unwrap_or(0),
        );
        let expected_rel = format_chunk_rel_path(chunk_start_id);
        // Reason for fallback: paths outside graph_dir or non-UTF-8 fail the equality check against expected_rel immediately below
        let actual_rel = chunk_rel_path_from_graph_dir(graph_dir, path)
            .unwrap_or_else(|| path.display().to_string());
        ensure!(
            actual_rel == expected_rel,
            "Expected sequential chunk file '{expected_rel}', found '{actual_rel}'"
        );
        let file_nodes = load_ledger_nodes_from_file(path)?;
        let chunk_count = file_nodes.len();
        for (line_idx, node) in file_nodes.into_iter().enumerate() {
            let line_no = line_idx.saturating_add(1);
            // Reason for fallback: nodes.len() fits in u128 on any platform
            let expected_id = SYSTEM_REGION_START
                .saturating_add(u128::try_from(nodes.len()).unwrap_or(0));
            ensure!(
                node.id == expected_id,
                "{actual_rel}:{line_no}: expected sequential node ID l{expected_id}, found l{}",
                node.id
            );
            nodes.push(node);
        }
        ensure!(
            chunk_count <= MAX_NODES_PER_CHUNK,
            "{actual_rel}: chunk contains {chunk_count} nodes, exceeding maximum {MAX_NODES_PER_CHUNK}"
        );
        if chunk_idx.saturating_add(1) < files.len() {
            ensure!(
                chunk_count == MAX_NODES_PER_CHUNK,
                "{actual_rel}: non-final chunk has {chunk_count} nodes, expected {MAX_NODES_PER_CHUNK}. \
                 CRITICAL: This indicates {actual_rel} was truncated or reverted (e.g. via 'git checkout') \
                 after later chunk files were created. NEVER delete or revert .ctbnodes files!"
            );
        }
    }
    Ok(nodes)
}

/// Verifies that `current_nodes` is a strict append-only extension of `baseline_nodes`
/// (every node in `baseline_nodes` must appear at the same index in `current_nodes`
/// with the exact same `id` and `tokens`).
pub fn verify_nodes_append_only_against_baseline(
    baseline_label: &str,
    baseline_nodes: &[LedgerNode],
    current_nodes: &[LedgerNode],
) -> Result<()> {
    // Reason for fallback: empty baseline has no last node ID
    let last_base_id = baseline_nodes.last().map_or(0, |n| n.id);
    ensure!(
        current_nodes.len() >= baseline_nodes.len(),
        "CRITICAL: Append-only graph ledger rollback detected against {baseline_label}! \
         Working tree has {} nodes, but {baseline_label} has {} nodes (up to l{last_base_id}). \
         Graph files in assets/data/graph/ are strictly append-only and must NEVER be reverted or deleted.",
        current_nodes.len(),
        baseline_nodes.len()
    );
    for (idx, base_node) in baseline_nodes.iter().enumerate() {
        let Some(cur_node) = current_nodes.get(idx) else {
            bail!(
                "CRITICAL: Missing node at index {idx} (expected l{}) when verifying against {baseline_label}",
                base_node.id
            );
        };
        ensure!(
            cur_node.id == base_node.id
                && cur_node.tokens == base_node.tokens
                && cur_node.created_ns == base_node.created_ns,
            "CRITICAL: Append-only graph ledger mutation detected at node l{} against {baseline_label}! \
             Existing node IDs, creation timestamps, and Dc statements are immutable (expected created_ns={:?} tokens {:?}, found created_ns={:?} tokens {:?}).",
            base_node.id,
            base_node.created_ns,
            base_node.tokens,
            cur_node.created_ns,
            cur_node.tokens
        );
    }
    Ok(())
}

fn find_repo_root_for_main_graph_dir(graph_dir: &Path) -> Option<PathBuf> {
    let data_dir = graph_dir.parent()?;
    if data_dir.file_name().and_then(|s| s.to_str()) != Some("data") {
        return None;
    }
    let assets_dir = data_dir.parent()?;
    if assets_dir.file_name().and_then(|s| s.to_str()) != Some("assets") {
        return None;
    }
    let repo_root = assets_dir.parent()?;
    if repo_root.file_name().and_then(|s| s.to_str()) == Some("built") {
        return None;
    }
    if repo_root.join(".git").exists() {
        Some(repo_root.to_path_buf())
    } else {
        None
    }
}

/// Relative repository path to the reconstructible high-water ledger cache.
pub const HIGHWATER_LEDGER_REL_PATH: &str = "built/graph-ledger/highwater";

fn highwater_shadow_dirs(repo_root: &Path) -> Vec<(String, PathBuf)> {
    vec![(
        format!(
            "reconstructible high-water ledger cache ({HIGHWATER_LEDGER_REL_PATH})"
        ),
        repo_root.join(HIGHWATER_LEDGER_REL_PATH),
    )]
}

/// Verifies `current_nodes` against both high-water shadow ledgers and `git HEAD`
/// (when `.git` and `git` are available).
pub fn verify_repo_graph_append_only(
    repo_root: &Path,
    current_nodes: &[LedgerNode],
) -> Result<()> {
    for (label, shadow_dir) in highwater_shadow_dirs(repo_root) {
        if shadow_dir.is_dir() {
            let shadow_nodes = load_ledger_nodes_from_dir_raw(&shadow_dir)
                .with_context(|| {
                    format!(
                        "Failed to load {label} from {}",
                        shadow_dir.display()
                    )
                })?;
            if !shadow_nodes.is_empty() {
                verify_nodes_append_only_against_baseline(
                    &label,
                    &shadow_nodes,
                    current_nodes,
                )?;
            }
        }
    }

    if repo_root.join(".git").exists() {
        if let Ok(ls_out) = std::process::Command::new("/usr/bin/git")
            .arg("-C")
            .arg(repo_root)
            .args([
                "ls-tree",
                "-r",
                "--name-only",
                "HEAD",
                "assets/data/graph/",
            ])
            .output()
            && ls_out.status.success()
        {
            let stdout = String::from_utf8_lossy(&ls_out.stdout);
            let mut keyed_paths: Vec<(u128, String)> = Vec::new();
            for line in stdout.lines() {
                let rel = line.trim();
                let Some(fname) =
                    Path::new(rel).file_name().and_then(|s| s.to_str())
                else {
                    continue;
                };
                if let Some(start_id) = parse_chunk_filename_start_id(fname) {
                    keyed_paths.push((start_id, rel.to_string()));
                }
            }
            keyed_paths
                .sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
            let mut head_nodes = Vec::new();
            for (_, rel) in keyed_paths {
                let show_out = std::process::Command::new("/usr/bin/git")
                    .arg("-C")
                    .arg(repo_root)
                    .arg("show")
                    .arg(format!("HEAD:{rel}"))
                    .output()
                    .with_context(|| {
                        format!("Failed to read HEAD:{rel} from git")
                    })?;
                if show_out.status.success() {
                    let text = String::from_utf8(show_out.stdout)
                        .with_context(|| {
                            format!("Invalid UTF-8 in HEAD:{rel}")
                        })?;
                    let chunk_nodes = load_ledger_nodes_from_str(&text, &rel)?;
                    head_nodes.extend(chunk_nodes);
                }
            }
            if !head_nodes.is_empty() {
                verify_nodes_append_only_against_baseline(
                    "git HEAD (assets/data/graph/*.ctbnodes)",
                    &head_nodes,
                    current_nodes,
                )?;
            }
        }
    }

    Ok(())
}

/// Writes or appends a single `.ctbnodes` chunk file using strictly append-only semantics:
/// - If `path` already exists and equals `content`, does not open the file for writing (`Ok(false)`).
/// - If `path` already exists and is a strict byte prefix of `content`, opens `path` with
///   `O_WRONLY | O_APPEND` (compatible with kernel `chattr +a`) and writes only the newly appended tail.
/// - If `path` already exists and is NOT a strict byte prefix of `content`, fails loudly (`Err`).
/// - If `path` does not exist yet, creates it with `O_WRONLY | O_CREAT | O_EXCL`.
pub fn write_chunk_file_append_only(
    path: &Path,
    content: &str,
) -> Result<bool> {
    use std::io::Write;

    if path.exists() {
        let existing = std::fs::read(path).with_context(|| {
            format!(
                "Failed to read existing .ctbnodes chunk {}",
                path.display()
            )
        })?;
        if existing == content.as_bytes() {
            return Ok(false);
        }
        ensure!(
            content.as_bytes().starts_with(&existing),
            "CRITICAL: Append-only invariant violated for {}: existing file content ({} bytes) \
             is not a strict byte prefix of new content ({} bytes). Existing .ctbnodes entries must never be modified or truncated!",
            path.display(),
            existing.len(),
            content.len()
        );
        let tail = content.as_bytes().get(existing.len()..).context(
            "Existing chunk exceeds verified replacement byte length",
        )?;
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(path)
            .with_context(|| {
                format!("Failed to open {} in O_APPEND mode", path.display())
            })?;
        file.write_all(tail).with_context(|| {
            format!(
                "Failed to append {} bytes to {}",
                tail.len(),
                path.display()
            )
        })?;
        return Ok(true);
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).with_context(|| {
            format!("Failed to create directory {}", parent.display())
        })?;
    }
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .with_context(|| {
            format!("Failed to create new .ctbnodes chunk {}", path.display())
        })?;
    file.write_all(content.as_bytes()).with_context(|| {
        format!("Failed to write new .ctbnodes chunk {}", path.display())
    })?;
    Ok(true)
}

/// Writes `nodes` into 100-node chunk files (`85899_34592.ctbnodes`, ...) in `graph_dir`
/// using strictly append-only file operations (`O_APPEND` on partial tail chunks, `O_CREAT | O_EXCL`
/// on new chunks, and no-op on unchanged full chunks). Returns the number of files written.
pub fn write_ledger_nodes_to_dir(
    graph_dir: &Path,
    nodes: &[LedgerNode],
) -> Result<usize> {
    std::fs::create_dir_all(graph_dir).with_context(|| {
        format!("Failed to create graph directory {}", graph_dir.display())
    })?;
    let existing_files = list_chunk_files(graph_dir)?;
    let expected_chunks = if nodes.is_empty() {
        0
    } else {
        nodes
            .len()
            .saturating_add(MAX_NODES_PER_CHUNK.saturating_sub(1))
            / MAX_NODES_PER_CHUNK
    };
    ensure!(
        expected_chunks >= existing_files.len(),
        "CRITICAL: Append-only invariant violated in {}: new node list has {} chunks, \
         fewer than the {} existing chunk files on disk!",
        graph_dir.display(),
        expected_chunks,
        existing_files.len()
    );

    let mut files_written = 0usize;
    for chunk in nodes.chunks(MAX_NODES_PER_CHUNK) {
        let Some(first_node) = chunk.first() else {
            continue;
        };
        let file_path = graph_dir.join(format_chunk_rel_path(first_node.id));
        let mut content = String::new();
        for node in chunk {
            content.push_str(&node.format_line());
            content.push('\n');
        }
        if write_chunk_file_append_only(&file_path, &content)? {
            files_written = files_written.saturating_add(1);
        }
    }
    Ok(files_written)
}

/// Synchronizes the append-only high-water shadow ledgers with `nodes`.
pub fn sync_highwater_shadow_ledgers(
    repo_root: &Path,
    nodes: &[LedgerNode],
) -> Result<()> {
    for (label, shadow_dir) in highwater_shadow_dirs(repo_root) {
        std::fs::create_dir_all(&shadow_dir).with_context(|| {
            format!("Failed to create {label} at {}", shadow_dir.display())
        })?;
        if let Ok(existing_files) = list_chunk_files(&shadow_dir) {
            if let Some(last_file) = existing_files.last() {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = std::fs::set_permissions(
                        last_file,
                        std::fs::Permissions::from_mode(0o644),
                    );
                }
            }
        }
        write_ledger_nodes_to_dir(&shadow_dir, nodes)
            .with_context(|| format!("Failed to update {label}"))?;
        if let Ok(updated_files) = list_chunk_files(&shadow_dir) {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                for file in updated_files {
                    let _ = std::fs::set_permissions(
                        &file,
                        std::fs::Permissions::from_mode(0o444),
                    );
                }
            }
        }
    }
    Ok(())
}

/// Loads all existing `LedgerNode` entries from `.ctbnodes` chunk files in `graph_dir`.
///
/// When `graph_dir` is `<repo_root>/assets/data/graph`, this also automatically verifies
/// that the loaded nodes have not been rolled back or mutated relative to the high-water
/// shadow ledgers.
pub fn load_ledger_nodes_from_dir(graph_dir: &Path) -> Result<Vec<LedgerNode>> {
    if let Some(repo_root) = find_repo_root_for_main_graph_dir(graph_dir) {
        let guard = LedgerReadGuard::acquire(&repo_root)?;
        let (ledger_sha, nodes_arc) =
            journal::ensure_ledger_journal_clean_and_load_committed(
                &repo_root, &guard,
            )?;
        for (label, shadow_dir) in highwater_shadow_dirs(&repo_root) {
            if shadow_dir.is_dir() {
                let shadow_sha = journal::compute_ledger_dir_sha256(&shadow_dir)
                    .with_context(|| {
                        format!(
                            "Failed to hash {label} at {}",
                            shadow_dir.display()
                        )
                    })?;
                if shadow_sha != ledger_sha {
                    let shadow_nodes =
                        load_ledger_nodes_from_dir_raw(&shadow_dir)
                            .with_context(|| {
                                format!(
                                    "Failed to load {label} from {}",
                                    shadow_dir.display()
                                )
                            })?;
                    if !shadow_nodes.is_empty() {
                        verify_nodes_append_only_against_baseline(
                            &label,
                            &shadow_nodes,
                            &nodes_arc,
                        )?;
                    }
                }
            }
        }
        drop(guard);
        return Ok(nodes_arc.as_ref().to_vec());
    }
    load_ledger_nodes_from_dir_raw(graph_dir)
}

/// Appends `tok` as an `[atom_dc]` token to `out`, escaping `Dc 203` (`Begin encapsulated binary data`),
/// `escape` (`Dc 255`), and `evaluate_as_atom` (`Dc 616`) with a leading `escape` (`Dc 255`).
pub fn push_atom_dc_with(
    out: &mut Vec<DcShorthand>,
    tok: DcShorthand,
    framing: CtbnbdlFramingDcs,
) {
    if tok == DcShorthand::Short(203)
        || tok == framing.escape
        || tok == framing.evaluate_as_atom
    {
        out.push(framing.escape);
    }
    out.push(tok);
}

/// Consumes an `[atom_dc]` token at `*idx` from `tokens`, unescaping `255 203`, `255 255`, and `255 616`.
#[must_use]
pub fn consume_atom_dc_with(
    tokens: &[DcShorthand],
    idx: &mut usize,
    framing: CtbnbdlFramingDcs,
) -> Option<DcShorthand> {
    let tok = *tokens.get(*idx)?;
    *idx = idx.saturating_add(1);
    if tok == framing.escape {
        let escaped = *tokens.get(*idx)?;
        if escaped == DcShorthand::Short(203)
            || escaped == framing.escape
            || escaped == framing.evaluate_as_atom
        {
            *idx = idx.saturating_add(1);
            Some(escaped)
        } else {
            None
        }
    } else if tok == framing.evaluate_as_atom {
        None
    } else {
        Some(tok)
    }
}

/// Encodes a single `LedgerNode` (`id` + `tokens`) into its `Dc 617 .. Dc 618` token sequence:
/// `617 [atom_dc: l<id>] <escaped body tokens...> 618`.
#[must_use]
pub fn encode_node_definition_tokens_with(
    node: &LedgerNode,
    framing: CtbnbdlFramingDcs,
) -> Vec<DcShorthand> {
    let mut out = Vec::with_capacity(node.tokens.len().saturating_add(4));
    out.push(framing.begin_node);
    push_atom_dc_with(&mut out, DcShorthand::from_global_id(node.id), framing);
    for &tok in &node.tokens {
        if tok == framing.escape || tok == framing.end_node {
            out.push(framing.escape);
        }
        out.push(tok);
    }
    out.push(framing.end_node);
    out
}

fn push_dctf_token(out: &mut Vec<u8>, tok: DcShorthand) -> Result<()> {
    let mut char_buf = [0u8; 24];
    let gid = tok.to_global_id()?;
    let n = encode_dctf_buf(&mut char_buf, gid);
    if let Some(slice) = char_buf.get(..n) {
        out.extend_from_slice(slice);
    }
    Ok(())
}

fn decode_dc_base64_payload(tokens: &[DcShorthand]) -> Result<Vec<u8>> {
    if tokens.is_empty() {
        return Ok(Vec::new());
    }
    let mut vals: Vec<u8> = Vec::with_capacity(tokens.len());
    let mut padding = 0usize;
    for &tok in tokens {
        match tok {
            DcShorthand::Short(n)
                if (BASE64_DIGIT_START..=BASE64_DIGIT_END).contains(&n) =>
            {
                ensure!(
                    padding == 0,
                    "Base64 digit {tok} after padding in Dc 203 encapsulation"
                );
                let digit = u8::try_from(n.saturating_sub(BASE64_DIGIT_START))?;
                vals.push(digit);
            }
            DcShorthand::Short(BASE64_PADDING_DC) => {
                padding = padding.saturating_add(1);
                ensure!(
                    padding <= 2,
                    "Too many base64 padding characters in Dc 203 encapsulation"
                );
            }
            _ => bail!(
                "Unexpected token {tok} inside base64 payload of Dc 203 encapsulation"
            ),
        }
    }

    let total_chars = vals.len().saturating_add(padding);
    ensure!(
        total_chars.is_multiple_of(4) && !vals.is_empty(),
        "Invalid base64 length ({total_chars} chars) in Dc 203 encapsulation"
    );

    let mut out = Vec::with_capacity(total_chars.saturating_div(4).saturating_mul(3));
    let mut idx = 0usize;
    while idx < vals.len() {
        let a = *vals.get(idx).context("Missing base64 char 0")?;
        let b = *vals.get(idx.saturating_add(1)).context("Missing base64 char 1")?;
        let c_opt = vals.get(idx.saturating_add(2)).copied();
        let d_opt = vals.get(idx.saturating_add(3)).copied();

        let b0 = (a << 2) | (b >> 4);
        out.push(b0);
        if let Some(c) = c_opt {
            let b1 = ((b & 0x0F) << 4) | (c >> 2);
            out.push(b1);
            if let Some(d) = d_opt {
                let b2 = ((c & 0x03) << 6) | d;
                out.push(b2);
            }
        }
        idx = idx.saturating_add(4);
    }
    Ok(out)
}

fn encode_dc_base64_u128(mut val: u128, out: &mut Vec<DcShorthand>) {
    if val == 0 {
        out.push(DcShorthand::Short(BASE64_DIGIT_START));
        return;
    }
    let mut digits = Vec::new();
    while val > 0 {
        let rem = u32::try_from(val & 0x3F).unwrap_or(0);
        digits.push(DcShorthand::Short(BASE64_DIGIT_START.saturating_add(rem)));
        val >>= 6;
    }
    for d in digits.into_iter().rev() {
        out.push(d);
    }
}

fn encode_dc_base64_payload(bytes: &[u8], out: &mut Vec<DcShorthand>) {
    let mut idx = 0usize;
    while idx < bytes.len() {
        let b0 = *bytes.get(idx).unwrap_or(&0);
        let b1_opt = bytes.get(idx.saturating_add(1)).copied();
        let b2_opt = bytes.get(idx.saturating_add(2)).copied();

        let c0 = u32::from(b0 >> 2);
        let c1 = u32::from(((b0 & 0x03) << 4) | (b1_opt.unwrap_or(0) >> 4));
        out.push(DcShorthand::Short(BASE64_DIGIT_START.saturating_add(c0)));
        out.push(DcShorthand::Short(BASE64_DIGIT_START.saturating_add(c1)));

        if let Some(b1) = b1_opt {
            let c2 = u32::from(((b1 & 0x0F) << 2) | (b2_opt.unwrap_or(0) >> 6));
            out.push(DcShorthand::Short(BASE64_DIGIT_START.saturating_add(c2)));
            if let Some(b2) = b2_opt {
                let c3 = u32::from(b2 & 0x3F);
                out.push(DcShorthand::Short(
                    BASE64_DIGIT_START.saturating_add(c3),
                ));
            } else {
                out.push(DcShorthand::Short(BASE64_PADDING_DC));
            }
        } else {
            out.push(DcShorthand::Short(BASE64_PADDING_DC));
            out.push(DcShorthand::Short(BASE64_PADDING_DC));
        }
        idx = idx.saturating_add(3);
    }
}

fn try_parse_textual_encapsulation(
    tokens: &[DcShorthand],
    start: usize,
) -> Result<Option<(Vec<u8>, usize)>> {
    if tokens.get(start) != Some(&DcShorthand::Short(203)) {
        return Ok(None);
    }
    let mut pos = start.saturating_add(1);
    let mut expected_len: Option<u128> = None;

    match tokens.get(pos) {
        Some(&DcShorthand::Short(6)) => {
            pos = pos.saturating_add(1);
            if tokens.get(pos) == Some(&DcShorthand::Format(199)) {
                pos = pos.saturating_add(1);
            }
            let mut len_val = 0u128;
            let mut digit_count = 0usize;
            while let Some(&DcShorthand::Short(n)) = tokens.get(pos) {
                if (BASE64_DIGIT_START..=BASE64_DIGIT_END).contains(&n) {
                    let d = u128::from(n.saturating_sub(BASE64_DIGIT_START));
                    len_val = len_val
                        .checked_mul(64)
                        .and_then(|v| v.checked_add(d))
                        .context("Length overflow in Dc 203 encapsulation")?;
                    digit_count = digit_count.saturating_add(1);
                    pos = pos.saturating_add(1);
                } else {
                    break;
                }
            }
            ensure!(
                digit_count > 0 && tokens.get(pos) == Some(&DcShorthand::Short(7)),
                "Malformed length block in Dc 203 textual encapsulation"
            );
            pos = pos.saturating_add(1);
            expected_len = Some(len_val);
        }
        Some(&DcShorthand::Short(n))
            if (BASE64_DIGIT_START..=BASE64_DIGIT_END).contains(&n)
                || n == BASE64_PADDING_DC
                || n == 204 => {}
        _ => return Ok(None),
    }

    let payload_start = pos;
    while let Some(&tok) = tokens.get(pos) {
        if tok == DcShorthand::Short(204) {
            let payload_slice = tokens
                .get(payload_start..pos)
                .context("Invalid textual encapsulation slice")?;
            let raw_bytes = decode_dc_base64_payload(payload_slice)?;
            if let Some(exp) = expected_len {
                ensure!(
                    u128::try_from(raw_bytes.len())? == exp,
                    "Dc 203 textual encapsulation declared length {exp} bytes, decoded {} bytes",
                    raw_bytes.len()
                );
            }
            return Ok(Some((raw_bytes, pos.saturating_add(1))));
        }
        pos = pos.saturating_add(1);
    }

    bail!("Unterminated Dc 203 textual encapsulation (missing Dc 204)");
}

/// Encodes a single `LedgerNode` into `DcMixed2` checksummed-section bytes:
/// `543 617 [atom_dc: l<id>] <escaped body tokens...> 618 [<SectionTimestampNodeCreationTime> 390 f487 6 f199 [11?] <base64> 7 391]? 544 545 <SHA-256 checksum>`.
pub fn encode_node_definition_dcmm2_bytes_with(
    node: &LedgerNode,
    framing: CtbnbdlFramingDcs,
) -> Result<Vec<u8>> {
    use crate::sha2::{Digest, Sha256};

    let mut out = Vec::with_capacity(
        node.tokens
            .len()
            .saturating_add(16)
            .saturating_mul(4)
            .saturating_add(80),
    );
    push_dctf_token(&mut out, framing.begin_checksummed_section)?;
    let section_body_start = out.len();

    push_dctf_token(&mut out, framing.begin_node)?;

    let mut id_tokens = Vec::with_capacity(2);
    push_atom_dc_with(
        &mut id_tokens,
        DcShorthand::from_global_id(node.id),
        framing,
    );
    for tok in id_tokens {
        push_dctf_token(&mut out, tok)?;
    }

    let mut i = 0usize;
    while let Some(&tok) = node.tokens.get(i) {
        if tok == framing.escape || tok == framing.end_node {
            push_dctf_token(&mut out, framing.escape)?;
            push_dctf_token(&mut out, tok)?;
            i = i.saturating_add(1);
        } else if tok == DcShorthand::Short(203)
            && let Some((raw_bytes, next_i)) =
                try_parse_textual_encapsulation(&node.tokens, i)?
        {
            out.extend_from_slice(&DC_203_DCTF_BYTES);
            out.push(DCMM_TYPE_BINARY_METADATA);
            let payload_len = u128::try_from(raw_bytes.len())?;
            out.extend_from_slice(&payload_len.to_be_bytes());
            out.extend_from_slice(&0u128.to_be_bytes());
            out.extend_from_slice(&raw_bytes);
            i = next_i;
        } else {
            push_dctf_token(&mut out, tok)?;
            i = i.saturating_add(1);
        }
    }

    push_dctf_token(&mut out, framing.end_node)?;

    if let Some(ts_ns) = node.created_ns {
        let ts_dc = framing.section_node_creation_time.context(
            "Cannot encode node creation timestamp in .ctbnbdl without SectionTimestampNodeCreationTime in framing Dcs",
        )?;
        push_dctf_token(&mut out, ts_dc)?;
        push_dctf_token(&mut out, framing.begin_time_moment)?;
        push_dctf_token(&mut out, framing.ns_since_epoch_format)?;
        push_dctf_token(&mut out, DcShorthand::Short(6))?;
        push_dctf_token(&mut out, DcShorthand::Format(199))?;
        if ts_ns < 0 {
            push_dctf_token(&mut out, DcShorthand::Short(11))?;
        }
        let mut digits = Vec::with_capacity(24);
        encode_dc_base64_u128(ts_ns.unsigned_abs(), &mut digits);
        for d in digits {
            push_dctf_token(&mut out, d)?;
        }
        push_dctf_token(&mut out, DcShorthand::Short(7))?;
        push_dctf_token(&mut out, framing.end_time_moment)?;
    }

    let section_body_end = out.len();
    let section_body = out
        .get(section_body_start..section_body_end)
        .context("Invalid checksummed section body slice")?;
    let digest: [u8; 32] = Sha256::digest(section_body).into();

    push_dctf_token(&mut out, framing.end_checksummed_section)?;
    push_dctf_token(&mut out, framing.section_digest)?;

    let mut meta_bytes = Vec::with_capacity(8);
    push_dctf_token(&mut meta_bytes, framing.sha256_format)?;
    out.extend_from_slice(&DC_203_DCTF_BYTES);
    out.push(DCMM_TYPE_BINARY_METADATA);
    out.extend_from_slice(&32u128.to_be_bytes());
    out.extend_from_slice(&u128::try_from(meta_bytes.len())?.to_be_bytes());
    out.extend_from_slice(&meta_bytes);
    out.extend_from_slice(&digest);

    Ok(out)
}

fn encode_ctbnbdl_header_bytes(
    most_recent_node_id: Option<u128>,
    framing: CtbnbdlFramingDcs,
) -> Result<Vec<u8>> {
    let mut header = Vec::with_capacity(
        CTBNBDL_MAGIC
            .len()
            .saturating_add(DCMM_HEADER.len())
            .saturating_add(16),
    );
    header.extend_from_slice(&CTBNBDL_MAGIC);
    header.extend_from_slice(&DCMM_HEADER);
    if let Some(wm_id) = most_recent_node_id {
        ensure!(
            (SYSTEM_REGION_START..=SYSTEM_REGION_END).contains(&wm_id),
            "System graph most_recent node ID l{wm_id} is outside the System block ({SYSTEM_REGION_START}..={SYSTEM_REGION_END})"
        );
        let mut wm_tokens = Vec::with_capacity(3);
        wm_tokens.push(DC_SYSTEM_BLOCK_MOST_RECENT_NODE_ID);
        push_atom_dc_with(
            &mut wm_tokens,
            DcShorthand::from_global_id(wm_id),
            framing,
        );
        let mut char_buf = [0u8; 24];
        for tok in wm_tokens {
            let gid = tok.to_global_id()?;
            let n = encode_dctf_buf(&mut char_buf, gid);
            if let Some(slice) = char_buf.get(..n) {
                header.extend_from_slice(slice);
            }
        }
    }
    Ok(header)
}

/// Compiles a slice of `LedgerNode` entries into a `.ctbnbdl` (`CtbNodeBundle2`) binary buffer:
/// 8-byte `CTBNBDL\x02` magic + 21-byte `DcMixed2` header + optional `641 l<max_node_id>` most_recent
/// + `DcTF` encoded `543 617 .. 618 [<ts>]? 544 545 <checksum>` checksummed node sections.
pub fn encode_nodes_to_ctbnbdl_bytes_with(
    nodes: &[LedgerNode],
    framing: CtbnbdlFramingDcs,
) -> Result<Vec<u8>> {
    let most_recent_node_id = nodes.iter().map(|n| n.id).max();
    let header_bytes = encode_ctbnbdl_header_bytes(most_recent_node_id, framing)?;
    let mut bytes = Vec::with_capacity(
        header_bytes
            .len()
            .saturating_add(nodes.len().saturating_mul(96)),
    );
    bytes.extend_from_slice(&header_bytes);
    for node in nodes {
        let node_bytes = encode_node_definition_dcmm2_bytes_with(node, framing)?;
        bytes.extend_from_slice(&node_bytes);
    }
    Ok(bytes)
}

#[derive(Debug)]
enum StreamItem<'a> {
    Token(DcShorthand),
    Binary {
        meta_tokens: Vec<DcShorthand>,
        payload_len_u128: u128,
        bin_payload: &'a [u8],
    },
}

fn read_stream_item<'a>(
    payload: &'a [u8],
    pos: usize,
    is_v2: bool,
    escaped: bool,
) -> Result<(StreamItem<'a>, usize)> {
    let rem = payload
        .get(pos..)
        .context("Stream cursor out of bounds in .ctbnbdl")?;
    let dc203_bytes: &[u8] = if is_v2 {
        &DC_203_DCTF_BYTES
    } else {
        &DC_203_UTF8E128_BYTES
    };
    if !escaped
        && let Some(after_dc203) = rem.strip_prefix(dc203_bytes)
        && let Some(&type_byte) = after_dc203.first()
        && matches!(
            type_byte,
            DCMM_TYPE_BINARY
                | DCMM_TYPE_LEGACY_TEXT
                | DCMM_TYPE_BINARY_METADATA
                | DCMM_TYPE_BINARY_SHA256
                | DCMM_TYPE_LEGACY_SHA256
        )
    {
        let size_bytes: [u8; 16] = after_dc203
            .get(1..17)
            .context("Truncated DcMixed encapsulation size in .ctbnbdl")?
            .try_into()?;
        let payload_len_u128 = u128::from_be_bytes(size_bytes);
        let payload_len = usize::try_from(payload_len_u128)
            .context("DcMixed encapsulation payload size overflow in .ctbnbdl")?;

        let mut cursor = 17usize;
        let mut meta_bytes: &[u8] = &[];
        if type_byte == DCMM_TYPE_BINARY_METADATA {
            let meta_len_bytes: [u8; 16] = after_dc203
                .get(cursor..cursor.saturating_add(16))
                .context("Truncated DcMixed metadata length in .ctbnbdl")?
                .try_into()?;
            let meta_len =
                usize::try_from(u128::from_be_bytes(meta_len_bytes))
                    .context("DcMixed metadata length overflow in .ctbnbdl")?;
            cursor = cursor.saturating_add(16);
            let meta_end = cursor
                .checked_add(meta_len)
                .context("DcMixed metadata offset overflow in .ctbnbdl")?;
            meta_bytes = after_dc203
                .get(cursor..meta_end)
                .context("Truncated DcMixed metadata in .ctbnbdl")?;
            cursor = meta_end;
        } else if type_byte == DCMM_TYPE_BINARY_SHA256
            || type_byte == DCMM_TYPE_LEGACY_SHA256
        {
            let sha_end = cursor.saturating_add(32);
            ensure!(
                after_dc203.len() >= sha_end,
                "Truncated DcMixed SHA-256 checksum in .ctbnbdl"
            );
            cursor = sha_end;
        }

        let payload_end = cursor
            .checked_add(payload_len)
            .context("DcMixed payload offset overflow in .ctbnbdl")?;
        let bin_payload = after_dc203
            .get(cursor..payload_end)
            .context("Truncated DcMixed binary payload in .ctbnbdl")?;

        let mut meta_tokens = Vec::new();
        let mut meta_rem = meta_bytes;
        while !meta_rem.is_empty() {
            let decoded = if is_v2 {
                decode_dctf_buf(meta_rem)
            } else {
                decode_utf_8e_128_buf(meta_rem)
            };
            let Some((cp, consumed)) = decoded else {
                bail!(
                    "Invalid character sequence in .ctbnbdl encapsulation metadata"
                );
            };
            meta_tokens.push(DcShorthand::from_global_id(cp));
            // Reason for fallback: consumed <= meta_rem.len() is guaranteed by decode_dctf_buf / decode_utf_8e_128_buf.
            meta_rem = meta_rem.get(consumed..).unwrap_or_default();
        }

        let next_pos = pos
            .checked_add(dc203_bytes.len())
            .and_then(|p| p.checked_add(payload_end))
            .context("Byte offset overflow in .ctbnbdl binary encapsulation")?;
        return Ok((
            StreamItem::Binary {
                meta_tokens,
                payload_len_u128,
                bin_payload,
            },
            next_pos,
        ));
    }

    let decoded = if is_v2 {
        decode_dctf_buf(rem)
    } else {
        decode_utf_8e_128_buf(rem)
    };
    let Some((codepoint, consumed)) = decoded else {
        bail!("Invalid character sequence in .ctbnbdl stream at offset {pos}");
    };
    let next_pos = pos
        .checked_add(consumed)
        .context("Byte offset overflow in .ctbnbdl character stream")?;
    Ok((StreamItem::Token(DcShorthand::from_global_id(codepoint)), next_pos))
}

fn push_binary_item_as_textual_tokens(
    meta_tokens: &[DcShorthand],
    payload_len_u128: u128,
    bin_payload: &[u8],
    out: &mut Vec<DcShorthand>,
) {
    out.push(DcShorthand::Short(203));
    out.extend_from_slice(meta_tokens);
    out.push(DcShorthand::Short(6));
    out.push(DcShorthand::Format(199));
    encode_dc_base64_u128(payload_len_u128, out);
    out.push(DcShorthand::Short(7));
    encode_dc_base64_payload(bin_payload, out);
    out.push(DcShorthand::Short(204));
}

fn read_atom_dc_from_bytes(
    payload: &[u8],
    pos: &mut usize,
    is_v2: bool,
    framing: CtbnbdlFramingDcs,
) -> Result<DcShorthand> {
    let (item, next_pos) = read_stream_item(payload, *pos, is_v2, false)?;
    let StreamItem::Token(tok) = item else {
        bail!("Unexpected binary encapsulation in [atom_dc] position in .ctbnbdl");
    };
    *pos = next_pos;
    if tok == framing.escape {
        let (esc_item, after_esc) = read_stream_item(payload, *pos, is_v2, true)?;
        let StreamItem::Token(escaped) = esc_item else {
            bail!("Unexpected binary encapsulation after escape in [atom_dc]");
        };
        ensure!(
            escaped == DcShorthand::Short(203)
                || escaped == framing.escape
                || escaped == framing.evaluate_as_atom,
            "Invalid escaped [atom_dc] token {escaped} in .ctbnbdl"
        );
        *pos = after_esc;
        Ok(escaped)
    } else if tok == framing.evaluate_as_atom {
        bail!("Unexpected unescaped EvaluateValueAsAtom in .ctbnbdl [atom_dc]");
    } else {
        Ok(tok)
    }
}

fn parse_bare_node_at(
    payload: &[u8],
    pos: &mut usize,
    is_v2: bool,
    framing: CtbnbdlFramingDcs,
) -> Result<(u128, Vec<DcShorthand>)> {
    let (open_item, after_open) = read_stream_item(payload, *pos, is_v2, false)?;
    let StreamItem::Token(open_tok) = open_item else {
        bail!(
            "Expected {} (Begin graph node definition), found binary encapsulation",
            framing.begin_node
        );
    };
    ensure!(
        open_tok == framing.begin_node,
        "Expected {} (Begin graph node definition), found {open_tok}",
        framing.begin_node
    );
    *pos = after_open;
    let id_sh = read_atom_dc_from_bytes(payload, pos, is_v2, framing)
        .context("Invalid or missing [atom_dc] node ID after BeginGraphNodeDefinition")?;
    let id = id_sh.to_global_id()?;

    let mut body_tokens = Vec::new();
    let mut closed = false;
    while *pos < payload.len() {
        let (item, next_pos) = read_stream_item(payload, *pos, is_v2, false)?;
        *pos = next_pos;
        match item {
            StreamItem::Token(tok) => {
                if tok == framing.end_node {
                    closed = true;
                    break;
                }
                if tok == framing.escape {
                    let (esc_item, after_esc) =
                        read_stream_item(payload, *pos, is_v2, true).context(
                            "Trailing escape token at end of .ctbnbdl node body",
                        )?;
                    let StreamItem::Token(escaped) = esc_item else {
                        bail!("Unexpected binary chunk after escape in .ctbnbdl node body");
                    };
                    ensure!(
                        escaped == framing.escape || escaped == framing.end_node,
                        "Unexpected escaped token {escaped} after {} in .ctbnbdl node body",
                        framing.escape
                    );
                    *pos = after_esc;
                    body_tokens.push(escaped);
                } else {
                    body_tokens.push(tok);
                }
            }
            StreamItem::Binary {
                meta_tokens,
                payload_len_u128,
                bin_payload,
            } => {
                push_binary_item_as_textual_tokens(
                    &meta_tokens,
                    payload_len_u128,
                    bin_payload,
                    &mut body_tokens,
                );
            }
        }
    }
    ensure!(
        closed,
        "Unterminated node definition for l{id} (missing {})",
        framing.end_node
    );
    Ok((id, body_tokens))
}

fn parse_signed_dc_i128_at(
    payload: &[u8],
    pos: &mut usize,
    is_v2: bool,
) -> Result<i128> {
    let (n_open, p1) = read_stream_item(payload, *pos, is_v2, false)?;
    ensure!(
        matches!(n_open, StreamItem::Token(DcShorthand::Short(6))),
        "Expected Dc 6 (Begin number) in node creation timestamp"
    );
    let (fmt_tok, p2) = read_stream_item(payload, p1, is_v2, false)?;
    ensure!(
        matches!(fmt_tok, StreamItem::Token(DcShorthand::Format(199))),
        "Expected f199 (Integer) in node creation timestamp"
    );
    *pos = p2;

    let mut is_negative = false;
    let (peek_item, p_after_sign) = read_stream_item(payload, *pos, is_v2, false)?;
    if matches!(peek_item, StreamItem::Token(DcShorthand::Short(11))) {
        is_negative = true;
        *pos = p_after_sign;
    }

    let mut mag: u128 = 0;
    let mut digit_count = 0usize;
    let mut closed = false;
    while *pos < payload.len() {
        let (item, next_pos) = read_stream_item(payload, *pos, is_v2, false)?;
        *pos = next_pos;
        match item {
            StreamItem::Token(DcShorthand::Short(7)) => {
                closed = true;
                break;
            }
            StreamItem::Token(DcShorthand::Short(n))
                if (BASE64_DIGIT_START..=BASE64_DIGIT_END).contains(&n) =>
            {
                let digit = u128::from(n.saturating_sub(BASE64_DIGIT_START));
                mag = mag
                    .checked_mul(64)
                    .and_then(|v| v.checked_add(digit))
                    .context("Integer magnitude overflow in node creation timestamp")?;
                digit_count = digit_count.saturating_add(1);
            }
            other => {
                bail!(
                    "Unexpected token {other:?} inside Dc 6..7 integer in node creation timestamp"
                );
            }
        }
    }
    ensure!(
        closed && digit_count > 0,
        "Malformed or unterminated Dc 6..7 integer in node creation timestamp"
    );

    if is_negative {
        let min_abs = 1u128 << 127;
        if mag == min_abs {
            Ok(i128::MIN)
        } else {
            let signed = i128::try_from(mag)
                .context("Negative timestamp magnitude exceeds i128::MIN")?;
            signed
                .checked_neg()
                .context("Negation overflow in node creation timestamp")
        }
    } else {
        i128::try_from(mag)
            .context("Positive timestamp magnitude exceeds i128::MAX")
    }
}

fn parse_node_record_at(
    payload: &[u8],
    pos: &mut usize,
    is_v2: bool,
    framing: CtbnbdlFramingDcs,
) -> Result<LedgerNode> {
    use crate::sha2::{Digest, Sha256};

    let (first_item, after_first) = read_stream_item(payload, *pos, is_v2, false)?;
    let StreamItem::Token(first_tok) = first_item else {
        bail!("Expected section opener or node opener in .ctbnbdl, found binary encapsulation");
    };

    if first_tok == framing.begin_node {
        let (id, tokens) = parse_bare_node_at(payload, pos, is_v2, framing)?;
        return Ok(LedgerNode {
            id,
            tokens,
            comment: String::new(),
            created_ns: None,
        });
    }

    ensure!(
        first_tok == framing.begin_checksummed_section,
        "Expected {} (Begin checksummed section) or {} (Begin graph node definition), found {first_tok}",
        framing.begin_checksummed_section,
        framing.begin_node
    );
    let section_body_start = after_first;
    *pos = after_first;

    let (id, tokens) = parse_bare_node_at(payload, pos, is_v2, framing)?;

    let before_meta_or_end = *pos;
    let (after_node_item, after_meta_tok_pos) =
        read_stream_item(payload, before_meta_or_end, is_v2, false)
            .with_context(|| {
                format!(
                    "Truncated checksummed section after node l{id} (missing {})",
                    framing.end_checksummed_section
                )
            })?;
    let StreamItem::Token(after_node_tok) = after_node_item else {
        bail!("Unexpected binary chunk after node l{id} inside checksummed section");
    };

    let (created_ns, section_body_end) =
        if after_node_tok == framing.end_checksummed_section {
            *pos = after_meta_tok_pos;
            (None, before_meta_or_end)
        } else {
            if let Some(expected_ts_dc) = framing.section_node_creation_time {
                ensure!(
                    after_node_tok == expected_ts_dc,
                    "Expected {} (SectionTimestampNodeCreationTime) or {} after node l{id}, found {after_node_tok}",
                    expected_ts_dc,
                    framing.end_checksummed_section
                );
            }
            *pos = after_meta_tok_pos;
            let (tm_open, after_tm_open) =
                read_stream_item(payload, *pos, is_v2, false)?;
            ensure!(
                matches!(tm_open, StreamItem::Token(t) if t == framing.begin_time_moment),
                "Expected {} (Begin time moment) after SectionTimestampNodeCreationTime for l{id}",
                framing.begin_time_moment
            );
            let (tm_fmt, after_tm_fmt) =
                read_stream_item(payload, after_tm_open, is_v2, false)?;
            ensure!(
                matches!(tm_fmt, StreamItem::Token(t) if t == framing.ns_since_epoch_format),
                "Expected {} (NsSinceEpoch) in creation timestamp for l{id}",
                framing.ns_since_epoch_format
            );
            *pos = after_tm_fmt;
            let ts_ns = parse_signed_dc_i128_at(payload, pos, is_v2)?;
            let (tm_close, after_tm_close) =
                read_stream_item(payload, *pos, is_v2, false)?;
            ensure!(
                matches!(tm_close, StreamItem::Token(t) if t == framing.end_time_moment),
                "Expected {} (End time moment) after creation timestamp integer for l{id}",
                framing.end_time_moment
            );
            let sec_end = after_tm_close;
            let (sec_close, after_sec_close) =
                read_stream_item(payload, sec_end, is_v2, false)?;
            ensure!(
                matches!(sec_close, StreamItem::Token(t) if t == framing.end_checksummed_section),
                "Expected {} (End checksummed section) after creation timestamp for l{id}",
                framing.end_checksummed_section
            );
            *pos = after_sec_close;
            (Some(ts_ns), sec_end)
        };

    let (digest_tag_item, after_digest_tag) =
        read_stream_item(payload, *pos, is_v2, false).with_context(|| {
            format!(
                "Missing {} (Section digest) after checksummed section for l{id}",
                framing.section_digest
            )
        })?;
    ensure!(
        matches!(digest_tag_item, StreamItem::Token(t) if t == framing.section_digest),
        "Expected {} (Section digest) after {} for l{id}",
        framing.section_digest,
        framing.end_checksummed_section
    );
    *pos = after_digest_tag;

    let (ck_item, after_ck) = read_stream_item(payload, *pos, is_v2, false)
        .with_context(|| {
            format!("Missing [checksum] payload after SectionDigest for l{id}")
        })?;
    *pos = after_ck;

    let expected_digest_bytes: &[u8] = match ck_item {
        StreamItem::Binary {
            meta_tokens,
            bin_payload,
            ..
        } => {
            ensure!(
                meta_tokens.as_slice() == [framing.sha256_format]
                    && bin_payload.len() == 32,
                "Expected 32-byte {} checksum for node l{id}, found meta={meta_tokens:?} len={}",
                framing.sha256_format,
                bin_payload.len()
            );
            bin_payload
        }
        StreamItem::Token(DcShorthand::Short(203)) => {
            // Also accept textual Dc 203 [sha256_format] [6 f199 ... 7]? <base64> 204 checksums
            let mut ck_tokens = vec![DcShorthand::Short(203)];
            let mut closed_204 = false;
            while *pos < payload.len() {
                let (it, np) = read_stream_item(payload, *pos, is_v2, false)?;
                *pos = np;
                let StreamItem::Token(t) = it else {
                    bail!("Unexpected binary chunk inside textual checksum for l{id}");
                };
                ck_tokens.push(t);
                if t == DcShorthand::Short(204) {
                    closed_204 = true;
                    break;
                }
            }
            ensure!(closed_204, "Unterminated textual checksum for l{id}");
            ensure!(
                ck_tokens.get(1) == Some(&framing.sha256_format),
                "Unsupported textual checksum format for l{id}"
            );
            let mut synth = Vec::with_capacity(ck_tokens.len());
            synth.push(DcShorthand::Short(203));
            if let Some(rest) = ck_tokens.get(2..) {
                synth.extend_from_slice(rest);
            }
            let (decoded_ck, consumed_ck) =
                try_parse_textual_encapsulation(&synth, 0)?
                    .context("Malformed textual checksum encapsulation")?;
            ensure!(
                consumed_ck == synth.len() && decoded_ck.len() == 32,
                "Expected 32-byte textual SHA-256 digest for l{id}"
            );
            let section_slice = payload
                .get(section_body_start..section_body_end)
                .context("Invalid checksummed section byte span")?;
            let actual_digest: [u8; 32] = Sha256::digest(section_slice).into();
            ensure!(
                actual_digest.as_slice() == decoded_ck.as_slice(),
                "Checksum mismatch in .ctbnbdl section for node l{id}"
            );
            return Ok(LedgerNode {
                id,
                tokens,
                comment: String::new(),
                created_ns,
            });
        }
        StreamItem::Token(other) => {
            bail!(
                "Expected Dc 203 [checksum] after SectionDigest for l{id}, found {other}"
            );
        }
    };

    let section_slice = payload
        .get(section_body_start..section_body_end)
        .context("Invalid checksummed section byte span")?;
    let actual_digest: [u8; 32] = Sha256::digest(section_slice).into();
    ensure!(
        actual_digest.as_slice() == expected_digest_bytes,
        "Checksum mismatch in .ctbnbdl section for node l{id}"
    );

    Ok(LedgerNode {
        id,
        tokens,
        comment: String::new(),
        created_ns,
    })
}

fn find_next_record_resync_offset(
    payload: &[u8],
    from_pos: usize,
    is_v2: bool,
    framing: CtbnbdlFramingDcs,
    checksummed_only: bool,
) -> usize {
    if !is_v2 || from_pos >= payload.len() {
        return payload.len();
    }
    let mut sec_marker = Vec::with_capacity(8);
    if push_dctf_token(&mut sec_marker, framing.begin_checksummed_section)
        .is_err()
        || sec_marker.is_empty()
    {
        return payload.len();
    }
    let mut node_marker = Vec::with_capacity(8);
    if !checksummed_only {
        let _ = push_dctf_token(&mut node_marker, framing.begin_node);
    }

    let mut cur = from_pos;
    while cur < payload.len() {
        // Reason for fallback: cur < payload.len() guarantees cur.. is in-bounds.
        let tail = payload.get(cur..).unwrap_or_default();
        if tail.starts_with(&sec_marker)
            || (!node_marker.is_empty() && tail.starts_with(&node_marker))
        {
            return cur;
        }
        cur = cur.saturating_add(1);
    }
    payload.len()
}

fn decode_ctbnbdl_bundle_inner(
    bytes: &[u8],
    framing: CtbnbdlFramingDcs,
    resilient: bool,
) -> Result<(Option<u128>, Vec<LedgerNode>, Vec<String>)> {
    let header_len = CTBNBDL_MAGIC.len().saturating_add(DCMM_HEADER.len());
    ensure!(
        bytes.len() >= header_len,
        ".ctbnbdl buffer is shorter than the {header_len}-byte header ({} bytes)",
        bytes.len()
    );
    // Reason for fallback: bytes.len() >= header_len was just verified above.
    let magic_part = bytes.get(..CTBNBDL_MAGIC.len()).unwrap_or_default();
    // Reason for fallback: bytes.len() >= header_len was just verified above.
    let dcmm_part = bytes
        .get(CTBNBDL_MAGIC.len()..header_len)
        .unwrap_or_default();
    let is_v2 = if magic_part == CTBNBDL_MAGIC {
        ensure!(
            dcmm_part == DCMM_HEADER,
            "Invalid DcMixed2 header in v2 .ctbnbdl buffer"
        );
        true
    } else if magic_part == CTBNBDL_MAGIC_V1 {
        ensure!(
            dcmm_part == DCMM_HEADER_V1,
            "Invalid DcMixed1 header in v1 .ctbnbdl buffer"
        );
        false
    } else {
        bail!(
            "Invalid .ctbnbdl magic bytes: expected {CTBNBDL_MAGIC:?} or {CTBNBDL_MAGIC_V1:?}, found {magic_part:?}"
        );
    };

    // Reason for fallback: bytes.len() >= header_len was verified above.
    let payload = bytes.get(header_len..).unwrap_or_default();
    let mut pos = 0usize;
    let mut most_recent_node_id = None;
    let mut warnings = Vec::new();

    if pos < payload.len() {
        if let Ok((StreamItem::Token(DC_SYSTEM_BLOCK_MOST_RECENT_NODE_ID), after_641)) =
            read_stream_item(payload, pos, is_v2, false)
        {
            let mut wm_pos = after_641;
            match read_atom_dc_from_bytes(payload, &mut wm_pos, is_v2, framing)
                .and_then(|wm_sh| wm_sh.to_global_id())
            {
                Ok(wm_id)
                    if (SYSTEM_REGION_START..=SYSTEM_REGION_END)
                        .contains(&wm_id) =>
                {
                    most_recent_node_id = Some(wm_id);
                    pos = wm_pos;
                }
                Ok(wm_id) => {
                    let msg = format!(
                        "System graph most_recent node ID l{wm_id} in .ctbnbdl is outside the System block ({SYSTEM_REGION_START}..={SYSTEM_REGION_END})"
                    );
                    if !resilient {
                        bail!("{msg}");
                    }
                    warnings.push(msg);
                    pos = find_next_record_resync_offset(
                        payload,
                        after_641,
                        is_v2,
                        framing,
                        false,
                    );
                }
                Err(err) => {
                    if !resilient {
                        return Err(err.context(
                            "Invalid or missing [atom_dc] node ID after Dc 641 in .ctbnbdl",
                        ));
                    }
                    warnings.push(format!(
                        "Corrupted Dc 641 header pin at byte offset {}: {err:#}",
                        header_len.saturating_add(pos)
                    ));
                    pos = find_next_record_resync_offset(
                        payload,
                        after_641,
                        is_v2,
                        framing,
                        false,
                    );
                }
            }
        }
    }

    let mut sec_marker = Vec::with_capacity(8);
    if is_v2 {
        let _ = push_dctf_token(
            &mut sec_marker,
            framing.begin_checksummed_section,
        );
    }
    let mut saw_checksummed_section = false;
    let mut nodes = Vec::new();
    while pos < payload.len() {
        let rec_start = pos;
        // Reason for fallback: rec_start < payload.len() guarantees rec_start.. is in-bounds.
        let starts_with_sec = !sec_marker.is_empty()
            && payload
                .get(rec_start..)
                .unwrap_or_default()
                .starts_with(&sec_marker);
        if starts_with_sec {
            saw_checksummed_section = true;
        }
        match parse_node_record_at(payload, &mut pos, is_v2, framing) {
            Ok(node) => nodes.push(node),
            Err(err) => {
                if !resilient {
                    return Err(err);
                }
                warnings.push(format!(
                    "Quarantined corrupted .ctbnbdl node section at byte offset {}: {err:#}",
                    header_len.saturating_add(rec_start)
                ));
                pos = find_next_record_resync_offset(
                    payload,
                    rec_start.saturating_add(1),
                    is_v2,
                    framing,
                    saw_checksummed_section,
                );
            }
        }
    }

    Ok((most_recent_node_id, nodes, warnings))
}

/// Decodes a `.ctbnbdl` (`CtbNodeBundle1` or `CtbNodeBundle2`) binary buffer into its optional
/// leading `Dc 641` system block most recent node ID and its list of `LedgerNode` entries,
/// strictly verifying every `Dc 543..544 545` section digest.
pub fn decode_ctbnbdl_bundle_with(
    bytes: &[u8],
    framing: CtbnbdlFramingDcs,
) -> Result<(Option<u128>, Vec<LedgerNode>)> {
    let (most_recent_node_id, nodes, _) =
        decode_ctbnbdl_bundle_inner(bytes, framing, false)?;
    Ok((most_recent_node_id, nodes))
}

/// Resiliently decodes a `.ctbnbdl` (`CtbNodeBundle1` or `CtbNodeBundle2`) binary buffer,
/// quarantining any corrupted or checksum-mismatched node section while recovering all
/// remaining valid nodes and returning diagnostic warnings for every skipped section.
pub fn decode_ctbnbdl_bundle_resilient_with(
    bytes: &[u8],
    framing: CtbnbdlFramingDcs,
) -> Result<(Option<u128>, Vec<LedgerNode>, Vec<String>)> {
    decode_ctbnbdl_bundle_inner(bytes, framing, true)
}

/// Decodes a `.ctbnbdl` (`CtbNodeBundle1` or `CtbNodeBundle2`) binary buffer back into a list of
/// `LedgerNode` entries (with empty comments, as `.ctbnbdl` stores only the semantic Dc stream).
pub fn decode_nodes_from_ctbnbdl_bytes_with(
    bytes: &[u8],
    framing: CtbnbdlFramingDcs,
) -> Result<Vec<LedgerNode>> {
    let (_most_recent, nodes) = decode_ctbnbdl_bundle_with(bytes, framing)?;
    Ok(nodes)
}

/// Parses a `.ctbnodes` file on disk and compiles its node definitions into `.ctbnbdl` bytes.
pub fn compile_ctbnodes_file_to_ctbnbdl_bytes(
    path: &Path,
    framing: CtbnbdlFramingDcs,
) -> Result<Vec<u8>> {
    let nodes = load_ledger_nodes_from_file(path)?;
    encode_nodes_to_ctbnbdl_bytes_with(&nodes, framing)
}

/// Splits a slice of `LedgerNode` entries into one or more `.ctbnbdl` bundle buffers,
/// each paired with its starting node ID (`(first_node_id, bundle_bytes)`), stamping each
/// bundle with the provided `most_recent_node_id` (`Dc 641 l<most_recent_node_id>`).
pub fn encode_nodes_to_ctbnbdl_bundles_with_limits_and_most_recent(
    nodes: &[LedgerNode],
    framing: CtbnbdlFramingDcs,
    max_nodes_per_bundle: usize,
    max_bytes_per_bundle: usize,
    most_recent_node_id: Option<u128>,
) -> Result<Vec<(u128, Vec<u8>)>> {
    ensure!(
        max_nodes_per_bundle > 0,
        "max_nodes_per_bundle must be greater than 0"
    );
    let header_bytes = encode_ctbnbdl_header_bytes(most_recent_node_id, framing)?;
    let header_len = header_bytes.len();
    ensure!(
        max_bytes_per_bundle > header_len,
        "max_bytes_per_bundle ({max_bytes_per_bundle}) must exceed the {header_len}-byte .ctbnbdl header"
    );
    if nodes.is_empty() {
        return Ok(Vec::new());
    }

    let max_nodes_u128 =
        u128::try_from(max_nodes_per_bundle).unwrap_or(1_000_000);
    let mut bundles: Vec<(u128, Vec<u8>)> = Vec::new();
    let mut current_first_id: Option<u128> = None;
    let mut current_bucket: u128 = 0;
    let mut current_bytes: Vec<u8> = Vec::new();

    for node in nodes {
        let node_bytes = encode_node_definition_dcmm2_bytes_with(node, framing)?;
        ensure!(
            header_len.saturating_add(node_bytes.len()) <= max_bytes_per_bundle,
            "Single encoded node l{} ({} bytes + {header_len}-byte header) exceeds max_bytes_per_bundle ({max_bytes_per_bundle})",
            node.id,
            node_bytes.len()
        );

        let bucket = node
            .id
            .saturating_sub(SYSTEM_REGION_START)
            .checked_div(max_nodes_u128)
            .unwrap_or(0);

        if let Some(first_id) = current_first_id {
            let crosses_bucket = bucket != current_bucket;
            let exceeds_bytes =
                current_bytes.len().saturating_add(node_bytes.len())
                    > max_bytes_per_bundle;
            if crosses_bucket || exceeds_bytes {
                bundles.push((first_id, std::mem::take(&mut current_bytes)));
                current_first_id = None;
            }
        }

        if current_first_id.is_none() {
            current_first_id = Some(node.id);
            current_bucket = bucket;
            current_bytes.extend_from_slice(&header_bytes);
        }
        current_bytes.extend_from_slice(&node_bytes);
    }

    if let Some(first_id) = current_first_id {
        bundles.push((first_id, current_bytes));
    }

    Ok(bundles)
}

/// Splits a slice of `LedgerNode` entries into one or more `.ctbnbdl` bundle buffers,
/// each paired with its starting node ID (`(first_node_id, bundle_bytes)`).
///
/// Bundles group up to `max_nodes_per_bundle` nodes or `max_bytes_per_bundle` bytes
/// (whichever is reached first). If a bundle must split early due to the byte limit,
/// the subsequent bundle is shortened so that bundle boundaries still align at
/// multiples of `max_nodes_per_bundle` relative to `SYSTEM_REGION_START`.
pub fn encode_nodes_to_ctbnbdl_bundles_with_limits(
    nodes: &[LedgerNode],
    framing: CtbnbdlFramingDcs,
    max_nodes_per_bundle: usize,
    max_bytes_per_bundle: usize,
) -> Result<Vec<(u128, Vec<u8>)>> {
    encode_nodes_to_ctbnbdl_bundles_with_limits_and_most_recent(
        nodes,
        framing,
        max_nodes_per_bundle,
        max_bytes_per_bundle,
        nodes.iter().map(|n| n.id).max(),
    )
}

/// Returns `true` if `tok` is one of the canonical relationship verbs in `framing`.
#[must_use]
pub fn is_relationship_verb_with(
    tok: DcShorthand,
    framing: CtbnbdlFramingDcs,
) -> bool {
    tok == framing.has_a
        || tok == framing.is_instance_of
        || tok == framing.is_subtype_of
        || tok == framing.is_based_on
        || tok == framing.is_deprecated
        || tok == framing.is_closing_delimiter_for
        || tok == framing.supersedes
        || tok == framing.retracts
        || tok == framing.is_associated_with
        || tok == framing.has_value
        || tok == DcShorthand::Short(645)
        || tok == DcShorthand::Short(646)
}

/// Parses a relationship statement `<verb> [atom_dc] [atom_dc]` back into `(verb, subj, obj)`.
#[must_use]
pub fn parse_statement_tokens_with(
    tokens: &[DcShorthand],
    framing: CtbnbdlFramingDcs,
) -> Option<(DcShorthand, DcShorthand, DcShorthand)> {
    let verb = *tokens.first()?;
    if !is_relationship_verb_with(verb, framing) {
        return None;
    }
    let mut idx = 1usize;
    let subj = consume_atom_dc_with(tokens, &mut idx, framing)?;
    let obj = consume_atom_dc_with(tokens, &mut idx, framing)?;
    if idx == tokens.len() {
        Some((verb, subj, obj))
    } else {
        None
    }
}

/// Encodes a `Dc 638 "For each"` relationship statement over a typed list of `[atom_dc]` targets:
/// `638 <inner_verb> [atom_dc: <inner_subj>] [atom_dc: <inner_obj>] 310 <list_type> <t_1> [312 <t_2> ...] 311`.
#[must_use]
pub fn encode_for_each_statement_tokens_with(
    inner_verb: DcShorthand,
    inner_subj: DcShorthand,
    inner_obj: DcShorthand,
    list_type: DcShorthand,
    targets: &[DcShorthand],
    framing: CtbnbdlFramingDcs,
) -> Vec<DcShorthand> {
    let mut out =
        Vec::with_capacity(targets.len().saturating_mul(2).saturating_add(7));
    out.push(DcShorthand::Short(638));
    out.push(inner_verb);
    push_atom_dc_with(&mut out, inner_subj, framing);
    push_atom_dc_with(&mut out, inner_obj, framing);
    out.push(DcShorthand::Short(310));
    out.push(list_type);
    for (i, &target) in targets.iter().enumerate() {
        if i > 0 {
            out.push(DcShorthand::Short(312));
        }
        push_atom_dc_with(&mut out, target, framing);
    }
    out.push(DcShorthand::Short(311));
    out
}

/// Parses a `Dc 638 "For each"` relationship statement node
/// (`638 <inner_verb> <inner_subj> <inner_obj> (310 <list_type> <elem_1> [312 <elem_2> ...] 311 | <atom_dc>)`)
/// into `(inner_verb, inner_subj, inner_obj, list_type, expanded_targets)`.
#[must_use]
pub fn parse_for_each_statement_tokens_with(
    tokens: &[DcShorthand],
    framing: CtbnbdlFramingDcs,
) -> Option<(
    DcShorthand,
    DcShorthand,
    DcShorthand,
    DcShorthand,
    Vec<DcShorthand>,
)> {
    if tokens.first() != Some(&DcShorthand::Short(638)) {
        return None;
    }
    let inner_verb = *tokens.get(1)?;
    if !is_relationship_verb_with(inner_verb, framing) {
        return None;
    }
    let mut idx = 2usize;
    let inner_subj = consume_atom_dc_with(tokens, &mut idx, framing)?;
    let inner_obj = consume_atom_dc_with(tokens, &mut idx, framing)?;

    if tokens.get(idx) == Some(&DcShorthand::Short(310)) {
        idx = idx.saturating_add(1);
        let list_type = *tokens.get(idx)?;
        idx = idx.saturating_add(1);
        let end_idx = tokens.len().checked_sub(1)?;
        if idx >= end_idx
            || tokens.get(end_idx) != Some(&DcShorthand::Short(311))
        {
            return None;
        }
        let mut targets = Vec::new();
        while idx < end_idx {
            if tokens.get(idx) == Some(&DcShorthand::Short(4)) {
                idx = idx.saturating_add(1);
                let start_tok =
                    consume_atom_dc_with(tokens, &mut idx, framing)?;
                let end_tok = consume_atom_dc_with(tokens, &mut idx, framing)?;
                match (start_tok, end_tok) {
                    (DcShorthand::Long(s), DcShorthand::Long(e))
                        if s <= e && e.saturating_sub(s) <= 1_000_000 =>
                    {
                        for id in s..=e {
                            targets.push(DcShorthand::Long(id));
                        }
                    }
                    (DcShorthand::Short(s), DcShorthand::Short(e))
                        if s <= e =>
                    {
                        for id in s..=e {
                            targets.push(DcShorthand::Short(id));
                        }
                    }
                    _ => return None,
                }
            } else {
                let elem = consume_atom_dc_with(tokens, &mut idx, framing)?;
                targets.push(elem);
            }
            if idx > end_idx {
                return None;
            }
            if idx < end_idx {
                if tokens.get(idx) != Some(&DcShorthand::Short(312)) {
                    return None;
                }
                idx = idx.saturating_add(1);
                if idx >= end_idx {
                    return None;
                }
            }
        }
        if targets.is_empty() {
            None
        } else {
            Some((inner_verb, inner_subj, inner_obj, list_type, targets))
        }
    } else {
        let elem = consume_atom_dc_with(tokens, &mut idx, framing)?;
        if idx == tokens.len() {
            Some((
                inner_verb,
                inner_subj,
                inner_obj,
                DcShorthand::Short(640),
                vec![elem],
            ))
        } else {
            None
        }
    }
}

/// Partitions an ordered slice of `LedgerNode` entries into `(active_nodes, inactive_nodes)`
/// preserving ascending node ID order in both output vectors.
///
/// Inactive nodes include:
/// 1. Any node targeted by an active `591 <new_id> l<old_id>` (`Supersedes`) or
///    `592 615 l<old_id>` (`Retracts`) statement.
/// 2. All `591` (`Supersedes`) and `592` (`Retracts`) revision/retraction statements themselves.
/// 3. Any `610 l<has_a_id> <val>` (`Has value`) or `584 l<has_a_id> 570` (`Has a Preferred`)
///    statement whose subject `has_a_id` is inactive, or any `584 <subj> <prop>` (`prop != 570`)
///    statement that has no active `610` value statement.
/// 4. Any non-statement value node that is transitively referenced by an inactive statement
///    and is not transitively referenced by any active statement.
#[must_use]
pub fn partition_active_and_inactive_nodes_with(
    nodes: &[LedgerNode],
    framing: CtbnbdlFramingDcs,
) -> (Vec<LedgerNode>, Vec<LedgerNode>) {
    use std::collections::{HashMap, HashSet};

    let mut node_by_id: HashMap<u128, &LedgerNode> =
        HashMap::with_capacity(nodes.len());
    for node in nodes {
        node_by_id.insert(node.id, node);
    }

    // Pass 1: Reverse chronological pass to identify superseded and retracted node IDs,
    // as well as all `591` (`Supersedes`) and `592` (`Retracts`) revision statement IDs.
    let mut superseded_or_retracted_ids: HashSet<u128> = HashSet::new();
    let mut revision_stmt_ids: HashSet<u128> = HashSet::new();
    for node in nodes.iter().rev() {
        let Some((verb, subj, obj)) =
            parse_statement_tokens_with(&node.tokens, framing)
        else {
            continue;
        };
        if verb == framing.supersedes {
            revision_stmt_ids.insert(node.id);
            if !superseded_or_retracted_ids.contains(&node.id) {
                if let DcShorthand::Long(old_id) = obj {
                    superseded_or_retracted_ids.insert(old_id);
                }
            }
        } else if verb == framing.retracts && subj == framing.enclosing_document
        {
            revision_stmt_ids.insert(node.id);
            if !superseded_or_retracted_ids.contains(&node.id) {
                if let DcShorthand::Long(old_id) = obj {
                    superseded_or_retracted_ids.insert(old_id);
                }
            }
        }
    }

    // Pass 2: Identify active vs. inactive statement nodes.
    let mut has_a_with_active_value: HashSet<u128> = HashSet::new();
    for node in nodes {
        if superseded_or_retracted_ids.contains(&node.id)
            || revision_stmt_ids.contains(&node.id)
        {
            continue;
        }
        if let Some((verb, DcShorthand::Long(has_a_id), _val)) =
            parse_statement_tokens_with(&node.tokens, framing)
        {
            if verb == framing.has_value
                && !superseded_or_retracted_ids.contains(&has_a_id)
            {
                has_a_with_active_value.insert(has_a_id);
            }
        }
    }

    let mut active_stmt_ids: HashSet<u128> = HashSet::new();
    let mut inactive_stmt_ids: HashSet<u128> = HashSet::new();
    let mut provenance_stmt_ids: HashSet<u128> = HashSet::new();
    let sh_data_source = DcShorthand::Short(635);
    for node in nodes {
        if let Some((inner_verb, _, inner_obj, _, _)) =
            parse_for_each_statement_tokens_with(&node.tokens, framing)
        {
            if inner_verb == framing.has_a && inner_obj == sh_data_source {
                provenance_stmt_ids.insert(node.id);
            }
            let is_active = if superseded_or_retracted_ids.contains(&node.id)
                || revision_stmt_ids.contains(&node.id)
            {
                false
            } else if inner_verb == framing.has_a
                && inner_obj != framing.preferred
            {
                has_a_with_active_value.contains(&node.id)
            } else {
                true
            };
            if is_active {
                active_stmt_ids.insert(node.id);
            } else {
                inactive_stmt_ids.insert(node.id);
            }
            continue;
        }

        let Some((verb, subj, obj)) =
            parse_statement_tokens_with(&node.tokens, framing)
        else {
            continue;
        };
        if verb == framing.has_a && obj == sh_data_source {
            provenance_stmt_ids.insert(node.id);
        } else if verb == framing.has_value {
            if let DcShorthand::Long(target_id) = subj
                && provenance_stmt_ids.contains(&target_id)
            {
                provenance_stmt_ids.insert(node.id);
            }
        }
        let is_active = if superseded_or_retracted_ids.contains(&node.id)
            || revision_stmt_ids.contains(&node.id)
        {
            false
        } else if verb == framing.has_value {
            matches!(
                subj,
                DcShorthand::Long(has_a_id)
                    if !superseded_or_retracted_ids.contains(&has_a_id)
            )
        } else if verb == framing.has_a && obj == framing.preferred {
            matches!(
                subj,
                DcShorthand::Long(has_a_id)
                    if !superseded_or_retracted_ids.contains(&has_a_id)
                        && has_a_with_active_value.contains(&has_a_id)
            )
        } else if verb == framing.has_a {
            has_a_with_active_value.contains(&node.id)
        } else {
            true
        };
        if is_active {
            active_stmt_ids.insert(node.id);
        } else {
            inactive_stmt_ids.insert(node.id);
        }
    }

    // Pass 3: Compute transitive reachability of `DcShorthand::Long(id)` value nodes from
    // active domain statements and from inactive domain statements.
    // Note: Provenance metadata statements (`638` / `584 _ 635` and their `610` values)
    // do not keep otherwise-unreferenced value nodes alive when their domain statement is retracted.
    let collect_reachable = |seed_stmt_ids: &HashSet<u128>| -> HashSet<u128> {
        let mut reachable: HashSet<u128> = HashSet::new();
        let mut worklist: Vec<u128> = Vec::new();
        for &stmt_id in seed_stmt_ids {
            if provenance_stmt_ids.contains(&stmt_id) {
                continue;
            }
            if let Some(stmt_node) = node_by_id.get(&stmt_id) {
                for &tok in &stmt_node.tokens {
                    if let DcShorthand::Long(ref_id) = tok {
                        worklist.push(ref_id);
                    }
                }
            }
        }
        while let Some(ref_id) = worklist.pop() {
            if active_stmt_ids.contains(&ref_id)
                || inactive_stmt_ids.contains(&ref_id)
                || superseded_or_retracted_ids.contains(&ref_id)
                || revision_stmt_ids.contains(&ref_id)
                || provenance_stmt_ids.contains(&ref_id)
            {
                continue;
            }
            if reachable.insert(ref_id) {
                if let Some(ref_node) = node_by_id.get(&ref_id) {
                    for &tok in &ref_node.tokens {
                        if let DcShorthand::Long(inner_id) = tok {
                            worklist.push(inner_id);
                        }
                    }
                }
            }
        }
        reachable
    };

    let reachable_from_active = collect_reachable(&active_stmt_ids);
    let reachable_from_inactive = collect_reachable(&inactive_stmt_ids);

    // Pass 4: Partition `nodes` in ascending ID order.
    let mut active_nodes = Vec::with_capacity(nodes.len());
    let mut inactive_nodes = Vec::new();
    for node in nodes {
        let is_active = if active_stmt_ids.contains(&node.id) {
            true
        } else if inactive_stmt_ids.contains(&node.id)
            || superseded_or_retracted_ids.contains(&node.id)
        {
            false
        } else if reachable_from_active.contains(&node.id) {
            true
        } else {
            !reachable_from_inactive.contains(&node.id)
        };
        if is_active {
            active_nodes.push(node.clone());
        } else {
            inactive_nodes.push(node.clone());
        }
    }

    (active_nodes, inactive_nodes)
}

fn write_bytes_if_changed(path: &Path, bytes: &[u8]) -> Result<bool> {
    use std::io::Write;

    if path.is_file() {
        if let Ok(existing) = std::fs::read(path) {
            if existing == bytes {
                return Ok(false);
            }
            if let Some(tail) = bytes.strip_prefix(existing.as_slice()) {
                let mut file = std::fs::OpenOptions::new()
                    .append(true)
                    .open(path)
                    .with_context(|| {
                        format!(
                            "Failed to open {} in append mode",
                            path.display()
                        )
                    })?;
                file.write_all(tail).with_context(|| {
                    format!(
                        "Failed to append {} bytes to {}",
                        tail.len(),
                        path.display()
                    )
                })?;
                return Ok(true);
            }
        }
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).with_context(|| {
            format!("Failed to create directory {}", parent.display())
        })?;
    }
    std::fs::write(path, bytes).with_context(|| {
        format!("Failed to write .ctbnbdl file {}", path.display())
    })?;
    Ok(true)
}

fn write_ctbnbdl_bundles_to_dir(
    dst_dir: &Path,
    bundles: &[(u128, Vec<u8>)],
    remove_stale_ctbnodes: bool,
) -> Result<usize> {
    std::fs::create_dir_all(dst_dir).with_context(|| {
        format!(
            "Failed to create destination directory {}",
            dst_dir.display()
        )
    })?;

    let mut expected_bundle_names =
        std::collections::HashSet::with_capacity(bundles.len());
    let mut compiled_count = 0usize;
    for (first_id, bytes) in bundles {
        let filename = format_chunk_bundle_filename(*first_id);
        let dst_path = dst_dir.join(&filename);
        expected_bundle_names.insert(filename);
        if write_bytes_if_changed(&dst_path, bytes)? {
            compiled_count = compiled_count.saturating_add(1);
        }
    }

    for entry in std::fs::read_dir(dst_dir).with_context(|| {
        format!("Failed to read destination directory {}", dst_dir.display())
    })? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let ext = path.extension().and_then(|e| e.to_str());
        if (remove_stale_ctbnodes && ext == Some("ctbnodes"))
            || (ext == Some("ctbnbdl") && !expected_bundle_names.contains(name))
        {
            std::fs::remove_file(&path).with_context(|| {
                format!("Failed to remove stale file {}", path.display())
            })?;
        }
    }

    Ok(compiled_count)
}

/// Compiles all `.ctbnodes` chunk files in `src_dir` into:
/// 1. Unsplit append-friendly `.ctbnbdl` bundle files directly in `bundles_dir`
///    (containing all `LedgerNode`s in ascending ID order without a `Dc 641` watermark header,
///    so newly appended nodes extend the tail of the bundle without rewriting invalidated entries),
/// 2. Live (active) `.ctbnbdl` bundle files in `bundles_dir/split/live/` (with `Dc 641` watermark), and
/// 3. Historical (inactive) `.ctbnbdl` bundle files in `bundles_dir/split/history/` (with `Dc 641` watermark).
///
/// Returns `(unsplit_written, live_written, history_written)`.
pub fn compile_graph_bundles_from_ctbnodes_dir_with_limits(
    src_dir: &Path,
    bundles_dir: &Path,
    framing: CtbnbdlFramingDcs,
    max_nodes_per_bundle: usize,
    max_bytes_per_bundle: usize,
) -> Result<(usize, usize, usize)> {
    if !src_dir.is_dir() {
        return Ok((0, 0, 0));
    }
    let nodes = load_ledger_nodes_from_dir(src_dir)?;
    let unsplit_bundles =
        encode_nodes_to_ctbnbdl_bundles_with_limits_and_most_recent(
            &nodes,
            framing,
            max_nodes_per_bundle,
            max_bytes_per_bundle,
            None,
        )?;
    let unsplit_written =
        write_ctbnbdl_bundles_to_dir(bundles_dir, &unsplit_bundles, true)?;

    let most_recent_node_id = nodes.iter().map(|n| n.id).max();
    let (active_nodes, inactive_nodes) =
        partition_active_and_inactive_nodes_with(&nodes, framing);
    let active_bundles =
        encode_nodes_to_ctbnbdl_bundles_with_limits_and_most_recent(
            &active_nodes,
            framing,
            max_nodes_per_bundle,
            max_bytes_per_bundle,
            most_recent_node_id,
        )?;
    let inactive_bundles =
        encode_nodes_to_ctbnbdl_bundles_with_limits_and_most_recent(
            &inactive_nodes,
            framing,
            max_nodes_per_bundle,
            max_bytes_per_bundle,
            most_recent_node_id,
        )?;

    let split_live_dir = bundles_dir.join(GRAPH_BUNDLES_SPLIT_LIVE_SUBDIR);
    let live_written =
        write_ctbnbdl_bundles_to_dir(&split_live_dir, &active_bundles, true)?;
    let split_history_dir =
        bundles_dir.join(GRAPH_BUNDLES_SPLIT_HISTORY_SUBDIR);
    let history_written = write_ctbnbdl_bundles_to_dir(
        &split_history_dir,
        &inactive_bundles,
        true,
    )?;

    Ok((unsplit_written, live_written, history_written))
}

/// Same as [`compile_graph_bundles_from_ctbnodes_dir_with_limits`] using default
/// `MAX_NODES_PER_BUNDLE` and `MAX_BYTES_PER_BUNDLE` limits.
pub fn compile_graph_bundles_from_ctbnodes_dir(
    src_dir: &Path,
    bundles_dir: &Path,
    framing: CtbnbdlFramingDcs,
) -> Result<(usize, usize, usize)> {
    compile_graph_bundles_from_ctbnodes_dir_with_limits(
        src_dir,
        bundles_dir,
        framing,
        MAX_NODES_PER_BUNDLE,
        MAX_BYTES_PER_BUNDLE,
    )
}

/// Compiles all `.ctbnodes` chunk files in `src_dir` into grouped active `.ctbnbdl` bundle files
/// in `dst_dir` using custom node-count and byte-size limits, removing any stale `.ctbnodes`
/// or obsolete `.ctbnbdl` files in `dst_dir`.
///
/// When `src_dir == dst_dir` (e.g., `assets/data/graph`), this delegates to
/// [`compile_graph_bundles_from_ctbnodes_dir_with_limits`] targeting
/// `bundles_dir_for_graph_dir(src_dir)` (`assets/data/graph-bundles/` and `split/{live,history}/`).
///
/// Returns the number of primary `.ctbnbdl` files written or updated.
pub fn compile_ctbnodes_dir_to_ctbnbdl_with_limits(
    src_dir: &Path,
    dst_dir: &Path,
    framing: CtbnbdlFramingDcs,
    max_nodes_per_bundle: usize,
    max_bytes_per_bundle: usize,
) -> Result<usize> {
    if !src_dir.is_dir() {
        return Ok(0);
    }

    let is_same_dir = if let (Ok(c_src), Ok(c_dst)) =
        (src_dir.canonicalize(), dst_dir.canonicalize())
    {
        c_src == c_dst
    } else {
        src_dir == dst_dir
    };

    if is_same_dir {
        let bundles_dir = bundles_dir_for_graph_dir(src_dir);
        let (unsplit_written, _, _) =
            compile_graph_bundles_from_ctbnodes_dir_with_limits(
                src_dir,
                &bundles_dir,
                framing,
                max_nodes_per_bundle,
                max_bytes_per_bundle,
            )?;
        return Ok(unsplit_written);
    }

    std::fs::create_dir_all(dst_dir).with_context(|| {
        format!(
            "Failed to create destination directory {}",
            dst_dir.display()
        )
    })?;

    let nodes = load_ledger_nodes_from_dir(src_dir)?;
    let most_recent_node_id = nodes.iter().map(|n| n.id).max();
    let (active_nodes, _inactive_nodes) =
        partition_active_and_inactive_nodes_with(&nodes, framing);

    let active_bundles =
        encode_nodes_to_ctbnbdl_bundles_with_limits_and_most_recent(
            &active_nodes,
            framing,
            max_nodes_per_bundle,
            max_bytes_per_bundle,
            most_recent_node_id,
        )?;
    write_ctbnbdl_bundles_to_dir(dst_dir, &active_bundles, true)
}

/// Compiles all `.ctbnodes` chunk files in `src_dir` into grouped active `.ctbnbdl` bundle files
/// in `dst_dir` (up to `MAX_NODES_PER_BUNDLE` = 1,000,000 nodes or `MAX_BYTES_PER_BUNDLE` = 50 MB
/// per bundle), removing any stale `.ctbnodes` or obsolete `.ctbnbdl` files in `dst_dir`.
///
/// When `src_dir == dst_dir` (e.g., `assets/data/graph`), this also compiles all filtered-out
/// inactive (superseded/retracted) nodes into `dst_dir.join("system")`.
///
/// Returns the number of active `.ctbnbdl` files written or updated in `dst_dir`.
pub fn compile_ctbnodes_dir_to_ctbnbdl(
    src_dir: &Path,
    dst_dir: &Path,
    framing: CtbnbdlFramingDcs,
) -> Result<usize> {
    compile_ctbnodes_dir_to_ctbnbdl_with_limits(
        src_dir,
        dst_dir,
        framing,
        MAX_NODES_PER_BUNDLE,
        MAX_BYTES_PER_BUNDLE,
    )
}

/// Compiles all filtered-out inactive (superseded/retracted) nodes from `.ctbnodes` chunk files
/// in `src_dir` into grouped `.ctbnbdl` bundle files in `history_dst_dir` using custom
/// node-count and byte-size limits.
///
/// Returns the number of `.ctbnbdl` files written or updated in `history_dst_dir`.
pub fn compile_inactive_ctbnodes_dir_to_ctbnbdl_with_limits(
    src_dir: &Path,
    history_dst_dir: &Path,
    framing: CtbnbdlFramingDcs,
    max_nodes_per_bundle: usize,
    max_bytes_per_bundle: usize,
) -> Result<usize> {
    if !src_dir.is_dir() {
        return Ok(0);
    }
    let nodes = load_ledger_nodes_from_dir(src_dir)?;
    let most_recent_node_id = nodes.iter().map(|n| n.id).max();
    let (_active_nodes, inactive_nodes) =
        partition_active_and_inactive_nodes_with(&nodes, framing);
    let inactive_bundles =
        encode_nodes_to_ctbnbdl_bundles_with_limits_and_most_recent(
            &inactive_nodes,
            framing,
            max_nodes_per_bundle,
            max_bytes_per_bundle,
            most_recent_node_id,
        )?;
    write_ctbnbdl_bundles_to_dir(history_dst_dir, &inactive_bundles, true)
}

/// Compiles all filtered-out inactive (superseded/retracted) nodes from `.ctbnodes` chunk files
/// in `src_dir` into grouped `.ctbnbdl` bundle files in `history_dst_dir` (up to
/// `MAX_NODES_PER_BUNDLE` = 1,000,000 nodes or `MAX_BYTES_PER_BUNDLE` = 50 MB per bundle).
///
/// Returns the number of `.ctbnbdl` files written or updated in `history_dst_dir`.
pub fn compile_inactive_ctbnodes_dir_to_ctbnbdl(
    src_dir: &Path,
    history_dst_dir: &Path,
    framing: CtbnbdlFramingDcs,
) -> Result<usize> {
    compile_inactive_ctbnodes_dir_to_ctbnbdl_with_limits(
        src_dir,
        history_dst_dir,
        framing,
        MAX_NODES_PER_BUNDLE,
        MAX_BYTES_PER_BUNDLE,
    )
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

    fn test_framing() -> CtbnbdlFramingDcs {
        CtbnbdlFramingDcs {
            begin_node: DcShorthand::Short(617),
            end_node: DcShorthand::Short(618),
            begin_checksummed_section: DcShorthand::Short(543),
            end_checksummed_section: DcShorthand::Short(544),
            section_digest: DcShorthand::Short(545),
            begin_time_moment: DcShorthand::Short(390),
            end_time_moment: DcShorthand::Short(391),
            section_node_creation_time: Some(DcShorthand::Short(1941)),
            ns_since_epoch_format: DcShorthand::Format(487),
            sha256_format: DcShorthand::Format(245),
            escape: DcShorthand::Short(255),
            evaluate_as_atom: DcShorthand::Short(616),
            has_a: DcShorthand::Short(584),
            has_value: DcShorthand::Short(610),
            preferred: DcShorthand::Short(570),
            supersedes: DcShorthand::Short(591),
            retracts: DcShorthand::Short(592),
            enclosing_document: DcShorthand::Short(615),
            is_instance_of: DcShorthand::Short(586),
            is_subtype_of: DcShorthand::Short(587),
            is_based_on: DcShorthand::Short(588),
            is_deprecated: DcShorthand::Short(589),
            is_closing_delimiter_for: DcShorthand::Short(590),
            is_associated_with: DcShorthand::Short(605),
        }
    }

    #[test]
    fn test_ctbnbdl_roundtrips_standalone_and_encapsulated_dc203() -> Result<()> {
        let framing = test_framing();
        let node = LedgerNode {
            id: SYSTEM_REGION_START + 1,
            tokens: vec![
                // Standalone Dc 203 token
                DcShorthand::Short(584),
                DcShorthand::Short(203),
                DcShorthand::Short(572),
                // Textual base64 encapsulation: b"," (length 1)
                DcShorthand::Short(203),
                DcShorthand::Short(6),
                DcShorthand::Format(199),
                DcShorthand::Short(128),
                DcShorthand::Short(7),
                DcShorthand::Short(138),
                DcShorthand::Short(127),
                DcShorthand::Short(195),
                DcShorthand::Short(195),
                DcShorthand::Short(204),
                // Empty textual base64 encapsulation: b"" (length 0)
                DcShorthand::Short(203),
                DcShorthand::Short(6),
                DcShorthand::Format(199),
                DcShorthand::Short(127),
                DcShorthand::Short(7),
                DcShorthand::Short(204),
            ],
            comment: String::new(),
            created_ns: Some(1_795_314_000_123_456_789),
        };

        let encoded = encode_nodes_to_ctbnbdl_bytes_with(&[node.clone()], framing)?;
        let (_, decoded) = decode_ctbnbdl_bundle_with(&encoded, framing)?;
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].id, node.id);
        assert_eq!(decoded[0].tokens, node.tokens);
        assert_eq!(decoded[0].created_ns, node.created_ns);
        Ok(())
    }

    #[test]
    fn test_ctbnbdl_checksum_and_resilient_section_recovery() -> Result<()> {
        let framing = test_framing();
        let n0 = LedgerNode {
            id: SYSTEM_REGION_START + 10,
            tokens: vec![
                DcShorthand::Short(584),
                DcShorthand::Short(544),
                DcShorthand::Short(572),
            ],
            comment: "legacy node without timestamp".to_string(),
            created_ns: None,
        };
        let n1 = LedgerNode {
            id: SYSTEM_REGION_START + 11,
            tokens: vec![
                DcShorthand::Unicode(72),
                DcShorthand::Unicode(105),
            ],
            comment: "timestamped node".to_string(),
            created_ns: Some(1_700_000_000_000_000_000),
        };
        let n2 = LedgerNode {
            id: SYSTEM_REGION_START + 12,
            tokens: vec![
                DcShorthand::Short(584),
                DcShorthand::Short(310),
                DcShorthand::Short(572),
            ],
            comment: "pre-epoch timestamped node".to_string(),
            created_ns: Some(-42_000_000_000),
        };

        for orig in [&n0, &n1, &n2] {
            let line = orig.format_line();
            let reparsed = LedgerNode::parse_line(&line, "test.ctbnodes", 1)?;
            assert_eq!(&reparsed, orig);
        }
        assert_eq!(
            n0.format_line(),
            format!("l{}: 584 544 572 # legacy node without timestamp", n0.id)
        );

        let bundle = encode_nodes_to_ctbnbdl_bytes_with(
            &[n0.clone(), n1.clone(), n2.clone()],
            framing,
        )?;
        let (wm, strict_decoded) = decode_ctbnbdl_bundle_with(&bundle, framing)?;
        assert_eq!(wm, Some(n2.id));
        assert_eq!(strict_decoded.len(), 3);
        assert_eq!(strict_decoded[0].created_ns, None);
        assert_eq!(strict_decoded[1].created_ns, n1.created_ns);
        assert_eq!(strict_decoded[2].created_ns, n2.created_ns);

        // Corrupt a byte inside n1's checksummed section body!
        let header_bytes = encode_ctbnbdl_header_bytes(Some(n2.id), framing)?;
        let n0_bytes = encode_node_definition_dcmm2_bytes_with(&n0, framing)?;
        let corrupt_offset = header_bytes.len() + n0_bytes.len() + 10;
        let mut corrupted = bundle.clone();
        corrupted[corrupt_offset] ^= 0xFF;

        // Strict decoder MUST fail on the corrupted section.
        assert!(decode_ctbnbdl_bundle_with(&corrupted, framing).is_err());

        // Resilient decoder MUST quarantine n1 with a warning and recover n0 and n2 intact!
        let (res_wm, res_nodes, warnings) =
            decode_ctbnbdl_bundle_resilient_with(&corrupted, framing)?;
        assert_eq!(res_wm, Some(n2.id));
        assert!(!warnings.is_empty());
        assert_eq!(res_nodes.len(), 2);
        assert_eq!(res_nodes[0].id, n0.id);
        assert_eq!(res_nodes[0].tokens, n0.tokens);
        assert_eq!(res_nodes[0].created_ns, None);
        assert_eq!(res_nodes[1].id, n2.id);
        assert_eq!(res_nodes[1].tokens, n2.tokens);
        assert_eq!(res_nodes[1].created_ns, n2.created_ns);
        Ok(())
    }
}
