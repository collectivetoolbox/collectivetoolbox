//! StageR 3 expression evaluator and test runner extensions for the StageL 3 VM.
//!
//! Handles total checked/exact arithmetic, arbitrary-precision `int` and `rat`,
//! standalone `dc` / `dcs` / `s` string operations, `option/T` / `result/T` /
//! `error` constructors and pattern matching, early-return `/` propagation,
//! named-field `struct`s, `brand` / opaque `type` wrapping, and closed `enum`s.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};
use std::rc::Rc;

use malachite::base::num::arithmetic::traits::UnsignedAbs;
use malachite::base::num::basic::traits::Zero;
use malachite::base::num::logic::traits::SignificantBits;
use malachite::{Integer, Rational};

use super::ast::{
    Expr, MatchArm, Span, StageLType, StageRBinaryOp, StageRErrorTag,
    StageRTernaryOp, StageRUnaryOp, StructTypeRef, SymbolTarget,
};
use super::eval::{
    EvalConfig, EvalSignal, LexFrame, ModuleRuntime, StageLWorkspaceVm,
    TestSuiteReport, Value,
};
use super::parser::parse_sr3_module;
use super::stager_strings::{
    dcs_from_s, explode_slice, implode_slices, is_valid_dc_u128,
    s_atomic_units, s_from_dcs, validate_and_measure_s_byte_len,
};
use super::stager_validator::is_stager_widening_conv;
use super::validator::{
    ModuleInput, ValidatedWorkspace, parse_and_validate_stager_workspace,
    validate_module,
};
use crate::DcChar;

/// Parses, validates, and runs all inline `test` blocks inside a standalone
/// StageR 3 `.sr3` module.
pub fn run_sr3_module_inline_tests(source: &str) -> Result<TestSuiteReport> {
    run_sr3_module_inline_tests_with_config(source, EvalConfig::default())
}

/// Parses, validates, and runs all inline `test` blocks inside a standalone
/// StageR 3 `.sr3` module using explicit [`EvalConfig`] limits.
pub fn run_sr3_module_inline_tests_with_config(
    source: &str,
    config: EvalConfig,
) -> Result<TestSuiteReport> {
    let module = parse_sr3_module(source)?;
    let validated = validate_module(&module)?;
    let ws = ValidatedWorkspace {
        modules: vec![validated],
    };
    let vm = StageLWorkspaceVm::from_validated_workspace(&ws, config)?;
    vm.run_all_tests(&ws)
}

