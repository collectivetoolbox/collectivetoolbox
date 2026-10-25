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

//! Pure `UTF-8e-128` buffer and slice encoding/decoding routines.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

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
        let cp = u32::try_from(codepoint)
            .expect("codepoint <= 0x10FFFF fits in u32");
        if cp <= 0x7F {
            if let Some(b) = buf.get_mut(0) {
                *b = u8::try_from(cp).expect("cp <= 0x7F fits in u8");
            }
            return 1;
        } else if cp <= 0x7FF {
            if buf.len() >= 2 {
                if let Some(b) = buf.get_mut(0) {
                    *b = 0xC0
                        | u8::try_from(cp >> 6)
                            .expect("cp >> 6 for cp <= 0x7FF is <= 31");
                }
                if let Some(b) = buf.get_mut(1) {
                    *b = 0x80
                        | u8::try_from(cp & 0x3F).expect("cp & 0x3F is <= 63");
                }
            }
            return 2;
        } else if cp <= 0xFFFF {
            if buf.len() >= 3 {
                if let Some(b) = buf.get_mut(0) {
                    *b = 0xE0
                        | u8::try_from(cp >> 12)
                            .expect("cp >> 12 for cp <= 0xFFFF is <= 15");
                }
                if let Some(b) = buf.get_mut(1) {
                    *b = 0x80
                        | u8::try_from((cp >> 6) & 0x3F)
                            .expect("shifted mask is <= 63");
                }
                if let Some(b) = buf.get_mut(2) {
                    *b = 0x80
                        | u8::try_from(cp & 0x3F).expect("cp & 0x3F is <= 63");
                }
            }
            return 3;
        }
        if buf.len() >= 4 {
            if let Some(b) = buf.get_mut(0) {
                *b = 0xF0
                    | u8::try_from(cp >> 18)
                        .expect("cp >> 18 for cp <= 0x10FFFF is <= 4");
            }
            if let Some(b) = buf.get_mut(1) {
                *b = 0x80
                    | u8::try_from((cp >> 12) & 0x3F)
                        .expect("shifted mask is <= 63");
            }
            if let Some(b) = buf.get_mut(2) {
                *b = 0x80
                    | u8::try_from((cp >> 6) & 0x3F)
                        .expect("shifted mask is <= 63");
            }
            if let Some(b) = buf.get_mut(3) {
                *b =
                    0x80 | u8::try_from(cp & 0x3F).expect("cp & 0x3F is <= 63");
            }
        }
        return 4;
    }

    // Extended form
    // Determine bit length
    let leading_zeros = usize::try_from(codepoint.leading_zeros())
        .expect("u32 leading_zeros (0..=128) fits in usize");
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
    debug_assert!(
        groups.first().copied().expect("groups array has length 22") != 0
    );

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
            (groups.first().copied().expect("groups array has length 22")
                & 0x3C)
                == 0,
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
        if l == 22
            && (groups.first().copied().expect("groups array has length 22")
                & 0x3C)
                != 0
        {
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
    buf.get(..encoded_len)
        .expect("encoded_len <= 24 fits in buf")
        .to_vec()
}

/// Decodes one generalized UTF-8 codepoint from bytes.
/// Returns Some((value, `length_consumed`)), or the replacement character on error.
#[must_use]
pub fn decode_utf_8e_128(bytes: &[u8]) -> Option<(u128, usize)> {
    if bytes.is_empty() {
        return None;
    }

    if let Some(x) = decode_utf_8e_128_buf(bytes) {
        Some(x)
    } else {
        // Reason for fallback: determine_utf_8e_128_error_len returns None when
        // `bytes` is a truncated valid canonical prefix at a chunk boundary; in
        // one-shot lossy decoding (`decode_utf_8e_128`), the end of `bytes` is
        // actual EOF, so the entire truncated prefix is consumed as one U+FFFD.
        let err_len = determine_utf_8e_128_error_len(bytes).unwrap_or(bytes.len());
        Some((0xFFFD, err_len))
    }
}

