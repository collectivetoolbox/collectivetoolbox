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

//! Pan numeric string helpers.

use ctb_utilities::math::exact_float::u64_to_f64_exact;

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

/// Converts a `MacRoman` code point into a string.
pub fn chr(code: u8) -> String {
    ctb_formats_encoding::macroman::chr(code)
}

/// Returns the `MacRoman` code point for a single-character string.
pub fn asc(s: &str) -> Option<u8> {
    ctb_formats_encoding::macroman::asc(s)
}

/// Formats a byte count using decimal SI units.
pub fn bytepattern(bytes: i64) -> Result<String> {
    if bytes < 0 {
        bail!("bytepattern expects a non-negative byte count");
    }

    let bytes_u64 = u64::try_from(bytes).context("bytes out of range")?;

    let (value, unit) = if bytes_u64 < 1_000 {
        ((u64_to_f64_exact(bytes_u64)?), "bytes")
    } else if bytes_u64 < 1_000_000 {
        ((u64_to_f64_exact(bytes_u64)? / 1_000.0), "kB")
    } else if bytes_u64 < 1_000_000_000 {
        ((u64_to_f64_exact(bytes_u64)? / 1_000_000.0), "MB")
    } else if bytes_u64 < 1_000_000_000_000 {
        ((u64_to_f64_exact(bytes_u64)? / 1_000_000_000.0), "GB")
    } else {
        ((u64_to_f64_exact(bytes_u64)? / 1_000_000_000_000.0), "TB")
    };

    let rendered = if unit == "bytes" {
        format!("{bytes_u64} {unit}")
    } else {
        let s = fmt_trim_trailing_zeros(format!("{value:.1}"));
        format!("{s} {unit}")
    };

    Ok(rendered)
}

/// Formats an integer with grouping commas.
pub fn commastr(number: i64) -> String {
    let (sign, abs) = if number < 0 {
        ("-", number.saturating_abs().to_string())
    } else {
        ("", number.to_string())
    };

    format!("{sign}{}", commify_digits(&abs))
}

/// Formats a number as words like "One dollar and 05 cents".
pub fn dollarsandcents(number: f64) -> Result<String> {
    if !number.is_finite() {
        bail!("dollarsandcents expects a finite number");
    }

    // Round to the nearest cent.
    let cents_total = (number * 100.0).round();
    if cents_total.abs() > 9_223_372_036_854_775_000.0 {
        bail!("dollarsandcents number is out of supported range");
    }

    let cents_total_i64 =
        f64_to_i64(cents_total).context("cents out of range")?;

    let (neg, cents_total_u64) = if cents_total_i64 < 0 {
        (
            true,
            u64::try_from(cents_total_i64.saturating_abs())
                .context("abs out of range")?,
        )
    } else {
        (
            false,
            u64::try_from(cents_total_i64).context("cents out of range")?,
        )
    };

    let dollars = cents_total_u64 / 100;
    let cents = u8::try_from(cents_total_u64 % 100)
        .context("cents modulo out of range")?;

    let mut dollars_words = number_to_words(dollars)?;
    if dollars_words.is_empty() {
        dollars_words = "zero".to_owned();
    }
    let dollars_words = capitalize_first(&dollars_words);

    let dollar_word = if dollars == 1 { "dollar" } else { "dollars" };
    let cents_word = if cents == 1 { "cent" } else { "cents" };

    let prefix = if neg { "Minus " } else { "" };
    Ok(format!(
        "{prefix}{dollars_words} {dollar_word} and {cents:02} {cents_word}"
    ))
}

/// Formats a value using its `Display` implementation.
pub fn exportcell<T: std::fmt::Display>(value: T) -> String {
    value.to_string()
}

/// Parses a hexadecimal string into a number.
pub fn hex(text: &str) -> Option<u64> {
    let t = text.trim();
    if t.is_empty() {
        return None;
    }
    u64::from_str_radix(t, 16).ok()
}

