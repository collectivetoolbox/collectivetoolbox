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

//! MIME media type string parsing, syntax validation, embedded datasets,
//! extension lookup, and MIME inheritance graph relationships.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use include_dir::{Dir, include_dir};
use std::collections::HashMap;
use std::sync::LazyLock;

static MIME_DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data");

/// Returns the embedded raw bytes of the Apache HTTPD `mime.types` file.
#[must_use]
pub fn get_httpd_mime_types() -> Option<&'static [u8]> {
    MIME_DATA_DIR
        .get_file("httpd/mime.types")
        .map(|f| f.contents())
}

/// Returns the embedded raw bytes of the IANA `media-types.txt` registry.
#[must_use]
pub fn get_iana_media_types() -> Option<&'static [u8]> {
    MIME_DATA_DIR
        .get_file("iana/media-types.txt")
        .map(|f| f.contents())
}

/// Returns the embedded raw bytes of the `mime-db/db.json` dataset.
#[must_use]
pub fn get_mime_db_json() -> Option<&'static [u8]> {
    MIME_DATA_DIR
        .get_file("mime-db/db.json")
        .map(|f| f.contents())
}

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

/// Parses lines of an Apache HTTPD `mime.types` configuration file into a list
/// of `(mime_type, Vec<extension>)` pairs.
#[must_use]
pub fn parse_apache_mime_types(content: &str) -> Vec<(String, Vec<String>)> {
    let mut results = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() >= 2 {
            if let Some(mime) = parts.first() {
                let mime_lower = mime.to_ascii_lowercase();
                let exts: Vec<String> = parts
                    .iter()
                    .skip(1)
                    .map(|e| e.to_ascii_lowercase())
                    .collect();
                results.push((mime_lower, exts));
            }
        }
    }
    results
}

/// Precompiled bidirectional lookup database mapping MIME types to file
/// extensions and extensions to primary MIME types.
#[derive(Debug, Clone, Default)]
pub struct MimeDatabase {
    by_mime: HashMap<String, Vec<String>>,
    by_extension: HashMap<String, String>,
}

impl MimeDatabase {
    /// Builds the database from embedded Apache `mime.types`.
    #[must_use]
    pub fn new() -> Self {
        let mut by_mime = HashMap::new();
        let mut by_extension = HashMap::new();

        if let Some(bytes) = get_httpd_mime_types() {
            if let Ok(content) = std::str::from_utf8(bytes) {
                for (mime, exts) in parse_apache_mime_types(content) {
                    for ext in &exts {
                        by_extension
                            .entry(ext.clone())
                            .or_insert_with(|| mime.clone());
                    }
                    by_mime.entry(mime).or_insert(exts);
                }
            }
        }

        Self {
            by_mime,
            by_extension,
        }
    }

    /// Looks up associated file extensions for a MIME media type.
    #[must_use]
    pub fn lookup_extensions(&self, mime: &str) -> Option<&[String]> {
        let lower = mime.trim().to_ascii_lowercase();
        self.by_mime.get(&lower).map(Vec::as_slice)
    }

    /// Looks up primary MIME media type for a file extension (without leading dot).
    #[must_use]
    pub fn lookup_mime(&self, extension: &str) -> Option<&str> {
        let clean = extension.trim().trim_start_matches('.').to_ascii_lowercase();
        self.by_extension.get(&clean).map(String::as_str)
    }
}

/// Global lazy-initialized MIME lookup database.
pub static MIME_DATABASE: LazyLock<MimeDatabase> = LazyLock::new(MimeDatabase::new);

/// Checks whether a child MIME type is a subclass or specialization of a parent
/// MIME type according to MIME taxonomy, structured suffixes, and inheritance
/// conventions.
///
/// Rules evaluated:
/// - Direct equality (case-insensitive)
/// - `+xml` subtypes inherit from `application/xml`
/// - `+json` subtypes inherit from `application/json`
/// - `+zip` subtypes, OpenDocument, OpenXML, JAR, APK inherit from `application/zip`
/// - XML, JSON, and non-plain `text/*` types inherit from `text/plain`
#[must_use]
pub fn is_mime_subclass_of(child_mime: &str, parent_mime: &str) -> bool {
    let child_norm = child_mime.trim().to_ascii_lowercase();
    let parent_norm = parent_mime.trim().to_ascii_lowercase();
    if child_norm == parent_norm {
        return true;
    }

    let mut visited = Vec::new();
    let mut queue = vec![child_norm];

    while let Some(current) = queue.pop() {
        if visited.contains(&current) {
            continue;
        }
        visited.push(current.clone());

        let mut dynamic_parents = Vec::new();
        if current.ends_with("+xml") && current != "application/xml" {
            dynamic_parents.push("application/xml".to_string());
        }
        if current == "application/xml"
            || current == "text/xml"
            || (current.starts_with("text/") && current != "text/plain")
            || current.ends_with("+json")
            || current == "application/json"
        {
            dynamic_parents.push("text/plain".to_string());
        }
        if (current.ends_with("+zip")
            || current.starts_with("application/vnd.openxmlformats-officedocument.")
            || current.starts_with("application/vnd.oasis.opendocument.")
            || current == "application/java-archive"
            || current == "application/vnd.android.package-archive")
            && current != "application/zip"
        {
            dynamic_parents.push("application/zip".to_string());
        }
        if current.ends_with("+json") && current != "application/json" {
            dynamic_parents.push("application/json".to_string());
        }

        for parent in dynamic_parents {
            if parent == parent_norm {
                return true;
            }
            if !visited.contains(&parent) {
                queue.push(parent);
            }
        }
    }

    false
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

    #[crate::ctb_test]
    fn test_embedded_mime_data_retrieval() {
        assert!(get_httpd_mime_types().is_some());
        assert!(get_iana_media_types().is_some());
        assert!(get_mime_db_json().is_some());
    }

    #[crate::ctb_test]
    fn test_mime_database_lookup() {
        let db = &*MIME_DATABASE;
        assert_eq!(db.lookup_mime("html"), Some("text/html"));
        assert_eq!(db.lookup_mime("json"), Some("application/json"));
        assert_eq!(db.lookup_mime(".pdf"), Some("application/pdf"));

        let pdf_exts = db.lookup_extensions("application/pdf").unwrap();
        assert!(pdf_exts.contains(&"pdf".to_string()));
    }

    #[crate::ctb_test]
    fn test_mime_subclass_hierarchy() {
        assert!(is_mime_subclass_of("image/svg+xml", "image/svg+xml"));
        assert!(is_mime_subclass_of("image/svg+xml", "application/xml"));
        assert!(is_mime_subclass_of("image/svg+xml", "text/plain"));
        assert!(is_mime_subclass_of("application/json", "text/plain"));
        assert!(is_mime_subclass_of("application/geo+json", "application/json"));
        assert!(is_mime_subclass_of("application/geo+json", "text/plain"));
        assert!(is_mime_subclass_of("application/vnd.openxmlformats-officedocument.wordprocessingml.document", "application/zip"));
        assert!(!is_mime_subclass_of("text/plain", "application/xml"));
        assert!(!is_mime_subclass_of("image/png", "application/zip"));
    }
}
