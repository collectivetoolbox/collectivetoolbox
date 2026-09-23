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
use ctb_storage_minimal::shorthand::parse_unicode_shorthand;

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

    // 2. Validate Scripts Registry table
    let known_scripts = if let Some(scripts_bytes) =
        crate::get_dc_data_file("README.scripts.csv")
    {
        validate_scripts_table(
            &scripts_bytes,
            "data/README.scripts.csv",
            &mut report,
        )
    } else {
        report.add_error(
            "data/README.scripts.csv",
            None,
            None,
            "Could not locate embedded README.scripts.csv",
            None,
        );
        HashSet::new()
    };

    // 3. Validate Formats category files
    let format_rows =
        validate_all_format_files(&FORMATS_CATEGORIES_DIR, Some(&known_scripts), &mut report);
    let known_format_ids: HashSet<usize> =
        format_rows.iter().filter_map(|r| r.short_id).collect();
    let known_format_with_syntax: HashSet<usize> =
        format_rows.iter().filter(|r| r.syntax.is_some()).filter_map(|r| r.short_id).collect();

    // 4. Validate Decompositions table
    let known_decomp_tags = if let Some(decomp_bytes) =
        crate::get_dc_data_file("README-decompositions.csv")
    {
        validate_decompositions_table(
            &decomp_bytes,
            "data/README-decompositions.csv",
            &mut report,
        )
    } else {
        report.add_error(
            "data/README-decompositions.csv",
            None,
            None,
            "Could not locate embedded README-decompositions.csv",
            None,
        );
        HashSet::new()
    };

    let named_types_bytes_opt = crate::get_dc_data_file("README-named-types.csv");
    // Reason for fallback: optional named types file defaults to empty set if not present
    let known_named_types = named_types_bytes_opt
        .as_ref()
        .map(|b| extract_named_type_names(b))
        .unwrap_or_default();

    // 5. Validate Document Characters category files
    let dc_rows = if let Some(dc_dir) = get_dc_categories_dir() {
        validate_all_dc_files(
            dc_dir,
            &known_format_ids,
            Some(&known_decomp_tags),
            Some(&known_named_types),
            Some(&known_scripts),
            Some(&known_format_with_syntax),
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

    // 5. Validate Cross-Table Name / Label Uniqueness
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

    // 6. Validate Named Types table
    let known_dc_ids: HashSet<u32> = dc_rows
        .iter()
        .filter_map(|r| r.short_id)
        .filter_map(|id| u32::try_from(id).ok())
        .collect();

    let mut known_syntax_targets: HashSet<CharTarget> = dc_rows
        .iter()
        .filter(|r| r.syntax.is_some())
        .filter_map(|r| r.short_id)
        .filter_map(|id| u32::try_from(id).ok())
        .map(CharTarget::Dc)
        .collect();
    for &fid in &known_format_with_syntax {
        known_syntax_targets.insert(CharTarget::Format(fid));
    }

    if let Some(named_types_bytes) = named_types_bytes_opt {
        validate_named_types_table(
            &named_types_bytes,
            "data/README-named-types.csv",
            &known_dc_ids,
            &known_format_ids,
            Some(&known_scripts),
            Some(&known_syntax_targets),
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

    // 2. Validate Scripts Registry table
    let scripts_path = repo_root.join("src/formats/dcdata/data/README.scripts.csv");
    let known_scripts = if let Ok(bytes) = std::fs::read(&scripts_path) {
        validate_scripts_table(
            &bytes,
            "src/formats/dcdata/data/README.scripts.csv",
            &mut report,
        )
    } else {
        report.add_error(
            "src/formats/dcdata/data/README.scripts.csv",
            None,
            None,
            "Could not locate README.scripts.csv on disk",
            Some("Ensure file exists in src/formats/dcdata/data/"),
        );
        HashSet::new()
    };

    // 3. Validate Formats category files
    let formats_dir = repo_root.join("src/formats/dcdata/data/categories/formats");
    let format_rows =
        validate_all_format_files_from_disk(&formats_dir, Some(&known_scripts), &mut report);
    let known_format_ids: HashSet<usize> =
        format_rows.iter().filter_map(|r| r.short_id).collect();
    let known_format_with_syntax: HashSet<usize> =
        format_rows.iter().filter(|r| r.syntax.is_some()).filter_map(|r| r.short_id).collect();

    // 4. Validate Decompositions table
    let decomp_path =
        repo_root.join("src/formats/dcdata/data/README-decompositions.csv");
    let known_decomp_tags = if let Ok(bytes) = std::fs::read(&decomp_path) {
        validate_decompositions_table(
            &bytes,
            "src/formats/dcdata/data/README-decompositions.csv",
            &mut report,
        )
    } else {
        report.add_error(
            "src/formats/dcdata/data/README-decompositions.csv",
            None,
            None,
            "Could not locate README-decompositions.csv on disk",
            Some("Ensure file exists in src/formats/dcdata/data/"),
        );
        HashSet::new()
    };

    let named_types_path =
        repo_root.join("src/formats/dcdata/data/README-named-types.csv");
    let named_types_bytes_opt = std::fs::read(&named_types_path).ok();
    // Reason for fallback: optional named types file defaults to empty set if not present on disk
    let known_named_types = named_types_bytes_opt
        .as_ref()
        .map(|b| extract_named_type_names(b))
        .unwrap_or_default();

    // 5. Validate Document Characters category files
    let dc_dir = repo_root.join("src/formats/dcdata/data/categories");
    let dc_rows = validate_all_dc_files_from_disk(
        &dc_dir,
        &known_format_ids,
        Some(&known_decomp_tags),
        Some(&known_named_types),
        Some(&known_scripts),
        Some(&known_format_with_syntax),
        &mut report,
    );

    // 6. Validate Cross-Table Name / Label Uniqueness
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

    // 7. Validate Named Types table
    let known_dc_ids: HashSet<u32> = dc_rows
        .iter()
        .filter_map(|r| r.short_id)
        .filter_map(|id| u32::try_from(id).ok())
        .collect();

    let mut known_syntax_targets: HashSet<CharTarget> = dc_rows
        .iter()
        .filter(|r| r.syntax.is_some())
        .filter_map(|r| r.short_id)
        .filter_map(|id| u32::try_from(id).ok())
        .map(CharTarget::Dc)
        .collect();
    for &fid in &known_format_with_syntax {
        known_syntax_targets.insert(CharTarget::Format(fid));
    }

    if let Some(bytes) = named_types_bytes_opt {
        validate_named_types_table(
            &bytes,
            "src/formats/dcdata/data/README-named-types.csv",
            &known_dc_ids,
            &known_format_ids,
            Some(&known_scripts),
            Some(&known_syntax_targets),
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

/// Validates the README.scripts.csv registry table, verifying schema and uniqueness,
/// and returning the set of valid script identifiers.
pub fn validate_scripts_table(
    csv_bytes: &[u8],
    file_path: &str,
    report: &mut ValidationReport,
) -> HashSet<String> {
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
            return HashSet::new();
        }
    };

    let mut valid_scripts = HashSet::new();
    for i in 0..table.row_count() {
        let line_no = i.saturating_add(2);
        let Some(row) = table.row(i) else {
            continue;
        };

        if row.len() < 3 {
            report.add_error(
                file_path,
                Some(line_no),
                None,
                format!("Row has {} columns, expected at least 3", row.len()),
                Some("Ensure row has 'Script', 'Category', and 'Description' columns"),
            );
            continue;
        }

        let script = row.get(0).map_or("", |s| s.trim());
        let category = row.get(1).map_or("", |s| s.trim());
        let desc = row.get(2).map_or("", |s| s.trim());

        if script.is_empty() {
            report.add_error(
                file_path,
                Some(line_no),
                Some("Script"),
                "Script identifier cannot be empty".to_string(),
                Some("Specify a script name"),
            );
        } else if !valid_scripts.insert(script.to_string()) {
            report.add_error(
                file_path,
                Some(line_no),
                Some("Script"),
                format!("Duplicate script '{script}'"),
                Some("Ensure each script is defined uniquely"),
            );
        }

        if category.is_empty() {
            report.add_error(
                file_path,
                Some(line_no),
                Some("Category"),
                "Script category cannot be empty".to_string(),
                Some("Specify 'Dc', 'Formats', or 'Unicode'"),
            );
        }

        if desc.is_empty() {
            report.add_error(
                file_path,
                Some(line_no),
                Some("Description"),
                "Script description cannot be empty".to_string(),
                Some("Provide a description explaining the script categorization"),
            );
        }
    }

    valid_scripts
}

/// Validates the README-decompositions.csv table, verifying schema and uniqueness,
/// and returning the set of valid decomposition tags.
pub fn validate_decompositions_table(
    csv_bytes: &[u8],
    file_path: &str,
    report: &mut ValidationReport,
) -> HashSet<String> {
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
            return HashSet::new();
        }
    };

    let mut valid_tags = HashSet::new();
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
                Some("Ensure row has 'Decomposition' and 'Description' columns"),
            );
            continue;
        }

        // Reason for fallback: row bounds checked above (len >= 2), extract trimmed decomposition tag
        let tag = row.get(0).map_or("", |s| s.trim());
        // Reason for fallback: row bounds checked above (len >= 2), extract trimmed description string
        let desc = row.get(1).map_or("", |s| s.trim());

        if tag.is_empty() {
            report.add_error(
                file_path,
                Some(line_no),
                Some("Decomposition"),
                "Decomposition tag cannot be empty".to_string(),
                Some("Specify a tag name for the decomposition"),
            );
        } else if !tag.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            report.add_error(
                file_path,
                Some(line_no),
                Some("Decomposition"),
                format!("Invalid characters in decomposition tag '{tag}'"),
                Some("Decomposition tags must be alphanumeric"),
            );
        } else if !valid_tags.insert(tag.to_string()) {
            report.add_error(
                file_path,
                Some(line_no),
                Some("Decomposition"),
                format!("Duplicate decomposition tag '{tag}'"),
                Some("Ensure each decomposition tag is defined uniquely"),
            );
        }

        if desc.is_empty() {
            report.add_error(
                file_path,
                Some(line_no),
                Some("Description"),
                "Decomposition description cannot be empty".to_string(),
                Some("Provide a description explaining the decomposition semantics"),
            );
        }
    }

    valid_tags
}

