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

//! Base definitions and utilities for number base representation and conversion.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::{Result, anyhow, bail, ensure};
use ctb_formats_utilities::FormatLog;
use malachite::Natural;
use malachite::base::num::basic::traits::Zero;

/// Represents a mathematical integer base / radix (e.g. 1, 2, 8, 10, 16, 64).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Base(u8);

/// Alphabet used by a numeral system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BaseAlphabet {
    /// Standard alphanumeric alphabet (`0-9`, `A-Z`), case-insensitive for input.
    #[default]
    Standard,
    /// RFC 4648 Base64 alphabet (`A-Z`, `a-z`, `0-9`, `+`, `/`), case-sensitive.
    Base64Standard,
}

impl BaseAlphabet {
    /// Returns the maximum radix supported by this alphabet.
    #[must_use]
    pub const fn max_base(self) -> u8 {
        match self {
            Self::Standard => 36,
            Self::Base64Standard => 64,
        }
    }

    /// Returns true if the given base radix is valid for this alphabet.
    #[must_use]
    pub fn is_supported_base(self, radix: u8) -> bool {
        (1..=self.max_base()).contains(&radix)
    }

    /// Returns true if the given `Base` is valid for this alphabet.
    #[must_use]
    pub fn is_supported_base_type(self, base: Base) -> bool {
        self.is_supported_base(base.radix())
    }

    /// Returns the zero digit character for this alphabet.
    #[must_use]
    pub const fn zero_char(self) -> char {
        match self {
            Self::Standard => '0',
            Self::Base64Standard => 'A',
        }
    }

    /// Returns whether this alphabet is case-sensitive.
    #[must_use]
    pub const fn is_case_sensitive(self) -> bool {
        match self {
            Self::Standard => false,
            Self::Base64Standard => true,
        }
    }

    /// Returns the digit value for a character in this alphabet.
    pub fn digit_for_char(self, c: char) -> Result<u8> {
        match self {
            Self::Standard => {
                let uc = c.to_ascii_uppercase();
                match uc {
                    '0'..='9' => {
                        let offset = u8::try_from(
                            u32::from(uc).saturating_sub(u32::from(b'0')),
                        )?;
                        Ok(offset)
                    }
                    'A'..='Z' => {
                        let offset = u8::try_from(
                            u32::from(uc).saturating_sub(u32::from(b'A')),
                        )?;
                        Ok(10u8.saturating_add(offset))
                    }
                    _ => bail!(
                        "Character '{c}' is not a valid digit in standard alphabet"
                    ),
                }
            }
            Self::Base64Standard => match c {
                'A'..='Z' => {
                    let offset = u8::try_from(
                        u32::from(c).saturating_sub(u32::from(b'A')),
                    )?;
                    Ok(offset)
                }
                'a'..='z' => {
                    let offset = u8::try_from(
                        u32::from(c).saturating_sub(u32::from(b'a')),
                    )?;
                    Ok(26u8.saturating_add(offset))
                }
                '0'..='9' => {
                    let offset = u8::try_from(
                        u32::from(c).saturating_sub(u32::from(b'0')),
                    )?;
                    Ok(52u8.saturating_add(offset))
                }
                '+' => Ok(62),
                '/' => Ok(63),
                _ => bail!(
                    "Character '{c}' is not a valid digit in Base64Standard alphabet"
                ),
            },
        }
    }

    /// Returns the character for a digit value in this alphabet.
    pub fn char_for_digit(self, digit: u8) -> Result<char> {
        match self {
            Self::Standard => {
                if digit <= 9 {
                    let code = b'0'.saturating_add(digit);
                    Ok(char::from(code))
                } else if digit <= 35 {
                    let code = b'A'.saturating_add(digit.saturating_sub(10));
                    Ok(char::from(code))
                } else {
                    bail!(
                        "Digit {digit} out of range for standard alphabet (0..=35)"
                    )
                }
            }
            Self::Base64Standard => {
                if digit <= 25 {
                    let code = b'A'.saturating_add(digit);
                    Ok(char::from(code))
                } else if digit <= 51 {
                    let code = b'a'.saturating_add(digit.saturating_sub(26));
                    Ok(char::from(code))
                } else if digit <= 61 {
                    let code = b'0'.saturating_add(digit.saturating_sub(52));
                    Ok(char::from(code))
                } else if digit == 62 {
                    Ok('+')
                } else if digit == 63 {
                    Ok('/')
                } else {
                    bail!(
                        "Digit {digit} out of range for base64 alphabet (0..=63)"
                    )
                }
            }
        }
    }

