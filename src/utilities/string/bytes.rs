/// Formats a byte size as a human-readable string using decimal units.
pub fn format_bytes_decimal(bytes: u64) -> String {
    format_bytes(bytes.into(), 1000, &DECIMAL_SUFFIXES)
}

/// Formats a byte size as a human-readable string using binary units.
pub fn format_bytes_binary(bytes: u64) -> String {
    format_bytes(bytes.into(), 1024, &BINARY_SUFFIXES)
}

/// Formats a byte size as a human-readable string.
pub fn format_bytes_both(bytes: u64) -> String {
    format_bytes_both_u128(bytes.into())
}

pub fn format_bytes_decimal_u128(bytes: u128) -> String {
    format_bytes(bytes, 1000, &DECIMAL_SUFFIXES)
}

/// Formats a byte size as a human-readable string using binary units.
pub fn format_bytes_binary_u128(bytes: u128) -> String {
    format_bytes(bytes, 1024, &BINARY_SUFFIXES)
}

/// Formats a byte size as a human-readable string.
pub fn format_bytes_both_u128(bytes: u128) -> String {
    let dec = format_bytes_decimal_u128(bytes);
    let bin = format_bytes_binary_u128(bytes);
    if dec == bin {
        return dec;
    }

    format!("{dec} ({bin})")
}

const DECIMAL_SUFFIXES: [&str; 11] = [
    "B", "kB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB", "RB", "QB",
];
const BINARY_SUFFIXES: [&str; 11] = [
    "B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB", "ZiB", "YiB", "RiB", "QiB",
];

fn format_bytes(bytes: u128, unit: u128, suffixes: &[&str]) -> String {
    if bytes < 1000 {
        return format!("{bytes} B");
    }

    let mut base = 0usize;
    let mut unit_pow = u128::from(1u8);

    while base.saturating_add(1) < suffixes.len() {
        let Some(next_unit_pow) = unit_pow.checked_mul(unit) else {
            break;
        };
        if bytes < next_unit_pow {
            break;
        }

        unit_pow = next_unit_pow;
        base = base.saturating_add(1);
    }

    if base == 0 {
        return format!("{bytes} B");
    }

    let (mut integer, mut decimal_digit) = scale_and_round(bytes, unit_pow);
    let mut show_decimal = integer < 10;

    if !show_decimal {
        decimal_digit = None;
    }

    // If rounding pushes us exactly to the next unit (e.g. 1024 KiB), carry it.
    if base.saturating_add(1) < suffixes.len() && integer >= unit {
        let Some(next_unit_pow) = unit_pow.checked_mul(unit) else {
            // No larger representable unit; keep current unit.
            let number = format_scaled_int(integer, decimal_digit);
            let Some(suffix) = suffixes.get(base) else {
                return format!("{bytes} B");
            };
            return format!("{number} {suffix}");
        };

        base = base.saturating_add(1);
        unit_pow = next_unit_pow;

        (integer, decimal_digit) = scale_and_round(bytes, unit_pow);
        show_decimal = integer < 10;
        if !show_decimal {
            decimal_digit = None;
        }
    }

    let number = format_scaled_int(integer, decimal_digit);
    let Some(suffix) = suffixes.get(base) else {
        return format!("{bytes} B");
    };
    format!("{number} {suffix}")
}

fn scale_and_round(bytes: u128, unit_pow: u128) -> (u128, Option<u8>) {
    let integer = bytes.checked_div(unit_pow).unwrap_or(0);
    let remainder = bytes.checked_rem(unit_pow).unwrap_or(0);

    if integer >= 10 {
        let half = unit_pow.checked_div(u128::from(2u8)).unwrap_or(0);
        let rounded = bytes
            .checked_add(half)
            .and_then(|sum| sum.checked_div(unit_pow))
            .unwrap_or(0);
        return (rounded, None);
    }

    let ten = u128::from(10u8);
    let half = unit_pow.checked_div(u128::from(2u8)).unwrap_or(0);

    let scaled_remainder = remainder.saturating_mul(ten);
    let digit_u128 = scaled_remainder
        .checked_add(half)
        .and_then(|sum| sum.checked_div(unit_pow))
        .unwrap_or(0);
    let mut digit = u8::try_from(digit_u128).unwrap_or(u8::MAX);

    let mut integer = integer;
    if digit >= 10 {
        integer = integer.saturating_add(u128::from(1u8));
        digit = 0;
    }

    (integer, Some(digit))
}

