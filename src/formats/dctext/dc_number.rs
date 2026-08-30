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

//! Document Character (Dc) numeric encoding and base64 encapsulation utilities.
//!
//! Provides encoding and decoding of numbers in Dc streams using Base64
//! encapsulation digits (short Dcs 127..=190 and 195).
//!
//! Standard structure of a Dc number:
//! 1. Dc 6 (`Begin number`)
//! 2. Dc for format 199 (Global Graph ID `2228423` or short Dc `199`)
//! 3. Optional Dc 10 (`Positive`) or Dc 11 (`Negative`)
//! 4. Absolute value of integer converted to Base64 (using `ctb_formats_math::base`)
//!    and encoded as Base64 encapsulation Dcs (127..=190)
//! 5. Dc 7 (`End number`)

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::{Context, Result, anyhow, bail, ensure};
use ctb_formats_math::base::{Base, format_natural, parse_natural};
use ctb_storage_minimal::global_graph_layout::{DC_REGION_START, dc_to_gid};
use malachite::{Integer, Natural};

/// Short Dc ID for `Begin number` (Dc 6).
pub const SHORT_DC_BEGIN_NUMBER: u32 = 6;
/// Short Dc ID for `End number` (Dc 7).
pub const SHORT_DC_END_NUMBER: u32 = 7;
/// Short Dc ID for `Positive` (Dc 10).
pub const SHORT_DC_POSITIVE: u32 = 10;
/// Short Dc ID for `Negative` (Dc 11).
pub const SHORT_DC_NEGATIVE: u32 = 11;
/// Short Format ID / Dc ID for Format 199.
pub const SHORT_ID_FORMAT_199: u32 = 199;

/// Global Graph ID for `Begin number` (Dc 6, `1114118`).
pub const GID_BEGIN_NUMBER: u128 = 1_114_118;
/// Global Graph ID for `End number` (Dc 7, `1114119`).
pub const GID_END_NUMBER: u128 = 1_114_119;
/// Global Graph ID for `Positive` (Dc 10, `1114122`).
pub const GID_POSITIVE: u128 = 1_114_122;
/// Global Graph ID for `Negative` (Dc 11, `1114123`).
pub const GID_NEGATIVE: u128 = 1_114_123;
/// Global Graph ID for Format 199 (`2228423`).
pub const GID_FORMAT_199: u128 = 2_228_423;

/// First short Dc ID for Base64 encapsulation digits (digit 0 = 'A' = Dc 127).
pub const SHORT_DC_BASE64_START: u32 = 127;
/// Last short Dc ID for Base64 encapsulation digits (digit 63 = '/' = Dc 190).
pub const SHORT_DC_BASE64_END: u32 = 190;
/// Short Dc ID for Base64 encapsulation padding character ('=' = Dc 195).
pub const SHORT_DC_BASE64_PADDING: u32 = 195;

/// First Global Graph ID for Base64 encapsulation digits (`1114239`).
pub const GID_BASE64_START: u128 = 1_114_239;
/// Last Global Graph ID for Base64 encapsulation digits (`1114302`).
pub const GID_BASE64_END: u128 = 1_114_302;
/// Global Graph ID for Base64 encapsulation padding character (`1114307`).
pub const GID_BASE64_PADDING: u128 = 1_114_307;

// ---------------------------------------------------------------------------
// Base64 Character <-> Short/Global Dc Mappings
// ---------------------------------------------------------------------------

/// Converts a single standard Base64 character or padding character (`=`) into
/// its corresponding short Document Character (Dc) ID (127..=190, 195).
pub fn base64_char_to_short_dc(c: char) -> Result<u32> {
    match c {
        'A'..='Z' => {
            let offset = u32::from(c).saturating_sub(u32::from('A'));
            Ok(SHORT_DC_BASE64_START.saturating_add(offset))
        }
        'a'..='z' => {
            let offset = u32::from(c).saturating_sub(u32::from('a'));
            Ok(153u32.saturating_add(offset))
        }
        '0'..='9' => {
            let offset = u32::from(c).saturating_sub(u32::from('0'));
            Ok(179u32.saturating_add(offset))
        }
        '+' => Ok(189),
        '/' => Ok(190),
        '=' => Ok(SHORT_DC_BASE64_PADDING),
        _ => bail!("Character '{c}' is not a valid standard Base64 digit or padding character"),
    }
}

