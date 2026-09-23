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
use std::collections::HashSet;

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
                    format!("Referenced Format ID 'f{id}' in syntax rule does not exist in formats registry"),
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
    known_scripts: &HashSet<String>,
    known_syntax_targets: &HashSet<CharTarget>,
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
    known_scripts: &HashSet<String>,
    known_syntax_targets: &HashSet<CharTarget>,
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
                    if !known_scripts.is_empty() && !known_scripts.contains(st) {
                        report.add_error(
                            source_file,
                            Some(line_no),
                            Some(col_name),
                            format!("Unknown script '[script:{st}]' in syntax rule"),
                            Some("Must be a registered script defined in README.scripts.csv"),
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
    known_scripts: &HashSet<String>,
    known_syntax_targets: &HashSet<CharTarget>,
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