/// Formats a byte as two uppercase hex digits.
pub fn hexbyte(number: u8) -> String {
    format!("{number:02X}")
}

/// Formats a 32-bit number as eight uppercase hex digits.
pub fn hexlong(number: u32) -> String {
    format!("{number:08X}")
}

/// Formats a number as an uppercase hex string, padded to 8 chars.
pub fn hexstr(number: u64) -> String {
    let s = format!("{number:X}");
    let min_width = 8usize;
    if s.len() >= min_width {
        return s;
    }
    let pad_len = min_width.saturating_sub(s.len());
    format!("{}{}", "0".repeat(pad_len), s)
}

/// Formats a 16-bit number as four uppercase hex digits.
pub fn hexword(number: u16) -> String {
    format!("{number:04X}")
}

/// Formats a number as dollars and cents with grouping commas.
pub fn money(number: f64) -> Result<String> {
    if !number.is_finite() {
        bail!("money expects a finite number");
    }

    let sign = if number < 0.0 { "-" } else { "" };
    let abs = number.abs();

    let rounded = (abs * 100.0).round() / 100.0;
    let whole = rounded.trunc();
    let frac = (rounded - whole).abs();

    let whole_i64 = f64_to_i64(whole).context("whole dollars out of range")?;
    let cents =
        f64_to_i64((frac * 100.0).round()).context("cents out of range")?;

    let cents_u8 = u8::try_from(cents).context("cents out of range")?;
    Ok(format!(
        "{sign}{}.{}",
        commastr(whole_i64),
        format!("{cents_u8:02}")
    ))
}

/// Formats an ordinal number (e.g. 1st, 2nd).
pub fn nth(number: i64) -> String {
    let abs = number.saturating_abs();
    let suffix = match abs % 100 {
        11..=13 => "th",
        _ => match abs % 10 {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        },
    };
    format!("{number}{suffix}")
}

