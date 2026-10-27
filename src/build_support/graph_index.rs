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

//! Build-time index over the `.ctbnodes` system graph ledger in `assets/data/graph/`.
//!
//! Self-bootstraps its core ontology predicates from the `"ProgrammaticIdentifier"`
//! string value node so build-time codegen (`dc_codegen`, `format_id_codegen`,
//! `extension_codegen`, `encoding_codegen`, `format_support_codegen`) can query
//! active statements, properties, qualifiers, and seed entities directly from the
//! graph without reading CSV files or hardcoding Document Character IDs.

use crate::ctbnbdl::{
    CtbnbdlFramingDcs, LedgerNode, load_ledger_nodes_from_dir,
    parse_statement_tokens_with, partition_active_and_inactive_nodes_with,
};
use crate::shorthand::DcShorthand;
use anyhow::{Context, Result, bail, ensure};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;

/// Classification of a System-block seed entity node (`586 615 <class_dc>`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BuildSeedEntity {
    /// Unicode Bidirectional Class code (e.g. `"BN"`, `"L"`).
    BidiClass(String),
    /// Character or Dc General Category code (e.g. `"!Cx"`, `"Cc"`, `"Zl"`).
    GeneralCategory(String),
    /// Script classification name (e.g. `".EL Types"`, `".Formats:compression"`).
    Script(String),
    /// Named syntax construct (e.g. `"value"`, `"boolean"`, `"number"`).
    NamedConstruct(String),
}

/// An active `Has a` (`Dc 584`) + `Has value` (`Dc 610`) property assignment on a subject.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildPropertyEntry {
    /// Node ID of the `Has a` statement (`584 <subj> <prop>`).
    pub has_a_id: u128,
    /// Node ID of the `Has value` statement (`610 l<has_a_id> <value>`).
    pub has_value_id: u128,
    /// Value operand token (`<value>`).
    pub value: DcShorthand,
    /// Whether this property claim is marked with `Preferred` (`584 l<has_a_id> 570`).
    pub is_preferred: bool,
}

/// Indexed view of active statements and value nodes in the system graph.
#[derive(Debug, Clone)]
pub struct BuildGraphIndex {
    framing: CtbnbdlFramingDcs,
    active_nodes: Vec<LedgerNode>,
    node_tokens_by_id: HashMap<u128, Vec<DcShorthand>>,
    strings_by_id: HashMap<u128, String>,
    ident_to_shorthand: HashMap<String, DcShorthand>,
    shorthand_to_ident: HashMap<DcShorthand, String>,
    seed_entity_by_id: HashMap<u128, BuildSeedEntity>,
    seed_id_by_entity: HashMap<BuildSeedEntity, u128>,
    properties_by_subj_prop:
        HashMap<(DcShorthand, DcShorthand), Vec<BuildPropertyEntry>>,
    qualifiers_by_claim_prop: HashMap<(u128, DcShorthand), Vec<DcShorthand>>,
    direct_objects_by_verb_subj:
        HashMap<(DcShorthand, DcShorthand), Vec<DcShorthand>>,
}

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

    // Find the self-referential `<HasA> <ProgrammaticIdentifier> <ProgrammaticIdentifier>` claim
    // paired with `<HasValue> l<claim_id> l<prog_ident_str_id>`.
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

    // First pass over raw ProgrammaticIdentifier claims to locate Supersedes, Retracts, and EnclosingDocument.
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

    // Compute inactive node IDs in reverse chronological order so superseded/retracted
    // identifier claims are ignored.
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

/// Resolves the 16 `CtbnbdlFramingDcs` from the `.ctbnodes` files in `graph_dir`.
///
/// # Errors
/// Returns an error if `graph_dir` cannot be read or any required framing Dc is missing.
pub fn resolve_ctbnbdl_framing_dcs_from_graph_dir(
    graph_dir: &Path,
) -> Result<CtbnbdlFramingDcs> {
    let nodes = load_ledger_nodes_from_dir(graph_dir)?;
    resolve_ctbnbdl_framing_dcs_from_nodes(&nodes)
}

impl BuildGraphIndex {
    /// Loads and indexes the system graph from `.ctbnodes` files in `graph_dir`.
    ///
    /// # Errors
    /// Returns an error if `graph_dir` cannot be read or graph bootstrapping fails.
    pub fn from_graph_dir(graph_dir: &Path) -> Result<Self> {
        let nodes = load_ledger_nodes_from_dir(graph_dir)?;
        Self::from_nodes(&nodes)
    }

