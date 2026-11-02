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

//! Conversion between `dceutils` formats (`CdceLegacy`, `Dce`, `Dce_3_0a`, `Dce_3_01a`, etc.) and `DcMstring`.
//!
//! Supported short Dc repertoires by DCE format version:
//! - **DCE 3.0a (`3_0a`, `3_0a_raw`, `hex_3_0a`, `hex_3_0a_raw`)**: short Dcs `0..=115`
//!   (excluding `2..=12`, which had no byte mappings in 3.0a).
//! - **DCE 3.01a (`3_01a`, `3_01a_raw`, `hex_3_01a`, `hex_3_01a_raw`) and `dce` / `hex_dce`**:
//!   short Dcs `0..=199`, unlocking Base64 encapsulation (`Dc 191` .. `Dc 127..=190` / `Dc 195` .. `Dc 192`)
//!   for data and UTF-8.
//! - **DCE 3.02a (`3_02a`, `3_02a_raw`, `hex_3_02a`, `hex_3_02a_raw`)**: stubbed (returns an error when
//!   invoked; version selector in `DCE_Version_Selectors` is TBD).
//!
//! Unlike EITE formats, `dceutils` formats do not support `Dc 308` (`Next number is a long (global graph) Dc`).
//! Unrepresentable characters are logged as warnings in [`FormatLog`], while DCE stateful-encoding control
//! characters (`13..=16`, `112..=115`, `194`, `196..=199`, `205`) are preserved as-is when converting to Dc documents.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::{Context, Result, bail};
use ctb_formats_dcstring::dc::{SHORT_DC_REGION_END, SHORT_DC_REGION_START};
use ctb_formats_utilities::FormatLog;

use ctb_formats_dcstring::converters::dctext::dclist_to_dctf;
use ctb_formats_dcstring::{DcChar, DcMixedChunk, DcMstring, DcStr};

fn short_dcs_to_dcmstring(short_dcs: &[u32]) -> Result<DcMstring> {
    let mut dclist = Vec::with_capacity(short_dcs.len());
    for &short_dc in short_dcs {
        dclist.push(DcChar::from_short(short_dc).to_long());
    }
    let dctf_bytes = dclist_to_dctf(&dclist);
    let dc_str = DcStr::from_bytes(&dctf_bytes)
        .context("Converting dceutils short Dc list to valid DcTF DcStr")?;
    DcMstring::from_dc_str(dc_str)
}

fn parse_comma_short_dcs(
    csv_str: &str,
    log: &mut FormatLog,
) -> Result<Vec<u32>> {
    let trimmed = csv_str.trim().trim_end_matches(',');
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for (idx, token) in trimmed.split(',').enumerate() {
        let piece = token.trim();
        if piece.is_empty() {
            out.push(0);
            continue;
        }
        match piece.parse::<u32>() {
            Ok(val) => out.push(val),
            Err(_) => {
                // Reason for fallback: diagnostic token index saturates at u64::MAX if usize exceeds 64 bits.
                log.import_error(
                    u64::try_from(idx).unwrap_or(u64::MAX),
                    &format!(
                        "Invalid short Dc ID token '{piece}' in dceutils stream"
                    ),
                );
            }
        }
    }
    Ok(out)
}

fn finalize_import_log(
    raw_log: FormatLog,
    permissive: bool,
    format_name: &str,
) -> Result<FormatLog> {
    if !permissive && raw_log.has_errors() {
        let errs = raw_log.get_errors().join("; ");
        bail!("Input does not conform to valid {format_name} format: {errs}");
    }
    if permissive && raw_log.has_errors() {
        let mut downgraded = FormatLog::default();
        for w in raw_log.get_warnings() {
            downgraded.warn(&w);
        }
        for err in raw_log.get_errors() {
            downgraded.warn(&err);
        }
        Ok(downgraded)
    } else {
        Ok(raw_log)
    }
}

