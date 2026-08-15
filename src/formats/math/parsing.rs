//! Mathematical expression and numeric string tokenization, analysis, and parsing.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::{Result, ensure};
use malachite::base::num::arithmetic::traits::UnsignedAbs;
use malachite::base::num::basic::traits::Zero;
use malachite::base::num::conversion::traits::RoundingFrom;
use malachite::base::rounding_modes::RoundingMode;
use malachite::{Integer, Natural, Rational};

use crate::base::{Base, format_natural, parse_natural};
use crate::calculator_classic::{CONST_E, CONST_PI, divide, power};

/// Returns the numerator and denominator for Unicode vulgar fraction characters.
#[must_use]
pub fn unicode_vulgar_fraction(c: char) -> Option<(u64, u64)> {
    match c {
        '↉' => Some((0, 3)),
        '½' => Some((1, 2)),
        '⅓' => Some((1, 3)),
        '⅔' => Some((2, 3)),
        '¼' => Some((1, 4)),
        '¾' => Some((3, 4)),
        '⅕' => Some((1, 5)),
        '⅖' => Some((2, 5)),
        '⅗' => Some((3, 5)),
        '⅘' => Some((4, 5)),
        '⅙' => Some((1, 6)),
        '⅚' => Some((5, 6)),
        '⅛' => Some((1, 8)),
        '⅜' => Some((3, 8)),
        '⅝' => Some((5, 8)),
        '⅞' => Some((7, 8)),
        '⅐' => Some((1, 7)),
        '⅑' => Some((1, 9)),
        '⅒' => Some((1, 10)),
        _ => None,
    }
}

/// The semantic value of a parsed number or mathematical constant.
#[derive(Debug, Clone, Eq)]
pub enum NumberValue {
    /// Exact rational value.
    Rational(Rational),
    /// Named constant $\pi$.
    Pi,
    /// Named constant $e$ (Euler's number).
    E,
    /// Imaginary unit with rational coefficient: $k \cdot i$.
    Imaginary(Rational),
    /// Imaginary unit constant $i$.
    ImaginaryI,
    /// Positive infinity $\infty$.
    Infinity,
    /// Negative infinity $-\infty$.
    NegativeInfinity,
}

impl PartialEq for NumberValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Rational(a), Self::Rational(b)) => a == b,
            (Self::Pi, Self::Pi) | (Self::E, Self::E) => true,
            (Self::Infinity, Self::Infinity) | (Self::NegativeInfinity, Self::NegativeInfinity) => true,
            (Self::ImaginaryI, Self::ImaginaryI) => true,
            (Self::Imaginary(a), Self::Imaginary(b)) => a == b,
            (Self::ImaginaryI, Self::Imaginary(b)) | (Self::Imaginary(b), Self::ImaginaryI) => {
                b == &Rational::from(1u8)
            }
            _ => false,
        }
    }
}

impl std::ops::Mul<i64> for NumberValue {
    type Output = Self;

    #[expect(
        clippy::arithmetic_side_effects,
        reason = "Rational is arbitrary-precision"
    )]
    fn mul(self, rhs: i64) -> Self {
        match self {
            Self::ImaginaryI => {
                if rhs == 1 {
                    Self::ImaginaryI
                } else {
                    Self::Imaginary(Rational::from(rhs))
                }
            }
            Self::Imaginary(r) => Self::Imaginary(r * Rational::from(rhs)),
            Self::Infinity => {
                if rhs < 0 {
                    Self::NegativeInfinity
                } else {
                    Self::Infinity
                }
            }
            Self::NegativeInfinity => {
                if rhs < 0 {
                    Self::Infinity
                } else {
                    Self::NegativeInfinity
                }
            }
            Self::Rational(r) => Self::Rational(r * Rational::from(rhs)),
            Self::Pi | Self::E => self,
        }
    }
}

impl std::ops::Mul<i32> for NumberValue {
    type Output = Self;

