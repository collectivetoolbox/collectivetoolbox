#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::{Result, anyhow, bail};

use ctb_formats_utf8::first_char_of_utf8_string;

/// Convert a UTF-8 byte slice into a vector of Unicode scalar codepoints (as i32),
/// analogous to utf8CharArrayFromByteArray in the original code.
pub fn utf8_char_array_from_byte_array(bytes: &[u8]) -> Result<Vec<u32>> {
    let mut out: Vec<u8> = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let slice = bytes
            .get(i..)
            .ok_or_else(|| anyhow!("Index {i} out of bounds for byte slice of len {}", bytes.len()))?;
        let (mut temp, consumed) = first_char_of_utf8_string(slice)?;
        out.append(&mut temp);
        i = i.saturating_add(consumed);
    }
    Ok(String::from_utf8(out)
        .context("first_char_of_utf8_string should produce valid UTF-8")?
        .chars()
        .map(u32::from)
        .collect())
}

/// Encode an array of Unicode codepoints (as u32) into UTF-8 bytes.
/// (byteArrayFromUtf8CharArray in original.)
pub fn byte_array_from_utf8_char_array(codepoints: &[u32]) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    for &cp in codepoints {
        if let Some(ch) = std::char::from_u32(cp) {
            let mut buf = [0u8; 4];
            let encoded = ch.encode_utf8(&mut buf);
            out.extend_from_slice(encoded.as_bytes());
        } else {
            return Err(anyhow!("Invalid Unicode scalar value: {cp}"));
        }
    }
    Ok(out)
}

pub fn unicode_scalar_from_utf8(bytes: &[u8]) -> Result<u32> {
    let (codepoint, len) = first_utf8_codepoint(bytes)?;

    if len > bytes.len() {
        Err(anyhow!("This function is for a single character"))
    } else {
        Ok(codepoint)
    }
}

/// Helper: decode first UTF-8 codepoint (or raw byte) returning (codepoint, `byte_len`).
pub fn first_utf8_codepoint(bytes: &[u8]) -> Result<(u32, usize)> {
    if bytes.is_empty() {
        return Ok((0, 0));
    }
    // Try valid UTF-8 for at least the first char.
    for end in 1..=bytes.len().min(4) {
        let slice = bytes
            .get(..end)
            .ok_or_else(|| anyhow!("End index {end} out of bounds"))?;
        if let Ok(s) = std::str::from_utf8(slice) {
            if let Some(ch) = s.chars().next() {
                return Ok((u32::from(ch), ch.len_utf8()));
            }
        }
    }
    // Fallback: treat first byte as standalone.
    bail!("Invalid UTF-8 sequence")
}

