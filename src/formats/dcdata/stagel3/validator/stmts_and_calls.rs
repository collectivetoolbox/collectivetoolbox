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

//! Routine body, statement, lexical scope, control flow, `match`
//! exhaustiveness, propagation `/`, linear buffer target checking, and
//! overload-aware call validation.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::super::ast::{
    BlockItem, CoreBinaryOp, Expr, MatchArm, RoutineDecl, Span, StageDialect,
    StageLType, StageRErrorTag, StructTypeRef, SymbolTarget,
};
use super::consts_and_flow::{
    BindingEntry, BindingKind, BodyCtx, BufState, FlowState, NestedVerbSummary,
    VerbTarget,
};
use super::imports_and_decls::{ModuleValidator, format_param_types};
use super::types::{
    BufPolicy, check_stager_value_type_shape, check_type_well_formed,
    types_compatible,
};
use super::workspace::is_stagel_internal_name;
use super::{
    EnumDef, ImportedSymbol, ResolvedCallTarget, ResolvedValueTarget,
    RoutineSig,
};
use std::collections::{HashMap, HashSet};

/// Resolved StageR 3 `match` scrutinee domain.
enum MatchDomain {
    Option(StageLType),
    Result(StageLType),
    Error,
    Enum { def: EnumDef },
}

impl<'a> ModuleValidator<'a> {
    /// Validates a top-level routine body against its canonicalized parameters
    /// and return type.
    pub(super) fn validate_routine_body(
        &mut self,
        r: &'a RoutineDecl,
        canon_params: &[StageLType],
        canon_ret: &StageLType,
    ) -> Result<()> {
        let mut param_scope = HashMap::new();
        let mut state = FlowState::new();
        for (p, canon_pty) in r.params.iter().zip(canon_params) {
            ensure!(
                !param_scope.contains_key(&p.name)
                    && !self.name_exists_in_local_or_top(&p.name),
                "Parameter '{}' of routine '{}' at {} duplicates/shadows an existing binding or symbol",
                p.name,
                r.name,
                p.span
            );
            let id = self.alloc_binding_id()?;
            state.assign.mark_assigned(id);
            param_scope.insert(
                p.name.clone(),
                BindingEntry {
                    id,
                    kind: BindingKind::Param,
                    ty: canon_pty.clone(),
                    decl_span: p.span,
                },
            );
        }

        self.local_scopes.push(param_scope);
        let allow_propagate = self.dialect == StageDialect::StageR3
            && matches!(
                canon_ret,
                StageLType::Option(_) | StageLType::Result(_)
            );
        let mut ctx = BodyCtx::new(canon_ret.clone(), allow_propagate);
        let body_res = self.check_expr(&r.body, &mut state, &mut ctx);
        self.local_scopes.pop();
        let body_ty = body_res?;

        if self.dialect == StageDialect::StageR3 {
            ensure!(
                !body_ty.is_or_contains_buf(),
                "Linear 'buf/T' cannot be returned from routine '{}' at {}",
                r.name,
                r.span
            );
        }
        ensure!(
            types_compatible(&body_ty, canon_ret),
            "Body of routine '{}' at {} evaluates to '{}', expected return type '{}'",
            r.name,
            r.span,
            body_ty,
            canon_ret
        );
        Ok(())
    }

    /// Validates a nested `routine` inside a `do` block and records its
    /// captured reads / nested call graph summary.
    fn validate_nested_routine_at_decl(
        &mut self,
        r: &RoutineDecl,
        nid: usize,
        canon_params: &[StageLType],
        canon_ret: &StageLType,
        enclosing_state: &FlowState,
        outer_ids: &HashSet<usize>,
    ) -> Result<()> {
        ensure!(
            r.tests.is_empty(),
            "Inline 'test' blocks are not allowed on nested routine '{}' at {}",
            r.name,
            r.span
        );

        let saved_pending = std::mem::take(&mut self.pending_nested_calls);
        let mut nested_state = enclosing_state.clone();
        for &oid in outer_ids {
            nested_state.assign.mark_assigned(oid);
        }

        let mut param_scope = HashMap::new();
        for (p, canon_pty) in r.params.iter().zip(canon_params) {
            if self.forbid_stagel_internal && is_stagel_internal_name(&p.name) {
                self.pending_nested_calls = saved_pending;
                bail!(
                    "StageL-internal parameter name '{}' in nested routine '{}' at {} is forbidden",
                    p.name,
                    r.name,
                    p.span
                );
            }
            if param_scope.contains_key(&p.name)
                || self.name_exists_in_local_or_top(&p.name)
            {
                self.pending_nested_calls = saved_pending;
                bail!(
                    "Parameter '{}' of nested routine '{}' at {} duplicates/shadows an existing binding or symbol",
                    p.name,
                    r.name,
                    p.span
                );
            }
            let id = self.alloc_binding_id()?;
            nested_state.assign.mark_assigned(id);
            param_scope.insert(
                p.name.clone(),
                BindingEntry {
                    id,
                    kind: BindingKind::Param,
                    ty: canon_pty.clone(),
                    decl_span: p.span,
                },
            );
        }

        self.local_scopes.push(param_scope);
        let allow_propagate = self.dialect == StageDialect::StageR3
            && matches!(
                canon_ret,
                StageLType::Option(_) | StageLType::Result(_)
            );
        let mut ctx = BodyCtx::for_nested(
            canon_ret.clone(),
            allow_propagate,
            outer_ids.clone(),
        );
        let body_res = self.check_expr(&r.body, &mut nested_state, &mut ctx);
        self.local_scopes.pop();
        self.pending_nested_calls = saved_pending;
        let body_ty = body_res?;

        if self.dialect == StageDialect::StageR3 {
            ensure!(
                !body_ty.is_or_contains_buf(),
                "Linear 'buf/T' cannot be returned from nested routine '{}' at {}",
                r.name,
                r.span
            );
        }
        ensure!(
            types_compatible(&body_ty, canon_ret),
            "Body of nested routine '{}' at {} evaluates to '{}', expected return type '{}'",
            r.name,
            r.span,
            body_ty,
            canon_ret
        );

        if let Some(sum) = self.nested_summaries.get_mut(&nid) {
            sum.captured_reads = ctx.captured_reads;
            if *canon_ret != StageLType::Never {
                sum.always_written = ctx.always_written;
            }
            sum.called_nested = ctx.called_nested;
            sum.captures_resolved = true;
        }
        Ok(())
    }

