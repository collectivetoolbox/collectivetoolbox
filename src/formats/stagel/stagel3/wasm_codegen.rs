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

//! StageL 3 (`f656` `.sl3` and StageL Token Dcs) compiler to `.WASM`
//! (`Dc 1927`..`Dc 1928`) and WebAssembly binary modules, linking Stage 0
//! `.WASM` built-in primitives from the System seed graph (`Dc 646`).

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

pub mod linker;
pub mod runner;
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
mod self_hosted_tests;

pub use runner::{
    WasmTestConfig, compile_and_run_pure_converter_sl3,
    run_dc_tokens_inline_tests_wasm,
    run_dc_tokens_inline_tests_wasm_with_seed_graph,
    run_module_inline_tests_wasm, run_sl3_inline_tests_wasm,
    run_sl3_inline_tests_wasm_with_seed_graph, run_workspace_inline_tests_wasm,
};

use super::ast::{
    BinaryOp, BlockItem, Expr, RoutineDecl, Span, StageDialect, StageLType,
    StructTypeRef, SymbolTarget, TernaryOp, TestExpected, TopDecl, TopValueDecl,
    TypedIntLit, UnaryOp,
};
use super::rust_codegen::{
    analyze_block_captures, sanitize_ident, stagel_params_slug,
    stagel_type_slug, topo_sort_top_values,
};
use super::seed_linker::{
    parse_and_validate_module_with_seed_graph,
    parse_and_validate_workspace_with_seed_graph,
    validate_dc_tokens_with_seed_graph,
};
use super::validator::{
    ImportedSymbol, ModuleInput, RoutineKey, RoutineSig, StructDef,
    ValidatedModule, ValidatedWorkspace, ValueKey,
};
use ctb_formats_dcdata::{DcChar, SeedGraphIndex};
use ctb_formats_dcdata::dc::{
    DC_ADD_OP, DC_BIT_AND_OP, DC_BIT_NOT_OP, DC_BIT_OR_OP, DC_BIT_XOR_OP,
    DC_BITCAST_OP, DC_CLZ_OP, DC_CONVERT_OP, DC_CTZ_OP, DC_DIV_OP, DC_EQ_OP,
    DC_GE_OP, DC_GT_OP, DC_LE_OP, DC_LOGICAL_NOT_OP, DC_LT_OP, DC_MUL_OP,
    DC_NE_OP, DC_NEG_OP, DC_POPCNT_OP, DC_REM_OP, DC_REQUEST_HOST_CAPABILITY_OP,
    DC_ROTL_OP, DC_ROTR_OP,
    DC_SEQ_CONCAT_OP, DC_SEQ_GET_OP, DC_SEQ_LEN_OP, DC_SEQ_SLICE_OP, DC_SHL_OP,
    DC_SHR_OP, DC_STAGEL_BUF_EXTEND, DC_STAGEL_BUF_FILL, DC_STAGEL_BUF_FREEZE,
    DC_STAGEL_BUF_NEW, DC_STAGEL_BUF_PUSH, DC_STAGEL_BUF_SET,
    DC_STAGEL_BUF_TYPE, DC_STAGEL_LIST_TYPE, DC_SUB_OP, DC_TYPE_ANY,
    DC_TYPE_CONVERSION, DC_TYPE_TRANSMUTATION, DC_WRAP_ADD_OP, DC_WRAP_MUL_OP,
    DC_WRAP_SUB_OP,
};
use ctb_formats_dcstring::DcMstring;
use ctb_formats_dcstring::converters::wasm::dcmstring_to_wasm;
use ctb_utilities::format_id::FormatId;
use linker::{WasmFuncBuilder, WasmModuleLinker, WasmValType};
use std::collections::{BTreeMap, HashMap, HashSet};

const SHADOW_STACK_BYTES: i32 = 65_536;
const SHADOW_CELL_BYTES: i32 = 16;

/// Compiles `.sl3` source into a `Dc 1927`..`Dc 1928` [`DcMstring`], linking
/// any external `use` namespaces and transitive `dc/<shorthand>` helpers from
/// [`SeedGraphIndex::shared`].
pub fn compile_sl3_to_dc_wasm(source: &str) -> Result<DcMstring> {
    compile_sl3_to_dc_wasm_with_seed_graph(source, SeedGraphIndex::shared()?)
}

/// Compiles `.sl3` source into a `Dc 1927`..`Dc 1928` [`DcMstring`], linking
/// any external `use` namespaces and transitive `dc/<shorthand>` helpers from
/// `seed_graph`.
pub fn compile_sl3_to_dc_wasm_with_seed_graph(
    source: &str,
    seed_graph: &SeedGraphIndex,
) -> Result<DcMstring> {
    let ws = parse_and_validate_module_with_seed_graph(source, seed_graph)?;
    compile_workspace_to_dc_wasm(&ws)
}

/// Compiles `.sl3` source directly into `.wasm` binary bytes.
pub fn compile_sl3_to_wasm_bytes(source: &str) -> Result<Vec<u8>> {
    let dc_wasm = compile_sl3_to_dc_wasm(source)?;
    dcmstring_to_wasm(&dc_wasm)
}

/// Compiles a StageL 3 Token-Dc stream (`Dc 2192`..`Dc 2193`) into a
/// `Dc 1927`..`Dc 1928` [`DcMstring`], linking external `use` namespaces and
/// transitive `dc/<shorthand>` helpers from [`SeedGraphIndex::shared`].
pub fn compile_dc_tokens_to_dc_wasm(tokens: &[DcChar]) -> Result<DcMstring> {
    compile_dc_tokens_to_dc_wasm_with_seed_graph(
        tokens,
        SeedGraphIndex::shared()?,
    )
}

/// Compiles a StageL 3 Token-Dc stream (`Dc 2192`..`Dc 2193`) into a
/// `Dc 1927`..`Dc 1928` [`DcMstring`], linking external `use` namespaces and
/// transitive `dc/<shorthand>` helpers from `seed_graph`.
pub fn compile_dc_tokens_to_dc_wasm_with_seed_graph(
    tokens: &[DcChar],
    seed_graph: &SeedGraphIndex,
) -> Result<DcMstring> {
    let ws = validate_dc_tokens_with_seed_graph(tokens, seed_graph)?;
    compile_workspace_to_dc_wasm(&ws)
}

/// Compiles a validated single StageL 3 module into a `Dc 1927`..`Dc 1928`
/// [`DcMstring`].
pub fn compile_module_to_dc_wasm(module: &ValidatedModule) -> Result<DcMstring> {
    let ws = ValidatedWorkspace {
        modules: vec![module.clone()],
    };
    compile_workspace_to_dc_wasm(&ws)
}