/// Helper: decode last UTF-8 codepoint (or raw byte) returning (codepoint, `byte_len`).
#[allow(
    clippy::expect_used,
    reason = "non-empty bytes check at function start guarantees len >= 1, so index len - 1 is in bounds"
)]
pub fn last_utf8_codepoint(bytes: &[u8]) -> (u32, usize) {
    if bytes.is_empty() {
        return (0, 0);
    }
    // Scan backwards up to 4 bytes.
    let len = bytes.len();
    for start in (0.max(len.saturating_sub(4))..len).rev() {
        if let Some(slice) = bytes.get(start..) {
            if let Ok(s) = std::str::from_utf8(slice) {
                if let Some(ch) = s.chars().next() {
                    return (u32::from(ch), ch.len_utf8());
                }
            }
        }
    }
    // Fallback: last byte.
    let last_byte = *bytes
        .get(len.saturating_sub(1))
        .expect("non-empty bytes check guarantees len - 1 is in bounds");
    (u32::from(last_byte), 1)
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
    use const_default::ConstDefault;

    use crate::formats::utf8::UTF8FormatSettings;
    use crate::formats::utf8::{dca_from_utf8, dca_to_utf8};
    use crate::utilities::{assert_vec_u8_ok_eq, assert_vec_u32_ok_eq};
    use ctb_formats_utilities::{
        assert_vec_u8_ok_eq_no_warnings, assert_vec_u32_ok_eq_no_warnings,
    };

    use super::*;

    const SETTINGS: UTF8FormatSettings =
        <UTF8FormatSettings as ConstDefault>::DEFAULT;

    #[crate::ctb_test]
    fn test_utf8_char_array_conversion() {
        let s = "hé🙂";
        let bytes = s.as_bytes();
        let cps = utf8_char_array_from_byte_array(bytes).expect("decode cps");
        let re = byte_array_from_utf8_char_array(&cps).expect("encode bytes");
        assert_eq!(re, bytes);
    }

    #[crate::ctb_test]
    fn test_format_utf8_conversions() {
        // /* FIXME: Update tests for new remainder character format. */
        // dcaFromUtf8([ 49, 32, 50 ]) -> [ 35, 18, 36 ]
        assert_vec_u32_ok_eq_no_warnings(
            &[35, 18, 36],
            dca_from_utf8(&[49, 32, 50], &SETTINGS),
        );

        // dcaToUtf8([ 35, 18, 36 ]) -> [ 49, 32, 50 ]
        assert_vec_u8_ok_eq_no_warnings(
            &[49, 32, 50],
            dca_to_utf8(&[35, 18, 36], &SETTINGS),
        );
    }

    #[crate::ctb_test]
    fn test_utf8_byte_array_conversions_work() {
        // utf8CharArrayFromByteArray
        let utf8_bytes = [
            50, 53, 54, 32, 50, 53, 56, 32, 50, 54, 48, 32, 50, 54, 50, 32, 50,
            54, 52, 32, 50, 54, 51, 32, 53, 55, 32, 56, 54, 32, 57, 51, 32, 57,
            51, 32, 57, 54, 32, 51, 48, 32, 49, 56, 32, 50, 56, 54, 32, 55, 50,
            32, 57, 54, 32, 57, 57, 32, 57, 51, 32, 56, 53, 32, 50, 56, 55, 32,
            49, 57, 32, 49, 56, 32, 50, 56, 52, 32, 50, 54, 49, 32, 50, 53, 57,
            32, 35, 32, 115, 97, 121, 32, 34, 72, 101, 108, 108, 111, 44, 32,
            47, 87, 111, 114, 108, 100, 47, 33, 32, 226, 154, 189, 34, 10, 49,
            32, 50, 32, 35, 32, 226, 154, 189, 10,
        ];
        let expected_codepoints = [
            50, 53, 54, 32, 50, 53, 56, 32, 50, 54, 48, 32, 50, 54, 50, 32, 50,
            54, 52, 32, 50, 54, 51, 32, 53, 55, 32, 56, 54, 32, 57, 51, 32, 57,
            51, 32, 57, 54, 32, 51, 48, 32, 49, 56, 32, 50, 56, 54, 32, 55, 50,
            32, 57, 54, 32, 57, 57, 32, 57, 51, 32, 56, 53, 32, 50, 56, 55, 32,
            49, 57, 32, 49, 56, 32, 50, 56, 52, 32, 50, 54, 49, 32, 50, 53, 57,
            32, 35, 32, 115, 97, 121, 32, 34, 72, 101, 108, 108, 111, 44, 32,
            47, 87, 111, 114, 108, 100, 47, 33, 32, 9917, 34, 10, 49, 32, 50,
            32, 35, 32, 9917, 10,
        ];
        assert_vec_u32_ok_eq(
            &expected_codepoints,
            utf8_char_array_from_byte_array(&utf8_bytes),
        );

        // byteArrayFromUtf8CharArray (round trip)
        assert_vec_u8_ok_eq(
            &utf8_bytes,
            byte_array_from_utf8_char_array(&expected_codepoints),
        );
    }
}
