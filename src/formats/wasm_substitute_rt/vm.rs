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

//! Virtual machine memory, table, passive segment, call-depth, and V2
//! structural-checkpoint quota state for native WebAssembly substitutes.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::{WasmSubstituteHost, WasmTrap};

/// Standard WebAssembly 64 KiB linear memory page size in bytes.
pub const WASM_PAGE_BYTES: usize = 65_536;

/// Maximum 32-bit WebAssembly linear memory page count (4 GiB).
pub const WASM_MAX_PAGES: u32 = 65_536;

/// Maximum table size clamped in `wasm_quota.rs` when no explicit max is declared.
const MAX_CLAMPED_DECLARED_TABLE_ELEMENTS: u32 = 10_000_000;

/// Non-tail call stack depth cap matching `tinywasm` (1,024 active frames).
pub const MAX_CALL_DEPTH: u32 = 1024;

/// Single 32-bit WebAssembly linear memory backed by a safe `Vec<u8>`.
#[derive(Debug, Clone)]
pub struct WasmLinearMemory {
    data: Vec<u8>,
    max_pages: Option<u32>,
}

impl WasmLinearMemory {
    /// Allocates a zero-initialized linear memory of `initial_pages` 64 KiB pages.
    pub fn new(initial_pages: u32, max_pages: Option<u32>) -> Self {
        // Reason for fallback: on 32-bit targets exceeding usize::MAX is unreachable
        // for valid <=65536-page WebAssembly memories validated prior to instantiation.
        let initial_bytes = usize::try_from(initial_pages)
            .unwrap_or(usize::MAX)
            .saturating_mul(WASM_PAGE_BYTES);
        Self {
            data: vec![0u8; initial_bytes],
            max_pages,
        }
    }

    /// Current size in 64 KiB pages.
    #[inline(always)]
    pub fn page_count(&self) -> usize {
        self.data.len() >> 16
    }

    /// Shared view of linear memory bytes.
    #[inline(always)]
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    /// Mutable view of linear memory bytes.
    #[inline(always)]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Reads a slice `[addr..addr + len]`, returning `OutOfBoundsMemoryAccess` on overflow.
    #[inline]
    pub fn load_bytes(
        &self,
        addr: usize,
        len: usize,
    ) -> std::result::Result<&[u8], WasmTrap> {
        let end = addr
            .checked_add(len)
            .ok_or(WasmTrap::OutOfBoundsMemoryAccess)?;
        self.data
            .get(addr..end)
            .ok_or(WasmTrap::OutOfBoundsMemoryAccess)
    }

    /// Writes `bytes` at `addr`, returning `OutOfBoundsMemoryAccess` on overflow.
    #[inline]
    pub fn store_bytes(
        &mut self,
        addr: usize,
        bytes: &[u8],
    ) -> std::result::Result<(), WasmTrap> {
        let end = addr
            .checked_add(bytes.len())
            .ok_or(WasmTrap::OutOfBoundsMemoryAccess)?;
        let dst = self
            .data
            .get_mut(addr..end)
            .ok_or(WasmTrap::OutOfBoundsMemoryAccess)?;
        dst.copy_from_slice(bytes);
        Ok(())
    }

    /// Initializes active segment data at startup.
    pub fn init_active(
        &mut self,
        offset: usize,
        bytes: &[u8],
    ) -> std::result::Result<(), WasmTrap> {
        self.store_bytes(offset, bytes)
    }

    fn declared_max_pages_u64(&self) -> u64 {
        if let Some(m) = self.max_pages {
            u64::from(m)
        } else {
            u64::from(WASM_MAX_PAGES)
        }
    }

    /// Grows linear memory on behalf of the host mailbox writer, returning `Some(old_pages)`
    /// or `None` if allocation or declared bounds fail.
    pub fn grow_host(&mut self, delta: u32) -> Option<i32> {
        let old = self.grow_raw(delta);
        (old >= 0).then_some(old)
    }

    fn grow_raw(&mut self, delta: u32) -> i32 {
        let old_pages = self.page_count();
        let Ok(old_pages_u32) = u32::try_from(old_pages) else {
            return -1;
        };
        let Some(new_pages) = old_pages_u32.checked_add(delta) else {
            return -1;
        };
        if u64::from(new_pages) > self.declared_max_pages_u64() {
            return -1;
        }
        let Ok(new_pages_usize) = usize::try_from(new_pages) else {
            return -1;
        };
        let Some(new_byte_len) = new_pages_usize.checked_mul(WASM_PAGE_BYTES)
        else {
            return -1;
        };
        let add_bytes = new_byte_len.saturating_sub(self.data.len());
        if self.data.try_reserve(add_bytes).is_err() {
            return -1;
        }
        self.data.resize(new_byte_len, 0);
        // Reason for fallback: old_pages <= 65_536 always fits in i32.
        i32::try_from(old_pages).unwrap_or(-1)
    }
}

/// WebAssembly table storing nullable function/extern reference indices.
#[derive(Debug, Clone)]
pub struct WasmTable {
    elems: Vec<Option<u32>>,
    max_elems: Option<u32>,
    initial_size: u32,
}

impl WasmTable {
    /// Allocates a null-initialized table of `initial_size` elements.
    pub fn new(initial_size: u32, max_elems: Option<u32>) -> Self {
        // Reason for fallback: WebAssembly table initial sizes are validated u32 bounds.
        let len = usize::try_from(initial_size).unwrap_or(0);
        Self {
            elems: vec![None; len],
            max_elems,
            initial_size,
        }
    }

    /// Current element count.
    #[inline(always)]
    pub fn size(&self) -> usize {
        self.elems.len()
    }

    /// Reads element `idx` or traps with `UndefinedElement` (`call_indirect` / `table.get`).
    #[inline]
    pub fn get(&self, idx: i32) -> std::result::Result<Option<u32>, WasmTrap> {
        let idx_u = u32::from_le_bytes(idx.to_le_bytes());
        let idx_usize =
            usize::try_from(idx_u).map_err(|_| WasmTrap::UndefinedElement)?;
        self.elems
            .get(idx_usize)
            .copied()
            .ok_or(WasmTrap::UndefinedElement)
    }

    /// Writes `val` into table slot `idx`.
    #[inline]
    pub fn set(
        &mut self,
        idx: i32,
        val: Option<u32>,
    ) -> std::result::Result<(), WasmTrap> {
        let idx_u = u32::from_le_bytes(idx.to_le_bytes());
        let idx_usize = usize::try_from(idx_u)
            .map_err(|_| WasmTrap::OutOfBoundsTableAccess)?;
        let slot = self
            .elems
            .get_mut(idx_usize)
            .ok_or(WasmTrap::OutOfBoundsTableAccess)?;
        *slot = val;
        Ok(())
    }

