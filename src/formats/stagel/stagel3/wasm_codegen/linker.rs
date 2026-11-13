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

//! `.WASM` (`Dc 1927`..`Dc 1928`) instruction builder and System-graph Stage 0
//! built-in linker for StageL 3 WebAssembly compilation.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use ctb_formats_dcdata::dc::{
    DC_BEGIN_IDENTIFIER, DC_BEGIN_KV_MAP, DC_BEGIN_KV_PAIR, DC_BEGIN_LIST,
    DC_BEGIN_LITERAL, DC_BEGIN_NUMBER, DC_BEGIN_TYPE_DEF,
    DC_BEGIN_WASM_PROGRAM, DC_END_IDENTIFIER, DC_END_KV_MAP, DC_END_KV_PAIR,
    DC_END_LIST, DC_END_LITERAL, DC_END_NUMBER, DC_END_TYPE_DEF,
    DC_END_WASM_PROGRAM, DC_ENTITY_TYPE, DC_ESCAPE,
    DC_LIST_ELEMENT_SEPARATOR, DC_OPTIONAL_ABSENT, DC_STAGEL_BUF_NEW,
    DC_STAGEL_BUF_TYPE, DC_TYPE_ANY, DC_TYPE_ROUTINE, DC_WASM_FB_START,
    DC_WASM_FC_START, DC_WASM_FD_PAGE0_START, DC_WASM_START,
};
use ctb_formats_dcdata::dc_number_minimal::read_dc_number_u128_with_resolver;
use ctb_formats_dcdata::seed_graph::SeedGraphIndex;
use ctb_formats_dcdata::support::wat::{
    encode_i64_dc_shorthands, encode_u128_dc_shorthands,
};
use ctb_formats_dcdata::syntax::default_syntax_resolver;
use ctb_formats_dcdata::{DcChar, DcShorthand};
use ctb_formats_dcstring::DcMstring;
use ctb_utilities::format_id::FormatId;
use std::collections::HashMap;

/// WebAssembly value type emitted in `.WASM` signatures, locals, and blocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WasmValType {
    /// 32-bit integer (`f208`).
    I32,
    /// 64-bit integer (`f210`).
    I64,
}

impl WasmValType {
    /// Converts this value type into its canonical format [`DcShorthand`].
    pub fn to_shorthand(self) -> Result<DcShorthand> {
        let fmt = match self {
            Self::I32 => FormatId::I32,
            Self::I64 => FormatId::I64,
        };
        let dc = fmt
            .dc_char()
            .with_context(|| format!("Missing DcChar for {fmt:?}"))?;
        Ok(dc_to_sh(dc))
    }
}

pub(super) fn dc_to_sh(dc: DcChar) -> DcShorthand {
    DcShorthand::from_global_id(dc.to_u128())
}

fn wasm_op_sh(subopcode: u8) -> Result<DcShorthand> {
    let gid = DC_WASM_START
        .to_u128()
        .checked_add(u128::from(subopcode))
        .context("WASM opcode global ID overflow")?;
    Ok(DcShorthand::from_global_id(gid))
}

fn wasm_fc_op_sh(subopcode: u8) -> Result<DcShorthand> {
    let gid = DC_WASM_FC_START
        .to_u128()
        .checked_add(u128::from(subopcode))
        .context("WASM 0xFC opcode global ID overflow")?;
    Ok(DcShorthand::from_global_id(gid))
}

pub(super) fn push_identifier_shorthands(
    name: &str,
    leading_type: &[DcShorthand],
    out: &mut Vec<DcShorthand>,
) {
    out.push(dc_to_sh(DC_BEGIN_IDENTIFIER));
    out.extend_from_slice(leading_type);
    for ch in name.chars() {
        out.push(DcShorthand::Unicode(u32::from(ch)));
    }
    out.push(dc_to_sh(DC_END_IDENTIFIER));
}

/// Builder for a single `.WASM` routine (`Dc 260`..`Dc 261`).
#[derive(Debug, Clone, Default)]
pub struct WasmFuncBuilder {
    params: Vec<(String, WasmValType)>,
    result: Option<WasmValType>,
    locals: Vec<WasmValType>,
    body: Vec<DcShorthand>,
}

impl WasmFuncBuilder {
    /// Creates a new function builder with the given parameters and return type.
    #[must_use]
    pub fn new(
        params: Vec<(String, WasmValType)>,
        result: Option<WasmValType>,
    ) -> Self {
        Self {
            params,
            result,
            locals: Vec::new(),
            body: Vec::new(),
        }
    }

    /// Allocates a new local variable of type `vt` and returns its 0-based
    /// WASM local index (offset past formal parameters).
    pub fn alloc_local(&mut self, vt: WasmValType) -> Result<u32> {
        let idx_usize = self
            .params
            .len()
            .checked_add(self.locals.len())
            .context("WASM local count overflow")?;
        let idx =
            u32::try_from(idx_usize).context("WASM local index exceeds u32")?;
        self.locals.push(vt);
        Ok(idx)
    }

