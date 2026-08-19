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
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::Feed;
use crate::rss::{
    RSS_09_091_MAX_ITEMS, ensure_http_or_ftp_uri, write_text_element,
};
use ctb_formats_utf8::ellipsize_to_max_bytes;
use ctb_utilities::string::remove_line;
use xml::writer::{EmitterConfig, XmlEvent};

const RSS_091_IMAGE_MAX_WIDTH: u32 = 144;
const RSS_091_IMAGE_MAX_HEIGHT: u32 = 400;

// ============================================================================
// RSS 0.91
// ============================================================================

/// RSS 0.91 rendering flavor.
#[derive(Debug, Clone, Copy)]
enum Rss091Flavor {
    /// Netscape RSS 0.91: requires a DOCTYPE declaration.
    Netscape,
    /// `UserLand` RSS 0.91: typically published without a DOCTYPE.
    UserLand,
}

/// Render a [`Feed`] into RSS 0.91 (Netscape) format.
///
/// This variant includes the required `<!DOCTYPE rss ...>` declaration and
/// enforces the RSS 0.91 URL constraints (`http://` or `ftp://`).
///
/// Constraints enforced:
/// - Maximum 15 items per channel
/// - Title: max 100 chars, Description: max 500 chars, Link: max 500 chars
/// - Item title: max 100 chars, Item link: max 500 chars
/// - Item description: max 500 chars
///
/// Specifications:
/// - <https://www.rssboard.org/rss-0-9-1> (UserLand variant)
/// - <https://www.rssboard.org/rss-0-9-1-netscape> (Netscape variant)
pub fn render_rss_091_netscape(feed: &Feed) -> Result<String> {
    render_rss_091(feed, Rss091Flavor::Netscape)
}

/// Render a [`Feed`] into RSS 0.91 (`UserLand`) format.
///
/// This variant omits the DOCTYPE declaration and enforces the RSS 0.91 URL
/// constraints (`http://` or `ftp://`).
///
/// Constraints enforced:
/// - Maximum 15 items per channel
/// - Title: max 100 chars, Description: max 500 chars, Link: max 500 chars
/// - Item title: max 100 chars, Item link: max 500 chars
/// - Item description: max 500 chars
pub fn render_rss_091_userland(feed: &Feed) -> Result<String> {
    render_rss_091(feed, Rss091Flavor::UserLand)
}

fn write_rss_091_userland_channel_image<W: std::io::Write>(
    writer: &mut xml::writer::EventWriter<W>,
    feed: &Feed,
    channel_link: &str,
) -> Result<()> {
    let Some(image) = feed.image() else {
        warn!(
            "RSS 0.91 (UserLand) requires channel <image>, but none provided"
        );
        return Ok(());
    };

    /*let Some(url) = image.url() else {
        warn!(
            "RSS 0.91 (UserLand) channel <image> missing required <url>; skipping <image>"
        );
        return Ok(());
    };*/
    let url = image.url();

    if url.len() > 500 {
        warn_fmt!(
            "RSS 0.91 (UserLand) channel URL is too long; skipping <image>"
        );
        return Ok(());
    }

    if let Err(err) = ensure_http_or_ftp_uri(url) {
        warn_fmt!(
            "RSS 0.91 (UserLand) channel <image><url> violates URL constraints (http/ftp only): {err}; skipping <image>"
        );
        return Ok(());
    }

    writer
        .write(XmlEvent::start_element("image"))
        .context("writing image start")?;

    let title = match image.alt() {
        Some(alt) if !alt.is_empty() => alt.to_string(),
        _ => format!("{} Channel Image", feed.title()),
    };
    write_text_element(writer, "title", &ellipsize_to_max_bytes(&title, 100))?;

    write_text_element(writer, "url", &ellipsize_to_max_bytes(url, 500))?;

    // RSS 0.91 requires <link> inside <image>. Spec says that in practice it
    // should match the channel link.
    write_text_element(
        writer,
        "link",
        &ellipsize_to_max_bytes(channel_link, 500),
    )?;

    if let Some(width) = image.width() {
        let width_clamped = if width > RSS_091_IMAGE_MAX_WIDTH {
            warn!(
                "RSS 0.91 (UserLand) channel <image><width> exceeds max {}; clamping",
                RSS_091_IMAGE_MAX_WIDTH
            );
            RSS_091_IMAGE_MAX_WIDTH
        } else {
            width
        };
        write_text_element(writer, "width", &width_clamped.to_string())?;
    }

    if let Some(height) = image.height() {
        let height_clamped = if height > RSS_091_IMAGE_MAX_HEIGHT {
            warn!(
                "RSS 0.91 (UserLand) channel <image><height> exceeds max {}; clamping",
                RSS_091_IMAGE_MAX_HEIGHT
            );
            RSS_091_IMAGE_MAX_HEIGHT
        } else {
            height
        };
        write_text_element(writer, "height", &height_clamped.to_string())?;
    }

    if let Some(description) = image.description() {
        if !description.is_empty() {
            write_text_element(
                writer,
                "description",
                &ellipsize_to_max_bytes(description, 500),
            )?;
        }
    }

    writer
        .write(XmlEvent::end_element())
        .context("closing image element")?;

    Ok(())
}