    /// Initializes active element segment entries at startup.
    pub fn init_active(
        &mut self,
        offset: usize,
        items: &[Option<u32>],
    ) -> std::result::Result<(), WasmTrap> {
        let end = offset
            .checked_add(items.len())
            .ok_or(WasmTrap::OutOfBoundsTableAccess)?;
        let dst = self
            .elems
            .get_mut(offset..end)
            .ok_or(WasmTrap::OutOfBoundsTableAccess)?;
        dst.copy_from_slice(items);
        Ok(())
    }

    fn declared_max_elems_u64(&self) -> u64 {
        // Reason for fallback: matches wasm_quota.rs clamping for unbounded tables
        // so logical denial semantics match across TinyWasm, V8, and substitutes.
        let clamped = self
            .max_elems
            .unwrap_or(MAX_CLAMPED_DECLARED_TABLE_ELEMENTS)
            .min(MAX_CLAMPED_DECLARED_TABLE_ELEMENTS.max(self.initial_size));
        u64::from(clamped)
    }

    fn grow_raw(&mut self, init_val: Option<u32>, delta: u32) -> i32 {
        let old_len = self.elems.len();
        let Ok(old_len_u32) = u32::try_from(old_len) else {
            return -1;
        };
        let Some(new_len) = old_len_u32.checked_add(delta) else {
            return -1;
        };
        if u64::from(new_len) > self.declared_max_elems_u64() {
            return -1;
        }
        let Ok(new_len_usize) = usize::try_from(new_len) else {
            return -1;
        };
        let add_elems = new_len_usize.saturating_sub(old_len);
        if self.elems.try_reserve(add_elems).is_err() {
            return -1;
        }
        self.elems.resize(new_len_usize, init_val);
        // Reason for fallback: old_len <= MAX_CLAMPED_DECLARED_TABLE_ELEMENTS fits in i32.
        i32::try_from(old_len).unwrap_or(-1)
    }
}

/// Function pointer wrapper for a substitute module's shared initializer.
#[derive(Clone, Copy)]
pub struct InitFnPtr(
    pub  fn(
        &mut SubstituteVm,
        &mut dyn WasmSubstituteHost,
    ) -> std::result::Result<(), WasmTrap>,
);

impl std::fmt::Debug for InitFnPtr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("InitFnPtr(..)")
    }
}

/// Complete WebAssembly VM state and V2 metering globals for a substitute instance.
#[derive(Debug, Clone)]
pub struct SubstituteVm {
    /// Inline fast-path storage for linear memory `0`.
    pub mem0: Option<WasmLinearMemory>,
    /// Linear memories `1..` declared by multi-memory modules.
    pub extra_memories: Vec<WasmLinearMemory>,
    /// Tables declared by the module.
    pub tables: Vec<WasmTable>,
    /// Passive data segments (cleared to `None` on `data.drop`).
    pub passive_data: Vec<Option<&'static [u8]>>,
    /// Passive element segments (cleared to `None` on `elem.drop`).
    pub passive_elems: Vec<Option<&'static [Option<u32>]>>,
    /// Dedicated guest `__heap_ptr` linear-memory bump pointer global (`i32`).
    pub heap_ptr: i32,
    /// Whether the instantiated module declares an exported mutable `__heap_ptr`.
    pub has_heap_ptr: bool,
    /// Module-private mutable/immutable `i32` globals (excluding `__heap_ptr`).
    pub globals_i32: Vec<i32>,
    /// Module-private `i64` globals.
    pub globals_i64: Vec<i64>,
    /// Module-private `f32` globals.
    pub globals_f32: Vec<f32>,
    /// Module-private `f64` globals.
    pub globals_f64: Vec<f64>,
    /// Module-private reference globals (`funcref` / `externref`).
    pub globals_ref: Vec<Option<u32>>,
    /// Optional module-specific shared initializer hook (e.g. `__init_top_vals`).
    pub init_fn: Option<InitFnPtr>,
    /// Active non-tail call depth.
    pub call_depth: u32,
    /// Remaining compute quota units (`__ctb_quota_remaining`).
    pub quota_remaining: u64,
    /// Total compute quota units consumed (`__ctb_quota_consumed`).
    pub quota_consumed: u64,
    /// Exhaustion reason code (`0=none`, `1=compute`, `2=memory`, `3=table`).
    pub exhausted: i32,
    /// 1-based paused checkpoint ordinal or requested memory/table units (`__ctb_interrupted_op`).
    pub interrupted_op: u64,
    /// Required cost of the interrupted checkpoint (`__ctb_required_cost`).
    pub required_cost: u64,
    /// 1-based executed checkpoint ordinal (`__ctb_op_ordinal`).
    pub op_ordinal: u64,
    /// Current total 64 KiB pages across all module memories (`__ctb_total_mem_pages`).
    pub total_mem_pages: u64,
    /// Logical maximum total pages (`__ctb_max_mem_pages`).
    pub max_mem_pages: u64,
    /// Physical host cap on total pages (`__ctb_phys_max_mem_pages`).
    pub phys_max_mem_pages: u64,
    /// Current total elements across all module tables (`__ctb_total_table_elems`).
    pub total_table_elems: u64,
    /// Logical maximum total table elements (`__ctb_max_table_elems`).
    pub max_table_elems: u64,
    /// Physical host cap on total table elements (`__ctb_phys_max_table_elems`).
    pub phys_max_table_elems: u64,
    /// Single-threaded replay resume target (`__ctb_resume_target_op`).
    pub resume_target_op: u64,
}

impl SubstituteVm {
    /// Constructs a new VM with the given initial memories, tables, and passive segments.
    pub fn new(
        memories: Vec<WasmLinearMemory>,
        tables: Vec<WasmTable>,
        passive_data: Vec<Option<&'static [u8]>>,
        passive_elems: Vec<Option<&'static [Option<u32>]>>,
    ) -> Self {
        let initial_mem_pages = memories.iter().fold(0u64, |acc, m| {
            // Reason for fallback: WASM page_count <= 65536 always fits in u64.
            acc.saturating_add(u64::try_from(m.page_count()).unwrap_or(0))
        });
        let initial_tbl_elems = tables.iter().fold(0u64, |acc, t| {
            // Reason for fallback: WASM table size is u32 and always fits in u64.
            acc.saturating_add(u64::try_from(t.size()).unwrap_or(0))
        });
        let mut mem_iter = memories.into_iter();
        let mem0 = mem_iter.next();
        let extra_memories: Vec<WasmLinearMemory> = mem_iter.collect();
        Self {
            mem0,
            extra_memories,
            tables,
            passive_data,
            passive_elems,
            heap_ptr: 0,
            has_heap_ptr: false,
            globals_i32: Vec::new(),
            globals_i64: Vec::new(),
            globals_f32: Vec::new(),
            globals_f64: Vec::new(),
            globals_ref: Vec::new(),
            init_fn: None,
            call_depth: 0,
            quota_remaining: u64::MAX >> 1,
            quota_consumed: 0,
            exhausted: 0,
            interrupted_op: 0,
            required_cost: 0,
            op_ordinal: 0,
            total_mem_pages: initial_mem_pages,
            max_mem_pages: 65_536,
            phys_max_mem_pages: 65_536,
            total_table_elems: initial_tbl_elems,
            max_table_elems: 65_536,
            phys_max_table_elems: 65_536,
            resume_target_op: 0,
        }
    }

