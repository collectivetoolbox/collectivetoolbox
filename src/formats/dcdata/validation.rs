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

//! Comprehensive table validators for Document Characters (Dcs), Formats, and
//! Global Graph Layout datasets.

#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

use std::collections::{HashMap, HashSet};
use std::path::Path;

use include_dir::Dir;

use crate::dc::{SHORT_DC_REGION_END, SHORT_DC_REGION_START};
use crate::dc_def::DcDefn;
use crate::format::{
    validate_all_format_files, validate_all_format_files_from_disk,
};
use crate::layout::validate_layout_table;
use crate::report::ValidationReport;
use crate::shared::{
    BidiClass, GeneralCategory, validate_bidi_class, validate_combining_class,
    validate_cross_table_uniqueness, validate_general_category,
};
use crate::syntax::{
    CharTarget, parse_dc_syntax, parse_target_token, validate_dc_syntax,
};
use crate::{FORMATS_CATEGORIES_DIR, get_dc_categories_dir};

/// Runs comprehensive validation across all repository data tables strictly in memory
/// using the embedded directory asset bundles.
pub fn validate_all_data_tables() -> ValidationReport {
    validate_all_data_tables_embedded()
}

/// Runs validation using the embedded directory snapshots.
pub fn validate_all_data_tables_embedded() -> ValidationReport {
    let mut report = ValidationReport::new();

    // 1. Validate Global Graph Layout table
    validate_layout_table(
        ctb_storage_minimal::global_graph_layout::GLOBAL_GRAPH_LAYOUT_CSV,
        "storage/minimal/data/global-graph-layout.csv",
        &mut report,
    );

    // 2. Validate Formats category files
    let format_rows =
        validate_all_format_files(&FORMATS_CATEGORIES_DIR, &mut report);
    let known_format_ids: HashSet<usize> =
        format_rows.iter().filter_map(|r| r.short_id).collect();

    // 3. Validate Document Characters category files
    let dc_rows = if let Some(dc_dir) = get_dc_categories_dir() {
        validate_all_dc_files(
            dc_dir,
            &known_format_ids,
            &mut report,
        )
    } else {
        report.add_error(
            "data/categories",
            None,
            None,
            "Could not locate embedded categories directory",
            None,
        );
        Vec::new()
    };

    // 4. Validate Cross-Table Name / Label Uniqueness
    let dc_names: Vec<(usize, &str, &str)> = dc_rows
        .iter()
        .filter_map(|r| {
            r.short_id
                .map(|sid| (sid, r.name.as_str(), r.source_file.as_str()))
        })
        .collect();

    let format_labels: Vec<(usize, &str, &str)> = format_rows
        .iter()
        .filter_map(|r| {
            r.short_id
                .map(|sid| (sid, r.name.as_str(), r.source_file.as_str()))
        })
        .collect();

    validate_cross_table_uniqueness(&dc_names, &format_labels, &mut report);

    // 5. Validate Named Types table
    let known_dc_ids: HashSet<u32> = dc_rows
        .iter()
        .filter_map(|r| r.short_id)
        .filter_map(|id| u32::try_from(id).ok())
        .collect();

    if let Some(named_types_bytes) = crate::get_dc_data_file("README-named-types.csv") {
        validate_named_types_table(
            &named_types_bytes,
            "data/README-named-types.csv",
            &known_dc_ids,
            &known_format_ids,
            &mut report,
        );
    } else {
        report.add_error(
            "data/README-named-types.csv",
            None,
            None,
            "Could not locate embedded README-named-types.csv",
            None,
        );
    }

    report
}

/// Runs validation directly against live files in a repository directory.
pub fn validate_all_data_tables_from_repo(
    repo_root: &Path,
) -> ValidationReport {
    let mut report = ValidationReport::new();

    // 1. Validate Global Graph Layout table
    let layout_path =
        repo_root.join("src/storage/minimal/data/global-graph-layout.csv");
    if let Ok(bytes) = std::fs::read(&layout_path) {
        validate_layout_table(
            &bytes,
            "storage/minimal/data/global-graph-layout.csv",
            &mut report,
        );
    } else {
        report.add_error(
            "storage/minimal/data/global-graph-layout.csv",
            None,
            None,
            "Could not locate global-graph-layout.csv on disk",
            Some("Ensure file exists in storage/minimal/data/"),
        );
    }

    // 2. Validate Formats category files
    let formats_dir = repo_root.join("src/formats/dcdata/data/categories/formats");
    let format_rows =
        validate_all_format_files_from_disk(&formats_dir, &mut report);
    let known_format_ids: HashSet<usize> =
        format_rows.iter().filter_map(|r| r.short_id).collect();

    // 3. Validate Document Characters category files
    let dc_dir = repo_root.join("src/formats/dcdata/data/categories");
    let dc_rows = validate_all_dc_files_from_disk(
        &dc_dir,
        &known_format_ids,
        &mut report,
    );

    // 4. Validate Cross-Table Name / Label Uniqueness
    let dc_names: Vec<(usize, &str, &str)> = dc_rows
        .iter()
        .filter_map(|r| {
            r.short_id
                .map(|sid| (sid, r.name.as_str(), r.source_file.as_str()))
        })
        .collect();

    let format_labels: Vec<(usize, &str, &str)> = format_rows
        .iter()
        .filter_map(|r| {
            r.short_id
                .map(|sid| (sid, r.name.as_str(), r.source_file.as_str()))
        })
        .collect();

    validate_cross_table_uniqueness(&dc_names, &format_labels, &mut report);

    // 5. Validate Named Types table
    let known_dc_ids: HashSet<u32> = dc_rows
        .iter()
        .filter_map(|r| r.short_id)
        .filter_map(|id| u32::try_from(id).ok())
        .collect();

    let named_types_path =
        repo_root.join("src/formats/dcdata/data/README-named-types.csv");
    if let Ok(bytes) = std::fs::read(&named_types_path) {
        validate_named_types_table(
            &bytes,
            "src/formats/dcdata/data/README-named-types.csv",
            &known_dc_ids,
            &known_format_ids,
            &mut report,
        );
    } else {
        report.add_error(
            "src/formats/dcdata/data/README-named-types.csv",
            None,
            None,
            "Could not locate README-named-types.csv on disk",
            Some("Ensure file exists in src/formats/dcdata/data/"),
        );
    }

    report
}

/// Validates the README-named-types.csv table, verifying syntax declarations
/// for each named type.
pub fn validate_named_types_table(
    csv_bytes: &[u8],
    file_path: &str,
    known_dc_ids: &HashSet<u32>,
    known_format_ids: &HashSet<usize>,
    report: &mut ValidationReport,
) {
    let vec_bytes = csv_bytes.to_vec();
    let table = match csv_tools::parse_csv_reader(
        &vec_bytes,
        csv_tools::CsvParseOptions {
            has_header: true,
            flexible: true,
            ..Default::default()
        },
    ) {
        Ok(t) => t,
        Err(e) => {
            report.add_error(
                file_path,
                None,
                None,
                format!("Failed to parse CSV: {e}"),
                Some("Verify CSV syntax and formatting"),
            );
            return;
        }
    };

    let mut seen_names = HashSet::new();
    for i in 0..table.row_count() {
        let line_no = i.saturating_add(2);
        let Some(row) = table.row(i) else {
            continue;
        };

        if row.len() < 2 {
            report.add_error(
                file_path,
                Some(line_no),
                None,
                format!("Row has {} columns, expected at least 2", row.len()),
                Some("Ensure row has 'Named type' and 'Syntax' columns"),
            );
            continue;
        }

        // Reason for fallback: out-of-bounds column indices on malformed rows default to empty string so schema validation can report all diagnostics without indexing panics
        let name = row.get(0).map_or("", |s| s.trim());
        // Reason for fallback: out-of-bounds column indices on malformed rows default to empty string so schema validation can report all diagnostics without indexing panics
        let syntax = row.get(1).map_or("", |s| s.trim());

        if name.is_empty() {
            report.add_error(
                file_path,
                Some(line_no),
                Some("Named type"),
                "Named type cannot be empty".to_string(),
                Some("Specify a name for the named type construct"),
            );
        } else if !seen_names.insert(name.to_string()) {
            report.add_error(
                file_path,
                Some(line_no),
                Some("Named type"),
                format!("Duplicate named type '{name}'"),
                Some("Ensure each named type is defined uniquely"),
            );
        }

        if syntax.is_empty() {
            report.add_error(
                file_path,
                Some(line_no),
                Some("Syntax"),
                "Syntax rule cannot be empty".to_string(),
                Some("Provide a Dc syntax declaration for the named type"),
            );
        } else {
            match parse_dc_syntax(syntax) {
                Ok(rule) => {
                    validate_dc_syntax(
                        &rule,
                        0,
                        known_dc_ids,
                        known_format_ids,
                        report,
                        file_path,
                        line_no,
                    );
                }
                Err(e) => {
                    report.add_error(
                        file_path,
                        Some(line_no),
                        Some("Syntax"),
                        format!("Failed to parse syntax DSL rule: {e}"),
                        Some("Verify syntax DSL grammar"),
                    );
                }
            }
        }
    }
}