/// Converts a single short Document Character (Dc) ID (127..=190, 195) into
/// its corresponding standard Base64 character or padding character (`=`).
pub fn short_dc_to_base64_char(dc: u32) -> Result<char> {
    match dc {
        127..=152 => {
            let offset = dc.saturating_sub(127);
            let code = u32::from(b'A').saturating_add(offset);
            char::from_u32(code).ok_or_else(|| anyhow!("Invalid char code {code}"))
        }
        153..=178 => {
            let offset = dc.saturating_sub(153);
            let code = u32::from(b'a').saturating_add(offset);
            char::from_u32(code).ok_or_else(|| anyhow!("Invalid char code {code}"))
        }
        179..=188 => {
            let offset = dc.saturating_sub(179);
            let code = u32::from(b'0').saturating_add(offset);
            char::from_u32(code).ok_or_else(|| anyhow!("Invalid char code {code}"))
        }
        189 => Ok('+'),
        190 => Ok('/'),
        195 => Ok('='),
        _ => bail!("Dc ID {dc} is not a valid Base64 encapsulation Dc (127..=190, 195)"),
    }
}

/// Converts a string of standard Base64 characters and padding into a vector
/// of short Document Characters (127..=190, 195).
pub fn base64_str_to_short_dcs(s: &str) -> Result<Vec<u32>> {
    let mut dcs = Vec::with_capacity(s.len());
    for c in s.chars() {
        dcs.push(base64_char_to_short_dc(c)?);
    }
    Ok(dcs)
}

/// Converts a sequence of short Document Characters (127..=190, 195) into
/// a standard Base64 string.
pub fn short_dcs_to_base64_str(dcs: &[u32]) -> Result<String> {
    let mut s = String::with_capacity(dcs.len());
    for &dc in dcs {
        s.push(short_dc_to_base64_char(dc)?);
    }
    Ok(s)
}

/// Converts a single standard Base64 character or padding character (`=`) into
/// its corresponding Global Graph ID.
pub fn base64_char_to_global_dc(c: char) -> Result<u128> {
    let short_dc = base64_char_to_short_dc(c)?;
    Ok(dc_to_gid(u64::from(short_dc)))
}

/// Converts a single Global Graph ID (in the Base64 encapsulation range) into
/// its corresponding standard Base64 character or padding character (`=`).
pub fn global_dc_to_base64_char(gid: u128) -> Result<char> {
    ensure!(
        (GID_BASE64_START..=GID_BASE64_END).contains(&gid) || gid == GID_BASE64_PADDING,
        "Global ID {gid} is not a valid Base64 encapsulation Dc"
    );
    let short_dc = u32::try_from(gid.saturating_sub(DC_REGION_START))
        .context("Global ID exceeds short Dc range")?;
    short_dc_to_base64_char(short_dc)
}

/// Converts a string of standard Base64 characters into Global Graph IDs.
pub fn base64_str_to_global_dcs(s: &str) -> Result<Vec<u128>> {
    let mut gids = Vec::with_capacity(s.len());
    for c in s.chars() {
        gids.push(base64_char_to_global_dc(c)?);
    }
    Ok(gids)
}

/// Converts a sequence of Global Graph IDs into a standard Base64 string.
pub fn global_dcs_to_base64_str(gids: &[u128]) -> Result<String> {
    let mut s = String::with_capacity(gids.len());
    for &gid in gids {
        s.push(global_dc_to_base64_char(gid)?);
    }
    Ok(s)
}

// ---------------------------------------------------------------------------
// Dc Number Serialization
// ---------------------------------------------------------------------------

/// Converts a [`Natural`] number (with an optional negative sign flag) into
/// a sequence of short Document Characters (Dcs).
///
/// Structure:
/// `[Dc 6, Format 199, (optional Dc 11 if negative), ...Base64 encapsulation Dcs..., Dc 7]`
pub fn natural_to_dc_number_short(val: &Natural, is_negative: bool) -> Result<Vec<u32>> {
    let b64_base = Base::new(64)?;
    let b64_str = format_natural(val, b64_base, 0)?;
    let digit_dcs = base64_str_to_short_dcs(&b64_str)?;

    let mut result = Vec::with_capacity(digit_dcs.len().saturating_add(4));
    result.push(SHORT_DC_BEGIN_NUMBER);
    result.push(SHORT_ID_FORMAT_199);
    if is_negative {
        result.push(SHORT_DC_NEGATIVE);
    }
    result.extend(digit_dcs);
    result.push(SHORT_DC_END_NUMBER);
    Ok(result)
}

