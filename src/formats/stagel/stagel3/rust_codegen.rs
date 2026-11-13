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

//! Direct StageL 3 (`f656` `.sl3`) to Rust transpiler for semantic smoke and
//! static-validity cross-checking against `rustc` and `clippy`.
//!
//! Translates validated StageL 3 modules and workspaces into self-contained
//! safe Rust using only `std` primitives: exact fixed-width integer types,
//! checked/wrapping arithmetic, non-coercive conversions, stack `let mut`
//! slots for non-captured locals (enabling Rust's own definite-assignment
//! check), explicit capture cells for closures over `let` bindings, constant-
//! stack defunctionalized tail-call trampolines across self, mutual, cross-
//! module, and nested-capture calls, and an optional inline `test` runner
//! catching deterministic `fault` traps.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::ast::{
    BlockItem, CoreBinaryOp, CoreTernaryOp, CoreUnaryOp, Expr, Module,
    RoutineDecl, StageLType, SymbolTarget, TestExpected, TopDecl, TopValueDecl,
    TypedIntLit,
};
use super::parser::parse_module;
use super::validator::{
    ImportedSymbol, RoutineSig, ValidatedModule, ValidatedWorkspace,
    validate_module,
};
use ctb_formats_dcdata::DcChar;
use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;

/// Configuration options controlling StageL 3 -> Rust transpilation.
#[derive(Debug, Clone, Default)]
pub struct RustTranspileOptions {
    /// Emit crate-level `#![allow(...)]` attributes tailored for standalone
    /// smoke-test compilation (allows camelCase identifiers matching StageL 3
    /// names and intentionally unused test helpers/variables).
    pub standalone_smoke_harness: bool,
    /// Emit `pub fn run_all_sl3_tests()` and `fn main()` executing every
    /// attached inline `test` block across transpiled modules.
    pub emit_test_runner: bool,
}

/// Parses, validates, and transpiles a single standalone `.sl3` source file to
/// Rust.
pub fn transpile_sl3_source_to_rust(
    source: &str,
    options: &RustTranspileOptions,
) -> Result<String> {
    let module = parse_module(source)?;
    transpile_module_to_rust(&module, options)
}

/// Validates and transpiles a single standalone StageL 3 `Module` AST to Rust.
pub fn transpile_module_to_rust(
    module: &Module,
    options: &RustTranspileOptions,
) -> Result<String> {
    let validated = validate_module(module)?;
    transpile_validated_module_to_rust(&validated, options)
}

/// Transpiles a validated single StageL 3 module (without external `use`
/// imports) into Rust source text.
pub fn transpile_validated_module_to_rust(
    validated: &ValidatedModule,
    options: &RustTranspileOptions,
) -> Result<String> {
    ensure!(
        validated.imports.is_empty() && validated.dc_imports.is_empty(),
        "Module has external 'use' imports; use transpile_workspace_to_rust for multi-module workspaces"
    );
    let ws = ValidatedWorkspace {
        modules: vec![validated.clone()],
    };
    transpile_workspace_to_rust(&ws, options)
}

/// Transpiles an entire topologically-validated multi-module StageL 3
/// workspace into self-contained Rust source text.
pub fn transpile_workspace_to_rust(
    workspace: &ValidatedWorkspace,
    options: &RustTranspileOptions,
) -> Result<String> {
    let ws_index = WorkspaceRoutineIndex::build(workspace);
    let mut trampolines = WorkspaceTrampolineRegistry::default();
    let mut modules_buf = String::new();
    let mut test_calls = Vec::new();

    for (idx, vm) in workspace.modules.iter().enumerate() {
        let mod_ident = ws_index
            .mod_idents
            .get(idx)
            .context("Internal module index mismatch")?;
        let mut emitter = ModuleEmitter::new(
            idx,
            vm,
            mod_ident,
            &ws_index,
            &mut trampolines,
        );
        emitter.emit_module(&mut modules_buf, options.emit_test_runner)?;
        if options.emit_test_runner {
            for fn_name in emitter.emitted_test_fns {
                test_calls.push(format!("{mod_ident}::{fn_name}()"));
            }
        }
    }

    let mut out = String::new();
    emit_file_header(&mut out, options)?;
    emit_runtime_prelude(&mut out, &trampolines)?;
    trampolines.emit_trampoline_defs(&mut out)?;
    out.push_str(&modules_buf);

    if options.emit_test_runner {
        writeln!(out)?;
        writeln!(out, "/// Executes all transpiled StageL 3 inline test cases.")?;
        writeln!(out, "pub fn run_all_sl3_tests() -> usize {{")?;
        let mut total_cases = 0usize;
        for vm in &workspace.modules {
            for decl in &vm.module.items {
                if let TopDecl::Routine(r) = decl {
                    for tb in &r.tests {
                        total_cases = total_cases.saturating_add(tb.cases.len());
                    }
                }
            }
        }
        for call in &test_calls {
            writeln!(out, "    {call};")?;
        }
        writeln!(out, "    {total_cases}usize")?;
        writeln!(out, "}}")?;
        writeln!(out)?;
        writeln!(out, "fn main() {{")?;
        writeln!(out, "    let count = run_all_sl3_tests();")?;
        writeln!(out, "    println!(\"StageL 3 Rust smoke tests passed: {{count}}\");")?;
        writeln!(out, "}}")?;
    }

    Ok(out)
}

fn emit_file_header(out: &mut String, options: &RustTranspileOptions) -> Result<()> {
    writeln!(out, "// Auto-generated from StageL 3 (.sl3) source by ctb-formats-stagel.")?;
    if options.standalone_smoke_harness {
        writeln!(out, "#![allow(")?;
        writeln!(out, "    non_snake_case,")?;
        writeln!(out, "    non_upper_case_globals,")?;
        writeln!(out, "    dead_code,")?;
        writeln!(out, "    unused_variables,")?;
        writeln!(out, "    unused_imports,")?;
        writeln!(out, "    clippy::needless_late_init,")?;
        writeln!(out, "    clippy::only_used_in_recursion")?;
        writeln!(out, ")]")?;
    }
    writeln!(out)?;
    Ok(())
}

fn emit_runtime_prelude(
    out: &mut String,
    trampolines: &WorkspaceTrampolineRegistry,
) -> Result<()> {
    writeln!(out, "#[allow(dead_code)]")?;
    writeln!(out, "mod sl3_rt {{")?;
    writeln!(out, "    use std::cell::RefCell;")?;
    writeln!(out, "    use std::rc::Rc;")?;
    writeln!(out, "    use std::sync::Arc;")?;
    writeln!(out)?;
    writeln!(out, "    pub type SlList<T> = Arc<[T]>;")?;
    writeln!(out, "    pub type SlCap<T> = Rc<RefCell<Option<T>>>;")?;
    if !trampolines.value_groups.is_empty() {
        writeln!(out)?;
        writeln!(out, "    pub enum SlStep<T, C> {{")?;
        writeln!(out, "        Done(T),")?;
        writeln!(out, "        Tail(C),")?;
        if trampolines.has_value_to_never_tail {
            writeln!(out, "        TailNever(super::SlTailNever),")?;
        }
        writeln!(out, "    }}")?;
    }
    writeln!(out)?;
    writeln!(out, "    #[derive(Debug, Clone)]")?;
    writeln!(out, "    struct SlBufInner<T> {{")?;
    writeln!(out, "        elems: Vec<T>,")?;
    writeln!(out, "        frozen: bool,")?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    writeln!(out, "    #[derive(Debug, Clone)]")?;
    writeln!(out, "    pub struct SlBuf<T> {{")?;
    writeln!(out, "        inner: Rc<RefCell<SlBufInner<T>>>,")?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    writeln!(out, "    impl<T> SlBuf<T> {{")?;
    writeln!(out, "        pub fn new_with_cap(cap: u64) -> Self {{")?;
    writeln!(out, "            let Ok(c) = usize::try_from(cap) else {{")?;
    writeln!(out, "                panic!(\"bufNew capacity overflows usize\");")?;
    writeln!(out, "            }};")?;
    writeln!(out, "            Self {{")?;
    writeln!(out, "                inner: Rc::new(RefCell::new(SlBufInner {{")?;
    writeln!(out, "                    elems: Vec::with_capacity(c.min(65_536)),")?;
    writeln!(out, "                    frozen: false,")?;
    writeln!(out, "                }})),")?;
    writeln!(out, "            }}")?;
    writeln!(out, "        }}")?;
    writeln!(out)?;
    writeln!(out, "        pub fn filled(val: T, len: u64) -> Self where T: Clone {{")?;
    writeln!(out, "            let Ok(n) = usize::try_from(len) else {{")?;
    writeln!(out, "                panic!(\"bufFill length overflows usize\");")?;
    writeln!(out, "            }};")?;
    writeln!(out, "            let mut elems = Vec::with_capacity(n.min(65_536));")?;
    writeln!(out, "            elems.resize(n, val);")?;
    writeln!(out, "            Self {{")?;
    writeln!(out, "                inner: Rc::new(RefCell::new(SlBufInner {{")?;
    writeln!(out, "                    elems,")?;
    writeln!(out, "                    frozen: false,")?;
    writeln!(out, "                }})),")?;
    writeln!(out, "            }}")?;
    writeln!(out, "        }}")?;
    writeln!(out)?;
    writeln!(out, "        pub fn len(&self) -> u64 {{")?;
    writeln!(out, "            let st = self.inner.borrow();")?;
    writeln!(out, "            let Ok(n) = u64::try_from(st.elems.len()) else {{")?;
    writeln!(out, "                panic!(\"buffer len overflows u64\");")?;
    writeln!(out, "            }};")?;
    writeln!(out, "            n")?;
    writeln!(out, "        }}")?;
    writeln!(out)?;
    writeln!(out, "        pub fn get(&self, idx: u64) -> T where T: Clone {{")?;
    writeln!(out, "            let Ok(i) = usize::try_from(idx) else {{")?;
    writeln!(out, "                panic!(\"buffer get index overflows usize\");")?;
    writeln!(out, "            }};")?;
    writeln!(out, "            let st = self.inner.borrow();")?;
    writeln!(out, "            let Some(v) = st.elems.get(i) else {{")?;
    writeln!(out, "                panic!(\"buffer get index out of bounds\");")?;
    writeln!(out, "            }};")?;
    writeln!(out, "            v.clone()")?;
    writeln!(out, "        }}")?;
    writeln!(out)?;
    writeln!(out, "        pub fn push(&self, val: T) {{")?;
    writeln!(out, "            let mut st = self.inner.borrow_mut();")?;
    writeln!(out, "            if st.frozen {{")?;
    writeln!(out, "                panic!(\"bufPush on frozen buffer\");")?;
    writeln!(out, "            }}")?;
    writeln!(out, "            st.elems.push(val);")?;
    writeln!(out, "        }}")?;
    writeln!(out)?;
    writeln!(out, "        pub fn extend_list(&self, xs: &[T]) where T: Clone {{")?;
    writeln!(out, "            let mut st = self.inner.borrow_mut();")?;
    writeln!(out, "            if st.frozen {{")?;
    writeln!(out, "                panic!(\"bufExtend on frozen buffer\");")?;
    writeln!(out, "            }}")?;
    writeln!(out, "            st.elems.extend_from_slice(xs);")?;
    writeln!(out, "        }}")?;
    writeln!(out)?;
    writeln!(out, "        pub fn set(&self, idx: u64, val: T) {{")?;
    writeln!(out, "            let Ok(i) = usize::try_from(idx) else {{")?;
    writeln!(out, "                panic!(\"bufSet index overflows usize\");")?;
    writeln!(out, "            }};")?;
    writeln!(out, "            let mut st = self.inner.borrow_mut();")?;
    writeln!(out, "            if st.frozen {{")?;
    writeln!(out, "                panic!(\"bufSet on frozen buffer\");")?;
    writeln!(out, "            }}")?;
    writeln!(out, "            let Some(slot) = st.elems.get_mut(i) else {{")?;
    writeln!(out, "                panic!(\"bufSet index out of bounds\");")?;
    writeln!(out, "            }};")?;
    writeln!(out, "            *slot = val;")?;
    writeln!(out, "        }}")?;
    writeln!(out)?;
    writeln!(out, "        pub fn freeze(&self) -> SlList<T> {{")?;
    writeln!(out, "            let mut st = self.inner.borrow_mut();")?;
    writeln!(out, "            if st.frozen {{")?;
    writeln!(out, "                panic!(\"bufFreeze called on already-frozen buffer\");")?;
    writeln!(out, "            }}")?;
    writeln!(out, "            st.frozen = true;")?;
    writeln!(out, "            Arc::from(std::mem::take(&mut st.elems))")?;
    writeln!(out, "        }}")?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    writeln!(out, "    pub fn bytes_lit(bs: &[u8]) -> SlList<u8> {{")?;
    writeln!(out, "        Arc::from(bs)")?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    writeln!(out, "    pub fn list_from_vec<T>(elems: Vec<T>) -> SlList<T> {{")?;
    writeln!(out, "        Arc::from(elems)")?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    writeln!(out, "    pub fn list_len<T>(xs: &[T]) -> u64 {{")?;
    writeln!(out, "        let Ok(n) = u64::try_from(xs.len()) else {{")?;
    writeln!(out, "            panic!(\"list len overflows u64\");")?;
    writeln!(out, "        }};")?;
    writeln!(out, "        n")?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    writeln!(out, "    pub fn list_get<T: Clone>(xs: &[T], idx: u64) -> T {{")?;
    writeln!(out, "        let Ok(i) = usize::try_from(idx) else {{")?;
    writeln!(out, "            panic!(\"get index overflows usize\");")?;
    writeln!(out, "        }};")?;
    writeln!(out, "        let Some(v) = xs.get(i) else {{")?;
    writeln!(out, "            panic!(\"get index out of bounds\");")?;
    writeln!(out, "        }};")?;
    writeln!(out, "        v.clone()")?;
    writeln!(out, "    }}")?;
    writeln!(out,)?;
    writeln!(out, "    pub fn list_slice<T: Clone>(xs: &[T], start: u64, end: u64) -> SlList<T> {{")?;
    writeln!(out, "        let Ok(s) = usize::try_from(start) else {{")?;
    writeln!(out, "            panic!(\"slice start overflows usize\");")?;
    writeln!(out, "        }};")?;
    writeln!(out, "        let Ok(e) = usize::try_from(end) else {{")?;
    writeln!(out, "            panic!(\"slice end overflows usize\");")?;
    writeln!(out, "        }};")?;
    writeln!(out, "        if s > e {{")?;
    writeln!(out, "            panic!(\"slice start > end\");")?;
    writeln!(out, "        }}")?;
    writeln!(out, "        let Some(sub) = xs.get(s..e) else {{")?;
    writeln!(out, "            panic!(\"slice out of bounds\");")?;
    writeln!(out, "        }};")?;
    writeln!(out, "        Arc::from(sub)")?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    writeln!(out, "    pub fn list_concat<T: Clone>(a: &[T], b: &[T]) -> SlList<T> {{")?;
    writeln!(out, "        let mut out = Vec::with_capacity(a.len().saturating_add(b.len()));")?;
    writeln!(out, "        out.extend_from_slice(a);")?;
    writeln!(out, "        out.extend_from_slice(b);")?;
    writeln!(out, "        Arc::from(out)")?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    writeln!(out, "    pub fn cap_new<T>() -> SlCap<T> {{")?;
    writeln!(out, "        Rc::new(RefCell::new(None))")?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    writeln!(out, "    pub fn cap_new_init<T>(val: T) -> SlCap<T> {{")?;
    writeln!(out, "        Rc::new(RefCell::new(Some(val)))")?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    writeln!(out, "    pub fn cap_get<T: Clone>(cell: &SlCap<T>, name: &str) -> T {{")?;
    writeln!(out, "        let guard = cell.borrow();")?;
    writeln!(out, "        let Some(ref v) = *guard else {{")?;
    writeln!(out, "            panic!(\"captured variable '{{name}}' read before assignment\");")?;
    writeln!(out, "        }};")?;
    writeln!(out, "        v.clone()")?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    writeln!(out, "    pub fn cap_set<T>(cell: &SlCap<T>, val: T) {{")?;
    writeln!(out, "        *cell.borrow_mut() = Some(val);")?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    writeln!(out, "    pub fn expect_fault<F: FnOnce()>(f: F, desc: &str) {{")?;
    writeln!(out, "        let prev = std::panic::take_hook();")?;
    writeln!(out, "        std::panic::set_hook(Box::new(|_| {{}}));")?;
    writeln!(out, "        let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));")?;
    writeln!(out, "        std::panic::set_hook(prev);")?;
    writeln!(out, "        if res.is_ok() {{")?;
    writeln!(out, "            panic!(\"StageL test case expected fault trap but returned normally: {{desc}}\");")?;
    writeln!(out, "        }}")?;
    writeln!(out, "    }}")?;
    writeln!(out, "}}")?;
    writeln!(out)?;
    Ok(())
}

pub(super) fn sanitize_ident(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    out
}

fn rust_type(ty: &StageLType) -> String {
    match ty {
        StageLType::B => String::from("bool"),
        StageLType::U8 => String::from("u8"),
        StageLType::I32 => String::from("i32"),
        StageLType::U32 => String::from("u32"),
        StageLType::I64 => String::from("i64"),
        StageLType::U64 => String::from("u64"),
        StageLType::Void => String::from("()"),
        StageLType::Never => String::from("!"),
        StageLType::List(inner) => format!("sl3_rt::SlList<{}>", rust_type(inner)),
        StageLType::Buf(inner) => format!("sl3_rt::SlBuf<{}>", rust_type(inner)),
    }
}

fn rust_type_copy_cheap(ty: &StageLType) -> bool {
    matches!(
        ty,
        StageLType::B
            | StageLType::U8
            | StageLType::I32
            | StageLType::U32
            | StageLType::I64
            | StageLType::U64
            | StageLType::Void
    )
}

const MAX_INLINE_VARIANT_FIELDS: usize = 8;

