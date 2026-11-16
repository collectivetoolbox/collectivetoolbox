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
//! for StageL 3 and StageR 3.
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

use malachite::base::num::basic::traits::Zero;
use malachite::{Integer, Rational};

use super::super::ast::{
    BinaryOp, Span, StageLType, StageRErrorTag, TernaryOp, UnaryOp,
};
use super::super::stager_strings::{
    dcs_from_s, explode_slice, implode_slices, s_atomic_units, s_from_dcs,
    validate_and_measure_s_byte_len,
};
use super::super::validator::is_stager_widening_conv;
use super::vm::{EvalSignal, StageLWorkspaceVm};
use super::{BufStorage, Value};
use crate::DcChar;

fn ok_res(val: Value) -> EvalSignal {
    EvalSignal::Value(Value::Result(Ok(Box::new(val))))
}

fn err_res(tag: StageRErrorTag) -> EvalSignal {
    EvalSignal::Value(Value::Result(Err(tag)))
}

/// Formats checked arithmetic/shift/negation results as `Value`/`Fault` in
/// trapping StageL 3 or `result/T` in total StageR 3.
fn wrap_checked(
    res: Option<Value>,
    total: bool,
    err_tag: StageRErrorTag,
    fault_msg: impl FnOnce() -> String,
) -> EvalSignal {
    match (res, total) {
        (Some(v), true) => ok_res(v),
        (None, true) => err_res(err_tag),
        (Some(v), false) => EvalSignal::Value(v),
        (None, false) => EvalSignal::Fault(fault_msg()),
    }
}

macro_rules! fixed_pair_arms {
    ($left:expr, $right:expr, $allow_128:expr, $span:expr, $op_name:expr, $handler:ident) => {
        match ($left, $right) {
            (Value::U8(a), Value::U8(b)) => $handler!(U8, false, a, b),
            (Value::I32(a), Value::I32(b)) => $handler!(I32, true, a, b),
            (Value::U32(a), Value::U32(b)) => $handler!(U32, false, a, b),
            (Value::I64(a), Value::I64(b)) => $handler!(I64, true, a, b),
            (Value::U64(a), Value::U64(b)) => $handler!(U64, false, a, b),
            (Value::I128(a), Value::I128(b)) if $allow_128 => {
                $handler!(I128, true, a, b)
            }
            (Value::U128(a), Value::U128(b)) if $allow_128 => {
                $handler!(U128, false, a, b)
            }
            (l, r) => {
                let span_opt: Option<Span> = $span;
                if let Some(sp) = span_opt {
                    bail!(
                        "Mismatched operands {l:?} vs {r:?} for StageR '{}' at {sp}",
                        $op_name
                    );
                }
                bail!("Mismatched types {l:?} vs {r:?} for '{}'", $op_name);
            }
        }
    };
}

#[derive(Clone, Copy)]
enum BitUnaryOp {
    BitNot,
    Clz,
    Ctz,
    Popcnt,
}

fn eval_fixed_unary_bit(op: BitUnaryOp, val: Value) -> Result<EvalSignal> {
    macro_rules! bit_unary_arms {
        ($($V:ident),+) => {
            match val {
                $(Value::$V(x) => match op {
                    BitUnaryOp::BitNot => Value::$V(!x),
                    BitUnaryOp::Clz => Value::U32(x.leading_zeros()),
                    BitUnaryOp::Ctz => Value::U32(x.trailing_zeros()),
                    BitUnaryOp::Popcnt => Value::U32(x.count_ones()),
                },)+
                other => {
                    let name = match op {
                        BitUnaryOp::BitNot => "bitNot",
                        BitUnaryOp::Clz => "clz",
                        BitUnaryOp::Ctz => "ctz",
                        BitUnaryOp::Popcnt => "popcnt",
                    };
                    bail!("Invalid operand {other:?} for '{name}'");
                }
            }
        };
    }
    Ok(EvalSignal::Value(bit_unary_arms!(
        U8, I32, U32, I64, U64, I128, U128
    )))
}

