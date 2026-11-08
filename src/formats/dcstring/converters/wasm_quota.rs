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

//! Portable, runtime-independent WebAssembly compute-quota instrumentation and
//! shared task-level resource accounting.
//!
//! Instruments standard `.wasm` module binaries so that compute-quota deductions
//! (`deduct-before-effect`), linear-memory and table logical vs. physical growth
//! checks, and interruption ordinals execute identically on any compliant
//! WebAssembly runtime (`tinywasm`, V8/Node `WebAssembly`, Wasmtime, etc.)
//! without requiring a forked WASM engine.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::sync::{Arc, Mutex};

use anyhow::anyhow;
use ctb_formats_dcdata::tinywasm::types::{
    ConstInstruction, DataKind, Global, MemoryArch, TinyWasmModule,
};

use super::wasm::{DcWasmResourceLimits, WasmResourceInterruption};

/// Exported `i64` global holding the remaining portable compute quota units.
pub const CTB_QUOTA_REMAINING_EXPORT: &str = "__ctb_quota_remaining";
/// Exported `i64` global holding the compute quota units consumed in the current run.
pub const CTB_QUOTA_CONSUMED_EXPORT: &str = "__ctb_quota_consumed";
/// Exported `i32` global indicating interruption status:
/// `0 = none`, `1 = ComputeQuotaExhausted`, `2 = MemoryLimitExceeded`, `3 = TableLimitExceeded`.
pub const CTB_EXHAUSTED_EXPORT: &str = "__ctb_exhausted";
/// Exported `i64` global recording the 1-based ordinal of the interrupted operation
/// (or requested total pages/elements on physical growth exhaustion).
pub const CTB_INTERRUPTED_OP_EXPORT: &str = "__ctb_interrupted_op";
/// Exported `i64` global recording the compute cost required by the interrupted operation.
pub const CTB_REQUIRED_COST_EXPORT: &str = "__ctb_required_cost";
/// Exported `i64` global tracking the 1-based ordinal of metered operations executed.
pub const CTB_OP_ORDINAL_EXPORT: &str = "__ctb_op_ordinal";
/// Exported `i64` global tracking total allocated 64 KiB pages across all module memories.
pub const CTB_TOTAL_MEM_PAGES_EXPORT: &str = "__ctb_total_mem_pages";
/// Exported `i64` global holding the logical maximum 64 KiB pages (`max_memory_pages`).
pub const CTB_MAX_MEM_PAGES_EXPORT: &str = "__ctb_max_mem_pages";
/// Exported `i64` global holding the physical host allocation cap in 64 KiB pages.
pub const CTB_PHYS_MAX_MEM_PAGES_EXPORT: &str = "__ctb_phys_max_mem_pages";
/// Exported `i64` global tracking total allocated elements across all module tables.
pub const CTB_TOTAL_TABLE_ELEMS_EXPORT: &str = "__ctb_total_table_elems";
/// Exported `i64` global holding the logical maximum table elements (`max_table_elements`).
pub const CTB_MAX_TABLE_ELEMS_EXPORT: &str = "__ctb_max_table_elems";
/// Exported `i64` global holding the physical host allocation cap in table elements.
pub const CTB_PHYS_MAX_TABLE_ELEMS_EXPORT: &str = "__ctb_phys_max_table_elems";
/// Exported function name used when a module's `Start` section (`section 8`) is hoisted
/// for explicit metered execution.
pub const CTB_WASM_START_EXPORT: &str = "__ctb_wasm_start";

const WASM_HEADER: [u8; 8] = [0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];

const SEC_CUSTOM: u8 = 0;
const SEC_TYPE: u8 = 1;
const SEC_IMPORT: u8 = 2;
const SEC_FUNCTION: u8 = 3;
const SEC_TABLE: u8 = 4;
const SEC_MEMORY: u8 = 5;
const SEC_GLOBAL: u8 = 6;
const SEC_EXPORT: u8 = 7;
const SEC_START: u8 = 8;
const SEC_ELEMENT: u8 = 9;
const SEC_CODE: u8 = 10;
const SEC_DATA: u8 = 11;
const SEC_DATA_COUNT: u8 = 12;

const VALTYPE_I32: u8 = 0x7F;
const VALTYPE_I64: u8 = 0x7E;

/// Maximum table size declared in Section 4 so that passing `0x4000_0000`
/// (`1_073_741_824`) on logical denial deterministically exceeds the table's
/// declared maximum on every WebAssembly engine (`tinywasm`, V8, Wasmtime)
/// and returns `-1` after popping the reference operand.
const MAX_CLAMPED_DECLARED_TABLE_ELEMENTS: u32 = 10_000_000;

/// Logical denial sentinel delta passed to `table.grow` (`1_073_741_824`).
const TABLE_GROW_DENIAL_SENTINEL_DELTA: i32 = 0x4000_0000;

/// Metadata and instrumented `.wasm` bytes produced by [`instrument_wasm_module_bytes`].
#[derive(Debug, Clone)]
pub struct InstrumentedWasmModule {
    /// Standard WebAssembly binary with embedded portable compute-quota and
    /// logical/physical memory & table growth accounting.
    pub wasm_bytes: Vec<u8>,
    /// Validated `TinyWasmModule` parsed from [`Self::wasm_bytes`].
    pub parsed_instrumented: TinyWasmModule,
    /// Whether the original module contained a `Start` section (hoisted to
    /// [`CTB_WASM_START_EXPORT`]).
    pub has_start_func: bool,
    /// Sum of initial 64 KiB pages declared across all memories in the module.
    pub initial_memory_pages: usize,
    /// Sum of initial elements declared across all tables in the module.
    pub initial_table_elements: usize,
    /// Highest byte offset covered by active data segments on memory 0.
    pub static_data_end: usize,
    /// Number of guest-defined globals (before the 12 appended `__ctb_*` globals).
    pub num_guest_globals: usize,
    /// Number of linear memories declared in the module.
    pub num_memories: usize,
    /// Number of tables declared in the module.
    pub num_tables: usize,
}

/// Mutable state inside a [`SharedWasmTaskQuota`].
#[derive(Debug, Clone)]
pub struct SharedWasmTaskQuotaState {
    /// Remaining compute quota units available across the shared task.
    pub remaining_compute_quota: u64,
    /// Active compute quota limit (initial limit plus any granted increases).
    pub current_compute_quota_limit: u64,
    /// Total compute quota units executed across all sessions attached to this task.
    pub total_instructions_executed: u64,
    /// Cumulative output bytes emitted across all sessions attached to this task.
    pub cumulative_output_bytes: usize,
    /// Maximum cumulative output bytes allowed across the shared task.
    pub max_session_output_bytes: usize,
    /// Most recent resource interruption raised by any session attached to this task.
    pub last_resource_interruption: Option<WasmResourceInterruption>,
}

/// Shared task-level compute and output quota tracker used to ensure that
/// repeated or nested helper invocations within a single document evaluation task
/// deduct from one shared allowance rather than silently replenishing quotas.
#[derive(Debug, Clone)]
pub struct SharedWasmTaskQuota {
    inner: Arc<Mutex<SharedWasmTaskQuotaState>>,
}

impl SharedWasmTaskQuota {
    /// Creates a new [`SharedWasmTaskQuota`] initialized from `limits`.
    #[must_use]
    pub fn new(limits: DcWasmResourceLimits) -> Self {
        Self {
            inner: Arc::new(Mutex::new(SharedWasmTaskQuotaState {
                remaining_compute_quota: limits.max_instructions_per_step,
                current_compute_quota_limit: limits.max_instructions_per_step,
                total_instructions_executed: 0,
                cumulative_output_bytes: 0,
                max_session_output_bytes: limits.max_session_output_bytes,
                last_resource_interruption: None,
            })),
        }
    }

