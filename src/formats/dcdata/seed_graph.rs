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

//! Runtime Seed Graph Loader & Index (Phase 2).
//!
//! Loads compiled `.ctbnbdl` (`CtbNodeBundle2` / `CtbNodeBundle1`) system graph
//! bundles from the runtime resource bundle (`data/graph/85899_34592.ctbnbdl`, etc.)
//! and indexes active statements, supersessions (`Dc 591`), retractions (`Dc 592`),
//! seed classification entities, value nodes, and `.Semantic:Pattern` rules at runtime.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;
use std::sync::{Arc, OnceLock};

use ctb_storage_minimal::global_graph_layout::{
    SYSTEM_REGION_END, SYSTEM_REGION_START,
};
use ctb_storage_minimal::shorthand::DcShorthand;
use ctb_utilities::format_id::DC_COLLECTIVE_TOOLBOX;

use crate::dc::{
    DC_APPROXIMATE_DECOMPOSITION, DC_BEGIN_ASSOCIATIVITY_GROUP, DC_BEGIN_LIST,
    DC_BEGIN_LITERAL, DC_BEGIN_TYPE_DEF, DC_BEGIN_WASM_PROGRAM,
    DC_BIDIRECTIONAL_CLASS, DC_BOOLEAN_TRUE, DC_CANONICAL_DECOMPOSITION,
    DC_CANONICAL_NAME, DC_CHARACTER_GENERAL_CATEGORY, DC_CHARACTER_SCRIPT,
    DC_COLLECTIVE_TOOLBOX_DC_CSV_SEED_DATABASE,
    DC_COLLECTIVE_TOOLBOX_SEED_GRAPH_DATA_SOURCE, DC_DC_GENERAL_CATEGORY,
    DC_DECOMPOSITION, DC_ENCLOSING_DOCUMENT, DC_END_ASSOCIATIVITY_GROUP,
    DC_END_LIST, DC_ENTITY_TYPE,
    DC_GLOBAL_GRAPH_SYSTEM_BLOCK_MOST_RECENT_NODE_ID, DC_HAS_A, DC_HAS_VALUE,
    DC_IN_CONTEXT_OF, DC_IS_INSTANCE_OF, DC_IS_STABLE, DC_MANUAL_ENTRY,
    DC_PATTERN_REPLACEMENT, DC_PREFERRED, DC_PROGRAMMATIC_IDENTIFIER,
    DC_REPLACEMENT, DC_RETRACTS, DC_SEMANTIC_DECOMPOSITION, DC_SUPERSEDES,
    DC_SYNTAX_RULE, DC_THIS_ELEMENT, DC_TYPE_ROUTINE, DC_UNICODE_SCRIPT,
    DcChar,
};
use crate::graph_ledger::{
    LedgerNode, SeedEntityKey, decode_ctbnbdl_bundle,
    decode_integer_value_tokens, decode_string_value_tokens,
    format_chunk_bundle_filename, parse_chunk_bundle_filename_start_id,
    parse_for_each_statement_tokens, parse_statement_tokens,
    single_valued_property_set,
};
pub use crate::support::seed_graph_reconstruct::ReconstructedNamedType;

/// Default glob pattern matching bundled active system graph `.ctbnbdl` files inside `ctoolbox.rsrc`.
pub const SEED_GRAPH_ASSET_GLOB: &str = "data/graph/*.ctbnbdl";

/// Default glob pattern matching optional historical/inactive system graph `.ctbnbdl` files
/// inside `system_graph_history.rsrc` (`assets/data/graph/system/*.ctbnbdl`).
pub const SYSTEM_GRAPH_HISTORY_ASSET_GLOB: &str = "data/graph/system/*.ctbnbdl";

/// Primary seed graph bundle asset key inside `ctoolbox.rsrc`.
pub const PRIMARY_SEED_GRAPH_ASSET_KEY: &str = "data/graph/85899_34592.ctbnbdl";

static SHARED_SEED_GRAPH: OnceLock<Result<Arc<SeedGraphIndex>, String>> =
    OnceLock::new();

#[must_use]
fn sh(dc: DcChar) -> DcShorthand {
    DcShorthand::from_global_id(dc.to_u128())
}

/// Active reified `Has a` (`Dc 584`) + `Has value` (`Dc 610`) property assignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActivePropertyValue {
    /// Node ID of the `584 <subject> <property>` (`Has a`) statement.
    pub has_a_id: u128,
    /// Node ID of the active `610 l<has_a_id> <value>` (`Has value`) statement.
    pub has_val_id: u128,
    /// Value token assigned by `has_val_id`.
    pub value: DcShorthand,
    /// Optional node ID of an active `584 l<has_a_id> 570` (`Preferred`) statement qualifying `has_a_id`.
    pub preferred_id: Option<u128>,
}

/// Runtime index over the System-block seed graph (`8589934592..=12884901887`).
#[derive(Debug, Clone)]
pub struct SeedGraphIndex {
    /// All loaded `LedgerNode`s in ascending ID order.
    pub(crate) nodes: Vec<LedgerNode>,
    /// Maps node ID (`u128`) to its index in `nodes`.
    pub(crate) node_index_by_id: HashMap<u128, usize>,
    /// Maps a superseded node ID (`old_id`) to the statement node ID (`new_id`)
    /// that superseded it via an active `591 <new_id> l<old_id>` (`Supersedes`) statement.
    pub(crate) superseded_by: HashMap<u128, u128>,
    /// Maps a retracted node ID (`old_id`) to the statement node ID (`retract_id`)
    /// that retracted it via an active `592 615 l<old_id>` (`Retracts`) statement.
    pub(crate) retracted_by: HashMap<u128, u128>,
    /// Set of all inactive node IDs (superseded or retracted).
    pub(crate) inactive_ids: HashSet<u128>,
    /// Decoded UTF-8 strings for string value nodes, keyed by `u128` node ID.
    pub(crate) string_by_node_id: HashMap<u128, String>,
    /// Decoded unsigned integers for `6 f199 .. 7` value nodes, keyed by `u128` node ID.
    pub(crate) integer_by_node_id: HashMap<u128, u128>,
    /// Maps a value node's canonical token sequence to its first allocated `u128` node ID.
    pub(crate) value_node_by_tokens: HashMap<Vec<DcShorthand>, u128>,
    /// Maps a `SeedEntityKey` (`BidiClass`, `GeneralCategory`, `Script`, `NamedConstruct`)
    /// to its allocated `u128` node ID.
    pub(crate) seed_entity_ids: HashMap<SeedEntityKey, u128>,
    /// Reverse lookup from allocated `u128` node ID to its `SeedEntityKey`.
    pub(crate) seed_entity_by_id: HashMap<u128, SeedEntityKey>,
    /// Active single-valued properties: `(subject, prop_token) -> (has_a_id, has_val_id, current_val_token)`.
    pub(crate) active_single_props:
        BTreeMap<(DcShorthand, DcShorthand), (u128, u128, DcShorthand)>,
    /// All active single-valued property assignments per `(subject, prop_token)` in ascending `has_a_id` order.
    pub(crate) single_props_by_subj_prop:
        BTreeMap<(DcShorthand, DcShorthand), Vec<(u128, u128, DcShorthand)>>,
    /// Active multi-valued properties: `(subject, prop_token, val_token) -> (has_a_id, has_val_id)`.
    pub(crate) active_multi_props:
        BTreeMap<(DcShorthand, DcShorthand, DcShorthand), (u128, u128)>,
    /// Active `584 l<has_a_id> 570` (`Preferred`) qualifiers on multi-valued properties:
    /// `(subject, prop_token, val_token) -> pref_qual_id`.
    pub(crate) active_multi_preferred:
        BTreeMap<(DcShorthand, DcShorthand, DcShorthand), u128>,
    /// Active multi-valued properties grouped by `(subject, prop_token)` in ascending `has_a_id` order
    /// (with `Preferred`-qualified entries placed first).
    pub(crate) multi_props_by_subj_prop:
        HashMap<(DcShorthand, DcShorthand), Vec<ActivePropertyValue>>,
    /// Active `<prefer>` replacements: `(subject, val_token) -> (has_a_id, has_val_id, pref_qual_id)`.
    pub(crate) active_preferred:
        BTreeMap<(DcShorthand, DcShorthand), (u128, u128, u128)>,
    /// All active `<prefer>` replacement triples per `(subject, val_token)` in ascending `has_a_id` order.
    pub(crate) preferred_all_by_subj_val:
        BTreeMap<(DcShorthand, DcShorthand), Vec<(u128, u128, u128)>>,
    /// Active `<prefer>` replacement values grouped by `subject` in ascending `has_a_id` order.
    pub(crate) preferred_by_subject: HashMap<DcShorthand, Vec<DcShorthand>>,
    /// Active direct binary statements: `(verb, subject, object) -> stmt_id`.
    pub(crate) active_direct_stmts:
        BTreeMap<(DcShorthand, DcShorthand, DcShorthand), u128>,
    /// All active direct binary statement node IDs per `(verb, subject, object)` in ascending `stmt_id` order.
    pub(crate) direct_stmts_all:
        BTreeMap<(DcShorthand, DcShorthand, DcShorthand), Vec<u128>>,
    /// Active direct binary statement objects grouped by `(verb, subject)` in ascending `stmt_id` order.
    pub(crate) direct_objects_by_verb_subj:
        HashMap<(DcShorthand, DcShorthand), Vec<DcShorthand>>,
    /// Active direct binary statement subjects grouped by `(verb, object)` in ascending `stmt_id` order.
    pub(crate) direct_subjects_by_verb_obj:
        HashMap<(DcShorthand, DcShorthand), Vec<DcShorthand>>,
    /// Active statement node IDs that are structurally orphaned (e.g. an active `584` property
    /// statement with no active `610` value). Multiple active statements for the same property or
    /// on the same `584` node are NOT shadowed here; all remain active in the index.
    pub(crate) orphaned_stmt_ids: Vec<u128>,
    /// In-graph provenance (`Dc 635` `"Collective Toolbox seed graph data source"`) mapped from
    /// target node ID (`u128`) to its declared data source token (e.g. `Dc 636` or `Dc 637`).
    pub(crate) node_data_source: HashMap<u128, DcShorthand>,
    /// Active statement node IDs (`Dc 638` `"For each"`, `Dc 584` `"Has a"`, and `Dc 610` `"Has value"`)
    /// that assert `Dc 635` `"Collective Toolbox seed graph data source"`.
    pub(crate) provenance_stmt_ids: HashSet<u128>,
    /// Set of node IDs loaded from the host-authenticated bundled asset (`ctoolbox.rsrc` /
    /// `system_graph_history.rsrc`), distinguishing bundled System authority from extra or
    /// unbundled resources (such as Wikidata).
    pub(crate) bundled_origin_node_ids: HashSet<u128>,
    /// Maps a superseded node ID (`old_id`) to the statement node ID (`supersede_stmt_id`)
    /// of the `Dc 591` `"Supersedes"` statement itself.
    pub(crate) superseding_stmt_by_old_id: HashMap<u128, u128>,
    /// All active `Dc 635` `"Collective Toolbox seed graph data source"` assertions targeting
    /// a node ID (`target_node_id -> Vec<(has_a_stmt_id, has_val_stmt_id, data_source_tok)>`).
    pub(crate) node_provenance_assertions:
        HashMap<u128, Vec<(u128, u128, DcShorthand)>>,
    /// All statement node IDs (active or inactive) whose subject is `subj`.
    pub(crate) subject_all_stmt_ids: HashMap<DcShorthand, Vec<u128>>,
    /// All `Dc 610` `"Has value"` statement nodes (`(has_val_id, val_tok)`, active or inactive)
    /// targeting a `Dc 584` / `Dc 638` statement node ID (`has_a_id`).
    pub(crate) has_val_stmt_ids_by_has_a:
        HashMap<u128, Vec<(u128, DcShorthand)>>,
    /// Highest allocated node ID in the System block at the time the graph bundle was created
    /// (`Dc 641` `"Global graph system block most recent node ID at time of document creation"`).
    pub(crate) system_block_most_recent_node_id: Option<u128>,
}

/// Verified resolution of an unpinned 1-Short-Dc System supervisor entry document
/// against the application-bundled System-block seed graph (`ctoolbox.rsrc`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrustedSystemSupervisorResolution {
    /// Unpinned Short Dc entry character that was resolved.
    pub entry_dc: DcChar,
    /// Active System-block statement node ID (`Dc 584` `"Has a"` or `Dc 594` `"Syntax rule"`)
    /// binding `entry_dc` to its active definition.
    pub statement_node_id: u128,
    /// Active `Dc 610` `"Has value"` statement node ID assigning the definition node to `statement_node_id`.
    pub value_statement_node_id: u128,
    /// Active System-block definition node ID (`Dc 1927` `"Begin WASM program"` or routine node)
    /// representing the resolved supervisor revision.
    pub supervisor_revision_node_id: u128,
    /// Highest allocated System-block node ID (`HEAD` watermark) of the trusted graph.
    pub system_head_node_id: u128,
    /// Deterministically sorted list of all transitive System-block dependency node IDs
    /// reachable from the supervisor definition (including `supervisor_revision_node_id`).
    pub dependency_snapshot_node_ids: Vec<u128>,
    /// Raw `.WASM` / supervisor definition tokens from `supervisor_revision_node_id`.
    pub definition_tokens: Vec<DcShorthand>,
}

/// Parses a `Dc 348 "Ambiguous values"` list node (`348 310 275 l<cand_1> 312 l<cand_2> ... 311`)
/// into its ordered candidate node IDs.
#[must_use]
pub fn parse_ambiguous_values_tokens(
    tokens: &[DcShorthand],
) -> Option<Vec<u128>> {
    let sh_ambiguous = DcShorthand::Short(348);
    let sh_begin_list = sh(DC_BEGIN_LIST);
    let sh_end_list = sh(DC_END_LIST);
    let sh_graph_node = DcShorthand::Short(275);
    let sh_list_sep = DcShorthand::Short(312);
    if tokens.len() < 4
        || tokens.first() != Some(&sh_ambiguous)
        || tokens.get(1) != Some(&sh_begin_list)
        || tokens.last() != Some(&sh_end_list)
    {
        return None;
    }
    let middle = tokens.get(2..tokens.len().saturating_sub(1))?;
    let mut candidates = Vec::new();
    for &tok in middle {
        match tok {
            DcShorthand::Long(id) => candidates.push(id),
            t if t == sh_graph_node || t == sh_list_sep => {}
            _ => return None,
        }
    }
    if candidates.len() >= 2 {
        Some(candidates)
    } else {
        None
    }
}

/// Computes the set of effectively inactive node IDs, along with `superseded_by` (`old_id -> new_id`)
/// and `retracted_by` (`old_id -> retract_stmt_id`), across `nodes` in reverse chronological order.
#[must_use]
pub fn compute_effective_inactive_ids(
    nodes: &[LedgerNode],
) -> (HashSet<u128>, HashMap<u128, u128>, HashMap<u128, u128>) {
    let sh_supersedes = sh(DC_SUPERSEDES);
    let sh_retracts = sh(DC_RETRACTS);

    let mut inactive_ids = HashSet::new();
    let mut superseded_by = HashMap::new();
    let mut retracted_by = HashMap::new();

    for node in nodes.iter().rev() {
        if inactive_ids.contains(&node.id) {
            continue;
        }
        if let Some((verb, subj, obj)) = parse_statement_tokens(&node.tokens) {
            if verb == sh_supersedes {
                if let DcShorthand::Long(old_id) = obj {
                    let new_id = match subj {
                        DcShorthand::Long(n_id) => n_id,
                        _ => node.id,
                    };
                    inactive_ids.insert(old_id);
                    superseded_by.insert(old_id, new_id);
                }
            } else if verb == sh_retracts {
                if let DcShorthand::Long(old_id) = obj {
                    inactive_ids.insert(old_id);
                    retracted_by.insert(old_id, node.id);
                }
            }
        }
    }

    (inactive_ids, superseded_by, retracted_by)
}

/// Returns the highest allocated node ID in the System block from the shared runtime seed graph.
pub fn current_system_block_most_recent_node_id() -> Result<u128> {
    let index = SeedGraphIndex::shared()?;
    index
        .system_block_most_recent_node_id()
        .context("System block seed graph has no allocated node IDs")
}

