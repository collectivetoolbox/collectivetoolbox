// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT
// SPDX-License-Identifier for parts derived from Base16b: MIT
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

/* Copyright © 2009 Base16b.org */

/* Based on https://web.archive.org/web/20090902074623/http://www.base16b.org/doc/specification/version/0.1/base16b.pdf */

// This code for the Base16b object is included under the following license:
/*
 * Base16b family encode / decode
 * http://base16b.org/lib/version/0.1/js/base16b.js
 * or http://base16b.org/lib/js/base16b.js
 */
/*
    Copyright (c) 2009 Base16b.org
    Permission is hereby granted, free of charge, to any person
    obtaining a copy of this software and associated documentation
    files (the "Software"), to deal in the Software without
    restriction, including without limitation the rights to use,
    copy, modify, merge, publish, distribute, sublicense, and/or sell
    copies of the Software, and to permit persons to whom the
    Software is furnished to do so, subject to the following
    conditions:
    The above copyright notice and this permission notice shall be
    included in all copies or substantial portions of the Software.
    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
    EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES
    OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
    NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT
    HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
    WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
    FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
    OTHER DEALINGS IN THE SOFTWARE.
*/

//! Implements the Base16b format.
//!
//! Based on
//! <https://web.archive.org/web/20090902074623/http://www.base16b.org/doc/specification/version/0.1/base16b.pdf>
//! Note that the Base16b and Basenb formats provided here are different from
//! the Base16b formats in the specification, due to what appears to be a bug in
//! the specification (requiring the remainder length to be stored to decode the
//! remainder correctly when it starts with a 0 bit and is not 16 bits long).

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use anyhow::{Result, bail};

use crate::bail_if_none;
use ctb_utilities::circular_dep_unicode::{
    js_like_slice_utf16, string_to_scalars, ucs2encode,
};


#[derive(Debug, Clone, Copy)]
struct CpValue {
    value: u32,
    cp: u32,
}

// Private constants
// +UF0000 is the first code point in the Asyntactic script
const AS_START: CpValue = CpValue {
    value: 0x0000,
    cp: 0xF0000,
};

// Private non-contiguous code points
fn noncont() -> [CpValue; 4] {
    [
        CpValue {
            value: 0xFFFE,
            cp: 0xF80A,
        },
        CpValue {
            value: 0xFFFF,
            cp: 0xF80B,
        },
        CpValue {
            value: 0x1FFFE,
            cp: 0xF80C,
        },
        CpValue {
            value: 0x1FFFF,
            cp: 0xF80D,
        },
    ]
}

// Private: bytes needed for a character (usually 2)
fn char_bytes(segm_cp: &[u16]) -> usize {
    if fixed_char_code_at(segm_cp, 0).is_ok()
        && fixed_char_code_at(segm_cp, 1).is_ok()
    {
        2
    } else {
        1
    }
}

// Private: bytes needed for a character (fixed surrogate logic)
fn char_bytes_fixed(segm_cp: &[u16]) -> u8 {
    // Reason for fallback: when slice is empty, first code point is None (code = 0), which is not a UTF-16 surrogate (0xD800..=0xDFFF), returning 1 byte.
    let code = segm_cp.first().copied().unwrap_or(0);
    if (0xD800..=0xDBFF).contains(&code) || (0xDC00..=0xDFFF).contains(&code) {
        2
    } else {
        1
    }
}

// Private: Two's complement for the value for this base
fn invert_val(segm_val: u32, base: u32) -> u32 {
    2u32.pow(base).saturating_sub(segm_val.saturating_add(1))
}

// Private: from code point to value
fn from_code_point(segm_cp: &[u16], bytes: u8) -> Result<u32> {
    // Map Code Point to a segment value as specified by the mapping table for this base in the Asyntactic script
    let code = fixed_char_code_at(segm_cp, 0)?;
    if bytes == 2 {
        return Ok(code.saturating_sub(AS_START.cp));
    }
    for nc in &noncont() {
        // handle non-contiguous code points for last two CPs in bases 16
        // and 17
        if nc.cp == code {
            return Ok(nc.value);
        }
    }
    bail!(format!("from_code_point: unknown code point {segm_cp:?}"))
}

