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
use std::collections::{BTreeSet, HashSet};
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

/// Extracts all `@roleDef("...")` composite role names from an aliases / annotations string.
fn extract_composite_role_defs(s: &str) -> Vec<String> {
    let mut roles = Vec::new();
    let mut rest = s;
    while let Some(pos) = rest.find("@roleDef(") {
        let after = rest.get(pos.saturating_add(9)..).unwrap_or("");
        if let Some(end) = after.find(')') {
            let inner = after.get(..end).unwrap_or("").trim().trim_matches('"');
            if !inner.is_empty() {
                roles.push(inner.to_string());
            }
            rest = after.get(end.saturating_add(1)..).unwrap_or("");
        } else {
            break;
        }
    }
    roles
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

/// Formats a set of short Dc IDs as a compact Rust match pattern, collapsing
/// contiguous integer spans into `start..=end` ranges.
fn format_short_id_pattern(ids: &BTreeSet<u32>) -> String {
    let mut parts = Vec::new();
    let id_list: Vec<u32> = ids.iter().copied().collect();
    let mut i = 0;
    while i < id_list.len() {
        let Some(&start) = id_list.get(i) else {
            break;
        };
        let mut end = start;
        while let Some(&next) = id_list.get(i.saturating_add(1)) {
            if next == end.saturating_add(1) {
                end = next;
                i = i.saturating_add(1);
            } else {
                break;
            }
        }
        if end > start.saturating_add(1) {
            parts.push(format!("{start}..={end}"));
        } else if end == start.saturating_add(1) {
            parts.push(format!("{start} | {end}"));
        } else {
            parts.push(format!("{start}"));
        }
        i = i.saturating_add(1);
    }
    parts.join(" | ")
}

/// Generates the contents of `role.generated.rs` from category CSV files.
///
/// # Errors
/// Returns an error if reading directory or CSV parsing fails.
pub fn generate_role_code(categories_dir: &Path) -> Result<String> {
    struct RoleRowData {
        short_id: u32,
        ident: String,
        name: String,
        description: String,
    }

    let mut records: Vec<RoleRowData> = Vec::new();
    let mut seen_idents = HashSet::new();

    let mut base64_digits: BTreeSet<u32> = BTreeSet::new();
    let mut base64_start_opt: Option<u32> = None;
    let mut base64_end_opt: Option<u32> = None;
    let mut base64_padding_opt: Option<u32> = None;
    let mut type_tokens: BTreeSet<u32> = BTreeSet::new();
    let mut primitive_atoms: BTreeSet<u32> = BTreeSet::new();
    let mut domain_atoms: BTreeSet<u32> = BTreeSet::new();
    let mut evaluable_starters: BTreeSet<u32> = BTreeSet::new();
    let mut structural_seps: BTreeSet<u32> = BTreeSet::new();

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
                        let name_raw = get(2);
                        let name = name_raw.trim_start_matches('!').trim().to_string();
                        let aliases = get(8);
                        let desc = get(9).trim().to_string();
                        let ident_opt = extract_ident_annotation(&aliases);

                        if aliases.contains("@roleDef()") {
                            if let Some(ref ident) = ident_opt {
                                if seen_idents.insert(ident.clone()) {
                                    records.push(RoleRowData {
                                        short_id,
                                        ident: ident.clone(),
                                        name: name.clone(),
                                        description: desc,
                                    });
                                }
                            }
                        }

                        match ident_opt.as_deref() {
                            Some("Base64Start") => base64_start_opt = Some(short_id),
                            Some("Base64End") => base64_end_opt = Some(short_id),
                            Some("Base64Padding") => base64_padding_opt = Some(short_id),
                            _ => {}
                        }

                        for role_name in extract_composite_role_defs(&aliases) {
                            match role_name.as_str() {
                                "Base64Digit" => {
                                    base64_digits.insert(short_id);
                                }
                                "TypeToken" => {
                                    type_tokens.insert(short_id);
                                }
                                "PrimitiveValueAtom" => {
                                    primitive_atoms.insert(short_id);
                                }
                                "DomainValueAtom" => {
                                    domain_atoms.insert(short_id);
                                }
                                "EvaluableExpressionStarter" => {
                                    evaluable_starters.insert(short_id);
                                }
                                "StructuralSeparator" => {
                                    structural_seps.insert(short_id);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }

    if let (Some(start), Some(end)) = (base64_start_opt, base64_end_opt) {
        for id in start..=end {
            base64_digits.insert(id);
        }
    }

    records.sort_by_key(|r| r.short_id);

    let base64_digits_pat = format_short_id_pattern(&base64_digits);
    let type_tokens_pat = format_short_id_pattern(&type_tokens);
    let primitive_atoms_pat = format_short_id_pattern(&primitive_atoms);
    let domain_atoms_pat = format_short_id_pattern(&domain_atoms);
    let evaluable_starters_pat = format_short_id_pattern(&evaluable_starters);
    let structural_seps_pat = format_short_id_pattern(&structural_seps);

    let base64_min = base64_digits.first().copied().unwrap_or(127);
    let base64_max = base64_digits.last().copied().unwrap_or(190);
    let base64_padding = base64_padding_opt.unwrap_or(195);

    let mut out = String::new();
    out.push_str(DEFAULT_AGPL_HEADER);
    out.push_str("\n\n");
    out.push_str("//! Document Character (Dc) syntactic and semantic role classifications.\n");
    out.push_str("//!\n");
    out.push_str("//! @generated by ctb-build-support::dc_codegen from category data tables.\n");
    out.push_str("//! Do not edit by hand.\n\n");
    out.push_str("#[allow(\n");
    out.push_str("    unused_imports,\n");
    out.push_str("    clippy::wildcard_imports,\n");
    out.push_str("    reason = \"Standard workspace module prelude\"\n");
    out.push_str(")]\n");
    out.push_str("use crate::utilities::*;\n\n");
    out.push_str("use ctb_utilities::dc_char::{FORMAT_REGION_END, FORMAT_REGION_START};\n");
    out.push_str("use serde::{Deserialize, Serialize};\n\n");
    out.push_str("/// Classification of syntactic framing and semantic roles for Document\n");
    out.push_str("/// Characters.\n");
    out.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]\n");
    out.push_str("pub enum DcRole {\n");

    for r in &records {
        let doc = if !r.description.is_empty() {
            &r.description
        } else {
            &r.name
        };
        out.push_str(&format!(
            "    /// {} (`{}`).\n    {},\n",
            doc.replace('\n', " "),
            r.short_id,
            r.ident
        ));
    }

    out.push_str(
        "    // Composite semantic roles\n\
    /// Integer format indicator (`FormatId::Integer` `f199`).\n\
    IntegerFormatIndicator,\n\
    /// Base64 encapsulation digit (`127..=190`).\n\
    Base64Digit,\n\
    /// General type token (script `.EL Types` or `.Formats`).\n\
    TypeToken,\n\
    /// Primitive value atom (absent optional or boolean literal).\n\
    PrimitiveValueAtom,\n\
    /// Domain-specific value atom.\n\
    DomainValueAtom,\n\
    /// Starter token for an evaluable Dc expression.\n\
    EvaluableExpressionStarter,\n\
    /// Structural separator (list element separator or whitespace delimiter).\n\
    StructuralSeparator,\n\
}\n\n",
    );

    out.push_str("impl DcRole {\n");

    out.push_str(
        "    /// Returns the defining short Dc ID for this role, if uniquely defined by a single Document Character.\n\
        #[must_use]\n\
        pub fn defining_short(self) -> Option<u32> {\n\
            match self {\n",
    );
    for r in &records {
        out.push_str(&format!(
            "            Self::{} => Some({}),\n",
            r.ident, r.short_id
        ));
    }
    out.push_str(
        "            _ => None,\n\
            }\n\
        }\n\n",
    );

    out.push_str(
        "    /// Returns the defining `DcChar` for this role, if uniquely defined by a single Document Character.\n\
        #[must_use]\n\
        pub fn defining_dc(self) -> Option<DcChar> {\n\
            self.defining_short().map(DcChar::from_short)\n\
        }\n\n",
    );

    out.push_str(
        "    /// Returns true if the given canonical token (in global graph `u128` layout)\n\
        /// directly matches this role.\n\
        #[must_use]\n\
        pub fn matches_canonical(self, token: u128) -> bool {\n\
            let short = DcChar::from_u128(token).to_short().ok();\n\
            match self {\n",
    );
    for r in &records {
        out.push_str(&format!(
            "            Self::{} => short == Some({}),\n",
            r.ident, r.short_id
        ));
    }
    out.push_str(&format!(
        "            Self::IntegerFormatIndicator => {{\n\
                (FORMAT_REGION_START..=FORMAT_REGION_END).contains(&token)\n\
                    && ctb_utilities::format_id::FormatId::from_dc_id(token)\n\
                        == Some(ctb_utilities::format_id::FormatId::Integer)\n\
            }}\n\
            Self::Base64Digit => matches!(short, Some({base64_digits_pat})),\n\
            Self::TypeToken => {{\n\
                (FORMAT_REGION_START..=FORMAT_REGION_END).contains(&token)\n\
                    || matches!(short, Some({type_tokens_pat}))\n\
            }}\n\
            Self::PrimitiveValueAtom => matches!(short, Some({primitive_atoms_pat})),\n\
            Self::DomainValueAtom => matches!(short, Some({domain_atoms_pat})),\n\
            Self::EvaluableExpressionStarter => {{\n\
                matches!(short, Some({evaluable_starters_pat}))\n\
            }}\n\
            Self::StructuralSeparator => {{\n\
                matches!(short, Some({structural_seps_pat}))\n\
            }}\n\
        }}\n\
    }}\n\n"
    ));

    out.push_str(&format!(
        "    /// Returns the Base64 numeral digit value (`0..=63`) or padding sentinel\n\
        /// (`64`) if `token` represents a Base64 encapsulation character.\n\
        #[must_use]\n\
        pub fn base64_digit_value(token: u128) -> Option<u8> {{\n\
            let short = DcChar::from_u128(token).to_short().ok()?;\n\
            if short == {base64_padding} {{\n\
                Some(64)\n\
            }} else if ({base64_min}..={base64_max}).contains(&short) {{\n\
                u8::try_from(short.saturating_sub({base64_min})).ok()\n\
            }} else {{\n\
                None\n\
            }}\n\
        }}\n\
}}\n"
    ));

    Ok(out)
}

/// Generates or updates `src/formats/dcdata/syntax/role.generated.rs` if contents changed.
///
/// # Errors
/// Returns an error if directory resolution, generation, or writing fails.
pub fn generate_role_file(base_dir: &Path) -> Result<bool> {
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
        .join("syntax")
        .join("role.generated.rs");

    let code = generate_role_code(&categories_dir)?;
    write_if_changed(&target_file, &code)
}
