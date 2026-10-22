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

//! Bidirectional converters between `DcMstring` and 7-bit ASCII documents,
//! including the ASCII Safe Subset (printable ASCII with CRLF line endings).

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::{Result, anyhow, bail};
use const_default::ConstDefault;
use ctb_formats_utilities::FormatLog;

use crate::DcMstring;

/// Policy for handling characters in `DcMstring` that cannot be directly
/// represented as 7-bit ASCII during export.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AsciiUnmappablePolicy {
    /// Skip unmappable characters and record an export warning in the
    /// `FormatLog` (default).
    #[default]
    Skip,
    /// Replace unmappable characters with an explicit ASCII replacement byte
    /// (e.g. `b'?'`) and record an export warning.
    Replace(u8),
    /// Transliterate Unicode characters to ASCII approximations (using
    /// `ctb_formats_utf8::to_ascii_translit`), falling back to skipping any
    /// characters that still cannot be represented.
    Transliterate,
    /// Stub for future generalized fallback encoding handler / transformation
    /// pipeline defined in `transformations.csv`.
    TransformationPipeline,
}

/// Configuration settings for ASCII conversion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsciiSettings {
    /// Policy for characters that cannot be represented in 7-bit ASCII.
    pub unmappable_policy: AsciiUnmappablePolicy,
    /// When true, rejects any non-ASCII bytes on import with an error. When
    /// false, replaces non-ASCII bytes with replacement characters.
    pub strict_import: bool,
}