    /// Returns true if character is a valid digit for the given numeric base.
    #[must_use]
    pub fn is_digit(self, c: char, base: u8) -> bool {
        if !self.is_supported_base(base) {
            return false;
        }
        match self.digit_for_char(c) {
            Ok(d) => d < base,
            Err(_) => false,
        }
    }
}

/// Represents a numeral system defined by a mathematical radix and a digit alphabet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NumeralSystem {
    pub radix: Base,
    pub alphabet: BaseAlphabet,
}

impl NumeralSystem {
    pub const UNARY: Self = Self {
        radix: Base::Unary,
        alphabet: BaseAlphabet::Standard,
    };
    pub const BINARY: Self = Self {
        radix: Base::Binary,
        alphabet: BaseAlphabet::Standard,
    };
    pub const OCTAL: Self = Self {
        radix: Base::Octal,
        alphabet: BaseAlphabet::Standard,
    };
    pub const DECIMAL: Self = Self {
        radix: Base::Decimal,
        alphabet: BaseAlphabet::Standard,
    };
    pub const HEX: Self = Self {
        radix: Base::Hex,
        alphabet: BaseAlphabet::Standard,
    };
    pub const BASE64: Self = Self {
        radix: Base::from_radix_const(64),
        alphabet: BaseAlphabet::Base64Standard,
    };

    /// Creates a new `NumeralSystem` with validation that the radix is supported by the alphabet.
    pub fn new(radix: Base, alphabet: BaseAlphabet) -> Result<Self> {
        ensure!(
            alphabet.is_supported_base_type(radix),
            "Radix {radix:?} (value {}) is not supported by alphabet {alphabet:?}",
            radix.radix()
        );
        Ok(Self { radix, alphabet })
    }

    /// Creates a `NumeralSystem` using the standard alphabet.
    pub fn standard(radix: Base) -> Result<Self> {
        Self::new(radix, BaseAlphabet::Standard)
    }

    /// Creates a `NumeralSystem` using the Base64 alphabet.
    pub fn base64(radix: Base) -> Result<Self> {
        Self::new(radix, BaseAlphabet::Base64Standard)
    }

    /// Resolves a `NumeralSystem` from a numeric radix.
    pub fn from_radix(radix: u8) -> Result<Self> {
        let base = Base::from_radix(radix)?;
        Ok(base.into())
    }

    /// Returns the character for a digit value in this numeral system.
    pub fn char_for_digit(self, digit: u8) -> Result<char> {
        if digit >= self.radix.radix() {
            bail!(
                "Digit {digit} is out of range for base {}",
                self.radix.radix()
            );
        }
        self.alphabet.char_for_digit(digit)
    }

    /// Parses a single character into a digit value in this numeral system.
    pub fn digit_for_char(self, c: char) -> Result<u8> {
        let digit = self.alphabet.digit_for_char(c)?;
        if digit >= self.radix.radix() {
            bail!(
                "Digit '{c}' ({digit}) is out of range for base {}",
                self.radix.radix()
            );
        }
        Ok(digit)
    }

    /// Returns true if character is a valid digit in this numeral system.
    #[must_use]
    pub fn is_digit(self, c: char) -> bool {
        self.digit_for_char(c).is_ok()
    }

    /// Returns the zero digit character for this numeral system.
    #[must_use]
    pub const fn zero_char(self) -> char {
        self.alphabet.zero_char()
    }
}

impl From<Base> for NumeralSystem {
    fn from(base: Base) -> Self {
        let alphabet = if base.radix() > 36 {
            BaseAlphabet::Base64Standard
        } else {
            BaseAlphabet::Standard
        };
        Self {
            radix: base,
            alphabet,
        }
    }
}

