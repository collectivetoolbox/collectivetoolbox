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

//! Implementation of Format 461 (Dcts: DcText short).
//!
//! Like DcText, but Document Characters (Dcs) are short by default:
//! - `@123@` = short Dc 123 (`SHORT_DC_REGION_START + 123`)
//! - `a` = literal Unicode `a`
//! - `@@` = codepoint 64 (`@`)
//! - `@l123@` = long Dc 123
//! - `@L123@` = local node reference (short Dc 296 followed by Dc number 123)
//! - `@u123a@` = Unicode codepoint U+123A (hex)
//! - `@f123@` = Format 123 (`FORMAT_REGION_START + 123`)

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::Result;
use ctb_formats_utf_8e_128::decode_utf_8e_128;
use ctb_formats_utilities::{ConversionOutput, FormatLog};
use ctb_storage_minimal::global_graph_layout::{
    FORMAT_REGION_END, FORMAT_REGION_START, SHORT_DC_REGION_END,
    SHORT_DC_REGION_START, UNICODE_REGION_END, dc_to_gid, format_to_gid,
};

use super::dctext::{
    DcList, dcarray_to_dclist, dclist_to_dcarray, dcutf_to_dclist,
};
use crate::dc_char::DcChar;
use crate::dc_number::{integer_to_dc_number_global, read_dc_number_global};
use crate::dc_str::DcStr;
use crate::DcString;

