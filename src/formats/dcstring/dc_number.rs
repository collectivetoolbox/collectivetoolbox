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

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::{Context, Result, anyhow, bail, ensure};
use ctb_formats_math::base::{Base, format_natural, parse_natural};
use malachite::{Integer, Natural};

pub use ctb_formats_dcdata::dc::{
    DC_BASE64_END, DC_BASE64_PADDING, DC_BASE64_START, DC_BEGIN_NUMBER,
    DC_END_NUMBER, DC_FORMAT_199, DC_NEGATIVE, DC_POSITIVE,
};
pub use ctb_formats_dcdata::dc_char::DcChar;
pub use ctb_formats_dcdata::dc_number_minimal::{
    dc_base64_char_to_digit, dc_base64_digit_to_char,
    is_dc_base64_encapsulation_char,
};

fn base64_char_to_digit(c: char) -> Result<u8> {
    match c {
        'A'..='Z' => {
            let offset = u32::from(c).saturating_sub(u32::from('A'));
            u8::try_from(offset).map_err(|e| anyhow!("Invalid offset: {e}"))
        }
        'a'..='z' => {
            let offset = u32::from(c).saturating_sub(u32::from('a'));
            u8::try_from(offset.saturating_add(26))
                .map_err(|e| anyhow!("Invalid offset: {e}"))
        }
        '0'..='9' => {
            let offset = u32::from(c).saturating_sub(u32::from('0'));
            u8::try_from(offset.saturating_add(52))
                .map_err(|e| anyhow!("Invalid offset: {e}"))
        }
        '+' => Ok(62),
        '/' => Ok(63),
        '=' => Ok(64),
        _ => bail!(
            "Character '{c}' is not a valid standard Base64 digit or padding character"
        ),
    }
}

fn digit_to_base64_char(digit: u8) -> Result<char> {
    match digit {
        0..=25 => {
            let code = u32::from(b'A').saturating_add(u32::from(digit));
            char::from_u32(code).ok_or_else(|| anyhow!("Invalid char code {code}"))
        }
        26..=51 => {
            let offset = u32::from(digit.saturating_sub(26));
            let code = u32::from(b'a').saturating_add(offset);
            char::from_u32(code).ok_or_else(|| anyhow!("Invalid char code {code}"))
        }
        52..=61 => {
            let offset = u32::from(digit.saturating_sub(52));
            let code = u32::from(b'0').saturating_add(offset);
            char::from_u32(code).ok_or_else(|| anyhow!("Invalid char code {code}"))
        }
        62 => Ok('+'),
        63 => Ok('/'),
        64 => Ok('='),
        _ => bail!("Invalid Base64 digit {digit}"),
    }
}

// ---------------------------------------------------------------------------
// Base64 Character <-> Short/Global Dc Mappings
// ---------------------------------------------------------------------------

/// Converts a single standard Base64 character or padding character (`=`) into
/// its corresponding short Document Character (Dc) ID (127..=190, 195).
pub fn base64_char_to_short_dc(c: char) -> Result<u32> {
    let digit = base64_char_to_digit(c)?;
    dc_base64_digit_to_char(digit).to_short()
}

/// Converts a single short Document Character (Dc) ID (127..=190, 195) into
/// its corresponding standard Base64 character or padding character (`=`).
pub fn short_dc_to_base64_char(dc: u32) -> Result<char> {
    let digit = dc_base64_char_to_digit(DcChar::from_short(dc))?;
    digit_to_base64_char(digit)
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
    let digit = base64_char_to_digit(c)?;
    Ok(dc_base64_digit_to_char(digit).to_long())
}

