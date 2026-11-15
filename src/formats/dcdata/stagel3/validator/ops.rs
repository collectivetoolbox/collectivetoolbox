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

//! Dialect-parameterized validation of literals, composite constructors and
//! projections, nominal encapsulation (`wrap`/`unwrap`), and Core / StageR
//! unary, binary, ternary, and numeric conversion operators.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::super::ast::{
    CoreBinaryOp, CoreTernaryOp, CoreUnaryOp, Expr, Span, StageDialect,
    StageLType, StageRBinaryOp, StageRTernaryOp, StageRUnaryOp, StructTypeRef,
    SymbolTarget, TypedIntLit,
};
use super::consts_and_flow::{BodyCtx, FlowState};
use super::imports_and_decls::ModuleValidator;
use super::types::{
    BufPolicy, check_stager_value_type_shape, is_stager_widening_conv,
    type_can_be_element_with_defs, types_compatible, unify_matching_pair,
};
use std::collections::HashSet;

fn is_stager_integer(ty: &StageLType) -> bool {
    ty.is_stager_fixed_int() || *ty == StageLType::Int
}

impl<'a> ModuleValidator<'a> {
    /// Checks `expr` in the current tail/loop/flow context and records its
    /// static type in `self.expr_types`.
    pub(super) fn check_expr(
        &mut self,
        expr: &Expr,
        state: &mut FlowState,
        ctx: &mut BodyCtx,
    ) -> Result<StageLType> {
        let span = expr.span();
        let ty = self.check_expr_inner(expr, state, ctx)?;
        self.expr_types.insert((span.line, span.col), ty.clone());
        Ok(ty)
    }

    /// Checks a non-tail subexpression while preserving the caller's `in_tail`
    /// flag and accumulating captured reads/nested calls on `ctx`.
    pub(super) fn check_subexpr(
        &mut self,
        expr: &Expr,
        state: &mut FlowState,
        ctx: &mut BodyCtx,
    ) -> Result<StageLType> {
        let prev_tail = ctx.in_tail;
        ctx.in_tail = false;
        let res = self.check_expr(expr, state, ctx);
        ctx.in_tail = prev_tail;
        res
    }

