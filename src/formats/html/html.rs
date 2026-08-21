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

//! HTML parsing, plain text rendering, and HTML table extraction utilities.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use include_dir::{Dir, include_dir};
use std::iter;

pub mod entities;
pub mod markdown;
pub mod text_clipper;
pub mod text_decorator;

static HTML_DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data");

pub(crate) fn get_html_data(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&HTML_DATA_DIR, key)
}

/// Convert HTML bytes to plain text with default width 80.
pub fn html2text(html: Vec<u8>) -> Result<Vec<u8>> {
    html2text_with_width(html, 80)
}

/// Convert HTML bytes to plain text wrapped to the specified width.
pub fn html2text_with_width(html: Vec<u8>, width: u16) -> Result<Vec<u8>> {
    let width_usize = usize::from(width);
    let text = html2text::from_read_with_decorator(
        html.as_slice(),
        width_usize,
        crate::text_decorator::PlainTextDecorator::new(),
    )
    .map_err(|e| anyhow::anyhow!("Failed to parse HTML to text: {e}"))?;
    Ok(text.into_bytes())
}

pub fn sanitize_html(document: Vec<u8>) -> Vec<u8> {
    let mut builder = ammonia::Builder::default();

    builder.add_generic_attributes(iter::once("class").chain(iter::once("id")));

    builder
        .clean(&String::from_utf8_lossy(&document))
        .to_string()
        .into_bytes()
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
    fn test_html2text_no_markdown_links() -> Result<()> {
        let html = b"<p>Visit <a href=\"https://example.com/\">our website</a> for more details.</p>";
        let text = html2text(html.to_vec())?;
        let text_str = String::from_utf8(text)?;
        assert_eq!(text_str.trim(), "Visit our website for more details.");
        Ok(())
    }

    #[crate::ctb_test]
    fn test_html2text_preserves_lists_and_quotes() -> Result<()> {
        let html = b"<ul><li>Alpha</li><li>Beta</li></ul><blockquote><p>Quote text</p></blockquote>";
        let text = html2text(html.to_vec())?;
        let text_str = String::from_utf8(text)?;
        assert!(text_str.contains("* Alpha"));
        assert!(text_str.contains("* Beta"));
        assert!(text_str.contains("> Quote text"));
        Ok(())
    }

}