// Private: value to code point
fn to_code_point(segm_val: u32, base: u32) -> u32 {
    // Map a segment value to the Code Point specified by the mapping table for this base in the Asyntactic script
    if base < 16 {
        AS_START.cp.saturating_add(segm_val)
    } else {
        for nc in &noncont() {
            // handle non-contiguous code points for bases 16 and 17
            if nc.value == segm_val {
                return nc.cp;
            }
        }
        AS_START.cp.saturating_add(segm_val)
    }
}

// Private: code point to String (fixed surrogate logic)
fn fixed_from_char_code(code_pt: u32) -> String {
    // Reason for fallback: when code point is not a valid Unicode scalar value, format standard replacement character '\u{FFFD}'.
    std::char::from_u32(code_pt)
        .map_or_else(|| String::from("\u{FFFD}"), |c| c.to_string())
}

// Private: get the code point at a string index (surrogate aware)
fn fixed_char_code_at(str_: &[u16], idx: usize) -> Result<u32> {
    // https://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/String/charCodeAt
    if idx >= str_.len() {
        return Err(anyhow::anyhow!("index out of bounds"));
    }
    let code = u32::from(*str_.get(idx).context("index out of bounds")?);
    if (0xD800..=0xDBFF).contains(&code) {
        // High surrogate (could change last hex to 0xDB7F to treat high private surrogates as single characters)
        let next_idx = idx.saturating_add(1);
        if next_idx < str_.len() {
            let hi = code;
            let low =
                u32::from(*str_.get(next_idx).context("index out of bounds")?);
            let val = hi
                .saturating_sub(0xD800)
                .saturating_mul(0x400)
                .saturating_add(low.saturating_sub(0xDC00))
                .saturating_add(0x10000);
            return Ok(val);
        }
    }
    if (0xDC00..=0xDFFF).contains(&code) {
        // Low surrogate
        if idx >= 1 {
            let prev_idx = idx.saturating_sub(1);
            let hi =
                u32::from(*str_.get(prev_idx).context("index out of bounds")?);
            let low = code;
            let val = hi
                .saturating_sub(0xD800)
                .saturating_mul(0x400)
                .saturating_add(low.saturating_sub(0xDC00))
                .saturating_add(0x10000);
            return Ok(val);
        }
    }
    Ok(code)
}

/// Encode an array of pseudo-booleans (0 or 1) to a string using the
/// Asyntactic script.
pub fn encode(input_arr: &[u8], base: u32) -> Result<String> {
    /*
    Encode an array of pseudo-booleans (0 or 1)
    The specification of the encoding is documented elsewhere on this site. (Search Asyntactic script and Base16b.)

    See `byte_array_to_int_bit_array` and other functions in bitwise.rs for
    converting to and from these arrays of bits.
    */
    if !(7..=17).contains(&base) {
        bail!("invalid base".to_string());
    }
    let base_usize = usize::try_from(base)?;
    let mut result_arr: Vec<String> = Vec::new();
    #[expect(
        clippy::expect_used,
        reason = "base_usize is in 7..=17, so division by zero is impossible"
    )]
    let full_segments = input_arr
        .len()
        .checked_div(base_usize)
        .expect("base_usize is non-zero (7..=17)");
    let remain_bits = input_arr
        .len()
        .saturating_sub(full_segments.saturating_mul(base_usize));
    for segment in 0..full_segments {
        let segmstart = base_usize.saturating_mul(segment);
        let segmend = segmstart.saturating_add(base_usize);
        let currsegm = input_arr
            .get(segmstart..segmend)
            .context("Invalid segment range")?;
        let mut segm_val = 0u32;
        for (bit_idx, &b) in currsegm.iter().enumerate() {
            let shift = u32::try_from(
                base_usize.saturating_sub(1).saturating_sub(bit_idx),
            )?;
            #[expect(
                clippy::expect_used,
                reason = "shift is < 17, so 32-bit shift cannot overflow"
            )]
            let bit_val = u32::from(b)
                .checked_shl(shift)
                .expect("shift is < 17, which fits in u32");
            segm_val = segm_val.saturating_add(bit_val);
        }
        let cp = to_code_point(segm_val, base);
        result_arr.push(fixed_from_char_code(cp));
    }
    // encode termination character
    let segmstart = base_usize.saturating_mul(full_segments);
    let currsegm = input_arr
        .get(segmstart..)
        .context("Invalid remainder range")?;
    // most significant bit at the start (left) / least significant bit at
    // the end (right).
    let mut segm_val = 0u32;
    if remain_bits > 0 {
        for (bit_idx, &b) in currsegm.iter().enumerate() {
            let shift = u32::try_from(
                remain_bits.saturating_sub(1).saturating_sub(bit_idx),
            )?;
            #[expect(
                clippy::expect_used,
                reason = "shift is < remain_bits <= 17, so 32-bit shift cannot overflow"
            )]
            let bit_val = u32::from(b)
                .checked_shl(shift)
                .expect("shift is < 17, which fits in u32");
            segm_val = segm_val.saturating_add(bit_val);
        }
    }
    let term_cp = to_code_point(invert_val(segm_val, base), base);
    result_arr.push(fixed_from_char_code(term_cp));
    Ok(result_arr.concat())
}