/// Parses the composite Aliases/cross-reference/decomposition/syntax column and
/// validates all spacing, delimiter, and formatting conventions in a single pass.
///
/// Ensures:
/// - No leading or trailing whitespace in the cell.
/// - Every comma is followed by exactly one space (no missing space, no
///   multiple spaces).
/// - No whitespace before a comma.
/// - No empty elements or misplaced commas (leading/trailing/consecutive
///   commas).
/// - Decompositions (`<tag>...`):
///   - No whitespace inside the tag brackets (e.g. `< approx>`, `<approx >`).
///   - No whitespace immediately following the closing `>` (e.g. `<approx> 0`
///     has extra space before 0).
///   - Tokens in decomposition payload are separated by single spaces.
/// - Cross-references (`>...`):
///   - No whitespace immediately following `>` (e.g. `> 32` has extra space
///     before 32).
/// - Plain aliases and syntax declarations:
///   - No multiple consecutive spaces.
pub fn parse_dc_aliases_column(
    raw: &str,
    file_path: &str,
    line_no: usize,
    report: &mut ValidationReport,
) -> (Vec<String>, Vec<String>, Vec<String>, Option<String>) {
    let mut aliases = Vec::new();
    let mut cross_references = Vec::new();
    let mut decompositions = Vec::new();
    let mut dc_syntax: Option<String> = None;

    if raw.trim().is_empty() {
        if !raw.is_empty() {
            report.add_error(
                file_path,
                Some(line_no),
                Some("Aliases"),
                format!("Cell consists solely of whitespace in Aliases column: '{raw}'"),
                Some("Leave the Aliases column blank if there are no aliases"),
            );
        }
        return (aliases, cross_references, decompositions, dc_syntax);
    }

    if raw.starts_with(char::is_whitespace) {
        report.add_error(
            file_path,
            Some(line_no),
            Some("Aliases"),
            format!("Cell has leading whitespace in Aliases column: '{raw}'"),
            Some("Remove leading spaces from the Aliases cell"),
        );
    }
    if raw.ends_with(char::is_whitespace) {
        report.add_error(
            file_path,
            Some(line_no),
            Some("Aliases"),
            format!("Cell has trailing whitespace in Aliases column: '{raw}'"),
            Some("Remove trailing spaces from the Aliases cell"),
        );
    }

    let chars: Vec<(usize, char)> = raw.char_indices().collect();
    let num_chars = chars.len();
    let mut depth_paren = 0usize;
    let mut depth_bracket = 0usize;
    let mut item_start = 0usize;

    for (i, &(byte_idx, ch)) in chars.iter().enumerate() {
        match ch {
            '(' => depth_paren = depth_paren.saturating_add(1),
            ')' => depth_paren = depth_paren.saturating_sub(1),
            '[' => depth_bracket = depth_bracket.saturating_add(1),
            ']' => depth_bracket = depth_bracket.saturating_sub(1),
            ',' if depth_paren == 0 && depth_bracket == 0 => {
                // Check space before comma
                if i > 0 {
                    if let Some(&(_, prev_ch)) = chars.get(i.saturating_sub(1)) {
                        if prev_ch.is_whitespace() {
                            let snippet_start = byte_idx.saturating_sub(10);
                            let snippet_end = byte_idx.saturating_add(1).min(raw.len());
                            // Reason for fallback: context slice might not align on UTF-8 char boundary, so default to comma
                            let snippet = raw.get(snippet_start..snippet_end).unwrap_or(",");
                            report.add_error(
                                file_path,
                                Some(line_no),
                                Some("Aliases"),
                                format!("Extra space before ',' in Aliases column: '{snippet}'"),
                                Some("Do not place spaces before commas"),
                            );
                        }
                    }
                } else {
                    report.add_error(
                        file_path,
                        Some(line_no),
                        Some("Aliases"),
                        format!("Leading comma in Aliases column: '{raw}'"),
                        Some("Remove leading comma"),
                    );
                }

                // Check space after comma
                let next_i = i.saturating_add(1);
                if next_i >= num_chars {
                    report.add_error(
                        file_path,
                        Some(line_no),
                        Some("Aliases"),
                        format!("Trailing comma in Aliases column: '{raw}'"),
                        Some("Remove trailing comma"),
                    );
                } else if let Some(&(_, next_ch)) = chars.get(next_i) {
                    if next_ch == ',' {
                        report.add_error(
                            file_path,
                            Some(line_no),
                            Some("Aliases"),
                            format!("Consecutive commas in Aliases column: '{raw}'"),
                            Some("Remove redundant comma"),
                        );
                    } else if next_ch != ' ' {
                        let snippet_end = byte_idx.saturating_add(15).min(raw.len());
                        // Reason for fallback: context slice might not align on UTF-8 char boundary, so default to comma
                        let snippet = raw.get(byte_idx..snippet_end).unwrap_or(",");
                        report.add_error(
                            file_path,
                            Some(line_no),
                            Some("Aliases"),
                            format!("Missing space after ',' in Aliases column: '{snippet}'"),
                            Some("Ensure every comma is followed by a single space (e.g. ', ')"),
                        );
                    } else {
                        let next_next_i = next_i.saturating_add(1);
                        if let Some(&(_, after_space_ch)) = chars.get(next_next_i) {
                            if after_space_ch.is_whitespace() {
                                let snippet_end = byte_idx.saturating_add(15).min(raw.len());
                                // Reason for fallback: context slice might not align on UTF-8 char boundary, so default to comma
                                let snippet = raw.get(byte_idx..snippet_end).unwrap_or(",");
                                report.add_error(
                                    file_path,
                                    Some(line_no),
                                    Some("Aliases"),
                                    format!(
                                        "Multiple spaces after ',' in Aliases column: '{snippet}'"
                                    ),
                                    Some("Use exactly one space after each comma"),
                                );
                            }
                        }
                    }
                }

                if let Some(item_raw) = raw.get(item_start..byte_idx) {
                    process_alias_item(
                        item_raw,
                        file_path,
                        line_no,
                        report,
                        &mut aliases,
                        &mut cross_references,
                        &mut decompositions,
                        &mut dc_syntax,
                    );
                } else {
                    report.add_error(
                        file_path,
                        Some(line_no),
                        Some("Aliases"),
                        format!(
                            "Invalid character boundary slice range {item_start}..{byte_idx} in Aliases column"
                        ),
                        Some("Ensure Aliases column contains valid UTF-8 characters"),
                    );
                }

                item_start = byte_idx.saturating_add(1);
            }
            _ => {}
        }
    }

    if let Some(last_raw) = raw.get(item_start..) {
        process_alias_item(
            last_raw,
            file_path,
            line_no,
            report,
            &mut aliases,
            &mut cross_references,
            &mut decompositions,
            &mut dc_syntax,
        );
    } else {
        report.add_error(
            file_path,
            Some(line_no),
            Some("Aliases"),
            format!(
                "Invalid trailing character boundary slice range {item_start}.. in Aliases column"
            ),
            Some("Ensure Aliases column contains valid UTF-8 characters"),
        );
    }

    (aliases, cross_references, decompositions, dc_syntax)
}

fn process_alias_item(
    item_raw: &str,
    file_path: &str,
    line_no: usize,
    report: &mut ValidationReport,
    aliases: &mut Vec<String>,
    cross_references: &mut Vec<String>,
    decompositions: &mut Vec<String>,
    dc_syntax: &mut Option<String>,
) {
    let item_trimmed = item_raw.trim();
    if item_trimmed.is_empty() {
        return;
    }

    if item_trimmed.starts_with(':') {
        if item_trimmed.contains("  ") {
            report.add_error(
                file_path,
                Some(line_no),
                Some("Aliases"),
                format!("Multiple consecutive spaces in alias syntax: '{item_trimmed}'"),
                Some("Use single spaces in syntax declarations"),
            );
        }
        *dc_syntax = Some(item_trimmed.to_string());
    } else if let Some(rest) = item_trimmed.strip_prefix('>') {
        if !rest.starts_with('<') && rest.starts_with(char::is_whitespace) {
            // Reason for fallback: missing token following whitespace defaults to empty string
            let token = rest.split_whitespace().next().unwrap_or("");
            report.add_error(
                file_path,
                Some(line_no),
                Some("Aliases"),
                format!("Extra space before '{token}' in cross-reference: '> {token}'"),
                Some(
                    "Cross-reference '>' must be followed immediately by target without spaces (e.g. '>{token}')",
                ),
            );
        }
        cross_references.push(item_trimmed.to_string());
    } else if item_trimmed.starts_with('<') {
        if let Some((tag_raw, payload)) = item_trimmed.split_once('>') {
            #[expect(
                clippy::expect_used,
                reason = "item_trimmed starts with '<', so split_once('>') prefix tag_raw is guaranteed to start with '<'"
            )]
            let tag_name = tag_raw
                .strip_prefix('<')
                .expect("tag_raw is guaranteed to start with '<'");
            let tag_full = format!("{tag_raw}>");

            if tag_name.starts_with(char::is_whitespace) {
                report.add_error(
                    file_path,
                    Some(line_no),
                    Some("Aliases"),
                    format!("Extra space after '<' in decomposition tag: '{tag_full}'"),
                    Some("Remove leading space inside '<...>' tag"),
                );
            }
            if tag_name.ends_with(char::is_whitespace) {
                report.add_error(
                    file_path,
                    Some(line_no),
                    Some("Aliases"),
                    format!("Extra space before '>' in decomposition tag: '{tag_full}'"),
                    Some("Remove trailing space inside '<...>' tag"),
                );
            }

            if payload.starts_with(char::is_whitespace) {
                // Reason for fallback: decomposition without non-empty payload tokens defaults to empty string
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
                    Some("Aliases"),
                    if first_token.is_empty() {
                        format!("Extra space after closing tag in decomposition: '{tag_full}'")
                    } else {
                        format!("Extra space before '{first_token}' in decomposition: '{full_sample}'")
                    },
                    Some(
                        "Decomposition tags must be followed immediately by target without spaces (e.g. '<tag>{token}')",
                    ),
                );
            }

            if payload.trim().contains("  ") {
                report.add_error(
                    file_path,
                    Some(line_no),
                    Some("Aliases"),
                    format!(
                        "Multiple spaces between tokens in decomposition payload: '{tag_full}{payload}'"
                    ),
                    Some("Use single spaces between decomposition tokens"),
                );
            }

            decompositions.push(item_trimmed.to_string());
        } else {
            report.add_error(
                file_path,
                Some(line_no),
                Some("Aliases"),
                format!("Unclosed decomposition tag in Aliases column: '{item_trimmed}'"),
                Some("Decomposition tags must be enclosed in '<...>'"),
            );
            decompositions.push(item_trimmed.to_string());
        }
    } else {
        if item_trimmed.contains("  ") {
            report.add_error(
                file_path,
                Some(line_no),
                Some("Aliases"),
                format!("Multiple consecutive spaces in alias: '{item_trimmed}'"),
                Some("Use single spaces between words in alias names"),
            );
        }
        aliases.push(item_trimmed.to_string());
    }
}

