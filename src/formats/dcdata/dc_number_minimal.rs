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

//! Minimal Document Character (`DcChar`) numeral encoding and Base64 encapsulation.
//!
//! Provides the foundational encoding of unsigned 128-bit integers into sequences
//! of short Document Characters (Dcs) using Format 199 and Base64 encapsulation Dcs
//! (short Dcs 127..=190 and 195).

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::{Result, anyhow, bail};

use crate::dc::{
    DC_BASE64_PADDING, DC_BASE64_START, DC_BEGIN_NUMBER, DC_END_NUMBER,
    DC_FORMAT_199,
};
use crate::dc_char::DcChar;

/// Converts a 6-bit Base64 numeral value (0..=63) or padding sentinel (64) into
/// its corresponding Base64 encapsulation `DcChar`.
///
/// Values `0..=63` map to short Dcs `127..=190`.
/// Value `64` maps to short Dc `195` (`DC_BASE64_PADDING`).
#[must_use]
pub fn dc_base64_digit_to_char(val: u8) -> DcChar {
    if val == 64 {
        DC_BASE64_PADDING
    } else {
        let offset = u32::from(val.min(63));
        // Reason for fallback: DC_BASE64_START is canonically Short Dc 127
        let start = DC_BASE64_START.to_short().unwrap_or(127);
        DcChar::from_short(start.saturating_add(offset))
    }
}

/// Converts a Base64 encapsulation `DcChar` into its 6-bit numeral value (0..=63)
/// or padding sentinel (64).
///
/// # Errors
/// Returns an error if `ch` is not a valid Base64 encapsulation character.
pub fn dc_base64_char_to_digit(ch: DcChar) -> Result<u8> {
    if ch == DC_BASE64_PADDING {
        return Ok(64);
    }
    let short_id = ch.to_short()?;
    // Reason for fallback: DC_BASE64_START is canonically Short Dc 127
    let start = DC_BASE64_START.to_short().unwrap_or(127);
    let end = start.saturating_add(63);
    if (start..=end).contains(&short_id) {
        let offset = short_id.saturating_sub(start);
        u8::try_from(offset)
            .map_err(|e| anyhow!("Failed to convert base64 digit to u8: {e}"))
    } else {
        bail!("DcChar({ch}) is not a Base64 encapsulation digit or padding")
    }
}

/// Returns `true` if `ch` is a Base64 encapsulation digit (127..=190) or padding (195).
#[must_use]
pub fn is_dc_base64_encapsulation_char(ch: DcChar) -> bool {
    if ch == DC_BASE64_PADDING {
        return true;
    }
    let Ok(short_id) = ch.to_short() else {
        return false;
    };
    // Reason for fallback: DC_BASE64_START is canonically Short Dc 127
    let start = DC_BASE64_START.to_short().unwrap_or(127);
    let end = start.saturating_add(63);
    (start..=end).contains(&short_id)
}

/// Encodes an unsigned 128-bit integer into a sequence of `DcChar`s.
///
/// Structure:
/// `[DC_BEGIN_NUMBER, DC_FORMAT_199, ...Base64 encapsulation digits..., DC_END_NUMBER]`
#[must_use]
pub fn u128_to_dc_number_chars(mut val: u128) -> Vec<DcChar> {
    let mut digits = Vec::new();
    if val == 0 {
        digits.push(DC_BASE64_START);
    } else {
        while val > 0 {
            // Reason for fallback: checked arithmetic remainder with non-zero divisor 64
            let rem = val.checked_rem(64).unwrap_or(0);
            // Reason for fallback: remainder mod 64 fits into u8 (0..=63)
            let rem_u8 = u8::try_from(rem).unwrap_or(0);
            digits.push(dc_base64_digit_to_char(rem_u8));
            // Reason for fallback: checked division with non-zero divisor 64
            val = val.checked_div(64).unwrap_or(0);
        }
        digits.reverse();
    }

    let mut result = Vec::with_capacity(digits.len().saturating_add(3));
    result.push(DC_BEGIN_NUMBER);
    result.push(DC_FORMAT_199);
    result.extend(digits);
    result.push(DC_END_NUMBER);
    result
}

/// Encodes an unsigned 128-bit integer into a sequence of short Document Character IDs (`u32`).
///
/// Structure:
/// `[6 (Begin number), 199 (Format 199), ...Base64 encapsulation digits (127..=190)..., 7 (End number)]`
#[must_use]
pub fn u128_to_dc_number_short(val: u128) -> Vec<u32> {
    let chars = u128_to_dc_number_chars(val);
    let mut out = Vec::with_capacity(chars.len());
    for ch in chars {
        if ch == DC_FORMAT_199 {
            // Reason for fallback: DC_FORMAT_199 canonically maps to format 199
            out.push(ch.to_format().unwrap_or(199));
        } else {
            // Reason for fallback: valid Base64 encapsulation chars and framing delimiters map to their short IDs
            out.push(ch.to_short().unwrap_or(0));
        }
    }
    out
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
    fn test_base64_digit_roundtrip() {
        for digit in 0..=63 {
            let ch = dc_base64_digit_to_char(digit);
            assert!(is_dc_base64_encapsulation_char(ch));
            assert_eq!(dc_base64_char_to_digit(ch).unwrap(), digit);
        }

        let pad = dc_base64_digit_to_char(64);
        assert_eq!(pad, DC_BASE64_PADDING);
        assert!(is_dc_base64_encapsulation_char(pad));
        assert_eq!(dc_base64_char_to_digit(pad).unwrap(), 64);
    }

    #[crate::ctb_test]
    fn test_u128_to_dc_number_short_zero() {
        let res = u128_to_dc_number_short(0);
        assert_eq!(res, vec![6, 199, 127, 7]);
    }

    #[crate::ctb_test]
    fn test_u128_to_dc_number_short_sixty_four() {
        // 64 in base 64 is digits 1 ('B' = 128) and 0 ('A' = 127)
        let res = u128_to_dc_number_short(64);
        assert_eq!(res, vec![6, 199, 128, 127, 7]);
    }
}
