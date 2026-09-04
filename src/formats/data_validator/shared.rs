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

//! Shared validation primitives for identifiers, MIME types, extensions, and schema constraints.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::report::ValidationReport;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Standard Unicode General Category codes and Collective non-Unicode extension (`!Cx`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GeneralCategory {
    Lu,
    Ll,
    Lt,
    Lm,
    Lo,
    Mn,
    Mc,
    Me,
    Nd,
    Nl,
    No,
    Pc,
    Pd,
    Ps,
    Pe,
    Pi,
    Pf,
    Po,
    Sm,
    Sc,
    Sk,
    So,
    Zs,
    Zl,
    Zp,
    Cc,
    Cf,
    Cs,
    Co,
    Cn,
    #[serde(rename = "!Cx")]
    NonUnicodeControl,
}

impl GeneralCategory {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Lu => "Lu",
            Self::Ll => "Ll",
            Self::Lt => "Lt",
            Self::Lm => "Lm",
            Self::Lo => "Lo",
            Self::Mn => "Mn",
            Self::Mc => "Mc",
            Self::Me => "Me",
            Self::Nd => "Nd",
            Self::Nl => "Nl",
            Self::No => "No",
            Self::Pc => "Pc",
            Self::Pd => "Pd",
            Self::Ps => "Ps",
            Self::Pe => "Pe",
            Self::Pi => "Pi",
            Self::Pf => "Pf",
            Self::Po => "Po",
            Self::Sm => "Sm",
            Self::Sc => "Sc",
            Self::Sk => "Sk",
            Self::So => "So",
            Self::Zs => "Zs",
            Self::Zl => "Zl",
            Self::Zp => "Zp",
            Self::Cc => "Cc",
            Self::Cf => "Cf",
            Self::Cs => "Cs",
            Self::Co => "Co",
            Self::Cn => "Cn",
            Self::NonUnicodeControl => "!Cx",
        }
    }
}

impl std::fmt::Display for GeneralCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for GeneralCategory {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "Lu" => Ok(Self::Lu),
            "Ll" => Ok(Self::Ll),
            "Lt" => Ok(Self::Lt),
            "Lm" => Ok(Self::Lm),
            "Lo" => Ok(Self::Lo),
            "Mn" => Ok(Self::Mn),
            "Mc" => Ok(Self::Mc),
            "Me" => Ok(Self::Me),
            "Nd" => Ok(Self::Nd),
            "Nl" => Ok(Self::Nl),
            "No" => Ok(Self::No),
            "Pc" => Ok(Self::Pc),
            "Pd" => Ok(Self::Pd),
            "Ps" => Ok(Self::Ps),
            "Pe" => Ok(Self::Pe),
            "Pi" => Ok(Self::Pi),
            "Pf" => Ok(Self::Pf),
            "Po" => Ok(Self::Po),
            "Sm" => Ok(Self::Sm),
            "Sc" => Ok(Self::Sc),
            "Sk" => Ok(Self::Sk),
            "So" => Ok(Self::So),
            "Zs" => Ok(Self::Zs),
            "Zl" => Ok(Self::Zl),
            "Zp" => Ok(Self::Zp),
            "Cc" => Ok(Self::Cc),
            "Cf" => Ok(Self::Cf),
            "Cs" => Ok(Self::Cs),
            "Co" => Ok(Self::Co),
            "Cn" => Ok(Self::Cn),
            "!Cx" => Ok(Self::NonUnicodeControl),
            other => bail!(
                "Invalid General Category '{other}' (expected standard Unicode category or '!Cx')"
            ),
        }
    }
}

/// Standard Unicode Bidirectional Class codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BidiClass {
    L,
    R,
    AL,
    EN,
    ES,
    ET,
    AN,
    CS,
    NSM,
    BN,
    B,
    S,
    WS,
    ON,
    LRE,
    LRO,
    RLE,
    RLO,
    PDF,
    LRI,
    RLI,
    FSI,
    PDI,
}

impl BidiClass {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::L => "L",
            Self::R => "R",
            Self::AL => "AL",
            Self::EN => "EN",
            Self::ES => "ES",
            Self::ET => "ET",
            Self::AN => "AN",
            Self::CS => "CS",
            Self::NSM => "NSM",
            Self::BN => "BN",
            Self::B => "B",
            Self::S => "S",
            Self::WS => "WS",
            Self::ON => "ON",
            Self::LRE => "LRE",
            Self::LRO => "LRO",
            Self::RLE => "RLE",
            Self::RLO => "RLO",
            Self::PDF => "PDF",
            Self::LRI => "LRI",
            Self::RLI => "RLI",
            Self::FSI => "FSI",
            Self::PDI => "PDI",
        }
    }
}

