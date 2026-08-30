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

//! Generator for ranges of numbers in various number bases.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::{Result, ensure};
use malachite::base::num::arithmetic::traits::UnsignedAbs;
use malachite::base::num::basic::traits::Zero;
use malachite::{Integer, Natural};

use crate::base::Base;
use crate::parsing::{ParsedNumber, format_scaled};

/// Generates a vector of strings representing numbers in the given base from
/// `start` to `end` (inclusive) stepping by `step`. The width of the start
/// string sets the minimum output width for zero-padding.
#[expect(
    clippy::arithmetic_side_effects,
    reason = "Natural and Integer are arbitrary-precision and cannot overflow"
)]
pub fn range(
    base: Base,
    start: &str,
    end: &str,
    step: &str,
) -> Result<Vec<String>> {
    let start_num = ParsedNumber::parse(start, base)?;
    let end_num = ParsedNumber::parse(end, base)?;
    let step_str = if step.is_empty() { "1" } else { step };
    let step_base = if base.radix() == 64 {
        Base::Decimal
    } else {
        base
    };
    let step_num = ParsedNumber::parse(step_str, step_base)?;

    let dec_places = start_num
        .frac_len
        .max(end_num.frac_len)
        .max(step_num.frac_len);

    let has_decimal =
        start_num.has_decimal || end_num.has_decimal || step_num.has_decimal;

    let start_val = start_num.to_scaled_integer(dec_places)?;
    let end_val = end_num.to_scaled_integer(dec_places)?;
    let step_raw = step_num.to_scaled_integer(dec_places)?;
    let step_val = step_raw.unsigned_abs();
    ensure!(step_val > Natural::ZERO, "Step cannot be zero");
    let step_int = Integer::from(step_val);

    let mut result = Vec::new();

    if start_val <= end_val {
        let mut curr = start_val;
        while curr <= end_val {
            result.push(format_scaled(
                &curr,
                base,
                dec_places,
                start_num.int_width,
                start_num.frac_len,
                has_decimal,
            )?);
            curr += &step_int;
        }
    } else {
        let mut curr = start_val;
        while curr >= end_val {
            result.push(format_scaled(
                &curr,
                base,
                dec_places,
                start_num.int_width,
                start_num.frac_len,
                has_decimal,
            )?);
            curr -= &step_int;
        }
    }

    Ok(result)
}

/// Generates a formatted string of numbers from `range`, joined by `separator`
/// with no trailing separator.
#[must_use]
pub fn range_format(range: &[String], separator: &str) -> String {
    range.join(separator)
}

/// Generates a formatted string of numbers from `range`, with `separator`
/// appended to every item (including the trailing one).
#[must_use]
pub fn range_format_trailing(range: &[String], separator: &str) -> String {
    let mut out = String::new();
    for item in range {
        out.push_str(item);
        out.push_str(separator);
    }
    out
}

/// Alias for [`range_format_trailing`].
#[must_use]
pub fn range_trailing(range: &[String], separator: &str) -> String {
    range_format_trailing(range, separator)
}

/// Arguments for the `range_gen` CLI command.
#[derive(clap::Args, Debug, Clone)]
#[command(
    after_help = "Examples:\n  $ ctoolbox range_gen 1 10\n  1\n  2\n  3\n  4\n  5\n  6\n  7\n  8\n  9\n  10\n\n  $ ctoolbox range_gen -s 2 1 10\n  1\n  3\n  5\n  7\n  9\n\n  $ ctoolbox range_gen -b 16 -t -S, 18D0C 18D12\n  18D0C,18D0D,18D0E,18D0F,18D10,18D11,18D12,\n\n  $ ctoolbox range_gen -b hex 0x00 0x10\n  00\n  01\n  02\n  03\n  04\n  05\n  06\n  07\n  08\n  09\n  0A\n  0B\n  0C\n  0D\n  0E\n  0F\n  10"
)]
pub struct RangeGenArgs {
    /// Starting value of the range
    pub start: String,
    /// Ending value of the range
    pub end: String,
    /// Step size (defaults to "1")
    #[arg(short, long, default_value = "1")]
    pub step: String,
    /// Number base (e.g. "10", "16", "2", "64", "hex", "bin", "oct")
    #[arg(short, long, default_value = "10")]
    pub base: String,
    /// Separator between output items (defaults to newline)
    #[arg(short = 'S', long, default_value = "\n")]
    pub separator: String,
    /// Append a trailing separator to the output
    #[arg(short, long)]
    pub trailing: bool,
}

/// CLI tool handler for range generation.
pub fn range_cli_handler(args: &RangeGenArgs) -> Result<String> {
    let base = Base::from_str_or_name(&args.base)?;
    let items = range(base, &args.start, &args.end, &args.step)?;
    if args.trailing {
        Ok(range_format_trailing(&items, &args.separator))
    } else {
        Ok(range_format(&items, &args.separator))
    }
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
    fn test_range_hex() -> Result<()> {
        let items = range(base::Hex, "0A", "12", "1")?;
        assert_eq!(
            items,
            vec!["0A", "0B", "0C", "0D", "0E", "0F", "10", "11", "12"]
        );

        let formatted =
            range_format(&range(base::Hex, "0A", "12", "1")?, ", ");
        assert_eq!(formatted, "0A, 0B, 0C, 0D, 0E, 0F, 10, 11, 12");
        Ok(())
    }

    #[crate::ctb_test]
    fn test_range_hex_prefix() -> Result<()> {
        let items = range(base::Hex, "0x00", "0x10", "1")?;
        assert_eq!(items.len(), 17);
        assert_eq!(items.first().map(String::as_str), Some("00"));
        assert_eq!(items.last().map(String::as_str), Some("10"));
        Ok(())
    }

    #[crate::ctb_test]
    fn test_range_trailing_decimal() -> Result<()> {
        let formatted =
            range_trailing(&range(base::Decimal, "9", "12", "1")?, "\n");
        assert_eq!(formatted, "9\n10\n11\n12\n");

        let formatted2 = range_format_trailing(
            &range(base::Decimal, "9", "12", "1")?,
            "\n",
        );
        assert_eq!(formatted2, "9\n10\n11\n12\n");
        Ok(())
    }

    #[crate::ctb_test]
    fn test_range_descending() -> Result<()> {
        let items = range(base::Decimal, "05", "02", "1")?;
        assert_eq!(items, vec!["05", "04", "03", "02"]);
        let items = range(base::Decimal, "02", "-2", "1")?;
        assert_eq!(items, vec!["02", "01", "00", "-01", "-02"]);
        let items = range(base::Decimal, "0.010", "-0.010", "0.02")?;
        assert_eq!(items, vec!["0.010", "-0.010"]);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_range_single_value() -> Result<()> {
        let items = range(base::Decimal, "42", "42", "1")?;
        assert_eq!(items, vec!["42"]);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_range_base64() -> Result<()> {
        let b64 = Base::new(64)?;
        let items = range(b64, "A", "D", "1")?;
        assert_eq!(items, vec!["A", "B", "C", "D"]);
        let items = range(b64, "9", "/", "1")?;
        assert_eq!(items, vec!["9", "+", "/"]);
        Ok(())
    }
}