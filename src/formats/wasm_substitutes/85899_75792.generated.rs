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

//! Generated native Rust WebAssembly substitute chunk `85899_75792`.
//! Do not edit by hand; regenerate via `./scripts/generate-wasm-substitutes`.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

#[allow(
    unused_imports,
    reason = "Generated substitute chunk runtime imports"
)]
use ctb_wasm_substitute_rt::{
    SubstituteModuleMeta, SubstituteVm, WasmLinearMemory,
    WasmSubstituteDescriptor, WasmSubstituteHost, WasmSubstituteInstance,
    WasmTable, WasmTrap, ops,
};

/// Signature-normalized deduplicated WebAssembly functions first defined in this `.ctbnodes` chunk.
pub(crate) struct ChunkFns;

#[allow(
dead_code,
non_camel_case_types,
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
impl ChunkFns {
    pub(crate) fn f_ca11fc7d2c3e(vm: &mut SubstituteVm, dispatch: &'static crate::ModuleDispatch, g_map: &'static [usize], c_map: &'static [u32], host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    s_i32_0 = 8i32;
    s_i32_1 = 0i32;
    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 0u32)?;
    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = 12i32;
    s_i32_1 = 4i32;
    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 0u32)?;
    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = 0i32;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn f_6002b0b90294(vm: &mut SubstituteVm, dispatch: &'static crate::ModuleDispatch, g_map: &'static [usize], c_map: &'static [u32], host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = 0i32;
    let mut l1: i32 = 0i32;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i32_2: i32 = 0i32;
    let mut s_i32_3: i32 = 0i32;
    vm.deduct(host, 1)?;
    s_i32_0 = 4i32;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    l1 = s_i32_0;
    s_i32_0 = l1;
    s_i32_1 = 29i32;
    s_i32_0 = ops::i32_lt_u(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = 0i32;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    l0 = s_i32_0;
    s_i32_0 = l0;
    s_i32_1 = l1;
    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
    l3 = s_i32_0;
    s_i32_0 = l3;
    l2 = s_i32_0;
    s_i32_0 = l3;
    s_i32_1 = 8i32;
    s_i32_0 = ops::i32_sub(s_i32_0, s_i32_1);
    l1 = s_i32_0;
    s_i32_0 = 8i32;
    s_i32_1 = l3;
    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
    l4 = s_i32_0;
    s_i32_0 = l4;
    s_i32_1 = 128i32;
    s_i32_0 = ops::i32_lt_u(s_i32_0, s_i32_1);
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
        } else {
            s_i32_0 = l4;
            s_i32_1 = 255i32;
            s_i32_0 = ops::i32_lt_u(s_i32_0, s_i32_1);
            'blk_3: {
                if s_i32_0 != 0 {
                    s_i32_0 = 255i32;
                    s_i32_1 = l4;
                    s_i32_0 = ops::i32_sub(s_i32_0, s_i32_1);
                    s_i32_1 = 24i32;
                    s_i32_0 = ops::i32_shl(s_i32_0, s_i32_1);
                    s_i32_0 = ops::i32_clz(s_i32_0);
                } else {
                    s_i32_0 = 8i32;
                    s_i32_1 = l0;
                    s_i32_1 = ops::i32_load8_u(vm, 0, s_i32_1, 1u32)?;
                    s_i32_2 = 2i32;
                    s_i32_1 = ops::i32_shr_u(s_i32_1, s_i32_2);
                    s_i32_2 = 15i32;
                    s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                }
            }
        }
    }
    l5 = s_i32_0;
    s_i32_0 = l0;
    s_i32_1 = l5;
    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
    l0 = s_i32_0;
    s_i32_0 = l0;
    s_i32_1 = 20i32;
    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
    s_i32_1 = l1;
    s_i32_0 = ops::i32_gt_u(s_i32_0, s_i32_1);
    'blk_4: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    s_i32_1 = -2105241356i32;
    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
    'blk_5: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    s_i32_1 = -2071686924i32;
    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
    'blk_6: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
    s_i32_1 = -2038132492i32;
    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
    'blk_7: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 12u32)?;
    s_i32_1 = -2004578060i32;
    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
    'blk_8: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 16u32)?;
    s_i32_1 = -2021355276i32;
    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
    'blk_9: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = l1;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    s_i32_1 = -2054909708i32;
    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
    'blk_10: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = l1;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    s_i32_1 = -2088464140i32;
    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
    'blk_11: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = l0;
    s_i32_1 = 20i32;
    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
    l0 = s_i32_0;
    'blk_12: {
        'blk_13: loop {
            vm.deduct(host, 1)?;
            s_i32_0 = l0;
            s_i32_1 = l1;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            if s_i32_0 != 0 {
                break 'blk_12;
            }
            s_i32_0 = l0;
            s_i32_1 = l1;
            s_i32_0 = ops::i32_gt_u(s_i32_0, s_i32_1);
            'blk_14: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = l0;
            s_i32_1 = 4i32;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            s_i32_1 = l1;
            s_i32_0 = ops::i32_le_u(s_i32_0, s_i32_1);
            'blk_15: {
                if s_i32_0 != 0 {
                    s_i32_0 = l0;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                    s_i32_1 = -2054909708i32;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                    'blk_16: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = l0;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                    s_i32_1 = -1081896716i32;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                    'blk_17: {
                        if s_i32_0 != 0 {
                            s_i32_0 = l0;
                            s_i32_1 = 4i32;
                            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                            l0 = s_i32_0;
                            s_i32_0 = l0;
                            s_i32_1 = l1;
                            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                            'blk_18: {
                                if s_i32_0 != 0 {
                                    return Err(WasmTrap::Unreachable);
                                }
                            }
                        }
                    }
                }
            }
            s_i32_0 = l0;
            s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
            l4 = s_i32_0;
            s_i32_0 = l4;
            s_i32_1 = 128i32;
            s_i32_0 = ops::i32_lt_u(s_i32_0, s_i32_1);
            'blk_19: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                } else {
                    s_i32_0 = l4;
                    s_i32_1 = 255i32;
                    s_i32_0 = ops::i32_lt_u(s_i32_0, s_i32_1);
                    'blk_20: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 255i32;
                            s_i32_1 = l4;
                            s_i32_0 = ops::i32_sub(s_i32_0, s_i32_1);
                            s_i32_1 = 24i32;
                            s_i32_0 = ops::i32_shl(s_i32_0, s_i32_1);
                            s_i32_0 = ops::i32_clz(s_i32_0);
                        } else {
                            s_i32_0 = 8i32;
                            s_i32_1 = l0;
                            s_i32_1 = ops::i32_load8_u(vm, 0, s_i32_1, 1u32)?;
                            s_i32_2 = 2i32;
                            s_i32_1 = ops::i32_shr_u(s_i32_1, s_i32_2);
                            s_i32_2 = 15i32;
                            s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
                            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                        }
                    }
                }
            }
            l5 = s_i32_0;
            s_i32_0 = l0;
            s_i32_1 = l5;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            s_i32_1 = l1;
            s_i32_0 = ops::i32_gt_u(s_i32_0, s_i32_1);
            'blk_21: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = l5;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            s_i32_1 = l4;
            s_i32_2 = 10i32;
            s_i32_1 = ops::i32_eq(s_i32_1, s_i32_2);
            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
            s_i32_1 = l5;
            s_i32_2 = 4i32;
            s_i32_1 = ops::i32_eq(s_i32_1, s_i32_2);
            s_i32_2 = l0;
            s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 0u32)?;
            s_i32_3 = -1199468300i32;
            s_i32_2 = ops::i32_eq(s_i32_2, s_i32_3);
            s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
            s_i32_0 = ops::i32_or(s_i32_0, s_i32_1);
            l6 = s_i32_0;
            'blk_22: {
                'blk_23: loop {
                    vm.deduct(host, 1)?;
                    s_i32_0 = l5;
                    s_i32_0 = ops::i32_eqz(s_i32_0);
                    if s_i32_0 != 0 {
                        break 'blk_22;
                    }
                    s_i32_0 = l2;
                    s_i32_1 = l0;
                    s_i32_1 = ops::i32_load8_u(vm, 0, s_i32_1, 0u32)?;
                    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l0;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    l0 = s_i32_0;
                    s_i32_0 = l2;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    l2 = s_i32_0;
                    s_i32_0 = l5;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_sub(s_i32_0, s_i32_1);
                    l5 = s_i32_0;
                    continue 'blk_23;
                    break 'blk_23;
                }
            }
            continue 'blk_13;
            break 'blk_13;
        }
    }
    s_i32_0 = l6;
    s_i32_0 = ops::i32_eqz(s_i32_0);
    'blk_24: {
        if s_i32_0 != 0 {
            s_i32_0 = l2;
            s_i32_1 = 10i32;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l2;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l2 = s_i32_0;
        }
    }
    s_i32_0 = 12i32;
    s_i32_1 = l2;
    s_i32_2 = l3;
    s_i32_1 = ops::i32_sub(s_i32_1, s_i32_2);
    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = 0i32;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn f_5c2d9ee25d4f(vm: &mut SubstituteVm, dispatch: &'static crate::ModuleDispatch, g_map: &'static [usize], c_map: &'static [u32], host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
            s_i32_1 = 1i32;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = 68i32;
            s_i64_1 = 1000000i64;
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
    s_i32_1 = 1i32;
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
non_camel_case_types,
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
pub(crate) mod desc_0 {
#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;
#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Generated descriptor submodule imports"
)]
use super::*;

