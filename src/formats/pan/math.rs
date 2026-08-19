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

//! Pan math and numeric helpers.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;
use ctb_utilities::math::exact_float::{f64_to_u64, u64_to_f64_exact};
use malachite::Natural;

/// Returns the absolute value of `number`.
pub fn abs(number: f64) -> f64 {
    number.abs()
}

/// Adds two numbers.
/// This will not match proper Pan math, which uses either ints/fixed point (I think both of those might wrap on overflow?) or floats (which I think are 32-bit).
#[expect(
    clippy::arithmetic_side_effects,
    reason = "standard operators are saturating with floats I think"
)]
pub fn add(numerator: f64, denominator: f64) -> f64 {
    numerator + denominator
}

/// Subtracts `denominator` from `numerator`.
#[expect(
    clippy::arithmetic_side_effects,
    reason = "standard operators are saturating with floats I think"
)]
pub fn sub(numerator: f64, denominator: f64) -> f64 {
    numerator - denominator
}

/// Multiplies two numbers.
#[expect(
    clippy::arithmetic_side_effects,
    reason = "standard operators are saturating with floats I think"
)]
pub fn mul(numerator: f64, denominator: f64) -> f64 {
    numerator * denominator
}

/// Divides `numerator` by `denominator`.
#[expect(
    clippy::arithmetic_side_effects,
    reason = "standard operators are saturating with floats I think"
)]
pub fn div(numerator: f64, denominator: f64) -> f64 {
    numerator / denominator
}

/// Divides, returning 0 when `denominator` is zero.
#[expect(
    clippy::arithmetic_side_effects,
    reason = "standard operators are saturating with floats I think"
)]
pub fn divzero(numerator: f64, denominator: f64) -> f64 {
    if denominator == 0.0 {
        0.0
    } else {
        numerator / denominator
    }
}

/// Truncates `number` toward zero.
pub fn fix(number: f64) -> Result<f64> {
    Ok(number.trunc())
}

/// Floors `number` to the nearest integer toward $-\infty$.
pub fn int(number: f64) -> Result<f64> {
    Ok(number.floor())
}

/// In theory, convert to fixed point. At the moment this returns `number`
/// unchanged.
pub fn fixed(number: f64) -> Result<f64> {
    Ok(number)
}

/// In theory, converts to floating point. At the moment this returns `number`
/// unchanged.
pub fn float(number: f64) -> Result<f64> {
    Ok(number)
}

/// Returns the larger of `a` and `b`.
pub fn max(a: f64, b: f64) -> f64 {
    a.max(b)
}

/// Returns the smaller of `a` and `b`.
pub fn min(a: f64, b: f64) -> f64 {
    a.min(b)
}

/// Adds `extra` unless `value` is zero.
pub fn numsandwich(value: f64, extra: f64) -> f64 {
    if value == 0.0 { 0.0 } else { value + extra }
}

/// Rounds `number` to the nearest multiple of `step`.
pub fn round(number: f64, step: f64) -> Result<f64> {
    if !step.is_finite() || step == 0.0 {
        bail!("round(): step must be finite and non-zero");
    }
    if !number.is_finite() {
        return Ok(number);
    }

    let step = step.abs();
    let q = number / step;
    Ok(q.round() * step)
}

#[derive(Debug, Clone)]
/// A small deterministic RNG (not cryptographically secure).
pub struct PanRng {
    state: u64,
}

impl PanRng {
    /// Creates a new RNG with the given `seed`.
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        // xorshift64* (small, fast, deterministic). Not crypto-safe.
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(2685821657736338717)
    }

    /// Returns a deterministic value in the range $[0, 1)$.
    ///
    /// This RNG is not cryptographically secure.
    pub fn rnd(&mut self) -> Result<f64> {
        // Map the top 53 bits to an f64 mantissa.
        let u = self.next_u64() >> 11;
        let denom = 9007199254740992.0_f64; // 2^53
        let val = u64_to_f64_exact(u).context("Error converting u64 to f64")?;
        Ok(val / denom)
    }

    /// Returns a deterministic integer in the inclusive range `start..=end`.
    ///
    /// This RNG is not cryptographically secure.
    pub fn randominteger(&mut self, start: i64, end: i64) -> Result<i64> {
        if start > end {
            bail!("randominteger(): start must be <= end");
        }
        let span = end
            .checked_sub(start)
            .context("randominteger(): range overflow")?;
        let span_u64 = u64::try_from(span)
            .context("randominteger(): range must fit into u64")?;
        let pick = if span_u64 == 0 {
            0
        } else {
            self.next_u64()
                .checked_rem(span_u64.saturating_add(1))
                .context("randominteger(): range division error")?
        };
        let pick_i64 = i64::try_from(pick)
            .context("randominteger(): pick must fit into i64")?;
        start
            .checked_add(pick_i64)
            .context("randominteger(): result overflow")
    }
}

