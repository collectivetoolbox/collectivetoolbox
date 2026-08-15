//! Generator for ranges of numbers in various number bases.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::Result;
use malachite::Natural;

use crate::base::{Base, format_natural, parse_natural};

/// Generates a vector of strings representing numbers in the given base from
/// `start` to `end` (inclusive). The width of the start string sets the
/// minimum output width for zero-padding.
#[expect(
    clippy::arithmetic_side_effects,
    reason = "Natural is arbitrary-precision and cannot overflow"
)]
pub fn range(base: Base, start: &str, end: &str) -> Result<Vec<String>> {
    let start_nat = parse_natural(start, base)?;
    let end_nat = parse_natural(end, base)?;
    let width = start.len();

    let mut result = Vec::new();
    let one = Natural::from(1u8);

    if start_nat <= end_nat {
        let mut curr = start_nat;
        while curr <= end_nat {
            result.push(format_natural(&curr, base, width)?);
            curr += &one;
        }
    } else {
        let mut curr = start_nat;
        while curr >= end_nat {
            result.push(format_natural(&curr, base, width)?);
            if curr == end_nat {
                break;
            }
            curr -= &one;
        }
    }

    Ok(result)
}

/// Generates a formatted string of numbers from `start` to `end` in `base`,
/// joined by `separator` with no trailing separator.
pub fn range_format(
    base: Base,
    start: &str,
    end: &str,
    separator: &str,
) -> Result<String> {
    let items = range(base, start, end)?;
    Ok(items.join(separator))
}

/// Generates a formatted string of numbers from `start` to `end` in `base`,
/// with `separator` appended to every item (including the trailing one).
pub fn range_format_trailing(
    base: Base,
    start: &str,
    end: &str,
    separator: &str,
) -> Result<String> {
    let items = range(base, start, end)?;
    let mut out = String::new();
    for item in items {
        out.push_str(&item);
        out.push_str(separator);
    }
    Ok(out)
}

/// Alias for [`range_format_trailing`].
pub fn range_trailing(
    base: Base,
    start: &str,
    end: &str,
    separator: &str,
) -> Result<String> {
    range_format_trailing(base, start, end, separator)
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
    use crate::base;

    #[crate::ctb_test]
    fn test_range_hex() {
        let items = range(base::Base16, "0A", "12").unwrap();
        assert_eq!(
            items,
            vec!["0A", "0B", "0C", "0D", "0E", "0F", "10", "11", "12"]
        );

        let formatted = range_format(base::Base16, "0A", "12", ", ").unwrap();
        assert_eq!(formatted, "0A, 0B, 0C, 0D, 0E, 0F, 10, 11, 12");
    }

    #[crate::ctb_test]
    fn test_range_trailing_decimal() {
        let formatted = range_trailing(base::Base10, "9", "12", "\n").unwrap();
        assert_eq!(formatted, "9\n10\n11\n12\n");

        let formatted2 =
            range_format_trailing(base::Base10, "9", "12", "\n").unwrap();
        assert_eq!(formatted2, "9\n10\n11\n12\n");
    }

    #[crate::ctb_test]
    fn test_range_descending() {
        let items = range(base::Base10, "05", "02").unwrap();
        assert_eq!(items, vec!["05", "04", "03", "02"]);
    }

    #[crate::ctb_test]
    fn test_range_single_value() {
        let items = range(base::Decimal, "42", "42").unwrap();
        assert_eq!(items, vec!["42"]);
    }

    #[crate::ctb_test]
    fn test_range_base64() {
        let items = range(base::Base64, "A", "D").unwrap();
        assert_eq!(items, vec!["A", "B", "C", "D"]);
    }
}