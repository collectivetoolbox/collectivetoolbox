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

//! Host offline WebAssembly-to-safe-Rust substitute transpiler with exact V2
//! structural-checkpoint metering (`V2CheckpointPlanner`) parity.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use anyhow::anyhow;
use ctb_formats_dcdata::tinywasm::types::{
    ConstInstruction, DataKind, ElementItem, ElementKind, ExternalKind,
    FuncType, Global, MemoryArch, TinyWasmModule, ValType,
};
use sha2::{Digest, Sha256};

use super::wasm::WASM_RESOURCE_ACCOUNTING_VERSION;
use super::wasm_quota::{
    CTB_CAPABILITY_YIELD_EXPORT, SEC_CODE, V2CheckpointPlanner, WASM_HEADER,
    WasmBlockType, compute_static_data_end, eval_const_offset, read_blocktype,
    read_byte, read_memarg, read_sleb_i32, read_sleb_i64, read_uleb_u32,
};

const MAX_SHARED_GLOBAL_PARAMS: usize = 8;
const MAX_SHARED_FNS_PER_SHARD: usize = 120;
const INLINE_NEVER_BYTECODE_THRESHOLD: usize = 128;

/// Options controlling generated Rust module metadata and source hashing.
#[derive(Debug, Clone, Copy, Default)]
pub struct TranspileWasmOptions<'a> {
    /// Human-readable substitute label (e.g. `"dc_1936"`, `"f1680"`).
    pub name: &'a str,
    /// Optional generator input fingerprint emitted as `// input-sha256: <hex>`.
    pub input_sha256_hex: Option<&'a str>,
}

/// Returns a deterministic SHA-256 fingerprint of the transpiler and V2 metering
/// planner sources so offline substitute regeneration invalidates whenever codegen changes.
#[must_use]
pub fn transpiler_fingerprint_sha256() -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"ctb-wasm-to-rust-v2");
    hasher.update(WASM_RESOURCE_ACCOUNTING_VERSION.to_le_bytes());
    hasher.update(include_bytes!("wasm_to_rust.rs"));
    hasher.update(include_bytes!("wasm_quota.rs"));
    hasher.update(include_bytes!("wasm.rs"));
    hasher.finalize().into()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum RustValTy {
    I32,
    I64,
    F32,
    F64,
    Ref,
}

impl RustValTy {
    fn from_val_type(vt: ValType) -> Result<Self> {
        match vt {
            ValType::I32 => Ok(Self::I32),
            ValType::I64 => Ok(Self::I64),
            ValType::F32 => Ok(Self::F32),
            ValType::F64 => Ok(Self::F64),
            ValType::RefFunc | ValType::RefExtern => Ok(Self::Ref),
            ValType::V128 => {
                bail!("WebAssembly SIMD v128 values are not supported in native substitutes")
            }
        }
    }

    fn from_wire_byte(b: u8) -> Result<Self> {
        match b {
            0x7F => Ok(Self::I32),
            0x7E => Ok(Self::I64),
            0x7D => Ok(Self::F32),
            0x7C => Ok(Self::F64),
            0x70 | 0x6F => Ok(Self::Ref),
            0x7B => bail!(
                "WebAssembly SIMD v128 locals/blocks are not supported in native substitutes"
            ),
            other => {
                bail!("Unsupported WebAssembly value type byte {other:#04x}")
            }
        }
    }

    const fn rust_ty(self) -> &'static str {
        match self {
            Self::I32 => "i32",
            Self::I64 => "i64",
            Self::F32 => "f32",
            Self::F64 => "f64",
            Self::Ref => "Option<u32>",
        }
    }

    const fn zero_expr(self) -> &'static str {
        match self {
            Self::I32 => "0i32",
            Self::I64 => "0i64",
            Self::F32 => "0.0f32",
            Self::F64 => "0.0f64",
            Self::Ref => "None",
        }
    }

    const fn slug(self) -> &'static str {
        match self {
            Self::I32 => "i32",
            Self::I64 => "i64",
            Self::F32 => "f32",
            Self::F64 => "f64",
            Self::Ref => "ref",
        }
    }

    const fn wire_tag(self) -> u8 {
        match self {
            Self::I32 => 1,
            Self::I64 => 2,
            Self::F32 => 3,
            Self::F64 => 4,
            Self::Ref => 5,
        }
    }

    const fn array_idx(self) -> usize {
        match self {
            Self::I32 => 0,
            Self::I64 => 1,
            Self::F32 => 2,
            Self::F64 => 3,
            Self::Ref => 4,
        }
    }
}

fn slot_ident(depth: usize, ty: RustValTy) -> String {
    format!("s_{}_{depth}", ty.slug())
}

fn ret_type_expr(results: &[RustValTy]) -> String {
    match results {
        [] => "()".to_owned(),
        [single] => single.rust_ty().to_owned(),
        many => {
            let parts: Vec<&str> = many.iter().map(|t| t.rust_ty()).collect();
            format!("({})", parts.join(", "))
        }
    }
}

fn ret_sig_slug(results: &[RustValTy]) -> String {
    if results.is_empty() {
        "void".to_owned()
    } else {
        let parts: Vec<&str> = results.iter().map(|t| t.slug()).collect();
        parts.join("_")
    }
}

fn rust_i32_literal(v: i32) -> String {
    if v == i32::MIN {
        "i32::MIN".to_owned()
    } else {
        format!("{v}i32")
    }
}

fn rust_i64_literal(v: i64) -> String {
    if v == i64::MIN {
        "i64::MIN".to_owned()
    } else {
        format!("{v}i64")
    }
}

fn rust_f32_literal(v: f32) -> String {
    format!("f32::from_bits(0x{:08x}u32)", v.to_bits())
}

fn rust_f64_literal(v: f64) -> String {
    format!("f64::from_bits(0x{:016x}u64)", v.to_bits())
}

fn emit_agpl_banner(out: &mut String) {
    out.push_str(
        "// SPDX-License-Identifier: AGPL-3.0-or-later\n\
         /*\n\
         This file is part of Collective Toolbox, a database and document workspace and utilities.\n\
         Copyright (C) 2026 Collective Toolbox Developers\n\
         Contact: info@collectivetoolbox.com\n\
         \n\
         This program is free software: you can redistribute it and/or modify it under\n\
         the terms of the GNU Affero General Public License as published by the Free\n\
         Software Foundation, either version 3 of the License, or (at your option) any\n\
         later version.\n\
         \n\
         This program is distributed in the hope that it will be useful, but WITHOUT ANY\n\
         WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR\n\
         A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.\n\
         \n\
         You should have received a copy of the GNU Affero General Public License along\n\
         with this program.  If not, see <https://www.gnu.org/licenses/>.\n\
         */\n\
         \n",
    );
}

fn emit_agpl_header(
    out: &mut String,
    module_name: &str,
    input_sha256_hex: Option<&str>,
) {
    emit_agpl_banner(out);
    let _ = writeln!(
        out,
        "//! Generated native Rust WebAssembly substitute for `{module_name}`."
    );
    out.push_str(
        "//! Do not edit by hand; regenerate via `./scripts/generate-wasm-substitutes`.\n",
    );
    if let Some(hex) = input_sha256_hex {
        let _ = writeln!(out, "// input-sha256: {hex}");
    }
    out.push_str(
        "\n#[allow(\n    unused_imports,\n    clippy::wildcard_imports,\n    reason = \"Standard workspace module prelude\"\n)]\nuse crate::utilities::*;\n\n\
         #[allow(\n    unused_imports,\n    reason = \"Generated substitute runtime imports\"\n)]\n\
         use ctb_wasm_substitute_rt::{\n\
         \x20   InitFnPtr, SubstituteModuleMeta, SubstituteVm, WasmLinearMemory,\n\
         \x20   WasmSubstituteDescriptor, WasmSubstituteHost, WasmSubstituteInstance,\n\
         \x20   WasmTable, WasmTrap, ops,\n\
         };\n\n",
    );
}

fn emit_shared_header(out: &mut String, shard_idx: usize) {
    emit_agpl_banner(out);
    let _ = writeln!(
        out,
        "//! Generated shared Merkle-deduplicated WASM substitute functions (shard {shard_idx})."
    );
    out.push_str(
        "//! Do not edit by hand; regenerate via `./scripts/generate-wasm-substitutes`.\n\n\
         #[allow(\n    unused_imports,\n    clippy::wildcard_imports,\n    reason = \"Standard workspace module prelude\"\n)]\nuse crate::utilities::*;\n\n\
         #[allow(\n    unused_imports,\n    reason = \"Generated shared substitute runtime imports\"\n)]\n\
         use ctb_wasm_substitute_rt::{\n\
         \x20   SubstituteVm, WasmSubstituteHost, WasmTrap, ops,\n\
         };\n\n",
    );
}

fn emit_registry_header(out: &mut String, input_sha256_hex: Option<&str>) {
    emit_agpl_banner(out);
    out.push_str(
        "// @generated by ctb-wasm-substitutes-gen\n\
         // Do not edit by hand; regenerate via `./scripts/generate-wasm-substitutes`.\n",
    );
    if let Some(hex) = input_sha256_hex {
        let _ = writeln!(out, "// input-sha256: {hex}");
    }
    out.push('\n');
}

fn emit_bytes_const(out: &mut String, const_name: &str, bytes: &[u8]) {
    let _ = writeln!(out, "const {const_name}: &[u8] = b\"\\");
    let mut line = String::from("    ");
    for &b in bytes {
        let piece = if b == b'\\' {
            "\\\\".to_owned()
        } else if b == b'"' {
            "\\\"".to_owned()
        } else if b.is_ascii_graphic() {
            char::from(b).to_string()
        } else {
            format!("\\x{b:02x}")
        };
        if line.len().saturating_add(piece.len()) > 78 {
            line.push('\\');
            out.push_str(&line);
            out.push('\n');
            line.clear();
            line.push_str("    ");
        }
        line.push_str(&piece);
    }
    line.push_str("\";\n");
    out.push_str(&line);
}

struct CodeBodyRaw<'a> {
    non_param_locals: Vec<RustValTy>,
    code_bytes: &'a [u8],
}

fn extract_raw_code_bodies(wasm_bytes: &[u8]) -> Result<Vec<CodeBodyRaw<'_>>> {
    ensure!(
        wasm_bytes.len() >= WASM_HEADER.len()
            && wasm_bytes.get(..WASM_HEADER.len()) == Some(&WASM_HEADER),
        "Invalid WASM header when extracting Section 10 code bodies"
    );
    let mut pos = WASM_HEADER.len();
    let mut code_payload: Option<&[u8]> = None;
    while pos < wasm_bytes.len() {
        let sec_id = read_byte(wasm_bytes, &mut pos)?;
        let sec_len = usize::try_from(read_uleb_u32(wasm_bytes, &mut pos)?)?;
        let end = pos
            .checked_add(sec_len)
            .context("WASM section length overflow")?;
        let payload = wasm_bytes
            .get(pos..end)
            .context("WASM section extends past EOF")?;
        pos = end;
        if sec_id == SEC_CODE {
            code_payload = Some(payload);
            break;
        }
    }
    let Some(code_bytes) = code_payload else {
        return Ok(Vec::new());
    };
    let mut cpos = 0usize;
    let body_count = usize::try_from(read_uleb_u32(code_bytes, &mut cpos)?)?;
    let mut bodies = Vec::with_capacity(body_count);
    for _ in 0..body_count {
        let body_len = usize::try_from(read_uleb_u32(code_bytes, &mut cpos)?)?;
        let body_end = cpos
            .checked_add(body_len)
            .context("WASM function body length overflow")?;
        let raw = code_bytes
            .get(cpos..body_end)
            .context("WASM function body extends past Section 10")?;
        cpos = body_end;

        let mut bpos = 0usize;
        let group_count = read_uleb_u32(raw, &mut bpos)?;
        let mut non_param_locals = Vec::new();
        for _ in 0..group_count {
            let count = usize::try_from(read_uleb_u32(raw, &mut bpos)?)?;
            let vt = RustValTy::from_wire_byte(read_byte(raw, &mut bpos)?)?;
            non_param_locals.resize(
                non_param_locals.len().saturating_add(count),
                vt,
            );
        }
        let instr_slice = raw
            .get(bpos..)
            .context("WASM function body truncated before instruction stream")?;
        bodies.push(CodeBodyRaw {
            non_param_locals,
            code_bytes: instr_slice,
        });
    }
    Ok(bodies)
}

#[derive(Debug, Clone)]
struct FnSigInfo {
    params: Vec<RustValTy>,
    results: Vec<RustValTy>,
}

fn convert_func_type(ft: &FuncType) -> Result<FnSigInfo> {
    let mut params = Vec::with_capacity(ft.params.len());
    for &p in &ft.params {
        params.push(RustValTy::from_val_type(p)?);
    }
    let mut results = Vec::with_capacity(ft.results.len());
    for &r in &ft.results {
        results.push(RustValTy::from_val_type(r)?);
    }
    Ok(FnSigInfo { params, results })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EntrySignatureKind {
    UnitToI32,
    I32PairToI32,
    UnitToUnit,
    I32PairToUnit,
}

fn classify_entry_sig(sig: &FnSigInfo) -> Result<EntrySignatureKind> {
    match (sig.params.as_slice(), sig.results.as_slice()) {
        ([], [RustValTy::I32]) => Ok(EntrySignatureKind::UnitToI32),
        ([RustValTy::I32, RustValTy::I32], [RustValTy::I32]) => {
            Ok(EntrySignatureKind::I32PairToI32)
        }
        ([], []) => Ok(EntrySignatureKind::UnitToUnit),
        ([RustValTy::I32, RustValTy::I32], []) => {
            Ok(EntrySignatureKind::I32PairToUnit)
        }
        _ => bail!(
            "Exported WASM entry function must have signature () -> i32, (i32, i32) -> i32, () -> (), or (i32, i32) -> ()"
        ),
    }
}

fn emit_const_instruction(
    instr: &ConstInstruction,
    globals: &[Global],
    global_prefix: &str,
) -> Result<String> {
    match instr {
        ConstInstruction::I32Const(v) => Ok(rust_i32_literal(*v)),
        ConstInstruction::I64Const(v) => Ok(rust_i64_literal(*v)),
        ConstInstruction::F32Const(v) => Ok(rust_f32_literal(*v)),
        ConstInstruction::F64Const(v) => Ok(rust_f64_literal(*v)),
        ConstInstruction::GlobalGet(idx) => {
            let idx_usize = usize::try_from(*idx)?;
            ensure!(
                idx_usize < globals.len(),
                "Constant expression global.get {idx_usize} out of bounds"
            );
            Ok(format!("{global_prefix}g{idx_usize}"))
        }
        ConstInstruction::RefNull(_) => Ok("None".to_owned()),
        ConstInstruction::RefFunc(idx) => Ok(format!("Some({idx}u32)")),
    }
}

fn eval_const_usize_expr(
    instr: &ConstInstruction,
    globals: &[Global],
) -> Result<usize> {
    if let Some(off) = eval_const_offset(instr, globals) {
        return Ok(off);
    }
    bail!("Unsupported active segment constant offset expression: {instr:?}")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum GlobalSlot {
    HeapPtr,
    Typed(RustValTy, usize),
}

#[derive(Debug, Clone)]
struct FuncMerkleInfo {
    hash: [u8; 32],
    used_globals: Vec<(RustValTy, usize)>,
    callee_hashes: Vec<[u8; 32]>,
}

struct PromotedSharedTarget {
    hash: [u8; 32],
    symbol_name: String,
    shard_idx: usize,
    shard_mod: String,
    repr_mod_idx: usize,
    repr_fn_idx: usize,
}

struct AnalyzedModule<'a> {
    name: &'a str,
    wasm_sha256: [u8; 32],
    parsed: TinyWasmModule,
    code_bodies: Vec<CodeBodyRaw<'a>>,
    initial_memory_pages: usize,
    initial_table_elements: usize,
    static_data_end: usize,
    block_types: Vec<FnSigInfo>,
    fn_sigs: Vec<FnSigInfo>,
    entry_name: &'static str,
    entry_idx: usize,
    entry_sig: EntrySignatureKind,
    memory_export_idx: Option<usize>,
    cap_yield_func_idx: Option<usize>,
    heap_ptr_global_idx: Option<usize>,
    global_slots: Vec<GlobalSlot>,
    scc_id: Vec<usize>,
    tail_target_fns: BTreeSet<usize>,
    has_self_tail: Vec<bool>,
    init_fn_idx: Option<usize>,
    merkle_info: Vec<Option<FuncMerkleInfo>>,
}

struct CallGraphSummary {
    scc_id: Vec<usize>,
    scc_size: Vec<usize>,
    topo_sccs_bottom_up: Vec<Vec<usize>>,
    tail_target_fns: BTreeSet<usize>,
    has_self_tail: Vec<bool>,
    distinct_callers: Vec<BTreeSet<usize>>,
}

fn tarjan_scc_bottom_up(
    adj: &[Vec<usize>],
) -> Result<(Vec<usize>, Vec<usize>, Vec<Vec<usize>>)> {
    struct Frame {
        u: usize,
        edge_pos: usize,
    }

    let n = adj.len();
    let mut index: Vec<Option<usize>> = vec![None; n];
    let mut lowlink: Vec<usize> = vec![0; n];
    let mut on_stack: Vec<bool> = vec![false; n];
    let mut stack: Vec<usize> = Vec::new();
    let mut call_stack: Vec<Frame> = Vec::new();
    let mut next_index = 0usize;
    let mut sccs: Vec<Vec<usize>> = Vec::new();
    let mut scc_id: Vec<usize> = vec![0; n];

    for start in 0..n {
        if index.get(start).copied().flatten().is_some() {
            continue;
        }
        if let Some(slot) = index.get_mut(start) {
            *slot = Some(next_index);
        }
        if let Some(slot) = lowlink.get_mut(start) {
            *slot = next_index;
        }
        next_index = next_index.saturating_add(1);
        if let Some(slot) = on_stack.get_mut(start) {
            *slot = true;
        }
        stack.push(start);
        call_stack.push(Frame { u: start, edge_pos: 0 });

        while let Some(top) = call_stack.last_mut() {
            let u = top.u;
            let edge_idx = top.edge_pos;
            let next_neighbor =
                adj.get(u).and_then(|edges| edges.get(edge_idx)).copied();
            if let Some(v) = next_neighbor {
                top.edge_pos = edge_idx.saturating_add(1);
                if index.get(v).copied().flatten().is_none() {
                    if let Some(slot) = index.get_mut(v) {
                        *slot = Some(next_index);
                    }
                    if let Some(slot) = lowlink.get_mut(v) {
                        *slot = next_index;
                    }
                    next_index = next_index.saturating_add(1);
                    if let Some(slot) = on_stack.get_mut(v) {
                        *slot = true;
                    }
                    stack.push(v);
                    call_stack.push(Frame { u: v, edge_pos: 0 });
                } else if *on_stack
                    .get(v)
                    .context("Tarjan SCC neighbor out of bounds in on_stack")?
                {
                    if let Some(v_idx) = index.get(v).copied().flatten()
                        && let Some(u_low) = lowlink.get_mut(u)
                    {
                        *u_low = (*u_low).min(v_idx);
                    }
                }
                continue;
            }

            let finished = call_stack.pop();
            let Some(Frame { u, .. }) = finished else {
                break;
            };
            let u_low = *lowlink
                .get(u)
                .context("Tarjan SCC node out of bounds in lowlink")?;
            if let Some(parent) = call_stack.last()
                && let Some(p_low) = lowlink.get_mut(parent.u)
            {
                *p_low = (*p_low).min(u_low);
            }
            let u_idx = index
                .get(u)
                .copied()
                .flatten()
                .context("Tarjan SCC node missing DFS index")?;
            if u_low == u_idx {
                let id = sccs.len();
                let mut component = Vec::new();
                while let Some(w) = stack.pop() {
                    if let Some(slot) = on_stack.get_mut(w) {
                        *slot = false;
                    }
                    if let Some(slot) = scc_id.get_mut(w) {
                        *slot = id;
                    }
                    component.push(w);
                    if w == u {
                        break;
                    }
                }
                sccs.push(component);
            }
        }
    }

    let mut scc_size = vec![1usize; n];
    for component in &sccs {
        let len = component.len();
        for &node in component {
            if let Some(slot) = scc_size.get_mut(node) {
                *slot = len;
            }
        }
    }
    Ok((scc_id, scc_size, sccs))
}

fn compute_call_graph_and_sccs(
    bodies: &[CodeBodyRaw<'_>],
    fn_sigs: &[FnSigInfo],
    block_types: &[FnSigInfo],
) -> Result<CallGraphSummary> {
    let n = bodies.len();
    let mut adj_sets: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); n];
    let mut distinct_callers: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); n];
    let mut has_self_tail: Vec<bool> = vec![false; n];
    let mut indirect_tail_targets: BTreeSet<usize> = BTreeSet::new();
    let mut cross_return_edges: Vec<(usize, usize)> = Vec::new();

    for (fn_idx, body) in bodies.iter().enumerate() {
        let bytes = body.code_bytes;
        let mut pos = 0usize;
        while pos < bytes.len() {
            let opcode = read_byte(bytes, &mut pos)?;
            match opcode {
                0x10 => {
                    let target =
                        usize::try_from(read_uleb_u32(bytes, &mut pos)?)?;
                    if let Some(set) = adj_sets.get_mut(fn_idx) {
                        set.insert(target);
                    }
                    if target != fn_idx
                        && let Some(callers) = distinct_callers.get_mut(target)
                    {
                        callers.insert(fn_idx);
                    }
                }
                0x11 => {
                    let type_idx =
                        usize::try_from(read_uleb_u32(bytes, &mut pos)?)?;
                    let _table_idx = read_uleb_u32(bytes, &mut pos)?;
                    let expected_sig = block_types
                        .get(type_idx)
                        .with_context(|| {
                            format!(
                                "call_indirect type {type_idx} out of bounds"
                            )
                        })?;
                    for (cand_idx, cand_sig) in fn_sigs.iter().enumerate() {
                        if cand_sig.params == expected_sig.params
                            && cand_sig.results == expected_sig.results
                            && let Some(set) = adj_sets.get_mut(fn_idx)
                        {
                            set.insert(cand_idx);
                        }
                    }
                }
                0x12 => {
                    let target =
                        usize::try_from(read_uleb_u32(bytes, &mut pos)?)?;
                    if let Some(set) = adj_sets.get_mut(fn_idx) {
                        set.insert(target);
                    }
                    if target == fn_idx {
                        if let Some(slot) = has_self_tail.get_mut(fn_idx) {
                            *slot = true;
                        }
                    } else {
                        cross_return_edges.push((fn_idx, target));
                        if let Some(callers) = distinct_callers.get_mut(target)
                        {
                            callers.insert(fn_idx);
                        }
                    }
                }
                0x13 => {
                    let type_idx =
                        usize::try_from(read_uleb_u32(bytes, &mut pos)?)?;
                    let _table_idx = read_uleb_u32(bytes, &mut pos)?;
                    let expected_sig = block_types
                        .get(type_idx)
                        .with_context(|| {
                            format!(
                                "return_call_indirect type {type_idx} out of bounds"
                            )
                        })?;
                    for (cand_idx, cand_sig) in fn_sigs.iter().enumerate() {
                        if cand_sig.params == expected_sig.params
                            && cand_sig.results == expected_sig.results
                        {
                            indirect_tail_targets.insert(cand_idx);
                            if let Some(set) = adj_sets.get_mut(fn_idx) {
                                set.insert(cand_idx);
                            }
                        }
                    }
                }
                _ => {
                    skip_instruction_immediates(opcode, bytes, &mut pos)?;
                }
            }
        }
    }

    let adj: Vec<Vec<usize>> = adj_sets
        .into_iter()
        .map(|s| s.into_iter().collect())
        .collect();
    let (scc_id, scc_size, topo_sccs_bottom_up) = tarjan_scc_bottom_up(&adj)?;

    let mut tail_target_fns = indirect_tail_targets;
    for (caller, target) in cross_return_edges {
        if scc_id.get(caller) == scc_id.get(target) {
            tail_target_fns.insert(target);
        }
    }

    Ok(CallGraphSummary {
        scc_id,
        scc_size,
        topo_sccs_bottom_up,
        tail_target_fns,
        has_self_tail,
        distinct_callers,
    })
}