/// Parses, validates, and runs all inline `test` blocks across a StageR 3
/// `.sr3` workspace.
pub fn run_sr3_workspace_inline_tests(
    inputs: &[ModuleInput<'_>],
) -> Result<TestSuiteReport> {
    run_sr3_workspace_inline_tests_with_config(inputs, EvalConfig::default())
}

/// Parses, validates, and runs all inline `test` blocks across a StageR 3
/// `.sr3` workspace using explicit [`EvalConfig`] limits.
pub fn run_sr3_workspace_inline_tests_with_config(
    inputs: &[ModuleInput<'_>],
    config: EvalConfig,
) -> Result<TestSuiteReport> {
    let ws = parse_and_validate_stager_workspace(inputs)?;
    let vm = StageLWorkspaceVm::from_validated_workspace(&ws, config)?;
    vm.run_all_tests(&ws)
}

pub(super) fn eval_stager_expr(
    vm: &StageLWorkspaceVm,
    mod_idx: usize,
    expr: &Expr,
    frame: &LexFrame,
    in_tail: bool,
    depth: usize,
    quota: &mut u64,
) -> Result<EvalSignal> {
    let mod_rt = vm.modules.get(mod_idx).context("Invalid module idx")?;
    match expr {
        Expr::DctsString { value, .. } => {
            if let Some(sig) =
                check_seq_limit(vm, value.len(), "Dcts string literal")
            {
                return Ok(sig);
            }
            Ok(EvalSignal::Value(Value::S(Rc::new(value.clone()))))
        }
        Expr::DcLit { value, .. } => Ok(EvalSignal::Value(Value::Dc(*value))),
        Expr::NamedStructNew {
            target,
            fields,
            span,
        } => {
            let mut map = BTreeMap::new();
            for (fname, fexpr, _) in fields {
                let val =
                    match vm.eval_expr(mod_idx, fexpr, frame, false, depth, quota)? {
                        EvalSignal::Value(v) => v,
                        sig => return Ok(sig),
                    };
                map.insert(fname.clone(), val);
            }
            let type_key = resolve_named_struct_key(mod_rt, target, *span)?;
            Ok(EvalSignal::Value(Value::NamedStruct {
                type_key,
                fields: Rc::new(map),
            }))
        }
        Expr::EnumVariant {
            enum_ty,
            variant,
            payload,
            span,
        } => {
            let pval = if let Some(pexpr) = payload {
                match vm.eval_expr(mod_idx, pexpr, frame, false, depth, quota)? {
                    EvalSignal::Value(v) => Some(Box::new(v)),
                    sig => return Ok(sig),
                }
            } else {
                None
            };
            let (type_key, resolved_variant) =
                resolve_enum_variant_key(mod_rt, enum_ty, variant, *span)?;
            Ok(EvalSignal::Value(Value::EnumVariant {
                type_key,
                variant: resolved_variant,
                payload: pval,
            }))
        }
        Expr::OptionNone { .. } => Ok(EvalSignal::Value(Value::Option(None))),
        Expr::OptionSome { value, .. } => {
            let v = match vm.eval_expr(mod_idx, value, frame, false, depth, quota)? {
                EvalSignal::Value(v) => v,
                sig => return Ok(sig),
            };
            Ok(EvalSignal::Value(Value::Option(Some(Box::new(v)))))
        }
        Expr::ResultOk { value, .. } => {
            let v = match vm.eval_expr(mod_idx, value, frame, false, depth, quota)? {
                EvalSignal::Value(v) => v,
                sig => return Ok(sig),
            };
            Ok(EvalSignal::Value(Value::Result(Ok(Box::new(v)))))
        }
        Expr::ResultErr { error, span, .. } => {
            let ev = match vm.eval_expr(mod_idx, error, frame, false, depth, quota)? {
                EvalSignal::Value(v) => v,
                sig => return Ok(sig),
            };
            let Value::Error(tag) = ev else {
                bail!(
                    "Expected 'error' value in 'result/T:err', got {ev:?} at {span}"
                );
            };
            Ok(EvalSignal::Value(Value::Result(Err(tag))))
        }
        Expr::ErrorTag { tag, .. } => Ok(EvalSignal::Value(Value::Error(*tag))),
        Expr::NamedField { base, field, span } => {
            let bval = match vm.eval_expr(mod_idx, base, frame, false, depth, quota)? {
                EvalSignal::Value(v) => v,
                sig => return Ok(sig),
            };
            let Value::NamedStruct { fields, .. } = bval else {
                bail!(
                    "Expected named struct in 'field {field}', got {bval:?} at {span}"
                );
            };
            let val = fields.get(field).cloned().with_context(|| {
                format!("Missing field '{field}' on named struct at {span}")
            })?;
            Ok(EvalSignal::Value(val))
        }
        Expr::NamedWith {
            base,
            field,
            value,
            span,
        } => {
            let bval = match vm.eval_expr(mod_idx, base, frame, false, depth, quota)? {
                EvalSignal::Value(v) => v,
                sig => return Ok(sig),
            };
            let rval =
                match vm.eval_expr(mod_idx, value, frame, false, depth, quota)? {
                    EvalSignal::Value(v) => v,
                    sig => return Ok(sig),
                };
            let Value::NamedStruct { type_key, fields } = bval else {
                bail!(
                    "Expected named struct in 'with {field}', got {bval:?} at {span}"
                );
            };
            let mut next_fields = (*fields).clone();
            ensure!(
                next_fields.contains_key(field),
                "Named struct '{}' has no field '{field}' at {span}",
                type_key.1
            );
            next_fields.insert(field.clone(), rval);
            Ok(EvalSignal::Value(Value::NamedStruct {
                type_key,
                fields: Rc::new(next_fields),
            }))
        }
        Expr::WrapNominal {
            expr: inner,
            target_ty,
            span,
        } => {
            let val =
                match vm.eval_expr(mod_idx, inner, frame, false, depth, quota)? {
                    EvalSignal::Value(v) => v,
                    sig => return Ok(sig),
                };
            let type_key = resolve_nominal_type_key(mod_rt, target_ty, *span)?;
            Ok(EvalSignal::Value(Value::NominalWrap {
                type_key,
                inner: Box::new(val),
            }))
        }
        Expr::UnwrapNominal { expr: inner, span } => {
            let val =
                match vm.eval_expr(mod_idx, inner, frame, false, depth, quota)? {
                    EvalSignal::Value(v) => v,
                    sig => return Ok(sig),
                };
            let Value::NominalWrap { inner: wrapped, .. } = val else {
                bail!("Expected nominal wrapped value in 'unwrap', got {val:?} at {span}");
            };
            Ok(EvalSignal::Value(*wrapped))
        }
        Expr::Match {
            scrutinee,
            arms,
            fallback,
            span,
        } => eval_match(
            vm,
            mod_idx,
            scrutinee,
            arms,
            fallback.as_deref(),
            *span,
            frame,
            in_tail,
            depth,
            quota,
        ),
        Expr::Propagate { expr: inner, span } => {
            eval_propagate(vm, mod_idx, inner, *span, frame, depth, quota)
        }
        Expr::StageRUnary { op, operand, span } => {
            let v =
                match vm.eval_expr(mod_idx, operand, frame, false, depth, quota)? {
                    EvalSignal::Value(v) => v,
                    sig => return Ok(sig),
                };
            eval_stager_unary(vm, *op, v, *span)
        }
        Expr::StageRBinary {
            op,
            left,
            right,
            span,
        } => {
            let v1 =
                match vm.eval_expr(mod_idx, left, frame, false, depth, quota)? {
                    EvalSignal::Value(v) => v,
                    sig => return Ok(sig),
                };
            let v2 =
                match vm.eval_expr(mod_idx, right, frame, false, depth, quota)? {
                    EvalSignal::Value(v) => v,
                    sig => return Ok(sig),
                };
            eval_stager_binary(vm, *op, v1, v2, *span)
        }
        Expr::StageRTernary {
            op,
            first,
            second,
            third,
            span,
        } => {
            let v1 =
                match vm.eval_expr(mod_idx, first, frame, false, depth, quota)? {
                    EvalSignal::Value(v) => v,
                    sig => return Ok(sig),
                };
            let v2 =
                match vm.eval_expr(mod_idx, second, frame, false, depth, quota)? {
                    EvalSignal::Value(v) => v,
                    sig => return Ok(sig),
                };
            let v3 =
                match vm.eval_expr(mod_idx, third, frame, false, depth, quota)? {
                    EvalSignal::Value(v) => v,
                    sig => return Ok(sig),
                };
            eval_stager_ternary(*op, v1, v2, v3, *span)
        }
        Expr::TotalConv {
            expr: inner,
            target_ty,
            span,
        } => {
            let v =
                match vm.eval_expr(mod_idx, inner, frame, false, depth, quota)? {
                    EvalSignal::Value(v) => v,
                    sig => return Ok(sig),
                };
            eval_total_conv(vm, v, target_ty, *span)
        }
        other => {
            bail!("Unexpected non-StageR expression {other:?} dispatched to StageR evaluator")
        }
    }
}

fn resolve_named_struct_key(
    mod_rt: &ModuleRuntime,
    target: &SymbolTarget,
    span: Span,
) -> Result<(Option<String>, String)> {
    if let Some(key) = mod_rt.resolved_named_structs.get(&(span.line, span.col)) {
        return Ok(key.clone());
    }
    match target {
        SymbolTarget::Ident(name) => {
            mod_rt.local_structs.get(name).cloned().with_context(|| {
                format!("Unknown named struct '{name}' in 'structNew' at {span}")
            })
        }
        SymbolTarget::Dc(dc) => {
            mod_rt.dc_structs.get(dc).cloned().with_context(|| {
                format!(
                    "Unknown named struct '{dc:?}' in 'structNew' at {span}"
                )
            })
        }
    }
}

fn resolve_enum_variant_key(
    mod_rt: &ModuleRuntime,
    enum_ty: &StructTypeRef,
    variant: &str,
    span: Span,
) -> Result<((Option<String>, String), String)> {
    if let Some(resolved) =
        mod_rt.resolved_enum_variants.get(&(span.line, span.col))
    {
        return Ok(resolved.clone());
    }
    let type_key = match enum_ty {
        StructTypeRef::Canonical { namespace, name } => {
            (namespace.clone(), name.clone())
        }
        StructTypeRef::Ident(name) => {
            mod_rt.local_nominals.get(name).cloned().with_context(|| {
                format!("Unknown enum type '{name}' at {span}")
            })?
        }
        StructTypeRef::Dc(dc) => {
            mod_rt.dc_nominals.get(dc).cloned().with_context(|| {
                format!("Unknown enum type '{dc:?}' at {span}")
            })?
        }
    };
    Ok((type_key, variant.to_owned()))
}

fn resolve_nominal_type_key(
    mod_rt: &ModuleRuntime,
    target_ty: &StageLType,
    span: Span,
) -> Result<(Option<String>, String)> {
    match target_ty {
        StageLType::Nominal(sref) | StageLType::Struct(sref) => match sref {
            StructTypeRef::Canonical { namespace, name } => {
                Ok((namespace.clone(), name.clone()))
            }
            StructTypeRef::Ident(name) => {
                mod_rt.local_nominals.get(name).cloned().with_context(|| {
                    format!("Unknown nominal type '{name}' in 'wrap' at {span}")
                })
            }
            StructTypeRef::Dc(dc) => {
                mod_rt.dc_nominals.get(dc).cloned().with_context(|| {
                    format!("Unknown nominal type '{dc:?}' in 'wrap' at {span}")
                })
            }
        },
        other => bail!(
            "Invalid 'wrap' target type '{other}' (expected nominal brand or opaque type) at {span}"
        ),
    }
}

#[allow(
    clippy::too_many_arguments,
    reason = "Forwards VM evaluation context and tail-position flag across match arms"
)]
fn eval_match(
    vm: &StageLWorkspaceVm,
    mod_idx: usize,
    scrutinee: &Expr,
    arms: &[MatchArm],
    fallback: Option<&Expr>,
    span: Span,
    frame: &LexFrame,
    in_tail: bool,
    depth: usize,
    quota: &mut u64,
) -> Result<EvalSignal> {
    let sval =
        match vm.eval_expr(mod_idx, scrutinee, frame, false, depth, quota)? {
            EvalSignal::Value(v) => v,
            sig => return Ok(sig),
        };
    let (variant_name, payload_opt): (String, Option<Value>) = match sval {
        Value::Option(None) => (String::from("none"), None),
        Value::Option(Some(inner)) => (String::from("some"), Some(*inner)),
        Value::Result(Ok(inner)) => (
            String::from("ok"),
            if *inner == Value::Void {
                None
            } else {
                Some(*inner)
            },
        ),
        Value::Result(Err(tag)) => {
            (String::from("err"), Some(Value::Error(tag)))
        }
        Value::Error(tag) => (tag.keyword().to_owned(), None),
        Value::EnumVariant {
            variant, payload, ..
        } => (variant, payload.map(|b| *b)),
        other => bail!(
            "Invalid scrutinee value {other:?} for 'match' at {span}"
        ),
    };

    if let Some(arm) = arms.iter().find(|a| a.variant == variant_name) {
        let mut arm_frame = LexFrame::new(Some(Rc::new(frame.clone())));
        match (&arm.binding, payload_opt) {
            (Some((_, bname, _)), Some(pval)) => {
                arm_frame
                    .vars
                    .insert(bname.clone(), Rc::new(RefCell::new(Some(pval))));
            }
            (None, None) => {}
            (Some((_, bname, bsp)), None) => {
                bail!(
                    "Unit variant '{variant_name}' cannot bind payload '{bname}' at {bsp}"
                );
            }
            (None, Some(_)) => {
                bail!(
                    "Payload variant '{variant_name}' missing binding in 'match' at {}",
                    arm.span
                );
            }
        }
        return vm.eval_expr(
            mod_idx,
            &arm.body,
            &arm_frame,
            in_tail,
            depth,
            quota,
        );
    }

    if let Some(fb) = fallback {
        return vm.eval_expr(mod_idx, fb, frame, in_tail, depth, quota);
    }

    bail!(
        "Non-exhaustive runtime 'match' for variant '{variant_name}' at {span}"
    )
}

