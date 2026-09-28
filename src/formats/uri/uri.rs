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

//! URI and URL parsing, normalization, and scheme validation utilities.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use ctb_utilities::anyhow::ensure;
use ctb_utilities::csv_tools::CsvTable;
use include_dir::{Dir, include_dir};
use std::sync::Arc;

static URI_DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data");

pub(crate) fn get_uri_data(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&URI_DATA_DIR, key)
}

fn uri_schemes() -> Result<Arc<CsvTable>> {
    csv_tools::get_or_load_cached(
        "ctb_formats_uri::data/uri-schemes-1.csv",
        || {
            csv_tools::parse_csv_reader(
                &bail_if_none!(get_uri_data("uri-schemes-1.csv")),
                csv_tools::CsvParseOptions {
                    has_header: true,
                    ..Default::default()
                },
            )
        },
    )
}

pub fn scheme_in(uri: &str, allowed_schemes: Vec<&str>) -> bool {
    if let Some(colon_pos) = uri.find(':') {
        let Some(scheme) = uri.get(..colon_pos) else {
            return false;
        };
        for allowed_scheme in allowed_schemes {
            if scheme.eq_ignore_ascii_case(allowed_scheme) {
                return true;
            }
        }
    }
    false
}

pub fn ensure_scheme_in(uri: &str, allowed_schemes: Vec<&str>) -> Result<()> {
    ensure!(scheme_in(uri, allowed_schemes), "URI scheme not allowed");
    Ok(())
}

pub fn list_iana_schemes() -> Result<Vec<String>> {
    let schemes_table = uri_schemes()?;
    let mut schemes = Vec::new();
    for row in schemes_table.rows_iter() {
        if !row.is_empty() {
            schemes.push(bail_if_none!(row.first()).to_string());
        }
    }
    Ok(schemes)
}

pub fn list_permanent_iana_schemes() -> Result<Vec<String>> {
    list_iana_schemes_by_status("Permanent")
}

pub fn list_provisional_iana_schemes() -> Result<Vec<String>> {
    list_iana_schemes_by_status("Provisional")
}

pub fn list_historic_iana_schemes() -> Result<Vec<String>> {
    list_iana_schemes_by_status("Historic")
}

pub fn list_permanent_or_historic_iana_schemes() -> Result<Vec<String>> {
    let mut schemes = list_permanent_iana_schemes()?;
    let historic_schemes = list_historic_iana_schemes()?;
    schemes.extend(historic_schemes);
    Ok(schemes)
}

fn list_iana_schemes_by_status(status: &str) -> Result<Vec<String>> {
    let schemes_table = uri_schemes()?;
    let mut schemes = Vec::new();
    for row in schemes_table.rows_iter() {
        if !row.is_empty() && row.get(3) == Some(&status.to_string()) {
            schemes.push(bail_if_none!(row.first()).to_string());
        }
    }
    Ok(schemes)
}

static IANA_SCHEMES_SET: std::sync::LazyLock<std::collections::HashSet<String>> =
    std::sync::LazyLock::new(|| {
        list_iana_schemes()
            .expect("IANA schemes database failed to load, but should be provided by asset bundle")
            .into_iter()
            .map(|s| s.to_ascii_lowercase())
            .collect()
    });

/// Checks whether `scheme` matches a known registered IANA URI scheme.
#[must_use]
pub fn is_known_scheme(scheme: &str) -> bool {
    // Reason for fallback: if no trailing colon is present, the trimmed scheme is already bare
    let clean = scheme.trim().strip_suffix(':').unwrap_or(scheme.trim());
    if clean.is_empty() {
        return false;
    }
    let lower = clean.to_ascii_lowercase();
    IANA_SCHEMES_SET.contains(&lower)
}

/// Result of URI or URI protocol scheme detection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UriDetection {
    /// FormatId: `FormatId::UriProtocol` for standalone schemes, `FormatId::Uri` (or `Magnet`) for full URIs.
    pub format_id: FormatId,
    /// Human-readable description.
    pub description: String,
    /// Lowercase URI scheme (e.g. "http", "https", "mailto", "urn", "ftp").
    pub scheme: String,
    /// True if the input represents only a scheme identifier rather than a complete URI.
    pub is_scheme_only: bool,
}

/// Detects whether `s` represents a full URI or a standalone URI scheme/protocol.
#[must_use]
pub fn detect_uri(s: &str) -> Option<UriDetection> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return None;
    }
    // URIs and URI schemes must not contain unencoded whitespace
    if trimmed.bytes().any(|b| b.is_ascii_whitespace()) {
        return None;
    }

    // 1. Check if it's a full URI with scheme prefix (e.g. "https://example.com", "mailto:user@domain.com")
    if let Some((scheme, rest)) = trimmed.split_once(':') {
        if !rest.is_empty() {
            let scheme_valid = !scheme.is_empty()
                && scheme.bytes().next().is_some_and(|b| b.is_ascii_alphabetic())
                && scheme
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'+' || b == b'-' || b == b'.');
            if scheme_valid && is_known_scheme(scheme) {
                let lower_scheme = scheme.to_ascii_lowercase();
                let fmt = if lower_scheme == "magnet" {
                    FormatId::Magnet
                } else {
                    FormatId::Uri
                };
                return Some(UriDetection {
                    format_id: fmt,
                    description: format!("URI ({lower_scheme})"),
                    scheme: lower_scheme,
                    is_scheme_only: false,
                });
            }
        }
    }

    // 2. Check if it's a standalone URI scheme / protocol (e.g. "http", "https", "ftp", "http:")
    // Reason for fallback: if no trailing colon is present, candidate is already stripped
    let scheme_candidate = trimmed.strip_suffix(':').unwrap_or(trimmed);
    let is_valid_scheme_syntax = !scheme_candidate.is_empty()
        && scheme_candidate
            .bytes()
            .next()
            .is_some_and(|b| b.is_ascii_alphabetic())
        && scheme_candidate
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'+' || b == b'-' || b == b'.');

    if is_valid_scheme_syntax && is_known_scheme(scheme_candidate) {
        let lower = scheme_candidate.to_ascii_lowercase();
        return Some(UriDetection {
            format_id: FormatId::UriProtocol,
            description: format!("URI protocol ({lower})"),
            scheme: lower,
            is_scheme_only: true,
        });
    }

    None
}