/// Formats `number` with a fixed number of decimal places.
pub fn places(number: f64, places: usize) -> Result<String> {
    if !number.is_finite() {
        bail!("places expects a finite number");
    }

    let places_i32 = i32::try_from(places).context("places too large")?;
    let factor = 10_f64
        .powi(places_i32)
        // Clamp extremely large multipliers to avoid `inf`.
        .min(1e18);

    let sign = if number < 0.0 { "-" } else { "" };
    let abs = number.abs();

    let truncated = (abs * factor).trunc() / factor;
    let mut s = format!("{truncated:.places$}",);

    if places == 0 {
        // Ensure no trailing decimal point.
        if let Some((int, _)) = s.split_once('.') {
            s = int.to_owned();
        }
    }

    Ok(format!("{sign}{s}"))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Radix selection for base conversion.
pub enum RadixSpec {
    /// Explicit base: 2, 4, 8, 16, or 32.
    Base(u32),
    Binary,
    Octal,
    Hex,
}

impl RadixSpec {
    /// Returns the numeric base for this radix specification.
    pub fn base(self) -> u32 {
        match self {
            Self::Base(b) => b,
            Self::Binary => 2,
            Self::Octal => 8,
            Self::Hex => 16,
        }
    }
}

impl TryFrom<&str> for RadixSpec {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        let v = value.trim().to_ascii_lowercase();
        match v.as_str() {
            "binary" => Ok(Self::Binary),
            "octal" => Ok(Self::Octal),
            "hex" => Ok(Self::Hex),
            _ => {
                let base = u32::from_str_radix(&v, 10)
                    .context("invalid radix string")?;
                Ok(Self::Base(base))
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Output of `radix`, either a number or raw binary bytes.
pub enum RadixValue {
    Number(i128),
    RawBinary(Vec<u8>),
}

/// Parses `text` in the given radix.
///
/// Large binary/hex strings are returned as raw bytes.
pub fn radix(radix: RadixSpec, text: &str) -> Result<RadixValue> {
    let base = radix.base();
    if !matches!(base, 2 | 4 | 8 | 16 | 32) {
        bail!("unsupported radix base: {base}");
    }

    let t = text.trim();
    if t.is_empty() {
        bail!("radix expects non-empty text");
    }

    // Emit raw binary for large binary/hex inputs.
    if base == 16 && t.len() > 8 {
        let bytes = parse_hex_bytes(t)?;
        return Ok(RadixValue::RawBinary(bytes));
    }
    if base == 2 && t.len() > 32 {
        let bytes = parse_binary_bits_to_bytes(t)?;
        return Ok(RadixValue::RawBinary(bytes));
    }

    let value = parse_int_custom_base(t, base)?;
    Ok(RadixValue::Number(value))
}

/// Input for `radixstr`, either a number or raw bytes.
pub enum RadixStrInput<'a> {
    Number(i128),
    RawBinary(&'a [u8]),
}

/// Formats a number or raw bytes in the given radix.
pub fn radixstr(radix: RadixSpec, input: RadixStrInput<'_>) -> Result<String> {
    let base = radix.base();
    if !matches!(base, 2 | 4 | 8 | 16 | 32) {
        bail!("unsupported radix base: {base}");
    }

    match input {
        RadixStrInput::Number(n) => format_number_in_base(radix, n),
        RadixStrInput::RawBinary(bytes) => format_bytes_in_base(radix, bytes),
    }
}

/// Formats `number` using scientific notation with 3 decimals.
pub fn scientificnotation(number: f64) -> Result<String> {
    if !number.is_finite() {
        bail!("scientificnotation expects a finite number");
    }

    let s = format!("{number:.3e}");
    let Some((mantissa, exp)) = s.split_once('e') else {
        bail!("unexpected scientific format");
    };

    let exp_i32 =
        i32::from_str_radix(exp, 10).context("unexpected exponent")?;
    let sign = if exp_i32 < 0 { "-" } else { "+" };
    let abs_exp = exp_i32.saturating_abs();

    Ok(format!("{mantissa}e{sign}{abs_exp}"))
}

/// Formats `number` without trailing decimal zeros.
pub fn str_(number: f64) -> Result<String> {
    if !number.is_finite() {
        bail!("str expects a finite number");
    }
    Ok(fmt_trim_trailing_zeros(number.to_string()))
}

/// Parses leading decimal digits into an integer.
pub fn val(text: &str) -> Option<i64> {
    let t = text.trim_start();
    let mut digits = String::new();

    for ch in t.chars() {
        if ch.is_ascii_digit() {
            digits.push(ch);
        } else {
            break;
        }
    }

    if digits.is_empty() {
        return None;
    }

    i64::from_str_radix(&digits, 10).ok()
}

// --- helpers (private) ---

fn fmt_trim_trailing_zeros(mut s: String) -> String {
    if let Some((head, tail)) = s.split_once('.') {
        let tail_trim = tail.trim_end_matches('0').to_owned();
        if tail_trim.is_empty() {
            return head.to_owned();
        }
        s = format!("{head}.{tail_trim}");
    }
    s
}

#[expect(
    clippy::expect_used,
    reason = "Division by constant 3 is non-zero and cannot fail"
)]
fn commify_digits(digits: &str) -> String {
    let extra = digits.len().checked_div(3).expect("3 is non-zero");
    let mut out = String::with_capacity(digits.len().saturating_add(extra));
    let mut count = 0usize;

    for ch in digits.chars().rev() {
        if count == 3 {
            out.push(',');
            count = 0;
        }
        out.push(ch);
        count = count.saturating_add(1);
    }

    out.chars().rev().collect()
}

fn f64_to_i64(n: f64) -> Option<i64> {
    // Truncation is fine for callers that already rounded where needed.
    let truncated = n.trunc();
    let s = fmt_trim_trailing_zeros(truncated.to_string());
    i64::from_str_radix(&s, 10).ok()
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    let rest: String = chars.collect();
    format!("{}{}", first.to_ascii_uppercase(), rest)
}

fn number_to_words(n: u64) -> Result<String> {
    if n == 0 {
        return Ok("zero".to_owned());
    }

    const SCALES: [&str; 5] =
        ["", "thousand", "million", "billion", "trillion"];

    let mut parts: Vec<String> = Vec::new();
    let mut remaining = n;
    let mut scale_idx = 0usize;

    while remaining > 0 {
        let chunk = u16::try_from(remaining % 1000)
            .context("Number modulo 1000 must fit in u16")?;
        if chunk != 0 {
            let mut chunk_words = chunk_to_words(chunk);
            let scale = SCALES
                .get(scale_idx)
                .copied()
                .ok_or_else(|| anyhow::anyhow!("Scale index out of bounds"))?;
            if !scale.is_empty() {
                chunk_words.push(scale.to_owned());
            }
            parts.push(chunk_words.join(" "));
        }

        remaining /= 1000;
        scale_idx = scale_idx.saturating_add(1);
    }

    parts.reverse();
    Ok(parts.join(" "))
}

fn chunk_to_words(n: u16) -> Vec<String> {
    let hundreds = n / 100;
    let rem = n % 100;

    let mut out: Vec<String> = Vec::new();

    if hundreds != 0 {
        out.push(unit_word(hundreds).to_owned());
        out.push("hundred".to_owned());
    }

    if rem != 0 {
        if rem < 10 {
            out.push(unit_word(rem).to_owned());
        } else if rem < 20 {
            out.push(teen_word(rem).to_owned());
        } else {
            let tens = rem / 10;
            let ones = rem % 10;
            out.push(tens_word(tens).to_owned());
            if ones != 0 {
                out.push(unit_word(ones).to_owned());
            }
        }
    }

    out
}

fn unit_word(n: u16) -> &'static str {
    match n {
        0 => "zero",
        1 => "one",
        2 => "two",
        3 => "three",
        4 => "four",
        5 => "five",
        6 => "six",
        7 => "seven",
        8 => "eight",
        9 => "nine",
        _ => "zero",
    }
}

fn teen_word(n: u16) -> &'static str {
    match n {
        10 => "ten",
        11 => "eleven",
        12 => "twelve",
        13 => "thirteen",
        14 => "fourteen",
        15 => "fifteen",
        16 => "sixteen",
        17 => "seventeen",
        18 => "eighteen",
        19 => "nineteen",
        _ => "ten",
    }
}

