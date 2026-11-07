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

#[cfg(test)]
pub(crate) use ctb_build_support::{dc_codegen, find_repository_root_from};
pub(crate) use ctb_build_support::{fs2, sha2};
#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

pub use crate as ctb_formats_dc_data;

#[path = "../../build_support/ctbnbdl.rs"]
pub mod ctbnbdl;
#[rustfmt::skip]
#[path = "dc.generated.rs"]
pub mod dc;
pub mod dc_char;
pub mod dc_def;
pub mod dc_number_minimal;
pub mod dcts;
pub mod format_spec;
pub mod lookup;
pub mod seed_graph;
pub mod support;
pub mod syntax;

pub use support::{
    column_spec, format, graph_ledger, layout, report, shared, updater,
    validation,
};

pub use column_spec::{
    ParsedAliasesOrBaseColumn, parse_aliases_or_base_column,
};

pub use ctb_formats_dctf as dctf;
pub use ctb_storage_minimal::global_graph_layout;
pub use ctb_storage_minimal::shorthand;
pub use ctb_storage_minimal::shorthand::{
    DcShorthand, is_valid_shorthand, parse_format_shorthand,
    parse_unicode_shorthand,
};
pub use dcts::{
    DctsOptions, dclist_to_dcts, dclist_to_dcts_string_with_options,
    dclist_to_dcts_with_options,
};

