/* SPDX-License-Identifier: MIT */

//! Pan date formatting and parsing built on Julian day numbers.
//! Day format is days between 1/1/4713 BC and the given date, with
//! adjustment for Gregorian calendar.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use chrono::{Datelike, Local, Timelike};

/// A Pan date represented as a Julian day number (JDN).
pub type PanDate = i64;

/// A combined date/time stored as seconds since the Pan superdate epoch.
pub type PanSuperDate = i64;

const GREGORIAN_START_JDN: PanDate = 2_299_161; // 1582-10-15 (Gregorian)
const GREGORIAN_START_YMD: (i32, u32, u32) = (1582, 10, 15);
const JULIAN_END_YMD: (i32, u32, u32) = (1582, 10, 4);

const SUPERDATE_EPOCH_JDN: PanDate = 2_416_481; // 1904-01-01 (Gregorian)
const SUPERDATE_MIN_YEAR: i32 = 1904;
const SUPERDATE_MAX_YEAR: i32 = 2040;
const SUPERDATE_SECONDS_PER_DAY: i64 = 86_400;

const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

const MONTH_ABBR: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct",
    "Nov", "Dec",
];

const WEEKDAY_NAMES: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];

const WEEKDAY_ABBR: [&str; 7] =
    ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Calendar {
    Julian,
    Gregorian,
}

/// Converts a year-month-day date into a Pan JDN value. Gregorian calendar
/// begins at 1582-10-15; dates 1582-10-05..14 are rejected.
pub fn datevalue(year: i32, month: u32, day: u32) -> Result<PanDate> {
    let cal = calendar_for_ymd(year, month, day)?;
    ymd_to_jdn(cal, year, month, day)
}

/// Converts a Pan JDN value into year, month, and day. Before 1582-10-15 dates
/// are in Julian calendar.
pub fn ymd_from_date(date: PanDate) -> Result<(i32, u32, u32)> {
    jdn_to_ymd(date)
}

/// Returns today's local date as a Pan JDN value.
pub fn today() -> Result<PanDate> {
    let d = Local::now().date_naive();
    datevalue(d.year(), d.month(), d.day())
}

/// Returns the day of week for `date` (Sunday = 0, Saturday = 6).
pub fn dayofweek(date: PanDate) -> i64 {
    (date.rem_euclid(7).saturating_add(1)).rem_euclid(7)
}

/// Returns the day of month for `date`.
pub fn dayvalue(date: PanDate) -> Result<u32> {
    let (_y, _m, d) = ymd_from_date(date)?;
    Ok(d)
}

/// Returns the month number for `date` (1-12).
pub fn monthvalue(date: PanDate) -> Result<u32> {
    let (_y, m, _d) = ymd_from_date(date)?;
    Ok(m)
}

/// Returns the year for `date`.
pub fn yearvalue(date: PanDate) -> Result<i32> {
    let (y, _m, _d) = ymd_from_date(date)?;
    Ok(y)
}

/// Returns the first day of the month containing `date`.
pub fn month1st(date: PanDate) -> Result<PanDate> {
    let (y, m, _d) = ymd_from_date(date)?;
    datevalue(y, m, 1)
}

/// Returns the number of days in the month containing `date`.
pub fn monthlength(date: PanDate) -> Result<i64> {
    let (y, m, _d) = ymd_from_date(date)?;
    let this = datevalue(y, m, 1)?;
    let (ny, nm) = add_months_ym(y, m, 1)?;
    let next = datevalue(ny, nm, 1)?;
    Ok(next.saturating_sub(this))
}

/// Adds `offset` months, clamping the day to the new month length.
pub fn monthmath(date: PanDate, offset: i64) -> Result<PanDate> {
    let (y, m, d) = ymd_from_date(date)?;
    let (ty, tm) = add_months_ym(y, m, offset)?;
    let max_day = month_length_ym(ty, tm)?;
    let day = if d > max_day { max_day } else { d };
    datevalue(ty, tm, day)
}

/// Returns the first day of the quarter containing `date`.
pub fn quarter1st(date: PanDate) -> Result<PanDate> {
    let (y, m, _d) = ymd_from_date(date)?;
    let quarter = (m.saturating_sub(1)) / 3;
    let qm = quarter.saturating_mul(3).saturating_add(1);
    datevalue(y, qm, 1)
}

/// Returns the quarter number (1-4) for `date`.
pub fn quartervalue(date: PanDate) -> Result<u32> {
    let (_y, m, _d) = ymd_from_date(date)?;
    Ok(((m.saturating_sub(1)) / 3).saturating_add(1))
}

/// Returns the first day of the week containing `date` (Sunday start).
pub fn week1st(date: PanDate) -> PanDate {
    let dow = dayofweek(date);
    date.saturating_sub(dow)
}

/// Returns the first day of the year containing `date`.
pub fn year1st(date: PanDate) -> Result<PanDate> {
    let (y, _m, _d) = ymd_from_date(date)?;
    datevalue(y, 1, 1)
}

/// Returns the 1-based week number within the year.
pub fn weekvalue(date: PanDate) -> Result<i64> {
    let y1 = year1st(date)?;
    let delta = date.saturating_sub(y1);
    Ok(delta.div_euclid(7).saturating_add(1))
}

