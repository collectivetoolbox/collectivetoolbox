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

//! Generated native Rust WebAssembly substitute for `dc_1936`.
//! Do not edit by hand; regenerate via `./scripts/generate-wasm-substitutes`.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

#[allow(
    unused_imports,
    reason = "Generated substitute runtime imports"
)]
use ctb_wasm_substitute_rt::{
    InitFnPtr, SubstituteModuleMeta, SubstituteVm, WasmLinearMemory,
    WasmSubstituteDescriptor, WasmSubstituteHost, WasmSubstituteInstance,
    WasmTable, WasmTrap, ops,
};

#[allow(
dead_code,
unused_mut,
unused_variables,
unused_assignments,
unreachable_code,
clippy::too_many_lines,
clippy::cognitive_complexity,
clippy::needless_continue,
clippy::redundant_else,
clippy::collapsible_if,
clippy::collapsible_else_if,
clippy::unnecessary_wraps,
clippy::match_same_arms,
clippy::single_match_else,
clippy::too_many_arguments,
clippy::let_and_return,
clippy::manual_let_else,
clippy::shadow_unrelated,
clippy::type_complexity,
clippy::almost_swapped,
reason = "Generated WASM-to-Rust substitute function"
)]
struct SubstituteInstance {
    vm: SubstituteVm,
}

const MODULE_META: SubstituteModuleMeta = SubstituteModuleMeta {
    name: "dc_1936",
    accounting_version: 2u32,
    wasm_sha256: [0x74, 0x2d, 0x47, 0x65, 0xc8, 0x22, 0x47, 0xdf, 0xee, 0x54, 0x91, 0xa0, 0xc9, 0x3d, 0xac, 0x58, 0x7c, 0x55, 0x87, 0x2c, 0xba, 0xc5, 0x71, 0x5f, 0x98, 0x0f, 0xab, 0xa6, 0xe0, 0x65, 0xd4, 0xa0],
    entry_func_name: "main",
    has_start_func: false,
    initial_memory_pages: 1usize,
    initial_table_elements: 0usize,
    static_data_end: 0usize,
    num_memories: 1usize,
    num_tables: 0usize,
};


#[allow(
dead_code,
unused_mut,
unused_variables,
unused_assignments,
unreachable_code,
clippy::too_many_lines,
clippy::cognitive_complexity,
clippy::needless_continue,
clippy::redundant_else,
clippy::collapsible_if,
clippy::collapsible_else_if,
clippy::unnecessary_wraps,
clippy::match_same_arms,
clippy::single_match_else,
clippy::too_many_arguments,
clippy::let_and_return,
clippy::manual_let_else,
clippy::shadow_unrelated,
clippy::type_complexity,
clippy::almost_swapped,
reason = "Generated WASM-to-Rust substitute function"
)]
fn instantiate_substitute() -> Box<dyn WasmSubstituteInstance> {
    let memories = vec![
        WasmLinearMemory::new(1u32, None),
    ];
    let tables = vec![
    ];
    let passive_data: Vec<Option<&'static [u8]>> = vec![
    ];
    let passive_elems: Vec<Option<&'static [Option<u32>]>> = vec![
    ];
    let mut vm = SubstituteVm::new(memories, tables, passive_data, passive_elems);
    Box::new(SubstituteInstance { vm })
}