fn tens_word(n: u16) -> &'static str {
    match n {
        2 => "twenty",
        3 => "thirty",
        4 => "forty",
        5 => "fifty",
        6 => "sixty",
        7 => "seventy",
        8 => "eighty",
        9 => "ninety",
        _ => "zero",
    }
}

fn parse_hex_bytes(text: &str) -> Result<Vec<u8>> {
    let t = text.trim();
    if !t.len().is_multiple_of(2) {
        bail!("hex raw binary input must have an even number of digits");
    }

    let mut out = Vec::with_capacity(t.len() / 2);
    let mut it = t.as_bytes().chunks_exact(2);

    for pair in &mut it {
        let s = std::str::from_utf8(pair).context("non-utf8 hex")?;
        let b = u8::from_str_radix(s, 16).context("invalid hex byte")?;
        out.push(b);
    }

    Ok(out)
}

fn parse_binary_bits_to_bytes(text: &str) -> Result<Vec<u8>> {
    let t = text.trim();
    if t.chars().any(|c| c != '0' && c != '1') {
        bail!("binary input contains non-bit characters");
    }

    // Left-pad to a whole number of bytes.
    #[expect(
        clippy::expect_used,
        reason = "Modulo by constant 8 is non-zero and cannot fail"
    )]
    let rem = t.len().checked_rem(8).expect("8 is non-zero");
    let pad = if rem == 0 {
        0
    } else {
        8_usize.saturating_sub(rem)
    };
    let padded = format!("{}{}", "0".repeat(pad), t);

    let mut out = Vec::with_capacity(padded.len() / 8);
    for chunk in padded.as_bytes().chunks_exact(8) {
        let s = std::str::from_utf8(chunk).context("non-utf8 bits")?;
        let b = u8::from_str_radix(s, 2).context("invalid byte bits")?;
        out.push(b);
    }

    Ok(out)
}