    #[expect(
        clippy::arithmetic_side_effects,
        reason = "Rational is arbitrary-precision"
    )]
    fn mul(self, rhs: i32) -> Self {
        self * i64::from(rhs)
    }
}

impl NumberValue {
    /// Converts this number value to an `f64`.
    #[must_use]
    pub fn to_f64(&self) -> f64 {
        match self {
            Self::Rational(r) => {
                f64::rounding_from(r, RoundingMode::Nearest).0
            }
            Self::Pi => CONST_PI,
            Self::E => CONST_E,
            Self::Infinity => f64::INFINITY,
            Self::NegativeInfinity => f64::NEG_INFINITY,
            Self::ImaginaryI | Self::Imaginary(_) => f64::NAN,
        }
    }
}

/// Represents a parsed numerical value with formatting metadata and optional units.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedNumber {
    /// Numerical value or named symbolic constant.
    pub value: NumberValue,
    /// Number base in which the number was parsed.
    pub base: Base,
    /// Indicates whether the input had an explicit negative sign.
    pub is_negative: bool,
    /// Width of the integer digits in the original input string.
    pub int_width: usize,
    /// Number of fractional digits after decimal point in the original input string.
    pub frac_len: usize,
    /// Indicates whether a decimal point was explicitly present in the input.
    pub has_decimal: bool,
    /// Optional unit suffix (e.g. "lb", "g", "oz"), if present.
    pub unit_suffix: Option<String>,
}

