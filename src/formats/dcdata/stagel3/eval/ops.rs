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

//! Unified built-in unary, binary, ternary, conversion, and bitcast evaluators
//! for StageL 3 (`Core*Op`) and StageR 3 (`StageR*Op`).
//!
//! Shares numeric arithmetic, division/remainder, shifts, sequence indexing,
//! slicing, buffer mutations, and exact conversions across trapping StageL
//! (`EvalSignal::Fault`) and total StageR (`Value::Result` / `Value::Option`)
//! semantics.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::cell::RefCell;
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};
use std::rc::Rc;

use malachite::base::num::arithmetic::traits::UnsignedAbs;
use malachite::base::num::basic::traits::Zero;
use malachite::{Integer, Rational};

use super::super::ast::{
    CoreBinaryOp, CoreTernaryOp, CoreUnaryOp, Span, StageLType, StageRBinaryOp,
    StageRErrorTag, StageRTernaryOp, StageRUnaryOp,
};
use super::super::stager_strings::{
    dcs_from_s, explode_slice, implode_slices, is_valid_dc_u128,
    s_atomic_units, s_from_dcs, validate_and_measure_s_byte_len,
};
use super::super::stager_validator::is_stager_widening_conv;
use super::vm::{EvalSignal, StageLWorkspaceVm};
use super::{BufStorage, Value};
use crate::DcChar;

fn ok_res(val: Value) -> EvalSignal {
    EvalSignal::Value(Value::Result(Ok(Box::new(val))))
}

fn err_res(tag: StageRErrorTag) -> EvalSignal {
    EvalSignal::Value(Value::Result(Err(tag)))
}

trait FixedIntScalar: Copy + Eq + Ord {
    const ZERO: Self;
    const NEG_ONE: Option<Self>;
    fn wrap_val(self) -> Value;
    fn checked_add(self, rhs: Self) -> Option<Self>;
    fn checked_sub(self, rhs: Self) -> Option<Self>;
    fn checked_mul(self, rhs: Self) -> Option<Self>;
    fn checked_div(self, rhs: Self) -> Option<Self>;
    fn checked_rem(self, rhs: Self) -> Option<Self>;
    fn checked_rem_euclid(self, rhs: Self) -> Option<Self>;
    fn checked_shl(self, cnt: u32) -> Option<Self>;
    fn checked_shr(self, cnt: u32) -> Option<Self>;
    fn wrapping_add(self, rhs: Self) -> Self;
    fn wrapping_sub(self, rhs: Self) -> Self;
    fn wrapping_mul(self, rhs: Self) -> Self;
    fn bit_and(self, rhs: Self) -> Self;
    fn bit_or(self, rhs: Self) -> Self;
    fn bit_xor(self, rhs: Self) -> Self;
    fn bit_not(self) -> Self;
    fn leading_zeros(self) -> u32;
    fn trailing_zeros(self) -> u32;
    fn count_ones(self) -> u32;
}

macro_rules! impl_fixed_int_scalar {
    ($ty:ty, $variant:ident, $neg_one:expr) => {
        impl FixedIntScalar for $ty {
            const ZERO: Self = 0;
            const NEG_ONE: Option<Self> = $neg_one;

            fn wrap_val(self) -> Value {
                Value::$variant(self)
            }
            fn checked_add(self, rhs: Self) -> Option<Self> {
                <$ty>::checked_add(self, rhs)
            }
            fn checked_sub(self, rhs: Self) -> Option<Self> {
                <$ty>::checked_sub(self, rhs)
            }
            fn checked_mul(self, rhs: Self) -> Option<Self> {
                <$ty>::checked_mul(self, rhs)
            }
            fn checked_div(self, rhs: Self) -> Option<Self> {
                <$ty>::checked_div(self, rhs)
            }
            fn checked_rem(self, rhs: Self) -> Option<Self> {
                <$ty>::checked_rem(self, rhs)
            }
            fn checked_rem_euclid(self, rhs: Self) -> Option<Self> {
                <$ty>::checked_rem_euclid(self, rhs)
            }
            fn checked_shl(self, cnt: u32) -> Option<Self> {
                <$ty>::checked_shl(self, cnt)
            }
            fn checked_shr(self, cnt: u32) -> Option<Self> {
                <$ty>::checked_shr(self, cnt)
            }
            fn wrapping_add(self, rhs: Self) -> Self {
                <$ty>::wrapping_add(self, rhs)
            }
            fn wrapping_sub(self, rhs: Self) -> Self {
                <$ty>::wrapping_sub(self, rhs)
            }
            fn wrapping_mul(self, rhs: Self) -> Self {
                <$ty>::wrapping_mul(self, rhs)
            }
            fn bit_and(self, rhs: Self) -> Self {
                self & rhs
            }
            fn bit_or(self, rhs: Self) -> Self {
                self | rhs
            }
            fn bit_xor(self, rhs: Self) -> Self {
                self ^ rhs
            }
            fn bit_not(self) -> Self {
                !self
            }
            fn leading_zeros(self) -> u32 {
                <$ty>::leading_zeros(self)
            }
            fn trailing_zeros(self) -> u32 {
                <$ty>::trailing_zeros(self)
            }
            fn count_ones(self) -> u32 {
                <$ty>::count_ones(self)
            }
        }
    };
}

impl_fixed_int_scalar!(u8, U8, None);
impl_fixed_int_scalar!(i32, I32, Some(-1));
impl_fixed_int_scalar!(u32, U32, None);
impl_fixed_int_scalar!(i64, I64, Some(-1));
impl_fixed_int_scalar!(u64, U64, None);
impl_fixed_int_scalar!(i128, I128, Some(-1));
impl_fixed_int_scalar!(u128, U128, None);

#[derive(Clone, Copy)]
enum FixedUnaryBitOp {
    BitNot,
    Clz,
    Ctz,
    Popcnt,
}

#[derive(Clone, Copy)]
enum FixedBinKernel {
    CheckedAdd,
    CheckedSub,
    CheckedMul,
    CheckedDiv,
    CoreRem,
    TotalMod,
    TotalTruncRem,
    WrapAdd,
    WrapSub,
    WrapMul,
    BitAnd,
    BitOr,
    BitXor,
}

fn apply_fixed_unary_bit<T: FixedIntScalar>(
    op: FixedUnaryBitOp,
    x: T,
) -> Value {
    match op {
        FixedUnaryBitOp::BitNot => x.bit_not().wrap_val(),
        FixedUnaryBitOp::Clz => Value::U32(x.leading_zeros()),
        FixedUnaryBitOp::Ctz => Value::U32(x.trailing_zeros()),
        FixedUnaryBitOp::Popcnt => Value::U32(x.count_ones()),
    }
}