/// Formats `date` as `m/d/yy`.
pub fn datestr(date: PanDate) -> Result<String> {
    let (y, m, d) = ymd_from_date(date)?;
    let yy = y.rem_euclid(100);
    Ok(format!("{m}/{d}/{yy:02}"))
}

/// Formats `date` as `dd-MON-YYYY`. e.g. 20-APR-2003
pub fn eurodatestr(date: PanDate) -> Result<String> {
    let (y, m, d) = ymd_from_date(date)?;
    let mon = month_abbr(m)?;
    Ok(format!("{:02}-{}-{:04}", d, mon.to_uppercase(), y))
}

/// Returns the full weekday name for `date`.
pub fn daystr(date: PanDate) -> Result<String> {
    let idx = usize::try_from(dayofweek(date))
        .context("day of week did not fit into usize")?;
    let Some(name) = WEEKDAY_NAMES.get(idx) else {
        bail!("day of week index out of range: {idx}");
    };
    Ok((*name).to_string())
}

/// Formats `date` as `Month ddnth, yyyy`.
pub fn longdatestr(date: PanDate) -> Result<String> {
    datepattern(date, "Month ddnth, yyyy")
}

/// Formats `date` with weekday and long date.
pub fn completedatestr(date: PanDate) -> Result<String> {
    let day = daystr(date)?;
    let long = longdatestr(date)?;
    Ok(format!("{day}, {long}"))
}

/// Formats `date` using a Pan date pattern.
///
/// Supported tokens include `yy`, `yyyy`, `mm`/`MM`, `mmnth`, `dd`/`DD`,
/// `ddnth`, `mon`, `month`, `day`, `dayofweek`, `dow`, `ww`, `wwnth`,
/// `qq`, `qqq`, `qtr`, and `quarter`. Double-quoted literals are copied
/// verbatim.
pub fn datepattern(date: PanDate, pattern: &str) -> Result<String> {
    let (y, m, d) = ymd_from_date(date)?;
    let yy = y.rem_euclid(100);
    let dow = dayofweek(date);
    let qq = ((m.saturating_sub(1)) / 3).saturating_add(1);
    let ww = weekvalue(date)?;

    let mut out = String::with_capacity(pattern.len().saturating_add(16));

    let mut i = 0usize;
    while i < pattern.len() {
        let slice = pattern.get(i..).context("Invalid pattern offset")?;

        // Quoted literal: "..."
        if slice.starts_with('"') {
            let rest = pattern
                .get((i.saturating_add(1))..)
                .context("Invalid pattern offset")?;
            let Some(end_rel) = rest.find('"') else {
                bail!("unterminated quote in date pattern");
            };
            let lit = rest.get(..end_rel).context("Invalid quote offset")?;
            out.push_str(lit);
            i = i
                .checked_add(1)
                .context("pattern index overflow")?
                .checked_add(end_rel)
                .context("pattern index overflow")?
                .checked_add(1)
                .context("pattern index overflow")?;
            continue;
        }

        let Some((tok, style, len)) = match_token(slice) else {
            let ch = slice
                .chars()
                .next()
                .context("failed to read next pattern char")?;
            out.push(ch);
            i = i.saturating_add(ch.len_utf8());
            continue;
        };

        apply_token(&mut out, tok, style, y, yy, m, d, dow, qq, ww)?;
        i = i.saturating_add(len);
    }

    Ok(out)
}

#[derive(Debug, Clone, Copy)]
enum Token {
    Yy,
    Yyyy,

    Mm, // month numeric, no leading zero
    MM, // month numeric, leading zero
    Mmnth,

    Dd, // day numeric, no leading zero
    DD, // day numeric, leading zero
    Ddnth,

    Mon,
    Month,

    Day,
    DayOfWeek,
    Dow,

    Ww,
    Wwnth,

    Qq,
    Qqq,     // quarter numeric + 'q'/'Q'
    Qtr,     // ordinal quarter number (e.g. 2nd)
    Quarter, // spelled out (e.g. second)
}

#[derive(Debug, Clone, Copy)]
enum CaseStyle {
    Lower,
    Title,
    Upper,
}

fn case_style_from_token(token: &str) -> CaseStyle {
    let has_alpha = token.chars().any(|c| c.is_ascii_alphabetic());
    let all_lower = has_alpha
        && token
            .chars()
            .all(|c| !c.is_ascii_alphabetic() || c.is_ascii_lowercase());
    let all_upper = has_alpha
        && token
            .chars()
            .all(|c| !c.is_ascii_alphabetic() || c.is_ascii_uppercase());

    if all_upper {
        return CaseStyle::Upper;
    }
    if all_lower {
        return CaseStyle::Lower;
    }
    CaseStyle::Title
}