/// Converts a [`Natural`] number (with an optional negative sign flag) into
/// a sequence of Global Graph IDs (`DcList`).
///
/// Structure:
/// `[GID 1114118 (Dc 6), GID 2228423 (Format 199), (optional GID 1114123 if negative), ...Base64 encapsulation GIDs..., GID 1114119 (Dc 7)]`
pub fn natural_to_dc_number_global(val: &Natural, is_negative: bool) -> Result<Vec<u128>> {
    let short_dcs = natural_to_dc_number_short(val, is_negative)?;
    let mut gids = Vec::with_capacity(short_dcs.len());
    for dc in short_dcs {
        if dc == SHORT_ID_FORMAT_199 {
            gids.push(GID_FORMAT_199);
        } else {
            gids.push(dc_to_gid(u64::from(dc)));
        }
    }
    Ok(gids)
}

/// Converts an [`Integer`] into a sequence of short Document Characters (Dcs).
pub fn integer_to_dc_number_short(val: &Integer) -> Result<Vec<u32>> {
    let is_negative = *val < 0;
    let abs_val = val.unsigned_abs_ref();
    natural_to_dc_number_short(abs_val, is_negative)
}

/// Converts an [`Integer`] into a sequence of Global Graph IDs (`DcList`).
pub fn integer_to_dc_number_global(val: &Integer) -> Result<Vec<u128>> {
    let is_negative = *val < 0;
    let abs_val = val.unsigned_abs_ref();
    natural_to_dc_number_global(abs_val, is_negative)
}

/// Converts an `i128` integer into a sequence of short Document Characters (Dcs).
pub fn i128_to_dc_number_short(val: i128) -> Result<Vec<u32>> {
    integer_to_dc_number_short(&Integer::from(val))
}

/// Converts an `i128` integer into a sequence of Global Graph IDs (`DcList`).
pub fn i128_to_dc_number_global(val: i128) -> Result<Vec<u128>> {
    integer_to_dc_number_global(&Integer::from(val))
}

/// Converts a `u128` unsigned integer into a sequence of short Document Characters (Dcs).
pub fn u128_to_dc_number_short(val: u128) -> Result<Vec<u32>> {
    natural_to_dc_number_short(&Natural::from(val), false)
}

/// Converts a `u128` unsigned integer into a sequence of Global Graph IDs (`DcList`).
pub fn u128_to_dc_number_global(val: u128) -> Result<Vec<u128>> {
    natural_to_dc_number_global(&Natural::from(val), false)
}

// ---------------------------------------------------------------------------
// Dc Number Deserialization / Reading
// ---------------------------------------------------------------------------

/// Reads a single Dc number from a slice of short Document Characters (Dcs),
/// returning the parsed [`Integer`] and the number of tokens consumed from the slice.
pub fn read_dc_number_short(dcs: &[u32]) -> Result<(Integer, usize)> {
    let first = dcs.first().copied().ok_or_else(|| anyhow!("Empty Dc stream"))?;
    ensure!(
        first == SHORT_DC_BEGIN_NUMBER,
        "Expected Dc 6 (Begin number), found Dc {first}"
    );

    let second = dcs.get(1).copied().ok_or_else(|| anyhow!("Unexpected end of stream after Dc 6"))?;
    ensure!(
        second == SHORT_ID_FORMAT_199,
        "Expected format 199 indicator after Dc 6, found Dc {second}"
    );

    let mut idx = 2usize;
    let mut is_negative = false;

    if let Some(&third) = dcs.get(idx) {
        if third == SHORT_DC_NEGATIVE {
            is_negative = true;
            idx = idx.saturating_add(1);
        } else if third == SHORT_DC_POSITIVE {
            idx = idx.saturating_add(1);
        }
    }

    let mut b64_str = String::new();
    let mut found_end = false;

    while idx < dcs.len() {
        let Some(&dc) = dcs.get(idx) else { break };
        idx = idx.saturating_add(1);

        if dc == SHORT_DC_END_NUMBER {
            found_end = true;
            break;
        }

        let ch = short_dc_to_base64_char(dc)?;
        b64_str.push(ch);
    }

    ensure!(found_end, "Missing Dc 7 (End number) terminating Dc number");
    ensure!(!b64_str.is_empty(), "Dc number contains no digit characters");

    let b64_base = Base::new(64)?;
    let nat = parse_natural(&b64_str, b64_base)
        .with_context(|| format!("Failed to parse Base64 number string '{b64_str}'"))?;

    let int_val = Integer::from_sign_and_abs(!is_negative, nat);

    Ok((int_val, idx))
}