/// Compiles multi-module `.sl3` sources into a linked `Dc 1927`..`Dc 1928`
/// [`DcMstring`], resolving any external `use` namespaces and `dc/<shorthand>`
/// helpers from [`SeedGraphIndex::shared`].
pub fn compile_workspace_sources_to_dc_wasm(
    inputs: &[ModuleInput<'_>],
) -> Result<DcMstring> {
    compile_workspace_sources_to_dc_wasm_with_seed_graph(
        inputs,
        SeedGraphIndex::shared()?,
    )
}

/// Compiles multi-module `.sl3` sources into a linked `Dc 1927`..`Dc 1928`
/// [`DcMstring`], resolving any external `use` namespaces and `dc/<shorthand>`
/// helpers from `seed_graph`.
pub fn compile_workspace_sources_to_dc_wasm_with_seed_graph(
    inputs: &[ModuleInput<'_>],
    seed_graph: &SeedGraphIndex,
) -> Result<DcMstring> {
    let ws = parse_and_validate_workspace_with_seed_graph(inputs, seed_graph)?;
    compile_workspace_to_dc_wasm(&ws)
}

/// Compiles a [`ValidatedWorkspace`] into a linked `Dc 1927`..`Dc 1928`
/// [`DcMstring`].
pub fn compile_workspace_to_dc_wasm(
    workspace: &ValidatedWorkspace,
) -> Result<DcMstring> {
    let (dc_wasm, _) = compile_workspace_internal(workspace, false)?;
    Ok(dc_wasm)
}

/// Compiles a [`ValidatedWorkspace`] directly into `.wasm` binary bytes.
pub fn compile_workspace_to_wasm_bytes(
    workspace: &ValidatedWorkspace,
) -> Result<Vec<u8>> {
    let dc_wasm = compile_workspace_to_dc_wasm(workspace)?;
    dcmstring_to_wasm(&dc_wasm)
}

#[derive(Debug, Clone)]
pub(crate) struct CompiledTestMeta {
    pub case_id: u32,
    pub namespace: Option<String>,
    pub routine_name: String,
    pub block_label: Option<String>,
    pub case_index: usize,
    pub expects_fault: bool,
}

pub(crate) fn compile_workspace_internal(
    workspace: &ValidatedWorkspace,
    emit_test_dispatcher: bool,
) -> Result<(DcMstring, Vec<CompiledTestMeta>)> {
    for vm in &workspace.modules {
        ensure!(
            vm.module.dialect == StageDialect::StageL3,
            "Direct WebAssembly codegen from StageR 3 (.sr3) is not yet implemented"
        );
    }
    let lowered_ws = workspace.lowered_for_execution();
    let mut compiler = WorkspaceWasmCompiler::new(&lowered_ws)?;
    compiler.compile_all(emit_test_dispatcher)
}

fn stagel_wasm_valtype(ty: &StageLType) -> Option<WasmValType> {
    match ty {
        StageLType::B
        | StageLType::U8
        | StageLType::I32
        | StageLType::U32
        | StageLType::List(_)
        | StageLType::Buf(_)
        | StageLType::Tuple(_)
        | StageLType::Struct(_) => Some(WasmValType::I32),
        StageLType::I64 | StageLType::U64 => Some(WasmValType::I64),
        StageLType::Void
        | StageLType::Never
        | StageLType::I128
        | StageLType::U128
        | StageLType::Int
        | StageLType::Rat
        | StageLType::Dc
        | StageLType::Dcs
        | StageLType::S
        | StageLType::Error
        | StageLType::View
        | StageLType::Cursor
        | StageLType::Option(_)
        | StageLType::Result(_)
        | StageLType::Nominal(_) => None,
    }
}

fn elem_storage_info(ty: &StageLType) -> Result<(u32, u32)> {
    // Returns (byte_stride, align_exp)
    match ty {
        StageLType::B | StageLType::U8 => Ok((1, 0)),
        StageLType::I32
        | StageLType::U32
        | StageLType::List(_)
        | StageLType::Buf(_)
        | StageLType::Tuple(_)
        | StageLType::Struct(_) => Ok((4, 2)),
        StageLType::I64 | StageLType::U64 => Ok((8, 3)),
        other => bail!("Type '{other}' cannot be stored in linear memory"),
    }
}

fn scalar_format_gid(ty: &StageLType) -> Result<u128> {
    let fmt = match ty {
        StageLType::B => FormatId::Boolean,
        StageLType::U8 => FormatId::U8,
        StageLType::I32 => FormatId::I32,
        StageLType::U32 => FormatId::U32,
        StageLType::I64 => FormatId::I64,
        StageLType::U64 => FormatId::U64,
        other => bail!("Type '{other}' is not a scalar StageL format type"),
    };
    let dc = fmt
        .dc_char()
        .with_context(|| format!("Missing DcChar for {fmt:?}"))?;
    Ok(dc.to_u128())
}

fn type_slug(ty: &StageLType) -> String {
    stagel_type_slug(ty)
}

#[derive(Debug, Clone)]
struct TopRoutineEntry {
    func_idx: u32,
    sig: RoutineSig,
}

#[derive(Debug, Clone)]
struct TopConstEntry {
    global_idx: u32,
    ty: StageLType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CapturePassMode {
    CellPtr,
    ImmVal(WasmValType),
}

#[derive(Debug, Clone)]
struct NestedCaptureSpec {
    sl_name: String,
    ty: StageLType,
    mode: CapturePassMode,
}

#[derive(Debug, Clone)]
struct NestedRoutineEntry {
    func_idx: u32,
    declaring_routine_id: usize,
    sig: RoutineSig,
    captures: Vec<NestedCaptureSpec>,
}

#[derive(Debug, Clone)]
enum ResolvedCallee {
    Top(TopRoutineEntry),
    Nested(NestedRoutineEntry),
}

impl ResolvedCallee {
    fn sig(&self) -> &RoutineSig {
        match self {
            Self::Top(t) => &t.sig,
            Self::Nested(n) => &n.sig,
        }
    }
}

#[derive(Debug, Clone)]
enum VarBinding {
    Local {
        local_idx: u32,
        ty: StageLType,
    },
    Cell {
        ptr_local_idx: u32,
        ty: StageLType,
    },
}

impl VarBinding {
    fn ty(&self) -> &StageLType {
        match self {
            Self::Local { ty, .. } | Self::Cell { ty, .. } => ty,
        }
    }
}

struct ScopeLayer {
    vars: HashMap<String, VarBinding>,
    routines: HashMap<String, NestedRoutineEntry>,
}

#[derive(Debug, Clone, Copy)]
struct LoopLabels {
    break_depth_after_open: u32,
    cont_depth_after_open: u32,
}

struct ShadowRuntime {
    base_global: u32,
    limit_global: u32,
    sp_global: u32,
    push_func: u32,
}

struct WorkspaceWasmCompiler<'a> {
    workspace: &'a ValidatedWorkspace,
    linker: WasmModuleLinker,
    uses_shadow_stack: bool,
    shadow_rt: Option<ShadowRuntime>,
    init_flag_global: Option<u32>,
    init_func_idx: Option<u32>,
    ns_to_mods: HashMap<String, Vec<usize>>,
    top_routines_by_key: HashMap<(usize, RoutineKey), TopRoutineEntry>,
    top_routines_by_dc: HashMap<(usize, DcChar), TopRoutineEntry>,
    top_consts_by_key: HashMap<(usize, ValueKey), TopConstEntry>,
    top_consts_by_dc: HashMap<(usize, DcChar), TopConstEntry>,
    struct_defs: BTreeMap<(Option<String>, String), StructDef>,
    structural_eq_helpers: BTreeMap<StageLType, u32>,
    next_routine_uid: usize,
}

impl<'a> WorkspaceWasmCompiler<'a> {
    fn new(workspace: &'a ValidatedWorkspace) -> Result<Self> {
        let uses_shadow_stack = workspace_has_nested_routines(workspace);
        let mut linker = WasmModuleLinker::new();
        let shadow_rt = if uses_shadow_stack {
            let base_global =
                linker.ensure_global("__shadow_base", WasmValType::I32, 0)?;
            let limit_global =
                linker.ensure_global("__shadow_limit", WasmValType::I32, 0)?;
            let sp_global =
                linker.ensure_global("__shadow_sp", WasmValType::I32, 0)?;
            let push_func = linker.reserve_routine("__sl_shadow_push")?;
            Some(ShadowRuntime {
                base_global,
                limit_global,
                sp_global,
                push_func,
            })
        } else {
            None
        };

        let has_top_values = workspace.modules.iter().any(|m| {
            m.module
                .items
                .iter()
                .any(|it| matches!(it, TopDecl::Value(_)))
        });
        let (init_flag_global, init_func_idx) = if has_top_values {
            let flag =
                linker.ensure_global("__sl_inited", WasmValType::I32, 0)?;
            let f_idx = linker.reserve_routine("__sl_init")?;
            (Some(flag), Some(f_idx))
        } else {
            (None, None)
        };

        let mut ns_to_mods: HashMap<String, Vec<usize>> = HashMap::new();
        let mut top_routines_by_key = HashMap::new();
        let mut top_routines_by_dc = HashMap::new();
        let mut top_consts_by_key = HashMap::new();
        let mut top_consts_by_dc = HashMap::new();
        let mut struct_defs: BTreeMap<(Option<String>, String), StructDef> =
            BTreeMap::new();

        for vm in &workspace.modules {
            for (k, sdef) in &vm.struct_defs {
                struct_defs.entry(k.clone()).or_insert_with(|| sdef.clone());
            }
        }

        for (mod_idx, vm) in workspace.modules.iter().enumerate() {
            let mod_prefix = if let Some(ns) = &vm.namespace_key {
                ns_to_mods.entry(ns.clone()).or_default().push(mod_idx);
                format!("m{mod_idx}_{}", sanitize_ident(ns))
            } else {
                format!("m{mod_idx}")
            };

            let mut routine_name_counts: HashMap<&str, usize> = HashMap::new();
            for item in &vm.module.items {
                if let TopDecl::Routine(r) = item {
                    let e = routine_name_counts.entry(r.name.as_str()).or_insert(0);
                    *e = e.saturating_add(1);
                }
            }

            for item in &vm.module.items {
                match item {
                    TopDecl::Value(v) => {
                        let canon_ty = vm.canonicalize_type(&v.ty);
                        let vt = stagel_wasm_valtype(&canon_ty).with_context(|| {
                            format!(
                                "Top-level constant '{}' has non-storable type '{}'",
                                v.name, canon_ty
                            )
                        })?;
                        let san = sanitize_ident(&v.name);
                        let g_name = format!("__sl_c_{mod_prefix}_{san}");
                        let global_idx = linker.ensure_global(&g_name, vt, 0)?;
                        let entry = TopConstEntry {
                            global_idx,
                            ty: canon_ty,
                        };
                        top_consts_by_key
                            .insert((mod_idx, vm.value_key(v)), entry.clone());
                        if let Some(dc) = v.dc_id {
                            top_consts_by_dc.insert((mod_idx, dc), entry);
                        }
                    }
                    TopDecl::Routine(r) => {
                        let params: Vec<StageLType> = r
                            .params
                            .iter()
                            .map(|p| vm.canonicalize_type(&p.ty))
                            .collect();
                        let ret_ty = vm.canonicalize_type(&r.return_type);
                        let overloaded = routine_name_counts
                            .get(r.name.as_str())
                            .copied()
                            .is_some_and(|n| n > 1);
                        let san = sanitize_ident(&r.name);
                        let f_name = if overloaded {
                            format!(
                                "sl_fn_{mod_prefix}_{san}__{}",
                                stagel_params_slug(&params)
                            )
                        } else {
                            format!("sl_fn_{mod_prefix}_{san}")
                        };
                        let func_idx = linker.reserve_routine(&f_name)?;
                        let sig = RoutineSig {
                            return_type: ret_ty,
                            params,
                            is_pub: r.is_pub,
                            dc_id: r.dc_id,
                            may_fault: false,
                        };
                        let entry = TopRoutineEntry { func_idx, sig };
                        top_routines_by_key
                            .insert((mod_idx, vm.routine_key(r)), entry.clone());
                        if let Some(dc) = r.dc_id {
                            top_routines_by_dc.insert((mod_idx, dc), entry);
                        }
                    }
                    TopDecl::Struct(_) | TopDecl::Comment(_) => {}
                    TopDecl::BrandType(_)
                    | TopDecl::NamedStruct(_)
                    | TopDecl::Enum(_) => {
                        bail!("Direct WebAssembly codegen from StageR 3 (.sr3) is not yet implemented");
                    }
                }
            }
        }

        Ok(Self {
            workspace,
            linker,
            uses_shadow_stack,
            shadow_rt,
            init_flag_global,
            init_func_idx,
            ns_to_mods,
            top_routines_by_key,
            top_routines_by_dc,
            top_consts_by_key,
            top_consts_by_dc,
            struct_defs,
            structural_eq_helpers: BTreeMap::new(),
            next_routine_uid: 1,
        })
    }

    fn canon_type(
        &self,
        mod_idx: usize,
        ty: &StageLType,
    ) -> Result<StageLType> {
        let vm = self
            .workspace
            .modules
            .get(mod_idx)
            .context("Invalid module index")?;
        Ok(vm.canonicalize_type(ty))
    }

    fn composite_layout(
        &self,
        canon_ty: &StageLType,
    ) -> Result<(u32, Vec<(u32, u32, u32, StageLType)>)> {
        let slots: &[StageLType] = match canon_ty {
            StageLType::Tuple(elems) => elems.as_slice(),
            StageLType::Struct(StructTypeRef::Canonical { namespace, name }) => {
                let sdef = self
                    .struct_defs
                    .get(&(namespace.clone(), name.clone()))
                    .with_context(|| {
                        format!(
                            "Unknown struct '{namespace:?}::{name}' in WASM layout"
                        )
                    })?;
                sdef.fields.as_slice()
            }
            other => {
                bail!("Type '{other}' is not a positional struct or n-ary tuple")
            }
        };
        if slots.is_empty() {
            return Ok((0, Vec::new()));
        }
        let mut cur = 0u32;
        let mut layout = Vec::with_capacity(slots.len());
        for s in slots {
            let (stride, align_exp) = elem_storage_info(s)?;
            let mask = stride.saturating_sub(1);
            let aligned = cur
                .checked_add(mask)
                .context("Composite slot alignment overflow")?
                & !mask;
            layout.push((aligned, stride, align_exp, s.clone()));
            cur = aligned
                .checked_add(stride)
                .context("Composite total byte size overflow")?;
        }
        let total = cur
            .checked_add(3)
            .context("Composite trailing padding overflow")?
            & !3;
        Ok((total.max(4), layout))
    }

    fn alloc_routine_uid(&mut self) -> usize {
        let id = self.next_routine_uid;
        self.next_routine_uid = self.next_routine_uid.saturating_add(1);
        id
    }

    fn compile_all(
        &mut self,
        emit_test_dispatcher: bool,
    ) -> Result<(DcMstring, Vec<CompiledTestMeta>)> {
        if let Some(ref srt) = self.shadow_rt {
            let srt_copy = ShadowRuntime {
                base_global: srt.base_global,
                limit_global: srt.limit_global,
                sp_global: srt.sp_global,
                push_func: srt.push_func,
            };
            self.emit_shadow_push_helper(&srt_copy)?;
        }

        if let (Some(flag_g), Some(init_fn)) =
            (self.init_flag_global, self.init_func_idx)
        {
            self.emit_top_values_init_fn(flag_g, init_fn)?;
        }

        for mod_idx in 0..self.workspace.modules.len() {
            self.emit_module_routines(mod_idx)?;
        }

        let mut test_metas = Vec::new();
        if emit_test_dispatcher {
            self.emit_test_dispatcher_entry(&mut test_metas)?;
        } else {
            self.emit_default_or_converter_step_entry()?;
        }

        let linker = std::mem::take(&mut self.linker);
        let dcm = linker.finish_to_dcmstring()?;
        Ok((dcm, test_metas))
    }

    fn emit_shadow_push_helper(&mut self, srt: &ShadowRuntime) -> Result<()> {
        let (alloc_idx, _) = self.linker.ensure_alloc_helper()?;
        let mut fb = WasmFuncBuilder::new(
            vec![(String::from("bytes"), WasmValType::I32)],
            Some(WasmValType::I32),
        );
        let bytes_loc = 0u32;
        let old_sp = fb.alloc_local(WasmValType::I32)?;
        let new_sp = fb.alloc_local(WasmValType::I32)?;

        // Lazily initialize 64 KiB shadow stack if `__shadow_base == 0`
        fb.global_get(srt.base_global)?;
        fb.i32_eqz()?;
        fb.if_open(None)?;
        fb.i32_const(SHADOW_STACK_BYTES)?;
        fb.call(alloc_idx)?;
        fb.local_tee(old_sp)?;
        fb.global_set(srt.base_global)?;
        fb.local_get(old_sp)?;
        fb.global_set(srt.sp_global)?;
        fb.local_get(old_sp)?;
        fb.i32_const(SHADOW_STACK_BYTES)?;
        fb.i32_add()?;
        fb.global_set(srt.limit_global)?;
        fb.end_op()?;

        fb.global_get(srt.sp_global)?;
        fb.local_tee(old_sp)?;
        fb.local_get(bytes_loc)?;
        fb.i32_add()?;
        fb.local_tee(new_sp)?;
        fb.global_get(srt.limit_global)?;
        fb.i32_gt_u()?;
        fb.if_open(None)?;
        fb.unreachable()?;
        fb.end_op()?;

        fb.local_get(new_sp)?;
        fb.global_set(srt.sp_global)?;
        // Mark newly pushed 16-byte cell as uninitialized (`init_flag = 0`)
        fb.local_get(old_sp)?;
        fb.i32_const(0)?;
        fb.i32_store(2, 0)?;
        fb.local_get(old_sp)?;

        let toks = fb.into_routine_shorthands()?;
        self.linker.define_routine(srt.push_func, toks)
    }

    fn emit_top_values_init_fn(
        &mut self,
        flag_g: u32,
        init_fn: u32,
    ) -> Result<()> {
        let mut fb = WasmFuncBuilder::new(Vec::new(), None);
        fb.global_get(flag_g)?;
        fb.if_open(None)?;
        fb.return_op()?;
        fb.end_op()?;

        fb.i32_const(1)?;
        fb.global_set(flag_g)?;

        let entry_sp_local = if let Some(ref srt) = self.shadow_rt {
            let loc = fb.alloc_local(WasmValType::I32)?;
            fb.global_get(srt.sp_global)?;
            fb.local_set(loc)?;
            Some((loc, srt.sp_global))
        } else {
            None
        };

        for mod_idx in 0..self.workspace.modules.len() {
            let vm = self
                .workspace
                .modules
                .get(mod_idx)
                .context("Invalid module index")?;
            let sorted: Vec<TopValueDecl> =
                topo_sort_top_values(&vm.module.items)
                    .into_iter()
                    .cloned()
                    .collect();
            for val_decl in &sorted {
                let vkey = self
                    .workspace
                    .modules
                    .get(mod_idx)
                    .context("Invalid module index")?
                    .value_key(val_decl);
                let entry = self
                    .top_consts_by_key
                    .get(&(mod_idx, vkey))
                    .cloned()
                    .context("Missing top-level constant entry")?;
                let r_uid = self.alloc_routine_uid();
                let mut fctx = FuncEmitCtx {
                    mod_idx,
                    routine_uid: r_uid,
                    is_nested: false,
                    return_type: StageLType::Void,
                    entry_sp_local: entry_sp_local.map(|(l, _)| l),
                    caller_restore_sp_local: None,
                    fb: &mut fb,
                    scopes: Vec::new(),
                    loop_stack: Vec::new(),
                    control_depth: 0,
                    nested_counter: 0,
                };
                let expr_ty = self.emit_expr(&mut fctx, &val_decl.value, false)?;
                if expr_ty != StageLType::Never {
                    fctx.fb.global_set(entry.global_idx)?;
                }
            }
        }

        if let Some((loc, sp_g)) = entry_sp_local {
            fb.local_get(loc)?;
            fb.global_set(sp_g)?;
        }

        let toks = fb.into_routine_shorthands()?;
        self.linker.define_routine(init_fn, toks)
    }

    fn emit_module_routines(&mut self, mod_idx: usize) -> Result<()> {
        let vm = self
            .workspace
            .modules
            .get(mod_idx)
            .context("Invalid module index")?;
        let routines: Vec<RoutineDecl> = vm
            .module
            .items
            .iter()
            .filter_map(|it| match it {
                TopDecl::Routine(r) => Some(r.clone()),
                _ => None,
            })
            .collect();

        for r in &routines {
            let rkey = self
                .workspace
                .modules
                .get(mod_idx)
                .context("Invalid module index")?
                .routine_key(r);
            let entry = self
                .top_routines_by_key
                .get(&(mod_idx, rkey))
                .cloned()
                .context("Missing reserved top-level routine")?;
            self.emit_routine_body(
                mod_idx,
                entry.func_idx,
                r,
                false,
                &[],
                &[],
            )?;
        }
        Ok(())
    }

    fn emit_routine_body(
        &mut self,
        mod_idx: usize,
        func_idx: u32,
        r: &RoutineDecl,
        is_nested: bool,
        captures: &[NestedCaptureSpec],
        inherited_scopes: &[ScopeLayer],
    ) -> Result<()> {
        let mut wasm_params = Vec::new();
        let caller_restore_sp_local = if is_nested && self.uses_shadow_stack {
            wasm_params.push((String::from("__restore_sp"), WasmValType::I32));
            Some(0u32)
        } else {
            None
        };

        let mut root_vars = HashMap::new();
        for cap in captures {
            let param_idx = u32::try_from(wasm_params.len())
                .context("WASM parameter index exceeds u32")?;
            let p_name = format!("__cap_{}", sanitize_ident(&cap.sl_name));
            match cap.mode {
                CapturePassMode::CellPtr => {
                    wasm_params.push((p_name, WasmValType::I32));
                    root_vars.insert(
                        cap.sl_name.clone(),
                        VarBinding::Cell {
                            ptr_local_idx: param_idx,
                            ty: cap.ty.clone(),
                        },
                    );
                }
                CapturePassMode::ImmVal(vt) => {
                    wasm_params.push((p_name, vt));
                    root_vars.insert(
                        cap.sl_name.clone(),
                        VarBinding::Local {
                            local_idx: param_idx,
                            ty: cap.ty.clone(),
                        },
                    );
                }
            }
        }

        // Check whether any formal parameter is captured & mutated? Formal
        // parameters in StageL 3 are immutable (`set` on a param is rejected by
        // validator), so formal parameters are ALWAYS `VarBinding::Local`!
        for p in &r.params {
            let param_idx = u32::try_from(wasm_params.len())
                .context("WASM parameter index exceeds u32")?;
            let canon_pty = self.canon_type(mod_idx, &p.ty)?;
            let vt = stagel_wasm_valtype(&canon_pty).with_context(|| {
                format!(
                    "Parameter '{}' of '{}' has non-value type '{}'",
                    p.name, r.name, canon_pty
                )
            })?;
            wasm_params.push((sanitize_ident(&p.name), vt));
            root_vars.insert(
                p.name.clone(),
                VarBinding::Local {
                    local_idx: param_idx,
                    ty: canon_pty,
                },
            );
        }

        let canon_ret = self.canon_type(mod_idx, &r.return_type)?;
        let ret_vt = stagel_wasm_valtype(&canon_ret);
        let mut fb = WasmFuncBuilder::new(wasm_params, ret_vt);

        // Top-level routines ensure module constants are initialized on entry
        if !is_nested
            && let Some(init_fn) = self.init_func_idx
        {
            fb.call(init_fn)?;
        }

        let entry_sp_local = if let Some(ref srt) = self.shadow_rt {
            let loc = fb.alloc_local(WasmValType::I32)?;
            fb.global_get(srt.sp_global)?;
            fb.local_set(loc)?;
            Some(loc)
        } else {
            None
        };

        let mut scopes = Vec::new();
        for layer in inherited_scopes {
            // Inherit visible enclosing nested routines (their captures are
            // forwarded transitively by `analyze_block_captures`).
            scopes.push(ScopeLayer {
                vars: HashMap::new(),
                routines: layer.routines.clone(),
            });
        }
        scopes.push(ScopeLayer {
            vars: root_vars,
            routines: HashMap::new(),
        });

        let routine_uid = self.alloc_routine_uid();
        let mut fctx = FuncEmitCtx {
            mod_idx,
            routine_uid,
            is_nested,
            return_type: canon_ret,
            entry_sp_local,
            caller_restore_sp_local,
            fb: &mut fb,
            scopes,
            loop_stack: Vec::new(),
            control_depth: 0,
            nested_counter: 0,
        };

        let body_ty = self.emit_expr(&mut fctx, &r.body, true)?;
        if body_ty != StageLType::Never {
            if let Some(vt) = ret_vt {
                let tmp = fctx.fb.alloc_local(vt)?;
                fctx.fb.local_set(tmp)?;
                self.emit_routine_shadow_pop(&mut fctx)?;
                fctx.fb.local_get(tmp)?;
            } else {
                self.emit_routine_shadow_pop(&mut fctx)?;
            }
        }

        let toks = fb.into_routine_shorthands()?;
        self.linker.define_routine(func_idx, toks)
    }

    fn emit_routine_shadow_pop(&self, fctx: &mut FuncEmitCtx<'_>) -> Result<()> {
        let Some(ref srt) = self.shadow_rt else {
            return Ok(());
        };
        if fctx.is_nested {
            if let Some(restore_loc) = fctx.caller_restore_sp_local {
                fctx.fb.local_get(restore_loc)?;
                fctx.fb.global_set(srt.sp_global)?;
            }
        } else if let Some(entry_loc) = fctx.entry_sp_local {
            fctx.fb.local_get(entry_loc)?;
            fctx.fb.global_set(srt.sp_global)?;
        }
        Ok(())
    }

    fn emit_default_or_converter_step_entry(&mut self) -> Result<()> {
        if self.linker.routine_index("step").is_some()
            || self.linker.routine_index("main").is_some()
        {
            return Ok(());
        }

        // Check if the final (entry) module defines `v/l/u8/step l/u8/input`
        let last_mod_idx = self.workspace.modules.len().saturating_sub(1);
        let bytes_ty = StageLType::List(Box::new(StageLType::U8));
        let converter_step = self
            .top_routines_by_key
            .get(&(
                last_mod_idx,
                RoutineKey {
                    name: String::from("step"),
                    params: vec![bytes_ty.clone()],
                },
            ))
            .filter(|e| e.sig.return_type == bytes_ty)
            .cloned();

        let step_idx = self.linker.reserve_routine("step")?;
        let mut fb = WasmFuncBuilder::new(Vec::new(), Some(WasmValType::I32));

        if let Some(init_fn) = self.init_func_idx {
            fb.call(init_fn)?;
        }

        if let Some(conv) = converter_step {
            let (alloc_idx, _) = self.linker.ensure_alloc_helper()?;
            let in_hdr = fb.alloc_local(WasmValType::I32)?;
            let out_hdr = fb.alloc_local(WasmValType::I32)?;
            let in_len = fb.alloc_local(WasmValType::I32)?;

            // Allocate 16-byte frozen `l/u8` header pointing at mailbox input `(0..8)`
            fb.i32_const(16)?;
            fb.call(alloc_idx)?;
            fb.local_set(in_hdr)?;

            fb.i32_const(4)?;
            fb.i32_load(2, 0)?;
            fb.local_set(in_len)?;

            fb.local_get(in_hdr)?;
            fb.i32_const(0)?;
            fb.i32_load(2, 0)?;
            fb.i32_store(2, 0)?;

            fb.local_get(in_hdr)?;
            fb.local_get(in_len)?;
            fb.i32_store(2, 4)?;

            fb.local_get(in_hdr)?;
            fb.local_get(in_len)?;
            fb.i32_store(2, 8)?;

            fb.local_get(in_hdr)?;
            fb.i32_const(1)?;
            fb.i32_store(2, 12)?;

            fb.local_get(in_hdr)?;
            fb.call(conv.func_idx)?;
            fb.local_set(out_hdr)?;

            // Write returned `l/u8` `(data_ptr, len)` to mailbox output `(8..16)`
            fb.i32_const(8)?;
            fb.local_get(out_hdr)?;
            fb.i32_load(2, 0)?;
            fb.i32_store(2, 0)?;

            fb.i32_const(12)?;
            fb.local_get(out_hdr)?;
            fb.i32_load(2, 4)?;
            fb.i32_store(2, 0)?;
        }

        fb.i32_const(0)?;
        let toks = fb.into_routine_shorthands()?;
        self.linker.define_routine(step_idx, toks)
    }

    fn emit_test_dispatcher_entry(
        &mut self,
        metas: &mut Vec<CompiledTestMeta>,
    ) -> Result<()> {
        let passed_global =
            self.linker.ensure_global("__test_passed", WasmValType::I32, 0)?;
        let mut case_funcs: Vec<u32> = Vec::new();

        for mod_idx in 0..self.workspace.modules.len() {
            let vm = self
                .workspace
                .modules
                .get(mod_idx)
                .context("Invalid module index")?;
            let ns_opt = vm.namespace_key.clone();
            let routines: Vec<RoutineDecl> = vm
                .module
                .items
                .iter()
                .filter_map(|it| match it {
                    TopDecl::Routine(r) => Some(r.clone()),
                    _ => None,
                })
                .collect();

            for r in &routines {
                let rkey = self
                    .workspace
                    .modules
                    .get(mod_idx)
                    .context("Invalid module index")?
                    .routine_key(r);
                let canon_ret = self.canon_type(mod_idx, &r.return_type)?;
                let target_entry = self
                    .top_routines_by_key
                    .get(&(mod_idx, rkey))
                    .cloned()
                    .context("Missing routine entry for test")?;
                for tb in &r.tests {
                    for (case_index, tc) in tb.cases.iter().enumerate() {
                        let case_id = u32::try_from(case_funcs.len())
                            .context("Test case count exceeds u32")?;
                        let case_fn_name = format!("__sl_test_case_{case_id}");
                        let case_fn_idx =
                            self.linker.reserve_routine(&case_fn_name)?;

                        let expects_fault =
                            matches!(tc.expected, TestExpected::Fault(_));
                        metas.push(CompiledTestMeta {
                            case_id,
                            namespace: ns_opt.clone(),
                            routine_name: r.name.clone(),
                            block_label: tb.label.clone(),
                            case_index,
                            expects_fault,
                        });

                        let mut cfb = WasmFuncBuilder::new(Vec::new(), None);
                        // Reset per-case state (`__test_passed = 0`, shadow SP, heap, init)
                        cfb.i32_const(0)?;
                        cfb.global_set(passed_global)?;
                        if let Some(ref srt) = self.shadow_rt {
                            cfb.i32_const(0)?;
                            cfb.global_set(srt.base_global)?;
                            cfb.i32_const(0)?;
                            cfb.global_set(srt.sp_global)?;
                            cfb.i32_const(0)?;
                            cfb.global_set(srt.limit_global)?;
                        }
                        if let Some(flag_g) = self.init_flag_global {
                            cfb.i32_const(0)?;
                            cfb.global_set(flag_g)?;
                        }
                        if let Some(init_fn) = self.init_func_idx {
                            cfb.call(init_fn)?;
                        }

                        let r_uid = self.alloc_routine_uid();
                        let mut fctx = FuncEmitCtx {
                            mod_idx,
                            routine_uid: r_uid,
                            is_nested: false,
                            return_type: StageLType::Void,
                            entry_sp_local: None,
                            caller_restore_sp_local: None,
                            fb: &mut cfb,
                            scopes: Vec::new(),
                            loop_stack: Vec::new(),
                            control_depth: 0,
                            nested_counter: 0,
                        };

                        let mut diverged = false;
                        for inp in &tc.inputs {
                            let ity = self.emit_expr(&mut fctx, inp, false)?;
                            if ity == StageLType::Never {
                                diverged = true;
                                break;
                            }
                        }

                        if !diverged {
                            fctx.fb.call(target_entry.func_idx)?;
                            if canon_ret == StageLType::Never {
                                fctx.fb.unreachable()?;
                            } else {
                                match &tc.expected {
                                    TestExpected::Expr(exp_expr) => {
                                        let ret_vt =
                                            stagel_wasm_valtype(&canon_ret)
                                                .context("Expected value return type in test")?;
                                        let actual_loc =
                                            fctx.fb.alloc_local(ret_vt)?;
                                        fctx.fb.local_set(actual_loc)?;
                                        let exp_ty = self.emit_expr(
                                            &mut fctx, exp_expr, false,
                                        )?;
                                        if exp_ty != StageLType::Never {
                                            let exp_loc =
                                                fctx.fb.alloc_local(ret_vt)?;
                                            fctx.fb.local_set(exp_loc)?;
                                            fctx.fb.local_get(actual_loc)?;
                                            fctx.fb.local_get(exp_loc)?;
                                            self.emit_eq_op(
                                                &mut fctx,
                                                &canon_ret,
                                                false,
                                            )?;
                                            fctx.fb
                                                .global_set(passed_global)?;
                                        }
                                    }
                                    TestExpected::Void(_) => {
                                        fctx.fb.i32_const(1)?;
                                        fctx.fb.global_set(passed_global)?;
                                    }
                                    TestExpected::Fault(_) => {
                                        if stagel_wasm_valtype(&canon_ret)
                                            .is_some()
                                        {
                                            fctx.fb.drop_op()?;
                                        }
                                        // Normal return when fault was expected: `__test_passed` stays 0
                                    }
                                }
                            }
                        }

                        let ctoks = cfb.into_routine_shorthands()?;
                        self.linker.define_routine(case_fn_idx, ctoks)?;
                        case_funcs.push(case_fn_idx);
                    }
                }
            }
        }

        // Synthesize exported `step() -> i32` parsing decimal ASCII `case_id`
        // from mailbox input `(ptr = mem[0], len = mem[4])`.
        let step_idx = self.linker.reserve_routine("step")?;
        let mut sfb = WasmFuncBuilder::new(Vec::new(), Some(WasmValType::I32));
        let in_ptr = sfb.alloc_local(WasmValType::I32)?;
        let in_len = sfb.alloc_local(WasmValType::I32)?;
        let idx_loc = sfb.alloc_local(WasmValType::I32)?;
        let case_id_loc = sfb.alloc_local(WasmValType::I32)?;

        // Reset bump heap pointer at start of each test invocation if present
        if let Some(&heap_g) = self.linker.global_by_name_opt("__heap_ptr") {
            sfb.i32_const(256)?;
            sfb.global_set(heap_g)?;
        }

        sfb.i32_const(0)?;
        sfb.i32_load(2, 0)?;
        sfb.local_set(in_ptr)?;
        sfb.i32_const(4)?;
        sfb.i32_load(2, 0)?;
        sfb.local_set(in_len)?;

        sfb.i32_const(0)?;
        sfb.local_set(idx_loc)?;
        sfb.i32_const(0)?;
        sfb.local_set(case_id_loc)?;

        sfb.block_open(None)?;
        sfb.loop_open(None)?;
        sfb.local_get(idx_loc)?;
        sfb.local_get(in_len)?;
        sfb.i32_ge_u()?;
        sfb.br_if(1)?;

        sfb.local_get(case_id_loc)?;
        sfb.i32_const(10)?;
        sfb.i32_mul()?;
        sfb.local_get(in_ptr)?;
        sfb.local_get(idx_loc)?;
        sfb.i32_add()?;
        sfb.i32_load8_u(0)?;
        sfb.i32_const(48)?;
        sfb.i32_sub()?;
        sfb.i32_add()?;
        sfb.local_set(case_id_loc)?;

        sfb.local_get(idx_loc)?;
        sfb.i32_const(1)?;
        sfb.i32_add()?;
        sfb.local_set(idx_loc)?;
        sfb.br(0)?;
        sfb.end_op()?;
        sfb.end_op()?;

        for (cid, &fn_idx) in case_funcs.iter().enumerate() {
            let cid_i32 =
                i32::try_from(cid).context("case_id exceeds i32")?;
            sfb.local_get(case_id_loc)?;
            sfb.i32_const(cid_i32)?;
            sfb.i32_eq()?;
            sfb.if_open(None)?;
            sfb.call(fn_idx)?;
            sfb.i32_const(0)?;
            sfb.return_op()?;
            sfb.end_op()?;
        }

        sfb.unreachable()?;
        let stoks = sfb.into_routine_shorthands()?;
        self.linker.define_routine(step_idx, stoks)
    }

    fn emit_expr(
        &mut self,
        fctx: &mut FuncEmitCtx<'_>,
        expr: &Expr,
        in_tail: bool,
    ) -> Result<StageLType> {
        match expr {
            Expr::Bool { value, .. } => {
                fctx.fb.i32_const(i32::from(*value))?;
                Ok(StageLType::B)
            }
            Expr::Int { value, .. } => {
                match value {
                    TypedIntLit::U8(v) => fctx.fb.i32_const(i32::from(*v))?,
                    TypedIntLit::I32(v) => fctx.fb.i32_const(*v)?,
                    TypedIntLit::U32(v) => {
                        fctx.fb.i32_const(i32::from_le_bytes(v.to_le_bytes()))?;
                    }
                    TypedIntLit::I64(v) => fctx.fb.i64_const(*v)?,
                    TypedIntLit::U64(v) => {
                        fctx.fb.i64_const(i64::from_le_bytes(v.to_le_bytes()))?;
                    }
                    TypedIntLit::I128(_)
                    | TypedIntLit::U128(_)
                    | TypedIntLit::Int(_) => {
                        bail!("Direct WebAssembly codegen from StageR 3 (.sr3) is not yet implemented")
                    }
                }
                Ok(value.stage_l_type())
            }
            Expr::Bytes { value, .. } => {
                self.emit_bytes_literal(fctx, value)?;
                Ok(StageLType::List(Box::new(StageLType::U8)))
            }
            Expr::List {
                elem_ty, elements, ..
            } => {
                let canon_elem = self.canon_type(fctx.mod_idx, elem_ty)?;
                self.emit_list_constructor(fctx, &canon_elem, elements)
            }
            Expr::TupleNew { elements, .. } => {
                self.emit_tuple_constructor(fctx, elements)
            }
            Expr::StructNew {
                target,
                fields,
                span,
            } => self.emit_struct_constructor(fctx, target, fields, *span),
            Expr::Field { base, index, .. } => {
                self.emit_field_expr(fctx, base, *index)
            }
            Expr::With {
                base,
                index,
                value,
                ..
            } => self.emit_with_expr(fctx, base, *index, value),
            Expr::BufNew {
                elem_ty, capacity, ..
            } => {
                let cap_ty = self.emit_expr(fctx, capacity, false)?;
                if cap_ty == StageLType::Never {
                    return Ok(StageLType::Never);
                }
                let canon_elem = self.canon_type(fctx.mod_idx, elem_ty)?;
                let fn_idx = self.link_seq_or_buf_builtin(
                    DC_STAGEL_BUF_NEW,
                    DC_STAGEL_BUF_TYPE,
                    &canon_elem,
                )?;
                fctx.fb.call(fn_idx)?;
                Ok(StageLType::Buf(Box::new(canon_elem)))
            }
            Expr::ValueRef { target, span, .. } => {
                self.emit_value_ref(fctx, target, *span)
            }
            Expr::Let { ty, name, init, .. } => {
                self.emit_let_stmt(fctx, ty, name, init.as_deref())
            }
            Expr::Const {
                ty, name, value, ..
            } => self.emit_const_stmt(fctx, ty, name, value),
            Expr::Set { name, value, .. } => {
                self.emit_set_stmt(fctx, name, value)
            }
            Expr::RoutineDecl(_) | Expr::Nop { .. } => Ok(StageLType::Void),
            Expr::AnonRoutine(r) => bail!(
                "Anonymous routine at {} cannot be emitted without call arguments",
                r.span
            ),
            Expr::InlineCall { span, .. } => bail!(
                "Internal WASM codegen error: unlowered inline routine call at {span}"
            ),
            Expr::Call { target, args, span } => {
                self.emit_call_expr(fctx, target, *span, args, in_tail)
            }
            Expr::Block { stmts, .. } => {
                self.emit_block_expr(fctx, stmts, in_tail)
            }
            Expr::If {
                cond,
                then_branch,
                else_branch,
                ..
            } => self.emit_if_expr(fctx, cond, then_branch, else_branch, in_tail),
            Expr::While { cond, body, .. } => {
                self.emit_while_expr(fctx, cond, body)
            }
            Expr::ForEach {
                elem_ty,
                elem_name,
                list,
                body,
                ..
            } => self.emit_foreach_expr(fctx, elem_ty, elem_name, list, body),
            Expr::Break { .. } => {
                let lbl = fctx
                    .loop_stack
                    .last()
                    .copied()
                    .context("'break' outside enclosing loop")?;
                let rel = fctx
                    .control_depth
                    .checked_sub(lbl.break_depth_after_open)
                    .context("Invalid break label depth")?;
                fctx.fb.br(rel)?;
                Ok(StageLType::Never)
            }
            Expr::Continue { .. } => {
                let lbl = fctx
                    .loop_stack
                    .last()
                    .copied()
                    .context("'continue' outside enclosing loop")?;
                let rel = fctx
                    .control_depth
                    .checked_sub(lbl.cont_depth_after_open)
                    .context("Invalid continue label depth")?;
                fctx.fb.br(rel)?;
                Ok(StageLType::Never)
            }
            Expr::Return { value, .. } => {
                if let Some(val_expr) = value {
                    let vty = self.emit_expr(fctx, val_expr, true)?;
                    if vty == StageLType::Never {
                        return Ok(StageLType::Never);
                    }
                    if let Some(vt) = stagel_wasm_valtype(&fctx.return_type) {
                        let tmp = fctx.fb.alloc_local(vt)?;
                        fctx.fb.local_set(tmp)?;
                        self.emit_routine_shadow_pop(fctx)?;
                        fctx.fb.local_get(tmp)?;
                    } else {
                        self.emit_routine_shadow_pop(fctx)?;
                    }
                } else {
                    self.emit_routine_shadow_pop(fctx)?;
                }
                fctx.fb.return_op()?;
                Ok(StageLType::Never)
            }
            Expr::Fault { .. } => {
                fctx.fb.unreachable()?;
                Ok(StageLType::Never)
            }
            Expr::Unary { op, operand, .. } => {
                self.emit_unary_expr(fctx, *op, operand)
            }
            Expr::Binary {
                op, left, right, ..
            } => self.emit_binary_expr(fctx, *op, left, right),
            Expr::Ternary {
                op,
                first,
                second,
                third,
                ..
            } => self.emit_ternary_expr(fctx, *op, first, second, third),
            Expr::Conv {
                total: false,
                expr: inner,
                target_ty,
                ..
            } => self.emit_conv_expr(fctx, inner, target_ty),
            Expr::Bitcast {
                expr: inner,
                target_ty,
                ..
            } => self.emit_bitcast_expr(fctx, inner, target_ty),
            Expr::Conv { total: true, .. }
            | Expr::DctsString { .. }
            | Expr::DcLit { .. }
            | Expr::NamedStructNew { .. }
            | Expr::EnumVariant { .. }
            | Expr::OptionNone { .. }
            | Expr::OptionSome { .. }
            | Expr::ResultOk { .. }
            | Expr::ResultErr { .. }
            | Expr::ErrorTag { .. }
            | Expr::NamedField { .. }
            | Expr::NamedWith { .. }
            | Expr::WrapNominal { .. }
            | Expr::UnwrapNominal { .. }
            | Expr::Match { .. }
            | Expr::Propagate { .. } => {
                bail!("Direct WebAssembly codegen from StageR 3 (.sr3) is not yet implemented")
            }
        }
    }

    fn emit_bytes_literal(
        &mut self,
        fctx: &mut FuncEmitCtx<'_>,
        bytes: &[u8],
    ) -> Result<()> {
        let (alloc_idx, _) = self.linker.ensure_alloc_helper()?;
        let len_i32 = i32::try_from(bytes.len())
            .context("Byte string literal exceeds i32 length")?;
        let total_alloc = 16i32
            .checked_add(len_i32)
            .context("Byte string literal allocation overflow")?;

        let hdr_loc = fctx.fb.alloc_local(WasmValType::I32)?;
        fctx.fb.i32_const(total_alloc)?;
        fctx.fb.call(alloc_idx)?;
        fctx.fb.local_set(hdr_loc)?;

        // +0 data_ptr = hdr + 16
        fctx.fb.local_get(hdr_loc)?;
        fctx.fb.local_get(hdr_loc)?;
        fctx.fb.i32_const(16)?;
        fctx.fb.i32_add()?;
        fctx.fb.i32_store(2, 0)?;

        // +4 len, +8 cap, +12 flags = 1 (frozen)
        fctx.fb.local_get(hdr_loc)?;
        fctx.fb.i32_const(len_i32)?;
        fctx.fb.i32_store(2, 4)?;
        fctx.fb.local_get(hdr_loc)?;
        fctx.fb.i32_const(len_i32)?;
        fctx.fb.i32_store(2, 8)?;
        fctx.fb.local_get(hdr_loc)?;
        fctx.fb.i32_const(1)?;
        fctx.fb.i32_store(2, 12)?;

        let mut pos = 0usize;
        while pos.saturating_add(8) <= bytes.len() {
            let chunk = bytes
                .get(pos..pos.saturating_add(8))
                .context("Invalid 8-byte literal slice")?;
            let mut arr = [0u8; 8];
            arr.copy_from_slice(chunk);
            let val64 = i64::from_le_bytes(arr);
            let mem_off = 16u64.saturating_add(u64::try_from(pos)?);
            fctx.fb.local_get(hdr_loc)?;
            fctx.fb.i64_const(val64)?;
            fctx.fb.i64_store(0, mem_off)?;
            pos = pos.saturating_add(8);
        }
        while let Some(&b) = bytes.get(pos) {
            let mem_off = 16u64.saturating_add(u64::try_from(pos)?);
            fctx.fb.local_get(hdr_loc)?;
            fctx.fb.i32_const(i32::from(b))?;
            fctx.fb.i32_store8(mem_off)?;
            pos = pos.saturating_add(1);
        }

        fctx.fb.local_get(hdr_loc)?;
        Ok(())
    }

    fn emit_list_constructor(
        &mut self,
        fctx: &mut FuncEmitCtx<'_>,
        elem_ty: &StageLType,
        elements: &[Expr],
    ) -> Result<StageLType> {
        let elem_vt = stagel_wasm_valtype(elem_ty).with_context(|| {
            format!("Invalid list element type '{elem_ty}'")
        })?;
        let (stride, align_exp) = elem_storage_info(elem_ty)?;

        let mut elem_locals = Vec::with_capacity(elements.len());
        for el_expr in elements {
            let ety = self.emit_expr(fctx, el_expr, false)?;
            if ety == StageLType::Never {
                return Ok(StageLType::Never);
            }
            let loc = fctx.fb.alloc_local(elem_vt)?;
            fctx.fb.local_set(loc)?;
            elem_locals.push(loc);
        }

        let (alloc_idx, _) = self.linker.ensure_alloc_helper()?;
        let count_u32 = u32::try_from(elements.len())
            .context("List constructor element count exceeds u32")?;
        let payload_u32 = count_u32
            .checked_mul(stride)
            .context("List constructor byte size overflow")?;
        let total_i32 = i32::try_from(
            16u32
                .checked_add(payload_u32)
                .context("List constructor allocation overflow")?,
        )
        .context("List constructor allocation exceeds i32")?;
        let count_i32 =
            i32::try_from(count_u32).context("List count exceeds i32")?;

        let hdr_loc = fctx.fb.alloc_local(WasmValType::I32)?;
        fctx.fb.i32_const(total_i32)?;
        fctx.fb.call(alloc_idx)?;
        fctx.fb.local_set(hdr_loc)?;

        fctx.fb.local_get(hdr_loc)?;
        fctx.fb.local_get(hdr_loc)?;
        fctx.fb.i32_const(16)?;
        fctx.fb.i32_add()?;
        fctx.fb.i32_store(2, 0)?;

        fctx.fb.local_get(hdr_loc)?;
        fctx.fb.i32_const(count_i32)?;
        fctx.fb.i32_store(2, 4)?;
        fctx.fb.local_get(hdr_loc)?;
        fctx.fb.i32_const(count_i32)?;
        fctx.fb.i32_store(2, 8)?;
        fctx.fb.local_get(hdr_loc)?;
        fctx.fb.i32_const(1)?;
        fctx.fb.i32_store(2, 12)?;

        for (idx, &el_loc) in elem_locals.iter().enumerate() {
            let idx_u64 = u64::try_from(idx)?;
            let off = 16u64.saturating_add(
                idx_u64.saturating_mul(u64::from(stride)),
            );
            fctx.fb.local_get(hdr_loc)?;
            fctx.fb.local_get(el_loc)?;
            match stride {
                1 => fctx.fb.i32_store8(off)?,
                4 => fctx.fb.i32_store(align_exp, off)?,
                8 => fctx.fb.i64_store(align_exp, off)?,
                _ => bail!("Unsupported element stride {stride}"),
            }
        }

        fctx.fb.local_get(hdr_loc)?;
        Ok(StageLType::List(Box::new(elem_ty.clone())))
    }

    fn emit_tuple_constructor(
        &mut self,
        fctx: &mut FuncEmitCtx<'_>,
        elements: &[Expr],
    ) -> Result<StageLType> {
        let mut elem_locals = Vec::with_capacity(elements.len());
        let mut elem_tys = Vec::with_capacity(elements.len());
        for el_expr in elements {
            let ety = self.emit_expr(fctx, el_expr, false)?;
            if ety == StageLType::Never {
                return Ok(StageLType::Never);
            }
            let canon_ety = self.canon_type(fctx.mod_idx, &ety)?;
            let vt = stagel_wasm_valtype(&canon_ety)
                .with_context(|| format!("Invalid tuple slot type '{canon_ety}'"))?;
            let loc = fctx.fb.alloc_local(vt)?;
            fctx.fb.local_set(loc)?;
            elem_locals.push(loc);
            elem_tys.push(canon_ety);
        }
        let tup_ty = StageLType::Tuple(elem_tys);
        if elem_locals.is_empty() {
            fctx.fb.i32_const(0)?;
            return Ok(tup_ty);
        }
        self.emit_store_composite_slots(fctx, &tup_ty, &elem_locals)?;
        Ok(tup_ty)
    }

    fn emit_struct_constructor(
        &mut self,
        fctx: &mut FuncEmitCtx<'_>,
        target: &SymbolTarget,
        fields: &[Expr],
        span: Span,
    ) -> Result<StageLType> {
        let mut field_locals = Vec::with_capacity(fields.len());
        for f_expr in fields {
            let fty = self.emit_expr(fctx, f_expr, false)?;
            if fty == StageLType::Never {
                return Ok(StageLType::Never);
            }
            let vt = stagel_wasm_valtype(&fty)
                .with_context(|| format!("Invalid struct field type '{fty}'"))?;
            let loc = fctx.fb.alloc_local(vt)?;
            fctx.fb.local_set(loc)?;
            field_locals.push(loc);
        }
        let struct_ty = {
            let vm = self
                .workspace
                .modules
                .get(fctx.mod_idx)
                .context("Invalid module index")?;
            if let Some(ty) = vm.expr_types.get(&(span.line, span.col)) {
                vm.canonicalize_type(ty)
            } else {
                let raw = match target {
                    SymbolTarget::Ident(name) => {
                        StageLType::Struct(StructTypeRef::Ident(name.clone()))
                    }
                    SymbolTarget::Dc(dc) => {
                        StageLType::Struct(StructTypeRef::Dc(*dc))
                    }
                };
                vm.canonicalize_type(&raw)
            }
        };
        self.emit_store_composite_slots(fctx, &struct_ty, &field_locals)?;
        Ok(struct_ty)
    }

    fn emit_store_composite_slots(
        &mut self,
        fctx: &mut FuncEmitCtx<'_>,
        canon_ty: &StageLType,
        slot_locals: &[u32],
    ) -> Result<()> {
        let (total_bytes, layout) = self.composite_layout(canon_ty)?;
        let (alloc_idx, _) = self.linker.ensure_alloc_helper()?;
        let total_i32 =
            i32::try_from(total_bytes).context("Composite size exceeds i32")?;
        let ptr_loc = fctx.fb.alloc_local(WasmValType::I32)?;
        fctx.fb.i32_const(total_i32)?;
        fctx.fb.call(alloc_idx)?;
        fctx.fb.local_set(ptr_loc)?;

        for ((offset, stride, align_exp, _), &val_loc) in
            layout.iter().zip(slot_locals.iter())
        {
            fctx.fb.local_get(ptr_loc)?;
            fctx.fb.local_get(val_loc)?;
            match *stride {
                1 => fctx.fb.i32_store8(u64::from(*offset))?,
                4 => fctx.fb.i32_store(*align_exp, u64::from(*offset))?,
                8 => fctx.fb.i64_store(*align_exp, u64::from(*offset))?,
                other => bail!("Unsupported composite slot stride {other}"),
            }
        }
        fctx.fb.local_get(ptr_loc)?;
        Ok(())
    }

    fn emit_field_expr(
        &mut self,
        fctx: &mut FuncEmitCtx<'_>,
        base: &Expr,
        index: u32,
    ) -> Result<StageLType> {
        let base_ty = self.emit_expr(fctx, base, false)?;
        if base_ty == StageLType::Never {
            return Ok(StageLType::Never);
        }
        let canon_base = self.canon_type(fctx.mod_idx, &base_ty)?;
        let (_, layout) = self.composite_layout(&canon_base)?;
        let idx = usize::try_from(index).context("Field index exceeds usize")?;
        let (offset, stride, align_exp, slot_ty) = layout
            .get(idx)
            .cloned()
            .with_context(|| format!("Field index {index} out of bounds"))?;
        match stride {
            1 => fctx.fb.i32_load8_u(u64::from(offset))?,
            4 => fctx.fb.i32_load(align_exp, u64::from(offset))?,
            8 => fctx.fb.i64_load(align_exp, u64::from(offset))?,
            other => bail!("Unsupported composite slot stride {other}"),
        }
        Ok(slot_ty)
    }

    fn emit_with_expr(
        &mut self,
        fctx: &mut FuncEmitCtx<'_>,
        base: &Expr,
        index: u32,
        value: &Expr,
    ) -> Result<StageLType> {
        let base_ty = self.emit_expr(fctx, base, false)?;
        if base_ty == StageLType::Never {
            return Ok(StageLType::Never);
        }
        let base_loc = fctx.fb.alloc_local(WasmValType::I32)?;
        fctx.fb.local_set(base_loc)?;

        let val_ty = self.emit_expr(fctx, value, false)?;
        if val_ty == StageLType::Never {
            return Ok(StageLType::Never);
        }
        let val_vt = stagel_wasm_valtype(&val_ty)
            .with_context(|| format!("Invalid 'with' slot value type '{val_ty}'"))?;
        let val_loc = fctx.fb.alloc_local(val_vt)?;
        fctx.fb.local_set(val_loc)?;

        let canon_base = self.canon_type(fctx.mod_idx, &base_ty)?;
        let (total_bytes, layout) = self.composite_layout(&canon_base)?;
        let idx = usize::try_from(index).context("With index exceeds usize")?;
        let (offset, stride, align_exp, _) = layout
            .get(idx)
            .cloned()
            .with_context(|| format!("With index {index} out of bounds"))?;

        let (alloc_idx, _) = self.linker.ensure_alloc_helper()?;
        let total_i32 =
            i32::try_from(total_bytes).context("Composite size exceeds i32")?;
        let new_loc = fctx.fb.alloc_local(WasmValType::I32)?;
        fctx.fb.i32_const(total_i32)?;
        fctx.fb.call(alloc_idx)?;
        fctx.fb.local_set(new_loc)?;

        fctx.fb.local_get(new_loc)?;
        fctx.fb.local_get(base_loc)?;
        fctx.fb.i32_const(total_i32)?;
        fctx.fb.memory_copy()?;

        fctx.fb.local_get(new_loc)?;
        fctx.fb.local_get(val_loc)?;
        match stride {
            1 => fctx.fb.i32_store8(u64::from(offset))?,
            4 => fctx.fb.i32_store(align_exp, u64::from(offset))?,
            8 => fctx.fb.i64_store(align_exp, u64::from(offset))?,
            other => bail!("Unsupported composite slot stride {other}"),
        }

        fctx.fb.local_get(new_loc)?;
        Ok(canon_base)
    }

    fn emit_value_ref(
        &self,
        fctx: &mut FuncEmitCtx<'_>,
        target: &SymbolTarget,
        span: Span,
    ) -> Result<StageLType> {
        match target {
            SymbolTarget::Ident(name) => {
                if let Some(binding) = fctx.lookup_var(name).cloned() {
                    return Self::emit_read_var_binding(fctx, &binding);
                }
                let vm = self
                    .workspace
                    .modules
                    .get(fctx.mod_idx)
                    .context("Invalid module index")?;
                if let Some(rv) =
                    vm.resolved_values.get(&(span.line, span.col))
                {
                    let c = if let Some(namespace) = &rv.namespace {
                        let cand_indices = self
                            .ns_to_mods
                            .get(namespace)
                            .with_context(|| format!("Unknown namespace '{namespace}'"))?;
                        let vkey = rv.value_key();
                        cand_indices
                            .iter()
                            .filter(|&&idx| idx < fctx.mod_idx)
                            .find_map(|&idx| {
                                self.top_consts_by_key.get(&(idx, vkey.clone()))
                            })
                            .with_context(|| {
                                format!(
                                    "Missing resolved constant '{}/{}' during WASM compilation",
                                    rv.ty, rv.original_name
                                )
                            })?
                    } else {
                        self.top_consts_by_key
                            .get(&(fctx.mod_idx, rv.value_key()))
                            .with_context(|| {
                                format!(
                                    "Missing resolved constant '{}/{}' during WASM compilation",
                                    rv.ty, rv.original_name
                                )
                            })?
                    };
                    fctx.fb.global_get(c.global_idx)?;
                    return Ok(c.ty.clone());
                }
                bail!("Unknown value reference '{name}' at {span} during WASM compilation")
            }
            SymbolTarget::Dc(dc) => {
                if let Some(c) =
                    self.top_consts_by_dc.get(&(fctx.mod_idx, *dc))
                {
                    fctx.fb.global_get(c.global_idx)?;
                    return Ok(c.ty.clone());
                }
                let vm = self
                    .workspace
                    .modules
                    .get(fctx.mod_idx)
                    .context("Invalid module index")?;
                if let Some(ImportedSymbol::Value {
                    namespace,
                    original_name,
                    ty,
                    ..
                }) = vm.dc_imports.get(dc)
                {
                    let cand_indices = self
                        .ns_to_mods
                        .get(namespace)
                        .with_context(|| format!("Unknown namespace '{namespace}'"))?;
                    let vkey = ValueKey {
                        name: original_name.clone(),
                        ty: ty.clone(),
                    };
                    let c = cand_indices
                        .iter()
                        .filter(|&&idx| idx < fctx.mod_idx)
                        .find_map(|&idx| {
                            self.top_consts_by_dc
                                .get(&(idx, *dc))
                                .or_else(|| {
                                    self.top_consts_by_key
                                        .get(&(idx, vkey.clone()))
                                })
                        })
                        .with_context(|| {
                            format!("Missing imported Dc value '{namespace}::{original_name}'")
                        })?;
                    fctx.fb.global_get(c.global_idx)?;
                    return Ok(c.ty.clone());
                }
                bail!("Unknown Dc value reference '{dc}' during WASM compilation")
            }
        }
    }

    fn emit_read_var_binding(
        fctx: &mut FuncEmitCtx<'_>,
        binding: &VarBinding,
    ) -> Result<StageLType> {
        match binding {
            VarBinding::Local { local_idx, ty } => {
                fctx.fb.local_get(*local_idx)?;
                Ok(ty.clone())
            }
            VarBinding::Cell { ptr_local_idx, ty } => {
                fctx.fb.local_get(*ptr_local_idx)?;
                fctx.fb.i32_load(2, 0)?;
                fctx.fb.i32_eqz()?;
                fctx.fb.if_open(None)?;
                fctx.fb.unreachable()?;
                fctx.fb.end_op()?;

                fctx.fb.local_get(*ptr_local_idx)?;
                let vt = stagel_wasm_valtype(ty)
                    .context("Captured variable must have storable value type")?;
                match vt {
                    WasmValType::I32 => fctx.fb.i32_load(2, 8)?,
                    WasmValType::I64 => fctx.fb.i64_load(3, 8)?,
                }
                Ok(ty.clone())
            }
        }
    }

    fn emit_write_cell(
        fctx: &mut FuncEmitCtx<'_>,
        ptr_local_idx: u32,
        ty: &StageLType,
        val_local_idx: u32,
    ) -> Result<()> {
        let vt = stagel_wasm_valtype(ty)
            .context("Captured variable must have storable value type")?;
        fctx.fb.local_get(ptr_local_idx)?;
        fctx.fb.local_get(val_local_idx)?;
        match vt {
            WasmValType::I32 => fctx.fb.i32_store(2, 8)?,
            WasmValType::I64 => fctx.fb.i64_store(3, 8)?,
        }
        fctx.fb.local_get(ptr_local_idx)?;
        fctx.fb.i32_const(1)?;
        fctx.fb.i32_store(2, 0)?;
        Ok(())
    }

    fn emit_let_stmt(
        &mut self,
        fctx: &mut FuncEmitCtx<'_>,
        ty: &StageLType,
        name: &str,
        init: Option<&Expr>,
    ) -> Result<StageLType> {
        let canon_ty = self.canon_type(fctx.mod_idx, ty)?;
        let vt = stagel_wasm_valtype(&canon_ty)
            .with_context(|| format!("Invalid 'let' variable type '{canon_ty}'"))?;
        // Check if pre-allocated as a captured `Cell` in the current block scope
        let pre_cell = fctx
            .scopes
            .last()
            .and_then(|s| s.vars.get(name))
            .and_then(|b| match b {
                VarBinding::Cell { ptr_local_idx, .. } => Some(*ptr_local_idx),
                VarBinding::Local { .. } => None,
            });

        if let Some(cell_ptr_loc) = pre_cell {
            if let Some(init_expr) = init {
                let ity = self.emit_expr(fctx, init_expr, false)?;
                if ity == StageLType::Never {
                    return Ok(StageLType::Never);
                }
                let tmp = fctx.fb.alloc_local(vt)?;
                fctx.fb.local_set(tmp)?;
                Self::emit_write_cell(fctx, cell_ptr_loc, &canon_ty, tmp)?;
            }
            return Ok(StageLType::Void);
        }

        let loc = fctx.fb.alloc_local(vt)?;
        if let Some(init_expr) = init {
            let ity = self.emit_expr(fctx, init_expr, false)?;
            if ity == StageLType::Never {
                return Ok(StageLType::Never);
            }
            fctx.fb.local_set(loc)?;
        }
        if let Some(top_scope) = fctx.scopes.last_mut() {
            top_scope.vars.insert(
                name.to_owned(),
                VarBinding::Local {
                    local_idx: loc,
                    ty: canon_ty,
                },
            );
        }
        Ok(StageLType::Void)
    }

    fn emit_const_stmt(
        &mut self,
        fctx: &mut FuncEmitCtx<'_>,
        ty: &StageLType,
        name: &str,
        value: &Expr,
    ) -> Result<StageLType> {
        let canon_ty = self.canon_type(fctx.mod_idx, ty)?;
        let vt = stagel_wasm_valtype(&canon_ty)
            .with_context(|| format!("Invalid 'const' type '{canon_ty}'"))?;
        let vty = self.emit_expr(fctx, value, false)?;
        if vty == StageLType::Never {
            return Ok(StageLType::Never);
        }
        let loc = fctx.fb.alloc_local(vt)?;
        fctx.fb.local_set(loc)?;
        if let Some(top_scope) = fctx.scopes.last_mut() {
            top_scope.vars.insert(
                name.to_owned(),
                VarBinding::Local {
                    local_idx: loc,
                    ty: canon_ty,
                },
            );
        }
        Ok(StageLType::Void)
    }

    fn emit_set_stmt(
        &mut self,
        fctx: &mut FuncEmitCtx<'_>,
        name: &str,
        value: &Expr,
    ) -> Result<StageLType> {
        let binding = fctx
            .lookup_var(name)
            .cloned()
            .with_context(|| format!("Unknown variable '{name}' in 'set'"))?;
        let vty = self.emit_expr(fctx, value, false)?;
        if vty == StageLType::Never {
            return Ok(StageLType::Never);
        }
        match binding {
            VarBinding::Local { local_idx, .. } => {
                fctx.fb.local_set(local_idx)?;
            }
            VarBinding::Cell { ptr_local_idx, ty } => {
                let vt = stagel_wasm_valtype(&ty)
                    .context("Captured variable must have storable value type")?;
                let tmp = fctx.fb.alloc_local(vt)?;
                fctx.fb.local_set(tmp)?;
                Self::emit_write_cell(fctx, ptr_local_idx, &ty, tmp)?;
            }
        }
        Ok(StageLType::Void)
    }

    fn emit_block_expr(
        &mut self,
        fctx: &mut FuncEmitCtx<'_>,
        stmts: &[BlockItem],
        in_tail: bool,
    ) -> Result<StageLType> {
        let exprs: Vec<&Expr> = stmts
            .iter()
            .filter_map(|it| match it {
                BlockItem::Expr(e) => Some(e),
                BlockItem::Comment(_) => None,
            })
            .collect();

        if exprs.is_empty() {
            return Ok(StageLType::Void);
        }

        let enclosing_caps = fctx.enclosing_routine_captures_map();
        let plan = analyze_block_captures(&exprs, &enclosing_caps);
        let mut set_targets_deep = HashSet::new();
        for stmt in &exprs {
            collect_set_targets_deep(stmt, &mut set_targets_deep);
        }

        let mut scope_layer = ScopeLayer {
            vars: HashMap::new(),
            routines: HashMap::new(),
        };

        // Pre-allocate shadow cells for local `let` bindings in this block that
        // are captured AND (mutated anywhere via `set` OR declared uninitialized).
        for stmt in &exprs {
            if let Expr::Let {
                name, ty, init, ..
            } = stmt
                && plan.local_captured_vars.contains_key(name)
                && (plan.mutated_locals.contains(name)
                    || set_targets_deep.contains(name)
                    || init.is_none())
            {
                let canon_ty = self.canon_type(fctx.mod_idx, ty)?;
                let srt = self
                    .shadow_rt
                    .as_ref()
                    .context("Shadow stack required for captured mutable variable")?;
                let ptr_loc = fctx.fb.alloc_local(WasmValType::I32)?;
                fctx.fb.i32_const(SHADOW_CELL_BYTES)?;
                fctx.fb.call(srt.push_func)?;
                fctx.fb.local_set(ptr_loc)?;
                scope_layer.vars.insert(
                    name.clone(),
                    VarBinding::Cell {
                        ptr_local_idx: ptr_loc,
                        ty: canon_ty,
                    },
                );
            }
        }

        // Build visible binding types & modes (including locals declared in this block)
        let mut block_var_modes: HashMap<String, (StageLType, CapturePassMode)> =
            fctx.visible_capture_modes();
        for stmt in &exprs {
            match stmt {
                Expr::Let { name, ty, .. } | Expr::Const { name, ty, .. } => {
                    let canon_ty = self.canon_type(fctx.mod_idx, ty)?;
                    if let Some(VarBinding::Cell { .. }) =
                        scope_layer.vars.get(name)
                    {
                        block_var_modes.insert(
                            name.clone(),
                            (canon_ty, CapturePassMode::CellPtr),
                        );
                    } else if let Some(vt) = stagel_wasm_valtype(&canon_ty) {
                        block_var_modes.insert(
                            name.clone(),
                            (canon_ty, CapturePassMode::ImmVal(vt)),
                        );
                    }
                }
                _ => {}
            }
        }

        // Reserve function indices and capture specs for all nested routines in this block
        let mut pending_nested: Vec<(&RoutineDecl, NestedRoutineEntry)> =
            Vec::new();
        for nr in &plan.nested_routines {
            let nid = fctx.nested_counter;
            fctx.nested_counter = fctx.nested_counter.saturating_add(1);
            let fn_name = format!(
                "__sl_nested_m{}_r{}_{nid}_{}",
                fctx.mod_idx,
                fctx.routine_uid,
                sanitize_ident(&nr.name)
            );
            let fn_idx = self.linker.reserve_routine(&fn_name)?;

            let mut cap_names: Vec<String> = plan
                .routine_captures
                .get(&nr.name)
                .with_context(|| {
                    format!(
                        "Missing capture analysis entry for nested routine '{}'",
                        nr.name
                    )
                })?
                .iter()
                .cloned()
                .collect();
            cap_names.sort();

            let mut cap_specs = Vec::new();
            for cname in cap_names {
                if let Some((cty, cmode)) = block_var_modes.get(&cname) {
                    cap_specs.push(NestedCaptureSpec {
                        sl_name: cname,
                        ty: cty.clone(),
                        mode: *cmode,
                    });
                }
            }

            let mut canon_params = Vec::with_capacity(nr.params.len());
            for p in &nr.params {
                canon_params.push(self.canon_type(fctx.mod_idx, &p.ty)?);
            }
            let sig = RoutineSig {
                return_type: self.canon_type(fctx.mod_idx, &nr.return_type)?,
                params: canon_params,
                is_pub: false,
                dc_id: nr.dc_id,
                may_fault: false,
            };
            let entry = NestedRoutineEntry {
                func_idx: fn_idx,
                declaring_routine_id: fctx.routine_uid,
                sig,
                captures: cap_specs,
            };
            scope_layer.routines.insert(nr.name.clone(), entry.clone());
            pending_nested.push((*nr, entry));
        }

        fctx.scopes.push(scope_layer);

        // Compile lifted nested routine bodies with inherited scope visibility
        let inherited_snapshot: Vec<ScopeLayer> = fctx
            .scopes
            .iter()
            .map(|s| ScopeLayer {
                vars: HashMap::new(),
                routines: s.routines.clone(),
            })
            .collect();
        for (nr, entry) in pending_nested {
            self.emit_routine_body(
                fctx.mod_idx,
                entry.func_idx,
                nr,
                true,
                &entry.captures,
                &inherited_snapshot,
            )?;
        }

        let last_idx = exprs.len().saturating_sub(1);
        let mut result_ty = StageLType::Void;
        for (idx, stmt) in exprs.iter().enumerate() {
            let is_last = idx == last_idx;
            let sty = self.emit_expr(fctx, stmt, in_tail && is_last)?;
            if sty == StageLType::Never {
                let _ = fctx.scopes.pop();
                return Ok(StageLType::Never);
            }
            if !is_last {
                if stagel_wasm_valtype(&sty).is_some() {
                    fctx.fb.drop_op()?;
                }
            } else {
                result_ty = sty;
            }
        }

        let _ = fctx.scopes.pop();
        Ok(result_ty)
    }

    fn emit_if_expr(
        &mut self,
        fctx: &mut FuncEmitCtx<'_>,
        cond: &Expr,
        then_branch: &Expr,
        else_branch: &Expr,
        in_tail: bool,
    ) -> Result<StageLType> {
        let cty = self.emit_expr(fctx, cond, false)?;
        if cty == StageLType::Never {
            return Ok(StageLType::Never);
        }

        let then_ty_static = self.infer_expr_type(fctx, then_branch)?;
        let else_ty_static = self.infer_expr_type(fctx, else_branch)?;
        let unified_ty = match (&then_ty_static, &else_ty_static) {
            (StageLType::Never, other) => other.clone(),
            (other, StageLType::Never) => other.clone(),
            (a, _) => a.clone(),
        };
        let bt = stagel_wasm_valtype(&unified_ty);

        fctx.fb.if_open(bt)?;
        fctx.control_depth = fctx.control_depth.saturating_add(1);

        let t_actual = self.emit_expr(fctx, then_branch, in_tail)?;
        fctx.fb.else_op()?;
        let e_actual = self.emit_expr(fctx, else_branch, in_tail)?;

        fctx.control_depth = fctx.control_depth.saturating_sub(1);
        fctx.fb.end_op()?;

        if t_actual == StageLType::Never && e_actual == StageLType::Never {
            fctx.fb.unreachable()?;
            return Ok(StageLType::Never);
        }
        Ok(unified_ty)
    }

    fn emit_while_expr(
        &mut self,
        fctx: &mut FuncEmitCtx<'_>,
        cond: &Expr,
        body: &Expr,
    ) -> Result<StageLType> {
        fctx.fb.block_open(None)?;
        fctx.control_depth = fctx.control_depth.saturating_add(1);
        let break_depth = fctx.control_depth;

        fctx.fb.loop_open(None)?;
        fctx.control_depth = fctx.control_depth.saturating_add(1);
        let cont_depth = fctx.control_depth;

        let cty = self.emit_expr(fctx, cond, false)?;
        if cty != StageLType::Never {
            fctx.fb.i32_eqz()?;
            let exit_rel = fctx
                .control_depth
                .checked_sub(break_depth)
                .context("Invalid while exit depth")?;
            fctx.fb.br_if(exit_rel)?;

            fctx.loop_stack.push(LoopLabels {
                break_depth_after_open: break_depth,
                cont_depth_after_open: cont_depth,
            });
            let bty = self.emit_expr(fctx, body, false)?;
            let _ = fctx.loop_stack.pop();

            if bty != StageLType::Never {
                let loop_rel = fctx
                    .control_depth
                    .checked_sub(cont_depth)
                    .context("Invalid while repeat depth")?;
                fctx.fb.br(loop_rel)?;
            }
        }

        fctx.control_depth = fctx.control_depth.saturating_sub(1);
        fctx.fb.end_op()?;
        fctx.control_depth = fctx.control_depth.saturating_sub(1);
        fctx.fb.end_op()?;
        Ok(StageLType::Void)
    }

    fn emit_foreach_expr(
        &mut self,
        fctx: &mut FuncEmitCtx<'_>,
        elem_ty: &StageLType,
        elem_name: &str,
        list: &Expr,
        body: &Expr,
    ) -> Result<StageLType> {
        let lty = self.emit_expr(fctx, list, false)?;
        if lty == StageLType::Never {
            return Ok(StageLType::Never);
        }

        let canon_elem = self.canon_type(fctx.mod_idx, elem_ty)?;
        let elem_vt = stagel_wasm_valtype(&canon_elem)
            .with_context(|| format!("Invalid foreach element type '{canon_elem}'"))?;
        let (stride, align_exp) = elem_storage_info(&canon_elem)?;
        let stride_i32 =
            i32::try_from(stride).context("Element stride exceeds i32")?;

        let list_hdr = fctx.fb.alloc_local(WasmValType::I32)?;
        let data_ptr = fctx.fb.alloc_local(WasmValType::I32)?;
        let len_loc = fctx.fb.alloc_local(WasmValType::I32)?;
        let idx_loc = fctx.fb.alloc_local(WasmValType::I32)?;
        let elem_loc = fctx.fb.alloc_local(elem_vt)?;

        fctx.fb.local_set(list_hdr)?;
        fctx.fb.local_get(list_hdr)?;
        fctx.fb.i32_load(2, 0)?;
        fctx.fb.local_set(data_ptr)?;
        fctx.fb.local_get(list_hdr)?;
        fctx.fb.i32_load(2, 4)?;
        fctx.fb.local_set(len_loc)?;
        fctx.fb.i32_const(0)?;
        fctx.fb.local_set(idx_loc)?;

        fctx.fb.block_open(None)?;
        fctx.control_depth = fctx.control_depth.saturating_add(1);
        let break_depth = fctx.control_depth;

        fctx.fb.loop_open(None)?;
        fctx.control_depth = fctx.control_depth.saturating_add(1);
        let top_depth = fctx.control_depth;

        fctx.fb.local_get(idx_loc)?;
        fctx.fb.local_get(len_loc)?;
        fctx.fb.i32_ge_u()?;
        let exit_rel = fctx
            .control_depth
            .checked_sub(break_depth)
            .context("Invalid foreach exit depth")?;
        fctx.fb.br_if(exit_rel)?;

        // Load `elem` at `data_ptr + idx_loc * stride`
        fctx.fb.local_get(data_ptr)?;
        fctx.fb.local_get(idx_loc)?;
        fctx.fb.i32_const(stride_i32)?;
        fctx.fb.i32_mul()?;
        fctx.fb.i32_add()?;
        match stride {
            1 => fctx.fb.i32_load8_u(0)?,
            4 => fctx.fb.i32_load(align_exp, 0)?,
            8 => fctx.fb.i64_load(align_exp, 0)?,
            _ => bail!("Unsupported foreach element stride {stride}"),
        }
        fctx.fb.local_set(elem_loc)?;

        // Inner block targeted by `continue` so `$idx_loc += 1` always runs
        fctx.fb.block_open(None)?;
        fctx.control_depth = fctx.control_depth.saturating_add(1);
        let cont_depth = fctx.control_depth;

        let mut iter_scope = ScopeLayer {
            vars: HashMap::new(),
            routines: HashMap::new(),
        };
        iter_scope.vars.insert(
            elem_name.to_owned(),
            VarBinding::Local {
                local_idx: elem_loc,
                ty: canon_elem,
            },
        );
        fctx.scopes.push(iter_scope);
        fctx.loop_stack.push(LoopLabels {
            break_depth_after_open: break_depth,
            cont_depth_after_open: cont_depth,
        });
        let _ = self.emit_expr(fctx, body, false)?;
        let _ = fctx.loop_stack.pop();
        let _ = fctx.scopes.pop();

        fctx.control_depth = fctx.control_depth.saturating_sub(1);
        fctx.fb.end_op()?;

        fctx.fb.local_get(idx_loc)?;
        fctx.fb.i32_const(1)?;
        fctx.fb.i32_add()?;
        fctx.fb.local_set(idx_loc)?;
        let repeat_rel = fctx
            .control_depth
            .checked_sub(top_depth)
            .context("Invalid foreach repeat depth")?;
        fctx.fb.br(repeat_rel)?;

        fctx.control_depth = fctx.control_depth.saturating_sub(1);
        fctx.fb.end_op()?;
        fctx.control_depth = fctx.control_depth.saturating_sub(1);
        fctx.fb.end_op()?;

        Ok(StageLType::Void)
    }

    fn resolve_callee(
        &self,
        fctx: &FuncEmitCtx<'_>,
        target: &SymbolTarget,
        span: Span,
    ) -> Result<ResolvedCallee> {
        match target {
            SymbolTarget::Ident(name) => {
                if let Some(nr) = fctx.lookup_nested_routine(name) {
                    return Ok(ResolvedCallee::Nested(nr.clone()));
                }
                let vm = self
                    .workspace
                    .modules
                    .get(fctx.mod_idx)
                    .context("Invalid module index")?;
                if let Some(rc) = vm.resolved_calls.get(&(span.line, span.col)) {
                    let tr = if let Some(namespace) = &rc.namespace {
                        let cand_indices = self
                            .ns_to_mods
                            .get(namespace)
                            .with_context(|| format!("Unknown namespace '{namespace}'"))?;
                        let rkey = rc.routine_key();
                        cand_indices
                            .iter()
                            .filter(|&&idx| idx < fctx.mod_idx)
                            .find_map(|&idx| {
                                self.top_routines_by_key
                                    .get(&(idx, rkey.clone()))
                            })
                            .with_context(|| {
                                format!(
                                    "Unknown resolved routine '{}' during WASM compilation",
                                    rc.routine_name
                                )
                            })?
                    } else {
                        self.top_routines_by_key
                            .get(&(fctx.mod_idx, rc.routine_key()))
                            .with_context(|| {
                                format!(
                                    "Unknown resolved routine '{}' during WASM compilation",
                                    rc.routine_name
                                )
                            })?
                    };
                    return Ok(ResolvedCallee::Top(tr.clone()));
                }
                bail!("Unknown routine '{name}' at {span} during WASM compilation")
            }
            SymbolTarget::Dc(dc) => {
                if let Some(tr) =
                    self.top_routines_by_dc.get(&(fctx.mod_idx, *dc))
                {
                    return Ok(ResolvedCallee::Top(tr.clone()));
                }
                let vm = self
                    .workspace
                    .modules
                    .get(fctx.mod_idx)
                    .context("Invalid module index")?;
                if let Some(rc) = vm.resolved_calls.get(&(span.line, span.col))
                    && rc.namespace.is_none()
                    && let Some(tr) = self
                        .top_routines_by_key
                        .get(&(fctx.mod_idx, rc.routine_key()))
                {
                    return Ok(ResolvedCallee::Top(tr.clone()));
                }
                if let Some(ImportedSymbol::Routine {
                    namespace,
                    original_name,
                    sig,
                }) = vm.dc_imports.get(dc)
                {
                    let cand_indices = self
                        .ns_to_mods
                        .get(namespace)
                        .with_context(|| format!("Unknown namespace '{namespace}'"))?;
                    let rkey = RoutineKey {
                        name: original_name.clone(),
                        params: sig.params.clone(),
                    };
                    let tr = cand_indices
                        .iter()
                        .filter(|&&idx| idx < fctx.mod_idx)
                        .find_map(|&idx| {
                            self.top_routines_by_dc
                                .get(&(idx, *dc))
                                .or_else(|| {
                                    self.top_routines_by_key
                                        .get(&(idx, rkey.clone()))
                                })
                        })
                        .with_context(|| {
                            format!("Unknown imported Dc routine '{namespace}::{original_name}'")
                        })?;
                    return Ok(ResolvedCallee::Top(tr.clone()));
                }
                bail!("Unknown Dc routine '{dc}' during WASM compilation")
            }
        }
    }

    fn emit_call_expr(
        &mut self,
        fctx: &mut FuncEmitCtx<'_>,
        target: &SymbolTarget,
        span: Span,
        args: &[Expr],
        in_tail: bool,
    ) -> Result<StageLType> {
        let callee = self.resolve_callee(fctx, target, span)?;
        let callee_ret_ty = callee.sig().return_type.clone();
        let can_return_call = in_tail
            && stagel_wasm_valtype(&fctx.return_type)
                == stagel_wasm_valtype(&callee_ret_ty);

        // Evaluate positional StageL arguments strictly left-to-right into temp locals
        let mut arg_locals = Vec::with_capacity(args.len());
        for arg_expr in args {
            let aty = self.emit_expr(fctx, arg_expr, false)?;
            if aty == StageLType::Never {
                return Ok(StageLType::Never);
            }
            let vt = stagel_wasm_valtype(&aty)
                .context("Routine argument must have a WASM value type")?;
            let loc = fctx.fb.alloc_local(vt)?;
            fctx.fb.local_set(loc)?;
            arg_locals.push(loc);
        }

        match callee {
            ResolvedCallee::Top(top) => {
                if can_return_call {
                    self.emit_routine_shadow_pop(fctx)?;
                    for loc in arg_locals {
                        fctx.fb.local_get(loc)?;
                    }
                    fctx.fb.return_call(top.func_idx)?;
                    return Ok(StageLType::Never);
                }
                for loc in arg_locals {
                    fctx.fb.local_get(loc)?;
                }
                fctx.fb.call(top.func_idx)?;
                if callee_ret_ty == StageLType::Never {
                    fctx.fb.unreachable()?;
                }
                Ok(callee_ret_ty)
            }
            ResolvedCallee::Nested(nested) => {
                let mut cap_locals = Vec::with_capacity(nested.captures.len());
                for cap in &nested.captures {
                    let binding = fctx
                        .lookup_var(&cap.sl_name)
                        .cloned()
                        .with_context(|| {
                            format!(
                                "Missing captured binding '{}' at call to nested routine",
                                cap.sl_name
                            )
                        })?;
                    match (cap.mode, binding) {
                        (
                            CapturePassMode::CellPtr,
                            VarBinding::Cell { ptr_local_idx, .. },
                        ) => {
                            cap_locals.push(ptr_local_idx);
                        }
                        (CapturePassMode::ImmVal(vt), b) => {
                            let _ = Self::emit_read_var_binding(fctx, &b)?;
                            let tmp = fctx.fb.alloc_local(vt)?;
                            fctx.fb.local_set(tmp)?;
                            cap_locals.push(tmp);
                        }
                        _ => bail!(
                            "Capture storage mismatch for '{}'",
                            cap.sl_name
                        ),
                    }
                }

                if can_return_call {
                    let child_of_current =
                        nested.declaring_routine_id == fctx.routine_uid;
                    let mut restore_arg_loc: Option<u32> = None;
                    if let Some(ref srt) = self.shadow_rt {
                        if child_of_current {
                            // Keep current routine's shadow cells live for child;
                            // pass current routine's restore target!
                            let r_loc = if fctx.is_nested {
                                fctx.caller_restore_sp_local
                            } else {
                                fctx.entry_sp_local
                            }
                            .context("Missing restore SP local")?;
                            restore_arg_loc = Some(r_loc);
                        } else {
                            // Sibling/self/ancestor nested routine: pop only
                            // current routine's own cells (`entry_sp_local`),
                            // keeping enclosing frames live!
                            if let Some(entry_loc) = fctx.entry_sp_local {
                                fctx.fb.local_get(entry_loc)?;
                                fctx.fb.global_set(srt.sp_global)?;
                            }
                            let r_loc = fctx
                                .caller_restore_sp_local
                                .or(fctx.entry_sp_local)
                                .context("Missing restore SP local")?;
                            restore_arg_loc = Some(r_loc);
                        }
                    }

                    if let Some(r_loc) = restore_arg_loc {
                        fctx.fb.local_get(r_loc)?;
                    }
                    for c_loc in cap_locals {
                        fctx.fb.local_get(c_loc)?;
                    }
                    for a_loc in arg_locals {
                        fctx.fb.local_get(a_loc)?;
                    }
                    fctx.fb.return_call(nested.func_idx)?;
                    return Ok(StageLType::Never);
                }

                if let Some(ref srt) = self.shadow_rt {
                    fctx.fb.global_get(srt.sp_global)?;
                }
                for c_loc in cap_locals {
                    fctx.fb.local_get(c_loc)?;
                }
                for a_loc in arg_locals {
                    fctx.fb.local_get(a_loc)?;
                }
                fctx.fb.call(nested.func_idx)?;
                if callee_ret_ty == StageLType::Never {
                    fctx.fb.unreachable()?;
                }
                Ok(callee_ret_ty)
            }
        }
    }

    fn emit_unary_expr(
        &mut self,
        fctx: &mut FuncEmitCtx<'_>,
        op: UnaryOp,
        operand: &Expr,
    ) -> Result<StageLType> {
        let op_ty = self.emit_expr(fctx, operand, false)?;
        if op_ty == StageLType::Never {
            return Ok(StageLType::Never);
        }
        match op {
            UnaryOp::Neg => {
                let fn_idx = self.link_scalar_builtin(DC_NEG_OP, &op_ty)?;
                fctx.fb.call(fn_idx)?;
                Ok(op_ty)
            }
            UnaryOp::BitNot => {
                let fn_idx = self.link_scalar_builtin(DC_BIT_NOT_OP, &op_ty)?;
                fctx.fb.call(fn_idx)?;
                Ok(op_ty)
            }
            UnaryOp::Not => {
                let fn_idx =
                    self.link_scalar_builtin(DC_LOGICAL_NOT_OP, &op_ty)?;
                fctx.fb.call(fn_idx)?;
                Ok(StageLType::B)
            }
            UnaryOp::Clz => {
                let fn_idx = self.link_scalar_builtin(DC_CLZ_OP, &op_ty)?;
                fctx.fb.call(fn_idx)?;
                Ok(StageLType::U32)
            }
            UnaryOp::Ctz => {
                let fn_idx = self.link_scalar_builtin(DC_CTZ_OP, &op_ty)?;
                fctx.fb.call(fn_idx)?;
                Ok(StageLType::U32)
            }
            UnaryOp::Popcnt => {
                let fn_idx = self.link_scalar_builtin(DC_POPCNT_OP, &op_ty)?;
                fctx.fb.call(fn_idx)?;
                Ok(StageLType::U32)
            }
            UnaryOp::Len => {
                let container_dc = match &op_ty {
                    StageLType::List(_) => DC_STAGEL_LIST_TYPE,
                    StageLType::Buf(_) => DC_STAGEL_BUF_TYPE,
                    other => bail!("Invalid operand type '{other}' for 'len'"),
                };
                let key = format!("__builtin_2543_{}", container_dc.to_u128());
                let fn_idx = self.linker.link_system_builtin(
                    DC_SEQ_LEN_OP.to_u128(),
                    &[container_dc.to_u128(), DC_TYPE_ANY.to_u128()],
                    &key,
                )?;
                fctx.fb.call(fn_idx)?;
                Ok(StageLType::U64)
            }
            UnaryOp::BufFreeze => {
                let StageLType::Buf(inner) = op_ty else {
                    bail!("Invalid operand type '{op_ty}' for 'bufFreeze'");
                };
                let fn_idx = self.linker.link_system_builtin(
                    DC_STAGEL_BUF_FREEZE.to_u128(),
                    &[DC_STAGEL_BUF_TYPE.to_u128(), DC_TYPE_ANY.to_u128()],
                    "__builtin_2585_buf_freeze",
                )?;
                fctx.fb.call(fn_idx)?;
                Ok(StageLType::List(inner))
            }
            UnaryOp::RequestCapability => {
                let StageLType::List(inner) = &op_ty else {
                    bail!(
                        "Invalid operand type '{op_ty}' for 'requestCapability'"
                    );
                };
                ensure!(
                    **inner == StageLType::U8,
                    "Invalid element type '{inner}' for 'requestCapability'; expected 'l/u8'"
                );
                let u8_dc = FormatId::U8
                    .dc_id()
                    .context("Missing FormatId::U8 Dc ID")?;
                let fn_idx = self.linker.link_system_builtin(
                    DC_REQUEST_HOST_CAPABILITY_OP.to_u128(),
                    &[DC_STAGEL_LIST_TYPE.to_u128(), u8_dc],
                    "__builtin_2616_request_capability",
                )?;
                fctx.fb.call(fn_idx)?;
                Ok(op_ty)
            }
            UnaryOp::TotalNeg
            | UnaryOp::IsSome
            | UnaryOp::IsNone
            | UnaryOp::IsOk
            | UnaryOp::IsErr
            | UnaryOp::ByteLen
            | UnaryOp::DcFromU128
            | UnaryOp::U128FromDc
            | UnaryOp::DcsFromS
            | UnaryOp::SFromDcs => {
                bail!("Direct WebAssembly codegen from StageR 3 (.sr3) is not yet implemented")
            }
        }
    }

    fn emit_binary_expr(
        &mut self,
        fctx: &mut FuncEmitCtx<'_>,
        op: BinaryOp,
        left: &Expr,
        right: &Expr,
    ) -> Result<StageLType> {
        let l_ty = self.emit_expr(fctx, left, false)?;
        if l_ty == StageLType::Never {
            return Ok(StageLType::Never);
        }

        // Short-circuiting `and` / `or`
        if matches!(op, BinaryOp::And | BinaryOp::Or) {
            fctx.fb.if_open(Some(WasmValType::I32))?;
            fctx.control_depth = fctx.control_depth.saturating_add(1);
            if op == BinaryOp::And {
                let r_ty = self.emit_expr(fctx, right, false)?;
                let _ = r_ty;
                fctx.fb.else_op()?;
                fctx.fb.i32_const(0)?;
            } else {
                fctx.fb.i32_const(1)?;
                fctx.fb.else_op()?;
                let r_ty = self.emit_expr(fctx, right, false)?;
                let _ = r_ty;
            }
            fctx.control_depth = fctx.control_depth.saturating_sub(1);
            fctx.fb.end_op()?;
            return Ok(StageLType::B);
        }

        let l_vt = stagel_wasm_valtype(&l_ty)
            .context("Binary left operand must have WASM value type")?;
        let l_loc = fctx.fb.alloc_local(l_vt)?;
        fctx.fb.local_set(l_loc)?;

        let r_ty = self.emit_expr(fctx, right, false)?;
        if r_ty == StageLType::Never {
            return Ok(StageLType::Never);
        }
        let r_vt = stagel_wasm_valtype(&r_ty)
            .context("Binary right operand must have WASM value type")?;
        let r_loc = fctx.fb.alloc_local(r_vt)?;
        fctx.fb.local_set(r_loc)?;

        fctx.fb.local_get(l_loc)?;
        fctx.fb.local_get(r_loc)?;

        match op {
            BinaryOp::Add => {
                let f = self.link_scalar_builtin(DC_ADD_OP, &l_ty)?;
                fctx.fb.call(f)?;
                Ok(l_ty)
            }
            BinaryOp::Sub => {
                let f = self.link_scalar_builtin(DC_SUB_OP, &l_ty)?;
                fctx.fb.call(f)?;
                Ok(l_ty)
            }
            BinaryOp::Mul => {
                let f = self.link_scalar_builtin(DC_MUL_OP, &l_ty)?;
                fctx.fb.call(f)?;
                Ok(l_ty)
            }
            BinaryOp::Div => {
                let f = self.link_scalar_builtin(DC_DIV_OP, &l_ty)?;
                fctx.fb.call(f)?;
                Ok(l_ty)
            }
            BinaryOp::Rem => {
                let f = self.link_scalar_builtin(DC_REM_OP, &l_ty)?;
                fctx.fb.call(f)?;
                Ok(l_ty)
            }
            BinaryOp::WrapAdd => {
                let f = self.link_scalar_builtin(DC_WRAP_ADD_OP, &l_ty)?;
                fctx.fb.call(f)?;
                Ok(l_ty)
            }
            BinaryOp::WrapSub => {
                let f = self.link_scalar_builtin(DC_WRAP_SUB_OP, &l_ty)?;
                fctx.fb.call(f)?;
                Ok(l_ty)
            }
            BinaryOp::WrapMul => {
                let f = self.link_scalar_builtin(DC_WRAP_MUL_OP, &l_ty)?;
                fctx.fb.call(f)?;
                Ok(l_ty)
            }
            BinaryOp::BitAnd => {
                let f = self.link_scalar_builtin(DC_BIT_AND_OP, &l_ty)?;
                fctx.fb.call(f)?;
                Ok(l_ty)
            }
            BinaryOp::BitOr => {
                let f = self.link_scalar_builtin(DC_BIT_OR_OP, &l_ty)?;
                fctx.fb.call(f)?;
                Ok(l_ty)
            }
            BinaryOp::BitXor => {
                let f = self.link_scalar_builtin(DC_BIT_XOR_OP, &l_ty)?;
                fctx.fb.call(f)?;
                Ok(l_ty)
            }
            BinaryOp::Shl => {
                let f = self.link_scalar_builtin(DC_SHL_OP, &l_ty)?;
                fctx.fb.call(f)?;
                Ok(l_ty)
            }
            BinaryOp::Shr => {
                let f = self.link_scalar_builtin(DC_SHR_OP, &l_ty)?;
                fctx.fb.call(f)?;
                Ok(l_ty)
            }
            BinaryOp::Rotl => {
                let f = self.link_scalar_builtin(DC_ROTL_OP, &l_ty)?;
                fctx.fb.call(f)?;
                Ok(l_ty)
            }
            BinaryOp::Rotr => {
                let f = self.link_scalar_builtin(DC_ROTR_OP, &l_ty)?;
                fctx.fb.call(f)?;
                Ok(l_ty)
            }
            BinaryOp::And | BinaryOp::Or => {
                bail!("Internal error: short-circuit ops handled above")
            }
            BinaryOp::Eq => {
                self.emit_eq_op(fctx, &l_ty, false)?;
                Ok(StageLType::B)
            }
            BinaryOp::Ne => {
                self.emit_eq_op(fctx, &l_ty, true)?;
                Ok(StageLType::B)
            }
            BinaryOp::Lt => {
                let f = self.link_scalar_builtin(DC_LT_OP, &l_ty)?;
                fctx.fb.call(f)?;
                Ok(StageLType::B)
            }
            BinaryOp::Le => {
                let f = self.link_scalar_builtin(DC_LE_OP, &l_ty)?;
                fctx.fb.call(f)?;
                Ok(StageLType::B)
            }
            BinaryOp::Gt => {
                let f = self.link_scalar_builtin(DC_GT_OP, &l_ty)?;
                fctx.fb.call(f)?;
                Ok(StageLType::B)
            }
            BinaryOp::Ge => {
                let f = self.link_scalar_builtin(DC_GE_OP, &l_ty)?;
                fctx.fb.call(f)?;
                Ok(StageLType::B)
            }
            BinaryOp::Get => match &l_ty {
                StageLType::List(inner) => {
                    let f = self.link_seq_or_buf_builtin(
                        DC_SEQ_GET_OP,
                        DC_STAGEL_LIST_TYPE,
                        inner,
                    )?;
                    fctx.fb.call(f)?;
                    Ok((**inner).clone())
                }
                StageLType::Buf(inner) => {
                    let f = self.link_seq_or_buf_builtin(
                        DC_SEQ_GET_OP,
                        DC_STAGEL_BUF_TYPE,
                        inner,
                    )?;
                    fctx.fb.call(f)?;
                    Ok((**inner).clone())
                }
                other => bail!("Invalid sequence type '{other}' for 'get'"),
            },
            BinaryOp::Concat => {
                let StageLType::List(ref inner) = l_ty else {
                    bail!("Invalid operand '{l_ty}' for 'concat'");
                };
                let f = self.link_seq_or_buf_builtin(
                    DC_SEQ_CONCAT_OP,
                    DC_STAGEL_LIST_TYPE,
                    inner,
                )?;
                fctx.fb.call(f)?;
                Ok(l_ty)
            }
            BinaryOp::BufFill => {
                let f = self.link_seq_or_buf_builtin(
                    DC_STAGEL_BUF_FILL,
                    DC_STAGEL_BUF_TYPE,
                    &l_ty,
                )?;
                fctx.fb.call(f)?;
                Ok(StageLType::Buf(Box::new(l_ty)))
            }
            BinaryOp::BufPush => {
                let StageLType::Buf(ref inner) = l_ty else {
                    bail!("Invalid operand '{l_ty}' for 'bufPush'");
                };
                let f = self.link_seq_or_buf_builtin(
                    DC_STAGEL_BUF_PUSH,
                    DC_STAGEL_BUF_TYPE,
                    inner,
                )?;
                fctx.fb.call(f)?;
                Ok(StageLType::Void)
            }
            BinaryOp::BufExtend => {
                let StageLType::Buf(ref inner) = l_ty else {
                    bail!("Invalid operand '{l_ty}' for 'bufExtend'");
                };
                let f = self.link_seq_or_buf_builtin(
                    DC_STAGEL_BUF_EXTEND,
                    DC_STAGEL_BUF_TYPE,
                    inner,
                )?;
                fctx.fb.call(f)?;
                Ok(StageLType::Void)
            }
            BinaryOp::TotalAdd
            | BinaryOp::TotalSub
            | BinaryOp::TotalMul
            | BinaryOp::TotalDiv
            | BinaryOp::TotalMod
            | BinaryOp::TotalTruncRem
            | BinaryOp::TotalShl
            | BinaryOp::TotalShr
            | BinaryOp::TotalGet
            | BinaryOp::UnwrapOr
            | BinaryOp::OkOr
            | BinaryOp::MakeRat
            | BinaryOp::Explode
            | BinaryOp::Implode => {
                bail!("Direct WebAssembly codegen from StageR 3 (.sr3) is not yet implemented")
            }
        }
    }

    fn emit_ternary_expr(
        &mut self,
        fctx: &mut FuncEmitCtx<'_>,
        op: TernaryOp,
        first: &Expr,
        second: &Expr,
        third: &Expr,
    ) -> Result<StageLType> {
        let t1 = self.emit_expr(fctx, first, false)?;
        if t1 == StageLType::Never {
            return Ok(StageLType::Never);
        }
        let vt1 = stagel_wasm_valtype(&t1)
            .context("Ternary arg 1 must have WASM value type")?;
        let l1 = fctx.fb.alloc_local(vt1)?;
        fctx.fb.local_set(l1)?;

        let t2 = self.emit_expr(fctx, second, false)?;
        if t2 == StageLType::Never {
            return Ok(StageLType::Never);
        }
        let vt2 = stagel_wasm_valtype(&t2)
            .context("Ternary arg 2 must have WASM value type")?;
        let l2 = fctx.fb.alloc_local(vt2)?;
        fctx.fb.local_set(l2)?;

        let t3 = self.emit_expr(fctx, third, false)?;
        if t3 == StageLType::Never {
            return Ok(StageLType::Never);
        }
        let vt3 = stagel_wasm_valtype(&t3)
            .context("Ternary arg 3 must have WASM value type")?;
        let l3 = fctx.fb.alloc_local(vt3)?;
        fctx.fb.local_set(l3)?;

        fctx.fb.local_get(l1)?;
        fctx.fb.local_get(l2)?;
        fctx.fb.local_get(l3)?;

        match op {
            TernaryOp::Slice => {
                let StageLType::List(ref inner) = t1 else {
                    bail!("Invalid sequence operand '{t1}' for 'slice'");
                };
                let f = self.link_seq_or_buf_builtin(
                    DC_SEQ_SLICE_OP,
                    DC_STAGEL_LIST_TYPE,
                    inner,
                )?;
                fctx.fb.call(f)?;
                Ok(t1)
            }
            TernaryOp::BufSet => {
                let StageLType::Buf(ref inner) = t1 else {
                    bail!("Invalid buffer operand '{t1}' for 'bufSet'");
                };
                let f = self.link_seq_or_buf_builtin(
                    DC_STAGEL_BUF_SET,
                    DC_STAGEL_BUF_TYPE,
                    inner,
                )?;
                fctx.fb.call(f)?;
                Ok(StageLType::Void)
            }
            TernaryOp::TotalSlice | TernaryOp::TotalBufSet => {
                bail!("Direct WebAssembly codegen from StageR 3 (.sr3) is not yet implemented")
            }
        }
    }

    fn emit_conv_expr(
        &mut self,
        fctx: &mut FuncEmitCtx<'_>,
        inner: &Expr,
        target_ty: &StageLType,
    ) -> Result<StageLType> {
        let src_ty = self.emit_expr(fctx, inner, false)?;
        if src_ty == StageLType::Never {
            return Ok(StageLType::Never);
        }
        let canon_target = self.canon_type(fctx.mod_idx, target_ty)?;
        let src_gid = scalar_format_gid(&src_ty)?;
        let dst_gid = scalar_format_gid(&canon_target)?;
        let key = format!(
            "__builtin_2541_{}_{}",
            type_slug(&src_ty),
            type_slug(&canon_target)
        );
        let fn_idx = self.linker.link_system_builtin(
            DC_CONVERT_OP.to_u128(),
            &[DC_TYPE_CONVERSION.to_u128(), src_gid, dst_gid],
            &key,
        )?;
        fctx.fb.call(fn_idx)?;
        Ok(canon_target)
    }

    fn emit_bitcast_expr(
        &mut self,
        fctx: &mut FuncEmitCtx<'_>,
        inner: &Expr,
        target_ty: &StageLType,
    ) -> Result<StageLType> {
        let src_ty = self.emit_expr(fctx, inner, false)?;
        if src_ty == StageLType::Never {
            return Ok(StageLType::Never);
        }
        let canon_target = self.canon_type(fctx.mod_idx, target_ty)?;
        let src_gid = scalar_format_gid(&src_ty)?;
        let dst_gid = scalar_format_gid(&canon_target)?;
        let key = format!(
            "__builtin_2542_{}_{}",
            type_slug(&src_ty),
            type_slug(&canon_target)
        );
        let fn_idx = self.linker.link_system_builtin(
            DC_BITCAST_OP.to_u128(),
            &[DC_TYPE_TRANSMUTATION.to_u128(), src_gid, dst_gid],
            &key,
        )?;
        fctx.fb.call(fn_idx)?;
        Ok(canon_target)
    }

    fn emit_eq_op(
        &mut self,
        fctx: &mut FuncEmitCtx<'_>,
        ty: &StageLType,
        negate: bool,
    ) -> Result<()> {
        let canon_ty = self.canon_type(fctx.mod_idx, ty)?;
        match &canon_ty {
            StageLType::B
            | StageLType::U8
            | StageLType::I32
            | StageLType::U32
            | StageLType::I64
            | StageLType::U64 => {
                let dc = if negate { DC_NE_OP } else { DC_EQ_OP };
                let f = self.link_scalar_builtin(dc, &canon_ty)?;
                fctx.fb.call(f)?;
                Ok(())
            }
            StageLType::List(inner)
                if matches!(
                    inner.as_ref(),
                    StageLType::B
                        | StageLType::U8
                        | StageLType::I32
                        | StageLType::U32
                        | StageLType::I64
                        | StageLType::U64
                ) =>
            {
                let dc = if negate { DC_NE_OP } else { DC_EQ_OP };
                let elem_gid = scalar_format_gid(inner)?;
                let key = format!(
                    "__builtin_{}_l_{}",
                    dc.to_u128(),
                    type_slug(inner)
                );
                let f = self.linker.link_system_builtin(
                    dc.to_u128(),
                    &[DC_STAGEL_LIST_TYPE.to_u128(), elem_gid],
                    &key,
                )?;
                fctx.fb.call(f)?;
                Ok(())
            }
            StageLType::Tuple(elems) if elems.is_empty() => {
                fctx.fb.drop_op()?;
                fctx.fb.drop_op()?;
                fctx.fb.i32_const(i32::from(!negate))?;
                Ok(())
            }
            StageLType::List(_)
            | StageLType::Tuple(_)
            | StageLType::Struct(_) => {
                let eq_fn = self.ensure_structural_eq_helper(&canon_ty)?;
                fctx.fb.call(eq_fn)?;
                if negate {
                    fctx.fb.i32_eqz()?;
                }
                Ok(())
            }
            other => bail!("Equality is not supported on '{other}'"),
        }
    }

    fn emit_inline_value_eq(
        &mut self,
        fb: &mut WasmFuncBuilder,
        canon_ty: &StageLType,
    ) -> Result<()> {
        match canon_ty {
            StageLType::B
            | StageLType::U8
            | StageLType::I32
            | StageLType::U32
            | StageLType::I64
            | StageLType::U64 => {
                let f = self.link_scalar_builtin(DC_EQ_OP, canon_ty)?;
                fb.call(f)?;
                Ok(())
            }
            StageLType::List(inner)
                if matches!(
                    inner.as_ref(),
                    StageLType::B
                        | StageLType::U8
                        | StageLType::I32
                        | StageLType::U32
                        | StageLType::I64
                        | StageLType::U64
                ) =>
            {
                let elem_gid = scalar_format_gid(inner)?;
                let key = format!(
                    "__builtin_{}_l_{}",
                    DC_EQ_OP.to_u128(),
                    type_slug(inner)
                );
                let f = self.linker.link_system_builtin(
                    DC_EQ_OP.to_u128(),
                    &[DC_STAGEL_LIST_TYPE.to_u128(), elem_gid],
                    &key,
                )?;
                fb.call(f)?;
                Ok(())
            }
            StageLType::Tuple(elems) if elems.is_empty() => {
                fb.drop_op()?;
                fb.drop_op()?;
                fb.i32_const(1)?;
                Ok(())
            }
            StageLType::List(_)
            | StageLType::Tuple(_)
            | StageLType::Struct(_) => {
                let f = self.ensure_structural_eq_helper(canon_ty)?;
                fb.call(f)?;
                Ok(())
            }
            other => bail!("Equality is not supported on '{other}'"),
        }
    }

    fn ensure_structural_eq_helper(
        &mut self,
        canon_ty: &StageLType,
    ) -> Result<u32> {
        if let Some(&existing) = self.structural_eq_helpers.get(canon_ty) {
            return Ok(existing);
        }

        let helper_name = format!("__sl_eq_{}", type_slug(canon_ty));
        let fn_idx = self.linker.reserve_routine(&helper_name)?;
        self.structural_eq_helpers.insert(canon_ty.clone(), fn_idx);

        let mut fb = WasmFuncBuilder::new(
            vec![
                (String::from("a"), WasmValType::I32),
                (String::from("b"), WasmValType::I32),
            ],
            Some(WasmValType::I32),
        );
        let a_loc = 0u32;
        let b_loc = 1u32;

        // Fast path: identical linear-memory pointer (`a == b`)
        fb.local_get(a_loc)?;
        fb.local_get(b_loc)?;
        fb.i32_eq()?;
        fb.if_open(None)?;
        fb.i32_const(1)?;
        fb.return_op()?;
        fb.end_op()?;

        match canon_ty {
            StageLType::List(inner_ty) => {
                let (stride, align_exp) = elem_storage_info(inner_ty)?;
                let stride_i32 = i32::try_from(stride)
                    .context("Element stride exceeds i32")?;
                let len_loc = fb.alloc_local(WasmValType::I32)?;
                let pa_loc = fb.alloc_local(WasmValType::I32)?;
                let pb_loc = fb.alloc_local(WasmValType::I32)?;
                let i_loc = fb.alloc_local(WasmValType::I32)?;

                fb.local_get(a_loc)?;
                fb.i32_load(2, 4)?;
                fb.local_tee(len_loc)?;
                fb.local_get(b_loc)?;
                fb.i32_load(2, 4)?;
                fb.i32_eq()?;
                fb.i32_eqz()?;
                fb.if_open(None)?;
                fb.i32_const(0)?;
                fb.return_op()?;
                fb.end_op()?;

                fb.local_get(a_loc)?;
                fb.i32_load(2, 0)?;
                fb.local_set(pa_loc)?;
                fb.local_get(b_loc)?;
                fb.i32_load(2, 0)?;
                fb.local_set(pb_loc)?;
                fb.i32_const(0)?;
                fb.local_set(i_loc)?;

                fb.block_open(None)?;
                fb.loop_open(None)?;
                fb.local_get(i_loc)?;
                fb.local_get(len_loc)?;
                fb.i32_ge_u()?;
                fb.br_if(1)?;

                fb.local_get(pa_loc)?;
                fb.local_get(i_loc)?;
                fb.i32_const(stride_i32)?;
                fb.i32_mul()?;
                fb.i32_add()?;
                match stride {
                    1 => fb.i32_load8_u(0)?,
                    4 => fb.i32_load(align_exp, 0)?,
                    8 => fb.i64_load(align_exp, 0)?,
                    other => bail!("Unsupported list eq stride {other}"),
                }

                fb.local_get(pb_loc)?;
                fb.local_get(i_loc)?;
                fb.i32_const(stride_i32)?;
                fb.i32_mul()?;
                fb.i32_add()?;
                match stride {
                    1 => fb.i32_load8_u(0)?,
                    4 => fb.i32_load(align_exp, 0)?,
                    8 => fb.i64_load(align_exp, 0)?,
                    other => bail!("Unsupported list eq stride {other}"),
                }

                self.emit_inline_value_eq(&mut fb, inner_ty)?;
                fb.i32_eqz()?;
                fb.if_open(None)?;
                fb.i32_const(0)?;
                fb.return_op()?;
                fb.end_op()?;

                fb.local_get(i_loc)?;
                fb.i32_const(1)?;
                fb.i32_add()?;
                fb.local_set(i_loc)?;
                fb.br(0)?;
                fb.end_op()?;
                fb.end_op()?;
            }
            StageLType::Tuple(_) | StageLType::Struct(_) => {
                let (_, layout) = self.composite_layout(canon_ty)?;
                for (offset, stride, align_exp, slot_ty) in layout {
                    fb.local_get(a_loc)?;
                    match stride {
                        1 => fb.i32_load8_u(u64::from(offset))?,
                        4 => fb.i32_load(align_exp, u64::from(offset))?,
                        8 => fb.i64_load(align_exp, u64::from(offset))?,
                        other => bail!("Unsupported composite eq stride {other}"),
                    }
                    fb.local_get(b_loc)?;
                    match stride {
                        1 => fb.i32_load8_u(u64::from(offset))?,
                        4 => fb.i32_load(align_exp, u64::from(offset))?,
                        8 => fb.i64_load(align_exp, u64::from(offset))?,
                        other => bail!("Unsupported composite eq stride {other}"),
                    }
                    self.emit_inline_value_eq(&mut fb, &slot_ty)?;
                    fb.i32_eqz()?;
                    fb.if_open(None)?;
                    fb.i32_const(0)?;
                    fb.return_op()?;
                    fb.end_op()?;
                }
            }
            other => bail!("Unsupported structural eq helper type '{other}'"),
        }

        fb.i32_const(1)?;
        let toks = fb.into_routine_shorthands()?;
        self.linker.define_routine(fn_idx, toks)?;
        Ok(fn_idx)
    }

    fn link_scalar_builtin(
        &mut self,
        op_dc: DcChar,
        ty: &StageLType,
    ) -> Result<u32> {
        let gid = scalar_format_gid(ty)?;
        let key = format!("__builtin_{}_{}", op_dc.to_u128(), type_slug(ty));
        self.linker
            .link_system_builtin(op_dc.to_u128(), &[gid], &key)
    }

    fn link_seq_or_buf_builtin(
        &mut self,
        op_dc: DcChar,
        container_dc: DcChar,
        elem_ty: &StageLType,
    ) -> Result<u32> {
        let mut ctx = vec![container_dc.to_u128()];
        let elem_tag = match elem_ty {
            StageLType::B
            | StageLType::U8
            | StageLType::I32
            | StageLType::U32
            | StageLType::I64
            | StageLType::U64 => {
                ctx.push(scalar_format_gid(elem_ty)?);
                type_slug(elem_ty)
            }
            StageLType::List(_)
            | StageLType::Tuple(_)
            | StageLType::Struct(_)
            | StageLType::Buf(_) => {
                ctx.push(DC_STAGEL_LIST_TYPE.to_u128());
                ctx.push(DC_TYPE_ANY.to_u128());
                String::from("list_ptr")
            }
            other => bail!("Invalid sequence element type '{other}'"),
        };
        let key = format!(
            "__builtin_{}_{}_{elem_tag}",
            op_dc.to_u128(),
            container_dc.to_u128()
        );
        self.linker
            .link_system_builtin(op_dc.to_u128(), &ctx, &key)
    }

    fn infer_expr_type(
        &self,
        fctx: &FuncEmitCtx<'_>,
        expr: &Expr,
    ) -> Result<StageLType> {
        match expr {
            Expr::Bool { .. } => Ok(StageLType::B),
            Expr::Int { value, .. } => Ok(value.stage_l_type()),
            Expr::Bytes { .. } => Ok(StageLType::List(Box::new(StageLType::U8))),
            Expr::List { elem_ty, .. } => {
                Ok(StageLType::List(Box::new(
                    self.canon_type(fctx.mod_idx, elem_ty)?,
                )))
            }
            Expr::TupleNew { elements, .. } => {
                let mut slot_tys = Vec::with_capacity(elements.len());
                for el in elements {
                    let ety = self.infer_expr_type(fctx, el)?;
                    if ety == StageLType::Never {
                        return Ok(StageLType::Never);
                    }
                    slot_tys.push(self.canon_type(fctx.mod_idx, &ety)?);
                }
                Ok(StageLType::Tuple(slot_tys))
            }
            Expr::StructNew {
                target,
                fields,
                span,
            } => {
                for f in fields {
                    if self.infer_expr_type(fctx, f)? == StageLType::Never {
                        return Ok(StageLType::Never);
                    }
                }
                let vm = self
                    .workspace
                    .modules
                    .get(fctx.mod_idx)
                    .context("Invalid module index")?;
                if let Some(ty) = vm.expr_types.get(&(span.line, span.col)) {
                    Ok(vm.canonicalize_type(ty))
                } else {
                    let raw = match target {
                        SymbolTarget::Ident(name) => {
                            StageLType::Struct(StructTypeRef::Ident(name.clone()))
                        }
                        SymbolTarget::Dc(dc) => {
                            StageLType::Struct(StructTypeRef::Dc(*dc))
                        }
                    };
                    Ok(vm.canonicalize_type(&raw))
                }
            }
            Expr::Field { base, index, .. } => {
                let base_ty = self.infer_expr_type(fctx, base)?;
                if base_ty == StageLType::Never {
                    return Ok(StageLType::Never);
                }
                let canon_base = self.canon_type(fctx.mod_idx, &base_ty)?;
                let (_, layout) = self.composite_layout(&canon_base)?;
                let idx =
                    usize::try_from(*index).context("Field index exceeds usize")?;
                let (_, _, _, slot_ty) = layout
                    .get(idx)
                    .cloned()
                    .with_context(|| format!("Field index {index} out of bounds"))?;
                Ok(slot_ty)
            }
            Expr::With { base, value, .. } => {
                let base_ty = self.infer_expr_type(fctx, base)?;
                if base_ty == StageLType::Never
                    || self.infer_expr_type(fctx, value)? == StageLType::Never
                {
                    return Ok(StageLType::Never);
                }
                self.canon_type(fctx.mod_idx, &base_ty)
            }
            Expr::BufNew { elem_ty, .. } => {
                Ok(StageLType::Buf(Box::new(
                    self.canon_type(fctx.mod_idx, elem_ty)?,
                )))
            }
            Expr::ValueRef { ty, target, span } => {
                if let Some(explicit_ty) = ty {
                    return self.canon_type(fctx.mod_idx, explicit_ty);
                }
                let vm = self
                    .workspace
                    .modules
                    .get(fctx.mod_idx)
                    .context("Invalid module index")?;
                if let Some(resolved_ty) =
                    vm.value_ref_types.get(&(span.line, span.col))
                {
                    return Ok(resolved_ty.clone());
                }
                if let SymbolTarget::Ident(name) = target
                    && let Some(var_entry) = fctx.lookup_var(name)
                {
                    return Ok(var_entry.ty().clone());
                }
                bail!("Missing resolved type for value reference at {span}")
            }
            Expr::Let { .. }
            | Expr::Const { .. }
            | Expr::Set { .. }
            | Expr::RoutineDecl(_)
            | Expr::Nop { .. }
            | Expr::While { .. }
            | Expr::ForEach { .. } => Ok(StageLType::Void),
            Expr::Break { .. }
            | Expr::Continue { .. }
            | Expr::Return { .. }
            | Expr::Fault { .. } => Ok(StageLType::Never),
            Expr::AnonRoutine(r) => bail!(
                "Anonymous routine at {} has no standalone value type",
                r.span
            ),
            Expr::InlineCall { span, .. } => bail!(
                "Internal WASM codegen error: unlowered inline routine call at {span}"
            ),
            Expr::Call { target, span, .. } => {
                let callee = self.resolve_callee(fctx, target, *span)?;
                Ok(callee.sig().return_type.clone())
            }
            Expr::Block { stmts, .. } => {
                let exprs: Vec<&Expr> = stmts
                    .iter()
                    .filter_map(|it| match it {
                        BlockItem::Expr(e) => Some(e),
                        BlockItem::Comment(_) => None,
                    })
                    .collect();
                let Some(last) = exprs.last() else {
                    return Ok(StageLType::Void);
                };
                // Account for nested routines declared inside `stmts`
                let mut temp_routines = HashMap::new();
                for e in &exprs {
                    if let Expr::RoutineDecl(r) = e {
                        let mut canon_params = Vec::with_capacity(r.params.len());
                        for p in &r.params {
                            canon_params.push(self.canon_type(fctx.mod_idx, &p.ty)?);
                        }
                        temp_routines.insert(
                            r.name.clone(),
                            NestedRoutineEntry {
                                func_idx: 0,
                                declaring_routine_id: fctx.routine_uid,
                                sig: RoutineSig {
                                    return_type: self
                                        .canon_type(fctx.mod_idx, &r.return_type)?,
                                    params: canon_params,
                                    is_pub: false,
                                    dc_id: r.dc_id,
                                    may_fault: false,
                                },
                                captures: Vec::new(),
                            },
                        );
                    }
                }
                for e in exprs.iter().take(exprs.len().saturating_sub(1)) {
                    if self.infer_expr_type_with_extra(fctx, e, &temp_routines)?
                        == StageLType::Never
                    {
                        return Ok(StageLType::Never);
                    }
                }
                self.infer_expr_type_with_extra(fctx, last, &temp_routines)
            }
            Expr::If {
                cond,
                then_branch,
                else_branch,
                ..
            } => {
                if self.infer_expr_type(fctx, cond)? == StageLType::Never {
                    return Ok(StageLType::Never);
                }
                let t = self.infer_expr_type(fctx, then_branch)?;
                let e = self.infer_expr_type(fctx, else_branch)?;
                Ok(match (t, e) {
                    (StageLType::Never, other) => other,
                    (other, _) => other,
                })
            }
            Expr::Unary { op, operand, .. } => {
                let oty = self.infer_expr_type(fctx, operand)?;
                if oty == StageLType::Never {
                    return Ok(StageLType::Never);
                }
                Ok(match op {
                    UnaryOp::Neg | UnaryOp::BitNot => oty,
                    UnaryOp::Not => StageLType::B,
                    UnaryOp::Clz | UnaryOp::Ctz | UnaryOp::Popcnt => {
                        StageLType::U32
                    }
                    UnaryOp::Len => StageLType::U64,
                    UnaryOp::BufFreeze => match oty {
                        StageLType::Buf(inner) => StageLType::List(inner),
                        other => other,
                    },
                    UnaryOp::RequestCapability => {
                        StageLType::List(Box::new(StageLType::U8))
                    }
                    UnaryOp::TotalNeg
                    | UnaryOp::IsSome
                    | UnaryOp::IsNone
                    | UnaryOp::IsOk
                    | UnaryOp::IsErr
                    | UnaryOp::ByteLen
                    | UnaryOp::DcFromU128
                    | UnaryOp::U128FromDc
                    | UnaryOp::DcsFromS
                    | UnaryOp::SFromDcs => {
                        bail!("Direct WebAssembly codegen from StageR 3 (.sr3) is not yet implemented")
                    }
                })
            }
            Expr::Binary {
                op, left, right, ..
            } => {
                let lty = self.infer_expr_type(fctx, left)?;
                if lty == StageLType::Never {
                    return Ok(StageLType::Never);
                }
                if !matches!(op, BinaryOp::And | BinaryOp::Or)
                    && self.infer_expr_type(fctx, right)? == StageLType::Never
                {
                    return Ok(StageLType::Never);
                }
                Ok(match op {
                    BinaryOp::Add
                    | BinaryOp::Sub
                    | BinaryOp::Mul
                    | BinaryOp::Div
                    | BinaryOp::Rem
                    | BinaryOp::WrapAdd
                    | BinaryOp::WrapSub
                    | BinaryOp::WrapMul
                    | BinaryOp::BitAnd
                    | BinaryOp::BitOr
                    | BinaryOp::BitXor
                    | BinaryOp::Shl
                    | BinaryOp::Shr
                    | BinaryOp::Rotl
                    | BinaryOp::Rotr
                    | BinaryOp::Concat => lty,
                    BinaryOp::And
                    | BinaryOp::Or
                    | BinaryOp::Eq
                    | BinaryOp::Ne
                    | BinaryOp::Lt
                    | BinaryOp::Le
                    | BinaryOp::Gt
                    | BinaryOp::Ge => StageLType::B,
                    BinaryOp::Get => match lty {
                        StageLType::List(inner) | StageLType::Buf(inner) => {
                            *inner
                        }
                        other => other,
                    },
                    BinaryOp::BufFill => StageLType::Buf(Box::new(lty)),
                    BinaryOp::BufPush | BinaryOp::BufExtend => StageLType::Void,
                    BinaryOp::TotalAdd
                    | BinaryOp::TotalSub
                    | BinaryOp::TotalMul
                    | BinaryOp::TotalDiv
                    | BinaryOp::TotalMod
                    | BinaryOp::TotalTruncRem
                    | BinaryOp::TotalShl
                    | BinaryOp::TotalShr
                    | BinaryOp::TotalGet
                    | BinaryOp::UnwrapOr
                    | BinaryOp::OkOr
                    | BinaryOp::MakeRat
                    | BinaryOp::Explode
                    | BinaryOp::Implode => {
                        bail!("Direct WebAssembly codegen from StageR 3 (.sr3) is not yet implemented")
                    }
                })
            }
            Expr::Ternary {
                op,
                first,
                second,
                third,
                ..
            } => {
                let t1 = self.infer_expr_type(fctx, first)?;
                if t1 == StageLType::Never
                    || self.infer_expr_type(fctx, second)? == StageLType::Never
                    || self.infer_expr_type(fctx, third)? == StageLType::Never
                {
                    return Ok(StageLType::Never);
                }
                Ok(match op {
                    TernaryOp::Slice => t1,
                    TernaryOp::BufSet => StageLType::Void,
                    TernaryOp::TotalSlice | TernaryOp::TotalBufSet => {
                        bail!("Direct WebAssembly codegen from StageR 3 (.sr3) is not yet implemented")
                    }
                })
            }
            Expr::Conv {
                total: false,
                expr: inner,
                target_ty,
                ..
            }
            | Expr::Bitcast {
                expr: inner,
                target_ty,
                ..
            } => {
                if self.infer_expr_type(fctx, inner)? == StageLType::Never {
                    return Ok(StageLType::Never);
                }
                self.canon_type(fctx.mod_idx, target_ty)
            }
            Expr::Conv { total: true, .. }
            | Expr::DctsString { .. }
            | Expr::DcLit { .. }
            | Expr::NamedStructNew { .. }
            | Expr::EnumVariant { .. }
            | Expr::OptionNone { .. }
            | Expr::OptionSome { .. }
            | Expr::ResultOk { .. }
            | Expr::ResultErr { .. }
            | Expr::ErrorTag { .. }
            | Expr::NamedField { .. }
            | Expr::NamedWith { .. }
            | Expr::WrapNominal { .. }
            | Expr::UnwrapNominal { .. }
            | Expr::Match { .. }
            | Expr::Propagate { .. } => {
                bail!("Direct WebAssembly codegen from StageR 3 (.sr3) is not yet implemented")
            }
        }
    }

    fn infer_expr_type_with_extra(
        &self,
        fctx: &FuncEmitCtx<'_>,
        expr: &Expr,
        extra_routines: &HashMap<String, NestedRoutineEntry>,
    ) -> Result<StageLType> {
        if let Expr::Call {
            target: SymbolTarget::Ident(name),
            ..
        } = expr
            && let Some(nr) = extra_routines.get(name)
        {
            return Ok(nr.sig.return_type.clone());
        }
        self.infer_expr_type(fctx, expr)
    }
}

struct FuncEmitCtx<'b> {
    mod_idx: usize,
    routine_uid: usize,
    is_nested: bool,
    return_type: StageLType,
    entry_sp_local: Option<u32>,
    caller_restore_sp_local: Option<u32>,
    fb: &'b mut WasmFuncBuilder,
    scopes: Vec<ScopeLayer>,
    loop_stack: Vec<LoopLabels>,
    control_depth: u32,
    nested_counter: usize,
}

impl FuncEmitCtx<'_> {
    fn lookup_var(&self, name: &str) -> Option<&VarBinding> {
        for layer in self.scopes.iter().rev() {
            if let Some(v) = layer.vars.get(name) {
                return Some(v);
            }
        }
        None
    }

    fn lookup_nested_routine(&self, name: &str) -> Option<&NestedRoutineEntry> {
        for layer in self.scopes.iter().rev() {
            if let Some(r) = layer.routines.get(name) {
                return Some(r);
            }
        }
        None
    }

    fn enclosing_routine_captures_map(&self) -> HashMap<String, Vec<String>> {
        let mut out = HashMap::new();
        for layer in &self.scopes {
            for (rname, entry) in &layer.routines {
                out.insert(
                    rname.clone(),
                    entry.captures.iter().map(|c| c.sl_name.clone()).collect(),
                );
            }
        }
        out
    }

    fn visible_capture_modes(
        &self,
    ) -> HashMap<String, (StageLType, CapturePassMode)> {
        let mut out = HashMap::new();
        for layer in &self.scopes {
            for (vname, binding) in &layer.vars {
                match binding {
                    VarBinding::Cell { ty, .. } => {
                        out.insert(
                            vname.clone(),
                            (ty.clone(), CapturePassMode::CellPtr),
                        );
                    }
                    VarBinding::Local { ty, .. } => {
                        if let Some(vt) = stagel_wasm_valtype(ty) {
                            out.insert(
                                vname.clone(),
                                (ty.clone(), CapturePassMode::ImmVal(vt)),
                            );
                        }
                    }
                }
            }
        }
        out
    }
}