impl Default for AsciiSettings {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl ConstDefault for AsciiSettings {
    const DEFAULT: Self = Self {
        unmappable_policy: AsciiUnmappablePolicy::Skip,
        strict_import: true,
    };
}

/// Converts a 7-bit ASCII text document to `DcMstring` using default settings.
pub fn ascii_to_dcmstring(bytes: &[u8]) -> Result<(DcMstring, FormatLog)> {
    ascii_to_dcmstring_with_settings(bytes, &AsciiSettings::default())
}

/// Converts a 7-bit ASCII text document to `DcMstring` with custom settings.
pub fn ascii_to_dcmstring_with_settings(
    bytes: &[u8],
    settings: &AsciiSettings,
) -> Result<(DcMstring, FormatLog)> {
    let mut log = FormatLog::default();

    if settings.strict_import {
        for (idx, &b) in bytes.iter().enumerate() {
            if b > 0x7F {
                bail!(
                    "Input is not valid ASCII: non-ASCII byte 0x{b:02X} at index {idx}"
                );
            }
        }
        let (doc, utf8_log) =
            crate::converters::utf8::utf8_to_dcmstring(bytes)?;
        log.merge(&utf8_log);
        Ok((doc, log))
    } else {
        let mut sanitized = Vec::with_capacity(bytes.len());
        for (idx, &b) in bytes.iter().enumerate() {
            if b <= 0x7F {
                sanitized.push(b);
            } else {
                let idx_u64 = u64::try_from(idx)
                    .map_err(|e| anyhow!("Index conversion overflow: {e}"))?;
                log.import_warning(
                    idx_u64,
                    &format!("Replaced non-ASCII byte 0x{b:02X} with replacement character"),
                );
                sanitized.extend_from_slice(ctb_formats_utf8::UTF8_REPLACEMENT_CHARACTER);
            }
        }
        let (doc, utf8_log) =
            crate::converters::utf8::utf8_to_dcmstring(&sanitized)?;
        log.merge(&utf8_log);
        Ok((doc, log))
    }
}

/// Converts a `DcMstring` to 7-bit ASCII document bytes using default settings.
pub fn dcmstring_to_ascii(doc: &DcMstring) -> Result<(Vec<u8>, FormatLog)> {
    dcmstring_to_ascii_with_settings(doc, &AsciiSettings::default())
}

/// Converts a `DcMstring` to 7-bit ASCII document bytes with custom settings.
pub fn dcmstring_to_ascii_with_settings(
    doc: &DcMstring,
    settings: &AsciiSettings,
) -> Result<(Vec<u8>, FormatLog)> {
    let (utf8_bytes, mut log) =
        crate::converters::utf8::dcmstring_to_utf8(doc)?;

    // Fast-path: if all bytes are valid 7-bit ASCII, return immediately.
    if utf8_bytes.iter().all(|&b| b <= 0x7F) {
        return Ok((utf8_bytes, log));
    }

    let utf8_text = std::str::from_utf8(&utf8_bytes)
        .map_err(|e| anyhow!("Internal UTF-8 conversion error: {e}"))?;

    match settings.unmappable_policy {
        AsciiUnmappablePolicy::Transliterate => {
            let transliterated =
                ctb_formats_utf8::to_ascii_translit(utf8_text);
            let mut out = Vec::with_capacity(transliterated.len());
            for (idx, ch) in transliterated.chars().enumerate() {
                if ch.is_ascii() {
                    let mut buf = [0u8; 4];
                    let encoded = ch.encode_utf8(&mut buf);
                    if let Some(&first) = encoded.as_bytes().first() {
                        out.push(first);
                    }
                } else {
                    let idx_u64 = u64::try_from(idx).map_err(|e| {
                        anyhow!("Index conversion overflow: {e}")
                    })?;
                    let cp = u32::from(ch);
                    log.export_warning(
                        idx_u64,
                        &format!(
                            "Character '{ch}' (U+{cp:04X}) remains unmappable after transliteration"
                        ),
                    );
                }
            }
            Ok((out, log))
        }
        AsciiUnmappablePolicy::TransformationPipeline => {
            // TODO: Hook into generalized Dc transformation pipeline defined
            // via transformations.csv. For now, fallback to transliteration.
            let transliterated =
                ctb_formats_utf8::to_ascii_translit(utf8_text);
            let mut out = Vec::with_capacity(transliterated.len());
            for ch in transliterated.chars() {
                if ch.is_ascii() {
                    let mut buf = [0u8; 4];
                    let encoded = ch.encode_utf8(&mut buf);
                    if let Some(&first) = encoded.as_bytes().first() {
                        out.push(first);
                    }
                }
            }
            Ok((out, log))
        }
        AsciiUnmappablePolicy::Replace(replacement_byte) => {
            if replacement_byte > 0x7F {
                bail!(
                    "Replacement byte 0x{replacement_byte:02X} is not a valid 7-bit ASCII character"
                );
            }
            let mut out = Vec::with_capacity(utf8_text.len());
            for (idx, ch) in utf8_text.chars().enumerate() {
                if ch.is_ascii() {
                    let mut buf = [0u8; 4];
                    let encoded = ch.encode_utf8(&mut buf);
                    if let Some(&first) = encoded.as_bytes().first() {
                        out.push(first);
                    }
                } else {
                    let idx_u64 = u64::try_from(idx).map_err(|e| {
                        anyhow!("Index conversion overflow: {e}")
                    })?;
                    let cp = u32::from(ch);
                    log.export_warning(
                        idx_u64,
                        &format!(
                            "Replaced non-ASCII character '{ch}' (U+{cp:04X}) with 0x{replacement_byte:02X}"
                        ),
                    );
                    out.push(replacement_byte);
                }
            }
            Ok((out, log))
        }
        AsciiUnmappablePolicy::Skip => {
            let mut out = Vec::with_capacity(utf8_text.len());
            for (idx, ch) in utf8_text.chars().enumerate() {
                if ch.is_ascii() {
                    let mut buf = [0u8; 4];
                    let encoded = ch.encode_utf8(&mut buf);
                    if let Some(&first) = encoded.as_bytes().first() {
                        out.push(first);
                    }
                } else {
                    let idx_u64 = u64::try_from(idx).map_err(|e| {
                        anyhow!("Index conversion overflow: {e}")
                    })?;
                    let cp = u32::from(ch);
                    log.export_warning(
                        idx_u64,
                        &format!(
                            "Skipped non-ASCII character '{ch}' (U+{cp:04X})"
                        ),
                    );
                }
            }
            Ok((out, log))
        }
    }
}

/// Returns true if the byte is considered printable ASCII (`0x20..=0x7E`).
pub fn is_ascii_printable(b: u8) -> bool {
    b >= 0x20 && b <= 0x7E
}

/// Returns true if the byte is an ASCII newline (`\r` or `\n`).
pub fn is_ascii_newline(b: u8) -> bool {
    b == b'\n' || b == b'\r'
}

/// Returns true if the byte is an allowed ASCII Safe Subset character
/// (printable ASCII or newline).
pub fn is_ascii_safe_subset_char(b: u8) -> bool {
    is_ascii_printable(b) || is_ascii_newline(b)
}

/// Converts an ASCII Safe Subset text document to `DcMstring`.
pub fn ascii_safe_subset_to_dcmstring(
    bytes: &[u8],
) -> Result<(DcMstring, FormatLog)> {
    ascii_safe_subset_to_dcmstring_with_settings(bytes, &AsciiSettings::default())
}

/// Converts an ASCII Safe Subset text document to `DcMstring` with custom settings.
pub fn ascii_safe_subset_to_dcmstring_with_settings(
    bytes: &[u8],
    settings: &AsciiSettings,
) -> Result<(DcMstring, FormatLog)> {
    let mut normalized = Vec::with_capacity(bytes.len());
    let mut i = 0usize;

    while i < bytes.len() {
        let Some(&b) = bytes.get(i) else { break };
        if !is_ascii_safe_subset_char(b) {
            bail!(
                "Invalid ASCII Safe Subset byte: 0x{b:02X} at index {i}"
            );
        }

        if b == b'\r' {
            normalized.push(b'\n');
            if bytes.get(i.saturating_add(1)) == Some(&b'\n') {
                i = i.saturating_add(2);
                continue;
            }
        } else {
            normalized.push(b);
        }
        i = i.saturating_add(1);
    }

    ascii_to_dcmstring_with_settings(&normalized, settings)
}

/// Converts a `DcMstring` to ASCII Safe Subset document bytes (printable ASCII
/// with CRLF newlines) using default settings.
pub fn dcmstring_to_ascii_safe_subset(
    doc: &DcMstring,
) -> Result<(Vec<u8>, FormatLog)> {
    dcmstring_to_ascii_safe_subset_with_settings(doc, &AsciiSettings::default())
}

/// Converts a `DcMstring` to ASCII Safe Subset document bytes with custom settings.
pub fn dcmstring_to_ascii_safe_subset_with_settings(
    doc: &DcMstring,
    settings: &AsciiSettings,
) -> Result<(Vec<u8>, FormatLog)> {
    let (ascii_bytes, mut log) =
        dcmstring_to_ascii_with_settings(doc, settings)?;

    let mut out = Vec::with_capacity(ascii_bytes.len().saturating_add(16));
    let mut i = 0usize;

    while i < ascii_bytes.len() {
        let Some(&b) = ascii_bytes.get(i) else { break };

        if b == b'\r' {
            out.extend_from_slice(b"\r\n");
            if ascii_bytes.get(i.saturating_add(1)) == Some(&b'\n') {
                i = i.saturating_add(2);
                continue;
            }
        } else if b == b'\n' {
            out.extend_from_slice(b"\r\n");
        } else if is_ascii_printable(b) {
            out.push(b);
        } else {
            let idx_u64 = u64::try_from(i)
                .map_err(|e| anyhow!("Index conversion overflow: {e}"))?;
            log.export_warning(
                idx_u64,
                &format!(
                    "Control byte 0x{b:02X} is not permitted in ASCII Safe Subset"
                ),
            );
        }

        i = i.saturating_add(1);
    }

    Ok((out, log))
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
    fn test_ascii_roundtrip_basic() {
        let input = b"Hello, World!\nTesting ASCII 123.";
        let (doc, log) = ascii_to_dcmstring(input).expect("parse ascii");
        assert!(log.has_no_warnings_or_errors());
        assert_eq!(doc, "Hello, World!\nTesting ASCII 123.");

        let (bytes, export_log) = dcmstring_to_ascii(&doc).expect("export ascii");
        assert!(export_log.has_no_warnings_or_errors());
        assert_eq!(bytes, input);
    }

    #[crate::ctb_test]
    fn test_ascii_strict_rejects_non_ascii() {
        let invalid = b"Hello \xFF World";
        let res = ascii_to_dcmstring(invalid);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("0xFF"));
    }