#[allow(
dead_code,
unused_mut,
unused_variables,
unused_assignments,
unreachable_code,
clippy::too_many_lines,
clippy::cognitive_complexity,
clippy::needless_continue,
clippy::redundant_else,
clippy::collapsible_if,
clippy::collapsible_else_if,
clippy::unnecessary_wraps,
clippy::match_same_arms,
clippy::single_match_else,
clippy::too_many_arguments,
clippy::let_and_return,
clippy::manual_let_else,
clippy::shadow_unrelated,
clippy::type_complexity,
clippy::almost_swapped,
reason = "Generated WASM-to-Rust substitute function"
)]
impl SubstituteInstance {
    #[inline(never)]
    fn fn_0(vm: &mut SubstituteVm, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = 0i32;
    let mut l1: i32 = 0i32;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i32_2: i32 = 0i32;
    vm.deduct(host, 1)?;
    s_i32_0 = 0i32;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    l0 = s_i32_0;
    s_i32_0 = l0;
    s_i32_1 = 4i32;
    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 0u32)?;
    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
    l1 = s_i32_0;
    s_i32_0 = l1;
    l4 = s_i32_0;
    s_i32_0 = 8i32;
    s_i32_1 = l4;
    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l0;
    s_i32_1 = l1;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 64i32;
            s_i32_1 = 2i32;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = 68i32;
            s_i64_1 = 10000000i64;
            ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
            s_i32_0 = 76i32;
            s_i64_1 = 256i64;
            ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
            s_i32_0 = 84i32;
            s_i64_1 = -1i64;
            ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
            s_i32_0 = 92i32;
            s_i64_1 = 65536i64;
            ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
            s_i32_0 = 100i32;
            s_i64_1 = -1i64;
            ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
            s_i32_0 = 108i32;
            s_i64_1 = 4194304i64;
            ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
            s_i32_0 = 116i32;
            s_i64_1 = 16777216i64;
            ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
            s_i32_0 = 124i32;
            s_i64_1 = 1048576i64;
            ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
            s_i32_0 = 132i32;
            s_i64_1 = 16777216i64;
            ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
        } else {
            s_i32_0 = 0i32;
            l2 = s_i32_0;
            'blk_2: {
                'blk_3: loop {
                    vm.deduct(host, 1)?;
                    s_i32_0 = l0;
                    s_i32_1 = l1;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                    if s_i32_0 != 0 {
                        break 'blk_2;
                    }
                    s_i32_0 = l2;
                    s_i32_1 = 76i32;
                    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                    'blk_4: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = l0;
                    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
                    l3 = s_i32_0;
                    s_i32_0 = l3;
                    s_i32_1 = 128i32;
                    s_i32_0 = ops::i32_lt_u(s_i32_0, s_i32_1);
                    'blk_5: {
                        if s_i32_0 != 0 {
                            s_i32_0 = l0;
                            s_i32_1 = 1i32;
                            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                            l0 = s_i32_0;
                        } else {
                            s_i32_0 = l0;
                            s_i32_1 = 2i32;
                            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                            s_i32_1 = l1;
                            s_i32_0 = ops::i32_gt_u(s_i32_0, s_i32_1);
                            'blk_6: {
                                if s_i32_0 != 0 {
                                    return Err(WasmTrap::Unreachable);
                                }
                            }
                            s_i32_0 = l3;
                            s_i32_1 = 254i32;
                            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                            s_i32_1 = 194i32;
                            s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
                            'blk_7: {
                                if s_i32_0 != 0 {
                                    return Err(WasmTrap::Unreachable);
                                }
                            }
                            s_i32_0 = l3;
                            s_i32_1 = 3i32;
                            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                            s_i32_1 = 6i32;
                            s_i32_0 = ops::i32_shl(s_i32_0, s_i32_1);
                            s_i32_1 = l0;
                            s_i32_1 = ops::i32_load8_u(vm, 0, s_i32_1, 1u32)?;
                            s_i32_2 = 63i32;
                            s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
                            s_i32_0 = ops::i32_or(s_i32_0, s_i32_1);
                            l3 = s_i32_0;
                            s_i32_0 = l0;
                            s_i32_1 = 2i32;
                            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                            l0 = s_i32_0;
                        }
                    }
                    s_i32_0 = l2;
                    s_i32_1 = l3;
                    ops::i32_store8(vm, 0, s_i32_0, 64u32, s_i32_1)?;
                    s_i32_0 = l2;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    l2 = s_i32_0;
                    continue 'blk_3;
                    break 'blk_3;
                }
            }
            s_i32_0 = l2;
            s_i32_1 = 76i32;
            s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
            'blk_8: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
        }
    }
    s_i32_0 = 64i32;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    s_i32_1 = 2i32;
    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
    'blk_9: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = 68i32;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 0u32)?;
    s_i32_0 = ops::i64_eqz(s_i64_0);
    'blk_10: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = 76i32;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 0u32)?;
    s_i32_0 = ops::i64_eqz(s_i64_0);
    'blk_11: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = 0i32;
    l2 = s_i32_0;
    'blk_12: {
        'blk_13: loop {
            vm.deduct(host, 1)?;
            s_i32_0 = l2;
            s_i32_1 = 76i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            if s_i32_0 != 0 {
                break 'blk_12;
            }
            s_i32_0 = l2;
            s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 64u32)?;
            l3 = s_i32_0;
            s_i32_0 = l3;
            s_i32_1 = 128i32;
            s_i32_0 = ops::i32_lt_u(s_i32_0, s_i32_1);
            'blk_14: {
                if s_i32_0 != 0 {
                    s_i32_0 = l4;
                    s_i32_1 = l3;
                    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l4;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    l4 = s_i32_0;
                } else {
                    s_i32_0 = l4;
                    s_i32_1 = l3;
                    s_i32_2 = 6i32;
                    s_i32_1 = ops::i32_shr_u(s_i32_1, s_i32_2);
                    s_i32_2 = 192i32;
                    s_i32_1 = ops::i32_or(s_i32_1, s_i32_2);
                    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l4;
                    s_i32_1 = l3;
                    s_i32_2 = 63i32;
                    s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
                    s_i32_2 = 128i32;
                    s_i32_1 = ops::i32_or(s_i32_1, s_i32_2);
                    ops::i32_store8(vm, 0, s_i32_0, 1u32, s_i32_1)?;
                    s_i32_0 = l4;
                    s_i32_1 = 2i32;
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    l4 = s_i32_0;
                }
            }
            s_i32_0 = l2;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l2 = s_i32_0;
            continue 'blk_13;
            break 'blk_13;
        }
    }
    s_i32_0 = 12i32;
    s_i32_1 = l4;
    s_i32_2 = l1;
    s_i32_1 = ops::i32_sub(s_i32_1, s_i32_2);
    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = 0i32;
    return Ok(s_i32_0);
    }

}

