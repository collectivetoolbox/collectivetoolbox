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

//! Generated native Rust WebAssembly substitute chunk `85899_79092`.
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
    #[inline(never)]
    pub(crate) fn f_0dd72d798dab(vm: &mut SubstituteVm, dispatch: &'static crate::ModuleDispatch, g_map: &'static [usize], c_map: &'static [u32], p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i32 = 0i32;
    let mut l7: i32 = 0i32;
    let mut l8: i32 = 0i32;
    let mut l9: i64 = 0i64;
    let mut l10: i64 = 0i64;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i32_2: i32 = 0i32;
    vm.deduct(host, 1)?;
    s_i32_0 = l0;
    s_i32_1 = l1;
    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
    l2 = s_i32_0;
    s_i32_0 = l2;
    s_i32_1 = 128i32;
    s_i32_0 = ops::i32_lt_u(s_i32_0, s_i32_1);
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = l2;
            vm.set_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?, s_i32_0)?;
            s_i32_0 = 1i32;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l2;
    s_i32_1 = 194i32;
    s_i32_0 = ops::i32_lt_u(s_i32_0, s_i32_1);
    'blk_3: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = l2;
    s_i32_1 = 255i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_4: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i32_1 = 2i32;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            s_i32_1 = l1;
            s_i32_0 = ops::i32_gt_u(s_i32_0, s_i32_1);
            'blk_5: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = l0;
            s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 1u32)?;
            l3 = s_i32_0;
            s_i32_0 = l3;
            s_i32_1 = 192i32;
            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
            s_i32_1 = 128i32;
            s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
            'blk_6: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = l3;
            s_i32_1 = 2i32;
            s_i32_0 = ops::i32_shr_u(s_i32_0, s_i32_1);
            s_i32_1 = 15i32;
            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
            l5 = s_i32_0;
            s_i32_0 = 8i32;
            s_i32_1 = l5;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l6 = s_i32_0;
            s_i32_0 = l0;
            s_i32_1 = l6;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            s_i32_1 = l1;
            s_i32_0 = ops::i32_gt_u(s_i32_0, s_i32_1);
            'blk_7: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = 2i32;
            l7 = s_i32_0;
            'blk_8: {
                'blk_9: loop {
                    vm.deduct(host, 1)?;
                    s_i32_0 = l7;
                    s_i32_1 = l6;
                    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                    if s_i32_0 != 0 {
                        break 'blk_8;
                    }
                    s_i32_0 = l0;
                    s_i32_1 = l7;
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
                    l8 = s_i32_0;
                    s_i32_0 = l8;
                    s_i32_1 = 192i32;
                    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                    s_i32_1 = 128i32;
                    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
                    'blk_10: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = l7;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    l7 = s_i32_0;
                    continue 'blk_9;
                    break 'blk_9;
                }
            }
            s_i32_0 = l0;
            s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 2u32)?;
            l4 = s_i32_0;
            s_i32_0 = l5;
            s_i32_0 = ops::i32_eqz(s_i32_0);
            'blk_11: {
                if s_i32_0 != 0 {
                    s_i32_0 = l3;
                    s_i32_1 = 3i32;
                    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                    s_i32_0 = ops::i32_eqz(s_i32_0);
                    'blk_12: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                } else {
                    s_i32_0 = l3;
                    s_i32_1 = 3i32;
                    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                    s_i32_0 = ops::i32_eqz(s_i32_0);
                    s_i32_1 = l4;
                    s_i32_2 = 60i32;
                    s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
                    s_i32_1 = ops::i32_eqz(s_i32_1);
                    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                    'blk_13: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                }
            }
            s_i32_0 = -1i32;
            vm.set_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?, s_i32_0)?;
            s_i32_0 = l6;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l2;
    s_i32_1 = 224i32;
    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
    s_i32_1 = 192i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_14: {
        if s_i32_0 != 0 {
            s_i32_0 = 2i32;
            l6 = s_i32_0;
            s_i32_0 = l2;
            s_i32_1 = 31i32;
            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
            s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
            l9 = s_i64_0;
            s_i64_0 = 128i64;
            l10 = s_i64_0;
        } else {
            s_i32_0 = l2;
            s_i32_1 = 240i32;
            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
            s_i32_1 = 224i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_15: {
                if s_i32_0 != 0 {
                    s_i32_0 = 3i32;
                    l6 = s_i32_0;
                    s_i32_0 = l2;
                    s_i32_1 = 15i32;
                    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
                    l9 = s_i64_0;
                    s_i64_0 = 2048i64;
                    l10 = s_i64_0;
                } else {
                    s_i32_0 = l2;
                    s_i32_1 = 248i32;
                    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                    s_i32_1 = 240i32;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                    'blk_16: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 4i32;
                            l6 = s_i32_0;
                            s_i32_0 = l2;
                            s_i32_1 = 7i32;
                            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                            s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
                            l9 = s_i64_0;
                            s_i64_0 = 65536i64;
                            l10 = s_i64_0;
                        } else {
                            s_i32_0 = l2;
                            s_i32_1 = 252i32;
                            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                            s_i32_1 = 248i32;
                            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                            'blk_17: {
                                if s_i32_0 != 0 {
                                    s_i32_0 = 5i32;
                                    l6 = s_i32_0;
                                    s_i32_0 = l2;
                                    s_i32_1 = 3i32;
                                    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                                    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
                                    l9 = s_i64_0;
                                    s_i64_0 = 2097152i64;
                                    l10 = s_i64_0;
                                } else {
                                    s_i32_0 = l2;
                                    s_i32_1 = 254i32;
                                    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                                    s_i32_1 = 252i32;
                                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                                    'blk_18: {
                                        if s_i32_0 != 0 {
                                            s_i32_0 = 6i32;
                                            l6 = s_i32_0;
                                            s_i32_0 = l2;
                                            s_i32_1 = 1i32;
                                            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                                            s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
                                            l9 = s_i64_0;
                                            s_i64_0 = 67108864i64;
                                            l10 = s_i64_0;
                                        } else {
                                            s_i32_0 = 7i32;
                                            l6 = s_i32_0;
                                            s_i64_0 = 0i64;
                                            l9 = s_i64_0;
                                            s_i64_0 = 2147483648i64;
                                            l10 = s_i64_0;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    s_i32_0 = l0;
    s_i32_1 = l6;
    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
    s_i32_1 = l1;
    s_i32_0 = ops::i32_gt_u(s_i32_0, s_i32_1);
    'blk_19: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = 1i32;
    l7 = s_i32_0;
    'blk_20: {
        'blk_21: loop {
            vm.deduct(host, 1)?;
            s_i32_0 = l7;
            s_i32_1 = l6;
            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
            if s_i32_0 != 0 {
                break 'blk_20;
            }
            s_i32_0 = l0;
            s_i32_1 = l7;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
            l8 = s_i32_0;
            s_i32_0 = l8;
            s_i32_1 = 192i32;
            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
            s_i32_1 = 128i32;
            s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
            'blk_22: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i64_0 = l9;
            s_i64_1 = 6i64;
            s_i64_0 = ops::i64_shl(s_i64_0, s_i64_1);
            s_i32_1 = l8;
            s_i32_2 = 63i32;
            s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
            s_i64_1 = ops::i64_extend_i32_u(s_i32_1);
            s_i64_0 = ops::i64_or(s_i64_0, s_i64_1);
            l9 = s_i64_0;
            s_i32_0 = l7;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l7 = s_i32_0;
            continue 'blk_21;
            break 'blk_21;
        }
    }
    s_i64_0 = l9;
    s_i64_1 = l10;
    s_i32_0 = ops::i64_lt_u(s_i64_0, s_i64_1);
    'blk_23: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = l6;
    s_i32_1 = 4i32;
    s_i32_0 = ops::i32_le_u(s_i32_0, s_i32_1);
    'blk_24: {
        if s_i32_0 != 0 {
            s_i64_0 = l9;
            s_i32_0 = ops::i32_wrap_i64(s_i64_0);
            vm.set_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?, s_i32_0)?;
        } else {
            s_i32_0 = -1i32;
            vm.set_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?, s_i32_0)?;
        }
    }
    s_i32_0 = l6;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn f_4bd10795638b(vm: &mut SubstituteVm, dispatch: &'static crate::ModuleDispatch, g_map: &'static [usize], c_map: &'static [u32], host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = 0i32;
    let mut l1: i32 = 0i32;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i32 = 0i32;
    let mut l7: i32 = 0i32;
    let mut l8: i32 = 0i32;
    let mut l9: i32 = 0i32;
    let mut l10: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i32_2: i32 = 0i32;
    vm.deduct(host, 1)?;
    s_i32_0 = 0i32;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    l0 = s_i32_0;
    s_i32_0 = 4i32;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    l1 = s_i32_0;
    s_i32_0 = l0;
    s_i32_1 = l1;
    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
    l2 = s_i32_0;
    s_i32_0 = l2;
    s_i32_1 = 7i32;
    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
    s_i32_1 = -8i32;
    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
    l3 = s_i32_0;
    s_i32_0 = l3;
    l4 = s_i32_0;
    s_i32_0 = l3;
    s_i32_1 = l1;
    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
    l6 = s_i32_0;
    s_i32_0 = vm.memory_size(0)?;
    s_i32_1 = 16i32;
    s_i32_0 = ops::i32_shl(s_i32_0, s_i32_1);
    l5 = s_i32_0;
    s_i32_0 = l6;
    s_i32_1 = l5;
    s_i32_0 = ops::i32_gt_u(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = l6;
            s_i32_1 = l5;
            s_i32_0 = ops::i32_sub(s_i32_0, s_i32_1);
            s_i32_1 = 65535i32;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            s_i32_1 = 16i32;
            s_i32_0 = ops::i32_shr_u(s_i32_0, s_i32_1);
            s_i32_0 = vm.memory_grow(0, s_i32_0, host)?;
            s_i32_1 = -1i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_2: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
        }
    }
    s_i32_0 = l0;
    s_i32_1 = l2;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l7 = s_i32_0;
    s_i32_0 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
    s_i32_1 = 1116720i32;
    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
    'blk_3: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = l0;
    s_i32_1 = l7;
    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
    l0 = s_i32_0;
    s_i32_0 = l0;
    s_i32_1 = l2;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t1 = __call_res?;
    s_i32_0 = __t1;
    l7 = s_i32_0;
    s_i32_0 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
    s_i32_1 = 1114370i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_4: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
            l8 = s_i32_0;
            s_i32_0 = l0;
            s_i32_1 = l7;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l0 = s_i32_0;
            s_i32_0 = l0;
            s_i32_1 = l2;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_0 = __t2;
            l7 = s_i32_0;
        } else {
            s_i32_0 = 0i32;
            l8 = s_i32_0;
        }
    }
    s_i32_0 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
    s_i32_1 = 1114372i32;
    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
    'blk_5: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = l0;
    s_i32_1 = l7;
    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
    l0 = s_i32_0;
    s_i32_0 = l0;
    s_i32_1 = l2;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t3 = __call_res?;
    s_i32_0 = __t3;
    l7 = s_i32_0;
    s_i32_0 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
    s_i32_1 = 1114374i32;
    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
    'blk_6: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = l0;
    s_i32_1 = l7;
    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
    l0 = s_i32_0;
    s_i32_0 = 1i32;
    l9 = s_i32_0;
    s_i32_0 = 0i32;
    l10 = s_i32_0;
    'blk_7: {
        'blk_8: loop {
            vm.deduct(host, 1)?;
            s_i32_0 = l0;
            s_i32_1 = l2;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t4 = __call_res?;
            s_i32_0 = __t4;
            l7 = s_i32_0;
            s_i32_0 = l0;
            s_i32_1 = l7;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l0 = s_i32_0;
            s_i32_0 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
            s_i32_1 = 1114367i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_9: {
                if s_i32_0 != 0 {
                    s_i32_0 = l0;
                    s_i32_1 = l2;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t5 = __call_res?;
                    s_i32_0 = __t5;
                    l7 = s_i32_0;
                    s_i32_0 = l0;
                    s_i32_1 = l7;
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    l0 = s_i32_0;
                    s_i32_0 = l10;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    l10 = s_i32_0;
                } else {
                    s_i32_0 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
                    s_i32_1 = 1114374i32;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                    'blk_10: {
                        if s_i32_0 != 0 {
                            s_i32_0 = l9;
                            s_i32_1 = 1i32;
                            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                            l9 = s_i32_0;
                            s_i32_0 = l10;
                            s_i32_1 = 1i32;
                            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                            l10 = s_i32_0;
                        } else {
                            s_i32_0 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
                            s_i32_1 = 1114375i32;
                            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                            'blk_11: {
                                if s_i32_0 != 0 {
                                    s_i32_0 = l9;
                                    s_i32_1 = 1i32;
                                    s_i32_0 = ops::i32_sub(s_i32_0, s_i32_1);
                                    l9 = s_i32_0;
                                    s_i32_0 = l9;
                                    s_i32_0 = ops::i32_eqz(s_i32_0);
                                    if s_i32_0 != 0 {
                                        break 'blk_7;
                                    }
                                } else {
                                    s_i32_0 = l10;
                                    s_i32_1 = 1i32;
                                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                                    l10 = s_i32_0;
                                }
                            }
                        }
                    }
                }
            }
            continue 'blk_8;
            break 'blk_8;
        }
    }
    s_i32_0 = l10;
    s_i32_0 = ops::i32_eqz(s_i32_0);
    'blk_12: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    'blk_13: {
        'blk_14: loop {
            vm.deduct(host, 1)?;
            s_i32_0 = l0;
            s_i32_1 = l2;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t6 = __call_res?;
            s_i32_0 = __t6;
            l7 = s_i32_0;
            s_i32_0 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
            s_i32_1 = 1114373i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            if s_i32_0 != 0 {
                break 'blk_13;
            }
            s_i32_0 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
            s_i32_1 = 1114367i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_15: {
                if s_i32_0 != 0 {
                    s_i32_0 = l0;
                    s_i32_1 = l7;
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    l0 = s_i32_0;
                    s_i32_0 = l0;
                    s_i32_1 = l2;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t7 = __call_res?;
                    s_i32_0 = __t7;
                    l7 = s_i32_0;
                    s_i32_0 = l4;
                    s_i32_1 = l0;
                    s_i32_2 = l7;
                    vm.memory_copy(0, 0, s_i32_0, s_i32_1, s_i32_2, host)?;
                    s_i32_0 = l0;
                    s_i32_1 = l7;
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    l0 = s_i32_0;
                    s_i32_0 = l4;
                    s_i32_1 = l7;
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    l4 = s_i32_0;
                } else {
                    s_i32_0 = l4;
                    s_i32_1 = l0;
                    s_i32_2 = l7;
                    vm.memory_copy(0, 0, s_i32_0, s_i32_1, s_i32_2, host)?;
                    s_i32_0 = l0;
                    s_i32_1 = l7;
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    l0 = s_i32_0;
                    s_i32_0 = l4;
                    s_i32_1 = l7;
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    l4 = s_i32_0;
                }
            }
            continue 'blk_14;
            break 'blk_14;
        }
    }
    s_i32_0 = l0;
    s_i32_1 = l7;
    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
    l0 = s_i32_0;
    s_i32_0 = l8;
    s_i32_1 = 0i32;
    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
    'blk_16: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i32_1 = l2;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t8 = __call_res?;
            s_i32_0 = __t8;
            l7 = s_i32_0;
            s_i32_0 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
            s_i32_1 = 1114371i32;
            s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
            'blk_17: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = l0;
            s_i32_1 = l7;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l0 = s_i32_0;
        }
    }
    s_i32_0 = l0;
    s_i32_1 = l2;
    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
    'blk_18: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = l4;
    s_i32_1 = l3;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_19: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = 8i32;
    s_i32_1 = l3;
    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = 12i32;
    s_i32_1 = l4;
    s_i32_2 = l3;
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