fn eval_fixed_unary_bit(op: FixedUnaryBitOp, val: Value) -> Result<EvalSignal> {
    let out = match val {
        Value::U8(x) => apply_fixed_unary_bit(op, x),
        Value::I32(x) => apply_fixed_unary_bit(op, x),
        Value::U32(x) => apply_fixed_unary_bit(op, x),
        Value::I64(x) => apply_fixed_unary_bit(op, x),
        Value::U64(x) => apply_fixed_unary_bit(op, x),
        Value::I128(x) => apply_fixed_unary_bit(op, x),
        Value::U128(x) => apply_fixed_unary_bit(op, x),
        other => {
            let name = match op {
                FixedUnaryBitOp::BitNot => "bitNot",
                FixedUnaryBitOp::Clz => "clz",
                FixedUnaryBitOp::Ctz => "ctz",
                FixedUnaryBitOp::Popcnt => "popcnt",
            };
            bail!("Invalid operand {other:?} for '{name}'");
        }
    };
    Ok(EvalSignal::Value(out))
}

fn wrap_checked_neg<T: FixedIntScalar>(
    res: Option<T>,
    ty_name: &str,
    total: bool,
) -> EvalSignal {
    match (res, total) {
        (Some(v), true) => ok_res(v.wrap_val()),
        (None, true) => err_res(StageRErrorTag::Overflow),
        (Some(v), false) => EvalSignal::Value(v.wrap_val()),
        (None, false) => EvalSignal::Fault(format!("{ty_name} neg overflow")),
    }
}

fn eval_fixed_neg(
    val: Value,
    total: bool,
    span: Option<Span>,
) -> Result<EvalSignal> {
    match val {
        Value::I32(x) => Ok(wrap_checked_neg(x.checked_neg(), "i32", total)),
        Value::I64(x) => Ok(wrap_checked_neg(x.checked_neg(), "i64", total)),
        Value::I128(x) if total => {
            Ok(wrap_checked_neg(x.checked_neg(), "i128", true))
        }
        other => {
            if let Some(sp) = span {
                bail!("Invalid operand {other:?} for StageR 'neg' at {sp}");
            }
            bail!("Invalid operand {other:?} for 'neg'");
        }
    }
}

pub(super) fn eval_unary(op: CoreUnaryOp, val: Value) -> Result<EvalSignal> {
    match op {
        CoreUnaryOp::Neg => eval_fixed_neg(val, false, None),
        CoreUnaryOp::BitNot => {
            eval_fixed_unary_bit(FixedUnaryBitOp::BitNot, val)
        }
        CoreUnaryOp::Not => match val {
            Value::Bool(b) => Ok(EvalSignal::Value(Value::Bool(!b))),
            other => bail!("Invalid operand {other:?} for 'not'"),
        },
        CoreUnaryOp::Clz => eval_fixed_unary_bit(FixedUnaryBitOp::Clz, val),
        CoreUnaryOp::Ctz => eval_fixed_unary_bit(FixedUnaryBitOp::Ctz, val),
        CoreUnaryOp::Popcnt => {
            eval_fixed_unary_bit(FixedUnaryBitOp::Popcnt, val)
        }
        CoreUnaryOp::Len => {
            let len = match val {
                Value::List(xs) => xs.len(),
                Value::Buf(b) => b.borrow().elems.len(),
                Value::Dcs(chars) => chars.len(),
                other => bail!("Invalid operand {other:?} for 'len'"),
            };
            Ok(EvalSignal::Value(Value::U64(
                u64::try_from(len).context("len exceeds u64")?,
            )))
        }
        CoreUnaryOp::BufFreeze => {
            let Value::Buf(b) = val else {
                bail!("Invalid operand {val:?} for 'bufFreeze'");
            };
            let mut st = b.borrow_mut();
            if st.frozen {
                return Ok(EvalSignal::Fault(String::from(
                    "bufFreeze called on an already-frozen buffer",
                )));
            }
            st.frozen = true;
            let frozen_vec = std::mem::take(&mut st.elems);
            Ok(EvalSignal::Value(Value::List(Rc::new(frozen_vec))))
        }
    }
}

pub(super) fn eval_stager_unary(
    vm: &StageLWorkspaceVm,
    op: StageRUnaryOp,
    val: Value,
    span: Span,
) -> Result<EvalSignal> {
    match op {
        StageRUnaryOp::TotalNeg => match val {
            Value::Int(x) => {
                let neg = x.neg();
                if let Some(sig) = vm.check_int_limit(&neg, "neg") {
                    return Ok(sig);
                }
                Ok(EvalSignal::Value(Value::Int(neg)))
            }
            Value::Rat(r) => {
                let neg = r.neg();
                if let Some(sig) = vm.check_rat_limit(&neg, "neg") {
                    return Ok(sig);
                }
                Ok(EvalSignal::Value(Value::Rat(neg)))
            }
            other => eval_fixed_neg(other, true, Some(span)),
        },
        StageRUnaryOp::IsSome => {
            let Value::Option(opt) = val else {
                bail!("'isSome' expected option/T, got {val:?} at {span}");
            };
            Ok(EvalSignal::Value(Value::Bool(opt.is_some())))
        }
        StageRUnaryOp::IsNone => {
            let Value::Option(opt) = val else {
                bail!("'isNone' expected option/T, got {val:?} at {span}");
            };
            Ok(EvalSignal::Value(Value::Bool(opt.is_none())))
        }
        StageRUnaryOp::IsOk => {
            let Value::Result(res) = val else {
                bail!("'isOk' expected result/T, got {val:?} at {span}");
            };
            Ok(EvalSignal::Value(Value::Bool(res.is_ok())))
        }
        StageRUnaryOp::IsErr => {
            let Value::Result(res) = val else {
                bail!("'isErr' expected result/T, got {val:?} at {span}");
            };
            Ok(EvalSignal::Value(Value::Bool(res.is_err())))
        }
        StageRUnaryOp::ByteLen => {
            let Value::S(chars) = val else {
                bail!("'byteLen' expected 's', got {val:?} at {span}");
            };
            let byte_len = validate_and_measure_s_byte_len(&chars)?;
            Ok(EvalSignal::Value(Value::U64(byte_len)))
        }
        StageRUnaryOp::DcFromU128 => {
            let Value::U128(raw) = val else {
                bail!("'dcFromU128' expected 'u128', got {val:?} at {span}");
            };
            if is_valid_dc_u128(raw) {
                Ok(ok_res(Value::Dc(DcChar::from_u128(raw))))
            } else {
                Ok(err_res(StageRErrorTag::InvalidDc))
            }
        }
        StageRUnaryOp::U128FromDc => {
            let Value::Dc(dc) = val else {
                bail!("'u128FromDc' expected 'dc', got {val:?} at {span}");
            };
            Ok(EvalSignal::Value(Value::U128(dc.to_u128())))
        }
        StageRUnaryOp::DcsFromS => {
            let Value::S(chars) = val else {
                bail!("'dcsFromS' expected 's', got {val:?} at {span}");
            };
            match dcs_from_s(&chars) {
                Ok(dcs) => Ok(ok_res(Value::Dcs(Rc::new(dcs)))),
                Err(()) => Ok(err_res(StageRErrorTag::InvalidEncoding)),
            }
        }
        StageRUnaryOp::SFromDcs => {
            let Value::Dcs(chars) = val else {
                bail!("'sFromDcs' expected 'dcs', got {val:?} at {span}");
            };
            Ok(EvalSignal::Value(Value::S(Rc::new(s_from_dcs(&chars)))))
        }
    }
}

