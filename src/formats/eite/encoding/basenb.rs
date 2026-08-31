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

//! Basenb encoding/decoding: pack binary data into Unicode PUA.
//!
//! Basenb is a way of encoding arbitrary binary data into a compact string
//! representation that can be embedded in Unicode text, as runs of Unicode
//! private-use characters.
//!
//! It's a modified version of Base16b that additionally encodes a "remainder"
//! length in a trailing character, which seems to be needed to reliably
//! round-trip values.
//!
//! Actually using it requires some sort of protocol for when to switch between
//! basenb and regular PUA characters. dcBasenb addresses that by encoding UUIDs
//! and using them as in-band start/end markers. (Round-tripping a UTF-8 file
//! that included dcBasenb UUIDs, for instance in reference to them, would
//! probably only be possible by encoding the UUIDs as UTF-8 encapsulated within
//! the Dcs, and being careful with the encode/decode settings.)
//!
//! dcBasenb is a way of encoding arbitrary Dcs into runs of Unicode private-use
//! characters; see dcbasenb.rs

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::{Result, anyhow, bail};

use crate::encoding::pack32::{is_pack32_char, pack32, unpack32};
use crate::util::array::subset;
use crate::util::bitwise::{
    byte_array_from_int_bit_array, byte_array_to_int_bit_array,
};
use crate::util::math::int_is_between_u32;
use crate::{bail_if_none, log};
use ctb_formats_base16b as base16b;

/// FIXME UNIMPLEMENTED
pub const ARMORED_BASE17B_UTF8_START_UUID_BYTES: [u8; 32] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

/// FIXME UNIMPLEMENTED
pub const ARMORED_BASE17B_UTF8_END_UUID_BYTES: [u8; 32] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

/// internalIntBitArrayToBasenbString(intBase, bytes)
/// Returns UTF-8 bytes of the encoded string (mirroring JS returning a byte array).
pub fn int_bit_array_to_basenb_no_remainder_marker(
    base: u32,
    input: &[u8],
) -> Result<Vec<u8>> {
    let encoded = base16b::encode(input, base)?;

    Ok(encoded.into_bytes())
}

/// internalIntBitArrayFromBasenbString(byteArrayInput, intRemainder)
/// JS passes a `Uint8Array` of UTF-8 bytes and an int remainder
pub fn int_bit_array_from_basenb_string(
    input_bytes: &[u8],
    remainder_len: Option<u32>,
) -> Result<Vec<u8>> {
    let s = std::str::from_utf8(input_bytes).map_err(|e| {
        anyhow!(
            "utf8 error on input {:?}, from_lossy {:?}: {e}",
            input_bytes,
            String::from_utf8_lossy(input_bytes)
        )
    })?;
    log!(
        "Decoding with input str {:?}, remainder {:?}",
        s,
        remainder_len
    );
    base16b::decode(s, remainder_len)
}

/// Is the provided base valid for Basenb? (Original: 7 through 17 inclusive.)
pub fn is_basenb_base(base: u32) -> bool {
    (7..=17).contains(&base)
}

/// True if the pack32 character represents a Basenb character codepoint.
///
/// The Basenb character ranges):
///   - 983040 ..= 1048573: U+F0000 to U+FFFFD
///   - 1048576 ..= 1114109: U+100000 to U+10FFFD
///   - 63481  ..= 63501   (special / remainder markers): U+F7F9 to U+F80D
///
/// Distinct remainder markers subset: 63481 ..= 63497: U+F7F9 to U+F809
pub fn is_basenb_char(packed_char: &[u8]) -> bool {
    if !is_pack32_char(packed_char) {
        return false;
    }
    if let Ok(cp) = unpack32(packed_char) {
        // U+F0000 to U+FFFFD
        if int_is_between_u32(cp, 983_040, 1_048_573) {
            return true;
        }
        // U+100000 to U+10FFFD
        if int_is_between_u32(cp, 1_048_576, 1_114_109) {
            return true;
        }
        // U+F7F9 to U+F80D. Remainders are U+F7F9 to U+F809; U+F80A to U+F80D
        // are used for Base16b and Base17b.
        if int_is_between_u32(cp, 63_481, 63_501) {
            return true;
        }
    }
    false
}

/// True if the pack32 character is one of the distinct remainder markers
/// (63481..=63497).
pub fn is_basenb_distinct_remainder_char(packed_char: &[u8]) -> bool {
    if !is_pack32_char(packed_char) {
        return false;
    }
    if let Ok(cp) = unpack32(packed_char) {
        // U+F7F9 to U+F809
        return int_is_between_u32(cp, 63_481, 63_497);
    }
    false
}