/// Converts a `dceutils` format byte slice to `DcMstring`.
///
/// # Errors
/// Returns an error if the input is malformed for `format_code` (unless `permissive` is true and recovery succeeds).
pub fn dceutils_to_dcmstring_with_options(
    bytes: &[u8],
    format_code: &str,
    permissive: bool,
) -> Result<(DcMstring, FormatLog)> {
    let mut raw_log = FormatLog::default();
    let doc = match format_code {
        "cdce" | "legacy_cdce" | "cdce_legacy" | "cdce_lstrict" | "cdce_legacy_strict" => {
            crate::converter::decode_cdce(
                bytes,
                format_code != "cdce",
                matches!(format_code, "cdce_lstrict" | "cdce_legacy_strict"),
                permissive,
                &mut raw_log,
            )?
        }
        "dce" | "hex_dce" | "dce_hex" | "3_0a" | "dce_3_0a" | "3_0a_raw"
        | "dce_3_0a_raw" | "hex_3_0a" | "3_0a_hex" | "dce_3_0a_hex"
        | "hex_3_0a_raw" | "3_0a_raw_hex" | "dce_3_0a_raw_hex" | "3_01a"
        | "dce_3_01a" | "3_01a_raw" | "dce_3_01a_raw" | "hex_3_01a"
        | "3_01a_hex" | "dce_3_01a_hex" | "hex_3_01a_raw" | "3_01a_raw_hex"
        | "dce_3_01a_raw_hex" | "3_02a" | "dce_3_02a" | "3_02a_raw"
        | "dce_3_02a_raw" | "hex_3_02a" | "3_02a_hex" | "dce_3_02a_hex"
        | "hex_3_02a_raw" | "3_02a_raw_hex" | "dce_3_02a_raw_hex" => {
            let short_dcs = crate::converter::decode_dce_preserving_controls(
                bytes,
                format_code,
                &mut raw_log,
            )?;
            short_dcs_to_dcmstring(&short_dcs)?
        }
        "dc" | "dc_id_list" => {
            let s = std::str::from_utf8(bytes)
                .context("Invalid UTF-8 in DcIdList document")?;
            short_dcs_to_dcmstring(&parse_comma_short_dcs(s, &mut raw_log)?)?
        }
        "utf8" | "utf32" | "utf8_base64" | "utf8_dc64"
        | "utf8_dc64_enc"
        | "utf8_dc64_bin"
        | "utf8_dc64_bin_hex"
        | "utf8_dc64_bin_enc"
        | "utf8_dc64_bin_enc_hex" => {
            let utf8 = match format_code {
                "utf8" => bytes.to_vec(),
                "utf32" => ctb_formats_encoding::unicode::utf32be_to_utf8(bytes)
                    .map_err(|_| anyhow::anyhow!("There was an error decoding UTF-32, and permissive implementation is not yet available."))?,
                "utf8_base64" => ctb_formats_base64::base64_decode(std::str::from_utf8(bytes)?)?,
                "utf8_dc64" | "utf8_dc64_enc" => crate::converter::decode_utf8_dc64_csv_to_bytes(
                    bytes, format_code == "utf8_dc64_enc", &mut raw_log,
                )?,
                _ => crate::converter::decode_utf8_dc64_bin_to_bytes(
                    bytes, format_code.contains("_enc"), format_code.ends_with("_hex"), &mut raw_log,
                )?,
            };
            let (doc, text_log) = ctb_formats_dcstring::converters::utf8::utf8_to_dcmstring_with_options(&utf8, permissive)?;
            raw_log.merge(&text_log);
            doc
        }
        _ => bail!("Unknown input format."),
    };

    let (doc, text_log) =
        ctb_formats_dcstring::converters::utf8::repair_utf8_encapsulations(
            &doc, permissive,
        )?;
    raw_log.merge(&text_log);
    let log = finalize_import_log(raw_log, permissive, format_code)?;
    Ok((doc, log))
}

fn strip_outer_unbounded_markers(dcs: &[u32]) -> Vec<u32> {
    if dcs.len() >= 2 && dcs.first() == Some(&114) && dcs.last() == Some(&115) {
        if let Some(sub) = dcs.get(1..dcs.len().saturating_sub(1)) {
            return sub.to_vec();
        }
    }
    dcs.to_vec()
}

