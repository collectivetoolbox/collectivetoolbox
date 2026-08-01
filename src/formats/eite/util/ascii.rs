//! ASCII helpers

#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

pub fn ascii_is_digit(b: u8) -> bool {
    b.is_ascii_digit()
}
pub fn ascii_is_alpha(b: u8) -> bool {
    b.is_ascii_alphabetic()
}
pub fn ascii_is_alphanum(b: u8) -> bool {
    b.is_ascii_alphanumeric()
}
pub fn ascii_is_letter_lower(b: u8) -> bool {
    (char::from(b)).is_ascii_lowercase()
}
pub fn ascii_is_letter(b: u8) -> bool {
    (char::from(b)).is_ascii_alphabetic()
}

/// True if value is a valid ASCII byte (0..=127).
pub fn is_ascii_byte(b: u8) -> bool {
    b <= 0x7F
}

/// Printable ASCII (32–126 inclusive)
pub fn ascii_is_printable(b: u8) -> bool {
    (32..=126).contains(&b)
}

/// Space (0x20)
pub fn ascii_is_space(b: u8) -> bool {
    b == 0x20
}

/// Newline (LF=10 or CR=13)
pub fn ascii_is_newline(b: u8) -> bool {
    b == 10 || b == 13
}

/// Uppercase A–Z
pub fn ascii_is_letter_upper(b: u8) -> bool {
    b.is_ascii_uppercase()
}

/// Letter (uppercase or lowercase)
pub fn ascii_is_letter_2(b: u8) -> bool {
    ascii_is_letter(b)
}

/// Alphanumeric (A–Z a–z 0–9)
pub fn ascii_is_alphanum_2(b: u8) -> bool {
    ascii_is_alphanum(b)
}

/// Convenience CRLF sequence.
pub fn crlf() -> Vec<u8> {
    vec![13, 10]
}

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use super::*;

    #[crate::ctb_test]
    fn test_ascii_helpers() {
        assert!(ascii_is_digit(b'5'));
        assert!(ascii_is_alphanum(b'A'));
        assert!(ascii_is_letter_lower(b'z'));
    }
}
