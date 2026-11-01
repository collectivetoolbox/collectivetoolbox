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

//! Manual and staged `.dcsh` node allocation pipeline (`nodes/inbox/` -> `nodes/allocated/`),
//! dynamic effective-state prevalidation, `BATCH_COMMITTED` provenance verification, and
//! cross-ownership protection between `csv` and `manual` graph nodes.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::{
    DC_ENCLOSING_DOCUMENT, DC_HAS_A, DC_HAS_VALUE, DC_PREFERRED, DC_RETRACTS,
    DC_SUPERSEDES, DcShorthand, LedgerNode, SYSTEM_REGION_END,
    SYSTEM_REGION_START, default_ctbnbdl_framing_dcs, parse_statement_tokens,
    sh,
};
use crate::seed_graph::{
    compute_effective_inactive_ids, parse_ambiguous_values_tokens,
};
use crate::ctbnbdl::{
    DurableTransactionOutcome, FaultInjectionPoint, JournalRecord,
    LEDGER_JOURNAL_REL_PATH, LedgerReadGuard, PlannedLedgerItem,
    chunk_filename_for_node_id, ensure_ledger_journal_clean,
    execute_durable_ledger_transaction_with_planner, inspect_ledger_journal,
    is_relationship_verb_with, load_ledger_nodes_from_dir, sha256_bytes_hex,
};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::Path;

/// Highest node ID in the pre-journal `bootstrap-a90464028` baseline (`17,833` nodes).
pub const BOOTSTRAP_BASELINE_LAST_ID: u128 = 8_589_952_424;

/// Provenance ownership classification of a ledger node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeProvenance {
    /// Derived from or reconciled by the CSV data tables in `src/formats/dcdata/data-src/`.
    Csv,
    /// Manually authored via `nodes/inbox/` and protected from automatic CSV withdrawal.
    Manual,
}

impl NodeProvenance {
    /// Parses `"csv"` or `"manual"`.
    pub fn parse(raw: &str) -> Result<Self> {
        match raw.trim() {
            "csv" => Ok(Self::Csv),
            "manual" => Ok(Self::Manual),
            other => bail!(
                "Invalid # @provenance value {other:?}; expected 'csv' or 'manual'"
            ),
        }
    }

    /// Canonical string representation (`"csv"` or `"manual"`).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Csv => "csv",
            Self::Manual => "manual",
        }
    }
}

/// Verified provenance metadata for an allocated node (`> BOOTSTRAP_BASELINE_LAST_ID`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeProvenanceEntry {
    /// Allocated node ID (`l<node_id>`).
    pub node_id: u128,
    /// Ownership (`Csv` or `Manual`).
    pub provenance: NodeProvenance,
    /// Optional human-readable rationale (required when `Manual`).
    pub rationale: Option<String>,
    /// Allocated `.dcsh` filename in `nodes/allocated/`, if allocated via inbox.
    pub allocated_file: Option<String>,
}

/// Verified provenance ownership map across the committed ledger.
#[derive(Debug, Clone, Default)]
pub struct NodeProvenanceMap {
    entries: HashMap<u128, NodeProvenanceEntry>,
}

impl NodeProvenanceMap {
    /// Returns the provenance owner of `node_id` (nodes `<= BOOTSTRAP_BASELINE_LAST_ID` are `Csv`).
    #[must_use]
    pub fn owner(&self, node_id: u128) -> NodeProvenance {
        // Reason for fallback: all pre-journal baseline nodes (<= BOOTSTRAP_BASELINE_LAST_ID) are CSV-derived.
        self.entries
            .get(&node_id)
            .map_or(NodeProvenance::Csv, |e| e.provenance)
    }

    /// Returns the full provenance entry for `node_id`, if recorded above the bootstrap baseline.
    #[must_use]
    pub fn get(&self, node_id: u128) -> Option<&NodeProvenanceEntry> {
        self.entries.get(&node_id)
    }

    /// Inserts a provenance entry (used by in-memory unit tests).
    pub fn insert(&mut self, entry: NodeProvenanceEntry) {
        self.entries.insert(entry.node_id, entry);
    }
}

/// Parsed contents of a single `.dcsh` file in `nodes/inbox/` or `nodes/allocated/`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedDcshInboxFile {
    /// Declared `# @provenance: csv | manual`.
    pub provenance: NodeProvenance,
    /// Optional `# @rationale: ...` (guaranteed `Some` when `provenance == Manual`).
    pub rationale: Option<String>,
    /// Optional `# @comment: ...` (or trailing `# comment` on the body line).
    pub comment: Option<String>,
    /// Sorted, deduplicated list of `# @allow-duplicate-of: l<id1>, ...` node IDs.
    pub allow_duplicate_of: Vec<u128>,
    /// Parsed `DcShorthand` token sequence from the body line.
    pub tokens: Vec<DcShorthand>,
}

/// Parses and validates the headers and body of a `.dcsh` inbox/allocated file.
pub fn parse_dcsh_inbox_file(
    content: &str,
    filename: &str,
) -> Result<ParsedDcshInboxFile> {
    let mut provenance: Option<NodeProvenance> = None;
    let mut rationale: Option<String> = None;
    let mut header_comment: Option<String> = None;
    let mut allow_duplicate_of: Vec<u128> = Vec::new();
    let mut body_lines: Vec<(usize, &str)> = Vec::new();

    for (idx, raw_line) in content.lines().enumerate() {
        let line_no = idx.saturating_add(1);
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix('#') {
            let directive = rest.trim();
            if let Some(val) = directive.strip_prefix("@provenance:") {
                ensure!(
                    provenance.is_none(),
                    "{filename}:{line_no}: duplicate # @provenance directive"
                );
                provenance = Some(NodeProvenance::parse(val).with_context(
                    || format!("{filename}:{line_no}: invalid # @provenance"),
                )?);
            } else if let Some(val) = directive.strip_prefix("@rationale:") {
                let clean = val.trim();
                ensure!(
                    !clean.is_empty(),
                    "{filename}:{line_no}: # @rationale cannot be empty"
                );
                rationale = Some(clean.to_string());
            } else if let Some(val) = directive.strip_prefix("@comment:") {
                let clean = val.trim();
                if !clean.is_empty() {
                    header_comment = Some(clean.to_string());
                }
            } else if let Some(val) =
                directive.strip_prefix("@allow-duplicate-of:")
            {
                for part in val.split([',', ' ']) {
                    let p = part.trim();
                    if p.is_empty() {
                        continue;
                    }
                    let Some(digits) = p.strip_prefix('l') else {
                        bail!(
                            "{filename}:{line_no}: # @allow-duplicate-of target {p:?} must use 'l<id>' syntax"
                        );
                    };
                    let id: u128 = digits.parse().with_context(|| {
                        format!(
                            "{filename}:{line_no}: invalid node ID in # @allow-duplicate-of: {p:?}"
                        )
                    })?;
                    allow_duplicate_of.push(id);
                }
            }
            continue;
        }
        body_lines.push((line_no, trimmed));
    }

    let prov = provenance.with_context(|| {
        format!("{filename}: missing required '# @provenance: csv|manual' header")
    })?;
    if prov == NodeProvenance::Manual {
        ensure!(
            rationale.as_ref().is_some_and(|r| !r.trim().is_empty()),
            "{filename}: '# @provenance: manual' requires a non-empty '# @rationale: <explanation>' header"
        );
    }

    ensure!(
        body_lines.len() == 1,
        "{filename}: expected exactly 1 non-comment node body line, found {}",
        body_lines.len()
    );
    let (line_no, raw_body) = body_lines[0];
    let (token_part, inline_comment) =
        if let Some((lhs, rhs)) = raw_body.split_once('#') {
            let c = rhs.trim();
            (lhs.trim(), if c.is_empty() { None } else { Some(c.to_string()) })
        } else {
            (raw_body, None)
        };

    let mut tokens = Vec::new();
    for tok_str in token_part.split_whitespace() {
        let sh_tok = DcShorthand::parse(tok_str).with_context(|| {
            format!("{filename}:{line_no}: invalid DcShorthand token {tok_str:?}")
        })?;
        tokens.push(sh_tok);
    }
    ensure!(
        !tokens.is_empty(),
        "{filename}:{line_no}: node body line contains zero tokens"
    );

    allow_duplicate_of.sort_unstable();
    allow_duplicate_of.dedup();

    Ok(ParsedDcshInboxFile {
        provenance: prov,
        rationale,
        comment: header_comment.or(inline_comment),
        allow_duplicate_of,
        tokens,
    })
}