fn select_init_fn_idx(
    fn_sigs: &[FnSigInfo],
    distinct_callers: &[BTreeSet<usize>],
    entry_idx: usize,
    start_func: Option<usize>,
    cap_yield_func_idx: Option<usize>,
    tail_target_fns: &BTreeSet<usize>,
) -> Result<Option<usize>> {
    ensure!(
        fn_sigs.len() == distinct_callers.len(),
        "Function signature count ({}) does not match distinct caller count ({})",
        fn_sigs.len(),
        distinct_callers.len()
    );
    Ok(fn_sigs
        .iter()
        .zip(distinct_callers.iter())
        .enumerate()
        .filter(|(idx, (sig, _))| {
            sig.params.is_empty()
                && sig.results.is_empty()
                && *idx != entry_idx
                && Some(*idx) != start_func
                && Some(*idx) != cap_yield_func_idx
                && !tail_target_fns.contains(idx)
        })
        .filter_map(|(idx, (_, callers))| {
            let count = callers.len();
            (count >= 2).then_some((count, Reverse(idx), idx))
        })
        .max_by_key(|&(count, rev_idx, _)| (count, rev_idx))
        .map(|(_, _, idx)| idx))
}

#[allow(
    clippy::too_many_arguments,
    reason = "Canonicalizes one WASM function body into a collision-free Merkle hash"
)]
fn canonicalize_and_hash_function(
    fn_idx: usize,
    sig: &FnSigInfo,
    body: &CodeBodyRaw<'_>,
    block_types: &[FnSigInfo],
    global_slots: &[GlobalSlot],
    cap_yield_func_idx: Option<usize>,
    init_fn_idx: Option<usize>,
    merkle_info: &[Option<FuncMerkleInfo>],
) -> Result<Option<FuncMerkleInfo>> {
    let mut hasher = Sha256::new();
    hasher.update(b"ctb-wasm-merkle-v1");
    hasher.update(u64::try_from(sig.params.len())?.to_le_bytes());
    for p in &sig.params {
        hasher.update([p.wire_tag()]);
    }
    hasher.update(u64::try_from(sig.results.len())?.to_le_bytes());
    for r in &sig.results {
        hasher.update([r.wire_tag()]);
    }
    hasher.update(u64::try_from(body.non_param_locals.len())?.to_le_bytes());
    for l in &body.non_param_locals {
        hasher.update([l.wire_tag()]);
    }

    let mut used_globals: Vec<(RustValTy, usize)> = Vec::new();
    let mut global_pos_map: BTreeMap<(RustValTy, usize), usize> = BTreeMap::new();
    let mut callee_hashes: Vec<[u8; 32]> = Vec::new();
    let bytes = body.code_bytes;
    let mut pos = 0usize;

    let register_global = |ty: RustValTy,
                           slot: usize,
                           used_globals: &mut Vec<(RustValTy, usize)>,
                           pos_map: &mut BTreeMap<(RustValTy, usize), usize>|
     -> Option<usize> {
        if let Some(&existing) = pos_map.get(&(ty, slot)) {
            return Some(existing);
        }
        if used_globals.len() >= MAX_SHARED_GLOBAL_PARAMS {
            return None;
        }
        let next_idx = used_globals.len();
        used_globals.push((ty, slot));
        pos_map.insert((ty, slot), next_idx);
        Some(next_idx)
    };

    while pos < bytes.len() {
        let opcode = read_byte(bytes, &mut pos)?;
        hasher.update([opcode]);
        match opcode {
            0x02 | 0x03 | 0x04 => {
                let bt = read_blocktype(bytes, &mut pos)?;
                let (b_params, b_results) = match bt {
                    WasmBlockType::Empty => (Vec::new(), Vec::new()),
                    WasmBlockType::ValType(b) => {
                        (Vec::new(), vec![RustValTy::from_wire_byte(b)?])
                    }
                    WasmBlockType::TypeIdx(idx) => {
                        let s = block_types
                            .get(usize::try_from(idx)?)
                            .with_context(|| {
                                format!("WASM blocktype index {idx} out of bounds")
                            })?;
                        (s.params.clone(), s.results.clone())
                    }
                };
                hasher.update([0xA0]);
                hasher.update(u64::try_from(b_params.len())?.to_le_bytes());
                for p in b_params {
                    hasher.update([p.wire_tag()]);
                }
                hasher.update(u64::try_from(b_results.len())?.to_le_bytes());
                for r in b_results {
                    hasher.update([r.wire_tag()]);
                }
            }
            0x0C | 0x0D | 0x20 | 0x21 | 0x22 | 0x25 | 0x26 | 0x3F | 0x40 => {
                let v = read_uleb_u32(bytes, &mut pos)?;
                hasher.update(v.to_le_bytes());
            }
            0x0E => {
                let count = read_uleb_u32(bytes, &mut pos)?;
                hasher.update(count.to_le_bytes());
                for _ in 0..count {
                    hasher.update(read_uleb_u32(bytes, &mut pos)?.to_le_bytes());
                }
                hasher.update(read_uleb_u32(bytes, &mut pos)?.to_le_bytes());
            }
            0x10 | 0x12 => {
                let target_idx =
                    usize::try_from(read_uleb_u32(bytes, &mut pos)?)?;
                if target_idx == fn_idx {
                    hasher.update([0x01]);
                } else if Some(target_idx) == cap_yield_func_idx {
                    hasher.update([0x02]);
                } else if Some(target_idx) == init_fn_idx {
                    hasher.update([0x03]);
                } else {
                    let Some(Some(callee_info)) = merkle_info.get(target_idx)
                    else {
                        return Ok(None);
                    };
                    hasher.update([0x04]);
                    hasher.update(callee_info.hash);
                    callee_hashes.push(callee_info.hash);
                    for &(c_ty, c_slot) in &callee_info.used_globals {
                        let Some(mapped_pos) = register_global(
                            c_ty,
                            c_slot,
                            &mut used_globals,
                            &mut global_pos_map,
                        ) else {
                            return Ok(None);
                        };
                        hasher.update(u32::try_from(mapped_pos)?.to_le_bytes());
                    }
                }
            }
            0x11 | 0x13 | 0xD2 => return Ok(None),
            0x1C => {
                let count = read_uleb_u32(bytes, &mut pos)?;
                hasher.update(count.to_le_bytes());
                for _ in 0..count {
                    hasher.update([read_byte(bytes, &mut pos)?]);
                }
            }
            0x23 | 0x24 => {
                let g_idx = usize::try_from(read_uleb_u32(bytes, &mut pos)?)?;
                let slot = global_slots.get(g_idx).with_context(|| {
                    format!("WASM global index {g_idx} out of bounds")
                })?;
                match slot {
                    GlobalSlot::HeapPtr => {
                        hasher.update([0x10]);
                    }
                    GlobalSlot::Typed(ty, per_type_slot) => {
                        let Some(mapped_pos) = register_global(
                            *ty,
                            *per_type_slot,
                            &mut used_globals,
                            &mut global_pos_map,
                        ) else {
                            return Ok(None);
                        };
                        hasher.update([0x11, ty.wire_tag()]);
                        hasher.update(u32::try_from(mapped_pos)?.to_le_bytes());
                    }
                }
            }
            0x28..=0x3E => {
                let (mem_idx, offset) = read_memarg(bytes, &mut pos)?;
                hasher.update(u32::try_from(mem_idx)?.to_le_bytes());
                hasher.update(offset.to_le_bytes());
            }
            0x41 => {
                hasher.update(read_sleb_i32(bytes, &mut pos)?.to_le_bytes());
            }
            0x42 | 0xD0 => {
                hasher.update(read_sleb_i64(bytes, &mut pos)?.to_le_bytes());
            }
            0x43 => {
                let end = pos
                    .checked_add(4)
                    .context("Unexpected EOF reading f32.const")?;
                let slice = bytes
                    .get(pos..end)
                    .context("Unexpected EOF reading f32.const")?;
                pos = end;
                hasher.update(slice);
            }
            0x44 => {
                let end = pos
                    .checked_add(8)
                    .context("Unexpected EOF reading f64.const")?;
                let slice = bytes
                    .get(pos..end)
                    .context("Unexpected EOF reading f64.const")?;
                pos = end;
                hasher.update(slice);
            }
            0xFC => {
                let subop = read_uleb_u32(bytes, &mut pos)?;
                hasher.update(subop.to_le_bytes());
                match subop {
                    0..=7 => {}
                    8 | 9 | 12 | 13 => return Ok(None),
                    10 | 14 => {
                        hasher.update(
                            read_uleb_u32(bytes, &mut pos)?.to_le_bytes(),
                        );
                        hasher.update(
                            read_uleb_u32(bytes, &mut pos)?.to_le_bytes(),
                        );
                    }
                    11 | 15 | 16 | 17 => {
                        hasher.update(
                            read_uleb_u32(bytes, &mut pos)?.to_le_bytes(),
                        );
                    }
                    other => {
                        bail!(
                            "Unsupported 0xFC sub-opcode {other} in native substitute transpiler"
                        );
                    }
                }
            }
            0xFD => {
                bail!(
                    "WebAssembly SIMD (0xFD) instructions are not supported in native substitutes"
                );
            }
            _ => {}
        }
    }

    hasher.update(u64::try_from(used_globals.len())?.to_le_bytes());
    for &(g_ty, _) in &used_globals {
        hasher.update([g_ty.wire_tag()]);
    }
    Ok(Some(FuncMerkleInfo {
        hash: hasher.finalize().into(),
        used_globals,
        callee_hashes,
    }))
}

#[allow(
    clippy::too_many_arguments,
    reason = "Computes bottom-up Merkle hashes for every eligible 1-node SCC in module"
)]
fn compute_module_merkle_info(
    code_bodies: &[CodeBodyRaw<'_>],
    fn_sigs: &[FnSigInfo],
    block_types: &[FnSigInfo],
    global_slots: &[GlobalSlot],
    cap_yield_func_idx: Option<usize>,
    init_fn_idx: Option<usize>,
    tail_target_fns: &BTreeSet<usize>,
    scc_size: &[usize],
    topo_sccs_bottom_up: &[Vec<usize>],
) -> Result<Vec<Option<FuncMerkleInfo>>> {
    let mut info: Vec<Option<FuncMerkleInfo>> = vec![None; code_bodies.len()];
    for scc in topo_sccs_bottom_up {
        if scc.len() != 1 {
            continue;
        }
        let Some(&fn_idx) = scc.first() else {
            continue;
        };
        if scc_size.get(fn_idx).copied() != Some(1)
            || Some(fn_idx) == cap_yield_func_idx
            || Some(fn_idx) == init_fn_idx
            || tail_target_fns.contains(&fn_idx)
        {
            continue;
        }
        let sig = fn_sigs
            .get(fn_idx)
            .with_context(|| format!("Function signature {fn_idx} missing"))?;
        let body = code_bodies
            .get(fn_idx)
            .with_context(|| format!("Code body {fn_idx} missing"))?;
        let computed = canonicalize_and_hash_function(
            fn_idx,
            sig,
            body,
            block_types,
            global_slots,
            cap_yield_func_idx,
            init_fn_idx,
            &info,
        )?;
        if let Some(slot) = info.get_mut(fn_idx) {
            *slot = computed;
        }
    }
    Ok(info)
}

fn build_global_slots(
    globals: &[Global],
    heap_ptr_global_idx: Option<usize>,
) -> Result<Vec<GlobalSlot>> {
    let mut counts = [0usize; 5];
    let mut slots = Vec::with_capacity(globals.len());
    for (idx, g) in globals.iter().enumerate() {
        if Some(idx) == heap_ptr_global_idx {
            slots.push(GlobalSlot::HeapPtr);
        } else {
            let ty = RustValTy::from_val_type(g.ty.ty)?;
            let arr_i = ty.array_idx();
            let slot_idx = *counts
                .get(arr_i)
                .context("Global type array index out of bounds")?;
            if let Some(c) = counts.get_mut(arr_i) {
                *c = slot_idx.saturating_add(1);
            }
            slots.push(GlobalSlot::Typed(ty, slot_idx));
        }
    }
    Ok(slots)
}

fn analyze_module<'a>(
    name: &'a str,
    wasm_bytes: &'a [u8],
) -> Result<AnalyzedModule<'a>> {
    let parsed = ctb_formats_dcdata::tinywasm::parser::Parser::new()
        .parse_module_bytes(wasm_bytes)
        .map_err(|error| {
            anyhow!("Failed to parse WASM binary in tinywasm: {error:?}")
        })?;
    let code_bodies = extract_raw_code_bodies(wasm_bytes)?;
    let (initial_memory_pages, initial_table_elements) =
        validate_transpilable_module(wasm_bytes, &parsed, &code_bodies)?;

    let block_types: Vec<FnSigInfo> = parsed
        .func_types
        .iter()
        .map(convert_func_type)
        .collect::<Result<_>>()?;
    let fn_sigs: Vec<FnSigInfo> = parsed
        .funcs
        .iter()
        .map(|f| convert_func_type(&f.ty))
        .collect::<Result<_>>()?;

    let (
        entry_name,
        entry_idx,
        entry_sig,
        memory_export_idx,
        cap_yield_func_idx,
        heap_ptr_global_idx,
    ) = resolve_entry_and_specials(&parsed, &fn_sigs)?;

    let global_slots = build_global_slots(&parsed.globals, heap_ptr_global_idx)?;
    let cg =
        compute_call_graph_and_sccs(&code_bodies, &fn_sigs, &block_types)?;
    let start_idx = parsed
        .start_func
        .map(usize::try_from)
        .transpose()?;
    let init_fn_idx = select_init_fn_idx(
        &fn_sigs,
        &cg.distinct_callers,
        entry_idx,
        start_idx,
        cap_yield_func_idx,
        &cg.tail_target_fns,
    )?;
    let merkle_info = compute_module_merkle_info(
        &code_bodies,
        &fn_sigs,
        &block_types,
        &global_slots,
        cap_yield_func_idx,
        init_fn_idx,
        &cg.tail_target_fns,
        &cg.scc_size,
        &cg.topo_sccs_bottom_up,
    )?;

    let wasm_sha256: [u8; 32] = Sha256::digest(wasm_bytes).into();
    let static_data_end = compute_static_data_end(&parsed);

    Ok(AnalyzedModule {
        name,
        wasm_sha256,
        parsed,
        code_bodies,
        initial_memory_pages,
        initial_table_elements,
        static_data_end,
        block_types,
        fn_sigs,
        entry_name,
        entry_idx,
        entry_sig,
        memory_export_idx,
        cap_yield_func_idx,
        heap_ptr_global_idx,
        global_slots,
        scc_id: cg.scc_id,
        tail_target_fns: cg.tail_target_fns,
        has_self_tail: cg.has_self_tail,
        init_fn_idx,
        merkle_info,
    })
}

