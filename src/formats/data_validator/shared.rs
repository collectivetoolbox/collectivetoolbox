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
use std::collections::HashMap;

const RUST_KEYWORDS: &[&str] = &[
    "as", "break", "const", "continue", "crate", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod",
    "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct",
    "super", "trait", "true", "type", "unsafe", "use", "where", "while",
    "async", "await", "dyn", "abstract", "become", "box", "do", "final",
    "macro", "override", "priv", "typeof", "unsized", "virtual", "yield",
    "try",
];

const VALID_BIDI_CLASSES: &[&str] = &[
    "L", "R", "AL", "EN", "ES", "ET", "AN", "CS", "NSM", "BN", "B", "S", "WS",
    "ON", "LRE", "LRO", "RLE", "RLO", "PDF", "LRI", "RLI", "FSI", "PDI",
];

const VALID_GENERAL_CATEGORIES: &[&str] = &[
    "Cc", "Cf", "Cs", "Co", "Cn", "Lu", "Ll", "Lt", "Lm", "Lo", "Mn", "Mc",
    "Me", "Nd", "Nl", "No", "Pc", "Pd", "Ps", "Pe", "Pi", "Pf", "Po", "Sm",
    "Sc", "Sk", "So", "Zs", "Zl", "Zp", "!Cx",
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
pub fn validate_general_category(cat: &str) -> Result<()> {
    let trimmed = cat.trim();
    if trimmed.is_empty() {
        bail!("General Category cannot be empty");
    }
    if !VALID_GENERAL_CATEGORIES.contains(&trimmed) {
        bail!(
            "Invalid General Category '{trimmed}' (expected standard Unicode category or '!Cx')"
        );
    }
    Ok(())
}

/// Validates Unicode Bidi class code.
pub fn validate_bidi_class(bidi: &str) -> Result<()> {
    let trimmed = bidi.trim();
    if trimmed.is_empty() {
        return Ok(());
    }
    if !VALID_BIDI_CLASSES.contains(&trimmed) {
        bail!(
            "Invalid Bidi Class '{trimmed}' (expected standard Unicode bidi class or empty)"
        );
    }
    Ok(())
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
    dc_names: &[(u32, &str, &str)], // (short_id, name, file_path)
    format_labels: &[(usize, &str, &str)], // (short_id, label, file_path)
    report: &mut ValidationReport,
) {
    let mut dc_name_map: HashMap<String, (u32, String)> = HashMap::new();
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