impl ParsedNumber {
    /// Parses a string into a [`ParsedNumber`], supporting integers, fixed-point decimals,
    /// mixed numbers, Unicode vulgar fractions (`½`, `¾`), ASCII fractions (`3 1/2`),
    /// named constants (`pi`, `e`, `i`), and optional unit suffixes.
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "Natural, Integer, and Rational are arbitrary-precision"
    )]
    pub fn parse(s: &str, base: Base) -> Result<Self> {
        let trimmed = s.trim();
        ensure!(!trimmed.is_empty(), "Cannot parse empty number string");

        let (is_negative, s_no_sign) =
            if let Some(stripped) = trimmed.strip_prefix('-') {
                (true, stripped.trim_start())
            } else if let Some(stripped) = trimmed.strip_prefix('+') {
                (false, stripped.trim_start())
            } else {
                (false, trimmed)
            };

        ensure!(!s_no_sign.is_empty(), "Cannot parse empty number string after sign");

        if s_no_sign.eq_ignore_ascii_case("pi") || s_no_sign == "π" {
            return Ok(Self {
                value: NumberValue::Pi,
                base,
                is_negative,
                int_width: 0,
                frac_len: 0,
                has_decimal: false,
                unit_suffix: None,
            });
        }
        if s_no_sign.eq_ignore_ascii_case("e") {
            return Ok(Self {
                value: NumberValue::E,
                base,
                is_negative,
                int_width: 0,
                frac_len: 0,
                has_decimal: false,
                unit_suffix: None,
            });
        }
        if s_no_sign == "∞" || s_no_sign.eq_ignore_ascii_case("inf") || s_no_sign.eq_ignore_ascii_case("infinity") {
            let val = if is_negative {
                NumberValue::NegativeInfinity
            } else {
                NumberValue::Infinity
            };
            return Ok(Self {
                value: val,
                base,
                is_negative,
                int_width: 0,
                frac_len: 0,
                has_decimal: false,
                unit_suffix: None,
            });
        }
        if s_no_sign.eq_ignore_ascii_case("i") {
            let val = if is_negative {
                NumberValue::Imaginary(Rational::from(-1i8))
            } else {
                NumberValue::ImaginaryI
            };
            return Ok(Self {
                value: val,
                base,
                is_negative,
                int_width: 0,
                frac_len: 0,
                has_decimal: false,
                unit_suffix: None,
            });
        }

        // Check for imaginary number with coefficient like "5i", "-5i"
        if base == Base::Base10
            && let Some(coeff_str) = s_no_sign.strip_suffix(['i', 'I'])
        {
            let trimmed_coeff = coeff_str.trim();
            if !trimmed_coeff.is_empty() {
                let parsed_coeff = Self::parse(trimmed_coeff, base)?;
                if let Some(r) = parsed_coeff.to_rational() {
                    let signed_r = if is_negative { -r.clone() } else { r.clone() };
                    return Ok(Self {
                        value: NumberValue::Imaginary(signed_r),
                        base,
                        is_negative,
                        int_width: parsed_coeff.int_width,
                        frac_len: parsed_coeff.frac_len,
                        has_decimal: parsed_coeff.has_decimal,
                        unit_suffix: None,
                    });
                }
            }
        }

        // Separate attached or whitespace-delimited unit suffixes if present
        let (num_part, unit_suffix) = separate_unit_suffix(s_no_sign, base);

        let int_width;
        let mut frac_len = 0usize;
        let mut has_decimal = false;

        // Check for Unicode vulgar fraction (e.g. "3½", "½")
        let mut vulgar_frac = None;
        for (i, c) in num_part.char_indices() {
            if let Some(frac) = unicode_vulgar_fraction(c) {
                vulgar_frac = Some((i, frac));
                break;
            }
        }

        let rational_mag: Rational = if let Some((idx, (num, den))) = vulgar_frac {
            let prefix = num_part.get(..idx).unwrap_or("").trim();
            let frac_rat = Rational::from_naturals(Natural::from(num), Natural::from(den));
            if prefix.is_empty() {
                int_width = 0;
                frac_rat
            } else {
                let int_nat = parse_natural(prefix, base)?;
                int_width = prefix.len();
                Rational::from(int_nat) + frac_rat
            }
        } else if let Some((prefix, den_str)) = num_part.split_once('⅟') {
            // Fraction numerator one ⅟ (e.g. "⅟2" or "3 ⅟2")
            let den_nat = parse_natural(den_str.trim(), base)?;
            ensure!(den_nat > Natural::ZERO, "Fraction denominator cannot be zero");
            let frac_rat = Rational::from_naturals(Natural::from(1u8), den_nat);
            let trimmed_prefix = prefix.trim();
            if trimmed_prefix.is_empty() {
                int_width = 0;
                frac_rat
            } else {
                let int_nat = parse_natural(trimmed_prefix, base)?;
                int_width = trimmed_prefix.len();
                Rational::from(int_nat) + frac_rat
            }
        } else if base != Base::Base64
            && (num_part.contains('/') || num_part.contains('⁄'))
        {
            let (num_str, den_str) = if let Some(parts) = num_part.split_once('⁄') {
                parts
            } else if let Some(parts) = num_part.split_once('/') {
                parts
            } else {
                unreachable!("contains verified");
            };

            let den_nat = parse_natural(den_str.trim(), base)?;
            ensure!(den_nat > Natural::ZERO, "Fraction denominator cannot be zero");

            let trimmed_num = num_str.trim();
            if let Some((int_str, frac_num_str)) = trimmed_num.rsplit_once([' ', '-']) {
                let int_nat = parse_natural(int_str.trim(), base)?;
                let f_num_nat = parse_natural(frac_num_str.trim(), base)?;
                int_width = int_str.trim().len();
                Rational::from(int_nat) + Rational::from_naturals(f_num_nat, den_nat)
            } else {
                let f_num_nat = parse_natural(trimmed_num, base)?;
                int_width = 0;
                Rational::from_naturals(f_num_nat, den_nat)
            }
        } else if let Some((int_part, frac_part)) = num_part.split_once('.') {
            // Fixed-point decimal
            has_decimal = true;
            int_width = int_part.len();
            frac_len = frac_part.len();

            let int_nat = if int_part.is_empty() {
                Natural::ZERO
            } else {
                parse_natural(int_part, base)?
            };

            let frac_nat = if frac_part.is_empty() {
                Natural::ZERO
            } else {
                parse_natural(frac_part, base)?
            };

            let radix_nat = Natural::from(base.radix());
            let mut scale_pow = Natural::from(1u8);
            for _ in 0..frac_len {
                scale_pow *= &radix_nat;
            }

            let fraction_rational = Rational::from_naturals(frac_nat, scale_pow);
            Rational::from(int_nat) + fraction_rational
        } else {
            // Pure integer
            int_width = num_part.len();
            let int_nat = parse_natural(num_part, base)?;
            Rational::from(int_nat)
        };

        let final_rat = if is_negative {
            -rational_mag
        } else {
            rational_mag
        };

        Ok(Self {
            value: NumberValue::Rational(final_rat),
            base,
            is_negative,
            int_width,
            frac_len,
            has_decimal,
            unit_suffix,
        })
    }

    /// Converts this parsed number to an `f64`.
    #[must_use]
    pub fn to_f64(&self) -> f64 {
        self.value.to_f64()
    }

    /// Scales the number value by $base^{dec\_places}$ and returns the resulting exact signed [`Integer`].
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "Natural, Integer, and Rational are arbitrary-precision"
    )]
    pub fn to_scaled_integer(&self, dec_places: usize) -> Result<Integer> {
        match &self.value {
            NumberValue::Rational(rat) => {
                let radix_nat = Natural::from(self.base.radix());
                let mut scale_pow = Natural::from(1u8);
                for _ in 0..dec_places {
                    scale_pow *= &radix_nat;
                }
                let is_neg = rat < &Rational::ZERO;
                let abs_rat = if is_neg { -rat.clone() } else { rat.clone() };
                let scaled = abs_rat * Rational::from(scale_pow);
                let (num, den) = scaled.into_numerator_and_denominator();
                let int_nat = &num / &den;
                let int_val = Integer::from(int_nat);
                if is_neg {
                    Ok(-int_val)
                } else {
                    Ok(int_val)
                }
            }
            _ => {
                bail!("Cannot convert symbolic constant to scaled integer")
            }
        }
    }

    /// Returns a reference to the exact rational value, if not a symbolic constant.
    #[must_use]
    pub fn to_rational(&self) -> Option<&Rational> {
        match &self.value {
            NumberValue::Rational(r) => Some(r),
            _ => None,
        }
    }
}