fn wrap_checked_bin<T: FixedIntScalar>(
    res: Option<T>,
    op_name: &str,
    total: bool,
) -> EvalSignal {
    match (res, total) {
        (Some(v), true) => ok_res(v.wrap_val()),
        (None, true) => err_res(StageRErrorTag::Overflow),
        (Some(v), false) => EvalSignal::Value(v.wrap_val()),
        (None, false) => {
            EvalSignal::Fault(format!("checked arithmetic fault in '{op_name}'"))
        }
    }
}

fn apply_fixed_bin<T: FixedIntScalar>(
    kernel: FixedBinKernel,
    a: T,
    b: T,
    op_name: &str,
    total: bool,
) -> Result<EvalSignal> {
    match kernel {
        FixedBinKernel::CheckedAdd => {
            Ok(wrap_checked_bin(a.checked_add(b), op_name, total))
        }
        FixedBinKernel::CheckedSub => {
            Ok(wrap_checked_bin(a.checked_sub(b), op_name, total))
        }
        FixedBinKernel::CheckedMul => {
            Ok(wrap_checked_bin(a.checked_mul(b), op_name, total))
        }
        FixedBinKernel::CheckedDiv => {
            if b == T::ZERO {
                return Ok(if total {
                    err_res(StageRErrorTag::DivisionByZero)
                } else {
                    EvalSignal::Fault(format!(
                        "checked arithmetic fault in '{op_name}'"
                    ))
                });
            }
            Ok(wrap_checked_bin(a.checked_div(b), op_name, total))
        }
        FixedBinKernel::CoreRem => {
            Ok(wrap_checked_bin(a.checked_rem(b), op_name, false))
        }
        FixedBinKernel::TotalMod => {
            if b == T::ZERO {
                return Ok(err_res(StageRErrorTag::DivisionByZero));
            }
            if T::NEG_ONE == Some(b) {
                return Ok(ok_res(T::ZERO.wrap_val()));
            }
            let out = a.checked_rem_euclid(b).context(
                "Euclidean modulo failed unexpectedly for divisor != 0, -1",
            )?;
            Ok(ok_res(out.wrap_val()))
        }
        FixedBinKernel::TotalTruncRem => {
            if b == T::ZERO {
                return Ok(err_res(StageRErrorTag::DivisionByZero));
            }
            if T::NEG_ONE == Some(b) {
                return Ok(ok_res(T::ZERO.wrap_val()));
            }
            let out = a.checked_rem(b).context(
                "Truncating remainder failed unexpectedly for divisor != 0, -1",
            )?;
            Ok(ok_res(out.wrap_val()))
        }
        FixedBinKernel::WrapAdd => {
            Ok(EvalSignal::Value(a.wrapping_add(b).wrap_val()))
        }
        FixedBinKernel::WrapSub => {
            Ok(EvalSignal::Value(a.wrapping_sub(b).wrap_val()))
        }
        FixedBinKernel::WrapMul => {
            Ok(EvalSignal::Value(a.wrapping_mul(b).wrap_val()))
        }
        FixedBinKernel::BitAnd => {
            Ok(EvalSignal::Value(a.bit_and(b).wrap_val()))
        }
        FixedBinKernel::BitOr => Ok(EvalSignal::Value(a.bit_or(b).wrap_val())),
        FixedBinKernel::BitXor => {
            Ok(EvalSignal::Value(a.bit_xor(b).wrap_val()))
        }
    }
}

fn dispatch_fixed_bin(
    kernel: FixedBinKernel,
    left: Value,
    right: Value,
    core_only: bool,
    total: bool,
    op_name: &str,
    span: Option<Span>,
) -> Result<EvalSignal> {
    match (left, right) {
        (Value::U8(a), Value::U8(b)) => {
            apply_fixed_bin(kernel, a, b, op_name, total)
        }
        (Value::I32(a), Value::I32(b)) => {
            apply_fixed_bin(kernel, a, b, op_name, total)
        }
        (Value::U32(a), Value::U32(b)) => {
            apply_fixed_bin(kernel, a, b, op_name, total)
        }
        (Value::I64(a), Value::I64(b)) => {
            apply_fixed_bin(kernel, a, b, op_name, total)
        }
        (Value::U64(a), Value::U64(b)) => {
            apply_fixed_bin(kernel, a, b, op_name, total)
        }
        (Value::I128(a), Value::I128(b)) if !core_only => {
            apply_fixed_bin(kernel, a, b, op_name, total)
        }
        (Value::U128(a), Value::U128(b)) if !core_only => {
            apply_fixed_bin(kernel, a, b, op_name, total)
        }
        (l, r) => {
            if let Some(sp) = span {
                bail!(
                    "Mismatched operands {l:?} vs {r:?} for StageR '{op_name}' at {sp}"
                );
            }
            bail!("Mismatched types {l:?} vs {r:?} for '{op_name}'");
        }
    }
}

fn apply_fixed_shift<T: FixedIntScalar>(
    is_left: bool,
    x: T,
    cnt: u32,
    total: bool,
) -> EvalSignal {
    let shifted = if is_left {
        x.checked_shl(cnt)
    } else {
        x.checked_shr(cnt)
    };
    match (shifted, total) {
        (Some(v), true) => ok_res(v.wrap_val()),
        (None, true) => err_res(StageRErrorTag::Overflow),
        (Some(v), false) => EvalSignal::Value(v.wrap_val()),
        (None, false) => {
            EvalSignal::Fault(format!("shift count {cnt} out of range"))
        }
    }
}

