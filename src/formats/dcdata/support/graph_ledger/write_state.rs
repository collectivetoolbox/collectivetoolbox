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

//! Unified System-block node validation, deduplication, and `Dc 592` retraction
//! cancellation shared by CSV/StageL/WAT reconciliation and manual inbox
//! allocation.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::collections::{BTreeSet, HashMap, HashSet};

use ctb_storage_minimal::global_graph_layout::{
    SYSTEM_REGION_END, SYSTEM_REGION_START,
};
use ctb_storage_minimal::shorthand::DcShorthand;

use super::inbox::{NodeProvenance, NodeProvenanceEntry, NodeProvenanceMap};
use super::{
    DC_COLLECTIVE_TOOLBOX_DC_CSV_SEED_DATABASE, DC_DISTRIBUTE_STATEMENT_OVER_EACH,
    DC_ENCLOSING_DOCUMENT, DC_HAS_A, DC_HAS_VALUE, DC_PREFERRED, DC_RETRACTS,
    DC_SUPERSEDES, LedgerNode, default_ctbnbdl_framing_dcs,
    encode_statement_tokens, is_relationship_verb_with,
    parse_for_each_statement_tokens, parse_statement_tokens, sh,
};
use crate::ctbnbdl::CtbnbdlFramingDcs;
use crate::seed_graph::{
    SeedGraphIndex, compute_effective_inactive_ids,
    parse_ambiguous_values_tokens,
};

/// Controls how exact active-node token duplicates are handled on append.
#[derive(Debug, Clone, Copy)]
pub(crate) enum DuplicatePolicy<'a> {
    /// Manual/inbox policy: every matching active duplicate ID must be listed
    /// in `# @allow-duplicate-of: l<id1>, ...`.
    RequireAllowDuplicateOf(&'a [u128]),
    /// Distinct entity or per-instance multi-valued statement allocation where
    /// the same token tuple legitimately instantiates multiple nodes.
    AllowDistinctInstance,
    /// Rejects appending a new active node whose exact tokens match an already
    /// active node in the System graph.
    ForbidActiveDuplicate,
}

/// Shared append-only write model for System-block graph nodes.
///
/// Centralizes node-ID allocation, forward/self-reference checks, statement
/// syntax and cross-ownership checks, active-token duplicate policies,
/// reverse-referrer tracking, and `Dc 592` (`Retracts`) cancellation when
/// re-asserting previously retracted facts.
#[derive(Debug, Clone)]
pub(crate) struct SystemGraphWriteState {
    pub(crate) nodes: Vec<LedgerNode>,
    pub(crate) node_index_by_id: HashMap<u128, usize>,
    pub(crate) initial_node_count: usize,
    pub(crate) had_initial_in_graph_provenance: bool,
    pub(crate) node_data_source: HashMap<u128, DcShorthand>,
    pub(crate) provenance_stmt_ids: HashSet<u128>,
    pub(crate) provenance_map: NodeProvenanceMap,
    pub(crate) manual_referenced_ids: HashMap<u128, u128>,
    pub(crate) inactive_ids: HashSet<u128>,
    pub(crate) superseded_by: HashMap<u128, u128>,
    pub(crate) retracted_by: HashMap<u128, u128>,
    pub(crate) active_nodes_by_tokens: HashMap<Vec<DcShorthand>, Vec<u128>>,
    pub(crate) retracted_nodes_by_tokens: HashMap<Vec<DcShorthand>, Vec<u128>>,
    pub(crate) expanded_tokens_by_value_id: HashMap<u128, Vec<DcShorthand>>,
    pub(crate) active_value_by_expanded_tokens:
        HashMap<Vec<DcShorthand>, Vec<u128>>,
    pub(crate) retracted_value_by_expanded_tokens:
        HashMap<Vec<DcShorthand>, Vec<u128>>,
    pub(crate) referrers_by_id: HashMap<u128, Vec<u128>>,
    pub(crate) framing: CtbnbdlFramingDcs,
}