fn render_rss_091(feed: &Feed, flavor: Rss091Flavor) -> Result<String> {
    let mut output = Vec::new();

    // Write the XML declaration explicitly so we can inject a DOCTYPE for
    // the Netscape variant before the root element.
    output.extend_from_slice(b"\n");

    if matches!(flavor, Rss091Flavor::Netscape) {
        output.extend_from_slice(br#"<?xml version="1.0"?>"#);
        output.extend_from_slice(
            br#"<!DOCTYPE rss SYSTEM "http://my.netscape.com/publish/formats/rss-0.91.dtd">"#,
        );
    } else {
        output.extend_from_slice(br#"<?xml version="1.0" encoding="UTF-8"?>"#);
    }
    output.extend_from_slice(b"\n");

    let config = EmitterConfig::new()
        .perform_indent(true)
        .indent_string("\t");
    let mut writer = config.create_writer(&mut output);

    // Start rss element with version.
    writer
        .write(XmlEvent::start_element("rss").attr("version", "0.91"))
        .context("writing rss start element")?;

    // Channel element.
    writer
        .write(XmlEvent::start_element("channel"))
        .context("writing channel start")?;

    write_text_element(
        &mut writer,
        "title",
        &ellipsize_to_max_bytes(feed.title(), 100),
    )?;

    let Some(link) = feed.home_page_url() else {
        bail!("RSS 0.91 requires a channel <link> URL");
    };
    ensure_http_or_ftp_uri(link)?;
    write_text_element(
        &mut writer,
        "link",
        &ellipsize_to_max_bytes(link, 500),
    )?;

    write_text_element(
        &mut writer,
        "description",
        &ellipsize_to_max_bytes(feed.title(), 500),
    )?;

    // Language is required for 0.91. FIXME: This shouldn't be hardcoded. It's
    // not totally trivial to implement non-hardcoded since the list of valid
    // language codes would need to be taken into consideration.
    write_text_element(&mut writer, "language", "en-us")?;

    if matches!(flavor, Rss091Flavor::UserLand) {
        write_rss_091_userland_channel_image(&mut writer, feed, link)?;
    }

    // Items (max 15).
    let items = feed.entries().iter().take(RSS_09_091_MAX_ITEMS);
    for entry in items {
        writer
            .write(XmlEvent::start_element("item"))
            .context("writing item start")?;

        write_text_element(
            &mut writer,
            "title",
            &ellipsize_to_max_bytes(entry.title(), 100),
        )?;

        // Reason for fallback: per RSS 0.91 specification, feed item link falls back to item ID if explicit URL is absent.
        let item_link = entry.url().unwrap_or(entry.id());
        ensure_http_or_ftp_uri(item_link)?;
        write_text_element(
            &mut writer,
            "link",
            &ellipsize_to_max_bytes(item_link, 500),
        )?;

        // Description (XML escapes markup; RSS 0.91 consumers may impose
        // additional restrictions beyond well-formed XML).
        if !entry.body().is_empty() {
            write_text_element(
                &mut writer,
                "description",
                &ellipsize_to_max_bytes(entry.body(), 500),
            )?;
        }

        writer
            .write(XmlEvent::end_element())
            .context("closing item element")?;
    }

    // Close channel.
    writer
        .write(XmlEvent::end_element())
        .context("closing channel element")?;

    // Close rss.
    writer
        .write(XmlEvent::end_element())
        .context("closing rss element")?;

    // Prevent duplicate XML declaration: remove 2nd line for UserLand variant,
    // and 3rd line for Netscape variant.
    let result = String::from_utf8(output)
        .context("RSS 0.91 output is not valid UTF-8")?;
    Ok(if matches!(flavor, Rss091Flavor::Netscape) {
        remove_line(&result, 2) // remove 3rd line
    } else {
        remove_line(&result, 1) // remove 2nd line
    })
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
    use crate::Image;

    // -------------------------------------------------------------------------
    // RSS 0.91 tests
    // -------------------------------------------------------------------------

    #[crate::ctb_test]
    fn test_rss_091_netscape_basic() -> Result<()> {
        let entry = Entry::new(
            "http://example.org/item1",
            "Test Item",
            "<p>Description</p>",
            "2024-01-15",
        )
        .with_url_opt(Some("http://example.org/item1".into()));

        let feed = Feed::new("Test Feed", vec![entry])
            .with_home_page_url_opt(Some("http://example.org/".into()));

        let result = render_rss_091_netscape(&feed)?;

        assert!(
            result.contains(r#"<?xml version="1.0"?>"#),
            "Got: '{result}'"
        );
        assert!(result.contains(r#"<!DOCTYPE rss SYSTEM "http://my.netscape.com/publish/formats/rss-0.91.dtd">"#), "Got: '{result}'");
        assert!(
            result.contains(r#"<rss version="0.91">"#),
            "Got: '{result}'"
        );
        assert!(result.contains("<channel>"), "Got: '{result}'");
        assert!(
            result.contains("<language>en-us</language>"),
            "Got: '{result}'"
        );
        assert!(
            result.contains("<title>Test Item</title>"),
            "Got: '{result}'"
        );
        assert!(result.contains("</rss>"), "Got: '{result}'");

        Ok(())
    }

    #[crate::ctb_test]
    fn test_rss_091_userland_basic() -> Result<()> {
        let entry = Entry::new("http://example.org/item1", "Test Item", "", "")
            .with_url_opt(Some("http://example.org/item1".into()));

        let feed = Feed::new("Test Feed", vec![entry])
            .with_home_page_url_opt(Some("http://example.org/".into()));

        let result = render_rss_091_userland(&feed)?;

        assert!(result.contains(r#"<?xml version="1.0" encoding="UTF-8"?>"#));
        assert!(!result.contains("<!DOCTYPE rss"));
        assert!(result.contains(r#"<rss version="0.91">"#));

        Ok(())
    }

    #[crate::ctb_test]
    fn test_rss_091_userland_with_content() {
        let entry = Entry::new(
            "http://example.org/item1",
            "Test Item",
            "<p>Description</p>",
            "2024-01-15",
        )
        .with_url_opt(Some("http://example.org/item1".into()));

        let feed = Feed::new("Test Feed", vec![entry])
            .with_home_page_url_opt(Some("http://example.org/".into()));

        let result = render_rss_091_userland(&feed).unwrap();

        assert!(
            result.contains("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"),
            "Got: '{result}'"
        );
        assert!(result.contains("<rss version=\"0.91\">"), "Got: '{result}'");
        assert!(result.contains("<channel>"), "Got: '{result}'");
        assert!(
            result.contains("<language>en-us</language>"),
            "Got: '{result}'"
        );
        assert!(
            result.contains("<title>Test Item</title>"),
            "Got: '{result}'"
        );
        assert!(result.contains("<description>"), "Got: '{result}'");
        assert!(result.contains("</rss>"), "Got: '{result}'");
    }

    #[crate::ctb_test]
    fn test_rss_091_max_15_items() -> Result<()> {
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

        let result = render_rss_091_userland(&feed)?;

        assert!(result.contains("Item 14"));
        assert!(!result.contains("Item 15"));

        Ok(())
    }

    #[crate::ctb_test]
    fn test_rss_091_url_constraints_reject_https() -> Result<()> {
        let entry =
            Entry::new("https://example.org/item1", "Test Item", "", "")
                .with_url_opt(Some("https://example.org/item1".into()));

        let feed = Feed::new("Test Feed", vec![entry])
            .with_home_page_url_opt(Some("https://example.org/".into()));

        match render_rss_091_userland(&feed) {
            Ok(_) => bail!("expected URL constraint error for https://"),
            Err(_) => Ok(()),
        }
    }

    #[crate::ctb_test]
    fn test_rss_091_requires_channel_link() -> Result<()> {
        let entry = Entry::new("http://example.org/item1", "Test Item", "", "")
            .with_url_opt(Some("http://example.org/item1".into()));

        let feed = Feed::new("Test Feed", vec![entry]);

        match render_rss_091_netscape(&feed) {
            Ok(_) => bail!("expected missing channel <link> error"),
            Err(_) => Ok(()),
        }
    }

    #[crate::ctb_test]
    fn test_rss_091_utf8() -> Result<()> {
        let entry = Entry::new(
            "http://example.org/item1",
            "日本語タイトル",
            "日本語の説明",
            "",
        )
        .with_url_opt(Some("http://example.org/item1".into()));

        let feed = Feed::new("日本語フィード", vec![entry])
            .with_home_page_url_opt(Some("http://example.org/".into()));

        let result = render_rss_091_userland(&feed)?;

        assert!(result.contains("日本語タイトル"));
        assert!(result.contains("日本語フィード"));

        Ok(())
    }

    #[crate::ctb_test]
    fn test_rss_091_userland_channel_image_best_effort() -> Result<()> {
        let entry = Entry::new("http://example.org/item1", "Test Item", "", "")
            .with_url_opt(Some("http://example.org/item1".into()));

        let feed = Feed::new("Test Feed", vec![entry])
            .with_home_page_url_opt(Some("http://example.org/".into()))
            // Omit alt/title on purpose; renderer should derive it from
            // feed.title() + " Channel Image".
            .with_image(
                Image::new()
                    .with_url("http://example.org/image.png".into())
                    .with_width_opt(Some(200))
                    .with_height_opt(Some(600))
                    .with_description_opt(Some("Image description".into())),
            );

        let result = render_rss_091_userland(&feed)?;

        assert!(result.contains("<image>"), "Got: '{result}'");
        assert!(
            result.contains("<title>Test Feed Channel Image</title>"),
            "Got: '{result}'"
        );
        assert!(
            result.contains("<url>http://example.org/image.png</url>"),
            "Got: '{result}'"
        );
        assert!(
            result.contains("<link>http://example.org/</link>"),
            "Got: '{result}'"
        );
        // Width/height should be clamped to the RSS 0.91 limits.
        assert!(result.contains("<width>144</width>"), "Got: '{result}'");
        assert!(result.contains("<height>400</height>"), "Got: '{result}'");
        assert!(
            result.contains("<description>Image description</description>"),
            "Got: '{result}'"
        );

        Ok(())
    }
}
