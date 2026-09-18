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

//! Unified parser and validator for the overloaded Aliases / Base / Chain / Syntax column.
//!
//! This column appears as Column 6 in Format category tables (`"Base/Related Format/Category,
//! Chain (@), or Syntax (:)"`) and Column 9 in Dc category tables and unified schemas
//! (`"Aliases; >=xref, <=decompos., :=Dc syntax, @chain"`).
//!
//! It parses comma-separated directives:
//! - `@base(...)`: Base / related format or parent category using Dc shorthand syntax.
//! - `@chain(...)` or `@(...)`: Format composition specification DSL.
//! - `:<syntax>`: Document Character syntax rule DSL.
//! - `><xref>`: Cross-reference.
//! - `<<tag>payload>`: Decomposition tag (`<approx>`, `<equiv>`, `<semantic>`, etc.).
//! - Bare words: Permitted as aliases in Character tables; disallowed in Format tables.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::report::ValidationReport;
use ctb_storage_minimal::shorthand::is_valid_shorthand;

/// Parsed formal alias with kind and alias name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedFormalAlias {
    pub kind: String,
    pub alias: String,
}

/// Structured representation of parsed elements from an Aliases / Base / Chain / Syntax cell.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParsedAliasesOrBaseColumn {
    /// Plain aliases (character tables only).
    pub aliases: Vec<String>,
    /// Formal aliases extracted from `@formalAliasCorrection(...)`, `@formalAliasControl(...)`, etc.
    pub formal_aliases: Vec<ParsedFormalAlias>,
    /// Base format or parent category references extracted from `@base(...)`.
    pub base_formats: Vec<String>,
    /// Cross-reference targets extracted from `@xref(...)` and `><target>`.
    pub cross_references: Vec<String>,
    /// Decomposition expressions extracted from `<<tag>payload>`.
    pub decompositions: Vec<String>,
    /// Annotations extracted from `@annotation(...)`.
    pub annotations: Vec<String>,
    /// Syntax specification string extracted from `:<syntax>`.
    pub syntax_raw: Option<String>,
    /// Format specification DSL string extracted from `@chain(...)` or `@(...)`.
    pub format_spec_raw: Option<String>,
    /// Rust identifier extracted from `@ident(...)`.
    pub rust_ident: Option<String>,
    /// Nicknames extracted from `@nick(...)`.
    pub nicknames: Vec<String>,
    /// OS associations extracted from `@os(...)` containing format shorthands.
    pub os_associations: Vec<String>,
}