/// Validates that all committed nodes above `BOOTSTRAP_BASELINE_LAST_ID` in `nodes` have matching
/// `BATCH_COMMITTED` entries in `committed_pairs` and (for `inbox_allocation` batches) matching
/// `.dcsh` files in `src/formats/dcdata/data-src/nodes/allocated/`, with zero unrecorded `.dcsh`
/// files in `nodes/allocated/`.
///
/// Note: Here "committed entry" means a `BATCH_PENDING` + `BATCH_COMMITTED` pair in `ledger-journal.jsonl`,
/// not a Git commit. Pending recovery (`recover_pending_ledger_journal_with_guard`) must always run
/// before calling this function.
pub fn load_and_validate_node_provenance(
    repo_root: &Path,
    nodes: &[LedgerNode],
    committed_pairs: &[(JournalRecord, JournalRecord)],
) -> Result<NodeProvenanceMap> {
    let allocated_dir =
        repo_root.join("src/formats/dcdata/data-src/nodes/allocated");
    let mut node_by_id: HashMap<u128, &LedgerNode> =
        HashMap::with_capacity(nodes.len());
    for node in nodes {
        node_by_id.insert(node.id, node);
    }

    let mut map = NodeProvenanceMap::default();
    let mut expected_allocated_files: HashSet<String> = HashSet::new();

    for (pending, committed) in committed_pairs {
        ensure!(
            pending.batch_id == committed.batch_id,
            "Mismatched committed journal pair: {} vs {}",
            pending.batch_id,
            committed.batch_id
        );
        for item in &pending.items {
            let Some(&ledger_node) = node_by_id.get(&item.node_id) else {
                bail!(
                    "CRITICAL: Journal BATCH_COMMITTED ({}) records node l{}, which is missing from .ctbnodes!",
                    committed.batch_id,
                    item.node_id
                );
            };
            let actual_line = ledger_node.format_line();
            ensure!(
                actual_line == item.raw_line,
                "CRITICAL: Node l{} in .ctbnodes ({:?}) does not match BATCH_COMMITTED ({}) raw_line ({:?})",
                item.node_id,
                actual_line,
                committed.batch_id,
                item.raw_line
            );
            let prov = NodeProvenance::parse(if item.provenance.is_empty() {
                "csv"
            } else {
                &item.provenance
            })?;

            if pending.kind == "inbox_allocation" {
                let alloc_name =
                    item.allocated_name.as_deref().with_context(|| {
                        format!(
                            "BATCH_COMMITTED ({}) inbox_allocation item l{} is missing allocated_name",
                            committed.batch_id, item.node_id
                        )
                    })?;
                let expected_prefix = format!("{}_", item.node_id);
                ensure!(
                    alloc_name.starts_with(&expected_prefix)
                        && alloc_name.ends_with(".dcsh"),
                    "Allocated filename {alloc_name:?} for l{} must start with {expected_prefix:?} and end with '.dcsh'",
                    item.node_id
                );
                let alloc_path = allocated_dir.join(alloc_name);
                ensure!(
                    alloc_path.is_file(),
                    "CRITICAL: Allocated .dcsh file {} for committed node l{} is missing from disk!",
                    alloc_path.display(),
                    item.node_id
                );
                let raw_bytes = fs::read(&alloc_path).with_context(|| {
                    format!("Failed to read {}", alloc_path.display())
                })?;
                if let Some(exp_sha) = item.inbox_sha256.as_deref() {
                    let actual_sha = sha256_bytes_hex(&raw_bytes);
                    ensure!(
                        actual_sha == exp_sha,
                        "CRITICAL: Allocated file {} SHA-256 {} does not match committed inbox_sha256 {}",
                        alloc_path.display(),
                        actual_sha,
                        exp_sha
                    );
                }
                let content =
                    String::from_utf8(raw_bytes).with_context(|| {
                        format!("Invalid UTF-8 in {}", alloc_path.display())
                    })?;
                let parsed_dcsh = parse_dcsh_inbox_file(&content, alloc_name)?;
                ensure!(
                    parsed_dcsh.provenance == prov,
                    "CRITICAL: Provenance mismatch for l{} between {} ({:?}) and BATCH_COMMITTED ({:?})",
                    item.node_id,
                    alloc_path.display(),
                    parsed_dcsh.provenance,
                    prov
                );
                ensure!(
                    parsed_dcsh.tokens == ledger_node.tokens,
                    "CRITICAL: Token body mismatch for l{} between {} and .ctbnodes",
                    item.node_id,
                    alloc_path.display()
                );
                expected_allocated_files.insert(alloc_name.to_string());
            }

            ensure!(
                map.entries
                    .insert(
                        item.node_id,
                        NodeProvenanceEntry {
                            node_id: item.node_id,
                            provenance: prov,
                            rationale: item.rationale.clone(),
                            allocated_file: item.allocated_name.clone(),
                        },
                    )
                    .is_none(),
                "CRITICAL: Duplicate provenance entry for node l{} in ledger-journal.jsonl",
                item.node_id
            );
        }
    }

    // Reason for fallback: when no bootstrap_checkpoint record is present in in-memory unit tests, default to BOOTSTRAP_BASELINE_LAST_ID.
    let bootstrap_last_id = committed_pairs
        .iter()
        .find(|(p, _)| p.kind == "bootstrap_checkpoint")
        .map_or(BOOTSTRAP_BASELINE_LAST_ID, |(p, _)| p.final_last_id);

    // Every node > bootstrap_last_id must have a committed journal record.
    for node in nodes {
        if node.id > bootstrap_last_id {
            ensure!(
                map.entries.contains_key(&node.id),
                "CRITICAL: Ledger node l{} (> l{bootstrap_last_id}) has no BATCH_COMMITTED provenance entry in ledger-journal.jsonl!",
                node.id
            );
        }
    }

    // Every .dcsh file in nodes/allocated/ must correspond to a BATCH_COMMITTED inbox_allocation item.
    if allocated_dir.is_dir() {
        for entry in fs::read_dir(&allocated_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("dcsh") {
                let Some(name) = path.file_name().and_then(|n| n.to_str())
                else {
                    continue;
                };
                ensure!(
                    expected_allocated_files.contains(name),
                    "CRITICAL: Unrecorded .dcsh file {} found in {} with no matching BATCH_COMMITTED journal entry!",
                    path.display(),
                    allocated_dir.display()
                );
            }
        }
    }

    Ok(map)
}

