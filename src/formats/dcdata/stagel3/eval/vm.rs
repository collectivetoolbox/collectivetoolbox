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

//! Unified workspace runtime VM, lexical environments, tail-call trampoline,
//! and expression evaluator for StageL 3 and StageR 3.
//!
//! Executes validated modules with per-module dialect tracking, strict
//! left-to-right operand evaluation, shared mutable capture cells, bounded
//! compute/memory quotas, and constant-stack tail-call trampolining.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

use malachite::base::num::basic::traits::Zero;
use malachite::base::num::conversion::traits::PowerOf2Digits;
use malachite::base::num::logic::traits::SignificantBits;
use malachite::{Integer, Natural, Rational};

use super::super::ast::{
    BinaryOp, BlockItem, Expr, MatchArm, RoutineDecl, Span, StageDialect,
    StageLType, StageRErrorTag, StructTypeRef, SymbolTarget, TestCase,
    TestExpected, TopDecl, TopValueDecl,
};
use super::super::validator::{
    ImportedSymbol, RoutineKey, ValidatedModule, ValidatedWorkspace, ValueKey,
    unbridge_stager_type_for_stage1,
};
use super::ops::{
    eval_binary, eval_bitcast, eval_conv, eval_ternary, eval_unary,
};
use super::{
    BufStorage, EvalConfig, RoutineOutcome, TestCaseReport, TestSuiteReport,
    Value,
};
use crate::DcChar;

type Cell = Rc<RefCell<Option<Value>>>;
type TypeKey = (Option<String>, String);

#[derive(Debug, Clone)]
pub(super) struct LexFrame {
    parent: Option<Rc<LexFrame>>,
    pub(super) vars: HashMap<String, Cell>,
    nested_verbs: Rc<RefCell<HashMap<String, ClosureVerb>>>,
    pub(super) return_ty: Option<StageLType>,
}

impl LexFrame {
    pub(super) fn new(parent: Option<Rc<LexFrame>>) -> Self {
        let return_ty = parent.as_ref().and_then(|p| p.return_ty.clone());
        Self {
            parent,
            vars: HashMap::new(),
            nested_verbs: Rc::new(RefCell::new(HashMap::new())),
            return_ty,
        }
    }

    fn lookup_var(&self, name: &str) -> Option<Cell> {
        if let Some(c) = self.vars.get(name) {
            return Some(Rc::clone(c));
        }
        let mut cur = self.parent.as_deref();
        while let Some(fr) = cur {
            if let Some(c) = fr.vars.get(name) {
                return Some(Rc::clone(c));
            }
            cur = fr.parent.as_deref();
        }
        None
    }

    fn lookup_nested_verb(&self, name: &str) -> Option<ClosureVerb> {
        if let Some(v) = self.nested_verbs.borrow().get(name) {
            return Some(v.clone());
        }
        let mut cur = self.parent.as_deref();
        while let Some(fr) = cur {
            if let Some(v) = fr.nested_verbs.borrow().get(name) {
                return Some(v.clone());
            }
            cur = fr.parent.as_deref();
        }
        None
    }
}

#[derive(Debug, Clone)]
pub(super) struct ClosureVerb {
    decl: RoutineDecl,
    module_idx: usize,
    captured_frame: Rc<LexFrame>,
}

#[derive(Debug, Clone)]
pub(super) enum CallableRoutine {
    Top {
        module_idx: usize,
        key: RoutineKey,
    },
    Nested(ClosureVerb),
}

pub(super) enum EvalSignal {
    Value(Value),
    Return(Value),
    Break,
    Continue,
    TailCall {
        callee: CallableRoutine,
        args: Vec<Value>,
    },
    Fault(String),
}

#[derive(Debug, Clone)]
enum ResolvedDcImportTarget {
    Value { module_idx: usize, key: ValueKey },
    Routine { module_idx: usize, key: RoutineKey },
}

pub(super) struct ModuleRuntime {
    #[allow(dead_code, reason = "Retained for diagnostics and debugging")]
    pub(super) namespace_key: Option<String>,
    #[allow(
        dead_code,
        reason = "Tracks per-module dialect for mixed StageL 3 / StageR 3 workspaces"
    )]
    pub(super) dialect: StageDialect,
    pub(super) constants: HashMap<ValueKey, Value>,
    pub(super) dc_constants: HashMap<DcChar, ValueKey>,
    pub(super) routines: HashMap<RoutineKey, RoutineDecl>,
    pub(super) dc_routines: HashMap<DcChar, RoutineKey>,
    pub(super) resolved_values: HashMap<(usize, usize), (usize, ValueKey)>,
    pub(super) resolved_calls: HashMap<(usize, usize), (usize, RoutineKey)>,
    pub(super) resolved_structs: HashMap<(usize, usize), TypeKey>,
    pub(super) resolved_named_structs: HashMap<(usize, usize), TypeKey>,
    pub(super) resolved_enum_variants:
        HashMap<(usize, usize), (TypeKey, String)>,
    pub(super) dc_structs: HashMap<DcChar, TypeKey>,
    pub(super) local_structs: HashMap<String, TypeKey>,
    pub(super) dc_nominals: HashMap<DcChar, TypeKey>,
    pub(super) local_nominals: HashMap<String, TypeKey>,
    imports_by_dc: HashMap<DcChar, ResolvedDcImportTarget>,
}

/// Multi-module StageL 3 / StageR 3 runtime VM supporting cross-namespace calls,
/// constant evaluation, tail-call trampolining, and inline `test` execution.
pub struct StageLWorkspaceVm {
    pub(super) modules: Vec<ModuleRuntime>,
    #[allow(dead_code, reason = "Retained for namespace lookup diagnostics")]
    ns_to_indices: HashMap<String, Vec<usize>>,
    pub(super) config: EvalConfig,
}

impl StageLWorkspaceVm {
    /// Initializes a workspace VM from validated modules, eagerly evaluating all
    /// top-level `const` and `pub` constant initializers in topological order.
    pub fn from_validated_workspace(
        ws: &ValidatedWorkspace,
        config: EvalConfig,
    ) -> Result<Self> {
        let mut ns_to_indices: HashMap<String, Vec<usize>> = HashMap::new();
        for (idx, vm) in ws.modules.iter().enumerate() {
            if let Some(ref key) = vm.namespace_key {
                ns_to_indices.entry(key.clone()).or_default().push(idx);
            }
        }

        let mut vm_state = Self {
            modules: Vec::with_capacity(ws.modules.len()),
            ns_to_indices,
            config,
        };

        for (mod_idx, vm) in ws.modules.iter().enumerate() {
            let (runtime, pending_consts) = build_module_runtime(
                &vm_state.ns_to_indices,
                &vm_state.modules,
                mod_idx,
                vm,
            )?;
            vm_state.modules.push(runtime);
            // Evaluate top-level constants in topological dependency order
            // (handling valid out-of-order pure references).
            eval_pending_constants(&mut vm_state, mod_idx, &pending_consts)?;
        }

        Ok(vm_state)
    }

    /// Invokes a named routine in module index `module_idx`.
    pub fn call_routine(
        &self,
        module_idx: usize,
        routine_name: &str,
        args: Vec<Value>,
    ) -> Result<RoutineOutcome> {
        let mod_rt = self
            .modules
            .get(module_idx)
            .context("Invalid module index during call")?;
        let mut by_name: Vec<&RoutineKey> = mod_rt
            .routines
            .keys()
            .filter(|k| k.name == routine_name)
            .collect();
        if by_name.len() > 1 {
            by_name.retain(|k| {
                k.params.len() == args.len()
                    && args
                        .iter()
                        .zip(&k.params)
                        .all(|(v, ty)| v.matches_stagel_type(ty))
            });
        }
        ensure!(
            by_name.len() == 1,
            "Routine '{routine_name}' not uniquely matched for runtime call ({} candidate(s))",
            by_name.len()
        );
        let key = (*by_name
            .first()
            .context("Expected single matched routine key")?)
        .clone();
        self.call_routine_by_key(module_idx, key, args)
    }

