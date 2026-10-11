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

//! High-level APIs for converting documents and files to `DcMstring`.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::{Context, Result, bail};
use std::path::Path;

use ctb_formats_utilities::FormatLog;
use ctb_formats_utilities::extension_data::lookup_format_by_extension;
use ctb_utilities::format_support::SupportedDcInputFormat;
use ctb_utilities::FormatId;

use crate::{
    DCMX_MAGIC, DcChar, DcMixed, DcMstr, DcMstring, DcStr, DcString,
};

/// Specification for input format: explicit format or automatic detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FormatSpec {
    /// Autodetect format from file extension, magic bytes, and content sniffing.
    #[default]
    Auto,
    /// Expect a specific supported Dc input format.
    Format(SupportedDcInputFormat),
    /// Expect an arbitrary FormatId.
    FormatId(FormatId),
}

impl From<SupportedDcInputFormat> for FormatSpec {
    fn from(fmt: SupportedDcInputFormat) -> Self {
        Self::Format(fmt)
    }
}

impl From<FormatId> for FormatSpec {
    fn from(id: FormatId) -> Self {
        Self::FormatId(id)
    }
}

/// Options controlling conversion to `DcMstring`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DcConversionOptions {
    /// Input format specification (default: Auto).
    pub format: FormatSpec,
    /// If format cannot be decoded as a known Dc/text document, encapsulate raw
    /// bytes directly as `DcMixed::Binary`.
    pub allow_binary_encapsulation: bool,
}

impl Default for DcConversionOptions {
    fn default() -> Self {
        Self {
            format: FormatSpec::Auto,
            allow_binary_encapsulation: false,
        }
    }
}

/// Result of a format conversion including the parsed `DcMstring`, detected format,
/// and accumulated diagnostic log.
#[derive(Debug)]
pub struct DcConversionResult {
    /// The decoded `DcMstring` document.
    pub document: DcMstring,
    /// The detected or specified input `FormatId`.
    pub detected_format: FormatId,
    /// Format conversion log containing warnings, errors, and debug notices.
    pub log: FormatLog,
}

/// Returns the list of all supported input formats for `DcMstring` conversion.
#[must_use]
pub fn list_supported_input_formats() -> &'static [FormatId] {
    SupportedDcInputFormat::ALL_FORMAT_IDS
}

/// Detects the most appropriate `SupportedDcInputFormat` from content and optional filename.
#[must_use]
pub fn detect_supported_dc_format(
    bytes: Option<&[u8]>,
    filename_or_ext: Option<&str>,
) -> Option<SupportedDcInputFormat> {
    // 1. Check magic header for DcMixed (.dcmm)
    if let Some(b) = bytes {
        if b.starts_with(&DCMX_MAGIC) {
            return Some(SupportedDcInputFormat::DcMixed);
        }
    }

    // 2. Check filename extension
    if let Some(name) = filename_or_ext {
        let clean = name.trim().trim_start_matches('.');
        let ext = if let Some(dot_idx) = clean.rfind('.') {
            if let Some(sub) = clean.get(dot_idx.saturating_add(1)..) {
                sub
            } else {
                clean
            }
        } else {
            clean
        };

        match ext.to_ascii_lowercase().as_str() {
            "dctext" => return Some(SupportedDcInputFormat::DcText),
            "dcts" => return Some(SupportedDcInputFormat::Dcts),
            "dcal" => return Some(SupportedDcInputFormat::Dcal),
            "dcil" => return Some(SupportedDcInputFormat::DcIntegerList),
            "dcmm" => return Some(SupportedDcInputFormat::DcMixed),
            "dcutf" => return Some(SupportedDcInputFormat::DcUtf),
            "sems" => return Some(SupportedDcInputFormat::Sems),
            "ascii" => return Some(SupportedDcInputFormat::Ascii),
            "txt" | "utf8" | "utf-8" => return Some(SupportedDcInputFormat::Utf8),
            _ => {}
        }

        for fid in lookup_format_by_extension(ext) {
            if let Some(supported) = SupportedDcInputFormat::from_format_id(fid) {
                return Some(supported);
            }
        }
    }

    // 3. Content sniffing
    if let Some(b) = bytes {
        if b.is_empty() {
            return Some(SupportedDcInputFormat::Utf8);
        }

        // Check if plain UTF-8
        if let Ok(text) = std::str::from_utf8(b) {
            // Check for DcText tokens @...
            if text.contains('@') {
                let has_dctext_token = text.split('@').enumerate().any(|(idx, part)| {
                    // Odd indices are between @ signs
                    if idx % 2 == 1 && !part.is_empty() {
                        part.chars().all(|c| c.is_ascii_digit())
                            || part.starts_with('L')
                            || part.starts_with('l')
                            || part.starts_with('u')
                            || part.starts_with('f')
                    } else {
                        false
                    }
                });
                if has_dctext_token {
                    return Some(SupportedDcInputFormat::DcText);
                }
            }

            // Check for space/newline-separated integer list
            let trimmed = text.trim();
            if !trimmed.is_empty()
                && trimmed
                    .split_whitespace()
                    .all(|tok| tok.chars().all(|c| c.is_ascii_digit()))
            {
                // If any number exceeds short Dc range (1114111), it's Dcal (global IDs)
                let has_large_id = trimmed.split_whitespace().any(|tok| {
                    if let Ok(num) = tok.parse::<u128>() {
                        num > 1_114_111
                    } else {
                        false
                    }
                });
                if has_large_id {
                    return Some(SupportedDcInputFormat::Dcal);
                }
                return Some(SupportedDcInputFormat::DcIntegerList);
            }

            return Some(SupportedDcInputFormat::Utf8);
        }
    }

    None
}

