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

//! Metadata lookup and formatting for registered document and stream formats.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::Result;
use std::collections::HashMap;
use std::sync::LazyLock;

/// Detailed metadata record for a format from format category CSV files.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FormatInfo {
    pub dc_id: u128,
    pub id: usize,
    pub ident: String,
    pub label: String,
    pub category: String,
    pub implies: String,
    pub based_on: String,
    pub extensions: String,
    pub mime: String,
    pub uti: String,
    pub apple_type: String,
    pub nicknames: String,
    pub import_support: String,
    pub export_support: String,
    pub tests: String,
    pub variant_types: String,
    pub comments: String,
    pub references: String,
}

fn parse_format_csv_data(bytes: &[u8], map: &mut HashMap<usize, FormatInfo>) {
    let vec_bytes = bytes.to_vec();
    let table = match csv_tools::parse_csv_reader(
        &vec_bytes,
        csv_tools::CsvParseOptions {
            has_header: true,
            ..Default::default()
        },
    ) {
        Ok(t) => t,
        Err(_) => return,
    };

    let (
        dc_col,
        id_col,
        ident_col,
        label_col,
        category_col,
        script_col,
        base_col,
        ext_col,
        mime_col,
        uti_col,
        apple_type_col,
        nicknames_col,
        import_col,
        export_col,
        tests_col,
        variant_col,
        comments_col,
        references_col,
    ) = if let Some(hdr) = table.header() {
        let find_col_opt = |prefixes: &[&str]| -> Option<usize> {
            hdr.iter().position(|name| {
                let trimmed = name.trim();
                prefixes.iter().any(|p| trimmed.starts_with(p))
            })
        };
        let find_col = |prefixes: &[&str], default: usize| -> usize {
            if let Some(pos) = find_col_opt(prefixes) {
                pos
            } else {
                default
            }
        };

        (
            find_col(&["Dc"], 0),
            find_col(&["Short"], 1),
            find_col_opt(&["Ident"]),
            find_col(&["Label", "Name"], 2),
            find_col_opt(&["Category"]),
            find_col_opt(&["Script"]),
            find_col_opt(&["Base", "Aliases"]),
            find_col_opt(&["Extensions"]),
            find_col_opt(&["MIME"]),
            find_col_opt(&["Apple Uniform", "Apple UTI"]),
            find_col_opt(&["Apple Type"]),
            find_col_opt(&["Nicknames"]),
            find_col_opt(&["Import"]),
            find_col_opt(&["Export"]),
            find_col_opt(&["Tests"]),
            find_col_opt(&["Variant"]),
            find_col_opt(&["Comments", "Description"]),
            find_col_opt(&["References"]),
        )
    } else {
        (
            0,
            1,
            Some(2),
            3,
            Some(4),
            None,
            Some(5),
            Some(6),
            Some(7),
            Some(8),
            Some(9),
            Some(10),
            Some(11),
            Some(12),
            Some(13),
            Some(14),
            Some(15),
            Some(16),
        )
    };

    for i in 0..table.row_count() {
        let get_str = |col: usize| -> String {
            match table.cell(i, col) {
                Some(s) => s.trim().to_string(),
                None => String::new(),
            }
        };
        let get_opt = |col_opt: Option<usize>| -> String {
            if let Some(col) = col_opt {
                get_str(col)
            } else {
                String::new()
            }
        };

        let dc_str = get_str(dc_col);
        let Ok(dc_id) = dc_str.parse::<u128>() else {
            continue;
        };

        let id_str = get_str(id_col);
        let Ok(id) = ctb_formats_dcdata::parse_format_shorthand(&id_str) else {
            continue;
        };

        let mut ident = get_opt(ident_col);
        let raw_label = get_str(label_col);
        let label = if let Some(s) = raw_label.strip_prefix('!') {
            s.trim().to_string()
        } else {
            raw_label
        };

        let raw_cat = get_opt(category_col);
        let category = if !raw_cat.is_empty() {
            raw_cat
        } else {
            let sc = get_opt(script_col);
            if let Some(stripped) = sc.strip_prefix(".Formats:") {
                stripped.to_string()
            } else {
                sc
            }
        };

        let base_or_aliases = get_opt(base_col);
        let mut nicknames = get_opt(nicknames_col);
        let mut implies = String::new();
        let mut based_on = String::new();

        if !base_or_aliases.is_empty() {
            let mut report = ctb_formats_dcdata::report::ValidationReport::new();
            let parsed = ctb_formats_dcdata::column_spec::parse_aliases_or_base_column(
                &base_or_aliases,
                "",
                0,
                &mut report,
                false,
            );
            if ident.is_empty() {
                if let Some(id_str) = parsed.rust_ident {
                    ident = id_str;
                }
            }
            if nicknames.is_empty() && !parsed.nicknames.is_empty() {
                nicknames = parsed.nicknames.join(", ");
            }
            if !parsed.implies.is_empty() {
                implies = parsed.implies.join(", ");
            }
            if !parsed.based_on.is_empty() {
                based_on = parsed.based_on.join(", ");
            }
        }

        let extensions = get_opt(ext_col);
        let mime = get_opt(mime_col);
        let uti = get_opt(uti_col);
        let apple_type = get_opt(apple_type_col);
        let import_support = get_opt(import_col);
        let export_support = get_opt(export_col);
        let tests = get_opt(tests_col);
        let variant_types = get_opt(variant_col);
        let comments = get_opt(comments_col);
        let references = get_opt(references_col);

        map.insert(
            id,
            FormatInfo {
                dc_id,
                id,
                ident,
                label,
                category,
                implies,
                based_on,
                extensions,
                mime,
                uti,
                apple_type,
                nicknames,
                import_support,
                export_support,
                tests,
                variant_types,
                comments,
                references,
            },
        );
    }
}

