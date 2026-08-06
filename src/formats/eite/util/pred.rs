//! Character tests

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::{
    encoding::ascii::byte_from_stagel_char, util::math::int_is_between_i32,
};

pub fn is_byte(v: i32) -> bool {
    int_is_between_i32(v, 0, 255)
}
pub fn is_int_bit(v: i32) -> bool {
    v == 0 || v == 1
}
pub fn is_char_byte(v: i32) -> bool {
    int_is_between_i32(v, 32, 126)
}

pub fn is_char_str(s: &str) -> bool {
    if s.chars().count() != 1 {
        return false;
    }
    let Ok(b) = byte_from_stagel_char(s) else {
        return false;
    };
    is_char_byte(i32::from(b))
}

// ---------------
// Ident validation (lowercase first letter, then alnum)
// ---------------

pub fn is_valid_ident(s: &str) -> bool {
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_lowercase() {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric())
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
    fn test_ident_validation() {
        assert!(is_valid_ident("abc1"));
        assert!(!is_valid_ident("Abc"));
        assert!(!is_valid_ident("a-b"));
    }
}
