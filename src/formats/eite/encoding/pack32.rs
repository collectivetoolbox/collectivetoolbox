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

//! 32-bit integer packing for EITE.
//!
//! Despite the name suggesting arbitrary 32-bit integers, `pack32` encodes
//! integers as single WTF-8 codepoints. Consequently, the supported range of
//! integers is restricted to the Unicode/WTF-8 codepoint range:
//! `0x0000_0000..=0x0010_FFFF` (0 through 1,114,111).
//!
//! Values in this range, including surrogate codepoints (`0xD800..=0xDFFF`),
//! are valid and round-trip successfully through `unpack32`.
//!
//! Any value exceeding `0x0010_FFFF` (i.e. `0x0011_0000..=u32::MAX`) will fail
//! with an error when passed to `pack32`.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::{Result, anyhow, bail};

use ctb_formats_wtf8::{
    decode_wtf8_single, encode_wtf8_single, is_unpackable_wtf8,
};

/// Minimum integer value supported by [`pack32`] (0).
pub const PACK32_MIN: u32 = 0;

/// Maximum integer value supported by [`pack32`] (`0x0010_FFFF`, 1,114,111).
///
/// Despite `pack32` accepting a `u32`, encoding uses single WTF-8 codepoints,
/// which are bounded by the Unicode codespace limit `0x10FFFF`. Values greater
/// than this will fail to encode.
pub const PACK32_MAX: u32 = 0x0010_FFFF;

/// Pack a 32-bit integer into a WTF-8 byte sequence representing a single
/// logical codepoint.
///
/// # Supported Range
///
/// Despite taking a `u32`, only values in the Unicode / WTF-8 codepoint
/// range `0x0000_0000..=0x0010_FFFF` (0 through 1,114,111, or
/// [`PACK32_MIN`]..=[`PACK32_MAX`]) are supported. This includes surrogate
/// codepoints `0xD800..=0xDFFF` supported by WTF-8.
///
/// Values strictly greater than `0x0010_FFFF` (up to `u32::MAX`) return an
/// error.
///
/// Currently a thin wrapper around WTF-8 single-codepoint encoding.
///
/// (Original: pack32)
pub fn pack32(value: u32) -> Result<Vec<u8>> {
    encode_wtf8_single(value)
}

/// Unpack a single WTF-8 sequence into its codepoint (as u32).
///
/// Decoded values are guaranteed to be within `0x0000_0000..=0x0010_FFFF`
/// ([`PACK32_MIN`]..=[`PACK32_MAX`]).
///
/// Errors if the input is not a single valid WTF-8 codepoint sequence
/// or if extra bytes remain after the first decoded codepoint.
///
/// (Original: unpack32)
pub fn unpack32(bytes: &[u8]) -> Result<u32> {
    let (cp, used) =
        decode_wtf8_single(bytes).map_err(|e| anyhow!("unpack32: {e}"))?;
    if used != bytes.len() {
        bail!(
            "unpack32: input contains extra bytes (decoded {} used {}, total {}, input {:?})",
            cp,
            used,
            bytes.len(),
            bytes
        );
    }
    Ok(cp)
}