    /// Invokes a routine identified by [`RoutineKey`] in `module_idx`.
    pub fn call_routine_by_key(
        &self,
        module_idx: usize,
        key: RoutineKey,
        args: Vec<Value>,
    ) -> Result<RoutineOutcome> {
        let mut quota = self.config.compute_quota;
        self.invoke_trampoline(
            CallableRoutine::Top { module_idx, key },
            args,
            0,
            &mut quota,
        )
    }

    /// Runs all inline `test` cases across every module in `ws`.
    pub fn run_all_tests(
        &self,
        ws: &ValidatedWorkspace,
    ) -> Result<TestSuiteReport> {
        let mut report = TestSuiteReport::default();
        for (mod_idx, vm) in ws.modules.iter().enumerate() {
            for decl in &vm.module.items {
                let TopDecl::Routine(r) = decl else {
                    continue;
                };
                let rkey = vm.routine_key(r);
                for tb in &r.tests {
                    for (case_idx, tc) in tb.cases.iter().enumerate() {
                        let case_report = self.run_single_test_case(
                            mod_idx,
                            vm.namespace_key.clone(),
                            r,
                            rkey.clone(),
                            tb.label.clone(),
                            case_idx,
                            tc,
                        )?;
                        if case_report.passed {
                            report.passed_count =
                                report.passed_count.saturating_add(1);
                        } else {
                            report.failed_count =
                                report.failed_count.saturating_add(1);
                        }
                        report.cases.push(case_report);
                    }
                }
            }
        }
        Ok(report)
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "Passes pre-canonicalized RoutineKey alongside diagnostic test context"
    )]
    fn run_single_test_case(
        &self,
        mod_idx: usize,
        namespace: Option<String>,
        routine: &RoutineDecl,
        routine_key: RoutineKey,
        block_label: Option<String>,
        case_index: usize,
        tc: &TestCase,
    ) -> Result<TestCaseReport> {
        let make_report = |passed, detail| TestCaseReport {
            namespace: namespace.clone(),
            routine_name: routine.name.clone(),
            block_label: block_label.clone(),
            case_index,
            passed,
            detail,
        };

        let mut args = Vec::with_capacity(tc.inputs.len());
        let empty_frame = LexFrame::new(None);
        for inp in &tc.inputs {
            let mut quota = self.config.compute_quota;
            match self.eval_expr(mod_idx, inp, &empty_frame, false, 0, &mut quota)?
            {
                EvalSignal::Value(v) => args.push(v),
                EvalSignal::Fault(msg) => {
                    return Ok(make_report(
                        false,
                        format!("Evaluating test input trapped: {msg}"),
                    ));
                }
                _ => bail!("Control-flow keyword disallowed in test input"),
            }
        }

        let outcome = self.call_routine_by_key(mod_idx, routine_key, args)?;
        match (&tc.expected, outcome) {
            (TestExpected::Fault(_), RoutineOutcome::Trapped(msg)) => {
                Ok(make_report(true, format!("trapped as expected ({msg})")))
            }
            (TestExpected::Fault(_), RoutineOutcome::Returned(v)) => {
                Ok(make_report(false, format!("expected fault trap, got {v:?}")))
            }
            (TestExpected::Void(_), RoutineOutcome::Returned(Value::Void)) => {
                Ok(make_report(true, String::from("void ok")))
            }
            (TestExpected::Void(_), other) => Ok(make_report(
                false,
                format!("expected void completion, got {other:?}"),
            )),
            (TestExpected::Expr(_), RoutineOutcome::Trapped(msg)) => Ok(make_report(
                false,
                format!("routine trapped unexpectedly: {msg}"),
            )),
            (TestExpected::Expr(exp_expr), RoutineOutcome::Returned(actual)) => {
                let mut quota = self.config.compute_quota;
                let expected = match self.eval_expr(
                    mod_idx,
                    exp_expr,
                    &empty_frame,
                    false,
                    0,
                    &mut quota,
                )? {
                    EvalSignal::Value(v) => v,
                    EvalSignal::Fault(msg) => {
                        return Ok(make_report(
                            false,
                            format!("evaluating expected output trapped: {msg}"),
                        ));
                    }
                    _ => bail!("Control-flow disallowed in test expected expr"),
                };
                if actual == expected {
                    Ok(make_report(true, format!("matched {expected:?}")))
                } else {
                    Ok(make_report(
                        false,
                        format!("expected {expected:?}, got {actual:?}"),
                    ))
                }
            }
        }
    }

    pub(super) fn consume_quota(
        &self,
        quota: &mut u64,
    ) -> std::result::Result<(), String> {
        if *quota == 0 {
            return Err(String::from("compute quota exceeded"));
        }
        *quota = quota.saturating_sub(1);
        Ok(())
    }

    pub(super) fn check_int_limit(
        &self,
        n: &Integer,
        context: &str,
    ) -> Option<EvalSignal> {
        let bits = n.significant_bits();
        if bits > self.config.max_bigint_bits {
            Some(EvalSignal::Fault(format!(
                "{context} integer ({bits} bits) exceeds max_bigint_bits ({})",
                self.config.max_bigint_bits
            )))
        } else {
            None
        }
    }

    pub(super) fn check_rat_limit(
        &self,
        r: &Rational,
        context: &str,
    ) -> Option<EvalSignal> {
        let max_bits = r
            .numerator_ref()
            .significant_bits()
            .max(r.denominator_ref().significant_bits());
        if max_bits > self.config.max_bigint_bits {
            Some(EvalSignal::Fault(format!(
                "{context} rational component ({max_bits} bits) exceeds max_bigint_bits ({})",
                self.config.max_bigint_bits
            )))
        } else {
            None
        }
    }

    pub(super) fn check_seq_limit(
        &self,
        len: usize,
        context: &str,
    ) -> Option<EvalSignal> {
        if len > self.config.max_seq_elems {
            Some(EvalSignal::Fault(format!(
                "{context} ({len}) exceeds max_seq_elems ({})",
                self.config.max_seq_elems
            )))
        } else {
            None
        }
    }

    pub(super) fn invoke_trampoline(
        &self,
        mut callee: CallableRoutine,
        mut args: Vec<Value>,
        depth: usize,
        quota: &mut u64,
    ) -> Result<RoutineOutcome> {
        if depth > self.config.max_call_depth {
            let name = match &callee {
                CallableRoutine::Top { key, .. } => key.name.as_str(),
                CallableRoutine::Nested(cv) => cv.decl.name.as_str(),
            };
            return Ok(RoutineOutcome::Trapped(format!(
                "max non-tail call depth ({}) exceeded calling '{name}'",
                self.config.max_call_depth
            )));
        }

        loop {
            if let Err(msg) = self.consume_quota(quota) {
                return Ok(RoutineOutcome::Trapped(msg));
            }
            let (mod_idx, decl, parent_frame) = match callee {
                CallableRoutine::Top { module_idx, key } => {
                    let mod_rt = self
                        .modules
                        .get(module_idx)
                        .context("Invalid module index during call")?;
                    let decl = mod_rt
                        .routines
                        .get(&key)
                        .cloned()
                        .with_context(|| {
                            format!(
                                "Routine '{}' not found at runtime",
                                key.name
                            )
                        })?;
                    (module_idx, decl, None)
                }
                CallableRoutine::Nested(cv) => {
                    (cv.module_idx, cv.decl, Some(cv.captured_frame))
                }
            };

            let mut frame = LexFrame::new(parent_frame);
            frame.return_ty = Some(decl.return_type.clone());
            ensure!(
                args.len() == decl.params.len(),
                "Runtime argument count mismatch calling '{}': expected {}, got {}",
                decl.name,
                decl.params.len(),
                args.len()
            );
            for (param, val) in decl.params.iter().zip(args.drain(..)) {
                frame
                    .vars
                    .insert(param.name.clone(), Rc::new(RefCell::new(Some(val))));
            }

            match self.eval_expr(mod_idx, &decl.body, &frame, true, depth, quota)?
            {
                EvalSignal::Value(v) | EvalSignal::Return(v) => {
                    return Ok(RoutineOutcome::Returned(v));
                }
                EvalSignal::Fault(msg) => {
                    return Ok(RoutineOutcome::Trapped(format!(
                        "{msg} <- {}",
                        decl.name
                    )));
                }
                EvalSignal::TailCall {
                    callee: next_callee,
                    args: next_args,
                } => {
                    callee = next_callee;
                    args = next_args;
                }
                EvalSignal::Break | EvalSignal::Continue => {
                    bail!(
                        "Uncaught loop control signal exiting routine '{}'",
                        decl.name
                    );
                }
            }
        }
    }

    pub(super) fn eval_expr(
        &self,
        mod_idx: usize,
        expr: &Expr,
        frame: &LexFrame,
        in_tail: bool,
        depth: usize,
        quota: &mut u64,
    ) -> Result<EvalSignal> {
        if let Err(msg) = self.consume_quota(quota) {
            return Ok(EvalSignal::Fault(msg));
        }

        match expr {
            Expr::Bool { value, .. } => Ok(EvalSignal::Value(Value::Bool(*value))),
            Expr::Int { value, .. } => {
                match Value::from_typed_int(value, self.config.max_bigint_bits) {
                    Ok(v) => Ok(EvalSignal::Value(v)),
                    Err(msg) => Ok(EvalSignal::Fault(msg)),
                }
            }
            Expr::Bytes { value, .. } => {
                if let Some(sig) =
                    self.check_seq_limit(value.len(), "byte sequence length")
                {
                    return Ok(sig);
                }
                let elems = value.iter().copied().map(Value::U8).collect();
                Ok(EvalSignal::Value(Value::List(Rc::new(elems))))
            }
            Expr::DctsString { value, .. } => {
                if let Some(sig) =
                    self.check_seq_limit(value.len(), "Dcts string literal length")
                {
                    return Ok(sig);
                }
                Ok(EvalSignal::Value(Value::S(Rc::new(value.clone()))))
            }
            Expr::DcLit { value, .. } => Ok(EvalSignal::Value(Value::Dc(*value))),
            Expr::ErrorTag { tag, .. } => {
                Ok(EvalSignal::Value(Value::Error(*tag)))
            }
            Expr::OptionNone { .. } => Ok(EvalSignal::Value(Value::Option(None))),
            Expr::Const { .. } | Expr::RoutineDecl(_) | Expr::Nop { .. } => {
                Ok(EvalSignal::Value(Value::Void))
            }
            Expr::Break { .. } => Ok(EvalSignal::Break),
            Expr::Continue { .. } => Ok(EvalSignal::Continue),
            Expr::Fault { span } => {
                Ok(EvalSignal::Fault(format!("explicit fault trap at {span}")))
            }
            Expr::Let { name, init, .. } => {
                if let Some(init_expr) = init {
                    match self.eval_expr(
                        mod_idx, init_expr, frame, false, depth, quota,
                    )? {
                        EvalSignal::Value(_) => {}
                        sig => return Ok(sig),
                    }
                }
                bail!("'let' ('{name}') must execute inside a block frame")
            }
            Expr::ValueRef { ty, target, span } => {
                let val = match target {
                    SymbolTarget::Ident(name) => {
                        if let Some(cell) = frame.lookup_var(name) {
                            cell.borrow().clone().context(
                                "Uninitialized variable read at runtime",
                            )?
                        } else {
                            self.lookup_module_value(
                                mod_idx,
                                *span,
                                name,
                                ty.as_ref(),
                            )?
                        }
                    }
                    SymbolTarget::Dc(dc) => self.lookup_dc_value(mod_idx, *dc)?,
                };
                Ok(EvalSignal::Value(val))
            }
            Expr::Set { name, value, .. } => {
                let new_val = match self.eval_expr(
                    mod_idx, value, frame, false, depth, quota,
                )? {
                    EvalSignal::Value(v) => v,
                    sig => return Ok(sig),
                };
                let cell = frame
                    .lookup_var(name)
                    .with_context(|| format!("Variable '{name}' missing at runtime"))?;
                *cell.borrow_mut() = Some(new_val);
                Ok(EvalSignal::Value(Value::Void))
            }
            Expr::Call { target, args, span } => self.eval_call(
                mod_idx, target, args, *span, frame, in_tail, depth, quota,
            ),
            Expr::Block { stmts, .. } => {
                self.eval_block(mod_idx, stmts, frame, in_tail, depth, quota)
            }
            Expr::If {
                cond,
                then_branch,
                else_branch,
                ..
            } => self.eval_if(
                mod_idx,
                cond,
                then_branch,
                else_branch,
                frame,
                in_tail,
                depth,
                quota,
            ),
            Expr::While { cond, body, .. } => {
                self.eval_while(mod_idx, cond, body, frame, depth, quota)
            }
            Expr::ForEach {
                elem_name,
                list,
                body,
                ..
            } => self.eval_foreach(
                mod_idx, elem_name, list, body, frame, depth, quota,
            ),
            Expr::Return { value, .. } => {
                if let Some(vexpr) = value {
                    match self.eval_expr(
                        mod_idx, vexpr, frame, true, depth, quota,
                    )? {
                        EvalSignal::Value(v) => Ok(EvalSignal::Return(v)),
                        sig => Ok(sig),
                    }
                } else {
                    Ok(EvalSignal::Return(Value::Void))
                }
            }
            Expr::Match {
                scrutinee,
                arms,
                fallback,
                span,
            } => self.eval_match(
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
                self.eval_propagate(mod_idx, inner, *span, frame, depth, quota)
            }
            Expr::Unary { .. }
            | Expr::Binary { .. }
            | Expr::Ternary { .. }
            | Expr::Conv { .. }
            | Expr::Bitcast { .. } => {
                self.eval_op_expr(mod_idx, expr, frame, depth, quota)
            }
            Expr::List { .. }
            | Expr::BufNew { .. }
            | Expr::TupleNew { .. }
            | Expr::StructNew { .. }
            | Expr::Field { .. }
            | Expr::With { .. }
            | Expr::NamedStructNew { .. }
            | Expr::EnumVariant { .. }
            | Expr::OptionSome { .. }
            | Expr::ResultOk { .. }
            | Expr::ResultErr { .. }
            | Expr::NamedField { .. }
            | Expr::NamedWith { .. }
            | Expr::WrapNominal { .. }
            | Expr::UnwrapNominal { .. } => {
                self.eval_data_expr(mod_idx, expr, frame, depth, quota)
            }
        }
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "Forwards VM execution state across call arguments and tail-call check"
    )]
    fn eval_call(
        &self,
        mod_idx: usize,
        target: &SymbolTarget,
        args: &[Expr],
        span: Span,
        frame: &LexFrame,
        in_tail: bool,
        depth: usize,
        quota: &mut u64,
    ) -> Result<EvalSignal> {
        let mut evaluated_args = Vec::with_capacity(args.len());
        for a in args {
            match self.eval_expr(mod_idx, a, frame, false, depth, quota)? {
                EvalSignal::Value(v) => evaluated_args.push(v),
                sig => return Ok(sig),
            }
        }
        let callee = self.resolve_callee(mod_idx, target, span, frame)?;
        let cross_stager_to_stage1 = match &callee {
            CallableRoutine::Top {
                module_idx: callee_mod_idx,
                ..
            } => {
                let caller_is_sr = self
                    .modules
                    .get(mod_idx)
                    .is_some_and(|rt| rt.dialect == StageDialect::StageR3);
                let callee_is_s1 = self
                    .modules
                    .get(*callee_mod_idx)
                    .is_some_and(|rt| rt.dialect == StageDialect::StageL3);
                caller_is_sr && callee_is_s1
            }
            CallableRoutine::Nested(_) => false,
        };
        if cross_stager_to_stage1 {
            let mut marshaled_args = Vec::with_capacity(evaluated_args.len());
            for arg in evaluated_args {
                marshaled_args.push(marshal_stager_val_to_stage1(arg)?);
            }
            return match self.invoke_trampoline(
                callee,
                marshaled_args,
                depth.saturating_add(1),
                quota,
            )? {
                RoutineOutcome::Returned(v) => {
                    let stager_val = marshal_stage1_val_to_stager(v)?;
                    if let Some(sig) = self.check_unmarshaled_limits(&stager_val)
                    {
                        return Ok(sig);
                    }
                    Ok(EvalSignal::Value(stager_val))
                }
                RoutineOutcome::Trapped(msg) => Ok(EvalSignal::Fault(msg)),
            };
        }
        if in_tail {
            Ok(EvalSignal::TailCall {
                callee,
                args: evaluated_args,
            })
        } else {
            match self.invoke_trampoline(
                callee,
                evaluated_args,
                depth.saturating_add(1),
                quota,
            )? {
                RoutineOutcome::Returned(v) => Ok(EvalSignal::Value(v)),
                RoutineOutcome::Trapped(msg) => Ok(EvalSignal::Fault(msg)),
            }
        }
    }

    fn eval_block(
        &self,
        mod_idx: usize,
        stmts: &[BlockItem],
        frame: &LexFrame,
        in_tail: bool,
        depth: usize,
        quota: &mut u64,
    ) -> Result<EvalSignal> {
        let mut block_frame = LexFrame::new(Some(Rc::new(frame.clone())));
        let mut pending_routines = Vec::new();
        for item in stmts {
            match item {
                BlockItem::Expr(Expr::RoutineDecl(r)) => {
                    pending_routines.push((**r).clone());
                }
                // Pre-create `let`/`const` variable cells before freezing
                // `live_frame` so nested closures share the exact cells
                // updated sequentially below.
                BlockItem::Expr(Expr::Let { name, .. })
                | BlockItem::Expr(Expr::Const { name, .. }) => {
                    block_frame
                        .vars
                        .insert(name.clone(), Rc::new(RefCell::new(None)));
                }
                _ => {}
            }
        }

        let live_frame = Rc::new(block_frame);
        {
            let mut map = live_frame.nested_verbs.borrow_mut();
            for r in pending_routines {
                map.insert(
                    r.name.clone(),
                    ClosureVerb {
                        decl: r,
                        module_idx: mod_idx,
                        captured_frame: Rc::clone(&live_frame),
                    },
                );
            }
        }

        let last_expr_idx = stmts
            .iter()
            .rposition(|it| matches!(it, BlockItem::Expr(_)));
        let mut last_val = Value::Void;

        for (idx, item) in stmts.iter().enumerate() {
            let BlockItem::Expr(e) = item else {
                continue;
            };
            let is_last = Some(idx) == last_expr_idx;
            match e {
                Expr::Let { name, init, .. } => {
                    let val = if let Some(init_e) = init {
                        match self.eval_expr(
                            mod_idx,
                            init_e,
                            &live_frame,
                            false,
                            depth,
                            quota,
                        )? {
                            EvalSignal::Value(v) => Some(v),
                            sig => return Ok(sig),
                        }
                    } else {
                        None
                    };
                    if let Some(cell) = live_frame.vars.get(name) {
                        *cell.borrow_mut() = val;
                    }
                    if is_last {
                        last_val = Value::Void;
                    }
                }
                Expr::Const { name, value, .. } => {
                    let val = match self.eval_expr(
                        mod_idx,
                        value,
                        &live_frame,
                        false,
                        depth,
                        quota,
                    )? {
                        EvalSignal::Value(v) => v,
                        sig => return Ok(sig),
                    };
                    if let Some(cell) = live_frame.vars.get(name) {
                        *cell.borrow_mut() = Some(val);
                    }
                    if is_last {
                        last_val = Value::Void;
                    }
                }
                Expr::RoutineDecl(_) => {
                    if is_last {
                        last_val = Value::Void;
                    }
                }
                _ => {
                    match self.eval_expr(
                        mod_idx,
                        e,
                        &live_frame,
                        in_tail && is_last,
                        depth,
                        quota,
                    )? {
                        EvalSignal::Value(v) => {
                            if is_last {
                                last_val = v;
                            }
                        }
                        sig => return Ok(sig),
                    }
                }
            }
        }
        Ok(EvalSignal::Value(last_val))
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "Forwards VM evaluation state and tail flag across conditional branches"
    )]
    fn eval_if(
        &self,
        mod_idx: usize,
        cond: &Expr,
        then_branch: &Expr,
        else_branch: &Expr,
        frame: &LexFrame,
        in_tail: bool,
        depth: usize,
        quota: &mut u64,
    ) -> Result<EvalSignal> {
        let cval =
            match self.eval_expr(mod_idx, cond, frame, false, depth, quota)? {
                EvalSignal::Value(v) => v,
                sig => return Ok(sig),
            };
        let Value::Bool(b) = cval else {
            bail!("'if' condition did not evaluate to bool");
        };
        let chosen = if b { then_branch } else { else_branch };
        self.eval_expr(mod_idx, chosen, frame, in_tail, depth, quota)
    }

    fn eval_while(
        &self,
        mod_idx: usize,
        cond: &Expr,
        body: &Expr,
        frame: &LexFrame,
        depth: usize,
        quota: &mut u64,
    ) -> Result<EvalSignal> {
        loop {
            if let Err(msg) = self.consume_quota(quota) {
                return Ok(EvalSignal::Fault(msg));
            }
            let cval =
                match self.eval_expr(mod_idx, cond, frame, false, depth, quota)? {
                    EvalSignal::Value(v) => v,
                    sig => return Ok(sig),
                };
            let Value::Bool(b) = cval else {
                bail!("'while' condition did not evaluate to bool");
            };
            if !b {
                return Ok(EvalSignal::Value(Value::Void));
            }
            match self.eval_expr(mod_idx, body, frame, false, depth, quota)? {
                EvalSignal::Value(_) | EvalSignal::Continue => {}
                EvalSignal::Break => return Ok(EvalSignal::Value(Value::Void)),
                sig => return Ok(sig),
            }
        }
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "Forwards VM evaluation state across loop iterations"
    )]
    fn eval_foreach(
        &self,
        mod_idx: usize,
        elem_name: &str,
        list: &Expr,
        body: &Expr,
        frame: &LexFrame,
        depth: usize,
        quota: &mut u64,
    ) -> Result<EvalSignal> {
        let lval =
            match self.eval_expr(mod_idx, list, frame, false, depth, quota)? {
                EvalSignal::Value(v) => v,
                sig => return Ok(sig),
            };
        enum SeqIter {
            Vals(Rc<Vec<Value>>),
            Dcs(Rc<Vec<DcChar>>),
        }
        let seq = match lval {
            Value::List(elems) => SeqIter::Vals(elems),
            Value::Dcs(chars) => SeqIter::Dcs(chars),
            _ => bail!("'foreach' iterable did not evaluate to l/T or dcs"),
        };
        let len = match &seq {
            SeqIter::Vals(v) => v.len(),
            SeqIter::Dcs(d) => d.len(),
        };
        // Snapshot ensures safe iteration even if other buffers mutate.
        for idx in 0..len {
            if let Err(msg) = self.consume_quota(quota) {
                return Ok(EvalSignal::Fault(msg));
            }
            let item = match &seq {
                SeqIter::Vals(v) => v.get(idx).cloned(),
                SeqIter::Dcs(d) => d.get(idx).copied().map(Value::Dc),
            }
            .context("Validated foreach index out of bounds")?;
            let mut iter_frame = LexFrame::new(Some(Rc::new(frame.clone())));
            iter_frame
                .vars
                .insert(elem_name.to_owned(), Rc::new(RefCell::new(Some(item))));
            match self.eval_expr(mod_idx, body, &iter_frame, false, depth, quota)?
            {
                EvalSignal::Value(_) | EvalSignal::Continue => {}
                EvalSignal::Break => return Ok(EvalSignal::Value(Value::Void)),
                sig => return Ok(sig),
            }
        }
        Ok(EvalSignal::Value(Value::Void))
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "Forwards VM evaluation context and tail-position flag across match arms"
    )]
    fn eval_match(
        &self,
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
        let sval = match self.eval_expr(
            mod_idx, scrutinee, frame, false, depth, quota,
        )? {
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
            other => {
                bail!("Invalid scrutinee value {other:?} for 'match' at {span}")
            }
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
            return self.eval_expr(
                mod_idx, &arm.body, &arm_frame, in_tail, depth, quota,
            );
        }

        if let Some(fb) = fallback {
            return self.eval_expr(mod_idx, fb, frame, in_tail, depth, quota);
        }

        bail!(
            "Non-exhaustive runtime 'match' for variant '{variant_name}' at {span}"
        )
    }

    fn eval_propagate(
        &self,
        mod_idx: usize,
        inner: &Expr,
        span: Span,
        frame: &LexFrame,
        depth: usize,
        quota: &mut u64,
    ) -> Result<EvalSignal> {
        let val =
            match self.eval_expr(mod_idx, inner, frame, false, depth, quota)? {
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

    fn eval_op_expr(
        &self,
        mod_idx: usize,
        expr: &Expr,
        frame: &LexFrame,
        depth: usize,
        quota: &mut u64,
    ) -> Result<EvalSignal> {
        let mut eval =
            |sub| self.eval_expr(mod_idx, sub, frame, false, depth, quota);
        match expr {
            Expr::Unary { op, operand, span } => match eval(operand)? {
                EvalSignal::Value(v) => eval_unary(self, *op, v, *span),
                sig => Ok(sig),
            },
            Expr::Binary {
                op,
                left,
                right,
                span,
            } => {
                let lv = match eval(left)? {
                    EvalSignal::Value(v) => v,
                    sig => return Ok(sig),
                };
                if matches!(op, BinaryOp::And | BinaryOp::Or) {
                    let Value::Bool(lb) = lv else {
                        bail!("Logical operand must be bool");
                    };
                    if *op == BinaryOp::And && !lb {
                        return Ok(EvalSignal::Value(Value::Bool(false)));
                    }
                    if *op == BinaryOp::Or && lb {
                        return Ok(EvalSignal::Value(Value::Bool(true)));
                    }
                    return eval(right);
                }
                let rv = match eval(right)? {
                    EvalSignal::Value(v) => v,
                    sig => return Ok(sig),
                };
                eval_binary(self, *op, lv, rv, *span)
            }
            Expr::Ternary {
                op,
                first,
                second,
                third,
                span,
            } => {
                let v1 = match eval(first)? {
                    EvalSignal::Value(v) => v,
                    sig => return Ok(sig),
                };
                let v2 = match eval(second)? {
                    EvalSignal::Value(v) => v,
                    sig => return Ok(sig),
                };
                let v3 = match eval(third)? {
                    EvalSignal::Value(v) => v,
                    sig => return Ok(sig),
                };
                eval_ternary(*op, v1, v2, v3, *span)
            }
            Expr::Conv {
                total,
                expr: inner,
                target_ty,
                span,
            } => match eval(inner)? {
                EvalSignal::Value(v) => {
                    eval_conv(self, *total, v, target_ty, *span)
                }
                sig => Ok(sig),
            },
            Expr::Bitcast {
                expr: inner,
                target_ty,
                ..
            } => match eval(inner)? {
                EvalSignal::Value(v) => eval_bitcast(v, target_ty),
                sig => Ok(sig),
            },
            other => bail!("Unexpected non-op expression {other:?} in eval_op_expr"),
        }
    }

    fn eval_data_expr(
        &self,
        mod_idx: usize,
        expr: &Expr,
        frame: &LexFrame,
        depth: usize,
        quota: &mut u64,
    ) -> Result<EvalSignal> {
        let mod_rt = self.modules.get(mod_idx).context("Invalid module idx")?;
        let mut eval =
            |sub| self.eval_expr(mod_idx, sub, frame, false, depth, quota);
        match expr {
            Expr::List { elements, .. } => {
                if let Some(sig) =
                    self.check_seq_limit(elements.len(), "list length")
                {
                    return Ok(sig);
                }
                let mut vals = Vec::with_capacity(elements.len());
                for el in elements {
                    match eval(el)? {
                        EvalSignal::Value(v) => vals.push(v),
                        sig => return Ok(sig),
                    }
                }
                Ok(EvalSignal::Value(Value::List(Rc::new(vals))))
            }
            Expr::BufNew { capacity, .. } => {
                let cap_val = match eval(capacity)? {
                    EvalSignal::Value(v) => v,
                    sig => return Ok(sig),
                };
                let cap = cap_val.as_u64_index("bufNew")?;
                if let Some(sig) = self.check_seq_limit(cap, "bufNew capacity") {
                    return Ok(sig);
                }
                let storage = BufStorage {
                    elems: Vec::with_capacity(cap.min(65_536)),
                    frozen: false,
                };
                Ok(EvalSignal::Value(Value::Buf(Rc::new(RefCell::new(storage)))))
            }
            Expr::TupleNew { elements, .. } => {
                let mut vals = Vec::with_capacity(elements.len());
                for el in elements {
                    match eval(el)? {
                        EvalSignal::Value(v) => vals.push(v),
                        sig => return Ok(sig),
                    }
                }
                Ok(EvalSignal::Value(Value::Tuple(Rc::new(vals))))
            }
            Expr::StructNew {
                target,
                fields,
                span,
            } => {
                let type_key = resolve_struct_key(
                    &mod_rt.resolved_structs,
                    &mod_rt.local_structs,
                    &mod_rt.dc_structs,
                    target,
                    *span,
                    "Struct",
                )?;
                let mut vals = Vec::with_capacity(fields.len());
                for f in fields {
                    match eval(f)? {
                        EvalSignal::Value(v) => vals.push(v),
                        sig => return Ok(sig),
                    }
                }
                Ok(EvalSignal::Value(Value::Struct {
                    type_key,
                    fields: Rc::new(vals),
                }))
            }
            Expr::NamedStructNew {
                target,
                fields,
                span,
            } => {
                let mut map = BTreeMap::new();
                for (fname, fexpr, _) in fields {
                    let val = match eval(fexpr)? {
                        EvalSignal::Value(v) => v,
                        sig => return Ok(sig),
                    };
                    map.insert(fname.clone(), val);
                }
                let type_key = resolve_struct_key(
                    &mod_rt.resolved_named_structs,
                    &mod_rt.local_structs,
                    &mod_rt.dc_structs,
                    target,
                    *span,
                    "Unknown named struct",
                )?;
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
                    match eval(pexpr)? {
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
            Expr::OptionSome { value, .. } => match eval(value)? {
                EvalSignal::Value(v) => {
                    Ok(EvalSignal::Value(Value::Option(Some(Box::new(v)))))
                }
                sig => Ok(sig),
            },
            Expr::ResultOk { value, .. } => match eval(value)? {
                EvalSignal::Value(v) => {
                    Ok(EvalSignal::Value(Value::Result(Ok(Box::new(v)))))
                }
                sig => Ok(sig),
            },
            Expr::ResultErr { error, span, .. } => {
                let ev = match eval(error)? {
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
            Expr::Field { index, base, span } => {
                let idx_usize = usize::try_from(*index)
                    .context("Field index exceeds usize")?;
                let base_val = match eval(base)? {
                    EvalSignal::Value(v) => v,
                    sig => return Ok(sig),
                };
                match base_val {
                    Value::Tuple(slots) | Value::Struct { fields: slots, .. } => {
                        let v =
                            slots.get(idx_usize).cloned().with_context(|| {
                                format!(
                                    "Field index {index} out of bounds at runtime ({span})"
                                )
                            })?;
                        Ok(EvalSignal::Value(v))
                    }
                    other => {
                        bail!("Invalid operand {other:?} for 'field' at {span}")
                    }
                }
            }
            Expr::With {
                index,
                base,
                value,
                span,
            } => {
                let idx_usize = usize::try_from(*index)
                    .context("Field index exceeds usize")?;
                let base_val = match eval(base)? {
                    EvalSignal::Value(v) => v,
                    sig => return Ok(sig),
                };
                let repl_val = match eval(value)? {
                    EvalSignal::Value(v) => v,
                    sig => return Ok(sig),
                };
                apply_positional_with(base_val, idx_usize, *index, repl_val, *span)
            }
            Expr::NamedField { base, field, span } => {
                let bval = match eval(base)? {
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
                let bval = match eval(base)? {
                    EvalSignal::Value(v) => v,
                    sig => return Ok(sig),
                };
                let rval = match eval(value)? {
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
                let val = match eval(inner)? {
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
                let val = match eval(inner)? {
                    EvalSignal::Value(v) => v,
                    sig => return Ok(sig),
                };
                let Value::NominalWrap { inner: wrapped, .. } = val else {
                    bail!(
                        "Expected nominal wrapped value in 'unwrap', got {val:?} at {span}"
                    );
                };
                Ok(EvalSignal::Value(*wrapped))
            }
            other => {
                bail!("Unexpected non-data expression {other:?} in eval_data_expr")
            }
        }
    }

    fn lookup_module_value(
        &self,
        mod_idx: usize,
        span: Span,
        name: &str,
        ty: Option<&StageLType>,
    ) -> Result<Value> {
        let mod_rt = self.modules.get(mod_idx).context("Invalid module idx")?;
        if let Some((target_mod_idx, vkey)) =
            mod_rt.resolved_values.get(&(span.line, span.col))
        {
            let target_rt = self
                .modules
                .get(*target_mod_idx)
                .context("Invalid resolved constant module idx")?;
            let raw = target_rt.constants.get(vkey).cloned().with_context(|| {
                format!(
                    "Constant '{}/{}' not yet initialized or missing at runtime",
                    vkey.ty, vkey.name
                )
            })?;
            return if mod_rt.dialect == StageDialect::StageR3
                && target_rt.dialect == StageDialect::StageL3
            {
                marshal_stage1_val_to_stager(raw)
            } else {
                Ok(raw)
            };
        }
        if let Some(expected_ty) = ty {
            let fallback_key = ValueKey {
                name: name.to_owned(),
                ty: expected_ty.clone(),
            };
            if let Some(v) = mod_rt.constants.get(&fallback_key) {
                return Ok(v.clone());
            }
        } else if let Some((_, v)) =
            mod_rt.constants.iter().find(|(k, _)| k.name == name)
        {
            return Ok(v.clone());
        }
        bail!("Constant or variable '{name}' not found at runtime")
    }

    fn lookup_dc_value(&self, mod_idx: usize, dc: DcChar) -> Result<Value> {
        let mod_rt = self.modules.get(mod_idx).context("Invalid module idx")?;
        if let Some(vkey) = mod_rt.dc_constants.get(&dc) {
            return mod_rt
                .constants
                .get(vkey)
                .cloned()
                .context("Missing local Dc constant value");
        }
        if let Some(ResolvedDcImportTarget::Value {
            module_idx: dep_idx,
            key: vkey,
        }) = mod_rt.imports_by_dc.get(&dc)
        {
            let dep_rt = self.modules.get(*dep_idx).context("Invalid dep idx")?;
            let raw = dep_rt
                .constants
                .get(vkey)
                .cloned()
                .context("Missing imported Dc constant value")?;
            return if mod_rt.dialect == StageDialect::StageR3
                && dep_rt.dialect == StageDialect::StageL3
            {
                marshal_stage1_val_to_stager(raw)
            } else {
                Ok(raw)
            };
        }
        bail!("Dc value reference '{dc}' not found at runtime")
    }

    fn check_unmarshaled_limits(&self, val: &Value) -> Option<EvalSignal> {
        match val {
            Value::Int(n) => self.check_int_limit(n, "Stage 1 unmarshaled"),
            Value::Rat(r) => self.check_rat_limit(r, "Stage 1 unmarshaled"),
            Value::List(xs) | Value::Tuple(xs) => xs
                .iter()
                .find_map(|elem| self.check_unmarshaled_limits(elem)),
            Value::Struct { fields, .. } => fields
                .iter()
                .find_map(|field| self.check_unmarshaled_limits(field)),
            _ => None,
        }
    }

    fn resolve_callee(
        &self,
        mod_idx: usize,
        target: &SymbolTarget,
        span: Span,
        frame: &LexFrame,
    ) -> Result<CallableRoutine> {
        match target {
            SymbolTarget::Ident(name) => {
                if let Some(cv) = frame.lookup_nested_verb(name) {
                    return Ok(CallableRoutine::Nested(cv));
                }
                let mod_rt =
                    self.modules.get(mod_idx).context("Invalid module idx")?;
                if let Some((target_mod_idx, rkey)) =
                    mod_rt.resolved_calls.get(&(span.line, span.col))
                {
                    return Ok(CallableRoutine::Top {
                        module_idx: *target_mod_idx,
                        key: rkey.clone(),
                    });
                }
                bail!("Routine '{name}' at {span} not found at runtime")
            }
            SymbolTarget::Dc(dc) => {
                let mod_rt =
                    self.modules.get(mod_idx).context("Invalid module idx")?;
                if let Some(rkey) = mod_rt.dc_routines.get(dc) {
                    return Ok(CallableRoutine::Top {
                        module_idx: mod_idx,
                        key: rkey.clone(),
                    });
                }
                if let Some(ResolvedDcImportTarget::Routine {
                    module_idx: dep_idx,
                    key: rkey,
                }) = mod_rt.imports_by_dc.get(dc)
                {
                    return Ok(CallableRoutine::Top {
                        module_idx: *dep_idx,
                        key: rkey.clone(),
                    });
                }
                bail!("Dc routine reference '{dc}' not found at runtime")
            }
        }
    }
}

#[allow(
    clippy::too_many_lines,
    reason = "Resolves multi-stratum constant, call, struct, and Dc imports into ModuleRuntime"
)]
fn build_module_runtime(
    ns_to_indices: &HashMap<String, Vec<usize>>,
    prev_modules: &[ModuleRuntime],
    mod_idx: usize,
    vm: &ValidatedModule,
) -> Result<(ModuleRuntime, Vec<(ValueKey, TopValueDecl)>)> {
    let is_cross_stager_to_stage1 = |target_idx: usize| -> bool {
        vm.module.dialect == StageDialect::StageR3
            && prev_modules
                .get(target_idx)
                .is_some_and(|rt| rt.dialect == StageDialect::StageL3)
    };
    let prior_strata = |ns_name: &str| -> Result<&[usize]> {
        let candidates = ns_to_indices
            .get(ns_name)
            .with_context(|| format!("Missing compiled namespace '{ns_name}'"))?;
        Ok(candidates.as_slice())
    };

    let mut resolved_values = HashMap::with_capacity(vm.resolved_values.len());
    for (&span_key, rv) in &vm.resolved_values {
        let base_vkey = rv.value_key();
        if let Some(ns_name) = &rv.namespace {
            let mut resolved_pair = None;
            for &cand_idx in prior_strata(ns_name)? {
                if cand_idx >= mod_idx {
                    continue;
                }
                let mut eff_vkey = base_vkey.clone();
                if is_cross_stager_to_stage1(cand_idx) {
                    eff_vkey.ty = unbridge_stager_type_for_stage1(&eff_vkey.ty);
                }
                if prev_modules
                    .get(cand_idx)
                    .is_some_and(|rt| rt.constants.contains_key(&eff_vkey))
                {
                    resolved_pair = Some((cand_idx, eff_vkey));
                    break;
                }
            }
            let pair = resolved_pair.with_context(|| {
                format!(
                    "Missing resolved constant '{ns_name}::{}' across compiled strata",
                    rv.original_name
                )
            })?;
            resolved_values.insert(span_key, pair);
        } else {
            resolved_values.insert(span_key, (mod_idx, base_vkey));
        }
    }

    let mut resolved_calls = HashMap::with_capacity(vm.resolved_calls.len());
    for (&span_key, rc) in &vm.resolved_calls {
        let base_rkey = rc.routine_key();
        if let Some(ns_name) = &rc.namespace {
            let mut resolved_pair = None;
            for &cand_idx in prior_strata(ns_name)? {
                if cand_idx >= mod_idx {
                    continue;
                }
                let mut eff_rkey = base_rkey.clone();
                if is_cross_stager_to_stage1(cand_idx) {
                    unbridge_routine_key_params(&mut eff_rkey);
                }
                if prev_modules
                    .get(cand_idx)
                    .is_some_and(|rt| rt.routines.contains_key(&eff_rkey))
                {
                    resolved_pair = Some((cand_idx, eff_rkey));
                    break;
                }
            }
            let pair = resolved_pair.with_context(|| {
                format!(
                    "Missing resolved routine '{ns_name}::{}' across compiled strata",
                    rc.routine_name
                )
            })?;
            resolved_calls.insert(span_key, pair);
        } else {
            resolved_calls.insert(span_key, (mod_idx, base_rkey));
        }
    }

    let resolved_structs = vm
        .resolved_structs
        .iter()
        .map(|(&k, s)| (k, (s.namespace.clone(), s.original_name.clone())))
        .collect();
    let resolved_named_structs = vm
        .resolved_named_structs
        .iter()
        .map(|(&k, s)| (k, (s.namespace.clone(), s.original_name.clone())))
        .collect();

    let mut imports_by_dc = HashMap::new();
    for (dc, sym) in &vm.dc_imports {
        let target = match sym {
            ImportedSymbol::Value {
                namespace,
                original_name,
                ty,
                ..
            } => {
                let mut resolved_target = None;
                for &cand_idx in prior_strata(namespace)? {
                    if cand_idx >= mod_idx {
                        continue;
                    }
                    let effective_ty = if is_cross_stager_to_stage1(cand_idx) {
                        unbridge_stager_type_for_stage1(ty)
                    } else {
                        ty.clone()
                    };
                    let eff_key = ValueKey {
                        name: original_name.clone(),
                        ty: effective_ty,
                    };
                    if prev_modules.get(cand_idx).is_some_and(|rt| {
                        rt.dc_constants.contains_key(dc)
                            || rt.constants.contains_key(&eff_key)
                    }) {
                        resolved_target = Some(ResolvedDcImportTarget::Value {
                            module_idx: cand_idx,
                            key: eff_key,
                        });
                        break;
                    }
                }
                resolved_target.with_context(|| {
                    format!(
                        "Missing imported Dc constant '{namespace}::{original_name}'"
                    )
                })?
            }
            ImportedSymbol::Routine {
                namespace,
                original_name,
                sig,
            } => {
                let mut resolved_target = None;
                for &cand_idx in prior_strata(namespace)? {
                    if cand_idx >= mod_idx {
                        continue;
                    }
                    let mut rkey = RoutineKey {
                        name: original_name.clone(),
                        params: sig.params.clone(),
                    };
                    if is_cross_stager_to_stage1(cand_idx) {
                        unbridge_routine_key_params(&mut rkey);
                    }
                    if prev_modules.get(cand_idx).is_some_and(|rt| {
                        rt.dc_routines.contains_key(dc)
                            || rt.routines.contains_key(&rkey)
                    }) {
                        resolved_target =
                            Some(ResolvedDcImportTarget::Routine {
                                module_idx: cand_idx,
                                key: rkey,
                            });
                        break;
                    }
                }
                resolved_target.with_context(|| {
                    format!(
                        "Missing imported Dc routine '{namespace}::{original_name}'"
                    )
                })?
            }
            ImportedSymbol::Struct { .. }
            | ImportedSymbol::BrandType { .. }
            | ImportedSymbol::NamedStruct { .. }
            | ImportedSymbol::Enum { .. } => continue,
        };
        imports_by_dc.insert(*dc, target);
    }

    let mut routines = HashMap::new();
    let mut dc_routines = HashMap::new();
    let mut dc_constants = HashMap::new();
    let mut pending_consts = Vec::new();

    for decl in &vm.module.items {
        match decl {
            TopDecl::Comment(_)
            | TopDecl::Struct(_)
            | TopDecl::BrandType(_)
            | TopDecl::NamedStruct(_)
            | TopDecl::Enum(_) => {}
            TopDecl::Value(v) => {
                let vkey = vm.value_key(v);
                if let Some(dc) = v.dc_id {
                    dc_constants.insert(dc, vkey.clone());
                }
                pending_consts.push((vkey, v.clone()));
            }
            TopDecl::Routine(r) => {
                let rkey = vm.routine_key(r);
                if let Some(dc) = r.dc_id {
                    dc_routines.insert(dc, rkey.clone());
                }
                routines.insert(rkey, r.clone());
            }
        }
    }

    let runtime = ModuleRuntime {
        namespace_key: vm.namespace_key.clone(),
        dialect: vm.module.dialect,
        constants: HashMap::new(),
        dc_constants,
        routines,
        dc_routines,
        resolved_values,
        resolved_calls,
        resolved_structs,
        resolved_named_structs,
        resolved_enum_variants: vm.resolved_enum_variants.clone(),
        dc_structs: vm.dc_structs.clone(),
        local_structs: vm.local_structs.clone(),
        dc_nominals: vm.dc_nominals.clone(),
        local_nominals: vm.local_nominals.clone(),
        imports_by_dc,
    };
    Ok((runtime, pending_consts))
}

fn eval_pending_constants(
    vm: &mut StageLWorkspaceVm,
    mod_idx: usize,
    pending: &[(ValueKey, TopValueDecl)],
) -> Result<()> {
    let mut remaining: Vec<(ValueKey, TopValueDecl)> = pending.to_vec();
    let empty_frame = LexFrame::new(None);
    while !remaining.is_empty() {
        let before = remaining.len();
        let mut next_remaining = Vec::new();
        for (vkey, v) in remaining {
            let mut quota = vm.config.compute_quota;
            match vm.eval_expr(
                mod_idx,
                &v.value,
                &empty_frame,
                false,
                0,
                &mut quota,
            ) {
                Ok(EvalSignal::Value(val)) => {
                    if let Some(m) = vm.modules.get_mut(mod_idx) {
                        m.constants.insert(vkey, val);
                    }
                }
                Ok(EvalSignal::Fault(msg)) => {
                    bail!(
                        "Constant '{}' trapped during initialization: {msg}",
                        v.name
                    );
                }
                Ok(_) => {
                    bail!("Invalid signal evaluating constant '{}'", v.name)
                }
                Err(_) => next_remaining.push((vkey, v)),
            }
        }
        if next_remaining.len() == before {
            bail!(
                "Failed to evaluate module constants (unresolved constant dependency)"
            );
        }
        remaining = next_remaining;
    }
    Ok(())
}

fn apply_positional_with(
    base_val: Value,
    idx_usize: usize,
    index: u32,
    repl_val: Value,
    span: Span,
) -> Result<EvalSignal> {
    match base_val {
        Value::Tuple(slots) => {
            let mut next_slots = (*slots).clone();
            let slot = next_slots.get_mut(idx_usize).with_context(|| {
                format!("Field index {index} out of bounds in 'with' ({span})")
            })?;
            *slot = repl_val;
            Ok(EvalSignal::Value(Value::Tuple(Rc::new(next_slots))))
        }
        Value::Struct { type_key, fields } => {
            let mut next_fields = (*fields).clone();
            let slot = next_fields.get_mut(idx_usize).with_context(|| {
                format!("Field index {index} out of bounds in 'with' ({span})")
            })?;
            *slot = repl_val;
            Ok(EvalSignal::Value(Value::Struct {
                type_key,
                fields: Rc::new(next_fields),
            }))
        }
        other => bail!("Invalid operand {other:?} for 'with' at {span}"),
    }
}

fn resolve_struct_key(
    resolved_map: &HashMap<(usize, usize), TypeKey>,
    local_map: &HashMap<String, TypeKey>,
    dc_map: &HashMap<DcChar, TypeKey>,
    target: &SymbolTarget,
    span: Span,
    err_prefix: &str,
) -> Result<TypeKey> {
    if let Some(key) = resolved_map.get(&(span.line, span.col)) {
        return Ok(key.clone());
    }
    match target {
        SymbolTarget::Ident(name) => {
            local_map.get(name).cloned().with_context(|| {
                format!("{err_prefix} '{name}' not found at runtime at {span}")
            })
        }
        SymbolTarget::Dc(dc) => dc_map.get(dc).cloned().with_context(|| {
            format!("{err_prefix} '{dc}' not found at runtime at {span}")
        }),
    }
}

fn resolve_struct_type_ref(
    local_map: &HashMap<String, TypeKey>,
    dc_map: &HashMap<DcChar, TypeKey>,
    sref: &StructTypeRef,
    span: Span,
    kind_desc: &str,
) -> Result<TypeKey> {
    match sref {
        StructTypeRef::Canonical { namespace, name } => {
            Ok((namespace.clone(), name.clone()))
        }
        StructTypeRef::Ident(name) => {
            local_map.get(name).cloned().with_context(|| {
                format!("Unknown {kind_desc} '{name}' at {span}")
            })
        }
        StructTypeRef::Dc(dc) => dc_map.get(dc).cloned().with_context(|| {
            format!("Unknown {kind_desc} '{dc:?}' at {span}")
        }),
    }
}

fn resolve_enum_variant_key(
    mod_rt: &ModuleRuntime,
    enum_ty: &StructTypeRef,
    variant: &str,
    span: Span,
) -> Result<(TypeKey, String)> {
    if let Some(resolved) =
        mod_rt.resolved_enum_variants.get(&(span.line, span.col))
    {
        return Ok(resolved.clone());
    }
    let type_key = resolve_struct_type_ref(
        &mod_rt.local_nominals,
        &mod_rt.dc_nominals,
        enum_ty,
        span,
        "enum type",
    )?;
    Ok((type_key, variant.to_owned()))
}

fn resolve_nominal_type_key(
    mod_rt: &ModuleRuntime,
    target_ty: &StageLType,
    span: Span,
) -> Result<TypeKey> {
    match target_ty {
        StageLType::Nominal(sref) | StageLType::Struct(sref) => {
            resolve_struct_type_ref(
                &mod_rt.local_nominals,
                &mod_rt.dc_nominals,
                sref,
                span,
                "nominal type",
            )
        }
        other => bail!(
            "Invalid 'wrap' target type '{other}' (expected nominal brand or opaque type) at {span}"
        ),
    }
}

fn unbridge_routine_key_params(rkey: &mut RoutineKey) {
    for p in &mut rkey.params {
        *p = unbridge_stager_type_for_stage1(p);
    }
}

fn split_u128_fields(bits: u128) -> Result<Rc<Vec<Value>>> {
    let lo = u64::try_from(bits & u128::from(u64::MAX))?;
    let hi = u64::try_from(bits.wrapping_shr(64u32))?;
    Ok(Rc::new(vec![Value::U64(lo), Value::U64(hi)]))
}

fn join_u128_fields(fields: &[Value], type_name: &str) -> Result<u128> {
    let [Value::U64(lo), Value::U64(hi)] = fields else {
        bail!("Invalid Stage 1 carrier '{type_name}' fields: {fields:?}");
    };
    Ok(u128::from(*lo) | u128::from(*hi).wrapping_shl(64u32))
}

fn integer_to_stage1_bigint(n: &Integer) -> Value {
    let (neg, limbs) = if *n == Integer::ZERO {
        (false, Vec::new())
    } else {
        let neg = *n < Integer::ZERO;
        let limbs = n
            .unsigned_abs_ref()
            .to_power_of_2_digits_asc(32u64)
            .into_iter()
            .map(Value::U32)
            .collect();
        (neg, limbs)
    };
    Value::Struct {
        type_key: (Some("std/bigint".to_owned()), "bigInt".to_owned()),
        fields: Rc::new(vec![Value::Bool(neg), Value::List(Rc::new(limbs))]),
    }
}

fn stage1_bigint_to_integer(fields: &[Value]) -> Result<Integer> {
    let [Value::Bool(neg), Value::List(limb_vals)] = fields else {
        bail!("Invalid Stage 1 carrier 'bigInt' fields: {fields:?}");
    };
    let mut limbs = Vec::with_capacity(limb_vals.len());
    for item in limb_vals.iter() {
        let Value::U32(limb) = item else {
            bail!("Invalid non-u32 limb in Stage 1 'bigInt': {item:?}");
        };
        limbs.push(*limb);
    }
    let nat = Natural::from_power_of_2_digits_asc(32u64, limbs.into_iter())
        .context("Failed to decode Stage 1 'bigInt' limbs")?;
    if nat == Natural::ZERO {
        Ok(Integer::ZERO)
    } else {
        Ok(Integer::from_sign_and_abs(!*neg, nat))
    }
}

fn marshal_stager_val_to_stage1(val: Value) -> Result<Value> {
    match val {
        Value::I128(v) => Ok(Value::Struct {
            type_key: (Some("std/i128".to_owned()), "i128Val".to_owned()),
            fields: split_u128_fields(u128::from_le_bytes(v.to_le_bytes()))?,
        }),
        Value::U128(bits) => Ok(Value::Struct {
            type_key: (Some("std/u128".to_owned()), "u128Val".to_owned()),
            fields: split_u128_fields(bits)?,
        }),
        Value::Dc(dc) => Ok(Value::Struct {
            type_key: (Some("std/dctf".to_owned()), "dcVal".to_owned()),
            fields: split_u128_fields(dc.to_u128())?,
        }),
        Value::Int(n) => Ok(integer_to_stage1_bigint(&n)),
        Value::Rat(r) => {
            let is_neg = r < Rational::ZERO;
            let (num_nat, den_nat) = r.into_numerator_and_denominator();
            let num_int = Integer::from_sign_and_abs(!is_neg, num_nat);
            let den_int = Integer::from(den_nat);
            Ok(Value::Struct {
                type_key: (Some("std/rat".to_owned()), "ratVal".to_owned()),
                fields: Rc::new(vec![
                    integer_to_stage1_bigint(&num_int),
                    integer_to_stage1_bigint(&den_int),
                ]),
            })
        }
        Value::List(xs) => {
            let mut out = Vec::with_capacity(xs.len());
            for elem in xs.iter() {
                out.push(marshal_stager_val_to_stage1(elem.clone())?);
            }
            Ok(Value::List(Rc::new(out)))
        }
        Value::Tuple(xs) => {
            let mut out = Vec::with_capacity(xs.len());
            for elem in xs.iter() {
                out.push(marshal_stager_val_to_stage1(elem.clone())?);
            }
            Ok(Value::Tuple(Rc::new(out)))
        }
        other => Ok(other),
    }
}

fn marshal_stage1_val_to_stager(val: Value) -> Result<Value> {
    match val {
        Value::Struct { type_key, fields } => match (
            type_key.0.as_deref(),
            type_key.1.as_str(),
        ) {
            (Some("std/i128"), "i128Val") => {
                let bits = join_u128_fields(&fields, "std/i128::i128Val")?;
                Ok(Value::I128(i128::from_le_bytes(bits.to_le_bytes())))
            }
            (Some("std/u128"), "u128Val") => {
                let bits = join_u128_fields(&fields, "std/u128::u128Val")?;
                Ok(Value::U128(bits))
            }
            (Some("std/dctf"), "dcVal") => {
                let bits = join_u128_fields(&fields, "std/dctf::dcVal")?;
                Ok(Value::Dc(DcChar::from_u128(bits)))
            }
            (Some("std/bigint"), "bigInt") => {
                Ok(Value::Int(stage1_bigint_to_integer(&fields)?))
            }
            (Some("std/rat"), "ratVal") => {
                let [num_v, den_v] = fields.as_ref().as_slice() else {
                    bail!("Invalid Stage 1 carrier 'ratVal' fields");
                };
                let Value::Int(num) =
                    marshal_stage1_val_to_stager(num_v.clone())?
                else {
                    bail!("Invalid 'ratVal' numerator value");
                };
                let Value::Int(den) =
                    marshal_stage1_val_to_stager(den_v.clone())?
                else {
                    bail!("Invalid 'ratVal' denominator value");
                };
                ensure!(den != Integer::ZERO, "Zero denominator in Stage 1 'ratVal'");
                Ok(Value::Rat(Rational::from_integers(num, den)))
            }
            _ => {
                let mut out = Vec::with_capacity(fields.len());
                for field in fields.iter() {
                    out.push(marshal_stage1_val_to_stager(field.clone())?);
                }
                Ok(Value::Struct {
                    type_key,
                    fields: Rc::new(out),
                })
            }
        },
        Value::List(xs) => {
            let mut out = Vec::with_capacity(xs.len());
            for elem in xs.iter() {
                out.push(marshal_stage1_val_to_stager(elem.clone())?);
            }
            Ok(Value::List(Rc::new(out)))
        }
        Value::Tuple(xs) => {
            let mut out = Vec::with_capacity(xs.len());
            for elem in xs.iter() {
                out.push(marshal_stage1_val_to_stager(elem.clone())?);
            }
            Ok(Value::Tuple(Rc::new(out)))
        }
        other => Ok(other),
    }
}
