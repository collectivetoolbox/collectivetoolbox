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

#[expect(
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

    for i in 0..table.row_count() {
        let get_str = |col: usize| -> String {
            match table.cell(i, col) {
                Some(s) => s.trim().to_string(),
                None => String::new(),
            }
        };

        let dc_str = get_str(0);
        let Ok(dc_id) = dc_str.parse::<u128>() else {
            continue;
        };

        let id_str = get_str(1);
        let Ok(id) = id_str.parse::<usize>() else {
            continue;
        };

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

        map.insert(
            id,
            FormatInfo {
                dc_id,
                id,
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
            },
        );
    }
}

static FORMATS_BY_ID: LazyLock<HashMap<usize, FormatInfo>> =
    LazyLock::new(|| {
        let mut map = HashMap::new();
        if let Some(file) =
            crate::FORMATS_UTILITIES_DATA_DIR.get_file("formats.generated.csv")
        {
            parse_format_csv_data(file.contents(), &mut map);
        } else if let Some(file) =
            crate::FORMATS_UTILITIES_DATA_DIR.get_file("formats.csv")
        {
            parse_format_csv_data(file.contents(), &mut map);
        }
        if map.is_empty() {
            if let Some(dir) =
                crate::FORMATS_UTILITIES_DATA_DIR.get_dir("formats")
            {
                for file in dir.files() {
                    if file.path().extension().and_then(|ext| ext.to_str())
                        == Some("csv")
                    {
                        parse_format_csv_data(file.contents(), &mut map);
                    }
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
    if !info.base_format.is_empty() {
        lines.push(format!("Base format: {base}", base = info.base_format));
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
#[expect(
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
}