/// Splits the composite aliases/cross-reference/decomposition/syntax column.
pub fn split_dc_aliases_column(
    raw: &str,
) -> (Vec<String>, Vec<String>, Vec<String>, Option<String>) {
    let mut dummy_report = ValidationReport::default();
    parse_dc_aliases_column(raw, "", 0, &mut dummy_report)
}

/// Validates spacing and formatting conventions in the Aliases column.
pub fn validate_dc_aliases_spacing(
    raw: &str,
    file_path: &str,
    line_no: usize,
    report: &mut ValidationReport,
) {
    let _ = parse_dc_aliases_column(raw, file_path, line_no, report);
}

/// Parses and validates a single Dc category CSV file.
pub fn validate_dc_category_file(
    csv_bytes: &[u8],
    file_path: &str,
    report: &mut ValidationReport,
) -> Vec<DcDefn> {
    let vec_bytes = csv_bytes.to_vec();
    let table = match csv_tools::parse_csv_reader(
        &vec_bytes,
        csv_tools::CsvParseOptions {
            has_header: true,
            flexible: true,
            ..Default::default()
        },
    ) {
        Ok(t) => t,
        Err(e) => {
            report.add_error(
                file_path,
                None,
                None,
                format!("Failed to parse CSV: {e}"),
                Some("Verify CSV syntax and formatting"),
            );
            return Vec::new();
        }
    };

    let mut rows = Vec::new();

    if let Some(header) = table.header() {
        if header.len() != 10 {
            report.add_error(
                file_path,
                Some(1),
                None,
                format!(
                    "CSV header has {} columns, expected 10 columns",
                    header.len()
                ),
                Some("Ensure CSV header has exactly 10 columns matching schema.csv"),
            );
        }
    }

    let Some(category) = std::path::Path::new(file_path)
        .file_stem()
        .and_then(|s| s.to_str())
    else {
        report.add_error(
            file_path,
            None,
            None,
            format!("Invalid category file path: unable to determine category name from '{file_path}'"),
            Some("Ensure category files have a valid filename stem (e.g. 'latin.csv')"),
        );
        return Vec::new();
    };
    let category = category.to_string();

    for i in 0..table.row_count() {
        let line_no = i.saturating_add(2);
        let row_opt = table.row(i);
        let Some(row) = row_opt else {
            continue;
        };

        if row.len() != 10 {
            report.add_error(
                file_path,
                Some(line_no),
                None,
                format!("Row has {} columns, expected 10", row.len()),
                Some("Each row in Dc categories must have exactly 10 columns matching schema"),
            );
        }

        // Reason for fallback: out-of-bounds column indices on malformed rows default to empty string so schema validation can report all diagnostics without indexing panics
        let get_str = |idx: usize| -> String {
            row.get(idx).map_or(String::new(), |s| s.trim().to_string())
        };

        let dc_str = get_str(0);
        let short_str = get_str(1);
        let raw_name = get_str(2);
        let combining_str = get_str(3);
        let bidi_str = get_str(4);
        let casing_str = get_str(5);
        let general_cat_str = get_str(6);
        let script = get_str(7);
        let raw_aliases_untrimmed = get_str(8);
        let description = get_str(9);

        if dc_str.is_empty() && short_str.is_empty() && raw_name.is_empty() {
            continue;
        }

        let is_generated_csv = file_path.ends_with(".generated.csv");
        let (dc_id, is_unicode_char) = if let Some(hex_part) = dc_str.strip_prefix('u') {
            // Strictly u<hex> format: 1..=6 lowercase hex digits
            if (1..=6).contains(&hex_part.len())
                && hex_part.chars().all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
            {
                if let Ok(cp) = u32::from_str_radix(hex_part, 16) {
                    if cp <= 0x10_FFFF {
                        (u128::from(cp), true)
                    } else {
                        report.add_error(
                            file_path,
                            Some(line_no),
                            Some("Dc"),
                            format!("Unicode codepoint 'u{hex_part}' exceeds maximum Unicode 0x10FFFF"),
                            Some("Ensure codepoint is within 0x0..=0x10FFFF"),
                        );
                        continue;
                    }
                } else {
                    report.add_error(
                        file_path,
                        Some(line_no),
                        Some("Dc"),
                        format!("Invalid hex in Unicode reference: '{dc_str}'"),
                        Some("Must be 'u' followed by valid lowercase hex"),
                    );
                    continue;
                }
            } else {
                report.add_error(
                    file_path,
                    Some(line_no),
                    Some("Dc"),
                    format!("Invalid Unicode notation '{dc_str}': must be 'u' followed by 1..=6 lowercase hex digits (e.g. u0020, u0b)"),
                    Some("Only lowercase u<hex> is permitted for Unicode characters in category tables"),
                );
                continue;
            }
        } else if let Ok(v) = dc_str.parse::<u128>() {
            if v <= 1_114_111 {
                if is_generated_csv {
                    (v, true)
                } else {
                    report.add_error(
                        file_path,
                        Some(line_no),
                        Some("Dc"),
                        format!("Unicode character must use 'u<hex>' format in category tables, found decimal integer: '{dc_str}'"),
                        Some("Use 'u<hex>' (e.g. u0a, u85) for Unicode characters in category CSVs"),
                    );
                    continue;
                }
            } else {
                (v, false)
            }
        } else {
            report.add_error(
                file_path,
                Some(line_no),
                Some("Dc"),
                format!("Invalid Global Dc ID integer or Unicode reference: '{dc_str}'"),
                Some("Must be an integer within Document Character region or 'u<hex>' for Unicode characters"),
            );
            continue;
        };

        let short_id = if is_unicode_char {
            if short_str.is_empty() || short_str.starts_with("308") {
                None
            } else {
                report.add_error(
                    file_path,
                    Some(line_no),
                    Some("Short"),
                    format!("Invalid Short ID for Unicode character: '{short_str}'"),
                    Some("Short ID for Unicode characters must be blank or start with 308"),
                );
                None
            }
        } else {
            let parsed_short = if let Ok(v) = short_str.parse::<u32>() {
                v
            } else {
                report.add_error(
                    file_path,
                    Some(line_no),
                    Some("Short"),
                    format!("Invalid Short Dc ID integer: '{short_str}'"),
                    Some("Must be a non-negative integer"),
                );
                continue;
            };

            let expected_dc = SHORT_DC_REGION_START.saturating_add(u128::from(parsed_short));
            if dc_id != expected_dc {
                report.add_error(
                    file_path,
                    Some(line_no),
                    Some("Dc"),
                    format!(
                        "Global Dc ID ({dc_id}) does not match SHORT_DC_REGION_START + Short ID ({expected_dc})"
                    ),
                    Some("Ensure Dc ID is offset from Short ID by 1114112"),
                );
            }

            if !(SHORT_DC_REGION_START..=SHORT_DC_REGION_END).contains(&dc_id) {
                report.add_error(
                    file_path,
                    Some(line_no),
                    Some("Dc"),
                    format!(
                        "Dc ID {dc_id} is out of the Document Characters region bounds ({SHORT_DC_REGION_START}..={SHORT_DC_REGION_END})"
                    ),
                    Some("Verify region boundaries"),
                );
            }

            let Ok(short_id_usize) = usize::try_from(parsed_short) else {
                report.add_error(
                    file_path,
                    Some(line_no),
                    Some("Short"),
                    format!("Short ID {parsed_short} exceeds usize limits"),
                    Some("Ensure Short ID fits in machine usize"),
                );
                continue;
            };

            Some(short_id_usize)
        };

        let (name, is_deprecated) = if is_unicode_char {
            #[expect(
                clippy::expect_used,
                reason = "is_unicode_char guarantees dc_id was constructed from a u32 <= 0x10_FFFF"
            )]
            let cp = u32::try_from(dc_id).expect("dc_id fits in u32 for unicode chars");
            // Reason for fallback: characters without standard Unicode names use raw name from CSV category definition
            let uni_name = ctb_formats_unicode::get_unicode_name(cp)
                .unwrap_or_else(|| {
                    if let Some(stripped) = raw_name.strip_prefix('!') {
                        stripped.trim().to_string()
                    } else {
                        raw_name.trim().to_string()
                    }
                });
            let dep = ctb_formats_unicode::is_deprecated_unicode(cp);
            (uni_name, dep)
        } else {
            let dep = raw_name.starts_with('!');
            let n = if let Some(stripped) = raw_name.strip_prefix('!') {
                stripped.trim().to_string()
            } else {
                raw_name.trim().to_string()
            };
            (n, dep)
        };

        let combining_class = match validate_combining_class(&combining_str) {
            Ok(val) => val,
            Err(e) => {
                report.add_error(
                    file_path,
                    Some(line_no),
                    Some("◌"),
                    format!("Invalid combining class: {e}"),
                    Some("Must be integer 0..=254"),
                );
                0
            }
        };

        let bidi_class = match validate_bidi_class(&bidi_str) {
            Ok(b) => b,
            Err(e) => {
                report.add_error(
                    file_path,
                    Some(line_no),
                    Some("⇆"),
                    format!("Invalid Bidi Class: {e}"),
                    Some("Use standard Unicode Bidi class abbreviation (e.g. BN, L, ON)"),
                );
                BidiClass::BN
            }
        };

        let casing_partner = if casing_str.is_empty() {
            None
        } else if let Ok(v) = casing_str.parse::<u32>() {
            Some(v)
        } else {
            report.add_error(
                file_path,
                Some(line_no),
                Some("Aa"),
                format!("Invalid Casing partner ID integer: '{casing_str}'"),
                Some("Must be a short Dc ID integer if present"),
            );
            None
        };

        let general_category = match validate_general_category(&general_cat_str) {
            Ok(cat) => cat,
            Err(e) => {
                report.add_error(
                    file_path,
                    Some(line_no),
                    Some("Type"),
                    format!("Invalid General Category: {e}"),
                    Some("Use standard Unicode category or '!Cx'"),
                );
                GeneralCategory::NonUnicodeControl
            }
        };

        let (mut aliases, cross_references, decompositions, raw_dc_syntax) =
            parse_dc_aliases_column(
                &raw_aliases_untrimmed,
                file_path,
                line_no,
                report,
            );

        if is_unicode_char && !raw_name.is_empty() {
            let clean_raw = raw_name.trim_start_matches('!').trim();
            if !clean_raw.is_empty()
                && !clean_raw.eq_ignore_ascii_case(&name)
                && !aliases.iter().any(|a| a.eq_ignore_ascii_case(clean_raw))
            {
                aliases.insert(0, clean_raw.to_string());
            }
        }

        let dc_syntax = if let Some(raw_syn) = &raw_dc_syntax {
            match parse_dc_syntax(raw_syn) {
                Ok(rule) => Some(rule),
                Err(e) => {
                    report.add_error(
                        file_path,
                        Some(line_no),
                        Some("Aliases (syntax)"),
                        format!("Failed to parse Dc syntax DSL rule: {e}"),
                        Some("Verify Dc syntax DSL grammar"),
                    );
                    None
                }
            }
        } else {
            None
        };

        rows.push(DcDefn {
            dc_id,
            short_id,
            ident: None,
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
            cross_references,
            syntax: dc_syntax,
            description,
            format: None,
            source_file: file_path.to_string(),
            line_number: line_no,
        });
    }

    rows
}