fn eval_propagate(
    vm: &StageLWorkspaceVm,
    mod_idx: usize,
    inner: &Expr,
    span: Span,
    frame: &LexFrame,
    depth: usize,
    quota: &mut u64,
) -> Result<EvalSignal> {
    let val = match vm.eval_expr(mod_idx, inner, frame, false, depth, quota)? {
        EvalSignal::Value(v) => v,
        sig => return Ok(sig),
    };
    match val {
        Value::Option(Some(inner_val)) | Value::Result(Ok(inner_val)) => {
            Ok(EvalSignal::Value(*inner_val))
        }
        Value::Result(Err(tag)) => {
            Ok(EvalSignal::Return(Value::Result(Err(tag))))
        }
        Value::Option(None) => match frame.return_ty.as_ref() {
            Some(StageLType::Option(_)) => {
                Ok(EvalSignal::Return(Value::Option(None)))
            }
            Some(StageLType::Result(_)) => Ok(EvalSignal::Return(
                Value::Result(Err(StageRErrorTag::UnexpectedNone)),
            )),
            other => bail!(
                "Cannot propagate 'option/T:none' inside routine with return type {other:?} at {span}"
            ),
        },
        other => bail!(
            "Invalid operand {other:?} for propagation '/' ('?') at {span}"
        ),
    }
}

fn check_int_limit(
    vm: &StageLWorkspaceVm,
    n: &Integer,
    context: &str,
) -> Option<EvalSignal> {
    if n.significant_bits() > vm.config.max_bigint_bits {
        Some(EvalSignal::Fault(format!(
            "{context} integer ({} bits) exceeds max_bigint_bits ({})",
            n.significant_bits(),
            vm.config.max_bigint_bits
        )))
    } else {
        None
    }
}

fn check_rat_limit(
    vm: &StageLWorkspaceVm,
    r: &Rational,
    context: &str,
) -> Option<EvalSignal> {
    let max_bits = r
        .numerator_ref()
        .significant_bits()
        .max(r.denominator_ref().significant_bits());
    if max_bits > vm.config.max_bigint_bits {
        Some(EvalSignal::Fault(format!(
            "{context} rational component ({max_bits} bits) exceeds max_bigint_bits ({})",
            vm.config.max_bigint_bits
        )))
    } else {
        None
    }
}

fn check_seq_limit(
    vm: &StageLWorkspaceVm,
    len: usize,
    context: &str,
) -> Option<EvalSignal> {
    if len > vm.config.max_seq_elems {
        Some(EvalSignal::Fault(format!(
            "{context} length {len} exceeds max_seq_elems ({})",
            vm.config.max_seq_elems
        )))
    } else {
        None
    }
}

fn ok_res(val: Value) -> EvalSignal {
    EvalSignal::Value(Value::Result(Ok(Box::new(val))))
}

fn err_res(tag: StageRErrorTag) -> EvalSignal {
    EvalSignal::Value(Value::Result(Err(tag)))
}

