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

//! JSON-RPC dispatcher for Calculator math functions.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use serde_json::Value;

/// Dispatches a Calculator RPC method call.
pub async fn handle_calculator_call(
    func: &str,
    args: &[Value],
) -> anyhow::Result<Value> {
    use ctb_formats_math::calculator_classic::*;

    match func {
        "evaluateExpression" => {
            let expr = args
                .first()
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: expression"))?;
            let res = evaluate_expression(expr)?;
            Ok(serde_json::to_value(res)?)
        }
        "getRandomScaleTable" => {
            // Reason for fallback: default to uniform random scale in 0.0..1.0 if raw argument omitted.
            let raw = args.first().and_then(Value::as_f64).unwrap_or_else(|| {
                use rand::Rng;
                rand::rng().random_range(0.0..1.0)
            });
            let table = generate_scaled_random_table(raw);
            Ok(serde_json::to_value(table)?)
        }
        "evaluateBasicOp" => {
            let op = args
                .first()
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: operator"))?;
            let a = args
                .get(1)
                .and_then(Value::as_f64)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 1: operand a"))?;
            let b = args
                .get(2)
                .and_then(Value::as_f64)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 2: operand b"))?;
            let res = evaluate_basic_op(op, a, b)?;
            Ok(serde_json::to_value(res)?)
        }
        "add" => {
            let a = args
                .first()
                .and_then(Value::as_f64)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: a"))?;
            let b = args
                .get(1)
                .and_then(Value::as_f64)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 1: b"))?;
            Ok(serde_json::to_value(add(a, b))?)
        }
        "subtract" => {
            let a = args
                .first()
                .and_then(Value::as_f64)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: a"))?;
            let b = args
                .get(1)
                .and_then(Value::as_f64)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 1: b"))?;
            Ok(serde_json::to_value(subtract(a, b))?)
        }
        "multiply" => {
            let a = args
                .first()
                .and_then(Value::as_f64)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: a"))?;
            let b = args
                .get(1)
                .and_then(Value::as_f64)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 1: b"))?;
            Ok(serde_json::to_value(multiply(a, b))?)
        }
        "divide" => {
            let a = args
                .first()
                .and_then(Value::as_f64)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: a"))?;
            let b = args
                .get(1)
                .and_then(Value::as_f64)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 1: b"))?;
            Ok(serde_json::to_value(divide(a, b)?)?)
        }
        "power" => {
            let a = args
                .first()
                .and_then(Value::as_f64)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: base"))?;
            let b = args
                .get(1)
                .and_then(Value::as_f64)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 1: exponent"))?;
            Ok(serde_json::to_value(power(a, b))?)
        }
        "integerDivide" => {
            let a = args
                .first()
                .and_then(Value::as_i64)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: a"))?;
            let b = args
                .get(1)
                .and_then(Value::as_i64)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 1: b"))?;
            Ok(serde_json::to_value(integer_divide(a, b)?)?)
        }
        "modulo" => {
            let a = args
                .first()
                .and_then(Value::as_i64)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: a"))?;
            let b = args
                .get(1)
                .and_then(Value::as_i64)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 1: b"))?;
            Ok(serde_json::to_value(modulo(a, b)?)?)
        }
        "circleArea" => {
            let r = args
                .first()
                .and_then(Value::as_f64)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: radius"))?;
            Ok(serde_json::to_value(circle_area(r))?)
        }
        "rectangleArea" => {
            let b = args
                .first()
                .and_then(Value::as_f64)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: base"))?;
            let h = args
                .get(1)
                .and_then(Value::as_f64)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 1: height"))?;
            Ok(serde_json::to_value(rectangle_area(b, h))?)
        }
        "rectanglePerimeter" => {
            let b = args
                .first()
                .and_then(Value::as_f64)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: base"))?;
            let h = args
                .get(1)
                .and_then(Value::as_f64)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 1: height"))?;
            Ok(serde_json::to_value(rectangle_perimeter(b, h))?)
        }
        "squareRoot" => {
            let x = args
                .first()
                .and_then(Value::as_f64)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: x"))?;
            let formatted = format_square_root(x);
            Ok(serde_json::to_value(formatted)?)
        }
        "fahrenheitToCelsius" => {
            let f = args
                .first()
                .and_then(Value::as_f64)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: fahrenheit"))?;
            Ok(serde_json::to_value(fahrenheit_to_celsius(f))?)
        }
        "celsiusToFahrenheit" => {
            let c = args
                .first()
                .and_then(Value::as_f64)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: celsius"))?;
            Ok(serde_json::to_value(celsius_to_fahrenheit(c))?)
        }
        "verifyPrimeAndFactors" => {
            let n = args
                .first()
                .and_then(Value::as_i64)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: n"))?;
            let result = verify_prime_and_factors(n);
            Ok(serde_json::json!({
                "is_prime": result.is_prime,
                "factor_a": result.factor_a,
                "factor_b": result.factor_b
            }))
        }
        "scaledRandom" => {
            let raw = args
                .first()
                .and_then(Value::as_f64)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: raw_random"))?;
            let mult = args
                .get(1)
                .and_then(Value::as_f64)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 1: multiplier"))?;
            Ok(serde_json::to_value(scaled_random(raw, mult))?)
        }
        "generateUniqueRandomTriplet" => {
            let min = args
                .first()
                .and_then(Value::as_i64)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: min"))?;
            let max = args
                .get(1)
                .and_then(Value::as_i64)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 1: max"))?;
            let min_i32 = i32::try_from(min)?;
            let max_i32 = i32::try_from(max)?;
            let mut rng = rand::rng();
            use rand::Rng;
            let triplet = generate_unique_random_triplet(
                || rng.random_range(min_i32..=max_i32),
                min_i32,
                max_i32,
            )?;
            Ok(serde_json::to_value(triplet)?)
        }
        "playRps" => {
            let user_int = args
                .first()
                .and_then(Value::as_i64)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: user_choice"))?;
            let user_i32 = i32::try_from(user_int)?;
            let user_choice = rps_choice_from_int(user_i32)?;
            let mut rng = rand::rng();
            use rand::Rng;
            let comp_int: i32 = rng.random_range(1..=3);
            let comp_choice = rps_choice_from_int(comp_int)?;
            let outcome = play_rps(user_choice, comp_choice);
            let outcome_str = match outcome {
                RpsOutcome::Win => "Win",
                RpsOutcome::Loss => "Loss",
                RpsOutcome::Draw => "Draw",
            };
            Ok(serde_json::json!({
                "userChoice": user_int,
                "computerChoice": comp_int,
                "outcome": outcome_str
            }))
        }
        "getConstants" => {
            let radical13_val = match square_root(13.0) {
                SquareRootResult::Real(v) => v,
                SquareRootResult::Imaginary(v) => v,
            };
            Ok(serde_json::json!({
                "pi": CONST_PI,
                "e": CONST_E,
                "radical13": radical13_val
            }))
        }
        _ => anyhow::bail!("Unknown calculator function: {func}"),
    }
}
