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

use crate::dc::{
    DC_ANNOTATION, DC_APPROXIMATE_DECOMPOSITION, DC_BEGIN_ASSOCIATIVITY_GROUP,
    DC_BIDIRECTIONAL_CLASS, DC_BOOLEAN_TRUE, DC_CANONICAL_COMBINING_CLASS,
    DC_CANONICAL_DECOMPOSITION, DC_CANONICAL_NAME, DC_CASE_MAPPING_PARTNER,
    DC_CHARACTER_GENERAL_CATEGORY, DC_CHARACTER_SCRIPT,
    DC_COMPATIBILITY_DECOMPOSITION, DC_CROSS_REFERENCE, DC_DC_GENERAL_CATEGORY,
    DC_DESCRIPTION, DC_ENCLOSING_DOCUMENT, DC_END_ASSOCIATIVITY_GROUP,
    DC_ENTITY_TYPE, DC_FORMAL_ALIAS, DC_HAS_A, DC_HAS_VALUE, DC_INFORMAL_ALIAS,
    DC_IS_ASSOCIATED_WITH, DC_IS_BASED_ON, DC_IS_CLOSING_DELIMITER_FOR,
    DC_IS_DEPRECATED, DC_IS_INSTANCE_OF, DC_IS_SUBTYPE_OF, DC_MAGIC_SIGNATURE,
    DC_NEGATED_PATTERN, DC_PATTERN_ALTERNATION, DC_PATTERN_CAPTURE,
    DC_PATTERN_REPLACEMENT, DC_PATTERN_REPETITION, DC_PREFERRED,
    DC_PROGRAMMATIC_IDENTIFIER, DC_RANGE, DC_REFERENCE_NAMED_OBJECT,
    DC_REPLACEMENT, DC_RETRACTS, DC_ROUTINE_INVOCATION,
    DC_SEMANTIC_DECOMPOSITION, DC_SELF_REFERENCE, DC_SUBPATTERN_MATCH,
    DC_SUPERSEDES, DC_SYNTAX_RULE, DC_UNICODE_SCRIPT, DcChar,
};
use crate::dc_def::DcDefn;
use crate::graph_ledger::{
    LedgerNode, SeedEntityKey, decode_integer_value_tokens,
    decode_nodes_from_ctbnbdl_bytes, decode_string_value_tokens,
    format_chunk_bundle_filename, parse_chunk_bundle_filename_start_id,
    parse_statement_tokens, single_valued_property_set,
};
use crate::shared::{BidiClass, GeneralCategory};
use crate::syntax::ast::{
    ActionArg, CharTarget, DcSyntaxRule, Quantifier, SyntaxAction,
    SyntaxElement, SyntaxPattern, SyntaxTerm,
};

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
    /// Active multi-valued properties: `(subject, prop_token, val_token) -> (has_a_id, has_val_id)`.
    pub(crate) active_multi_props:
        BTreeMap<(DcShorthand, DcShorthand, DcShorthand), (u128, u128)>,
    /// Active multi-valued properties grouped by `(subject, prop_token)` in ascending `has_a_id` order.
    pub(crate) multi_props_by_subj_prop:
        HashMap<(DcShorthand, DcShorthand), Vec<ActivePropertyValue>>,
    /// Active `<prefer>` replacements: `(subject, val_token) -> (has_a_id, has_val_id, pref_qual_id)`.
    pub(crate) active_preferred:
        BTreeMap<(DcShorthand, DcShorthand), (u128, u128, u128)>,
    /// Active `<prefer>` replacement values grouped by `subject` in ascending `has_a_id` order.
    pub(crate) preferred_by_subject: HashMap<DcShorthand, Vec<DcShorthand>>,
    /// Active direct binary statements: `(verb, subject, object) -> stmt_id`.
    pub(crate) active_direct_stmts:
        BTreeMap<(DcShorthand, DcShorthand, DcShorthand), u128>,
    /// Active direct binary statement objects grouped by `(verb, subject)` in ascending `stmt_id` order.
    pub(crate) direct_objects_by_verb_subj:
        HashMap<(DcShorthand, DcShorthand), Vec<DcShorthand>>,
    /// Active direct binary statement subjects grouped by `(verb, object)` in ascending `stmt_id` order.
    pub(crate) direct_subjects_by_verb_obj:
        HashMap<(DcShorthand, DcShorthand), Vec<DcShorthand>>,
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
        let active_nodes = Self::load_nodes_from_asset_dir(
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
        Ok(Self::from_nodes(active_nodes))
    }

    /// Loads both active `data/graph/*.ctbnbdl` bundles (from `ctoolbox.rsrc`) and optional
    /// historical/inactive `data/graph/system/*.ctbnbdl` bundles (from `system_graph_history.rsrc`
    /// if installed, or `assets/data/graph/system/` in dev mode), merging them in ascending
    /// node ID order and resolving all `Dc 591` (`Supersedes`) and `Dc 592` (`Retracts`) history.
    pub fn load_with_history_from_asset_bundle() -> Result<Self> {
        let active_nodes = Self::load_nodes_from_asset_dir(
            SEED_GRAPH_ASSET_GLOB,
            "data/graph",
            Some(PRIMARY_SEED_GRAPH_ASSET_KEY),
            true,
        )?;
        let history_nodes = Self::load_nodes_from_asset_dir(
            SYSTEM_GRAPH_HISTORY_ASSET_GLOB,
            "data/graph/system",
            None,
            false,
        )?;
        if history_nodes.is_empty() {
            return Ok(Self::from_nodes(active_nodes));
        }

        let mut merged =
            Vec::with_capacity(active_nodes.len().saturating_add(history_nodes.len()));
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

        Ok(Self::from_nodes(merged))
    }

    fn load_nodes_from_asset_dir(
        glob_pattern: &str,
        expected_parent_dir: &str,
        fallback_key: Option<&str>,
        require_non_empty: bool,
    ) -> Result<Vec<LedgerNode>> {
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
            return Ok(Vec::new());
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
        for (expected_first_id, asset_path) in keyed_paths {
            let bytes = ctb_storage_asset_bundle::get_asset(&asset_path)
                .with_context(|| {
                    format!(
                        "Failed to read asset '{asset_path}' from runtime asset bundle"
                    )
                })?;
            let bundle_nodes = decode_nodes_from_ctbnbdl_bytes(&bytes)
                .with_context(|| {
                    format!("Failed to decode .ctbnbdl bundle '{asset_path}'")
                })?;
            let Some(first_node) = bundle_nodes.first() else {
                bail!("Decoded .ctbnbdl bundle '{asset_path}' contains 0 nodes");
            };
            ensure!(
                first_node.id == expected_first_id,
                "Bundle '{asset_path}' filename declares start ID l{expected_first_id}, but first decoded node is l{}",
                first_node.id
            );
            for (idx, node) in bundle_nodes.into_iter().enumerate() {
                ensure!(
                    (SYSTEM_REGION_START..=SYSTEM_REGION_END).contains(&node.id),
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

        Ok(all_nodes)
    }

    /// Decodes a single `.ctbnbdl` byte slice and builds a `SeedGraphIndex`.
    pub fn from_ctbnbdl_bytes(bytes: &[u8]) -> Result<Self> {
        let nodes = decode_nodes_from_ctbnbdl_bytes(bytes)?;
        Ok(Self::from_nodes(nodes))
    }

    /// Builds a `SeedGraphIndex` from an ordered list of `LedgerNode`s using the default
    /// single-valued property set.
    #[must_use]
    pub fn from_nodes(nodes: Vec<LedgerNode>) -> Self {
        let single_props = single_valued_property_set();
        Self::from_nodes_with_single_props(nodes, &single_props)
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

        let mut node_index_by_id = HashMap::with_capacity(nodes.len());
        for (idx, node) in nodes.iter().enumerate() {
            node_index_by_id.insert(node.id, idx);
        }

        // Resolve supersessions (`591 <new_node> l<old_node>`) and retractions (`592 615 l<old_node>`).
        // Process in reverse chronological order (highest node ID to lowest) so that if a
        // retraction or supersession statement was itself retracted later, it does not take effect.
        let mut inactive_ids = HashSet::new();
        let mut superseded_by = HashMap::new();
        let mut retracted_by = HashMap::new();

        for node in nodes.iter().rev() {
            if inactive_ids.contains(&node.id) {
                continue;
            }
            if let Some((verb, subj, obj)) =
                parse_statement_tokens(&node.tokens)
            {
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

        // First pass: index value nodes and find active `610` (`Has value`) and `584 <s1> 570` (`Preferred`) nodes.
        let mut value_node_by_tokens = HashMap::new();
        let mut string_by_node_id: HashMap<u128, String> = HashMap::new();
        let mut integer_by_node_id: HashMap<u128, u128> = HashMap::new();
        let mut active_has_val_by_stmt: HashMap<u128, (u128, DcShorthand)> =
            HashMap::new();
        let mut active_pref_qual_by_stmt: HashMap<u128, u128> = HashMap::new();

        for node in &nodes {
            if let Some(decoded_str) = decode_string_value_tokens(&node.tokens)
            {
                string_by_node_id.insert(node.id, decoded_str);
            } else if let Some(decoded_int) =
                decode_integer_value_tokens(&node.tokens)
            {
                integer_by_node_id.insert(node.id, decoded_int);
            }
            let stmt_opt = parse_statement_tokens(&node.tokens);
            if stmt_opt.is_none() {
                value_node_by_tokens
                    .entry(node.tokens.clone())
                    .or_insert(node.id);
            }
            if inactive_ids.contains(&node.id) {
                continue;
            }
            if let Some((verb, subj, obj)) = stmt_opt {
                if verb == sh_has_value {
                    if let DcShorthand::Long(has_a_id) = subj {
                        if !inactive_ids.contains(&has_a_id) {
                            active_has_val_by_stmt
                                .insert(has_a_id, (node.id, obj));
                        } else {
                            inactive_ids.insert(node.id);
                        }
                    }
                } else if verb == sh_has_a && obj == sh_preferred {
                    if let DcShorthand::Long(has_a_id) = subj {
                        if !inactive_ids.contains(&has_a_id) {
                            active_pref_qual_by_stmt.insert(has_a_id, node.id);
                        } else {
                            inactive_ids.insert(node.id);
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
        let mut active_multi_props = BTreeMap::new();
        let mut multi_props_by_subj_prop: HashMap<
            (DcShorthand, DcShorthand),
            Vec<ActivePropertyValue>,
        > = HashMap::new();
        let mut active_preferred = BTreeMap::new();
        let mut preferred_by_subject: HashMap<DcShorthand, Vec<DcShorthand>> =
            HashMap::new();
        let mut active_direct_stmts = BTreeMap::new();
        let mut direct_objects_by_verb_subj: HashMap<
            (DcShorthand, DcShorthand),
            Vec<DcShorthand>,
        > = HashMap::new();
        let mut direct_subjects_by_verb_obj: HashMap<
            (DcShorthand, DcShorthand),
            Vec<DcShorthand>,
        > = HashMap::new();

        for node in &nodes {
            if inactive_ids.contains(&node.id) {
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
                            parse_statement_tokens(&n.tokens).is_some_and(
                                |(v, _, o)| {
                                    v == sh_has_a && o == sh_replacement
                                },
                            )
                        })
                    {
                        continue;
                    }
                }
            }

            if verb == sh_has_a {
                if let Some(&(has_val_id, val_tok)) =
                    active_has_val_by_stmt.get(&node.id)
                {
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
                                Some(SeedEntityKey::BidiClass(label.clone()))
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
                                seed_entity_by_id
                                    .insert(seed_id, seed_key.clone());
                                seed_entity_ids.insert(seed_key, seed_id);
                            }
                        }
                    }

                    if obj == sh_replacement {
                        if let Some(&pref_id) =
                            active_pref_qual_by_stmt.get(&node.id)
                        {
                            active_preferred.insert(
                                (subj, val_tok),
                                (node.id, has_val_id, pref_id),
                            );
                            preferred_by_subject
                                .entry(subj)
                                .or_default()
                                .push(val_tok);
                            continue;
                        }
                    }

                    if single_valued_props.contains(&obj) {
                        active_single_props.insert(
                            (subj, obj),
                            (node.id, has_val_id, val_tok),
                        );
                    } else {
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
                            });
                    }
                }
            } else {
                active_direct_stmts.insert((verb, subj, obj), node.id);
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
            active_multi_props,
            multi_props_by_subj_prop,
            active_preferred,
            preferred_by_subject,
            active_direct_stmts,
            direct_objects_by_verb_subj,
            direct_subjects_by_verb_obj,
        }
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

    /// Resolves `tok` as a string value node (`DcShorthand::Long(id)`), returning its decoded UTF-8 string.
    #[must_use]
    pub fn get_string(&self, tok: DcShorthand) -> Option<&str> {
        let DcShorthand::Long(id) = tok else {
            return None;
        };
        self.string_by_node_id.get(&id).map(String::as_str)
    }

    /// Resolves `tok` as an integer value node (`DcShorthand::Long(id)`), returning its decoded `u128`.
    #[must_use]
    pub fn get_integer(&self, tok: DcShorthand) -> Option<u128> {
        let DcShorthand::Long(id) = tok else {
            return None;
        };
        self.integer_by_node_id.get(&id).copied()
    }

    /// Looks up the allocated `u128` node ID for a `SeedEntityKey`.
    #[must_use]
    pub fn get_seed_entity_id(&self, key: &SeedEntityKey) -> Option<u128> {
        self.seed_entity_ids.get(key).copied()
    }

    /// Looks up the `SeedEntityKey` for a seed entity node ID (`u128`).
    #[must_use]
    pub fn get_seed_entity(&self, id: u128) -> Option<&SeedEntityKey> {
        self.seed_entity_by_id.get(&id)
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

    /// Reconstructs all Document Character definitions (`DcDefn` for Unicode clarifications and Short Dcs)
    /// directly from the indexed seed graph, sorted in ascending `dc_id` order.
    pub fn reconstruct_dc_defns(&self) -> Result<Vec<DcDefn>> {
        let sh_name = sh(DC_CANONICAL_NAME);
        let sh_desc = sh(DC_DESCRIPTION);
        let sh_ident = sh(DC_PROGRAMMATIC_IDENTIFIER);
        let sh_cc = sh(DC_CANONICAL_COMBINING_CLASS);
        let sh_bidi = sh(DC_BIDIRECTIONAL_CLASS);
        let sh_inst = sh(DC_IS_INSTANCE_OF);
        let sh_case = sh(DC_CASE_MAPPING_PARTNER);
        let sh_dep = sh(DC_IS_DEPRECATED);
        let sh_true = sh(DC_BOOLEAN_TRUE);
        let sh_closes = sh(DC_IS_CLOSING_DELIMITER_FOR);
        let sh_subtype = sh(DC_IS_SUBTYPE_OF);
        let sh_based = sh(DC_IS_BASED_ON);
        let sh_assoc = sh(DC_IS_ASSOCIATED_WITH);
        let sh_alias = sh(DC_INFORMAL_ALIAS);
        let sh_falias = sh(DC_FORMAL_ALIAS);
        let sh_ann = sh(DC_ANNOTATION);
        let sh_magic = sh(DC_MAGIC_SIGNATURE);
        let sh_xref = sh(DC_CROSS_REFERENCE);
        let sh_equiv = sh(DC_CANONICAL_DECOMPOSITION);
        let sh_approx = sh(DC_APPROXIMATE_DECOMPOSITION);
        let sh_sem = sh(DC_SEMANTIC_DECOMPOSITION);
        let sh_compat = sh(DC_COMPATIBILITY_DECOMPOSITION);

        let mut subjects: Vec<DcShorthand> = self
            .active_single_props
            .keys()
            .filter_map(|&(subj, prop)| {
                if prop == sh_name
                    && matches!(
                        subj,
                        DcShorthand::Unicode(_) | DcShorthand::Short(_)
                    )
                {
                    Some(subj)
                } else {
                    None
                }
            })
            .collect();
        subjects.sort_by_key(|s| {
            // Reason for fallback: Unicode and Short shorthands always have valid global IDs.
            s.to_global_id().unwrap_or(0)
        });
        subjects.dedup();

        let mut defns = Vec::with_capacity(subjects.len());
        for subj in subjects {
            let dc_id = subj.to_global_id()?;
            let short_id = match subj {
                DcShorthand::Short(sid) => usize::try_from(sid).ok(),
                _ => None,
            };
            // Reason for fallback: subjects were selected from active_single_props by sh_name.
            let name = self
                .get_single_property_string(subj, sh_name)
                .unwrap_or_default()
                .to_string();
            // Reason for fallback: characters without an explicit description default to empty string.
            let description = self
                .get_single_property_string(subj, sh_desc)
                .unwrap_or_default()
                .to_string();
            let ident = self
                .get_single_property_string(subj, sh_ident)
                .map(str::to_string);
            // Reason for fallback: characters without a combining class statement have combining class 0.
            let combining_class = self
                .get_single_property_integer(subj, sh_cc)
                .and_then(|v| u8::try_from(v).ok())
                .unwrap_or(0);

            let bidi_class = if let Some(DcShorthand::Long(seed_id)) =
                self.get_single_property(subj, sh_bidi)
            {
                if let Some(SeedEntityKey::BidiClass(code)) =
                    self.get_seed_entity(seed_id)
                {
                    code.parse::<BidiClass>()?
                } else {
                    BidiClass::BN
                }
            } else {
                BidiClass::BN
            };

            let mut general_category = GeneralCategory::NonUnicodeControl;
            let mut script = String::new();
            for &obj in self.get_direct_objects(sh_inst, subj) {
                if let DcShorthand::Long(seed_id) = obj {
                    match self.get_seed_entity(seed_id) {
                        Some(SeedEntityKey::GeneralCategory(code)) => {
                            general_category =
                                code.parse::<GeneralCategory>()?;
                        }
                        Some(SeedEntityKey::Script(s)) => {
                            script.clone_from(s);
                        }
                        _ => {}
                    }
                }
            }

            let category = category_for_dc_subject(subj, &script);
            let casing_partner = match self.get_single_property(subj, sh_case) {
                Some(DcShorthand::Short(partner)) => Some(partner),
                _ => None,
            };
            let is_deprecated =
                self.has_direct_statement(sh_dep, subj, sh_true);
            let closes =
                self.get_direct_objects(sh_closes, subj).first().copied();
            let syntax = self.decode_syntax_rule_for_subject(subj)?;

            let aliases: Vec<String> = self
                .get_multi_properties(subj, sh_alias)
                .iter()
                .filter_map(|e| self.get_string(e.value).map(str::to_string))
                .collect();
            let formal_aliases: Vec<(String, String)> = self
                .get_multi_properties(subj, sh_falias)
                .iter()
                .filter_map(|e| {
                    self.get_string(e.value)
                        .map(|s| (String::new(), s.to_string()))
                })
                .collect();
            let annotations: Vec<String> = self
                .get_multi_properties(subj, sh_ann)
                .iter()
                .filter_map(|e| self.get_string(e.value).map(str::to_string))
                .collect();
            let magic_names: Vec<String> = self
                .get_multi_properties(subj, sh_magic)
                .iter()
                .filter_map(|e| self.get_string(e.value).map(str::to_string))
                .collect();
            let cross_references: Vec<String> = self
                .get_multi_properties(subj, sh_xref)
                .iter()
                .map(|e| {
                    if let Some(s) = self.get_string(e.value) {
                        s.to_string()
                    } else {
                        e.value.to_string()
                    }
                })
                .collect();

            let mut decompositions = Vec::new();
            for (decomp_prop, tag) in [
                (sh_equiv, "equiv"),
                (sh_approx, "approx"),
                (sh_sem, "semantic"),
                (sh_compat, "compat"),
            ] {
                for entry in self.get_multi_properties(subj, decomp_prop) {
                    let payload = self.decode_decomposition_payload(entry.value);
                    decompositions.push(format!("<{tag}>{payload}"));
                }
            }
            for &pref_val in self.get_preferred_replacements(subj) {
                let payload = self.decode_decomposition_payload(pref_val);
                decompositions.push(format!("<prefer>{payload}"));
            }

            let implies: Vec<String> = self
                .get_direct_objects(sh_subtype, subj)
                .iter()
                .map(|&obj| self.decode_relation_target_string(obj))
                .collect();
            let based_on: Vec<String> = self
                .get_direct_objects(sh_based, subj)
                .iter()
                .map(|&obj| self.decode_relation_target_string(obj))
                .collect();
            let os_associations: Vec<String> = self
                .get_direct_objects(sh_assoc, subj)
                .iter()
                .map(ToString::to_string)
                .collect();

            defns.push(DcDefn {
                dc_id,
                short_id,
                ident,
                role_def: false,
                title: None,
                name,
                category: category.clone(),
                combining_class,
                bidi_class,
                casing_partner,
                general_category,
                script,
                is_deprecated,
                decompositions,
                aliases,
                formal_aliases,
                cross_references,
                annotations,
                implies,
                based_on,
                os_associations,
                magic_names,
                syntax,
                closes,
                description,
                format: None,
                source_file: format!("{category}.csv"),
                line_number: 0,
            });
        }

        Ok(defns)
    }

    fn decode_decomposition_payload(&self, val_tok: DcShorthand) -> String {
        if let Some(s) = self.get_string(val_tok) {
            return s.to_string();
        }
        if let DcShorthand::Long(node_id) = val_tok {
            if let Some(node) = self.get_node(node_id) {
                let parts: Vec<String> =
                    node.tokens.iter().map(ToString::to_string).collect();
                return parts.join(" ");
            }
        }
        val_tok.to_string()
    }

    fn decode_relation_target_string(&self, obj: DcShorthand) -> String {
        if let Some(s) = self.get_string(obj) {
            return s.to_string();
        }
        if let DcShorthand::Long(node_id) = obj {
            if let Ok(rule) = self.decode_pattern_node(node_id) {
                return rule.pattern.format_dsl();
            }
        }
        obj.to_string()
    }

    /// Decodes a `.Semantic:Pattern` value node by its `node_id` into a `DcSyntaxRule`.
    pub fn decode_pattern_node(&self, node_id: u128) -> Result<DcSyntaxRule> {
        let node = self.get_node(node_id).with_context(|| {
            format!("Pattern value node l{node_id} not found in seed graph")
        })?;
        self.decode_syntax_rule_tokens(&node.tokens)
            .with_context(|| {
                format!(
                    "Failed to decode .Semantic:Pattern tokens in node l{node_id}"
                )
            })
    }

    /// Resolves and decodes the active `Dc 594` (`Syntax rule`) property for `subj`, if present.
    pub fn decode_syntax_rule_for_subject(
        &self,
        subj: DcShorthand,
    ) -> Result<Option<DcSyntaxRule>> {
        let Some(val_tok) =
            self.get_single_property(subj, sh(DC_SYNTAX_RULE))
        else {
            return Ok(None);
        };
        let DcShorthand::Long(node_id) = val_tok else {
            bail!(
                "Expected Long node ID for Syntax rule (Dc 594) on {subj}, found {val_tok}"
            );
        };
        self.decode_pattern_node(node_id).map(Some)
    }

    /// Resolves and decodes the active `SyntaxPattern` for a named syntax construct
    /// (e.g. `"number"`, `"string"`, `"value"`), if present in the seed graph.
    pub fn decode_named_construct_pattern(
        &self,
        name: &str,
    ) -> Result<Option<SyntaxPattern>> {
        let key = SeedEntityKey::NamedConstruct(name.to_string());
        let Some(seed_id) = self.get_seed_entity_id(&key) else {
            return Ok(None);
        };
        let Some(rule) =
            self.decode_syntax_rule_for_subject(DcShorthand::Long(seed_id))?
        else {
            return Ok(None);
        };
        Ok(Some(rule.pattern))
    }

    /// Decodes a `.Semantic:Pattern` token sequence into a `DcSyntaxRule`.
    pub fn decode_syntax_rule_tokens(
        &self,
        tokens: &[DcShorthand],
    ) -> Result<DcSyntaxRule> {
        ensure!(
            !tokens.is_empty(),
            "Cannot decode empty .Semantic:Pattern token sequence"
        );
        let (pattern_tokens, action) =
            self.split_trailing_syntax_action(tokens)?;
        let pattern = self.decode_syntax_pattern_tokens(pattern_tokens)?;
        let raw = DcSyntaxRule::format_dsl_parts(&pattern, action.as_ref());
        Ok(DcSyntaxRule {
            pattern,
            action,
            raw,
        })
    }

    fn split_trailing_syntax_action<'a>(
        &self,
        tokens: &'a [DcShorthand],
    ) -> Result<(&'a [DcShorthand], Option<SyntaxAction>)> {
        let sh_open = sh(DC_BEGIN_ASSOCIATIVITY_GROUP);
        let sh_close = sh(DC_END_ASSOCIATIVITY_GROUP);
        let sh_repl = sh(DC_PATTERN_REPLACEMENT);
        let sh_invoke = sh(DC_ROUTINE_INVOCATION);
        let sh_ref_var = sh(DC_REFERENCE_NAMED_OBJECT);

        if tokens.last() != Some(&sh_close) {
            return Ok((tokens, None));
        }
        // Find the matching `298` (`Begin associativity group`) for the final `299`.
        let mut depth = 0usize;
        let mut open_idx = None;
        for (idx, &tok) in tokens.iter().enumerate().rev() {
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
        let Some(start) = open_idx else {
            bail!("Unbalanced EndAssociativityGroup (299) in pattern token stream");
        };
        let group_slice = tokens
            .get(start..)
            .context("Invalid syntax action group token start index")?;
        if group_slice.get(1) != Some(&sh_repl) {
            return Ok((tokens, None));
        }
        ensure!(
            group_slice.get(2) == Some(&sh_invoke),
            "Expected RoutineInvocation (273) after PatternReplacement (607) in syntax action"
        );
        let method_tok = *group_slice
            .get(3)
            .context("Missing method string node in syntax action")?;
        let method = self
            .get_string(method_tok)
            .with_context(|| {
                format!(
                    "Syntax action method token {method_tok} is not a string value node"
                )
            })?
            .to_string();

        let mut args = Vec::new();
        let mut idx = 4usize;
        let end_exclusive = group_slice.len().saturating_sub(1);
        while idx < end_exclusive {
            let tok = *group_slice
                .get(idx)
                .context("Unexpected end of syntax action arguments")?;
            if tok == sh_ref_var {
                idx = idx.saturating_add(1);
                let var_tok = *group_slice.get(idx).context(
                    "Missing variable name token after ReferenceNamedObject (272)",
                )?;
                let var_name = self
                    .get_string(var_tok)
                    .with_context(|| {
                        format!(
                            "Action variable token {var_tok} is not a string value node"
                        )
                    })?
                    .to_string();
                args.push(ActionArg::Variable(var_name));
                idx = idx.saturating_add(1);
            } else {
                let lit_val = self
                    .get_string(tok)
                    .with_context(|| {
                        format!(
                            "Action literal token {tok} is not a string value node"
                        )
                    })?
                    .to_string();
                args.push(ActionArg::Literal(lit_val));
                idx = idx.saturating_add(1);
            }
        }

        let pattern_slice = tokens
            .get(..start)
            .context("Invalid token slice preceding syntax action")?;
        Ok((pattern_slice, Some(SyntaxAction { method, args })))
    }

    fn decode_syntax_pattern_tokens(
        &self,
        tokens: &[DcShorthand],
    ) -> Result<SyntaxPattern> {
        let sh_open = sh(DC_BEGIN_ASSOCIATIVITY_GROUP);
        let sh_alt = sh(DC_PATTERN_ALTERNATION);

        // Check if the entire `tokens` slice is a top-level `SyntaxPattern::Alternation`:
        // `298 602 (branch1) (branch2) ... 299`
        if tokens.first() == Some(&sh_open) && tokens.get(1) == Some(&sh_alt) {
            if let Some(close_idx) = find_matching_group_close(tokens, 0)? {
                if close_idx.saturating_add(1) == tokens.len() {
                    let inner = tokens
                        .get(2..close_idx)
                        .context("Invalid alternation group bounds")?;
                    if inner.first() == Some(&sh_open) {
                        let mut branches = Vec::new();
                        let mut idx = 0usize;
                        while idx < inner.len() {
                            ensure!(
                                inner.get(idx) == Some(&sh_open),
                                "Expected BeginAssociativityGroup (298) for alternation branch at index {idx}"
                            );
                            let branch_close =
                                find_matching_group_close(inner, idx)?
                                    .context(
                                        "Unclosed alternation branch group",
                                    )?;
                            let branch_tokens = inner
                                .get(idx.saturating_add(1)..branch_close)
                                .context(
                                    "Invalid alternation branch slice bounds",
                                )?;
                            branches.push(
                                self.decode_syntax_pattern_tokens(
                                    branch_tokens,
                                )?,
                            );
                            idx = branch_close.saturating_add(1);
                        }
                        return Ok(SyntaxPattern::Alternation(branches));
                    }
                }
            }
        }

        let mut elements = Vec::new();
        let mut idx = 0usize;
        while idx < tokens.len() {
            let (term, after_term) =
                self.decode_syntax_term_at(tokens, idx)?;
            let (quantifier, after_quant) =
                self.decode_optional_quantifier_at(tokens, after_term)?;
            elements.push(SyntaxElement::with_quantifier(term, quantifier));
            idx = after_quant;
        }
        Ok(SyntaxPattern::Sequence(elements))
    }

    fn decode_syntax_term_at(
        &self,
        tokens: &[DcShorthand],
        idx: usize,
    ) -> Result<(SyntaxTerm, usize)> {
        let sh_self = sh(DC_SELF_REFERENCE);
        let sh_range = sh(DC_RANGE);
        let sh_open = sh(DC_BEGIN_ASSOCIATIVITY_GROUP);
        let sh_neg = sh(DC_NEGATED_PATTERN);
        let sh_alt = sh(DC_PATTERN_ALTERNATION);
        let sh_sub = sh(DC_SUBPATTERN_MATCH);
        let sh_has_a = sh(DC_HAS_A);
        let sh_capture = sh(DC_PATTERN_CAPTURE);

        let tok = *tokens
            .get(idx)
            .context("Unexpected end of pattern token stream")?;

        if tok == sh_self {
            return Ok((SyntaxTerm::SelfChar, idx.saturating_add(1)));
        }

        if tok == sh_range {
            let start_sh = *tokens
                .get(idx.saturating_add(1))
                .context("Missing start token after Range (Dc 4)")?;
            let end_sh = *tokens
                .get(idx.saturating_add(2))
                .context("Missing end token after Range (Dc 4)")?;
            let start = shorthand_to_char_target(start_sh)?;
            let end = shorthand_to_char_target(end_sh)?;
            return Ok((
                SyntaxTerm::CharRange { start, end },
                idx.saturating_add(3),
            ));
        }

        if tok == sh_open {
            let close_idx = find_matching_group_close(tokens, idx)?
                .context("Unclosed BeginAssociativityGroup (298) in pattern")?;
            let inner = tokens
                .get(idx.saturating_add(1)..close_idx)
                .context("Invalid group token range")?;
            let next_idx = close_idx.saturating_add(1);

            let Some(&first_inner) = inner.first() else {
                return Ok((
                    SyntaxTerm::Group(SyntaxPattern::Sequence(Vec::new())),
                    next_idx,
                ));
            };

            if first_inner == sh_neg {
                let mut members =
                    Vec::with_capacity(inner.len().saturating_sub(1));
                let members_slice = inner
                    .get(1..)
                    .context("Invalid negated character set members range")?;
                for &m in members_slice {
                    members.push(shorthand_to_char_target(m)?);
                }
                return Ok((
                    SyntaxTerm::CharSet {
                        negated: true,
                        members,
                    },
                    next_idx,
                ));
            }

            if first_inner == sh_alt && inner.get(1) != Some(&sh_open) {
                let mut members =
                    Vec::with_capacity(inner.len().saturating_sub(1));
                let members_slice = inner
                    .get(1..)
                    .context("Invalid character set members range")?;
                for &m in members_slice {
                    members.push(shorthand_to_char_target(m)?);
                }
                return Ok((
                    SyntaxTerm::CharSet {
                        negated: false,
                        members,
                    },
                    next_idx,
                ));
            }

            if first_inner == sh_sub {
                let target_tok = *inner.get(1).context(
                    "Missing target token after SubpatternMatch (605)",
                )?;
                if let DcShorthand::Long(seed_id) = target_tok {
                    if let Some(SeedEntityKey::NamedConstruct(name)) =
                        self.get_seed_entity(seed_id)
                    {
                        let mut subtype = None;
                        let mut capture_var = None;
                        let mut pos = 2usize;
                        while pos < inner.len() {
                            let qual = *inner
                                .get(pos)
                                .context("Unexpected end of SubpatternMatch")?;
                            if qual == sh_has_a {
                                let sub_tok = *inner
                                    .get(pos.saturating_add(1))
                                    .context(
                                        "Missing subtype string node after HasA (584)",
                                    )?;
                                let sub_str = self
                                    .get_string(sub_tok)
                                    .with_context(|| {
                                        format!(
                                            "NamedConstruct subtype token {sub_tok} is not a string node"
                                        )
                                    })?
                                    .to_string();
                                subtype = Some(sub_str);
                                pos = pos.saturating_add(2);
                            } else if qual == sh_capture {
                                let var_tok = *inner
                                    .get(pos.saturating_add(1))
                                    .context(
                                        "Missing capture variable string node after PatternCapture (606)",
                                    )?;
                                let var_str = self
                                    .get_string(var_tok)
                                    .with_context(|| {
                                        format!(
                                            "NamedConstruct capture variable token {var_tok} is not a string node"
                                        )
                                    })?
                                    .to_string();
                                capture_var = Some(var_str);
                                pos = pos.saturating_add(2);
                            } else {
                                bail!(
                                    "Unexpected qualifier token {qual} in NamedConstruct SubpatternMatch"
                                );
                            }
                        }
                        return Ok((
                            SyntaxTerm::NamedConstruct {
                                name: name.clone(),
                                subtype,
                                capture_var,
                            },
                            next_idx,
                        ));
                    }
                }
                ensure!(
                    inner.len() == 2,
                    "Unexpected extra tokens in RuleRef SubpatternMatch: {inner:?}"
                );
                let target = shorthand_to_char_target(target_tok)?;
                return Ok((SyntaxTerm::RuleRef { target }, next_idx));
            }

            let group_pat = self.decode_syntax_pattern_tokens(inner)?;
            return Ok((SyntaxTerm::Group(group_pat), next_idx));
        }

        let target = shorthand_to_char_target(tok)?;
        Ok((SyntaxTerm::CharRef(target), idx.saturating_add(1)))
    }

    fn decode_optional_quantifier_at(
        &self,
        tokens: &[DcShorthand],
        idx: usize,
    ) -> Result<(Quantifier, usize)> {
        let sh_open = sh(DC_BEGIN_ASSOCIATIVITY_GROUP);
        let sh_rep = sh(DC_PATTERN_REPETITION);
        let sh_range = sh(DC_RANGE);

        if tokens.get(idx) != Some(&sh_open)
            || tokens.get(idx.saturating_add(1)) != Some(&sh_rep)
        {
            return Ok((Quantifier::ExactOne, idx));
        }

        let close_idx = find_matching_group_close(tokens, idx)?
            .context("Unclosed PatternRepetition (603) group")?;
        let inner = tokens
            .get(idx.saturating_add(1)..close_idx)
            .context("Invalid token slice for PatternRepetition (603) group")?;
        ensure!(
            inner.get(1) == Some(&sh_range),
            "Expected Range (Dc 4) after PatternRepetition (603)"
        );
        let min_tok = *inner
            .get(2)
            .context("Missing minimum bound in PatternRepetition (603)")?;
        let min_u128 = self.get_integer(min_tok).with_context(|| {
            format!(
                "PatternRepetition min bound token {min_tok} is not an integer value node"
            )
        })?;
        let min = usize::try_from(min_u128)
            .context("PatternRepetition min bound overflows usize")?;

        let max = if let Some(&max_tok) = inner.get(3) {
            ensure!(
                inner.len() == 4,
                "Unexpected extra tokens in PatternRepetition (603): {inner:?}"
            );
            let max_u128 = self.get_integer(max_tok).with_context(|| {
                format!(
                    "PatternRepetition max bound token {max_tok} is not an integer value node"
                )
            })?;
            Some(
                usize::try_from(max_u128)
                    .context("PatternRepetition max bound overflows usize")?,
            )
        } else {
            ensure!(
                inner.len() == 3,
                "Unexpected tokens in PatternRepetition (603): {inner:?}"
            );
            None
        };

        let quantifier = match (min, max) {
            (0, None) => Quantifier::ZeroOrMore,
            (1, None) => Quantifier::OneOrMore,
            (0, Some(1)) => Quantifier::Optional,
            _ => Quantifier::Range { min, max },
        };
        Ok((quantifier, close_idx.saturating_add(1)))
    }
}

fn find_matching_group_close(
    tokens: &[DcShorthand],
    open_idx: usize,
) -> Result<Option<usize>> {
    let sh_open = sh(DC_BEGIN_ASSOCIATIVITY_GROUP);
    let sh_close = sh(DC_END_ASSOCIATIVITY_GROUP);
    if tokens.get(open_idx) != Some(&sh_open) {
        return Ok(None);
    }
    let mut depth = 0usize;
    for (idx, &tok) in tokens.iter().enumerate().skip(open_idx) {
        if tok == sh_open {
            depth = depth.saturating_add(1);
        } else if tok == sh_close {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Ok(Some(idx));
            }
        }
    }
    bail!(
        "Unclosed BeginAssociativityGroup (298) starting at token index {open_idx}"
    )
}

fn shorthand_to_char_target(shorthand: DcShorthand) -> Result<CharTarget> {
    let gid = shorthand.to_global_id()?;
    CharTarget::from_u128(gid).with_context(|| {
        format!(
            "Token shorthand '{shorthand}' (GID {gid}) cannot be represented as a CharTarget"
        )
    })
}

fn category_for_dc_subject(subj: DcShorthand, script: &str) -> String {
    if matches!(subj, DcShorthand::Unicode(_)) {
        return "unicode-clarifications".to_string();
    }
    if let Some(fmt_cat) = script
        .strip_prefix(".Formats:")
        .or_else(|| script.strip_prefix("Formats:"))
    {
        let cat_trimmed = fmt_cat.trim();
        if cat_trimmed.starts_with("format_") {
            return cat_trimmed.to_string();
        }
        return format!("format_{cat_trimmed}");
    }
    if script == ".Semantic" || script.starts_with(".Semantic:") {
        return "semantic".to_string();
    }
    if script == ".EL" || script.starts_with(".EL ") {
        return "el".to_string();
    }
    if script == ".DCE" || script.starts_with(".DCE ") {
        return "dce".to_string();
    }
    let static_cat = match script {
        ".Colors" => "colors",
        ".Controls" => "controls",
        ".Encapsulation" => "encapsulation",
        ".Financial" => "financial",
        "Latin" => "latin",
        ".Mathematics" => "mathematics",
        ".Miscellaneous" => "miscellaneous",
        ".Noncharacters" => "noncharacters",
        ".Punctuation" => "punctuation",
        ".Styling" => "styling",
        ".Symbols" | ".Emoji" => "symbols",
        ".Variant selectors" => "variant_selectors",
        _ => "controls",
    };
    static_cat.to_string()
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
        DC_SEMANTIC_DECOMPOSITION,
    };
    use crate::dc_def::DcDefn;
    use crate::graph_ledger::{
        encode_integer_value_tokens, encode_nodes_to_ctbnbdl_bytes,
        encode_statement_tokens, encode_string_value_tokens,
        update_ledger_in_memory,
    };
    use crate::shared::{BidiClass, GeneralCategory};
    use crate::syntax::parse_dc_syntax;
    use ctb_utilities::format_id::DC_FILE_EXTENSION;
    use std::collections::BTreeSet;

    fn make_test_defn(short_id: usize, name: &str, desc: &str) -> DcDefn {
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
    fn test_supersession_chains_and_retractions_in_seed_graph_index() -> Result<()>
    {
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
            sh( crate::dc::DC_BOOLEAN_TRUE )
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
        assert!(idx3.get_direct_objects(sh(DC_IS_SUBTYPE_OF), subj).is_empty());
        assert!(
            !idx3.has_direct_statement(
                sh(DC_IS_DEPRECATED),
                subj,
                sh(crate::dc::DC_BOOLEAN_TRUE)
            )
        );
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
        for row_idx in 0..table.row_count() {
            let Some(row) = table.row(row_idx) else {
                continue;
            };
            if row.iter().all(|c| c.trim().is_empty()) {
                continue;
            }
            let name = row[0].trim();
            let raw_syntax = row[1].trim();
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
        }

        // 2. Verify every Dc syntax rule in the dataset decodes identically from the seed graph
        for defn in crate::lookup::get_all_dc_defns() {
            let Some(expected_rule) = &defn.syntax else {
                continue;
            };
            let subj = DcShorthand::from_global_id(defn.dc_id);
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
        assert_eq!(
            index.active_single_props,
            full_index.active_single_props
        );
        assert_eq!(
            index.active_multi_props,
            full_index.active_multi_props
        );
        assert_eq!(index.active_preferred, full_index.active_preferred);
        assert_eq!(
            index.active_direct_stmts,
            full_index.active_direct_stmts
        );
        assert_eq!(index.seed_entity_ids, full_index.seed_entity_ids);

        Ok(())
    }
}