    pub fn unreachable(&mut self) -> Result<()> {
        self.body.push(wasm_op_sh(0x00)?);
        Ok(())
    }

    pub fn block_open(&mut self, bt: Option<WasmValType>) -> Result<()> {
        self.body.push(wasm_op_sh(0x02)?);
        self.push_blocktype(bt)
    }

    pub fn loop_open(&mut self, bt: Option<WasmValType>) -> Result<()> {
        self.body.push(wasm_op_sh(0x03)?);
        self.push_blocktype(bt)
    }

    pub fn if_open(&mut self, bt: Option<WasmValType>) -> Result<()> {
        self.body.push(wasm_op_sh(0x04)?);
        self.push_blocktype(bt)
    }

    pub fn else_op(&mut self) -> Result<()> {
        self.body.push(wasm_op_sh(0x05)?);
        Ok(())
    }

    pub fn end_op(&mut self) -> Result<()> {
        self.body.push(wasm_op_sh(0x0B)?);
        Ok(())
    }

    pub fn br(&mut self, depth: u32) -> Result<()> {
        self.body.push(wasm_op_sh(0x0C)?);
        self.body.extend(encode_u128_dc_shorthands(u128::from(depth)));
        Ok(())
    }

    pub fn br_if(&mut self, depth: u32) -> Result<()> {
        self.body.push(wasm_op_sh(0x0D)?);
        self.body.extend(encode_u128_dc_shorthands(u128::from(depth)));
        Ok(())
    }

    pub fn return_op(&mut self) -> Result<()> {
        self.body.push(wasm_op_sh(0x0F)?);
        Ok(())
    }

    pub fn call(&mut self, func_idx: u32) -> Result<()> {
        self.body.push(wasm_op_sh(0x10)?);
        self.body
            .extend(encode_u128_dc_shorthands(u128::from(func_idx)));
        Ok(())
    }

    /// Emits `Dc 665 "WASM return_call"` (`0x12`).
    pub fn return_call(&mut self, func_idx: u32) -> Result<()> {
        self.body.push(wasm_op_sh(0x12)?);
        self.body
            .extend(encode_u128_dc_shorthands(u128::from(func_idx)));
        Ok(())
    }

    pub fn drop_op(&mut self) -> Result<()> {
        self.body.push(wasm_op_sh(0x1A)?);
        Ok(())
    }

    pub fn local_get(&mut self, local_idx: u32) -> Result<()> {
        self.body.push(wasm_op_sh(0x20)?);
        self.body
            .extend(encode_u128_dc_shorthands(u128::from(local_idx)));
        Ok(())
    }

    pub fn local_set(&mut self, local_idx: u32) -> Result<()> {
        self.body.push(wasm_op_sh(0x21)?);
        self.body
            .extend(encode_u128_dc_shorthands(u128::from(local_idx)));
        Ok(())
    }

    pub fn local_tee(&mut self, local_idx: u32) -> Result<()> {
        self.body.push(wasm_op_sh(0x22)?);
        self.body
            .extend(encode_u128_dc_shorthands(u128::from(local_idx)));
        Ok(())
    }

    pub fn global_get(&mut self, global_idx: u32) -> Result<()> {
        self.body.push(wasm_op_sh(0x23)?);
        self.body
            .extend(encode_u128_dc_shorthands(u128::from(global_idx)));
        Ok(())
    }

    pub fn global_set(&mut self, global_idx: u32) -> Result<()> {
        self.body.push(wasm_op_sh(0x24)?);
        self.body
            .extend(encode_u128_dc_shorthands(u128::from(global_idx)));
        Ok(())
    }

    pub fn i32_load(&mut self, align_exp: u32, offset: u64) -> Result<()> {
        self.body.push(wasm_op_sh(0x28)?);
        self.push_memarg(align_exp, offset);
        Ok(())
    }

    pub fn i64_load(&mut self, align_exp: u32, offset: u64) -> Result<()> {
        self.body.push(wasm_op_sh(0x29)?);
        self.push_memarg(align_exp, offset);
        Ok(())
    }

    pub fn i32_load8_u(&mut self, offset: u64) -> Result<()> {
        self.body.push(wasm_op_sh(0x2D)?);
        self.push_memarg(0, offset);
        Ok(())
    }

    pub fn i32_store(&mut self, align_exp: u32, offset: u64) -> Result<()> {
        self.body.push(wasm_op_sh(0x36)?);
        self.push_memarg(align_exp, offset);
        Ok(())
    }

    pub fn i64_store(&mut self, align_exp: u32, offset: u64) -> Result<()> {
        self.body.push(wasm_op_sh(0x37)?);
        self.push_memarg(align_exp, offset);
        Ok(())
    }

    pub fn i32_store8(&mut self, offset: u64) -> Result<()> {
        self.body.push(wasm_op_sh(0x3A)?);
        self.push_memarg(0, offset);
        Ok(())
    }

    pub fn i32_const(&mut self, val: i32) -> Result<()> {
        self.body.push(wasm_op_sh(0x41)?);
        self.body.extend(encode_i64_dc_shorthands(i64::from(val)));
        Ok(())
    }

