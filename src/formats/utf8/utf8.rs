#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use anyhow::{Result, anyhow, bail};

pub const UTF8_REPLACEMENT_CHARACTER: &[u8; 3] = &[0xEF, 0xBF, 0xBD];

/// Truncate a string to a maximum byte length, respecting UTF-8 boundaries.
#[expect(
    clippy::expect_used,
    reason = "end is guaranteed to be a valid char boundary within s"
)]
pub fn truncate_to_max_bytes(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        return s.to_owned();
    }
    // Find the last valid char boundary at or before max_len.
    let mut end = max_len;
    while end > 0 && !s.is_char_boundary(end) {
        end = end.saturating_sub(1);
    }
    s.get(..end).expect("end is a valid char boundary within s").to_owned()
}

/// Ellipsize a string to a maximum byte length, respecting UTF-8 boundaries.
pub fn ellipsize_to_max_bytes(s: &str, max_len: usize) -> String {
    let start_len = s.len();
    if start_len <= max_len {
        return s.to_owned();
    }
    let max_len = max_len.saturating_sub(3); // for "..."
    format!("{}...", truncate_to_max_bytes(s, max_len))
}

/// Roughly transliterate a UTF-8 string to ASCII.
pub fn to_ascii_translit(input: &str) -> String {
    deunicode::deunicode(input)
}

/// Return a character (or for invalid part of the string, the replacement
/// character) of a UTF-8 string as bytes, as well as the number of input bytes
/// consumed. For valid UTF-8 input, the number of input bytes will always match
/// the number of output bytes.
pub fn first_char_of_utf8_string(bytes: &[u8]) -> Result<(Vec<u8>, usize)> {
    let (bytes, (consumed, _valid)) = _first_char_of_utf8_string(bytes, true)?;
    Ok((bytes, consumed))
}

/// Return a character or invalid part of a UTF-8 string as bytes, as well as
/// the number of input bytes consumed.
pub fn first_char_of_utf8_string_lossless(
    bytes: &[u8],
) -> Result<(Vec<u8>, (usize, bool))> {
    _first_char_of_utf8_string(bytes, false)
}

fn _first_char_of_utf8_string(
    bytes: &[u8],
    replace_invalid: bool,
) -> Result<(Vec<u8>, (usize, bool))> {
    // This is inefficient because it operates on the WHOLE string, meaning it
    // will decode all valid characters in a single chunk, then all but the
    // first is discarded. Could possibly trim the string to the maximum length
    // of a valid character to make it faster, but I don't know if that might
    // have side effects on how many replacement characters would be returned
    // compared to using String from_utf8_lossy(). It could also be made more efficient by returning a Cow slice into the input buffer instead of a Vec.
    if bytes.is_empty() {
        return Err(anyhow!("Empty input in first_char_of_utf8_string"));
    }
    let mut iter = bytes.utf8_chunks();
    let chunk = iter.next().ok_or_else(|| {
        anyhow!("At least some chunk should be found for non-empty string")
    })?;

    let valid = chunk.valid();
    if valid.is_empty() {
        let invalid = chunk.invalid();
        if !invalid.is_empty() {
            if replace_invalid {
                // Return replacement character for invalid sequence
                return Ok((vec![0xEF, 0xBF, 0xBD], (invalid.len(), false)));
            }
            return Ok((invalid.to_vec(), (invalid.len(), false)));
        }
        let first_char = valid
            .chars()
            .next()
            .context("Chunk contains no valid character")?;
        let out = &mut [0u8; 4];
        let first_char_len = first_char.encode_utf8(out).len();

        let char_bytes = out
            .get(..first_char_len)
            .context("Invalid character length")?
            .to_vec();
        return Ok((char_bytes, (first_char_len, true)));
    }

    bail!("Chunk contained neither valid nor invalid data")
}

