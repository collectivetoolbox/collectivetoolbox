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

//! Codegen for Document Character (Dc) constants and `dc.generated.rs` from
//! category CSV data tables.

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

fn to_snake_case(s: &str) -> String {
    let mut res = String::new();
    let mut prev_is_upper = false;
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 && !prev_is_upper {
                res.push('_');
            }
            res.push(c.to_ascii_lowercase());
            prev_is_upper = true;
        } else {
            res.push(c);
            prev_is_upper = false;
        }
    }
    res
}

fn to_screaming_snake_case(s: &str) -> String {
    to_snake_case(s).to_ascii_uppercase()
}

/// Extracts the ident annotation from an aliases / annotations string, if present.
fn extract_ident_annotation(s: &str) -> Option<String> {
    if let Some(pos) = s.find("@ident(") {
        let rest = s.get(pos.saturating_add(7)..)?;
        if let Some(end) = rest.find(')') {
            let inner = rest.get(..end).unwrap_or("").trim().trim_matches('"');
            if !inner.is_empty() {
                return Some(inner.to_string());
            }
        }
    }
    None
}

/// Generates the contents of `dc.generated.rs` from category CSV files.
///
/// # Errors
/// Returns an error if reading directory or CSV parsing fails.
pub fn generate_dc_code(categories_dir: &Path) -> Result<String> {
    struct DcRowData {
        short_id: u32,
        ident: String,
        name: String,
        description: String,
    }

    let mut records: Vec<DcRowData> = Vec::new();
    let mut seen_idents = HashSet::new();

    if categories_dir.is_dir() {
        let mut entries = Vec::new();
        for entry in fs::read_dir(categories_dir)? {
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
                        let get = |idx: usize| -> String {
                            record.get(idx).unwrap_or("").trim().to_string()
                        };
                        let Ok(short_id) = get(1).parse::<u32>() else {
                            continue;
                        };
                        let aliases = get(8);
                        let Some(ident) = extract_ident_annotation(&aliases) else {
                            continue;
                        };
                        if seen_idents.contains(&ident) {
                            continue;
                        }
                        seen_idents.insert(ident.clone());
                        let name = get(2).trim_start_matches('!').trim().to_string();
                        let desc = get(9).trim().to_string();

                        records.push(DcRowData {
                            short_id,
                            ident,
                            name,
                            description: desc,
                        });
                    }
                }
            }
        }
    }

    records.sort_by_key(|r| r.short_id);

    let mut out = String::new();
    out.push_str(DEFAULT_AGPL_HEADER);
    out.push_str("\n\n");
    out.push_str("//! Constants and definitions for Document Character (Dc) data.\n");
    out.push_str("//! @generated by ctb-build-support::dc_codegen from category data tables.\n");
    out.push_str("//! Do not edit by hand.\n\n");
    out.push_str("#[expect(\n");
    out.push_str("    unused_imports,\n");
    out.push_str("    clippy::wildcard_imports,\n");
    out.push_str("    reason = \"Standard workspace module prelude\"\n");
    out.push_str(")]\n");
    out.push_str("use crate::utilities::*;\n\n");
    out.push_str("pub use ctb_storage_minimal::global_graph_layout::{\n");
    out.push_str("    SHORT_DC_REGION_END, SHORT_DC_REGION_START,\n");
    out.push_str("};\n\n");
    out.push_str("pub use crate::dc_char::DcChar;\n\n");

    for r in &records {
        let screaming = to_screaming_snake_case(&r.ident);
        let doc = if !r.description.is_empty() {
            &r.description
        } else {
            &r.name
        };
        out.push_str(&format!(
            "/// {} (Dc {}).\npub const DC_{screaming}: DcChar = DcChar::from_short({});\n",
            doc.replace('\n', " "),
            r.short_id,
            r.short_id
        ));
    }

    out.push_str(
        "\n/// Converts a short Document Character (Dc) ID to its long (Global Graph) ID.\n\
        #[must_use]\n\
        pub fn short_to_long_dc(short_id: u32) -> u128 {\n    \
            SHORT_DC_REGION_START.saturating_add(u128::from(short_id))\n\
        }\n\n\
        /// Converts a long (Global Graph) Document Character ID to its short Dc ID, if within short range.\n\
        #[must_use]\n\
        pub fn long_to_short_dc(dc_id: u128) -> Option<u32> {\n    \
            if (SHORT_DC_REGION_START..=SHORT_DC_REGION_END).contains(&dc_id) {\n        \
                u32::try_from(dc_id.saturating_sub(SHORT_DC_REGION_START)).ok()\n    \
            } else {\n        \
                None\n    \
            }\n\
        }\n",
    );

    Ok(out)
}

/// Generates or updates `src/formats/dcdata/dc.generated.rs` if contents changed.
///
/// # Errors
/// Returns an error if directory resolution, generation, or writing fails.
pub fn generate_dc_file(base_dir: &Path) -> Result<bool> {
    let repo_root = find_repository_root_from(base_dir)?;
    let categories_dir = repo_root
        .join("src")
        .join("formats")
        .join("dcdata")
        .join("data")
        .join("categories");
    ensure!(
        categories_dir.is_dir(),
        "Could not locate categories directory at {}",
        categories_dir.display()
    );

    let target_file = repo_root
        .join("src")
        .join("formats")
        .join("dcdata")
        .join("dc.generated.rs");

    let code = generate_dc_code(&categories_dir)?;
    write_if_changed(&target_file, &code)
}