fn eval_stager_unary(
    vm: &StageLWorkspaceVm,
    op: StageRUnaryOp,
    val: Value,
    span: Span,
) -> Result<EvalSignal> {
    match op {
        StageRUnaryOp::TotalNeg => match val {
            Value::I32(x) => Ok(match x.checked_neg() {
                Some(v) => ok_res(Value::I32(v)),
                None => err_res(StageRErrorTag::Overflow),
            }),
            Value::I64(x) => Ok(match x.checked_neg() {
                Some(v) => ok_res(Value::I64(v)),
                None => err_res(StageRErrorTag::Overflow),
            }),
            Value::I128(x) => Ok(match x.checked_neg() {
                Some(v) => ok_res(Value::I128(v)),
                None => err_res(StageRErrorTag::Overflow),
            }),
            Value::Int(x) => {
                let neg = x.neg();
                if let Some(sig) = check_int_limit(vm, &neg, "neg") {
                    return Ok(sig);
                }
                Ok(EvalSignal::Value(Value::Int(neg)))
            }
            Value::Rat(r) => {
                let neg = r.neg();
                if let Some(sig) = check_rat_limit(vm, &neg, "neg") {
                    return Ok(sig);
                }
                Ok(EvalSignal::Value(Value::Rat(neg)))
            }
            other => bail!("Invalid operand {other:?} for StageR 'neg' at {span}"),
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

fn eval_stager_binary(
    vm: &StageLWorkspaceVm,
    op: StageRBinaryOp,
    v1: Value,
    v2: Value,
    span: Span,
) -> Result<EvalSignal> {
    match op {
        StageRBinaryOp::TotalAdd => eval_total_arith(
            vm,
            "add",
            v1,
            v2,
            span,
            u8::checked_add,
            i32::checked_add,
            u32::checked_add,
            i64::checked_add,
            u64::checked_add,
            i128::checked_add,
            u128::checked_add,
            Integer::add,
            Rational::add,
        ),
        StageRBinaryOp::TotalSub => eval_total_arith(
            vm,
            "sub",
            v1,
            v2,
            span,
            u8::checked_sub,
            i32::checked_sub,
            u32::checked_sub,
            i64::checked_sub,
            u64::checked_sub,
            i128::checked_sub,
            u128::checked_sub,
            Integer::sub,
            Rational::sub,
        ),
        StageRBinaryOp::TotalMul => eval_total_arith(
            vm,
            "mul",
            v1,
            v2,
            span,
            u8::checked_mul,
            i32::checked_mul,
            u32::checked_mul,
            i64::checked_mul,
            u64::checked_mul,
            i128::checked_mul,
            u128::checked_mul,
            Integer::mul,
            Rational::mul,
        ),
        StageRBinaryOp::TotalDiv => eval_total_div(vm, v1, v2, span),
        StageRBinaryOp::TotalMod => eval_total_mod(vm, v1, v2, span),
        StageRBinaryOp::TotalTruncRem => eval_total_trunc_rem(vm, v1, v2, span),
        StageRBinaryOp::TotalShl => eval_total_shift(true, v1, v2, span),
        StageRBinaryOp::TotalShr => eval_total_shift(false, v1, v2, span),
        StageRBinaryOp::TotalGet => eval_total_get(v1, v2, span),
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
                bail!("'okOr' expected 'error' second operand, got {v2:?} at {span}");
            };
            match v1 {
                Value::Option(Some(inner)) => Ok(EvalSignal::Value(Value::Result(Ok(inner)))),
                Value::Option(None) => Ok(err_res(tag)),
                other => bail!("Invalid first operand {other:?} for 'okOr' at {span}"),
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
            if let Some(sig) = check_rat_limit(vm, &rat, "makeRat") {
                return Ok(sig);
            }
            Ok(ok_res(Value::Rat(rat)))
        }
        StageRBinaryOp::Explode => eval_explode(vm, v1, v2, span),
        StageRBinaryOp::Implode => eval_implode(vm, v1, v2, span),
    }
}

#[allow(
    clippy::too_many_arguments,
    reason = "Parameterizes exact StageR total arithmetic across all numeric representations"
)]
fn eval_total_arith(
    vm: &StageLWorkspaceVm,
    op_name: &str,
    v1: Value,
    v2: Value,
    span: Span,
    f_u8: fn(u8, u8) -> Option<u8>,
    f_i32: fn(i32, i32) -> Option<i32>,
    f_u32: fn(u32, u32) -> Option<u32>,
    f_i64: fn(i64, i64) -> Option<i64>,
    f_u64: fn(u64, u64) -> Option<u64>,
    f_i128: fn(i128, i128) -> Option<i128>,
    f_u128: fn(u128, u128) -> Option<u128>,
    f_int: fn(Integer, Integer) -> Integer,
    f_rat: fn(Rational, Rational) -> Rational,
) -> Result<EvalSignal> {
    let sig = match (v1, v2) {
        (Value::U8(a), Value::U8(b)) => match f_u8(a, b) {
            Some(v) => ok_res(Value::U8(v)),
            None => err_res(StageRErrorTag::Overflow),
        },
        (Value::I32(a), Value::I32(b)) => match f_i32(a, b) {
            Some(v) => ok_res(Value::I32(v)),
            None => err_res(StageRErrorTag::Overflow),
        },
        (Value::U32(a), Value::U32(b)) => match f_u32(a, b) {
            Some(v) => ok_res(Value::U32(v)),
            None => err_res(StageRErrorTag::Overflow),
        },
        (Value::I64(a), Value::I64(b)) => match f_i64(a, b) {
            Some(v) => ok_res(Value::I64(v)),
            None => err_res(StageRErrorTag::Overflow),
        },
        (Value::U64(a), Value::U64(b)) => match f_u64(a, b) {
            Some(v) => ok_res(Value::U64(v)),
            None => err_res(StageRErrorTag::Overflow),
        },
        (Value::I128(a), Value::I128(b)) => match f_i128(a, b) {
            Some(v) => ok_res(Value::I128(v)),
            None => err_res(StageRErrorTag::Overflow),
        },
        (Value::U128(a), Value::U128(b)) => match f_u128(a, b) {
            Some(v) => ok_res(Value::U128(v)),
            None => err_res(StageRErrorTag::Overflow),
        },
        (Value::Int(a), Value::Int(b)) => {
            let res = f_int(a, b);
            if let Some(fault) = check_int_limit(vm, &res, op_name) {
                return Ok(fault);
            }
            EvalSignal::Value(Value::Int(res))
        }
        (Value::Rat(a), Value::Rat(b)) => {
            let res = f_rat(a, b);
            if let Some(fault) = check_rat_limit(vm, &res, op_name) {
                return Ok(fault);
            }
            EvalSignal::Value(Value::Rat(res))
        }
        (l, r) => bail!(
            "Mismatched operands {l:?} vs {r:?} for StageR '{op_name}' at {span}"
        ),
    };
    Ok(sig)
}

fn div_unsigned_or_signed<T: Copy + Eq>(
    a: T,
    b: T,
    zero: T,
    checked_div: fn(T, T) -> Option<T>,
    wrap: fn(T) -> Value,
) -> EvalSignal {
    if b == zero {
        return err_res(StageRErrorTag::DivisionByZero);
    }
    match checked_div(a, b) {
        Some(v) => ok_res(wrap(v)),
        None => err_res(StageRErrorTag::Overflow),
    }
}

