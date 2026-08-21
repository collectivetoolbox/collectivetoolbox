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

//! RSS 0.90 syndication feed serializer and validator.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::Feed;
use crate::rss::{
    RDF_NAMESPACE, RSS_09_091_MAX_ITEMS, RSS_09_MAX_SIZE, RSS_09_NAMESPACE,
    ensure_http_https_ftp_or_mailto_uri, write_text_element,
};
use ctb_formats_html::entities::to_entities_netscape_1999;
use ctb_formats_utf8::{ellipsize_to_max_bytes, to_ascii_translit};
use ctb_utilities::anyhow::ensure;
use xml::writer::{EmitterConfig, XmlEvent};

// ============================================================================
// RSS 0.9 (RDF Site Summary)
// ============================================================================

fn prep_string(input: &str, max_bytes: usize) -> Result<String> {
    let input =
        to_ascii_translit(&to_entities_netscape_1999(input.to_string())?);
    let input = ellipsize_to_max_bytes(&input, max_bytes);
    Ok(to_ascii_translit(&input))
}

/// Render a [`Feed`] into RSS 0.9 (RDF Site Summary) format.
///
/// Constraints enforced:
/// - ASCII characters only (non-ASCII replaced with `?`)
/// - Maximum 15 items per channel
/// - Maximum 8KB total file size
/// - Lowercase tags only
/// - Title: max 40 chars, Description: max 500 chars, Link: max 500 chars
/// - Item title: max 100 chars, Item link: max 500 chars
///
/// Specification: <https://www.rssboard.org/rss-0-9-0>
pub fn render_rss_09(feed: &Feed) -> Result<String> {
    let mut output = Vec::new();

    // Use a custom config with no escaping because `prep_string()` already
    // returns entity-escaped ASCII.
    let mut config = EmitterConfig::new().perform_indent(false);
    config.perform_escaping = false;
    let mut writer = config.create_writer(&mut output);

    // XML declaration
    writer
        .write(XmlEvent::StartDocument {
            version: xml::common::XmlVersion::Version10,
            encoding: None,
            standalone: None,
        })
        .context("writing XML declaration")?;

    // Start rdf:RDF element with namespaces.
    writer
        .write(
            XmlEvent::start_element("rdf:RDF")
                .ns("rdf", RDF_NAMESPACE)
                .default_ns(RSS_09_NAMESPACE),
        )
        .context("writing rdf:RDF start element")?;

    // Channel element.
    writer
        .write(XmlEvent::start_element("channel"))
        .context("writing channel start")?;

    write_text_element(&mut writer, "title", &prep_string(feed.title(), 40)?)?;

    if let Some(home_page_url) = feed.home_page_url() {
        ensure_http_https_ftp_or_mailto_uri(home_page_url)?;

        write_text_element(
            &mut writer,
            "link",
            &prep_string(home_page_url, 500)?,
        )?;
    }

    // Use title as description fallback since Feed doesn't have description.
    write_text_element(
        &mut writer,
        "description",
        &prep_string(feed.title(), 500)?,
    )?;

    writer
        .write(XmlEvent::end_element())
        .context("closing channel element")?;

    // Items (max 15).
    let items = feed.entries().iter().take(RSS_09_091_MAX_ITEMS);
    for entry in items {
        writer
            .write(XmlEvent::start_element("item"))
            .context("writing item start")?;

        write_text_element(
            &mut writer,
            "title",
            &prep_string(entry.title(), 100)?,
        )?;

        if let Some(item_link) = entry.url() {
            ensure_http_https_ftp_or_mailto_uri(item_link)?;

            write_text_element(
                &mut writer,
                "link",
                &prep_string(item_link, 500)?,
            )?;
        } else {
            warn!("RSS 0.9 item is missing link URL: '{}'", entry.title());
        }

        writer
            .write(XmlEvent::end_element())
            .context("closing item element")?;
    }

    // Close rdf:RDF
    writer
        .write(XmlEvent::end_element())
        .context("closing rdf:RDF element")?;

    let result = String::from_utf8(output)
        .context("RSS 0.9 output is not valid UTF-8")?
        .replace(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>",
            "<?xml version=\"1.0\"?>",
        );

    ensure!(
        result.bytes().all(|b| b.is_ascii()),
        "RSS 0.9 output contains non-ASCII bytes"
    );

    // Enforce 8KB limit.
    if result.len() > RSS_09_MAX_SIZE {
        bail!("RSS 0.9 output exceeds 8KB limit ({} bytes)", result.len());
    }

    Ok(result)
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
    use crate::Entry;
    use crate::rss::rss_09::render_rss_09;

    // -------------------------------------------------------------------------
    // RSS 0.9 tests
    // -------------------------------------------------------------------------

    #[crate::ctb_test]
    fn test_rss_09_basic() {
        let entry = Entry::new(
            "http://example.org/item1",
            "Test Item",
            "<p>Content</p>",
            "2024-01-15T10:00:00Z",
        )
        .with_url_opt(Some("http://example.org/item1".into()));

        let feed = Feed::new("Test Feed", vec![entry])
            .with_home_page_url_opt(Some("http://example.org/".into()));

        let result = render_rss_09(&feed).unwrap();

        assert!(
            result.contains("<?xml version=\"1.0\"?>"),
            "Got: '{result}'"
        );
        assert!(result.contains("xmlns:rdf="), "Got: '{result}'");
        assert!(result.contains("<channel>"), "Got: '{result}'");
        assert!(
            result.contains("<title>Test Feed</title>"),
            "Got: '{result}'"
        );
        assert!(
            result.contains("<link>http://example.org/</link>"),
            "Got: '{result}'"
        );
        assert!(result.contains("<item>"), "Got: '{result}'");
        assert!(
            result.contains("<title>Test Item</title>"),
            "Got: '{result}'"
        );
        assert!(result.contains("</rdf:RDF>"), "Got: '{result}'");
    }

    #[crate::ctb_test]
    fn test_rss_09_ascii_only() {
        let entry = Entry::new(
            "http://example.org/item1",
            "Tëst with語 ümlauts",
            "<p>Cönt語ent</p>",
            "2024-01-15",
        )
        .with_url_opt(Some("http://example.org/item1".into()));

        let feed = Feed::new("Fëed with spë語cial chars", vec![entry])
            .with_home_page_url_opt(Some("http://example.org/".into()));

        let result = render_rss_09(&feed).unwrap();

        // Non-ASCII should be replaced with `?`.
        assert!(
            result.contains("<title>T&euml;st withYu &uuml;mlauts</title"),
            "Got: '{result}'"
        );
        assert!(
            result.contains("<description>F&euml;ed with sp&euml;Yu cial chars</description>"),
            "Got: '{result}'"
        );
        // Should be valid ASCII.
        assert!(result.bytes().all(|b| b.is_ascii()));
    }

    #[crate::ctb_test]
    fn test_rss_09_max_15_items() {
        let entries: Vec<_> = (0..20)
            .map(|i| {
                Entry::new(
                    format!("http://example.org/item{i}"),
                    format!("Item {i}"),
                    "",
                    "",
                )
                .with_url_opt(Some(format!("http://example.org/item{i}")))
            })
            .collect();

        let feed = Feed::new("Many Items", entries)
            .with_home_page_url_opt(Some("http://example.org/".into()));

        let result = render_rss_09(&feed).unwrap();

        // Should contain items 0-14, but not 15-19.
        assert!(result.contains("Item 0"));
        assert!(result.contains("Item 14"));
        assert!(!result.contains("Item 15"));
        assert!(!result.contains("Item 19"));
    }

    #[crate::ctb_test]
    fn test_rss_09_truncation() {
        let long_title = "A".repeat(50);
        let entry = Entry::new("http://example.org/item1", long_title, "", "")
            .with_url_opt(Some("http://example.org/item1".into()));

        let feed = Feed::new("Test", vec![entry])
            .with_home_page_url_opt(Some("http://example.org/".into()));

        let result = render_rss_09(&feed).unwrap();

        // Channel title max is 40, but item title max is 100.
        // The feed title "Test" is short, so it's fine.
        // The item title of 50 chars is under 100, so it's fine.
        assert!(result.contains(&"A".repeat(50)));
    }
}
