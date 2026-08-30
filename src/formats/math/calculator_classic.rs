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

//! Mathematical utilities translated faithfully from the legacy calculator
//! application suite (`old/calculator`), including Calculator 4.0, assistance/
//! error solvers, the `6r2` unique random number generator, and the `R.P.S.`
//! sidecar game.

#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

pub use crate::constant::{CONST_E, CONST_PI};

/// Result of a square root evaluation, representing real or imaginary output.
#[derive(Debug, Clone, PartialEq)]
pub enum SquareRootResult {
    /// Real square root result for non-negative numbers.
    Real(f64),
    /// Imaginary square root result for negative numbers ($x \cdot i$).
    Imaginary(f64),
}

/// Outcome of a Rock-Paper-Scissors game round.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RpsOutcome {
    /// User wins the round.
    Win,
    /// User loses the round.
    Loss,
    /// Round is a draw.
    Draw,
}

/// Choice options in Rock-Paper-Scissors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RpsChoice {
    /// Rock choice.
    Rock = 1,
    /// Paper choice.
    Paper = 2,
    /// Scissors choice.
    Scissors = 3,
}

/// Structure holding the result of a prime number and factor verification test.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrimeVerificationResult {
    /// Indicates whether the number is prime.
    pub is_prime: bool,
    /// Smallest non-trivial factor discovered if the number is composite.
    pub factor_a: Option<i64>,
    /// Complementary factor discovered if the number is composite.
    pub factor_b: Option<i64>,
}

/// Performs addition of two floating-point numbers.
pub fn add(a: f64, b: f64) -> f64 {
    a + b
}

/// Performs subtraction of two floating-point numbers.
pub fn subtract(a: f64, b: f64) -> f64 {
    a - b
}

/// Performs multiplication of two floating-point numbers.
pub fn multiply(a: f64, b: f64) -> f64 {
    a * b
}

/// Performs floating-point division of two numbers.
/// Returns an error if the divisor is zero.
pub fn divide(a: f64, b: f64) -> Result<f64> {
    if b == 0.0 {
        bail!("Division by zero");
    }
    Ok(a / b)
}

/// Performs integer division (quotient without remainder).
/// Returns an error if dividing by zero or overflowing.
pub fn integer_divide(a: i64, b: i64) -> Result<i64> {
    let Some(res) = a.checked_div(b) else {
        bail!("Integer division by zero or overflow");
    };
    Ok(res)
}

/// Computes integer remainder (modulo).
/// Returns an error if dividing by zero or overflowing.
pub fn modulo(a: i64, b: i64) -> Result<i64> {
    let Some(res) = a.checked_rem(b) else {
        bail!("Modulo by zero or overflow");
    };
    Ok(res)
}

/// Computes both quotient and remainder for integer division.
/// Faithfully models legacy Visual Basic modulo formatting logic.
pub fn quotient_and_remainder(a: i64, b: i64) -> Result<(i64, i64)> {
    let q = integer_divide(a, b)?;
    let r = modulo(a, b)?;
    Ok((q, r))
}

/// Formats modulo quotient and remainder into legacy output string (`"Q r R"`).
/// FIXME: In original version, if n2 is 0 it looks like it returns "{ }" as output string. I guess that would need to be implemented in the UI code.
pub fn format_modulo_result(quotient: i64, remainder: i64) -> String {
    format!("{quotient} r{remainder}")
}

/// Performs exponentiation ($base^{exp}$).
pub fn power(base: f64, exp: f64) -> f64 {
    base.powf(exp)
}

