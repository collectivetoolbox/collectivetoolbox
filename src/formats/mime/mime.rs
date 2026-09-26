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

//! MIME media type string parsing, syntax validation, and structure.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

/// Standard IANA and widely recognized top-level media type trees.
pub const STANDARD_TOP_LEVEL_TYPES: &[&str] = &[
    "application",
    "audio",
    "example",
    "font",
    "image",
    "message",
    "model",
    "multipart",
    "text",
    "video",
];

/// Structured representation of a parsed and validated MIME media type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MimeDetection {
    /// Normalized media type string without parameters (e.g. `image/jpeg`).
    pub media_type: String,
    /// Top-level media type component (e.g. `image`).
    pub top_level: String,
    /// Subtype component (e.g. `jpeg`).
    pub subtype: String,
    /// Optional parameters (e.g. `charset=utf-8`).
    pub parameters: Vec<(String, String)>,
    /// Descriptive summary of the detected MIME type.
    pub description: String,
}

/// Checks whether a byte is a valid MIME token character under RFC 2045/6838.
#[must_use]
pub fn is_valid_token_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric()
        || b == b'!'
        || b == b'#'
        || b == b'$'
        || b == b'%'
        || b == b'&'
        || b == b'\''
        || b == b'*'
        || b == b'+'
        || b == b'-'
        || b == b'.'
        || b == b'^'
        || b == b'_'
        || b == b'`'
        || b == b'|'
        || b == b'~'
}

/// Checks whether a string conforms to the syntax of a MIME media type.
#[must_use]
pub fn is_valid_mime_type(s: &str) -> bool {
    parse_mime_type(s).is_some()
}

/// Parses a string into a validated `MimeDetection` if it represents a valid
/// MIME media type (e.g. `text/plain`, `application/json; charset=utf-8`).
#[must_use]
pub fn parse_mime_type(s: &str) -> Option<MimeDetection> {
    let trimmed = s.trim();
    if trimmed.is_empty() || !trimmed.contains('/') {
        return None;
    }

    let mut parts = trimmed.split(';');
    let base = parts.next()?.trim();
    if base.is_empty() || base.contains(' ') {
        return None;
    }

    let (top, sub) = base.split_once('/')?;
    let top_clean = top.trim();
    let sub_clean = sub.trim();
    if top_clean.is_empty() || sub_clean.is_empty() {
        return None;
    }

    let top_lower = top_clean.to_ascii_lowercase();
    let is_standard_top = STANDARD_TOP_LEVEL_TYPES
        .iter()
        .any(|&t| t == top_lower.as_str());
    let is_x_top = top_lower.starts_with("x-");
    if !is_standard_top && !is_x_top {
        return None;
    }

    let valid_sub = sub_clean
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'+' || b == b'.' || b == b'-' || b == b'_');
    if !valid_sub {
        return None;
    }

    let mut parameters = Vec::new();
    for param in parts {
        let p_trimmed = param.trim();
        if p_trimmed.is_empty() {
            continue;
        }
        if let Some((k, v)) = p_trimmed.split_once('=') {
            parameters.push((
                k.trim().to_ascii_lowercase(),
                v.trim().trim_matches('"').to_string(),
            ));
        }
    }

    let normalized_media_type = format!("{}/{}", top_lower, sub_clean.to_ascii_lowercase());
    let description = format!("MIME type string: {trimmed}");

    Some(MimeDetection {
        media_type: normalized_media_type,
        top_level: top_lower,
        subtype: sub_clean.to_string(),
        parameters,
        description,
    })
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
    use super::*;

    #[crate::ctb_test]
    fn test_parse_valid_mime_types() {
        let json = parse_mime_type("application/json").unwrap();
        assert_eq!(json.media_type, "application/json");
        assert_eq!(json.top_level, "application");
        assert_eq!(json.subtype, "json");
        assert!(json.parameters.is_empty());

        let text = parse_mime_type("text/plain; charset=utf-8").unwrap();
        assert_eq!(text.media_type, "text/plain");
        assert_eq!(text.parameters, vec![("charset".to_string(), "utf-8".to_string())]);

        let svg = parse_mime_type("image/svg+xml").unwrap();
        assert_eq!(svg.media_type, "image/svg+xml");

        let custom = parse_mime_type("x-custom/vendor-format").unwrap();
        assert_eq!(custom.media_type, "x-custom/vendor-format");
    }

    #[crate::ctb_test]
    fn test_parse_invalid_mime_types() {
        assert!(parse_mime_type("").is_none());
        assert!(parse_mime_type("plain text").is_none());
        assert!(parse_mime_type("invalid/").is_none());
        assert!(parse_mime_type("/json").is_none());
        assert!(parse_mime_type("unknown_tree/json").is_none());
        assert!(parse_mime_type("application/json with space").is_none());
    }
}
