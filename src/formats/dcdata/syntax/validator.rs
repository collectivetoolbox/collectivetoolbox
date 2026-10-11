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

//! Semantic validator and cross-reference checker for Dc syntax AST trees.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::ast::{
    ActionArg, CharTarget, DcSyntaxRule, SyntaxElement, SyntaxPattern,
    SyntaxTerm,
};
use crate::report::ValidationReport;
use std::collections::{HashMap, HashSet};

/// Format metadata needed to validate format references in syntax rules (e.g. `[data:<format>]`).
#[derive(Debug, Clone, Default)]
pub struct FormatValidationInfo {
    pub short_id: usize,
    pub ident: Option<String>,
    pub category: String,
    pub nicknames: Vec<String>,
    pub has_import_support: bool,
    pub has_export_support: bool,
}

/// Registry of format definitions and support levels for syntax validation.
#[derive(Debug, Clone, Default)]
pub struct FormatValidationRegistry {
    pub categories: HashSet<String>,
    pub formats_by_id: HashMap<usize, FormatValidationInfo>,
    pub id_by_ident: HashMap<String, usize>,
    pub id_by_nick: HashMap<String, usize>,
}

impl FormatValidationRegistry {
    /// Creates a new empty format validation registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a format record to the registry.
    pub fn add_format(&mut self, info: FormatValidationInfo) {
        if !info.category.is_empty() {
            let cat = info.category.trim();
            // Reason for fallback: Category strings not starting with standard format prefixes are already in normalized form.
            let clean = cat
                .strip_prefix(".Formats:")
                .or_else(|| cat.strip_prefix("Formats:"))
                .or_else(|| cat.strip_prefix("format_"))
                .unwrap_or(cat);
            self.categories.insert(clean.to_string());
            self.categories.insert(clean.to_ascii_lowercase());
        }
        if let Some(ref ident) = info.ident {
            self.id_by_ident.insert(ident.clone(), info.short_id);
            self.id_by_ident.insert(ident.to_ascii_lowercase(), info.short_id);
        }
        for nick in &info.nicknames {
            self.id_by_nick.insert(nick.clone(), info.short_id);
            self.id_by_nick.insert(nick.to_ascii_lowercase(), info.short_id);
        }
        self.formats_by_id.insert(info.short_id, info);
    }

    /// Finds a format by shorthand (`f<N>`), ident, or nickname.
    #[must_use]
    pub fn find_format(&self, token: &str) -> Option<&FormatValidationInfo> {
        let trimmed = token.trim();
        if let Some(target) = super::parser::parse_target_token(trimmed) {
            if let CharTarget::Format(id) = target {
                return self.formats_by_id.get(&id);
            }
        }
        if let Some(&id) = self.id_by_ident.get(trimmed) {
            return self.formats_by_id.get(&id);
        }
        let lower = trimmed.to_ascii_lowercase();
        if let Some(&id) = self.id_by_ident.get(&lower) {
            return self.formats_by_id.get(&id);
        }
        if let Some(&id) = self.id_by_nick.get(&lower) {
            return self.formats_by_id.get(&id);
        }
        None
    }
}

/// Constructs a `FormatValidationRegistry` from parsed format definitions.
#[must_use]
pub fn build_format_validation_registry(format_rows: &[crate::DcDefn]) -> FormatValidationRegistry {
    let mut reg = FormatValidationRegistry::new();
    for row in format_rows {
        if let Some(short_id) = row.short_id {
            let mut nicknames = Vec::new();
            let mut has_import_support = false;
            let mut has_export_support = false;
            if let Some(ref f) = row.format {
                if let Some(ref n) = f.nicknames {
                    for nick in n.split(',') {
                        let t = nick.trim();
                        if !t.is_empty() {
                            nicknames.push(t.to_string());
                        }
                    }
                }
                if let Some(ref imp) = f.import_support {
                    if let Ok(lvl) = crate::SupportLevel::parse(imp) {
                        has_import_support = lvl.general > 0 || lvl.dc > 0;
                    }
                }
                if let Some(ref exp) = f.export_support {
                    if let Ok(lvl) = crate::SupportLevel::parse(exp) {
                        has_export_support = lvl.general > 0 || lvl.dc > 0;
                    }
                }
            }
            reg.add_format(FormatValidationInfo {
                short_id,
                ident: row.ident.clone(),
                category: row.category.clone(),
                nicknames,
                has_import_support,
                has_export_support,
            });
        }
    }
    reg
}

