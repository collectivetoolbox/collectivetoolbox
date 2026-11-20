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

//! Safe WebAssembly numeric, bit-manipulation, conversion, and typed memory
//! operations for generated native substitutes (zero `as` casts, zero panics).

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::SubstituteVm;
use crate::WasmTrap;

const F32_CANONICAL_NAN: f32 = f32::from_bits(0x7FC0_0000);
const F64_CANONICAL_NAN: f64 = f64::from_bits(0x7FF8_0000_0000_0000);

#[inline(always)]
fn as_u32(v: i32) -> u32 {
    u32::from_le_bytes(v.to_le_bytes())
}

#[inline(always)]
fn as_i32(v: u32) -> i32 {
    i32::from_le_bytes(v.to_le_bytes())
}

#[inline(always)]
fn as_u64(v: i64) -> u64 {
    u64::from_le_bytes(v.to_le_bytes())
}

#[inline(always)]
fn as_i64(v: u64) -> i64 {
    i64::from_le_bytes(v.to_le_bytes())
}

// ============================================================================
// i32 operations
// ============================================================================

/// Executes `i32.eqz`.
#[inline(always)]
pub fn i32_eqz(a: i32) -> i32 {
    i32::from(a == 0)
}

/// Executes `i32.eq`.
#[inline(always)]
pub fn i32_eq(a: i32, b: i32) -> i32 {
    i32::from(a == b)
}

/// Executes `i32.ne`.
#[inline(always)]
pub fn i32_ne(a: i32, b: i32) -> i32 {
    i32::from(a != b)
}

/// Executes `i32.lt_s`.
#[inline(always)]
pub fn i32_lt_s(a: i32, b: i32) -> i32 {
    i32::from(a < b)
}

/// Executes `i32.lt_u`.
#[inline(always)]
pub fn i32_lt_u(a: i32, b: i32) -> i32 {
    i32::from(as_u32(a) < as_u32(b))
}

/// Executes `i32.gt_s`.
#[inline(always)]
pub fn i32_gt_s(a: i32, b: i32) -> i32 {
    i32::from(a > b)
}

/// Executes `i32.gt_u`.
#[inline(always)]
pub fn i32_gt_u(a: i32, b: i32) -> i32 {
    i32::from(as_u32(a) > as_u32(b))
}

/// Executes `i32.le_s`.
#[inline(always)]
pub fn i32_le_s(a: i32, b: i32) -> i32 {
    i32::from(a <= b)
}

/// Executes `i32.le_u`.
#[inline(always)]
pub fn i32_le_u(a: i32, b: i32) -> i32 {
    i32::from(as_u32(a) <= as_u32(b))
}

/// Executes `i32.ge_s`.
#[inline(always)]
pub fn i32_ge_s(a: i32, b: i32) -> i32 {
    i32::from(a >= b)
}

/// Executes `i32.ge_u`.
#[inline(always)]
pub fn i32_ge_u(a: i32, b: i32) -> i32 {
    i32::from(as_u32(a) >= as_u32(b))
}

/// Executes `i32.clz`.
#[inline(always)]
pub fn i32_clz(a: i32) -> i32 {
    as_i32(a.leading_zeros())
}

/// Executes `i32.ctz`.
#[inline(always)]
pub fn i32_ctz(a: i32) -> i32 {
    as_i32(a.trailing_zeros())
}

/// Executes `i32.popcnt`.
#[inline(always)]
pub fn i32_popcnt(a: i32) -> i32 {
    as_i32(a.count_ones())
}

/// Executes `i32.add`.
#[inline(always)]
pub fn i32_add(a: i32, b: i32) -> i32 {
    a.wrapping_add(b)
}

/// Executes `i32.sub`.
#[inline(always)]
pub fn i32_sub(a: i32, b: i32) -> i32 {
    a.wrapping_sub(b)
}

/// Executes `i32.mul`.
#[inline(always)]
pub fn i32_mul(a: i32, b: i32) -> i32 {
    a.wrapping_mul(b)
}

/// Executes `i32.div_s`.
#[inline(always)]
pub fn i32_div_s(a: i32, b: i32) -> std::result::Result<i32, WasmTrap> {
    if b == 0 {
        return Err(WasmTrap::IntegerDivideByZero);
    }
    a.checked_div(b).ok_or(WasmTrap::IntegerOverflow)
}

/// Executes `i32.div_u`.
#[inline(always)]
pub fn i32_div_u(a: i32, b: i32) -> std::result::Result<i32, WasmTrap> {
    as_u32(a)
        .checked_div(as_u32(b))
        .map(as_i32)
        .ok_or(WasmTrap::IntegerDivideByZero)
}

/// Executes `i32.rem_s` (note `i32::MIN % -1 == 0` in WebAssembly).
#[inline(always)]
pub fn i32_rem_s(a: i32, b: i32) -> std::result::Result<i32, WasmTrap> {
    if b == 0 {
        return Err(WasmTrap::IntegerDivideByZero);
    }
    if b == -1 {
        return Ok(0);
    }
    a.checked_rem(b).ok_or(WasmTrap::IntegerOverflow)
}

/// Executes `i32.rem_u`.
#[inline(always)]
pub fn i32_rem_u(a: i32, b: i32) -> std::result::Result<i32, WasmTrap> {
    as_u32(a)
        .checked_rem(as_u32(b))
        .map(as_i32)
        .ok_or(WasmTrap::IntegerDivideByZero)
}

/// Executes `i32.and`.
#[inline(always)]
pub fn i32_and(a: i32, b: i32) -> i32 {
    a & b
}

/// Executes `i32.or`.
#[inline(always)]
pub fn i32_or(a: i32, b: i32) -> i32 {
    a | b
}

/// Executes `i32.xor`.
#[inline(always)]
pub fn i32_xor(a: i32, b: i32) -> i32 {
    a ^ b
}

/// Executes `i32.shl`.
#[inline(always)]
pub fn i32_shl(a: i32, b: i32) -> i32 {
    a.wrapping_shl(as_u32(b) & 31)
}

/// Executes `i32.shr_s`.
#[inline(always)]
pub fn i32_shr_s(a: i32, b: i32) -> i32 {
    a.wrapping_shr(as_u32(b) & 31)
}

/// Executes `i32.shr_u`.
#[inline(always)]
pub fn i32_shr_u(a: i32, b: i32) -> i32 {
    as_i32(as_u32(a).wrapping_shr(as_u32(b) & 31))
}

/// Executes `i32.rotl`.
#[inline(always)]
pub fn i32_rotl(a: i32, b: i32) -> i32 {
    as_i32(as_u32(a).rotate_left(as_u32(b) & 31))
}

/// Executes `i32.rotr`.
#[inline(always)]
pub fn i32_rotr(a: i32, b: i32) -> i32 {
    as_i32(as_u32(a).rotate_right(as_u32(b) & 31))
}

/// Executes `i32.wrap_i64`.
#[inline(always)]
pub fn i32_wrap_i64(a: i64) -> i32 {
    let [b0, b1, b2, b3, ..] = a.to_le_bytes();
    i32::from_le_bytes([b0, b1, b2, b3])
}

