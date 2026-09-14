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

//! Implements DcText and related formats DcList and DcUTF. These formats are
//! encodings for sequences of integer global graph IDs (which are long Dcs).
//! Format looks like (w/o backticks): `Unicode (UTF-8) text @123@miesu@214748364@@L662@`
//! where `Unicode text` is actual unicode text, and between each pair of @ signs, is a DcId. A DcId can be any int 128 bits (u128) in decimal, and it may have an `L` prefix.
//! Output format is sort of UTF-8 text. For normal Unicode input characters, the output character is the same. For DcIds less than or equal to 1114111 (the largest Unicode character, I believe), the output character is the corresponding "generalized UTF-8", the numeric value encoded in the same underlying algorithm as UTF-8. For DcIds greater than 1114111 and not prefixed with an L, the output character is the decimal DcId represented by extending the usual algorithm of UTF-8 encoding, but for those larger numbers. For DcIds prefixed with an L, the output is equivalent to @1114408@ (short Dc 296) followed by a Dc number for the number that followed the L (the L is just a shorthand for that 1114408 Dc). That is to say, it's not a true Unicode encoding, it's simply using an extension of the algorithm underlying UTF-8 as a convenient encoding of ints.
//! Currently, DcList is used as the internal format for pivoting between other formats, but DcUtf might make more sense eventually for space efficiency.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::Result;
use ctb_formats_utilities::{ConversionOutput, FormatLog};
use ctb_formats_dcdata::dc::{
    GID_ESCAPE, GID_LONG_DC, SHORT_DC_ESCAPE, SHORT_DC_LONG_DC,
    SHORT_DC_REGION_END, SHORT_DC_REGION_START,
};
use crate::dc_number::{
    integer_to_dc_number_global, read_dc_number_global, read_dc_number_short,
    u128_to_dc_number_short,
};
use crate::dc_char::DcChar;
use crate::dc_str::DcStr;
use crate::DcString;
use ctb_formats_utf_8e_128::{decode_utf_8e_128, encode_utf_8e_128_buf};

/// A list of global graph Document Character IDs represented as `u128` values.
pub type DcList = Vec<u128>;

/// Parses a DcText document (`&[u8]`) into a `DcList` (`Vec<u128>`).
///
/// Plain text characters become their corresponding Unicode codepoint IDs (`0..=0x10FFFF`).
/// Tokens in `@<dcid>@` format are parsed into `u128` values.
/// `@L<number>@` tokens expand to `1114408` followed by the Dc number representation of `<number>`.
/// `@@` tokens represent codepoint `64` (`@`).
///
/// # Errors
/// Parses a DcText document (`&[u8]`) directly into a `DcString`.
///
/// Plain text characters become their corresponding Unicode codepoint IDs (`0..=0x10FFFF`).
/// Tokens in `@<dcid>@` format are parsed into `u128` values.
/// `@L<number>@` tokens expand to `1114408` followed by the Dc number representation of `<number>`.
/// `@@` tokens represent codepoint `64` (`@`).
///
/// # Errors
/// Returns an error if the document contains invalid UTF-8 or malformed syntax.
pub fn dctext_to_dcstring(document: &[u8]) -> Result<ConversionOutput<DcString>> {
    let mut log = FormatLog::default();
    let mut dc_string = DcString::with_capacity(document.len());
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
                        if let Ok(token_str) = std::str::from_utf8(token_bytes)
                        {
                            let mut dcid_str = token_str;
                            let mut is_l = false;

                            if dcid_str.is_empty() {
                                dcid_str = "64"; // @@ token represents @ (codepoint 64)
                            }
                            if let Some(stripped) = dcid_str.strip_prefix('L') {
                                is_l = true;
                                dcid_str = stripped;
                            }

                            if is_l {
                                if let Ok(int_val) =
                                    dcid_str.parse::<malachite::Integer>()
                                {
                                    match integer_to_dc_number_global(&int_val)
                                    {
                                        Ok(dc_num_gids) => {
                                            dc_string.push(DcChar(1_114_408u128));
                                            for gid in dc_num_gids {
                                                dc_string.push(DcChar(gid));
                                            }
                                            i = i
                                                .saturating_add(2)
                                                .saturating_add(end_rel);
                                            continue;
                                        }
                                        Err(e) => {
                                            log.warn(&format!(
                                                "Failed to encode Dc number for @{token_str}@: {e}"
                                            ));
                                        }
                                    }
                                } else {
                                    log.warn(&format!(
                                        "Invalid local reference token @{token_str}@ in DcText"
                                    ));
                                }
                            } else if let Ok(dcid) = dcid_str.parse::<u128>() {
                                dc_string.push(DcChar(dcid));
                                i = i.saturating_add(2).saturating_add(end_rel);
                                continue;
                            }
                        }
                    }
                }
            }
        }

        if let Some((codepoint, size)) = decode_utf_8e_128(slice) {
            if let Some(raw_bytes) = slice.get(..size) {
                dc_string.inner.extend_from_slice(raw_bytes);
            } else {
                dc_string.push(DcChar(codepoint));
            }
            i = i.saturating_add(size);
        } else {
            dc_string.push(DcChar(u128::from(first_byte)));
            i = i.saturating_add(1);
        }
    }

    Ok(ConversionOutput::new(dc_string, log))
}

