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

//! Comprehensive table validators and facet splitters for Document Characters (Dcs),
//! Formats, and Global Graph Layout datasets.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

pub mod dc;
pub mod format;
pub mod layout;
pub mod report;
pub mod shared;
pub mod updater;

pub use dc::{
    DC_REGION_END, DC_REGION_START, ParsedDcRow, split_dc_aliases_column,
    validate_all_dc_files, validate_all_dc_files_from_disk,
    validate_dc_category_file, validate_dc_files_data,
};
pub use format::{
    FORMAT_REGION_END, FORMAT_REGION_START, ParsedFormatRow,
    validate_all_format_files, validate_all_format_files_from_disk,
    validate_format_files_data, validate_formats_category_file,
};
pub use layout::{
    EXPECTED_DC_END, EXPECTED_DC_START, EXPECTED_FORMAT_END,
    EXPECTED_FORMAT_START, EXPECTED_UNICODE_END, EXPECTED_UNICODE_START,
    ParsedLayoutRow, validate_layout_table,
};
pub use report::{ValidationDiagnostic, ValidationReport, ValidationSeverity};
pub use shared::{
    validate_bidi_class, validate_combining_class,
    validate_cross_table_uniqueness, validate_extension_entry,
    validate_extensions_field, validate_general_category, validate_mime_field,
    validate_rust_identifier, validate_support_level,
};
pub use updater::{
    MergedGenerationStats, TableUpdateStats, assign_and_update_dc_categories,
    assign_and_update_format_categories, find_repository_root,
    generate_merged_csvs, is_empty_row, is_unassigned_id, read_csv_file,
    write_csv_file,
};

use std::collections::HashSet;
use std::path::Path;
use include_dir::{Dir, include_dir};

pub static DCTEXT_CATEGORIES_DIR: Dir =
    include_dir!("$CARGO_MANIFEST_DIR/../dctext/data/categories");

pub static FORMATS_CATEGORIES_DIR: Dir =
    include_dir!("$CARGO_MANIFEST_DIR/../utilities/data/formats");

pub static STORAGE_MINIMAL_DATA_DIR: Dir =
    include_dir!("$CARGO_MANIFEST_DIR/../../storage/minimal/data");

/// Runs comprehensive validation across all repository data tables.
/// Automatically validates the live on-disk files if the repository root is
/// found, otherwise validates the embedded snapshots.
pub fn validate_all_data_tables() -> ValidationReport {
    if let Ok(repo_root) = find_repository_root() {
        validate_all_data_tables_from_repo(&repo_root)
    } else {
        validate_all_data_tables_embedded()
    }
}

/// Runs validation using the embedded directory snapshots.
pub fn validate_all_data_tables_embedded() -> ValidationReport {
    let mut report = ValidationReport::new();

    // 1. Validate Global Graph Layout table
    if let Some(file) = STORAGE_MINIMAL_DATA_DIR.get_file("global-graph-layout.csv") {
        validate_layout_table(
            file.contents(),
            "storage/minimal/data/global-graph-layout.csv",
            &mut report,
        );
    } else {
        report.add_error(
            "storage/minimal/data/global-graph-layout.csv",
            None,
            None,
            "Could not locate global-graph-layout.csv",
            Some("Ensure file exists in storage/minimal/data/"),
        );
    }

    // 2. Validate Formats category files
    let format_rows = validate_all_format_files(&FORMATS_CATEGORIES_DIR, &mut report);
    let known_format_ids: HashSet<usize> = format_rows.iter().map(|r| r.short_id).collect();

    // 3. Validate Document Characters category files
    let dc_rows = validate_all_dc_files(
        &DCTEXT_CATEGORIES_DIR,
        &known_format_ids,
        &mut report,
    );

    // 4. Validate Cross-Table Name / Label Uniqueness
    let dc_names: Vec<(u32, &str, &str)> = dc_rows
        .iter()
        .map(|r| (r.short_id, r.name.as_str(), r.source_file.as_str()))
        .collect();

    let format_labels: Vec<(usize, &str, &str)> = format_rows
        .iter()
        .map(|r| (r.short_id, r.label.as_str(), r.source_file.as_str()))
        .collect();

    validate_cross_table_uniqueness(&dc_names, &format_labels, &mut report);

    report
}