fn dclist_to_dcmstring(dclist: &[u128]) -> Result<DcMstring> {
    let dcutf_bytes = crate::converters::dctext::dclist_to_dcutf(dclist);
    let dc_str = DcStr::from_bytes(&dcutf_bytes)
        .context("Converting dclist to valid UTF-8e-128 DcStr")?;
    DcMstring::from_dc_str(dc_str)
}

/// Converts in-memory bytes with an optional filename hint to a `DcMstring`.
pub fn convert_bytes_to_dcmstring(
    bytes: &[u8],
    filename_hint: Option<&str>,
    options: &DcConversionOptions,
) -> Result<DcConversionResult> {
    let resolved_format: SupportedDcInputFormat = match options.format {
        FormatSpec::Format(fmt) => fmt,
        FormatSpec::FormatId(id) => {
            if let Some(supported) = SupportedDcInputFormat::from_format_id(id) {
                supported
            } else {
                bail!(
                    "Format {id:?} is not supported for conversion to DcMstring. Supported formats: {:?}",
                    SupportedDcInputFormat::ALL
                );
            }
        }
        FormatSpec::Auto => {
            if let Some(detected) = detect_supported_dc_format(Some(bytes), filename_hint) {
                detected
            } else if options.allow_binary_encapsulation {
                let mut doc = DcMstring::new();
                doc.push_binary(bytes);
                return Ok(DcConversionResult {
                    document: doc,
                    detected_format: FormatId::DcMixed,
                    log: FormatLog::default(),
                });
            } else {
                bail!(
                    "Unable to autodetect document format for input. Specify --from with one of: {:?}",
                    SupportedDcInputFormat::ALL
                );
            }
        }
    };

    let mut log = FormatLog::default();
    let document = match resolved_format {
        SupportedDcInputFormat::Utf8 => {
            let text = std::str::from_utf8(bytes).with_context(|| {
                "Input is not valid UTF-8 text\n\nNOTE: Parsing as UTF-8. If you want DcText, pass `--from dctext` on the command line."
            })?;
            let dc_str = DcStr::from_bytes(text.as_bytes())
                .context("UTF-8 text contains invalid Dc sequences")?;
            DcMstring::from_dc_str(dc_str)?
        }
        SupportedDcInputFormat::DcText => {
            let conv = crate::converters::dctext::dctext_to_dcstring(bytes)?;
            log.merge(&conv.log);
            DcMstring::from_dc_string(&conv.result)?
        }
        SupportedDcInputFormat::Dcts => {
            let conv = crate::converters::dcts::dcts_to_dcstring(bytes)?;
            log.merge(&conv.log);
            DcMstring::from_dc_string(&conv.result)?
        }
        SupportedDcInputFormat::Dcal => {
            let conv = crate::converters::dcal::dcal_to_dclist(bytes)?;
            log.merge(&conv.log);
            dclist_to_dcmstring(&conv.result)?
        }
        SupportedDcInputFormat::DcIntegerList => {
            let (dca, dca_log) = ctb_formats_eite::formats::integer_list::dca_from_integer_list(
                bytes,
                &ctb_formats_eite::formats::integer_list::IntegerListFormatSettings::default(),
            )?;
            log.merge(&dca_log);
            let conv = crate::converters::dctext::dcarray_to_dclist(&dca)?;
            log.merge(&conv.log);
            dclist_to_dcmstring(&conv.result)?
        }
        SupportedDcInputFormat::DcMixed => {
            DcMstring::from_slice(bytes)
                .context("Input bytes do not conform to valid DcMixed (.dcmm) format")?
        }
        SupportedDcInputFormat::DcUtf => {
            let dc_str = DcStr::from_bytes(bytes)
                .context("Input bytes do not conform to valid DcUtf format")?;
            DcMstring::from_dc_str(dc_str)?
        }
        SupportedDcInputFormat::Sems => {
            let (dca, dca_log) = ctb_formats_eite::formats::sems::dca_from_sems(
                bytes,
                &ctb_formats_eite::formats::sems::SEMSFormatSettings::default(),
            )?;
            log.merge(&dca_log);
            let conv = crate::converters::dctext::dcarray_to_dclist(&dca)?;
            log.merge(&conv.log);
            dclist_to_dcmstring(&conv.result)?
        }
        SupportedDcInputFormat::Ascii => {
            let (dca, dca_log) = ctb_formats_eite::formats::ascii::dca_from_ascii(
                bytes,
            )?;
            log.merge(&dca_log);
            let conv = crate::converters::dctext::dcarray_to_dclist(&dca)?;
            log.merge(&conv.log);
            dclist_to_dcmstring(&conv.result)?
        }
        SupportedDcInputFormat::AsciiSafeSubset => {
            let (dca, dca_log) = ctb_formats_eite::formats::ascii::dca_from_ascii_safe_subset(
                bytes,
                &ctb_formats_eite::formats::ascii::AsciiSafeSubsetFormatSettings::default(),
            )?;
            log.merge(&dca_log);
            let conv = crate::converters::dctext::dcarray_to_dclist(&dca)?;
            log.merge(&conv.log);
            dclist_to_dcmstring(&conv.result)?
        }
    };

    Ok(DcConversionResult {
        document,
        detected_format: resolved_format.to_format_id(),
        log,
    })
}