/// Convenience wrapper for external callers/tests to validate committed node provenance from `repo_root`.
pub fn verify_repo_committed_node_provenance(
    repo_root: &Path,
) -> Result<NodeProvenanceMap> {
    let read_guard = LedgerReadGuard::acquire(repo_root)?;
    ensure_ledger_journal_clean(repo_root, &read_guard)?;
    let graph_dir = repo_root.join("assets/data/graph");
    let journal_path = repo_root.join(LEDGER_JOURNAL_REL_PATH);
    let nodes = load_ledger_nodes_from_dir(&graph_dir)?;
    let parsed = inspect_ledger_journal(&journal_path)?;
    load_and_validate_node_provenance(repo_root, &nodes, &parsed.committed_pairs)
}

/// Prevalidates a batch of `.dcsh` inbox files `(inbox_name, inbox_sha256, content)` in memory
/// against `existing_nodes` and `provenance_map`, returning the planned `PlannedLedgerItem`s.
///
/// Any validation failure aborts before `BATCH_PENDING` is written.
pub fn prevalidate_and_plan_inbox_items(
    existing_nodes: &[LedgerNode],
    provenance_map: &NodeProvenanceMap,
    inbox_files: &[(String, String, String)],
) -> Result<Vec<PlannedLedgerItem>> {
    let framing = default_ctbnbdl_framing_dcs();
    let sh_supersedes = sh(DC_SUPERSEDES);
    let sh_retracts = sh(DC_RETRACTS);
    let sh_has_a = sh(DC_HAS_A);
    let sh_has_value = sh(DC_HAS_VALUE);
    let sh_preferred = sh(DC_PREFERRED);
    let sh_enclosing_doc = sh(DC_ENCLOSING_DOCUMENT);

    let mut working_nodes: Vec<LedgerNode> = existing_nodes.to_vec();
    let mut node_by_id: HashMap<u128, LedgerNode> = working_nodes
        .iter()
        .map(|n| (n.id, n.clone()))
        .collect();
    let mut working_prov = provenance_map.clone();
    let mut planned_items: Vec<PlannedLedgerItem> =
        Vec::with_capacity(inbox_files.len());

    // Compute initial effective inactive state once, then update incrementally as only
    // 591/592 statements or new nodes are appended!
    let (mut inactive_ids, mut superseded_by, _retracted_by) =
        compute_effective_inactive_ids(&working_nodes);
    let mut active_nodes_by_tokens: HashMap<Vec<DcShorthand>, Vec<u128>> =
        HashMap::with_capacity(working_nodes.len());
    for n in &working_nodes {
        if !inactive_ids.contains(&n.id) {
            if let Some((verb, _, _)) = parse_statement_tokens(&n.tokens)
                && (verb == sh_supersedes || verb == sh_retracts)
            {
                continue;
            }
            active_nodes_by_tokens
                .entry(n.tokens.clone())
                .or_default()
                .push(n.id);
        }
    }

    for (inbox_name, inbox_sha256, content) in inbox_files {
        let parsed = parse_dcsh_inbox_file(content, inbox_name)?;
        let next_id = SYSTEM_REGION_START
            .saturating_add(u128::try_from(working_nodes.len())?);
        ensure!(
            next_id <= SYSTEM_REGION_END,
            "{inbox_name}: system graph region exhausted at l{next_id}"
        );

        // 1. Validate that all Long(ref_id) references are strictly `< next_id` and exist.
        for &tok in &parsed.tokens {
            if let DcShorthand::Long(ref_id) = tok {
                ensure!(
                    ref_id < next_id && node_by_id.contains_key(&ref_id),
                    "{inbox_name} (l{next_id}): invalid self or forward reference to non-existent node l{ref_id}"
                );
            }
        }

        // 2. Validate Dc 348 "Ambiguous values" syntax or relationship statement syntax.
        let mut is_revision_stmt = false;
        if parsed.tokens.first() == Some(&DcShorthand::Short(348)) {
            let Some(cands) = parse_ambiguous_values_tokens(&parsed.tokens)
            else {
                bail!(
                    "{inbox_name} (l{next_id}): malformed Dc 348 'Ambiguous values' syntax; expected '348 310 275 l<cand_1> 312 l<cand_2> ... 311'"
                );
            };
            let mut unique_cands = HashSet::with_capacity(cands.len());
            for cand_id in cands {
                ensure!(
                    cand_id < next_id && node_by_id.contains_key(&cand_id),
                    "{inbox_name} (l{next_id}): Dc 348 candidate l{cand_id} does not exist"
                );
                ensure!(
                    unique_cands.insert(cand_id),
                    "{inbox_name} (l{next_id}): duplicate candidate l{cand_id} in Dc 348 list"
                );
            }
        } else if let Some(&first_tok) = parsed.tokens.first()
            && is_relationship_verb_with(first_tok, framing)
        {
            let Some((verb, subj, obj)) =
                parse_statement_tokens(&parsed.tokens)
            else {
                bail!(
                    "{inbox_name} (l{next_id}): malformed relationship statement starting with {first_tok}"
                );
            };

            if verb == sh_supersedes {
                is_revision_stmt = true;
                let (DcShorthand::Long(new_id), DcShorthand::Long(old_id)) =
                    (subj, obj)
                else {
                    bail!(
                        "{inbox_name} (l{next_id}): Dc 591 'Supersedes' requires Long node IDs '591 l<new_id> l<old_id>'"
                    );
                };
                ensure!(
                    new_id != old_id,
                    "{inbox_name} (l{next_id}): self-supersession '591 l{new_id} l{old_id}' is forbidden"
                );
                ensure!(
                    !inactive_ids.contains(&old_id),
                    "{inbox_name} (l{next_id}): cannot supersede l{old_id} because it is already superseded or retracted"
                );
                ensure!(
                    !inactive_ids.contains(&new_id),
                    "{inbox_name} (l{next_id}): replacement node l{new_id} is already inactive"
                );
                let mut cur = new_id;
                while let Some(&next_sup) = superseded_by.get(&cur) {
                    ensure!(
                        next_sup != old_id,
                        "{inbox_name} (l{next_id}): supersession cycle detected between l{new_id} and l{old_id}"
                    );
                    cur = next_sup;
                }
                if parsed.provenance == NodeProvenance::Manual {
                    ensure!(
                        working_prov.owner(old_id) == NodeProvenance::Manual,
                        "{inbox_name} (l{next_id}): cross-ownership violation: manual node cannot supersede CSV-owned node l{old_id}"
                    );
                }
            } else if verb == sh_retracts {
                is_revision_stmt = true;
                let DcShorthand::Long(old_id) = obj else {
                    bail!(
                        "{inbox_name} (l{next_id}): Dc 592 'Retracts' object must be a Long node ID 'l<old_id>'"
                    );
                };
                ensure!(
                    subj == sh_enclosing_doc,
                    "{inbox_name} (l{next_id}): Dc 592 'Retracts' subject must be EnclosingDocument ({sh_enclosing_doc})"
                );
                ensure!(
                    !inactive_ids.contains(&old_id),
                    "{inbox_name} (l{next_id}): cannot retract l{old_id} because it is already superseded or retracted"
                );
                if parsed.provenance == NodeProvenance::Manual {
                    ensure!(
                        working_prov.owner(old_id) == NodeProvenance::Manual,
                        "{inbox_name} (l{next_id}): cross-ownership violation: manual node cannot retract CSV-owned node l{old_id}"
                    );
                }
            } else if verb == sh_has_value
                || (verb == sh_has_a && obj == sh_preferred)
            {
                let DcShorthand::Long(has_a_id) = subj else {
                    bail!(
                        "{inbox_name} (l{next_id}): {verb} subject must be a Long 'Has a' statement ID 'l<has_a_id>'"
                    );
                };
                let Some(target_node) = node_by_id.get(&has_a_id) else {
                    bail!(
                        "{inbox_name} (l{next_id}): target 'Has a' node l{has_a_id} does not exist"
                    );
                };
                let check_node_is_has_a = |n: &LedgerNode| -> bool {
                    parse_statement_tokens(&n.tokens)
                        .is_some_and(|(v, _, o)| v == sh_has_a && o != sh_preferred)
                };
                let mut is_has_a = check_node_is_has_a(target_node);
                if !is_has_a {
                    // If `has_a_id` has been superseded by a Dc 348 "Ambiguous values" node,
                    // accept if any candidate of that ambiguity node is a property `Has a` statement.
                    let mut cur = has_a_id;
                    while let Some(&next_sup) = superseded_by.get(&cur) {
                        cur = next_sup;
                    }
                    if let Some(eff_node) = node_by_id.get(&cur)
                        && let Some(cands) =
                            parse_ambiguous_values_tokens(&eff_node.tokens)
                    {
                        is_has_a = cands.iter().any(|cand_id| {
                            node_by_id
                                .get(cand_id)
                                .is_some_and(check_node_is_has_a)
                        });
                    }
                }
                ensure!(
                    is_has_a,
                    "{inbox_name} (l{next_id}): target l{has_a_id} is not a property 'Has a' statement"
                );
                if parsed.provenance == NodeProvenance::Manual {
                    ensure!(
                        working_prov.owner(has_a_id) == NodeProvenance::Manual,
                        "{inbox_name} (l{next_id}): cross-ownership violation: manual qualifier/value cannot attach to CSV-owned 'Has a' node l{has_a_id}"
                    );
                }
            }
        }

        // 3. Validate `# @allow-duplicate-of` against currently active nodes with identical tokens.
        if !is_revision_stmt {
            // Reason for fallback: token sequences with no existing active nodes have zero active duplicates.
            let mut active_matches = active_nodes_by_tokens
                .get(&parsed.tokens)
                .cloned()
                .unwrap_or_default();
            active_matches.retain(|id| !inactive_ids.contains(id));
            active_matches.sort_unstable();
            ensure!(
                parsed.allow_duplicate_of == active_matches,
                "{inbox_name} (l{next_id}): active duplicate mismatch for token body {:?}: expected # @allow-duplicate-of: {:?}, found {:?}",
                parsed.tokens,
                active_matches,
                parsed.allow_duplicate_of
            );
        } else {
            ensure!(
                parsed.allow_duplicate_of.is_empty(),
                "{inbox_name} (l{next_id}): revision statements (591/592) must not specify # @allow-duplicate-of"
            );
        }

        // Reason for fallback: .dcsh files without an explicit '# @comment:' header or inline comment emit an uncommented .ctbnodes line.
        let comment = parsed.comment.unwrap_or_default();
        let new_node = LedgerNode {
            id: next_id,
            tokens: parsed.tokens.clone(),
            comment: comment.clone(),
        };
        let raw_line = new_node.format_line();
        let target_chunk = chunk_filename_for_node_id(next_id);
        let allocated_name = format!("{next_id}_{inbox_name}");

        working_nodes.push(new_node.clone());
        node_by_id.insert(next_id, new_node);
        working_prov.insert(NodeProvenanceEntry {
            node_id: next_id,
            provenance: parsed.provenance,
            rationale: parsed.rationale.clone(),
            allocated_file: Some(allocated_name.clone()),
        });

        if is_revision_stmt {
            // Check if this 591/592 statement targets an earlier 591/592 revision statement
            // (revision cancellation!) or an ordinary node.
            if let Some((verb, subj, DcShorthand::Long(old_id))) =
                parse_statement_tokens(&parsed.tokens)
            {
                let old_was_revision = node_by_id
                    .get(&old_id)
                    .and_then(|n| parse_statement_tokens(&n.tokens))
                    .is_some_and(|(v, _, _)| {
                        v == sh_supersedes || v == sh_retracts
                    });
                if old_was_revision {
                    // Full recompute when a revision statement itself is cancelled so its target is reactivated!
                    let (re_inactive, re_sup, _) =
                        compute_effective_inactive_ids(&working_nodes);
                    inactive_ids = re_inactive;
                    superseded_by = re_sup;
                    active_nodes_by_tokens.clear();
                    for n in &working_nodes {
                        if !inactive_ids.contains(&n.id) {
                            if let Some((v, _, _)) =
                                parse_statement_tokens(&n.tokens)
                                && (v == sh_supersedes || v == sh_retracts)
                            {
                                continue;
                            }
                            active_nodes_by_tokens
                                .entry(n.tokens.clone())
                                .or_default()
                                .push(n.id);
                        }
                    }
                } else {
                    inactive_ids.insert(old_id);
                    if verb == sh_supersedes
                        && let DcShorthand::Long(new_id) = subj
                    {
                        superseded_by.insert(old_id, new_id);
                    }
                }
            }
        } else {
            active_nodes_by_tokens
                .entry(parsed.tokens)
                .or_default()
                .push(next_id);
        }

        planned_items.push(PlannedLedgerItem {
            node_id: next_id,
            raw_line,
            target_chunk,
            tokens: Vec::new(),
            comment: String::new(),
            provenance: parsed.provenance.as_str().to_string(),
            rationale: parsed.rationale,
            allow_duplicate_of: parsed.allow_duplicate_of,
            inbox_name: Some(inbox_name.clone()),
            allocated_name: Some(allocated_name),
            inbox_sha256: Some(inbox_sha256.clone()),
            dcsh_content: None,
        });
    }

    Ok(planned_items)
}