fn apply_case(s: &str, style: CaseStyle) -> String {
    match style {
        CaseStyle::Lower => s.to_lowercase(),
        CaseStyle::Upper => s.to_uppercase(),
        CaseStyle::Title => {
            let mut it = s.chars();
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

fn quarter_word(q: u32) -> Result<&'static str> {
    match q {
        1 => Ok("first"),
        2 => Ok("second"),
        3 => Ok("third"),
        4 => Ok("fourth"),
        _ => bail!("quarter out of range: {q}"),
    }
}

fn ordinal_suffix_i64(n: i64) -> &'static str {
    let d = n.rem_euclid(100);
    if (11..=13).contains(&d) {
        return "th";
    }
    match n.rem_euclid(10) {
        1 => "st",
        2 => "nd",
        3 => "rd",
        _ => "th",
    }
}

fn match_token(slice: &str) -> Option<(Token, CaseStyle, usize)> {
    // Longest-first to avoid prefix collisions (e.g. dayofweek vs day).
    const SPECS: &[(&str, Token, bool)] = &[
        ("dayofweek", Token::DayOfWeek, false),
        ("ddnth", Token::Ddnth, false),
        ("mmnth", Token::Mmnth, false),
        ("wwnth", Token::Wwnth, false),
        ("quarter", Token::Quarter, false),
        ("yyyy", Token::Yyyy, false),
        ("qqq", Token::Qqq, false),
        ("qtr", Token::Qtr, false),
        ("month", Token::Month, false),
        ("mon", Token::Mon, false),
        ("dow", Token::Dow, false),
        ("day", Token::Day, false),
        ("yy", Token::Yy, false),
        // Case-sensitive numeric formatting tokens.
        ("MM", Token::MM, true),
        ("mm", Token::Mm, true),
        ("DD", Token::DD, true),
        ("dd", Token::Dd, true),
        ("ww", Token::Ww, false),
        ("qq", Token::Qq, false),
    ];

    for (raw, tok, case_sensitive) in SPECS {
        let len = raw.len();
        if slice.len() < len {
            continue;
        }
        let Some(head) = slice.get(..len) else {
            continue;
        };
        let matches = if *case_sensitive {
            head == *raw
        } else {
            head.eq_ignore_ascii_case(raw)
        };
        if matches {
            return Some((*tok, case_style_from_token(head), len));
        }
    }

    None
}

fn apply_token(
    out: &mut String,
    tok: Token,
    style: CaseStyle,
    y: i32,
    yy: i32,
    m: u32,
    d: u32,
    dow: i64,
    qq: u32,
    ww: i64,
) -> Result<()> {
    use std::fmt::Write;
    match tok {
        Token::Yy => {
            let _ = write!(out, "{yy:02}");
        }
        Token::Yyyy => {
            let _ = write!(out, "{y:04}");
        }

        Token::Mm => {
            let _ = write!(out, "{m}");
        }
        Token::MM => {
            let _ = write!(out, "{m:02}");
        }
        Token::Mmnth => {
            let mi = i64::from(m);
            let _ = write!(out, "{m}{}", ordinal_suffix_i64(mi));
        }

        Token::Dd => {
            let _ = write!(out, "{d}");
        }
        Token::DD => {
            let _ = write!(out, "{d:02}");
        }
        Token::Ddnth => {
            let _ = write!(out, "{}{}", d, ordinal_suffix(d));
        }

        Token::Mon => {
            let mon = month_abbr(m)?;
            out.push_str(&apply_case(mon, style));
        }
        Token::Month => {
            let mon = month_name(m)?;
            out.push_str(&apply_case(mon, style));
        }

        Token::Day => {
            let ab = weekday_abbr(dow)?;
            out.push_str(&apply_case(ab, style));
        }
        Token::DayOfWeek => {
            let full = daystr_from_dow(dow)?;
            out.push_str(&apply_case(full, style));
        }
        Token::Dow => {
            let _ = write!(out, "{dow}");
        }

        Token::Ww => {
            let _ = write!(out, "{ww}");
        }
        Token::Wwnth => {
            let _ = write!(out, "{ww}{}", ordinal_suffix_i64(ww));
        }

        Token::Qq => {
            let _ = write!(out, "{qq}");
        }
        Token::Qqq => {
            let qch = match style {
                CaseStyle::Upper => 'Q',
                _ => 'q',
            };
            let _ = write!(out, "{qq}{qch}");
        }
        Token::Qtr => {
            let qi = i64::from(qq);
            let _ = write!(out, "{qq}{}", ordinal_suffix_i64(qi));
        }
        Token::Quarter => {
            let w = quarter_word(qq)?;
            out.push_str(&apply_case(w, style));
        }
    }

    Ok(())
}

fn daystr_from_dow(dow: i64) -> Result<&'static str> {
    let idx =
        usize::try_from(dow).context("day of week did not fit into usize")?;
    let Some(name) = WEEKDAY_NAMES.get(idx) else {
        bail!("day of week index out of range: {idx}");
    };
    Ok(*name)
}

/// Formats a date with a short, friendly representation.
/// - Today: "Today"
/// - Within 6 months: "Wed, Apr 20"
/// - Older: "4/20/03"
pub fn naturaldatestr(date: PanDate) -> Result<String> {
    let t = today()?;
    naturaldatestr_with_today(date, t)
}

/// Parses a human-friendly date string into a Pan JDN value.
pub fn date(text: &str) -> Result<PanDate> {
    let t = today()?;
    date_with_today(text, t)
}

