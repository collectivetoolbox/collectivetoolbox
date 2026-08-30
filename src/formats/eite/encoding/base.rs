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

//! Base conversion

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::{Result, anyhow, bail, ensure};
use malachite::Natural;
use malachite::base::num::basic::traits::Zero;

use crate::encoding::ascii::{byte_from_stagel_char, stagel_char_from_byte};
use ctb_formats_utilities::FormatLog;

#[derive(Clone)]
pub struct BaseConversionPaddingMode {
    /// If true, pad to the left of each number to at least this many digits.
    pub pad_l: u32,
    /// If true, pad to fit the limit. Requires a limit to be set.
    pub pad_fit: bool,
}

impl Default for BaseConversionPaddingMode {
    fn default() -> Self {
        Self {
            pad_l: 1,
            pad_fit: false,
        }
    }
}

impl BaseConversionPaddingMode {
    pub fn none() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BaseAlphabet {
    /// Standard alphanumeric alphabet (0-9, A-Z/a-z), case-insensitive, base 2..=36.
    #[default]
    Standard,
    /// Standard RFC 4648 Base64 alphabet (A-Z = 0..25, a-z = 26..51, 0-9 = 52..61, + = 62, / = 63),
    /// case-sensitive, base 2..=64.
    Base64Standard,
}

impl BaseAlphabet {
    /// Returns the maximum base supported by this alphabet.
    pub fn max_base(self) -> u8 {
        match self {
            Self::Standard => 36,
            Self::Base64Standard => 64,
        }
    }

    /// Whether digits in this alphabet are case-sensitive.
    pub fn is_case_sensitive(self) -> bool {
        match self {
            Self::Standard => false,
            Self::Base64Standard => true,
        }
    }

    /// Returns the zero digit character for this alphabet.
    pub fn zero_char(self) -> char {
        match self {
            Self::Standard => '0',
            Self::Base64Standard => 'A',
        }
    }

    /// Returns the character for the given digit value.
    pub fn char_for_digit(self, d: u8) -> Result<char> {
        match self {
            Self::Standard => {
                if !(0..=35).contains(&d) {
                    bail!("{d} is not within range 0..=35 for standard alphabet");
                }
                if d <= 9 {
                    char::from_u32(u32::from(d).saturating_add(48))
                        .ok_or_else(|| anyhow!("Invalid ASCII char"))
                } else {
                    char::from_u32(u32::from(d).saturating_add(55))
                        .ok_or_else(|| anyhow!("Invalid ASCII char"))
                }
            }
            Self::Base64Standard => {
                if !(0..=63).contains(&d) {
                    bail!("{d} is not within range 0..=63 for Base64Standard alphabet");
                }
                match d {
                    0..=25 => char::from_u32(u32::from(d).saturating_add(65))
                        .ok_or_else(|| anyhow!("Invalid ASCII char")),
                    26..=51 => char::from_u32(
                        u32::from(d).saturating_sub(26).saturating_add(97),
                    )
                    .ok_or_else(|| anyhow!("Invalid ASCII char")),
                    52..=61 => char::from_u32(
                        u32::from(d).saturating_sub(52).saturating_add(48),
                    )
                    .ok_or_else(|| anyhow!("Invalid ASCII char")),
                    62 => Ok('+'),
                    63 => Ok('/'),
                    _ => bail!("Digit out of range"),
                }
            }
        }
    }

    /// Returns the digit value for the given character.
    pub fn digit_for_char(self, ch: char) -> Result<u8> {
        match self {
            Self::Standard => {
                let ch_uc = ch.to_ascii_uppercase();
                let b = u8::try_from(ch_uc).map_err(|_| anyhow!("'{ch}' is not ASCII"))?;
                if (48..=57).contains(&b) {
                    Ok(b.saturating_sub(48))
                } else if (65..=90).contains(&b) {
                    Ok(b.saturating_sub(55))
                } else {
                    bail!("'{ch}' is not within range 0..=9, A..=Z for standard alphabet");
                }
            }
            Self::Base64Standard => {
                let b = u8::try_from(ch).map_err(|_| anyhow!("'{ch}' is not ASCII"))?;
                match b {
                    65..=90 => Ok(b.saturating_sub(65)), // 'A'..='Z' -> 0..25
                    97..=122 => Ok(b.saturating_sub(97).saturating_add(26)), // 'a'..='z' -> 26..51
                    48..=57 => Ok(b.saturating_sub(48).saturating_add(52)), // '0'..='9' -> 52..61
                    43 => Ok(62), // '+' -> 62
                    47 => Ok(63), // '/' -> 63
                    _ => bail!("'{ch}' is not a valid digit in Base64Standard alphabet"),
                }
            }
        }
    }

