#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::Feed;
use crate::rss::{
    RDF_NAMESPACE, RSS_10_NAMESPACE, ensure_http_https_or_ftp_uri,
    write_text_element,
};
use xml::writer::{EmitterConfig, XmlEvent};

// ============================================================================
// RSS 1.0 (RDF-based)
// ============================================================================

/// Render a [`Feed`] into RSS 1.0 (RDF) format.
///
/// RSS 1.0 uses RDF and requires `rdf:about` attributes and an items sequence.
///
/// Specification: <https://web.resource.org/rss/1.0/spec>
pub fn render_rss_10(feed: &Feed) -> Result<String> {
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

    let channel_url = feed.feed_url().or(feed.home_page_url());

    if let Some(channel_url) = channel_url {
        ensure_http_https_or_ftp_uri(channel_url)?;
    } else {
        // RSS 1.0 requires a channel link.
        bail!("RSS 1.0 requires a channel <link> URL");
    }

    let channel_url = bail_if_none!(channel_url);

    // Start rdf:RDF with namespaces.
    writer
        .write(
            XmlEvent::start_element("rdf:RDF")
                .ns("rdf", RDF_NAMESPACE)
                .default_ns(RSS_10_NAMESPACE),
        )
        .context("writing rdf:RDF start element")?;

    // Channel element with rdf:about.
    writer
        .write(
            XmlEvent::start_element("channel").attr("rdf:about", channel_url),
        )
        .context("writing channel start")?;

    write_text_element(&mut writer, "title", feed.title())?;

    if let Some(home_page_url) = feed.home_page_url() {
        ensure_http_https_or_ftp_uri(home_page_url)?;
    } else {
        // RSS 1.0 requires a channel link.
        bail!("RSS 1.0 requires a channel <link> URL");
    }

    let link = bail_if_none!(feed.home_page_url());
    write_text_element(&mut writer, "link", link)?;

    write_text_element(&mut writer, "description", feed.title())?;

    // Items sequence.
    writer
        .write(XmlEvent::start_element("items"))
        .context("writing items start")?;

    writer
        .write(XmlEvent::start_element("rdf:Seq"))
        .context("writing rdf:Seq start")?;

    for entry in feed.entries() {
        // Reason for fallback: per RSS 1.0 specification, item resource URL falls back to item ID if explicit URL is absent.
        let item_url = entry.url().unwrap_or(entry.id());
        writer
            .write(XmlEvent::start_element("rdf:li").attr("resource", item_url))
            .context("writing rdf:li")?;
        writer
            .write(XmlEvent::end_element())
            .context("closing rdf:li")?;
    }

    writer
        .write(XmlEvent::end_element())
        .context("closing rdf:Seq")?;

    writer
        .write(XmlEvent::end_element())
        .context("closing items")?;

    writer
        .write(XmlEvent::end_element())
        .context("closing channel")?;

    // Item elements (outside channel, at rdf:RDF level).
    for entry in feed.entries() {
        if let Some(item_url) = entry.url() {
            ensure_http_https_or_ftp_uri(item_url)?;
        }

        // Reason for fallback: per RSS 1.0 specification, item resource URL falls back to item ID if explicit URL is absent.
        let item_url = entry.url().unwrap_or(entry.id());

        writer
            .write(XmlEvent::start_element("item").attr("rdf:about", item_url))
            .context("writing item start")?;

        write_text_element(&mut writer, "title", entry.title())?;
        write_text_element(&mut writer, "link", item_url)?;

        if !entry.body().is_empty() {
            write_text_element(&mut writer, "description", entry.body())?;
        }

        writer
            .write(XmlEvent::end_element())
            .context("closing item")?;
    }

    writer
        .write(XmlEvent::end_element())
        .context("closing rdf:RDF")?;

    String::from_utf8(output).context("RSS 1.0 output is not valid UTF-8")
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
    // RSS 1.0 tests
    // -------------------------------------------------------------------------

    #[crate::ctb_test]
    fn test_rss_10_basic() {
        let entry = Entry::new(
            "http://example.org/item1",
            "Test Item",
            "<p>Description</p>",
            "2024-01-15",
        )
        .with_url_opt(Some("http://example.org/item1".into()));

        let feed = Feed::new("Test Feed", vec![entry])
            .with_home_page_url_opt(Some("http://example.org/".into()))
            .with_feed_url_opt(Some("http://example.org/feed.rss".into()));

        let result = render_rss_10(&feed).unwrap();

        assert!(result.contains("xmlns=\"http://purl.org/rss/1.0/\""));
        assert!(result.contains("xmlns:rdf="));
        assert!(result.contains("rdf:about=\"http://example.org/feed.rss\""));
        assert!(result.contains("<rdf:Seq>"));
        assert!(
            result.contains("<rdf:li resource=\"http://example.org/item1\"")
        );
        assert!(
            result.contains("<item rdf:about=\"http://example.org/item1\">")
        );
    }

    #[crate::ctb_test]
    fn test_rss_10_items_in_sequence() {
        let entries: Vec<_> = (0..3)
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

        let feed = Feed::new("Test Feed", entries)
            .with_home_page_url_opt(Some("http://example.org/".into()));

        let result = render_rss_10(&feed).unwrap();

        // All items should be in the sequence.
        assert!(result.contains("resource=\"http://example.org/item0\""));
        assert!(result.contains("resource=\"http://example.org/item1\""));
        assert!(result.contains("resource=\"http://example.org/item2\""));

        // All items should also be as standalone item elements.
        assert!(result.contains("rdf:about=\"http://example.org/item0\""));
        assert!(result.contains("rdf:about=\"http://example.org/item1\""));
        assert!(result.contains("rdf:about=\"http://example.org/item2\""));
    }
}