/// Executes `i32.extend8_s`.
#[inline(always)]
pub fn i32_extend8_s(a: i32) -> i32 {
    let [b0, ..] = a.to_le_bytes();
    i32::from(i8::from_le_bytes([b0]))
}

/// Executes `i32.extend16_s`.
#[inline(always)]
pub fn i32_extend16_s(a: i32) -> i32 {
    let [b0, b1, ..] = a.to_le_bytes();
    i32::from(i16::from_le_bytes([b0, b1]))
}

// ============================================================================
// i64 operations
// ============================================================================

/// Executes `i64.eqz`.
#[inline(always)]
pub fn i64_eqz(a: i64) -> i32 {
    i32::from(a == 0)
}

/// Executes `i64.eq`.
#[inline(always)]
pub fn i64_eq(a: i64, b: i64) -> i32 {
    i32::from(a == b)
}

/// Executes `i64.ne`.
#[inline(always)]
pub fn i64_ne(a: i64, b: i64) -> i32 {
    i32::from(a != b)
}

/// Executes `i64.lt_s`.
#[inline(always)]
pub fn i64_lt_s(a: i64, b: i64) -> i32 {
    i32::from(a < b)
}

/// Executes `i64.lt_u`.
#[inline(always)]
pub fn i64_lt_u(a: i64, b: i64) -> i32 {
    i32::from(as_u64(a) < as_u64(b))
}

/// Executes `i64.gt_s`.
#[inline(always)]
pub fn i64_gt_s(a: i64, b: i64) -> i32 {
    i32::from(a > b)
}

/// Executes `i64.gt_u`.
#[inline(always)]
pub fn i64_gt_u(a: i64, b: i64) -> i32 {
    i32::from(as_u64(a) > as_u64(b))
}

/// Executes `i64.le_s`.
#[inline(always)]
pub fn i64_le_s(a: i64, b: i64) -> i32 {
    i32::from(a <= b)
}

/// Executes `i64.le_u`.
#[inline(always)]
pub fn i64_le_u(a: i64, b: i64) -> i32 {
    i32::from(as_u64(a) <= as_u64(b))
}

/// Executes `i64.ge_s`.
#[inline(always)]
pub fn i64_ge_s(a: i64, b: i64) -> i32 {
    i32::from(a >= b)
}

/// Executes `i64.ge_u`.
#[inline(always)]
pub fn i64_ge_u(a: i64, b: i64) -> i32 {
    i32::from(as_u64(a) >= as_u64(b))
}

/// Executes `i64.clz`.
#[inline(always)]
pub fn i64_clz(a: i64) -> i64 {
    i64::from(a.leading_zeros())
}

/// Executes `i64.ctz`.
#[inline(always)]
pub fn i64_ctz(a: i64) -> i64 {
    i64::from(a.trailing_zeros())
}

/// Executes `i64.popcnt`.
#[inline(always)]
pub fn i64_popcnt(a: i64) -> i64 {
    i64::from(a.count_ones())
}

/// Executes `i64.add`.
#[inline(always)]
pub fn i64_add(a: i64, b: i64) -> i64 {
    a.wrapping_add(b)
}

/// Executes `i64.sub`.
#[inline(always)]
pub fn i64_sub(a: i64, b: i64) -> i64 {
    a.wrapping_sub(b)
}

/// Executes `i64.mul`.
#[inline(always)]
pub fn i64_mul(a: i64, b: i64) -> i64 {
    a.wrapping_mul(b)
}

/// Executes `i64.div_s`.
#[inline(always)]
pub fn i64_div_s(a: i64, b: i64) -> std::result::Result<i64, WasmTrap> {
    if b == 0 {
        return Err(WasmTrap::IntegerDivideByZero);
    }
    a.checked_div(b).ok_or(WasmTrap::IntegerOverflow)
}

/// Executes `i64.div_u`.
#[inline(always)]
pub fn i64_div_u(a: i64, b: i64) -> std::result::Result<i64, WasmTrap> {
    as_u64(a)
        .checked_div(as_u64(b))
        .map(as_i64)
        .ok_or(WasmTrap::IntegerDivideByZero)
}

/// Executes `i64.rem_s` (note `i64::MIN % -1 == 0` in WebAssembly).
#[inline(always)]
pub fn i64_rem_s(a: i64, b: i64) -> std::result::Result<i64, WasmTrap> {
    if b == 0 {
        return Err(WasmTrap::IntegerDivideByZero);
    }
    if b == -1 {
        return Ok(0);
    }
    a.checked_rem(b).ok_or(WasmTrap::IntegerOverflow)
}

/// Executes `i64.rem_u`.
#[inline(always)]
pub fn i64_rem_u(a: i64, b: i64) -> std::result::Result<i64, WasmTrap> {
    as_u64(a)
        .checked_rem(as_u64(b))
        .map(as_i64)
        .ok_or(WasmTrap::IntegerDivideByZero)
}

/// Executes `i64.and`.
#[inline(always)]
pub fn i64_and(a: i64, b: i64) -> i64 {
    a & b
}

/// Executes `i64.or`.
#[inline(always)]
pub fn i64_or(a: i64, b: i64) -> i64 {
    a | b
}

/// Executes `i64.xor`.
#[inline(always)]
pub fn i64_xor(a: i64, b: i64) -> i64 {
    a ^ b
}

#[inline(always)]
fn shift64_amt(b: i64) -> u32 {
    let [b0, ..] = b.to_le_bytes();
    u32::from(b0 & 63)
}

/// Executes `i64.shl`.
#[inline(always)]
pub fn i64_shl(a: i64, b: i64) -> i64 {
    a.wrapping_shl(shift64_amt(b))
}

/// Executes `i64.shr_s`.
#[inline(always)]
pub fn i64_shr_s(a: i64, b: i64) -> i64 {
    a.wrapping_shr(shift64_amt(b))
}

/// Executes `i64.shr_u`.
#[inline(always)]
pub fn i64_shr_u(a: i64, b: i64) -> i64 {
    as_i64(as_u64(a).wrapping_shr(shift64_amt(b)))
}

/// Executes `i64.rotl`.
#[inline(always)]
pub fn i64_rotl(a: i64, b: i64) -> i64 {
    as_i64(as_u64(a).rotate_left(shift64_amt(b)))
}

/// Executes `i64.rotr`.
#[inline(always)]
pub fn i64_rotr(a: i64, b: i64) -> i64 {
    as_i64(as_u64(a).rotate_right(shift64_amt(b)))
}

/// Executes `i64.extend_i32_s`.
#[inline(always)]
pub fn i64_extend_i32_s(a: i32) -> i64 {
    i64::from(a)
}

/// Executes `i64.extend_i32_u`.
#[inline(always)]
pub fn i64_extend_i32_u(a: i32) -> i64 {
    i64::from(as_u32(a))
}

/// Executes `i64.extend8_s`.
#[inline(always)]
pub fn i64_extend8_s(a: i64) -> i64 {
    let [b0, ..] = a.to_le_bytes();
    i64::from(i8::from_le_bytes([b0]))
}

