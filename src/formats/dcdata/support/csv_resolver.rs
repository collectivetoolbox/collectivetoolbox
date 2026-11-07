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

//! CSV-based dataset loading for `DatasetRuleResolver` (used for CSV validation,
//! codegen, and seed-graph consistency verification).

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::dc_char::{DcChar, DcCharExt};
use crate::dc_def::DcDefn;
use crate::support::syntax_validator::build_format_validation_registry;
use crate::syntax::ast::CharTarget;
use crate::syntax::parser::parse_dc_syntax;
use crate::syntax::resolver::{DatasetRuleResolver, detect_container_closer};
use std::collections::HashMap;

fn parse_prefer_target(decompositions: &[String]) -> Option<CharTarget> {
    parse_decomposition_target(decompositions, "<prefer>")
}

fn parse_decomposition_target(
    decompositions: &[String],
    marker: &str,
) -> Option<CharTarget> {
    for item in decompositions {
        if let Some(rest) = item.trim().strip_prefix(marker) {
            let trimmed = rest.trim();
            if let Ok(shorthand) =
                ctb_storage_minimal::shorthand::DcShorthand::parse(trimmed)
            {
                return match shorthand {
                    ctb_storage_minimal::shorthand::DcShorthand::Short(id) => {
                        u32::try_from(id).ok().map(CharTarget::Dc)
                    }
                    ctb_storage_minimal::shorthand::DcShorthand::Format(_) => {
                        ctb_utilities::format_id::FormatId::from_shorthand(
                            trimmed,
                        )
                        .map(CharTarget::Format)
                    }
                    ctb_storage_minimal::shorthand::DcShorthand::Unicode(
                        cp,
                    ) => Some(CharTarget::Unicode(cp)),
                    _ => None,
                };
            }
        }
    }
    None
}

impl DatasetRuleResolver {
    /// Builds a resolver by indexing the CSV fixture dataset
    /// in `src/formats/dcdata/data-src/`.
    ///
    /// Used by the consistency test verifying that the seed graph
    /// syntax resolver is up to date with the CSV dataset.
    ///
    /// # Errors
    /// Returns an error if the fixture dataset is missing or invalid.
    pub fn load_from_embedded_dataset() -> Result<Self> {
        let named_types_csv = crate::get_dc_data_file("README-named-types.csv")
            .context("Missing README-named-types.csv fixture")?;
        let named_resolver =
            Self::load_from_dataset(&[], &[], Some(&named_types_csv))?;
        let named_types = named_resolver.named_types.keys().cloned().collect();
        let mut fmt_report = crate::report::ValidationReport::new();
        let formats_dir = crate::get_formats_categories_dir_path();
        let format_defns = crate::format::validate_all_format_files_from_disk(
            &formats_dir,
            None,
            &mut fmt_report,
        );
        fmt_report
            .ensure_valid()
            .context("Invalid format dataset for syntax resolution")?;
        let format_ids = format_defns
            .iter()
            .filter_map(|defn| defn.short_id)
            .collect();
        let format_syntax_ids = format_defns
            .iter()
            .filter(|defn| defn.syntax.is_some())
            .filter_map(|defn| defn.short_id)
            .collect();
        let format_registry = build_format_validation_registry(&format_defns);
        let categories_dir = crate::get_dc_categories_dir_path();
        let mut dc_defns = crate::validation::validate_all_dc_files_from_disk(
            &categories_dir,
            &format_ids,
            None,
            Some(&named_types),
            None,
            Some(&format_syntax_ids),
            Some(&format_registry),
            &mut fmt_report,
        );
        fmt_report
            .ensure_valid()
            .context("Invalid Dc dataset for syntax resolution")?;
        dc_defns.sort_by_key(|defn| defn.dc_id);
        Self::load_from_dataset(
            &dc_defns,
            &format_defns,
            Some(&named_types_csv),
        )
    }

