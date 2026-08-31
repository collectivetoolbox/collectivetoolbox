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

pub use dc::{
    DC_REGION_END, DC_REGION_START, ParsedDcRow, split_dc_aliases_column,
    validate_all_dc_files, validate_dc_category_file,
};
pub use format::{
    FORMAT_REGION_END, FORMAT_REGION_START, ParsedFormatRow,
    validate_all_format_files, validate_formats_category_file,
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

use std::collections::HashSet;
use include_dir::{Dir, include_dir};

pub static DCTEXT_CATEGORIES_DIR: Dir =
    include_dir!("$CARGO_MANIFEST_DIR/../dctext/data/categories");

pub static FORMATS_CATEGORIES_DIR: Dir =
    include_dir!("$CARGO_MANIFEST_DIR/../utilities/data/formats");

pub static STORAGE_MINIMAL_DATA_DIR: Dir =
    include_dir!("$CARGO_MANIFEST_DIR/../../storage/minimal/data");

/// Runs comprehensive validation across all repository data tables.
pub fn validate_all_data_tables() -> ValidationReport {
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
}
