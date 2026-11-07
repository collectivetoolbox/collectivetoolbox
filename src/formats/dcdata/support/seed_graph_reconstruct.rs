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

//! Offline CSV definition and `SyntaxPattern` AST reconstruction from `SeedGraphIndex`.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::collections::HashMap;

use ctb_storage_minimal::shorthand::DcShorthand;
use ctb_utilities::format_id::{
    DC_APPLE_OS_TYPE, DC_COLLECTIVE_TOOLBOX, DC_FILE_EXTENSION,
    DC_LINE_ENDING_CR, DC_LINE_ENDING_CR_LF, DC_LINE_ENDING_LF,
    DC_LINE_ENDING_LF_CR, DC_LINE_ENDING_NL, DC_LINE_ENDING_RS, DC_MEDIA_TYPE,
    DC_UNIFORM_TYPE_IDENTIFIER,
};

use crate::dc::{
    DC_ANNOTATION, DC_APPROXIMATE_DECOMPOSITION, DC_BEGIN_ASSOCIATIVITY_GROUP,
    DC_BEGIN_PARAMETER, DC_BIDIRECTIONAL_CLASS, DC_BOOLEAN_TRUE,
    DC_CANONICAL_COMBINING_CLASS, DC_CANONICAL_DECOMPOSITION,
    DC_CANONICAL_NAME, DC_CASE_MAPPING_PARTNER,
    DC_COMPATIBILITY_DECOMPOSITION, DC_CROSS_REFERENCE, DC_DATA_BYTE_LENGTH,
    DC_DATA_FORMAT_DESCRIPTION, DC_DC_FORMAT_EXPORT_SUPPORT,
    DC_DC_FORMAT_IMPORT_SUPPORT, DC_DC_FORMAT_TEST_COVERAGE,
    DC_DC_FORMAT_VARIATION_NICKNAME, DC_DEFAULT_FORMAT_VARIANT, DC_DESCRIPTION,
    DC_END_ASSOCIATIVITY_GROUP, DC_ESCAPE, DC_EXCLUSIVE_RANGE_ENDPOINT,
    DC_FORMAL_ALIAS, DC_FORMAT_PARAM_APPLICATION, DC_FORMAT_VARIATION_ASPECT,
    DC_GENERAL_FORMAT_EXPORT_SUPPORT, DC_GENERAL_FORMAT_IMPORT_SUPPORT,
    DC_GENERAL_FORMAT_TEST_COVERAGE, DC_HAS_A,
    DC_IMPLEMENTATION_SUPPORT_LEVEL, DC_INFORMAL_ALIAS, DC_IS_ASSOCIATED_WITH,
    DC_IS_BASED_ON, DC_IS_CLOSING_DELIMITER_FOR, DC_IS_DEPRECATED,
    DC_IS_INSTANCE_OF, DC_IS_SUBTYPE_OF, DC_MAGIC_SIGNATURE,
    DC_NEGATED_PATTERN, DC_NICKNAME, DC_OPTIONAL_ABSENT,
    DC_PATTERN_ALTERNATION, DC_PATTERN_CAPTURE, DC_PATTERN_REPETITION,
    DC_PATTERN_REPLACEMENT, DC_PROGRAMMATIC_IDENTIFIER, DC_RANGE,
    DC_REFERENCE_BOUND_VALUE, DC_REFERENCE_NAMED_OBJECT, DC_ROUTINE_INVOCATION,
    DC_SELF_REFERENCE, DC_SEMANTIC_DECOMPOSITION,
    DC_START_ENCAPSULATION_BINARY, DC_SUBPATTERN_MATCH, DC_SUPPORT_COMPLETE,
    DC_SUPPORT_LOSSLESS_ROUNDTRIP, DC_SUPPORT_LOSSLESS_STRICT_VALIDATION,
    DC_SUPPORT_NOT_APPLICABLE, DC_SUPPORT_PARTIAL, DC_SUPPORT_WORK_IN_PROGRESS,
    DC_SYNTAX_RULE, DC_TYPE_ATOM_DC, DC_TYPE_CONVERSION, DC_TYPE_INTERSECTION,
    DC_TYPE_NUMBER, DC_TYPE_ROUTINE, DC_TYPE_TRANSFORMATION,
    DC_TYPE_TRANSMUTATION, DC_TYPE_UNION, DC_VALUE_RANGE, DcChar,
};
use crate::dc_def::{BidiClass, DcDefn, FormatDetails, GeneralCategory};
use crate::format_spec::{DcToken, FormatExpr, decode_dc_stream};
use crate::graph_ledger::{NumericBoundValue, SeedEntityKey};
use crate::seed_graph::SeedGraphIndex;
use crate::support::syntax_ast::{
    ActionArg, CharTarget, DcSyntaxRule, NumberSubtypeConstraint, Quantifier,
    SubtypeConstraint, SyntaxAction, SyntaxElement, SyntaxPattern, SyntaxTerm,
};
use crate::support::syntax_parser::format_extension_regex_pattern;

#[must_use]
fn sh(dc: DcChar) -> DcShorthand {
    DcShorthand::from_global_id(dc.to_u128())
}

/// Named syntax construct reconstructed from the System-block seed graph (`README-named-types.csv`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconstructedNamedType {
    /// System-block seed entity node ID (`586 615 565`).
    pub seed_id: u128,
    /// Canonical construct name (`Dc 572` `"Canonical name"`).
    pub name: String,
    /// Decoded syntax rule (`Dc 594` `"Syntax rule"`).
    pub syntax: DcSyntaxRule,
    /// Optional notes / description (`Dc 573` `"Description"`), or empty string if absent.
    pub description: String,
}

impl SeedGraphIndex {
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
                    let payload =
                        self.decode_decomposition_payload(entry.value);
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

            let format_details = if script.starts_with(".Formats:")
                || script.starts_with("Formats:")
            {
                Some(FormatDetails::default())
            } else {
                None
            };

