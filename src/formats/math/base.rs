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

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::{Result, anyhow, bail, ensure};
use malachite::Natural;
use malachite::base::num::basic::traits::Zero;

/// Represents a mathematical integer base / radix (e.g. 2, 8, 10, 16, 64).
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

    /// Returns true if the given radix is valid for this alphabet.
    #[must_use]
    pub fn is_supported_base(self, radix: Base) -> bool {
        (1..=self.max_base()).contains(&radix.radix())
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
                        let offset = u8::try_from(u32::from(uc).saturating_sub(u32::from(b'0')))?;
                        Ok(offset)
                    }
                    'A'..='Z' => {
                        let offset = u8::try_from(u32::from(uc).saturating_sub(u32::from(b'A')))?;
                        Ok(10u8.saturating_add(offset))
                    }
                    _ => bail!("Character '{c}' is not a valid digit in standard alphabet"),
                }
            }
            Self::Base64Standard => match c {
                'A'..='Z' => {
                    let offset = u8::try_from(u32::from(c).saturating_sub(u32::from(b'A')))?;
                    Ok(offset)
                }
                'a'..='z' => {
                    let offset = u8::try_from(u32::from(c).saturating_sub(u32::from(b'a')))?;
                    Ok(26u8.saturating_add(offset))
                }
                '0'..='9' => {
                    let offset = u8::try_from(u32::from(c).saturating_sub(u32::from(b'0')))?;
                    Ok(52u8.saturating_add(offset))
                }
                '+' => Ok(62),
                '/' => Ok(63),
                _ => bail!("Character '{c}' is not a valid digit in base64 alphabet"),
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
                    bail!("Digit {digit} out of range for standard alphabet (0..=35)")
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
                    bail!("Digit {digit} out of range for base64 alphabet (0..=63)")
                }
            }
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
        radix: Base::Base64,
        alphabet: BaseAlphabet::Base64Standard,
    };

    /// Creates a new `NumeralSystem` with validation that the radix is supported by the alphabet.
    pub fn new(radix: Base, alphabet: BaseAlphabet) -> Result<Self> {
        ensure!(
            alphabet.is_supported_base(radix),
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
    #[allow(non_upper_case_globals, reason = "Base name alias constants")]
    pub const Binary: Self = Self(2);
    #[allow(non_upper_case_globals, reason = "Base name alias constants")]
    pub const Octal: Self = Self(8);
    #[allow(non_upper_case_globals, reason = "Base name alias constants")]
    pub const Decimal: Self = Self(10);
    #[allow(non_upper_case_globals, reason = "Base name alias constants")]
    pub const Hex: Self = Self(16);
    #[allow(non_upper_case_globals, reason = "Base name alias constants")]
    pub const Hexadecimal: Self = Self(16);
    #[allow(non_upper_case_globals, reason = "Base name alias constants")]

    /// Creates a `Base` with range validation (2..=64).
    pub fn new(radix: u8) -> Result<Self> {
        ensure!((1..).contains(&radix), "Unsupported base radix: {radix}");
        Ok(Self(radix))
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

    /// Parses a base from a string representation, including numeric radices ("2".."64")
    /// and standard names/aliases ("bin", "binary", "oct", "octal", "dec", "decimal",
    /// "hex", "hexadecimal", "base64", etc.).
    pub fn from_str_or_name(s: &str) -> Result<Self> {
        let trimmed = s.trim();
        if let Ok(radix) = trimmed.parse::<u8>() {
            return Self::from_radix(radix);
        }
        match trimmed.to_ascii_lowercase().as_str() {
            "bin" | "binary" => Ok(Self::Binary),
            "oct" | "octal" => Ok(Self::Octal),
            "dec" | "decimal" => Ok(Self::Decimal),
            "hex" | "hexadecimal" => Ok(Self::Hex),
            "base64" | "b64" => Ok(Self::Base64),
            _ => bail!("Unknown or unsupported base: '{s}'"),
        }
    }
}

#[allow(non_upper_case_globals, reason = "Base name alias constants")]
pub const Binary: Base = Base::Binary;
#[allow(non_upper_case_globals, reason = "Base name alias constants")]
pub const Octal: Base = Base::Octal;
#[allow(non_upper_case_globals, reason = "Base name alias constants")]
pub const Decimal: Base = Base::Decimal;
#[allow(non_upper_case_globals, reason = "Base name alias constants")]
pub const Hex: Base = Base::Hex;
#[allow(non_upper_case_globals, reason = "Base name alias constants")]
pub const Hexadecimal: Base = Base::Hexadecimal;
#[allow(non_upper_case_globals, reason = "Base name alias constants")]

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
    ensure!(!s.is_empty(), "Cannot parse empty number string after prefix");
    let radix = system.radix.radix();
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
    let mut raw = if *n == Natural::ZERO {
        let zero_char = system.zero_char();
        zero_char.to_string()
    } else {
        let radix = system.radix.radix();
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
    fn test_base_radices_and_aliases() {
        assert_eq!(Base::Binary.radix(), 2);
        assert_eq!(Base::Octal.radix(), 8);
        assert_eq!(Base::Decimal.radix(), 10);
        assert_eq!(Base::Hex.radix(), 16);
        assert_eq!(Base::Hexadecimal.radix(), 16);
        assert_eq!(Base::Base64.radix(), 64);
        assert_eq!(Base::Base64_Standard.radix(), 64);

        for r in 2..=64 {
            let b = Base::from_radix(r).unwrap();
            assert_eq!(b.radix(), r);
        }
        assert_eq!(Base::from_radix(64).unwrap(), Base::Base64);
        assert!(Base::from_radix(0).is_err());
        assert!(Base::from_radix(1).is_err());
        assert!(Base::from_radix(65).is_err());
    }

    #[crate::ctb_test]
    fn test_numeral_system() {
        let dec = NumeralSystem::DECIMAL;
        assert_eq!(dec.radix.radix(), 10);
        assert_eq!(dec.alphabet, BaseAlphabet::Standard);
        assert_eq!(dec.char_for_digit(9).unwrap(), '9');
        assert!(dec.char_for_digit(10).is_err());

        let hex = NumeralSystem::HEX;
        assert_eq!(hex.char_for_digit(15).unwrap(), 'F');
        assert_eq!(hex.digit_for_char('f').unwrap(), 15);

        let b64 = NumeralSystem::BASE64;
        assert_eq!(b64.char_for_digit(63).unwrap(), '/');
        assert_eq!(b64.digit_for_char('/').unwrap(), 63);

        // Custom base 30 with Base64 alphabet
        let b30_b64 = NumeralSystem::new(Base::Base30, BaseAlphabet::Base64Standard).unwrap();
        assert_eq!(b30_b64.char_for_digit(0).unwrap(), 'A');
        assert_eq!(b30_b64.char_for_digit(20).unwrap(), 'U');
        assert!(b30_b64.char_for_digit(30).is_err());
    }

    #[crate::ctb_test]
    fn test_digit_conversions() {
        // Base 16
        assert_eq!(digit_to_char(10, Base::Hex).unwrap(), 'A');
        assert_eq!(char_to_digit('a', Base::Hex).unwrap(), 10);
        assert_eq!(char_to_digit('A', Base::Hex).unwrap(), 10);
        assert_eq!(char_to_digit('F', Base::Hex).unwrap(), 15);
        assert!(char_to_digit('G', Base::Hex).is_err());

        // Base 64
        assert_eq!(digit_to_char(0, Base::Base64).unwrap(), 'A');
        assert_eq!(digit_to_char(25, Base::Base64).unwrap(), 'Z');
        assert_eq!(digit_to_char(26, Base::Base64).unwrap(), 'a');
        assert_eq!(digit_to_char(51, Base::Base64).unwrap(), 'z');
        assert_eq!(digit_to_char(52, Base::Base64).unwrap(), '0');
        assert_eq!(digit_to_char(61, Base::Base64).unwrap(), '9');
        assert_eq!(digit_to_char(62, Base::Base64).unwrap(), '+');
        assert_eq!(digit_to_char(63, Base::Base64).unwrap(), '/');

        assert_eq!(char_to_digit('A', Base::Base64).unwrap(), 0);
        assert_eq!(char_to_digit('Z', Base::Base64).unwrap(), 25);
        assert_eq!(char_to_digit('a', Base::Base64).unwrap(), 26);
        assert_eq!(char_to_digit('z', Base::Base64).unwrap(), 51);
        assert_eq!(char_to_digit('0', Base::Base64).unwrap(), 52);
        assert_eq!(char_to_digit('9', Base::Base64).unwrap(), 61);
        assert_eq!(char_to_digit('+', Base::Base64).unwrap(), 62);
        assert_eq!(char_to_digit('/', Base::Base64).unwrap(), 63);
    }

    #[crate::ctb_test]
    fn test_parse_and_format_natural() {
        let n = parse_natural("1A", Base::Hex).unwrap();
        assert_eq!(n, Natural::from(26u32));
        assert_eq!(format_natural(&n, Base::Hex, 4).unwrap(), "001A");
        assert_eq!(format_natural(&n, Base::Decimal, 1).unwrap(), "26");

        let b64_n = parse_natural("BA", Base::Base64).unwrap();
        // 'B' = 1, 'A' = 0 -> 1 * 64 + 0 = 64
        assert_eq!(b64_n, Natural::from(64u32));
        assert_eq!(format_natural(&b64_n, Base::Base64, 3).unwrap(), "ABA");

        // Format and parse with custom NumeralSystem
        let b30_b64 = NumeralSystem::new(Base::Base30, BaseAlphabet::Base64Standard).unwrap();
        let val = Natural::from(25516010u32);
        let s = format_natural_system(&val, b30_b64, 0).unwrap();
        assert_eq!(s, "BBPBDU");
        let parsed = parse_natural_system(&s, b30_b64).unwrap();
        assert_eq!(parsed, val);
    }
}