impl std::fmt::Display for BidiClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for BidiClass {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "L" => Ok(Self::L),
            "R" => Ok(Self::R),
            "AL" => Ok(Self::AL),
            "EN" => Ok(Self::EN),
            "ES" => Ok(Self::ES),
            "ET" => Ok(Self::ET),
            "AN" => Ok(Self::AN),
            "CS" => Ok(Self::CS),
            "NSM" => Ok(Self::NSM),
            "BN" => Ok(Self::BN),
            "B" => Ok(Self::B),
            "S" => Ok(Self::S),
            "WS" => Ok(Self::WS),
            "ON" => Ok(Self::ON),
            "LRE" => Ok(Self::LRE),
            "LRO" => Ok(Self::LRO),
            "RLE" => Ok(Self::RLE),
            "RLO" => Ok(Self::RLO),
            "PDF" => Ok(Self::PDF),
            "LRI" => Ok(Self::LRI),
            "RLI" => Ok(Self::RLI),
            "FSI" => Ok(Self::FSI),
            "PDI" => Ok(Self::PDI),
            other => bail!(
                "Invalid Bidi Class '{other}' (expected standard Unicode bidi class)"
            ),
        }
    }
}

const RUST_KEYWORDS: &[&str] = &[
    "as", "break", "const", "continue", "crate", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod",
    "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct",
    "super", "trait", "true", "type", "unsafe", "use", "where", "while",
    "async", "await", "dyn", "abstract", "become", "box", "do", "final",
    "macro", "override", "priv", "typeof", "unsized", "virtual", "yield",
    "try",
];

/// Validates that a string is a syntactically valid Rust identifier and not a keyword.
pub fn validate_rust_identifier(ident: &str) -> Result<()> {
    if ident.is_empty() {
        bail!("Identifier cannot be empty");
    }

    let mut chars = ident.chars();
    let Some(first) = chars.next() else {
        bail!("Identifier cannot be empty");
    };

    if !first.is_ascii_alphabetic() && first != '_' {
        bail!(
            "Identifier '{ident}' must start with an ASCII letter or underscore"
        );
    }

    for ch in chars {
        if !ch.is_ascii_alphanumeric() && ch != '_' {
            bail!(
                "Identifier '{ident}' contains invalid character '{ch}' (must be alphanumeric or '_')"
            );
        }
    }

    if RUST_KEYWORDS.contains(&ident) {
        bail!("Identifier '{ident}' is a reserved Rust keyword");
    }

    Ok(())
}

