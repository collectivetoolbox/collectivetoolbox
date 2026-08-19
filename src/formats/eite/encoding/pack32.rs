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

//! 32-bit integer packing and bit-field manipulation for EITE.

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

/// Pack a (signed) 32-bit value (interpreted as Unicode/WTF-8 codepoint domain)
/// into a WTF-8 byte sequence representing a single logical codepoint.
///
/// Currently a thin wrapper around WTF-8 single-codepoint encoding.
///
/// (Original: pack32)
pub fn pack32(value: u32) -> Result<Vec<u8>> {
    encode_wtf8_single(value)
}

/// Unpack a single WTF-8 sequence into its codepoint (as u32).
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
/// sequence.
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
}