/// Parses a Dcts document (`&[u8]`) into a `DcList` (`Vec<u128>`).
///
/// Plain text characters become their corresponding Unicode codepoint IDs (`0..=0x10FFFF`).
/// Tokens enclosed in `@...@` are parsed according to their prefix:
/// - `@@`: codepoint 64 (`@`)
/// - `@<number>@`: short Dc ID (e.g. `@123@` -> `SHORT_DC_REGION_START + 123`)
/// - `@l<number>@`: long Dc ID (e.g. `@l123@` -> `123`)
/// - `@L<number>@`: local node reference (short Dc 296 + Dc number `<number>`)
/// - `@u<hex>@`: Unicode codepoint (e.g. `@u123a@` -> `U+123A`)
/// - `@f<number>@`: Format ID (e.g. `@f123@` -> `FORMAT_REGION_START + 123`)
///
/// # Errors
/// Returns an error if the document cannot be parsed into a `DcList`.
pub fn dcts_to_dclist(document: &[u8]) -> Result<ConversionOutput<DcList>> {
    let mut log = FormatLog::default();
    let mut list = Vec::new();
    let mut i = 0usize;

    while i < document.len() {
        let Some(slice) = document.get(i..) else {
            break;
        };
        let Some(&first_byte) = slice.first() else {
            break;
        };

        if first_byte == b'@' {
            if let Some(rest) = slice.get(1..) {
                if let Some(end_rel) = rest.iter().position(|&b| b == b'@') {
                    if let Some(token_bytes) = rest.get(..end_rel) {
                        if let Ok(token_str) = std::str::from_utf8(token_bytes) {
                            if token_str.is_empty() {
                                // @@ token represents @ (codepoint 64)
                                list.push(64u128);
                                i = i.saturating_add(2);
                                continue;
                            }

                            // 1) Local node: @L<number>@
                            if let Some(num_str) = token_str.strip_prefix('L') {
                                if let Ok(int_val) =
                                    num_str.parse::<malachite::Integer>()
                                {
                                    match integer_to_dc_number_global(&int_val) {
                                        Ok(dc_num_gids) => {
                                            list.push(1_114_408u128);
                                            list.extend(dc_num_gids);
                                            i = i
                                                .saturating_add(2)
                                                .saturating_add(end_rel);
                                            continue;
                                        }
                                        Err(e) => {
                                            log.warn(&format!(
                                                "Failed to encode Dc number for local node @{token_str}@: {e}"
                                            ));
                                        }
                                    }
                                } else {
                                    log.warn(&format!(
                                        "Invalid local reference token @{token_str}@ in Dcts"
                                    ));
                                }
                            }
                            // 2) Long Dc: @l<number>@
                            else if let Some(num_str) = token_str.strip_prefix('l') {
                                if let Ok(dcid) = num_str.parse::<u128>() {
                                    list.push(dcid);
                                    i = i
                                        .saturating_add(2)
                                        .saturating_add(end_rel);
                                    continue;
                                } else {
                                    log.warn(&format!(
                                        "Invalid long Dc token @{token_str}@ in Dcts"
                                    ));
                                }
                            }
                            // 3) Unicode codepoint hex: @u123a@ or @U123A@
                            else if let Some(hex_raw) = token_str
                                .strip_prefix('u')
                                .or_else(|| token_str.strip_prefix('U'))
                            {
                                let hex_str =
                                    hex_raw.strip_prefix('+').unwrap_or(hex_raw);
                                if let Ok(cp) = u32::from_str_radix(hex_str, 16) {
                                    if u128::from(cp) <= UNICODE_REGION_END {
                                        list.push(u128::from(cp));
                                        i = i
                                            .saturating_add(2)
                                            .saturating_add(end_rel);
                                        continue;
                                    }
                                    log.warn(&format!(
                                        "Unicode codepoint @{token_str}@ exceeds 0x10FFFF maximum"
                                    ));
                                } else {
                                    log.warn(&format!(
                                        "Invalid hex Unicode codepoint token @{token_str}@ in Dcts"
                                    ));
                                }
                            }
                            // 4) Format: @f123@ or @F123@
                            else if let Some(fmt_str) = token_str
                                .strip_prefix('f')
                                .or_else(|| token_str.strip_prefix('F'))
                            {
                                if let Ok(fmt_id) = fmt_str.parse::<u64>() {
                                    let gid = format_to_gid(fmt_id);
                                    if gid <= FORMAT_REGION_END {
                                        list.push(gid);
                                        i = i
                                            .saturating_add(2)
                                            .saturating_add(end_rel);
                                        continue;
                                    }
                                    log.warn(&format!(
                                        "Format ID @{token_str}@ exceeds format region bounds ({FORMAT_REGION_START}..={FORMAT_REGION_END})"
                                    ));
                                } else {
                                    log.warn(&format!(
                                        "Invalid format token @{token_str}@ in Dcts"
                                    ));
                                }
                            }
                            // 5) Short Dc by default: @123@
                            else if let Ok(short_id) = token_str.parse::<u64>() {
                                let gid = dc_to_gid(short_id);
                                if gid > SHORT_DC_REGION_END {
                                    log.warn(&format!(
                                        "Short Dc ID @{token_str}@ exceeds short Dc region bounds ({SHORT_DC_REGION_START}..={SHORT_DC_REGION_END})"
                                    ));
                                }
                                list.push(gid);
                                i = i
                                    .saturating_add(2)
                                    .saturating_add(end_rel);
                                continue;
                            } else {
                                log.warn(&format!(
                                    "Unrecognized token @{token_str}@ in Dcts"
                                ));
                            }
                        }
                    }
                }
            }
        }

        if let Some((codepoint, size)) = decode_utf_8e_128(slice) {
            list.push(codepoint);
            i = i.saturating_add(size);
        } else {
            list.push(u128::from(first_byte));
            i = i.saturating_add(1);
        }
    }

    Ok(ConversionOutput::new(list, log))
}

/// Serializes a `DcList` (`&[u128]`) to Dcts format bytes (`Vec<u8>`).
#[must_use]
pub fn dclist_to_dcts(dclist: &[u128]) -> Vec<u8> {
    let mut output = String::new();
    let mut i = 0usize;

    while i < dclist.len() {
        let Some(&dcid) = dclist.get(i) else { break };

        if dcid == 64 {
            output.push_str("@@");
        } else if dcid <= UNICODE_REGION_END {
            if let Ok(cp) = u32::try_from(dcid) {
                if let Some(ch) = char::from_u32(cp) {
                    output.push(ch);
                } else {
                    output.push_str(&format!("@u{cp:x}@"));
                }
            } else {
                output.push_str(&format!("@u{dcid:x}@"));
            }
        } else if dcid == 1_114_408 {
            if let Some(rest) = dclist.get(i.saturating_add(1)..) {
                if let Ok((int_val, consumed)) = read_dc_number_global(rest) {
                    output.push_str(&format!("@L{int_val}@"));
                    i = i.saturating_add(1).saturating_add(consumed);
                    continue;
                }
            }
            output.push_str("@296@");
        } else if (SHORT_DC_REGION_START..=SHORT_DC_REGION_END).contains(&dcid) {
            let short_id = dcid.saturating_sub(SHORT_DC_REGION_START);
            output.push_str(&format!("@{short_id}@"));
        } else if (FORMAT_REGION_START..=FORMAT_REGION_END).contains(&dcid) {
            let fmt_id = dcid.saturating_sub(FORMAT_REGION_START);
            output.push_str(&format!("@f{fmt_id}@"));
        } else {
            output.push_str(&format!("@l{dcid}@"));
        }
        i = i.saturating_add(1);
    }

    output.into_bytes()
}