pub fn utf8_from_scalar(cp: u32) -> Result<Vec<u8>> {
    if cp > 0x10FFFF {
        bail!("Invalid Unicode codepoint U+{cp:X}");
    }
    let mut buf = [0u8; 4];
    let s = char::from_u32(cp)
        .ok_or_else(|| anyhow!("Invalid Unicode codepoint U+{cp:X}"))?
        .encode_utf8(&mut buf);
    Ok(s.as_bytes().to_vec())
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
    fn test_truncate_to_max_bytes() {
        assert_eq!(truncate_to_max_bytes("hello", 10), "hello");
        assert_eq!(truncate_to_max_bytes("hello", 5), "hello");
        assert_eq!(truncate_to_max_bytes("hello", 3), "hel");

        // UTF-8 boundary handling.
        let s = "日本語";
        // Each Japanese char is 3 bytes. "日本語" = 9 bytes.
        assert_eq!(truncate_to_max_bytes(s, 9), "日本語");
        assert_eq!(truncate_to_max_bytes(s, 8), "日本");
        assert_eq!(truncate_to_max_bytes(s, 7), "日本");
        assert_eq!(truncate_to_max_bytes(s, 6), "日本");
        assert_eq!(truncate_to_max_bytes(s, 3), "日");
        assert_eq!(truncate_to_max_bytes(s, 2), "");
        assert_eq!(truncate_to_max_bytes(s, 0), "");
    }

    #[crate::ctb_test]
    fn test_first_char_of_utf8_string_ascii() {
        let input = b"hello";
        let (ch, consumed) = first_char_of_utf8_string(input).unwrap();
        assert_eq!(ch, vec![b'h']);
        assert_eq!(consumed, 1);
    }

    #[crate::ctb_test]
    fn test_first_char_of_utf8_string_multibyte() {
        let input = "éclair".as_bytes();
        let (ch, consumed) = first_char_of_utf8_string(input).unwrap();
        assert_eq!(ch, "é".as_bytes());
        assert_eq!(consumed, "é".len());
    }

    #[crate::ctb_test]
    fn test_first_char_of_utf8_string_astral() {
        let input = "🥴test".as_bytes();
        let (ch, consumed) = first_char_of_utf8_string(input).unwrap();
        assert_eq!(ch, "🥴".as_bytes());
        assert_eq!(consumed, "🥴".len());
    }

    #[crate::ctb_test]
    fn test_first_char_of_utf8_string_invalid() {
        let input = &[0xFF, 0x61, 0x62];
        let (ch, consumed) = first_char_of_utf8_string(input).unwrap();
        assert_eq!(ch, vec![0xEF, 0xBF, 0xBD]); // UTF-8 replacement character
        assert_eq!(consumed, 1);
    }

    #[crate::ctb_test]
    fn test_first_char_of_utf8_string_overlong() {
        let input = &[0xC1, 0x81];
        let (ch, consumed) = first_char_of_utf8_string(input).unwrap();
        assert_eq!(ch, vec![0xEF, 0xBF, 0xBD]); // UTF-8 replacement character
        assert_eq!(consumed, 1);
    }

    #[crate::ctb_test]
    fn test_first_char_of_utf8_string_partly_invalid() {
        let input = &[0xE2, 0x80, 0xA9, 0xFF, 0x61, 0x62];
        let (ch, consumed) = first_char_of_utf8_string(input).unwrap();
        assert_eq!(ch, vec![0xE2, 0x80, 0xA9]);
        assert_eq!(consumed, 3);
    }

    #[crate::ctb_test]
    fn test_first_char_of_utf8_string_empty() {
        let input = b"";
        let result = first_char_of_utf8_string(input);
        result.unwrap_err();
    }

    #[crate::ctb_test]
    fn test_first_char_of_utf8_string_only_invalid() {
        let input = &[0xFF, 0xFE];
        let (ch, consumed) = first_char_of_utf8_string(input).unwrap();
        assert_eq!(ch, vec![0xEF, 0xBF, 0xBD]);
        assert_eq!(consumed, 1);
    }

    #[crate::ctb_test]
    fn test_first_char_of_utf8_string_lossless_valid() {
        let input = &[0x61, 0x62];
        let (ch, (consumed, valid)) =
            first_char_of_utf8_string_lossless(input).unwrap();
        assert_eq!(ch, vec![0x61]);
        assert_eq!(consumed, 1);
        assert!(valid);
    }

    #[crate::ctb_test]
    fn test_first_char_of_utf8_string_lossless_multibyte() {
        let input = "🥴test".as_bytes();
        let (ch, (consumed, valid)) =
            first_char_of_utf8_string_lossless(input).unwrap();
        assert_eq!(ch, "🥴".as_bytes());
        assert_eq!(consumed, "🥴".len());
        assert!(valid);
    }

    #[crate::ctb_test]
    fn test_first_char_of_utf8_string_lossless_invalid() {
        let input = &[0xFF, 0x61, 0x62];
        let (ch, (consumed, valid)) =
            first_char_of_utf8_string_lossless(input).unwrap();
        assert_eq!(ch, vec![0xFF]); // UTF-8 replacement character
        assert_eq!(consumed, 1);
        assert!(!valid);
    }

    #[crate::ctb_test]
    fn test_first_char_of_utf8_string_lossless_only_invalid() {
        let input = &[0xFF, 0xFE];
        let (ch, (consumed, valid)) =
            first_char_of_utf8_string_lossless(input).unwrap();
        assert_eq!(ch, vec![0xFF]);
        assert_eq!(consumed, 1);
        assert!(!valid);
    }
}
