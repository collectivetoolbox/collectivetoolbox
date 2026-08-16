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
use malachite::base::num::conversion::traits::ToStringBase;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Base {
    Base2,
    Base3,
    Base4,
    Base5,
    Base6,
    Base7,
    Base8,
    Base9,
    Base10,
    Base11,
    Base12,
    Base13,
    Base14,
    Base15,
    Base16,
    Base17,
    Base18,
    Base19,
    Base20,
    Base21,
    Base22,
    Base23,
    Base24,
    Base25,
    Base26,
    Base27,
    Base28,
    Base29,
    Base30,
    Base31,
    Base32,
    Base33,
    Base34,
    Base35,
    Base36,
    /// Base64 encoding without padding using the standard RFC 4648 alphabet.
    Base64,
}

pub use Base::*;

#[allow(non_upper_case_globals, reason = "Base name alias constants")]
pub const Binary: Base = Base::Base2;
#[allow(non_upper_case_globals, reason = "Base name alias constants")]
pub const Octal: Base = Base::Base8;
#[allow(non_upper_case_globals, reason = "Base name alias constants")]
pub const Decimal: Base = Base::Base10;
#[allow(non_upper_case_globals, reason = "Base name alias constants")]
pub const Hex: Base = Base::Base16;
#[allow(non_upper_case_globals, reason = "Base name alias constants")]
pub const Hexadecimal: Base = Base::Base16;
#[allow(non_upper_case_globals, reason = "Base name alias constants")]
pub const Hexcadecimal: Base = Base::Base16;
#[allow(non_upper_case_globals, reason = "Base name alias constants")]
pub const Base64_Standard: Base = Base::Base64;

impl Base {
    #[allow(non_upper_case_globals, reason = "Base name alias constants")]
    pub const Binary: Self = Self::Base2;
    #[allow(non_upper_case_globals, reason = "Base name alias constants")]
    pub const Octal: Self = Self::Base8;
    #[allow(non_upper_case_globals, reason = "Base name alias constants")]
    pub const Decimal: Self = Self::Base10;
    #[allow(non_upper_case_globals, reason = "Base name alias constants")]
    pub const Hex: Self = Self::Base16;
    #[allow(non_upper_case_globals, reason = "Base name alias constants")]
    pub const Hexadecimal: Self = Self::Base16;
    #[allow(non_upper_case_globals, reason = "Base name alias constants")]
    pub const Hexcadecimal: Self = Self::Base16;
    #[allow(non_upper_case_globals, reason = "Base name alias constants")]
    pub const Base64_Standard: Self = Self::Base64;

    /// Returns the integer radix represented by this base.
    #[must_use]
    pub fn radix(self) -> u8 {
        match self {
            Self::Base2 => 2,
            Self::Base3 => 3,
            Self::Base4 => 4,
            Self::Base5 => 5,
            Self::Base6 => 6,
            Self::Base7 => 7,
            Self::Base8 => 8,
            Self::Base9 => 9,
            Self::Base10 => 10,
            Self::Base11 => 11,
            Self::Base12 => 12,
            Self::Base13 => 13,
            Self::Base14 => 14,
            Self::Base15 => 15,
            Self::Base16 => 16,
            Self::Base17 => 17,
            Self::Base18 => 18,
            Self::Base19 => 19,
            Self::Base20 => 20,
            Self::Base21 => 21,
            Self::Base22 => 22,
            Self::Base23 => 23,
            Self::Base24 => 24,
            Self::Base25 => 25,
            Self::Base26 => 26,
            Self::Base27 => 27,
            Self::Base28 => 28,
            Self::Base29 => 29,
            Self::Base30 => 30,
            Self::Base31 => 31,
            Self::Base32 => 32,
            Self::Base33 => 33,
            Self::Base34 => 34,
            Self::Base35 => 35,
            Self::Base36 => 36,
            Self::Base64 => 64,
        }
    }

    /// Returns the `Base` variant corresponding to the numeric radix.
    pub fn from_radix(radix: u8) -> Result<Self> {
        match radix {
            2 => Ok(Self::Base2),
            3 => Ok(Self::Base3),
            4 => Ok(Self::Base4),
            5 => Ok(Self::Base5),
            6 => Ok(Self::Base6),
            7 => Ok(Self::Base7),
            8 => Ok(Self::Base8),
            9 => Ok(Self::Base9),
            10 => Ok(Self::Base10),
            11 => Ok(Self::Base11),
            12 => Ok(Self::Base12),
            13 => Ok(Self::Base13),
            14 => Ok(Self::Base14),
            15 => Ok(Self::Base15),
            16 => Ok(Self::Base16),
            17 => Ok(Self::Base17),
            18 => Ok(Self::Base18),
            19 => Ok(Self::Base19),
            20 => Ok(Self::Base20),
            21 => Ok(Self::Base21),
            22 => Ok(Self::Base22),
            23 => Ok(Self::Base23),
            24 => Ok(Self::Base24),
            25 => Ok(Self::Base25),
            26 => Ok(Self::Base26),
            27 => Ok(Self::Base27),
            28 => Ok(Self::Base28),
            29 => Ok(Self::Base29),
            30 => Ok(Self::Base30),
            31 => Ok(Self::Base31),
            32 => Ok(Self::Base32),
            33 => Ok(Self::Base33),
            34 => Ok(Self::Base34),
            35 => Ok(Self::Base35),
            36 => Ok(Self::Base36),
            64 => Ok(Self::Base64),
            _ => bail!("Unsupported base radix: {radix}"),
        }
    }