fn skip_instruction_immediates(
    opcode: u8,
    bytes: &[u8],
    pos: &mut usize,
) -> Result<()> {
    match opcode {
        0x02 | 0x03 | 0x04 => {
            let _ = read_blocktype(bytes, pos)?;
        }
        0x0C | 0x0D | 0x10 | 0x12 | 0x20 | 0x21 | 0x22 | 0x23 | 0x24
        | 0x25 | 0x26 | 0x3F | 0x40 | 0xD2 => {
            let _ = read_uleb_u32(bytes, pos)?;
        }
        0x0E => {
            let count = read_uleb_u32(bytes, pos)?;
            for _ in 0..count {
                let _ = read_uleb_u32(bytes, pos)?;
            }
            let _ = read_uleb_u32(bytes, pos)?;
        }
        0x11 | 0x13 => {
            let _ = read_uleb_u32(bytes, pos)?;
            let _ = read_uleb_u32(bytes, pos)?;
        }
        0x1C => {
            let count = read_uleb_u32(bytes, pos)?;
            for _ in 0..count {
                let _ = read_byte(bytes, pos)?;
            }
        }
        0x28..=0x3E => {
            let _ = read_memarg(bytes, pos)?;
        }
        0x41 => {
            let _ = read_sleb_i32(bytes, pos)?;
        }
        0x42 | 0xD0 => {
            let _ = read_sleb_i64(bytes, pos)?;
        }
        0x43 => {
            *pos = pos
                .checked_add(4)
                .context("Unexpected EOF reading f32.const")?;
        }
        0x44 => {
            *pos = pos
                .checked_add(8)
                .context("Unexpected EOF reading f64.const")?;
        }
        0xFC => {
            let subop = read_uleb_u32(bytes, pos)?;
            match subop {
                0..=7 => {}
                8 | 10 | 12 | 14 => {
                    let _ = read_uleb_u32(bytes, pos)?;
                    let _ = read_uleb_u32(bytes, pos)?;
                }
                9 | 11 | 13 | 15 | 16 | 17 => {
                    let _ = read_uleb_u32(bytes, pos)?;
                }
                other => {
                    bail!(
                        "Unsupported 0xFC sub-opcode {other} in native substitute transpiler"
                    );
                }
            }
        }
        0xFD => {
            bail!(
                "WebAssembly SIMD (0xFD) instructions are not supported in native substitutes"
            );
        }
        _ => {}
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CtrlKind {
    Function,
    Block,
    Loop,
    If { has_else: bool },
}

#[derive(Debug, Clone)]
struct CtrlFrame {
    kind: CtrlKind,
    label_id: usize,
    indent: usize,
    params: Vec<RustValTy>,
    results: Vec<RustValTy>,
    stack_base: usize,
    parent_unreachable: bool,
    unreachable: bool,
}

struct SharedEmitContext<'a> {
    self_shard_idx: usize,
    self_symbol_name: &'a str,
    global_pos_map: BTreeMap<(RustValTy, usize), usize>,
}

fn format_call_invocation(
    target_idx: usize,
    arg_exprs: &[String],
    module: &AnalyzedModule<'_>,
    promoted_by_hash: &BTreeMap<[u8; 32], PromotedSharedTarget>,
    shared_ctx: Option<&SharedEmitContext<'_>>,
    caller_fn_idx: Option<usize>,
) -> Result<String> {
    if Some(target_idx) == module.cap_yield_func_idx {
        return Ok("host.request_capability(vm)".to_owned());
    }
    if Some(target_idx) == module.init_fn_idx
        && caller_fn_idx != Some(target_idx)
    {
        return Ok("vm.call_init(host)".to_owned());
    }

    if let Some(ctx) = shared_ctx {
        if caller_fn_idx == Some(target_idx) {
            let num_globals = ctx.global_pos_map.len();
            let mut call_args =
                Vec::with_capacity(arg_exprs.len().saturating_add(num_globals).saturating_add(2));
            call_args.push("vm".to_owned());
            call_args.extend(arg_exprs.iter().cloned());
            for g_pos in 0..num_globals {
                call_args.push(format!("g{g_pos}"));
            }
            call_args.push("host".to_owned());
            return Ok(format!(
                "Self::{}({})",
                ctx.self_symbol_name,
                call_args.join(", ")
            ));
        }

        let callee_merkle = module
            .merkle_info
            .get(target_idx)
            .and_then(Option::as_ref)
            .with_context(|| {
                format!("Shared function called non-Merkle callee fn_{target_idx}")
            })?;
        let target = promoted_by_hash
            .get(&callee_merkle.hash)
            .with_context(|| {
                format!(
                    "Shared callee fn_{target_idx} missing from promoted transitive closure"
                )
            })?;
        let mut call_args = Vec::with_capacity(
            arg_exprs
                .len()
                .saturating_add(callee_merkle.used_globals.len())
                .saturating_add(2),
        );
        call_args.push("vm".to_owned());
        call_args.extend(arg_exprs.iter().cloned());
        for &g_key in &callee_merkle.used_globals {
            let g_pos = ctx.global_pos_map.get(&g_key).copied().with_context(|| {
                format!("Shared caller missing transitive global slot {g_key:?}")
            })?;
            call_args.push(format!("g{g_pos}"));
        }
        call_args.push("host".to_owned());
        let prefix = if target.shard_idx == ctx.self_shard_idx {
            "Self".to_owned()
        } else {
            format!("crate::{}::Shared", target.shard_mod)
        };
        return Ok(format!(
            "{prefix}::{}({})",
            target.symbol_name,
            call_args.join(", ")
        ));
    }

    if let Some(Some(callee_merkle)) = module.merkle_info.get(target_idx)
        && let Some(target) = promoted_by_hash.get(&callee_merkle.hash)
    {
        let mut call_args = Vec::with_capacity(
            arg_exprs
                .len()
                .saturating_add(callee_merkle.used_globals.len())
                .saturating_add(2),
        );
        call_args.push("vm".to_owned());
        call_args.extend(arg_exprs.iter().cloned());
        for &(_ty, slot) in &callee_merkle.used_globals {
            call_args.push(format!("{slot}usize"));
        }
        call_args.push("host".to_owned());
        return Ok(format!(
            "crate::{}::Shared::{}({})",
            target.shard_mod,
            target.symbol_name,
            call_args.join(", ")
        ));
    }

    let mut call_args = Vec::with_capacity(arg_exprs.len().saturating_add(2));
    call_args.push("vm".to_owned());
    call_args.extend(arg_exprs.iter().cloned());
    call_args.push("host".to_owned());
    Ok(format!(
        "SubstituteInstance::fn_{target_idx}({})",
        call_args.join(", ")
    ))
}

struct FuncLowerer<'a> {
    fn_idx: usize,
    fn_results: Vec<RustValTy>,
    module: &'a AnalyzedModule<'a>,
    promoted_by_hash: &'a BTreeMap<[u8; 32], PromotedSharedTarget>,
    shared_ctx: Option<&'a SharedEmitContext<'a>>,
    locals: Vec<RustValTy>,
    num_params: usize,
    val_stack: Vec<RustValTy>,
    ctrl_stack: Vec<CtrlFrame>,
    used_slots: BTreeSet<(usize, RustValTy)>,
    planner: V2CheckpointPlanner,
    body_out: String,
    next_label: usize,
    next_temp: usize,
    is_cross_tail_step: bool,
    use_self_tail_loop: bool,
}

impl<'a> FuncLowerer<'a> {
    fn new(
        fn_idx: usize,
        module: &'a AnalyzedModule<'a>,
        promoted_by_hash: &'a BTreeMap<[u8; 32], PromotedSharedTarget>,
        shared_ctx: Option<&'a SharedEmitContext<'a>>,
        is_cross_tail_step: bool,
        use_self_tail_loop: bool,
    ) -> Result<Self> {
        let sig = module
            .fn_sigs
            .get(fn_idx)
            .context("Function signature out of bounds during lowering")?;
        let body = module
            .code_bodies
            .get(fn_idx)
            .context("Function body out of bounds during lowering")?;
        let num_params = sig.params.len();
        let mut locals = sig.params.clone();
        locals.extend_from_slice(&body.non_param_locals);

        let initial_indent = if use_self_tail_loop { 2 } else { 1 };
        let fn_results = sig.results.clone();
        let fn_frame = CtrlFrame {
            kind: CtrlKind::Function,
            label_id: 0,
            indent: initial_indent,
            params: Vec::new(),
            results: fn_results.clone(),
            stack_base: 0,
            parent_unreachable: false,
            unreachable: false,
        };

        Ok(Self {
            fn_idx,
            fn_results,
            module,
            promoted_by_hash,
            shared_ctx,
            locals,
            num_params,
            val_stack: Vec::new(),
            ctrl_stack: vec![fn_frame],
            used_slots: BTreeSet::new(),
            planner: V2CheckpointPlanner::new(),
            body_out: String::new(),
            next_label: 1,
            next_temp: 0,
            is_cross_tail_step,
            use_self_tail_loop,
        })
    }

    fn current_frame(&self) -> Result<&CtrlFrame> {
        self.ctrl_stack
            .last()
            .context("Control stack unexpectedly empty during WASM lowering")
    }

    fn current_frame_mut(&mut self) -> Result<&mut CtrlFrame> {
        self.ctrl_stack
            .last_mut()
            .context("Control stack unexpectedly empty during WASM lowering")
    }

    fn is_unreachable(&self) -> Result<bool> {
        Ok(self.current_frame()?.unreachable)
    }

    fn set_unreachable(&mut self) -> Result<()> {
        self.current_frame_mut()?.unreachable = true;
        Ok(())
    }

    fn current_indent(&self) -> usize {
        // Reason for fallback: ctrl_stack always holds at least Function frame.
        self.ctrl_stack.last().map_or(1, |f| f.indent)
    }

    fn emit_line(&mut self, line: &str) {
        let indent = self.current_indent();
        for _ in 0..indent {
            self.body_out.push_str("    ");
        }
        self.body_out.push_str(line);
        self.body_out.push('\n');
    }

    fn emit_line_at(&mut self, indent: usize, line: &str) {
        for _ in 0..indent {
            self.body_out.push_str("    ");
        }
        self.body_out.push_str(line);
        self.body_out.push('\n');
    }

    fn alloc_temp(&mut self) -> String {
        let id = self.next_temp;
        self.next_temp = self.next_temp.saturating_add(1);
        format!("__t{id}")
    }

    fn use_slot(&mut self, depth: usize, ty: RustValTy) -> String {
        self.used_slots.insert((depth, ty));
        slot_ident(depth, ty)
    }

    fn push_val(&mut self, ty: RustValTy, expr: &str) {
        let depth = self.val_stack.len();
        let slot = self.use_slot(depth, ty);
        self.val_stack.push(ty);
        self.emit_line(&format!("{slot} = {expr};"));
    }

    fn pop_val(&mut self, expected: Option<RustValTy>) -> Result<(String, RustValTy)> {
        let ty = self
            .val_stack
            .pop()
            .context("WebAssembly value stack underflow during lowering")?;
        if let Some(exp) = expected {
            ensure!(
                ty == exp,
                "WebAssembly stack type mismatch: expected {exp:?}, found {ty:?}"
            );
        }
        let depth = self.val_stack.len();
        let slot = self.use_slot(depth, ty);
        Ok((slot, ty))
    }

    fn peek_top_n(&mut self, types: &[RustValTy]) -> Result<Vec<String>> {
        let n = types.len();
        ensure!(
            self.val_stack.len() >= n,
            "WebAssembly stack underflow reading {n} branch operands"
        );
        let start = self.val_stack.len().saturating_sub(n);
        let mut slots = Vec::with_capacity(n);
        for (i, &ty) in types.iter().enumerate() {
            let depth = start.saturating_add(i);
            let actual = *self
                .val_stack
                .get(depth)
                .context("Stack slot index out of range")?;
            ensure!(
                actual == ty,
                "Branch operand type mismatch: expected {ty:?}, found {actual:?}"
            );
            slots.push(self.use_slot(depth, ty));
        }
        Ok(slots)
    }

    fn maybe_emit_ordinary_deduct(&mut self) -> Result<()> {
        let emit_check = self.planner.before_ordinary_op();
        if emit_check && !self.is_unreachable()? {
            self.emit_line("vm.deduct(host, 1)?;");
        }
        Ok(())
    }

    fn decode_blocktype_sig(
        &self,
        bt: WasmBlockType,
    ) -> Result<(Vec<RustValTy>, Vec<RustValTy>)> {
        match bt {
            WasmBlockType::Empty => Ok((Vec::new(), Vec::new())),
            WasmBlockType::ValType(b) => {
                Ok((Vec::new(), vec![RustValTy::from_wire_byte(b)?]))
            }
            WasmBlockType::TypeIdx(idx) => {
                let sig = self
                    .module
                    .block_types
                    .get(usize::try_from(idx)?)
                    .with_context(|| {
                        format!("WASM blocktype index {idx} out of bounds")
                    })?;
                Ok((sig.params.clone(), sig.results.clone()))
            }
        }
    }

    fn return_expr_from_slots(&self, slots: &[String]) -> String {
        let inner = match slots {
            [] => "()".to_owned(),
            [single] => single.clone(),
            many => format!("({})", many.join(", ")),
        };
        if self.is_cross_tail_step {
            let ret_slug = ret_sig_slug(&self.fn_results);
            format!("TailStep_{ret_slug}::Done({inner})")
        } else {
            inner
        }
    }

    fn return_expr_from_top(&mut self) -> Result<String> {
        let ret_types = self.fn_results.clone();
        let slots = self.peek_top_n(&ret_types)?;
        Ok(self.return_expr_from_slots(&slots))
    }

    fn emit_branch_to_depth(&mut self, depth: u32, indent: usize) -> Result<()> {
        let depth_usize = usize::try_from(depth)?;
        let frame_idx = self
            .ctrl_stack
            .len()
            .checked_sub(depth_usize.saturating_add(1))
            .with_context(|| {
                format!("WebAssembly branch depth {depth_usize} out of range")
            })?;
        let target = self
            .ctrl_stack
            .get(frame_idx)
            .context("Branch target frame missing")?
            .clone();
        let arity_types = if target.kind == CtrlKind::Loop {
            target.params
        } else {
            target.results
        };
        let src_slots = self.peek_top_n(&arity_types)?;

        if target.kind == CtrlKind::Function {
            let temps: Vec<String> = src_slots
                .iter()
                .map(|src| {
                    let t = self.alloc_temp();
                    self.emit_line_at(indent, &format!("let {t} = {src};"));
                    t
                })
                .collect();
            let ret_expr = self.return_expr_from_slots(&temps);
            self.emit_line_at(indent, &format!("return Ok({ret_expr});"));
            return Ok(());
        }

        let mut temps = Vec::with_capacity(arity_types.len());
        for src in &src_slots {
            let t = self.alloc_temp();
            self.emit_line_at(indent, &format!("let {t} = {src};"));
            temps.push(t);
        }
        for (i, (t, &ty)) in temps.iter().zip(arity_types.iter()).enumerate() {
            let dst = self.use_slot(target.stack_base.saturating_add(i), ty);
            self.emit_line_at(indent, &format!("{dst} = {t};"));
        }
        if target.kind == CtrlKind::Loop {
            self.emit_line_at(indent, &format!("continue 'blk_{};", target.label_id));
        } else {
            self.emit_line_at(indent, &format!("break 'blk_{};", target.label_id));
        }
        Ok(())
    }