/// Checks whether `uri` begins with a known registered IANA URI scheme.
#[must_use]
pub fn is_iana_scheme(uri: &str) -> bool {
    if let Some(colon_pos) = uri.find(':') {
        let Some(scheme) = uri.get(..colon_pos) else {
            return false;
        };
        return is_known_scheme(scheme);
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
    use anyhow::{Result, anyhow};

    #[crate::ctb_test]
    fn test_get_uri_data_contains_header() -> Result<()> {
        let bytes = if let Some(b) = get_uri_data("uri-schemes-1.csv") {
            b
        } else {
            return Err(anyhow!("embedded CSV 'uri-schemes-1.csv' not found"));
        };
        let s = String::from_utf8(bytes)
            .map_err(|e| anyhow!("invalid utf8 in CSV: {e}"))?;
        if !s.contains("URI Scheme") {
            return Err(anyhow!(
                "CSV does not contain expected header 'URI Scheme'"
            ));
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_scheme_in_variants() -> Result<()> {
        // positive case
        if !scheme_in("aaa:resource", vec!["aaa"]) {
            return Err(anyhow!("scheme_in failed to recognize 'aaa'"));
        }
        // missing colon -> false
        if scheme_in("no-colon", vec!["no-colon"]) {
            return Err(anyhow!(
                "scheme_in should be false when no ':' present"
            ));
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_list_iana_schemes_contains_expected() -> Result<()> {
        let schemes = list_iana_schemes()?;
        if !schemes.iter().any(|s| s.eq_ignore_ascii_case("aaa")) {
            return Err(anyhow!("expected scheme 'aaa' missing from list"));
        }
        if !schemes.iter().any(|s| s.eq_ignore_ascii_case("z39.50s")) {
            return Err(anyhow!("expected scheme 'z39.50s' missing from list"));
        }
        if schemes.iter().any(|s| s.eq_ignore_ascii_case("URI Scheme")) {
            return Err(anyhow!(
                "header 'URI Scheme' should not be present in schemes list"
            ));
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_list_permanent_iana_schemes_contains_expected() -> Result<()> {
        let schemes = list_permanent_iana_schemes()?;
        if !schemes.iter().any(|s| s.eq_ignore_ascii_case("http")) {
            return Err(anyhow!(
                "expected scheme 'http' missing from permanent schemes list"
            ));
        }
        if schemes.iter().any(|s| s.eq_ignore_ascii_case("URI Scheme")) {
            return Err(anyhow!(
                "header 'URI Scheme' should not be present in permanent schemes list"
            ));
        }
        if schemes.iter().any(|s| s.eq_ignore_ascii_case("acd")) {
            return Err(anyhow!(
                "provisional scheme 'acd' should not be present in permanent schemes list"
            ));
        }
        if schemes.iter().any(|s| s.eq_ignore_ascii_case("bb")) {
            return Err(anyhow!(
                "historic scheme 'bb' should not be present in permanent schemes list"
            ));
        }

        Ok(())
    }

    #[crate::ctb_test]
    fn test_is_iana_scheme_checks() -> Result<()> {
        if !is_iana_scheme("z39.50s:example") {
            return Err(anyhow!("is_iana_scheme failed for 'z39.50s'"));
        }
        if is_iana_scheme("URI Scheme:example") {
            return Err(anyhow!(
                "header 'URI Scheme' incorrectly classified as scheme"
            ));
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_detect_uri_and_scheme() {
        let http_scheme = detect_uri("http").unwrap();
        assert_eq!(http_scheme.format_id, FormatId::UriProtocol);
        assert!(http_scheme.is_scheme_only);
        assert_eq!(http_scheme.scheme, "http");

        let https_scheme_colon = detect_uri("https:").unwrap();
        assert_eq!(https_scheme_colon.format_id, FormatId::UriProtocol);
        assert!(https_scheme_colon.is_scheme_only);

        let full_uri = detect_uri("https://collectivetoolbox.com/path?q=1#top").unwrap();
        assert_eq!(full_uri.format_id, FormatId::Uri);
        assert!(!full_uri.is_scheme_only);
        assert_eq!(full_uri.scheme, "https");

        let mailto_uri = detect_uri("mailto:info@example.com").unwrap();
        assert_eq!(mailto_uri.format_id, FormatId::Uri);
        assert!(!mailto_uri.is_scheme_only);

        let magnet = detect_uri("magnet:?xt=urn:btih:c12fe1c06bba254a9dc9f519b335aa7c1367a88a").unwrap();
        assert_eq!(magnet.format_id, FormatId::Magnet);

        assert!(detect_uri("not a uri with spaces").is_none());
        assert!(detect_uri("unknownscheme12345:resource").is_none());
    }
}