    pub fn i64_const(&mut self, val: i64) -> Result<()> {
        self.body.push(wasm_op_sh(0x42)?);
        self.body.extend(encode_i64_dc_shorthands(val));
        Ok(())
    }

    pub fn i32_eqz(&mut self) -> Result<()> {
        self.body.push(wasm_op_sh(0x45)?);
        Ok(())
    }

    pub fn i32_eq(&mut self) -> Result<()> {
        self.body.push(wasm_op_sh(0x46)?);
        Ok(())
    }

    pub fn i32_lt_u(&mut self) -> Result<()> {
        self.body.push(wasm_op_sh(0x49)?);
        Ok(())
    }

    pub fn i32_gt_u(&mut self) -> Result<()> {
        self.body.push(wasm_op_sh(0x4B)?);
        Ok(())
    }

    pub fn i32_ge_u(&mut self) -> Result<()> {
        self.body.push(wasm_op_sh(0x4F)?);
        Ok(())
    }

    pub fn i32_add(&mut self) -> Result<()> {
        self.body.push(wasm_op_sh(0x6A)?);
        Ok(())
    }

    pub fn i32_sub(&mut self) -> Result<()> {
        self.body.push(wasm_op_sh(0x6B)?);
        Ok(())
    }

    pub fn i32_mul(&mut self) -> Result<()> {
        self.body.push(wasm_op_sh(0x6C)?);
        Ok(())
    }

    pub fn memory_copy(&mut self) -> Result<()> {
        self.body.push(wasm_fc_op_sh(10)?);
        self.body.extend(encode_u128_dc_shorthands(0));
        self.body.extend(encode_u128_dc_shorthands(0));
        Ok(())
    }

    fn push_blocktype(&mut self, bt: Option<WasmValType>) -> Result<()> {
        if let Some(vt) = bt {
            self.body.push(vt.to_shorthand()?);
        } else {
            self.body.push(dc_to_sh(DC_OPTIONAL_ABSENT));
        }
        Ok(())
    }

    fn push_memarg(&mut self, align_exp: u32, offset: u64) {
        self.body.extend(encode_u128_dc_shorthands(0));
        self.body
            .extend(encode_u128_dc_shorthands(u128::from(align_exp)));
        self.body
            .extend(encode_u128_dc_shorthands(u128::from(offset)));
    }

    /// Finalizes this routine into canonical `Dc 260`..`Dc 261` shorthands.
    pub fn into_routine_shorthands(self) -> Result<Vec<DcShorthand>> {
        let mut out = Vec::new();
        out.push(dc_to_sh(DC_BEGIN_LITERAL));
        out.push(dc_to_sh(DC_BEGIN_TYPE_DEF));
        out.push(dc_to_sh(DC_TYPE_ROUTINE));

        if let Some(ret_vt) = self.result {
            out.push(ret_vt.to_shorthand()?);
        } else if !self.params.is_empty() {
            out.push(dc_to_sh(DC_OPTIONAL_ABSENT));
        }

        if !self.params.is_empty() {
            out.push(dc_to_sh(DC_BEGIN_LIST));
            out.push(dc_to_sh(DC_TYPE_ANY));
            for (idx, (pname, pvt)) in self.params.iter().enumerate() {
                if idx > 0 {
                    out.push(dc_to_sh(DC_LIST_ELEMENT_SEPARATOR));
                }
                push_identifier_shorthands(pname, &[pvt.to_shorthand()?], &mut out);
            }
            out.push(dc_to_sh(DC_END_LIST));
        }

        out.push(dc_to_sh(DC_END_TYPE_DEF));

        if !self.locals.is_empty() {
            out.push(dc_to_sh(DC_BEGIN_LIST));
            out.push(dc_to_sh(DC_ENTITY_TYPE));
            for (idx, lvt) in self.locals.iter().enumerate() {
                if idx > 0 {
                    out.push(dc_to_sh(DC_LIST_ELEMENT_SEPARATOR));
                }
                out.push(lvt.to_shorthand()?);
            }
            out.push(dc_to_sh(DC_END_LIST));
        }

        out.extend(self.body);
        out.push(dc_to_sh(DC_END_LITERAL));
        Ok(out)
    }
}

#[derive(Debug, Clone)]
struct LinkedGlobal {
    name: String,
    key_type: Vec<DcShorthand>,
    init_number: Vec<DcShorthand>,
}

#[derive(Debug, Clone)]
struct LinkedRoutine {
    name: String,
    routine_tokens: Vec<DcShorthand>,
}

/// Module-level `.WASM` linker combining compiled StageL 3 routines, module
/// globals, and System-graph Stage 0 `.WASM` built-ins into a single
/// `Dc 1927 "Begin WASM program"`..`Dc 1928 "End WASM program"` document.
#[derive(Debug, Default)]
pub struct WasmModuleLinker {
    globals: Vec<LinkedGlobal>,
    global_by_name: HashMap<String, u32>,
    routines: Vec<LinkedRoutine>,
    routine_by_name: HashMap<String, u32>,
}

