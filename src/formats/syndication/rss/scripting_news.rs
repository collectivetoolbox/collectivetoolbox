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

//! `ScriptingNews` format renderer for the syndication data model.
//!
//! `ScriptingNews` is an XML syndication format predating RSS, first published
//! in December 1997.
//!
//! I'm not sure about the encoding or specific entities used, so I made a best
//! guess.
//!
//! This module implements:
//! - **`ScriptingNews` 1.0** (version string `1.0a2`): ASCII-only output using
//!   HTML 3.2 entities.
//! - **`ScriptingNews` 2.0** (version string `2.0b1`): ASCII-only output using
//!   Netscape 1999 entities, with additional header elements for parity with
//!   RSS 0.9.
//!
//! Specification for 2.0b1: <https://web.archive.org/web/19990902194349/http://my.userland.com/stories/storyReader$11>
//! DTD: <https://web.archive.org/web/19980131193200if_/http://www.scripting.com:80/dtd/scriptingNews.dtd>

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::Feed;
use chrono::Utc;
use ctb_formats_html::{to_entities_html32, to_entities_netscape_1999};
use ctb_formats_utf8::to_ascii_translit;

const TIME_FMT: &str = "%a, %d %b %Y %H:%M:%S GMT";

fn write_feed_dates_to_strings(feed: &Feed) -> (String, String) {
    // Use current date for pubDate and lastBuildDate if not available.
    // Format: "Fri, 26 Dec 1997 08:00:00 GMT"

    let now_date = Utc::now().format(TIME_FMT).to_string();

    let pub_date = if let Some(entry) = feed.entries().first() {
        entry.try_updated_try_format(TIME_FMT)
    } else {
        now_date.clone()
    };

    (pub_date, now_date)
}

fn push_open(out: &mut String, indent: usize, name: &str) {
    out.push_str(&"\t".repeat(indent));
    out.push('<');
    out.push_str(name);
    out.push_str(">\n");
}

fn push_close(out: &mut String, indent: usize, name: &str) {
    out.push_str(&"\t".repeat(indent));
    out.push_str("</");
    out.push_str(name);
    out.push_str(">\n");
}

fn push_text_element_raw(
    out: &mut String,
    indent: usize,
    name: &str,
    value: &str,
) {
    out.push_str(&"\t".repeat(indent));
    out.push('<');
    out.push_str(name);
    out.push('>');
    out.push_str(value);
    out.push_str("</");
    out.push_str(name);
    out.push_str(">\n");
}

// ============================================================================
// `ScriptingNews` 1.0 (version 1.0a2)
// ============================================================================

/// Prepare a string for `ScriptingNews` 1.0 (ASCII-only, HTML 3.2 entities).
fn prep_string_10(input: &str) -> Result<String> {
    let input = to_entities_html32(input.to_string())?;
    Ok(to_ascii_translit(&input))
}

