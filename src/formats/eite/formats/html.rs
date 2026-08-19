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

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::Result;

use crate::{
    dc::{dc_get_script, dc_get_type},
    formats::dc_to_format,
};
use ctb_formats_utilities::FormatLog;

/// Build an HTML fragment (div with pre-wrap) from a Dc array using dc->html conversion.
pub fn dca_to_html_fragment(dc_array: &[u32]) -> Result<(Vec<u8>, FormatLog)> {
    let mut out = Vec::new();
    out.extend_from_slice(b"<div style=\"white-space:pre-wrap\">");
    let mut log = FormatLog::default();
    for &dc in dc_array {
        let (encoded, dc_log) = dc_to_format("html", dc)?;
        out.extend_from_slice(&encoded);
        log.merge(&dc_log);
    }
    out.extend_from_slice(b"</div>");
    Ok((out, log))
}

/// Build a minimal HTML document containing the HTML fragment for a Dc array.
pub fn dca_to_html(dc_array: &[u32]) -> Result<(Vec<u8>, FormatLog)> {
    let mut out = Vec::new();
    out.extend_from_slice(
        b"<!DOCTYPE html><html><head><title></title></head><body>",
    );
    let (fragment, log) = dca_to_html_fragment(dc_array)?;
    out.extend(fragment);
    out.extend_from_slice(b"</body></html>");
    Ok((out, log))
}

/// Convert Dc array to a full HTML document with color coding legend + fragment.
pub fn dca_to_colorcoded(dc_array: &[u32]) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    out.extend_from_slice(
        b"<!DOCTYPE html><html><head><title></title></head><body>\
        <p>Key: <span style=\"color:black\">Letter</span> \
<span style=\"color:gray\">Control</span> \
<span style=\"color:blue\">Semantic</span> \
<span style=\"color:salmon\">Mathematics</span> \
<span style=\"color:rebeccapurple\">Symbols</span> \
<span style=\"color:red\">Programming</span> \
<span style=\"color:green\">Financial</span> \
<span style=\"color:orange\">Punctuation</span> \
<span style=\"color:purple\">Emoji</span> \
<span style=\"color:maroon\">Styling</span> \
<span style=\"color:brown\">Other</span></p>",
    );
    out.extend(dca_to_colorcoded_fragment(dc_array)?);
    out.extend_from_slice(b"</body></html>");
    Ok(out)
}

/// Color-coded fragment (div wrapper) for Dc array.
pub fn dca_to_colorcoded_fragment(dc_array: &[u32]) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    out.extend_from_slice(b"<div style=\"white-space:pre-wrap\">");
    for &dc in dc_array {
        out.extend(dc_to_colorcoded(dc)?);
    }
    out.extend_from_slice(b"</div>");
    Ok(out)
}

/// Color-coded span for a single Dc.
pub fn dc_to_colorcoded(dc: u32) -> Result<Vec<u8>> {
    // Determine classification
    // Reason for fallback: when dc_get_type returns an error or unclassified codepoint, defaulting to empty string falls through to default black text color.
    let dc_type = dc_get_type(dc).unwrap_or_default();
    // Reason for fallback: when dc_get_script returns an error or unclassified codepoint, defaulting to empty string falls through to default black text color.
    let script = dc_get_script(dc).unwrap_or_default();
    let color = if dc_type.starts_with('L') {
        // Letter
        "black"
    } else if script == "Controls" {
        "gray"
    } else if script == "Semantic" {
        "blue"
    } else if script == "Mathematics" {
        "salmon"
    } else if script == "Symbols" {
        "rebeccapurple"
    } else if script.starts_with("EL ") {
        "red" // Programming
    } else if script == "Financial" {
        "green"
    } else if script == "Punctuation" {
        "orange"
    } else if script == "Emoji" {
        "purple"
    } else if script == "Colors" {
        "maroon"
    } else {
        "brown"
    };
    let mut out = Vec::new();
    out.extend_from_slice(b"<span style=\"color:");
    out.extend_from_slice(color.as_bytes());
    out.extend_from_slice(b"\">");
    out.extend_from_slice(dc.to_string().as_bytes());
    out.extend_from_slice(b"</span> ");
    Ok(out)
}

#[cfg(test)]
#[allow(
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
    use crate::util::string::str_to_byte_array;
    use ctb_formats_utilities::assert_vec_u8_ok_eq_no_warnings;

    use super::*;

    #[crate::ctb_test]
    fn test_dca_to_html_fragment() -> Result<()> {
        let actual = dca_to_html_fragment(&[39, 46, 40]);
        let expected = str_to_byte_array(
            "<div style=\"white-space:pre-wrap\">5&lt;6</div>",
        )?;
        assert_vec_u8_ok_eq_no_warnings(&expected, actual);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_format_html() -> Result<()> {
        // Input Dcs: [39,46,40] expected to produce: "<div style=\"white-space:pre-wrap\">5&lt;6</div>"
        // Wrapped with full HTML skeleton per original JS dcaToHtml test.
        let expected_html = "<!DOCTYPE html><html><head><title></title></head><body><div style=\"white-space:pre-wrap\">5&lt;6</div></body></html>";
        let result = dca_to_html(&[39, 46, 40]);
        assert_vec_u8_ok_eq_no_warnings(
            &str_to_byte_array(expected_html)?,
            result,
        );
        Ok(())
    }

    #[crate::ctb_test]
    fn test_dca_to_colorcoded_basic() {
        let dc_array = vec![65, 66]; // Example Dcs
        let html = dca_to_colorcoded(&dc_array).unwrap();
        let s = String::from_utf8(html).unwrap();
        assert!(s.contains("<span style=\"color:"));
    }
}