/// Converts a `DcMstring` into a list of short Dc IDs suitable for `dceutils` export,
/// logging warnings in `log` for any binary chunks, long Dcs (`> SHORT_DC_REGION_END`),
/// or unmappable Unicode characters (when targeting formats that do not support
/// DCE 3.01a+ Base64 encapsulation).
pub(crate) fn dcmstring_to_dceutils_short_dcs(
    doc: &ctb_formats_dcstring::DcMstr,
    target_format: &str,
    log: &mut FormatLog,
) -> Result<Vec<u32>> {
    ctb_formats_dcstring::validate_dcmst(doc.as_bytes())?;
    let mut short_dcs = Vec::new();
    let mut char_idx = 0_u64;
    let mut unmapped_utf8_buf = String::new();

    let supports_b64_encap = matches!(
        target_format,
        "dce"
            | "hex_dce"
            | "dce_hex"
            | "3_01a"
            | "dce_3_01a"
            | "3_01a_raw"
            | "dce_3_01a_raw"
            | "hex_3_01a"
            | "3_01a_hex"
            | "dce_3_01a_hex"
            | "hex_3_01a_raw"
            | "3_01a_raw_hex"
            | "dce_3_01a_raw_hex"
            | "3_02a"
            | "dce_3_02a"
            | "3_02a_raw"
            | "dce_3_02a_raw"
            | "hex_3_02a"
            | "3_02a_hex"
            | "dce_3_02a_hex"
            | "hex_3_02a_raw"
            | "3_02a_raw_hex"
            | "dce_3_02a_raw_hex"
            | "dc"
            | "dc_id_list"
            | "cdce"
            | "utf8_dc64"
            | "utf8_dc64_enc"
            | "utf8_dc64_bin"
            | "utf8_dc64_bin_hex"
            | "utf8_dc64_bin_enc"
            | "utf8_dc64_bin_enc_hex"
    );

    let flush_unmapped = |buf: &mut String, out: &mut Vec<u32>| {
        if !buf.is_empty() {
            out.extend(crate::converter::encode_utf8_to_dc64_encapsulated(
                buf.as_bytes(),
            ));
            buf.clear();
        }
    };

    for chunk in doc.chunks() {
        match chunk {
            DcMixedChunk::Binary { data, .. }
            | DcMixedChunk::BinaryWithMetadata { data, .. } => {
                flush_unmapped(&mut unmapped_utf8_buf, &mut short_dcs);
                log.export_warning(
                    char_idx,
                    &format!(
                        "Binary encapsulation ({} bytes) cannot be represented in dceutils format ({target_format}) and was omitted.",
                        data.len()
                    ),
                );
                char_idx = char_idx.saturating_add(1);
            }
            DcMixedChunk::Text(text) => {
                for DcChar(dcid) in text.chars() {
                    if (SHORT_DC_REGION_START..=SHORT_DC_REGION_END)
                        .contains(&dcid)
                    {
                        flush_unmapped(&mut unmapped_utf8_buf, &mut short_dcs);
                        let offset = dcid.saturating_sub(SHORT_DC_REGION_START);
                        if let Ok(short_id) = u32::try_from(offset) {
                            short_dcs.push(short_id);
                        }
                    } else if dcid <= 0x10_FFFF {
                        let cp = u32::try_from(dcid)
                            .context("Unicode codepoint fits in u32")?;
                        let direct = if supports_b64_encap {
                            crate::converter::unicode_codepoint_to_short_dc(cp)
                        } else {
                            crate::converter::unicode_codepoint_to_short_dc_lossy(cp)
                        };
                        if let Some(short_id) = direct {
                            flush_unmapped(
                                &mut unmapped_utf8_buf,
                                &mut short_dcs,
                            );
                            if cp == 0x2D {
                                log.export_warning(
                                    char_idx,
                                    "U+002D (HYPHEN-MINUS) mapped lossily to short Dc 31 (U+2010 HYPHEN).",
                                );
                            }
                            short_dcs.push(short_id);
                        } else if supports_b64_encap
                            && let Some(ch) = char::from_u32(cp)
                        {
                            unmapped_utf8_buf.push(ch);
                        } else {
                            flush_unmapped(
                                &mut unmapped_utf8_buf,
                                &mut short_dcs,
                            );
                            log.export_warning(
                                char_idx,
                                &format!(
                                    "Unicode character U+{dcid:04X} cannot be represented in dceutils format ({target_format}) and was omitted."
                                ),
                            );
                        }
                    } else {
                        flush_unmapped(&mut unmapped_utf8_buf, &mut short_dcs);
                        log.export_warning(
                            char_idx,
                            &format!(
                                "Long Dc {dcid} cannot be represented in dceutils format ({target_format}); dceutils formats do not support Dc 308."
                            ),
                        );
                    }
                    char_idx = char_idx.saturating_add(1);
                }
            }
        }
    }

    flush_unmapped(&mut unmapped_utf8_buf, &mut short_dcs);
    Ok(short_dcs)
}