/// Test whether the given byte slice is a valid single WTF-8 codepoint
/// sequence (representing a value in [`PACK32_MIN`]..=[`PACK32_MAX`]).
///
/// (Original: isPack32Char)
pub fn is_pack32_char(bytes: &[u8]) -> bool {
    is_unpackable_wtf8(bytes)
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
    fn test_pack32_roundtrip_basic() {
        // ASCII 'A'
        let bytes = pack32(0x41).unwrap();
        assert!(is_pack32_char(&bytes));
        let cp = unpack32(&bytes).unwrap();
        assert_eq!(cp, 0x41);

        // BMP codepoint
        let smile = 0x263A;
        let bytes2 = pack32(smile).unwrap();
        assert!(is_pack32_char(&bytes2));
        let cp2 = unpack32(&bytes2).unwrap();
        assert_eq!(cp2, smile);
    }

    #[crate::ctb_test]
    fn test_pack32_surrogate_wtf8() {
        // Surrogate (valid in WTF-8)
        let hi = 0xD83D;
        let bytes = pack32(hi).unwrap();
        assert!(is_pack32_char(&bytes));
        let round = unpack32(&bytes).unwrap();
        assert_eq!(round, hi);
    }

    #[crate::ctb_test]
    fn test_pack32_round_trip() {
        for &val in &[0, 10, 100, 1000, 10_000] {
            let packed = pack32(val).unwrap();
            let unpacked = unpack32(&packed).expect("unpack32 failed");
            assert_eq!(val, unpacked, "pack32/unpack32 mismatch for {}", val);
        }
    }

    #[crate::ctb_test]
    fn test_pack32_supported_range_boundaries() {
        // Test key boundary values within the supported range [0, 0x10FFFF].
        let boundary_values = [
            PACK32_MIN, // 0x000000 (1-byte start)
            0x7F,       // 1-byte end
            0x80,       // 2-byte start
            0x7FF,      // 2-byte end
            0x800,      // 3-byte start
            0xD7FF,     // Just before surrogate range
            0xD800,     // Lead surrogate start (valid in WTF-8)
            0xDBFF,     // Lead surrogate end
            0xDC00,     // Trail surrogate start
            0xDFFF,     // Trail surrogate end
            0xE000,     // Just after surrogate range
            0xFFFF,     // 3-byte end (BMP end)
            0x10000,    // 4-byte start
            0x100000,   // Supplementary plane
            PACK32_MAX, // 0x10FFFF (4-byte end / max supported)
        ];

        for &val in &boundary_values {
            let packed = pack32(val)
                .unwrap_or_else(|e| panic!("failed to pack supported value 0x{val:X}: {e}"));
            assert!(
                is_pack32_char(&packed),
                "is_pack32_char returned false for packed 0x{val:X}"
            );
            let unpacked = unpack32(&packed)
                .unwrap_or_else(|e| panic!("failed to unpack 0x{val:X}: {e}"));
            assert_eq!(
                val, unpacked,
                "pack32/unpack32 roundtrip mismatch for 0x{val:X}"
            );
        }
    }

    #[crate::ctb_test]
    fn test_pack32_unsupported_range_errors() {
        // Values strictly above PACK32_MAX (0x10FFFF) must fail to pack.
        let unsupported_values = [
            PACK32_MAX.saturating_add(1), // 0x110000
            0x110001,
            0x120000,
            0x1FFFFF,   // End of 4-byte UTF-8 space (historic)
            0x200000,   // 5-byte UTF-8 space (historic)
            0x03FFFFFF, // 5-byte UTF-8 end (historic)
            0x04000000, // 6-byte UTF-8 space (historic)
            0x7FFFFFFF, // UNICODE_HISTORIC_MAX / i32::MAX
            0x80000000, // i32 sign bit set
            0xDEADBEEF,
            u32::MAX, // 0xFFFFFFFF
        ];

        for &val in &unsupported_values {
            assert!(
                pack32(val).is_err(),
                "expected pack32(0x{val:X}) to fail, but it succeeded"
            );
        }
    }

    #[crate::ctb_test]
    fn test_unpack32_invalid_and_out_of_range() {
        // Empty slice
        assert!(unpack32(&[]).is_err());

        // Extra trailing bytes rejected by unpack32
        let packed_a = pack32(0x41).unwrap();
        assert!(unpack32(&packed_a).is_ok());
        let mut with_extra = packed_a;
        with_extra.push(0x42);
        assert!(unpack32(&with_extra).is_err());

        // Invalid WTF-8 bytes
        assert!(unpack32(&[0xFF]).is_err());
        assert!(!is_pack32_char(&[0xFF]));

        // UTF-8 byte sequence encoding codepoint 0x110000 (F4 90 80 80),
        // which exceeds PACK32_MAX. unpack32 and is_pack32_char must reject it.
        let out_of_range_bytes = [0xF4, 0x90, 0x80, 0x80];
        assert!(unpack32(&out_of_range_bytes).is_err());
        assert!(!is_pack32_char(&out_of_range_bytes));
    }
}