    #[crate::ctb_test]
    fn test_ascii_permissive_replaces_non_ascii() {
        let invalid = b"Hello \x80 World";
        let settings = AsciiSettings {
            unmappable_policy: AsciiUnmappablePolicy::Skip,
            strict_import: false,
        };
        let (doc, log) =
            ascii_to_dcmstring_with_settings(invalid, &settings).expect("parse");
        assert!(log.has_warnings());
        assert_eq!(doc, "Hello \u{FFFD} World");

        let byte_test = b"a\xffb";
        let (doc2, log2) =
            ascii_to_dcmstring_with_settings(byte_test, &settings).expect("parse");
        assert!(log2.has_warnings());
        assert_eq!(doc2, "a\u{FFFD}b");
    }

    #[crate::ctb_test]
    fn test_ascii_export_transliterate() {
        let (doc, _) =
            crate::converters::utf8::utf8_to_dcmstring("Café naïve".as_bytes())
                .expect("parse utf8");
        let settings = AsciiSettings {
            unmappable_policy: AsciiUnmappablePolicy::Transliterate,
            strict_import: true,
        };
        let (bytes, _) =
            dcmstring_to_ascii_with_settings(&doc, &settings).expect("export");
        assert_eq!(std::str::from_utf8(&bytes).unwrap(), "Cafe naive");
    }