/// Validates target references strictly (Dc short IDs, lowercase Unicode `uXXXX`, Formats `fXX`).
fn validate_target_token(
    token: &str,
    source_file: &str,
    line_no: usize,
    col_name: &str,
    known_dc_ids: &HashSet<u32>,
    deprecated_dc_ids: Option<&HashSet<u32>>,
    tag_name: Option<&str>,
    known_format_ids: &HashSet<usize>,
    report: &mut ValidationReport,
) {
    let clean = token
        .trim()
        .trim_matches(|c| c == '(' || c == ')' || c == '>' || c == '<');
    if clean.is_empty() {
        return;
    }

    let Some(target) = parse_target_token(clean) else {
        report.add_error(
            source_file,
            Some(line_no),
            Some(col_name),
            format!(
                "Invalid target reference token '{clean}': must be a Short Dc ID (e.g. 240), format ID (e.g. f80), or lowercase Unicode hex (e.g. u12ab)"
            ),
            Some("Use canonical format IDs (f80), lowercase Unicode hex (u0020), or Short Dc IDs (240)"),
        );
        return;
    };

    match target {
        CharTarget::Format(fmt_id) => {
            if !known_format_ids.contains(&fmt_id) {
                report.add_error(
                    source_file,
                    Some(line_no),
                    Some(col_name),
                    format!("Referenced Format ID 'f{fmt_id}' does not exist in formats registry"),
                    Some("Ensure referenced format ID is defined in formats category files"),
                );
            }
        }
        CharTarget::Unicode(cp) => {
            if !ctb_formats_unicode::is_assigned_unicode(cp) {
                report.add_error(
                    source_file,
                    Some(line_no),
                    Some(col_name),
                    format!(
                        "Referenced Unicode codepoint 'u{cp:04x}' (U+{cp:04X}) is not an assigned Unicode character"
                    ),
                    Some("Ensure referenced Unicode character exists in Unicode standard"),
                );
            } else if let Some(tag) = tag_name {
                if ctb_formats_unicode::is_deprecated_unicode(cp) {
                    report.add_error(
                        source_file,
                        Some(line_no),
                        Some(col_name),
                        format!(
                            "Canonical equivalency or approximate relationship '<{tag}>' references deprecated Unicode character 'u{cp:04x}' (U+{cp:04X})"
                        ),
                        Some("Canonical equivalencies and approximations must target active, non-deprecated Unicode characters"),
                    );
                }
            }
        }
        CharTarget::Dc(target_dc) => {
            if !known_dc_ids.contains(&target_dc) {
                report.add_error(
                    source_file,
                    Some(line_no),
                    Some(col_name),
                    format!("Referenced Dc ID '{target_dc}' does not exist in Document Characters registry"),
                    Some("Ensure referenced Dc ID is defined in a Dc category file"),
                );
            } else if let (Some(dep_ids), Some(tag)) = (deprecated_dc_ids, tag_name) {
                if dep_ids.contains(&target_dc) {
                    report.add_error(
                        source_file,
                        Some(line_no),
                        Some(col_name),
                        format!(
                            "Canonical equivalency or approximate relationship '<{tag}>' references deprecated Dc ID '{target_dc}'"
                        ),
                        Some("Canonical equivalencies and approximations must target active, non-deprecated Document Characters"),
                    );
                }
            }
        }
    }
}