#[allow(
dead_code,
unused_mut,
unused_variables,
unused_assignments,
unreachable_code,
clippy::too_many_lines,
clippy::cognitive_complexity,
clippy::needless_continue,
clippy::redundant_else,
clippy::collapsible_if,
clippy::collapsible_else_if,
clippy::unnecessary_wraps,
clippy::match_same_arms,
clippy::single_match_else,
clippy::too_many_arguments,
clippy::let_and_return,
clippy::manual_let_else,
clippy::shadow_unrelated,
clippy::type_complexity,
clippy::almost_swapped,
reason = "Generated WASM-to-Rust substitute function"
)]
impl WasmSubstituteInstance for SubstituteInstance {
    fn meta(&self) -> SubstituteModuleMeta {
        MODULE_META
    }

    fn vm(&self) -> &SubstituteVm {
        &self.vm
    }

    fn vm_mut(&mut self) -> &mut SubstituteVm {
        &mut self.vm
    }

    fn exported_memory_index(&self) -> Option<usize> {
        Some(0usize)
    }

    fn exported_global_i32(&self, name: &str) -> Option<i32> {
        match name {
            _ => None,
        }
    }

    fn exported_global_i64(&self, name: &str) -> Option<i64> {
        match name {
            _ => None,
        }
    }

    fn set_heap_ptr(&mut self, _val: i32) -> bool {
        false
    }

    fn call_start(&mut self, _host: &mut dyn WasmSubstituteHost) -> std::result::Result<(), WasmTrap> {
        Ok(())
    }

    fn call_entry(&mut self, in_ptr: i32, in_len: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
        let _ = (in_ptr, in_len);
        let vm = &mut self.vm;
        vm.enter_call()?;
        let res = SubstituteInstance::fn_0(vm, host);
        vm.exit_call();
        res
    }
}

/// Returns the native WebAssembly substitute descriptor for `dc_1936`.
#[must_use]
pub fn descriptor() -> WasmSubstituteDescriptor {
    WasmSubstituteDescriptor {
        meta: MODULE_META,
        instantiate: instantiate_substitute,
    }
}