static FORMATS_BY_ID: LazyLock<HashMap<usize, FormatInfo>> =
    LazyLock::new(|| {
        let mut map = HashMap::new();
        if let Some(bytes) =
            ctb_formats_dcdata::get_dc_data_file("formats.generated.csv")
        {
            parse_format_csv_data(&bytes, &mut map);
        } else if let Some(bytes) =
            ctb_formats_dcdata::get_dc_data_file("formats.csv")
        {
            parse_format_csv_data(&bytes, &mut map);
        }
        if map.is_empty() {
            for file in ctb_formats_dcdata::FORMATS_CATEGORIES_DIR.files() {
                if file.path().extension().and_then(|ext| ext.to_str())
                    == Some("csv")
                {
                    parse_format_csv_data(file.contents(), &mut map);
                }
            }
        }
        map
    });

/// Look up a `FormatInfo` record by short Format ID.
pub fn get_format_info(fmt_id: usize) -> Option<FormatInfo> {
    FORMATS_BY_ID.get(&fmt_id).cloned()
}

fn format_support_level(level: &str) -> String {
    match level.trim() {
        "-1" => "-1 (N/A)".to_string(),
        "0" => "0 (none)".to_string(),
        "1" => "1 (WIP)".to_string(),
        "2" => "2 (mostly / partial)".to_string(),
        "3" => "3 (fully implemented)".to_string(),
        "4" => "4 (lossless / roundtrippable)".to_string(),
        "5" => "5 (lossless with strict validation)".to_string(),
        other => other.to_string(),
    }
}

