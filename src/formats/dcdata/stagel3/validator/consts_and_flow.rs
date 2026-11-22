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

//! Flow-state tracking (definite assignment and linear buffer state),
//! module-level constant purity and acyclicity checks, inline test block
//! validation, and `ModuleValidator` type/scope lookup helpers.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::super::ast::{
    BlockItem, Expr, RoutineDecl, Span, StageDialect, StageLType, SymbolTarget,
    TestExpected, TopValueDecl, TypedIntLit,
};
use super::imports_and_decls::ModuleValidator;
use super::types::{
    BufPolicy, TypeDefs, TypeEnv, canonicalize_type, check_type_well_formed,
    type_supports_eq, types_compatible,
};
use super::RoutineSig;
use crate::DcChar;
use std::collections::{BTreeMap, HashMap, HashSet};

/// Definitely-assigned variable bitset tracked across structured control flow.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct AssignState {
    initialized: HashSet<usize>,
}

impl AssignState {
    /// Creates an empty definite-assignment state.
    pub(super) fn new() -> Self {
        Self {
            initialized: HashSet::new(),
        }
    }

    /// Records binding `id` as definitely assigned on the current path.
    pub(super) fn mark_assigned(&mut self, id: usize) {
        self.initialized.insert(id);
    }

    /// Returns `true` if `id` is definitely assigned on all paths reaching
    /// here.
    pub(super) fn is_assigned(&self, id: usize) -> bool {
        self.initialized.contains(&id)
    }

    /// Intersects `self` with `other` at a structured control-flow join.
    pub(super) fn intersect_with(&mut self, other: &Self) {
        self.initialized.retain(|id| other.initialized.contains(id));
    }
}

/// Linear buffer lifecycle state in StageR 3.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BufState {
    /// Buffer has not been frozen yet and accepts buffer verbs.
    Unfrozen,
    /// Buffer was consumed by `bufFreeze` and can never be accessed again.
    Frozen,
}

/// Combined definite-assignment and StageR 3 linear buffer flow state.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct FlowState {
    pub(super) assign: AssignState,
    pub(super) bufs: BTreeMap<usize, BufState>,
}

impl FlowState {
    /// Creates an empty initial [`FlowState`].
    pub(super) fn new() -> Self {
        Self {
            assign: AssignState::new(),
            bufs: BTreeMap::new(),
        }
    }

    /// Joins two converging execution paths conservatively: assignment takes
    /// intersection and any buffer frozen on either path becomes `Frozen`.
    pub(super) fn intersect_with(&mut self, other: &Self) {
        self.assign.intersect_with(&other.assign);
        self.bufs.retain(|id, state| {
            let Some(other_state) = other.bufs.get(id) else {
                return false;
            };
            if *state == BufState::Frozen || *other_state == BufState::Frozen {
                *state = BufState::Frozen;
            }
            true
        });
    }
}

/// Lexical binding kind within a routine scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BindingKind {
    /// Routine formal parameter (`Param`).
    Param,
    /// Mutable local variable (`let`).
    Let,
    /// Immutable local constant (`const`).
    Const,
}

/// Single entry inside a routine's active lexical scope stack.
#[derive(Debug, Clone)]
pub(super) struct BindingEntry {
    pub(super) id: usize,
    pub(super) kind: BindingKind,
    pub(super) ty: StageLType,
    pub(super) decl_span: Span,
}

/// Nested routine signature and unique summary ID registered in a `do` block.
#[derive(Debug, Clone)]
pub(super) struct VerbTarget {
    pub(super) id: usize,
    pub(super) sig: RoutineSig,
}

/// Summary of enclosing bindings read/written and sibling nested routines
/// invoked by a nested routine.
#[derive(Debug, Clone, Default)]
pub(super) struct NestedVerbSummary {
    pub(super) captured_reads: HashSet<usize>,
    pub(super) always_written: HashSet<usize>,
    pub(super) called_nested: HashSet<usize>,
    pub(super) captures_resolved: bool,
}

