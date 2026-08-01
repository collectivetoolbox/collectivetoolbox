#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::{Entry, Feed};
use serde::Serialize;

const JSON_FEED_VERSION: &str = "https://jsonfeed.org/version/1.1";

/// Render a [`Feed`] into JSON Feed 1.1.
///
/// This intentionally produces a minimal valid JSON Feed with a stable shape,
/// while leaving room to extend the data model later (attachments, tags, etc.).
///
/// Specification: <https://www.jsonfeed.org/version/1.1/>
pub(crate) fn render(feed: &Feed) -> Result<String> {
    let out = JsonFeed::from_feed(feed);
    serde_json::to_string_pretty(&out).context("serializing JSON Feed")
}

#[derive(Debug, Serialize)]
struct JsonFeed<'a> {
    version: &'static str,
    title: &'a str,

    #[serde(skip_serializing_if = "Option::is_none")]
    home_page_url: Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    feed_url: Option<&'a str>,

    items: Vec<JsonFeedItem<'a>>,
}

impl<'a> JsonFeed<'a> {
    fn from_feed(feed: &'a Feed) -> Self {
        Self {
            version: JSON_FEED_VERSION,
            title: feed.title(),
            home_page_url: feed.home_page_url(),
            feed_url: feed.feed_url(),
            items: feed
                .entries()
                .iter()
                .map(JsonFeedItem::from_entry)
                .collect(),
        }
    }
}

#[derive(Debug, Serialize)]
struct JsonFeedItem<'a> {
    id: &'a str,

    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    content_html: Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    date_published: Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    author: Option<JsonFeedAuthor<'a>>,
}

impl<'a> JsonFeedItem<'a> {
    fn from_entry(entry: &'a Entry) -> Self {
        let title = entry.title();
        let title = (!title.is_empty()).then_some(title);

        let body_html = entry.body();
        let body_html = (!body_html.is_empty()).then_some(body_html);

        let date_published = entry.date();
        let date_published =
            (!date_published.is_empty()).then_some(date_published);

        Self {
            id: entry.id(),
            url: entry.url(),
            title,
            content_html: body_html,
            date_published,
            author: entry.author().map(|name| JsonFeedAuthor { name }),
        }
    }
}

#[derive(Debug, Serialize)]
struct JsonFeedAuthor<'a> {
    name: &'a str,
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
    fn test_render_empty_feed() {
        let feed = Feed::empty("Empty Test Feed");
        let result = render(&feed).unwrap();

        assert!(
            result.contains(r#""version": "https://jsonfeed.org/version/1.1""#)
        );
        assert!(result.contains(r#""title": "Empty Test Feed""#), "{result}");
        assert!(result.contains(r#""items": []"#));
    }

    #[crate::ctb_test]
    fn test_render_feed_with_urls() {
        let feed = Feed::empty("Test Feed")
            .with_home_page_url_opt(Some("https://example.org/".into()))
            .with_feed_url_opt(Some("https://example.org/feed.json".into()));
        let result = render(&feed).unwrap();

        assert!(result.contains(r#""home_page_url": "https://example.org/""#));
        assert!(
            result.contains(r#""feed_url": "https://example.org/feed.json""#)
        );
    }

    #[crate::ctb_test]
    fn test_render_feed_without_optional_urls() {
        let feed = Feed::empty("Test Feed");
        let result = render(&feed).unwrap();

        // Optional fields should not appear when not set
        assert!(!result.contains("home_page_url"));
        assert!(!result.contains("feed_url"));
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

        assert!(result.contains(r#""id": "entry-1""#));
        assert!(result.contains(r#""title": "Test Entry""#));
        assert!(result.contains(r#""content_html": "<p>Test content</p>""#));
        assert!(result.contains(r#""date_published": "2024-01-15T10:30:00Z""#));
        assert!(result.contains(r#""url": "https://example.org/entry-1""#));
        assert!(result.contains(r#""author""#));
        assert!(result.contains(r#""name": "Test Author""#));
    }

    #[crate::ctb_test]
    fn test_render_entry_minimal() {
        // Entry with only required fields (id is required, others optional)
        let entry = Entry::new("minimal-id", "", "", "");
        let feed = Feed::new("Minimal Feed", vec![entry]);
        let result = render(&feed).unwrap();

        assert!(result.contains(r#""id": "minimal-id""#));
        // Empty strings should not produce fields
        assert!(!result.contains(r#""title": """#), "{result}");
        assert!(!result.contains(r#""content_html""#));
        assert!(!result.contains(r#""date_published""#));
        assert!(!result.contains(r#""author""#));
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

        assert!(result.contains(r#""id": "id-1""#));
        assert!(result.contains(r#""id": "id-2""#));
        assert!(result.contains(r#""title": "First""#));
        assert!(result.contains(r#""title": "Second""#));
    }

    #[crate::ctb_test]
    fn test_render_json_structure() {
        let feed = Feed::empty("Structure Test");
        let result = render(&feed).unwrap();

        // Should be valid JSON that can be parsed
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(
            parsed.get("version").unwrap(),
            "https://jsonfeed.org/version/1.1"
        );
        assert_eq!(parsed.get("title").unwrap(), "Structure Test");
        assert!(parsed.get("items").unwrap().is_array());
    }

    #[crate::ctb_test]
    fn test_render_special_characters() {
        let entry = Entry::new(
            "special-chars",
            "Title with \"quotes\" and <angle>",
            "<p>Content with special chars: &amp; \"quotes\"</p>",
            "2024-01-01T00:00:00Z",
        );
        let feed = Feed::new("Special Chars Test", vec![entry]);
        let result = render(&feed).unwrap();

        // Should produce valid JSON even with special characters
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        let items = parsed.get("items").unwrap().as_array().unwrap();
        let first_item = items.first().unwrap();
        assert_eq!(
            first_item.get("title").unwrap(),
            "Title with \"quotes\" and <angle>"
        );
    }
}