pub fn byte_array_to_basenb_no_remainder_marker(
    base: u32,
    input: &[u8],
) -> Result<Vec<u8>> {
    if !is_basenb_base(base) {
        return Err(anyhow!(
            "byte_array_to_basenb_no_remainder_marker: invalid base {base}, expected 7..=17"
        ));
    }
    let bit_array = byte_array_to_int_bit_array(input);
    let encoded =
        int_bit_array_to_basenb_no_remainder_marker(base, &bit_array)?;
    Ok(encoded)
}

/// Encode a raw byte array into Basenb (UTF-8 sequence of pack32 codepoints).
///
/// Steps:
/// 1. Convert bytes to bit array.
/// 2. Encode bit array via `int_bit_array_to_basenb_string`.
/// 3. Append remainder length marker: pack32(63497 - (`bit_len` % 17)).
///    (Matches the original implementation’s workaround re: remainder storage.)
///
/// Returns the full UTF-8 (actually just raw bytes containing appended pack32
/// code units) representing the Basenb encoding.
pub fn byte_array_to_basenb_utf8(base: u32, input: &[u8]) -> Result<Vec<u8>> {
    let mut encoded = byte_array_to_basenb_no_remainder_marker(base, input)?;
    /* Remainder marker. The remainder length also needs to be stored, to be able to decode successfully. We'll calculate, encode, and append it. It's always 4 bytes, 1 UTF-8 character, and 2 UTF-16 characters long, after encoding (it has 2 added to it to make it always be the same byte length and UTF-16 length; this must be subtracted before passing it to the Base16b.decode function). */
    let total_bits = input.len().checked_mul(8).ok_or_else(|| {
        anyhow!("Input length is too large to represent bits")
    })?;
    let base_usize = usize::try_from(base)?;
    let remainder = total_bits
        .checked_rem(base_usize)
        .ok_or_else(|| anyhow!("Base cannot be zero"))?;
    // Start with U+F809, and subtract the remainder to find the codepoint.
    let remainder_u32 = u32::try_from(remainder)?;
    let codepoint = 63_497_u32.checked_sub(remainder_u32).ok_or_else(|| {
        anyhow!("Remainder too large for codepoint calculation")
    })?;
    encoded.extend(pack32(codepoint)?);
    Ok(encoded)
}

/// Sentinel UUID returned by the legacy JS implementation to indicate an invalid
/// basenb UTF-8 decode input (only remainder char present or incomplete data).
/// UUID: 3362daa3-1705-40ec-9a97-59d052fd4037
pub const BYTE_ARRAY_FROM_BASENB_UTF8_INVALID_INPUT_EXCEPTION_BYTES: [u8; 16] = [
    51, 98, 218, 163, 23, 5, 64, 236, 154, 151, 89, 208, 82, 253, 64, 55,
];

/// Decode a Basenb UTF-8 byte sequence into the original byte array.
///
/// This replicates the JS `byteArrayFromBasenbUtf8(intArrayIn)` logic:
/// - Determines the encoded remainder-length indicator (either a distinct
///   3‑byte remainder char or a generic 4‑byte packed char).
/// - For a distinct remainder char (3 bytes): remainder = 63497 - unpack32
///   (last3)
/// - For a generic (4 bytes) remainder char: decode that char as an 8‑bit
///   value, then `remainder = decoded_byte - 2`
/// - If the full input length is exactly (or smaller than) the remainder char
///   length, returns the sentinel UUID bytes to indicate invalid input (legacy
///   behavior).
///
/// Remainder length (in bits) is passed to `int_bit_array_from_basenb_string`
/// which reconstructs the concatenated bit array; that is then converted back
/// to bytes.
///
/// Returns:
/// - `Ok(Vec<u8>)` with decoded bytes (or sentinel bytes if invalid input).
pub fn byte_array_from_basenb_utf8(input: &[u8]) -> Result<Vec<u8>> {
    /* Extract remainder length */
    let remainder: u32;
    /* last 3 bytes (1 character), which represent the remainder */
    let mut remainder_arr: Vec<u8>;
    remainder_arr = bail_if_none!(subset(input, -3, -1)?);
    if is_basenb_distinct_remainder_char(&remainder_arr) {
        let raw_remainder = unpack32(&remainder_arr)?;
        remainder = 63497_u32.checked_sub(raw_remainder).ok_or_else(|| {
            anyhow!("Underflow in basenb remainder calculation")
        })?;
    } else {
        /* last 4 bytes (1 character), which represent the remainder */
        remainder_arr = bail_if_none!(subset(input, -4, -1)?);
        let remainder_decoded: Vec<u8> = byte_array_from_int_bit_array(
            &int_bit_array_from_basenb_string(&remainder_arr, Some(8))?,
        )?;
        let temp: &u8 = bail_if_none!(remainder_decoded.first());
        let temp: i16 = bail_if_none!(i16::from(*temp).checked_add(-2));
        remainder = u32::try_from(temp)?;
    }
    if input.len() <= remainder_arr.len() {
        // Mirrors legacy path: only a (missing) remainder => sentinel.
        return Ok(
            BYTE_ARRAY_FROM_BASENB_UTF8_INVALID_INPUT_EXCEPTION_BYTES.to_vec()
        );
    }
    let len_i64 = i64::try_from(remainder_arr.len())?;
    let subset_end = len_i64
        .checked_neg()
        .and_then(|val| val.checked_sub(1))
        .ok_or_else(|| {
        anyhow!("Subset end calculation overflow/underflow")
    })?;

    let subset = bail_if_none!(subset(input, 0, subset_end)?);

    log!("Getting bits from subset: {:?}, {:?}", &subset, remainder);

    let bits = &int_bit_array_from_basenb_string(&subset, Some(remainder))?;
    log!("Bits from decoder: {:?}", &bits);
    byte_array_from_int_bit_array(bits)
}

