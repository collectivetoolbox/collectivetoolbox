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

//! Codegen for file extension mappings and `extension_data.generated.rs` from
//! format category CSV data tables.

use anyhow::{Context, Result, ensure};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::find_repository_root_from;
use crate::license_consts::DEFAULT_AGPL_HEADER;

fn write_if_changed(path: &Path, content: &str) -> Result<bool> {
    if path.is_file() {
        if let Ok(existing) = fs::read_to_string(path) {
            if existing == content {
                return Ok(false);
            }
        }
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(true)
}

/// A parsed file extension mapping record.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ExtensionRecord {
    format_ident: String,
    extension: String,
    case_sensitive: bool,
}

/// Generates the contents of `extension_data.generated.rs` from formats category CSV files.
///
/// # Errors
/// Returns an error if reading directory or CSV parsing fails.
pub fn generate_extension_data_code(formats_dir: &Path) -> Result<String> {
    let mut records: Vec<ExtensionRecord> = Vec::new();
    let mut seen: HashSet<(String, String)> = HashSet::new();

    if formats_dir.is_dir() {
        let mut entries = Vec::new();
        for entry in fs::read_dir(formats_dir)? {
            entries.push(entry?);
        }
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let path = entry.path();
            if path.is_file() {
                let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else {
                    continue;
                };
                if file_name.ends_with(".csv")
                    && file_name != "schema.csv"
                    && !file_name.ends_with(".generated.csv")
                {
                    let mut rdr = csv::ReaderBuilder::new()
                        .has_headers(true)
                        .flexible(true)
                        .from_path(&path)
                        .with_context(|| {
                            format!("Failed to open CSV file {}", path.display())
                        })?;

                    for result in rdr.records() {
                        let record = result.with_context(|| {
                            format!("Failed to read record in {}", path.display())
                        })?;
                        let format_ident = record.get(2).unwrap_or("").trim().to_string();
                        if format_ident.is_empty() {
                            continue;
                        }
                        let raw_extensions = record.get(6).unwrap_or("").trim();
                        if raw_extensions.is_empty() {
                            continue;
                        }

                        for part in raw_extensions.split(',') {
                            let trimmed = part.trim();
                            if trimmed.is_empty() {
                                continue;
                            }
                            let (ext_str, case_sensitive) = if let Some(rest) =
                                trimmed.strip_prefix("case:")
                            {
                                (rest.trim(), true)
                            } else {
                                (trimmed, false)
                            };

                            if ext_str.starts_with('~') && ext_str.ends_with('~') {
                                continue;
                            }
                            let clean = ext_str.trim_start_matches('.');
                            if clean.is_empty() {
                                continue;
                            }

                            if seen.insert((format_ident.clone(), clean.to_string())) {
                                records.push(ExtensionRecord {
                                    format_ident: format_ident.clone(),
                                    extension: clean.to_string(),
                                    case_sensitive,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    let mut out = String::new();
    out.push_str(DEFAULT_AGPL_HEADER);
    out.push_str("\n\n");
    out.push_str("//! Centralized registry of file extension rules across format types.\n");
    out.push_str("//! @generated by ctb-build-support::extension_codegen from format category data tables.\n");
    out.push_str("//! Do not edit by hand.\n\n");
    out.push_str("use crate::detection::extension::ExtensionRule;\n");
    out.push_str("use crate::format_id::FormatId;\n");
    out.push_str("#[expect(\n");
    out.push_str("    unused_imports,\n");
    out.push_str("    clippy::wildcard_imports,\n");
    out.push_str("    reason = \"Standard workspace module prelude\"\n");
    out.push_str(")]\n");
    out.push_str("use ctb_utilities::*;\n\n");

    out.push_str("/// An entry associating a `FormatId` with an `ExtensionRule`.\n");
    out.push_str("#[derive(Debug, Clone, Copy)]\n");
    out.push_str("pub struct ExtensionEntry {\n");
    out.push_str("    pub format_id: FormatId,\n");
    out.push_str("    pub rule: ExtensionRule,\n");
    out.push_str("}\n\n");

    out.push_str("/// Global static registry of single file extension mappings.\n");
    out.push_str("pub static EXTENSION_REGISTRY: &[ExtensionEntry] = &[\n");
    for r in &records {
        let rule_ctor = if r.case_sensitive {
            format!("ExtensionRule::sensitive(\"{}\")", r.extension)
        } else {
            format!("ExtensionRule::insensitive(\"{}\")", r.extension)
        };
        out.push_str("    ExtensionEntry {\n");
        out.push_str(&format!("        format_id: FormatId::{},\n", r.format_ident));
        out.push_str(&format!("        rule: {rule_ctor},\n"));
        out.push_str("    },\n");
    }
    out.push_str("];\n\n");

    out.push_str("/// Helper to lookup `FormatId` from a single extension string.\n");
    out.push_str("pub fn lookup_format_by_extension(ext: &str) -> Vec<FormatId> {\n");
    out.push_str("    let mut matches = Vec::new();\n");
    out.push_str("    for entry in EXTENSION_REGISTRY {\n");
    out.push_str("        if entry.rule.matches(ext) && !matches.contains(&entry.format_id) {\n");
    out.push_str("            matches.push(entry.format_id);\n");
    out.push_str("        }\n");
    out.push_str("    }\n");
    out.push_str("    matches\n");
    out.push_str("}\n\n");

    out.push_str("/// Returns all extension rules associated with the given `FormatId`.\n");
    out.push_str("pub fn extension_rules_for_format(format_id: FormatId) -> Vec<ExtensionRule> {\n");
    out.push_str("    EXTENSION_REGISTRY\n");
    out.push_str("        .iter()\n");
    out.push_str("        .filter(|entry| entry.format_id == format_id)\n");
    out.push_str("        .map(|entry| entry.rule)\n");
    out.push_str("        .collect()\n");
    out.push_str("}\n\n");

    out.push_str("/// Returns the primary (first registered) extension for the given `FormatId`.\n");
    out.push_str("pub fn primary_extension_for_format(format_id: FormatId) -> Option<&'static str> {\n");
    out.push_str("    EXTENSION_REGISTRY\n");
    out.push_str("        .iter()\n");
    out.push_str("        .find(|entry| entry.format_id == format_id)\n");
    out.push_str("        .map(|entry| entry.rule.extension)\n");
    out.push_str("}\n");

    Ok(out)
}

/// Generates or updates `src/formats/utilities/extension_data.generated.rs` if contents changed.
///
/// # Errors
/// Returns an error if directory resolution, generation, or writing fails.
pub fn generate_extension_data_file(base_dir: &Path) -> Result<bool> {
    let repo_root = find_repository_root_from(base_dir)?;
    let formats_dir = repo_root
        .join("src")
        .join("formats")
        .join("dcdata")
        .join("data")
        .join("categories")
        .join("formats");
    ensure!(
        formats_dir.is_dir(),
        "Could not locate formats directory at {}",
        formats_dir.display()
    );

    let target_file = repo_root
        .join("src")
        .join("formats")
        .join("utilities")
        .join("extension_data.generated.rs");

    let code = generate_extension_data_code(&formats_dir)?;
    write_if_changed(&target_file, &code)
}
