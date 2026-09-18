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

//! Schema validator for Formats registry category files (`src/formats/dcdata/data/categories/formats/*.csv`).

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::report::ValidationReport;
use crate::dc_def::{DcDefn, FormatDetails};
use crate::shared::{
    BidiClass, GeneralCategory, split_comma_separated_items,
    validate_extensions_field, validate_mime_field, validate_rust_identifier,
    validate_support_level,
};
use crate::format_spec::{parse_format_expr, validate_format_expr};
use crate::syntax::parse_dc_syntax;
use include_dir::Dir;
use std::collections::{HashMap, HashSet};

pub use ctb_storage_minimal::global_graph_layout::{
    FORMAT_REGION_END, FORMAT_REGION_START,
};

/// Parses and validates a single format category CSV file.
pub fn validate_formats_category_file(
    csv_bytes: &[u8],
    file_path: &str,
    valid_variant_names: &HashSet<String>,
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
                format!("Failed to parse CSV file: {e}"),
                Some("Verify CSV syntax and RFC4180 quotes"),
            );
            return Vec::new();
        }
    };

    let mut rows = Vec::new();

    if let Some(header) = table.header() {
        if header.len() != 17 {
            report.add_error(
                file_path,
                Some(1),
                None,
                format!(
                    "CSV header has {} columns, expected 17 columns",
                    header.len()
                ),
                Some("Ensure CSV header has exactly 17 columns matching schema.csv"),
            );
        }
    }

    for i in 0..table.row_count() {
        let line_no = i.saturating_add(2);

        if let Some(row_slice) = table.row(i) {
            if row_slice.iter().all(|s| s.trim().is_empty()) {
                continue;
            }
            if row_slice.len() != 17 {
                report.add_error(
                    file_path,
                    Some(line_no),
                    None,
                    format!(
                        "Row has {} columns, expected 17 columns (mismatched field count)",
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
        let ident = get_str(2);
        let label = get_str(3);
        let category = get_str(4);
        let base_format_raw = get_str(5);
        let extensions = get_str(6);
        let mime = get_str(7);
        let uti = get_str(8);
        let apple_type = get_str(9);
        let nicknames = get_str(10);
        let import_support = get_str(11);
        let export_support = get_str(12);
        let tests = get_str(13);
        let variant_types = get_str(14);
        let comments = get_str(15);
        let references = get_str(16);

        if dc_str.is_empty() && short_str.is_empty() && label.is_empty() {
            continue;
        }

        let short_id = match ctb_storage_minimal::shorthand::parse_format_shorthand(&short_str) {
            Ok(v) => v,
            Err(_) => {
                report.add_error(
                    file_path,
                    Some(line_no),
                    Some("Short"),
                    format!("Invalid Short format ID integer: '{short_str}'"),
                    Some("Must be a non-negative integer or lowercase 'f<digits>'"),
                );
                continue;
            }
        };

        let dc_id = if let Ok(v) = dc_str.parse::<u128>() {
            v
        } else {
            report.add_error(
                file_path,
                Some(line_no),
                Some("Dc"),
                format!("Invalid Global Dc ID integer: '{dc_str}'"),
                Some("Must be an integer within format region"),
            );
            continue;
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

        if !(FORMAT_REGION_START..=FORMAT_REGION_END).contains(&dc_id) {
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

        let is_deprecated = ident.starts_with('!') || label.starts_with('!');
        let clean_ident = if let Some(stripped) = ident.strip_prefix('!') {
            stripped.trim()
        } else {
            ident.as_str()
        };
        let clean_label = if let Some(stripped) = label.strip_prefix('!') {
            stripped.trim()
        } else {
            label.as_str()
        };

        if clean_label.is_empty() {
            report.add_error(
                file_path,
                Some(line_no),
                Some("Label"),
                "Format must have a Label",
                Some("Provide a human-readable display label for the format"),
            );
        }

        if !ident.is_empty() {
            if let Err(e) = validate_rust_identifier(clean_ident) {
                report.add_error(
                    file_path,
                    Some(line_no),
                    Some("Ident"),
                    format!("Invalid Rust identifier '{ident}': {e}"),
                    Some("Identifiers must match [a-zA-Z_][a-zA-Z0-9_]* (optionally prefixed with '!' for deprecated formats) and not clash with keywords"),
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

        let parsed = crate::column_spec::parse_aliases_or_base_column(
            &base_format_raw,
            file_path,
            line_no,
            report,
            true,
        );
        let base_format = if parsed.base_formats.is_empty() {
            None
        } else {
            Some(parsed.base_formats.join(", "))
        };
        let chain = None;
        let format_spec_raw = parsed.format_spec_raw;
        let syntax_raw = parsed.syntax_raw;
        let decompositions = parsed.decompositions;
        let format_spec = if let Some(raw_spec) = &format_spec_raw {
            match parse_format_expr(raw_spec) {
                Ok(expr) => {
                    if let Err(e) = validate_format_expr(&expr) {
                        report.add_error(
                            file_path,
                            Some(line_no),
                            Some("Base/Related Format (format_spec)"),
                            format!("Semantic validation failed for format spec '{raw_spec}': {e}"),
                            Some("Check format IDs and registered named types in expression"),
                        );
                    }
                    Some(expr)
                }
                Err(e) => {
                    report.add_error(
                        file_path,
                        Some(line_no),
                        Some("Base/Related Format (format_spec)"),
                        format!("Failed to parse format specification DSL rule '{raw_spec}': {e}"),
                        Some("Verify format specification DSL syntax (@chain(...))"),
                    );
                    None
                }
            }
        } else {
            None
        };
        let syntax = if let Some(raw_syn) = &syntax_raw {
            match parse_dc_syntax(raw_syn) {
                Ok(rule) => Some(rule),
                Err(e) => {
                    report.add_error(
                        file_path,
                        Some(line_no),
                        Some("Base/Related Format (syntax)"),
                        format!("Failed to parse Format syntax DSL rule: {e}"),
                        Some("Verify syntax DSL grammar"),
                    );
                    None
                }
            }
        } else {
            None
        };

        let formatted_category = if category.starts_with("format_") {
            category.clone()
        } else {
            format!("format_{category}")
        };

        let format_details = FormatDetails {
            base_format,
            chain,
            format_spec,
            extensions: if extensions.is_empty() {
                None
            } else {
                Some(extensions)
            },
            mime: if mime.is_empty() { None } else { Some(mime) },
            uti: if uti.is_empty() { None } else { Some(uti) },
            apple_type: if apple_type.is_empty() {
                None
            } else {
                Some(apple_type)
            },
            nicknames: if nicknames.is_empty() {
                None
            } else {
                Some(nicknames.clone())
            },
            import_support: if import_support.is_empty() {
                None
            } else {
                Some(import_support)
            },
            export_support: if export_support.is_empty() {
                None
            } else {
                Some(export_support)
            },
            tests: if tests.is_empty() { None } else { Some(tests) },
            variant_types: if variant_types.is_empty() {
                None
            } else {
                Some(variant_types)
            },
        };

        let aliases: Vec<String> = if nicknames.is_empty() {
            Vec::new()
        } else {
            split_comma_separated_items(&nicknames)
        };
        let cross_references: Vec<String> = if references.is_empty() {
            Vec::new()
        } else {
            split_comma_separated_items(&references)
        };

        rows.push(DcDefn {
            dc_id,
            short_id: Some(short_id),
            ident: if clean_ident.is_empty() {
                None
            } else {
                Some(clean_ident.to_string())
            },
            name: clean_label.to_string(),
            category: formatted_category,
            combining_class: 0,
            bidi_class: BidiClass::BN,
            casing_partner: None,
            general_category: GeneralCategory::NonUnicodeControl,
            script: "Formats".to_string(),
            is_deprecated,
            decompositions,
            aliases,
            cross_references,
            syntax,
            description: comments,
            format: Some(format_details),
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
) -> Vec<DcDefn>
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
        if !path_str.ends_with(".csv")
            || path_str.ends_with("schema.csv")
            || path_str.ends_with(".generated.csv")
        {
            continue;
        }

        let rows = validate_formats_category_file(
            contents,
            path_str,
            &variant_categories,
            report,
        );

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
                            "Duplicate Short Format ID {short_id} already defined in {prev_file}:{prev_line}",
                        ),
                        Some("Assign a unique Short ID to each format"),
                    );
                } else {
                    short_id_map.insert(
                        short_id,
                        (row.source_file.clone(), row.line_number),
                    );
                }
            }

            if let Some(ident) = &row.ident {
                if let Some((prev_file, prev_line)) = ident_map.get(ident) {
                    report.add_error(
                        &row.source_file,
                        Some(row.line_number),
                        Some("Ident"),
                        format!(
                            "Duplicate Format Ident '{ident}' already defined in {prev_file}:{prev_line}"
                        ),
                        Some("Assign a unique Rust-friendly Ident to each format"),
                    );
                } else {
                    ident_map.insert(
                        ident.clone(),
                        (row.source_file.clone(), row.line_number),
                    );
                }
            }

            all_rows.push(row);
        }
    }

    // Validate that Short Format IDs form a contiguous sequence starting from 0 with no gaps/holes
    let known_fmt_ids: HashSet<usize> =
        all_rows.iter().filter_map(|r| r.short_id).collect();
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
) -> Vec<DcDefn> {
    let mut files = Vec::new();
    for f in formats_dir.files() {
        if let Some(path_str) = f.path().to_str() {
            files.push((path_str, f.contents()));
        }
    }
    validate_format_files_data(
        files,
        "src/formats/dcdata/data/categories/formats/",
        report,
    )
}

/// Discovers all format category files from an on-disk directory and validates uniqueness across files.
pub fn validate_all_format_files_from_disk(
    formats_dir: &std::path::Path,
    report: &mut ValidationReport,
) -> Vec<DcDefn> {
    let Ok(entries) = std::fs::read_dir(formats_dir) else {
        report.add_error(
            &formats_dir.display().to_string(),
            None,
            None,
            format!(
                "Could not read Formats directory at {}",
                formats_dir.display()
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

    validate_format_files_data(
        files_iter,
        &formats_dir.display().to_string(),
        report,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[crate::ctb_test]
    fn test_format_label_required() {
        let mut report = ValidationReport::new();
        let valid_variants = HashSet::new();

        // Row with blank label but present ident
        let csv_data = b"Dc,Short,Ident (Rust-friendly),Label,Category,Base,Ext,MIME,UTI,Apple,Nick,Imp,Exp,Tests,Var,Comments,Ref\n2228224,0,MyFormat,,document,,,,,,,,,,,, \n";
        validate_formats_category_file(
            csv_data,
            "test/formats/test.csv",
            &valid_variants,
            &mut report,
        );

        assert!(report.has_errors());
        assert!(report.format_report().contains("Format must have a Label"));
    }

    #[crate::ctb_test]
    fn test_deprecated_format_ident_allowed() {
        let mut report = ValidationReport::new();
        let valid_variants = HashSet::new();

        let csv_data = b"Dc,Short,Ident (Rust-friendly),Label,Category,Base,Ext,MIME,UTI,Apple,Nick,Imp,Exp,Tests,Var,Comments,Ref\n2228224,0,!Gregorian,Gregorian calendar,calendar,,,,,,,,,,,, \n";
        let rows = validate_formats_category_file(
            csv_data,
            "test/formats/calendar.csv",
            &valid_variants,
            &mut report,
        );

        assert!(!report.has_errors(), "Unexpected errors: {}", report.format_report());
        assert_eq!(rows.len(), 1);
        assert!(rows[0].is_deprecated);
        assert_eq!(rows[0].ident.as_deref(), Some("Gregorian"));
        assert_eq!(rows[0].name, "Gregorian calendar");
    }

    #[crate::ctb_test]
    fn test_deprecated_format_invalid_ident_fails() {
        let mut report = ValidationReport::new();
        let valid_variants = HashSet::new();

        let csv_data = b"Dc,Short,Ident (Rust-friendly),Label,Category,Base,Ext,MIME,UTI,Apple,Nick,Imp,Exp,Tests,Var,Comments,Ref\n2228224,0,!123Invalid,Invalid ident format,calendar,,,,,,,,,,,, \n";
        validate_formats_category_file(
            csv_data,
            "test/formats/calendar.csv",
            &valid_variants,
            &mut report,
        );

        assert!(report.has_errors());
        assert!(report.format_report().contains("Invalid Rust identifier '!123Invalid'"));
    }

    #[crate::ctb_test]
    fn test_format_spec_chain_directive_parsed() {
        let mut report = ValidationReport::new();
        let valid_variants = HashSet::new();

        let csv_data = b"Dc,Short,Ident (Rust-friendly),Label,Category,\"Base/Related Format/Category, Chain (@), or Syntax (:)\",Ext,MIME,UTI,Apple,Nick,Imp,Exp,Tests,Var,Comments,Ref\n2228224,0,TestMojibake,Test Mojibake,encoding,\"@chain(((f15 > f542) ! f0) > f0)\",,,,,,,,,,, \n";
        let rows = validate_formats_category_file(
            csv_data,
            "test/formats/encoding.csv",
            &valid_variants,
            &mut report,
        );

        assert!(!report.has_errors(), "Unexpected errors: {}", report.format_report());
        assert_eq!(rows.len(), 1);
        let details = rows[0].format.as_ref().unwrap();
        assert!(details.format_spec.is_some());
        let spec_str = format!("{}", details.format_spec.as_ref().unwrap());
        assert_eq!(spec_str, "((f15 > f542) ! f0) > f0");
    }

    #[crate::ctb_test]
    fn test_format_spec_chain_invalid_syntax_reports_error() {
        let mut report = ValidationReport::new();
        let valid_variants = HashSet::new();

        // Ambiguous unparenthesized mixing of & and | inside @chain(...)
        let csv_data = b"Dc,Short,Ident (Rust-friendly),Label,Category,\"Base/Related Format/Category, Chain (@), or Syntax (:)\",Ext,MIME,UTI,Apple,Nick,Imp,Exp,Tests,Var,Comments,Ref\n2228224,0,TestBadChain,Test Bad Chain,encoding,\"@chain(f15 & f542 | f0)\",,,,,,,,,,, \n";
        validate_formats_category_file(
            csv_data,
            "test/formats/encoding.csv",
            &valid_variants,
            &mut report,
        );

        assert!(report.has_errors());
        let err = report.format_report();
        assert!(err.contains("Failed to parse format specification DSL rule"));
    }

    #[crate::ctb_test]
    fn test_legacy_chain_syntax_reports_error() {
        let mut report = ValidationReport::new();
        let valid_variants = HashSet::new();

        let csv_data = b"Dc,Short,Ident (Rust-friendly),Label,Category,\"Base/Related Format/Category, Chain (@), or Syntax (:)\",Ext,MIME,UTI,Apple,Nick,Imp,Exp,Tests,Var,Comments,Ref\n2228224,0,TestOldChain,Test Old Chain,encoding,\"=f15 > f0\",,,,,,,,,,, \n";
        validate_formats_category_file(
            csv_data,
            "test/formats/encoding.csv",
            &valid_variants,
            &mut report,
        );

        assert!(report.has_errors());
        let err = report.format_report();
        assert!(err.contains("Legacy chain syntax '=' is deprecated"));
    }
}