    /// Parses a base from a string representation, including numeric radices ("2".."36", "64")
    /// and standard names/aliases ("bin", "binary", "oct", "octal", "dec", "decimal",
    /// "hex", "hexadecimal", "base64", etc.).
    pub fn from_str_or_name(s: &str) -> Result<Self> {
        let trimmed = s.trim();
        if let Ok(radix) = trimmed.parse::<u8>() {
            return Self::from_radix(radix);
        }
        match trimmed.to_ascii_lowercase().as_str() {
            "bin" | "binary" => Ok(Self::Base2),
            "oct" | "octal" => Ok(Self::Base8),
            "dec" | "decimal" => Ok(Self::Base10),
            "hex" | "hexadecimal" | "hexcadecimal" => Ok(Self::Base16),
            "base64" | "b64" => Ok(Self::Base64),
            _ => bail!("Unknown or unsupported base: '{s}'"),
        }
    }
}

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
    let radix = base.radix();
    if digit >= radix {
        bail!("Digit {digit} is out of range for base {radix}");
    }
    match base {
        Base::Base64 => {
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
                bail!("Invalid digit {digit} for Base64")
            }
        }
        _ => {
            if digit <= 9 {
                let code = b'0'.saturating_add(digit);
                Ok(char::from(code))
            } else {
                let code = b'A'.saturating_add(digit.saturating_sub(10));
                Ok(char::from(code))
            }
        }
    }
}

/// Parses a single character into a digit value in the given base.
pub fn char_to_digit(c: char, base: Base) -> Result<u8> {
    let radix = base.radix();
    let digit = match base {
        Base::Base64 => match c {
            'A'..='Z' => {
                u8::try_from(u32::from(c).saturating_sub(u32::from(b'A')))?
            }
            'a'..='z' => {
                let offset =
                    u8::try_from(u32::from(c).saturating_sub(u32::from(b'a')))?;
                26u8.saturating_add(offset)
            }
            '0'..='9' => {
                let offset =
                    u8::try_from(u32::from(c).saturating_sub(u32::from(b'0')))?;
                52u8.saturating_add(offset)
            }
            '+' => 62,
            '/' => 63,
            _ => bail!("Invalid character '{c}' for Base64"),
        },
        _ => {
            let uc = c.to_ascii_uppercase();
            match uc {
                '0'..='9' => {
                    u8::try_from(u32::from(uc).saturating_sub(u32::from(b'0')))?
                }
                'A'..='Z' => {
                    let offset = u8::try_from(
                        u32::from(uc).saturating_sub(u32::from(b'A')),
                    )?;
                    10u8.saturating_add(offset)
                }
                _ => bail!("Invalid character '{c}' for base {radix}"),
            }
        }
    };
    if digit >= radix {
        bail!("Digit '{c}' ({digit}) is out of range for base {radix}");
    }
    Ok(digit)
}

/// Parses a string representing a number in the given base into a `Natural`.
#[expect(
    clippy::arithmetic_side_effects,
    reason = "Natural is arbitrary-precision and cannot overflow"
)]
pub fn parse_natural(s: &str, base: Base) -> Result<Natural> {
    ensure!(!s.is_empty(), "Cannot parse empty string as number");
    let s = match base {
        Base::Base16 => s
            .strip_prefix("0x")
            .or_else(|| s.strip_prefix("0X"))
            .unwrap_or(s),
        Base::Base2 => s
            .strip_prefix("0b")
            .or_else(|| s.strip_prefix("0B"))
            .unwrap_or(s),
        Base::Base8 => s
            .strip_prefix("0o")
            .or_else(|| s.strip_prefix("0O"))
            .unwrap_or(s),
        _ => s,
    };
    ensure!(!s.is_empty(), "Cannot parse empty number string after prefix");
    let radix = base.radix();
    let mut acc = Natural::ZERO;
    let base_nat = Natural::from(radix);
    for ch in s.chars() {
        let d = char_to_digit(ch, base)?;
        acc = acc * &base_nat + Natural::from(d);
    }
    Ok(acc)
}

/// Formats a `Natural` number in the given base with optional minimum width.
#[expect(
    clippy::arithmetic_side_effects,
    reason = "Natural is arbitrary-precision and cannot overflow"
)]
pub fn format_natural(
    n: &Natural,
    base: Base,
    min_width: usize,
) -> Result<String> {
    let mut raw = if *n == Natural::ZERO {
        let zero_char = digit_to_char(0, base)?;
        zero_char.to_string()
    } else {
        match base {
            Base::Base64 => {
                let mut digits = Vec::new();
                let mut val = n.clone();
                let base_nat = Natural::from(64u8);
                while val > Natural::ZERO {
                    let rem = &val % &base_nat;
                    let digit_u8 = u8::try_from(&rem)
                        .map_err(|e| anyhow!("Remainder out of u8 range: {e:?}"))?;
                    digits.push(digit_to_char(digit_u8, base)?);
                    val /= &base_nat;
                }
                digits.into_iter().rev().collect::<String>()
            }
            _ => n.to_string_base(base.radix()).to_uppercase(),
        }
    };

    if raw.len() < min_width {
        let zero_char = digit_to_char(0, base)?;
        let pad_len = min_width.saturating_sub(raw.len());
        let padding: String = std::iter::repeat_n(zero_char, pad_len).collect();
        raw = format!("{padding}{raw}");
    }
    Ok(raw)
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
        assert_eq!(Base::Hexcadecimal.radix(), 16);
        assert_eq!(Base::Base64.radix(), 64);
        assert_eq!(Base::Base64_Standard.radix(), 64);

        for r in 2..=36 {
            let b = Base::from_radix(r).unwrap();
            assert_eq!(b.radix(), r);
        }
        assert_eq!(Base::from_radix(64).unwrap(), Base::Base64);
        assert!(Base::from_radix(0).is_err());
        assert!(Base::from_radix(1).is_err());
        assert!(Base::from_radix(37).is_err());
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
    }
}