/// Active checking context threaded through routine body statements and
/// expressions.
#[derive(Debug, Clone)]
pub(super) struct BodyCtx {
    pub(super) return_type: StageLType,
    pub(super) loop_depth: usize,
    pub(super) in_tail: bool,
    pub(super) allow_buf: bool,
    pub(super) allow_propagate: bool,
    pub(super) outer_ids: HashSet<usize>,
    pub(super) loop_protected_bufs: HashSet<usize>,
    pub(super) captured_reads: HashSet<usize>,
    pub(super) always_written: HashSet<usize>,
    pub(super) called_nested: HashSet<usize>,
    pub(super) self_top_routine: Option<(String, RoutineSig)>,
}

impl BodyCtx {
    /// Constructs a top-level routine/initializer checking context.
    pub(super) fn new(return_type: StageLType, allow_propagate: bool) -> Self {
        Self {
            return_type,
            loop_depth: 0,
            in_tail: true,
            allow_buf: false,
            allow_propagate,
            outer_ids: HashSet::new(),
            loop_protected_bufs: HashSet::new(),
            captured_reads: HashSet::new(),
            always_written: HashSet::new(),
            called_nested: HashSet::new(),
            self_top_routine: None,
        }
    }

    /// Constructs a checking context for a nested routine inside a `do` block.
    pub(super) fn for_nested(
        return_type: StageLType,
        allow_propagate: bool,
        outer_ids: HashSet<usize>,
    ) -> Self {
        Self {
            return_type,
            loop_depth: 0,
            in_tail: true,
            allow_buf: false,
            allow_propagate,
            outer_ids,
            loop_protected_bufs: HashSet::new(),
            captured_reads: HashSet::new(),
            always_written: HashSet::new(),
            called_nested: HashSet::new(),
            self_top_routine: None,
        }
    }

    /// Forks a child branch/loop context preserving current `always_written`
    /// while isolating new reads, writes, and nested calls for branch join.
    pub(super) fn fork_child(
        &self,
        in_tail: bool,
        loop_depth: usize,
        loop_protected_bufs: HashSet<usize>,
    ) -> Self {
        Self {
            return_type: self.return_type.clone(),
            loop_depth,
            in_tail,
            allow_buf: self.allow_buf,
            allow_propagate: self.allow_propagate,
            outer_ids: self.outer_ids.clone(),
            loop_protected_bufs,
            captured_reads: HashSet::new(),
            always_written: self.always_written.clone(),
            called_nested: HashSet::new(),
            self_top_routine: self.self_top_routine.clone(),
        }
    }
}