    /// Loads and indexes the system graph from `repo_root/assets/data/graph`.
    ///
    /// # Errors
    /// Returns an error if `assets/data/graph` cannot be read or graph bootstrapping fails.
    pub fn from_repo_root(repo_root: &Path) -> Result<Self> {
        let graph_dir = repo_root.join("assets").join("data").join("graph");
        ensure!(
            graph_dir.is_dir(),
            "Could not locate system graph directory at {}",
            graph_dir.display()
        );
        Self::from_graph_dir(&graph_dir)
    }

    /// Builds a [`BuildGraphIndex`] from an in-memory slice of [`LedgerNode`]s.
    ///
    /// # Errors
    /// Returns an error if framing Dc resolution fails.
    pub fn from_nodes(nodes: &[LedgerNode]) -> Result<Self> {
        let framing = resolve_ctbnbdl_framing_dcs_from_nodes(nodes)?;
        let (active_nodes, _inactive_nodes) =
            partition_active_and_inactive_nodes_with(nodes, framing);

        let sh_has_a = framing.has_a;
        let sh_has_value = framing.has_value;
        let sh_preferred = framing.preferred;
        let sh_enclosing_doc = framing.enclosing_document;
        let sh_is_instance_of = framing.is_instance_of;

        let mut node_tokens_by_id: HashMap<u128, Vec<DcShorthand>> =
            HashMap::with_capacity(active_nodes.len());
        let mut strings_by_id: HashMap<u128, String> = HashMap::new();
        let mut has_a_claims: HashMap<u128, (DcShorthand, DcShorthand)> =
            HashMap::new();
        let mut preferred_claims: HashSet<u128> = HashSet::new();
        let mut seed_candidate_class: HashMap<u128, DcShorthand> =
            HashMap::new();

        for node in &active_nodes {
            node_tokens_by_id.insert(node.id, node.tokens.clone());
            if let Some(s) = decode_unicode_string_tokens(&node.tokens) {
                strings_by_id.insert(node.id, s);
            }
            if let Some((verb, subj, obj)) =
                parse_statement_tokens_with(&node.tokens, framing)
            {
                if verb == sh_is_instance_of
                    && subj == sh_enclosing_doc
                    && matches!(obj, DcShorthand::Short(_))
                {
                    seed_candidate_class.insert(node.id, obj);
                } else if verb == sh_has_a {
                    has_a_claims.insert(node.id, (subj, obj));
                    if obj == sh_preferred {
                        if let DcShorthand::Long(claim_id) = subj {
                            preferred_claims.insert(claim_id);
                        }
                    }
                }
            }
        }

        let mut properties_by_subj_prop: HashMap<
            (DcShorthand, DcShorthand),
            Vec<BuildPropertyEntry>,
        > = HashMap::new();
        let mut qualifiers_by_claim_prop: HashMap<
            (u128, DcShorthand),
            Vec<DcShorthand>,
        > = HashMap::new();
        let mut direct_objects_by_verb_subj: HashMap<
            (DcShorthand, DcShorthand),
            Vec<DcShorthand>,
        > = HashMap::new();

        for node in &active_nodes {
            let Some((verb, subj, obj)) =
                parse_statement_tokens_with(&node.tokens, framing)
            else {
                continue;
            };
            if verb == sh_is_instance_of
                && subj == sh_enclosing_doc
                && matches!(obj, DcShorthand::Short(_))
            {
                continue;
            }
            if verb == sh_has_value {
                if let DcShorthand::Long(claim_id) = subj {
                    if let Some(&(claim_subj, claim_prop)) =
                        has_a_claims.get(&claim_id)
                    {
                        let entry = BuildPropertyEntry {
                            has_a_id: claim_id,
                            has_value_id: node.id,
                            value: obj,
                            is_preferred: preferred_claims.contains(&claim_id),
                        };
                        properties_by_subj_prop
                            .entry((claim_subj, claim_prop))
                            .or_default()
                            .push(entry);
                        if let DcShorthand::Long(parent_claim_id) = claim_subj
                        {
                            qualifiers_by_claim_prop
                                .entry((parent_claim_id, claim_prop))
                                .or_default()
                                .push(obj);
                        }
                    }
                }
            } else if verb != sh_has_a {
                direct_objects_by_verb_subj
                    .entry((verb, subj))
                    .or_default()
                    .push(obj);
            }
        }

        // Locate ProgrammaticIdentifier Dc via its self-referential property entry.
        let mut prog_ident_dc_opt: Option<DcShorthand> = None;
        for (&(subj, prop), entries) in &properties_by_subj_prop {
            if subj == prop && matches!(subj, DcShorthand::Short(_)) {
                if let Some(last) = entries.last() {
                    if let DcShorthand::Long(str_id) = last.value {
                        if strings_by_id
                            .get(&str_id)
                            .is_some_and(|s| s == "ProgrammaticIdentifier")
                        {
                            prog_ident_dc_opt = Some(prop);
                            break;
                        }
                    }
                }
            }
        }
        let sh_prog_ident = prog_ident_dc_opt.context(
            "Failed to locate ProgrammaticIdentifier property in active graph nodes",
        )?;

        let mut ident_to_shorthand: HashMap<String, DcShorthand> =
            HashMap::new();
        let mut shorthand_to_ident: HashMap<DcShorthand, String> =
            HashMap::new();

        // Sort subjects deterministically so Short Dcs and Format Dcs register their identifiers cleanly.
        let mut ident_pairs: Vec<(DcShorthand, String)> = Vec::new();
        for (&(subj, prop), entries) in &properties_by_subj_prop {
            if prop == sh_prog_ident
                && matches!(
                    subj,
                    DcShorthand::Short(_) | DcShorthand::Format(_)
                )
            {
                if let Some(last) = entries.last() {
                    if let DcShorthand::Long(str_id) = last.value {
                        if let Some(ident) = strings_by_id.get(&str_id) {
                            ident_pairs.push((subj, ident.clone()));
                        }
                    }
                }
            }
        }
        ident_pairs.sort_by_key(|(subj, _)| {
            // Reason for fallback: Short and Format shorthands always have valid global IDs.
            subj.to_global_id().unwrap_or(0)
        });
        for (subj, ident) in ident_pairs {
            shorthand_to_ident.insert(subj, ident.clone());
            ident_to_shorthand.entry(ident).or_insert(subj);
        }

        // Decode empty string literal `260 262 264 263 261` if present.
        if let (
            Some(&begin_lit),
            Some(&begin_type),
            Some(&type_str),
            Some(&end_type),
            Some(&end_lit),
        ) = (
            ident_to_shorthand.get("BeginLiteral"),
            ident_to_shorthand.get("BeginTypeDef"),
            ident_to_shorthand.get("TypeString"),
            ident_to_shorthand.get("EndTypeDef"),
            ident_to_shorthand.get("EndLiteral"),
        ) {
            let empty_lit = [begin_lit, begin_type, type_str, end_type, end_lit];
            for node in &active_nodes {
                if node.tokens.as_slice() == empty_lit {
                    strings_by_id.insert(node.id, String::new());
                }
            }
        }

        // Index seed entities (`BidiClass`, `GeneralCategory`, `Script`, `NamedConstruct`).
        let sh_canon_name = ident_to_shorthand
            .get("CanonicalName")
            .copied()
            .context("Missing CanonicalName in system graph")?;
        let sh_bidi_class = ident_to_shorthand
            .get("BidirectionalClass")
            .copied();
        let sh_char_gen_cat = ident_to_shorthand
            .get("CharacterGeneralCategory")
            .copied();
        let sh_dc_gen_cat =
            ident_to_shorthand.get("DcGeneralCategory").copied();
        let sh_char_script =
            ident_to_shorthand.get("CharacterScript").copied();
        let sh_uni_script = ident_to_shorthand.get("UnicodeScript").copied();
        let sh_entity_type = ident_to_shorthand.get("EntityType").copied();

        let mut seed_entity_by_id: HashMap<u128, BuildSeedEntity> =
            HashMap::new();
        let mut seed_id_by_entity: HashMap<BuildSeedEntity, u128> =
            HashMap::new();

        let mut sorted_seed_candidates: Vec<(u128, DcShorthand)> =
            seed_candidate_class.into_iter().collect();
        sorted_seed_candidates.sort_by_key(|&(seed_id, _)| seed_id);

        for (seed_id, class_sh) in sorted_seed_candidates {
            let seed_subj = DcShorthand::Long(seed_id);
            let get_str_prop = |prop: DcShorthand| -> Option<String> {
                let entries =
                    properties_by_subj_prop.get(&(seed_subj, prop))?;
                let last = entries.last()?;
                let DcShorthand::Long(str_id) = last.value else {
                    return None;
                };
                strings_by_id.get(&str_id).cloned()
            };

            let seed_opt = if Some(class_sh) == sh_bidi_class {
                get_str_prop(sh_prog_ident).map(BuildSeedEntity::BidiClass)
            } else if Some(class_sh) == sh_char_gen_cat
                || Some(class_sh) == sh_dc_gen_cat
            {
                get_str_prop(sh_prog_ident)
                    .map(BuildSeedEntity::GeneralCategory)
            } else if Some(class_sh) == sh_char_script
                || Some(class_sh) == sh_uni_script
            {
                get_str_prop(sh_canon_name).map(BuildSeedEntity::Script)
            } else if Some(class_sh) == sh_entity_type {
                get_str_prop(sh_canon_name).map(BuildSeedEntity::NamedConstruct)
            } else {
                None
            };

            if let Some(seed_entity) = seed_opt {
                seed_entity_by_id.insert(seed_id, seed_entity.clone());
                seed_id_by_entity.insert(seed_entity, seed_id);
            }
        }

        Ok(Self {
            framing,
            active_nodes,
            node_tokens_by_id,
            strings_by_id,
            ident_to_shorthand,
            shorthand_to_ident,
            seed_entity_by_id,
            seed_id_by_entity,
            properties_by_subj_prop,
            qualifiers_by_claim_prop,
            direct_objects_by_verb_subj,
        })
    }