/// Parses a DcText document (`&[u8]`) into a `DcList` (`Vec<u128>`).
///
/// # Errors
/// Returns an error if the document contains invalid UTF-8 or malformed syntax.
pub fn dctext_to_dclist(document: &[u8]) -> Result<ConversionOutput<DcList>> {
    let out = dctext_to_dcstring(document)?;
    Ok(ConversionOutput::new(out.result.to_dclist(), out.log))
}

/// Serializes a `DcList` (`&[u128]`) to DcText format bytes (`Vec<u8>`).
pub fn dclist_to_dctext(dclist: &[u128]) -> Vec<u8> {
    let mut output = String::new();
    let mut i = 0;
    while i < dclist.len() {
        let Some(&dcid) = dclist.get(i) else { break };

        if dcid == 64 {
            output.push_str("@@");
        } else if dcid <= 0x10_FFFF {
            if let Ok(cp) = u32::try_from(dcid) {
                if let Some(ch) = char::from_u32(cp) {
                    output.push(ch);
                } else {
                    output.push_str(&format!("@{dcid}@"));
                }
            } else {
                output.push_str(&format!("@{dcid}@"));
            }
        } else if dcid == 1_114_408 {
            if let Some(rest) = dclist.get(i.saturating_add(1)..) {
                if let Ok((int_val, consumed)) = read_dc_number_global(rest) {
                    output.push_str(&format!("@L{int_val}@"));
                    i = i.saturating_add(1).saturating_add(consumed);
                    continue;
                }
            }
            output.push_str("@1114408@");
        } else {
            output.push_str(&format!("@{dcid}@"));
        }
        i = i.saturating_add(1);
    }
    output.into_bytes()
}

/// Converts a `DcList` (`&[u128]`) to generalized UTF-8 bytes (`DcUtf`).
pub fn dclist_to_dcutf(dclist: &[u128]) -> Vec<u8> {
    let mut output = Vec::new();
    let mut buf = [0u8; 24];
    for &dcid in dclist {
        let n = encode_utf_8e_128_buf(&mut buf, dcid);
        if let Some(slice) = buf.get(..n) {
            output.extend_from_slice(slice);
        }
    }
    output
}

/// Decodes generalized UTF-8 bytes (`DcUtf`) into a `DcList` (`Vec<u128>`).
pub fn dcutf_to_dclist(document: &[u8]) -> DcList {
    let mut list = Vec::new();
    let mut i = 0;
    while i < document.len() {
        let Some(slice) = document.get(i..) else {
            break;
        };
        if let Some((codepoint, size)) = decode_utf_8e_128(slice) {
            list.push(codepoint);
            i = i.saturating_add(size);
        } else {
            if let Some(&b) = document.get(i) {
                list.push(u128::from(b));
            }
            i = i.saturating_add(1);
        }
    }
    list
}

/// Parses a DcText document (`&[u8]`) directly into a `DcString`.
///
/// Converts an EITE DcArray (short Dcs, `&[u32]`) to a `DcString`.
pub fn dcarray_to_dcstring(
    dc_array: &[u32],
) -> Result<ConversionOutput<DcString>> {
    let conv = dcarray_to_dclist(dc_array)?;
    let mut dc_string = DcString::with_capacity(conv.result.len());
    for &dc in &conv.result {
        dc_string.push(DcChar(dc));
    }
    Ok(ConversionOutput::new(dc_string, conv.log))
}

