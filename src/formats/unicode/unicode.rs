/* SPDX-License-Identifier: MIT */
/* Copyright © Mathias Bynens <https://mathiasbynens.be/> */

//! Utilities for Unicode, including:
//! - Conversion of scalars to surrogates and vice versa
//! - UCS-2 encoding and decoding from scalars

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use anyhow::Result;

// Based on https://web.archive.org/web/20190305073920/https://github.com/mathiasbynens/wtf-8/blob/58c6b976c6678144d180b2307bee5615457e2cc7/wtf-8.js
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

pub const UNICODE_MAX: u32 = 0x10FFFF;
pub const UNICODE_HISTORIC_MAX: u32 = 0x7FFFFFFF; // UTF-8 0xFDBFBFBFBFBF

/// Combines surrogate pairs in an iterator of code units into Unicode scalar values.
/// Unpaired surrogates are left as-is.
fn combine_surrogates<I>(input: I) -> Vec<u32>
where
    I: IntoIterator,
    I::Item: Into<u32> + Copy,
{
    let mut codepoints = Vec::new();
    let mut iter = input.into_iter().peekable();
    while let Some(cp) = iter.next() {
        let cp = cp.into();
        if (0xD800..=0xDBFF).contains(&cp) {
            if let Some(&next) = iter.peek() {
                let next = next.into();
                if (0xDC00..=0xDFFF).contains(&next) {
                    let high = cp.saturating_sub(0xD800) << 10;
                    let low = next.saturating_sub(0xDC00);
                    codepoints.push(0x10000_u32.saturating_add(high | low));
                    iter.next(); // consume low surrogate
                    continue;
                }
            }
        }
        codepoints.push(cp);
    }
    codepoints
}

/// Decodes a slice of UCS-2/UTF-16 code units (`u16`) into Unicode scalar values (`u32`).
/// Surrogate pairs are combined into scalars; unpaired surrogates are left as-is.
pub fn ucs2decode(ucs2: &[u16]) -> Vec<u32> {
    combine_surrogates(ucs2.iter().copied())
}

/// Decodes a slice of code units (`u32`), combining surrogate pairs into scalars.
/// Unpaired surrogates are left as-is.
pub fn unpaired_surrogates_to_scalars(surrogates: &[u32]) -> Vec<u32> {
    combine_surrogates(surrogates.iter().copied())
}

/// Encodes an array of Unicode scalar values (`u32`) to UTF-16/UCS-2 code units (`Vec<u16>`).
/// Surrogate pairs are generated for codepoints above U+FFFF.
/// Returns an error for invalid codepoints.
pub fn ucs2encode(array: &[u32]) -> Result<Vec<u16>> {
    let mut output = Vec::new();
    for &value in array {
        if value > 0x10FFFF {
            return Err(anyhow::anyhow!("Invalid codepoint: {value}"));
        } else if value > 0xFFFF {
            let mut v = value.saturating_sub(0x10000);
            output.push(
                u16::try_from((v >> 10) & 0x3FF | 0xD800)
                    .context("Failed to create u16")?,
            );
            v = 0xDC00 | (v & 0x3FF);
            output.push(u16::try_from(v).context("Failed to create u16")?);
        } else if value <= 0x10FFFF {
            output.push(u16::try_from(value).context("Failed to create u16")?);
        }
    }
    Ok(output)
}

/// Converts a slice of Unicode scalar values (`u32`) to unpaired surrogates.
/// Codepoints above U+FFFF are split into surrogate pairs.
/// Returns an error for invalid codepoints.
pub fn scalars_to_unpaired_surrogates(codepoints: &[u32]) -> Result<Vec<u32>> {
    let mut result = Vec::new();
    for &cp in codepoints {
        if cp <= 0xFFFF {
            result.push(cp);
        } else if cp <= 0x10FFFF {
            let scalar = cp.saturating_sub(0x10000);
            let high = 0xD800_u32.saturating_add((scalar >> 10) & 0x3FF);
            let low = 0xDC00_u32.saturating_add(scalar & 0x3FF);
            result.push(high);
            result.push(low);
        } else {
            return Err(anyhow::anyhow!("Invalid codepoint: {cp}"));
        }
    }
    Ok(result)
}