    /// Returns a shared reference to linear memory `mem_idx` (`0` uses inline `mem0`).
    #[inline(always)]
    pub fn memory(&self, mem_idx: usize) -> Option<&WasmLinearMemory> {
        if mem_idx == 0 {
            self.mem0.as_ref()
        } else {
            mem_idx
                .checked_sub(1)
                .and_then(|i| self.extra_memories.get(i))
        }
    }

    /// Returns a mutable reference to linear memory `mem_idx` (`0` uses inline `mem0`).
    #[inline(always)]
    pub fn memory_mut(&mut self, mem_idx: usize) -> Option<&mut WasmLinearMemory> {
        if mem_idx == 0 {
            self.mem0.as_mut()
        } else {
            mem_idx
                .checked_sub(1)
                .and_then(|i| self.extra_memories.get_mut(i))
        }
    }

    /// Reads an `i32` global from `globals_i32[slot]`.
    #[inline(always)]
    pub fn get_i32(&self, slot: usize) -> std::result::Result<i32, WasmTrap> {
        self.globals_i32
            .get(slot)
            .copied()
            .ok_or(WasmTrap::Unreachable)
    }

    /// Writes `val` to `globals_i32[slot]`.
    #[inline(always)]
    pub fn set_i32(
        &mut self,
        slot: usize,
        val: i32,
    ) -> std::result::Result<(), WasmTrap> {
        let entry = self
            .globals_i32
            .get_mut(slot)
            .ok_or(WasmTrap::Unreachable)?;
        *entry = val;
        Ok(())
    }

    /// Reads an `i64` global from `globals_i64[slot]`.
    #[inline(always)]
    pub fn get_i64(&self, slot: usize) -> std::result::Result<i64, WasmTrap> {
        self.globals_i64
            .get(slot)
            .copied()
            .ok_or(WasmTrap::Unreachable)
    }

    /// Writes `val` to `globals_i64[slot]`.
    #[inline(always)]
    pub fn set_i64(
        &mut self,
        slot: usize,
        val: i64,
    ) -> std::result::Result<(), WasmTrap> {
        let entry = self
            .globals_i64
            .get_mut(slot)
            .ok_or(WasmTrap::Unreachable)?;
        *entry = val;
        Ok(())
    }

    /// Reads an `f32` global from `globals_f32[slot]`.
    #[inline(always)]
    pub fn get_f32(&self, slot: usize) -> std::result::Result<f32, WasmTrap> {
        self.globals_f32
            .get(slot)
            .copied()
            .ok_or(WasmTrap::Unreachable)
    }

    /// Writes `val` to `globals_f32[slot]`.
    #[inline(always)]
    pub fn set_f32(
        &mut self,
        slot: usize,
        val: f32,
    ) -> std::result::Result<(), WasmTrap> {
        let entry = self
            .globals_f32
            .get_mut(slot)
            .ok_or(WasmTrap::Unreachable)?;
        *entry = val;
        Ok(())
    }

    /// Reads an `f64` global from `globals_f64[slot]`.
    #[inline(always)]
    pub fn get_f64(&self, slot: usize) -> std::result::Result<f64, WasmTrap> {
        self.globals_f64
            .get(slot)
            .copied()
            .ok_or(WasmTrap::Unreachable)
    }

    /// Writes `val` to `globals_f64[slot]`.
    #[inline(always)]
    pub fn set_f64(
        &mut self,
        slot: usize,
        val: f64,
    ) -> std::result::Result<(), WasmTrap> {
        let entry = self
            .globals_f64
            .get_mut(slot)
            .ok_or(WasmTrap::Unreachable)?;
        *entry = val;
        Ok(())
    }

    /// Reads a reference global from `globals_ref[slot]`.
    #[inline(always)]
    pub fn get_ref(
        &self,
        slot: usize,
    ) -> std::result::Result<Option<u32>, WasmTrap> {
        self.globals_ref
            .get(slot)
            .copied()
            .ok_or(WasmTrap::Unreachable)
    }

    /// Writes `val` to `globals_ref[slot]`.
    #[inline(always)]
    pub fn set_ref(
        &mut self,
        slot: usize,
        val: Option<u32>,
    ) -> std::result::Result<(), WasmTrap> {
        let entry = self
            .globals_ref
            .get_mut(slot)
            .ok_or(WasmTrap::Unreachable)?;
        *entry = val;
        Ok(())
    }

    /// Invokes the module's registered shared initializer hook, if present.
    #[inline(always)]
    pub fn call_init(
        &mut self,
        host: &mut dyn WasmSubstituteHost,
    ) -> std::result::Result<(), WasmTrap> {
        let init = self.init_fn.ok_or(WasmTrap::Unreachable)?;
        (init.0)(self, host)
    }

    /// Deducts `cost` compute units under exact V2 `deduct-before-effect`
    /// semantics, suspending via `host` whenever `quota_remaining < cost`.
    #[inline(always)]
    pub fn deduct(
        &mut self,
        host: &mut dyn WasmSubstituteHost,
        cost: u64,
    ) -> std::result::Result<(), WasmTrap> {
        self.op_ordinal = self.op_ordinal.saturating_add(1);
        if self.op_ordinal < self.resume_target_op {
            return Ok(());
        }
        if self.quota_remaining < cost {
            return self.deduct_slow(host, cost);
        }
        self.quota_remaining = self.quota_remaining.saturating_sub(cost);
        self.quota_consumed = self.quota_consumed.saturating_add(cost);
        Ok(())
    }

    #[cold]
    fn deduct_slow(
        &mut self,
        host: &mut dyn WasmSubstituteHost,
        cost: u64,
    ) -> std::result::Result<(), WasmTrap> {
        while self.quota_remaining < cost {
            self.exhausted = 1;
            self.interrupted_op = self.op_ordinal;
            self.required_cost = cost;
            host.suspend_on_quota(self)?;
        }
        self.quota_remaining = self.quota_remaining.saturating_sub(cost);
        self.quota_consumed = self.quota_consumed.saturating_add(cost);
        Ok(())
    }

    /// Increments non-tail call depth, trapping on overflow past [`MAX_CALL_DEPTH`].
    #[inline(always)]
    pub fn enter_call(&mut self) -> std::result::Result<(), WasmTrap> {
        let next = self.call_depth.saturating_add(1);
        if next > MAX_CALL_DEPTH {
            return Err(WasmTrap::CallStackExhausted);
        }
        self.call_depth = next;
        Ok(())
    }