fn eval_shift_shared(
    is_left: bool,
    val: Value,
    count: Value,
    allow_128: bool,
    total: bool,
    span: Option<Span>,
) -> Result<EvalSignal> {
    let Value::U32(cnt) = count else {
        if let Some(sp) = span {
            bail!("Shift count must be u32, got {count:?} at {sp}");
        }
        bail!("Shift count must be u32, got {count:?}");
    };
    match val {
        Value::U8(x) => Ok(apply_fixed_shift(is_left, x, cnt, total)),
        Value::I32(x) => Ok(apply_fixed_shift(is_left, x, cnt, total)),
        Value::U32(x) => Ok(apply_fixed_shift(is_left, x, cnt, total)),
        Value::I64(x) => Ok(apply_fixed_shift(is_left, x, cnt, total)),
        Value::U64(x) => Ok(apply_fixed_shift(is_left, x, cnt, total)),
        Value::I128(x) if allow_128 => {
            Ok(apply_fixed_shift(is_left, x, cnt, total))
        }
        Value::U128(x) if allow_128 => {
            Ok(apply_fixed_shift(is_left, x, cnt, total))
        }
        other => {
            if let Some(sp) = span {
                bail!("Invalid operand {other:?} for StageR shift at {sp}");
            }
            bail!("Invalid operand {other:?} for shift");
        }
    }
}

fn eval_rotate(is_left: bool, val: Value, count: Value) -> Result<EvalSignal> {
    let Value::U32(cnt) = count else {
        bail!("Rotate count must be u32, got {count:?}");
    };
    let res = match val {
        Value::U8(x) => Value::U8(if is_left {
            x.rotate_left(cnt)
        } else {
            x.rotate_right(cnt)
        }),
        Value::U32(x) => Value::U32(if is_left {
            x.rotate_left(cnt)
        } else {
            x.rotate_right(cnt)
        }),
        Value::U64(x) => Value::U64(if is_left {
            x.rotate_left(cnt)
        } else {
            x.rotate_right(cnt)
        }),
        Value::U128(x) => Value::U128(if is_left {
            x.rotate_left(cnt)
        } else {
            x.rotate_right(cnt)
        }),
        other => bail!("Invalid operand {other:?} for rotate"),
    };
    Ok(EvalSignal::Value(res))
}

fn cmp_numeric(
    left: &Value,
    right: &Value,
    pred: fn(std::cmp::Ordering) -> bool,
) -> Result<EvalSignal> {
    let ord = match (left, right) {
        (Value::U8(a), Value::U8(b)) => a.cmp(b),
        (Value::I32(a), Value::I32(b)) => a.cmp(b),
        (Value::U32(a), Value::U32(b)) => a.cmp(b),
        (Value::I64(a), Value::I64(b)) => a.cmp(b),
        (Value::U64(a), Value::U64(b)) => a.cmp(b),
        (Value::I128(a), Value::I128(b)) => a.cmp(b),
        (Value::U128(a), Value::U128(b)) => a.cmp(b),
        (Value::Int(a), Value::Int(b)) => a.cmp(b),
        (Value::Rat(a), Value::Rat(b)) => a.cmp(b),
        (l, r) => bail!("Mismatched types {l:?} vs {r:?} for comparison"),
    };
    Ok(EvalSignal::Value(Value::Bool(pred(ord))))
}

fn eval_get_shared(
    seq: Value,
    index: Value,
    allow_dcs: bool,
    total: bool,
    span: Option<Span>,
) -> Result<EvalSignal> {
    let idx = index.as_u64_index("get")?;
    let opt = match seq {
        Value::List(xs) => xs.get(idx).cloned(),
        Value::Buf(b) => b.borrow().elems.get(idx).cloned(),
        Value::Dcs(chars) if allow_dcs => {
            chars.get(idx).copied().map(Value::Dc)
        }
        other => {
            if let Some(sp) = span {
                bail!(
                    "Invalid sequence operand {other:?} for StageR 'get' at {sp}"
                );
            }
            bail!("Invalid operand {other:?} for 'get'");
        }
    };
    if total {
        Ok(EvalSignal::Value(Value::Option(opt.map(Box::new))))
    } else {
        match opt {
            Some(v) => Ok(EvalSignal::Value(v)),
            None => Ok(EvalSignal::Fault(format!(
                "get index {idx} out of bounds"
            ))),
        }
    }
}

fn concat_slices<T: Clone>(
    a: &[T],
    b: &[T],
    wrap: fn(Rc<Vec<T>>) -> Value,
    vm: &StageLWorkspaceVm,
) -> Result<EvalSignal> {
    let total = a
        .len()
        .checked_add(b.len())
        .context("concat length overflow")?;
    if let Some(fault) = vm.check_seq_limit(total, "concat result") {
        return Ok(fault);
    }
    let mut merged = Vec::with_capacity(total);
    merged.extend_from_slice(a);
    merged.extend_from_slice(b);
    Ok(EvalSignal::Value(wrap(Rc::new(merged))))
}

fn eval_core_buf_binary(
    vm: &StageLWorkspaceVm,
    op: CoreBinaryOp,
    left: Value,
    right: Value,
) -> Result<EvalSignal> {
    match op {
        CoreBinaryOp::BufFill => {
            let len = right.as_u64_index("bufFill")?;
            if let Some(fault) = vm.check_seq_limit(len, "bufFill capacity") {
                return Ok(fault);
            }
            let mut elems = Vec::with_capacity(len.min(65_536));
            elems.resize(len, left);
            Ok(EvalSignal::Value(Value::Buf(Rc::new(RefCell::new(
                BufStorage {
                    elems,
                    frozen: false,
                },
            )))))
        }
        CoreBinaryOp::BufPush => {
            let Value::Buf(b) = left else {
                bail!("Invalid buffer operand for 'bufPush'");
            };
            let mut st = b.borrow_mut();
            if st.frozen {
                return Ok(EvalSignal::Fault(String::from(
                    "bufPush on frozen buffer",
                )));
            }
            let next_len = st
                .elems
                .len()
                .checked_add(1)
                .context("bufPush length overflow")?;
            if let Some(fault) = vm.check_seq_limit(next_len, "bufPush") {
                return Ok(fault);
            }
            st.elems.push(right);
            Ok(EvalSignal::Value(Value::Void))
        }
        CoreBinaryOp::BufExtend => {
            let (Value::Buf(b), Value::List(xs)) = (left, right) else {
                bail!("Invalid operands for 'bufExtend'");
            };
            let mut st = b.borrow_mut();
            if st.frozen {
                return Ok(EvalSignal::Fault(String::from(
                    "bufExtend on frozen buffer",
                )));
            }
            let total = st
                .elems
                .len()
                .checked_add(xs.len())
                .context("bufExtend length overflow")?;
            if let Some(fault) = vm.check_seq_limit(total, "bufExtend result") {
                return Ok(fault);
            }
            st.elems.extend_from_slice(&xs);
            Ok(EvalSignal::Value(Value::Void))
        }
        _ => bail!("Unexpected non-buffer binary op {op:?}"),
    }
}

