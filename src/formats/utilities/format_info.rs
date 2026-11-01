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
use std::path::{Path, PathBuf};
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

static FORMATS_BY_ID: LazyLock<HashMap<usize, FormatInfo>> =
    LazyLock::new(|| {
        let mut map = HashMap::new();
        for defn in ctb_formats_dcdata::get_all_format_defns() {
            let Some(id) = defn.short_id else {
                continue;
            };
            let Some(fmt) = &defn.format else {
                continue;
            };
            // Reason for fallback: script strings without the ".Formats:" prefix are already bare category names.
            let category = defn
                .script
                .strip_prefix(".Formats:")
                .unwrap_or(&defn.script)
                .to_string();
            // Reason for fallback: optional format metadata fields are represented as empty strings in FormatInfo when absent.
            let ident = defn.ident.clone().unwrap_or_default();
            // Reason for fallback: optional format metadata fields are represented as empty strings in FormatInfo when absent.
            let implies = fmt.implies.clone().unwrap_or_default();
            // Reason for fallback: optional format metadata fields are represented as empty strings in FormatInfo when absent.
            let based_on = fmt.based_on.clone().unwrap_or_default();
            // Reason for fallback: optional format metadata fields are represented as empty strings in FormatInfo when absent.
            let extensions = fmt.extensions.clone().unwrap_or_default();
            // Reason for fallback: optional format metadata fields are represented as empty strings in FormatInfo when absent.
            let mime = fmt.mime.clone().unwrap_or_default();
            // Reason for fallback: optional format metadata fields are represented as empty strings in FormatInfo when absent.
            let uti = fmt.uti.clone().unwrap_or_default();
            // Reason for fallback: optional format metadata fields are represented as empty strings in FormatInfo when absent.
            let apple_type = fmt.apple_type.clone().unwrap_or_default();
            // Reason for fallback: optional format metadata fields are represented as empty strings in FormatInfo when absent.
            let nicknames = fmt.nicknames.clone().unwrap_or_default();
            // Reason for fallback: optional format metadata fields are represented as empty strings in FormatInfo when absent.
            let import_support = fmt.import_support.clone().unwrap_or_default();
            // Reason for fallback: optional format metadata fields are represented as empty strings in FormatInfo when absent.
            let export_support = fmt.export_support.clone().unwrap_or_default();
            // Reason for fallback: optional format metadata fields are represented as empty strings in FormatInfo when absent.
            let tests = fmt.tests.clone().unwrap_or_default();
            // Reason for fallback: optional format metadata fields are represented as empty strings in FormatInfo when absent.
            let variant_types = fmt.variant_types.clone().unwrap_or_default();

            map.insert(
                id,
                FormatInfo {
                    dc_id: defn.dc_id,
                    id,
                    ident,
                    label: defn.name.clone(),
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
                    comments: defn.description.clone(),
                    references: defn.cross_references.join(", "),
                },
            );
        }
        map
    });

impl FormatInfo {
    /// Returns the nicknames as a sorted list of strings: shortest first, then
    /// alphabetically for ties.
    #[must_use]
    pub fn sorted_nicknames(&self) -> Vec<String> {
        if self.nicknames.is_empty() {
            return Vec::new();
        }
        let mut nicks: Vec<String> = self
            .nicknames
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        nicks.sort_by(|a, b| a.len().cmp(&b.len()).then_with(|| a.cmp(b)));
        nicks
    }

    /// Returns the primary file extension (without leading dot or case prefix).
    #[must_use]
    pub fn primary_extension(&self) -> Option<&str> {
        self.get_primary_extension()
    }

    /// Returns the primary file extension (without leading dot or case prefix).
    #[must_use]
    pub fn get_primary_extension(&self) -> Option<&str> {
        if self.extensions.is_empty() {
            return None;
        }
        self.extensions
            .split(',')
            .next()
            .map(|ext| {
                let trimmed = ext.trim();
                // Reason for fallback: extensions without "case:" prefix are kept as-is
                let clean = trimmed.strip_prefix("case:").unwrap_or(trimmed);
                clean.trim_start_matches('.')
            })
            .filter(|s| !s.is_empty())
    }
}

