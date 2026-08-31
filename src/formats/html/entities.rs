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

//! HTML parsing, plain text rendering, and HTML table extraction utilities.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::get_html_data;
use ctb_utilities::csv_tools::{self, CsvTable};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

/// Supported HTML / XML / MathML entity sets.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum EntitySet {
    /// HTML 3.2 Latin 1 entities.
    Html32,
    /// Netscape 1999 RSS board HTML entity set.
    Netscape1999,
    /// XML standard entity set (including &apos;).
    Xml,
    /// HTML 4.0 character entity set.
    Html4,
    /// HTML 5 named character references (including multi-codepoint and legacy semicolon-less references).
    Html5,
    /// MathML entity set.
    MathMl,
}

impl EntitySet {
    /// Returns the CSV filename corresponding to this entity set.
    #[must_use]
    pub const fn filename(self) -> &'static str {
        match self {
            Self::Html32 => "entities-html32.csv",
            Self::Netscape1999 => "entities-netscape-1999.csv",
            Self::Xml => "entities-xml.csv",
            Self::Html4 => "entities-html4.0.csv",
            Self::Html5 => "entities-html5.csv",
            Self::MathMl => "entities-mathml.csv",
        }
    }

    /// Returns the cache key for this entity set.
    #[must_use]
    pub const fn cache_key(self) -> &'static str {
        match self {
            Self::Html32 => "ctb_formats_html::data/entities-html32.csv",
            Self::Netscape1999 => {
                "ctb_formats_html::data/entities-netscape-1999.csv"
            }
            Self::Xml => "ctb_formats_html::data/entities-xml.csv",
            Self::Html4 => "ctb_formats_html::data/entities-html4.0.csv",
            Self::Html5 => "ctb_formats_html::data/entities-html5.csv",
            Self::MathMl => "ctb_formats_html::data/entities-mathml.csv",
        }
    }
}

/// In-memory table storing bidirectional entity lookups.
#[derive(Clone, Debug, Default)]
pub struct EntityTable {
    char_to_entity: HashMap<String, String>,
    entity_to_text: HashMap<String, String>,
    max_entity_len: usize,
}

fn parse_entity_table(
    set: EntitySet,
    records: &CsvTable,
) -> Result<EntityTable> {
    let mut char_to_entity = HashMap::new();
    let mut entity_to_text = HashMap::new();

    for row_idx in 0..records.row_count() {
        let Some(row) = records.row(row_idx) else {
            continue;
        };
        if row.is_empty() {
            continue;
        }

        match set {
            EntitySet::Html32 | EntitySet::Netscape1999 => {
                // Format: Col 0 = Char, Col 1 = Numeric (&#...;), Col 2 = Named (&...;)
                if let (Some(col0), Some(col2)) = (row.first(), row.get(2)) {
                    let char_str = col0.clone();
                    let named_entity = col2.clone();
                    if !char_str.is_empty() && !named_entity.is_empty() {
                        char_to_entity
                            .entry(char_str.clone())
                            .or_insert_with(|| named_entity.clone());
                        entity_to_text.insert(named_entity, char_str);
                    }
                }
            }
            EntitySet::Xml | EntitySet::Html4 => {
                // Format: Col 0 = Decimal codepoint, Col 1 = name (without & or ;)
                if let (Some(col0), Some(col1)) = (row.first(), row.get(1)) {
                    if let Ok(ch) =
                        crate::utilities::string::parse_dec_char(col0)
                    {
                        let char_str = ch.to_string();
                        let named_entity = format!("&{col1};");
                        char_to_entity
                            .entry(char_str.clone())
                            .or_insert_with(|| named_entity.clone());
                        entity_to_text.insert(named_entity, char_str);
                    }
                }
            }
            EntitySet::Html5 => {
                // Format: Col 0 = Hex codepoints space-separated, Col 1 = name (with or without ;)
                if let (Some(col0), Some(col1)) = (row.first(), row.get(1)) {
                    let mut text = String::new();
                    for part in col0.split_whitespace() {
                        if let Ok(ch) =
                            crate::utilities::string::parse_hex_char(part)
                        {
                            text.push(ch);
                        }
                    }
                    if !text.is_empty() {
                        let named_entity = format!("&{col1}");
                        entity_to_text
                            .insert(named_entity.clone(), text.clone());
                        if named_entity.ends_with(';') {
                            char_to_entity.entry(text).or_insert(named_entity);
                        }
                    }
                }
            }
            EntitySet::MathMl => {
                // Format: Col 0 = Codepoints space-separated (decimal or hex), Col 1 = name (without & or ;)
                if let (Some(col0), Some(col1)) = (row.first(), row.get(1)) {
                    let mut text = String::new();
                    for part in col0.split_whitespace() {
                        let ch_opt = if col0.starts_with('x') {
                            crate::utilities::string::parse_hex_char(part).ok()
                        } else {
                            crate::utilities::string::parse_dec_char(part).ok()
                        };
                        if let Some(ch) = ch_opt {
                            text.push(ch);
                        }
                    }
                    if !text.is_empty() {
                        let named_entity = format!("&{col1};");
                        entity_to_text
                            .insert(named_entity.clone(), text.clone());
                        char_to_entity.entry(text).or_insert(named_entity);
                    }
                }
            }
        }
    }

    let max_entity_len = match entity_to_text.keys().map(String::len).max() {
        Some(len) => len,
        None => 0,
    };

    Ok(EntityTable {
        char_to_entity,
        entity_to_text,
        max_entity_len,
    })
}

