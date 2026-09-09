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

//! Human-readable byte size formatting using decimal and binary units.

use anyhow::{Context, Result};

const DECIMAL_SUFFIXES: [&str; 11] = [
    "B", "kB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB", "RB", "QB",
];
const BINARY_SUFFIXES: [&str; 11] = [
    "B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB", "ZiB", "YiB", "RiB", "QiB",
];

/// Parses human-readable size specifications like "10k", "20KiB", "5M", "1G" into bytes.
pub fn parse_bytes(s: &str) -> Result<u64> {
    let val_u128 = parse_bytes_u128(s)?;
    u64::try_from(val_u128).context("Byte size exceeds u64 limit")
}

fn suffix_power(c: char) -> Option<usize> {
    match c {
        'k' => Some(1),
        'm' => Some(2),
        'g' => Some(3),
        't' => Some(4),
        'p' => Some(5),
        'e' => Some(6),
        'z' => Some(7),
        'y' => Some(8),
        'r' => Some(9),
        'q' => Some(10),
        _ => None,
    }
}

fn unit_multiplier(base: u128, power: usize) -> Result<u128> {
    let mut mult = 1_u128;
    for _ in 0..power {
        mult = mult
            .checked_mul(base)
            .context("Byte size multiplier overflow")?;
    }
    Ok(mult)
}

/// Parses human-readable size specifications into a 128-bit byte count.
///
/// Supports:
/// - Plain numbers (e.g. "500", "500B", "500 bytes")
/// - Binary IEC units (e.g. "20KiB", "5MiB", "1GiB")
/// - Decimal units (e.g. "20kB", "5MB", "1GB")
/// - Shorthand units (e.g. "20k", "5M", "1G", interpreted as binary units)
pub fn parse_bytes_u128(s: &str) -> Result<u128> {
    let s = s.trim();
    if s.is_empty() {
        anyhow::bail!("Empty size string");
    }

    let num_end = s
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(s.len());

    let (num_str, suffix_str) = s.split_at(num_end);
    if num_str.is_empty() {
        anyhow::bail!("Invalid byte size specification '{s}': missing numeric value");
    }

    let base_val: u128 = num_str
        .parse()
        .with_context(|| format!("Invalid number in byte size specification: '{num_str}'"))?;

    let suffix = suffix_str.trim().to_ascii_lowercase();

    let multiplier = if suffix.is_empty()
        || suffix == "b"
        || suffix == "byte"
        || suffix == "bytes"
    {
        1_u128
    } else if let Some(stripped) = suffix.strip_suffix("ib") {
        if stripped.chars().count() != 1 {
            anyhow::bail!("Unrecognized binary unit '{suffix}' in byte specification: '{s}'");
        }
        let c = stripped.chars().next().context("Missing unit prefix")?;
        let power = suffix_power(c)
            .with_context(|| format!("Unknown binary unit prefix '{c}' in: '{s}'"))?;
        unit_multiplier(1024, power)?
    } else if let Some(stripped) = suffix.strip_suffix('b') {
        if stripped.chars().count() != 1 {
            anyhow::bail!("Unrecognized decimal unit '{suffix}' in byte specification: '{s}'");
        }
        let c = stripped.chars().next().context("Missing unit prefix")?;
        let power = suffix_power(c)
            .with_context(|| format!("Unknown decimal unit prefix '{c}' in: '{s}'"))?;
        unit_multiplier(1000, power)?
    } else {
        if suffix.chars().count() != 1 {
            anyhow::bail!("Unrecognized unit suffix '{suffix}' in byte specification: '{s}'");
        }
        let c = suffix.chars().next().context("Missing unit prefix")?;
        let power = suffix_power(c)
            .with_context(|| format!("Unknown unit prefix '{c}' in: '{s}'"))?;
        unit_multiplier(1024, power)?
    };

    base_val
        .checked_mul(multiplier)
        .context("Byte size calculation overflowed u128")
}

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
    let Some(integer) = bytes.checked_div(unit_pow) else {
        return (0, None);
    };
    let Some(remainder) = bytes.checked_rem(unit_pow) else {
        return (0, None);
    };

    let Some(half) = unit_pow.checked_div(2) else {
        return (integer, None);
    };

    if integer >= 10 {
        // Reason for fallback: arithmetic addition overflow during rounding retains unrounded integer scale
        let rounded = bytes
            .checked_add(half)
            .and_then(|sum| sum.checked_div(unit_pow))
            .unwrap_or(integer);
        return (rounded, None);
    }

    let ten = u128::from(10u8);
    let scaled_remainder = remainder.saturating_mul(ten);
    // Reason for fallback: rounding precision is cosmetic; falling back to 0 omits the decimal component on math overflow
    let digit_u128 = scaled_remainder
        .checked_add(half)
        .and_then(|sum| sum.checked_div(unit_pow))
        .unwrap_or(0);
    // Reason for fallback: u128 multiplication overflow during rounding truncates to u8::MAX to maintain maximum scale
    let digit = u8::try_from(digit_u128).unwrap_or(u8::MAX);
    if digit >= 10 {
        let rounded = integer.saturating_add(1);
        (rounded, None)
    } else {
        (integer, Some(digit))
    }
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

    #[crate::ctb_test]
    fn test_parse_bytes_various_formats() {
        assert_eq!(parse_bytes("500").unwrap(), 500);
        assert_eq!(parse_bytes("500B").unwrap(), 500);
        assert_eq!(parse_bytes("500 b").unwrap(), 500);
        assert_eq!(parse_bytes("500 bytes").unwrap(), 500);
        assert_eq!(parse_bytes("10k").unwrap(), 10 * 1024);
        assert_eq!(parse_bytes("20K").unwrap(), 20 * 1024);
        assert_eq!(parse_bytes("20KiB").unwrap(), 20 * 1024);
        assert_eq!(parse_bytes("20kib").unwrap(), 20 * 1024);
        assert_eq!(parse_bytes("20kB").unwrap(), 20 * 1000);
        assert_eq!(parse_bytes("20 kb").unwrap(), 20 * 1000);
        assert_eq!(parse_bytes("5M").unwrap(), 5 * 1024 * 1024);
        assert_eq!(parse_bytes("5MiB").unwrap(), 5 * 1024 * 1024);
        assert_eq!(parse_bytes("5MB").unwrap(), 5 * 1000 * 1000);
        assert_eq!(parse_bytes("1G").unwrap(), 1024 * 1024 * 1024);
        assert_eq!(parse_bytes("1GiB").unwrap(), 1024 * 1024 * 1024);
        assert_eq!(parse_bytes("1GB").unwrap(), 1_000_000_000);
        assert_eq!(parse_bytes("2T").unwrap(), 2 * 1024 * 1024 * 1024 * 1024);
        assert_eq!(parse_bytes("2TiB").unwrap(), 2 * 1024 * 1024 * 1024 * 1024);
        assert_eq!(parse_bytes("2TB").unwrap(), 2_000_000_000_000);
        assert!(parse_bytes("").is_err());
        assert!(parse_bytes("   ").is_err());
        assert!(parse_bytes("not_a_number").is_err());
        assert!(parse_bytes("-50").is_err());
        assert!(parse_bytes("10xyz").is_err());
    }
}