/// Formats `date` relative to the provided `today` reference.
fn naturaldatestr_with_today(date: PanDate, today: PanDate) -> Result<String> {
    if date == today {
        return Ok("Today".to_string());
    }

    let delta = today.saturating_sub(date);
    if (0..=180).contains(&delta) {
        let (_y, m, d) = ymd_from_date(date)?;
        let w = weekday_abbr(dayofweek(date))?;
        let mon = month_abbr(m)?;
        return Ok(format!("{w}, {mon} {d}"));
    }

    datestr(date)
}

/// Parses a date string with a provided `today` reference.
fn date_with_today(text: &str, today: PanDate) -> Result<PanDate> {
    let raw = text.trim();
    if raw.is_empty() {
        bail!("empty date string");
    }

    // Slash formats: mm/dd/yy or mm/dd/yyyy
    if raw.contains('/') {
        let parts: Vec<&str> = raw.split('/').collect();
        if parts.len() != 3 {
            bail!("invalid mm/dd/yy format: {raw}");
        }

        let month = parts
            .first()
            .context("Missing month part")?
            .trim()
            .parse::<u32>()
            .context("month")?;
        let day = parts
            .get(1)
            .context("Missing day part")?
            .trim()
            .parse::<u32>()
            .context("day")?;
        let mut year = parts
            .get(2)
            .context("Missing year part")?
            .trim()
            .parse::<i32>()
            .context("year")?;
        if (0..100).contains(&year) {
            year = expand_two_digit_year(year)?;
        }

        return datevalue(year, month, day);
    }

    // European-ish: dd-MON-YYYY (not documented for `date()`, but useful)
    if raw.contains('-') {
        let parts: Vec<&str> = raw.split('-').collect();
        if parts.len() == 3 {
            let day = parts
                .first()
                .context("Missing day part")?
                .trim()
                .parse::<u32>()
                .context("day")?;
            let month =
                parse_month_name(parts.get(1).context("Missing month part")?)?;
            let year = parts
                .get(2)
                .context("Missing year part")?
                .trim()
                .parse::<i32>()
                .context("year")?;
            return datevalue(year, month, day);
        }
    }

    // Normalize for keyword parsing.
    let lower = raw.to_lowercase();
    let norm = lower.split_whitespace().collect::<Vec<_>>().join(" ");

    // last/next weekday
    if let Some(rest) = norm.strip_prefix("last ") {
        let target = parse_weekday_name(rest)?;
        let base = week1st(today);
        let candidate = base.saturating_add(target).saturating_sub(7);
        return Ok(candidate);
    }
    if let Some(rest) = norm.strip_prefix("next ") {
        let target = parse_weekday_name(rest)?;
        let base = week1st(today);
        let candidate = base.saturating_add(target).saturating_add(7);
        return Ok(candidate);
    }

    // weekday in current week
    if let Ok(target) = parse_weekday_name(&norm) {
        let base = week1st(today);
        return Ok(base.saturating_add(target));
    }

    // Month dd, yyyy / Mon dd, yyyy
    // Accept optional comma after the day.
    let parts = raw.split_whitespace().collect::<Vec<_>>();
    if parts.len() >= 3 {
        let month =
            parse_month_name(parts.first().context("Missing month part")?)?;
        let day_str = parts
            .get(1)
            .context("Missing day part")?
            .trim_end_matches(',');
        let day = day_str.parse::<u32>().context("day")?;
        let year_str = parts
            .get(2)
            .context("Missing year part")?
            .trim_start_matches(',');
        let year = year_str.parse::<i32>().context("year")?;
        return datevalue(year, month, day);
    }

    bail!("unsupported date format: {raw}");
}

fn expand_two_digit_year(yy: i32) -> Result<i32> {
    // Conventional pivot (can be adjusted later if needed).
    if (0..=69).contains(&yy) {
        Ok(2000i32.saturating_add(yy))
    } else if (70..=99).contains(&yy) {
        Ok(1900i32.saturating_add(yy))
    } else {
        bail!("two-digit year out of range: {yy}");
    }
}

fn weekday_abbr(dow: i64) -> Result<&'static str> {
    let idx = usize::try_from(dow).context("weekday did not fit into usize")?;
    let Some(s) = WEEKDAY_ABBR.get(idx) else {
        bail!("weekday index out of range: {idx}");
    };
    Ok(*s)
}

fn parse_weekday_name(s: &str) -> Result<i64> {
    let t = s.trim().to_lowercase();
    let t = t.as_str();

    for (idx, name) in WEEKDAY_NAMES.iter().enumerate() {
        if t == name.to_lowercase() {
            let v = i64::try_from(idx).context("weekday idx")?;
            return Ok(v);
        }
    }

    for (idx, abbr) in WEEKDAY_ABBR.iter().enumerate() {
        if t == abbr.to_lowercase() {
            let v = i64::try_from(idx).context("weekday idx")?;
            return Ok(v);
        }
    }

    bail!("unrecognized weekday: {s}");
}

fn parse_month_name(s: &str) -> Result<u32> {
    let t = s.trim().trim_end_matches(',').to_lowercase();

    for (i, name) in MONTH_NAMES.iter().enumerate() {
        if t == name.to_lowercase() {
            return u32::try_from(i.saturating_add(1)).context("month idx");
        }
    }
    for (i, abbr) in MONTH_ABBR.iter().enumerate() {
        if t == abbr.to_lowercase() {
            return u32::try_from(i.saturating_add(1)).context("month idx");
        }
    }

    bail!("unrecognized month name: {s}");
}