/// Converts a `DcStr` to an EITE DcArray (short Dcs, `Vec<u32>`).
pub fn dcstring_to_dcarray(
    s: &DcStr,
) -> Result<ConversionOutput<Vec<u32>>> {
    dclist_to_dcarray(&s.to_dclist())
}

/// Serializes a `DcStr` to DcText format bytes (`Vec<u8>`).
#[must_use]
pub fn dcstring_to_dctext(s: &DcStr) -> Vec<u8> {
    let dclist = s.to_dclist();
    dclist_to_dctext(&dclist)
}

/// Converts DcText format bytes to DcUtf format bytes.
///
/// # Errors
/// Returns an error if the document cannot be parsed into a `DcString`.
pub fn dctext_to_dcutf(document: Vec<u8>) -> Result<Vec<u8>> {
    let out = dctext_to_dcstring(&document)?;
    Ok(out.result.into_bytes())
}

/// Converts DcUtf format bytes to DcText format bytes.
#[must_use]
pub fn dcutf_to_dctext(document: Vec<u8>) -> Vec<u8> {
    let dclist = dcutf_to_dclist(&document);
    dclist_to_dctext(&dclist)
}

/// Converts an EITE DcArray (short Dcs, `&[u32]`) to a `DcList` (`Vec<u128>`).
///
/// Handles:
/// - Escaped Dc 308 (`[255, 308]` -> long Dc `1_114_420`).
/// - Escaped Dc 255 (`[255, 255]` -> long Dc `1_114_367`).
/// - Standalone Dc 255 (`[255]` -> long Dc `1_114_367`).
/// - Embedded long Dc IDs (`[308, ...DcNumber...]` -> decoded `u128` long Dc ID).
/// - Direct short Dcs `c` -> mapped to `SHORT_DC_OFFSET + c` (`1_114_112 + c`).
pub fn dcarray_to_dclist(dc_array: &[u32]) -> Result<ConversionOutput<DcList>> {
    let mut log = FormatLog::default();
    let mut list = Vec::with_capacity(dc_array.len());
    let mut i = 0usize;

    while i < dc_array.len() {
        let Some(&dc) = dc_array.get(i) else {
            break;
        };

        if dc == SHORT_DC_ESCAPE {
            if let Some(&next_dc) = dc_array.get(i.saturating_add(1)) {
                if next_dc == SHORT_DC_LONG_DC {
                    list.push(GID_LONG_DC);
                    i = i.saturating_add(2);
                    continue;
                }
                if next_dc == SHORT_DC_ESCAPE {
                    list.push(GID_ESCAPE);
                    i = i.saturating_add(2);
                    continue;
                }
            }
            list.push(GID_ESCAPE);
            i = i.saturating_add(1);
            continue;
        }

        if dc == SHORT_DC_LONG_DC {
            if let Some(rest) = dc_array.get(i.saturating_add(1)..) {
                match read_dc_number_short(rest) {
                    Ok((int_val, consumed)) => {
                        if int_val >= 0 {
                            if let Ok(abs_u128) =
                                u128::try_from(int_val.unsigned_abs_ref())
                            {
                                list.push(abs_u128);
                                i = i.saturating_add(1).saturating_add(consumed);
                                continue;
                            }
                        }
                        log.warn(&format!(
                            "Embedded long Dc ID {int_val} at index {i} cannot be represented as u128"
                        ));
                    }
                    Err(e) => {
                        log.warn(&format!(
                            "Unescaped Dc 308 at index {i} not followed by valid Dc number: {e}"
                        ));
                    }
                }
            } else {
                log.warn(&format!(
                    "Unescaped Dc 308 at end of stream (index {i}) missing Dc number"
                ));
            }
            list.push(GID_LONG_DC);
            i = i.saturating_add(1);
            continue;
        }

        if dc > ctb_formats_eite::encoding::pack32::PACK32_MAX {
            log.warn(&format!(
                "Short Dc ID {dc} at index {i} exceeds pack32 maximum range (1114111)"
            ));
        }

        let long_dc_id = SHORT_DC_REGION_START.saturating_add(u128::from(dc));
        list.push(long_dc_id);
        i = i.saturating_add(1);
    }

    Ok(ConversionOutput::new(list, log))
}