    /// Increases the shared compute quota by `additional_units`.
    ///
    /// # Errors
    /// Returns an error if the shared quota mutex is poisoned.
    pub fn increase_compute_quota(&self, additional_units: u64) -> Result<()> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| anyhow!("SharedWasmTaskQuota mutex poisoned"))?;
        guard.remaining_compute_quota = guard
            .remaining_compute_quota
            .saturating_add(additional_units);
        guard.current_compute_quota_limit = guard
            .current_compute_quota_limit
            .saturating_add(additional_units);
        Ok(())
    }

    /// Returns a snapshot of the current [`SharedWasmTaskQuotaState`].
    ///
    /// # Errors
    /// Returns an error if the shared quota mutex is poisoned.
    pub fn snapshot(&self) -> Result<SharedWasmTaskQuotaState> {
        let guard = self
            .inner
            .lock()
            .map_err(|_| anyhow!("SharedWasmTaskQuota mutex poisoned"))?;
        Ok(guard.clone())
    }

    pub(crate) fn with_lock<T>(
        &self,
        f: impl FnOnce(&mut SharedWasmTaskQuotaState) -> T,
    ) -> Result<T> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| anyhow!("SharedWasmTaskQuota mutex poisoned"))?;
        Ok(f(&mut guard))
    }
}

fn push_uleb_u32(mut val: u32, out: &mut Vec<u8>) {
    loop {
        #[allow(
            clippy::expect_used,
            reason = "Bitmask `val & 0x7F` restricts value to `0..=127`, which always fits in `u8`"
        )]
        let mut byte = u8::try_from(val & 0x7F)
            .expect("7-bit masked u32 always fits in u8");
        val >>= 7;
        if val != 0 {
            byte |= 0x80;
        }
        out.push(byte);
        if val == 0 {
            break;
        }
    }
}

fn push_sleb_i32(mut val: i32, out: &mut Vec<u8>) {
    loop {
        #[allow(
            clippy::expect_used,
            reason = "Bitmask `val & 0x7F` restricts value to `0..=127`, which always fits in `u8`"
        )]
        let byte = u8::try_from(val & 0x7F)
            .expect("7-bit masked i32 always fits in u8");
        val >>= 7;
        let done = (val == 0 && (byte & 0x40) == 0)
            || (val == -1 && (byte & 0x40) != 0);
        if done {
            out.push(byte);
            break;
        }
        out.push(byte | 0x80);
    }
}

fn push_sleb_i64(mut val: i64, out: &mut Vec<u8>) {
    loop {
        #[allow(
            clippy::expect_used,
            reason = "Bitmask `val & 0x7F` restricts value to `0..=127`, which always fits in `u8`"
        )]
        let byte = u8::try_from(val & 0x7F)
            .expect("7-bit masked i64 always fits in u8");
        val >>= 7;
        let done = (val == 0 && (byte & 0x40) == 0)
            || (val == -1 && (byte & 0x40) != 0);
        if done {
            out.push(byte);
            break;
        }
        out.push(byte | 0x80);
    }
}

fn read_byte(bytes: &[u8], pos: &mut usize) -> Result<u8> {
    let b = *bytes
        .get(*pos)
        .context("Unexpected EOF while reading WASM binary")?;
    *pos = pos.saturating_add(1);
    Ok(b)
}

fn read_uleb_u32(bytes: &[u8], pos: &mut usize) -> Result<u32> {
    let mut result = 0u32;
    let mut shift = 0u32;
    loop {
        let byte = read_byte(bytes, pos)?;
        let low = u32::from(byte & 0x7F);
        let shifted = low
            .checked_shl(shift)
            .context("ULEB128 shift overflow in WASM binary")?;
        result |= shifted;
        if (byte & 0x80) == 0 {
            return Ok(result);
        }
        shift = shift.saturating_add(7);
        ensure!(shift < 35, "Invalid u32 LEB128 encoding in WASM binary");
    }
}

fn skip_leb(bytes: &[u8], pos: &mut usize, max_bytes: usize) -> Result<()> {
    for _ in 0..max_bytes {
        let b = read_byte(bytes, pos)?;
        if (b & 0x80) == 0 {
            return Ok(());
        }
    }
    bail!("Overlong LEB128 immediate in WASM binary")
}

fn skip_blocktype(bytes: &[u8], pos: &mut usize) -> Result<()> {
    let b = read_byte(bytes, pos)?;
    if (b & 0x80) != 0 {
        // Multi-byte signed s33 type index
        skip_leb(bytes, pos, 4)?;
    } else if b == 0x63 || b == 0x64 {
        // Typed nullable/non-nullable reference `(ref null ht)` / `(ref ht)`
        skip_leb(bytes, pos, 5)?;
    }
    Ok(())
}

fn skip_memarg(bytes: &[u8], pos: &mut usize) -> Result<()> {
    let align_flags = read_uleb_u32(bytes, pos)?;
    if (align_flags & 0x40) != 0 {
        let _mem_idx = read_uleb_u32(bytes, pos)?;
    }
    skip_leb(bytes, pos, 10)
}

fn push_section(id: u8, payload: &[u8], out: &mut Vec<u8>) -> Result<()> {
    out.push(id);
    push_uleb_u32(
        u32::try_from(payload.len())
            .context("Instrumented WASM section exceeds u32 max length")?,
        out,
    );
    out.extend_from_slice(payload);
    Ok(())
}

fn push_name(name: &str, out: &mut Vec<u8>) -> Result<()> {
    push_uleb_u32(
        u32::try_from(name.len())
            .context("WASM export name exceeds u32 max length")?,
        out,
    );
    out.extend_from_slice(name.as_bytes());
    Ok(())
}

fn eval_const_offset(
    instr: &ConstInstruction,
    globals: &[Global],
) -> Option<usize> {
    match instr {
        ConstInstruction::I32Const(v) => {
            usize::try_from(u32::from_le_bytes(v.to_le_bytes())).ok()
        }
        ConstInstruction::I64Const(v) => {
            usize::try_from(u64::from_le_bytes(v.to_le_bytes())).ok()
        }
        ConstInstruction::GlobalGet(idx) => {
            let g_idx = usize::try_from(*idx).ok()?;
            let g = globals.get(g_idx)?;
            eval_const_offset(&g.init, globals)
        }
        _ => None,
    }
}

fn compute_static_data_end(parsed: &TinyWasmModule) -> usize {
    let mut max_end = 0usize;
    for seg in &parsed.data {
        if let DataKind::Active { mem: 0, offset } = &seg.kind
            && !seg.data.is_empty()
            && let Some(start) = eval_const_offset(offset, &parsed.globals)
        {
            let end = start.saturating_add(seg.data.len());
            max_end = max_end.max(end);
        }
    }
    max_end
}

#[derive(Clone, Copy)]
struct QuotaIndices {
    deduct_func_idx: u32,
    g_quota_rem: u32,
    g_quota_consumed: u32,
    g_exhausted: u32,
    g_interrupted_op: u32,
    g_required_cost: u32,
    g_op_ordinal: u32,
    g_total_mem_pages: u32,
    g_max_mem_pages: u32,
    g_phys_max_mem_pages: u32,
    g_total_table_elems: u32,
    g_max_table_elems: u32,
    g_phys_max_table_elems: u32,
}