/// Validates a single `CharTarget` against registered datasets.
fn check_target_reference(
    target: &CharTarget,
    source_file: &str,
    line_no: usize,
    col_name: &str,
    known_dc_ids: &HashSet<u32>,
    known_format_ids: &HashSet<usize>,
    is_range_endpoint: bool,
    report: &mut ValidationReport,
) {
    match target {
        CharTarget::Dc(id) => {
            if is_range_endpoint {
                let max_short_dc = match u32::try_from(
                    ctb_storage_minimal::global_graph_layout::SHORT_DC_REGION_END
                        .saturating_sub(ctb_storage_minimal::global_graph_layout::SHORT_DC_REGION_START),
                ) {
                    Ok(m) => m,
                    Err(_) => u32::MAX,
                };
                if *id > max_short_dc {
                    report.add_error(
                        source_file,
                        Some(line_no),
                        Some(col_name),
                        format!("Dc ID '{id}' in syntax rule range exceeds maximum Short Dc ID {max_short_dc}"),
                        Some("Ensure Dc range is within valid Dc space"),
                    );
                }
            } else if !known_dc_ids.contains(id) {
                report.add_error(
                    source_file,
                    Some(line_no),
                    Some(col_name),
                    format!("Referenced Dc ID '{id}' in syntax rule does not exist in Document Characters registry"),
                    Some("Ensure referenced Dc ID is defined in a Dc category file"),
                );
            }
        }
        CharTarget::Format(id) => {
            if is_range_endpoint {
                let max_format_id = match usize::try_from(
                    ctb_storage_minimal::global_graph_layout::FORMAT_REGION_END
                        .saturating_sub(ctb_storage_minimal::global_graph_layout::FORMAT_REGION_START),
                ) {
                    Ok(m) => m,
                    Err(_) => usize::MAX,
                };
                if *id > max_format_id {
                    report.add_error(
                        source_file,
                        Some(line_no),
                        Some(col_name),
                        format!("Format ID 'f{id}' in syntax rule range exceeds maximum format ID f{max_format_id}"),
                        Some("Ensure format range is within valid format space"),
                    );
                }
            } else if !known_format_ids.contains(id) {
                report.add_error(
                    source_file,
                    Some(line_no),
                    Some(col_name),
                    format!("Format ID 'f{id}' in syntax rule does not exist in formats registry"),
                    Some("Ensure referenced format ID is defined in formats category files"),
                );
            }
        }
        CharTarget::Unicode(cp) => {
            if *cp > 0x0010_FFFF {
                report.add_error(
                    source_file,
                    Some(line_no),
                    Some(col_name),
                    format!("Referenced Unicode codepoint 'u{cp:04x}' (U+{cp:04X}) exceeds maximum Unicode codepoint U+10FFFF"),
                    Some("Ensure referenced Unicode codepoint is <= 0x10FFFF"),
                );
            } else if !is_range_endpoint && !ctb_formats_unicode::is_assigned_unicode(*cp) {
                report.add_error(
                    source_file,
                    Some(line_no),
                    Some(col_name),
                    format!("Referenced Unicode codepoint 'u{cp:04x}' (U+{cp:04X}) in syntax rule is not an assigned Unicode character"),
                    Some("Ensure referenced Unicode character exists in Unicode standard"),
                );
            }
        }
    }
}

/// Recursively validates pattern terms and accumulates bound capture variables.
fn validate_pattern_node(
    pattern: &SyntaxPattern,
    source_file: &str,
    line_no: usize,
    col_name: &str,
    self_dc_id: u32,
    known_dc_ids: &HashSet<u32>,
    known_format_ids: &HashSet<usize>,
    known_named_types: &HashSet<String>,
    known_scripts: Option<&HashSet<String>>,
    known_syntax_targets: &HashSet<CharTarget>,
    format_registry: Option<&FormatValidationRegistry>,
    bound_vars: &mut HashSet<String>,
    report: &mut ValidationReport,
) {
    match pattern {
        SyntaxPattern::Alternation(branches) => {
            for branch in branches {
                validate_pattern_node(
                    branch,
                    source_file,
                    line_no,
                    col_name,
                    self_dc_id,
                    known_dc_ids,
                    known_format_ids,
                    known_named_types,
                    known_scripts,
                    known_syntax_targets,
                    format_registry,
                    bound_vars,
                    report,
                );
            }
        }
        SyntaxPattern::Sequence(elements) => {
            for elem in elements {
                validate_syntax_element(
                    elem,
                    source_file,
                    line_no,
                    col_name,
                    self_dc_id,
                    known_dc_ids,
                    known_format_ids,
                    known_named_types,
                    known_scripts,
                    known_syntax_targets,
                    format_registry,
                    bound_vars,
                    report,
                );
            }
        }
    }
}