pub fn scalars_to_string_lossy(scalars: &[u32]) -> String {
    combine_surrogates(scalars.to_vec())
        .iter()
        .map(|&cp| char::from_u32(cp).unwrap_or('\u{FFFD}'))
        .collect()
}

pub fn string_to_scalars(s: &str) -> Vec<u32> {
    s.chars().map(u32::from).collect()
}

// Rust equivalent of JS 'string'.slice(...), replicating the behavior of being
// able to slice in between surrogates.
pub fn js_like_slice_utf16(input: &str, start: usize, len: usize) -> Vec<u16> {
    // Encode as UTF-16 code units
    let utf16: Vec<u16> = input.encode_utf16().collect();
    // Compute end index (clamp to bounds)
    let end = usize::min(start.saturating_add(len), utf16.len());
    if start >= end {
        Vec::new()
    } else {
        utf16.get(start..end).unwrap_or(&[]).to_vec()
    }
}

// Convert a Vec<u16> to a byte array (little-endian)
pub fn u16_vec_to_le_bytes(v: &[u16]) -> Vec<u8> {
    let mut b = Vec::with_capacity(v.len().saturating_mul(2));
    for &u in v {
        b.push(u8::try_from(u & 0xFF).expect("Failed to create byte"));
        b.push(u8::try_from(u >> 8).expect("Failed to create byte"));
    }
    b
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
    fn test_unpaired_surrogates_to_scalars() {
        let surrogates = vec![0xD800, 0xDC00, 0xD800, 0xDC01];
        let scalars = unpaired_surrogates_to_scalars(&surrogates);
        assert_eq!(scalars, vec![0x10000, 0x10001]);
    }

    #[crate::ctb_test]
    fn test_scalars_to_unpaired_surrogates() {
        let scalars = vec![0x10000, 0x10001];
        let surrogates = scalars_to_unpaired_surrogates(&scalars)
            .expect("Failed to convert scalars to unpaired surrogates");
        assert_eq!(surrogates, vec![0xD800, 0xDC00, 0xD800, 0xDC01]);
    }

    #[crate::ctb_test]
    fn test_ucs2decode_basic() {
        // U+0041, U+10000 (surrogate pair), U+0042
        let ucs2 = vec![0x0041, 0xD800, 0xDC00, 0x0042];
        let scalars = ucs2decode(&ucs2);
        assert_eq!(scalars, vec![0x0041, 0x10000, 0x0042]);
    }

    #[crate::ctb_test]
    fn test_ucs2encode_basic() {
        // U+0041, U+10000, U+0042
        let scalars = vec![0x0041, 0x10000, 0x0042];
        let ucs2 = ucs2encode(&scalars).unwrap();
        assert_eq!(ucs2, vec![0x0041, 0xD800, 0xDC00, 0x0042]);
    }

    #[crate::ctb_test]
    fn test_ucs2encode_invalid() {
        // Invalid codepoint above U+10FFFF
        let scalars = vec![0x110000];
        let result = ucs2encode(&scalars);
        result.unwrap_err();
    }

    #[crate::ctb_test]
    fn test_scalars_to_unpaired_surrogates_invalid() {
        // Invalid codepoint above U+10FFFF
        let scalars = vec![0x110000];
        let result = scalars_to_unpaired_surrogates(&scalars);
        result.unwrap_err();
    }

    #[crate::ctb_test]
    fn test_js_like_slice_utf16() {
        let input = "a𠜎";
        let result = js_like_slice_utf16(input, 0, 2);
        assert_eq!(result, vec![0x61, 0xD841]);
    }

    #[crate::ctb_test]
    fn test_u16_vec_to_le_bytes() {
        let input = vec![0x61, 0xD841];
        let result = u16_vec_to_le_bytes(&input);
        assert_eq!(result, vec![0x61, 0x00, 0x41, 0xD8]);
    }
}