/// Returns the encoded byte length of `codepoint` in legacy `UTF-8e-128` (`DcUtf`, `f89`).
#[must_use]
#[expect(
    clippy::expect_used,
    reason = "u32 leading_zeros (0..=128) always fits in usize"
)]
pub fn len_utf_8e_128(codepoint: u128) -> usize {
    if codepoint <= 0x7F {
        1
    } else if codepoint <= 0x7FF {
        2
    } else if codepoint <= 0xFFFF {
        3
    } else if codepoint <= 0x10_FFFF {
        4
    } else {
        let leading_zeros = usize::try_from(codepoint.leading_zeros())
            .expect("u32 leading_zeros fits in usize");
        let bits = 128usize.saturating_sub(leading_zeros);
        let l = bits.div_ceil(6).max(1);
        2usize.saturating_add(l)
    }
}

/// Computes the maximal-subpart error length for an invalid or incomplete legacy
/// `UTF-8e-128` (`DcUtf`, `f89`) byte slice.
///
/// Returns `Some(n)` (`n >= 1`) when `slice[..n]` is the longest initial subsequence
/// that can prefix a canonical `DcUtf` sequence (or `Some(1)` if `slice[0]` cannot
/// start any canonical sequence), or `None` when the entire `slice` is a valid
/// incomplete prefix of a canonical `DcUtf` sequence (truncated at the end of `slice`).
#[must_use]
pub fn determine_utf_8e_128_error_len(slice: &[u8]) -> Option<usize> {
    let first = *slice.first()?;
    if first < 0x80 {
        return Some(1);
    }
    if first < 0xC2 || (0xF5..=0xFE).contains(&first) {
        return Some(1);
    }
    if first == 0xFF {
        let Some(&b1) = slice.get(1) else {
            return None;
        };
        if !(0x84..=0x96).contains(&b1) {
            return Some(1);
        }
        let l = usize::from(b1 & 0x3F);
        let expected_len = 2usize.saturating_add(l);
        for i in 2..expected_len {
            let Some(&b) = slice.get(i) else {
                return None;
            };
            if (b & 0xC0) != 0x80 {
                return Some(i);
            }
            if i == 2 {
                if l == 4 && b < 0x84 {
                    return Some(2);
                }
                if l == 22 && !(0x81..=0x83).contains(&b) {
                    return Some(2);
                }
                if (5..=21).contains(&l) && b < 0x81 {
                    return Some(2);
                }
            } else if i == 3 && l == 4 && slice.get(2).copied() == Some(0x84) && b < 0x90 {
                return Some(3);
            }
        }
        return Some(expected_len);
    }

    let (expected_len, min_b1, max_b1) = if first <= 0xDF {
        (2usize, 0x80u8, 0xBFu8)
    } else if first == 0xE0 {
        (3usize, 0xA0u8, 0xBFu8)
    } else if first <= 0xEF {
        (3usize, 0x80u8, 0xBFu8)
    } else if first == 0xF0 {
        (4usize, 0x90u8, 0xBFu8)
    } else if first <= 0xF3 {
        (4usize, 0x80u8, 0xBFu8)
    } else {
        // first == 0xF4
        (4usize, 0x80u8, 0x8Fu8)
    };

    for i in 1..expected_len {
        let Some(&b) = slice.get(i) else {
            return None;
        };
        let (lo, hi) = if i == 1 {
            (min_b1, max_b1)
        } else {
            (0x80u8, 0xBFu8)
        };
        if b < lo || b > hi {
            return Some(i);
        }
    }
    Some(expected_len)
}

/// Maximum byte length of a `DcTF` (`.dctf`, `f641`) character sequence (23 bytes for 128-bit values).
pub const MAX_DCTF_LEN: usize = 23;