fn eval_total_div(
    vm: &StageLWorkspaceVm,
    v1: Value,
    v2: Value,
    span: Span,
) -> Result<EvalSignal> {
    let sig = match (v1, v2) {
        (Value::U8(a), Value::U8(b)) => {
            div_unsigned_or_signed(a, b, 0, u8::checked_div, Value::U8)
        }
        (Value::I32(a), Value::I32(b)) => {
            div_unsigned_or_signed(a, b, 0, i32::checked_div, Value::I32)
        }
        (Value::U32(a), Value::U32(b)) => {
            div_unsigned_or_signed(a, b, 0, u32::checked_div, Value::U32)
        }
        (Value::I64(a), Value::I64(b)) => {
            div_unsigned_or_signed(a, b, 0, i64::checked_div, Value::I64)
        }
        (Value::U64(a), Value::U64(b)) => {
            div_unsigned_or_signed(a, b, 0, u64::checked_div, Value::U64)
        }
        (Value::I128(a), Value::I128(b)) => {
            div_unsigned_or_signed(a, b, 0, i128::checked_div, Value::I128)
        }
        (Value::U128(a), Value::U128(b)) => {
            div_unsigned_or_signed(a, b, 0, u128::checked_div, Value::U128)
        }
        (Value::Int(a), Value::Int(b)) => {
            if b == Integer::ZERO {
                return Ok(err_res(StageRErrorTag::DivisionByZero));
            }
            let q = a.div(b);
            if let Some(fault) = check_int_limit(vm, &q, "div") {
                return Ok(fault);
            }
            ok_res(Value::Int(q))
        }
        (Value::Rat(a), Value::Rat(b)) => {
            if b == Rational::ZERO {
                return Ok(err_res(StageRErrorTag::DivisionByZero));
            }
            let q = a.div(b);
            if let Some(fault) = check_rat_limit(vm, &q, "div") {
                return Ok(fault);
            }
            ok_res(Value::Rat(q))
        }
        (l, r) => bail!(
            "Mismatched operands {l:?} vs {r:?} for StageR 'div' at {span}"
        ),
    };
    Ok(sig)
}

fn mod_signed<T: Copy + Eq>(
    a: T,
    b: T,
    zero: T,
    neg_one: T,
    rem_euclid: fn(T, T) -> Option<T>,
    wrap: fn(T) -> Value,
) -> Result<EvalSignal> {
    if b == zero {
        return Ok(err_res(StageRErrorTag::DivisionByZero));
    }
    if b == neg_one {
        return Ok(ok_res(wrap(zero)));
    }
    let out = rem_euclid(a, b)
        .context("Euclidean modulo failed unexpectedly for divisor != 0, -1")?;
    Ok(ok_res(wrap(out)))
}

fn mod_or_rem_unsigned<T: Copy + Eq>(
    a: T,
    b: T,
    zero: T,
    checked_rem: fn(T, T) -> Option<T>,
    wrap: fn(T) -> Value,
) -> Result<EvalSignal> {
    if b == zero {
        return Ok(err_res(StageRErrorTag::DivisionByZero));
    }
    let out = checked_rem(a, b)
        .context("Unsigned remainder failed unexpectedly for nonzero divisor")?;
    Ok(ok_res(wrap(out)))
}

fn eval_total_mod(
    vm: &StageLWorkspaceVm,
    v1: Value,
    v2: Value,
    span: Span,
) -> Result<EvalSignal> {
    match (v1, v2) {
        (Value::U8(a), Value::U8(b)) => {
            mod_or_rem_unsigned(a, b, 0, u8::checked_rem_euclid, Value::U8)
        }
        (Value::I32(a), Value::I32(b)) => {
            mod_signed(a, b, 0, -1, i32::checked_rem_euclid, Value::I32)
        }
        (Value::U32(a), Value::U32(b)) => {
            mod_or_rem_unsigned(a, b, 0, u32::checked_rem_euclid, Value::U32)
        }
        (Value::I64(a), Value::I64(b)) => {
            mod_signed(a, b, 0, -1, i64::checked_rem_euclid, Value::I64)
        }
        (Value::U64(a), Value::U64(b)) => {
            mod_or_rem_unsigned(a, b, 0, u64::checked_rem_euclid, Value::U64)
        }
        (Value::I128(a), Value::I128(b)) => {
            mod_signed(a, b, 0, -1, i128::checked_rem_euclid, Value::I128)
        }
        (Value::U128(a), Value::U128(b)) => {
            mod_or_rem_unsigned(a, b, 0, u128::checked_rem_euclid, Value::U128)
        }
        (Value::Int(a), Value::Int(b)) => {
            if b == Integer::ZERO {
                return Ok(err_res(StageRErrorTag::DivisionByZero));
            }
            let mut r = (&a).rem(&b);
            if r < Integer::ZERO {
                r = r.add(Integer::from(b.unsigned_abs_ref()));
            }
            if let Some(fault) = check_int_limit(vm, &r, "mod") {
                return Ok(fault);
            }
            Ok(ok_res(Value::Int(r)))
        }
        (l, r) => bail!(
            "Mismatched operands {l:?} vs {r:?} for StageR 'mod' at {span}"
        ),
    }
}

fn eval_total_trunc_rem(
    vm: &StageLWorkspaceVm,
    v1: Value,
    v2: Value,
    span: Span,
) -> Result<EvalSignal> {
    match (v1, v2) {
        (Value::U8(a), Value::U8(b)) => {
            mod_or_rem_unsigned(a, b, 0, u8::checked_rem, Value::U8)
        }
        (Value::I32(a), Value::I32(b)) => {
            mod_signed(a, b, 0, -1, i32::checked_rem, Value::I32)
        }
        (Value::U32(a), Value::U32(b)) => {
            mod_or_rem_unsigned(a, b, 0, u32::checked_rem, Value::U32)
        }
        (Value::I64(a), Value::I64(b)) => {
            mod_signed(a, b, 0, -1, i64::checked_rem, Value::I64)
        }
        (Value::U64(a), Value::U64(b)) => {
            mod_or_rem_unsigned(a, b, 0, u64::checked_rem, Value::U64)
        }
        (Value::I128(a), Value::I128(b)) => {
            mod_signed(a, b, 0, -1, i128::checked_rem, Value::I128)
        }
        (Value::U128(a), Value::U128(b)) => {
            mod_or_rem_unsigned(a, b, 0, u128::checked_rem, Value::U128)
        }
        (Value::Int(a), Value::Int(b)) => {
            if b == Integer::ZERO {
                return Ok(err_res(StageRErrorTag::DivisionByZero));
            }
            let r = a.rem(b);
            if let Some(fault) = check_int_limit(vm, &r, "truncRem") {
                return Ok(fault);
            }
            Ok(ok_res(Value::Int(r)))
        }
        (l, r) => bail!(
            "Mismatched operands {l:?} vs {r:?} for StageR 'truncRem' at {span}"
        ),
    }
}