/// Helper function to separate trailing unit identifiers from a numeric substring.
fn separate_unit_suffix(s: &str, base: Base) -> (&str, Option<String>) {
    if let Some((num, unit)) = s.rsplit_once(' ') {
        let trimmed_unit = unit.trim();
        let is_fraction_part = trimmed_unit.contains('/')
            || trimmed_unit.contains('⁄')
            || trimmed_unit.starts_with('⅟')
            || trimmed_unit.chars().all(|c| c.is_ascii_digit() || unicode_vulgar_fraction(c).is_some());
        if !trimmed_unit.is_empty() && !is_fraction_part {
            return (num.trim(), Some(trimmed_unit.to_owned()));
        }
    }

    if let Some(stripped) = s.strip_suffix('"') {
        return (stripped.trim_end(), Some("\"".to_owned()));
    }

    // Attached unit letters for Base10
    if base == Base::Base10 {
        let mut split_idx = s.len();
        for (i, c) in s.char_indices().rev() {
            if c.is_alphabetic() || c.is_ascii_digit() {
                // Check if starting a unit like "mm2" or "lb"
                if c.is_alphabetic() {
                    split_idx = i;
                }
            } else {
                break;
            }
        }
        if split_idx > 0 && split_idx < s.len() {
            let (num, unit) = s.split_at(split_idx);
            // Don't split if it's purely digits or imaginary
            if !num.is_empty() && unit != "i" && unit != "I" {
                return (num.trim(), Some(unit.to_owned()));
            }
        }
    }

    (s, None)
}