    /// Decrements non-tail call depth when returning from a normal call.
    #[inline(always)]
    pub fn exit_call(&mut self) {
        self.call_depth = self.call_depth.saturating_sub(1);
    }

    /// Executes `memory.size mem_idx`.
    #[inline]
    pub fn memory_size(
        &self,
        mem_idx: usize,
    ) -> std::result::Result<i32, WasmTrap> {
        let mem = self
            .memory(mem_idx)
            .ok_or(WasmTrap::OutOfBoundsMemoryAccess)?;
        // Reason for fallback: page_count <= 65_536 always fits in i32.
        Ok(i32::try_from(mem.page_count()).unwrap_or(i32::MAX))
    }

    /// Executes `memory.grow mem_idx` matching `wasm_quota.rs` cost and logical vs. physical caps.
    pub fn memory_grow(
        &mut self,
        mem_idx: usize,
        delta: i32,
        host: &mut dyn WasmSubstituteHost,
    ) -> std::result::Result<i32, WasmTrap> {
        let (cur_pages, declared_max) = {
            let mem = self
                .memory(mem_idx)
                .ok_or(WasmTrap::OutOfBoundsMemoryAccess)?;
            (
                // Reason for fallback: page_count <= 65_536 always fits in u64.
                u64::try_from(mem.page_count()).unwrap_or(u64::MAX),
                mem.declared_max_pages_u64(),
            )
        };
        let delta_u32 = u32::from_le_bytes(delta.to_le_bytes());
        let delta_u64 = u64::from(delta_u32);
        let logically_denied = delta < 0
            || self.total_mem_pages.saturating_add(delta_u64)
                > self.max_mem_pages
            || cur_pages.saturating_add(delta_u64) > declared_max;
        if logically_denied {
            self.deduct(host, 1)?;
            return Ok(-1);
        }
        let cost = 1u64.saturating_add(64u64.saturating_mul(delta_u64));
        self.deduct(host, cost)?;
        let next_total = self.total_mem_pages.saturating_add(delta_u64);
        if next_total > self.phys_max_mem_pages {
            self.exhausted = 2;
            self.interrupted_op = next_total;
            return Err(WasmTrap::Unreachable);
        }
        let mem = self
            .memory_mut(mem_idx)
            .ok_or(WasmTrap::OutOfBoundsMemoryAccess)?;
        let old = mem.grow_raw(delta_u32);
        if old < 0 {
            self.exhausted = 2;
            self.interrupted_op = next_total;
            return Err(WasmTrap::Unreachable);
        }
        self.total_mem_pages = next_total;
        Ok(old)
    }

    /// Initializes active data bytes in linear memory `mem_idx` at startup.
    pub fn init_active_data(
        &mut self,
        mem_idx: usize,
        offset: usize,
        bytes: &[u8],
    ) -> std::result::Result<(), WasmTrap> {
        let mem = self
            .memory_mut(mem_idx)
            .ok_or(WasmTrap::OutOfBoundsMemoryAccess)?;
        mem.init_active(offset, bytes)
    }

    /// Initializes active table entries in `tables[table_idx]` at startup.
    pub fn init_active_elem(
        &mut self,
        table_idx: usize,
        offset: usize,
        items: &[Option<u32>],
    ) -> std::result::Result<(), WasmTrap> {
        let tbl = self
            .tables
            .get_mut(table_idx)
            .ok_or(WasmTrap::OutOfBoundsTableAccess)?;
        tbl.init_active(offset, items)
    }

    /// Executes `table.get table_idx`.
    #[inline]
    pub fn table_get(
        &self,
        table_idx: usize,
        idx: i32,
    ) -> std::result::Result<Option<u32>, WasmTrap> {
        let tbl = self
            .tables
            .get(table_idx)
            .ok_or(WasmTrap::OutOfBoundsTableAccess)?;
        tbl.get(idx).map_err(|_| WasmTrap::OutOfBoundsTableAccess)
    }

    /// Executes `table.set table_idx`.
    #[inline]
    pub fn table_set(
        &mut self,
        table_idx: usize,
        idx: i32,
        val: Option<u32>,
    ) -> std::result::Result<(), WasmTrap> {
        let tbl = self
            .tables
            .get_mut(table_idx)
            .ok_or(WasmTrap::OutOfBoundsTableAccess)?;
        tbl.set(idx, val)
    }

    /// Resolves a function index from `tables[table_idx][idx]` for `call_indirect`
    /// or `return_call_indirect`.
    #[inline]
    pub fn call_indirect_target(
        &self,
        table_idx: usize,
        idx: i32,
    ) -> std::result::Result<u32, WasmTrap> {
        let tbl = self
            .tables
            .get(table_idx)
            .ok_or(WasmTrap::UndefinedElement)?;
        tbl.get(idx)?.ok_or(WasmTrap::UninitializedElement)
    }

    /// Executes `table.size table_idx`.
    #[inline]
    pub fn table_size(
        &self,
        table_idx: usize,
    ) -> std::result::Result<i32, WasmTrap> {
        let tbl = self
            .tables
            .get(table_idx)
            .ok_or(WasmTrap::OutOfBoundsTableAccess)?;
        // Reason for fallback: WASM table sizes are bounded within u32/i32 limits.
        Ok(i32::try_from(tbl.size()).unwrap_or(i32::MAX))
    }

    /// Executes `table.grow table_idx` matching `wasm_quota.rs` cost and logical vs. physical caps.
    pub fn table_grow(
        &mut self,
        table_idx: usize,
        init_val: Option<u32>,
        delta: i32,
        host: &mut dyn WasmSubstituteHost,
    ) -> std::result::Result<i32, WasmTrap> {
        let (cur_elems, declared_max) = {
            let tbl = self
                .tables
                .get(table_idx)
                .ok_or(WasmTrap::OutOfBoundsTableAccess)?;
            (
                // Reason for fallback: table size is validated u32 and always fits in u64.
                u64::try_from(tbl.size()).unwrap_or(u64::MAX),
                tbl.declared_max_elems_u64(),
            )
        };
        let delta_u32 = u32::from_le_bytes(delta.to_le_bytes());
        let delta_u64 = u64::from(delta_u32);
        let logically_denied = delta < 0
            || self.total_table_elems.saturating_add(delta_u64)
                > self.max_table_elems
            || cur_elems.saturating_add(delta_u64) > declared_max;
        if logically_denied {
            self.deduct(host, 1)?;
            return Ok(-1);
        }
        let cost = 1u64.saturating_add(delta_u64);
        self.deduct(host, cost)?;
        let next_total = self.total_table_elems.saturating_add(delta_u64);
        if next_total > self.phys_max_table_elems {
            self.exhausted = 3;
            self.interrupted_op = next_total;
            return Err(WasmTrap::Unreachable);
        }
        let tbl = self
            .tables
            .get_mut(table_idx)
            .ok_or(WasmTrap::OutOfBoundsTableAccess)?;
        let old = tbl.grow_raw(init_val, delta_u32);
        if old < 0 {
            self.exhausted = 3;
            self.interrupted_op = next_total;
            return Err(WasmTrap::Unreachable);
        }
        self.total_table_elems = next_total;
        Ok(old)
    }