fn core_fixed_bin_spec(
    op: CoreBinaryOp,
) -> Result<(FixedBinKernel, bool, &'static str)> {
    match op {
        CoreBinaryOp::Add => Ok((FixedBinKernel::CheckedAdd, true, "add")),
        CoreBinaryOp::Sub => Ok((FixedBinKernel::CheckedSub, true, "sub")),
        CoreBinaryOp::Mul => Ok((FixedBinKernel::CheckedMul, true, "mul")),
        CoreBinaryOp::Div => Ok((FixedBinKernel::CheckedDiv, true, "div")),
        CoreBinaryOp::Rem => Ok((FixedBinKernel::CoreRem, true, "rem")),
        CoreBinaryOp::WrapAdd => Ok((FixedBinKernel::WrapAdd, false, "wrapAdd")),
        CoreBinaryOp::WrapSub => Ok((FixedBinKernel::WrapSub, false, "wrapSub")),
        CoreBinaryOp::WrapMul => Ok((FixedBinKernel::WrapMul, false, "wrapMul")),
        CoreBinaryOp::BitAnd => Ok((FixedBinKernel::BitAnd, false, "bitAnd")),
        CoreBinaryOp::BitOr => Ok((FixedBinKernel::BitOr, false, "bitOr")),
        CoreBinaryOp::BitXor => Ok((FixedBinKernel::BitXor, false, "bitXor")),
        other => bail!("Unexpected fixed binary op {other:?}"),
    }
}

pub(super) fn eval_binary(
    vm: &StageLWorkspaceVm,
    op: CoreBinaryOp,
    left: Value,
    right: Value,
) -> Result<EvalSignal> {
    match op {
        CoreBinaryOp::Add
        | CoreBinaryOp::Sub
        | CoreBinaryOp::Mul
        | CoreBinaryOp::Div
        | CoreBinaryOp::Rem
        | CoreBinaryOp::WrapAdd
        | CoreBinaryOp::WrapSub
        | CoreBinaryOp::WrapMul
        | CoreBinaryOp::BitAnd
        | CoreBinaryOp::BitOr
        | CoreBinaryOp::BitXor => {
            let (kernel, core_only, name) = core_fixed_bin_spec(op)?;
            dispatch_fixed_bin(
                kernel, left, right, core_only, false, name, None,
            )
        }
        CoreBinaryOp::Shl => {
            eval_shift_shared(true, left, right, false, false, None)
        }
        CoreBinaryOp::Shr => {
            eval_shift_shared(false, left, right, false, false, None)
        }
        CoreBinaryOp::Rotl => eval_rotate(true, left, right),
        CoreBinaryOp::Rotr => eval_rotate(false, left, right),
        CoreBinaryOp::Eq => Ok(EvalSignal::Value(Value::Bool(left == right))),
        CoreBinaryOp::Ne => Ok(EvalSignal::Value(Value::Bool(left != right))),
        CoreBinaryOp::Lt => {
            cmp_numeric(&left, &right, std::cmp::Ordering::is_lt)
        }
        CoreBinaryOp::Le => {
            cmp_numeric(&left, &right, std::cmp::Ordering::is_le)
        }
        CoreBinaryOp::Gt => {
            cmp_numeric(&left, &right, std::cmp::Ordering::is_gt)
        }
        CoreBinaryOp::Ge => {
            cmp_numeric(&left, &right, std::cmp::Ordering::is_ge)
        }
        CoreBinaryOp::Get => eval_get_shared(left, right, false, false, None),
        CoreBinaryOp::Concat => match (left, right) {
            (Value::List(a), Value::List(b)) => {
                concat_slices(&a, &b, Value::List, vm)
            }
            (Value::S(a), Value::S(b)) => concat_slices(&a, &b, Value::S, vm),
            (Value::Dcs(a), Value::Dcs(b)) => {
                concat_slices(&a, &b, Value::Dcs, vm)
            }
            _ => bail!("Invalid operands for 'concat'"),
        },
        CoreBinaryOp::BufFill
        | CoreBinaryOp::BufPush
        | CoreBinaryOp::BufExtend => eval_core_buf_binary(vm, op, left, right),
        CoreBinaryOp::And | CoreBinaryOp::Or => {
            bail!(
                "Short-circuit logical operators must be evaluated directly in eval_expr"
            );
        }
    }
}

fn eval_total_arith(
    vm: &StageLWorkspaceVm,
    kernel: FixedBinKernel,
    op_name: &str,
    v1: Value,
    v2: Value,
    span: Span,
    f_int: fn(Integer, Integer) -> Integer,
    f_rat: fn(Rational, Rational) -> Rational,
) -> Result<EvalSignal> {
    match (v1, v2) {
        (Value::Int(a), Value::Int(b)) => {
            let res = f_int(a, b);
            if let Some(fault) = vm.check_int_limit(&res, op_name) {
                return Ok(fault);
            }
            Ok(EvalSignal::Value(Value::Int(res)))
        }
        (Value::Rat(a), Value::Rat(b)) => {
            let res = f_rat(a, b);
            if let Some(fault) = vm.check_rat_limit(&res, op_name) {
                return Ok(fault);
            }
            Ok(EvalSignal::Value(Value::Rat(res)))
        }
        (l, r) => {
            dispatch_fixed_bin(kernel, l, r, false, true, op_name, Some(span))
        }
    }
}