/// Evaluates a basic arithmetic operation given an operator string.
pub fn evaluate_basic_op(op: &str, a: f64, b: f64) -> Result<f64> {
    match op {
        "+" => Ok(add(a, b)),
        "-" => Ok(subtract(a, b)),
        "*" | "x" => Ok(multiply(a, b)),
        "/" => divide(a, b),
        "^" => Ok(power(a, b)),
        "\\" => {
            let ai = math::approx_float::f64_to_i64_approx(a)?;
            let bi = math::approx_float::f64_to_i64_approx(b)?;
            let res = integer_divide(ai, bi)?;
            let res_i32 = i32::try_from(res).context(
                "Integer division output out of i32 range for float conversion",
            )?;
            Ok(f64::from(res_i32))
        }
        "Mod" => {
            let ai = math::approx_float::f64_to_i64_approx(a)?;
            let bi = math::approx_float::f64_to_i64_approx(b)?;
            let res = modulo(ai, bi)?;
            let res_i32 = i32::try_from(res).context(
                "Modulo output out of i32 range for float conversion",
            )?;
            Ok(f64::from(res_i32))
        }
        _ => bail!("Unsupported operator: {op}"),
    }
}

/// Calculates the area of a circle given its radius ($\pi \cdot r^2$).
pub fn circle_area(radius: f64) -> f64 {
    CONST_PI * radius * radius
}

/// Calculates the area of a rectangle given base and height ($base \cdot height$).
pub fn rectangle_area(base: f64, height: f64) -> f64 {
    base * height
}

/// Calculates the perimeter of a rectangle ($2 \cdot (base + height)$).
pub fn rectangle_perimeter(base: f64, height: f64) -> f64 {
    2.0 * (base + height)
}

/// Computes the square root of a number. Supports negative inputs by returning
/// an imaginary component representation.
pub fn square_root(x: f64) -> SquareRootResult {
    if x >= 0.0 {
        SquareRootResult::Real(x.sqrt())
    } else {
        SquareRootResult::Imaginary((-x).sqrt())
    }
}

/// Formats square root result into a human-readable display string.
pub fn format_square_root(x: f64) -> String {
    match square_root(x) {
        SquareRootResult::Real(val) => val.to_string(),
        SquareRootResult::Imaginary(val) => format!("{val}i"),
    }
}

/// Converts temperature from Fahrenheit to Celsius.
pub fn fahrenheit_to_celsius(fahrenheit: f64) -> f64 {
    (fahrenheit - 32.0) * 5.0 / 9.0
}

/// Converts temperature from Celsius to Fahrenheit.
pub fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    (celsius * 9.0 / 5.0) + 32.0
}

/// Checks prime status and calculates factors, matching legacy trial division logic.
pub fn verify_prime_and_factors(n: i64) -> PrimeVerificationResult {
    if n <= 1 {
        return PrimeVerificationResult {
            is_prime: false,
            factor_a: None,
            factor_b: None,
        };
    }

    let mut divisor: i64 = 2;
    while divisor < n {
        if n.checked_rem(divisor) == Some(0) {
            let Some(complement) = n.checked_div(divisor) else {
                break;
            };
            return PrimeVerificationResult {
                is_prime: false,
                factor_a: Some(divisor),
                factor_b: Some(complement),
            };
        }
        divisor = divisor.saturating_add(1);
    }

    PrimeVerificationResult {
        is_prime: true,
        factor_a: None,
        factor_b: None,
    }
}

/// Computes a scaled random number value based on a base $[0.0, 1.0)$ input.
pub fn scaled_random(raw_rand: f64, multiplier: f64) -> f64 {
    raw_rand * multiplier
}

/// Generates a triplet of non-repeating random integers within $[min, max]$.
/// Faithfully reproduces the loop validation in sidecar `6r2`.
pub fn generate_unique_random_triplet<R: FnMut() -> i32>(
    mut rand_func: R,
    min: i32,
    max: i32,
) -> Result<[i32; 3]> {
    if min >= max || (max.saturating_sub(min)) < 2 {
        bail!("Range too small to generate 3 unique random integers");
    }

    let mut attempts: u32 = 0;
    while attempts < 1000 {
        attempts = attempts.saturating_add(1);
        let a = rand_func();
        let b = rand_func();
        let c = rand_func();

        if a >= min && a <= max && b >= min && b <= max && c >= min && c <= max
        {
            if a != b && b != c && a != c {
                return Ok([a, b, c]);
            }
        }
    }

    bail!("Failed to generate unique random numbers within attempt limit");
}

