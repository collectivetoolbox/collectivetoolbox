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
//! encodings for sequences of integer global graph IDs (which are new new Dcs).
//! Format looks like (w/o backticks): `Unicode (UTF-8) text @123@miesu@214748364@@L662@`
//! where `Unicode text` is actual unicode text, and between each pair of @ signs, is a DcId. A DcId can be any int 128 bits (u128) in decimal, and it may have an `L` prefix.
//! Output format is sort of UTF-8 text. For normal Unicode input characters, the output character is the same. For DcIds less than or equal to 1114111 (the largest Unicode character, I believe), the output character is the corresponding "generalized UTF-8", the numeric value encoded in the same underlying algorithm as UTF-8. For DcIds greater than 1114111 and not prefixed with an L, the output character is the decimal DcId represented by extending the usual algorithm of UTF-8 encoding, but for those larger numbers. For DcIds prefixed with an L, the output is equivalent to @1114408@ (short Dc 296) followed by a Dc number for the number that followed the L (the L is just a shorthand for that 1114408 Dc). That is to say, it's not a true Unicode encoding, it's simply using an extension of the algorithm underlying UTF-8 as a convenient encoding of ints.
//! Currently, DcList is used as the internal format for pivoting between other formats, but DcUtf might make more sense eventually for space efficiency.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use ctb_formats_utf_8e_128::{decode_utf_8e_128, encode_utf_8e_128_buf};
pub use ctb_formats_utilities::ConversionOutput;
use ctb_formats_utilities::FormatLog;

pub mod character_description;
pub mod cli;
pub mod cli_identifiers;
pub mod dc_number;
pub mod dcal;
pub mod utf8;

pub use dc_number::{
    GID_BASE64_END, GID_BASE64_PADDING, GID_BASE64_START, GID_BEGIN_NUMBER,
    GID_DC_199, GID_END_NUMBER, GID_FORMAT_199, GID_NEGATIVE, GID_POSITIVE,
    SHORT_DC_BASE64_END, SHORT_DC_BASE64_PADDING, SHORT_DC_BASE64_START,
    SHORT_DC_BEGIN_NUMBER, SHORT_DC_END_NUMBER, SHORT_DC_NEGATIVE,
    SHORT_DC_POSITIVE, SHORT_ID_FORMAT_199, base64_char_to_global_dc,
    base64_char_to_short_dc, base64_str_to_global_dcs, base64_str_to_short_dcs,
    global_dc_to_base64_char, global_dcs_to_base64_str, i128_to_dc_number_global,
    i128_to_dc_number_short, integer_to_dc_number_global,
    integer_to_dc_number_short, natural_to_dc_number_global,
    natural_to_dc_number_short, parse_dc_number_global,
    parse_dc_number_global_i128, parse_dc_number_short,
    parse_dc_number_short_i128, read_dc_number_global, read_dc_number_short,
    short_dc_to_base64_char, short_dcs_to_base64_str, u128_to_dc_number_global,
    u128_to_dc_number_short,
};

pub use character_description::{
    describe_dcal, describe_dclist, describe_graph_id,
};
pub use cli::{
    CharacterDescriptionArgs, CharacterDescriptionInputFormat,
    execute_cli_character_description,
};
pub use cli_identifiers::{
    GidArgs, ShortDcArgs, ShortFmtArgs, execute_cli_gid, execute_cli_short_dc,
    execute_cli_short_fmt, parse_graph_or_short_id,
};
pub use dcal::{dcal_to_dclist, dclist_to_dcal};
pub use utf8::{
    DcListUtf8Settings, dclist_from_utf8, dclist_to_utf8, utf8_to_dclist,
};

/// Base offset for classic Document Characters in the global graph layout.
/// Former classic Dc 0 starts at 1114112 (0x110000).
pub const CLASSIC_DC_OFFSET: u128 = 1_114_112;

/// A list of global graph Document Character IDs represented as `u128` values.
pub type DcList = Vec<u128>;

