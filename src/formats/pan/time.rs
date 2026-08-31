/* SPDX-License-Identifier: MIT */
/*
This file is part of Collective Toolbox, a database and document workspace and utilities.
Copyright (C) 2026 Collective Toolbox Developers
Contact: info@collectivetoolbox.com

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the “Software”), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software is furnished to do so,
subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

//! Pan time parsing and formatting helpers.

#![allow(
    clippy::module_name_repetitions,
    reason = "idiomatic module structure names"
)]

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use chrono::{Local, Timelike};

/// A time represented as seconds.
///
/// Most functions interpret this as seconds since midnight, but some parsing
/// helpers accept elapsed times beyond 24 hours.
pub type PanTime = i64;

const SECONDS_PER_MINUTE: i64 = 60;
const SECONDS_PER_HOUR: i64 = 60 * SECONDS_PER_MINUTE;
const SECONDS_PER_DAY: i64 = 24 * SECONDS_PER_HOUR;

/// Returns the current local time as seconds since midnight.
pub fn now() -> Result<PanTime> {
    let t = Local::now().time();
    let h = i64::from(t.hour());
    let m = i64::from(t.minute());
    let s = i64::from(t.second());

    let secs = h
        .checked_mul(SECONDS_PER_HOUR)
        .context("hour overflow")?
        .checked_add(
            m.checked_mul(SECONDS_PER_MINUTE)
                .context("minute overflow")?,
        )
        .context("time overflow")?
        .checked_add(s)
        .context("time overflow")?;

    Ok(secs)
}

/// Parses a time string into seconds.
///
/// When no meridiem is provided, elapsed hours beyond 24 are allowed.
pub fn seconds(text: &str) -> Result<PanTime> {
    let raw = text.trim();
    if raw.is_empty() {
        bail!("empty time string");
    }

    let (core, mer) = split_meridiem(raw);
    let (h, m, s) = parse_colon_time(core)?;

    let Some(m) = normalize_minute_second(m, s)? else {
        bail!("invalid time: {raw}");
    };

    let (m, s) = m;

    if let Some(mer) = mer {
        let h12 = validate_hour_12(h)?;
        return hms_to_seconds(to_24h(h12, mer), m, s);
    }

    // Elapsed time: allow hours beyond 24.
    hms_to_seconds_elapsed(h, m, s)
}

/// Formats a time using a Pan time pattern.
///
/// The tokens `hh`, `mm`, `ss`, and `am/pm` are recognized (case-insensitive).
pub fn timepattern(number: PanTime, pattern: &str) -> Result<String> {
    if number < 0 {
        bail!("timepattern expects a non-negative number, got {number}");
    }

    let has_mer = contains_meridiem_token(pattern);

    let (h, m, s, mer) = if has_mer {
        let n = time24(number);
        let (h24, m, s) = split_hms(n)?;
        let (h12, mer) = to_12h(h24)?;
        (h12, m, s, Some(mer))
    } else {
        let (h, m, s) = split_hms_elapsed(number)?;
        (h, m, s, None)
    };

    let mut out = String::with_capacity(pattern.len().saturating_add(8));
    let mut i = 0usize;
    while i < pattern.len() {
        let slice = pattern.get(i..).context("Invalid pattern index")?;

        if let Some((style, len)) = match_meridiem_token(slice) {
            let Some(mer) = mer else {
                bail!("pattern contains am/pm but meridiem is unavailable");
            };
            out.push_str(&apply_case_meridiem(mer, style));
            i = i.checked_add(len).context("pattern index overflow")?;
            continue;
        }

        if slice.len() >= 2
            && slice.get(..2).is_some_and(|s| s.eq_ignore_ascii_case("hh"))
        {
            out.push_str(&format!("{h}"));
            i = i.checked_add(2).context("pattern index overflow")?;
            continue;
        }
        if slice.len() >= 2
            && slice.get(..2).is_some_and(|s| s.eq_ignore_ascii_case("mm"))
        {
            out.push_str(&format!("{m:02}"));
            i = i.checked_add(2).context("pattern index overflow")?;
            continue;
        }
        if slice.len() >= 2
            && slice.get(..2).is_some_and(|s| s.eq_ignore_ascii_case("ss"))
        {
            out.push_str(&format!("{s:02}"));
            i = i.checked_add(2).context("pattern index overflow")?;
            continue;
        }

        let ch = slice
            .chars()
            .next()
            .context("failed to read next pattern char")?;
        out.push(ch);
        i = i
            .checked_add(ch.len_utf8())
            .context("pattern index overflow")?;
    }

    Ok(out)
}

/// Formats `number` as `hh:mm AM/PM`.
pub fn timestr(number: PanTime) -> Result<String> {
    timepattern(number, "hh:mm AM/PM")
}

/// Wraps a time into the 24-hour range.
pub fn time24(time: PanTime) -> PanTime {
    time.rem_euclid(SECONDS_PER_DAY)
}

/// Returns the signed shortest difference from `start` to `end`.
pub fn timedifference(start: PanTime, end: PanTime) -> PanTime {
    let diff = end.saturating_sub(start);
    #[expect(
        clippy::expect_used,
        reason = "Division by constant 2 is non-zero and cannot fail"
    )]
    let half_day = SECONDS_PER_DAY.checked_div(2).expect("2 is non-zero");
    diff.saturating_add(half_day)
        .rem_euclid(SECONDS_PER_DAY)
        .saturating_sub(half_day)
}

/// Returns the forward interval from `start` to `end` (wraps at 24h).
pub fn timeinterval(start: PanTime, end: PanTime) -> PanTime {
    (end.saturating_sub(start)).rem_euclid(SECONDS_PER_DAY)
}

/// Parses a flexible time string into seconds since midnight.
pub fn time(text: &str) -> Result<PanTime> {
    let raw = text.trim();
    if raw.is_empty() {
        bail!("empty time string");
    }

    let lower = raw.to_lowercase();
    let norm = lower.split_whitespace().collect::<Vec<_>>().join(" ");
    let t = norm.as_str();

    if let Some(named) = parse_named_time(t)? {
        return Ok(named);
    }

    // Accept "4p" / "4a" as special short suffix.
    if let Some((core, mer)) = split_short_meridiem(t) {
        let h = core.parse::<i64>().context("hour")?;
        let h12 = validate_hour_12(h)?;
        return hms_to_seconds(to_24h(h12, mer), 0, 0);
    }

    let (core, mer) = split_meridiem(t);

    let (h, m, s) = if core.contains(':') {
        parse_colon_time(core)?
    } else {
        parse_compact_time(core)?
    };

    let Some((m, s)) = normalize_minute_second(m, s)? else {
        bail!("invalid time: {raw}");
    };

    if let Some(mer) = mer {
        let h12 = validate_hour_12(h)?;
        return hms_to_seconds(to_24h(h12, mer), m, s);
    }

    // No meridiem:
    // - If hour >= 13, interpret as 24-hour.
    // - If hour <= 12, use default guessing (6..11 AM, else PM).
    if h >= 13 {
        return hms_to_seconds(validate_hour_24(h)?, m, s);
    }

    // Special-case "24:00[:00]" as midnight.
    if h == 24 && m == 0 && s == 0 {
        return Ok(0);
    }

    let h12 = validate_hour_12(h)?;
    let mer = guess_meridiem(h12);
    hms_to_seconds(to_24h(h12, mer), m, s)
}

/// Parses two times and formats the signed difference as `H:MM[:SS]`.
pub fn texttimedifference(start: &str, end: &str) -> Result<String> {
    let s = time(start)?;
    let e = time(end)?;
    Ok(format_interval_signed(timedifference(s, e)))
}

/// Parses two times and formats the forward interval as `H:MM[:SS]`.
pub fn texttimeinterval(start: &str, end: &str) -> Result<String> {
    let s = time(start)?;
    let e = time(end)?;
    Ok(format_interval_unsigned(timeinterval(s, e)))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Meridiem {
    Am,
    Pm,
}

#[derive(Debug, Clone, Copy)]
enum CaseStyle {
    Lower,
    Title,
    Upper,
}

fn match_meridiem_token(slice: &str) -> Option<(CaseStyle, usize)> {
    const RAW: &str = "am/pm";
    let len = RAW.len();
    if slice.len() < len {
        return None;
    }
    let Some(head) = slice.get(..len) else {
        return None;
    };
    if !head.eq_ignore_ascii_case(RAW) {
        return None;
    }

    let has_alpha = head.chars().any(|c| c.is_ascii_alphabetic());
    let all_lower = has_alpha
        && head
            .chars()
            .all(|c| !c.is_ascii_alphabetic() || c.is_ascii_lowercase());
    let all_upper = has_alpha
        && head
            .chars()
            .all(|c| !c.is_ascii_alphabetic() || c.is_ascii_uppercase());

    let style = if all_upper {
        CaseStyle::Upper
    } else if all_lower {
        CaseStyle::Lower
    } else {
        CaseStyle::Title
    };

    Some((style, len))
}

fn apply_case_meridiem(mer: Meridiem, style: CaseStyle) -> String {
    let base = match mer {
        Meridiem::Am => "am",
        Meridiem::Pm => "pm",
    };

    match style {
        CaseStyle::Lower => base.to_string(),
        CaseStyle::Upper => base.to_uppercase(),
        CaseStyle::Title => {
            let mut it = base.chars();
            let Some(first) = it.next() else {
                return String::new();
            };
            let mut out = String::new();
            for c in first.to_uppercase() {
                out.push(c);
            }
            out.push_str(&it.as_str().to_lowercase());
            out
        }
    }
}

fn contains_meridiem_token(pattern: &str) -> bool {
    pattern.to_lowercase().contains("am/pm")
}

fn parse_named_time(t: &str) -> Result<Option<PanTime>> {
    let secs = match t.trim() {
        "midnight" => 0,
        "noon" => 12 * SECONDS_PER_HOUR,
        "morning" => 9 * SECONDS_PER_HOUR,
        "afternoon" => 13 * SECONDS_PER_HOUR,
        "evening" => 18 * SECONDS_PER_HOUR,
        "night" | "nite" => 22 * SECONDS_PER_HOUR,
        _ => return Ok(None),
    };
    Ok(Some(secs))
}

fn split_short_meridiem(t: &str) -> Option<(&str, Meridiem)> {
    let t = t.trim();
    if t.len() < 2 {
        return None;
    }

    let (core, suf) = t.split_at(t.len().saturating_sub(1));
    let mer = match suf {
        "a" => Meridiem::Am,
        "p" => Meridiem::Pm,
        _ => return None,
    };

    if core.chars().all(|c| c.is_ascii_digit()) {
        Some((core, mer))
    } else {
        None
    }
}

#[expect(
    clippy::expect_used,
    reason = "suffix is ASCII so t.len() - suffix.len() is an infallible UTF-8 character boundary in t"
)]
fn split_meridiem(raw: &str) -> (&str, Option<Meridiem>) {
    let t = raw.trim();
    let lower = t.to_lowercase();

    // Prefer explicit suffixes (with optional dots).
    for (suffix, mer) in [
        (" a.m.", Meridiem::Am),
        (" p.m.", Meridiem::Pm),
        (" am", Meridiem::Am),
        (" pm", Meridiem::Pm),
        ("a.m.", Meridiem::Am),
        ("p.m.", Meridiem::Pm),
        ("am", Meridiem::Am),
        ("pm", Meridiem::Pm),
    ] {
        if lower.ends_with(suffix) {
            let core_len = t.len().saturating_sub(suffix.len());
            let core = t
                .get(..core_len)
                .expect("suffix is ASCII so t.len() - suffix.len() is a valid character boundary in t");
            return (core.trim(), Some(mer));
        }
    }

    (t, None)
}

fn parse_colon_time(t: &str) -> Result<(i64, i64, i64)> {
    let parts: Vec<&str> = t.trim().split(':').collect();
    if parts.len() != 2 && parts.len() != 3 {
        bail!("invalid colon time: {t}");
    }

    let h = parts
        .first()
        .context("Missing hour part")?
        .trim()
        .parse::<i64>()
        .context("hour")?;
    let m = parts
        .get(1)
        .context("Missing minute part")?
        .trim()
        .parse::<i64>()
        .context("minute")?;
    let s = if parts.len() == 3 {
        parts
            .get(2)
            .context("Missing second part")?
            .trim()
            .parse::<i64>()
            .context("second")?
    } else {
        0
    };

    Ok((h, m, s))
}

fn parse_compact_time(t: &str) -> Result<(i64, i64, i64)> {
    let digits = t.trim();
    if digits.is_empty() {
        bail!("empty compact time");
    }
    if !digits.chars().all(|c| c.is_ascii_digit()) {
        bail!("invalid compact time: {t}");
    }

    match digits.len() {
        1 | 2 => {
            let h = digits.parse::<i64>().context("hour")?;
            Ok((h, 0, 0))
        }
        3 => {
            let (h, mm) = digits.split_at(1);
            let h = h.parse::<i64>().context("hour")?;
            let m = mm.parse::<i64>().context("minute")?;
            Ok((h, m, 0))
        }
        4 => {
            let (hh, mm) = digits.split_at(2);
            let h = hh.parse::<i64>().context("hour")?;
            let m = mm.parse::<i64>().context("minute")?;
            Ok((h, m, 0))
        }
        _ => bail!("compact time must have 1..=4 digits, got {}", digits.len()),
    }
}

fn normalize_minute_second(m: i64, s: i64) -> Result<Option<(i64, i64)>> {
    if !(0..=59).contains(&m) {
        return Ok(None);
    }
    if !(0..=59).contains(&s) {
        return Ok(None);
    }
    Ok(Some((m, s)))
}

fn validate_hour_12(h: i64) -> Result<i64> {
    if !(1..=12).contains(&h) {
        bail!("hour out of range for 12-hour time: {h}");
    }
    Ok(h)
}

fn validate_hour_24(h: i64) -> Result<i64> {
    if !(0..=23).contains(&h) {
        bail!("hour out of range for 24-hour time: {h}");
    }
    Ok(h)
}

fn to_24h(h12: i64, mer: Meridiem) -> i64 {
    match (h12, mer) {
        (12, Meridiem::Am) => 0,
        (12, Meridiem::Pm) => 12,
        (_, Meridiem::Am) => h12,
        (_, Meridiem::Pm) => h12.saturating_add(12),
    }
}

fn to_12h(h24: i64) -> Result<(i64, Meridiem)> {
    let h24 = validate_hour_24(h24)?;
    let mer = if h24 >= 12 {
        Meridiem::Pm
    } else {
        Meridiem::Am
    };
    let h12 = match h24.rem_euclid(12) {
        0 => 12,
        n => n,
    };
    Ok((h12, mer))
}

fn guess_meridiem(h12: i64) -> Meridiem {
    // 6:00..11:59 => AM, 12:00..5:59 => PM.
    if (6..=11).contains(&h12) {
        Meridiem::Am
    } else {
        Meridiem::Pm
    }
}

fn hms_to_seconds(h: i64, m: i64, s: i64) -> Result<PanTime> {
    let h = validate_hour_24(h)?;

    let secs = h
        .checked_mul(SECONDS_PER_HOUR)
        .context("hour overflow")?
        .checked_add(
            m.checked_mul(SECONDS_PER_MINUTE)
                .context("minute overflow")?,
        )
        .context("time overflow")?
        .checked_add(s)
        .context("time overflow")?;

    Ok(secs)
}

fn hms_to_seconds_elapsed(h: i64, m: i64, s: i64) -> Result<PanTime> {
    if h < 0 {
        bail!("elapsed hours must be non-negative, got {h}");
    }

    let secs = h
        .checked_mul(SECONDS_PER_HOUR)
        .context("hour overflow")?
        .checked_add(
            m.checked_mul(SECONDS_PER_MINUTE)
                .context("minute overflow")?,
        )
        .context("time overflow")?
        .checked_add(s)
        .context("time overflow")?;

    Ok(secs)
}

fn split_hms(n: PanTime) -> Result<(i64, i64, i64)> {
    let n = validate_non_negative(n)?;
    let h = n.div_euclid(SECONDS_PER_HOUR);
    let rem = n.rem_euclid(SECONDS_PER_HOUR);
    let m = rem.div_euclid(SECONDS_PER_MINUTE);
    let s = rem.rem_euclid(SECONDS_PER_MINUTE);

    let h = validate_hour_24(h)?;
    Ok((h, m, s))
}

fn split_hms_elapsed(n: PanTime) -> Result<(i64, i64, i64)> {
    let n = validate_non_negative(n)?;
    let h = n.div_euclid(SECONDS_PER_HOUR);
    let rem = n.rem_euclid(SECONDS_PER_HOUR);
    let m = rem.div_euclid(SECONDS_PER_MINUTE);
    let s = rem.rem_euclid(SECONDS_PER_MINUTE);
    Ok((h, m, s))
}

fn validate_non_negative(n: PanTime) -> Result<PanTime> {
    if n < 0 {
        bail!("expected non-negative seconds, got {n}");
    }
    Ok(n)
}

fn format_interval_parts(total_seconds: i64) -> Result<(i64, i64, i64)> {
    let n = validate_non_negative(total_seconds)?;
    let h = n.div_euclid(SECONDS_PER_HOUR);
    let rem = n.rem_euclid(SECONDS_PER_HOUR);
    let m = rem.div_euclid(SECONDS_PER_MINUTE);
    let s = rem.rem_euclid(SECONDS_PER_MINUTE);
    Ok((h, m, s))
}

fn format_interval_unsigned(total_seconds: i64) -> String {
    let Ok((h, m, s)) = format_interval_parts(total_seconds) else {
        // ...best-effort fallback for formatting only...
        return "0:00".to_string();
    };

    if s != 0 {
        return format!("{h}:{m:02}:{s:02}");
    }
    format!("{h}:{m:02}")
}

fn format_interval_signed(total_seconds: i64) -> String {
    if total_seconds < 0 {
        let abs = total_seconds.saturating_abs();
        return format!("-{}", format_interval_unsigned(abs));
    }
    format_interval_unsigned(total_seconds)
}

mod film;
mod tickcount;
mod timecode;

pub use film::{feetandframes, kcadd, kcdiff, kcframes, kcoutfromlength};
pub use tickcount::tickcount;
pub use timecode::{
    outcode, tc24to30, tc30to24, tcadd, tcdiff, tcframes, timecode,
};

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
    use ctb_utilities::anyhow::ensure;

    use super::*;

    #[crate::ctb_test]
    fn now_is_in_day_range() -> Result<()> {
        let n = now()?;
        ensure!((0..SECONDS_PER_DAY).contains(&n), "got {n}");
        Ok(())
    }

    #[crate::ctb_test]
    fn seconds_parses_doc_examples() -> Result<()> {
        ensure!(seconds("4:13 PM")? == 58_380);
        ensure!(seconds("11:00 AM")? == 39_600);
        ensure!(seconds("2:30")? == 9_000);
        ensure!(seconds("18:45")? == 67_500);
        Ok(())
    }

    #[crate::ctb_test]
    fn timepattern_doc_examples() -> Result<()> {
        let n = 16 * SECONDS_PER_HOUR + 32 * SECONDS_PER_MINUTE + 17;
        ensure!(timepattern(n, "hh:mm:ss am/pm")? == "4:32:17 pm");
        ensure!(timepattern(n, "hh:mm am/pm")? == "4:32 pm");
        ensure!(timepattern(n, "hh:mm:ss")? == "16:32:17");
        Ok(())
    }

    #[crate::ctb_test]
    fn time_lenient_parsing_examples() -> Result<()> {
        ensure!(
            time("230")? == 14 * SECONDS_PER_HOUR + 30 * SECONDS_PER_MINUTE
        );
        ensure!(time("4p")? == 16 * SECONDS_PER_HOUR);
        ensure!(
            time("425 pm")? == 16 * SECONDS_PER_HOUR + 25 * SECONDS_PER_MINUTE
        );
        ensure!(time("midnight")? == 0);
        ensure!(time("noon")? == 12 * SECONDS_PER_HOUR);
        ensure!(time("evening")? == 18 * SECONDS_PER_HOUR);
        Ok(())
    }

    #[crate::ctb_test]
    fn time24_wraps_over_midnight() -> Result<()> {
        let start = 22 * SECONDS_PER_HOUR + 30 * SECONDS_PER_MINUTE; // 10:30 PM
        let duration = 4 * SECONDS_PER_HOUR;
        let end = time24(start + duration);
        ensure!(end == 2 * SECONDS_PER_HOUR + 30 * SECONDS_PER_MINUTE);
        Ok(())
    }

    #[crate::ctb_test]
    fn timedifference_and_timeinterval_doc_examples() -> Result<()> {
        let start = time("9:30 PM")?;
        let end = time("2:05 AM")?;

        ensure!(
            timedifference(start, end)
                == 4 * SECONDS_PER_HOUR + 35 * SECONDS_PER_MINUTE
        );
        ensure!(
            timedifference(end, start)
                == -(4 * SECONDS_PER_HOUR + 35 * SECONDS_PER_MINUTE)
        );

        ensure!(
            timeinterval(start, end)
                == 4 * SECONDS_PER_HOUR + 35 * SECONDS_PER_MINUTE
        );
        ensure!(
            timeinterval(end, start)
                == 19 * SECONDS_PER_HOUR + 25 * SECONDS_PER_MINUTE
        );

        Ok(())
    }

    #[crate::ctb_test]
    fn text_time_interval_helpers_format_like_docs() -> Result<()> {
        ensure!(texttimedifference("9:30 PM", "2:05 AM")? == "4:35");
        ensure!(texttimedifference("2:05 AM", "9:30 PM")? == "-4:35");
        ensure!(texttimeinterval("2:05 AM", "9:30 PM")? == "19:25");
        Ok(())
    }

    #[crate::ctb_test]
    fn timestr_is_uppercase_ampm() -> Result<()> {
        let n = time("9:34 AM")?;
        ensure!(timestr(n)? == "9:34 AM");
        Ok(())
    }
}