    /// Returns the resolved [`CtbnbdlFramingDcs`] for this graph.
    #[must_use]
    pub const fn framing(&self) -> CtbnbdlFramingDcs {
        self.framing
    }

    /// Returns all active [`LedgerNode`]s in ascending node ID order.
    #[must_use]
    pub fn active_nodes(&self) -> &[LedgerNode] {
        &self.active_nodes
    }

    /// Returns the raw shorthand tokens of an active node by its node ID.
    #[must_use]
    pub fn node_tokens(&self, node_id: u128) -> Option<&[DcShorthand]> {
        self.node_tokens_by_id.get(&node_id).map(Vec::as_slice)
    }

    /// Returns the decoded UTF-8 string for a string value node ID.
    #[must_use]
    pub fn get_string(&self, node_id: u128) -> Option<&str> {
        self.strings_by_id.get(&node_id).map(String::as_str)
    }

    /// Returns the decoded UTF-8 string for a `DcShorthand::Long(node_id)` token.
    #[must_use]
    pub fn get_shorthand_string(&self, tok: DcShorthand) -> Option<&str> {
        let DcShorthand::Long(node_id) = tok else {
            return None;
        };
        self.get_string(node_id)
    }

    /// Looks up a Short Dc or Format Dc by its `ProgrammaticIdentifier` (`@ident`).
    #[must_use]
    pub fn shorthand_by_ident(&self, ident: &str) -> Option<DcShorthand> {
        self.ident_to_shorthand.get(ident).copied()
    }