static ENTITY_TABLES: OnceLock<Mutex<HashMap<EntitySet, Arc<EntityTable>>>> =
    OnceLock::new();

fn get_entity_table(set: EntitySet) -> Result<Arc<EntityTable>> {
    let map_mutex = ENTITY_TABLES.get_or_init(|| Mutex::new(HashMap::new()));
    let mut map = map_mutex
        .lock()
        .map_err(|e| anyhow::anyhow!("Entity cache lock poisoned: {e}"))?;
    if let Some(table) = map.get(&set) {
        return Ok(Arc::clone(table));
    }
    let csv = csv_tools::get_or_load_cached(set.cache_key(), || {
        let bytes = bail_if_none!(get_html_data(set.filename()));
        csv_tools::parse_csv_reader(
            &bytes,
            csv_tools::CsvParseOptions {
                has_header: false,
                ..Default::default()
            },
        )
    })?;
    let table = Arc::new(parse_entity_table(set, &csv)?);
    map.insert(set, Arc::clone(&table));
    Ok(table)
}

/// Escape HTML special characters to be used in text content.
pub fn escape_text(text: &str) -> String {
    html_escape::encode_text(text).to_string()
}

/// Escape special characters for use in HTML attribute values.
pub fn escape_quoted_attr(text: &str) -> String {
    html_escape::encode_quoted_attribute(text).to_string()
}

fn core_entity_for_char(ch: char, set: EntitySet) -> Option<&'static str> {
    match ch {
        '<' => Some("&lt;"),
        '>' => Some("&gt;"),
        '&' => Some("&amp;"),
        '"' => Some("&quot;"),
        '\'' if matches!(set, EntitySet::Xml) => Some("&apos;"),
        _ => None,
    }
}

fn core_char_for_entity(entity: &str) -> Option<char> {
    match entity {
        "&lt;" => Some('<'),
        "&gt;" => Some('>'),
        "&amp;" => Some('&'),
        "&quot;" => Some('"'),
        "&apos;" => Some('\''),
        _ => None,
    }
}

fn decode_numeric_entity(entity: &str) -> Option<char> {
    let numeric = entity.strip_prefix("&#")?.strip_suffix(';')?;

    let (radix, digits) = if let Some(hex) = numeric
        .strip_prefix('x')
        .or_else(|| numeric.strip_prefix('X'))
    {
        (16, hex)
    } else {
        (10, numeric)
    };

    let value = u32::from_str_radix(digits, radix).ok()?;
    char::from_u32(value)
}

/// Replace applicable characters with named HTML entities from the specified entity set.
///
/// Unknown characters are left as-is.
pub fn to_entities(input: String, set: EntitySet) -> Result<String> {
    let table = get_entity_table(set)?;
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        if let Some(entity) = core_entity_for_char(ch, set) {
            out.push_str(entity);
        } else if let Some(entity) = table.char_to_entity.get(&ch.to_string()) {
            out.push_str(entity);
        } else {
            out.push(ch);
        }
    }
    Ok(out)
}