/// Validates an individual `SyntaxElement` node.
fn validate_syntax_element(
    element: &SyntaxElement,
    source_file: &str,
    line_no: usize,
    col_name: &str,
    self_dc_id: u32,
    known_dc_ids: &HashSet<u32>,
    known_format_ids: &HashSet<usize>,
    known_named_types: &HashSet<String>,
    known_scripts: Option<&HashSet<String>>,
    known_syntax_targets: &HashSet<CharTarget>,
    format_registry: Option<&FormatValidationRegistry>,
    bound_vars: &mut HashSet<String>,
    report: &mut ValidationReport,
) {
    match &element.term {
        SyntaxTerm::SelfChar => {
            // Self-referential '~' is valid
        }
        SyntaxTerm::CharRef(target) => {
            check_target_reference(
                target,
                source_file,
                line_no,
                col_name,
                known_dc_ids,
                known_format_ids,
                false,
                report,
            );
        }
        SyntaxTerm::RuleRef { target } => {
            check_target_reference(
                target,
                source_file,
                line_no,
                col_name,
                known_dc_ids,
                known_format_ids,
                false,
                report,
            );
            if target != &CharTarget::Dc(self_dc_id)
                && !known_syntax_targets.contains(target)
            {
                report.add_error(
                    source_file,
                    Some(line_no),
                    Some(col_name),
                    format!(
                        "Macro expansion '[{target}:]' references target that does not define its own syntax rule"
                    ),
                    Some(
                        "Define syntax for the referenced character or format before expanding it as a macro rule",
                    ),
                );
            }
        }
        SyntaxTerm::CharSet { members, .. } => {
            for target in members {
                check_target_reference(
                    target,
                    source_file,
                    line_no,
                    col_name,
                    known_dc_ids,
                    known_format_ids,
                    false,
                    report,
                );
            }
        }
        SyntaxTerm::CharRange { start, end } => {
            check_target_reference(
                start,
                source_file,
                line_no,
                col_name,
                known_dc_ids,
                known_format_ids,
                true,
                report,
            );
            check_target_reference(
                end,
                source_file,
                line_no,
                col_name,
                known_dc_ids,
                known_format_ids,
                true,
                report,
            );
            if start > end {
                report.add_error(
                    source_file,
                    Some(line_no),
                    Some(col_name),
                    format!("Invalid character range: start '{start}' is greater than end '{end}'"),
                    Some("Ensure character range is in ascending order"),
                );
            }
        }
        SyntaxTerm::NamedConstruct { name, subtype, capture_var } => {
            if name.is_empty() {
                report.add_error(
                    source_file,
                    Some(line_no),
                    Some(col_name),
                    "Named construct has an empty name".to_string(),
                    Some("Ensure named construct has a non-empty name e.g. [name]"),
                );
            } else if name == "data" {
                if let Some(sub) = subtype {
                    let sub_trimmed = sub.trim();
                    if sub_trimmed.is_empty() {
                        report.add_error(
                            source_file,
                            Some(line_no),
                            Some(col_name),
                            "Named construct '[data:]' is missing a format identifier".to_string(),
                            Some("Provide a format identifier or use bare '[data]'"),
                        );
                    } else if let Some(formats) = format_registry {
                        // Reason for fallback: Category names without formal prefixes are matched verbatim against known category sets.
                        let clean_cat = sub_trimmed
                            .strip_prefix(".Formats:")
                            .or_else(|| sub_trimmed.strip_prefix("Formats:"))
                            .or_else(|| sub_trimmed.strip_prefix("format_"))
                            .unwrap_or(sub_trimmed);
                        let is_category = formats.categories.contains(clean_cat)
                            || formats.categories.contains(&clean_cat.to_ascii_lowercase())
                            || clean_cat.eq_ignore_ascii_case("hash")
                            || clean_cat.eq_ignore_ascii_case("compression")
                            || clean_cat.eq_ignore_ascii_case("container")
                            || clean_cat.eq_ignore_ascii_case("document")
                            || clean_cat.eq_ignore_ascii_case("database");
                        if is_category {
                            report.add_error(
                                source_file,
                                Some(line_no),
                                Some(col_name),
                                format!(
                                    "Construct '[data:{sub_trimmed}]' specifies a format category instead of a specific format identifier"
                                ),
                                Some(
                                    "Specify a specific format identifier with import and export support rather than a category name",
                                ),
                            );
                        } else if let Some(fmt_info) = formats.find_format(sub_trimmed) {
                            if !fmt_info.has_import_support || !fmt_info.has_export_support {
                                // Reason for fallback: Formats lacking a canonical identifier use the subtype string specified in the syntax definition.
                                let fmt_name = fmt_info.ident.as_deref().unwrap_or(sub_trimmed);
                                report.add_error(
                                    source_file,
                                    Some(line_no),
                                    Some(col_name),
                                    format!(
                                        "Format '{fmt_name}' referenced in '[data:{sub_trimmed}]' must have both import and export support"
                                    ),
                                    Some(
                                        "Ensure referenced format has both import and export support in its category table",
                                    ),
                                );
                            }
                        } else {
                            report.add_error(
                                source_file,
                                Some(line_no),
                                Some(col_name),
                                format!(
                                    "Unknown format identifier '{sub_trimmed}' in '[data:{sub_trimmed}]'"
                                ),
                                Some(
                                    "Specify a registered format identifier that has both import and export support",
                                ),
                            );
                        }
                    }
                }
            } else if name == "script" {
                if subtype.as_ref().is_none_or(|s| s.is_empty()) {
                    report.add_error(
                        source_file,
                        Some(line_no),
                        Some(col_name),
                        "Named construct '[script]' requires a non-empty script name (e.g. '[script:.EL Types]')".to_string(),
                        Some("Specify a valid script name following a colon"),
                    );
                } else if let Some(st) = subtype {
                    let is_valid = if st.starts_with('.') {
                        if let Some(scripts) = known_scripts {
                            scripts.contains(st)
                        } else {
                            true
                        }
                    } else {
                        ctb_formats_unicode::is_valid_unicode_script(st)
                    };
                    if !is_valid {
                        report.add_error(
                            source_file,
                            Some(line_no),
                            Some(col_name),
                            format!("Unknown script '[script:{st}]' in syntax rule"),
                            Some("Must be a registered Dc script or valid Unicode script"),
                        );
                    }
                }
            } else if !known_named_types.is_empty()
                && !known_named_types.contains(name)
                && name != "formats"
            {
                report.add_error(
                    source_file,
                    Some(line_no),
                    Some(col_name),
                    format!("Unknown named type construct '[{name}]' in syntax rule"),
                    Some("Must be a registered named type defined in README-named-types.csv"),
                );
            }
            if let Some(var) = capture_var {
                bound_vars.insert(var.clone());
            }
        }
        SyntaxTerm::Group(nested_pattern) => {
            validate_pattern_node(
                nested_pattern,
                source_file,
                line_no,
                col_name,
                self_dc_id,
                known_dc_ids,
                known_format_ids,
                known_named_types,
                known_scripts,
                known_syntax_targets,
                format_registry,
                bound_vars,
                report,
            );
        }
    }
}