fn workspace_has_nested_routines(workspace: &ValidatedWorkspace) -> bool {
    for vm in &workspace.modules {
        for item in &vm.module.items {
            match item {
                TopDecl::Routine(r) => {
                    if expr_contains_nested_routine(&r.body) {
                        return true;
                    }
                }
                TopDecl::Value(v) => {
                    if expr_contains_nested_routine(&v.value) {
                        return true;
                    }
                }
                TopDecl::Struct(_)
                | TopDecl::Comment(_)
                | TopDecl::BrandType(_)
                | TopDecl::NamedStruct(_)
                | TopDecl::Enum(_) => {}
            }
        }
    }
    false
}

fn expr_contains_nested_routine(expr: &Expr) -> bool {
    match expr {
        Expr::RoutineDecl(_) => true,
        Expr::Block { stmts, .. } => stmts.iter().any(|it| match it {
            BlockItem::Expr(e) => expr_contains_nested_routine(e),
            BlockItem::Comment(_) => false,
        }),
        Expr::If {
            cond,
            then_branch,
            else_branch,
            ..
        } => {
            expr_contains_nested_routine(cond)
                || expr_contains_nested_routine(then_branch)
                || expr_contains_nested_routine(else_branch)
        }
        Expr::While { cond, body, .. } => {
            expr_contains_nested_routine(cond)
                || expr_contains_nested_routine(body)
        }
        Expr::ForEach { list, body, .. } => {
            expr_contains_nested_routine(list)
                || expr_contains_nested_routine(body)
        }
        Expr::Let { init: Some(e), .. }
        | Expr::Const { value: e, .. }
        | Expr::Set { value: e, .. }
        | Expr::Return { value: Some(e), .. }
        | Expr::Unary { operand: e, .. }
        | Expr::Field { base: e, .. }
        | Expr::Conv { expr: e, .. }
        | Expr::Bitcast { expr: e, .. }
        | Expr::BufNew { capacity: e, .. } => expr_contains_nested_routine(e),
        Expr::Call { args, .. }
        | Expr::List { elements: args, .. }
        | Expr::TupleNew { elements: args, .. }
        | Expr::StructNew { fields: args, .. } => {
            args.iter().any(expr_contains_nested_routine)
        }
        Expr::Binary { left, right, .. }
        | Expr::With {
            base: left,
            value: right,
            ..
        } => {
            expr_contains_nested_routine(left)
                || expr_contains_nested_routine(right)
        }
        Expr::Ternary {
            first,
            second,
            third,
            ..
        } => {
            expr_contains_nested_routine(first)
                || expr_contains_nested_routine(second)
                || expr_contains_nested_routine(third)
        }
        _ => false,
    }
}