    /// Builds a resolver by indexing syntax rules directly from parsed
    /// dataset definitions.
    ///
    /// This is the CSV-driven authoritative indexing path used by
    /// consistency tests verifying that the seed graph syntax resolver
    /// matches the CSV dataset. `dc_defns` must be sorted by long ID so
    /// delimiter precedence is deterministic (matching
    /// `lookup::get_all_dc_defns`).
    ///
    /// # Errors
    /// Returns an error with CSV row and named type context when a named
    /// type record or its grammar is invalid.
    pub fn load_from_dataset(
        dc_defns: &[DcDefn],
        format_defns: &[DcDefn],
        named_types_csv: Option<&[u8]>,
    ) -> Result<Self> {
        let mut named_types = Vec::new();
        if let Some(bytes) = named_types_csv {
            let mut rdr = csv::ReaderBuilder::new()
                .has_headers(true)
                .flexible(true)
                .from_reader(bytes);
            for (index, result) in rdr.records().enumerate() {
                let row = index.saturating_add(2);
                let record = result.with_context(|| {
                    format!(
                        "README-named-types.csv row {row}: invalid CSV record"
                    )
                })?;
                let name = record.get(0).map(str::trim)
                    .filter(|name| !name.is_empty())
                    .with_context(|| format!("README-named-types.csv row {row}: missing named type"))?;
                let syntax_raw = record.get(1).map(str::trim)
                    .filter(|syntax| !syntax.is_empty())
                    .with_context(|| format!("README-named-types.csv row {row}: named type '{name}' has no syntax"))?;
                let rule = parse_dc_syntax(syntax_raw).with_context(|| {
                    format!("README-named-types.csv row {row}: invalid syntax for named type '{name}': {syntax_raw}")
                })?;
                named_types.push((name.to_string(), rule.pattern));
            }
        }
        Self::load_from_definitions(dc_defns, format_defns, named_types)
    }