/// Reads a single Dc number from a slice of Global Graph IDs (`DcList`),
/// returning the parsed [`Integer`] and the number of tokens consumed from the slice.
pub fn read_dc_number_global(gids: &[u128]) -> Result<(Integer, usize)> {
    let first = gids.first().copied().ok_or_else(|| anyhow!("Empty GID stream"))?;
    ensure!(
        first == GID_BEGIN_NUMBER || first == u128::from(SHORT_DC_BEGIN_NUMBER),
        "Expected GID 1114118 / Dc 6 (Begin number), found {first}"
    );

    let second = gids.get(1).copied().ok_or_else(|| anyhow!("Unexpected end of stream after Dc 6"))?;
    ensure!(
        second == GID_FORMAT_199,
        "Expected format 199 indicator after Dc 6, found {second}"
    );

    let mut idx = 2usize;
    let mut is_negative = false;

    if let Some(&third) = gids.get(idx) {
        if third == GID_NEGATIVE || third == u128::from(SHORT_DC_NEGATIVE) {
            is_negative = true;
            idx = idx.saturating_add(1);
        } else if third == GID_POSITIVE || third == u128::from(SHORT_DC_POSITIVE) {
            idx = idx.saturating_add(1);
        }
    }

    let mut b64_str = String::new();
    let mut found_end = false;

    while idx < gids.len() {
        let Some(&gid) = gids.get(idx) else { break };
        idx = idx.saturating_add(1);

        if gid == GID_END_NUMBER || gid == u128::from(SHORT_DC_END_NUMBER) {
            found_end = true;
            break;
        }

        let ch = if (GID_BASE64_START..=GID_BASE64_END).contains(&gid) || gid == GID_BASE64_PADDING {
            global_dc_to_base64_char(gid)?
        } else if let Ok(short_dc) = u32::try_from(gid)
            && ((SHORT_DC_BASE64_START..=SHORT_DC_BASE64_END).contains(&short_dc)
                || short_dc == SHORT_DC_BASE64_PADDING)
        {
            short_dc_to_base64_char(short_dc)?
        } else {
            bail!("Unexpected token {gid} inside Dc number Base64 body");
        };

        b64_str.push(ch);
    }

    ensure!(found_end, "Missing GID 1114119 / Dc 7 (End number) terminating Dc number");
    ensure!(!b64_str.is_empty(), "Dc number contains no digit characters");

    let b64_base = Base::new(64)?;
    let nat = parse_natural(&b64_str, b64_base)
        .with_context(|| format!("Failed to parse Base64 number string '{b64_str}'"))?;

    let int_val = Integer::from_sign_and_abs(!is_negative, nat);

    Ok((int_val, idx))
}

/// Parses an entire sequence of short Document Characters (Dcs) as a single Dc number.
pub fn parse_dc_number_short(dcs: &[u32]) -> Result<Integer> {
    let (val, consumed) = read_dc_number_short(dcs)?;
    ensure!(
        consumed == dcs.len(),
        "Trailing tokens after Dc number: consumed {consumed} of {} tokens",
        dcs.len()
    );
    Ok(val)
}

/// Parses an entire sequence of Global Graph IDs (`DcList`) as a single Dc number.
pub fn parse_dc_number_global(gids: &[u128]) -> Result<Integer> {
    let (val, consumed) = read_dc_number_global(gids)?;
    ensure!(
        consumed == gids.len(),
        "Trailing tokens after Dc number: consumed {consumed} of {} tokens",
        gids.len()
    );
    Ok(val)
}

/// Parses an entire sequence of short Document Characters (Dcs) as an `i128`.
pub fn parse_dc_number_short_i128(dcs: &[u32]) -> Result<i128> {
    let val = parse_dc_number_short(dcs)?;
    i128::try_from(&val).map_err(|_| anyhow!("Parsed Dc number exceeds i128 range"))
}

