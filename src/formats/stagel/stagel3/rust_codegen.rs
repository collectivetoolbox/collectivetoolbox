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
//! check), explicit capture cells for closures over `let` bindings, and an
//! optional inline `test` runner catching deterministic `fault` traps.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::ast::{
    BlockItem, CoreBinaryOp, CoreTernaryOp, CoreUnaryOp, Expr, Module, Param,
    RoutineDecl, Span, StageLType, SymbolTarget, TestExpected, TopDecl,
    TopValueDecl, TypedIntLit,
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
    let mut ns_to_mod_ident: HashMap<String, String> = HashMap::new();
    let mut mod_idents = Vec::with_capacity(workspace.modules.len());

    for (idx, vm) in workspace.modules.iter().enumerate() {
        let ident = if let Some(ns) = &vm.namespace_key {
            format!("sl_mod_{idx}_{}", sanitize_ident(ns))
        } else {
            format!("sl_mod_anon_{idx}")
        };
        if let Some(ns) = &vm.namespace_key {
            ns_to_mod_ident.insert(ns.clone(), ident.clone());
        }
        mod_idents.push(ident);
    }

    let mut out = String::new();
    emit_file_header(&mut out, options)?;
    emit_runtime_prelude(&mut out)?;

    let mut test_calls = Vec::new();
    for (idx, vm) in workspace.modules.iter().enumerate() {
        let mod_ident = mod_idents
            .get(idx)
            .context("Internal module index mismatch")?;
        let mut emitter = ModuleEmitter::new(vm, mod_ident, &ns_to_mod_ident)?;
        emitter.emit_module(&mut out, options.emit_test_runner)?;
        if options.emit_test_runner {
            for fn_name in emitter.emitted_test_fns {
                test_calls.push(format!("{mod_ident}::{fn_name}()"));
            }
        }
    }

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

