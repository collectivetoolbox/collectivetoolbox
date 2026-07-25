//! Atom 1.0 (RFC 4287) renderer for the syndication data model.
//!
//! Specification: <https://www.rfc-editor.org/rfc/rfc4287>

#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

use super::{Entry, Feed};
use xml::writer::{EmitterConfig, XmlEvent};

const ATOM_NAMESPACE: &str = "http://www.w3.org/2005/Atom";

/// Render a [`Feed`] into Atom 1.0 XML.
///
/// This produces a minimal valid Atom feed, including the required `id`,
/// `title`, and `updated` elements. Entries include required fields plus
/// optional content.
pub(crate) fn render(feed: &Feed) -> Result<String> {
    let mut output = Vec::new();
    let config = EmitterConfig::new()
        .perform_indent(true)
        .indent_string("\t");
    let mut writer = config.create_writer(&mut output);

    // Start feed element with namespace.
    writer
        .write(XmlEvent::start_element("feed").default_ns(ATOM_NAMESPACE))
        .context("writing feed start element")?;

    // Required: title
    write_text_element(&mut writer, "title", feed.title())?;

    // Required: id (use feed_url or home_page_url or a generated URN)
    let uuid = format!("urn:uuid:{}", uuid::Uuid::new_v4());
    let feed_id = feed.feed_url().or(feed.home_page_url()).unwrap_or(&uuid);
    write_text_element(&mut writer, "id", feed_id)?;

    // Required: updated (use latest entry date or a placeholder)
    let updated = feed
        .entries()
        .iter()
        .filter_map(|e| {
            let d = e.date();
            if d.is_empty() { None } else { Some(d) }
        })
        .max()
        .unwrap_or("1970-01-01T00:00:00Z");
    write_text_element(&mut writer, "updated", updated)?;

    // Optional: link rel="self"
    if let Some(feed_url) = feed.feed_url() {
        writer
            .write(
                XmlEvent::start_element("link")
                    .attr("href", feed_url)
                    .attr("rel", "self"),
            )
            .context("writing self link start")?;
        writer
            .write(XmlEvent::end_element())
            .context("writing self link end")?;
    }

    // Optional: link rel="alternate" for home page
    if let Some(home_url) = feed.home_page_url() {
        writer
            .write(XmlEvent::start_element("link").attr("href", home_url))
            .context("writing alternate link start")?;
        writer
            .write(XmlEvent::end_element())
            .context("writing alternate link end")?;
    }

    // Entries
    for entry in feed.entries() {
        write_entry(&mut writer, entry)?;
    }

    // Close feed element
    writer
        .write(XmlEvent::end_element())
        .context("closing feed element")?;

    String::from_utf8(output).context("Atom output is not valid UTF-8")
}