/// Returns `None` when `number` is zero, otherwise `Some(number)`.
pub fn zeroblank(number: f64) -> Option<f64> {
    if number == 0.0 { None } else { Some(number) }
}

fn ensure_in_range_inclusive(
    name: &str,
    x: f64,
    lo: f64,
    hi: f64,
) -> Result<()> {
    if !x.is_finite() {
        bail!("{name}(): input must be finite");
    }
    if x < lo || x > hi {
        bail!("{name}(): input must be between {lo} and {hi}");
    }
    Ok(())
}

/// Returns $\arccos(number)$ for inputs in $[-1, 1]$.
pub fn arccos(number: f64) -> Result<f64> {
    ensure_in_range_inclusive("arccos", number, -1.0, 1.0)?;
    Ok(number.acos())
}

/// Returns $\operatorname{arccosh}(number)$ for inputs $\ge 1$.
pub fn arccosh(number: f64) -> Result<f64> {
    if !number.is_finite() {
        bail!("arccosh(): input must be finite");
    }
    if number < 1.0 {
        bail!("arccosh(): input must be >= 1");
    }
    Ok(number.acosh())
}

/// Returns $\arcsin(number)$ for inputs in $[-1, 1]$.
pub fn arcsin(number: f64) -> Result<f64> {
    ensure_in_range_inclusive("arcsin", number, -1.0, 1.0)?;
    Ok(number.asin())
}

/// Returns $\operatorname{arcsinh}(number)$.
pub fn arcsinh(number: f64) -> Result<f64> {
    if !number.is_finite() {
        bail!("arcsinh(): input must be finite");
    }
    Ok(number.asinh())
}

/// Returns $\arctan(number)$.
pub fn arctan(number: f64) -> Result<f64> {
    if !number.is_finite() {
        bail!("arctan(): input must be finite");
    }
    Ok(number.atan())
}

/// Returns $\operatorname{arctanh}(number)$ for inputs in $(-1, 1)$.
pub fn arctanh(number: f64) -> Result<f64> {
    if !number.is_finite() {
        bail!("arctanh(): input must be finite");
    }
    if number <= -1.0 || number >= 1.0 {
        bail!("arctanh(): input must be between -1 and +1 (exclusive)");
    }
    Ok(number.atanh())
}

/// Returns $\cos(number)$.
pub fn cos(number: f64) -> Result<f64> {
    if !number.is_finite() {
        bail!("cos(): input must be finite");
    }
    Ok(number.cos())
}

/// Returns $\cosh(number)$.
pub fn cosh(number: f64) -> Result<f64> {
    if !number.is_finite() {
        bail!("cosh(): input must be finite");
    }
    Ok(number.cosh())
}

/// Returns $e^{number}$.
pub fn exp(number: f64) -> Result<f64> {
    if !number.is_finite() {
        bail!("exp(): input must be finite");
    }
    Ok(number.exp())
}

/// Returns the natural logarithm of `number`.
pub fn log(number: f64) -> Result<f64> {
    if !number.is_finite() {
        bail!("log(): input must be finite");
    }
    if number <= 0.0 {
        bail!("log(): input must be > 0");
    }
    Ok(number.ln())
}

/// Returns the base-10 logarithm of `number`.
pub fn log10(number: f64) -> Result<f64> {
    if !number.is_finite() {
        bail!("log10(): input must be finite");
    }
    if number <= 0.0 {
        bail!("log10(): input must be > 0");
    }
    Ok(number.log10())
}

/// Returns $\sin(number)$.
pub fn sin(number: f64) -> Result<f64> {
    if !number.is_finite() {
        bail!("sin(): input must be finite");
    }
    Ok(number.sin())
}

/// Returns $\sinh(number)$.
pub fn sinh(number: f64) -> Result<f64> {
    if !number.is_finite() {
        bail!("sinh(): input must be finite");
    }
    Ok(number.sinh())
}