fn eval_total_div(
    vm: &StageLWorkspaceVm,
    v1: Value,
    v2: Value,
    span: Span,
) -> Result<EvalSignal> {
    match (v1, v2) {
        (Value::Int(a), Value::Int(b)) => {
            if b == Integer::ZERO {
                return Ok(err_res(StageRErrorTag::DivisionByZero));
            }
            let q = a.div(b);
            if let Some(fault) = vm.check_int_limit(&q, "div") {
                return Ok(fault);
            }
            Ok(ok_res(Value::Int(q)))
        }
        (Value::Rat(a), Value::Rat(b)) => {
            if b == Rational::ZERO {
                return Ok(err_res(StageRErrorTag::DivisionByZero));
            }
            let q = a.div(b);
            if let Some(fault) = vm.check_rat_limit(&q, "div") {
                return Ok(fault);
            }
            Ok(ok_res(Value::Rat(q)))
        }
        (l, r) => dispatch_fixed_bin(
            FixedBinKernel::CheckedDiv,
            l,
            r,
            false,
            true,
            "div",
            Some(span),
        ),
    }
}

fn eval_total_rem(
    vm: &StageLWorkspaceVm,
    euclid: bool,
    v1: Value,
    v2: Value,
    span: Span,
) -> Result<EvalSignal> {
    let op_name = if euclid { "mod" } else { "truncRem" };
    match (v1, v2) {
        (Value::Int(a), Value::Int(b)) => {
            if b == Integer::ZERO {
                return Ok(err_res(StageRErrorTag::DivisionByZero));
            }
            let r = if euclid {
                let mut rem = (&a).rem(&b);
                if rem < Integer::ZERO {
                    rem = rem.add(Integer::from(b.unsigned_abs_ref()));
                }
                rem
            } else {
                a.rem(b)
            };
            if let Some(fault) = vm.check_int_limit(&r, op_name) {
                return Ok(fault);
            }
            Ok(ok_res(Value::Int(r)))
        }
        (l, r) => dispatch_fixed_bin(
            if euclid {
                FixedBinKernel::TotalMod
            } else {
                FixedBinKernel::TotalTruncRem
            },
            l,
            r,
            false,
            true,
            op_name,
            Some(span),
        ),
    }
}

pub(super) fn eval_stager_binary(
    vm: &StageLWorkspaceVm,
    op: StageRBinaryOp,
    v1: Value,
    v2: Value,
    span: Span,
) -> Result<EvalSignal> {
    match op {
        StageRBinaryOp::TotalAdd => eval_total_arith(
            vm,
            FixedBinKernel::CheckedAdd,
            "add",
            v1,
            v2,
            span,
            Integer::add,
            Rational::add,
        ),
        StageRBinaryOp::TotalSub => eval_total_arith(
            vm,
            FixedBinKernel::CheckedSub,
            "sub",
            v1,
            v2,
            span,
            Integer::sub,
            Rational::sub,
        ),
        StageRBinaryOp::TotalMul => eval_total_arith(
            vm,
            FixedBinKernel::CheckedMul,
            "mul",
            v1,
            v2,
            span,
            Integer::mul,
            Rational::mul,
        ),
        StageRBinaryOp::TotalDiv => eval_total_div(vm, v1, v2, span),
        StageRBinaryOp::TotalMod => eval_total_rem(vm, true, v1, v2, span),
        StageRBinaryOp::TotalTruncRem => {
            eval_total_rem(vm, false, v1, v2, span)
        }
        StageRBinaryOp::TotalShl => {
            eval_shift_shared(true, v1, v2, true, true, Some(span))
        }
        StageRBinaryOp::TotalShr => {
            eval_shift_shared(false, v1, v2, true, true, Some(span))
        }
        StageRBinaryOp::TotalGet => {
            eval_get_shared(v1, v2, true, true, Some(span))
        }
        StageRBinaryOp::UnwrapOr => match v1 {
            Value::Option(Some(inner)) | Value::Result(Ok(inner)) => {
                Ok(EvalSignal::Value(*inner))
            }
            Value::Option(None) | Value::Result(Err(_)) => {
                Ok(EvalSignal::Value(v2))
            }
            other => bail!(
                "Invalid operand {other:?} for 'unwrapOr' at {span}"
            ),
        },
        StageRBinaryOp::OkOr => {
            let Value::Error(tag) = v2 else {
                bail!(
                    "'okOr' expected 'error' second operand, got {v2:?} at {span}"
                );
            };
            match v1 {
                Value::Option(Some(inner)) => {
                    Ok(EvalSignal::Value(Value::Result(Ok(inner))))
                }
                Value::Option(None) => Ok(err_res(tag)),
                other => {
                    bail!("Invalid first operand {other:?} for 'okOr' at {span}")
                }
            }
        }
        StageRBinaryOp::MakeRat => {
            let (Value::Int(num), Value::Int(den)) = (v1, v2) else {
                bail!("'makeRat' requires (int, int) operands at {span}");
            };
            if den == Integer::ZERO {
                return Ok(err_res(StageRErrorTag::DivisionByZero));
            }
            let rat = Rational::from_integers(num, den);
            if let Some(sig) = vm.check_rat_limit(&rat, "makeRat") {
                return Ok(sig);
            }
            Ok(ok_res(Value::Rat(rat)))
        }
        StageRBinaryOp::Explode => eval_explode(vm, v1, v2, span),
        StageRBinaryOp::Implode => eval_implode(vm, v1, v2, span),
    }
}

fn eval_slice_shared(
    seq: Value,
    start_val: Value,
    end_val: Value,
    allow_dcs: bool,
    total: bool,
    span: Option<Span>,
) -> Result<EvalSignal> {
    let start = start_val.as_u64_index("slice start")?;
    let end = end_val.as_u64_index("slice end")?;
    let (len, sliced) = match seq {
        Value::List(xs) => {
            if start > end || end > xs.len() {
                (xs.len(), None)
            } else {
                let sub = xs
                    .get(start..end)
                    .context("Validated slice bounds failed")?
                    .to_vec();
                (xs.len(), Some(Value::List(Rc::new(sub))))
            }
        }
        Value::Dcs(chars) if allow_dcs => {
            if start > end || end > chars.len() {
                (chars.len(), None)
            } else {
                let sub = chars
                    .get(start..end)
                    .context("Validated dcs slice bounds failed")?
                    .to_vec();
                (chars.len(), Some(Value::Dcs(Rc::new(sub))))
            }
        }
        other => {
            if let Some(sp) = span {
                bail!(
                    "Invalid sequence operand {other:?} for StageR 'slice' at {sp}"
                );
            }
            bail!("Invalid sequence operand {other:?} for 'slice'");
        }
    };
    if total {
        Ok(EvalSignal::Value(Value::Option(sliced.map(Box::new))))
    } else {
        match sliced {
            Some(v) => Ok(EvalSignal::Value(v)),
            None => Ok(EvalSignal::Fault(format!(
                "slice bounds [{start}..{end}) out of range for len {len}"
            ))),
        }
    }
}

