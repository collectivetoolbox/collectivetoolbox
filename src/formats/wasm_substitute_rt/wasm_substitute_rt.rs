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

//! Safe WebAssembly runtime primitives, V2 structural-checkpoint metering,
//! and Guix-style native substitute registry.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

pub mod ops;
pub mod registry;
pub mod vm;

use std::fmt;

pub use registry::{
    WasmSubstituteDescriptor, lookup_substitute,
    lookup_substitute_by_graph_revision, register_substitute,
};
pub use vm::{
    InitFnPtr, MAX_CALL_DEPTH, SubstituteVm, WASM_PAGE_BYTES, WasmLinearMemory,
    WasmTable,
};

/// Deterministic WebAssembly execution trap raised by native substitutes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmTrap {
    /// Guest executed `unreachable` or a physical growth trap guard.
    Unreachable,
    /// Linear memory access or bulk/init span out of bounds.
    OutOfBoundsMemoryAccess,
    /// Table access or bulk/init span out of bounds.
    OutOfBoundsTableAccess,
    /// Integer division or remainder by zero.
    IntegerDivideByZero,
    /// Signed integer division or truncation overflow.
    IntegerOverflow,
    /// Trapping float-to-integer conversion encountered `NaN`.
    InvalidConversionToInteger,
    /// Indirect call index out of table bounds or null element.
    UndefinedElement,
    /// Indirect call table slot uninitialized (`ref.null`).
    UninitializedElement,
    /// Indirect call table entry signature did not match expected type.
    IndirectCallTypeMismatch,
    /// Non-tail WebAssembly call depth exceeded [`MAX_CALL_DEPTH`].
    CallStackExhausted,
    /// Host aborted suspension or capability continuation.
    HostAbort,
}

impl fmt::Display for WasmTrap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreachable => f.write_str("unreachable executed"),
            Self::OutOfBoundsMemoryAccess => {
                f.write_str("out of bounds memory access")
            }
            Self::OutOfBoundsTableAccess => {
                f.write_str("out of bounds table access")
            }
            Self::IntegerDivideByZero => f.write_str("integer divide by zero"),
            Self::IntegerOverflow => f.write_str("integer overflow"),
            Self::InvalidConversionToInteger => {
                f.write_str("invalid conversion to integer")
            }
            Self::UndefinedElement => f.write_str("undefined element"),
            Self::UninitializedElement => f.write_str("uninitialized element"),
            Self::IndirectCallTypeMismatch => {
                f.write_str("indirect call type mismatch")
            }
            Self::CallStackExhausted => f.write_str("call stack exhausted"),
            Self::HostAbort => f.write_str("host aborted substitute execution"),
        }
    }
}

impl std::error::Error for WasmTrap {}

/// Static metadata describing a transpiled WebAssembly substitute module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubstituteModuleMeta {
    /// Human-readable target label (e.g. `dc_1936`, `f1680`).
    pub name: &'static str,
    /// Resource-accounting rule version matching `WASM_RESOURCE_ACCOUNTING_VERSION`.
    pub accounting_version: u32,
    /// SHA-256 digest of the canonical uninstrumented `.wasm` module bytes.
    pub wasm_sha256: [u8; 32],
    /// Entry function export name (`"step"` preferred, else `"main"`).
    pub entry_func_name: &'static str,
    /// Whether the module declared a WASM `Start` section (`section 8`).
    pub has_start_func: bool,
    /// Initial linear memory pages summed across declared memories.
    pub initial_memory_pages: usize,
    /// Initial table elements summed across declared tables.
    pub initial_table_elements: usize,
    /// Highest byte offset covered by active data segments on memory 0.
    pub static_data_end: usize,
    /// Number of linear memories declared by the module.
    pub num_memories: usize,
    /// Number of tables declared by the module.
    pub num_tables: usize,
    /// Graph `(target_global_id, revision_sha256)` closure keys compiling to this module.
    pub graph_targets: &'static [(u128, [u8; 32])],
}

/// Host suspension callbacks invoked by native substitutes on V2 quota exhaustion
/// and Option B mid-step host capability yields.
pub trait WasmSubstituteHost {
    /// Pauses execution when `vm.quota_remaining < vm.required_cost` until the
    /// host replenishes `vm.quota_remaining` (returning `Ok(())`) or aborts
    /// (`Err(WasmTrap::HostAbort)`).
    fn suspend_on_quota(
        &mut self,
        vm: &mut SubstituteVm,
    ) -> std::result::Result<(), WasmTrap>;

    /// Yields control for an Option B host capability request (`Dc 2616`),
    /// returning once the host writes the capability response into memory `0`.
    fn request_capability(
        &mut self,
        vm: &mut SubstituteVm,
    ) -> std::result::Result<(), WasmTrap>;
}

/// Live instantiated state of a native WebAssembly substitute module.
pub trait WasmSubstituteInstance: Send {
    /// Returns static metadata for this substitute module.
    fn meta(&self) -> SubstituteModuleMeta;

    /// Returns a shared reference to the substitute VM state.
    fn vm(&self) -> &SubstituteVm;

    /// Returns a mutable reference to the substitute VM state.
    fn vm_mut(&mut self) -> &mut SubstituteVm;

    /// Returns the memory index of the exported `"memory"`, if present.
    fn exported_memory_index(&self) -> Option<usize>;

    /// Returns the value of an exported `i32` global by name, if present.
    fn exported_global_i32(&self, name: &str) -> Option<i32>;

    /// Returns the value of an exported `i64` global by name, if present.
    fn exported_global_i64(&self, name: &str) -> Option<i64>;

    /// Returns the guest `__heap_ptr` `i32` global value if exported.
    fn heap_ptr(&self) -> Option<i32> {
        self.exported_global_i32("__heap_ptr")
    }

    /// Updates the guest `__heap_ptr` `i32` global if exported (`true` if updated).
    fn set_heap_ptr(&mut self, val: i32) -> bool;

    /// Executes the module `Start` section under V2 metering, if present.
    fn call_start(
        &mut self,
        host: &mut dyn WasmSubstituteHost,
    ) -> std::result::Result<(), WasmTrap>;

    /// Executes the exported entry function (`"step"` / `"main"`) once.
    fn call_entry(
        &mut self,
        in_ptr: i32,
        in_len: i32,
        host: &mut dyn WasmSubstituteHost,
    ) -> std::result::Result<i32, WasmTrap>;
}