/// Extension trait allowing `.get_primary_extension()` on `Option<&'static FormatInfo>`.
pub trait FormatInfoOptionExt {
    /// Returns the primary file extension if format info is present.
    fn get_primary_extension(&self) -> Option<&'static str>;
}

impl FormatInfoOptionExt for Option<&'static FormatInfo> {
    fn get_primary_extension(&self) -> Option<&'static str> {
        self.and_then(FormatInfo::get_primary_extension)
    }
}

static FORMATS_BY_FORMAT_ID: LazyLock<
    HashMap<ctb_utilities::FormatId, FormatInfo>,
> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for info in FORMATS_BY_ID.values() {
        if let Some(fid) = ctb_utilities::FormatId::from_ident(&info.ident) {
            map.insert(fid, info.clone());
        } else if let Some(fid) =
            ctb_utilities::FormatId::from_shorthand(&format!("f{}", info.id))
        {
            map.insert(fid, info.clone());
        }
    }
    map
});

/// Look up a `FormatInfo` record by short Format ID.
pub fn get_format_info(fmt_id: usize) -> Option<&'static FormatInfo> {
    FORMATS_BY_ID.get(&fmt_id)
}

/// Look up a `FormatInfo` record by `FormatId`.
pub fn get_format_info_by_id(
    format_id: ctb_utilities::FormatId,
) -> Option<&'static FormatInfo> {
    FORMATS_BY_FORMAT_ID.get(&format_id)
}