pub use ctb_formats_unicode::describe_general_category;
pub use ctb_utilities::DC_INTEGER;
pub use dc::{
    DC_ASSIGN, DC_BASE64_END, DC_BASE64_PADDING, DC_BASE64_START,
    DC_BEGIN_BLOCK_COMMENT, DC_BEGIN_BLOCK_COMMENT_NEW_LINE,
    DC_BEGIN_CHECKSUMMED_SECTION, DC_BEGIN_DATE_DURATION,
    DC_BEGIN_DATE_MAGNITUDE, DC_BEGIN_ENVIRONMENT_DESCRIPTION,
    DC_BEGIN_ERROR_RECORD, DC_BEGIN_FILE_FLAGS_DESCRIPTION,
    DC_BEGIN_FORMAT_REPRESENTATION, DC_BEGIN_IDENTIFIER, DC_BEGIN_KV_MAP,
    DC_BEGIN_KV_PAIR, DC_BEGIN_LEGACY_KV_MAP, DC_BEGIN_LIST, DC_BEGIN_LITERAL,
    DC_BEGIN_NUMBER, DC_BEGIN_OPERATION_SESSION, DC_BEGIN_PARAMETER,
    DC_BEGIN_SINGLE_LINE_COMMENT, DC_BEGIN_SINGLE_LINE_COMMENT_NEW_LINE,
    DC_BEGIN_TIME_MOMENT, DC_BEGIN_TYPE_DEF, DC_BEGIN_WARNING_RECORD,
    DC_BOOLEAN_FALSE, DC_BOOLEAN_TRUE, DC_DESTINATION_PATH,
    DC_END_BLOCK_COMMENT, DC_END_CHECKSUMMED_SECTION, DC_END_DATE_DURATION,
    DC_END_DATE_MAGNITUDE, DC_END_ENCAPSULATION_BINARY,
    DC_END_ENCAPSULATION_UTF8, DC_END_ENVIRONMENT_DESCRIPTION,
    DC_END_ERROR_RECORD, DC_END_FILE_FLAGS_DESCRIPTION,
    DC_END_FORMAT_REPRESENTATION, DC_END_IDENTIFIER, DC_END_KV_MAP,
    DC_END_KV_PAIR, DC_END_LEGACY_KV_MAP, DC_END_LIST, DC_END_LITERAL,
    DC_END_NUMBER, DC_END_OPERATION_SESSION, DC_END_PARAMETER,
    DC_END_SINGLE_LINE_COMMENT, DC_END_TIME_MOMENT, DC_END_TYPE_DEF,
    DC_END_WARNING_RECORD, DC_ENTER_IDENTIFIER_SCOPE,
    DC_EQUIVALENT_REPRESENTATIONS, DC_ESCAPE, DC_EXIT_IDENTIFIER_SCOPE,
    DC_FILE_PATH_DELIMITER, DC_FORMAT_PARAM_APPLICATION, DC_LANG_ASSIGN,
    DC_LANG_SAY, DC_LIST_ELEMENT_SEPARATOR, DC_LOG_MESSAGE, DC_LONG_DC,
    DC_NEGATIVE, DC_OPERATION_COMPLETED, DC_OPERATION_STATUS_CODE,
    DC_OPTIONAL_ABSENT, DC_OPTIONAL_PRESENT,
    DC_PARTIALLY_EQUIVALENT_REPRESENTATIONS, DC_POSITIVE, DC_RAW_FILE_FLAGS,
    DC_REFERENCE_BOUND_VALUE, DC_REFERENCE_NAMED_OBJECT,
    DC_REPLACEMENT_UNAVAIL_DC, DC_REPLACEMENT_UNAVAIL_UNICODE,
    DC_ROUTINE_INVOCATION, DC_SECTION_DIGEST, DC_SEQUENCE_IDENTIFIER,
    DC_SOURCE_PATH, DC_START_ENCAPSULATION_BINARY, DC_START_ENCAPSULATION_UTF8,
    DC_TRACE_IDENTIFIER, DC_TYPE_ANY, DC_TYPE_CONVERSION, DC_TYPE_INTERSECTION,
    DC_TYPE_NUMBER, DC_TYPE_ROUTINE, DC_TYPE_ROUTINE_NAME, DC_TYPE_STRING,
    DC_TYPE_TRANSFORMATION, DC_TYPE_TRANSMUTATION, DC_TYPE_UNION,
    SHORT_DC_REGION_END, SHORT_DC_REGION_START, long_to_short_dc,
    short_to_long_dc,
};
pub use dc_char::{DcChar, DcCharExt, is_type_token};
pub use dc_def::{DcDefn, FormatDetails};
pub use dc_number_minimal::{
    dc_base64_char_to_digit, dc_base64_char_to_digit_with_resolver,
    dc_base64_digit_to_char, is_dc_base64_encapsulation_char,
    is_dc_base64_encapsulation_char_with_resolver, read_dc_number_u128,
    read_dc_number_u128_with_resolver, u128_to_dc_number_chars,
    u128_to_dc_number_short,
};
pub use format::{
    FORMAT_REGION_END, FORMAT_REGION_START, validate_all_format_files,
    validate_all_format_files_from_disk, validate_format_files_data,
    validate_formats_category_file,
};
pub use format_spec::{
    DcToken, FormatExpr, FormatOp, MAX_FORMAT_EXPR_DEPTH,
    MAX_FORMAT_EXPR_NODES, decode_dc_stream, decode_dc_stream_string,
    encode_dc_stream, encode_dc_stream_string, format_chain_directive,
    format_expr, parse_format_expr, validate_format_expr,
};
pub use layout::{
    ParsedLayoutRow, UNICODE_REGION_END, UNICODE_REGION_START,
    validate_layout_table,
};
pub use lookup::{
    get_all_dc_defns, get_all_format_defns, get_dc_count, get_dc_defn,
    get_dc_name, get_eite_dc_data_rows, get_short_dc_defn, get_short_dc_name,
    maximum_known_short_dc,
};
pub use report::{ValidationDiagnostic, ValidationReport, ValidationSeverity};
pub use seed_graph::{
    ActivePropertyValue, PRIMARY_SEED_GRAPH_ASSET_KEY, ReconstructedNamedType,
    SEED_GRAPH_ASSET_GLOB, SYSTEM_GRAPH_HISTORY_ASSET_GLOB, SeedGraphIndex,
};
pub use shared::{
    BidiClass, GeneralCategory, SupportLevel, validate_bidi_class,
    validate_combining_class, validate_cross_table_uniqueness,
    validate_extension_entry, validate_extensions_field,
    validate_general_category, validate_mime_field, validate_rust_identifier,
    validate_support_level,
};
pub use syntax::{
    ActionArg, CharTarget, DcSyntaxExpr, DcSyntaxRule, Quantifier,
    SyntaxAction, SyntaxElement, SyntaxPattern, SyntaxTerm, parse_dc_syntax,
    parse_target_token, validate_dc_syntax,
};
pub use updater::{
    MergedGenerationStats, TableUpdateStats, assign_and_update_dc_categories,
    assign_and_update_format_categories,
    check_category_csvs_need_no_id_updates, find_repository_root,
    generate_merged_csvs, is_empty_row, is_unassigned_id, read_csv_file,
    write_csv_file,
};
pub use validation::{
    parse_dc_aliases_column, split_dc_aliases_column, validate_all_data_tables,
    validate_all_data_tables_embedded, validate_all_data_tables_from_repo,
    validate_all_dc_files, validate_all_dc_files_from_disk,
    validate_dc_aliases_spacing, validate_dc_category_file,
    validate_dc_files_data,
};

use std::sync::LazyLock;

static DROID_SIGNATURE_XML: LazyLock<Option<Vec<u8>>> = LazyLock::new(|| {
    let key = "data/detection/droid/signatures/DROID_SignatureFile_V125.xml";
    if let Some(bytes) = ctb_storage_asset_bundle::get_asset(key) {
        return Some(bytes);
    }
    // Reason for fallback: when running tests or before asset bundle repack, load from repository assets directory
    let disk_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../assets")
        .join(key);
    std::fs::read(disk_path).ok()
});

static DROID_CONTAINER_SIGNATURE_XML: LazyLock<Option<Vec<u8>>> =
    LazyLock::new(|| {
        let key =
            "data/detection/droid/signatures/container-signature-20260119.xml";
        if let Some(bytes) = ctb_storage_asset_bundle::get_asset(key) {
            return Some(bytes);
        }
        // Reason for fallback: when running tests or before asset bundle repack, load from repository assets directory
        let disk_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../assets")
            .join(key);
        std::fs::read(disk_path).ok()
    });

