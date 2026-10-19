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

//! Build-time codegen for the generated static Dc syntax table.
//!
//! Reads the authoritative CSV dataset in `src/formats/dcdata/data/`,
//! indexes it through [`DatasetRuleResolver::load_from_dataset`], and emits
//! `syntax_table.generated.rs` containing the fully derived resolver state
//! (syntax rules, named types, scripts, opener/closer pairs, `<prefer>`
//! deprecations, container headers, infix rules, starter sets, and opaque
//! containers) as static Rust data. Runtime loading then materializes the
//! generated table directly, with no CSV parsing, syntax DSL parsing, or
//! derived-table recomputation.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::{DatasetRuleResolver, StarterSet};
use crate::report::ValidationReport;
use crate::syntax::ast::{
    ActionArg, CharTarget, DcSyntaxRule, Quantifier, SyntaxAction,
    SyntaxElement, SyntaxPattern, SyntaxTerm,
};
use std::collections::HashSet;
use std::path::Path;

/// Returns indentation of `indent` levels (4 spaces each).
fn pad(indent: usize) -> String {
    "    ".repeat(indent)
}

/// Groups a decimal digit string with `_` separators every three digits so
/// generated literals satisfy `clippy::unreadable_literal`. Values of four
/// digits or fewer are returned unchanged.
fn group_digits(digits: &str) -> String {
    let chars: Vec<char> = digits.chars().collect();
    if chars.len() <= 4 {
        return digits.to_string();
    }
    let mut groups: Vec<String> =
        chars.rchunks(3).map(|group| group.iter().collect()).collect();
    groups.reverse();
    groups.join("_")
}

/// Emits a `CharTarget` as a Rust literal expression.
fn emit_char_target(target: &CharTarget) -> String {
    match target {
        CharTarget::Dc(id) => format!("CharTarget::Dc({id})"),
        CharTarget::Format(fid) => {
            format!("CharTarget::Format(FormatId::{})", fid.ident())
        }
        CharTarget::Unicode(cp) => {
            format!("CharTarget::Unicode({})", group_digits(&cp.to_string()))
        }
    }
}

/// Formats a `u128` as a Rust literal with digit separators.
fn format_u128_literal(value: u128) -> String {
    let digits = value.to_string();
    let grouped = group_digits(&digits);
    if grouped == digits {
        format!("{value}u128")
    } else {
        format!("{grouped}_u128")
    }
}

/// Emits an optional string as a Rust literal expression.
fn emit_option_string(value: Option<&str>) -> String {
    match value {
        Some(text) => format!("Some({text:?}.to_string())"),
        None => "None".to_string(),
    }
}

/// Emits a `Quantifier` as a Rust literal expression.
fn emit_quantifier(quantifier: Quantifier) -> String {
    match quantifier {
        Quantifier::ExactOne => "Quantifier::ExactOne".to_string(),
        Quantifier::OneOrMore => "Quantifier::OneOrMore".to_string(),
        Quantifier::ZeroOrMore => "Quantifier::ZeroOrMore".to_string(),
        Quantifier::Optional => "Quantifier::Optional".to_string(),
        Quantifier::Range { min, max } => {
            format!("Quantifier::Range {{ min: {min}, max: {max:?} }}")
        }
    }
}

/// Emits a `SyntaxTerm` as a Rust literal expression. Nested groups wrap
/// onto separate lines closing at `indent`.
fn emit_term(term: &SyntaxTerm, indent: usize) -> String {
    match term {
        SyntaxTerm::SelfChar => "SyntaxTerm::SelfChar".to_string(),
        SyntaxTerm::CharRef(target) => {
            format!("SyntaxTerm::CharRef({})", emit_char_target(target))
        }
        SyntaxTerm::CharSet { negated, members } => {
            let members: Vec<String> =
                members.iter().map(emit_char_target).collect();
            format!(
                "SyntaxTerm::CharSet {{ negated: {negated}, members: vec![{}] }}",
                members.join(", ")
            )
        }
        SyntaxTerm::CharRange { start, end } => format!(
            "SyntaxTerm::CharRange {{ start: {}, end: {} }}",
            emit_char_target(start),
            emit_char_target(end)
        ),
        SyntaxTerm::RuleRef { target } => format!(
            "SyntaxTerm::RuleRef {{ target: {} }}",
            emit_char_target(target)
        ),
        SyntaxTerm::NamedConstruct {
            name,
            subtype,
            capture_var,
        } => format!(
            "SyntaxTerm::NamedConstruct {{ name: {name:?}.to_string(), subtype: {}, capture_var: {} }}",
            emit_option_string(subtype.as_deref()),
            emit_option_string(capture_var.as_deref())
        ),
        SyntaxTerm::Group(pattern) => {
            format!("SyntaxTerm::Group({})", emit_pattern(pattern, indent))
        }
    }
}