/// Executes `i64.extend16_s`.
#[inline(always)]
pub fn i64_extend16_s(a: i64) -> i64 {
    let [b0, b1, ..] = a.to_le_bytes();
    i64::from(i16::from_le_bytes([b0, b1]))
}

/// Executes `i64.extend32_s`.
#[inline(always)]
pub fn i64_extend32_s(a: i64) -> i64 {
    i64::from(i32_wrap_i64(a))
}

// ============================================================================
// f32 operations
// ============================================================================

/// Executes `f32.eq`.
#[inline(always)]
pub fn f32_eq(a: f32, b: f32) -> i32 {
    i32::from(a == b)
}

/// Executes `f32.ne`.
#[inline(always)]
pub fn f32_ne(a: f32, b: f32) -> i32 {
    i32::from(a != b)
}

/// Executes `f32.lt`.
#[inline(always)]
pub fn f32_lt(a: f32, b: f32) -> i32 {
    i32::from(a < b)
}

/// Executes `f32.gt`.
#[inline(always)]
pub fn f32_gt(a: f32, b: f32) -> i32 {
    i32::from(a > b)
}

/// Executes `f32.le`.
#[inline(always)]
pub fn f32_le(a: f32, b: f32) -> i32 {
    i32::from(a <= b)
}

/// Executes `f32.ge`.
#[inline(always)]
pub fn f32_ge(a: f32, b: f32) -> i32 {
    i32::from(a >= b)
}

/// Executes `f32.abs`.
#[inline(always)]
pub fn f32_abs(a: f32) -> f32 {
    a.abs()
}

/// Executes `f32.neg`.
#[inline(always)]
pub fn f32_neg(a: f32) -> f32 {
    -a
}

/// Executes `f32.ceil`.
#[inline(always)]
pub fn f32_ceil(a: f32) -> f32 {
    a.ceil()
}

/// Executes `f32.floor`.
#[inline(always)]
pub fn f32_floor(a: f32) -> f32 {
    a.floor()
}

/// Executes `f32.trunc`.
#[inline(always)]
pub fn f32_trunc(a: f32) -> f32 {
    a.trunc()
}

/// Executes `f32.nearest` (round half to even).
#[inline(always)]
pub fn f32_nearest(a: f32) -> f32 {
    a.round_ties_even()
}

/// Executes `f32.sqrt`.
#[inline(always)]
pub fn f32_sqrt(a: f32) -> f32 {
    a.sqrt()
}

/// Executes `f32.add`.
#[inline(always)]
#[allow(
    clippy::arithmetic_side_effects,
    reason = "IEEE-754 floating-point arithmetic is total and never panics"
)]
pub fn f32_add(a: f32, b: f32) -> f32 {
    a + b
}

/// Executes `f32.sub`.
#[inline(always)]
#[allow(
    clippy::arithmetic_side_effects,
    reason = "IEEE-754 floating-point arithmetic is total and never panics"
)]
pub fn f32_sub(a: f32, b: f32) -> f32 {
    a - b
}

/// Executes `f32.mul`.
#[inline(always)]
#[allow(
    clippy::arithmetic_side_effects,
    reason = "IEEE-754 floating-point arithmetic is total and never panics"
)]
pub fn f32_mul(a: f32, b: f32) -> f32 {
    a * b
}

/// Executes `f32.div`.
#[inline(always)]
#[allow(
    clippy::arithmetic_side_effects,
    reason = "IEEE-754 floating-point division produces Inf/NaN and never panics"
)]
pub fn f32_div(a: f32, b: f32) -> f32 {
    a / b
}

/// Executes `f32.min` with WebAssembly NaN propagation and `-0.0 < +0.0`.
#[inline(always)]
pub fn f32_min(a: f32, b: f32) -> f32 {
    if a.is_nan() || b.is_nan() {
        return F32_CANONICAL_NAN;
    }
    if a == 0.0 && b == 0.0 {
        return if a.is_sign_negative() { a } else { b };
    }
    if a < b { a } else { b }
}

/// Executes `f32.max` with WebAssembly NaN propagation and `+0.0 > -0.0`.
#[inline(always)]
pub fn f32_max(a: f32, b: f32) -> f32 {
    if a.is_nan() || b.is_nan() {
        return F32_CANONICAL_NAN;
    }
    if a == 0.0 && b == 0.0 {
        return if a.is_sign_positive() { a } else { b };
    }
    if a > b { a } else { b }
}

/// Executes `f32.copysign`.
#[inline(always)]
pub fn f32_copysign(a: f32, b: f32) -> f32 {
    a.copysign(b)
}

// ============================================================================
// f64 operations
// ============================================================================

/// Executes `f64.eq`.
#[inline(always)]
pub fn f64_eq(a: f64, b: f64) -> i32 {
    i32::from(a == b)
}

/// Executes `f64.ne`.
#[inline(always)]
pub fn f64_ne(a: f64, b: f64) -> i32 {
    i32::from(a != b)
}

/// Executes `f64.lt`.
#[inline(always)]
pub fn f64_lt(a: f64, b: f64) -> i32 {
    i32::from(a < b)
}

/// Executes `f64.gt`.
#[inline(always)]
pub fn f64_gt(a: f64, b: f64) -> i32 {
    i32::from(a > b)
}

/// Executes `f64.le`.
#[inline(always)]
pub fn f64_le(a: f64, b: f64) -> i32 {
    i32::from(a <= b)
}

/// Executes `f64.ge`.
#[inline(always)]
pub fn f64_ge(a: f64, b: f64) -> i32 {
    i32::from(a >= b)
}

/// Executes `f64.abs`.
#[inline(always)]
pub fn f64_abs(a: f64) -> f64 {
    a.abs()
}

/// Executes `f64.neg`.
#[inline(always)]
pub fn f64_neg(a: f64) -> f64 {
    -a
}

/// Executes `f64.ceil`.
#[inline(always)]
pub fn f64_ceil(a: f64) -> f64 {
    a.ceil()
}

/// Executes `f64.floor`.
#[inline(always)]
pub fn f64_floor(a: f64) -> f64 {
    a.floor()
}

/// Executes `f64.trunc`.
#[inline(always)]
pub fn f64_trunc(a: f64) -> f64 {
    a.trunc()
}

/// Executes `f64.nearest` (round half to even).
#[inline(always)]
pub fn f64_nearest(a: f64) -> f64 {
    a.round_ties_even()
}

/// Executes `f64.sqrt`.
#[inline(always)]
pub fn f64_sqrt(a: f64) -> f64 {
    a.sqrt()
}

/// Executes `f64.add`.
#[inline(always)]
#[allow(
    clippy::arithmetic_side_effects,
    reason = "IEEE-754 floating-point arithmetic is total and never panics"
)]
pub fn f64_add(a: f64, b: f64) -> f64 {
    a + b
}

/// Executes `f64.sub`.
#[inline(always)]
#[allow(
    clippy::arithmetic_side_effects,
    reason = "IEEE-754 floating-point arithmetic is total and never panics"
)]
pub fn f64_sub(a: f64, b: f64) -> f64 {
    a - b
}