#[allow(
dead_code,
non_camel_case_types,
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
static BOUND_SIG_P_UNIT_RET_I32: &[crate::Bound_p_unit_ret_i32] = &[
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_75792::ChunkFns::f_ca11fc7d2c3e, globals: &[], callees: &[] },
];

#[allow(
dead_code,
non_camel_case_types,
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
static DISPATCH: crate::ModuleDispatch = crate::ModuleDispatch {
    sig_p_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_ret_i32: &[],
    sig_p_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i32_i32_i32_ret_i32: &[],
    sig_p_i32_i32_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i32_i32_ret_i32: &[],
    sig_p_i32_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i32_ret_i32: &[],
    sig_p_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i64_i32_i32_i32_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i64_i32_i32_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i64_i32_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i64_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_ret_i32: &[],
    sig_p_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_ret_i32: &[],
    sig_p_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i64_ret_void: &[],
    sig_p_i32_i32_i32_ret_i32: &[],
    sig_p_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i64_i32_i32_ret_i32: &[],
    sig_p_i32_i32_i64_ret_void: &[],
    sig_p_i32_i32_ret_i32: &[],
    sig_p_i32_i32_ret_i64: &[],
    sig_p_i32_i32_ret_void: &[],
    sig_p_i32_i64_i32_i32_ret_i32: &[],
    sig_p_i32_i64_i32_i32_ret_i64: &[],
    sig_p_i32_i64_i32_ret_i32: &[],
    sig_p_i32_i64_i32_ret_i64: &[],
    sig_p_i32_i64_i32_ret_void: &[],
    sig_p_i32_i64_i64_ret_i32: &[],
    sig_p_i32_i64_i64_ret_i64: &[],
    sig_p_i32_i64_i64_ret_void: &[],
    sig_p_i32_i64_ret_i32: &[],
    sig_p_i32_i64_ret_i64: &[],
    sig_p_i32_i64_ret_void: &[],
    sig_p_i32_ret_i32: &[],
    sig_p_i32_ret_i64: &[],
    sig_p_i32_ret_void: &[],
    sig_p_i64_i32_ret_i64: &[],
    sig_p_i64_i64_ret_i32: &[],
    sig_p_i64_i64_ret_i64: &[],
    sig_p_i64_ret_i32: &[],
    sig_p_i64_ret_i64: &[],
    sig_p_unit_ret_i32: BOUND_SIG_P_UNIT_RET_I32,
    sig_p_unit_ret_void: &[],
};

