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

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::{Result, anyhow};

/// Equivalent to JS internal bitwise mask (force into 0..=255).
#[expect(
    clippy::expect_used,
    reason = "v & 0xFF is <= 255 so it is guaranteed to fit in u8"
)]
fn mask_byte(v: i32) -> u8 {
    u8::try_from(v & 0xFF).expect("v & 0xFF is <= 255")
}

/// Bitwise AND (8-bit) – direct translation of `bitAnd8`.
pub fn bit_and_8(a: u8, b: u8) -> u8 {
    mask_byte(i32::from(a & b))
}

/// Bitwise NOT (8-bit) – direct translation of `bitNot8`.
pub fn bit_not_8(a: u8) -> u8 {
    mask_byte(i32::from(!a))
}

/// Bitwise left shift (8-bit) – bounds restricted to 0..=8 like original.
pub fn bit_lshift_8(a: u8, places: u8) -> u8 {
    assert!(places <= 8);
    let shifted = u32::from(a) << places;
    // Reason for fallback: shifted max value is 65280 which fits in i32::MAX, so try_from conversion never fails.
    mask_byte(i32::try_from(shifted).unwrap_or(0))
}

/// Bitwise logical right shift (8-bit).
pub fn bit_rshift_8(a: u8, places: u8) -> u8 {
    assert!(places <= 8);
    let shifted = u32::from(a) >> places;
    // Reason for fallback: shifted max value is 255 which fits in i32::MAX, so try_from conversion never fails.
    mask_byte(i32::try_from(shifted).unwrap_or(0))
}

/// Return least significant byte from a 32-bit int (matches original intent).
pub fn least_significant_byte(v: i32) -> u8 {
    mask_byte(v)
}

/// Convert a single byte to an 8-length vector of bits (MSB first).
///
/// Original code padded leading zeros until length 8.
pub fn byte_to_int_bit_array(byte: u8) -> Vec<u8> {
    let mut bits = Vec::with_capacity(8);
    for i in (0..8).rev() {
        bits.push((byte >> i) & 1);
    }
    bits
}

/// Convert exactly 8 bits (0/1) into a single byte.
///
/// Returns error if:
/// - `bits.len()` != 8
/// - any bit is not 0 or 1
pub fn byte_from_int_bit_array(bits: &[u8]) -> Result<u8> {
    if bits.len() != 8 {
        return Err(anyhow!(
            "byte_from_int_bit_array: expected 8 bits, got {}",
            bits.len()
        ));
    }
    let mut val: u8 = 0;
    for &b in bits {
        if b > 1 {
            return Err(anyhow!(
                "byte_from_int_bit_array: invalid bit value {b}"
            ));
        }
        val = (val << 1) | b;
    }
    Ok(val)
}

/// Flatten a byte slice into a vector of 0/1 bits (MSB first per byte).
pub fn byte_array_to_int_bit_array(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len().saturating_mul(8));
    for &b in bytes {
        out.extend(byte_to_int_bit_array(b));
    }
    out
}

/// Convert a flat bit array (length multiple of 8) into bytes.
///
/// Ignores trailing bits < 8 (i.e., requires perfect multiple). If leftover bits exist,
/// returns error to mirror strictness of original logic.
///
/// Returns error if any bit is not 0 or 1.
pub fn byte_array_from_int_bit_array(bits: &[u8]) -> Result<Vec<u8>> {
    if !bits.len().is_multiple_of(8) {
        return Err(anyhow!(
            "byte_array_from_int_bit_array: bit length {} not divisible by 8",
            bits.len()
        ));
    }
    let mut out = Vec::with_capacity(bits.len() / 8);
    for chunk in bits.chunks(8) {
        out.push(byte_from_int_bit_array(chunk)?);
    }
    Ok(out)
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
    fn test_bitwise_ops() {
        assert_eq!(bit_and_8(0b1111_0000, 0b1010_1010), 0b1010_0000);
        assert_eq!(bit_not_8(0b0000_1111), 0b1111_0000);
        assert_eq!(bit_lshift_8(0b0000_1111, 2), 0b0011_1100);
        assert_eq!(bit_rshift_8(0b1111_0000, 4), 0b0000_1111);
    }

    #[crate::ctb_test]
    fn test_byte_to_bits_and_back() -> Result<()> {
        let cases = [
            (0x00u8, [0, 0, 0, 0, 0, 0, 0, 0]),
            (0xFFu8, [1, 1, 1, 1, 1, 1, 1, 1]),
            (0x5Au8, [0, 1, 0, 1, 1, 0, 1, 0]), // 0x5A = 01011010
            (0xC3u8, [1, 1, 0, 0, 0, 0, 1, 1]), // 0xC3 = 11000011
        ];
        for (byte, bits_expected) in cases.iter() {
            let bits = byte_to_int_bit_array(*byte);
            assert_eq!(bits.as_slice(), bits_expected);
            let round = byte_from_int_bit_array(&bits)?;
            assert_eq!(round, *byte);
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_bytes_to_bits_round_trip() -> Result<()> {
        let data = b"\x00\x01\x7F\x80\xFF";
        let bits = byte_array_to_int_bit_array(data);
        assert_eq!(bits.len(), data.len() * 8);
        let restored = byte_array_from_int_bit_array(&bits)?;
        assert_eq!(restored, data);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_byte_array_from_int_bit_array_invalid_length() {
        let bits: Vec<u8> = vec![0u8, 1, 0]; // length 3
        assert!(byte_array_from_int_bit_array(&bits).is_err());
    }
}