    /// Looks up a required Short Dc or Format Dc by its `ProgrammaticIdentifier` (`@ident`).
    ///
    /// # Errors
    /// Returns an error if no entity in the graph has `@ident(ident)`.
    pub fn require_ident(&self, ident: &str) -> Result<DcShorthand> {
        self.shorthand_by_ident(ident).with_context(|| {
            format!("Required @ident(\"{ident}\") not found in system graph")
        })
    }

    /// Returns the `ProgrammaticIdentifier` (`@ident`) for a subject if defined.
    #[must_use]
    pub fn ident_for_shorthand(&self, subj: DcShorthand) -> Option<&str> {
        self.shorthand_to_ident.get(&subj).map(String::as_str)
    }

    /// Returns the [`BuildSeedEntity`] for a System-block seed node ID.
    #[must_use]
    pub fn seed_entity(&self, node_id: u128) -> Option<&BuildSeedEntity> {
        self.seed_entity_by_id.get(&node_id)
    }

    /// Returns the System-block seed node ID for a [`BuildSeedEntity`].
    #[must_use]
    pub fn seed_node_id(&self, entity: &BuildSeedEntity) -> Option<u128> {
        self.seed_id_by_entity.get(entity).copied()
    }

    /// Returns all active property entries for `(subject, property)` in ascending `has_a_id` order.
    #[must_use]
    pub fn property_entries(
        &self,
        subj: DcShorthand,
        prop: DcShorthand,
    ) -> &[BuildPropertyEntry] {
        self.properties_by_subj_prop
            .get(&(subj, prop))
            .map_or(&[], Vec::as_slice)
    }