/// Returns $\sqrt{number}$ for non-negative inputs.
pub fn sqr(number: f64) -> Result<f64> {
    if !number.is_finite() {
        bail!("sqr(): input must be finite");
    }
    if number < 0.0 {
        bail!("sqr(): input must be >= 0");
    }
    Ok(number.sqrt())
}

/// Returns $\tan(number)$.
pub fn tan(number: f64) -> Result<f64> {
    if !number.is_finite() {
        bail!("tan(): input must be finite");
    }
    Ok(number.tan())
}

/// Returns $\tanh(number)$.
pub fn tanh(number: f64) -> Result<f64> {
    if !number.is_finite() {
        bail!("tanh(): input must be finite");
    }
    Ok(number.tanh())
}

/// Returns the factorial of `number` as an $f64$.
///
/// The input must be an integer in the range 0..=170.
pub fn fact(number: f64) -> Result<f64> {
    if !number.is_finite() {
        bail!("fact(): input must be finite");
    }
    if number < 0.0 {
        bail!("fact(): input must be >= 0");
    }
    if number.fract() != 0.0 {
        bail!("fact(): input must be an integer");
    }

    let n: u64 = f64_to_u64(number).context("fact(): input out of range")?;
    if n > 170 {
        bail!("fact(): input must be <= 170");
    }

    let mut acc = 1.0_f64;
    let mut i = 2_u64;
    while i <= n {
        acc = <f64 as std::ops::Mul<f64>>::mul(acc, u64_to_f64_exact(i)?);
        i = i
            .checked_add(1)
            .ok_or_else(|| anyhow::anyhow!("i overflowed"))?;
    }
    Ok(acc)
}

/// Returns the factorial of `n` as an exact big integer.
pub fn fact_exact(n: u64) -> Result<Natural> {
    let mut acc = Natural::from(1_u32);
    let mut i = 2_u64;
    while i <= n {
        <Natural as std::ops::MulAssign<Natural>>::mul_assign(
            &mut acc,
            Natural::from(i),
        );
        i = i
            .checked_add(1)
            .ok_or_else(|| anyhow::anyhow!("i overflowed"))?;
    }
    Ok(acc)
}

fn validate_begin(begin: f64) -> Result<f64> {
    if !begin.is_finite() {
        bail!("begin must be finite");
    }
    if begin == 0.0 || begin == 1.0 {
        Ok(begin)
    } else {
        bail!("begin must be 0 or 1");
    }
}

/// Computes the payment for a loan or annuity.
///
/// `begin` must be 0 (end of period) or 1 (begin of period).
pub fn pmt(
    rate: f64,
    periods: f64,
    amount: f64,
    fv: f64,
    begin: f64,
) -> Result<f64> {
    if !rate.is_finite()
        || !periods.is_finite()
        || !amount.is_finite()
        || !fv.is_finite()
    {
        bail!("pmt(): inputs must be finite");
    }
    let begin = validate_begin(begin)?;

    if periods == 0.0 {
        bail!("pmt(): periods must be non-zero");
    }

    if rate == 0.0 {
        return Ok(-(amount + fv) / periods);
    }

    let one_plus_r = 1.0 + rate;
    let pow = one_plus_r.powf(periods);
    let annuity = (pow - 1.0) / rate;
    let adj = 1.0 + rate * begin;
    let denom = adj * annuity;
    if denom == 0.0 {
        bail!("pmt(): invalid parameters (division by zero)");
    }

    Ok(-(fv + amount * pow) / denom)
}

/// Computes the future value for a payment stream.
///
/// `begin` must be 0 (end of period) or 1 (begin of period).
pub fn fv(
    rate: f64,
    periods: f64,
    payment: f64,
    pv: f64,
    begin: f64,
) -> Result<f64> {
    if !rate.is_finite()
        || !periods.is_finite()
        || !payment.is_finite()
        || !pv.is_finite()
    {
        bail!("fv(): inputs must be finite");
    }
    let begin = validate_begin(begin)?;

    if rate == 0.0 {
        return Ok(-(pv + payment * periods));
    }

    let one_plus_r = 1.0 + rate;
    let pow = one_plus_r.powf(periods);
    let annuity = (pow - 1.0) / rate;
    let adj = 1.0 + rate * begin;

    Ok(-(pv * pow + payment * adj * annuity))
}

