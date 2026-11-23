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

//! Generated native Rust WebAssembly substitute chunk `85900_04092`.
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
    pub(crate) fn f_1d2f475e0c00(vm: &mut SubstituteVm, dispatch: &'static crate::ModuleDispatch, g_map: &'static [usize], c_map: &'static [usize], host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = 0i32;
    let mut l1: i32 = 0i32;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i32_2: i32 = 0i32;
    let mut s_i32_3: i32 = 0i32;
    vm.deduct_1(host)?;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 0usize, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l0 = s_i32_0;
    s_i32_0 = 32i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_0, host);
    vm.exit_call();
    let __t1 = __call_res?;
    s_i32_0 = __t1;
    l1 = s_i32_0;
    s_i32_0 = l0;
    s_i32_1 = 1114629i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            s_i32_1 = 1i32;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l1;
            s_i32_1 = 1i32;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l1;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l0;
    s_i32_1 = 1114630i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            s_i32_1 = 1i32;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l1;
            s_i32_1 = 0i32;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l1;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l0;
    s_i32_1 = 1114118i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_3: {
        if s_i32_0 != 0 {
            vm.enter_call()?;
            let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 2usize, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_0 = __t2;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l0;
    s_i32_1 = 1114315i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_4: {
        if s_i32_0 != 0 {
            vm.enter_call()?;
            let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 3usize, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i32_0 = __t3;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l0;
    s_i32_1 = 1116690i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_5: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            s_i32_1 = 4i32;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l1;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 4usize, host);
            vm.exit_call();
            let __t4 = __call_res?;
            s_i32_1 = __t4;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 5usize, s_i32_0, host);
            vm.exit_call();
            let __t5 = __call_res?;
            s_i32_0 = __t5;
            l2 = s_i32_0;
            'blk_6: {
                'blk_7: loop {
                    vm.deduct_1(host)?;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 6usize, host);
                    vm.exit_call();
                    let __t6 = __call_res?;
                    s_i32_0 = __t6;
                    s_i32_1 = 1116691i32;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                    if s_i32_0 != 0 {
                        break 'blk_6;
                    }
                    s_i32_0 = l2;
                    vm.enter_call()?;
                    let __call_res = Self::f_1d2f475e0c00(vm, dispatch, g_map, c_map, host);
                    vm.exit_call();
                    let __t7 = __call_res?;
                    s_i32_1 = __t7;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 7usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                    continue 'blk_7;
                    break 'blk_7;
                }
            }
            s_i32_0 = 1116691i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_void(vm, c_map, 8usize, s_i32_0, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l1;
            s_i32_1 = l2;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l1;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l0;
    s_i32_1 = 1116327i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_8: {
        if s_i32_0 != 0 {
            vm.enter_call()?;
            let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 9usize, host);
            vm.exit_call();
            let __t8 = __call_res?;
            s_i32_0 = __t8;
            l3 = s_i32_0;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 5usize, s_i32_0, host);
            vm.exit_call();
            let __t9 = __call_res?;
            s_i32_0 = __t9;
            l2 = s_i32_0;
            s_i32_0 = 0i32;
            l4 = s_i32_0;
            'blk_9: {
                'blk_10: loop {
                    vm.deduct_1(host)?;
                    s_i32_0 = l4;
                    s_i32_1 = l3;
                    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                    if s_i32_0 != 0 {
                        break 'blk_9;
                    }
                    s_i32_0 = l4;
                    s_i32_1 = 0i32;
                    s_i32_0 = ops::i32_gt_u(s_i32_0, s_i32_1);
                    'blk_11: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 1114424i32;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_ret_void(vm, c_map, 8usize, s_i32_0, host);
                            vm.exit_call();
                            __call_res?;
                        }
                    }
                    s_i32_0 = l2;
                    vm.enter_call()?;
                    let __call_res = Self::f_1d2f475e0c00(vm, dispatch, g_map, c_map, host);
                    vm.exit_call();
                    let __t10 = __call_res?;
                    s_i32_1 = __t10;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 7usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l4;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    l4 = s_i32_0;
                    continue 'blk_10;
                    break 'blk_10;
                }
            }
            s_i32_0 = 1116328i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_void(vm, c_map, 8usize, s_i32_0, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l1;
            s_i32_1 = 5i32;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l1;
            s_i32_1 = l2;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l1;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l0;
    s_i32_1 = 1116331i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_12: {
        if s_i32_0 != 0 {
            vm.enter_call()?;
            let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 10usize, host);
            vm.exit_call();
            let __t11 = __call_res?;
            s_i32_0 = __t11;
            l5 = s_i32_0;
            s_i32_0 = l1;
            s_i32_1 = 6i32;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l1;
            s_i32_1 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l1;
            s_i32_1 = l5;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l1;
            s_i32_1 = vm.get_i32(*g_map.get(1usize).ok_or(WasmTrap::Unreachable)?)?;
            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 5usize, s_i32_0, host);
            vm.exit_call();
            let __t12 = __call_res?;
            s_i32_0 = __t12;
            l2 = s_i32_0;
            'blk_13: {
                'blk_14: loop {
                    vm.deduct_1(host)?;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 6usize, host);
                    vm.exit_call();
                    let __t13 = __call_res?;
                    s_i32_0 = __t13;
                    s_i32_1 = 1116332i32;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                    if s_i32_0 != 0 {
                        break 'blk_13;
                    }
                    s_i32_0 = l2;
                    vm.enter_call()?;
                    let __call_res = Self::f_1d2f475e0c00(vm, dispatch, g_map, c_map, host);
                    vm.exit_call();
                    let __t14 = __call_res?;
                    s_i32_1 = __t14;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 7usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                    continue 'blk_14;
                    break 'blk_14;
                }
            }
            s_i32_0 = 1116332i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_void(vm, c_map, 8usize, s_i32_0, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l2;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 11usize, s_i32_0, host);
            vm.exit_call();
            let __t15 = __call_res?;
            s_i32_0 = __t15;
            s_i32_0 = ops::i32_eqz(s_i32_0);
            'blk_15: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = l1;
            s_i32_1 = l2;
            ops::i32_store(vm, 0, s_i32_0, 16u32, s_i32_1)?;
            s_i32_0 = l1;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l0;
    s_i32_1 = 1116659i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_16: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            s_i32_1 = 7i32;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l1;
            vm.enter_call()?;
            let __call_res = Self::f_1d2f475e0c00(vm, dispatch, g_map, c_map, host);
            vm.exit_call();
            let __t16 = __call_res?;
            s_i32_1 = __t16;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l1;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 9usize, host);
            vm.exit_call();
            let __t17 = __call_res?;
            s_i32_1 = __t17;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l1;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l0;
    s_i32_1 = 1116660i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_17: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            s_i32_1 = 8i32;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l1;
            vm.enter_call()?;
            let __call_res = Self::f_1d2f475e0c00(vm, dispatch, g_map, c_map, host);
            vm.exit_call();
            let __t18 = __call_res?;
            s_i32_1 = __t18;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l1;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 9usize, host);
            vm.exit_call();
            let __t19 = __call_res?;
            s_i32_1 = __t19;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l1;
            vm.enter_call()?;
            let __call_res = Self::f_1d2f475e0c00(vm, dispatch, g_map, c_map, host);
            vm.exit_call();
            let __t20 = __call_res?;
            s_i32_1 = __t20;
            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
            s_i32_0 = l1;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l0;
    s_i32_1 = 1116692i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_18: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            s_i32_1 = 9i32;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l1;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 4usize, host);
            vm.exit_call();
            let __t21 = __call_res?;
            s_i32_1 = __t21;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l1;
            vm.enter_call()?;
            let __call_res = Self::f_1d2f475e0c00(vm, dispatch, g_map, c_map, host);
            vm.exit_call();
            let __t22 = __call_res?;
            s_i32_1 = __t22;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l1;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l0;
    s_i32_1 = 1116314i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_19: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            s_i32_1 = 10i32;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 6usize, host);
            vm.exit_call();
            let __t23 = __call_res?;
            s_i32_0 = __t23;
            s_i32_1 = 1114428i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_20: {
                if s_i32_0 != 0 {
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 0usize, host);
                    vm.exit_call();
                    let __t24 = __call_res?;
                    s_i32_0 = __t24;
                    s_i32_0 = l1;
                    s_i32_1 = 0i32;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                } else {
                    s_i32_0 = l1;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 4usize, host);
                    vm.exit_call();
                    let __t25 = __call_res?;
                    s_i32_1 = __t25;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                }
            }
            vm.enter_call()?;
            let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 10usize, host);
            vm.exit_call();
            let __t26 = __call_res?;
            s_i32_0 = __t26;
            l5 = s_i32_0;
            s_i32_0 = l1;
            s_i32_1 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l1;
            s_i32_1 = l5;
            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
            s_i32_0 = l1;
            s_i32_1 = vm.get_i32(*g_map.get(1usize).ok_or(WasmTrap::Unreachable)?)?;
            ops::i32_store(vm, 0, s_i32_0, 16u32, s_i32_1)?;
            s_i32_0 = l1;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l0;
    s_i32_1 = 1116310i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_21: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            s_i32_1 = 11i32;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l1;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 4usize, host);
            vm.exit_call();
            let __t27 = __call_res?;
            s_i32_1 = __t27;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 12usize, host);
            vm.exit_call();
            let __t28 = __call_res?;
            s_i32_0 = __t28;
            l5 = s_i32_0;
            s_i32_0 = l1;
            s_i32_1 = l5;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l1;
            s_i32_1 = vm.get_i32(*g_map.get(1usize).ok_or(WasmTrap::Unreachable)?)?;
            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 6usize, host);
            vm.exit_call();
            let __t29 = __call_res?;
            s_i32_0 = __t29;
            s_i32_1 = 1114428i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_22: {
                if s_i32_0 != 0 {
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 0usize, host);
                    vm.exit_call();
                    let __t30 = __call_res?;
                    s_i32_0 = __t30;
                    s_i32_0 = l1;
                    s_i32_1 = 0i32;
                    ops::i32_store(vm, 0, s_i32_0, 16u32, s_i32_1)?;
                } else {
                    s_i32_0 = l1;
                    vm.enter_call()?;
                    let __call_res = Self::f_1d2f475e0c00(vm, dispatch, g_map, c_map, host);
                    vm.exit_call();
                    let __t31 = __call_res?;
                    s_i32_1 = __t31;
                    ops::i32_store(vm, 0, s_i32_0, 16u32, s_i32_1)?;
                }
            }
            s_i32_0 = l1;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l0;
    s_i32_1 = 1116311i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_23: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            s_i32_1 = 12i32;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 13usize, host);
            vm.exit_call();
            let __t32 = __call_res?;
            s_i32_0 = __t32;
            s_i32_0 = l1;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 4usize, host);
            vm.exit_call();
            let __t33 = __call_res?;
            s_i32_1 = __t33;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 12usize, host);
            vm.exit_call();
            let __t34 = __call_res?;
            s_i32_0 = __t34;
            l5 = s_i32_0;
            s_i32_0 = l1;
            s_i32_1 = l5;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l1;
            s_i32_1 = vm.get_i32(*g_map.get(1usize).ok_or(WasmTrap::Unreachable)?)?;
            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
            s_i32_0 = l1;
            vm.enter_call()?;
            let __call_res = Self::f_1d2f475e0c00(vm, dispatch, g_map, c_map, host);
            vm.exit_call();
            let __t35 = __call_res?;
            s_i32_1 = __t35;
            ops::i32_store(vm, 0, s_i32_0, 16u32, s_i32_1)?;
            s_i32_0 = l1;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l0;
    s_i32_1 = 1116313i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_24: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            s_i32_1 = 13i32;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            vm.deduct_1(host)?;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 6usize, host);
            vm.exit_call();
            let __t36 = __call_res?;
            s_i32_0 = __t36;
            s_i32_1 = 1114428i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_25: {
                if s_i32_0 != 0 {
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 0usize, host);
                    vm.exit_call();
                    let __t37 = __call_res?;
                    s_i32_0 = __t37;
                    s_i32_0 = l1;
                    s_i32_1 = 0i32;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                } else {
                    s_i32_0 = l1;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 4usize, host);
                    vm.exit_call();
                    let __t38 = __call_res?;
                    s_i32_1 = __t38;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                }
            }
            vm.enter_call()?;
            let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 12usize, host);
            vm.exit_call();
            let __t39 = __call_res?;
            s_i32_0 = __t39;
            l5 = s_i32_0;
            s_i32_0 = l1;
            s_i32_1 = l5;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l1;
            s_i32_1 = vm.get_i32(*g_map.get(1usize).ok_or(WasmTrap::Unreachable)?)?;
            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
            s_i32_0 = l1;
            vm.enter_call()?;
            let __call_res = Self::f_1d2f475e0c00(vm, dispatch, g_map, c_map, host);
            vm.exit_call();
            let __t40 = __call_res?;
            s_i32_1 = __t40;
            ops::i32_store(vm, 0, s_i32_0, 16u32, s_i32_1)?;
            s_i32_0 = l1;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l0;
    s_i32_1 = 1116315i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    vm.deduct_1(host)?;
    'blk_26: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
            vm.set_i32(*g_map.get(2usize).ok_or(WasmTrap::Unreachable)?, s_i32_0)?;
            s_i32_0 = l1;
            s_i32_1 = 14i32;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l1;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 14usize, host);
            vm.exit_call();
            let __t41 = __call_res?;
            s_i32_1 = __t41;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l1;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l0;
    s_i32_1 = 1116323i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_27: {
        if s_i32_0 != 0 {
            vm.enter_call()?;
            let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 6usize, host);
            vm.exit_call();
            let __t42 = __call_res?;
            s_i32_0 = __t42;
            s_i32_1 = 1116352i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_28: {
                if s_i32_0 != 0 {
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 0usize, host);
                    vm.exit_call();
                    let __t43 = __call_res?;
                    s_i32_0 = __t43;
                    s_i32_0 = 1i32;
                    vm.set_i32(*g_map.get(2usize).ok_or(WasmTrap::Unreachable)?, s_i32_0)?;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 15usize, host);
                    vm.exit_call();
                    let __t44 = __call_res?;
                    s_i32_0 = __t44;
                    l5 = s_i32_0;
                    s_i32_0 = 16i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 5usize, s_i32_0, host);
                    vm.exit_call();
                    let __t45 = __call_res?;
                    s_i32_0 = __t45;
                    l2 = s_i32_0;
                    'blk_29: {
                        'blk_30: loop {
                            vm.deduct_1(host)?;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 6usize, host);
                            vm.exit_call();
                            let __t46 = __call_res?;
                            s_i32_0 = __t46;
                            s_i32_1 = 1116324i32;
                            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                            if s_i32_0 != 0 {
                                break 'blk_29;
                            }
                            s_i32_0 = l2;
                            vm.enter_call()?;
                            let __call_res = Self::f_1d2f475e0c00(vm, dispatch, g_map, c_map, host);
                            vm.exit_call();
                            let __t47 = __call_res?;
                            s_i32_1 = __t47;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 7usize, s_i32_0, s_i32_1, host);
                            vm.exit_call();
                            __call_res?;
                            continue 'blk_30;
                            break 'blk_30;
                        }
                    }
                    s_i32_0 = 1116324i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_void(vm, c_map, 8usize, s_i32_0, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = 32i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_0, host);
                    vm.exit_call();
                    let __t48 = __call_res?;
                    s_i32_0 = __t48;
                    l3 = s_i32_0;
                    s_i32_0 = l3;
                    s_i32_1 = 14i32;
                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l3;
                    s_i32_1 = l5;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = 32i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_0, host);
                    vm.exit_call();
                    let __t49 = __call_res?;
                    s_i32_0 = __t49;
                    l4 = s_i32_0;
                    s_i32_0 = l4;
                    s_i32_1 = 15i32;
                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l4;
                    s_i32_1 = 0i32;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l4;
                    s_i32_1 = l5;
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 16u32)?;
                    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                    s_i32_0 = l4;
                    s_i32_1 = l5;
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 20u32)?;
                    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                    s_i32_0 = l4;
                    s_i32_1 = l2;
                    ops::i32_store(vm, 0, s_i32_0, 16u32, s_i32_1)?;
                    s_i32_0 = 16i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 5usize, s_i32_0, host);
                    vm.exit_call();
                    let __t50 = __call_res?;
                    s_i32_0 = __t50;
                    l2 = s_i32_0;
                    s_i32_0 = l2;
                    s_i32_1 = l3;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 7usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l2;
                    s_i32_1 = l4;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 7usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l1;
                    s_i32_1 = 16i32;
                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l1;
                    s_i32_1 = l2;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l1;
                    return Ok(s_i32_0);
                } else {
                    s_i32_0 = l1;
                    s_i32_1 = 15i32;
                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 10usize, host);
                    vm.exit_call();
                    let __t51 = __call_res?;
                    s_i32_0 = __t51;
                    l5 = s_i32_0;
                    s_i32_0 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
                    s_i32_1 = vm.get_i32(*g_map.get(3usize).ok_or(WasmTrap::Unreachable)?)?;
                    s_i32_2 = 1116354i32;
                    s_i32_1 = ops::i32_eq(s_i32_1, s_i32_2);
                    s_i32_2 = vm.get_i32(*g_map.get(3usize).ok_or(WasmTrap::Unreachable)?)?;
                    s_i32_3 = 1114711i32;
                    s_i32_2 = ops::i32_eq(s_i32_2, s_i32_3);
                    s_i32_1 = ops::i32_or(s_i32_1, s_i32_2);
                    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                    'blk_31: {
                        if s_i32_0 != 0 {
                            s_i32_0 = vm.get_i32(*g_map.get(4usize).ok_or(WasmTrap::Unreachable)?)?;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 11usize, s_i32_0, host);
                            vm.exit_call();
                            let __t52 = __call_res?;
                            s_i32_0 = __t52;
                            l4 = s_i32_0;
                            s_i32_0 = l4;
                            s_i32_0 = ops::i32_eqz(s_i32_0);
                            'blk_32: {
                                if s_i32_0 != 0 {
                                    return Err(WasmTrap::Unreachable);
                                }
                            }
                            s_i32_0 = vm.get_i32(*g_map.get(4usize).ok_or(WasmTrap::Unreachable)?)?;
                            s_i32_1 = l4;
                            s_i32_2 = 1i32;
                            s_i32_1 = ops::i32_sub(s_i32_1, s_i32_2);
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 16usize, s_i32_0, s_i32_1, host);
                            vm.exit_call();
                            let __t53 = __call_res?;
                            s_i32_0 = __t53;
                            l3 = s_i32_0;
                            s_i32_0 = l1;
                            s_i32_1 = l3;
                            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 0u32)?;
                            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                            s_i32_0 = l1;
                            s_i32_1 = l3;
                            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
                            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                            s_i32_0 = l1;
                            s_i32_1 = l3;
                            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 8u32)?;
                            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                        } else {
                            s_i32_0 = l1;
                            s_i32_1 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
                            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                            s_i32_0 = l1;
                            s_i32_1 = l5;
                            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                            s_i32_0 = l1;
                            s_i32_1 = vm.get_i32(*g_map.get(1usize).ok_or(WasmTrap::Unreachable)?)?;
                            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                        }
                    }
                    s_i32_0 = 16i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 5usize, s_i32_0, host);
                    vm.exit_call();
                    let __t54 = __call_res?;
                    s_i32_0 = __t54;
                    l2 = s_i32_0;
                    'blk_33: {
                        'blk_34: loop {
                            vm.deduct_1(host)?;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 6usize, host);
                            vm.exit_call();
                            let __t55 = __call_res?;
                            s_i32_0 = __t55;
                            s_i32_1 = 1116324i32;
                            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                            if s_i32_0 != 0 {
                                break 'blk_33;
                            }
                            s_i32_0 = l2;
                            vm.enter_call()?;
                            let __call_res = Self::f_1d2f475e0c00(vm, dispatch, g_map, c_map, host);
                            vm.exit_call();
                            let __t56 = __call_res?;
                            s_i32_1 = __t56;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 7usize, s_i32_0, s_i32_1, host);
                            vm.exit_call();
                            __call_res?;
                            continue 'blk_34;
                            break 'blk_34;
                        }
                    }
                    s_i32_0 = 1116324i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_void(vm, c_map, 8usize, s_i32_0, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l1;
                    s_i32_1 = l2;
                    ops::i32_store(vm, 0, s_i32_0, 16u32, s_i32_1)?;
                    s_i32_0 = l1;
                    return Ok(s_i32_0);
                }
            }
        }
    }
    s_i32_0 = l0;
    s_i32_1 = 1116325i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_35: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            s_i32_1 = 16i32;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 5usize, s_i32_0, host);
            vm.exit_call();
            let __t57 = __call_res?;
            s_i32_0 = __t57;
            l2 = s_i32_0;
            'blk_36: {
                'blk_37: loop {
                    vm.deduct_1(host)?;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 6usize, host);
                    vm.exit_call();
                    let __t58 = __call_res?;
                    s_i32_0 = __t58;
                    s_i32_1 = 1116326i32;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                    if s_i32_0 != 0 {
                        break 'blk_36;
                    }
                    s_i32_0 = l2;
                    vm.enter_call()?;
                    let __call_res = Self::f_1d2f475e0c00(vm, dispatch, g_map, c_map, host);
                    vm.exit_call();
                    let __t59 = __call_res?;
                    s_i32_1 = __t59;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 7usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                    continue 'blk_37;
                    break 'blk_37;
                }
            }
            s_i32_0 = 1116326i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_void(vm, c_map, 8usize, s_i32_0, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l1;
            s_i32_1 = l2;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l1;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l0;
    s_i32_1 = 1116560i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_38: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            s_i32_1 = 17i32;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l1;
            vm.enter_call()?;
            let __call_res = Self::f_1d2f475e0c00(vm, dispatch, g_map, c_map, host);
            vm.exit_call();
            let __t60 = __call_res?;
            s_i32_1 = __t60;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l1;
            vm.enter_call()?;
            let __call_res = Self::f_1d2f475e0c00(vm, dispatch, g_map, c_map, host);
            vm.exit_call();
            let __t61 = __call_res?;
            s_i32_1 = __t61;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l1;
            vm.enter_call()?;
            let __call_res = Self::f_1d2f475e0c00(vm, dispatch, g_map, c_map, host);
            vm.exit_call();
            let __t62 = __call_res?;
            s_i32_1 = __t62;
            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
            s_i32_0 = l1;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l0;
    s_i32_1 = 1116561i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_39: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            s_i32_1 = 18i32;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l1;
            vm.enter_call()?;
            let __call_res = Self::f_1d2f475e0c00(vm, dispatch, g_map, c_map, host);
            vm.exit_call();
            let __t63 = __call_res?;
            s_i32_1 = __t63;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l1;
            vm.enter_call()?;
            let __call_res = Self::f_1d2f475e0c00(vm, dispatch, g_map, c_map, host);
            vm.exit_call();
            let __t64 = __call_res?;
            s_i32_1 = __t64;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l1;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l0;
    s_i32_1 = 1116562i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_40: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            s_i32_1 = 19i32;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l1;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 4usize, host);
            vm.exit_call();
            let __t65 = __call_res?;
            s_i32_1 = __t65;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 12usize, host);
            vm.exit_call();
            let __t66 = __call_res?;
            s_i32_0 = __t66;
            l5 = s_i32_0;
            s_i32_0 = l1;
            s_i32_1 = l5;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l1;
            s_i32_1 = vm.get_i32(*g_map.get(1usize).ok_or(WasmTrap::Unreachable)?)?;
            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
            s_i32_0 = l1;
            vm.enter_call()?;
            let __call_res = Self::f_1d2f475e0c00(vm, dispatch, g_map, c_map, host);
            vm.exit_call();
            let __t67 = __call_res?;
            s_i32_1 = __t67;
            ops::i32_store(vm, 0, s_i32_0, 16u32, s_i32_1)?;
            s_i32_0 = l1;
            vm.enter_call()?;
            let __call_res = Self::f_1d2f475e0c00(vm, dispatch, g_map, c_map, host);
            vm.exit_call();
            let __t68 = __call_res?;
            s_i32_1 = __t68;
            ops::i32_store(vm, 0, s_i32_0, 20u32, s_i32_1)?;
            s_i32_0 = l1;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l0;
    s_i32_1 = 1116563i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_41: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            s_i32_1 = 20i32;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l1;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l0;
    s_i32_1 = 1116564i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_42: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            s_i32_1 = 21i32;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l1;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l0;
    s_i32_1 = 1116565i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_43: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            s_i32_1 = 22i32;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 6usize, host);
            vm.exit_call();
            let __t69 = __call_res?;
            s_i32_0 = __t69;
            s_i32_1 = 1114428i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_44: {
                if s_i32_0 != 0 {
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 0usize, host);
                    vm.exit_call();
                    let __t70 = __call_res?;
                    s_i32_0 = __t70;
                    s_i32_0 = l1;
                    s_i32_1 = 0i32;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                } else {
                    s_i32_0 = l1;
                    vm.enter_call()?;
                    let __call_res = Self::f_1d2f475e0c00(vm, dispatch, g_map, c_map, host);
                    vm.exit_call();
                    let __t71 = __call_res?;
                    s_i32_1 = __t71;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                }
            }
            s_i32_0 = l1;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l0;
    s_i32_1 = 1116566i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_45: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            s_i32_1 = 23i32;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l1;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l0;
    s_i32_1 = 1116567i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_46: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            s_i32_1 = 24i32;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l1;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l0;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 17usize, s_i32_0, host);
    vm.exit_call();
    let __t72 = __call_res?;
    s_i32_0 = __t72;
    'blk_47: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            s_i32_1 = 25i32;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l1;
            s_i32_1 = l0;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l1;
            vm.enter_call()?;
            let __call_res = Self::f_1d2f475e0c00(vm, dispatch, g_map, c_map, host);
            vm.exit_call();
            let __t73 = __call_res?;
            s_i32_1 = __t73;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l1;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l0;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 18usize, s_i32_0, host);
    vm.exit_call();
    let __t74 = __call_res?;
    s_i32_0 = __t74;
    'blk_48: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            s_i32_1 = 26i32;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l1;
            s_i32_1 = l0;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l1;
            vm.enter_call()?;
            let __call_res = Self::f_1d2f475e0c00(vm, dispatch, g_map, c_map, host);
            vm.exit_call();
            let __t75 = __call_res?;
            s_i32_1 = __t75;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l1;
            vm.enter_call()?;
            let __call_res = Self::f_1d2f475e0c00(vm, dispatch, g_map, c_map, host);
            vm.exit_call();
            let __t76 = __call_res?;
            s_i32_1 = __t76;
            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
            s_i32_0 = l1;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l0;
    s_i32_1 = 1116657i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    s_i32_1 = l0;
    s_i32_2 = 1116696i32;
    s_i32_1 = ops::i32_eq(s_i32_1, s_i32_2);
    s_i32_0 = ops::i32_or(s_i32_0, s_i32_1);
    'blk_49: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            s_i32_1 = 27i32;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l1;
            s_i32_1 = l0;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l1;
            vm.enter_call()?;
            let __call_res = Self::f_1d2f475e0c00(vm, dispatch, g_map, c_map, host);
            vm.exit_call();
            let __t77 = __call_res?;
            s_i32_1 = __t77;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            vm.deduct_1(host)?;
            s_i32_0 = l1;
            vm.enter_call()?;
            let __call_res = Self::f_1d2f475e0c00(vm, dispatch, g_map, c_map, host);
            vm.exit_call();
            let __t78 = __call_res?;
            s_i32_1 = __t78;
            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
            s_i32_0 = l1;
            vm.enter_call()?;
            let __call_res = Self::f_1d2f475e0c00(vm, dispatch, g_map, c_map, host);
            vm.exit_call();
            let __t79 = __call_res?;
            s_i32_1 = __t79;
            ops::i32_store(vm, 0, s_i32_0, 16u32, s_i32_1)?;
            s_i32_0 = l1;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l0;
    s_i32_1 = 1116653i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_50: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            s_i32_1 = 28i32;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l1;
            vm.enter_call()?;
            let __call_res = Self::f_1d2f475e0c00(vm, dispatch, g_map, c_map, host);
            vm.exit_call();
            let __t80 = __call_res?;
            s_i32_1 = __t80;
            vm.deduct_1(host)?;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l1;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 4usize, host);
            vm.exit_call();
            let __t81 = __call_res?;
            s_i32_1 = __t81;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l1;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l0;
    s_i32_1 = 1116654i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_51: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            vm.deduct_1(host)?;
            s_i32_1 = 29i32;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l1;
            vm.enter_call()?;
            let __call_res = Self::f_1d2f475e0c00(vm, dispatch, g_map, c_map, host);
            vm.exit_call();
            let __t82 = __call_res?;
            s_i32_1 = __t82;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l1;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 4usize, host);
            vm.exit_call();
            let __t83 = __call_res?;
            s_i32_1 = __t83;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l1;
            return Ok(s_i32_0);
        }
    }
    return Err(WasmTrap::Unreachable);
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