/// Extracts the set of named type identifiers from the README-named-types.csv table.
pub fn extract_named_type_names(csv_bytes: &[u8]) -> HashSet<String> {
    let vec_bytes = csv_bytes.to_vec();
    let Ok(table) = csv_tools::parse_csv_reader(
        &vec_bytes,
        csv_tools::CsvParseOptions {
            has_header: true,
            flexible: true,
            ..Default::default()
        },
    ) else {
        return HashSet::new();
    };

    let mut names = HashSet::new();
    for i in 0..table.row_count() {
        if let Some(row) = table.row(i) {
            if let Some(name) = row.get(0) {
                let trimmed = name.trim();
                if !trimmed.is_empty() {
                    names.insert(trimmed.to_string());
                }
            }
        }
    }
    names
}

/// Validates the README-named-types.csv table, verifying syntax declarations
/// for each named type, and returning the set of valid named type names.
pub fn validate_named_types_table(
    csv_bytes: &[u8],
    file_path: &str,
    known_dc_ids: &HashSet<u32>,
    known_format_ids: &HashSet<usize>,
    known_scripts: Option<&HashSet<String>>,
    known_syntax_targets: Option<&HashSet<CharTarget>>,
    report: &mut ValidationReport,
) -> HashSet<String> {
    let all_known_names = extract_named_type_names(csv_bytes);
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
            return HashSet::new();
        }
    };

    let default_scripts = HashSet::new();
    let scripts = known_scripts.unwrap_or(&default_scripts);
    let default_syntax_targets = HashSet::new();
    let syntax_targets = known_syntax_targets.unwrap_or(&default_syntax_targets);

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
                        &all_known_names,
                        scripts,
                        syntax_targets,
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

    seen_names
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
    let parsed = crate::column_spec::parse_aliases_or_base_column(
        raw,
        file_path,
        line_no,
        report,
        false,
    );
    (
        parsed.aliases,
        parsed.cross_references,
        parsed.decompositions,
        parsed.syntax_raw,
    )
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
    let is_generated_csv = file_path.ends_with(".generated.csv");
    let expected_cols = if is_generated_csv { 19 } else { 10 };

    if let Some(header) = table.header() {
        if header.len() != expected_cols {
            report.add_error(
                file_path,
                Some(1),
                None,
                format!(
                    "CSV header has {} columns, expected {} columns",
                    header.len(),
                    expected_cols
                ),
                Some("Ensure CSV header matches schema"),
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

        if row.len() != expected_cols {
            report.add_error(
                file_path,
                Some(line_no),
                None,
                format!("Row has {} columns, expected {}", row.len(), expected_cols),
                Some("Each row in Dc categories must have columns matching schema"),
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
        let (dc_id, is_unicode_char) = if dc_str.starts_with('u') {
            match parse_unicode_shorthand(&dc_str) {
                Ok(cp) => (u128::from(cp), true),
                Err(err) => {
                    let err_msg = err.to_string();
                    if err_msg.contains("exceeds maximum") {
                        // Reason for fallback: starts_with('u') confirmed above, strip 'u' prefix for error message
                        let hex_part = dc_str.strip_prefix('u').unwrap_or("");
                        report.add_error(
                            file_path,
                            Some(line_no),
                            Some("Dc"),
                            format!("Unicode codepoint 'u{hex_part}' exceeds maximum Unicode 0x10FFFF"),
                            Some("Ensure codepoint is within 0x0..=0x10FFFF"),
                        );
                    } else if dc_str
                        .strip_prefix('u')
                        .is_some_and(|h| !h.is_empty() && h.chars().all(|c| c.is_ascii_hexdigit()))
                    {
                        report.add_error(
                            file_path,
                            Some(line_no),
                            Some("Dc"),
                            format!("Invalid hex in Unicode reference: '{dc_str}'"),
                            Some("Must be 'u' followed by valid lowercase hex"),
                        );
                    } else {
                        report.add_error(
                            file_path,
                            Some(line_no),
                            Some("Dc"),
                            format!("Invalid Unicode notation '{dc_str}': must be 'u' followed by 1..=6 lowercase hex digits (e.g. u0020, u0b)"),
                            Some("Only lowercase u<hex> is permitted for Unicode characters in category tables"),
                        );
                    }
                    continue;
                }
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
            if short_str.is_empty()
                || short_str.starts_with("308")
                || parse_unicode_shorthand(&short_str).is_ok()
            {
                None
            } else {
                report.add_error(
                    file_path,
                    Some(line_no),
                    Some("Short"),
                    format!("Invalid Short ID for Unicode character: '{short_str}'"),
                    Some("Short ID for Unicode characters must be blank, lowercase 'u<hex>', or start with 308 (case-sensitive per README.shorthand.md)"),
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

        let (combining_class, bidi_class, general_category, script) = if is_unicode_char {
            #[expect(
                clippy::expect_used,
                reason = "is_unicode_char guarantees dc_id was constructed from a u32 <= 0x10_FFFF"
            )]
            let cp = u32::try_from(dc_id).expect("dc_id fits in u32 for unicode chars");
            let cc = ctb_formats_unicode::combining_class(cp);
            // Reason for fallback: unassigned Unicode codepoints default to Boundary Neutral bidi class
            let bc = validate_bidi_class(ctb_formats_unicode::bidi_class_code(cp))
                .unwrap_or(BidiClass::BN);
            // Reason for fallback: unassigned Unicode codepoints default to NonUnicodeControl general category
            let gc = validate_general_category(ctb_formats_unicode::general_category_code(cp))
                .unwrap_or(GeneralCategory::NonUnicodeControl);
            // Reason for fallback: unassigned Unicode codepoints default to Common script
            let sc = ctb_formats_unicode::script_name(cp)
                .unwrap_or("Common")
                .to_string();
            (cc, bc, gc, sc)
        } else {
            let cc = match validate_combining_class(&combining_str) {
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
            let bc = match validate_bidi_class(&bidi_str) {
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
            let gc = match validate_general_category(&general_cat_str) {
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
            let sc = script.trim().to_string();
            (cc, bc, gc, sc)
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

        let parsed_col = crate::column_spec::parse_aliases_or_base_column(
            &raw_aliases_untrimmed,
            file_path,
            line_no,
            report,
            false,
        );
        let aliases = parsed_col.aliases;
        let cross_references = parsed_col.cross_references;
        let decompositions = parsed_col.decompositions;
        let raw_dc_syntax = parsed_col.syntax_raw;
        let formal_aliases = parsed_col
            .formal_aliases
            .into_iter()
            .map(|f| (f.kind, f.alias))
            .collect();
        let annotations = parsed_col.annotations;

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

        let (row_category, format_details) = if let Some(fmt_cat) = script.strip_prefix("Formats:") {
            let cat_trimmed = fmt_cat.trim();
            if cat_trimmed.is_empty() {
                report.add_error(
                    file_path,
                    Some(line_no),
                    Some("Script"),
                    "Format script 'Formats:' is missing category name (e.g. 'Formats:calendar')".to_string(),
                    Some("Specify category name after 'Formats:'"),
                );
            }
            let formatted_cat = if cat_trimmed.starts_with("format_") {
                cat_trimmed.to_string()
            } else {
                format!("format_{cat_trimmed}")
            };
            (
                formatted_cat,
                Some(crate::dc_def::FormatDetails {
                    implies: None,
                    based_on: None,
                    chain: None,
                    format_spec: None,
                    extensions: None,
                    mime: None,
                    uti: None,
                    apple_type: None,
                    nicknames: if parsed_col.nicknames.is_empty() {
                        None
                    } else {
                        Some(parsed_col.nicknames.join(", "))
                    },
                    import_support: None,
                    export_support: None,
                    tests: None,
                    variant_types: None,
                }),
            )
        } else {
            (category.clone(), None)
        };

        rows.push(DcDefn {
            dc_id,
            short_id,
            ident: parsed_col.rust_ident,
            name,
            category: row_category,
            combining_class,
            bidi_class,
            casing_partner,
            general_category,
            script,
            is_deprecated,
            decompositions,
            aliases,
            formal_aliases,
            cross_references,
            annotations,
            syntax: dc_syntax,
            description,
            format: format_details,
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
    known_decomp_tags: Option<&HashSet<String>>,
    known_named_types: Option<&HashSet<String>>,
    known_scripts: Option<&HashSet<String>>,
    known_format_with_syntax: Option<&HashSet<usize>>,
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
            if let Some(scripts) = known_scripts {
                if !scripts.contains(&row.script) {
                    report.add_error(
                        &row.source_file,
                        Some(row.line_number),
                        Some("Script"),
                        format!("Unknown script '{}'", row.script),
                        Some("Ensure script is registered in README.scripts.csv"),
                    );
                }
            }

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

    let mut known_syntax_targets: HashSet<CharTarget> = all_rows
        .iter()
        .filter(|r| r.syntax.is_some())
        .filter_map(|r| r.short_id)
        .filter_map(|id| u32::try_from(id).ok())
        .map(CharTarget::Dc)
        .collect();
    if let Some(fmts_with_syntax) = known_format_with_syntax {
        for &fid in fmts_with_syntax {
            known_syntax_targets.insert(CharTarget::Format(fid));
        }
    }

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
            if let Some(tags) = known_decomp_tags {
                if !tags.contains(tag_name) {
                    report.add_error(
                        &row.source_file,
                        Some(row.line_number),
                        Some("Aliases (decomposition)"),
                        format!("Unknown decomposition tag '<{tag_name}>'"),
                        Some("Ensure decomposition tag is defined in README-decompositions.csv"),
                    );
                }
            }
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

            let default_named_types = HashSet::new();
            // Reason for fallback: optional named types registry defaults to empty set when omitted
            let named_types = known_named_types.unwrap_or(&default_named_types);
            let default_scripts = HashSet::new();
            let scripts = known_scripts.unwrap_or(&default_scripts);

            validate_dc_syntax(
                syntax_rule,
                short_id_u32,
                &known_dc_ids,
                known_format_ids,
                named_types,
                scripts,
                &known_syntax_targets,
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
    known_decomp_tags: Option<&HashSet<String>>,
    known_named_types: Option<&HashSet<String>>,
    known_scripts: Option<&HashSet<String>>,
    known_format_with_syntax: Option<&HashSet<usize>>,
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
        known_decomp_tags,
        known_named_types,
        known_scripts,
        known_format_with_syntax,
        report,
    )
}

/// Discovers and validates all Dc category files from an on-disk directory.
pub fn validate_all_dc_files_from_disk(
    dc_dir: &std::path::Path,
    known_format_ids: &HashSet<usize>,
    known_decomp_tags: Option<&HashSet<String>>,
    known_named_types: Option<&HashSet<String>>,
    known_scripts: Option<&HashSet<String>>,
    known_format_with_syntax: Option<&HashSet<usize>>,
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
        known_decomp_tags,
        known_named_types,
        known_scripts,
        known_format_with_syntax,
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
        UNIFIED_SCHEMA_HEADER,
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
    fn test_math_format_metadata() -> Result<()> {
        let repo_root = crate::find_repository_root()?;
        let report = validate_all_data_tables_from_repo(&repo_root);
        ensure!(
            !report.has_errors(),
            "Live data table validation failed:\n{}",
            report.format_report()
        );

        let mut report = ValidationReport::new();
        let rows = validate_all_format_files_from_disk(
            &repo_root.join("src/formats/dcdata/data/categories/formats"),
            &mut report,
        );
        ensure!(!report.has_errors(), "{}", report.format_report());

        for (short_id, ident, base) in [
            (505, "Point1D", "f504"),
            (506, "Point2D", "f504"),
            (507, "Point3D", "f504"),
            (509, "Vector1D", "f508"),
            (510, "Vector2D", "f508"),
            (511, "Vector3D", "f508"),
        ] {
            let row = rows
                .iter()
                .find(|row| row.short_id == Some(short_id))
                .context("Missing fixed-dimensional math format")?;
            assert_eq!(row.ident.as_deref(), Some(ident));
            let details = row
                .format
                .as_ref()
                .context("Missing format details")?;
            assert_eq!(details.implies.as_deref(), Some(base));
            assert!(row.syntax.is_some());
            assert!(row.decompositions.is_empty());
        }

        for short_id in 515..=524 {
            let row = rows
                .iter()
                .find(|row| row.short_id == Some(short_id))
                .context("Missing specialized math format")?;
            assert!(row.decompositions.iter().all(|decomposition| {
                !decomposition.starts_with("<semantic>")
            }));
            assert!(row.description.contains("metadata relationships"));
        }
        Ok(())
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
        assert!(gen_stats.unicode_records_merged > 0);
        assert!(gen_stats.total_records_merged >= 4);

        let canonical_header: Vec<String> = UNIFIED_SCHEMA_HEADER
            .iter()
            .map(|s| (*s).to_string())
            .collect();

        // Verify generated DcList.generated.csv contents and order
        let (dc_gen_hdr, dc_gen_rows) = read_csv_file(
            &repo.join("src/formats/dcdata/data/DcList.generated.csv"),
        )
        .unwrap();
        assert_eq!(dc_gen_hdr, canonical_header);
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
        assert_eq!(fmt_gen_hdr, canonical_header);
        assert_eq!(fmt_gen_rows.len(), 2);
        assert_eq!(fmt_gen_rows[0][0], "2228224");
        assert_eq!(fmt_gen_rows[0][1], "f0");
        assert_eq!(fmt_gen_rows[1][0], "2228225");
        assert_eq!(fmt_gen_rows[1][1], "f1");

        // Verify generated unicode.generated.csv
        let (uni_gen_hdr, uni_gen_rows) = read_csv_file(
            &repo.join("src/formats/dcdata/data/unicode.generated.csv"),
        )
        .unwrap();
        assert_eq!(uni_gen_hdr, canonical_header);
        assert_eq!(uni_gen_rows.len(), gen_stats.unicode_records_merged);
        assert_eq!(uni_gen_rows[0][0], "0");
        assert_eq!(uni_gen_rows[0][1], "u0");

        // Verify generated all.generated.csv
        let (all_gen_hdr, all_gen_rows) = read_csv_file(
            &repo.join("src/formats/dcdata/data/all.generated.csv"),
        )
        .unwrap();
        assert_eq!(all_gen_hdr, canonical_header);
        assert_eq!(all_gen_rows.len(), gen_stats.total_records_merged);

        // Verify JSON files are NOT generated
        let dc_json_path = repo.join("src/formats/dcdata/data/DcList.generated.json");
        assert!(!dc_json_path.exists());
        let fmt_json_path = repo.join("src/formats/dcdata/data/formats.generated.json");
        assert!(!fmt_json_path.exists());
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
        let mut known_named_types: HashSet<String> = HashSet::new();
        known_named_types.insert("identifier".to_string());
        let mut known_scripts: HashSet<String> = HashSet::new();
        known_scripts.insert(".EL Types".to_string());
        let known_syntax_targets: HashSet<CharTarget> = HashSet::new();

        // Valid rule
        let valid_rule = parse_dc_syntax(":~ [^248 255]+ 248").unwrap();
        validate_dc_syntax(
            &valid_rule,
            246,
            &known_dcs,
            &known_fmts,
            &known_named_types,
            &known_scripts,
            &known_syntax_targets,
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
            &known_named_types,
            &known_scripts,
            &known_syntax_targets,
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
            &known_named_types,
            &known_scripts,
            &known_syntax_targets,
            &mut report2,
            "test/syntax.csv",
            12,
        );
        assert!(report2.has_errors());
        assert!(report2.format_report().contains("Variable '$unbound'"));

        // Unknown named type construct: [unknown_type]
        let mut report3 = ValidationReport::new();
        let unknown_type_rule = parse_dc_syntax(":[unknown_type]").unwrap();
        validate_dc_syntax(
            &unknown_type_rule,
            246,
            &known_dcs,
            &known_fmts,
            &known_named_types,
            &known_scripts,
            &known_syntax_targets,
            &mut report3,
            "test/syntax.csv",
            13,
        );
        assert!(report3.has_errors());
        assert!(report3.format_report().contains("Unknown named type construct '[unknown_type]'"));

        // Macro expansion of target without syntax: f80 has no syntax
        let mut report4 = ValidationReport::new();
        let macro_no_syntax_rule = parse_dc_syntax(":~ 511 [f80:]").unwrap();
        validate_dc_syntax(
            &macro_no_syntax_rule,
            246,
            &known_dcs,
            &known_fmts,
            &known_named_types,
            &known_scripts,
            &known_syntax_targets,
            &mut report4,
            "test/syntax.csv",
            14,
        );
        assert!(report4.has_errors());
        assert!(report4.format_report().contains("references target that does not define its own syntax rule"));

        // Unknown script in [script:...]: [script:Nonexistent]
        let mut report5 = ValidationReport::new();
        let unknown_script_rule = parse_dc_syntax(":[script:Nonexistent]").unwrap();
        validate_dc_syntax(
            &unknown_script_rule,
            246,
            &known_dcs,
            &known_fmts,
            &known_named_types,
            &known_scripts,
            &known_syntax_targets,
            &mut report5,
            "test/syntax.csv",
            15,
        );
        assert!(report5.has_errors());
        assert!(report5.format_report().contains("Unknown script '[script:Nonexistent]'"));
    }

    #[crate::ctb_test]
    fn test_expression_data_boundaries() -> Result<()> {
        use crate::syntax::{DatasetRuleResolver, MatchMode};
        use std::sync::Arc;

        let resolver = Arc::new(DatasetRuleResolver::load());

        let cases: &[(&str, &[u32], bool)] = &[
            ("[string]", &[260, 262, 264, 263, 261], true),
            ("[string]", &[260, 262, 264, 263, 255, 261, 261], true),
            ("[string]", &[260, 262, 264, 263, 255, 255, 261], true),
            ("[string]", &[260, 262, 264, 263, 255, 65, 261], true),
            ("[string]", &[260, 262, 264, 263, 279, 270, 65, 271, 261], true),
            ("[string]", &[260, 262, 264, 263, 260, 262, 264, 263, 255, 261, 261], true),
            ("[string]", &[], false),
            ("[string]", &[65], false),
            ("[string]", &[260, 261], false),
            ("[string]", &[260, 262, 264, 263, 255], false),
            ("[string]", &[260, 262, 264, 263, 255, 261], false),
            ("[identifier]", &[270, 65, 271], true),
            ("[identifier]", &[270, 255, 271, 271], true),
            ("[identifier]", &[270, 255, 255, 271], true),
            ("[identifier]", &[], false),
            ("[identifier]", &[270, 271], false),
            ("[identifier]", &[270, 65, 255], false),
            ("[value]", &[], false),
            ("[value]", &[65], false),
            ("[value]", &[270, 65, 271], false),
            ("[value]", &[276, 270, 65, 271], true),
            ("[value]", &[276, 262, 264, 263, 270, 65, 271], true),
            ("[value]", &[262, 264, 263, 270, 65, 271], false),
            ("[value]", &[279, 270, 65, 271], true),
            ("[value]", &[279], false),
            ("[statement]", &[], false),
            ("[statement]", &[65], false),
            ("[statement]", &[260, 262, 264, 263, 261], true),
            ("[statement]", &[279, 270, 65, 271], true),
            ("[statement]", &[270, 65, 271, 269, 276, 270, 66, 271], true),
            ("[statement]", &[270, 65, 271, 269], false),
            ("258:", &[258, 279, 270, 65, 271, 259], true),
            ("258:", &[258, 260, 262, 264, 263, 259, 261, 259], true),
            ("258:", &[258, 259], false),
            ("315:", &[315, 276, 270, 65, 271], true),
        ];
        for (syntax, stream, expected) in cases {
            let rule = parse_dc_syntax(syntax)?;
            let mut context = crate::syntax::MatchContext::default()
                .with_resolver(resolver.clone())
                .with_mode(MatchMode::Strict);
            let outcome = crate::syntax::match_pattern(stream, &rule.pattern, &mut context);
            let complete = outcome == (MatchOutcome::Matched { consumed: stream.len() })
                && !context.has_errors;
            assert_eq!(complete, *expected, "{syntax}: {stream:?}: {outcome:?}");
        }

        let rule = parse_dc_syntax("[value]")?;
        let stream = [260, 262, 264, 263, 255, 261, 261, 279, 270, 65, 271];
        let mut context = crate::syntax::MatchContext::default()
            .with_resolver(resolver.clone())
            .with_mode(MatchMode::Strict);
        assert_eq!(
            crate::syntax::match_pattern(&stream, &rule.pattern, &mut context),
            MatchOutcome::Matched { consumed: 7 },
        );
        assert!(!context.has_errors);
        Ok(())
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
    fn test_dc_syntax_framing_and_escaping() {
        use crate::syntax::{
            MatchMode, scan_identifier_frame, scan_literal_frame,
        };

        // 1. Valid typed literal with escape sequences
        let stream = vec![260, 262, 264, 263, 255, 261, 255, 255, 65, 261];
        let mut diags = Vec::new();
        let scanned = scan_literal_frame(&stream, MatchMode::Strict, &mut diags, 0);
        assert!(scanned.is_some());
        let (lit, consumed) = scanned.unwrap();
        assert_eq!(consumed, 10);
        assert_eq!(lit.raw_tokens, stream);
        assert_eq!(lit.type_header, vec![264]);
        assert_eq!(lit.decoded_payload, vec![261, 255, 65]);
        assert!(diags.is_empty());

        // 2. Delimiter belonging to another construct inside literal payload
        let param_stream = vec![260, 262, 264, 263, 259, 261];
        let (lit_p, consumed_p) = scan_literal_frame(
            &param_stream,
            MatchMode::Strict,
            &mut diags,
            0,
        )
        .unwrap();
        assert_eq!(consumed_p, 6);
        assert_eq!(lit_p.decoded_payload, vec![259]);

        // 3. Dangling escape 255 at EOF
        let dangling_stream = vec![260, 262, 264, 263, 65, 255];
        let mut strict_diags = Vec::new();
        assert!(
            scan_literal_frame(
                &dangling_stream,
                MatchMode::Strict,
                &mut strict_diags,
                0
            )
            .is_none()
        );
        let mut perm_diags = Vec::new();
        let perm_scanned = scan_literal_frame(
            &dangling_stream,
            MatchMode::Permissive,
            &mut perm_diags,
            0,
        );
        assert!(perm_scanned.is_some());
        assert!(
            perm_diags
                .iter()
                .any(|d| d.is_error && d.message.contains("Dangling escape"))
        );

        // 4. Missing terminator 261
        let unclosed_stream = vec![260, 262, 264, 263, 65, 66];
        let mut strict_diags2 = Vec::new();
        assert!(
            scan_literal_frame(
                &unclosed_stream,
                MatchMode::Strict,
                &mut strict_diags2,
                0
            )
            .is_none()
        );
        let mut perm_diags2 = Vec::new();
        let perm_scanned2 = scan_literal_frame(
            &unclosed_stream,
            MatchMode::Permissive,
            &mut perm_diags2,
            0,
        );
        assert!(perm_scanned2.is_some());
        assert!(
            perm_diags2
                .iter()
                .any(|d| d.is_error && d.message.contains("Unclosed literal"))
        );

        // 5. Identifier framing
        let ident_stream = vec![270, 65, 255, 271, 271];
        let mut ident_diags = Vec::new();
        let (ident, ident_consumed) = scan_identifier_frame(
            &ident_stream,
            MatchMode::Strict,
            &mut ident_diags,
            0,
        )
        .unwrap();
        assert_eq!(ident_consumed, 5);
        assert_eq!(ident.raw_tokens, ident_stream);
        assert!(ident_diags.is_empty());

        // 6. Empty identifier in strict mode
        let empty_ident = vec![270, 271];
        let mut empty_diags = Vec::new();
        assert!(
            scan_identifier_frame(
                &empty_ident,
                MatchMode::Strict,
                &mut empty_diags,
                0
            )
            .is_none()
        );
    }

    #[crate::ctb_test]
    fn test_dc_syntax_typed_literal_headers_el_types() {
        use crate::syntax::{
            DatasetRuleResolver, MatchContext, MatchOutcome,
            match_syntax_rule_with_context,
        };
        use std::sync::Arc;

        let resolver = Arc::new(DatasetRuleResolver::load());
        let rule = parse_dc_syntax(":~ [script:EL Types] 263").unwrap();

        // Dc 262 defines type definition header: matches String (264), Object (275), Number (278), Routine (280), Routine name (306)
        let valid_type_dcs = [264u32, 275, 278, 280, 306];
        for type_dc in valid_type_dcs {
            let stream = vec![262, type_dc, 263];
            let mut ctx = MatchContext::new(Some(262))
                .with_resolver(resolver.clone());
            let outcome = match_syntax_rule_with_context(&stream, &rule, &mut ctx);
            assert_eq!(
                outcome,
                MatchOutcome::Matched { consumed: 3 },
                "Failed for type Dc {type_dc}"
            );
        }

        // Non-EL Types character (e.g. comment begin Dc 246) must fail to match
        let invalid_stream = vec![262, 246, 263];
        let mut ctx = MatchContext::new(Some(262))
            .with_resolver(resolver.clone());
        let outcome = match_syntax_rule_with_context(&invalid_stream, &rule, &mut ctx);
        assert_eq!(outcome, MatchOutcome::Mismatch);
    }

    #[crate::ctb_test]
    fn test_dc_syntax_non_evaluating_document_parser() {
        use crate::syntax::{MatchMode, ParsedElement, parse_document_tokens};

        // 1. Parse idiomatic-hello-world.sems: say 'Hello, World!'
        let stream = vec![
            256, 258, 260, 262, 264, 263, 57, 86, 93, 93, 96, 30, 18, 72, 96,
            99, 93, 85, 19, 261, 259,
        ];
        let doc = parse_document_tokens(&stream, MatchMode::Permissive);
        assert!(!doc.has_errors);
        assert!(doc.diagnostics.is_empty());
        assert_eq!(doc.elements.len(), 1);

        let Some(ParsedElement::Invocation { target, args }) = doc.elements.first()
        else {
            panic!("Expected Invocation element");
        };
        assert_eq!(**target, ParsedElement::RawTokens(vec![256]));
        assert_eq!(args.len(), 1);

        let Some(ParsedElement::Parameter(param_children)) = args.first() else {
            panic!("Expected Parameter element");
        };
        assert_eq!(param_children.len(), 1);

        let Some(ParsedElement::Literal(lit)) = param_children.first() else {
            panic!("Expected Literal element");
        };
        assert_eq!(lit.type_header, vec![264]);
        assert_eq!(
            lit.decoded_payload,
            vec![57, 86, 93, 93, 96, 30, 18, 72, 96, 99, 93, 85, 19]
        );

        // 2. Mangled / broken document: unclosed literal followed by unparsed tokens
        let mangled = vec![260, 262, 264, 263, 10, 20, 99, 100];
        let mangled_doc = parse_document_tokens(&mangled, MatchMode::Permissive);
        assert!(mangled_doc.has_errors);
        assert!(!mangled_doc.diagnostics.is_empty());
        assert!(!mangled_doc.elements.is_empty());
    }

    #[crate::ctb_test]
    fn test_dc_syntax_resolver_and_expansion_bounds() {
        use crate::syntax::{
            DatasetRuleResolver, MatchContext, MatchOutcome,
            SyntaxRuleResolver, match_pattern,
        };
        use std::sync::Arc;

        let resolver = Arc::new(DatasetRuleResolver::load());

        // Verify resolver loads rules from el.csv and README-named-types.csv
        assert!(resolver.resolve_named_type("list").is_some());
        assert!(resolver.resolve_named_type("key_value").is_some());
        assert!(resolver.resolve_named_type("string").is_some());

        // Bounded recursion test: verify recursion depth limit protects against stack overflow
        let recursive_rule = parse_dc_syntax(":[value]").unwrap();
        let mut ctx = MatchContext::new(None)
            .with_mode(crate::syntax::MatchMode::Permissive)
            .with_resolver(resolver.clone());
        ctx.max_depth = 5;
        ctx.depth = 5;
        let outcome = match_pattern(&[1, 2, 3], &recursive_rule.pattern, &mut ctx);
        assert_eq!(outcome, MatchOutcome::Mismatch);
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
            None,
            None,
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
            None,
            None,
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
            None,
            None,
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
            None,
            None,
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
            None,
            None,
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
            None,
            None,
            &mut report_valid,
        );
        assert!(!report_valid.has_errors());
    }

    #[crate::ctb_test]
    fn test_decomposition_tag_validation() {
        let known_formats: HashSet<usize> = HashSet::new();
        let mut known_tags: HashSet<String> = HashSet::new();
        known_tags.insert("equiv".to_string());
        known_tags.insert("approx".to_string());

        // Unknown decomposition tag <unknown_decomp>
        let mut report = ValidationReport::default();
        let csv_unknown_tag = b"Dc,Short,Name (!=deprecated),\xe2\x97\x8c,\xe2\x87\x86,Aa,Type,Script,Aliases,Description\n1114112,0,Zero,0,BN,,Cc,Controls,,\n1114113,1,One,0,BN,,Po,Controls,<unknown_decomp>0,\n";
        validate_dc_files_data(
            [("test.csv", &csv_unknown_tag[..])],
            "test",
            &known_formats,
            Some(&known_tags),
            None,
            &mut report,
        );
        assert!(report.has_errors());
        assert!(report.format_report().contains("Unknown decomposition tag '<unknown_decomp>'"));

        // Valid decomposition tag <equiv>
        let mut report_valid = ValidationReport::default();
        let csv_valid = b"Dc,Short,Name (!=deprecated),\xe2\x97\x8c,\xe2\x87\x86,Aa,Type,Script,Aliases,Description\n1114112,0,Zero,0,BN,,Cc,Controls,,\n1114113,1,One,0,BN,,Po,Controls,<equiv>0,\n";
        validate_dc_files_data(
            [("test.csv", &csv_valid[..])],
            "test",
            &known_formats,
            Some(&known_tags),
            None,
            &mut report_valid,
        );
        assert!(!report_valid.has_errors());
    }

    #[crate::ctb_test]
    fn test_reject_uppercase_f_format_shorthand() {
        let mut report = ValidationReport::default();
        let invalid_csv = b"Dc,Short,Ident (Rust-friendly),Label,Category,BaseFormat,Extensions,MIME,UTI,Apple Type,Nicknames,Import,Export,Tests,Variants,Comments,References\n2228224,F0,FmtZero,Format Zero,document,,.fz,,,,3,3,1,,,\n";
        let valid_variants = HashSet::new();
        validate_formats_category_file(
            invalid_csv,
            "invalid_fmt.csv",
            &valid_variants,
            &mut report,
        );
        assert!(report.has_errors());
        assert!(report.format_report().contains("Invalid Short format ID integer: 'F0'"));
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