/// Executes `f64.mul`.
#[inline(always)]
#[allow(
    clippy::arithmetic_side_effects,
    reason = "IEEE-754 floating-point arithmetic is total and never panics"
)]
pub fn f64_mul(a: f64, b: f64) -> f64 {
    a * b
}

/// Executes `f64.div`.
#[inline(always)]
#[allow(
    clippy::arithmetic_side_effects,
    reason = "IEEE-754 floating-point division produces Inf/NaN and never panics"
)]
pub fn f64_div(a: f64, b: f64) -> f64 {
    a / b
}

/// Executes `f64.min` with WebAssembly NaN propagation and `-0.0 < +0.0`.
#[inline(always)]
pub fn f64_min(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        return F64_CANONICAL_NAN;
    }
    if a == 0.0 && b == 0.0 {
        return if a.is_sign_negative() { a } else { b };
    }
    if a < b { a } else { b }
}

/// Executes `f64.max` with WebAssembly NaN propagation and `+0.0 > -0.0`.
#[inline(always)]
pub fn f64_max(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        return F64_CANONICAL_NAN;
    }
    if a == 0.0 && b == 0.0 {
        return if a.is_sign_positive() { a } else { b };
    }
    if a > b { a } else { b }
}

/// Executes `f64.copysign`.
#[inline(always)]
pub fn f64_copysign(a: f64, b: f64) -> f64 {
    a.copysign(b)
}

// ============================================================================
// Pure-integer IEEE-754 float <-> integer conversions and reinterprets
// ============================================================================

enum TruncParts {
    Nan,
    InU64Range { neg: bool, mag: u64 },
    OutOfU64Range { neg: bool },
}

#[inline(always)]
fn decompose_f64_trunc(x: f64) -> TruncParts {
    if x.is_nan() {
        return TruncParts::Nan;
    }
    let bits = x.to_bits();
    let neg = (bits >> 63) != 0;
    let [.., b6, b7] = bits.to_le_bytes();
    let exp_bits = u32::from((u16::from_le_bytes([b6, b7]) >> 4) & 0x7FF);
    if exp_bits < 1023 {
        return TruncParts::InU64Range { neg, mag: 0 };
    }
    let exp = exp_bits.saturating_sub(1023);
    if exp >= 64 {
        return TruncParts::OutOfU64Range { neg };
    }
    let frac = bits & 0x000F_FFFF_FFFF_FFFF;
    let sig = (1u64 << 52) | frac;
    let mag = if exp < 52 {
        sig.wrapping_shr(52u32.saturating_sub(exp))
    } else {
        sig.wrapping_shl(exp.saturating_sub(52))
    };
    TruncParts::InU64Range { neg, mag }
}

fn u64_to_f64(mag: u64, neg: bool) -> f64 {
    let sign_bit = if neg { 1u64 << 63 } else { 0 };
    if mag == 0 {
        return f64::from_bits(sign_bit);
    }
    let bit = 63u32.saturating_sub(mag.leading_zeros());
    let (mant, carry) = if bit <= 52 {
        (mag.wrapping_shl(52u32.saturating_sub(bit)), 0u64)
    } else {
        let shift = bit.saturating_sub(52);
        let kept = mag.wrapping_shr(shift);
        let rem_mask = (1u64.wrapping_shl(shift)).wrapping_sub(1);
        let rem = mag & rem_mask;
        let half = 1u64.wrapping_shl(shift.saturating_sub(1));
        let round_up = rem > half || (rem == half && (kept & 1) == 1);
        let mut rounded = kept.wrapping_add(u64::from(round_up));
        if rounded == (1u64 << 53) {
            rounded >>= 1;
            (rounded, 1u64)
        } else {
            (rounded, 0u64)
        }
    };
    let exp = u64::from(1023u32.saturating_add(bit)).wrapping_add(carry);
    let frac = mant & 0x000F_FFFF_FFFF_FFFF;
    f64::from_bits(sign_bit | (exp << 52) | frac)
}

fn u64_to_f32(mag: u64, neg: bool) -> f32 {
    let sign_bit = if neg { 0x8000_0000u32 } else { 0u32 };
    if mag == 0 {
        return f32::from_bits(sign_bit);
    }
    let bit = 63u32.saturating_sub(mag.leading_zeros());
    let (mant, carry) = if bit <= 23 {
        let shifted = mag.wrapping_shl(23u32.saturating_sub(bit));
        let [b0, b1, b2, b3, ..] = shifted.to_le_bytes();
        (u32::from_le_bytes([b0, b1, b2, b3]), 0u32)
    } else {
        let shift = bit.saturating_sub(23);
        let kept = mag.wrapping_shr(shift);
        let rem_mask = (1u64.wrapping_shl(shift)).wrapping_sub(1);
        let rem = mag & rem_mask;
        let half = 1u64.wrapping_shl(shift.saturating_sub(1));
        let round_up = rem > half || (rem == half && (kept & 1) == 1);
        let rounded = kept.wrapping_add(u64::from(round_up));
        let [b0, b1, b2, b3, ..] = rounded.to_le_bytes();
        let mut r32 = u32::from_le_bytes([b0, b1, b2, b3]);
        if r32 == (1u32 << 24) {
            r32 >>= 1;
            (r32, 1u32)
        } else {
            (r32, 0u32)
        }
    };
    let exp = 127u32.saturating_add(bit).saturating_add(carry);
    let frac = mant & 0x007F_FFFF;
    f32::from_bits(sign_bit | (exp << 23) | frac)
}

/// Executes `f32.demote_f64`.
#[inline]
pub fn f32_demote_f64(x: f64) -> f32 {
    let bits = x.to_bits();
    let neg = (bits >> 63) != 0;
    let sign32 = if neg { 0x8000_0000u32 } else { 0u32 };
    let [.., b6, b7] = bits.to_le_bytes();
    let exp64 = u32::from((u16::from_le_bytes([b6, b7]) >> 4) & 0x7FF);
    let frac64 = bits & 0x000F_FFFF_FFFF_FFFF;
    if exp64 == 0x7FF {
        if frac64 == 0 {
            return f32::from_bits(sign32 | 0x7F80_0000);
        }
        let payload_bits = frac64 >> 29;
        let [p0, p1, p2, p3, ..] = payload_bits.to_le_bytes();
        let payload = u32::from_le_bytes([p0, p1, p2, p3]) & 0x003F_FFFF;
        return f32::from_bits(sign32 | 0x7FC0_0000 | payload);
    }
    if exp64 == 0 {
        return f32::from_bits(sign32);
    }
    if exp64 >= 1151 {
        return f32::from_bits(sign32 | 0x7F80_0000);
    }
    let sig53 = (1u64 << 52) | frac64;
    if exp64 >= 897 {
        let mut target_exp = exp64.saturating_sub(896);
        let kept = sig53 >> 29;
        let rem = sig53 & ((1u64 << 29) - 1);
        let half = 1u64 << 28;
        let round_up = rem > half || (rem == half && (kept & 1) == 1);
        let rounded64 = kept.wrapping_add(u64::from(round_up));
        let [r0, r1, r2, r3, ..] = rounded64.to_le_bytes();
        let mut rounded = u32::from_le_bytes([r0, r1, r2, r3]);
        if rounded == (1u32 << 24) {
            rounded >>= 1;
            target_exp = target_exp.saturating_add(1);
        }
        if target_exp >= 255 {
            return f32::from_bits(sign32 | 0x7F80_0000);
        }
        return f32::from_bits(
            sign32 | (target_exp << 23) | (rounded & 0x007F_FFFF),
        );
    }
    let shift = 926u32.saturating_sub(exp64);
    if shift >= 54 {
        return f32::from_bits(sign32);
    }
    let kept = sig53.wrapping_shr(shift);
    let rem_mask = (1u64.wrapping_shl(shift)).wrapping_sub(1);
    let rem = sig53 & rem_mask;
    let half = 1u64.wrapping_shl(shift.saturating_sub(1));
    let round_up = rem > half || (rem == half && (kept & 1) == 1);
    let rounded64 = kept.wrapping_add(u64::from(round_up));
    let [r0, r1, r2, r3, ..] = rounded64.to_le_bytes();
    let rounded = u32::from_le_bytes([r0, r1, r2, r3]);
    f32::from_bits(sign32 | rounded)
}