/// Returns the encoded byte length of `codepoint` in `DcTF` (`.dctf`, `f641`).
#[must_use]
#[expect(
    clippy::expect_used,
    reason = "u32 leading_zeros (0..=128) always fits in usize"
)]
pub fn len_dctf(codepoint: u128) -> usize {
    if codepoint <= 0x7F {
        1
    } else if codepoint <= 0x7FF {
        2
    } else if codepoint <= 0xFFFF {
        3
    } else if codepoint <= 0x1F_FFFF {
        4
    } else if codepoint <= 0x3FF_FFFF {
        5
    } else if codepoint <= 0x7FFF_FFFF {
        6
    } else if codepoint <= 0xF_FFFF_FFFF {
        7
    } else {
        let leading_zeros = usize::try_from(codepoint.leading_zeros())
            .expect("u32 leading_zeros fits in usize");
        let bits = 128usize.saturating_sub(leading_zeros);
        let k = bits.saturating_sub(38).div_ceil(6);
        8usize.saturating_add(k)
    }
}

/// Encodes a 128-bit Document Character / integer value using the `DcTF` (`.dctf`, `f641`) scheme.
///
/// Tier layout:
/// - 1B (`0x00..=0x7F`): `0..=0x7F` (7 bits)
/// - 2B (`0xC2..=0xDF` + 1 cont.): `0x80..=0x7FF` (11 bits)
/// - 3B (`0xE0..=0xEF` + 2 cont.): `0x800..=0xFFFF` (16 bits, including surrogates `0xD800..=0xDFFF`)
/// - 4B (`0xF0..=0xF7` + 3 cont.): `0x1_0000..=0x1F_FFFF` (21 bits)
/// - 5B (`0xF8..=0xFB` + 4 cont.): `0x20_0000..=0x3FF_FFFF` (26 bits)
/// - 6B (`0xFC..=0xFD` + 5 cont.): `0x400_0000..=0x7FFF_FFFF` (31 bits)
/// - 7B (`0xFE` + 6 cont.): `0x8000_0000..=0xF_FFFF_FFFF` (36 bits)
/// - `8 + k` B (`0 <= k <= 15`): `0xFF`, `10_kkkk_bb`, followed by `k + 6` continuation bytes (`38 + 6k` bits)
///
/// Returns the number of bytes written (1..=23).
/// Panics if `buf` is smaller than the required encoded length.
#[expect(
    clippy::expect_used,
    reason = "Bitmask and range-checked shifts guarantee infallible u8/usize conversions"
)]
pub fn encode_dctf_buf(buf: &mut [u8], codepoint: u128) -> usize {
    if codepoint <= 0x7F {
        assert!(!buf.is_empty(), "Buffer too small for 1-byte DcTF encoding");
        if let Some(b) = buf.get_mut(0) {
            *b = u8::try_from(codepoint).expect("<= 0x7F fits in u8");
        }
        return 1;
    }

    if codepoint <= 0xF_FFFF_FFFF {
        let (len, lead_prefix): (usize, u8) = if codepoint <= 0x7FF {
            (2, 0xC0)
        } else if codepoint <= 0xFFFF {
            (3, 0xE0)
        } else if codepoint <= 0x1F_FFFF {
            (4, 0xF0)
        } else if codepoint <= 0x3FF_FFFF {
            (5, 0xF8)
        } else if codepoint <= 0x7FFF_FFFF {
            (6, 0xFC)
        } else {
            (7, 0xFE)
        };
        assert!(
            buf.len() >= len,
            "Buffer too small for {len}-byte DcTF encoding"
        );
        let cont_count = len.saturating_sub(1);
        let top_shift = cont_count.saturating_mul(6);
        if let Some(b0) = buf.get_mut(0) {
            *b0 = lead_prefix
                | u8::try_from(codepoint >> top_shift)
                    .expect("top bits fit in lead byte payload");
        }
        for i in 0..cont_count {
            let shift = cont_count
                .saturating_sub(1)
                .saturating_sub(i)
                .saturating_mul(6);
            if let Some(b) = buf.get_mut(1usize.saturating_add(i)) {
                *b = 0x80
                    | u8::try_from((codepoint >> shift) & 0x3F)
                        .expect("masked 6-bit group fits in u8");
            }
        }
        return len;
    }

    // Extended 0xFF tiers (k = 0..=15, total length 8..=23)
    let leading_zeros = usize::try_from(codepoint.leading_zeros())
        .expect("u32 leading_zeros fits in usize");
    let bits = 128usize.saturating_sub(leading_zeros);
    let k = bits.saturating_sub(38).div_ceil(6);
    debug_assert!(k <= 15);
    let cont_count = k.saturating_add(6);
    let total_len = k.saturating_add(8);
    assert!(
        buf.len() >= total_len,
        "Buffer too small for {total_len}-byte extended DcTF encoding"
    );

    let top_shift = cont_count.saturating_mul(6);
    let top_2 = if top_shift < 128 {
        u8::try_from((codepoint >> top_shift) & 0x03)
            .expect("2-bit value fits in u8")
    } else {
        0
    };
    let k_u8 = u8::try_from(k).expect("k <= 15 fits in u8");
    if let Some(b0) = buf.get_mut(0) {
        *b0 = 0xFF;
    }
    if let Some(b1) = buf.get_mut(1) {
        *b1 = 0x80 | (k_u8 << 2) | top_2;
    }
    for i in 0..cont_count {
        let shift = cont_count
            .saturating_sub(1)
            .saturating_sub(i)
            .saturating_mul(6);
        if let Some(b) = buf.get_mut(2usize.saturating_add(i)) {
            *b = 0x80
                | u8::try_from((codepoint >> shift) & 0x3F)
                    .expect("masked 6-bit group fits in u8");
        }
    }
    total_len
}

