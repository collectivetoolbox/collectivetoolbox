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
use crate::dc_def::DcDefn;
use crate::shared::{
    BidiClass, GeneralCategory, split_comma_separated_items,
    validate_bidi_class, validate_combining_class, validate_general_category,
};
use crate::syntax::{
    CharTarget, parse_dc_syntax, parse_target_token, validate_dc_syntax,
};
use include_dir::Dir;
use std::collections::{HashMap, HashSet};

pub const DC_REGION_START: u128 = 1_114_112;
pub const DC_REGION_END: u128 = 2_228_223;

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

    let items = split_comma_separated_items(trimmed);
    for item in items {
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

    // Reason for fallback: file paths without stems default to "general" category
    let category = std::path::Path::new(file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("general")
        .to_string();

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

        // Reason for fallback: missing columns produce empty string so schema validation can report column count error
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

        let Ok(short_id_usize) = usize::try_from(short_id) else {
            report.add_error(
                file_path,
                Some(line_no),
                Some("Short"),
                format!("Short ID {short_id} exceeds usize limits"),
                Some("Ensure Short ID fits in machine usize"),
            );
            continue;
        };

        rows.push(DcDefn {
            dc_id,
            short_id: short_id_usize,
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
) -> Vec<DcDefn>
where
    I: IntoIterator<Item = (&'a str, &'a [u8])>,
{
    let mut all_rows = Vec::new();
    let mut short_id_map: HashMap<usize, (String, usize)> = HashMap::new();

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

    // Reason for fallback: short ID values are within u32 range, default to 0 if out of range for syntax checks
    let known_dc_ids: HashSet<u32> = all_rows
        .iter()
        .map(|r| u32::try_from(r.short_id).unwrap_or(0))
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

        if let Some(syntax_rule) = &row.syntax {
            // Reason for fallback: short ID values fit u32, default to 0 if conversion fails
            let short_id_u32 = u32::try_from(row.short_id).unwrap_or(0);
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