fn eval_buf_set_shared(
    buf: Value,
    index: Value,
    new_val: Value,
    total: bool,
    span: Option<Span>,
) -> Result<EvalSignal> {
    let Value::Buf(b) = buf else {
        if let Some(sp) = span {
            bail!(
                "Invalid buffer operand {buf:?} for StageR 'bufSet' at {sp}"
            );
        }
        bail!("Invalid buffer operand {buf:?} for 'bufSet'");
    };
    let idx = index.as_u64_index("bufSet")?;
    let mut st = b.borrow_mut();
    if st.frozen {
        return Ok(if total {
            err_res(StageRErrorTag::FrozenBuffer)
        } else {
            EvalSignal::Fault(String::from("bufSet on frozen buffer"))
        });
    }
    let Some(slot) = st.elems.get_mut(idx) else {
        return Ok(if total {
            err_res(StageRErrorTag::OutOfBounds)
        } else {
            EvalSignal::Fault(format!("bufSet index {idx} out of bounds"))
        });
    };
    *slot = new_val;
    Ok(if total {
        ok_res(Value::Void)
    } else {
        EvalSignal::Value(Value::Void)
    })
}

pub(super) fn eval_ternary(
    op: CoreTernaryOp,
    v1: Value,
    v2: Value,
    v3: Value,
) -> Result<EvalSignal> {
    match op {
        CoreTernaryOp::Slice => {
            eval_slice_shared(v1, v2, v3, false, false, None)
        }
        CoreTernaryOp::BufSet => eval_buf_set_shared(v1, v2, v3, false, None),
    }
}

pub(super) fn eval_stager_ternary(
    op: StageRTernaryOp,
    v1: Value,
    v2: Value,
    v3: Value,
    span: Span,
) -> Result<EvalSignal> {
    match op {
        StageRTernaryOp::TotalSlice => {
            eval_slice_shared(v1, v2, v3, true, true, Some(span))
        }
        StageRTernaryOp::TotalBufSet => {
            eval_buf_set_shared(v1, v2, v3, true, Some(span))
        }
    }
}

fn finish_explode<T>(
    parts: Vec<Vec<T>>,
    vm: &StageLWorkspaceVm,
    wrap: impl FnMut(Vec<T>) -> Value,
) -> EvalSignal {
    if let Some(fault) = vm.check_seq_limit(parts.len(), "explode") {
        return fault;
    }
    let out: Vec<Value> = parts.into_iter().map(wrap).collect();
    EvalSignal::Value(Value::List(Rc::new(out)))
}

fn eval_explode(
    vm: &StageLWorkspaceVm,
    v1: Value,
    v2: Value,
    span: Span,
) -> Result<EvalSignal> {
    match (v1, v2) {
        (Value::S(sep), Value::S(inp)) => {
            let sep_units = s_atomic_units(&sep)?;
            let inp_units = s_atomic_units(&inp)?;
            let parts = explode_slice(&sep_units, &inp_units);
            Ok(finish_explode(parts, vm, |part| {
                let flat: Vec<DcChar> = part.into_iter().flatten().collect();
                Value::S(Rc::new(flat))
            }))
        }
        (Value::Dcs(sep), Value::Dcs(inp)) => {
            let parts = explode_slice(&sep, &inp);
            Ok(finish_explode(parts, vm, |part| {
                Value::Dcs(Rc::new(part))
            }))
        }
        (Value::List(sep), Value::List(inp)) => {
            let parts = explode_slice(&sep, &inp);
            Ok(finish_explode(parts, vm, |part| {
                Value::List(Rc::new(part))
            }))
        }
        (l, r) => bail!(
            "Invalid operands {l:?} and {r:?} for 'explode' at {span}"
        ),
    }
}

fn eval_implode_seq<T: Clone>(
    sep: &[T],
    parts: &[Value],
    extract: impl Fn(&Value) -> Option<&Rc<Vec<T>>>,
    wrap: fn(Rc<Vec<T>>) -> Value,
    expected_desc: &str,
    vm: &StageLWorkspaceVm,
    span: Span,
) -> Result<EvalSignal> {
    let mut vecs = Vec::with_capacity(parts.len());
    for p in parts {
        let Some(chars) = extract(p) else {
            bail!(
                "'implode' expected {expected_desc} parts, found {p:?} at {span}"
            );
        };
        vecs.push((**chars).clone());
    }
    let joined = implode_slices(sep, &vecs);
    if let Some(fault) = vm.check_seq_limit(joined.len(), "implode") {
        return Ok(fault);
    }
    Ok(EvalSignal::Value(wrap(Rc::new(joined))))
}

fn eval_implode(
    vm: &StageLWorkspaceVm,
    v1: Value,
    v2: Value,
    span: Span,
) -> Result<EvalSignal> {
    match (v1, v2) {
        (Value::S(sep), Value::List(parts)) => eval_implode_seq(
            &sep,
            &parts,
            |p| match p {
                Value::S(chars) => Some(chars),
                _ => None,
            },
            Value::S,
            "l/s",
            vm,
            span,
        ),
        (Value::Dcs(sep), Value::List(parts)) => eval_implode_seq(
            &sep,
            &parts,
            |p| match p {
                Value::Dcs(chars) => Some(chars),
                _ => None,
            },
            Value::Dcs,
            "l/dcs",
            vm,
            span,
        ),
        (Value::List(sep), Value::List(parts)) => eval_implode_seq(
            &sep,
            &parts,
            |p| match p {
                Value::List(xs) => Some(xs),
                _ => None,
            },
            Value::List,
            "l/l/T",
            vm,
            span,
        ),
        (l, r) => bail!(
            "Invalid operands {l:?} and {r:?} for 'implode' at {span}"
        ),
    }
}

fn numeric_value_type(val: &Value, span: Span) -> Result<StageLType> {
    match val {
        Value::U8(_) => Ok(StageLType::U8),
        Value::I32(_) => Ok(StageLType::I32),
        Value::U32(_) => Ok(StageLType::U32),
        Value::I64(_) => Ok(StageLType::I64),
        Value::U64(_) => Ok(StageLType::U64),
        Value::I128(_) => Ok(StageLType::I128),
        Value::U128(_) => Ok(StageLType::U128),
        Value::Int(_) => Ok(StageLType::Int),
        Value::Rat(_) => Ok(StageLType::Rat),
        other => {
            bail!("Expected numeric operand for 'conv', got {other:?} at {span}")
        }
    }
}