/// Executes `f64.promote_f32`.
#[inline(always)]
pub fn f64_promote_f32(a: f32) -> f64 {
    f64::from(a)
}

#[inline]
fn trunc_i32_s_impl(x: f64, saturating: bool) -> std::result::Result<i32, WasmTrap> {
    match decompose_f64_trunc(x) {
        TruncParts::Nan => {
            if saturating {
                Ok(0)
            } else {
                Err(WasmTrap::InvalidConversionToInteger)
            }
        }
        TruncParts::OutOfU64Range { neg } => {
            if saturating {
                Ok(if neg { i32::MIN } else { i32::MAX })
            } else {
                Err(WasmTrap::IntegerOverflow)
            }
        }
        TruncParts::InU64Range { neg, mag } => {
            if !neg {
                if mag > 0x7FFF_FFFF {
                    return if saturating {
                        Ok(i32::MAX)
                    } else {
                        Err(WasmTrap::IntegerOverflow)
                    };
                }
                i32::try_from(mag).map_err(|_| WasmTrap::IntegerOverflow)
            } else if mag > 0x8000_0000 {
                if saturating {
                    Ok(i32::MIN)
                } else {
                    Err(WasmTrap::IntegerOverflow)
                }
            } else if mag == 0x8000_0000 {
                Ok(i32::MIN)
            } else {
                let pos = i32::try_from(mag)
                    .map_err(|_| WasmTrap::IntegerOverflow)?;
                Ok(pos.wrapping_neg())
            }
        }
    }
}

#[inline]
fn trunc_i32_u_impl(x: f64, saturating: bool) -> std::result::Result<i32, WasmTrap> {
    match decompose_f64_trunc(x) {
        TruncParts::Nan => {
            if saturating {
                Ok(0)
            } else {
                Err(WasmTrap::InvalidConversionToInteger)
            }
        }
        TruncParts::OutOfU64Range { neg } => {
            if saturating {
                Ok(if neg { 0 } else { as_i32(u32::MAX) })
            } else {
                Err(WasmTrap::IntegerOverflow)
            }
        }
        TruncParts::InU64Range { neg, mag } => {
            if neg && mag > 0 {
                return if saturating {
                    Ok(0)
                } else {
                    Err(WasmTrap::IntegerOverflow)
                };
            }
            if let Ok(u) = u32::try_from(mag) {
                Ok(as_i32(u))
            } else if saturating {
                Ok(as_i32(u32::MAX))
            } else {
                Err(WasmTrap::IntegerOverflow)
            }
        }
    }
}

#[inline]
fn trunc_i64_s_impl(x: f64, saturating: bool) -> std::result::Result<i64, WasmTrap> {
    match decompose_f64_trunc(x) {
        TruncParts::Nan => {
            if saturating {
                Ok(0)
            } else {
                Err(WasmTrap::InvalidConversionToInteger)
            }
        }
        TruncParts::OutOfU64Range { neg } => {
            if saturating {
                Ok(if neg { i64::MIN } else { i64::MAX })
            } else {
                Err(WasmTrap::IntegerOverflow)
            }
        }
        TruncParts::InU64Range { neg, mag } => {
            if !neg {
                if mag > 0x7FFF_FFFF_FFFF_FFFF {
                    return if saturating {
                        Ok(i64::MAX)
                    } else {
                        Err(WasmTrap::IntegerOverflow)
                    };
                }
                Ok(as_i64(mag))
            } else if mag > 0x8000_0000_0000_0000 {
                if saturating {
                    Ok(i64::MIN)
                } else {
                    Err(WasmTrap::IntegerOverflow)
                }
            } else {
                Ok(as_i64(mag.wrapping_neg()))
            }
        }
    }
}

#[inline]
fn trunc_i64_u_impl(x: f64, saturating: bool) -> std::result::Result<i64, WasmTrap> {
    match decompose_f64_trunc(x) {
        TruncParts::Nan => {
            if saturating {
                Ok(0)
            } else {
                Err(WasmTrap::InvalidConversionToInteger)
            }
        }
        TruncParts::OutOfU64Range { neg } => {
            if saturating {
                Ok(if neg { 0 } else { as_i64(u64::MAX) })
            } else {
                Err(WasmTrap::IntegerOverflow)
            }
        }
        TruncParts::InU64Range { neg, mag } => {
            if neg && mag > 0 {
                return if saturating {
                    Ok(0)
                } else {
                    Err(WasmTrap::IntegerOverflow)
                };
            }
            Ok(as_i64(mag))
        }
    }
}

/// Executes `i32.trunc_f32_s`.
#[inline]
pub fn i32_trunc_f32_s(a: f32) -> std::result::Result<i32, WasmTrap> {
    trunc_i32_s_impl(f64::from(a), false)
}

/// Executes `i32.trunc_f32_u`.
#[inline]
pub fn i32_trunc_f32_u(a: f32) -> std::result::Result<i32, WasmTrap> {
    trunc_i32_u_impl(f64::from(a), false)
}

/// Executes `i32.trunc_f64_s`.
#[inline]
pub fn i32_trunc_f64_s(a: f64) -> std::result::Result<i32, WasmTrap> {
    trunc_i32_s_impl(a, false)
}

/// Executes `i32.trunc_f64_u`.
#[inline]
pub fn i32_trunc_f64_u(a: f64) -> std::result::Result<i32, WasmTrap> {
    trunc_i32_u_impl(a, false)
}

/// Executes `i64.trunc_f32_s`.
#[inline]
pub fn i64_trunc_f32_s(a: f32) -> std::result::Result<i64, WasmTrap> {
    trunc_i64_s_impl(f64::from(a), false)
}

/// Executes `i64.trunc_f32_u`.
#[inline]
pub fn i64_trunc_f32_u(a: f32) -> std::result::Result<i64, WasmTrap> {
    trunc_i64_u_impl(f64::from(a), false)
}