fn eval_total_shift(
    is_left: bool,
    val: Value,
    count: Value,
    span: Span,
) -> Result<EvalSignal> {
    let Value::U32(cnt) = count else {
        bail!("Shift count must be u32, got {count:?} at {span}");
    };
    let shifted = match val {
        Value::U8(x) => {
            (if is_left { x.checked_shl(cnt) } else { x.checked_shr(cnt) })
                .map(Value::U8)
        }
        Value::I32(x) => {
            (if is_left { x.checked_shl(cnt) } else { x.checked_shr(cnt) })
                .map(Value::I32)
        }
        Value::U32(x) => {
            (if is_left { x.checked_shl(cnt) } else { x.checked_shr(cnt) })
                .map(Value::U32)
        }
        Value::I64(x) => {
            (if is_left { x.checked_shl(cnt) } else { x.checked_shr(cnt) })
                .map(Value::I64)
        }
        Value::U64(x) => {
            (if is_left { x.checked_shl(cnt) } else { x.checked_shr(cnt) })
                .map(Value::U64)
        }
        Value::I128(x) => {
            (if is_left { x.checked_shl(cnt) } else { x.checked_shr(cnt) })
                .map(Value::I128)
        }
        Value::U128(x) => {
            (if is_left { x.checked_shl(cnt) } else { x.checked_shr(cnt) })
                .map(Value::U128)
        }
        other => bail!("Invalid operand {other:?} for StageR shift at {span}"),
    };
    match shifted {
        Some(v) => Ok(ok_res(v)),
        None => Ok(err_res(StageRErrorTag::Overflow)),
    }
}

fn eval_total_get(v1: Value, v2: Value, span: Span) -> Result<EvalSignal> {
    let idx = v2.as_u64_index("get")?;
    let opt = match v1 {
        Value::List(xs) => xs.get(idx).cloned(),
        Value::Buf(b) => b.borrow().elems.get(idx).cloned(),
        Value::Dcs(chars) => chars.get(idx).copied().map(Value::Dc),
        other => bail!(
            "Invalid sequence operand {other:?} for StageR 'get' at {span}"
        ),
    };
    Ok(EvalSignal::Value(Value::Option(opt.map(Box::new))))
}

fn eval_stager_ternary(
    op: StageRTernaryOp,
    v1: Value,
    v2: Value,
    v3: Value,
    span: Span,
) -> Result<EvalSignal> {
    match op {
        StageRTernaryOp::TotalSlice => {
            let start = v2.as_u64_index("slice start")?;
            let end = v3.as_u64_index("slice end")?;
            match v1 {
                Value::List(xs) => {
                    if start > end || end > xs.len() {
                        return Ok(EvalSignal::Value(Value::Option(None)));
                    }
                    let sub = xs
                        .get(start..end)
                        .context("Validated slice bounds failed")?
                        .to_vec();
                    Ok(EvalSignal::Value(Value::Option(Some(Box::new(
                        Value::List(Rc::new(sub)),
                    )))))
                }
                Value::Dcs(chars) => {
                    if start > end || end > chars.len() {
                        return Ok(EvalSignal::Value(Value::Option(None)));
                    }
                    let sub = chars
                        .get(start..end)
                        .context("Validated dcs slice bounds failed")?
                        .to_vec();
                    Ok(EvalSignal::Value(Value::Option(Some(Box::new(
                        Value::Dcs(Rc::new(sub)),
                    )))))
                }
                other => bail!(
                    "Invalid sequence operand {other:?} for StageR 'slice' at {span}"
                ),
            }
        }
        StageRTernaryOp::TotalBufSet => {
            let Value::Buf(b) = v1 else {
                bail!("Invalid buffer operand {v1:?} for StageR 'bufSet' at {span}");
            };
            let idx = v2.as_u64_index("bufSet")?;
            let mut st = b.borrow_mut();
            if st.frozen {
                return Ok(err_res(StageRErrorTag::FrozenBuffer));
            }
            let Some(slot) = st.elems.get_mut(idx) else {
                return Ok(err_res(StageRErrorTag::OutOfBounds));
            };
            *slot = v3;
            Ok(ok_res(Value::Void))
        }
    }
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
            if let Some(fault) = check_seq_limit(vm, parts.len(), "explode") {
                return Ok(fault);
            }
            let out: Vec<Value> = parts
                .into_iter()
                .map(|part| {
                    let flat: Vec<DcChar> = part.into_iter().flatten().collect();
                    Value::S(Rc::new(flat))
                })
                .collect();
            Ok(EvalSignal::Value(Value::List(Rc::new(out))))
        }
        (Value::Dcs(sep), Value::Dcs(inp)) => {
            let parts = explode_slice(&sep, &inp);
            if let Some(fault) = check_seq_limit(vm, parts.len(), "explode") {
                return Ok(fault);
            }
            let out: Vec<Value> = parts
                .into_iter()
                .map(|part| Value::Dcs(Rc::new(part)))
                .collect();
            Ok(EvalSignal::Value(Value::List(Rc::new(out))))
        }
        (Value::List(sep), Value::List(inp)) => {
            let parts = explode_slice(&sep, &inp);
            if let Some(fault) = check_seq_limit(vm, parts.len(), "explode") {
                return Ok(fault);
            }
            let out: Vec<Value> = parts
                .into_iter()
                .map(|part| Value::List(Rc::new(part)))
                .collect();
            Ok(EvalSignal::Value(Value::List(Rc::new(out))))
        }
        (l, r) => bail!(
            "Invalid operands {l:?} and {r:?} for 'explode' at {span}"
        ),
    }
}

fn eval_implode(
    vm: &StageLWorkspaceVm,
    v1: Value,
    v2: Value,
    span: Span,
) -> Result<EvalSignal> {
    match (v1, v2) {
        (Value::S(sep), Value::List(parts)) => {
            let mut vecs = Vec::with_capacity(parts.len());
            for p in parts.iter() {
                let Value::S(chars) = p else {
                    bail!("'implode' expected l/s parts, found {p:?} at {span}");
                };
                vecs.push((**chars).clone());
            }
            let joined = implode_slices(&sep, &vecs);
            if let Some(fault) = check_seq_limit(vm, joined.len(), "implode") {
                return Ok(fault);
            }
            Ok(EvalSignal::Value(Value::S(Rc::new(joined))))
        }
        (Value::Dcs(sep), Value::List(parts)) => {
            let mut vecs = Vec::with_capacity(parts.len());
            for p in parts.iter() {
                let Value::Dcs(chars) = p else {
                    bail!("'implode' expected l/dcs parts, found {p:?} at {span}");
                };
                vecs.push((**chars).clone());
            }
            let joined = implode_slices(&sep, &vecs);
            if let Some(fault) = check_seq_limit(vm, joined.len(), "implode") {
                return Ok(fault);
            }
            Ok(EvalSignal::Value(Value::Dcs(Rc::new(joined))))
        }
        (Value::List(sep), Value::List(parts)) => {
            let mut vecs = Vec::with_capacity(parts.len());
            for p in parts.iter() {
                let Value::List(xs) = p else {
                    bail!("'implode' expected l/l/T parts, found {p:?} at {span}");
                };
                vecs.push((**xs).clone());
            }
            let joined = implode_slices(&sep, &vecs);
            if let Some(fault) = check_seq_limit(vm, joined.len(), "implode") {
                return Ok(fault);
            }
            Ok(EvalSignal::Value(Value::List(Rc::new(joined))))
        }
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
        other => bail!("Expected numeric operand for 'conv', got {other:?} at {span}"),
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
        other => bail!("Expected numeric operand for 'conv', got {other:?} at {span}"),
    }
}

