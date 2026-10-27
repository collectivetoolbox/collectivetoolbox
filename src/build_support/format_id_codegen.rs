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

//! Build-time code generator for `src/utilities/format_id.generated.rs` from
//! the system graph in `assets/data/graph/`.

use crate::global_graph_layout::FORMAT_REGION_START;
use crate::graph_index::BuildGraphIndex;
use crate::license_consts::DEFAULT_AGPL_HEADER;
use crate::shorthand::DcShorthand;
use anyhow::{Context, Result, bail, ensure};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
struct FormatRecord {
    dc_id: u128,
    short_id: u32,
    ident: String,
    label: String,
    category: String,
    nicknames: Vec<String>,
    title: Option<String>,
    implies_shorts: Vec<u32>,
}

/// Validates that an identifier is a valid non-empty Rust identifier.
fn is_valid_rust_ident(ident: &str) -> bool {
    if ident.is_empty() {
        return false;
    }
    let mut chars = ident.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_alphabetic() && first != '_' {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Converts a format category name (such as `base_alphabet` or `v:lineEndings`) into a
/// `PascalCase` Rust enum variant name for `FormatCategory`.
fn category_to_variant(category: &str) -> String {
    let base = category
        .strip_prefix("v.")
        .or_else(|| category.strip_prefix("v:"))
        .unwrap_or(category);
    let mut out = String::new();
    let mut capitalize_next = true;
    for ch in base.chars() {
        if ch == '_' || ch == '-' || ch == '.' || ch == ':' {
            capitalize_next = true;
        } else if capitalize_next {
            out.extend(ch.to_uppercase());
            capitalize_next = false;
        } else {
            out.push(ch);
        }
    }
    if out.is_empty() {
        "Other".to_string()
    } else {
        out
    }
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

/// Generates the contents of `format_id.generated.rs` from a [`BuildGraphIndex`].
///
/// # Errors
/// Returns an error if the graph is missing required ontology Dcs or contains duplicate/invalid formats.
#[expect(
    clippy::too_many_lines,
    reason = "Code generator for FormatId enum with all variants and mappings"
)]
pub fn generate_format_id_code_from_index(
    index: &BuildGraphIndex,
) -> Result<String> {
    let sh_name = index.require_ident("CanonicalName")?;
    let sh_nick = index.require_ident("Nickname")?;
    let sh_formal_alias = index.require_ident("FormalAlias")?;
    let sh_subtype = index.framing().is_subtype_of;
    let sh_begin_group = index.require_ident("BeginAssociativityGroup")?;
    let sh_end_group = index.require_ident("EndAssociativityGroup")?;
    let sh_type_union = index.require_ident("TypeUnion")?;

    let mut records = Vec::new();
    let mut seen_idents = HashSet::new();
    let mut seen_shorts = HashSet::new();
    let mut seen_dcs = HashSet::new();

    for short_id in index.all_format_short_ids()? {
        let fid_usize = usize::try_from(short_id).with_context(|| {
            format!("Format short ID {short_id} exceeds usize::MAX")
        })?;
        let subj = DcShorthand::Format(fid_usize);
        let dc_id = FORMAT_REGION_START.saturating_add(u128::from(short_id));

        let ident = index
            .ident_for_shorthand(subj)
            .with_context(|| {
                format!("Format f{short_id} has no @ident(\"...\") in system graph")
            })?
            .trim()
            .to_string();
        ensure!(
            is_valid_rust_ident(&ident),
            "Format f{short_id} has invalid Rust identifier '{ident}'"
        );

        // Reason for fallback: subjects returned by all_format_short_ids have a CanonicalName.
        let label = index
            .single_property_string(subj, sh_name)
            .unwrap_or_default()
            .trim()
            .to_string();
        ensure!(
            !label.is_empty(),
            "Format f{short_id} ({ident}) has empty CanonicalName"
        );

        // Reason for fallback: formats without an explicit script default to "other".
        let script = index.subject_script(subj).unwrap_or_default();
        let category = script
            .strip_prefix(".Formats:")
            .or_else(|| script.strip_prefix("Formats:"))
            .map_or_else(|| "other".to_string(), |s| s.trim().to_string());

        let nicknames: Vec<String> = index
            .multi_property_strings_preferred_first(subj, sh_nick)
            .into_iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let title = index
            .multi_property_strings_preferred_first(subj, sh_formal_alias)
            .into_iter()
            .map(|s| s.trim().to_string())
            .find(|s| !s.is_empty());

        let mut implies_shorts: Vec<u32> = Vec::new();
        for &obj in index.direct_objects(sh_subtype, subj) {
            match obj {
                DcShorthand::Format(target_fid) => {
                    if let Ok(target_sid) = u32::try_from(target_fid) {
                        if !implies_shorts.contains(&target_sid) {
                            implies_shorts.push(target_sid);
                        }
                    }
                }
                DcShorthand::Long(expr_node_id) => {
                    if let Some(tokens) = index.node_tokens(expr_node_id) {
                        let is_pure_format_conjunction =
                            !tokens.is_empty()
                                && tokens.iter().all(|&tok| {
                                    tok == sh_begin_group
                                        || tok == sh_end_group
                                        || tok == sh_type_union
                                        || matches!(tok, DcShorthand::Format(_))
                                });
                        if is_pure_format_conjunction {
                            for &tok in tokens {
                                if let DcShorthand::Format(target_fid) = tok {
                                    if let Ok(target_sid) =
                                        u32::try_from(target_fid)
                                    {
                                        if !implies_shorts.contains(&target_sid)
                                        {
                                            implies_shorts.push(target_sid);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                DcShorthand::Unicode(_)
                | DcShorthand::Short(_)
                | DcShorthand::Local(_) => {}
            }
        }

        if !seen_idents.insert(ident.clone()) {
            bail!("Duplicate FormatId variant '{ident}' on f{short_id}");
        }
        if !seen_shorts.insert(short_id) {
            bail!("Duplicate Format short ID {short_id}");
        }
        if !seen_dcs.insert(dc_id) {
            bail!("Duplicate Format Dc ID {dc_id}");
        }

        records.push(FormatRecord {
            dc_id,
            short_id,
            ident,
            label,
            category,
            nicknames,
            title,
            implies_shorts,
        });
    }

    records.sort_by_key(|r| r.short_id);

    // Collect distinct categories dynamically
    let mut category_variants: BTreeSet<String> = BTreeSet::new();
    let mut cat_to_variant: HashMap<String, String> = HashMap::new();
    for r in &records {
        let variant = category_to_variant(&r.category);
        cat_to_variant.insert(r.category.clone(), variant.clone());
        category_variants.insert(variant);
    }
    category_variants.insert("Other".to_string());

    let mut out = String::new();
    out.push_str(DEFAULT_AGPL_HEADER);
    out.push_str("\n\n");
    out.push_str(
        "//! @generated by ctb-formats-dcdata::updater from format category data tables.\n",
    );
    out.push_str("//! Do not edit by hand.\n\n");
    out.push_str("#[allow(\n");
    out.push_str("    unused_imports,\n");
    out.push_str("    clippy::wildcard_imports,\n");
    out.push_str("    reason = \"Standard workspace module prelude\"\n");
    out.push_str(")]\n");
    out.push_str(
        "use crate::dc_char::DcChar;\nuse serde::{Serialize, Deserialize};\n\n",
    );
    out.push_str(
        "/// High-level category of file formats for domain filtering and score boosting.\n",
    );
    out.push_str(
        "#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]\n",
    );
    out.push_str("pub enum FormatCategory {\n");
    for cat_var in &category_variants {
        out.push_str(&format!("    {cat_var},\n"));
    }
    out.push_str("}\n\n");

    out.push_str(
        "/// Standardized format identifier enum derived from formats category tables.\n",
    );
    out.push_str("#[expect(\n");
    out.push_str("    non_camel_case_types,\n");
    out.push_str("    reason = \"Language and format variants use underscores to reflect canonical format abbreviations\"\n");
    out.push_str(")]\n");
    out.push_str(
        "#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]\n",
    );
    out.push_str("pub enum FormatId {\n");

    for r in &records {
        let escaped_label = r.label.replace('\n', " ").replace('"', "\\\"");
        out.push_str(&format!(
            "    /// {} (Short {}, Category: {})\n",
            escaped_label, r.short_id, r.category
        ));
        out.push_str(&format!("    {},\n", r.ident));
    }
    if !seen_idents.contains("Unknown") {
        out.push_str("    /// Unrecognized format.\n");
        out.push_str("    Unknown,\n");
    }
    out.push_str("}\n\n");

    out.push_str("impl FormatId {\n");
    out.push_str(
        "    /// Returns the primary format category for this format ID.\n",
    );
    out.push_str("    #[must_use]\n");
    out.push_str("    pub fn category(&self) -> FormatCategory {\n");
    out.push_str("        match self {\n");

    let mut grouped_by_cat: BTreeMap<String, Vec<&str>> = BTreeMap::new();
    for r in &records {
        if let Some(var) = cat_to_variant.get(&r.category) {
            grouped_by_cat
                .entry(var.clone())
                .or_default()
                .push(&r.ident);
        }
    }
    for (var, idents) in grouped_by_cat {
        out.push_str("            ");
        out.push_str(
            &idents
                .iter()
                .map(|s| format!("Self::{s}"))
                .collect::<Vec<_>>()
                .join("\n            | "),
        );
        out.push_str(&format!(" => FormatCategory::{var},\n"));
    }
    out.push_str("            _ => FormatCategory::Other,\n");
    out.push_str("        }\n");
    out.push_str("    }\n\n");

    out.push_str(
        "    /// Looks up a `FormatId` variant from its identifier name or nickname.\n",
    );
    out.push_str("    #[must_use]\n");
    out.push_str("    pub fn from_ident(ident: &str) -> Option<Self> {\n");
    out.push_str("        let trimmed = ident.trim();\n");
    out.push_str("        match trimmed.to_ascii_lowercase().as_str() {\n");
    for r in &records {
        let lower = r.ident.to_ascii_lowercase();
        let mut patterns = vec![format!("\"{lower}\"")];
        let snake = to_snake_case(&r.ident);
        if snake != lower && !patterns.contains(&format!("\"{snake}\"")) {
            patterns.push(format!("\"{snake}\""));
        }
        for nick in &r.nicknames {
            let nick_lower = nick.trim().to_ascii_lowercase();
            if !nick_lower.is_empty()
                && !patterns.contains(&format!("\"{nick_lower}\""))
            {
                patterns.push(format!("\"{nick_lower}\""));
            }
        }
        out.push_str(&format!(
            "            {} => Some(Self::{}),\n",
            patterns.join(" | "),
            r.ident
        ));
    }
    if !seen_idents.contains("Unknown") {
        out.push_str("            \"unknown\" => Some(Self::Unknown),\n");
    }
    out.push_str("            _ => None,\n");
    out.push_str("        }\n");
    out.push_str("    }\n\n");

    out.push_str(
        "    /// Returns the canonical Rust identifier string for this format.\n",
    );
    out.push_str("    #[must_use]\n");
    out.push_str("    pub const fn ident(&self) -> &'static str {\n");
    out.push_str("        match self {\n");
    for r in &records {
        out.push_str(&format!(
            "            Self::{} => \"{}\",\n",
            r.ident, r.ident
        ));
    }
    if !seen_idents.contains("Unknown") {
        out.push_str("            Self::Unknown => \"Unknown\",\n");
    }
    out.push_str("        }\n");
    out.push_str("    }\n\n");

    out.push_str(
        "    /// Returns the canonical human-readable title (\"true name\") for this format if defined.\n",
    );
    out.push_str("    #[must_use]\n");
    out.push_str("    pub const fn title(&self) -> Option<&'static str> {\n");
    out.push_str("        match self {\n");
    for r in &records {
        if let Some(t) = &r.title {
            let escaped = t.replace('\\', "\\\\").replace('"', "\\\"");
            out.push_str(&format!(
                "            Self::{} => Some(\"{}\"),\n",
                r.ident, escaped
            ));
        }
    }
    out.push_str("            _ => None,\n");
    out.push_str("        }\n");
    out.push_str("    }\n\n");

    let short_to_ident: HashMap<u32, String> = records
        .iter()
        .map(|r| (r.short_id, r.ident.clone()))
        .collect();
    let direct_implies: HashMap<u32, Vec<u32>> = records
        .iter()
        .map(|r| (r.short_id, r.implies_shorts.clone()))
        .collect();

    let mut transitive_implies: HashMap<u32, Vec<String>> = HashMap::new();
    for r in &records {
        let mut seen = HashSet::new();
        let mut queue = r.implies_shorts.clone();
        let mut implied_idents = Vec::new();
        while let Some(short_id) = queue.pop() {
            if short_id != r.short_id && seen.insert(short_id) {
                if let Some(ident) = short_to_ident.get(&short_id) {
                    implied_idents.push(ident.clone());
                }
                if let Some(next_shorts) = direct_implies.get(&short_id) {
                    for &next in next_shorts {
                        if !seen.contains(&next) {
                            queue.push(next);
                        }
                    }
                }
            }
        }
        implied_idents.sort();
        implied_idents.dedup();
        if !implied_idents.is_empty() {
            transitive_implies.insert(r.short_id, implied_idents);
        }
    }

    out.push_str(
        "    /// Returns the slice of formats directly and transitively implied by this format,\n",
    );
    out.push_str(
        "    /// as declared by `@implies(...)` directives in format tables.\n",
    );
    out.push_str("    #[must_use]\n");
    out.push_str("    pub const fn implies(&self) -> &'static [Self] {\n");
    out.push_str("        match self {\n");
    for r in &records {
        if let Some(implied) = transitive_implies.get(&r.short_id) {
            if !implied.is_empty() {
                out.push_str(&format!("            Self::{} => &[", r.ident));
                for (i, id) in implied.iter().enumerate() {
                    if i > 0 {
                        out.push_str(", ");
                    }
                    out.push_str(&format!("Self::{id}"));
                }
                out.push_str("],\n");
            }
        }
    }
    out.push_str("            _ => &[],\n");
    out.push_str("        }\n");
    out.push_str("    }\n\n");

    // NOTE: Do NOT add `short_id()` or `from_short_id()` integer methods here!
    // There is no such thing as a "short format ID". Bare integers are confusing
    // because they look identical to short Dc IDs. Format IDs must always use
    // the `FormatId` enum, `FormatId::shorthand()` ("f<N>"), or `FormatId::ident()`.


    out.push_str(
        "    /// Returns the format shorthand string (e.g. \"f0\", \"f405\").\n",
    );
    out.push_str("    #[must_use]\n");
    out.push_str("    pub const fn shorthand(&self) -> &'static str {\n");
    out.push_str("        match self {\n");
    for r in &records {
        out.push_str(&format!(
            "            Self::{} => \"f{}\",\n",
            r.ident, r.short_id
        ));
    }
    if !seen_idents.contains("Unknown") {
        out.push_str("            Self::Unknown => \"unknown\",\n");
    }
    out.push_str("        }\n");
    out.push_str("    }\n\n");

    out.push_str(
        "    /// Looks up a `FormatId` from its format shorthand string (e.g. \"f405\").\n",
    );
    out.push_str("    #[must_use]\n");
    out.push_str("    pub fn from_shorthand(shorthand: &str) -> Option<Self> {\n");
    out.push_str("        let trimmed = shorthand.trim();\n");
    out.push_str("        match trimmed {\n");
    for r in &records {
        out.push_str(&format!(
            "            \"f{}\" => Some(Self::{}),\n",
            r.short_id, r.ident
        ));
    }
    out.push_str("            _ => None,\n");
    out.push_str("        }\n");
    out.push_str("    }\n\n");

    out.push_str(
        "    /// Returns the Global Document Character ID if known.\n",
    );
    out.push_str("    #[must_use]\n");
    out.push_str("    pub const fn dc_id(&self) -> Option<u128> {\n");
    out.push_str("        match self {\n");
    for r in &records {
        out.push_str(&format!(
            "            Self::{} => Some({}_u128),\n",
            r.ident, r.dc_id
        ));
    }
    if !seen_idents.contains("Unknown") {
        out.push_str("            Self::Unknown => None,\n");
    }
    out.push_str("        }\n");
    out.push_str("    }\n\n");

    out.push_str(
        "    /// Returns the format character as a `DcChar`, if known.\n",
    );
    out.push_str("    #[must_use]\n");
    out.push_str("    pub const fn dc_char(&self) -> Option<DcChar> {\n");
    out.push_str("        match self {\n");
    out.push_str("            Self::Unknown => None,\n");
    out.push_str("            _ => if let Some(dc) = self.dc_id() {\n");
    out.push_str("                Some(DcChar::from_u128(dc))\n");
    out.push_str("            } else {\n");
    out.push_str("                None\n");
    out.push_str("            }\n");
    out.push_str("        }\n");
    out.push_str("    }\n\n");

    out.push_str(
        "    /// Looks up a `FormatId` from its Global Document Character ID.\n",
    );
    out.push_str("    #[must_use]\n");
    out.push_str("    pub const fn from_dc_id(id: u128) -> Option<Self> {\n");
    out.push_str("        match id {\n");
    for r in &records {
        out.push_str(&format!(
            "            {}_u128 => Some(Self::{}),\n",
            r.dc_id, r.ident
        ));
    }
    out.push_str("            _ => None,\n");
    out.push_str("        }\n");
    out.push_str("    }\n");
    out.push_str("}\n\n");

    // Generate individual format DcChar constants
    for r in &records {
        let screaming = to_screaming_snake_case(&r.ident);
        let escaped_label = r.label.replace('\n', " ").replace('"', "\\\"");
        out.push_str(&format!(
            "/// DcChar constant for Format `{}` (Short f{}, Category: {}): {}\n",
            r.ident, r.short_id, r.category, escaped_label
        ));
        out.push_str(&format!(
            "pub const DC_{screaming}: DcChar = DcChar::from_format({});\n",
            r.short_id
        ));
    }

    Ok(out)
}

/// Generates the contents of `format_id.generated.rs` from the system graph in `graph_dir`.
///
/// # Errors
/// Returns an error if `graph_dir` cannot be read or contains invalid data.
pub fn generate_format_id_code(graph_dir: &Path) -> Result<String> {
    let index = BuildGraphIndex::from_graph_dir(graph_dir)?;
    generate_format_id_code_from_index(&index)
}

/// Generates `src/utilities/format_id.generated.rs` if its contents have changed.
///
/// # Errors
/// Returns an error if reading the system graph or writing the generated file fails.
pub fn generate_format_id_file(base_dir: &Path) -> Result<bool> {
    let repo_root = crate::find_repository_root_from(base_dir)?;
    let graph_dir = repo_root.join("assets").join("data").join("graph");
    let out_file = repo_root
        .join("src")
        .join("utilities")
        .join("format_id.generated.rs");

    ensure!(
        graph_dir.is_dir(),
        "Could not locate system graph directory at {}",
        graph_dir.display()
    );

    let code = generate_format_id_code(&graph_dir)?;

    if let Ok(existing) = fs::read_to_string(&out_file) {
        if existing == code {
            return Ok(false);
        }
    }

    fs::write(&out_file, code).with_context(|| {
        format!("Failed to write {}", out_file.display())
    })?;
    Ok(true)
}