            defns.push(DcDefn {
                dc_id,
                short_id,
                ident,
                stable_ident: self.has_stable_programmatic_identifier(subj),
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
                format: format_details,
                source_file: format!("{category}.csv"),
                line_number: 0,
            });
        }

        Ok(defns)
    }

    /// Reconstructs all Format definitions (`DcShorthand::Format(_)`) from the active seed graph
    /// statements, sorted by ascending `dc_id`.
    pub fn reconstruct_format_defns(&self) -> Result<Vec<DcDefn>> {
        let sh_name = sh(DC_CANONICAL_NAME);
        let sh_desc = sh(DC_DESCRIPTION);
        let sh_ident = sh(DC_PROGRAMMATIC_IDENTIFIER);
        let sh_inst = sh(DC_IS_INSTANCE_OF);
        let sh_dep = sh(DC_IS_DEPRECATED);
        let sh_true = sh(DC_BOOLEAN_TRUE);
        let sh_closes = sh(DC_IS_CLOSING_DELIMITER_FOR);
        let sh_subtype = sh(DC_IS_SUBTYPE_OF);
        let sh_based = sh(DC_IS_BASED_ON);
        let sh_assoc = sh(DC_IS_ASSOCIATED_WITH);
        let sh_nick = sh(DC_NICKNAME);
        let sh_falias = sh(DC_FORMAL_ALIAS);
        let sh_ann = sh(DC_ANNOTATION);
        let sh_magic = sh(DC_MAGIC_SIGNATURE);
        let sh_xref = sh(DC_CROSS_REFERENCE);
        let sh_equiv = sh(DC_CANONICAL_DECOMPOSITION);
        let sh_approx = sh(DC_APPROXIMATE_DECOMPOSITION);
        let sh_sem = sh(DC_SEMANTIC_DECOMPOSITION);
        let sh_compat = sh(DC_COMPATIBILITY_DECOMPOSITION);
        let sh_fmt_desc = sh(DC_DATA_FORMAT_DESCRIPTION);
        let sh_ext = sh(DC_FILE_EXTENSION);
        let sh_mime = sh(DC_MEDIA_TYPE);
        let sh_uti = sh(DC_UNIFORM_TYPE_IDENTIFIER);
        let sh_ostype = sh(DC_APPLE_OS_TYPE);
        let sh_var_aspect = sh(DC_FORMAT_VARIATION_ASPECT);
        let sh_var_nick = sh(DC_DC_FORMAT_VARIATION_NICKNAME);
        let sh_def_variant = sh(DC_DEFAULT_FORMAT_VARIANT);
        let sh_ctb = sh(DC_COLLECTIVE_TOOLBOX);
        let sh_supp_level = sh(DC_IMPLEMENTATION_SUPPORT_LEVEL);

        let mut support_by_cap_and_fmt: HashMap<
            (DcShorthand, DcShorthand),
            i8,
        > = HashMap::new();
        for cap_dc in [
            DC_GENERAL_FORMAT_IMPORT_SUPPORT,
            DC_DC_FORMAT_IMPORT_SUPPORT,
            DC_GENERAL_FORMAT_EXPORT_SUPPORT,
            DC_DC_FORMAT_EXPORT_SUPPORT,
            DC_GENERAL_FORMAT_TEST_COVERAGE,
            DC_DC_FORMAT_TEST_COVERAGE,
        ] {
            let cap_sh = sh(cap_dc);
            for entry in self.get_multi_properties(sh_ctb, cap_sh) {
                if let Some(level_tok) = self.get_single_property(
                    DcShorthand::Long(entry.has_a_id),
                    sh_supp_level,
                ) {
                    if let Some(level) = dc_to_support_level(level_tok) {
                        support_by_cap_and_fmt
                            .insert((cap_sh, entry.value), level);
                    }
                }
            }
        }

        let mut subjects: Vec<DcShorthand> = self
            .active_single_props
            .keys()
            .filter_map(|&(subj, prop)| {
                if prop == sh_name && matches!(subj, DcShorthand::Format(_)) {
                    Some(subj)
                } else {
                    None
                }
            })
            .collect();
        subjects.sort_by_key(|s| {
            // Reason for fallback: Format shorthands always have valid global IDs.
            s.to_global_id().unwrap_or(0)
        });
        subjects.dedup();

        let mut defns = Vec::with_capacity(subjects.len());
        for subj in subjects {
            let dc_id = subj.to_global_id()?;
            let short_id = match subj {
                DcShorthand::Format(sid) => usize::try_from(sid).ok(),
                _ => None,
            };
            // Reason for fallback: subjects were selected from active_single_props by sh_name.
            let name = self
                .get_single_property_string(subj, sh_name)
                .unwrap_or_default()
                .to_string();
            // Reason for fallback: formats without an explicit description default to empty string.
            let description = self
                .get_single_property_string(subj, sh_desc)
                .unwrap_or_default()
                .to_string();
            let ident = self
                .get_single_property_string(subj, sh_ident)
                .map(str::to_string);

            let mut script = String::new();
            for &obj in self.get_direct_objects(sh_inst, subj) {
                if let DcShorthand::Long(seed_id) = obj {
                    if let Some(SeedEntityKey::Script(s)) =
                        self.get_seed_entity(seed_id)
                    {
                        script.clone_from(s);
                    }
                }
            }

            let category = category_for_dc_subject(subj, &script);
            let is_deprecated =
                self.has_direct_statement(sh_dep, subj, sh_true);
            let closes =
                self.get_direct_objects(sh_closes, subj).first().copied();
            let syntax = self.decode_syntax_rule_for_subject(subj)?;

            let aliases: Vec<String> = self
                .get_multi_properties(subj, sh_nick)
                .iter()
                .filter_map(|e| self.get_string(e.value).map(str::to_string))
                .collect();
            let title: Option<String> = self
                .get_multi_properties(subj, sh_falias)
                .iter()
                .find_map(|e| self.get_string(e.value).map(str::to_string));
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
                    let payload =
                        self.decode_decomposition_payload(entry.value);
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

            let format_spec = if let Some(DcShorthand::Long(expr_node_id)) =
                self.get_single_property(subj, sh_fmt_desc)
            {
                Some(self.decode_format_expr_node(expr_node_id)?)
            } else {
                None
            };
            let chain = None;

            let ext_items: Vec<String> = self
                .get_multi_properties(subj, sh_ext)
                .iter()
                .filter_map(|e| {
                    if let Some(s) = self.get_string(e.value) {
                        Some(s.to_string())
                    } else if let DcShorthand::Long(pat_id) = e.value {
                        self.decode_pattern_node(pat_id).ok().and_then(|r| {
                            format_extension_regex_pattern(&r.pattern)
                        })
                    } else {
                        None
                    }
                })
                .collect();
            let extensions = if ext_items.is_empty() {
                None
            } else {
                Some(ext_items.join(", "))
            };

            let mime_items: Vec<String> = self
                .get_multi_properties(subj, sh_mime)
                .iter()
                .filter_map(|e| self.get_string(e.value).map(str::to_string))
                .collect();
            let mime = if mime_items.is_empty() {
                None
            } else {
                Some(mime_items.join(", "))
            };

            let uti_items: Vec<String> = self
                .get_multi_properties(subj, sh_uti)
                .iter()
                .filter_map(|e| self.get_string(e.value).map(str::to_string))
                .collect();
            let uti = if uti_items.is_empty() {
                None
            } else {
                Some(uti_items.join(", "))
            };

            let apple_items: Vec<String> = self
                .get_multi_properties(subj, sh_ostype)
                .iter()
                .filter_map(|e| self.get_string(e.value).map(str::to_string))
                .collect();
            let apple_type = if apple_items.is_empty() {
                None
            } else {
                Some(apple_items.join(", "))
            };

            let nicknames = if aliases.is_empty() {
                None
            } else {
                Some(aliases.join(", "))
            };

            let format_support_pair =
                |gen_dc: DcChar, dc_dc: DcChar| -> Option<String> {
                    // Reason for fallback: unannotated capability defaults to support level 0.
                    let gen_lvl = support_by_cap_and_fmt
                        .get(&(sh(gen_dc), subj))
                        .copied()
                        .unwrap_or(0);
                    // Reason for fallback: unannotated capability defaults to support level 0.
                    let dc_lvl = support_by_cap_and_fmt
                        .get(&(sh(dc_dc), subj))
                        .copied()
                        .unwrap_or(0);
                    if gen_lvl == 0 && dc_lvl == 0 {
                        None
                    } else {
                        Some(format!("{gen_lvl} {dc_lvl}"))
                    }
                };
            let import_support = format_support_pair(
                DC_GENERAL_FORMAT_IMPORT_SUPPORT,
                DC_DC_FORMAT_IMPORT_SUPPORT,
            );
            let export_support = format_support_pair(
                DC_GENERAL_FORMAT_EXPORT_SUPPORT,
                DC_DC_FORMAT_EXPORT_SUPPORT,
            );
            let tests = format_support_pair(
                DC_GENERAL_FORMAT_TEST_COVERAGE,
                DC_DC_FORMAT_TEST_COVERAGE,
            );

            let mut varies = Vec::new();
            let mut variant_type_items = Vec::new();
            let vary_as = self
                .get_single_property_string(subj, sh_var_nick)
                .map(str::to_string);
            for entry in self.get_multi_properties(subj, sh_var_aspect) {
                if let DcShorthand::Format(_) = entry.value {
                    if let Some(nick) = self
                        .get_single_property_string(entry.value, sh_var_nick)
                    {
                        let v_script = format!(".Formats:v:{nick}");
                        let direct_script = format!(".Formats:{nick}");
                        if self
                            .seed_entity_ids
                            .contains_key(&SeedEntityKey::Script(v_script))
                            || self.seed_entity_ids.contains_key(
                                &SeedEntityKey::Script(direct_script),
                            )
                        {
                            variant_type_items.push(nick.to_string());
                        } else {
                            varies.push(entry.value);
                        }
                    } else {
                        varies.push(entry.value);
                    }
                }
            }
            let variant_types = if variant_type_items.is_empty() {
                None
            } else {
                Some(variant_type_items.join(", "))
            };

            let default_line_ending =
                match self.get_single_property(subj, sh_def_variant) {
                    Some(tok) if tok == sh(DC_LINE_ENDING_LF) => {
                        Some("lf".to_string())
                    }
                    Some(tok) if tok == sh(DC_LINE_ENDING_CR) => {
                        Some("cr".to_string())
                    }
                    Some(tok) if tok == sh(DC_LINE_ENDING_CR_LF) => {
                        Some("crlf".to_string())
                    }
                    Some(tok) if tok == sh(DC_LINE_ENDING_LF_CR) => {
                        Some("lfcr".to_string())
                    }
                    Some(tok) if tok == sh(DC_LINE_ENDING_RS) => {
                        Some("rs".to_string())
                    }
                    Some(tok) if tok == sh(DC_LINE_ENDING_NL) => {
                        Some("nl".to_string())
                    }
                    _ => None,
                };

            let fmt_implies = if implies.is_empty() {
                None
            } else {
                Some(implies.join(", "))
            };
            let fmt_based_on = if based_on.is_empty() {
                None
            } else {
                Some(based_on.join(", "))
            };

            let format_details = FormatDetails {
                implies: fmt_implies,
                based_on: fmt_based_on,
                chain,
                format_spec,
                extensions,
                mime,
                uti,
                apple_type,
                nicknames,
                import_support,
                export_support,
                tests,
                variant_types,
                title: title.clone(),
                default_line_ending,
                vary_as,
                varies,
            };

            defns.push(DcDefn {
                dc_id,
                short_id,
                ident,
                stable_ident: self.has_stable_programmatic_identifier(subj),
                title,
                name,
                category: category.clone(),
                combining_class: 0,
                bidi_class: BidiClass::BN,
                casing_partner: None,
                general_category: GeneralCategory::NonUnicodeControl,
                script,
                is_deprecated,
                decompositions,
                aliases,
                formal_aliases: Vec::new(),
                cross_references,
                annotations,
                implies,
                based_on,
                os_associations,
                magic_names,
                syntax,
                closes,
                description,
                format: Some(format_details),
                source_file: format!("{category}.csv"),
                line_number: 0,
            });
        }

        Ok(defns)
    }

    /// Reconstructs all active named syntax constructs (`SeedEntityKey::NamedConstruct`)
    /// that define a `Dc 594` (`Syntax rule`) in the seed graph, ordered by ascending
    /// entity node ID.
    pub fn reconstruct_named_types(&self) -> Result<Vec<ReconstructedNamedType>> {
        let sh_desc = sh(DC_DESCRIPTION);
        let mut constructs: Vec<(u128, String)> = self
            .seed_entity_ids
            .iter()
            .filter_map(|(key, &seed_id)| match key {
                SeedEntityKey::NamedConstruct(name) => {
                    Some((seed_id, name.clone()))
                }
                _ => None,
            })
            .collect();
        constructs.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

        let mut out = Vec::with_capacity(constructs.len());
        for (seed_id, name) in constructs {
            let subj = DcShorthand::Long(seed_id);
            let Some(syntax) = self.decode_syntax_rule_for_subject(subj)?
            else {
                continue;
            };
            // Reason for fallback: named constructs without an explicit Dc 573 description default to empty string.
            let description = self
                .get_single_property_string(subj, sh_desc)
                .unwrap_or_default()
                .to_string();
            out.push(ReconstructedNamedType {
                seed_id,
                name,
                syntax,
                description,
            });
        }
        Ok(out)
    }

    /// Decodes a `FormatExpr` value node by its `node_id`.
    pub fn decode_format_expr_node(&self, node_id: u128) -> Result<FormatExpr> {
        let node = self.get_node(node_id).with_context(|| {
            format!("FormatExpr value node l{node_id} not found in seed graph")
        })?;
        let sh_var_nick = sh(DC_DC_FORMAT_VARIATION_NICKNAME);
        let sh_begin_param = sh(DC_BEGIN_PARAMETER);
        let mut dc_tokens = Vec::with_capacity(node.tokens.len());
        let mut prev_tok = None;
        for &tok in &node.tokens {
            if prev_tok == Some(sh_begin_param) {
                if let Some(nick) =
                    self.get_single_property_string(tok, sh_var_nick)
                {
                    dc_tokens.push(DcToken::NamedType(nick.to_string()));
                    prev_tok = Some(tok);
                    continue;
                }
            }
            if let DcShorthand::Long(id) = tok {
                if let Some(SeedEntityKey::NamedConstruct(name)) =
                    self.get_seed_entity(id)
                {
                    dc_tokens.push(DcToken::NamedType(name.clone()));
                } else if let Some(int_val) = self.get_integer(tok) {
                    dc_tokens.push(DcToken::NamedType(int_val.to_string()));
                } else if let Some(str_val) = self.get_string(tok) {
                    dc_tokens.push(DcToken::NamedType(str_val.to_string()));
                } else {
                    dc_tokens.push(DcToken::Dc(tok));
                }
            } else {
                dc_tokens.push(DcToken::Dc(tok));
            }
            prev_tok = Some(tok);
        }
        decode_dc_stream(&dc_tokens).with_context(|| {
            format!("Failed to decode FormatExpr tokens in node l{node_id}")
        })
    }

    fn decode_decomposition_payload(&self, val_tok: DcShorthand) -> String {
        if let Some(s) = self.get_string(val_tok) {
            return s.to_string();
        }
        if let DcShorthand::Long(node_id) = val_tok {
            if let Some(node) = self.get_node(node_id) {
                let is_format_op = |tok: DcShorthand| -> bool {
                    tok == sh(DC_TYPE_UNION)
                        || tok == sh(DC_TYPE_TRANSFORMATION)
                        || tok == sh(DC_TYPE_CONVERSION)
                        || tok == sh(DC_TYPE_TRANSMUTATION)
                        || tok == sh(DC_TYPE_INTERSECTION)
                        || tok == sh(DC_FORMAT_PARAM_APPLICATION)
                };
                let starts_format_expr =
                    node.tokens.first().is_some_and(|&t| {
                        is_format_op(t)
                            || (t == sh(DC_BEGIN_ASSOCIATIVITY_GROUP)
                                && node
                                    .tokens
                                    .get(1)
                                    .copied()
                                    .is_some_and(is_format_op))
                    });
                if starts_format_expr {
                    if let Ok(expr) = self.decode_format_expr_node(node_id) {
                        return crate::format_spec::format_expr(&expr);
                    }
                }
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
            if let Some(node) = self.get_node(node_id) {
                let is_format_op = |tok: DcShorthand| -> bool {
                    tok == sh(DC_TYPE_UNION)
                        || tok == sh(DC_TYPE_TRANSFORMATION)
                        || tok == sh(DC_TYPE_CONVERSION)
                        || tok == sh(DC_TYPE_TRANSMUTATION)
                        || tok == sh(DC_TYPE_INTERSECTION)
                        || tok == sh(DC_FORMAT_PARAM_APPLICATION)
                };
                let starts_format_expr =
                    node.tokens.first().is_some_and(|&t| {
                        is_format_op(t)
                            || (t == sh(DC_BEGIN_ASSOCIATIVITY_GROUP)
                                && node
                                    .tokens
                                    .get(1)
                                    .copied()
                                    .is_some_and(is_format_op))
                    });
                if starts_format_expr {
                    if let Ok(expr) = self.decode_format_expr_node(node_id) {
                        return crate::format_spec::format_expr(&expr);
                    }
                }
            }
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
        let Some(val_tok) = self.get_single_property(subj, sh(DC_SYNTAX_RULE))
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
            bail!(
                "Unbalanced EndAssociativityGroup (299) in pattern token stream"
            );
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

    /// Strips any trailing `298 607 ... 299` (`Pattern replacement`) action group from `tokens`.
    pub fn strip_pattern_action_tokens<'a>(
        &self,
        tokens: &'a [DcShorthand],
    ) -> Result<&'a [DcShorthand]> {
        let (pattern_slice, _) = self.strip_trailing_action(tokens)?;
        Ok(pattern_slice)
    }

    /// Returns true if the `.Semantic:Pattern` token sequence begins with `Dc 599 "Self reference"`
    /// (or an alternation whose first branch begins with `Dc 599 "Self reference"`).
    #[must_use]
    pub fn pattern_tokens_start_with_self(
        &self,
        tokens: &[DcShorthand],
    ) -> bool {
        let sh_self = sh(DC_SELF_REFERENCE);
        if tokens.first() == Some(&sh_self) {
            return true;
        }
        if let Ok(Some(branches)) = self.split_alternation_branches(tokens) {
            if let Some(first_branch) = branches.first() {
                return self.pattern_tokens_start_with_self(first_branch);
            }
        }
        false
    }

    /// If `tokens` is a top-level `298 602 (branch1) (branch2) ... 299` alternation, returns
    /// the inner token slices for each branch without allocating a recursive AST.
    pub fn split_alternation_branches<'a>(
        &self,
        tokens: &'a [DcShorthand],
    ) -> Result<Option<Vec<&'a [DcShorthand]>>> {
        let sh_open = sh(DC_BEGIN_ASSOCIATIVITY_GROUP);
        let sh_alt = sh(DC_PATTERN_ALTERNATION);

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
                            branches.push(branch_tokens);
                            idx = branch_close.saturating_add(1);
                        }
                        return Ok(Some(branches));
                    }
                }
            }
        }
        Ok(None)
    }

    /// Decodes a single quantified [`SyntaxElement`] at `idx` from a `.Semantic:Pattern` token slice,
    /// returning `(element, next_idx)`.
    pub fn step_pattern_element_tokens(
        &self,
        tokens: &[DcShorthand],
        idx: usize,
    ) -> Result<(SyntaxElement, usize)> {
        let (term, after_term) = self.decode_syntax_term_at(tokens, idx)?;
        let (quantifier, after_quant) =
            self.decode_optional_quantifier_at(tokens, after_term)?;
        Ok((SyntaxElement::with_quantifier(term, quantifier), after_quant))
    }

    fn decode_syntax_pattern_tokens(
        &self,
        tokens: &[DcShorthand],
    ) -> Result<SyntaxPattern> {
        if let Some(branch_slices) = self.split_alternation_branches(tokens)? {
            let mut branches = Vec::with_capacity(branch_slices.len());
            for branch_tokens in branch_slices {
                branches
                    .push(self.decode_syntax_pattern_tokens(branch_tokens)?);
            }
            return Ok(SyntaxPattern::Alternation(branches));
        }

        let mut elements = Vec::new();
        let mut idx = 0usize;
        while idx < tokens.len() {
            let (elem, after_quant) =
                self.step_pattern_element_tokens(tokens, idx)?;
            elements.push(elem);
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
        let sh_value_range = sh(DC_VALUE_RANGE);
        let sh_atom_dc = sh(DC_TYPE_ATOM_DC);
        let sh_open = sh(DC_BEGIN_ASSOCIATIVITY_GROUP);
        let sh_neg = sh(DC_NEGATED_PATTERN);
        let sh_alt = sh(DC_PATTERN_ALTERNATION);
        let sh_sub = sh(DC_SUBPATTERN_MATCH);
        let sh_type_union = sh(DC_TYPE_UNION);
        let sh_has_a = sh(DC_HAS_A);
        let sh_capture = sh(DC_PATTERN_CAPTURE);

        let tok = *tokens
            .get(idx)
            .context("Unexpected end of pattern token stream")?;

        if tok == sh_self {
            return Ok((SyntaxTerm::SelfChar, idx.saturating_add(1)));
        }

        if tok == sh_value_range
            && tokens.get(idx.saturating_add(1)) == Some(&sh_atom_dc)
        {
            let mut next_idx = idx.saturating_add(2);
            let start = consume_pattern_char_target(tokens, &mut next_idx)
                .context(
                    "Missing start token after Range (Dc 643) AtomDc (Dc 640)",
                )?;
            let end = consume_pattern_char_target(tokens, &mut next_idx)
                .context(
                    "Missing end token after Range (Dc 643) AtomDc (Dc 640)",
                )?;
            return Ok((SyntaxTerm::CharRange { start, end }, next_idx));
        }

        if tok == sh_range {
            let mut next_idx = idx.saturating_add(1);
            let start = consume_pattern_char_target(tokens, &mut next_idx)
                .context("Missing start token after Range (Dc 4)")?;
            let end = consume_pattern_char_target(tokens, &mut next_idx)
                .context("Missing end token after Range (Dc 4)")?;
            return Ok((SyntaxTerm::CharRange { start, end }, next_idx));
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
                let mut m_idx = 1usize;
                while m_idx < inner.len() {
                    members.push(consume_pattern_char_target(
                        inner, &mut m_idx,
                    )?);
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
                let mut m_idx = 1usize;
                while m_idx < inner.len() {
                    members.push(consume_pattern_char_target(
                        inner, &mut m_idx,
                    )?);
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
                if target_tok == sh_type_union {
                    let construct_tok = *inner.get(2).context(
                        "Missing construct token after TypeUnion (300) in SubpatternMatch (605)",
                    )?;
                    let DcShorthand::Long(seed_id) = construct_tok else {
                        bail!(
                            "Expected Long seed entity ID after TypeUnion (300) in SubpatternMatch (605), found {construct_tok}"
                        );
                    };
                    let Some(SeedEntityKey::NamedConstruct(name)) =
                        self.get_seed_entity(seed_id)
                    else {
                        bail!(
                            "Expected NamedConstruct seed entity l{seed_id} in SubpatternMatch (605) TypeUnion (300)"
                        );
                    };
                    let (constraint_tokens, rem) = if inner.get(3)
                        == Some(&sh_open)
                    {
                        let group_close = find_matching_group_close(inner, 3)?
                            .context(
                                "Unclosed nested pattern group in NamedConstruct subtype",
                            )?;
                        let c_slice = inner
                            .get(3..=group_close)
                            .context("Invalid nested subtype group bounds")?;
                        let r_slice = inner
                            .get(group_close.saturating_add(1)..)
                            .context("Invalid trailing subtype slice bounds")?;
                        (c_slice, r_slice)
                    } else if inner.len() >= 5
                        && inner.get(inner.len().saturating_sub(2))
                            == Some(&sh_capture)
                    {
                        let split_at = inner.len().saturating_sub(2);
                        let c_slice = inner
                            .get(3..split_at)
                            .context("Invalid subtype token slice")?;
                        let r_slice = inner
                            .get(split_at..)
                            .context("Invalid capture token slice")?;
                        (c_slice, r_slice)
                    } else {
                        let c_slice = inner
                            .get(3..)
                            .context("Missing subtype constraint tokens")?;
                        (c_slice, &[][..])
                    };
                    let sub_constraint = self
                        .decode_named_construct_subtype_tokens(
                            name,
                            constraint_tokens,
                        )?;
                    let capture_var = if rem.is_empty() {
                        None
                    } else {
                        ensure!(
                            rem.len() == 2 && rem.first() == Some(&sh_capture),
                            "Unexpected trailing tokens after NamedConstruct subtype: {rem:?}"
                        );
                        let var_tok = *rem.get(1).context(
                            "Missing capture variable token after PatternCapture (606)",
                        )?;
                        let var_str = self
                            .get_string(var_tok)
                            .with_context(|| {
                                format!(
                                    "NamedConstruct capture variable token {var_tok} is not a string node"
                                )
                            })?
                            .to_string();
                        Some(var_str)
                    };
                    return Ok((
                        SyntaxTerm::NamedConstruct {
                            name: name.clone(),
                            subtype: Some(sub_constraint),
                            capture_var,
                        },
                        next_idx,
                    ));
                }
                if let DcShorthand::Long(seed_id) = target_tok {
                    match self.get_seed_entity(seed_id) {
                        Some(SeedEntityKey::Script(script_name)) => {
                            let mut capture_var = None;
                            let mut pos = 2usize;
                            while pos < inner.len() {
                                let qual = *inner.get(pos).context(
                                    "Unexpected end of SubpatternMatch",
                                )?;
                                if qual == sh_capture {
                                    let var_tok = *inner
                                        .get(pos.saturating_add(1))
                                        .context(
                                            "Missing capture variable string node after PatternCapture (606)",
                                        )?;
                                    let var_str = self
                                        .get_string(var_tok)
                                        .with_context(|| {
                                            format!(
                                                "Script capture variable token {var_tok} is not a string node"
                                            )
                                        })?
                                        .to_string();
                                    capture_var = Some(var_str);
                                    pos = pos.saturating_add(2);
                                } else {
                                    bail!(
                                        "Unexpected qualifier token {qual} in Script SubpatternMatch"
                                    );
                                }
                            }
                            return Ok((
                                SyntaxTerm::NamedConstruct {
                                    name: "script".to_string(),
                                    subtype: Some(SubtypeConstraint::Target(
                                        script_name.clone(),
                                    )),
                                    capture_var,
                                },
                                next_idx,
                            ));
                        }
                        Some(SeedEntityKey::NamedConstruct(name)) => {
                            let mut subtype = None;
                            let mut capture_var = None;
                            let mut pos = 2usize;
                            while pos < inner.len() {
                                let qual = *inner.get(pos).context(
                                    "Unexpected end of SubpatternMatch",
                                )?;
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
                                        })?;
                                    subtype = Some(SubtypeConstraint::parse(
                                        name, sub_str,
                                    )?);
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
                        _ => {}
                    }
                }
                let mut r_idx = 1usize;
                let target = consume_pattern_char_target(inner, &mut r_idx)?;
                ensure!(
                    r_idx == inner.len(),
                    "Unexpected extra tokens in RuleRef SubpatternMatch: {inner:?}"
                );
                return Ok((SyntaxTerm::RuleRef { target }, next_idx));
            }

            let group_pat = self.decode_syntax_pattern_tokens(inner)?;
            return Ok((SyntaxTerm::Group(group_pat), next_idx));
        }

        let mut next_idx = idx;
        let target = consume_pattern_char_target(tokens, &mut next_idx)?;
        Ok((SyntaxTerm::CharRef(target), next_idx))
    }

    fn decode_named_construct_subtype_tokens(
        &self,
        name: &str,
        tokens: &[DcShorthand],
    ) -> Result<SubtypeConstraint> {
        ensure!(
            !tokens.is_empty(),
            "Empty subtype constraint token slice for [{name}]"
        );
        let sh_open = sh(DC_BEGIN_ASSOCIATIVITY_GROUP);
        let sh_close = sh(DC_END_ASSOCIATIVITY_GROUP);
        if tokens.first() == Some(&sh_open) {
            ensure!(
                tokens.last() == Some(&sh_close) && tokens.len() >= 2,
                "Unbalanced group in [{name}] nested pattern subtype: {tokens:?}"
            );
            let inner = tokens
                .get(1..tokens.len().saturating_sub(1))
                .context("Invalid nested subtype slice")?;
            let pat = self.decode_syntax_pattern_tokens(inner)?;
            return Ok(SubtypeConstraint::Pattern(Box::new(pat)));
        }
        if tokens.len() == 2
            && tokens.first() == Some(&sh(DC_REFERENCE_BOUND_VALUE))
        {
            let var_tok = *tokens.get(1).context("Missing bound value token")?;
            let var_name = self.get_string(var_tok).with_context(|| {
                format!(
                    "Bound variable token {var_tok} in [{name}] subtype is not a string node"
                )
            })?;
            return Ok(SubtypeConstraint::BoundVar(var_name.to_string()));
        }
        if name == "data" {
            if tokens.len() == 2
                && tokens.first() == Some(&sh(DC_DATA_BYTE_LENGTH))
            {
                let len_tok =
                    *tokens.get(1).context("Missing DataByteLength value")?;
                let bytes_u128 = self.get_integer(len_tok).with_context(|| {
                    format!(
                        "DataByteLength token {len_tok} in [data] subtype is not an integer node"
                    )
                })?;
                let bytes = usize::try_from(bytes_u128).context(
                    "DataByteLength in [data] subtype overflows usize",
                )?;
                return Ok(SubtypeConstraint::Data {
                    format: None,
                    format_id: None,
                    bytes: Some(bytes),
                });
            }
            if tokens.len() == 4
                && tokens.first() == Some(&sh(DC_TYPE_UNION))
                && tokens.get(2) == Some(&sh(DC_DATA_BYTE_LENGTH))
            {
                let fmt_tok =
                    *tokens.get(1).context("Missing format token in [data]")?;
                let len_tok =
                    *tokens.get(3).context("Missing DataByteLength value")?;
                let bytes_u128 = self.get_integer(len_tok).with_context(|| {
                    format!(
                        "DataByteLength token {len_tok} in [data] subtype is not an integer node"
                    )
                })?;
                let bytes = usize::try_from(bytes_u128).context(
                    "DataByteLength in [data] subtype overflows usize",
                )?;
                let fmt_str = self.format_stable_or_shorthand_token(fmt_tok);
                let format_id = fmt_tok
                    .to_global_id()
                    .ok()
                    .and_then(ctb_utilities::format_id::FormatId::from_dc_id);
                return Ok(SubtypeConstraint::Data {
                    format: Some(fmt_str),
                    format_id,
                    bytes: Some(bytes),
                });
            }
            if let [single] = tokens {
                let format_id = single
                    .to_global_id()
                    .ok()
                    .and_then(ctb_utilities::format_id::FormatId::from_dc_id);
                return Ok(SubtypeConstraint::Data {
                    format: Some(self.format_stable_or_shorthand_token(*single)),
                    format_id,
                    bytes: None,
                });
            }
            bail!(
                "Unsupported token sequence for [data:...] subtype: {tokens:?}"
            );
        }
        if name == "number" {
            return Ok(SubtypeConstraint::Number(
                self.decode_number_subtype_tokens(tokens)?,
            ));
        }
        if tokens.len() == 4
            && tokens.first() == Some(&sh(DC_VALUE_RANGE))
            && tokens.get(1) == Some(&DcShorthand::Format(200))
        {
            let min_tok = *tokens.get(2).context("Missing range min token")?;
            let max_tok = *tokens.get(3).context("Missing range max token")?;
            let min = self.get_integer(min_tok).with_context(|| {
                format!(
                    "Range min token {min_tok} in [{name}] is not an integer node"
                )
            })?;
            let max = self.get_integer(max_tok).with_context(|| {
                format!(
                    "Range max token {max_tok} in [{name}] is not an integer node"
                )
            })?;
            return Ok(SubtypeConstraint::LengthRange { min, max });
        }
        if let [tok] = tokens {
            if *tok == sh(DC_TYPE_ROUTINE) {
                return Ok(SubtypeConstraint::Target("Routine".to_string()));
            }
            if let DcShorthand::Long(seed_id) = *tok {
                match self.get_seed_entity(seed_id) {
                    Some(SeedEntityKey::NamedConstruct(c)) => {
                        return Ok(SubtypeConstraint::Target(c.clone()));
                    }
                    Some(SeedEntityKey::Script(s)) => {
                        if name == "format"
                            || name == "type"
                            || name == "typed_data"
                        {
                            // Reason for fallback: scripts outside `.Formats:` strip only leading dot.
                            return Ok(SubtypeConstraint::Target(
                                s.strip_prefix(".Formats:")
                                    .or_else(|| s.strip_prefix('.'))
                                    .unwrap_or(s)
                                    .to_string(),
                            ));
                        }
                        // Reason for fallback: scripts without leading dot are returned as-is.
                        return Ok(SubtypeConstraint::Target(
                            s.strip_prefix('.').unwrap_or(s).to_string(),
                        ));
                    }
                    other => {
                        bail!(
                            "Unexpected seed entity {other:?} (l{seed_id}) in [{name}] subtype"
                        );
                    }
                }
            }
            return Ok(SubtypeConstraint::Target(tok.to_string()));
        }
        bail!(
            "Unsupported subtype constraint tokens for [{name}]: {tokens:?}"
        )
    }

    fn decode_number_subtype_tokens(
        &self,
        tokens: &[DcShorthand],
    ) -> Result<NumberSubtypeConstraint> {
        if let [single] = tokens {
            ensure!(
                matches!(*single, DcShorthand::Format(_)),
                "Unexpected single token {single} in [number:...] subtype"
            );
            let format_id = single
                .to_global_id()
                .ok()
                .and_then(ctb_utilities::format_id::FormatId::from_dc_id);
            return Ok(NumberSubtypeConstraint {
                format: Some(self.format_stable_or_shorthand_token(*single)),
                format_id,
                lower_bound: None,
                upper_bound: None,
            });
        }
        ensure!(
            tokens.first() == Some(&sh(DC_VALUE_RANGE)) && tokens.len() >= 4,
            "Expected Range (Dc 643) in [number:...] subtype tokens, found {tokens:?}"
        );
        let domain_tok = *tokens.get(1).context(
            "Missing domain type token in [number:...] Range (Dc 643)",
        )?;
        let mut idx = 2usize;
        let lower_bound = self.consume_number_range_endpoint(tokens, &mut idx)?;
        let upper_bound = self.consume_number_range_endpoint(tokens, &mut idx)?;
        ensure!(
            idx == tokens.len(),
            "Unexpected trailing tokens in [number:...] Range (Dc 643): {tokens:?}"
        );

        let (format, format_id) = match domain_tok {
            DcShorthand::Format(_) => (
                Some(self.format_stable_or_shorthand_token(domain_tok)),
                domain_tok
                    .to_global_id()
                    .ok()
                    .and_then(ctb_utilities::format_id::FormatId::from_dc_id),
            ),
            tok if tok == sh(DC_TYPE_NUMBER) => (None, None),
            other => {
                bail!(
                    "Unexpected domain type token {other} in [number:...] Range (Dc 643)"
                );
            }
        };
        Ok(NumberSubtypeConstraint {
            format,
            format_id,
            lower_bound,
            upper_bound,
        })
    }

    fn consume_number_range_endpoint(
        &self,
        tokens: &[DcShorthand],
        idx: &mut usize,
    ) -> Result<Option<(bool, NumericBoundValue)>> {
        let tok = *tokens
            .get(*idx)
            .context("Missing range endpoint token in Range (Dc 643)")?;
        if tok == sh(DC_OPTIONAL_ABSENT) {
            *idx = idx.saturating_add(1);
            return Ok(None);
        }
        let (is_exclusive, val_tok) =
            if tok == sh(DC_EXCLUSIVE_RANGE_ENDPOINT) {
                let next_tok = *tokens.get(idx.saturating_add(1)).context(
                    "Missing endpoint value token after ExclusiveRangeEndpoint (Dc 644)",
                )?;
                *idx = idx.saturating_add(2);
                (true, next_tok)
            } else {
                *idx = idx.saturating_add(1);
                (false, tok)
            };
        let bound_val = self.decode_numeric_bound_node(val_tok)?;
        Ok(Some((is_exclusive, bound_val)))
    }

    fn decode_numeric_bound_node(
        &self,
        tok: DcShorthand,
    ) -> Result<NumericBoundValue> {
        if let Some(val) = self.get_integer(tok) {
            return Ok(NumericBoundValue::Unsigned(val));
        }
        let DcShorthand::Long(node_id) = tok else {
            bail!(
                "Expected Long value node ID for numeric range bound, found {tok}"
            );
        };
        let node = self.get_node(node_id).with_context(|| {
            format!("Numeric range bound node l{node_id} not found in seed graph")
        })?;
        NumericBoundValue::decode_tokens(&node.tokens).with_context(|| {
            format!(
                "Failed to decode numeric range bound tokens in node l{node_id}: {:?}",
                node.tokens
            )
        })
    }

    fn decode_optional_quantifier_at(
        &self,
        tokens: &[DcShorthand],
        idx: usize,
    ) -> Result<(Quantifier, usize)> {
        let sh_open = sh(DC_BEGIN_ASSOCIATIVITY_GROUP);
        let sh_rep = sh(DC_PATTERN_REPETITION);
        let sh_range = sh(DC_RANGE);
        let sh_value_range = sh(DC_VALUE_RANGE);
        let sh_absent = sh(DC_OPTIONAL_ABSENT);

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

        let (min, max) = if inner.get(1) == Some(&sh_value_range)
            && inner.get(2) == Some(&DcShorthand::Format(200))
        {
            ensure!(
                inner.len() == 5,
                "Expected 5 tokens in PatternRepetition (603) Range (643) group, found {inner:?}"
            );
            let min_tok = *inner
                .get(3)
                .context("Missing minimum bound in PatternRepetition (603)")?;
            let min_u128 = self.get_integer(min_tok).with_context(|| {
                format!(
                    "PatternRepetition min bound token {min_tok} is not an integer value node"
                )
            })?;
            let min = usize::try_from(min_u128)
                .context("PatternRepetition min bound overflows usize")?;
            let max_tok = *inner
                .get(4)
                .context("Missing maximum bound in PatternRepetition (603)")?;
            let max = if max_tok == sh_absent {
                None
            } else {
                let max_u128 = self.get_integer(max_tok).with_context(|| {
                    format!(
                        "PatternRepetition max bound token {max_tok} is not an integer value node"
                    )
                })?;
                Some(
                    usize::try_from(max_u128).context(
                        "PatternRepetition max bound overflows usize",
                    )?,
                )
            };
            (min, max)
        } else {
            ensure!(
                inner.get(1) == Some(&sh_range),
                "Expected Range (Dc 643 or Dc 4) after PatternRepetition (603)"
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
                    usize::try_from(max_u128).context(
                        "PatternRepetition max bound overflows usize",
                    )?,
                )
            } else {
                ensure!(
                    inner.len() == 3,
                    "Unexpected tokens in PatternRepetition (603): {inner:?}"
                );
                None
            };
            (min, max)
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

pub(crate) fn find_matching_group_close(
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

fn consume_pattern_char_target(
    tokens: &[DcShorthand],
    idx: &mut usize,
) -> Result<CharTarget> {
    let tok = *tokens
        .get(*idx)
        .context("Unexpected end of pattern token stream")?;
    *idx = idx.saturating_add(1);
    if tok == sh(DC_ESCAPE)
        && tokens.get(*idx) == Some(&sh(DC_START_ENCAPSULATION_BINARY))
    {
        let escaped = *tokens
            .get(*idx)
            .context("Unexpected end of pattern token stream after Dc 255")?;
        *idx = idx.saturating_add(1);
        return shorthand_to_char_target(escaped);
    }
    shorthand_to_char_target(tok)
}

fn shorthand_to_char_target(shorthand: DcShorthand) -> Result<CharTarget> {
    let gid = shorthand.to_global_id()?;
    CharTarget::from_u128(gid).with_context(|| {
        format!(
            "Token shorthand '{shorthand}' (GID {gid}) cannot be represented as a CharTarget"
        )
    })
}

fn dc_to_support_level(shorthand: DcShorthand) -> Option<i8> {
    if shorthand == sh(DC_SUPPORT_NOT_APPLICABLE) {
        Some(-1)
    } else if shorthand == sh(DC_SUPPORT_WORK_IN_PROGRESS) {
        Some(1)
    } else if shorthand == sh(DC_SUPPORT_PARTIAL) {
        Some(2)
    } else if shorthand == sh(DC_SUPPORT_COMPLETE) {
        Some(3)
    } else if shorthand == sh(DC_SUPPORT_LOSSLESS_ROUNDTRIP) {
        Some(4)
    } else if shorthand == sh(DC_SUPPORT_LOSSLESS_STRICT_VALIDATION) {
        Some(5)
    } else {
        None
    }
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
