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
    report: &mut ValidationReport,
) {
    match target {
        CharTarget::Dc(id) => {
            if !known_dc_ids.contains(id) {
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
            if !known_format_ids.contains(id) {
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
            if !ctb_formats_unicode::is_assigned_unicode(*cp) {
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
    known_dc_ids: &HashSet<u32>,
    known_format_ids: &HashSet<usize>,
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
                    known_dc_ids,
                    known_format_ids,
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
                    known_dc_ids,
                    known_format_ids,
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
    known_dc_ids: &HashSet<u32>,
    known_format_ids: &HashSet<usize>,
    bound_vars: &mut HashSet<String>,
    report: &mut ValidationReport,
) {
    match &element.term {
        SyntaxTerm::SelfChar => {
            // Self-referential '~' is valid
        }
        SyntaxTerm::CharRef(target) | SyntaxTerm::RuleRef { target } => {
            check_target_reference(
                target,
                source_file,
                line_no,
                col_name,
                known_dc_ids,
                known_format_ids,
                report,
            );
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
                report,
            );
            check_target_reference(
                end,
                source_file,
                line_no,
                col_name,
                known_dc_ids,
                known_format_ids,
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
        SyntaxTerm::NamedConstruct { capture_var, .. } => {
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
                known_dc_ids,
                known_format_ids,
                bound_vars,
                report,
            );
        }
    }
}

/// Validates a parsed `DcSyntaxRule` AST against database registries.
pub fn validate_dc_syntax(
    rule: &DcSyntaxRule,
    _self_dc_id: u32,
    known_dc_ids: &HashSet<u32>,
    known_format_ids: &HashSet<usize>,
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
        known_dc_ids,
        known_format_ids,
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