/// Decode a string encoded in the Asyntactic script. Return an array of
/// pseudo-booleans (0 or 1).
///
/// remainderLength is not in the original version of this code. It should
/// be provided to get round-trip output. It is the input length in bits,
/// mod the number of bits per character (the second argument to the encode
/// function). Other fixes to decoding are also made if remainderLength is
/// provided. If it is not provided, the output should be the same as with
/// original API (if not, that's a bug) - WITH THE EXCEPTION that in some
/// cases the original version sometimes returned NaN for some bits. See the
/// tests for an example. I haven't looked into why that is.
///
/// Pass None as the `remainder_length` to use per the original API.
pub fn decode(
    input_str: &str,
    remainder_length: Option<u32>,
) -> Result<Vec<u8>> {
    let input_str = ucs2encode(&string_to_scalars(input_str))?;
    let original_api = remainder_length.is_none();
    let mut result_arr: Vec<u8> = Vec::new();
    let term_char_bytes_slice = input_str
        .get(input_str.len().saturating_sub(1)..)
        .context("Empty input string")?;
    let term_char_bytes = char_bytes_fixed(term_char_bytes_slice);
    let term_char_bytes_usize: usize = term_char_bytes.into();
    let term_char_cp_slice = input_str
        .get(input_str.len().saturating_sub(term_char_bytes_usize)..)
        .context("Invalid termination character range")?;
    let term_char_val = from_code_point(term_char_cp_slice, term_char_bytes)?;
    let mut bit: u32 = 17;
    // decode the base from the termination character
    while bit >= 7 {
        let bit_minus_1 = bit.saturating_sub(1);
        let divisor = 2u32.pow(bit_minus_1);
        #[expect(
            clippy::expect_used,
            reason = "divisor is 2^(bit-1) >= 64, which is non-zero"
        )]
        let div_result = term_char_val
            .checked_div(divisor)
            .expect("divisor is non-zero power of 2");
        if div_result != 0 {
            break;
        }
        bit = bit.saturating_sub(1);
    }
    let base: u32 = if (7..=17).contains(&bit) {
        bit
    } else {
        bail!("invalid base decoded");
    };
    let mut bytes_used: usize = 0;
    let full_bytes = input_str.len().saturating_sub(term_char_bytes_usize);
    while bytes_used < full_bytes {
        // decode the code point segments in sequence
        let slice_for_bytes = input_str
            .get(bytes_used..=bytes_used)
            .context("Invalid bytes used range")?;
        let curr_char_bytes = char_bytes_fixed(slice_for_bytes); // taste before taking a byte
        let curr_char_bytes_usize: usize = curr_char_bytes.into();
        let segment_bit_length: u32 = if original_api {
            u32::from(curr_char_bytes).saturating_mul(8)
        } else {
            base
        };
        let seg_end = bytes_used.saturating_add(curr_char_bytes_usize);
        let segment = input_str
            .get(bytes_used..seg_end)
            .context("Invalid segment index")?;
        let segm_val = from_code_point(segment, curr_char_bytes)?;
        // most significant bit at the start (left) / least significant bit at the end (right).
        let sbl_usize = usize::try_from(segment_bit_length)?;
        for bit_pos_usize in (0..sbl_usize).rev() {
            let bit_pos = u32::try_from(bit_pos_usize)?;
            let divisor = 2u64.pow(bit_pos);
            #[expect(
                clippy::expect_used,
                reason = "divisor is 2^bit_pos >= 1, which is non-zero"
            )]
            let div_val = u64::from(segm_val)
                .checked_div(divisor)
                .expect("divisor is non-zero power of 2");
            let raw: u32 = u32::try_from(div_val.rem_euclid(2))?;
            let decoded_bit = u8::try_from(raw)?;
            if !original_api && decoded_bit > 1 {
                bail!("Found incorrect bit while decoding");
            }
            result_arr.push(decoded_bit);
        }
        bytes_used = bytes_used.saturating_add(curr_char_bytes_usize);
    }
    // remainder
    let remain_val = invert_val(term_char_val, base); // decode the remainder from the termination character
    let mut bit: i16 =
        i16::from(term_char_bytes.saturating_mul(8)).saturating_sub(1);
    if (bit > 0) && !original_api {
        let rem_len = bail_if_none!(remainder_length);
        bit = i16::try_from(rem_len)?.saturating_sub(1);
    }
    if bit >= 0 {
        while bit >= 0 {
            let bit_u32: u32 = bit.try_into()?;
            let divisor = 2u32.pow(bit_u32);
            #[expect(
                clippy::expect_used,
                reason = "divisor is 2^bit_u32 >= 1, which is non-zero"
            )]
            let div_val = remain_val
                .checked_div(divisor)
                .expect("divisor is non-zero power of 2");
            let bit_val = u8::try_from(div_val.rem_euclid(2))?;
            result_arr.push(bit_val);
            bit = bit.saturating_sub(1);
        }
    }
    Ok(result_arr)
}

