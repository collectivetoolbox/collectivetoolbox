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

//! Codegen for `FormatId` enum and `format_id.generated.rs` from format
//! category CSV data tables.

use anyhow::{Context, Result, bail, ensure};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::Path;

use crate::find_repository_root_from;
use crate::license_consts::DEFAULT_AGPL_HEADER;

pub const FORMAT_REGION_START: u128 = 2_228_224;
pub const MAX_FORMAT_ID: usize = 1_114_111;

/// Parses a format shorthand string (e.g. `"f405"` or `"405"`) into its integer ID.
///
/// # Errors
/// Returns an error if the shorthand is invalid or exceeds `MAX_FORMAT_ID`.
pub fn parse_format_shorthand(s: &str) -> Result<usize> {
    let trimmed = s.trim();
    if let Some(rest) = trimmed.strip_prefix('f') {
        ensure!(
            !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()),
            "Invalid format ID '{trimmed}': prefix 'f' must be followed by decimal digits"
        );
        let id = rest
            .parse::<usize>()
            .map_err(|e| anyhow::anyhow!("Invalid format ID integer '{trimmed}': {e}"))?;
        ensure!(
            id <= MAX_FORMAT_ID,
            "Format ID {id} exceeds maximum {MAX_FORMAT_ID}"
        );
        Ok(id)
    } else if !trimmed.is_empty() && trimmed.chars().all(|c| c.is_ascii_digit()) {
        let id = trimmed
            .parse::<usize>()
            .map_err(|e| anyhow::anyhow!("Invalid format ID integer '{trimmed}': {e}"))?;
        ensure!(
            id <= MAX_FORMAT_ID,
            "Format ID {id} exceeds maximum {MAX_FORMAT_ID}"
        );
        Ok(id)
    } else {
        bail!("Invalid format shorthand '{s}'");
    }
}

/// Splits a comma-separated string while respecting parentheses, brackets, and quotes.
#[must_use]
pub fn split_comma_separated_items(raw: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut depth_paren = 0usize;
    let mut depth_bracket = 0usize;
    let mut in_quotes = false;
    let mut is_escaped = false;
    let mut current = String::new();

    for ch in raw.chars() {
        if is_escaped {
            is_escaped = false;
            current.push(ch);
            continue;
        }
        if ch == '\\' {
            is_escaped = true;
            current.push(ch);
            continue;
        }
        if ch == '"' {
            in_quotes = !in_quotes;
            current.push(ch);
            continue;
        }
        if in_quotes {
            current.push(ch);
            continue;
        }

        match ch {
            '(' => {
                depth_paren = depth_paren.saturating_add(1);
                current.push(ch);
            }
            ')' => {
                depth_paren = depth_paren.saturating_sub(1);
                current.push(ch);
            }
            '[' => {
                depth_bracket = depth_bracket.saturating_add(1);
                current.push(ch);
            }
            ']' => {
                depth_bracket = depth_bracket.saturating_sub(1);
                current.push(ch);
            }
            ',' if depth_paren == 0 && depth_bracket == 0 => {
                let trimmed = current.trim();
                if !trimmed.is_empty() {
                    items.push(trimmed.to_string());
                }
                current.clear();
            }
            _ => {
                current.push(ch);
            }
        }
    }
    let trimmed = current.trim();
    if !trimmed.is_empty() {
        items.push(trimmed.to_string());
    }
    items
}

/// Unescapes a double-quoted string payload inside a directive.
fn unescape_quoted_directive_payload(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    let content = trimmed.strip_prefix('"')?.strip_suffix('"')?;
    let mut res = String::with_capacity(content.len());
    let mut chars = content.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(&next_c) = chars.peek() {
                if next_c == '"' || next_c == '\\' {
                    res.push(next_c);
                    let _ = chars.next();
                    continue;
                }
            }
            res.push('\\');
        } else {
            res.push(c);
        }
    }
    Some(res)
}