/// Evaluates a Rock-Paper-Scissors turn outcome.
pub fn play_rps(user: RpsChoice, computer: RpsChoice) -> RpsOutcome {
    if user == computer {
        return RpsOutcome::Draw;
    }

    match (user, computer) {
        (RpsChoice::Rock, RpsChoice::Scissors)
        | (RpsChoice::Paper, RpsChoice::Rock)
        | (RpsChoice::Scissors, RpsChoice::Paper) => RpsOutcome::Win,
        _ => RpsOutcome::Loss,
    }
}

/// Converts integer code ($1 \dots 3$) to `RpsChoice`.
pub fn rps_choice_from_int(val: i32) -> Result<RpsChoice> {
    match val {
        1 => Ok(RpsChoice::Rock),
        2 => Ok(RpsChoice::Paper),
        3 => Ok(RpsChoice::Scissors),
        _ => bail!("Invalid RPS choice integer: {val}"),
    }
}

/// Standard scale multipliers for random table generation.
pub const RANDOM_SCALES: [f64; 8] = [1.0, 5.0, 10.0, 50.0, 100.0, 500.0, 1000.0, 5000.0];

/// Generates a table of 8 scaled random numbers from a given base raw random value ($0 \dots 1$).
pub fn generate_scaled_random_table(raw: f64) -> [f64; 8] {
    [
        scaled_random(raw, 1.0),
        scaled_random(raw, 5.0),
        scaled_random(raw, 10.0),
        scaled_random(raw, 50.0),
        scaled_random(raw, 100.0),
        scaled_random(raw, 500.0),
        scaled_random(raw, 1000.0),
        scaled_random(raw, 5000.0),
    ]
}