/// Emits a `SyntaxElement` as a Rust literal expression. Elements without
/// nested groups fit on a single line.
fn emit_element(element: &SyntaxElement, indent: usize) -> String {
    format!(
        "SyntaxElement {{ term: {}, quantifier: {} }}",
        emit_term(&element.term, indent),
        emit_quantifier(element.quantifier)
    )
}

/// Emits a `Vec<SyntaxElement>` literal, one element per line, closing the
/// list at `indent`.
fn emit_element_vec(elements: &[SyntaxElement], indent: usize) -> String {
    if elements.is_empty() {
        return "Vec::new()".to_string();
    }
    let inner = indent.saturating_add(1);
    let mut out = String::from("vec![\n");
    for element in elements {
        out.push_str(&pad(inner));
        out.push_str(&emit_element(element, inner));
        out.push_str(",\n");
    }
    out.push_str(&pad(indent));
    out.push(']');
    out
}

/// Emits a `SyntaxPattern` as a Rust literal expression, wrapping sequence
/// elements and alternation branches onto separate lines.
fn emit_pattern(pattern: &SyntaxPattern, indent: usize) -> String {
    match pattern {
        SyntaxPattern::Sequence(elements) => format!(
            "SyntaxPattern::Sequence({})",
            emit_element_vec(elements, indent)
        ),
        SyntaxPattern::Alternation(branches) => {
            if branches.is_empty() {
                return "SyntaxPattern::Alternation(Vec::new())".to_string();
            }
            let inner = indent.saturating_add(1);
            let mut out = String::from("SyntaxPattern::Alternation(vec![\n");
            for branch in branches {
                out.push_str(&pad(inner));
                out.push_str(&emit_pattern(branch, inner));
                out.push_str(",\n");
            }
            out.push_str(&pad(indent));
            out.push_str("])");
            out
        }
    }
}

/// Emits an optional `SyntaxAction` as a Rust literal expression.
fn emit_action(action: Option<&SyntaxAction>) -> String {
    let Some(action) = action else {
        return "None".to_string();
    };
    let args: Vec<String> = action
        .args
        .iter()
        .map(|arg| match arg {
            ActionArg::Variable(name) => {
                format!("ActionArg::Variable({name:?}.to_string())")
            }
            ActionArg::Literal(value) => {
                format!("ActionArg::Literal({value:?}.to_string())")
            }
        })
        .collect();
    format!(
        "Some(SyntaxAction {{ method: {:?}.to_string(), args: vec![{}] }})",
        action.method,
        args.join(", ")
    )
}

/// Emits a `DcSyntaxRule` literal with fields at `indent + 1`.
fn emit_rule(rule: &DcSyntaxRule, indent: usize) -> String {
    let field_indent = indent.saturating_add(1);
    let field_pad = pad(field_indent);
    let mut out = String::from("DcSyntaxRule {\n");
    out.push_str(&field_pad);
    out.push_str("pattern: ");
    out.push_str(&emit_pattern(&rule.pattern, field_indent));
    out.push_str(",\n");
    out.push_str(&field_pad);
    out.push_str("action: ");
    out.push_str(&emit_action(rule.action.as_ref()));
    out.push_str(",\n");
    out.push_str(&field_pad);
    out.push_str(&format!("raw: {:?}.to_string(),\n", rule.raw));
    out.push_str(&pad(indent));
    out.push('}');
    out
}

