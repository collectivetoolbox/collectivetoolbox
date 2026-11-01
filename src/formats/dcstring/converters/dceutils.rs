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
//! - **DCE 3.0a (`3_0a`, `3_0a_raw`, `hex_3_0a`, `hex_3_0a_raw`)**: short Dcs `0..=115`.
//! - **DCE 3.01a (`3_01a`, `3_01a_raw`, `hex_3_01a`, `hex_3_01a_raw`)**: short Dcs `0..=199`,
//!   unlocking Base64 encapsulation (`Dc 191` .. `Dc 127..=190` / `Dc 195` .. `Dc 192`) for data and UTF-8.
//! - **DCE 3.02a (`3_02a`, `3_02a_raw`, `hex_3_02a`, `hex_3_02a_raw`) and `dce` / `hex_dce`**:
//!   short Dcs `0..=202` (plus version selector `Dc 205`) and Base64 encapsulation.
//!
//! Unlike EITE formats, `dceutils` formats do not support `Dc 308` (`Next number is a long (global graph) Dc`).
//! Unrepresentable characters are logged as warnings in [`FormatLog`], while DCE stateful-encoding control
//! characters (`13..=16`, `112..=115`, `194`, `196..=199`, `205`) are preserved as-is when converting to Dc documents.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::{Context, Result, bail};
use ctb_formats_utilities::FormatLog;
use ctb_storage_minimal::global_graph_layout::{
    SHORT_DC_REGION_END, SHORT_DC_REGION_START,
};