    /// Resolves a variable/constant reference (`ValueRef`), checking definite
    /// assignment, StageL-internal gating, nested captures, and linear buffer
    /// value rules.
    pub(super) fn check_value_ref(
        &mut self,
        ty: &Option<StageLType>,
        target: &SymbolTarget,
        span: Span,
        state: &mut FlowState,
        ctx: &mut BodyCtx,
        allow_buf_direct: bool,
    ) -> Result<StageLType> {
        match target {
            SymbolTarget::Ident(name) => {
                if self.forbid_stagel_internal && is_stagel_internal_name(name)
                {
                    bail!(
                        "StageL-internal value '{name}' cannot be referenced in Stage 2/StageR module at {span}"
                    );
                }
                if let Some(entry) = self.lookup_local(name).cloned() {
                    if let Some(ann) = ty {
                        let policy = if self.dialect == StageDialect::StageL3 {
                            BufPolicy::LocalAllowed
                        } else if allow_buf_direct {
                            BufPolicy::StageRDirectBufOnly
                        } else {
                            BufPolicy::Disallowed
                        };
                        let canon_ann = self.check_and_canonicalize_type(
                            ann, policy, false, false, span,
                        )?;
                        ensure!(
                            canon_ann == entry.ty,
                            "Type annotation '{canon_ann}' on reference to '{name}' at {span} does not match declared type '{}'",
                            entry.ty
                        );
                    }
                    ensure!(
                        state.assign.is_assigned(entry.id),
                        "Use of potentially uninitialized variable '{name}' before definite assignment at {span} (declared at {})",
                        entry.decl_span
                    );
                    if ctx.outer_ids.contains(&entry.id) {
                        if self.dialect == StageDialect::StageR3
                            && matches!(entry.ty, StageLType::Buf(_))
                        {
                            bail!(
                                "Linear buffer '{name}' at {span} cannot be captured or accessed inside a nested routine"
                            );
                        }
                        if !ctx.always_written.contains(&entry.id) {
                            ctx.captured_reads.insert(entry.id);
                        }
                    }
                    if self.dialect == StageDialect::StageR3
                        && matches!(entry.ty, StageLType::Buf(_))
                        && !allow_buf_direct
                    {
                        bail!(
                            "Linear 'buf/T' variable '{name}' at {span} cannot be moved or passed as a value; use buffer verbs directly on '/{name}'"
                        );
                    }
                    self.value_ref_types
                        .insert((span.line, span.col), entry.ty.clone());
                    return Ok(entry.ty);
                }

                if let Some(resolved) = self.value_candidates.get(name).cloned()
                {
                    if let Some(ann) = ty {
                        let canon_ann = self.canonicalize_type(ann, span)?;
                        ensure!(
                            canon_ann == resolved.ty,
                            "Type annotation '{canon_ann}' on reference to constant '{name}' at {span} does not match declared type '{}'",
                            resolved.ty
                        );
                    }
                    self.resolved_values
                        .insert((span.line, span.col), resolved.clone());
                    self.value_ref_types
                        .insert((span.line, span.col), resolved.ty.clone());
                    return Ok(resolved.ty);
                }

                bail!("Unknown variable or constant '{name}' at {span}");
            }
            SymbolTarget::Dc(dc) => {
                if let Some(val_ty) = self.top_dc_values.get(dc).cloned() {
                    if let Some(orig_name) = self.top_dc_names.get(dc).cloned()
                        && let Some(resolved) =
                            self.value_candidates.get(&orig_name).cloned()
                    {
                        self.resolved_values
                            .insert((span.line, span.col), resolved);
                    }
                    if let Some(ann) = ty {
                        let canon_ann = self.canonicalize_type(ann, span)?;
                        ensure!(
                            canon_ann == val_ty,
                            "Type annotation '{canon_ann}' on Dc reference at {span} does not match constant type '{val_ty}'"
                        );
                    }
                    self.value_ref_types
                        .insert((span.line, span.col), val_ty.clone());
                    return Ok(val_ty);
                }
                if let Some(sym) = self.dc_imports.get(dc).cloned() {
                    match sym {
                        ImportedSymbol::Value {
                            namespace,
                            original_name,
                            ty: val_ty,
                            dc_id,
                        } => {
                            if self.forbid_stagel_internal
                                && is_stagel_internal_name(&original_name)
                            {
                                bail!(
                                    "StageL-internal imported constant '{original_name}' cannot be referenced in Stage 2/StageR module at {span}"
                                );
                            }
                            check_type_well_formed(
                                &val_ty,
                                &self.type_defs(),
                                self.dialect,
                                BufPolicy::Disallowed,
                                span,
                            )?;
                            if let Some(ann) = ty {
                                let canon_ann =
                                    self.canonicalize_type(ann, span)?;
                                ensure!(
                                    canon_ann == val_ty,
                                    "Type annotation '{canon_ann}' on Dc reference to '{original_name}' at {span} does not match constant type '{val_ty}'"
                                );
                            }
                            self.resolved_values.insert(
                                (span.line, span.col),
                                ResolvedValueTarget {
                                    namespace: Some(namespace),
                                    original_name,
                                    ty: val_ty.clone(),
                                    dc_id,
                                },
                            );
                            self.value_ref_types
                                .insert((span.line, span.col), val_ty.clone());
                            return Ok(val_ty);
                        }
                        _ => bail!(
                            "Dc reference at {span} does not refer to a constant value"
                        ),
                    }
                }
                bail!("Unresolved Dc value reference at {span}");
            }
        }
    }

    /// Validates direct buffer verb target `/b` (for `bufPush`, `bufExtend`,
    /// `bufFreeze`, and `len`), enforcing StageR 3 linearity and loop freeze
    /// protection.
    pub(super) fn check_buf_target(
        &mut self,
        expr: &Expr,
        op_name: &str,
        require_buf: bool,
        state: &mut FlowState,
        ctx: &mut BodyCtx,
    ) -> Result<StageLType> {
        if self.dialect == StageDialect::StageL3 {
            return self.check_subexpr(expr, state, ctx);
        }

        if let Expr::ValueRef {
            ty,
            target: SymbolTarget::Ident(name),
            span,
        } = expr
            && let Some(entry) = self.lookup_local(name).cloned()
            && matches!(entry.ty, StageLType::Buf(_))
        {
            let target_sym = SymbolTarget::Ident(name.clone());
            let prev_tail = ctx.in_tail;
            ctx.in_tail = false;
            let ref_res = self.check_value_ref(
                ty,
                &target_sym,
                *span,
                state,
                ctx,
                true,
            );
            ctx.in_tail = prev_tail;
            let buf_ty = ref_res?;

            match state.bufs.get(&entry.id).copied() {
                Some(BufState::Unfrozen) => {}
                Some(BufState::Frozen) => bail!(
                    "Linear buffer '{name}' was already consumed by 'bufFreeze' and cannot be used in '{op_name}' at {span}"
                ),
                None => bail!(
                    "Linear buffer '{name}' is not active in current scope for '{op_name}' at {span}"
                ),
            }

            if op_name == "bufFreeze" {
                ensure!(
                    !ctx.loop_protected_bufs.contains(&entry.id),
                    "Cannot freeze outer buffer '{name}' with 'bufFreeze' inside a loop at {span}"
                );
                state.bufs.insert(entry.id, BufState::Frozen);
            }

            self.expr_types
                .insert((span.line, span.col), buf_ty.clone());
            return Ok(buf_ty);
        }

        if require_buf {
            bail!(
                "'{op_name}' in StageR 3 requires a local 'buf/T' identifier target at {}",
                expr.span()
            );
        }
        self.check_subexpr(expr, state, ctx)
    }