/// Emits a `u128` array literal, wrapping tokens onto lines of up to eight
/// entries and closing the list at `indent`.
fn emit_u128_array(tokens: &[u128], indent: usize) -> String {
    if tokens.is_empty() {
        return "[]".to_string();
    }
    let inner = indent.saturating_add(1);
    let mut out = String::from("[\n");
    for chunk in tokens.chunks(8) {
        out.push_str(&pad(inner));
        let items: Vec<String> = chunk
            .iter()
            .copied()
            .map(format_u128_literal)
            .collect();
        out.push_str(&items.join(", "));
        out.push_str(",\n");
    }
    out.push_str(&pad(indent));
    out.push(']');
    out
}

/// Emits a `HashSet<u128>` literal built from a sorted token slice.
fn emit_hashset(tokens: &[u128], indent: usize) -> String {
    if tokens.is_empty() {
        return "HashSet::new()".to_string();
    }
    format!("HashSet::from({})", emit_u128_array(tokens, indent))
}

/// Emits the `populate_rules` section (character and format syntax rules).
fn emit_rules_section(resolver: &DatasetRuleResolver) -> String {
    let mut entries: Vec<(&CharTarget, &DcSyntaxRule)> =
        resolver.rules.iter().collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));
    let mut out = String::new();
    out.push_str("fn populate_rules(resolver: &mut DatasetRuleResolver) {\n");
    for (target, rule) in entries {
        out.push_str("    resolver.rules.insert(\n");
        out.push_str(&format!("        {},\n", emit_char_target(target)));
        out.push_str(&format!("        {},\n", emit_rule(rule, 2)));
        out.push_str("    );\n");
    }
    out.push_str("}\n\n");
    out
}

/// Emits the `populate_named_types` section.
fn emit_named_types_section(resolver: &DatasetRuleResolver) -> String {
    let mut entries: Vec<(&String, &SyntaxPattern)> =
        resolver.named_types.iter().collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));
    let mut out = String::new();
    out.push_str(
        "fn populate_named_types(resolver: &mut DatasetRuleResolver) {\n",
    );
    for (name, pattern) in entries {
        out.push_str("    resolver.named_types.insert(\n");
        out.push_str(&format!("        {name:?}.to_string(),\n"));
        out.push_str(&format!("        {},\n", emit_pattern(pattern, 2)));
        out.push_str("    );\n");
    }
    out.push_str("}\n\n");
    out
}

/// Emits the `populate_scripts` section (script membership sets).
fn emit_scripts_section(resolver: &DatasetRuleResolver) -> String {
    let mut entries: Vec<(&String, &HashSet<u128>)> =
        resolver.script_chars.iter().collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));
    let mut out = String::new();
    out.push_str(
        "fn populate_scripts(resolver: &mut DatasetRuleResolver) {\n",
    );
    for (script, tokens) in entries {
        let mut sorted: Vec<u128> = tokens.iter().copied().collect();
        sorted.sort_unstable();
        out.push_str("    resolver.script_chars.insert(\n");
        out.push_str(&format!("        {script:?}.to_string(),\n"));
        out.push_str(&format!("        {},\n", emit_hashset(&sorted, 2)));
        out.push_str("    );\n");
    }
    out.push_str("}\n\n");
    out
}

/// Emits the `populate_closing_delimiters` section.
fn emit_closing_delimiters_section(
    resolver: &DatasetRuleResolver,
) -> String {
    let mut entries: Vec<(u128, &CharTarget)> = resolver
        .closing_delimiters
        .iter()
        .map(|(k, v)| (*k, v))
        .collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    let mut out = String::new();
    out.push_str(
        "fn populate_closing_delimiters(resolver: &mut DatasetRuleResolver) {\n",
    );
    for (closer, opener) in entries {
        out.push_str(&format!(
            "    resolver.closing_delimiters.insert({}, {});\n",
            format_u128_literal(closer),
            emit_char_target(opener)
        ));
    }
    out.push_str("}\n\n");
    out
}