/// Parses and validates the contents of an Aliases / Base / Chain / Syntax column cell.
///
/// Enforces RFC4180 spacing and comma rules:
/// - No leading or trailing whitespace.
/// - Every comma must be followed by exactly one space (`", "`).
/// - No whitespace before a comma.
/// - No consecutive commas or leading/trailing commas.
///
/// When `is_format_table` is `true`, bare format names lacking a directive are rejected with
/// an error instructing migration to `@base(...)`.
pub fn parse_aliases_or_base_column(
    raw: &str,
    file_path: &str,
    line_no: usize,
    report: &mut ValidationReport,
    is_format_table: bool,
) -> ParsedAliasesOrBaseColumn {
    let mut parsed = ParsedAliasesOrBaseColumn::default();
    let col_name = if is_format_table {
        "Base/Related Format"
    } else {
        "Aliases"
    };

    if raw.trim().is_empty() {
        if !raw.is_empty() {
            report.add_error(
                file_path,
                Some(line_no),
                Some(col_name),
                format!("Cell consists solely of whitespace in {col_name} column: '{raw}'"),
                Some("Leave the cell blank if there are no directives or aliases"),
            );
        }
        return parsed;
    }

    if raw.starts_with(char::is_whitespace) {
        report.add_error(
            file_path,
            Some(line_no),
            Some(col_name),
            format!("Cell has leading whitespace in {col_name} column: '{raw}'"),
            Some("Remove leading spaces from the cell"),
        );
    }
    if raw.ends_with(char::is_whitespace) {
        report.add_error(
            file_path,
            Some(line_no),
            Some(col_name),
            format!("Cell has trailing whitespace in {col_name} column: '{raw}'"),
            Some("Remove trailing spaces from the cell"),
        );
    }

    // Tokenize on commas while checking spacing rules.
    // Parentheses in format specs or syntax rules can contain commas or complex nested syntax,
    // but top-level column items are separated by ", ".
    let chars: Vec<(usize, char)> = raw.char_indices().collect();
    let num_chars = chars.len();
    let mut item_start = 0_usize;
    let mut paren_depth = 0_usize;
    let mut bracket_depth = 0_usize;
    let mut in_quotes = false;
    let mut is_escaped = false;

    for (i, &(byte_idx, ch)) in chars.iter().enumerate() {
        if is_escaped {
            is_escaped = false;
            continue;
        }
        if ch == '\\' {
            is_escaped = true;
            continue;
        }
        if ch == '"' {
            in_quotes = !in_quotes;
            continue;
        }
        if in_quotes {
            continue;
        }
        match ch {
            '(' => paren_depth = paren_depth.saturating_add(1),
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '[' => bracket_depth = bracket_depth.saturating_add(1),
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            ',' if paren_depth == 0 && bracket_depth == 0 => {
                // Check space before comma
                if i > 0 {
                    if let Some(&(_, prev_ch)) = chars.get(i.saturating_sub(1)) {
                        if prev_ch.is_whitespace() {
                            let snippet_start = byte_idx.saturating_sub(10);
                            let snippet_end = byte_idx.saturating_add(1).min(raw.len());
                            let snippet = raw.get(snippet_start..snippet_end).unwrap_or(",");
                            report.add_error(
                                file_path,
                                Some(line_no),
                                Some(col_name),
                                format!(
                                    "Extra space before ',' in {col_name} column: '{snippet}'"
                                ),
                                Some("Do not place spaces before commas"),
                            );
                        }
                    }
                } else {
                    report.add_error(
                        file_path,
                        Some(line_no),
                        Some(col_name),
                        format!("Leading comma in {col_name} column: '{raw}'"),
                        Some("Remove leading comma"),
                    );
                }

                // Check space after comma
                let next_i = i.saturating_add(1);
                if next_i >= num_chars {
                    report.add_error(
                        file_path,
                        Some(line_no),
                        Some(col_name),
                        format!("Trailing comma in {col_name} column: '{raw}'"),
                        Some("Remove trailing comma"),
                    );
                } else if let Some(&(_, next_ch)) = chars.get(next_i) {
                    if next_ch == ',' {
                        report.add_error(
                            file_path,
                            Some(line_no),
                            Some(col_name),
                            format!("Consecutive commas in {col_name} column: '{raw}'"),
                            Some("Remove redundant comma"),
                        );
                    } else if next_ch != ' ' {
                        let snippet_end = byte_idx.saturating_add(15).min(raw.len());
                        let snippet = raw.get(byte_idx..snippet_end).unwrap_or(",");
                        report.add_error(
                            file_path,
                            Some(line_no),
                            Some(col_name),
                            format!("Missing space after ',' in {col_name} column: '{snippet}'"),
                            Some("Ensure every comma is followed by a single space (e.g. ', ')"),
                        );
                    } else {
                        let next_next_i = next_i.saturating_add(1);
                        if let Some(&(_, after_space_ch)) = chars.get(next_next_i) {
                            if after_space_ch.is_whitespace() {
                                let snippet_end = byte_idx.saturating_add(15).min(raw.len());
                                let snippet = raw.get(byte_idx..snippet_end).unwrap_or(",");
                                report.add_error(
                                    file_path,
                                    Some(line_no),
                                    Some(col_name),
                                    format!(
                                        "Multiple spaces after ',' in {col_name} column: '{snippet}'"
                                    ),
                                    Some("Use exactly one space after each comma"),
                                );
                            }
                        }
                    }
                }

                if let Some(item_raw) = raw.get(item_start..byte_idx) {
                    process_column_item(
                        item_raw,
                        file_path,
                        line_no,
                        report,
                        is_format_table,
                        col_name,
                        &mut parsed,
                    );
                }

                item_start = byte_idx.saturating_add(1);
            }
            _ => {}
        }
    }

    if in_quotes {
        report.add_error(
            file_path,
            Some(line_no),
            Some(col_name),
            format!("Unterminated string quote in {col_name} column: '{raw}'"),
            Some("Ensure all double quotes are closed"),
        );
    }

    if let Some(last_raw) = raw.get(item_start..) {
        process_column_item(
            last_raw,
            file_path,
            line_no,
            report,
            is_format_table,
            col_name,
            &mut parsed,
        );
    }

    parsed
}