impl SystemGraphWriteState {
    /// Builds a `SystemGraphWriteState` from pre-indexed `SeedGraphIndex` data
    /// and a loaded `NodeProvenanceMap`.
    #[must_use]
    pub(crate) fn from_seed_index_and_provenance(
        index: &SeedGraphIndex,
        provenance_map: NodeProvenanceMap,
    ) -> Self {
        let initial_node_count = index.nodes.len();
        let had_initial_in_graph_provenance = index.has_in_graph_provenance();
        let mut manual_referenced_ids = HashMap::new();
        for node in &index.nodes {
            if !index.inactive_ids.contains(&node.id)
                && !index.provenance_stmt_ids.contains(&node.id)
                && Self::is_initial_non_csv_node(
                    node.id,
                    initial_node_count,
                    &index.node_data_source,
                    &index.provenance_stmt_ids,
                    had_initial_in_graph_provenance,
                    &provenance_map,
                )
            {
                for &tok in &node.tokens {
                    if let DcShorthand::Long(ref_id) = tok
                        && !index.inactive_ids.contains(&ref_id)
                        && !Self::is_initial_non_csv_node(
                            ref_id,
                            initial_node_count,
                            &index.node_data_source,
                            &index.provenance_stmt_ids,
                            had_initial_in_graph_provenance,
                            &provenance_map,
                        )
                    {
                        manual_referenced_ids.entry(ref_id).or_insert(node.id);
                    }
                }
            }
        }

        let mut node_index_by_id = HashMap::with_capacity(index.nodes.len());
        let mut referrers_by_id: HashMap<u128, Vec<u128>> = HashMap::new();
        for (pos, node) in index.nodes.iter().enumerate() {
            node_index_by_id.insert(node.id, pos);
            Self::record_referrers(&mut referrers_by_id, node);
        }

        let (inactive_ids, superseded_by, retracted_by) =
            compute_effective_inactive_ids(&index.nodes);
        let (
            active_nodes_by_tokens,
            retracted_nodes_by_tokens,
            expanded_tokens_by_value_id,
            active_value_by_expanded_tokens,
            retracted_value_by_expanded_tokens,
        ) = Self::build_token_indices(
            &index.nodes,
            &node_index_by_id,
            &inactive_ids,
            &superseded_by,
            &retracted_by,
        );

        Self {
            nodes: index.nodes.clone(),
            node_index_by_id,
            initial_node_count,
            had_initial_in_graph_provenance,
            node_data_source: index.node_data_source.clone(),
            provenance_stmt_ids: index.provenance_stmt_ids.clone(),
            provenance_map,
            manual_referenced_ids,
            inactive_ids,
            superseded_by,
            retracted_by,
            active_nodes_by_tokens,
            retracted_nodes_by_tokens,
            expanded_tokens_by_value_id,
            active_value_by_expanded_tokens,
            retracted_value_by_expanded_tokens,
            referrers_by_id,
            framing: default_ctbnbdl_framing_dcs(),
        }
    }

    /// Lightweight initializer from raw ledger nodes and verified provenance
    /// (used by inbox prevalidation).
    #[must_use]
    pub(crate) fn from_nodes_and_provenance(
        nodes: &[LedgerNode],
        provenance_map: NodeProvenanceMap,
    ) -> Self {
        let mut node_index_by_id = HashMap::with_capacity(nodes.len());
        let mut referrers_by_id: HashMap<u128, Vec<u128>> = HashMap::new();
        for (pos, node) in nodes.iter().enumerate() {
            node_index_by_id.insert(node.id, pos);
            Self::record_referrers(&mut referrers_by_id, node);
        }

        let (inactive_ids, superseded_by, retracted_by) =
            compute_effective_inactive_ids(nodes);
        let (
            active_nodes_by_tokens,
            retracted_nodes_by_tokens,
            expanded_tokens_by_value_id,
            active_value_by_expanded_tokens,
            retracted_value_by_expanded_tokens,
        ) = Self::build_token_indices(
            nodes,
            &node_index_by_id,
            &inactive_ids,
            &superseded_by,
            &retracted_by,
        );

        Self {
            nodes: nodes.to_vec(),
            node_index_by_id,
            initial_node_count: nodes.len(),
            had_initial_in_graph_provenance: false,
            node_data_source: HashMap::new(),
            provenance_stmt_ids: HashSet::new(),
            provenance_map,
            manual_referenced_ids: HashMap::new(),
            inactive_ids,
            superseded_by,
            retracted_by,
            active_nodes_by_tokens,
            retracted_nodes_by_tokens,
            expanded_tokens_by_value_id,
            active_value_by_expanded_tokens,
            retracted_value_by_expanded_tokens,
            referrers_by_id,
            framing: default_ctbnbdl_framing_dcs(),
        }
    }

    fn is_initial_non_csv_node(
        node_id: u128,
        initial_node_count: usize,
        node_data_source: &HashMap<u128, DcShorthand>,
        provenance_stmt_ids: &HashSet<u128>,
        had_initial_in_graph_provenance: bool,
        provenance_map: &NodeProvenanceMap,
    ) -> bool {
        // Reason for fallback: usize always fits in u128 on all supported targets.
        let initial_end_id = SYSTEM_REGION_START
            .saturating_add(u128::try_from(initial_node_count).unwrap_or(0));
        if node_id >= initial_end_id {
            return false;
        }
        if let Some(entry) = provenance_map.get(node_id) {
            return entry.provenance == NodeProvenance::Manual;
        }
        if let Some(&src) = node_data_source.get(&node_id) {
            return src != sh(DC_COLLECTIVE_TOOLBOX_DC_CSV_SEED_DATABASE);
        }
        if had_initial_in_graph_provenance {
            return !provenance_stmt_ids.contains(&node_id);
        }
        false
    }

    fn record_referrers(
        referrers_by_id: &mut HashMap<u128, Vec<u128>>,
        node: &LedgerNode,
    ) {
        let mut seen = BTreeSet::new();
        for &tok in &node.tokens {
            if let DcShorthand::Long(ref_id) = tok
                && seen.insert(ref_id)
            {
                referrers_by_id.entry(ref_id).or_default().push(node.id);
            }
        }
    }