impl WasmModuleLinker {
    /// Creates an empty `.WASM` module linker.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers or reuses a mutable module global by `name`, returning its
    /// 0-based `globalidx`.
    pub fn ensure_global(
        &mut self,
        name: &str,
        vt: WasmValType,
        init_val: i64,
    ) -> Result<u32> {
        if let Some(&existing) = self.global_by_name.get(name) {
            return Ok(existing);
        }
        let idx = u32::try_from(self.globals.len())
            .context("WASM global index exceeds u32")?;
        self.globals.push(LinkedGlobal {
            name: name.to_owned(),
            key_type: vec![vt.to_shorthand()?],
            init_number: encode_i64_dc_shorthands(init_val),
        });
        self.global_by_name.insert(name.to_owned(), idx);
        Ok(idx)
    }

    /// Reserves a slot for a routine named `name`, returning its 0-based
    /// `funcidx` before its body is emitted (supporting mutual recursion).
    pub fn reserve_routine(&mut self, name: &str) -> Result<u32> {
        ensure!(
            !self.routine_by_name.contains_key(name),
            "Duplicate WASM routine export/member name '{name}'"
        );
        let idx = u32::try_from(self.routines.len())
            .context("WASM function index exceeds u32")?;
        self.routines.push(LinkedRoutine {
            name: name.to_owned(),
            routine_tokens: Vec::new(),
        });
        self.routine_by_name.insert(name.to_owned(), idx);
        Ok(idx)
    }

    /// Populates the `.WASM` routine tokens for a previously reserved `func_idx`.
    pub fn define_routine(
        &mut self,
        func_idx: u32,
        routine_tokens: Vec<DcShorthand>,
    ) -> Result<()> {
        let idx = usize::try_from(func_idx)
            .context("WASM function index exceeds usize")?;
        let slot = self
            .routines
            .get_mut(idx)
            .with_context(|| format!("Unreserved WASM func_idx {func_idx}"))?;
        ensure!(
            slot.routine_tokens.is_empty(),
            "WASM routine '{}' (func_idx {func_idx}) defined more than once",
            slot.name
        );
        slot.routine_tokens = routine_tokens;
        Ok(())
    }

    /// Looks up a previously registered routine by `name`.
    #[must_use]
    pub fn routine_index(&self, name: &str) -> Option<u32> {
        self.routine_by_name.get(name).copied()
    }

    /// Looks up a previously registered global by `name`.
    #[must_use]
    pub fn global_by_name_opt(&self, name: &str) -> Option<&u32> {
        self.global_by_name.get(name)
    }

    /// Ensures the shared linear-memory bump allocator (`$_alloc` and
    /// `$__heap_ptr`) from Stage 0 `Dc 2580` (`bufNew`) is linked into this
    /// module, returning `(alloc_func_idx, heap_ptr_global_idx)`.
    pub fn ensure_alloc_helper(&mut self) -> Result<(u32, u32)> {
        if let (Some(&alloc_idx), Some(&heap_idx)) = (
            self.routine_by_name.get("_alloc"),
            self.global_by_name.get("__heap_ptr"),
        ) {
            return Ok((alloc_idx, heap_idx));
        }
        let u8_dc = FormatId::U8
            .dc_char()
            .context("Missing DcChar for FormatId::U8")?;
        let _ = self.link_system_builtin(
            DC_STAGEL_BUF_NEW.to_u128(),
            &[DC_STAGEL_BUF_TYPE.to_u128(), u8_dc.to_u128()],
            "__builtin_2580_2577_f203",
        )?;
        let alloc_idx = *self
            .routine_by_name
            .get("_alloc")
            .context("Stage 0 bufNew WASM definition did not provide '_alloc'")?;
        let heap_idx = *self
            .global_by_name
            .get("__heap_ptr")
            .context("Stage 0 bufNew WASM definition did not provide '__heap_ptr'")?;
        Ok((alloc_idx, heap_idx))
    }