fn emit_const_deduct(cost: i64, indices: &QuotaIndices, out: &mut Vec<u8>) {
    out.push(0x42); // i64.const
    push_sleb_i64(cost, out);
    out.push(0x10); // call $__ctb_deduct
    push_uleb_u32(indices.deduct_func_idx, out);
}

fn emit_bulk_deduct(
    scratch_delta_local: u32,
    indices: &QuotaIndices,
    out: &mut Vec<u8>,
) {
    // Operand stack ends with `[..., n: i32]`.
    // `local.tee $scratch_delta` keeps `n` on the stack for the bulk op;
    // `local.get $scratch_delta` pushes a second copy to compute `1 + u64(n)`.
    out.push(0x22); // local.tee
    push_uleb_u32(scratch_delta_local, out);
    out.push(0x20); // local.get
    push_uleb_u32(scratch_delta_local, out);
    out.push(0xAD); // i64.extend_i32_u
    out.push(0x42); // i64.const 1
    push_sleb_i64(1, out);
    out.push(0x7C); // i64.add
    out.push(0x10); // call $__ctb_deduct
    push_uleb_u32(indices.deduct_func_idx, out);
}

fn emit_memory_grow_sequence(
    mem_idx: u32,
    declared_max_pages: i64,
    scratch_delta_local: u32,
    scratch_res_local: u32,
    indices: &QuotaIndices,
    out: &mut Vec<u8>,
) {
    // Pop `delta: i32` into `$scratch_delta`
    out.push(0x21); // local.set
    push_uleb_u32(scratch_delta_local, out);

    // Condition 1: `delta < 0`
    out.push(0x20); // local.get $scratch_delta
    push_uleb_u32(scratch_delta_local, out);
    out.push(0x41); // i32.const 0
    push_sleb_i32(0, out);
    out.push(0x48); // i32.lt_s

    // Condition 2: `total_mem_pages + u64(delta) > max_mem_pages`
    out.push(0x23); // global.get $g_total_mem_pages
    push_uleb_u32(indices.g_total_mem_pages, out);
    out.push(0x20); // local.get $scratch_delta
    push_uleb_u32(scratch_delta_local, out);
    out.push(0xAD); // i64.extend_i32_u
    out.push(0x7C); // i64.add
    out.push(0x23); // global.get $g_max_mem_pages
    push_uleb_u32(indices.g_max_mem_pages, out);
    out.push(0x56); // i64.gt_u

    // Condition 3: `memory.size(mem_idx) + u64(delta) > declared_max_pages`
    out.push(0x3F); // memory.size
    push_uleb_u32(mem_idx, out);
    out.push(0xAD); // i64.extend_i32_u
    out.push(0x20); // local.get $scratch_delta
    push_uleb_u32(scratch_delta_local, out);
    out.push(0xAD); // i64.extend_i32_u
    out.push(0x7C); // i64.add
    out.push(0x42); // i64.const declared_max_pages
    push_sleb_i64(declared_max_pages, out);
    out.push(0x56); // i64.gt_u

    out.push(0x72); // i32.or
    out.push(0x72); // i32.or

    // `if (result i32)`
    out.push(0x04);
    out.push(VALTYPE_I32);
    // Logically denied: deduct 1 unit, push -1
    emit_const_deduct(1, indices, out);
    out.push(0x41); // i32.const -1
    push_sleb_i32(-1, out);
    out.push(0x05); // else (logically allowed)
    // Deduct `1 + 64 * u64(delta)` before effect
    out.push(0x20); // local.get $scratch_delta
    push_uleb_u32(scratch_delta_local, out);
    out.push(0xAD); // i64.extend_i32_u
    out.push(0x42); // i64.const 64
    push_sleb_i64(64, out);
    out.push(0x7E); // i64.mul
    out.push(0x42); // i64.const 1
    push_sleb_i64(1, out);
    out.push(0x7C); // i64.add
    out.push(0x10); // call $__ctb_deduct
    push_uleb_u32(indices.deduct_func_idx, out);

    // Check physical host allocation cap: `total_mem_pages + u64(delta) > phys_max_mem_pages`
    out.push(0x23); // global.get $g_total_mem_pages
    push_uleb_u32(indices.g_total_mem_pages, out);
    out.push(0x20); // local.get $scratch_delta
    push_uleb_u32(scratch_delta_local, out);
    out.push(0xAD); // i64.extend_i32_u
    out.push(0x7C); // i64.add
    out.push(0x23); // global.get $g_phys_max_mem_pages
    push_uleb_u32(indices.g_phys_max_mem_pages, out);
    out.push(0x56); // i64.gt_u
    out.push(0x04); // if (empty)
    out.push(0x40);
    out.push(0x41); // i32.const 2 (MemoryLimitExceeded)
    push_sleb_i32(2, out);
    out.push(0x24); // global.set $g_exhausted
    push_uleb_u32(indices.g_exhausted, out);
    out.push(0x23); // global.get $g_total_mem_pages
    push_uleb_u32(indices.g_total_mem_pages, out);
    out.push(0x20); // local.get $scratch_delta
    push_uleb_u32(scratch_delta_local, out);
    out.push(0xAD); // i64.extend_i32_u
    out.push(0x7C); // i64.add
    out.push(0x24); // global.set $g_interrupted_op (records requested_pages)
    push_uleb_u32(indices.g_interrupted_op, out);
    out.push(0x00); // unreachable
    out.push(0x0B); // end if

    // Execute underlying `memory.grow $mem_idx`
    out.push(0x20); // local.get $scratch_delta
    push_uleb_u32(scratch_delta_local, out);
    out.push(0x40); // memory.grow
    push_uleb_u32(mem_idx, out);
    out.push(0x22); // local.tee $scratch_res
    push_uleb_u32(scratch_res_local, out);
    out.push(0x41); // i32.const 0
    push_sleb_i32(0, out);
    out.push(0x48); // i32.lt_s (engine physical allocation failed despite logical allow)
    out.push(0x04); // if (empty)
    out.push(0x40);
    out.push(0x41); // i32.const 2 (MemoryLimitExceeded)
    push_sleb_i32(2, out);
    out.push(0x24); // global.set $g_exhausted
    push_uleb_u32(indices.g_exhausted, out);
    out.push(0x23); // global.get $g_total_mem_pages
    push_uleb_u32(indices.g_total_mem_pages, out);
    out.push(0x20); // local.get $scratch_delta
    push_uleb_u32(scratch_delta_local, out);
    out.push(0xAD); // i64.extend_i32_u
    out.push(0x7C); // i64.add
    out.push(0x24); // global.set $g_interrupted_op
    push_uleb_u32(indices.g_interrupted_op, out);
    out.push(0x00); // unreachable
    out.push(0x0B); // end if

    // Update `$g_total_mem_pages += u64(delta)` and push `$scratch_res` (`old_pages`)
    out.push(0x23); // global.get $g_total_mem_pages
    push_uleb_u32(indices.g_total_mem_pages, out);
    out.push(0x20); // local.get $scratch_delta
    push_uleb_u32(scratch_delta_local, out);
    out.push(0xAD); // i64.extend_i32_u
    out.push(0x7C); // i64.add
    out.push(0x24); // global.set $g_total_mem_pages
    push_uleb_u32(indices.g_total_mem_pages, out);
    out.push(0x20); // local.get $scratch_res
    push_uleb_u32(scratch_res_local, out);
    out.push(0x0B); // end outer if (result i32)
}