/// Decodes one `DcTF` (`.dctf`, `f641`) character from the provided byte slice.
///
/// Returns `Some((value, length_consumed))` if `bytes` begins with a valid,
/// canonical `DcTF` sequence, or `None` if malformed, non-canonical, or truncated.
#[must_use]
pub fn decode_dctf_buf(bytes: &[u8]) -> Option<(u128, usize)> {
    let first = *bytes.first()?;
    if first < 0x80 {
        return Some((u128::from(first), 1));
    }
    if first < 0xC2 {
        return None;
    }
    if first == 0xFF {
        let b1 = *bytes.get(1)?;
        if (b1 & 0xC0) != 0x80 {
            return None;
        }
        let k = usize::from((b1 >> 2) & 0x0F);
        let total_len = 8usize.saturating_add(k);
        if bytes.len() < total_len {
            return None;
        }
        let mut value = u128::from(b1 & 0x03);
        for i in 2..total_len {
            let b = *bytes.get(i)?;
            if (b & 0xC0) != 0x80 {
                return None;
            }
            value = (value << 6) | u128::from(b & 0x3F);
        }
        let min_val = if k == 0 {
            1u128 << 36
        } else {
            1u128 << (32usize.saturating_add(6usize.saturating_mul(k)))
        };
        if value < min_val {
            return None;
        }
        return Some((value, total_len));
    }

    let (len, min_val, lead_mask): (usize, u128, u8) = if (first & 0xE0) == 0xC0 {
        (2, 1u128 << 7, 0x1F)
    } else if (first & 0xF0) == 0xE0 {
        (3, 1u128 << 11, 0x0F)
    } else if (first & 0xF8) == 0xF0 {
        (4, 1u128 << 16, 0x07)
    } else if (first & 0xFC) == 0xF8 {
        (5, 1u128 << 21, 0x03)
    } else if (first & 0xFE) == 0xFC {
        (6, 1u128 << 26, 0x01)
    } else {
        // first == 0xFE
        (7, 1u128 << 31, 0x00)
    };

    if bytes.len() < len {
        return None;
    }

    let mut value = u128::from(first & lead_mask);
    for i in 1..len {
        let b = *bytes.get(i)?;
        if (b & 0xC0) != 0x80 {
            return None;
        }
        value = (value << 6) | u128::from(b & 0x3F);
    }

    if value < min_val {
        return None;
    }

    Some((value, len))
}