/// Converts a `DcMstring` to a `dceutils` format byte vector.
///
/// # Errors
/// Returns an error if the target format is unknown or serialization fails.
pub fn dcmstring_to_dceutils(
    doc: &DcMstring,
    format_code: &str,
) -> Result<(Vec<u8>, FormatLog)> {
    dcmstr_to_dceutils_with_options(doc, format_code, false)
}

/// Encodes a document using shared Unicode conversion and UTF-8 recovery.
///
/// # Errors
/// Returns an error for unsupported output formats or invalid text.
pub fn dcmstr_to_dceutils_with_options(
    doc: &ctb_formats_dcstring::DcMstr,
    format_code: &str,
    permissive: bool,
) -> Result<(Vec<u8>, FormatLog)> {
    let mut log = FormatLog::default();
    if matches!(format_code, "utf8" | "utf32" | "utf8_base64" | "utf8_dc64" | "utf8_dc64_enc" | "utf8_dc64_bin" | "utf8_dc64_bin_hex" | "utf8_dc64_bin_enc" | "utf8_dc64_bin_enc_hex") {
        let (utf8, text_log) = ctb_formats_dcstring::converters::utf8::dcmstr_to_utf8_with_options(
            doc,
            &ctb_formats_dcstring::converters::utf8::DcListUtf8Settings {
                canonicalize_equivalent_dcs: true,
                permissive,
                ..Default::default()
            },
        )?;
        log.merge(&text_log);
        let bytes = match format_code {
            "utf8" => utf8,
            "utf32" => ctb_formats_encoding::unicode::utf8_to_utf32be(&utf8)?,
            "utf8_base64" => ctb_formats_base64::base64_encode(&utf8).into_bytes(),
            "utf8_dc64" | "utf8_dc64_enc" => {
                let digits = if format_code == "utf8_dc64_enc" {
                    crate::converter::encode_utf8_to_dc64_encapsulated(&utf8)
                } else {
                    crate::converter::encode_bytes_to_dc64(&utf8)
                };
                digits.iter().map(u32::to_string).collect::<Vec<_>>().join(",").into_bytes()
            }
            _ => crate::converter::encode_dc64_to_core_bytes(
                &crate::converter::encode_bytes_to_dc64(&utf8),
                format_code.contains("_enc"),
                format_code.ends_with("_hex"),
            )?,
        };
        return Ok((bytes, log));
    }
    if matches!(format_code, "cdce" | "legacy_cdce" | "cdce_legacy" | "html_l" | "html_legacy" | "html_legacy_cdce" | "html_snippet_l" | "html_s_legacy" | "html_legacy_cdce_snippet") {
        let bytes = crate::converter::encode_cdce(doc, format_code != "cdce", permissive, &mut log)?;
        let bytes = match format_code {
            "html_l" | "html_legacy" | "html_legacy_cdce" => crate::legacy::onestep_legacy_cdce_to_html_l(&bytes)?,
            "html_snippet_l" | "html_s_legacy" | "html_legacy_cdce_snippet" => crate::legacy::onestep_legacy_cdce_to_html_snippet_l(&bytes)?,
            _ => bytes,
        };
        return Ok((bytes, log));
    }
    let short_dcs =
        dcmstring_to_dceutils_short_dcs(doc, format_code, &mut log)?;

    match format_code {
        "dce" | "hex_dce" | "dce_hex" | "3_0a" | "dce_3_0a" | "3_0a_raw"
        | "dce_3_0a_raw" | "hex_3_0a" | "3_0a_hex" | "dce_3_0a_hex"
        | "hex_3_0a_raw" | "3_0a_raw_hex" | "dce_3_0a_raw_hex" | "3_01a"
        | "dce_3_01a" | "3_01a_raw" | "dce_3_01a_raw" | "hex_3_01a"
        | "3_01a_hex" | "dce_3_01a_hex" | "hex_3_01a_raw" | "3_01a_raw_hex"
        | "dce_3_01a_raw_hex" | "3_02a" | "dce_3_02a" | "3_02a_raw"
        | "dce_3_02a_raw" | "hex_3_02a" | "3_02a_hex" | "dce_3_02a_hex"
        | "hex_3_02a_raw" | "3_02a_raw_hex" | "dce_3_02a_raw_hex" => {
            let bytes = crate::converter::encode_dce_preserving_controls(
                &short_dcs,
                format_code,
                &mut log,
            )?;
            Ok((bytes, log))
        }
        "dc" | "dc_id_list" => {
            let short_dcs = crate::converter::content_short_dcs(&short_dcs)?;
            let csv = short_dcs
                .iter()
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join(",");
            let csv = crate::convert_dc_to_dc_output(&csv, &mut log);
            Ok((csv.into_bytes(), log))
        }
        _ => bail!("Unknown output format."),
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "Standard repository test boilerplate"
)]
mod tests {
    use super::*;