/// Emits the `populate_opener_to_closer` section.
fn emit_opener_to_closer_section(resolver: &DatasetRuleResolver) -> String {
    let mut entries: Vec<(&CharTarget, u128)> = resolver
        .opener_to_closer
        .iter()
        .map(|(k, v)| (k, *v))
        .collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));
    let mut out = String::new();
    out.push_str(
        "fn populate_opener_to_closer(resolver: &mut DatasetRuleResolver) {\n",
    );
    for (opener, closer) in entries {
        out.push_str(&format!(
            "    resolver.opener_to_closer.insert({}, {});\n",
            emit_char_target(opener),
            format_u128_literal(closer)
        ));
    }
    out.push_str("}\n\n");
    out
}

/// Emits the `populate_deprecations` section (`<prefer>` replacements).
fn emit_deprecations_section(resolver: &DatasetRuleResolver) -> String {
    let mut entries: Vec<(&CharTarget, &Option<CharTarget>)> =
        resolver.deprecated_targets.iter().collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));
    let mut out = String::new();
    out.push_str(
        "fn populate_deprecations(resolver: &mut DatasetRuleResolver) {\n",
    );
    for (target, preferred) in entries {
        let preferred = match preferred {
            Some(replacement) => {
                format!("Some({})", emit_char_target(replacement))
            }
            None => "None".to_string(),
        };
        out.push_str(&format!(
            "    resolver.deprecated_targets.insert({}, {preferred});\n",
            emit_char_target(target)
        ));
    }
    out.push_str("}\n\n");
    out
}

/// Emits a section inserting `Vec<SyntaxElement>` values keyed by opener
/// target into the named resolver field.
fn emit_element_map_section(
    fn_name: &str,
    field_name: &str,
    entries: Vec<(&CharTarget, &Vec<SyntaxElement>)>,
) -> String {
    let mut entries = entries;
    entries.sort_by(|a, b| a.0.cmp(b.0));
    let mut out = String::new();
    out.push_str(&format!(
        "fn {fn_name}(resolver: &mut DatasetRuleResolver) {{\n"
    ));
    for (opener, elements) in entries {
        out.push_str(&format!("    resolver.{field_name}.insert(\n"));
        out.push_str(&format!("        {},\n", emit_char_target(opener)));
        out.push_str(&format!("        {},\n", emit_element_vec(elements, 2)));
        out.push_str("    );\n");
    }
    out.push_str("}\n\n");
    out
}

/// Emits the `populate_infix_rules` section.
fn emit_infix_rules_section(resolver: &DatasetRuleResolver) -> String {
    let mut entries: Vec<(u128, &Vec<SyntaxElement>, &Vec<SyntaxElement>)> =
        resolver
            .infix_rules
            .iter()
            .map(|(token, (lhs, rhs))| (*token, lhs, rhs))
            .collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    let mut out = String::new();
    out.push_str(
        "fn populate_infix_rules(resolver: &mut DatasetRuleResolver) {\n",
    );
    for (token, lhs, rhs) in entries {
        out.push_str(&format!(
            "    resolver.infix_rules.insert({}, (\n",
            format_u128_literal(token)
        ));
        out.push_str(&format!("        {},\n", emit_element_vec(lhs, 2)));
        out.push_str(&format!("        {},\n", emit_element_vec(rhs, 2)));
        out.push_str("    ));\n");
    }
    out.push_str("}\n\n");
    out
}

/// Emits the `populate_named_type_starters` section.
fn emit_named_type_starters_section(
    resolver: &DatasetRuleResolver,
) -> String {
    let mut entries: Vec<(&String, &StarterSet)> =
        resolver.named_type_starters.iter().collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));
    let mut out = String::new();
    out.push_str(
        "fn populate_named_type_starters(resolver: &mut DatasetRuleResolver) {\n",
    );
    for (name, starters) in entries {
        let mut tokens: Vec<u128> =
            starters.exact_tokens.iter().copied().collect();
        tokens.sort_unstable();
        out.push_str("    resolver.named_type_starters.insert(\n");
        out.push_str(&format!("        {name:?}.to_string(),\n"));
        out.push_str("        StarterSet {\n");
        out.push_str(&format!(
            "            exact_tokens: {},\n",
            emit_hashset(&tokens, 3)
        ));
        if starters.ranges.is_empty() {
            out.push_str("            ranges: Vec::new(),\n");
        } else {
            out.push_str("            ranges: vec![\n");
            for (start, end) in &starters.ranges {
                out.push_str(&format!(
                    "                ({}, {}),\n",
                    format_u128_literal(*start),
                    format_u128_literal(*end)
                ));
            }
            out.push_str("            ],\n");
        }
        if starters.scripts.is_empty() {
            out.push_str("            scripts: Vec::new(),\n");
        } else {
            out.push_str("            scripts: vec![\n");
            for script in &starters.scripts {
                out.push_str(&format!(
                    "                {script:?}.to_string(),\n"
                ));
            }
            out.push_str("            ],\n");
        }
        out.push_str("        },\n");
        out.push_str("    );\n");
    }
    out.push_str("}\n\n");
    out
}