const FN_0_GLOBALS: &[usize] = &[0usize, 1usize];
const FN_2_CALLEES: &[usize] = &[0usize];
const FN_3_CALLEES: &[usize] = &[0usize];
const FN_4_CALLEES: &[usize] = &[0usize];
const FN_5_CALLEES: &[usize] = &[0usize];
const FN_6_CALLEES: &[usize] = &[0usize];
const FN_10_CALLEES: &[usize] = &[1usize, 0usize];
const FN_11_GLOBALS: &[usize] = &[8usize];
const FN_11_CALLEES: &[usize] = &[1usize, 1usize, 3usize];
const FN_12_GLOBALS: &[usize] = &[8usize];
const FN_12_CALLEES: &[usize] = &[0usize];
const FN_13_GLOBALS: &[usize] = &[5usize, 4usize, 6usize];
const FN_14_GLOBALS: &[usize] = &[2usize, 3usize, 4usize];
const FN_14_CALLEES: &[usize] = &[2usize];
const FN_15_GLOBALS: &[usize] = &[2usize, 3usize, 4usize];
const FN_15_CALLEES: &[usize] = &[2usize];
const FN_16_CALLEES: &[usize] = &[1usize];
const FN_17_GLOBALS: &[usize] = &[2usize, 3usize, 4usize];
const FN_17_CALLEES: &[usize] = &[2usize, 0usize];
const FN_18_CALLEES: &[usize] = &[0usize, 0usize];
const FN_19_CALLEES: &[usize] = &[0usize, 1usize];
const FN_20_CALLEES: &[usize] = &[3usize];
const FN_21_CALLEES: &[usize] = &[1usize];
const FN_22_CALLEES: &[usize] = &[4usize, 0usize];
const FN_23_CALLEES: &[usize] = &[0usize, 4usize, 0usize];
const FN_24_GLOBALS: &[usize] = &[4usize];
const FN_24_CALLEES: &[usize] = &[2usize];
const FN_25_GLOBALS: &[usize] = &[4usize];
const FN_25_CALLEES: &[usize] = &[2usize, 1usize];
const FN_26_GLOBALS: &[usize] = &[2usize, 3usize, 10usize];
const FN_26_CALLEES: &[usize] = &[2usize, 1usize, 1usize, 0usize, 2usize];
const FN_27_GLOBALS: &[usize] = &[4usize, 7usize, 8usize, 9usize];
const FN_27_CALLEES: &[usize] = &[2usize, 3usize, 1usize, 4usize, 0usize];
const FN_28_GLOBALS: &[usize] = &[4usize, 9usize];
const FN_28_CALLEES: &[usize] = &[2usize];
const FN_29_CALLEES: &[usize] = &[1usize, 0usize];
const FN_30_GLOBALS: &[usize] = &[4usize, 9usize, 7usize, 8usize];
const FN_30_CALLEES: &[usize] = &[2usize, 4usize, 2usize, 0usize, 0usize, 5usize];
const FN_31_GLOBALS: &[usize] = &[4usize, 9usize, 8usize];
const FN_31_CALLEES: &[usize] = &[2usize, 3usize, 1usize, 1usize, 0usize, 2usize, 0usize, 0usize, 0usize, 4usize];
const FN_32_GLOBALS: &[usize] = &[11usize];
const FN_32_CALLEES: &[usize] = &[2usize, 0usize, 0usize];
const FN_33_GLOBALS: &[usize] = &[11usize];
const FN_33_CALLEES: &[usize] = &[6usize, 2usize, 0usize, 2usize];
const FN_34_CALLEES: &[usize] = &[6usize, 1usize, 4usize, 1usize, 0usize];
const FN_35_GLOBALS: &[usize] = &[12usize];
const FN_35_CALLEES: &[usize] = &[2usize, 0usize, 0usize];
const FN_36_GLOBALS: &[usize] = &[12usize];
const FN_36_CALLEES: &[usize] = &[7usize, 2usize, 0usize, 2usize];
const FN_37_GLOBALS: &[usize] = &[12usize];
const FN_37_CALLEES: &[usize] = &[0usize];
const FN_38_GLOBALS: &[usize] = &[10usize, 9usize];
const FN_38_CALLEES: &[usize] = &[2usize, 0usize];
const FN_39_GLOBALS: &[usize] = &[8usize, 4usize, 7usize, 9usize];
const FN_39_CALLEES: &[usize] = &[1usize, 2usize, 8usize, 2usize, 1usize, 3usize, 1usize, 0usize, 4usize, 2usize, 0usize, 0usize, 7usize, 2usize, 0usize, 0usize];
const FN_40_GLOBALS: &[usize] = &[9usize];
const FN_40_CALLEES: &[usize] = &[0usize, 9usize];
const FN_41_GLOBALS: &[usize] = &[4usize, 9usize];
const FN_41_CALLEES: &[usize] = &[2usize, 3usize, 5usize, 1usize, 0usize];
const FN_42_GLOBALS: &[usize] = &[9usize];
const FN_42_CALLEES: &[usize] = &[0usize, 2usize, 9usize, 6usize];
const FN_43_CALLEES: &[usize] = &[7usize, 0usize];
const FN_44_GLOBALS: &[usize] = &[13usize];
const FN_44_CALLEES: &[usize] = &[1usize, 0usize, 7usize];
const FN_45_GLOBALS: &[usize] = &[14usize];
const FN_45_CALLEES: &[usize] = &[0usize, 2usize];
const FN_48_CALLEES: &[usize] = &[4usize];
const FN_51_GLOBALS: &[usize] = &[14usize];
const FN_51_CALLEES: &[usize] = &[2usize, 0usize, 0usize, 2usize];
const FN_52_GLOBALS: &[usize] = &[14usize];
const FN_52_CALLEES: &[usize] = &[2usize, 0usize, 0usize, 2usize];
const FN_53_CALLEES: &[usize] = &[10usize];
const FN_54_GLOBALS: &[usize] = &[8usize];
const FN_54_CALLEES: &[usize] = &[1usize, 1usize, 1usize, 4usize];
const FN_55_GLOBALS: &[usize] = &[8usize];
const FN_55_CALLEES: &[usize] = &[1usize, 1usize, 1usize, 1usize, 4usize];
const FN_56_GLOBALS: &[usize] = &[6usize, 8usize, 5usize];
const FN_56_CALLEES: &[usize] = &[3usize, 1usize];
const FN_57_GLOBALS: &[usize] = &[8usize];
const FN_57_CALLEES: &[usize] = &[2usize, 3usize, 7usize];
const FN_58_GLOBALS: &[usize] = &[9usize];
const FN_58_CALLEES: &[usize] = &[2usize, 5usize, 7usize];
const FN_59_GLOBALS: &[usize] = &[2usize, 3usize, 9usize];
const FN_59_CALLEES: &[usize] = &[0usize, 5usize];
const FN_60_GLOBALS: &[usize] = &[9usize, 8usize];
const FN_60_CALLEES: &[usize] = &[3usize, 0usize, 1usize, 1usize, 2usize, 2usize, 2usize, 9usize];
const FN_61_CALLEES: &[usize] = &[1usize, 0usize, 0usize];
const FN_62_CALLEES: &[usize] = &[10usize, 1usize, 1usize, 1usize, 0usize];
const FN_63_CALLEES: &[usize] = &[1usize];
const FN_64_GLOBALS: &[usize] = &[26usize];
const FN_64_CALLEES: &[usize] = &[0usize, 2usize];
const FN_65_GLOBALS: &[usize] = &[26usize];
const FN_66_GLOBALS: &[usize] = &[8usize];
const FN_66_CALLEES: &[usize] = &[0usize, 3usize, 8usize, 11usize, 5usize, 1usize, 2usize, 2usize, 3usize, 3usize, 16usize, 4usize, 1usize];
const FN_67_GLOBALS: &[usize] = &[8usize];
const FN_67_CALLEES: &[usize] = &[1usize, 1usize, 5usize, 18usize, 3usize, 0usize, 11usize, 2usize, 3usize, 5usize, 2usize, 3usize, 16usize, 4usize, 1usize];
const FN_70_GLOBALS: &[usize] = &[9usize, 8usize, 17usize, 4usize, 26usize];
const FN_70_CALLEES: &[usize] = &[3usize, 0usize, 12usize, 13usize, 11usize, 1usize, 2usize, 2usize, 1usize, 10usize, 9usize, 2usize, 5usize, 8usize, 14usize, 15usize, 0usize, 9usize, 10usize];
const FN_71_GLOBALS: &[usize] = &[8usize, 9usize, 15usize, 22usize];
const FN_71_CALLEES: &[usize] = &[1usize, 0usize, 1usize, 3usize, 6usize, 2usize, 9usize, 5usize, 2usize, 8usize, 11usize, 2usize, 16usize, 14usize];
const FN_72_CALLEES: &[usize] = &[1usize];
const FN_73_CALLEES: &[usize] = &[1usize];
const FN_74_GLOBALS: &[usize] = &[14usize, 15usize, 16usize];
const FN_74_CALLEES: &[usize] = &[1usize, 5usize, 0usize, 3usize, 4usize];
const FN_75_GLOBALS: &[usize] = &[16usize];
const FN_75_CALLEES: &[usize] = &[0usize, 0usize];
const FN_76_GLOBALS: &[usize] = &[16usize];
const FN_76_CALLEES: &[usize] = &[0usize, 0usize];
const FN_77_GLOBALS: &[usize] = &[16usize];
const FN_77_CALLEES: &[usize] = &[2usize, 0usize, 0usize];
const FN_78_GLOBALS: &[usize] = &[16usize, 15usize];
const FN_78_CALLEES: &[usize] = &[0usize, 2usize, 0usize];
const FN_79_GLOBALS: &[usize] = &[16usize, 15usize];
const FN_79_CALLEES: &[usize] = &[0usize, 2usize, 0usize];
const FN_80_GLOBALS: &[usize] = &[16usize];
const FN_80_CALLEES: &[usize] = &[3usize, 0usize, 2usize, 0usize, 3usize, 4usize];
const FN_81_GLOBALS: &[usize] = &[16usize];
const FN_81_CALLEES: &[usize] = &[4usize, 0usize, 2usize, 0usize, 3usize];
const FN_82_CALLEES: &[usize] = &[10usize, 0usize, 11usize, 4usize, 8usize, 5usize];
const FN_83_GLOBALS: &[usize] = &[14usize, 15usize];
const FN_83_CALLEES: &[usize] = &[0usize];
const FN_84_GLOBALS: &[usize] = &[14usize, 15usize];
const FN_84_CALLEES: &[usize] = &[0usize];
const FN_85_GLOBALS: &[usize] = &[14usize];
const FN_85_CALLEES: &[usize] = &[0usize, 12usize];
const FN_86_GLOBALS: &[usize] = &[14usize, 15usize];
const FN_86_CALLEES: &[usize] = &[0usize, 0usize, 4usize, 5usize];
const FN_87_CALLEES: &[usize] = &[2usize, 0usize];
const FN_88_GLOBALS: &[usize] = &[16usize];
const FN_88_CALLEES: &[usize] = &[4usize, 3usize, 0usize, 2usize, 0usize];
const FN_89_GLOBALS: &[usize] = &[16usize, 15usize, 8usize];
const FN_89_CALLEES: &[usize] = &[2usize, 0usize, 3usize, 16usize, 12usize, 0usize, 0usize, 14usize, 8usize, 2usize, 3usize, 12usize, 3usize, 1usize, 1usize, 15usize, 1usize, 5usize, 3usize, 1usize, 4usize, 5usize, 8usize];
const FN_90_GLOBALS: &[usize] = &[16usize];
const FN_90_CALLEES: &[usize] = &[0usize, 2usize, 0usize];
const FN_91_GLOBALS: &[usize] = &[16usize];
const FN_91_CALLEES: &[usize] = &[1usize, 0usize, 2usize, 0usize, 3usize];
const FN_92_CALLEES: &[usize] = &[2usize, 0usize];
const FN_93_GLOBALS: &[usize] = &[16usize];
const FN_93_CALLEES: &[usize] = &[0usize, 2usize, 0usize, 6usize];
const FN_94_GLOBALS: &[usize] = &[16usize];
const FN_94_CALLEES: &[usize] = &[0usize, 0usize, 2usize, 0usize, 3usize];
const FN_95_CALLEES: &[usize] = &[0usize, 1usize];
const FN_96_CALLEES: &[usize] = &[2usize, 0usize, 2usize];
const FN_97_CALLEES: &[usize] = &[2usize, 2usize];
const FN_98_CALLEES: &[usize] = &[4usize];
const FN_99_CALLEES: &[usize] = &[4usize, 0usize];
const FN_100_CALLEES: &[usize] = &[4usize];
const FN_101_CALLEES: &[usize] = &[4usize, 0usize];
const FN_102_CALLEES: &[usize] = &[4usize, 1usize];
const FN_103_CALLEES: &[usize] = &[4usize, 1usize];
const FN_104_CALLEES: &[usize] = &[4usize, 0usize];
const FN_105_CALLEES: &[usize] = &[4usize, 0usize];
const FN_106_GLOBALS: &[usize] = &[8usize];
const FN_106_CALLEES: &[usize] = &[2usize, 1usize, 4usize, 0usize, 0usize, 0usize];
const FN_107_GLOBALS: &[usize] = &[11usize, 12usize, 8usize];
const FN_107_CALLEES: &[usize] = &[1usize, 4usize, 2usize, 0usize, 0usize, 0usize];
const FN_108_GLOBALS: &[usize] = &[17usize, 18usize, 19usize, 20usize, 21usize, 22usize, 23usize, 24usize];
const FN_108_CALLEES: &[usize] = &[0usize, 1usize, 8usize];
const FN_109_GLOBALS: &[usize] = &[17usize, 18usize, 20usize, 19usize, 21usize, 8usize];
const FN_109_CALLEES: &[usize] = &[4usize, 13usize, 0usize, 5usize, 17usize, 6usize, 6usize, 7usize, 7usize, 1usize, 14usize, 2usize];
const FN_110_CALLEES: &[usize] = &[7usize, 1usize];
const FN_111_CALLEES: &[usize] = &[6usize, 7usize, 1usize];
const FN_112_CALLEES: &[usize] = &[1usize];
const FN_113_CALLEES: &[usize] = &[1usize];
const FN_114_GLOBALS: &[usize] = &[14usize];
const FN_114_CALLEES: &[usize] = &[12usize, 6usize, 0usize, 6usize, 7usize, 0usize, 7usize, 1usize, 6usize, 15usize];
const FN_115_GLOBALS: &[usize] = &[14usize, 8usize, 15usize];
const FN_115_CALLEES: &[usize] = &[12usize, 0usize, 1usize, 8usize, 13usize, 0usize, 5usize, 6usize, 6usize, 7usize, 7usize, 4usize, 5usize, 17usize, 1usize, 2usize, 8usize, 4usize, 14usize, 2usize];
const FN_116_GLOBALS: &[usize] = &[25usize];
const FN_117_CALLEES: &[usize] = &[2usize, 0usize, 0usize];
const FN_118_CALLEES: &[usize] = &[2usize, 0usize, 0usize];
const FN_119_CALLEES: &[usize] = &[2usize, 0usize, 0usize];
const FN_120_CALLEES: &[usize] = &[10usize, 0usize, 2usize];
const FN_121_CALLEES: &[usize] = &[2usize, 1usize, 0usize, 2usize];
const FN_122_CALLEES: &[usize] = &[10usize, 11usize, 2usize, 0usize, 16usize];
const FN_123_CALLEES: &[usize] = &[0usize, 2usize, 0usize];
const FN_124_CALLEES: &[usize] = &[16usize, 2usize, 0usize, 11usize, 1usize, 4usize, 0usize, 10usize, 9usize];
const FN_125_GLOBALS: &[usize] = &[14usize, 15usize];
const FN_125_CALLEES: &[usize] = &[10usize, 14usize, 2usize, 0usize, 0usize, 11usize, 5usize, 4usize, 8usize, 4usize, 8usize, 2usize, 9usize, 6usize, 2usize, 1usize, 2usize];
const FN_126_GLOBALS: &[usize] = &[17usize, 20usize];
const FN_126_CALLEES: &[usize] = &[6usize];
const FN_127_CALLEES: &[usize] = &[3usize, 6usize, 1usize, 6usize, 7usize];
const FN_128_CALLEES: &[usize] = &[6usize, 3usize, 1usize, 7usize];
const FN_129_GLOBALS: &[usize] = &[14usize];
const FN_129_CALLEES: &[usize] = &[4usize, 0usize, 4usize, 17usize, 7usize, 6usize, 2usize, 3usize];
const FN_130_GLOBALS: &[usize] = &[21usize, 12usize, 8usize];
const FN_130_CALLEES: &[usize] = &[2usize, 1usize, 0usize, 14usize, 3usize, 3usize, 10usize, 11usize, 2usize, 4usize, 9usize, 0usize, 0usize, 0usize, 12usize, 17usize, 7usize, 6usize, 1usize, 8usize, 12usize, 1usize, 14usize, 6usize];
const FN_131_GLOBALS: &[usize] = &[15usize, 14usize, 17usize, 20usize];
const FN_131_CALLEES: &[usize] = &[7usize, 3usize, 2usize, 4usize, 17usize, 6usize, 6usize, 1usize, 10usize, 14usize, 3usize, 12usize, 4usize, 5usize, 2usize, 1usize, 0usize, 2usize, 3usize, 0usize, 11usize, 8usize, 5usize, 4usize, 8usize, 4usize, 2usize, 5usize, 7usize, 8usize, 20usize, 2usize, 0usize, 5usize, 13usize, 19usize, 9usize, 6usize, 2usize, 6usize, 7usize, 0usize, 1usize, 0usize, 6usize, 7usize, 15usize];
const FN_132_GLOBALS: &[usize] = &[17usize, 8usize, 22usize, 24usize, 20usize];
const FN_132_CALLEES: &[usize] = &[3usize, 13usize, 0usize, 5usize, 1usize, 2usize, 0usize, 1usize, 0usize, 2usize, 11usize, 1usize, 4usize, 6usize, 17usize, 18usize, 14usize, 6usize, 14usize, 2usize];
const FN_133_CALLEES: &[usize] = &[10usize, 14usize, 1usize, 6usize, 16usize, 2usize, 0usize, 11usize];
const FN_134_CALLEES: &[usize] = &[1usize, 1usize, 0usize, 18usize, 14usize, 6usize];
const FN_135_GLOBALS: &[usize] = &[22usize, 23usize, 17usize, 20usize, 16usize, 24usize, 8usize];
const FN_135_CALLEES: &[usize] = &[13usize, 6usize, 7usize, 6usize, 7usize, 17usize, 2usize, 0usize, 6usize, 14usize, 2usize];
const FN_136_GLOBALS: &[usize] = &[16usize];
const FN_136_CALLEES: &[usize] = &[2usize, 0usize, 1usize];
const FN_137_GLOBALS: &[usize] = &[16usize, 22usize, 24usize, 8usize];
const FN_137_CALLEES: &[usize] = &[0usize, 7usize, 2usize, 10usize, 0usize, 0usize, 8usize, 13usize, 6usize, 4usize, 17usize, 7usize, 1usize, 14usize, 2usize];
const FN_138_GLOBALS: &[usize] = &[1usize, 0usize, 2usize, 3usize, 10usize, 11usize, 12usize, 13usize, 14usize, 15usize, 16usize, 17usize, 22usize, 25usize, 26usize, 8usize];
const FN_138_CALLEES: &[usize] = &[1usize, 2usize, 1usize, 0usize, 11usize, 2usize, 6usize, 5usize, 7usize, 8usize, 9usize, 10usize, 17usize];
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
static BOUND_SIG_P_I32_I32_I32_I32_I32_I32_I32_I32_RET_I32: &[crate::Bound_p_i32_i32_i32_i32_i32_i32_i32_i32_ret_i32] = &[
    crate::Bound_p_i32_i32_i32_i32_i32_i32_i32_i32_ret_i32 { func: crate::c_85899_97792::ChunkFns::f_b218de90020d, globals: FN_31_GLOBALS, callees: FN_31_CALLEES },
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
static BOUND_SIG_P_I32_I32_I32_I32_I32_I32_I32_RET_I32: &[crate::Bound_p_i32_i32_i32_i32_i32_i32_i32_ret_i32] = &[
    crate::Bound_p_i32_i32_i32_i32_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_517bbdea3a61, globals: FN_93_GLOBALS, callees: FN_93_CALLEES },
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
static BOUND_SIG_P_I32_I32_I32_I32_I32_I32_I32_RET_VOID: &[crate::Bound_p_i32_i32_i32_i32_i32_i32_i32_ret_void] = &[
    crate::Bound_p_i32_i32_i32_i32_i32_i32_i32_ret_void { func: crate::c_85899_93892::ChunkFns::f_528d930e602d, globals: FN_42_GLOBALS, callees: FN_42_CALLEES },
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
static BOUND_SIG_P_I32_I32_I32_I32_I32_I32_RET_I32: &[crate::Bound_p_i32_i32_i32_i32_i32_i32_ret_i32] = &[
    crate::Bound_p_i32_i32_i32_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_ba80a369cdcb, globals: FN_33_GLOBALS, callees: FN_33_CALLEES },
    crate::Bound_p_i32_i32_i32_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_e778f0ae372c, globals: FN_90_GLOBALS, callees: FN_90_CALLEES },
    crate::Bound_p_i32_i32_i32_i32_i32_i32_ret_i32 { func: crate::c_85899_87792::ChunkFns::f_fb262af913b2, globals: FN_94_GLOBALS, callees: FN_94_CALLEES },
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
static BOUND_SIG_P_I32_I32_I32_I32_I32_I32_RET_VOID: &[crate::Bound_p_i32_i32_i32_i32_i32_i32_ret_void] = &[
    crate::Bound_p_i32_i32_i32_i32_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_e3d239a807a4, globals: &[], callees: FN_124_CALLEES },
    crate::Bound_p_i32_i32_i32_i32_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_0eb69cfbacde, globals: FN_132_GLOBALS, callees: FN_132_CALLEES },
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
static BOUND_SIG_P_I32_I32_I32_I32_I32_RET_I32: &[crate::Bound_p_i32_i32_i32_i32_i32_ret_i32] = &[
    crate::Bound_p_i32_i32_i32_i32_i32_ret_i32 { func: crate::c_85899_93892::ChunkFns::f_08e1c638d613, globals: FN_38_GLOBALS, callees: FN_38_CALLEES },
    crate::Bound_p_i32_i32_i32_i32_i32_ret_i32 { func: crate::c_85899_93892::ChunkFns::f_a8363740d1c8, globals: FN_40_GLOBALS, callees: FN_40_CALLEES },
    crate::Bound_p_i32_i32_i32_i32_i32_ret_i32 { func: crate::c_85899_87792::ChunkFns::f_78eba914826e, globals: FN_91_GLOBALS, callees: FN_91_CALLEES },
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
static BOUND_SIG_P_I32_I32_I32_I32_I32_RET_VOID: &[crate::Bound_p_i32_i32_i32_i32_i32_ret_void] = &[
    crate::Bound_p_i32_i32_i32_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_07908887d2ca, globals: &[], callees: FN_105_CALLEES },
    crate::Bound_p_i32_i32_i32_i32_i32_ret_void { func: crate::c_85899_79292_1::ChunkFns::f_58deb2a85fe1, globals: &[], callees: FN_133_CALLEES },
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
static BOUND_SIG_P_I32_I32_I32_I32_RET_I32: &[crate::Bound_p_i32_i32_i32_i32_ret_i32] = &[
    crate::Bound_p_i32_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_7795c7de478d, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_i32_i32_ret_i32 { func: crate::c_85899_98092::ChunkFns::f_7e7316b9a8fe, globals: FN_24_GLOBALS, callees: FN_24_CALLEES },
    crate::Bound_p_i32_i32_i32_i32_ret_i32 { func: crate::c_85899_97792::ChunkFns::f_e1ee448cdcd2, globals: FN_30_GLOBALS, callees: FN_30_CALLEES },
    crate::Bound_p_i32_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_68ca86b09415, globals: FN_78_GLOBALS, callees: FN_78_CALLEES },
    crate::Bound_p_i32_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_152ec9a5d530, globals: FN_79_GLOBALS, callees: FN_79_CALLEES },
    crate::Bound_p_i32_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_d7490f9cf842, globals: &[], callees: FN_96_CALLEES },
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
static BOUND_SIG_P_I32_I32_I32_I32_RET_VOID: &[crate::Bound_p_i32_i32_i32_i32_ret_void] = &[
    crate::Bound_p_i32_i32_i32_i32_ret_void { func: crate::c_85899_93892::ChunkFns::f_df2b9e09dcca, globals: &[], callees: FN_43_CALLEES },
    crate::Bound_p_i32_i32_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_5d566823fb6c, globals: &[], callees: FN_101_CALLEES },
    crate::Bound_p_i32_i32_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_293c7d635704, globals: &[], callees: FN_112_CALLEES },
    crate::Bound_p_i32_i32_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_ff67f9ea2c5b, globals: &[], callees: FN_113_CALLEES },
    crate::Bound_p_i32_i32_i32_i32_ret_void { func: crate::c_85899_99792::ChunkFns::f_a4b58c35f972, globals: &[], callees: FN_122_CALLEES },
    crate::Bound_p_i32_i32_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_62dc121ee7e1, globals: &[], callees: FN_128_CALLEES },
    crate::Bound_p_i32_i32_i32_i32_ret_void { func: crate::c_85899_79292_1::ChunkFns::f_23c7b6aa52a7, globals: &[], callees: FN_134_CALLEES },
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
static BOUND_SIG_P_I32_I32_I32_RET_I32: &[crate::Bound_p_i32_i32_i32_ret_i32] = &[
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_31301d689ffa, globals: &[], callees: FN_29_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_31b8ad2e76d3, globals: &[], callees: FN_34_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_93892::ChunkFns::f_02c361261aaa, globals: FN_41_GLOBALS, callees: FN_41_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_87792::ChunkFns::f_7e9d84c9daa2, globals: FN_75_GLOBALS, callees: FN_75_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_87792::ChunkFns::f_c8fc591b4686, globals: FN_80_GLOBALS, callees: FN_80_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_87792::ChunkFns::f_116a2da5e6bc, globals: FN_81_GLOBALS, callees: FN_81_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_87717d28a526, globals: &[], callees: FN_92_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_f5420555aefe, globals: &[], callees: FN_111_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_ed3ff760701d, globals: &[], callees: FN_117_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_004b66504437, globals: &[], callees: FN_118_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_81cc25430500, globals: &[], callees: FN_119_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_a98617ea6ec7, globals: &[], callees: FN_120_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_99792::ChunkFns::f_34ae77fb28db, globals: &[], callees: FN_123_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_cbe6ccd58a27, globals: FN_130_GLOBALS, callees: FN_130_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_93892::ChunkFns::f_e4738ced9eb5, globals: FN_131_GLOBALS, callees: FN_131_CALLEES },
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
static BOUND_SIG_P_I32_I32_I32_RET_VOID: &[crate::Bound_p_i32_i32_i32_ret_void] = &[
    crate::Bound_p_i32_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_3028790a1b68, globals: &[], callees: FN_5_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_a4939e783822, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_6569a18ff452, globals: FN_37_GLOBALS, callees: FN_37_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_void { func: crate::c_85899_99792::ChunkFns::f_ed165787f004, globals: FN_64_GLOBALS, callees: FN_64_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_c60b575467fb, globals: &[], callees: FN_72_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_aca97c761850, globals: &[], callees: FN_73_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_dc65747ad20e, globals: &[], callees: FN_99_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_a562805ce2e0, globals: &[], callees: FN_100_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_534b766828b2, globals: FN_129_GLOBALS, callees: FN_129_CALLEES },
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
static BOUND_SIG_P_I32_I32_RET_I32: &[crate::Bound_p_i32_i32_ret_i32] = &[
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_21734e8b60c2, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_ab7d330bbfdb, globals: FN_11_GLOBALS, callees: FN_11_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_d015d1902229, globals: FN_13_GLOBALS, callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_e2ef5761c543, globals: FN_25_GLOBALS, callees: FN_25_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_98092::ChunkFns::f_35cc0348d5fa, globals: FN_27_GLOBALS, callees: FN_27_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_a657653b64d1, globals: FN_28_GLOBALS, callees: FN_28_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_f8be9dff3f3b, globals: FN_32_GLOBALS, callees: FN_32_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_f8be9dff3f3b, globals: FN_35_GLOBALS, callees: FN_35_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_b2d0372afbef, globals: FN_36_GLOBALS, callees: FN_36_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_97792::ChunkFns::f_8e3a351d9bba, globals: FN_39_GLOBALS, callees: FN_39_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_98a40a4e126c, globals: FN_51_GLOBALS, callees: FN_51_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_44bf2eebebc5, globals: FN_52_GLOBALS, callees: FN_52_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_87792::ChunkFns::f_d36e1b31003e, globals: FN_76_GLOBALS, callees: FN_76_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_f80c06cb23fe, globals: FN_77_GLOBALS, callees: FN_77_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_ca9b528594d9, globals: &[], callees: FN_82_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_6abe7e1add88, globals: &[], callees: FN_87_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_6f29b496c61b, globals: FN_88_GLOBALS, callees: FN_88_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_2488f523e9ab, globals: &[], callees: FN_97_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_1632f6fcfe85, globals: &[], callees: FN_110_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_92692::ChunkFns::f_ed28f33ecf20, globals: FN_125_GLOBALS, callees: FN_125_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_8ef52de6b9da, globals: &[], callees: FN_127_CALLEES },
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
static BOUND_SIG_P_I32_I32_RET_VOID: &[crate::Bound_p_i32_i32_ret_void] = &[
    crate::Bound_p_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_3c7cbd5626ed, globals: &[], callees: FN_3_CALLEES },
    crate::Bound_p_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_a561099042f5, globals: &[], callees: FN_4_CALLEES },
    crate::Bound_p_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_3e621adaab63, globals: &[], callees: FN_6_CALLEES },
    crate::Bound_p_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_3ef90e00723e, globals: &[], callees: FN_10_CALLEES },
    crate::Bound_p_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_e6decced73c7, globals: &[], callees: FN_21_CALLEES },
    crate::Bound_p_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_0c1b387702b2, globals: FN_74_GLOBALS, callees: FN_74_CALLEES },
    crate::Bound_p_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_76a01cf8b94d, globals: &[], callees: FN_98_CALLEES },
    crate::Bound_p_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_090940f804f9, globals: &[], callees: FN_102_CALLEES },
    crate::Bound_p_i32_i32_ret_void { func: crate::c_85899_93892::ChunkFns::f_a90324b40919, globals: FN_114_GLOBALS, callees: FN_114_CALLEES },
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
static BOUND_SIG_P_I32_I64_I64_RET_I32: &[crate::Bound_p_i32_i64_i64_ret_i32] = &[
    crate::Bound_p_i32_i64_i64_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_1b46daed85da, globals: FN_12_GLOBALS, callees: FN_12_CALLEES },
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
static BOUND_SIG_P_I32_I64_RET_VOID: &[crate::Bound_p_i32_i64_ret_void] = &[
    crate::Bound_p_i32_i64_ret_void { func: crate::c_85899_79292::ChunkFns::f_bfa2344d4114, globals: &[], callees: FN_22_CALLEES },
    crate::Bound_p_i32_i64_ret_void { func: crate::c_85899_79292::ChunkFns::f_3f09393fea7b, globals: &[], callees: FN_23_CALLEES },
    crate::Bound_p_i32_i64_ret_void { func: crate::c_85899_79292::ChunkFns::f_6ffdd7253944, globals: &[], callees: FN_103_CALLEES },
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
static BOUND_SIG_P_I32_RET_I32: &[crate::Bound_p_i32_ret_i32] = &[
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_ef9a8739b105, globals: FN_0_GLOBALS, callees: &[] },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_2d99912f2f20, globals: &[], callees: FN_2_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_11c79dbac1df, globals: &[], callees: &[] },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_39449c9f4c18, globals: &[], callees: &[] },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_9a0344420cf1, globals: &[], callees: &[] },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_25abec857daa, globals: &[], callees: FN_48_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_934dd2b0a015, globals: &[], callees: &[] },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_23ce34ed2575, globals: &[], callees: &[] },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_948b576b5c16, globals: &[], callees: FN_53_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92692::ChunkFns::f_9401d1c479c0, globals: &[], callees: &[] },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_3a2d43ddb2f5, globals: &[], callees: &[] },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_7c241a508314, globals: FN_71_GLOBALS, callees: FN_71_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_621550e77e3b, globals: FN_84_GLOBALS, callees: FN_84_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_c9226330fb94, globals: &[], callees: FN_95_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_b7b055bd5416, globals: FN_106_GLOBALS, callees: FN_106_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_239cb2f96e82, globals: FN_115_GLOBALS, callees: FN_115_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_c0e8f275fac9, globals: &[], callees: FN_121_CALLEES },
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
static BOUND_SIG_P_I32_RET_VOID: &[crate::Bound_p_i32_ret_void] = &[
    crate::Bound_p_i32_ret_void { func: crate::c_85899_98092::ChunkFns::f_62598845f747, globals: &[], callees: FN_16_CALLEES },
    crate::Bound_p_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_2640236e5dc3, globals: &[], callees: FN_20_CALLEES },
    crate::Bound_p_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_60b618eb1778, globals: FN_83_GLOBALS, callees: FN_83_CALLEES },
    crate::Bound_p_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_a1d02c5fb9c2, globals: FN_85_GLOBALS, callees: FN_85_CALLEES },
    crate::Bound_p_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_717050e4ee19, globals: FN_86_GLOBALS, callees: FN_86_CALLEES },
    crate::Bound_p_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_29d13b4bd6bc, globals: &[], callees: FN_104_CALLEES },
    crate::Bound_p_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_9386bb430299, globals: FN_126_GLOBALS, callees: FN_126_CALLEES },
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
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_6f9771270980, globals: FN_14_GLOBALS, callees: FN_14_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_13938ba711bf, globals: FN_15_GLOBALS, callees: FN_15_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_4dbaf9e3a937, globals: &[], callees: FN_18_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_4dbaf9e3a937, globals: &[], callees: FN_19_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_d7a26b86c8c3, globals: FN_44_GLOBALS, callees: FN_44_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_98092::ChunkFns::f_20003fcfab9c, globals: FN_54_GLOBALS, callees: FN_54_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_ed88189ac803, globals: FN_55_GLOBALS, callees: FN_55_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_98092::ChunkFns::f_4af71d2d97f0, globals: FN_56_GLOBALS, callees: FN_56_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_a802fe84ff33, globals: FN_57_GLOBALS, callees: FN_57_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_8d9629285dfc, globals: FN_58_GLOBALS, callees: FN_58_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_2d0311fa7074, globals: FN_59_GLOBALS, callees: FN_59_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_c4aab8465645, globals: FN_60_GLOBALS, callees: FN_60_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_9d76726450cf, globals: &[], callees: FN_61_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_892d46fb695c, globals: &[], callees: FN_62_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_99792::ChunkFns::f_e83370c8fe7b, globals: FN_66_GLOBALS, callees: FN_66_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_99792::ChunkFns::f_e517fffec819, globals: FN_67_GLOBALS, callees: FN_67_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85900_04092::ChunkFns::f_1d2f475e0c00, globals: FN_70_GLOBALS, callees: FN_70_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_af93137c5c01, globals: FN_107_GLOBALS, callees: FN_107_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_24fd9031937d, globals: FN_116_GLOBALS, callees: &[] },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_99792::ChunkFns::f_4393a4465e33, globals: FN_138_GLOBALS, callees: FN_138_CALLEES },
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
static BOUND_SIG_P_UNIT_RET_VOID: &[crate::Bound_p_unit_ret_void] = &[
    crate::Bound_p_unit_ret_void { func: crate::c_85899_79292::ChunkFns::f_53b3c125f8e9, globals: FN_17_GLOBALS, callees: FN_17_CALLEES },
    crate::Bound_p_unit_ret_void { func: crate::c_85899_79292::ChunkFns::f_93af50877a01, globals: FN_26_GLOBALS, callees: FN_26_CALLEES },
    crate::Bound_p_unit_ret_void { func: crate::c_85899_79292::ChunkFns::f_b2a9ea0711d2, globals: FN_45_GLOBALS, callees: FN_45_CALLEES },
    crate::Bound_p_unit_ret_void { func: crate::c_85899_98092::ChunkFns::f_6e7bc7586206, globals: &[], callees: FN_63_CALLEES },
    crate::Bound_p_unit_ret_void { func: crate::c_85899_99792::ChunkFns::f_9559ac14dd6f, globals: FN_65_GLOBALS, callees: &[] },
    crate::Bound_p_unit_ret_void { func: crate::c_85899_87792::ChunkFns::f_fc806df2edb4, globals: FN_89_GLOBALS, callees: FN_89_CALLEES },
    crate::Bound_p_unit_ret_void { func: crate::c_85899_79292::ChunkFns::f_81a252fc3c25, globals: FN_108_GLOBALS, callees: FN_108_CALLEES },
    crate::Bound_p_unit_ret_void { func: crate::c_85899_79292::ChunkFns::f_3d58e151fe65, globals: FN_109_GLOBALS, callees: FN_109_CALLEES },
    crate::Bound_p_unit_ret_void { func: crate::c_85899_79292_1::ChunkFns::f_c71faaa3ebde, globals: FN_135_GLOBALS, callees: FN_135_CALLEES },
    crate::Bound_p_unit_ret_void { func: crate::c_85899_79292_1::ChunkFns::f_9809d582b386, globals: FN_136_GLOBALS, callees: FN_136_CALLEES },
    crate::Bound_p_unit_ret_void { func: crate::c_85899_79292_1::ChunkFns::f_6356e8ed2b9e, globals: FN_137_GLOBALS, callees: FN_137_CALLEES },
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
    sig_p_i32_i32_i32_i32_i32_i32_i32_i32_ret_i32: BOUND_SIG_P_I32_I32_I32_I32_I32_I32_I32_I32_RET_I32,
    sig_p_i32_i32_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i32_i32_ret_i32: BOUND_SIG_P_I32_I32_I32_I32_I32_I32_I32_RET_I32,
    sig_p_i32_i32_i32_i32_i32_i32_i32_ret_void: BOUND_SIG_P_I32_I32_I32_I32_I32_I32_I32_RET_VOID,
    sig_p_i32_i32_i32_i32_i32_i32_ret_i32: BOUND_SIG_P_I32_I32_I32_I32_I32_I32_RET_I32,
    sig_p_i32_i32_i32_i32_i32_i32_ret_void: BOUND_SIG_P_I32_I32_I32_I32_I32_I32_RET_VOID,
    sig_p_i32_i32_i32_i32_i32_i64_i32_i32_i32_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i64_i32_i32_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i64_i32_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i64_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_ret_i32: BOUND_SIG_P_I32_I32_I32_I32_I32_RET_I32,
    sig_p_i32_i32_i32_i32_i32_ret_void: BOUND_SIG_P_I32_I32_I32_I32_I32_RET_VOID,
    sig_p_i32_i32_i32_i32_i64_ret_i32: &[],
    sig_p_i32_i32_i32_i32_ret_i32: BOUND_SIG_P_I32_I32_I32_I32_RET_I32,
    sig_p_i32_i32_i32_i32_ret_void: BOUND_SIG_P_I32_I32_I32_I32_RET_VOID,
    sig_p_i32_i32_i32_i64_ret_void: &[],
    sig_p_i32_i32_i32_ret_i32: BOUND_SIG_P_I32_I32_I32_RET_I32,
    sig_p_i32_i32_i32_ret_void: BOUND_SIG_P_I32_I32_I32_RET_VOID,
    sig_p_i32_i32_i64_i32_i32_i64_i32_ret_i32: &[],
    sig_p_i32_i32_i64_i32_i32_ret_i32: &[],
    sig_p_i32_i32_i64_i32_ret_i32: &[],
    sig_p_i32_i32_i64_ret_i32: &[],
    sig_p_i32_i32_i64_ret_void: &[],
    sig_p_i32_i32_ret_i32: BOUND_SIG_P_I32_I32_RET_I32,
    sig_p_i32_i32_ret_i64: &[],
    sig_p_i32_i32_ret_void: BOUND_SIG_P_I32_I32_RET_VOID,
    sig_p_i32_i64_i32_i32_ret_i32: &[],
    sig_p_i32_i64_i32_i32_ret_i64: &[],
    sig_p_i32_i64_i32_ret_i32: &[],
    sig_p_i32_i64_i32_ret_i64: &[],
    sig_p_i32_i64_i32_ret_void: &[],
    sig_p_i32_i64_i64_ret_i32: BOUND_SIG_P_I32_I64_I64_RET_I32,
    sig_p_i32_i64_i64_ret_i64: &[],
    sig_p_i32_i64_i64_ret_void: &[],
    sig_p_i32_i64_ret_i32: &[],
    sig_p_i32_i64_ret_i64: &[],
    sig_p_i32_i64_ret_void: BOUND_SIG_P_I32_I64_RET_VOID,
    sig_p_i32_ret_i32: BOUND_SIG_P_I32_RET_I32,
    sig_p_i32_ret_i64: &[],
    sig_p_i32_ret_void: BOUND_SIG_P_I32_RET_VOID,
    sig_p_i64_i32_ret_i64: &[],
    sig_p_i64_i64_ret_i32: &[],
    sig_p_i64_i64_ret_i64: &[],
    sig_p_i64_ret_i32: &[],
    sig_p_i64_ret_i64: &[],
    sig_p_unit_ret_i32: BOUND_SIG_P_UNIT_RET_I32,
    sig_p_unit_ret_void: BOUND_SIG_P_UNIT_RET_VOID,
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

const GRAPH_TARGETS: &[(u128, [u8; 32])] = &[
    (2229904u128, [0xbb, 0x63, 0xe3, 0x2e, 0x22, 0x1e, 0x6d, 0xfa, 0x97, 0xd3, 0xff, 0x5f, 0xd6, 0x67, 0x41, 0x1c, 0x4d, 0x49, 0x9d, 0xf7, 0xa5, 0x13, 0x7b, 0xc0, 0xa9, 0xcc, 0x8a, 0x68, 0x64, 0x19, 0x96, 0xba]),
];

const MODULE_META: SubstituteModuleMeta = SubstituteModuleMeta {
    name: "85900_04092/f1680@l8590004215",
    accounting_version: 2u32,
    wasm_sha256: [0x26, 0x46, 0xd4, 0x6f, 0xe2, 0xdb, 0x8e, 0x9b, 0xe2, 0x3d, 0xca, 0x92, 0x1b, 0xff, 0xd2, 0x6b, 0xe8, 0xb1, 0xca, 0xff, 0x7a, 0xc1, 0xc6, 0x53, 0x97, 0xe7, 0xc5, 0x89, 0x00, 0x50, 0x82, 0x6e],
    entry_func_name: "step",
    has_start_func: false,
    initial_memory_pages: 1usize,
    initial_table_elements: 0usize,
    static_data_end: 0usize,
    num_memories: 1usize,
    num_tables: 0usize,
    graph_targets: GRAPH_TARGETS,
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
    let g1: i32 = 0i32;
    let g2: i32 = 0i32;
    let g3: i32 = 0i32;
    let g4: i32 = 0i32;
    let g5: i32 = 0i32;
    let g6: i32 = 0i32;
    let g7: i32 = 0i32;
    let g8: i32 = 0i32;
    let g9: i32 = 0i32;
    let g10: i32 = 0i32;
    let g11: i32 = 0i32;
    let g12: i32 = 0i32;
    let g13: i32 = 0i32;
    let g14: i32 = 0i32;
    let g15: i32 = 0i32;
    let g16: i32 = 0i32;
    let g17: i32 = 0i32;
    let g18: i32 = 0i32;
    let g19: i32 = 0i32;
    let g20: i32 = 0i32;
    let g21: i32 = 0i32;
    let g22: i32 = 0i32;
    let g23: i32 = 0i32;
    let g24: i32 = 0i32;
    let g25: i32 = 1i32;
    let g26: i32 = 0i32;
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
    vm.globals_i32 = vec![g0, g1, g2, g3, g4, g5, g6, g7, g8, g9, g10, g11, g12, g13, g14, g15, g16, g17, g18, g19, g20, g21, g22, g23, g24, g25, g26];
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
            "arena_ptr" => self.vm.globals_i32.get(0usize).copied(),
            "arena_cap" => self.vm.globals_i32.get(1usize).copied(),
            "cur_ptr" => self.vm.globals_i32.get(2usize).copied(),
            "cur_end" => self.vm.globals_i32.get(3usize).copied(),
            "dec_cp" => self.vm.globals_i32.get(4usize).copied(),
            "dec_raw_ptr" => self.vm.globals_i32.get(5usize).copied(),
            "dec_raw_len" => self.vm.globals_i32.get(6usize).copied(),
            "ret_ptr" => self.vm.globals_i32.get(7usize).copied(),
            "ret_len" => self.vm.globals_i32.get(8usize).copied(),
            "ret_aux" => self.vm.globals_i32.get(9usize).copied(),
            "builtin_entries" => self.vm.globals_i32.get(10usize).copied(),
            "out_globals" => self.vm.globals_i32.get(11usize).copied(),
            "out_routines" => self.vm.globals_i32.get(12usize).copied(),
            "alloc_fn_plus1" => self.vm.globals_i32.get(13usize).copied(),
            "type_table" => self.vm.globals_i32.get(14usize).copied(),
            "all_structs" => self.vm.globals_i32.get(15usize).copied(),
            "modules_buf" => self.vm.globals_i32.get(16usize).copied(),
            "uses_shadow" => self.vm.globals_i32.get(17usize).copied(),
            "sh_base_glob" => self.vm.globals_i32.get(18usize).copied(),
            "sh_lim_glob" => self.vm.globals_i32.get(19usize).copied(),
            "sh_sp_glob" => self.vm.globals_i32.get(20usize).copied(),
            "sh_push_fn" => self.vm.globals_i32.get(21usize).copied(),
            "has_top_vals" => self.vm.globals_i32.get(22usize).copied(),
            "init_flag_glob" => self.vm.globals_i32.get(23usize).copied(),
            "init_fn_idx" => self.vm.globals_i32.get(24usize).copied(),
            "next_rout_uid" => self.vm.globals_i32.get(25usize).copied(),
            "self_target_stk" => self.vm.globals_i32.get(26usize).copied(),
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
        vm.begin_fast_epoch();
        let res = DISPATCH.call_direct_p_unit_ret_i32(vm, 19usize, host);
        vm.end_fast_epoch();
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

