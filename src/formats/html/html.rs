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

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use ctb_utilities::csv_tools::CsvTable;
use html2text::from_read;
use include_dir::{Dir, include_dir};
use std::iter;
use std::sync::Arc;

pub mod text_clipper;

static HTML_DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data");

pub(crate) fn get_html_data(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&HTML_DATA_DIR, key)
}

pub fn html2text(html: Vec<u8>) -> Result<Vec<u8>> {
    html2text_with_width(html, 80)
}

pub fn html2text_with_width(html: Vec<u8>, width: u16) -> Result<Vec<u8>> {
    let text = from_read(html.as_slice(), width.into())
        .map_err(|e| anyhow::anyhow!("Failed to parse HTML to text: {e}"))?;
    Ok(text.into_bytes())
}

pub fn sanitize_html(document: Vec<u8>) -> Vec<u8> {
    let mut builder = ammonia::Builder::default();

    builder.add_generic_attributes(iter::once("class").chain(iter::once("id")));

    builder
        .clean(&String::from_utf8_lossy(&document))
        .to_string()
        .into_bytes()
}

/// Escape HTML special characters to be used in text content.
pub fn escape_text(text: &str) -> String {
    html_escape::encode_text(text).to_string()
}

/// Escape special characters for use in HTML attribute values.
pub fn escape_quoted_attr(text: &str) -> String {
    html_escape::encode_quoted_attribute(text).to_string()
}

fn entities_netscape_1999() -> Result<Arc<CsvTable>> {
    csv_tools::get_or_load_cached(
        "ctb_formats_html::data/entities-netscape-1999.csv",
        || {
            csv_tools::parse_csv_reader(
                &bail_if_none!(get_html_data("entities-netscape-1999.csv")),
                csv_tools::CsvParseOptions {
                    has_header: false,
                    ..Default::default()
                },
            )
        },
    )
}

fn entities_html32() -> Result<Arc<CsvTable>> {
    csv_tools::get_or_load_cached(
        "ctb_formats_html::data/entities-html32.csv",
        || {
            csv_tools::parse_csv_reader(
                &bail_if_none!(get_html_data("entities-html32.csv")),
                csv_tools::CsvParseOptions {
                    has_header: false,
                    ..Default::default()
                },
            )
        },
    )
}

fn core_entity_for_char(ch: char) -> Option<&'static str> {
    match ch {
        '<' => Some("&lt;"),
        '>' => Some("&gt;"),
        '&' => Some("&amp;"),
        '"' => Some("&quot;"),
        _ => None,
    }
}

fn core_char_for_entity(entity: &str) -> Option<char> {
    match entity {
        "&lt;" => Some('<'),
        "&gt;" => Some('>'),
        "&amp;" => Some('&'),
        "&quot;" => Some('"'),
        _ => None,
    }
}

fn decode_numeric_entity(entity: &str) -> Option<char> {
    let numeric = entity.strip_prefix("&#")?.strip_suffix(';')?;

    let (radix, digits) = if let Some(hex) = numeric.strip_prefix('x') {
        (16, hex)
    } else if let Some(hex) = numeric.strip_prefix('X') {
        (16, hex)
    } else {
        (10, numeric)
    };

    let value = u32::from_str_radix(digits, radix).ok()?;
    char::from_u32(value)
}

/// Replace characters with their named HTML entities.
///
/// Unknown characters are left as-is.
fn to_entities(input: String, data: Arc<CsvTable>) -> Result<String> {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        if let Some(entity) = core_entity_for_char(ch) {
            out.push_str(entity);
        } else if let Some(entity) =
            data.cell_where_col_eq(2, 0, &ch.to_string())
        {
            out.push_str(entity);
        } else {
            out.push(ch);
        }
    }
    Ok(out)
}

/// Replace HTML entities (named or numeric) with characters.
///
/// Unknown entities are left as-is.
fn from_entities(input: String, data: Arc<CsvTable>) -> Result<String> {
    let mut i = 0;
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

        let Some(semi_pos) = rest.find(';') else {
            out.push('&');
            i = i.saturating_add('&'.len_utf8());
            continue;
        };

        let Some(candidate) = rest.get(..=semi_pos) else {
            out.push('&');
            i = i.saturating_add('&'.len_utf8());
            continue;
        };
        if let Some(ch) = core_char_for_entity(candidate) {
            out.push(ch);
            i = i.saturating_add(semi_pos).saturating_add(1);
        } else if let Some(ch) = decode_numeric_entity(candidate) {
            out.push(ch);
            i = i.saturating_add(semi_pos).saturating_add(1);
        } else if let Some(ch) = data.cell_where_col_eq(0, 2, candidate) {
            out.push_str(ch);
            i = i.saturating_add(semi_pos).saturating_add(1);
        } else if let Some(ch) = data.cell_where_col_eq(0, 1, candidate) {
            out.push_str(ch);
            i = i.saturating_add(semi_pos).saturating_add(1);
        } else {
            out.push('&');
            i = i.saturating_add('&'.len_utf8());
        }
    }

    Ok(out)
}

/// Replace applicable characters with their Netscape 1999 named HTML entities.
///
/// Unknown characters are left as-is. Those with HTML syntax significance are
/// encoded, except '.
pub fn to_entities_netscape_1999(input: String) -> Result<String> {
    let data = entities_netscape_1999()?;
    to_entities(input, data)
}

/// Replace Netscape 1999 HTML entities (named or numeric) with characters.
///
/// Unknown entities are left as-is.
pub fn from_entities_netscape_1999(input: String) -> Result<String> {
    let data = entities_netscape_1999()?;
    from_entities(input, data)
}

/// Replace applicable characters with their HTML 3.2 named HTML entities.
///
/// Unknown characters and those with HTML syntax significance are left as-is.
pub fn to_entities_html32(input: String) -> Result<String> {
    let data = entities_html32()?;
    to_entities(input, data)
}

/// Replace HTML 3.2 named HTML entities (named or numeric) with characters.
///
/// Unknown entities are left as-is.
pub fn from_entities_html32(input: String) -> Result<String> {
    let data = entities_html32()?;
    from_entities(input, data)
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
}