/// Validates a sequence of Dc category files.
pub fn validate_dc_files_data<'a, I>(
    files: I,
    category_dir_label: &str,
    known_format_ids: &HashSet<usize>,
    report: &mut ValidationReport,
) -> Vec<DcDefn>
where
    I: IntoIterator<Item = (&'a str, &'a [u8])>,
{
    let mut all_rows = Vec::new();
    let mut short_id_map: HashMap<usize, (String, usize)> = HashMap::new();
    let mut dc_id_map: HashMap<u128, (String, usize)> = HashMap::new();

    for (path_str, contents) in files {
        if !path_str.ends_with(".csv")
            || path_str.ends_with("schema.csv")
            || path_str.ends_with(".generated.csv")
        {
            continue;
        }

        let rows = validate_dc_category_file(contents, path_str, report);

        for row in rows {
            if let Some(short_id) = row.short_id {
                if let Some((prev_file, prev_line)) =
                    short_id_map.get(&short_id)
                {
                    report.add_error(
                        &row.source_file,
                        Some(row.line_number),
                        Some("Short"),
                        format!(
                            "Duplicate Short Dc ID {short_id} already defined in {prev_file}:{prev_line}",
                        ),
                        Some("Assign a unique Short ID to each Document Character"),
                    );
                } else {
                    short_id_map.insert(
                        short_id,
                        (row.source_file.clone(), row.line_number),
                    );
                }
            }

            if let Some((prev_file, prev_line)) = dc_id_map.get(&row.dc_id) {
                report.add_error(
                    &row.source_file,
                    Some(row.line_number),
                    Some("Dc"),
                    format!(
                        "Duplicate Dc ID {} already defined in {prev_file}:{prev_line}",
                        row.dc_id
                    ),
                    Some("Ensure each character has a unique Dc ID"),
                );
            } else {
                dc_id_map.insert(row.dc_id, (row.source_file.clone(), row.line_number));
            }

            all_rows.push(row);
        }
    }

    let known_dc_ids: HashSet<u32> = all_rows
        .iter()
        .filter_map(|r| r.short_id)
        .filter_map(|id| u32::try_from(id).ok())
        .collect();

    let deprecated_dc_ids: HashSet<u32> = all_rows
        .iter()
        .filter(|r| r.is_deprecated)
        .filter_map(|r| r.short_id)
        .filter_map(|id| u32::try_from(id).ok())
        .collect();

    // Validate that Short Dc IDs form a contiguous sequence starting from 0 with no gaps/holes
    if let Some(&max_id) = known_dc_ids.iter().max() {
        let mut missing_ids = Vec::new();
        for id in 0..=max_id {
            if !known_dc_ids.contains(&id) {
                missing_ids.push(id);
            }
        }
        if !missing_ids.is_empty() {
            let missing_str = if missing_ids.len() <= 10 {
                missing_ids
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            } else {
                format!(
                    "{} (and {} more)",
                    missing_ids
                        .iter()
                        .take(10)
                        .map(std::string::ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", "),
                    missing_ids.len().saturating_sub(10)
                )
            };
            report.add_error(
                category_dir_label,
                None,
                Some("Short"),
                format!(
                    "Document Character Short IDs have gaps/holes. Missing {} ID(s): [{missing_str}] in range 0..={max_id}",
                    missing_ids.len()
                ),
                Some("Ensure Short Dc IDs are contiguous with no missing numbers"),
            );
        }
    }

    // Second pass: Validate cross-references, decompositions, and syntax rules
    for row in &all_rows {
        for xref in &row.cross_references {
            let target = if let Some(stripped) = xref.strip_prefix('>') {
                stripped
            } else {
                xref
            };
            validate_target_token(
                target,
                &row.source_file,
                row.line_number,
                "Aliases (cross-reference)",
                &known_dc_ids,
                None,
                None,
                known_format_ids,
                report,
            );
        }

        for decomp in &row.decompositions {
            let Some((tag_raw, payload)) = decomp.split_once('>') else {
                continue;
            };
            let tag_name = tag_raw.trim_start_matches('<').trim();
            let enforce_deprecation = tag_name == "equiv" || tag_name == "approx";
            let tag_opt = if enforce_deprecation {
                Some(tag_name)
            } else {
                None
            };
            let dep_opt = if enforce_deprecation {
                Some(&deprecated_dc_ids)
            } else {
                None
            };

            for token in payload.split_whitespace() {
                validate_target_token(
                    token,
                    &row.source_file,
                    row.line_number,
                    "Aliases (decomposition)",
                    &known_dc_ids,
                    dep_opt,
                    tag_opt,
                    known_format_ids,
                    report,
                );
            }
        }

        if let Some(syntax_rule) = &row.syntax {
            let Some(short_id) = row.short_id else {
                report.add_error(
                    &row.source_file,
                    Some(row.line_number),
                    Some("Aliases"),
                    "Document Character syntax rule defined on an entry without a valid Short ID",
                    Some("Assign a Short ID to characters that define syntax rules"),
                );
                continue;
            };

            let Ok(short_id_u32) = u32::try_from(short_id) else {
                report.add_error(
                    &row.source_file,
                    Some(row.line_number),
                    Some("Short"),
                    format!("Short ID {short_id} exceeds u32 limits for syntax rule validation"),
                    Some("Ensure Short ID fits in u32"),
                );
                continue;
            };

            validate_dc_syntax(
                syntax_rule,
                short_id_u32,
                &known_dc_ids,
                known_format_ids,
                report,
                &row.source_file,
                row.line_number,
            );
        }
    }

    all_rows
}

/// Discovers and validates all Dc category files from an embedded directory.
pub fn validate_all_dc_files(
    dc_dir: &Dir,
    known_format_ids: &HashSet<usize>,
    report: &mut ValidationReport,
) -> Vec<DcDefn> {
    let mut files = Vec::new();
    for f in dc_dir.files() {
        if let Some(path_str) = f.path().to_str() {
            files.push((path_str, f.contents()));
        }
    }
    validate_dc_files_data(
        files,
        "src/formats/dcdata/data/categories/",
        known_format_ids,
        report,
    )
}