impl Base {
    #[expect(non_upper_case_globals, reason = "Base name alias constants")]
    pub const Unary: Self = Self(1);
    #[expect(non_upper_case_globals, reason = "Base name alias constants")]
    pub const Binary: Self = Self(2);
    #[expect(non_upper_case_globals, reason = "Base name alias constants")]
    pub const Octal: Self = Self(8);
    #[expect(non_upper_case_globals, reason = "Base name alias constants")]
    pub const Decimal: Self = Self(10);
    #[expect(non_upper_case_globals, reason = "Base name alias constants")]
    pub const Hex: Self = Self(16);
    #[expect(non_upper_case_globals, reason = "Base name alias constants")]
    pub const Hexadecimal: Self = Self(16);

    /// Creates a `Base` with range validation (1..=64).
    pub fn new(radix: u8) -> Result<Self> {
        ensure!((1..=64).contains(&radix), "Unsupported base radix: {radix}");
        Ok(Self(radix))
    }

    /// Creates a `Base` with const evaluation without range check.
    #[must_use]
    pub const fn from_radix_const(radix: u8) -> Self {
        Self(radix)
    }

    /// Creates a `Base` from a numeric radix with range validation.
    pub fn from_radix(radix: u8) -> Result<Self> {
        Self::new(radix)
    }

    /// Returns the integer radix represented by this base.
    #[must_use]
    pub const fn radix(self) -> u8 {
        self.0
    }

    /// Parses a base from a string representation, including numeric radices ("1".."64")
    /// and standard names/aliases ("unary", "bin", "binary", "oct", "octal", "dec", "decimal",
    /// "hex", "hexadecimal", "base64", etc.).
    pub fn from_str_or_name(s: &str) -> Result<Self> {
        let trimmed = s.trim();
        if let Ok(radix) = trimmed.parse::<u8>() {
            return Self::from_radix(radix);
        }
        match trimmed.to_ascii_lowercase().as_str() {
            "unary" | "un" => Ok(Self::Unary),
            "bin" | "binary" => Ok(Self::Binary),
            "oct" | "octal" => Ok(Self::Octal),
            "dec" | "decimal" => Ok(Self::Decimal),
            "hex" | "hexadecimal" => Ok(Self::Hex),
            "base64" | "b64" => Self::new(64),
            _ => bail!("Unknown or unsupported base: '{s}'"),
        }
    }
}

#[expect(non_upper_case_globals, reason = "Base name alias constants")]
pub const Unary: Base = Base::Unary;
#[expect(non_upper_case_globals, reason = "Base name alias constants")]
pub const Binary: Base = Base::Binary;
#[expect(non_upper_case_globals, reason = "Base name alias constants")]
pub const Octal: Base = Base::Octal;
#[expect(non_upper_case_globals, reason = "Base name alias constants")]
pub const Decimal: Base = Base::Decimal;
#[expect(non_upper_case_globals, reason = "Base name alias constants")]
pub const Hex: Base = Base::Hex;
#[expect(non_upper_case_globals, reason = "Base name alias constants")]
pub const Hexadecimal: Base = Base::Hexadecimal;

impl TryFrom<u8> for Base {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::from_radix(value)
    }
}

impl From<Base> for u8 {
    fn from(base: Base) -> Self {
        base.radix()
    }
}

/// Returns the single-character digit for a numeric value in the given base.
pub fn digit_to_char(digit: u8, base: Base) -> Result<char> {
    NumeralSystem::from(base).char_for_digit(digit)
}

/// Parses a single character into a digit value in the given base.
pub fn char_to_digit(c: char, base: Base) -> Result<u8> {
    NumeralSystem::from(base).digit_for_char(c)
}

