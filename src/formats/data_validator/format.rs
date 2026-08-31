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

//! Schema validator for Formats registry category files (`src/formats/utilities/data/formats/*.csv`).

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::collections::{HashMap, HashSet};
use include_dir::Dir;
use crate::report::ValidationReport;
use crate::shared::{
    validate_extensions_field, validate_mime_field, validate_rust_identifier,
    validate_support_level,
};

pub const FORMAT_REGION_START: u128 = 2_228_224;
pub const FORMAT_REGION_END: u128 = 3_342_335;

/// Validated format record from a category CSV file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedFormatRow {
    pub dc_id: u128,
    pub short_id: usize,
    pub ident: String,
    pub label: String,
    pub category: String,
    pub mime: String,
    pub extensions: String,
    pub uti: String,
    pub apple_type: String,
    pub nicknames: String,
    pub base_format: String,
    pub import_support: String,
    pub export_support: String,
    pub tests: String,
    pub variant_types: String,
    pub comments: String,
    pub references: String,
    pub source_file: String,
    pub line_number: usize,
}

/// Parses and validates a single format category CSV file.
pub fn validate_formats_category_file(
    csv_bytes: &[u8],
    file_path: &str,
    valid_variant_names: &HashSet<String>,
    report: &mut ValidationReport,
) -> Vec<ParsedFormatRow> {
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
                format!("Failed to parse CSV file: {e}"),
                Some("Verify CSV syntax and RFC4180 quotes"),
            );
            return Vec::new();
        }
    };

    let mut rows = Vec::new();

    for i in 0..table.row_count() {
        let line_no = i.saturating_add(2);
        let get_str = |col: usize| -> String {
            match table.cell(i, col) {
                Some(s) => s.trim().to_string(),
                None => String::new(),
            }
        };

        let dc_str = get_str(0);
        let short_str = get_str(1);
        let ident = get_str(2);
        let label = get_str(3);
        let category = get_str(4);
        let mime = get_str(5);
        let extensions = get_str(6);
        let uti = get_str(7);
        let apple_type = get_str(8);
        let nicknames = get_str(9);
        let base_format = get_str(10);
        let import_support = get_str(11);
        let export_support = get_str(12);
        let tests = get_str(13);
        let variant_types = get_str(14);
        let comments = get_str(15);
        let references = get_str(16);

        if dc_str.is_empty() && short_str.is_empty() && label.is_empty() {
            continue;
        }

        let short_id = match short_str.parse::<usize>() {
            Ok(v) => v,
            Err(_) => {
                report.add_error(
                    file_path,
                    Some(line_no),
                    Some("Short"),
                    format!("Invalid Short format ID integer: '{short_str}'"),
                    Some("Must be a non-negative integer"),
                );
                continue;
            }
        };

        let dc_id = match dc_str.parse::<u128>() {
            Ok(v) => v,
            Err(_) => {
                report.add_error(
                    file_path,
                    Some(line_no),
                    Some("Dc"),
                    format!("Invalid Global Dc ID integer: '{dc_str}'"),
                    Some("Must be an integer within format region"),
                );
                continue;
            }
        };

        let Ok(short_id_u128) = u128::try_from(short_id) else {
            report.add_error(
                file_path,
                Some(line_no),
                Some("Short"),
                format!("Short ID {short_id} exceeds u128 range"),
                Some("Short IDs must fit within numeric limits"),
            );
            continue;
        };

        let expected_dc = FORMAT_REGION_START.saturating_add(short_id_u128);
        if dc_id != expected_dc {
            report.add_error(
                file_path,
                Some(line_no),
                Some("Dc"),
                format!(
                    "Global Dc ID ({dc_id}) does not match FORMAT_REGION_START + Short ID ({expected_dc})"
                ),
                Some("Ensure Dc ID is offset from Short ID by 2228224"),
            );
        }

        if dc_id < FORMAT_REGION_START || dc_id > FORMAT_REGION_END {
            report.add_error(
                file_path,
                Some(line_no),
                Some("Dc"),
                format!(
                    "Dc ID {dc_id} is out of the Formats region bounds ({FORMAT_REGION_START}..={FORMAT_REGION_END})"
                ),
                Some("Verify region boundaries"),
            );
        }

        let effective_label = if !label.is_empty() {
            label.clone()
        } else if !ident.is_empty() {
            ident.clone()
        } else {
            String::new()
        };

        if effective_label.is_empty() {
            report.add_error(
                file_path,
                Some(line_no),
                Some("Label"),
                "Format must have either a Label or a valid Ident",
                Some("Provide a human-readable display label or Ident for the format"),
            );
        }

        if !ident.is_empty() {
            if let Err(e) = validate_rust_identifier(&ident) {
                report.add_error(
                    file_path,
                    Some(line_no),
                    Some("Ident"),
                    format!("Invalid Rust identifier '{ident}': {e}"),
                    Some("Identifiers must match [a-zA-Z_][a-zA-Z0-9_]* and not clash with keywords"),
                );
            }
        }

        if category.is_empty() {
            report.add_error(
                file_path,
                Some(line_no),
                Some("Category"),
                "Category column cannot be empty",
                Some("Specify the format category (e.g. document, encoding, compression)"),
            );
        }

        if let Err(e) = validate_mime_field(&mime) {
            report.add_error(
                file_path,
                Some(line_no),
                Some("MIME"),
                format!("Invalid MIME field '{mime}': {e}"),
                Some("Format as 'type/subtype' (e.g. 'text/plain')"),
            );
        }

        if let Err(e) = validate_extensions_field(&extensions) {
            report.add_error(
                file_path,
                Some(line_no),
                Some("Extensions"),
                format!("Invalid Extensions field '{extensions}': {e}"),
                Some("Extensions must start with '.' or regexes enclosed in '~...~'"),
            );
        }

        if !apple_type.is_empty() && apple_type.chars().count() != 4 {
            report.add_warning(
                file_path,
                Some(line_no),
                Some("Apple Type code"),
                format!("Apple Type code '{apple_type}' is not 4 characters"),
                Some("Classic Mac OSTypes are usually 4 bytes/characters"),
            );
        }

        if let Err(e) = validate_support_level(&import_support) {
            report.add_error(
                file_path,
                Some(line_no),
                Some("Import support"),
                format!("Invalid Import support value: {e}"),
                Some("Use -1..=5 or leave blank/0"),
            );
        }

        if let Err(e) = validate_support_level(&export_support) {
            report.add_error(
                file_path,
                Some(line_no),
                Some("Export support"),
                format!("Invalid Export support value: {e}"),
                Some("Use -1..=5 or leave blank/0"),
            );
        }

        if let Err(e) = validate_support_level(&tests) {
            report.add_error(
                file_path,
                Some(line_no),
                Some("Tests"),
                format!("Invalid Tests support value: {e}"),
                Some("Use -1..=5 or leave blank/0"),
            );
        }

        if !variant_types.is_empty() {
            for v in variant_types.split(',') {
                let v_trimmed = v.trim();
                if v_trimmed.is_empty() {
                    continue;
                }
                if !valid_variant_names.contains(v_trimmed) {
                    report.add_error(
                        file_path,
                        Some(line_no),
                        Some("Variant Types"),
                        format!(
                            "Variant Type '{v_trimmed}' is not among registered 'v.*' category files"
                        ),
                        Some("Ensure Variant Types reference existing 'v.*' category file stems (e.g. lineEndings, unicodePua)"),
                    );
                }
            }
        }

        rows.push(ParsedFormatRow {
            dc_id,
            short_id,
            ident,
            label,
            category,
            mime,
            extensions,
            uti,
            apple_type,
            nicknames,
            base_format,
            import_support,
            export_support,
            tests,
            variant_types,
            comments,
            references,
            source_file: file_path.to_string(),
            line_number: line_no,
        });
    }

    rows
}