fn emit_table_grow_sequence(
    table_idx: u32,
    declared_max_elems: i64,
    scratch_delta_local: u32,
    scratch_res_local: u32,
    indices: &QuotaIndices,
    out: &mut Vec<u8>,
) {
    // Operand stack has `[..., init_val: ref, delta: i32]`.
    // Pop `delta` into `$scratch_delta`, leaving `init_val: ref` on top of the stack.
    out.push(0x21); // local.set $scratch_delta
    push_uleb_u32(scratch_delta_local, out);

    // Condition 1: `delta < 0`
    out.push(0x20); // local.get $scratch_delta
    push_uleb_u32(scratch_delta_local, out);
    out.push(0x41); // i32.const 0
    push_sleb_i32(0, out);
    out.push(0x48); // i32.lt_s

    // Condition 2: `total_table_elems + u64(delta) > max_table_elems`
    out.push(0x23); // global.get $g_total_table_elems
    push_uleb_u32(indices.g_total_table_elems, out);
    out.push(0x20); // local.get $scratch_delta
    push_uleb_u32(scratch_delta_local, out);
    out.push(0xAD); // i64.extend_i32_u
    out.push(0x7C); // i64.add
    out.push(0x23); // global.get $g_max_table_elems
    push_uleb_u32(indices.g_max_table_elems, out);
    out.push(0x56); // i64.gt_u

    // Condition 3: `table.size(table_idx) + u64(delta) > declared_max_elems`
    out.push(0xFC); // table.size
    push_uleb_u32(16, out);
    push_uleb_u32(table_idx, out);
    out.push(0xAD); // i64.extend_i32_u
    out.push(0x20); // local.get $scratch_delta
    push_uleb_u32(scratch_delta_local, out);
    out.push(0xAD); // i64.extend_i32_u
    out.push(0x7C); // i64.add
    out.push(0x42); // i64.const declared_max_elems
    push_sleb_i64(declared_max_elems, out);
    out.push(0x56); // i64.gt_u

    out.push(0x72); // i32.or
    out.push(0x72); // i32.or

    // `if (result i32)` computes the effective delta to pass to `table.grow`
    out.push(0x04);
    out.push(VALTYPE_I32);
    // Logically denied: deduct 1 unit, mark `$scratch_delta = -1`, push sentinel delta
    emit_const_deduct(1, indices, out);
    out.push(0x41); // i32.const -1
    push_sleb_i32(-1, out);
    out.push(0x21); // local.set $scratch_delta
    push_uleb_u32(scratch_delta_local, out);
    out.push(0x41); // i32.const TABLE_GROW_DENIAL_SENTINEL_DELTA
    push_sleb_i32(TABLE_GROW_DENIAL_SENTINEL_DELTA, out);
    out.push(0x05); // else (logically allowed)
    // Deduct `1 + u64(delta)` before effect
    out.push(0x20); // local.get $scratch_delta
    push_uleb_u32(scratch_delta_local, out);
    out.push(0xAD); // i64.extend_i32_u
    out.push(0x42); // i64.const 1
    push_sleb_i64(1, out);
    out.push(0x7C); // i64.add
    out.push(0x10); // call $__ctb_deduct
    push_uleb_u32(indices.deduct_func_idx, out);

    // Check physical host allocation cap: `total_table_elems + u64(delta) > phys_max_table_elems`
    out.push(0x23); // global.get $g_total_table_elems
    push_uleb_u32(indices.g_total_table_elems, out);
    out.push(0x20); // local.get $scratch_delta
    push_uleb_u32(scratch_delta_local, out);
    out.push(0xAD); // i64.extend_i32_u
    out.push(0x7C); // i64.add
    out.push(0x23); // global.get $g_phys_max_table_elems
    push_uleb_u32(indices.g_phys_max_table_elems, out);
    out.push(0x56); // i64.gt_u
    out.push(0x04); // if (empty)
    out.push(0x40);
    out.push(0x41); // i32.const 3 (TableLimitExceeded)
    push_sleb_i32(3, out);
    out.push(0x24); // global.set $g_exhausted
    push_uleb_u32(indices.g_exhausted, out);
    out.push(0x23); // global.get $g_total_table_elems
    push_uleb_u32(indices.g_total_table_elems, out);
    out.push(0x20); // local.get $scratch_delta
    push_uleb_u32(scratch_delta_local, out);
    out.push(0xAD); // i64.extend_i32_u
    out.push(0x7C); // i64.add
    out.push(0x24); // global.set $g_interrupted_op (records requested_elements)
    push_uleb_u32(indices.g_interrupted_op, out);
    out.push(0x00); // unreachable
    out.push(0x0B); // end if

    out.push(0x20); // local.get $scratch_delta
    push_uleb_u32(scratch_delta_local, out);
    out.push(0x0B); // end if (result i32)

    // Stack now has `[..., init_val: ref, effective_delta: i32]`.
    // Execute `table.grow $table_idx`
    out.push(0xFC);
    push_uleb_u32(15, out);
    push_uleb_u32(table_idx, out);
    out.push(0x22); // local.tee $scratch_res
    push_uleb_u32(scratch_res_local, out);

    // If `$scratch_delta >= 0` (logically allowed branch was taken):
    out.push(0x20); // local.get $scratch_delta
    push_uleb_u32(scratch_delta_local, out);
    out.push(0x41); // i32.const 0
    push_sleb_i32(0, out);
    out.push(0x4E); // i32.ge_s
    out.push(0x04); // if (empty)
    out.push(0x40);
    // If `$scratch_res < 0`, underlying engine failed physical table growth
    out.push(0x20); // local.get $scratch_res
    push_uleb_u32(scratch_res_local, out);
    out.push(0x41); // i32.const 0
    push_sleb_i32(0, out);
    out.push(0x48); // i32.lt_s
    out.push(0x04); // if (empty)
    out.push(0x40);
    out.push(0x41); // i32.const 3 (TableLimitExceeded)
    push_sleb_i32(3, out);
    out.push(0x24); // global.set $g_exhausted
    push_uleb_u32(indices.g_exhausted, out);
    out.push(0x23); // global.get $g_total_table_elems
    push_uleb_u32(indices.g_total_table_elems, out);
    out.push(0x20); // local.get $scratch_delta
    push_uleb_u32(scratch_delta_local, out);
    out.push(0xAD); // i64.extend_i32_u
    out.push(0x7C); // i64.add
    out.push(0x24); // global.set $g_interrupted_op
    push_uleb_u32(indices.g_interrupted_op, out);
    out.push(0x00); // unreachable
    out.push(0x0B); // end if

    // Update `$g_total_table_elems += u64(delta)`
    out.push(0x23); // global.get $g_total_table_elems
    push_uleb_u32(indices.g_total_table_elems, out);
    out.push(0x20); // local.get $scratch_delta
    push_uleb_u32(scratch_delta_local, out);
    out.push(0xAD); // i64.extend_i32_u
    out.push(0x7C); // i64.add
    out.push(0x24); // global.set $g_total_table_elems
    push_uleb_u32(indices.g_total_table_elems, out);
    out.push(0x0B); // end if
}