    /// Executes `memory.fill mem_idx` with V2 `1 + u64(n)` deduction before effect.
    pub fn memory_fill(
        &mut self,
        mem_idx: usize,
        dst: i32,
        val: i32,
        len: i32,
        host: &mut dyn WasmSubstituteHost,
    ) -> std::result::Result<(), WasmTrap> {
        let n_u32 = u32::from_le_bytes(len.to_le_bytes());
        self.deduct(host, 1u64.saturating_add(u64::from(n_u32)))?;
        let dst_usize = usize::try_from(u32::from_le_bytes(dst.to_le_bytes()))
            .map_err(|_| WasmTrap::OutOfBoundsMemoryAccess)?;
        let n_usize = usize::try_from(n_u32)
            .map_err(|_| WasmTrap::OutOfBoundsMemoryAccess)?;
        let end = dst_usize
            .checked_add(n_usize)
            .ok_or(WasmTrap::OutOfBoundsMemoryAccess)?;
        let mem = self
            .memory_mut(mem_idx)
            .ok_or(WasmTrap::OutOfBoundsMemoryAccess)?;
        let target = mem
            .as_mut_slice()
            .get_mut(dst_usize..end)
            .ok_or(WasmTrap::OutOfBoundsMemoryAccess)?;
        let byte = u8::try_from(u32::from_le_bytes(val.to_le_bytes()) & 0xFF)
            .map_err(|_| WasmTrap::Unreachable)?;
        target.fill(byte);
        Ok(())
    }

    /// Executes `memory.copy dst_mem src_mem` with V2 `1 + u64(n)` deduction before effect.
    pub fn memory_copy(
        &mut self,
        dst_mem: usize,
        src_mem: usize,
        dst: i32,
        src: i32,
        len: i32,
        host: &mut dyn WasmSubstituteHost,
    ) -> std::result::Result<(), WasmTrap> {
        let n_u32 = u32::from_le_bytes(len.to_le_bytes());
        self.deduct(host, 1u64.saturating_add(u64::from(n_u32)))?;
        let dst_usize = usize::try_from(u32::from_le_bytes(dst.to_le_bytes()))
            .map_err(|_| WasmTrap::OutOfBoundsMemoryAccess)?;
        let src_usize = usize::try_from(u32::from_le_bytes(src.to_le_bytes()))
            .map_err(|_| WasmTrap::OutOfBoundsMemoryAccess)?;
        let n_usize = usize::try_from(n_u32)
            .map_err(|_| WasmTrap::OutOfBoundsMemoryAccess)?;
        let dst_end = dst_usize
            .checked_add(n_usize)
            .ok_or(WasmTrap::OutOfBoundsMemoryAccess)?;
        let src_end = src_usize
            .checked_add(n_usize)
            .ok_or(WasmTrap::OutOfBoundsMemoryAccess)?;

        if dst_mem == src_mem {
            let mem = self
                .memory_mut(dst_mem)
                .ok_or(WasmTrap::OutOfBoundsMemoryAccess)?;
            let buf = mem.as_mut_slice();
            if src_end > buf.len() || dst_end > buf.len() {
                return Err(WasmTrap::OutOfBoundsMemoryAccess);
            }
            buf.copy_within(src_usize..src_end, dst_usize);
        } else {
            let copied = {
                let s_mem = self
                    .memory(src_mem)
                    .ok_or(WasmTrap::OutOfBoundsMemoryAccess)?;
                s_mem
                    .as_slice()
                    .get(src_usize..src_end)
                    .ok_or(WasmTrap::OutOfBoundsMemoryAccess)?
                    .to_vec()
            };
            let d_mem = self
                .memory_mut(dst_mem)
                .ok_or(WasmTrap::OutOfBoundsMemoryAccess)?;
            d_mem.store_bytes(dst_usize, &copied)?;
        }
        Ok(())
    }

    /// Executes `memory.init mem_idx seg_idx` with V2 `1 + u64(n)` deduction before effect.
    pub fn memory_init(
        &mut self,
        mem_idx: usize,
        seg_idx: usize,
        dst: i32,
        src: i32,
        len: i32,
        host: &mut dyn WasmSubstituteHost,
    ) -> std::result::Result<(), WasmTrap> {
        let n_u32 = u32::from_le_bytes(len.to_le_bytes());
        self.deduct(host, 1u64.saturating_add(u64::from(n_u32)))?;
        let dst_usize = usize::try_from(u32::from_le_bytes(dst.to_le_bytes()))
            .map_err(|_| WasmTrap::OutOfBoundsMemoryAccess)?;
        let src_usize = usize::try_from(u32::from_le_bytes(src.to_le_bytes()))
            .map_err(|_| WasmTrap::OutOfBoundsMemoryAccess)?;
        let n_usize = usize::try_from(n_u32)
            .map_err(|_| WasmTrap::OutOfBoundsMemoryAccess)?;
        let src_end = src_usize
            .checked_add(n_usize)
            .ok_or(WasmTrap::OutOfBoundsMemoryAccess)?;
        let seg = self
            .passive_data
            .get(seg_idx)
            .copied()
            .ok_or(WasmTrap::OutOfBoundsMemoryAccess)?
            // Reason for fallback: a dropped passive segment behaves as length 0 per WASM spec.
            .unwrap_or(&[]);
        let slice = seg
            .get(src_usize..src_end)
            .ok_or(WasmTrap::OutOfBoundsMemoryAccess)?;
        let mem = self
            .memory_mut(mem_idx)
            .ok_or(WasmTrap::OutOfBoundsMemoryAccess)?;
        mem.store_bytes(dst_usize, slice)
    }

    /// Executes `data.drop seg_idx`.
    #[inline]
    pub fn data_drop(&mut self, seg_idx: usize) {
        if let Some(slot) = self.passive_data.get_mut(seg_idx) {
            *slot = None;
        }
    }

    /// Executes `table.fill table_idx` with V2 `1 + u64(n)` deduction before effect.
    pub fn table_fill(
        &mut self,
        table_idx: usize,
        dst: i32,
        val: Option<u32>,
        len: i32,
        host: &mut dyn WasmSubstituteHost,
    ) -> std::result::Result<(), WasmTrap> {
        let n_u32 = u32::from_le_bytes(len.to_le_bytes());
        self.deduct(host, 1u64.saturating_add(u64::from(n_u32)))?;
        let dst_usize = usize::try_from(u32::from_le_bytes(dst.to_le_bytes()))
            .map_err(|_| WasmTrap::OutOfBoundsTableAccess)?;
        let n_usize = usize::try_from(n_u32)
            .map_err(|_| WasmTrap::OutOfBoundsTableAccess)?;
        let end = dst_usize
            .checked_add(n_usize)
            .ok_or(WasmTrap::OutOfBoundsTableAccess)?;
        let tbl = self
            .tables
            .get_mut(table_idx)
            .ok_or(WasmTrap::OutOfBoundsTableAccess)?;
        let slice = tbl
            .elems
            .get_mut(dst_usize..end)
            .ok_or(WasmTrap::OutOfBoundsTableAccess)?;
        slice.fill(val);
        Ok(())
    }

