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

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::report::ValidationReport;
use std::collections::HashMap;

pub use crate::dc_def::{BidiClass, GeneralCategory, SupportLevel};

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

    let trimmed = if let Some(stripped) = trimmed.strip_prefix("case:") {
        stripped.trim()
    } else {
        trimmed
    };

    if trimmed.is_empty() {
        bail!("Extension entry cannot be blank after case prefix");
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
        crate::syntax::parse_extension_regex(trimmed).with_context(|| {
            format!(
                "Extension regex pattern '{trimmed}' cannot be compiled to a Dc pattern rule"
            )
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

/// Validates support level values: blank or two space-separated levels `<general> <dc>` in `-1..=5`.
///
/// Scale for each component:
/// - `-1`: N/A (e.g. reverse transformation for one-way functions, or non-Dc formats)
/// - `0`: None / unsupported
/// - `1`: WIP
/// - `2`: Basic / stream / partial format support
/// - `3`: Full general format support / complete semantic roundtripping to Dc syntax
/// - `4`: Lossless and roundtrippable (for Dc: unambiguous bit-for-bit reconstruction)
/// - `5`: Level 4 + optional strict validation
pub fn validate_support_level(val: &str) -> Result<()> {
    SupportLevel::parse(val).map(|_| ())
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

        if !file_path.contains("unicode-clarifications.csv")
            && ctb_formats_unicode::is_known_unicode_name(clean_name)
        {
            report.add_error(
                file_path,
                None,
                Some("Name"),
                format!(
                    "Dc name '{clean_name}' (Short ID {short_id}) collides with official Unicode character name"
                ),
                Some("Disambiguate name, or do not add Dc as duplicate"),
            );
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

        // Cross-table check: Format label cannot collide with a Unicode character name
        if ctb_formats_unicode::is_known_unicode_name(clean_label) {
            report.add_error(
                file_path,
                None,
                Some("Label"),
                format!(
                    "Format label '{clean_label}' (Format ID {short_id}) collides with official Unicode character name"
                ),
                Some("Rename format label to avoid colliding with Unicode character"),
            );
        }
    }
}
