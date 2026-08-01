/* SPDX-License-Identifier: MIT */
/* Copyright © Mathias Bynens <https://mathiasbynens.be/> */

//! Utilities for WTF-8 encoding and decoding.
//! Reference: <https://simonsapin.github.io/wtf-8>/
//! See also the standard library version:
//! <https://doc.rust-lang.org/src/std/sys_common/wtf8.rs.html>
//!
//! TODO?: This could have some more optional features added for other uses:
//! - Allow encoding of values above U+10FFFF (up to U+7FFFFFFF) as 6-byte
//!   sequences.
//! - Allow retaining surrogates unpaired when encoding, in which case it would
//!   be CESU-8 implementation.

// Based on <https://web.archive.org/web/20190305073920/https://github.com/mathiasbynens/wtf-8/blob/58c6b976c6678144d180b2307bee5615457e2cc7/wtf-8.js>
// This code for wtf8 is included under the following license (from https://web.archive.org/web/20190305074047/https://github.com/mathiasbynens/wtf-8/blob/58c6b976c6678144d180b2307bee5615457e2cc7/LICENSE-MIT.txt):
/*
Copyright Mathias Bynens <https://mathiasbynens.be/>

Permission is hereby granted, free of charge, to any person obtaining
a copy of this software and associated documentation files (the
"Software"), to deal in the Software without restriction, including
without limitation the rights to use, copy, modify, merge, publish,
distribute, sublicense, and/or sell copies of the Software, and to
permit persons to whom the Software is furnished to do so, subject to
the following conditions:

The above copyright notice and this permission notice shall be
included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE
LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use anyhow::{Context, Result, ensure};
use ctb_formats_unicode::{
    ucs2decode, ucs2encode, unpaired_surrogates_to_scalars,
};

/// Encode a single Unicode codepoint as WTF-8 byte array.
/// Returns a `Vec<u8>` containing the WTF-8 encoding of the codepoint.
/// Surrogates and non-scalar values are encoded as WTF-8 allows.
/// Values higher than 0x10FFFF are not supported.
pub fn encode_wtf8_single(int_value: u32) -> Result<Vec<u8>> {
    ensure!(
        int_value <= 0x10FFFF,
        format!("WTF-8: Codepoint {int_value} out of range")
    );
    fn create_byte(int_value: u32, shift: u32) -> Result<u8> {
        u8::try_from(((int_value >> shift) & 0x3F) | 0x80)
            .context("Failed to create byte")
    }

    let mut symbol = Vec::new();
    if (int_value & 0xFFFFFF80) == 0 {
        // 1-byte sequence
        symbol.push(u8::try_from(int_value).context("Failed to create byte")?);
    } else {
        if (int_value & 0xFFFFF800) == 0 {
            // 2-byte sequence
            symbol.push(
                u8::try_from((int_value >> 6) & 0x1F | 0xC0)
                    .context("Failed to create byte")?,
            );
        } else if (int_value & 0xFFFF0000) == 0 {
            // 3-byte sequence
            symbol.push(
                u8::try_from((int_value >> 12) & 0x0F | 0xE0)
                    .context("Failed to create byte")?,
            );
            symbol.push(create_byte(int_value, 6)?);
        } else if (int_value & 0xFFE00000) == 0 {
            // 4-byte sequence
            symbol.push(
                u8::try_from((int_value >> 18) & 0x07 | 0xF0)
                    .context("Failed to create byte")?,
            );
            symbol.push(create_byte(int_value, 12)?);
            symbol.push(create_byte(int_value, 6)?);
        }
        symbol.push(
            u8::try_from((int_value & 0x3F) | 0x80)
                .context("Failed to create byte")?,
        );
    }
    Ok(symbol)
}

/// More concise `encode_wtf8` using `ucs2decode/encode_wtf8_single` logic.
/// Encodes a slice of Unicode scalar codepoints to WTF-8 bytes.
/// Surrogate pairs are combined and encoded as a single codepoint.
/// Unpaired surrogates are encoded as 3-byte WTF-8 sequences.
/// Values higher than 0x10FFFF are not supported.
pub fn encode_wtf8_from_scalars(codepoints: &[u32]) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < codepoints.len() {
        let cp = *codepoints.get(i).context("Invalid codepoint index")?;
        // Check for surrogate pair
        if (0xD800..=0xDBFF).contains(&cp)
            && i.saturating_add(1) < codepoints.len()
        {
            let next = *codepoints
                .get(i.saturating_add(1))
                .context("Invalid surrogate pair index")?;
            if (0xDC00..=0xDFFF).contains(&next) {
                // Combine surrogate pair
                let high = cp;
                let low = next;
                let full = 0x10000u32.saturating_add(
                    ((high.saturating_sub(0xD800)) << 10)
                        | (low.saturating_sub(0xDC00)),
                );
                out.extend(encode_wtf8_single(full)?);
                i = i.saturating_add(2);
                continue;
            }
        }
        // Otherwise, encode as single codepoint
        out.extend(encode_wtf8_single(cp)?);
        i = i.saturating_add(1);
    }
    Ok(out)
}

/// WTF-8 encode a UTF-16/UCS-2/JS string (as &[u16]) into `Vec<u8>`.
/// Uses ucs2decode to convert to codepoints, then encodes as WTF-8.
pub fn encode_wtf8_from_ucs2(ucs2: &[u16]) -> Vec<u8> {
    let codepoints = ucs2decode(ucs2);
    encode_wtf8_from_scalars(codepoints.as_slice())
        .expect("It should not be possible to fail here")
}

fn read_continuation_byte(input: &[u8], byte_index: &mut usize) -> Result<u8> {
    if *byte_index >= input.len() {
        return Err(anyhow::anyhow!("WTF-8: Invalid byte index"));
    }
    let continuation_byte = *input
        .get(*byte_index)
        .context("WTF-8: Invalid byte index")?;
    *byte_index = byte_index.saturating_add(1);
    if (continuation_byte & 0xC0) == 0x80 {
        Ok(continuation_byte & 0x3F)
    } else {
        // If we end up here, it’s not a continuation byte.
        Err(anyhow::anyhow!("WTF-8: Invalid continuation byte"))
    }
}

/// Decode one WTF-8 codepoint from a byte array slice.
/// Returns Ok((codepoint, length)) or Err on error.
pub fn decode_wtf8_single(
    byte_array_input: &[u8],
) -> anyhow::Result<(u32, usize), anyhow::Error> {
    let mut byte_index = 0;
    let byte_count = byte_array_input.len();

    if byte_index > byte_count {
        return Err(anyhow::anyhow!("Invalid WTF-8 sequence"));
    }
    if byte_index == byte_count {
        return Err(anyhow::anyhow!(
            "The original WTF-8 returned false here, not sure why"
        ));
    }

    let byte1 = *byte_array_input
        .get(byte_index)
        .context("Invalid WTF-8 sequence")?;
    byte_index = byte_index.saturating_add(1);

    // 1-byte sequence (no continuation bytes)
    if (byte1 & 0x80) == 0 {
        return Ok((u32::from(byte1), 1));
    }

    // 2-byte sequence
    if (byte1 & 0xE0) == 0xC0 {
        let byte2 = read_continuation_byte(byte_array_input, &mut byte_index)?;
        let int_value = (u32::from(byte1 & 0x1F) << 6) | u32::from(byte2);
        if int_value >= 0x80 {
            return Ok((int_value, 2));
        }
        return Err(anyhow::anyhow!("WTF-8: Invalid 2-byte sequence"));
    }

    // 3-byte sequence (may include unpaired surrogates)
    if (byte1 & 0xF0) == 0xE0 {
        let byte2 = read_continuation_byte(byte_array_input, &mut byte_index)?;
        let byte3 = read_continuation_byte(byte_array_input, &mut byte_index)?;
        let int_value = (u32::from(byte1 & 0x0F) << 12)
            | (u32::from(byte2) << 6)
            | u32::from(byte3);
        if int_value >= 0x0800 {
            return Ok((int_value, 3));
        }
        return Err(anyhow::anyhow!("WTF-8: Invalid 3-byte sequence"));
    }

    // 4-byte sequence
    if (byte1 & 0xF8) == 0xF0 {
        let byte2 = read_continuation_byte(byte_array_input, &mut byte_index)?;
        let byte3 = read_continuation_byte(byte_array_input, &mut byte_index)?;
        let byte4 = read_continuation_byte(byte_array_input, &mut byte_index)?;
        let int_value = (u32::from(byte1 & 0x0F)) << 18
            | (u32::from(byte2)) << 12
            | (u32::from(byte3)) << 6
            | u32::from(byte4);
        if (0x010000..=0x10FFFF).contains(&int_value) {
            return Ok((int_value, 4));
        }
    }

    Err(anyhow::anyhow!("WTF-8: Invalid 4-byte sequence"))
}

/// Decode a WTF-8 byte array into a `Vec<u16>` (UCS-2).
pub fn decode_wtf8_to_ucs2(byte_array_input: &[u8]) -> Result<Vec<u16>> {
    let mut codepoints_with_unpaired_surrogates: Vec<u16> = Vec::new();
    let mut byte_index = 0;
    while byte_index < byte_array_input.len() {
        let sub_slice = byte_array_input
            .get(byte_index..)
            .context("WTF-8: Invalid byte index")?;
        let (cp, len) = decode_wtf8_single(sub_slice).context(format!(
            "WTF-8: Invalid byte sequence at index {byte_index}"
        ))?;
        if cp > 0xFFFF {
            let surrogates = ucs2encode(&[cp]).context(format!(
                "WTF-8: Failed to convert codepoint to unpaired surrogates: {cp:?}"
            ))?;
            codepoints_with_unpaired_surrogates.extend(surrogates);
        } else {
            codepoints_with_unpaired_surrogates
                .push(u16::try_from(cp).context("Failed to create u16")?);
        }
        byte_index = byte_index.saturating_add(len);
    }

    Ok(codepoints_with_unpaired_surrogates)
}

/// Decode a WTF-8 byte array into a `Vec<u16>` (UCS-2).
pub fn decode_wtf8_to_scalars(byte_array_input: &[u8]) -> Result<Vec<u32>> {
    let codepoints_with_unpaired_surrogates =
        decode_wtf8_to_ucs2(byte_array_input)
            .context("WTF-8: Failed to decode to UCS-2")?;
    let codepoints_as_u32: Vec<u32> = codepoints_with_unpaired_surrogates
        .iter()
        .map(|&cp| u32::from(cp))
        .collect();
    Ok(unpaired_surrogates_to_scalars(&codepoints_as_u32))
}

/// Returns true if the given slice is a valid WTF-8 single codepoint sequence.
pub fn is_unpackable_wtf8(byte_array_input: &[u8]) -> bool {
    decode_wtf8_single(byte_array_input).is_ok()
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
    use ctb_formats_unicode::{UNICODE_HISTORIC_MAX, UNICODE_MAX};

    use super::*;

    // Based on https://github.com/mathiasbynens/wtf-8/blob/bdab8ed45a2446eddffae28d27b353bb817189c5/tests/tests.js

    struct TestCase {
        decoded: &'static [u16],
        encoded: &'static [u8],
        description: &'static str,
    }

    #[crate::ctb_test]
    fn test_wtf8_encode_decode() {
        let cases = [
            // 1-byte
            TestCase {
                decoded: &[0x0000],
                encoded: b"\0",
                description: "U+0000",
            },
            TestCase {
                decoded: &[0x005C],
                encoded: b"\x5C",
                description: "U+005C",
            },
            TestCase {
                decoded: &[0x007F],
                encoded: b"\x7F",
                description: "U+007F",
            },
            // 2-byte
            TestCase {
                decoded: &[0x0080],
                encoded: &[0xC2, 0x80],
                description: "U+0080",
            },
            TestCase {
                decoded: &[0x05CA],
                encoded: &[0xD7, 0x8A],
                description: "U+05CA",
            },
            TestCase {
                decoded: &[0x07FF],
                encoded: &[0xDF, 0xBF],
                description: "U+07FF",
            },
            // 3-byte
            TestCase {
                decoded: &[0x0800],
                encoded: &[0xE0, 0xA0, 0x80],
                description: "U+0800",
            },
            TestCase {
                decoded: &[0x2C3C],
                encoded: &[0xE2, 0xB0, 0xBC],
                description: "U+2C3C",
            },
            TestCase {
                decoded: &[0xFFFF],
                encoded: &[0xEF, 0xBF, 0xBF],
                description: "U+FFFF",
            },
            // Unmatched surrogate halves

            // high surrogates: 0xD800 to 0xDBFF
            TestCase {
                decoded: &[0xD800],
                encoded: &[0xED, 0xA0, 0x80],
                description: "U+D800",
            },
            TestCase {
                decoded: &[0xD800, 0xD800],
                encoded: &[0xED, 0xA0, 0x80, 0xED, 0xA0, 0x80],
                description: "High surrogate followed by another high surrogate",
            },
            TestCase {
                decoded: &[0xD800, 0x41_u16],
                encoded: &[0xED, 0xA0, 0x80, b'A'],
                description: "High surrogate followed by a symbol that is not a surrogate",
            },
            TestCase {
                decoded: &[0xD800, 0xD834, 0xDF06, 0xD800],
                encoded: &[
                    0xED, 0xA0, 0x80, 0xF0, 0x9D, 0x8C, 0x86, 0xED, 0xA0, 0x80,
                ],
                description: "Unmatched high surrogate, followed by a surrogate pair, followed by an unmatched high surrogate",
            },
            TestCase {
                decoded: &[0xD9AF],
                encoded: &[0xED, 0xA6, 0xAF],
                description: "U+D9AF",
            },
            TestCase {
                decoded: &[0xDBFF],
                encoded: &[0xED, 0xAF, 0xBF],
                description: "U+DBFF",
            },
            // low surrogates: 0xDC00 to 0xDFFF
            TestCase {
                decoded: &[0xDC00],
                encoded: &[0xED, 0xB0, 0x80],
                description: "U+DC00",
            },
            TestCase {
                decoded: &[0xDC00, 0xDC00],
                encoded: &[0xED, 0xB0, 0x80, 0xED, 0xB0, 0x80],
                description: "Low surrogate followed by another low surrogate",
            },
            TestCase {
                decoded: &[0xDC00, 0x41_u16],
                encoded: &[0xED, 0xB0, 0x80, b'A'],
                description: "Low surrogate followed by a symbol that is not a surrogate",
            },
            TestCase {
                decoded: &[0xDC00, 0xD834, 0xDF06, 0xDC00],
                encoded: &[
                    0xED, 0xB0, 0x80, 0xF0, 0x9D, 0x8C, 0x86, 0xED, 0xB0, 0x80,
                ],
                description: "Unmatched low surrogate, followed by a surrogate pair, followed by an unmatched low surrogate",
            },
            TestCase {
                decoded: &[0xDEEE],
                encoded: &[0xED, 0xBB, 0xAE],
                description: "U+DEEE",
            },
            TestCase {
                decoded: &[0xDFFF],
                encoded: &[0xED, 0xBF, 0xBF],
                description: "U+DFFF",
            },
            // 4-byte
            TestCase {
                // 0x010000 as surrogates
                decoded: &[0xD800, 0xDC00],
                encoded: &[0xF0, 0x90, 0x80, 0x80],
                description: "U+10000",
            },
            TestCase {
                // 0x01D306 as surrogates
                decoded: &[0xD834, 0xDF06],
                encoded: &[0xF0, 0x9D, 0x8C, 0x86],
                description: "U+1D306",
            },
            TestCase {
                // 0x10FFFF as surrogates
                decoded: &[0xDBFF, 0xDFFF],
                encoded: &[0xF4, 0x8F, 0xBF, 0xBF],
                description: "U+10FFFF",
            },
        ];
        for case in &cases {
            // Encode
            let encoded = encode_wtf8_from_ucs2(case.decoded);
            assert_eq!(encoded, case.encoded, "Encoding: {}", case.description);
            // Decode
            let decoded = decode_wtf8_to_ucs2(case.encoded);
            if let Ok(decoded) = &decoded {
                assert_eq!(
                    decoded.as_slice(),
                    case.decoded,
                    "Decoding: {}, encoded: {:?}, decoded: {:?}, expected: {:?}",
                    case.description,
                    case.encoded,
                    decoded,
                    case.decoded
                );
            } else {
                panic!(
                    "Decoding error for case '{}':\n  error: {:?}\n  encoded: {:?}\n  expected decoded: {:?}",
                    case.description,
                    decoded.err(),
                    case.encoded,
                    case.decoded,
                );
            }
        }
    }

    #[crate::ctb_test]
    fn test_wtf8_decode_errors() {
        // Invalid WTF-8 detected
        decode_wtf8_single(&[0xFF]).unwrap_err();
        // Invalid continuation byte (4-byte sequence expected)
        decode_wtf8_single(&[0xE9, 0x00, 0x00]).unwrap_err();
        // Invalid continuation byte
        decode_wtf8_single(&[0xC2, 0xFF, 0xFF]).unwrap_err();
        decode_wtf8_single(&[0xC2, 0xEF, 0xBF, 0xBF]).unwrap_err();
        // Invalid byte index
        decode_wtf8_single(&[0xF0, 0x9D]).unwrap_err();
        decode_wtf8_to_scalars(&[0xFD, 0xBF, 0xBF, 0xBF, 0xBF, 0xBF])
            .unwrap_err(); // UNICODE_HISTORIC_MAX
    }

    #[crate::ctb_test]
    fn test_encode_wtf8_single_basic() {
        // 1-byte
        assert_eq!(encode_wtf8_single(0x00).unwrap(), vec![0x00]);
        assert_eq!(encode_wtf8_single(0x7F).unwrap(), vec![0x7F]);
        // 2-byte
        assert_eq!(encode_wtf8_single(0x80).unwrap(), vec![0xC2, 0x80]);
        assert_eq!(encode_wtf8_single(0x7FF).unwrap(), vec![0xDF, 0xBF]);
        // 3-byte
        assert_eq!(encode_wtf8_single(0x800).unwrap(), vec![0xE0, 0xA0, 0x80]);
        assert_eq!(encode_wtf8_single(0xFFFF).unwrap(), vec![0xEF, 0xBF, 0xBF]);
        // 4-byte
        assert_eq!(
            encode_wtf8_single(0x10000).unwrap(),
            vec![0xF0, 0x90, 0x80, 0x80]
        );
        assert_eq!(
            encode_wtf8_single(UNICODE_MAX).unwrap(),
            vec![0xF4, 0x8F, 0xBF, 0xBF]
        );
        encode_wtf8_single(UNICODE_HISTORIC_MAX).unwrap_err();
    }

    #[crate::ctb_test]
    fn test_encode_wtf8_from_scalars_surrogates() {
        // Unpaired high surrogate
        assert_eq!(
            encode_wtf8_from_scalars(&[0xD800]).unwrap(),
            vec![0xED, 0xA0, 0x80]
        );
        // Surrogate pair
        assert_eq!(
            encode_wtf8_from_scalars(&[0xD834, 0xDF06]).unwrap(),
            vec![0xF0, 0x9D, 0x8C, 0x86]
        );
        // Mixed
        assert_eq!(
            encode_wtf8_from_scalars(&[0xD800, 0xD834, 0xDF06, 0xD800])
                .unwrap(),
            vec![0xED, 0xA0, 0x80, 0xF0, 0x9D, 0x8C, 0x86, 0xED, 0xA0, 0x80]
        );
    }

    #[crate::ctb_test]
    fn test_decode_wtf8_to_scalars_basic() {
        // 1-byte
        assert_eq!(decode_wtf8_to_scalars(&[0x00]).unwrap(), vec![0x00]);
        // 2-byte
        assert_eq!(decode_wtf8_to_scalars(&[0xC2, 0x80]).unwrap(), vec![0x80]);
        // 3-byte
        assert_eq!(
            decode_wtf8_to_scalars(&[0xE0, 0xA0, 0x80]).unwrap(),
            vec![0x800]
        );
        // 4-byte
        assert_eq!(
            decode_wtf8_to_scalars(&[0xF0, 0x90, 0x80, 0x80]).unwrap(),
            vec![0x10000]
        );
        assert_eq!(
            decode_wtf8_to_scalars(&[0xF4, 0x8F, 0xBF, 0xBF]).unwrap(),
            vec![UNICODE_MAX]
        );
        decode_wtf8_to_scalars(&[0xFD, 0xBF, 0xBF, 0xBF, 0xBF, 0xBF])
            .unwrap_err();
    }

    #[crate::ctb_test]
    fn test_is_unpackable_wtf8() {
        assert!(is_unpackable_wtf8(&[0x00]));
        assert!(is_unpackable_wtf8(b"AAAA"));
        assert!(is_unpackable_wtf8(&[b'A', b'A', b'A', 0xE0, 0xA0, 0x80]));
        assert!(is_unpackable_wtf8(&[0xE0, 0xA0, 0x80, b'A', b'A', b'A']));
        assert!(is_unpackable_wtf8(&[0xC2, 0x80]));
        assert!(is_unpackable_wtf8(&[0xE0, 0xA0, 0x80]));
        assert!(is_unpackable_wtf8(&[0xF0, 0x90, 0x80, 0x80]));
        assert!(!is_unpackable_wtf8(&[0xC2])); // incomplete
        assert!(!is_unpackable_wtf8(&[0xF0, 0x90])); // incomplete
        assert!(!is_unpackable_wtf8(&[0xFF])); // invalid
    }
}
