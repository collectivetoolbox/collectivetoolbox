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

//! RSS renderer for the syndication data model.
//!
//! This module supports rendering feeds to multiple RSS versions:
//!
//! - **RSS 0.9**: Original Netscape RDF-based format. ASCII-only, max 8KB, max
//!   15 items, lowercase tags.
//! - **RSS 0.91**: `UserLand`'s simplified DTD-based format. Max 15 items,
//!   field length limits per the schema.
//! - **RSS 0.92**: Extended 0.91 with optional elements (cloud, enclosure,
//!   category, source).
//! - **RSS 1.0**: W3C RDF-based format with namespaces and `rdf:about`
//!   attributes.
//! - **RSS 2.0**: Most common modern format, with Atom namespace for self-link.

use crate::rss::rss_20::render_rss_20;
#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::Feed;
use ctb_formats_uri::{
    ensure_scheme_in, list_permanent_or_historic_iana_schemes,
};
use xml::writer::XmlEvent;

pub mod rss_09;
pub mod rss_091;
pub mod rss_092;
pub mod rss_10;
pub mod rss_20;
pub mod scripting_news;

// ============================================================================
// Constants
// ============================================================================

/// Maximum file size for RSS 0.9 (8KB).
const RSS_09_MAX_SIZE: usize = 8 * 1024;

/// Maximum number of items per channel for RSS 0.9 and 0.91.
const RSS_09_091_MAX_ITEMS: usize = 15;

/// RSS 0.9 namespace.
const RSS_09_NAMESPACE: &str = "http://channel.netscape.com/rdf/simple/0.9/";

/// RDF namespace.
const RDF_NAMESPACE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";

/// RSS 1.0 namespace.
const RSS_10_NAMESPACE: &str = "http://purl.org/rss/1.0/";

/// Atom namespace for RSS 2.0 self-link.
const ATOM_NAMESPACE: &str = "http://www.w3.org/2005/Atom";

// ============================================================================
// Public render function (dispatcher)
// ============================================================================

/// Render a [`Feed`] into RSS 2.0 format (default).
///
/// This is the main entry point that matches the signature expected by
/// `FeedFormat::Rss`.
pub(crate) fn render(feed: &Feed) -> Result<String> {
    render_rss_20(feed)
}

// ============================================================================
// Helper functions
// ============================================================================

pub(crate) fn ensure_http_or_ftp_uri(url: &str) -> Result<()> {
    ensure_scheme_in(url, vec!["http", "ftp"])?;
    Ok(())
}

pub(crate) fn ensure_http_https_or_ftp_uri(url: &str) -> Result<()> {
    ensure_scheme_in(url, vec!["http", "https", "ftp"])?;
    Ok(())
}

pub(crate) fn ensure_http_https_ftp_or_mailto_uri(url: &str) -> Result<()> {
    ensure_scheme_in(url, vec!["http", "https", "ftp", "mailto"])?;
    Ok(())
}

pub(crate) fn ensure_permanent_or_historic_iana_uri(url: &str) -> Result<()> {
    ensure_scheme_in(
        url,
        list_permanent_or_historic_iana_schemes()?
            .iter()
            .map(std::string::String::as_str)
            .collect(),
    )?;
    Ok(())
}

/// Write a simple text element like `<tag>text</tag>`.
fn write_text_element<W: std::io::Write>(
    writer: &mut xml::writer::EventWriter<W>,
    tag: &str,
    text: &str,
) -> Result<()> {
    writer
        .write(XmlEvent::start_element(tag))
        .context("writing element start")?;
    writer
        .write(XmlEvent::characters(text))
        .context("writing element text")?;
    writer
        .write(XmlEvent::end_element())
        .context("writing element end")?;
    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

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
    fn test_render_dispatcher() {
        // The default render() should produce RSS 2.0.
        let feed = Feed::empty("Test")
            .with_home_page_url_opt(Some("http://example.org/".into()));
        let result = render(&feed).unwrap();

        assert!(result.contains("<rss version=\"2.0\""), "Got: '{result}'");
    }
}