    fn is_revision_tokens(tokens: &[DcShorthand]) -> bool {
        let sh_sup = sh(DC_SUPERSEDES);
        let sh_ret = sh(DC_RETRACTS);
        parse_statement_tokens(tokens)
            .is_some_and(|(v, _, _)| v == sh_sup || v == sh_ret)
    }

    #[must_use]
    pub(crate) fn is_value_node_tokens(tokens: &[DcShorthand]) -> bool {
        parse_statement_tokens(tokens).is_none()
            && parse_for_each_statement_tokens(tokens).is_none()
            && parse_ambiguous_values_tokens(tokens).is_none()
    }

    fn expand_tokens_from_parts(
        tokens: &[DcShorthand],
        nodes: &[LedgerNode],
        node_index_by_id: &HashMap<u128, usize>,
        inactive_ids: &HashSet<u128>,
    ) -> Vec<DcShorthand> {
        SeedGraphIndex::expand_interned_nodes_with_lookup(tokens, &|child_id| {
            if inactive_ids.contains(&child_id) {
                return None;
            }
            node_index_by_id
                .get(&child_id)
                .and_then(|&idx| nodes.get(idx))
                .map(|n| n.tokens.as_slice())
        })
    }

    #[allow(
        clippy::type_complexity,
        reason = "Private index tuple keeps exact/expanded and active/retracted build pass cohesive"
    )]
    fn build_token_indices(
        nodes: &[LedgerNode],
        node_index_by_id: &HashMap<u128, usize>,
        inactive_ids: &HashSet<u128>,
        superseded_by: &HashMap<u128, u128>,
        retracted_by: &HashMap<u128, u128>,
    ) -> (
        HashMap<Vec<DcShorthand>, Vec<u128>>,
        HashMap<Vec<DcShorthand>, Vec<u128>>,
        HashMap<u128, Vec<DcShorthand>>,
        HashMap<Vec<DcShorthand>, Vec<u128>>,
        HashMap<Vec<DcShorthand>, Vec<u128>>,
    ) {
        let mut active: HashMap<Vec<DcShorthand>, Vec<u128>> =
            HashMap::with_capacity(nodes.len());
        let mut retracted: HashMap<Vec<DcShorthand>, Vec<u128>> =
            HashMap::new();
        let mut expanded_by_value_id: HashMap<u128, Vec<DcShorthand>> =
            HashMap::new();
        let mut active_expanded: HashMap<Vec<DcShorthand>, Vec<u128>> =
            HashMap::new();
        let mut retracted_expanded: HashMap<Vec<DcShorthand>, Vec<u128>> =
            HashMap::new();
        for n in nodes {
            if Self::is_revision_tokens(&n.tokens) {
                continue;
            }
            let is_val = Self::is_value_node_tokens(&n.tokens);
            if !inactive_ids.contains(&n.id) {
                active.entry(n.tokens.clone()).or_default().push(n.id);
                if is_val {
                    let expanded = Self::expand_tokens_from_parts(
                        &n.tokens,
                        nodes,
                        node_index_by_id,
                        inactive_ids,
                    );
                    expanded_by_value_id.insert(n.id, expanded.clone());
                    active_expanded.entry(expanded).or_default().push(n.id);
                }
            } else if retracted_by.contains_key(&n.id)
                && !superseded_by.contains_key(&n.id)
            {
                retracted.entry(n.tokens.clone()).or_default().push(n.id);
                if is_val {
                    let expanded = Self::expand_tokens_from_parts(
                        &n.tokens,
                        nodes,
                        node_index_by_id,
                        inactive_ids,
                    );
                    expanded_by_value_id.insert(n.id, expanded.clone());
                    retracted_expanded.entry(expanded).or_default().push(n.id);
                }
            }
        }
        (
            active,
            retracted,
            expanded_by_value_id,
            active_expanded,
            retracted_expanded,
        )
    }

    /// Expands interned `[string]`, comment, and identifier nodes within
    /// `tokens` against currently active nodes in this write state.
    #[must_use]
    pub(crate) fn expand_tokens_active(
        &self,
        tokens: &[DcShorthand],
    ) -> Vec<DcShorthand> {
        Self::expand_tokens_from_parts(
            tokens,
            &self.nodes,
            &self.node_index_by_id,
            &self.inactive_ids,
        )
    }

    /// Finds the highest (newest/most-compact) active value node ID whose
    /// expanded token stream equals `expanded`, if any.
    #[must_use]
    pub(crate) fn find_active_value_by_expanded_tokens(
        &self,
        expanded: &[DcShorthand],
    ) -> Option<u128> {
        self.active_value_by_expanded_tokens
            .get(expanded)?
            .iter()
            .copied()
            .filter(|id| !self.inactive_ids.contains(id))
            .max()
    }

    /// Returns the node with `node_id`, if present.
    #[must_use]
    pub(crate) fn get_node(&self, node_id: u128) -> Option<&LedgerNode> {
        let &idx = self.node_index_by_id.get(&node_id)?;
        self.nodes.get(idx)
    }

    /// Returns the next append-only System-region node ID (`SYSTEM_REGION_START + len`).
    pub(crate) fn next_node_id(&self, context_label: &str) -> Result<u128> {
        let next_id = SYSTEM_REGION_START
            .saturating_add(u128::try_from(self.nodes.len())?);
        ensure!(
            next_id <= SYSTEM_REGION_END,
            "{context_label}: system graph region exhausted at node ID l{next_id}"
        );
        Ok(next_id)
    }

    /// Returns the provenance owner (`Csv` or `Manual`) of an existing node.
    #[must_use]
    pub(crate) fn owner_of(&self, node_id: u128) -> NodeProvenance {
        if let Some(entry) = self.provenance_map.get(node_id) {
            return entry.provenance;
        }
        if Self::is_initial_non_csv_node(
            node_id,
            self.initial_node_count,
            &self.node_data_source,
            &self.provenance_stmt_ids,
            self.had_initial_in_graph_provenance,
            &self.provenance_map,
        ) {
            NodeProvenance::Manual
        } else {
            NodeProvenance::Csv
        }
    }

    /// Convenience predicate matching `LedgerBuilder::is_manual_node`.
    #[must_use]
    pub(crate) fn is_manual_node(&self, node_id: u128) -> bool {
        self.owner_of(node_id) == NodeProvenance::Manual
    }

    /// Verifies that `actor_prov` is permitted to supersede, retract, or attach
    /// qualifiers to `target_id`.
    pub(crate) fn ensure_can_modify_target(
        &self,
        target_id: u128,
        actor_prov: NodeProvenance,
        label: &str,
        action: &str,
    ) -> Result<()> {
        let target_owner = self.owner_of(target_id);
        ensure!(
            target_owner == actor_prov,
            "Cross-ownership conflict (cross-ownership violation) ({label}): {actor_prov:?} cannot {action} {target_owner:?}-owned node l{target_id}"
        );
        if actor_prov == NodeProvenance::Csv
            && let Some(&manual_id) = self.manual_referenced_ids.get(&target_id)
        {
            bail!(
                "Cross-ownership conflict ({label}): CSV reconciliation cannot {action} CSV node l{target_id} because active manual node l{manual_id} references it"
            );
        }
        Ok(())
    }

    /// Records a node ID as having an active in-graph `Dc 635` CSV provenance
    /// assertion so subsequent passes do not re-tag it.
    pub(crate) fn mark_in_graph_csv_source(&mut self, node_id: u128) {
        self.node_data_source
            .insert(node_id, sh(DC_COLLECTIVE_TOOLBOX_DC_CSV_SEED_DATABASE));
    }

    /// Records a newly created `Dc 638` / `Dc 610` CSV provenance statement node.
    pub(crate) fn mark_csv_provenance_stmt(&mut self, stmt_id: u128) {
        self.provenance_stmt_ids.insert(stmt_id);
    }

    /// Validates `tokens` against all System-block structural, syntactic,
    /// provenance, and duplicate rules, appends the node, and updates active
    /// and retracted indices atomically.
    pub(crate) fn validate_and_append_node(
        &mut self,
        tokens: Vec<DcShorthand>,
        comment: String,
        provenance: NodeProvenance,
        rationale: Option<String>,
        allocated_file: Option<String>,
        duplicate_policy: DuplicatePolicy<'_>,
        label: &str,
    ) -> Result<u128> {
        ensure!(!tokens.is_empty(), "{label}: node body cannot be empty");
        let next_id = self.next_node_id(label)?;

        // 1. No self or forward `Long` references.
        for &tok in &tokens {
            if let DcShorthand::Long(ref_id) = tok {
                ensure!(
                    ref_id < next_id
                        && self.node_index_by_id.contains_key(&ref_id),
                    "{label} (l{next_id}): invalid self or forward reference to non-existent node l{ref_id}"
                );
            }
        }

        // 2. Validate structural statement rules.
        let is_revision_stmt = self.validate_statement_structure_for_append(
            &tokens,
            next_id,
            provenance,
            label,
        )?;

        // 3. Enforce active duplicate policy.
        self.enforce_duplicate_policy(
            &tokens,
            next_id,
            is_revision_stmt,
            duplicate_policy,
            label,
        )?;

        // 4. Append node and update indices.
        let new_node = LedgerNode {
            id: next_id,
            tokens: tokens.clone(),
            comment,
        };
        let new_pos = self.nodes.len();
        Self::record_referrers(&mut self.referrers_by_id, &new_node);
        self.nodes.push(new_node);
        self.node_index_by_id.insert(next_id, new_pos);
        self.provenance_map.insert(NodeProvenanceEntry {
            node_id: next_id,
            provenance,
            rationale,
            allocated_file,
        });

        self.update_indices_after_append(next_id, tokens, is_revision_stmt);
        Ok(next_id)
    }

    fn validate_statement_structure_for_append(
        &self,
        tokens: &[DcShorthand],
        next_id: u128,
        provenance: NodeProvenance,
        label: &str,
    ) -> Result<bool> {
        let sh_supersedes = sh(DC_SUPERSEDES);
        let sh_retracts = sh(DC_RETRACTS);
        let sh_has_a = sh(DC_HAS_A);
        let sh_has_value = sh(DC_HAS_VALUE);
        let sh_preferred = sh(DC_PREFERRED);
        let sh_enclosing_doc = sh(DC_ENCLOSING_DOCUMENT);
        let sh_dist = sh(DC_DISTRIBUTE_STATEMENT_OVER_EACH);

        if tokens.first() == Some(&DcShorthand::Short(348)) {
            let Some(cands) = parse_ambiguous_values_tokens(tokens) else {
                bail!(
                    "{label} (l{next_id}): malformed Dc 348 'Ambiguous values' syntax; expected '348 310 275 l<cand_1> 312 l<cand_2> ... 311'"
                );
            };
            let mut unique_cands = HashSet::with_capacity(cands.len());
            for cand_id in cands {
                ensure!(
                    cand_id < next_id
                        && self.node_index_by_id.contains_key(&cand_id),
                    "{label} (l{next_id}): Dc 348 candidate l{cand_id} does not exist"
                );
                ensure!(
                    unique_cands.insert(cand_id),
                    "{label} (l{next_id}): duplicate candidate l{cand_id} in Dc 348 list"
                );
            }
            return Ok(false);
        }

        if tokens.first() == Some(&sh_dist) {
            let Some((inner_verb, _, _, _, elements)) =
                parse_for_each_statement_tokens(tokens)
            else {
                bail!(
                    "{label} (l{next_id}): malformed Dc 638 'Distribute statement over each element' statement syntax"
                );
            };
            ensure!(
                is_relationship_verb_with(inner_verb, self.framing),
                "{label} (l{next_id}): Dc 638 'Distribute statement over each element' inner verb {inner_verb} is not a relationship verb"
            );
            ensure!(
                !elements.is_empty(),
                "{label} (l{next_id}): Dc 638 'Distribute statement over each element' list cannot be empty"
            );
            return Ok(false);
        }

        let Some(&first_tok) = tokens.first() else {
            return Ok(false);
        };
        if !is_relationship_verb_with(first_tok, self.framing) {
            return Ok(false);
        }

        let Some((verb, subj, obj)) = parse_statement_tokens(tokens) else {
            bail!(
                "{label} (l{next_id}): malformed relationship statement starting with {first_tok}"
            );
        };

        if verb == sh_supersedes {
            let (DcShorthand::Long(new_id), DcShorthand::Long(old_id)) =
                (subj, obj)
            else {
                bail!(
                    "{label} (l{next_id}): Dc 591 'Supersedes' requires Long node IDs '591 l<new_id> l<old_id>'"
                );
            };
            ensure!(
                new_id != old_id,
                "{label} (l{next_id}): self-supersession '591 l{new_id} l{old_id}' is forbidden"
            );
            ensure!(
                !self.inactive_ids.contains(&old_id),
                "{label} (l{next_id}): cannot supersede l{old_id} because it is already superseded or retracted"
            );
            ensure!(
                !self.inactive_ids.contains(&new_id),
                "{label} (l{next_id}): replacement node l{new_id} is already inactive"
            );
            let mut cur = new_id;
            while let Some(&next_sup) = self.superseded_by.get(&cur) {
                ensure!(
                    next_sup != old_id,
                    "{label} (l{next_id}): supersession cycle detected between l{new_id} and l{old_id}"
                );
                cur = next_sup;
            }
            self.ensure_can_modify_target(
                old_id,
                provenance,
                label,
                "supersede",
            )?;
            return Ok(true);
        }

        if verb == sh_retracts {
            let DcShorthand::Long(old_id) = obj else {
                bail!(
                    "{label} (l{next_id}): Dc 592 'Retracts' object must be a Long node ID 'l<old_id>'"
                );
            };
            ensure!(
                subj == sh_enclosing_doc,
                "{label} (l{next_id}): Dc 592 'Retracts' subject must be EnclosingDocument ({sh_enclosing_doc})"
            );
            ensure!(
                !self.inactive_ids.contains(&old_id),
                "{label} (l{next_id}): cannot retract l{old_id} because it is already superseded or retracted"
            );
            self.ensure_can_modify_target(
                old_id,
                provenance,
                label,
                "retract",
            )?;
            return Ok(true);
        }

        if verb == sh_has_value || (verb == sh_has_a && obj == sh_preferred) {
            let DcShorthand::Long(has_a_id) = subj else {
                bail!(
                    "{label} (l{next_id}): {verb} subject must be a Long 'Has a' statement ID 'l<has_a_id>'"
                );
            };
            let target_node = self.get_node(has_a_id).with_context(|| {
                format!(
                    "{label} (l{next_id}): target 'Has a' node l{has_a_id} does not exist"
                )
            })?;
            let check_node_is_has_a = |n: &LedgerNode| -> bool {
                parse_statement_tokens(&n.tokens)
                    .is_some_and(|(v, _, o)| v == sh_has_a && o != sh_preferred)
                    || parse_for_each_statement_tokens(&n.tokens).is_some_and(
                        |(v, _, o, _, _)| v == sh_has_a && o != sh_preferred,
                    )
            };
            let mut is_has_a = check_node_is_has_a(target_node);
            if !is_has_a {
                // If `has_a_id` was superseded by Dc 348 "Ambiguous values",
                // accept when any candidate is a property `Has a` node.
                let mut cur = has_a_id;
                while let Some(&next_sup) = self.superseded_by.get(&cur) {
                    cur = next_sup;
                }
                if let Some(eff_node) = self.get_node(cur)
                    && let Some(cands) =
                        parse_ambiguous_values_tokens(&eff_node.tokens)
                {
                    is_has_a = cands.iter().any(|&cand_id| {
                        self.get_node(cand_id).is_some_and(check_node_is_has_a)
                    });
                }
            }
            ensure!(
                is_has_a,
                "{label} (l{next_id}): target l{has_a_id} is not a property 'Has a' statement"
            );
            self.ensure_can_modify_target(
                has_a_id,
                provenance,
                label,
                "attach qualifier/value to",
            )?;
        }

        Ok(false)
    }

    fn enforce_duplicate_policy(
        &self,
        tokens: &[DcShorthand],
        next_id: u128,
        is_revision_stmt: bool,
        duplicate_policy: DuplicatePolicy<'_>,
        label: &str,
    ) -> Result<()> {
        match duplicate_policy {
            DuplicatePolicy::RequireAllowDuplicateOf(allowed) => {
                if is_revision_stmt {
                    ensure!(
                        allowed.is_empty(),
                        "{label} (l{next_id}): revision statements (591/592) must not specify # @allow-duplicate-of"
                    );
                } else {
                    // Reason for fallback: token bodies absent from active_nodes_by_tokens have zero active duplicates.
                    let mut active_matches = self
                        .active_nodes_by_tokens
                        .get(tokens)
                        .cloned()
                        .unwrap_or_default();
                    active_matches.retain(|id| !self.inactive_ids.contains(id));
                    active_matches.sort_unstable();
                    ensure!(
                        allowed == active_matches.as_slice(),
                        "{label} (l{next_id}): active duplicate mismatch for token body {tokens:?}: expected # @allow-duplicate-of: {active_matches:?}, found {allowed:?}"
                    );
                }
            }
            DuplicatePolicy::AllowDistinctInstance => {}
            DuplicatePolicy::ForbidActiveDuplicate => {
                if !is_revision_stmt
                    && let Some(existing_ids) =
                        self.active_nodes_by_tokens.get(tokens)
                    && existing_ids
                        .iter()
                        .any(|id| !self.inactive_ids.contains(id))
                {
                    bail!(
                        "{label} (l{next_id}): active duplicate already exists for {tokens:?}: {existing_ids:?}"
                    );
                }
            }
        }
        Ok(())
    }

    fn remove_id_from_token_map(
        map: &mut HashMap<Vec<DcShorthand>, Vec<u128>>,
        key: &[DcShorthand],
        target_id: u128,
    ) {
        if let Some(ids) = map.get_mut(key) {
            ids.retain(|&id| id != target_id);
            if ids.is_empty() {
                map.remove(key);
            }
        }
    }

    fn update_indices_after_append(
        &mut self,
        next_id: u128,
        tokens: Vec<DcShorthand>,
        is_revision_stmt: bool,
    ) {
        if !is_revision_stmt {
            if Self::is_value_node_tokens(&tokens) {
                let expanded = self.expand_tokens_active(&tokens);
                self.expanded_tokens_by_value_id
                    .insert(next_id, expanded.clone());
                self.active_value_by_expanded_tokens
                    .entry(expanded)
                    .or_default()
                    .push(next_id);
            }
            self.active_nodes_by_tokens
                .entry(tokens)
                .or_default()
                .push(next_id);
            return;
        }
        let Some((verb, subj, DcShorthand::Long(old_id))) =
            parse_statement_tokens(&tokens)
        else {
            return;
        };
        let old_was_revision = self
            .get_node(old_id)
            .is_some_and(|n| Self::is_revision_tokens(&n.tokens));
        if old_was_revision {
            let (re_inactive, re_sup, re_ret) =
                compute_effective_inactive_ids(&self.nodes);
            self.inactive_ids = re_inactive;
            self.superseded_by = re_sup;
            self.retracted_by = re_ret;
            let (
                re_active,
                re_retracted_map,
                re_expanded_by_val,
                re_active_expanded,
                re_retracted_expanded,
            ) = Self::build_token_indices(
                &self.nodes,
                &self.node_index_by_id,
                &self.inactive_ids,
                &self.superseded_by,
                &self.retracted_by,
            );
            self.active_nodes_by_tokens = re_active;
            self.retracted_nodes_by_tokens = re_retracted_map;
            self.expanded_tokens_by_value_id = re_expanded_by_val;
            self.active_value_by_expanded_tokens = re_active_expanded;
            self.retracted_value_by_expanded_tokens = re_retracted_expanded;
            return;
        }

        self.inactive_ids.insert(old_id);
        let old_tokens_opt = self.get_node(old_id).map(|n| n.tokens.clone());
        if let Some(ref old_tokens) = old_tokens_opt {
            Self::remove_id_from_token_map(
                &mut self.active_nodes_by_tokens,
                old_tokens,
                old_id,
            );
        }
        let old_expanded_opt =
            self.expanded_tokens_by_value_id.get(&old_id).cloned();
        if let Some(ref old_expanded) = old_expanded_opt {
            Self::remove_id_from_token_map(
                &mut self.active_value_by_expanded_tokens,
                old_expanded,
                old_id,
            );
        }

        if verb == sh(DC_SUPERSEDES) {
            if let DcShorthand::Long(new_id) = subj {
                self.superseded_by.insert(old_id, new_id);
                self.expanded_tokens_by_value_id.remove(&old_id);
            }
        } else if verb == sh(DC_RETRACTS) {
            self.retracted_by.insert(old_id, next_id);
            if let Some(old_tokens) = old_tokens_opt
                && !Self::is_revision_tokens(&old_tokens)
                && !self.superseded_by.contains_key(&old_id)
            {
                if let Some(old_expanded) = old_expanded_opt {
                    self.retracted_value_by_expanded_tokens
                        .entry(old_expanded)
                        .or_default()
                        .push(old_id);
                }
                self.retracted_nodes_by_tokens
                    .entry(old_tokens)
                    .or_default()
                    .push(old_id);
            }
        }
    }

    /// Checks whether a retracted node `target_id` can be safely reactivated
    /// by cancelling its `Dc 592` retraction statement on behalf of `actor_prov`.
    ///
    /// Returns `Some(retract_stmt_id)` when:
    /// - `target_id` is currently retracted (`retracted_by`) and not superseded,
    /// - `retract_stmt_id` is active and modifiable by `actor_prov`,
    /// - every `Long(dep_id)` referenced by `target_id` is active (or listed in
    ///   `extra_allowed_deps`), and
    /// - every non-revision, non-provenance node referencing `target_id` is
    ///   already inactive (or listed in `extra_allowed_dependents`), so
    ///   reactivating `target_id` cannot wake unintended stale dependents.
    #[must_use]
    pub(crate) fn can_safely_unretract_node(
        &self,
        target_id: u128,
        actor_prov: NodeProvenance,
        extra_allowed_deps: &[u128],
        extra_allowed_dependents: &[u128],
    ) -> Option<u128> {
        if !self.inactive_ids.contains(&target_id)
            || self.superseded_by.contains_key(&target_id)
        {
            return None;
        }
        let &retract_stmt_id = self.retracted_by.get(&target_id)?;
        if self.inactive_ids.contains(&retract_stmt_id) {
            return None;
        }
        if self.owner_of(target_id) != actor_prov
            || self.owner_of(retract_stmt_id) != actor_prov
        {
            return None;
        }
        if actor_prov == NodeProvenance::Csv
            && (self.manual_referenced_ids.contains_key(&target_id)
                || self.manual_referenced_ids.contains_key(&retract_stmt_id))
        {
            return None;
        }

        let target_node = self.get_node(target_id)?;
        for &tok in &target_node.tokens {
            if let DcShorthand::Long(dep_id) = tok
                && self.inactive_ids.contains(&dep_id)
                && !extra_allowed_deps.contains(&dep_id)
            {
                return None;
            }
        }

        if let Some(referrers) = self.referrers_by_id.get(&target_id) {
            for &ref_id in referrers {
                if ref_id == retract_stmt_id
                    || self.provenance_stmt_ids.contains(&ref_id)
                    || extra_allowed_dependents.contains(&ref_id)
                {
                    continue;
                }
                if let Some(ref_node) = self.get_node(ref_id)
                    && Self::is_revision_tokens(&ref_node.tokens)
                {
                    continue;
                }
                if !self.inactive_ids.contains(&ref_id) {
                    return None;
                }
            }
        }

        Some(retract_stmt_id)
    }

    fn try_unretract_first_safe_candidate(
        &mut self,
        cands: &[u128],
        descending: bool,
        actor_prov: NodeProvenance,
        format_comment: &dyn Fn(&[DcShorthand]) -> String,
    ) -> Result<Option<(u128, u128)>> {
        let mut candidates = cands.to_vec();
        candidates.sort_unstable();
        candidates.dedup();
        if descending {
            candidates.reverse();
        }

        for target_id in candidates {
            if let Some(retract_stmt_id) =
                self.can_safely_unretract_node(target_id, actor_prov, &[], &[])
            {
                let cancel_tokens = encode_statement_tokens(
                    sh(DC_RETRACTS),
                    sh(DC_ENCLOSING_DOCUMENT),
                    DcShorthand::Long(retract_stmt_id),
                );
                let _cancel_id = self.validate_and_append_node(
                    cancel_tokens.clone(),
                    format_comment(&cancel_tokens),
                    actor_prov,
                    None,
                    None,
                    DuplicatePolicy::AllowDistinctInstance,
                    "un-retract node",
                )?;
                return Ok(Some((target_id, retract_stmt_id)));
            }
        }
        Ok(None)
    }

    /// Attempts to reactivate an earlier retracted node whose exact tokens match
    /// `tokens` by appending `Dc 592 "Retracts" Dc 615 "Enclosing document" l<retract_stmt_id>`.
    pub(crate) fn try_unretract_matching_node(
        &mut self,
        tokens: &[DcShorthand],
        actor_prov: NodeProvenance,
        format_comment: &dyn Fn(&[DcShorthand]) -> String,
    ) -> Result<Option<(u128, u128)>> {
        let Some(cands) = self.retracted_nodes_by_tokens.get(tokens).cloned()
        else {
            return Ok(None);
        };
        self.try_unretract_first_safe_candidate(
            &cands,
            false,
            actor_prov,
            format_comment,
        )
    }

    /// Attempts to reactivate an earlier retracted value node whose expanded
    /// tokens match `expanded` by cancelling its active `Dc 592` retraction,
    /// preferring newer compact retracted nodes over older legacy duplicates.
    pub(crate) fn try_unretract_matching_expanded_value_node(
        &mut self,
        expanded: &[DcShorthand],
        actor_prov: NodeProvenance,
        format_comment: &dyn Fn(&[DcShorthand]) -> String,
    ) -> Result<Option<(u128, u128)>> {
        let Some(cands) =
            self.retracted_value_by_expanded_tokens.get(expanded).cloned()
        else {
            return Ok(None);
        };
        self.try_unretract_first_safe_candidate(
            &cands,
            true,
            actor_prov,
            format_comment,
        )
    }

    /// Attempts to reactivate a previously retracted `(584 subj prop, 610 l<ha> val_tok)`
    /// property statement pair when `val_tok` is already active.
    pub(crate) fn try_unretract_property_pair(
        &mut self,
        subj: DcShorthand,
        prop: DcShorthand,
        val_tok: DcShorthand,
        actor_prov: NodeProvenance,
        format_comment: &dyn Fn(&[DcShorthand]) -> String,
    ) -> Result<Option<(u128, u128)>> {
        if subj == sh(DC_ENCLOSING_DOCUMENT) {
            return Ok(None);
        }
        if let DcShorthand::Long(val_id) = val_tok
            && self.inactive_ids.contains(&val_id)
        {
            return Ok(None);
        }

        let has_a_tokens = encode_statement_tokens(sh(DC_HAS_A), subj, prop);
        let Some(ha_cands) = self.retracted_nodes_by_tokens.get(&has_a_tokens)
        else {
            return Ok(None);
        };
        let mut ha_candidates = ha_cands.clone();
        ha_candidates.sort_unstable();

        for has_a_id in ha_candidates {
            let has_val_tokens = encode_statement_tokens(
                sh(DC_HAS_VALUE),
                DcShorthand::Long(has_a_id),
                val_tok,
            );
            let Some(hv_cands) =
                self.retracted_nodes_by_tokens.get(&has_val_tokens)
            else {
                continue;
            };
            let mut hv_candidates = hv_cands.clone();
            hv_candidates.sort_unstable();

            for has_val_id in hv_candidates {
                let Some(ret_ha_id) = self.can_safely_unretract_node(
                    has_a_id,
                    actor_prov,
                    &[],
                    &[has_val_id],
                ) else {
                    continue;
                };
                let Some(ret_hv_id) = self.can_safely_unretract_node(
                    has_val_id,
                    actor_prov,
                    &[has_a_id],
                    &[],
                ) else {
                    continue;
                };
                if ret_ha_id == ret_hv_id {
                    continue;
                }

                let cancel_ha_tokens = encode_statement_tokens(
                    sh(DC_RETRACTS),
                    sh(DC_ENCLOSING_DOCUMENT),
                    DcShorthand::Long(ret_ha_id),
                );
                let _ = self.validate_and_append_node(
                    cancel_ha_tokens.clone(),
                    format_comment(&cancel_ha_tokens),
                    actor_prov,
                    None,
                    None,
                    DuplicatePolicy::AllowDistinctInstance,
                    "un-retract has_a",
                )?;

                let cancel_hv_tokens = encode_statement_tokens(
                    sh(DC_RETRACTS),
                    sh(DC_ENCLOSING_DOCUMENT),
                    DcShorthand::Long(ret_hv_id),
                );
                let _ = self.validate_and_append_node(
                    cancel_hv_tokens.clone(),
                    format_comment(&cancel_hv_tokens),
                    actor_prov,
                    None,
                    None,
                    DuplicatePolicy::AllowDistinctInstance,
                    "un-retract has_value",
                )?;

                return Ok(Some((has_a_id, has_val_id)));
            }
        }

        Ok(None)
    }
}