/// Converts an EITE DcArray (short Dcs, `&[u32]`) to the DcText format (`Vec<u8>`).
pub fn dcarray_to_dctext(
    dc_array: &[u32],
) -> Result<ConversionOutput<Vec<u8>>> {
    let conv = dcarray_to_dclist(dc_array)?;
    let text_bytes = dclist_to_dctext(&conv.result);
    Ok(ConversionOutput::new(text_bytes, conv.log))
}

/// Converts a `DcList` (`&[u128]`) to an EITE DcArray (short Dcs, `Vec<u32>`).
///
/// Directly represents Document Characters in range `1_114_112..=2_228_223`
/// ([`SHORT_DC_REGION_START`]..=[`SHORT_DC_REGION_END`], short IDs `0..=1_114_111`),
/// with escaping for Dc 308 (`[255, 308]`) and Dc 255 (`[255, 255]`).
/// All other long Dcs (`<= 0x10_FFFF` Unicode codepoints and `> 2_228_223` out-of-range IDs)
/// are embedded via Dc 308 (`[308, ...u128_to_dc_number_short(dcid)...]`).
pub fn dclist_to_dcarray(
    dclist: &[u128],
) -> Result<ConversionOutput<Vec<u32>>> {
    let log = FormatLog::default();
    let mut result = Vec::new();

    for &dcid in dclist {
        if dcid == GID_LONG_DC {
            result.push(SHORT_DC_ESCAPE);
            result.push(SHORT_DC_LONG_DC);
        } else if dcid == GID_ESCAPE {
            result.push(SHORT_DC_ESCAPE);
            result.push(SHORT_DC_ESCAPE);
        } else if (SHORT_DC_REGION_START..=SHORT_DC_REGION_END).contains(&dcid) {
            let diff = dcid.saturating_sub(SHORT_DC_REGION_START);
            let short_dc = u32::try_from(diff)
                .context("Direct short Dc offset exceeds u32 range")?;
            result.push(short_dc);
        } else {
            result.push(SHORT_DC_LONG_DC);
            let num_dcs = u128_to_dc_number_short(dcid)?;
            result.extend(num_dcs);
        }
    }

    Ok(ConversionOutput::new(result, log))
}

/// Converts a DcText document (`&[u8]`) to an EITE DcArray (short Dcs, `Vec<u32>`).
pub fn dctext_to_dcarray(
    document: &[u8],
) -> Result<ConversionOutput<Vec<u32>>> {
    let conv = dctext_to_dclist(document)?;
    let array_conv = dclist_to_dcarray(&conv.result)?;
    let mut total_log = conv.log;
    total_log.merge(&array_conv.log);
    Ok(ConversionOutput::new(array_conv.result, total_log))
}