    /// Executes `table.copy dst_tbl src_tbl` with V2 `1 + u64(n)` deduction before effect.
    pub fn table_copy(
        &mut self,
        dst_tbl: usize,
        src_tbl: usize,
        dst: i32,
        src: i32,
        len: i32,
        host: &mut dyn WasmSubstituteHost,
    ) -> std::result::Result<(), WasmTrap> {
        let n_u32 = u32::from_le_bytes(len.to_le_bytes());
        self.deduct(host, 1u64.saturating_add(u64::from(n_u32)))?;
        let dst_usize = usize::try_from(u32::from_le_bytes(dst.to_le_bytes()))
            .map_err(|_| WasmTrap::OutOfBoundsTableAccess)?;
        let src_usize = usize::try_from(u32::from_le_bytes(src.to_le_bytes()))
            .map_err(|_| WasmTrap::OutOfBoundsTableAccess)?;
        let n_usize = usize::try_from(n_u32)
            .map_err(|_| WasmTrap::OutOfBoundsTableAccess)?;
        let dst_end = dst_usize
            .checked_add(n_usize)
            .ok_or(WasmTrap::OutOfBoundsTableAccess)?;
        let src_end = src_usize
            .checked_add(n_usize)
            .ok_or(WasmTrap::OutOfBoundsTableAccess)?;

        if dst_tbl == src_tbl {
            let tbl = self
                .tables
                .get_mut(dst_tbl)
                .ok_or(WasmTrap::OutOfBoundsTableAccess)?;
            if src_end > tbl.elems.len() || dst_end > tbl.elems.len() {
                return Err(WasmTrap::OutOfBoundsTableAccess);
            }
            tbl.elems.copy_within(src_usize..src_end, dst_usize);
        } else {
            let copied = {
                let s_tbl = self
                    .tables
                    .get(src_tbl)
                    .ok_or(WasmTrap::OutOfBoundsTableAccess)?;
                s_tbl
                    .elems
                    .get(src_usize..src_end)
                    .ok_or(WasmTrap::OutOfBoundsTableAccess)?
                    .to_vec()
            };
            let d_tbl = self
                .tables
                .get_mut(dst_tbl)
                .ok_or(WasmTrap::OutOfBoundsTableAccess)?;
            let slice = d_tbl
                .elems
                .get_mut(dst_usize..dst_end)
                .ok_or(WasmTrap::OutOfBoundsTableAccess)?;
            slice.copy_from_slice(&copied);
        }
        Ok(())
    }

    /// Executes `table.init table_idx seg_idx` with V2 `1 + u64(n)` deduction before effect.
    pub fn table_init(
        &mut self,
        table_idx: usize,
        seg_idx: usize,
        dst: i32,
        src: i32,
        len: i32,
        host: &mut dyn WasmSubstituteHost,
    ) -> std::result::Result<(), WasmTrap> {
        let n_u32 = u32::from_le_bytes(len.to_le_bytes());
        self.deduct(host, 1u64.saturating_add(u64::from(n_u32)))?;
        let dst_usize = usize::try_from(u32::from_le_bytes(dst.to_le_bytes()))
            .map_err(|_| WasmTrap::OutOfBoundsTableAccess)?;
        let src_usize = usize::try_from(u32::from_le_bytes(src.to_le_bytes()))
            .map_err(|_| WasmTrap::OutOfBoundsTableAccess)?;
        let n_usize = usize::try_from(n_u32)
            .map_err(|_| WasmTrap::OutOfBoundsTableAccess)?;
        let dst_end = dst_usize
            .checked_add(n_usize)
            .ok_or(WasmTrap::OutOfBoundsTableAccess)?;
        let src_end = src_usize
            .checked_add(n_usize)
            .ok_or(WasmTrap::OutOfBoundsTableAccess)?;
        let seg = self
            .passive_elems
            .get(seg_idx)
            .copied()
            .ok_or(WasmTrap::OutOfBoundsTableAccess)?
            // Reason for fallback: a dropped passive element segment behaves as length 0 per WASM spec.
            .unwrap_or(&[]);
        let slice = seg
            .get(src_usize..src_end)
            .ok_or(WasmTrap::OutOfBoundsTableAccess)?;
        let tbl = self
            .tables
            .get_mut(table_idx)
            .ok_or(WasmTrap::OutOfBoundsTableAccess)?;
        let dst_slice = tbl
            .elems
            .get_mut(dst_usize..dst_end)
            .ok_or(WasmTrap::OutOfBoundsTableAccess)?;
        dst_slice.copy_from_slice(slice);
        Ok(())
    }

    /// Executes `elem.drop seg_idx`.
    #[inline]
    pub fn elem_drop(&mut self, seg_idx: usize) {
        if let Some(slot) = self.passive_elems.get_mut(seg_idx) {
            *slot = None;
        }
    }

    /// Reads 1 byte from linear memory `mem_idx` with `(addr_u32 + offset)` bounds checking.
    #[inline(always)]
    pub fn load_u8(
        &self,
        mem_idx: usize,
        addr: i32,
        offset: u32,
    ) -> std::result::Result<u8, WasmTrap> {
        let base = u64::from(u32::from_le_bytes(addr.to_le_bytes()));
        let eff = base
            .checked_add(u64::from(offset))
            .ok_or(WasmTrap::OutOfBoundsMemoryAccess)?;
        let idx =
            usize::try_from(eff).map_err(|_| WasmTrap::OutOfBoundsMemoryAccess)?;
        let mem = self
            .memory(mem_idx)
            .ok_or(WasmTrap::OutOfBoundsMemoryAccess)?;
        mem.as_slice()
            .get(idx)
            .copied()
            .ok_or(WasmTrap::OutOfBoundsMemoryAccess)
    }