/// Backward-compatible wrapper parsing a numeric string to `f64`.
pub fn parse_number(s: &str) -> Result<f64> {
    let parsed = ParsedNumber::parse(s, Base::Base10)?;
    Ok(parsed.to_f64())
}

/// Metadata regarding a parsed numeric input string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedInput {
    /// Width of the integer portion of the string.
    pub int_width: usize,
    /// Width of the fractional portion of the string.
    pub frac_width: usize,
    /// Indicates whether a decimal point was present in the input.
    pub has_decimal: bool,
    /// Number of fractional digits.
    pub frac_len: usize,
}

/// Backward-compatible wrapper analyzing layout of a numeric string.
#[must_use]
pub fn analyze_input(s: &str) -> ParsedInput {
    if let Ok(num) = ParsedNumber::parse(s, Base::Base10) {
        ParsedInput {
            int_width: num.int_width,
            frac_width: num.frac_len,
            has_decimal: num.has_decimal,
            frac_len: num.frac_len,
        }
    } else {
        let s_clean = s.strip_prefix('+').or_else(|| s.strip_prefix('-')).unwrap_or(s);
        if let Some((int_part, frac_part)) = s_clean.split_once('.') {
            ParsedInput {
                int_width: int_part.len(),
                frac_width: frac_part.len(),
                has_decimal: true,
                frac_len: frac_part.len(),
            }
        } else {
            ParsedInput {
                int_width: s_clean.len(),
                frac_width: 0,
                has_decimal: false,
                frac_len: 0,
            }
        }
    }
}

/// Backward-compatible wrapper parsing a scaled number.
pub fn parse_scaled(s: &str, base: Base, dec_places: usize) -> Result<Integer> {
    let num = ParsedNumber::parse(s, base)?;
    num.to_scaled_integer(dec_places)
}

/// Formats a scaled [`Integer`] back into a string in the given base, respecting
/// decimal places, integer width, and fractional width.
#[expect(
    clippy::arithmetic_side_effects,
    reason = "Natural is arbitrary-precision and cannot overflow"
)]
pub fn format_scaled(
    val: &Integer,
    base: Base,
    dec_places: usize,
    int_width: usize,
    frac_width: usize,
    has_decimal: bool,
) -> Result<String> {
    let is_negative = *val < Integer::ZERO;
    let abs_nat = val.unsigned_abs();

    let formatted_mag = if dec_places == 0 && !has_decimal {
        format_natural(&abs_nat, base, int_width)?
    } else {
        let radix_nat = Natural::from(base.radix());
        let mut scale_pow = Natural::from(1u8);
        for _ in 0..dec_places {
            scale_pow *= &radix_nat;
        }

        let int_part = &abs_nat / &scale_pow;
        let frac_part = &abs_nat % &scale_pow;

        let int_str = format_natural(&int_part, base, int_width)?;
        let frac_str =
            format_natural(&frac_part, base, frac_width.max(dec_places))?;
        format!("{int_str}.{frac_str}")
    };

    if is_negative {
        Ok(format!("-{formatted_mag}"))
    } else {
        Ok(formatted_mag)
    }
}

/// Token representation in a mathematical expression.
#[derive(Debug, Clone, PartialEq)]
pub enum MathToken {
    /// Parsed numeric literal or constant.
    Number(ParsedNumber),
    /// Addition operator `+`.
    Plus,
    /// Subtraction operator `-`.
    Minus,
    /// Multiplication operator `*`.
    Multiply,
    /// Division operator `/`.
    Divide,
    /// Integer division operator `\`.
    IntDivide,
    /// Exponentiation operator `^`.
    Power,
    /// Modulo operator `%` or `Mod`.
    Modulo,
    /// Left parenthesis `(`.
    LParen,
    /// Right parenthesis `)`.
    RParen,
}