#[allow(
    clippy::too_many_lines,
    reason = "Single-pass WebAssembly bytecode instruction scanner and quota instrumenter"
)]
fn instrument_function_body(
    body_bytes: &[u8],
    num_params: u32,
    mem_declared_max: &[i64],
    table_declared_max: &[i64],
    indices: &QuotaIndices,
) -> Result<Vec<u8>> {
    let mut pos = 0usize;
    let num_local_runs = read_uleb_u32(body_bytes, &mut pos)?;
    let mut local_runs: Vec<(u32, u8)> =
        Vec::with_capacity(usize::try_from(num_local_runs)?.saturating_add(1));
    let mut total_locals = num_params;
    for _ in 0..num_local_runs {
        let count = read_uleb_u32(body_bytes, &mut pos)?;
        let vt = read_byte(body_bytes, &mut pos)?;
        total_locals = total_locals
            .checked_add(count)
            .context("WASM function local count overflow")?;
        local_runs.push((count, vt));
    }
    let scratch_delta_local = total_locals;
    let scratch_res_local = total_locals
        .checked_add(1)
        .context("WASM scratch local index overflow")?;
    local_runs.push((2, VALTYPE_I32));

    let mut out = Vec::with_capacity(body_bytes.len().saturating_mul(3));
    push_uleb_u32(u32::try_from(local_runs.len())?, &mut out);
    for (count, vt) in local_runs {
        push_uleb_u32(count, &mut out);
        out.push(vt);
    }

    // Deduct 1 compute unit on function entry
    emit_const_deduct(1, indices, &mut out);

    while pos < body_bytes.len() {
        let instr_start = pos;
        let opcode = read_byte(body_bytes, &mut pos)?;

        // Final `end` of the function body
        if opcode == 0x0B && pos == body_bytes.len() {
            out.push(0x0B);
            break;
        }

        // Structural delimiters `else` (0x05) and `end` (0x0B)
        if opcode == 0x05 || opcode == 0x0B {
            out.push(opcode);
            continue;
        }

        // `loop <blocktype>` (0x03): emit loop header, then deduct 1 unit at loop body entry
        if opcode == 0x03 {
            skip_blocktype(body_bytes, &mut pos)?;
            if let Some(slice) = body_bytes.get(instr_start..pos) {
                out.extend_from_slice(slice);
            }
            emit_const_deduct(1, indices, &mut out);
            continue;
        }

        // `memory.grow $mem_idx` (0x40)
        if opcode == 0x40 {
            let mem_idx = read_uleb_u32(body_bytes, &mut pos)?;
            let declared_max = *mem_declared_max
                .get(usize::try_from(mem_idx)?)
                .context(
                    "WASM memory.grow references out-of-bounds memory index",
                )?;
            emit_memory_grow_sequence(
                mem_idx,
                declared_max,
                scratch_delta_local,
                scratch_res_local,
                indices,
                &mut out,
            );
            continue;
        }

        // `0xFC` prefix instructions
        if opcode == 0xFC {
            let subop = read_uleb_u32(body_bytes, &mut pos)?;
            match subop {
                0..=7 => {
                    emit_const_deduct(1, indices, &mut out);
                    if let Some(slice) = body_bytes.get(instr_start..pos) {
                        out.extend_from_slice(slice);
                    }
                }
                8 | 10 | 12 | 14 => {
                    // `memory.init` (8), `memory.copy` (10), `table.init` (12), `table.copy` (14)
                    let _idx1 = read_uleb_u32(body_bytes, &mut pos)?;
                    let _idx2 = read_uleb_u32(body_bytes, &mut pos)?;
                    emit_bulk_deduct(scratch_delta_local, indices, &mut out);
                    if let Some(slice) = body_bytes.get(instr_start..pos) {
                        out.extend_from_slice(slice);
                    }
                }
                11 | 17 => {
                    // `memory.fill` (11), `table.fill` (17)
                    let _idx = read_uleb_u32(body_bytes, &mut pos)?;
                    emit_bulk_deduct(scratch_delta_local, indices, &mut out);
                    if let Some(slice) = body_bytes.get(instr_start..pos) {
                        out.extend_from_slice(slice);
                    }
                }
                9 | 13 | 16 => {
                    // `data.drop` (9), `elem.drop` (13), `table.size` (16)
                    let _idx = read_uleb_u32(body_bytes, &mut pos)?;
                    emit_const_deduct(1, indices, &mut out);
                    if let Some(slice) = body_bytes.get(instr_start..pos) {
                        out.extend_from_slice(slice);
                    }
                }
                15 => {
                    // `table.grow $table_idx`
                    let table_idx = read_uleb_u32(body_bytes, &mut pos)?;
                    let declared_max = *table_declared_max
                        .get(usize::try_from(table_idx)?)
                        .context("WASM table.grow references out-of-bounds table index")?;
                    emit_table_grow_sequence(
                        table_idx,
                        declared_max,
                        scratch_delta_local,
                        scratch_res_local,
                        indices,
                        &mut out,
                    );
                }
                other => {
                    bail!("Unsupported 0xFC subopcode {other} in WASM module")
                }
            }
            continue;
        }

        // `0xFD` SIMD prefix instructions
        if opcode == 0xFD {
            let subop = read_uleb_u32(body_bytes, &mut pos)?;
            match subop {
                0..=11 | 92..=93 => {
                    skip_memarg(body_bytes, &mut pos)?;
                }
                12 | 13 => {
                    pos = pos.saturating_add(16);
                    ensure!(
                        pos <= body_bytes.len(),
                        "Truncated 16-byte SIMD immediate in WASM module"
                    );
                }
                21..=34 => {
                    let _lane = read_byte(body_bytes, &mut pos)?;
                }
                84..=91 => {
                    skip_memarg(body_bytes, &mut pos)?;
                    let _lane = read_byte(body_bytes, &mut pos)?;
                }
                _ => {}
            }
            emit_const_deduct(1, indices, &mut out);
            if let Some(slice) = body_bytes.get(instr_start..pos) {
                out.extend_from_slice(slice);
            }
            continue;
        }

        // Standard single-byte instructions (cost = 1 before effect)
        match opcode {
            0x00 | 0x01 | 0x0F | 0x1A | 0x1B | 0xD1 | 0xD3 | 0xD4 => {}
            0x02 | 0x04 => {
                skip_blocktype(body_bytes, &mut pos)?;
            }
            0x0C | 0x0D | 0x10 | 0x12 | 0x14 | 0x15 | 0x20..=0x26 | 0x3F
            | 0xD2 | 0xD5 | 0xD6 => {
                let _idx = read_uleb_u32(body_bytes, &mut pos)?;
            }
            0x11 | 0x13 => {
                let _type_idx = read_uleb_u32(body_bytes, &mut pos)?;
                let _table_idx = read_uleb_u32(body_bytes, &mut pos)?;
            }
            0x0E => {
                let count = read_uleb_u32(body_bytes, &mut pos)?;
                for _ in 0..count {
                    let _lbl = read_uleb_u32(body_bytes, &mut pos)?;
                }
                let _default_lbl = read_uleb_u32(body_bytes, &mut pos)?;
            }
            0x1C => {
                let count = read_uleb_u32(body_bytes, &mut pos)?;
                for _ in 0..count {
                    skip_blocktype(body_bytes, &mut pos)?;
                }
            }
            0x28..=0x3E => {
                skip_memarg(body_bytes, &mut pos)?;
            }
            0x41 => {
                skip_leb(body_bytes, &mut pos, 5)?;
            }
            0x42 => {
                skip_leb(body_bytes, &mut pos, 10)?;
            }
            0x43 => {
                pos = pos.saturating_add(4);
                ensure!(
                    pos <= body_bytes.len(),
                    "Truncated f32.const immediate in WASM module"
                );
            }
            0x44 => {
                pos = pos.saturating_add(8);
                ensure!(
                    pos <= body_bytes.len(),
                    "Truncated f64.const immediate in WASM module"
                );
            }
            0x45..=0xC4 => {}
            0xD0 => {
                skip_leb(body_bytes, &mut pos, 5)?;
            }
            other => {
                bail!("Unsupported WASM opcode {other:#04x} during quota instrumentation");
            }
        }

        emit_const_deduct(1, indices, &mut out);
        if let Some(slice) = body_bytes.get(instr_start..pos) {
            out.extend_from_slice(slice);
        }
    }

    Ok(out)
}