fn parse_int_custom_base(text: &str, base: u32) -> Result<i128> {
    let t = text.trim();
    let mut value: i128 = 0;

    for ch in t.chars() {
        let digit =
            digit_value(ch).with_context(|| format!("invalid digit '{ch}'"))?;
        let digit_u32 = u32::from(digit);
        if digit_u32 >= base {
            bail!("digit '{ch}' out of range for base {base}");
        }

        let base_i128 =
            i128::from_str_radix(&base.to_string(), 10).context("base conv")?;
        value = value
            .checked_mul(base_i128)
            .context("overflow in radix parsing")?;
        value = value
            .checked_add(i128::from(digit))
            .context("overflow in radix parsing")?;
    }

    Ok(value)
}

fn digit_value(ch: char) -> Result<u8> {
    if ch.is_ascii_digit() {
        let s = ch.to_string();
        let v = u8::from_str_radix(&s, 10).context("digit parse")?;
        return Ok(v);
    }

    let up = ch.to_ascii_uppercase();
    if up.is_ascii_uppercase() {
        let idx = u32::from(up)
            .checked_sub(u32::from('A'))
            .context("letter digit underflow")?;
        let v_u8 = u8::try_from(idx)
            .context("letter digit out of range")?
            .saturating_add(10);
        return Ok(v_u8);
    }

    bail!("unsupported digit character");
}

fn format_number_in_base(radix: RadixSpec, n: i128) -> Result<String> {
    let base = radix.base();
    if n < 0 {
        bail!("radixstr does not support negative numbers");
    }

    let mut value = n;
    let base_i128 =
        i128::from_str_radix(&base.to_string(), 10).context("base conv")?;

    if value == 0 {
        return Ok("0".to_owned());
    }

    let mut digits: Vec<char> = Vec::new();
    while value > 0 {
        let rem = (value
            .checked_rem(base_i128)
            .context("remainder by base failed")?)
            .try_into()
            .context("remainder out of range")?;
        digits.push(digit_char(rem)?);
        value = value
            .checked_div(base_i128)
            .context("division by base failed")?;
    }

    digits.reverse();
    let mut s: String = digits.into_iter().collect();

    // 32-bit padded binary.
    if base == 2 {
        let max_bits = 32usize;
        if s.len() < max_bits {
            let pad = max_bits.saturating_sub(s.len());
            s = format!("{}{}", "0".repeat(pad), s);
        }
    }

    Ok(s)
}

fn format_bytes_in_base(radix: RadixSpec, bytes: &[u8]) -> Result<String> {
    let base = radix.base();
    match base {
        16 => Ok(bytes.iter().map(|b| format!("{b:02X}")).collect()),
        2 => Ok(bytes.iter().map(|b| format!("{b:08b}")).collect()),
        8 | 4 | 32 => {
            // This interprets bytes as a big-endian integer and format. Hopefully this is right.
            let mut n: i128 = 0;
            for &b in bytes {
                let shift = 8i32;
                let n_next = n
                    .checked_mul(i128::from(1u16 << shift))
                    .context("overflow building integer from bytes")?;
                n = n_next
                    .checked_add(i128::from(b))
                    .context("overflow building integer from bytes")?;
            }
            format_number_in_base(radix, n)
        }
        _ => bail!("unsupported base for bytes"),
    }
}