    /// Validates a mutable local declaration (`let TYPE/name [init]`).
    pub(super) fn check_let(
        &mut self,
        ty: &StageLType,
        name: &str,
        init: Option<&Expr>,
        span: Span,
        state: &mut FlowState,
        ctx: &mut BodyCtx,
    ) -> Result<StageLType> {
        if self.forbid_stagel_internal && is_stagel_internal_name(name) {
            bail!(
                "StageL-internal variable name '{name}' cannot be declared in Stage 2/StageR module at {span}"
            );
        }
        ensure!(
            !self.name_exists_in_local_or_top(name),
            "Local variable '{name}' at {span} duplicates/shadows an existing binding or symbol"
        );

        let buf_policy = match self.dialect {
            StageDialect::StageL3 => BufPolicy::LocalAllowed,
            StageDialect::StageR3 => BufPolicy::StageRDirectBufOnly,
        };
        let canon_ty = self.check_and_canonicalize_type(
            ty, buf_policy, false, false, span,
        )?;

        if self.dialect == StageDialect::StageR3 {
            if matches!(canon_ty, StageLType::Buf(_)) {
                let init_expr = init.with_context(|| {
                    format!(
                        "Linear 'buf/T' variable '{name}' at {span} must be immediately initialized with 'bufNew' or 'bufFill'"
                    )
                })?;
                ensure!(
                    matches!(
                        init_expr,
                        Expr::BufNew { .. }
                            | Expr::Binary {
                                op: CoreBinaryOp::BufFill,
                                ..
                            }
                    ),
                    "Linear 'buf/T' variable '{name}' at {span} must be initialized directly with 'bufNew' or 'bufFill'"
                );
                let init_ty = self.check_subexpr(init_expr, state, ctx)?;
                ensure!(
                    types_compatible(&init_ty, &canon_ty),
                    "Buffer variable '{name}' at {span} has declared type '{canon_ty}', initialized with '{init_ty}'"
                );
                let id = self.alloc_binding_id()?;
                state.assign.mark_assigned(id);
                state.bufs.insert(id, BufState::Unfrozen);
                if let Some(scope) = self.local_scopes.last_mut() {
                    scope.insert(
                        name.to_owned(),
                        BindingEntry {
                            id,
                            kind: BindingKind::Let,
                            ty: canon_ty,
                            decl_span: span,
                        },
                    );
                }
                return Ok(StageLType::Void);
            }
            check_stager_value_type_shape(&canon_ty, &self.type_defs(), span)?;
        }

        let id = self.alloc_binding_id()?;
        if let Some(init_expr) = init {
            let init_ty = self.check_subexpr(init_expr, state, ctx)?;
            if self.dialect == StageDialect::StageR3 {
                ensure!(
                    !init_ty.is_or_contains_buf(),
                    "Linear 'buf/T' cannot initialize non-buffer binding '{name}' at {span}"
                );
            }
            ensure!(
                types_compatible(&init_ty, &canon_ty),
                "Variable '{name}' at {span} has declared type '{canon_ty}', initialized with '{init_ty}'"
            );
            state.assign.mark_assigned(id);
        }

        if let Some(scope) = self.local_scopes.last_mut() {
            scope.insert(
                name.to_owned(),
                BindingEntry {
                    id,
                    kind: BindingKind::Let,
                    ty: canon_ty,
                    decl_span: span,
                },
            );
        }
        Ok(StageLType::Void)
    }

    /// Validates an immutable local constant (`const TYPE/name value`).
    pub(super) fn check_local_const(
        &mut self,
        ty: &StageLType,
        name: &str,
        value: &Expr,
        span: Span,
        state: &mut FlowState,
        ctx: &mut BodyCtx,
    ) -> Result<StageLType> {
        if self.forbid_stagel_internal && is_stagel_internal_name(name) {
            bail!(
                "StageL-internal constant name '{name}' cannot be declared in Stage 2/StageR module at {span}"
            );
        }
        ensure!(
            !self.name_exists_in_local_or_top(name),
            "Local constant '{name}' at {span} duplicates/shadows an existing binding or symbol"
        );

        let buf_policy = match self.dialect {
            StageDialect::StageL3 => BufPolicy::LocalAllowed,
            StageDialect::StageR3 => BufPolicy::Disallowed,
        };
        let canon_ty = self.check_and_canonicalize_type(
            ty, buf_policy, false, false, span,
        )?;
        if self.dialect == StageDialect::StageR3 {
            check_stager_value_type_shape(&canon_ty, &self.type_defs(), span)?;
        }

        let val_ty = self.check_subexpr(value, state, ctx)?;
        if self.dialect == StageDialect::StageR3 {
            ensure!(
                !val_ty.is_or_contains_buf(),
                "Linear 'buf/T' cannot initialize local constant '{name}' at {span}"
            );
        }
        ensure!(
            types_compatible(&val_ty, &canon_ty),
            "Local constant '{name}' at {span} has declared type '{canon_ty}', initialized with '{val_ty}'"
        );

        let id = self.alloc_binding_id()?;
        state.assign.mark_assigned(id);
        if let Some(scope) = self.local_scopes.last_mut() {
            scope.insert(
                name.to_owned(),
                BindingEntry {
                    id,
                    kind: BindingKind::Const,
                    ty: canon_ty,
                    decl_span: span,
                },
            );
        }
        Ok(StageLType::Void)
    }

    /// Validates mutable variable assignment (`set [TYPE/]name value`).
    pub(super) fn check_set(
        &mut self,
        ty: &Option<StageLType>,
        name: &str,
        value: &Expr,
        span: Span,
        state: &mut FlowState,
        ctx: &mut BodyCtx,
    ) -> Result<StageLType> {
        let Some(entry) = self.lookup_local(name).cloned() else {
            if self.value_candidates.contains_key(name) {
                bail!("Cannot reassign module constant '{name}' at {span}");
            }
            bail!("Unknown variable '{name}' in 'set' at {span}");
        };
        match entry.kind {
            BindingKind::Let => {}
            BindingKind::Const => {
                bail!("Cannot reassign immutable constant '{name}' at {span}");
            }
            BindingKind::Param => {
                bail!(
                    "Cannot reassign immutable routine parameter '{name}' at {span}"
                );
            }
        }
        if self.dialect == StageDialect::StageR3
            && matches!(entry.ty, StageLType::Buf(_))
        {
            bail!(
                "Cannot re-assign linear buffer '{name}' at {span} with 'set'"
            );
        }

        if let Some(ann) = ty {
            let policy = match self.dialect {
                StageDialect::StageL3 => BufPolicy::LocalAllowed,
                StageDialect::StageR3 => BufPolicy::Disallowed,
            };
            let canon_ann = self.check_and_canonicalize_type(
                ann, policy, false, false, span,
            )?;
            ensure!(
                canon_ann == entry.ty,
                "Type annotation '{canon_ann}' on 'set' for '{name}' at {span} does not match declared type '{}'",
                entry.ty
            );
        }

        let val_ty = self.check_subexpr(value, state, ctx)?;
        if self.dialect == StageDialect::StageR3 {
            ensure!(
                !val_ty.is_or_contains_buf(),
                "Linear 'buf/T' cannot be assigned via 'set' at {span}"
            );
        }
        ensure!(
            types_compatible(&val_ty, &entry.ty),
            "Cannot assign value of type '{val_ty}' to variable '{name}' of type '{}' at {span}",
            entry.ty
        );
        state.assign.mark_assigned(entry.id);
        if ctx.outer_ids.contains(&entry.id) {
            ctx.always_written.insert(entry.id);
        }
        Ok(StageLType::Void)
    }

