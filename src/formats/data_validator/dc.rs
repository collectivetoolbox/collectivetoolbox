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

//! Schema validator and facet splitter for Document Character category tables (`src/formats/dctext/data/categories/*.csv`).

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::report::ValidationReport;
use crate::shared::{
    validate_bidi_class, validate_combining_class, validate_general_category,
};
use crate::syntax::{
    CharTarget, DcSyntaxRule, parse_dc_syntax, parse_target_token,
    validate_dc_syntax,
};
use include_dir::Dir;
use std::collections::{HashMap, HashSet};

pub const DC_REGION_START: u128 = 1_114_112;
pub const DC_REGION_END: u128 = 2_228_223;

/// Validated and split Document Character record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedDcRow {
    pub dc_id: u128,
    pub short_id: u32,
    pub name: String,
    pub is_deprecated: bool,
    pub combining_class: u8,
    pub bidi_class: Option<String>,
    pub casing_partner: Option<u32>,
    pub general_category: String,
    pub script: String,
    pub aliases: Vec<String>,
    pub cross_references: Vec<String>,
    pub decompositions: Vec<String>,
    pub dc_syntax: Option<DcSyntaxRule>,
    pub description: String,
    pub source_file: String,
    pub line_number: usize,
}

/// Splits the composite aliases/cross-reference/decomposition/syntax column.
pub fn split_dc_aliases_column(
    raw: &str,
) -> (Vec<String>, Vec<String>, Vec<String>, Option<String>) {
    let mut aliases = Vec::new();
    let mut cross_references = Vec::new();
    let mut decompositions = Vec::new();
    let mut dc_syntax: Option<String> = None;

    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return (aliases, cross_references, decompositions, dc_syntax);
    }

    // If the entire cell or an entry begins with a syntax declaration `:`, isolate it.
    if trimmed.starts_with(':') {
        dc_syntax = Some(trimmed.to_string());
        return (aliases, cross_references, decompositions, dc_syntax);
    }

    // Parse comma-separated items outside of syntax definitions
    for item in trimmed.split(',') {
        let item_trimmed = item.trim();
        if item_trimmed.is_empty() {
            continue;
        }

        if item_trimmed.starts_with(':') {
            dc_syntax = Some(item_trimmed.to_string());
        } else if item_trimmed.starts_with('>') {
            cross_references.push(item_trimmed.to_string());
        } else if item_trimmed.starts_with('<') && item_trimmed.contains('>') {
            decompositions.push(item_trimmed.to_string());
        } else {
            aliases.push(item_trimmed.to_string());
        }
    }

    (aliases, cross_references, decompositions, dc_syntax)
}