/// Computes the maximal-subpart error length for an invalid or incomplete
/// `DcTF` (`.dctf`, `f641`) byte slice.
///
/// Returns `Some(n)` (`n >= 1`) when `slice[..n]` is the longest initial subsequence
/// that can prefix a canonical `DcTF` sequence (or `Some(1)` if `slice[0]` cannot
/// start any canonical sequence), or `None` when the entire `slice` is a valid
/// incomplete prefix of a canonical `DcTF` sequence (truncated at the end of `slice`).
#[must_use]
pub fn determine_dctf_error_len(slice: &[u8]) -> Option<usize> {
    let first = *slice.first()?;
    if first < 0xC2 {
        return Some(1);
    }
    if first == 0xFF {
        let Some(&b1) = slice.get(1) else {
            return None;
        };
        if (b1 & 0xC0) != 0x80 || b1 == 0x80 {
            return Some(1);
        }
        let k = usize::from((b1 >> 2) & 0x0F);
        let expected_len = 8usize.saturating_add(k);
        for i in 2..expected_len {
            let Some(&b) = slice.get(i) else {
                return None;
            };
            if (b & 0xC0) != 0x80 {
                return Some(i);
            }
            if i == 2 && k >= 1 && (b1 & 0x03) == 0 && b < 0x84 {
                return Some(2);
            }
        }
        return Some(expected_len);
    }

    let (expected_len, min_b1): (usize, u8) = if first <= 0xDF {
        (2, 0x80)
    } else if first == 0xE0 {
        (3, 0xA0)
    } else if first <= 0xEF {
        (3, 0x80)
    } else if first == 0xF0 {
        (4, 0x90)
    } else if first <= 0xF7 {
        (4, 0x80)
    } else if first == 0xF8 {
        (5, 0x88)
    } else if first <= 0xFB {
        (5, 0x80)
    } else if first == 0xFC {
        (6, 0x84)
    } else if first == 0xFD {
        (6, 0x80)
    } else {
        // first == 0xFE
        (7, 0x82)
    };

    for i in 1..expected_len {
        let Some(&b) = slice.get(i) else {
            return None;
        };
        let lo = if i == 1 { min_b1 } else { 0x80 };
        if b < lo || b > 0xBF {
            return Some(i);
        }
    }
    Some(expected_len)
}

/// Encodes `codepoint` in `DcTF` (`.dctf`, `f641`) and returns a `Vec<u8>`.
#[must_use]
#[expect(
    clippy::expect_used,
    reason = "encode_dctf_buf returns length <= MAX_DCTF_LEN (23) which fits in buf"
)]
pub fn encode_dctf(codepoint: u128) -> Vec<u8> {
    let mut buf = [0u8; MAX_DCTF_LEN];
    let encoded_len = encode_dctf_buf(&mut buf, codepoint);
    buf.get(..encoded_len)
        .expect("encoded_len <= 23 fits in buf")
        .to_vec()
}

/// Decodes one `DcTF` (`.dctf`, `f641`) codepoint from `bytes`, or returns
/// `(0xFFFD, error_len)` using maximal-subpart error recovery on ill-formed input.
#[must_use]
pub fn decode_dctf(bytes: &[u8]) -> Option<(u128, usize)> {
    if bytes.is_empty() {
        return None;
    }

    if let Some(x) = decode_dctf_buf(bytes) {
        Some(x)
    } else {
        // Reason for fallback: determine_dctf_error_len returns None when `bytes`
        // is a truncated valid canonical prefix at a chunk boundary; in one-shot
        // lossy decoding (`decode_dctf`), the end of `bytes` is actual EOF, so
        // the entire truncated prefix is consumed as one U+FFFD.
        let err_len = determine_dctf_error_len(bytes).unwrap_or(bytes.len());
        Some((0xFFFD, err_len))
    }
}