    /// Validates a direct routine call (`SymbolTarget::Ident` or
    /// `SymbolTarget::Dc`), supporting nested routines, overloaded top/imported
    /// routines, and tail-call span tracking.
    pub(super) fn check_call(
        &mut self,
        target: &SymbolTarget,
        args: &[Expr],
        span: Span,
        state: &mut FlowState,
        ctx: &mut BodyCtx,
    ) -> Result<StageLType> {
        let was_in_tail = ctx.in_tail;
        let mut arg_tys = Vec::with_capacity(args.len());
        for a in args {
            let aty = self.check_subexpr(a, state, ctx)?;
            if self.dialect == StageDialect::StageR3 {
                ensure!(
                    !aty.is_or_contains_buf(),
                    "Linear 'buf/T' cannot be passed as a call argument at {}",
                    a.span()
                );
            }
            arg_tys.push(aty);
        }

        match target {
            SymbolTarget::Ident(name) => {
                if self.forbid_stagel_internal && is_stagel_internal_name(name)
                {
                    bail!(
                        "StageL-internal routine '{name}' cannot be called in Stage 2/StageR module at {span}"
                    );
                }

                if let Some(verb) = self.lookup_nested_routine(name).cloned() {
                    ensure!(
                        arg_tys.len() == verb.sig.params.len(),
                        "Nested routine '{name}' at {span} expects {} argument(s), found {}",
                        verb.sig.params.len(),
                        arg_tys.len()
                    );
                    for (idx, (aty, pty)) in
                        arg_tys.iter().zip(&verb.sig.params).enumerate()
                    {
                        let arg_pos = idx.checked_add(1).unwrap_or(idx); // Reason for fallback: formatting-only 1-based position on usize::MAX
                        ensure!(
                            types_compatible(aty, pty),
                            "Call argument type mismatch for nested routine '{name}' argument {arg_pos} at {span}: expected '{pty}', found '{aty}'"
                        );
                    }
                    ctx.called_nested.insert(verb.id);
                    self.pending_nested_calls.push((
                        verb.id,
                        span,
                        state.assign.clone(),
                    ));
                    if let Some(summary) =
                        self.nested_summaries.get(&verb.id).cloned()
                        && summary.captures_resolved
                    {
                        for cap_id in &summary.captured_reads {
                            ensure!(
                                state.assign.is_assigned(*cap_id),
                                "Calling nested routine '{name}' at {span} before captured 'let' variable is definitely assigned"
                            );
                            if ctx.outer_ids.contains(cap_id)
                                && !ctx.always_written.contains(cap_id)
                            {
                                ctx.captured_reads.insert(*cap_id);
                            }
                        }
                        for &written_id in &summary.always_written {
                            state.assign.mark_assigned(written_id);
                            ctx.always_written.insert(written_id);
                        }
                    }
                    if was_in_tail {
                        self.tail_calls.insert((span.line, span.col));
                    }
                    return Ok(verb.sig.return_type);
                }

                if let Some(cands) = self.routine_candidates.get(name).cloned()
                {
                    let Some(first_cand) = cands.first() else {
                        bail!("Unknown routine '{name}' at {span}");
                    };
                    let expected_arity = first_cand.sig.params.len();
                    ensure!(
                        arg_tys.len() == expected_arity,
                        "Routine '{name}' at {span} expects {expected_arity} argument(s), found {}",
                        arg_tys.len()
                    );

                    let matches: Vec<&ResolvedCallTarget> = cands
                        .iter()
                        .filter(|cand| {
                            cand.sig
                                .params
                                .iter()
                                .zip(&arg_tys)
                                .all(|(p, a)| types_compatible(a, p))
                        })
                        .collect();

                    if matches.is_empty() {
                        let sigs_desc = cands
                            .iter()
                            .map(|c| {
                                format!(
                                    "({}) -> {}",
                                    format_param_types(&c.sig.params),
                                    c.sig.return_type
                                )
                            })
                            .collect::<Vec<_>>()
                            .join(" | ");
                        bail!(
                            "No matching overload for routine '{name}' at {span} with argument types ({}) (Call argument type mismatch; available signatures: {sigs_desc})",
                            format_param_types(&arg_tys)
                        );
                    }
                    if matches.len() > 1 {
                        bail!(
                            "Ambiguous call to overloaded routine '{name}' at {span} with argument types ({})",
                            format_param_types(&arg_tys)
                        );
                    }
                    let Some(winner) = matches.first().copied() else {
                        bail!("Internal empty overload match for '{name}'");
                    };
                    self.resolved_calls
                        .insert((span.line, span.col), winner.clone());
                    if was_in_tail {
                        self.tail_calls.insert((span.line, span.col));
                    }
                    return Ok(winner.sig.return_type.clone());
                }

                bail!("Unknown routine '{name}' at {span}");
            }
            SymbolTarget::Dc(dc) => {
                if let Some(sig) = self.top_dc_routines.get(dc).cloned() {
                    ensure!(
                        arg_tys.len() == sig.params.len(),
                        "Routine Dc reference at {span} expects {} argument(s), found {}",
                        sig.params.len(),
                        arg_tys.len()
                    );
                    for (idx, (aty, pty)) in
                        arg_tys.iter().zip(&sig.params).enumerate()
                    {
                        let arg_pos = idx.checked_add(1).unwrap_or(idx); // Reason for fallback: formatting-only 1-based position on usize::MAX
                        ensure!(
                            types_compatible(aty, pty),
                            "Call argument type mismatch at argument {arg_pos} of Dc call at {span}: expected '{pty}', found '{aty}'"
                        );
                    }
                    if let Some(orig_name) = self.top_dc_names.get(dc).cloned()
                    {
                        self.resolved_calls.insert(
                            (span.line, span.col),
                            ResolvedCallTarget {
                                namespace: None,
                                routine_name: orig_name,
                                sig: sig.clone(),
                            },
                        );
                    }
                    if was_in_tail {
                        self.tail_calls.insert((span.line, span.col));
                    }
                    return Ok(sig.return_type);
                }

                if let Some(sym) = self.dc_imports.get(dc).cloned() {
                    match sym {
                        ImportedSymbol::Routine {
                            namespace,
                            original_name,
                            sig,
                        } => {
                            if self.forbid_stagel_internal
                                && is_stagel_internal_name(&original_name)
                            {
                                bail!(
                                    "StageL-internal routine '{original_name}' cannot be called in Stage 2/StageR module at {span}"
                                );
                            }
                            let defs = self.type_defs();
                            check_type_well_formed(
                                &sig.return_type,
                                &defs,
                                self.dialect,
                                BufPolicy::Disallowed,
                                span,
                            )?;
                            for pty in &sig.params {
                                check_type_well_formed(
                                    pty,
                                    &defs,
                                    self.dialect,
                                    BufPolicy::Disallowed,
                                    span,
                                )?;
                            }
                            ensure!(
                                arg_tys.len() == sig.params.len(),
                                "Imported routine '{original_name}' at {span} expects {} argument(s), found {}",
                                sig.params.len(),
                                arg_tys.len()
                            );
                            for (idx, (aty, pty)) in
                                arg_tys.iter().zip(&sig.params).enumerate()
                            {
                                let arg_pos = idx.checked_add(1).unwrap_or(idx); // Reason for fallback: formatting-only 1-based position on usize::MAX
                                ensure!(
                                    types_compatible(aty, pty),
                                    "Call argument type mismatch at argument {arg_pos} of '{original_name}' at {span}: expected '{pty}', found '{aty}'"
                                );
                            }
                            self.resolved_calls.insert(
                                (span.line, span.col),
                                ResolvedCallTarget {
                                    namespace: Some(namespace),
                                    routine_name: original_name,
                                    sig: sig.clone(),
                                },
                            );
                            if was_in_tail {
                                self.tail_calls.insert((span.line, span.col));
                            }
                            return Ok(sig.return_type);
                        }
                        _ => bail!(
                            "Dc reference at {span} does not refer to a routine"
                        ),
                    }
                }

                bail!("Unresolved Dc routine call at {span}");
            }
        }
    }