/// Parses an entire sequence of Global Graph IDs (`DcList`) as an `i128`.
pub fn parse_dc_number_global_i128(gids: &[u128]) -> Result<i128> {
    let val = parse_dc_number_global(gids)?;
    i128::try_from(&val).map_err(|_| anyhow!("Parsed Dc number exceeds i128 range"))
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
    fn test_base64_char_to_short_dc_mappings() {
        assert_eq!(base64_char_to_short_dc('A').unwrap(), 127);
        assert_eq!(base64_char_to_short_dc('B').unwrap(), 128);
        assert_eq!(base64_char_to_short_dc('Z').unwrap(), 152);
        assert_eq!(base64_char_to_short_dc('a').unwrap(), 153);
        assert_eq!(base64_char_to_short_dc('z').unwrap(), 178);
        assert_eq!(base64_char_to_short_dc('0').unwrap(), 179);
        assert_eq!(base64_char_to_short_dc('9').unwrap(), 188);
        assert_eq!(base64_char_to_short_dc('+').unwrap(), 189);
        assert_eq!(base64_char_to_short_dc('/').unwrap(), 190);
        assert_eq!(base64_char_to_short_dc('=').unwrap(), 195);
        assert!(base64_char_to_short_dc('$').is_err());
    }

    #[crate::ctb_test]
    fn test_short_dc_to_base64_char_mappings() {
        assert_eq!(short_dc_to_base64_char(127).unwrap(), 'A');
        assert_eq!(short_dc_to_base64_char(128).unwrap(), 'B');
        assert_eq!(short_dc_to_base64_char(152).unwrap(), 'Z');
        assert_eq!(short_dc_to_base64_char(153).unwrap(), 'a');
        assert_eq!(short_dc_to_base64_char(178).unwrap(), 'z');
        assert_eq!(short_dc_to_base64_char(179).unwrap(), '0');
        assert_eq!(short_dc_to_base64_char(188).unwrap(), '9');
        assert_eq!(short_dc_to_base64_char(189).unwrap(), '+');
        assert_eq!(short_dc_to_base64_char(190).unwrap(), '/');
        assert_eq!(short_dc_to_base64_char(195).unwrap(), '=');
        assert!(short_dc_to_base64_char(191).is_err());
        assert!(short_dc_to_base64_char(126).is_err());
    }

    #[crate::ctb_test]
    fn test_base64_string_conversion_roundtrip() {
        let text = "Hello+World/123==";
        let dcs = base64_str_to_short_dcs(text).unwrap();
        let roundtrip = short_dcs_to_base64_str(&dcs).unwrap();
        assert_eq!(roundtrip, text);

        let gids = base64_str_to_global_dcs(text).unwrap();
        let g_roundtrip = global_dcs_to_base64_str(&gids).unwrap();
        assert_eq!(g_roundtrip, text);
    }

    #[crate::ctb_test]
    fn test_dc_number_short_roundtrip() {
        // Zero: [6, 199, 127 ('A'), 7]
        let zero_dcs = i128_to_dc_number_short(0).unwrap();
        assert_eq!(zero_dcs, vec![6, 199, 127, 7]);
        assert_eq!(parse_dc_number_short_i128(&zero_dcs).unwrap(), 0);

        // 42: in base64 'q' (digit 42) -> short Dc 127 + 42 = 169
        let dcs_42 = i128_to_dc_number_short(42).unwrap();
        assert_eq!(dcs_42, vec![6, 199, 169, 7]);
        assert_eq!(parse_dc_number_short_i128(&dcs_42).unwrap(), 42);

        // -42: [6, 199, 11 (negative), 169 ('q'), 7]
        let dcs_neg_42 = i128_to_dc_number_short(-42).unwrap();
        assert_eq!(dcs_neg_42, vec![6, 199, 11, 169, 7]);
        assert_eq!(parse_dc_number_short_i128(&dcs_neg_42).unwrap(), -42);

        // Large number: 1_000_000
        let dcs_large = i128_to_dc_number_short(1_000_000).unwrap();
        assert_eq!(parse_dc_number_short_i128(&dcs_large).unwrap(), 1_000_000);
    }

    #[crate::ctb_test]
    fn test_dc_number_global_roundtrip() {
        // 42: [GID 1114118, GID 2228423, GID 1114281, GID 1114119]
        let gids_42 = i128_to_dc_number_global(42).unwrap();
        assert_eq!(gids_42, vec![1_114_118, 2_228_423, 1_114_281, 1_114_119]);
        assert_eq!(parse_dc_number_global_i128(&gids_42).unwrap(), 42);

        // -42: [GID 1114118, GID 2228423, GID 1114123 (neg), GID 1114281, GID 1114119]
        let gids_neg = i128_to_dc_number_global(-42).unwrap();
        assert_eq!(gids_neg, vec![1_114_118, 2_228_423, 1_114_123, 1_114_281, 1_114_119]);
        assert_eq!(parse_dc_number_global_i128(&gids_neg).unwrap(), -42);

        // Read stream with trailing tokens
        let mut stream = gids_42.clone();
        stream.push(65); // Unicode 'A'
        let (val, consumed) = read_dc_number_global(&stream).unwrap();
        assert_eq!(val, Integer::from(42));
        assert_eq!(consumed, 4);
    }
}
