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
use malachite::{Integer, Natural};

use crate::base::{Base, format_natural, parse_natural};
use crate::calculator_classic::{CONST_E, CONST_PI, divide, power};

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

/// Analyzes a numeric input string to extract layout and precision details.
#[must_use]
pub fn analyze_input(s: &str) -> ParsedInput {
    let s_clean =
        s.strip_prefix('+').or_else(|| s.strip_prefix('-')).unwrap_or(s);
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

/// Parses a signed, potentially fractional numeric string in a given base,
/// scaling the result by $base^{dec\_places}$ to return an exact signed [`Integer`].
#[expect(
    clippy::arithmetic_side_effects,
    reason = "Natural is arbitrary-precision and cannot overflow"
)]
pub fn parse_scaled(s: &str, base: Base, dec_places: usize) -> Result<Integer> {
    let is_negative = s.starts_with('-');
    let s_clean =
        s.strip_prefix('+').or_else(|| s.strip_prefix('-')).unwrap_or(s);
    ensure!(!s_clean.is_empty(), "Cannot parse empty number string");

    let (int_str, frac_str) = if let Some((i, f)) = s_clean.split_once('.') {
        (i, f)
    } else {
        (s_clean, "")
    };

    let int_nat = if int_str.is_empty() {
        Natural::ZERO
    } else {
        parse_natural(int_str, base)?
    };

    let frac_nat = if frac_str.is_empty() {
        Natural::ZERO
    } else {
        parse_natural(frac_str, base)?
    };

    let radix_nat = Natural::from(base.radix());
    let mut scale_pow = Natural::from(1u8);
    for _ in 0..dec_places {
        scale_pow *= &radix_nat;
    }

    let mut shift_pow = Natural::from(1u8);
    let shift = dec_places.saturating_sub(frac_str.len());
    for _ in 0..shift {
        shift_pow *= &radix_nat;
    }

    let scaled = int_nat * scale_pow + frac_nat * shift_pow;
    let int_val = Integer::from(scaled);
    if is_negative {
        Ok(-int_val)
    } else {
        Ok(int_val)
    }
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
    /// Floating-point numeric literal or constant.
    Number(f64),
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

/// Parses a string into a number ($f64$), handling numbers and named constants (`pi`, `e`).
pub fn parse_number(s: &str) -> Result<f64> {
    let trimmed = s.trim();
    if trimmed.eq_ignore_ascii_case("pi") {
        Ok(CONST_PI)
    } else if trimmed.eq_ignore_ascii_case("e") {
        Ok(CONST_E)
    } else {
        trimmed
            .parse::<f64>()
            .with_context(|| format!("Invalid number: '{s}'"))
    }
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
        if c.is_ascii_digit() || c == '.' {
            let mut num_str = String::new();
            while let Some(&nc) = chars.peek() {
                if nc.is_ascii_digit() || nc == '.' {
                    num_str.push(nc);
                    chars.next();
                } else {
                    break;
                }
            }
            let val = parse_number(&num_str)?;
            tokens.push(MathToken::Number(val));
            continue;
        }
        if c.is_ascii_alphabetic() {
            let mut word = String::new();
            while let Some(&wc) = chars.peek() {
                if wc.is_ascii_alphabetic() {
                    word.push(wc);
                    chars.next();
                } else {
                    break;
                }
            }
            if word.eq_ignore_ascii_case("mod") {
                tokens.push(MathToken::Modulo);
            } else if let Ok(val) = parse_number(&word) {
                tokens.push(MathToken::Number(val));
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
            Ok(*n)
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
        evaluate_expression("10 / 0").unwrap_err();
        evaluate_expression("2 + (3 *").unwrap_err();
        evaluate_expression("2 + +").unwrap_err();
    }
}