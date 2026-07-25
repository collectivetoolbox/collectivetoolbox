//! hAtom microformat renderer for the syndication data model.
//!
//! Produces (X)HTML output with hAtom microformat markup, allowing feeds to
//! be embedded in web pages and parsed by microformat-aware tools.
//!
//! The output follows the hAtom specification from microformats.org, using
//! class names like `hfeed`, `hentry`, `entry-title`, `entry-content`,
//! `author`, `updated`, `published`, and `rel-bookmark` for permalinks.
//!
//! Also includes microformats2 classes such as `h-feed`/`h-entry` and common
//! properties like `p-name`, `u-url`, `p-author`, `dt-updated`, `e-content`.
//!
//! # Example Output
//!
//! ```html
//! <div class="h-feed hfeed">
//!   <h1 class="p-name">My Feed</h1>
//!   <article class="h-entry hentry">
//!     <h2 class="p-name entry-title">
//!       <a href="https://example.org/post-1" rel="bookmark" class="u-url">My Post</a>
//!     </h2>
//!     <address class="p-author author h-card vcard">
//!       <span class="p-name fn">Author Name</span>
//!     </address>
//!     <time class="dt-updated updated" datetime="2024-01-15T10:30:00Z">
//!       2024-01-15T10:30:00Z
//!     </time>
//!     <div class="e-content entry-content">
//!       <p>Post content here...</p>
//!     </div>
//!   </article>
//! </div>
//! ```
//!
//! Specifications:
//!
//! - hAtom: <https://microformats.org/wiki/hatom>
//! - h-feed: <https://microformats.org/wiki/h-feed>
//! - h-entry: <https://microformats.org/wiki/h-entry>

use ctb_formats_html::{escape_quoted_attr, escape_text};

#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

use super::{Entry, Feed};
use std::fmt::Write as _;

/// Render a [`Feed`] into HTML with hAtom microformat markup.
///
/// Produces a complete HTML fragment with:
/// - A container `<div class="h-feed hfeed">` wrapping all entries
/// - Each entry as an `<article class="h-entry hentry">` with required fields
/// - Proper semantic markup for authors, dates, and permalinks
///
/// The output is self-contained and can be embedded in a larger HTML page.
pub(crate) fn render(feed: &Feed) -> Result<String> {
    let mut output = String::new();

    // Open container. Publish both mf2 + classic for backcompat.
    writeln!(output, "<div class=\"h-feed hfeed\">")?;

    // Feed name (mf2 p-name). hAtom doesn't standardize a feed-title property.
    if !feed.title().is_empty() {
        writeln!(
            output,
            "\t<h1 class=\"p-name\">{}</h1>",
            escape_text(feed.title())
        )?;
    }

    // Feed URLs (mf2 u-url). Use <a> (valid in fragments); avoid <link> which
    // belongs in <head>.
    if let Some(home_url) = feed.home_page_url() {
        writeln!(
            output,
            "\t<a class=\"u-url\" rel=\"home\" href=\"{}\">{}</a>",
            escape_quoted_attr(home_url),
            escape_text(home_url)
        )?;
    }

    if let Some(feed_url) = feed.feed_url() {
        writeln!(
            output,
            "\t<a class=\"u-url\" rel=\"self\" href=\"{}\">{}</a>",
            escape_quoted_attr(feed_url),
            escape_text(feed_url)
        )?;
    }

    for entry in feed.entries() {
        write_entry(&mut output, entry)?;
    }

    writeln!(output, "</div>")?;
    Ok(output)
}