/// Convenience wrapper (encode bytes to Basenb 17 UTF-8 representation).
///
/// JS original: `byteArrayToBase17bUtf8(intArrayIn)` calling `byteArrayToBasenbUtf8(17, ...)`.
/// Renamed to try to make it clearer that this isn't the original base17b
/// format.
pub fn byte_array_to_basenb_17_utf8(input: &[u8]) -> Result<Vec<u8>> {
    byte_array_to_basenb_utf8(17, input)
}

/// Convenience wrapper (decode Basenb  17 UTF-8 representation).
///
/// JS original simply forwarded to `byteArrayFromBasenbUtf8`.
pub fn byte_array_from_basenb_17_utf8(input: &[u8]) -> Result<Vec<u8>> {
    byte_array_from_basenb_utf8(input)
}

// “Armored” Base17b UTF-8 helpers.

/// Produce an “armored” Base17b UTF-8 run, encoding arbitrary binary data:
///   armored = `start_uuid` || `base17b_encode(bytes)` || `end_uuid`
///
/// This mirrors the original JS `byteArrayToArmoredBase17bUtf8`.
/// FIXME Unimplemented/untested!
pub fn byte_array_to_armored_base17b_utf8(input: &[u8]) -> Result<Vec<u8>> {
    // Encode payload to Base17b UTF-8 (already provided in earlier translation).
    let encoded = byte_array_to_basenb_17_utf8(input)?;
    let mut out = ARMORED_BASE17B_UTF8_START_UUID_BYTES.to_vec();
    out.extend(encoded);
    out.extend(ARMORED_BASE17B_UTF8_END_UUID_BYTES);
    Ok(out)
}