    /// Returns true if the character represents a valid digit in this base.
    pub fn is_digit(self, ch: char, base: u8) -> bool {
        if !self.is_supported_base(base) {
            return false;
        }
        match self.digit_for_char(ch) {
            Ok(d) => d < base,
            Err(_) => false,
        }
    }

    /// Returns true if the given base is valid for this alphabet.
    pub fn is_supported_base(self, base: u8) -> bool {
        (1..=self.max_base()).contains(&base)
    }
}

#[derive(Clone)]
#[expect(clippy::struct_excessive_bools, reason = "conversion format flags")]
pub struct BaseStringFormatSettings {
    /// The prefix to use for each number (the quintessential example being 0x for hexadecimal)
    pub prefix: String,
    /// The separator to use between numbers.
    pub separator: String,
    /// Should the string be lowercased?
    pub lowercase: bool,
    /// Should runs of characters (other than space) not in the base be replaced
    /// with the configured separator?
    pub filter_chars: bool,
    /// Should filtered characters be totally ignored for parsing numbers? E.g.
    /// `10_000` would get the _ filtered out and be treated as 10000.
    pub collapse_filtered: bool,
    /// A list of filtered characters to collapse, leaving others as spaces.
    pub collapse_only: Vec<String>,
    /// Determines whether to treat prefixes like 0x as part of the number while
    /// parsing. If `false`, the existing prefix will be treated as a number 0
    /// followed by a string.
    pub parse_prefixes: bool,
    /// Limit the number of digits for each number to be able to hold at least
    /// this value. Set to 0 for no limiting. This requires a limit instead of a
    /// number of digits because limiting to 2 for hex input of bytes, for
    /// instance, and converting to decimal, would result in at least *three*
    /// digits per output byte.
    pub limit: u64,
    /// Zero-pad the left of each number to at least this many digits.
    pub pad: BaseConversionPaddingMode,
    /// Alphabet to use for parsing input numbers.
    pub input_alphabet: BaseAlphabet,
    /// Alphabet to use for formatting output numbers.
    pub output_alphabet: BaseAlphabet,
}

impl Default for BaseStringFormatSettings {
    fn default() -> Self {
        Self {
            prefix: String::new(),
            separator: " ".into(),
            lowercase: false,
            limit: 0,
            filter_chars: true,
            collapse_only: Vec::new(),
            collapse_filtered: false,
            parse_prefixes: true,
            pad: BaseConversionPaddingMode {
                pad_l: 1,
                pad_fit: false,
            },
            input_alphabet: BaseAlphabet::Standard,
            output_alphabet: BaseAlphabet::Standard,
        }
    }
}

/// Returns the nth digit in base 36 or less (using capitalized digits).
/// The original JS version had a bug where it would accept 36 as a base, when 0
/// to 35 is expected (36 digits).
pub fn int_to_base36_char(n: u8) -> Result<String> {
    if !(0..=35).contains(&n) {
        bail!("{n} is not within range 0..=35");
    }
    if n <= 9 {
        stagel_char_from_byte(n.saturating_add(48))
    } else {
        stagel_char_from_byte(n.saturating_add(55))
    }
}

/// Returns an int given the nth digit in base 36 or less (using capitalized digits).
pub fn int_from_base36_char(ch: &str) -> Result<u8> {
    // Validate input: must be a single character string
    if ch.len() != 1 {
        bail!("'{ch}' is not a single character");
    }

    // Convert to uppercase
    let ch_uc = ch.to_ascii_uppercase();
    let b = byte_from_stagel_char(&ch_uc)?;

    let int_res = if b >= 65 {
        if b > 90 {
            bail!(
                "'{ch_uc}' is not within the supported range of digits between 0 and Z (35)."
            );
        }
        b.saturating_sub(55)
    } else {
        if !(48..=57).contains(&b) {
            bail!(
                "'{ch}' is not within the supported range of digits between 0 and Z (35)."
            );
        }
        b.saturating_sub(48)
    };

    if !(0..=35).contains(&int_res) {
        bail!("Internal error in int_from_base36_char called with n='{ch}'.");
    }

    Ok(int_res)
}

/// Returns the integer represented by n in the requested base.
pub fn int_from_base_str_u32(s: &str, base: u8) -> Result<u32> {
    if !is_supported_base(base) {
        bail!("Unsupported base {base}");
    }
    let mut acc: u64 = 0;
    for ch in s.chars() {
        let d = int_from_base36_char(&ch.to_string())?;
        if d >= base {
            bail!("Digit {d} >= base {base}");
        }
        acc = acc
            .checked_mul(u64::from(base))
            .ok_or_else(|| anyhow!("Overflow converting {s} base {base}"))?;
        acc = acc
            .checked_add(u64::from(d))
            .ok_or_else(|| anyhow!("Overflow converting {s} base {base}"))?;
        if acc > u64::from(u32::MAX) {
            bail!("Overflow converting {s} base {base}");
        }
    }
    u32::try_from(acc).context("Did not fit in u32")
}

/// Returns the integer represented by n in the requested base.
pub fn int_from_base_str_u128(s: &str, base: u8) -> Result<u128> {
    if !is_supported_base(base) {
        bail!("Unsupported base {base}");
    }
    let mut acc: u128 = 0;
    for ch in s.chars() {
        let d = int_from_base36_char(&ch.to_string())?;
        if d >= base {
            bail!("Digit {d} >= base {base}");
        }
        acc = acc
            .checked_mul(u128::from(base))
            .ok_or_else(|| anyhow!("Overflow converting {s} base {base}"))?;
        acc = acc
            .checked_add(u128::from(d))
            .ok_or_else(|| anyhow!("Overflow converting {s} base {base}"))?;
    }
    Ok(acc)
}

/// Returns the integer represented by n in the requested base using the given alphabet.
#[expect(
    clippy::arithmetic_side_effects,
    reason = "Natural is arbitrary-precision and cannot overflow"
)]
pub fn int_from_base_str_big_alphabet(
    s: &str,
    base: u8,
    alphabet: BaseAlphabet,
) -> Result<Natural> {
    ensure!(
        alphabet.is_supported_base(base),
        "Unsupported base {base} for alphabet {alphabet:?}"
    );
    let mut acc = Natural::ZERO;
    let base_nat = Natural::from(base);
    for ch in s.chars() {
        let d = alphabet.digit_for_char(ch)?;
        if d >= base {
            bail!("Digit '{ch}' (value {d}) >= base {base}");
        }
        acc = acc * &base_nat + Natural::from(d);
    }
    Ok(acc)
}