const FN_0_GLOBALS: &[usize] = &[0usize];
const FN_1_GLOBALS: &[usize] = &[0usize];
const FN_1_CALLEES: &[u32] = &[0u32];
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
static BOUND_SIG_P_I32_I32_RET_I32: &[crate::Bound_p_i32_i32_ret_i32] = &[
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79092::ChunkFns::f_0dd72d798dab, globals: FN_0_GLOBALS, callees: &[] },
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
static BOUND_SIG_P_UNIT_RET_I32: &[crate::Bound_p_unit_ret_i32] = &[
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_79092::ChunkFns::f_4bd10795638b, globals: FN_1_GLOBALS, callees: FN_1_CALLEES },
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
    sig_p_i32_i32_ret_i32: BOUND_SIG_P_I32_I32_RET_I32,
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
    name: "85899_79092/2608@l8589979106",
    accounting_version: 2u32,
    wasm_sha256: [0xce, 0xab, 0x47, 0xd6, 0x68, 0x83, 0x46, 0x5a, 0x6c, 0x61, 0x60, 0x2d, 0xe9, 0x54, 0x0e, 0x8f, 0xaf, 0x31, 0x42, 0xbe, 0x1c, 0xea, 0xd7, 0x9f, 0x31, 0xb8, 0xe8, 0x42, 0x6d, 0xe3, 0x68, 0xe9],
    entry_func_name: "step",
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
    let g0: i32 = 0i32;
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
    vm.globals_i32 = vec![g0];
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
            "tok_cp" => self.vm.globals_i32.get(0usize).copied(),
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