use super::dctext::dclist_to_dctf;
use crate::{DcChar, DcMixedChunk, DcMstring, DcStr};

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
                    &format!("Invalid short Dc ID token '{piece}' in dceutils stream"),
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
        bail!(
            "Input does not conform to valid {format_name} format: {errs}"
        );
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
    let short_dcs = match format_code {
        "cdce" | "legacy_cdce" | "cdce_legacy" => {
            let (dc_bytes, sub_log) =
                ctb_formats_dceutils::converter::dce_convert_with_log(
                    bytes,
                    format_code,
                    "dc",
                )?;
            raw_log.merge(&sub_log);
            let dc_csv = String::from_utf8(dc_bytes)
                .context("Invalid UTF-8 from dceutils dc conversion")?;
            let parsed = parse_comma_short_dcs(&dc_csv, &mut raw_log)?;
            strip_outer_unbounded_markers(&parsed)
        }
        "cdce_lstrict" | "cdce_legacy_strict" => {
            let dc_csv = ctb_formats_dceutils::cdce::convert_legacy_cdce_to_dc(
                bytes,
                !permissive,
                &mut raw_log,
            )?;
            if !raw_log.has_errors() {
                parse_comma_short_dcs(&dc_csv, &mut raw_log)?
            } else {
                Vec::new()
            }
        }
        "dce" | "hex_dce" | "dce_hex" | "3_0a" | "dce_3_0a" | "3_0a_raw"
        | "dce_3_0a_raw" | "hex_3_0a" | "3_0a_hex" | "dce_3_0a_hex"
        | "hex_3_0a_raw" | "3_0a_raw_hex" | "dce_3_0a_raw_hex" | "3_01a"
        | "dce_3_01a" | "3_01a_raw" | "dce_3_01a_raw" | "hex_3_01a"
        | "3_01a_hex" | "dce_3_01a_hex" | "hex_3_01a_raw"
        | "3_01a_raw_hex" | "dce_3_01a_raw_hex" | "3_02a" | "dce_3_02a"
        | "3_02a_raw" | "dce_3_02a_raw" | "hex_3_02a" | "3_02a_hex"
        | "dce_3_02a_hex" | "hex_3_02a_raw" | "3_02a_raw_hex"
        | "dce_3_02a_raw_hex" => {
            ctb_formats_dceutils::converter::decode_dce_preserving_controls(
                bytes,
                format_code,
                &mut raw_log,
            )?
        }
        "dc" | "dc_id_list" => {
            let s = std::str::from_utf8(bytes)
                .context("Invalid UTF-8 in DcIdList document")?;
            parse_comma_short_dcs(s, &mut raw_log)?
        }
        "utf8_dc64"
        | "utf8_dc64_enc"
        | "utf8_dc64_bin"
        | "utf8_dc64_bin_hex"
        | "utf8_dc64_bin_enc"
        | "utf8_dc64_bin_enc_hex" => {
            let (dc_bytes, sub_log) =
                ctb_formats_dceutils::converter::dce_convert_with_log(
                    bytes,
                    format_code,
                    "dc",
                )?;
            raw_log.merge(&sub_log);
            let dc_csv = String::from_utf8(dc_bytes)
                .context("Invalid UTF-8 from dceutils dc conversion")?;
            parse_comma_short_dcs(&dc_csv, &mut raw_log)?
        }
        _ => bail!("Unsupported dceutils input format: {format_code}"),
    };

    let log = finalize_import_log(raw_log, permissive, format_code)?;
    let doc = short_dcs_to_dcmstring(&short_dcs)?;
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
fn dcmstring_to_dceutils_short_dcs(
    doc: &DcMstring,
    target_format: &str,
    log: &mut FormatLog,
) -> Result<Vec<u32>> {
    crate::validate_dcmst(doc.as_bytes())?;
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
            out.extend(
                ctb_formats_dceutils::converter::encode_utf8_to_dc64_encapsulated(
                    buf.as_bytes(),
                ),
            );
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
                        if let Some(short_id) = ctb_formats_dceutils::converter::unicode_codepoint_to_short_dc_lossy(cp) {
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
    let mut log = FormatLog::default();
    let short_dcs =
        dcmstring_to_dceutils_short_dcs(doc, format_code, &mut log)?;

    match format_code {
        "dce" | "hex_dce" | "dce_hex" | "3_0a" | "dce_3_0a" | "3_0a_raw"
        | "dce_3_0a_raw" | "hex_3_0a" | "3_0a_hex" | "dce_3_0a_hex"
        | "hex_3_0a_raw" | "3_0a_raw_hex" | "dce_3_0a_raw_hex" | "3_01a"
        | "dce_3_01a" | "3_01a_raw" | "dce_3_01a_raw" | "hex_3_01a"
        | "3_01a_hex" | "dce_3_01a_hex" | "hex_3_01a_raw"
        | "3_01a_raw_hex" | "dce_3_01a_raw_hex" | "3_02a" | "dce_3_02a"
        | "3_02a_raw" | "dce_3_02a_raw" | "hex_3_02a" | "3_02a_hex"
        | "dce_3_02a_hex" | "hex_3_02a_raw" | "3_02a_raw_hex"
        | "dce_3_02a_raw_hex" => {
            let bytes =
                ctb_formats_dceutils::converter::encode_dce_preserving_controls(
                    &short_dcs,
                    format_code,
                    &mut log,
                )?;
            Ok((bytes, log))
        }
        "cdce"
        | "legacy_cdce"
        | "cdce_legacy"
        | "html_l"
        | "html_legacy"
        | "html_legacy_cdce"
        | "html_snippet_l"
        | "html_s_legacy"
        | "html_legacy_cdce_snippet"
        | "dc"
        | "dc_id_list"
        | "utf8_dc64"
        | "utf8_dc64_enc"
        | "utf8_dc64_bin"
        | "utf8_dc64_bin_hex"
        | "utf8_dc64_bin_enc"
        | "utf8_dc64_bin_enc_hex" => {
            let csv = short_dcs
                .iter()
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join(",");
            let (bytes, sub_log) =
                ctb_formats_dceutils::converter::dce_convert_with_log(
                    csv.as_bytes(),
                    "dc",
                    format_code,
                )?;
            log.merge(&sub_log);
            Ok((bytes, log))
        }
        _ => bail!("Unsupported dceutils output format: {format_code}"),
    }
}