/// Decode an armored Base17b UTF-8 run to a byte array. FIXME untested!
pub fn byte_array_from_armored_base17b_utf8(input: &[u8]) -> Result<Vec<u8>> {
    let start = ARMORED_BASE17B_UTF8_START_UUID_BYTES;
    let end = ARMORED_BASE17B_UTF8_END_UUID_BYTES;

    let min_len = start
        .len()
        .checked_add(end.len())
        .ok_or_else(|| anyhow!("Overflow calculating min_len"))?;
    if input.len() < min_len {
        bail!(
            "Armored Base17b input too short: {} < required framing {}",
            input.len(),
            min_len
        );
    }

    if !input.starts_with(&start) {
        bail!("Armored Base17b input missing or corrupt start UUID marker");
    }
    if !input.ends_with(&end) {
        bail!("Armored Base17b input missing or corrupt end UUID marker");
    }

    let inner_len = input
        .len()
        .checked_sub(start.len())
        .and_then(|val| val.checked_sub(end.len()))
        .ok_or_else(|| {
            anyhow!("Input length underflow relative to start/end delimiters")
        })?;
    let end_idx = start
        .len()
        .checked_add(inner_len)
        .ok_or_else(|| anyhow!("End index overflow"))?;
    let inner = input
        .get(start.len()..end_idx)
        .ok_or_else(|| anyhow!("Input slice out of bounds"))?;
    // Decode the inner Base17b UTF-8 segment back to raw bytes.
    let decoded = byte_array_from_basenb_17_utf8(inner)?;
    Ok(decoded)
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

    use crate::formats::dcbasenb::DC_BASENB_EMBEDDED_START_BYTES;
    use crate::utilities::{assert_vec_u8_eq, assert_vec_u8_ok_eq};
    use crate::{
        encoding::pack32::pack32, util::bitwise::byte_array_to_int_bit_array,
    };

    use super::*;

    #[crate::ctb_test]
    fn test_is_basenb_base() {
        for b in 0..=30 {
            let valid = is_basenb_base(b);
            if (7..=17).contains(&b) {
                assert!(valid);
            } else {
                assert!(!valid);
            }
        }
    }

    #[crate::ctb_test]
    fn test_basenb_remainder_marker_range() {
        // Distinct remainder chars subset 63481..=63497
        for cp in 63_480..=63_500 {
            let packed = pack32(cp).unwrap();
            if (63_481..=63_497).contains(&cp) {
                assert!(is_basenb_distinct_remainder_char(&packed));
            } else {
                assert!(!is_basenb_distinct_remainder_char(&packed));
            }
        }
    }

    #[crate::ctb_test]
    fn test_basenb_char_ranges() {
        // sample points
        for cp in [
            63_480, 63_481, 63_495, 63_501, 63_502, 983_040, 983_100,
            1_048_573, 1_048_574,
        ] {
            let packed = pack32(cp).unwrap();
            let is_char = is_basenb_char(&packed);
            let expected = (63_481..=63_501).contains(&cp)
                || (983_040..=1_048_573).contains(&cp)
                || (1_048_576..=1_114_109).contains(&cp);
            assert_eq!(
                is_char, expected,
                "cp={} expected {} got {}",
                cp, expected, is_char
            );
        }
    }

    #[crate::ctb_test]
    fn test_byte_array_to_basenb_utf8_remainder_marker() {
        // The remainder length also needs to be stored, to be able to decode successfully. We'll calculate, encode, and append it. It's always 4 bytes, 1 UTF-8 character, and 2 UTF-16 characters long, after encoding (it has 2 added to it to make it always be the same byte length and UTF-16 length; this must be subtracted before passing it to the Base16b.decode function).
        // Known: remainder marker = 63497 - (bit_len % 17)
        let data = b"\xAB\xCD"; // 16 bits
        let bits = byte_array_to_int_bit_array(data);
        assert_eq!(bits.len(), 16);
        let base = 10;
        let encoded_with_remainder =
            byte_array_to_basenb_utf8(base, data).unwrap();
        let encoded_b10b =
            byte_array_to_basenb_no_remainder_marker(base, data).unwrap();
        assert!(
            encoded_with_remainder.len() >= 4,
            "Expected at least one codepoint + remainder marker"
        );
        let len_diff = encoded_with_remainder
            .len()
            .checked_sub(encoded_b10b.len())
            .expect("encoded_with_remainder is shorter than encoded_b10b");
        let start_idx = encoded_with_remainder
            .len()
            .checked_sub(len_diff)
            .expect("len_diff is greater than encoded_with_remainder length");
        let lastn = &encoded_with_remainder[start_idx..];
        assert!(is_basenb_distinct_remainder_char(lastn));
        let cp = unpack32(lastn).unwrap();
        let bits_len_u32 =
            u32::try_from(bits.len()).expect("Could not fit length in u32");
        let expected_rem =
            bits_len_u32.checked_rem(base).expect("base is zero");
        let expected = 63_497_u32
            .checked_sub(expected_rem)
            .expect("Remainder too large for expected codepoint");
        assert_eq!(cp, expected);
        assert_vec_u8_ok_eq(
            data,
            byte_array_from_basenb_utf8(&encoded_with_remainder),
        );
    }

    #[crate::ctb_test]
    fn test_byte_array_to_basenb_utf8_invalid_base() {
        let data = b"abc";
        assert!(byte_array_to_basenb_utf8(6, data).is_err());
        assert!(byte_array_to_basenb_utf8(18, data).is_err());
    }

    fn is_sentinel(bytes: &[u8]) -> bool {
        bytes == BYTE_ARRAY_FROM_BASENB_UTF8_INVALID_INPUT_EXCEPTION_BYTES
    }

    #[crate::ctb_test]
    fn test_decode_empty_input() {
        let enc = assert_vec_u8_ok_eq(
            "\u{f80d}\u{f809}".as_bytes(),
            byte_array_to_basenb_utf8(17, &[]),
        );
        let dec = byte_array_from_basenb_utf8(&enc);
        assert_vec_u8_ok_eq(&[], dec);
    }

    #[crate::ctb_test]
    fn test_decode_invalid_only_remainder() {
        let dec = byte_array_from_basenb_utf8("\u{f809}".as_bytes()).unwrap();
        assert!(
            is_sentinel(&dec),
            "Expected sentinel for empty input decode; got {:?}",
            dec
        );
    }

    #[crate::ctb_test]
    fn test_round_trip_base17_small_samples() {
        let samples: Vec<Vec<u8>> = vec![
            vec![0u8],
            vec![1, 2, 3],
            vec![255],
            b"hello".to_vec(),
            b"\x00\x01\x02\x03\xFE\xFF".to_vec(),
            (0u8..32u8).collect(),
        ];

        for sample in samples {
            let enc = byte_array_to_basenb_17_utf8(&sample).unwrap();
            crate::log!(
                "Sample {:?} encoded to Basenb 17 UTF-8 bytes: {:?}",
                sample.clone(),
                enc.clone()
            );
            let dec = byte_array_from_basenb_17_utf8(&enc).unwrap();
            if is_sentinel(&dec) && !sample.is_empty() {
                panic!(
                    "Unexpected sentinel for non-empty sample {:?} (encoded {:?})",
                    sample, enc
                );
            }
            if !sample.is_empty() {
                assert_vec_u8_eq(&sample, &dec);
            }
        }
    }

    #[crate::ctb_test]
    fn test_basenb_encode_uuid() {
        // e82eef60-19bc-4a00-a44a-763a3445c16f
        let input: Vec<u8> = vec![
            0xe8, 0x2e, 0xef, 0x60, //
            0x19, 0xbc, 0x4a, 0x00, //
            0xa4, 0x4a, 0x76, 0x3a, //
            0x34, 0x45, 0xc1, 0x6f,
        ];

        // Working out the remainder byte by hand:
        // 16 bytes * 8 bits = 128 bits
        // 128 % 17 = 9
        // The last remainder codepoint is U+F809, - 9 = U+F800.

        let expected_uuid = DC_BASENB_EMBEDDED_START_BYTES.to_vec();
        let remainder = "\u{F800}".as_bytes().to_vec();
        let expected = [expected_uuid, remainder].concat();

        let enc = assert_vec_u8_ok_eq(
            &expected,
            byte_array_to_basenb_17_utf8(&input),
        );

        let dec = byte_array_from_basenb_17_utf8(&enc).unwrap();
        if is_sentinel(&dec) && !input.is_empty() {
            panic!(
                "Unexpected exception UUID for {:?} (encoded {:?})",
                input, enc
            );
        }

        assert_vec_u8_eq(&input, &dec);
    }

    #[crate::ctb_test]
    fn test_armored_round_trip() {
        let payload = b"Hello Base17b Armored!";
        let armored = byte_array_to_armored_base17b_utf8(payload).unwrap();
        // Basic framing checks
        let start = ARMORED_BASE17B_UTF8_START_UUID_BYTES;
        let end = ARMORED_BASE17B_UTF8_END_UUID_BYTES;
        assert!(armored.starts_with(&start));
        assert!(armored.ends_with(&end));

        let decoded = byte_array_from_armored_base17b_utf8(&armored);
        assert_vec_u8_ok_eq(payload, decoded);
    }

    #[crate::ctb_test]
    fn test_armored_invalid_prefix() {
        let payload = b"xyz";
        let mut armored = byte_array_to_armored_base17b_utf8(payload).unwrap();
        // Corrupt first byte
        armored[0] ^= 0xFF;
        let err = byte_array_from_armored_base17b_utf8(&armored).unwrap_err();
        assert!(
            err.to_string().contains("missing or corrupt start UUID"),
            "Unexpected error: {err}"
        );
    }

    #[crate::ctb_test]
    fn test_armored_invalid_suffix() {
        let payload = b"xyz";
        let mut armored = byte_array_to_armored_base17b_utf8(payload).unwrap();
        // Corrupt last byte
        let last = armored
            .len()
            .checked_sub(1)
            .expect("armored array is empty");
        armored[last] ^= 0xAA;
        let err = byte_array_from_armored_base17b_utf8(&armored).unwrap_err();
        assert!(
            err.to_string().contains("missing or corrupt end UUID"),
            "Unexpected error: {err}"
        );
    }

    #[crate::ctb_test]
    fn test_armored_too_short() {
        let data: Vec<u8> = vec![1, 2, 3, 4]; // shorter than any plausible framing
        let err = byte_array_from_armored_base17b_utf8(&data).unwrap_err();
        assert!(
            err.to_string().contains("too short"),
            "Unexpected error: {err}"
        );
    }
}