/// Formats detailed metadata for a short Format ID.
///
/// Output format includes the Global Graph ID (offset by 2,228,224), label, category,
/// extensions, MIME types, import/export support levels, and comments.
pub fn describe_format(fmt_id: usize) -> Result<String> {
    let info = get_format_info(fmt_id)
        .ok_or_else(|| anyhow::anyhow!("Unknown Format ID: {fmt_id}"))?;

    let gid = if info.dc_id != 0 {
        info.dc_id
    } else {
        2_228_224_u128.saturating_add(u128::try_from(fmt_id)?)
    };
    let title = if !info.label.is_empty() {
        &info.label
    } else if !info.ident.is_empty() {
        &info.ident
    } else {
        "Unknown Format"
    };

    let mut lines = Vec::new();
    lines.push(format!("{gid}"));
    lines.push(title.to_string());
    lines.push(String::new());

    if !info.ident.is_empty() && info.ident != info.label {
        lines.push(format!("Ident: {ident}", ident = info.ident));
    }
    if !info.category.is_empty() {
        lines.push(format!("Category: {cat}", cat = info.category));
    }
    if !info.mime.is_empty() {
        lines.push(format!("MIME: {mime}", mime = info.mime));
    }
    if !info.extensions.is_empty() {
        lines.push(format!("Extensions: {ext}", ext = info.extensions));
    }
    if !info.uti.is_empty() {
        lines.push(format!("Apple UTI: {uti}", uti = info.uti));
    }
    if !info.apple_type.is_empty() {
        lines.push(format!("Apple Type code: {at}", at = info.apple_type));
    }
    if !info.nicknames.is_empty() {
        lines.push(format!("Nicknames: {nick}", nick = info.nicknames));
    }
    if !info.implies.is_empty() {
        lines.push(format!("Implies: {imp}", imp = info.implies));
    }
    if !info.based_on.is_empty() {
        lines.push(format!("Based on: {base}", base = info.based_on));
    }
    if !info.import_support.is_empty() {
        lines.push(format!(
            "Import support: {imp}",
            imp = format_support_level(&info.import_support)
        ));
    }
    if !info.export_support.is_empty() {
        lines.push(format!(
            "Export support: {exp}",
            exp = format_support_level(&info.export_support)
        ));
    }
    if !info.tests.is_empty() {
        lines.push(format!(
            "Tests: {tst}",
            tst = format_support_level(&info.tests)
        ));
    }
    if !info.variant_types.is_empty() {
        lines.push(format!("Variant types: {vt}", vt = info.variant_types));
    }
    if !info.comments.is_empty() {
        lines.push(format!("Comments: {comm}", comm = info.comments));
    }
    if !info.references.is_empty() {
        lines.push(format!("References: {refs}", refs = info.references));
    }

    Ok(lines.join("\n"))
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
    fn test_get_format_info_80() {
        let info = get_format_info(80).expect("Format 80 exists");
        assert_eq!(info.id, 80);
        assert_eq!(info.ident, "String");
        assert_eq!(info.label, "String");
        assert_eq!(info.category, "semantic");
    }

    #[crate::ctb_test]
    fn test_describe_format_80() {
        let desc = describe_format(80).expect("Describe format 80");
        assert!(desc.starts_with("2228304\nString\n\nCategory: semantic"));
    }

    #[crate::ctb_test]
    fn test_describe_format_0_utf8() {
        let desc = describe_format(0).expect("Describe format 0");
        assert!(desc.starts_with("2228224\nUTF-8"));
        assert!(desc.contains("Ident: Utf8"));
        assert!(desc.contains("Category: encoding"));
        assert!(desc.contains("Extensions: .txt, .utf8"));
    }

    #[crate::ctb_test]
    fn test_implies_and_based_on_loading_and_formatting() {
        // f546 = QuartzDisplay, implies f271 (RasterDisplay)
        let quartz = get_format_info(546).expect("Format 546 exists");
        assert_eq!(quartz.ident, "QuartzDisplay");
        assert_eq!(quartz.implies, "f271");
        assert!(quartz.based_on.is_empty());
        let quartz_desc = describe_format(546).expect("Describe 546");
        assert!(quartz_desc.contains("Implies: f271"));

        // f561 = Guix, based on f580 (Nix)
        let guix = get_format_info(561).expect("Format 561 exists");
        assert_eq!(guix.ident, "Guix");
        assert_eq!(guix.based_on, "f580");
        assert!(guix.implies.is_empty());
        let guix_desc = describe_format(561).expect("Describe 561");
        assert!(guix_desc.contains("Based on: f580"));

        // f572 = Ubuntu, based on f571 (Debian) and implies f398 (GnuLinux)
        let ubuntu = get_format_info(572).expect("Format 572 exists");
        assert_eq!(ubuntu.ident, "Ubuntu");
        assert_eq!(ubuntu.based_on, "f571");
        assert_eq!(ubuntu.implies, "f398");
        let ubuntu_desc = describe_format(572).expect("Describe 572");
        assert!(ubuntu_desc.contains("Based on: f571"));
        assert!(ubuntu_desc.contains("Implies: f398"));
    }
}