#[allow(
dead_code,
non_camel_case_types,
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
    name: "85899_75792/1936@l8589975814",
    accounting_version: 2u32,
    wasm_sha256: [0x27, 0xdb, 0x95, 0x90, 0xed, 0x4b, 0x26, 0xd9, 0x48, 0xbb, 0xf2, 0x05, 0x52, 0xb7, 0x38, 0x4c, 0xb2, 0x41, 0x48, 0x8a, 0x8a, 0x40, 0xdd, 0x16, 0x6a, 0xab, 0x1b, 0xff, 0xd4, 0x89, 0xd2, 0xe4],
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
non_camel_case_types,
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
non_camel_case_types,
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
        let res = DISPATCH.call_direct_p_unit_ret_i32(vm, 0usize, host);
        vm.exit_call();
        res
    }
}

#[must_use]
pub(crate) fn descriptor() -> WasmSubstituteDescriptor {
    WasmSubstituteDescriptor {
        meta: MODULE_META,
        instantiate: instantiate_substitute,
    }
}
}

#[allow(
dead_code,
non_camel_case_types,
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
pub(crate) mod desc_1 {
#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;
#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Generated descriptor submodule imports"
)]
use super::*;

#[allow(
dead_code,
non_camel_case_types,
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
static BOUND_SIG_P_UNIT_RET_I32: &[crate::Bound_p_unit_ret_i32] = &[
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_75792::ChunkFns::f_6002b0b90294, globals: &[], callees: &[] },
];