    /// Builds a resolver from reconstructed or parsed character, format, and named construct definitions.
    pub(crate) fn load_from_definitions(
        dc_defns: &[DcDefn],
        format_defns: &[DcDefn],
        named_types: Vec<(String, crate::syntax::ast::SyntaxPattern)>,
    ) -> Result<Self> {
        let mut resolver = Self::new();

        let mut explicit_closes_pairs: Vec<(u128, CharTarget)> = Vec::new();

        // 1. Index rules and scripts from Document Character definitions
        for defn in dc_defns {
            if let Some(short_id) = defn.short_id {
                let Ok(sid_u32) = u32::try_from(short_id) else {
                    continue;
                };
                let target = CharTarget::Dc(sid_u32);
                let sid_u128 = DcChar::from_short(sid_u32).to_u128();
                if !defn.script.is_empty() {
                    resolver.add_script_char(defn.script.clone(), sid_u128);
                }
                let preferred = parse_prefer_target(&defn.decompositions);
                if defn.is_deprecated || preferred.is_some() {
                    resolver.register_deprecated(target.clone(), preferred);
                }
                if let Some(ref rule) = defn.syntax {
                    resolver.add_rule(target, rule.clone());
                }
                if let Some(ref closes) = defn.closes {
                    let opener_target = match closes {
                        ctb_storage_minimal::shorthand::DcShorthand::Short(id) => {
                            u32::try_from(*id).ok().map(CharTarget::Dc)
                        }
                        ctb_storage_minimal::shorthand::DcShorthand::Format(id) => {
                            ctb_utilities::format_id::FormatId::from_shorthand(&format!("f{id}"))
                                .map(CharTarget::Format)
                        }
                        ctb_storage_minimal::shorthand::DcShorthand::Unicode(cp) => {
                            Some(CharTarget::Unicode(*cp))
                        }
                        _ => None,
                    };
                    if let Some(target) = opener_target {
                        explicit_closes_pairs.push((sid_u128, target.clone()));
                        resolver.register_closing_delimiter(sid_u128, target);
                    }
                }
            }
        }

        // Declared delimiters take precedence: a field ending in a nested
        // representation's closer does not itself open that representation.
        for defn in dc_defns {
            if defn.script == ".EL Routines" {
                continue;
            }
            if let (Some(short_id), Some(rule)) = (defn.short_id, &defn.syntax)
            {
                if let (Ok(id), Some(closer)) = (
                    u32::try_from(short_id),
                    detect_container_closer(&rule.pattern),
                ) {
                    let target = CharTarget::Dc(id);
                    if !resolver.opener_to_closer.contains_key(&target) {
                        if !resolver.closing_delimiters.contains_key(&closer) {
                            resolver.register_closing_delimiter(closer, target);
                        } else if let crate::syntax::ast::SyntaxPattern::Sequence(
                            elems,
                        ) = &rule.pattern
                            && elems.len() >= 3
                            && elems
                                .get(elems.len().saturating_sub(2))
                                .is_some_and(|el| {
                                    el.quantifier
                                        != crate::syntax::ast::Quantifier::ExactOne
                                })
                        {
                            resolver.opener_to_closer.insert(target, closer);
                        }
                    }
                }
            }
        }

        // Index formats from formats category files under their script
        for defn in format_defns {
            if let Some(short_id) = defn.short_id {
                if !defn.script.is_empty() {
                    resolver.add_script_char(defn.script.clone(), defn.dc_id);
                }
                let Some(fid) =
                    ctb_utilities::format_id::FormatId::from_shorthand(
                        &format!("f{short_id}"),
                    )
                    .or_else(|| {
                        defn.ident.as_deref().and_then(
                            ctb_utilities::format_id::FormatId::from_ident,
                        )
                    })
                else {
                    continue;
                };
                let target = CharTarget::Format(fid);
                let preferred = parse_prefer_target(&defn.decompositions);
                if defn.is_deprecated || preferred.is_some() {
                    resolver.register_deprecated(target.clone(), preferred);
                }
                if let Some(ref rule) = defn.syntax {
                    resolver.add_rule(target.clone(), rule.clone());
                    if let Some(closer_tok) =
                        detect_container_closer(&rule.pattern)
                    {
                        if !resolver.opener_to_closer.contains_key(&target)
                            && !resolver
                                .closing_delimiters
                                .contains_key(&closer_tok)
                        {
                            resolver
                                .register_closing_delimiter(closer_tok, target);
                        }
                    }
                }
            }
        }

        // 2. Index named types
        for (name, pattern) in named_types {
            resolver.add_named_type(name, pattern);
        }

        for defn in dc_defns.iter().chain(format_defns.iter()) {
            for imp in &defn.implies {
                for raw_part in imp.split('&') {
                    let trimmed = raw_part.trim();
                    if let Ok(sh) =
                        ctb_storage_minimal::shorthand::DcShorthand::parse(
                            trimmed,
                        )
                    {
                        if let Ok(super_gid) = sh.to_global_id() {
                            resolver.add_subtype(defn.dc_id, super_gid);
                        }
                    }
                }
            }
        }

        let equivalents: Vec<_> = dc_defns
            .iter()
            .chain(format_defns.iter())
            .flat_map(|defn| {
                defn.decompositions.iter().filter_map(move |decomposition| {
                    parse_decomposition_target(
                        std::slice::from_ref(decomposition),
                        "<equiv>",
                    )
                    .map(|target| (defn.dc_id, target))
                })
            })
            .collect();
        let closes_by_id: HashMap<
            u128,
            ctb_storage_minimal::shorthand::DcShorthand,
        > = dc_defns
            .iter()
            .chain(format_defns.iter())
            .filter_map(|d| d.closes.map(|c| (d.dc_id, c)))
            .collect();
        let non_routine_short_ids: Vec<u32> = dc_defns
            .iter()
            .filter(|d| d.script != ".EL Routines" && d.syntax.is_some())
            .filter_map(|d| d.short_id.and_then(|sid| u32::try_from(sid).ok()))
            .collect();

        resolver.finalize_equivalences_and_derived_tables(
            &equivalents,
            &explicit_closes_pairs,
            &closes_by_id,
            &non_routine_short_ids,
        )?;
        let mut text_decompositions = Vec::new();
        for defn in dc_defns.iter().chain(format_defns.iter()) {
            for decomposition in &defn.decompositions {
                let (payload, approximate) = if let Some(payload) =
                    decomposition.trim().strip_prefix("<equiv>")
                {
                    (payload, false)
                } else if let Some(payload) =
                    decomposition.trim().strip_prefix("<approx>")
                {
                    (payload, true)
                } else {
                    continue;
                };
                let targets: Option<Vec<_>> = payload
                    .split_whitespace()
                    .map(|token| {
                        ctb_storage_minimal::shorthand::DcShorthand::parse(
                            token,
                        )
                        .ok()?
                        .to_global_id()
                        .ok()
                    })
                    .collect();
                if let Some(targets) = targets
                    && !targets.is_empty()
                {
                    text_decompositions.push((
                        defn.dc_id,
                        targets,
                        approximate,
                    ));
                }
            }
        }
        resolver.build_unicode_text_mappings(&text_decompositions);
        Ok(resolver)
    }
}