/// Emits the `populate_opaque_containers` section.
fn emit_opaque_containers_section(resolver: &DatasetRuleResolver) -> String {
    let mut tokens: Vec<u128> =
        resolver.opaque_containers.iter().copied().collect();
    tokens.sort_unstable();
    let mut out = String::new();
    out.push_str(
        "fn populate_opaque_containers(resolver: &mut DatasetRuleResolver) {\n",
    );
    out.push_str(&format!(
        "    resolver.opaque_containers.extend({});\n",
        emit_u128_array(&tokens, 1)
    ));
    out.push_str("}\n");
    out
}

/// Appends a generated section to the output, prepending an item-level
/// `clippy::too_many_lines` expectation when the section exceeds the lint's
/// default budget of 100 lines (generated literal sections cannot be split).
/// Section strings end with `}\n\n`, so the function span itself is one line
/// shorter than the string's line count.
fn push_section(out: &mut String, section: &str) {
    if section.lines().count() > 101 {
        out.push_str("#[expect(\n");
        out.push_str("    clippy::too_many_lines,\n");
        out.push_str(
            "    reason = \"Generated static syntax table section\"\n",
        );
        out.push_str(")]\n");
    }
    out.push_str(section);
}

/// Generates the full Rust source contents of
/// `syntax_table.generated.rs` from a fully loaded resolver.
#[must_use]
pub fn generate_syntax_table_code(resolver: &DatasetRuleResolver) -> String {
    let mut out = String::new();
    out.push_str(ctb_build_support::license_consts::DEFAULT_AGPL_HEADER);
    out.push_str("\n\n");
    out.push_str("//! Static Document Character (Dc) syntax table.\n");
    out.push_str(
        "//! @generated by `ctb_formats_dcdata::syntax::codegen` from the CSV\n",
    );
    out.push_str(
        "//! syntax rules in src/formats/dcdata/data/. Do not edit by hand;\n",
    );
    out.push_str(
        "//! regenerate with ./scripts/validate-data-tables.\n\n",
    );
    out.push_str("#[expect(\n");
    out.push_str("    unused_imports,\n");
    out.push_str("    clippy::wildcard_imports,\n");
    out.push_str("    reason = \"Standard workspace module prelude\"\n");
    out.push_str(")]\n");
    out.push_str("use crate::utilities::*;\n\n");
    out.push_str("use super::{DatasetRuleResolver, StarterSet};\n");
    out.push_str("use crate::syntax::ast::{\n");
    out.push_str(
        "    ActionArg, CharTarget, DcSyntaxRule, Quantifier, SyntaxAction,\n",
    );
    out.push_str("    SyntaxElement, SyntaxPattern, SyntaxTerm,\n");
    out.push_str("};\n");
    out.push_str("use ctb_utilities::format_id::FormatId;\n");
    out.push_str("use std::collections::HashSet;\n\n");
    out.push_str(
        "/// Materializes the complete generated dataset into a resolver.\n",
    );
    out.push_str(
        "pub(super) fn populate(resolver: &mut DatasetRuleResolver) {\n",
    );
    out.push_str("    populate_rules(resolver);\n");
    out.push_str("    populate_named_types(resolver);\n");
    out.push_str("    populate_scripts(resolver);\n");
    out.push_str("    populate_closing_delimiters(resolver);\n");
    out.push_str("    populate_opener_to_closer(resolver);\n");
    out.push_str("    populate_deprecations(resolver);\n");
    out.push_str("    populate_container_headers(resolver);\n");
    out.push_str("    populate_trailing_after_closer(resolver);\n");
    out.push_str("    populate_infix_rules(resolver);\n");
    out.push_str("    populate_named_type_starters(resolver);\n");
    out.push_str("    populate_semantic_equivalents(resolver);\n");
    out.push_str("    populate_opaque_containers(resolver);\n");
    out.push_str("}\n\n");

    push_section(&mut out, &emit_rules_section(resolver));
    push_section(&mut out, &emit_named_types_section(resolver));
    push_section(&mut out, &emit_scripts_section(resolver));
    push_section(&mut out, &emit_closing_delimiters_section(resolver));
    push_section(&mut out, &emit_opener_to_closer_section(resolver));
    push_section(&mut out, &emit_deprecations_section(resolver));
    push_section(
        &mut out,
        &emit_element_map_section(
            "populate_container_headers",
            "container_headers",
            resolver.container_headers.iter().collect(),
        ),
    );
    push_section(
        &mut out,
        &emit_element_map_section(
            "populate_trailing_after_closer",
            "trailing_after_closer",
            resolver.trailing_after_closer.iter().collect(),
        ),
    );
    push_section(&mut out, &emit_infix_rules_section(resolver));
    push_section(&mut out, &emit_named_type_starters_section(resolver));
    let mut equivalents: Vec<_> = resolver.semantic_equivalents.iter().collect();
    equivalents.sort_unstable_by_key(|(token, _)| **token);
    out.push_str("fn populate_semantic_equivalents(resolver: &mut DatasetRuleResolver) {\n");
    for (token, members) in equivalents {
        let mut members: Vec<_> = members.iter().copied().collect();
        members.sort_unstable();
        let literals: Vec<_> = members.into_iter().map(format_u128_literal).collect();
        out.push_str(&format!("    resolver.semantic_equivalents.insert({}, HashSet::from([{}]));\n",
            format_u128_literal(*token), literals.join(", ")));
    }
    out.push_str("}\n\n");
    push_section(&mut out, &emit_opaque_containers_section(resolver));
    out
}