#[allow(
dead_code,
non_camel_case_types,
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
static DISPATCH: crate::ModuleDispatch = crate::ModuleDispatch {
    sig_p_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_ret_i32: &[],
    sig_p_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i32_i32_i32_ret_i32: &[],
    sig_p_i32_i32_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i32_i32_ret_i32: &[],
    sig_p_i32_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i32_ret_i32: &[],
    sig_p_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i64_i32_i32_i32_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i64_i32_i32_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i64_i32_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i64_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_ret_i32: &[],
    sig_p_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_ret_i32: &[],
    sig_p_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i64_ret_void: &[],
    sig_p_i32_i32_i32_ret_i32: &[],
    sig_p_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i64_i32_i32_ret_i32: &[],
    sig_p_i32_i32_i64_ret_void: &[],
    sig_p_i32_i32_ret_i32: &[],
    sig_p_i32_i32_ret_i64: &[],
    sig_p_i32_i32_ret_void: &[],
    sig_p_i32_i64_i32_i32_ret_i32: &[],
    sig_p_i32_i64_i32_i32_ret_i64: &[],
    sig_p_i32_i64_i32_ret_i32: &[],
    sig_p_i32_i64_i32_ret_i64: &[],
    sig_p_i32_i64_i32_ret_void: &[],
    sig_p_i32_i64_i64_ret_i32: &[],
    sig_p_i32_i64_i64_ret_i64: &[],
    sig_p_i32_i64_i64_ret_void: &[],
    sig_p_i32_i64_ret_i32: &[],
    sig_p_i32_i64_ret_i64: &[],
    sig_p_i32_i64_ret_void: &[],
    sig_p_i32_ret_i32: &[],
    sig_p_i32_ret_i64: &[],
    sig_p_i32_ret_void: &[],
    sig_p_i64_i32_ret_i64: &[],
    sig_p_i64_i64_ret_i32: &[],
    sig_p_i64_i64_ret_i64: &[],
    sig_p_i64_ret_i32: &[],
    sig_p_i64_ret_i64: &[],
    sig_p_unit_ret_i32: BOUND_SIG_P_UNIT_RET_I32,
    sig_p_unit_ret_void: &[],
};

#[allow(
dead_code,
non_camel_case_types,
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
    name: "85899_75792/256@l8589975828",
    accounting_version: 2u32,
    wasm_sha256: [0x17, 0x13, 0x89, 0xb0, 0x16, 0x86, 0x7c, 0x0b, 0xe2, 0xc7, 0x3b, 0xd2, 0xb5, 0x17, 0x1f, 0xa6, 0xf6, 0x3b, 0x2f, 0x38, 0x92, 0x21, 0xbd, 0xd5, 0xbd, 0xbd, 0xa3, 0x79, 0x30, 0xcd, 0x8c, 0x7f],
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
non_camel_case_types,
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
non_camel_case_types,
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
        let res = DISPATCH.call_direct_p_unit_ret_i32(vm, 0usize, host);
        vm.exit_call();
        res
    }
}