/// Validates a parsed `DcSyntaxRule` AST against database registries.
pub fn validate_dc_syntax(
    rule: &DcSyntaxRule,
    self_dc_id: u32,
    known_dc_ids: &HashSet<u32>,
    known_format_ids: &HashSet<usize>,
    known_named_types: &HashSet<String>,
    known_scripts: Option<&HashSet<String>>,
    known_syntax_targets: &HashSet<CharTarget>,
    format_registry: Option<&FormatValidationRegistry>,
    report: &mut ValidationReport,
    source_file: &str,
    line_no: usize,
) {
    let col_name = "Aliases (syntax)";
    let mut bound_vars = HashSet::new();

    // 1. Validate Pattern Tree and collect bound variables
    validate_pattern_node(
        &rule.pattern,
        source_file,
        line_no,
        col_name,
        self_dc_id,
        known_dc_ids,
        known_format_ids,
        known_named_types,
        known_scripts,
        known_syntax_targets,
        format_registry,
        &mut bound_vars,
        report,
    );

    // 2. Validate Action Invocation variables
    if let Some(action) = &rule.action {
        for arg in &action.args {
            if let ActionArg::Variable(var_name) = arg {
                if !bound_vars.contains(var_name) {
                    report.add_error(
                        source_file,
                        Some(line_no),
                        Some(col_name),
                        format!(
                            "Variable '${var_name}' in action invocation '{}' is not bound in syntax pattern",
                            action.method
                        ),
                        Some("Ensure all action parameters correspond to captured variables ($var) in the pattern"),
                    );
                }
            }
        }
    }
}

