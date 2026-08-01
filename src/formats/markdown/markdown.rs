#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use ctb_utilities::anyhow::anyhow;
use markdown::Options;

fn get_markdown_options(allow_dangerous_html: bool) -> Options {
    let mut options = Options::default();
    options.parse.constructs.gfm_table = true;
    options.parse.constructs.gfm_strikethrough = true;
    options.parse.constructs.gfm_footnote_definition = true;
    options.parse.constructs.gfm_label_start_footnote = true;
    options.compile.allow_dangerous_html = allow_dangerous_html;
    options
}

/// Embedded HTML will be broken by this function.
pub fn markdown2html(markdown: Vec<u8>) -> Vec<u8> {
    markdown2html_str(&String::from_utf8_lossy(&markdown)).into_bytes()
}

pub fn markdown2html_str(markdown: &str) -> String {
    let replaced = branding::replace_magic_strings(markdown);
    markdown::to_html_with_options(&replaced, &get_markdown_options(false))
        .unwrap_or_default()
}

/// Allows Embedded HTML, which can cause security issues. Use only for trusted markdown content.
pub fn markdown2html_unsafe(markdown: Vec<u8>) -> Result<Vec<u8>> {
    Ok(
        markdown2html_str_unsafe(&String::from_utf8_lossy(&markdown))?
            .into_bytes(),
    )
}

pub fn markdown2html_str_unsafe(markdown: &str) -> Result<String> {
    let replaced = branding::replace_magic_strings(markdown);
    markdown::to_html_with_options(&replaced, &get_markdown_options(true))
        .map_err(|e| anyhow!("Failed to convert markdown to HTML: {e}"))
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
    fn test_markdown_gfm_features() {
        // Test Table
        let table_md = "| Head 1 | Head 2 |\n|---|---|\n| Cell 1 | Cell 2 |";
        let table_html = markdown2html_str(table_md);
        assert!(table_html.contains("<table>"));
        assert!(table_html.contains("<th>Head 1</th>"));
        assert!(table_html.contains("<td>Cell 1</td>"));

        // Test Strikethrough (both single and double tildes are supported by GFM)
        let strikethrough_md = "~strike~ and ~~double strike~~";
        let strikethrough_html = markdown2html_str(strikethrough_md);
        assert!(strikethrough_html.contains("<del>strike</del>"));
        assert!(strikethrough_html.contains("<del>double strike</del>"));

        // Test Footnotes
        let footnote_md = "Here is a footnote[^1].\n\n[^1]: Footnote content.";
        let footnote_html = markdown2html_str(footnote_md);
        assert!(footnote_html.contains("<sup"));
        assert!(footnote_html.contains("data-footnote-ref"));
        assert!(footnote_html.contains("class=\"footnotes\""));
        assert!(footnote_html.contains("Footnote content."));
    }
}