/// Parses a string representing a number in the given numeral system into a `Natural`.
#[expect(
    clippy::arithmetic_side_effects,
    reason = "Natural is arbitrary-precision and cannot overflow"
)]
pub fn parse_natural_system(s: &str, system: NumeralSystem) -> Result<Natural> {
    ensure!(!s.is_empty(), "Cannot parse empty string as number");
    let radix = system.radix.radix();
    if radix == 1 {
        for ch in s.chars() {
            let d = system.alphabet.digit_for_char(ch)?;
            ensure!(d == 0, "Invalid digit '{ch}' for base 1");
        }
        return Ok(Natural::from(s.chars().count()));
    }

    let s = match system.radix {
        // Reason for fallback: numbers without base prefix retain original string representation.
        Base::Hex => s
            .strip_prefix("0x")
            .or_else(|| s.strip_prefix("0X"))
            .unwrap_or(s),
        // Reason for fallback: numbers without base prefix retain original string representation.
        Base::Binary => s
            .strip_prefix("0b")
            .or_else(|| s.strip_prefix("0B"))
            .unwrap_or(s),
        // Reason for fallback: numbers without base prefix retain original string representation.
        Base::Octal => s
            .strip_prefix("0o")
            .or_else(|| s.strip_prefix("0O"))
            .unwrap_or(s),
        _ => s,
    };
    ensure!(
        !s.is_empty(),
        "Cannot parse empty number string after prefix"
    );
    let mut acc = Natural::ZERO;
    let base_nat = Natural::from(radix);
    for ch in s.chars() {
        let d = system.digit_for_char(ch)?;
        acc = acc * &base_nat + Natural::from(d);
    }
    Ok(acc)
}

/// Parses a string representing a number in the given base into a `Natural`.
pub fn parse_natural(s: &str, base: Base) -> Result<Natural> {
    parse_natural_system(s, base.into())
}

/// Formats a `Natural` number in the given numeral system with optional minimum width.
#[expect(
    clippy::arithmetic_side_effects,
    reason = "Natural is arbitrary-precision and cannot overflow"
)]
pub fn format_natural_system(
    n: &Natural,
    system: NumeralSystem,
    min_width: usize,
) -> Result<String> {
    let radix = system.radix.radix();
    let mut raw = if radix == 1 {
        if *n == Natural::ZERO {
            String::new()
        } else {
            let count = usize::try_from(n).map_err(|e| {
                anyhow!("Value too large for base 1 format: {e:?}")
            })?;
            std::iter::repeat_n(system.zero_char(), count).collect()
        }
    } else if *n == Natural::ZERO {
        let zero_char = system.zero_char();
        zero_char.to_string()
    } else {
        let base_nat = Natural::from(radix);
        let mut digits = Vec::new();
        let mut val = n.clone();
        while val > Natural::ZERO {
            let rem = &val % &base_nat;
            let digit_u8 = u8::try_from(&rem)
                .map_err(|e| anyhow!("Remainder out of u8 range: {e:?}"))?;
            digits.push(system.char_for_digit(digit_u8)?);
            val /= &base_nat;
        }
        digits.into_iter().rev().collect::<String>()
    };

    if raw.len() < min_width {
        let zero_char = system.zero_char();
        let pad_len = min_width.saturating_sub(raw.len());
        let padding: String = std::iter::repeat_n(zero_char, pad_len).collect();
        raw = format!("{padding}{raw}");
    }
    Ok(raw)
}

/// Formats a `Natural` number in the given base with optional minimum width.
pub fn format_natural(
    n: &Natural,
    base: Base,
    min_width: usize,
) -> Result<String> {
    format_natural_system(n, base.into(), min_width)
}