/// Collects value identifier and Dc dependencies referenced by `expr` for
/// top-level constant cycle analysis.
pub(super) fn collect_value_deps(
    expr: &Expr,
    deps: &mut Vec<(String, Span)>,
    dc_deps: &mut Vec<(DcChar, Span)>,
) {
    match expr {
        Expr::Bool { .. }
        | Expr::Int { .. }
        | Expr::Bytes { .. }
        | Expr::DctsString { .. }
        | Expr::DcLit { .. }
        | Expr::ErrorTag { .. }
        | Expr::OptionNone { .. }
        | Expr::Break { .. }
        | Expr::Continue { .. }
        | Expr::Fault { .. }
        | Expr::Nop { .. } => {}
        Expr::ValueRef { target, span, .. } => match target {
            SymbolTarget::Ident(name) => deps.push((name.clone(), *span)),
            SymbolTarget::Dc(dc) => dc_deps.push((*dc, *span)),
        },
        Expr::List { elements, .. } | Expr::TupleNew { elements, .. } => {
            for el in elements {
                collect_value_deps(el, deps, dc_deps);
            }
        }
        Expr::StructNew { fields, .. } => {
            for f in fields {
                collect_value_deps(f, deps, dc_deps);
            }
        }
        Expr::NamedStructNew { fields, .. } => {
            for (_, fexpr, _) in fields {
                collect_value_deps(fexpr, deps, dc_deps);
            }
        }
        Expr::EnumVariant { payload, .. } => {
            if let Some(p) = payload {
                collect_value_deps(p, deps, dc_deps);
            }
        }
        Expr::Field { base, .. } | Expr::NamedField { base, .. } => {
            collect_value_deps(base, deps, dc_deps);
        }
        Expr::With { base, value, .. }
        | Expr::NamedWith { base, value, .. } => {
            collect_value_deps(base, deps, dc_deps);
            collect_value_deps(value, deps, dc_deps);
        }
        Expr::BufNew { capacity, .. } => {
            collect_value_deps(capacity, deps, dc_deps);
        }
        Expr::Let { init, .. } => {
            if let Some(i) = init {
                collect_value_deps(i, deps, dc_deps);
            }
        }
        Expr::Const { value, .. } | Expr::Set { value, .. } => {
            collect_value_deps(value, deps, dc_deps);
        }
        Expr::RoutineDecl(r) => {
            collect_value_deps(&r.body, deps, dc_deps);
        }
        Expr::AnonRoutine(r) => {
            collect_value_deps(&r.body, deps, dc_deps);
        }
        Expr::InlineCall { routine, args, .. } => {
            collect_value_deps(&routine.body, deps, dc_deps);
            for a in args {
                collect_value_deps(a, deps, dc_deps);
            }
        }
        Expr::Call { args, .. } => {
            for a in args {
                collect_value_deps(a, deps, dc_deps);
            }
        }
        Expr::Block { stmts, .. } => {
            for item in stmts {
                if let BlockItem::Expr(sub) = item {
                    collect_value_deps(sub, deps, dc_deps);
                }
            }
        }
        Expr::If {
            cond,
            then_branch,
            else_branch,
            ..
        } => {
            collect_value_deps(cond, deps, dc_deps);
            collect_value_deps(then_branch, deps, dc_deps);
            collect_value_deps(else_branch, deps, dc_deps);
        }
        Expr::While { cond, body, .. } => {
            collect_value_deps(cond, deps, dc_deps);
            collect_value_deps(body, deps, dc_deps);
        }
        Expr::ForEach { list, body, .. } => {
            collect_value_deps(list, deps, dc_deps);
            collect_value_deps(body, deps, dc_deps);
        }
        Expr::Match {
            scrutinee,
            arms,
            fallback,
            ..
        } => {
            collect_value_deps(scrutinee, deps, dc_deps);
            for arm in arms {
                collect_value_deps(&arm.body, deps, dc_deps);
            }
            if let Some(fb) = fallback {
                collect_value_deps(fb, deps, dc_deps);
            }
        }
        Expr::Return { value, .. } => {
            if let Some(v) = value {
                collect_value_deps(v, deps, dc_deps);
            }
        }
        Expr::OptionSome { value, .. } | Expr::ResultOk { value, .. } => {
            collect_value_deps(value, deps, dc_deps);
        }
        Expr::ResultErr { error, .. } => {
            collect_value_deps(error, deps, dc_deps);
        }
        Expr::Unary { operand, .. } => {
            collect_value_deps(operand, deps, dc_deps);
        }
        Expr::Binary { left, right, .. } => {
            collect_value_deps(left, deps, dc_deps);
            collect_value_deps(right, deps, dc_deps);
        }
        Expr::Ternary {
            first,
            second,
            third,
            ..
        } => {
            collect_value_deps(first, deps, dc_deps);
            collect_value_deps(second, deps, dc_deps);
            collect_value_deps(third, deps, dc_deps);
        }
        Expr::Conv { expr, .. }
        | Expr::Bitcast { expr, .. }
        | Expr::WrapNominal { expr, .. }
        | Expr::UnwrapNominal { expr, .. }
        | Expr::Propagate { expr, .. } => {
            collect_value_deps(expr, deps, dc_deps);
        }
    }
}