fn eval_neg(
    vm: &StageLWorkspaceVm,
    total: bool,
    val: Value,
    span: Span,
) -> Result<EvalSignal> {
    macro_rules! neg_arm {
        ($x:expr, $V:ident, $ty_name:literal) => {
            Ok(wrap_checked(
                $x.checked_neg().map(Value::$V),
                total,
                StageRErrorTag::Overflow,
                || format!("{} neg overflow", $ty_name),
            ))
        };
    }
    match val {
        Value::I32(x) => neg_arm!(x, I32, "i32"),
        Value::I64(x) => neg_arm!(x, I64, "i64"),
        Value::I128(x) if total => neg_arm!(x, I128, "i128"),
        Value::Int(x) if total => {
            let neg = x.neg();
            if let Some(sig) = vm.check_int_limit(&neg, "neg") {
                return Ok(sig);
            }
            Ok(EvalSignal::Value(Value::Int(neg)))
        }
        Value::Rat(r) if total => {
            let neg = r.neg();
            if let Some(sig) = vm.check_rat_limit(&neg, "neg") {
                return Ok(sig);
            }
            Ok(EvalSignal::Value(Value::Rat(neg)))
        }
        other => {
            if total {
                bail!("Invalid operand {other:?} for StageR 'neg' at {span}");
            }
            bail!("Invalid operand {other:?} for 'neg'");
        }
    }
}