/// Parses a Dcts document (`&[u8]`) directly into a `DcString`.
///
/// # Errors
/// Returns an error if the document cannot be parsed into a `DcString`.
pub fn dcts_to_dcstring(document: &[u8]) -> Result<ConversionOutput<DcString>> {
    let out = dcts_to_dclist(document)?;
    let mut dc_string = DcString::with_capacity(document.len());
    for &dc in &out.result {
        dc_string.push(DcChar(dc));
    }
    Ok(ConversionOutput::new(dc_string, out.log))
}

/// Serializes a `DcStr` to Dcts format bytes (`Vec<u8>`).
#[must_use]
pub fn dcstring_to_dcts(s: &DcStr) -> Vec<u8> {
    let dclist = s.to_dclist();
    dclist_to_dcts(&dclist)
}

/// Converts Dcts format bytes to DcUtf format bytes.
#[must_use]
pub fn dcts_to_dcutf(document: Vec<u8>) -> Vec<u8> {
    if let Ok(out) = dcts_to_dcstring(&document) {
        out.result.into_bytes()
    } else {
        Vec::new()
    }
}

/// Converts DcUtf format bytes to Dcts format bytes.
#[must_use]
pub fn dcutf_to_dcts(document: Vec<u8>) -> Vec<u8> {
    if let Ok(dc_str) = DcStr::from_bytes(&document) {
        dcstring_to_dcts(dc_str)
    } else {
        let dclist = dcutf_to_dclist(&document);
        dclist_to_dcts(&dclist)
    }
}

/// Converts an EITE DcArray (short Dcs, `&[u32]`) to Dcts format (`Vec<u8>`).
///
/// # Errors
/// Returns an error if the array cannot be converted.
pub fn dcarray_to_dcts(
    dc_array: &[u32],
) -> Result<ConversionOutput<Vec<u8>>> {
    let conv = dcarray_to_dclist(dc_array)?;
    let text_bytes = dclist_to_dcts(&conv.result);
    Ok(ConversionOutput::new(text_bytes, conv.log))
}

