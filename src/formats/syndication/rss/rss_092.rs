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

//! RSS 0.92 syndication feed serializer and validator.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::Feed;
use crate::rss::{ensure_http_or_ftp_uri, write_text_element};
use xml::writer::{EmitterConfig, XmlEvent};

// ============================================================================
// RSS 0.92
// ============================================================================

/// Render a [`Feed`] into RSS 0.92 format.
///
/// RSS 0.92 is similar to 0.91 but allows more optional elements. No explicit
/// item limit is defined in the spec.
///
/// Possibly TODO: may require an `<image>` element for the channel. However,
/// the sample file doesn't include one, so it may be intended to be newly
/// optional in 0.92 and just not documented as such.
///
/// Specification: <https://www.rssboard.org/rss-0-9-2>
pub fn render_rss_092(feed: &Feed) -> Result<String> {
    let mut output = Vec::new();

    let config = EmitterConfig::new()
        .perform_indent(true)
        .indent_string("\t");
    let mut writer = config.create_writer(&mut output);

    writer
        .write(XmlEvent::StartDocument {
            version: xml::common::XmlVersion::Version10,
            encoding: Some("utf-8"),
            standalone: None,
        })
        .context("writing XML declaration")?;

    writer
        .write(XmlEvent::start_element("rss").attr("version", "0.92"))
        .context("writing rss start element")?;

    writer
        .write(XmlEvent::start_element("channel"))
        .context("writing channel start")?;

    write_text_element(&mut writer, "title", feed.title())?;

    if let Some(home_page_url) = feed.home_page_url() {
        ensure_http_or_ftp_uri(home_page_url)?;

        write_text_element(&mut writer, "link", home_page_url)?;
    }

    write_text_element(&mut writer, "description", feed.title())?;

    // Items (no explicit limit in 0.92).
    for entry in feed.entries() {
        writer
            .write(XmlEvent::start_element("item"))
            .context("writing item start")?;

        if !entry.title().is_empty() {
            write_text_element(&mut writer, "title", entry.title())?;
        }

        if let Some(url) = entry.url() {
            ensure_http_or_ftp_uri(url)?;

            write_text_element(&mut writer, "link", url)?;
        }

        if !entry.body().is_empty() {
            write_text_element(&mut writer, "description", entry.body())?;
        }

        writer
            .write(XmlEvent::end_element())
            .context("closing item element")?;
    }

    writer
        .write(XmlEvent::end_element())
        .context("closing channel element")?;

    writer
        .write(XmlEvent::end_element())
        .context("closing rss element")?;

    String::from_utf8(output).context("RSS 0.92 output is not valid UTF-8")
}