/// Generates a help table of supported formats and their shorthand aliases for
/// CLI `after_help` text.
pub fn format_help_table(
    header: &str,
    formats: &[ctb_utilities::FormatId],
) -> String {
    let mut lines = Vec::new();
    lines.push(header.to_string());
    let mut entries: Vec<(String, String)> = Vec::new();
    for &fid in formats {
        if let Some(info) = get_format_info_by_id(fid) {
            let aliases = info.sorted_nicknames();
            let alias_str = if aliases.is_empty() {
                info.ident.clone()
            } else {
                aliases.join(", ")
            };
            let sort_key = alias_str.to_ascii_lowercase();
            entries
                .push((sort_key, format!("  {}: {}", alias_str, info.label)));
        }
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    for (_, line) in entries {
        lines.push(line);
    }
    lines.join("\n")
}

/// Generates a categorized, alphabetically sorted help table of supported formats
/// showing both CLI nicknames and Rust `FormatId` names alongside their descriptions.
pub fn format_categorized_help_table(
    header: &str,
    formats: &[ctb_utilities::FormatId],
) -> String {
    let mut lines = Vec::new();
    if !header.is_empty() {
        lines.push(header.to_string());
    }

    let category_order = [
        ("document", "Document formats:"),
        ("encoding", "Text encodings:"),
        ("database", "Database & archive formats:"),
        ("container", "Container formats:"),
        ("compression", "Compression formats:"),
        ("hash", "Hash formats:"),
    ];

    let mut by_category: std::collections::BTreeMap<
        String,
        Vec<(String, String)>,
    > = std::collections::BTreeMap::new();

    for &fid in formats {
        if let Some(info) = get_format_info_by_id(fid) {
            let nicks: Vec<&str> = info
                .nicknames
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .collect();
            // Reason for fallback: formats without short CLI nicknames sort by their lowercased canonical FormatId identifier.
            let primary_key = nicks.first().map_or_else(
                || info.ident.to_ascii_lowercase(),
                |s| s.to_ascii_lowercase(),
            );
            let name_display = if nicks.is_empty() {
                format!("{} ({})", info.ident.to_ascii_lowercase(), info.ident)
            } else {
                format!("{} ({})", nicks.join(", "), info.ident)
            };
            let cat_key = info.category.trim().to_ascii_lowercase();
            by_category.entry(cat_key).or_default().push((
                primary_key,
                format!("    {name_display}: {}", info.label),
            ));
        }
    }

    for &(cat_key, cat_title) in &category_order {
        if let Some(mut entries) = by_category.remove(cat_key) {
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            lines.push(format!("  {cat_title}"));
            for (_, line) in entries {
                lines.push(line);
            }
        }
    }

    for (cat_key, mut entries) in by_category {
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        lines.push(format!("  {cat_key} formats:"));
        for (_, line) in entries {
            lines.push(line);
        }
    }

    lines.join("\n")
}

/// Strips an extension belonging to the given `FormatId` from the input path.
pub fn strip_format_extension(
    input_path: &Path,
    format_id: ctb_utilities::FormatId,
) -> Option<PathBuf> {
    let path_str = input_path.to_string_lossy();
    let mut rules =
        crate::extension_data::extension_rules_for_format(format_id);
    rules.sort_by(|a, b| b.extension.len().cmp(&a.extension.len()));
    for rule in rules {
        if let Some(prefix) = rule.strip_from_filename(&path_str) {
            return Some(PathBuf::from(prefix));
        }
    }
    None
}

/// Infers the output file path when decompressing without an explicit output
/// path. Checks registered extension rules for compression formats (longest
/// extensions first) and strips the matched extension, or appends
/// `.decompressed` if none is recognized.
pub fn infer_decompressed_path(input_path: &Path) -> PathBuf {
    let path_str = input_path.to_string_lossy();
    let mut compression_rules: Vec<crate::detection::extension::ExtensionRule> =
        crate::extension_data::EXTENSION_REGISTRY
            .iter()
            .filter(|e| {
                e.format_id.category()
                    == ctb_utilities::format_id::FormatCategory::Compression
            })
            .map(|e| e.rule)
            .collect();
    // Sort longer extensions first so e.g. .old.z or .Z1.0 matches before .z or .Z
    compression_rules.sort_by(|a, b| b.extension.len().cmp(&a.extension.len()));
    for rule in compression_rules {
        if let Some(prefix) = rule.strip_from_filename(&path_str) {
            return PathBuf::from(prefix);
        }
    }
    PathBuf::from(format!("{}.decompressed", input_path.display()))
}

fn format_support_level(level: &str) -> String {
    let trimmed = level.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    if let [gen_part, dc_part] = parts.as_slice() {
        let gen_str = format_single_support_level(gen_part);
        let dc_str = format_single_support_level(dc_part);
        format!("{gen_str} [Dc: {dc_str}]")
    } else {
        format_single_support_level(trimmed)
    }
}

fn format_single_support_level(level: &str) -> String {
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
        assert!(
            desc.contains(
                "Import support: 3 (fully implemented) [Dc: 1 (WIP)]"
            )
        );
        assert!(desc.contains(
            "Export support: 3 (fully implemented) [Dc: 2 (mostly / partial)]"
        ));
        assert!(desc.contains("Tests: 1 (WIP) [Dc: 1 (WIP)]"));
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

    #[crate::ctb_test]
    fn test_format_help_table_and_nicknames() {
        let table = format_help_table(
            "Supported compression formats:",
            &[FormatId::Brotli, FormatId::Gzip, FormatId::Xz],
        );
        assert!(table.contains("Supported compression formats:"));
        assert!(table.contains("br, brotli: Brotli compressed stream"));
        assert!(table.contains("gz, gzip: GNU gzip format"));
        assert!(table.contains("xz: XZ compression"));
        assert!(!table.contains("xzip"));
    }

    #[crate::ctb_test]
    fn test_infer_decompressed_path_shared() {
        assert_eq!(
            infer_decompressed_path(Path::new("file.txt.gz")),
            PathBuf::from("file.txt")
        );
        assert_eq!(
            infer_decompressed_path(Path::new("archive.tar.Z")),
            PathBuf::from("archive.tar")
        );
        assert_eq!(
            infer_decompressed_path(Path::new("document.z")),
            PathBuf::from("document")
        );
        assert_eq!(
            infer_decompressed_path(Path::new("document.C")),
            PathBuf::from("document")
        );
        assert_eq!(
            infer_decompressed_path(Path::new("data.tar.xz")),
            PathBuf::from("data.tar")
        );
        // .xzip is NOT .xz, should not strip as xz
        assert_eq!(
            infer_decompressed_path(Path::new("data.tar.xzip")),
            PathBuf::from("data.tar.xzip.decompressed")
        );
        assert_eq!(
            infer_decompressed_path(Path::new("unknown.bin")),
            PathBuf::from("unknown.bin.decompressed")
        );
    }
}