    /// Returns all active property entries for `(subject, property)` ordered with
    /// `Preferred` (`Dc 570`) entries first, followed by ascending `has_a_id` order.
    #[must_use]
    pub fn property_entries_preferred_first(
        &self,
        subj: DcShorthand,
        prop: DcShorthand,
    ) -> Vec<BuildPropertyEntry> {
        let mut entries = self.property_entries(subj, prop).to_vec();
        entries.sort_by_key(|e| (!e.is_preferred, e.has_a_id));
        entries
    }

    /// Returns the latest active property value token for a single-valued property `(subject, property)`.
    #[must_use]
    pub fn single_property(
        &self,
        subj: DcShorthand,
        prop: DcShorthand,
    ) -> Option<DcShorthand> {
        self.property_entries(subj, prop).last().map(|e| e.value)
    }

    /// Returns the latest active UTF-8 string value for `(subject, property)`.
    #[must_use]
    pub fn single_property_string(
        &self,
        subj: DcShorthand,
        prop: DcShorthand,
    ) -> Option<&str> {
        let val = self.single_property(subj, prop)?;
        self.get_shorthand_string(val)
    }

    /// Returns all active UTF-8 string values for `(subject, property)` with `Preferred` entries first.
    #[must_use]
    pub fn multi_property_strings_preferred_first(
        &self,
        subj: DcShorthand,
        prop: DcShorthand,
    ) -> Vec<String> {
        self.property_entries_preferred_first(subj, prop)
            .into_iter()
            .filter_map(|e| {
                self.get_shorthand_string(e.value).map(str::to_owned)
            })
            .collect()
    }

    /// Returns all qualifier value tokens attached to `claim_id` via `qual_prop`.
    #[must_use]
    pub fn claim_qualifiers(
        &self,
        claim_id: u128,
        qual_prop: DcShorthand,
    ) -> &[DcShorthand] {
        self.qualifiers_by_claim_prop
            .get(&(claim_id, qual_prop))
            .map_or(&[], Vec::as_slice)
    }

    /// Returns all direct binary statement objects `obj` for `(verb, subj)` in ascending node ID order.
    #[must_use]
    pub fn direct_objects(
        &self,
        verb: DcShorthand,
        subj: DcShorthand,
    ) -> &[DcShorthand] {
        self.direct_objects_by_verb_subj
            .get(&(verb, subj))
            .map_or(&[], Vec::as_slice)
    }

    /// Returns the script name (e.g. `".EL Types"`, `".Formats:compression"`) for `subj` if declared via `IsInstanceOf`.
    #[must_use]
    pub fn subject_script(&self, subj: DcShorthand) -> Option<&str> {
        let sh_inst = self.framing.is_instance_of;
        for &obj in self.direct_objects(sh_inst, subj) {
            if let DcShorthand::Long(seed_id) = obj {
                if let Some(BuildSeedEntity::Script(s)) =
                    self.seed_entity(seed_id)
                {
                    return Some(s.as_str());
                }
            }
        }
        None
    }

    /// Returns the general category code (e.g. `"!Cx"`, `"Cc"`, `"Zl"`) for `subj` if declared via `IsInstanceOf`.
    #[must_use]
    pub fn subject_general_category(&self, subj: DcShorthand) -> Option<&str> {
        let sh_inst = self.framing.is_instance_of;
        for &obj in self.direct_objects(sh_inst, subj) {
            if let DcShorthand::Long(seed_id) = obj {
                if let Some(BuildSeedEntity::GeneralCategory(gc)) =
                    self.seed_entity(seed_id)
                {
                    return Some(gc.as_str());
                }
            }
        }
        None
    }

    /// Returns all Short Dc IDs (`DcShorthand::Short(id)`) that have an active `CanonicalName` in the graph,
    /// in ascending `short_id` order.
    ///
    /// # Errors
    /// Returns an error if `CanonicalName` is missing from the graph.
    pub fn all_short_dc_ids(&self) -> Result<Vec<u32>> {
        let sh_name = self.require_ident("CanonicalName")?;
        let mut ids: Vec<u32> = self
            .properties_by_subj_prop
            .keys()
            .filter_map(|&(subj, prop)| {
                if prop == sh_name {
                    if let DcShorthand::Short(sid) = subj {
                        return Some(sid);
                    }
                }
                None
            })
            .collect();
        ids.sort_unstable();
        ids.dedup();
        Ok(ids)
    }