/// Unescapes a double-quoted string payload inside a directive (e.g. `"foo \"bar\""` -> `foo "bar"`).
fn unescape_quoted_directive_payload(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    let content = trimmed.strip_prefix('"')?.strip_suffix('"')?;
    let mut res = String::with_capacity(content.len());
    let mut chars = content.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(&next_c) = chars.peek() {
                if next_c == '"' || next_c == '\\' {
                    res.push(next_c);
                    chars.next();
                    continue;
                }
            }
            res.push('\\');
        } else {
            res.push(c);
        }
    }
    Some(res)
}

fn process_column_item(
    item_raw: &str,
    file_path: &str,
    line_no: usize,
    report: &mut ValidationReport,
    is_format_table: bool,
    col_name: &str,
    parsed: &mut ParsedAliasesOrBaseColumn,
) {
    let item_trimmed = item_raw.trim();
    if item_trimmed.is_empty() {
        return;
    }

    if item_trimmed.starts_with('@') {
        if let Some(inner) = item_trimmed.strip_prefix("@base(") {
            if let Some(stripped) = inner.strip_suffix(')') {
                let base_val = stripped.trim();
                if base_val.is_empty() {
                    report.add_error(
                        file_path,
                        Some(line_no),
                        Some(col_name),
                        format!("Empty '@base()' directive in {col_name} column: '{item_trimmed}'"),
                        Some("Specify a format shorthand inside @base(...) (e.g. '@base(f161)')"),
                    );
                } else {
                    validate_base_shorthand_expr(base_val, file_path, line_no, report, col_name);
                    parsed.base_formats.push(base_val.to_string());
                }
            } else {
                report.add_error(
                    file_path,
                    Some(line_no),
                    Some(col_name),
                    format!("Malformed '@base(...)': missing closing parenthesis in '{item_trimmed}'"),
                    Some("Ensure '@base(...)' closes with a parenthesis"),
                );
            }
        } else if let Some(inner) = item_trimmed.strip_prefix("@xref(") {
            if let Some(stripped) = inner.strip_suffix(')') {
                let target = stripped.trim();
                if target.is_empty() {
                    report.add_error(
                        file_path,
                        Some(line_no),
                        Some(col_name),
                        format!("Empty '@xref()' directive in {col_name} column: '{item_trimmed}'"),
                        Some("Specify a target shorthand inside @xref(...) (e.g. '@xref(u22ee)')"),
                    );
                } else if !is_valid_shorthand(target) {
                    report.add_error(
                        file_path,
                        Some(line_no),
                        Some(col_name),
                        format!("Invalid target shorthand in '@xref({target})'"),
                        Some("Specify a valid shorthand like 'u22ee', short Dc, or format 'f161'"),
                    );
                } else {
                    parsed.cross_references.push(target.to_string());
                }
            } else {
                report.add_error(
                    file_path,
                    Some(line_no),
                    Some(col_name),
                    format!("Malformed '@xref(...)': missing closing parenthesis in '{item_trimmed}'"),
                    Some("Ensure '@xref(...)' closes with a parenthesis"),
                );
            }
        } else if let Some(inner) = item_trimmed.strip_prefix("@annotation(") {
            if let Some(stripped) = inner.strip_suffix(')') {
                if let Some(unescaped) = unescape_quoted_directive_payload(stripped) {
                    parsed.annotations.push(unescaped);
                } else {
                    report.add_error(
                        file_path,
                        Some(line_no),
                        Some(col_name),
                        format!("Malformed '@annotation(...)': content must be enclosed in double quotes in '{item_trimmed}'"),
                        Some("Use '@annotation(\"...\")' with internal quotes escaped as '\\\"'"),
                    );
                }
            } else {
                report.add_error(
                    file_path,
                    Some(line_no),
                    Some(col_name),
                    format!("Malformed '@annotation(...)': missing closing parenthesis in '{item_trimmed}'"),
                    Some("Ensure '@annotation(...)' closes with a parenthesis"),
                );
            }
        } else if item_trimmed.starts_with("@formalAlias") {
            let formal_types = [
                ("Correction", "@formalAliasCorrection("),
                ("Control", "@formalAliasControl("),
                ("Alternate", "@formalAliasAlternate("),
                ("Figment", "@formalAliasFigment("),
                ("Abbreviation", "@formalAliasAbbreviation("),
            ];
            let mut matched = false;
            for (kind, prefix) in formal_types {
                if let Some(inner) = item_trimmed.strip_prefix(prefix) {
                    matched = true;
                    if let Some(stripped) = inner.strip_suffix(')') {
                        if let Some(unescaped) = unescape_quoted_directive_payload(stripped) {
                            parsed.formal_aliases.push(ParsedFormalAlias {
                                kind: kind.to_string(),
                                alias: unescaped,
                            });
                        } else {
                            report.add_error(
                                file_path,
                                Some(line_no),
                                Some(col_name),
                                format!("Malformed '{prefix}...)': alias must be enclosed in double quotes in '{item_trimmed}'"),
                                Some("Use '@formalAlias<Type>(\"...\")'"),
                            );
                        }
                    } else {
                        report.add_error(
                            file_path,
                            Some(line_no),
                            Some(col_name),
                            format!("Malformed '{prefix}...)': missing closing parenthesis in '{item_trimmed}'"),
                            Some("Ensure directive closes with a parenthesis"),
                        );
                    }
                    break;
                }
            }
            if !matched {
                report.add_error(
                    file_path,
                    Some(line_no),
                    Some(col_name),
                    format!("Unknown formal alias directive '{item_trimmed}' in {col_name} column"),
                    Some("Supported formal alias directives are: @formalAliasCorrection, @formalAliasControl, @formalAliasAlternate, @formalAliasFigment, @formalAliasAbbreviation"),
                );
            }
        } else if item_trimmed.starts_with("@chain(") || item_trimmed.starts_with("@(") {
            parsed.format_spec_raw = Some(item_trimmed.to_string());
        } else if let Some(inner) = item_trimmed.strip_prefix("@ident(") {
            if let Some(stripped) = inner.strip_suffix(')') {
                if let Some(unescaped) = unescape_quoted_directive_payload(stripped) {
                    if let Err(e) = crate::validate_rust_identifier(&unescaped) {
                        report.add_error(
                            file_path,
                            Some(line_no),
                            Some(col_name),
                            format!("Invalid Rust identifier in '@ident(\"{unescaped}\")': {e}"),
                            Some("Identifier must be a valid Rust identifier (e.g. '@ident(\"TarGz\")')"),
                        );
                    }
                    parsed.rust_ident = Some(unescaped);
                } else {
                    report.add_error(
                        file_path,
                        Some(line_no),
                        Some(col_name),
                        format!("Malformed '@ident(...)': content must be enclosed in double quotes in '{item_trimmed}'"),
                        Some("Use '@ident(\"...\")' with internal quotes escaped as '\\\"'"),
                    );
                }
            } else {
                report.add_error(
                    file_path,
                    Some(line_no),
                    Some(col_name),
                    format!("Malformed '@ident(...)': missing closing parenthesis in '{item_trimmed}'"),
                    Some("Ensure '@ident(...)' closes with a parenthesis"),
                );
            }
        } else if let Some(inner) = item_trimmed.strip_prefix("@nick(") {
            if let Some(stripped) = inner.strip_suffix(')') {
                if let Some(unescaped) = unescape_quoted_directive_payload(stripped) {
                    if unescaped.trim().is_empty() {
                        report.add_error(
                            file_path,
                            Some(line_no),
                            Some(col_name),
                            format!("Empty '@nick()' in '{item_trimmed}'"),
                            Some("Provide a non-empty nickname string inside '@nick(\"...\")'"),
                        );
                    } else {
                        parsed.nicknames.push(unescaped);
                    }
                } else {
                    report.add_error(
                        file_path,
                        Some(line_no),
                        Some(col_name),
                        format!("Malformed '@nick(...)': content must be enclosed in double quotes in '{item_trimmed}'"),
                        Some("Use '@nick(\"...\")' with internal quotes escaped as '\\\"'"),
                    );
                }
            } else {
                report.add_error(
                    file_path,
                    Some(line_no),
                    Some(col_name),
                    format!("Malformed '@nick(...)': missing closing parenthesis in '{item_trimmed}'"),
                    Some("Ensure '@nick(...)' closes with a parenthesis"),
                );
            }
        } else if let Some(inner) = item_trimmed.strip_prefix("@os(") {
            if let Some(stripped) = inner.strip_suffix(')') {
                let os_val = stripped.trim();
                if os_val.is_empty() {
                    report.add_error(
                        file_path,
                        Some(line_no),
                        Some(col_name),
                        format!("Empty '@os()' directive in {col_name} column: '{item_trimmed}'"),
                        Some("Specify a format shorthand inside @os(...) (e.g. '@os(f405)')"),
                    );
                } else {
                    for token in os_val.split(',') {
                        let token_trimmed = token.trim();
                        if token_trimmed.is_empty() {
                            report.add_error(
                                file_path,
                                Some(line_no),
                                Some(col_name),
                                format!("Empty OS shorthand token in '{item_trimmed}'"),
                                Some("Provide a non-empty format shorthand in '@os(...)'"),
                            );
                            continue;
                        }
                        match ctb_storage_minimal::shorthand::parse_format_shorthand(token_trimmed) {
                            Ok(_) => {
                                parsed.os_associations.push(token_trimmed.to_string());
                            }
                            Err(_) => {
                                report.add_error(
                                    file_path,
                                    Some(line_no),
                                    Some(col_name),
                                    format!("Invalid format shorthand '{token_trimmed}' in '@os(...)'. Only format shorthands (e.g. 'f405') are supported"),
                                    Some("Use format shorthands like 'f405' (MacOs), 'f402' (Windows), 'f492' (Unix)"),
                                );
                            }
                        }
                    }
                }
            } else {
                report.add_error(
                    file_path,
                    Some(line_no),
                    Some(col_name),
                    format!("Malformed '@os(...)': missing closing parenthesis in '{item_trimmed}'"),
                    Some("Ensure '@os(...)' closes with a parenthesis"),
                );
            }
        } else {
            report.add_error(
                file_path,
                Some(line_no),
                Some(col_name),
                format!("Unknown directive '{item_trimmed}' in {col_name} column"),
                Some("Supported '@' directives are '@base(...)', '@chain(...)', '@xref(...)', '@formalAlias<Type>(...)', '@annotation(...)', '@ident(...)', '@nick(...)', and '@os(...)'"),
            );
        }
    } else if item_trimmed.starts_with('=') {
        report.add_error(
            file_path,
            Some(line_no),
            Some(col_name),
            format!("Legacy chain syntax '=' is deprecated: '{item_trimmed}'. Use '@chain(...)' instead"),
            Some("Update to '@chain(...)' format specification syntax"),
        );
    } else if item_trimmed.starts_with(':') {
        if item_trimmed.contains("  ") {
            report.add_error(
                file_path,
                Some(line_no),
                Some(col_name),
                format!("Multiple consecutive spaces in syntax declaration: '{item_trimmed}'"),
                Some("Use single spaces in syntax declarations"),
            );
        }
        parsed.syntax_raw = Some(item_trimmed.to_string());
    } else if let Some(rest) = item_trimmed.strip_prefix('>') {
        if !rest.starts_with('<') && rest.starts_with(char::is_whitespace) {
            let token = rest.split_whitespace().next().unwrap_or("");
            report.add_error(
                file_path,
                Some(line_no),
                Some(col_name),
                format!("Extra space before '{token}' in cross-reference: '> {token}'"),
                Some(
                    "Cross-reference '>' must be followed immediately by target without spaces (e.g. '>32')",
                ),
            );
        }
        parsed.cross_references.push(item_trimmed.to_string());
    } else if item_trimmed.starts_with('<') {
        if let Some((tag_raw, payload)) = item_trimmed.split_once('>') {
            let tag_name = tag_raw.strip_prefix('<').unwrap_or(tag_raw);
            let tag_full = format!("{tag_raw}>");
            if tag_name.is_empty() {
                report.add_error(
                    file_path,
                    Some(line_no),
                    Some(col_name),
                    format!("Empty decomposition tag: '<>' in '{item_trimmed}'"),
                    Some("Provide a tag name between angle brackets (e.g. '<approx>', '<equiv>')"),
                );
            } else {
                if tag_name.starts_with(char::is_whitespace) {
                    report.add_error(
                        file_path,
                        Some(line_no),
                        Some(col_name),
                        format!("Extra space after '<' in decomposition tag: '{tag_full}'"),
                        Some("Remove leading space inside '<...>' tag"),
                    );
                }
                if tag_name.ends_with(char::is_whitespace) {
                    report.add_error(
                        file_path,
                        Some(line_no),
                        Some(col_name),
                        format!("Extra space before '>' in decomposition tag: '{tag_full}'"),
                        Some("Remove trailing space inside '<...>' tag"),
                    );
                }
                if tag_name.trim().contains(char::is_whitespace) {
                    report.add_error(
                        file_path,
                        Some(line_no),
                        Some(col_name),
                        format!("Whitespace inside decomposition tag: '{tag_full}'"),
                        Some("Remove whitespace from decomposition tag name (e.g. '<approx>')"),
                    );
                }
            }
            if payload.starts_with(char::is_whitespace) {
                let first_token = payload
                    .split(|c: char| c.is_whitespace() || c == ',')
                    .find(|s| !s.is_empty())
                    .unwrap_or("");
                let full_sample = if first_token.is_empty() {
                    tag_full.clone()
                } else {
                    format!("{tag_full} {first_token}")
                };
                report.add_error(
                    file_path,
                    Some(line_no),
                    Some(col_name),
                    if first_token.is_empty() {
                        format!("Extra space after closing tag in decomposition: '{tag_full}'")
                    } else {
                        format!("Extra space before '{first_token}' in decomposition: '{full_sample}'")
                    },
                    Some(
                        "Decomposition tags must be followed immediately by target without spaces (e.g. '<tag>{token}')",
                    ),
                );
                let trimmed_payload = payload.trim_start();
                let corrected = format!("{tag_full}{trimmed_payload}");
                parsed.decompositions.push(corrected);
            } else {
                parsed.decompositions.push(item_trimmed.to_string());
            }

            if payload.trim().contains("  ") {
                report.add_error(
                    file_path,
                    Some(line_no),
                    Some(col_name),
                    format!(
                        "Multiple spaces between tokens in decomposition payload: '{tag_full}{payload}'"
                    ),
                    Some("Use single spaces between decomposition tokens"),
                );
            }
        } else {
            report.add_error(
                file_path,
                Some(line_no),
                Some(col_name),
                format!("Unclosed decomposition tag: '{item_trimmed}'"),
                Some("Close tag with '>'"),
            );
            parsed.decompositions.push(item_trimmed.to_string());
        }
    } else {
        // Bare item without a prefix/directive
        if is_format_table {
            report.add_error(
                file_path,
                Some(line_no),
                Some(col_name),
                format!("Bare base format '{item_trimmed}' is disallowed in format tables. Use '@base(...)' instead"),
                Some("Wrap base format references in '@base(shorthand)' (e.g. '@base(f161)')"),
            );
            parsed.base_formats.push(item_trimmed.to_string());
        } else {
            if item_trimmed.contains("  ") {
                report.add_error(
                    file_path,
                    Some(line_no),
                    Some(col_name),
                    format!("Multiple consecutive spaces in alias: '{item_trimmed}'"),
                    Some("Use single spaces between words in aliases"),
                );
            }
            parsed.aliases.push(item_trimmed.to_string());
        }
    }
}