pub(super) fn eval_unary(
    vm: &StageLWorkspaceVm,
    op: UnaryOp,
    val: Value,
    span: Span,
) -> Result<EvalSignal> {
    match op {
        UnaryOp::Neg | UnaryOp::TotalNeg => {
            eval_neg(vm, op == UnaryOp::TotalNeg, val, span)
        }
        UnaryOp::BitNot => eval_fixed_unary_bit(BitUnaryOp::BitNot, val),
        UnaryOp::Clz => eval_fixed_unary_bit(BitUnaryOp::Clz, val),
        UnaryOp::Ctz => eval_fixed_unary_bit(BitUnaryOp::Ctz, val),
        UnaryOp::Popcnt => eval_fixed_unary_bit(BitUnaryOp::Popcnt, val),
        UnaryOp::Not => match val {
            Value::Bool(b) => Ok(EvalSignal::Value(Value::Bool(!b))),
            other => bail!("Invalid operand {other:?} for 'not'"),
        },
        UnaryOp::Len => {
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
        UnaryOp::BufFreeze => {
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
        UnaryOp::IsSome | UnaryOp::IsNone => {
            let Value::Option(opt) = val else {
                bail!(
                    "'{}' expected option/T, got {val:?} at {span}",
                    op.keyword()
                );
            };
            Ok(EvalSignal::Value(Value::Bool(if op == UnaryOp::IsSome {
                opt.is_some()
            } else {
                opt.is_none()
            })))
        }
        UnaryOp::IsOk | UnaryOp::IsErr => {
            let Value::Result(res) = val else {
                bail!(
                    "'{}' expected result/T, got {val:?} at {span}",
                    op.keyword()
                );
            };
            Ok(EvalSignal::Value(Value::Bool(if op == UnaryOp::IsOk {
                res.is_ok()
            } else {
                res.is_err()
            })))
        }
        UnaryOp::ByteLen => {
            let Value::S(chars) = val else {
                bail!("'byteLen' expected 's', got {val:?} at {span}");
            };
            let byte_len = validate_and_measure_s_byte_len(&chars)?;
            Ok(EvalSignal::Value(Value::U64(byte_len)))
        }
        UnaryOp::DcFromU128 => {
            let Value::U128(raw) = val else {
                bail!("'dcFromU128' expected 'u128', got {val:?} at {span}");
            };
            Ok(EvalSignal::Value(Value::Dc(DcChar::from_u128(raw))))
        }
        UnaryOp::U128FromDc => {
            let Value::Dc(dc) = val else {
                bail!("'u128FromDc' expected 'dc', got {val:?} at {span}");
            };
            Ok(EvalSignal::Value(Value::U128(dc.to_u128())))
        }
        UnaryOp::DcsFromS => {
            let Value::S(chars) = val else {
                bail!("'dcsFromS' expected 's', got {val:?} at {span}");
            };
            match dcs_from_s(&chars) {
                Ok(dcs) => Ok(ok_res(Value::Dcs(Rc::new(dcs)))),
                Err(()) => Ok(err_res(StageRErrorTag::InvalidEncoding)),
            }
        }
        UnaryOp::SFromDcs => {
            let Value::Dcs(chars) = val else {
                bail!("'sFromDcs' expected 'dcs', got {val:?} at {span}");
            };
            Ok(EvalSignal::Value(Value::S(Rc::new(s_from_dcs(&chars)))))
        }
    }
}

fn eval_shift_shared(
    is_left: bool,
    val: Value,
    count: Value,
    total: bool,
    span: Option<Span>,
) -> Result<EvalSignal> {
    let Value::U32(cnt) = count else {
        if let Some(sp) = span {
            bail!("Shift count must be u32, got {count:?} at {sp}");
        }
        bail!("Shift count must be u32, got {count:?}");
    };
    macro_rules! shift_arm {
        ($V:ident, $x:expr) => {{
            let shifted = if is_left {
                $x.checked_shl(cnt)
            } else {
                $x.checked_shr(cnt)
            };
            Ok(wrap_checked(
                shifted.map(Value::$V),
                total,
                StageRErrorTag::Overflow,
                || format!("shift count {cnt} out of range"),
            ))
        }};
    }
    match val {
        Value::U8(x) => shift_arm!(U8, x),
        Value::I32(x) => shift_arm!(I32, x),
        Value::U32(x) => shift_arm!(U32, x),
        Value::I64(x) => shift_arm!(I64, x),
        Value::U64(x) => shift_arm!(U64, x),
        Value::I128(x) if total => shift_arm!(I128, x),
        Value::U128(x) if total => shift_arm!(U128, x),
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
    macro_rules! rotate_arms {
        ($($V:ident),+) => {
            match val {
                $(Value::$V(x) => Value::$V(if is_left {
                    x.rotate_left(cnt)
                } else {
                    x.rotate_right(cnt)
                }),)+
                other => bail!("Invalid operand {other:?} for rotate"),
            }
        };
    }
    Ok(EvalSignal::Value(rotate_arms!(U8, U32, U64, U128)))
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

#[derive(Clone, Copy)]
enum BufBinOp {
    Fill,
    Push,
    Extend,
}

fn eval_buf_binary(
    vm: &StageLWorkspaceVm,
    op: BufBinOp,
    left: Value,
    right: Value,
) -> Result<EvalSignal> {
    match op {
        BufBinOp::Fill => {
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
        BufBinOp::Push => {
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
        BufBinOp::Extend => {
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
    }
}

#[derive(Clone, Copy)]
enum ArithBinOp {
    Add,
    Sub,
    Mul,
}

impl ArithBinOp {
    const fn keyword(self) -> &'static str {
        match self {
            Self::Add => "add",
            Self::Sub => "sub",
            Self::Mul => "mul",
        }
    }
}

fn eval_add_sub_mul(
    vm: &StageLWorkspaceVm,
    op: ArithBinOp,
    total: bool,
    left: Value,
    right: Value,
    span: Span,
) -> Result<EvalSignal> {
    let op_name = op.keyword();
    match (left, right) {
        (Value::Int(a), Value::Int(b)) if total => {
            let res = match op {
                ArithBinOp::Add => a.add(b),
                ArithBinOp::Sub => a.sub(b),
                ArithBinOp::Mul => a.mul(b),
            };
            if let Some(fault) = vm.check_int_limit(&res, op_name) {
                return Ok(fault);
            }
            Ok(EvalSignal::Value(Value::Int(res)))
        }
        (Value::Rat(a), Value::Rat(b)) if total => {
            let res = match op {
                ArithBinOp::Add => a.add(b),
                ArithBinOp::Sub => a.sub(b),
                ArithBinOp::Mul => a.mul(b),
            };
            if let Some(fault) = vm.check_rat_limit(&res, op_name) {
                return Ok(fault);
            }
            Ok(EvalSignal::Value(Value::Rat(res)))
        }
        (l, r) => {
            macro_rules! checked_arm {
                ($V:ident, $_signed:expr, $a:expr, $b:expr) => {{
                    let res = match op {
                        ArithBinOp::Add => $a.checked_add($b),
                        ArithBinOp::Sub => $a.checked_sub($b),
                        ArithBinOp::Mul => $a.checked_mul($b),
                    };
                    Ok(wrap_checked(
                        res.map(Value::$V),
                        total,
                        StageRErrorTag::Overflow,
                        || format!("checked arithmetic fault in '{op_name}'"),
                    ))
                }};
            }
            let sp_opt = total.then_some(span);
            fixed_pair_arms!(l, r, total, sp_opt, op_name, checked_arm)
        }
    }
}

#[derive(Clone, Copy)]
enum WrapOrBitOp {
    WrapAdd,
    WrapSub,
    WrapMul,
    BitAnd,
    BitOr,
    BitXor,
}

impl WrapOrBitOp {
    const fn keyword(self) -> &'static str {
        match self {
            Self::WrapAdd => "wrapAdd",
            Self::WrapSub => "wrapSub",
            Self::WrapMul => "wrapMul",
            Self::BitAnd => "bitAnd",
            Self::BitOr => "bitOr",
            Self::BitXor => "bitXor",
        }
    }
}

fn eval_wrap_or_bit(
    op: WrapOrBitOp,
    left: Value,
    right: Value,
) -> Result<EvalSignal> {
    macro_rules! wrap_arm {
        ($V:ident, $_signed:expr, $a:expr, $b:expr) => {
            Ok(EvalSignal::Value(Value::$V(match op {
                WrapOrBitOp::WrapAdd => $a.wrapping_add($b),
                WrapOrBitOp::WrapSub => $a.wrapping_sub($b),
                WrapOrBitOp::WrapMul => $a.wrapping_mul($b),
                WrapOrBitOp::BitAnd => $a & $b,
                WrapOrBitOp::BitOr => $a | $b,
                WrapOrBitOp::BitXor => $a ^ $b,
            })))
        };
    }
    fixed_pair_arms!(left, right, true, None, op.keyword(), wrap_arm)
}

fn eval_div(
    vm: &StageLWorkspaceVm,
    total: bool,
    left: Value,
    right: Value,
    span: Span,
) -> Result<EvalSignal> {
    match (left, right) {
        (Value::Int(a), Value::Int(b)) if total => {
            if b == Integer::ZERO {
                return Ok(err_res(StageRErrorTag::DivisionByZero));
            }
            let q = a.div(b);
            if let Some(fault) = vm.check_int_limit(&q, "div") {
                return Ok(fault);
            }
            Ok(ok_res(Value::Int(q)))
        }
        (Value::Rat(a), Value::Rat(b)) if total => {
            if b == Rational::ZERO {
                return Ok(err_res(StageRErrorTag::DivisionByZero));
            }
            let q = a.div(b);
            if let Some(fault) = vm.check_rat_limit(&q, "div") {
                return Ok(fault);
            }
            Ok(ok_res(Value::Rat(q)))
        }
        (l, r) => {
            macro_rules! div_arm {
                ($V:ident, $_signed:expr, $a:expr, $b:expr) => {{
                    if $b == 0 {
                        return Ok(if total {
                            err_res(StageRErrorTag::DivisionByZero)
                        } else {
                            EvalSignal::Fault(String::from(
                                "checked arithmetic fault in 'div'",
                            ))
                        });
                    }
                    Ok(wrap_checked(
                        $a.checked_div($b).map(Value::$V),
                        total,
                        StageRErrorTag::Overflow,
                        || String::from("checked arithmetic fault in 'div'"),
                    ))
                }};
            }
            let sp_opt = total.then_some(span);
            fixed_pair_arms!(l, r, total, sp_opt, "div", div_arm)
        }
    }
}

#[derive(Clone, Copy)]
enum RemMode {
    CoreTruncRem,
    TotalEuclidMod,
    TotalTruncRem,
}

impl RemMode {
    const fn keyword(self) -> &'static str {
        match self {
            Self::CoreTruncRem => "rem",
            Self::TotalEuclidMod => "mod",
            Self::TotalTruncRem => "truncRem",
        }
    }
}

fn eval_rem(
    vm: &StageLWorkspaceVm,
    mode: RemMode,
    left: Value,
    right: Value,
    span: Span,
) -> Result<EvalSignal> {
    let op_name = mode.keyword();
    let total = !matches!(mode, RemMode::CoreTruncRem);
    if total && let (Value::Int(a), Value::Int(b)) = (&left, &right) {
        if *b == Integer::ZERO {
            return Ok(err_res(StageRErrorTag::DivisionByZero));
        }
        let r = if matches!(mode, RemMode::TotalEuclidMod) {
            let mut rem = a.rem(b);
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
        return Ok(ok_res(Value::Int(r)));
    }
    macro_rules! rem_arm {
        ($V:ident, $signed:expr, $a:expr, $b:expr) => {
            if !total {
                Ok(wrap_checked(
                    $a.checked_rem($b).map(Value::$V),
                    false,
                    StageRErrorTag::Overflow,
                    || format!("checked arithmetic fault in '{op_name}'"),
                ))
            } else {
                if $b == 0 {
                    return Ok(err_res(StageRErrorTag::DivisionByZero));
                }
                // Only signed `b == -1` satisfies `b.checked_add(1) == Some(0)`,
                // preventing MIN % -1 overflow cleanly across signed/unsigned types.
                if $signed && $b.checked_add(1) == Some(0) {
                    return Ok(ok_res(Value::$V(0)));
                }
                let out = match mode {
                    RemMode::TotalEuclidMod => $a.checked_rem_euclid($b),
                    RemMode::TotalTruncRem | RemMode::CoreTruncRem => {
                        $a.checked_rem($b)
                    }
                }
                .context("Remainder failed unexpectedly for divisor != 0, -1")?;
                Ok(ok_res(Value::$V(out)))
            }
        };
    }
    let sp_opt = total.then_some(span);
    fixed_pair_arms!(left, right, total, sp_opt, op_name, rem_arm)
}

pub(super) fn eval_binary(
    vm: &StageLWorkspaceVm,
    op: BinaryOp,
    left: Value,
    right: Value,
    span: Span,
) -> Result<EvalSignal> {
    match op {
        BinaryOp::Add => {
            eval_add_sub_mul(vm, ArithBinOp::Add, false, left, right, span)
        }
        BinaryOp::Sub => {
            eval_add_sub_mul(vm, ArithBinOp::Sub, false, left, right, span)
        }
        BinaryOp::Mul => {
            eval_add_sub_mul(vm, ArithBinOp::Mul, false, left, right, span)
        }
        BinaryOp::TotalAdd => {
            eval_add_sub_mul(vm, ArithBinOp::Add, true, left, right, span)
        }
        BinaryOp::TotalSub => {
            eval_add_sub_mul(vm, ArithBinOp::Sub, true, left, right, span)
        }
        BinaryOp::TotalMul => {
            eval_add_sub_mul(vm, ArithBinOp::Mul, true, left, right, span)
        }
        BinaryOp::Div => eval_div(vm, false, left, right, span),
        BinaryOp::TotalDiv => eval_div(vm, true, left, right, span),
        BinaryOp::Rem => {
            eval_rem(vm, RemMode::CoreTruncRem, left, right, span)
        }
        BinaryOp::TotalMod => {
            eval_rem(vm, RemMode::TotalEuclidMod, left, right, span)
        }
        BinaryOp::TotalTruncRem => {
            eval_rem(vm, RemMode::TotalTruncRem, left, right, span)
        }
        BinaryOp::WrapAdd => {
            eval_wrap_or_bit(WrapOrBitOp::WrapAdd, left, right)
        }
        BinaryOp::WrapSub => {
            eval_wrap_or_bit(WrapOrBitOp::WrapSub, left, right)
        }
        BinaryOp::WrapMul => {
            eval_wrap_or_bit(WrapOrBitOp::WrapMul, left, right)
        }
        BinaryOp::BitAnd => eval_wrap_or_bit(WrapOrBitOp::BitAnd, left, right),
        BinaryOp::BitOr => eval_wrap_or_bit(WrapOrBitOp::BitOr, left, right),
        BinaryOp::BitXor => eval_wrap_or_bit(WrapOrBitOp::BitXor, left, right),
        BinaryOp::Shl => eval_shift_shared(true, left, right, false, None),
        BinaryOp::Shr => eval_shift_shared(false, left, right, false, None),
        BinaryOp::TotalShl => {
            eval_shift_shared(true, left, right, true, Some(span))
        }
        BinaryOp::TotalShr => {
            eval_shift_shared(false, left, right, true, Some(span))
        }
        BinaryOp::Rotl => eval_rotate(true, left, right),
        BinaryOp::Rotr => eval_rotate(false, left, right),
        BinaryOp::Eq => Ok(EvalSignal::Value(Value::Bool(left == right))),
        BinaryOp::Ne => Ok(EvalSignal::Value(Value::Bool(left != right))),
        BinaryOp::Lt => cmp_numeric(&left, &right, std::cmp::Ordering::is_lt),
        BinaryOp::Le => cmp_numeric(&left, &right, std::cmp::Ordering::is_le),
        BinaryOp::Gt => cmp_numeric(&left, &right, std::cmp::Ordering::is_gt),
        BinaryOp::Ge => cmp_numeric(&left, &right, std::cmp::Ordering::is_ge),
        BinaryOp::Get => eval_get_shared(left, right, false, false, None),
        BinaryOp::TotalGet => {
            eval_get_shared(left, right, true, true, Some(span))
        }
        BinaryOp::Concat => match (left, right) {
            (Value::List(a), Value::List(b)) => {
                concat_slices(&a, &b, Value::List, vm)
            }
            (Value::S(a), Value::S(b)) => concat_slices(&a, &b, Value::S, vm),
            (Value::Dcs(a), Value::Dcs(b)) => {
                concat_slices(&a, &b, Value::Dcs, vm)
            }
            _ => bail!("Invalid operands for 'concat'"),
        },
        BinaryOp::BufFill => eval_buf_binary(vm, BufBinOp::Fill, left, right),
        BinaryOp::BufPush => eval_buf_binary(vm, BufBinOp::Push, left, right),
        BinaryOp::BufExtend => {
            eval_buf_binary(vm, BufBinOp::Extend, left, right)
        }
        BinaryOp::UnwrapOr => match left {
            Value::Option(Some(inner)) | Value::Result(Ok(inner)) => {
                Ok(EvalSignal::Value(*inner))
            }
            Value::Option(None) | Value::Result(Err(_)) => {
                Ok(EvalSignal::Value(right))
            }
            other => bail!(
                "Invalid operand {other:?} for 'unwrapOr' at {span}"
            ),
        },
        BinaryOp::OkOr => {
            let Value::Error(tag) = right else {
                bail!(
                    "'okOr' expected 'error' second operand, got {right:?} at {span}"
                );
            };
            match left {
                Value::Option(Some(inner)) => {
                    Ok(EvalSignal::Value(Value::Result(Ok(inner))))
                }
                Value::Option(None) => Ok(err_res(tag)),
                other => {
                    bail!("Invalid first operand {other:?} for 'okOr' at {span}")
                }
            }
        }
        BinaryOp::MakeRat => {
            let (Value::Int(num), Value::Int(den)) = (left, right) else {
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
        BinaryOp::Explode => eval_explode(vm, left, right, span),
        BinaryOp::Implode => eval_implode(vm, left, right, span),
        BinaryOp::And | BinaryOp::Or => {
            bail!(
                "Short-circuit logical operators must be evaluated directly in eval_expr"
            );
        }
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
    op: TernaryOp,
    v1: Value,
    v2: Value,
    v3: Value,
    span: Span,
) -> Result<EvalSignal> {
    match op {
        TernaryOp::Slice => {
            eval_slice_shared(v1, v2, v3, false, false, None)
        }
        TernaryOp::BufSet => eval_buf_set_shared(v1, v2, v3, false, None),
        TernaryOp::TotalSlice => {
            eval_slice_shared(v1, v2, v3, true, true, Some(span))
        }
        TernaryOp::TotalBufSet => {
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

pub(super) fn eval_conv(
    vm: &StageLWorkspaceVm,
    total: bool,
    val: Value,
    target_ty: &StageLType,
    span: Span,
) -> Result<EvalSignal> {
    if !total {
        let n = value_to_exact_integer(val, false, None)?
            .context("Core integer operand must be integral")?;
        let converted = convert_int_to_fixed_target(&n, target_ty, false, None)?;
        return Ok(match converted {
            Some(v) => EvalSignal::Value(v),
            None => EvalSignal::Fault(format!(
                "conv out of range converting {n} to {target_ty}"
            )),
        });
    }

    match (&val, target_ty) {
        (Value::List(items), StageLType::Dcs) => {
            let mut chars = Vec::with_capacity(items.len());
            for elem in items.iter() {
                let Value::Dc(dc) = elem else {
                    bail!("StageR 'conv' from 'l/dc' to 'dcs' encountered non-dc element {elem:?} at {span}");
                };
                chars.push(*dc);
            }
            return Ok(EvalSignal::Value(Value::Dcs(Rc::new(chars))));
        }
        (Value::Dcs(chars), StageLType::List(inner))
            if **inner == StageLType::Dc =>
        {
            let elems: Vec<Value> =
                chars.iter().copied().map(Value::Dc).collect();
            return Ok(EvalSignal::Value(Value::List(Rc::new(elems))));
        }
        (Value::Dcs(_), StageLType::Dcs) => {
            return Ok(EvalSignal::Value(val));
        }
        _ => {}
    }

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