/// Generates or updates `syntax/resolver/syntax_table.generated.rs` from the
/// CSV dataset on disk under `repo_root`.
///
/// # Errors
/// Returns an error if the dataset directories or the named types CSV are
/// missing or empty, validation fails, or writing the generated file fails.
/// Invalid datasets are rejected before the generated file is written.
pub fn generate_syntax_table_file(repo_root: &Path) -> Result<bool> {
    let data_dir = repo_root
        .join("src")
        .join("formats")
        .join("dcdata")
        .join("data");
    let categories_dir = data_dir.join("categories");
    let formats_dir = categories_dir.join("formats");
    let named_types_path = data_dir.join("README-named-types.csv");
    ensure!(
        categories_dir.is_dir(),
        "Could not locate Dc categories directory at {}",
        categories_dir.display()
    );
    ensure!(
        formats_dir.is_dir(),
        "Could not locate formats categories directory at {}",
        formats_dir.display()
    );
    let named_types_bytes = std::fs::read(&named_types_path)
        .with_context(|| {
            format!("Failed to read {}", named_types_path.display())
        })?;
    ensure!(
        !named_types_bytes.is_empty(),
        "README-named-types.csv at {} is empty",
        named_types_path.display()
    );

    let resolver = load_resolver_from_disk(
        &categories_dir,
        &formats_dir,
        &named_types_bytes,
    )?;
    let code = generate_syntax_table_code(&resolver);
    let target_file = repo_root
        .join("src")
        .join("formats")
        .join("dcdata")
        .join("syntax")
        .join("resolver")
        .join("syntax_table.generated.rs");
    crate::updater::write_if_changed(&target_file, &code)
}