    /// Returns all Format short IDs (`DcShorthand::Format(id)`) that have an active `CanonicalName` in the graph,
    /// in ascending `short_id` order.
    ///
    /// # Errors
    /// Returns an error if `CanonicalName` is missing from the graph.
    pub fn all_format_short_ids(&self) -> Result<Vec<u32>> {
        let sh_name = self.require_ident("CanonicalName")?;
        let mut ids: Vec<u32> = Vec::new();
        for &(subj, prop) in self.properties_by_subj_prop.keys() {
            if prop == sh_name {
                if let DcShorthand::Format(fid) = subj {
                    let sid = u32::try_from(fid).with_context(|| {
                        format!("Format short ID {fid} exceeds u32::MAX")
                    })?;
                    ids.push(sid);
                }
            }
        }
        ids.sort_unstable();
        ids.dedup();
        Ok(ids)
    }

    /// Extracts `(general, dc)` support levels for every format supported by `CollectiveToolbox` (`f649`),
    /// returning `(import_support_by_format, export_support_by_format)`.
    ///
    /// # Errors
    /// Returns an error if any required support vocabulary Dc is missing from the graph.
    pub fn collective_toolbox_format_support(
        &self,
    ) -> Result<(
        BTreeMap<DcShorthand, (i8, i8)>,
        BTreeMap<DcShorthand, (i8, i8)>,
    )> {
        let sh_ctb = self.require_ident("CollectiveToolbox")?;
        let sh_gen_in = self.require_ident("GeneralFormatImportSupport")?;
        let sh_dc_in = self.require_ident("DcFormatImportSupport")?;
        let sh_gen_out = self.require_ident("GeneralFormatExportSupport")?;
        let sh_dc_out = self.require_ident("DcFormatExportSupport")?;
        let sh_level_prop =
            self.require_ident("ImplementationSupportLevel")?;

        let sh_na = self.require_ident("SupportNotApplicable")?;
        let sh_wip = self.require_ident("SupportWorkInProgress")?;
        let sh_partial = self.require_ident("SupportPartial")?;
        let sh_complete = self.require_ident("SupportComplete")?;
        let sh_lossless = self.require_ident("SupportLosslessRoundtrip")?;
        let sh_strict =
            self.require_ident("SupportLosslessStrictValidation")?;

        let decode_level = |claim_id: u128| -> Result<i8> {
            let Some(&level_tok) = self
                .claim_qualifiers(claim_id, sh_level_prop)
                .last()
            else {
                bail!(
                    "Support claim l{claim_id} on CollectiveToolbox is missing ImplementationSupportLevel qualifier"
                );
            };
            if level_tok == sh_na {
                Ok(-1)
            } else if level_tok == sh_wip {
                Ok(1)
            } else if level_tok == sh_partial {
                Ok(2)
            } else if level_tok == sh_complete {
                Ok(3)
            } else if level_tok == sh_lossless {
                Ok(4)
            } else if level_tok == sh_strict {
                Ok(5)
            } else {
                bail!(
                    "Unrecognized ImplementationSupportLevel token {level_tok} on claim l{claim_id}"
                )
            }
        };

        let mut import_support: BTreeMap<DcShorthand, (i8, i8)> =
            BTreeMap::new();
        let mut export_support: BTreeMap<DcShorthand, (i8, i8)> =
            BTreeMap::new();

        for entry in self.property_entries(sh_ctb, sh_gen_in) {
            let lvl = decode_level(entry.has_a_id)?;
            import_support.entry(entry.value).or_insert((0, 0)).0 = lvl;
        }
        for entry in self.property_entries(sh_ctb, sh_dc_in) {
            let lvl = decode_level(entry.has_a_id)?;
            import_support.entry(entry.value).or_insert((0, 0)).1 = lvl;
        }
        for entry in self.property_entries(sh_ctb, sh_gen_out) {
            let lvl = decode_level(entry.has_a_id)?;
            export_support.entry(entry.value).or_insert((0, 0)).0 = lvl;
        }
        for entry in self.property_entries(sh_ctb, sh_dc_out) {
            let lvl = decode_level(entry.has_a_id)?;
            export_support.entry(entry.value).or_insert((0, 0)).1 = lvl;
        }

        Ok((import_support, export_support))
    }
}
