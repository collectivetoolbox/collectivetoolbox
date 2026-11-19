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

//! Shared timestamp formatting, parsing, and serde helpers backed by `jiff`.

use anyhow::{Context, Result};
use jiff::{Timestamp, civil::Time, fmt::temporal::DateTimePrinter};

/// Formats `ts` as UTC RFC 3339 with fixed 6-digit microsecond precision.
pub fn format_rfc3339_micros(ts: Timestamp) -> String {
    DateTimePrinter::new()
        .precision(Some(6))
        .timestamp_to_string(&ts)
}

/// Formats `ts` as UTC RFC 3339 using minimal SI fractional second width
/// (`0`, `3`, `6`, or `9` digits) matching historical `chrono` `to_rfc3339`
/// canonical signing output.
pub fn format_rfc3339_auto_si(ts: Timestamp) -> String {
    let nanos = ts.subsec_nanosecond().unsigned_abs();
    let precision = if nanos == 0 {
        Some(0)
    } else if nanos.checked_rem(1_000_000) == Some(0) {
        Some(3)
    } else if nanos.checked_rem(1_000) == Some(0) {
        Some(6)
    } else {
        Some(9)
    };
    DateTimePrinter::new()
        .precision(precision)
        .timestamp_to_string(&ts)
}

/// Parses an RFC 3339 timestamp string (with `Z` or explicit offset) into UTC
/// `Timestamp`.
pub fn parse_rfc3339(input: &str) -> Result<Timestamp> {
    input
        .parse::<Timestamp>()
        .with_context(|| format!("Failed to parse RFC 3339 timestamp: {input}"))
}

/// Returns whole seconds elapsed since midnight for a civil clock time.
pub fn seconds_from_midnight(time: Time) -> u32 {
    let h = u32::from(time.hour().unsigned_abs());
    let m = u32::from(time.minute().unsigned_abs());
    let s = u32::from(time.second().unsigned_abs());
    h.saturating_mul(3600)
        .saturating_add(m.saturating_mul(60))
        .saturating_add(s)
}

/// Serde adapter serializing `jiff::Timestamp` with
/// [`format_rfc3339_auto_si`] for canonical signature compatibility.
pub mod serde_rfc3339_auto_si {
    use jiff::Timestamp;
    use serde::{Deserialize, Deserializer, Serializer};

    /// Serializes a timestamp using minimal SI fractional precision.
    pub fn serialize<S>(
        ts: &Timestamp,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&super::format_rfc3339_auto_si(*ts))
    }

    /// Deserializes a timestamp from an RFC 3339 string.
    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> std::result::Result<Timestamp, D::Error>
    where
        D: Deserializer<'de>,
    {
        Timestamp::deserialize(deserializer)
    }
}

/// Serde adapter serializing `Option<jiff::Timestamp>` with
/// [`format_rfc3339_auto_si`].
pub mod serde_rfc3339_auto_si_opt {
    use jiff::Timestamp;
    use serde::{Deserialize, Deserializer, Serializer};

    /// Serializes an optional timestamp using minimal SI fractional precision.
    pub fn serialize<S>(
        ts: &Option<Timestamp>,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match ts {
            Some(value) => serializer
                .serialize_some(&super::format_rfc3339_auto_si(*value)),
            None => serializer.serialize_none(),
        }
    }

    /// Deserializes an optional timestamp from JSON `null` or RFC 3339 text.
    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> std::result::Result<Option<Timestamp>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::<Timestamp>::deserialize(deserializer)
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
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct SampleRecord {
        #[serde(with = "serde_rfc3339_auto_si")]
        required: Timestamp,
        #[serde(default, with = "serde_rfc3339_auto_si_opt")]
        optional: Option<Timestamp>,
    }

    #[crate::ctb_test]
    fn test_format_rfc3339_micros_fixed_six_digits() {
        let ts = Timestamp::new(1_700_000_000, 120_000_000).unwrap();
        assert_eq!(format_rfc3339_micros(ts), "2023-11-14T22:13:20.120000Z");

        let exact = Timestamp::from_second(1_700_000_000).unwrap();
        assert_eq!(format_rfc3339_micros(exact), "2023-11-14T22:13:20.000000Z");
    }

    #[crate::ctb_test]
    fn test_format_rfc3339_auto_si_precisions() {
        let s0 = Timestamp::new(1_700_000_000, 0).unwrap();
        let s3 = Timestamp::new(1_700_000_000, 120_000_000).unwrap();
        let s6 = Timestamp::new(1_700_000_000, 123_400_000).unwrap();
        let s9 = Timestamp::new(1_700_000_000, 123_456_700).unwrap();

        assert_eq!(format_rfc3339_auto_si(s0), "2023-11-14T22:13:20Z");
        assert_eq!(format_rfc3339_auto_si(s3), "2023-11-14T22:13:20.120Z");
        assert_eq!(format_rfc3339_auto_si(s6), "2023-11-14T22:13:20.123400Z");
        assert_eq!(
            format_rfc3339_auto_si(s9),
            "2023-11-14T22:13:20.123456700Z"
        );
    }

    #[crate::ctb_test]
    fn test_parse_rfc3339_utc_and_offset() {
        let z = parse_rfc3339("2026-11-19T03:00:00Z").unwrap();
        let off = parse_rfc3339("2026-11-18T22:00:00-05:00").unwrap();
        assert_eq!(z, off);
        assert!(parse_rfc3339("not-a-timestamp").is_err());
    }

    #[crate::ctb_test]
    fn test_seconds_from_midnight() {
        let t = Time::new(1, 2, 3, 999_999_999).unwrap();
        assert_eq!(seconds_from_midnight(t), 3723);
    }

    #[crate::ctb_test]
    fn test_serde_rfc3339_roundtrip() {
        let ts = Timestamp::new(1_700_000_000, 500_000_000).unwrap();
        let rec = SampleRecord {
            required: ts,
            optional: Some(ts),
        };
        let json = serde_json::to_string(&rec).unwrap();
        assert_eq!(
            json,
            r#"{"required":"2023-11-14T22:13:20.500Z","optional":"2023-11-14T22:13:20.500Z"}"#
        );
        let parsed: SampleRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, rec);

        let missing_opt: SampleRecord =
            serde_json::from_str(r#"{"required":"2023-11-14T22:13:20Z"}"#)
                .unwrap();
        assert_eq!(missing_opt.optional, None);
    }
}