/// Runs validation directly against live files in a repository directory.
pub fn validate_all_data_tables_from_repo(repo_root: &Path) -> ValidationReport {
    let mut report = ValidationReport::new();

    // 1. Validate Global Graph Layout table
    let layout_path = repo_root.join("src/storage/minimal/data/global-graph-layout.csv");
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
    let formats_dir = repo_root.join("src/formats/utilities/data/formats");
    let format_rows = validate_all_format_files_from_disk(&formats_dir, &mut report);
    let known_format_ids: HashSet<usize> = format_rows.iter().map(|r| r.short_id).collect();

    // 3. Validate Document Characters category files
    let dc_dir = repo_root.join("src/formats/dctext/data/categories");
    let dc_rows = validate_all_dc_files_from_disk(
        &dc_dir,
        &known_format_ids,
        &mut report,
    );

    // 4. Validate Cross-Table Name / Label Uniqueness
    let dc_names: Vec<(u32, &str, &str)> = dc_rows
        .iter()
        .map(|r| (r.short_id, r.name.as_str(), r.source_file.as_str()))
        .collect();

    let format_labels: Vec<(usize, &str, &str)> = format_rows
        .iter()
        .map(|r| (r.short_id, r.label.as_str(), r.source_file.as_str()))
        .collect();

    validate_cross_table_uniqueness(&dc_names, &format_labels, &mut report);

    report
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

    #[crate::ctb_test]
    fn test_validate_all_data_tables_repository() {
        let report = validate_all_data_tables();
        if report.has_errors() {
            panic!("Data table validation failed:\n{}", report.format_report());
        }
    }

    #[crate::ctb_test]
    fn test_validate_rust_identifier() {
        assert!(validate_rust_identifier("Utf8").is_ok());
        assert!(validate_rust_identifier("_Valid123").is_ok());
        assert!(validate_rust_identifier("MyFormat").is_ok());

        assert!(validate_rust_identifier("").is_err());
        assert!(validate_rust_identifier("123abc").is_err());
        assert!(validate_rust_identifier("bad-name").is_err());
        assert!(validate_rust_identifier("type").is_err());
        assert!(validate_rust_identifier("match").is_err());
    }

    #[crate::ctb_test]
    fn test_validate_extension_rules() {
        // Plain extensions must have leading dots
        assert!(validate_extension_entry(".txt").is_ok());
        assert!(validate_extension_entry(".tar.gz").is_ok());
        assert!(validate_extension_entry("txt").is_err());

        // Regex patterns with ~...~
        assert!(validate_extension_entry(r"~^\._~").is_ok());
        assert!(validate_extension_entry(r"~/\.AppleDouble/~").is_ok());
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
        let (aliases_s, xrefs_s, decomps_s, syntax_s) = split_dc_aliases_column(raw_syntax);
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
        let rows = validate_dc_category_file(csv_data, "test/categories/test.csv", &mut report);
        assert_eq!(rows.len(), 2);

        // Run gap validation logic
        let known_dc_ids: HashSet<u32> = rows.iter().map(|r| r.short_id).collect();
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

        let dc_cats = repo.join("src/formats/dctext/data/categories");
        let fmt_cats = repo.join("src/formats/utilities/data/formats");
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
            "".to_string(),
        ];
        write_csv_file(&repo.join("src/formats/dctext/data/schema.csv"), &dc_schema_header, &[]).unwrap();

        let fmt_schema_header = vec![
            "Dc".to_string(),
            "Short".to_string(),
            "Ident (Rust-friendly)".to_string(),
            "Label".to_string(),
            "Category".to_string(),
            "MIME".to_string(),
            "Extensions".to_string(),
            "UTI".to_string(),
            "Apple Type".to_string(),
            "Nicknames".to_string(),
            "BaseFormat".to_string(),
            "Import".to_string(),
            "Export".to_string(),
            "Tests".to_string(),
            "Variants".to_string(),
            "Comments".to_string(),
            "References".to_string(),
        ];
        write_csv_file(&repo.join("src/formats/utilities/data/schema.csv"), &fmt_schema_header, &[]).unwrap();

        // Write category files with AUTO IDs
        let dc_rows = vec![
            vec!["1114112".to_string(), "0".to_string(), "Null".to_string(), "0".to_string(), "BN".to_string(), "".to_string(), "Cc".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string()],
            vec!["".to_string(), "AUTO".to_string(), "CustomDc".to_string(), "0".to_string(), "BN".to_string(), "".to_string(), "Po".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string()],
        ];
        write_csv_file(&dc_cats.join("custom.csv"), &dc_schema_header, &dc_rows).unwrap();

        let fmt_rows = vec![
            vec!["2228224".to_string(), "0".to_string(), "FormatZero".to_string(), "Format Zero".to_string(), "document".to_string(), "".to_string(), ".fz".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "3".to_string(), "3".to_string(), "1".to_string(), "".to_string(), "".to_string(), "".to_string()],
            vec!["".to_string(), "AUTO".to_string(), "FormatOne".to_string(), "Format One".to_string(), "document".to_string(), "".to_string(), ".fo".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "3".to_string(), "3".to_string(), "1".to_string(), "".to_string(), "".to_string(), "".to_string()],
        ];
        write_csv_file(&fmt_cats.join("doc.csv"), &fmt_schema_header, &fmt_rows).unwrap();

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
        let (dc_gen_hdr, dc_gen_rows) = read_csv_file(&repo.join("src/formats/dctext/data/DcList.generated.csv")).unwrap();
        assert_eq!(dc_gen_hdr, dc_schema_header);
        assert_eq!(dc_gen_rows.len(), 2);
        assert_eq!(dc_gen_rows[0][0], "1114112");
        assert_eq!(dc_gen_rows[0][1], "0");
        assert_eq!(dc_gen_rows[1][0], "1114113");
        assert_eq!(dc_gen_rows[1][1], "1");

        // Verify generated formats.generated.csv contents and order
        let (fmt_gen_hdr, fmt_gen_rows) = read_csv_file(&repo.join("src/formats/utilities/data/formats.generated.csv")).unwrap();
        assert_eq!(fmt_gen_hdr, fmt_schema_header);
        assert_eq!(fmt_gen_rows.len(), 2);
        assert_eq!(fmt_gen_rows[0][0], "2228224");
        assert_eq!(fmt_gen_rows[0][1], "0");
        assert_eq!(fmt_gen_rows[1][0], "2228225");
        assert_eq!(fmt_gen_rows[1][1], "1");
    }
}