impl<'a> ModuleValidator<'a> {
    /// Borrows all canonical type definition maps for shared type checks.
    pub(super) fn type_defs(&self) -> TypeDefs<'_> {
        TypeDefs {
            struct_defs: &self.struct_defs,
            brand_types: &self.brand_types,
            named_structs: &self.named_structs,
            enums: &self.enums,
        }
    }

    /// Borrows full symbol tables for syntactic type canonicalization.
    pub(super) fn type_env(&self) -> TypeEnv<'_> {
        TypeEnv {
            dialect: self.dialect,
            forbid_stagel_internal: self.forbid_stagel_internal,
            local_structs: &self.local_structs,
            dc_structs: &self.dc_structs,
            local_nominals: &self.local_nominals,
            dc_nominals: &self.dc_nominals,
            defs: self.type_defs(),
            top_dc_names: &self.top_dc_names,
        }
    }

    /// Allocates a fresh lexical binding or nested-routine summary ID.
    pub(super) fn alloc_binding_id(&mut self) -> Result<usize> {
        let id = self.next_binding_id;
        self.next_binding_id = id
            .checked_add(1)
            .context("Internal validator binding ID space exhausted")?;
        Ok(id)
    }

    /// Resolves `name` across active lexical variable/param/const scopes.
    pub(super) fn lookup_local(&self, name: &str) -> Option<&BindingEntry> {
        self.local_scopes.iter().rev().find_map(|s| s.get(name))
    }

    /// Resolves `name` across enclosing `do`-block nested routine scopes.
    pub(super) fn lookup_nested_routine(&self, name: &str) -> Option<&VerbTarget> {
        self.routine_scopes.iter().rev().find_map(|s| s.get(name))
    }

    /// Returns `true` if `id` belongs to a binding declared in `scope`.
    pub(super) fn lookup_is_declared_in_scope(
        scope: Option<&HashMap<String, BindingEntry>>,
        id: usize,
    ) -> bool {
        scope.is_some_and(|map| map.values().any(|e| e.id == id))
    }

    /// Returns `true` if `name` is already bound anywhere lexically or at
    /// module scope (enforcing strict non-shadowing).
    pub(super) fn name_exists_in_local_or_top(&self, name: &str) -> bool {
        self.local_scopes.iter().any(|s| s.contains_key(name))
            || self.routine_scopes.iter().any(|s| s.contains_key(name))
            || self.value_candidates.contains_key(name)
            || self.routine_candidates.contains_key(name)
            || self.local_structs.contains_key(name)
            || self.local_nominals.contains_key(name)
    }

    /// Canonicalizes syntactic `ty` under the active module's dialect and
    /// symbol environment.
    pub(super) fn canonicalize_type(
        &self,
        ty: &StageLType,
        span: Span,
    ) -> Result<StageLType> {
        canonicalize_type(ty, &self.type_env(), span)
    }

    /// Canonicalizes `ty`, validates `void`/`never` allowance, and checks
    /// dialect & buffer placement rules.
    pub(super) fn check_and_canonicalize_type(
        &self,
        ty: &StageLType,
        buf_policy: BufPolicy,
        allow_void: bool,
        allow_never: bool,
        span: Span,
    ) -> Result<StageLType> {
        let canon = self.canonicalize_type(ty, span)?;
        if !allow_void && canon == StageLType::Void {
            bail!("'void' type is not allowed in this position at {span}");
        }
        if !allow_never && canon == StageLType::Never {
            bail!("'never' type is not allowed in this position at {span}");
        }
        check_type_well_formed(
            &canon,
            &self.type_defs(),
            self.dialect,
            buf_policy,
            span,
        )?;
        Ok(canon)
    }

    /// Returns `true` if `ty` supports structural `eq` / `ne` in `self.dialect`.
    pub(super) fn type_supports_eq(&self, ty: &StageLType) -> bool {
        type_supports_eq(ty, &self.type_defs(), self.dialect)
    }

    /// Validates all top-level constant initializers for acyclicity, constant
    /// expression purity, and type compatibility.
    pub(super) fn validate_top_values_acyclic(
        &mut self,
        top_values: &[&'a TopValueDecl],
    ) -> Result<()> {
        let mut by_name: HashMap<&str, &'a TopValueDecl> = HashMap::new();
        let mut top_dc_to_name: HashMap<DcChar, &str> = HashMap::new();
        for v in top_values {
            by_name.insert(v.name.as_str(), *v);
            if let Some(dc) = v.dc_id {
                top_dc_to_name.insert(dc, v.name.as_str());
            }
        }

        let mut deps_map: HashMap<&str, Vec<&str>> = HashMap::new();
        for v in top_values {
            let mut raw_deps = Vec::new();
            let mut dc_deps = Vec::new();
            collect_value_deps(&v.value, &mut raw_deps, &mut dc_deps);
            let mut edges = Vec::new();
            for (dname, _) in &raw_deps {
                if let Some(target) = by_name.get(dname.as_str()) {
                    edges.push(target.name.as_str());
                }
            }
            for (ddc, _) in &dc_deps {
                if let Some(target_name) = top_dc_to_name.get(ddc) {
                    edges.push(*target_name);
                }
            }
            deps_map.insert(v.name.as_str(), edges);
        }

        // 0 = unvisited, 1 = visiting, 2 = visited
        let mut color: HashMap<&str, u8> = HashMap::new();
        fn dfs<'b>(
            node: &'b str,
            deps_map: &HashMap<&'b str, Vec<&'b str>>,
            color: &mut HashMap<&'b str, u8>,
        ) -> Result<()> {
            match color.get(node).copied() {
                Some(2) => return Ok(()),
                Some(1) => {
                    bail!(
                        "Cyclic constant initializer dependency involving '{node}'"
                    );
                }
                _ => {}
            }
            color.insert(node, 1);
            if let Some(neighbors) = deps_map.get(node) {
                for next in neighbors {
                    dfs(next, deps_map, color)?;
                }
            }
            color.insert(node, 2);
            Ok(())
        }

        for v in top_values {
            dfs(v.name.as_str(), &deps_map, &mut color)?;
        }

        for v in top_values {
            self.check_const_purity(&v.value)?;
            let canon_ty = match self.value_candidates.get(&v.name) {
                Some(target) => target.ty.clone(),
                None => self.canonicalize_type(&v.ty, v.span)?,
            };
            let mut state = FlowState::new();
            let mut ctx = BodyCtx::new(canon_ty.clone(), false);
            let calls_before = self.resolved_calls.len();
            let actual_ty =
                self.check_expr(&v.value, &mut state, &mut ctx)?;
            ensure!(
                self.resolved_calls.len() == calls_before,
                "Routine/overloaded operator calls are not allowed in constant expression for '{}' at {}",
                v.name,
                v.span
            );
            ensure!(
                types_compatible(&actual_ty, &canon_ty),
                "Constant '{}' at {} has declared type '{}' but initializer has type '{}'",
                v.name,
                v.span,
                canon_ty,
                actual_ty
            );
        }

        Ok(())
    }

    /// Verifies that `expr` uses only side-effect-free constant operations valid
    /// in `self.dialect`.
    #[allow(
        clippy::too_many_lines,
        reason = "Exhaustive purity match across all AST Expr nodes"
    )]
    pub(super) fn check_const_purity(&self, expr: &Expr) -> Result<()> {
        match expr {
            Expr::Bool { .. }
            | Expr::Bytes { .. }
            | Expr::ValueRef { .. }
            | Expr::Nop { .. } => Ok(()),
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
                        "StageR 3 integer literal '{value:?}' is not valid in StageL 3 at {span}"
                    );
                }
                Ok(())
            }
            Expr::List { elements, .. } | Expr::TupleNew { elements, .. } => {
                for el in elements {
                    self.check_const_purity(el)?;
                }
                Ok(())
            }
            Expr::StructNew { fields, span, .. } => {
                ensure!(
                    self.dialect == StageDialect::StageL3,
                    "Positional 'structNew' at {span} is forbidden in StageR 3"
                );
                for f in fields {
                    self.check_const_purity(f)?;
                }
                Ok(())
            }
            Expr::Field { base, .. } => self.check_const_purity(base),
            Expr::With { base, value, .. } => {
                self.check_const_purity(base)?;
                self.check_const_purity(value)
            }
            Expr::If {
                cond,
                then_branch,
                else_branch,
                ..
            } => {
                self.check_const_purity(cond)?;
                self.check_const_purity(then_branch)?;
                self.check_const_purity(else_branch)
            }
            Expr::Unary { op, operand, span } => {
                ensure!(
                    op.allowed_in(self.dialect),
                    "Operator '{}' at {span} is forbidden in {:?}",
                    op.keyword(),
                    self.dialect
                );
                ensure!(
                    !op.is_imperative_buf_op(),
                    "Imperative buffer operation '{}' is not allowed in constant expression at {span}",
                    op.keyword()
                );
                self.check_const_purity(operand)
            }
            Expr::Binary {
                op,
                left,
                right,
                span,
            } => {
                ensure!(
                    op.allowed_in(self.dialect),
                    "Operator '{}' at {span} is forbidden in {:?}",
                    op.keyword(),
                    self.dialect
                );
                ensure!(
                    !op.is_imperative_buf_op(),
                    "Imperative buffer operation '{}' is not allowed in constant expression at {span}",
                    op.keyword()
                );
                self.check_const_purity(left)?;
                self.check_const_purity(right)
            }
            Expr::Ternary {
                op,
                first,
                second,
                third,
                span,
            } => {
                ensure!(
                    op.allowed_in(self.dialect),
                    "Operator '{}' at {span} is forbidden in {:?}",
                    op.keyword(),
                    self.dialect
                );
                ensure!(
                    !op.is_imperative_buf_op(),
                    "Imperative buffer operation '{}' is not allowed in constant expression at {span}",
                    op.keyword()
                );
                self.check_const_purity(first)?;
                self.check_const_purity(second)?;
                self.check_const_purity(third)
            }
            Expr::Fault { span } => {
                ensure!(
                    self.dialect == StageDialect::StageL3,
                    "'fault' trap at {span} is forbidden in total StageR 3"
                );
                Ok(())
            }
            Expr::Conv {
                total, expr, span, ..
            } => {
                ensure!(
                    *total == (self.dialect == StageDialect::StageR3),
                    "Numeric 'conv' dialect mismatch at {span}"
                );
                self.check_const_purity(expr)
            }
            Expr::Bitcast { expr, span, .. } => {
                ensure!(
                    self.dialect == StageDialect::StageL3,
                    "StageL 3 conversion/bitcast node at {span} is forbidden in StageR 3"
                );
                self.check_const_purity(expr)
            }
            Expr::DctsString { span, .. }
            | Expr::DcLit { span, .. }
            | Expr::OptionNone { span, .. }
            | Expr::ErrorTag { span, .. } => {
                ensure!(
                    self.dialect == StageDialect::StageR3,
                    "StageR 3 literal/tag expression is not supported in StageL 3 at {span}"
                );
                Ok(())
            }
            Expr::NamedStructNew { fields, span, .. } => {
                ensure!(
                    self.dialect == StageDialect::StageR3,
                    "StageR 3 'structNew' is not supported in StageL 3 at {span}"
                );
                for (_, fexpr, _) in fields {
                    self.check_const_purity(fexpr)?;
                }
                Ok(())
            }
            Expr::EnumVariant { payload, span, .. } => {
                ensure!(
                    self.dialect == StageDialect::StageR3,
                    "StageR 3 enum constructor is not supported in StageL 3 at {span}"
                );
                if let Some(p) = payload {
                    self.check_const_purity(p)?;
                }
                Ok(())
            }
            Expr::OptionSome { value, span, .. }
            | Expr::ResultOk { value, span, .. } => {
                ensure!(
                    self.dialect == StageDialect::StageR3,
                    "StageR 3 wrapper expression is not supported in StageL 3 at {span}"
                );
                self.check_const_purity(value)
            }
            Expr::ResultErr { error, span, .. } => {
                ensure!(
                    self.dialect == StageDialect::StageR3,
                    "StageR 3 result:err expression is not supported in StageL 3 at {span}"
                );
                self.check_const_purity(error)
            }
            Expr::NamedField { base, span, .. }
            | Expr::WrapNominal { expr: base, span, .. }
            | Expr::UnwrapNominal { expr: base, span, .. } => {
                ensure!(
                    self.dialect == StageDialect::StageR3,
                    "StageR 3 expression is not supported in StageL 3 at {span}"
                );
                self.check_const_purity(base)
            }
            Expr::NamedWith {
                base, value, span, ..
            } => {
                ensure!(
                    self.dialect == StageDialect::StageR3,
                    "StageR 3 expression is not supported in StageL 3 at {span}"
                );
                self.check_const_purity(base)?;
                self.check_const_purity(value)
            }
            Expr::Match {
                scrutinee,
                arms,
                fallback,
                span,
            } => {
                ensure!(
                    self.dialect == StageDialect::StageR3,
                    "StageR 3 'match' expression is not supported in StageL 3 at {span}"
                );
                self.check_const_purity(scrutinee)?;
                for arm in arms {
                    self.check_const_purity(&arm.body)?;
                }
                if let Some(fb) = fallback {
                    self.check_const_purity(fb)?;
                }
                Ok(())
            }
            Expr::BufNew { span, .. } => bail!(
                "Mutable buffer allocation 'bufNew' is not allowed in constant expression at {span}"
            ),
            Expr::Block { span, .. } => bail!(
                "Block statement 'do' is not allowed in constant expression at {span}"
            ),
            Expr::Call { span, .. } | Expr::InlineCall { span, .. } => bail!(
                "Routine calls are not allowed in constant expression at {span}"
            ),
            Expr::AnonRoutine(r) => bail!(
                "Anonymous routine at {} is not allowed in constant expression",
                r.span
            ),
            Expr::Let { .. }
            | Expr::Const { .. }
            | Expr::Set { .. }
            | Expr::RoutineDecl(_)
            | Expr::While { .. }
            | Expr::ForEach { .. }
            | Expr::Break { .. }
            | Expr::Continue { .. }
            | Expr::Return { .. }
            | Expr::Propagate { .. } => {
                bail!(
                    "Imperative/control-flow construct is not allowed in constant expression at {}",
                    expr.span()
                )
            }
        }
    }

    /// Validates all inline `test` blocks attached to `r` and then checks its
    /// routine body.
    pub(super) fn validate_top_routine(
        &mut self,
        r: &'a RoutineDecl,
    ) -> Result<()> {
        let mut canon_params = Vec::with_capacity(r.params.len());
        for p in &r.params {
            canon_params.push(self.canonicalize_type(&p.ty, p.span)?);
        }
        let canon_ret = self.canonicalize_type(&r.return_type, r.span)?;

        for tb in &r.tests {
            for tc in &tb.cases {
                ensure!(
                    tc.inputs.len() == canon_params.len(),
                    "Test case in routine '{}' at {} provides {} arguments, expected {}",
                    r.name,
                    tc.span,
                    tc.inputs.len(),
                    canon_params.len()
                );
                for (arg, param_ty) in tc.inputs.iter().zip(&canon_params) {
                    self.check_const_purity(arg)?;
                    let mut state = FlowState::new();
                    let mut ctx = BodyCtx::new(param_ty.clone(), false);
                    let calls_before = self.resolved_calls.len();
                    let arg_ty =
                        self.check_expr(arg, &mut state, &mut ctx)?;
                    ensure!(
                        self.resolved_calls.len() == calls_before,
                        "Routine/overloaded operator calls are not allowed in constant test input for '{}' at {}",
                        r.name,
                        tc.span
                    );
                    ensure!(
                        types_compatible(&arg_ty, param_ty),
                        "Test input in routine '{}' at {} has type '{}', expected '{}'",
                        r.name,
                        tc.span,
                        arg_ty,
                        param_ty
                    );
                }
                match &tc.expected {
                    TestExpected::Expr(expected_expr) => {
                        ensure!(
                            canon_ret != StageLType::Void,
                            "Test case in void routine '{}' at {} must omit 'out' value expectation",
                            r.name,
                            tc.span
                        );
                        self.check_const_purity(expected_expr)?;
                        let mut state = FlowState::new();
                        let mut ctx = BodyCtx::new(canon_ret.clone(), false);
                        let calls_before = self.resolved_calls.len();
                        let exp_ty = self.check_expr(
                            expected_expr,
                            &mut state,
                            &mut ctx,
                        )?;
                        ensure!(
                            self.resolved_calls.len() == calls_before,
                            "Routine/overloaded operator calls are not allowed in constant test output for '{}' at {}",
                            r.name,
                            tc.span
                        );
                        ensure!(
                            types_compatible(&exp_ty, &canon_ret),
                            "Test expected output in routine '{}' at {} has type '{}', expected '{}'",
                            r.name,
                            tc.span,
                            exp_ty,
                            canon_ret
                        );
                    }
                    TestExpected::Fault(sp) => {
                        ensure!(
                            self.dialect == StageDialect::StageL3,
                            "'out fault' test expectation for '{}' at {sp} is forbidden in StageR 3 (StageR 3 is total)",
                            r.name
                        );
                    }
                    TestExpected::Void(sp) => {
                        ensure!(
                            canon_ret == StageLType::Void,
                            "Test case in non-void routine '{}' (returns '{}') at {sp} must provide an expected output",
                            r.name,
                            canon_ret
                        );
                    }
                }
            }
        }

        self.validate_routine_body(r, &canon_params, &canon_ret)
    }
}