impl SeedGraphIndex {
    /// Returns the lazily loaded process-wide `SeedGraphIndex` from the runtime asset bundle.
    pub fn shared() -> Result<&'static Arc<Self>> {
        let res = SHARED_SEED_GRAPH.get_or_init(|| {
            Self::load_from_asset_bundle()
                .map(Arc::new)
                .map_err(|err| format!("{err:#}"))
        });
        match res {
            Ok(arc) => Ok(arc),
            Err(msg) => bail!("{msg}"),
        }
    }

    /// Loads all active `data/graph/*.ctbnbdl` bundle files (beginning with
    /// `data/graph/85899_34592.ctbnbdl`) from the runtime asset bundle (`ctoolbox.rsrc`)
    /// and indexes all active statements, seed entities, and value nodes.
    pub fn load_from_asset_bundle() -> Result<Self> {
        let (active_most_recent, active_nodes) = Self::load_nodes_from_asset_dir(
            SEED_GRAPH_ASSET_GLOB,
            "data/graph",
            Some(PRIMARY_SEED_GRAPH_ASSET_KEY),
            true,
        )?;
        if let Some(first) = active_nodes.first() {
            ensure!(
                first.id == SYSTEM_REGION_START,
                "Expected first active system graph node to start at l{SYSTEM_REGION_START} ('{}'), found l{}",
                format_chunk_bundle_filename(SYSTEM_REGION_START),
                first.id
            );
        }
        let mut index = Self::from_bundled_nodes(active_nodes);
        if let Some(wm) = active_most_recent {
            index.system_block_most_recent_node_id = Some(
                index
                    .system_block_most_recent_node_id
                    .map_or(wm, |cur| cur.max(wm)),
            );
        }
        Ok(index)
    }

    /// Loads both active `data/graph/*.ctbnbdl` bundles (from `ctoolbox.rsrc`) and optional
    /// historical/inactive `data/graph/system/*.ctbnbdl` bundles (from `system_graph_history.rsrc`
    /// if installed, or `assets/data/graph/system/` in dev mode), merging them in ascending
    /// node ID order and resolving all `Dc 591` (`Supersedes`) and `Dc 592` (`Retracts`) history.
    pub fn load_with_history_from_asset_bundle() -> Result<Self> {
        let (bundle_most_recent, merged) =
            Self::load_merged_nodes_with_history()?;
        let mut index = Self::from_bundled_nodes(merged);
        if let Some(wm) = bundle_most_recent {
            index.system_block_most_recent_node_id = Some(
                index
                    .system_block_most_recent_node_id
                    .map_or(wm, |cur| cur.max(wm)),
            );
        }
        Ok(index)
    }

    /// Loads a `SeedGraphIndex` pinned to the System-block state as of `most_recent_node_id`
    /// (`Dc 641` `"Global graph system block most recent node ID at time of document creation"`).
    ///
    /// If `most_recent_node_id` matches the shared `HEAD` seed graph's
    /// `system_block_most_recent_node_id()`, returns the shared `HEAD` index in $O(1)$.
    /// If `most_recent_node_id` exceeds `HEAD`, returns an error rather than silently
    /// substituting `HEAD`. Otherwise, loads active and historical `.ctbnbdl` bundles,
    /// verifies that historical coverage is complete up to `most_recent_node_id`, and
    /// reconstructs the graph state as of `most_recent_node_id`.
    pub fn load_at_most_recent_from_asset_bundle(
        most_recent_node_id: u128,
    ) -> Result<Arc<Self>> {
        if let Some(block_desc) = ctb_storage_minimal::global_graph_layout::unsupported_graph_block_description(most_recent_node_id) {
            bail!(
                "Document pin Dc 641 targets unsupported graph block ({block_desc}): l{most_recent_node_id} is outside System region ({SYSTEM_REGION_START}..={SYSTEM_REGION_END})"
            );
        }
        ensure!(
            (SYSTEM_REGION_START..=SYSTEM_REGION_END)
                .contains(&most_recent_node_id),
            "Most recent System block node ID l{most_recent_node_id} is outside System region ({SYSTEM_REGION_START}..={SYSTEM_REGION_END})"
        );
        let shared = Self::shared()?;
        if let Some(head_id) = shared.system_block_most_recent_node_id() {
            if most_recent_node_id == head_id {
                return Ok(Arc::clone(shared));
            }
            if most_recent_node_id > head_id {
                bail!(
                    "Document pin Dc 641 requires a newer System graph snapshot than the local graph (l{most_recent_node_id} > l{head_id})"
                );
            }
        }
        let (active_most_recent, active_nodes) = Self::load_nodes_from_asset_dir(
            SEED_GRAPH_ASSET_GLOB,
            "data/graph",
            Some(PRIMARY_SEED_GRAPH_ASSET_KEY),
            true,
        )?;
        let (history_most_recent, history_nodes) =
            Self::load_nodes_from_asset_dir(
                SYSTEM_GRAPH_HISTORY_ASSET_GLOB,
                "data/graph/system",
                None,
                false,
            )?;
        let index = Self::from_active_and_history_nodes_at_most_recent(
            active_most_recent,
            active_nodes,
            history_most_recent,
            history_nodes,
            most_recent_node_id,
        )?;
        Ok(Arc::new(index))
    }

    /// Reconstructs a `SeedGraphIndex` pinned to `most_recent_node_id` from separate
    /// active (`data/graph/*.ctbnbdl`) and historical (`data/graph/system/*.ctbnbdl`)
    /// node lists, enforcing snapshot bounds and historical bundle completeness.
    pub fn from_active_and_history_nodes_at_most_recent(
        active_most_recent: Option<u128>,
        active_nodes: Vec<LedgerNode>,
        history_most_recent: Option<u128>,
        history_nodes: Vec<LedgerNode>,
        most_recent_node_id: u128,
    ) -> Result<Self> {
        if let Some(block_desc) = ctb_storage_minimal::global_graph_layout::unsupported_graph_block_description(most_recent_node_id) {
            bail!(
                "Document pin Dc 641 targets unsupported graph block ({block_desc}): l{most_recent_node_id} is outside System region ({SYSTEM_REGION_START}..={SYSTEM_REGION_END})"
            );
        }
        ensure!(
            (SYSTEM_REGION_START..=SYSTEM_REGION_END)
                .contains(&most_recent_node_id),
            "Most recent System block node ID l{most_recent_node_id} is outside System region ({SYSTEM_REGION_START}..={SYSTEM_REGION_END})"
        );
        let head_id = active_most_recent
            .max(history_most_recent)
            .max(
                active_nodes
                    .iter()
                    .chain(&history_nodes)
                    .map(|node| node.id)
                    .max(),
            )
            .context("Cannot resolve Dc 641 pin against an empty System graph")?;
        if most_recent_node_id > head_id {
            bail!(
                "Document pin Dc 641 requires a newer System graph snapshot than the local graph (l{most_recent_node_id} > l{head_id})"
            );
        }
        let (_, merged) = Self::merge_active_and_history_nodes(
            active_most_recent,
            active_nodes,
            history_most_recent,
            history_nodes,
        )?;
        if most_recent_node_id < head_id {
            let mut expected_id = SYSTEM_REGION_START;
            for node in &merged {
                if node.id > most_recent_node_id {
                    break;
                }
                ensure!(
                    node.id == expected_id,
                    "Historical graph bundle system_graph_history.rsrc (data/graph/system/*.ctbnbdl) is incomplete: missing node l{expected_id} required for historical pin l{most_recent_node_id}"
                );
                expected_id = expected_id.saturating_add(1);
            }
            ensure!(
                expected_id > most_recent_node_id,
                "Historical graph bundle system_graph_history.rsrc (data/graph/system/*.ctbnbdl) is incomplete: missing node l{expected_id}..=l{most_recent_node_id} for historical pin l{most_recent_node_id}"
            );
        }
        let mut index = Self::from_nodes_at_most_recent(
            merged,
            most_recent_node_id,
        );
        index.bundled_origin_node_ids =
            index.nodes.iter().map(|node| node.id).collect();
        Ok(index)
    }

    fn load_merged_nodes_with_history() -> Result<(Option<u128>, Vec<LedgerNode>)>
    {
        let (active_most_recent, active_nodes) = Self::load_nodes_from_asset_dir(
            SEED_GRAPH_ASSET_GLOB,
            "data/graph",
            Some(PRIMARY_SEED_GRAPH_ASSET_KEY),
            true,
        )?;
        let (history_most_recent, history_nodes) =
            Self::load_nodes_from_asset_dir(
                SYSTEM_GRAPH_HISTORY_ASSET_GLOB,
                "data/graph/system",
                None,
                false,
            )?;
        Self::merge_active_and_history_nodes(
            active_most_recent,
            active_nodes,
            history_most_recent,
            history_nodes,
        )
    }

    fn merge_active_and_history_nodes(
        active_most_recent: Option<u128>,
        active_nodes: Vec<LedgerNode>,
        history_most_recent: Option<u128>,
        history_nodes: Vec<LedgerNode>,
    ) -> Result<(Option<u128>, Vec<LedgerNode>)> {
        let bundle_most_recent = active_most_recent.max(history_most_recent);
        if history_nodes.is_empty() {
            return Ok((bundle_most_recent, active_nodes));
        }

        let mut merged = Vec::with_capacity(
            active_nodes.len().saturating_add(history_nodes.len()),
        );
        let mut a_iter = active_nodes.into_iter().peekable();
        let mut h_iter = history_nodes.into_iter().peekable();
        while let (Some(a), Some(h)) = (a_iter.peek(), h_iter.peek()) {
            ensure!(
                a.id != h.id,
                "Duplicate node ID l{} present in both active data/graph/ and historical data/graph/system/ bundles",
                a.id
            );
            if a.id < h.id {
                if let Some(node) = a_iter.next() {
                    merged.push(node);
                }
            } else if let Some(node) = h_iter.next() {
                merged.push(node);
            }
        }
        merged.extend(a_iter);
        merged.extend(h_iter);
        Ok((bundle_most_recent, merged))
    }

    fn load_nodes_from_asset_dir(
        glob_pattern: &str,
        expected_parent_dir: &str,
        fallback_key: Option<&str>,
        require_non_empty: bool,
    ) -> Result<(Option<u128>, Vec<LedgerNode>)> {
        let raw_paths = ctb_storage_asset_bundle::find_assets(glob_pattern)
            .with_context(|| {
                format!(
                    "Failed to query runtime asset bundle for '{glob_pattern}'"
                )
            })?;
        let expected_parent = Path::new(expected_parent_dir);
        let mut asset_paths: Vec<String> = raw_paths
            .into_iter()
            .filter(|p| Path::new(p).parent() == Some(expected_parent))
            .collect();

        if asset_paths.is_empty() {
            if let Some(fallback) = fallback_key {
                if ctb_storage_asset_bundle::get_asset(fallback).is_some() {
                    asset_paths.push(fallback.to_string());
                }
            }
        }
        if asset_paths.is_empty() {
            if require_non_empty {
                bail!(
                    "No system graph .ctbnbdl bundles found in runtime asset bundle matching '{glob_pattern}'"
                );
            }
            return Ok((None, Vec::new()));
        }

        let mut keyed_paths: Vec<(u128, String)> =
            Vec::with_capacity(asset_paths.len());
        for asset_path in asset_paths {
            let filename = Path::new(&asset_path)
                .file_name()
                .and_then(|s| s.to_str())
                .with_context(|| {
                    format!("Asset path '{asset_path}' has no valid filename")
                })?;
            let start_id = parse_chunk_bundle_filename_start_id(filename)
                .with_context(|| {
                    format!(
                        "Invalid .ctbnbdl bundle filename '{filename}' in asset '{asset_path}'"
                    )
                })?;
            ensure!(
                (SYSTEM_REGION_START..=SYSTEM_REGION_END).contains(&start_id),
                "Bundle '{asset_path}' start ID l{start_id} is outside System region ({SYSTEM_REGION_START}..={SYSTEM_REGION_END})"
            );
            keyed_paths.push((start_id, asset_path));
        }
        keyed_paths.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

        let mut all_nodes: Vec<LedgerNode> = Vec::new();
        let mut max_most_recent: Option<u128> = None;
        for (expected_first_id, asset_path) in keyed_paths {
            let bytes = ctb_storage_asset_bundle::get_asset(&asset_path)
                .with_context(|| {
                    format!(
                        "Failed to read asset '{asset_path}' from runtime asset bundle"
                    )
                })?;
            let (bundle_most_recent, bundle_nodes) =
                decode_ctbnbdl_bundle(&bytes).with_context(|| {
                    format!("Failed to decode .ctbnbdl bundle '{asset_path}'")
                })?;
            if let Some(wm) = bundle_most_recent {
                max_most_recent =
                    Some(max_most_recent.map_or(wm, |cur| cur.max(wm)));
            }
            let Some(first_node) = bundle_nodes.first() else {
                bail!(
                    "Decoded .ctbnbdl bundle '{asset_path}' contains 0 nodes"
                );
            };
            ensure!(
                first_node.id == expected_first_id,
                "Bundle '{asset_path}' filename declares start ID l{expected_first_id}, but first decoded node is l{}",
                first_node.id
            );
            for (idx, node) in bundle_nodes.into_iter().enumerate() {
                ensure!(
                    (SYSTEM_REGION_START..=SYSTEM_REGION_END)
                        .contains(&node.id),
                    "{asset_path} (node index {idx}): node ID l{} is outside System region ({SYSTEM_REGION_START}..={SYSTEM_REGION_END})",
                    node.id
                );
                if let Some(prev) = all_nodes.last() {
                    ensure!(
                        node.id > prev.id,
                        "{asset_path} (node index {idx}): expected strictly increasing node ID > l{}, found l{}",
                        prev.id,
                        node.id
                    );
                }
                all_nodes.push(node);
            }
        }

        Ok((max_most_recent, all_nodes))
    }

    /// Decodes a single `.ctbnbdl` byte slice and builds a `SeedGraphIndex`.
    pub fn from_ctbnbdl_bytes(bytes: &[u8]) -> Result<Self> {
        let (most_recent, nodes) = decode_ctbnbdl_bundle(bytes)?;
        let mut index = Self::from_nodes(nodes);
        if let Some(wm) = most_recent {
            index.system_block_most_recent_node_id = Some(
                index
                    .system_block_most_recent_node_id
                    .map_or(wm, |cur| cur.max(wm)),
            );
        }
        Ok(index)
    }

    /// Builds a `SeedGraphIndex` from an ordered list of `LedgerNode`s using the default
    /// single-valued property set.
    #[must_use]
    pub fn from_nodes(nodes: Vec<LedgerNode>) -> Self {
        let single_props = single_valued_property_set();
        Self::from_nodes_with_single_props(nodes, &single_props)
    }

    /// Builds a `SeedGraphIndex` from an ordered list of host-authenticated bundled `LedgerNode`s
    /// (loaded from `ctoolbox.rsrc` / `system_graph_history.rsrc`), marking all supplied System-region
    /// nodes as having authenticated bundled origin.
    #[must_use]
    pub fn from_bundled_nodes(nodes: Vec<LedgerNode>) -> Self {
        let bundled_ids: HashSet<u128> = nodes
            .iter()
            .filter(|n| (SYSTEM_REGION_START..=SYSTEM_REGION_END).contains(&n.id))
            .map(|n| n.id)
            .collect();
        let mut index = Self::from_nodes(nodes);
        index.bundled_origin_node_ids = bundled_ids;
        index
    }

    /// Builds a `SeedGraphIndex` combining host-authenticated bundled System nodes (`bundled_nodes`)
    /// with extra/unbundled nodes (`extra_nodes`, such as Wikidata or external resource bundles).
    ///
    /// Only nodes originating from `bundled_nodes` are recorded in `bundled_origin_node_ids`;
    /// `extra_nodes` cannot supply bootstrap authority even if their numeric node IDs or
    /// self-asserted `Dc 635` provenance claims mimic bundled System nodes.
    pub fn from_bundled_and_extra_nodes(
        bundled_nodes: Vec<LedgerNode>,
        extra_nodes: Vec<LedgerNode>,
    ) -> Result<Self> {
        let mut bundled_ids = HashSet::with_capacity(bundled_nodes.len());
        let bundled_most_recent = bundled_nodes.iter().map(|n| n.id).max();
        for node in &bundled_nodes {
            ensure!(
                (SYSTEM_REGION_START..=SYSTEM_REGION_END).contains(&node.id),
                "Bundled System node l{} is outside System region ({SYSTEM_REGION_START}..={SYSTEM_REGION_END})",
                node.id
            );
            bundled_ids.insert(node.id);
        }
        for extra in &extra_nodes {
            ensure!(
                !bundled_ids.contains(&extra.id),
                "Extra unbundled node ID l{} collides with an existing bundled System node ID",
                extra.id
            );
        }
        let mut combined = Vec::with_capacity(
            bundled_nodes.len().saturating_add(extra_nodes.len()),
        );
        combined.extend(bundled_nodes);
        combined.extend(extra_nodes);
        combined.sort_by_key(|n| n.id);
        let mut index = Self::from_nodes(combined);
        index.bundled_origin_node_ids = bundled_ids;
        index.system_block_most_recent_node_id = bundled_most_recent;
        Ok(index)
    }

    /// Builds a `SeedGraphIndex` reflecting the System-block graph state as of `most_recent_node_id`
    /// (`Dc 641` `"Global graph system block most recent node ID at time of document creation"`),
    /// ignoring any statements, supersessions (`Dc 591`), or retractions (`Dc 592`) allocated
    /// with `node.id > most_recent_node_id`.
    #[must_use]
    pub fn from_nodes_at_most_recent(
        nodes: Vec<LedgerNode>,
        most_recent_node_id: u128,
    ) -> Self {
        let filtered: Vec<LedgerNode> = nodes
            .into_iter()
            .filter(|node| node.id <= most_recent_node_id)
            .collect();
        let mut index = Self::from_nodes(filtered);
        index.system_block_most_recent_node_id = Some(most_recent_node_id);
        index
    }

    /// Builds a `SeedGraphIndex` from `nodes` and a custom set of single-valued property Dcs.
    #[must_use]
    pub fn from_nodes_with_single_props(
        nodes: Vec<LedgerNode>,
        single_valued_props: &HashSet<DcShorthand>,
    ) -> Self {
        let sh_supersedes = sh(DC_SUPERSEDES);
        let sh_retracts = sh(DC_RETRACTS);
        let sh_has_value = sh(DC_HAS_VALUE);
        let sh_has_a = sh(DC_HAS_A);
        let sh_preferred = sh(DC_PREFERRED);
        let sh_replacement = sh(DC_REPLACEMENT);
        let sh_is_instance_of = sh(DC_IS_INSTANCE_OF);
        let sh_enclosing_doc = sh(DC_ENCLOSING_DOCUMENT);
        let sh_prog_ident = sh(DC_PROGRAMMATIC_IDENTIFIER);
        let sh_canon_name = sh(DC_CANONICAL_NAME);
        let sh_bidi_class = sh(DC_BIDIRECTIONAL_CLASS);
        let sh_char_gen_cat = sh(DC_CHARACTER_GENERAL_CATEGORY);
        let sh_dc_gen_cat = sh(DC_DC_GENERAL_CATEGORY);
        let sh_char_script = sh(DC_CHARACTER_SCRIPT);
        let sh_uni_script = sh(DC_UNICODE_SCRIPT);
        let sh_entity_type = sh(DC_ENTITY_TYPE);
        let sh_data_source = sh(DC_COLLECTIVE_TOOLBOX_SEED_GRAPH_DATA_SOURCE);
        let sh_this_element = sh(DC_THIS_ELEMENT);

        let mut node_index_by_id = HashMap::with_capacity(nodes.len());
        for (idx, node) in nodes.iter().enumerate() {
            node_index_by_id.insert(node.id, idx);
        }

        let (mut inactive_ids, superseded_by, retracted_by) =
            compute_effective_inactive_ids(&nodes);

        let mut superseding_stmt_by_old_id: HashMap<u128, u128> =
            HashMap::new();
        let mut subject_all_stmt_ids: HashMap<DcShorthand, Vec<u128>> =
            HashMap::new();
        let mut has_val_stmt_ids_by_has_a: HashMap<
            u128,
            Vec<(u128, DcShorthand)>,
        > = HashMap::new();
        let mut node_provenance_assertions: HashMap<
            u128,
            Vec<(u128, u128, DcShorthand)>,
        > = HashMap::new();

        for node in &nodes {
            if let Some((verb, subj, obj)) =
                parse_statement_tokens(&node.tokens)
            {
                if verb == sh_supersedes {
                    if let DcShorthand::Long(old_id) = obj
                        && !inactive_ids.contains(&node.id)
                    {
                        superseding_stmt_by_old_id.insert(old_id, node.id);
                    }
                } else if verb == sh_has_value {
                    if let DcShorthand::Long(has_a_id) = subj {
                        has_val_stmt_ids_by_has_a
                            .entry(has_a_id)
                            .or_default()
                            .push((node.id, obj));
                    }
                } else if verb != sh_retracts {
                    let eff_subj = if subj == sh_enclosing_doc {
                        DcShorthand::Long(node.id)
                    } else {
                        subj
                    };
                    subject_all_stmt_ids
                        .entry(eff_subj)
                        .or_default()
                        .push(node.id);
                }
            } else if let Some((
                inner_verb,
                inner_subj,
                _inner_obj,
                _list_type,
                targets,
            )) = parse_for_each_statement_tokens(&node.tokens)
                && inner_verb != sh_has_value
                && inner_verb != sh_supersedes
                && inner_verb != sh_retracts
            {
                for &elem in &targets {
                    let eff_subj = if inner_subj == sh_this_element {
                        elem
                    } else if inner_subj == sh_enclosing_doc {
                        DcShorthand::Long(node.id)
                    } else {
                        inner_subj
                    };
                    subject_all_stmt_ids
                        .entry(eff_subj)
                        .or_default()
                        .push(node.id);
                }
            }
        }

        // First pass: index active value nodes and find active `610` (`Has value`) and `584 <s1> 570` (`Preferred`) nodes.
        let mut value_node_by_tokens = HashMap::new();
        let mut string_by_node_id: HashMap<u128, String> = HashMap::new();
        let mut integer_by_node_id: HashMap<u128, u128> = HashMap::new();
        let mut active_has_val_by_stmt: HashMap<
            u128,
            Vec<(u128, DcShorthand)>,
        > = HashMap::new();
        let mut active_pref_qual_by_stmt: HashMap<u128, Vec<u128>> =
            HashMap::new();
        let mut orphaned_stmt_ids: Vec<u128> = Vec::new();
        let mut node_data_source: HashMap<u128, DcShorthand> = HashMap::new();
        let mut provenance_stmt_ids: HashSet<u128> = HashSet::new();

        for node in &nodes {
            if let Some(decoded_str) = decode_string_value_tokens(&node.tokens)
            {
                string_by_node_id.insert(node.id, decoded_str);
            } else if let Some(decoded_int) =
                decode_integer_value_tokens(&node.tokens)
            {
                integer_by_node_id.insert(node.id, decoded_int);
            }
            if inactive_ids.contains(&node.id) {
                continue;
            }
            let stmt_opt = parse_statement_tokens(&node.tokens);
            let for_each_opt = if stmt_opt.is_none() {
                parse_for_each_statement_tokens(&node.tokens)
            } else {
                None
            };
            if stmt_opt.is_none()
                && for_each_opt.is_none()
                && parse_ambiguous_values_tokens(&node.tokens).is_none()
            {
                value_node_by_tokens
                    .entry(node.tokens.clone())
                    .or_insert(node.id);
            }
            if let Some((verb, subj, obj)) = stmt_opt {
                if verb == sh_has_value {
                    if let DcShorthand::Long(has_a_id) = subj {
                        if inactive_ids.contains(&has_a_id) {
                            inactive_ids.insert(node.id);
                        } else {
                            let is_prop_has_a = node_index_by_id
                                .get(&has_a_id)
                                .and_then(|&idx| nodes.get(idx))
                                .is_some_and(|n| {
                                    parse_statement_tokens(&n.tokens)
                                        .is_some_and(|(v, _, _)| v == sh_has_a)
                                        || parse_for_each_statement_tokens(
                                            &n.tokens,
                                        )
                                        .is_some_and(|(v, _, _, _, _)| {
                                            v == sh_has_a
                                        })
                                });
                            if is_prop_has_a {
                                active_has_val_by_stmt
                                    .entry(has_a_id)
                                    .or_default()
                                    .push((node.id, obj));
                            } else {
                                orphaned_stmt_ids.push(node.id);
                            }
                        }
                    } else {
                        orphaned_stmt_ids.push(node.id);
                    }
                } else if verb == sh_has_a && obj == sh_preferred {
                    if let DcShorthand::Long(has_a_id) = subj {
                        let targets_has_a = node_index_by_id
                            .get(&has_a_id)
                            .and_then(|&idx| nodes.get(idx))
                            .is_some_and(|n| {
                                parse_statement_tokens(&n.tokens)
                                    .is_some_and(|(v, _, _)| v == sh_has_a)
                                    || parse_for_each_statement_tokens(
                                        &n.tokens,
                                    )
                                    .is_some_and(|(v, _, _, _, _)| {
                                        v == sh_has_a
                                    })
                            });
                        if targets_has_a {
                            if inactive_ids.contains(&has_a_id) {
                                inactive_ids.insert(node.id);
                            } else {
                                active_pref_qual_by_stmt
                                    .entry(has_a_id)
                                    .or_default()
                                    .push(node.id);
                            }
                        }
                    }
                }
            }
        }

        // Second pass: reconstruct seed entities and active statements.
        let mut seed_candidate_class: HashMap<u128, DcShorthand> =
            HashMap::new();
        let mut seed_entity_ids = HashMap::new();
        let mut seed_entity_by_id = HashMap::new();
        let mut active_single_props = BTreeMap::new();
        let mut single_props_by_subj_prop: BTreeMap<
            (DcShorthand, DcShorthand),
            Vec<(u128, u128, DcShorthand)>,
        > = BTreeMap::new();
        let mut active_multi_props = BTreeMap::new();
        let mut active_multi_preferred = BTreeMap::new();
        let mut multi_props_by_subj_prop: HashMap<
            (DcShorthand, DcShorthand),
            Vec<ActivePropertyValue>,
        > = HashMap::new();
        let mut active_preferred = BTreeMap::new();
        let mut preferred_all_by_subj_val: BTreeMap<
            (DcShorthand, DcShorthand),
            Vec<(u128, u128, u128)>,
        > = BTreeMap::new();
        let mut preferred_by_subject: HashMap<DcShorthand, Vec<DcShorthand>> =
            HashMap::new();
        let mut active_direct_stmts = BTreeMap::new();
        let mut direct_stmts_all: BTreeMap<
            (DcShorthand, DcShorthand, DcShorthand),
            Vec<u128>,
        > = BTreeMap::new();
        let mut direct_objects_by_verb_subj: HashMap<
            (DcShorthand, DcShorthand),
            Vec<DcShorthand>,
        > = HashMap::new();
        let mut direct_subjects_by_verb_obj: HashMap<
            (DcShorthand, DcShorthand),
            Vec<DcShorthand>,
        > = HashMap::new();
        let mut consumed_pref_qual_has_a_ids: HashSet<u128> = HashSet::new();

        for node in &nodes {
            if inactive_ids.contains(&node.id) {
                continue;
            }
            if let Some((
                inner_verb,
                inner_subj,
                inner_obj,
                _list_type,
                targets,
            )) = parse_for_each_statement_tokens(&node.tokens)
            {
                let resolve_iter_tok =
                    |pat: DcShorthand, elem: DcShorthand| -> DcShorthand {
                        if pat == sh_this_element {
                            elem
                        } else if pat == sh_enclosing_doc {
                            DcShorthand::Long(node.id)
                        } else {
                            pat
                        }
                    };
                if inner_verb == sh_has_a {
                    if let Some(has_vals) = active_has_val_by_stmt.get(&node.id)
                        && !has_vals.is_empty()
                    {
                        if inner_obj == sh_data_source {
                            provenance_stmt_ids.insert(node.id);
                            for &(has_val_id, val_tok) in has_vals {
                                provenance_stmt_ids.insert(has_val_id);
                                for &elem in &targets {
                                    let subj =
                                        resolve_iter_tok(inner_subj, elem);
                                    if let DcShorthand::Long(target_node_id) =
                                        subj
                                    {
                                        node_provenance_assertions
                                            .entry(target_node_id)
                                            .or_default()
                                            .push((
                                                node.id,
                                                has_val_id,
                                                val_tok,
                                            ));
                                        node_data_source
                                            .insert(target_node_id, val_tok);
                                    }
                                }
                            }
                        } else {
                            let pref_ids =
                                active_pref_qual_by_stmt.get(&node.id);
                            let preferred_id =
                                pref_ids.and_then(|ids| ids.last().copied());
                            if preferred_id.is_some() {
                                consumed_pref_qual_has_a_ids.insert(node.id);
                            }
                            for &(has_val_id, val_tok) in has_vals {
                                for &elem in &targets {
                                    let subj =
                                        resolve_iter_tok(inner_subj, elem);
                                    let obj = resolve_iter_tok(inner_obj, elem);
                                    if single_valued_props.contains(&obj) {
                                        single_props_by_subj_prop
                                            .entry((subj, obj))
                                            .or_default()
                                            .push((
                                                node.id, has_val_id, val_tok,
                                            ));
                                        active_single_props.insert(
                                            (subj, obj),
                                            (node.id, has_val_id, val_tok),
                                        );
                                    } else {
                                        if let Some(pref_id) = preferred_id {
                                            active_multi_preferred.insert(
                                                (subj, obj, val_tok),
                                                pref_id,
                                            );
                                        }
                                        active_multi_props.insert(
                                            (subj, obj, val_tok),
                                            (node.id, has_val_id),
                                        );
                                        multi_props_by_subj_prop
                                            .entry((subj, obj))
                                            .or_default()
                                            .push(ActivePropertyValue {
                                                has_a_id: node.id,
                                                has_val_id,
                                                value: val_tok,
                                                preferred_id,
                                            });
                                    }
                                }
                            }
                        }
                    } else {
                        orphaned_stmt_ids.push(node.id);
                    }
                } else if inner_verb != sh_has_value
                    && inner_verb != sh_supersedes
                    && inner_verb != sh_retracts
                {
                    for &elem in &targets {
                        let subj = resolve_iter_tok(inner_subj, elem);
                        let obj = resolve_iter_tok(inner_obj, elem);
                        direct_stmts_all
                            .entry((inner_verb, subj, obj))
                            .or_default()
                            .push(node.id);
                        if active_direct_stmts
                            .insert((inner_verb, subj, obj), node.id)
                            .is_none()
                        {
                            direct_objects_by_verb_subj
                                .entry((inner_verb, subj))
                                .or_default()
                                .push(obj);
                            direct_subjects_by_verb_obj
                                .entry((inner_verb, obj))
                                .or_default()
                                .push(subj);
                        }
                    }
                }
                continue;
            }

            let Some((verb, subj, obj)) = parse_statement_tokens(&node.tokens)
            else {
                continue;
            };

            // Seed entity declaration: `586 615 <class_dc>` where `<class_dc>` is a Short Dc.
            if verb == sh_is_instance_of
                && subj == sh_enclosing_doc
                && matches!(obj, DcShorthand::Short(_))
            {
                seed_candidate_class.insert(node.id, obj);
                continue;
            }

            if verb == sh_has_value
                || verb == sh_supersedes
                || verb == sh_retracts
            {
                continue;
            }
            if verb == sh_has_a && obj == sh_preferred {
                if let DcShorthand::Long(target_id) = subj {
                    if node_index_by_id
                        .get(&target_id)
                        .and_then(|&idx| nodes.get(idx))
                        .is_some_and(|n| {
                            parse_statement_tokens(&n.tokens)
                                .is_some_and(|(v, _, _)| v == sh_has_a)
                        })
                    {
                        continue;
                    }
                }
            }

            if verb == sh_has_a {
                if let Some(has_vals) = active_has_val_by_stmt.get(&node.id)
                    && !has_vals.is_empty()
                {
                    if obj == sh_data_source {
                        if let DcShorthand::Long(target_node_id) = subj {
                            provenance_stmt_ids.insert(node.id);
                            for &(has_val_id, val_tok) in has_vals {
                                provenance_stmt_ids.insert(has_val_id);
                                node_provenance_assertions
                                    .entry(target_node_id)
                                    .or_default()
                                    .push((node.id, has_val_id, val_tok));
                                node_data_source
                                    .insert(target_node_id, val_tok);
                            }
                            continue;
                        }
                    }
                    let pref_ids = active_pref_qual_by_stmt.get(&node.id);
                    let first_pref_id =
                        pref_ids.and_then(|ids| ids.last().copied());
                    if let Some(ids) = pref_ids {
                        for &extra_pref_id in ids
                            .get(..ids.len().saturating_sub(1))
                            .into_iter()
                            .flatten()
                        {
                            orphaned_stmt_ids.push(extra_pref_id);
                        }
                    }
                    for &(has_val_id, val_tok) in has_vals {
                        // Check if this names a seed entity (`584 l<seed_id> 572/574` -> `610 ... l<str_id>`).
                        if let (
                            DcShorthand::Long(seed_id),
                            DcShorthand::Long(str_id),
                        ) = (subj, val_tok)
                        {
                            if let (Some(&class_sh), Some(label)) = (
                                seed_candidate_class.get(&seed_id),
                                string_by_node_id.get(&str_id),
                            ) {
                                let key_opt = if class_sh == sh_bidi_class
                                    && obj == sh_prog_ident
                                {
                                    Some(SeedEntityKey::BidiClass(
                                        label.clone(),
                                    ))
                                } else if (class_sh == sh_char_gen_cat
                                    || class_sh == sh_dc_gen_cat)
                                    && obj == sh_prog_ident
                                {
                                    Some(SeedEntityKey::GeneralCategory(
                                        label.clone(),
                                    ))
                                } else if (class_sh == sh_char_script
                                    || class_sh == sh_uni_script)
                                    && obj == sh_canon_name
                                {
                                    Some(SeedEntityKey::Script(label.clone()))
                                } else if class_sh == sh_entity_type
                                    && obj == sh_canon_name
                                {
                                    Some(SeedEntityKey::NamedConstruct(
                                        label.clone(),
                                    ))
                                } else {
                                    None
                                };
                                if let Some(seed_key) = key_opt {
                                    if let Some(&existing_seed_id) =
                                        seed_entity_ids.get(&seed_key)
                                    {
                                        if existing_seed_id != seed_id {
                                            orphaned_stmt_ids.push(has_val_id);
                                            orphaned_stmt_ids.push(node.id);
                                            orphaned_stmt_ids.push(seed_id);
                                            continue;
                                        }
                                    } else {
                                        seed_entity_by_id
                                            .insert(seed_id, seed_key.clone());
                                        seed_entity_ids
                                            .insert(seed_key, seed_id);
                                    }
                                }
                            }
                        }

                        if obj == sh_replacement {
                            if let Some(pref_id) = first_pref_id {
                                consumed_pref_qual_has_a_ids.insert(node.id);
                                preferred_all_by_subj_val
                                    .entry((subj, val_tok))
                                    .or_default()
                                    .push((node.id, has_val_id, pref_id));
                                if let std::collections::btree_map::Entry::Vacant(e) = active_preferred.entry((subj, val_tok)) {
                                    e.insert((node.id, has_val_id, pref_id));
                                    preferred_by_subject
                                        .entry(subj)
                                        .or_default()
                                        .push(val_tok);
                                }
                                continue;
                            }
                        }

                        if single_valued_props.contains(&obj) {
                            single_props_by_subj_prop
                                .entry((subj, obj))
                                .or_default()
                                .push((node.id, has_val_id, val_tok));
                            active_single_props.insert(
                                (subj, obj),
                                (node.id, has_val_id, val_tok),
                            );
                        } else {
                            let preferred_id = first_pref_id;
                            if let Some(pref_id) = preferred_id {
                                consumed_pref_qual_has_a_ids.insert(node.id);
                                active_multi_preferred
                                    .insert((subj, obj, val_tok), pref_id);
                            }
                            active_multi_props.insert(
                                (subj, obj, val_tok),
                                (node.id, has_val_id),
                            );
                            multi_props_by_subj_prop
                                .entry((subj, obj))
                                .or_default()
                                .push(ActivePropertyValue {
                                    has_a_id: node.id,
                                    has_val_id,
                                    value: val_tok,
                                    preferred_id,
                                });
                        }
                    }
                } else {
                    orphaned_stmt_ids.push(node.id);
                }
            } else {
                direct_stmts_all
                    .entry((verb, subj, obj))
                    .or_default()
                    .push(node.id);
                if active_direct_stmts
                    .insert((verb, subj, obj), node.id)
                    .is_none()
                {
                    direct_objects_by_verb_subj
                        .entry((verb, subj))
                        .or_default()
                        .push(obj);
                    direct_subjects_by_verb_obj
                        .entry((verb, obj))
                        .or_default()
                        .push(subj);
                }
            }
        }

        for (&has_a_id, pref_ids) in &active_pref_qual_by_stmt {
            if !consumed_pref_qual_has_a_ids.contains(&has_a_id) {
                orphaned_stmt_ids.extend(pref_ids.iter().copied());
            }
        }
        for &seed_id in seed_candidate_class.keys() {
            if !seed_entity_by_id.contains_key(&seed_id) {
                orphaned_stmt_ids.push(seed_id);
            }
        }
        orphaned_stmt_ids.sort_unstable();
        orphaned_stmt_ids.dedup();

        for list in multi_props_by_subj_prop.values_mut() {
            if list.iter().any(|e| e.preferred_id.is_some()) {
                list.sort_by_key(|e| e.preferred_id.is_none());
            }
        }

        let system_block_most_recent_node_id = nodes.iter().map(|n| n.id).max();

        Self {
            nodes,
            node_index_by_id,
            superseded_by,
            retracted_by,
            inactive_ids,
            string_by_node_id,
            integer_by_node_id,
            value_node_by_tokens,
            seed_entity_ids,
            seed_entity_by_id,
            active_single_props,
            single_props_by_subj_prop,
            active_multi_props,
            active_multi_preferred,
            multi_props_by_subj_prop,
            active_preferred,
            preferred_all_by_subj_val,
            preferred_by_subject,
            active_direct_stmts,
            direct_stmts_all,
            direct_objects_by_verb_subj,
            direct_subjects_by_verb_obj,
            orphaned_stmt_ids,
            node_data_source,
            provenance_stmt_ids,
            bundled_origin_node_ids: HashSet::new(),
            superseding_stmt_by_old_id,
            node_provenance_assertions,
            subject_all_stmt_ids,
            has_val_stmt_ids_by_has_a,
            system_block_most_recent_node_id,
        }
    }

    /// Returns the highest allocated node ID in the System block at the time the graph bundle
    /// was created (`Dc 641` `"Global graph system block most recent node ID at time of document creation"`).
    #[must_use]
    pub fn system_block_most_recent_node_id(&self) -> Option<u128> {
        self.system_block_most_recent_node_id
    }

    /// Returns the declared `Dc 635` (`"Collective Toolbox seed graph data source"`) token
    /// for `node_id` in the seed graph, if present.
    #[must_use]
    pub fn node_data_source(&self, node_id: u128) -> Option<DcShorthand> {
        self.node_data_source.get(&node_id).copied()
    }

    /// Returns `true` if the seed graph contains any `Dc 635` (`"Collective Toolbox seed graph data source"`)
    /// provenance declarations.
    #[must_use]
    pub fn has_in_graph_provenance(&self) -> bool {
        !self.node_data_source.is_empty()
    }

    /// Returns the total number of nodes in the loaded seed graph.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns a slice of all loaded `LedgerNode`s.
    #[must_use]
    pub fn nodes(&self) -> &[LedgerNode] {
        &self.nodes
    }

    /// Looks up a `LedgerNode` by its global node ID (`u128`).
    #[must_use]
    pub fn get_node(&self, id: u128) -> Option<&LedgerNode> {
        let &idx = self.node_index_by_id.get(&id)?;
        self.nodes.get(idx)
    }

    /// Returns `true` if `id` exists in the seed graph and has not been superseded or retracted.
    #[must_use]
    pub fn is_node_active(&self, id: u128) -> bool {
        self.node_index_by_id.contains_key(&id)
            && !self.inactive_ids.contains(&id)
    }

    /// Returns the total number of inactive (superseded or retracted) nodes in the seed graph.
    #[must_use]
    pub fn inactive_count(&self) -> usize {
        self.inactive_ids.len()
    }

    /// Returns the number of superseded nodes (`Dc 591`) in the seed graph.
    #[must_use]
    pub fn superseded_count(&self) -> usize {
        self.superseded_by.len()
    }

    /// Returns the number of retracted nodes (`Dc 592`) in the seed graph.
    #[must_use]
    pub fn retracted_count(&self) -> usize {
        self.retracted_by.len()
    }

    /// Follows any `Dc 591 "Supersedes"` chain from `id` to its terminal replacement node,
    /// and if that node (or `id` itself) is a `Dc 348 "Ambiguous values"` list (`348 310 ... 311`),
    /// returns the ordered candidate node IDs.
    #[must_use]
    pub fn get_ambiguous_candidates(&self, id: u128) -> Option<Vec<u128>> {
        let mut cur = id;
        let mut visited = HashSet::new();
        while let Some(&next) = self.superseded_by.get(&cur) {
            if !visited.insert(cur) {
                break;
            }
            cur = next;
        }
        let target_node = self.get_node(cur)?;
        parse_ambiguous_values_tokens(&target_node.tokens)
    }

    /// Returns the node ID that directly superseded `id` via `Dc 591` (`Supersedes`), if any.
    #[must_use]
    pub fn superseded_by(&self, id: u128) -> Option<u128> {
        self.superseded_by.get(&id).copied()
    }

    /// Returns the statement node ID that retracted `id` via `Dc 592` (`Retracts`), if any.
    #[must_use]
    pub fn retracted_by(&self, id: u128) -> Option<u128> {
        self.retracted_by.get(&id).copied()
    }

    /// Follows any `Dc 591` (`Supersedes`) chain starting from `id` to its latest active node ID,
    /// returning `None` if `id` is unknown, if the terminal revision was retracted (`Dc 592`),
    /// or if a supersession cycle is encountered.
    #[must_use]
    pub fn resolve_latest_node_id(&self, id: u128) -> Option<u128> {
        if !self.node_index_by_id.contains_key(&id) {
            return None;
        }
        let mut current = id;
        let mut steps = 0usize;
        while let Some(&next) = self.superseded_by.get(&current) {
            current = next;
            steps = steps.saturating_add(1);
            if steps > self.nodes.len() {
                return None;
            }
        }
        if self.inactive_ids.contains(&current) {
            None
        } else {
            Some(current)
        }
    }

    /// Resolves `id` through any active `Dc 591` (`Supersedes`) chain within this snapshot
    /// and returns the active terminal `LedgerNode`, or `None` if `id` is unknown, retracted
    /// (`Dc 592`), or outside the snapshot.
    #[must_use]
    pub fn get_active_node(&self, id: u128) -> Option<&LedgerNode> {
        let resolved_id = self.resolve_latest_node_id(id)?;
        self.get_node(resolved_id)
    }

    /// Resolves `tok` as a string value node (`DcShorthand::Long(id)`), returning its decoded UTF-8 string.
    #[must_use]
    pub fn get_string(&self, tok: DcShorthand) -> Option<&str> {
        let DcShorthand::Long(id) = tok else {
            return None;
        };
        let resolved_id = self.resolve_latest_node_id(id)?;
        self.string_by_node_id.get(&resolved_id).map(String::as_str)
    }

    /// Resolves `tok` as an integer value node (`DcShorthand::Long(id)`), returning its decoded `u128`.
    #[must_use]
    pub fn get_integer(&self, tok: DcShorthand) -> Option<u128> {
        let DcShorthand::Long(id) = tok else {
            return None;
        };
        let resolved_id = self.resolve_latest_node_id(id)?;
        self.integer_by_node_id.get(&resolved_id).copied()
    }

    /// Looks up the allocated `u128` node ID for a `SeedEntityKey`.
    #[must_use]
    pub fn get_seed_entity_id(&self, key: &SeedEntityKey) -> Option<u128> {
        self.seed_entity_ids.get(key).copied()
    }

    /// Looks up the `SeedEntityKey` for a seed entity node ID (`u128`),
    /// resolving through any active `Dc 591` (`Supersedes`) chain in the snapshot.
    #[must_use]
    pub fn get_seed_entity(&self, id: u128) -> Option<&SeedEntityKey> {
        let resolved_id = self.resolve_latest_node_id(id)?;
        self.seed_entity_by_id.get(&resolved_id)
    }

    /// Returns the active value token for a single-valued property `(subj, prop)`.
    #[must_use]
    pub fn get_single_property(
        &self,
        subj: DcShorthand,
        prop: DcShorthand,
    ) -> Option<DcShorthand> {
        self.active_single_props
            .get(&(subj, prop))
            .map(|&(_, _, val)| val)
    }

    /// Returns the full `ActivePropertyValue` (`has_a_id`, `has_val_id`, `value`) for a
    /// single-valued property `(subj, prop)`.
    #[must_use]
    pub fn get_single_property_entry(
        &self,
        subj: DcShorthand,
        prop: DcShorthand,
    ) -> Option<ActivePropertyValue> {
        self.active_single_props.get(&(subj, prop)).map(
            |&(has_a_id, has_val_id, value)| ActivePropertyValue {
                has_a_id,
                has_val_id,
                value,
                preferred_id: None,
            },
        )
    }

    /// Returns the decoded UTF-8 string of a single-valued string property `(subj, prop)`.
    #[must_use]
    pub fn get_single_property_string(
        &self,
        subj: DcShorthand,
        prop: DcShorthand,
    ) -> Option<&str> {
        let val_tok = self.get_single_property(subj, prop)?;
        self.get_string(val_tok)
    }

    /// Returns the decoded `u128` integer of a single-valued integer property `(subj, prop)`.
    #[must_use]
    pub fn get_single_property_integer(
        &self,
        subj: DcShorthand,
        prop: DcShorthand,
    ) -> Option<u128> {
        let val_tok = self.get_single_property(subj, prop)?;
        self.get_integer(val_tok)
    }

    /// Returns all active `ActivePropertyValue` entries for a multi-valued property `(subj, prop)`.
    #[must_use]
    pub fn get_multi_properties(
        &self,
        subj: DcShorthand,
        prop: DcShorthand,
    ) -> &[ActivePropertyValue] {
        // Reason for fallback: subjects without any active multi-valued property values return an empty slice.
        self.multi_props_by_subj_prop
            .get(&(subj, prop))
            .map_or(&[], Vec::as_slice)
    }

    /// Returns all active preferred replacement target tokens (`<prefer>`) for `subj`.
    #[must_use]
    pub fn get_preferred_replacements(
        &self,
        subj: DcShorthand,
    ) -> &[DcShorthand] {
        // Reason for fallback: subjects without preferred replacements return an empty slice.
        self.preferred_by_subject
            .get(&subj)
            .map_or(&[], Vec::as_slice)
    }

    /// Returns `true` if the direct binary statement `(verb, subj, obj)` is currently active.
    #[must_use]
    pub fn has_direct_statement(
        &self,
        verb: DcShorthand,
        subj: DcShorthand,
        obj: DcShorthand,
    ) -> bool {
        self.active_direct_stmts.contains_key(&(verb, subj, obj))
    }

    /// Returns `true` if `subj`'s active `Dc 574 "Programmatic identifier"` (`Has a` statement node)
    /// has an active `Dc 645 "Is stable"` statement (`645 l<ident_has_a_id> 517`) that is
    /// `Dc 646 "In context of"` `f649` (`Collective Toolbox`).
    #[must_use]
    pub fn has_stable_programmatic_identifier(&self, subj: DcShorthand) -> bool {
        let Some(&(ident_has_a_id, _, _)) = self
            .active_single_props
            .get(&(subj, sh(DC_PROGRAMMATIC_IDENTIFIER)))
        else {
            return false;
        };
        let Some(&stable_stmt_id) = self.active_direct_stmts.get(&(
            sh(DC_IS_STABLE),
            DcShorthand::Long(ident_has_a_id),
            sh(DC_BOOLEAN_TRUE),
        )) else {
            return false;
        };
        self.active_direct_stmts.contains_key(&(
            sh(DC_IN_CONTEXT_OF),
            DcShorthand::Long(stable_stmt_id),
            sh(DC_COLLECTIVE_TOOLBOX),
        ))
    }

    pub(crate) fn format_stable_or_shorthand_token(
        &self,
        tok: DcShorthand,
    ) -> String {
        if self.has_stable_programmatic_identifier(tok) {
            if let Some(ident) = self
                .get_single_property_string(tok, sh(DC_PROGRAMMATIC_IDENTIFIER))
            {
                return ident.to_string();
            }
        }
        // Fallback for historical graph snapshots prior to Dc 645 (Is stable) or synthetic unit tests
        if !self
            .active_direct_stmts
            .keys()
            .any(|&(v, _, _)| v == sh(DC_IS_STABLE))
        {
            match tok {
                DcShorthand::Format(199) => return "Integer".to_string(),
                DcShorthand::Format(200) => return "Natural0".to_string(),
                DcShorthand::Format(351) => return "Natural1".to_string(),
                _ => {}
            }
        }
        tok.to_string()
    }

    /// Returns all active object tokens `obj` for direct binary statements `(verb, subj, obj)`.
    #[must_use]
    pub fn get_direct_objects(
        &self,
        verb: DcShorthand,
        subj: DcShorthand,
    ) -> &[DcShorthand] {
        // Reason for fallback: (verb, subj) pairs without active statements return an empty slice.
        self.direct_objects_by_verb_subj
            .get(&(verb, subj))
            .map_or(&[], Vec::as_slice)
    }

    /// Returns all active subject tokens `subj` for direct binary statements `(verb, subj, obj)`.
    #[must_use]
    pub fn get_direct_subjects(
        &self,
        verb: DcShorthand,
        obj: DcShorthand,
    ) -> &[DcShorthand] {
        // Reason for fallback: (verb, obj) pairs without active statements return an empty slice.
        self.direct_subjects_by_verb_obj
            .get(&(verb, obj))
            .map_or(&[], Vec::as_slice)
    }

    /// Returns a map of all indexed `SeedEntityKey` entries to their allocated `u128` node IDs.
    #[must_use]
    pub fn seed_entities(&self) -> &HashMap<SeedEntityKey, u128> {
        &self.seed_entity_ids
    }

    /// Returns the map of all active single-valued properties `(subject, prop_token) -> (has_a_id, has_val_id, val_token)`.
    #[must_use]
    pub fn active_single_props(
        &self,
    ) -> &BTreeMap<(DcShorthand, DcShorthand), (u128, u128, DcShorthand)> {
        &self.active_single_props
    }

    /// Returns the map of all active multi-valued properties `(subject, prop_token, val_token) -> (has_a_id, has_val_id)`.
    #[must_use]
    pub fn active_multi_props(
        &self,
    ) -> &BTreeMap<(DcShorthand, DcShorthand, DcShorthand), (u128, u128)> {
        &self.active_multi_props
    }

    /// Returns the map of all active preferred replacements `(subject, val_token) -> (has_a_id, has_val_id, pref_qual_id)`.
    #[must_use]
    pub fn active_preferred(
        &self,
    ) -> &BTreeMap<(DcShorthand, DcShorthand), (u128, u128, u128)> {
        &self.active_preferred
    }

    /// Returns the map of all active direct binary statements `(verb, subject, object) -> stmt_id`.
    #[must_use]
    pub fn active_direct_stmts(
        &self,
    ) -> &BTreeMap<(DcShorthand, DcShorthand, DcShorthand), u128> {
        &self.active_direct_stmts
    }

    /// Returns the raw `.Semantic:Pattern` token slice for the active `Dc 594` (`Syntax rule`)
    /// property on `subj`, without allocating or decoding a Rust `SyntaxPattern` AST.
    #[must_use]
    pub fn get_syntax_rule_tokens(
        &self,
        subj: DcShorthand,
    ) -> Option<&[DcShorthand]> {
        let DcShorthand::Long(node_id) =
            self.get_single_property(subj, sh(DC_SYNTAX_RULE))?
        else {
            return None;
        };
        self.get_active_node(node_id).map(|n| n.tokens.as_slice())
    }

    /// Returns the raw `.Semantic:Pattern` token slice for a named syntax construct
    /// (e.g. `"number"`, `"string"`, `"value"`), without allocating a Rust `SyntaxPattern` AST.
    #[must_use]
    pub fn get_named_construct_tokens(
        &self,
        name: &str,
    ) -> Option<&[DcShorthand]> {
        let key = SeedEntityKey::NamedConstruct(name.to_string());
        let seed_id = self.get_seed_entity_id(&key)?;
        self.get_syntax_rule_tokens(DcShorthand::Long(seed_id))
    }

    /// Returns `true` if `tokens` begins with a Dc-WASM program (`Dc 1927 "Begin WASM program"`)
    /// or a `wasm_routine` literal (`Dc 260 "Begin literal value"`, `Dc 262 "Begin type definition"`,
    /// `Dc 280 "Built-in type identifier: Routine"`).
    #[must_use]
    pub fn tokens_start_wasm_definition(tokens: &[DcShorthand]) -> bool {
        let Some(&first) = tokens.first() else {
            return false;
        };
        if first == sh(DC_BEGIN_WASM_PROGRAM) {
            return true;
        }
        first == sh(DC_BEGIN_LITERAL)
            && tokens.get(1) == Some(&sh(DC_BEGIN_TYPE_DEF))
            && tokens.get(2) == Some(&sh(DC_TYPE_ROUTINE))
    }

    /// Resolves any active `.WASM` program (`Dc 1927 "Begin WASM program"`..`Dc 1928 "End WASM program"`)
    /// or `wasm_routine` definition attached to `subj` via `Dc 611 "Canonical decomposition"`,
    /// `Dc 571 "Decomposition"`, `Dc 614 "Semantic decomposition"`, or a trailing
    /// `Dc 607 "Pattern replacement"` action inside `subj`'s `Dc 594 "Syntax rule"`.
    #[must_use]
    pub fn get_wasm_definition_tokens(
        &self,
        subj: DcShorthand,
    ) -> Option<&[DcShorthand]> {
        for prop_dc in [
            DC_CANONICAL_DECOMPOSITION,
            DC_DECOMPOSITION,
            DC_SEMANTIC_DECOMPOSITION,
        ] {
            for entry in self.get_multi_properties(subj, sh(prop_dc)) {
                if let DcShorthand::Long(node_id) = entry.value
                    && let Some(node) = self.get_active_node(node_id)
                    && Self::tokens_start_wasm_definition(&node.tokens)
                {
                    return Some(node.tokens.as_slice());
                }
            }
        }

        let rule_tokens = self.get_syntax_rule_tokens(subj)?;
        let sh_open = sh(DC_BEGIN_ASSOCIATIVITY_GROUP);
        let sh_close = sh(DC_END_ASSOCIATIVITY_GROUP);
        let sh_repl = sh(DC_PATTERN_REPLACEMENT);
        if rule_tokens.last() != Some(&sh_close) {
            return None;
        }
        let mut depth = 0usize;
        let mut open_idx = None;
        for (idx, &tok) in rule_tokens.iter().enumerate().rev() {
            if tok == sh_close {
                depth = depth.saturating_add(1);
            } else if tok == sh_open {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    open_idx = Some(idx);
                    break;
                }
            }
        }
        let start = open_idx?;
        let group_slice = rule_tokens.get(start..)?;
        if group_slice.get(1) != Some(&sh_repl) {
            return None;
        }
        let inner = group_slice.get(2..group_slice.len().saturating_sub(1))?;
        if Self::tokens_start_wasm_definition(inner) {
            return Some(inner);
        }
        if inner.len() == 1
            && let Some(&DcShorthand::Long(node_id)) = inner.first()
            && let Some(node) = self.get_active_node(node_id)
            && Self::tokens_start_wasm_definition(&node.tokens)
        {
            return Some(node.tokens.as_slice());
        }
        None
    }

    /// Returns `true` if `source` is a trusted Collective Toolbox seed graph provenance source:
    /// `Dc 636 "Collective Toolbox Dc CSV seed database"` or `Dc 637 "Manual entry"`.
    #[must_use]
    pub fn is_trusted_seed_graph_data_source(source: DcShorthand) -> bool {
        source == sh(DC_COLLECTIVE_TOOLBOX_DC_CSV_SEED_DATABASE)
            || source == sh(DC_MANUAL_ENTRY)
    }

    /// Returns `true` if `node_id` originated from the host-authenticated bundled asset
    /// (`ctoolbox.rsrc` / `system_graph_history.rsrc`).
    #[must_use]
    pub fn is_bundled_origin_node(&self, node_id: u128) -> bool {
        self.bundled_origin_node_ids.contains(&node_id)
    }

    /// Verifies that `node_id` resides in the System block (`8_589_934_592..=17_179_869_183`),
    /// originated from the host-authenticated bundled asset (`ctoolbox.rsrc` /
    /// `system_graph_history.rsrc`), and carries verified `Dc 635`
    /// (`"Collective Toolbox seed graph data source"`) provenance (`Dc 636` or `Dc 637`)
    /// asserted by bundled System-block provenance statements.
    ///
    /// # Errors
    /// Returns an error if `node_id` is outside the System block, missing from the index,
    /// from an extra/unbundled resource, missing `Dc 635` provenance, or associated with an
    /// untrusted or unbundled `Dc 635` provenance statement.
    pub fn verify_authenticated_bundled_system_provenance(
        &self,
        node_id: u128,
    ) -> Result<()> {
        ensure!(
            (SYSTEM_REGION_START..=SYSTEM_REGION_END).contains(&node_id),
            "Node l{node_id} is outside the System block ({SYSTEM_REGION_START}..={SYSTEM_REGION_END}) and cannot supply bootstrap or supervisor authority"
        );
        ensure!(
            self.node_index_by_id.contains_key(&node_id),
            "System node l{node_id} is not present in the loaded seed graph"
        );
        ensure!(
            self.bundled_origin_node_ids.contains(&node_id),
            "Node l{node_id} did not originate from the authenticated bundled System asset (ctoolbox.rsrc); extra or unbundled resources cannot supply bootstrap authority"
        );

        if self.provenance_stmt_ids.contains(&node_id) {
            if let Some(assertions) = self.node_provenance_assertions.get(&node_id)
            {
                for &(has_a_id, has_val_id, source_tok) in assertions {
                    ensure!(
                        (SYSTEM_REGION_START..=SYSTEM_REGION_END)
                            .contains(&has_a_id)
                            && (SYSTEM_REGION_START..=SYSTEM_REGION_END)
                                .contains(&has_val_id),
                        "Provenance statement for l{node_id} (l{has_a_id} / l{has_val_id}) is outside the System block"
                    );
                    ensure!(
                        self.bundled_origin_node_ids.contains(&has_a_id)
                            && self.bundled_origin_node_ids.contains(&has_val_id),
                        "Provenance statement for l{node_id} (l{has_a_id} / l{has_val_id}) did not originate from the authenticated bundled asset"
                    );
                    ensure!(
                        Self::is_trusted_seed_graph_data_source(source_tok),
                        "Provenance statement for l{node_id} declares untrusted data source {source_tok} (expected Dc 636 or Dc 637)"
                    );
                }
            }
            return Ok(());
        }

        let Some(assertions) = self.node_provenance_assertions.get(&node_id)
        else {
            bail!(
                "System node l{node_id} has no active Dc 635 (Collective Toolbox seed graph data source) provenance assertion"
            );
        };
        ensure!(
            !assertions.is_empty(),
            "System node l{node_id} has no active Dc 635 (Collective Toolbox seed graph data source) provenance assertion"
        );
        for &(has_a_id, has_val_id, source_tok) in assertions {
            ensure!(
                (SYSTEM_REGION_START..=SYSTEM_REGION_END).contains(&has_a_id)
                    && (SYSTEM_REGION_START..=SYSTEM_REGION_END)
                        .contains(&has_val_id),
                "Provenance statement for System node l{node_id} (l{has_a_id} / l{has_val_id}) is outside the System block"
            );
            ensure!(
                self.bundled_origin_node_ids.contains(&has_a_id)
                    && self.bundled_origin_node_ids.contains(&has_val_id),
                "Provenance statement for System node l{node_id} (l{has_a_id} / l{has_val_id}) did not originate from the authenticated bundled asset"
            );
            ensure!(
                Self::is_trusted_seed_graph_data_source(source_tok),
                "System node l{node_id} has untrusted Dc 635 data source {source_tok} (expected Dc 636 or Dc 637)"
            );
        }
        Ok(())
    }

    /// Verifies that `node_id` and every `Dc 591 "Supersedes"` / `Dc 592 "Retracts"` statement
    /// along its revision chain satisfy [`Self::verify_authenticated_bundled_system_provenance`].
    pub fn verify_node_supersession_and_retraction_chain(
        &self,
        node_id: u128,
    ) -> Result<()> {
        let mut current = node_id;
        let mut visited = HashSet::new();
        loop {
            if !visited.insert(current) {
                bail!(
                    "Detected supersession cycle while verifying provenance chain starting at l{node_id}"
                );
            }
            self.verify_authenticated_bundled_system_provenance(current)?;
            if let Some(&retract_stmt_id) = self.retracted_by.get(&current) {
                self.verify_authenticated_bundled_system_provenance(
                    retract_stmt_id,
                )
                .with_context(|| {
                    format!(
                        "Untrusted or unbundled Dc 592 retraction statement l{retract_stmt_id} targeting l{current}"
                    )
                })?;
            }
            if let Some(&sup_stmt_id) =
                self.superseding_stmt_by_old_id.get(&current)
            {
                self.verify_authenticated_bundled_system_provenance(
                    sup_stmt_id,
                )
                .with_context(|| {
                    format!(
                        "Untrusted or unbundled Dc 591 supersession statement l{sup_stmt_id} targeting l{current}"
                    )
                })?;
            }
            if let Some(&next_id) = self.superseded_by.get(&current) {
                current = next_id;
            } else {
                break;
            }
        }
        Ok(())
    }

    /// Verifies that every statement (active or historical) touching `subj`, together with all
    /// `Dc 610 "Has value"` value statements and `Dc 591 "Supersedes"` / `Dc 592 "Retracts"`
    /// statements on those chains, satisfies [`Self::verify_authenticated_bundled_system_provenance`].
    pub fn verify_subject_provenance_and_history(
        &self,
        subj: DcShorthand,
    ) -> Result<()> {
        if let Some(stmt_ids) = self.subject_all_stmt_ids.get(&subj) {
            for &stmt_id in stmt_ids {
                self.verify_node_supersession_and_retraction_chain(stmt_id)
                    .with_context(|| {
                        format!(
                            "Statement l{stmt_id} on subject {subj} failed bundled System provenance verification"
                        )
                    })?;
                if let Some(has_vals) =
                    self.has_val_stmt_ids_by_has_a.get(&stmt_id)
                {
                    for &(has_val_id, _) in has_vals {
                        self.verify_node_supersession_and_retraction_chain(
                            has_val_id,
                        )
                        .with_context(|| {
                            format!(
                                "Value statement l{has_val_id} on statement l{stmt_id} (subject {subj}) failed bundled System provenance verification"
                            )
                        })?;
                    }
                }
            }
        }
        Ok(())
    }

    fn resolve_active_wasm_definition_entry(
        &self,
        subj: DcShorthand,
    ) -> Option<(u128, u128, u128, Vec<DcShorthand>)> {
        for prop_dc in [
            DC_CANONICAL_DECOMPOSITION,
            DC_DECOMPOSITION,
            DC_SEMANTIC_DECOMPOSITION,
            DC_APPROXIMATE_DECOMPOSITION,
        ] {
            for entry in self.get_multi_properties(subj, sh(prop_dc)) {
                if let DcShorthand::Long(def_id) = entry.value
                    && let Some(resolved_def_id) =
                        self.resolve_latest_node_id(def_id)
                    && let Some(node) = self.get_node(resolved_def_id)
                    && Self::tokens_start_wasm_definition(&node.tokens)
                {
                    return Some((
                        entry.has_a_id,
                        entry.has_val_id,
                        resolved_def_id,
                        node.tokens.clone(),
                    ));
                }
            }
        }

        let rule_entry =
            self.get_single_property_entry(subj, sh(DC_SYNTAX_RULE))?;
        let DcShorthand::Long(rule_node_id) = rule_entry.value else {
            return None;
        };
        let resolved_rule_node_id = self.resolve_latest_node_id(rule_node_id)?;
        let rule_node = self.get_node(resolved_rule_node_id)?;
        let rule_tokens = rule_node.tokens.as_slice();
        let sh_open = sh(DC_BEGIN_ASSOCIATIVITY_GROUP);
        let sh_close = sh(DC_END_ASSOCIATIVITY_GROUP);
        let sh_repl = sh(DC_PATTERN_REPLACEMENT);
        if rule_tokens.last() != Some(&sh_close) {
            return None;
        }
        let mut depth = 0usize;
        let mut open_idx = None;
        for (idx, &tok) in rule_tokens.iter().enumerate().rev() {
            if tok == sh_close {
                depth = depth.saturating_add(1);
            } else if tok == sh_open {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    open_idx = Some(idx);
                    break;
                }
            }
        }
        let start = open_idx?;
        let group_slice = rule_tokens.get(start..)?;
        if group_slice.get(1) != Some(&sh_repl) {
            return None;
        }
        let inner = group_slice.get(2..group_slice.len().saturating_sub(1))?;
        if Self::tokens_start_wasm_definition(inner) {
            return Some((
                rule_entry.has_a_id,
                rule_entry.has_val_id,
                resolved_rule_node_id,
                inner.to_vec(),
            ));
        }
        if inner.len() == 1
            && let Some(&DcShorthand::Long(target_id)) = inner.first()
            && let Some(resolved_target_id) =
                self.resolve_latest_node_id(target_id)
            && let Some(node) = self.get_node(resolved_target_id)
            && Self::tokens_start_wasm_definition(&node.tokens)
        {
            return Some((
                rule_entry.has_a_id,
                rule_entry.has_val_id,
                resolved_target_id,
                node.tokens.clone(),
            ));
        }
        None
    }

    /// Resolves an unpinned 1-Short-Dc bootstrap entry document (`bootstrap_tokens`) to its
    /// latest trusted System-block definition from the application-bundled trusted base (`ctoolbox.rsrc`).
    ///
    /// # Errors
    /// Returns an error if `bootstrap_tokens` is empty, begins with a `Dc 641` pin, contains
    /// multiple tokens or a non-Short Dc, or if any statement, supersession/retraction chain,
    /// or transitive dependency fails authenticated bundled System provenance verification.
    pub fn resolve_trusted_system_supervisor_bootstrap_tokens(
        &self,
        bootstrap_tokens: &[DcShorthand],
    ) -> Result<TrustedSystemSupervisorResolution> {
        let Some(&first) = bootstrap_tokens.first() else {
            bail!(
                "System supervisor bootstrap entry document cannot be empty (expected 1 unpinned Short Dc)"
            );
        };
        if first
            == sh(DC_GLOBAL_GRAPH_SYSTEM_BLOCK_MOST_RECENT_NODE_ID)
        {
            bail!(
                "System supervisor bootstrap entry document must be unpinned; Dc 641 (Global graph system block most recent node ID at time of document creation) pin is not permitted"
            );
        }
        ensure!(
            bootstrap_tokens.len() == 1,
            "System supervisor bootstrap entry document must consist of exactly 1 unpinned Short Dc, found {} tokens",
            bootstrap_tokens.len()
        );
        let DcShorthand::Short(short_id) = first else {
            bail!(
                "System supervisor bootstrap entry document must be a Short Dc, found {first}"
            );
        };
        self.resolve_trusted_system_supervisor_dc(DcChar::from_short(short_id))
    }

    /// Resolves the unpinned Short Dc `entry_dc` to its latest active System-block supervisor
    /// definition, verifying authenticated bundled-origin statement provenance (`ctoolbox.rsrc` +
    /// trusted `Dc 635` = `Dc 636` or `Dc 637`) across the entry character's statements,
    /// supersession (`Dc 591`) and retraction (`Dc 592`) chains, and all transitive dependencies.
    pub fn resolve_trusted_system_supervisor_dc(
        &self,
        entry_dc: DcChar,
    ) -> Result<TrustedSystemSupervisorResolution> {
        ensure!(
            matches!(sh(entry_dc), DcShorthand::Short(_)),
            "System supervisor entry Dc must be a Short Dc, found {entry_dc}"
        );
        ensure!(
            entry_dc != DC_GLOBAL_GRAPH_SYSTEM_BLOCK_MOST_RECENT_NODE_ID,
            "Dc 641 (Global graph system block most recent node ID at time of document creation) cannot be used as the System supervisor entry Dc"
        );
        let system_head_node_id = self
            .system_block_most_recent_node_id()
            .context("System seed graph has no allocated System-block HEAD node ID")?;
        ensure!(
            (SYSTEM_REGION_START..=SYSTEM_REGION_END)
                .contains(&system_head_node_id),
            "System seed graph HEAD l{system_head_node_id} is outside the System block"
        );

        let subj = sh(entry_dc);
        self.verify_subject_provenance_and_history(subj)?;

        let (
            statement_node_id,
            value_statement_node_id,
            supervisor_revision_node_id,
            definition_tokens,
        ) = self
            .resolve_active_wasm_definition_entry(subj)
            .with_context(|| {
                format!(
                    "No active System-block WASM supervisor definition found for {entry_dc}"
                )
            })?;

        self.verify_node_supersession_and_retraction_chain(statement_node_id)?;
        self.verify_node_supersession_and_retraction_chain(
            value_statement_node_id,
        )?;
        self.verify_node_supersession_and_retraction_chain(
            supervisor_revision_node_id,
        )?;

        // Traverse and verify all transitive dependencies reachable from the supervisor definition.
        let mut dependency_set = std::collections::BTreeSet::new();
        let mut queue = std::collections::VecDeque::new();
        let mut visited_subjects = HashSet::new();
        visited_subjects.insert(subj);
        queue.push_back(supervisor_revision_node_id);

        while let Some(node_id) = queue.pop_front() {
            if !dependency_set.insert(node_id) {
                continue;
            }
            self.verify_node_supersession_and_retraction_chain(node_id)
                .with_context(|| {
                    format!(
                        "Transitive supervisor dependency node l{node_id} failed bundled System provenance verification"
                    )
                })?;
            let resolved_id = self
                .resolve_latest_node_id(node_id)
                .with_context(|| {
                    format!(
                        "Transitive supervisor dependency node l{node_id} is retracted or unresolvable"
                    )
                })?;
            if resolved_id != node_id && dependency_set.insert(resolved_id) {
                self.verify_node_supersession_and_retraction_chain(resolved_id)?;
            }
            let node = self.get_node(resolved_id).with_context(|| {
                format!(
                    "Transitive supervisor dependency node l{resolved_id} is missing from seed graph"
                )
            })?;
            for &tok in &node.tokens {
                if let DcShorthand::Long(dep_id) = tok {
                    ensure!(
                        (SYSTEM_REGION_START..=SYSTEM_REGION_END)
                            .contains(&dep_id),
                        "Supervisor definition node l{resolved_id} references non-System node l{dep_id}"
                    );
                    if !dependency_set.contains(&dep_id) {
                        queue.push_back(dep_id);
                    }
                }
                if visited_subjects.insert(tok)
                    && let Some((dep_stmt_id, dep_val_id, dep_def_id, _)) =
                        self.resolve_active_wasm_definition_entry(tok)
                {
                    self.verify_subject_provenance_and_history(tok)?;
                    self.verify_node_supersession_and_retraction_chain(
                        dep_stmt_id,
                    )?;
                    self.verify_node_supersession_and_retraction_chain(
                        dep_val_id,
                    )?;
                    if !dependency_set.contains(&dep_def_id) {
                        queue.push_back(dep_def_id);
                    }
                }
            }
        }

        Ok(TrustedSystemSupervisorResolution {
            entry_dc,
            statement_node_id,
            value_statement_node_id,
            supervisor_revision_node_id,
            system_head_node_id,
            dependency_snapshot_node_ids: dependency_set.into_iter().collect(),
            definition_tokens,
        })
    }
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
    use crate::dc::{
        DC_APPROXIMATE_DECOMPOSITION, DC_BEGIN_GRAPH_NODE_DEFINITION,
        DC_CANONICAL_COMBINING_CLASS, DC_CANONICAL_DECOMPOSITION,
        DC_CASE_MAPPING_PARTNER, DC_COMPATIBILITY_DECOMPOSITION,
        DC_DESCRIPTION, DC_END_GRAPH_NODE_DEFINITION, DC_INFORMAL_ALIAS,
        DC_IS_CLOSING_DELIMITER_FOR, DC_IS_DEPRECATED, DC_IS_SUBTYPE_OF,
        DC_RANGE, DC_SEMANTIC_DECOMPOSITION,
    };
    use crate::dc_def::{BidiClass, DcDefn, GeneralCategory};
    use crate::graph_ledger::{
        encode_integer_value_tokens, encode_nodes_to_ctbnbdl_bytes,
        encode_statement_tokens, encode_string_value_tokens,
        update_ledger_in_memory,
    };
    use crate::syntax::parse_dc_syntax;
    use ctb_utilities::format_id::DC_FILE_EXTENSION;
    use std::collections::BTreeSet;

    fn make_test_defn(short_id: usize, name: &str, desc: &str) -> DcDefn {
        DcDefn {
            dc_id: 1_114_112 + u128::try_from(short_id).unwrap(),
            short_id: Some(short_id),
            ident: Some(format!("DC_TEST_{short_id}")),
            stable_ident: false,
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
            aliases: vec!["alias-one".to_string()],
            formal_aliases: Vec::new(),
            cross_references: Vec::new(),
            annotations: Vec::new(),
            implies: vec!["571".to_string()],
            based_on: Vec::new(),
            os_associations: Vec::new(),
            magic_names: Vec::new(),
            syntax: Some(parse_dc_syntax(":~ [value]").unwrap()),
            closes: None,
            description: desc.to_string(),
            format: None,
            source_file: "test.csv".to_string(),
            line_number: 2,
        }
    }

    #[crate::ctb_test]
    fn test_load_seed_graph_from_asset_bundle() -> Result<()> {
        let index = SeedGraphIndex::shared()?;
        assert!(
            index.node_count() >= 14_310,
            "Expected at least 14,310 nodes in bundled seed graph, found {}",
            index.node_count()
        );
        assert_eq!(
            index.superseded_count(),
            0,
            "Active-only bundled seed graph should have 0 superseded nodes, found {}",
            index.superseded_count()
        );
        let history_index =
            SeedGraphIndex::load_with_history_from_asset_bundle()?;
        assert!(
            history_index.superseded_count() >= 13,
            "Expected live supersessions in seed graph with history, found {}",
            history_index.superseded_count()
        );

        // Verify Dc 617 (`Begin graph node definition`) has its superseded description and syntax rule
        let dc_617 = sh(DC_BEGIN_GRAPH_NODE_DEFINITION);
        assert_eq!(
            index.get_single_property_string(dc_617, sh(DC_CANONICAL_NAME)),
            Some("Begin graph node definition")
        );
        assert_eq!(
            index.get_single_property_string(
                dc_617,
                sh(DC_PROGRAMMATIC_IDENTIFIER)
            ),
            Some("BeginGraphNodeDefinition")
        );
        let rule_617 = index
            .decode_syntax_rule_for_subject(dc_617)?
            .expect("Expected syntax rule for Dc 617");
        let expected_617 = parse_dc_syntax(":~ [atom_dc] [^618]* 618")?;
        assert_eq!(rule_617.pattern, expected_617.pattern);

        // Verify Dc 618 closes Dc 617 (`590 618 617`)
        let dc_618 = sh(DC_END_GRAPH_NODE_DEFINITION);
        assert!(index.has_direct_statement(
            sh(DC_IS_CLOSING_DELIMITER_FOR),
            dc_618,
            dc_617
        ));
        assert_eq!(
            index.get_direct_objects(sh(DC_IS_CLOSING_DELIMITER_FOR), dc_618),
            &[dc_617]
        );

        // Verify f192 (`IaMetaXml`) has its superseded `~_meta\.xml$~` pattern extension
        // and f316 (`AppleDouble`) has both literal (`.ADF`, `.rsrc`) and regex pattern extensions
        let f192 = DcShorthand::Format(192);
        let exts_192 = index.get_multi_properties(f192, sh(DC_FILE_EXTENSION));
        assert_eq!(exts_192.len(), 1);
        let DcShorthand::Long(pat_node_id) = exts_192[0].value else {
            panic!("Expected Long node ID for f192 pattern extension");
        };
        let decoded_ext_rule = index.decode_pattern_node(pat_node_id)?;
        let expected_ext_rule =
            crate::syntax::parse_extension_regex("~_meta\\.xml$~")?;
        assert_eq!(decoded_ext_rule.pattern, expected_ext_rule.pattern);

        let f316 = DcShorthand::Format(316);
        let exts_316 = index.get_multi_properties(f316, sh(DC_FILE_EXTENSION));
        assert_eq!(exts_316.len(), 7);
        assert!(
            exts_316
                .iter()
                .any(|e| index.get_string(e.value) == Some(".ADF"))
        );
        assert!(
            exts_316
                .iter()
                .any(|e| index.get_string(e.value) == Some(".rsrc"))
        );

        Ok(())
    }

    #[crate::ctb_test]
    fn test_supersession_chains_and_retractions_in_seed_graph_index()
    -> Result<()> {
        let mut defn = make_test_defn(900, "Name v1", "Desc v1");
        defn.combining_class = 230;
        defn.casing_partner = Some(901);
        defn.is_deprecated = true;
        defn.decompositions = vec![
            "<prefer> 310".to_string(),
            "<equiv> 311".to_string(),
            "<approx> 312".to_string(),
            "<semantic> 313".to_string(),
            "<compat> 314".to_string(),
        ];
        let scripts = BTreeSet::from([".Semantic".to_string()]);

        // Revision 1
        let (nodes1, _) = update_ledger_in_memory(
            Vec::new(),
            &[defn.clone()],
            &[],
            &scripts,
            None,
        )?;
        let idx1 = SeedGraphIndex::from_nodes(nodes1.clone());
        let subj = DcShorthand::Short(900);
        let v1_name_entry = idx1
            .get_single_property_entry(subj, sh(DC_CANONICAL_NAME))
            .unwrap();
        assert_eq!(
            idx1.get_single_property_string(subj, sh(DC_CANONICAL_NAME)),
            Some("Name v1")
        );
        assert_eq!(
            idx1.get_single_property_integer(
                subj,
                sh(DC_CANONICAL_COMBINING_CLASS)
            ),
            Some(230)
        );
        assert_eq!(
            idx1.get_single_property(subj, sh(DC_CASE_MAPPING_PARTNER)),
            Some(DcShorthand::Short(901))
        );
        assert!(idx1.has_direct_statement(
            sh(DC_IS_DEPRECATED),
            subj,
            sh(crate::dc::DC_BOOLEAN_TRUE)
        ));
        assert_eq!(
            idx1.get_preferred_replacements(subj),
            &[DcShorthand::Short(310)]
        );
        assert_eq!(
            idx1.get_multi_properties(subj, sh(DC_CANONICAL_DECOMPOSITION))
                .len(),
            1
        );
        assert_eq!(
            idx1.get_multi_properties(subj, sh(DC_APPROXIMATE_DECOMPOSITION))
                .len(),
            1
        );
        assert_eq!(
            idx1.get_multi_properties(subj, sh(DC_SEMANTIC_DECOMPOSITION))
                .len(),
            1
        );
        assert_eq!(
            idx1.get_multi_properties(subj, sh(DC_COMPATIBILITY_DECOMPOSITION))
                .len(),
            1
        );

        // Revision 2: update name to v2 (supersedes v1)
        defn.name = "Name v2".to_string();
        let (nodes2, _) = update_ledger_in_memory(
            nodes1,
            &[defn.clone()],
            &[],
            &scripts,
            None,
        )?;
        let idx2 = SeedGraphIndex::from_nodes(nodes2.clone());
        let v2_name_entry = idx2
            .get_single_property_entry(subj, sh(DC_CANONICAL_NAME))
            .unwrap();
        assert_eq!(v2_name_entry.has_a_id, v1_name_entry.has_a_id);
        assert_ne!(v2_name_entry.has_val_id, v1_name_entry.has_val_id);

        // Revision 3: update name to v3 (chained supersession: v3 -> v2 -> v1) and retract alias, implies, deprecation, and <prefer>
        defn.name = "Name v3".to_string();
        defn.aliases.clear();
        defn.implies.clear();
        defn.is_deprecated = false;
        defn.decompositions.clear();
        let (nodes3, _) =
            update_ledger_in_memory(nodes2, &[defn], &[], &scripts, None)?;

        // Roundtrip through `.ctbnbdl` binary encoding and load via `from_ctbnbdl_bytes`
        let bundle_bytes = encode_nodes_to_ctbnbdl_bytes(&nodes3)?;
        let idx3 = SeedGraphIndex::from_ctbnbdl_bytes(&bundle_bytes)?;
        let v3_name_entry = idx3
            .get_single_property_entry(subj, sh(DC_CANONICAL_NAME))
            .unwrap();

        assert_eq!(
            idx3.get_single_property_string(subj, sh(DC_CANONICAL_NAME)),
            Some("Name v3")
        );
        assert!(!idx3.is_node_active(v1_name_entry.has_val_id));
        assert!(!idx3.is_node_active(v2_name_entry.has_val_id));
        assert!(idx3.is_node_active(v3_name_entry.has_val_id));
        assert_eq!(
            idx3.superseded_by(v1_name_entry.has_val_id),
            Some(v2_name_entry.has_val_id)
        );
        assert_eq!(
            idx3.superseded_by(v2_name_entry.has_val_id),
            Some(v3_name_entry.has_val_id)
        );
        assert_eq!(
            idx3.resolve_latest_node_id(v1_name_entry.has_val_id),
            Some(v3_name_entry.has_val_id)
        );

        // Verify retractions removed the alias, subtype statement, deprecation, and preferred replacement
        assert!(
            idx3.get_multi_properties(subj, sh(DC_INFORMAL_ALIAS))
                .is_empty()
        );
        assert!(
            idx3.get_direct_objects(sh(DC_IS_SUBTYPE_OF), subj)
                .is_empty()
        );
        assert!(!idx3.has_direct_statement(
            sh(DC_IS_DEPRECATED),
            subj,
            sh(crate::dc::DC_BOOLEAN_TRUE)
        ));
        assert!(idx3.get_preferred_replacements(subj).is_empty());
        assert!(idx3.retracted_count() > 0);

        Ok(())
    }

    #[crate::ctb_test]
    fn test_retracted_retraction_restores_statement() {
        // Node 0: string "test"
        let id0 = SYSTEM_REGION_START;
        // Node 1: `587 900 571` (`Is a subtype of (900, 571)`)
        let id1 = SYSTEM_REGION_START + 1;
        // Node 2: `592 615 l<id1>` (retracts Node 1)
        let id2 = SYSTEM_REGION_START + 2;
        // Node 3: `592 615 l<id2>` (retracts the retraction in Node 2!)
        let id3 = SYSTEM_REGION_START + 3;

        let nodes = vec![
            LedgerNode {
                id: id0,
                tokens: encode_string_value_tokens("test"),
                comment: String::new(),
            },
            LedgerNode {
                id: id1,
                tokens: encode_statement_tokens(
                    sh(DC_IS_SUBTYPE_OF),
                    DcShorthand::Short(900),
                    DcShorthand::Short(571),
                ),
                comment: String::new(),
            },
            LedgerNode {
                id: id2,
                tokens: encode_statement_tokens(
                    sh(DC_RETRACTS),
                    sh(DC_ENCLOSING_DOCUMENT),
                    DcShorthand::Long(id1),
                ),
                comment: String::new(),
            },
            LedgerNode {
                id: id3,
                tokens: encode_statement_tokens(
                    sh(DC_RETRACTS),
                    sh(DC_ENCLOSING_DOCUMENT),
                    DcShorthand::Long(id2),
                ),
                comment: String::new(),
            },
        ];

        let index = SeedGraphIndex::from_nodes(nodes);
        assert!(!index.is_node_active(id2));
        assert!(index.is_node_active(id1));
        assert!(index.has_direct_statement(
            sh(DC_IS_SUBTYPE_OF),
            DcShorthand::Short(900),
            DcShorthand::Short(571)
        ));
    }

    #[crate::ctb_test]
    fn test_all_live_syntax_rules_and_named_constructs_roundtrip_from_seed_graph()
    -> Result<()> {
        let index = SeedGraphIndex::shared()?;

        // 1. Verify every named syntax construct in README-named-types.csv decodes identically
        let named_types_bytes =
            crate::get_dc_data_file("README-named-types.csv")
                .context("Missing embedded README-named-types.csv")?;
        let table = csv_tools::parse_csv_reader(
            &named_types_bytes,
            csv_tools::CsvParseOptions {
                has_header: true,
                flexible: true,
                ..Default::default()
            },
        )?;
        let reconstructed_named: HashMap<String, ReconstructedNamedType> = index
            .reconstruct_named_types()?
            .into_iter()
            .map(|r| (r.name.clone(), r))
            .collect();
        for row_idx in 0..table.row_count() {
            let Some(row) = table.row(row_idx) else {
                continue;
            };
            if row.iter().all(|c| c.trim().is_empty()) {
                continue;
            }
            let name = row[0].trim();
            let raw_syntax = row[1].trim();
            let expected_notes = row.get(2).map_or("", |s| s.trim());
            let expected = parse_dc_syntax(raw_syntax)?;
            let decoded_pat = index
                .decode_named_construct_pattern(name)?
                .with_context(|| {
                    format!(
                        "Named construct '{name}' missing from SeedGraphIndex"
                    )
                })?;
            assert_eq!(
                decoded_pat, expected.pattern,
                "Named construct '{name}' pattern mismatch after decoding from seed graph"
            );
            let rec = reconstructed_named.get(name).with_context(|| {
                format!(
                    "Named construct '{name}' missing from reconstruct_named_types()"
                )
            })?;
            assert_eq!(rec.syntax.pattern, expected.pattern);
            assert_eq!(
                rec.description, expected_notes,
                "Named construct '{name}' description (Dc 573) mismatch after decoding from seed graph"
            );
        }

        // 2. Verify every Dc syntax rule and @stable_ident() declaration in the dataset decodes identically from the seed graph
        for defn in crate::lookup::get_all_dc_defns() {
            let subj = DcShorthand::from_global_id(defn.dc_id);
            assert_eq!(
                index.has_stable_programmatic_identifier(subj),
                defn.stable_ident,
                "@stable_ident() mismatch in SeedGraphIndex for {subj} ({})",
                defn.name
            );
            let Some(expected_rule) = &defn.syntax else {
                continue;
            };
            let decoded_rule = index
                .decode_syntax_rule_for_subject(subj)?
                .with_context(|| {
                    format!(
                        "Missing syntax rule in SeedGraphIndex for {subj} ({})",
                        defn.name
                    )
                })?;
            assert_eq!(
                decoded_rule.pattern, expected_rule.pattern,
                "Syntax pattern mismatch in SeedGraphIndex for {subj} ({})",
                defn.name
            );
            assert_eq!(
                decoded_rule.action, expected_rule.action,
                "Syntax action mismatch in SeedGraphIndex for {subj} ({})",
                defn.name
            );
        }

        // Verify f199 (`Integer`), f200 (`Natural0`), and f351 (`Natural1`) reconstruct `stable_ident: true` in a vacuum
        assert!(index.has_stable_programmatic_identifier(DcShorthand::Format(199)));
        assert!(index.has_stable_programmatic_identifier(DcShorthand::Format(200)));
        assert!(index.has_stable_programmatic_identifier(DcShorthand::Format(351)));
        assert!(!index.has_stable_programmatic_identifier(DcShorthand::Format(207)));
        assert!(!index.has_stable_programmatic_identifier(DcShorthand::Format(84)));
        let reconstructed_formats = index.reconstruct_format_defns()?;
        for (fid, expected_ident) in [(199usize, "Integer"), (200, "Natural0"), (351, "Natural1")] {
            let rec_fmt = reconstructed_formats
                .iter()
                .find(|d| d.short_id == Some(fid))
                .expect("Expected format in reconstruct_format_defns()");
            assert!(
                rec_fmt.stable_ident,
                "Expected f{fid} ({expected_ident}) to reconstruct stable_ident == true from seed graph"
            );
            assert_eq!(rec_fmt.ident.as_deref(), Some(expected_ident));
        }

        // 3. Verify integer value node decoding roundtrip
        for &val in &[0u128, 1, 2, 63, 64, 230, 1024, 1_000_000] {
            let tokens = encode_integer_value_tokens(val);
            assert_eq!(decode_integer_value_tokens(&tokens), Some(val));
        }

        // 4. Verify that the active bundle (`SeedGraphIndex::shared()`) has all inactive/historical
        // nodes filtered out at build time, while `load_with_history_from_asset_bundle()` loads
        // both `data/graph/*.ctbnbdl` and `data/graph/system/*.ctbnbdl` (100% of `.ctbnodes`),
        // and both resolve identical active properties and statements.
        assert_eq!(
            index.inactive_count(),
            0,
            "Active-only SeedGraphIndex::shared() should contain 0 inactive nodes"
        );
        let full_index = SeedGraphIndex::load_with_history_from_asset_bundle()?;
        let repo_root = crate::updater::find_repository_root()?;
        let raw_ledger_nodes = crate::graph_ledger::load_ledger_nodes_from_dir(
            &repo_root.join("assets/data/graph"),
        )?;
        assert_eq!(
            full_index.node_count(),
            raw_ledger_nodes.len(),
            "Active + history .ctbnbdl bundles must account for 100% of nodes in assets/data/graph/*.ctbnodes"
        );
        assert!(
            full_index.inactive_count() > 0,
            "Full index with history should include superseded nodes from data/graph/system/*.ctbnbdl"
        );
        assert_eq!(index.active_single_props, full_index.active_single_props);
        assert_eq!(index.active_multi_props, full_index.active_multi_props);
        assert_eq!(index.active_preferred, full_index.active_preferred);
        assert_eq!(index.active_direct_stmts, full_index.active_direct_stmts);
        assert_eq!(index.seed_entity_ids, full_index.seed_entity_ids);

        Ok(())
    }

    #[crate::ctb_test]
    fn test_ambiguity_candidates_require_explicit_revisions() -> Result<()> {
        let original_id = 8_589_934_592_u128;
        let candidate_one = original_id.saturating_add(1);
        let candidate_two = original_id.saturating_add(2);
        let ambiguity_id = original_id.saturating_add(3);
        let supersession_id = original_id.saturating_add(4);
        let reference_id = original_id.saturating_add(5);
        let retraction_id = original_id.saturating_add(6);
        for (first, second) in [
            (candidate_one, candidate_two),
            (candidate_two, candidate_one),
        ] {
            let mut nodes = crate::graph_ledger::load_ledger_nodes_from_str(
                &format!(
                    "l{original_id}: u78\nl{candidate_one}: 587 900 901\nl{candidate_two}: 587 900 902\nl{ambiguity_id}: 348 310 275 l{first} 312 l{second} 311\nl{supersession_id}: 591 l{ambiguity_id} l{original_id}\nl{reference_id}: 588 900 l{candidate_one}\n"
                ),
                "ambiguity-test",
            )?;
            let index = SeedGraphIndex::from_nodes(nodes.clone());
            assert_eq!(
                index.get_ambiguous_candidates(original_id),
                Some(vec![first, second])
            );
            assert!(index.has_direct_statement(
                DcShorthand::Short(587),
                DcShorthand::Short(900),
                DcShorthand::Short(901)
            ));
            assert!(index.has_direct_statement(
                DcShorthand::Short(587),
                DcShorthand::Short(900),
                DcShorthand::Short(902)
            ));
            assert!(index.has_direct_statement(
                DcShorthand::Short(588),
                DcShorthand::Short(900),
                DcShorthand::Long(candidate_one)
            ));

            nodes.extend(crate::graph_ledger::load_ledger_nodes_from_str(
                &format!("l{retraction_id}: 592 615 l{candidate_one}\n"),
                "ambiguity-test",
            )?);
            let revised = SeedGraphIndex::from_nodes(nodes);
            assert!(!revised.is_node_active(candidate_one));
            assert!(revised.is_node_active(candidate_two));
            assert!(!revised.has_direct_statement(
                DcShorthand::Short(587),
                DcShorthand::Short(900),
                DcShorthand::Short(901)
            ));
            assert!(revised.has_direct_statement(
                DcShorthand::Short(587),
                DcShorthand::Short(900),
                DcShorthand::Short(902)
            ));
            assert_eq!(
                revised.get_ambiguous_candidates(original_id),
                Some(vec![first, second])
            );
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_ambiguity_recovery_prefix_invariants() -> Result<()> {
        let full_index = SeedGraphIndex::load_with_history_from_asset_bundle()?;
        let full_index = SeedGraphIndex::from_nodes(
            full_index
                .nodes
                .into_iter()
                .filter(|node| node.id <= 8_589_958_343)
                .collect(),
        );
        let (active_nodes, _) =
            crate::ctbnbdl::partition_active_and_inactive_nodes_with(
                &full_index.nodes,
                crate::graph_ledger::default_ctbnbdl_framing_dcs(),
            );
        let active_index = SeedGraphIndex::from_nodes(active_nodes);

        // 1. Exact node counts after Step 2a (2,465), Step 2b (2,838), and Step 3 (616 retractions):
        // Only the immutable recovery prefix is checked; later allocations and revisions are excluded.
        assert_eq!(active_index.node_count(), 17_147);
        assert_eq!(active_index.inactive_count(), 0);
        assert_eq!(full_index.node_count(), 23_752);
        assert_eq!(full_index.inactive_count(), 2_931);

        // 2. Verify all 24 unambiguous overwritten IDs have no Dc 348 "Ambiguous values" candidates
        let unambiguous_ids: Vec<u128> = (8_589_951_799..=8_589_951_801)
            .chain(8_589_951_803..=8_589_951_812)
            .chain(8_589_951_814..=8_589_951_824)
            .collect();
        assert_eq!(unambiguous_ids.len(), 24);
        for &id in &unambiguous_ids {
            assert_eq!(
                full_index.get_ambiguous_candidates(id),
                None,
                "Unambiguous ID l{id} must not resolve to a Dc 348 'Ambiguous values' list"
            );
        }

        // 3. Verify all 602 dependency-ambiguous overwritten IDs resolve to `[r1_eff(N), r2_eff(N)]`,
        // where `r2_eff(N)` is active for all 602 IDs and `r1_eff(N)` is retracted for all 559 statements
        // (and active for the 43 value nodes).
        let ambiguous_ids: Vec<u128> = std::iter::once(8_589_951_802)
            .chain(std::iter::once(8_589_951_813))
            .chain(8_589_951_825..=8_589_952_424)
            .collect();
        assert_eq!(ambiguous_ids.len(), 602);
        let mut r1_retracted_stmts = 0usize;
        let mut r1_active_values = 0usize;
        for &orig_id in &ambiguous_ids {
            assert!(
                !full_index.is_node_active(orig_id),
                "Overwritten ID l{orig_id} must be superseded"
            );
            let cands = full_index
                .get_ambiguous_candidates(orig_id)
                .with_context(|| {
                    format!(
                        "Ambiguous ID l{orig_id} is missing Dc 348 'Ambiguous values' candidates"
                    )
                })?;
            assert_eq!(
                cands.len(),
                2,
                "Expected 2 Dc 348 candidates for l{orig_id}"
            );
            let r1_eff = cands[0];
            let r2_eff = cands[1];
            assert!(
                full_index.is_node_active(r2_eff),
                "Run 2 candidate l{r2_eff} (for l{orig_id}) must be active"
            );
            assert!(
                active_index.get_node(r2_eff).is_some(),
                "Run 2 candidate l{r2_eff} must be present in active recovery prefix"
            );
            if full_index.is_node_active(r1_eff) {
                r1_active_values += 1;
            } else {
                r1_retracted_stmts += 1;
                assert!(
                    active_index.get_node(r1_eff).is_none(),
                    "Retracted Run 1 statement candidate l{r1_eff} must be excluded from active recovery prefix"
                );
            }
        }
        assert_eq!(r1_retracted_stmts, 559);
        assert_eq!(r1_active_values, 43);

        // 4. Verify all 57 restored Run 1 tail nodes (`l8589952425`..`l8589952481`) are inactive in the
        // live index (54 superseded by `l8589957620`..`l8589957727` and retracted in Step 3, plus 3 directly retracted in Step 3).
        for tail_id in 8_589_952_425..=8_589_952_481 {
            assert!(
                full_index.get_node(tail_id).is_some(),
                "Restored Run 1 tail node l{tail_id} must exist in historical index"
            );
            assert!(
                !full_index.is_node_active(tail_id),
                "Restored Run 1 tail node l{tail_id} must be inactive after Step 3"
            );
            assert!(
                active_index.get_node(tail_id).is_none(),
                "Restored Run 1 tail node l{tail_id} must not be in active recovery prefix"
            );
        }

        // 5. Verify committed journal provenance and allocated .ctbnode-incoming files across the repository.
        let repo_root = crate::updater::find_repository_root()?;
        let prov_map =
            crate::graph_ledger::inbox::verify_repo_committed_node_provenance(
                &repo_root,
            )?;
        for node_id in 8_589_952_425..=8_589_958_343 {
            assert_eq!(
                prov_map.owner(node_id),
                crate::graph_ledger::inbox::NodeProvenance::Csv,
                "Expected recovered/reconciled node l{node_id} to have Csv provenance"
            );
        }

        Ok(())
    }

    #[crate::ctb_test]
    fn test_seed_graph_and_syntax_resolver_at_most_recent() -> Result<()> {
        use crate::syntax::{CharTarget, SyntaxRuleResolver};

        let scripts = BTreeSet::from([".Semantic".to_string()]);
        let subj = DcShorthand::Short(915);

        // Revision 1: Dc 915 has syntax rule `:~ [atom_dc]`
        let mut defn = make_test_defn(915, "Test Op", "Test syntax rule op");
        defn.syntax = Some(parse_dc_syntax(":~ [atom_dc]")?);
        let (nodes1, _) = update_ledger_in_memory(
            Vec::new(),
            &[defn.clone()],
            &[],
            &scripts,
            None,
        )?;
        let t1 = nodes1.last().expect("Expected Revision 1 nodes").id;

        // Revision 2: Dc 915 syntax rule superseded by `:~ [atom_dc] [atom_dc]`
        defn.syntax = Some(parse_dc_syntax(":~ [atom_dc] [atom_dc]")?);
        let (nodes2, _) = update_ledger_in_memory(
            nodes1,
            &[defn.clone()],
            &[],
            &scripts,
            None,
        )?;
        let t2 = nodes2.last().expect("Expected Revision 2 nodes").id;
        assert!(t2 > t1);

        // Revision 3: Dc 915 syntax rule retracted
        defn.syntax = None;
        let (nodes3, _) =
            update_ledger_in_memory(nodes2, &[defn], &[], &scripts, None)?;
        let t3 = nodes3.last().expect("Expected Revision 3 nodes").id;
        assert!(t3 > t2);

        // Verify SeedGraphIndex::from_nodes_at_most_recent & DatasetRuleResolver::from_seed_graph at t1, t2, t3
        let idx_t1 = SeedGraphIndex::from_nodes_at_most_recent(nodes3.clone(), t1);
        assert_eq!(idx_t1.system_block_most_recent_node_id(), Some(t1));
        let resolver_t1 = crate::syntax::DatasetRuleResolver::from_seed_graph(&idx_t1)?;
        let rule_t1 = resolver_t1
            .resolve_rule(&CharTarget::Dc(915))
            .expect("Expected syntax rule at t1");
        assert_eq!(rule_t1.pattern, parse_dc_syntax(":~ [atom_dc]")?.pattern);

        let idx_t2 = SeedGraphIndex::from_nodes_at_most_recent(nodes3.clone(), t2);
        assert_eq!(idx_t2.system_block_most_recent_node_id(), Some(t2));
        let resolver_t2 = crate::syntax::DatasetRuleResolver::from_seed_graph(&idx_t2)?;
        let rule_t2 = resolver_t2
            .resolve_rule(&CharTarget::Dc(915))
            .expect("Expected syntax rule at t2");
        assert_eq!(
            rule_t2.pattern,
            parse_dc_syntax(":~ [atom_dc] [atom_dc]")?.pattern
        );

        let idx_t3 = SeedGraphIndex::from_nodes_at_most_recent(nodes3, t3);
        assert_eq!(idx_t3.system_block_most_recent_node_id(), Some(t3));
        assert!(idx_t3.decode_syntax_rule_for_subject(subj)?.is_none());
        let resolver_t3 = crate::syntax::DatasetRuleResolver::from_seed_graph(&idx_t3)?;
        assert!(resolver_t3.resolve_rule(&CharTarget::Dc(915)).is_none());

        // Verify historical asset bundle reconstruction on a real historical System-block node ID:
        // At `l8589958834` (before the `AddSystemGraphMostRecent` rename supersession at `l8589958838`..`l8589958845`),
        // `f655` had programmatic identifier `"AddSystemGraphWatermark"`, whereas at HEAD it is `"AddSystemGraphMostRecent"`.
        let f655 = DcShorthand::Format(655);
        let hist_idx =
            SeedGraphIndex::load_at_most_recent_from_asset_bundle(8_589_958_834)?;
        assert_eq!(
            hist_idx.system_block_most_recent_node_id(),
            Some(8_589_958_834)
        );
        assert_eq!(
            hist_idx.get_single_property_string(
                f655,
                sh(DC_PROGRAMMATIC_IDENTIFIER)
            ),
            Some("AddSystemGraphWatermark")
        );

        let head_idx = SeedGraphIndex::shared()?;
        let head_id = head_idx
            .system_block_most_recent_node_id()
            .expect("Expected HEAD system_block_most_recent_node_id");
        let head_pinned =
            SeedGraphIndex::load_at_most_recent_from_asset_bundle(head_id)?;
        assert!(Arc::ptr_eq(head_idx, &head_pinned));
        assert_eq!(
            head_pinned.get_single_property_string(
                f655,
                sh(DC_PROGRAMMATIC_IDENTIFIER)
            ),
            Some("AddSystemGraphMostRecent")
        );

        // Gate 1: Reject pins newer than the local graph (`most_recent_node_id > head_id`)
        // rather than silently substituting `HEAD`.
        let future_id = head_id.saturating_add(1);
        let future_err =
            SeedGraphIndex::load_at_most_recent_from_asset_bundle(future_id)
                .expect_err("Pin newer than HEAD must fail");
        assert!(
            future_err
                .to_string()
                .contains(&format!("l{future_id} > l{head_id}")),
            "Unexpected error for future pin: {future_err}"
        );
        let future_res_err =
            crate::syntax::syntax_resolver_at_most_recent(future_id)
                .expect_err("syntax_resolver_at_most_recent must reject pin newer than HEAD");
        assert!(
            future_res_err
                .to_string()
                .contains(&format!("l{future_id} > l{head_id}")),
            "Unexpected resolver error for future pin: {future_res_err}"
        );

        // Gate 1: Reject unsupported graph blocks (Data block and blocks above System).
        for unsupported_id in [
            ctb_storage_minimal::global_graph_layout::DATA_REGION_START,
            ctb_storage_minimal::global_graph_layout::SYSTEM_REGION_END
                .saturating_add(1),
        ] {
            let err = SeedGraphIndex::load_at_most_recent_from_asset_bundle(
                unsupported_id,
            )
            .expect_err("Unsupported graph block pin must fail");
            assert!(
                err.to_string().contains("unsupported graph block"),
                "Expected unsupported graph block error for l{unsupported_id}, got: {err}"
            );
        }

        // Gate 1: Require complete historical graph bundles when resolving historical pins.
        let prefix_id = SYSTEM_REGION_START;
        let prefix_nodes = vec![
            LedgerNode {
                id: prefix_id,
                tokens: Vec::new(),
                comment: String::new(),
            },
            LedgerNode {
                id: prefix_id.saturating_add(2),
                tokens: Vec::new(),
                comment: String::new(),
            },
        ];
        let prefix_index =
            SeedGraphIndex::from_active_and_history_nodes_at_most_recent(
                Some(prefix_id.saturating_add(2)),
                prefix_nodes,
                None,
                Vec::new(),
                prefix_id,
            )?;
        assert_eq!(
            prefix_index.system_block_most_recent_node_id(),
            Some(prefix_id),
        );

        let full_hist = SeedGraphIndex::load_with_history_from_asset_bundle()?;
        let (part_active, part_history) =
            crate::ctbnbdl::partition_active_and_inactive_nodes_with(
                &full_hist.nodes,
                crate::graph_ledger::default_ctbnbdl_framing_dcs(),
            );
        let missing_hist_err =
            SeedGraphIndex::from_active_and_history_nodes_at_most_recent(
                Some(head_id),
                part_active.clone(),
                None,
                Vec::new(),
                8_589_958_834,
            )
            .expect_err("Historical pin without history bundle must fail");
        assert!(
            missing_hist_err
                .to_string()
                .contains("system_graph_history.rsrc"),
            "Expected system_graph_history.rsrc error when history is absent, got: {missing_hist_err}"
        );

        let mut gapped_history = part_history;
        let removed_hist_node = gapped_history.remove(0);
        let incomplete_hist_err =
            SeedGraphIndex::from_active_and_history_nodes_at_most_recent(
                Some(head_id),
                part_active,
                Some(head_id),
                gapped_history,
                8_589_958_834,
            )
            .expect_err("Historical pin with gap in history bundle must fail");
        assert!(
            incomplete_hist_err.to_string().contains(&format!(
                "missing node l{}",
                removed_hist_node.id
            )),
            "Expected missing node error for incomplete history, got: {incomplete_hist_err}"
        );

        // Verify that no active Dc 594 "Syntax rule" pattern node at HEAD uses deprecated Dc 4 "!Range (deprecated)"
        // or legacy Dc 584 "Has a" string subtype qualifiers, while pinning to `l8589958931` (before structured
        // range/subtype encoding) still decodes historical Dc 4 and Dc 584 pattern nodes cleanly.
        let sh_syntax_rule = sh(DC_SYNTAX_RULE);
        let sh_legacy_range = sh(DC_RANGE);
        let sh_has_a = sh(DC_HAS_A);
        for (&(subj, prop), &(_, _, val_tok)) in &head_idx.active_single_props {
            if prop == sh_syntax_rule
                && let DcShorthand::Long(pat_node_id) = val_tok
            {
                let pat_node = head_idx
                    .get_node(pat_node_id)
                    .expect("Active syntax rule node must exist");
                assert!(
                    !pat_node.tokens.contains(&sh_legacy_range),
                    "Active syntax rule node l{pat_node_id} on {subj} must not contain deprecated Dc 4 '!Range (deprecated)'"
                );
                assert!(
                    !pat_node.tokens.contains(&sh_has_a),
                    "Active syntax rule node l{pat_node_id} on {subj} must not contain legacy Dc 584 string subtype qualifier"
                );
            }
        }

        let pre_range_idx =
            SeedGraphIndex::load_at_most_recent_from_asset_bundle(8_589_958_931)?;
        let pre_range_resolver =
            crate::syntax::DatasetRuleResolver::from_seed_graph(&pre_range_idx)?;
        assert!(
            pre_range_resolver
                .resolve_rule(&CharTarget::Dc(405))
                .is_some(),
            "Historical Dc 405 rule with legacy subtype encoding should decode at l8589958931"
        );
        assert!(
            pre_range_resolver
                .resolve_named_type("asciiprintable")
                .is_some(),
            "Historical asciiprintable construct with legacy Dc 4 range should decode at l8589958931"
        );

        Ok(())
    }

    #[crate::ctb_test]
    fn test_gate2_trusted_system_supervisor_resolution_and_bundled_provenance()
    -> Result<()> {
        let base_id = SYSTEM_REGION_START;
        let entry_dc = DcChar::from_short(527);
        let helper_dc = DcChar::from_short(528);
        let sh_entry = sh(entry_dc);
        let sh_helper = sh(helper_dc);

        // Helper WASM routine node `l(base_id)`
        let helper_def_id = base_id;
        let helper_tokens = vec![
            sh(DC_BEGIN_WASM_PROGRAM),
            DcShorthand::Short(712),
            sh(crate::dc::DC_BEGIN_NUMBER),
            DcShorthand::Format(199),
            sh(crate::dc::DC_END_NUMBER),
            DcShorthand::Short(1928),
        ];
        // Helper statements `584 528 611` (`l(base_id + 1)`) and `610 l(base_id + 1) l(base_id)` (`l(base_id + 2)`)
        let helper_has_a_id = base_id + 1;
        let helper_has_val_id = base_id + 2;

        // Supervisor revision 1 (`l(base_id + 3)`), superseded by revision 2 (`l(base_id + 6)`)
        let sup_v1_def_id = base_id + 3;
        let sup_v1_has_a_id = base_id + 4;
        let sup_v1_has_val_id = base_id + 5;

        // Supervisor revision 2 (`l(base_id + 6)`) referencing `helper_dc` (`528`) and `l(helper_def_id)`
        let sup_v2_def_id = base_id + 6;
        let sup_v2_tokens = vec![
            sh(DC_BEGIN_WASM_PROGRAM),
            sh_helper,
            DcShorthand::Long(helper_def_id),
            DcShorthand::Short(1928),
        ];
        let sup_v2_has_a_id = base_id + 7;
        let sup_v2_has_val_id = base_id + 8;
        let sup_stmt_id = base_id + 9; // `591 l(sup_v2_has_a_id) l(sup_v1_has_a_id)`

        // Bundled `Dc 635` provenance (`Dc 636`) covering all bundled nodes `base_id..=sup_stmt_id`
        let prov_has_a_id = base_id + 10;
        let prov_has_val_id = base_id + 11;
        let mut prov_for_each_tokens = vec![
            sh(crate::dc::DC_FOR_EACH),
            sh(DC_HAS_A),
            sh(DC_THIS_ELEMENT),
            sh(DC_COLLECTIVE_TOOLBOX_SEED_GRAPH_DATA_SOURCE),
            sh(DC_BEGIN_LIST),
            sh(crate::dc::DC_TYPE_ATOM_DC),
        ];
        for id in base_id..=sup_stmt_id {
            if id > base_id {
                prov_for_each_tokens.push(DcShorthand::Short(312));
            }
            prov_for_each_tokens.push(DcShorthand::Long(id));
        }
        prov_for_each_tokens.push(sh(DC_END_LIST));

        let bundled_nodes = vec![
            LedgerNode {
                id: helper_def_id,
                tokens: helper_tokens.clone(),
                comment: String::new(),
            },
            LedgerNode {
                id: helper_has_a_id,
                tokens: encode_statement_tokens(
                    sh(DC_HAS_A),
                    sh_helper,
                    sh(DC_CANONICAL_DECOMPOSITION),
                ),
                comment: String::new(),
            },
            LedgerNode {
                id: helper_has_val_id,
                tokens: encode_statement_tokens(
                    sh(DC_HAS_VALUE),
                    DcShorthand::Long(helper_has_a_id),
                    DcShorthand::Long(helper_def_id),
                ),
                comment: String::new(),
            },
            LedgerNode {
                id: sup_v1_def_id,
                tokens: helper_tokens.clone(),
                comment: String::new(),
            },
            LedgerNode {
                id: sup_v1_has_a_id,
                tokens: encode_statement_tokens(
                    sh(DC_HAS_A),
                    sh_entry,
                    sh(DC_CANONICAL_DECOMPOSITION),
                ),
                comment: String::new(),
            },
            LedgerNode {
                id: sup_v1_has_val_id,
                tokens: encode_statement_tokens(
                    sh(DC_HAS_VALUE),
                    DcShorthand::Long(sup_v1_has_a_id),
                    DcShorthand::Long(sup_v1_def_id),
                ),
                comment: String::new(),
            },
            LedgerNode {
                id: sup_v2_def_id,
                tokens: sup_v2_tokens.clone(),
                comment: String::new(),
            },
            LedgerNode {
                id: sup_v2_has_a_id,
                tokens: encode_statement_tokens(
                    sh(DC_HAS_A),
                    sh_entry,
                    sh(DC_CANONICAL_DECOMPOSITION),
                ),
                comment: String::new(),
            },
            LedgerNode {
                id: sup_v2_has_val_id,
                tokens: encode_statement_tokens(
                    sh(DC_HAS_VALUE),
                    DcShorthand::Long(sup_v2_has_a_id),
                    DcShorthand::Long(sup_v2_def_id),
                ),
                comment: String::new(),
            },
            LedgerNode {
                id: sup_stmt_id,
                tokens: encode_statement_tokens(
                    sh(DC_SUPERSEDES),
                    DcShorthand::Long(sup_v2_has_a_id),
                    DcShorthand::Long(sup_v1_has_a_id),
                ),
                comment: String::new(),
            },
            LedgerNode {
                id: prov_has_a_id,
                tokens: prov_for_each_tokens,
                comment: String::new(),
            },
            LedgerNode {
                id: prov_has_val_id,
                tokens: encode_statement_tokens(
                    sh(DC_HAS_VALUE),
                    DcShorthand::Long(prov_has_a_id),
                    sh(DC_COLLECTIVE_TOOLBOX_DC_CSV_SEED_DATABASE),
                ),
                comment: String::new(),
            },
        ];

        let trusted_idx =
            SeedGraphIndex::from_bundled_nodes(bundled_nodes.clone());
        let resolved = trusted_idx
            .resolve_trusted_system_supervisor_bootstrap_tokens(&[sh_entry])?;
        assert_eq!(resolved.entry_dc, entry_dc);
        assert_eq!(resolved.statement_node_id, sup_v2_has_a_id);
        assert_eq!(resolved.value_statement_node_id, sup_v2_has_val_id);
        assert_eq!(resolved.supervisor_revision_node_id, sup_v2_def_id);
        assert_eq!(resolved.system_head_node_id, prov_has_val_id);
        assert_eq!(
            resolved.dependency_snapshot_node_ids,
            vec![helper_def_id, sup_v2_def_id]
        );
        assert_eq!(resolved.definition_tokens, sup_v2_tokens);

        // 1. Reject `Dc 641` pinned or multi-token bootstrap documents.
        let pinned_err = trusted_idx
            .resolve_trusted_system_supervisor_bootstrap_tokens(&[
                sh(DC_GLOBAL_GRAPH_SYSTEM_BLOCK_MOST_RECENT_NODE_ID),
                DcShorthand::Long(prov_has_val_id),
                sh_entry,
            ])
            .expect_err("Pinned bootstrap document must be rejected");
        assert!(pinned_err.to_string().contains("unpinned"));
        assert!(
            trusted_idx
                .resolve_trusted_system_supervisor_bootstrap_tokens(&[
                    sh_entry, sh_helper
                ])
                .is_err()
        );
        assert!(
            trusted_idx
                .resolve_trusted_system_supervisor_bootstrap_tokens(&[
                    DcShorthand::Long(sup_v2_def_id)
                ])
                .is_err()
        );

        // 2. Reject unbundled nodes even if they have System-block IDs and forged `Dc 635 = Dc 636` claims.
        let unbundled_idx = SeedGraphIndex::from_nodes(bundled_nodes.clone());
        let unbundled_err = unbundled_idx
            .resolve_trusted_system_supervisor_dc(entry_dc)
            .expect_err("Unbundled nodes must not supply bootstrap authority");
        assert!(
            format!("{unbundled_err:#}")
                .contains("authenticated bundled System asset"),
            "Unexpected error: {unbundled_err:#}"
        );

        // 3. Reject extra/unbundled nodes attempting to supersede (`Dc 591`) the bundled supervisor definition.
        let extra_def_id = base_id + 20;
        let extra_has_a_id = base_id + 21;
        let extra_has_val_id = base_id + 22;
        let extra_sup_id = base_id + 23;
        let extra_prov_a = base_id + 24;
        let extra_prov_v = base_id + 25;
        let extra_nodes = vec![
            LedgerNode {
                id: extra_def_id,
                tokens: helper_tokens.clone(),
                comment: String::new(),
            },
            LedgerNode {
                id: extra_has_a_id,
                tokens: encode_statement_tokens(
                    sh(DC_HAS_A),
                    sh_entry,
                    sh(DC_CANONICAL_DECOMPOSITION),
                ),
                comment: String::new(),
            },
            LedgerNode {
                id: extra_has_val_id,
                tokens: encode_statement_tokens(
                    sh(DC_HAS_VALUE),
                    DcShorthand::Long(extra_has_a_id),
                    DcShorthand::Long(extra_def_id),
                ),
                comment: String::new(),
            },
            LedgerNode {
                id: extra_sup_id,
                tokens: encode_statement_tokens(
                    sh(DC_SUPERSEDES),
                    DcShorthand::Long(extra_has_a_id),
                    DcShorthand::Long(sup_v2_has_a_id),
                ),
                comment: String::new(),
            },
            LedgerNode {
                id: extra_prov_a,
                tokens: encode_statement_tokens(
                    sh(DC_HAS_A),
                    DcShorthand::Long(extra_sup_id),
                    sh(DC_COLLECTIVE_TOOLBOX_SEED_GRAPH_DATA_SOURCE),
                ),
                comment: String::new(),
            },
            LedgerNode {
                id: extra_prov_v,
                tokens: encode_statement_tokens(
                    sh(DC_HAS_VALUE),
                    DcShorthand::Long(extra_prov_a),
                    sh(DC_COLLECTIVE_TOOLBOX_DC_CSV_SEED_DATABASE),
                ),
                comment: String::new(),
            },
        ];
        let mixed_idx = SeedGraphIndex::from_bundled_and_extra_nodes(
            bundled_nodes.clone(),
            extra_nodes,
        )?;
        let poison_err = mixed_idx
            .resolve_trusted_system_supervisor_dc(entry_dc)
            .expect_err("Extra unbundled supersession must fail provenance check");
        assert!(
            format!("{poison_err:#}")
                .contains("authenticated bundled System asset"),
            "Unexpected error: {poison_err:#}"
        );

        // 4. Reject bundled nodes whose `Dc 635` provenance is an untrusted source (e.g. Wikidata / Format(999)).
        let mut untrusted_source_nodes = bundled_nodes;
        untrusted_source_nodes.last_mut().unwrap().tokens =
            encode_statement_tokens(
                sh(DC_HAS_VALUE),
                DcShorthand::Long(prov_has_a_id),
                DcShorthand::Format(999),
            );
        let untrusted_idx =
            SeedGraphIndex::from_bundled_nodes(untrusted_source_nodes);
        let untrusted_err = untrusted_idx
            .resolve_trusted_system_supervisor_dc(entry_dc)
            .expect_err("Untrusted Dc 635 data source must be rejected");
        assert!(
            format!("{untrusted_err:#}")
                .contains("untrusted Dc 635 data source"),
            "Unexpected error: {untrusted_err:#}"
        );

        Ok(())
    }
}