fn collect_set_targets_deep(expr: &Expr, out: &mut HashSet<String>) {
    match expr {
        Expr::Set { name, value, .. } => {
            out.insert(name.clone());
            collect_set_targets_deep(value, out);
        }
        Expr::RoutineDecl(r) => {
            collect_set_targets_deep(&r.body, out);
        }
        Expr::Block { stmts, .. } => {
            for it in stmts {
                if let BlockItem::Expr(e) = it {
                    collect_set_targets_deep(e, out);
                }
            }
        }
        Expr::If {
            cond,
            then_branch,
            else_branch,
            ..
        } => {
            collect_set_targets_deep(cond, out);
            collect_set_targets_deep(then_branch, out);
            collect_set_targets_deep(else_branch, out);
        }
        Expr::While { cond, body, .. } => {
            collect_set_targets_deep(cond, out);
            collect_set_targets_deep(body, out);
        }
        Expr::ForEach { list, body, .. } => {
            collect_set_targets_deep(list, out);
            collect_set_targets_deep(body, out);
        }
        Expr::Let { init: Some(e), .. }
        | Expr::Const { value: e, .. }
        | Expr::Return { value: Some(e), .. }
        | Expr::Unary { operand: e, .. }
        | Expr::Field { base: e, .. }
        | Expr::Conv { expr: e, .. }
        | Expr::Bitcast { expr: e, .. }
        | Expr::BufNew { capacity: e, .. } => collect_set_targets_deep(e, out),
        Expr::Call { args, .. }
        | Expr::List { elements: args, .. }
        | Expr::TupleNew { elements: args, .. }
        | Expr::StructNew { fields: args, .. } => {
            for arg in args {
                collect_set_targets_deep(arg, out);
            }
        }
        Expr::Binary { left, right, .. }
        | Expr::With {
            base: left,
            value: right,
            ..
        } => {
            collect_set_targets_deep(left, out);
            collect_set_targets_deep(right, out);
        }
        Expr::Ternary {
            first,
            second,
            third,
            ..
        } => {
            collect_set_targets_deep(first, out);
            collect_set_targets_deep(second, out);
            collect_set_targets_deep(third, out);
        }
        _ => {}
    }
}