/// Replace HTML entities (named or numeric) with characters using the specified entity set.
///
/// Unknown entities are left as-is.
pub fn from_entities(input: String, set: EntitySet) -> Result<String> {
    let table = get_entity_table(set)?;
    let mut i = 0usize;
    let mut out = String::with_capacity(input.len());

    while i < input.len() {
        let Some(rest) = input.get(i..) else {
            break;
        };

        if !rest.starts_with('&') {
            let Some(ch) = rest.chars().next() else {
                break;
            };
            out.push(ch);
            i = i.saturating_add(ch.len_utf8());
            continue;
        }

        // Numeric entity: &#123; or &#x12a;
        if rest.starts_with("&#") {
            if let Some(semi_pos) = rest.find(';') {
                if let Some(candidate) = rest.get(..=semi_pos) {
                    if let Some(ch) = decode_numeric_entity(candidate) {
                        out.push(ch);
                        i = i.saturating_add(semi_pos).saturating_add(1);
                        continue;
                    }
                }
            }
        }

        // Named entity lookup
        // 1. Try finding a ';' up to max_entity_len
        let max_search = table.max_entity_len.min(rest.len());
        let mut matched = false;

        if let Some(semi_pos) = rest.get(..max_search).and_then(|s| s.find(';'))
        {
            if let Some(candidate) = rest.get(..=semi_pos) {
                if let Some(ch) = core_char_for_entity(candidate) {
                    out.push(ch);
                    i = i.saturating_add(semi_pos).saturating_add(1);
                    matched = true;
                } else if let Some(replacement) =
                    table.entity_to_text.get(candidate)
                {
                    out.push_str(replacement);
                    i = i.saturating_add(semi_pos).saturating_add(1);
                    matched = true;
                }
            }
        }

        if matched {
            continue;
        }

        // 2. If no semicolon match, check for semicolon-less entity (only registered for HTML5)
        let mut longest_len = 0usize;
        let mut longest_replacement: Option<&str> = None;

        for prefix_len in (2..=max_search).rev() {
            if let Some(candidate) = rest.get(..prefix_len) {
                if let Some(replacement) = table.entity_to_text.get(candidate) {
                    longest_len = prefix_len;
                    longest_replacement = Some(replacement.as_str());
                    break;
                }
            }
        }

        if let Some(replacement) = longest_replacement {
            out.push_str(replacement);
            i = i.saturating_add(longest_len);
            continue;
        }

        // Unrecognized entity: output '&' and advance 1 byte
        out.push('&');
        i = i.saturating_add('&'.len_utf8());
    }

    Ok(out)
}

/// Replace applicable characters with their Netscape 1999 named HTML entities.
///
/// Unknown characters are left as-is. Those with HTML syntax significance are
/// encoded, except '.
pub fn to_entities_netscape_1999(input: String) -> Result<String> {
    to_entities(input, EntitySet::Netscape1999)
}

/// Replace Netscape 1999 HTML entities (named or numeric) with characters.
///
/// Unknown entities are left as-is.
pub fn from_entities_netscape_1999(input: String) -> Result<String> {
    from_entities(input, EntitySet::Netscape1999)
}

/// Replace applicable characters with their HTML 3.2 named HTML entities.
///
/// Unknown characters are left as-is. Those with HTML syntax significance are
/// encoded, except '.
pub fn to_entities_html32(input: String) -> Result<String> {
    to_entities(input, EntitySet::Html32)
}

/// Replace HTML 3.2 named HTML entities (named or numeric) with characters.
///
/// Unknown entities are left as-is.
pub fn from_entities_html32(input: String) -> Result<String> {
    from_entities(input, EntitySet::Html32)
}

/// Replace applicable characters with their XML named entities.
///
/// Unknown characters are left as-is. Those with XML syntax significance (<, >, &, ", ')
/// are encoded.
pub fn to_entities_xml(input: String) -> Result<String> {
    to_entities(input, EntitySet::Xml)
}

/// Replace XML named entities (named or numeric) with characters.
///
/// Unknown entities are left as-is.
pub fn from_entities_xml(input: String) -> Result<String> {
    from_entities(input, EntitySet::Xml)
}

/// Replace applicable characters with their HTML 4.0 named HTML entities.
///
/// Unknown characters are left as-is. Those with HTML syntax significance are
/// encoded, except '.
pub fn to_entities_html4(input: String) -> Result<String> {
    to_entities(input, EntitySet::Html4)
}

/// Replace HTML 4.0 named HTML entities (named or numeric) with characters.
///
/// Unknown entities are left as-is.
pub fn from_entities_html4(input: String) -> Result<String> {
    from_entities(input, EntitySet::Html4)
}

/// Replace applicable characters with their HTML 5 named HTML entities.
///
/// Unknown characters are left as-is. Those with HTML syntax significance are
/// encoded, except '.
pub fn to_entities_html5(input: String) -> Result<String> {
    to_entities(input, EntitySet::Html5)
}

/// Replace HTML 5 named HTML entities (named or numeric) with characters.
///
/// Unknown entities are left as-is.
pub fn from_entities_html5(input: String) -> Result<String> {
    from_entities(input, EntitySet::Html5)
}

