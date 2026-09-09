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

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

pub mod dc;
pub mod dc_def;
pub mod format;
pub mod layout;
pub mod report;
pub mod shared;
pub mod syntax;
pub mod updater;
pub mod validation;

pub use dc::{SHORT_DC_REGION_END, SHORT_DC_REGION_START};
pub use dc_def::{DcDefn, FormatDetails};
pub use format::{
    FORMAT_REGION_END, FORMAT_REGION_START,
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
    BidiClass, GeneralCategory, validate_bidi_class, validate_combining_class,
    validate_cross_table_uniqueness, validate_extension_entry,
    validate_extensions_field, validate_general_category, validate_mime_field,
    validate_rust_identifier, validate_support_level,
};
pub use syntax::{
    ActionArg, CharTarget, DcSyntaxRule, MatchContext, MatchOutcome,
    Quantifier, SyntaxAction, SyntaxElement, SyntaxPattern, SyntaxTerm,
    match_pattern, match_syntax_rule, parse_dc_syntax, parse_target_token,
    validate_dc_syntax,
};
pub use updater::{
    MergedGenerationStats, TableUpdateStats, assign_and_update_dc_categories,
    assign_and_update_format_categories, find_repository_root,
    generate_merged_csvs, is_empty_row, is_unassigned_id, read_csv_file,
    write_csv_file,
};
pub use validation::{
    split_dc_aliases_column, validate_all_data_tables,
    validate_all_data_tables_embedded, validate_all_data_tables_from_repo,
    validate_all_dc_files, validate_all_dc_files_from_disk,
    validate_dc_category_file, validate_dc_files_data,
};

use include_dir::{Dir, include_dir};

pub static DC_CATEGORIES_DIR: Dir =
    include_dir!("$CARGO_MANIFEST_DIR/data/categories");

pub static DCTEXT_CATEGORIES_DIR: Dir =
    include_dir!("$CARGO_MANIFEST_DIR/data/categories");

pub static FORMATS_CATEGORIES_DIR: Dir =
    include_dir!("$CARGO_MANIFEST_DIR/../utilities/data/formats");

pub static STORAGE_MINIMAL_DATA_DIR: Dir =
    include_dir!("$CARGO_MANIFEST_DIR/../../storage/minimal/data");