fn encode_deduct_helper_body(indices: &QuotaIndices) -> Vec<u8> {
    let mut body = Vec::with_capacity(64);
    body.push(0x00); // 0 local runs (1 param: `$cost: i64` at local 0)

    // `__ctb_op_ordinal = __ctb_op_ordinal + 1`
    body.push(0x23); // global.get $g_op_ordinal
    push_uleb_u32(indices.g_op_ordinal, &mut body);
    body.push(0x42); // i64.const 1
    push_sleb_i64(1, &mut body);
    body.push(0x7C); // i64.add
    body.push(0x24); // global.set $g_op_ordinal
    push_uleb_u32(indices.g_op_ordinal, &mut body);

    // `if (i64.lt_u(global.get $g_quota_rem, local.get 0))`
    body.push(0x23); // global.get $g_quota_rem
    push_uleb_u32(indices.g_quota_rem, &mut body);
    body.push(0x20); // local.get 0 ($cost)
    push_uleb_u32(0, &mut body);
    body.push(0x54); // i64.lt_u
    body.push(0x04); // if (empty)
    body.push(0x40);
    body.push(0x41); // i32.const 1 (ComputeQuotaExhausted)
    push_sleb_i32(1, &mut body);
    body.push(0x24); // global.set $g_exhausted
    push_uleb_u32(indices.g_exhausted, &mut body);
    body.push(0x23); // global.get $g_op_ordinal
    push_uleb_u32(indices.g_op_ordinal, &mut body);
    body.push(0x24); // global.set $g_interrupted_op
    push_uleb_u32(indices.g_interrupted_op, &mut body);
    body.push(0x20); // local.get 0 ($cost)
    push_uleb_u32(0, &mut body);
    body.push(0x24); // global.set $g_required_cost
    push_uleb_u32(indices.g_required_cost, &mut body);
    body.push(0x00); // unreachable
    body.push(0x0B); // end if

    // `__ctb_quota_remaining -= $cost`
    body.push(0x23); // global.get $g_quota_rem
    push_uleb_u32(indices.g_quota_rem, &mut body);
    body.push(0x20); // local.get 0 ($cost)
    push_uleb_u32(0, &mut body);
    body.push(0x7D); // i64.sub
    body.push(0x24); // global.set $g_quota_rem
    push_uleb_u32(indices.g_quota_rem, &mut body);

    // `__ctb_quota_consumed += $cost`
    body.push(0x23); // global.get $g_quota_consumed
    push_uleb_u32(indices.g_quota_consumed, &mut body);
    body.push(0x20); // local.get 0 ($cost)
    push_uleb_u32(0, &mut body);
    body.push(0x7C); // i64.add
    body.push(0x24); // global.set $g_quota_consumed
    push_uleb_u32(indices.g_quota_consumed, &mut body);

    body.push(0x0B); // end function
    body
}