fn emit_runtime_prelude(out: &mut String) -> Result<()> {
    writeln!(out, "mod sl3_rt {{")?;
    writeln!(out, "    use std::cell::RefCell;")?;
    writeln!(out, "    use std::rc::Rc;")?;
    writeln!(out, "    use std::sync::Arc;")?;
    writeln!(out)?;
    writeln!(out, "    pub type SlList<T> = Arc<[T]>;")?;
    writeln!(out, "    pub type SlCap<T> = Rc<RefCell<Option<T>>>;")?;
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

fn sanitize_ident(raw: &str) -> String {
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

#[derive(Debug, Clone)]
enum VarStorage {
    /// Stack variable (`let x` / param / foreach / non-captured local `const`).
    Direct { rust_name: String, ty: StageLType },
    /// Heap `SlCap<T>` shared across nested routine captures (`is_ref` when
    /// received as `&SlCap<T>` parameter inside a nested routine).
    CapCell {
        rust_name: String,
        ty: StageLType,
        is_ref: bool,
    },
    /// Immutable value passed by reference `&T` into a nested routine.
    ByRef { rust_name: String, ty: StageLType },
}

impl VarStorage {
    fn ty(&self) -> &StageLType {
        match self {
            Self::Direct { ty, .. }
            | Self::CapCell { ty, .. }
            | Self::ByRef { ty, .. } => ty,
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
            Self::CapCell {
                rust_name, is_ref, ..
            } => {
                if *is_ref {
                    format!("sl3_rt::cap_get({rust_name}, {sl_name:?})")
                } else {
                    format!("sl3_rt::cap_get(&{rust_name}, {sl_name:?})")
                }
            }
            Self::ByRef { rust_name, ty } => {
                if rust_type_copy_cheap(ty) {
                    format!("*{rust_name}")
                } else {
                    format!("{rust_name}.clone()")
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
enum CapturePassKind {
    CapCell,
    ImmRef,
}

#[derive(Debug, Clone)]
struct CapturedArg {
    sl_name: String,
    rust_param: String,
    ty: StageLType,
    kind: CapturePassKind,
}

#[derive(Debug, Clone)]
enum RoutineTarget {
    TopLevel { rust_path: String, sig: RoutineSig },
    Nested {
        rust_name: String,
        sig: RoutineSig,
        captures: Vec<CapturedArg>,
    },
}

struct ScopeFrame {
    vars: HashMap<String, VarStorage>,
    routines: HashMap<String, RoutineTarget>,
}

struct ModuleEmitter<'a> {
    validated: &'a ValidatedModule,
    mod_ident: &'a str,
    ns_to_mod_ident: &'a HashMap<String, String>,
    local_consts: HashMap<String, (String, StageLType)>,
    local_dc_consts: HashMap<DcChar, (String, StageLType)>,
    local_routines: HashMap<String, (String, RoutineSig)>,
    local_dc_routines: HashMap<DcChar, (String, RoutineSig)>,
    scopes: Vec<ScopeFrame>,
    temp_counter: usize,
    emitted_test_fns: Vec<String>,
}

impl<'a> ModuleEmitter<'a> {
    fn new(
        validated: &'a ValidatedModule,
        mod_ident: &'a str,
        ns_to_mod_ident: &'a HashMap<String, String>,
    ) -> Result<Self> {
        let mut local_consts = HashMap::new();
        let mut local_dc_consts = HashMap::new();
        let mut local_routines = HashMap::new();
        let mut local_dc_routines = HashMap::new();

        for decl in &validated.module.items {
            match decl {
                TopDecl::Value(v) => {
                    let rname = format!("sl_c_{}", sanitize_ident(&v.name));
                    local_consts.insert(v.name.clone(), (rname.clone(), v.ty.clone()));
                    if let Some(dc) = v.dc_id {
                        local_dc_consts.insert(dc, (rname, v.ty.clone()));
                    }
                }
                TopDecl::Routine(r) => {
                    let rname = format!("sl_fn_{}", sanitize_ident(&r.name));
                    let sig = RoutineSig {
                        return_type: r.return_type.clone(),
                        params: r.params.iter().map(|p| p.ty.clone()).collect(),
                        is_pub: r.is_pub,
                        dc_id: r.dc_id,
                    };
                    local_routines.insert(r.name.clone(), (rname.clone(), sig.clone()));
                    if let Some(dc) = r.dc_id {
                        local_dc_routines.insert(dc, (rname, sig));
                    }
                }
                TopDecl::Comment(_) => {}
            }
        }

        Ok(Self {
            validated,
            mod_ident,
            ns_to_mod_ident,
            local_consts,
            local_dc_consts,
            local_routines,
            local_dc_routines,
            scopes: Vec::new(),
            temp_counter: 0,
            emitted_test_fns: Vec::new(),
        })
    }

    fn next_tmp(&mut self, prefix: &str) -> String {
        let id = self.temp_counter;
        self.temp_counter = self.temp_counter.saturating_add(1);
        format!("__sl_{prefix}_{id}")
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
        for decl in &self.validated.module.items {
            if let TopDecl::Routine(r) = decl {
                self.emit_top_routine(out, r)?;
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

    fn emit_top_routine(&mut self, out: &mut String, r: &RoutineDecl) -> Result<()> {
        let vis = if r.is_pub { "pub " } else { "" };
        let rname = format!("sl_fn_{}", sanitize_ident(&r.name));
        self.emit_routine_fn(out, vis, &rname, r, &[], 1)
    }

    fn emit_routine_fn(
        &mut self,
        out: &mut String,
        vis: &str,
        rust_name: &str,
        r: &RoutineDecl,
        extra_captures: &[CapturedArg],
        indent_level: usize,
    ) -> Result<()> {
        let ind = "    ".repeat(indent_level);
        // If any nested routine inside `r.body` captures a formal param directly,
        // pass `&sl_v_<param>` as ImmRef automatically cleanly via ByRef/Direct.
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
                    param_strs.push(format!("{}: &sl3_rt::SlCap<{rty}>", cap.rust_param));
                    frame.vars.insert(
                        cap.sl_name.clone(),
                        VarStorage::CapCell {
                            rust_name: cap.rust_param.clone(),
                            ty: cap.ty.clone(),
                            is_ref: true,
                        },
                    );
                }
                CapturePassKind::ImmRef => {
                    param_strs.push(format!("{}: &{rty}", cap.rust_param));
                    frame.vars.insert(
                        cap.sl_name.clone(),
                        VarStorage::ByRef {
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

        self.scopes.push(frame);
        let body_ind = indent_level.saturating_add(1);
        let is_void = r.return_type == StageLType::Void;
        if let Expr::Block { stmts, .. } = &r.body {
            let _ = self.emit_block_items(out, stmts, is_void, body_ind)?;
        } else {
            let (body_code, body_ty) = self.emit_expr(&r.body, body_ind)?;
            let body_pad = "    ".repeat(body_ind);
            if is_void && body_ty != StageLType::Never && !stmt_ends_without_semi(&r.body) {
                writeln!(out, "{body_pad}{body_code};")?;
            } else {
                writeln!(out, "{body_pad}{body_code}")?;
            }
        }
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
                        writeln!(
                            out,
                            "        sl3_rt::expect_fault(|| {{ let _ = {call_expr}; }}, {desc:?});"
                        )?;
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
        for scope in self.scopes.iter().rev() {
            if let Some(vs) = scope.vars.get(name) {
                return Some(vs);
            }
        }
        None
    }

    fn visible_var_snapshot(&self) -> HashMap<String, VarStorage> {
        let mut map = HashMap::new();
        for scope in &self.scopes {
            for (k, v) in &scope.vars {
                map.insert(k.clone(), v.clone());
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
                if let Some((rname, sig)) = self.local_routines.get(name) {
                    return Ok(RoutineTarget::TopLevel {
                        rust_path: rname.clone(),
                        sig: sig.clone(),
                    });
                }
                if let Some(ImportedSymbol::Routine {
                    namespace,
                    original_name,
                    sig,
                }) = self.validated.imports.get(name)
                {
                    let mod_ident = self
                        .ns_to_mod_ident
                        .get(namespace)
                        .with_context(|| format!("Unknown module namespace '{namespace}'"))?;
                    return Ok(RoutineTarget::TopLevel {
                        rust_path: format!("super::{mod_ident}::sl_fn_{}", sanitize_ident(original_name)),
                        sig: sig.clone(),
                    });
                }
                bail!("Unknown routine '{name}' during Rust transpilation")
            }
            SymbolTarget::Dc(dc) => {
                if let Some((rname, sig)) = self.local_dc_routines.get(dc) {
                    return Ok(RoutineTarget::TopLevel {
                        rust_path: rname.clone(),
                        sig: sig.clone(),
                    });
                }
                if let Some(ImportedSymbol::Routine {
                    namespace,
                    original_name,
                    sig,
                }) = self.validated.dc_imports.get(dc)
                {
                    let mod_ident = self
                        .ns_to_mod_ident
                        .get(namespace)
                        .with_context(|| format!("Unknown module namespace '{namespace}'"))?;
                    return Ok(RoutineTarget::TopLevel {
                        rust_path: format!("super::{mod_ident}::sl_fn_{}", sanitize_ident(original_name)),
                        sig: sig.clone(),
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
                    let mod_ident = self
                        .ns_to_mod_ident
                        .get(namespace)
                        .with_context(|| format!("Unknown module namespace '{namespace}'"))?;
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
                    let mod_ident = self
                        .ns_to_mod_ident
                        .get(namespace)
                        .with_context(|| format!("Unknown module namespace '{namespace}'"))?;
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
                    let (c, _) = self.emit_expr(el, indent_level)?;
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
                    VarStorage::CapCell {
                        rust_name, is_ref, ..
                    } => {
                        if is_ref {
                            format!("sl3_rt::cap_set({rust_name}, {val_code})")
                        } else {
                            format!("sl3_rt::cap_set(&{rust_name}, {val_code})")
                        }
                    }
                    VarStorage::ByRef { .. } => {
                        bail!("Cannot mutate immutable by-ref captured binding '{name}'")
                    }
                };
                Ok((code, StageLType::Void))
            }
            Expr::Call { target, args, .. } => {
                let resolved = self.resolve_routine_target(target)?;
                let mut arg_codes = Vec::with_capacity(args.len());
                let mut diverged = false;
                for a in args {
                    let (ac, aty) = self.emit_expr(a, indent_level)?;
                    if aty == StageLType::Never {
                        diverged = true;
                    }
                    arg_codes.push((ac, aty));
                }
                let _ = diverged;
                match resolved {
                    RoutineTarget::TopLevel { rust_path, sig } => {
                        let rendered_args: Vec<String> =
                            arg_codes.into_iter().map(|(c, _)| c).collect();
                        Ok((
                            format!("{rust_path}({})", rendered_args.join(", ")),
                            sig.return_type,
                        ))
                    }
                    RoutineTarget::Nested {
                        rust_name,
                        sig,
                        captures,
                    } => {
                        let mut rendered_args: Vec<String> =
                            arg_codes.into_iter().map(|(c, _)| c).collect();
                        for cap in captures {
                            let cur_vs = self
                                .lookup_var(&cap.sl_name)
                                .with_context(|| {
                                    format!("Missing captured binding '{}'", cap.sl_name)
                                })?;
                            match (cur_vs, cap.kind) {
                                (
                                    VarStorage::CapCell {
                                        rust_name: rn,
                                        is_ref,
                                        ..
                                    },
                                    CapturePassKind::CapCell,
                                ) => {
                                    if *is_ref {
                                        rendered_args.push(rn.clone());
                                    } else {
                                        rendered_args.push(format!("&{rn}"));
                                    }
                                }
                                (VarStorage::Direct { rust_name: rn, .. }, CapturePassKind::ImmRef) => {
                                    rendered_args.push(format!("&{rn}"));
                                }
                                (VarStorage::ByRef { rust_name: rn, .. }, CapturePassKind::ImmRef) => {
                                    rendered_args.push(rn.clone());
                                }
                                _ => {
                                    bail!("Inconsistent capture storage for '{}'", cap.sl_name);
                                }
                            }
                        }
                        Ok((
                            format!("{rust_name}({})", rendered_args.join(", ")),
                            sig.return_type,
                        ))
                    }
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
                    self.emit_braced_branch(then_branch, false, indent_level)?;
                let (else_block, else_ty) =
                    self.emit_braced_branch(else_branch, false, indent_level)?;
                let res_ty = match (then_ty == StageLType::Never, else_ty == StageLType::Never) {
                    (true, true) => StageLType::Never,
                    (true, false) => else_ty,
                    (false, true) => then_ty,
                    (false, false) => then_ty,
                };
                let cond_clean = strip_matched_outer_parens(&cond_code);
                let code = if else_block == "{}" {
                    format!("if {cond_clean} {then_block}")
                } else {
                    format!("if {cond_clean} {then_block} else {else_block}")
                };
                Ok((code, res_ty))
            }
            Expr::While { cond, body, .. } => {
                let (cond_code, _) = self.emit_expr(cond, indent_level)?;
                let (body_block, _) = self.emit_braced_branch(body, true, indent_level)?;
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
                let (body_block, _) = self.emit_braced_branch(body, true, indent_level)?;
                let _ = self.scopes.pop();

                let code =
                    format!("for {r_elem} in ({list_code}).iter().cloned() {body_block}");
                Ok((code, StageLType::Void))
            }
            Expr::Break { .. } => Ok((String::from("break"), StageLType::Never)),
            Expr::Continue { .. } => Ok((String::from("continue"), StageLType::Never)),
            Expr::Return { value, .. } => {
                if let Some(v) = value {
                    let (vc, _) = self.emit_expr(v, indent_level)?;
                    Ok((format!("return {vc}"), StageLType::Never))
                } else {
                    Ok((String::from("return"), StageLType::Never))
                }
            }
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
        indent_level: usize,
    ) -> Result<(String, StageLType)> {
        if let Expr::Block { stmts, .. } = expr {
            let mut buf = String::from("{\n");
            let ty = self.emit_block_items(
                &mut buf,
                stmts,
                expect_void,
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
        let (code, ty) = self.emit_expr(expr, body_ind)?;
        let ind = "    ".repeat(indent_level);
        let inner_ind = "    ".repeat(body_ind);
        if (expect_void || ty == StageLType::Void)
            && ty != StageLType::Never
            && !stmt_ends_without_semi(expr)
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
        let ty = self.emit_block_items(&mut buf, stmts, false, indent_level.saturating_add(1))?;
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
        expect_void: bool,
        stmt_indent: usize,
    ) -> Result<StageLType> {
        let mut expr_stmts = Vec::new();
        for item in stmts {
            if let BlockItem::Expr(e) = item {
                expr_stmts.push(e);
            }
        }
        if expr_stmts.is_empty() {
            return Ok(StageLType::Void);
        }

        // Collect block-level declarations & capture plan.
        let plan = analyze_block_captures(&expr_stmts);
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
                    VarStorage::Direct { .. } | VarStorage::ByRef { .. } => {
                        CapturePassKind::ImmRef
                    }
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
        // sibling nested routines use `SlCap<T>` pre-declared before nested fns.
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
                        is_ref: false,
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

        let mut routine_caps_list: Vec<(&RoutineDecl, Vec<CapturedArg>)> =
            Vec::with_capacity(plan.nested_routines.len());
        for &r in &plan.nested_routines {
            let needed = plan.routine_captures.get(&r.name);
            let r_caps: Vec<CapturedArg> = shared_captures
                .iter()
                .filter(|c| needed.is_some_and(|set| set.contains(&c.sl_name)))
                .cloned()
                .collect();
            let rname = format!("sl_fn_{}", sanitize_ident(&r.name));
            let sig = RoutineSig {
                return_type: r.return_type.clone(),
                params: r.params.iter().map(|p| p.ty.clone()).collect(),
                is_pub: false,
                dc_id: None,
            };
            frame.routines.insert(
                r.name.clone(),
                RoutineTarget::Nested {
                    rust_name: rname,
                    sig,
                    captures: r_caps.clone(),
                },
            );
            routine_caps_list.push((r, r_caps));
        }

        self.scopes.push(frame);

        let inner_ind = "    ".repeat(stmt_indent);

        for (name, ty) in &block_cap_decls {
            let rname = format!("sl_v_{}", sanitize_ident(name));
            let rty = rust_type(ty);
            writeln!(
                block_buf,
                "{inner_ind}let {rname}: sl3_rt::SlCap<{rty}> = sl3_rt::cap_new();"
            )?;
        }

        for (r, r_caps) in &routine_caps_list {
            let rname = format!("sl_fn_{}", sanitize_ident(&r.name));
            self.emit_routine_fn(
                block_buf,
                "",
                &rname,
                r,
                r_caps,
                stmt_indent,
            )?;
        }

        let mut result_ty = StageLType::Void;
        let mut terminated = false;
        let count = expr_stmts.len();
        for (idx, stmt) in expr_stmts.iter().enumerate() {
            let is_last = idx.saturating_add(1) == count;
            match stmt {
                Expr::RoutineDecl(_) => {
                    if is_last && terminated {
                        result_ty = StageLType::Never;
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
                        } else {
                            writeln!(
                                block_buf,
                                "{inner_ind}let {mut_kw}{rname}: {rty} = {init_code};"
                            )?;
                            if init_ty == StageLType::Never {
                                terminated = true;
                            }
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
                        result_ty = if terminated {
                            StageLType::Never
                        } else {
                            StageLType::Void
                        };
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
                    } else {
                        writeln!(block_buf, "{inner_ind}let {rname}: {rty} = {val_code};")?;
                        if val_ty == StageLType::Never {
                            terminated = true;
                        }
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
                        result_ty = if terminated {
                            StageLType::Never
                        } else {
                            StageLType::Void
                        };
                    }
                }
                other => {
                    let (code, ty) = self.emit_expr(other, stmt_indent)?;
                    if ty == StageLType::Never {
                        terminated = true;
                    }
                    if is_last {
                        result_ty = if terminated {
                            StageLType::Never
                        } else {
                            ty.clone()
                        };
                        if (expect_void || ty == StageLType::Void)
                            && ty != StageLType::Never
                            && !stmt_ends_without_semi(other)
                        {
                            writeln!(block_buf, "{inner_ind}{code};")?;
                        } else {
                            writeln!(block_buf, "{inner_ind}{code}")?;
                        }
                    } else if ty == StageLType::Void || ty == StageLType::Never {
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
        let (r_code, r_ty) = self.emit_expr(right, indent_level)?;

        if l_ty == StageLType::Never && r_ty == StageLType::Never {
            return Ok((format!("{{ let _ = {l_code}; {r_code} }}"), StageLType::Never));
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
                let res_ty = if l_ty == StageLType::Never { r_ty } else { l_ty };
                let tmp = self.next_tmp("v");
                let msg = format!("checked arithmetic fault in '{}'", op.keyword());
                Ok((
                    format!("{{ let Some({tmp}) = ({l_code}).{method}({r_code}) else {{ panic!({msg:?}); }}; {tmp} }}"),
                    res_ty,
                ))
            }
            CoreBinaryOp::WrapAdd | CoreBinaryOp::WrapSub | CoreBinaryOp::WrapMul => {
                let method = match op {
                    CoreBinaryOp::WrapAdd => "wrapping_add",
                    CoreBinaryOp::WrapSub => "wrapping_sub",
                    CoreBinaryOp::WrapMul => "wrapping_mul",
                    _ => unreachable!("handled above"),
                };
                let res_ty = if l_ty == StageLType::Never { r_ty } else { l_ty };
                Ok((format!("({l_code}).{method}({r_code})"), res_ty))
            }
            CoreBinaryOp::BitAnd | CoreBinaryOp::BitOr | CoreBinaryOp::BitXor => {
                let sym = match op {
                    CoreBinaryOp::BitAnd => "&",
                    CoreBinaryOp::BitOr => "|",
                    CoreBinaryOp::BitXor => "^",
                    _ => unreachable!("handled above"),
                };
                let res_ty = if l_ty == StageLType::Never { r_ty } else { l_ty };
                Ok((format!("(({l_code}) {sym} ({r_code}))"), res_ty))
            }
            CoreBinaryOp::Shl | CoreBinaryOp::Shr => {
                let method = if op == CoreBinaryOp::Shl {
                    "checked_shl"
                } else {
                    "checked_shr"
                };
                let tmp = self.next_tmp("sh");
                if l_ty == StageLType::Never {
                    return Ok((format!("{{ let _ = {l_code}; {r_code} }}"), StageLType::Never));
                }
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
                if l_ty == StageLType::Never {
                    return Ok((format!("{{ let _ = {l_code}; {r_code} }}"), StageLType::Never));
                }
                Ok((format!("({l_code}).{method}({r_code})"), l_ty))
            }
            CoreBinaryOp::And | CoreBinaryOp::Or => {
                let sym = if op == CoreBinaryOp::And { "&&" } else { "||" };
                Ok((format!("(({l_code}) {sym} ({r_code}))"), StageLType::B))
            }
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
                StageLType::Never => Ok((l_code, StageLType::Never)),
                other => bail!("Invalid target '{other}' for 'get'"),
            },
            CoreBinaryOp::Concat => {
                let res_ty = if l_ty == StageLType::Never { r_ty } else { l_ty };
                Ok((
                    format!("sl3_rt::list_concat(&({l_code}), &({r_code}))"),
                    res_ty,
                ))
            }
            CoreBinaryOp::BufFill => {
                if l_ty == StageLType::Never {
                    return Ok((l_code, StageLType::Never));
                }
                let r_elem = rust_type(&l_ty);
                Ok((
                    format!("sl3_rt::SlBuf::<{r_elem}>::filled({l_code}, {r_code})"),
                    StageLType::Buf(Box::new(l_ty)),
                ))
            }
            CoreBinaryOp::BufPush => {
                if l_ty == StageLType::Never || r_ty == StageLType::Never {
                    return Ok((format!("{{ let _ = {l_code}; {r_code}; }}"), StageLType::Void));
                }
                Ok((
                    format!("({l_code}).push({r_code})"),
                    StageLType::Void,
                ))
            }
            CoreBinaryOp::BufExtend => {
                if l_ty == StageLType::Never || r_ty == StageLType::Never {
                    return Ok((format!("{{ let _ = {l_code}; {r_code}; }}"), StageLType::Void));
                }
                Ok((
                    format!("({l_code}).extend_list(&({r_code}))"),
                    StageLType::Void,
                ))
            }
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
        let (c2, t2) = self.emit_expr(second, indent_level)?;
        let (c3, t3) = self.emit_expr(third, indent_level)?;
        if t1 == StageLType::Never {
            return Ok((c1, StageLType::Never));
        }
        if t2 == StageLType::Never || t3 == StageLType::Never {
            return Ok((format!("{{ let _ = {c1}; let _ = {c2}; {c3} }}"), StageLType::Never));
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
            return Ok((in_code, target_ty.clone()));
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
        if in_ty == StageLType::Never || &in_ty == target_ty {
            return Ok((in_code, target_ty.clone()));
        }
        let r_target = rust_type(target_ty);
        Ok((
            format!("{r_target}::from_le_bytes(({in_code}).to_le_bytes())"),
            target_ty.clone(),
        ))
    }
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

struct BlockCapturePlan<'a> {
    nested_routines: Vec<&'a RoutineDecl>,
    local_captured_vars: HashMap<String, StageLType>,
    all_captured_outer: HashSet<String>,
    routine_captures: HashMap<String, HashSet<String>>,
    mutated_locals: HashSet<String>,
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

fn analyze_block_captures<'a>(stmts: &[&'a Expr]) -> BlockCapturePlan<'a> {
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

    // Collect variable references and sibling routine calls per nested routine,
    // then compute transitive captures so sibling callers forward captures.
    let mut routine_captures: HashMap<String, HashSet<String>> = HashMap::new();
    let mut routine_calls: HashMap<String, HashSet<String>> = HashMap::new();
    let nested_names: HashSet<String> = nested_routines.iter().map(|r| r.name.clone()).collect();
    let mut all_refs_in_nested = HashSet::new();

    for r in &nested_routines {
        let mut bound = HashSet::new();
        for p in &r.params {
            bound.insert(p.name.clone());
        }
        let mut direct_vars = HashSet::new();
        collect_free_var_refs(&r.body, &bound, &mut direct_vars);
        for v in &direct_vars {
            all_refs_in_nested.insert(v.clone());
        }
        let mut called = HashSet::new();
        collect_called_routine_names(&r.body, &nested_names, &mut called);
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

    let mut local_captured_vars = HashMap::new();
    let mut all_captured_outer = HashSet::new();
    for var_name in all_refs_in_nested {
        if let Some(ty) = local_decls.get(&var_name) {
            local_captured_vars.insert(var_name, ty.clone());
        } else {
            all_captured_outer.insert(var_name);
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

fn topo_sort_top_values(items: &[TopDecl]) -> Vec<&TopValueDecl> {
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
    use super::super::eval::run_module_inline_tests;
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
            let bin_path = dir.path().join("smoke_bin");
            let compile_out = Command::new("rustc")
                .arg("--edition")
                .arg("2021")
                .arg("-C")
                .arg("opt-level=2")
                .arg("-D")
                .arg("warnings")
                .arg("-o")
                .arg(&bin_path)
                .arg(&rs_path)
                .output()
                .expect("invoke rustc");
            assert!(
                compile_out.status.success(),
                "rustc failed:\n{}\n--- SOURCE ---\n{rust_src}",
                String::from_utf8_lossy(&compile_out.stderr)
            );

            let exec_out = Command::new(&bin_path)
                .output()
                .expect("run compiled rust smoke binary");
            assert!(
                exec_out.status.success(),
                "compiled rust binary failed:\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&exec_out.stdout),
                String::from_utf8_lossy(&exec_out.stderr)
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
}