/// Parses a DcText document (`&[u8]`) into a `DcList` (`Vec<u128>`).
///
/// Plain text characters become their corresponding Unicode codepoint IDs (`0..=0x10FFFF`).
/// Tokens in `@<dcid>@` format are parsed into `u128` values.
/// `@L<dcid>@` tokens expand to `1114408` followed by `<dcid>`.
/// `@@` tokens represent codepoint `64` (`@`).
///
/// # Errors
/// Returns an error if the document contains invalid UTF-8 or malformed syntax.
#[expect(
    clippy::expect_used,
    reason = "start and end byte offsets are returned by rest.find('@'), guaranteeing valid char boundaries within rest"
)]
pub fn dctext_to_dclist(document: &[u8]) -> Result<ConversionOutput<DcList>> {
    let mut log = FormatLog::default();
    let text = match std::str::from_utf8(document) {
        Ok(s) => s,
        Err(e) => {
            log.warn(&format!("Invalid UTF-8 in DcText input: {e}"));
            return Ok(ConversionOutput::new(Vec::new(), log));
        }
    };

    let mut list = Vec::new();
    for line in text.lines() {
        let mut rest = line;
        while let Some(start) = rest.find('@') {
            let prefix = rest.get(..start).expect("start is index returned by rest.find('@')");
            for ch in prefix.chars() {
                list.push(u128::from(u32::from(ch)));
            }

            // Reason for fallback: if @ is the last character in rest, start + 1 exceeds string length and empty slice indicates no remaining text.
            rest = rest.get(start.saturating_add(1)..).unwrap_or("");

            if let Some(end) = rest.find('@') {
                let token = rest.get(..end).expect("end is index returned by rest.find('@')");
                let mut dcid_str = token;
                let mut l_prefix = false;

                if dcid_str.is_empty() {
                    dcid_str = "64"; // @@ token represents @ (codepoint 64)
                }
                if let Some(stripped) = dcid_str.strip_prefix('L') {
                    l_prefix = true;
                    dcid_str = stripped;
                }

                if let Ok(dcid) = dcid_str.parse::<u128>() {
                    if l_prefix {
                        list.push(1_114_408u128);
                        list.push(dcid);
                    } else {
                        list.push(dcid);
                    }
                } else {
                    log.warn(&format!(
                        "Invalid DcID token @{token}@ in DcText"
                    ));
                }
                // Reason for fallback: if closing @ is the last character in rest, end + 1 exceeds string length and empty slice indicates no remaining text.
                rest = rest.get(end.saturating_add(1)..).unwrap_or("");
            } else {
                for ch in rest.chars() {
                    list.push(u128::from(u32::from(ch)));
                }
                rest = "";
                break;
            }
        }
        for ch in rest.chars() {
            list.push(u128::from(u32::from(ch)));
        }
    }

    Ok(ConversionOutput::new(list, log))
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
            if let Some(&next_dc) = dclist.get(i.saturating_add(1)) {
                output.push_str(&format!("@L{next_dc}@"));
                i = i.saturating_add(2);
                continue;
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
        let Some(slice) = document.get(i..) else { break };
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

/// Converts DcText format bytes to DcUtf format bytes.
pub fn dctext_to_dcutf(document: Vec<u8>) -> Vec<u8> {
    let dclist = match dctext_to_dclist(&document) {
        Ok(out) => out.result,
        Err(_) => Vec::new(),
    };
    dclist_to_dcutf(&dclist)
}

/// Converts DcUtf format bytes to DcText format bytes.
pub fn dcutf_to_dctext(document: Vec<u8>) -> Vec<u8> {
    let dclist = dcutf_to_dclist(&document);
    dclist_to_dctext(&dclist)
}

/// Converts a classic EITE DcArray (`&[u32]`) to a `DcList` (`Vec<u128>`).
///
/// Each classic Dc ID `c` is mapped to its new global graph Dc ID (`1114112 + c`).
pub fn dcarray_to_dclist(dc_array: &[u32]) -> Result<ConversionOutput<DcList>> {
    let mut log = FormatLog::default();
    let max_known = ctb_formats_eite::dc::maximum_known_dc()?;
    let mut list = Vec::with_capacity(dc_array.len());

    for (idx, &dc) in dc_array.iter().enumerate() {
        let dc_usize = if let Ok(val) = usize::try_from(dc) {
            val
        } else {
            log.warn(&format!(
                "Classic Dc ID {dc} at index {idx} overflows usize"
            ));
            usize::MAX
        };

        if dc_usize > max_known {
            log.warn(&format!(
                "Classic Dc ID {dc} at index {idx} exceeds maximum known classic Dc ID ({max_known})"
            ));
        }

        let new_dc_id = CLASSIC_DC_OFFSET.saturating_add(u128::from(dc));
        list.push(new_dc_id);
    }

    Ok(ConversionOutput::new(list, log))
}

/// Converts an old-style EITE DcArray (`&[u32]`) to the newer DcText format (`Vec<u8>`).
pub fn dcarray_to_dctext(
    dc_array: &[u32],
) -> Result<ConversionOutput<Vec<u8>>> {
    let conv = dcarray_to_dclist(dc_array)?;
    let text_bytes = dclist_to_dctext(&conv.result);
    Ok(ConversionOutput::new(text_bytes, conv.log))
}

/// Converts a `DcList` (`&[u128]`) to an old-style EITE DcArray (`Vec<u32>`).
///
/// Note: DcText / DcList is a superset of classic Dcs, so this is a lossy operation.
/// Runs of unmappable UTF-8 characters are embedded into encapsulated UTF-8 ranges
/// (classic Dcs 191..192), while out-of-range Dc IDs are substituted with
/// replacement Dc ID (`207`).
pub fn dclist_to_dcarray(
    dclist: &[u128],
) -> Result<ConversionOutput<Vec<u32>>> {
    let mut log = FormatLog::default();
    let max_known_u128 = u128::try_from(
        ctb_formats_eite::dc::maximum_known_dc()?,
    )
    .map_err(|e| {
        anyhow::anyhow!("Failed to convert maximum_known_dc to u128: {e}")
    })?;

    let mut result = Vec::new();
    let mut utf8_chunk = String::new();
    let utf8_settings =
        ctb_formats_eite::formats::utf8::UTF8FormatSettings::default();

    let flush_utf8_chunk = |chunk: &mut String,
                            res: &mut Vec<u32>,
                            l: &mut FormatLog|
     -> Result<()> {
        if !chunk.is_empty() {
            let (dcs, chunk_log) =
                ctb_formats_eite::formats::utf8::dca_from_utf8(
                    chunk.as_bytes(),
                    &utf8_settings,
                )?;
            res.extend(dcs);
            l.merge(&chunk_log);
            chunk.clear();
        }
        Ok(())
    };

    for &dcid in dclist {
        if dcid >= CLASSIC_DC_OFFSET {
            let diff = dcid.saturating_sub(CLASSIC_DC_OFFSET);
            if diff <= max_known_u128 {
                if let Ok(classic_dc) = u32::try_from(diff) {
                    flush_utf8_chunk(&mut utf8_chunk, &mut result, &mut log)?;
                    result.push(classic_dc);
                    continue;
                }
            }
            flush_utf8_chunk(&mut utf8_chunk, &mut result, &mut log)?;
            log.warn(&format!(
                "Dc ID {dcid} exceeds maximum classic Dc ID range, replaced with 207"
            ));
            result.push(ctb_formats_eite::dc::DC_REPLACEMENT_UNAVAIL_DC);
        } else if dcid <= 0x10_FFFF {
            if let Ok(cp_u32) = u32::try_from(dcid) {
                if let Some(ch) = char::from_u32(cp_u32) {
                    utf8_chunk.push(ch);
                } else {
                    flush_utf8_chunk(&mut utf8_chunk, &mut result, &mut log)?;
                    log.warn(&format!(
                        "Invalid Unicode codepoint {dcid}, replaced with 207"
                    ));
                    result
                        .push(ctb_formats_eite::dc::DC_REPLACEMENT_UNAVAIL_DC);
                }
            } else {
                flush_utf8_chunk(&mut utf8_chunk, &mut result, &mut log)?;
                log.warn(&format!(
                    "DcID {dcid} overflows u32, replaced with 207"
                ));
                result.push(ctb_formats_eite::dc::DC_REPLACEMENT_UNAVAIL_DC);
            }
        } else {
            flush_utf8_chunk(&mut utf8_chunk, &mut result, &mut log)?;
            log.warn(&format!(
                "Dc ID {dcid} outside classic Dc range, replaced with 207"
            ));
            result.push(ctb_formats_eite::dc::DC_REPLACEMENT_UNAVAIL_DC);
        }
    }

    flush_utf8_chunk(&mut utf8_chunk, &mut result, &mut log)?;
    Ok(ConversionOutput::new(result, log))
}

/// Converts a DcText document (`&[u8]`) to an old-style EITE DcArray (`Vec<u32>`).
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
#[expect(
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
        let dcutf = dctext_to_dcutf(text.as_bytes().to_vec());
        assert_eq!(
            "686920402040204120c28020746865726520f09fa5b420ff84849084a82a206e6f6e63686172616374657220f48fbfbf20737572726f6761746520edadbf20756e69636f6465206e756c6c2000206463206e756c6c20ff848490808020ff8682808080808020325e3132382d3120ff9683bfbfbfbfbfbfbfbfbfbfbfbfbfbfbfbfbfbfbfbfbf",
            bin2hex(&dcutf)
        );

        let roundtrip = dcutf_to_dctext(dcutf.clone());
        let roundtrip_str = String::from_utf8(roundtrip).unwrap();

        // Should match original
        let expected_roundtrip = "hi @@ @@ A \u{80} there 🥴 @L42@ noncharacter \u{10ffff} surrogate @56191@ unicode null \u{0} dc null @1114112@ @2147483648@ 2^128-1 @340282366920938463463374607431768211455@";
        assert!(roundtrip_str.eq(expected_roundtrip));
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
    fn test_dctext_to_dcarray_lossy_warnings() {
        // Out-of-range DcText Dc ID (e.g. 1114500)
        let lossy_input = b"@1114500@";
        let out =
            dctext_to_dcarray(lossy_input).expect("conversion should succeed");
        assert!(out.log.has_warnings());
        assert_eq!(out.result, vec![207]);
    }

    #[crate::ctb_test]
    fn test_dctext_to_dcarray_encapsulated_utf8() {
        // Unmappable UTF-8 character 🥴 (U+1F974)
        let unmappable_input = "hi 🥴 bye".as_bytes();
        let out = dctext_to_dcarray(unmappable_input)
            .expect("conversion should succeed");
        assert!(out.log.has_warnings());
        // Should contain classic Dcs for "hi ", then 191 (start encapsulation), Base64 Dcs, 192 (end encapsulation), then " bye"
        assert!(out.result.contains(&191));
        assert!(out.result.contains(&192));
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
}