fn digit_char(d: u8) -> Result<char> {
    match d {
        0..=9 => {
            let ch_code = u32::from(b'0').saturating_add(u32::from(d));
            std::char::from_u32(ch_code).ok_or_else(|| {
                anyhow::anyhow!("Invalid ASCII digit codepoint {ch_code}")
            })
        }
        10..=35 => {
            let idx = u32::from(d.saturating_sub(10));
            let ch_code = u32::from(b'A').saturating_add(idx);
            std::char::from_u32(ch_code).ok_or_else(|| {
                anyhow::anyhow!("Invalid ASCII letter codepoint {ch_code}")
            })
        }
        _ => bail!("Digit value {d} is outside supported base range 0..35"),
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

    #[crate::ctb_test]
    fn chr_asc_roundtrip_basic() {
        let s = chr(65);
        assert_eq!(asc(&s), Some(65));
    }

    #[crate::ctb_test]
    fn commastr_formats() {
        assert_eq!(commastr(1234567), "1,234,567");
        assert_eq!(commastr(-987654), "-987,654");
    }

    #[crate::ctb_test]
    fn hex_helpers() {
        assert_eq!(hex("0C2"), Some(194));
        assert_eq!(hexbyte(68), "44");
        assert_eq!(hexword(68), "0044");
        assert_eq!(hexlong(68), "00000044");
        assert_eq!(hexstr(68), "00000044");
    }

    #[crate::ctb_test]
    fn val_parses_leading_digits() {
        assert_eq!(val("123 S. Main St."), Some(123));
        assert_eq!(val(" 123 S. Main St."), Some(123));
        assert_eq!(val("No digits"), None);
    }

    #[crate::ctb_test]
    fn nth_ordinals() {
        assert_eq!(nth(1), "1st");
        assert_eq!(nth(2), "2nd");
        assert_eq!(nth(3), "3rd");
        assert_eq!(nth(4), "4th");
        assert_eq!(nth(11), "11th");
        assert_eq!(nth(12), "12th");
        assert_eq!(nth(13), "13th");
        assert_eq!(nth(21), "21st");
    }

    #[crate::ctb_test]
    fn places_truncates() -> Result<()> {
        assert_eq!(places(1.239, 2)?, "1.23");
        assert_eq!(places(-1.2, 3)?, "-1.200");
        assert_eq!(places(9.999, 0)?, "9");
        Ok(())
    }

    #[crate::ctb_test]
    fn scientificnotation_formats() -> Result<()> {
        assert_eq!(scientificnotation(98120.0)?, "9.812e+4");
        Ok(())
    }

    #[crate::ctb_test]
    fn money_formats() -> Result<()> {
        assert_eq!(money(98123.45)?, "98,123.45");
        assert_eq!(money(-12.3)?, "-12.30");
        Ok(())
    }

    #[crate::ctb_test]
    fn dollarsandcents_words() -> Result<()> {
        assert_eq!(
            dollarsandcents(98123.45)?,
            "Ninety eight thousand one hundred twenty three dollars and 45 cents"
        );
        assert_eq!(
            dollarsandcents(98123.05)?,
            "Ninety eight thousand one hundred twenty three dollars and 05 cents"
        );
        Ok(())
    }

    #[crate::ctb_test]
    fn radix_number_and_raw_binary() -> Result<()> {
        assert_eq!(radix(RadixSpec::Hex, "FF")?, RadixValue::Number(255));

        // >8 hex digits -> raw binary bytes.
        assert_eq!(
            radix(RadixSpec::Hex, "0102030405060708")?,
            RadixValue::RawBinary(vec![1, 2, 3, 4, 5, 6, 7, 8])
        );

        assert_eq!(
            radixstr(
                RadixSpec::Hex,
                RadixStrInput::RawBinary(&[1, 2, 3, 4, 5, 6, 7, 8])
            )?,
            "0102030405060708"
        );

        // >32 bits binary -> raw binary bytes.
        let v = radix(RadixSpec::Binary, "1".repeat(40).as_str())?;
        let RadixValue::RawBinary(bytes) = v else {
            bail!("expected raw binary from long binary input");
        };
        assert_eq!(bytes.len(), 5);

        Ok(())
    }
}
