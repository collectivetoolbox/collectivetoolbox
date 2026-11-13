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

//! Deterministic reference evaluator and inline `test` runner for validated
//! StageL 3 (`f656` `.sl3`) modules and workspaces.
//!
//! Enforces strict left-to-right operand evaluation (short-circuiting `if`,
//! `and`, and `or`), checked vs. wrapping integer traps, mutable local `buf/T`
//! freeze tracking, shared mutable capture cells for nested verbs, bounded
//! compute quotas, and constant-stack tail-call trampolining.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::ast::{
    BlockItem, CoreBinaryOp, CoreTernaryOp, CoreUnaryOp, Expr, RoutineDecl,
    StageLType, SymbolTarget, TestExpected, TopDecl, TypedIntLit,
};
use super::parser::parse_module;
use super::validator::{
    ImportedSymbol, ModuleInput, ValidatedModule, ValidatedWorkspace,
    parse_and_validate_workspace, validate_module,
};
use ctb_formats_dcdata::DcChar;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Configuration limits for StageL 3 execution.
#[derive(Debug, Clone, Copy)]
pub struct EvalConfig {
    /// Maximum execution steps allowed before trapping with quota exhaustion.
    pub compute_quota: u64,
    /// Maximum non-tail call depth (tail calls execute in O(1) stack).
    pub max_call_depth: usize,
}

impl Default for EvalConfig {
    fn default() -> Self {
        Self {
            compute_quota: 2_000_000,
            max_call_depth: 1_024,
        }
    }
}

/// Mutable backing storage for a StageL 3 `buf/T` handle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BufStorage {
    /// Stored element values.
    pub elems: Vec<Value>,
    /// Set to `true` by `bufFreeze`; subsequent mutations trap.
    pub frozen: bool,
}

/// Runtime value in StageL 3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// Boolean (`b`).
    Bool(bool),
    /// Unsigned 8-bit integer (`u8`).
    U8(u8),
    /// Signed 32-bit integer (`i32`).
    I32(i32),
    /// Unsigned 32-bit integer (`u32`).
    U32(u32),
    /// Signed 64-bit integer (`i64`).
    I64(i64),
    /// Unsigned 64-bit integer (`u64`).
    U64(u64),
    /// Unit completion value (`void`).
    Void,
    /// Immutable value-semantics list (`l/T`).
    List(Rc<Vec<Value>>),
    /// Mutable local builder buffer handle (`buf/T`).
    Buf(Rc<RefCell<BufStorage>>),
}

impl Value {
    fn from_typed_int(lit: TypedIntLit) -> Self {
        match lit {
            TypedIntLit::U8(v) => Self::U8(v),
            TypedIntLit::I32(v) => Self::I32(v),
            TypedIntLit::U32(v) => Self::U32(v),
            TypedIntLit::I64(v) => Self::I64(v),
            TypedIntLit::U64(v) => Self::U64(v),
        }
    }

    fn as_u64_index(&self, context_msg: &str) -> Result<usize> {
        let Self::U64(n) = self else {
            bail!("{context_msg}: expected u64 index/count, found {self:?}");
        };
        usize::try_from(*n).with_context(|| {
            format!("{context_msg}: u64 index {n} exceeds host usize range")
        })
    }
}

/// Result of running a single StageL 3 routine invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoutineOutcome {
    /// Returned normally with a concrete value (or `Value::Void`).
    Returned(Value),
    /// Deterministically trapped (`fault`, arithmetic overflow, bounds, quota).
    Trapped(String),
}

/// Outcome for one inline `test` case.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestCaseReport {
    /// Namespace of the tested routine (if any).
    pub namespace: Option<String>,
    /// Routine name.
    pub routine_name: String,
    /// Optional `test` block label.
    pub block_label: Option<String>,
    /// 0-based index of the case within the test block.
    pub case_index: usize,
    /// Whether this test case passed.
    pub passed: bool,
    /// Diagnostic detail on failure (or concise note on pass).
    pub detail: String,
}

/// Aggregated report across all inline `test` cases executed.
#[derive(Debug, Clone, Default)]
pub struct TestSuiteReport {
    /// Individual test case outcomes.
    pub cases: Vec<TestCaseReport>,
    /// Number of passed cases.
    pub passed_count: usize,
    /// Number of failed cases.
    pub failed_count: usize,
}

impl TestSuiteReport {
    /// Returns an error summarizing any failed inline tests, or the total
    /// passing count when all tests passed.
    pub fn ensure_all_passed(&self) -> Result<usize> {
        if self.failed_count > 0 {
            let failures: Vec<String> = self
                .cases
                .iter()
                .filter(|c| !c.passed)
                .map(|c| {
                    // Reason for fallback: Anonymous modules display as '<module>'.
                    let ns = c.namespace.as_deref().unwrap_or("<module>");
                    // Reason for fallback: Unlabeled test blocks display as 'unlabeled'.
                    let label = c.block_label.as_deref().unwrap_or("unlabeled");
                    format!(
                        "{ns}::{} [{label} #{}]: {}",
                        c.routine_name, c.case_index, c.detail
                    )
                })
                .collect();
            bail!(
                "{} StageL 3 inline test case(s) failed:\n{}",
                self.failed_count,
                failures.join("\n")
            );
        }
        Ok(self.passed_count)
    }
}

type Cell = Rc<RefCell<Option<Value>>>;

#[derive(Debug, Clone)]
struct LexFrame {
    parent: Option<Rc<LexFrame>>,
    vars: HashMap<String, Cell>,
    nested_verbs: Rc<RefCell<HashMap<String, ClosureVerb>>>,
}