    /// Resolves and links a Stage 0 `.WASM` built-in from the System seed graph
    /// (`Dc 646` context lookup with trailing `Dc 275` wildcard fallback),
    /// deduplicating shared globals/helpers and relocating immediate indices.
    pub fn link_system_builtin(
        &mut self,
        dc_gid: u128,
        context_gids: &[u128],
        symbol_key: &str,
    ) -> Result<u32> {
        if let Some(&existing) = self.routine_by_name.get(symbol_key) {
            return Ok(existing);
        }
        let sg = SeedGraphIndex::shared()?;
        let target_sh = DcShorthand::from_global_id(dc_gid);
        let ctx_shs: Vec<DcShorthand> = context_gids
            .iter()
            .copied()
            .map(DcShorthand::from_global_id)
            .collect();
        let raw_tokens = sg
            .get_wasm_definition_tokens_in_context(target_sh, &ctx_shs)
            .with_context(|| {
                format!(
                    "Missing System-graph Stage 0 WASM built-in for Dc {dc_gid} in context {context_gids:?}"
                )
            })?;
        let expanded = expand_interned_strings_in_wasm(&raw_tokens, sg);
        let parsed = parse_system_wasm_unit(&expanded)?;

        let mut global_remap = Vec::with_capacity(parsed.globals.len());
        for g in parsed.globals {
            let new_g_idx = if let Some(&existing_g) =
                self.global_by_name.get(&g.name)
            {
                existing_g
            } else {
                let idx = u32::try_from(self.globals.len())
                    .context("WASM global index exceeds u32")?;
                self.global_by_name.insert(g.name.clone(), idx);
                self.globals.push(g);
                idx
            };
            global_remap.push(new_g_idx);
        }

        let mut func_remap = Vec::with_capacity(parsed.routines.len());
        let mut pending_defs: Vec<(u32, Vec<DcShorthand>)> = Vec::new();
        let mut main_func_idx: Option<u32> = None;

        for r in parsed.routines {
            let target_name = if r.name == "main" {
                symbol_key.to_owned()
            } else {
                r.name
            };
            let new_f_idx = if let Some(&existing_f) =
                self.routine_by_name.get(&target_name)
            {
                existing_f
            } else {
                let idx = self.reserve_routine(&target_name)?;
                pending_defs.push((idx, r.routine_tokens));
                idx
            };
            if target_name == symbol_key {
                main_func_idx = Some(new_f_idx);
            }
            func_remap.push(new_f_idx);
        }

        for (target_idx, raw_routine) in pending_defs {
            let relocated = relocate_routine_immediates(
                &raw_routine,
                &func_remap,
                &global_remap,
            )?;
            self.define_routine(target_idx, relocated)?;
        }

        main_func_idx.with_context(|| {
            format!("System-graph WASM built-in '{symbol_key}' had no 'main' routine")
        })
    }

    /// Emits the complete `Dc 1927`..`Dc 1928` `.WASM` shorthand sequence.
    pub fn finish_to_shorthands(self) -> Result<Vec<DcShorthand>> {
        let mut out = Vec::new();
        out.push(dc_to_sh(DC_BEGIN_WASM_PROGRAM));
        out.push(dc_to_sh(DC_BEGIN_KV_MAP));
        out.push(dc_to_sh(DC_TYPE_ANY));
        out.push(dc_to_sh(DC_TYPE_ANY));

        for g in self.globals {
            out.push(dc_to_sh(DC_BEGIN_KV_PAIR));
            push_identifier_shorthands(&g.name, &g.key_type, &mut out);
            out.push(dc_to_sh(DC_LIST_ELEMENT_SEPARATOR));
            out.extend(g.init_number);
            out.push(dc_to_sh(DC_END_KV_PAIR));
        }

        for r in self.routines {
            ensure!(
                !r.routine_tokens.is_empty(),
                "WASM routine '{}' was reserved but never defined",
                r.name
            );
            out.push(dc_to_sh(DC_BEGIN_KV_PAIR));
            push_identifier_shorthands(&r.name, &[], &mut out);
            out.push(dc_to_sh(DC_LIST_ELEMENT_SEPARATOR));
            out.extend(r.routine_tokens);
            out.push(dc_to_sh(DC_END_KV_PAIR));
        }

        out.push(dc_to_sh(DC_END_KV_MAP));
        out.push(dc_to_sh(DC_END_WASM_PROGRAM));
        Ok(out)
    }

    /// Emits the complete `Dc 1927`..`Dc 1928` `.WASM` document as a [`DcMstring`].
    pub fn finish_to_dcmstring(self) -> Result<DcMstring> {
        let shorthands = self.finish_to_shorthands()?;
        let mut mstr = DcMstring::new();
        for sh in shorthands {
            let gid = sh.to_global_id()?;
            mstr.push_char(DcChar::from_u128(gid));
        }
        Ok(mstr)
    }
}

struct ParsedWasmUnit {
    globals: Vec<LinkedGlobal>,
    routines: Vec<LinkedRoutine>,
}

fn expand_interned_strings_in_wasm(
    tokens: &[DcShorthand],
    sg: &SeedGraphIndex,
) -> Vec<DcShorthand> {
    let mut out = Vec::with_capacity(tokens.len());
    for &sh in tokens {
        if let DcShorthand::Long(_) = sh
            && let Some(s) = sg.get_string(sh)
        {
            for ch in s.chars() {
                out.push(DcShorthand::Unicode(u32::from(ch)));
            }
            continue;
        }
        out.push(sh);
    }
    out
}