/// Executes `i64.trunc_f64_s`.
#[inline]
pub fn i64_trunc_f64_s(a: f64) -> std::result::Result<i64, WasmTrap> {
    trunc_i64_s_impl(a, false)
}

/// Executes `i64.trunc_f64_u`.
#[inline]
pub fn i64_trunc_f64_u(a: f64) -> std::result::Result<i64, WasmTrap> {
    trunc_i64_u_impl(a, false)
}

/// Executes `i32.trunc_sat_f32_s`.
#[inline]
pub fn i32_trunc_sat_f32_s(a: f32) -> i32 {
    // Reason for fallback: saturating trunc_i32_s_impl never returns Err.
    trunc_i32_s_impl(f64::from(a), true).unwrap_or(0)
}

/// Executes `i32.trunc_sat_f32_u`.
#[inline]
pub fn i32_trunc_sat_f32_u(a: f32) -> i32 {
    // Reason for fallback: saturating trunc_i32_u_impl never returns Err.
    trunc_i32_u_impl(f64::from(a), true).unwrap_or(0)
}

/// Executes `i32.trunc_sat_f64_s`.
#[inline]
pub fn i32_trunc_sat_f64_s(a: f64) -> i32 {
    // Reason for fallback: saturating trunc_i32_s_impl never returns Err.
    trunc_i32_s_impl(a, true).unwrap_or(0)
}

/// Executes `i32.trunc_sat_f64_u`.
#[inline]
pub fn i32_trunc_sat_f64_u(a: f64) -> i32 {
    // Reason for fallback: saturating trunc_i32_u_impl never returns Err.
    trunc_i32_u_impl(a, true).unwrap_or(0)
}

/// Executes `i64.trunc_sat_f32_s`.
#[inline]
pub fn i64_trunc_sat_f32_s(a: f32) -> i64 {
    // Reason for fallback: saturating trunc_i64_s_impl never returns Err.
    trunc_i64_s_impl(f64::from(a), true).unwrap_or(0)
}

/// Executes `i64.trunc_sat_f32_u`.
#[inline]
pub fn i64_trunc_sat_f32_u(a: f32) -> i64 {
    // Reason for fallback: saturating trunc_i64_u_impl never returns Err.
    trunc_i64_u_impl(f64::from(a), true).unwrap_or(0)
}

/// Executes `i64.trunc_sat_f64_s`.
#[inline]
pub fn i64_trunc_sat_f64_s(a: f64) -> i64 {
    // Reason for fallback: saturating trunc_i64_s_impl never returns Err.
    trunc_i64_s_impl(a, true).unwrap_or(0)
}

/// Executes `i64.trunc_sat_f64_u`.
#[inline]
pub fn i64_trunc_sat_f64_u(a: f64) -> i64 {
    // Reason for fallback: saturating trunc_i64_u_impl never returns Err.
    trunc_i64_u_impl(a, true).unwrap_or(0)
}

/// Executes `f32.convert_i32_s`.
#[inline]
pub fn f32_convert_i32_s(a: i32) -> f32 {
    // i32 is represented losslessly in f64, so demotion performs a single IEEE-754 rounding.
    f32_demote_f64(f64::from(a))
}

/// Executes `f32.convert_i32_u`.
#[inline]
pub fn f32_convert_i32_u(a: i32) -> f32 {
    // u32 is represented losslessly in f64, so demotion performs a single IEEE-754 rounding.
    f32_demote_f64(f64::from(as_u32(a)))
}

/// Executes `f32.convert_i64_s`.
#[inline]
pub fn f32_convert_i64_s(a: i64) -> f32 {
    let neg = a < 0;
    let mag = if neg {
        as_u64(a.wrapping_neg())
    } else {
        as_u64(a)
    };
    u64_to_f32(mag, neg)
}

/// Executes `f32.convert_i64_u`.
#[inline]
pub fn f32_convert_i64_u(a: i64) -> f32 {
    u64_to_f32(as_u64(a), false)
}

/// Executes `f64.convert_i32_s`.
#[inline(always)]
pub fn f64_convert_i32_s(a: i32) -> f64 {
    f64::from(a)
}

/// Executes `f64.convert_i32_u`.
#[inline(always)]
pub fn f64_convert_i32_u(a: i32) -> f64 {
    f64::from(as_u32(a))
}

/// Executes `f64.convert_i64_s`.
#[inline]
pub fn f64_convert_i64_s(a: i64) -> f64 {
    let neg = a < 0;
    let mag = if neg {
        as_u64(a.wrapping_neg())
    } else {
        as_u64(a)
    };
    u64_to_f64(mag, neg)
}

/// Executes `f64.convert_i64_u`.
#[inline]
pub fn f64_convert_i64_u(a: i64) -> f64 {
    u64_to_f64(as_u64(a), false)
}

/// Executes `i32.reinterpret_f32`.
#[inline(always)]
pub fn i32_reinterpret_f32(a: f32) -> i32 {
    as_i32(a.to_bits())
}

/// Executes `f32.reinterpret_i32`.
#[inline(always)]
pub fn f32_reinterpret_i32(a: i32) -> f32 {
    f32::from_bits(as_u32(a))
}

/// Executes `i64.reinterpret_f64`.
#[inline(always)]
pub fn i64_reinterpret_f64(a: f64) -> i64 {
    as_i64(a.to_bits())
}

/// Executes `f64.reinterpret_i64`.
#[inline(always)]
pub fn f64_reinterpret_i64(a: i64) -> f64 {
    f64::from_bits(as_u64(a))
}

// ============================================================================
// Typed linear memory loads and stores
// ============================================================================

/// Executes `i32.load`.
#[inline(always)]
pub fn i32_load(
    vm: &SubstituteVm,
    mem_idx: usize,
    addr: i32,
    offset: u32,
) -> std::result::Result<i32, WasmTrap> {
    vm.load_bytes_n::<4>(mem_idx, addr, offset)
        .map(i32::from_le_bytes)
}

/// Executes `i64.load`.
#[inline(always)]
pub fn i64_load(
    vm: &SubstituteVm,
    mem_idx: usize,
    addr: i32,
    offset: u32,
) -> std::result::Result<i64, WasmTrap> {
    vm.load_bytes_n::<8>(mem_idx, addr, offset)
        .map(i64::from_le_bytes)
}

/// Executes `f32.load`.
#[inline(always)]
pub fn f32_load(
    vm: &SubstituteVm,
    mem_idx: usize,
    addr: i32,
    offset: u32,
) -> std::result::Result<f32, WasmTrap> {
    vm.load_bytes_n::<4>(mem_idx, addr, offset)
        .map(f32::from_le_bytes)
}

/// Executes `f64.load`.
#[inline(always)]
pub fn f64_load(
    vm: &SubstituteVm,
    mem_idx: usize,
    addr: i32,
    offset: u32,
) -> std::result::Result<f64, WasmTrap> {
    vm.load_bytes_n::<8>(mem_idx, addr, offset)
        .map(f64::from_le_bytes)
}