    #[allow(
        clippy::too_many_lines,
        reason = "Central AST expression type-checking dispatch across Core and StageR nodes"
    )]
    fn check_expr_inner(
        &mut self,
        expr: &Expr,
        state: &mut FlowState,
        ctx: &mut BodyCtx,
    ) -> Result<StageLType> {
        match expr {
            Expr::Bool { .. } => Ok(StageLType::B),
            Expr::Bytes { .. } => Ok(StageLType::List(Box::new(StageLType::U8))),
            Expr::Int { value, span } => {
                if self.dialect == StageDialect::StageL3
                    && matches!(
                        value,
                        TypedIntLit::I128(_)
                            | TypedIntLit::U128(_)
                            | TypedIntLit::Int(_)
                    )
                {
                    bail!(
                        "StageR 3 integer type '{}' is not supported in StageL 3 validator at {span}",
                        value.stage_l_type().display_name()
                    );
                }
                Ok(value.stage_l_type())
            }
            Expr::List {
                elem_ty,
                elements,
                span,
            } => {
                let canon_elem = self.check_and_canonicalize_type(
                    elem_ty,
                    BufPolicy::Disallowed,
                    false,
                    false,
                    *span,
                )?;
                ensure!(
                    type_can_be_element_with_defs(
                        &canon_elem,
                        &self.type_defs()
                    ),
                    "Invalid list element type '{}' in 'l/{}' at {span}",
                    canon_elem.display_name(),
                    canon_elem.display_name()
                );
                if self.dialect == StageDialect::StageR3 {
                    check_stager_value_type_shape(
                        &canon_elem,
                        &self.type_defs(),
                        *span,
                    )?;
                }
                for el in elements {
                    let el_ty = self.check_subexpr(el, state, ctx)?;
                    if self.dialect == StageDialect::StageR3 {
                        ensure!(
                            !el_ty.is_or_contains_buf(),
                            "Linear 'buf/T' cannot be stored inside list at {}",
                            el.span()
                        );
                    }
                    ensure!(
                        types_compatible(&el_ty, &canon_elem),
                        "List element at {} has type '{el_ty}', expected '{canon_elem}'",
                        el.span()
                    );
                }
                Ok(StageLType::List(Box::new(canon_elem)))
            }
            Expr::TupleNew { elements, span: _ } => {
                let mut slot_tys = Vec::with_capacity(elements.len());
                let mut any_never = false;
                for el in elements {
                    let el_ty = self.check_subexpr(el, state, ctx)?;
                    ensure!(
                        el_ty != StageLType::Void,
                        "Tuple element cannot have type 'void' at {}",
                        el.span()
                    );
                    if self.dialect == StageDialect::StageR3 {
                        ensure!(
                            !el_ty.is_or_contains_buf(),
                            "Linear 'buf/T' cannot be stored inside tuple at {}",
                            el.span()
                        );
                    }
                    if el_ty == StageLType::Never {
                        any_never = true;
                    }
                    slot_tys.push(el_ty);
                }
                if any_never {
                    Ok(StageLType::Never)
                } else {
                    Ok(StageLType::Tuple(slot_tys))
                }
            }
            Expr::StructNew {
                target,
                fields,
                span,
            } => {
                ensure!(
                    self.dialect == StageDialect::StageL3,
                    "Positional 'structNew' at {span} is forbidden in StageR 3 (use named-field struct syntax)"
                );
                let key = match target {
                    SymbolTarget::Ident(name) => self
                        .local_structs
                        .get(name)
                        .cloned()
                        .with_context(|| {
                            format!("Unknown struct '{name}' at {span}")
                        })?,
                    SymbolTarget::Dc(dc) => self
                        .dc_structs
                        .get(dc)
                        .cloned()
                        .with_context(|| {
                            format!("Unresolved struct Dc reference at {span}")
                        })?,
                };
                let sdef = self
                    .struct_defs
                    .get(&key)
                    .cloned()
                    .with_context(|| {
                        format!(
                            "Unknown positional struct '{}' at {span}",
                            key.1
                        )
                    })?;
                ensure!(
                    fields.len() == sdef.fields.len(),
                    "Struct '{}' expects {} field(s), found {} at {span}",
                    sdef.original_name,
                    sdef.fields.len(),
                    fields.len()
                );
                for (idx, (arg, fty)) in
                    fields.iter().zip(&sdef.fields).enumerate()
                {
                    let arg_ty = self.check_subexpr(arg, state, ctx)?;
                    ensure!(
                        types_compatible(&arg_ty, fty),
                        "Struct '{}' slot {} at {} has type '{arg_ty}', expected '{fty}'",
                        sdef.original_name,
                        idx,
                        arg.span()
                    );
                }
                self.resolved_structs
                    .insert((span.line, span.col), sdef);
                Ok(StageLType::Struct(StructTypeRef::Canonical {
                    namespace: key.0,
                    name: key.1,
                }))
            }
            Expr::Field { base, index, span } => {
                let base_ty = self.check_subexpr(base, state, ctx)?;
                if base_ty == StageLType::Never {
                    return Ok(StageLType::Never);
                }
                let idx = usize::try_from(*index)
                    .context("Positional field index exceeds usize")?;
                match &base_ty {
                    StageLType::Tuple(slots) => {
                        let Some(slot_ty) = slots.get(idx) else {
                            bail!(
                                "Tuple field index {index} out of bounds for '{base_ty}' (has {} slot(s)) at {span}",
                                slots.len()
                            );
                        };
                        Ok(slot_ty.clone())
                    }
                    StageLType::Struct(StructTypeRef::Canonical {
                        namespace,
                        name,
                    }) => {
                        ensure!(
                            self.dialect == StageDialect::StageL3,
                            "Positional 'field' on struct '{name}' at {span} is forbidden in StageR 3"
                        );
                        let key = (namespace.clone(), name.clone());
                        let sdef =
                            self.struct_defs.get(&key).with_context(|| {
                                format!("Unknown struct '{name}' at {span}")
                            })?;
                        let Some(fty) = sdef.fields.get(idx) else {
                            bail!(
                                "Struct '{name}' field index {index} out of bounds (has {} field(s)) at {span}",
                                sdef.fields.len()
                            );
                        };
                        Ok(fty.clone())
                    }
                    _ => bail!(
                        "Positional 'field' requires a tuple{} at {span}, found '{base_ty}'",
                        if self.dialect == StageDialect::StageL3 {
                            " or positional struct"
                        } else {
                            ""
                        }
                    ),
                }
            }
            Expr::With {
                base,
                index,
                value,
                span,
            } => {
                let base_ty = self.check_subexpr(base, state, ctx)?;
                let val_ty = self.check_subexpr(value, state, ctx)?;
                if self.dialect == StageDialect::StageR3 {
                    ensure!(
                        !val_ty.is_or_contains_buf(),
                        "Linear 'buf/T' cannot be placed into 'with' update at {span}"
                    );
                }
                if base_ty == StageLType::Never {
                    return Ok(StageLType::Never);
                }
                let idx = usize::try_from(*index)
                    .context("Positional 'with' index exceeds usize")?;
                match &base_ty {
                    StageLType::Tuple(slots) => {
                        let Some(expected) = slots.get(idx) else {
                            bail!(
                                "Tuple 'with' index {index} out of bounds for '{base_ty}' (has {} slot(s)) at {span}",
                                slots.len()
                            );
                        };
                        ensure!(
                            types_compatible(&val_ty, expected),
                            "Tuple 'with' replacement at index {index} has type '{val_ty}', expected '{expected}' at {span}"
                        );
                        if val_ty == StageLType::Never {
                            Ok(StageLType::Never)
                        } else {
                            Ok(base_ty)
                        }
                    }
                    StageLType::Struct(StructTypeRef::Canonical {
                        namespace,
                        name,
                    }) => {
                        ensure!(
                            self.dialect == StageDialect::StageL3,
                            "Positional 'with' on struct '{name}' at {span} is forbidden in StageR 3"
                        );
                        let key = (namespace.clone(), name.clone());
                        let sdef =
                            self.struct_defs.get(&key).with_context(|| {
                                format!("Unknown struct '{name}' at {span}")
                            })?;
                        let Some(expected) = sdef.fields.get(idx) else {
                            bail!(
                                "Struct '{name}' 'with' index {index} out of bounds (has {} field(s)) at {span}",
                                sdef.fields.len()
                            );
                        };
                        ensure!(
                            types_compatible(&val_ty, expected),
                            "Struct '{name}' 'with' replacement at index {index} has type '{val_ty}', expected '{expected}' at {span}"
                        );
                        if val_ty == StageLType::Never {
                            Ok(StageLType::Never)
                        } else {
                            Ok(base_ty)
                        }
                    }
                    _ => bail!(
                        "Positional 'with' requires a tuple{} at {span}, found '{base_ty}'",
                        if self.dialect == StageDialect::StageL3 {
                            " or positional struct"
                        } else {
                            ""
                        }
                    ),
                }
            }
            Expr::BufNew {
                elem_ty,
                capacity,
                span,
            } => {
                let buf_policy = match self.dialect {
                    StageDialect::StageL3 => BufPolicy::LocalAllowed,
                    StageDialect::StageR3 => BufPolicy::Disallowed,
                };
                let canon_elem = self.check_and_canonicalize_type(
                    elem_ty, buf_policy, false, false, *span,
                )?;
                ensure!(
                    type_can_be_element_with_defs(
                        &canon_elem,
                        &self.type_defs()
                    ),
                    "Invalid buffer element type '{}' in 'bufNew' at {span}",
                    canon_elem.display_name()
                );
                if self.dialect == StageDialect::StageR3 {
                    check_stager_value_type_shape(
                        &canon_elem,
                        &self.type_defs(),
                        *span,
                    )?;
                }
                let cap_ty = self.check_subexpr(capacity, state, ctx)?;
                ensure!(
                    cap_ty == StageLType::U64 || cap_ty == StageLType::Never,
                    "'bufNew' capacity must have type 'u64', found '{cap_ty}' at {span}"
                );
                Ok(StageLType::Buf(Box::new(canon_elem)))
            }
            Expr::ValueRef { ty, target, span } => {
                self.check_value_ref(ty, target, *span, state, ctx, false)
            }
            Expr::Let {
                ty,
                name,
                init,
                span,
            } => self.check_let(ty, name, init.as_deref(), *span, state, ctx),
            Expr::Const {
                ty,
                name,
                value,
                span,
                ..
            } => self.check_local_const(ty, name, value, *span, state, ctx),
            Expr::Set {
                ty,
                name,
                value,
                span,
            } => self.check_set(ty, name, value, *span, state, ctx),
            Expr::RoutineDecl(r) => bail!(
                "Nested routine declaration '{}' at {} must appear inside a 'do' block",
                r.name,
                r.span
            ),
            Expr::Call { target, args, span } => {
                self.check_call(target, args, *span, state, ctx)
            }
            Expr::Block { stmts, span } => {
                self.check_block(stmts, *span, state, ctx)
            }
            Expr::If {
                cond,
                then_branch,
                else_branch,
                span,
            } => self.check_if(cond, then_branch, else_branch, *span, state, ctx),
            Expr::While { cond, body, span } => {
                self.check_while(cond, body, *span, state, ctx)
            }
            Expr::ForEach {
                elem_ty,
                elem_name,
                list,
                body,
                span,
            } => self.check_foreach(
                elem_ty, elem_name, list, body, *span, state, ctx,
            ),
            Expr::Match {
                scrutinee,
                arms,
                fallback,
                span,
            } => self.check_match(
                scrutinee,
                arms,
                fallback.as_deref(),
                *span,
                state,
                ctx,
            ),
            Expr::Break { span } => {
                ensure!(
                    ctx.loop_depth > 0,
                    "'break' used outside of an enclosing loop at {span}"
                );
                Ok(StageLType::Never)
            }
            Expr::Continue { span } => {
                ensure!(
                    ctx.loop_depth > 0,
                    "'continue' used outside of an enclosing loop at {span}"
                );
                Ok(StageLType::Never)
            }
            Expr::Return { value, span } => {
                self.check_return(value.as_deref(), *span, state, ctx)
            }
            Expr::Fault { span } => {
                ensure!(
                    self.dialect == StageDialect::StageL3,
                    "'fault' trap at {span} is forbidden in total StageR 3"
                );
                Ok(StageLType::Never)
            }
            Expr::Nop { .. } => Ok(StageLType::Void),
            Expr::Propagate { expr, span } => {
                self.check_propagate(expr, *span, state, ctx)
            }
            Expr::Unary { op, operand, span } => {
                self.check_core_unary(*op, operand, *span, state, ctx)
            }
            Expr::Binary {
                op,
                left,
                right,
                span,
            } => self.check_core_binary(*op, left, right, *span, state, ctx),
            Expr::Ternary {
                op,
                first,
                second,
                third,
                span,
            } => self.check_core_ternary(
                *op, first, second, third, *span, state, ctx,
            ),
            Expr::Conv {
                expr,
                target_ty,
                span,
            } => {
                ensure!(
                    self.dialect == StageDialect::StageL3,
                    "Trapping StageL 3 'conv' node at {span} is forbidden in StageR 3 (use StageR total 'conv')"
                );
                let src_ty = self.check_subexpr(expr, state, ctx)?;
                let dst_ty = self.check_and_canonicalize_type(
                    target_ty,
                    BufPolicy::Disallowed,
                    false,
                    false,
                    *span,
                )?;
                ensure!(
                    dst_ty.is_integer(),
                    "'conv' target type must be a StageL 3 integer type, found '{dst_ty}' at {span}"
                );
                ensure!(
                    src_ty == StageLType::Never || src_ty.is_integer(),
                    "'conv' source operand must be a StageL 3 integer type, found '{src_ty}' at {span}"
                );
                Ok(dst_ty)
            }
            Expr::Bitcast {
                expr,
                target_ty,
                span,
            } => {
                ensure!(
                    self.dialect == StageDialect::StageL3,
                    "'bitcast' at {span} is forbidden in StageR 3"
                );
                let src_ty = self.check_subexpr(expr, state, ctx)?;
                let dst_ty = self.check_and_canonicalize_type(
                    target_ty,
                    BufPolicy::Disallowed,
                    false,
                    false,
                    *span,
                )?;
                ensure!(
                    dst_ty.is_integer(),
                    "'bitcast' target type must be a StageL 3 integer type, found '{dst_ty}' at {span}"
                );
                if src_ty != StageLType::Never {
                    ensure!(
                        src_ty.is_integer()
                            && src_ty.bit_width() == dst_ty.bit_width(),
                        "'bitcast' requires matching bit widths at {span}: cannot bitcast '{src_ty}' to '{dst_ty}'"
                    );
                }
                Ok(dst_ty)
            }
            Expr::DctsString { span, .. } => {
                ensure!(
                    self.dialect == StageDialect::StageR3,
                    "StageR 3 's' string literal is not supported in StageL 3 at {span}"
                );
                Ok(StageLType::S)
            }
            Expr::DcLit { span, .. } => {
                ensure!(
                    self.dialect == StageDialect::StageR3,
                    "StageR 3 'dc' scalar literal is not supported in StageL 3 at {span}"
                );
                Ok(StageLType::Dc)
            }
            Expr::ErrorTag { span, .. } => {
                ensure!(
                    self.dialect == StageDialect::StageR3,
                    "StageR 3 'error' tag literal is not supported in StageL 3 at {span}"
                );
                Ok(StageLType::Error)
            }
            Expr::OptionNone { inner_ty, span } => {
                ensure!(
                    self.dialect == StageDialect::StageR3,
                    "StageR 3 'option' literal is not supported in StageL 3 at {span}"
                );
                let canon = self.check_and_canonicalize_type(
                    inner_ty,
                    BufPolicy::Disallowed,
                    false,
                    false,
                    *span,
                )?;
                check_stager_value_type_shape(&canon, &self.type_defs(), *span)?;
                Ok(StageLType::Option(Box::new(canon)))
            }
            Expr::OptionSome {
                inner_ty,
                value,
                span,
            } => {
                ensure!(
                    self.dialect == StageDialect::StageR3,
                    "StageR 3 'option' constructor is not supported in StageL 3 at {span}"
                );
                let canon = self.check_and_canonicalize_type(
                    inner_ty,
                    BufPolicy::Disallowed,
                    false,
                    false,
                    *span,
                )?;
                check_stager_value_type_shape(&canon, &self.type_defs(), *span)?;
                let val_ty = self.check_subexpr(value, state, ctx)?;
                ensure!(
                    !val_ty.is_or_contains_buf(),
                    "Linear 'buf/T' cannot be wrapped in 'option' at {span}"
                );
                ensure!(
                    types_compatible(&val_ty, &canon),
                    "'option/{canon}:some' payload at {span} has type '{val_ty}', expected '{canon}'"
                );
                Ok(StageLType::Option(Box::new(canon)))
            }
            Expr::ResultOk { ok_ty, value, span } => {
                ensure!(
                    self.dialect == StageDialect::StageR3,
                    "StageR 3 'result' constructor is not supported in StageL 3 at {span}"
                );
                let canon = self.check_and_canonicalize_type(
                    ok_ty,
                    BufPolicy::Disallowed,
                    true,
                    false,
                    *span,
                )?;
                if canon != StageLType::Void {
                    check_stager_value_type_shape(
                        &canon,
                        &self.type_defs(),
                        *span,
                    )?;
                }
                let val_ty = self.check_subexpr(value, state, ctx)?;
                ensure!(
                    !val_ty.is_or_contains_buf(),
                    "Linear 'buf/T' cannot be wrapped in 'result' at {span}"
                );
                ensure!(
                    types_compatible(&val_ty, &canon),
                    "'result/{canon}:ok' value at {span} has type '{val_ty}', expected '{canon}'"
                );
                Ok(StageLType::Result(Box::new(canon)))
            }
            Expr::ResultErr { ok_ty, error, span } => {
                ensure!(
                    self.dialect == StageDialect::StageR3,
                    "StageR 3 'result' constructor is not supported in StageL 3 at {span}"
                );
                let canon = self.check_and_canonicalize_type(
                    ok_ty,
                    BufPolicy::Disallowed,
                    true,
                    false,
                    *span,
                )?;
                if canon != StageLType::Void {
                    check_stager_value_type_shape(
                        &canon,
                        &self.type_defs(),
                        *span,
                    )?;
                }
                let err_ty = self.check_subexpr(error, state, ctx)?;
                ensure!(
                    types_compatible(&err_ty, &StageLType::Error),
                    "'result/{canon}:err' requires 'error' operand at {span}, found '{err_ty}'"
                );
                Ok(StageLType::Result(Box::new(canon)))
            }
            Expr::NamedStructNew {
                target,
                fields,
                span,
            } => self.check_named_struct_new(target, fields, *span, state, ctx),
            Expr::NamedField { base, field, span } => {
                self.check_named_field(base, field, *span, state, ctx)
            }
            Expr::NamedWith {
                base,
                field,
                value,
                span,
            } => self.check_named_with(base, field, value, *span, state, ctx),
            Expr::EnumVariant {
                enum_ty,
                variant,
                payload,
                span,
            } => self.check_enum_variant(
                enum_ty,
                variant,
                payload.as_deref(),
                *span,
                state,
                ctx,
            ),
            Expr::WrapNominal {
                expr,
                target_ty,
                span,
            } => {
                ensure!(
                    self.dialect == StageDialect::StageR3,
                    "StageR 3 'wrap' is not supported in StageL 3 at {span}"
                );
                let val_ty = self.check_subexpr(expr, state, ctx)?;
                ensure!(
                    !val_ty.is_or_contains_buf(),
                    "Linear 'buf/T' cannot be wrapped inside nominal type at {span}"
                );
                let canon_target =
                    self.canonicalize_type(target_ty, *span)?;
                let StageLType::Nominal(StructTypeRef::Canonical {
                    namespace,
                    name,
                }) = &canon_target
                else {
                    bail!(
                        "'wrap' target at {span} must be a 'brand' or 'type' definition, found '{canon_target}'"
                    );
                };
                let key = (namespace.clone(), name.clone());
                let Some(bdef) = self.brand_types.get(&key).cloned() else {
                    if self.enums.contains_key(&key) {
                        bail!(
                            "Cannot use 'wrap' on enum '{name}' at {span}; construct a variant directly"
                        );
                    }
                    bail!("Unknown nominal/brand type '{name}' in 'wrap' at {span}");
                };
                ensure!(
                    !bdef.is_opaque || self.local_declared_opaque.contains(&key),
                    "Cannot 'wrap' module-opaque type '{name}' outside its defining module at {span}"
                );
                ensure!(
                    types_compatible(&val_ty, &bdef.inner_ty),
                    "'wrap' operand at {span} has type '{val_ty}', expected underlying representation '{}' for '{name}'",
                    bdef.inner_ty
                );
                Ok(canon_target)
            }
            Expr::UnwrapNominal { expr, span } => {
                ensure!(
                    self.dialect == StageDialect::StageR3,
                    "StageR 3 'unwrap' is not supported in StageL 3 at {span}"
                );
                let val_ty = self.check_subexpr(expr, state, ctx)?;
                if val_ty == StageLType::Never {
                    return Ok(StageLType::Never);
                }
                let StageLType::Nominal(StructTypeRef::Canonical {
                    namespace,
                    name,
                }) = &val_ty
                else {
                    bail!(
                        "'unwrap' requires a 'brand' or 'type' operand at {span}, found '{val_ty}'"
                    );
                };
                let key = (namespace.clone(), name.clone());
                let Some(bdef) = self.brand_types.get(&key) else {
                    if self.enums.contains_key(&key) {
                        bail!(
                            "Cannot use 'unwrap' on enum '{name}' at {span}; use 'match'"
                        );
                    }
                    bail!("Unknown nominal/brand type '{name}' in 'unwrap' at {span}");
                };
                ensure!(
                    !bdef.is_opaque || self.local_declared_opaque.contains(&key),
                    "Cannot 'unwrap' module-opaque type '{name}' outside its defining module at {span}"
                );
                Ok(bdef.inner_ty.clone())
            }
            Expr::StageRUnary { op, operand, span } => {
                self.check_stager_unary(*op, operand, *span, state, ctx)
            }
            Expr::StageRBinary {
                op,
                left,
                right,
                span,
            } => self.check_stager_binary(*op, left, right, *span, state, ctx),
            Expr::StageRTernary {
                op,
                first,
                second,
                third,
                span,
            } => self.check_stager_ternary(
                *op, first, second, third, *span, state, ctx,
            ),
            Expr::TotalConv {
                expr,
                target_ty,
                span,
            } => self.check_total_conv(expr, target_ty, *span, state, ctx),
        }
    }

    fn check_named_struct_new(
        &mut self,
        target: &SymbolTarget,
        fields: &[(String, Expr, Span)],
        span: Span,
        state: &mut FlowState,
        ctx: &mut BodyCtx,
    ) -> Result<StageLType> {
        ensure!(
            self.dialect == StageDialect::StageR3,
            "StageR 3 named 'structNew' is not supported in StageL 3 at {span}"
        );
        let key = match target {
            SymbolTarget::Ident(name) => self
                .local_structs
                .get(name)
                .cloned()
                .with_context(|| {
                    format!("Unknown struct '{name}' in 'structNew' at {span}")
                })?,
            SymbolTarget::Dc(dc) => self
                .dc_structs
                .get(dc)
                .cloned()
                .with_context(|| {
                    format!("Unresolved struct Dc reference in 'structNew' at {span}")
                })?,
        };
        let Some(nsdef) = self.named_structs.get(&key).cloned() else {
            if self.struct_defs.contains_key(&key) {
                bail!(
                    "Imported Stage 1 positional struct '{}' is opaque in StageR 3 and cannot be constructed with 'structNew' at {span}",
                    key.1
                );
            }
            bail!("Unknown named struct '{}' at {span}", key.1);
        };

        let expected_fields: HashSet<&str> =
            nsdef.fields.iter().map(|(n, _)| n.as_str()).collect();
        let mut seen = HashSet::new();
        for (fname, fexpr, fspan) in fields {
            ensure!(
                seen.insert(fname.as_str()),
                "Duplicate field '{fname}' in 'structNew {}' at {fspan}",
                nsdef.original_name
            );
            let Some((_, expected_ty)) =
                nsdef.fields.iter().find(|(n, _)| n == fname)
            else {
                bail!(
                    "Unknown field '{fname}' in 'structNew {}' at {fspan}",
                    nsdef.original_name
                );
            };
            let val_ty = self.check_subexpr(fexpr, state, ctx)?;
            ensure!(
                !val_ty.is_or_contains_buf(),
                "Linear 'buf/T' cannot be stored in field '{fname}' at {fspan}"
            );
            ensure!(
                types_compatible(&val_ty, expected_ty),
                "Field '{fname}' in 'structNew {}' at {fspan} has type '{val_ty}', expected '{expected_ty}'",
                nsdef.original_name
            );
        }

        let missing: Vec<&str> = expected_fields
            .into_iter()
            .filter(|f| !seen.contains(*f))
            .collect();
        ensure!(
            missing.is_empty(),
            "Missing field(s) in 'structNew {}' at {span}: {}",
            nsdef.original_name,
            missing.join(", ")
        );

        self.resolved_named_structs
            .insert((span.line, span.col), nsdef);
        Ok(StageLType::Struct(StructTypeRef::Canonical {
            namespace: key.0,
            name: key.1,
        }))
    }

    fn check_named_field(
        &mut self,
        base: &Expr,
        field: &str,
        span: Span,
        state: &mut FlowState,
        ctx: &mut BodyCtx,
    ) -> Result<StageLType> {
        ensure!(
            self.dialect == StageDialect::StageR3,
            "StageR 3 named 'field' is not supported in StageL 3 at {span}"
        );
        let base_ty = self.check_subexpr(base, state, ctx)?;
        if base_ty == StageLType::Never {
            return Ok(StageLType::Never);
        }
        let StageLType::Struct(StructTypeRef::Canonical { namespace, name }) =
            &base_ty
        else {
            bail!(
                "Named 'field' projection requires a named struct at {span}, found '{base_ty}'"
            );
        };
        let key = (namespace.clone(), name.clone());
        let Some(nsdef) = self.named_structs.get(&key) else {
            if self.struct_defs.contains_key(&key) {
                bail!(
                    "Imported Stage 1 positional struct '{name}' is opaque in StageR 3 and cannot project field '{field}' at {span}"
                );
            }
            bail!("Unknown struct '{name}' in 'field' at {span}");
        };
        let (_, fty) = nsdef
            .fields
            .iter()
            .find(|(n, _)| n == field)
            .with_context(|| {
                format!(
                    "Unknown field '{field}' on named struct '{name}' at {span}"
                )
            })?;
        Ok(fty.clone())
    }

    fn check_named_with(
        &mut self,
        base: &Expr,
        field: &str,
        value: &Expr,
        span: Span,
        state: &mut FlowState,
        ctx: &mut BodyCtx,
    ) -> Result<StageLType> {
        ensure!(
            self.dialect == StageDialect::StageR3,
            "StageR 3 named 'with' is not supported in StageL 3 at {span}"
        );
        let base_ty = self.check_subexpr(base, state, ctx)?;
        let val_ty = self.check_subexpr(value, state, ctx)?;
        ensure!(
            !val_ty.is_or_contains_buf(),
            "Linear 'buf/T' cannot be stored via 'with' at {span}"
        );
        if base_ty == StageLType::Never {
            return Ok(StageLType::Never);
        }
        let StageLType::Struct(StructTypeRef::Canonical { namespace, name }) =
            &base_ty
        else {
            bail!(
                "Named 'with' update requires a named struct at {span}, found '{base_ty}'"
            );
        };
        let key = (namespace.clone(), name.clone());
        let Some(nsdef) = self.named_structs.get(&key) else {
            if self.struct_defs.contains_key(&key) {
                bail!(
                    "Imported Stage 1 positional struct '{name}' is opaque in StageR 3 and cannot update field '{field}' at {span}"
                );
            }
            bail!("Unknown struct '{name}' in 'with' at {span}");
        };
        let (_, expected) = nsdef
            .fields
            .iter()
            .find(|(n, _)| n == field)
            .with_context(|| {
                format!(
                    "Unknown field '{field}' on named struct '{name}' at {span}"
                )
            })?;
        ensure!(
            types_compatible(&val_ty, expected),
            "Field '{field}' in 'with {name}' at {span} has type '{val_ty}', expected '{expected}'"
        );
        if val_ty == StageLType::Never {
            Ok(StageLType::Never)
        } else {
            Ok(base_ty)
        }
    }

    fn check_enum_variant(
        &mut self,
        enum_ty: &StructTypeRef,
        variant: &str,
        payload: Option<&Expr>,
        span: Span,
        state: &mut FlowState,
        ctx: &mut BodyCtx,
    ) -> Result<StageLType> {
        ensure!(
            self.dialect == StageDialect::StageR3,
            "StageR 3 enum constructor is not supported in StageL 3 at {span}"
        );
        let canon_enum = self.canonicalize_type(
            &StageLType::Nominal(enum_ty.clone()),
            span,
        )?;
        let StageLType::Nominal(StructTypeRef::Canonical { namespace, name }) =
            &canon_enum
        else {
            bail!(
                "Expected 'enum' type in variant constructor at {span}, found '{canon_enum}'"
            );
        };
        let key = (namespace.clone(), name.clone());
        let edef = self.enums.get(&key).cloned().with_context(|| {
            format!("'{name}' is not an 'enum' type at {span}")
        })?;
        let vdef = edef
            .variants
            .iter()
            .find(|v| v.name == variant)
            .with_context(|| {
                format!("Unknown enum variant '{name}:{variant}' at {span}")
            })?;

        match (&vdef.payload, payload) {
            (Some(exp_ty), Some(pexpr)) => {
                let pty = self.check_subexpr(pexpr, state, ctx)?;
                ensure!(
                    !pty.is_or_contains_buf(),
                    "Linear 'buf/T' cannot be stored inside enum payload at {span}"
                );
                ensure!(
                    types_compatible(&pty, exp_ty),
                    "Enum variant '{name}:{variant}' payload at {span} has type '{pty}', expected '{exp_ty}'"
                );
            }
            (Some(exp_ty), None) => bail!(
                "Enum variant '{name}:{variant}' at {span} requires a payload of type '{exp_ty}'"
            ),
            (None, Some(pexpr)) => bail!(
                "Unit enum variant '{name}:{variant}' at {} does not carry a payload",
                pexpr.span()
            ),
            (None, None) => {}
        }

        self.resolved_enum_variants
            .insert((span.line, span.col), (key, variant.to_owned()));
        Ok(canon_enum)
    }

    fn check_core_unary(
        &mut self,
        op: CoreUnaryOp,
        operand: &Expr,
        span: Span,
        state: &mut FlowState,
        ctx: &mut BodyCtx,
    ) -> Result<StageLType> {
        match op {
            CoreUnaryOp::Len => {
                let ty = self.check_buf_target(operand, "len", false, state, ctx)?;
                if ty == StageLType::Never {
                    return Ok(if self.dialect == StageDialect::StageL3 {
                        StageLType::U64
                    } else {
                        StageLType::Never
                    });
                }
                let valid = match self.dialect {
                    StageDialect::StageL3 => {
                        matches!(ty, StageLType::List(_) | StageLType::Buf(_))
                    }
                    StageDialect::StageR3 => {
                        matches!(
                            ty,
                            StageLType::List(_)
                                | StageLType::Buf(_)
                                | StageLType::Dcs
                        )
                    }
                };
                ensure!(
                    valid,
                    "'len' requires sequence or buffer operand at {span}, found '{ty}'"
                );
                Ok(StageLType::U64)
            }
            CoreUnaryOp::BufFreeze => {
                let ty = self.check_buf_target(
                    operand,
                    "bufFreeze",
                    self.dialect == StageDialect::StageR3,
                    state,
                    ctx,
                )?;
                if ty == StageLType::Never {
                    return Ok(StageLType::Never);
                }
                let StageLType::Buf(inner) = ty else {
                    bail!(
                        "'bufFreeze' requires 'buf/T' operand at {span}, found '{ty}'"
                    );
                };
                Ok(StageLType::List(inner))
            }
            CoreUnaryOp::Neg => {
                ensure!(
                    self.dialect == StageDialect::StageL3,
                    "Trapping unary 'neg' at {span} is forbidden in StageR 3 (use StageR total 'neg')"
                );
                let ty = self.check_subexpr(operand, state, ctx)?;
                ensure!(
                    ty == StageLType::Never || ty.is_signed_int(),
                    "'neg' requires a signed integer operand, found '{ty}' at {span}"
                );
                Ok(ty)
            }
            CoreUnaryOp::BitNot => {
                let ty = self.check_subexpr(operand, state, ctx)?;
                let ok = match self.dialect {
                    StageDialect::StageL3 => ty.is_integer(),
                    StageDialect::StageR3 => ty.is_stager_fixed_int(),
                };
                ensure!(
                    ty == StageLType::Never || ok,
                    "'bitNot' requires a fixed-width integer operand, found '{ty}' at {span}"
                );
                Ok(ty)
            }
            CoreUnaryOp::Not => {
                let ty = self.check_subexpr(operand, state, ctx)?;
                ensure!(
                    ty == StageLType::B || ty == StageLType::Never,
                    "'not' requires a 'b' boolean operand, found '{ty}' at {span}"
                );
                Ok(ty)
            }
            CoreUnaryOp::Clz | CoreUnaryOp::Ctz | CoreUnaryOp::Popcnt => {
                let ty = self.check_subexpr(operand, state, ctx)?;
                if ty == StageLType::Never {
                    return Ok(if self.dialect == StageDialect::StageL3 {
                        StageLType::U32
                    } else {
                        StageLType::Never
                    });
                }
                let ok = match self.dialect {
                    StageDialect::StageL3 => ty.is_integer(),
                    StageDialect::StageR3 => ty.is_stager_fixed_int(),
                };
                ensure!(
                    ok,
                    "'{}' requires a fixed-width integer operand, found '{ty}' at {span}",
                    op.keyword()
                );
                Ok(StageLType::U32)
            }
        }
    }

    #[allow(
        clippy::too_many_lines,
        reason = "Uniform CoreBinaryOp dispatch across StageL 3 and StageR 3 rules"
    )]
    fn check_core_binary(
        &mut self,
        op: CoreBinaryOp,
        left: &Expr,
        right: &Expr,
        span: Span,
        state: &mut FlowState,
        ctx: &mut BodyCtx,
    ) -> Result<StageLType> {
        if self.dialect == StageDialect::StageR3
            && matches!(
                op,
                CoreBinaryOp::Add
                    | CoreBinaryOp::Sub
                    | CoreBinaryOp::Mul
                    | CoreBinaryOp::Div
                    | CoreBinaryOp::Rem
                    | CoreBinaryOp::Shl
                    | CoreBinaryOp::Shr
                    | CoreBinaryOp::Get
            )
        {
            bail!(
                "Trapping StageL 3 operator '{}' at {span} is forbidden in StageR 3 (use StageR total '{}')",
                op.keyword(),
                op.keyword()
            );
        }

        if matches!(op, CoreBinaryOp::And | CoreBinaryOp::Or) {
            let lty = self.check_subexpr(left, state, ctx)?;
            ensure!(
                lty == StageLType::B || lty == StageLType::Never,
                "'{}' left operand must be 'b', found '{lty}' at {span}",
                op.keyword()
            );
            let mut rhs_state = state.clone();
            let mut rhs_ctx = ctx.fork_child(
                false,
                ctx.loop_depth,
                ctx.loop_protected_bufs.clone(),
            );
            let rty = self.check_expr(right, &mut rhs_state, &mut rhs_ctx)?;
            ctx.captured_reads.extend(rhs_ctx.captured_reads);
            ctx.called_nested.extend(rhs_ctx.called_nested);
            ensure!(
                rty == StageLType::B || rty == StageLType::Never,
                "'{}' right operand must be 'b', found '{rty}' at {span}",
                op.keyword()
            );
            state.intersect_with(&rhs_state);
            return Ok(if lty == StageLType::Never {
                StageLType::Never
            } else {
                StageLType::B
            });
        }

        if op == CoreBinaryOp::BufPush {
            let buf_ty =
                self.check_buf_target(left, "bufPush", true, state, ctx)?;
            let val_ty = self.check_subexpr(right, state, ctx)?;
            if buf_ty == StageLType::Never {
                return Ok(StageLType::Never);
            }
            let StageLType::Buf(inner) = buf_ty else {
                bail!(
                    "'bufPush' requires 'buf/T' first argument at {span}, found '{buf_ty}'"
                );
            };
            ensure!(
                types_compatible(&val_ty, &inner),
                "'bufPush' element at {span} has type '{val_ty}', expected '{inner}'"
            );
            return Ok(StageLType::Void);
        }

        if op == CoreBinaryOp::BufExtend {
            let buf_ty =
                self.check_buf_target(left, "bufExtend", true, state, ctx)?;
            let list_ty = self.check_subexpr(right, state, ctx)?;
            if buf_ty == StageLType::Never {
                return Ok(StageLType::Never);
            }
            let StageLType::Buf(inner) = buf_ty else {
                bail!(
                    "'bufExtend' requires 'buf/T' first argument at {span}, found '{buf_ty}'"
                );
            };
            let expected_list = StageLType::List(inner);
            ensure!(
                types_compatible(&list_ty, &expected_list),
                "'bufExtend' list at {span} has type '{list_ty}', expected '{expected_list}'"
            );
            return Ok(StageLType::Void);
        }

        let lty = self.check_subexpr(left, state, ctx)?;
        let rty = self.check_subexpr(right, state, ctx)?;

        match op {
            CoreBinaryOp::Add
            | CoreBinaryOp::Sub
            | CoreBinaryOp::Mul
            | CoreBinaryOp::Div
            | CoreBinaryOp::Rem => unify_matching_pair(
                op.keyword(),
                &lty,
                &rty,
                StageLType::is_integer,
                "integer",
                span,
            ),
            CoreBinaryOp::WrapAdd
            | CoreBinaryOp::WrapSub
            | CoreBinaryOp::WrapMul
            | CoreBinaryOp::BitAnd
            | CoreBinaryOp::BitOr
            | CoreBinaryOp::BitXor => {
                let (pred, desc): (fn(&StageLType) -> bool, &str) =
                    match self.dialect {
                        StageDialect::StageL3 => {
                            (StageLType::is_integer, "integer")
                        }
                        StageDialect::StageR3 => (
                            StageLType::is_stager_fixed_int,
                            "fixed-width integer",
                        ),
                    };
                unify_matching_pair(op.keyword(), &lty, &rty, pred, desc, span)
            }
            CoreBinaryOp::Shl | CoreBinaryOp::Shr => {
                if lty == StageLType::Never {
                    ensure!(
                        rty == StageLType::U32 || rty == StageLType::Never,
                        "'{}' shift count must be 'u32', found '{rty}' at {span}",
                        op.keyword()
                    );
                    return Ok(StageLType::Never);
                }
                ensure!(
                    lty.is_integer(),
                    "'{}' target must be integer, found '{lty}' at {span}",
                    op.keyword()
                );
                ensure!(
                    rty == StageLType::U32 || rty == StageLType::Never,
                    "'{}' shift count must be 'u32', found '{rty}' at {span}",
                    op.keyword()
                );
                Ok(lty)
            }
            CoreBinaryOp::Rotl | CoreBinaryOp::Rotr => {
                let ok_target = match self.dialect {
                    StageDialect::StageL3 => lty.is_unsigned_int(),
                    StageDialect::StageR3 => lty.is_stager_unsigned_int(),
                };
                if lty != StageLType::Never {
                    ensure!(
                        ok_target,
                        "'{}' requires an unsigned fixed-width integer operand, found '{lty}' at {span}",
                        op.keyword()
                    );
                }
                ensure!(
                    rty == StageLType::U32 || rty == StageLType::Never,
                    "'{}' rotation count must be 'u32', found '{rty}' at {span}",
                    op.keyword()
                );
                if lty == StageLType::Never || rty == StageLType::Never {
                    Ok(if self.dialect == StageDialect::StageR3 {
                        StageLType::Never
                    } else {
                        lty
                    })
                } else {
                    Ok(lty)
                }
            }
            CoreBinaryOp::Eq | CoreBinaryOp::Ne => {
                match (lty == StageLType::Never, rty == StageLType::Never) {
                    (true, true) => {
                        Ok(if self.dialect == StageDialect::StageR3 {
                            StageLType::Never
                        } else {
                            StageLType::B
                        })
                    }
                    (true, false) => {
                        ensure!(
                            self.type_supports_eq(&rty),
                            "'{}' does not support type '{rty}' at {span}",
                            op.keyword()
                        );
                        Ok(StageLType::B)
                    }
                    (false, true) => {
                        ensure!(
                            self.type_supports_eq(&lty),
                            "'{}' does not support type '{lty}' at {span}",
                            op.keyword()
                        );
                        Ok(StageLType::B)
                    }
                    (false, false) => {
                        ensure!(
                            lty == rty && self.type_supports_eq(&lty),
                            "'{}' requires matching comparable types at {span}, found '{lty}' and '{rty}'",
                            op.keyword()
                        );
                        Ok(StageLType::B)
                    }
                }
            }
            CoreBinaryOp::Lt
            | CoreBinaryOp::Le
            | CoreBinaryOp::Gt
            | CoreBinaryOp::Ge => {
                let (pred, desc): (fn(&StageLType) -> bool, &str) =
                    match self.dialect {
                        StageDialect::StageL3 => {
                            (StageLType::is_integer, "integer")
                        }
                        StageDialect::StageR3 => {
                            (StageLType::is_stager_numeric, "numeric")
                        }
                    };
                let joined = unify_matching_pair(
                    op.keyword(),
                    &lty,
                    &rty,
                    pred,
                    desc,
                    span,
                )?;
                if joined == StageLType::Never
                    && self.dialect == StageDialect::StageR3
                {
                    Ok(StageLType::Never)
                } else {
                    Ok(StageLType::B)
                }
            }
            CoreBinaryOp::Get => {
                ensure!(
                    rty == StageLType::U64 || rty == StageLType::Never,
                    "'get' index must be 'u64', found '{rty}' at {span}"
                );
                if lty == StageLType::Never {
                    return Ok(StageLType::Never);
                }
                match &lty {
                    StageLType::List(inner) | StageLType::Buf(inner) => {
                        Ok((**inner).clone())
                    }
                    _ => bail!(
                        "'get' requires 'l/T' or 'buf/T' operand, found '{lty}' at {span}"
                    ),
                }
            }
            CoreBinaryOp::Concat => match self.dialect {
                StageDialect::StageL3 => unify_matching_pair(
                    "concat",
                    &lty,
                    &rty,
                    |t| matches!(t, StageLType::List(_)),
                    "list",
                    span,
                ),
                StageDialect::StageR3 => unify_matching_pair(
                    "concat",
                    &lty,
                    &rty,
                    |t| {
                        matches!(
                            t,
                            StageLType::List(_)
                                | StageLType::S
                                | StageLType::Dcs
                        )
                    },
                    "sequence ('l/T', 's', or 'dcs')",
                    span,
                ),
            },
            CoreBinaryOp::BufFill => {
                ensure!(
                    rty == StageLType::U64 || rty == StageLType::Never,
                    "'bufFill' count must be 'u64', found '{rty}' at {span}"
                );
                if lty == StageLType::Never {
                    return Ok(StageLType::Never);
                }
                ensure!(
                    type_can_be_element_with_defs(&lty, &self.type_defs()),
                    "'bufFill' element type '{lty}' cannot contain 'buf/T', 'void', or 'never' at {span}"
                );
                if self.dialect == StageDialect::StageR3 {
                    check_stager_value_type_shape(
                        &lty,
                        &self.type_defs(),
                        span,
                    )?;
                }
                Ok(StageLType::Buf(Box::new(lty)))
            }
            CoreBinaryOp::And
            | CoreBinaryOp::Or
            | CoreBinaryOp::BufPush
            | CoreBinaryOp::BufExtend => {
                // Handled directly above.
                bail!("Internal operator dispatch error at {span}")
            }
        }
    }

    fn check_core_ternary(
        &mut self,
        op: CoreTernaryOp,
        first: &Expr,
        second: &Expr,
        third: &Expr,
        span: Span,
        state: &mut FlowState,
        ctx: &mut BodyCtx,
    ) -> Result<StageLType> {
        ensure!(
            self.dialect == StageDialect::StageL3,
            "Trapping StageL 3 ternary '{}' at {span} is forbidden in StageR 3 (use StageR total '{}')",
            op.keyword(),
            op.keyword()
        );
        let first_ty = self.check_subexpr(first, state, ctx)?;
        let second_ty = self.check_subexpr(second, state, ctx)?;
        let third_ty = self.check_subexpr(third, state, ctx)?;
        match op {
            CoreTernaryOp::Slice => {
                ensure!(
                    second_ty == StageLType::U64
                        || second_ty == StageLType::Never,
                    "'slice' start index must be 'u64', found '{second_ty}' at {span}"
                );
                ensure!(
                    third_ty == StageLType::U64
                        || third_ty == StageLType::Never,
                    "'slice' end index must be 'u64', found '{third_ty}' at {span}"
                );
                if first_ty == StageLType::Never {
                    return Ok(StageLType::Never);
                }
                match first_ty {
                    StageLType::List(inner) | StageLType::Buf(inner) => {
                        Ok(StageLType::List(inner))
                    }
                    _ => bail!(
                        "'slice' requires 'l/T' or 'buf/T' first operand, found '{first_ty}' at {span}"
                    ),
                }
            }
            CoreTernaryOp::BufSet => {
                ensure!(
                    second_ty == StageLType::U64
                        || second_ty == StageLType::Never,
                    "'bufSet' index must be 'u64', found '{second_ty}' at {span}"
                );
                if first_ty == StageLType::Never {
                    return Ok(StageLType::Void);
                }
                let StageLType::Buf(inner) = first_ty else {
                    bail!(
                        "'bufSet' requires 'buf/T' first operand, found '{first_ty}' at {span}"
                    );
                };
                ensure!(
                    types_compatible(&third_ty, &inner),
                    "'bufSet' value at {span} has type '{third_ty}', expected '{inner}'"
                );
                Ok(StageLType::Void)
            }
        }
    }

    fn check_stager_unary(
        &mut self,
        op: StageRUnaryOp,
        operand: &Expr,
        span: Span,
        state: &mut FlowState,
        ctx: &mut BodyCtx,
    ) -> Result<StageLType> {
        ensure!(
            self.dialect == StageDialect::StageR3,
            "StageR 3 operator '{}' is not supported in StageL 3 at {span}",
            op.keyword()
        );
        let ty = self.check_subexpr(operand, state, ctx)?;
        if ty == StageLType::Never {
            return Ok(StageLType::Never);
        }
        match op {
            StageRUnaryOp::TotalNeg => match &ty {
                StageLType::I32 | StageLType::I64 | StageLType::I128 => {
                    Ok(StageLType::Result(Box::new(ty)))
                }
                StageLType::Int | StageLType::Rat => Ok(ty),
                _ => bail!(
                    "StageR 'neg' requires signed integer or 'rat' operand, found '{ty}' at {span}"
                ),
            },
            StageRUnaryOp::IsSome | StageRUnaryOp::IsNone => {
                ensure!(
                    matches!(ty, StageLType::Option(_)),
                    "'{}' requires 'option/T' operand, found '{ty}' at {span}",
                    op.keyword()
                );
                Ok(StageLType::B)
            }
            StageRUnaryOp::IsOk | StageRUnaryOp::IsErr => {
                ensure!(
                    matches!(ty, StageLType::Result(_)),
                    "'{}' requires 'result/T' operand, found '{ty}' at {span}",
                    op.keyword()
                );
                Ok(StageLType::B)
            }
            StageRUnaryOp::ByteLen => {
                ensure!(
                    ty == StageLType::S,
                    "'byteLen' requires 's' operand, found '{ty}' at {span}"
                );
                Ok(StageLType::U64)
            }
            StageRUnaryOp::DcFromU128 => {
                ensure!(
                    ty == StageLType::U128,
                    "'dcFromU128' requires 'u128' operand, found '{ty}' at {span}"
                );
                Ok(StageLType::Result(Box::new(StageLType::Dc)))
            }
            StageRUnaryOp::U128FromDc => {
                ensure!(
                    ty == StageLType::Dc,
                    "'u128FromDc' requires 'dc' operand, found '{ty}' at {span}"
                );
                Ok(StageLType::U128)
            }
            StageRUnaryOp::DcsFromS => {
                ensure!(
                    ty == StageLType::S,
                    "'dcsFromS' requires 's' operand, found '{ty}' at {span}"
                );
                Ok(StageLType::Result(Box::new(StageLType::Dcs)))
            }
            StageRUnaryOp::SFromDcs => {
                ensure!(
                    ty == StageLType::Dcs,
                    "'sFromDcs' requires 'dcs' operand, found '{ty}' at {span}"
                );
                Ok(StageLType::S)
            }
        }
    }

    #[allow(
        clippy::too_many_lines,
        reason = "Total arithmetic, option/result, rat, and sequence binary operator checks"
    )]
    fn check_stager_binary(
        &mut self,
        op: StageRBinaryOp,
        left: &Expr,
        right: &Expr,
        span: Span,
        state: &mut FlowState,
        ctx: &mut BodyCtx,
    ) -> Result<StageLType> {
        ensure!(
            self.dialect == StageDialect::StageR3,
            "StageR 3 operator '{}' is not supported in StageL 3 at {span}",
            op.keyword()
        );
        let lty = self.check_subexpr(left, state, ctx)?;
        let rty = self.check_subexpr(right, state, ctx)?;

        match op {
            StageRBinaryOp::TotalAdd
            | StageRBinaryOp::TotalSub
            | StageRBinaryOp::TotalMul => {
                let joined = unify_matching_pair(
                    op.keyword(),
                    &lty,
                    &rty,
                    StageLType::is_stager_numeric,
                    "numeric",
                    span,
                )?;
                if joined == StageLType::Never {
                    return Ok(StageLType::Never);
                }
                if joined.is_stager_fixed_int() {
                    Ok(StageLType::Result(Box::new(joined)))
                } else {
                    Ok(joined)
                }
            }
            StageRBinaryOp::TotalDiv => {
                let joined = unify_matching_pair(
                    "div",
                    &lty,
                    &rty,
                    StageLType::is_stager_numeric,
                    "numeric",
                    span,
                )?;
                if joined == StageLType::Never {
                    Ok(StageLType::Never)
                } else {
                    Ok(StageLType::Result(Box::new(joined)))
                }
            }
            StageRBinaryOp::TotalMod | StageRBinaryOp::TotalTruncRem => {
                let joined = unify_matching_pair(
                    op.keyword(),
                    &lty,
                    &rty,
                    is_stager_integer,
                    "integer",
                    span,
                )?;
                if joined == StageLType::Never {
                    Ok(StageLType::Never)
                } else {
                    Ok(StageLType::Result(Box::new(joined)))
                }
            }
            StageRBinaryOp::TotalShl | StageRBinaryOp::TotalShr => {
                if lty != StageLType::Never {
                    ensure!(
                        lty.is_stager_fixed_int(),
                        "'{}' target must be fixed-width integer, found '{lty}' at {span}",
                        op.keyword()
                    );
                }
                ensure!(
                    rty == StageLType::U32 || rty == StageLType::Never,
                    "'{}' shift count must be 'u32', found '{rty}' at {span}",
                    op.keyword()
                );
                if lty == StageLType::Never || rty == StageLType::Never {
                    Ok(StageLType::Never)
                } else {
                    Ok(StageLType::Result(Box::new(lty)))
                }
            }
            StageRBinaryOp::TotalGet => {
                ensure!(
                    rty == StageLType::U64 || rty == StageLType::Never,
                    "'get' index must be 'u64', found '{rty}' at {span}"
                );
                if lty == StageLType::Never || rty == StageLType::Never {
                    return Ok(StageLType::Never);
                }
                match lty {
                    StageLType::List(inner) => Ok(StageLType::Option(inner)),
                    StageLType::Dcs => {
                        Ok(StageLType::Option(Box::new(StageLType::Dc)))
                    }
                    _ => bail!(
                        "StageR 'get' requires 'l/T' or 'dcs' operand, found '{lty}' at {span}"
                    ),
                }
            }
            StageRBinaryOp::UnwrapOr => {
                if lty == StageLType::Never {
                    return Ok(StageLType::Never);
                }
                let ok_ty = match lty {
                    StageLType::Option(inner) | StageLType::Result(inner) => {
                        *inner
                    }
                    _ => bail!(
                        "'unwrapOr' requires 'option/T' or 'result/T' first operand, found '{lty}' at {span}"
                    ),
                };
                ensure!(
                    types_compatible(&rty, &ok_ty),
                    "'unwrapOr' fallback at {span} has type '{rty}', expected '{ok_ty}'"
                );
                Ok(ok_ty)
            }
            StageRBinaryOp::OkOr => {
                ensure!(
                    rty == StageLType::Error || rty == StageLType::Never,
                    "'okOr' second operand must be 'error', found '{rty}' at {span}"
                );
                if lty == StageLType::Never {
                    return Ok(StageLType::Never);
                }
                let StageLType::Option(inner) = lty else {
                    bail!(
                        "'okOr' requires 'option/T' first operand, found '{lty}' at {span}"
                    );
                };
                Ok(StageLType::Result(inner))
            }
            StageRBinaryOp::MakeRat => {
                ensure!(
                    lty == StageLType::Int || lty == StageLType::Never,
                    "'makeRat' numerator must be 'int', found '{lty}' at {span}"
                );
                ensure!(
                    rty == StageLType::Int || rty == StageLType::Never,
                    "'makeRat' denominator must be 'int', found '{rty}' at {span}"
                );
                if lty == StageLType::Never || rty == StageLType::Never {
                    Ok(StageLType::Never)
                } else {
                    Ok(StageLType::Result(Box::new(StageLType::Rat)))
                }
            }
            StageRBinaryOp::Explode => {
                let seq = match (&lty, &rty) {
                    (StageLType::Never, _) | (_, StageLType::Never) => {
                        return Ok(StageLType::Never);
                    }
                    (StageLType::S, StageLType::S) => StageLType::S,
                    (StageLType::Dcs, StageLType::Dcs) => StageLType::Dcs,
                    (StageLType::List(a), StageLType::List(b)) => {
                        ensure!(
                            a == b,
                            "'explode' list types must match at {span}: '{lty}' vs '{rty}'"
                        );
                        ensure!(
                            self.type_supports_eq(a),
                            "'explode' element type '{a}' must support 'eq' at {span}"
                        );
                        StageLType::List(a.clone())
                    }
                    _ => bail!(
                        "'explode' requires matching 's', 'dcs', or 'l/T' operands at {span}, found '{lty}' and '{rty}'"
                    ),
                };
                Ok(StageLType::List(Box::new(seq)))
            }
            StageRBinaryOp::Implode => match (&lty, &rty) {
                (StageLType::Never, _) | (_, StageLType::Never) => {
                    Ok(StageLType::Never)
                }
                (seq, StageLType::List(part_elem)) => {
                    ensure!(
                        matches!(
                            seq,
                            StageLType::S
                                | StageLType::Dcs
                                | StageLType::List(_)
                        ),
                        "'implode' separator must be 's', 'dcs', or 'l/T', found '{seq}' at {span}"
                    );
                    ensure!(
                        seq == part_elem.as_ref(),
                        "'implode' parts list element '{part_elem}' must match separator '{seq}' at {span}"
                    );
                    Ok(seq.clone())
                }
                (sep, other) => bail!(
                    "'implode' expects 'l/({sep})' parts list as second operand, found '{other}' at {span}"
                ),
            },
        }
    }

    fn check_stager_ternary(
        &mut self,
        op: StageRTernaryOp,
        first: &Expr,
        second: &Expr,
        third: &Expr,
        span: Span,
        state: &mut FlowState,
        ctx: &mut BodyCtx,
    ) -> Result<StageLType> {
        ensure!(
            self.dialect == StageDialect::StageR3,
            "StageR 3 operator '{}' is not supported in StageL 3 at {span}",
            op.keyword()
        );
        match op {
            StageRTernaryOp::TotalSlice => {
                let seq_ty = self.check_subexpr(first, state, ctx)?;
                let start_ty = self.check_subexpr(second, state, ctx)?;
                let end_ty = self.check_subexpr(third, state, ctx)?;
                ensure!(
                    start_ty == StageLType::U64
                        || start_ty == StageLType::Never,
                    "StageR 'slice' start index must be 'u64', found '{start_ty}' at {span}"
                );
                ensure!(
                    end_ty == StageLType::U64 || end_ty == StageLType::Never,
                    "StageR 'slice' end index must be 'u64', found '{end_ty}' at {span}"
                );
                if seq_ty == StageLType::Never
                    || start_ty == StageLType::Never
                    || end_ty == StageLType::Never
                {
                    return Ok(StageLType::Never);
                }
                match seq_ty {
                    StageLType::List(_)
                    | StageLType::S
                    | StageLType::Dcs => Ok(StageLType::Option(Box::new(seq_ty))),
                    _ => bail!(
                        "StageR 'slice' requires 'l/T', 's', or 'dcs', found '{seq_ty}' at {span}"
                    ),
                }
            }
            StageRTernaryOp::TotalBufSet => {
                let buf_ty =
                    self.check_buf_target(first, "bufSet", true, state, ctx)?;
                let idx_ty = self.check_subexpr(second, state, ctx)?;
                let val_ty = self.check_subexpr(third, state, ctx)?;
                ensure!(
                    idx_ty == StageLType::U64 || idx_ty == StageLType::Never,
                    "StageR 'bufSet' index must be 'u64', found '{idx_ty}' at {span}"
                );
                if buf_ty == StageLType::Never {
                    return Ok(StageLType::Never);
                }
                let StageLType::Buf(inner) = buf_ty else {
                    bail!(
                        "StageR 'bufSet' requires 'buf/T' first operand, found '{buf_ty}' at {span}"
                    );
                };
                ensure!(
                    types_compatible(&val_ty, &inner),
                    "StageR 'bufSet' value at {span} has type '{val_ty}', expected '{inner}'"
                );
                Ok(StageLType::Result(Box::new(StageLType::Void)))
            }
        }
    }

    fn check_total_conv(
        &mut self,
        expr: &Expr,
        target_ty: &StageLType,
        span: Span,
        state: &mut FlowState,
        ctx: &mut BodyCtx,
    ) -> Result<StageLType> {
        ensure!(
            self.dialect == StageDialect::StageR3,
            "StageR 3 total 'conv' is not supported in StageL 3 at {span}"
        );
        let src_ty = self.check_subexpr(expr, state, ctx)?;
        let dst_ty = self.check_and_canonicalize_type(
            target_ty,
            BufPolicy::Disallowed,
            false,
            false,
            span,
        )?;
        ensure!(
            dst_ty.is_stager_numeric(),
            "StageR 'conv' target type must be numeric, found '{dst_ty}' at {span}"
        );
        if src_ty == StageLType::Never {
            return Ok(StageLType::Never);
        }
        ensure!(
            src_ty.is_stager_numeric(),
            "StageR 'conv' source operand must be numeric, found '{src_ty}' at {span}"
        );
        if is_stager_widening_conv(&src_ty, &dst_ty) {
            Ok(dst_ty)
        } else {
            Ok(StageLType::Result(Box::new(dst_ty)))
        }
    }
}