fn parse_system_wasm_unit(tokens: &[DcShorthand]) -> Result<ParsedWasmUnit> {
    let sh_prog_open = dc_to_sh(DC_BEGIN_WASM_PROGRAM);
    let sh_prog_close = dc_to_sh(DC_END_WASM_PROGRAM);
    let sh_lit_open = dc_to_sh(DC_BEGIN_LITERAL);
    let sh_lit_close = dc_to_sh(DC_END_LITERAL);
    let sh_map_open = dc_to_sh(DC_BEGIN_KV_MAP);
    let sh_map_close = dc_to_sh(DC_END_KV_MAP);
    let sh_pair_open = dc_to_sh(DC_BEGIN_KV_PAIR);
    let sh_pair_close = dc_to_sh(DC_END_KV_PAIR);
    let sh_sep = dc_to_sh(DC_LIST_ELEMENT_SEPARATOR);

    ensure!(
        tokens.first() == Some(&sh_prog_open)
            && tokens.last() == Some(&sh_prog_close),
        "System WASM built-in must be framed by Dc 1927 .. Dc 1928"
    );
    let inner = tokens
        .get(1..tokens.len().saturating_sub(1))
        .context("Invalid System WASM built-in slice")?;

    // Standalone routine `1927 260 ... 261 1928`
    if inner.first() == Some(&sh_lit_open) {
        ensure!(
            inner.last() == Some(&sh_lit_close),
            "Standalone System WASM routine must end with Dc 261"
        );
        return Ok(ParsedWasmUnit {
            globals: Vec::new(),
            routines: vec![LinkedRoutine {
                name: String::from("main"),
                routine_tokens: inner.to_vec(),
            }],
        });
    }

    ensure!(
        inner.first() == Some(&sh_map_open)
            && inner.last() == Some(&sh_map_close),
        "Expected KV-map (Dc 559..560) or routine (Dc 260..261) inside System WASM built-in"
    );

    let mut pos = 1usize;
    // Optional map length number
    if inner.get(pos) == Some(&dc_to_sh(DC_BEGIN_NUMBER)) {
        pos = skip_balanced(
            inner,
            pos,
            dc_to_sh(DC_BEGIN_NUMBER),
            dc_to_sh(DC_END_NUMBER),
        )?;
    }
    // Skip keyType and valType headers of `Dc 559`
    for _ in 0..2 {
        pos = skip_type_expr(inner, pos)?;
    }

    let end_limit = inner.len().saturating_sub(1);
    let mut globals = Vec::new();
    let mut routines = Vec::new();

    while pos < end_limit {
        ensure!(
            inner.get(pos) == Some(&sh_pair_open),
            "Expected Dc 514 (Begin key-value pair) at index {pos} in System WASM map"
        );
        pos = pos.saturating_add(1);
        let (name, key_type, after_ident) = parse_identifier_at(inner, pos)?;
        pos = after_ident;
        ensure!(
            inner.get(pos) == Some(&sh_sep),
            "Expected Dc 312 after identifier '{name}' in System WASM map"
        );
        pos = pos.saturating_add(1);

        let val_first = *inner
            .get(pos)
            .context("Missing value in System WASM key-value pair")?;
        if val_first == sh_lit_open {
            let after_lit =
                skip_balanced(inner, pos, sh_lit_open, sh_lit_close)?;
            let r_slice = inner
                .get(pos..after_lit)
                .context("Invalid routine slice in System WASM map")?;
            routines.push(LinkedRoutine {
                name,
                routine_tokens: r_slice.to_vec(),
            });
            pos = after_lit;
        } else if val_first == dc_to_sh(DC_BEGIN_NUMBER) {
            let after_num = skip_balanced(
                inner,
                pos,
                dc_to_sh(DC_BEGIN_NUMBER),
                dc_to_sh(DC_END_NUMBER),
            )?;
            let num_slice = inner
                .get(pos..after_num)
                .context("Invalid global init number slice in System WASM map")?;
            globals.push(LinkedGlobal {
                name,
                key_type,
                init_number: num_slice.to_vec(),
            });
            pos = after_num;
        } else {
            bail!(
                "Unsupported member value token {val_first} for '{name}' in System WASM built-in"
            );
        }

        ensure!(
            inner.get(pos) == Some(&sh_pair_close),
            "Expected Dc 515 (End key-value pair) after '{pos}' in System WASM map"
        );
        pos = pos.saturating_add(1);
    }

    Ok(ParsedWasmUnit { globals, routines })
}

fn skip_balanced(
    tokens: &[DcShorthand],
    start: usize,
    opener: DcShorthand,
    closer: DcShorthand,
) -> Result<usize> {
    let sh_esc = dc_to_sh(DC_ESCAPE);
    ensure!(
        tokens.get(start) == Some(&opener),
        "Expected opener {opener} at index {start}"
    );
    let mut pos = start.saturating_add(1);
    let mut depth = 1usize;
    while depth > 0 {
        let tok = *tokens
            .get(pos)
            .with_context(|| format!("Unclosed {opener}..{closer} starting at {start}"))?;
        pos = pos.saturating_add(1);
        if tok == sh_esc {
            pos = pos.saturating_add(1);
            continue;
        }
        if tok == opener {
            depth = depth.saturating_add(1);
        } else if tok == closer {
            depth = depth.saturating_sub(1);
        }
    }
    Ok(pos)
}

fn skip_type_expr(tokens: &[DcShorthand], pos: usize) -> Result<usize> {
    let first = *tokens
        .get(pos)
        .with_context(|| format!("Expected type token at index {pos}"))?;
    if first == dc_to_sh(DC_BEGIN_TYPE_DEF) {
        skip_balanced(
            tokens,
            pos,
            dc_to_sh(DC_BEGIN_TYPE_DEF),
            dc_to_sh(DC_END_TYPE_DEF),
        )
    } else {
        Ok(pos.saturating_add(1))
    }
}