/// Write an entry element to the Atom feed.
fn write_entry<W: std::io::Write>(
    writer: &mut xml::writer::EventWriter<W>,
    entry: &Entry,
) -> Result<()> {
    writer
        .write(XmlEvent::start_element("entry"))
        .context("writing entry start")?;

    // Required: title
    write_text_element(writer, "title", entry.title())?;

    // Required: id
    write_text_element(writer, "id", entry.id())?;

    // Required: updated (use published date as fallback)
    let updated = if entry.date().is_empty() {
        "1970-01-01T00:00:00Z"
    } else {
        entry.date()
    };
    write_text_element(writer, "updated", updated)?;

    // Optional: published
    if !entry.date().is_empty() {
        write_text_element(writer, "published", entry.date())?;
    }

    // Optional: link
    if let Some(url) = entry.url() {
        writer
            .write(XmlEvent::start_element("link").attr("href", url))
            .context("writing entry link start")?;
        writer
            .write(XmlEvent::end_element())
            .context("writing entry link end")?;
    }

    // Optional: author
    if let Some(author_name) = entry.author() {
        writer
            .write(XmlEvent::start_element("author"))
            .context("writing author start")?;
        write_text_element(writer, "name", author_name)?;
        writer
            .write(XmlEvent::end_element())
            .context("writing author end")?;
    }

    // Optional: content (HTML)
    if !entry.body().is_empty() {
        writer
            .write(XmlEvent::start_element("content").attr("type", "html"))
            .context("writing content start")?;
        writer
            .write(XmlEvent::characters(entry.body()))
            .context("writing content body")?;
        writer
            .write(XmlEvent::end_element())
            .context("writing content end")?;
    }

    writer
        .write(XmlEvent::end_element())
        .context("closing entry element")?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[crate::ctb_test]
    fn test_render_empty_feed() {
        let feed = Feed::empty("Empty Test Feed");
        let result = render(&feed).unwrap();

        assert!(
            result.contains(r#"<feed xmlns="http://www.w3.org/2005/Atom">"#)
        );
        assert!(result.contains("<title>Empty Test Feed</title>"));
        assert!(result.contains("<id>"));
        assert!(result.contains("<updated>"));
        assert!(result.contains("</feed>"));
    }

    #[crate::ctb_test]
    fn test_render_feed_with_urls() {
        let feed = Feed::empty("Test Feed")
            .with_home_page_url_opt(Some("https://example.org/".into()))
            .with_feed_url_opt(Some("https://example.org/feed.atom".into()));
        let result = render(&feed).unwrap();

        assert!(result.contains(
            r#"<link href="https://example.org/feed.atom" rel="self" />"#
        ));
        assert!(result.contains(r#"<link href="https://example.org/" />"#));
        // feed_url should be used as the id
        assert!(result.contains("<id>https://example.org/feed.atom</id>"));
    }

    #[crate::ctb_test]
    fn test_render_feed_with_entry() {
        let entry = Entry::new(
            "entry-1",
            "Test Entry",
            "<p>Test content</p>",
            "2024-01-15T10:30:00Z",
        )
        .with_author("Test Author")
        .with_url_opt(Some("https://example.org/entry-1".into()));

        let feed = Feed::new("Test Feed", vec![entry]);
        let result = render(&feed).unwrap();

        assert!(result.contains("<entry>"));
        assert!(result.contains("<title>Test Entry</title>"));
        assert!(result.contains("<id>entry-1</id>"));
        assert!(result.contains("<updated>2024-01-15T10:30:00Z</updated>"));
        assert!(result.contains("<published>2024-01-15T10:30:00Z</published>"));
        assert!(
            result.contains(r#"<link href="https://example.org/entry-1" />"#)
        );
        assert!(result.contains("<author>"));
        assert!(result.contains("<name>Test Author</name>"));
        assert!(result.contains(r#"<content type="html">"#));
        assert!(result.contains("&lt;p&gt;Test content&lt;/p&gt;"));
        assert!(result.contains("</entry>"));
    }

    #[crate::ctb_test]
    fn test_render_entry_minimal() {
        // Entry with only required fields
        let entry = Entry::new("minimal-id", "Minimal Entry", "", "");
        let feed = Feed::new("Minimal Feed", vec![entry]);
        let result = render(&feed).unwrap();

        assert!(result.contains("<title>Minimal Entry</title>"));
        assert!(result.contains("<id>minimal-id</id>"));
        // Should have fallback updated date
        assert!(result.contains("<updated>1970-01-01T00:00:00Z</updated>"));
        // Should not have published (empty date)
        assert!(!result.contains("<published>"));
        // Should not have content (empty body)
        assert!(!result.contains("<content"));
        // Should not have author
        assert!(!result.contains("<author>"));
    }

    #[crate::ctb_test]
    fn test_render_multiple_entries() {
        let entry1 =
            Entry::new("id-1", "First", "<p>First</p>", "2024-01-10T00:00:00Z");
        let entry2 = Entry::new(
            "id-2",
            "Second",
            "<p>Second</p>",
            "2024-01-15T00:00:00Z",
        );
        let feed = Feed::new("Multi Entry Feed", vec![entry1, entry2]);
        let result = render(&feed).unwrap();

        // Should have both entries
        assert!(result.contains("<id>id-1</id>"));
        assert!(result.contains("<id>id-2</id>"));

        // Feed updated should be the latest entry date
        assert!(result.contains("<updated>2024-01-15T00:00:00Z</updated>"));
    }

    #[crate::ctb_test]
    fn test_render_xml_escaping() {
        let entry = Entry::new(
            "escape-test",
            "Title with <special> & \"chars\"",
            "<p>Content with <b>bold</b> & more</p>",
            "2024-01-01T00:00:00Z",
        );
        let feed = Feed::new("Escape Test", vec![entry]);
        let result = render(&feed).unwrap();

        // Title should be escaped
        assert!(result.contains(
            "<title>Title with &lt;special&gt; &amp; \"chars\"</title>"
        ));
        // Content should be escaped (HTML in CDATA-style escape)
        assert!(
            result.contains("&lt;p&gt;Content with &lt;b&gt;bold&lt;/b&gt;")
        );
    }
}