/// Executes `i32.load8_s`.
#[inline(always)]
pub fn i32_load8_s(
    vm: &SubstituteVm,
    mem_idx: usize,
    addr: i32,
    offset: u32,
) -> std::result::Result<i32, WasmTrap> {
    vm.load_u8(mem_idx, addr, offset)
        .map(|b| i32::from(i8::from_le_bytes([b])))
}

/// Executes `i32.load8_u`.
#[inline(always)]
pub fn i32_load8_u(
    vm: &SubstituteVm,
    mem_idx: usize,
    addr: i32,
    offset: u32,
) -> std::result::Result<i32, WasmTrap> {
    vm.load_u8(mem_idx, addr, offset).map(i32::from)
}

/// Executes `i32.load16_s`.
#[inline(always)]
pub fn i32_load16_s(
    vm: &SubstituteVm,
    mem_idx: usize,
    addr: i32,
    offset: u32,
) -> std::result::Result<i32, WasmTrap> {
    vm.load_bytes_n::<2>(mem_idx, addr, offset)
        .map(|b| i32::from(i16::from_le_bytes(b)))
}

/// Executes `i32.load16_u`.
#[inline(always)]
pub fn i32_load16_u(
    vm: &SubstituteVm,
    mem_idx: usize,
    addr: i32,
    offset: u32,
) -> std::result::Result<i32, WasmTrap> {
    vm.load_bytes_n::<2>(mem_idx, addr, offset)
        .map(|b| i32::from(u16::from_le_bytes(b)))
}

/// Executes `i64.load8_s`.
#[inline(always)]
pub fn i64_load8_s(
    vm: &SubstituteVm,
    mem_idx: usize,
    addr: i32,
    offset: u32,
) -> std::result::Result<i64, WasmTrap> {
    vm.load_u8(mem_idx, addr, offset)
        .map(|b| i64::from(i8::from_le_bytes([b])))
}

/// Executes `i64.load8_u`.
#[inline(always)]
pub fn i64_load8_u(
    vm: &SubstituteVm,
    mem_idx: usize,
    addr: i32,
    offset: u32,
) -> std::result::Result<i64, WasmTrap> {
    vm.load_u8(mem_idx, addr, offset).map(i64::from)
}

/// Executes `i64.load16_s`.
#[inline(always)]
pub fn i64_load16_s(
    vm: &SubstituteVm,
    mem_idx: usize,
    addr: i32,
    offset: u32,
) -> std::result::Result<i64, WasmTrap> {
    vm.load_bytes_n::<2>(mem_idx, addr, offset)
        .map(|b| i64::from(i16::from_le_bytes(b)))
}

/// Executes `i64.load16_u`.
#[inline(always)]
pub fn i64_load16_u(
    vm: &SubstituteVm,
    mem_idx: usize,
    addr: i32,
    offset: u32,
) -> std::result::Result<i64, WasmTrap> {
    vm.load_bytes_n::<2>(mem_idx, addr, offset)
        .map(|b| i64::from(u16::from_le_bytes(b)))
}

/// Executes `i64.load32_s`.
#[inline(always)]
pub fn i64_load32_s(
    vm: &SubstituteVm,
    mem_idx: usize,
    addr: i32,
    offset: u32,
) -> std::result::Result<i64, WasmTrap> {
    vm.load_bytes_n::<4>(mem_idx, addr, offset)
        .map(|b| i64::from(i32::from_le_bytes(b)))
}

/// Executes `i64.load32_u`.
#[inline(always)]
pub fn i64_load32_u(
    vm: &SubstituteVm,
    mem_idx: usize,
    addr: i32,
    offset: u32,
) -> std::result::Result<i64, WasmTrap> {
    vm.load_bytes_n::<4>(mem_idx, addr, offset)
        .map(|b| i64::from(u32::from_le_bytes(b)))
}

/// Executes `i32.store`.
#[inline(always)]
pub fn i32_store(
    vm: &mut SubstituteVm,
    mem_idx: usize,
    addr: i32,
    offset: u32,
    val: i32,
) -> std::result::Result<(), WasmTrap> {
    vm.store_bytes_n(mem_idx, addr, offset, val.to_le_bytes())
}

/// Executes `i64.store`.
#[inline(always)]
pub fn i64_store(
    vm: &mut SubstituteVm,
    mem_idx: usize,
    addr: i32,
    offset: u32,
    val: i64,
) -> std::result::Result<(), WasmTrap> {
    vm.store_bytes_n(mem_idx, addr, offset, val.to_le_bytes())
}

/// Executes `f32.store`.
#[inline(always)]
pub fn f32_store(
    vm: &mut SubstituteVm,
    mem_idx: usize,
    addr: i32,
    offset: u32,
    val: f32,
) -> std::result::Result<(), WasmTrap> {
    vm.store_bytes_n(mem_idx, addr, offset, val.to_le_bytes())
}

/// Executes `f64.store`.
#[inline(always)]
pub fn f64_store(
    vm: &mut SubstituteVm,
    mem_idx: usize,
    addr: i32,
    offset: u32,
    val: f64,
) -> std::result::Result<(), WasmTrap> {
    vm.store_bytes_n(mem_idx, addr, offset, val.to_le_bytes())
}

/// Executes `i32.store8`.
#[inline(always)]
pub fn i32_store8(
    vm: &mut SubstituteVm,
    mem_idx: usize,
    addr: i32,
    offset: u32,
    val: i32,
) -> std::result::Result<(), WasmTrap> {
    let [b0, ..] = val.to_le_bytes();
    vm.store_bytes_n(mem_idx, addr, offset, [b0])
}

/// Executes `i32.store16`.
#[inline(always)]
pub fn i32_store16(
    vm: &mut SubstituteVm,
    mem_idx: usize,
    addr: i32,
    offset: u32,
    val: i32,
) -> std::result::Result<(), WasmTrap> {
    let [b0, b1, ..] = val.to_le_bytes();
    vm.store_bytes_n(mem_idx, addr, offset, [b0, b1])
}

/// Executes `i64.store8`.
#[inline(always)]
pub fn i64_store8(
    vm: &mut SubstituteVm,
    mem_idx: usize,
    addr: i32,
    offset: u32,
    val: i64,
) -> std::result::Result<(), WasmTrap> {
    let [b0, ..] = val.to_le_bytes();
    vm.store_bytes_n(mem_idx, addr, offset, [b0])
}

/// Executes `i64.store16`.
#[inline(always)]
pub fn i64_store16(
    vm: &mut SubstituteVm,
    mem_idx: usize,
    addr: i32,
    offset: u32,
    val: i64,
) -> std::result::Result<(), WasmTrap> {
    let [b0, b1, ..] = val.to_le_bytes();
    vm.store_bytes_n(mem_idx, addr, offset, [b0, b1])
}