/// Tokenizes a mathematical expression string into a sequence of [`MathToken`]s.
pub fn tokenize_expression(expr: &str) -> Result<Vec<MathToken>> {
    let mut tokens = Vec::new();
    let mut chars = expr.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
            continue;
        }
        if c.is_ascii_digit() || c == '.' || unicode_vulgar_fraction(c).is_some() || c == '⅟' {
            let mut num_str = String::new();
            while let Some(&nc) = chars.peek() {
                if nc.is_ascii_digit() || nc == '.' || unicode_vulgar_fraction(nc).is_some() || nc == '⅟' {
                    num_str.push(nc);
                    chars.next();
                } else {
                    break;
                }
            }
            let parsed = ParsedNumber::parse(&num_str, Base::Base10)?;
            tokens.push(MathToken::Number(parsed));
            continue;
        }
        if c.is_alphabetic() || c == 'π' || c == '∞' {
            let mut word = String::new();
            while let Some(&wc) = chars.peek() {
                if wc.is_alphabetic() || wc == 'π' || wc == '∞' {
                    word.push(wc);
                    chars.next();
                } else {
                    break;
                }
            }
            if word.eq_ignore_ascii_case("mod") {
                tokens.push(MathToken::Modulo);
            } else if let Ok(parsed) = ParsedNumber::parse(&word, Base::Base10) {
                tokens.push(MathToken::Number(parsed));
            } else {
                bail!("Unknown identifier in expression: {word}");
            }
            continue;
        }
        match c {
            '+' => {
                tokens.push(MathToken::Plus);
                chars.next();
            }
            '-' => {
                tokens.push(MathToken::Minus);
                chars.next();
            }
            '*' => {
                tokens.push(MathToken::Multiply);
                chars.next();
            }
            '/' => {
                tokens.push(MathToken::Divide);
                chars.next();
            }
            '\\' => {
                tokens.push(MathToken::IntDivide);
                chars.next();
            }
            '^' => {
                tokens.push(MathToken::Power);
                chars.next();
            }
            '%' => {
                tokens.push(MathToken::Modulo);
                chars.next();
            }
            '(' => {
                tokens.push(MathToken::LParen);
                chars.next();
            }
            ')' => {
                tokens.push(MathToken::RParen);
                chars.next();
            }
            _ => bail!("Unexpected character in expression: {c}"),
        }
    }
    Ok(tokens)
}

/// Parses an expression with addition and subtraction.
pub fn parse_expr(tokens: &[MathToken], pos: &mut usize) -> Result<f64> {
    let mut left = parse_term(tokens, pos)?;
    while let Some(tok) = tokens.get(*pos) {
        match tok {
            MathToken::Plus => {
                *pos = pos.saturating_add(1);
                let right = parse_term(tokens, pos)?;
                left += right;
            }
            MathToken::Minus => {
                *pos = pos.saturating_add(1);
                let right = parse_term(tokens, pos)?;
                left -= right;
            }
            _ => break,
        }
    }
    Ok(left)
}

/// Parses a term with multiplication, division, integer division, and modulo.
pub fn parse_term(tokens: &[MathToken], pos: &mut usize) -> Result<f64> {
    let mut left = parse_power(tokens, pos)?;
    while let Some(tok) = tokens.get(*pos) {
        match tok {
            MathToken::Multiply => {
                *pos = pos.saturating_add(1);
                let right = parse_power(tokens, pos)?;
                left *= right;
            }
            MathToken::Divide => {
                *pos = pos.saturating_add(1);
                let right = parse_power(tokens, pos)?;
                left = divide(left, right)?;
            }
            MathToken::IntDivide => {
                *pos = pos.saturating_add(1);
                let right = parse_power(tokens, pos)?;
                if right == 0.0 {
                    bail!("Division by zero");
                }
                left = (left / right).trunc();
            }
            MathToken::Modulo => {
                *pos = pos.saturating_add(1);
                let right = parse_power(tokens, pos)?;
                if right == 0.0 {
                    bail!("Modulo by zero");
                }
                left %= right;
            }
            _ => break,
        }
    }
    Ok(left)
}

