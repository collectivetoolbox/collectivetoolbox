//! Numeric predicates for i32

#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

pub fn is_nonnegative_i32(n: i32) -> bool {
    n >= 0
}
pub fn is_negative_i32(n: i32) -> bool {
    n < 0
}
pub fn is_positive_i32(n: i32) -> bool {
    n > 0
}
pub fn is_nonpositive_i32(n: i32) -> bool {
    n <= 0
}
pub fn is_even_i32(n: i32) -> bool {
    n % 2 == 0
}
pub fn is_odd_i32(n: i32) -> bool {
    n % 2 != 0
}

pub fn int_is_between_i32(n: i32, a: i32, b: i32) -> bool {
    n >= a && n <= b
}

// Numeric predicates for u32
pub fn is_nonnegative_u32(_n: u32) -> bool {
    true
} // Always true for u32
pub fn is_negative_u32(_n: u32) -> bool {
    false
} // Always false for u32
pub fn is_positive_u32(n: u32) -> bool {
    n > 0
}
pub fn is_nonpositive_u32(n: u32) -> bool {
    n == 0
}
pub fn is_even_u32(n: u32) -> bool {
    n.is_multiple_of(2)
}
pub fn is_odd_u32(n: u32) -> bool {
    !n.is_multiple_of(2)
}

pub fn int_is_between_u32(n: u32, a: u32, b: u32) -> bool {
    n >= a && n <= b
}

// ---------------
// Inequality wrappers
// ---------------

pub fn ge_int(a: i32, b: i32) -> bool {
    a >= b
}
pub fn le_int(a: i32, b: i32) -> bool {
    a <= b
}
pub fn ngt_int(a: i32, b: i32) -> bool {
    a <= b
}
pub fn nlt_int(a: i32, b: i32) -> bool {
    a >= b
}
pub fn nge_int(a: i32, b: i32) -> bool {
    a < b
}
pub fn nle_int(a: i32, b: i32) -> bool {
    a > b
}

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use super::*;

    #[crate::ctb_test]
    fn test_number_predicates() {
        assert!(is_nonnegative_u32(0));
        assert!(!is_negative_u32(0));
        assert!(is_positive_u32(5));
        assert!(is_nonpositive_u32(0));
        assert!(is_even_u32(4));
        assert!(is_odd_u32(5));
        assert!(int_is_between_u32(5, 1, 10));
        assert!(!int_is_between_u32(0, 1, 10));

        assert!(is_nonnegative_i32(0));
        assert!(is_negative_i32(-1));
        assert!(!is_negative_i32(0));
        assert!(is_positive_i32(5));
        assert!(is_nonpositive_i32(0));
        assert!(is_even_i32(4));
        assert!(is_odd_i32(5));
        assert!(int_is_between_i32(5, 1, 10));
        assert!(!int_is_between_i32(0, 1, 10));
    }
}