    #[allow(
        clippy::too_many_lines,
        reason = "Dispatches core WebAssembly opcodes identically to V2CheckpointPlanner"
    )]
    fn lower_body(&mut self, bytes: &[u8]) -> Result<String> {
        let is_cap_yield = self.module.cap_yield_func_idx == Some(self.fn_idx);
        if is_cap_yield {
            self.emit_line("host.request_capability(vm)?;");
            let ret_expr = self.return_expr_from_slots(&[]);
            self.emit_line(&format!("return Ok({ret_expr});"));
            return Ok(self.finalize_function_source());
        }

        if !self.use_self_tail_loop {
            self.emit_line("vm.deduct(host, 1)?;");
        } else {
            self.emit_line_at(1, "'self_tail: loop {");
            self.emit_line("vm.deduct(host, 1)?;");
            for idx in self.num_params..self.locals.len() {
                let ty = *self
                    .locals
                    .get(idx)
                    .context("Local variable index out of range")?;
                self.emit_line(&format!(
                    "let mut l{idx}: {} = {};",
                    ty.rust_ty(),
                    ty.zero_expr()
                ));
            }
        }

        let mut pos = 0usize;
        while pos < bytes.len() {
            let opcode = read_byte(bytes, &mut pos)?;

            // 1. Structural `else` (0x05)
            if opcode == 0x05 {
                self.planner.on_else();
                let (frame_indent, parent_unreach, stack_base, params) = {
                    let top = self.current_frame()?;
                    ensure!(
                        matches!(top.kind, CtrlKind::If { has_else: false }),
                        "Unexpected `else` (0x05) without matching `if`"
                    );
                    (
                        top.indent,
                        top.parent_unreachable,
                        top.stack_base,
                        top.params.clone(),
                    )
                };
                if !parent_unreach {
                    let outer_indent = frame_indent.saturating_sub(1);
                    self.emit_line_at(outer_indent, "} else {");
                    self.val_stack.truncate(stack_base);
                    self.val_stack.extend(params);
                }
                let top_mut = self.current_frame_mut()?;
                top_mut.kind = CtrlKind::If { has_else: true };
                top_mut.unreachable = parent_unreach;
                continue;
            }

            // 2. Structural `end` (0x0B)
            if opcode == 0x0B {
                self.planner.on_end();
                let popped = self
                    .ctrl_stack
                    .pop()
                    .context("Unexpected `end` (0x0B) with empty control stack")?;
                if popped.kind == CtrlKind::Function {
                    if !popped.unreachable {
                        let ret_expr = self.return_expr_from_top()?;
                        self.emit_line_at(
                            popped.indent,
                            &format!("return Ok({ret_expr});"),
                        );
                    }
                    if self.use_self_tail_loop {
                        if popped.unreachable {
                            self.emit_line_at(
                                popped.indent,
                                "return Err(WasmTrap::Unreachable);",
                            );
                        }
                        self.emit_line_at(1, "}");
                    }
                    break;
                }

                if !popped.parent_unreachable {
                    let outer_indent = popped.indent.saturating_sub(1);
                    match popped.kind {
                        CtrlKind::Block => {
                            self.emit_line_at(outer_indent, "}");
                        }
                        CtrlKind::Loop => {
                            self.emit_line_at(
                                popped.indent,
                                &format!("break 'blk_{};", popped.label_id),
                            );
                            self.emit_line_at(outer_indent, "}");
                        }
                        CtrlKind::If { .. } => {
                            let if_indent = outer_indent.saturating_sub(1);
                            self.emit_line_at(outer_indent, "}");
                            self.emit_line_at(if_indent, "}");
                        }
                        CtrlKind::Function => {}
                    }
                    self.val_stack.truncate(popped.stack_base);
                    for &ty in &popped.results {
                        let d = self.val_stack.len();
                        let _ = self.use_slot(d, ty);
                        self.val_stack.push(ty);
                    }
                }
                continue;
            }

            // 3. `loop` (0x03)
            if opcode == 0x03 {
                let bt = read_blocktype(bytes, &mut pos)?;
                let (params, results) = self.decode_blocktype_sig(bt)?;
                self.planner.on_loop_header();
                let parent_unreach = self.is_unreachable()?;
                let label_id = self.next_label;
                self.next_label = self.next_label.saturating_add(1);
                let base_indent = self.current_indent();
                let stack_base = if parent_unreach {
                    self.val_stack.len()
                } else {
                    ensure!(
                        self.val_stack.len() >= params.len(),
                        "Stack underflow entering `loop` block parameters"
                    );
                    self.val_stack.len().saturating_sub(params.len())
                };
                if !parent_unreach {
                    self.emit_line_at(
                        base_indent,
                        &format!("'blk_{label_id}: loop {{"),
                    );
                    self.emit_line_at(
                        base_indent.saturating_add(1),
                        "vm.deduct(host, 1)?;",
                    );
                }
                self.ctrl_stack.push(CtrlFrame {
                    kind: CtrlKind::Loop,
                    label_id,
                    indent: if parent_unreach {
                        base_indent
                    } else {
                        base_indent.saturating_add(1)
                    },
                    params,
                    results,
                    stack_base,
                    parent_unreachable: parent_unreach,
                    unreachable: parent_unreach,
                });
                continue;
            }

            // 4. `block` (0x02) and `if` (0x04)
            if opcode == 0x02 || opcode == 0x04 {
                let bt = read_blocktype(bytes, &mut pos)?;
                let (params, results) = self.decode_blocktype_sig(bt)?;
                let emit_check = self.planner.before_block_or_if();
                let parent_unreach = self.is_unreachable()?;
                if emit_check && !parent_unreach {
                    self.emit_line("vm.deduct(host, 1)?;");
                }
                let label_id = self.next_label;
                self.next_label = self.next_label.saturating_add(1);
                let base_indent = self.current_indent();

                if opcode == 0x02 {
                    let stack_base = if parent_unreach {
                        self.val_stack.len()
                    } else {
                        ensure!(
                            self.val_stack.len() >= params.len(),
                            "Stack underflow entering `block` parameters"
                        );
                        self.val_stack.len().saturating_sub(params.len())
                    };
                    if !parent_unreach {
                        self.emit_line_at(
                            base_indent,
                            &format!("'blk_{label_id}: {{"),
                        );
                    }
                    self.ctrl_stack.push(CtrlFrame {
                        kind: CtrlKind::Block,
                        label_id,
                        indent: if parent_unreach {
                            base_indent
                        } else {
                            base_indent.saturating_add(1)
                        },
                        params,
                        results,
                        stack_base,
                        parent_unreachable: parent_unreach,
                        unreachable: parent_unreach,
                    });
                } else {
                    let cond_slot = if !parent_unreach {
                        Some(self.pop_val(Some(RustValTy::I32))?.0)
                    } else {
                        None
                    };
                    let stack_base = if parent_unreach {
                        self.val_stack.len()
                    } else {
                        ensure!(
                            self.val_stack.len() >= params.len(),
                            "Stack underflow entering `if` parameters"
                        );
                        self.val_stack.len().saturating_sub(params.len())
                    };
                    if let Some(cond) = cond_slot {
                        self.emit_line_at(
                            base_indent,
                            &format!("'blk_{label_id}: {{"),
                        );
                        self.emit_line_at(
                            base_indent.saturating_add(1),
                            &format!("if {cond} != 0 {{"),
                        );
                    }
                    self.ctrl_stack.push(CtrlFrame {
                        kind: CtrlKind::If { has_else: false },
                        label_id,
                        indent: if parent_unreach {
                            base_indent
                        } else {
                            base_indent.saturating_add(2)
                        },
                        params,
                        results,
                        stack_base,
                        parent_unreachable: parent_unreach,
                        unreachable: parent_unreach,
                    });
                }
                continue;
            }

            // 5. `memory.grow` (0x40) — dynamic bulk/grow checkpoint
            if opcode == 0x40 {
                self.planner.before_bulk_or_grow_op();
                let mem_idx = usize::try_from(read_uleb_u32(bytes, &mut pos)?)?;
                if !self.is_unreachable()? {
                    let (delta_slot, _) = self.pop_val(Some(RustValTy::I32))?;
                    let res = format!(
                        "vm.memory_grow({mem_idx}, {delta_slot}, host)?"
                    );
                    self.push_val(RustValTy::I32, &res);
                }
                continue;
            }

            // 6. `0xFC` extended operations
            if opcode == 0xFC {
                let subop = read_uleb_u32(bytes, &mut pos)?;
                self.lower_fc_subop(subop, bytes, &mut pos)?;
                continue;
            }

            // All remaining instructions are ordinary operations for V2 metering.
            self.maybe_emit_ordinary_deduct()?;
            if self.is_unreachable()? {
                skip_instruction_immediates(opcode, bytes, &mut pos)?;
                continue;
            }
            self.lower_ordinary_op(opcode, bytes, &mut pos)?;
        }

        Ok(self.finalize_function_source())
    }

    fn lower_fc_subop(
        &mut self,
        subop: u32,
        bytes: &[u8],
        pos: &mut usize,
    ) -> Result<()> {
        match subop {
            8 => {
                // `memory.init $seg_idx $mem_idx`
                self.planner.before_bulk_or_grow_op();
                let seg_idx = usize::try_from(read_uleb_u32(bytes, pos)?)?;
                let mem_idx = usize::try_from(read_uleb_u32(bytes, pos)?)?;
                if !self.is_unreachable()? {
                    let (len, _) = self.pop_val(Some(RustValTy::I32))?;
                    let (src, _) = self.pop_val(Some(RustValTy::I32))?;
                    let (dst, _) = self.pop_val(Some(RustValTy::I32))?;
                    self.emit_line(&format!(
                        "vm.memory_init({mem_idx}, {seg_idx}, {dst}, {src}, {len}, host)?;"
                    ));
                }
            }
            10 => {
                // `memory.copy $dst_mem $src_mem`
                self.planner.before_bulk_or_grow_op();
                let dst_mem = usize::try_from(read_uleb_u32(bytes, pos)?)?;
                let src_mem = usize::try_from(read_uleb_u32(bytes, pos)?)?;
                if !self.is_unreachable()? {
                    let (len, _) = self.pop_val(Some(RustValTy::I32))?;
                    let (src, _) = self.pop_val(Some(RustValTy::I32))?;
                    let (dst, _) = self.pop_val(Some(RustValTy::I32))?;
                    self.emit_line(&format!(
                        "vm.memory_copy({dst_mem}, {src_mem}, {dst}, {src}, {len}, host)?;"
                    ));
                }
            }
            11 => {
                // `memory.fill $mem_idx`
                self.planner.before_bulk_or_grow_op();
                let mem_idx = usize::try_from(read_uleb_u32(bytes, pos)?)?;
                if !self.is_unreachable()? {
                    let (len, _) = self.pop_val(Some(RustValTy::I32))?;
                    let (val, _) = self.pop_val(Some(RustValTy::I32))?;
                    let (dst, _) = self.pop_val(Some(RustValTy::I32))?;
                    self.emit_line(&format!(
                        "vm.memory_fill({mem_idx}, {dst}, {val}, {len}, host)?;"
                    ));
                }
            }
            12 => {
                // `table.init $seg_idx $table_idx`
                self.planner.before_bulk_or_grow_op();
                let seg_idx = usize::try_from(read_uleb_u32(bytes, pos)?)?;
                let tbl_idx = usize::try_from(read_uleb_u32(bytes, pos)?)?;
                if !self.is_unreachable()? {
                    let (len, _) = self.pop_val(Some(RustValTy::I32))?;
                    let (src, _) = self.pop_val(Some(RustValTy::I32))?;
                    let (dst, _) = self.pop_val(Some(RustValTy::I32))?;
                    self.emit_line(&format!(
                        "vm.table_init({tbl_idx}, {seg_idx}, {dst}, {src}, {len}, host)?;"
                    ));
                }
            }
            14 => {
                // `table.copy $dst_tbl $src_tbl`
                self.planner.before_bulk_or_grow_op();
                let dst_tbl = usize::try_from(read_uleb_u32(bytes, pos)?)?;
                let src_tbl = usize::try_from(read_uleb_u32(bytes, pos)?)?;
                if !self.is_unreachable()? {
                    let (len, _) = self.pop_val(Some(RustValTy::I32))?;
                    let (src, _) = self.pop_val(Some(RustValTy::I32))?;
                    let (dst, _) = self.pop_val(Some(RustValTy::I32))?;
                    self.emit_line(&format!(
                        "vm.table_copy({dst_tbl}, {src_tbl}, {dst}, {src}, {len}, host)?;"
                    ));
                }
            }
            15 => {
                // `table.grow $table_idx`
                self.planner.before_bulk_or_grow_op();
                let tbl_idx = usize::try_from(read_uleb_u32(bytes, pos)?)?;
                if !self.is_unreachable()? {
                    let (delta, _) = self.pop_val(Some(RustValTy::I32))?;
                    let (init_val, _) = self.pop_val(Some(RustValTy::Ref))?;
                    let res = format!(
                        "vm.table_grow({tbl_idx}, {init_val}, {delta}, host)?"
                    );
                    self.push_val(RustValTy::I32, &res);
                }
            }
            17 => {
                // `table.fill $table_idx`
                self.planner.before_bulk_or_grow_op();
                let tbl_idx = usize::try_from(read_uleb_u32(bytes, pos)?)?;
                if !self.is_unreachable()? {
                    let (len, _) = self.pop_val(Some(RustValTy::I32))?;
                    let (val, _) = self.pop_val(Some(RustValTy::Ref))?;
                    let (dst, _) = self.pop_val(Some(RustValTy::I32))?;
                    self.emit_line(&format!(
                        "vm.table_fill({tbl_idx}, {dst}, {val}, {len}, host)?;"
                    ));
                }
            }
            0..=7 => {
                self.maybe_emit_ordinary_deduct()?;
                if !self.is_unreachable()? {
                    let (helper, src_ty, dst_ty) = match subop {
                        0 => (
                            "i32_trunc_sat_f32_s",
                            RustValTy::F32,
                            RustValTy::I32,
                        ),
                        1 => (
                            "i32_trunc_sat_f32_u",
                            RustValTy::F32,
                            RustValTy::I32,
                        ),
                        2 => (
                            "i32_trunc_sat_f64_s",
                            RustValTy::F64,
                            RustValTy::I32,
                        ),
                        3 => (
                            "i32_trunc_sat_f64_u",
                            RustValTy::F64,
                            RustValTy::I32,
                        ),
                        4 => (
                            "i64_trunc_sat_f32_s",
                            RustValTy::F32,
                            RustValTy::I64,
                        ),
                        5 => (
                            "i64_trunc_sat_f32_u",
                            RustValTy::F32,
                            RustValTy::I64,
                        ),
                        6 => (
                            "i64_trunc_sat_f64_s",
                            RustValTy::F64,
                            RustValTy::I64,
                        ),
                        _ => (
                            "i64_trunc_sat_f64_u",
                            RustValTy::F64,
                            RustValTy::I64,
                        ),
                    };
                    let (a, _) = self.pop_val(Some(src_ty))?;
                    self.push_val(dst_ty, &format!("ops::{helper}({a})"));
                }
            }
            9 => {
                let seg_idx = usize::try_from(read_uleb_u32(bytes, pos)?)?;
                self.maybe_emit_ordinary_deduct()?;
                if !self.is_unreachable()? {
                    self.emit_line(&format!("vm.data_drop({seg_idx});"));
                }
            }
            13 => {
                let seg_idx = usize::try_from(read_uleb_u32(bytes, pos)?)?;
                self.maybe_emit_ordinary_deduct()?;
                if !self.is_unreachable()? {
                    self.emit_line(&format!("vm.elem_drop({seg_idx});"));
                }
            }
            16 => {
                let tbl_idx = usize::try_from(read_uleb_u32(bytes, pos)?)?;
                self.maybe_emit_ordinary_deduct()?;
                if !self.is_unreachable()? {
                    self.push_val(
                        RustValTy::I32,
                        &format!("vm.table_size({tbl_idx})?"),
                    );
                }
            }
            other => {
                bail!("Unsupported 0xFC sub-opcode {other} in native substitute transpiler");
            }
        }
        Ok(())
    }

    fn emit_call_and_push_results(
        &mut self,
        call_expr: &str,
        results: &[RustValTy],
    ) {
        self.emit_line("vm.enter_call()?;");
        self.emit_line(&format!("let __call_res = {call_expr};"));
        self.emit_line("vm.exit_call();");
        match results {
            [] => {
                self.emit_line("__call_res?;");
            }
            [single] => {
                let r = self.alloc_temp();
                self.emit_line(&format!("let {r} = __call_res?;"));
                self.push_val(*single, &r);
            }
            many => {
                let temps: Vec<String> =
                    (0..many.len()).map(|_| self.alloc_temp()).collect();
                self.emit_line(&format!(
                    "let ({}) = __call_res?;",
                    temps.join(", ")
                ));
                for (t, &ty) in temps.iter().zip(many.iter()) {
                    self.push_val(ty, t);
                }
            }
        }
    }

    fn lower_ordinary_op(
        &mut self,
        opcode: u8,
        bytes: &[u8],
        pos: &mut usize,
    ) -> Result<()> {
        match opcode {
            0x00 => {
                self.emit_line("return Err(WasmTrap::Unreachable);");
                self.set_unreachable()?;
            }
            0x01 => {}
            0x0C => {
                let depth = read_uleb_u32(bytes, pos)?;
                let indent = self.current_indent();
                self.emit_branch_to_depth(depth, indent)?;
                self.set_unreachable()?;
            }
            0x0D => {
                let depth = read_uleb_u32(bytes, pos)?;
                let (cond, _) = self.pop_val(Some(RustValTy::I32))?;
                let indent = self.current_indent();
                self.emit_line(&format!("if {cond} != 0 {{"));
                self.emit_branch_to_depth(depth, indent.saturating_add(1))?;
                self.emit_line("}");
            }
            0x0E => {
                let count = read_uleb_u32(bytes, pos)?;
                let mut targets = Vec::with_capacity(usize::try_from(count)?);
                for _ in 0..count {
                    targets.push(read_uleb_u32(bytes, pos)?);
                }
                let default_depth = read_uleb_u32(bytes, pos)?;
                let (idx, _) = self.pop_val(Some(RustValTy::I32))?;
                let base_indent = self.current_indent();
                self.emit_line(&format!("match {idx} {{"));
                for (arm_idx, &depth) in targets.iter().enumerate() {
                    let arm_i32 = i32::try_from(arm_idx).context(
                        "br_table target index exceeds i32::MAX",
                    )?;
                    self.emit_line_at(
                        base_indent.saturating_add(1),
                        &format!("{arm_i32}i32 => {{"),
                    );
                    self.emit_branch_to_depth(
                        depth,
                        base_indent.saturating_add(2),
                    )?;
                    self.emit_line_at(base_indent.saturating_add(1), "}");
                }
                self.emit_line_at(base_indent.saturating_add(1), "_ => {");
                self.emit_branch_to_depth(
                    default_depth,
                    base_indent.saturating_add(2),
                )?;
                self.emit_line_at(base_indent.saturating_add(1), "}");
                self.emit_line("}");
                self.set_unreachable()?;
            }
            0x0F => {
                let ret_expr = self.return_expr_from_top()?;
                self.emit_line(&format!("return Ok({ret_expr});"));
                self.set_unreachable()?;
            }
            0x10 => {
                let fn_idx = usize::try_from(read_uleb_u32(bytes, pos)?)?;
                let callee_sig = self
                    .module
                    .fn_sigs
                    .get(fn_idx)
                    .with_context(|| format!("call fn_{fn_idx} out of bounds"))?
                    .clone();
                let mut arg_slots = Vec::with_capacity(callee_sig.params.len());
                for &pt in callee_sig.params.iter().rev() {
                    arg_slots.push(self.pop_val(Some(pt))?.0);
                }
                arg_slots.reverse();
                let expr = format_call_invocation(
                    fn_idx,
                    &arg_slots,
                    self.module,
                    self.promoted_by_hash,
                    self.shared_ctx,
                    Some(self.fn_idx),
                )?;
                self.emit_call_and_push_results(&expr, &callee_sig.results);
            }
            0x11 => {
                let type_idx = usize::try_from(read_uleb_u32(bytes, pos)?)?;
                let table_idx = usize::try_from(read_uleb_u32(bytes, pos)?)?;
                let expected_sig = self
                    .module
                    .block_types
                    .get(type_idx)
                    .with_context(|| {
                        format!("call_indirect type {type_idx} out of bounds")
                    })?
                    .clone();
                let (elem_slot, _) = self.pop_val(Some(RustValTy::I32))?;
                let mut arg_slots =
                    Vec::with_capacity(expected_sig.params.len());
                for &pt in expected_sig.params.iter().rev() {
                    arg_slots.push(self.pop_val(Some(pt))?.0);
                }
                arg_slots.reverse();

                let temps: Vec<String> = arg_slots
                    .iter()
                    .map(|slot| {
                        let t = self.alloc_temp();
                        self.emit_line(&format!("let {t} = {slot};"));
                        t
                    })
                    .collect();
                let target_var = self.alloc_temp();
                self.emit_line(&format!(
                    "let {target_var} = vm.call_indirect_target({table_idx}, {elem_slot})?;"
                ));

                self.emit_line("vm.enter_call()?;");
                let res_var = self.alloc_temp();
                let indent = self.current_indent();
                self.emit_line(&format!("let {res_var} = match {target_var} {{"));
                for (cand_idx, cand_sig) in self.module.fn_sigs.iter().enumerate()
                {
                    if cand_sig.params == expected_sig.params
                        && cand_sig.results == expected_sig.results
                    {
                        let cand_call = format_call_invocation(
                            cand_idx,
                            &temps,
                            self.module,
                            self.promoted_by_hash,
                            self.shared_ctx,
                            None,
                        )?;
                        self.emit_line_at(
                            indent.saturating_add(1),
                            &format!("{cand_idx}u32 => {cand_call},"),
                        );
                    }
                }
                self.emit_line_at(
                    indent.saturating_add(1),
                    "_ => Err(WasmTrap::IndirectCallTypeMismatch),",
                );
                self.emit_line("};");
                self.emit_line("vm.exit_call();");
                match expected_sig.results.as_slice() {
                    [] => {
                        self.emit_line(&format!("{res_var}?;"));
                    }
                    [single] => {
                        let out_t = self.alloc_temp();
                        self.emit_line(&format!("let {out_t} = {res_var}?;"));
                        self.push_val(*single, &out_t);
                    }
                    many => {
                        let out_ts: Vec<String> =
                            (0..many.len()).map(|_| self.alloc_temp()).collect();
                        self.emit_line(&format!(
                            "let ({}) = {res_var}?;",
                            out_ts.join(", ")
                        ));
                        for (t, &ty) in out_ts.iter().zip(many.iter()) {
                            self.push_val(ty, t);
                        }
                    }
                }
            }
            0x12 => {
                // `return_call $fn_idx`
                let target_idx = usize::try_from(read_uleb_u32(bytes, pos)?)?;
                let callee_sig = self
                    .module
                    .fn_sigs
                    .get(target_idx)
                    .with_context(|| {
                        format!("return_call fn_{target_idx} out of bounds")
                    })?
                    .clone();
                let mut arg_slots = Vec::with_capacity(callee_sig.params.len());
                for &pt in callee_sig.params.iter().rev() {
                    arg_slots.push(self.pop_val(Some(pt))?.0);
                }
                arg_slots.reverse();
                let temps: Vec<String> = arg_slots
                    .iter()
                    .map(|s| {
                        let t = self.alloc_temp();
                        self.emit_line(&format!("let {t} = {s};"));
                        t
                    })
                    .collect();
                if target_idx == self.fn_idx {
                    for (i, t) in temps.iter().enumerate() {
                        self.emit_line(&format!("l{i} = {t};"));
                    }
                    self.emit_line("continue 'self_tail;");
                } else if Some(target_idx) == self.module.cap_yield_func_idx {
                    self.emit_line("host.request_capability(vm)?;");
                    let done_expr = self.return_expr_from_slots(&[]);
                    self.emit_line(&format!("return Ok({done_expr});"));
                } else if Some(target_idx) == self.module.init_fn_idx {
                    self.emit_line("vm.call_init(host)?;");
                    let done_expr = self.return_expr_from_slots(&[]);
                    self.emit_line(&format!("return Ok({done_expr});"));
                } else {
                    let caller_scc = self.module.scc_id.get(self.fn_idx);
                    let target_scc = self.module.scc_id.get(target_idx);
                    if caller_scc == target_scc {
                        let ret_slug = ret_sig_slug(&callee_sig.results);
                        let arg_str = temps.join(", ");
                        if self.is_cross_tail_step {
                            self.emit_line(&format!(
                                "return Ok(TailStep_{ret_slug}::Tail(TailTarget_{ret_slug}::Func{target_idx}({arg_str})));"
                            ));
                        } else {
                            self.emit_line(&format!(
                                "return SubstituteInstance::tail_dispatch_{ret_slug}(vm, TailStep_{ret_slug}::Tail(TailTarget_{ret_slug}::Func{target_idx}({arg_str})), host);"
                            ));
                        }
                    } else {
                        // Condensation DAG `return_call`: target cannot cycle back
                        // to caller, so call directly in tail position without
                        // pushing a `vm.enter_call()` frame or trampoline enum.
                        let call_expr = format_call_invocation(
                            target_idx,
                            &temps,
                            self.module,
                            self.promoted_by_hash,
                            self.shared_ctx,
                            Some(self.fn_idx),
                        )?;
                        if self.is_cross_tail_step {
                            let ret_slug = ret_sig_slug(&callee_sig.results);
                            self.emit_line(&format!(
                                "return {call_expr}.map(TailStep_{ret_slug}::Done);"
                            ));
                        } else {
                            self.emit_line(&format!("return {call_expr};"));
                        }
                    }
                }
                self.set_unreachable()?;
            }
            0x13 => {
                // `return_call_indirect $type_idx $table_idx`
                let type_idx = usize::try_from(read_uleb_u32(bytes, pos)?)?;
                let table_idx = usize::try_from(read_uleb_u32(bytes, pos)?)?;
                let expected_sig = self
                    .module
                    .block_types
                    .get(type_idx)
                    .with_context(|| {
                        format!(
                            "return_call_indirect type {type_idx} out of bounds"
                        )
                    })?
                    .clone();
                let (elem_slot, _) = self.pop_val(Some(RustValTy::I32))?;
                let mut arg_slots =
                    Vec::with_capacity(expected_sig.params.len());
                for &pt in expected_sig.params.iter().rev() {
                    arg_slots.push(self.pop_val(Some(pt))?.0);
                }
                arg_slots.reverse();
                let temps: Vec<String> = arg_slots
                    .iter()
                    .map(|s| {
                        let t = self.alloc_temp();
                        self.emit_line(&format!("let {t} = {s};"));
                        t
                    })
                    .collect();
                let target_var = self.alloc_temp();
                self.emit_line(&format!(
                    "let {target_var} = vm.call_indirect_target({table_idx}, {elem_slot})?;"
                ));
                let ret_slug = ret_sig_slug(&expected_sig.results);
                let arg_str = temps.join(", ");
                let indent = self.current_indent();
                if self.is_cross_tail_step {
                    self.emit_line(&format!("return match {target_var} {{"));
                    for (cand_idx, cand_sig) in
                        self.module.fn_sigs.iter().enumerate()
                    {
                        if cand_sig.params == expected_sig.params
                            && cand_sig.results == expected_sig.results
                        {
                            self.emit_line_at(
                                indent.saturating_add(1),
                                &format!(
                                    "{cand_idx}u32 => Ok(TailStep_{ret_slug}::Tail(TailTarget_{ret_slug}::Func{cand_idx}({arg_str}))),"
                                ),
                            );
                        }
                    }
                    self.emit_line_at(
                        indent.saturating_add(1),
                        "_ => Err(WasmTrap::IndirectCallTypeMismatch),",
                    );
                    self.emit_line("};");
                } else {
                    let step_var = self.alloc_temp();
                    self.emit_line(&format!("let {step_var} = match {target_var} {{"));
                    for (cand_idx, cand_sig) in
                        self.module.fn_sigs.iter().enumerate()
                    {
                        if cand_sig.params == expected_sig.params
                            && cand_sig.results == expected_sig.results
                        {
                            self.emit_line_at(
                                indent.saturating_add(1),
                                &format!(
                                    "{cand_idx}u32 => TailStep_{ret_slug}::Tail(TailTarget_{ret_slug}::Func{cand_idx}({arg_str})),"
                                ),
                            );
                        }
                    }
                    self.emit_line_at(
                        indent.saturating_add(1),
                        "_ => return Err(WasmTrap::IndirectCallTypeMismatch),",
                    );
                    self.emit_line("};");
                    self.emit_line(&format!(
                        "return SubstituteInstance::tail_dispatch_{ret_slug}(vm, {step_var}, host);"
                    ));
                }
                self.set_unreachable()?;
            }
            0x1A => {
                let _ = self.pop_val(None)?;
            }
            0x1B => {
                let (cond, _) = self.pop_val(Some(RustValTy::I32))?;
                let (v2, t2) = self.pop_val(None)?;
                let (v1, t1) = self.pop_val(Some(t2))?;
                self.push_val(t1, &format!("if {cond} != 0 {{ {v1} }} else {{ {v2} }}"));
            }
            0x1C => {
                let cnt = read_uleb_u32(bytes, pos)?;
                ensure!(cnt == 1, "Typed select must specify exactly 1 type");
                let ty = RustValTy::from_wire_byte(read_byte(bytes, pos)?)?;
                let (cond, _) = self.pop_val(Some(RustValTy::I32))?;
                let (v2, _) = self.pop_val(Some(ty))?;
                let (v1, _) = self.pop_val(Some(ty))?;
                self.push_val(ty, &format!("if {cond} != 0 {{ {v1} }} else {{ {v2} }}"));
            }
            0x20 => {
                let idx = usize::try_from(read_uleb_u32(bytes, pos)?)?;
                let ty = *self
                    .locals
                    .get(idx)
                    .with_context(|| format!("local.get {idx} out of bounds"))?;
                self.push_val(ty, &format!("l{idx}"));
            }
            0x21 => {
                let idx = usize::try_from(read_uleb_u32(bytes, pos)?)?;
                let ty = *self
                    .locals
                    .get(idx)
                    .with_context(|| format!("local.set {idx} out of bounds"))?;
                let (val, _) = self.pop_val(Some(ty))?;
                self.emit_line(&format!("l{idx} = {val};"));
            }
            0x22 => {
                let idx = usize::try_from(read_uleb_u32(bytes, pos)?)?;
                let ty = *self
                    .locals
                    .get(idx)
                    .with_context(|| format!("local.tee {idx} out of bounds"))?;
                let (val, _) = self.pop_val(Some(ty))?;
                self.emit_line(&format!("l{idx} = {val};"));
                self.push_val(ty, &format!("l{idx}"));
            }
            0x23 => {
                let idx = usize::try_from(read_uleb_u32(bytes, pos)?)?;
                let slot = self
                    .module
                    .global_slots
                    .get(idx)
                    .with_context(|| format!("global.get {idx} out of bounds"))?;
                match slot {
                    GlobalSlot::HeapPtr => {
                        self.push_val(RustValTy::I32, "vm.heap_ptr");
                    }
                    GlobalSlot::Typed(ty, per_type_slot) => {
                        let expr = if let Some(ctx) = self.shared_ctx {
                            let param_pos = ctx
                                .global_pos_map
                                .get(&(*ty, *per_type_slot))
                                .copied()
                                .context("Shared global.get slot missing")?;
                            format!("vm.get_{}(g{param_pos})?", ty.slug())
                        } else {
                            format!("vm.get_{}({per_type_slot}usize)?", ty.slug())
                        };
                        self.push_val(*ty, &expr);
                    }
                }
            }
            0x24 => {
                let idx = usize::try_from(read_uleb_u32(bytes, pos)?)?;
                let slot = self
                    .module
                    .global_slots
                    .get(idx)
                    .with_context(|| format!("global.set {idx} out of bounds"))?;
                match slot {
                    GlobalSlot::HeapPtr => {
                        let (val, _) = self.pop_val(Some(RustValTy::I32))?;
                        self.emit_line(&format!("vm.heap_ptr = {val};"));
                    }
                    GlobalSlot::Typed(ty, per_type_slot) => {
                        let (val, _) = self.pop_val(Some(*ty))?;
                        if let Some(ctx) = self.shared_ctx {
                            let param_pos = ctx
                                .global_pos_map
                                .get(&(*ty, *per_type_slot))
                                .copied()
                                .context("Shared global.set slot missing")?;
                            self.emit_line(&format!(
                                "vm.set_{}(g{param_pos}, {val})?;",
                                ty.slug()
                            ));
                        } else {
                            self.emit_line(&format!(
                                "vm.set_{}({per_type_slot}usize, {val})?;",
                                ty.slug()
                            ));
                        }
                    }
                }
            }
            0x25 => {
                let tbl_idx = usize::try_from(read_uleb_u32(bytes, pos)?)?;
                let (idx, _) = self.pop_val(Some(RustValTy::I32))?;
                self.push_val(
                    RustValTy::Ref,
                    &format!("vm.table_get({tbl_idx}, {idx})?"),
                );
            }
            0x26 => {
                let tbl_idx = usize::try_from(read_uleb_u32(bytes, pos)?)?;
                let (val, _) = self.pop_val(Some(RustValTy::Ref))?;
                let (idx, _) = self.pop_val(Some(RustValTy::I32))?;
                self.emit_line(&format!(
                    "vm.table_set({tbl_idx}, {idx}, {val})?;"
                ));
            }
            0x28..=0x35 => self.lower_mem_load(opcode, bytes, pos)?,
            0x36..=0x3E => self.lower_mem_store(opcode, bytes, pos)?,
            0x3F => {
                let mem_idx = usize::try_from(read_uleb_u32(bytes, pos)?)?;
                self.push_val(
                    RustValTy::I32,
                    &format!("vm.memory_size({mem_idx})?"),
                );
            }
            0x41 => {
                let v = read_sleb_i32(bytes, pos)?;
                self.push_val(RustValTy::I32, &rust_i32_literal(v));
            }
            0x42 => {
                let v = read_sleb_i64(bytes, pos)?;
                self.push_val(RustValTy::I64, &rust_i64_literal(v));
            }
            0x43 => {
                let end = pos
                    .checked_add(4)
                    .context("Unexpected EOF reading f32.const")?;
                let slice = bytes
                    .get(*pos..end)
                    .context("Unexpected EOF reading f32.const")?;
                *pos = end;
                let arr: [u8; 4] = slice.try_into()?;
                self.push_val(
                    RustValTy::F32,
                    &rust_f32_literal(f32::from_le_bytes(arr)),
                );
            }
            0x44 => {
                let end = pos
                    .checked_add(8)
                    .context("Unexpected EOF reading f64.const")?;
                let slice = bytes
                    .get(*pos..end)
                    .context("Unexpected EOF reading f64.const")?;
                *pos = end;
                let arr: [u8; 8] = slice.try_into()?;
                self.push_val(
                    RustValTy::F64,
                    &rust_f64_literal(f64::from_le_bytes(arr)),
                );
            }
            0xD0 => {
                let _heap_ty = read_sleb_i64(bytes, pos)?;
                self.push_val(RustValTy::Ref, "None");
            }
            0xD1 => {
                let (r, _) = self.pop_val(Some(RustValTy::Ref))?;
                self.push_val(RustValTy::I32, &format!("i32::from({r}.is_none())"));
            }
            0xD2 => {
                let fn_idx = read_uleb_u32(bytes, pos)?;
                self.push_val(RustValTy::Ref, &format!("Some({fn_idx}u32)"));
            }
            0x45..=0xC4 => self.lower_numeric_op(opcode)?,
            other => {
                bail!("Unsupported WASM opcode {other:#04x} in native substitute transpiler");
            }
        }
        Ok(())
    }

    fn lower_mem_load(
        &mut self,
        opcode: u8,
        bytes: &[u8],
        pos: &mut usize,
    ) -> Result<()> {
        let (mem_idx, offset) = read_memarg(bytes, pos)?;
        let (addr, _) = self.pop_val(Some(RustValTy::I32))?;
        let (helper, dst_ty) = match opcode {
            0x28 => ("i32_load", RustValTy::I32),
            0x29 => ("i64_load", RustValTy::I64),
            0x2A => ("f32_load", RustValTy::F32),
            0x2B => ("f64_load", RustValTy::F64),
            0x2C => ("i32_load8_s", RustValTy::I32),
            0x2D => ("i32_load8_u", RustValTy::I32),
            0x2E => ("i32_load16_s", RustValTy::I32),
            0x2F => ("i32_load16_u", RustValTy::I32),
            0x30 => ("i64_load8_s", RustValTy::I64),
            0x31 => ("i64_load8_u", RustValTy::I64),
            0x32 => ("i64_load16_s", RustValTy::I64),
            0x33 => ("i64_load16_u", RustValTy::I64),
            0x34 => ("i64_load32_s", RustValTy::I64),
            0x35 => ("i64_load32_u", RustValTy::I64),
            other => bail!("Unexpected load opcode {other:#04x}"),
        };
        self.push_val(
            dst_ty,
            &format!("ops::{helper}(vm, {mem_idx}, {addr}, {offset}u32)?"),
        );
        Ok(())
    }

    fn lower_mem_store(
        &mut self,
        opcode: u8,
        bytes: &[u8],
        pos: &mut usize,
    ) -> Result<()> {
        let (mem_idx, offset) = read_memarg(bytes, pos)?;
        let (helper, val_ty) = match opcode {
            0x36 => ("i32_store", RustValTy::I32),
            0x37 => ("i64_store", RustValTy::I64),
            0x38 => ("f32_store", RustValTy::F32),
            0x39 => ("f64_store", RustValTy::F64),
            0x3A => ("i32_store8", RustValTy::I32),
            0x3B => ("i32_store16", RustValTy::I32),
            0x3C => ("i64_store8", RustValTy::I64),
            0x3D => ("i64_store16", RustValTy::I64),
            0x3E => ("i64_store32", RustValTy::I64),
            other => bail!("Unexpected store opcode {other:#04x}"),
        };
        let (val, _) = self.pop_val(Some(val_ty))?;
        let (addr, _) = self.pop_val(Some(RustValTy::I32))?;
        self.emit_line(&format!(
            "ops::{helper}(vm, {mem_idx}, {addr}, {offset}u32, {val})?;"
        ));
        Ok(())
    }

    fn lower_numeric_op(&mut self, opcode: u8) -> Result<()> {
        match opcode {
            0x45 => self.unary_op(RustValTy::I32, RustValTy::I32, "i32_eqz", false),
            0x50 => self.unary_op(RustValTy::I64, RustValTy::I32, "i64_eqz", false),

            0x46 => self.bin_op(RustValTy::I32, RustValTy::I32, "i32_eq", false),
            0x47 => self.bin_op(RustValTy::I32, RustValTy::I32, "i32_ne", false),
            0x48 => self.bin_op(RustValTy::I32, RustValTy::I32, "i32_lt_s", false),
            0x49 => self.bin_op(RustValTy::I32, RustValTy::I32, "i32_lt_u", false),
            0x4A => self.bin_op(RustValTy::I32, RustValTy::I32, "i32_gt_s", false),
            0x4B => self.bin_op(RustValTy::I32, RustValTy::I32, "i32_gt_u", false),
            0x4C => self.bin_op(RustValTy::I32, RustValTy::I32, "i32_le_s", false),
            0x4D => self.bin_op(RustValTy::I32, RustValTy::I32, "i32_le_u", false),
            0x4E => self.bin_op(RustValTy::I32, RustValTy::I32, "i32_ge_s", false),
            0x4F => self.bin_op(RustValTy::I32, RustValTy::I32, "i32_ge_u", false),

            0x51 => self.bin_op(RustValTy::I64, RustValTy::I32, "i64_eq", false),
            0x52 => self.bin_op(RustValTy::I64, RustValTy::I32, "i64_ne", false),
            0x53 => self.bin_op(RustValTy::I64, RustValTy::I32, "i64_lt_s", false),
            0x54 => self.bin_op(RustValTy::I64, RustValTy::I32, "i64_lt_u", false),
            0x55 => self.bin_op(RustValTy::I64, RustValTy::I32, "i64_gt_s", false),
            0x56 => self.bin_op(RustValTy::I64, RustValTy::I32, "i64_gt_u", false),
            0x57 => self.bin_op(RustValTy::I64, RustValTy::I32, "i64_le_s", false),
            0x58 => self.bin_op(RustValTy::I64, RustValTy::I32, "i64_le_u", false),
            0x59 => self.bin_op(RustValTy::I64, RustValTy::I32, "i64_ge_s", false),
            0x5A => self.bin_op(RustValTy::I64, RustValTy::I32, "i64_ge_u", false),

            0x5B => self.bin_op(RustValTy::F32, RustValTy::I32, "f32_eq", false),
            0x5C => self.bin_op(RustValTy::F32, RustValTy::I32, "f32_ne", false),
            0x5D => self.bin_op(RustValTy::F32, RustValTy::I32, "f32_lt", false),
            0x5E => self.bin_op(RustValTy::F32, RustValTy::I32, "f32_gt", false),
            0x5F => self.bin_op(RustValTy::F32, RustValTy::I32, "f32_le", false),
            0x60 => self.bin_op(RustValTy::F32, RustValTy::I32, "f32_ge", false),

            0x61 => self.bin_op(RustValTy::F64, RustValTy::I32, "f64_eq", false),
            0x62 => self.bin_op(RustValTy::F64, RustValTy::I32, "f64_ne", false),
            0x63 => self.bin_op(RustValTy::F64, RustValTy::I32, "f64_lt", false),
            0x64 => self.bin_op(RustValTy::F64, RustValTy::I32, "f64_gt", false),
            0x65 => self.bin_op(RustValTy::F64, RustValTy::I32, "f64_le", false),
            0x66 => self.bin_op(RustValTy::F64, RustValTy::I32, "f64_ge", false),

            0x67 => self.unary_op(RustValTy::I32, RustValTy::I32, "i32_clz", false),
            0x68 => self.unary_op(RustValTy::I32, RustValTy::I32, "i32_ctz", false),
            0x69 => self.unary_op(RustValTy::I32, RustValTy::I32, "i32_popcnt", false),
            0x6A => self.bin_op(RustValTy::I32, RustValTy::I32, "i32_add", false),
            0x6B => self.bin_op(RustValTy::I32, RustValTy::I32, "i32_sub", false),
            0x6C => self.bin_op(RustValTy::I32, RustValTy::I32, "i32_mul", false),
            0x6D => self.bin_op(RustValTy::I32, RustValTy::I32, "i32_div_s", true),
            0x6E => self.bin_op(RustValTy::I32, RustValTy::I32, "i32_div_u", true),
            0x6F => self.bin_op(RustValTy::I32, RustValTy::I32, "i32_rem_s", true),
            0x70 => self.bin_op(RustValTy::I32, RustValTy::I32, "i32_rem_u", true),
            0x71 => self.bin_op(RustValTy::I32, RustValTy::I32, "i32_and", false),
            0x72 => self.bin_op(RustValTy::I32, RustValTy::I32, "i32_or", false),
            0x73 => self.bin_op(RustValTy::I32, RustValTy::I32, "i32_xor", false),
            0x74 => self.bin_op(RustValTy::I32, RustValTy::I32, "i32_shl", false),
            0x75 => self.bin_op(RustValTy::I32, RustValTy::I32, "i32_shr_s", false),
            0x76 => self.bin_op(RustValTy::I32, RustValTy::I32, "i32_shr_u", false),
            0x77 => self.bin_op(RustValTy::I32, RustValTy::I32, "i32_rotl", false),
            0x78 => self.bin_op(RustValTy::I32, RustValTy::I32, "i32_rotr", false),

            0x79 => self.unary_op(RustValTy::I64, RustValTy::I64, "i64_clz", false),
            0x7A => self.unary_op(RustValTy::I64, RustValTy::I64, "i64_ctz", false),
            0x7B => self.unary_op(RustValTy::I64, RustValTy::I64, "i64_popcnt", false),
            0x7C => self.bin_op(RustValTy::I64, RustValTy::I64, "i64_add", false),
            0x7D => self.bin_op(RustValTy::I64, RustValTy::I64, "i64_sub", false),
            0x7E => self.bin_op(RustValTy::I64, RustValTy::I64, "i64_mul", false),
            0x7F => self.bin_op(RustValTy::I64, RustValTy::I64, "i64_div_s", true),
            0x80 => self.bin_op(RustValTy::I64, RustValTy::I64, "i64_div_u", true),
            0x81 => self.bin_op(RustValTy::I64, RustValTy::I64, "i64_rem_s", true),
            0x82 => self.bin_op(RustValTy::I64, RustValTy::I64, "i64_rem_u", true),
            0x83 => self.bin_op(RustValTy::I64, RustValTy::I64, "i64_and", false),
            0x84 => self.bin_op(RustValTy::I64, RustValTy::I64, "i64_or", false),
            0x85 => self.bin_op(RustValTy::I64, RustValTy::I64, "i64_xor", false),
            0x86 => self.bin_op(RustValTy::I64, RustValTy::I64, "i64_shl", false),
            0x87 => self.bin_op(RustValTy::I64, RustValTy::I64, "i64_shr_s", false),
            0x88 => self.bin_op(RustValTy::I64, RustValTy::I64, "i64_shr_u", false),
            0x89 => self.bin_op(RustValTy::I64, RustValTy::I64, "i64_rotl", false),
            0x8A => self.bin_op(RustValTy::I64, RustValTy::I64, "i64_rotr", false),

            0x8B => self.unary_op(RustValTy::F32, RustValTy::F32, "f32_abs", false),
            0x8C => self.unary_op(RustValTy::F32, RustValTy::F32, "f32_neg", false),
            0x8D => self.unary_op(RustValTy::F32, RustValTy::F32, "f32_ceil", false),
            0x8E => self.unary_op(RustValTy::F32, RustValTy::F32, "f32_floor", false),
            0x8F => self.unary_op(RustValTy::F32, RustValTy::F32, "f32_trunc", false),
            0x90 => self.unary_op(RustValTy::F32, RustValTy::F32, "f32_nearest", false),
            0x91 => self.unary_op(RustValTy::F32, RustValTy::F32, "f32_sqrt", false),
            0x92 => self.bin_op(RustValTy::F32, RustValTy::F32, "f32_add", false),
            0x93 => self.bin_op(RustValTy::F32, RustValTy::F32, "f32_sub", false),
            0x94 => self.bin_op(RustValTy::F32, RustValTy::F32, "f32_mul", false),
            0x95 => self.bin_op(RustValTy::F32, RustValTy::F32, "f32_div", false),
            0x96 => self.bin_op(RustValTy::F32, RustValTy::F32, "f32_min", false),
            0x97 => self.bin_op(RustValTy::F32, RustValTy::F32, "f32_max", false),
            0x98 => self.bin_op(RustValTy::F32, RustValTy::F32, "f32_copysign", false),

            0x99 => self.unary_op(RustValTy::F64, RustValTy::F64, "f64_abs", false),
            0x9A => self.unary_op(RustValTy::F64, RustValTy::F64, "f64_neg", false),
            0x9B => self.unary_op(RustValTy::F64, RustValTy::F64, "f64_ceil", false),
            0x9C => self.unary_op(RustValTy::F64, RustValTy::F64, "f64_floor", false),
            0x9D => self.unary_op(RustValTy::F64, RustValTy::F64, "f64_trunc", false),
            0x9E => self.unary_op(RustValTy::F64, RustValTy::F64, "f64_nearest", false),
            0x9F => self.unary_op(RustValTy::F64, RustValTy::F64, "f64_sqrt", false),
            0xA0 => self.bin_op(RustValTy::F64, RustValTy::F64, "f64_add", false),
            0xA1 => self.bin_op(RustValTy::F64, RustValTy::F64, "f64_sub", false),
            0xA2 => self.bin_op(RustValTy::F64, RustValTy::F64, "f64_mul", false),
            0xA3 => self.bin_op(RustValTy::F64, RustValTy::F64, "f64_div", false),
            0xA4 => self.bin_op(RustValTy::F64, RustValTy::F64, "f64_min", false),
            0xA5 => self.bin_op(RustValTy::F64, RustValTy::F64, "f64_max", false),
            0xA6 => self.bin_op(RustValTy::F64, RustValTy::F64, "f64_copysign", false),

            0xA7 => self.unary_op(RustValTy::I64, RustValTy::I32, "i32_wrap_i64", false),
            0xA8 => self.unary_op(RustValTy::F32, RustValTy::I32, "i32_trunc_f32_s", true),
            0xA9 => self.unary_op(RustValTy::F32, RustValTy::I32, "i32_trunc_f32_u", true),
            0xAA => self.unary_op(RustValTy::F64, RustValTy::I32, "i32_trunc_f64_s", true),
            0xAB => self.unary_op(RustValTy::F64, RustValTy::I32, "i32_trunc_f64_u", true),
            0xAC => self.unary_op(RustValTy::I32, RustValTy::I64, "i64_extend_i32_s", false),
            0xAD => self.unary_op(RustValTy::I32, RustValTy::I64, "i64_extend_i32_u", false),
            0xAE => self.unary_op(RustValTy::F32, RustValTy::I64, "i64_trunc_f32_s", true),
            0xAF => self.unary_op(RustValTy::F32, RustValTy::I64, "i64_trunc_f32_u", true),
            0xB0 => self.unary_op(RustValTy::F64, RustValTy::I64, "i64_trunc_f64_s", true),
            0xB1 => self.unary_op(RustValTy::F64, RustValTy::I64, "i64_trunc_f64_u", true),
            0xB2 => self.unary_op(RustValTy::I32, RustValTy::F32, "f32_convert_i32_s", false),
            0xB3 => self.unary_op(RustValTy::I32, RustValTy::F32, "f32_convert_i32_u", false),
            0xB4 => self.unary_op(RustValTy::I64, RustValTy::F32, "f32_convert_i64_s", false),
            0xB5 => self.unary_op(RustValTy::I64, RustValTy::F32, "f32_convert_i64_u", false),
            0xB6 => self.unary_op(RustValTy::F64, RustValTy::F32, "f32_demote_f64", false),
            0xB7 => self.unary_op(RustValTy::I32, RustValTy::F64, "f64_convert_i32_s", false),
            0xB8 => self.unary_op(RustValTy::I32, RustValTy::F64, "f64_convert_i32_u", false),
            0xB9 => self.unary_op(RustValTy::I64, RustValTy::F64, "f64_convert_i64_s", false),
            0xBA => self.unary_op(RustValTy::I64, RustValTy::F64, "f64_convert_i64_u", false),
            0xBB => self.unary_op(RustValTy::F32, RustValTy::F64, "f64_promote_f32", false),
            0xBC => self.unary_op(RustValTy::F32, RustValTy::I32, "i32_reinterpret_f32", false),
            0xBD => self.unary_op(RustValTy::F64, RustValTy::I64, "i64_reinterpret_f64", false),
            0xBE => self.unary_op(RustValTy::I32, RustValTy::F32, "f32_reinterpret_i32", false),
            0xBF => self.unary_op(RustValTy::I64, RustValTy::F64, "f64_reinterpret_i64", false),

            0xC0 => self.unary_op(RustValTy::I32, RustValTy::I32, "i32_extend8_s", false),
            0xC1 => self.unary_op(RustValTy::I32, RustValTy::I32, "i32_extend16_s", false),
            0xC2 => self.unary_op(RustValTy::I64, RustValTy::I64, "i64_extend8_s", false),
            0xC3 => self.unary_op(RustValTy::I64, RustValTy::I64, "i64_extend16_s", false),
            0xC4 => self.unary_op(RustValTy::I64, RustValTy::I64, "i64_extend32_s", false),
            other => bail!("Unsupported numeric opcode {other:#04x}"),
        }
    }

    fn unary_op(
        &mut self,
        src: RustValTy,
        dst: RustValTy,
        helper: &str,
        fallible: bool,
    ) -> Result<()> {
        let (a, _) = self.pop_val(Some(src))?;
        let suffix = if fallible { "?" } else { "" };
        self.push_val(dst, &format!("ops::{helper}({a}){suffix}"));
        Ok(())
    }

    fn bin_op(
        &mut self,
        op_ty: RustValTy,
        dst: RustValTy,
        helper: &str,
        fallible: bool,
    ) -> Result<()> {
        let (b, _) = self.pop_val(Some(op_ty))?;
        let (a, _) = self.pop_val(Some(op_ty))?;
        let suffix = if fallible { "?" } else { "" };
        self.push_val(dst, &format!("ops::{helper}({a}, {b}){suffix}"));
        Ok(())
    }

    fn finalize_function_source(&self) -> String {
        let mut header = String::new();
        for idx in 0..self.num_params {
            if let Some(ty) = self.locals.get(idx) {
                let _ = writeln!(
                    header,
                    "    let mut l{idx}: {} = p{idx};",
                    ty.rust_ty()
                );
            }
        }
        if !self.use_self_tail_loop {
            for idx in self.num_params..self.locals.len() {
                if let Some(ty) = self.locals.get(idx) {
                    let _ = writeln!(
                        header,
                        "    let mut l{idx}: {} = {};",
                        ty.rust_ty(),
                        ty.zero_expr()
                    );
                }
            }
        }
        for &(depth, ty) in &self.used_slots {
            let ident = slot_ident(depth, ty);
            let _ = writeln!(
                header,
                "    let mut {ident}: {} = {};",
                ty.rust_ty(),
                ty.zero_expr()
            );
        }
        format!("{header}{}", self.body_out)
    }
}