fn parse_identifier_at(
    tokens: &[DcShorthand],
    start: usize,
) -> Result<(String, Vec<DcShorthand>, usize)> {
    let sh_id_open = dc_to_sh(DC_BEGIN_IDENTIFIER);
    let sh_id_close = dc_to_sh(DC_END_IDENTIFIER);
    let sh_esc = dc_to_sh(DC_ESCAPE);
    ensure!(
        tokens.get(start) == Some(&sh_id_open),
        "Expected Dc 270 (Begin identifier) at index {start}"
    );
    let mut pos = start.saturating_add(1);
    let mut key_type = Vec::new();

    if let Some(&peek) = tokens.get(pos)
        && !matches!(peek, DcShorthand::Unicode(_))
        && peek != sh_esc
        && peek != sh_id_close
    {
        let after_ty = skip_type_expr(tokens, pos)?;
        let ty_slice = tokens
            .get(pos..after_ty)
            .context("Invalid identifier key_type slice")?;
        key_type.extend_from_slice(ty_slice);
        pos = after_ty;
    }

    let mut name = String::new();
    while let Some(&tok) = tokens.get(pos) {
        pos = pos.saturating_add(1);
        if tok == sh_id_close {
            return Ok((name, key_type, pos));
        }
        let ch_tok = if tok == sh_esc {
            let escaped = *tokens
                .get(pos)
                .context("Truncated escape inside WASM identifier")?;
            pos = pos.saturating_add(1);
            escaped
        } else {
            tok
        };
        let DcShorthand::Unicode(cp) = ch_tok else {
            bail!("Non-Unicode token {ch_tok} inside WASM identifier");
        };
        let c = char::from_u32(cp)
            .with_context(|| format!("Invalid Unicode codepoint U+{cp:04X}"))?;
        name.push(c);
    }
    bail!("Unclosed Dc 270 (Begin identifier) starting at {start}")
}

fn read_u32_number_at(
    tokens: &[DcShorthand],
    start: usize,
) -> Result<(u32, usize)> {
    let after_num = skip_balanced(
        tokens,
        start,
        dc_to_sh(DC_BEGIN_NUMBER),
        dc_to_sh(DC_END_NUMBER),
    )?;
    let slice = tokens
        .get(start..after_num)
        .context("Invalid Dc number slice")?;
    let mut gids = Vec::with_capacity(slice.len());
    for sh in slice {
        gids.push(sh.to_global_id()?);
    }
    let resolver = default_syntax_resolver();
    let (val_u128, consumed) =
        read_dc_number_u128_with_resolver(&gids, resolver.as_ref())?;
    ensure!(
        consumed == gids.len(),
        "Unconsumed tokens in WASM immediate number"
    );
    let val = u32::try_from(val_u128)
        .context("WASM immediate index exceeds u32 range")?;
    Ok((val, after_num))
}

fn copy_number_at(
    tokens: &[DcShorthand],
    start: usize,
    out: &mut Vec<DcShorthand>,
) -> Result<usize> {
    let after = skip_balanced(
        tokens,
        start,
        dc_to_sh(DC_BEGIN_NUMBER),
        dc_to_sh(DC_END_NUMBER),
    )?;
    let slice = tokens
        .get(start..after)
        .context("Invalid number slice while relocating WASM routine")?;
    out.extend_from_slice(slice);
    Ok(after)
}