    /// Reads `N` little-endian bytes from linear memory `mem_idx`.
    #[inline(always)]
    pub fn load_bytes_n<const N: usize>(
        &self,
        mem_idx: usize,
        addr: i32,
        offset: u32,
    ) -> std::result::Result<[u8; N], WasmTrap> {
        let base = u64::from(u32::from_le_bytes(addr.to_le_bytes()));
        let eff = base
            .checked_add(u64::from(offset))
            .ok_or(WasmTrap::OutOfBoundsMemoryAccess)?;
        let start =
            usize::try_from(eff).map_err(|_| WasmTrap::OutOfBoundsMemoryAccess)?;
        let end = start
            .checked_add(N)
            .ok_or(WasmTrap::OutOfBoundsMemoryAccess)?;
        let mem = self
            .memory(mem_idx)
            .ok_or(WasmTrap::OutOfBoundsMemoryAccess)?;
        let slice = mem
            .as_slice()
            .get(start..end)
            .ok_or(WasmTrap::OutOfBoundsMemoryAccess)?;
        slice
            .try_into()
            .map_err(|_| WasmTrap::OutOfBoundsMemoryAccess)
    }

    /// Writes `N` little-endian bytes into linear memory `mem_idx`.
    #[inline(always)]
    pub fn store_bytes_n<const N: usize>(
        &mut self,
        mem_idx: usize,
        addr: i32,
        offset: u32,
        bytes: [u8; N],
    ) -> std::result::Result<(), WasmTrap> {
        let base = u64::from(u32::from_le_bytes(addr.to_le_bytes()));
        let eff = base
            .checked_add(u64::from(offset))
            .ok_or(WasmTrap::OutOfBoundsMemoryAccess)?;
        let start =
            usize::try_from(eff).map_err(|_| WasmTrap::OutOfBoundsMemoryAccess)?;
        let end = start
            .checked_add(N)
            .ok_or(WasmTrap::OutOfBoundsMemoryAccess)?;
        let mem = self
            .memory_mut(mem_idx)
            .ok_or(WasmTrap::OutOfBoundsMemoryAccess)?;
        let slice = mem
            .as_mut_slice()
            .get_mut(start..end)
            .ok_or(WasmTrap::OutOfBoundsMemoryAccess)?;
        slice.copy_from_slice(&bytes);
        Ok(())
    }
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

    struct MockHost {
        suspend_refill: Option<u64>,
        suspend_calls: usize,
        last_interrupted: u64,
        last_required: u64,
        capability_calls: usize,
    }

    impl WasmSubstituteHost for MockHost {
        fn suspend_on_quota(
            &mut self,
            vm: &mut SubstituteVm,
        ) -> std::result::Result<(), WasmTrap> {
            self.suspend_calls += 1;
            self.last_interrupted = vm.interrupted_op;
            self.last_required = vm.required_cost;
            match self.suspend_refill {
                Some(extra) => {
                    vm.quota_remaining = vm.quota_remaining.saturating_add(extra);
                    vm.exhausted = 0;
                    Ok(())
                }
                None => Err(WasmTrap::HostAbort),
            }
        }

        fn request_capability(
            &mut self,
            vm: &mut SubstituteVm,
        ) -> std::result::Result<(), WasmTrap> {
            self.capability_calls += 1;
            if vm.has_heap_ptr {
                vm.heap_ptr = vm.heap_ptr.saturating_add(8);
            }
            Ok(())
        }
    }

    #[crate::ctb_test]
    fn test_deduct_resume_target_and_suspension() {
        let mut vm = SubstituteVm::new(Vec::new(), Vec::new(), Vec::new(), Vec::new());
        vm.quota_remaining = 5;
        vm.resume_target_op = 3;
        let mut host = MockHost {
            suspend_refill: Some(10),
            suspend_calls: 0,
            last_interrupted: 0,
            last_required: 0,
            capability_calls: 0,
        };

        // Ordinals 1 and 2 are skipped before hitting resume_target_op=3.
        vm.deduct(&mut host, 100).unwrap();
        vm.deduct(&mut host, 100).unwrap();
        assert_eq!(vm.op_ordinal, 2);
        assert_eq!(vm.quota_consumed, 0);
        assert_eq!(vm.quota_remaining, 5);
        assert_eq!(host.suspend_calls, 0);

        // Ordinal 3 requires 8 (> 5) -> suspends once, refills +10 -> 15 - 8 = 7.
        vm.deduct(&mut host, 8).unwrap();
        assert_eq!(host.suspend_calls, 1);
        assert_eq!(host.last_interrupted, 3);
        assert_eq!(host.last_required, 8);
        assert_eq!(vm.quota_remaining, 7);
        assert_eq!(vm.quota_consumed, 8);
    }

    #[crate::ctb_test]
    fn test_memory_and_table_grow_metering_and_caps() {
        let mut vm = SubstituteVm::new(
            vec![WasmLinearMemory::new(1, Some(4))],
            vec![WasmTable::new(2, Some(5))],
            Vec::new(),
            Vec::new(),
        );
        vm.quota_remaining = 1000;
        vm.max_mem_pages = 3;
        vm.phys_max_mem_pages = 2;
        vm.max_table_elems = 4;
        vm.phys_max_table_elems = 3;
        let mut host = MockHost {
            suspend_refill: None,
            suspend_calls: 0,
            last_interrupted: 0,
            last_required: 0,
            capability_calls: 0,
        };

        // Logical denial (> max_mem_pages=3) costs 1 and returns -1.
        assert_eq!(vm.memory_grow(0, 3, &mut host), Ok(-1));
        assert_eq!(vm.quota_consumed, 1);

        // Allowed logically (1+1=2 <= 3) costs 1 + 64*1 = 65 and succeeds (2 <= phys 2).
        assert_eq!(vm.memory_grow(0, 1, &mut host), Ok(1));
        assert_eq!(vm.quota_consumed, 66);

        // Allowed logically (2+1=3 <= 3), deducts 65 first, then hits phys cap (3 > 2).
        assert_eq!(vm.memory_grow(0, 1, &mut host), Err(WasmTrap::Unreachable));
        assert_eq!(vm.exhausted, 2);
        assert_eq!(vm.interrupted_op, 3);
        assert_eq!(vm.quota_consumed, 131);

        // Table logical denial costs 1 and returns -1.
        assert_eq!(vm.table_grow(0, Some(7), 3, &mut host), Ok(-1));
        // Table grow allowed logically (2+1=3 <= 4), costs 1+1=2.
        assert_eq!(vm.table_grow(0, Some(7), 1, &mut host), Ok(2));
        // Next grow allowed logically (3+1=4 <= 4), hits phys cap 3.
        assert_eq!(
            vm.table_grow(0, Some(9), 1, &mut host),
            Err(WasmTrap::Unreachable)
        );
        assert_eq!(vm.exhausted, 3);
        assert_eq!(vm.interrupted_op, 4);
    }