fn emit_generated_allows(out: &mut String, indent: usize) {
    for _ in 0..indent {
        out.push_str("    ");
    }
    out.push_str(
        "#[allow(\n\
         dead_code,\n\
         unused_mut,\n\
         unused_variables,\n\
         unused_assignments,\n\
         unreachable_code,\n\
         clippy::too_many_lines,\n\
         clippy::cognitive_complexity,\n\
         clippy::needless_continue,\n\
         clippy::redundant_else,\n\
         clippy::collapsible_if,\n\
         clippy::collapsible_else_if,\n\
         clippy::unnecessary_wraps,\n\
         clippy::match_same_arms,\n\
         clippy::single_match_else,\n\
         clippy::too_many_arguments,\n\
         clippy::let_and_return,\n\
         clippy::manual_let_else,\n\
         clippy::shadow_unrelated,\n\
         clippy::type_complexity,\n\
         clippy::almost_swapped,\n\
         reason = \"Generated WASM-to-Rust substitute function\"\n\
         )]\n",
    );
}

fn validate_transpilable_module(
    wasm_bytes: &[u8],
    parsed: &TinyWasmModule,
    code_bodies: &[CodeBodyRaw<'_>],
) -> Result<(usize, usize)> {
    ensure!(
        parsed.imports.is_empty(),
        "WebAssembly modules with host imports are not supported in native substitutes"
    );
    ensure!(
        parsed.funcs.len() == code_bodies.len(),
        "WASM Section 10 body count ({}) does not match parsed function count ({})",
        code_bodies.len(),
        parsed.funcs.len()
    );
    ensure!(
        parsed
            .exports
            .iter()
            .all(|exp| {
                !exp.name.starts_with("__ctb_")
                    || exp.name.as_ref() == CTB_CAPABILITY_YIELD_EXPORT
            }),
        "WebAssembly module must not declare reserved '__ctb_*' exports"
    );
    let initial_memory_pages = parsed
        .memory_types
        .iter()
        .try_fold(0usize, |acc, m| -> Result<usize> {
            ensure!(
                m.arch == MemoryArch::I32,
                "memory64 is not supported in native substitutes"
            );
            let init = usize::try_from(m.page_count_initial)?;
            Ok(acc.saturating_add(init))
        })?;
    let initial_table_elements =
        parsed
            .table_types
            .iter()
            .try_fold(0usize, |acc, t| -> Result<usize> {
                let init = usize::try_from(t.size_initial)?;
                Ok(acc.saturating_add(init))
            })?;
    let _ = wasm_bytes;
    Ok((initial_memory_pages, initial_table_elements))
}

fn resolve_entry_and_specials(
    parsed: &TinyWasmModule,
    fn_sigs: &[FnSigInfo],
) -> Result<(
    &'static str,
    usize,
    EntrySignatureKind,
    Option<usize>,
    Option<usize>,
    Option<usize>,
)> {
    let step_exp = parsed
        .exports
        .iter()
        .find(|e| e.kind == ExternalKind::Func && e.name.as_ref() == "step");
    let main_exp = parsed
        .exports
        .iter()
        .find(|e| e.kind == ExternalKind::Func && e.name.as_ref() == "main");
    let (entry_name, entry_idx) = match (step_exp, main_exp) {
        (Some(exp), _) => ("step", usize::try_from(exp.index)?),
        (None, Some(exp)) => ("main", usize::try_from(exp.index)?),
        (None, None) => {
            bail!("WebAssembly module must export a 'step' or 'main' entry function")
        }
    };
    let entry_sig = classify_entry_sig(
        fn_sigs
            .get(entry_idx)
            .context("Exported entry function index out of bounds")?,
    )?;

    let memory_export_idx = parsed
        .exports
        .iter()
        .find(|e| e.kind == ExternalKind::Memory && e.name.as_ref() == "memory")
        .map(|e| usize::try_from(e.index))
        .transpose()?;

    let mut cap_yield_func_idx = None;
    if let Some(exp) = parsed
        .exports
        .iter()
        .find(|e| e.name.as_ref() == CTB_CAPABILITY_YIELD_EXPORT)
    {
        ensure!(
            exp.kind == ExternalKind::Func,
            "Reserved export '{CTB_CAPABILITY_YIELD_EXPORT}' must be a function"
        );
        let idx = usize::try_from(exp.index)?;
        let sig = fn_sigs
            .get(idx)
            .context("Reserved capability yield index out of bounds")?;
        ensure!(
            sig.params.is_empty() && sig.results.is_empty(),
            "Reserved export '{CTB_CAPABILITY_YIELD_EXPORT}' must have signature () -> ()"
        );
        cap_yield_func_idx = Some(idx);
    }

    let heap_ptr_global_idx = parsed
        .exports
        .iter()
        .find(|e| {
            e.kind == ExternalKind::Global && e.name.as_ref() == "__heap_ptr"
        })
        .and_then(|e| {
            let idx = usize::try_from(e.index).ok()?;
            let g = parsed.globals.get(idx)?;
            (g.ty.mutable && g.ty.ty == ValType::I32).then_some(idx)
        });

    Ok((
        entry_name,
        entry_idx,
        entry_sig,
        memory_export_idx,
        cap_yield_func_idx,
        heap_ptr_global_idx,
    ))
}

fn emit_tail_call_types_and_dispatch(
    out: &mut String,
    tail_target_fns: &BTreeSet<usize>,
    fn_sigs: &[FnSigInfo],
) {
    let mut grouped: BTreeMap<String, (&[RustValTy], Vec<(usize, &FnSigInfo)>)> =
        BTreeMap::new();
    for (idx, sig) in fn_sigs.iter().enumerate() {
        if tail_target_fns.contains(&idx) {
            let slug = ret_sig_slug(&sig.results);
            let entry = grouped
                .entry(slug)
                .or_insert_with(|| (&sig.results, Vec::new()));
            entry.1.push((idx, sig));
        }
    }

    for (slug, (ret_tys, members)) in &grouped {
        let ret_ty = ret_type_expr(ret_tys);
        emit_generated_allows(out, 0);
        let _ = writeln!(out, "enum TailTarget_{slug} {{");
        for &(fn_idx, sig) in members {
            let ptys: Vec<&str> = sig.params.iter().map(|t| t.rust_ty()).collect();
            let _ = writeln!(out, "    Func{fn_idx}({}),", ptys.join(", "));
        }
        out.push_str("}\n\n");

        emit_generated_allows(out, 0);
        let _ = writeln!(out, "enum TailStep_{slug} {{");
        let _ = writeln!(out, "    Done({ret_ty}),");
        let _ = writeln!(out, "    Tail(TailTarget_{slug}),");
        out.push_str("}\n\n");
    }

    if !grouped.is_empty() {
        emit_generated_allows(out, 0);
        out.push_str("impl SubstituteInstance {\n");
        for (slug, (ret_tys, members)) in &grouped {
            let ret_ty = ret_type_expr(ret_tys);
            out.push_str("    #[inline(never)]\n");
            let _ = writeln!(
                out,
                "    fn tail_dispatch_{slug}(vm: &mut SubstituteVm, mut step_val: TailStep_{slug}, host: &mut dyn WasmSubstituteHost) -> std::result::Result<{ret_ty}, WasmTrap> {{"
            );
            out.push_str("        loop {\n");
            out.push_str("            match step_val {\n");
            out.push_str("                TailStep_");
            out.push_str(slug);
            out.push_str("::Done(res) => return Ok(res),\n");
            out.push_str("                TailStep_");
            out.push_str(slug);
            out.push_str("::Tail(target) => match target {\n");
            for &(fn_idx, sig) in members {
                let pnames: Vec<String> =
                    (0..sig.params.len()).map(|i| format!("p{i}")).collect();
                let mut call_args = Vec::with_capacity(pnames.len().saturating_add(2));
                call_args.push("vm".to_owned());
                call_args.extend(pnames.iter().cloned());
                call_args.push("host".to_owned());
                let _ = writeln!(
                    out,
                    "                    TailTarget_{slug}::Func{fn_idx}({}) => step_val = SubstituteInstance::fn_{fn_idx}_step({})?,",
                    pnames.join(", "),
                    call_args.join(", ")
                );
            }
            out.push_str("                },\n");
            out.push_str("            }\n");
            out.push_str("        }\n");
            out.push_str("    }\n\n");
        }
        out.push_str("}\n\n");
    }
}

fn emit_data_and_elem_consts(out: &mut String, parsed: &TinyWasmModule) {
    for (i, seg) in parsed.data.iter().enumerate() {
        emit_bytes_const(out, &format!("DATA_SEG_{i}"), &seg.data);
    }
    if !parsed.data.is_empty() {
        out.push('\n');
    }

    for (i, seg) in parsed.elements.iter().enumerate() {
        let mut items = Vec::with_capacity(seg.items.len());
        for item in &seg.items {
            match item {
                ElementItem::Func(idx)
                | ElementItem::Expr(ConstInstruction::RefFunc(idx)) => {
                    items.push(format!("Some({idx}u32)"));
                }
                ElementItem::Expr(ConstInstruction::RefNull(_)) => {
                    items.push("None".to_owned());
                }
                other => {
                    let _ = other;
                    items.push("None".to_owned());
                }
            }
        }
        let _ = writeln!(
            out,
            "const ELEM_SEG_{i}: &[Option<u32>] = &[{}];",
            items.join(", ")
        );
    }
    if !parsed.elements.is_empty() {
        out.push('\n');
    }
}

fn emit_struct_and_instantiate(
    out: &mut String,
    module: &AnalyzedModule<'_>,
) -> Result<()> {
    emit_generated_allows(out, 0);
    out.push_str("struct SubstituteInstance {\n    vm: SubstituteVm,\n}\n\n");

    let hex_bytes: Vec<String> =
        module.wasm_sha256.iter().map(|b| format!("0x{b:02x}")).collect();
    let _ = writeln!(
        out,
        "const MODULE_META: SubstituteModuleMeta = SubstituteModuleMeta {{\n\
         \x20   name: {:?},\n\
         \x20   accounting_version: {WASM_RESOURCE_ACCOUNTING_VERSION}u32,\n\
         \x20   wasm_sha256: [{}],\n\
         \x20   entry_func_name: {:?},\n\
         \x20   has_start_func: {},\n\
         \x20   initial_memory_pages: {}usize,\n\
         \x20   initial_table_elements: {}usize,\n\
         \x20   static_data_end: {}usize,\n\
         \x20   num_memories: {}usize,\n\
         \x20   num_tables: {}usize,\n\
         }};\n\n",
        module.name,
        hex_bytes.join(", "),
        module.entry_name,
        module.parsed.start_func.is_some(),
        module.initial_memory_pages,
        module.initial_table_elements,
        module.static_data_end,
        module.parsed.memory_types.len(),
        module.parsed.table_types.len(),
    );

    emit_generated_allows(out, 0);
    out.push_str("fn instantiate_substitute() -> Box<dyn WasmSubstituteInstance> {\n");
    for (i, g) in module.parsed.globals.iter().enumerate() {
        let prefix_globals = module
            .parsed
            .globals
            .get(..i)
            .context("WASM global prefix index out of bounds")?;
        let expr = emit_const_instruction(&g.init, prefix_globals, "")?;
        let ty = RustValTy::from_val_type(g.ty.ty)?;
        let _ = writeln!(out, "    let g{i}: {} = {expr};", ty.rust_ty());
    }
    out.push_str("    let memories = vec![\n");
    for m in &module.parsed.memory_types {
        let init = u32::try_from(m.page_count_initial)?;
        let max_s = match m.page_count_max {
            Some(max_p) => format!("Some({}u32)", u32::try_from(max_p)?),
            None => "None".to_owned(),
        };
        let _ = writeln!(
            out,
            "        WasmLinearMemory::new({init}u32, {max_s}),"
        );
    }
    out.push_str("    ];\n");
    out.push_str("    let tables = vec![\n");
    for t in &module.parsed.table_types {
        let max_s = match t.size_max {
            Some(m) => format!("Some({m}u32)"),
            None => "None".to_owned(),
        };
        let _ = writeln!(
            out,
            "        WasmTable::new({}u32, {max_s}),",
            t.size_initial
        );
    }
    out.push_str("    ];\n");
    out.push_str("    let passive_data: Vec<Option<&'static [u8]>> = vec![\n");
    for (i, seg) in module.parsed.data.iter().enumerate() {
        if matches!(seg.kind, DataKind::Passive) {
            let _ = writeln!(out, "        Some(DATA_SEG_{i}),");
        } else {
            out.push_str("        None,\n");
        }
    }
    out.push_str("    ];\n");
    out.push_str(
        "    let passive_elems: Vec<Option<&'static [Option<u32>]>> = vec![\n",
    );
    for (i, seg) in module.parsed.elements.iter().enumerate() {
        if matches!(seg.kind, ElementKind::Passive) {
            let _ = writeln!(out, "        Some(ELEM_SEG_{i}),");
        } else {
            out.push_str("        None,\n");
        }
    }
    out.push_str("    ];\n");
    out.push_str(
        "    let mut vm = SubstituteVm::new(memories, tables, passive_data, passive_elems);\n",
    );

    if let Some(hp_idx) = module.heap_ptr_global_idx {
        let _ = writeln!(out, "    vm.heap_ptr = g{hp_idx};");
        out.push_str("    vm.has_heap_ptr = true;\n");
    }

    let mut typed_inits: [Vec<String>; 5] = Default::default();
    for (i, slot) in module.global_slots.iter().enumerate() {
        if let GlobalSlot::Typed(ty, _) = slot
            && let Some(bucket) = typed_inits.get_mut(ty.array_idx())
        {
            bucket.push(format!("g{i}"));
        }
    }
    let field_names = [
        "globals_i32",
        "globals_i64",
        "globals_f32",
        "globals_f64",
        "globals_ref",
    ];
    for (field, bucket) in field_names.into_iter().zip(typed_inits.iter()) {
        if !bucket.is_empty() {
            let _ = writeln!(out, "    vm.{field} = vec![{}];", bucket.join(", "));
        }
    }

    if let Some(init_idx) = module.init_fn_idx {
        let _ = writeln!(
            out,
            "    vm.init_fn = Some(InitFnPtr(SubstituteInstance::fn_{init_idx}));"
        );
    }

    for (i, seg) in module.parsed.data.iter().enumerate() {
        if let DataKind::Active { mem, offset } = &seg.kind {
            let off = eval_const_usize_expr(offset, &module.parsed.globals)?;
            let _ = writeln!(
                out,
                "    let _ = vm.init_active_data({mem}usize, {off}usize, DATA_SEG_{i});"
            );
        }
    }
    for (i, seg) in module.parsed.elements.iter().enumerate() {
        if let ElementKind::Active { table, offset } = &seg.kind {
            let off = eval_const_usize_expr(offset, &module.parsed.globals)?;
            let _ = writeln!(
                out,
                "    let _ = vm.init_active_elem({table}usize, {off}usize, ELEM_SEG_{i});"
            );
        }
    }

    out.push_str("    Box::new(SubstituteInstance { vm })\n}\n\n");
    Ok(())
}

fn emit_trait_impl(
    out: &mut String,
    module: &AnalyzedModule<'_>,
    promoted_by_hash: &BTreeMap<[u8; 32], PromotedSharedTarget>,
) -> Result<()> {
    emit_generated_allows(out, 0);
    out.push_str("impl WasmSubstituteInstance for SubstituteInstance {\n");
    out.push_str(
        "    fn meta(&self) -> SubstituteModuleMeta {\n        MODULE_META\n    }\n\n",
    );
    out.push_str(
        "    fn vm(&self) -> &SubstituteVm {\n        &self.vm\n    }\n\n",
    );
    out.push_str(
        "    fn vm_mut(&mut self) -> &mut SubstituteVm {\n        &mut self.vm\n    }\n\n",
    );
    match module.memory_export_idx {
        Some(idx) => {
            let _ = writeln!(
                out,
                "    fn exported_memory_index(&self) -> Option<usize> {{\n        Some({idx}usize)\n    }}\n"
            );
        }
        None => {
            out.push_str(
                "    fn exported_memory_index(&self) -> Option<usize> {\n        None\n    }\n\n",
            );
        }
    }

    out.push_str("    fn exported_global_i32(&self, name: &str) -> Option<i32> {\n        match name {\n");
    for exp in &module.parsed.exports {
        if exp.kind == ExternalKind::Global {
            let idx = usize::try_from(exp.index)?;
            if let Some(slot) = module.global_slots.get(idx) {
                match slot {
                    GlobalSlot::HeapPtr => {
                        let _ = writeln!(
                            out,
                            "            {:?} => Some(self.vm.heap_ptr),",
                            exp.name.as_ref()
                        );
                    }
                    GlobalSlot::Typed(RustValTy::I32, per_type_slot) => {
                        let _ = writeln!(
                            out,
                            "            {:?} => self.vm.globals_i32.get({per_type_slot}usize).copied(),",
                            exp.name.as_ref()
                        );
                    }
                    GlobalSlot::Typed(_, _) => {}
                }
            }
        }
    }
    out.push_str("            _ => None,\n        }\n    }\n\n");

    out.push_str("    fn exported_global_i64(&self, name: &str) -> Option<i64> {\n        match name {\n");
    for exp in &module.parsed.exports {
        if exp.kind == ExternalKind::Global {
            let idx = usize::try_from(exp.index)?;
            if let Some(GlobalSlot::Typed(RustValTy::I64, per_type_slot)) =
                module.global_slots.get(idx)
            {
                let _ = writeln!(
                    out,
                    "            {:?} => self.vm.globals_i64.get({per_type_slot}usize).copied(),",
                    exp.name.as_ref()
                );
            }
        }
    }
    out.push_str("            _ => None,\n        }\n    }\n\n");

    if module.heap_ptr_global_idx.is_some() {
        out.push_str(
            "    fn set_heap_ptr(&mut self, val: i32) -> bool {\n        self.vm.heap_ptr = val;\n        true\n    }\n\n",
        );
    } else {
        out.push_str(
            "    fn set_heap_ptr(&mut self, _val: i32) -> bool {\n        false\n    }\n\n",
        );
    }

    match module.parsed.start_func {
        Some(start_u32) => {
            let start_idx = usize::try_from(start_u32)?;
            let start_call = format_call_invocation(
                start_idx,
                &[],
                module,
                promoted_by_hash,
                None,
                None,
            )?;
            let _ = writeln!(
                out,
                "    fn call_start(&mut self, host: &mut dyn WasmSubstituteHost) -> std::result::Result<(), WasmTrap> {{\n\
                 \x20       let vm = &mut self.vm;\n\
                 \x20       vm.enter_call()?;\n\
                 \x20       let res = {start_call};\n\
                 \x20       vm.exit_call();\n\
                 \x20       res\n\
                 \x20   }}\n"
            );
        }
        None => {
            out.push_str(
                "    fn call_start(&mut self, _host: &mut dyn WasmSubstituteHost) -> std::result::Result<(), WasmTrap> {\n        Ok(())\n    }\n\n",
            );
        }
    }

    let raw_entry_args: Vec<String> = match module.entry_sig {
        EntrySignatureKind::UnitToI32 | EntrySignatureKind::UnitToUnit => {
            Vec::new()
        }
        EntrySignatureKind::I32PairToI32 | EntrySignatureKind::I32PairToUnit => {
            vec!["in_ptr".to_owned(), "in_len".to_owned()]
        }
    };
    let entry_call = format_call_invocation(
        module.entry_idx,
        &raw_entry_args,
        module,
        promoted_by_hash,
        None,
        None,
    )?;
    let entry_body = match module.entry_sig {
        EntrySignatureKind::UnitToI32 | EntrySignatureKind::I32PairToI32 => {
            entry_call
        }
        EntrySignatureKind::UnitToUnit | EntrySignatureKind::I32PairToUnit => {
            format!("{entry_call}.map(|()| 0i32)")
        }
    };
    let _ = writeln!(
        out,
        "    fn call_entry(&mut self, in_ptr: i32, in_len: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {{\n\
         \x20       let _ = (in_ptr, in_len);\n\
         \x20       let vm = &mut self.vm;\n\
         \x20       vm.enter_call()?;\n\
         \x20       let res = {entry_body};\n\
         \x20       vm.exit_call();\n\
         \x20       res\n\
         \x20   }}\n\
         }}\n"
    );
    Ok(())
}

fn emit_analyzed_module_source(
    module: &AnalyzedModule<'_>,
    promoted_by_hash: &BTreeMap<[u8; 32], PromotedSharedTarget>,
    input_sha256_hex: Option<&str>,
) -> Result<String> {
    let mut out = String::with_capacity(module.code_bodies.len().saturating_mul(256));
    emit_agpl_header(&mut out, module.name, input_sha256_hex);
    emit_data_and_elem_consts(&mut out, &module.parsed);
    emit_struct_and_instantiate(&mut out, module)?;
    emit_tail_call_types_and_dispatch(
        &mut out,
        &module.tail_target_fns,
        &module.fn_sigs,
    );

    let mut local_fn_indices: Vec<usize> = Vec::new();
    for fn_idx in 0..module.code_bodies.len() {
        let is_promoted = module
            .merkle_info
            .get(fn_idx)
            .and_then(Option::as_ref)
            .is_some_and(|m| promoted_by_hash.contains_key(&m.hash));
        if !is_promoted {
            local_fn_indices.push(fn_idx);
        }
    }

    if !local_fn_indices.is_empty() {
        emit_generated_allows(&mut out, 0);
        out.push_str("impl SubstituteInstance {\n");
        for fn_idx in local_fn_indices {
            let body = module
                .code_bodies
                .get(fn_idx)
                .context("Code body out of bounds during emission")?;
            let sig = module
                .fn_sigs
                .get(fn_idx)
                .context("Function signature out of bounds during emission")?;
            let ret_slug = ret_sig_slug(&sig.results);
            let is_cross_tail_step = module.tail_target_fns.contains(&fn_idx);
            let use_self_tail = *module
                .has_self_tail
                .get(fn_idx)
                .context("Function index out of bounds in has_self_tail")?;

            let mut params_decl = vec!["vm: &mut SubstituteVm".to_owned()];
            let mut forward_args =
                Vec::with_capacity(sig.params.len().saturating_add(2));
            forward_args.push("vm".to_owned());
            for (i, pt) in sig.params.iter().enumerate() {
                params_decl.push(format!("p{i}: {}", pt.rust_ty()));
                forward_args.push(format!("p{i}"));
            }
            params_decl.push("host: &mut dyn WasmSubstituteHost".to_owned());
            forward_args.push("host".to_owned());

            let ret_ty = ret_type_expr(&sig.results);
            let mut lowerer = FuncLowerer::new(
                fn_idx,
                module,
                promoted_by_hash,
                None,
                is_cross_tail_step,
                use_self_tail,
            )?;
            let lowered_body = lowerer.lower_body(body.code_bytes)?;

            if is_cross_tail_step {
                out.push_str("    #[inline(never)]\n");
                let _ = writeln!(
                    out,
                    "    fn fn_{fn_idx}_step({}) -> std::result::Result<TailStep_{ret_slug}, WasmTrap> {{",
                    params_decl.join(", ")
                );
                out.push_str(&lowered_body);
                out.push_str("    }\n\n");

                let _ = writeln!(
                    out,
                    "    fn fn_{fn_idx}({}) -> std::result::Result<{ret_ty}, WasmTrap> {{",
                    params_decl.join(", ")
                );
                let _ = writeln!(
                    out,
                    "        let step = SubstituteInstance::fn_{fn_idx}_step({})?;",
                    forward_args.join(", ")
                );
                let _ = writeln!(
                    out,
                    "        SubstituteInstance::tail_dispatch_{ret_slug}(vm, step, host)"
                );
                out.push_str("    }\n\n");
            } else {
                if body.code_bytes.len() > INLINE_NEVER_BYTECODE_THRESHOLD {
                    out.push_str("    #[inline(never)]\n");
                }
                let _ = writeln!(
                    out,
                    "    fn fn_{fn_idx}({}) -> std::result::Result<{ret_ty}, WasmTrap> {{",
                    params_decl.join(", ")
                );
                out.push_str(&lowered_body);
                out.push_str("    }\n\n");
            }
        }
        out.push_str("}\n\n");
    }

    emit_trait_impl(&mut out, module, promoted_by_hash)?;

    let _ = writeln!(
        out,
        "/// Returns the native WebAssembly substitute descriptor for `{}`.\n\
         #[must_use]\n\
         pub fn descriptor() -> WasmSubstituteDescriptor {{\n\
         \x20   WasmSubstituteDescriptor {{\n\
         \x20       meta: MODULE_META,\n\
         \x20       instantiate: instantiate_substitute,\n\
         \x20   }}\n\
         }}",
        module.name
    );
    Ok(out)
}

fn encode_hex_prefix_lower(bytes: &[u8], hex_chars: usize) -> String {
    let mut out = String::with_capacity(hex_chars);
    for &b in bytes {
        if out.len() >= hex_chars {
            break;
        }
        let _ = write!(out, "{b:02x}");
    }
    out.truncate(hex_chars);
    out
}

fn validate_bundle_module_stem(stem: &str) -> Result<()> {
    ensure!(
        !stem.is_empty(),
        "Substitute bundle module stem must not be empty"
    );
    ensure!(
        stem != "registry" && !stem.starts_with("shared_"),
        "Reserved substitute bundle module stem '{stem}'"
    );
    let mut chars = stem.chars();
    let Some(first) = chars.next() else {
        bail!("Empty substitute module stem");
    };
    ensure!(
        first.is_ascii_alphabetic() || first == '_',
        "Substitute module stem '{stem}' must start with ASCII letter or '_'"
    );
    ensure!(
        chars.all(|c| c.is_ascii_alphanumeric() || c == '_'),
        "Substitute module stem '{stem}' contains invalid identifier characters"
    );
    Ok(())
}

fn emit_shared_shard_source(
    shard_idx: usize,
    targets: &[&PromotedSharedTarget],
    analyzed_modules: &[AnalyzedModule<'_>],
    promoted_by_hash: &BTreeMap<[u8; 32], PromotedSharedTarget>,
) -> Result<String> {
    let mut out = String::with_capacity(targets.len().saturating_mul(256));
    emit_shared_header(&mut out, shard_idx);
    out.push_str("/// Shared Merkle-deduplicated WebAssembly function container.\npub(crate) struct Shared;\n\n");
    emit_generated_allows(&mut out, 0);
    out.push_str("impl Shared {\n");
    for target in targets {
        let module = analyzed_modules
            .get(target.repr_mod_idx)
            .context("Representative module index out of bounds")?;
        let fn_idx = target.repr_fn_idx;
        let body = module
            .code_bodies
            .get(fn_idx)
            .context("Representative code body out of bounds")?;
        let sig = module
            .fn_sigs
            .get(fn_idx)
            .context("Representative signature out of bounds")?;
        let merkle = module
            .merkle_info
            .get(fn_idx)
            .and_then(Option::as_ref)
            .context("Promoted shared target missing Merkle metadata")?;

        let mut global_pos_map: BTreeMap<(RustValTy, usize), usize> =
            BTreeMap::new();
        for (pos, &g_key) in merkle.used_globals.iter().enumerate() {
            global_pos_map.insert(g_key, pos);
        }
        let shared_ctx = SharedEmitContext {
            self_shard_idx: shard_idx,
            self_symbol_name: &target.symbol_name,
            global_pos_map,
        };

        let use_self_tail = *module
            .has_self_tail
            .get(fn_idx)
            .context("Function index out of bounds in has_self_tail")?;
        let mut lowerer = FuncLowerer::new(
            fn_idx,
            module,
            promoted_by_hash,
            Some(&shared_ctx),
            false,
            use_self_tail,
        )?;
        let lowered_body = lowerer.lower_body(body.code_bytes)?;

        let mut params_decl = vec!["vm: &mut SubstituteVm".to_owned()];
        for (i, pt) in sig.params.iter().enumerate() {
            params_decl.push(format!("p{i}: {}", pt.rust_ty()));
        }
        for pos in 0..merkle.used_globals.len() {
            params_decl.push(format!("g{pos}: usize"));
        }
        params_decl.push("host: &mut dyn WasmSubstituteHost".to_owned());

        let ret_ty = ret_type_expr(&sig.results);
        if body.code_bytes.len() > INLINE_NEVER_BYTECODE_THRESHOLD {
            out.push_str("    #[inline(never)]\n");
        }
        let _ = writeln!(
            out,
            "    pub(crate) fn {}({}) -> std::result::Result<{ret_ty}, WasmTrap> {{",
            target.symbol_name,
            params_decl.join(", ")
        );
        out.push_str(&lowered_body);
        out.push_str("    }\n\n");
    }
    out.push_str("}\n");
    Ok(out)
}

/// Transpiles a bundle of uninstrumented canonical WebAssembly modules into
/// composable, Merkle-deduplicated safe-Rust files plus a generated
/// `registry.generated.rs` manifest.
///
/// # Errors
/// Returns an error if any module stem is invalid or duplicated, if any module
/// fails parsing or validation, or if codegen encounters unsupported opcodes.
pub fn transpile_wasm_bundle_to_safe_rust(
    modules: &[(&str, &[u8])],
    input_sha256_hex: Option<&str>,
) -> Result<BTreeMap<String, String>> {
    let mut seen_stems: BTreeSet<&str> = BTreeSet::new();
    let mut analyzed_modules: Vec<AnalyzedModule<'_>> =
        Vec::with_capacity(modules.len());
    for &(stem, bytes) in modules {
        validate_bundle_module_stem(stem)?;
        ensure!(
            seen_stems.insert(stem),
            "Duplicate substitute module stem '{stem}'"
        );
        analyzed_modules.push(analyze_module(stem, bytes)?);
    }

    // Count Merkle hashes across all modules and remember the first occurrence
    // as the deterministic representative for shared code emission.
    let mut counts: BTreeMap<[u8; 32], usize> = BTreeMap::new();
    let mut first_repr: BTreeMap<[u8; 32], (usize, usize)> = BTreeMap::new();
    let mut callees_by_hash: BTreeMap<[u8; 32], Vec<[u8; 32]>> = BTreeMap::new();

    for (mod_idx, m) in analyzed_modules.iter().enumerate() {
        for (fn_idx, info_opt) in m.merkle_info.iter().enumerate() {
            if let Some(info) = info_opt {
                let c = counts.entry(info.hash).or_insert(0);
                *c = c.saturating_add(1);
                first_repr.entry(info.hash).or_insert((mod_idx, fn_idx));
                callees_by_hash
                    .entry(info.hash)
                    .or_insert_with(|| info.callee_hashes.clone());
            }
        }
    }

    // Promote every hash occurring >= 2 times plus the transitive closure of
    // its Merkle callees so shared functions never call private module code.
    let mut promoted_hashes: BTreeSet<[u8; 32]> = BTreeSet::new();
    let mut queue: Vec<[u8; 32]> = Vec::new();
    for (&hash, &count) in &counts {
        if count >= 2 && promoted_hashes.insert(hash) {
            queue.push(hash);
        }
    }
    while let Some(h) = queue.pop() {
        if let Some(callees) = callees_by_hash.get(&h) {
            for &ch in callees {
                if promoted_hashes.insert(ch) {
                    queue.push(ch);
                }
            }
        }
    }

    let sorted_promoted: Vec<[u8; 32]> = promoted_hashes.into_iter().collect();
    let mut prefix_hex_len = 12usize;
    for candidate_len in [12usize, 16, 24, 64] {
        let mut names_check: BTreeSet<String> = BTreeSet::new();
        let all_unique = sorted_promoted.iter().all(|h| {
            names_check.insert(encode_hex_prefix_lower(h, candidate_len))
        });
        if all_unique {
            prefix_hex_len = candidate_len;
            break;
        }
    }

    let mut promoted_by_hash: BTreeMap<[u8; 32], PromotedSharedTarget> =
        BTreeMap::new();
    for (ordinal, &hash) in sorted_promoted.iter().enumerate() {
        let shard_idx = ordinal
            .checked_div(MAX_SHARED_FNS_PER_SHARD)
            .context("Invalid shard size")?;
        let shard_mod = format!("shared_{shard_idx}");
        let hex_prefix = encode_hex_prefix_lower(&hash, prefix_hex_len);
        let symbol_name = format!("s_{hex_prefix}");
        let &(repr_mod_idx, repr_fn_idx) = first_repr
            .get(&hash)
            .context("Promoted hash missing representative module/function")?;
        promoted_by_hash.insert(
            hash,
            PromotedSharedTarget {
                hash,
                symbol_name,
                shard_idx,
                shard_mod,
                repr_mod_idx,
                repr_fn_idx,
            },
        );
    }

    let mut files: BTreeMap<String, String> = BTreeMap::new();

    // Emit shared shards sorted deterministically.
    let mut shard_buckets: BTreeMap<usize, Vec<&PromotedSharedTarget>> =
        BTreeMap::new();
    for target in promoted_by_hash.values() {
        shard_buckets
            .entry(target.shard_idx)
            .or_default()
            .push(target);
    }
    let mut shard_mods: Vec<String> = Vec::with_capacity(shard_buckets.len());
    for (shard_idx, targets) in &shard_buckets {
        let shard_mod = format!("shared_{shard_idx}");
        let shard_file = format!("{shard_mod}.generated.rs");
        let src = emit_shared_shard_source(
            *shard_idx,
            targets,
            &analyzed_modules,
            &promoted_by_hash,
        )?;
        files.insert(shard_file, src);
        shard_mods.push(shard_mod);
    }

    // Emit each module file.
    for m in &analyzed_modules {
        let mod_file = format!("{}.generated.rs", m.name);
        let src =
            emit_analyzed_module_source(m, &promoted_by_hash, None)?;
        files.insert(mod_file, src);
    }

    // Emit registry.generated.rs manifest included by wasm_substitutes.rs.
    let mut reg_out = String::new();
    emit_registry_header(&mut reg_out, input_sha256_hex);
    for shard_mod in &shard_mods {
        let _ = writeln!(
            reg_out,
            "#[path = \"{shard_mod}.generated.rs\"]\nmod {shard_mod};"
        );
    }
    for m in &analyzed_modules {
        let stem = m.name;
        let _ = writeln!(
            reg_out,
            "#[path = \"{stem}.generated.rs\"]\nmod {stem};"
        );
    }
    reg_out.push_str(
        "\nfn register_all_substitutes() {\n",
    );
    for m in &analyzed_modules {
        let _ = writeln!(
            reg_out,
            "    ctb_wasm_substitute_rt::register_substitute({}::descriptor());",
            m.name
        );
    }
    reg_out.push_str("}\n");
    files.insert("registry.generated.rs".to_owned(), reg_out);

    Ok(files)
}

/// Transpiles an uninstrumented canonical WebAssembly binary into a standalone
/// safe-Rust substitute module implementing exact V2 structural-checkpoint
/// metering.
///
/// # Errors
/// Returns an error if parsing or validating `wasm_bytes` fails, if the module
/// uses unsupported features (host imports, `memory64`, `v128` SIMD, missing
/// `"step"`/`"main"` entry, or unsupported opcodes), or if constant evaluation fails.
pub fn transpile_wasm_to_safe_rust(
    wasm_bytes: &[u8],
    opts: &TranspileWasmOptions<'_>,
) -> Result<String> {
    let analyzed = analyze_module(opts.name, wasm_bytes)?;
    let empty_promoted = BTreeMap::new();
    emit_analyzed_module_source(
        &analyzed,
        &empty_promoted,
        opts.input_sha256_hex,
    )
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
    use crate::converters::wasm::{
        dcmstring_to_wasm_with_resolver,
        expand_graph_wasm_shorthands_to_dcmstring,
    };
    use crate::converters::wat::wat_to_wasm;
    use ctb_formats_dcdata::syntax::{SyntaxRuleResolver, default_syntax_resolver};
    use ctb_storage_minimal::shorthand::DcShorthand;

    fn assert_clean_rust_common(src: &str) {
        assert!(!src.contains("#![allow"), "Unexpected crate-level allow");
        let after_header = src
            .split_once("use crate::utilities::*;")
            .expect("Generated file missing standard prelude")
            .1;
        let code_lines: String = after_header
            .lines()
            .filter(|l| {
                let t = l.trim_start();
                !t.starts_with("//")
                    && !t.starts_with("const DATA_SEG_")
                    && !l.ends_with('\\')
                    && !t.ends_with("\";")
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            !code_lines.contains(" as "),
            "Unexpected `as` cast in generated code"
        );
        assert!(
            !code_lines.contains(".unwrap("),
            "Unexpected `.unwrap(` in generated code"
        );
        assert!(
            !code_lines.contains(".expect("),
            "Unexpected `.expect(` in generated code"
        );
        assert!(
            !code_lines.contains(".unwrap_or("),
            "Unexpected `.unwrap_or(` in generated code"
        );
        assert!(
            !code_lines.contains("unsafe"),
            "Unexpected `unsafe` block in generated code"
        );
    }

    fn assert_clean_generated_rust(src: &str) {
        assert_clean_rust_common(src);
        assert!(src.contains("pub fn descriptor() -> WasmSubstituteDescriptor"));
    }

    #[crate::ctb_test]
    fn test_transpile_structured_control_flow_tail_calls_and_segments() {
        let wat = r#"(module
            (memory 1 4)
            (global $__heap_ptr (export "__heap_ptr") (mut i32) (i32.const 1024))
            (data (i32.const 64) "hello\00world")
            (func $__ctb_capability_yield (export "__ctb_capability_yield")
                nop
            )
            (func $dag_leaf (param $x i32) (result i32)
                (i32.add (local.get $x) (i32.const 7))
            )
            (func $dag_caller (param $x i32) (result i32)
                (return_call $dag_leaf (i32.add (local.get $x) (i32.const 1)))
            )
            (func $even (param $n i32) (result i32)
                (if (result i32) (i32.eqz (local.get $n))
                    (then (i32.const 1))
                    (else (return_call $odd (i32.sub (local.get $n) (i32.const 1))))
                )
            )
            (func $odd (param $n i32) (result i32)
                (if (result i32) (i32.eqz (local.get $n))
                    (then (i32.const 0))
                    (else (return_call $even (i32.sub (local.get $n) (i32.const 1))))
                )
            )
            (func $self_tail (param $n i32) (param $acc i32) (result i32)
                (if (result i32) (i32.eqz (local.get $n))
                    (then (local.get $acc))
                    (else
                        (return_call $self_tail
                            (i32.sub (local.get $n) (i32.const 1))
                            (i32.add (local.get $acc) (i32.const 2))
                        )
                    )
                )
            )
            (func (export "step") (result i32)
                (local $i i32)
                (call $__ctb_capability_yield)
                (drop (call $dag_caller (i32.const 3)))
                (drop (call $even (i32.const 5)))
                (drop (call $self_tail (i32.const 4) (i32.const 0)))
                (drop (memory.grow (i32.const 1)))
                (loop $l
                    (local.set $i (i32.add (local.get $i) (i32.const 1)))
                    (br_if $l (i32.lt_s (local.get $i) (i32.const 3)))
                )
                (block
                    (br_table 0 0 (i32.const 1))
                    (drop (i32.const 999))
                )
                (i32.const 0)
            )
        )"#;
        let wasm_bytes = wat_to_wasm(wat).expect("WAT compilation must succeed");
        let generated = transpile_wasm_to_safe_rust(
            &wasm_bytes,
            &TranspileWasmOptions {
                name: "unit_smoke",
                input_sha256_hex: Some("deadbeef"),
            },
        )
        .expect("Transpiling test WASM must succeed");

        assert_clean_generated_rust(&generated);
        assert!(generated.contains("TailTarget_i32"));
        // DAG `return_call $dag_leaf` (fn_1) calls directly without a TailTarget variant.
        assert!(!generated.contains("TailTarget_i32::Func1("));
        assert!(generated.contains("return SubstituteInstance::fn_1(vm, "));
        assert!(generated.contains("'self_tail: loop"));
        assert!(generated.contains("host.request_capability(vm)"));
        assert!(generated.contains("vm.memory_grow("));
        assert!(generated.contains("input-sha256: deadbeef"));
    }

    #[crate::ctb_test]
    fn test_transpile_raw_binary_indirect_calls_passive_data_and_start() {
        let mut raw_wasm = WASM_HEADER.to_vec();
        let push_sec = |id: u8, payload: &[u8], out: &mut Vec<u8>| {
            out.push(id);
            out.push(u8::try_from(payload.len()).unwrap());
            out.extend_from_slice(payload);
        };
        // Type section: 0=(i32,i32)->i32, 1=()->(), 2=()->i32
        push_sec(
            1,
            &[3, 0x60, 2, 0x7F, 0x7F, 1, 0x7F, 0x60, 0, 0, 0x60, 0, 1, 0x7F],
            &mut raw_wasm,
        );
        // Function section: 3 funcs [type 1, type 0, type 2]
        push_sec(3, &[3, 1, 0, 2], &mut raw_wasm);
        // Table section: 1 funcref table min=2
        push_sec(4, &[1, 0x70, 0, 2], &mut raw_wasm);
        // Memory section: 1 memory min=1
        push_sec(5, &[1, 0, 1], &mut raw_wasm);
        // Export section: "memory" (mem 0), "main" (fn 2)
        push_sec(
            7,
            &[
                2, 6, b'm', b'e', b'm', b'o', b'r', b'y', 2, 0, 4, b'm',
                b'a', b'i', b'n', 0, 2,
            ],
            &mut raw_wasm,
        );
        // Start section: fn 0
        push_sec(8, &[0], &mut raw_wasm);
        // Element section: 1 active segment at offset 0: [fn 1]
        push_sec(9, &[1, 0, 0x41, 0, 0x0B, 1, 1], &mut raw_wasm);
        // DataCount section: 1
        push_sec(12, &[1], &mut raw_wasm);
        // Code section: 3 bodies
        #[rustfmt::skip]
        let bodies: [&[u8]; 3] = [
            // fn 0 (start): nop; end
            &[0, 0x01, 0x0B],
            // fn 1 (p0, p1) -> i32: if (p0 == 0) { p1 } else { return_call_indirect type 0 (p0 - 1, p1 + 1, 0) }
            &[
                0,
                0x20, 0x00, 0x45,
                0x04, 0x7F,
                    0x20, 0x01,
                0x05,
                    0x20, 0x00, 0x41, 0x01, 0x6B,
                    0x20, 0x01, 0x41, 0x01, 0x6A,
                    0x41, 0x00,
                    0x13, 0x00, 0x00,
                0x0B,
                0x0B,
            ],
            // fn 2 () -> i32: memory.init 0 0; data.drop 0; call_indirect(2, 10, 0)
            &[
                0,
                0x41, 0x10, 0x41, 0x00, 0x41, 0x02, 0xFC, 0x08, 0x00, 0x00,
                0xFC, 0x09, 0x00,
                0x41, 0x02, 0x41, 0x0A, 0x41, 0x00, 0x11, 0x00, 0x00,
                0x0B,
            ],
        ];
        let mut code_payload = vec![u8::try_from(bodies.len()).unwrap()];
        for body in bodies {
            code_payload.push(u8::try_from(body.len()).unwrap());
            code_payload.extend_from_slice(body);
        }
        push_sec(10, &code_payload, &mut raw_wasm);
        // Data section: 1 passive segment b"ok"
        push_sec(11, &[1, 1, 2, b'o', b'k'], &mut raw_wasm);

        let generated = transpile_wasm_to_safe_rust(
            &raw_wasm,
            &TranspileWasmOptions {
                name: "raw_smoke",
                input_sha256_hex: None,
            },
        )
        .expect("Transpiling raw binary must succeed");
        assert_clean_generated_rust(&generated);
        assert!(generated.contains("call_indirect_target"));
        assert!(generated.contains("vm.memory_init("));
        assert!(generated.contains("vm.data_drop("));
        assert!(generated.contains("TailTarget_i32"));
        assert!(generated.contains("has_start_func: true"));
    }

    #[crate::ctb_test]
    fn test_transpile_wasm_bundle_merkle_dedup_and_global_remapping()
    -> Result<()> {
        // Module A: $cnt at typed i32 slot 0, $__heap_ptr at global 1,
        // $init_vals sets $cnt, and $sum_rec + $work share structure with Module B.
        let wat_a = r#"(module
            (memory (export "memory") 1)
            (global $cnt (mut i32) (i32.const 10))
            (global $__heap_ptr (export "__heap_ptr") (mut i32) (i32.const 256))
            (func $init_vals
                (global.set $cnt (i32.const 20))
            )
            (func $sum_rec (param $n i32) (param $acc i32) (result i32)
                (if (result i32) (i32.eqz (local.get $n))
                    (then (local.get $acc))
                    (else
                        (return_call $sum_rec
                            (i32.sub (local.get $n) (i32.const 1))
                            (i32.add (local.get $acc) (local.get $n))
                        )
                    )
                )
            )
            (func $work (param $x i32) (result i32)
                (call $init_vals)
                (block (result i32)
                    (i32.add (local.get $x) (global.get $cnt))
                    (global.set $__heap_ptr (i32.add (global.get $__heap_ptr) (i32.const 8)))
                )
                (return_call $sum_rec (i32.const 4))
            )
            (func (export "step") (result i32)
                (call $init_vals)
                (i32.add (call $work (i32.const 5)) (global.get $cnt))
            )
        )"#;

        // Module B: introduces a leading unique signature ($extra_sig) so all
        // WASM type indices shift, reorders globals ($__heap_ptr first, extra $pad
        // at typed slot 0 -> $cnt is typed slot 1!), plus unique step body.
        let wat_b = r#"(module
            (memory (export "memory") 1)
            (global $__heap_ptr (export "__heap_ptr") (mut i32) (i32.const 512))
            (global $pad (mut i32) (i32.const 99))
            (global $cnt (mut i32) (i32.const 30))
            (func $extra_sig (param $v i64) (result i64)
                (i64.add (local.get $v) (i64.const 1))
            )
            (func $init_vals_b
                (global.set $cnt (i32.add (global.get $pad) (i32.const 1)))
            )
            (func $sum_rec (param $n i32) (param $acc i32) (result i32)
                (if (result i32) (i32.eqz (local.get $n))
                    (then (local.get $acc))
                    (else
                        (return_call $sum_rec
                            (i32.sub (local.get $n) (i32.const 1))
                            (i32.add (local.get $acc) (local.get $n))
                        )
                    )
                )
            )
            (func $work (param $x i32) (result i32)
                (call $init_vals_b)
                (block (result i32)
                    (i32.add (local.get $x) (global.get $cnt))
                    (global.set $__heap_ptr (i32.add (global.get $__heap_ptr) (i32.const 8)))
                )
                (return_call $sum_rec (i32.const 4))
            )
            (func (export "step") (result i32)
                (call $init_vals_b)
                (drop (call $extra_sig (i64.const 7)))
                (i32.mul (call $work (i32.const 6)) (i32.const 2))
            )
        )"#;

        let wasm_a = wat_to_wasm(wat_a)?;
        let wasm_b = wat_to_wasm(wat_b)?;
        let files = transpile_wasm_bundle_to_safe_rust(
            &[("mod_a", &wasm_a), ("mod_b", &wasm_b)],
            Some("bundle123"),
        )?;

        let keys: Vec<&str> = files.keys().map(String::as_str).collect();
        assert_eq!(
            keys,
            vec![
                "mod_a.generated.rs",
                "mod_b.generated.rs",
                "registry.generated.rs",
                "shared_0.generated.rs",
            ]
        );

        let shared_rs = files
            .get("shared_0.generated.rs")
            .context("Missing shared_0.generated.rs")?;
        assert_clean_rust_common(shared_rs);
        assert!(shared_rs.contains("pub(crate) struct Shared;"));
        assert!(shared_rs.contains("pub(crate) fn s_"));
        assert!(shared_rs.contains("vm.call_init(host)"));
        assert!(shared_rs.contains("vm.heap_ptr"));
        assert!(shared_rs.contains("'self_tail: loop"));

        let mod_a_rs = files
            .get("mod_a.generated.rs")
            .context("Missing mod_a.generated.rs")?;
        let mod_b_rs = files
            .get("mod_b.generated.rs")
            .context("Missing mod_b.generated.rs")?;
        assert_clean_generated_rust(mod_a_rs);
        assert_clean_generated_rust(mod_b_rs);
        assert!(mod_a_rs.contains("crate::shared_0::Shared::s_"));
        assert!(mod_b_rs.contains("crate::shared_0::Shared::s_"));
        // Module A passes typed i32 slot `0usize` for `$cnt`; Module B passes `1usize`.
        assert!(mod_a_rs.contains(", 0usize, host)"));
        assert!(mod_b_rs.contains(", 1usize, host)"));

        let reg_rs = files
            .get("registry.generated.rs")
            .context("Missing registry.generated.rs")?;
        assert!(reg_rs.contains("// @generated by ctb-wasm-substitutes-gen"));
        assert!(reg_rs.contains("// input-sha256: bundle123"));
        assert!(reg_rs.contains("#[path = \"shared_0.generated.rs\"]"));
        assert!(reg_rs.contains("register_substitute(mod_a::descriptor());"));
        assert!(reg_rs.contains("register_substitute(mod_b::descriptor());"));
        Ok(())
    }

    #[crate::ctb_test]
    fn test_transpile_real_dc_1936_and_f1680_from_seed_graph() -> Result<()> {
        let resolver = default_syntax_resolver();
        let sg = resolver
            .seed_graph()
            .context("Default resolver missing seed_graph")?;

        // Dc 1936 ("Dc document execution kernel")
        let kernel_sh = DcShorthand::from_global_id(
            ctb_formats_dcdata::dc::DC_DC_DOCUMENT_EXECUTION_KERNEL.to_u128(),
        );
        let (_, kernel_tokens) = sg
            .resolve_wasm_helper_shorthand(kernel_sh)?
            .context("Dc 1936 missing active WASM decomposition")?;
        let kernel_doc = expand_graph_wasm_shorthands_to_dcmstring(
            kernel_tokens,
            resolver.as_ref(),
        )?;
        let kernel_wasm =
            dcmstring_to_wasm_with_resolver(&kernel_doc, resolver.as_ref())?;
        let kernel_rs = transpile_wasm_to_safe_rust(
            &kernel_wasm,
            &TranspileWasmOptions {
                name: "dc_1936",
                input_sha256_hex: None,
            },
        )?;
        assert_clean_generated_rust(&kernel_rs);
        assert!(
            kernel_rs.contains("entry_func_name: \"main\"")
                || kernel_rs.contains("entry_func_name: \"step\"")
        );

        // Dc f1680 ("StageL 3 Dcs to WASM")
        let stagel_src = DcShorthand::from_global_id(
            ctb_formats_dcdata::dc::DC_BEGIN_MODULE.to_u128(),
        );
        let wasm_dst = DcShorthand::from_global_id(
            ctb_formats_dcdata::dc::DC_BEGIN_WASM_PROGRAM.to_u128(),
        );
        let (_, f1680_tokens) = sg
            .resolve_active_converter_routine(stagel_src, wasm_dst)?
            .context("Dc f1680 missing active WASM converter routine")?;
        let f1680_doc = expand_graph_wasm_shorthands_to_dcmstring(
            f1680_tokens,
            resolver.as_ref(),
        )?;
        let f1680_wasm =
            dcmstring_to_wasm_with_resolver(&f1680_doc, resolver.as_ref())?;
        let f1680_rs = transpile_wasm_to_safe_rust(
            &f1680_wasm,
            &TranspileWasmOptions {
                name: "f1680",
                input_sha256_hex: None,
            },
        )?;
        assert_clean_generated_rust(&f1680_rs);
        assert!(f1680_rs.contains("entry_func_name: \"step\"") || f1680_rs.contains("entry_func_name: \"main\""));

        Ok(())
    }
}