fn month_name(month: u32) -> Result<&'static str> {
    let idx = usize::try_from(month)
        .context("month did not fit into usize")?
        .checked_sub(1)
        .context("month is 0")?;
    let Some(s) = MONTH_NAMES.get(idx) else {
        bail!("month out of range: {month}");
    };
    Ok(*s)
}

fn month_abbr(month: u32) -> Result<&'static str> {
    let idx = usize::try_from(month)
        .context("month did not fit into usize")?
        .checked_sub(1)
        .context("month is 0")?;
    let Some(s) = MONTH_ABBR.get(idx) else {
        bail!("month out of range: {month}");
    };
    Ok(*s)
}

fn ordinal_suffix(day: u32) -> &'static str {
    let d = day % 100;
    if (11..=13).contains(&d) {
        return "th";
    }
    match day % 10 {
        1 => "st",
        2 => "nd",
        3 => "rd",
        _ => "th",
    }
}

/// Selects the calendar around the Gregorian cutover date.
fn calendar_for_ymd(year: i32, month: u32, day: u32) -> Result<Calendar> {
    if (year, month, day) >= GREGORIAN_START_YMD {
        return Ok(Calendar::Gregorian);
    }
    if (year, month, day) <= JULIAN_END_YMD {
        return Ok(Calendar::Julian);
    }
    bail!("invalid date in Gregorian cutover gap: {year}-{month:02}-{day:02}");
}

/// Converts a year-month-day date to a Julian day number.
fn ymd_to_jdn(
    cal: Calendar,
    year: i32,
    month: u32,
    day: u32,
) -> Result<PanDate> {
    let y = i64::from(year);
    let m = i64::from(month);
    let d = i64::from(day);

    let a = 14_i64.saturating_sub(m).div_euclid(12);
    let y2 = y.saturating_add(4800).saturating_sub(a);
    let m2 = m.saturating_add(12_i64.saturating_mul(a)).saturating_sub(3);

    let jdn = match cal {
        Calendar::Gregorian => d
            .saturating_add(
                (153_i64.saturating_mul(m2).saturating_add(2)).div_euclid(5),
            )
            .saturating_add(365_i64.saturating_mul(y2))
            .saturating_add(y2.div_euclid(4))
            .saturating_sub(y2.div_euclid(100))
            .saturating_add(y2.div_euclid(400))
            .saturating_sub(32045),
        Calendar::Julian => d
            .saturating_add(
                (153_i64.saturating_mul(m2).saturating_add(2)).div_euclid(5),
            )
            .saturating_add(365_i64.saturating_mul(y2))
            .saturating_add(y2.div_euclid(4))
            .saturating_sub(32083),
    };

    Ok(jdn)
}

/// Converts a Julian day number into year, month, and day.
fn jdn_to_ymd(jdn: PanDate) -> Result<(i32, u32, u32)> {
    if jdn >= GREGORIAN_START_JDN {
        jdn_to_ymd_gregorian(jdn)
    } else {
        jdn_to_ymd_julian(jdn)
    }
}

fn jdn_to_ymd_gregorian(jdn: PanDate) -> Result<(i32, u32, u32)> {
    let a = jdn.saturating_add(32044);
    let b = (4_i64.saturating_mul(a).saturating_add(3)).div_euclid(146097);
    let c = a.saturating_sub((146097_i64.saturating_mul(b)).div_euclid(4));
    let d = (4_i64.saturating_mul(c).saturating_add(3)).div_euclid(1461);
    let e = c.saturating_sub((1461_i64.saturating_mul(d)).div_euclid(4));
    let m = (5_i64.saturating_mul(e).saturating_add(2)).div_euclid(153);

    let day = e
        .saturating_sub(
            (153_i64.saturating_mul(m).saturating_add(2)).div_euclid(5),
        )
        .saturating_add(1);
    let month = m
        .saturating_add(3)
        .saturating_sub(12_i64.saturating_mul(m.div_euclid(10)));
    let year = 100_i64
        .saturating_mul(b)
        .saturating_add(d)
        .saturating_sub(4800)
        .saturating_add(m.div_euclid(10));

    let y = i32::try_from(year).context("year did not fit into i32")?;
    let mo = u32::try_from(month).context("month did not fit into u32")?;
    let da = u32::try_from(day).context("day did not fit into u32")?;
    Ok((y, mo, da))
}

fn jdn_to_ymd_julian(jdn: PanDate) -> Result<(i32, u32, u32)> {
    let c = jdn.saturating_add(32082);
    let d = (4_i64.saturating_mul(c).saturating_add(3)).div_euclid(1461);
    let e = c.saturating_sub((1461_i64.saturating_mul(d)).div_euclid(4));
    let m = (5_i64.saturating_mul(e).saturating_add(2)).div_euclid(153);

    let day = e
        .saturating_sub(
            (153_i64.saturating_mul(m).saturating_add(2)).div_euclid(5),
        )
        .saturating_add(1);
    let month = m
        .saturating_add(3)
        .saturating_sub(12_i64.saturating_mul(m.div_euclid(10)));
    let year = d.saturating_sub(4800).saturating_add(m.div_euclid(10));

    let y = i32::try_from(year).context("year did not fit into i32")?;
    let mo = u32::try_from(month).context("month did not fit into u32")?;
    let da = u32::try_from(day).context("day did not fit into u32")?;
    Ok((y, mo, da))
}