/// Parses exponentiation operations (right-associative).
pub fn parse_power(tokens: &[MathToken], pos: &mut usize) -> Result<f64> {
    let left = parse_unary(tokens, pos)?;
    if let Some(MathToken::Power) = tokens.get(*pos) {
        *pos = pos.saturating_add(1);
        let right = parse_power(tokens, pos)?;
        Ok(power(left, right))
    } else {
        Ok(left)
    }
}

/// Parses unary operators (`+` or `-`).
pub fn parse_unary(tokens: &[MathToken], pos: &mut usize) -> Result<f64> {
    if let Some(tok) = tokens.get(*pos) {
        match tok {
            MathToken::Plus => {
                *pos = pos.saturating_add(1);
                parse_unary(tokens, pos)
            }
            MathToken::Minus => {
                *pos = pos.saturating_add(1);
                let val = parse_unary(tokens, pos)?;
                Ok(-val)
            }
            _ => parse_primary(tokens, pos),
        }
    } else {
        bail!("Unexpected end of expression");
    }
}

/// Parses primary tokens (numbers or parenthesized expressions).
pub fn parse_primary(tokens: &[MathToken], pos: &mut usize) -> Result<f64> {
    let Some(tok) = tokens.get(*pos) else {
        bail!("Unexpected end of expression");
    };
    match tok {
        MathToken::Number(n) => {
            *pos = pos.saturating_add(1);
            Ok(n.to_f64())
        }
        MathToken::LParen => {
            *pos = pos.saturating_add(1);
            let val = parse_expr(tokens, pos)?;
            let Some(MathToken::RParen) = tokens.get(*pos) else {
                bail!("Missing closing parenthesis");
            };
            *pos = pos.saturating_add(1);
            Ok(val)
        }
        _ => bail!("Unexpected token in expression"),
    }
}

