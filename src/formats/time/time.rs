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

//! Time parsing, formatting, and conversion functions.
//!
//! Note: Calculations and conversions may not be quite correct as it seems
//! there is currently no Rust library that fully implements leap seconds.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use jiff::civil::{Date, DateTime, Time};
use jiff::tz::TimeZone;
use jiff::{Span, Timestamp, Zoned};
use std::str::FromStr;

/// Parses an absolute epoch timestamp or relative duration like "7d", "24h", "60m".
///
/// Supports:
/// - Raw epoch seconds (e.g. "1725846721")
/// - RFC 3339 / ISO 8601 date and time strings (e.g. "2026-09-09T01:52:01Z", "2026-09-09")
/// - Relative duration strings (e.g. "7d", "24h", "60m", "30s", "1 week")
///   which are interpreted as being relative to current time (`now - duration`).
///
/// Note: Results may not be quite correct as it seems there is currently no Rust
/// library that fully implements leap seconds.
pub fn parse_time_spec(s: &str) -> Result<i64> {
    let s = s.trim();
    if s.is_empty() {
        bail!("Empty time specification");
    }

    // 1. Raw epoch seconds integer
    if let Ok(raw_sec) = s.parse::<i64>() {
        return Ok(raw_sec);
    }

    // 2. RFC 3339 / ISO 8601 timestamp with date and time
    if let Ok(ts) = Timestamp::from_str(s) {
        return Ok(ts.as_second());
    }

    // 3. RFC 3339 / ISO 8601 date without timezone/time (e.g. "2026-09-09")
    if let Ok(date) = Date::from_str(s) {
        let dt = date.to_datetime(Time::midnight());
        let zoned = dt
            .to_zoned(TimeZone::UTC)
            .context("Failed to convert date to UTC zoned datetime")?;
        return Ok(zoned.timestamp().as_second());
    }

    // 4. ISO 8601 datetime without timezone (e.g. "2026-09-09T12:00:00")
    if let Ok(dt) = DateTime::from_str(s) {
        let zoned = dt
            .to_zoned(TimeZone::UTC)
            .context("Failed to convert datetime to UTC zoned datetime")?;
        return Ok(zoned.timestamp().as_second());
    }

    // 5. Relative duration via Jiff friendly or ISO duration syntax
    let s_lower = s.to_ascii_lowercase();
    if let Ok(span) = Span::from_str(&s_lower) {
        let now = Zoned::now();
        let target = if s.starts_with('+') {
            now.checked_add(span)
                .context("Time addition overflowed supported range")?
        } else {
            now.checked_sub(span.abs())
                .context("Time subtraction underflowed supported range")?
        };
        return Ok(target.timestamp().as_second());
    }

    bail!(
        "Unrecognized time format: '{s}'. Expected timestamp (e.g. 1700000000, 2026-01-01) or relative duration (e.g. '7d', '24h', '60m')"
    );
}

/// Formats an epoch timestamp in seconds to "YYYY-MM-DD HH:MM" in UTC.
///
/// Note: Results may not be quite correct as it seems there is currently no Rust
/// library that fully implements leap seconds.
pub fn format_timestamp(sec: i64) -> String {
    if let Ok(ts) = Timestamp::from_second(sec) {
        ts.strftime("%Y-%m-%d %H:%M").to_string()
    } else {
        // Fallback for extreme timestamps outside supported range
        format!("{sec}")
    }
}

/// Determines if a given year is a leap year in the proleptic Gregorian calendar.
#[expect(
    clippy::expect_used,
    reason = "Divisors 4, 100, and 400 are non-zero constants"
)]
pub fn is_leap_year(year: i64) -> bool {
    if let Ok(y) = i16::try_from(year) {
        if let Ok(d) = Date::new(y, 1, 1) {
            return d.in_leap_year();
        }
    }
    (year.checked_rem(4).expect("divisor 4 is non-zero") == 0
        && year.checked_rem(100).expect("divisor 100 is non-zero") != 0)
        || (year.checked_rem(400).expect("divisor 400 is non-zero") == 0)
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
    fn test_parse_time_spec_epoch_seconds() {
        assert_eq!(parse_time_spec("0").unwrap(), 0);
        assert_eq!(parse_time_spec("1700000000").unwrap(), 1_700_000_000);
        assert_eq!(parse_time_spec("-1000").unwrap(), -1000);
    }

    #[crate::ctb_test]
    fn test_parse_time_spec_iso_and_rfc3339() {
        assert_eq!(
            parse_time_spec("1970-01-01T00:00:00Z").unwrap(),
            0
        );
        assert_eq!(
            parse_time_spec("1970-01-01").unwrap(),
            0
        );
        assert_eq!(
            parse_time_spec("1970-01-02T00:00:00Z").unwrap(),
            86400
        );
    }

    #[crate::ctb_test]
    fn test_parse_time_spec_relative_durations() {
        let now = Zoned::now().timestamp().as_second();

        // 60m should be approx now - 3600 seconds
        let t_60m = parse_time_spec("60m").unwrap();
        assert!((now - t_60m - 3600).abs() <= 5);

        // 24h should be approx now - 86400 seconds
        let t_24h = parse_time_spec("24h").unwrap();
        assert!((now - t_24h - 86400).abs() <= 5);

        // Case insensitivity: 24H
        let t_24h_upper = parse_time_spec("24H").unwrap();
        assert!((now - t_24h_upper - 86400).abs() <= 5);

        // 7d should be approx now - 7*86400 seconds
        let t_7d = parse_time_spec("7d").unwrap();
        assert!((now - t_7d - 7 * 86400).abs() <= 5);

        // Friendly phrases like "1 week" or "30s"
        let t_30s = parse_time_spec("30s").unwrap();
        assert!((now - t_30s - 30).abs() <= 5);

        let t_1w = parse_time_spec("1 week").unwrap();
        assert!((now - t_1w - 7 * 86400).abs() <= 5);
    }

    #[crate::ctb_test]
    fn test_parse_time_spec_invalid() {
        assert!(parse_time_spec("").is_err());
        assert!(parse_time_spec("   ").is_err());
        assert!(parse_time_spec("not_a_time").is_err());
        assert!(parse_time_spec("7xyz").is_err());
    }

    #[crate::ctb_test]
    fn test_format_timestamp() {
        assert_eq!(format_timestamp(0), "1970-01-01 00:00");
        assert_eq!(format_timestamp(86400), "1970-01-02 00:00");
        assert_eq!(format_timestamp(1_700_000_000), "2023-11-14 22:13");
    }

    #[crate::ctb_test]
    fn test_is_leap_year() {
        assert!(is_leap_year(2000));
        assert!(is_leap_year(2024));
        assert!(!is_leap_year(1900));
        assert!(!is_leap_year(2023));
        assert!(!is_leap_year(2025));
    }
}