fn value_to_exact_integer(val: Value, span: Span) -> Result<Option<Integer>> {
    match val {
        Value::U8(x) => Ok(Some(Integer::from(x))),
        Value::I32(x) => Ok(Some(Integer::from(x))),
        Value::U32(x) => Ok(Some(Integer::from(x))),
        Value::I64(x) => Ok(Some(Integer::from(x))),
        Value::U64(x) => Ok(Some(Integer::from(x))),
        Value::I128(x) => Ok(Some(Integer::from(x))),
        Value::U128(x) => Ok(Some(Integer::from(x))),
        Value::Int(x) => Ok(Some(x)),
        Value::Rat(r) => Ok(Integer::try_from(r).ok()),
        other => bail!("Expected numeric operand for 'conv', got {other:?} at {span}"),
    }
}

fn eval_total_conv(
    vm: &StageLWorkspaceVm,
    val: Value,
    target_ty: &StageLType,
    span: Span,
) -> Result<EvalSignal> {
    let src_ty = numeric_value_type(&val, span)?;
    let widening = is_stager_widening_conv(&src_ty, target_ty);

    if *target_ty == StageLType::Rat {
        let rat = value_to_rational(val, span)?;
        if let Some(fault) = check_rat_limit(vm, &rat, "conv") {
            return Ok(fault);
        }
        return if widening {
            Ok(EvalSignal::Value(Value::Rat(rat)))
        } else {
            Ok(ok_res(Value::Rat(rat)))
        };
    }

    let converted = match value_to_exact_integer(val, span)? {
        None => None,
        Some(n) => match target_ty {
            StageLType::U8 => u8::try_from(&n).ok().map(Value::U8),
            StageLType::I32 => i32::try_from(&n).ok().map(Value::I32),
            StageLType::U32 => u32::try_from(&n).ok().map(Value::U32),
            StageLType::I64 => i64::try_from(&n).ok().map(Value::I64),
            StageLType::U64 => u64::try_from(&n).ok().map(Value::U64),
            StageLType::I128 => i128::try_from(&n).ok().map(Value::I128),
            StageLType::U128 => u128::try_from(&n).ok().map(Value::U128),
            StageLType::Int => {
                if let Some(fault) = check_int_limit(vm, &n, "conv") {
                    return Ok(fault);
                }
                Some(Value::Int(n))
            }
            other => bail!("Invalid target type '{other}' for StageR 'conv' at {span}"),
        },
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
    use super::super::ast::StageDialect;
    use super::super::validator::ModuleInput;

    #[crate::ctb_test]
    fn test_stager_eval_i128_u128_int_rat_and_shifts() -> Result<()> {
        let src = r#"ns eval/nums.

pub v/b/checkWideLimits
    test checks i128 u128 overflow and mod truncRem MIN -1
        in.
        out true
    const i128/min i128:-170141183460469231731687303715884105728.
    const b/ok1 eq neg /min result/i128:err error:overflow.
    const b/ok2 eq div /min i128:-1 result/i128:err error:overflow.
    const b/ok3 eq mod /min i128:-1 result/i128:ok i128:0.
    const b/ok4 eq truncRem /min i128:-1 result/i128:ok i128:0.
    const b/ok5 eq sub u128:0 u128:1 result/u128:err error:overflow.
    const b/ok6 eq shl u32:1 u32:31 result/u32:ok u32:2147483648.
    const b/ok7 eq shl u32:1 u32:32 result/u32:err error:overflow.
    const b/ok8 eq shr u128:8 u32:127 result/u128:ok u128:0.
    const b/ok9 eq shr u128:8 u32:128 result/u128:err error:overflow.
    and /ok1 and /ok2 and /ok3 and /ok4 and /ok5 and /ok6 and /ok7 and /ok8 /ok9.

pub v/result/b/checkBigIntAndRat
    test checks arbitrary int and canonical rational arithmetic
        in.
        out result/b:ok true
    const int/big mul int:100000000000000000000000000000 int:200000000000000000000000000000.
    const b/okInt and
        eq /big int:20000000000000000000000000000000000000000000000000000000000
        eq div int:5 int:0 result/int:err error:divisionByZero.
    const rat/r1 makeRat? int:6 int:-4.
    const rat/r2 makeRat? int:-3 int:2.
    const rat/r3 add makeRat? int:1 int:3 makeRat? int:1 int:6.
    const rat/half makeRat? int:1 int:2.
    const b/badDen eq makeRat int:1 int:0 result/rat:err error:divisionByZero.
    result/b:ok and /okInt and eq /r1 /r2 and eq /r3 /half /badDen.
"#;
        let report = run_sr3_module_inline_tests(src)?;
        assert_eq!(report.ensure_all_passed()?, 2);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_stager_eval_euclidean_mod_vs_trunc_rem_and_exact_conv()
    -> Result<()> {
        let src = r#"ns eval/modconv.

pub v/result/b/checkModAndTruncRem
    test checks euclidean mod vs truncRem across signed operands
        in.
        out result/b:ok true
    const i32/m1 mod? i32:-7 i32:3.
    const i32/m2 mod? i32:-7 i32:-3.
    const i32/t1 truncRem? i32:-7 i32:3.
    const i32/t2 truncRem? i32:-7 i32:-3.
    const i32/m0 mod? i32:5 i32:-1.
    const i32/t0 truncRem? i32:5 i32:-1.
    const b/okMod and eq /m1 i32:2 and eq /m2 i32:2 eq /m0 i32:0.
    const b/okRem and eq /t1 i32:-1 and eq /t2 i32:-1 eq /t0 i32:0.
    result/b:ok and /okMod /okRem.

pub v/result/b/checkExactConv
    test checks exact value-preserving conv rules
        in.
        out result/b:ok true
    const rat/exactIntRat makeRat? int:6 int:2.
    const rat/nonIntRat makeRat? int:5 int:2.
    const int/fromRat conv? /exactIntRat int.
    const b/okExact eq /fromRat int:3.
    const b/failNonInt eq conv /nonIntRat int result/int:err error:overflow.
    const b/okNarrow eq conv int:255 u8 result/u8:ok u8:255.
    const b/failRange eq conv int:256 u8 result/u8:err error:overflow.
    const rat/wide conv int:-42 rat.
    const b/okWide eq /wide makeRat? int:-42 int:1.
    result/b:ok and /okExact and /failNonInt and /okNarrow and /failRange /okWide.
"#;
        let report = run_sr3_module_inline_tests(src)?;
        assert_eq!(report.ensure_all_passed()?, 2);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_stager_eval_dc_dcs_s_explode_implode_get_slice_and_bufset()
    -> Result<()> {
        let src = r#"ns eval/strings.

pub v/result/b/checkDcAndStrings
    test checks dcFromU128 dcsFromS sFromDcs byteLen and explode implode
        in.
        out result/b:ok true
    const dc/cu100 dcFromU128? u128:256.
    const dc/c256 dcFromU128? u128:1114368.
    const b/okDc and eq /cu100 dc:u100 and eq /c256 dc:256 eq u128FromDc /c256 u128:1114368.
    const b/badSurrogate eq dcFromU128 u128:55296 result/dc:err error:invalidDc.

    const dcs/pure dcsFromS? "hi @256@".
    const b/okLen and eq len /pure u64:4 eq get /pure u64:3 option/dc:some dc:256.
    const b/okOob and eq get /pure u64:9 option/dc:none eq slice /pure u64:3 u64:1 option/dcs:none.
    const dcs/sl slice? /pure u64:1 u64:3.
    const s/back sFromDcs /sl.
    const b/okRound eq /back "i ".

    : Embedded valid Base64 binary run Dc 203..204 (21-byte header + 3 payload bytes = 24 bytes)
    const s/withBin "pre:@203@@127@@127@@127@@127@@204@:post".
    const b/binRejected eq dcsFromS /withBin result/dcs:err error:invalidEncoding.
    const u64/blen byteLen /withBin.
    const l/s/parts explode ":" /withBin.
    const s/rejoined implode ":" /parts.
    const l/s/chars explode "" "a@256@@203@@127@@127@@127@@127@@204@".
    const b/okBinSplit and eq /rejoined /withBin and eq len /parts u64:3 and eq len /chars u64:3 eq get /parts u64:1 option/s:some "@203@@127@@127@@127@@127@@204@".

    const l/i32/ints l/i32: i32:10 i32:20 i32:30 ::.
    const l/l/i32/iParts explode l/i32: i32:20 :: /ints.
    const b/okIntList and eq implode l/i32: i32:20 :: /iParts /ints eq len /iParts u64:2.

    let buf/i32/mbuf bufFill i32:0 u64:2.
    const b/okSet0 eq bufSet /mbuf u64:0 i32:11 result/void:ok nop.
    const b/okMatchSet
        match bufSet /mbuf u64:1 i32:99
            case ok true.
            case err error/_ev false.
        ::.
    const b/badSet eq bufSet /mbuf u64:5 i32:1 result/void:err error:outOfBounds.
    const l/i32/frozen bufFreeze /mbuf.
    const b/okFrozen and eq get /frozen u64:0 option/i32:some i32:11 eq get /frozen u64:1 option/i32:some i32:99.

    result/b:ok and /okDc and /badSurrogate and /okLen and /okOob and /okRound and /binRejected and gt /blen u64:24 and /okBinSplit and /okIntList and /okSet0 and /okMatchSet and /badSet /okFrozen.
"#;
        let report = run_sr3_module_inline_tests(src)?;
        assert_eq!(report.ensure_all_passed()?, 1);

        // Also test runtime `err error:frozenBuffer` directly on a frozen Value::Buf handle.
        let frozen_buf = Value::Buf(std::rc::Rc::new(std::cell::RefCell::new(
            crate::stagel3::eval::BufStorage {
                elems: vec![Value::I32(1)],
                frozen: true,
            },
        )));
        let sig = eval_stager_ternary(
            StageRTernaryOp::TotalBufSet,
            frozen_buf,
            Value::U64(0),
            Value::I32(7),
            Span { line: 1, col: 1 },
        )?;
        assert!(matches!(
            sig,
            EvalSignal::Value(Value::Result(Err(StageRErrorTag::FrozenBuffer)))
        ));
        Ok(())
    }

    #[crate::ctb_test]
    fn test_stager_eval_propagate_match_tail_trampoline_and_resource_traps()
    -> Result<()> {
        let src = r#"ns eval/control.

enum step
    int/done.
    int/next.

v/option/i32/optPropagate option/i32/x
    const i32/unw /x?.
    option/i32:some wrapAdd /unw i32:1.

v/result/i32/optToRes option/i32/x
    const i32/unw /x/.
    result/i32:ok /unw.

v/int/countRec step/st int/acc
    match /st
        case done int/fin /fin.
        case next int/n if le /n int:0
            countRec step:done /acc int:0
            countRec step:next sub /n int:1 add /acc /n.
    ::.

pub v/int/tailMatchSum int/n
    test deep match tail call with tight max_call_depth
        in int:500
        out int:125250
    countRec step:next /n int:0.

pub v/b/checkPropagation
    test verifies option and result propagation conversions
        in.
        out true
    const b/o1 eq optPropagate option/i32:none option/i32:none.
    const b/o2 eq optToRes option/i32:none result/i32:err error:unexpectedNone.
    const b/o3 eq optToRes option/i32:some i32:7 result/i32:ok i32:7.
    and /o1 and /o2 /o3.
"#;
        let report = run_sr3_module_inline_tests_with_config(
            src,
            EvalConfig {
                compute_quota: 2_000_000,
                max_call_depth: 16,
                ..EvalConfig::default()
            },
        )?;
        assert_eq!(report.ensure_all_passed()?, 2);

        // Verify `max_bigint_bits` and `max_seq_elems` cleanly trap (`EvalSignal::Fault`).
        let big_trap = r#"ns eval/bigtrap.
pub v/int/boom
    test bigint trap
        in.
        out int:0
    mul int:1000000000000 int:1000000000000.
"#;
        let big_rep = run_sr3_module_inline_tests_with_config(
            big_trap,
            EvalConfig {
                max_bigint_bits: 32,
                ..EvalConfig::default()
            },
        )?;
        let big_err = big_rep.ensure_all_passed().unwrap_err().to_string();
        assert!(big_err.contains("max_bigint_bits"), "got: {big_err}");

        let seq_trap = r#"ns eval/seqtrap.
pub v/u64/boom
    test seq trap
        in.
        out u64:0
    let buf/u8/mbuf bufFill u8:1 u64:16.
    len bufFreeze /mbuf.
"#;
        let seq_rep = run_sr3_workspace_inline_tests_with_config(
            &[ModuleInput {
                source_path: Some("eval/seqtrap.sr3"),
                source: seq_trap,
            }],
            EvalConfig {
                max_seq_elems: 4,
                ..EvalConfig::default()
            },
        )?;
        let seq_err = seq_rep.ensure_all_passed().unwrap_err().to_string();
        assert!(
            seq_err.contains("max_seq_elems") || seq_err.contains("capacity"),
            "got: {seq_err}"
        );
        let _ = StageDialect::StageR3;
        Ok(())
    }
}