/// Replace applicable characters with their MathML named entities.
///
/// Unknown characters are left as-is. Those with HTML/XML syntax significance are
/// encoded, except '.
pub fn to_entities_mathml(input: String) -> Result<String> {
    to_entities(input, EntitySet::MathMl)
}

/// Replace MathML named entities (named or numeric) with characters.
///
/// Unknown entities are left as-is.
pub fn from_entities_mathml(input: String) -> Result<String> {
    from_entities(input, EntitySet::MathMl)
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
    fn test_escape_html() {
        assert_eq!(escape_text("Hello"), "Hello");
        assert_eq!(escape_text("<script>"), "&lt;script&gt;");
        assert_eq!(escape_text("A & B"), "A &amp; B");
        assert_eq!(escape_text("1 < 2 > 0"), "1 &lt; 2 &gt; 0");
    }

    #[crate::ctb_test]
    fn test_escape_attr() {
        assert_eq!(escape_quoted_attr("Hello"), "Hello");
        assert_eq!(escape_quoted_attr("a\"b"), "a&quot;b");
        assert_eq!(escape_quoted_attr("it's"), "it&#x27;s");
        assert_eq!(escape_quoted_attr("a&b"), "a&amp;b");
    }

    #[crate::ctb_test]
    fn test_netscape_entities_1999_roundtrip_basic() -> Result<()> {
        assert_eq!(
            &*to_entities_netscape_1999("Å & B".to_string())?,
            "&ring; &amp; B"
        );

        let input = "<&\"".to_string();
        let encoded = to_entities_netscape_1999(input.clone())?;
        assert_eq!(encoded, "&lt;&amp;&quot;".to_string());
        let decoded = from_entities_netscape_1999(encoded)?;
        assert_eq!(decoded, input);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_netscape_entities_1999_decodes_numeric() -> Result<()> {
        let decoded =
            from_entities_netscape_1999("&#60;&#38;&#34;".to_string())?;
        assert_eq!(decoded, "<&\"".to_string());
        Ok(())
    }

    #[crate::ctb_test]
    fn test_html32_roundtrip_basic() -> Result<()> {
        assert_eq!(
            &*to_entities_html32("Å & B".to_string())?,
            "&Aring; &amp; B"
        );

        let input = "<&\"".to_string();
        let encoded = to_entities_html32(input.clone())?;
        assert_eq!(encoded, "&lt;&amp;&quot;".to_string());
        let decoded = from_entities_html32(encoded)?;
        assert_eq!(decoded, input);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_html32_decodes_numeric() -> Result<()> {
        let decoded = from_entities_html32("&#60;&#38;&#34;".to_string())?;
        assert_eq!(decoded, "<&\"".to_string());
        Ok(())
    }

    #[crate::ctb_test]
    fn test_xml_entities() -> Result<()> {
        let input = "<&\"'>".to_string();
        let encoded = to_entities_xml(input.clone())?;
        assert_eq!(encoded, "&lt;&amp;&quot;&apos;&gt;");
        let decoded = from_entities_xml(encoded)?;
        assert_eq!(decoded, input);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_html4_entities() -> Result<()> {
        let input = "© 2026 & Co. ™".to_string();
        let encoded = to_entities_html4(input.clone())?;
        assert_eq!(encoded, "&copy; 2026 &amp; Co. &trade;");
        let decoded = from_entities_html4(encoded)?;
        assert_eq!(decoded, input);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_html5_entities_multi_codepoint_and_semicolonless() -> Result<()> {
        // Multi-codepoint entity decoding
        let decoded = from_entities_html5("&ncongdot;".to_string())?;
        assert_eq!(decoded, "\u{2A6D}\u{338}");

        // Semicolon-less legacy entity in HTML5
        let decoded_legacy = from_entities_html5("&copy 2026".to_string())?;
        assert_eq!(decoded_legacy, "© 2026");

        // Non-legacy entity with semicolon only should NOT decode without semicolon
        let not_decoded = from_entities_html5("&NewLine text".to_string())?;
        assert_eq!(not_decoded, "&NewLine text");

        // With semicolon it does decode
        let decoded_nl = from_entities_html5("&NewLine;text".to_string())?;
        assert_eq!(decoded_nl, "\ntext");

        Ok(())
    }

    #[crate::ctb_test]
    fn test_mathml_entities() -> Result<()> {
        let decoded = from_entities_mathml("&ncongdot;".to_string())?;
        assert_eq!(decoded, "\u{2A6D}\u{338}");

        let decoded_amp = from_entities_mathml("&amp;&AMP;".to_string())?;
        assert_eq!(decoded_amp, "&&");
        Ok(())
    }
}
