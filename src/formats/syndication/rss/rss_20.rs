// SPDX-License-Identifier: AGPL-3.0-or-later
/*
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
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::Feed;
use crate::rss::{
    ATOM_NAMESPACE, ensure_permanent_or_historic_iana_uri, write_text_element,
};
use xml::writer::{EmitterConfig, XmlEvent};

// ============================================================================
// RSS 2.0
// ============================================================================

/// Render a [`Feed`] into RSS 2.0 format.
///
/// RSS 2.0 is the most common modern RSS format. Includes optional atom:link
/// for self-reference.
///
/// Specification: <https://www.rssboard.org/rss-specification>
pub fn render_rss_20(feed: &Feed) -> Result<String> {
    let mut output = Vec::new();

    let config = EmitterConfig::new()
        .perform_indent(true)
        .indent_string("  ");
    let mut writer = config.create_writer(&mut output);

    writer
        .write(XmlEvent::StartDocument {
            version: xml::common::XmlVersion::Version10,
            encoding: Some("utf-8"),
            standalone: None,
        })
        .context("writing XML declaration")?;

    // Start rss element with version and optional atom namespace.
    writer
        .write(
            XmlEvent::start_element("rss")
                .ns("atom", ATOM_NAMESPACE)
                .attr("version", "2.0"),
        )
        .context("writing rss start element")?;

    writer
        .write(XmlEvent::start_element("channel"))
        .context("writing channel start")?;

    write_text_element(&mut writer, "title", feed.title())?;

    if let Some(home_page_url) = feed.home_page_url() {
        // The spec says "The data in these elements must begin with an
        // IANA-registered URI scheme". It's not clear to me if provisional or
        // historic schemes are allowed; I made a guess here.
        ensure_permanent_or_historic_iana_uri(home_page_url)?;

        write_text_element(&mut writer, "link", home_page_url)?;
    } else {
        // RSS 2.0 requires a channel link.
        bail!("RSS 2.0 requires a channel <link> URL");
    }

    write_text_element(&mut writer, "description", feed.title())?;

    // Optional atom:link for self.
    if let Some(feed_url) = feed.feed_url() {
        writer
            .write(
                XmlEvent::start_element("atom:link")
                    .attr("href", feed_url)
                    .attr("rel", "self")
                    .attr("type", "application/rss+xml"),
            )
            .context("writing atom:link")?;
        writer
            .write(XmlEvent::end_element())
            .context("closing atom:link")?;
    }

    // Items.
    for entry in feed.entries() {
        writer
            .write(XmlEvent::start_element("item"))
            .context("writing item start")?;

        if !entry.title().is_empty() {
            write_text_element(&mut writer, "title", entry.title())?;
        }

        if let Some(url) = entry.url() {
            ensure_permanent_or_historic_iana_uri(url)?;

            write_text_element(&mut writer, "link", url)?;
        }

        if !entry.body().is_empty() {
            write_text_element(&mut writer, "description", entry.body())?;
        }

        // pubDate
        if !entry.date().is_empty() {
            write_text_element(&mut writer, "pubDate", entry.date())?;
        }

        // Reason for fallback: per RSS 2.0 specification, item guid falls back to item ID if explicit URL is absent.
        let guid = entry.url().unwrap_or(entry.id());
        if !guid.is_empty() {
            write_text_element(&mut writer, "guid", guid)?;
        }

        writer
            .write(XmlEvent::end_element())
            .context("closing item")?;
    }

    writer
        .write(XmlEvent::end_element())
        .context("closing channel")?;

    writer
        .write(XmlEvent::end_element())
        .context("closing rss")?;

    Ok(String::from_utf8(output)
        .context("RSS 2.0 output is not valid UTF-8")?
        .replace(
            "<rss xmlns:atom=\"http://www.w3.org/2005/Atom\" version=\"2.0\">",
            "<rss version=\"2.0\" xmlns:atom=\"http://www.w3.org/2005/Atom\">",
        ))
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

    // -------------------------------------------------------------------------
    // RSS 2.0 tests
    // -------------------------------------------------------------------------

    #[crate::ctb_test]
    fn test_rss_20_basic() {
        let entry = Entry::new(
            "http://example.org/item1",
            "Test Item",
            "<p>Description</p>",
            "2024-01-15T10:00:00Z",
        )
        .with_url_opt(Some("http://example.org/item1".into()));

        let feed = Feed::new("Test Feed", vec![entry])
            .with_home_page_url_opt(Some("http://example.org/".into()))
            .with_feed_url_opt(Some("http://example.org/feed.rss".into()));

        let result = render_rss_20(&feed).unwrap();

        assert!(result.contains("<rss version=\"2.0\" xmlns:atom=\"http://www.w3.org/2005/Atom\">"), "Got: '{result}'");
        assert!(
            result.contains("xmlns:atom=\"http://www.w3.org/2005/Atom\""),
            "Got: '{result}'"
        );
        assert!(
            result.contains("<atom:link href=\"http://example.org/feed.rss\""),
            "Got: '{result}'"
        );
        assert!(
            result.contains("<pubDate>2024-01-15T10:00:00Z</pubDate>"),
            "Got: '{result}'"
        );
        assert!(
            result.contains("<guid>http://example.org/item1</guid>"),
            "Got: '{result}'"
        );
    }

    #[crate::ctb_test]
    fn test_rss_20_no_atom_link_without_feed_url() {
        let feed = Feed::new("Test Feed", vec![])
            .with_home_page_url_opt(Some("http://example.org/".into()));

        let result = render_rss_20(&feed).unwrap();

        assert!(!result.contains("<atom:link"));
    }

    #[crate::ctb_test]
    fn test_rss_20_item_without_title() {
        let entry = Entry::new(
            "http://example.org/item1",
            "",
            "Just a description",
            "",
        )
        .with_url_opt(Some("http://example.org/item1".into()));

        let feed = Feed::new("Test Feed", vec![entry])
            .with_home_page_url_opt(Some("http://example.org/".into()));

        let result = render_rss_20(&feed).unwrap();

        assert!(
            result.contains("<description>Just a description</description>")
        );
        assert!(!result.contains("<title></title>"));
    }
}