/// Parses and validates a single Dc category CSV file.
pub fn validate_dc_category_file(
    csv_bytes: &[u8],
    file_path: &str,
    report: &mut ValidationReport,
) -> Vec<ParsedDcRow> {
    let vec_bytes = csv_bytes.to_vec();
    let table = match csv_tools::parse_csv_reader(
        &vec_bytes,
        csv_tools::CsvParseOptions {
            has_header: true,
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

    for i in 0..table.row_count() {
        let line_no = i.saturating_add(2);

        if let Some(row_slice) = table.row(i) {
            if row_slice.iter().all(|s| s.trim().is_empty()) {
                continue;
            }
            if row_slice.len() != 10 {
                report.add_error(
                    file_path,
                    Some(line_no),
                    None,
                    format!(
                        "Row has {} columns, expected 10 columns (mismatched field count)",
                        row_slice.len()
                    ),
                    Some("Check for unquoted commas, missing commas, or extra columns to avoid misaligned data"),
                );
            }
        }

        let get_str = |col: usize| -> String {
            match table.cell(i, col) {
                Some(s) => s.trim().to_string(),
                None => String::new(),
            }
        };

        let dc_str = get_str(0);
        let short_str = get_str(1);
        let raw_name = get_str(2);
        let combining_str = get_str(3);
        let bidi_str = get_str(4);
        let casing_str = get_str(5);
        let general_cat_str = get_str(6);
        let script = get_str(7);
        let raw_aliases = get_str(8);
        let description = get_str(9);

        if dc_str.is_empty() && short_str.is_empty() && raw_name.is_empty() {
            continue;
        }

        let short_id = if let Ok(v) = short_str.parse::<u32>() {
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

        let dc_id = if let Ok(v) = dc_str.parse::<u128>() {
            v
        } else {
            report.add_error(
                file_path,
                Some(line_no),
                Some("Dc"),
                format!("Invalid Global Dc ID integer: '{dc_str}'"),
                Some("Must be an integer within Document Character region"),
            );
            continue;
        };

        let expected_dc = DC_REGION_START.saturating_add(u128::from(short_id));
        if dc_id != expected_dc {
            report.add_error(
                file_path,
                Some(line_no),
                Some("Dc"),
                format!(
                    "Global Dc ID ({dc_id}) does not match DC_REGION_START + Short ID ({expected_dc})"
                ),
                Some("Ensure Dc ID is offset from Short ID by 1114112"),
            );
        }

        if !(DC_REGION_START..=DC_REGION_END).contains(&dc_id) {
            report.add_error(
                file_path,
                Some(line_no),
                Some("Dc"),
                format!(
                    "Dc ID {dc_id} is out of the Document Characters region bounds ({DC_REGION_START}..={DC_REGION_END})"
                ),
                Some("Verify region boundaries"),
            );
        }

        if raw_name.is_empty() {
            report.add_error(
                file_path,
                Some(line_no),
                Some("Name"),
                "Dc Name cannot be empty",
                Some("Provide a character name"),
            );
        }

        let is_deprecated = raw_name.starts_with('!');
        let name = if let Some(stripped) = raw_name.strip_prefix('!') {
            stripped.trim().to_string()
        } else {
            raw_name.trim().to_string()
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

        if let Err(e) = validate_bidi_class(&bidi_str) {
            report.add_error(
                file_path,
                Some(line_no),
                Some("⇆"),
                format!("Invalid Bidi Class: {e}"),
                Some("Use standard Unicode Bidi class abbreviation or leave empty"),
            );
        }
        let bidi_class = if bidi_str.is_empty() {
            None
        } else {
            Some(bidi_str)
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

        if let Err(e) = validate_general_category(&general_cat_str) {
            report.add_error(
                file_path,
                Some(line_no),
                Some("Type"),
                format!("Invalid General Category: {e}"),
                Some("Use standard Unicode category or '!Cx'"),
            );
        }

        let (aliases, cross_references, decompositions, raw_dc_syntax) =
            split_dc_aliases_column(&raw_aliases);

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

        rows.push(ParsedDcRow {
            dc_id,
            short_id,
            name,
            is_deprecated,
            combining_class,
            bidi_class,
            casing_partner,
            general_category: general_cat_str,
            script,
            aliases,
            cross_references,
            decompositions,
            dc_syntax,
            description,
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
) -> Vec<ParsedDcRow>
where
    I: IntoIterator<Item = (&'a str, &'a [u8])>,
{
    let mut all_rows = Vec::new();
    let mut short_id_map: HashMap<u32, (String, usize)> = HashMap::new();

    for (path_str, contents) in files {
        if !path_str.ends_with(".csv")
            || path_str.ends_with("schema.csv")
            || path_str.ends_with(".generated.csv")
        {
            continue;
        }

        let rows = validate_dc_category_file(contents, path_str, report);

        for row in rows {
            if let Some((prev_file, prev_line)) =
                short_id_map.get(&row.short_id)
            {
                report.add_error(
                    &row.source_file,
                    Some(row.line_number),
                    Some("Short"),
                    format!(
                        "Duplicate Short Dc ID {} already defined in {prev_file}:{prev_line}",
                        row.short_id
                    ),
                    Some("Assign a unique Short ID to each Document Character"),
                );
            } else {
                short_id_map.insert(
                    row.short_id,
                    (row.source_file.clone(), row.line_number),
                );
            }

            all_rows.push(row);
        }
    }

    let known_dc_ids: HashSet<u32> =
        all_rows.iter().map(|r| r.short_id).collect();

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
                known_format_ids,
                report,
            );
        }

        for decomp in &row.decompositions {
            let Some((_tag, payload)) = decomp.split_once('>') else {
                continue;
            };
            for token in payload.split_whitespace() {
                validate_target_token(
                    token,
                    &row.source_file,
                    row.line_number,
                    "Aliases (decomposition)",
                    &known_dc_ids,
                    known_format_ids,
                    report,
                );
            }
        }

        if let Some(syntax_rule) = &row.dc_syntax {
            validate_dc_syntax(
                syntax_rule,
                row.short_id,
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
) -> Vec<ParsedDcRow> {
    let mut files = Vec::new();
    for f in dc_dir.files() {
        if let Some(path_str) = f.path().to_str() {
            files.push((path_str, f.contents()));
        }
    }
    validate_dc_files_data(
        files,
        "src/formats/dctext/data/categories/",
        known_format_ids,
        report,
    )
}

/// Discovers and validates all Dc category files from an on-disk directory.
pub fn validate_all_dc_files_from_disk(
    dc_dir: &std::path::Path,
    known_format_ids: &HashSet<usize>,
    report: &mut ValidationReport,
) -> Vec<ParsedDcRow> {
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