#[derive(Clone, Debug, PartialEq, Eq)]
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
    #[must_use]
    pub fn none() -> Self {
        Self::default()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[expect(clippy::struct_excessive_bools, reason = "conversion format flags")]
pub struct BaseStringFormatSettings {
    /// The prefix to use for each number (e.g. 0x for hexadecimal)
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

/// Returns the integer represented by n in the requested base.
pub fn int_from_base_str_u32(s: &str, base: u8) -> Result<u32> {
    let nat = int_from_base_str_big_alphabet(s, base, BaseAlphabet::Standard)?;
    u32::try_from(&nat).map_err(|_| anyhow!("Did not fit in u32"))
    /* Old implementation:
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
    */
}

/// Returns the integer represented by n in the requested base.
pub fn int_from_base_str_u128(s: &str, base: u8) -> Result<u128> {
    let nat = int_from_base_str_big_alphabet(s, base, BaseAlphabet::Standard)?;
    u128::try_from(&nat).map_err(|_| anyhow!("Did not fit in u128"))
    /* Old implementation:

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
    Ok(acc) */
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
    if base == 1 {
        for ch in s.chars() {
            let d = alphabet.digit_for_char(ch)?;
            if d >= 1 {
                bail!("Digit '{ch}' (value {d}) >= base 1");
            }
        }
        return Ok(Natural::from(s.chars().count()));
    }
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
    if base == 1 {
        if n == 0 {
            return Ok(String::new());
        }
        let count = usize::try_from(&n).map_err(|e| {
            anyhow!("Value too large for base 1 conversion: {e:?}")
        })?;
        return Ok(std::iter::repeat_n(alphabet.zero_char(), count).collect());
    }
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
pub fn int_to_base_str(n: u32, base: u8) -> Result<String> {
    int_to_base_str_big_alphabet(Natural::from(n), base, BaseAlphabet::Standard)

    /* Old implementation:
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
    */
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
    if base == 1 {
        return Ok(n);
    }
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
    ensure!(
        is_supported_base_with_default_alphabet(base),
        "Unsupported base {base} (case folding is only supported for bases up to 36)"
    );
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

    let formatted =
        _format_base_string(out, base, settings, settings.output_alphabet)?;
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

    let (formatted_res, formatted_log) = _format_base_string(
        res,
        to_base,
        format_settings,
        format_settings.output_alphabet,
    )?;
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
    let max_digits =
        get_digits_needed(Natural::from(limit), from_base, from_alphabet)?;
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
        let nat = int_from_base_str_big_alphabet(
            num_chars,
            from_base,
            from_alphabet,
        )?;
        let formatted =
            int_to_base_str_big_alphabet(nat, to_base, to_alphabet)?;
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

fn _format_base_string(
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
        let max_digits =
            get_digits_needed(Natural::from(limit), base, to_alphabet)?;
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
            let pad_len = usize::try_from(padded_width)?
                .saturating_sub(token.chars().count());
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

/// Bases > 36 require the use of a different alphabet.
pub fn is_supported_base(base: u8) -> bool {
    (1..=64).contains(&base)
}

/// Bases > 36 require the use of a different alphabet.
pub fn is_supported_base_with_default_alphabet(base: u8) -> bool {
    (1..=36).contains(&base)
}
pub fn is_base_digit_alphabet(
    ch: char,
    base: u8,
    alphabet: BaseAlphabet,
) -> bool {
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
    fn test_base_radices_and_aliases() {
        assert_eq!(Base::Unary.radix(), 1);
        assert_eq!(Base::Binary.radix(), 2);
        assert_eq!(Base::Octal.radix(), 8);
        assert_eq!(Base::Decimal.radix(), 10);
        assert_eq!(Base::Hex.radix(), 16);
        assert_eq!(Base::Hexadecimal.radix(), 16);

        for r in 1..=64 {
            let b = Base::from_radix(r).unwrap();
            assert_eq!(b.radix(), r);
        }
        Base::from_radix(0).unwrap_err();
        Base::from_radix(65).unwrap_err();
    }

    #[crate::ctb_test]
    fn test_numeral_system() {
        let unary = NumeralSystem::UNARY;
        assert_eq!(unary.radix.radix(), 1);
        assert_eq!(unary.char_for_digit(0).unwrap(), '0');
        unary.char_for_digit(1).unwrap_err();

        let dec = NumeralSystem::DECIMAL;
        assert_eq!(dec.radix.radix(), 10);
        assert_eq!(dec.alphabet, BaseAlphabet::Standard);
        assert_eq!(dec.char_for_digit(9).unwrap(), '9');
        dec.char_for_digit(10).unwrap_err();

        let hex = NumeralSystem::HEX;
        assert_eq!(hex.char_for_digit(15).unwrap(), 'F');
        assert_eq!(hex.digit_for_char('f').unwrap(), 15);

        let b64 = NumeralSystem::BASE64;
        assert_eq!(b64.char_for_digit(63).unwrap(), '/');
        assert_eq!(b64.digit_for_char('/').unwrap(), 63);

        // Custom base 30 with Base64 alphabet
        let b30_b64 = NumeralSystem::new(
            Base::new(30).unwrap(),
            BaseAlphabet::Base64Standard,
        )
        .unwrap();
        assert_eq!(b30_b64.char_for_digit(0).unwrap(), 'A');
        assert_eq!(b30_b64.char_for_digit(20).unwrap(), 'U');
        b30_b64.char_for_digit(30).unwrap_err();
    }

    #[crate::ctb_test]
    fn test_digit_conversions() {
        // Base 16
        assert_eq!(digit_to_char(10, Base::Hex).unwrap(), 'A');
        assert_eq!(char_to_digit('a', Base::Hex).unwrap(), 10);
        assert_eq!(char_to_digit('A', Base::Hex).unwrap(), 10);
        assert_eq!(char_to_digit('F', Base::Hex).unwrap(), 15);
        char_to_digit('G', Base::Hex).unwrap_err();

        let b64_base = Base::new(64).unwrap();
        assert_eq!(digit_to_char(0, b64_base).unwrap(), 'A');
        assert_eq!(digit_to_char(25, b64_base).unwrap(), 'Z');
        assert_eq!(digit_to_char(26, b64_base).unwrap(), 'a');
        assert_eq!(digit_to_char(51, b64_base).unwrap(), 'z');
        assert_eq!(digit_to_char(52, b64_base).unwrap(), '0');
        assert_eq!(digit_to_char(61, b64_base).unwrap(), '9');
        assert_eq!(digit_to_char(62, b64_base).unwrap(), '+');
        assert_eq!(digit_to_char(63, b64_base).unwrap(), '/');

        assert_eq!(char_to_digit('A', b64_base).unwrap(), 0);
        assert_eq!(char_to_digit('Z', b64_base).unwrap(), 25);
        assert_eq!(char_to_digit('a', b64_base).unwrap(), 26);
        assert_eq!(char_to_digit('z', b64_base).unwrap(), 51);
        assert_eq!(char_to_digit('0', b64_base).unwrap(), 52);
        assert_eq!(char_to_digit('9', b64_base).unwrap(), 61);
        assert_eq!(char_to_digit('+', b64_base).unwrap(), 62);
        assert_eq!(char_to_digit('/', b64_base).unwrap(), 63);
    }

    #[crate::ctb_test]
    fn test_parse_and_format_natural() {
        let n = parse_natural("1A", Base::Hex).unwrap();
        assert_eq!(n, Natural::from(26u32));
        assert_eq!(format_natural(&n, Base::Hex, 4).unwrap(), "001A");
        assert_eq!(format_natural(&n, Base::Decimal, 1).unwrap(), "26");

        let b64_base = Base::new(64).unwrap();
        let b64_n = parse_natural("BA", b64_base).unwrap();
        // 'B' = 1, 'A' = 0 -> 1 * 64 + 0 = 64
        assert_eq!(b64_n, Natural::from(64u32));
        assert_eq!(format_natural(&b64_n, b64_base, 3).unwrap(), "ABA");

        // Format and parse with custom NumeralSystem
        let b30_b64 = NumeralSystem::new(
            Base::new(30).unwrap(),
            BaseAlphabet::Base64Standard,
        )
        .unwrap();
        let val = Natural::from(25516010u32);
        let s = format_natural_system(&val, b30_b64, 0).unwrap();
        assert_eq!(s, "BBPBDU");
        let parsed = parse_natural_system(&s, b30_b64).unwrap();
        assert_eq!(parsed, val);

        // Base 1 (unary) tests
        let un_val = Natural::from(5u32);
        let un_str = format_natural(&un_val, Base::Unary, 0).unwrap();
        assert_eq!(un_str, "00000");
        let un_parsed = parse_natural(&un_str, Base::Unary).unwrap();
        assert_eq!(un_parsed, un_val);

        let un_b64_sys =
            NumeralSystem::new(Base::Unary, BaseAlphabet::Base64Standard)
                .unwrap();
        let un_b64_str = format_natural_system(&un_val, un_b64_sys, 0).unwrap();
        assert_eq!(un_b64_str, "AAAAA");
        let un_b64_parsed =
            parse_natural_system(&un_b64_str, un_b64_sys).unwrap();
        assert_eq!(un_b64_parsed, un_val);
    }
}