/// Evaluates a math expression string with standard operator precedence.
pub fn evaluate_expression(expr: &str) -> Result<f64> {
    crate::parsing::evaluate_expression(expr)
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
    fn test_basic_arithmetic() {
        assert_eq!(add(2.0, 3.0), 5.0);
        assert_eq!(subtract(10.0, 4.0), 6.0);
        assert_eq!(multiply(3.0, 7.0), 21.0);
        assert_eq!(divide(10.0, 2.0).unwrap(), 5.0);
        divide(10.0, 0.0).unwrap_err();
        assert_eq!(power(2.0, 3.0), 8.0);
    }

    #[crate::ctb_test]
    fn test_integer_divide_and_modulo() {
        assert_eq!(integer_divide(7, 3).unwrap(), 2);
        assert_eq!(modulo(7, 3).unwrap(), 1);
        assert_eq!(quotient_and_remainder(7, 3).unwrap(), (2, 1));
        assert_eq!(format_modulo_result(2, 1), "2 r1");
        integer_divide(5, 0).unwrap_err();
        modulo(5, 0).unwrap_err();
    }

    #[crate::ctb_test]
    fn test_evaluate_basic_op() {
        assert_eq!(evaluate_basic_op("+", 4.0, 5.0).unwrap(), 9.0);
        assert_eq!(evaluate_basic_op("-", 9.0, 2.0).unwrap(), 7.0);
        assert_eq!(evaluate_basic_op("*", 3.0, 4.0).unwrap(), 12.0);
        assert_eq!(evaluate_basic_op("/", 12.0, 3.0).unwrap(), 4.0);
        assert_eq!(evaluate_basic_op("^", 2.0, 4.0).unwrap(), 16.0);
        assert_eq!(evaluate_basic_op("\\", 9.0, 2.0).unwrap(), 4.0);
        assert_eq!(evaluate_basic_op("Mod", 9.0, 2.0).unwrap(), 1.0);
        evaluate_basic_op("invalid", 1.0, 1.0).unwrap_err();
    }

    #[crate::ctb_test]
    fn test_geometry() {
        let area = circle_area(2.0);
        assert!((area - (CONST_PI * 4.0)).abs() < 1e-6);
        assert_eq!(rectangle_area(4.0, 5.0), 20.0);
        assert_eq!(rectangle_perimeter(4.0, 5.0), 18.0);
    }

    #[crate::ctb_test]
    fn test_square_root() {
        assert_eq!(square_root(9.0), SquareRootResult::Real(3.0));
        assert_eq!(square_root(-4.0), SquareRootResult::Imaginary(2.0));
        assert_eq!(format_square_root(9.0), "3");
        assert_eq!(format_square_root(-4.0), "2i");
        assert_eq!(
            square_root(13.0),
            SquareRootResult::Real(13.0f64.sqrt())
        );
    }

    #[crate::ctb_test]
    fn test_temperature() {
        assert_eq!(fahrenheit_to_celsius(32.0), 0.0);
        assert_eq!(fahrenheit_to_celsius(212.0), 100.0);
        assert_eq!(celsius_to_fahrenheit(0.0), 32.0);
        assert_eq!(celsius_to_fahrenheit(100.0), 212.0);
    }

    #[crate::ctb_test]
    fn test_prime_verification() {
        let res7 = verify_prime_and_factors(7);
        assert!(res7.is_prime);
        assert_eq!(res7.factor_a, None);

        let res12 = verify_prime_and_factors(12);
        assert!(!res12.is_prime);
        assert_eq!(res12.factor_a, Some(2));
        assert_eq!(res12.factor_b, Some(6));

        assert!(!verify_prime_and_factors(1).is_prime);
        assert!(!verify_prime_and_factors(0).is_prime);
        assert!(!verify_prime_and_factors(-5).is_prime);
    }

    #[crate::ctb_test]
    fn test_random_generators() {
        assert_eq!(scaled_random(0.5, 10.0), 5.0);

        let mut seq = vec![1, 2, 3].into_iter();
        let res =
            generate_unique_random_triplet(|| seq.next().unwrap_or(0), 0, 5)
                .unwrap();
        assert_eq!(res, [1, 2, 3]);

        let mut fail_seq = vec![1, 1, 1].into_iter();
        let fail_res = generate_unique_random_triplet(
            || fail_seq.next().unwrap_or(1),
            0,
            5,
        );
        fail_res.unwrap_err();
    }

    #[crate::ctb_test]
    fn test_rps() {
        assert_eq!(
            play_rps(RpsChoice::Rock, RpsChoice::Rock),
            RpsOutcome::Draw
        );
        assert_eq!(
            play_rps(RpsChoice::Rock, RpsChoice::Scissors),
            RpsOutcome::Win
        );
        assert_eq!(
            play_rps(RpsChoice::Rock, RpsChoice::Paper),
            RpsOutcome::Loss
        );

        assert_eq!(rps_choice_from_int(1).unwrap(), RpsChoice::Rock);
        assert_eq!(rps_choice_from_int(2).unwrap(), RpsChoice::Paper);
        assert_eq!(rps_choice_from_int(3).unwrap(), RpsChoice::Scissors);
        rps_choice_from_int(4).unwrap_err();
    }

    #[crate::ctb_test]
    fn test_evaluate_expression() {
        assert_eq!(evaluate_expression("2 + 3 * 4").unwrap(), 14.0);
        assert_eq!(evaluate_expression("(2 + 3) * 4").unwrap(), 20.0);
        assert_eq!(evaluate_expression("10 - 4 - 2").unwrap(), 4.0);
        assert_eq!(evaluate_expression("2 ^ 3").unwrap(), 8.0);
        assert_eq!(evaluate_expression("10 \\ 3").unwrap(), 3.0);
        assert_eq!(evaluate_expression("10 Mod 3").unwrap(), 1.0);
        assert_eq!(evaluate_expression("-5 + 10").unwrap(), 5.0);
        evaluate_expression("10 / 0").unwrap_err();
        evaluate_expression("2 + (3 *").unwrap_err();
    }

    #[crate::ctb_test]
    fn test_generate_scaled_random_table() {
        let table = generate_scaled_random_table(0.1);
        assert_eq!(table.len(), 8);
        assert_eq!(table[0], 0.1);
        assert_eq!(table[1], 0.5);
        assert_eq!(table[2], 1.0);
    }
}
