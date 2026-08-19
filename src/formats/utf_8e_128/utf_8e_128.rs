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

//! "UTF-8e-128"/DcUtf encoding and decoding (UTF-8 extended to 128-bit integers)

#[expect(
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

// FIXME: Can this be simplified by leaning on Rust's native UTF-8 en/decoding?

/// Encodes a Unicode scalar value or an extended (> U+10FFFF) 128‑bit integer
/// using the UTF‑8e‑128 scheme.
///
/// For values <= `0x10_FFFF` this produces standard UTF‑8 (1–4 bytes).
/// For larger values it emits:
///   0:  0xFF
///   1:  10LLLLLL   (1 <= L <= 22) number of payload continuation bytes
///   2+: L payload continuation bytes 10bbbbbb ... (big‑endian 6‑bit groups)
/// Returns the number of bytes written.
///
/// Panics if the provided buffer is too small (needs up to 24 bytes).
#[expect(
    clippy::expect_used,
    reason = "Integer casts and array indexing are provably infallible due to preceding range checks and fixed array bounds"
)]
pub fn encode_utf_8e_128_buf(buf: &mut [u8], codepoint: u128) -> usize {
    // Standard UTF-8 path for values within Unicode range

    // (Optional) Reject surrogate range if you only want Unicode scalar values.
    // if (0xD800..=0xDFFF).contains(&cp) {
    //     // An alternative here might be to assign them higher Dcs, or
    //     // since this encoding can hold 132 bits, to stuff them into that
    //     // unused space
    //     panic!("Cannot encode surrogate as scalar");
    // }
    if codepoint <= 0x10FFFF {
        let cp = u32::try_from(codepoint).expect("codepoint <= 0x10FFFF fits in u32");
        if cp <= 0x7F {
            if let Some(b) = buf.get_mut(0) {
                *b = u8::try_from(cp).expect("cp <= 0x7F fits in u8");
            }
            return 1;
        } else if cp <= 0x7FF {
            if buf.len() >= 2 {
                if let Some(b) = buf.get_mut(0) {
                    *b = 0xC0 | u8::try_from(cp >> 6).expect("cp >> 6 for cp <= 0x7FF is <= 31");
                }
                if let Some(b) = buf.get_mut(1) {
                    *b = 0x80 | u8::try_from(cp & 0x3F).expect("cp & 0x3F is <= 63");
                }
            }
            return 2;
        } else if cp <= 0xFFFF {
            if buf.len() >= 3 {
                if let Some(b) = buf.get_mut(0) {
                    *b = 0xE0 | u8::try_from(cp >> 12).expect("cp >> 12 for cp <= 0xFFFF is <= 15");
                }
                if let Some(b) = buf.get_mut(1) {
                    *b = 0x80 | u8::try_from((cp >> 6) & 0x3F).expect("shifted mask is <= 63");
                }
                if let Some(b) = buf.get_mut(2) {
                    *b = 0x80 | u8::try_from(cp & 0x3F).expect("cp & 0x3F is <= 63");
                }
            }
            return 3;
        }
        if buf.len() >= 4 {
            if let Some(b) = buf.get_mut(0) {
                *b = 0xF0 | u8::try_from(cp >> 18).expect("cp >> 18 for cp <= 0x10FFFF is <= 4");
            }
            if let Some(b) = buf.get_mut(1) {
                *b = 0x80 | u8::try_from((cp >> 12) & 0x3F).expect("shifted mask is <= 63");
            }
            if let Some(b) = buf.get_mut(2) {
                *b = 0x80 | u8::try_from((cp >> 6) & 0x3F).expect("shifted mask is <= 63");
            }
            if let Some(b) = buf.get_mut(3) {
                *b = 0x80 | u8::try_from(cp & 0x3F).expect("cp & 0x3F is <= 63");
            }
        }
        return 4;
    }

    // Extended form
    // Determine bit length
    let leading_zeros = usize::try_from(codepoint.leading_zeros()).expect("u32 leading_zeros (0..=128) fits in usize");
    let bits = 128usize.saturating_sub(leading_zeros);
    let mut l = bits.div_ceil(6); // minimal number of 6-bit groups
    if l == 0 {
        l = 1;
    }
    assert!(l <= 22, "Value requires more than 132 bits?");

    assert!(
        buf.len() >= 2usize.saturating_add(l),
        "Buffer too small for extended encoding"
    );

    // Extract groups big-endian: groups[0] is first (most significant) group
    let mut groups = [0u8; 22];
    {
        let mut tmp = codepoint;
        for i in 0..l {
            let idx = l.saturating_sub(1).saturating_sub(i);
            if let Some(g) = groups.get_mut(idx) {
                *g = u8::try_from(tmp & 0x3F).expect("tmp & 0x3F is <= 63");
            }
            tmp >>= 6;
        }
        debug_assert!(tmp == 0);
    }

    // Canonical rule: first payload group must be non-zero (value > 0)
    debug_assert!(groups.first().copied().expect("groups array has length 22") != 0);

    if let Some(b) = buf.get_mut(0) {
        *b = 0xFF;
    }
    if let Some(b) = buf.get_mut(1) {
        *b = 0x80 | u8::try_from(l).expect("l <= 22 fits in u8");
    }

    for i in 0..l {
        let dest_idx = 2usize.saturating_add(i);
        if let (Some(b), Some(&g)) = (buf.get_mut(dest_idx), groups.get(i)) {
            *b = 0x80 | g;
        }
    }

    // Additional canonical check for 128-bit max if l == 22:
    // top 4 bits of first payload group must be zero (they are the unused padding bits).
    if l == 22 {
        debug_assert!(
            (groups.first().copied().expect("groups array has length 22") & 0x3C) == 0,
            "Non-zero padding bits in 22-byte encoding"
        );
    }

    2usize.saturating_add(l)
}