#[must_use]
pub(crate) fn descriptor() -> WasmSubstituteDescriptor {
    WasmSubstituteDescriptor {
        meta: MODULE_META,
        instantiate: instantiate_substitute,
    }
}
}

#[allow(
dead_code,
non_camel_case_types,
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
pub(crate) mod desc_2 {
#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;
#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Generated descriptor submodule imports"
)]
use super::*;

#[allow(
dead_code,
non_camel_case_types,
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
static BOUND_SIG_P_UNIT_RET_I32: &[crate::Bound_p_unit_ret_i32] = &[
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_75792::ChunkFns::f_5c2d9ee25d4f, globals: &[], callees: &[] },
];

#[allow(
dead_code,
non_camel_case_types,
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
static DISPATCH: crate::ModuleDispatch = crate::ModuleDispatch {
    sig_p_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_ret_i32: &[],
    sig_p_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i32_i32_i32_ret_i32: &[],
    sig_p_i32_i32_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i32_i32_ret_i32: &[],
    sig_p_i32_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i32_ret_i32: &[],
    sig_p_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i64_i32_i32_i32_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i64_i32_i32_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i64_i32_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i64_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_ret_i32: &[],
    sig_p_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_ret_i32: &[],
    sig_p_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i64_ret_void: &[],
    sig_p_i32_i32_i32_ret_i32: &[],
    sig_p_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i64_i32_i32_ret_i32: &[],
    sig_p_i32_i32_i64_ret_void: &[],
    sig_p_i32_i32_ret_i32: &[],
    sig_p_i32_i32_ret_i64: &[],
    sig_p_i32_i32_ret_void: &[],
    sig_p_i32_i64_i32_i32_ret_i32: &[],
    sig_p_i32_i64_i32_i32_ret_i64: &[],
    sig_p_i32_i64_i32_ret_i32: &[],
    sig_p_i32_i64_i32_ret_i64: &[],
    sig_p_i32_i64_i32_ret_void: &[],
    sig_p_i32_i64_i64_ret_i32: &[],
    sig_p_i32_i64_i64_ret_i64: &[],
    sig_p_i32_i64_i64_ret_void: &[],
    sig_p_i32_i64_ret_i32: &[],
    sig_p_i32_i64_ret_i64: &[],
    sig_p_i32_i64_ret_void: &[],
    sig_p_i32_ret_i32: &[],
    sig_p_i32_ret_i64: &[],
    sig_p_i32_ret_void: &[],
    sig_p_i64_i32_ret_i64: &[],
    sig_p_i64_i64_ret_i32: &[],
    sig_p_i64_i64_ret_i64: &[],
    sig_p_i64_ret_i32: &[],
    sig_p_i64_ret_i64: &[],
    sig_p_unit_ret_i32: BOUND_SIG_P_UNIT_RET_I32,
    sig_p_unit_ret_void: &[],
};

#[allow(
dead_code,
non_camel_case_types,
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
    name: "85899_75792/1936@l8589975828",
    accounting_version: 2u32,
    wasm_sha256: [0x43, 0x1d, 0x4e, 0xca, 0x7d, 0xfc, 0xb7, 0x97, 0x38, 0x42, 0x4d, 0x84, 0x28, 0x98, 0x9d, 0xf7, 0x7b, 0x0c, 0x14, 0xd9, 0x3c, 0xcb, 0xc1, 0x95, 0x16, 0x46, 0xb1, 0x98, 0x29, 0x25, 0x76, 0x56],
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
non_camel_case_types,
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
non_camel_case_types,
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
        let res = DISPATCH.call_direct_p_unit_ret_i32(vm, 0usize, host);
        vm.exit_call();
        res
    }
}

#[must_use]
pub(crate) fn descriptor() -> WasmSubstituteDescriptor {
    WasmSubstituteDescriptor {
        meta: MODULE_META,
        instantiate: instantiate_substitute,
    }
}
}