/// Returns the string representation of an arbitrary-precision integer in the requested base and alphabet.
#[expect(
    clippy::arithmetic_side_effects,
    reason = "Natural is arbitrary-precision and cannot overflow"
)]
pub fn int_to_base_str_big_alphabet(
    mut n: Natural,
    base: u8,
    alphabet: BaseAlphabet,
) -> Result<String> {
    ensure!(
        alphabet.is_supported_base(base),
        "Unsupported base {base} for alphabet {alphabet:?}"
    );
    if n == 0 {
        return Ok(alphabet.char_for_digit(0)?.to_string());
    }
    let base_nat = Natural::from(base);
    let mut digits = Vec::new();
    while n > 0 {
        let rem_nat = &n % &base_nat;
        let rem = u8::try_from(&rem_nat)
            .map_err(|e| anyhow!("Digit out of range: {e:?}"))?;
        digits.push(alphabet.char_for_digit(rem)?);
        n /= &base_nat;
    }
    digits.reverse();
    Ok(digits.into_iter().collect())
}

/// Returns the integer represented by n in the requested base.
pub fn int_from_base_str_big(s: &str, base: u8) -> Result<Natural> {
    int_from_base_str_big_alphabet(s, base, BaseAlphabet::Standard)
}

/// Returns the integer represented by n in the requested base.
pub fn int_to_base_str(mut n: u32, base: u8) -> Result<String> {
    if !is_supported_base(base) {
        bail!("Unsupported base {base}");
    }
    if n == 0 {
        return Ok("0".into());
    }
    let mut out = String::new();
    while n > 0 {
        let digit = n
            .checked_rem(u32::from(base))
            .ok_or_else(|| anyhow!("Base cannot be zero"))?;
        let c_str = int_to_base36_char(digit.try_into()?)?;
        let ch = c_str
            .chars()
            .next()
            .ok_or_else(|| anyhow!("int_to_base36_char returned empty string"))?;
        out.push(ch);
        n = n
            .checked_div(u32::from(base))
            .ok_or_else(|| anyhow!("Base cannot be zero"))?;
    }
    Ok(out.chars().rev().collect())
}

pub fn hex_to_dec_single(s: &str) -> Result<u32> {
    int_from_base_str_u32(s, 16)
}

pub fn dec_to_hex_single(n: u32) -> Result<String> {
    int_to_base_str(n, 16)
}

pub fn hex_to_dec_string(s: &str) -> Result<(String, FormatLog)> {
    base_to_base_string(s, 16, 10, &BaseStringFormatSettings::default())
}

pub fn dec_to_hex_string(s: &str) -> Result<(String, FormatLog)> {
    base_to_base_string(s, 10, 16, &BaseStringFormatSettings::default())
}