/// Computes the present value for a payment stream.
///
/// `begin` must be 0 (end of period) or 1 (begin of period).
pub fn pv(
    rate: f64,
    periods: f64,
    payment: f64,
    fv: f64,
    begin: f64,
) -> Result<f64> {
    if !rate.is_finite()
        || !periods.is_finite()
        || !payment.is_finite()
        || !fv.is_finite()
    {
        bail!("pv(): inputs must be finite");
    }
    let begin = validate_begin(begin)?;

    if rate == 0.0 {
        return Ok(-(payment * periods + fv));
    }

    let one_plus_r = 1.0 + rate;
    let pow = one_plus_r.powf(periods);
    let annuity = (pow - 1.0) / rate;
    let adj = 1.0 + rate * begin;

    Ok(-(fv + payment * adj * annuity) / pow)
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
    use ctb_utilities::anyhow::ensure;

    use super::*;

    fn approx_eq(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() <= tol
    }

    #[crate::ctb_test]
    fn basic_math() -> Result<()> {
        ensure!(abs(-3.5) == 3.5);

        ensure!(divzero(10.0, 0.0) == 0.0);
        ensure!(divzero(10.0, 2.0) == 5.0);

        ensure!(fix(4.6)? == 4.0);
        ensure!(fix(-4.6)? == -4.0);

        ensure!(int(4.6)? == 4.0);
        ensure!(int(-4.6)? == -5.0);

        ensure!(max(3.0, 4.0) == 4.0);
        ensure!(min(3.0, 4.0) == 3.0);

        ensure!(numsandwich(0.0, 7.0) == 0.0);
        ensure!(numsandwich(20.0, 7.0) == 27.0);

        ensure!(approx_eq(round(16.0, 12.0)?, 12.0, 1e-12));
        ensure!(approx_eq(round(20.0, 12.0)?, 24.0, 1e-12));

        ensure!(zeroblank(0.0).is_none());
        ensure!(zeroblank(1.0) == Some(1.0));

        Ok(())
    }

    #[crate::ctb_test]
    fn trig_and_logs_domains() -> Result<()> {
        ensure!(arcsin(2.0).is_err());
        ensure!(arccos(-2.0).is_err());
        ensure!(arctanh(1.0).is_err());
        ensure!(arccosh(0.5).is_err());
        ensure!(log(0.0).is_err());
        ensure!(log10(-1.0).is_err());
        ensure!(sqr(-1.0).is_err());

        Ok(())
    }

    #[crate::ctb_test]
    fn trig_and_logs_values() -> Result<()> {
        let pi = core::f64::consts::PI;

        ensure!(approx_eq(sin(pi / 2.0)?, 1.0, 1e-12));
        ensure!(approx_eq(cos(0.0)?, 1.0, 1e-12));
        ensure!(approx_eq(tan(0.0)?, 0.0, 1e-12));

        ensure!(approx_eq(exp(0.0)?, 1.0, 1e-12));
        ensure!(approx_eq(log(exp(2.0)?)?, 2.0, 1e-12));
        ensure!(approx_eq(log10(1000.0)?, 3.0, 1e-12));

        Ok(())
    }

    #[crate::ctb_test]
    fn factorial() -> Result<()> {
        ensure!(fact(0.0)? == 1.0);
        ensure!(fact(5.0)? == 120.0);
        ensure!(fact(171.0).is_err());
        ensure!(fact(-1.0).is_err());
        ensure!(fact(3.5).is_err());

        let n = fact_exact(10)?;
        ensure!(n == Natural::from(3628800_u32));

        Ok(())
    }

    #[crate::ctb_test]
    fn random_is_in_range() -> Result<()> {
        let mut rng = PanRng::new(123456);

        let r = rng.rnd()?;
        ensure!((0.0..1.0).contains(&r));

        for _ in 0..100 {
            let x = rng.randominteger(-3, 3)?;
            ensure!((-3..=3).contains(&x));
        }

        Ok(())
    }

    #[crate::ctb_test]
    fn financial_examples_magnitude() -> Result<()> {
        let rate = 0.135 / 12.0;
        let payment = pmt(rate, 36.0, 20000.0, 0.0, 0.0)?;
        ensure!(approx_eq(payment.abs(), 678.71, 0.02));

        let f1 = fv(0.09, 10.0, -500.0, 0.0, 1.0)?;
        ensure!(approx_eq(f1, 8280.15, 0.02));

        let f2 = fv(0.09, 10.0, -500.0, -2000.0, 1.0)?;
        ensure!(approx_eq(f2, 13014.87, 0.02));

        let p = pv(0.1, 3.0, 1000.0, 0.0, 0.0)?;
        ensure!(approx_eq(p.abs(), 2486.0, 1.0));

        Ok(())
    }
}
