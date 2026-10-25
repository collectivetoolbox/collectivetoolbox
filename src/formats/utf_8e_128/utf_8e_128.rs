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
    MAX_DCTF_LEN, decode_dctf, decode_dctf_buf, decode_utf_8e_128,
    decode_utf_8e_128_buf, determine_dctf_error_len,
    determine_utf_8e_128_error_len, encode_dctf, encode_dctf_buf,
    encode_utf_8e_128, encode_utf_8e_128_buf, len_dctf, len_utf_8e_128,
};

#[ipc_method]
/// Encode a u128 codepoint using the `utf_8e_128` format.
pub fn encode(codepoint: u128) -> Vec<u8> {
    encode_utf_8e_128(codepoint)
}

#[ipc_method]
/// Lossily decode the first character from `bytes` using the legacy `utf_8e_128` (`DcUtf`) format.
///
/// Returns `(value, used_len)` (yielding `U+FFFD` and the maximal subpart length on
/// malformed sequences), or `None` if `bytes` is empty.
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
        // Invalid input returns replacement character consuming exact maximal subpart
        let res = decode_utf_8e_128(&[0xFF]);
        assert_eq!(res, Some((0xFFFD, 1)));
    }

    #[crate::ctb_test]
    fn test_dctf_standard_ascii() {
        let mut buf = [0u8; 24];
        for ch in [0x00u128, 0x41, 0x7F] {
            let n = encode_dctf_buf(&mut buf, ch);
            let (v, m) = decode_dctf_buf(&buf[..n]).unwrap();
            assert_eq!(v, ch);
            assert_eq!(n, m);
        }
    }

    #[crate::ctb_test]
    fn test_dctf_standard_multibyte() {
        let samples = [
            0x80u128, 0x7FF, 0x800, 0x1234, 0x20AC, 0xFFFF, 0x10000, 0x10FFFF,
        ];
        let mut buf = [0u8; 24];
        for cp in samples {
            let n = encode_dctf_buf(&mut buf, cp);
            let (v, m) = decode_dctf_buf(&buf[..n]).unwrap();
            assert_eq!(v, cp);
            assert_eq!(n, m);
        }
    }

    #[crate::ctb_test]
    fn test_dctf_extended_simple() {
        let mut buf = [0u8; 24];
        let cp = 0x10FFFFu128 + 1;
        let n = encode_dctf_buf(&mut buf, cp);
        assert_eq!(n, 4);
        assert_eq!(buf[0], 0xF4);
        let (v, m) = decode_dctf_buf(&buf[..n]).unwrap();
        assert_eq!(v, cp);
        assert_eq!(n, m);
    }

    #[crate::ctb_test]
    fn test_dctf_extended_large() {
        let mut buf = [0u8; 24];
        let cp = u128::MAX;
        let n = encode_dctf_buf(&mut buf, cp);
        assert_eq!(buf[0], 0xFF);
        let (v, m) = decode_dctf_buf(&buf[..n]).unwrap();
        assert_eq!(v, cp);
        assert_eq!(n, m);
    }

    #[crate::ctb_test]
    fn test_dctf_malformed() {
        assert!(decode_dctf_buf(&[]).is_none());
        assert!(decode_dctf_buf(&[0x80]).is_none()); // continuation as start
        assert!(decode_dctf_buf(&[0xFF]).is_none()); // incomplete extended
    }

    #[crate::ctb_test]
    fn test_dctf_overlaps_rejected() {
        // If value <= U+10FFFF must be encoded in standard form; constructing extended form should be rejected.
        // Manually craft extended for 0x41
        let bytes = vec![0xFF, 0x81, 0xC1]; // length=1, payload=0x01 -> value=1 (<= U+10FFFF)
        assert!(decode_dctf_buf(&bytes).is_none());

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
        assert!(decode_dctf_buf(&bytes).is_none());
    }

    #[crate::ctb_test]
    fn test_dctf_encode_dctf_buf_basic() {
        let mut buf = [0u8; 24];
        // ASCII
        let n = encode_dctf_buf(&mut buf, 0x41);
        assert_eq!(&buf[..n], &[0x41]);
        // 2-byte
        let n = encode_dctf_buf(&mut buf, 0x80);
        assert_eq!(&buf[..n], &[0xC2, 0x80]);
        // 3-byte
        let n = encode_dctf_buf(&mut buf, 0x800);
        assert_eq!(&buf[..n], &[0xE0, 0xA0, 0x80]);
        // 4-byte
        let n = encode_dctf_buf(&mut buf, 0x10000);
        assert_eq!(&buf[..n], &[0xF0, 0x90, 0x80, 0x80]);
        // 7-byte (36-bit tier)
        let n = encode_dctf_buf(&mut buf, 0x1_0000_0000);
        assert_eq!(n, 7);
        assert_eq!(buf[0], 0xFE);
        // 8-byte (0xFF tier 0, >= 2^36)
        let n = encode_dctf_buf(&mut buf, 0x10_0000_0000);
        assert_eq!(n, 8);
        assert_eq!(buf[0], 0xFF);
    }

    #[crate::ctb_test]
    fn test_dctf_decode_dctf_buf_basic() {
        // ASCII
        let res = decode_dctf_buf(&[0x41]);
        assert_eq!(res, Some((0x41, 1)));
        // 2-byte
        let res = decode_dctf_buf(&[0xC2, 0x80]);
        assert_eq!(res, Some((0x80, 2)));
        // 3-byte
        let res = decode_dctf_buf(&[0xE0, 0xA0, 0x80]);
        assert_eq!(res, Some((0x800, 3)));
        // 4-byte
        let res = decode_dctf_buf(&[0xF0, 0x90, 0x80, 0x80]);
        assert_eq!(res, Some((0x10000, 4)));
    }

    #[crate::ctb_test]
    fn test_dctf_encode_decode_dctf() {
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
            let encoded = encode_dctf(cp);
            let decoded = decode_dctf(&encoded).unwrap();
            assert_eq!(decoded.0, cp);
        }
    }

    #[crate::ctb_test]
    fn test_dctf_decode_dctf_replacement() {
        // Invalid input returns replacement character consuming exact maximal subpart
        let res = decode_dctf(&[0xFF]);
        assert_eq!(res, Some((0xFFFD, 1)));
    }

    #[crate::ctb_test]
    fn test_dctf_all_tier_boundaries_and_ordering() {
        // Every tier boundary from 1B through 23B (k = 0..=15), plus surrogates
        let mut boundaries: Vec<(u128, usize)> = vec![
            (0x00, 1),
            (0x7F, 1),
            (0x80, 2),
            (0x7FF, 2),
            (0x800, 3),
            (0xD7FF, 3),
            (0xD800, 3),
            (0xDFFF, 3),
            (0xE000, 3),
            (0xFFFF, 3),
            (0x1_0000, 4),
            (0x10_FFFF, 4),
            (0x11_0000, 4),
            (0x11_00CB, 4), // Dc 203
            (0x1F_FFFF, 4),
            (0x20_0000, 5),
            (0x22_0000, 5), // Format Dc start
            (0x3FF_FFFF, 5),
            (0x400_0000, 6),
            (0x7FFF_FFFF, 6),
            (0x8000_0000, 7),
            (0x2_0000_0000, 7), // System ID start
            (0xF_FFFF_FFFF, 7),
        ];
        for k in 0..=15usize {
            let min_val = if k == 0 {
                1u128 << 36
            } else {
                1u128 << (32 + 6 * k)
            };
            let max_bits = 38 + 6 * k;
            let max_val = if max_bits == 128 {
                u128::MAX
            } else {
                (1u128 << max_bits) - 1
            };
            boundaries.push((min_val, 8 + k));
            boundaries.push((max_val, 8 + k));
        }

        let mut prev_val: Option<u128> = None;
        let mut prev_bytes: Option<Vec<u8>> = None;
        for (val, expected_len) in boundaries {
            assert_eq!(len_dctf(val), expected_len, "len_dctf mismatch for {val:#x}");
            let encoded = encode_dctf(val);
            assert_eq!(
                encoded.len(),
                expected_len,
                "encode_dctf length mismatch for {val:#x}"
            );
            let (decoded, consumed) = decode_dctf_buf(&encoded)
                .unwrap_or_else(|| panic!("decode_dctf_buf failed for {val:#x}"));
            assert_eq!(decoded, val);
            assert_eq!(consumed, expected_len);

            // Unsigned byte-lexicographic order must strictly match numeric order
            if let (Some(pv), Some(pb)) = (prev_val, &prev_bytes) {
                if val > pv {
                    assert!(
                        pb.as_slice() < encoded.as_slice(),
                        "Lexicographic order violated between {pv:#x} ({pb:02X?}) and {val:#x} ({encoded:02X?})"
                    );
                }
            }
            prev_val = Some(val);
            prev_bytes = Some(encoded);
        }
    }

    #[crate::ctb_test]
    fn test_dctf_overlong_rejections() {
        // 2B overlong: 0x00 and 0x7F
        assert!(decode_dctf_buf(&[0xC0, 0x80]).is_none());
        assert!(decode_dctf_buf(&[0xC1, 0xBF]).is_none());
        // 3B overlong: 0x7FF
        assert!(decode_dctf_buf(&[0xE0, 0x9F, 0xBF]).is_none());
        // 4B overlong: 0xFFFF
        assert!(decode_dctf_buf(&[0xF0, 0x8F, 0xBF, 0xBF]).is_none());
        // 5B overlong: 0x1F_FFFF
        assert!(decode_dctf_buf(&[0xF8, 0x87, 0xBF, 0xBF, 0xBF]).is_none());
        // 6B overlong: 0x3FF_FFFF
        assert!(decode_dctf_buf(&[0xFC, 0x83, 0xBF, 0xBF, 0xBF, 0xBF]).is_none());
        // 7B overlong: 0x7FFF_FFFF
        assert!(decode_dctf_buf(&[0xFE, 0x81, 0xBF, 0xBF, 0xBF, 0xBF, 0xBF]).is_none());
        // 8B (k=0) overlong: 2^36 - 1 (0xFF, 0x80, 0xBF x 6)
        assert!(decode_dctf_buf(&[0xFF, 0x80, 0xBF, 0xBF, 0xBF, 0xBF, 0xBF, 0xBF]).is_none());
        // 8B (k=0) min valid: 2^36 (0xFF, 0x81, 0x80 x 6)
        assert_eq!(
            decode_dctf_buf(&[0xFF, 0x81, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80]),
            Some((1u128 << 36, 8))
        );
        // k = 1..=15 overlong (2^(32+6k) - 1) vs min valid (2^(32+6k))
        for k in 1..=15usize {
            let total_len = 8 + k;
            let k_u8 = u8::try_from(k).unwrap();
            let mut overlong = vec![0xFF, 0x80 | (k_u8 << 2), 0x83];
            overlong.resize(total_len, 0xBF);
            assert!(
                decode_dctf_buf(&overlong).is_none(),
                "Expected overlong rejection at k={k}"
            );

            let mut min_valid = vec![0xFF, 0x80 | (k_u8 << 2), 0x84];
            min_valid.resize(total_len, 0x80);
            assert_eq!(
                decode_dctf_buf(&min_valid),
                Some((1u128 << (32 + 6 * k), total_len)),
                "Expected min_valid acceptance at k={k}"
            );
        }
    }

    #[crate::ctb_test]
    fn test_dctf_and_dcutf_maximal_subpart_recovery() {
        // Helper that decodes an entire byte slice lossily into Vec<(u128, usize)>
        fn lossy_dctf_stream(mut slice: &[u8]) -> Vec<(u128, usize)> {
            let mut out = Vec::new();
            while let Some((cp, len)) = decode_dctf(slice) {
                out.push((cp, len));
                slice = &slice[len..];
            }
            out
        }

        fn lossy_dcutf_stream(mut slice: &[u8]) -> Vec<(u128, usize)> {
            let mut out = Vec::new();
            while let Some((cp, len)) = decode_utf_8e_128(slice) {
                out.push((cp, len));
                slice = &slice[len..];
            }
            out
        }

        // 1. E1 80 41 -> (FFFD, 2), ('A', 1) (must not swallow ASCII 'A'!)
        assert_eq!(determine_dctf_error_len(&[0xE1, 0x80, 0x41]), Some(2));
        assert_eq!(determine_utf_8e_128_error_len(&[0xE1, 0x80, 0x41]), Some(2));
        assert_eq!(lossy_dctf_stream(&[0xE1, 0x80, 0x41]), vec![(0xFFFD, 2), (0x41, 1)]);
        assert_eq!(lossy_dcutf_stream(&[0xE1, 0x80, 0x41]), vec![(0xFFFD, 2), (0x41, 1)]);

        // 2. C0 80 -> two 1-byte errors
        assert_eq!(determine_dctf_error_len(&[0xC0, 0x80]), Some(1));
        assert_eq!(lossy_dctf_stream(&[0xC0, 0x80]), vec![(0xFFFD, 1), (0xFFFD, 1)]);
        assert_eq!(lossy_dcutf_stream(&[0xC0, 0x80]), vec![(0xFFFD, 1), (0xFFFD, 1)]);

        // 3. E0 80 80 -> three 1-byte errors
        assert_eq!(determine_dctf_error_len(&[0xE0, 0x80, 0x80]), Some(1));
        assert_eq!(
            lossy_dctf_stream(&[0xE0, 0x80, 0x80]),
            vec![(0xFFFD, 1), (0xFFFD, 1), (0xFFFD, 1)]
        );
        assert_eq!(
            lossy_dcutf_stream(&[0xE0, 0x80, 0x80]),
            vec![(0xFFFD, 1), (0xFFFD, 1), (0xFFFD, 1)]
        );

        // 4. F0 80 80 80 -> four 1-byte errors
        assert_eq!(determine_dctf_error_len(&[0xF0, 0x80, 0x80, 0x80]), Some(1));
        assert_eq!(
            lossy_dctf_stream(&[0xF0, 0x80, 0x80, 0x80]),
            vec![(0xFFFD, 1); 4]
        );

        // 5. F8 80 80 80 80 -> five 1-byte errors in DcTF (and in DcUtf since F8 is invalid start)
        assert_eq!(determine_dctf_error_len(&[0xF8, 0x80, 0x80, 0x80, 0x80]), Some(1));
        assert_eq!(
            lossy_dctf_stream(&[0xF8, 0x80, 0x80, 0x80, 0x80]),
            vec![(0xFFFD, 1); 5]
        );
        assert_eq!(
            lossy_dcutf_stream(&[0xF8, 0x80, 0x80, 0x80, 0x80]),
            vec![(0xFFFD, 1); 5]
        );

        // 6. FC 80 80 80 80 80 -> six 1-byte errors
        assert_eq!(
            lossy_dctf_stream(&[0xFC, 0x80, 0x80, 0x80, 0x80, 0x80]),
            vec![(0xFFFD, 1); 6]
        );

        // 7. FE 80 80 80 80 80 80 -> seven 1-byte errors
        assert_eq!(
            lossy_dctf_stream(&[0xFE, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80]),
            vec![(0xFFFD, 1); 7]
        );

        // 8. FF 80 80 80 80 80 80 80 -> eight 1-byte errors in DcTF (0xFF 0x80 cannot reach 2^36)
        assert_eq!(
            lossy_dctf_stream(&[0xFF, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80]),
            vec![(0xFFFD, 1); 8]
        );

        // 9. FF 84 80 80 80 80 80 80 80 -> in DcTF (k=1, bb=00), [0xFF, 0x84] is a valid 2-byte
        // prefix, but byte 2 (0x80 < 0x84) cannot continue it -> 1 two-byte error + 7 one-byte errors!
        assert_eq!(
            determine_dctf_error_len(&[0xFF, 0x84, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80]),
            Some(2)
        );
        assert_eq!(
            lossy_dctf_stream(&[0xFF, 0x84, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80]),
            vec![
                (0xFFFD, 2),
                (0xFFFD, 1),
                (0xFFFD, 1),
                (0xFFFD, 1),
                (0xFFFD, 1),
                (0xFFFD, 1),
                (0xFFFD, 1),
                (0xFFFD, 1),
            ]
        );

        // 10. Chunk-boundary vs actual-EOF truncation:
        // [0xE2, 0x82] is an incomplete 2-byte prefix of U+20AC:
        assert_eq!(determine_dctf_error_len(&[0xE2, 0x82]), None);
        assert_eq!(determine_utf_8e_128_error_len(&[0xE2, 0x82]), None);
        // At actual EOF (decode_dctf / decode_utf_8e_128), it consumes all 2 bytes as one U+FFFD:
        assert_eq!(decode_dctf(&[0xE2, 0x82]), Some((0xFFFD, 2)));
        assert_eq!(decode_utf_8e_128(&[0xE2, 0x82]), Some((0xFFFD, 2)));

        // 11. Malformed extended (0xFF) sequences followed by ASCII 'A' (0x41):
        // Must preserve 'A' and never consume it into the replacement span.
        // DcTF: 0xFF 0x81 (k=0, expects 8 bytes) truncated after 3 bytes by 'A':
        assert_eq!(
            determine_dctf_error_len(&[0xFF, 0x81, 0x80, 0x41, 0x80, 0x80, 0x80, 0x80]),
            Some(3)
        );
        assert_eq!(
            lossy_dctf_stream(&[0xFF, 0x81, 0x80, 0x41]),
            vec![(0xFFFD, 3), (0x41, 1)]
        );
        // DcUtf: 0xFF 0x84 (L=4, expects 6 bytes) with canonical byte 2 (0x84) truncated by 'A':
        assert_eq!(
            determine_utf_8e_128_error_len(&[0xFF, 0x84, 0x84, 0x41, 0x80, 0x80]),
            Some(3)
        );
        assert_eq!(
            lossy_dcutf_stream(&[0xFF, 0x84, 0x84, 0x41]),
            vec![(0xFFFD, 3), (0x41, 1)]
        );

        // 12. Overlapping byte sequence FF 87 81 80 80 80 80 80 80:
        // Valid in both DcUtf (2^36 = 68,719,476,736) and DcTF (13,262,859,010,048).
        let overlap = [0xFF, 0x87, 0x81, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80];
        assert_eq!(decode_utf_8e_128_buf(&overlap), Some((1u128 << 36, 9)));
        assert_eq!(decode_dctf_buf(&overlap), Some((13_262_859_010_048u128, 9)));
    }
}