/// Adds `offset` months to a year-month pair.
fn add_months_ym(year: i32, month: u32, offset: i64) -> Result<(i32, u32)> {
    let y = i64::from(year);
    let m0 = i64::from(month)
        .checked_sub(1)
        .context("month out of range")?;
    let total = y
        .checked_mul(12)
        .context("year overflow")?
        .checked_add(m0)
        .context("month overflow")?
        .checked_add(offset)
        .context("offset overflow")?;

    let ty = total.div_euclid(12);
    let tm0 = total.rem_euclid(12);
    let tm = tm0.saturating_add(1);

    let out_y = i32::try_from(ty).context("year did not fit into i32")?;
    let out_m = u32::try_from(tm).context("month did not fit into u32")?;
    Ok((out_y, out_m))
}

/// Returns the month length, honoring the Gregorian cutover.
fn month_length_ym(year: i32, month: u32) -> Result<u32> {
    let cal = calendar_for_ymd(year, month, 1)?;
    let mut len = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            let leap = match cal {
                Calendar::Gregorian => is_leap_year_gregorian(year),
                Calendar::Julian => is_leap_year_julian(year),
            };
            if leap { 29 } else { 28 }
        }
        _ => bail!("month out of range: {month}"),
    };

    // Account for Gregorian cutover month length when relevant: October 1582
    // loses 10 calendar dates (5..14) under the historical reform.
    if (year, month) == (1582, 10) {
        len = 21;
    }

    Ok(len)
}

fn is_leap_year_gregorian(year: i32) -> bool {
    (year.rem_euclid(4) == 0 && year.rem_euclid(100) != 0)
        || year.rem_euclid(400) == 0
}

fn is_leap_year_julian(year: i32) -> bool {
    year.rem_euclid(4) == 0
}

/// Combines a date and seconds into a superdate timestamp.
pub fn superdate(date: PanDate, time: i64) -> Result<PanSuperDate> {
    let (y, _m, _d) = ymd_from_date(date)?;
    if !(SUPERDATE_MIN_YEAR..=SUPERDATE_MAX_YEAR).contains(&y) {
        bail!(
            "superdate date out of range: {y} (supported {SUPERDATE_MIN_YEAR}..={SUPERDATE_MAX_YEAR})"
        );
    }
    if !(0..SUPERDATE_SECONDS_PER_DAY).contains(&time) {
        bail!(
            "superdate time out of range: {time} (expected 0..{SUPERDATE_SECONDS_PER_DAY})"
        );
    }

    let days = date
        .checked_sub(SUPERDATE_EPOCH_JDN)
        .context("date before superdate epoch")?;
    let base = days
        .checked_mul(SUPERDATE_SECONDS_PER_DAY)
        .context("superdate overflow (days)")?;
    base.checked_add(time).context("superdate overflow (time)")
}

/// Extracts the date component from a superdate timestamp.
pub fn regulardate(number: PanSuperDate) -> Result<PanDate> {
    let (days, _secs) = split_superdate(number)?;
    let date = SUPERDATE_EPOCH_JDN
        .checked_add(days)
        .context("superdate overflow converting to date")?;

    let (y, _m, _d) = ymd_from_date(date)?;
    if !(SUPERDATE_MIN_YEAR..=SUPERDATE_MAX_YEAR).contains(&y) {
        bail!("superdate date out of range after conversion: {y}");
    }

    Ok(date)
}

/// Extracts the seconds-since-midnight from a superdate timestamp.
pub fn regulartime(number: PanSuperDate) -> Result<i64> {
    let (_days, secs) = split_superdate(number)?;
    Ok(secs)
}

/// Formats a superdate as `m/d/yy h:mm AM/PM`.
pub fn superdatestr(number: PanSuperDate) -> Result<String> {
    let d = regulardate(number)?;
    let t = regulartime(number)?;
    let ds = datestr(d)?;
    let ts = format_time_ampm(t, false)?;
    Ok(format!("{ds} {ts}"))
}

/// Formats a superdate including seconds.
pub fn superdatesecondsstr(number: PanSuperDate) -> Result<String> {
    let d = regulardate(number)?;
    let t = regulartime(number)?;
    let ds = datestr(d)?;
    let ts = format_time_ampm(t, true)?;
    Ok(format!("{ds} {ts}"))
}

/// Formats a superdate using separate date and time patterns.
pub fn superdatepattern(
    number: PanSuperDate,
    date_pat: &str,
    time_pat: &str,
) -> Result<String> {
    let d = regulardate(number)?;
    let t = regulartime(number)?;
    let ds = datepattern(d, date_pat)?;
    let ts = super::time::timepattern(t, time_pat)?;
    Ok(format!("{ds}{ts}"))
}