    /// Validates a two-phase `do ... ::` lexical block, pre-declaring nested
    /// routines, tracking statement flow, and checking transitive nested read
    /// initialization invariants.
    #[allow(
        clippy::too_many_lines,
        reason = "Two-phase block scoping, nested capture verification, and cleanup"
    )]
    pub(super) fn check_block(
        &mut self,
        stmts: &[BlockItem],
        span: Span,
        state: &mut FlowState,
        ctx: &mut BodyCtx,
    ) -> Result<StageLType> {
        let exprs: Vec<&Expr> = stmts
            .iter()
            .filter_map(|item| match item {
                BlockItem::Expr(e) => Some(e),
                BlockItem::Comment(_) => None,
            })
            .collect();

        if exprs.is_empty() {
            return Ok(StageLType::Void);
        }

        let block_in_tail = ctx.in_tail;
        let saved_pending = std::mem::take(&mut self.pending_nested_calls);
        self.local_scopes.push(HashMap::new());
        self.routine_scopes.push(HashMap::new());

        let block_res: Result<StageLType> = (|| {
            let mut pre_routines: Vec<(usize, &RoutineDecl)> = Vec::new();
            let mut decl_ids_by_pos: HashMap<usize, usize> = HashMap::new();

            // Phase 1: Pre-declare all nested routines in this block.
            for (idx, e) in exprs.iter().enumerate() {
                let Expr::RoutineDecl(r) = *e else {
                    continue;
                };
                if self.forbid_stagel_internal
                    && is_stagel_internal_name(&r.name)
                {
                    bail!(
                        "StageL-internal nested routine '{}' cannot be declared in Stage 2/StageR module at {}",
                        r.name,
                        r.span
                    );
                }
                ensure!(
                    !r.is_pub && !r.pub_dc_syntax && r.dc_id.is_none(),
                    "Nested routine '{}' at {} cannot carry 'pub' or Dc annotations",
                    r.name,
                    r.span
                );
                ensure!(
                    !self.name_exists_in_local_or_top(&r.name),
                    "Nested routine '{}' at {} duplicates/shadows an existing binding, routine, or import",
                    r.name,
                    r.span
                );

                let ret_policy = match self.dialect {
                    StageDialect::StageL3 => BufPolicy::LocalAllowed,
                    StageDialect::StageR3 => BufPolicy::Disallowed,
                };
                let canon_ret = self.check_and_canonicalize_type(
                    &r.return_type,
                    ret_policy,
                    true,
                    true,
                    r.span,
                )?;
                let mut seen_params = HashSet::new();
                let mut param_tys = Vec::with_capacity(r.params.len());
                for p in &r.params {
                    if self.forbid_stagel_internal
                        && is_stagel_internal_name(&p.name)
                    {
                        bail!(
                            "StageL-internal parameter name '{}' in nested routine '{}' at {} is forbidden",
                            p.name,
                            r.name,
                            p.span
                        );
                    }
                    ensure!(
                        seen_params.insert(p.name.clone()),
                        "Duplicate parameter '{}' in nested routine '{}' at {}",
                        p.name,
                        r.name,
                        p.span
                    );
                    let canon_pty = self.check_and_canonicalize_type(
                        &p.ty,
                        ret_policy,
                        false,
                        false,
                        p.span,
                    )?;
                    if self.dialect == StageDialect::StageR3 {
                        check_stager_value_type_shape(
                            &canon_pty,
                            &self.type_defs(),
                            p.span,
                        )?;
                    }
                    param_tys.push(canon_pty);
                }

                let nid = self.alloc_binding_id()?;
                self.nested_summaries
                    .insert(nid, NestedVerbSummary::default());
                if let Some(scope) = self.routine_scopes.last_mut() {
                    scope.insert(
                        r.name.clone(),
                        VerbTarget {
                            id: nid,
                            sig: RoutineSig {
                                return_type: canon_ret,
                                params: param_tys,
                                is_pub: false,
                                dc_id: r.dc_id,
                            },
                        },
                    );
                }
                decl_ids_by_pos.insert(idx, nid);
                pre_routines.push((nid, r));
            }

            // Phase 2: Validate statements sequentially in lexical order.
            let mut last_ty = StageLType::Void;
            let mut terminated = false;
            for (idx, e) in exprs.iter().enumerate() {
                let is_last = idx.checked_add(1) == Some(exprs.len());
                if let Expr::RoutineDecl(r) = *e {
                    ctx.in_tail = false;
                    let nid = decl_ids_by_pos.get(&idx).copied().context(
                        "Missing pre-registered nested routine ID",
                    )?;
                    let sig = self
                        .routine_scopes
                        .last()
                        .and_then(|s| s.get(&r.name))
                        .map(|v| v.sig.clone())
                        .context("Missing pre-registered nested routine sig")?;
                    let outer_ids: HashSet<usize> = self
                        .local_scopes
                        .iter()
                        .flat_map(|s| s.values().map(|entry| entry.id))
                        .collect();
                    self.validate_nested_routine_at_decl(
                        r,
                        nid,
                        &sig.params,
                        &sig.return_type,
                        state,
                        &outer_ids,
                    )?;
                    self.expr_types
                        .insert((r.span.line, r.span.col), StageLType::Void);
                    if is_last {
                        last_ty = StageLType::Void;
                    }
                } else {
                    ctx.in_tail = block_in_tail && is_last;
                    let ety = self.check_expr(e, state, ctx)?;
                    if ety == StageLType::Never {
                        terminated = true;
                    }
                    if is_last {
                        last_ty = ety;
                    }
                }
            }

            // Propagate captured reads across nested-to-nested calls before
            // verifying every pending call site in this block.
            let summary_ids: Vec<usize> =
                self.nested_summaries.keys().copied().collect();
            loop {
                let mut changed = false;
                for &id in &summary_ids {
                    let Some(cur) = self.nested_summaries.get(&id).cloned()
                    else {
                        continue;
                    };
                    let mut extra_reads = Vec::new();
                    for &callee in &cur.called_nested {
                        if let Some(callee_sum) =
                            self.nested_summaries.get(&callee)
                        {
                            for &rid in &callee_sum.captured_reads {
                                if !cur.captured_reads.contains(&rid) {
                                    extra_reads.push(rid);
                                }
                            }
                        }
                    }
                    if !extra_reads.is_empty()
                        && let Some(slot) = self.nested_summaries.get_mut(&id)
                    {
                        slot.captured_reads.extend(extra_reads);
                        changed = true;
                    }
                }
                if !changed {
                    break;
                }
            }

            let pending = std::mem::take(&mut self.pending_nested_calls);
            let mut called_nids = HashSet::new();
            for (nid, call_span, assign_at_call) in pending {
                called_nids.insert(nid);
                if let Some(summary) = self.nested_summaries.get(&nid) {
                    for &req_id in &summary.captured_reads {
                        ensure!(
                            assign_at_call.is_assigned(req_id),
                            "Nested routine called at {call_span} reads enclosing variable that may be uninitialized at this call site"
                        );
                        if ctx.outer_ids.contains(&req_id)
                            && !ctx.always_written.contains(&req_id)
                        {
                            ctx.captured_reads.insert(req_id);
                        }
                    }
                }
            }

            for (nid, r) in pre_routines {
                if !called_nids.contains(&nid)
                    && let Some(sum) = self.nested_summaries.get(&nid)
                {
                    for &cap_id in &sum.captured_reads {
                        if Self::lookup_is_declared_in_scope(
                            self.local_scopes.last(),
                            cap_id,
                        ) && !state.assign.is_assigned(cap_id)
                        {
                            bail!(
                                "Nested routine '{}' at {} reads uninitialized 'let' variable",
                                r.name,
                                r.span
                            );
                        }
                    }
                }
            }

            Ok(if terminated {
                StageLType::Never
            } else {
                last_ty
            })
        })();

        ctx.in_tail = block_in_tail;
        self.pending_nested_calls = saved_pending;
        self.routine_scopes.pop();
        let popped_locals = self.local_scopes.pop();
        if self.dialect == StageDialect::StageR3
            && let Some(popped) = popped_locals
        {
            for entry in popped.into_values() {
                state.bufs.remove(&entry.id);
            }
        }

        let last_ty = block_res?;
        if self.dialect == StageDialect::StageR3 {
            ensure!(
                !last_ty.is_or_contains_buf(),
                "Linear 'buf/T' cannot be returned from a 'do' block at {span}"
            );
        }
        Ok(last_ty)
    }

    /// Validates an `if COND THEN ELSE` branch expression with 4-way `never`
    /// unification and path-sensitive flow intersection.
    pub(super) fn check_if(
        &mut self,
        cond: &Expr,
        then_branch: &Expr,
        else_branch: &Expr,
        span: Span,
        state: &mut FlowState,
        ctx: &mut BodyCtx,
    ) -> Result<StageLType> {
        let cond_ty = self.check_subexpr(cond, state, ctx)?;
        ensure!(
            cond_ty == StageLType::B || cond_ty == StageLType::Never,
            "'if' condition must have type 'b', found '{cond_ty}' at {span}"
        );

        let mut then_state = state.clone();
        let mut then_ctx = ctx.fork_child(
            ctx.in_tail,
            ctx.loop_depth,
            ctx.loop_protected_bufs.clone(),
        );
        let then_ty =
            self.check_expr(then_branch, &mut then_state, &mut then_ctx)?;

        let mut else_state = state.clone();
        let mut else_ctx = ctx.fork_child(
            ctx.in_tail,
            ctx.loop_depth,
            ctx.loop_protected_bufs.clone(),
        );
        let else_ty =
            self.check_expr(else_branch, &mut else_state, &mut else_ctx)?;

        ctx.captured_reads.extend(then_ctx.captured_reads);
        ctx.captured_reads.extend(else_ctx.captured_reads);
        ctx.called_nested.extend(then_ctx.called_nested);
        ctx.called_nested.extend(else_ctx.called_nested);

        if self.dialect == StageDialect::StageR3 {
            ensure!(
                !then_ty.is_or_contains_buf() && !else_ty.is_or_contains_buf(),
                "Linear 'buf/T' cannot be returned from an 'if' expression at {span}"
            );
        }

        match (then_ty == StageLType::Never, else_ty == StageLType::Never) {
            (true, true) => {
                then_state.intersect_with(&else_state);
                *state = then_state;
                Ok(StageLType::Never)
            }
            (true, false) => {
                *state = else_state;
                ctx.always_written.extend(else_ctx.always_written);
                Ok(else_ty)
            }
            (false, true) => {
                *state = then_state;
                ctx.always_written.extend(then_ctx.always_written);
                Ok(then_ty)
            }
            (false, false) => {
                ensure!(
                    then_ty == else_ty,
                    "'if' branches have mismatched types '{then_ty}' and '{else_ty}' at {span}"
                );
                then_state.intersect_with(&else_state);
                *state = then_state;
                for id in &then_ctx.always_written {
                    if else_ctx.always_written.contains(id) {
                        ctx.always_written.insert(*id);
                    }
                }
                Ok(then_ty)
            }
        }
    }

    /// Validates a pre-test `while COND BODY` loop.
    pub(super) fn check_while(
        &mut self,
        cond: &Expr,
        body: &Expr,
        span: Span,
        state: &mut FlowState,
        ctx: &mut BodyCtx,
    ) -> Result<StageLType> {
        let cond_ty = self.check_subexpr(cond, state, ctx)?;
        ensure!(
            cond_ty == StageLType::B || cond_ty == StageLType::Never,
            "'while' condition must have type 'b', found '{cond_ty}' at {span}"
        );

        let mut body_state = state.clone();
        let next_depth = ctx
            .loop_depth
            .checked_add(1)
            .context("Loop nesting depth overflow")?;
        let mut protected = ctx.loop_protected_bufs.clone();
        if self.dialect == StageDialect::StageR3 {
            protected.extend(state.bufs.keys().copied());
        }
        let mut body_ctx = ctx.fork_child(false, next_depth, protected);
        let body_ty = self.check_expr(body, &mut body_state, &mut body_ctx)?;
        ctx.captured_reads.extend(body_ctx.captured_reads);
        ctx.called_nested.extend(body_ctx.called_nested);

        ensure!(
            body_ty == StageLType::Void || body_ty == StageLType::Never,
            "'while' body must have type 'void', found '{body_ty}' at {span}"
        );
        Ok(StageLType::Void)
    }

    /// Validates an immutable `foreach T/elem LIST BODY` loop over `l/T` (or
    /// `dcs -> dc` in StageR 3).
    pub(super) fn check_foreach(
        &mut self,
        elem_ty: &StageLType,
        elem_name: &str,
        list: &Expr,
        body: &Expr,
        span: Span,
        state: &mut FlowState,
        ctx: &mut BodyCtx,
    ) -> Result<StageLType> {
        if self.forbid_stagel_internal && is_stagel_internal_name(elem_name) {
            bail!(
                "StageL-internal loop variable '{elem_name}' cannot be declared in Stage 2/StageR module at {span}"
            );
        }
        ensure!(
            !self.name_exists_in_local_or_top(elem_name),
            "Loop variable '{elem_name}' at {span} duplicates/shadows an existing binding or symbol"
        );

        let canon_elem = self.check_and_canonicalize_type(
            elem_ty,
            BufPolicy::Disallowed,
            false,
            false,
            span,
        )?;
        if self.dialect == StageDialect::StageR3 {
            check_stager_value_type_shape(
                &canon_elem,
                &self.type_defs(),
                span,
            )?;
        }

        let list_ty = self.check_subexpr(list, state, ctx)?;
        if list_ty != StageLType::Never {
            match self.dialect {
                StageDialect::StageL3 => {
                    ensure!(
                        list_ty == StageLType::List(Box::new(canon_elem.clone())),
                        "'foreach' requires sequence 'l/{canon_elem}', found '{list_ty}' at {span}"
                    );
                }
                StageDialect::StageR3 => {
                    let valid = list_ty
                        == StageLType::List(Box::new(canon_elem.clone()))
                        || (canon_elem == StageLType::Dc
                            && list_ty == StageLType::Dcs);
                    ensure!(
                        valid,
                        "'foreach' requires 'l/{canon_elem}' (or 'dcs' for 'dc/{elem_name}'), found '{list_ty}' at {span}"
                    );
                }
            }
        }

        let elem_id = self.alloc_binding_id()?;
        let mut loop_scope = HashMap::new();
        loop_scope.insert(
            elem_name.to_owned(),
            BindingEntry {
                id: elem_id,
                kind: BindingKind::Const,
                ty: canon_elem,
                decl_span: span,
            },
        );

        let mut body_state = state.clone();
        body_state.assign.mark_assigned(elem_id);
        let next_depth = ctx
            .loop_depth
            .checked_add(1)
            .context("Loop nesting depth overflow")?;
        let mut protected = ctx.loop_protected_bufs.clone();
        if self.dialect == StageDialect::StageR3 {
            protected.extend(state.bufs.keys().copied());
        }
        let mut body_ctx = ctx.fork_child(false, next_depth, protected);

        self.local_scopes.push(loop_scope);
        let body_res = self.check_expr(body, &mut body_state, &mut body_ctx);
        self.local_scopes.pop();
        ctx.captured_reads.extend(body_ctx.captured_reads);
        ctx.called_nested.extend(body_ctx.called_nested);
        let body_ty = body_res?;

        ensure!(
            body_ty == StageLType::Void || body_ty == StageLType::Never,
            "'foreach' body must have type 'void', found '{body_ty}' at {span}"
        );
        Ok(StageLType::Void)
    }

    fn resolve_match_domain(
        &self,
        scrut_ty: &StageLType,
        span: Span,
    ) -> Result<MatchDomain> {
        match scrut_ty {
            StageLType::Option(inner) => {
                Ok(MatchDomain::Option((**inner).clone()))
            }
            StageLType::Result(inner) => {
                Ok(MatchDomain::Result((**inner).clone()))
            }
            StageLType::Error => Ok(MatchDomain::Error),
            StageLType::Nominal(StructTypeRef::Canonical {
                namespace,
                name,
            }) => {
                let key = (namespace.clone(), name.clone());
                if let Some(def) = self.enums.get(&key) {
                    Ok(MatchDomain::Enum { def: def.clone() })
                } else {
                    bail!(
                        "'match' cannot destructure nominal/brand type '{name}' at {span} (only 'enum', 'option/T', 'result/T', and 'error' are matchable)"
                    );
                }
            }
            _ => bail!(
                "'match' requires an 'enum', 'option/T', 'result/T', or 'error' scrutinee, found '{scrut_ty}' at {span}"
            ),
        }
    }

    fn expected_match_variants(domain: &MatchDomain) -> Vec<&str> {
        match domain {
            MatchDomain::Option(_) => vec!["none", "some"],
            MatchDomain::Result(_) => vec!["ok", "err"],
            MatchDomain::Error => vec![
                "overflow",
                "divisionByZero",
                "outOfBounds",
                "frozenBuffer",
                "invalidEncoding",
                "invalidDc",
                "parseFailure",
                "unexpectedNone",
            ],
            MatchDomain::Enum { def } => {
                def.variants.iter().map(|v| v.name.as_str()).collect()
            }
        }
    }

    fn arm_expected_payload(
        domain: &MatchDomain,
        arm: &MatchArm,
    ) -> Result<Option<StageLType>> {
        match domain {
            MatchDomain::Option(inner) => match arm.variant.as_str() {
                "none" => Ok(None),
                "some" => Ok(Some(inner.clone())),
                other => bail!(
                    "Unknown 'option' variant 'case {other}' in 'match' at {}",
                    arm.span
                ),
            },
            MatchDomain::Result(inner) => match arm.variant.as_str() {
                "ok" => {
                    if *inner == StageLType::Void {
                        Ok(None)
                    } else {
                        Ok(Some(inner.clone()))
                    }
                }
                "err" => Ok(Some(StageLType::Error)),
                other => bail!(
                    "Unknown 'result' variant 'case {other}' in 'match' at {}",
                    arm.span
                ),
            },
            MatchDomain::Error => {
                ensure!(
                    StageRErrorTag::from_keyword(&arm.variant).is_some(),
                    "Unknown 'error' tag 'case {}' in 'match' at {}",
                    arm.variant,
                    arm.span
                );
                Ok(None)
            }
            MatchDomain::Enum { def } => {
                let variant_def = def
                    .variants
                    .iter()
                    .find(|v| v.name == arm.variant)
                    .with_context(|| {
                        format!(
                            "Unknown enum variant 'case {}' for enum '{}' at {}",
                            arm.variant, def.original_name, arm.span
                        )
                    })?;
                Ok(variant_def.payload.clone())
            }
        }
    }

    /// Validates a StageR 3 shallow exhaustive `match` expression.
    #[allow(
        clippy::too_many_lines,
        reason = "Exhaustiveness, variant payload checking, and multi-arm flow join"
    )]
    pub(super) fn check_match(
        &mut self,
        scrutinee: &Expr,
        arms: &[MatchArm],
        fallback: Option<&Expr>,
        span: Span,
        state: &mut FlowState,
        ctx: &mut BodyCtx,
    ) -> Result<StageLType> {
        ensure!(
            self.dialect == StageDialect::StageR3,
            "StageR 3 'match' expression is not supported in StageL 3 at {span}"
        );
        let scrut_ty = self.check_subexpr(scrutinee, state, ctx)?;
        let domain = self.resolve_match_domain(&scrut_ty, span)?;

        let mut seen: HashSet<String> = HashSet::new();
        let mut branch_results: Vec<(FlowState, StageLType, BodyCtx)> =
            Vec::with_capacity(arms.len());

        for arm in arms {
            ensure!(
                seen.insert(arm.variant.clone()),
                "Duplicate 'case {}' arm in 'match' at {}",
                arm.variant,
                arm.span
            );
            let expected_payload = Self::arm_expected_payload(&domain, arm)?;
            let mut arm_state = state.clone();
            let mut arm_ctx = ctx.fork_child(
                ctx.in_tail,
                ctx.loop_depth,
                ctx.loop_protected_bufs.clone(),
            );
            let mut pushed_scope = false;

            match (expected_payload, &arm.binding) {
                (Some(exp_pty), Some((ann_ty, bname, bspan))) => {
                    if self.forbid_stagel_internal
                        && is_stagel_internal_name(bname)
                    {
                        bail!(
                            "StageL-internal match binding '{bname}' cannot be declared in Stage 2/StageR module at {bspan}"
                        );
                    }
                    ensure!(
                        !self.name_exists_in_local_or_top(bname),
                        "Match binding '{bname}' at {bspan} duplicates/shadows an existing binding or symbol"
                    );
                    let canon_ann = self.check_and_canonicalize_type(
                        ann_ty,
                        BufPolicy::Disallowed,
                        false,
                        false,
                        *bspan,
                    )?;
                    check_stager_value_type_shape(
                        &canon_ann,
                        &self.type_defs(),
                        *bspan,
                    )?;
                    ensure!(
                        canon_ann == exp_pty,
                        "Match arm 'case {}' binding '{bname}' at {bspan} has type '{canon_ann}', expected '{exp_pty}'",
                        arm.variant
                    );
                    let id = self.alloc_binding_id()?;
                    arm_state.assign.mark_assigned(id);
                    let mut arm_scope = HashMap::new();
                    arm_scope.insert(
                        bname.clone(),
                        BindingEntry {
                            id,
                            kind: BindingKind::Const,
                            ty: canon_ann,
                            decl_span: *bspan,
                        },
                    );
                    self.local_scopes.push(arm_scope);
                    pushed_scope = true;
                }
                (Some(exp_pty), None) => {
                    bail!(
                        "Variant 'case {}' at {} carries payload of type '{exp_pty}' and requires a binding",
                        arm.variant,
                        arm.span
                    );
                }
                (None, Some((_, _, bspan))) => {
                    bail!(
                        "Unit variant 'case {}' at {bspan} does not carry a payload binding",
                        arm.variant
                    );
                }
                (None, None) => {}
            }

            let arm_res = self.check_expr(&arm.body, &mut arm_state, &mut arm_ctx);
            if pushed_scope {
                self.local_scopes.pop();
            }
            ctx.captured_reads.extend(arm_ctx.captured_reads.iter().copied());
            ctx.called_nested.extend(arm_ctx.called_nested.iter().copied());
            let arm_ty = arm_res?;
            ensure!(
                !arm_ty.is_or_contains_buf(),
                "Linear 'buf/T' cannot be returned from a 'match' arm at {}",
                arm.span
            );
            branch_results.push((arm_state, arm_ty, arm_ctx));
        }

        let expected = Self::expected_match_variants(&domain);
        let missing: Vec<&str> = expected
            .iter()
            .copied()
            .filter(|v| !seen.contains(*v))
            .collect();

        if let Some(fb_expr) = fallback {
            ensure!(
                !missing.is_empty(),
                "Redundant 'else' branch in exhaustive 'match' at {span}"
            );
            let mut fb_state = state.clone();
            let mut fb_ctx = ctx.fork_child(
                ctx.in_tail,
                ctx.loop_depth,
                ctx.loop_protected_bufs.clone(),
            );
            let fb_ty = self.check_expr(fb_expr, &mut fb_state, &mut fb_ctx)?;
            ctx.captured_reads.extend(fb_ctx.captured_reads.iter().copied());
            ctx.called_nested.extend(fb_ctx.called_nested.iter().copied());
            ensure!(
                !fb_ty.is_or_contains_buf(),
                "Linear 'buf/T' cannot be returned from a 'match' else branch at {span}"
            );
            branch_results.push((fb_state, fb_ty, fb_ctx));
        } else {
            ensure!(
                missing.is_empty(),
                "Non-exhaustive 'match' at {span}: missing case(s): {}",
                missing.join(", ")
            );
        }

        if branch_results.is_empty() {
            return Ok(StageLType::Never);
        }

        let mut unified_ty: Option<StageLType> = None;
        let mut joined_state: Option<FlowState> = None;
        let mut joined_writes: Option<HashSet<usize>> = None;
        for (bstate, bty, bctx) in branch_results {
            if bty == StageLType::Never {
                continue;
            }
            match &unified_ty {
                None => unified_ty = Some(bty),
                Some(prev) => {
                    ensure!(
                        *prev == bty,
                        "'match' arms have mismatched types '{prev}' and '{bty}' at {span}"
                    );
                }
            }
            match &mut joined_state {
                None => joined_state = Some(bstate),
                Some(acc) => acc.intersect_with(&bstate),
            }
            match &mut joined_writes {
                None => joined_writes = Some(bctx.always_written),
                Some(acc) => acc.retain(|id| bctx.always_written.contains(id)),
            }
        }

        if let Some(js) = joined_state {
            *state = js;
        }
        if let Some(jw) = joined_writes {
            ctx.always_written.extend(jw);
        }
        Ok(unified_ty.unwrap_or(StageLType::Never)) // Reason for fallback: when all match arms evaluate to `never`, unified result type is `never`
    }

    /// Validates StageR 3 trailing propagation `/` (`Expr::Propagate`).
    pub(super) fn check_propagate(
        &mut self,
        expr: &Expr,
        span: Span,
        state: &mut FlowState,
        ctx: &mut BodyCtx,
    ) -> Result<StageLType> {
        ensure!(
            self.dialect == StageDialect::StageR3,
            "StageR 3 propagation '/' is not supported in StageL 3 at {span}"
        );
        ensure!(
            ctx.allow_propagate,
            "Propagation operator '/' at {span} is only allowed inside a routine returning 'option/T' or 'result/T'"
        );
        let inner_ty = self.check_subexpr(expr, state, ctx)?;
        match (&inner_ty, &ctx.return_type) {
            (StageLType::Never, _) => Ok(StageLType::Never),
            (StageLType::Option(ok), StageLType::Option(_))
            | (StageLType::Option(ok), StageLType::Result(_))
            | (StageLType::Result(ok), StageLType::Result(_)) => {
                Ok((**ok).clone())
            }
            (StageLType::Result(_), StageLType::Option(_)) => bail!(
                "Cannot propagate 'result/T' error via '/' inside an 'option/R' routine at {span}"
            ),
            _ => bail!(
                "Propagation operator '/' at {span} requires 'option/T' or 'result/T' operand compatible with routine return type '{}', found '{inner_ty}'",
                ctx.return_type
            ),
        }
    }

    /// Validates `ret` / `return [EXPR]`.
    pub(super) fn check_return(
        &mut self,
        value: Option<&Expr>,
        span: Span,
        state: &mut FlowState,
        ctx: &mut BodyCtx,
    ) -> Result<StageLType> {
        let Some(val_expr) = value else {
            ensure!(
                ctx.return_type == StageLType::Void
                    || ctx.return_type == StageLType::Never,
                "Bare 'return' at {span} is only allowed in 'void' routines (expected '{}')",
                ctx.return_type
            );
            return Ok(StageLType::Never);
        };

        let prev_tail = ctx.in_tail;
        ctx.in_tail = true;
        let val_res = self.check_expr(val_expr, state, ctx);
        ctx.in_tail = prev_tail;
        let val_ty = val_res?;

        if self.dialect == StageDialect::StageR3 {
            ensure!(
                !val_ty.is_or_contains_buf(),
                "Linear 'buf/T' cannot be returned at {span}"
            );
        }
        ensure!(
            types_compatible(&val_ty, &ctx.return_type),
            "'return' expression at {span} evaluates to '{val_ty}', expected routine return type '{}'",
            ctx.return_type
        );
        Ok(StageLType::Never)
    }
}