    #[crate::ctb_test]
    fn test_ascii_export_replace() {
        let (doc, _) =
            crate::converters::utf8::utf8_to_dcmstring("Hello 🌍!".as_bytes())
                .expect("parse utf8");
        let settings = AsciiSettings {
            unmappable_policy: AsciiUnmappablePolicy::Replace(b'?'),
            strict_import: true,
        };
        let (bytes, log) =
            dcmstring_to_ascii_with_settings(&doc, &settings).expect("export");
        assert_eq!(std::str::from_utf8(&bytes).unwrap(), "Hello ?!");
        assert!(log.has_warnings());
    }

    #[crate::ctb_test]
    fn test_ascii_export_skip() {
        let (doc, _) =
            crate::converters::utf8::utf8_to_dcmstring("Hello 🌍!".as_bytes())
                .expect("parse utf8");
        let (bytes, log) = dcmstring_to_ascii(&doc).expect("export");
        assert_eq!(std::str::from_utf8(&bytes).unwrap(), "Hello !");
        assert!(log.has_warnings());
    }

    #[crate::ctb_test]
    fn test_ascii_safe_subset_roundtrip() {
        let input = b"Line 1\r\nLine 2\r\n";
        let (doc, log) =
            ascii_safe_subset_to_dcmstring(input).expect("parse safe subset");
        assert!(log.has_no_warnings_or_errors());
        assert_eq!(doc, "Line 1\nLine 2\n");

        let (out, out_log) =
            dcmstring_to_ascii_safe_subset(&doc).expect("export safe subset");
        assert!(out_log.has_no_warnings_or_errors());
        assert_eq!(out, input);
    }

    #[crate::ctb_test]
    fn test_ascii_safe_subset_rejects_control_chars() {
        // Tab (0x09) and Bell (0x07) are not printable ASCII
        let input = b"Hello\tWorld\n";
        let res = ascii_safe_subset_to_dcmstring(input);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("0x09"));
    }

    #[crate::ctb_test]
    fn test_ascii_safe_subset_export_normalizes_crlf_and_filters_controls() {
        let (doc, _) =
            crate::converters::utf8::utf8_to_dcmstring("Line 1\twith tab\nLine 2".as_bytes())
                .expect("parse utf8");
        let (out, log) =
            dcmstring_to_ascii_safe_subset(&doc).expect("export safe subset");
        // Tab was filtered out with a warning, newline became CRLF
        assert_eq!(out, b"Line 1with tab\r\nLine 2");
        assert!(log.has_warnings());
    }

}