/// Discovers and validates all Dc category files from an on-disk directory.
pub fn validate_all_dc_files_from_disk(
    dc_dir: &std::path::Path,
    known_format_ids: &HashSet<usize>,
    report: &mut ValidationReport,
) -> Vec<DcDefn> {
    let Ok(entries) = std::fs::read_dir(dc_dir) else {
        report.add_error(
            &dc_dir.display().to_string(),
            None,
            None,
            format!(
                "Could not read Dc categories directory at {}",
                dc_dir.display()
            ),
            None,
        );
        return Vec::new();
    };

    let mut paths = Vec::new();
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_file() {
            let Some(file_name) = p.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if file_name.ends_with(".csv")
                && file_name != "schema.csv"
                && !file_name.ends_with(".generated.csv")
            {
                paths.push(p);
            }
        }
    }
    paths.sort();

    let mut file_data = Vec::new();
    for p in &paths {
        let Some(file_name) = p.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if let Ok(bytes) = std::fs::read(p) {
            file_data.push((file_name.to_string(), bytes));
        } else {
            report.add_error(
                &p.display().to_string(),
                None,
                None,
                format!("Failed to read file {}", p.display()),
                None,
            );
        }
    }

    let files_iter: Vec<(&str, &[u8])> = file_data
        .iter()
        .map(|(name, bytes)| (name.as_str(), bytes.as_slice()))
        .collect();

    validate_dc_files_data(
        files_iter,
        &dc_dir.display().to_string(),
        known_format_ids,
        report,
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
    use crate::format::validate_formats_category_file;
    use crate::shared::{validate_extension_entry, validate_rust_identifier};
    use crate::syntax::{
        ActionArg, MatchOutcome, Quantifier, SyntaxPattern, SyntaxTerm,
        match_syntax_rule,
    };
    use crate::updater::{
        assign_and_update_dc_categories, assign_and_update_format_categories,
        generate_merged_csvs, read_csv_file, write_csv_file,
    };

    #[crate::ctb_test]
    fn test_validate_all_data_tables_repository() {
        let report = validate_all_data_tables();
        assert!(
            !report.has_errors(),
            "Data table validation failed:\n{}",
            report.format_report()
        );
    }

    #[crate::ctb_test]
    fn test_validate_rust_identifier() {
        validate_rust_identifier("Utf8").unwrap();
        validate_rust_identifier("_Valid123").unwrap();
        validate_rust_identifier("MyFormat").unwrap();

        assert!(validate_rust_identifier("").is_err());
        assert!(validate_rust_identifier("123abc").is_err());
        assert!(validate_rust_identifier("bad-name").is_err());
        assert!(validate_rust_identifier("type").is_err());
        assert!(validate_rust_identifier("match").is_err());
    }

    #[crate::ctb_test]
    fn test_validate_extension_rules() {
        // Plain extensions must have leading dots
        validate_extension_entry(".txt").unwrap();
        validate_extension_entry(".tar.gz").unwrap();
        assert!(validate_extension_entry("txt").is_err());

        // Regex patterns with ~...~
        validate_extension_entry(r"~^\._~").unwrap();
        validate_extension_entry(r"~/\.AppleDouble/~").unwrap();
        assert!(validate_extension_entry(r"~[invalid regex(+~").is_err());
    }

    #[crate::ctb_test]
    fn test_split_dc_aliases_column() {
        let raw = "xon, resume transmission, >32, <equiv>240 239";
        let (aliases, xrefs, decomps, syntax) = split_dc_aliases_column(raw);

        assert_eq!(aliases, vec!["xon", "resume transmission"]);
        assert_eq!(xrefs, vec![">32"]);
        assert_eq!(decomps, vec!["<equiv>240 239"]);
        assert!(syntax.is_none());

        let raw_syntax = ":~ [^248 255]+ 248";
        let (aliases_s, xrefs_s, decomps_s, syntax_s) =
            split_dc_aliases_column(raw_syntax);
        assert!(aliases_s.is_empty());
        assert!(xrefs_s.is_empty());
        assert!(decomps_s.is_empty());
        assert_eq!(syntax_s.as_deref(), Some(":~ [^248 255]+ 248"));
    }

    #[crate::ctb_test]
    fn test_cross_table_uniqueness() {
        let mut report = ValidationReport::new();
        let dc_names = vec![(10, "UniqueDc", "test/dc.csv")];
        let fmt_labels = vec![(20, "UniqueDc", "test/fmt.csv")];

        validate_cross_table_uniqueness(&dc_names, &fmt_labels, &mut report);
        assert!(report.has_errors());
        assert!(report.format_report().contains("collides with Dc name"));
    }

    #[crate::ctb_test]
    fn test_dc_gap_detection() {
        let mut report = ValidationReport::new();
        let _known_formats: HashSet<usize> = HashSet::new();

        // Create a simulated in-memory category with gap: Short IDs 0 and 2 (missing 1)
        let csv_data = b"Dc,Short,Name (!=deprecated),comb,bidi,Aa,Type,Script,Aliases,Description,\n1114112,0,Null,0,BN,,Cc,,,, \n1114114,2,Two,0,BN,,Po,,,, \n";
        let rows = validate_dc_category_file(
            csv_data,
            "test/categories/test.csv",
            &mut report,
        );
        assert_eq!(rows.len(), 2);

        // Run gap validation logic
        let known_dc_ids: HashSet<usize> =
            rows.iter().filter_map(|r| r.short_id).collect();
        if let Some(&max_id) = known_dc_ids.iter().max() {
            let mut missing_ids = Vec::new();
            for id in 0..=max_id {
                if !known_dc_ids.contains(&id) {
                    missing_ids.push(id);
                }
            }
            if !missing_ids.is_empty() {
                report.add_error(
                    "test/categories/",
                    None,
                    Some("Short"),
                    format!("Missing IDs: {missing_ids:?}"),
                    None,
                );
            }
        }

        assert!(report.has_errors());
        assert!(report.format_report().contains("Missing IDs: [1]"));
    }

    #[crate::ctb_test]
    fn test_end_to_end_assign_and_merge_cycle() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo = temp_dir.path();

        let dc_cats = repo.join("src/formats/dcdata/data/categories");
        let fmt_cats = repo.join("src/formats/dcdata/data/categories/formats");
        std::fs::create_dir_all(&dc_cats).unwrap();
        std::fs::create_dir_all(&fmt_cats).unwrap();

        // Write schema.csv files
        let dc_schema_header = vec![
            "Dc".to_string(),
            "Short".to_string(),
            "Name (!=deprecated)".to_string(),
            "◌".to_string(),
            "⇆".to_string(),
            "Aa".to_string(),
            "Type".to_string(),
            "Script".to_string(),
            "Aliases; >=xref, <=decompos., :=Dc syntax".to_string(),
            "Description".to_string(),
        ];
        write_csv_file(
            &repo.join("src/formats/dcdata/data/schema.csv"),
            &dc_schema_header,
            &[],
        )
        .unwrap();

        let fmt_schema_header = vec![
            "Dc".to_string(),
            "Short".to_string(),
            "Ident (Rust-friendly)".to_string(),
            "Label".to_string(),
            "Category".to_string(),
            "BaseFormat".to_string(),
            "Extensions".to_string(),
            "MIME".to_string(),
            "UTI".to_string(),
            "Apple Type".to_string(),
            "Nicknames".to_string(),
            "Import".to_string(),
            "Export".to_string(),
            "Tests".to_string(),
            "Variants".to_string(),
            "Comments".to_string(),
            "References".to_string(),
        ];
        write_csv_file(
            &repo.join("src/formats/dcdata/data/categories/formats/schema.csv"),
            &fmt_schema_header,
            &[],
        )
        .unwrap();

        // Write category files with AUTO IDs
        let dc_rows = vec![
            vec![
                "1114112".to_string(),
                "0".to_string(),
                "Null".to_string(),
                "0".to_string(),
                "BN".to_string(),
                String::new(),
                "Cc".to_string(),
                String::new(),
                String::new(),
                String::new(),
            ],
            vec![
                String::new(),
                "AUTO".to_string(),
                "CustomDc".to_string(),
                "0".to_string(),
                "BN".to_string(),
                String::new(),
                "Po".to_string(),
                String::new(),
                String::new(),
                String::new(),
            ],
        ];
        write_csv_file(
            &dc_cats.join("custom.csv"),
            &dc_schema_header,
            &dc_rows,
        )
        .unwrap();

        let fmt_rows = vec![
            vec![
                "2228224".to_string(),
                "0".to_string(),
                "FormatZero".to_string(),
                "Format Zero".to_string(),
                "document".to_string(),
                String::new(),
                ".fz".to_string(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                "3".to_string(),
                "3".to_string(),
                "1".to_string(),
                String::new(),
                String::new(),
                String::new(),
            ],
            vec![
                String::new(),
                "AUTO".to_string(),
                "FormatOne".to_string(),
                "Format One".to_string(),
                "document".to_string(),
                String::new(),
                ".fo".to_string(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                "3".to_string(),
                "3".to_string(),
                "1".to_string(),
                String::new(),
                String::new(),
                String::new(),
            ],
        ];
        write_csv_file(
            &fmt_cats.join("doc.csv"),
            &fmt_schema_header,
            &fmt_rows,
        )
        .unwrap();

        // Run auto assignment
        let dc_stats = assign_and_update_dc_categories(repo).unwrap();
        assert_eq!(dc_stats.new_ids_assigned, 1);
        assert_eq!(dc_stats.max_short_id, 1);

        let fmt_stats = assign_and_update_format_categories(repo).unwrap();
        assert_eq!(fmt_stats.new_ids_assigned, 1);
        assert_eq!(fmt_stats.max_short_id, 1);

        // Run merge generation
        let gen_stats = generate_merged_csvs(repo).unwrap();
        assert_eq!(gen_stats.dc_records_merged, 2);
        assert_eq!(gen_stats.format_records_merged, 2);

        // Verify generated DcList.generated.csv contents and order
        let (dc_gen_hdr, dc_gen_rows) = read_csv_file(
            &repo.join("src/formats/dcdata/data/DcList.generated.csv"),
        )
        .unwrap();
        assert_eq!(dc_gen_hdr, dc_schema_header);
        assert_eq!(dc_gen_rows.len(), 2);
        assert_eq!(dc_gen_rows[0][0], "1114112");
        assert_eq!(dc_gen_rows[0][1], "0");
        assert_eq!(dc_gen_rows[1][0], "1114113");
        assert_eq!(dc_gen_rows[1][1], "1");

        // Verify generated formats.generated.csv contents and order
        let (fmt_gen_hdr, fmt_gen_rows) = read_csv_file(
            &repo.join("src/formats/dcdata/data/formats.generated.csv"),
        )
        .unwrap();
        assert_eq!(fmt_gen_hdr, fmt_schema_header);
        assert_eq!(fmt_gen_rows.len(), 2);
        assert_eq!(fmt_gen_rows[0][0], "2228224");
        assert_eq!(fmt_gen_rows[0][1], "0");
        assert_eq!(fmt_gen_rows[1][0], "2228225");
        assert_eq!(fmt_gen_rows[1][1], "1");

        // Verify generated JSON files
        let dc_json_path = repo.join("src/formats/dcdata/data/DcList.generated.json");
        assert!(dc_json_path.exists());
        let dc_json_str = std::fs::read_to_string(&dc_json_path).unwrap();
        assert!(dc_json_str.contains("CustomDc"));

        let fmt_json_path = repo.join("src/formats/dcdata/data/formats.generated.json");
        assert!(fmt_json_path.exists());
        let fmt_json_str = std::fs::read_to_string(&fmt_json_path).unwrap();
        assert!(fmt_json_str.contains("FormatOne"));
        assert!(fmt_json_str.contains("format_document"));
        assert!(fmt_json_str.contains("\"BN\""));
        assert!(fmt_json_str.contains("\"!Cx\""));
        assert!(fmt_json_str.contains("\"Formats\""));
    }

    #[crate::ctb_test]
    fn test_strict_target_token_rules() {
        // Valid Short Dc IDs
        assert_eq!(parse_target_token("246"), Some(CharTarget::Dc(246)));
        assert_eq!(parse_target_token("0"), Some(CharTarget::Dc(0)));

        // Valid Format IDs
        assert_eq!(parse_target_token("f80"), Some(CharTarget::Format(80)));
        assert_eq!(parse_target_token("f0"), Some(CharTarget::Format(0)));

        // Valid Unicode codepoints (lowercase hex only)
        assert_eq!(parse_target_token("u0020"), Some(CharTarget::Unicode(0x20)));
        assert_eq!(parse_target_token("u12ab"), Some(CharTarget::Unicode(0x12ab)));
        assert_eq!(parse_target_token("u0"), Some(CharTarget::Unicode(0)));

        // Rejected format variants
        assert_eq!(parse_target_token("format 80"), None);
        assert_eq!(parse_target_token("format80"), None);
        assert_eq!(parse_target_token("F80"), None);
        assert_eq!(parse_target_token("fmt80"), None);

        // Rejected Unicode variants (uppercase, U+, u+)
        assert_eq!(parse_target_token("U+12AB"), None);
        assert_eq!(parse_target_token("U+12ab"), None);
        assert_eq!(parse_target_token("u12AB"), None);
        assert_eq!(parse_target_token("u+0020"), None);
        assert_eq!(parse_target_token("U0020"), None);
        assert_eq!(parse_target_token("u1234567"), None); // Too long
    }

    #[crate::ctb_test]
    fn test_dc_syntax_dsl_parser() {
        // Comment rule: :~ [^248 255]+ 248
        let rule1 = parse_dc_syntax(":~ [^248 255]+ 248").unwrap();
        assert!(rule1.action.is_none());
        if let SyntaxPattern::Sequence(elements) = &rule1.pattern {
            assert_eq!(elements.len(), 3);
            assert_eq!(elements[0].term, SyntaxTerm::SelfChar);
            assert_eq!(
                elements[1].term,
                SyntaxTerm::CharSet {
                    negated: true,
                    members: vec![CharTarget::Dc(248), CharTarget::Dc(255)],
                }
            );
            assert_eq!(elements[1].quantifier, Quantifier::OneOrMore);
            assert_eq!(elements[2].term, SyntaxTerm::CharRef(CharTarget::Dc(248)));
        } else {
            panic!("Expected sequence pattern");
        }

        // Rule with optional macro expansion: :~ [260:] 259
        let rule2 = parse_dc_syntax(":~ [260:] 259").unwrap();
        if let SyntaxPattern::Sequence(elements) = &rule2.pattern {
            assert_eq!(elements.len(), 3);
            assert_eq!(
                elements[1].term,
                SyntaxTerm::RuleRef {
                    target: CharTarget::Dc(260)
                }
            );
            assert_eq!(elements[1].quantifier, Quantifier::Optional);
        } else {
            panic!("Expected sequence pattern");
        }

        // Rule with named constructs and action invocation:
        // :[identifier $ident] ~ [value $val] : lang.assign($ident, $val)
        let rule3 = parse_dc_syntax(
            ":[identifier $ident] ~ [value $val] : lang.assign($ident, $val)",
        )
        .unwrap();
        assert!(rule3.action.is_some());
        let action = rule3.action.unwrap();
        assert_eq!(action.method, "lang.assign");
        assert_eq!(
            action.args,
            vec![
                ActionArg::Variable("ident".to_string()),
                ActionArg::Variable("val".to_string()),
            ]
        );

        // Grouped alternation: :([statement] ~) | (~ ~)
        let rule4 = parse_dc_syntax(":([statement] ~) | (~ ~)").unwrap();
        if let SyntaxPattern::Alternation(branches) = &rule4.pattern {
            assert_eq!(branches.len(), 2);
        } else {
            panic!("Expected alternation pattern");
        }

        // Parameterized named construct with nested pattern: :[list:([filename] | 513)]
        let rule5 = parse_dc_syntax(":[list:([filename] | 513)]").unwrap();
        if let SyntaxPattern::Sequence(elements) = &rule5.pattern {
            assert_eq!(elements.len(), 1);
            assert_eq!(
                elements[0].term,
                SyntaxTerm::NamedConstruct {
                    name: "list".to_string(),
                    subtype: Some("([filename] | 513)".to_string()),
                    capture_var: None,
                }
            );
        } else {
            panic!("Expected sequence pattern");
        }

        // Parameterized named construct with capture variable: :[list:([filename] | 513) $path]
        let rule6 = parse_dc_syntax(":[list:([filename] | 513) $path]").unwrap();
        if let SyntaxPattern::Sequence(elements) = &rule6.pattern {
            assert_eq!(elements.len(), 1);
            assert_eq!(
                elements[0].term,
                SyntaxTerm::NamedConstruct {
                    name: "list".to_string(),
                    subtype: Some("([filename] | 513)".to_string()),
                    capture_var: Some("path".to_string()),
                }
            );
        } else {
            panic!("Expected sequence pattern");
        }

        // Parenthesized alternation in negated character set: :[^(314 | 312)]
        let rule7 = parse_dc_syntax(":[^(314 | 312)]").unwrap();
        if let SyntaxPattern::Sequence(elements) = &rule7.pattern {
            assert_eq!(elements.len(), 1);
            assert_eq!(
                elements[0].term,
                SyntaxTerm::CharSet {
                    negated: true,
                    members: vec![CharTarget::Dc(314), CharTarget::Dc(312)],
                }
            );
        } else {
            panic!("Expected sequence pattern");
        }

        // Bare pipe in negated character set must be an error
        let err_rule = parse_dc_syntax(":[^314 | 312]");
        assert!(err_rule.is_err());
        assert!(err_rule.unwrap_err().to_string().contains("enclosed in parentheses"));

        // Parenthesized alternation in positive character set: :[(246 | 247)]
        let rule8 = parse_dc_syntax(":[(246 | 247)]").unwrap();
        if let SyntaxPattern::Sequence(elements) = &rule8.pattern {
            assert_eq!(elements.len(), 1);
            assert_eq!(
                elements[0].term,
                SyntaxTerm::CharSet {
                    negated: false,
                    members: vec![CharTarget::Dc(246), CharTarget::Dc(247)],
                }
            );
        } else {
            panic!("Expected sequence pattern");
        }
    }

    #[crate::ctb_test]
    fn test_dc_syntax_validator_checks() {
        let mut report = ValidationReport::new();
        let known_dcs: HashSet<u32> = [246, 248, 255, 260].into_iter().collect();
        let known_fmts: HashSet<usize> = [80].into_iter().collect();

        // Valid rule
        let valid_rule = parse_dc_syntax(":~ [^248 255]+ 248").unwrap();
        validate_dc_syntax(
            &valid_rule,
            246,
            &known_dcs,
            &known_fmts,
            &mut report,
            "test/syntax.csv",
            10,
        );
        assert!(!report.has_errors());

        // Unknown Dc reference: 9999
        let invalid_rule = parse_dc_syntax(":~ 9999").unwrap();
        validate_dc_syntax(
            &invalid_rule,
            246,
            &known_dcs,
            &known_fmts,
            &mut report,
            "test/syntax.csv",
            11,
        );
        assert!(report.has_errors());
        assert!(report.format_report().contains("Referenced Dc ID '9999'"));

        // Unbound variable in action
        let mut report2 = ValidationReport::new();
        let unbound_action_rule =
            parse_dc_syntax(":[identifier $ident] ~ : lang.assign($ident, $unbound)").unwrap();
        validate_dc_syntax(
            &unbound_action_rule,
            269,
            &known_dcs,
            &known_fmts,
            &mut report2,
            "test/syntax.csv",
            12,
        );
        assert!(report2.has_errors());
        assert!(report2.format_report().contains("Variable '$unbound'"));
    }

    #[crate::ctb_test]
    fn test_dc_syntax_matcher_resilient_tag_soup() {
        // Test matching single line comment: :~ [^248 255]+ 248
        let rule = parse_dc_syntax(":~ [^248 255]+ 248").unwrap();

        // Exact match with Dc 246 (self), content [65, 66], closing Dc 248
        let stream = vec![246, 65, 66, 248];
        let (outcome, ctx) = match_syntax_rule(&stream, &rule, Some(246));
        assert_eq!(outcome, MatchOutcome::Matched { consumed: 4 });
        assert!(ctx.warnings.is_empty());

        // Resilient tag-soup recovery: Unclosed comment at EOF [246, 65, 66] (missing 248)
        let unclosed_stream = vec![246, 65, 66];
        let (outcome_rec, ctx_rec) =
            match_syntax_rule(&unclosed_stream, &rule, Some(246));
        assert!(outcome_rec.is_matched());
        assert_eq!(outcome_rec.consumed_tokens(), 3);
        assert!(!ctx_rec.warnings.is_empty());
        assert!(ctx_rec.warnings[0].contains("Unclosed syntax structure"));

        // Variable capture test
        let assign_rule =
            parse_dc_syntax(":[identifier $ident] ~ [value $val]").unwrap();
        let assign_stream = vec![100, 269, 200];
        let (outcome_assign, ctx_assign) =
            match_syntax_rule(&assign_stream, &assign_rule, Some(269));
        assert_eq!(outcome_assign, MatchOutcome::Matched { consumed: 3 });
        assert_eq!(ctx_assign.captured_vars.get("ident"), Some(&vec![100]));
        assert_eq!(ctx_assign.captured_vars.get("val"), Some(&vec![200]));
    }

    #[crate::ctb_test]
    fn test_column_count_mismatch_validation() {
        let mut report = ValidationReport::default();

        // 1. Dc category with row having 9 columns instead of 10
        let invalid_dc_csv = b"Dc,Short,Name (!=deprecated),\xe2\x97\x8c,\xe2\x87\x86,Aa,Type,Script,Aliases,Description\n1114112,0,Null,0,BN,,Cc,Controls,\n";
        validate_dc_category_file(invalid_dc_csv, "invalid_dc.csv", &mut report);
        assert!(report.has_errors());
        assert!(
            report
                .format_report()
                .contains("Row has 9 columns, expected 10")
        );

        // 2. Format category with row having 16 columns instead of 17
        let mut fmt_report = ValidationReport::default();
        let invalid_fmt_csv = b"Dc,Short,Ident (Rust-friendly),Label,Category,BaseFormat,Extensions,MIME,UTI,Apple Type,Nicknames,Import,Export,Tests,Variants,Comments,References\n2228224,0,FmtZero,Format Zero,document,,.fz,,,,3,3,1,,,\n";
        let valid_variants = HashSet::new();
        validate_formats_category_file(
            invalid_fmt_csv,
            "invalid_fmt.csv",
            &valid_variants,
            &mut fmt_report,
        );
        assert!(fmt_report.has_errors());
        assert!(
            fmt_report
                .format_report()
                .contains("Row has 16 columns, expected 17")
        );

        // 3. Layout table with row having 4 columns instead of 5
        let mut layout_report = ValidationReport::default();
        let invalid_layout_csv = b"Partition Name,First ID,Last ID,Count,Description\n0,1114111,1114112,Unicode\n";
        validate_layout_table(
            invalid_layout_csv,
            "invalid_layout.csv",
            &mut layout_report,
        );
        assert!(layout_report.has_errors());
        assert!(
            layout_report
                .format_report()
                .contains("Row has 4 columns, expected 5")
        );
    }

    #[crate::ctb_test]
    fn test_equiv_and_approx_deprecated_target_validation() {
        let known_formats: HashSet<usize> = HashSet::new();

        // 1. <equiv> referencing deprecated Dc (Short ID 0 is !Null)
        let mut report_equiv_dc = ValidationReport::default();
        let csv_equiv_dc = b"Dc,Short,Name (!=deprecated),\xe2\x97\x8c,\xe2\x87\x86,Aa,Type,Script,Aliases,Description\n1114112,0,!Null,0,BN,,Cc,Controls,,\n1114113,1,One,0,BN,,Po,Controls,<equiv>0,\n";
        validate_dc_files_data(
            [("test.csv", &csv_equiv_dc[..])],
            "test",
            &known_formats,
            &mut report_equiv_dc,
        );
        assert!(report_equiv_dc.has_errors());
        assert!(
            report_equiv_dc
                .format_report()
                .contains("references deprecated Dc ID '0'")
        );

        // 2. <approx> referencing deprecated Dc (Short ID 0 is !Null)
        let mut report_approx_dc = ValidationReport::default();
        let csv_approx_dc = b"Dc,Short,Name (!=deprecated),\xe2\x97\x8c,\xe2\x87\x86,Aa,Type,Script,Aliases,Description\n1114112,0,!Null,0,BN,,Cc,Controls,,\n1114113,1,One,0,BN,,Po,Controls,<approx>0,\n";
        validate_dc_files_data(
            [("test.csv", &csv_approx_dc[..])],
            "test",
            &known_formats,
            &mut report_approx_dc,
        );
        assert!(report_approx_dc.has_errors());
        assert!(
            report_approx_dc
                .format_report()
                .contains("references deprecated Dc ID '0'")
        );

        // 3. <equiv> referencing deprecated Unicode codepoint (u17a3)
        let mut report_equiv_uni = ValidationReport::default();
        let csv_equiv_uni = b"Dc,Short,Name (!=deprecated),\xe2\x97\x8c,\xe2\x87\x86,Aa,Type,Script,Aliases,Description\n1114112,0,Zero,0,BN,,Cc,Controls,,\n1114113,1,One,0,BN,,Po,Controls,<equiv>u17a3,\n";
        validate_dc_files_data(
            [("test.csv", &csv_equiv_uni[..])],
            "test",
            &known_formats,
            &mut report_equiv_uni,
        );
        assert!(report_equiv_uni.has_errors());
        assert!(
            report_equiv_uni
                .format_report()
                .contains("references deprecated Unicode character 'u17a3' (U+17A3)")
        );

        // 4. <approx> referencing deprecated Unicode codepoint (u0149)
        let mut report_approx_uni = ValidationReport::default();
        let csv_approx_uni = b"Dc,Short,Name (!=deprecated),\xe2\x97\x8c,\xe2\x87\x86,Aa,Type,Script,Aliases,Description\n1114112,0,Zero,0,BN,,Cc,Controls,,\n1114113,1,One,0,BN,,Po,Controls,<approx>u0149,\n";
        validate_dc_files_data(
            [("test.csv", &csv_approx_uni[..])],
            "test",
            &known_formats,
            &mut report_approx_uni,
        );
        assert!(report_approx_uni.has_errors());
        assert!(
            report_approx_uni
                .format_report()
                .contains("references deprecated Unicode character 'u0149' (U+0149)")
        );

        // 5. Cross references and <ambiguous> may reference deprecated Dcs without error
        let mut report_allowed = ValidationReport::default();
        let csv_allowed = b"Dc,Short,Name (!=deprecated),\xe2\x97\x8c,\xe2\x87\x86,Aa,Type,Script,Aliases,Description\n1114112,0,!Null,0,BN,,Cc,Controls,,\n1114113,1,One,0,BN,,Po,Controls,\">0, <ambiguous>0\",\n";
        validate_dc_files_data(
            [("test.csv", &csv_allowed[..])],
            "test",
            &known_formats,
            &mut report_allowed,
        );
        assert!(!report_allowed.has_errors());

        // 6. Valid non-deprecated <equiv> and <approx> targets pass
        let mut report_valid = ValidationReport::default();
        let csv_valid = b"Dc,Short,Name (!=deprecated),\xe2\x97\x8c,\xe2\x87\x86,Aa,Type,Script,Aliases,Description\n1114112,0,Zero,0,BN,,Cc,Controls,,\n1114113,1,One,0,BN,,Po,Controls,\"<equiv>u0020, <approx>0\",\n";
        validate_dc_files_data(
            [("test.csv", &csv_valid[..])],
            "test",
            &known_formats,
            &mut report_valid,
        );
        assert!(!report_valid.has_errors());
    }

    #[crate::ctb_test]
    fn test_dc_aliases_spacing_validation() {
        // 1. Contrived example with two spacing errors:
        //    "<approx> 0,<equiv>u15"
        //    - extra space before 0
        //    - missing space after ,
        let mut report_contrived = ValidationReport::default();
        validate_dc_aliases_spacing(
            "<approx> 0,<equiv>u15",
            "test/categories.csv",
            2,
            &mut report_contrived,
        );
        assert!(report_contrived.has_errors());
        assert_eq!(report_contrived.error_count(), 2);
        let report_text = report_contrived.format_report();
        assert!(
            report_text.contains("Extra space before '0' in decomposition")
        );
        assert!(
            report_text.contains("Missing space after ',' in Aliases column")
        );

        // 2. Extra space before comma: "<approx>0 , <equiv>u15"
        let mut report_space_before_comma = ValidationReport::default();
        validate_dc_aliases_spacing(
            "<approx>0 , <equiv>u15",
            "test/categories.csv",
            2,
            &mut report_space_before_comma,
        );
        assert!(report_space_before_comma.has_errors());
        assert!(
            report_space_before_comma
                .format_report()
                .contains("Extra space before ','")
        );

        // 3. Multiple spaces after comma: "<approx>0,  <equiv>u15"
        let mut report_multiple_spaces = ValidationReport::default();
        validate_dc_aliases_spacing(
            "<approx>0,  <equiv>u15",
            "test/categories.csv",
            2,
            &mut report_multiple_spaces,
        );
        assert!(report_multiple_spaces.has_errors());
        assert!(
            report_multiple_spaces
                .format_report()
                .contains("Multiple spaces after ','")
        );

        // 4. Extra space after '>' in cross-reference: "> 32"
        let mut report_xref_space = ValidationReport::default();
        validate_dc_aliases_spacing(
            "> 32",
            "test/categories.csv",
            2,
            &mut report_xref_space,
        );
        assert!(report_xref_space.has_errors());
        assert!(
            report_xref_space
                .format_report()
                .contains("Extra space before '32' in cross-reference")
        );

        // 5. Extra space inside tag name: "< approx>0"
        let mut report_tag_space = ValidationReport::default();
        validate_dc_aliases_spacing(
            "< approx>0",
            "test/categories.csv",
            2,
            &mut report_tag_space,
        );
        assert!(report_tag_space.has_errors());
        assert!(
            report_tag_space
                .format_report()
                .contains("Extra space after '<'")
        );

        // 6. Cell leading and trailing whitespace
        let mut report_cell_ws = ValidationReport::default();
        validate_dc_aliases_spacing(
            " <approx>0",
            "test/categories.csv",
            2,
            &mut report_cell_ws,
        );
        assert!(report_cell_ws.has_errors());
        assert!(
            report_cell_ws
                .format_report()
                .contains("Cell has leading whitespace")
        );

        // 7. Multiple spaces in alias name: "word  separator"
        let mut report_alias_spaces = ValidationReport::default();
        validate_dc_aliases_spacing(
            "word  separator",
            "test/categories.csv",
            2,
            &mut report_alias_spaces,
        );
        assert!(report_alias_spaces.has_errors());
        assert!(
            report_alias_spaces
                .format_report()
                .contains("Multiple consecutive spaces in alias")
        );

        // 8. Correctly formatted aliases pass without any error
        let mut report_ok = ValidationReport::default();
        validate_dc_aliases_spacing(
            "<approx>0, <equiv>u15",
            "test/categories.csv",
            2,
            &mut report_ok,
        );
        validate_dc_aliases_spacing(
            "word separator, <equiv>u20, >32, <approx>u2d",
            "test/categories.csv",
            2,
            &mut report_ok,
        );
        validate_dc_aliases_spacing(
            "line feed, end of line, <equiv>240 239",
            "test/categories.csv",
            2,
            &mut report_ok,
        );
        validate_dc_aliases_spacing(
            ":~ [number]",
            "test/categories.csv",
            2,
            &mut report_ok,
        );
        assert!(!report_ok.has_errors());
    }
}