/// Formats a byte slice as a displayable preview string.
/// If `is_dctext` is true, first converts DcText to plain text.
/// If the text is displayable (only graphic/whitespace chars), truncates it to
/// 60 chars and returns it. Otherwise, returns a `<binary data: N bytes>` preview.
pub fn format_blob_preview(data: &[u8], is_dctext: bool) -> String {
    let raw_bytes = if is_dctext {
        dcutf_to_dctext(data.to_vec())
    } else {
        data.to_vec()
    };
    if let Ok(s) = String::from_utf8(raw_bytes) {
        let displayable = s
            .chars()
            .all(|c| c.is_ascii_graphic() || c.is_ascii_whitespace());
        if displayable {
            if s.len() > 60 {
                let truncated: String = s.chars().take(60).collect();
                if s.chars().count() > 60 {
                    format!("{truncated}...")
                } else {
                    truncated
                }
            } else {
                s
            }
        } else {
            format!("<binary data: {} bytes>", data.len())
        }
    } else {
        format!("<binary data: {} bytes>", data.len())
    }
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
    use crate::dc_number::*;

    #[crate::ctb_test]
    fn test_format_blob_preview() {
        assert_eq!(format_blob_preview(b"hello world", false), "hello world");
        assert_eq!(
            format_blob_preview(&[0u8, 1u8, 2u8], false),
            "<binary data: 3 bytes>"
        );
        let long_str = "a".repeat(70);
        assert_eq!(
            format_blob_preview(long_str.as_bytes(), false),
            format!("{}...", "a".repeat(60))
        );
    }

    #[crate::ctb_test]
    fn test_dctext_to_dcutf() {
        let text = "hi @64@ @@ @65@ @128@ there 🥴 @L42@ noncharacter @1114111@ surrogate @56191@ unicode null @0@ dc null @1114112@ @2147483648@ 2^128-1 @340282366920938463463374607431768211455@";
        let dcutf = dctext_to_dcutf(text.as_bytes().to_vec()).unwrap();
        assert_eq!(
            "686920402040204120c28020746865726520f09fa5b420ff84849084a8ff8484908086ff8488a08387ff84849082a9ff8484908087206e6f6e63686172616374657220f48fbfbf20737572726f6761746520edadbf20756e69636f6465206e756c6c2000206463206e756c6c20ff848490808020ff8682808080808020325e3132382d3120ff9683bfbfbfbfbfbfbfbfbfbfbfbfbfbfbfbfbfbfbfbfbf",
            bin2hex(&dcutf)
        );

        let roundtrip = dcutf_to_dctext(dcutf.clone());
        let roundtrip_str = String::from_utf8(roundtrip).unwrap();

        // Should match original
        let expected_roundtrip = "hi @@ @@ A \u{80} there 🥴 @L42@ noncharacter \u{10ffff} surrogate @56191@ unicode null \u{0} dc null @1114112@ @2147483648@ 2^128-1 @340282366920938463463374607431768211455@";
        assert!(roundtrip_str.eq(expected_roundtrip));
    }

    #[crate::ctb_test]
    fn test_dctext_to_dcstring() {
        let text = "hi @64@ @@ @65@ @128@ there 🥴 @L42@";
        let out = dctext_to_dcstring(text.as_bytes()).unwrap();
        let dc_string = out.result;
        assert_eq!(
            dc_string.as_bytes(),
            dctext_to_dcutf(text.as_bytes().to_vec()).unwrap().as_slice()
        );

        let roundtrip = dcstring_to_dctext(&dc_string);
        let roundtrip_str = String::from_utf8(roundtrip).unwrap();
        assert_eq!(roundtrip_str, "hi @@ @@ A \u{80} there 🥴 @L42@");
    }

    #[crate::ctb_test]
    fn test_dcarray_to_dctext_and_back() {
        let original_dcarray = vec![0, 1, 18, 50, 200, 297];
        let converted = dcarray_to_dctext(&original_dcarray)
            .expect("conversion should succeed");
        assert!(!converted.log.has_warnings());
        assert_eq!(
            String::from_utf8(converted.result.clone()).expect("valid utf-8"),
            "@1114112@@1114113@@1114130@@1114162@@1114312@@1114409@"
        );

        let back = dctext_to_dcarray(&converted.result)
            .expect("reverse conversion should succeed");
        assert!(!back.log.has_warnings());
        assert_eq!(back.result, original_dcarray);
    }

    #[crate::ctb_test]
    fn test_dctext_to_dcarray_direct_short_dc() {
        // DcText Dc ID 1114500 (offset 388, valid direct short Dc)
        let input = b"@1114500@";
        let out =
            dctext_to_dcarray(input).expect("conversion should succeed");
        assert!(!out.log.has_warnings());
        assert_eq!(out.result, vec![388]);

        let back = dcarray_to_dclist(&out.result)
            .expect("reverse conversion should succeed");
        assert_eq!(back.result, vec![1_114_500]);
    }

    #[crate::ctb_test]
    fn test_dclist_to_dcarray_dc308_embedding_roundtrip() {
        // Format ID (2228423), 32-bit+ ID (4294967296), and large global ID (100000000000)
        let original_dclist = vec![2_228_423, 4_294_967_296, 100_000_000_000];
        let array_out =
            dclist_to_dcarray(&original_dclist).expect("should succeed");
        assert!(!array_out.log.has_warnings());
        assert!(array_out.result.contains(&SHORT_DC_LONG_DC));

        let back = dcarray_to_dclist(&array_out.result)
            .expect("reverse conversion should succeed");
        assert!(!back.log.has_warnings());
        assert_eq!(back.result, original_dclist);
    }

    #[crate::ctb_test]
    fn test_dclist_to_dcarray_unicode_lossless_roundtrip() {
        // Unicode codepoints: 'A' (65), ' ' (32), emoji 🥴 (129396), surrogate 0xD800 (55296)
        let original_dclist = vec![65, 32, 129_396, 55_296];
        let array_out =
            dclist_to_dcarray(&original_dclist).expect("should succeed");
        assert!(!array_out.log.has_warnings());
        assert!(array_out.result.contains(&SHORT_DC_LONG_DC));

        let back = dcarray_to_dclist(&array_out.result)
            .expect("reverse conversion should succeed");
        assert!(!back.log.has_warnings());
        assert_eq!(back.result, original_dclist);
    }

    #[crate::ctb_test]
    fn test_dc308_escaping_roundtrip() {
        // Long Dc 1114420 (Dc 308) escaped with 255 -> [255, 308]
        let original_dclist = vec![GID_LONG_DC];
        let array_out =
            dclist_to_dcarray(&original_dclist).expect("should succeed");
        assert_eq!(array_out.result, vec![SHORT_DC_ESCAPE, SHORT_DC_LONG_DC]);

        let back = dcarray_to_dclist(&array_out.result)
            .expect("reverse conversion should succeed");
        assert_eq!(back.result, original_dclist);
    }

    #[crate::ctb_test]
    fn test_dc255_escaping_roundtrip() {
        // Long Dc 1114367 (Dc 255) escaped with 255 -> [255, 255]
        let original_dclist = vec![GID_ESCAPE];
        let array_out =
            dclist_to_dcarray(&original_dclist).expect("should succeed");
        assert_eq!(array_out.result, vec![SHORT_DC_ESCAPE, SHORT_DC_ESCAPE]);

        let back = dcarray_to_dclist(&array_out.result)
            .expect("reverse conversion should succeed");
        assert_eq!(back.result, original_dclist);
    }

    #[crate::ctb_test]
    fn test_dc308_followed_by_dc_number_disambiguation() {
        // Long Dc 1114420 followed by a Dc number in DcList
        let original_dclist = vec![
            GID_LONG_DC,
            GID_BEGIN_NUMBER,
            GID_FORMAT_199,
            GID_BASE64_START,
            GID_END_NUMBER,
        ];
        let array_out =
            dclist_to_dcarray(&original_dclist).expect("should succeed");
        assert_eq!(
            array_out.result.get(0..2),
            Some(&[SHORT_DC_ESCAPE, SHORT_DC_LONG_DC][..])
        );

        let back = dcarray_to_dclist(&array_out.result)
            .expect("reverse conversion should succeed");
        assert_eq!(back.result, original_dclist);
    }

    #[crate::ctb_test]
    fn test_dcarray_to_dclist_standalone_dc308() {
        // Standalone short Dc 308 not followed by a Dc number (graceful fallback)
        let dc_array = vec![SHORT_DC_LONG_DC, 65];
        let back = dcarray_to_dclist(&dc_array).expect("should succeed");
        assert!(back.log.has_warnings());
        assert_eq!(
            back.result,
            vec![GID_LONG_DC, SHORT_DC_REGION_START.saturating_add(65)]
        );
    }

    #[crate::ctb_test]
    fn test_dclist_roundtrip() {
        let text = b"hi @1114112@ @L42@ @2147483648@";
        let dclist_out = dctext_to_dclist(text).expect("should succeed");
        assert!(!dclist_out.log.has_warnings());

        let restored_text = dclist_to_dctext(&dclist_out.result);
        assert_eq!(
            std::str::from_utf8(&restored_text).unwrap(),
            "hi @1114112@ @L42@ @2147483648@"
        );
    }

    // #[crate::ctb_test]
    // fn test_dctext_to_dcarray_lossy_warnings() {
    //     // Out-of-range DcText Dc ID (e.g. 1114500)
    //     let lossy_input = b"@1114500@";
    //     let out =
    //         dctext_to_dcarray(lossy_input).expect("conversion should succeed");
    //     assert!(out.log.has_warnings());
    //     assert_eq!(out.result, vec![207]);
    // }

    // #[crate::ctb_test]
    // fn test_dctext_to_dcarray_encapsulated_utf8() {
    //     // Unmappable UTF-8 character 🥴 (U+1F974)
    //     let unmappable_input = "hi 🥴 bye".as_bytes();
    //     let out = dctext_to_dcarray(unmappable_input)
    //         .expect("conversion should succeed");
    //     assert!(out.log.has_warnings());
    //     // Should contain short Dcs for "hi ", then 191 (start encapsulation), Base64 Dcs, 192 (end encapsulation), then " bye"
    //     assert!(out.result.contains(&191));
    //     assert!(out.result.contains(&192));
    // }


}