/// Splits a comma-separated string while respecting parentheses and brackets.
pub fn split_comma_separated_items(raw: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut depth_paren = 0usize;
    let mut depth_bracket = 0usize;
    let mut current = String::new();

    for ch in raw.chars() {
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

/// Validates a single extension entry or regex-delimited pattern (`~...~`).
pub fn validate_extension_entry(entry: &str) -> Result<()> {
    let trimmed = entry.trim();
    if trimmed.is_empty() {
        bail!("Extension entry cannot be blank");
    }

    if trimmed.starts_with('~') && trimmed.ends_with('~') && trimmed.len() >= 2
    {
        let pattern = if let Some(stripped) =
            trimmed.strip_prefix('~').and_then(|s| s.strip_suffix('~'))
        {
            stripped
        } else {
            trimmed
        };
        if pattern.is_empty() {
            bail!("Regex extension pattern '~...~' cannot be empty");
        }
        regex::Regex::new(pattern).with_context(|| {
            format!("Invalid regex pattern in extension rule: '{trimmed}'")
        })?;
        return Ok(());
    }

    if !trimmed.starts_with('.') {
        bail!(
            "Extension '{trimmed}' must start with a leading dot '.' or be a regex enclosed in '~...~'"
        );
    }

    Ok(())
}

/// Validates a comma-separated list of extensions or regex patterns.
pub fn validate_extensions_field(field: &str) -> Result<()> {
    let trimmed = field.trim();
    if trimmed.is_empty() {
        return Ok(());
    }

    for item in trimmed.split(',') {
        let item_trimmed = item.trim();
        if item_trimmed.is_empty() {
            continue;
        }
        validate_extension_entry(item_trimmed)?;
    }
    Ok(())
}

/// Validates a comma-separated list of MIME types (e.g. `text/plain, text/plain;charset=utf-8`).
pub fn validate_mime_field(field: &str) -> Result<()> {
    let trimmed = field.trim();
    if trimmed.is_empty() {
        return Ok(());
    }

    for item in trimmed.split(',') {
        let item_trimmed = item.trim();
        if item_trimmed.is_empty() {
            continue;
        }
        let mime_base = match item_trimmed.split(';').next() {
            Some(base) => base.trim(),
            None => item_trimmed,
        };
        let Some((typ, subtyp)) = mime_base.split_once('/') else {
            bail!(
                "MIME type '{item_trimmed}' missing '/' separator (expected 'type/subtype')"
            );
        };
        if typ.trim().is_empty() || subtyp.trim().is_empty() {
            bail!("MIME type '{item_trimmed}' has empty type or subtype");
        }
    }
    Ok(())
}

/// Validates support level values: blank or `-1..=5`.
pub fn validate_support_level(val: &str) -> Result<()> {
    let trimmed = val.trim();
    if trimmed.is_empty() || trimmed == "0" {
        return Ok(());
    }
    match trimmed {
        "-1" | "1" | "2" | "3" | "4" | "5" => Ok(()),
        other => bail!(
            "Invalid support level '{other}' (expected -1..=5 or blank/0)"
        ),
    }
}

/// Validates Unicode General Category code (e.g. `Cc`, `Sm`, `!Cx`).
pub fn validate_general_category(cat: &str) -> Result<GeneralCategory> {
    cat.parse::<GeneralCategory>()
}

/// Validates Unicode Bidi class code (e.g. `L`, `BN`, `ON`).
pub fn validate_bidi_class(bidi: &str) -> Result<BidiClass> {
    bidi.parse::<BidiClass>()
}

/// Validates Canonical Combining Class integer (`0..=254`).
pub fn validate_combining_class(cls: &str) -> Result<u8> {
    let trimmed = cls.trim();
    if trimmed.is_empty() {
        return Ok(0);
    }
    let val = trimmed.parse::<u8>().with_context(|| {
        format!("Invalid combining class integer '{trimmed}'")
    })?;
    if val > 254 {
        bail!("Combining class {val} out of valid Unicode range (0..=254)");
    }
    Ok(val)
}

/// Validates that format labels and Dc names are unique across all records.
pub fn validate_cross_table_uniqueness(
    dc_names: &[(usize, &str, &str)], // (short_id, name, file_path)
    format_labels: &[(usize, &str, &str)], // (short_id, label, file_path)
    report: &mut ValidationReport,
) {
    let mut dc_name_map: HashMap<String, (usize, String)> = HashMap::new();
    for &(short_id, raw_name, file_path) in dc_names {
        let clean_name = if let Some(stripped) = raw_name.strip_prefix('!') {
            stripped.trim()
        } else {
            raw_name.trim()
        };
        if clean_name.is_empty() {
            continue;
        }
        let key = clean_name.to_lowercase();
        if let Some((prev_id, prev_file)) = dc_name_map.get(&key) {
            report.add_error(
                file_path,
                None,
                Some("Name"),
                format!(
                    "Duplicate Dc name '{clean_name}' (Short ID {short_id}) already defined in {prev_file} (Short ID {prev_id})"
                ),
                Some("Ensure all Dc names are distinct"),
            );
        } else {
            dc_name_map.insert(key, (short_id, file_path.to_string()));
        }
    }

    let mut fmt_label_map: HashMap<String, (usize, String)> = HashMap::new();
    for &(short_id, raw_label, file_path) in format_labels {
        let clean_label = raw_label.trim();
        if clean_label.is_empty() {
            continue;
        }
        let key = clean_label.to_lowercase();
        if let Some((prev_id, prev_file)) = fmt_label_map.get(&key) {
            report.add_error(
                file_path,
                None,
                Some("Label"),
                format!(
                    "Duplicate Format label '{clean_label}' (Format ID {short_id}) already defined in {prev_file} (Format ID {prev_id})"
                ),
                Some("Ensure all Format labels are distinct"),
            );
        } else {
            fmt_label_map
                .insert(key.clone(), (short_id, file_path.to_string()));
        }

        // Cross-table check: Format label cannot collide with a Dc name
        if let Some((dc_id, dc_file)) = dc_name_map.get(&key) {
            report.add_error(
                file_path,
                None,
                Some("Label"),
                format!(
                    "Format label '{clean_label}' (Format ID {short_id}) collides with Dc name in {dc_file} (Dc ID {dc_id})"
                ),
                Some("Rename format label or Dc name to resolve collision"),
            );
        }
    }
}