/// Converts a single Global Graph ID (in the Base64 encapsulation range) into
/// its corresponding standard Base64 character or padding character (`=`).
pub fn global_dc_to_base64_char(gid: u128) -> Result<char> {
    let ch = if let Ok(s) = u32::try_from(gid) && ((127..=190).contains(&s) || s == 195) {
        DcChar::from_short(s)
    } else {
        DcChar::from_long(gid)
    };
    let digit = dc_base64_char_to_digit(ch)?;
    digit_to_base64_char(digit)
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
/// a sequence of [`DcChar`]s.
pub fn natural_to_dc_number_chars(
    val: &Natural,
    is_negative: bool,
) -> Result<Vec<DcChar>> {
    let b64_base = Base::new(64)?;
    let b64_str = format_natural(val, b64_base, 0)?;

    let mut result = Vec::with_capacity(b64_str.len().saturating_add(4));
    result.push(DC_BEGIN_NUMBER);
    result.push(DC_FORMAT_199);
    if is_negative {
        result.push(DC_NEGATIVE);
    }
    for c in b64_str.chars() {
        let digit = base64_char_to_digit(c)?;
        result.push(dc_base64_digit_to_char(digit));
    }
    result.push(DC_END_NUMBER);
    Ok(result)
}

/// Converts a [`Natural`] number (with an optional negative sign flag) into
/// a sequence of short Document Characters (Dcs).
///
/// Structure:
/// `[Dc 6, Format 199, (optional Dc 11 if negative), ...Base64 encapsulation Dcs..., Dc 7]`
pub fn natural_to_dc_number_short(
    val: &Natural,
    is_negative: bool,
) -> Result<Vec<u32>> {
    let chars = natural_to_dc_number_chars(val, is_negative)?;
    let mut out = Vec::with_capacity(chars.len());
    for ch in chars {
        if ch == DC_FORMAT_199 {
            out.push(ch.to_format()?);
        } else {
            out.push(ch.to_short()?);
        }
    }
    Ok(out)
}

/// Converts a [`Natural`] number (with an optional negative sign flag) into
/// a sequence of Global Graph IDs (`DcList`).
///
/// Structure:
/// `[GID 1114118 (Dc 6), GID 2228423 (Format 199), (optional GID 1114123 if negative), ...Base64 encapsulation GIDs..., GID 1114119 (Dc 7)]`
pub fn natural_to_dc_number_global(
    val: &Natural,
    is_negative: bool,
) -> Result<Vec<u128>> {
    let chars = natural_to_dc_number_chars(val, is_negative)?;
    Ok(chars.into_iter().map(DcChar::to_long).collect())
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
    let first = dcs
        .first()
        .copied()
        .ok_or_else(|| anyhow!("Empty Dc stream"))?;
    let first_dc = DcChar::from_short(first);
    ensure!(
        first_dc == DC_BEGIN_NUMBER,
        "Expected Dc 6 (Begin number), found Dc {first}"
    );

    let second = dcs
        .get(1)
        .copied()
        .ok_or_else(|| anyhow!("Unexpected end of stream after Dc 6"))?;
    let second_dc = DcChar::from_format(second);
    ensure!(
        second_dc == DC_FORMAT_199,
        "Expected format 199 indicator after Dc 6, found Dc {second}"
    );

    let mut idx = 2usize;
    let mut is_negative = false;

    if let Some(&third) = dcs.get(idx) {
        let third_dc = DcChar::from_short(third);
        if third_dc == DC_NEGATIVE {
            is_negative = true;
            idx = idx.saturating_add(1);
        } else if third_dc == DC_POSITIVE {
            idx = idx.saturating_add(1);
        }
    }

    let mut b64_str = String::new();
    let mut found_end = false;

    while idx < dcs.len() {
        let Some(&raw_dc) = dcs.get(idx) else { break };
        idx = idx.saturating_add(1);

        let dc = DcChar::from_short(raw_dc);
        if dc == DC_END_NUMBER {
            found_end = true;
            break;
        }

        let digit = dc_base64_char_to_digit(dc)?;
        let ch = digit_to_base64_char(digit)?;
        b64_str.push(ch);
    }

    ensure!(found_end, "Missing Dc 7 (End number) terminating Dc number");
    ensure!(
        !b64_str.is_empty(),
        "Dc number contains no digit characters"
    );

    let b64_base = Base::new(64)?;
    let nat = parse_natural(&b64_str, b64_base).with_context(|| {
        format!("Failed to parse Base64 number string '{b64_str}'")
    })?;

    let int_val = Integer::from_sign_and_abs(!is_negative, nat);

    Ok((int_val, idx))
}

/// Reads a single Dc number from a slice of Global Graph IDs (`DcList`),
/// returning the parsed [`Integer`] and the number of tokens consumed from the slice.
pub fn read_dc_number_global(gids: &[u128]) -> Result<(Integer, usize)> {
    let first = gids
        .first()
        .copied()
        .ok_or_else(|| anyhow!("Empty GID stream"))?;
    let first_dc = if let Ok(s) = u32::try_from(first) && s == 6 {
        DcChar::from_short(s)
    } else {
        DcChar::from_long(first)
    };
    ensure!(
        first_dc == DC_BEGIN_NUMBER,
        "Expected GID 1114118 / Dc 6 (Begin number), found {first}"
    );

    let second = gids
        .get(1)
        .copied()
        .ok_or_else(|| anyhow!("Unexpected end of stream after Dc 6"))?;
    let second_dc = DcChar::from_long(second);
    ensure!(
        second_dc == DC_FORMAT_199,
        "Expected format 199 indicator after Dc 6, found {second}"
    );

    let mut idx = 2usize;
    let mut is_negative = false;

    if let Some(&third) = gids.get(idx) {
        let third_dc = if let Ok(s) = u32::try_from(third) && (s == 10 || s == 11) {
            DcChar::from_short(s)
        } else {
            DcChar::from_long(third)
        };
        if third_dc == DC_NEGATIVE {
            is_negative = true;
            idx = idx.saturating_add(1);
        } else if third_dc == DC_POSITIVE {
            idx = idx.saturating_add(1);
        }
    }

    let mut b64_str = String::new();
    let mut found_end = false;

    while idx < gids.len() {
        let Some(&gid) = gids.get(idx) else { break };
        idx = idx.saturating_add(1);

        let dc = if let Ok(s) = u32::try_from(gid) && s == 7 {
            DcChar::from_short(s)
        } else {
            DcChar::from_long(gid)
        };
        if dc == DC_END_NUMBER {
            found_end = true;
            break;
        }

        let digit = if let Ok(digit) = dc_base64_char_to_digit(dc) {
            digit
        } else if let Ok(s) = u32::try_from(gid) {
            dc_base64_char_to_digit(DcChar::from_short(s))?
        } else {
            bail!("Global ID {gid} is not a valid Base64 encapsulation Dc");
        };
        let ch = digit_to_base64_char(digit)?;
        b64_str.push(ch);
    }

    ensure!(
        found_end,
        "Missing GID 1114119 / Dc 7 (End number) terminating Dc number"
    );
    ensure!(
        !b64_str.is_empty(),
        "Dc number contains no digit characters"
    );

    let b64_base = Base::new(64)?;
    let nat = parse_natural(&b64_str, b64_base).with_context(|| {
        format!("Failed to parse Base64 number string '{b64_str}'")
    })?;

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
    i128::try_from(&val)
        .map_err(|_| anyhow!("Parsed Dc number exceeds i128 range"))
}

/// Parses an entire sequence of Global Graph IDs (`DcList`) as an `i128`.
pub fn parse_dc_number_global_i128(gids: &[u128]) -> Result<i128> {
    let val = parse_dc_number_global(gids)?;
    i128::try_from(&val)
        .map_err(|_| anyhow!("Parsed Dc number exceeds i128 range"))
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
        base64_char_to_short_dc('$').unwrap_err();
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
        short_dc_to_base64_char(191).unwrap_err();
        short_dc_to_base64_char(126).unwrap_err();
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
        assert_eq!(
            gids_neg,
            vec![1_114_118, 2_228_423, 1_114_123, 1_114_281, 1_114_119]
        );
        assert_eq!(parse_dc_number_global_i128(&gids_neg).unwrap(), -42);

        // Read stream with trailing tokens
        let mut stream = gids_42.clone();
        stream.push(65); // Unicode 'A'
        let (val, consumed) = read_dc_number_global(&stream).unwrap();
        assert_eq!(val, Integer::from(42));
        assert_eq!(consumed, 4);
    }
}