#[expect(
    clippy::arithmetic_side_effects,
    reason = "Natural is arbitrary-precision and cannot overflow"
)]
pub fn get_digits_needed(
    n: Natural,
    base: u8,
    alphabet: BaseAlphabet,
) -> Result<Natural> {
    ensure!(alphabet.is_supported_base(base), "Unsupported base {base}");
    let mut digits = Natural::ZERO;
    let mut value = n;
    while value > 0 {
        value /= Natural::from(base);
        digits += Natural::from(1u8);
    }
    Ok(digits)
}

pub fn casefold_base_chars_in_string(
    s: &str,
    base: u8,
    uppercase: bool,
) -> Result<String> {
    ensure!(is_supported_base(base), "Unsupported base {base}");
    let mut result = String::new();
    for c in s.chars() {
        if is_base_digit(c.to_string().as_str(), base)? {
            result.push(if uppercase {
                c.to_ascii_uppercase()
            } else {
                c.to_ascii_lowercase()
            });
        } else {
            result.push(c);
        }
    }
    Ok(result)
}

/// Parse a string contaning numbers in base 2 through 36, and print it
/// formatted. Will warn for extra characters other than spaces and commas.
pub fn format_base_string(
    s: &str,
    base: u8,
    settings: &BaseStringFormatSettings,
) -> Result<(String, FormatLog)> {
    let parsed = _parse_base_string(
        s,
        base,
        base,
        settings.parse_prefixes,
        settings.filter_chars,
        settings.limit,
        settings.collapse_filtered,
        &settings.collapse_only,
        settings.input_alphabet,
        settings.output_alphabet,
    )?;
    let (out, mut log) = parsed;

    let formatted = _format_base_string(out, base, settings, settings.output_alphabet)?;
    log.merge(&formatted.1);

    Ok((formatted.0, log))
}

/// Parse a string contaning numbers in base 2 through 64, convert it to the
/// target base, and print it formatted. Will warn for extra characters other
/// than spaces and commas.
pub fn base_to_base_string(
    s: &str,
    from_base: u8,
    to_base: u8,
    format_settings: &BaseStringFormatSettings,
) -> Result<(String, FormatLog)> {
    let converted = _parse_base_string(
        s,
        from_base,
        to_base,
        format_settings.parse_prefixes,
        format_settings.filter_chars,
        format_settings.limit,
        format_settings.collapse_filtered,
        &format_settings.collapse_only,
        format_settings.input_alphabet,
        format_settings.output_alphabet,
    )?;

    let (res, mut log) = converted;

    let (formatted_res, formatted_log) =
        _format_base_string(res, to_base, format_settings, format_settings.output_alphabet)?;
    log.merge(&formatted_log);

    Ok((formatted_res, log))
}