fn format_scaled_int(integer: u128, decimal_digit: Option<u8>) -> String {
    let Some(digit) = decimal_digit else {
        return format!("{integer}");
    };
    if digit == 0 {
        return format!("{integer}");
    }
    format!("{integer}.{digit}")
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
    fn test_format_bytes_binary_basic() {
        assert_eq!(format_bytes_binary(500), "500 B");
        assert_eq!(format_bytes_binary(1001), "1001 B");
        assert_eq!(format_bytes_binary(1024), "1 KiB");
        assert_eq!(format_bytes_binary(1536), "1.5 KiB");
        assert_eq!(format_bytes_binary(1024 * 1024), "1 MiB");
        assert_eq!(format_bytes_binary(1024 * 1024 * 1024), "1 GiB");
        assert_eq!(format_bytes_binary(1024 * 1024 * 1024 * 1024), "1 TiB");
    }

    #[crate::ctb_test]
    fn test_format_bytes_decimal_basic() {
        assert_eq!(format_bytes_decimal(500), "500 B");
        assert_eq!(format_bytes_decimal(1000), "1 kB");
        assert_eq!(format_bytes_decimal(1001), "1 kB");
        assert_eq!(format_bytes_decimal(1024), "1 kB");
        assert_eq!(format_bytes_decimal(1536), "1.5 kB");
        assert_eq!(format_bytes_decimal(1024 * 1024), "1 MB");
        assert_eq!(format_bytes_decimal(1024 * 1024 * 1024), "1.1 GB");
        assert_eq!(format_bytes_decimal(1024 * 1024 * 1024 * 1024), "1.1 TB");
    }

    #[crate::ctb_test]
    fn test_rounding_carry_to_next_unit_decimal() {
        // Previously this would format as "1000 kB"; carrying produces a more
        // natural "1 MB".
        assert_eq!(format_bytes_decimal(999_500), "1 MB");
    }

    #[crate::ctb_test]
    fn test_rounding_carry_to_next_unit_binary() {
        // Previously this could become "1024 KiB"; carrying produces "1 MiB".
        assert_eq!(format_bytes_binary(1_048_575), "1 MiB"); // 1024*1024 - 1
    }

    #[crate::ctb_test]
    fn test_format_bytes_both_selected_cases() {
        assert_eq!(format_bytes_both(500), "500 B");
        assert_eq!(format_bytes_both(1001), "1 kB (1001 B)");
        assert_eq!(format_bytes_both(1024), "1 kB (1 KiB)");
        assert_eq!(format_bytes_both(1536), "1.5 kB (1.5 KiB)");
        assert_eq!(format_bytes_both(1024 * 1024), "1 MB (1 MiB)");

        // Carry behavior affects “-1” cases.
        assert_eq!(format_bytes_both(1024 * 1024 - 1), "1 MB (1 MiB)");
        assert_eq!(format_bytes_both(1024 * 1024 * 1024 - 1), "1.1 GB (1 GiB)");
        assert_eq!(
            format_bytes_both(1024 * 1024 * 1024 * 1024 - 1),
            "1.1 TB (1 TiB)"
        );

        // A non-boundary large value.
        assert_eq!(format_bytes_both(1024 * 1024 * 500), "524 MB (500 MiB)");
    }

    #[crate::ctb_test]
    fn test_small_values_show_one_decimal_place() {
        assert_eq!(format_bytes_decimal(9_500), "9.5 kB");
        assert_eq!(format_bytes_binary(9_728), "9.5 KiB"); // 9.5 * 1024 = 9728
    }

    #[crate::ctb_test]
    fn test_binary_large_suffixes_u64() {
        assert_eq!(format_bytes_binary(1u64 << 60), "1 EiB");
    }

    #[crate::ctb_test]
    fn test_u128_decimal_large_sizes() {
        let qb = 1000u128.pow(10);
        assert_eq!(format_bytes_decimal_u128(qb), "1 QB");
        assert_eq!(format_bytes_decimal_u128(qb + qb / 2), "1.5 QB");
        assert_eq!(format_bytes_both_u128(qb), "1 QB (808 RiB)");
    }

    #[crate::ctb_test]
    fn test_u128_binary_large_sizes() {
        let qib = 1u128 << 100; // 1024^10
        assert_eq!(format_bytes_binary_u128(qib), "1 QiB");
        assert_eq!(format_bytes_binary_u128(qib + (1u128 << 99)), "1.5 QiB");
        assert!(format_bytes_both_u128(qib).contains("QiB"));
    }

    #[crate::ctb_test]
    fn test_u128_binary_does_not_overflow_suffixes_on_carry() {
        // This is just below 1024 QiB; with carry we'd want “1 (next)” but there
        // is no larger suffix, so it must stay at QiB.
        let near_next = (1u128 << 110) - 1; // 1024^11 - 1
        assert_eq!(format_bytes_binary_u128(near_next), "1024 QiB");

        let exact_next = 1u128 << 110; // 1024^11
        assert_eq!(format_bytes_binary_u128(exact_next), "1024 QiB");
    }
}