/// Relocates `call` (`0x10`), `return_call` (`0x12`), `ref.func` (`0xD2`),
/// `global.get` (`0x23`), and `global.set` (`0x24`) immediate indices inside a
/// `.WASM` routine (`Dc 260`..`Dc 261`).
fn relocate_routine_immediates(
    routine_tokens: &[DcShorthand],
    func_remap: &[u32],
    global_remap: &[u32],
) -> Result<Vec<DcShorthand>> {
    let sh_lit_open = dc_to_sh(DC_BEGIN_LITERAL);
    let sh_lit_close = dc_to_sh(DC_END_LITERAL);
    ensure!(
        routine_tokens.first() == Some(&sh_lit_open)
            && routine_tokens.last() == Some(&sh_lit_close),
        "Expected Dc 260 .. Dc 261 framing for WASM routine"
    );

    let mut pos = 1usize;
    pos = skip_type_expr(routine_tokens, pos)?;
    if routine_tokens.get(pos) == Some(&dc_to_sh(DC_BEGIN_LIST)) {
        pos = skip_balanced(
            routine_tokens,
            pos,
            dc_to_sh(DC_BEGIN_LIST),
            dc_to_sh(DC_END_LIST),
        )?;
    }

    let mut out = Vec::with_capacity(routine_tokens.len());
    let header_slice = routine_tokens
        .get(..pos)
        .context("Invalid routine header slice")?;
    out.extend_from_slice(header_slice);

    let body_end = routine_tokens.len().saturating_sub(1);
    let start_00 = DC_WASM_START.to_u128();
    let start_fb = DC_WASM_FB_START.to_u128();
    let start_fc = DC_WASM_FC_START.to_u128();
    let start_fd0 = DC_WASM_FD_PAGE0_START.to_u128();

    while pos < body_end {
        let instr_sh = *routine_tokens
            .get(pos)
            .context("Unexpected end of routine body")?;
        out.push(instr_sh);
        pos = pos.saturating_add(1);
        let gid = instr_sh.to_global_id()?;

        if gid >= start_00 && gid < start_fb {
            let sub = u8::try_from(gid.saturating_sub(start_00))
                .context("Single-byte WASM opcode out of u8 range")?;
            match sub {
                // block, loop, if: 1 blocktype token/type-expr
                0x02..=0x04 => {
                    let after_bt = skip_type_expr(routine_tokens, pos)?;
                    let bt_slice = routine_tokens
                        .get(pos..after_bt)
                        .context("Invalid blocktype slice")?;
                    out.extend_from_slice(bt_slice);
                    pos = after_bt;
                }
                // call, return_call, ref.func -> remap funcidx
                0x10 | 0x12 | 0xD2 => {
                    let (old_idx, after_num) =
                        read_u32_number_at(routine_tokens, pos)?;
                    let old_usize = usize::try_from(old_idx)
                        .context("funcidx exceeds usize")?;
                    let &new_idx = func_remap.get(old_usize).with_context(
                        || format!("Out-of-bounds funcidx {old_idx} in built-in"),
                    )?;
                    out.extend(encode_u128_dc_shorthands(u128::from(new_idx)));
                    pos = after_num;
                }
                // global.get, global.set -> remap globalidx
                0x23 | 0x24 => {
                    let (old_idx, after_num) =
                        read_u32_number_at(routine_tokens, pos)?;
                    let old_usize = usize::try_from(old_idx)
                        .context("globalidx exceeds usize")?;
                    let &new_idx = global_remap.get(old_usize).with_context(
                        || {
                            format!(
                                "Out-of-bounds globalidx {old_idx} in built-in"
                            )
                        },
                    )?;
                    out.extend(encode_u128_dc_shorthands(u128::from(new_idx)));
                    pos = after_num;
                }
                // 1 numeric immediate: br, br_if, local.get/set/tee, memory.size/grow, *.const
                0x0C | 0x0D | 0x20..=0x22 | 0x3F | 0x40 | 0x41..=0x44 => {
                    pos = copy_number_at(routine_tokens, pos, &mut out)?;
                }
                // br_table: [list] + default [number]
                0x0E => {
                    let after_list = skip_balanced(
                        routine_tokens,
                        pos,
                        dc_to_sh(DC_BEGIN_LIST),
                        dc_to_sh(DC_END_LIST),
                    )?;
                    let list_slice = routine_tokens
                        .get(pos..after_list)
                        .context("Invalid br_table list slice")?;
                    out.extend_from_slice(list_slice);
                    pos = copy_number_at(routine_tokens, after_list, &mut out)?;
                }
                // call_indirect: 2 numeric immediates
                0x11 | 0x13 => {
                    pos = copy_number_at(routine_tokens, pos, &mut out)?;
                    pos = copy_number_at(routine_tokens, pos, &mut out)?;
                }
                // select_t: 1 valtype immediate
                0x1C => {
                    let after_vt = skip_type_expr(routine_tokens, pos)?;
                    let vt_slice = routine_tokens
                        .get(pos..after_vt)
                        .context("Invalid select_t valtype slice")?;
                    out.extend_from_slice(vt_slice);
                    pos = after_vt;
                }
                // memory load/store: 3 numeric immediates (memidx, align, offset)
                0x28..=0x3E => {
                    pos = copy_number_at(routine_tokens, pos, &mut out)?;
                    pos = copy_number_at(routine_tokens, pos, &mut out)?;
                    pos = copy_number_at(routine_tokens, pos, &mut out)?;
                }
                _ => {}
            }
        } else if gid >= start_fc && gid < start_fd0 {
            let sub = u32::try_from(gid.saturating_sub(start_fc))
                .context("0xFC WASM opcode out of u32 range")?;
            match sub {
                // memory.copy / table.copy: 2 numeric immediates
                10 | 14 => {
                    pos = copy_number_at(routine_tokens, pos, &mut out)?;
                    pos = copy_number_at(routine_tokens, pos, &mut out)?;
                }
                // memory.init: 2 numeric immediates; data.drop, memory.fill, elem.drop, table.grow/size/fill: 1
                8 | 12 => {
                    pos = copy_number_at(routine_tokens, pos, &mut out)?;
                    pos = copy_number_at(routine_tokens, pos, &mut out)?;
                }
                9 | 11 | 13 | 15..=17 => {
                    pos = copy_number_at(routine_tokens, pos, &mut out)?;
                }
                _ => {}
            }
        }
    }

    out.push(sh_lit_close);
    Ok(out)
}