/// Converts all characters that match the requested base into the target base.
/// It will leave other characters alone, so you can convert a list of numbers.
/// It allows hex input numbers like 0x1A.
fn _parse_base_string(
    s: &str,
    from_base: u8,
    to_base: u8,
    parse_prefixes: bool,
    filter_chars: bool,
    limit: u64,
    collapse_filtered: bool,
    collapse_only: &Vec<String>,
    from_alphabet: BaseAlphabet,
    to_alphabet: BaseAlphabet,
) -> Result<(Vec<String>, FormatLog)> {
    ensure!(
        from_alphabet.is_supported_base(from_base),
        "Unsupported from_base {from_base} for alphabet {from_alphabet:?}"
    );
    ensure!(
        to_alphabet.is_supported_base(to_base),
        "Unsupported to_base {to_base} for alphabet {to_alphabet:?}"
    );
    let mut log = FormatLog::default();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    let mut in_num = false;
    let mut num_chars: String = String::new();
    let mut out: Vec<String> = Vec::new();
    let max_digits = get_digits_needed(Natural::from(limit), from_base, from_alphabet)?;
    let max_digits: usize = usize::try_from(&max_digits).map_err(|e| {
        anyhow!(
            "Base conversion length of digits greater than usize, limited by String.len(): {e:?}"
        )
    })?;
    let max_digits = if limit == 0 { 0 } else { max_digits };

    let multiple_of_base = limit
        .checked_add(1)
        .is_some_and(|val| val.is_multiple_of(u64::from(from_base)));
    if (limit > 1) && !multiple_of_base {
        log.warn(format!("The limit was derived from the number of digits required to represent {limit}, but {limit} + 1 is not a multiple of the input base {from_base}. That's not necessarily wrong, but note that the limit is not directly the maximum number of digits, but the maximum value representable in the number of digits to limit to.").as_str());
    }

    let base_prefix_char: Option<char> = if parse_prefixes {
        match from_base {
            2 => Some('b'),
            8 => Some('o'),
            16 => Some('x'),
            _ => None,
        }
    } else {
        None
    };

    let base_prefix: Option<String> = base_prefix_char.map(|c| format!("0{c}"));

    let finalize_num = |num_chars: &mut String,
                        out: &mut Vec<String>|
     -> Result<()> {
        if let Some(base_prefix) = &base_prefix
            && num_chars.starts_with(base_prefix)
        {
            *num_chars = num_chars.trim_start_matches(base_prefix).to_string();
        }
        let nat = int_from_base_str_big_alphabet(num_chars, from_base, from_alphabet)?;
        let formatted = int_to_base_str_big_alphabet(nat, to_base, to_alphabet)?;
        out.push(formatted);
        Ok(())
    };
    let normalize_or_push_char = |out: &mut Vec<String>, c: char| {
        if !filter_chars {
            out.push(c.to_string());
        }
    };
    while i < chars.len() {
        let c: char = chars
            .get(i)
            .copied()
            .ok_or_else(|| anyhow!("Index out of bounds"))?;

        let this_is_base_digit = from_alphabet.is_digit(c, from_base);

        if let Some(base_prefix_char) = base_prefix_char {
            let potential_prefix = if let Some(potential_prefix) =
                i.checked_add(2).and_then(|end| chars.get(i..end))
            {
                potential_prefix
                    .first()
                    .copied()
                    .zip(potential_prefix.get(1).copied())
            } else {
                None
            };

            let next = i.checked_add(2).and_then(|idx| chars.get(idx));
            let next_is_base_digit = if let Some(next) = next {
                from_alphabet.is_digit(*next, from_base)
            } else {
                false
            };
            if let Some(potential_prefix) = potential_prefix
                && potential_prefix.0 == '0'
                && potential_prefix.1 == base_prefix_char
                && next_is_base_digit
            {
                if in_num {
                    finalize_num(&mut num_chars, &mut out)?;
                    in_num = false;
                    num_chars.clear();
                }

                i = i.saturating_add(2);
                continue;
            }
        }

        if this_is_base_digit {
            in_num = true;

            num_chars.push(c);
        } else {
            let mut this_collapse_filtered = false;
            let mut in_collapse_only = false;
            if c != ' ' && c != ',' {
                // Potentially filtered character
                in_collapse_only =
                    collapse_only.iter().any(|s| s == &c.to_string());
                this_collapse_filtered = collapse_filtered;
                if !in_collapse_only {
                    // Assume that if the character is being explicitly
                    // collapsed, it's not worth warning about.
                    log.import_warning(
                        i.try_into()?,
                        &format!(
                            "Unexpected character '{c}' in base {from_base}"
                        ),
                    );
                }
            }
            if !this_collapse_filtered && !in_collapse_only {
                if in_num {
                    finalize_num(&mut num_chars, &mut out)?;
                    in_num = false;
                    num_chars.clear();
                }

                normalize_or_push_char(&mut out, c);
            }
        }

        if in_num && (max_digits > 0) && (num_chars.len() == max_digits) {
            finalize_num(&mut num_chars, &mut out)?;
            in_num = false;
            num_chars.clear();
        }

        i = i.saturating_add(1);
    }

    if in_num && !num_chars.is_empty() {
        finalize_num(&mut num_chars, &mut out)?;
    }

    Ok((out, log))
}

pub fn _format_base_string(
    tokens: Vec<String>,
    base: u8,
    settings: &BaseStringFormatSettings,
    to_alphabet: BaseAlphabet,
) -> Result<(String, FormatLog)> {
    let mut log: FormatLog = FormatLog::default();

    let pad = &settings.pad;
    let limit = settings.limit;
    let num_prefix = &settings.prefix;

    let padded_width: u32 = if pad.pad_fit {
        let max_digits = get_digits_needed(Natural::from(limit), base, to_alphabet)?;
        u32::try_from(&max_digits)
            .map_err(|e| anyhow!("Padding to more than 32 bits of digits is not supported just because it seems unnecessary, but could be increased: {e:?}"))?
    } else {
        pad.pad_l
    };
    if (pad.pad_fit) && (limit == 0) {
        log.import_error(
            0,
            "Padding to fit limit was requested, but no limit was set.",
        );
        bail!("Incompatible padding and limit settings");
    }
    if (pad.pad_fit) && (limit == 1) {
        log.import_warning(0, "Padding to fit limit was requested, but limit was set to 1. 0 is always shown as 0 anyway, so the padding option will do nothing.");
    }
    if (pad.pad_fit) && (pad.pad_l > 1) {
        // Some cases of this don't technically need to be a fatal error, and it
        // could conceivably be useful to allow in some cases, for instance when
        // programmatically building CLI argument strings, but it is redundant,
        // and it's simplest to just require one or the other.
        log.import_error(0, "Padding to fit limit was requested, but a separate padding width was also requested. Please set one or the other.");
        bail!("Multiple padding configurations given");
    }

    let padded_width = std::cmp::max(pad.pad_l, padded_width);

    let mut out: String = String::new();
    for (index, token) in tokens.iter().enumerate() {
        let formatted = if is_base_str_alphabet(token, base, to_alphabet) {
            let separator = if index < tokens.len().saturating_sub(1) {
                &settings.separator
            } else {
                ""
            };
            let pad_len = usize::try_from(padded_width)?.saturating_sub(token.chars().count());
            let padding = to_alphabet.zero_char().to_string().repeat(pad_len);
            format!("{num_prefix}{padding}{token}{separator}")
        } else {
            token.clone()
        };
        out.push_str(&formatted);
    }

    Ok((
        if to_alphabet.is_case_sensitive() {
            out
        } else if settings.lowercase {
            casefold_base_chars_in_string(&out, base, false)?
        } else {
            casefold_base_chars_in_string(&out, base, true)?
        },
        log,
    ))
}