/// Extracts title and directly implied format IDs from a raw Base / Related column cell.
fn extract_title_and_implies(raw_base: &str) -> (Option<String>, Vec<usize>) {
    if raw_base.is_empty() {
        return (None, Vec::new());
    }
    let items = split_comma_separated_items(raw_base);
    let mut title = None;
    let mut shorts = Vec::new();

    for item in items {
        let item_trimmed = item.trim();
        if let Some(inner) = item_trimmed.strip_prefix("@title(") {
            if let Some(stripped) = inner.strip_suffix(')') {
                if let Some(unescaped) = unescape_quoted_directive_payload(stripped) {
                    if !unescaped.trim().is_empty() && title.is_none() {
                        title = Some(unescaped);
                    }
                }
            }
        } else if let Some(inner) = item_trimmed.strip_prefix("@implies(") {
            if let Some(stripped) = inner.strip_suffix(')') {
                for conj in stripped.split('&') {
                    let trimmed = conj.trim().trim_start_matches('(').trim_end_matches(')');
                    if !trimmed.contains('|') {
                        if let Some(num_str) = trimmed.strip_prefix('f') {
                            if let Ok(id) = num_str.parse::<usize>() {
                                if !shorts.contains(&id) {
                                    shorts.push(id);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    (title, shorts)
}

/// Reads a CSV file returning all parsed rows.
///
/// # Errors
/// Returns an error if reading or CSV parsing fails.
pub fn read_csv_rows(path: &Path) -> Result<Vec<Vec<String>>> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read CSV at {}", path.display()))?;

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(content.as_bytes());

    let mut rows = Vec::new();
    for result in rdr.records() {
        let record = result.with_context(|| {
            format!("Failed to parse record in {}", path.display())
        })?;
        rows.push(
            record
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<String>>(),
        );
    }

    if rows.is_empty() {
        return Ok(Vec::new());
    }

    // Drop header row
    rows.remove(0);
    Ok(rows)
}

/// Checks whether an entire row consists solely of empty cells.
#[must_use]
pub fn is_empty_row(row: &[String]) -> bool {
    row.iter().all(|cell| cell.trim().is_empty())
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

fn category_to_variant_name(cat: &str) -> String {
    let s = if let Some(stripped) = cat.strip_prefix("v:") {
        stripped
    } else {
        cat
    };
    let mut result = String::new();
    let mut capitalize_next = true;
    for ch in s.chars() {
        if ch == '_' || ch == '-' || ch == ':' {
            capitalize_next = true;
        } else if capitalize_next {
            result.extend(ch.to_uppercase());
            capitalize_next = false;
        } else {
            result.push(ch);
        }
    }
    result
}

fn write_if_changed(path: &Path, content: &str) -> Result<bool> {
    if path.exists() {
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

/// Generates the contents of `format_id.generated.rs` from format category CSV files.
///
/// # Errors
/// Returns an error if reading files or CSV parsing fails.
#[expect(
    clippy::too_many_lines,
    reason = "Code generator for FormatId enum with all variants and mappings"
)]
pub fn generate_format_id_code(formats_dir: &Path) -> Result<String> {
    struct FormatRowData {
        dc_id: u128,
        short_id: usize,
        ident: String,
        label: String,
        category: String,
        nicknames: Vec<String>,
        title: Option<String>,
        implies_shorts: Vec<usize>,
    }

    let mut records: Vec<FormatRowData> = Vec::new();
    let mut seen_idents = HashSet::new();

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
                    let rows = read_csv_rows(&path)?;
                    for row in rows {
                        if is_empty_row(&row) {
                            continue;
                        }
                        // Reason for fallback: cells beyond the row length default to empty string
                        let get = |idx: usize| -> String {
                            row.get(idx).cloned().unwrap_or_default()
                        };
                        let raw_ident = get(2);
                        let clean_ident = raw_ident.trim_start_matches('!').trim().to_string();
                        if clean_ident.is_empty() {
                            continue;
                        }
                        if seen_idents.contains(&clean_ident) {
                            continue;
                        }
                        seen_idents.insert(clean_ident.clone());

                        let short_id = if let Ok(s_id) = parse_format_shorthand(&get(1)) {
                            s_id
                        } else if let Ok(dc_id) = get(0).parse::<u128>() {
                            // Reason for fallback: out-of-range short format id conversion defaults to 0 placeholder
                            usize::try_from(dc_id.saturating_sub(FORMAT_REGION_START)).unwrap_or(0)
                        } else {
                            0
                        };

                        let dc_id = if let Ok(dc) = get(0).parse::<u128>() {
                            dc
                        } else {
                            // Reason for fallback: short format id conversion to u128 defaults to 0 on conversion failure
                            FORMAT_REGION_START.saturating_add(u128::try_from(short_id).unwrap_or(0))
                        };

                        let raw_label = get(3);
                        let label = raw_label.trim_start_matches('!').trim().to_string();
                        let category = get(4).trim().to_string();

                        let raw_nicknames = get(10);
                        let nicknames: Vec<String> = if raw_nicknames.is_empty() {
                            Vec::new()
                        } else {
                            split_comma_separated_items(&raw_nicknames)
                                .into_iter()
                                .filter(|s| !s.is_empty())
                                .collect()
                        };

                        let raw_base = get(5);
                        let (title, implies_shorts) = extract_title_and_implies(&raw_base);

                        records.push(FormatRowData {
                            dc_id,
                            short_id,
                            ident: clean_ident,
                            label,
                            category,
                            nicknames,
                            title,
                            implies_shorts,
                        });
                    }
                }
            }
        }
    }

    records.sort_by_key(|r| r.short_id);

    // Collect distinct categories dynamically
    let mut category_variants: BTreeSet<String> = BTreeSet::new();
    let mut cat_to_variant: HashMap<String, String> = HashMap::new();
    for r in &records {
        let variant = category_to_variant_name(&r.category);
        cat_to_variant.insert(r.category.clone(), variant.clone());
        category_variants.insert(variant);
    }
    category_variants.insert("Other".to_string());

    let mut out = String::new();
    out.push_str(DEFAULT_AGPL_HEADER);
    out.push_str("\n\n");
    out.push_str("//! @generated by ctb-formats-dcdata::updater from format category data tables.\n");
    out.push_str("//! Do not edit by hand.\n\n");
    out.push_str("#[allow(\n");
    out.push_str("    unused_imports,\n");
    out.push_str("    clippy::wildcard_imports,\n");
    out.push_str("    reason = \"Standard workspace module prelude\"\n");
    out.push_str(")]\n");
    out.push_str("use crate::dc_char::DcChar;\nuse serde::{Serialize, Deserialize};\n\n");
    out.push_str("/// High-level category of file formats for domain filtering and score boosting.\n");
    out.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]\n");
    out.push_str("pub enum FormatCategory {\n");
    for cat_var in &category_variants {
        out.push_str(&format!("    {cat_var},\n"));
    }
    out.push_str("}\n\n");

    out.push_str("/// Standardized format identifier enum derived from formats category tables.\n");
    out.push_str("#[expect(\n");
    out.push_str("    non_camel_case_types,\n");
    out.push_str("    reason = \"Language and format variants use underscores to reflect canonical format abbreviations\"\n");
    out.push_str(")]\n");
    out.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]\n");
    out.push_str("pub enum FormatId {\n");

    for r in &records {
        let escaped_label = r.label.replace('\n', " ").replace('"', "\\\"");
        out.push_str(&format!("    /// {} (Short {}, Category: {})\n", escaped_label, r.short_id, r.category));
        out.push_str(&format!("    {},\n", r.ident));
    }
    if !seen_idents.contains("Unknown") {
        out.push_str("    /// Unrecognized format.\n");
        out.push_str("    Unknown,\n");
    }
    out.push_str("}\n\n");

    out.push_str("impl FormatId {\n");
    out.push_str("    /// Returns the primary format category for this format ID.\n");
    out.push_str("    #[must_use]\n");
    out.push_str("    pub fn category(&self) -> FormatCategory {\n");
    out.push_str("        match self {\n");

    let mut grouped_by_cat: BTreeMap<String, Vec<&str>> = BTreeMap::new();
    for r in &records {
        if let Some(var) = cat_to_variant.get(&r.category) {
            grouped_by_cat.entry(var.clone()).or_default().push(&r.ident);
        }
    }
    for (var, idents) in grouped_by_cat {
        out.push_str("            ");
        out.push_str(&idents.iter().map(|s| format!("Self::{s}")).collect::<Vec<_>>().join("\n            | "));
        out.push_str(&format!(" => FormatCategory::{var},\n"));
    }
    out.push_str("            _ => FormatCategory::Other,\n");
    out.push_str("        }\n");
    out.push_str("    }\n\n");

    out.push_str("    /// Looks up a `FormatId` variant from its identifier name or nickname.\n");
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
            if !nick_lower.is_empty() && !patterns.contains(&format!("\"{nick_lower}\"")) {
                patterns.push(format!("\"{nick_lower}\""));
            }
        }
        out.push_str(&format!("            {} => Some(Self::{}),\n", patterns.join(" | "), r.ident));
    }
    if !seen_idents.contains("Unknown") {
        out.push_str("            \"unknown\" => Some(Self::Unknown),\n");
    }
    out.push_str("            _ => None,\n");
    out.push_str("        }\n");
    out.push_str("    }\n\n");

    out.push_str("    /// Returns the canonical Rust identifier string for this format.\n");
    out.push_str("    #[must_use]\n");
    out.push_str("    pub const fn ident(&self) -> &'static str {\n");
    out.push_str("        match self {\n");
    for r in &records {
        out.push_str(&format!("            Self::{} => \"{}\",\n", r.ident, r.ident));
    }
    if !seen_idents.contains("Unknown") {
        out.push_str("            Self::Unknown => \"Unknown\",\n");
    }
    out.push_str("        }\n");
    out.push_str("    }\n\n");

    out.push_str("    /// Returns the canonical human-readable title (\"true name\") for this format if defined.\n");
    out.push_str("    #[must_use]\n");
    out.push_str("    pub const fn title(&self) -> Option<&'static str> {\n");
    out.push_str("        match self {\n");
    for r in &records {
        if let Some(t) = &r.title {
            let escaped = t.replace('\\', "\\\\").replace('"', "\\\"");
            out.push_str(&format!("            Self::{} => Some(\"{}\"),\n", r.ident, escaped));
        }
    }
    out.push_str("            _ => None,\n");
    out.push_str("        }\n");
    out.push_str("    }\n\n");

    let short_to_ident: HashMap<usize, String> = records
        .iter()
        .map(|r| (r.short_id, r.ident.clone()))
        .collect();
    let direct_implies: HashMap<usize, Vec<usize>> = records
        .iter()
        .map(|r| (r.short_id, r.implies_shorts.clone()))
        .collect();

    let mut transitive_implies: HashMap<usize, Vec<String>> = HashMap::new();
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

    out.push_str("    /// Returns the slice of formats directly and transitively implied by this format,\n");
    out.push_str("    /// as declared by `@implies(...)` directives in format tables.\n");
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

    out.push_str("    /// Returns the format shorthand string (e.g. \"f0\", \"f405\").\n");
    out.push_str("    #[must_use]\n");
    out.push_str("    pub const fn shorthand(&self) -> &'static str {\n");
    out.push_str("        match self {\n");
    for r in &records {
        out.push_str(&format!("            Self::{} => \"f{}\",\n", r.ident, r.short_id));
    }
    if !seen_idents.contains("Unknown") {
        out.push_str("            Self::Unknown => \"unknown\",\n");
    }
    out.push_str("        }\n");
    out.push_str("    }\n\n");

    out.push_str("    /// Looks up a `FormatId` from its format shorthand string (e.g. \"f405\").\n");
    out.push_str("    #[must_use]\n");
    out.push_str("    pub fn from_shorthand(shorthand: &str) -> Option<Self> {\n");
    out.push_str("        let trimmed = shorthand.trim();\n");
    out.push_str("        match trimmed {\n");
    for r in &records {
        out.push_str(&format!("            \"f{}\" => Some(Self::{}),\n", r.short_id, r.ident));
    }
    out.push_str("            _ => None,\n");
    out.push_str("        }\n");
    out.push_str("    }\n\n");

    out.push_str("    /// Returns the Global Document Character ID if known.\n");
    out.push_str("    #[must_use]\n");
    out.push_str("    pub const fn dc_id(&self) -> Option<u128> {\n");
    out.push_str("        match self {\n");
    for r in &records {
        out.push_str(&format!("            Self::{} => Some({}_u128),\n", r.ident, r.dc_id));
    }
    if !seen_idents.contains("Unknown") {
        out.push_str("            Self::Unknown => None,\n");
    }
    out.push_str("        }\n");
    out.push_str("    }\n\n");

    out.push_str("    /// Returns the format character as a `DcChar`, if known.\n");
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

    out.push_str("    /// Looks up a `FormatId` from its Global Document Character ID.\n");
    out.push_str("    #[must_use]\n");
    out.push_str("    pub const fn from_dc_id(id: u128) -> Option<Self> {\n");
    out.push_str("        match id {\n");
    for r in &records {
        out.push_str(&format!("            {}_u128 => Some(Self::{}),\n", r.dc_id, r.ident));
    }
    out.push_str("            _ => None,\n");
    out.push_str("        }\n");
    out.push_str("    }\n");
    out.push_str("}\n\n");

    // Generate individual format DcChar constants
    for r in &records {
        let screaming = to_screaming_snake_case(&r.ident);
        let escaped_label = r.label.replace('\n', " ").replace('"', "\\\"");
        out.push_str(&format!("/// DcChar constant for Format `{}` (Short f{}, Category: {}): {}\n", r.ident, r.short_id, r.category, escaped_label));
        out.push_str(&format!("pub const DC_{screaming}: DcChar = DcChar::from_format({});\n", r.short_id));
    }

    Ok(out)
}

/// Generates or updates `src/utilities/format_id.generated.rs` if the contents changed.
///
/// # Errors
/// Returns an error if directory resolution, generation, or writing fails.
pub fn generate_format_id_file(base_dir: &Path) -> Result<bool> {
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

    let target_file_utilities = repo_root
        .join("src")
        .join("utilities")
        .join("format_id.generated.rs");

    let code = generate_format_id_code(&formats_dir)?;
    write_if_changed(&target_file_utilities, &code)
}