static MAGDIR_FILES: LazyLock<Vec<(String, Vec<u8>)>> = LazyLock::new(|| {
    let mut files = Vec::new();
    if let Ok(paths) = ctb_storage_asset_bundle::find_assets(
        "data/detection/magic/upstream/magic/Magdir/*",
    ) {
        for path in paths {
            if let Some(bytes) = ctb_storage_asset_bundle::get_asset(&path) {
                files.push((path, bytes));
            }
        }
    }
    if files.is_empty() {
        // Reason for fallback: when running tests or before asset bundle repack, load from repository assets directory
        let magdir_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../assets/data/detection/magic/upstream/magic/Magdir");
        if let Ok(entries) = std::fs::read_dir(&magdir_path) {
            let mut sorted_entries = entries.flatten().collect::<Vec<_>>();
            sorted_entries.sort_by_key(|e| e.path());
            for entry in sorted_entries {
                let p = entry.path();
                if p.is_file() {
                    if let Ok(bytes) = std::fs::read(&p) {
                        let name = p
                            .file_name()
                            .map(|n| n.to_string_lossy().into_owned())
                            // Reason for fallback: default to empty string if path has no file name
                            .unwrap_or_default();
                        files.push((name, bytes));
                    }
                }
            }
        }
    }
    files
});

/// Returns the raw bytes of a data file within the runtime asset bundle or source repository,
/// or a source data file from `src/formats/dcdata/data-src/` when running in test mode.
pub fn get_dc_data_file(key: &str) -> Option<Vec<u8>> {
    let load_from_data_src = |rel: &str| -> Option<Vec<u8>> {
        // Reason for fallback: strip optional data-src/ or fixtures/ prefix, defaulting to full relative key
        let clean = rel
            .strip_prefix("data-src/")
            .or_else(|| rel.strip_prefix("fixtures/"))
            .unwrap_or(rel);
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("data-src")
            .join(clean);
        std::fs::read(path).ok()
    };
    if key.starts_with("data-src/") || key.starts_with("fixtures/") {
        return load_from_data_src(key);
    }

    let candidate_keys = [
        if key.starts_with("data/detection/") {
            key.to_string()
        } else if let Some(stripped) = key.strip_prefix("data/") {
            format!("data/detection/{stripped}")
        } else if let Some(stripped) = key.strip_prefix("detection/") {
            format!("data/detection/{stripped}")
        } else {
            format!("data/detection/{key}")
        },
        if key.starts_with("data/") {
            key.to_string()
        } else {
            format!("data/{key}")
        },
        key.to_string(),
    ];

    for asset_key in &candidate_keys {
        if let Some(bytes) = ctb_storage_asset_bundle::get_asset(asset_key) {
            return Some(bytes);
        }
        // Reason for fallback: when running tests or before asset bundle repack, load from repository assets directory
        let disk_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../assets")
            .join(asset_key);
        if let Ok(bytes) = std::fs::read(&disk_path) {
            return Some(bytes);
        }
    }

    if ctb_utilities::testing::is_in_test() {
        return load_from_data_src(key);
    }
    None
}

pub(crate) fn get_dc_categories_dir_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("data-src")
        .join("categories")
}

pub(crate) fn get_formats_categories_dir_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("data-src")
        .join("categories/formats")
}

/// Returns all bundled Magdir rule files as (`relative_path`, `content_bytes`) pairs.
pub fn get_dc_magdir_files() -> Vec<(&'static str, &'static [u8])> {
    MAGDIR_FILES
        .iter()
        .map(|(path, bytes)| (path.as_str(), bytes.as_slice()))
        .collect()
}

/// Returns the canonical DROID PRONOM binary signature file (V125) as raw bytes.
pub fn get_droid_signature_xml() -> Option<&'static [u8]> {
    DROID_SIGNATURE_XML.as_deref()
}

/// Returns the canonical DROID container signature file as raw bytes.
pub fn get_droid_container_signature_xml() -> Option<&'static [u8]> {
    DROID_CONTAINER_SIGNATURE_XML.as_deref()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[crate::ctb_test]
    fn test_detection_assets_loading_from_asset_bundle() {
        let ctb_magic = get_dc_data_file("magic/ctoolbox.magic");
        assert!(
            ctb_magic.is_some(),
            "Expected magic/ctoolbox.magic to be accessible"
        );

        let droid_sig = get_droid_signature_xml();
        assert!(
            droid_sig.is_some(),
            "Expected DROID signature XML to be accessible"
        );

        let container_sig = get_droid_container_signature_xml();
        assert!(
            container_sig.is_some(),
            "Expected DROID container signature XML to be accessible"
        );

        let magdir_files = get_dc_magdir_files();
        assert!(
            !magdir_files.is_empty(),
            "Expected Magdir files to be loaded from asset bundle"
        );
    }
}