pub fn is_supported_base(base: u8) -> bool {
    (1..=36).contains(&base)
}

pub fn is_base_digit_alphabet(ch: char, base: u8, alphabet: BaseAlphabet) -> bool {
    alphabet.is_digit(ch, base)
}

pub fn is_base_str_alphabet(s: &str, base: u8, alphabet: BaseAlphabet) -> bool {
    if !alphabet.is_supported_base(base) {
        return false;
    }
    for ch in s.chars() {
        if !alphabet.is_digit(ch, base) {
            return false;
        }
    }
    true
}

pub fn is_base_digit(ch: &str, base: u8) -> Result<bool> {
    if ch.chars().count() != 1 {
        bail!("Invalid digit");
    }
    if !is_supported_base(base) {
        bail!("Unsupported base {base}");
    }
    let Some(c) = ch.chars().next() else {
        bail!("Empty character string");
    };
    Ok(BaseAlphabet::Standard.is_digit(c, base))
}

pub fn is_base_str(s: &str, base: u8) -> Result<bool> {
    if !is_supported_base(base) {
        bail!("Unsupported base {base}");
    }
    Ok(is_base_str_alphabet(s, base, BaseAlphabet::Standard))
}

/// Convert two hex digits to a single byte -> char (StageL: charFromHexByte)
/// StageL operated on bytes, not Unicode scalar validation beyond 0xFF.
pub fn char_from_hex_byte(hex: &str) -> Result<char> {
    if hex.len() != 2 {
        return Err(anyhow!("Expected 2 hex digits, got {}", hex.len()));
    }
    let v = int_from_base_str_u32(hex, 16)?;
    if v > 0xFF {
        return Err(anyhow!("Hex byte out of range"));
    }
    char::from_u32(v)
        .ok_or_else(|| anyhow!("Invalid Unicode scalar value: {v}"))
}