impl LexFrame {
    fn new(parent: Option<Rc<LexFrame>>) -> Self {
        Self {
            parent,
            vars: HashMap::new(),
            nested_verbs: Rc::new(RefCell::new(HashMap::new())),
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
struct ClosureVerb {
    decl: RoutineDecl,
    module_idx: usize,
    captured_frame: Rc<LexFrame>,
}

#[derive(Debug, Clone)]
enum CallableRoutine {
    Top {
        module_idx: usize,
        routine_name: String,
    },
    Nested(ClosureVerb),
}

enum EvalSignal {
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
enum ResolvedImportTarget {
    Value { module_idx: usize, name: String },
    Routine { module_idx: usize, name: String },
}

struct ModuleRuntime {
    #[allow(dead_code, reason = "Retained for diagnostics and debugging")]
    namespace_key: Option<String>,
    constants: HashMap<String, Value>,
    dc_constants: HashMap<DcChar, String>,
    routines: HashMap<String, RoutineDecl>,
    dc_routines: HashMap<DcChar, String>,
    imports_by_name: HashMap<String, ResolvedImportTarget>,
    imports_by_dc: HashMap<DcChar, ResolvedImportTarget>,
}

/// Multi-module StageL 3 runtime VM supporting cross-namespace calls, constant
/// evaluation, tail-call trampolining, and inline `test` execution.
pub struct StageLWorkspaceVm {
    modules: Vec<ModuleRuntime>,
    ns_to_idx: HashMap<String, usize>,
    config: EvalConfig,
}

impl StageLWorkspaceVm {
    /// Initializes a workspace VM from validated modules, eagerly evaluating all
    /// top-level `const` and `pub` constant initializers in topological order.
    pub fn from_validated_workspace(
        ws: &ValidatedWorkspace,
        config: EvalConfig,
    ) -> Result<Self> {
        let mut ns_to_idx = HashMap::new();
        for (idx, vm) in ws.modules.iter().enumerate() {
            if let Some(ref key) = vm.namespace_key {
                ns_to_idx.insert(key.clone(), idx);
            }
        }

        let mut vm_state = Self {
            modules: Vec::with_capacity(ws.modules.len()),
            ns_to_idx,
            config,
        };

        for (mod_idx, vm) in ws.modules.iter().enumerate() {
            let mut imports_by_name = HashMap::new();
            for (local_name, sym) in &vm.imports {
                let target = match sym {
                    ImportedSymbol::Value {
                        namespace,
                        original_name,
                        ..
                    } => {
                        let dep_idx = *vm_state
                            .ns_to_idx
                            .get(namespace)
                            .with_context(|| {
                                format!("Missing compiled namespace '{namespace}'")
                            })?;
                        ResolvedImportTarget::Value {
                            module_idx: dep_idx,
                            name: original_name.clone(),
                        }
                    }
                    ImportedSymbol::Routine {
                        namespace,
                        original_name,
                        ..
                    } => {
                        let dep_idx = *vm_state
                            .ns_to_idx
                            .get(namespace)
                            .with_context(|| {
                                format!("Missing compiled namespace '{namespace}'")
                            })?;
                        ResolvedImportTarget::Routine {
                            module_idx: dep_idx,
                            name: original_name.clone(),
                        }
                    }
                };
                imports_by_name.insert(local_name.clone(), target);
            }
            let mut imports_by_dc = HashMap::new();
            for (dc, sym) in &vm.dc_imports {
                let target = match sym {
                    ImportedSymbol::Value {
                        namespace,
                        original_name,
                        ..
                    } => {
                        let dep_idx = *vm_state
                            .ns_to_idx
                            .get(namespace)
                            .context("Missing imported dc namespace")?;
                        ResolvedImportTarget::Value {
                            module_idx: dep_idx,
                            name: original_name.clone(),
                        }
                    }
                    ImportedSymbol::Routine {
                        namespace,
                        original_name,
                        ..
                    } => {
                        let dep_idx = *vm_state
                            .ns_to_idx
                            .get(namespace)
                            .context("Missing imported dc namespace")?;
                        ResolvedImportTarget::Routine {
                            module_idx: dep_idx,
                            name: original_name.clone(),
                        }
                    }
                };
                imports_by_dc.insert(*dc, target);
            }

            let mut routines = HashMap::new();
            let mut dc_routines = HashMap::new();
            let mut dc_constants = HashMap::new();
            let mut pending_consts = Vec::new();

            for decl in &vm.module.items {
                match decl {
                    TopDecl::Comment(_) => {}
                    TopDecl::Value(v) => {
                        if let Some(dc) = v.dc_id {
                            dc_constants.insert(dc, v.name.clone());
                        }
                        pending_consts.push(v.clone());
                    }
                    TopDecl::Routine(r) => {
                        if let Some(dc) = r.dc_id {
                            dc_routines.insert(dc, r.name.clone());
                        }
                        routines.insert(r.name.clone(), r.clone());
                    }
                }
            }

            vm_state.modules.push(ModuleRuntime {
                namespace_key: vm.namespace_key.clone(),
                constants: HashMap::new(),
                dc_constants,
                routines,
                dc_routines,
                imports_by_name,
                imports_by_dc,
            });

            // Evaluate top-level constants in topological dependency order (handling
            // valid out-of-order pure references).
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
        let mut quota = self.config.compute_quota;
        self.invoke_trampoline(
            CallableRoutine::Top {
                module_idx,
                routine_name: routine_name.to_owned(),
            },
            args,
            0,
            &mut quota,
        )
    }

    /// Runs all inline `test` cases across every module in `ws`.
    pub fn run_all_tests(&self, ws: &ValidatedWorkspace) -> Result<TestSuiteReport> {
        let mut report = TestSuiteReport::default();
        for (mod_idx, vm) in ws.modules.iter().enumerate() {
            for decl in &vm.module.items {
                let TopDecl::Routine(r) = decl else {
                    continue;
                };
                for tb in &r.tests {
                    for (case_idx, tc) in tb.cases.iter().enumerate() {
                        let case_report = self.run_single_test_case(
                            mod_idx,
                            vm.namespace_key.clone(),
                            r,
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

    fn run_single_test_case(
        &self,
        mod_idx: usize,
        namespace: Option<String>,
        routine: &RoutineDecl,
        block_label: Option<String>,
        case_index: usize,
        tc: &super::ast::TestCase,
    ) -> Result<TestCaseReport> {
        let mut args = Vec::with_capacity(tc.inputs.len());
        let empty_frame = LexFrame::new(None);
        for inp in &tc.inputs {
            let mut quota = self.config.compute_quota;
            match self.eval_expr(
                mod_idx,
                inp,
                &empty_frame,
                false,
                0,
                &mut quota,
            )? {
                EvalSignal::Value(v) => args.push(v),
                EvalSignal::Fault(msg) => {
                    return Ok(TestCaseReport {
                        namespace,
                        routine_name: routine.name.clone(),
                        block_label,
                        case_index,
                        passed: false,
                        detail: format!("Evaluating test input trapped: {msg}"),
                    });
                }
                _ => bail!("Control-flow keyword disallowed in test input"),
            }
        }

        let outcome =
            self.call_routine(mod_idx, &routine.name, args)?;
        match (&tc.expected, outcome) {
            (TestExpected::Fault(_), RoutineOutcome::Trapped(msg)) => {
                Ok(TestCaseReport {
                    namespace,
                    routine_name: routine.name.clone(),
                    block_label,
                    case_index,
                    passed: true,
                    detail: format!("trapped as expected ({msg})"),
                })
            }
            (TestExpected::Fault(_), RoutineOutcome::Returned(v)) => {
                Ok(TestCaseReport {
                    namespace,
                    routine_name: routine.name.clone(),
                    block_label,
                    case_index,
                    passed: false,
                    detail: format!("expected fault trap, got {v:?}"),
                })
            }
            (TestExpected::Void(_), RoutineOutcome::Returned(Value::Void)) => {
                Ok(TestCaseReport {
                    namespace,
                    routine_name: routine.name.clone(),
                    block_label,
                    case_index,
                    passed: true,
                    detail: String::from("void ok"),
                })
            }
            (TestExpected::Void(_), other) => Ok(TestCaseReport {
                namespace,
                routine_name: routine.name.clone(),
                block_label,
                case_index,
                passed: false,
                detail: format!("expected void completion, got {other:?}"),
            }),
            (TestExpected::Expr(exp_expr), RoutineOutcome::Trapped(msg)) => {
                let _ = exp_expr;
                Ok(TestCaseReport {
                    namespace,
                    routine_name: routine.name.clone(),
                    block_label,
                    case_index,
                    passed: false,
                    detail: format!("routine trapped unexpectedly: {msg}"),
                })
            }
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
                        return Ok(TestCaseReport {
                            namespace,
                            routine_name: routine.name.clone(),
                            block_label,
                            case_index,
                            passed: false,
                            detail: format!("evaluating expected output trapped: {msg}"),
                        });
                    }
                    _ => bail!("Control-flow disallowed in test expected expr"),
                };
                if actual == expected {
                    Ok(TestCaseReport {
                        namespace,
                        routine_name: routine.name.clone(),
                        block_label,
                        case_index,
                        passed: true,
                        detail: format!("matched {expected:?}"),
                    })
                } else {
                    Ok(TestCaseReport {
                        namespace,
                        routine_name: routine.name.clone(),
                        block_label,
                        case_index,
                        passed: false,
                        detail: format!("expected {expected:?}, got {actual:?}"),
                    })
                }
            }
        }
    }

    fn consume_quota(&self, quota: &mut u64) -> std::result::Result<(), String> {
        if *quota == 0 {
            return Err(String::from("compute quota exceeded"));
        }
        *quota = quota.saturating_sub(1);
        Ok(())
    }

    fn invoke_trampoline(
        &self,
        mut callee: CallableRoutine,
        mut args: Vec<Value>,
        depth: usize,
        quota: &mut u64,
    ) -> Result<RoutineOutcome> {
        if depth > self.config.max_call_depth {
            return Ok(RoutineOutcome::Trapped(format!(
                "max non-tail call depth ({}) exceeded",
                self.config.max_call_depth
            )));
        }

        loop {
            if let Err(msg) = self.consume_quota(quota) {
                return Ok(RoutineOutcome::Trapped(msg));
            }
            let (mod_idx, decl, parent_frame) = match callee {
                CallableRoutine::Top {
                    module_idx,
                    routine_name,
                } => {
                    let mod_rt = self
                        .modules
                        .get(module_idx)
                        .context("Invalid module index during call")?;
                    let decl = mod_rt
                        .routines
                        .get(&routine_name)
                        .cloned()
                        .with_context(|| {
                            format!("Routine '{routine_name}' not found at runtime")
                        })?;
                    (module_idx, decl, None)
                }
                CallableRoutine::Nested(cv) => {
                    (cv.module_idx, cv.decl, Some(cv.captured_frame))
                }
            };

            let mut frame = LexFrame::new(parent_frame);
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

            match self.eval_expr(
                mod_idx,
                &decl.body,
                &frame,
                true,
                depth,
                quota,
            )? {
                EvalSignal::Value(v) | EvalSignal::Return(v) => {
                    return Ok(RoutineOutcome::Returned(v));
                }
                EvalSignal::Fault(msg) => {
                    return Ok(RoutineOutcome::Trapped(msg));
                }
                EvalSignal::TailCall {
                    callee: next_callee,
                    args: next_args,
                } => {
                    callee = next_callee;
                    args = next_args;
                }
                EvalSignal::Break | EvalSignal::Continue => {
                    bail!("Uncaught loop control signal exiting routine '{}'", decl.name);
                }
            }
        }
    }

    fn eval_expr(
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
                Ok(EvalSignal::Value(Value::from_typed_int(*value)))
            }
            Expr::Bytes { value, .. } => {
                let elems = value.iter().copied().map(Value::U8).collect();
                Ok(EvalSignal::Value(Value::List(Rc::new(elems))))
            }
            Expr::List { elements, .. } => {
                let mut vals = Vec::with_capacity(elements.len());
                for el in elements {
                    match self.eval_expr(mod_idx, el, frame, false, depth, quota)? {
                        EvalSignal::Value(v) => vals.push(v),
                        sig => return Ok(sig),
                    }
                }
                Ok(EvalSignal::Value(Value::List(Rc::new(vals))))
            }
            Expr::BufNew { capacity, .. } => {
                let cap_val =
                    match self.eval_expr(mod_idx, capacity, frame, false, depth, quota)? {
                        EvalSignal::Value(v) => v,
                        sig => return Ok(sig),
                    };
                let cap = cap_val.as_u64_index("bufNew")?;
                let storage = BufStorage {
                    elems: Vec::with_capacity(cap.min(65_536)),
                    frozen: false,
                };
                Ok(EvalSignal::Value(Value::Buf(Rc::new(RefCell::new(storage)))))
            }
            Expr::ValueRef { target, .. } => {
                let val = match target {
                    SymbolTarget::Ident(name) => {
                        if let Some(cell) = frame.lookup_var(name) {
                            cell.borrow()
                                .clone()
                                .context("Uninitialized variable read at runtime")?
                        } else {
                            self.lookup_module_value(mod_idx, name)?
                        }
                    }
                    SymbolTarget::Dc(dc) => {
                        self.lookup_dc_value(mod_idx, *dc)?
                    }
                };
                Ok(EvalSignal::Value(val))
            }
            Expr::Let { name, init, .. } => {
                let init_val = if let Some(init_expr) = init {
                    match self.eval_expr(mod_idx, init_expr, frame, false, depth, quota)? {
                        EvalSignal::Value(v) => Some(v),
                        sig => return Ok(sig),
                    }
                } else {
                    None
                };
                let _ = (name, init_val);
                bail!("'let' must execute inside a block frame")
            }
            Expr::Const { .. } | Expr::RoutineDecl(_) => {
                Ok(EvalSignal::Value(Value::Void))
            }
            Expr::Set { name, value, .. } => {
                let new_val =
                    match self.eval_expr(mod_idx, value, frame, false, depth, quota)? {
                        EvalSignal::Value(v) => v,
                        sig => return Ok(sig),
                    };
                let cell = frame
                    .lookup_var(name)
                    .with_context(|| format!("Variable '{name}' missing at runtime"))?;
                *cell.borrow_mut() = Some(new_val);
                Ok(EvalSignal::Value(Value::Void))
            }
            Expr::Call { target, args, .. } => {
                let mut evaluated_args = Vec::with_capacity(args.len());
                for a in args {
                    match self.eval_expr(mod_idx, a, frame, false, depth, quota)? {
                        EvalSignal::Value(v) => evaluated_args.push(v),
                        sig => return Ok(sig),
                    }
                }
                let callee = self.resolve_callee(mod_idx, target, frame)?;
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
            Expr::Block { stmts, .. } => {
                let mut block_frame = LexFrame::new(Some(Rc::new(frame.clone())));
                // Register all nested routines defined directly in this block
                // into the shared `nested_verbs` map before running statements so
                // forward and mutually recursive calls capture the block frame.
                let mut pending_routines = Vec::new();
                for item in stmts {
                    if let BlockItem::Expr(Expr::RoutineDecl(r)) = item {
                        pending_routines.push((**r).clone());
                    }
                }
                let block_rc_for_closures = Rc::new(block_frame.clone());
                {
                    let mut map = block_frame.nested_verbs.borrow_mut();
                    for r in &pending_routines {
                        map.insert(
                            r.name.clone(),
                            ClosureVerb {
                                decl: r.clone(),
                                module_idx: mod_idx,
                                captured_frame: Rc::clone(&block_rc_for_closures),
                            },
                        );
                    }
                }
                // Pre-create variable cells in `block_frame` so `ClosureVerb`
                // captures them even if `let` runs sequentially below. Wait: in
                // order for `captured_frame` to share newly added `let` cells,
                // keep `block_frame` itself mutated OR pre-register `let` / `const`
                // slots (`None`) in `block_frame.vars` right now, then update
                // their `Cell` contents sequentially!
                for item in stmts {
                    match item {
                        BlockItem::Expr(Expr::Let { name, .. })
                        | BlockItem::Expr(Expr::Const { name, .. }) => {
                            let cell = Rc::new(RefCell::new(None));
                            block_frame.vars.insert(name.clone(), cell);
                        }
                        _ => {}
                    }
                }
                // Update closures' `captured_frame` with finalized `block_frame`
                // carrying all variable cells!
                let live_frame = Rc::new(block_frame);
                {
                    let mut map = live_frame.nested_verbs.borrow_mut();
                    for cv in map.values_mut() {
                        cv.captured_frame = Rc::clone(&live_frame);
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
            Expr::If {
                cond,
                then_branch,
                else_branch,
                ..
            } => {
                let cval =
                    match self.eval_expr(mod_idx, cond, frame, false, depth, quota)? {
                        EvalSignal::Value(v) => v,
                        sig => return Ok(sig),
                    };
                let Value::Bool(b) = cval else {
                    bail!("'if' condition did not evaluate to bool");
                };
                if b {
                    self.eval_expr(
                        mod_idx,
                        then_branch,
                        frame,
                        in_tail,
                        depth,
                        quota,
                    )
                } else {
                    self.eval_expr(
                        mod_idx,
                        else_branch,
                        frame,
                        in_tail,
                        depth,
                        quota,
                    )
                }
            }
            Expr::While { cond, body, .. } => loop {
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
                    EvalSignal::Break => {
                        return Ok(EvalSignal::Value(Value::Void));
                    }
                    sig => return Ok(sig),
                }
            },
            Expr::ForEach {
                elem_name,
                list,
                body,
                ..
            } => {
                let lval =
                    match self.eval_expr(mod_idx, list, frame, false, depth, quota)? {
                        EvalSignal::Value(v) => v,
                        sig => return Ok(sig),
                    };
                let Value::List(elems) = lval else {
                    bail!("'foreach' iterable did not evaluate to l/T");
                };
                // Snapshot ensures safe iteration.
                for item in elems.iter() {
                    if let Err(msg) = self.consume_quota(quota) {
                        return Ok(EvalSignal::Fault(msg));
                    }
                    let mut iter_frame =
                        LexFrame::new(Some(Rc::new(frame.clone())));
                    iter_frame.vars.insert(
                        elem_name.clone(),
                        Rc::new(RefCell::new(Some(item.clone()))),
                    );
                    match self.eval_expr(
                        mod_idx,
                        body,
                        &iter_frame,
                        false,
                        depth,
                        quota,
                    )? {
                        EvalSignal::Value(_) | EvalSignal::Continue => {}
                        EvalSignal::Break => {
                            return Ok(EvalSignal::Value(Value::Void));
                        }
                        sig => return Ok(sig),
                    }
                }
                Ok(EvalSignal::Value(Value::Void))
            }
            Expr::Break { .. } => Ok(EvalSignal::Break),
            Expr::Continue { .. } => Ok(EvalSignal::Continue),
            Expr::Return { value, .. } => {
                if let Some(vexpr) = value {
                    match self.eval_expr(mod_idx, vexpr, frame, true, depth, quota)? {
                        EvalSignal::Value(v) => Ok(EvalSignal::Return(v)),
                        sig => Ok(sig),
                    }
                } else {
                    Ok(EvalSignal::Return(Value::Void))
                }
            }
            Expr::Fault { span } => {
                Ok(EvalSignal::Fault(format!("explicit fault trap at {span}")))
            }
            Expr::Unary { op, operand, .. } => {
                let v = match self.eval_expr(
                    mod_idx, operand, frame, false, depth, quota,
                )? {
                    EvalSignal::Value(v) => v,
                    sig => return Ok(sig),
                };
                eval_unary(*op, v)
            }
            Expr::Binary {
                op, left, right, ..
            } => {
                if matches!(op, CoreBinaryOp::And | CoreBinaryOp::Or) {
                    let lv = match self.eval_expr(
                        mod_idx, left, frame, false, depth, quota,
                    )? {
                        EvalSignal::Value(v) => v,
                        sig => return Ok(sig),
                    };
                    let Value::Bool(lb) = lv else {
                        bail!("Logical operand must be bool");
                    };
                    if *op == CoreBinaryOp::And && !lb {
                        return Ok(EvalSignal::Value(Value::Bool(false)));
                    }
                    if *op == CoreBinaryOp::Or && lb {
                        return Ok(EvalSignal::Value(Value::Bool(true)));
                    }
                    return self.eval_expr(
                        mod_idx, right, frame, false, depth, quota,
                    );
                }
                let lv = match self.eval_expr(
                    mod_idx, left, frame, false, depth, quota,
                )? {
                    EvalSignal::Value(v) => v,
                    sig => return Ok(sig),
                };
                let rv = match self.eval_expr(
                    mod_idx, right, frame, false, depth, quota,
                )? {
                    EvalSignal::Value(v) => v,
                    sig => return Ok(sig),
                };
                eval_binary(*op, lv, rv)
            }
            Expr::Ternary {
                op,
                first,
                second,
                third,
                ..
            } => {
                let v1 = match self.eval_expr(
                    mod_idx, first, frame, false, depth, quota,
                )? {
                    EvalSignal::Value(v) => v,
                    sig => return Ok(sig),
                };
                let v2 = match self.eval_expr(
                    mod_idx, second, frame, false, depth, quota,
                )? {
                    EvalSignal::Value(v) => v,
                    sig => return Ok(sig),
                };
                let v3 = match self.eval_expr(
                    mod_idx, third, frame, false, depth, quota,
                )? {
                    EvalSignal::Value(v) => v,
                    sig => return Ok(sig),
                };
                eval_ternary(*op, v1, v2, v3)
            }
            Expr::Conv {
                expr, target_ty, ..
            } => {
                let v = match self.eval_expr(
                    mod_idx, expr, frame, false, depth, quota,
                )? {
                    EvalSignal::Value(v) => v,
                    sig => return Ok(sig),
                };
                eval_conv(v, target_ty)
            }
            Expr::Bitcast {
                expr, target_ty, ..
            } => {
                let v = match self.eval_expr(
                    mod_idx, expr, frame, false, depth, quota,
                )? {
                    EvalSignal::Value(v) => v,
                    sig => return Ok(sig),
                };
                eval_bitcast(v, target_ty)
            }
        }
    }

    fn lookup_module_value(&self, mod_idx: usize, name: &str) -> Result<Value> {
        let mod_rt = self.modules.get(mod_idx).context("Invalid module idx")?;
        if let Some(v) = mod_rt.constants.get(name) {
            return Ok(v.clone());
        }
        if let Some(ResolvedImportTarget::Value {
            module_idx: dep_idx,
            name: orig,
        }) = mod_rt.imports_by_name.get(name)
        {
            let dep_rt = self.modules.get(*dep_idx).context("Invalid dep idx")?;
            return dep_rt.constants.get(orig).cloned().with_context(|| {
                format!("Missing constant '{orig}' in dependency module")
            });
        }
        bail!("Constant or variable '{name}' not found at runtime")
    }

    fn lookup_dc_value(&self, mod_idx: usize, dc: DcChar) -> Result<Value> {
        let mod_rt = self.modules.get(mod_idx).context("Invalid module idx")?;
        if let Some(name) = mod_rt.dc_constants.get(&dc) {
            return mod_rt
                .constants
                .get(name)
                .cloned()
                .context("Missing local Dc constant value");
        }
        if let Some(ResolvedImportTarget::Value {
            module_idx: dep_idx,
            name: orig,
        }) = mod_rt.imports_by_dc.get(&dc)
        {
            let dep_rt = self.modules.get(*dep_idx).context("Invalid dep idx")?;
            return dep_rt
                .constants
                .get(orig)
                .cloned()
                .context("Missing imported Dc constant value");
        }
        bail!("Dc value reference '{dc}' not found at runtime")
    }

    fn resolve_callee(
        &self,
        mod_idx: usize,
        target: &SymbolTarget,
        frame: &LexFrame,
    ) -> Result<CallableRoutine> {
        match target {
            SymbolTarget::Ident(name) => {
                if let Some(cv) = frame.lookup_nested_verb(name) {
                    return Ok(CallableRoutine::Nested(cv));
                }
                let mod_rt =
                    self.modules.get(mod_idx).context("Invalid module idx")?;
                if mod_rt.routines.contains_key(name) {
                    return Ok(CallableRoutine::Top {
                        module_idx: mod_idx,
                        routine_name: name.clone(),
                    });
                }
                if let Some(ResolvedImportTarget::Routine {
                    module_idx: dep_idx,
                    name: orig,
                }) = mod_rt.imports_by_name.get(name)
                {
                    return Ok(CallableRoutine::Top {
                        module_idx: *dep_idx,
                        routine_name: orig.clone(),
                    });
                }
                bail!("Routine '{name}' not found at runtime")
            }
            SymbolTarget::Dc(dc) => {
                let mod_rt =
                    self.modules.get(mod_idx).context("Invalid module idx")?;
                if let Some(rname) = mod_rt.dc_routines.get(dc) {
                    return Ok(CallableRoutine::Top {
                        module_idx: mod_idx,
                        routine_name: rname.clone(),
                    });
                }
                if let Some(ResolvedImportTarget::Routine {
                    module_idx: dep_idx,
                    name: orig,
                }) = mod_rt.imports_by_dc.get(dc)
                {
                    return Ok(CallableRoutine::Top {
                        module_idx: *dep_idx,
                        routine_name: orig.clone(),
                    });
                }
                bail!("Dc routine reference '{dc}' not found at runtime")
            }
        }
    }
}

fn eval_pending_constants(
    vm: &mut StageLWorkspaceVm,
    mod_idx: usize,
    pending: &[super::ast::TopValueDecl],
) -> Result<()> {
    let mut remaining: Vec<super::ast::TopValueDecl> = pending.to_vec();
    let empty_frame = LexFrame::new(None);
    while !remaining.is_empty() {
        let before = remaining.len();
        let mut next_remaining = Vec::new();
        for v in remaining {
            let mut quota = vm.config.compute_quota;
            match vm.eval_expr(mod_idx, &v.value, &empty_frame, false, 0, &mut quota) {
                Ok(EvalSignal::Value(val)) => {
                    if let Some(m) = vm.modules.get_mut(mod_idx) {
                        m.constants.insert(v.name, val);
                    }
                }
                Ok(EvalSignal::Fault(msg)) => {
                    bail!("Constant '{}' trapped during initialization: {msg}", v.name);
                }
                Ok(_) => bail!("Invalid signal evaluating constant '{}'", v.name),
                Err(_) => next_remaining.push(v),
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

/// Parses, validates, and runs all inline `test` blocks of a single `.sl3`
/// source module.
pub fn run_module_inline_tests(source: &str) -> Result<TestSuiteReport> {
    run_module_inline_tests_with_config(source, EvalConfig::default())
}

/// Parses, validates, and runs all inline `test` blocks of a single `.sl3`
/// source module with custom compute quota and depth configuration.
pub fn run_module_inline_tests_with_config(
    source: &str,
    config: EvalConfig,
) -> Result<TestSuiteReport> {
    let parsed = parse_module(source)?;
    let validated = validate_module(&parsed)?;
    let ws = ValidatedWorkspace {
        modules: vec![validated],
    };
    let vm = StageLWorkspaceVm::from_validated_workspace(&ws, config)?;
    vm.run_all_tests(&ws)
}

/// Parses, validates (with PSR-4 `ns` and `use` DAG), and runs all inline
/// `test` blocks across a multi-module workspace.
pub fn run_workspace_inline_tests(
    inputs: &[ModuleInput<'_>],
) -> Result<TestSuiteReport> {
    let ws = parse_and_validate_workspace(inputs)?;
    let vm =
        StageLWorkspaceVm::from_validated_workspace(&ws, EvalConfig::default())?;
    vm.run_all_tests(&ws)
}

fn eval_unary(op: CoreUnaryOp, val: Value) -> Result<EvalSignal> {
    match op {
        CoreUnaryOp::Neg => match val {
            Value::I32(x) => match x.checked_neg() {
                Some(r) => Ok(EvalSignal::Value(Value::I32(r))),
                None => Ok(EvalSignal::Fault(String::from("i32 neg overflow"))),
            },
            Value::I64(x) => match x.checked_neg() {
                Some(r) => Ok(EvalSignal::Value(Value::I64(r))),
                None => Ok(EvalSignal::Fault(String::from("i64 neg overflow"))),
            },
            other => bail!("Invalid operand {other:?} for 'neg'"),
        },
        CoreUnaryOp::BitNot => Ok(EvalSignal::Value(match val {
            Value::U8(x) => Value::U8(!x),
            Value::I32(x) => Value::I32(!x),
            Value::U32(x) => Value::U32(!x),
            Value::I64(x) => Value::I64(!x),
            Value::U64(x) => Value::U64(!x),
            other => bail!("Invalid operand {other:?} for 'bitNot'"),
        })),
        CoreUnaryOp::Not => match val {
            Value::Bool(b) => Ok(EvalSignal::Value(Value::Bool(!b))),
            other => bail!("Invalid operand {other:?} for 'not'"),
        },
        CoreUnaryOp::Clz => Ok(EvalSignal::Value(Value::U32(match val {
            Value::U8(x) => x.leading_zeros(),
            Value::I32(x) => x.leading_zeros(),
            Value::U32(x) => x.leading_zeros(),
            Value::I64(x) => x.leading_zeros(),
            Value::U64(x) => x.leading_zeros(),
            other => bail!("Invalid operand {other:?} for 'clz'"),
        }))),
        CoreUnaryOp::Ctz => Ok(EvalSignal::Value(Value::U32(match val {
            Value::U8(x) => x.trailing_zeros(),
            Value::I32(x) => x.trailing_zeros(),
            Value::U32(x) => x.trailing_zeros(),
            Value::I64(x) => x.trailing_zeros(),
            Value::U64(x) => x.trailing_zeros(),
            other => bail!("Invalid operand {other:?} for 'ctz'"),
        }))),
        CoreUnaryOp::Popcnt => Ok(EvalSignal::Value(Value::U32(match val {
            Value::U8(x) => x.count_ones(),
            Value::I32(x) => x.count_ones(),
            Value::U32(x) => x.count_ones(),
            Value::I64(x) => x.count_ones(),
            Value::U64(x) => x.count_ones(),
            other => bail!("Invalid operand {other:?} for 'popcnt'"),
        }))),
        CoreUnaryOp::Len => {
            let len = match val {
                Value::List(xs) => xs.len(),
                Value::Buf(b) => b.borrow().elems.len(),
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

fn eval_binary(op: CoreBinaryOp, left: Value, right: Value) -> Result<EvalSignal> {
    match op {
        CoreBinaryOp::Add => checked_bin_int("add", left, right, |a, b| a.checked_add(b), |a, b| a.checked_add(b), |a, b| a.checked_add(b), |a, b| a.checked_add(b), |a, b| a.checked_add(b)),
        CoreBinaryOp::Sub => checked_bin_int("sub", left, right, |a, b| a.checked_sub(b), |a, b| a.checked_sub(b), |a, b| a.checked_sub(b), |a, b| a.checked_sub(b), |a, b| a.checked_sub(b)),
        CoreBinaryOp::Mul => checked_bin_int("mul", left, right, |a, b| a.checked_mul(b), |a, b| a.checked_mul(b), |a, b| a.checked_mul(b), |a, b| a.checked_mul(b), |a, b| a.checked_mul(b)),
        CoreBinaryOp::Div => checked_bin_int("div", left, right, |a, b| a.checked_div(b), |a, b| a.checked_div(b), |a, b| a.checked_div(b), |a, b| a.checked_div(b), |a, b| a.checked_div(b)),
        CoreBinaryOp::Rem => checked_bin_int("rem", left, right, |a, b| a.checked_rem(b), |a, b| a.checked_rem(b), |a, b| a.checked_rem(b), |a, b| a.checked_rem(b), |a, b| a.checked_rem(b)),
        CoreBinaryOp::WrapAdd => pure_bin_int(left, right, |a, b| a.wrapping_add(b), |a, b| a.wrapping_add(b), |a, b| a.wrapping_add(b), |a, b| a.wrapping_add(b), |a, b| a.wrapping_add(b)),
        CoreBinaryOp::WrapSub => pure_bin_int(left, right, |a, b| a.wrapping_sub(b), |a, b| a.wrapping_sub(b), |a, b| a.wrapping_sub(b), |a, b| a.wrapping_sub(b), |a, b| a.wrapping_sub(b)),
        CoreBinaryOp::WrapMul => pure_bin_int(left, right, |a, b| a.wrapping_mul(b), |a, b| a.wrapping_mul(b), |a, b| a.wrapping_mul(b), |a, b| a.wrapping_mul(b), |a, b| a.wrapping_mul(b)),
        CoreBinaryOp::BitAnd => pure_bin_int(left, right, |a, b| a & b, |a, b| a & b, |a, b| a & b, |a, b| a & b, |a, b| a & b),
        CoreBinaryOp::BitOr => pure_bin_int(left, right, |a, b| a | b, |a, b| a | b, |a, b| a | b, |a, b| a | b, |a, b| a | b),
        CoreBinaryOp::BitXor => pure_bin_int(left, right, |a, b| a ^ b, |a, b| a ^ b, |a, b| a ^ b, |a, b| a ^ b, |a, b| a ^ b),
        CoreBinaryOp::Shl => eval_shift(true, left, right),
        CoreBinaryOp::Shr => eval_shift(false, left, right),
        CoreBinaryOp::Rotl => eval_rotate(true, left, right),
        CoreBinaryOp::Rotr => eval_rotate(false, left, right),
        CoreBinaryOp::Eq => Ok(EvalSignal::Value(Value::Bool(left == right))),
        CoreBinaryOp::Ne => Ok(EvalSignal::Value(Value::Bool(left != right))),
        CoreBinaryOp::Lt => cmp_int(left, right, |a, b| a < b, |a, b| a < b, |a, b| a < b, |a, b| a < b, |a, b| a < b),
        CoreBinaryOp::Le => cmp_int(left, right, |a, b| a <= b, |a, b| a <= b, |a, b| a <= b, |a, b| a <= b, |a, b| a <= b),
        CoreBinaryOp::Gt => cmp_int(left, right, |a, b| a > b, |a, b| a > b, |a, b| a > b, |a, b| a > b, |a, b| a > b),
        CoreBinaryOp::Ge => cmp_int(left, right, |a, b| a >= b, |a, b| a >= b, |a, b| a >= b, |a, b| a >= b, |a, b| a >= b),
        CoreBinaryOp::Get => {
            let idx = right.as_u64_index("get")?;
            let opt_elem = match left {
                Value::List(xs) => xs.get(idx).cloned(),
                Value::Buf(b) => b.borrow().elems.get(idx).cloned(),
                other => bail!("Invalid operand {other:?} for 'get'"),
            };
            match opt_elem {
                Some(v) => Ok(EvalSignal::Value(v)),
                None => Ok(EvalSignal::Fault(format!("get index {idx} out of bounds"))),
            }
        }
        CoreBinaryOp::Concat => {
            let (Value::List(a), Value::List(b)) = (left, right) else {
                bail!("Invalid operands for 'concat'");
            };
            let mut merged = Vec::with_capacity(a.len().saturating_add(b.len()));
            merged.extend_from_slice(&a);
            merged.extend_from_slice(&b);
            Ok(EvalSignal::Value(Value::List(Rc::new(merged))))
        }
        CoreBinaryOp::BufFill => {
            let len = right.as_u64_index("bufFill")?;
            let mut elems = Vec::with_capacity(len.min(65_536));
            elems.resize(len, left);
            Ok(EvalSignal::Value(Value::Buf(Rc::new(RefCell::new(BufStorage {
                elems,
                frozen: false,
            })))))
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
            st.elems.extend_from_slice(&xs);
            Ok(EvalSignal::Value(Value::Void))
        }
        CoreBinaryOp::And | CoreBinaryOp::Or => {
            unreachable!("Short-circuit handled directly in eval_expr")
        }
    }
}

#[allow(
    clippy::too_many_arguments,
    reason = "Parameterizes checked integer arithmetic across 5 fixed integer widths without code duplication"
)]
fn checked_bin_int(
    name: &str,
    left: Value,
    right: Value,
    f_u8: fn(u8, u8) -> Option<u8>,
    f_i32: fn(i32, i32) -> Option<i32>,
    f_u32: fn(u32, u32) -> Option<u32>,
    f_i64: fn(i64, i64) -> Option<i64>,
    f_u64: fn(u64, u64) -> Option<u64>,
) -> Result<EvalSignal> {
    let res = match (left, right) {
        (Value::U8(a), Value::U8(b)) => f_u8(a, b).map(Value::U8),
        (Value::I32(a), Value::I32(b)) => f_i32(a, b).map(Value::I32),
        (Value::U32(a), Value::U32(b)) => f_u32(a, b).map(Value::U32),
        (Value::I64(a), Value::I64(b)) => f_i64(a, b).map(Value::I64),
        (Value::U64(a), Value::U64(b)) => f_u64(a, b).map(Value::U64),
        (l, r) => bail!("Mismatched types {l:?} vs {r:?} for '{name}'"),
    };
    match res {
        Some(v) => Ok(EvalSignal::Value(v)),
        None => Ok(EvalSignal::Fault(format!("checked arithmetic fault in '{name}'"))),
    }
}

fn pure_bin_int(
    left: Value,
    right: Value,
    f_u8: fn(u8, u8) -> u8,
    f_i32: fn(i32, i32) -> i32,
    f_u32: fn(u32, u32) -> u32,
    f_i64: fn(i64, i64) -> i64,
    f_u64: fn(u64, u64) -> u64,
) -> Result<EvalSignal> {
    let val = match (left, right) {
        (Value::U8(a), Value::U8(b)) => Value::U8(f_u8(a, b)),
        (Value::I32(a), Value::I32(b)) => Value::I32(f_i32(a, b)),
        (Value::U32(a), Value::U32(b)) => Value::U32(f_u32(a, b)),
        (Value::I64(a), Value::I64(b)) => Value::I64(f_i64(a, b)),
        (Value::U64(a), Value::U64(b)) => Value::U64(f_u64(a, b)),
        (l, r) => bail!("Mismatched types {l:?} vs {r:?} for integer op"),
    };
    Ok(EvalSignal::Value(val))
}

fn cmp_int(
    left: Value,
    right: Value,
    f_u8: fn(u8, u8) -> bool,
    f_i32: fn(i32, i32) -> bool,
    f_u32: fn(u32, u32) -> bool,
    f_i64: fn(i64, i64) -> bool,
    f_u64: fn(u64, u64) -> bool,
) -> Result<EvalSignal> {
    let res = match (left, right) {
        (Value::U8(a), Value::U8(b)) => f_u8(a, b),
        (Value::I32(a), Value::I32(b)) => f_i32(a, b),
        (Value::U32(a), Value::U32(b)) => f_u32(a, b),
        (Value::I64(a), Value::I64(b)) => f_i64(a, b),
        (Value::U64(a), Value::U64(b)) => f_u64(a, b),
        (l, r) => bail!("Mismatched types {l:?} vs {r:?} for integer comparison"),
    };
    Ok(EvalSignal::Value(Value::Bool(res)))
}

fn eval_shift(is_left: bool, val: Value, count: Value) -> Result<EvalSignal> {
    let Value::U32(cnt) = count else {
        bail!("Shift count must be u32, got {count:?}");
    };
    let shifted = match val {
        Value::U8(x) => {
            if is_left {
                x.checked_shl(cnt).map(Value::U8)
            } else {
                x.checked_shr(cnt).map(Value::U8)
            }
        }
        Value::I32(x) => {
            if is_left {
                x.checked_shl(cnt).map(Value::I32)
            } else {
                x.checked_shr(cnt).map(Value::I32)
            }
        }
        Value::U32(x) => {
            if is_left {
                x.checked_shl(cnt).map(Value::U32)
            } else {
                x.checked_shr(cnt).map(Value::U32)
            }
        }
        Value::I64(x) => {
            if is_left {
                x.checked_shl(cnt).map(Value::I64)
            } else {
                x.checked_shr(cnt).map(Value::I64)
            }
        }
        Value::U64(x) => {
            if is_left {
                x.checked_shl(cnt).map(Value::U64)
            } else {
                x.checked_shr(cnt).map(Value::U64)
            }
        }
        other => bail!("Invalid operand {other:?} for shift"),
    };
    match shifted {
        Some(v) => Ok(EvalSignal::Value(v)),
        None => Ok(EvalSignal::Fault(format!("shift count {cnt} out of range"))),
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
        other => bail!("Invalid operand {other:?} for rotate"),
    };
    Ok(EvalSignal::Value(res))
}

fn eval_ternary(
    op: CoreTernaryOp,
    v1: Value,
    v2: Value,
    v3: Value,
) -> Result<EvalSignal> {
    match op {
        CoreTernaryOp::Slice => {
            let Value::List(xs) = v1 else {
                bail!("Invalid sequence operand for 'slice'");
            };
            let start = v2.as_u64_index("slice start")?;
            let end = v3.as_u64_index("slice end")?;
            if start > end || end > xs.len() {
                return Ok(EvalSignal::Fault(format!(
                    "slice bounds [{start}..{end}) out of range for len {}",
                    xs.len()
                )));
            }
            let sub = xs
                .get(start..end)
                .context("Validated slice bounds failed")?
                .to_vec();
            Ok(EvalSignal::Value(Value::List(Rc::new(sub))))
        }
        CoreTernaryOp::BufSet => {
            let Value::Buf(b) = v1 else {
                bail!("Invalid buffer operand for 'bufSet'");
            };
            let idx = v2.as_u64_index("bufSet")?;
            let mut st = b.borrow_mut();
            if st.frozen {
                return Ok(EvalSignal::Fault(String::from("bufSet on frozen buffer")));
            }
            let Some(slot) = st.elems.get_mut(idx) else {
                return Ok(EvalSignal::Fault(format!(
                    "bufSet index {idx} out of bounds"
                )));
            };
            *slot = v3;
            Ok(EvalSignal::Value(Value::Void))
        }
    }
}

fn eval_conv(val: Value, target_ty: &StageLType) -> Result<EvalSignal> {
    let v128: i128 = match val {
        Value::U8(x) => i128::from(x),
        Value::I32(x) => i128::from(x),
        Value::U32(x) => i128::from(x),
        Value::I64(x) => i128::from(x),
        Value::U64(x) => i128::from(x),
        other => bail!("Invalid source operand {other:?} for 'conv'"),
    };
    let converted = match target_ty {
        StageLType::U8 => u8::try_from(v128).ok().map(Value::U8),
        StageLType::I32 => i32::try_from(v128).ok().map(Value::I32),
        StageLType::U32 => u32::try_from(v128).ok().map(Value::U32),
        StageLType::I64 => i64::try_from(v128).ok().map(Value::I64),
        StageLType::U64 => u64::try_from(v128).ok().map(Value::U64),
        other => bail!("Invalid target type '{other}' for 'conv'"),
    };
    match converted {
        Some(v) => Ok(EvalSignal::Value(v)),
        None => Ok(EvalSignal::Fault(format!(
            "conv out of range converting {v128} to {target_ty}"
        ))),
    }
}

fn eval_bitcast(val: Value, target_ty: &StageLType) -> Result<EvalSignal> {
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
        (src, dst) => bail!("Invalid bitcast from {src:?} to {dst}"),
    };
    Ok(EvalSignal::Value(out))
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
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "Unit tests"
)]
mod tests {
    use super::*;
    use super::super::validator::{ModuleInput, parse_and_validate_workspace};

    #[crate::ctb_test]
    fn test_eval_arithmetic_bitwise_conv_bitcast_and_traps() {
        let src = r#"
v/u32/testArith
  test checked
    in.
    out u32:42.
  let u32/prod mul u32:6 u32:7.
  u32/prod.

v/u32/trapAddOverflow
  test overflow
    in.
    out fault.
  add u32:4294967295 u32:1.

v/i32/trapNegMin
  test negMin
    in.
    out fault.
  neg i32:-2147483648.

v/i32/trapDivMinMinusOne
  test divOverflow
    in.
    out fault.
  div i32:-2147483648 i32:-1.

v/i32/trapDivZero
  test divZero
    in.
    out fault.
  div i32:5 i32:0.

v/i32/trapRemMinMinusOne
  test remOverflow
    in.
    out fault.
  rem i32:-2147483648 i32:-1.

v/u32/testWrappingAndBitops
  test wraps
    in.
    out u32:80.
  let u32/wrapped wrapAdd u32:4294967295 u32:3.
  let u32/wrapped2 wrapSub u32/wrapped u32:5.
  : w = 2, w2 = u32::MAX - 2 = 0xFFFFFFFD -> popcnt = 31
  let u32/pop popcnt u32/wrapped2.
  : ctz(8) = 3, clz(16) = 27
  let u32/tz ctz u32:8.
  let u32/lz clz u32:16.
  : rotl(1, 4) = 16, rotr(16, 3) = 2
  let u32/rotLeft rotl u32:1 u32:4.
  let u32/rotRight rotr u32/rotLeft u32:3.
  : 31 + 3 + 27 + 16 + 2 + 1 = 80
  add add u32/pop u32/tz add u32/lz add u32/rotLeft add u32/rotRight u32:1.

v/i32/testSignedShiftAndBitcast
  test shifted
    in.
    out i32:-4.
  let i32/negVal i32:-16.
  : arithmetic right shift by 2 -> -4
  let i32/shiftedVal shr i32/negVal u32:2.
  : Verify bitcast round-trip preserves bit pattern
  let u32/bits bitcast i32/shiftedVal u32.
  bitcast u32/bits i32.

v/i32/trapShiftWidth
  test shiftWidth
    in.
    out fault.
  shl i32:1 u32:32.

v/u8/testConvSuccessAndTrap u32/n
  test ok
    in u32:200
    out u8:200.
  test hi
    in u32:256
    out fault.
  conv u32/n u8.
"#;
        let report = run_module_inline_tests(src).unwrap();
        let passed = report.ensure_all_passed().unwrap();
        assert_eq!(passed, 11);
    }

    #[crate::ctb_test]
    fn test_eval_buffers_captures_and_foreach_snapshots() {
        let src = r#"
v/l/u8/buildAndFreeze u8/fillByte
  test ok
    in u8:65
    out b"AAA!".
  let buf/u8/buf1 bufFill u8/fillByte u64:4.
  bufSet buf/u8/buf1 u64:3 u8:33.
  bufFreeze buf/u8/buf1.

v/u64/trapPostFreeze
  test frozenWrite
    in.
    out fault.
  let buf/u8/buf2 bufNew u8 u64:2.
  let l/u8/frozen bufFreeze buf/u8/buf2.
  bufSet buf/u8/buf2 u64:0 u8:99.
  len l/u8/frozen.

v/u32/sharedCaptureMutation
  test shared
    in.
    out u32:15.
  let u32/total u32:0.
  v/void/addToTotal u32/delta :: set u32/total add u32/total u32/delta.
  addToTotal u32:4.
  addToTotal u32:11.
  u32/total.

v/u32/foreachSnapshot
  test nestedLoops
    in.
    out u32:20.
  let l/u32/nums l/u32: u32:1 u32:2 u32:3 ::.
  let u32/sum u32:0.
  let buf/u32/dynBuf bufNew u32 u64:0.
  bufPush buf/u32/dynBuf u32:4.
  bufExtend buf/u32/dynBuf l/u32/nums.
  : dyn has [4, 1, 2, 3], frozen snapshot has sum 10
  let l/u32/snap bufFreeze buf/u32/dynBuf.
  let buf/u32/otherBuf bufNew u32 u64:0.
  foreach u32/itemX l/u32/snap do
    : Mutating another buffer while iterating snap does not disturb snap
    bufPush buf/u32/otherBuf u32:99.
    set u32/sum add u32/sum u32/itemX.
  ::.
  : Check snap again equals 10 -> total 20
  foreach u32/itemY l/u32/snap do
    set u32/sum add u32/sum u32/itemY.
  ::.
  u32/sum.
"#;
        let report = run_module_inline_tests(src).unwrap();
        let passed = report.ensure_all_passed().unwrap();
        assert_eq!(passed, 4);
    }

    #[crate::ctb_test]
    fn test_cross_module_use_alias_dc_import_and_tail_call_trampoline() {
        let inputs = [
            ModuleInput {
                source_path: Some("math/parity.sl3"),
                source: r#"ns math/parity.

pub u32/zeroConst u32:0.
pub dc/257 u32/oneConst u32:1.

pub v/b/isEven u32/n
  if eq u32/n u32/zeroConst true isOdd sub u32/n u32/oneConst.

pub v/b/isOdd u32/n
  if eq u32/n u32/zeroConst false isEven sub u32/n u32/oneConst.

pub dc/256 v/u32/incByOne u32/argVal :: add u32/argVal u32/oneConst.

: @dc l9000100
pub v/u32/doubleIt u32/argVal :: mul u32/argVal u32:2.
"#,
            },
            ModuleInput {
                source_path: Some("app/runner.sl3"),
                source: r#"ns app/runner.
use math/parity: isEven doubleIt as timesTwo dc/256 dc/257 ::.

pub v/b/checkDeepTail
  test even50k
    in.
    out true.
  isEven u32:50000.

pub v/u32/useImports u32/inputVal
  test imported
    in u32:6
    out u32:13.
  : Uses aliased 'timesTwo' AND direct Dc routine call 'dc/256' (incByOne)
  dc/256 timesTwo u32/inputVal.
"#,
            },
        ];

        let ws = parse_and_validate_workspace(&inputs)
            .expect("workspace parse and validate");

        // Verify 50,000 mutual tail calls run cleanly with max_call_depth = 32!
        let vm = StageLWorkspaceVm::from_validated_workspace(
            &ws,
            EvalConfig {
                compute_quota: 5_000_000,
                max_call_depth: 32,
            },
        )
        .expect("construct workspace VM");
        let report = vm.run_all_tests(&ws).expect("workspace inline tests");
        let passed = report.ensure_all_passed().unwrap();
        assert_eq!(passed, 2);

        // Also verify compute quota exhaustion produces a clean Fault (caught by 'out fault')
        let quota_src = r#"
v/void/spin :: spin.
v/u32/testQuota
  test hitsQuota
    in.
    out fault.
  spin.
  u32:0.
"#;
        let qreport = run_module_inline_tests_with_config(
            quota_src,
            EvalConfig {
                compute_quota: 250,
                max_call_depth: 32,
            },
        )
        .unwrap();
        assert_eq!(qreport.ensure_all_passed().unwrap(), 1);
    }
}
