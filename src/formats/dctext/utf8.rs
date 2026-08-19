// SPDX-License-Identifier: AGPL-3.0-or-later
/*
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

//! Conversions between `DcList` (`&[u128]`) and UTF-8 (`&[u8]`).
//!
//! `DcList` is a superset format of UTF-8, where values `0..=0x10FFFF` directly
//! represent Unicode scalar codepoints, and `CLASSIC_DC_OFFSET` (`1_114_112`)
//! offsets classic EITE Document Character IDs. Unmappable `DcList` values can
//! be encapsulated into UTF-8 using `dcl_basenb` (base17) armoring, replaced
//! with UTF-8 replacement characters (`U+FFFD`), or skipped.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use ctb_utilities::*;

use anyhow::{Result, anyhow};
use const_default::ConstDefault;

use ctb_formats_eite::dc::{
    DC_END_ENCAPSULATION_UTF8, DC_START_ENCAPSULATION_UTF8,
    dc_encapsulated_raw_to_bytes, is_dc_base64_encapsulation_character,
    maximum_known_dc,
};
use ctb_formats_eite::encoding::basenb::{
    byte_array_from_basenb_17_utf8, byte_array_to_basenb_17_utf8,
    is_basenb_char,
};
use ctb_formats_utf8::{UTF8_REPLACEMENT_CHARACTER, first_char_of_utf8_string};
use ctb_formats_utilities::{ConversionOutput, FormatLog};

use crate::{CLASSIC_DC_OFFSET, DcList, dclist_to_dcutf, dcutf_to_dclist};

/// Raw 16-byte array for Start UUID: `1880aba3-21df-42b2-9c96-e32cd647ffc5`
pub const DCL_BASENB_START_UUID_RAW: [u8; 16] = [
    0x18, 0x80, 0xab, 0xa3, 0x21, 0xdf, 0x42, 0xb2, 0x9c, 0x96, 0xe3, 0x2c,
    0xd6, 0x47, 0xff, 0xc5,
];

/// Raw 16-byte array for End UUID: `27efca19-0439-4bec-b58f-dfff5cd8db9f`
pub const DCL_BASENB_END_UUID_RAW: [u8; 16] = [
    0x27, 0xef, 0xca, 0x19, 0x04, 0x39, 0x4b, 0xec, 0xb5, 0x8f, 0xdf, 0xff,
    0x5c, 0xd8, 0xdb, 0x9f,
];

/// Returns the base17 UTF-8 bytes for the `DcList` start UUID sentinel.
pub fn dcl_basenb_start_uuid_bytes() -> Result<Vec<u8>> {
    byte_array_to_basenb_17_utf8(&DCL_BASENB_START_UUID_RAW)
}

/// Returns the base17 UTF-8 bytes for the `DcList` end UUID sentinel.
pub fn dcl_basenb_end_uuid_bytes() -> Result<Vec<u8>> {
    byte_array_to_basenb_17_utf8(&DCL_BASENB_END_UUID_RAW)
}

/// Configuration settings for `DcList` to/from UTF-8 conversion.
#[derive(Debug, Clone)]
pub struct DcListUtf8Settings {
    /// Enable `dcl_basenb` (base17) encoding for unmappable Dcs in output/input.
    pub dcl_basenb_enabled: bool,
    /// If true, `dcl_basenb` regions are raw runs of base17 characters without
    /// UUID sentinels.
    pub dcl_basenb_fragment_enabled: bool,
    /// If true, fail strictly on fragment decoding errors instead of emitting a warning.
    pub dcl_basenb_fragment_strict: bool,
    /// Skip unmappable characters entirely when outputting to UTF-8 (when
    /// `dcl_basenb_enabled` is false).
    pub skip_unmappable: bool,
    /// When true, map classic Dcs to their corresponding Unicode output character
    /// (if available), and parse legacy base64 UTF-8 embeds (`191..192`) in input documents.
    pub canonicalize_equivalent_dcs: bool,
    /// Enable debug logging.
    pub debug: bool,
}

impl Default for DcListUtf8Settings {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl ConstDefault for DcListUtf8Settings {
    const DEFAULT: Self = Self {
        dcl_basenb_enabled: false,
        dcl_basenb_fragment_enabled: false,
        dcl_basenb_fragment_strict: true,
        skip_unmappable: false,
        canonicalize_equivalent_dcs: false,
        debug: false,
    };
}

/// Converts a `DcList` (`&[u128]`) into UTF-8 bytes (`Vec<u8>`).
pub fn dclist_to_utf8(
    dclist: &[u128],
    settings: &DcListUtf8Settings,
) -> Result<ConversionOutput<Vec<u8>>> {
    let mut log = FormatLog::default();
    let mut out = Vec::new();
    let mut unmappables: Vec<u128> = Vec::new();
    let mut found_any_unmappables = false;
    let max_classic_u128 = u128::try_from(maximum_known_dc()?)
        .map_err(|e| anyhow!("Failed to convert maximum_known_dc: {e}"))?;

    let start_uuid_bytes = dcl_basenb_start_uuid_bytes()?;
    let end_uuid_bytes = dcl_basenb_end_uuid_bytes()?;

    let mut i = 0;
    while i < dclist.len() {
        let Some(&dc) = dclist.get(i) else { break };

        // Case 1: Standard Unicode codepoint (0..=0x10FFFF, excluding surrogates)
        if dc <= 0x10_FFFF {
            if let Ok(cp_u32) = u32::try_from(dc) {
                if let Some(ch) = char::from_u32(cp_u32) {
                    flush_unmappables(
                        &mut out,
                        &mut unmappables,
                        &mut found_any_unmappables,
                        false,
                        settings,
                        &start_uuid_bytes,
                    )?;
                    let mut buf = [0u8; 4];
                    let encoded = ch.encode_utf8(&mut buf);
                    out.extend_from_slice(encoded.as_bytes());
                    i = i.saturating_add(1);
                    continue;
                }
            }
        }

        // Case 2: Classic Dc offset range (CLASSIC_DC_OFFSET..=CLASSIC_DC_OFFSET + max_classic)
        if dc >= CLASSIC_DC_OFFSET {
            let diff = dc.saturating_sub(CLASSIC_DC_OFFSET);
            if diff <= max_classic_u128 {
                let classic_dc = u32::try_from(diff)
                    .map_err(|e| anyhow!("Classic Dc overflow: {e}"))?;

                if settings.canonicalize_equivalent_dcs {
                    // Sub-case 2a: Legacy base64 UTF-8 sequence starting with DC_START_ENCAPSULATION_UTF8 (191)
                    if classic_dc == DC_START_ENCAPSULATION_UTF8 {
                        let mut j = i.saturating_add(1);
                        let mut truncated = true;
                        #[allow(
                            clippy::expect_used,
                            reason = "j is bounded by dclist.len() loop condition, so dclist.get(j) is in bounds"
                        )]
                        while j < dclist.len() {
                            let cur_dc = *dclist
                                .get(j)
                                .expect("j < dclist.len() guarantees in-bounds access");
                            if cur_dc >= CLASSIC_DC_OFFSET {
                                let cur_diff =
                                    cur_dc.saturating_sub(CLASSIC_DC_OFFSET);
                                if let Ok(cur_classic) = u32::try_from(cur_diff)
                                {
                                    if cur_classic == DC_END_ENCAPSULATION_UTF8
                                    {
                                        truncated = false;
                                        break;
                                    }
                                    if !is_dc_base64_encapsulation_character(
                                        cur_classic,
                                    ) {
                                        break;
                                    }
                                } else {
                                    break;
                                }
                            } else {
                                break;
                            }
                            j = j.saturating_add(1);
                        }

                        if !truncated {
                            let mut inner_dcs = Vec::new();
                            for k in i.saturating_add(1)..j {
                                if let Some(&k_dc) = dclist.get(k) {
                                    if let Ok(k_classic) = u32::try_from(
                                        k_dc.saturating_sub(CLASSIC_DC_OFFSET),
                                    ) {
                                        inner_dcs.push(k_classic);
                                    }
                                }
                            }
                            if let Ok(decoded_bytes) =
                                dc_encapsulated_raw_to_bytes(&inner_dcs)
                            {
                                flush_unmappables(
                                    &mut out,
                                    &mut unmappables,
                                    &mut found_any_unmappables,
                                    false,
                                    settings,
                                    &start_uuid_bytes,
                                )?;
                                out.extend_from_slice(&decoded_bytes);
                                i = j.saturating_add(1);
                                continue;
                            }
                        }
                    }

                    // Sub-case 2b: Standard classic Dc to UTF-8 mapping
                    if let Ok((mapped_bytes, dc_log)) =
                        ctb_formats_eite::formats::dc_to_format(
                            "utf8", classic_dc,
                        )
                    {
                        log.merge(&dc_log);
                        if !mapped_bytes.is_empty() {
                            flush_unmappables(
                                &mut out,
                                &mut unmappables,
                                &mut found_any_unmappables,
                                false,
                                settings,
                                &start_uuid_bytes,
                            )?;
                            out.extend_from_slice(&mapped_bytes);
                            i = i.saturating_add(1);
                            continue;
                        }
                    }
                }
            }
        }

        // Case 3: Unmappable Dc
        if settings.dcl_basenb_enabled {
            unmappables.push(dc);
        } else {
            // Reason for fallback: if usize index fails u64 conversion, fallback 0 is used for log position.
            let idx_u64 = u64::try_from(i).unwrap_or(0);
            log.export_warning(
                idx_u64,
                &format!("Dc {dc} has no UTF-8 mapping"),
            );
            if !settings.skip_unmappable {
                out.extend_from_slice(UTF8_REPLACEMENT_CHARACTER);
            }
        }

        i = i.saturating_add(1);
    }

    // Flush any remaining unmappables
    flush_unmappables(
        &mut out,
        &mut unmappables,
        &mut found_any_unmappables,
        true,
        settings,
        &start_uuid_bytes,
    )?;

    // Append end UUID sentinel if armoring was active
    if settings.dcl_basenb_enabled
        && found_any_unmappables
        && !settings.dcl_basenb_fragment_enabled
    {
        out.extend_from_slice(&end_uuid_bytes);
    }

    Ok(ConversionOutput::new(out, log))
}

/// Flush accumulated unmappables using `dclist_to_dcutf` and `byte_array_to_basenb_17_utf8`.
fn flush_unmappables(
    out: &mut Vec<u8>,
    unmappables: &mut Vec<u128>,
    found_any_unmappables: &mut bool,
    force: bool,
    settings: &DcListUtf8Settings,
    start_uuid_bytes: &[u8],
) -> Result<()> {
    if settings.dcl_basenb_enabled
        && (force || !unmappables.is_empty())
        && !unmappables.is_empty()
    {
        if !*found_any_unmappables && !settings.dcl_basenb_fragment_enabled {
            out.extend_from_slice(start_uuid_bytes);
        }
        *found_any_unmappables = true;

        let dcutf_bytes = dclist_to_dcutf(unmappables);
        let encoded_base17 = byte_array_to_basenb_17_utf8(&dcutf_bytes)?;
        out.extend_from_slice(&encoded_base17);

        unmappables.clear();
    }
    Ok(())
}

/// Decodes UTF-8 bytes (`&[u8]`) into a `DcList` (`Vec<u128>`).
pub fn dclist_from_utf8(
    utf8_bytes: &[u8],
    settings: &DcListUtf8Settings,
) -> Result<ConversionOutput<DcList>> {
    let mut log = FormatLog::default();
    let mut result = Vec::new();
    let mut remaining = utf8_bytes;

    let start_uuid = dcl_basenb_start_uuid_bytes()?;
    let end_uuid = dcl_basenb_end_uuid_bytes()?;

    while !remaining.is_empty() {
        if settings.dcl_basenb_enabled {
            let is_start_basenb = if settings.dcl_basenb_fragment_enabled {
                if let Ok((first_char, consumed)) =
                    first_char_of_utf8_string(remaining)
                {
                    consumed > 0 && is_basenb_char(&first_char)
                } else {
                    false
                }
            } else {
                remaining.starts_with(&start_uuid)
            };

            if is_start_basenb {
                if !settings.dcl_basenb_fragment_enabled {
                    if let Some(rem) = remaining.get(start_uuid.len()..) {
                        remaining = rem;
                    } else {
                        remaining = &[];
                    }
                }

                let end_pos = if settings.dcl_basenb_fragment_enabled {
                    let mut consumed_total = 0;
                    while consumed_total < remaining.len() {
                        let Some(sub) = remaining.get(consumed_total..) else {
                            break;
                        };
                        if let Ok((ch_bytes, ch_len)) =
                            first_char_of_utf8_string(sub)
                        {
                            if ch_len == 0 || !is_basenb_char(&ch_bytes) {
                                break;
                            }
                            consumed_total =
                                consumed_total.saturating_add(ch_len);
                        } else {
                            break;
                        }
                    }
                    Some(consumed_total)
                } else {
                    remaining
                        .windows(end_uuid.len())
                        .position(|w| w == end_uuid)
                };

                if let Some(pos) = end_pos {
                    let Some(inner_region) = remaining.get(..pos) else {
                        break;
                    };
                    let mut basenb_run = Vec::new();
                    let mut inner_rem = inner_region;

                    while !inner_rem.is_empty() {
                        if let Ok((ch_bytes, ch_len)) =
                            first_char_of_utf8_string(inner_rem)
                        {
                            if ch_len > 0 && is_basenb_char(&ch_bytes) {
                                basenb_run.extend_from_slice(&ch_bytes);
                            } else {
                                if !basenb_run.is_empty() {
                                    match byte_array_from_basenb_17_utf8(
                                        &basenb_run,
                                    ) {
                                        Ok(dcutf_bytes) => {
                                            result.extend(dcutf_to_dclist(
                                                &dcutf_bytes,
                                            ));
                                        }
                                        Err(e) => {
                                            let msg = format!(
                                                "Failed to decode dcl_basenb 17 run: {e}"
                                            );
                                            if settings
                                                .dcl_basenb_fragment_strict
                                            {
                                                log.import_error(0, &msg);
                                            } else {
                                                log.import_warning(0, &msg);
                                            }
                                        }
                                    }
                                    basenb_run.clear();
                                }
                                if ch_len > 0 {
                                    if let Ok(s) =
                                        std::str::from_utf8(&ch_bytes)
                                    {
                                        for ch in s.chars() {
                                            result.push(u128::from(u32::from(
                                                ch,
                                            )));
                                        }
                                    }
                                }
                            }
                            if let Some(rem) = inner_rem.get(ch_len..) {
                                inner_rem = rem;
                            } else {
                                inner_rem = &[];
                            }
                        } else {
                            break;
                        }
                    }

                    if !basenb_run.is_empty() {
                        match byte_array_from_basenb_17_utf8(&basenb_run) {
                            Ok(dcutf_bytes) => {
                                result.extend(dcutf_to_dclist(&dcutf_bytes));
                            }
                            Err(e) => {
                                let msg = format!(
                                    "Failed to decode dcl_basenb 17 run: {e}"
                                );
                                if settings.dcl_basenb_fragment_strict {
                                    log.import_error(0, &msg);
                                } else {
                                    log.import_warning(0, &msg);
                                }
                            }
                        }
                        basenb_run.clear();
                    }

                    if settings.dcl_basenb_fragment_enabled {
                        if let Some(rem) = remaining.get(pos..) {
                            remaining = rem;
                        } else {
                            remaining = &[];
                        }
                    } else {
                        if let Some(rem) = remaining.get(pos.saturating_add(end_uuid.len())..) {
                            remaining = rem;
                        } else {
                            remaining = &[];
                        }
                    }
                    continue;
                }
            }
        }

        // Standard UTF-8 character decoding
        if let Ok((ch_bytes, consumed)) = first_char_of_utf8_string(remaining) {
            if consumed > 0 {
                if let Ok(s) = std::str::from_utf8(&ch_bytes) {
                    for ch in s.chars() {
                        result.push(u128::from(u32::from(ch)));
                    }
                }
                if let Some(rem) = remaining.get(consumed..) {
                    remaining = rem;
                } else {
                    remaining = &[];
                }
                continue;
            }
        }

        // Advance by 1 byte if invalid UTF-8
        if let Some(&b) = remaining.first() {
            log.import_warning(0, &format!("Invalid UTF-8 byte 0x{b:02X}"));
            if let Some(rem) = remaining.get(1..) {
                remaining = rem;
            } else {
                remaining = &[];
            }
        } else {
            break;
        }
    }

    Ok(ConversionOutput::new(result, log))
}

/// Alias for `dclist_from_utf8`.
pub fn utf8_to_dclist(
    utf8_bytes: &[u8],
    settings: &DcListUtf8Settings,
) -> Result<ConversionOutput<DcList>> {
    dclist_from_utf8(utf8_bytes, settings)
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
    fn test_dclist_utf8_basic_unicode() {
        let input: DcList = vec![72, 101, 108, 108, 111, 32, 129314]; // "Hello 🤢"
        let settings = DcListUtf8Settings::default();
        let conv = dclist_to_utf8(&input, &settings).unwrap();
        assert_eq!(String::from_utf8(conv.result.clone()).unwrap(), "Hello 🤢");

        let back = dclist_from_utf8(&conv.result, &settings).unwrap();
        assert_eq!(back.result, input);
    }

    #[crate::ctb_test]
    fn test_dclist_utf8_basenb_armored_roundtrip() {
        let unmappable_dc = CLASSIC_DC_OFFSET + 999_999;
        let input: DcList = vec![72, 105, unmappable_dc, 33];
        let mut settings = DcListUtf8Settings::default();
        settings.dcl_basenb_enabled = true;

        let conv = dclist_to_utf8(&input, &settings).unwrap();
        let back = dclist_from_utf8(&conv.result, &settings).unwrap();
        assert_eq!(back.result, input);
    }

    #[crate::ctb_test]
    fn test_dclist_utf8_basenb_fragment_roundtrip() {
        let unmappable_dc = CLASSIC_DC_OFFSET + 888_888;
        let input: DcList = vec![65, unmappable_dc, 66];
        let mut settings = DcListUtf8Settings::default();
        settings.dcl_basenb_enabled = true;
        settings.dcl_basenb_fragment_enabled = true;

        let conv = dclist_to_utf8(&input, &settings).unwrap();
        let back = dclist_from_utf8(&conv.result, &settings).unwrap();
        assert_eq!(back.result, input);
    }

    #[crate::ctb_test]
    fn test_dclist_utf8_replacement_and_skip() {
        let unmappable_dc = CLASSIC_DC_OFFSET + 777_777;
        let input: DcList = vec![65, unmappable_dc, 66];

        // Replacement mode
        let settings_replace = DcListUtf8Settings::default();
        let conv_replace = dclist_to_utf8(&input, &settings_replace).unwrap();
        assert_eq!(
            String::from_utf8(conv_replace.result).unwrap(),
            "A\u{FFFD}B"
        );

        // Skip mode
        let mut settings_skip = DcListUtf8Settings::default();
        settings_skip.skip_unmappable = true;
        let conv_skip = dclist_to_utf8(&input, &settings_skip).unwrap();
        assert_eq!(String::from_utf8(conv_skip.result).unwrap(), "AB");
    }

    #[crate::ctb_test]
    fn test_dclist_utf8_canonicalize_equivalent_dcs() {
        // Classic Dc 65 maps to 'P'
        let classic_p = CLASSIC_DC_OFFSET + 65;
        let input: DcList = vec![classic_p];

        // Without canonicalization: treated as unmappable (replaced with \u{FFFD} by default)
        let settings_no_canon = DcListUtf8Settings::default();
        let conv_no_canon = dclist_to_utf8(&input, &settings_no_canon).unwrap();
        assert_eq!(
            String::from_utf8(conv_no_canon.result).unwrap(),
            "\u{FFFD}"
        );

        // With canonicalization: maps classic Dc 65 to 'P'
        let mut settings_canon = DcListUtf8Settings::default();
        settings_canon.canonicalize_equivalent_dcs = true;
        let conv_canon = dclist_to_utf8(&input, &settings_canon).unwrap();
        assert_eq!(String::from_utf8(conv_canon.result).unwrap(), "P");
    }
}