/// Validates a sequence of format category files.
pub fn validate_format_files_data<'a, I>(
    files: I,
    category_dir_label: &str,
    report: &mut ValidationReport,
) -> Vec<ParsedFormatRow>
where
    I: IntoIterator<Item = (&'a str, &'a [u8])> + Clone,
{
    let mut variant_categories = HashSet::new();

    // First pass: discover all category file names (including "v.*" variant category files and standard categories)
    for (path_str, _) in files.clone() {
        if path_str.ends_with(".csv") {
            let path = std::path::Path::new(path_str);
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            variant_categories.insert(stem.to_string());
            if let Some(variant_name) = stem.strip_prefix("v.") {
                variant_categories.insert(variant_name.to_string());
            }
        }
    }

    let mut all_rows = Vec::new();
    let mut short_id_map: HashMap<usize, (String, usize)> = HashMap::new();
    let mut ident_map: HashMap<String, (String, usize)> = HashMap::new();

    for (path_str, contents) in files {
        if !path_str.ends_with(".csv") || path_str.ends_with("schema.csv") || path_str.ends_with(".generated.csv") {
            continue;
        }

        let rows = validate_formats_category_file(
            contents,
            path_str,
            &variant_categories,
            report,
        );

        for row in rows {
            if let Some((prev_file, prev_line)) = short_id_map.get(&row.short_id) {
                report.add_error(
                    &row.source_file,
                    Some(row.line_number),
                    Some("Short"),
                    format!(
                        "Duplicate Short Format ID {} already defined in {prev_file}:{prev_line}",
                        row.short_id
                    ),
                    Some("Assign a unique Short ID to each format"),
                );
            } else {
                short_id_map.insert(
                    row.short_id,
                    (row.source_file.clone(), row.line_number),
                );
            }

            if !row.ident.is_empty() {
                if let Some((prev_file, prev_line)) = ident_map.get(&row.ident) {
                    report.add_error(
                        &row.source_file,
                        Some(row.line_number),
                        Some("Ident"),
                        format!(
                            "Duplicate Format Ident '{}' already defined in {prev_file}:{prev_line}",
                            row.ident
                        ),
                        Some("Assign a unique Rust-friendly Ident to each format"),
                    );
                } else {
                    ident_map.insert(
                        row.ident.clone(),
                        (row.source_file.clone(), row.line_number),
                    );
                }
            }

            all_rows.push(row);
        }
    }

    // Validate that Short Format IDs form a contiguous sequence starting from 0 with no gaps/holes
    let known_fmt_ids: HashSet<usize> = all_rows.iter().map(|r| r.short_id).collect();
    if let Some(&max_id) = known_fmt_ids.iter().max() {
        let mut missing_ids = Vec::new();
        for id in 0..=max_id {
            if !known_fmt_ids.contains(&id) {
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
                    "Format Short IDs have gaps/holes. Missing {} ID(s): [{missing_str}] in range 0..={max_id}",
                    missing_ids.len()
                ),
                Some("Ensure Format Short IDs are contiguous with no missing numbers"),
            );
        }
    }

    all_rows
}

/// Discovers all format category files from an embedded directory and validates uniqueness across files.
pub fn validate_all_format_files(
    formats_dir: &Dir,
    report: &mut ValidationReport,
) -> Vec<ParsedFormatRow> {
    let mut files = Vec::new();
    for f in formats_dir.files() {
        if let Some(path_str) = f.path().to_str() {
            files.push((path_str, f.contents()));
        }
    }
    validate_format_files_data(files, "src/formats/utilities/data/formats/", report)
}

/// Discovers all format category files from an on-disk directory and validates uniqueness across files.
pub fn validate_all_format_files_from_disk(
    formats_dir: &std::path::Path,
    report: &mut ValidationReport,
) -> Vec<ParsedFormatRow> {
    let Ok(entries) = std::fs::read_dir(formats_dir) else {
        report.add_error(
            &formats_dir.display().to_string(),
            None,
            None,
            format!("Could not read Formats directory at {}", formats_dir.display()),
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

    validate_format_files_data(
        files_iter,
        &formats_dir.display().to_string(),
        report,
    )
}