fn value_to_rational(val: Value, span: Span) -> Result<Rational> {
    match val {
        Value::U8(x) => Ok(Rational::from(x)),
        Value::I32(x) => Ok(Rational::from(x)),
        Value::U32(x) => Ok(Rational::from(x)),
        Value::I64(x) => Ok(Rational::from(x)),
        Value::U64(x) => Ok(Rational::from(x)),
        Value::I128(x) => Ok(Rational::from(x)),
        Value::U128(x) => Ok(Rational::from(x)),
        Value::Int(x) => Ok(Rational::from(x)),
        Value::Rat(r) => Ok(r),
        other => {
            bail!("Expected numeric operand for 'conv', got {other:?} at {span}")
        }
    }
}

fn value_to_exact_integer(
    val: Value,
    allow_wide: bool,
    span: Option<Span>,
) -> Result<Option<Integer>> {
    match val {
        Value::U8(x) => Ok(Some(Integer::from(x))),
        Value::I32(x) => Ok(Some(Integer::from(x))),
        Value::U32(x) => Ok(Some(Integer::from(x))),
        Value::I64(x) => Ok(Some(Integer::from(x))),
        Value::U64(x) => Ok(Some(Integer::from(x))),
        Value::I128(x) if allow_wide => Ok(Some(Integer::from(x))),
        Value::U128(x) if allow_wide => Ok(Some(Integer::from(x))),
        Value::Int(x) if allow_wide => Ok(Some(x)),
        Value::Rat(r) if allow_wide => Ok(Integer::try_from(r).ok()),
        other => {
            if let Some(sp) = span {
                bail!(
                    "Expected numeric operand for 'conv', got {other:?} at {sp}"
                );
            }
            bail!("Invalid source operand {other:?} for 'conv'");
        }
    }
}

fn convert_int_to_fixed_target(
    n: &Integer,
    target_ty: &StageLType,
    allow_128: bool,
    span: Option<Span>,
) -> Result<Option<Value>> {
    match target_ty {
        StageLType::U8 => Ok(u8::try_from(n).ok().map(Value::U8)),
        StageLType::I32 => Ok(i32::try_from(n).ok().map(Value::I32)),
        StageLType::U32 => Ok(u32::try_from(n).ok().map(Value::U32)),
        StageLType::I64 => Ok(i64::try_from(n).ok().map(Value::I64)),
        StageLType::U64 => Ok(u64::try_from(n).ok().map(Value::U64)),
        StageLType::I128 if allow_128 => {
            Ok(i128::try_from(n).ok().map(Value::I128))
        }
        StageLType::U128 if allow_128 => {
            Ok(u128::try_from(n).ok().map(Value::U128))
        }
        other => {
            if let Some(sp) = span {
                bail!(
                    "Invalid target type '{other}' for StageR 'conv' at {sp}"
                );
            }
            bail!("Invalid target type '{other}' for 'conv'");
        }
    }
}

pub(super) fn eval_conv(val: Value, target_ty: &StageLType) -> Result<EvalSignal> {
    let n = value_to_exact_integer(val, false, None)?
        .context("Core integer operand must be integral")?;
    let converted = convert_int_to_fixed_target(&n, target_ty, false, None)?;
    match converted {
        Some(v) => Ok(EvalSignal::Value(v)),
        None => Ok(EvalSignal::Fault(format!(
            "conv out of range converting {n} to {target_ty}"
        ))),
    }
}

pub(super) fn eval_total_conv(
    vm: &StageLWorkspaceVm,
    val: Value,
    target_ty: &StageLType,
    span: Span,
) -> Result<EvalSignal> {
    let src_ty = numeric_value_type(&val, span)?;
    let widening = is_stager_widening_conv(&src_ty, target_ty);

    if *target_ty == StageLType::Rat {
        let rat = value_to_rational(val, span)?;
        if let Some(fault) = vm.check_rat_limit(&rat, "conv") {
            return Ok(fault);
        }
        return if widening {
            Ok(EvalSignal::Value(Value::Rat(rat)))
        } else {
            Ok(ok_res(Value::Rat(rat)))
        };
    }

    let converted = match value_to_exact_integer(val, true, Some(span))? {
        None => None,
        Some(n) => {
            if *target_ty == StageLType::Int {
                if let Some(fault) = vm.check_int_limit(&n, "conv") {
                    return Ok(fault);
                }
                Some(Value::Int(n))
            } else {
                convert_int_to_fixed_target(&n, target_ty, true, Some(span))?
            }
        }
    };

    if widening {
        let out_val = converted.with_context(|| {
            format!(
                "Widening StageR 'conv' from '{src_ty}' to '{target_ty}' unexpectedly failed at {span}"
            )
        })?;
        Ok(EvalSignal::Value(out_val))
    } else {
        Ok(match converted {
            Some(v) => ok_res(v),
            None => err_res(StageRErrorTag::Overflow),
        })
    }
}

pub(super) fn eval_bitcast(
    val: Value,
    target_ty: &StageLType,
) -> Result<EvalSignal> {
    let out = match (val, target_ty) {
        (Value::U8(x), StageLType::U8) => Value::U8(x),
        (Value::I32(x), StageLType::I32) => Value::I32(x),
        (Value::I32(x), StageLType::U32) => {
            Value::U32(u32::from_le_bytes(x.to_le_bytes()))
        }
        (Value::U32(x), StageLType::U32) => Value::U32(x),
        (Value::U32(x), StageLType::I32) => {
            Value::I32(i32::from_le_bytes(x.to_le_bytes()))
        }
        (Value::I64(x), StageLType::I64) => Value::I64(x),
        (Value::I64(x), StageLType::U64) => {
            Value::U64(u64::from_le_bytes(x.to_le_bytes()))
        }
        (Value::U64(x), StageLType::U64) => Value::U64(x),
        (Value::U64(x), StageLType::I64) => {
            Value::I64(i64::from_le_bytes(x.to_le_bytes()))
        }
        (Value::I128(x), StageLType::I128) => Value::I128(x),
        (Value::I128(x), StageLType::U128) => {
            Value::U128(u128::from_le_bytes(x.to_le_bytes()))
        }
        (Value::U128(x), StageLType::U128) => Value::U128(x),
        (Value::U128(x), StageLType::I128) => {
            Value::I128(i128::from_le_bytes(x.to_le_bytes()))
        }
        (src, dst) => bail!("Invalid bitcast from {src:?} to {dst}"),
    };
    Ok(EvalSignal::Value(out))
}