/// Converts a Dcts document (`&[u8]`) to an EITE DcArray (short Dcs, `Vec<u32>`).
///
/// # Errors
/// Returns an error if the document cannot be converted.
pub fn dcts_to_dcarray(
    document: &[u8],
) -> Result<ConversionOutput<Vec<u32>>> {
    let conv = dcts_to_dclist(document)?;
    let array_conv = dclist_to_dcarray(&conv.result)?;
    let mut total_log = conv.log;
    total_log.merge(&array_conv.log);
    Ok(ConversionOutput::new(array_conv.result, total_log))
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
    fn test_dcts_tokens() {
        // @123@ = short Dc 123 (1114112 + 123 = 1114235)
        // a = literal Unicode a (97)
        // @@ = literal @ (64)
        // @l123@ = long Dc 123 (123)
        // @L42@ = local node 42
        // @u123a@ = U+123A (4666)
        // @f123@ = format 123 (2228224 + 123 = 2228347)
        let input = b"a @123@ @@ @l123@ @L42@ @u123a@ @f123@";
        let conv = dcts_to_dclist(input).expect("parse dcts");
        assert!(!conv.log.has_warnings());

        // Verify the decoded IDs
        assert_eq!(conv.result.first().copied(), Some(97)); // 'a'
        assert_eq!(conv.result.get(1).copied(), Some(32)); // ' '
        assert_eq!(conv.result.get(2).copied(), Some(1_114_235)); // short Dc 123
        assert_eq!(conv.result.get(3).copied(), Some(32)); // ' '
        assert_eq!(conv.result.get(4).copied(), Some(64)); // '@@'
        assert_eq!(conv.result.get(5).copied(), Some(32)); // ' '
        assert_eq!(conv.result.get(6).copied(), Some(123)); // long Dc 123
        assert_eq!(conv.result.get(7).copied(), Some(32)); // ' '
        assert_eq!(conv.result.get(8).copied(), Some(1_114_408)); // local node prefix
        // local node Dc number follows...
        assert_eq!(conv.result.get(conv.result.len().saturating_sub(3)).copied(), Some(4666)); // U+123A
        assert_eq!(conv.result.get(conv.result.len().saturating_sub(2)).copied(), Some(32)); // ' '
        assert_eq!(conv.result.last().copied(), Some(2_228_347)); // format 123
    }

    #[crate::ctb_test]
    fn test_dcts_serialization_roundtrip() {
        let input = "a @123@ @@ @l100000000000@ @L42@ \u{123a} @f123@";
        let conv = dcts_to_dclist(input.as_bytes()).expect("parse dcts");
        assert!(!conv.log.has_warnings());

        let serialized = dclist_to_dcts(&conv.result);
        let s = String::from_utf8(serialized).expect("valid utf-8");
        assert_eq!(s, input);
    }

    #[crate::ctb_test]
    fn test_dcts_u_token_serializes_to_char_or_surrogate() {
        // @u61@ parses to 97 ('a'), which serializes as literal 'a'
        let conv = dcts_to_dclist(b"@u61@").expect("parse");
        assert_eq!(conv.result, vec![97]);
        let out = dclist_to_dcts(&conv.result);
        assert_eq!(String::from_utf8(out).unwrap(), "a");

        // Surrogate codepoints cannot be represented as char, so serialize as @u...
        let conv_surr = dcts_to_dclist(b"@ud800@").expect("parse surrogate");
        assert_eq!(conv_surr.result, vec![0xd800]);
        let out_surr = dclist_to_dcts(&conv_surr.result);
        assert_eq!(String::from_utf8(out_surr).unwrap(), "@ud800@");
    }

    #[crate::ctb_test]
    fn test_dcts_dcarray_conversions() {
        let original_dcarray = vec![0, 1, 18, 50, 200, 297];
        let conv = dcarray_to_dcts(&original_dcarray).expect("to dcts");
        assert!(!conv.log.has_warnings());
        assert_eq!(
            String::from_utf8(conv.result.clone()).unwrap(),
            "@0@@1@@18@@50@@200@@297@"
        );

        let back = dcts_to_dcarray(&conv.result).expect("back to dcarray");
        assert!(!back.log.has_warnings());
        assert_eq!(back.result, original_dcarray);
    }

    #[crate::ctb_test]
    fn test_dcts_dcstring_and_dcutf_roundtrip() {
        let input = "hello @123@ @f461@ @L7@ world";
        let out = dcts_to_dcstring(input.as_bytes()).expect("dcstring");
        assert!(!out.log.has_warnings());

        let roundtrip_bytes = dcstring_to_dcts(&out.result);
        assert_eq!(String::from_utf8(roundtrip_bytes).unwrap(), input);

        let dcutf = dcts_to_dcutf(input.as_bytes().to_vec());
        let roundtrip_from_dcutf = dcutf_to_dcts(dcutf);
        assert_eq!(String::from_utf8(roundtrip_from_dcutf).unwrap(), input);
    }

    #[crate::ctb_test]
    fn test_dcts_warnings_on_invalid_tokens() {
        // Invalid hex in @u...
        let conv = dcts_to_dclist(b"@uzzz@").expect("parse");
        assert!(conv.log.has_warnings());

        // Unicode codepoint > 0x10FFFF
        let conv2 = dcts_to_dclist(b"@u110000@").expect("parse");
        assert!(conv2.log.has_warnings());

        // Invalid format number
        let conv3 = dcts_to_dclist(b"@fxyz@").expect("parse");
        assert!(conv3.log.has_warnings());

        // Invalid long Dc
        let conv4 = dcts_to_dclist(b"@lxyz@").expect("parse");
        assert!(conv4.log.has_warnings());
    }
}