/// Decodes one UTF‑8 / UTF‑8e‑128 codepoint from the provided byte slice.
/// On success returns Some((value, `length_consumed`)), else None.
/// Enforces canonical (no overlong) encodings for both standard and extended forms.
#[expect(
    clippy::expect_used,
    clippy::unwrap_in_result,
    reason = "Fixed-size 22-element groups array guarantees first element presence and buf slice fits encoded length"
)]
pub fn decode_utf_8e_128_buf(bytes: &[u8]) -> Option<(u128, usize)> {
    let first = *bytes.first()?;
    if first == 0xFF {
        // Extended form
        let h = *bytes.get(1)?;
        if (h & 0xC0) != 0x80 {
            return None;
        }
        let l = usize::from(h & 0x3F);
        if l == 0 || l > 22 {
            return None;
        }
        if bytes.len() < l.saturating_add(2) {
            return None;
        }

        // Gather groups
        let mut groups = [0u8; 22];
        for i in 0..l {
            let b = *bytes.get(i.saturating_add(2))?;
            if (b & 0xC0) != 0x80 {
                return None;
            }
            if let Some(g) = groups.get_mut(i) {
                *g = b & 0x3F;
            }
        }

        // Canonical: first group not zero
        if groups.first().copied().expect("groups array has length 22") == 0 {
            return None;
        }

        // If l == 22, top 4 bits of first group (padding) must be zero.
        if l == 22 && (groups.first().copied().expect("groups array has length 22") & 0x3C) != 0 {
            return None;
        }

        // Reconstruct value pruning leading padding bits if total bits > 128
        let total_bits = l.saturating_mul(6);
        let extra = total_bits.saturating_sub(128); // 0..=4
        if extra > 4 {
            return None; // should not happen with l<=22 and u128 output
        }

        let g0 = groups.first().copied().expect("groups array has length 22");
        // Ensure the extra (padding) high bits are zero
        if extra > 0 && (g0 >> (6usize.saturating_sub(extra))) != 0 {
            return None;
        }

        let mut value: u128 = 0;
        if extra < 6 {
            // Take lower (6 - extra) bits of first group
            let first_payload_bits = g0
                & ((1u8 << (6usize.saturating_sub(extra))).saturating_sub(1));
            value = u128::from(first_payload_bits);
        }
        for i in 1..l {
            if let Some(&g) = groups.get(i) {
                value = (value << 6) | u128::from(g);
            }
        }

        // Must not overlap with standard range
        if value <= 0x10FFFF {
            return None;
        }

        return Some((value, l.saturating_add(2)));
    }

    // Standard UTF-8 decoding
    if first < 0x80 {
        return Some((u128::from(first), 1));
    }

    // Determine expected length and initial mask / prefix
    let (len, min_val, max_val_mask) = if (first & 0xE0) == 0xC0 {
        // 110xxxxx
        (2usize, 0x80u32, 0x1F)
    } else if (first & 0xF0) == 0xE0 {
        // 1110xxxx
        (3usize, 0x800u32, 0x0F)
    } else if (first & 0xF8) == 0xF0 {
        // 11110xxx
        (4usize, 0x10000u32, 0x07)
    } else {
        return None;
    };

    if bytes.len() < len {
        return None;
    }

    let mut val: u32 = u32::from(first & max_val_mask);
    for i in 1..len {
        let b = *bytes.get(i)?;
        if (b & 0xC0) != 0x80 {
            return None;
        }
        val = (val << 6) | u32::from(b & 0x3F);
    }

    // Overlong check
    if val < min_val {
        return None;
    }

    // Unicode max (U+10FFFF)
    if val > 0x10FFFF {
        return None;
    }

    // Optional: reject surrogate range for scalar value canonicality.
    // if (0xD800..=0xDFFF).contains(&val) {
    //     return None;
    // }

    Some((u128::from(val), len))
}

/// Generalized UTF-8 encoding for u128.
/// Returns a `Vec<u8>` containing the encoded bytes.
#[expect(
    clippy::expect_used,
    reason = "encode_utf_8e_128_buf returns length <= 24 which fits in 24-byte buf"
)]
pub fn encode_utf_8e_128(codepoint: u128) -> Vec<u8> {
    let mut buf = [0u8; 24];
    let encoded_len = encode_utf_8e_128_buf(&mut buf, codepoint);
    buf.get(..encoded_len).expect("encoded_len <= 24 fits in buf").to_vec()
}

/// Decodes one generalized UTF-8 codepoint from bytes.
/// Returns Some((value, `length_consumed`)), or the replacement character on error.
pub fn decode_utf_8e_128(bytes: &[u8]) -> Option<(u128, usize)> {
    if bytes.is_empty() {
        return None;
    }

    let mut buf = [0u8; 24];
    let used_len = bytes.len().min(24);
    if let (Some(dst), Some(src)) =
        (buf.get_mut(..used_len), bytes.get(..used_len))
    {
        dst.copy_from_slice(src);
    }

    if let Some(x) = decode_utf_8e_128_buf(&buf) {
        Some(x)
    } else {
        // Overwrite buffer with replacement character [0xEF, 0xBF, 0xBD]
        if let Some(b) = buf.get_mut(0) {
            *b = 0xEF;
        }
        if let Some(b) = buf.get_mut(1) {
            *b = 0xBF;
        }
        if let Some(b) = buf.get_mut(2) {
            *b = 0xBD;
        }
        if let Some(slice) = buf.get_mut(3..) {
            for b in slice {
                *b = 0;
            }
        }
        // Return replacement character and length
        Some((0xFFFD, 3))
    }
}

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