/// Public: count Unicode characters in the way Base16b does
pub fn true_length(input_str: &str) -> Result<usize> {
    /*
    Count the number of characters in a string.
    This function can handle stings of mixed BMP plane and higher Unicode planes.
    Fixes a problem with Javascript which incorrectly that assumes each character is only one byte.
    */
    let str_bytes = ucs2encode(&string_to_scalars(input_str))?.len();
    let mut str_length: usize = 0;
    let mut tally_bytes: usize = 0;
    while tally_bytes < str_bytes {
        let slice_end = if tally_bytes.saturating_add(2) > input_str.len() {
            input_str.len()
        } else {
            tally_bytes.saturating_add(2)
        };
        // select utf-16 slices. This will sometimes index into the middle
        // of surrogate pairs.
        let slice = js_like_slice_utf16(input_str, tally_bytes, slice_end);
        let char_bytes = char_bytes(&slice);
        tally_bytes = tally_bytes.saturating_add(char_bytes);
        str_length = str_length.saturating_add(1);
    }
    Ok(str_length)
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

    use crate::utilities::assert_vec_u8_ok_eq;
    use ctb_formats_utilities::assert_string_ok_eq;

    #[crate::ctb_test]
    fn test_char_bytes() {
        assert_eq!(char_bytes(&"a".encode_utf16().collect::<Vec<u16>>()), 1);
        assert_eq!(char_bytes(&"𠜎".encode_utf16().collect::<Vec<u16>>()), 2);
        assert_eq!(char_bytes(&"🥴".encode_utf16().collect::<Vec<u16>>()), 2);
    }

    #[crate::ctb_test]
    fn test_true_length() {
        assert_eq!(true_length("aa").unwrap(), 1);
        assert_eq!(true_length("a𠜎").unwrap(), 2);
        assert_eq!(true_length("a🥴").unwrap(), 2);
    }

    #[crate::ctb_test]

    fn test_encode_decode() {
        // This is UUID e82eef60-19bc-4a00-a44a-763a3445c16f as bits
        // To convert:
        // ctoolbox base2base 2 2 -q --limit 1 --pad --separator ', ' "$(ctoolbox base2base 16 2 -q --limit 255 --pad --separator ', ' 'e82eef60-19bc-4a00-a44a-763a3445c16f')"
        let input = vec![
            1, 1, 1, 0, 1, 0, 0, 0, //
            0, 0, 1, 0, 1, 1, 1, 0, //
            1, 1, 1, 0, 1, 1, 1, 1, //
            0, 1, 1, 0, 0, 0, 0, 0, //
            0, 0, 0, 1, 1, 0, 0, 1, //
            1, 0, 1, 1, 1, 1, 0, 0, //
            0, 1, 0, 0, 1, 0, 1, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, //
            1, 0, 1, 0, 0, 1, 0, 0, //
            0, 1, 0, 0, 1, 0, 1, 0, //
            0, 1, 1, 1, 0, 1, 1, 0, //
            0, 0, 1, 1, 1, 0, 1, 0, //
            0, 0, 1, 1, 0, 1, 0, 0, //
            0, 1, 0, 0, 0, 1, 0, 1, //
            1, 1, 0, 0, 0, 0, 0, 1, //
            0, 1, 1, 0, 1, 1, 1, 1,
        ];
        let expected7 = "\u{f0074}\u{f000b}\u{f005d}\u{f0076}\u{f0000}\u{f0066}\u{f0078}\u{f004a}\u{f0000}\u{f0029}\u{f0009}\u{f0027}\u{f0031}\u{f0068}\u{f0068}\u{f0045}\u{f0060}\u{f005b}\u{f007c}";
        let expected8 = "\u{f00e8}\u{f002e}\u{f00ef}\u{f0060}\u{f0019}\u{f00bc}\u{f004a}\u{f0000}\u{f00a4}\u{f004a}\u{f0076}\u{f003a}\u{f0034}\u{f0045}\u{f00c1}\u{f006f}\u{f00ff}";
        let expected9 = "\u{f01d0}\u{f00bb}\u{f017b}\u{f0001}\u{f0137}\u{f0112}\u{f0100}\u{f00a4}\u{f0094}\u{f01d8}\u{f01d1}\u{f0144}\u{f00b8}\u{f005b}\u{f01fc}";
        let expected10 = "\u{f03a0}\u{f02ee}\u{f03d8}\u{f0019}\u{f02f1}\u{f00a0}\u{f0029}\u{f004a}\u{f01d8}\u{f03a3}\u{f0111}\u{f01c1}\u{f0390}";
        let expected11 = "\u{f0741}\u{f03bb}\u{f06c0}\u{f019b}\u{f0625}\u{f0002}\u{f0489}\u{f0276}\u{f01d1}\u{f0511}\u{f0382}\u{f0790}";
        let expected12 = "\u{f0e82}\u{f0eef}\u{f0601}\u{f09bc}\u{f04a0}\u{f00a4}\u{f04a7}\u{f063a}\u{f0344}\u{f05c1}\u{f0f90}";
        let expected13 = "\u{f1d05}\u{f1bbd}\u{f100c}\u{f1bc4}\u{f1401}\u{f0912}\u{f13b1}\u{f1a34}\u{f08b8}\u{f1e90}";
        let expected14 = "\u{f3a0b}\u{f2ef6}\u{f0066}\u{f3c4a}\u{f0029}\u{f04a7}\u{f18e8}\u{f3445}\u{f305b}\u{f3ffc}";
        let expected15 = "\u{f7417}\u{f3bd8}\u{f0337}\u{f44a0}\u{f0522}\u{f29d8}\u{f7468}\u{f45c1}\u{f7f90}";
        let expected16 = "\u{fe82e}\u{fef60}\u{f19bc}\u{f4a00}\u{fa44a}\u{f763a}\u{f3445}\u{fc16f}\u{f80b}";
        let expected17 = "\u{10d05d}\u{10bd80}\u{fcde2}\u{fa00a}\u{f894e}\u{108e8d}\u{f22e0}\u{10fe90}";

        encode(&input, 6).unwrap_err();
        encode(&input, 18).unwrap_err();
        let result7 = assert_string_ok_eq(expected7, encode(&input, 7));
        let result8 = assert_string_ok_eq(expected8, encode(&input, 8));
        let result9 = assert_string_ok_eq(expected9, encode(&input, 9));
        let result10 = assert_string_ok_eq(expected10, encode(&input, 10));
        let result11 = assert_string_ok_eq(expected11, encode(&input, 11));
        let result12 = assert_string_ok_eq(expected12, encode(&input, 12));
        let result13 = assert_string_ok_eq(expected13, encode(&input, 13));
        let result14 = assert_string_ok_eq(expected14, encode(&input, 14));
        let result15 = assert_string_ok_eq(expected15, encode(&input, 15));
        let result16 = assert_string_ok_eq(expected16, encode(&input, 16));
        let result17 = assert_string_ok_eq(expected17, encode(&input, 17));

        decode("abcd", None).unwrap_err();

        let input_len = u32::try_from(input.len()).unwrap();
        assert_vec_u8_ok_eq(&input, decode(&result7, Some(input_len % 7)));
        assert_vec_u8_ok_eq(&input, decode(&result8, Some(input_len % 8)));
        assert_vec_u8_ok_eq(&input, decode(&result9, Some(input_len % 9)));
        assert_vec_u8_ok_eq(&input, decode(&result10, Some(input_len % 10)));
        assert_vec_u8_ok_eq(&input, decode(&result11, Some(input_len % 11)));
        assert_vec_u8_ok_eq(&input, decode(&result12, Some(input_len % 12)));
        assert_vec_u8_ok_eq(&input, decode(&result13, Some(input_len % 13)));
        assert_vec_u8_ok_eq(&input, decode(&result14, Some(input_len % 14)));
        assert_vec_u8_ok_eq(&input, decode(&result15, Some(input_len % 15)));
        assert_vec_u8_ok_eq(&input, decode(&result16, Some(input_len % 16)));
        assert_vec_u8_ok_eq(&input, decode(&result17, Some(input_len % 17)));

        #[rustfmt::skip]
        let no_remainder_res7 = &[0,0,0,0,0,0,0,0,0,1,1,1,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,1,1,0,0,0,0,0,0,0,0,0,1,0,1,1,1,0,1,0,0,0,0,0,0,0,0,0,1,1,1,0,1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1,0,0,1,1,0,0,0,0,0,0,0,0,0,0,1,1,1,1,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,1,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,1,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,1,0,0,0,0,0,0,0,0,0,0,1,0,0,1,1,1,0,0,0,0,0,0,0,0,0,0,1,1,0,0,0,1,0,0,0,0,0,0,0,0,0,1,1,0,1,0,0,0,0,0,0,0,0,0,0,0,0,1,1,0,1,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,1,0,1,0,0,0,0,0,0,0,0,0,1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,1,1,0,1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1];
        #[rustfmt::skip]
        let no_remainder_res8 = &[0,0,0,0,0,0,0,0,1,1,1,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,1,1,1,0,0,0,0,0,0,0,0,0,1,1,1,0,1,1,1,1,0,0,0,0,0,0,0,0,0,1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1,0,0,1,0,0,0,0,0,0,0,0,1,0,1,1,1,1,0,0,0,0,0,0,0,0,0,0,0,1,0,0,1,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,1,0,0,1,0,0,0,0,0,0,0,0,0,0,0,1,0,0,1,0,1,0,0,0,0,0,0,0,0,0,0,1,1,1,0,1,1,0,0,0,0,0,0,0,0,0,0,0,1,1,1,0,1,0,0,0,0,0,0,0,0,0,0,0,1,1,0,1,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,1,0,1,0,0,0,0,0,0,0,0,1,1,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,1,1,0,1,1,1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0];
        #[rustfmt::skip]
        let no_remainder_res9 = &[0,0,0,0,0,0,0,1,1,1,0,1,0,0,0,0,0,0,0,0,0,0,0,0,1,0,1,1,1,0,1,1,0,0,0,0,0,0,0,1,0,1,1,1,1,0,1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,1,0,0,1,1,0,1,1,1,0,0,0,0,0,0,0,1,0,0,0,1,0,0,1,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,1,0,0,1,0,0,0,0,0,0,0,0,0,0,1,0,0,1,0,1,0,0,0,0,0,0,0,0,0,1,1,1,0,1,1,0,0,0,0,0,0,0,0,0,0,1,1,1,0,1,0,0,0,1,0,0,0,0,0,0,0,1,0,1,0,0,0,1,0,0,0,0,0,0,0,0,0,0,1,0,1,1,1,0,0,0,0,0,0,0,0,0,0,0,0,1,0,1,1,0,1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1];
        #[rustfmt::skip]
        let no_remainder_res10 = &[0,0,0,0,0,0,1,1,1,0,1,0,0,0,0,0,0,0,0,0,0,0,1,0,1,1,1,0,1,1,1,0,0,0,0,0,0,0,1,1,1,1,0,1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1,0,0,1,0,0,0,0,0,0,1,0,1,1,1,1,0,0,0,1,0,0,0,0,0,0,0,0,1,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,1,0,0,1,0,0,0,0,0,0,0,0,0,1,0,0,1,0,1,0,0,0,0,0,0,0,0,1,1,1,0,1,1,0,0,0,0,0,0,0,0,0,1,1,1,0,1,0,0,0,1,1,0,0,0,0,0,0,0,1,0,0,0,1,0,0,0,1,0,0,0,0,0,0,0,1,1,1,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,1,1,0,1,1,1,1];
        #[rustfmt::skip]
        let no_remainder_res11 = &[0,0,0,0,0,1,1,1,0,1,0,0,0,0,0,1,0,0,0,0,0,0,1,1,1,0,1,1,1,0,1,1,0,0,0,0,0,1,1,0,1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1,0,0,1,1,0,1,1,0,0,0,0,0,1,1,0,0,0,1,0,0,1,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,1,0,0,1,0,0,0,1,0,0,1,0,0,0,0,0,0,1,0,0,1,1,1,0,1,1,0,0,0,0,0,0,0,0,1,1,1,0,1,0,0,0,1,0,0,0,0,0,1,0,1,0,0,0,1,0,0,0,1,0,0,0,0,0,0,1,1,1,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,1,1,0,1,1,1,1];
        #[rustfmt::skip]
        let no_remainder_res12 = &[0,0,0,0,1,1,1,0,1,0,0,0,0,0,1,0,0,0,0,0,1,1,1,0,1,1,1,0,1,1,1,1,0,0,0,0,0,1,1,0,0,0,0,0,0,0,0,1,0,0,0,0,1,0,0,1,1,0,1,1,1,1,0,0,0,0,0,0,0,1,0,0,1,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,1,0,0,1,0,0,0,0,0,0,0,1,0,0,1,0,1,0,0,1,1,1,0,0,0,0,0,1,1,0,0,0,1,1,1,0,1,0,0,0,0,0,0,0,1,1,0,1,0,0,0,1,0,0,0,0,0,0,0,1,0,1,1,1,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,1,1,0,1,1,1,1];
        #[rustfmt::skip]
        let no_remainder_res13 = &[0,0,0,1,1,1,0,1,0,0,0,0,0,1,0,1,0,0,0,1,1,0,1,1,1,0,1,1,1,1,0,1,0,0,0,1,0,0,0,0,0,0,0,0,1,1,0,0,0,0,0,1,1,0,1,1,1,1,0,0,0,1,0,0,0,0,0,1,0,1,0,0,0,0,0,0,0,0,0,1,0,0,0,0,1,0,0,1,0,0,0,1,0,0,1,0,0,0,0,1,0,0,1,1,1,0,1,1,0,0,0,1,0,0,0,1,1,0,1,0,0,0,1,1,0,1,0,0,0,0,0,0,1,0,0,0,1,0,1,1,1,0,0,0,0,0,0,0,0,0,0,1,0,1,1,0,1,1,1,1];
        #[rustfmt::skip]
        let no_remainder_res14 = &[0,0,1,1,1,0,1,0,0,0,0,0,1,0,1,1,0,0,1,0,1,1,1,0,1,1,1,1,0,1,1,0,0,0,0,0,0,0,0,0,0,1,1,0,0,1,1,0,0,0,1,1,1,1,0,0,0,1,0,0,1,0,1,0,0,0,0,0,0,0,0,0,0,0,1,0,1,0,0,1,0,0,0,0,0,1,0,0,1,0,1,0,0,1,1,1,0,0,0,1,1,0,0,0,1,1,1,0,1,0,0,0,0,0,1,1,0,1,0,0,0,1,0,0,0,1,0,1,0,0,1,1,0,0,0,0,0,1,0,1,1,0,1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1];
        #[rustfmt::skip]
        let no_remainder_res15 = &[0,1,1,1,0,1,0,0,0,0,0,1,0,1,1,1,0,0,1,1,1,0,1,1,1,1,0,1,1,0,0,0,0,0,0,0,0,0,1,1,0,0,1,1,0,1,1,1,0,1,0,0,0,1,0,0,1,0,1,0,0,0,0,0,0,0,0,0,0,1,0,1,0,0,1,0,0,0,1,0,0,0,1,0,1,0,0,1,1,1,0,1,1,0,0,0,0,1,1,1,0,1,0,0,0,1,1,0,1,0,0,0,0,1,0,0,0,1,0,1,1,1,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,1,1,0,1,1,1,1];
        // NOTE: The original returns NaN for some bits:
        // I haven't looked into why.
        // let no_remainder_res16 = &[1,1,1,0,1,0,0,0,0,0,1,0,1,1,1,0,1,1,1,0,1,1,1,1,0,1,1,0,0,0,0,0,0,0,0,1,1,0,0,1,1,0,1,1,1,1,0,0,0,1,0,0,1,0,1,0,0,0,0,0,0,0,0,0,1,0,1,0,0,1,0,0,0,1,0,0,1,0,1,0,0,1,1,1,0,1,1,0,0,0,1,1,1,0,1,0,0,0,1,1,0,1,0,0,0,1,0,0,0,1,0,1,NaN,NaN,NaN,NaN,NaN,NaN,NaN,NaN,NaN,NaN,NaN,NaN,NaN,NaN,NaN,NaN,0,0,0,0,0,0,0,0];
        #[rustfmt::skip]
        let no_remainder_res16 = &[1,1,1,0,1,0,0,0,0,0,1,0,1,1,1,0,1,1,1,0,1,1,1,1,0,1,1,0,0,0,0,0,0,0,0,1,1,0,0,1,1,0,1,1,1,1,0,0,0,1,0,0,1,0,1,0,0,0,0,0,0,0,0,0,1,0,1,0,0,1,0,0,0,1,0,0,1,0,1,0,0,1,1,1,0,1,1,0,0,0,1,1,1,0,1,0,0,0,1,1,0,1,0,0,0,1,0,0,0,1,0,1,1,1,0,0,0,0,0,1,0,1,1,0,1,1,1,1,0,0,0,0,0,0,0,0];
        #[rustfmt::skip]
        let no_remainder_res17 = &[1,1,0,1,0,0,0,0,0,1,0,1,1,1,0,1,1,0,1,1,1,1,0,1,1,0,0,0,0,0,0,0,1,1,0,0,1,1,0,1,1,1,1,0,0,0,1,0,1,0,1,0,0,0,0,0,0,0,0,0,1,0,1,0,1,0,0,0,1,0,0,1,0,1,0,0,1,1,1,0,1,0,0,0,1,1,1,0,1,0,0,0,1,1,0,1,0,0,1,0,0,0,1,0,1,1,1,0,0,0,0,0,0,0,0,0,0,0,0,1,0,1,1,0,1,1,1,1];
        assert_vec_u8_ok_eq(no_remainder_res7, decode(&result7, None));
        assert_vec_u8_ok_eq(no_remainder_res8, decode(&result8, None));
        assert_vec_u8_ok_eq(no_remainder_res9, decode(&result9, None));
        assert_vec_u8_ok_eq(no_remainder_res10, decode(&result10, None));
        assert_vec_u8_ok_eq(no_remainder_res11, decode(&result11, None));
        assert_vec_u8_ok_eq(no_remainder_res12, decode(&result12, None));
        assert_vec_u8_ok_eq(no_remainder_res13, decode(&result13, None));
        assert_vec_u8_ok_eq(no_remainder_res14, decode(&result14, None));
        assert_vec_u8_ok_eq(no_remainder_res15, decode(&result15, None));
        assert_vec_u8_ok_eq(no_remainder_res16, decode(&result16, None));
        assert_vec_u8_ok_eq(no_remainder_res17, decode(&result17, None));

        // 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31
        let input: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, //
            0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1, //
            0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, //
            0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 1, //
            0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, //
            0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 1, //
            0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 1, //
            0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 1, 1, 1, 1, //
            0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, //
            0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 1, 1, //
            0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, //
            0, 0, 0, 1, 0, 1, 1, 0, 0, 0, 0, 1, 0, 1, 1, 1, //
            0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 1, //
            0, 0, 0, 1, 1, 0, 1, 0, 0, 0, 0, 1, 1, 0, 1, 1, //
            0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 1, 1, 1, 0, 1, //
            0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 1,
        ];
        let encoded = "\u{f0002}\u{f080c}\u{f2028}\u{f6070}\u{100121}\u{f82c3}\u{f0687}\u{f0f10}\u{f2224}\u{f4c50}\u{fa8b0}\u{107181}\u{102343}\u{fc707}\u{f8f0f}\u{f80c}";
        let res = assert_string_ok_eq(encoded, encode(&input, 17));
        crate::log!(res);
        assert_vec_u8_ok_eq(
            &input,
            decode(&res, Some(u32::try_from(input.len() % 17).unwrap())),
        );
    }
}