/// Processes all `.dcsh` files in `<repo_root>/src/formats/dcdata/data-src/nodes/inbox/`
/// inside a single durable ledger transaction under `LedgerWriteGuard` (`LOCK_EX`).
pub fn process_inbox_nodes(
    repo_root: &Path,
    required_manifest_path: Option<&Path>,
    dry_run: bool,
) -> Result<DurableTransactionOutcome<()>> {
    process_inbox_nodes_with_fault(
        repo_root,
        required_manifest_path,
        dry_run,
        None,
    )
}

/// Same as [`process_inbox_nodes`], with optional [`FaultInjectionPoint`] for crash/recovery testing.
pub fn process_inbox_nodes_with_fault(
    repo_root: &Path,
    required_manifest_path: Option<&Path>,
    dry_run: bool,
    fault: Option<FaultInjectionPoint>,
) -> Result<DurableTransactionOutcome<()>> {
    let inbox_dir = repo_root.join("src/formats/dcdata/data-src/nodes/inbox");
    let outcome = execute_durable_ledger_transaction_with_planner(
        repo_root,
        "inbox_allocation",
        "inbox",
        required_manifest_path,
        dry_run,
        fault,
        |_write_guard, baseline_nodes, committed_pairs| {
            let prov_map = load_and_validate_node_provenance(
                repo_root,
                baseline_nodes,
                committed_pairs,
            )?;

            let mut sorted_files: BTreeMap<String, (String, String)> =
                BTreeMap::new();
            if inbox_dir.is_dir() {
                for entry in fs::read_dir(&inbox_dir).with_context(|| {
                    format!("Failed to read {}", inbox_dir.display())
                })? {
                    let entry = entry?;
                    let path = entry.path();
                    if !path.is_file() {
                        continue;
                    }
                    if path.extension().and_then(|e| e.to_str()) != Some("dcsh")
                    {
                        continue;
                    }
                    let Some(name) =
                        path.file_name().and_then(|n| n.to_str())
                    else {
                        continue;
                    };
                    let raw_bytes = fs::read(&path).with_context(|| {
                        format!("Failed to read {}", path.display())
                    })?;
                    let sha = sha256_bytes_hex(&raw_bytes);
                    let text = String::from_utf8(raw_bytes).with_context(
                        || format!("Invalid UTF-8 in {}", path.display()),
                    )?;
                    sorted_files.insert(name.to_string(), (sha, text));
                }
            }

            let mut pinned_input_sha256s = BTreeMap::new();
            let mut inbox_vec = Vec::with_capacity(sorted_files.len());
            for (name, (sha, text)) in sorted_files {
                pinned_input_sha256s.insert(name.clone(), sha.clone());
                inbox_vec.push((name, sha, text));
            }

            let planned = prevalidate_and_plan_inbox_items(
                baseline_nodes,
                &prov_map,
                &inbox_vec,
            )?;
            Ok((planned, pinned_input_sha256s, ()))
        },
    )?;

    if !dry_run {
        let _ = verify_repo_committed_node_provenance(repo_root)?;
    }

    Ok(outcome)
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
    use crate::ctbnbdl::{
        JOURNAL_INITIALIZED_REL_PATH, LedgerWriteGuard, TransactionManifest,
        compile_nodes_slice_to_ctbnbdl_with_write_guard,
        compute_ledger_dir_sha256, compute_nodes_slice_ledger_sha256,
        format_chunk_filename, load_ledger_nodes_from_str,
        recover_pending_ledger_journal, sync_highwater_shadow_ledgers,
        write_ledger_nodes_to_dir,
    };
    use crate::dc::DC_CANONICAL_NAME;
    use crate::dc_def::DcDefn;
    use crate::graph_ledger::{
        encode_statement_tokens, encode_string_value_tokens,
        update_ledger_in_memory, update_ledger_in_memory_with_provenance,
    };
    use crate::shared::{BidiClass, GeneralCategory};
    use crate::syntax::parse_dc_syntax;
    use std::collections::BTreeSet;
    use tempfile::TempDir;

    fn sample_dc_defn(short_id: usize, name: &str, desc: &str) -> DcDefn {
        DcDefn {
            dc_id: 1_114_112 + u128::try_from(short_id).unwrap(),
            short_id: Some(short_id),
            ident: Some(format!("DC_TEST_{short_id}")),
            role_def: false,
            title: None,
            name: name.to_string(),
            category: "semantic".to_string(),
            combining_class: 0,
            bidi_class: BidiClass::BN,
            casing_partner: None,
            general_category: GeneralCategory::NonUnicodeControl,
            script: ".Semantic".to_string(),
            is_deprecated: false,
            decompositions: Vec::new(),
            aliases: vec!["test-alias".to_string()],
            formal_aliases: Vec::new(),
            cross_references: Vec::new(),
            annotations: Vec::new(),
            implies: vec!["571".to_string()],
            based_on: Vec::new(),
            os_associations: Vec::new(),
            magic_names: Vec::new(),
            syntax: Some(parse_dc_syntax(":~").unwrap()),
            closes: None,
            description: desc.to_string(),
            format: None,
            source_file: "test.csv".to_string(),
            line_number: 2,
        }
    }

    fn setup_minimal_repo_for_inbox_tests() -> Result<(TempDir, Vec<LedgerNode>)>
    {
        let tmp = TempDir::new()?;
        let root = tmp.path();
        fs::write(root.join(".ctoolbox_workspace_root"), b"")?;
        fs::write(root.join("Cargo.toml"), b"[workspace]\n")?;
        fs::create_dir_all(root.join("src/formats/dcdata/data-src/nodes/inbox"))?;
        fs::create_dir_all(
            root.join("src/formats/dcdata/data-src/nodes/allocated"),
        )?;
        fs::create_dir_all(root.join("assets/data/graph/system"))?;
        fs::create_dir_all(root.join(".git/ctb-graph-baseline"))?;

        // Build a minimal self-bootstrapping framing graph
        let framing = default_ctbnbdl_framing_dcs();
        let required_idents: &[(&str, DcShorthand)] = &[
            ("ProgrammaticIdentifier", DcShorthand::Short(574)),
            ("BeginGraphNodeDefinition", framing.begin_node),
            ("EndGraphNodeDefinition", framing.end_node),
            ("Escape", framing.escape),
            ("EvaluateValueAsAtom", framing.evaluate_as_atom),
            ("HasA", framing.has_a),
            ("HasValue", framing.has_value),
            ("Preferred", framing.preferred),
            ("Supersedes", framing.supersedes),
            ("Retracts", framing.retracts),
            ("EnclosingDocument", framing.enclosing_document),
            ("IsInstanceOf", framing.is_instance_of),
            ("IsSubtypeOf", framing.is_subtype_of),
            ("IsBasedOn", framing.is_based_on),
            ("IsDeprecated", framing.is_deprecated),
            ("IsClosingDelimiterFor", framing.is_closing_delimiter_for),
            ("IsAssociatedWith", framing.is_associated_with),
        ];

        let mut nodes = Vec::new();
        let mut next_id = SYSTEM_REGION_START;
        let sh_prog_ident = DcShorthand::Short(574);
        for &(ident, subj) in required_idents {
            let str_id = next_id;
            nodes.push(LedgerNode {
                id: str_id,
                tokens: encode_string_value_tokens(ident),
                comment: format!("{ident:?}"),
            });
            next_id += 1;

            let has_a_id = next_id;
            nodes.push(LedgerNode {
                id: has_a_id,
                tokens: encode_statement_tokens(
                    framing.has_a,
                    subj,
                    sh_prog_ident,
                ),
                comment: format!("Has a ({subj}, 574)"),
            });
            next_id += 1;

            let has_val_id = next_id;
            nodes.push(LedgerNode {
                id: has_val_id,
                tokens: encode_statement_tokens(
                    framing.has_value,
                    DcShorthand::Long(has_a_id),
                    DcShorthand::Long(str_id),
                ),
                comment: format!("Has value (l{has_a_id}, l{str_id})"),
            });
            next_id += 1;
        }

        let graph_dir = root.join("assets/data/graph");
        let write_guard = LedgerWriteGuard::acquire(root)?;
        write_ledger_nodes_to_dir(&graph_dir, &nodes)?;
        sync_highwater_shadow_ledgers(root, &nodes)?;
        let (active_bundles_sha256, history_bundles_sha256) =
            compile_nodes_slice_to_ctbnbdl_with_write_guard(
                &write_guard,
                &nodes,
                &graph_dir,
                framing,
            )?;

        let sha = compute_ledger_dir_sha256(&graph_dir)?;
        let last_id = nodes.last().unwrap().id;
        let pending = JournalRecord {
            state: "BATCH_PENDING".to_string(),
            batch_id: "bootstrap-test".to_string(),
            kind: "bootstrap_checkpoint".to_string(),
            baseline_last_id: last_id,
            baseline_ledger_sha256: sha.clone(),
            final_last_id: last_id,
            final_ledger_sha256: sha.clone(),
            manifest_sha256: None,
            items: Vec::new(),
            active_bundles_sha256: BTreeMap::new(),
            history_bundles_sha256: BTreeMap::new(),
            record_sha256: String::new(),
        }
        .with_computed_sha256()?;
        let committed = JournalRecord {
            state: "BATCH_COMMITTED".to_string(),
            batch_id: "bootstrap-test".to_string(),
            kind: "bootstrap_checkpoint".to_string(),
            baseline_last_id: last_id,
            baseline_ledger_sha256: sha.clone(),
            final_last_id: last_id,
            final_ledger_sha256: sha,
            manifest_sha256: None,
            items: Vec::new(),
            active_bundles_sha256,
            history_bundles_sha256,
            record_sha256: String::new(),
        }
        .with_computed_sha256()?;
        let journal_content = format!(
            "{}\n{}\n",
            serde_json::to_string(&pending)?,
            serde_json::to_string(&committed)?
        );
        fs::write(root.join(LEDGER_JOURNAL_REL_PATH), journal_content)?;
        fs::write(
            root.join(JOURNAL_INITIALIZED_REL_PATH),
            b"initialized\n",
        )?;

        Ok((tmp, nodes))
    }

    #[crate::ctb_test]
    fn test_inbox_prevalidation_rejects_malformed_and_invalid_dcsh_inputs()
    -> Result<()> {
        let (_tmp, baseline_nodes) = setup_minimal_repo_for_inbox_tests()?;
        let prov_map = NodeProvenanceMap::default();
        let next_id = SYSTEM_REGION_START
            + u128::try_from(baseline_nodes.len()).unwrap();

        // 1. Missing `# @provenance`
        assert!(
            prevalidate_and_plan_inbox_items(
                &baseline_nodes,
                &prov_map,
                &[(
                    "01.dcsh".to_string(),
                    "sha".to_string(),
                    "u61 u62\n".to_string()
                )],
            )
            .is_err()
        );

        // 2. `# @provenance: manual` without `# @rationale:`
        assert!(
            prevalidate_and_plan_inbox_items(
                &baseline_nodes,
                &prov_map,
                &[(
                    "01.dcsh".to_string(),
                    "sha".to_string(),
                    "# @provenance: manual\nu61 u62\n".to_string()
                )],
            )
            .is_err()
        );

        // 3. Self-reference (`l<next_id>`) and forward reference (`l<next_id + 1>`)
        assert!(
            prevalidate_and_plan_inbox_items(
                &baseline_nodes,
                &prov_map,
                &[(
                    "01.dcsh".to_string(),
                    "sha".to_string(),
                    format!("# @provenance: csv\n586 l{next_id} 571\n"),
                )],
            )
            .is_err()
        );
        assert!(
            prevalidate_and_plan_inbox_items(
                &baseline_nodes,
                &prov_map,
                &[(
                    "01.dcsh".to_string(),
                    "sha".to_string(),
                    format!("# @provenance: csv\n586 l{} 571\n", next_id + 1),
                )],
            )
            .is_err()
        );

        // 4. Malformed `Dc 348 "Ambiguous values"` list (< 2 candidates or duplicate candidates)
        assert!(
            prevalidate_and_plan_inbox_items(
                &baseline_nodes,
                &prov_map,
                &[(
                    "01.dcsh".to_string(),
                    "sha".to_string(),
                    format!(
                        "# @provenance: csv\n348 310 275 l{SYSTEM_REGION_START} 311\n"
                    ),
                )],
            )
            .is_err()
        );
        assert!(
            prevalidate_and_plan_inbox_items(
                &baseline_nodes,
                &prov_map,
                &[(
                    "01.dcsh".to_string(),
                    "sha".to_string(),
                    format!(
                        "# @provenance: csv\n348 310 275 l{SYSTEM_REGION_START} 312 l{SYSTEM_REGION_START} 311\n"
                    ),
                )],
            )
            .is_err()
        );

        // 5. Attaching `610` (`Has value`) to a non-`584` (`Has a`) node (`l8589934592` is a string value node)
        assert!(
            prevalidate_and_plan_inbox_items(
                &baseline_nodes,
                &prov_map,
                &[(
                    "01.dcsh".to_string(),
                    "sha".to_string(),
                    format!(
                        "# @provenance: csv\n610 l{SYSTEM_REGION_START} l{SYSTEM_REGION_START}\n"
                    ),
                )],
            )
            .is_err()
        );

        // 6. Unauthorized active duplicate (`"ProgrammaticIdentifier"` string tokens already exist at `l8589934592`)
        let dup_body = baseline_nodes[0]
            .tokens
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" ");
        assert!(
            prevalidate_and_plan_inbox_items(
                &baseline_nodes,
                &prov_map,
                &[(
                    "01.dcsh".to_string(),
                    "sha".to_string(),
                    format!("# @provenance: csv\n{dup_body}\n"),
                )],
            )
            .is_err()
        );

        // 7. Authorized active duplicate succeeds when `# @allow-duplicate-of: l8589934592` is declared
        let planned = prevalidate_and_plan_inbox_items(
            &baseline_nodes,
            &prov_map,
            &[(
                "01.dcsh".to_string(),
                "sha".to_string(),
                format!(
                    "# @provenance: csv\n# @allow-duplicate-of: l{SYSTEM_REGION_START}\n{dup_body}\n"
                ),
            )],
        )?;
        assert_eq!(planned.len(), 1);
        assert_eq!(planned[0].node_id, next_id);

        // 8. Revision cancellation (`592 615 l<retract_stmt_id>`) reactivates the retracted node,
        // so a subsequent duplicate of that node must specify `# @allow-duplicate-of`!
        let mut batch = Vec::new();
        // Item 0 (`next_id`): new unique value node `u78 u79`
        batch.push((
            "01_val.dcsh".to_string(),
            "s1".to_string(),
            "# @provenance: csv\nu78 u79\n".to_string(),
        ));
        // Item 1 (`next_id + 1`): retract `next_id` (`592 615 l<next_id>`)
        batch.push((
            "02_retract.dcsh".to_string(),
            "s2".to_string(),
            format!("# @provenance: csv\n592 615 l{next_id}\n"),
        ));
        // Item 2 (`next_id + 2`): cancel the retraction (`592 615 l<next_id + 1>`) -> reactivates `next_id`!
        batch.push((
            "03_cancel_retract.dcsh".to_string(),
            "s3".to_string(),
            format!("# @provenance: csv\n592 615 l{}\n", next_id + 1),
        ));
        // Item 3 (`next_id + 3`): `u78 u79` WITHOUT `# @allow-duplicate-of` must FAIL because `next_id` is active again!
        let mut bad_batch = batch.clone();
        bad_batch.push((
            "04_dup_unauth.dcsh".to_string(),
            "s4".to_string(),
            "# @provenance: csv\nu78 u79\n".to_string(),
        ));
        assert!(
            prevalidate_and_plan_inbox_items(
                &baseline_nodes,
                &prov_map,
                &bad_batch
            )
            .is_err()
        );

        // With `# @allow-duplicate-of: l<next_id>`, item 3 succeeds!
        batch.push((
            "04_dup_auth.dcsh".to_string(),
            "s4".to_string(),
            format!(
                "# @provenance: csv\n# @allow-duplicate-of: l{next_id}\nu78 u79\n"
            ),
        ));
        let ok_planned =
            prevalidate_and_plan_inbox_items(&baseline_nodes, &prov_map, &batch)?;
        assert_eq!(ok_planned.len(), 4);

        Ok(())
    }

    #[crate::ctb_test]
    fn test_inbox_and_manifest_resume_after_partial_append_and_move()
    -> Result<()> {
        let (temp_dir, baseline_nodes) = setup_minimal_repo_for_inbox_tests()?;
        let root = temp_dir.path();
        let inbox_dir = root.join("src/formats/dcdata/data-src/nodes/inbox");
        let alloc_dir =
            root.join("src/formats/dcdata/data-src/nodes/allocated");
        let graph_dir = root.join("assets/data/graph");

        let f1_content =
            "# @provenance: csv\n# @comment: \"item1\"\nu61 u62\n".to_string();
        let f2_content =
            "# @provenance: csv\n# @comment: \"item2\"\nu63 u64\n".to_string();
        let f3_content =
            "# @provenance: csv\n# @comment: \"item3\"\nu65 u66\n".to_string();

        fs::write(inbox_dir.join("001_a.dcsh"), &f1_content)?;
        fs::write(inbox_dir.join("002_b.dcsh"), &f2_content)?;
        fs::write(inbox_dir.join("003_c.dcsh"), &f3_content)?;

        // Build an approved manifest for the 3-item inbox batch
        let prov_map = NodeProvenanceMap::default();
        let inbox_vec = vec![
            (
                "001_a.dcsh".to_string(),
                sha256_bytes_hex(f1_content.as_bytes()),
                f1_content.clone(),
            ),
            (
                "002_b.dcsh".to_string(),
                sha256_bytes_hex(f2_content.as_bytes()),
                f2_content.clone(),
            ),
            (
                "003_c.dcsh".to_string(),
                sha256_bytes_hex(f3_content.as_bytes()),
                f3_content.clone(),
            ),
        ];
        let planned = prevalidate_and_plan_inbox_items(
            &baseline_nodes,
            &prov_map,
            &inbox_vec,
        )?;
        let mut final_nodes = baseline_nodes.clone();
        for item in &planned {
            let parsed = load_ledger_nodes_from_str(
                &format!("{}\n", item.raw_line),
                &item.target_chunk,
            )?;
            final_nodes.extend(parsed);
        }
        let mut pinned_input_sha256s = BTreeMap::new();
        for (name, sha, _) in &inbox_vec {
            pinned_input_sha256s.insert(name.clone(), sha.clone());
        }
        let manifest = TransactionManifest {
            manifest_id: "test-inbox-3".to_string(),
            kind: "inbox_allocation".to_string(),
            baseline_last_id: baseline_nodes.last().unwrap().id,
            baseline_node_count: baseline_nodes.len(),
            baseline_ledger_sha256: compute_nodes_slice_ledger_sha256(
                &baseline_nodes,
            ),
            final_last_id: final_nodes.last().unwrap().id,
            final_node_count: final_nodes.len(),
            final_ledger_sha256: compute_nodes_slice_ledger_sha256(
                &final_nodes,
            ),
            pinned_input_sha256s,
            planned_items: planned.clone(),
        };
        let manifest_path = root.join("manifest.json");
        fs::write(
            &manifest_path,
            serde_json::to_string_pretty(&manifest).unwrap(),
        )?;

        // 1. Dry-run with `--require-manifest` mutates zero files
        let dry_outcome =
            process_inbox_nodes(root, Some(&manifest_path), true)?;
        assert_eq!(dry_outcome.appended_count, 3);
        assert_eq!(dry_outcome.files_written, 0);
        assert!(inbox_dir.join("001_a.dcsh").is_file());

        // 2. Inject a crash after item 0 (`001_a.dcsh`) is appended to `.ctbnodes`, synced to shadows,
        // AND moved from `nodes/inbox/001_a.dcsh` to `nodes/allocated/<id>_001_a.dcsh`!
        let err = process_inbox_nodes_with_fault(
            root,
            Some(&manifest_path),
            false,
            Some(FaultInjectionPoint::AfterItemAppendedAndMoved {
                item_index: 0,
            }),
        )
        .unwrap_err();
        assert!(format!("{err:#}").contains("Simulated fault injection"));

        // Verify item 0 was moved out of `inbox/` into `allocated/`, while items 1 and 2 remain in `inbox/`
        assert!(!inbox_dir.join("001_a.dcsh").exists());
        assert!(
            alloc_dir
                .join(planned[0].allocated_name.as_deref().unwrap())
                .is_file()
        );
        assert!(inbox_dir.join("002_b.dcsh").is_file());
        assert!(inbox_dir.join("003_c.dcsh").is_file());

        // External gated reader (`load_ledger_nodes_from_dir`) MUST reject the open `BATCH_PENDING` journal!
        assert!(load_ledger_nodes_from_dir(&graph_dir).is_err());

        // 3. Retry the EXACT SAME `process_inbox_nodes(root, Some(&manifest_path), false)` call!
        // It must take the Manifest-Enforced Resume Path (accepting the partially appended ledger
        // and the already-moved `001_a.dcsh` file without replanning or allocating new IDs) and finish!
        let resumed_outcome =
            process_inbox_nodes(root, Some(&manifest_path), false)?;
        assert!(resumed_outcome.resumed_or_already_committed);
        assert_eq!(resumed_outcome.total_nodes, final_nodes.len());

        // All 3 inbox files are now in `nodes/allocated/`, `nodes/inbox/` is empty,
        // and external gated reader + provenance validator succeed!
        assert!(!inbox_dir.join("002_b.dcsh").exists());
        assert!(!inbox_dir.join("003_c.dcsh").exists());
        let loaded = load_ledger_nodes_from_dir(&graph_dir)?;
        assert_eq!(loaded.len(), final_nodes.len());
        let prov = verify_repo_committed_node_provenance(root)?;
        assert_eq!(prov.owner(planned[0].node_id), NodeProvenance::Csv);
        assert_eq!(prov.owner(planned[1].node_id), NodeProvenance::Csv);
        assert_eq!(prov.owner(planned[2].node_id), NodeProvenance::Csv);

        // 4. Re-running the exact same `--require-manifest` command after commit is a clean idempotent no-op!
        let idempotent_outcome =
            process_inbox_nodes(root, Some(&manifest_path), false)?;
        assert!(idempotent_outcome.resumed_or_already_committed);
        assert_eq!(idempotent_outcome.total_nodes, final_nodes.len());

        // 5. Also test torn `BATCH_COMMITTED` recovery on a subsequent manual node
        let next_manual_id = final_nodes.last().unwrap().id + 1;
        let manual_dcsh = "# @provenance: manual\n# @rationale: Manual test node\n# @comment: \"manual\"\nu6d u6e\n";
        fs::write(inbox_dir.join("004_manual.dcsh"), manual_dcsh)?;
        let err = process_inbox_nodes_with_fault(
            root,
            None,
            false,
            Some(FaultInjectionPoint::TornCommittedRecord {
                bytes_to_keep: 24,
            }),
        )
        .unwrap_err();
        assert!(format!("{err:#}").contains("Simulated fault injection"));
        assert!(load_ledger_nodes_from_dir(&graph_dir).is_err());

        // Standalone recovery finishes the torn `BATCH_COMMITTED` record!
        assert!(recover_pending_ledger_journal(root, None, false)?);
        let prov2 = verify_repo_committed_node_provenance(root)?;
        assert_eq!(prov2.owner(next_manual_id), NodeProvenance::Manual);
        assert_eq!(
            prov2.get(next_manual_id).unwrap().rationale.as_deref(),
            Some("Manual test node")
        );

        // 6. Tampering with an allocated `.dcsh` file in `nodes/allocated/` is caught by `verify_repo_committed_node_provenance`!
        let manual_alloc_path =
            alloc_dir.join(format!("{next_manual_id}_004_manual.dcsh"));
        let orig_bytes = fs::read(&manual_alloc_path)?;
        fs::write(&manual_alloc_path, b"# tampered\n")?;
        assert!(verify_repo_committed_node_provenance(root).is_err());
        fs::write(&manual_alloc_path, orig_bytes)?;
        assert!(verify_repo_committed_node_provenance(root).is_ok());

        // 7. Adding an unrecorded `.dcsh` file in `nodes/allocated/` is caught!
        let rogue_path = alloc_dir.join("9999999999_rogue.dcsh");
        fs::write(&rogue_path, manual_dcsh)?;
        assert!(verify_repo_committed_node_provenance(root).is_err());
        fs::remove_file(&rogue_path)?;
        assert!(verify_repo_committed_node_provenance(root).is_ok());

        Ok(())
    }

    #[crate::ctb_test]
    fn test_manual_vs_csv_cross_ownership_protection() -> Result<()> {
        let mut defn =
            sample_dc_defn(900, "Original name", "Original description");
        let scripts = BTreeSet::from([".Semantic".to_string()]);

        let (mut nodes, _) =
            update_ledger_in_memory(Vec::new(), &[defn.clone()], &[], &scripts, None)?;

        // Append a manual direct statement node (`586 c900 571` is already CSV; add `588 c900 571` as Manual)
        let manual_stmt_id =
            SYSTEM_REGION_START + u128::try_from(nodes.len()).unwrap();
        nodes.push(LedgerNode {
            id: manual_stmt_id,
            tokens: encode_statement_tokens(
                DcShorthand::Short(588),
                DcShorthand::Short(900),
                DcShorthand::Short(571),
            ),
            comment: "Manual based_on".to_string(),
        });
        let mut prov_map = NodeProvenanceMap::default();
        prov_map.insert(NodeProvenanceEntry {
            node_id: manual_stmt_id,
            provenance: NodeProvenance::Manual,
            rationale: Some("Manual domain link".to_string()),
            allocated_file: Some(format!("{manual_stmt_id}_manual.dcsh")),
        });

        // 1. Running CSV reconciliation preserves the `Manual` statement untouched (0 retractions!)
        let (nodes_after, stats) = update_ledger_in_memory_with_provenance(
            nodes.clone(),
            prov_map.clone(),
            &[defn.clone()],
            &[],
            &scripts,
            None,
        )?;
        assert_eq!(stats.nodes_appended, 0);
        assert_eq!(stats.nodes_retracted, 0);
        assert_eq!(nodes_after.len(), nodes.len());

        // 2. Manual inbox node attempting to retract or supersede a CSV node is rejected in prevalidation
        let csv_node_id = nodes[0].id;
        assert!(
            prevalidate_and_plan_inbox_items(
                &nodes,
                &prov_map,
                &[(
                    "bad_manual_retract.dcsh".to_string(),
                    "sha".to_string(),
                    format!(
                        "# @provenance: manual\n# @rationale: try retract csv\n592 615 l{csv_node_id}\n"
                    ),
                )],
            )
            .is_err()
        );

        // 3. If an active Manual node references a CSV statement node that CSV reconciliation wants to retract,
        // CSV reconciliation aborts with a Cross-ownership conflict error!
        let csv_implies_id = nodes
            .iter()
            .find(|n| {
                n.tokens
                    == encode_statement_tokens(
                        DcShorthand::Short(587),
                        DcShorthand::Short(900),
                        DcShorthand::Short(571),
                    )
            })
            .unwrap()
            .id;
        let manual_ref_id =
            SYSTEM_REGION_START + u128::try_from(nodes.len()).unwrap();
        let mut nodes_with_ref = nodes.clone();
        nodes_with_ref.push(LedgerNode {
            id: manual_ref_id,
            tokens: encode_statement_tokens(
                DcShorthand::Short(588),
                DcShorthand::Short(900),
                DcShorthand::Long(csv_implies_id),
            ),
            comment: "Manual referencing CSV implies stmt".to_string(),
        });
        let mut prov_with_ref = prov_map.clone();
        prov_with_ref.insert(NodeProvenanceEntry {
            node_id: manual_ref_id,
            provenance: NodeProvenance::Manual,
            rationale: Some("References CSV statement".to_string()),
            allocated_file: None,
        });

        // Removing `implies` from `defn` causes CSV reconciliation to try to retract `csv_implies_id`, which must fail!
        let mut defn_no_implies = defn.clone();
        defn_no_implies.implies.clear();
        let err = update_ledger_in_memory_with_provenance(
            nodes_with_ref,
            prov_with_ref,
            &[defn_no_implies],
            &[],
            &scripts,
            None,
        )
        .unwrap_err();
        assert!(format!("{err:#}").contains("Cross-ownership conflict"));

        // 4. If a single-valued property (`572` `Canonical name`) is owned by a `Manual` node and CSV
        // changes the name to a different value, CSV reconciliation aborts with Cross-ownership conflict!
        let canon_name_sh = sh(DC_CANONICAL_NAME);
        let name_has_a_id = nodes
            .iter()
            .find(|n| {
                n.tokens
                    == encode_statement_tokens(
                        DcShorthand::Short(584),
                        DcShorthand::Short(900),
                        canon_name_sh,
                    )
            })
            .unwrap()
            .id;
        let mut prov_manual_name = prov_map;
        prov_manual_name.insert(NodeProvenanceEntry {
            node_id: name_has_a_id,
            provenance: NodeProvenance::Manual,
            rationale: Some("Manual canonical name".to_string()),
            allocated_file: None,
        });
        defn.name = "Conflicting CSV name".to_string();
        let err = update_ledger_in_memory_with_provenance(
            nodes,
            prov_manual_name,
            &[defn],
            &[],
            &scripts,
            None,
        )
        .unwrap_err();
        assert!(format!("{err:#}").contains("Cross-ownership conflict"));

        Ok(())
    }

    #[crate::ctb_test]
    fn test_100th_line_torn_write_and_watcher_sealing_interaction() -> Result<()>
    {
        let (temp_dir, baseline_nodes) = setup_minimal_repo_for_inbox_tests()?;
        let root = temp_dir.path();
        let inbox_dir = root.join("src/formats/dcdata/data-src/nodes/inbox");
        let graph_dir = root.join("assets/data/graph");

        // Pad chunk 1 (`85899_34592.ctbnodes`) to 99 lines so the next inbox item is the 100th line!
        assert!(baseline_nodes.len() < 99);
        let needed_for_99 = 99 - baseline_nodes.len();
        for idx in 0..needed_for_99 {
            let name = format!("pad_{idx:03}.dcsh");
            let body = format!(
                "# @provenance: csv\n# @comment: \"pad-{idx}\"\nu{:x} u{:x}\n",
                0x100 + idx,
                0x200 + idx
            );
            fs::write(inbox_dir.join(name), body)?;
        }
        let outcome = process_inbox_nodes(root, None, false)?;
        assert_eq!(outcome.total_nodes, 99);

        // Now stage the 100th node (`line_100.dcsh`) and inject a torn chunk write on item 0!
        fs::write(
            inbox_dir.join("line_100.dcsh"),
            "# @provenance: csv\n# @comment: \"node-100\"\nu300 u301\n",
        )?;
        let err = process_inbox_nodes_with_fault(
            root,
            None,
            false,
            Some(FaultInjectionPoint::TornChunkLine {
                item_index: 0,
                bytes_to_keep: 10,
            }),
        )
        .unwrap_err();
        assert!(format!("{err:#}").contains("Simulated fault injection"));

        // Verify the chunk file has 99 complete `\n`-terminated lines + 1 unterminated trailing fragment
        let chunk_path = graph_dir.join(format_chunk_filename(SYSTEM_REGION_START));
        let raw_chunk = fs::read(&chunk_path)?;
        assert_ne!(raw_chunk.last().copied(), Some(b'\n'));
        let newline_count = raw_chunk.iter().filter(|&&b| b == b'\n').count();
        assert_eq!(newline_count, 99);

        // Because the 100th line is NOT `\n`-terminated and `BATCH_PENDING` is open, the chunk is NOT sealed
        // and `recover_pending_ledger_journal` can repair the torn 100th line in place and commit!
        assert!(recover_pending_ledger_journal(root, None, false)?);
        let repaired_chunk = fs::read(&chunk_path)?;
        assert_eq!(repaired_chunk.last().copied(), Some(b'\n'));
        let repaired_newlines =
            repaired_chunk.iter().filter(|&&b| b == b'\n').count();
        assert_eq!(repaired_newlines, 100);

        let loaded = load_ledger_nodes_from_dir(&graph_dir)?;
        assert_eq!(loaded.len(), 100);
        Ok(())
    }
}