/*
Maybe useful:

fn u32_slice_as_bytes_le(values: &[u32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(values.len() * 4);
    for &v in values {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

fn u32_slice_as_bytes_be(values: &[u32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(values.len() * 4);
    for &v in values {
        out.extend_from_slice(&v.to_be_bytes());
    }
    out
}
 */

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
    use ctb_formats_utilities::{
        assert_string_ok_eq_no_errors, assert_string_ok_eq_no_warnings,
    };

    use super::*;

    #[crate::ctb_test]
    fn test_base36_digit_roundtrip() {
        for n in 0..=35 {
            let ch = int_to_base36_char(n).unwrap();
            let v = int_from_base36_char(&ch).unwrap();
            assert_eq!(n, v);
        }
        assert!(int_to_base36_char(36).is_err());
    }

    #[crate::ctb_test]
    fn test_hex_conversion_examples() {
        // Mirrors runTestsMath base conversion portion
        let hex = dec_to_hex_single(9917).unwrap();
        assert_eq!(hex, "26BD");
        let dec = hex_to_dec_single("26BD").unwrap();
        assert_eq!(dec, 9917);
    }

    #[crate::ctb_test]
    fn test_char_from_hex_byte() {
        assert_eq!(char_from_hex_byte("41").unwrap(), 'A');
        assert_eq!(char_from_hex_byte("7f").unwrap(), '\u{007F}');
        assert!(char_from_hex_byte("XYZ").is_err());
    }

    #[crate::ctb_test]
    fn test_base_to_base_string() {
        let format_settings = BaseStringFormatSettings::default();
        assert_string_ok_eq_no_warnings(
            "26",
            base_to_base_string("1A", 16, 10, &format_settings),
        );

        let (_result, _log) = assert_string_ok_eq_no_warnings(
            "26 16 4",
            base_to_base_string("0x1A, 0x10, 0x04", 16, 10, &format_settings),
        );

        let (_result, log) = assert_string_ok_eq_no_errors(
            // This result doesn't make mathematical sense, as the outputs are
            // base 10.
            "0x26, 0x16, 0x4",
            base_to_base_string(
                "0x1A, 0x10, 0x04",
                16,
                10,
                &BaseStringFormatSettings {
                    separator: "".to_string(),
                    filter_chars: false,
                    parse_prefixes: false,
                    ..Default::default()
                },
            ),
        );
        assert!(log.has_warnings());

        assert_string_ok_eq_no_warnings(
            "12",
            base_to_base_string("10", 10, 8, &format_settings),
        );

        let (_result, log) = assert_string_ok_eq_no_errors(
            "26,uuuu 4F,é 16, 4",
            base_to_base_string(
                "26,uuuu 4F,é 16, 0x04",
                16,
                16,
                &BaseStringFormatSettings {
                    separator: "".to_string(),
                    filter_chars: false,
                    ..Default::default()
                },
            ),
        );
        assert!(log.has_warnings());

        assert_string_ok_eq_no_warnings(
            "0x026!0x04f!0x016!0x004",
            base_to_base_string(
                "26, 4F, 16, 0x04",
                16,
                16,
                &BaseStringFormatSettings {
                    prefix: "0x".to_string(),
                    separator: "!".to_string(),
                    lowercase: true,
                    pad: BaseConversionPaddingMode {
                        pad_l: 3,
                        pad_fit: false,
                    },
                    ..Default::default()
                },
            ),
        );

        assert_string_ok_eq_no_warnings(
            "0x26!0x4F!0x16!0x04",
            base_to_base_string(
                "26, 4F, 16, 0x04",
                16,
                16,
                &BaseStringFormatSettings {
                    prefix: "0x".to_string(),
                    separator: "!".to_string(),
                    lowercase: false,
                    limit: u64::from(u8::MAX),
                    pad: BaseConversionPaddingMode {
                        pad_l: 0,
                        pad_fit: true,
                    },
                    ..Default::default()
                },
            ),
        );

        let (_result, log) = assert_string_ok_eq_no_errors(
            "26 4 16",
            format_base_string(
                "26, 4F, 16",
                10,
                &BaseStringFormatSettings::default(),
            ),
        );
        assert!(log.has_warnings());

        assert_string_ok_eq_no_warnings(
            "26, 4F, 16, F, 0",
            format_base_string(
                "0x26, 4f, 16f, 0",
                16,
                &BaseStringFormatSettings {
                    separator: ", ".to_string(),
                    limit: 255,
                    ..Default::default()
                },
            ),
        );

        assert_string_ok_eq_no_warnings(
            "2 6 4 F 1 6",
            format_base_string(
                "0x26, 4f, 16",
                16,
                &BaseStringFormatSettings {
                    limit: 1,
                    ..Default::default()
                },
            ),
        );
    }

    #[crate::ctb_test]
    fn test_format_base_string() {
        let (_result, log) = assert_string_ok_eq_no_errors(
            "26 0 4F 0 16F",
            format_base_string(
                "26, 0n4F, 0x16fZz",
                16,
                &BaseStringFormatSettings {
                    parse_prefixes: false,
                    ..Default::default()
                },
            ),
        );
        assert!(log.has_warnings());

        let (_result, log) = assert_string_ok_eq_no_errors(
            "0x26!, 0x0!n0x4f!, 0x0!x0x16f!Zz",
            format_base_string(
                "26, 0n4F, 0x16fZz",
                16,
                &BaseStringFormatSettings {
                    prefix: "0x".to_string(),
                    separator: "!".to_string(),
                    lowercase: true,
                    parse_prefixes: false,
                    filter_chars: false,
                    ..Default::default()
                },
            ),
        );
        assert!(log.has_warnings());
    }

    #[crate::ctb_test]
    fn test_collapse_filtered() {
        let settings = BaseStringFormatSettings {
            collapse_filtered: true,
            ..Default::default()
        };
        // "10_000" should collapse '_' and parse as "10000"
        let res = base_to_base_string("10_0!00", 10, 10, &settings);
        let (res, log) = res.expect("Error");
        assert_eq!("10000", res);
        assert!(log.has_warnings());
    }

    #[crate::ctb_test]
    fn test_collapse_only() {
        let settings = BaseStringFormatSettings {
            collapse_only: vec!["_".to_string()],
            filter_chars: true,
            ..Default::default()
        };
        // "10_000" should collapse '_' and parse as "10000", leaving other filtered chars as spaces
        assert_string_ok_eq_no_warnings(
            "10000",
            base_to_base_string("10_000", 10, 10, &settings),
        );
        let conv = base_to_base_string("10_000!", 10, 10, &settings);
        assert!(conv.is_ok());
        let (conv, log) = conv.expect("checked");
        assert_eq!("10000", conv);
        assert!(log.has_warnings());
    }

    #[crate::ctb_test]
    fn test_base64_alphabet_digits() {
        let alpha = BaseAlphabet::Base64Standard;
        assert_eq!(alpha.char_for_digit(0).unwrap(), 'A');
        assert_eq!(alpha.char_for_digit(25).unwrap(), 'Z');
        assert_eq!(alpha.char_for_digit(26).unwrap(), 'a');
        assert_eq!(alpha.char_for_digit(51).unwrap(), 'z');
        assert_eq!(alpha.char_for_digit(52).unwrap(), '0');
        assert_eq!(alpha.char_for_digit(61).unwrap(), '9');
        assert_eq!(alpha.char_for_digit(62).unwrap(), '+');
        assert_eq!(alpha.char_for_digit(63).unwrap(), '/');
        assert!(alpha.char_for_digit(64).is_err());

        for d in 0..=63 {
            let ch = alpha.char_for_digit(d).unwrap();
            let parsed = alpha.digit_for_char(ch).unwrap();
            assert_eq!(d, parsed);
        }
    }

    #[crate::ctb_test]
    fn test_base64_standard_conversion() {
        // Base 10 (Standard) -> Base 64 (Base64Standard)
        let dec_to_b64 = BaseStringFormatSettings {
            input_alphabet: BaseAlphabet::Standard,
            output_alphabet: BaseAlphabet::Base64Standard,
            ..Default::default()
        };
        assert_string_ok_eq_no_warnings(
            "A B / BA D/",
            base_to_base_string("0 1 63 64 255", 10, 64, &dec_to_b64),
        );

        // Base 64 (Base64Standard) -> Base 10 (Standard)
        let b64_to_dec = BaseStringFormatSettings {
            input_alphabet: BaseAlphabet::Base64Standard,
            output_alphabet: BaseAlphabet::Standard,
            ..Default::default()
        };
        assert_string_ok_eq_no_warnings(
            "0 1 63 64 255",
            base_to_base_string("A B / BA D/", 64, 10, &b64_to_dec),
        );

        // Base 16 (Standard) -> Base 64 (Base64Standard)
        let hex_to_b64 = BaseStringFormatSettings {
            input_alphabet: BaseAlphabet::Standard,
            output_alphabet: BaseAlphabet::Base64Standard,
            ..Default::default()
        };
        assert_string_ok_eq_no_warnings(
            "D/ EA",
            base_to_base_string("ff 100", 16, 64, &hex_to_b64),
        );

        // Case sensitivity: 'a' is 26, 'A' is 0, '0' is 52
        assert_string_ok_eq_no_warnings(
            "26 0 52",
            base_to_base_string("a A 0", 64, 10, &b64_to_dec),
        );

        // Zero-padding with 'A'
        let pad_settings = BaseStringFormatSettings {
            output_alphabet: BaseAlphabet::Base64Standard,
            pad: BaseConversionPaddingMode {
                pad_l: 4,
                pad_fit: false,
            },
            ..Default::default()
        };
        assert_string_ok_eq_no_warnings(
            "AAAF",
            base_to_base_string("5", 10, 64, &pad_settings),
        );

        // Base 10 (Standard) -> Base 30 (Base64Standard)
        let dec_to_b30_b64 = BaseStringFormatSettings {
            input_alphabet: BaseAlphabet::Standard,
            output_alphabet: BaseAlphabet::Base64Standard,
            ..Default::default()
        };
        assert_string_ok_eq_no_warnings(
            "BBPBDU",
            base_to_base_string("25516010", 10, 30, &dec_to_b30_b64),
        );

        // Base 30 (Base64Standard) -> Base 10 (Standard)
        let b30_b64_to_dec = BaseStringFormatSettings {
            input_alphabet: BaseAlphabet::Base64Standard,
            output_alphabet: BaseAlphabet::Standard,
            ..Default::default()
        };
        assert_string_ok_eq_no_warnings(
            "25516010",
            base_to_base_string("BBPBDU", 30, 10, &b30_b64_to_dec),
        );

        // Base > 36 fails if Standard alphabet is configured
        let standard_settings = BaseStringFormatSettings::default();
        assert!(base_to_base_string("255", 10, 64, &standard_settings).is_err());
    }
}