fn tail_variant_ident(idx: usize) -> String {
    let (prefix, suffix) = match idx & 7 {
        0 => ("Alpha", "P"),
        1 => ("Bravo", "Q"),
        2 => ("Charlie", "R"),
        3 => ("Delta", "S"),
        4 => ("Echo", "T"),
        5 => ("Foxtrot", "U"),
        6 => ("Golf", "V"),
        _ => ("Hotel", "W"),
    };
    format!("{prefix}{idx}{suffix}")
}

fn format_variant_ctor(
    enum_path: &str,
    variant_ident: &str,
    rendered_args: &[String],
    boxed_struct_ident: Option<&str>,
) -> String {
    if rendered_args.is_empty() {
        format!("{enum_path}::{variant_ident}")
    } else if let Some(struct_ident) = boxed_struct_ident {
        let fields: Vec<String> = rendered_args
            .iter()
            .enumerate()
            .map(|(i, val)| format!("f{i}: {val}"))
            .collect();
        format!(
            "{enum_path}::{variant_ident}(Box::new(super::{struct_ident} {{ {} }}))",
            fields.join(", ")
        )
    } else {
        format!(
            "{enum_path}::{variant_ident}({})",
            rendered_args.join(", ")
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum CalleeKey {
    TopLevel { mod_idx: usize, routine_idx: usize },
    Nested { mod_idx: usize, nested_id: usize },
}

#[derive(Debug, Clone)]
struct TailVariantInfo {
    variant_ident: String,
    field_rust_types: Vec<String>,
    trampoline_call_path: String,
    callee_is_step: bool,
    boxed_struct_ident: Option<String>,
}

#[derive(Debug, Clone)]
struct ValueTailGroup {
    group_id: usize,
    ret_rust_ty: String,
    variants: Vec<TailVariantInfo>,
    by_callee: HashMap<CalleeKey, usize>,
}

#[derive(Debug, Default)]
struct WorkspaceTrampolineRegistry {
    value_groups: Vec<ValueTailGroup>,
    group_by_rust_ty: HashMap<String, usize>,
    has_never_trampoline: bool,
    has_value_to_never_tail: bool,
    never_variants: Vec<TailVariantInfo>,
    never_by_callee: HashMap<CalleeKey, usize>,
}

impl WorkspaceTrampolineRegistry {
    fn ensure_value_group(&mut self, ret_ty: &StageLType) -> usize {
        let ret_rust_ty = rust_type(ret_ty);
        if let Some(&gid) = self.group_by_rust_ty.get(&ret_rust_ty) {
            return gid;
        }
        let gid = self.value_groups.len();
        self.group_by_rust_ty.insert(ret_rust_ty.clone(), gid);
        self.value_groups.push(ValueTailGroup {
            group_id: gid,
            ret_rust_ty,
            variants: Vec::new(),
            by_callee: HashMap::new(),
        });
        gid
    }

    fn ensure_never_group(&mut self) {
        self.has_never_trampoline = true;
    }

    fn register_value_target(
        &mut self,
        ret_ty: &StageLType,
        callee_key: CalleeKey,
        trampoline_call_path: String,
        callee_is_step: bool,
        field_rust_types: Vec<String>,
    ) -> Result<(usize, String, Option<String>)> {
        let gid = self.ensure_value_group(ret_ty);
        let group = self
            .value_groups
            .get_mut(gid)
            .context("Internal trampoline value group index out of bounds")?;
        if let Some(&v_idx) = group.by_callee.get(&callee_key) {
            let existing = group
                .variants
                .get(v_idx)
                .context("Internal trampoline variant index out of bounds")?;
            return Ok((
                gid,
                existing.variant_ident.clone(),
                existing.boxed_struct_ident.clone(),
            ));
        }
        let v_idx = group.variants.len();
        let variant_ident = tail_variant_ident(v_idx);
        let boxed_struct_ident = if field_rust_types.len() > MAX_INLINE_VARIANT_FIELDS {
            Some(format!("SlTailPayloadG{gid}V{v_idx}"))
        } else {
            None
        };
        group.by_callee.insert(callee_key, v_idx);
        group.variants.push(TailVariantInfo {
            variant_ident: variant_ident.clone(),
            field_rust_types,
            trampoline_call_path,
            callee_is_step,
            boxed_struct_ident: boxed_struct_ident.clone(),
        });
        Ok((gid, variant_ident, boxed_struct_ident))
    }

    fn register_never_target(
        &mut self,
        callee_key: CalleeKey,
        trampoline_call_path: String,
        callee_is_step: bool,
        field_rust_types: Vec<String>,
    ) -> Result<(String, Option<String>)> {
        self.has_never_trampoline = true;
        if let Some(&v_idx) = self.never_by_callee.get(&callee_key) {
            let existing = self
                .never_variants
                .get(v_idx)
                .context("Internal never-trampoline variant index out of bounds")?;
            return Ok((
                existing.variant_ident.clone(),
                existing.boxed_struct_ident.clone(),
            ));
        }
        let v_idx = self.never_variants.len();
        let variant_ident = tail_variant_ident(v_idx);
        let boxed_struct_ident = if field_rust_types.len() > MAX_INLINE_VARIANT_FIELDS {
            Some(format!("SlTailPayloadNeverV{v_idx}"))
        } else {
            None
        };
        self.never_by_callee.insert(callee_key, v_idx);
        self.never_variants.push(TailVariantInfo {
            variant_ident: variant_ident.clone(),
            field_rust_types,
            trampoline_call_path,
            callee_is_step,
            boxed_struct_ident: boxed_struct_ident.clone(),
        });
        Ok((variant_ident, boxed_struct_ident))
    }

    fn emit_trampoline_defs(&self, out: &mut String) -> Result<()> {
        if self.has_never_trampoline {
            emit_enum_decl(out, "SlTailNever", &self.never_variants)?;
            let any_step = self.never_variants.iter().any(|v| v.callee_is_step);
            if self.never_variants.is_empty() {
                writeln!(out, "fn sl_trampoline_never(call: SlTailNever) -> ! {{")?;
                writeln!(out, "    match call {{}}")?;
                writeln!(out, "}}")?;
            } else if !any_step {
                writeln!(out, "fn sl_trampoline_never(call: SlTailNever) -> ! {{")?;
                writeln!(out, "    match call {{")?;
                for v in &self.never_variants {
                    emit_variant_dispatch_arm(out, "SlTailNever", v, false, "        ")?;
                }
                writeln!(out, "    }}")?;
                writeln!(out, "}}")?;
            } else {
                writeln!(out, "fn sl_trampoline_never(mut call: SlTailNever) -> ! {{")?;
                writeln!(out, "    loop {{")?;
                writeln!(out, "        call = match call {{")?;
                for v in &self.never_variants {
                    emit_variant_dispatch_arm(
                        out,
                        "SlTailNever",
                        v,
                        false,
                        "            ",
                    )?;
                }
                writeln!(out, "        }};")?;
                writeln!(out, "    }}")?;
                writeln!(out, "}}")?;
            }
            writeln!(out)?;
        }

        for group in &self.value_groups {
            let enum_name = format!("SlTailGroup{}", group.group_id);
            let fn_name = format!("sl_trampoline_{}", group.group_id);
            let ret_ty = &group.ret_rust_ty;
            let is_unit = ret_ty == "()";
            let ret_sig = if is_unit {
                String::new()
            } else {
                format!(" -> {ret_ty}")
            };
            emit_enum_decl(out, &enum_name, &group.variants)?;
            if group.variants.is_empty() {
                writeln!(
                    out,
                    "fn {fn_name}(step: sl3_rt::SlStep<{ret_ty}, {enum_name}>){ret_sig} {{"
                )?;
                writeln!(out, "    match step {{")?;
                if is_unit {
                    writeln!(out, "        sl3_rt::SlStep::Done(()) => {{}}")?;
                } else {
                    writeln!(out, "        sl3_rt::SlStep::Done(val) => val,")?;
                }
                writeln!(out, "        sl3_rt::SlStep::Tail(call) => match call {{}},")?;
                if self.has_value_to_never_tail {
                    writeln!(
                        out,
                        "        sl3_rt::SlStep::TailNever(ncall) => sl_trampoline_never(ncall),"
                    )?;
                }
                writeln!(out, "    }}")?;
                writeln!(out, "}}")?;
            } else {
                writeln!(
                    out,
                    "fn {fn_name}(mut step: sl3_rt::SlStep<{ret_ty}, {enum_name}>){ret_sig} {{"
                )?;
                writeln!(out, "    loop {{")?;
                writeln!(out, "        step = match step {{")?;
                if is_unit {
                    writeln!(out, "            sl3_rt::SlStep::Done(()) => return,")?;
                } else {
                    writeln!(out, "            sl3_rt::SlStep::Done(val) => return val,")?;
                }
                writeln!(out, "            sl3_rt::SlStep::Tail(call) => match call {{")?;
                for v in &group.variants {
                    emit_variant_dispatch_arm(
                        out,
                        &enum_name,
                        v,
                        !v.callee_is_step,
                        "                ",
                    )?;
                }
                writeln!(out, "            }},")?;
                if self.has_value_to_never_tail {
                    writeln!(
                        out,
                        "            sl3_rt::SlStep::TailNever(ncall) => sl_trampoline_never(ncall),"
                    )?;
                }
                writeln!(out, "        }};")?;
                writeln!(out, "    }}")?;
                writeln!(out, "}}")?;
            }
            writeln!(out)?;
        }

        Ok(())
    }
}

fn emit_enum_decl(
    out: &mut String,
    enum_name: &str,
    variants: &[TailVariantInfo],
) -> Result<()> {
    for v in variants {
        if let Some(ref struct_ident) = v.boxed_struct_ident {
            writeln!(out, "struct {struct_ident} {{")?;
            for (i, fty) in v.field_rust_types.iter().enumerate() {
                writeln!(out, "    f{i}: {fty},")?;
            }
            writeln!(out, "}}")?;
        }
    }
    if variants.is_empty() {
        writeln!(out, "enum {enum_name} {{}}")?;
        return Ok(());
    }
    writeln!(out, "enum {enum_name} {{")?;
    for v in variants {
        if v.field_rust_types.is_empty() {
            writeln!(out, "    {},", v.variant_ident)?;
        } else if let Some(ref struct_ident) = v.boxed_struct_ident {
            writeln!(out, "    {}(Box<{struct_ident}>),", v.variant_ident)?;
        } else {
            writeln!(
                out,
                "    {}({}),",
                v.variant_ident,
                v.field_rust_types.join(", ")
            )?;
        }
    }
    writeln!(out, "}}")?;
    Ok(())
}

fn emit_variant_dispatch_arm(
    out: &mut String,
    enum_name: &str,
    v: &TailVariantInfo,
    wrap_done: bool,
    pad: &str,
) -> Result<()> {
    let arity = v.field_rust_types.len();
    let arg_names: Vec<String> = (0..arity).map(|i| format!("a{i}")).collect();
    let joined = arg_names.join(", ");
    let raw_call = format!("{}({joined})", v.trampoline_call_path);
    let rhs = if wrap_done {
        format!("sl3_rt::SlStep::Done({raw_call})")
    } else {
        raw_call
    };
    if arity == 0 {
        writeln!(out, "{pad}{enum_name}::{} => {rhs},", v.variant_ident)?;
    } else if let Some(ref struct_ident) = v.boxed_struct_ident {
        let bindings: Vec<String> = (0..arity).map(|i| format!("f{i}: a{i}")).collect();
        writeln!(out, "{pad}{enum_name}::{}(boxed_args) => {{", v.variant_ident)?;
        writeln!(
            out,
            "{pad}    let {struct_ident} {{ {} }} = *boxed_args;",
            bindings.join(", ")
        )?;
        writeln!(out, "{pad}    {rhs}")?;
        writeln!(out, "{pad}}}")?;
    } else {
        writeln!(
            out,
            "{pad}{enum_name}::{}({joined}) => {rhs},",
            v.variant_ident
        )?;
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct TopRoutineMeta {
    mod_idx: usize,
    routine_idx: usize,
    mod_ident: String,
    entry_fn: String,
    step_fn: String,
    is_step: bool,
    sig: RoutineSig,
}

struct WorkspaceRoutineIndex {
    mod_idents: Vec<String>,
    ns_to_mod_idx: HashMap<String, usize>,
    by_name: HashMap<(usize, String), TopRoutineMeta>,
    by_dc: HashMap<(usize, DcChar), TopRoutineMeta>,
}

impl WorkspaceRoutineIndex {
    fn build(workspace: &ValidatedWorkspace) -> Self {
        let mut mod_idents = Vec::with_capacity(workspace.modules.len());
        let mut ns_to_mod_idx = HashMap::new();
        let mut by_name = HashMap::new();
        let mut by_dc = HashMap::new();

        for (mod_idx, vm) in workspace.modules.iter().enumerate() {
            let mod_ident = if let Some(ns) = &vm.namespace_key {
                ns_to_mod_idx.insert(ns.clone(), mod_idx);
                format!("sl_mod_{mod_idx}_{}", sanitize_ident(ns))
            } else {
                format!("sl_mod_anon_{mod_idx}")
            };

            let mut routine_idx = 0usize;
            for decl in &vm.module.items {
                if let TopDecl::Routine(r) = decl {
                    let san = sanitize_ident(&r.name);
                    let entry_fn = format!("sl_fn_{san}");
                    let step_fn = format!("__sl_step_top_{routine_idx}_{san}");
                    let is_step = expr_has_tail_call(&r.body, true);
                    let sig = RoutineSig {
                        return_type: r.return_type.clone(),
                        params: r.params.iter().map(|p| p.ty.clone()).collect(),
                        is_pub: r.is_pub,
                        dc_id: r.dc_id,
                    };
                    let meta = TopRoutineMeta {
                        mod_idx,
                        routine_idx,
                        mod_ident: mod_ident.clone(),
                        entry_fn,
                        step_fn,
                        is_step,
                        sig,
                    };
                    by_name.insert((mod_idx, r.name.clone()), meta.clone());
                    if let Some(dc) = r.dc_id {
                        by_dc.insert((mod_idx, dc), meta);
                    }
                    routine_idx = routine_idx.saturating_add(1);
                }
            }
            mod_idents.push(mod_ident);
        }

        Self {
            mod_idents,
            ns_to_mod_idx,
            by_name,
            by_dc,
        }
    }

    fn mod_ident_by_ns(&self, namespace: &str) -> Result<&str> {
        let &idx = self
            .ns_to_mod_idx
            .get(namespace)
            .with_context(|| format!("Unknown module namespace '{namespace}'"))?;
        let ident = self
            .mod_idents
            .get(idx)
            .context("Internal module index out of bounds")?;
        Ok(ident.as_str())
    }
}

#[derive(Debug, Clone)]
enum VarStorage {
    /// Owned value (`let x` / param / foreach / non-captured local `const` /
    /// immutable lexical value captured by a nested routine).
    Direct { rust_name: String, ty: StageLType },
    /// Heap `SlCap<T>` (`Rc<RefCell<Option<T>>>`) shared across nested captures.
    CapCell { rust_name: String, ty: StageLType },
}

impl VarStorage {
    fn ty(&self) -> &StageLType {
        match self {
            Self::Direct { ty, .. } | Self::CapCell { ty, .. } => ty,
        }
    }

    fn read_expr(&self, sl_name: &str) -> String {
        match self {
            Self::Direct { rust_name, ty } => {
                if rust_type_copy_cheap(ty) {
                    rust_name.clone()
                } else {
                    format!("{rust_name}.clone()")
                }
            }
            Self::CapCell { rust_name, .. } => {
                format!("sl3_rt::cap_get(&{rust_name}, {sl_name:?})")
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CapturePassKind {
    CapCell,
    ImmVal,
}

#[derive(Debug, Clone)]
struct CapturedArg {
    sl_name: String,
    rust_param: String,
    ty: StageLType,
    kind: CapturePassKind,
}

impl CapturedArg {
    fn field_rust_type(&self) -> String {
        let rty = rust_type(&self.ty);
        match self.kind {
            CapturePassKind::CapCell => format!("sl3_rt::SlCap<{rty}>"),
            CapturePassKind::ImmVal => rty,
        }
    }
}

#[derive(Debug, Clone)]
enum RoutineTarget {
    TopLevel {
        callee_key: CalleeKey,
        direct_call_path: String,
        trampoline_call_path: String,
        is_step: bool,
        sig: RoutineSig,
    },
    Nested {
        callee_key: CalleeKey,
        local_fn_name: String,
        trampoline_call_path: String,
        step_group_id: Option<usize>,
        is_step: bool,
        sig: RoutineSig,
        captures: Vec<CapturedArg>,
    },
}

struct ScopeFrame {
    vars: HashMap<String, VarStorage>,
    routines: HashMap<String, RoutineTarget>,
}

#[derive(Debug, Clone)]
enum ActiveRoutineMode {
    Direct,
    Step {
        ret_ty: StageLType,
        /// `None` iff `ret_ty == StageLType::Never`.
        group_id: Option<usize>,
    },
}

struct ActiveRoutineCtx {
    mode: ActiveRoutineMode,
    var_scope_floor: usize,
}

struct ModuleEmitter<'a> {
    mod_idx: usize,
    validated: &'a ValidatedModule,
    mod_ident: &'a str,
    ws_index: &'a WorkspaceRoutineIndex,
    trampolines: &'a mut WorkspaceTrampolineRegistry,
    local_consts: HashMap<String, (String, StageLType)>,
    local_dc_consts: HashMap<DcChar, (String, StageLType)>,
    scopes: Vec<ScopeFrame>,
    routine_ctx_stack: Vec<ActiveRoutineCtx>,
    lifted_routines: Vec<String>,
    temp_counter: usize,
    nested_routine_counter: usize,
    emitted_test_fns: Vec<String>,
}

impl<'a> ModuleEmitter<'a> {
    fn new(
        mod_idx: usize,
        validated: &'a ValidatedModule,
        mod_ident: &'a str,
        ws_index: &'a WorkspaceRoutineIndex,
        trampolines: &'a mut WorkspaceTrampolineRegistry,
    ) -> Self {
        let mut local_consts = HashMap::new();
        let mut local_dc_consts = HashMap::new();

        for decl in &validated.module.items {
            if let TopDecl::Value(v) = decl {
                let rname = format!("sl_c_{}", sanitize_ident(&v.name));
                local_consts.insert(v.name.clone(), (rname.clone(), v.ty.clone()));
                if let Some(dc) = v.dc_id {
                    local_dc_consts.insert(dc, (rname, v.ty.clone()));
                }
            }
        }

        Self {
            mod_idx,
            validated,
            mod_ident,
            ws_index,
            trampolines,
            local_consts,
            local_dc_consts,
            scopes: Vec::new(),
            routine_ctx_stack: Vec::new(),
            lifted_routines: Vec::new(),
            temp_counter: 0,
            nested_routine_counter: 0,
            emitted_test_fns: Vec::new(),
        }
    }

    fn next_tmp(&mut self, prefix: &str) -> String {
        let id = self.temp_counter;
        self.temp_counter = self.temp_counter.saturating_add(1);
        format!("__sl_{prefix}_{id}")
    }

    fn next_nested_id(&mut self) -> usize {
        let id = self.nested_routine_counter;
        self.nested_routine_counter = self.nested_routine_counter.saturating_add(1);
        id
    }

    fn current_routine_mode(&self) -> ActiveRoutineMode {
        self.routine_ctx_stack
            .last()
            .map_or(ActiveRoutineMode::Direct, |ctx| ctx.mode.clone())
    }

    fn current_var_scope_floor(&self) -> usize {
        self.routine_ctx_stack
            .last()
            .map_or(0, |ctx| ctx.var_scope_floor)
    }

    fn emit_module(&mut self, out: &mut String, emit_tests: bool) -> Result<()> {
        if let Some(ns) = &self.validated.namespace_key {
            writeln!(out, "// Module namespace: {ns}")?;
        }
        writeln!(out, "pub mod {} {{", self.mod_ident)?;
        writeln!(out, "    use super::sl3_rt;")?;
        writeln!(out)?;

        // Emit top-level values (topologically sorted so forward constant
        // references initialize in valid Rust static order).
        let sorted_vals = topo_sort_top_values(&self.validated.module.items);
        for v in &sorted_vals {
            self.emit_top_value(out, v)?;
        }

        // Emit top-level routines.
        let mut routine_idx = 0usize;
        for decl in &self.validated.module.items {
            if let TopDecl::Routine(r) = decl {
                self.emit_top_routine(out, routine_idx, r)?;
                routine_idx = routine_idx.saturating_add(1);
            }
        }

        // Emit inline test runners cleanly inside module so private routines
        // and private constants remain directly callable by their own tests.
        if emit_tests {
            for decl in &self.validated.module.items {
                if let TopDecl::Routine(r) = decl {
                    self.emit_routine_inline_tests(out, r)?;
                }
            }
        }

        for lifted in &self.lifted_routines {
            out.push_str(lifted);
        }

        writeln!(out, "}}")?;
        Ok(())
    }

    fn emit_top_value(&mut self, out: &mut String, v: &TopValueDecl) -> Result<()> {
        let vis = if v.is_pub { "pub " } else { "" };
        let rname = format!("sl_c_{}", sanitize_ident(&v.name));
        let rty = rust_type(&v.ty);
        let (init_code, _init_ty) = self.emit_expr(&v.value, 2)?;
        writeln!(
            out,
            "    {vis}static {rname}: std::sync::LazyLock<{rty}> = std::sync::LazyLock::new(|| {init_code});"
        )?;
        Ok(())
    }

    fn emit_top_routine(
        &mut self,
        out: &mut String,
        routine_idx: usize,
        r: &RoutineDecl,
    ) -> Result<()> {
        let vis = if r.is_pub { "pub " } else { "pub(super) " };
        let san = sanitize_ident(&r.name);
        let entry_name = format!("sl_fn_{san}");
        let meta = self
            .ws_index
            .by_name
            .get(&(self.mod_idx, r.name.clone()))
            .context("Internal top-level routine metadata missing")?;

        if !meta.is_step {
            return self.emit_routine_fn(
                out,
                vis,
                &entry_name,
                r,
                &[],
                ActiveRoutineMode::Direct,
                1,
            );
        }

        let step_name = format!("__sl_step_top_{routine_idx}_{san}");
        let group_id = if r.return_type == StageLType::Never {
            self.trampolines.ensure_never_group();
            None
        } else {
            Some(self.trampolines.ensure_value_group(&r.return_type))
        };

        self.emit_routine_fn(
            out,
            "pub(super) ",
            &step_name,
            r,
            &[],
            ActiveRoutineMode::Step {
                ret_ty: r.return_type.clone(),
                group_id,
            },
            1,
        )?;

        let mut sig_params = Vec::with_capacity(r.params.len());
        let mut call_args = Vec::with_capacity(r.params.len());
        for p in &r.params {
            let vp = format!("sl_v_{}", sanitize_ident(&p.name));
            sig_params.push(format!("{vp}: {}", rust_type(&p.ty)));
            call_args.push(vp);
        }
        if sig_params.len() > 7 {
            writeln!(out, "    #[allow(clippy::too_many_arguments)]")?;
        }
        if r.return_type == StageLType::Void {
            writeln!(out, "    {vis}fn {entry_name}({}) {{", sig_params.join(", "))?;
        } else {
            writeln!(
                out,
                "    {vis}fn {entry_name}({}) -> {} {{",
                sig_params.join(", "),
                rust_type(&r.return_type)
            )?;
        }
        let step_call = format!("{step_name}({})", call_args.join(", "));
        if let Some(gid) = group_id {
            writeln!(out, "        super::sl_trampoline_{gid}({step_call})")?;
        } else {
            writeln!(out, "        super::sl_trampoline_never({step_call})")?;
        }
        writeln!(out, "    }}")?;
        writeln!(out)?;
        Ok(())
    }

    fn emit_routine_fn(
        &mut self,
        out: &mut String,
        vis: &str,
        rust_name: &str,
        r: &RoutineDecl,
        extra_captures: &[CapturedArg],
        mode: ActiveRoutineMode,
        indent_level: usize,
    ) -> Result<()> {
        let ind = "    ".repeat(indent_level);
        let mut param_strs = Vec::new();
        let mut frame = ScopeFrame {
            vars: HashMap::new(),
            routines: HashMap::new(),
        };

        for p in &r.params {
            let vp = format!("sl_v_{}", sanitize_ident(&p.name));
            let rty = rust_type(&p.ty);
            param_strs.push(format!("{vp}: {rty}"));
            frame.vars.insert(
                p.name.clone(),
                VarStorage::Direct {
                    rust_name: vp,
                    ty: p.ty.clone(),
                },
            );
        }

        for cap in extra_captures {
            let rty = rust_type(&cap.ty);
            match cap.kind {
                CapturePassKind::CapCell => {
                    param_strs.push(format!("{}: sl3_rt::SlCap<{rty}>", cap.rust_param));
                    frame.vars.insert(
                        cap.sl_name.clone(),
                        VarStorage::CapCell {
                            rust_name: cap.rust_param.clone(),
                            ty: cap.ty.clone(),
                        },
                    );
                }
                CapturePassKind::ImmVal => {
                    param_strs.push(format!("{}: {rty}", cap.rust_param));
                    frame.vars.insert(
                        cap.sl_name.clone(),
                        VarStorage::Direct {
                            rust_name: cap.rust_param.clone(),
                            ty: cap.ty.clone(),
                        },
                    );
                }
            }
        }

        if !extra_captures.is_empty() {
            writeln!(out, "{ind}#[allow(clippy::only_used_in_recursion)]")?;
        }
        if param_strs.len() > 7 {
            writeln!(out, "{ind}#[allow(clippy::too_many_arguments)]")?;
        }
        match &mode {
            ActiveRoutineMode::Direct => {
                if r.return_type == StageLType::Void {
                    writeln!(out, "{ind}{vis}fn {rust_name}({}) {{", param_strs.join(", "))?;
                } else {
                    let ret_ty_str = rust_type(&r.return_type);
                    writeln!(
                        out,
                        "{ind}{vis}fn {rust_name}({}) -> {ret_ty_str} {{",
                        param_strs.join(", ")
                    )?;
                }
            }
            ActiveRoutineMode::Step { ret_ty, group_id } => {
                let step_ret_str = if let Some(gid) = group_id {
                    format!(
                        "sl3_rt::SlStep<{}, super::SlTailGroup{gid}>",
                        rust_type(ret_ty)
                    )
                } else {
                    String::from("super::SlTailNever")
                };
                writeln!(
                    out,
                    "{ind}{vis}fn {rust_name}({}) -> {step_ret_str} {{",
                    param_strs.join(", ")
                )?;
            }
        }

        let var_scope_floor = self.scopes.len();
        self.scopes.push(frame);
        self.routine_ctx_stack.push(ActiveRoutineCtx {
            mode: mode.clone(),
            var_scope_floor,
        });

        let body_ind = indent_level.saturating_add(1);
        match mode {
            ActiveRoutineMode::Direct => {
                let is_void = r.return_type == StageLType::Void;
                if let Expr::Block { stmts, .. } = &r.body {
                    let _ = self.emit_block_items(
                        out,
                        stmts,
                        BlockEmitMode::Value {
                            expect_void: is_void,
                            is_fn_tail: true,
                        },
                        body_ind,
                    )?;
                } else {
                    let body_expr = match &r.body {
                        Expr::Return { value: Some(v), .. } => v.as_ref(),
                        other => other,
                    };
                    if matches!(&r.body, Expr::Return { value: None, .. }) {
                        // Bare `return.` as sole body of a void routine needs nothing.
                    } else if matches!(body_expr, Expr::If { .. }) {
                        let (body_code, _) =
                            self.emit_direct_fn_tail_if(body_expr, is_void, body_ind)?;
                        let body_pad = "    ".repeat(body_ind);
                        writeln!(out, "{body_pad}{body_code}")?;
                    } else {
                        let (body_code, body_ty) = self.emit_expr(body_expr, body_ind)?;
                        let body_pad = "    ".repeat(body_ind);
                        if is_void
                            && body_ty != StageLType::Never
                            && !stmt_ends_without_semi(body_expr)
                        {
                            writeln!(out, "{body_pad}{body_code};")?;
                        } else {
                            writeln!(out, "{body_pad}{body_code}")?;
                        }
                    }
                }
            }
            ActiveRoutineMode::Step { .. } => {
                if let Expr::Block { stmts, .. } = &r.body {
                    let _ = self.emit_block_items(
                        out,
                        stmts,
                        BlockEmitMode::TailStep,
                        body_ind,
                    )?;
                } else {
                    let (step_code, _) = self.emit_tail_step_expr(&r.body, body_ind)?;
                    let body_pad = "    ".repeat(body_ind);
                    writeln!(out, "{body_pad}{step_code}")?;
                }
            }
        }

        let _ = self.routine_ctx_stack.pop();
        let _ = self.scopes.pop();

        writeln!(out, "{ind}}}")?;
        writeln!(out)?;
        Ok(())
    }

    fn emit_routine_inline_tests(
        &mut self,
        out: &mut String,
        r: &RoutineDecl,
    ) -> Result<()> {
        if r.tests.is_empty() {
            return Ok(());
        }
        let fn_target = format!("sl_fn_{}", sanitize_ident(&r.name));
        let test_fn = format!("sl_run_tests_{}", sanitize_ident(&r.name));
        writeln!(out, "    pub fn {test_fn}() {{")?;
        for (tb_idx, tb) in r.tests.iter().enumerate() {
            for (case_idx, tc) in tb.cases.iter().enumerate() {
                let mut arg_codes = Vec::with_capacity(tc.inputs.len());
                for inp in &tc.inputs {
                    let (code, _) = self.emit_expr(inp, 2)?;
                    arg_codes.push(code);
                }
                let call_expr = format!("{fn_target}({})", arg_codes.join(", "));
                let desc = format!(
                    "{}::test#{}#{}",
                    r.name, tb_idx, case_idx
                );
                match &tc.expected {
                    TestExpected::Expr(exp) => {
                        let (exp_code, _) = self.emit_expr(exp, 2)?;
                        writeln!(out, "        let __actual = {call_expr};")?;
                        writeln!(out, "        let __expected = {exp_code};")?;
                        writeln!(
                            out,
                            "        assert_eq!(__actual, __expected, \"StageL test failed: {desc}\");"
                        )?;
                    }
                    TestExpected::Fault(_) => {
                        if r.return_type == StageLType::Void
                            || r.return_type == StageLType::Never
                        {
                            writeln!(
                                out,
                                "        sl3_rt::expect_fault(|| {{ {call_expr}; }}, {desc:?});"
                            )?;
                        } else {
                            writeln!(
                                out,
                                "        sl3_rt::expect_fault(|| {{ let _ = {call_expr}; }}, {desc:?});"
                            )?;
                        }
                    }
                    TestExpected::Void(_) => {
                        writeln!(out, "        {call_expr};")?;
                    }
                }
            }
        }
        writeln!(out, "    }}")?;
        writeln!(out)?;
        self.emitted_test_fns.push(test_fn);
        Ok(())
    }

    fn lookup_var(&self, name: &str) -> Option<&VarStorage> {
        let floor = self.current_var_scope_floor();
        let active_scopes = self.scopes.get(floor..)?;
        for scope in active_scopes.iter().rev() {
            if let Some(vs) = scope.vars.get(name) {
                return Some(vs);
            }
        }
        None
    }

    fn visible_var_snapshot(&self) -> HashMap<String, VarStorage> {
        let mut map = HashMap::new();
        let floor = self.current_var_scope_floor();
        if let Some(active_scopes) = self.scopes.get(floor..) {
            for scope in active_scopes {
                for (k, v) in &scope.vars {
                    map.insert(k.clone(), v.clone());
                }
            }
        }
        map
    }

    fn visible_nested_routine_captures(&self) -> HashMap<String, Vec<String>> {
        let mut map = HashMap::new();
        for scope in &self.scopes {
            for (rname, rtarget) in &scope.routines {
                if let RoutineTarget::Nested { captures, .. } = rtarget {
                    map.insert(
                        rname.clone(),
                        captures.iter().map(|c| c.sl_name.clone()).collect(),
                    );
                }
            }
        }
        map
    }

    fn resolve_routine_target(&self, target: &SymbolTarget) -> Result<RoutineTarget> {
        match target {
            SymbolTarget::Ident(name) => {
                for scope in self.scopes.iter().rev() {
                    if let Some(rt) = scope.routines.get(name) {
                        return Ok(rt.clone());
                    }
                }
                if let Some(meta) = self.ws_index.by_name.get(&(self.mod_idx, name.clone())) {
                    let trampoline_call_path = if meta.is_step {
                        format!("{}::{}", meta.mod_ident, meta.step_fn)
                    } else {
                        format!("{}::{}", meta.mod_ident, meta.entry_fn)
                    };
                    return Ok(RoutineTarget::TopLevel {
                        callee_key: CalleeKey::TopLevel {
                            mod_idx: meta.mod_idx,
                            routine_idx: meta.routine_idx,
                        },
                        direct_call_path: meta.entry_fn.clone(),
                        trampoline_call_path,
                        is_step: meta.is_step,
                        sig: meta.sig.clone(),
                    });
                }
                if let Some(ImportedSymbol::Routine {
                    namespace,
                    original_name,
                    ..
                }) = self.validated.imports.get(name)
                {
                    let &target_mod_idx = self
                        .ws_index
                        .ns_to_mod_idx
                        .get(namespace)
                        .with_context(|| format!("Unknown module namespace '{namespace}'"))?;
                    let meta = self
                        .ws_index
                        .by_name
                        .get(&(target_mod_idx, original_name.clone()))
                        .with_context(|| {
                            format!("Unknown imported routine '{namespace}::{original_name}'")
                        })?;
                    let trampoline_call_path = if meta.is_step {
                        format!("{}::{}", meta.mod_ident, meta.step_fn)
                    } else {
                        format!("{}::{}", meta.mod_ident, meta.entry_fn)
                    };
                    return Ok(RoutineTarget::TopLevel {
                        callee_key: CalleeKey::TopLevel {
                            mod_idx: meta.mod_idx,
                            routine_idx: meta.routine_idx,
                        },
                        direct_call_path: format!("super::{}::{}", meta.mod_ident, meta.entry_fn),
                        trampoline_call_path,
                        is_step: meta.is_step,
                        sig: meta.sig.clone(),
                    });
                }
                bail!("Unknown routine '{name}' during Rust transpilation")
            }
            SymbolTarget::Dc(dc) => {
                if let Some(meta) = self.ws_index.by_dc.get(&(self.mod_idx, *dc)) {
                    let trampoline_call_path = if meta.is_step {
                        format!("{}::{}", meta.mod_ident, meta.step_fn)
                    } else {
                        format!("{}::{}", meta.mod_ident, meta.entry_fn)
                    };
                    return Ok(RoutineTarget::TopLevel {
                        callee_key: CalleeKey::TopLevel {
                            mod_idx: meta.mod_idx,
                            routine_idx: meta.routine_idx,
                        },
                        direct_call_path: meta.entry_fn.clone(),
                        trampoline_call_path,
                        is_step: meta.is_step,
                        sig: meta.sig.clone(),
                    });
                }
                if let Some(ImportedSymbol::Routine {
                    namespace,
                    original_name,
                    ..
                }) = self.validated.dc_imports.get(dc)
                {
                    let &target_mod_idx = self
                        .ws_index
                        .ns_to_mod_idx
                        .get(namespace)
                        .with_context(|| format!("Unknown module namespace '{namespace}'"))?;
                    let meta = self
                        .ws_index
                        .by_name
                        .get(&(target_mod_idx, original_name.clone()))
                        .with_context(|| {
                            format!("Unknown imported Dc routine '{namespace}::{original_name}'")
                        })?;
                    let trampoline_call_path = if meta.is_step {
                        format!("{}::{}", meta.mod_ident, meta.step_fn)
                    } else {
                        format!("{}::{}", meta.mod_ident, meta.entry_fn)
                    };
                    return Ok(RoutineTarget::TopLevel {
                        callee_key: CalleeKey::TopLevel {
                            mod_idx: meta.mod_idx,
                            routine_idx: meta.routine_idx,
                        },
                        direct_call_path: format!("super::{}::{}", meta.mod_ident, meta.entry_fn),
                        trampoline_call_path,
                        is_step: meta.is_step,
                        sig: meta.sig.clone(),
                    });
                }
                bail!("Unknown Dc routine '{dc}' during Rust transpilation")
            }
        }
    }

    fn resolve_value_ref(&self, target: &SymbolTarget) -> Result<(String, StageLType)> {
        match target {
            SymbolTarget::Ident(name) => {
                if let Some(vs) = self.lookup_var(name) {
                    return Ok((vs.read_expr(name), vs.ty().clone()));
                }
                if let Some((rname, ty)) = self.local_consts.get(name) {
                    let expr = if rust_type_copy_cheap(ty) {
                        format!("*{rname}")
                    } else {
                        format!("{rname}.clone()")
                    };
                    return Ok((expr, ty.clone()));
                }
                if let Some(ImportedSymbol::Value {
                    namespace,
                    original_name,
                    ty,
                    ..
                }) = self.validated.imports.get(name)
                {
                    let mod_ident = self.ws_index.mod_ident_by_ns(namespace)?;
                    let path = format!("super::{mod_ident}::sl_c_{}", sanitize_ident(original_name));
                    let expr = if rust_type_copy_cheap(ty) {
                        format!("*{path}")
                    } else {
                        format!("{path}.clone()")
                    };
                    return Ok((expr, ty.clone()));
                }
                bail!("Unknown value '{name}' during Rust transpilation")
            }
            SymbolTarget::Dc(dc) => {
                if let Some((rname, ty)) = self.local_dc_consts.get(dc) {
                    let expr = if rust_type_copy_cheap(ty) {
                        format!("*{rname}")
                    } else {
                        format!("{rname}.clone()")
                    };
                    return Ok((expr, ty.clone()));
                }
                if let Some(ImportedSymbol::Value {
                    namespace,
                    original_name,
                    ty,
                    ..
                }) = self.validated.dc_imports.get(dc)
                {
                    let mod_ident = self.ws_index.mod_ident_by_ns(namespace)?;
                    let path = format!("super::{mod_ident}::sl_c_{}", sanitize_ident(original_name));
                    let expr = if rust_type_copy_cheap(ty) {
                        format!("*{path}")
                    } else {
                        format!("{path}.clone()")
                    };
                    return Ok((expr, ty.clone()));
                }
                bail!("Unknown Dc value '{dc}' during Rust transpilation")
            }
        }
    }

    fn emit_call_operands(
        &mut self,
        args: &[Expr],
        captures: &[CapturedArg],
        indent_level: usize,
    ) -> Result<OperandEval> {
        let mut rendered = Vec::with_capacity(args.len().saturating_add(captures.len()));
        for a in args {
            let (ac, aty) = self.emit_expr(a, indent_level)?;
            if aty == StageLType::Never {
                return Ok(OperandEval::Diverged(prefix_prior_exprs(&rendered, &ac)));
            }
            rendered.push(ac);
        }
        for cap in captures {
            let cur_vs = self
                .lookup_var(&cap.sl_name)
                .with_context(|| format!("Missing captured binding '{}'", cap.sl_name))?;
            match (cur_vs, cap.kind) {
                (VarStorage::CapCell { rust_name: rn, .. }, CapturePassKind::CapCell) => {
                    rendered.push(format!("{rn}.clone()"));
                }
                (VarStorage::Direct { .. }, CapturePassKind::ImmVal) => {
                    rendered.push(cur_vs.read_expr(&cap.sl_name));
                }
                _ => {
                    bail!("Inconsistent capture storage for '{}'", cap.sl_name);
                }
            }
        }
        Ok(OperandEval::Ready(rendered))
    }

    fn emit_tail_step_expr(
        &mut self,
        expr: &Expr,
        indent_level: usize,
    ) -> Result<(String, StageLType)> {
        match expr {
            Expr::Call { target, args, .. } => {
                let resolved = self.resolve_routine_target(target)?;
                let (callee_key, trampoline_call_path, callee_is_step, sig, captures) =
                    match resolved {
                        RoutineTarget::TopLevel {
                            callee_key,
                            trampoline_call_path,
                            is_step,
                            sig,
                            ..
                        } => (callee_key, trampoline_call_path, is_step, sig, Vec::new()),
                        RoutineTarget::Nested {
                            callee_key,
                            trampoline_call_path,
                            is_step,
                            sig,
                            captures,
                            ..
                        } => (callee_key, trampoline_call_path, is_step, sig, captures),
                    };

                let rendered_args =
                    match self.emit_call_operands(args, &captures, indent_level)? {
                        OperandEval::Diverged(div_code) => {
                            return Ok((div_code, StageLType::Never));
                        }
                        OperandEval::Ready(r) => r,
                    };

                let mut field_rust_types =
                    Vec::with_capacity(sig.params.len().saturating_add(captures.len()));
                for pty in &sig.params {
                    field_rust_types.push(rust_type(pty));
                }
                for cap in &captures {
                    field_rust_types.push(cap.field_rust_type());
                }

                if sig.return_type == StageLType::Never {
                    let (variant_ident, boxed_struct) = self.trampolines.register_never_target(
                        callee_key,
                        trampoline_call_path,
                        callee_is_step,
                        field_rust_types,
                    )?;
                    let ncall = format_variant_ctor(
                        "super::SlTailNever",
                        &variant_ident,
                        &rendered_args,
                        boxed_struct.as_deref(),
                    );
                    match self.current_routine_mode() {
                        ActiveRoutineMode::Step { group_id: None, .. } => {
                            Ok((ncall, StageLType::Void))
                        }
                        ActiveRoutineMode::Step {
                            group_id: Some(_), ..
                        } => {
                            self.trampolines.has_value_to_never_tail = true;
                            Ok((format!("sl3_rt::SlStep::TailNever({ncall})"), StageLType::Void))
                        }
                        ActiveRoutineMode::Direct => {
                            bail!("Internal error: emit_tail_step_expr invoked outside step routine")
                        }
                    }
                } else {
                    let (gid, variant_ident, boxed_struct) =
                        self.trampolines.register_value_target(
                            &sig.return_type,
                            callee_key,
                            trampoline_call_path,
                            callee_is_step,
                            field_rust_types,
                        )?;
                    let vcall = format_variant_ctor(
                        &format!("super::SlTailGroup{gid}"),
                        &variant_ident,
                        &rendered_args,
                        boxed_struct.as_deref(),
                    );
                    Ok((format!("sl3_rt::SlStep::Tail({vcall})"), StageLType::Void))
                }
            }
            Expr::Block { stmts, .. } => {
                let mut buf = String::from("{\n");
                let ty = self.emit_block_items(
                    &mut buf,
                    stmts,
                    BlockEmitMode::TailStep,
                    indent_level.saturating_add(1),
                )?;
                let ind = "    ".repeat(indent_level);
                write!(buf, "{ind}}}")?;
                Ok((buf, ty))
            }
            Expr::If {
                cond,
                then_branch,
                else_branch,
                ..
            } => {
                let (cond_code, cond_ty) = self.emit_expr(cond, indent_level)?;
                if cond_ty == StageLType::Never {
                    return Ok((cond_code, StageLType::Never));
                }
                let (then_block, then_ty) =
                    self.emit_braced_tail_step_branch(then_branch, indent_level)?;
                let (else_code, else_ty) = if matches!(else_branch.as_ref(), Expr::If { .. }) {
                    self.emit_tail_step_expr(else_branch, indent_level)?
                } else {
                    self.emit_braced_tail_step_branch(else_branch, indent_level)?
                };
                let res_ty = if then_ty == StageLType::Never && else_ty == StageLType::Never {
                    StageLType::Never
                } else {
                    StageLType::Void
                };
                let cond_clean = strip_matched_outer_parens(&cond_code);
                Ok((
                    format_if_expr(cond_clean, &then_block, &else_code, indent_level),
                    res_ty,
                ))
            }
            Expr::Return { value, .. } => {
                if let Some(v) = value {
                    self.emit_tail_step_expr(v, indent_level)
                } else {
                    Ok((String::from("sl3_rt::SlStep::Done(())"), StageLType::Void))
                }
            }
            other => {
                let (code, ty) = self.emit_expr(other, indent_level)?;
                match ty {
                    StageLType::Never => Ok((code, StageLType::Never)),
                    StageLType::Void => {
                        if stmt_ends_without_semi(other) {
                            Ok((
                                format!("{{ {code} sl3_rt::SlStep::Done(()) }}"),
                                StageLType::Void,
                            ))
                        } else {
                            Ok((
                                format!("{{ {code}; sl3_rt::SlStep::Done(()) }}"),
                                StageLType::Void,
                            ))
                        }
                    }
                    _ => Ok((format!("sl3_rt::SlStep::Done({code})"), StageLType::Void)),
                }
            }
        }
    }

    fn emit_braced_tail_step_branch(
        &mut self,
        expr: &Expr,
        indent_level: usize,
    ) -> Result<(String, StageLType)> {
        if let Expr::Block { stmts, .. } = expr {
            let mut buf = String::from("{\n");
            let ty = self.emit_block_items(
                &mut buf,
                stmts,
                BlockEmitMode::TailStep,
                indent_level.saturating_add(1),
            )?;
            let ind = "    ".repeat(indent_level);
            write!(buf, "{ind}}}")?;
            return Ok((buf, ty));
        }
        let body_ind = indent_level.saturating_add(1);
        let (code, ty) = self.emit_tail_step_expr(expr, body_ind)?;
        let ind = "    ".repeat(indent_level);
        let inner_ind = "    ".repeat(body_ind);
        Ok((format!("{{\n{inner_ind}{code}\n{ind}}}"), ty))
    }

    fn emit_expr(&mut self, expr: &Expr, indent_level: usize) -> Result<(String, StageLType)> {
        match expr {
            Expr::Bool { value, .. } => {
                Ok((if *value { "true" } else { "false" }.to_owned(), StageLType::B))
            }
            Expr::Int { value, .. } => Ok((emit_int_lit(*value), value.stage_l_type())),
            Expr::Bytes { value, .. } => {
                let code = format!("sl3_rt::bytes_lit(&{})", format_u8_slice_lit(value));
                Ok((code, StageLType::List(Box::new(StageLType::U8))))
            }
            Expr::List { elem_ty, elements, .. } => {
                let mut codes = Vec::with_capacity(elements.len());
                for el in elements {
                    let (c, ety) = self.emit_expr(el, indent_level)?;
                    if ety == StageLType::Never {
                        return Ok((prefix_prior_exprs(&codes, &c), StageLType::Never));
                    }
                    codes.push(c);
                }
                let r_elem = rust_type(elem_ty);
                let code = if codes.is_empty() {
                    format!("sl3_rt::list_from_vec::<{r_elem}>(Vec::new())")
                } else {
                    format!("sl3_rt::list_from_vec::<{r_elem}>(vec![{}])", codes.join(", "))
                };
                Ok((code, StageLType::List(Box::new(elem_ty.clone()))))
            }
            Expr::BufNew { elem_ty, capacity, .. } => {
                let (cap_code, cap_ty) = self.emit_expr(capacity, indent_level)?;
                if cap_ty == StageLType::Never {
                    return Ok((cap_code, StageLType::Never));
                }
                let r_elem = rust_type(elem_ty);
                Ok((
                    format!("sl3_rt::SlBuf::<{r_elem}>::new_with_cap({cap_code})"),
                    StageLType::Buf(Box::new(elem_ty.clone())),
                ))
            }
            Expr::ValueRef { target, .. } => self.resolve_value_ref(target),
            Expr::Let { .. } | Expr::Const { .. } | Expr::RoutineDecl(_) => {
                Ok((String::from("()"), StageLType::Void))
            }
            Expr::Set { name, value, .. } => {
                let (val_code, val_ty) = self.emit_expr(value, indent_level)?;
                let target_var = self
                    .lookup_var(name)
                    .with_context(|| format!("Unknown variable '{name}' in 'set'"))?
                    .clone();
                if val_ty == StageLType::Never {
                    return Ok((val_code, StageLType::Never));
                }
                let code = match target_var {
                    VarStorage::Direct { rust_name, .. } => {
                        format!("{rust_name} = {val_code}")
                    }
                    VarStorage::CapCell { rust_name, .. } => {
                        format!("sl3_rt::cap_set(&{rust_name}, {val_code})")
                    }
                };
                Ok((code, StageLType::Void))
            }
            Expr::Call { target, args, .. } => {
                let resolved = self.resolve_routine_target(target)?;
                match resolved {
                    RoutineTarget::TopLevel {
                        direct_call_path,
                        sig,
                        ..
                    } => match self.emit_call_operands(args, &[], indent_level)? {
                        OperandEval::Diverged(div_code) => Ok((div_code, StageLType::Never)),
                        OperandEval::Ready(rendered_args) => Ok((
                            format!("{direct_call_path}({})", rendered_args.join(", ")),
                            sig.return_type,
                        )),
                    },
                    RoutineTarget::Nested {
                        local_fn_name,
                        step_group_id,
                        is_step,
                        sig,
                        captures,
                        ..
                    } => match self.emit_call_operands(args, &captures, indent_level)? {
                        OperandEval::Diverged(div_code) => Ok((div_code, StageLType::Never)),
                        OperandEval::Ready(rendered_args) => {
                            let raw_call =
                                format!("{local_fn_name}({})", rendered_args.join(", "));
                            if !is_step {
                                Ok((raw_call, sig.return_type))
                            } else if let Some(gid) = step_group_id {
                                Ok((
                                    format!("super::sl_trampoline_{gid}({raw_call})"),
                                    sig.return_type,
                                ))
                            } else {
                                Ok((
                                    format!("super::sl_trampoline_never({raw_call})"),
                                    StageLType::Never,
                                ))
                            }
                        }
                    },
                }
            }
            Expr::Block { stmts, .. } => self.emit_block(stmts, indent_level),
            Expr::If {
                cond,
                then_branch,
                else_branch,
                ..
            } => {
                let (cond_code, cond_ty) = self.emit_expr(cond, indent_level)?;
                if cond_ty == StageLType::Never {
                    return Ok((cond_code, StageLType::Never));
                }
                let (then_block, then_ty) =
                    self.emit_braced_branch(then_branch, false, false, indent_level)?;
                let (else_code, else_ty) = if matches!(else_branch.as_ref(), Expr::If { .. }) {
                    self.emit_expr(else_branch, indent_level)?
                } else {
                    self.emit_braced_branch(else_branch, false, false, indent_level)?
                };
                let res_ty = match (then_ty == StageLType::Never, else_ty == StageLType::Never) {
                    (true, true) => StageLType::Never,
                    (true, false) => else_ty,
                    (false, true) => then_ty,
                    (false, false) => then_ty,
                };
                let cond_clean = strip_matched_outer_parens(&cond_code);
                Ok((
                    format_if_expr(cond_clean, &then_block, &else_code, indent_level),
                    res_ty,
                ))
            }
            Expr::While { cond, body, .. } => {
                let (cond_code, cond_ty) = self.emit_expr(cond, indent_level)?;
                if cond_ty == StageLType::Never {
                    return Ok((cond_code, StageLType::Never));
                }
                let (body_block, _) =
                    self.emit_braced_branch(body, true, false, indent_level)?;
                let cond_clean = strip_matched_outer_parens(&cond_code);
                let code = format!("while {cond_clean} {body_block}");
                Ok((code, StageLType::Void))
            }
            Expr::ForEach {
                elem_ty,
                elem_name,
                list,
                body,
                ..
            } => {
                let (list_code, list_ty) = self.emit_expr(list, indent_level)?;
                if list_ty == StageLType::Never {
                    return Ok((list_code, StageLType::Never));
                }
                let r_elem = format!("sl_v_{}", sanitize_ident(elem_name));
                let mut frame = ScopeFrame {
                    vars: HashMap::new(),
                    routines: HashMap::new(),
                };
                frame.vars.insert(
                    elem_name.clone(),
                    VarStorage::Direct {
                        rust_name: r_elem.clone(),
                        ty: elem_ty.clone(),
                    },
                );
                self.scopes.push(frame);
                let (body_block, _) =
                    self.emit_braced_branch(body, true, false, indent_level)?;
                let _ = self.scopes.pop();

                let code =
                    format!("for {r_elem} in ({list_code}).iter().cloned() {body_block}");
                Ok((code, StageLType::Void))
            }
            Expr::Break { .. } => Ok((String::from("break"), StageLType::Never)),
            Expr::Continue { .. } => Ok((String::from("continue"), StageLType::Never)),
            Expr::Return { value, .. } => match self.current_routine_mode() {
                ActiveRoutineMode::Direct => {
                    if let Some(v) = value {
                        let (vc, vty) = self.emit_expr(v, indent_level)?;
                        if vty == StageLType::Never {
                            Ok((vc, StageLType::Never))
                        } else {
                            Ok((format!("return {vc}"), StageLType::Never))
                        }
                    } else {
                        Ok((String::from("return"), StageLType::Never))
                    }
                }
                ActiveRoutineMode::Step { .. } => {
                    if let Some(v) = value {
                        let (step_code, step_ty) =
                            self.emit_tail_step_expr(v, indent_level)?;
                        if step_ty == StageLType::Never {
                            Ok((step_code, StageLType::Never))
                        } else {
                            Ok((format!("return {step_code}"), StageLType::Never))
                        }
                    } else {
                        Ok((
                            String::from("return sl3_rt::SlStep::Done(())"),
                            StageLType::Never,
                        ))
                    }
                }
            },
            Expr::Fault { span } => {
                let msg = format!("explicit fault trap at {span}");
                Ok((format!("panic!({msg:?})"), StageLType::Never))
            }
            Expr::Unary { op, operand, .. } => self.emit_unary(*op, operand, indent_level),
            Expr::Binary {
                op, left, right, ..
            } => self.emit_binary(*op, left, right, indent_level),
            Expr::Ternary {
                op,
                first,
                second,
                third,
                ..
            } => self.emit_ternary(*op, first, second, third, indent_level),
            Expr::Conv {
                expr: inner,
                target_ty,
                ..
            } => self.emit_conv(inner, target_ty, indent_level),
            Expr::Bitcast {
                expr: inner,
                target_ty,
                ..
            } => self.emit_bitcast(inner, target_ty, indent_level),
        }
    }

    fn emit_braced_branch(
        &mut self,
        expr: &Expr,
        expect_void: bool,
        is_fn_tail: bool,
        indent_level: usize,
    ) -> Result<(String, StageLType)> {
        if let Expr::Block { stmts, .. } = expr {
            let mut buf = String::from("{\n");
            let ty = self.emit_block_items(
                &mut buf,
                stmts,
                BlockEmitMode::Value {
                    expect_void,
                    is_fn_tail,
                },
                indent_level.saturating_add(1),
            )?;
            if buf == "{\n" {
                return Ok((String::from("{}"), StageLType::Void));
            }
            let ind = "    ".repeat(indent_level);
            write!(buf, "{ind}}}")?;
            return Ok((buf, ty));
        }
        let body_ind = indent_level.saturating_add(1);
        let ind = "    ".repeat(indent_level);
        let inner_ind = "    ".repeat(body_ind);
        if is_fn_tail && matches!(expr, Expr::Return { value: None, .. }) {
            return Ok((String::from("{}"), StageLType::Void));
        }
        let target_expr = if is_fn_tail {
            match expr {
                Expr::Return { value: Some(v), .. } => v.as_ref(),
                other => other,
            }
        } else {
            expr
        };
        let (code, ty) = self.emit_expr(target_expr, body_ind)?;
        if (expect_void || ty == StageLType::Void)
            && ty != StageLType::Never
            && !stmt_ends_without_semi(target_expr)
        {
            Ok((format!("{{\n{inner_ind}{code};\n{ind}}}"), ty))
        } else {
            Ok((format!("{{\n{inner_ind}{code}\n{ind}}}"), ty))
        }
    }

    fn emit_block(
        &mut self,
        stmts: &[BlockItem],
        indent_level: usize,
    ) -> Result<(String, StageLType)> {
        let mut buf = String::from("{\n");
        let ty = self.emit_block_items(
            &mut buf,
            stmts,
            BlockEmitMode::Value {
                expect_void: false,
                is_fn_tail: false,
            },
            indent_level.saturating_add(1),
        )?;
        if buf == "{\n" {
            return Ok((String::from("{}"), StageLType::Void));
        }
        let ind = "    ".repeat(indent_level);
        write!(buf, "{ind}}}")?;
        Ok((buf, ty))
    }

    fn emit_block_items(
        &mut self,
        block_buf: &mut String,
        stmts: &[BlockItem],
        mode: BlockEmitMode,
        stmt_indent: usize,
    ) -> Result<StageLType> {
        let mut expr_stmts = Vec::new();
        for item in stmts {
            if let BlockItem::Expr(e) = item {
                expr_stmts.push(e);
            }
        }
        let inner_ind = "    ".repeat(stmt_indent);
        if expr_stmts.is_empty() {
            if matches!(mode, BlockEmitMode::TailStep) {
                writeln!(block_buf, "{inner_ind}sl3_rt::SlStep::Done(())")?;
            }
            return Ok(StageLType::Void);
        }

        // Collect block-level declarations & capture plan.
        let enclosing_caps = self.visible_nested_routine_captures();
        let plan = analyze_block_captures(&expr_stmts, &enclosing_caps);
        let outer_snapshot = self.visible_var_snapshot();

        let mut frame = ScopeFrame {
            vars: HashMap::new(),
            routines: HashMap::new(),
        };

        // Pre-compute capture params shared by nested routines declared directly
        // in this block (any outer variable referenced by nested routines, plus
        // any captured `let` or post-routine `const` declared inside this block).
        let mut ordered_capture_names: Vec<String> =
            plan.all_captured_outer.iter().cloned().collect();
        ordered_capture_names.sort();

        let mut shared_captures = Vec::new();
        for cap_name in &ordered_capture_names {
            if let Some(vs) = outer_snapshot.get(cap_name) {
                let kind = match vs {
                    VarStorage::CapCell { .. } => CapturePassKind::CapCell,
                    VarStorage::Direct { .. } => CapturePassKind::ImmVal,
                };
                shared_captures.push(CapturedArg {
                    sl_name: cap_name.clone(),
                    rust_param: format!("sl_v_{}", sanitize_ident(cap_name)),
                    ty: vs.ty().clone(),
                    kind,
                });
            }
        }

        // Local `let` variables or `const`s in THIS block that are captured by
        // direct or descendant nested routines use `SlCap<T>` pre-declared upfront.
        let mut block_cap_decls: Vec<(String, StageLType)> = Vec::new();
        let mut local_cap_names: Vec<String> = plan.local_captured_vars.keys().cloned().collect();
        local_cap_names.sort();
        for name in local_cap_names {
            if let Some(ty) = plan.local_captured_vars.get(&name) {
                let rname = format!("sl_v_{}", sanitize_ident(&name));
                frame.vars.insert(
                    name.clone(),
                    VarStorage::CapCell {
                        rust_name: rname.clone(),
                        ty: ty.clone(),
                    },
                );
                block_cap_decls.push((name.clone(), ty.clone()));
                shared_captures.push(CapturedArg {
                    sl_name: name,
                    rust_param: rname,
                    ty: ty.clone(),
                    kind: CapturePassKind::CapCell,
                });
            }
        }

        let mut routine_emit_list: Vec<(
            &RoutineDecl,
            Vec<CapturedArg>,
            String,
            ActiveRoutineMode,
        )> = Vec::with_capacity(plan.nested_routines.len());
        for &r in &plan.nested_routines {
            let needed = plan.routine_captures.get(&r.name);
            let r_caps: Vec<CapturedArg> = shared_captures
                .iter()
                .filter(|c| needed.is_some_and(|set| set.contains(&c.sl_name)))
                .cloned()
                .collect();
            let nid = self.next_nested_id();
            let san = sanitize_ident(&r.name);
            let is_step = expr_has_tail_call(&r.body, true);
            let (local_fn_name, step_group_id, nested_mode) = if is_step {
                let gid = if r.return_type == StageLType::Never {
                    self.trampolines.ensure_never_group();
                    None
                } else {
                    Some(self.trampolines.ensure_value_group(&r.return_type))
                };
                (
                    format!("__sl_step_nested_{nid}_{san}"),
                    gid,
                    ActiveRoutineMode::Step {
                        ret_ty: r.return_type.clone(),
                        group_id: gid,
                    },
                )
            } else {
                (
                    format!("__sl_nfn_{nid}_{san}"),
                    None,
                    ActiveRoutineMode::Direct,
                )
            };
            let trampoline_call_path = format!("{}::{local_fn_name}", self.mod_ident);
            let sig = RoutineSig {
                return_type: r.return_type.clone(),
                params: r.params.iter().map(|p| p.ty.clone()).collect(),
                is_pub: false,
                dc_id: None,
            };
            frame.routines.insert(
                r.name.clone(),
                RoutineTarget::Nested {
                    callee_key: CalleeKey::Nested {
                        mod_idx: self.mod_idx,
                        nested_id: nid,
                    },
                    local_fn_name: local_fn_name.clone(),
                    trampoline_call_path,
                    step_group_id,
                    is_step,
                    sig,
                    captures: r_caps.clone(),
                },
            );
            routine_emit_list.push((r, r_caps, local_fn_name, nested_mode));
        }

        self.scopes.push(frame);

        for (name, ty) in &block_cap_decls {
            let rname = format!("sl_v_{}", sanitize_ident(name));
            let rty = rust_type(ty);
            writeln!(
                block_buf,
                "{inner_ind}let {rname}: sl3_rt::SlCap<{rty}> = sl3_rt::cap_new();"
            )?;
        }

        for (r, r_caps, local_fn_name, nested_mode) in routine_emit_list {
            let mut lifted_buf = String::new();
            self.emit_routine_fn(
                &mut lifted_buf,
                "pub(super) ",
                &local_fn_name,
                r,
                &r_caps,
                nested_mode,
                1,
            )?;
            self.lifted_routines.push(lifted_buf);
        }

        let mut result_ty = StageLType::Void;
        let mut terminated = false;
        let count = expr_stmts.len();
        for (idx, stmt) in expr_stmts.iter().enumerate() {
            if terminated {
                result_ty = StageLType::Never;
                break;
            }
            let is_last = idx.saturating_add(1) == count;
            match stmt {
                Expr::RoutineDecl(_) => {
                    if is_last {
                        result_ty = StageLType::Void;
                        if matches!(mode, BlockEmitMode::TailStep) {
                            writeln!(block_buf, "{inner_ind}sl3_rt::SlStep::Done(())")?;
                        }
                    }
                }
                Expr::Let { ty, name, init, .. } => {
                    let is_cap = plan.local_captured_vars.contains_key(name);
                    let rname = format!("sl_v_{}", sanitize_ident(name));
                    let rty = rust_type(ty);
                    let mut_kw = if plan.mutated_locals.contains(name) {
                        "mut "
                    } else {
                        ""
                    };
                    if let Some(init_expr) = init {
                        let (init_code, init_ty) = self.emit_expr(init_expr, stmt_indent)?;
                        if is_cap {
                            if init_ty == StageLType::Never {
                                writeln!(block_buf, "{inner_ind}{init_code};")?;
                                terminated = true;
                            } else {
                                writeln!(
                                    block_buf,
                                    "{inner_ind}sl3_rt::cap_set(&{rname}, {init_code});"
                                )?;
                            }
                        } else if init_ty == StageLType::Never {
                            writeln!(block_buf, "{inner_ind}{init_code};")?;
                            terminated = true;
                        } else {
                            writeln!(
                                block_buf,
                                "{inner_ind}let {mut_kw}{rname}: {rty} = {init_code};"
                            )?;
                            if let Some(top) = self.scopes.last_mut() {
                                top.vars.insert(
                                    name.clone(),
                                    VarStorage::Direct {
                                        rust_name: rname,
                                        ty: ty.clone(),
                                    },
                                );
                            }
                        }
                    } else if !is_cap {
                        writeln!(block_buf, "{inner_ind}let {mut_kw}{rname}: {rty};")?;
                        if let Some(top) = self.scopes.last_mut() {
                            top.vars.insert(
                                name.clone(),
                                VarStorage::Direct {
                                    rust_name: rname,
                                    ty: ty.clone(),
                                },
                            );
                        }
                    }
                    if is_last {
                        if terminated {
                            result_ty = StageLType::Never;
                        } else {
                            result_ty = StageLType::Void;
                            if matches!(mode, BlockEmitMode::TailStep) {
                                writeln!(block_buf, "{inner_ind}sl3_rt::SlStep::Done(())")?;
                            }
                        }
                    }
                }
                Expr::Const { ty, name, value, .. } => {
                    let is_cap = plan.local_captured_vars.contains_key(name);
                    let rname = format!("sl_v_{}", sanitize_ident(name));
                    let rty = rust_type(ty);
                    let (val_code, val_ty) = self.emit_expr(value, stmt_indent)?;
                    if is_cap {
                        if val_ty == StageLType::Never {
                            writeln!(block_buf, "{inner_ind}{val_code};")?;
                            terminated = true;
                        } else {
                            writeln!(
                                block_buf,
                                "{inner_ind}sl3_rt::cap_set(&{rname}, {val_code});"
                            )?;
                        }
                    } else if val_ty == StageLType::Never {
                        writeln!(block_buf, "{inner_ind}{val_code};")?;
                        terminated = true;
                    } else {
                        writeln!(block_buf, "{inner_ind}let {rname}: {rty} = {val_code};")?;
                        if let Some(top) = self.scopes.last_mut() {
                            top.vars.insert(
                                name.clone(),
                                VarStorage::Direct {
                                    rust_name: rname,
                                    ty: ty.clone(),
                                },
                            );
                        }
                    }
                    if is_last {
                        if terminated {
                            result_ty = StageLType::Never;
                        } else {
                            result_ty = StageLType::Void;
                            if matches!(mode, BlockEmitMode::TailStep) {
                                writeln!(block_buf, "{inner_ind}sl3_rt::SlStep::Done(())")?;
                            }
                        }
                    }
                }
                other => {
                    if is_last {
                        match mode {
                            BlockEmitMode::TailStep => {
                                let (step_code, step_ty) =
                                    self.emit_tail_step_expr(other, stmt_indent)?;
                                result_ty = if step_ty == StageLType::Never {
                                    StageLType::Never
                                } else {
                                    StageLType::Void
                                };
                                writeln!(block_buf, "{inner_ind}{step_code}")?;
                            }
                            BlockEmitMode::Value {
                                expect_void,
                                is_fn_tail,
                            } => {
                                if is_fn_tail
                                    && matches!(other, Expr::Return { value: None, .. })
                                {
                                    result_ty = StageLType::Void;
                                } else if is_fn_tail
                                    && matches!(other, Expr::If { .. })
                                {
                                    let (if_code, if_ty) = self.emit_direct_fn_tail_if(
                                        other,
                                        expect_void,
                                        stmt_indent,
                                    )?;
                                    result_ty = if_ty;
                                    writeln!(block_buf, "{inner_ind}{if_code}")?;
                                } else {
                                    let target_expr = if is_fn_tail {
                                        match other {
                                            Expr::Return { value: Some(v), .. } => v.as_ref(),
                                            e => e,
                                        }
                                    } else {
                                        other
                                    };
                                    let (code, ty) =
                                        self.emit_expr(target_expr, stmt_indent)?;
                                    result_ty = ty.clone();
                                    if (expect_void || ty == StageLType::Void)
                                        && ty != StageLType::Never
                                        && !stmt_ends_without_semi(target_expr)
                                    {
                                        writeln!(block_buf, "{inner_ind}{code};")?;
                                    } else {
                                        writeln!(block_buf, "{inner_ind}{code}")?;
                                    }
                                }
                            }
                        }
                    } else {
                        let (code, ty) = self.emit_expr(other, stmt_indent)?;
                        if ty == StageLType::Never {
                            terminated = true;
                        }
                        if ty == StageLType::Void || ty == StageLType::Never {
                            if stmt_ends_without_semi(other) {
                                writeln!(block_buf, "{inner_ind}{code}")?;
                            } else {
                                writeln!(block_buf, "{inner_ind}{code};")?;
                            }
                        } else {
                            writeln!(block_buf, "{inner_ind}let _ = {code};")?;
                        }
                    }
                }
            }
        }

        let _ = self.scopes.pop();
        Ok(result_ty)
    }

    fn emit_unary(
        &mut self,
        op: CoreUnaryOp,
        operand: &Expr,
        indent_level: usize,
    ) -> Result<(String, StageLType)> {
        let (op_code, op_ty) = self.emit_expr(operand, indent_level)?;
        if op_ty == StageLType::Never {
            return Ok((op_code, StageLType::Never));
        }
        match op {
            CoreUnaryOp::Neg => {
                let code = format!(
                    "{{ let Some(__neg_v) = ({op_code}).checked_neg() else {{ panic!(\"checked neg overflow\"); }}; __neg_v }}"
                );
                Ok((code, op_ty))
            }
            CoreUnaryOp::BitNot | CoreUnaryOp::Not => Ok((format!("!({op_code})"), op_ty)),
            CoreUnaryOp::Clz => Ok((format!("({op_code}).leading_zeros()"), StageLType::U32)),
            CoreUnaryOp::Ctz => Ok((format!("({op_code}).trailing_zeros()"), StageLType::U32)),
            CoreUnaryOp::Popcnt => Ok((format!("({op_code}).count_ones()"), StageLType::U32)),
            CoreUnaryOp::Len => match &op_ty {
                StageLType::List(_) => Ok((format!("sl3_rt::list_len(&({op_code}))"), StageLType::U64)),
                StageLType::Buf(_) => Ok((format!("({op_code}).len()"), StageLType::U64)),
                other => bail!("Invalid operand type '{other}' for 'len'"),
            },
            CoreUnaryOp::BufFreeze => match &op_ty {
                StageLType::Buf(inner) => Ok((
                    format!("({op_code}).freeze()"),
                    StageLType::List(inner.clone()),
                )),
                other => bail!("Invalid operand type '{other}' for 'bufFreeze'"),
            },
        }
    }

    fn emit_binary(
        &mut self,
        op: CoreBinaryOp,
        left: &Expr,
        right: &Expr,
        indent_level: usize,
    ) -> Result<(String, StageLType)> {
        let (l_code, l_ty) = self.emit_expr(left, indent_level)?;
        if l_ty == StageLType::Never {
            return Ok((l_code, StageLType::Never));
        }

        if matches!(op, CoreBinaryOp::And | CoreBinaryOp::Or) {
            let (r_code, _) = self.emit_expr(right, indent_level)?;
            let sym = if op == CoreBinaryOp::And { "&&" } else { "||" };
            return Ok((format!("(({l_code}) {sym} ({r_code}))"), StageLType::B));
        }

        let (r_code, r_ty) = self.emit_expr(right, indent_level)?;
        if r_ty == StageLType::Never {
            return Ok((
                prefix_prior_exprs(std::slice::from_ref(&l_code), &r_code),
                StageLType::Never,
            ));
        }

        match op {
            CoreBinaryOp::Add
            | CoreBinaryOp::Sub
            | CoreBinaryOp::Mul
            | CoreBinaryOp::Div
            | CoreBinaryOp::Rem => {
                let method = match op {
                    CoreBinaryOp::Add => "checked_add",
                    CoreBinaryOp::Sub => "checked_sub",
                    CoreBinaryOp::Mul => "checked_mul",
                    CoreBinaryOp::Div => "checked_div",
                    CoreBinaryOp::Rem => "checked_rem",
                    _ => unreachable!("handled above"),
                };
                let tmp = self.next_tmp("v");
                let msg = format!("checked arithmetic fault in '{}'", op.keyword());
                Ok((
                    format!("{{ let Some({tmp}) = ({l_code}).{method}({r_code}) else {{ panic!({msg:?}); }}; {tmp} }}"),
                    l_ty,
                ))
            }
            CoreBinaryOp::WrapAdd | CoreBinaryOp::WrapSub | CoreBinaryOp::WrapMul => {
                let method = match op {
                    CoreBinaryOp::WrapAdd => "wrapping_add",
                    CoreBinaryOp::WrapSub => "wrapping_sub",
                    CoreBinaryOp::WrapMul => "wrapping_mul",
                    _ => unreachable!("handled above"),
                };
                Ok((format!("({l_code}).{method}({r_code})"), l_ty))
            }
            CoreBinaryOp::BitAnd | CoreBinaryOp::BitOr | CoreBinaryOp::BitXor => {
                let sym = match op {
                    CoreBinaryOp::BitAnd => "&",
                    CoreBinaryOp::BitOr => "|",
                    CoreBinaryOp::BitXor => "^",
                    _ => unreachable!("handled above"),
                };
                Ok((format!("(({l_code}) {sym} ({r_code}))"), l_ty))
            }
            CoreBinaryOp::Shl | CoreBinaryOp::Shr => {
                let method = if op == CoreBinaryOp::Shl {
                    "checked_shl"
                } else {
                    "checked_shr"
                };
                let tmp = self.next_tmp("sh");
                Ok((
                    format!("{{ let Some({tmp}) = ({l_code}).{method}({r_code}) else {{ panic!(\"shift count out of range\"); }}; {tmp} }}"),
                    l_ty,
                ))
            }
            CoreBinaryOp::Rotl | CoreBinaryOp::Rotr => {
                let method = if op == CoreBinaryOp::Rotl {
                    "rotate_left"
                } else {
                    "rotate_right"
                };
                Ok((format!("({l_code}).{method}({r_code})"), l_ty))
            }
            CoreBinaryOp::And | CoreBinaryOp::Or => unreachable!("handled above"),
            CoreBinaryOp::Eq | CoreBinaryOp::Ne => {
                let sym = if op == CoreBinaryOp::Eq { "==" } else { "!=" };
                Ok((format!("(({l_code}) {sym} ({r_code}))"), StageLType::B))
            }
            CoreBinaryOp::Lt | CoreBinaryOp::Le | CoreBinaryOp::Gt | CoreBinaryOp::Ge => {
                let sym = match op {
                    CoreBinaryOp::Lt => "<",
                    CoreBinaryOp::Le => "<=",
                    CoreBinaryOp::Gt => ">",
                    CoreBinaryOp::Ge => ">=",
                    _ => unreachable!("handled above"),
                };
                Ok((format!("(({l_code}) {sym} ({r_code}))"), StageLType::B))
            }
            CoreBinaryOp::Get => match &l_ty {
                StageLType::List(inner) => Ok((
                    format!("sl3_rt::list_get(&({l_code}), {r_code})"),
                    (**inner).clone(),
                )),
                StageLType::Buf(inner) => Ok((
                    format!("({l_code}).get({r_code})"),
                    (**inner).clone(),
                )),
                other => bail!("Invalid target '{other}' for 'get'"),
            },
            CoreBinaryOp::Concat => Ok((
                format!("sl3_rt::list_concat(&({l_code}), &({r_code}))"),
                l_ty,
            )),
            CoreBinaryOp::BufFill => {
                let r_elem = rust_type(&l_ty);
                Ok((
                    format!("sl3_rt::SlBuf::<{r_elem}>::filled({l_code}, {r_code})"),
                    StageLType::Buf(Box::new(l_ty)),
                ))
            }
            CoreBinaryOp::BufPush => Ok((
                format!("({l_code}).push({r_code})"),
                StageLType::Void,
            )),
            CoreBinaryOp::BufExtend => Ok((
                format!("({l_code}).extend_list(&({r_code}))"),
                StageLType::Void,
            )),
        }
    }

    fn emit_ternary(
        &mut self,
        op: CoreTernaryOp,
        first: &Expr,
        second: &Expr,
        third: &Expr,
        indent_level: usize,
    ) -> Result<(String, StageLType)> {
        let (c1, t1) = self.emit_expr(first, indent_level)?;
        if t1 == StageLType::Never {
            return Ok((c1, StageLType::Never));
        }
        let (c2, t2) = self.emit_expr(second, indent_level)?;
        if t2 == StageLType::Never {
            return Ok((
                prefix_prior_exprs(std::slice::from_ref(&c1), &c2),
                StageLType::Never,
            ));
        }
        let (c3, t3) = self.emit_expr(third, indent_level)?;
        if t3 == StageLType::Never {
            return Ok((prefix_prior_exprs(&[c1, c2], &c3), StageLType::Never));
        }
        match op {
            CoreTernaryOp::Slice => Ok((
                format!("sl3_rt::list_slice(&({c1}), {c2}, {c3})"),
                t1,
            )),
            CoreTernaryOp::BufSet => Ok((
                format!("({c1}).set({c2}, {c3})"),
                StageLType::Void,
            )),
        }
    }

    fn emit_conv(
        &mut self,
        inner: &Expr,
        target_ty: &StageLType,
        indent_level: usize,
    ) -> Result<(String, StageLType)> {
        let (in_code, in_ty) = self.emit_expr(inner, indent_level)?;
        if in_ty == StageLType::Never {
            return Ok((in_code, StageLType::Never));
        }
        if &in_ty == target_ty {
            return Ok((in_code, target_ty.clone()));
        }
        let r_target = rust_type(target_ty);
        if is_lossless_widening(&in_ty, target_ty) {
            Ok((format!("{r_target}::from({in_code})"), target_ty.clone()))
        } else {
            let tmp = self.next_tmp("cv");
            Ok((
                format!(
                    "{{ let Ok({tmp}) = {r_target}::try_from({in_code}) else {{ panic!(\"conv overflow\"); }}; {tmp} }}"
                ),
                target_ty.clone(),
            ))
        }
    }

    fn emit_bitcast(
        &mut self,
        inner: &Expr,
        target_ty: &StageLType,
        indent_level: usize,
    ) -> Result<(String, StageLType)> {
        let (in_code, in_ty) = self.emit_expr(inner, indent_level)?;
        if in_ty == StageLType::Never {
            return Ok((in_code, StageLType::Never));
        }
        if &in_ty == target_ty {
            return Ok((in_code, target_ty.clone()));
        }
        let r_target = rust_type(target_ty);
        Ok((
            format!("{r_target}::from_le_bytes(({in_code}).to_le_bytes())"),
            target_ty.clone(),
        ))
    }

    fn emit_direct_fn_tail_if(
        &mut self,
        expr: &Expr,
        expect_void: bool,
        indent_level: usize,
    ) -> Result<(String, StageLType)> {
        let Expr::If {
            cond,
            then_branch,
            else_branch,
            ..
        } = expr
        else {
            bail!("Internal error: emit_direct_fn_tail_if requires Expr::If");
        };
        let (cond_code, cond_ty) = self.emit_expr(cond, indent_level)?;
        if cond_ty == StageLType::Never {
            return Ok((cond_code, StageLType::Never));
        }
        let (then_block, then_ty) =
            self.emit_braced_branch(then_branch, expect_void, true, indent_level)?;
        let (else_code, else_ty) = if matches!(else_branch.as_ref(), Expr::If { .. }) {
            self.emit_direct_fn_tail_if(else_branch, expect_void, indent_level)?
        } else {
            self.emit_braced_branch(else_branch, expect_void, true, indent_level)?
        };
        let res_ty = match (then_ty == StageLType::Never, else_ty == StageLType::Never) {
            (true, true) => StageLType::Never,
            (true, false) => else_ty,
            (false, true) => then_ty,
            (false, false) => then_ty,
        };
        let cond_clean = strip_matched_outer_parens(&cond_code);
        Ok((
            format_if_expr(cond_clean, &then_block, &else_code, indent_level),
            res_ty,
        ))
    }
}

fn format_if_expr(
    cond_clean: &str,
    then_block: &str,
    else_code: &str,
    indent_level: usize,
) -> String {
    if then_block == else_code {
        if then_block == "{}" {
            return format!("{{ let _: bool = {cond_clean}; }}");
        }
        if let Some(inner) = then_block
            .strip_prefix('{')
            .and_then(|s| s.strip_suffix('}'))
        {
            let inner_ind = "    ".repeat(indent_level.saturating_add(1));
            return format!("{{\n{inner_ind}let _: bool = {cond_clean};{inner}}}");
        }
        return format!("{{ let _: bool = {cond_clean}; {then_block} }}");
    }
    if else_code == "{}" {
        format!("if {cond_clean} {then_block}")
    } else {
        format!("if {cond_clean} {then_block} else {else_code}")
    }
}

#[derive(Debug, Clone, Copy)]
enum BlockEmitMode {
    Value { expect_void: bool, is_fn_tail: bool },
    TailStep,
}

enum OperandEval {
    Ready(Vec<String>),
    Diverged(String),
}

fn prefix_prior_exprs(prior: &[String], diverging: &str) -> String {
    if prior.is_empty() {
        return diverging.to_owned();
    }
    let mut out = String::from("{ ");
    for p in prior {
        let _ = write!(out, "let _ = {p}; ");
    }
    out.push_str(diverging);
    out.push_str(" }");
    out
}

fn is_lossless_widening(from: &StageLType, to: &StageLType) -> bool {
    matches!(
        (from, to),
        (StageLType::U8, StageLType::U32 | StageLType::I32 | StageLType::U64 | StageLType::I64)
            | (StageLType::U32, StageLType::U64 | StageLType::I64)
            | (StageLType::I32, StageLType::I64)
    )
}

fn emit_int_lit(lit: TypedIntLit) -> String {
    match lit {
        TypedIntLit::U8(v) => format!("{v}u8"),
        TypedIntLit::I32(v) => {
            if v == i32::MIN {
                String::from("i32::MIN")
            } else {
                format!("{v}i32")
            }
        }
        TypedIntLit::U32(v) => format!("{v}u32"),
        TypedIntLit::I64(v) => {
            if v == i64::MIN {
                String::from("i64::MIN")
            } else {
                format!("{v}i64")
            }
        }
        TypedIntLit::U64(v) => format!("{v}u64"),
    }
}

fn format_u8_slice_lit(bytes: &[u8]) -> String {
    let items: Vec<String> = bytes.iter().map(|b| format!("{b}u8")).collect();
    format!("[{}]", items.join(", "))
}

fn strip_matched_outer_parens(expr: &str) -> &str {
    let mut cur = expr.trim();
    while let Some(inner) = cur.strip_prefix('(').and_then(|s| s.strip_suffix(')')) {
        let mut depth = 0usize;
        let mut closes_early = false;
        for ch in inner.chars() {
            match ch {
                '(' => depth = depth.saturating_add(1),
                ')' => {
                    if depth == 0 {
                        closes_early = true;
                        break;
                    }
                    depth = depth.saturating_sub(1);
                }
                _ => {}
            }
        }
        if closes_early || depth != 0 {
            break;
        }
        cur = inner;
    }
    cur
}

pub(super) struct BlockCapturePlan<'a> {
    pub(super) nested_routines: Vec<&'a RoutineDecl>,
    pub(super) local_captured_vars: HashMap<String, StageLType>,
    pub(super) all_captured_outer: HashSet<String>,
    pub(super) routine_captures: HashMap<String, HashSet<String>>,
    pub(super) mutated_locals: HashSet<String>,
}

fn stmt_ends_without_semi(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::If { .. } | Expr::While { .. } | Expr::ForEach { .. } | Expr::Block { .. }
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AssignState {
    Uninit,
    AssignedOnce,
    Multi,
}

fn join_assign(a: Option<AssignState>, b: Option<AssignState>) -> AssignState {
    match (a, b) {
        (Some(AssignState::Multi), _) | (_, Some(AssignState::Multi)) => AssignState::Multi,
        (Some(AssignState::AssignedOnce), _) | (_, Some(AssignState::AssignedOnce)) => {
            AssignState::AssignedOnce
        }
        _ => AssignState::Uninit,
    }
}

/// Returns `true` iff `expr` contains at least one tail-position `Expr::Call`
/// belonging to the current routine (excluding nested `RoutineDecl` bodies).
fn expr_has_tail_call(expr: &Expr, in_tail: bool) -> bool {
    match expr {
        Expr::Call { args, .. } => {
            in_tail || args.iter().any(|a| expr_has_tail_call(a, false))
        }
        Expr::Return { value: Some(v), .. } => expr_has_tail_call(v, true),
        Expr::Return { value: None, .. } => false,
        Expr::Block { stmts, .. } => {
            let mut expr_stmts = Vec::new();
            for item in stmts {
                if let BlockItem::Expr(e) = item {
                    expr_stmts.push(e);
                }
            }
            let count = expr_stmts.len();
            for (idx, stmt) in expr_stmts.iter().enumerate() {
                let is_last = idx.saturating_add(1) == count;
                match stmt {
                    Expr::RoutineDecl(_) => {}
                    Expr::Let { init: Some(ie), .. } => {
                        if expr_has_tail_call(ie, false) {
                            return true;
                        }
                    }
                    Expr::Let { init: None, .. } => {}
                    Expr::Const { value, .. } => {
                        if expr_has_tail_call(value, false) {
                            return true;
                        }
                    }
                    other => {
                        if expr_has_tail_call(other, in_tail && is_last) {
                            return true;
                        }
                    }
                }
            }
            false
        }
        Expr::If {
            cond,
            then_branch,
            else_branch,
            ..
        } => {
            expr_has_tail_call(cond, false)
                || expr_has_tail_call(then_branch, in_tail)
                || expr_has_tail_call(else_branch, in_tail)
        }
        Expr::While { cond, body, .. } => {
            expr_has_tail_call(cond, false) || expr_has_tail_call(body, false)
        }
        Expr::ForEach { list, body, .. } => {
            expr_has_tail_call(list, false) || expr_has_tail_call(body, false)
        }
        Expr::Set { value, .. }
        | Expr::Conv { expr: value, .. }
        | Expr::Bitcast { expr: value, .. }
        | Expr::Unary { operand: value, .. }
        | Expr::BufNew {
            capacity: value, ..
        } => expr_has_tail_call(value, false),
        Expr::List { elements, .. } => {
            elements.iter().any(|e| expr_has_tail_call(e, false))
        }
        Expr::Binary { left, right, .. } => {
            expr_has_tail_call(left, false) || expr_has_tail_call(right, false)
        }
        Expr::Ternary {
            first,
            second,
            third,
            ..
        } => {
            expr_has_tail_call(first, false)
                || expr_has_tail_call(second, false)
                || expr_has_tail_call(third, false)
        }
        Expr::RoutineDecl(_)
        | Expr::Bool { .. }
        | Expr::Int { .. }
        | Expr::Bytes { .. }
        | Expr::ValueRef { .. }
        | Expr::Let { .. }
        | Expr::Const { .. }
        | Expr::Break { .. }
        | Expr::Continue { .. }
        | Expr::Fault { .. } => false,
    }
}

pub(super) fn analyze_block_captures<'a>(
    stmts: &[&'a Expr],
    enclosing_routine_captures: &HashMap<String, Vec<String>>,
) -> BlockCapturePlan<'a> {
    let mut nested_routines = Vec::new();
    let mut local_decls: HashMap<String, StageLType> = HashMap::new();
    let mut assign_states: HashMap<String, AssignState> = HashMap::new();
    let no_shadow = HashSet::new();

    for stmt in stmts {
        match stmt {
            Expr::RoutineDecl(r) => nested_routines.push(&**r),
            Expr::Let { name, ty, init, .. } => {
                if let Some(ie) = init {
                    track_assign_expr(ie, &no_shadow, &mut assign_states, false);
                }
                local_decls.insert(name.clone(), ty.clone());
                let init_state = if init.is_some() {
                    AssignState::AssignedOnce
                } else {
                    AssignState::Uninit
                };
                assign_states.insert(name.clone(), init_state);
            }
            Expr::Const { name, ty, value, .. } => {
                track_assign_expr(value, &no_shadow, &mut assign_states, false);
                local_decls.insert(name.clone(), ty.clone());
            }
            other => {
                track_assign_expr(other, &no_shadow, &mut assign_states, false);
            }
        }
    }

    let mut mutated_locals = HashSet::new();
    for (name, st) in assign_states {
        if st == AssignState::Multi {
            mutated_locals.insert(name);
        }
    }

    // Collect variable references and sibling/enclosing routine calls per
    // nested routine, then compute transitive captures so callers forward all
    // required lexical bindings.
    let mut routine_captures: HashMap<String, HashSet<String>> = HashMap::new();
    let mut routine_calls: HashMap<String, HashSet<String>> = HashMap::new();
    let nested_names: HashSet<String> = nested_routines.iter().map(|r| r.name.clone()).collect();
    let mut callable_nested_names = nested_names.clone();
    for enc_name in enclosing_routine_captures.keys() {
        callable_nested_names.insert(enc_name.clone());
    }

    for r in &nested_routines {
        let mut bound = HashSet::new();
        for p in &r.params {
            bound.insert(p.name.clone());
        }
        let mut direct_vars = HashSet::new();
        collect_free_var_refs(&r.body, &bound, &mut direct_vars);
        let mut called = HashSet::new();
        collect_called_routine_names(&r.body, &callable_nested_names, &mut called);
        for callee in &called {
            if !nested_names.contains(callee)
                && let Some(enc_caps) = enclosing_routine_captures.get(callee)
            {
                for v in enc_caps {
                    if !bound.contains(v) {
                        direct_vars.insert(v.clone());
                    }
                }
            }
        }
        routine_captures.insert(r.name.clone(), direct_vars);
        routine_calls.insert(r.name.clone(), called);
    }

    loop {
        let mut changed = false;
        for r in &nested_routines {
            let Some(called_set) = routine_calls.get(&r.name) else {
                continue;
            };
            let callees: Vec<String> = called_set.iter().cloned().collect();
            for callee in callees {
                let Some(callee_set) = routine_captures.get(&callee) else {
                    continue;
                };
                let extra: Vec<String> = callee_set.iter().cloned().collect();
                if let Some(target_set) = routine_captures.get_mut(&r.name) {
                    for v in extra {
                        if target_set.insert(v) {
                            changed = true;
                        }
                    }
                }
            }
        }
        if !changed {
            break;
        }
    }

    let mut all_refs_in_direct_nested = HashSet::new();
    for vars in routine_captures.values() {
        for v in vars {
            all_refs_in_direct_nested.insert(v.clone());
        }
    }

    let mut local_captured_vars = HashMap::new();
    let mut all_captured_outer = HashSet::new();
    for var_name in all_refs_in_direct_nested {
        if let Some(ty) = local_decls.get(&var_name) {
            local_captured_vars.insert(var_name, ty.clone());
        } else {
            all_captured_outer.insert(var_name);
        }
    }

    // Also inspect child expressions (such as `if`/`do`/`while`/`foreach` blocks)
    // for nested routines declared in inner scopes that reference or mutate
    // local `let`/`const` bindings of THIS block, so they use `SlCap<T>` too.
    if !local_decls.is_empty() {
        let mut descendant_refs = HashSet::new();
        for stmt in stmts {
            if !matches!(stmt, Expr::RoutineDecl(_)) {
                collect_descendant_routine_var_refs(
                    stmt,
                    &callable_nested_names,
                    &routine_captures,
                    enclosing_routine_captures,
                    &mut descendant_refs,
                );
            }
        }
        for var_name in descendant_refs {
            if let Some(ty) = local_decls.get(&var_name) {
                local_captured_vars.insert(var_name, ty.clone());
            }
        }
    }

    BlockCapturePlan {
        nested_routines,
        local_captured_vars,
        all_captured_outer,
        routine_captures,
        mutated_locals,
    }
}

fn collect_descendant_routine_var_refs(
    expr: &Expr,
    callable_nested_names: &HashSet<String>,
    sibling_captures: &HashMap<String, HashSet<String>>,
    enclosing_captures: &HashMap<String, Vec<String>>,
    out: &mut HashSet<String>,
) {
    match expr {
        Expr::RoutineDecl(r) => {
            let mut bound = HashSet::new();
            for p in &r.params {
                bound.insert(p.name.clone());
            }
            collect_free_var_refs(&r.body, &bound, out);
            let mut called = HashSet::new();
            collect_called_routine_names(&r.body, callable_nested_names, &mut called);
            for callee in called {
                if let Some(sib_set) = sibling_captures.get(&callee) {
                    for v in sib_set {
                        if !bound.contains(v) {
                            out.insert(v.clone());
                        }
                    }
                } else if let Some(enc_vec) = enclosing_captures.get(&callee) {
                    for v in enc_vec {
                        if !bound.contains(v) {
                            out.insert(v.clone());
                        }
                    }
                }
            }
        }
        Expr::Set { value, .. }
        | Expr::Conv { expr: value, .. }
        | Expr::Bitcast { expr: value, .. }
        | Expr::Unary { operand: value, .. }
        | Expr::BufNew {
            capacity: value, ..
        }
        | Expr::Const { value, .. } => {
            collect_descendant_routine_var_refs(
                value,
                callable_nested_names,
                sibling_captures,
                enclosing_captures,
                out,
            );
        }
        Expr::Let { init: Some(e), .. } | Expr::Return { value: Some(e), .. } => {
            collect_descendant_routine_var_refs(
                e,
                callable_nested_names,
                sibling_captures,
                enclosing_captures,
                out,
            );
        }
        Expr::Call { args, .. } => {
            for a in args {
                collect_descendant_routine_var_refs(
                    a,
                    callable_nested_names,
                    sibling_captures,
                    enclosing_captures,
                    out,
                );
            }
        }
        Expr::List { elements, .. } => {
            for e in elements {
                collect_descendant_routine_var_refs(
                    e,
                    callable_nested_names,
                    sibling_captures,
                    enclosing_captures,
                    out,
                );
            }
        }
        Expr::Block { stmts, .. } => {
            for item in stmts {
                if let BlockItem::Expr(e) = item {
                    collect_descendant_routine_var_refs(
                        e,
                        callable_nested_names,
                        sibling_captures,
                        enclosing_captures,
                        out,
                    );
                }
            }
        }
        Expr::If {
            cond,
            then_branch,
            else_branch,
            ..
        } => {
            collect_descendant_routine_var_refs(
                cond,
                callable_nested_names,
                sibling_captures,
                enclosing_captures,
                out,
            );
            collect_descendant_routine_var_refs(
                then_branch,
                callable_nested_names,
                sibling_captures,
                enclosing_captures,
                out,
            );
            collect_descendant_routine_var_refs(
                else_branch,
                callable_nested_names,
                sibling_captures,
                enclosing_captures,
                out,
            );
        }
        Expr::While { cond, body, .. } => {
            collect_descendant_routine_var_refs(
                cond,
                callable_nested_names,
                sibling_captures,
                enclosing_captures,
                out,
            );
            collect_descendant_routine_var_refs(
                body,
                callable_nested_names,
                sibling_captures,
                enclosing_captures,
                out,
            );
        }
        Expr::ForEach { list, body, .. } => {
            collect_descendant_routine_var_refs(
                list,
                callable_nested_names,
                sibling_captures,
                enclosing_captures,
                out,
            );
            collect_descendant_routine_var_refs(
                body,
                callable_nested_names,
                sibling_captures,
                enclosing_captures,
                out,
            );
        }
        Expr::Binary { left, right, .. } => {
            collect_descendant_routine_var_refs(
                left,
                callable_nested_names,
                sibling_captures,
                enclosing_captures,
                out,
            );
            collect_descendant_routine_var_refs(
                right,
                callable_nested_names,
                sibling_captures,
                enclosing_captures,
                out,
            );
        }
        Expr::Ternary {
            first,
            second,
            third,
            ..
        } => {
            collect_descendant_routine_var_refs(
                first,
                callable_nested_names,
                sibling_captures,
                enclosing_captures,
                out,
            );
            collect_descendant_routine_var_refs(
                second,
                callable_nested_names,
                sibling_captures,
                enclosing_captures,
                out,
            );
            collect_descendant_routine_var_refs(
                third,
                callable_nested_names,
                sibling_captures,
                enclosing_captures,
                out,
            );
        }
        _ => {}
    }
}

fn collect_called_routine_names(
    expr: &Expr,
    sibling_names: &HashSet<String>,
    out: &mut HashSet<String>,
) {
    match expr {
        Expr::Call { target, args, .. } => {
            if let SymbolTarget::Ident(name) = target
                && sibling_names.contains(name)
            {
                out.insert(name.clone());
            }
            for a in args {
                collect_called_routine_names(a, sibling_names, out);
            }
        }
        Expr::Set { value, .. }
        | Expr::Conv { expr: value, .. }
        | Expr::Bitcast { expr: value, .. }
        | Expr::Unary { operand: value, .. }
        | Expr::BufNew {
            capacity: value, ..
        }
        | Expr::Const { value, .. } => {
            collect_called_routine_names(value, sibling_names, out);
        }
        Expr::Let { init: Some(e), .. } | Expr::Return { value: Some(e), .. } => {
            collect_called_routine_names(e, sibling_names, out);
        }
        Expr::List { elements, .. } => {
            for e in elements {
                collect_called_routine_names(e, sibling_names, out);
            }
        }
        Expr::Block { stmts, .. } => {
            for item in stmts {
                if let BlockItem::Expr(e) = item {
                    collect_called_routine_names(e, sibling_names, out);
                }
            }
        }
        Expr::If {
            cond,
            then_branch,
            else_branch,
            ..
        } => {
            collect_called_routine_names(cond, sibling_names, out);
            collect_called_routine_names(then_branch, sibling_names, out);
            collect_called_routine_names(else_branch, sibling_names, out);
        }
        Expr::While { cond, body, .. } => {
            collect_called_routine_names(cond, sibling_names, out);
            collect_called_routine_names(body, sibling_names, out);
        }
        Expr::ForEach { list, body, .. } => {
            collect_called_routine_names(list, sibling_names, out);
            collect_called_routine_names(body, sibling_names, out);
        }
        Expr::Binary { left, right, .. } => {
            collect_called_routine_names(left, sibling_names, out);
            collect_called_routine_names(right, sibling_names, out);
        }
        Expr::Ternary {
            first,
            second,
            third,
            ..
        } => {
            collect_called_routine_names(first, sibling_names, out);
            collect_called_routine_names(second, sibling_names, out);
            collect_called_routine_names(third, sibling_names, out);
        }
        Expr::RoutineDecl(r) => collect_called_routine_names(&r.body, sibling_names, out),
        _ => {}
    }
}

fn track_assign_expr(
    expr: &Expr,
    shadowed: &HashSet<String>,
    states: &mut HashMap<String, AssignState>,
    in_loop: bool,
) {
    match expr {
        Expr::Set { name, value, .. } => {
            track_assign_expr(value, shadowed, states, in_loop);
            if !shadowed.contains(name)
                && let Some(st) = states.get_mut(name)
            {
                if in_loop || *st != AssignState::Uninit {
                    *st = AssignState::Multi;
                } else {
                    *st = AssignState::AssignedOnce;
                }
            }
        }
        Expr::List { elements, .. } => {
            for e in elements {
                track_assign_expr(e, shadowed, states, in_loop);
            }
        }
        Expr::BufNew { capacity, .. }
        | Expr::Conv { expr: capacity, .. }
        | Expr::Bitcast { expr: capacity, .. }
        | Expr::Unary {
            operand: capacity, ..
        } => {
            track_assign_expr(capacity, shadowed, states, in_loop);
        }
        Expr::Call { args, .. } => {
            for a in args {
                track_assign_expr(a, shadowed, states, in_loop);
            }
        }
        Expr::Block { stmts, .. } => {
            let mut inner_shadowed = shadowed.clone();
            for item in stmts {
                if let BlockItem::Expr(e) = item {
                    match e {
                        Expr::Let { name, init, .. } => {
                            if let Some(ie) = init {
                                track_assign_expr(ie, &inner_shadowed, states, in_loop);
                            }
                            inner_shadowed.insert(name.clone());
                        }
                        Expr::Const { name, value, .. } => {
                            track_assign_expr(value, &inner_shadowed, states, in_loop);
                            inner_shadowed.insert(name.clone());
                        }
                        Expr::RoutineDecl(_) => {}
                        other => track_assign_expr(other, &inner_shadowed, states, in_loop),
                    }
                }
            }
        }
        Expr::If {
            cond,
            then_branch,
            else_branch,
            ..
        } => {
            track_assign_expr(cond, shadowed, states, in_loop);
            let mut then_s = states.clone();
            track_assign_expr(then_branch, shadowed, &mut then_s, in_loop);
            let mut else_s = states.clone();
            track_assign_expr(else_branch, shadowed, &mut else_s, in_loop);
            for (k, st) in states.iter_mut() {
                *st = join_assign(then_s.get(k).copied(), else_s.get(k).copied());
            }
        }
        Expr::While { cond, body, .. } => {
            track_assign_expr(cond, shadowed, states, true);
            track_assign_expr(body, shadowed, states, true);
        }
        Expr::ForEach {
            elem_name,
            list,
            body,
            ..
        } => {
            track_assign_expr(list, shadowed, states, in_loop);
            let mut body_shadowed = shadowed.clone();
            body_shadowed.insert(elem_name.clone());
            track_assign_expr(body, &body_shadowed, states, true);
        }
        Expr::Return { value: Some(e), .. } => {
            track_assign_expr(e, shadowed, states, in_loop);
        }
        Expr::Binary { left, right, .. } => {
            track_assign_expr(left, shadowed, states, in_loop);
            track_assign_expr(right, shadowed, states, in_loop);
        }
        Expr::Ternary {
            first,
            second,
            third,
            ..
        } => {
            track_assign_expr(first, shadowed, states, in_loop);
            track_assign_expr(second, shadowed, states, in_loop);
            track_assign_expr(third, shadowed, states, in_loop);
        }
        Expr::Let { .. } | Expr::Const { .. } | Expr::RoutineDecl(_) => {}
        _ => {}
    }
}

fn collect_free_var_refs(
    expr: &Expr,
    bound: &HashSet<String>,
    out: &mut HashSet<String>,
) {
    match expr {
        Expr::ValueRef { target: SymbolTarget::Ident(n), .. } => {
            if !bound.contains(n) {
                out.insert(n.clone());
            }
        }
        Expr::Set { name, value, .. } => {
            if !bound.contains(name) {
                out.insert(name.clone());
            }
            collect_free_var_refs(value, bound, out);
        }
        Expr::List { elements, .. } => {
            for e in elements {
                collect_free_var_refs(e, bound, out);
            }
        }
        Expr::BufNew { capacity, .. }
        | Expr::Conv { expr: capacity, .. }
        | Expr::Bitcast { expr: capacity, .. }
        | Expr::Unary { operand: capacity, .. } => {
            collect_free_var_refs(capacity, bound, out);
        }
        Expr::Return { value: Some(v), .. } => {
            collect_free_var_refs(v, bound, out);
        }
        Expr::Call { args, .. } => {
            for a in args {
                collect_free_var_refs(a, bound, out);
            }
        }
        Expr::Block { stmts, .. } => {
            let mut inner_bound = bound.clone();
            for item in stmts {
                if let BlockItem::Expr(e) = item {
                    match e {
                        Expr::Let { name, init, .. } => {
                            if let Some(ie) = init {
                                collect_free_var_refs(ie, &inner_bound, out);
                            }
                            inner_bound.insert(name.clone());
                        }
                        Expr::Const { name, value, .. } => {
                            collect_free_var_refs(value, &inner_bound, out);
                            inner_bound.insert(name.clone());
                        }
                        Expr::RoutineDecl(r) => {
                            let mut r_bound = inner_bound.clone();
                            for p in &r.params {
                                r_bound.insert(p.name.clone());
                            }
                            collect_free_var_refs(&r.body, &r_bound, out);
                        }
                        other => collect_free_var_refs(other, &inner_bound, out),
                    }
                }
            }
        }
        Expr::If { cond, then_branch, else_branch, .. } => {
            collect_free_var_refs(cond, bound, out);
            collect_free_var_refs(then_branch, bound, out);
            collect_free_var_refs(else_branch, bound, out);
        }
        Expr::While { cond, body, .. } => {
            collect_free_var_refs(cond, bound, out);
            collect_free_var_refs(body, bound, out);
        }
        Expr::ForEach { elem_name, list, body, .. } => {
            collect_free_var_refs(list, bound, out);
            let mut loop_bound = bound.clone();
            loop_bound.insert(elem_name.clone());
            collect_free_var_refs(body, &loop_bound, out);
        }
        Expr::Binary { left, right, .. } => {
            collect_free_var_refs(left, bound, out);
            collect_free_var_refs(right, bound, out);
        }
        Expr::Ternary { first, second, third, .. } => {
            collect_free_var_refs(first, bound, out);
            collect_free_var_refs(second, bound, out);
            collect_free_var_refs(third, bound, out);
        }
        Expr::RoutineDecl(r) => {
            let mut r_bound = bound.clone();
            for p in &r.params {
                r_bound.insert(p.name.clone());
            }
            collect_free_var_refs(&r.body, &r_bound, out);
        }
        _ => {}
    }
}

pub(super) fn topo_sort_top_values(items: &[TopDecl]) -> Vec<&TopValueDecl> {
    let mut vals = Vec::new();
    let mut by_name = HashMap::new();
    for item in items {
        if let TopDecl::Value(v) = item {
            by_name.insert(v.name.as_str(), vals.len());
            vals.push(v);
        }
    }
    let mut visited = vec![false; vals.len()];
    let mut ordered = Vec::with_capacity(vals.len());

    fn visit<'a>(
        idx: usize,
        vals: &[&'a TopValueDecl],
        by_name: &HashMap<&'a str, usize>,
        visited: &mut [bool],
        ordered: &mut Vec<&'a TopValueDecl>,
    ) {
        if visited.get(idx).copied() == Some(true) {
            return;
        }
        if let Some(slot) = visited.get_mut(idx) {
            *slot = true;
        }
        if let Some(v) = vals.get(idx) {
            let mut deps = HashSet::new();
            collect_free_var_refs(&v.value, &HashSet::new(), &mut deps);
            for d in deps {
                if let Some(&dep_idx) = by_name.get(d.as_str()) {
                    visit(dep_idx, vals, by_name, visited, ordered);
                }
            }
            ordered.push(v);
        }
    }

    for i in 0..vals.len() {
        visit(i, &vals, &by_name, &mut visited, &mut ordered);
    }
    ordered
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
    use super::super::eval::{
        EvalConfig, StageLWorkspaceVm, run_module_inline_tests,
    };
    use super::super::validator::{ModuleInput, parse_and_validate_workspace};
    use std::process::Command;

    struct TestTempDir(std::path::PathBuf);

    impl TestTempDir {
        fn new(prefix: &str) -> Self {
            static SEQ: std::sync::atomic::AtomicUsize =
                std::sync::atomic::AtomicUsize::new(0);
            let n = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let path = std::env::temp_dir()
                .join(format!("{prefix}_{}_{n}", std::process::id()));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).expect("create test temp dir");
            Self(path)
        }

        fn path(&self) -> &std::path::Path {
            &self.0
        }
    }

    impl Drop for TestTempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn run_rustc_and_clippy_smoke(rust_src: &str, execute_binary: bool) {
        let dir = TestTempDir::new("ctb_stagel3_rust_smoke");
        let rs_path = dir.path().join("smoke.rs");
        std::fs::write(&rs_path, rust_src).expect("write generated rust");

        let clippy_out = Command::new("clippy-driver")
            .arg("--edition")
            .arg("2021")
            .arg(if execute_binary { "--crate-type=bin" } else { "--crate-type=lib" })
            .arg("-D")
            .arg("warnings")
            .arg("-D")
            .arg("clippy::arithmetic_side_effects")
            .arg("-D")
            .arg("clippy::indexing_slicing")
            .arg("-D")
            .arg("clippy::unwrap_used")
            .arg("-D")
            .arg("clippy::expect_used")
            .arg("-o")
            .arg(dir.path().join("smoke.clippy"))
            .arg(&rs_path)
            .output()
            .expect("invoke clippy-driver");
        assert!(
            clippy_out.status.success(),
            "clippy-driver failed:\n{}\n--- SOURCE ---\n{rust_src}",
            String::from_utf8_lossy(&clippy_out.stderr)
        );

        if execute_binary {
            for opt_level in ["0", "2"] {
                let bin_path = dir.path().join(format!("smoke_bin_o{opt_level}"));
                let compile_out = Command::new("rustc")
                    .arg("--edition")
                    .arg("2021")
                    .arg("-C")
                    .arg(format!("opt-level={opt_level}"))
                    .arg("-D")
                    .arg("warnings")
                    .arg("-o")
                    .arg(&bin_path)
                    .arg(&rs_path)
                    .output()
                    .expect("invoke rustc");
                assert!(
                    compile_out.status.success(),
                    "rustc (opt-level={opt_level}) failed:\n{}\n--- SOURCE ---\n{rust_src}",
                    String::from_utf8_lossy(&compile_out.stderr)
                );

                // Run under a 512 KiB native stack cap (`ulimit -s 512`) so any
                // non-trampolined deep tail call immediately overflows even on
                // machines with large default OS stacks.
                let exec_out = Command::new("sh")
                    .arg("-c")
                    .arg("ulimit -s 512 && exec \"$1\"")
                    .arg("--")
                    .arg(&bin_path)
                    .output()
                    .expect("run compiled rust smoke binary");
                assert!(
                    exec_out.status.success(),
                    "compiled rust binary (opt-level={opt_level}) failed:\nstdout: {}\nstderr: {}",
                    String::from_utf8_lossy(&exec_out.stdout),
                    String::from_utf8_lossy(&exec_out.stderr)
                );
            }
        } else {
            let rlib_path = dir.path().join("libsmoke.rlib");
            let compile_out = Command::new("rustc")
                .arg("--edition")
                .arg("2021")
                .arg("--crate-type=lib")
                .arg("-C")
                .arg("opt-level=0")
                .arg("-D")
                .arg("warnings")
                .arg("-o")
                .arg(&rlib_path)
                .arg(&rs_path)
                .output()
                .expect("invoke rustc lib");
            assert!(
                compile_out.status.success(),
                "rustc lib failed:\n{}\n--- SOURCE ---\n{rust_src}",
                String::from_utf8_lossy(&compile_out.stderr)
            );
        }
    }

    #[crate::ctb_test]
    fn test_rust_codegen_arithmetic_control_flow_captures_buffers_and_tests() {
        let src = r#"
const u32/forwardRef add u32/baseOffset u32:2.
const u32/baseOffset u32:40.

v/u32/arithAndMin i32/x u32/y
  test smoke
    in i32:-16 u32:10
    out u32:102.
  let u32/branchOut.
  if gt u32/y u32:5 do
    set u32/branchOut u32/forwardRef.
  :: do
    fault.
  ::.
  let i32/shifted shr i32/x u32:2.
  let u32/negSign.
  let i64/minWide i64:-9223372036854775808.
  let i32/minSmall i32:-2147483648.
  if lt i64/minWide i64:0 do
    set u32/negSign add popcnt bitcast i32/minSmall u32 bitcast i32/shifted u32.
  :: do
    set u32/negSign u32:0.
  ::.
  : match Rust keyword identifiers too!
  let u32/match add u32/branchOut u32:64.
  : wrapAdd(1 + 4294967292) overflows -4u32 cleanly when using wrapSub:
  wrapSub wrapAdd u32/match u32/negSign wrapAdd u32:1 u32:0.

v/l/u8/bufWorkflow u8/fillByte
  test ok
    in u8:65
    out b"AAA!".
  let buf/u8/buf1 bufFill u8/fillByte u64:4.
  bufSet buf/u8/buf1 u64:3 u8:33.
  bufFreeze buf/u8/buf1.

v/u64/trapFrozen
  test frozen
    in.
    out fault.
  let buf/u8/b2 bufNew u8 u64:2.
  let l/u8/frozen bufFreeze buf/u8/b2.
  bufSet buf/u8/b2 u64:0 u8:77.
  len l/u8/frozen.

v/u32/capturesAndLoops u32/limit
  test total
    in u32:4
    out u32:24.
  let u32/acc u32:0.
  v/void/addAcc u32/d :: set u32/acc add u32/acc u32/d.
  v/u32/factAcc u32/n
    if le u32/n u32:1 u32:1 add u32/n factAcc sub u32/n u32:1.
  let u32/i u32:0.
  while lt u32/i u32/limit do
    set u32/i add u32/i u32:1.
    if eq u32/i u32:2 continue do ::.
    addAcc u32/i.
  ::.
  let l/u32/xs l/u32: u32:2 u32:4 u32:6 ::.
  foreach u32/el slice l/u32/xs u64:1 u64:3 do
    addAcc u32/el.
  ::.
  : i=1,3,4 -> 8; slice [4,6] -> 10; factAcc(3) -> 6 => 24
  add u32/acc factAcc u32:3.

v/void/voidReturn u32/x
  test ok
    in u32:5
    out.
  if eq u32/x u32:0 fault do ::.
"#;
        // Verify StageL 3 VM agrees first.
        let vm_report = run_module_inline_tests(src).expect("VM inline tests");
        assert_eq!(vm_report.ensure_all_passed().unwrap(), 5);

        let rust_src = transpile_sl3_source_to_rust(
            src,
            &RustTranspileOptions {
                standalone_smoke_harness: true,
                emit_test_runner: true,
            },
        )
        .expect("transpile to Rust");

        run_rustc_and_clippy_smoke(&rust_src, true);
    }

    #[crate::ctb_test]
    fn test_rust_codegen_workspace_imports_aliases_and_dc_calls() {
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
  dc/256 timesTwo u32/inputVal.
"#,
            },
        ];

        let ws = parse_and_validate_workspace(&inputs).expect("validate ws");
        let rust_src = transpile_workspace_to_rust(
            &ws,
            &RustTranspileOptions {
                standalone_smoke_harness: true,
                emit_test_runner: true,
            },
        )
        .expect("transpile workspace");

        run_rustc_and_clippy_smoke(&rust_src, true);
    }

    #[crate::ctb_test]
    fn test_rust_codegen_rejects_invalid_module_and_unlinked_imports() {
        let bad = "v/u32/bad\n  let u32/x.\n  u32/x.\n";
        let err = transpile_sl3_source_to_rust(
            bad,
            &RustTranspileOptions::default(),
        )
        .expect_err("uninitialized variable must fail validation");
        assert!(err.to_string().contains("definitely") || err.to_string().contains("uninitialized"));

        let inputs = [
            ModuleInput {
                source_path: Some("lib/a.sl3"),
                source: "ns lib/a.\npub u32/k u32:1.\n",
            },
            ModuleInput {
                source_path: Some("app/b.sl3"),
                source: "ns app/b.\nuse lib/a: k ::.\npub v/u32/getK :: u32/k.\n",
            },
        ];
        let ws = parse_and_validate_workspace(&inputs).expect("ws");
        let err_single = transpile_validated_module_to_rust(
            &ws.modules[1],
            &RustTranspileOptions::default(),
        )
        .expect_err("single module with imports must require workspace transpiler");
        assert!(err_single.to_string().contains("transpile_workspace_to_rust"));
    }

    #[crate::ctb_test]
    fn test_rust_codegen_unoptimized_deep_tail_trampolines_and_captures() {
        let src = r#"
: Self tail recursion returning u64
v/u64/sumDown u32/cnt u64/acc
  test deep120k
    in u32:120000 u64:0
    out u64:7200060000.
  if eq u32/cnt u32:0
    u64/acc
    sumDown sub u32/cnt u32:1 add u64/acc conv u32/cnt u64.

: Non-step leaf routine tail-called at end of 3-way mutual recursion
v/u32/directLeaf u32/val :: add u32/val u32:7.

v/u32/triA u32/cnt u32/acc
  test deepTri120k
    in u32:120000 u32:0
    out u32:240007.
  if eq u32/cnt u32:0
    directLeaf u32/acc
    triB sub u32/cnt u32:1 add u32/acc u32:1.

v/u32/triB u32/cnt u32/acc
  if eq u32/cnt u32:0
    directLeaf u32/acc
    triC sub u32/cnt u32:1 add u32/acc u32:2.

v/u32/triC u32/cnt u32/acc
  if eq u32/cnt u32:0
    directLeaf u32/acc
    triA sub u32/cnt u32:1 add u32/acc u32:3.

: Tail call via 'return CALL' inside while and foreach loops
v/u32/loopEarlyTail u32/limit
  test loopReturnTail
    in u32:90000
    out u32:180030.
  let u32/idx u32:0.
  while lt u32/idx u32:5 do
    set u32/idx add u32/idx u32:1.
    if eq u32/idx u32:3 do
      foreach u32/item l/u32: u32:10 u32:20 :: do
        if eq u32/item u32:20 do
          return triA u32/limit add u32/idx u32/item.
        :: do ::.
      ::.
    :: do ::.
  ::.
  u32:0.

: Top-level -> nested and sibling mutual tail recursion with SlCap, ImmVal, l/u8, and buf/u8
v/l/u8/nestedPingPong u32/steps u8/tagByte
  test pingPong100k
    in u32:100000 u8:33
    out l/u8: u8:79 u8:75 u8:58 u8:33 u8:153 ::.
  let u32/sharedHits u32:0.
  let buf/u8/outBuf bufNew u8 u64:6.
  const l/u8/prefixBytes b"OK:".
  bufExtend buf/u8/outBuf l/u8/prefixBytes.
  v/l/u8/pingStep u32/leftCount
    if eq u32/leftCount u32:0 do
      bufPush buf/u8/outBuf u8/tagByte.
      bufPush buf/u8/outBuf conv rem u32/sharedHits u32:251 u8.
      bufFreeze buf/u8/outBuf.
    :: do
      set u32/sharedHits add u32/sharedHits u32:1.
      pongStep sub u32/leftCount u32:1.
    ::.
  v/l/u8/pongStep u32/leftCount
    if eq u32/leftCount u32:0 do
      pingStep u32:0.
    :: do
      set u32/sharedHits add u32/sharedHits u32:2.
      pingStep sub u32/leftCount u32:1.
    ::.
  pingStep u32/steps.

: Child if-block nested routine mutating outer let + doubly-nested routine tail-calling enclosing nested routine
v/u32/enclosingAndChildBlockCaps u32/baseVal u32/steps
  test doublyNested90k
    in u32:5 u32:90000
    out u32:90015.
  let u32/tally u32:0.
  if gt u32/baseVal u32:0 do
    v/void/bumpFromIfBranch u32/deltaVal :: set u32/tally add u32/tally u32/deltaVal.
    bumpFromIfBranch u32:10.
  :: do
    set u32/tally u32:1.
  ::.
  v/u32/outerLoop u32/cnt
    if eq u32/cnt u32:0
      add u32/baseVal u32/tally
      do
        set u32/tally add u32/tally u32:1.
        v/u32/innerStep u32/nextCnt :: outerLoop sub u32/nextCnt u32:1.
        innerStep u32/cnt.
      ::.
  outerLoop u32/steps.

: Void-returning deep mutual tail recursion (both top-level and nested side-effecting)
v/void/voidStepA u32/cnt
  test void100k
    in u32:100000
    out.
  if eq u32/cnt u32:0 do :: do
    voidStepB sub u32/cnt u32:1.
  ::.

v/void/voidStepB u32/cnt
  if eq u32/cnt u32:0 do
    return.
  :: do
    voidStepA sub u32/cnt u32:1.
  ::.

v/u32/verifyVoidTail u32/steps
  test ticks100k
    in u32:100000
    out u32:100000.
  let u32/ticks u32:0.
  v/void/voidTickA u32/cnt
    if eq u32/cnt u32:0 do :: do
      set u32/ticks add u32/ticks u32:1.
      voidTickB sub u32/cnt u32:1.
    ::.
  v/void/voidTickB u32/cnt
    if eq u32/cnt u32:0 do :: do
      set u32/ticks add u32/ticks u32:1.
      voidTickA sub u32/cnt u32:1.
    ::.
  voidTickA u32/steps.
  u32/ticks.

: Never-returning deep mutual tail recursion and Value -> Never tail transition
v/never/haltNow u32/errCode :: fault.

v/never/neverSpinA u32/cnt
  test deepNeverFault
    in u32:90000
    out fault.
  if eq u32/cnt u32:0
    haltNow u32:1
    neverSpinB sub u32/cnt u32:1.

v/never/neverSpinB u32/cnt
  if eq u32/cnt u32:0
    haltNow u32:2
    neverSpinA sub u32/cnt u32:1.

v/u32/maybeNeverTail u32/modeFlag
  test normalBranch
    in u32:0
    out u32:77
  test neverBranch
    in u32:1
    out fault.
  if eq u32/modeFlag u32:0
    u32:77
    neverSpinA u32:80000.

: 10-parameter tail recursion exercising boxed tuple variant payloads (>8 fields)
v/u32/wideTail10 u32/cnt u32/a1 u32/a2 u32/a3 u32/a4 u32/a5 u32/a6 u32/a7 u32/a8 u32/acc
  test wide80k
    in u32:80000 u32:3 u32:0 u32:0 u32:0 u32:0 u32:0 u32:0 u32:4 u32:0
    out u32:80007.
  if eq u32/cnt u32:0
    add u32/acc add u32/a1 u32/a8
    wideTail10 sub u32/cnt u32:1 u32/a2 u32/a3 u32/a4 u32/a5 u32/a6 u32/a7 u32/a8 u32/a1 add u32/acc u32:1.

: Diverging ('never') operands inside conv, bitcast, list, binary, and call arguments
v/u32/divergingExprSmoke u32/which
  test okCase
    in u32:0
    out u32:19
  test trapCase
    in u32:2
    out fault.
  if eq u32/which u32:0
    u32:19
    if eq u32/which u32:1
      conv haltNow u32:1 u32
      if eq u32/which u32:2
        add u32:5 bitcast haltNow u32:2 u32
        if eq u32/which u32:3
          directLeaf haltNow u32:3
          do
            let l/u32/deadList l/u32: u32:1 haltNow u32:4 u32:2 ::.
            get l/u32/deadList u64:0.
          ::.
"#;

        let ws = parse_and_validate_workspace(&[ModuleInput {
            source_path: None,
            source: src,
        }])
        .expect("validate tail-call module");
        let vm = StageLWorkspaceVm::from_validated_workspace(
            &ws,
            EvalConfig {
                compute_quota: 50_000_000,
                max_call_depth: 64,
            },
        )
        .expect("init VM");
        let vm_report = vm.run_all_tests(&ws).expect("run VM inline tests");
        assert_eq!(vm_report.ensure_all_passed().unwrap(), 13);

        let rust_src = transpile_workspace_to_rust(
            &ws,
            &RustTranspileOptions {
                standalone_smoke_harness: true,
                emit_test_runner: true,
            },
        )
        .expect("transpile tail-call module to Rust");

        assert!(rust_src.contains("fn sl_trampoline_"));
        assert!(rust_src.contains("fn sl_trampoline_never"));
        run_rustc_and_clippy_smoke(&rust_src, true);
    }

    #[crate::ctb_test]
    fn test_rust_codegen_cross_module_tail_chains_and_strict_lib_mode() {
        let inputs = [
            ModuleInput {
                source_path: Some("chain/core.sl3"),
                source: r#"ns chain/core.

pub v/u32/coreEven u32/cnt u32/acc
  if eq u32/cnt u32:0
    u32/acc
    coreOdd sub u32/cnt u32:1 add u32/acc u32:2.

pub dc/256 v/u32/coreOdd u32/cnt u32/acc
  if eq u32/cnt u32:0
    u32/acc
    coreEven sub u32/cnt u32:1 add u32/acc u32:1.
"#,
            },
            ModuleInput {
                source_path: Some("chain/mid.sl3"),
                source: r#"ns chain/mid.
use chain/core: coreEven as stepEven dc/256 ::.

pub dc/257 v/u32/bounceMid u32/cnt u32/acc
  if eq rem u32/cnt u32:2 u32:0
    stepEven u32/cnt u32/acc
    dc/256 u32/cnt u32/acc.
"#,
            },
            ModuleInput {
                source_path: Some("chain/entry.sl3"),
                source: r#"ns chain/entry.
use chain/mid: bounceMid as launchChain dc/257 ::.

pub v/u32/runCrossModuleTail u32/cnt
  test even100k
    in u32:100000
    out u32:150000
  test odd100k1
    in u32:100001
    out u32:150001.
  if eq rem u32/cnt u32:2 u32:0
    launchChain u32/cnt u32:0
    dc/257 u32/cnt u32:0.
"#,
            },
        ];

        let ws = parse_and_validate_workspace(&inputs).expect("validate cross-module chain");
        let vm = StageLWorkspaceVm::from_validated_workspace(
            &ws,
            EvalConfig {
                compute_quota: 20_000_000,
                max_call_depth: 32,
            },
        )
        .expect("init workspace VM");
        let vm_report = vm.run_all_tests(&ws).expect("VM workspace tests");
        assert_eq!(vm_report.ensure_all_passed().unwrap(), 2);

        let rust_ws = transpile_workspace_to_rust(
            &ws,
            &RustTranspileOptions {
                standalone_smoke_harness: true,
                emit_test_runner: true,
            },
        )
        .expect("transpile workspace");
        run_rustc_and_clippy_smoke(&rust_ws, true);

        // Also verify strict library output (`standalone_smoke_harness: false`,
        // no crate-level `#![allow(...)]` attributes) compiles cleanly under
        // both `clippy-driver -D warnings` and `rustc -C opt-level=0 -D warnings`.
        let strict_lib_sl3 = r#"ns clean/strict_lib.

pub v/u32/count_down u32/steps u32/acc
  if eq u32/steps u32:0
    u32/acc
    count_down_alt sub u32/steps u32:1 add u32/acc u32:1.

pub v/u32/count_down_alt u32/steps u32/acc
  if eq u32/steps u32:0
    do
      v/u32/nested_finish u32/cur
        if eq u32/cur u32:0 u32/acc nested_finish sub u32/cur u32:1.
      nested_finish u32:3.
    ::
    count_down sub u32/steps u32:1 add u32/acc u32:2.

pub v/void/spin_void u32/steps
  if eq u32/steps u32:0 do :: do
    spin_void sub u32/steps u32:1.
  ::.

pub v/never/abort_now :: fault.

pub v/u32/guard_or_abort u32/val
  if eq u32/val u32:0
    abort_now
    count_down u32/val u32:0.
"#;
        let strict_rust = transpile_sl3_source_to_rust(
            strict_lib_sl3,
            &RustTranspileOptions {
                standalone_smoke_harness: false,
                emit_test_runner: false,
            },
        )
        .expect("transpile strict lib");
        assert!(!strict_rust.contains("#![allow("));
        run_rustc_and_clippy_smoke(&strict_rust, false);
    }
}