/// Executes `i64.store32`.
#[inline(always)]
pub fn i64_store32(
    vm: &mut SubstituteVm,
    mem_idx: usize,
    addr: i32,
    offset: u32,
    val: i64,
) -> std::result::Result<(), WasmTrap> {
    let [b0, b1, b2, b3, ..] = val.to_le_bytes();
    vm.store_bytes_n(mem_idx, addr, offset, [b0, b1, b2, b3])
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
    use crate::vm::WasmLinearMemory;

    #[crate::ctb_test]
    fn test_integer_div_and_rem_edge_cases() {
        assert_eq!(i32_div_s(i32::MIN, -1), Err(WasmTrap::IntegerOverflow));
        assert_eq!(i32_div_s(7, 0), Err(WasmTrap::IntegerDivideByZero));
        assert_eq!(i32_rem_s(i32::MIN, -1), Ok(0));
        assert_eq!(i32_rem_s(7, 0), Err(WasmTrap::IntegerDivideByZero));
        assert_eq!(i32_rem_u(-1, 10), Ok(5));
        assert_eq!(i32_div_u(-1, 2), Ok(i32::MAX));

        assert_eq!(i64_div_s(i64::MIN, -1), Err(WasmTrap::IntegerOverflow));
        assert_eq!(i64_rem_s(i64::MIN, -1), Ok(0));
        assert_eq!(i64_div_s(1, 0), Err(WasmTrap::IntegerDivideByZero));
        assert_eq!(i64_rem_u(1, 0), Err(WasmTrap::IntegerDivideByZero));
    }

    #[crate::ctb_test]
    fn test_shifts_rotates_and_sign_extensions() {
        assert_eq!(i32_shl(1, 33), 2);
        assert_eq!(i32_shr_s(i32::MIN, 1), i32::MIN >> 1);
        assert_eq!(i32_shr_u(i32::MIN, 1), 0x4000_0000);
        assert_eq!(i32_rotl(0x1234_5678, 8), 0x3456_7812);
        assert_eq!(i32_rotr(0x1234_5678, 8), 0x7812_3456);
        assert_eq!(i32_extend8_s(0xFF), -1);
        assert_eq!(i32_extend16_s(0x8000), -32768);
        assert_eq!(i64_extend8_s(0x80), -128);
        assert_eq!(i64_extend16_s(0xFFFF), -1);
        assert_eq!(i64_extend32_s(0xFFFF_FFFF), -1);
        assert_eq!(i64_shr_u(-1, 63), 1);
    }

    #[crate::ctb_test]
    fn test_float_min_max_nearest_and_conversions() {
        assert!(f32_min(f32::NAN, 1.0).is_nan());
        assert!(f32_max(1.0, f32::NAN).is_nan());
        assert_eq!(f32_min(-0.0, 0.0).to_bits(), 0x8000_0000);
        assert_eq!(f32_min(0.0, -0.0).to_bits(), 0x8000_0000);
        assert_eq!(f32_max(-0.0, 0.0).to_bits(), 0);
        assert_eq!(f32_max(0.0, -0.0).to_bits(), 0);
        assert_eq!(f32_nearest(2.5), 2.0);
        assert_eq!(f32_nearest(3.5), 4.0);
        assert_eq!(f32_nearest(-0.5).to_bits(), 0x8000_0000);

        assert!(f64_min(1.0, f64::NAN).is_nan());
        assert_eq!(f64_min(-0.0, 0.0).to_bits(), 0x8000_0000_0000_0000);
        assert_eq!(f64_max(-0.0, 0.0).to_bits(), 0);
        assert_eq!(f64_nearest(2.5), 2.0);
        assert_eq!(f64_nearest(3.5), 4.0);

        // Trunc edge cases (WASM allows -2147483648.5 -> i32::MIN, -0.9 -> 0u32)
        assert_eq!(i32_trunc_f64_s(-2_147_483_648.5), Ok(i32::MIN));
        assert_eq!(i32_trunc_f64_s(-2_147_483_649.0), Err(WasmTrap::IntegerOverflow));
        assert_eq!(i32_trunc_f64_s(2_147_483_648.0), Err(WasmTrap::IntegerOverflow));
        assert_eq!(i32_trunc_f64_s(f64::NAN), Err(WasmTrap::InvalidConversionToInteger));
        assert_eq!(i32_trunc_f64_u(-0.9), Ok(0));
        assert_eq!(i32_trunc_f64_u(-1.0), Err(WasmTrap::IntegerOverflow));
        assert_eq!(i64_trunc_f64_s(-2.0_f64.powi(63)), Ok(i64::MIN));
        assert_eq!(i64_trunc_sat_f64_s(f64::INFINITY), i64::MAX);
        assert_eq!(i64_trunc_sat_f64_s(f64::NEG_INFINITY), i64::MIN);
        assert_eq!(i64_trunc_sat_f64_s(f64::NAN), 0);
        assert_eq!(i32_trunc_sat_f32_u(f32::INFINITY), -1);
        assert_eq!(i32_trunc_sat_f32_u(-5.0), 0);

        // Int -> Float round-to-nearest-even & demotion
        assert_eq!(f64_convert_i64_u(i64::MAX), 2.0_f64.powi(63));
        assert_eq!(f64_convert_i64_s(i64::MIN), -2.0_f64.powi(63));
        assert_eq!(f32_convert_i64_s(i64::MIN), -2.0_f32.powi(63));
        assert_eq!(f32_convert_i32_s(-1), -1.0);
        assert_eq!(f32_convert_i32_u(-1), 2.0_f32.powi(32));
        assert_eq!(f32_demote_f64(f64::from(f32::MIN_POSITIVE)).to_bits(), f32::MIN_POSITIVE.to_bits());
        assert_eq!(f32_demote_f64(-0.0).to_bits(), 0x8000_0000);
        assert_eq!(f32_demote_f64(1e40), f32::INFINITY);
        assert_eq!(f32_demote_f64(1e-50).to_bits(), 0);
        let subnorm = f32::from_bits(1);
        assert_eq!(f32_demote_f64(f64::from(subnorm)).to_bits(), 1);
    }

    #[crate::ctb_test]
    fn test_typed_memory_loads_and_stores() {
        let mut vm = SubstituteVm::new(
            vec![WasmLinearMemory::new(1, Some(1))],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );
        i32_store8(&mut vm, 0, 10, 0, -2).unwrap();
        assert_eq!(i32_load8_s(&vm, 0, 10, 0), Ok(-2));
        assert_eq!(i32_load8_u(&vm, 0, 10, 0), Ok(254));
        i64_store32(&mut vm, 0, 20, 4, -5).unwrap();
        assert_eq!(i64_load32_s(&vm, 0, 20, 4), Ok(-5));
        assert_eq!(i64_load32_u(&vm, 0, 20, 4), Ok(0xFFFF_FFFB));
        f64_store(&mut vm, 0, 65528, 0, 3.25).unwrap();
        assert_eq!(f64_load(&vm, 0, 65528, 0), Ok(3.25));
        assert_eq!(
            i32_load(&vm, 0, 65533, 0),
            Err(WasmTrap::OutOfBoundsMemoryAccess)
        );
        assert_eq!(
            i32_load(&vm, 0, -1, 10),
            Err(WasmTrap::OutOfBoundsMemoryAccess)
        );
    }
}