/// Write an entry element with hAtom markup.
fn write_entry(output: &mut String, entry: &Entry) -> Result<()> {
    // Entry container (mf2 + classic).
    writeln!(output, "\t<article class=\"h-entry hentry\">")?;

    // Entry title (classic entry-title + mf2 p-name).
    // If there's a URL, emit a permalink with rel-bookmark + mf2 u-url.
    if let Some(url) = entry.url() {
        writeln!(
            output,
            "\t\t<h2 class=\"p-name entry-title\"><a href=\"{}\" rel=\"bookmark\" class=\"u-url\">{}</a></h2>",
            escape_quoted_attr(url),
            escape_text(entry.title())
        )?;
    } else {
        writeln!(
            output,
            "\t\t<h2 class=\"p-name entry-title\">{}</h2>",
            escape_text(entry.title())
        )?;
    }

    // Author (classic author + hCard; mf2 p-author h-card). hAtom requires this
    // for validity; if missing we omit rather than inventing data.
    if let Some(author_name) = entry.author() {
        writeln!(
            output,
            "\t\t<address class=\"p-author author h-card vcard\"><span class=\"p-name fn\">{}</span></address>",
            escape_text(author_name)
        )?;
    } else {
        warn_fmt!("hAtom entry missing required author: id={}", entry.id());
    }

    // Updated is required for valid hAtom. If absent, hAtom says "use published"
    // if present.
    let date_updated = entry.updated();
    let date_published = entry.date();
    if !date_published.is_empty() {
        writeln!(
            output,
            "\t\t<time class=\"published dt-published\" datetime=\"{}\">{}</time>",
            escape_quoted_attr(date_published),
            escape_text(date_published)
        )?;
    }
    if let Some(updated) = date_updated {
        if !updated.is_empty() {
            writeln!(
                output,
                "\t\t<time class=\"updated dt-updated\" datetime=\"{}\">{}</time>",
                escape_quoted_attr(updated),
                escape_text(updated)
            )?;
        }
    }
    let has_date_updated = if let Some(updated) = date_updated {
        !updated.is_empty()
    } else {
        false
    };
    if !has_date_updated && date_published.is_empty() {
        warn!(
            "No published or updated date was found. hAtom requires at least one."
        );
    }

    // Entry content (classic entry-content + mf2 e-content). If missing, empty
    // string is implied; we simply omit the element.
    if !entry.body().is_empty() {
        writeln!(output, "\t\t<div class=\"e-content entry-content\">")?;
        writeln!(output, "\t\t\t{}", entry.body())?;
        writeln!(output, "\t\t</div>")?;
    }

    writeln!(output, "\t</article>")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[crate::ctb_test]
    fn test_render_empty_feed() -> Result<()> {
        let feed = Feed::empty("Empty Test Feed");
        let result = render(&feed)?;

        assert!(result.contains("<div class=\"h-feed hfeed\">"));
        assert!(result.contains("<h1 class=\"p-name\">Empty Test Feed</h1>"));
        assert!(result.contains("</div>"));
        Ok(())
    }

    #[crate::ctb_test]
    fn test_render_feed_with_urls() -> Result<()> {
        let feed = Feed::empty("Test Feed")
            .with_home_page_url_opt(Some("https://example.org/".into()))
            .with_feed_url_opt(Some("https://example.org/feed.html".into()));
        let result = render(&feed)?;

        assert!(
            result.contains(
                r#"<a class="u-url" rel="home" href="https://example.org/">"#
            ),
            "{result}"
        );
        assert!(
            result.contains(r#"<a class="u-url" rel="self" href="https://example.org/feed.html">"#),
            "{result}"
        );
        Ok(())
    }

    #[crate::ctb_test]
    fn test_render_feed_with_entry() -> Result<()> {
        let entry = Entry::new(
            "entry-1",
            "Test Entry",
            "<p>Test content</p>",
            "2024-01-15T10:30:00Z",
        )
        .with_author("Test Author")
        .with_url_opt(Some("https://example.org/entry-1".into()));

        let feed = Feed::new("Test Feed", vec![entry]);
        let result = render(&feed)?;

        assert!(result.contains("<article class=\"h-entry hentry\">"));
        assert!(result.contains("<h2 class=\"p-name entry-title\">"));
        assert!(
            result.contains(
                r#"<a href="https://example.org/entry-1" rel="bookmark" class="u-url">Test Entry</a>"#
            ),
            "{result}"
        );
        assert!(
            result.contains("<address class=\"p-author author h-card vcard\">"),
            "{result}"
        );
        assert!(
            result.contains("<span class=\"p-name fn\">Test Author</span>")
        );
        assert!(
            result.contains(r#"<time class="published dt-published" datetime="2024-01-15T10:30:00Z">"#),
            "{result}"
        );
        assert!(result.contains("<div class=\"e-content entry-content\">"));
        assert!(result.contains("<p>Test content</p>"));
        assert!(result.contains("</article>"));
        Ok(())
    }

    #[crate::ctb_test]
    fn test_render_feed_with_updated_date() -> Result<()> {
        let entry = Entry::new(
            "entry-1",
            "Test Entry",
            "<p>Test content</p>",
            "2024-01-15T10:30:00Z",
        )
        .with_author("Test Author")
        .with_updated_opt(Some("2024-01-20T12:00:00Z".to_string()))
        .with_url_opt(Some("https://example.org/entry-1".into()));

        let feed = Feed::new("Test Feed", vec![entry]);
        let result = render(&feed)?;

        assert!(
            result.contains(r#"<time class="published dt-published" datetime="2024-01-15T10:30:00Z">"#),
            "{result}"
        );
        assert!(
            result.contains(r#"<time class="updated dt-updated" datetime="2024-01-20T12:00:00Z">"#),
            "{result}"
        );
        Ok(())
    }

    #[crate::ctb_test]
    fn test_render_entry_minimal() -> Result<()> {
        let entry = Entry::new("minimal-id", "Minimal Entry", "", "");
        let feed = Feed::new("Minimal Feed", vec![entry]);
        let result = render(&feed)?;

        assert!(
            result.contains(
                "<h2 class=\"p-name entry-title\">Minimal Entry</h2>"
            )
        );
        assert!(!result.contains("rel=\"bookmark\""));
        assert!(!result.contains("<time"));
        assert!(!result.contains("entry-content"));
        assert!(!result.contains("author"));
        Ok(())
    }

    #[crate::ctb_test]
    fn test_render_entry_without_url() -> Result<()> {
        let entry = Entry::new(
            "no-url",
            "Entry Without URL",
            "<p>Content</p>",
            "2024-01-10T00:00:00Z",
        )
        .with_author("Author");

        let feed = Feed::new("Test", vec![entry]);
        let result = render(&feed)?;

        assert!(result.contains(
            "<h2 class=\"p-name entry-title\">Entry Without URL</h2>"
        ));
        assert!(!result.contains("rel=\"bookmark\""));
        Ok(())
    }

    #[crate::ctb_test]
    fn test_render_multiple_entries() -> Result<()> {
        let entry1 =
            Entry::new("id-1", "First", "<p>First</p>", "2024-01-10T00:00:00Z");
        let entry2 = Entry::new(
            "id-2",
            "Second",
            "<p>Second</p>",
            "2024-01-15T00:00:00Z",
        );
        let feed = Feed::new("Multi Entry Feed", vec![entry1, entry2]);
        let result = render(&feed)?;

        assert!(result.contains(">First</"));
        assert!(result.contains(">Second</"));

        let article_count =
            result.matches("<article class=\"h-entry hentry\">").count();
        assert_eq!(article_count, 2);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_render_html_escaping_in_title() -> Result<()> {
        let entry = Entry::new(
            "escape-test",
            "Title with <special> & \"chars\"",
            "<p>Content</p>",
            "2024-01-01T00:00:00Z",
        );
        let feed = Feed::new("Escape Test", vec![entry]);
        let result = render(&feed)?;

        assert!(result.contains("Title with &lt;special&gt; &amp; \"chars\""));
        Ok(())
    }

    #[crate::ctb_test]
    fn test_render_html_escaping_in_author() -> Result<()> {
        let entry =
            Entry::new("author-test", "Title", "", "2024-01-01T00:00:00Z")
                .with_author("O'Brien & Sons <LLC>");

        let feed = Feed::new("Author Escape Test", vec![entry]);
        let result = render(&feed)?;

        assert!(result.contains("O'Brien &amp; Sons &lt;LLC&gt;"));
        Ok(())
    }

    #[crate::ctb_test]
    fn test_render_url_escaping_in_permalink() -> Result<()> {
        let entry = Entry::new("url-test", "Title", "", "2024-01-01T00:00:00Z")
            .with_url_opt(Some(
                "https://example.org/page?foo's=1&bar=2".into(),
            ));

        let feed = Feed::new("URL Escape Test", vec![entry]);
        let result = render(&feed)?;

        assert!(result.contains(
            "href=\"https://example.org/page?foo&#x27;s=1&amp;bar=2\""
        ));
        Ok(())
    }

    #[crate::ctb_test]
    fn test_hfeed_structure() -> Result<()> {
        let feed = Feed::empty("Structure Test");
        let result = render(&feed)?;

        let Some(hfeed_start) = result.find("<div class=\"h-feed hfeed\">")
        else {
            bail!("missing h-feed container: {result}");
        };
        let Some(title_start) = result.find("<h1 class=\"p-name\">") else {
            bail!("missing p-name title: {result}");
        };
        let Some(hfeed_end) = result.rfind("</div>") else {
            bail!("missing closing div: {result}");
        };

        assert!(hfeed_start < title_start);
        assert!(title_start < hfeed_end);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_vcard_author_structure() -> Result<()> {
        let entry =
            Entry::new("vcard-test", "Title", "", "2024-01-01T00:00:00Z")
                .with_author("Jane Doe");

        let feed = Feed::new("vCard Test", vec![entry]);
        let result = render(&feed)?;

        assert!(result.contains("class=\"p-author author h-card vcard\""));
        assert!(result.contains("class=\"p-name fn\""));
        Ok(())
    }
}
