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

//! Apple Uniform Type Identifier (UTI) and 4-character OS / Creator Type
//! syntax parsing, validation, and detection.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

/// Structured detection result for an Apple Uniform Type Identifier (UTI).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppleUtiDetection {
    /// Normalized lowercase UTI string (e.g. `public.jpeg`, `com.apple.alias-file`).
    pub uti: String,
    /// Human-readable descriptive label for detection reporting.
    pub description: String,
}

/// Structured detection result for an Apple 4-character OS / Creator Type code (OSType).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppleTypeCodeDetection {
    /// Four-byte raw character code array (e.g. `*b"alis"`, `*b"TEXT"`).
    pub code: [u8; 4],
    /// Human-readable descriptive label for detection reporting.
    pub description: String,
}

/// Checks whether a string conforms to Apple Uniform Type Identifier (UTI) syntax.
#[must_use]
pub fn is_valid_apple_uti(s: &str) -> bool {
    let trimmed = s.trim();
    if trimmed.is_empty() || trimmed.contains(' ') || trimmed.contains('/') {
        return false;
    }

    let is_known_prefix = trimmed.starts_with("public.")
        || trimmed.starts_with("com.apple.")
        || trimmed.starts_with("com.adobe.")
        || trimmed.starts_with("dyn.");

    let is_dot_separated = trimmed.contains('.')
        && trimmed
            .split('.')
            .all(|seg| !seg.is_empty() && seg.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-'));

    is_known_prefix || is_dot_separated
}

/// Detects and validates an Apple Uniform Type Identifier (UTI) candidate string.
#[must_use]
pub fn detect_apple_uti(s: &str) -> Option<AppleUtiDetection> {
    let trimmed = s.trim();
    if !is_valid_apple_uti(trimmed) {
        return None;
    }

    Some(AppleUtiDetection {
        uti: trimmed.to_ascii_lowercase(),
        description: format!("Apple Uniform Type Identifier (UTI): {trimmed}"),
    })
}

/// Checks whether a byte slice represents a valid 4-character Mac OS type code.
#[must_use]
pub fn is_valid_apple_type_code(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.len() != 4 {
        return false;
    }
    bytes.iter().all(|&b| b.is_ascii_graphic() || b == b' ')
}

/// Detects and validates an Apple 4-character OS / Creator Type code (OSType).
#[must_use]
pub fn detect_apple_type_code(s: &str) -> Option<AppleTypeCodeDetection> {
    let trimmed = s.trim();
    if !is_valid_apple_type_code(trimmed) {
        return None;
    }

    let bytes = <[u8; 4]>::try_from(trimmed.as_bytes()).ok()?;
    Some(AppleTypeCodeDetection {
        code: bytes,
        description: format!("Apple OS / Creator Type: '{trimmed}'"),
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
    fn test_apple_uti_detection() {
        assert!(is_valid_apple_uti("public.jpeg"));
        assert!(is_valid_apple_uti("com.apple.alias-file"));
        assert!(is_valid_apple_uti("com.adobe.pdf"));
        assert!(is_valid_apple_uti("org.videolan.vlc"));

        let uti = detect_apple_uti("public.png").unwrap();
        assert_eq!(uti.uti, "public.png");

        assert!(!is_valid_apple_uti(""));
        assert!(!is_valid_apple_uti("no-dots"));
        assert!(!is_valid_apple_uti("has space.uti"));
        assert!(!is_valid_apple_uti("path/like.uti"));
    }

    #[crate::ctb_test]
    fn test_apple_type_code_detection() {
        assert!(is_valid_apple_type_code("TEXT"));
        assert!(is_valid_apple_type_code("alis"));
        assert!(is_valid_apple_type_code("PDF "));
        assert!(is_valid_apple_type_code("PNGf"));

        let code = detect_apple_type_code("alis").unwrap();
        assert_eq!(code.code, *b"alis");

        assert!(!is_valid_apple_type_code("TEX"));
        assert!(!is_valid_apple_type_code("TEXT5"));
        assert!(!is_valid_apple_type_code(""));
    }
}