/// Converts a file on disk (or '-' for stdin) to a `DcMstring`.
pub fn convert_file_to_dcmstring(
    path: &Path,
    options: &DcConversionOptions,
) -> Result<DcConversionResult> {
    let (bytes, filename_hint) = if path == Path::new("-") {
        let mut buf = Vec::new();
        std::io::Read::read_to_end(&mut std::io::stdin(), &mut buf)
            .context("Reading document from standard input")?;
        (buf, None)
    } else {
        let data = std::fs::read(path).with_context(|| {
            format!("Failed to read document file: {}", path.display())
        })?;
        let name = path.file_name().map(|n| n.to_string_lossy().into_owned());
        (data, name)
    };

    convert_bytes_to_dcmstring(&bytes, filename_hint.as_deref(), options)
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
    fn test_convert_utf8() {
        let input = b"Hello, world!";
        let res = convert_bytes_to_dcmstring(
            input,
            Some("test.txt"),
            &DcConversionOptions::default(),
        )
        .expect("convert utf8");
        assert_eq!(res.detected_format, FormatId::Utf8);
        assert_eq!(
            res.document.to_dc_string().unwrap().as_str().unwrap(),
            "Hello, world!"
        );
    }

    #[crate::ctb_test]
    fn test_convert_dcal() {
        let input = b"65 66 67";
        let res = convert_bytes_to_dcmstring(
            input,
            Some("test.dcal"),
            &DcConversionOptions::default(),
        )
        .expect("convert dcal");
        assert_eq!(res.detected_format, FormatId::Dcal);
        assert_eq!(
            res.document.to_dc_string().unwrap().as_str().unwrap(),
            "ABC"
        );
    }

    #[crate::ctb_test]
    fn test_convert_dctext() {
        let input = b"Hello @65@!";
        let res = convert_bytes_to_dcmstring(
            input,
            Some("test.dctext"),
            &DcConversionOptions::default(),
        )
        .expect("convert dctext");
        assert_eq!(res.detected_format, FormatId::DcText);
        assert_eq!(
            res.document.to_dc_string().unwrap().as_str().unwrap(),
            "Hello A!"
        );
    }

    #[crate::ctb_test]
    fn test_convert_dcmm() {
        let mut dcmm = DcMstring::new();
        dcmm.push_str("Mixed document: ");
        dcmm.push_binary(b"raw-bytes");
        let bytes = dcmm.as_bytes();

        let res = convert_bytes_to_dcmstring(
            bytes,
            Some("test.dcmm"),
            &DcConversionOptions::default(),
        )
        .expect("convert dcmm");
        assert_eq!(res.detected_format, FormatId::DcMixed);
        assert_eq!(res.document.as_bytes(), bytes);
    }
}