/// Validates and instruments a standard WebAssembly module binary with portable
/// `deduct-before-effect` compute-quota accounting and logical/physical memory
/// and table growth guards.
///
/// # Errors
/// Returns an error if `wasm_bytes` is not a valid WebAssembly module, declares
/// host imports or `memory64`, declares any reserved `"__ctb_*"` export, or uses
/// unsupported opcodes.
#[allow(
    clippy::too_many_lines,
    reason = "Complete binary WebAssembly section parser and instrumenter"
)]
pub fn instrument_wasm_module_bytes(
    wasm_bytes: &[u8],
) -> Result<InstrumentedWasmModule> {
    let parsed_uninstrumented =
        ctb_formats_dcdata::tinywasm::parser::Parser::new()
            .parse_module_bytes(wasm_bytes)
            .map_err(|error| {
                anyhow!(
                    "Failed to parse WebAssembly module in tinywasm: {error:?}"
                )
            })?;
    ensure!(
        parsed_uninstrumented.imports.is_empty(),
        "WebAssembly host imports are not supported"
    );
    ensure!(
        parsed_uninstrumented
            .memory_types
            .iter()
            .all(|memory| memory.arch == MemoryArch::I32),
        "WebAssembly memory64 is not supported"
    );
    ensure!(
        parsed_uninstrumented
            .exports
            .iter()
            .all(|exp| !exp.name.starts_with("__ctb_")),
        "WebAssembly module must not declare reserved '__ctb_*' exports"
    );

    let initial_memory_pages = parsed_uninstrumented
        .memory_types
        .iter()
        .try_fold(0usize, |total: usize, memory| -> Result<usize> {
            total
                .checked_add(usize::try_from(memory.page_count_initial)?)
                .context("Initial WASM memory page count overflow")
        })?;
    let initial_table_elements = parsed_uninstrumented
        .table_types
        .iter()
        .try_fold(0usize, |total: usize, table| -> Result<usize> {
            total
                .checked_add(usize::try_from(table.size_initial)?)
                .context("Initial WASM table element count overflow")
        })?;
    let static_data_end = compute_static_data_end(&parsed_uninstrumented);
    let num_guest_globals = parsed_uninstrumented.globals.len();
    let num_memories = parsed_uninstrumented.memory_types.len();
    let num_tables = parsed_uninstrumented.table_types.len();

    let mem_declared_max: Vec<i64> = parsed_uninstrumented
        .memory_types
        .iter()
        .map(|m| -> Result<i64> {
            // Reason for fallback: in the WebAssembly specification, a 32-bit memory without an
            // explicit maximum (`None`) has an architectural maximum of 65,536 pages (4 GiB).
            let max_pages = m.page_count_max.unwrap_or(65_536);
            Ok(i64::try_from(max_pages)?)
        })
        .collect::<Result<Vec<i64>>>()?;
    let table_declared_max: Vec<i64> = parsed_uninstrumented
        .table_types
        .iter()
        .map(|t| {
            // Reason for fallback: an unbounded table (`size_max == None`) is clamped to
            // MAX_CLAMPED_DECLARED_TABLE_ELEMENTS so the logical-denial sentinel delta
            // (`TABLE_GROW_DENIAL_SENTINEL_DELTA`) always exceeds the declared table limit.
            let clamped = t
                .size_max
                .unwrap_or(MAX_CLAMPED_DECLARED_TABLE_ELEMENTS)
                .min(MAX_CLAMPED_DECLARED_TABLE_ELEMENTS.max(t.size_initial));
            i64::from(clamped)
        })
        .collect();

    ensure!(
        wasm_bytes.get(0..8) == Some(WASM_HEADER.as_slice()),
        "Invalid WebAssembly binary magic/version header"
    );
    let mut pos = 8usize;
    let mut sec_type: Option<&[u8]> = None;
    let mut sec_import: Option<&[u8]> = None;
    let mut sec_func: Option<&[u8]> = None;
    let mut sec_table: Option<&[u8]> = None;
    let mut sec_memory: Option<&[u8]> = None;
    let mut sec_global: Option<&[u8]> = None;
    let mut sec_export: Option<&[u8]> = None;
    let mut sec_start: Option<&[u8]> = None;
    let mut sec_element: Option<&[u8]> = None;
    let mut sec_datacount: Option<&[u8]> = None;
    let mut sec_code: Option<&[u8]> = None;
    let mut sec_data: Option<&[u8]> = None;

    while pos < wasm_bytes.len() {
        let sec_id = read_byte(wasm_bytes, &mut pos)?;
        let sec_len = usize::try_from(read_uleb_u32(wasm_bytes, &mut pos)?)?;
        let sec_end = pos
            .checked_add(sec_len)
            .context("WASM section length overflow")?;
        let payload = wasm_bytes
            .get(pos..sec_end)
            .context("Truncated WASM section payload")?;
        pos = sec_end;
        match sec_id {
            SEC_CUSTOM => {}
            SEC_TYPE => sec_type = Some(payload),
            SEC_IMPORT => sec_import = Some(payload),
            SEC_FUNCTION => sec_func = Some(payload),
            SEC_TABLE => sec_table = Some(payload),
            SEC_MEMORY => sec_memory = Some(payload),
            SEC_GLOBAL => sec_global = Some(payload),
            SEC_EXPORT => sec_export = Some(payload),
            SEC_START => sec_start = Some(payload),
            SEC_ELEMENT => sec_element = Some(payload),
            SEC_DATA_COUNT => sec_datacount = Some(payload),
            SEC_CODE => sec_code = Some(payload),
            SEC_DATA => sec_data = Some(payload),
            other => bail!("Unsupported WASM section id {other}"),
        }
    }

    // 1. Parse Type section count
    let (num_guest_types, raw_types_tail) = if let Some(payload) = sec_type {
        let mut p = 0usize;
        let count = read_uleb_u32(payload, &mut p)?;
        let tail = payload
            .get(p..)
            .context("Truncated WASM Type section payload")?;
        (count, tail)
    } else {
        (0u32, &[][..])
    };
    let deduct_type_idx = num_guest_types;

    // 3. Parse Function section and collect each guest function's parameter count
    let mut guest_func_type_indices: Vec<u32> = Vec::new();
    let raw_funcs_tail = if let Some(payload) = sec_func {
        let mut p = 0usize;
        let count = read_uleb_u32(payload, &mut p)?;
        let tail_start = p;
        for _ in 0..count {
            guest_func_type_indices.push(read_uleb_u32(payload, &mut p)?);
        }
        payload
            .get(tail_start..)
            .context("Truncated WASM Function section payload")?
    } else {
        &[][..]
    };
    let num_guest_funcs = u32::try_from(guest_func_type_indices.len())?;
    let deduct_func_idx = num_guest_funcs;

    // 6. Parse Global section count
    let (num_guest_globals_u32, raw_globals_tail) =
        if let Some(payload) = sec_global {
            let mut p = 0usize;
            let count = read_uleb_u32(payload, &mut p)?;
            let tail = payload
                .get(p..)
                .context("Truncated WASM Global section payload")?;
            (count, tail)
        } else {
            (0u32, &[][..])
        };

    let indices = QuotaIndices {
        deduct_func_idx,
        g_quota_rem: num_guest_globals_u32,
        g_quota_consumed: num_guest_globals_u32.saturating_add(1),
        g_exhausted: num_guest_globals_u32.saturating_add(2),
        g_interrupted_op: num_guest_globals_u32.saturating_add(3),
        g_required_cost: num_guest_globals_u32.saturating_add(4),
        g_op_ordinal: num_guest_globals_u32.saturating_add(5),
        g_total_mem_pages: num_guest_globals_u32.saturating_add(6),
        g_max_mem_pages: num_guest_globals_u32.saturating_add(7),
        g_phys_max_mem_pages: num_guest_globals_u32.saturating_add(8),
        g_total_table_elems: num_guest_globals_u32.saturating_add(9),
        g_max_table_elems: num_guest_globals_u32.saturating_add(10),
        g_phys_max_table_elems: num_guest_globals_u32.saturating_add(11),
    };

    // 8. Parse Start section if present
    let start_func_idx = if let Some(payload) = sec_start {
        let mut p = 0usize;
        Some(read_uleb_u32(payload, &mut p)?)
    } else {
        None
    };

    let mut out = Vec::with_capacity(wasm_bytes.len().saturating_mul(2));
    out.extend_from_slice(&WASM_HEADER);

    // Emit Section 1 (Type): original types + `(func (param i64))`
    {
        let mut sec = Vec::with_capacity(raw_types_tail.len().saturating_add(8));
        push_uleb_u32(num_guest_types.saturating_add(1), &mut sec);
        sec.extend_from_slice(raw_types_tail);
        sec.extend_from_slice(&[0x60, 0x01, VALTYPE_I64, 0x00]);
        push_section(SEC_TYPE, &sec, &mut out)?;
    }

    // Emit Section 2 (Import) if present
    if let Some(payload) = sec_import {
        push_section(SEC_IMPORT, payload, &mut out)?;
    }

    // Emit Section 3 (Function): original functions + `$__ctb_deduct`
    {
        let mut sec = Vec::with_capacity(raw_funcs_tail.len().saturating_add(8));
        push_uleb_u32(num_guest_funcs.saturating_add(1), &mut sec);
        sec.extend_from_slice(raw_funcs_tail);
        push_uleb_u32(deduct_type_idx, &mut sec);
        push_section(SEC_FUNCTION, &sec, &mut out)?;
    }

    // Emit Section 4 (Table) with clamped declared maximums (`<= 10_000_000`)
    if !parsed_uninstrumented.table_types.is_empty() {
        let mut sec = Vec::new();
        push_uleb_u32(
            u32::try_from(parsed_uninstrumented.table_types.len())?,
            &mut sec,
        );
        for t in &parsed_uninstrumented.table_types {
            let elem_byte = match t.element_type {
                ctb_formats_dcdata::tinywasm::types::ValType::RefFunc => 0x70,
                ctb_formats_dcdata::tinywasm::types::ValType::RefExtern => 0x6F,
                other => bail!("Unsupported WASM table element type {other:?}"),
            };
            // Reason for fallback: an unbounded table (`size_max == None`) is clamped to
            // MAX_CLAMPED_DECLARED_TABLE_ELEMENTS so the logical-denial sentinel delta
            // (`TABLE_GROW_DENIAL_SENTINEL_DELTA`) always exceeds the declared table limit.
            let max_clamped = t
                .size_max
                .unwrap_or(MAX_CLAMPED_DECLARED_TABLE_ELEMENTS)
                .min(MAX_CLAMPED_DECLARED_TABLE_ELEMENTS.max(t.size_initial));
            sec.push(elem_byte);
            sec.push(0x01); // has max
            push_uleb_u32(t.size_initial, &mut sec);
            push_uleb_u32(max_clamped, &mut sec);
        }
        push_section(SEC_TABLE, &sec, &mut out)?;
    } else if let Some(payload) = sec_table {
        push_section(SEC_TABLE, payload, &mut out)?;
    }

    // Emit Section 5 (Memory)
    if let Some(payload) = sec_memory {
        push_section(SEC_MEMORY, payload, &mut out)?;
    }

    // Emit Section 6 (Global): original globals + 12 `__ctb_*` globals
    {
        let mut sec =
            Vec::with_capacity(raw_globals_tail.len().saturating_add(96));
        push_uleb_u32(num_guest_globals_u32.saturating_add(12), &mut sec);
        sec.extend_from_slice(raw_globals_tail);

        let initial_mem_i64 = i64::try_from(initial_memory_pages)?;
        let initial_tbl_i64 = i64::try_from(initial_table_elements)?;
        let appended_globals: [(u8, i64); 12] = [
            (VALTYPE_I64, i64::MAX),        // __ctb_quota_remaining
            (VALTYPE_I64, 0),               // __ctb_quota_consumed
            (VALTYPE_I32, 0),               // __ctb_exhausted
            (VALTYPE_I64, 0),               // __ctb_interrupted_op
            (VALTYPE_I64, 0),               // __ctb_required_cost
            (VALTYPE_I64, 0),               // __ctb_op_ordinal
            (VALTYPE_I64, initial_mem_i64), // __ctb_total_mem_pages
            (VALTYPE_I64, 65_536),          // __ctb_max_mem_pages
            (VALTYPE_I64, 65_536),          // __ctb_phys_max_mem_pages
            (VALTYPE_I64, initial_tbl_i64), // __ctb_total_table_elems
            (VALTYPE_I64, 65_536),          // __ctb_max_table_elems
            (VALTYPE_I64, 65_536),          // __ctb_phys_max_table_elems
        ];
        for (vt, init_val) in appended_globals {
            sec.push(vt);
            sec.push(0x01); // mutable
            if vt == VALTYPE_I64 {
                sec.push(0x42); // i64.const
                push_sleb_i64(init_val, &mut sec);
            } else {
                sec.push(0x41); // i32.const
                push_sleb_i32(i32::try_from(init_val)?, &mut sec);
            }
            sec.push(0x0B); // end
        }
        push_section(SEC_GLOBAL, &sec, &mut out)?;
    }

    // Emit Section 7 (Export): original exports + 12 `__ctb_*` globals + optional `__ctb_wasm_start`
    {
        let (num_guest_exports, raw_exports_tail) =
            if let Some(payload) = sec_export {
                let mut p = 0usize;
                let count = read_uleb_u32(payload, &mut p)?;
                let tail = payload
                    .get(p..)
                    .context("Truncated WASM Export section payload")?;
                (count, tail)
            } else {
                (0u32, &[][..])
            };
        let extra_exports = if start_func_idx.is_some() { 13u32 } else { 12u32 };
        let mut sec =
            Vec::with_capacity(raw_exports_tail.len().saturating_add(320));
        push_uleb_u32(num_guest_exports.saturating_add(extra_exports), &mut sec);
        sec.extend_from_slice(raw_exports_tail);

        let global_exports: [(&str, u32); 12] = [
            (CTB_QUOTA_REMAINING_EXPORT, indices.g_quota_rem),
            (CTB_QUOTA_CONSUMED_EXPORT, indices.g_quota_consumed),
            (CTB_EXHAUSTED_EXPORT, indices.g_exhausted),
            (CTB_INTERRUPTED_OP_EXPORT, indices.g_interrupted_op),
            (CTB_REQUIRED_COST_EXPORT, indices.g_required_cost),
            (CTB_OP_ORDINAL_EXPORT, indices.g_op_ordinal),
            (CTB_TOTAL_MEM_PAGES_EXPORT, indices.g_total_mem_pages),
            (CTB_MAX_MEM_PAGES_EXPORT, indices.g_max_mem_pages),
            (CTB_PHYS_MAX_MEM_PAGES_EXPORT, indices.g_phys_max_mem_pages),
            (CTB_TOTAL_TABLE_ELEMS_EXPORT, indices.g_total_table_elems),
            (CTB_MAX_TABLE_ELEMS_EXPORT, indices.g_max_table_elems),
            (
                CTB_PHYS_MAX_TABLE_ELEMS_EXPORT,
                indices.g_phys_max_table_elems,
            ),
        ];
        for (name, g_idx) in global_exports {
            push_name(name, &mut sec)?;
            sec.push(0x03); // global export
            push_uleb_u32(g_idx, &mut sec);
        }
        if let Some(start_idx) = start_func_idx {
            push_name(CTB_WASM_START_EXPORT, &mut sec)?;
            sec.push(0x00); // func export
            push_uleb_u32(start_idx, &mut sec);
        }
        push_section(SEC_EXPORT, &sec, &mut out)?;
    }

    // Note: Section 8 (Start) is intentionally omitted and exported as `CTB_WASM_START_EXPORT`
    // so the host initializes the compute-quota globals before running `start`.

    // Emit Section 9 (Element) if present
    if let Some(payload) = sec_element {
        push_section(SEC_ELEMENT, payload, &mut out)?;
    }

    // Emit Section 12 (DataCount) if present
    if let Some(payload) = sec_datacount {
        push_section(SEC_DATA_COUNT, payload, &mut out)?;
    }

    // Emit Section 10 (Code): instrumented guest function bodies + `$__ctb_deduct`
    {
        let mut instrumented_bodies: Vec<Vec<u8>> =
            Vec::with_capacity(usize::try_from(num_guest_funcs)?.saturating_add(1));
        if let Some(payload) = sec_code {
            let mut p = 0usize;
            let count = read_uleb_u32(payload, &mut p)?;
            ensure!(
                count == num_guest_funcs,
                "WASM Code section count ({count}) does not match Function section count ({num_guest_funcs})"
            );
            for func_i in 0..count {
                let body_len = usize::try_from(read_uleb_u32(payload, &mut p)?)?;
                let body_end = p
                    .checked_add(body_len)
                    .context("WASM function body length overflow")?;
                let body_slice = payload
                    .get(p..body_end)
                    .context("Truncated WASM function body")?;
                p = body_end;
                let type_idx = *guest_func_type_indices
                    .get(usize::try_from(func_i)?)
                    .context(
                        "WASM function index out of bounds of Function section",
                    )?;
                let ft = parsed_uninstrumented
                    .func_types
                    .get(usize::try_from(type_idx)?)
                    .context(
                        "WASM function type index out of bounds of Type section",
                    )?;
                let num_params = u32::try_from(ft.params.len())?;
                let rewritten = instrument_function_body(
                    body_slice,
                    num_params,
                    &mem_declared_max,
                    &table_declared_max,
                    &indices,
                )?;
                instrumented_bodies.push(rewritten);
            }
        }
        instrumented_bodies.push(encode_deduct_helper_body(&indices));

        let mut sec = Vec::new();
        push_uleb_u32(u32::try_from(instrumented_bodies.len())?, &mut sec);
        for body in &instrumented_bodies {
            push_uleb_u32(u32::try_from(body.len())?, &mut sec);
            sec.extend_from_slice(body);
        }
        push_section(SEC_CODE, &sec, &mut out)?;
    }

    // Emit Section 11 (Data) if present
    if let Some(payload) = sec_data {
        push_section(SEC_DATA, payload, &mut out)?;
    }

    let parsed_instrumented =
        ctb_formats_dcdata::tinywasm::parser::Parser::new()
            .parse_module_bytes(&out)
            .map_err(|error| {
                anyhow!(
                    "Failed to validate instrumented WebAssembly module: {error:?}"
                )
            })?;

    Ok(InstrumentedWasmModule {
        wasm_bytes: out,
        parsed_instrumented,
        has_start_func: start_func_idx.is_some(),
        initial_memory_pages,
        initial_table_elements,
        static_data_end,
        num_guest_globals,
        num_memories,
        num_tables,
    })
}