    #[crate::ctb_test]
    fn test_bulk_ops_and_call_depth() {
        let mut vm = SubstituteVm::new(
            vec![WasmLinearMemory::new(1, None)],
            vec![WasmTable::new(4, None)],
            vec![Some(b"hello")],
            vec![Some(&[Some(10), None, Some(20)])],
        );
        let mut host = MockHost {
            suspend_refill: None,
            suspend_calls: 0,
            last_interrupted: 0,
            last_required: 0,
            capability_calls: 0,
        };

        vm.memory_init(0, 0, 16, 1, 3, &mut host).unwrap();
        assert_eq!(&vm.memory(0).unwrap().as_slice()[16..19], b"ell");
        vm.data_drop(0);
        assert_eq!(
            vm.memory_init(0, 0, 0, 0, 1, &mut host),
            Err(WasmTrap::OutOfBoundsMemoryAccess)
        );
        vm.memory_init(0, 0, 0, 0, 0, &mut host).unwrap();

        vm.memory_fill(0, 32, 0xAB, 4, &mut host).unwrap();
        vm.memory_copy(0, 0, 34, 32, 4, &mut host).unwrap();
        assert_eq!(&vm.memory(0).unwrap().as_slice()[32..38], &[0xAB; 6]);

        vm.table_init(0, 0, 1, 0, 3, &mut host).unwrap();
        assert_eq!(vm.tables[0].get(1).unwrap(), Some(10));
        vm.elem_drop(0);
        assert_eq!(
            vm.table_init(0, 0, 0, 0, 1, &mut host),
            Err(WasmTrap::OutOfBoundsTableAccess)
        );
        vm.table_fill(0, 0, Some(55), 2, &mut host).unwrap();
        vm.table_copy(0, 0, 2, 0, 2, &mut host).unwrap();
        assert_eq!(vm.tables[0].get(3).unwrap(), Some(55));

        for _ in 0..MAX_CALL_DEPTH {
            vm.enter_call().unwrap();
        }
        assert_eq!(vm.enter_call(), Err(WasmTrap::CallStackExhausted));
        vm.exit_call();
        vm.enter_call().unwrap();

        assert_eq!(vm.init_active_data(0, 64, b"hi"), Ok(()));
        assert_eq!(vm.memory(0).unwrap().load_bytes(64, 2).unwrap(), b"hi");
        assert_eq!(vm.init_active_elem(0, 0, &[Some(7), None]), Ok(()));
        assert_eq!(vm.call_indirect_target(0, 0), Ok(7));
        assert_eq!(
            vm.call_indirect_target(0, 1),
            Err(WasmTrap::UninitializedElement)
        );
        assert_eq!(
            vm.call_indirect_target(0, 99),
            Err(WasmTrap::UndefinedElement)
        );
        assert_eq!(vm.table_set(0, 1, Some(12)), Ok(()));
        assert_eq!(vm.table_get(0, 1), Ok(Some(12)));
        assert_eq!(
            vm.table_get(0, 99),
            Err(WasmTrap::OutOfBoundsTableAccess)
        );
        assert_eq!(vm.memory_mut(0).unwrap().grow_host(1), Some(1));
    }

    #[crate::ctb_test]
    fn test_mem0_fast_path_extra_memories_and_typed_globals() {
        fn sample_init(
            vm: &mut SubstituteVm,
            _host: &mut dyn WasmSubstituteHost,
        ) -> std::result::Result<(), WasmTrap> {
            let cur = vm.get_i32(0)?;
            vm.set_i32(0, cur.wrapping_add(42))?;
            Ok(())
        }

        let mut vm = SubstituteVm::new(
            vec![
                WasmLinearMemory::new(1, Some(3)),
                WasmLinearMemory::new(2, Some(4)),
            ],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );
        let mut host = MockHost {
            suspend_refill: None,
            suspend_calls: 0,
            last_interrupted: 0,
            last_required: 0,
            capability_calls: 0,
        };

        assert_eq!(vm.memory_size(0), Ok(1));
        assert_eq!(vm.memory_size(1), Ok(2));
        assert_eq!(vm.memory_size(2), Err(WasmTrap::OutOfBoundsMemoryAccess));
        assert!(vm.memory(0).is_some());
        assert!(vm.memory(1).is_some());
        assert!(vm.memory(2).is_none());

        vm.store_bytes_n(0, 10, 0, [0xDE, 0xAD, 0xBE, 0xEF]).unwrap();
        assert_eq!(
            vm.load_bytes_n::<4>(0, 10, 0),
            Ok([0xDE, 0xAD, 0xBE, 0xEF])
        );
        assert_eq!(vm.load_u8(0, 10, 1), Ok(0xAD));

        // Copy across mem0 -> mem1 and read back from extra_memories.
        vm.memory_copy(1, 0, 128, 10, 4, &mut host).unwrap();
        assert_eq!(
            vm.load_bytes_n::<4>(1, 128, 0),
            Ok([0xDE, 0xAD, 0xBE, 0xEF])
        );
        assert_eq!(vm.load_u8(1, 128, 2), Ok(0xBE));
        assert_eq!(vm.load_u8(2, 0, 0), Err(WasmTrap::OutOfBoundsMemoryAccess));
        assert_eq!(
            vm.memory_grow(1, 1, &mut host),
            Ok(2)
        );
        assert_eq!(vm.memory_size(1), Ok(3));
        assert_eq!(
            vm.memory_grow(2, 1, &mut host),
            Err(WasmTrap::OutOfBoundsMemoryAccess)
        );

        // Typed globals + init_fn + heap_ptr.
        assert_eq!(vm.call_init(&mut host), Err(WasmTrap::Unreachable));
        vm.globals_i32 = vec![10];
        vm.globals_i64 = vec![100];
        vm.globals_f32 = vec![1.5];
        vm.globals_f64 = vec![2.5];
        vm.globals_ref = vec![Some(7)];
        vm.has_heap_ptr = true;
        vm.heap_ptr = 256;
        vm.init_fn = Some(InitFnPtr(sample_init));

        vm.call_init(&mut host).unwrap();
        assert_eq!(vm.get_i32(0), Ok(52));
        vm.set_i64(0, -99).unwrap();
        assert_eq!(vm.get_i64(0), Ok(-99));
        vm.set_f32(0, -3.25).unwrap();
        assert_eq!(vm.get_f32(0), Ok(-3.25));
        vm.set_f64(0, 9.75).unwrap();
        assert_eq!(vm.get_f64(0), Ok(9.75));
        vm.set_ref(0, None).unwrap();
        assert_eq!(vm.get_ref(0), Ok(None));

        assert_eq!(vm.get_i32(1), Err(WasmTrap::Unreachable));
        assert_eq!(vm.set_i32(1, 5), Err(WasmTrap::Unreachable));
        assert_eq!(vm.get_i64(1), Err(WasmTrap::Unreachable));
        assert_eq!(vm.set_f32(1, 0.0), Err(WasmTrap::Unreachable));
        assert_eq!(vm.get_f64(1), Err(WasmTrap::Unreachable));
        assert_eq!(vm.set_ref(1, None), Err(WasmTrap::Unreachable));

        host.request_capability(&mut vm).unwrap();
        assert_eq!(host.capability_calls, 1);
        assert_eq!(vm.heap_ptr, 264);
    }
}