/// Evaluates a math expression string with standard operator precedence.
pub fn evaluate_expression(expr: &str) -> Result<f64> {
    let tokens = tokenize_expression(expr)?;
    let mut pos = 0usize;
    let result = parse_expr(&tokens, &mut pos)?;
    if pos < tokens.len() {
        bail!("Unexpected trailing token in expression");
    }
    Ok(result)
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
    fn test_parse_number() {
        assert_eq!(parse_number("123").unwrap(), 123.0);
        assert_eq!(parse_number("12.34").unwrap(), 12.34);
        assert_eq!(parse_number("pi").unwrap(), CONST_PI);
        assert_eq!(parse_number("PI").unwrap(), CONST_PI);
        assert_eq!(parse_number("e").unwrap(), CONST_E);
        assert_eq!(parse_number("E").unwrap(), CONST_E);
        parse_number("abc").unwrap_err();
    }

    #[crate::ctb_test]
    fn test_parsed_number_fractions_and_units() {
        let p1 = ParsedNumber::parse("3½", Base::Base10).unwrap();
        assert_eq!(p1.to_f64(), 3.5);
        assert_eq!(
            p1.to_rational().unwrap(),
            &Rational::from_naturals(Natural::from(7u8), Natural::from(2u8))
        );

        let p2 = ParsedNumber::parse("3 1/2 oz", Base::Base10).unwrap();
        assert_eq!(p2.to_f64(), 3.5);
        assert_eq!(p2.unit_suffix.as_deref(), Some("oz"));

        let p2 = ParsedNumber::parse("3 1⁄2\"", Base::Base10).unwrap();
        assert_eq!(p2.to_f64(), 3.5);
        assert_eq!(p2.unit_suffix.as_deref(), Some("\""));

        let p2 = ParsedNumber::parse("⅟2 mm2", Base::Base10).unwrap();
        assert_eq!(p2.to_f64(), 0.5);
        assert_eq!(p2.unit_suffix.as_deref(), Some("mm2"));

        let p3 = ParsedNumber::parse("3lb", Base::Base10).unwrap();
        assert_eq!(p3.to_f64(), 3.0);
        assert_eq!(p3.unit_suffix.as_deref(), Some("lb"));

        let p4 = ParsedNumber::parse("3½ g", Base::Base10).unwrap();
        assert_eq!(p4.to_f64(), 3.5);
        assert_eq!(p4.unit_suffix.as_deref(), Some("g"));

        let p4 = ParsedNumber::parse("↉ g", Base::Base10).unwrap();
        assert_eq!(p4.to_f64(), 0.0);
        assert_eq!(p4.unit_suffix.as_deref(), Some("g"));
    }

    #[crate::ctb_test]
    fn test_symbolic_constants_preserved() {
        let pi = ParsedNumber::parse("PI", Base::Base10).unwrap();
        assert_eq!(pi.value, NumberValue::Pi);
        assert_eq!(pi.to_f64(), CONST_PI);
        let pi = ParsedNumber::parse("π", Base::Base10).unwrap();
        assert_eq!(pi.value, NumberValue::Pi);

        let e = ParsedNumber::parse("e", Base::Base10).unwrap();
        assert_eq!(e.value, NumberValue::E);
        assert_eq!(e.to_f64(), CONST_E);

        let i = ParsedNumber::parse("i", Base::Base10).unwrap();
        assert_eq!(i.value, NumberValue::ImaginaryI);

        let i = ParsedNumber::parse("5i", Base::Base10).unwrap();
        assert_eq!(i.value, NumberValue::ImaginaryI * 5);

        let i = ParsedNumber::parse("-i", Base::Base10).unwrap();
        assert_eq!(i.value, NumberValue::ImaginaryI * -1);

        let i = ParsedNumber::parse("∞", Base::Base10).unwrap();
        assert_eq!(i.value, NumberValue::Infinity);
        let i = ParsedNumber::parse("-∞", Base::Base10).unwrap();
        assert_eq!(i.value, NumberValue::Infinity * -1);
    }

    #[crate::ctb_test]
    fn test_analyze_and_scaled() {
        let info = analyze_input("-0.010");
        assert_eq!(info.int_width, 1);
        assert_eq!(info.frac_width, 3);
        assert!(info.has_decimal);
        assert_eq!(info.frac_len, 3);

        let scaled = parse_scaled("-0.010", Base::Base10, 3).unwrap();
        assert_eq!(scaled, Integer::from(-10));
        let formatted =
            format_scaled(&scaled, Base::Base10, 3, 1, 3, true).unwrap();
        assert_eq!(formatted, "-0.010");
    }

    #[crate::ctb_test]
    fn test_evaluate_expression() {
        assert_eq!(evaluate_expression("2 + 3 * 4").unwrap(), 14.0);
        assert_eq!(evaluate_expression("(2 + 3) * 4").unwrap(), 20.0);
        assert_eq!(evaluate_expression("10 - 4 - 2").unwrap(), 4.0);
        assert_eq!(evaluate_expression("2 ^ 3").unwrap(), 8.0);
        assert_eq!(evaluate_expression("10 \\ 3").unwrap(), 3.0);
        assert_eq!(evaluate_expression("10 Mod 3").unwrap(), 1.0);
        assert_eq!(evaluate_expression("10 % 3").unwrap(), 1.0);
        assert_eq!(evaluate_expression("-5 + 10").unwrap(), 5.0);
        assert_eq!(evaluate_expression("pi * 2").unwrap(), CONST_PI * 2.0);
        assert_eq!(evaluate_expression("3½ / 5").unwrap(), 0.7);
        evaluate_expression("10 / 0").unwrap_err();
        evaluate_expression("2 + (3 *").unwrap_err();
        evaluate_expression("2 + +").unwrap_err();
    }
}