/// Returns the current local time as a superdate timestamp.
pub fn supernow() -> Result<PanSuperDate> {
    let now = Local::now();
    let d = now.date_naive();
    let t = now.time();

    let date = datevalue(d.year(), d.month(), d.day())?;
    let secs = i64::from(t.num_seconds_from_midnight());
    superdate(date, secs)
}

fn split_superdate(number: PanSuperDate) -> Result<(i64, i64)> {
    if number < 0 {
        bail!("superdate must be non-negative: {number}");
    }
    let days = number.div_euclid(SUPERDATE_SECONDS_PER_DAY);
    let secs = number.rem_euclid(SUPERDATE_SECONDS_PER_DAY);
    Ok((days, secs))
}

/// Formats seconds since midnight as a 12-hour time with AM/PM.
fn format_time_ampm(seconds: i64, include_seconds: bool) -> Result<String> {
    if !(0..SUPERDATE_SECONDS_PER_DAY).contains(&seconds) {
        bail!(
            "time seconds out of range: {seconds} (expected 0..{SUPERDATE_SECONDS_PER_DAY})"
        );
    }

    let h24 = seconds.div_euclid(3600);
    let rem = seconds.rem_euclid(3600);
    let min = rem.div_euclid(60);
    let sec = rem.rem_euclid(60);

    let ampm = if h24 >= 12 { "PM" } else { "AM" };
    let h12 = match h24.rem_euclid(12) {
        0 => 12,
        x => x,
    };

    if include_seconds {
        Ok(format!("{h12}:{min:02}:{sec:02} {ampm}"))
    } else {
        Ok(format!("{h12}:{min:02} {ampm}"))
    }
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

    use ctb_utilities::anyhow::ensure;

    #[crate::ctb_test]
    fn datevalue_matches_doc_example() -> Result<()> {
        // Aug 7, 1991 => 2,448,476.
        let n = datevalue(1991, 8, 7)?;
        ensure!(n == 2_448_476, "got {n}");
        Ok(())
    }

    #[crate::ctb_test]
    fn jdn_roundtrip_ymd() -> Result<()> {
        let n = datevalue(2003, 4, 20)?;
        let (y, m, d) = ymd_from_date(n)?;
        ensure!((y, m, d) == (2003, 4, 20), "got {y}-{m}-{d}");
        Ok(())
    }

    #[crate::ctb_test]
    fn dayofweek_known_date() -> Result<()> {
        // 2000-01-01 is Saturday.
        let n = datevalue(2000, 1, 1)?;
        ensure!(dayofweek(n) == 6, "got {}", dayofweek(n));
        ensure!(daystr(n)? == "Saturday");
        Ok(())
    }

    #[crate::ctb_test]
    fn string_formats_examples() -> Result<()> {
        let n = datevalue(2003, 4, 20)?;
        ensure!(datestr(n)? == "4/20/03");
        ensure!(eurodatestr(n)? == "20-APR-2003");
        ensure!(longdatestr(n)? == "April 20th, 2003");
        ensure!(
            completedatestr(n)? == "Sunday, April 20th, 2003",
            "got {}",
            completedatestr(n)?
        );
        Ok(())
    }

    #[crate::ctb_test]
    fn datepattern_documented_example() -> Result<()> {
        let n = datevalue(2003, 5, 12)?;
        let s = datepattern(n, "Month ddnth, yyyy")?;
        ensure!(s == "May 12th, 2003", "got {s}");
        Ok(())
    }

    #[crate::ctb_test]
    fn parse_date_slash_and_monthname() -> Result<()> {
        let n1 = date_with_today("12/9/1979", datevalue(2024, 1, 1)?)?;
        let n2 = datevalue(1979, 12, 9)?;
        ensure!(n1 == n2);

        let n3 = date_with_today("Apr 20, 2003", datevalue(2024, 1, 1)?)?;
        let n4 = datevalue(2003, 4, 20)?;
        ensure!(n3 == n4);

        let n5 = date_with_today("April 20, 2003", datevalue(2024, 1, 1)?)?;
        ensure!(n5 == n4);

        Ok(())
    }

    #[crate::ctb_test]
    fn monthmath_clamps_to_last_day() -> Result<()> {
        // From docs: March 31 + 1 month => April 30.
        let src = datevalue(1997, 3, 31)?;
        let dst = monthmath(src, 1)?;
        let (y, m, d) = ymd_from_date(dst)?;
        ensure!((y, m, d) == (1997, 4, 30), "got {y}-{m}-{d}");
        Ok(())
    }

    #[crate::ctb_test]
    fn monthlength_leap_year_behavior() -> Result<()> {
        let feb_2000 = datevalue(2000, 2, 10)?;
        ensure!(monthlength(feb_2000)? == 29);

        let feb_1900 = datevalue(1900, 2, 10)?;
        ensure!(monthlength(feb_1900)? == 28);

        let feb_1600 = datevalue(1600, 2, 10)?;
        ensure!(monthlength(feb_1600)? == 29);

        Ok(())
    }

    #[crate::ctb_test]
    fn week_helpers() -> Result<()> {
        let weds = datevalue(1995, 7, 12)?;
        let sun = week1st(weds);
        let (y, m, d) = ymd_from_date(sun)?;
        ensure!((y, m, d) == (1995, 7, 9), "got {y}-{m}-{d}");
        Ok(())
    }

    #[crate::ctb_test]
    fn parse_weekday_keywords_are_deterministic_with_today() -> Result<()> {
        // Today: 2024-01-03 (Wednesday). week1st => 2023-12-31 (Sunday).
        let t = datevalue(2024, 1, 3)?;

        let tue = date_with_today("Tuesday", t)?;
        ensure!(ymd_from_date(tue)? == (2024, 1, 2));

        let next_wed = date_with_today("next wed", t)?;
        ensure!(ymd_from_date(next_wed)? == (2024, 1, 10));

        let last_fri = date_with_today("last fri", t)?;
        ensure!(ymd_from_date(last_fri)? == (2023, 12, 29));

        Ok(())
    }

    #[crate::ctb_test]
    fn naturaldatestr_rules() -> Result<()> {
        let t = datevalue(2024, 1, 3)?;
        ensure!(naturaldatestr_with_today(t, t)? == "Today");

        let recent = datevalue(2023, 12, 29)?;
        let s = naturaldatestr_with_today(recent, t)?;
        ensure!(s.starts_with("Fri, "), "got {s}");

        let old = datevalue(2023, 1, 1)?;
        ensure!(naturaldatestr_with_today(old, t)? == "1/1/23");

        let future = datevalue(2024, 2, 1)?;
        ensure!(naturaldatestr_with_today(future, t)? == "2/1/24");

        Ok(())
    }

    #[crate::ctb_test]
    fn datepattern_mm_mm_dd_dd_casing_and_quotes() -> Result<()> {
        let n = datevalue(2003, 4, 20)?;

        ensure!(datepattern(n, "mm/dd/yy")? == "4/20/03");
        ensure!(datepattern(n, "MM/DD/YY")? == "04/20/03");

        ensure!(datepattern(n, "dd-MON-yy")? == "20-APR-03");
        ensure!(
            datepattern(n, "DayOfWeek, Month ddnth, yyyy")?
                == "Sunday, April 20th, 2003"
        );
        ensure!(datepattern(n, "dayofweek")? == "sunday");
        ensure!(datepattern(n, "DOW")? == "0");

        let n2 = datevalue(2002, 5, 23)?;
        ensure!(
            datepattern(n2, "Quarter \"Quarter\" yyyy")?
                == "Second Quarter 2002"
        );

        let n3 = datevalue(2004, 7, 11)?;
        ensure!(datepattern(n3, "Qtr \"Qtr\" yyyy")? == "3rd Qtr 2004");

        Ok(())
    }

    #[crate::ctb_test]
    fn datepattern_quarter_and_nth_variants() -> Result<()> {
        let n = datevalue(2002, 3, 9)?; // Q1
        ensure!(datepattern(n, "qqqyy")? == "1q02");

        let n2 = datevalue(1867, 3, 1)?;
        ensure!(
            datepattern(n2, "mmnth \"month of\" yyyy")? == "3rd month of 1867"
        );

        let n3 = datevalue(2024, 1, 1)?;
        ensure!(datepattern(n3, "wwnth")? == "1st");

        Ok(())
    }

    #[crate::ctb_test]
    fn superdate_epoch_and_roundtrip() -> Result<()> {
        let d0 = datevalue(1904, 1, 1)?;
        let n0 = superdate(d0, 0)?;
        ensure!(n0 == 0);

        ensure!(ymd_from_date(regulardate(0)?)? == (1904, 1, 1));
        ensure!(regulartime(0)? == 0);

        let d1 = datevalue(1904, 1, 2)?;
        let n1 = superdate(d1, 0)?;
        ensure!(n1 == SUPERDATE_SECONDS_PER_DAY);

        Ok(())
    }

    #[crate::ctb_test]
    fn superdate_roundtrip_date_and_time() -> Result<()> {
        let d = datevalue(2003, 4, 20)?;
        let t = 9 * 3600 + 56 * 60 + 37;
        let n = superdate(d, t)?;

        ensure!(regulardate(n)? == d);
        ensure!(regulartime(n)? == t);
        Ok(())
    }

    #[crate::ctb_test]
    fn superdate_min_and_max_bounds() -> Result<()> {
        let min_d = datevalue(1904, 1, 1)?;
        ensure!(superdate(min_d, 0)? == 0);

        let max_d = datevalue(2040, 12, 31)?;
        ensure!(superdate(max_d, SUPERDATE_SECONDS_PER_DAY - 1).is_ok());

        ensure!(superdate(min_d, -1).is_err());
        ensure!(superdate(min_d, SUPERDATE_SECONDS_PER_DAY).is_err());

        ensure!(regulardate(-1).is_err());
        ensure!(regulartime(-1).is_err());

        Ok(())
    }

    #[crate::ctb_test]
    fn superdate_string_formatting_matches_examples() -> Result<()> {
        let d = datevalue(2003, 4, 20)?;
        let n = superdate(d, 9 * 3600 + 56 * 60 + 37)?;

        ensure!(superdatesecondsstr(n)? == "4/20/03 9:56:37 AM");
        ensure!(superdatestr(n)? == "4/20/03 9:56 AM");

        Ok(())
    }
}