    #[crate::ctb_test]
    fn test_dceutils_modern_unicode_export() {
        let text = "a-b\t\n\r\u{8}\u{b}\u{c}\u{85}\u{fe00}\u{1f304}@123@";
        let doc = DcMstring::from(text);
        for format in ["dc", "cdce", "3_01a", "3_01a_raw"] {
            let (encoded, log) = dcmstring_to_dceutils(&doc, format).unwrap();
            assert!(
                log.has_no_errors() && !log.has_warnings(),
                "{format}: {log:?}"
            );
            let decoded =
                crate::dce_convert_new(&encoded, format, "utf8").unwrap();
            assert_eq!(decoded, text.as_bytes(), "{format}");
        }
        for stubbed in ["3_02a", "3_02a_raw", "hex_3_02a", "hex_3_02a_raw"] {
            assert!(dcmstring_to_dceutils(&doc, stubbed).is_err());
            assert!(
                dceutils_to_dcmstring_with_options(b"\x80A\x81", stubbed, true)
                    .is_err()
            );
        }
    }

    #[crate::ctb_test]
    fn test_dceutils_modern_wire_roundtrip() {
        for format in ["3_0a", "3_01a", "3_01a_raw"] {
            let text = if format == "3_0a" {
                "Hello!"
            } else {
                "\t\n\r\u{8}\u{b}\u{c}\u{fe00}\u{1f304}"
            };
            let encoded =
                crate::dce_convert_new(text.as_bytes(), "utf8", format)
                    .unwrap();
            let (doc, log) =
                dceutils_to_dcmstring_with_options(&encoded, format, false)
                    .unwrap();
            assert!(log.has_no_errors() && !log.has_warnings());
            let (reencoded, log) = dcmstring_to_dceutils(&doc, format).unwrap();
            assert!(log.has_no_errors() && !log.has_warnings());
            assert_eq!(reencoded, encoded, "{format}");
        }
    }

    #[crate::ctb_test]
    fn test_dceutils_permissive_text_import() {
        for (format, bytes) in [
            ("3_01a_raw", &b"\x80A\xc3\x9a\xa1\xbf\xa4\xc4B\x81"[..]),
            ("dc", &b"50,191,151,158,188,161,192,51"[..]),
            ("utf8_dc64", &b"151,158,188,161"[..]),
        ] {
            assert!(
                dceutils_to_dcmstring_with_options(bytes, format, false)
                    .is_err()
            );
            let (doc, log) =
                dceutils_to_dcmstring_with_options(bytes, format, true)
                    .unwrap();
            assert!(log.has_warnings() && !log.has_errors());
            let (text, _) = dcmstring_to_dceutils(&doc, "utf8_dc64").unwrap();
            let decoded =
                crate::dce_convert_new(&text, "utf8_dc64", "utf8").unwrap();
            let expected = if format == "utf8_dc64" {
                "a\u{FFFD}b"
            } else {
                "Aa\u{FFFD}bB"
            };
            assert_eq!(decoded, expected.as_bytes(), "{format}");
        }
    }
}
