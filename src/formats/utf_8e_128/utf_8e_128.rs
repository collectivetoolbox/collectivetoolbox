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

//! "UTF-8e-128"/DcUtf encoding and decoding (UTF-8 extended to 128-bit integers)

#[allow(
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities as utilities;
pub(crate) use ctb_utilities::*;

mod codec;
pub use codec::{
    decode_utf_8e_128, decode_utf_8e_128_buf, encode_utf_8e_128,
    encode_utf_8e_128_buf,
};

#[ipc_method]
/// Encode a u128 codepoint using the `utf_8e_128` format.
pub fn encode(codepoint: u128) -> Vec<u8> {
    encode_utf_8e_128(codepoint)
}

#[ipc_method]
/// Decode bytes using the `utf_8e_128` format.
///
/// Returns `(value, used_len)` on success, or `None` if the bytes are not a
/// valid prefix for the encoding.
#[expect(
    clippy::needless_pass_by_value,
    reason = "IPC method signature requires Vec<u8>"
)]
pub fn decode(bytes: Vec<u8>) -> Option<(u128, usize)> {
    decode_utf_8e_128(&bytes)
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
    fn test_standard_ascii() {
        let mut buf = [0u8; 24];
        for ch in [0x00u128, 0x41, 0x7F] {
            let n = encode_utf_8e_128_buf(&mut buf, ch);
            let (v, m) = decode_utf_8e_128_buf(&buf[..n]).unwrap();
            assert_eq!(v, ch);
            assert_eq!(n, m);
        }
    }

    #[crate::ctb_test]
    fn test_standard_multibyte() {
        let samples = [
            0x80u128, 0x7FF, 0x800, 0x1234, 0x20AC, 0xFFFF, 0x10000, 0x10FFFF,
        ];
        let mut buf = [0u8; 24];
        for cp in samples {
            let n = encode_utf_8e_128_buf(&mut buf, cp);
            let (v, m) = decode_utf_8e_128_buf(&buf[..n]).unwrap();
            assert_eq!(v, cp);
            assert_eq!(n, m);
        }
    }

    #[crate::ctb_test]
    fn test_extended_simple() {
        let mut buf = [0u8; 24];
        let cp = 0x10FFFFu128 + 1;
        let n = encode_utf_8e_128_buf(&mut buf, cp);
        assert!(n >= 3);
        assert_eq!(buf[0], 0xFF);
        let (v, m) = decode_utf_8e_128_buf(&buf[..n]).unwrap();
        assert_eq!(v, cp);
        assert_eq!(n, m);
    }

    #[crate::ctb_test]
    fn test_extended_large() {
        let mut buf = [0u8; 24];
        let cp = u128::MAX;
        let n = encode_utf_8e_128_buf(&mut buf, cp);
        assert_eq!(buf[0], 0xFF);
        let (v, m) = decode_utf_8e_128_buf(&buf[..n]).unwrap();
        assert_eq!(v, cp);
        assert_eq!(n, m);
    }

    #[crate::ctb_test]
    fn test_malformed() {
        assert!(decode_utf_8e_128_buf(&[]).is_none());
        assert!(decode_utf_8e_128_buf(&[0x80]).is_none()); // continuation as start
        assert!(decode_utf_8e_128_buf(&[0xFF]).is_none()); // incomplete extended
    }

    #[crate::ctb_test]
    fn test_overlaps_rejected() {
        // If value <= U+10FFFF must be encoded in standard form; constructing extended form should be rejected.
        // Manually craft extended for 0x41
        let bytes = vec![0xFF, 0x81, 0xC1]; // length=1, payload=0x01 -> value=1 (<= U+10FFFF)
        assert!(decode_utf_8e_128_buf(&bytes).is_none());

        // Construct an extended encoding for a value in standard range (should decode to None)
        // Manually: value = 0x10FFFF (should have used standard form)
        let mut bytes = Vec::new();
        bytes.push(0xFF);
        // Determine minimal groups for 0x10FFFF
        let val = 0x10FFFFu128;
        let bits = 128usize.saturating_sub(
            usize::try_from(val.leading_zeros())
                .expect("Failed to create usize"),
        );
        let l = bits.div_ceil(6);
        bytes.push(0x80 | u8::try_from(l).expect("Failed to create byte"));
        let mut groups = [0u8; 22];
        let mut tmp = val;
        for i in 0..l {
            groups[l.saturating_sub(1).saturating_sub(i)] =
                u8::try_from(tmp & 0x3F).expect("Failed to create byte");
            tmp >>= 6;
        }
        for i in 0..l {
            bytes.push(0x80 | groups[i]);
        }
        assert!(decode_utf_8e_128_buf(&bytes).is_none());
    }

    #[crate::ctb_test]
    fn test_encode_utf_8e_128_buf_basic() {
        let mut buf = [0u8; 24];
        // ASCII
        let n = encode_utf_8e_128_buf(&mut buf, 0x41);
        assert_eq!(&buf[..n], &[0x41]);
        // 2-byte
        let n = encode_utf_8e_128_buf(&mut buf, 0x80);
        assert_eq!(&buf[..n], &[0xC2, 0x80]);
        // 3-byte
        let n = encode_utf_8e_128_buf(&mut buf, 0x800);
        assert_eq!(&buf[..n], &[0xE0, 0xA0, 0x80]);
        // 4-byte
        let n = encode_utf_8e_128_buf(&mut buf, 0x10000);
        assert_eq!(&buf[..n], &[0xF0, 0x90, 0x80, 0x80]);
        // Extended
        encode_utf_8e_128_buf(&mut buf, 0x1_0000_0000);
        assert_eq!(buf[0], 0xFF);
    }

    #[crate::ctb_test]
    fn test_decode_utf_8e_128_buf_basic() {
        // ASCII
        let res = decode_utf_8e_128_buf(&[0x41]);
        assert_eq!(res, Some((0x41, 1)));
        // 2-byte
        let res = decode_utf_8e_128_buf(&[0xC2, 0x80]);
        assert_eq!(res, Some((0x80, 2)));
        // 3-byte
        let res = decode_utf_8e_128_buf(&[0xE0, 0xA0, 0x80]);
        assert_eq!(res, Some((0x800, 3)));
        // 4-byte
        let res = decode_utf_8e_128_buf(&[0xF0, 0x90, 0x80, 0x80]);
        assert_eq!(res, Some((0x10000, 4)));
    }

    #[crate::ctb_test]
    fn test_encode_decode_utf_8e_128() {
        // Roundtrip
        for &cp in &[
            0x41u128,
            0x80,
            0x800,
            0x10000,
            0x10FFFF,
            0x1_0000_0000,
            u128::MAX,
        ] {
            let encoded = encode_utf_8e_128(cp);
            let decoded = decode_utf_8e_128(&encoded).unwrap();
            assert_eq!(decoded.0, cp);
        }
    }

    #[crate::ctb_test]
    fn test_decode_utf_8e_128_replacement() {
        // Invalid input returns replacement character
        let res = decode_utf_8e_128(&[0xFF]);
        assert_eq!(res, Some((0xFFFD, 3)));
    }
}