/// Validates that tokens in a `@base(...)` expression conform to Dc shorthand syntax.
fn validate_base_shorthand_expr(
    expr: &str,
    file_path: &str,
    line_no: usize,
    report: &mut ValidationReport,
    col_name: &str,
) {
    // Split by operational delimiters &, |, (, ) to validate atomic identifiers
    for token in expr
        .split(|c: char| c == '&' || c == '|' || c == '(' || c == ')')
        .map(str::trim)
        .filter(|t| !t.is_empty())
    {
        if !is_valid_shorthand(token) {
            report.add_error(
                file_path,
                Some(line_no),
                Some(col_name),
                format!(
                    "Invalid Dc shorthand token '{token}' in '@base({expr})'. Must use format shorthands (e.g. 'f161')"
                ),
                Some("Ensure all base references inside '@base(...)' use Dc shorthand syntax"),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[crate::ctb_test]
    fn test_parse_aliases_or_base_column_directives() {
        let mut report = ValidationReport::new();
        let parsed = parse_aliases_or_base_column(
            "@base(f161), @chain(f161 > f35)",
            "test.csv",
            1,
            &mut report,
            true,
        );
        assert!(!report.has_errors(), "Report errors: {}", report.format_report());
        assert_eq!(parsed.base_formats, vec!["f161"]);
        assert_eq!(parsed.format_spec_raw.as_deref(), Some("@chain(f161 > f35)"));
    }

    #[crate::ctb_test]
    fn test_bare_base_in_format_table_fails_validation() {
        let mut report = ValidationReport::new();
        let _ = parse_aliases_or_base_column(
            "Tar, @chain(f161 > f35)",
            "test.csv",
            1,
            &mut report,
            true,
        );
        assert!(report.has_errors());
        let err = report.format_report();
        assert!(err.contains("Bare base format 'Tar' is disallowed in format tables"));
    }

    #[crate::ctb_test]
    fn test_bare_alias_in_character_table_succeeds() {
        let mut report = ValidationReport::new();
        let parsed = parse_aliases_or_base_column(
            "dot, point, >32, <approx>u2e",
            "test.csv",
            1,
            &mut report,
            false,
        );
        assert!(!report.has_errors(), "Report errors: {}", report.format_report());
        assert_eq!(parsed.aliases, vec!["dot", "point"]);
        assert_eq!(parsed.cross_references, vec![">32"]);
        assert_eq!(parsed.decompositions, vec!["<approx>u2e"]);
    }

    #[crate::ctb_test]
    fn test_parse_formal_aliases_xrefs_and_annotations() {
        let mut report = ValidationReport::new();
        let parsed = parse_aliases_or_base_column(
            r#"@formalAliasCorrection("PRESENTATION FORM FOR VERTICAL RIGHT WHITE LENTICULAR BRACKET"), @formalAliasAbbreviation("NUL"), @xref(u22ee), @annotation("misspelling of \"BRACKET\" in character name is a known defect"), @annotation("German, note with comma")"#,
            "test.csv",
            1,
            &mut report,
            false,
        );
        assert!(!report.has_errors(), "Report errors: {}", report.format_report());
        assert_eq!(
            parsed.formal_aliases,
            vec![
                ParsedFormalAlias {
                    kind: "Correction".to_string(),
                    alias: "PRESENTATION FORM FOR VERTICAL RIGHT WHITE LENTICULAR BRACKET".to_string(),
                },
                ParsedFormalAlias {
                    kind: "Abbreviation".to_string(),
                    alias: "NUL".to_string(),
                }
            ]
        );
        assert_eq!(parsed.cross_references, vec!["u22ee"]);
        assert_eq!(
            parsed.annotations,
            vec![
                "misspelling of \"BRACKET\" in character name is a known defect",
                "German, note with comma",
            ]
        );
    }

    #[crate::ctb_test]
    fn test_parse_ident_and_nick_directives() {
        let mut report = ValidationReport::default();
        let parsed = parse_aliases_or_base_column(
            r#"@ident("TarGz"), @nick("tgz"), @nick("tar-gz"), @base(f161)"#,
            "test_format.csv",
            1,
            &mut report,
            true,
        );
        assert!(!report.has_errors(), "Errors: {}", report.format_report());
        assert_eq!(parsed.rust_ident.as_deref(), Some("TarGz"));
        assert_eq!(parsed.nicknames, vec!["tgz", "tar-gz"]);
        assert_eq!(parsed.base_formats, vec!["f161"]);

        // Test invalid ident fails validation
        let mut err_report = ValidationReport::default();
        let _ = parse_aliases_or_base_column(
            r#"@ident("123Invalid")"#,
            "test_format.csv",
            1,
            &mut err_report,
            true,
        );
        assert!(err_report.has_errors());
        assert!(err_report.format_report().contains("Invalid Rust identifier"));
    }

    #[crate::ctb_test]
    fn test_parse_os_directive() {
        let mut report = ValidationReport::default();
        let parsed = parse_aliases_or_base_column(
            r#"@os(f405), @os(f406), @base(f161)"#,
            "test_format.csv",
            1,
            &mut report,
            true,
        );
        assert!(!report.has_errors(), "Errors: {}", report.format_report());
        assert_eq!(parsed.os_associations, vec!["f405", "f406"]);
        assert_eq!(parsed.base_formats, vec!["f161"]);

        // Test invalid OS token (non-shorthand text like "macos" is disallowed)
        let mut err_report = ValidationReport::default();
        let _ = parse_aliases_or_base_column(
            r#"@os(macos)"#,
            "test_format.csv",
            1,
            &mut err_report,
            true,
        );
        assert!(err_report.has_errors());
        assert!(err_report.format_report().contains("Invalid format shorthand 'macos' in '@os(...)'"));
    }
}