/// Render a [`Feed`] into `ScriptingNews` 1.0 (version 1.0a2) format.
///
/// `ScriptingNews` 1.0 is an early XML syndication format with:
/// - A header containing copyright, version, dates, and docs URL
/// - Multiple items, each with text and a single link (url + linetext)
/// - ASCII-only output (non-ASCII replaced with `?`)
/// TODO: Make sure it is *not* escaping HTML syntax in item text, unlike 2.0.
pub fn render_scripting_news_10(feed: &Feed) -> Result<String> {
    let mut out = String::new();

    // Original format: uppercase XML declaration + external DTD.
    out.push_str(r#"<?XML VERSION="1.0"?>"#);
    out.push('\n');
    out.push_str(
        r#"<!DOCTYPE scriptingNews SYSTEM "http://www.scripting.com/dtd/scriptingNews.dtd">"#,
    );
    out.push('\n');

    push_open(&mut out, 0, "scriptingNews");
    push_open(&mut out, 1, "header");

    /* write_text_element(
        &mut writer,
        "copyright",
        &prep_string_10(&format!("Copyright {}", feed.title()))?,
    )?; */
    push_text_element_raw(&mut out, 2, "scriptingNewsVersion", "1.0a2");

    let (pub_date, last_build_date) = write_feed_dates_to_strings(feed);
    push_text_element_raw(&mut out, 2, "pubDate", &pub_date);
    push_text_element_raw(&mut out, 2, "lastBuildDate", &last_build_date);

    // Reason for fallback: docs element defaults to empty string if home_page_url is absent.
    let docs_url = feed.home_page_url().unwrap_or("");
    push_text_element_raw(&mut out, 2, "docs", &prep_string_10(docs_url)?);

    push_close(&mut out, 1, "header");

    for entry in feed.entries() {
        push_open(&mut out, 1, "item");

        // 1.0a2: do not escape embedded HTML markup inside `<text>`.
        // FIXME? This doesn't try to convert modern HTML to HTML 3.2 (guessing
        // that is the appropriate version to use).
        let text = to_ascii_translit(entry.body());
        push_text_element_raw(&mut out, 2, "text", &text);

        push_open(&mut out, 2, "link");

        // Reason for fallback: item URL falls back to item ID per scripting news specification.
        let url = entry.url().unwrap_or(entry.id());
        push_text_element_raw(&mut out, 3, "url", &prep_string_10(url)?);
        push_text_element_raw(
            &mut out,
            3,
            "linetext",
            &prep_string_10(entry.title())?,
        );

        push_close(&mut out, 2, "link");
        push_close(&mut out, 1, "item");
    }

    push_close(&mut out, 0, "scriptingNews");

    if !out.bytes().all(|b| b.is_ascii()) {
        bail!("ScriptingNews 1.0 output contains non-ASCII characters");
    }

    Ok(out)
}

// ============================================================================
// `ScriptingNews` 2.0 (version 2.0b1)
// ============================================================================

/// Prepare a string for `ScriptingNews` 2.0 (ASCII-only, Netscape 1999
/// entities).
fn prep_string_20(input: &str) -> Result<String> {
    let input = to_entities_netscape_1999(input.to_string())?;
    Ok(to_ascii_translit(&input))
}

/// Render a [`Feed`] into `ScriptingNews` 2.0 (version 2.0b1) format.
///
/// `ScriptingNews` 2.0 extends 1.0 with additional header elements for parity
/// with RSS 0.9:
/// - channelTitle, channelLink, channelDescription
/// - imageTitle, imageUrl, imageLink
/// - managingEditor, webmaster, language
/// - imageHeight, imageWidth, imageCaption
/// - skipHours, skipDays (not implemented here)
///
/// ASCII-only output (non-ASCII replaced with `?`).
pub fn render_scripting_news_20(feed: &Feed) -> Result<String> {
    let mut out = String::new();

    // Sample output uses no `encoding="..."` attribute.
    out.push_str(r#"<?xml version="1.0"?>"#);
    out.push('\n');

    push_open(&mut out, 0, "scriptingNews");
    push_open(&mut out, 1, "header");

    /* write_text_element(
        &mut writer,
        "copyright",
        &prep_string_20(&format!("Copyright {}", feed.title()))?,
    )?; */
    push_text_element_raw(&mut out, 2, "scriptingNewsVersion", "2.0b1");

    let (pub_date, last_build_date) = write_feed_dates_to_strings(feed);
    push_text_element_raw(&mut out, 2, "pubDate", &pub_date);
    push_text_element_raw(&mut out, 2, "lastBuildDate", &last_build_date);

    // Reason for fallback: docs element defaults to empty string if home_page_url is absent.
    let docs_url = feed.home_page_url().unwrap_or("");
    push_text_element_raw(&mut out, 2, "docs", &prep_string_20(docs_url)?);

    push_text_element_raw(
        &mut out,
        2,
        "channelDescription",
        &prep_string_20(feed.title())?,
    );

    // Reason for fallback: channelLink defaults to empty string if home_page_url is absent.
    let channel_link = feed.home_page_url().unwrap_or("");
    push_text_element_raw(
        &mut out,
        2,
        "channelLink",
        &prep_string_20(channel_link)?,
    );
    push_text_element_raw(
        &mut out,
        2,
        "channelTitle",
        &prep_string_20(feed.title())?,
    );

    push_text_element_raw(&mut out, 2, "language", "en-US");

    push_close(&mut out, 1, "header");

    for entry in feed.entries() {
        push_open(&mut out, 1, "item");

        // 2.0b1: `<text>` should be escaped (e.g. `<i>` -> `&lt;i&gt;`).
        push_text_element_raw(
            &mut out,
            2,
            "text",
            &prep_string_20(entry.body())?,
        );

        push_open(&mut out, 2, "link");

        // Reason for fallback: item URL falls back to item ID per scripting news specification.
        let url = entry.url().unwrap_or(entry.id());
        push_text_element_raw(&mut out, 3, "url", &prep_string_20(url)?);
        push_text_element_raw(
            &mut out,
            3,
            "linetext",
            &prep_string_20(entry.title())?,
        );

        push_close(&mut out, 2, "link");
        push_close(&mut out, 1, "item");
    }

    push_close(&mut out, 0, "scriptingNews");

    if !out.bytes().all(|b| b.is_ascii()) {
        bail!("ScriptingNews 2.0 output contains non-ASCII characters");
    }

    Ok(out)
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
    use crate::Entry;

    // -------------------------------------------------------------------------
    // ScriptingNews 1.0 tests
    // -------------------------------------------------------------------------

    #[crate::ctb_test]
    fn test_scripting_news_10_basic() {
        let entry = Entry::new(
            "http://example.org/item1",
            "Test Item",
            "The example content. Lorem ipsum dolor sit amet.",
            "2024-01-15T10:00:00Z",
        )
        .with_url_opt(Some("http://example.org/item1".into()));

        let feed = Feed::new("Test Feed", vec![entry])
            .with_home_page_url_opt(Some("http://example.org/".into()));

        let result = render_scripting_news_10(&feed).unwrap();

        // Check XML declaration (uppercase VERSION per original format).
        assert!(result.contains(r#"<?XML VERSION="1.0"?>"#));
        // Check DOCTYPE.
        assert!(result.contains(
            r#"<!DOCTYPE scriptingNews SYSTEM "http://www.scripting.com/dtd/scriptingNews.dtd">"#
        ));
        // Check structure.
        assert!(result.contains("<scriptingNews>"));
        assert!(result.contains("<header>"));
        assert!(
            result
                .contains("<scriptingNewsVersion>1.0a2</scriptingNewsVersion>")
        );
        assert!(result.contains("<item>"));
        assert!(result.contains(
            "<text>The example content. Lorem ipsum dolor sit amet.</text>"
        ));
        assert!(result.contains("<link>"));
        assert!(result.contains("<url>http://example.org/item1</url>"));
        assert!(result.contains("<linetext>Test Item</linetext>"));
        assert!(result.contains("</scriptingNews>"));
    }

    #[crate::ctb_test]
    fn test_scripting_news_10_ascii_only() {
        let entry = Entry::new(
            "http://example.org/item1",
            "Tëst with ümlauts",
            "Cöntent with spëcial chärs",
            "2024-01-15",
        )
        .with_url_opt(Some("http://example.org/item1".into()));

        let feed = Feed::new("Fëed with spëcial chars", vec![entry])
            .with_home_page_url_opt(Some("http://example.org/".into()));

        let result = render_scripting_news_10(&feed).unwrap();

        // Non-ASCII should be replaced with `?` or transliterated.
        assert!(result.bytes().all(|b| b.is_ascii()));
        // Should contain transliterated or entity-replaced versions.
        assert!(!result.contains("ë"));
        assert!(!result.contains("ü"));
        assert!(!result.contains("ö"));
        assert!(!result.contains("ä"));
    }

    #[crate::ctb_test]
    fn test_scripting_news_10_html_in_text() {
        let entry = Entry::new(
            "http://example.org/item1",
            "Item with HTML",
            "Content with <i>italic</i> and <b>bold</b>",
            "2024-01-15",
        )
        .with_url_opt(Some("http://example.org/item1".into()));

        let feed = Feed::new("Test Feed", vec![entry])
            .with_home_page_url_opt(Some("http://example.org/".into()));

        let result = render_scripting_news_10(&feed).unwrap();

        // HTML tags should probably be escaped in XML. However, in practice, it
        // doesn't seem to be in ScriptingNews 1.0a2 based on a sample file.
        // ScriptingNews 2.0b1 does escape them.
        assert!(result.contains("<i>italic</i>"), "Got: '{result}'");
        assert!(result.contains("<b>bold</b>"), "Got: '{result}'");
    }

    #[crate::ctb_test]
    fn test_scripting_news_10_multiple_items() {
        let entries: Vec<_> = (0..5)
            .map(|i| {
                Entry::new(
                    format!("http://example.org/item{i}"),
                    format!("Item {i}"),
                    format!("Content for item {i}"),
                    "",
                )
                .with_url_opt(Some(format!("http://example.org/item{i}")))
            })
            .collect();

        let feed = Feed::new("Multi-Item Feed", entries)
            .with_home_page_url_opt(Some("http://example.org/".into()));

        let result = render_scripting_news_10(&feed).unwrap();

        assert!(result.contains("Item 0"));
        assert!(result.contains("Item 4"));
        assert!(result.contains("Content for item 2"));
    }

    // -------------------------------------------------------------------------
    // ScriptingNews 2.0 tests
    // -------------------------------------------------------------------------

    #[crate::ctb_test]
    fn test_scripting_news_20_basic() {
        let entry = Entry::new(
            "http://example.org/item1",
            "Test Item",
            "The example content. Lorem ipsum dolor sit amet.",
            "2024-01-15T10:00:00Z",
        )
        .with_url_opt(Some("http://example.org/item1".into()));

        let feed = Feed::new("Test Feed", vec![entry])
            .with_home_page_url_opt(Some("http://example.org/".into()));

        let result = render_scripting_news_20(&feed).unwrap();

        // Check XML declaration (lowercase, standard).
        assert!(
            result.contains(r#"<?xml version="1.0"?>"#),
            "Got: '{result}'"
        );
        // Check structure.
        assert!(result.contains("<scriptingNews>"), "Got: '{result}'");
        assert!(result.contains("<header>"), "Got: '{result}'");
        assert!(
            result
                .contains("<scriptingNewsVersion>2.0b1</scriptingNewsVersion>"),
            "Got: '{result}'"
        );

        // Check 2.0-specific header elements.
        assert!(
            result.contains("<channelTitle>Test Feed</channelTitle>"),
            "Got: '{result}'"
        );
        assert!(
            result.contains("<channelLink>http://example.org/</channelLink>"),
            "Got: '{result}'"
        );
        assert!(
            result
                .contains("<channelDescription>Test Feed</channelDescription>"),
            "Got: '{result}'"
        );
        assert!(
            result.contains("<language>en-US</language>"),
            "Got: '{result}'"
        );

        // Check item structure.
        assert!(result.contains("<item>"), "Got: '{result}'");
        assert!(
            result.contains(
                "<text>The example content. Lorem ipsum dolor sit amet.</text>"
            ),
            "Got: '{result}'"
        );
        assert!(result.contains("<link>"), "Got: '{result}'");
        assert!(
            result.contains("<url>http://example.org/item1</url>"),
            "Got: '{result}'"
        );
        assert!(
            result.contains("<linetext>Test Item</linetext>"),
            "Got: '{result}'"
        );
        assert!(result.contains("</scriptingNews>"), "Got: '{result}'");
    }

    #[crate::ctb_test]
    fn test_scripting_news_20_ascii_only() {
        let entry = Entry::new(
            "http://example.org/item1",
            "Tëst with ümlauts",
            "Cöntent with spëcial chärs",
            "2024-01-15",
        )
        .with_url_opt(Some("http://example.org/item1".into()));

        let feed = Feed::new("Fëed with spëcial chars", vec![entry])
            .with_home_page_url_opt(Some("http://example.org/".into()));

        let result = render_scripting_news_20(&feed).unwrap();

        // Non-ASCII should be replaced with `?` or transliterated.
        assert!(result.bytes().all(|b| b.is_ascii()));
        // Should not contain raw non-ASCII characters.
        assert!(!result.contains("ë"));
        assert!(!result.contains("ü"));
        assert!(!result.contains("ö"));
        assert!(!result.contains("ä"));
    }

    #[crate::ctb_test]
    fn test_scripting_news_20_no_doctype() {
        let feed = Feed::empty("Test Feed")
            .with_home_page_url_opt(Some("http://example.org/".into()));

        let result = render_scripting_news_20(&feed).unwrap();

        // Version 2.0 should NOT have a DOCTYPE declaration (per the sample).
        assert!(!result.contains("<!DOCTYPE"), "Got: '{result}'");
    }

    #[crate::ctb_test]
    fn test_scripting_news_20_html_in_text() {
        let entry = Entry::new(
            "http://example.org/item1",
            "Item with HTML",
            "Content with <i>italic</i> text",
            "2024-01-15",
        )
        .with_url_opt(Some("http://example.org/item1".into()));

        let feed = Feed::new("Test Feed", vec![entry])
            .with_home_page_url_opt(Some("http://example.org/".into()));

        let result = render_scripting_news_20(&feed).unwrap();

        // HTML tags should be escaped in XML.
        assert!(
            result.contains("&lt;i&gt;italic&lt;/i&gt;"),
            "Got: '{result}'"
        );
    }

    #[crate::ctb_test]
    fn test_scripting_news_20_multiple_items() {
        let entries: Vec<_> = (0..3)
            .map(|i| {
                Entry::new(
                    format!("http://example.org/item{i}"),
                    format!("Item {i}"),
                    format!("Content for item {i}"),
                    "",
                )
                .with_url_opt(Some(format!("http://example.org/item{i}")))
            })
            .collect();

        let feed = Feed::new("Multi-Item Feed", entries)
            .with_home_page_url_opt(Some("http://example.org/".into()));

        let result = render_scripting_news_20(&feed).unwrap();

        assert!(result.contains("Item 0"));
        assert!(result.contains("Item 2"));
        assert!(result.contains("Content for item 1"));
    }

    #[crate::ctb_test]
    fn test_scripting_news_10_vs_20_version_strings() {
        let feed = Feed::empty("Test")
            .with_home_page_url_opt(Some("http://example.org/".into()));

        let result_10 = render_scripting_news_10(&feed).unwrap();
        let result_20 = render_scripting_news_20(&feed).unwrap();

        assert!(
            result_10
                .contains("<scriptingNewsVersion>1.0a2</scriptingNewsVersion>"),
            "Got: '{result_10}'"
        );
        assert!(
            result_20
                .contains("<scriptingNewsVersion>2.0b1</scriptingNewsVersion>"),
            "Got: '{result_20}'"
        );
    }

    #[crate::ctb_test]
    fn test_scripting_news_20_has_channel_elements() {
        let feed = Feed::new("My Channel", vec![])
            .with_home_page_url_opt(Some("http://example.org/".into()));

        let result_10 = render_scripting_news_10(&feed).unwrap();
        let result_20 = render_scripting_news_20(&feed).unwrap();

        // Version 1.0 should NOT have channel* elements.
        assert!(!result_10.contains("<channelTitle>"), "Got: '{result_10}'");
        assert!(!result_10.contains("<channelLink>"), "Got: '{result_10}'");
        assert!(
            !result_10.contains("<channelDescription>"),
            "Got: '{result_10}'"
        );

        // Version 2.0 SHOULD have channel* elements.
        assert!(
            result_20.contains("<channelTitle>My Channel</channelTitle>"),
            "Got: '{result_20}'"
        );
        assert!(
            result_20
                .contains("<channelLink>http://example.org/</channelLink>"),
            "Got: '{result_20}'"
        );
        assert!(
            result_20.contains(
                "<channelDescription>My Channel</channelDescription>"
            ),
            "Got: '{result_20}'"
        );
    }
}