/// Indexes the on-disk CSV dataset into a resolver, mirroring the canonical
/// embedded loading order used by
/// [`DatasetRuleResolver::load_from_embedded_dataset`].
fn load_resolver_from_disk(
    categories_dir: &Path,
    formats_dir: &Path,
    named_types_bytes: &[u8],
) -> Result<DatasetRuleResolver> {
    let named_resolver = DatasetRuleResolver::load_from_dataset(
        &[], &[], Some(named_types_bytes),
    )?;
    let named_types = named_resolver.named_types.keys().cloned().collect();
    let mut report = ValidationReport::new();
    let format_defns = crate::format::validate_all_format_files_from_disk(
        formats_dir,
        None,
        &mut report,
    );
    let format_ids = format_defns.iter().filter_map(|defn| defn.short_id).collect();
    let format_syntax_ids = format_defns.iter()
        .filter(|defn| defn.syntax.is_some())
        .filter_map(|defn| defn.short_id).collect();
    let format_registry = crate::syntax::build_format_validation_registry(&format_defns);
    let mut dc_defns = crate::validation::validate_all_dc_files_from_disk(
        categories_dir,
        &format_ids,
        None,
        Some(&named_types),
        None,
        Some(&format_syntax_ids),
        Some(&format_registry),
        &mut report,
    );
    report.ensure_valid().with_context(|| {
        format!("Invalid syntax dataset under {}", categories_dir.display())
    })?;
    ensure!(
        !dc_defns.is_empty(),
        "No Document Character definitions were loaded from {}",
        categories_dir.display()
    );
    // Match the canonical embedded ordering (lookup.rs sorts by long ID) so
    // delimiter precedence is identical to the embedded dataset build.
    dc_defns.sort_by_key(|defn| defn.dc_id);
    ensure!(
        !format_defns.is_empty(),
        "No format definitions were loaded from {}",
        formats_dir.display()
    );
    DatasetRuleResolver::load_from_dataset(
        &dc_defns,
        &format_defns,
        Some(named_types_bytes),
    )
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
    use crate::syntax::parser::parse_dc_syntax;

    #[crate::ctb_test]
    fn test_authoritative_grammar_rejects_invalid_named_types() -> Result<()> {
        for (csv, expected) in [
            ("Named type,Syntax\nbroken,:[\n", "invalid syntax"),
            ("Named type,Syntax\nbroken,:(260){1..bad}\n", "invalid syntax"),
            ("Named type,Syntax\nbroken\n", "has no syntax"),
            ("Named type,Syntax\nbroken,\n", "has no syntax"),
            ("Named type,Syntax\n,:(260)\n", "missing named type"),
        ] {
            let error = DatasetRuleResolver::load_from_dataset(
                &[], &[], Some(csv.as_bytes()),
            ).err().context("Invalid named grammar was accepted")?;
            let message = format!("{error:#}");
            assert!(message.contains("README-named-types.csv row 2"), "{message}");
            assert!(message.contains(expected), "{message}");
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_authoritative_grammar_generation_preserves_output_on_error() -> Result<()> {
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("data");
        for invalid_source in ["named", "category", "format"] {
            let temp = tempfile::tempdir()?;
            let data = temp.path().join("src/formats/dcdata/data");
            for relative in ["categories", "categories/formats"] {
                let target_dir = data.join(relative);
                std::fs::create_dir_all(&target_dir)?;
                for entry in std::fs::read_dir(source.join(relative))? {
                    let entry = entry?;
                    if entry.file_type()?.is_file() {
                        std::fs::copy(entry.path(), target_dir.join(entry.file_name()))?;
                    }
                }
            }
            std::fs::copy(
                source.join("README-named-types.csv"),
                data.join("README-named-types.csv"),
            )?;
            let output = temp.path().join(
                "src/formats/dcdata/syntax/resolver/syntax_table.generated.rs",
            );
            std::fs::create_dir_all(output.parent().context("Missing output parent")?)?;
            generate_syntax_table_file(temp.path())?;
            let original_output = std::fs::read(&output)?;
            assert!(!original_output.is_empty());
            let expected = if invalid_source == "named" {
                std::fs::write(
                    data.join("README-named-types.csv"),
                    "Named type,Syntax\nbroken,:[\n",
                )?;
                "README-named-types.csv"
            } else {
                let relative = if invalid_source == "category" {
                    "categories/el.csv"
                } else {
                    "categories/formats/hash.csv"
                };
                let path = data.join(relative);
                let mut reader = csv::ReaderBuilder::new().flexible(true).from_path(&path)?;
                let headers = reader.headers()?.clone();
                let syntax_column = headers.iter().position(|header| {
                    header.contains("Syntax") || header.contains("syntax")
                }).context("Missing fixture syntax column")?;
                let mut records = reader.records().collect::<std::result::Result<Vec<_>, _>>()?;
                let record = records.first_mut().context("Missing fixture row")?;
                let fields: Vec<String> = record.iter().enumerate().map(|(index, value)| {
                    if index == syntax_column { ":~ [".to_string() } else { value.to_string() }
                }).collect();
                *record = csv::StringRecord::from(fields);
                let mut writer = csv::WriterBuilder::new().flexible(true).from_path(&path)?;
                writer.write_record(&headers)?;
                for record in records {
                    writer.write_record(&record)?;
                }
                writer.flush()?;
                if invalid_source == "category" { "el.csv" } else { "hash.csv" }
            };
            let error = generate_syntax_table_file(temp.path())
                .err().context("Invalid dataset generated a reduced table")?;
            let message = format!("{error:#}");
            assert!(message.contains(expected), "{message}");
            assert!(message.contains("syntax"), "{message}");
            assert_eq!(std::fs::read(&output)?, original_output);
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_authoritative_grammar_rejects_invalid_quantifiers() -> Result<()> {
        for bounds in [
            "bad..3", "1..bad", "3..1", "bad,3", "1,bad", "3,1",
            "1..2..3", "1,2,3", "", "bad", "-1..3",
            "999999999999999999999999999999999999999999..3",
            "1..999999999999999999999999999999999999999999",
        ] {
            let raw = format!(":(260){{{bounds}}}");
            assert!(parse_dc_syntax(&raw).is_err(), "accepted {raw}");
        }
        assert!(parse_dc_syntax(":(260){1..3").is_err());
        for bounds in ["2", "0..3", "..3", "1..", "..", ",3", "1,", ","] {
            parse_dc_syntax(&format!(":(260){{{bounds}}}"))?;
        }
        Ok(())
    }

    /// Smoke-tests the emitter against a small hand-built resolver so
    /// emitter regressions surface without regenerating the full table.
    #[crate::ctb_test]
    fn test_syntax_table_codegen_smoke() -> Result<()> {
        let mut resolver = DatasetRuleResolver::new();
        let rule = parse_dc_syntax(":~ [^204 255]+ 204")?;
        resolver.add_rule(CharTarget::Dc(203), rule);
        resolver.register_closing_delimiter(
            crate::dc::DC_END_ENCAPSULATION_BINARY.to_u128(),
            CharTarget::Dc(203),
        );
        resolver.add_named_type(
            "string".to_string(),
            parse_dc_syntax(":260:")?.pattern,
        );
        resolver.register_deprecated(
            CharTarget::Dc(313),
            Some(CharTarget::Dc(559)),
        );
        resolver.precompute_derived_tables();

        let code = generate_syntax_table_code(&resolver);
        assert!(
            code.contains(
                "pub(super) fn populate(resolver: &mut DatasetRuleResolver)"
            ),
            "missing populate entry point"
        );
        assert!(
            code.contains("CharTarget::Dc(203)"),
            "missing binary encapsulation rule"
        );
        assert!(
            code.contains("SyntaxTerm::CharSet { negated: true,"),
            "missing negated charset term"
        );
        assert!(
            code.contains("resolver.closing_delimiters.insert("),
            "missing closing delimiter registration"
        );
        assert!(
            code.contains(
                "resolver.deprecated_targets.insert(CharTarget::Dc(313), Some(CharTarget::Dc(559)));"
            ),
            "missing <prefer> deprecation"
        );
        assert!(
            code.contains("\"string\".to_string()"),
            "missing named type registration"
        );
        assert!(
            code.contains("resolver.opaque_containers.extend("),
            "missing opaque container registration"
        );
        Ok(())
    }
}
