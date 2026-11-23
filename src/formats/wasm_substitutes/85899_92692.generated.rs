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

//! Generated native Rust WebAssembly substitute chunk `85899_92692`.
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
    pub(crate) fn f_9401d1c479c0(vm: &mut SubstituteVm, dispatch: &'static crate::ModuleDispatch, g_map: &'static [usize], c_map: &'static [usize], p0: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i32_2: i32 = 0i32;
    let mut s_i32_3: i32 = 0i32;
    let mut s_i32_4: i32 = 0i32;
    vm.deduct_1(host)?;
    s_i32_0 = l0;
    s_i32_1 = 1116629i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    s_i32_1 = l0;
    s_i32_2 = 1116636i32;
    s_i32_1 = ops::i32_eq(s_i32_1, s_i32_2);
    s_i32_0 = ops::i32_or(s_i32_0, s_i32_1);
    s_i32_1 = l0;
    s_i32_2 = 1116644i32;
    s_i32_1 = ops::i32_eq(s_i32_1, s_i32_2);
    s_i32_2 = l0;
    s_i32_3 = 1116655i32;
    s_i32_2 = ops::i32_eq(s_i32_2, s_i32_3);
    s_i32_1 = ops::i32_or(s_i32_1, s_i32_2);
    s_i32_0 = ops::i32_or(s_i32_0, s_i32_1);
    s_i32_1 = l0;
    s_i32_2 = 1116641i32;
    s_i32_1 = ops::i32_ge_u(s_i32_1, s_i32_2);
    s_i32_2 = l0;
    s_i32_3 = 1116643i32;
    s_i32_2 = ops::i32_le_u(s_i32_2, s_i32_3);
    s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
    s_i32_2 = l0;
    s_i32_3 = 1116697i32;
    s_i32_2 = ops::i32_eq(s_i32_2, s_i32_3);
    s_i32_3 = l0;
    s_i32_4 = 1116728i32;
    s_i32_3 = ops::i32_eq(s_i32_3, s_i32_4);
    s_i32_2 = ops::i32_or(s_i32_2, s_i32_3);
    s_i32_1 = ops::i32_or(s_i32_1, s_i32_2);
    s_i32_0 = ops::i32_or(s_i32_0, s_i32_1);
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn f_ed28f33ecf20(vm: &mut SubstituteVm, dispatch: &'static crate::ModuleDispatch, g_map: &'static [usize], c_map: &'static [usize], p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i32 = 0i32;
    let mut l7: i32 = 0i32;
    let mut l8: i32 = 0i32;
    let mut l9: i32 = 0i32;
    let mut l10: i32 = 0i32;
    let mut l11: i32 = 0i32;
    let mut l12: i32 = 0i32;
    let mut l13: i32 = 0i32;
    let mut l14: i32 = 0i32;
    let mut l15: i32 = 0i32;
    let mut l16: i32 = 0i32;
    let mut l17: i32 = 0i32;
    let mut l18: i32 = 0i32;
    let mut l19: i32 = 0i32;
    let mut l20: i32 = 0i32;
    let mut l21: i32 = 0i32;
    let mut l22: i32 = 0i32;
    let mut l23: i32 = 0i32;
    let mut l24: i32 = 0i32;
    let mut l25: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i32_2: i32 = 0i32;
    let mut s_i32_3: i32 = 0i32;
    let mut s_i32_4: i32 = 0i32;
    let mut s_i32_5: i32 = 0i32;
    vm.deduct_1(host)?;
    s_i32_0 = l1;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    l2 = s_i32_0;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    l3 = s_i32_0;
    s_i32_0 = l2;
    s_i32_1 = 1i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 2i32;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l2;
    s_i32_1 = 2i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l2;
    s_i32_1 = 3i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_3: {
        if s_i32_0 != 0 {
            s_i32_0 = 8i32;
            s_i32_1 = 3i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t0 = __call_res?;
            s_i32_0 = __t0;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l2;
    s_i32_1 = 4i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_4: {
        if s_i32_0 != 0 {
            s_i32_0 = 8i32;
            s_i32_1 = l3;
            s_i32_2 = l1;
            s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 4u32)?;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_1, s_i32_2, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_1 = __t1;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_0 = __t2;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l2;
    s_i32_1 = 5i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_5: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
            l4 = s_i32_0;
            s_i32_0 = l4;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 2usize, s_i32_0, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i32_0 = __t3;
            l5 = s_i32_0;
            s_i32_0 = l5;
            s_i32_0 = ops::i32_eqz(s_i32_0);
            'blk_6: {
                if s_i32_0 != 0 {
                    s_i32_0 = 8i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l5;
            s_i32_1 = 2i32;
            s_i32_0 = ops::i32_shl(s_i32_0, s_i32_1);
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
            vm.exit_call();
            let __t4 = __call_res?;
            s_i32_0 = __t4;
            l7 = s_i32_0;
            s_i32_0 = 0i32;
            l6 = s_i32_0;
            'blk_7: {
                'blk_8: loop {
                    vm.deduct_1(host)?;
                    s_i32_0 = l6;
                    s_i32_1 = l5;
                    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                    if s_i32_0 != 0 {
                        break 'blk_7;
                    }
                    s_i32_0 = l0;
                    s_i32_1 = l4;
                    s_i32_2 = l6;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    let __t5 = __call_res?;
                    s_i32_1 = __t5;
                    vm.enter_call()?;
                    let __call_res = Self::f_ed28f33ecf20(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t6 = __call_res?;
                    s_i32_0 = __t6;
                    l8 = s_i32_0;
                    s_i32_0 = l8;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                    'blk_9: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 1i32;
                            return Ok(s_i32_0);
                        }
                    }
                    s_i32_0 = l7;
                    s_i32_1 = l6;
                    s_i32_2 = 2i32;
                    s_i32_1 = ops::i32_shl(s_i32_1, s_i32_2);
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    s_i32_1 = l8;
                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l6;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    l6 = s_i32_0;
                    continue 'blk_8;
                    break 'blk_8;
                }
            }
            s_i32_0 = l5;
            s_i32_1 = l7;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 5usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t7 = __call_res?;
            s_i32_0 = __t7;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l2;
    s_i32_1 = 6i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_10: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 16u32)?;
            l4 = s_i32_0;
            s_i32_0 = l4;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 2usize, s_i32_0, host);
            vm.exit_call();
            let __t8 = __call_res?;
            s_i32_0 = __t8;
            l5 = s_i32_0;
            s_i32_0 = 0i32;
            l6 = s_i32_0;
            'blk_11: {
                'blk_12: loop {
                    vm.deduct_1(host)?;
                    s_i32_0 = l6;
                    s_i32_1 = l5;
                    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                    if s_i32_0 != 0 {
                        break 'blk_11;
                    }
                    s_i32_0 = l0;
                    s_i32_1 = l4;
                    s_i32_2 = l6;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    let __t9 = __call_res?;
                    s_i32_1 = __t9;
                    vm.enter_call()?;
                    let __call_res = Self::f_ed28f33ecf20(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t10 = __call_res?;
                    s_i32_0 = __t10;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                    'blk_13: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 1i32;
                            return Ok(s_i32_0);
                        }
                    }
                    s_i32_0 = l6;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    l6 = s_i32_0;
                    continue 'blk_12;
                    break 'blk_12;
                }
            }
            s_i32_0 = l1;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
            'blk_14: {
                if s_i32_0 != 0 {
                    s_i32_0 = l3;
                    s_i32_1 = l1;
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 8u32)?;
                    s_i32_2 = l1;
                    s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 12u32)?;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 6usize, s_i32_0, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    let __t11 = __call_res?;
                    s_i32_0 = __t11;
                } else {
                    s_i32_0 = l3;
                    s_i32_1 = l1;
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 8u32)?;
                    s_i32_2 = l1;
                    s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 12u32)?;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 7usize, s_i32_0, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    let __t12 = __call_res?;
                    s_i32_0 = __t12;
                }
            }
            l9 = s_i32_0;
            s_i32_0 = l9;
            s_i32_1 = 0i32;
            s_i32_0 = ops::i32_lt_s(s_i32_0, s_i32_1);
            'blk_15: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = l9;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 8usize, s_i32_0, host);
            vm.exit_call();
            let __t13 = __call_res?;
            s_i32_0 = __t13;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l2;
    s_i32_1 = 7i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_16: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
            vm.enter_call()?;
            let __call_res = Self::f_ed28f33ecf20(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t14 = __call_res?;
            s_i32_0 = __t14;
            l10 = s_i32_0;
            s_i32_0 = l10;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_17: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l10;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_void(vm, c_map, 9usize, s_i32_0, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
            s_i32_1 = l10;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t15 = __call_res?;
            s_i32_0 = __t15;
            l11 = s_i32_0;
            s_i32_0 = l11;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
            s_i32_1 = 10i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_18: {
                if s_i32_0 != 0 {
                    s_i32_0 = l11;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
                } else {
                    s_i32_0 = vm.get_i32(*g_map.get(1usize).ok_or(WasmTrap::Unreachable)?)?;
                    s_i32_1 = l11;
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t16 = __call_res?;
                    s_i32_0 = __t16;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 32u32)?;
                }
            }
            l5 = s_i32_0;
            s_i32_0 = l1;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
            l13 = s_i32_0;
            s_i32_0 = l13;
            s_i32_1 = l5;
            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
            'blk_19: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = l11;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 20u32)?;
            l12 = s_i32_0;
            s_i32_0 = l12;
            s_i32_1 = l13;
            s_i32_2 = 4i32;
            s_i32_1 = ops::i32_shl(s_i32_1, s_i32_2);
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 12u32)?;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l2;
    s_i32_1 = 8i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_20: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
            vm.enter_call()?;
            let __call_res = Self::f_ed28f33ecf20(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t17 = __call_res?;
            s_i32_0 = __t17;
            l10 = s_i32_0;
            s_i32_0 = l10;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_21: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l0;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 12u32)?;
            vm.enter_call()?;
            let __call_res = Self::f_ed28f33ecf20(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t18 = __call_res?;
            s_i32_0 = __t18;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_22: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l10;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l2;
    s_i32_1 = 9i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_23: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 8u32)?;
            vm.enter_call()?;
            let __call_res = Self::f_ed28f33ecf20(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t19 = __call_res?;
            s_i32_0 = __t19;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_24: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = 9i32;
            s_i32_1 = l3;
            s_i32_2 = l1;
            s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 4u32)?;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_1, s_i32_2, host);
            vm.exit_call();
            let __t20 = __call_res?;
            s_i32_1 = __t20;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t21 = __call_res?;
            s_i32_0 = __t21;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l2;
    s_i32_1 = 10i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_25: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            l15 = s_i32_0;
            s_i32_0 = l1;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
            'blk_26: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                    s_i32_1 = l3;
                    s_i32_2 = l1;
                    s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 4u32)?;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    let __t22 = __call_res?;
                    s_i32_1 = __t22;
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    l15 = s_i32_0;
                }
            }
            s_i32_0 = l1;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
            s_i32_0 = ops::i32_eqz(s_i32_0);
            'blk_27: {
                if s_i32_0 != 0 {
                    s_i32_0 = l0;
                    s_i32_1 = l1;
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 12u32)?;
                    s_i32_2 = l1;
                    s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 16u32)?;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 10usize, s_i32_0, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    let __t23 = __call_res?;
                    s_i32_0 = __t23;
                    l14 = s_i32_0;
                    s_i32_0 = l14;
                    'blk_28: {
                        if s_i32_0 != 0 {
                            s_i32_0 = l15;
                            s_i32_1 = l15;
                            s_i32_2 = 1i32;
                            s_i32_1 = ops::i32_sub(s_i32_1, s_i32_2);
                            s_i32_2 = l14;
                            s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 16u32)?;
                            s_i32_1 = ops::i32_ne(s_i32_1, s_i32_2);
                            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                            'blk_29: {
                                if s_i32_0 != 0 {
                                    return Err(WasmTrap::Unreachable);
                                }
                            }
                            s_i32_0 = l14;
                            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 16u32)?;
                            return Ok(s_i32_0);
                        }
                    }
                }
            }
            s_i32_0 = l3;
            s_i32_1 = l15;
            s_i32_2 = l1;
            s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 8u32)?;
            s_i32_3 = l1;
            s_i32_3 = ops::i32_load(vm, 0, s_i32_3, 12u32)?;
            s_i32_4 = l1;
            s_i32_4 = ops::i32_load(vm, 0, s_i32_4, 16u32)?;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_i32_i32_ret_i32(vm, c_map, 11usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, s_i32_4, host);
            vm.exit_call();
            let __t24 = __call_res?;
            s_i32_0 = __t24;
            l16 = s_i32_0;
            s_i32_0 = l16;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 28u32)?;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l2;
    s_i32_1 = 11i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_30: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 16u32)?;
            'blk_31: {
                if s_i32_0 != 0 {
                    s_i32_0 = l0;
                    s_i32_1 = l1;
                    vm.deduct_1(host)?;
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 16u32)?;
                    vm.enter_call()?;
                    let __call_res = Self::f_ed28f33ecf20(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t25 = __call_res?;
                    s_i32_0 = __t25;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                    'blk_32: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 1i32;
                            return Ok(s_i32_0);
                        }
                    }
                }
            }
            s_i32_0 = 0i32;
            return Ok(s_i32_0);
        }
    }
    vm.deduct_1(host)?;
    s_i32_0 = l2;
    s_i32_1 = 12i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    s_i32_1 = l2;
    s_i32_2 = 13i32;
    s_i32_1 = ops::i32_eq(s_i32_1, s_i32_2);
    s_i32_0 = ops::i32_or(s_i32_0, s_i32_1);
    'blk_33: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 16u32)?;
            vm.enter_call()?;
            let __call_res = Self::f_ed28f33ecf20(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t26 = __call_res?;
            s_i32_0 = __t26;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_34: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = 0i32;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l2;
    s_i32_1 = 14i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    s_i32_1 = l2;
    s_i32_2 = 24i32;
    s_i32_1 = ops::i32_eq(s_i32_1, s_i32_2);
    s_i32_0 = ops::i32_or(s_i32_0, s_i32_1);
    'blk_35: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l2;
    s_i32_1 = 18i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    s_i32_1 = l2;
    s_i32_2 = 19i32;
    s_i32_1 = ops::i32_eq(s_i32_1, s_i32_2);
    s_i32_0 = ops::i32_or(s_i32_0, s_i32_1);
    'blk_36: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l2;
    s_i32_1 = 20i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    s_i32_1 = l2;
    s_i32_2 = 21i32;
    s_i32_1 = ops::i32_eq(s_i32_1, s_i32_2);
    s_i32_0 = ops::i32_or(s_i32_0, s_i32_1);
    s_i32_1 = l2;
    s_i32_2 = 22i32;
    s_i32_1 = ops::i32_eq(s_i32_1, s_i32_2);
    s_i32_2 = l2;
    s_i32_3 = 23i32;
    s_i32_2 = ops::i32_eq(s_i32_2, s_i32_3);
    s_i32_1 = ops::i32_or(s_i32_1, s_i32_2);
    s_i32_0 = ops::i32_or(s_i32_0, s_i32_1);
    'blk_37: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l2;
    s_i32_1 = 15i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_38: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 16u32)?;
            l4 = s_i32_0;
            s_i32_0 = l4;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 2usize, s_i32_0, host);
            vm.exit_call();
            let __t27 = __call_res?;
            s_i32_0 = __t27;
            l5 = s_i32_0;
            s_i32_0 = 4i32;
            s_i32_1 = l5;
            s_i32_2 = 2i32;
            s_i32_1 = ops::i32_shl(s_i32_1, s_i32_2);
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
            vm.exit_call();
            let __t28 = __call_res?;
            s_i32_0 = __t28;
            l7 = s_i32_0;
            s_i32_0 = 0i32;
            l6 = s_i32_0;
            'blk_39: {
                'blk_40: loop {
                    vm.deduct_1(host)?;
                    s_i32_0 = l6;
                    s_i32_1 = l5;
                    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                    if s_i32_0 != 0 {
                        break 'blk_39;
                    }
                    s_i32_0 = l0;
                    s_i32_1 = l4;
                    s_i32_2 = l6;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    let __t29 = __call_res?;
                    s_i32_1 = __t29;
                    vm.enter_call()?;
                    let __call_res = Self::f_ed28f33ecf20(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t30 = __call_res?;
                    s_i32_0 = __t30;
                    l8 = s_i32_0;
                    s_i32_0 = l8;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                    'blk_41: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 1i32;
                            return Ok(s_i32_0);
                        }
                    }
                    s_i32_0 = l7;
                    s_i32_1 = l6;
                    s_i32_2 = 2i32;
                    s_i32_1 = ops::i32_shl(s_i32_1, s_i32_2);
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    s_i32_1 = l8;
                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l6;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    l6 = s_i32_0;
                    continue 'blk_40;
                    break 'blk_40;
                }
            }
            s_i32_0 = l1;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
            s_i32_0 = ops::i32_eqz(s_i32_0);
            'blk_42: {
                if s_i32_0 != 0 {
                    s_i32_0 = l0;
                    s_i32_1 = l1;
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 8u32)?;
                    s_i32_2 = l1;
                    s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 12u32)?;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 12usize, s_i32_0, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    let __t31 = __call_res?;
                    s_i32_0 = __t31;
                    l17 = s_i32_0;
                    s_i32_0 = l17;
                    'blk_43: {
                        if s_i32_0 != 0 {
                            s_i32_0 = l17;
                            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 16u32)?;
                            l18 = s_i32_0;
                            s_i32_0 = l18;
                            s_i32_1 = l5;
                            s_i32_2 = l7;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 13usize, s_i32_0, s_i32_1, s_i32_2, host);
                            vm.exit_call();
                            let __t32 = __call_res?;
                            s_i32_0 = __t32;
                            s_i32_0 = ops::i32_eqz(s_i32_0);
                            'blk_44: {
                                if s_i32_0 != 0 {
                                    return Err(WasmTrap::Unreachable);
                                }
                            }
                            s_i32_0 = l18;
                            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 32u32)?;
                            return Ok(s_i32_0);
                        }
                    }
                }
            }
            s_i32_0 = l3;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
            s_i32_2 = l1;
            s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 8u32)?;
            s_i32_3 = l1;
            s_i32_3 = ops::i32_load(vm, 0, s_i32_3, 12u32)?;
            s_i32_4 = l5;
            s_i32_5 = l7;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_i32_i32_i32_ret_i32(vm, c_map, 14usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, s_i32_4, s_i32_5, host);
            vm.exit_call();
            let __t33 = __call_res?;
            s_i32_0 = __t33;
            l18 = s_i32_0;
            s_i32_0 = l18;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 32u32)?;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l2;
    s_i32_1 = 16i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_45: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
            l4 = s_i32_0;
            s_i32_0 = l4;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 2usize, s_i32_0, host);
            vm.exit_call();
            let __t34 = __call_res?;
            s_i32_0 = __t34;
            l5 = s_i32_0;
            s_i32_0 = l5;
            s_i32_0 = ops::i32_eqz(s_i32_0);
            'blk_46: {
                if s_i32_0 != 0 {
                    s_i32_0 = 0i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l0;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 28u32)?;
            l19 = s_i32_0;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
            vm.exit_call();
            let __t35 = __call_res?;
            s_i32_0 = __t35;
            l20 = s_i32_0;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 15usize, s_i32_0, host);
            vm.exit_call();
            let __t36 = __call_res?;
            s_i32_0 = __t36;
            l21 = s_i32_0;
            s_i32_0 = l20;
            s_i32_1 = l21;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l20;
            s_i32_1 = 16i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 15usize, s_i32_1, host);
            vm.exit_call();
            let __t37 = __call_res?;
            s_i32_1 = __t37;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = 0i32;
            l6 = s_i32_0;
            'blk_47: {
                'blk_48: loop {
                    vm.deduct_1(host)?;
                    s_i32_0 = l6;
                    s_i32_1 = l5;
                    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                    if s_i32_0 != 0 {
                        break 'blk_47;
                    }
                    s_i32_0 = l4;
                    s_i32_1 = l6;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t38 = __call_res?;
                    s_i32_0 = __t38;
                    l22 = s_i32_0;
                    s_i32_0 = l22;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                    s_i32_1 = 14i32;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                    'blk_49: {
                        if s_i32_0 != 0 {
                            s_i32_0 = l22;
                            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
                            l18 = s_i32_0;
                            s_i32_0 = l18;
                            s_i32_1 = l3;
                            s_i32_2 = l18;
                            s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 12u32)?;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_1, s_i32_2, host);
                            vm.exit_call();
                            let __t39 = __call_res?;
                            s_i32_1 = __t39;
                            ops::i32_store(vm, 0, s_i32_0, 32u32, s_i32_1)?;
                            s_i32_0 = l18;
                            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 24u32)?;
                            l11 = s_i32_0;
                            s_i32_0 = 0i32;
                            l13 = s_i32_0;
                            'blk_50: {
                                'blk_51: loop {
                                    vm.deduct_1(host)?;
                                    s_i32_0 = l13;
                                    s_i32_1 = l11;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 2usize, s_i32_1, host);
                                    vm.exit_call();
                                    let __t40 = __call_res?;
                                    s_i32_1 = __t40;
                                    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                                    if s_i32_0 != 0 {
                                        break 'blk_50;
                                    }
                                    s_i32_0 = l11;
                                    s_i32_1 = l13;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, host);
                                    vm.exit_call();
                                    let __t41 = __call_res?;
                                    s_i32_0 = __t41;
                                    l14 = s_i32_0;
                                    s_i32_0 = l14;
                                    s_i32_1 = l3;
                                    s_i32_2 = l14;
                                    s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 0u32)?;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_1, s_i32_2, host);
                                    vm.exit_call();
                                    let __t42 = __call_res?;
                                    s_i32_1 = __t42;
                                    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                                    s_i32_0 = l13;
                                    s_i32_1 = 1i32;
                                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                                    l13 = s_i32_0;
                                    continue 'blk_51;
                                    break 'blk_51;
                                }
                            }
                            s_i32_0 = 32i32;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
                            vm.exit_call();
                            let __t43 = __call_res?;
                            s_i32_0 = __t43;
                            l17 = s_i32_0;
                            s_i32_0 = l17;
                            s_i32_1 = l18;
                            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 16u32)?;
                            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                            s_i32_0 = l17;
                            s_i32_1 = l18;
                            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 20u32)?;
                            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                            s_i32_0 = l17;
                            s_i32_1 = l18;
                            ops::i32_store(vm, 0, s_i32_0, 16u32, s_i32_1)?;
                            s_i32_0 = l20;
                            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
                            s_i32_1 = l17;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 16usize, s_i32_0, s_i32_1, host);
                            vm.exit_call();
                            __call_res?;
                        }
                    }
                    s_i32_0 = l6;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    l6 = s_i32_0;
                    continue 'blk_48;
                    break 'blk_48;
                }
            }
            s_i32_0 = l19;
            s_i32_1 = l20;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 16usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = 0i32;
            l8 = s_i32_0;
            s_i32_0 = 0i32;
            l6 = s_i32_0;
            'blk_52: {
                'blk_53: loop {
                    vm.deduct_1(host)?;
                    s_i32_0 = l6;
                    s_i32_1 = l5;
                    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                    if s_i32_0 != 0 {
                        break 'blk_52;
                    }
                    s_i32_0 = l4;
                    s_i32_1 = l6;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t44 = __call_res?;
                    s_i32_0 = __t44;
                    l22 = s_i32_0;
                    s_i32_0 = l0;
                    s_i32_1 = l22;
                    vm.enter_call()?;
                    let __call_res = Self::f_ed28f33ecf20(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t45 = __call_res?;
                    s_i32_0 = __t45;
                    l8 = s_i32_0;
                    s_i32_0 = l22;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                    s_i32_1 = 11i32;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                    s_i32_1 = l22;
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 0u32)?;
                    s_i32_2 = 12i32;
                    s_i32_1 = ops::i32_eq(s_i32_1, s_i32_2);
                    s_i32_0 = ops::i32_or(s_i32_0, s_i32_1);
                    'blk_54: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 32i32;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
                            vm.exit_call();
                            let __t46 = __call_res?;
                            s_i32_0 = __t46;
                            l14 = s_i32_0;
                            s_i32_0 = l14;
                            s_i32_1 = l22;
                            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 8u32)?;
                            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                            s_i32_0 = l14;
                            s_i32_1 = l22;
                            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 12u32)?;
                            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                            s_i32_0 = l14;
                            s_i32_1 = l3;
                            s_i32_2 = l22;
                            s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 4u32)?;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_1, s_i32_2, host);
                            vm.exit_call();
                            let __t47 = __call_res?;
                            s_i32_1 = __t47;
                            ops::i32_store(vm, 0, s_i32_0, 16u32, s_i32_1)?;
                            s_i32_0 = l21;
                            s_i32_1 = l14;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 16usize, s_i32_0, s_i32_1, host);
                            vm.exit_call();
                            __call_res?;
                        }
                    }
                    s_i32_0 = l8;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                    'blk_55: {
                        if s_i32_0 != 0 {
                            s_i32_0 = l19;
                            s_i32_1 = l19;
                            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
                            s_i32_2 = 4i32;
                            s_i32_1 = ops::i32_sub(s_i32_1, s_i32_2);
                            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                            s_i32_0 = 1i32;
                            return Ok(s_i32_0);
                        }
                    }
                    s_i32_0 = l6;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    l6 = s_i32_0;
                    continue 'blk_53;
                    break 'blk_53;
                }
            }
            s_i32_0 = l19;
            s_i32_1 = l19;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
            s_i32_2 = 4i32;
            s_i32_1 = ops::i32_sub(s_i32_1, s_i32_2);
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l8;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l2;
    s_i32_1 = 17i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_56: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
            vm.enter_call()?;
            let __call_res = Self::f_ed28f33ecf20(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t48 = __call_res?;
            s_i32_0 = __t48;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_57: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l0;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 8u32)?;
            vm.enter_call()?;
            let __call_res = Self::f_ed28f33ecf20(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t49 = __call_res?;
            s_i32_0 = __t49;
            l24 = s_i32_0;
            s_i32_0 = l0;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 12u32)?;
            vm.enter_call()?;
            let __call_res = Self::f_ed28f33ecf20(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t50 = __call_res?;
            s_i32_0 = __t50;
            l25 = s_i32_0;
            s_i32_0 = l24;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_58: {
                if s_i32_0 != 0 {
                    s_i32_0 = l25;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l25;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_59: {
                if s_i32_0 != 0 {
                    s_i32_0 = l24;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l24;
            s_i32_1 = l25;
            s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
            'blk_60: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = l24;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l2;
    s_i32_1 = 25i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_61: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
            l23 = s_i32_0;
            s_i32_0 = l0;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 8u32)?;
            vm.enter_call()?;
            let __call_res = Self::f_ed28f33ecf20(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t51 = __call_res?;
            s_i32_0 = __t51;
            l24 = s_i32_0;
            s_i32_0 = l24;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_62: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l23;
            s_i32_1 = 1116644i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_63: {
                if s_i32_0 != 0 {
                    s_i32_0 = 2i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l23;
            s_i32_1 = 1116641i32;
            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
            s_i32_1 = l23;
            s_i32_2 = 1116643i32;
            s_i32_1 = ops::i32_le_u(s_i32_1, s_i32_2);
            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
            'blk_64: {
                if s_i32_0 != 0 {
                    s_i32_0 = 5i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l23;
            s_i32_1 = 1116655i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_65: {
                if s_i32_0 != 0 {
                    s_i32_0 = 7i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l23;
            s_i32_1 = 1116697i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_66: {
                if s_i32_0 != 0 {
                    s_i32_0 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
                    s_i32_1 = l24;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t52 = __call_res?;
                    s_i32_0 = __t52;
                    vm.deduct_1(host)?;
                    l11 = s_i32_0;
                    s_i32_0 = l11;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                    s_i32_1 = 9i32;
                    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
                    'blk_67: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = 8i32;
                    s_i32_1 = l11;
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t53 = __call_res?;
                    s_i32_0 = __t53;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l23;
            s_i32_1 = 1116728i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            vm.deduct_1(host)?;
            'blk_68: {
                if s_i32_0 != 0 {
                    s_i32_0 = l24;
                    s_i32_1 = 8i32;
                    s_i32_0 = ops::i32_lt_u(s_i32_0, s_i32_1);
                    'blk_69: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
                    s_i32_1 = l24;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t54 = __call_res?;
                    s_i32_0 = __t54;
                    l11 = s_i32_0;
                    s_i32_0 = l11;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                    s_i32_1 = 8i32;
                    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
                    s_i32_1 = l11;
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
                    s_i32_2 = 3i32;
                    s_i32_1 = ops::i32_ne(s_i32_1, s_i32_2);
                    s_i32_0 = ops::i32_or(s_i32_0, s_i32_1);
                    'blk_70: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = l24;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l24;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l2;
    s_i32_1 = 26i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_71: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
            l23 = s_i32_0;
            s_i32_0 = l0;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 8u32)?;
            vm.enter_call()?;
            let __call_res = Self::f_ed28f33ecf20(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t55 = __call_res?;
            s_i32_0 = __t55;
            l24 = s_i32_0;
            s_i32_0 = l24;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_72: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l0;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 12u32)?;
            vm.enter_call()?;
            let __call_res = Self::f_ed28f33ecf20(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t56 = __call_res?;
            s_i32_0 = __t56;
            l25 = s_i32_0;
            s_i32_0 = l25;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            s_i32_1 = l23;
            s_i32_2 = 1116645i32;
            s_i32_1 = ops::i32_ne(s_i32_1, s_i32_2);
            s_i32_2 = l23;
            s_i32_3 = 1116646i32;
            s_i32_2 = ops::i32_ne(s_i32_2, s_i32_3);
            s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
            'blk_73: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l23;
            s_i32_1 = 1116645i32;
            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
            s_i32_1 = l23;
            s_i32_2 = 1116652i32;
            s_i32_1 = ops::i32_le_u(s_i32_1, s_i32_2);
            vm.deduct_1(host)?;
            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
            'blk_74: {
                if s_i32_0 != 0 {
                    s_i32_0 = 2i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l23;
            s_i32_1 = 1116656i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_75: {
                if s_i32_0 != 0 {
                    s_i32_0 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
                    s_i32_1 = l24;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t57 = __call_res?;
                    s_i32_0 = __t57;
                    l11 = s_i32_0;
                    s_i32_0 = l11;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l23;
            s_i32_1 = 1116693i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_76: {
                if s_i32_0 != 0 {
                    s_i32_0 = 9i32;
                    s_i32_1 = l24;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t58 = __call_res?;
                    s_i32_0 = __t58;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l23;
            s_i32_1 = 1116694i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            s_i32_1 = l23;
            s_i32_2 = 1116695i32;
            s_i32_1 = ops::i32_eq(s_i32_1, s_i32_2);
            s_i32_0 = ops::i32_or(s_i32_0, s_i32_1);
            'blk_77: {
                if s_i32_0 != 0 {
                    s_i32_0 = 0i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l24;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l2;
    s_i32_1 = 27i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_78: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
            l23 = s_i32_0;
            s_i32_0 = l0;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 8u32)?;
            vm.enter_call()?;
            let __call_res = Self::f_ed28f33ecf20(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t59 = __call_res?;
            s_i32_0 = __t59;
            l24 = s_i32_0;
            s_i32_0 = l24;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_79: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l0;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 12u32)?;
            vm.enter_call()?;
            let __call_res = Self::f_ed28f33ecf20(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t60 = __call_res?;
            s_i32_0 = __t60;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_80: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l0;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 16u32)?;
            vm.enter_call()?;
            let __call_res = Self::f_ed28f33ecf20(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t61 = __call_res?;
            s_i32_0 = __t61;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_81: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l23;
            s_i32_1 = 1116696i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            vm.deduct_1(host)?;
            'blk_82: {
                if s_i32_0 != 0 {
                    s_i32_0 = 0i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l24;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l2;
    s_i32_1 = 28i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    s_i32_1 = l2;
    s_i32_2 = 29i32;
    s_i32_1 = ops::i32_eq(s_i32_1, s_i32_2);
    s_i32_0 = ops::i32_or(s_i32_0, s_i32_1);
    'blk_83: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
            vm.enter_call()?;
            let __call_res = Self::f_ed28f33ecf20(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t62 = __call_res?;
            s_i32_0 = __t62;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_84: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l3;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 8u32)?;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t63 = __call_res?;
            s_i32_0 = __t63;
            return Ok(s_i32_0);
        }
    }
    return Err(WasmTrap::Unreachable);
    }

    #[inline(never)]
    pub(crate) fn f_17fe7444ad16(vm: &mut SubstituteVm, dispatch: &'static crate::ModuleDispatch, g_map: &'static [usize], c_map: &'static [usize], p0: i32, p1: i32, p2: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = p2;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i32 = 0i32;
    let mut l7: i32 = 0i32;
    let mut l8: i32 = 0i32;
    let mut l9: i32 = 0i32;
    let mut l10: i32 = 0i32;
    let mut l11: i32 = 0i32;
    let mut l12: i32 = 0i32;
    let mut l13: i32 = 0i32;
    let mut l14: i32 = 0i32;
    let mut l15: i32 = 0i32;
    let mut l16: i32 = 0i32;
    let mut l17: i32 = 0i32;
    let mut l18: i32 = 0i32;
    let mut l19: i32 = 0i32;
    let mut l20: i32 = 0i32;
    let mut l21: i32 = 0i32;
    let mut l22: i32 = 0i32;
    let mut l23: i32 = 0i32;
    let mut l24: i32 = 0i32;
    let mut l25: i32 = 0i32;
    let mut l26: i32 = 0i32;
    let mut l27: i32 = 0i32;
    let mut l28: i32 = 0i32;
    let mut l29: i32 = 0i32;
    let mut l30: i32 = 0i32;
    let mut l31: i32 = 0i32;
    let mut l32: i32 = 0i32;
    let mut l33: i32 = 0i32;
    let mut l34: i32 = 0i32;
    let mut l35: i32 = 0i32;
    let mut l36: i32 = 0i32;
    let mut l37: i32 = 0i32;
    let mut l38: i32 = 0i32;
    let mut l39: i32 = 0i32;
    let mut l40: i32 = 0i32;
    let mut l41: i32 = 0i32;
    let mut l42: i32 = 0i32;
    let mut l43: i32 = 0i32;
    let mut l44: i32 = 0i32;
    let mut l45: i32 = 0i32;
    let mut l46: i32 = 0i32;
    let mut l47: i32 = 0i32;
    let mut l48: i32 = 0i32;
    let mut l49: i32 = 0i32;
    let mut l50: i32 = 0i32;
    let mut l51: i32 = 0i32;
    let mut l52: i32 = 0i32;
    let mut l53: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i32_2: i32 = 0i32;
    let mut s_i32_3: i32 = 0i32;
    let mut s_i32_4: i32 = 0i32;
    let mut s_i32_5: i32 = 0i32;
    let mut s_i32_6: i32 = 0i32;
    vm.deduct_1(host)?;
    s_i32_0 = l1;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    l3 = s_i32_0;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    l4 = s_i32_0;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 24u32)?;
    l5 = s_i32_0;
    s_i32_0 = l3;
    s_i32_1 = 1i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = l5;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = 2i32;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l3;
    s_i32_1 = 2i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
            l14 = s_i32_0;
            s_i32_0 = l14;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_0, host);
            vm.exit_call();
            let __t0 = __call_res?;
            s_i32_0 = __t0;
            s_i32_1 = 2228434i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_3: {
                if s_i32_0 != 0 {
                    s_i32_0 = l5;
                    s_i32_1 = l1;
                    s_i64_1 = ops::i64_load(vm, 0, s_i32_1, 24u32)?;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i64_ret_void(vm, c_map, 2usize, s_i32_0, s_i64_1, host);
                    vm.exit_call();
                    __call_res?;
                } else {
                    s_i32_0 = l5;
                    s_i32_1 = l1;
                    s_i64_1 = ops::i64_load(vm, 0, s_i32_1, 24u32)?;
                    s_i32_1 = ops::i32_wrap_i64(s_i64_1);
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                }
            }
            s_i32_0 = l14;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l3;
    s_i32_1 = 3i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_4: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
            l7 = s_i32_0;
            s_i32_0 = l1;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
            l8 = s_i32_0;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 3usize, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            l6 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = 2228432i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_0 = __t2;
            l9 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = 16i32;
            s_i32_2 = l8;
            s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 16i32;
            s_i32_2 = l6;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 33i32;
            s_i32_2 = l9;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 32i32;
            s_i32_2 = l9;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 32i32;
            s_i32_2 = l9;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 16i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 106i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 6usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 54i32;
            s_i32_2 = 2i32;
            s_i32_3 = 0i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_void(vm, c_map, 7usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 32i32;
            s_i32_2 = l9;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = l8;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 54i32;
            s_i32_2 = 2i32;
            s_i32_3 = 4i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_void(vm, c_map, 7usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 32i32;
            s_i32_2 = l9;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = l8;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 54i32;
            s_i32_2 = 2i32;
            s_i32_3 = 8i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_void(vm, c_map, 7usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 32i32;
            s_i32_2 = l9;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 1i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 54i32;
            s_i32_2 = 2i32;
            s_i32_3 = 12i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_void(vm, c_map, 7usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = 0i32;
            l10 = s_i32_0;
            'blk_5: {
                'blk_6: loop {
                    vm.deduct_1(host)?;
                    s_i32_0 = l10;
                    s_i32_1 = 8i32;
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    s_i32_1 = l8;
                    s_i32_0 = ops::i32_gt_u(s_i32_0, s_i32_1);
                    if s_i32_0 != 0 {
                        break 'blk_5;
                    }
                    s_i32_0 = l5;
                    s_i32_1 = 32i32;
                    s_i32_2 = l9;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l5;
                    s_i32_1 = l7;
                    s_i32_2 = l10;
                    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
                    s_i64_1 = ops::i64_load(vm, 0, s_i32_1, 0u32)?;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i64_ret_void(vm, c_map, 2usize, s_i32_0, s_i64_1, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l5;
                    s_i32_1 = 55i32;
                    s_i32_2 = 0i32;
                    s_i32_3 = 16i32;
                    s_i32_4 = l10;
                    s_i32_3 = ops::i32_add(s_i32_3, s_i32_4);
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_void(vm, c_map, 7usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l10;
                    s_i32_1 = 8i32;
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    l10 = s_i32_0;
                    continue 'blk_6;
                    break 'blk_6;
                }
            }
            'blk_7: {
                'blk_8: loop {
                    vm.deduct_1(host)?;
                    s_i32_0 = l10;
                    s_i32_1 = l8;
                    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                    if s_i32_0 != 0 {
                        break 'blk_7;
                    }
                    s_i32_0 = l5;
                    s_i32_1 = 32i32;
                    s_i32_2 = l9;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l5;
                    s_i32_1 = l7;
                    s_i32_2 = l10;
                    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
                    s_i32_1 = ops::i32_load8_u(vm, 0, s_i32_1, 0u32)?;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l5;
                    s_i32_1 = 58i32;
                    s_i32_2 = 0i32;
                    s_i32_3 = 16i32;
                    s_i32_4 = l10;
                    s_i32_3 = ops::i32_add(s_i32_3, s_i32_4);
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_void(vm, c_map, 7usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l10;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    l10 = s_i32_0;
                    continue 'blk_8;
                    break 'blk_8;
                }
            }
            s_i32_0 = l5;
            s_i32_1 = 32i32;
            s_i32_2 = l9;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = 8i32;
            s_i32_1 = 3i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 8usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i32_0 = __t3;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l3;
    s_i32_1 = 4i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_9: {
        if s_i32_0 != 0 {
            s_i32_0 = l4;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 9usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t4 = __call_res?;
            s_i32_0 = __t4;
            l14 = s_i32_0;
            s_i32_0 = l14;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_void(vm, c_map, 10usize, s_i32_0, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l14;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 11usize, s_i32_0, host);
            vm.exit_call();
            let __t5 = __call_res?;
            s_i32_0 = __t5;
            'blk_10: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = l14;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_0, host);
            vm.exit_call();
            let __t6 = __call_res?;
            s_i32_0 = __t6;
            l16 = s_i32_0;
            s_i32_0 = l16;
            s_i32_0 = ops::i32_eqz(s_i32_0);
            'blk_11: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = l14;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 12usize, s_i32_0, host);
            vm.exit_call();
            let __t7 = __call_res?;
            s_i32_0 = __t7;
            l20 = s_i32_0;
            s_i32_0 = l14;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 13usize, s_i32_0, host);
            vm.exit_call();
            let __t8 = __call_res?;
            s_i32_0 = __t8;
            l21 = s_i32_0;
            s_i32_0 = l1;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
            l11 = s_i32_0;
            s_i32_0 = l11;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 14usize, s_i32_0, host);
            vm.exit_call();
            let __t9 = __call_res?;
            s_i32_0 = __t9;
            l12 = s_i32_0;
            s_i32_0 = 16i32;
            s_i32_1 = l12;
            s_i32_2 = 2i32;
            s_i32_1 = ops::i32_shl(s_i32_1, s_i32_2);
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 15usize, s_i32_0, host);
            vm.exit_call();
            let __t10 = __call_res?;
            s_i32_0 = __t10;
            l17 = s_i32_0;
            s_i32_0 = 0i32;
            l13 = s_i32_0;
            'blk_12: {
                'blk_13: loop {
                    vm.deduct_1(host)?;
                    s_i32_0 = l13;
                    s_i32_1 = l12;
                    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                    if s_i32_0 != 0 {
                        break 'blk_12;
                    }
                    s_i32_0 = l0;
                    s_i32_1 = l11;
                    s_i32_2 = l13;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 16usize, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    let __t11 = __call_res?;
                    s_i32_1 = __t11;
                    s_i32_2 = 0i32;
                    vm.enter_call()?;
                    let __call_res = Self::f_17fe7444ad16(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    let __t12 = __call_res?;
                    s_i32_0 = __t12;
                    l15 = s_i32_0;
                    s_i32_0 = l15;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                    'blk_14: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 1i32;
                            return Ok(s_i32_0);
                        }
                    }
                    s_i32_0 = l15;
                    s_i32_1 = l14;
                    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
                    'blk_15: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = l5;
                    s_i32_1 = l16;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t13 = __call_res?;
                    s_i32_0 = __t13;
                    l19 = s_i32_0;
                    s_i32_0 = l5;
                    s_i32_1 = 33i32;
                    s_i32_2 = l19;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l17;
                    s_i32_1 = l19;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 17usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l13;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    l13 = s_i32_0;
                    continue 'blk_13;
                    break 'blk_13;
                }
            }
            vm.enter_call()?;
            let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 3usize, host);
            vm.exit_call();
            let __t14 = __call_res?;
            s_i32_0 = __t14;
            l6 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = 2228432i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t15 = __call_res?;
            s_i32_0 = __t15;
            l9 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = 16i32;
            s_i32_2 = l12;
            s_i32_3 = l20;
            s_i32_2 = ops::i32_mul(s_i32_2, s_i32_3);
            s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 16i32;
            s_i32_2 = l6;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 33i32;
            s_i32_2 = l9;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 32i32;
            s_i32_2 = l9;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 32i32;
            s_i32_2 = l9;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 16i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 106i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 6usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 54i32;
            s_i32_2 = 2i32;
            s_i32_3 = 0i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_void(vm, c_map, 7usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 32i32;
            s_i32_2 = l9;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = l12;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 54i32;
            s_i32_2 = 2i32;
            s_i32_3 = 4i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_void(vm, c_map, 7usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 32i32;
            s_i32_2 = l9;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = l12;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 54i32;
            s_i32_2 = 2i32;
            s_i32_3 = 8i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_void(vm, c_map, 7usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 32i32;
            s_i32_2 = l9;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 1i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 54i32;
            s_i32_2 = 2i32;
            s_i32_3 = 12i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_void(vm, c_map, 7usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = 0i32;
            l13 = s_i32_0;
            vm.deduct_1(host)?;
            'blk_16: {
                'blk_17: loop {
                    vm.deduct_1(host)?;
                    s_i32_0 = l13;
                    s_i32_1 = l12;
                    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                    if s_i32_0 != 0 {
                        break 'blk_16;
                    }
                    s_i32_0 = l5;
                    s_i32_1 = 32i32;
                    s_i32_2 = l9;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l5;
                    s_i32_1 = 32i32;
                    s_i32_2 = l17;
                    s_i32_3 = l13;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 16usize, s_i32_2, s_i32_3, host);
                    vm.exit_call();
                    let __t16 = __call_res?;
                    s_i32_2 = __t16;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l5;
                    s_i32_1 = l20;
                    s_i32_2 = l21;
                    s_i32_3 = 16i32;
                    s_i32_4 = l13;
                    s_i32_5 = l20;
                    s_i32_4 = ops::i32_mul(s_i32_4, s_i32_5);
                    s_i32_3 = ops::i32_add(s_i32_3, s_i32_4);
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_void(vm, c_map, 18usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l13;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    l13 = s_i32_0;
                    continue 'blk_17;
                    break 'blk_17;
                }
            }
            s_i32_0 = l5;
            s_i32_1 = 32i32;
            s_i32_2 = l9;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = 8i32;
            s_i32_1 = l14;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 8usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t17 = __call_res?;
            s_i32_0 = __t17;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l3;
    s_i32_1 = 5i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_18: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
            l11 = s_i32_0;
            s_i32_0 = l11;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 14usize, s_i32_0, host);
            vm.exit_call();
            let __t18 = __call_res?;
            s_i32_0 = __t18;
            l12 = s_i32_0;
            s_i32_0 = l12;
            s_i32_0 = ops::i32_eqz(s_i32_0);
            'blk_19: {
                if s_i32_0 != 0 {
                    s_i32_0 = l5;
                    s_i32_1 = 0i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = 8i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l12;
            s_i32_1 = 2i32;
            s_i32_0 = ops::i32_shl(s_i32_0, s_i32_1);
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 15usize, s_i32_0, host);
            vm.exit_call();
            let __t19 = __call_res?;
            s_i32_0 = __t19;
            l17 = s_i32_0;
            s_i32_0 = l12;
            s_i32_1 = 2i32;
            s_i32_0 = ops::i32_shl(s_i32_0, s_i32_1);
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 19usize, s_i32_0, host);
            vm.exit_call();
            let __t20 = __call_res?;
            s_i32_0 = __t20;
            l18 = s_i32_0;
            s_i32_0 = 0i32;
            l13 = s_i32_0;
            'blk_20: {
                'blk_21: loop {
                    vm.deduct_1(host)?;
                    s_i32_0 = l13;
                    s_i32_1 = l12;
                    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                    if s_i32_0 != 0 {
                        break 'blk_20;
                    }
                    s_i32_0 = l0;
                    s_i32_1 = l11;
                    s_i32_2 = l13;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 16usize, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    let __t21 = __call_res?;
                    s_i32_1 = __t21;
                    s_i32_2 = 0i32;
                    vm.enter_call()?;
                    let __call_res = Self::f_17fe7444ad16(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    let __t22 = __call_res?;
                    s_i32_0 = __t22;
                    l15 = s_i32_0;
                    s_i32_0 = l15;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                    'blk_22: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 1i32;
                            return Ok(s_i32_0);
                        }
                    }
                    s_i32_0 = l15;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_0, host);
                    vm.exit_call();
                    let __t23 = __call_res?;
                    s_i32_0 = __t23;
                    l16 = s_i32_0;
                    s_i32_0 = l16;
                    s_i32_0 = ops::i32_eqz(s_i32_0);
                    'blk_23: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = l5;
                    s_i32_1 = l16;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t24 = __call_res?;
                    s_i32_0 = __t24;
                    l19 = s_i32_0;
                    s_i32_0 = l5;
                    s_i32_1 = 33i32;
                    s_i32_2 = l19;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l17;
                    s_i32_1 = l19;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 17usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l18;
                    s_i32_1 = l13;
                    s_i32_2 = 2i32;
                    s_i32_1 = ops::i32_shl(s_i32_1, s_i32_2);
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    s_i32_1 = l15;
                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l13;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    l13 = s_i32_0;
                    continue 'blk_21;
                    break 'blk_21;
                }
            }
            s_i32_0 = l12;
            s_i32_1 = l18;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 20usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t25 = __call_res?;
            s_i32_0 = __t25;
            l14 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = l14;
            s_i32_2 = l17;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 21usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l14;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l3;
    s_i32_1 = 6i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_24: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
            'blk_25: {
                if s_i32_0 != 0 {
                    s_i32_0 = l4;
                    s_i32_1 = l1;
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 8u32)?;
                    s_i32_2 = l1;
                    s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 12u32)?;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 22usize, s_i32_0, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    let __t26 = __call_res?;
                    s_i32_0 = __t26;
                } else {
                    s_i32_0 = l4;
                    s_i32_1 = l1;
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 8u32)?;
                    s_i32_2 = l1;
                    s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 12u32)?;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 23usize, s_i32_0, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    let __t27 = __call_res?;
                    s_i32_0 = __t27;
                }
            }
            l22 = s_i32_0;
            s_i32_0 = l22;
            s_i32_1 = 0i32;
            s_i32_0 = ops::i32_lt_s(s_i32_0, s_i32_1);
            'blk_26: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
            s_i32_1 = l22;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 16usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t28 = __call_res?;
            s_i32_0 = __t28;
            l23 = s_i32_0;
            s_i32_0 = l1;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 16u32)?;
            l11 = s_i32_0;
            s_i32_0 = l11;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 14usize, s_i32_0, host);
            vm.exit_call();
            let __t29 = __call_res?;
            s_i32_0 = __t29;
            l12 = s_i32_0;
            s_i32_0 = l12;
            s_i32_1 = l23;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 32u32)?;
            s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
            'blk_27: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = l23;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 28u32)?;
            l18 = s_i32_0;
            s_i32_0 = l12;
            s_i32_1 = 2i32;
            s_i32_0 = ops::i32_shl(s_i32_0, s_i32_1);
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 15usize, s_i32_0, host);
            vm.exit_call();
            let __t30 = __call_res?;
            s_i32_0 = __t30;
            l17 = s_i32_0;
            s_i32_0 = 0i32;
            l13 = s_i32_0;
            'blk_28: {
                'blk_29: loop {
                    vm.deduct_1(host)?;
                    s_i32_0 = l13;
                    s_i32_1 = l12;
                    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                    if s_i32_0 != 0 {
                        break 'blk_28;
                    }
                    s_i32_0 = l0;
                    s_i32_1 = l11;
                    s_i32_2 = l13;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 16usize, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    let __t31 = __call_res?;
                    s_i32_1 = __t31;
                    s_i32_2 = 0i32;
                    vm.enter_call()?;
                    let __call_res = Self::f_17fe7444ad16(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    let __t32 = __call_res?;
                    s_i32_0 = __t32;
                    l15 = s_i32_0;
                    s_i32_0 = l15;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                    'blk_30: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 1i32;
                            return Ok(s_i32_0);
                        }
                    }
                    s_i32_0 = l15;
                    s_i32_1 = l18;
                    s_i32_2 = l13;
                    s_i32_3 = 2i32;
                    s_i32_2 = ops::i32_shl(s_i32_2, s_i32_3);
                    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 0u32)?;
                    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
                    'blk_31: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = l5;
                    s_i32_1 = l15;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_1, host);
                    vm.exit_call();
                    let __t33 = __call_res?;
                    s_i32_1 = __t33;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t34 = __call_res?;
                    s_i32_0 = __t34;
                    l19 = s_i32_0;
                    s_i32_0 = l5;
                    s_i32_1 = 33i32;
                    s_i32_2 = l19;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l17;
                    s_i32_1 = l19;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 17usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l13;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    l13 = s_i32_0;
                    continue 'blk_29;
                    break 'blk_29;
                }
            }
            s_i32_0 = l22;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 24usize, s_i32_0, host);
            vm.exit_call();
            let __t35 = __call_res?;
            s_i32_0 = __t35;
            l14 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = l14;
            s_i32_2 = l17;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 21usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l14;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l3;
    s_i32_1 = 7i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_32: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
            s_i32_2 = 0i32;
            vm.enter_call()?;
            let __call_res = Self::f_17fe7444ad16(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            let __t36 = __call_res?;
            s_i32_0 = __t36;
            l14 = s_i32_0;
            s_i32_0 = l14;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_33: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l14;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_void(vm, c_map, 25usize, s_i32_0, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = vm.get_i32(*g_map.get(1usize).ok_or(WasmTrap::Unreachable)?)?;
            s_i32_1 = l14;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 16usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t37 = __call_res?;
            s_i32_0 = __t37;
            l24 = s_i32_0;
            vm.deduct_1(host)?;
            s_i32_0 = l24;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
            s_i32_1 = 10i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_34: {
                if s_i32_0 != 0 {
                    s_i32_0 = l24;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
                } else {
                    s_i32_0 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
                    s_i32_1 = l24;
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 16usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t38 = __call_res?;
                    s_i32_0 = __t38;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 32u32)?;
                }
            }
            l12 = s_i32_0;
            s_i32_0 = l1;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
            l13 = s_i32_0;
            s_i32_0 = l13;
            s_i32_1 = l12;
            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
            'blk_35: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = l24;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 20u32)?;
            s_i32_1 = l13;
            s_i32_2 = 4i32;
            s_i32_1 = ops::i32_shl(s_i32_1, s_i32_2);
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l26 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = l26;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
            s_i32_2 = l26;
            s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 8u32)?;
            s_i32_3 = l26;
            s_i32_3 = ops::i32_load(vm, 0, s_i32_3, 0u32)?;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_void(vm, c_map, 26usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l26;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 12u32)?;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l3;
    s_i32_1 = 8i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_36: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
            s_i32_2 = 0i32;
            vm.enter_call()?;
            let __call_res = Self::f_17fe7444ad16(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            let __t39 = __call_res?;
            s_i32_0 = __t39;
            l14 = s_i32_0;
            s_i32_0 = l14;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_37: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l14;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_void(vm, c_map, 25usize, s_i32_0, host);
            vm.exit_call();
            __call_res?;
            vm.deduct_1(host)?;
            s_i32_0 = vm.get_i32(*g_map.get(1usize).ok_or(WasmTrap::Unreachable)?)?;
            s_i32_1 = l14;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 16usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t40 = __call_res?;
            s_i32_0 = __t40;
            l24 = s_i32_0;
            s_i32_0 = l24;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
            s_i32_1 = 10i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_38: {
                if s_i32_0 != 0 {
                    s_i32_0 = l24;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
                } else {
                    s_i32_0 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
                    s_i32_1 = l24;
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 16usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t41 = __call_res?;
                    s_i32_0 = __t41;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 32u32)?;
                }
            }
            l12 = s_i32_0;
            s_i32_0 = l1;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
            l13 = s_i32_0;
            s_i32_0 = l13;
            s_i32_1 = l12;
            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
            'blk_39: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = l24;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 20u32)?;
            s_i32_1 = l13;
            s_i32_2 = 4i32;
            s_i32_1 = ops::i32_shl(s_i32_1, s_i32_2);
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l26 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = 2228432i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t42 = __call_res?;
            s_i32_0 = __t42;
            l27 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = 33i32;
            s_i32_2 = l27;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l0;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 12u32)?;
            s_i32_2 = 0i32;
            vm.enter_call()?;
            let __call_res = Self::f_17fe7444ad16(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            let __t43 = __call_res?;
            s_i32_0 = __t43;
            l15 = s_i32_0;
            s_i32_0 = l15;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_40: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l15;
            s_i32_1 = l26;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 12u32)?;
            s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
            'blk_41: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = l5;
            s_i32_1 = l15;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_1, host);
            vm.exit_call();
            let __t44 = __call_res?;
            s_i32_1 = __t44;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t45 = __call_res?;
            s_i32_0 = __t45;
            l28 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = 33i32;
            s_i32_2 = l28;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l24;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 16u32)?;
            l8 = s_i32_0;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 3usize, host);
            vm.exit_call();
            let __t46 = __call_res?;
            s_i32_0 = __t46;
            l6 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = 2228432i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t47 = __call_res?;
            s_i32_0 = __t47;
            l29 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = l8;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 16i32;
            s_i32_2 = l6;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 33i32;
            s_i32_2 = l29;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 32i32;
            s_i32_2 = l29;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 32i32;
            s_i32_2 = l27;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = l8;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_void(vm, c_map, 27usize, s_i32_0, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 32i32;
            s_i32_2 = l29;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 32i32;
            s_i32_2 = l28;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = l26;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
            s_i32_2 = l26;
            s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 8u32)?;
            s_i32_3 = l26;
            s_i32_3 = ops::i32_load(vm, 0, s_i32_3, 0u32)?;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_void(vm, c_map, 18usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 32i32;
            s_i32_2 = l29;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l14;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l3;
    s_i32_1 = 9i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_42: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 40u32)?;
            'blk_43: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = l0;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 8u32)?;
            s_i32_2 = 0i32;
            vm.enter_call()?;
            let __call_res = Self::f_17fe7444ad16(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            let __t48 = __call_res?;
            s_i32_0 = __t48;
            l15 = s_i32_0;
            vm.deduct_1(host)?;
            s_i32_0 = l15;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_44: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l15;
            s_i32_1 = 7i32;
            s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
            'blk_45: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = l4;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 9usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t49 = __call_res?;
            s_i32_0 = __t49;
            l14 = s_i32_0;
            s_i32_0 = l14;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_void(vm, c_map, 10usize, s_i32_0, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l14;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_0, host);
            vm.exit_call();
            let __t50 = __call_res?;
            s_i32_0 = __t50;
            s_i32_0 = ops::i32_eqz(s_i32_0);
            'blk_46: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = l5;
            s_i32_1 = 16i32;
            s_i32_2 = 1116692i32;
            s_i32_3 = 1116689i32;
            s_i32_4 = l14;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 28usize, s_i32_2, s_i32_3, s_i32_4, host);
            vm.exit_call();
            let __t51 = __call_res?;
            s_i32_2 = __t51;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = 9i32;
            s_i32_1 = l14;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 8usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t52 = __call_res?;
            s_i32_0 = __t52;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l3;
    s_i32_1 = 10i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_47: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            l53 = s_i32_0;
            s_i32_0 = l1;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
            'blk_48: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                    vm.deduct_1(host)?;
                    s_i32_1 = l4;
                    s_i32_2 = l1;
                    s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 4u32)?;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 9usize, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    let __t53 = __call_res?;
                    s_i32_1 = __t53;
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    l53 = s_i32_0;
                }
            }
            s_i32_0 = l1;
            vm.deduct_1(host)?;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
            s_i32_0 = ops::i32_eqz(s_i32_0);
            'blk_49: {
                if s_i32_0 != 0 {
                    s_i32_0 = l0;
                    s_i32_1 = l1;
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 12u32)?;
                    s_i32_2 = l1;
                    s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 16u32)?;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 29usize, s_i32_0, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    let __t54 = __call_res?;
                    s_i32_0 = __t54;
                    l30 = s_i32_0;
                    s_i32_0 = l30;
                    'blk_50: {
                        if s_i32_0 != 0 {
                            s_i32_0 = l53;
                            s_i32_1 = l53;
                            s_i32_2 = 1i32;
                            s_i32_1 = ops::i32_sub(s_i32_1, s_i32_2);
                            s_i32_2 = l30;
                            s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 16u32)?;
                            s_i32_1 = ops::i32_ne(s_i32_1, s_i32_2);
                            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                            'blk_51: {
                                if s_i32_0 != 0 {
                                    return Err(WasmTrap::Unreachable);
                                }
                            }
                            s_i32_0 = l0;
                            s_i32_1 = l30;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 30usize, s_i32_0, s_i32_1, host);
                            vm.exit_call();
                            let __t55 = __call_res?;
                            s_i32_0 = __t55;
                            return Ok(s_i32_0);
                        }
                    }
                }
            }
            s_i32_0 = l4;
            s_i32_1 = l53;
            s_i32_2 = l1;
            s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 8u32)?;
            s_i32_3 = l1;
            s_i32_3 = ops::i32_load(vm, 0, s_i32_3, 12u32)?;
            s_i32_4 = l1;
            s_i32_4 = ops::i32_load(vm, 0, s_i32_4, 16u32)?;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_i32_i32_ret_i32(vm, c_map, 31usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, s_i32_4, host);
            vm.exit_call();
            let __t56 = __call_res?;
            s_i32_0 = __t56;
            l31 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = 35i32;
            s_i32_2 = l31;
            s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 32u32)?;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l31;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 28u32)?;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l3;
    s_i32_1 = 11i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_52: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 40u32)?;
            vm.deduct_1(host)?;
            'blk_53: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = l4;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 9usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t57 = __call_res?;
            s_i32_0 = __t57;
            l14 = s_i32_0;
            s_i32_0 = l14;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_void(vm, c_map, 10usize, s_i32_0, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l14;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_0, host);
            vm.exit_call();
            let __t58 = __call_res?;
            s_i32_0 = __t58;
            l16 = s_i32_0;
            s_i32_0 = l16;
            s_i32_0 = ops::i32_eqz(s_i32_0);
            'blk_54: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = l0;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 28u32)?;
            l32 = s_i32_0;
            s_i32_0 = l32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 14usize, s_i32_0, host);
            vm.exit_call();
            let __t59 = __call_res?;
            s_i32_0 = __t59;
            l12 = s_i32_0;
            s_i32_0 = l12;
            s_i32_1 = 0i32;
            s_i32_0 = ops::i32_gt_u(s_i32_0, s_i32_1);
            'blk_55: {
                if s_i32_0 != 0 {
                    s_i32_0 = l32;
                    s_i32_1 = l12;
                    s_i32_2 = 1i32;
                    s_i32_1 = ops::i32_sub(s_i32_1, s_i32_2);
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 16usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t60 = __call_res?;
                    s_i32_0 = __t60;
                    l33 = s_i32_0;
                    s_i32_0 = l33;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                    l34 = s_i32_0;
                    s_i32_0 = 0i32;
                    l13 = s_i32_0;
                    'blk_56: {
                        'blk_57: loop {
                            vm.deduct_1(host)?;
                            s_i32_0 = l13;
                            s_i32_1 = l34;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 14usize, s_i32_1, host);
                            vm.exit_call();
                            let __t61 = __call_res?;
                            s_i32_1 = __t61;
                            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                            if s_i32_0 != 0 {
                                break 'blk_56;
                            }
                            s_i32_0 = l34;
                            s_i32_1 = l13;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 16usize, s_i32_0, s_i32_1, host);
                            vm.exit_call();
                            let __t62 = __call_res?;
                            s_i32_0 = __t62;
                            l30 = s_i32_0;
                            s_i32_0 = l30;
                            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
                            s_i32_1 = l30;
                            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 0u32)?;
                            s_i32_2 = l30;
                            s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 4u32)?;
                            s_i32_3 = l1;
                            s_i32_3 = ops::i32_load(vm, 0, s_i32_3, 8u32)?;
                            s_i32_4 = l1;
                            s_i32_4 = ops::i32_load(vm, 0, s_i32_4, 12u32)?;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_i32(vm, c_map, 32usize, s_i32_1, s_i32_2, s_i32_3, s_i32_4, host);
                            vm.exit_call();
                            let __t63 = __call_res?;
                            s_i32_1 = __t63;
                            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                            'blk_58: {
                                if s_i32_0 != 0 {
                                    s_i32_0 = l1;
                                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 16u32)?;
                                    'blk_59: {
                                        if s_i32_0 != 0 {
                                            s_i32_0 = l0;
                                            s_i32_1 = l1;
                                            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 16u32)?;
                                            s_i32_2 = 0i32;
                                            vm.enter_call()?;
                                            let __call_res = Self::f_17fe7444ad16(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, s_i32_2, host);
                                            vm.exit_call();
                                            let __t64 = __call_res?;
                                            s_i32_0 = __t64;
                                            l15 = s_i32_0;
                                            s_i32_0 = l15;
                                            s_i32_1 = 1i32;
                                            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                                            'blk_60: {
                                                if s_i32_0 != 0 {
                                                    s_i32_0 = 1i32;
                                                    return Ok(s_i32_0);
                                                }
                                            }
                                            s_i32_0 = l15;
                                            s_i32_1 = l14;
                                            s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
                                            'blk_61: {
                                                if s_i32_0 != 0 {
                                                    return Err(WasmTrap::Unreachable);
                                                }
                                            }
                                            s_i32_0 = l5;
                                            s_i32_1 = l16;
                                            vm.enter_call()?;
                                            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, host);
                                            vm.exit_call();
                                            let __t65 = __call_res?;
                                            s_i32_0 = __t65;
                                            l19 = s_i32_0;
                                            s_i32_0 = l5;
                                            s_i32_1 = 33i32;
                                            s_i32_2 = l19;
                                            vm.enter_call()?;
                                            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                                            vm.exit_call();
                                            __call_res?;
                                            s_i32_0 = l5;
                                            s_i32_1 = l30;
                                            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 12u32)?;
                                            s_i32_2 = l14;
                                            s_i32_3 = l19;
                                            vm.enter_call()?;
                                            let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_void(vm, c_map, 33usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
                                            vm.exit_call();
                                            __call_res?;
                                        }
                                    }
                                    s_i32_0 = 0i32;
                                    return Ok(s_i32_0);
                                }
                            }
                            s_i32_0 = l13;
                            s_i32_1 = 1i32;
                            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                            l13 = s_i32_0;
                            continue 'blk_57;
                            break 'blk_57;
                        }
                    }
                }
            }
            s_i32_0 = l5;
            s_i32_1 = l16;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t66 = __call_res?;
            s_i32_0 = __t66;
            l19 = s_i32_0;
            s_i32_0 = -1i32;
            l35 = s_i32_0;
            s_i32_0 = l1;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 16u32)?;
            'blk_62: {
                if s_i32_0 != 0 {
                    s_i32_0 = l0;
                    s_i32_1 = l1;
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 16u32)?;
                    s_i32_2 = 0i32;
                    vm.enter_call()?;
                    let __call_res = Self::f_17fe7444ad16(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    let __t67 = __call_res?;
                    s_i32_0 = __t67;
                    l15 = s_i32_0;
                    s_i32_0 = l15;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                    'blk_63: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 1i32;
                            return Ok(s_i32_0);
                        }
                    }
                    s_i32_0 = l15;
                    s_i32_1 = l14;
                    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
                    'blk_64: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = l5;
                    s_i32_1 = 33i32;
                    s_i32_2 = l19;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    __call_res?;
                } else {
                    s_i32_0 = l5;
                    s_i32_1 = 2228432i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t68 = __call_res?;
                    s_i32_0 = __t68;
                    l35 = s_i32_0;
                }
            }
            s_i32_0 = l12;
            s_i32_1 = 0i32;
            s_i32_0 = ops::i32_gt_u(s_i32_0, s_i32_1);
            'blk_65: {
                if s_i32_0 != 0 {
                    s_i32_0 = l32;
                    s_i32_1 = l12;
                    s_i32_2 = 1i32;
                    s_i32_1 = ops::i32_sub(s_i32_1, s_i32_2);
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 16usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t69 = __call_res?;
                    s_i32_0 = __t69;
                    l33 = s_i32_0;
                    s_i32_0 = 32i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 19usize, s_i32_0, host);
                    vm.exit_call();
                    let __t70 = __call_res?;
                    s_i32_0 = __t70;
                    l30 = s_i32_0;
                    s_i32_0 = l30;
                    s_i32_1 = l1;
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 8u32)?;
                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l30;
                    s_i32_1 = l1;
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 12u32)?;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l30;
                    s_i32_1 = 0i32;
                    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                    s_i32_0 = l30;
                    s_i32_1 = l19;
                    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                    s_i32_0 = l30;
                    s_i32_1 = l14;
                    ops::i32_store(vm, 0, s_i32_0, 16u32, s_i32_1)?;
                    s_i32_0 = l30;
                    s_i32_1 = 1i32;
                    ops::i32_store(vm, 0, s_i32_0, 20u32, s_i32_1)?;
                    s_i32_0 = l30;
                    s_i32_1 = l35;
                    ops::i32_store(vm, 0, s_i32_0, 24u32, s_i32_1)?;
                    s_i32_0 = l33;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                    s_i32_1 = l30;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 17usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                }
            }
            s_i32_0 = 0i32;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l3;
    s_i32_1 = 12i32;
    vm.deduct_1(host)?;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_66: {
        if s_i32_0 != 0 {
            s_i32_0 = l4;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 9usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t71 = __call_res?;
            s_i32_0 = __t71;
            l14 = s_i32_0;
            s_i32_0 = l14;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_void(vm, c_map, 10usize, s_i32_0, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l14;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_0, host);
            vm.exit_call();
            let __t72 = __call_res?;
            s_i32_0 = __t72;
            l16 = s_i32_0;
            s_i32_0 = l16;
            s_i32_0 = ops::i32_eqz(s_i32_0);
            'blk_67: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = l0;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 16u32)?;
            s_i32_2 = 0i32;
            vm.enter_call()?;
            let __call_res = Self::f_17fe7444ad16(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            let __t73 = __call_res?;
            s_i32_0 = __t73;
            l15 = s_i32_0;
            s_i32_0 = l15;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_68: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l15;
            s_i32_1 = l14;
            s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
            'blk_69: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = l5;
            s_i32_1 = l16;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t74 = __call_res?;
            s_i32_0 = __t74;
            l19 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = 33i32;
            s_i32_2 = l19;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l0;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 28u32)?;
            l32 = s_i32_0;
            s_i32_0 = l32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 14usize, s_i32_0, host);
            vm.exit_call();
            let __t75 = __call_res?;
            s_i32_0 = __t75;
            l12 = s_i32_0;
            s_i32_0 = l12;
            s_i32_1 = 0i32;
            s_i32_0 = ops::i32_gt_u(s_i32_0, s_i32_1);
            'blk_70: {
                if s_i32_0 != 0 {
                    s_i32_0 = l32;
                    s_i32_1 = l12;
                    s_i32_2 = 1i32;
                    s_i32_1 = ops::i32_sub(s_i32_1, s_i32_2);
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 16usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t76 = __call_res?;
                    s_i32_0 = __t76;
                    l33 = s_i32_0;
                    s_i32_0 = 32i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 19usize, s_i32_0, host);
                    vm.exit_call();
                    let __t77 = __call_res?;
                    s_i32_0 = __t77;
                    l30 = s_i32_0;
                    s_i32_0 = l30;
                    s_i32_1 = l1;
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 8u32)?;
                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l30;
                    s_i32_1 = l1;
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 12u32)?;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l30;
                    s_i32_1 = 0i32;
                    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                    s_i32_0 = l30;
                    s_i32_1 = l19;
                    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                    s_i32_0 = l30;
                    s_i32_1 = l14;
                    ops::i32_store(vm, 0, s_i32_0, 16u32, s_i32_1)?;
                    s_i32_0 = l30;
                    s_i32_1 = 0i32;
                    ops::i32_store(vm, 0, s_i32_0, 20u32, s_i32_1)?;
                    s_i32_0 = l30;
                    s_i32_1 = -1i32;
                    ops::i32_store(vm, 0, s_i32_0, 24u32, s_i32_1)?;
                    s_i32_0 = l33;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                    s_i32_1 = l30;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 17usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                }
            }
            s_i32_0 = 0i32;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l3;
    s_i32_1 = 13i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_71: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 40u32)?;
            'blk_72: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = l0;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 8u32)?;
            s_i32_2 = l1;
            s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 12u32)?;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 29usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            let __t78 = __call_res?;
            s_i32_0 = __t78;
            l30 = s_i32_0;
            s_i32_0 = l30;
            s_i32_0 = ops::i32_eqz(s_i32_0);
            'blk_73: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = l30;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 20u32)?;
            s_i32_0 = ops::i32_eqz(s_i32_0);
            'blk_74: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = l30;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 16u32)?;
            l14 = s_i32_0;
            s_i32_0 = l1;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
            'blk_75: {
                if s_i32_0 != 0 {
                    s_i32_0 = l4;
                    s_i32_1 = l1;
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 9usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t79 = __call_res?;
                    s_i32_0 = __t79;
                    s_i32_1 = l14;
                    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
                    'blk_76: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                }
            }
            s_i32_0 = l0;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 16u32)?;
            s_i32_2 = 0i32;
            vm.enter_call()?;
            let __call_res = Self::f_17fe7444ad16(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            let __t80 = __call_res?;
            s_i32_0 = __t80;
            l15 = s_i32_0;
            s_i32_0 = l15;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_77: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l15;
            s_i32_1 = l14;
            s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
            'blk_78: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = l30;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
            'blk_79: {
                if s_i32_0 != 0 {
                    s_i32_0 = l5;
                    s_i32_1 = l14;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_1, host);
                    vm.exit_call();
                    let __t81 = __call_res?;
                    s_i32_1 = __t81;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t82 = __call_res?;
                    s_i32_0 = __t82;
                    l19 = s_i32_0;
                    s_i32_0 = l5;
                    s_i32_1 = 33i32;
                    s_i32_2 = l19;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l5;
                    s_i32_1 = l30;
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 12u32)?;
                    s_i32_2 = l14;
                    s_i32_3 = l19;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_void(vm, c_map, 33usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
                    vm.exit_call();
                    __call_res?;
                } else {
                    s_i32_0 = l5;
                    s_i32_1 = 33i32;
                    s_i32_2 = l30;
                    s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 12u32)?;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l30;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 24u32)?;
                    l35 = s_i32_0;
                    s_i32_0 = l35;
                    s_i32_1 = 0i32;
                    s_i32_0 = ops::i32_ge_s(s_i32_0, s_i32_1);
                    'blk_80: {
                        if s_i32_0 != 0 {
                            s_i32_0 = l5;
                            s_i32_1 = 1i32;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
                            vm.exit_call();
                            __call_res?;
                            s_i32_0 = l5;
                            s_i32_1 = 33i32;
                            s_i32_2 = l35;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                            vm.exit_call();
                            __call_res?;
                        }
                    }
                }
            }
            s_i32_0 = 0i32;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l3;
    s_i32_1 = 14i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    s_i32_1 = l3;
    s_i32_2 = 24i32;
    s_i32_1 = ops::i32_eq(s_i32_1, s_i32_2);
    s_i32_0 = ops::i32_or(s_i32_0, s_i32_1);
    'blk_81: {
        if s_i32_0 != 0 {
            s_i32_0 = l3;
            s_i32_1 = 14i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            s_i32_1 = l0;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 40u32)?;
            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
            'blk_82: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = 0i32;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l3;
    s_i32_1 = 16i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_83: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
            s_i32_2 = l2;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 34usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            let __t83 = __call_res?;
            s_i32_0 = __t83;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l3;
    s_i32_1 = 15i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_84: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 40u32)?;
            'blk_85: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = l1;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 16u32)?;
            l11 = s_i32_0;
            s_i32_0 = l11;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 14usize, s_i32_0, host);
            vm.exit_call();
            let __t84 = __call_res?;
            s_i32_0 = __t84;
            l12 = s_i32_0;
            s_i32_0 = 16i32;
            s_i32_1 = l12;
            s_i32_2 = 2i32;
            s_i32_1 = ops::i32_shl(s_i32_1, s_i32_2);
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 15usize, s_i32_0, host);
            vm.exit_call();
            let __t85 = __call_res?;
            s_i32_0 = __t85;
            l17 = s_i32_0;
            s_i32_0 = 4i32;
            s_i32_1 = l12;
            s_i32_2 = 2i32;
            s_i32_1 = ops::i32_shl(s_i32_1, s_i32_2);
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 19usize, s_i32_0, host);
            vm.exit_call();
            let __t86 = __call_res?;
            s_i32_0 = __t86;
            l18 = s_i32_0;
            s_i32_0 = 0i32;
            l13 = s_i32_0;
            'blk_86: {
                'blk_87: loop {
                    vm.deduct_1(host)?;
                    s_i32_0 = l13;
                    s_i32_1 = l12;
                    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                    if s_i32_0 != 0 {
                        break 'blk_86;
                    }
                    s_i32_0 = l18;
                    s_i32_1 = l13;
                    s_i32_2 = 2i32;
                    s_i32_1 = ops::i32_shl(s_i32_1, s_i32_2);
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    s_i32_1 = l0;
                    s_i32_2 = l11;
                    s_i32_3 = l13;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 16usize, s_i32_2, s_i32_3, host);
                    vm.exit_call();
                    let __t87 = __call_res?;
                    s_i32_2 = __t87;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 35usize, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    let __t88 = __call_res?;
                    s_i32_1 = __t88;
                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l13;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    l13 = s_i32_0;
                    continue 'blk_87;
                    break 'blk_87;
                }
            }
            s_i32_0 = 0i32;
            l36 = s_i32_0;
            s_i32_0 = l1;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
            s_i32_0 = ops::i32_eqz(s_i32_0);
            'blk_88: {
                if s_i32_0 != 0 {
                    s_i32_0 = l0;
                    s_i32_1 = l1;
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 8u32)?;
                    s_i32_2 = l1;
                    s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 12u32)?;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 36usize, s_i32_0, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    let __t89 = __call_res?;
                    s_i32_0 = __t89;
                    l36 = s_i32_0;
                }
            }
            s_i32_0 = l36;
            'blk_89: {
                if s_i32_0 != 0 {
                    s_i32_0 = l36;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 16u32)?;
                    l37 = s_i32_0;
                    s_i32_0 = l37;
                    s_i32_1 = l12;
                    s_i32_2 = l18;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 37usize, s_i32_0, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    let __t90 = __call_res?;
                    s_i32_0 = __t90;
                    s_i32_0 = ops::i32_eqz(s_i32_0);
                    'blk_90: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                } else {
                    s_i32_0 = l4;
                    s_i32_1 = l1;
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
                    s_i32_2 = l1;
                    s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 8u32)?;
                    s_i32_3 = l1;
                    s_i32_3 = ops::i32_load(vm, 0, s_i32_3, 12u32)?;
                    s_i32_4 = l12;
                    s_i32_5 = l18;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_i32_i32_i32_ret_i32(vm, c_map, 38usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, s_i32_4, s_i32_5, host);
                    vm.exit_call();
                    let __t91 = __call_res?;
                    s_i32_0 = __t91;
                    l37 = s_i32_0;
                }
            }
            s_i32_0 = l37;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 32u32)?;
            l14 = s_i32_0;
            vm.deduct_1(host)?;
            s_i32_0 = l2;
            s_i32_1 = l0;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 12u32)?;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_1, host);
            vm.exit_call();
            let __t92 = __call_res?;
            s_i32_1 = __t92;
            s_i32_2 = l14;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_2, host);
            vm.exit_call();
            let __t93 = __call_res?;
            s_i32_2 = __t93;
            s_i32_1 = ops::i32_eq(s_i32_1, s_i32_2);
            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
            l38 = s_i32_0;
            s_i32_0 = 0i32;
            l13 = s_i32_0;
            'blk_91: {
                'blk_92: loop {
                    vm.deduct_1(host)?;
                    s_i32_0 = l13;
                    s_i32_1 = l12;
                    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                    if s_i32_0 != 0 {
                        break 'blk_91;
                    }
                    s_i32_0 = l0;
                    s_i32_1 = l11;
                    s_i32_2 = l13;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 16usize, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    let __t94 = __call_res?;
                    s_i32_1 = __t94;
                    s_i32_2 = 0i32;
                    vm.enter_call()?;
                    let __call_res = Self::f_17fe7444ad16(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    let __t95 = __call_res?;
                    s_i32_0 = __t95;
                    l15 = s_i32_0;
                    s_i32_0 = l15;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                    'blk_93: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 1i32;
                            return Ok(s_i32_0);
                        }
                    }
                    s_i32_0 = l5;
                    s_i32_1 = l15;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_1, host);
                    vm.exit_call();
                    let __t96 = __call_res?;
                    s_i32_1 = __t96;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t97 = __call_res?;
                    s_i32_0 = __t97;
                    l19 = s_i32_0;
                    s_i32_0 = l5;
                    s_i32_1 = 33i32;
                    s_i32_2 = l19;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l17;
                    s_i32_1 = l19;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 17usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l13;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    l13 = s_i32_0;
                    continue 'blk_92;
                    break 'blk_92;
                }
            }
            s_i32_0 = l36;
            s_i32_0 = ops::i32_eqz(s_i32_0);
            'blk_94: {
                if s_i32_0 != 0 {
                    s_i32_0 = l38;
                    'blk_95: {
                        if s_i32_0 != 0 {
                            s_i32_0 = l0;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_ret_void(vm, c_map, 39usize, s_i32_0, host);
                            vm.exit_call();
                            __call_res?;
                            s_i32_0 = 0i32;
                            l13 = s_i32_0;
                            'blk_96: {
                                'blk_97: loop {
                                    vm.deduct_1(host)?;
                                    s_i32_0 = l13;
                                    s_i32_1 = l12;
                                    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                                    if s_i32_0 != 0 {
                                        break 'blk_96;
                                    }
                                    s_i32_0 = l5;
                                    s_i32_1 = 32i32;
                                    s_i32_2 = l17;
                                    s_i32_3 = l13;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 16usize, s_i32_2, s_i32_3, host);
                                    vm.exit_call();
                                    let __t98 = __call_res?;
                                    s_i32_2 = __t98;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                                    vm.exit_call();
                                    __call_res?;
                                    s_i32_0 = l13;
                                    s_i32_1 = 1i32;
                                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                                    l13 = s_i32_0;
                                    continue 'blk_97;
                                    break 'blk_97;
                                }
                            }
                            s_i32_0 = l5;
                            s_i32_1 = 18i32;
                            s_i32_2 = l37;
                            s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 36u32)?;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                            vm.exit_call();
                            __call_res?;
                            s_i32_0 = 1i32;
                            return Ok(s_i32_0);
                        } else {
                            s_i32_0 = 0i32;
                            l13 = s_i32_0;
                            'blk_98: {
                                'blk_99: loop {
                                    vm.deduct_1(host)?;
                                    s_i32_0 = l13;
                                    s_i32_1 = l12;
                                    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                                    if s_i32_0 != 0 {
                                        break 'blk_98;
                                    }
                                    s_i32_0 = l5;
                                    s_i32_1 = 32i32;
                                    s_i32_2 = l17;
                                    s_i32_3 = l13;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 16usize, s_i32_2, s_i32_3, host);
                                    vm.exit_call();
                                    let __t99 = __call_res?;
                                    s_i32_2 = __t99;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                                    vm.exit_call();
                                    __call_res?;
                                    s_i32_0 = l13;
                                    s_i32_1 = 1i32;
                                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                                    l13 = s_i32_0;
                                    continue 'blk_99;
                                    break 'blk_99;
                                }
                            }
                            s_i32_0 = l5;
                            s_i32_1 = 16i32;
                            s_i32_2 = l37;
                            s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 36u32)?;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                            vm.exit_call();
                            __call_res?;
                            s_i32_0 = l14;
                            s_i32_1 = 1i32;
                            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                            'blk_100: {
                                if s_i32_0 != 0 {
                                    s_i32_0 = l5;
                                    s_i32_1 = 0i32;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 6usize, s_i32_0, s_i32_1, host);
                                    vm.exit_call();
                                    __call_res?;
                                }
                            }
                            s_i32_0 = l14;
                            return Ok(s_i32_0);
                        }
                    }
                } else {
                    s_i32_0 = l36;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 20u32)?;
                    l39 = s_i32_0;
                    s_i32_0 = 16i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 15usize, s_i32_0, host);
                    vm.exit_call();
                    let __t100 = __call_res?;
                    s_i32_0 = __t100;
                    l40 = s_i32_0;
                    s_i32_0 = 0i32;
                    l13 = s_i32_0;
                    'blk_101: {
                        'blk_102: loop {
                            vm.deduct_1(host)?;
                            s_i32_0 = l13;
                            s_i32_1 = l39;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 14usize, s_i32_1, host);
                            vm.exit_call();
                            let __t101 = __call_res?;
                            s_i32_1 = __t101;
                            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                            if s_i32_0 != 0 {
                                break 'blk_101;
                            }
                            s_i32_0 = l39;
                            s_i32_1 = l13;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 16usize, s_i32_0, s_i32_1, host);
                            vm.exit_call();
                            let __t102 = __call_res?;
                            s_i32_0 = __t102;
                            l41 = s_i32_0;
                            s_i32_0 = l0;
                            s_i32_1 = l41;
                            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 0u32)?;
                            s_i32_2 = l41;
                            s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 4u32)?;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 29usize, s_i32_0, s_i32_1, s_i32_2, host);
                            vm.exit_call();
                            let __t103 = __call_res?;
                            s_i32_0 = __t103;
                            l30 = s_i32_0;
                            s_i32_0 = l30;
                            s_i32_0 = ops::i32_eqz(s_i32_0);
                            'blk_103: {
                                if s_i32_0 != 0 {
                                    return Err(WasmTrap::Unreachable);
                                }
                            }
                            s_i32_0 = l41;
                            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 12u32)?;
                            'blk_104: {
                                if s_i32_0 != 0 {
                                    s_i32_0 = l30;
                                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
                                    s_i32_0 = ops::i32_eqz(s_i32_0);
                                    'blk_105: {
                                        if s_i32_0 != 0 {
                                            return Err(WasmTrap::Unreachable);
                                        }
                                    }
                                    s_i32_0 = l40;
                                    s_i32_1 = l30;
                                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 12u32)?;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 17usize, s_i32_0, s_i32_1, host);
                                    vm.exit_call();
                                    __call_res?;
                                } else {
                                    s_i32_0 = l0;
                                    s_i32_1 = l30;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 30usize, s_i32_0, s_i32_1, host);
                                    vm.exit_call();
                                    let __t104 = __call_res?;
                                    s_i32_0 = __t104;
                                    s_i32_0 = l5;
                                    s_i32_1 = l41;
                                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 8u32)?;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_1, host);
                                    vm.exit_call();
                                    let __t105 = __call_res?;
                                    s_i32_1 = __t105;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, host);
                                    vm.exit_call();
                                    let __t106 = __call_res?;
                                    s_i32_0 = __t106;
                                    l19 = s_i32_0;
                                    s_i32_0 = l5;
                                    s_i32_1 = 33i32;
                                    s_i32_2 = l19;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                                    vm.exit_call();
                                    __call_res?;
                                    s_i32_0 = l40;
                                    s_i32_1 = l19;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 17usize, s_i32_0, s_i32_1, host);
                                    vm.exit_call();
                                    __call_res?;
                                }
                            }
                            s_i32_0 = l13;
                            s_i32_1 = 1i32;
                            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                            l13 = s_i32_0;
                            continue 'blk_102;
                            break 'blk_102;
                        }
                    }
                    s_i32_0 = l38;
                    'blk_106: {
                        if s_i32_0 != 0 {
                            s_i32_0 = vm.get_i32(*g_map.get(2usize).ok_or(WasmTrap::Unreachable)?)?;
                            'blk_107: {
                                if s_i32_0 != 0 {
                                    s_i32_0 = l36;
                                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 12u32)?;
                                    s_i32_1 = l0;
                                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
                                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                                    'blk_108: {
                                        if s_i32_0 != 0 {
                                            s_i32_0 = l0;
                                            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
                                            'blk_109: {
                                                if s_i32_0 != 0 {
                                                    s_i32_0 = l0;
                                                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 20u32)?;
                                                } else {
                                                    s_i32_0 = l0;
                                                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 16u32)?;
                                                }
                                            }
                                            l19 = s_i32_0;
                                            s_i32_0 = l5;
                                            s_i32_1 = 32i32;
                                            s_i32_2 = l19;
                                            vm.enter_call()?;
                                            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                                            vm.exit_call();
                                            __call_res?;
                                        } else {
                                            s_i32_0 = l0;
                                            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 16u32)?;
                                            l19 = s_i32_0;
                                            s_i32_0 = l19;
                                            s_i32_1 = 0i32;
                                            s_i32_0 = ops::i32_ge_s(s_i32_0, s_i32_1);
                                            'blk_110: {
                                                if s_i32_0 != 0 {
                                                    s_i32_0 = l5;
                                                    s_i32_1 = 32i32;
                                                    s_i32_2 = l19;
                                                    vm.enter_call()?;
                                                    let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                                                    vm.exit_call();
                                                    __call_res?;
                                                    s_i32_0 = l5;
                                                    s_i32_1 = 36i32;
                                                    s_i32_2 = vm.get_i32(*g_map.get(3usize).ok_or(WasmTrap::Unreachable)?)?;
                                                    vm.enter_call()?;
                                                    let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                                                    vm.exit_call();
                                                    __call_res?;
                                                }
                                            }
                                            s_i32_0 = l0;
                                            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 20u32)?;
                                            s_i32_1 = 0i32;
                                            s_i32_0 = ops::i32_ge_s(s_i32_0, s_i32_1);
                                            'blk_111: {
                                                if s_i32_0 != 0 {
                                                    s_i32_0 = l0;
                                                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 20u32)?;
                                                } else {
                                                    s_i32_0 = l0;
                                                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 16u32)?;
                                                }
                                            }
                                            l19 = s_i32_0;
                                            s_i32_0 = l5;
                                            s_i32_1 = 32i32;
                                            s_i32_2 = l19;
                                            vm.enter_call()?;
                                            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                                            vm.exit_call();
                                            __call_res?;
                                        }
                                    }
                                }
                            }
                            s_i32_0 = 0i32;
                            l13 = s_i32_0;
                            'blk_112: {
                                'blk_113: loop {
                                    vm.deduct_1(host)?;
                                    s_i32_0 = l13;
                                    s_i32_1 = l40;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 14usize, s_i32_1, host);
                                    vm.exit_call();
                                    let __t107 = __call_res?;
                                    s_i32_1 = __t107;
                                    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                                    if s_i32_0 != 0 {
                                        break 'blk_112;
                                    }
                                    s_i32_0 = l5;
                                    s_i32_1 = 32i32;
                                    s_i32_2 = l40;
                                    s_i32_3 = l13;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 16usize, s_i32_2, s_i32_3, host);
                                    vm.exit_call();
                                    let __t108 = __call_res?;
                                    s_i32_2 = __t108;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                                    vm.exit_call();
                                    __call_res?;
                                    s_i32_0 = l13;
                                    s_i32_1 = 1i32;
                                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                                    l13 = s_i32_0;
                                    continue 'blk_113;
                                    break 'blk_113;
                                }
                            }
                            s_i32_0 = 0i32;
                            l13 = s_i32_0;
                            'blk_114: {
                                'blk_115: loop {
                                    vm.deduct_1(host)?;
                                    s_i32_0 = l13;
                                    s_i32_1 = l12;
                                    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                                    if s_i32_0 != 0 {
                                        break 'blk_114;
                                    }
                                    s_i32_0 = l5;
                                    s_i32_1 = 32i32;
                                    s_i32_2 = l17;
                                    s_i32_3 = l13;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 16usize, s_i32_2, s_i32_3, host);
                                    vm.exit_call();
                                    let __t109 = __call_res?;
                                    s_i32_2 = __t109;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                                    vm.exit_call();
                                    __call_res?;
                                    s_i32_0 = l13;
                                    s_i32_1 = 1i32;
                                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                                    l13 = s_i32_0;
                                    continue 'blk_115;
                                    break 'blk_115;
                                }
                            }
                            s_i32_0 = l5;
                            s_i32_1 = 18i32;
                            s_i32_2 = l36;
                            s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 8u32)?;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                            vm.exit_call();
                            __call_res?;
                            s_i32_0 = 1i32;
                            return Ok(s_i32_0);
                        } else {
                            s_i32_0 = vm.get_i32(*g_map.get(2usize).ok_or(WasmTrap::Unreachable)?)?;
                            'blk_116: {
                                if s_i32_0 != 0 {
                                    s_i32_0 = l5;
                                    s_i32_1 = 35i32;
                                    s_i32_2 = vm.get_i32(*g_map.get(3usize).ok_or(WasmTrap::Unreachable)?)?;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                                    vm.exit_call();
                                    __call_res?;
                                }
                            }
                            s_i32_0 = 0i32;
                            l13 = s_i32_0;
                            'blk_117: {
                                'blk_118: loop {
                                    vm.deduct_1(host)?;
                                    s_i32_0 = l13;
                                    s_i32_1 = l40;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 14usize, s_i32_1, host);
                                    vm.exit_call();
                                    let __t110 = __call_res?;
                                    s_i32_1 = __t110;
                                    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                                    if s_i32_0 != 0 {
                                        break 'blk_117;
                                    }
                                    s_i32_0 = l5;
                                    s_i32_1 = 32i32;
                                    s_i32_2 = l40;
                                    s_i32_3 = l13;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 16usize, s_i32_2, s_i32_3, host);
                                    vm.exit_call();
                                    let __t111 = __call_res?;
                                    s_i32_2 = __t111;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                                    vm.exit_call();
                                    __call_res?;
                                    s_i32_0 = l13;
                                    s_i32_1 = 1i32;
                                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                                    l13 = s_i32_0;
                                    continue 'blk_118;
                                    break 'blk_118;
                                }
                            }
                            s_i32_0 = 0i32;
                            l13 = s_i32_0;
                            'blk_119: {
                                'blk_120: loop {
                                    vm.deduct_1(host)?;
                                    s_i32_0 = l13;
                                    s_i32_1 = l12;
                                    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                                    if s_i32_0 != 0 {
                                        break 'blk_119;
                                    }
                                    s_i32_0 = l5;
                                    s_i32_1 = 32i32;
                                    s_i32_2 = l17;
                                    s_i32_3 = l13;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 16usize, s_i32_2, s_i32_3, host);
                                    vm.exit_call();
                                    let __t112 = __call_res?;
                                    s_i32_2 = __t112;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                                    vm.exit_call();
                                    __call_res?;
                                    s_i32_0 = l13;
                                    s_i32_1 = 1i32;
                                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                                    l13 = s_i32_0;
                                    continue 'blk_120;
                                    break 'blk_120;
                                }
                            }
                            s_i32_0 = l5;
                            s_i32_1 = 16i32;
                            s_i32_2 = l36;
                            s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 8u32)?;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                            vm.exit_call();
                            __call_res?;
                            s_i32_0 = l14;
                            s_i32_1 = 1i32;
                            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                            'blk_121: {
                                if s_i32_0 != 0 {
                                    s_i32_0 = l5;
                                    s_i32_1 = 0i32;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 6usize, s_i32_0, s_i32_1, host);
                                    vm.exit_call();
                                    __call_res?;
                                }
                            }
                            s_i32_0 = l14;
                            return Ok(s_i32_0);
                        }
                    }
                }
            }
        }
    }
    s_i32_0 = l3;
    s_i32_1 = 17i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_122: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
            s_i32_2 = 0i32;
            vm.enter_call()?;
            let __call_res = Self::f_17fe7444ad16(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            let __t113 = __call_res?;
            s_i32_0 = __t113;
            l15 = s_i32_0;
            s_i32_0 = l15;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_123: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l15;
            s_i32_1 = 2i32;
            s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
            'blk_124: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = l0;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 8u32)?;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 35usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t114 = __call_res?;
            s_i32_0 = __t114;
            l42 = s_i32_0;
            s_i32_0 = l0;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 12u32)?;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 35usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t115 = __call_res?;
            s_i32_0 = __t115;
            l43 = s_i32_0;
            s_i32_0 = l42;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_125: {
                if s_i32_0 != 0 {
                    s_i32_0 = l43;
                } else {
                    s_i32_0 = l43;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                    'blk_126: {
                        if s_i32_0 != 0 {
                            s_i32_0 = l42;
                        } else {
                            s_i32_0 = l42;
                            s_i32_1 = l43;
                            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                            'blk_127: {
                                if s_i32_0 != 0 {
                                    s_i32_0 = l42;
                                } else {
                                    return Err(WasmTrap::Unreachable);
                                }
                            }
                        }
                    }
                }
            }
            l14 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = 4i32;
            s_i32_2 = l14;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_2, host);
            vm.exit_call();
            let __t116 = __call_res?;
            s_i32_2 = __t116;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 40usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l0;
            s_i32_1 = l0;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 36u32)?;
            s_i32_2 = 1i32;
            s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
            ops::i32_store(vm, 0, s_i32_0, 36u32, s_i32_1)?;
            vm.deduct_1(host)?;
            s_i32_0 = l0;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 8u32)?;
            s_i32_2 = l2;
            vm.enter_call()?;
            let __call_res = Self::f_17fe7444ad16(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            let __t117 = __call_res?;
            s_i32_0 = __t117;
            l42 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = 5i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 6usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l0;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 12u32)?;
            s_i32_2 = l2;
            vm.enter_call()?;
            let __call_res = Self::f_17fe7444ad16(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            let __t118 = __call_res?;
            s_i32_0 = __t118;
            l43 = s_i32_0;
            s_i32_0 = l0;
            s_i32_1 = l0;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 36u32)?;
            s_i32_2 = 1i32;
            s_i32_1 = ops::i32_sub(s_i32_1, s_i32_2);
            ops::i32_store(vm, 0, s_i32_0, 36u32, s_i32_1)?;
            s_i32_0 = l5;
            s_i32_1 = 11i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 6usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l42;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            s_i32_1 = l43;
            s_i32_2 = 1i32;
            s_i32_1 = ops::i32_eq(s_i32_1, s_i32_2);
            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
            'blk_128: {
                if s_i32_0 != 0 {
                    s_i32_0 = l5;
                    s_i32_1 = 0i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 6usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = 1i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l14;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l3;
    s_i32_1 = 18i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_129: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 40u32)?;
            'blk_130: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = l5;
            s_i32_1 = 2i32;
            s_i32_2 = 0i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 40usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l0;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 36u32)?;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l48 = s_i32_0;
            s_i32_0 = l0;
            s_i32_1 = l48;
            ops::i32_store(vm, 0, s_i32_0, 36u32, s_i32_1)?;
            s_i32_0 = l5;
            s_i32_1 = 3i32;
            s_i32_2 = 0i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 40usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l48;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l49 = s_i32_0;
            s_i32_0 = l0;
            s_i32_1 = l49;
            ops::i32_store(vm, 0, s_i32_0, 36u32, s_i32_1)?;
            s_i32_0 = l0;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
            s_i32_2 = 0i32;
            vm.enter_call()?;
            let __call_res = Self::f_17fe7444ad16(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            let __t119 = __call_res?;
            s_i32_0 = __t119;
            l15 = s_i32_0;
            s_i32_0 = l15;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
            'blk_131: {
                if s_i32_0 != 0 {
                    s_i32_0 = l15;
                    s_i32_1 = 2i32;
                    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
                    'blk_132: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = l5;
                    s_i32_1 = 69i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 6usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l5;
                    s_i32_1 = 13i32;
                    s_i32_2 = l0;
                    vm.deduct_1(host)?;
                    s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 36u32)?;
                    s_i32_3 = l48;
                    s_i32_2 = ops::i32_sub(s_i32_2, s_i32_3);
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = 8i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 19usize, s_i32_0, host);
                    vm.exit_call();
                    let __t120 = __call_res?;
                    s_i32_0 = __t120;
                    l51 = s_i32_0;
                    s_i32_0 = l51;
                    s_i32_1 = l48;
                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l51;
                    s_i32_1 = l49;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l0;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 32u32)?;
                    l52 = s_i32_0;
                    s_i32_0 = l52;
                    s_i32_1 = l51;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 17usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l0;
                    s_i32_1 = l1;
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 8u32)?;
                    s_i32_2 = 0i32;
                    vm.enter_call()?;
                    let __call_res = Self::f_17fe7444ad16(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    let __t121 = __call_res?;
                    s_i32_0 = __t121;
                    l42 = s_i32_0;
                    s_i32_0 = l52;
                    s_i32_1 = l52;
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
                    s_i32_2 = 4i32;
                    s_i32_1 = ops::i32_sub(s_i32_1, s_i32_2);
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l42;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
                    'blk_133: {
                        if s_i32_0 != 0 {
                            s_i32_0 = l42;
                            s_i32_1 = 0i32;
                            s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
                            'blk_134: {
                                if s_i32_0 != 0 {
                                    return Err(WasmTrap::Unreachable);
                                }
                            }
                            s_i32_0 = l5;
                            s_i32_1 = 12i32;
                            s_i32_2 = l0;
                            s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 36u32)?;
                            s_i32_3 = l49;
                            s_i32_2 = ops::i32_sub(s_i32_2, s_i32_3);
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                            vm.exit_call();
                            __call_res?;
                        }
                    }
                }
            }
            s_i32_0 = l0;
            s_i32_1 = l0;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 36u32)?;
            s_i32_2 = 1i32;
            s_i32_1 = ops::i32_sub(s_i32_1, s_i32_2);
            ops::i32_store(vm, 0, s_i32_0, 36u32, s_i32_1)?;
            s_i32_0 = l5;
            s_i32_1 = 11i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 6usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l0;
            s_i32_1 = l0;
            vm.deduct_1(host)?;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 36u32)?;
            s_i32_2 = 1i32;
            s_i32_1 = ops::i32_sub(s_i32_1, s_i32_2);
            ops::i32_store(vm, 0, s_i32_0, 36u32, s_i32_1)?;
            s_i32_0 = l5;
            s_i32_1 = 11i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 6usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = 0i32;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l3;
    s_i32_1 = 19i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_135: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 40u32)?;
            'blk_136: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = l0;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 16u32)?;
            s_i32_2 = 0i32;
            vm.enter_call()?;
            let __call_res = Self::f_17fe7444ad16(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            let __t122 = __call_res?;
            s_i32_0 = __t122;
            l42 = s_i32_0;
            s_i32_0 = l42;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_137: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l4;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 9usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t123 = __call_res?;
            s_i32_0 = __t123;
            l14 = s_i32_0;
            s_i32_0 = l14;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_void(vm, c_map, 10usize, s_i32_0, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l42;
            s_i32_1 = 8i32;
            s_i32_2 = l14;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 8usize, s_i32_1, s_i32_2, host);
            vm.exit_call();
            let __t124 = __call_res?;
            s_i32_1 = __t124;
            s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
            'blk_138: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = l14;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_0, host);
            vm.exit_call();
            let __t125 = __call_res?;
            s_i32_0 = __t125;
            l16 = s_i32_0;
            s_i32_0 = l14;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 12usize, s_i32_0, host);
            vm.exit_call();
            let __t126 = __call_res?;
            s_i32_0 = __t126;
            l20 = s_i32_0;
            s_i32_0 = l14;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 13usize, s_i32_0, host);
            vm.exit_call();
            let __t127 = __call_res?;
            s_i32_0 = __t127;
            l21 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = 2228432i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t128 = __call_res?;
            s_i32_0 = __t128;
            l9 = s_i32_0;
            s_i32_0 = l5;
            vm.deduct_1(host)?;
            s_i32_1 = 2228432i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t129 = __call_res?;
            s_i32_0 = __t129;
            l45 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = 2228432i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t130 = __call_res?;
            s_i32_0 = __t130;
            l46 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = 2228432i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t131 = __call_res?;
            s_i32_0 = __t131;
            l47 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = l16;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t132 = __call_res?;
            s_i32_0 = __t132;
            l19 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = 33i32;
            s_i32_2 = l9;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 32i32;
            s_i32_2 = l9;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 40i32;
            s_i32_2 = 2i32;
            s_i32_3 = 0i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_void(vm, c_map, 7usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 33i32;
            s_i32_2 = l45;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 32i32;
            s_i32_2 = l9;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 40i32;
            s_i32_2 = 2i32;
            s_i32_3 = 4i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_void(vm, c_map, 7usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 33i32;
            s_i32_2 = l46;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 0i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 33i32;
            s_i32_2 = l47;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 2i32;
            s_i32_2 = 0i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 40usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l0;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 36u32)?;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l48 = s_i32_0;
            s_i32_0 = l0;
            s_i32_1 = l48;
            ops::i32_store(vm, 0, s_i32_0, 36u32, s_i32_1)?;
            s_i32_0 = l5;
            s_i32_1 = 3i32;
            s_i32_2 = 0i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 40usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l48;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l50 = s_i32_0;
            s_i32_0 = l0;
            s_i32_1 = l50;
            ops::i32_store(vm, 0, s_i32_0, 36u32, s_i32_1)?;
            s_i32_0 = l5;
            s_i32_1 = 32i32;
            s_i32_2 = l47;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 32i32;
            s_i32_2 = l46;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 79i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 6usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 13i32;
            s_i32_2 = l0;
            s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 36u32)?;
            s_i32_3 = l48;
            s_i32_2 = ops::i32_sub(s_i32_2, s_i32_3);
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 32i32;
            s_i32_2 = l45;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 32i32;
            s_i32_2 = l47;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = l20;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 108i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 6usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 106i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 6usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = l20;
            s_i32_2 = l21;
            s_i32_3 = 0i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_void(vm, c_map, 26usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 33i32;
            s_i32_2 = l19;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 2i32;
            s_i32_2 = 0i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 40usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l0;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 36u32)?;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l49 = s_i32_0;
            s_i32_0 = l0;
            s_i32_1 = l49;
            ops::i32_store(vm, 0, s_i32_0, 36u32, s_i32_1)?;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 19usize, s_i32_0, host);
            vm.exit_call();
            let __t133 = __call_res?;
            s_i32_0 = __t133;
            l33 = s_i32_0;
            s_i32_0 = 8i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 15usize, s_i32_0, host);
            vm.exit_call();
            let __t134 = __call_res?;
            s_i32_0 = __t134;
            l34 = s_i32_0;
            s_i32_0 = 32i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 19usize, s_i32_0, host);
            vm.exit_call();
            let __t135 = __call_res?;
            s_i32_0 = __t135;
            l30 = s_i32_0;
            s_i32_0 = l30;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 8u32)?;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l30;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 12u32)?;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l30;
            s_i32_1 = l19;
            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
            s_i32_0 = l30;
            s_i32_1 = l14;
            ops::i32_store(vm, 0, s_i32_0, 16u32, s_i32_1)?;
            s_i32_0 = l30;
            s_i32_1 = -1i32;
            ops::i32_store(vm, 0, s_i32_0, 24u32, s_i32_1)?;
            s_i32_0 = l34;
            s_i32_1 = l30;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 17usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l33;
            s_i32_1 = l34;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l33;
            s_i32_1 = 8i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 15usize, s_i32_1, host);
            vm.exit_call();
            let __t136 = __call_res?;
            s_i32_1 = __t136;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l0;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 28u32)?;
            l32 = s_i32_0;
            s_i32_0 = l32;
            s_i32_1 = l33;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 17usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = 8i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 19usize, s_i32_0, host);
            vm.exit_call();
            let __t137 = __call_res?;
            s_i32_0 = __t137;
            l51 = s_i32_0;
            s_i32_0 = l51;
            s_i32_1 = l48;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l51;
            s_i32_1 = l49;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l0;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 32u32)?;
            l52 = s_i32_0;
            s_i32_0 = l52;
            s_i32_1 = l51;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 17usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l0;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 20u32)?;
            s_i32_2 = 0i32;
            vm.enter_call()?;
            let __call_res = Self::f_17fe7444ad16(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            let __t138 = __call_res?;
            s_i32_0 = __t138;
            l15 = s_i32_0;
            s_i32_0 = l15;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
            s_i32_1 = l15;
            s_i32_2 = 0i32;
            s_i32_1 = ops::i32_ne(s_i32_1, s_i32_2);
            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
            'blk_139: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = l52;
            s_i32_1 = l52;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
            s_i32_2 = 4i32;
            s_i32_1 = ops::i32_sub(s_i32_1, s_i32_2);
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l32;
            s_i32_1 = l32;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
            s_i32_2 = 4i32;
            s_i32_1 = ops::i32_sub(s_i32_1, s_i32_2);
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l0;
            s_i32_1 = l0;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 36u32)?;
            s_i32_2 = 1i32;
            s_i32_1 = ops::i32_sub(s_i32_1, s_i32_2);
            ops::i32_store(vm, 0, s_i32_0, 36u32, s_i32_1)?;
            s_i32_0 = l5;
            s_i32_1 = 11i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 6usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 32i32;
            s_i32_2 = l47;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 1i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 106i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 6usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 33i32;
            s_i32_2 = l47;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 12i32;
            s_i32_2 = l0;
            s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 36u32)?;
            s_i32_3 = l50;
            s_i32_2 = ops::i32_sub(s_i32_2, s_i32_3);
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l0;
            s_i32_1 = l0;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 36u32)?;
            s_i32_2 = 1i32;
            s_i32_1 = ops::i32_sub(s_i32_1, s_i32_2);
            ops::i32_store(vm, 0, s_i32_0, 36u32, s_i32_1)?;
            s_i32_0 = l5;
            s_i32_1 = 11i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 6usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l0;
            s_i32_1 = l0;
            vm.deduct_1(host)?;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 36u32)?;
            s_i32_2 = 1i32;
            s_i32_1 = ops::i32_sub(s_i32_1, s_i32_2);
            ops::i32_store(vm, 0, s_i32_0, 36u32, s_i32_1)?;
            s_i32_0 = l5;
            s_i32_1 = 11i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 6usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = 0i32;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l3;
    s_i32_1 = 20i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_140: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 32u32)?;
            l52 = s_i32_0;
            s_i32_0 = l52;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 14usize, s_i32_0, host);
            vm.exit_call();
            let __t139 = __call_res?;
            s_i32_0 = __t139;
            l12 = s_i32_0;
            s_i32_0 = l12;
            s_i32_0 = ops::i32_eqz(s_i32_0);
            'blk_141: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = l52;
            s_i32_1 = l12;
            s_i32_2 = 1i32;
            s_i32_1 = ops::i32_sub(s_i32_1, s_i32_2);
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 16usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t140 = __call_res?;
            s_i32_0 = __t140;
            l51 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = 12i32;
            s_i32_2 = l0;
            s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 36u32)?;
            s_i32_3 = l51;
            s_i32_3 = ops::i32_load(vm, 0, s_i32_3, 0u32)?;
            s_i32_2 = ops::i32_sub(s_i32_2, s_i32_3);
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = 1i32;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l3;
    s_i32_1 = 21i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_142: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 32u32)?;
            l52 = s_i32_0;
            s_i32_0 = l52;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 14usize, s_i32_0, host);
            vm.exit_call();
            let __t141 = __call_res?;
            s_i32_0 = __t141;
            l12 = s_i32_0;
            s_i32_0 = l12;
            s_i32_0 = ops::i32_eqz(s_i32_0);
            'blk_143: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            vm.deduct_1(host)?;
            s_i32_0 = l52;
            s_i32_1 = l12;
            s_i32_2 = 1i32;
            s_i32_1 = ops::i32_sub(s_i32_1, s_i32_2);
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 16usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t142 = __call_res?;
            s_i32_0 = __t142;
            l51 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = 12i32;
            s_i32_2 = l0;
            s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 36u32)?;
            s_i32_3 = l51;
            s_i32_3 = ops::i32_load(vm, 0, s_i32_3, 4u32)?;
            s_i32_2 = ops::i32_sub(s_i32_2, s_i32_3);
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = 1i32;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l3;
    s_i32_1 = 22i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_144: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 40u32)?;
            'blk_145: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = l0;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 12u32)?;
            vm.deduct_1(host)?;
            l14 = s_i32_0;
            s_i32_0 = l1;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
            'blk_146: {
                if s_i32_0 != 0 {
                    s_i32_0 = l14;
                    s_i32_1 = 0i32;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                    'blk_147: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = l0;
                    s_i32_1 = l1;
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
                    s_i32_2 = 1i32;
                    vm.enter_call()?;
                    let __call_res = Self::f_17fe7444ad16(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    let __t143 = __call_res?;
                    s_i32_0 = __t143;
                    l15 = s_i32_0;
                    s_i32_0 = l15;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                    'blk_148: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 1i32;
                            return Ok(s_i32_0);
                        }
                    }
                    s_i32_0 = l15;
                    s_i32_1 = l14;
                    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
                    'blk_149: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = l14;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_0, host);
                    vm.exit_call();
                    let __t144 = __call_res?;
                    s_i32_0 = __t144;
                    l16 = s_i32_0;
                    s_i32_0 = l16;
                    'blk_150: {
                        if s_i32_0 != 0 {
                            s_i32_0 = l5;
                            s_i32_1 = l16;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, host);
                            vm.exit_call();
                            let __t145 = __call_res?;
                            s_i32_0 = __t145;
                            l19 = s_i32_0;
                            s_i32_0 = l5;
                            s_i32_1 = 33i32;
                            s_i32_2 = l19;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                            vm.exit_call();
                            __call_res?;
                            s_i32_0 = l0;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_ret_void(vm, c_map, 39usize, s_i32_0, host);
                            vm.exit_call();
                            __call_res?;
                            s_i32_0 = l5;
                            s_i32_1 = 32i32;
                            s_i32_2 = l19;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                            vm.exit_call();
                            __call_res?;
                        } else {
                            s_i32_0 = l0;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_ret_void(vm, c_map, 39usize, s_i32_0, host);
                            vm.exit_call();
                            __call_res?;
                        }
                    }
                } else {
                    s_i32_0 = l14;
                    s_i32_1 = 0i32;
                    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
                    'blk_151: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = l0;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_void(vm, c_map, 39usize, s_i32_0, host);
                    vm.exit_call();
                    __call_res?;
                }
            }
            s_i32_0 = l5;
            s_i32_1 = 15i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 6usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = 1i32;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l3;
    s_i32_1 = 23i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_152: {
        if s_i32_0 != 0 {
            s_i32_0 = l5;
            s_i32_1 = 0i32;
            vm.deduct_1(host)?;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 6usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = 1i32;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l3;
    s_i32_1 = 25i32;
    vm.deduct_1(host)?;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_153: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
            l53 = s_i32_0;
            s_i32_0 = l0;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 8u32)?;
            s_i32_2 = 0i32;
            vm.enter_call()?;
            let __call_res = Self::f_17fe7444ad16(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            let __t146 = __call_res?;
            s_i32_0 = __t146;
            l42 = s_i32_0;
            s_i32_0 = l42;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_154: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l53;
            s_i32_1 = 1116655i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_155: {
                if s_i32_0 != 0 {
                    s_i32_0 = l42;
                    s_i32_1 = 8i32;
                    s_i32_0 = ops::i32_lt_u(s_i32_0, s_i32_1);
                    'blk_156: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = vm.get_i32(*g_map.get(1usize).ok_or(WasmTrap::Unreachable)?)?;
                    s_i32_1 = l42;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 16usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t147 = __call_res?;
                    s_i32_0 = __t147;
                    l24 = s_i32_0;
                    s_i32_0 = l24;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                    l12 = s_i32_0;
                    s_i32_0 = l12;
                    s_i32_1 = 8i32;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                    'blk_157: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 1116688i32;
                        } else {
                            s_i32_0 = l12;
                            s_i32_1 = 9i32;
                            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                            'blk_158: {
                                if s_i32_0 != 0 {
                                    s_i32_0 = 1116689i32;
                                } else {
                                    return Err(WasmTrap::Unreachable);
                                }
                            }
                        }
                    }
                    l7 = s_i32_0;
                    s_i32_0 = l5;
                    s_i32_1 = 16i32;
                    s_i32_2 = 1116655i32;
                    s_i32_3 = 2i32;
                    s_i32_4 = l7;
                    s_i32_5 = 1114387i32;
                    s_i32_6 = 0i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_i32_i32_ret_i32(vm, c_map, 41usize, s_i32_2, s_i32_3, s_i32_4, s_i32_5, s_i32_6, host);
                    vm.exit_call();
                    let __t148 = __call_res?;
                    s_i32_2 = __t148;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = 7i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l53;
            s_i32_1 = 1116697i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_159: {
                if s_i32_0 != 0 {
                    s_i32_0 = l0;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 40u32)?;
                    'blk_160: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = l42;
                    s_i32_1 = 8i32;
                    s_i32_0 = ops::i32_lt_u(s_i32_0, s_i32_1);
                    'blk_161: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = vm.get_i32(*g_map.get(1usize).ok_or(WasmTrap::Unreachable)?)?;
                    s_i32_1 = l42;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 16usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t149 = __call_res?;
                    s_i32_0 = __t149;
                    l24 = s_i32_0;
                    s_i32_0 = l24;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                    s_i32_1 = 9i32;
                    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
                    'blk_162: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = l24;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
                    l15 = s_i32_0;
                    s_i32_0 = l15;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 11usize, s_i32_0, host);
                    vm.exit_call();
                    let __t150 = __call_res?;
                    s_i32_0 = __t150;
                    'blk_163: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = l5;
                    s_i32_1 = 16i32;
                    s_i32_2 = 1116697i32;
                    s_i32_3 = 2i32;
                    s_i32_4 = 1116689i32;
                    s_i32_5 = 1114387i32;
                    s_i32_6 = 0i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_i32_i32_ret_i32(vm, c_map, 41usize, s_i32_2, s_i32_3, s_i32_4, s_i32_5, s_i32_6, host);
                    vm.exit_call();
                    let __t151 = __call_res?;
                    s_i32_2 = __t151;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = 8i32;
                    s_i32_1 = l15;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 8usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t152 = __call_res?;
                    s_i32_0 = __t152;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l53;
            s_i32_1 = 1116728i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_164: {
                if s_i32_0 != 0 {
                    s_i32_0 = l42;
                    s_i32_1 = 8i32;
                    s_i32_0 = ops::i32_lt_u(s_i32_0, s_i32_1);
                    'blk_165: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = vm.get_i32(*g_map.get(1usize).ok_or(WasmTrap::Unreachable)?)?;
                    s_i32_1 = l42;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 16usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t153 = __call_res?;
                    s_i32_0 = __t153;
                    l24 = s_i32_0;
                    s_i32_0 = l24;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                    s_i32_1 = 8i32;
                    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
                    s_i32_1 = l24;
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
                    s_i32_2 = 3i32;
                    s_i32_1 = ops::i32_ne(s_i32_1, s_i32_2);
                    s_i32_0 = ops::i32_or(s_i32_0, s_i32_1);
                    'blk_166: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = l5;
                    s_i32_1 = 16i32;
                    s_i32_2 = 1116728i32;
                    s_i32_3 = 2i32;
                    s_i32_4 = 1116688i32;
                    s_i32_5 = 2228427i32;
                    s_i32_6 = 0i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_i32_i32_ret_i32(vm, c_map, 41usize, s_i32_2, s_i32_3, s_i32_4, s_i32_5, s_i32_6, host);
                    vm.exit_call();
                    let __t154 = __call_res?;
                    s_i32_2 = __t154;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l42;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l5;
            s_i32_1 = 16i32;
            s_i32_2 = l53;
            s_i32_3 = l42;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 42usize, s_i32_2, s_i32_3, host);
            vm.exit_call();
            let __t155 = __call_res?;
            s_i32_2 = __t155;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l53;
            s_i32_1 = 1116644i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_167: {
                if s_i32_0 != 0 {
                    s_i32_0 = 2i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l53;
            s_i32_1 = 1116641i32;
            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
            s_i32_1 = l53;
            s_i32_2 = 1116643i32;
            s_i32_1 = ops::i32_le_u(s_i32_1, s_i32_2);
            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
            'blk_168: {
                if s_i32_0 != 0 {
                    s_i32_0 = 5i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l42;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l3;
    s_i32_1 = 26i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_169: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
            l53 = s_i32_0;
            s_i32_0 = l0;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 8u32)?;
            s_i32_2 = 0i32;
            vm.enter_call()?;
            let __call_res = Self::f_17fe7444ad16(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            let __t156 = __call_res?;
            s_i32_0 = __t156;
            l42 = s_i32_0;
            s_i32_0 = l42;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_170: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l53;
            s_i32_1 = 1116645i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            s_i32_1 = l53;
            s_i32_2 = 1116646i32;
            s_i32_1 = ops::i32_eq(s_i32_1, s_i32_2);
            s_i32_0 = ops::i32_or(s_i32_0, s_i32_1);
            'blk_171: {
                if s_i32_0 != 0 {
                    s_i32_0 = l42;
                    s_i32_1 = 2i32;
                    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
                    'blk_172: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = l5;
                    s_i32_1 = 4i32;
                    s_i32_2 = 2228432i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 40usize, s_i32_0, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l0;
                    s_i32_1 = l0;
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 36u32)?;
                    s_i32_2 = 1i32;
                    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
                    ops::i32_store(vm, 0, s_i32_0, 36u32, s_i32_1)?;
                    s_i32_0 = l53;
                    s_i32_1 = 1116645i32;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                    'blk_173: {
                        if s_i32_0 != 0 {
                            s_i32_0 = l0;
                            s_i32_1 = l1;
                            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 12u32)?;
                            s_i32_2 = 0i32;
                            vm.enter_call()?;
                            let __call_res = Self::f_17fe7444ad16(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, s_i32_2, host);
                            vm.exit_call();
                            let __t157 = __call_res?;
                            s_i32_0 = __t157;
                            l43 = s_i32_0;
                            s_i32_0 = l43;
                            s_i32_1 = 1i32;
                            s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
                            s_i32_1 = l43;
                            s_i32_2 = 2i32;
                            s_i32_1 = ops::i32_ne(s_i32_1, s_i32_2);
                            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                            'blk_174: {
                                if s_i32_0 != 0 {
                                    return Err(WasmTrap::Unreachable);
                                }
                            }
                            s_i32_0 = l5;
                            s_i32_1 = 5i32;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 6usize, s_i32_0, s_i32_1, host);
                            vm.exit_call();
                            __call_res?;
                            s_i32_0 = l5;
                            s_i32_1 = 0i32;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
                            vm.exit_call();
                            __call_res?;
                        } else {
                            s_i32_0 = l5;
                            s_i32_1 = 1i32;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
                            vm.exit_call();
                            __call_res?;
                            s_i32_0 = l5;
                            s_i32_1 = 5i32;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 6usize, s_i32_0, s_i32_1, host);
                            vm.exit_call();
                            __call_res?;
                            s_i32_0 = l0;
                            s_i32_1 = l1;
                            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 12u32)?;
                            s_i32_2 = 0i32;
                            vm.enter_call()?;
                            let __call_res = Self::f_17fe7444ad16(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, s_i32_2, host);
                            vm.exit_call();
                            let __t158 = __call_res?;
                            s_i32_0 = __t158;
                            l43 = s_i32_0;
                            s_i32_0 = l43;
                            s_i32_1 = 1i32;
                            s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
                            s_i32_1 = l43;
                            s_i32_2 = 2i32;
                            s_i32_1 = ops::i32_ne(s_i32_1, s_i32_2);
                            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                            'blk_175: {
                                if s_i32_0 != 0 {
                                    return Err(WasmTrap::Unreachable);
                                }
                            }
                        }
                    }
                    s_i32_0 = l0;
                    s_i32_1 = l0;
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 36u32)?;
                    s_i32_2 = 1i32;
                    s_i32_1 = ops::i32_sub(s_i32_1, s_i32_2);
                    ops::i32_store(vm, 0, s_i32_0, 36u32, s_i32_1)?;
                    s_i32_0 = l5;
                    s_i32_1 = 11i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 6usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = 2i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l42;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_0, host);
            vm.exit_call();
            let __t159 = __call_res?;
            s_i32_0 = __t159;
            l16 = s_i32_0;
            s_i32_0 = l16;
            s_i32_0 = ops::i32_eqz(s_i32_0);
            'blk_176: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = l5;
            s_i32_1 = l16;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t160 = __call_res?;
            s_i32_0 = __t160;
            l45 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = 33i32;
            s_i32_2 = l45;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l0;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 12u32)?;
            s_i32_2 = 0i32;
            vm.enter_call()?;
            let __call_res = Self::f_17fe7444ad16(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            let __t161 = __call_res?;
            s_i32_0 = __t161;
            l43 = s_i32_0;
            s_i32_0 = l43;
            vm.deduct_1(host)?;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_177: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l43;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_0, host);
            vm.exit_call();
            let __t162 = __call_res?;
            s_i32_0 = __t162;
            l16 = s_i32_0;
            s_i32_0 = l16;
            s_i32_0 = ops::i32_eqz(s_i32_0);
            'blk_178: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = l5;
            s_i32_1 = l16;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t163 = __call_res?;
            s_i32_0 = __t163;
            l46 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = 33i32;
            s_i32_2 = l46;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 32i32;
            s_i32_2 = l45;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 32i32;
            s_i32_2 = l46;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l53;
            s_i32_1 = 1116647i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            s_i32_1 = l53;
            s_i32_2 = 1116648i32;
            s_i32_1 = ops::i32_eq(s_i32_1, s_i32_2);
            s_i32_0 = ops::i32_or(s_i32_0, s_i32_1);
            'blk_179: {
                if s_i32_0 != 0 {
                    s_i32_0 = l42;
                    s_i32_1 = l43;
                    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
                    'blk_180: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = l42;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 11usize, s_i32_0, host);
                    vm.exit_call();
                    let __t164 = __call_res?;
                    s_i32_0 = __t164;
                    'blk_181: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = l42;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 43usize, s_i32_0, host);
                    vm.exit_call();
                    let __t165 = __call_res?;
                    s_i32_0 = __t165;
                    'blk_182: {
                        if s_i32_0 != 0 {
                            s_i32_0 = l5;
                            s_i32_1 = 16i32;
                            s_i32_2 = l53;
                            s_i32_3 = l42;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 42usize, s_i32_2, s_i32_3, host);
                            vm.exit_call();
                            let __t166 = __call_res?;
                            s_i32_2 = __t166;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                            vm.exit_call();
                            __call_res?;
                            s_i32_0 = 2i32;
                            return Ok(s_i32_0);
                        }
                    }
                    s_i32_0 = l42;
                    s_i32_1 = 8i32;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                    'blk_183: {
                        if s_i32_0 != 0 {
                            s_i32_0 = l5;
                            s_i32_1 = 26i32;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 6usize, s_i32_0, s_i32_1, host);
                            vm.exit_call();
                            __call_res?;
                            s_i32_0 = l5;
                            s_i32_1 = 26i32;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 6usize, s_i32_0, s_i32_1, host);
                            vm.exit_call();
                            __call_res?;
                            s_i32_0 = l5;
                            s_i32_1 = l53;
                            s_i32_2 = 1116647i32;
                            s_i32_1 = ops::i32_eq(s_i32_1, s_i32_2);
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
                            vm.exit_call();
                            __call_res?;
                            s_i32_0 = 2i32;
                            return Ok(s_i32_0);
                        }
                    }
                    s_i32_0 = vm.get_i32(*g_map.get(1usize).ok_or(WasmTrap::Unreachable)?)?;
                    s_i32_1 = l42;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 16usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t167 = __call_res?;
                    s_i32_0 = __t167;
                    l24 = s_i32_0;
                    s_i32_0 = l24;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                    s_i32_1 = 8i32;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                    'blk_184: {
                        if s_i32_0 != 0 {
                            s_i32_0 = l24;
                            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
                            l15 = s_i32_0;
                            s_i32_0 = l15;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 43usize, s_i32_0, host);
                            vm.exit_call();
                            let __t168 = __call_res?;
                            s_i32_0 = __t168;
                            'blk_185: {
                                if s_i32_0 != 0 {
                                    s_i32_0 = l5;
                                    s_i32_1 = 16i32;
                                    s_i32_2 = l53;
                                    s_i32_3 = 2i32;
                                    s_i32_4 = 1116688i32;
                                    s_i32_5 = l15;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 44usize, s_i32_5, host);
                                    vm.exit_call();
                                    let __t169 = __call_res?;
                                    s_i32_5 = __t169;
                                    s_i32_6 = 0i32;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_i32_i32_i32_i32_ret_i32(vm, c_map, 41usize, s_i32_2, s_i32_3, s_i32_4, s_i32_5, s_i32_6, host);
                                    vm.exit_call();
                                    let __t170 = __call_res?;
                                    s_i32_2 = __t170;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                                    vm.exit_call();
                                    __call_res?;
                                    s_i32_0 = 2i32;
                                    return Ok(s_i32_0);
                                }
                            }
                        }
                    }
                    s_i32_0 = l5;
                    s_i32_1 = 16i32;
                    s_i32_2 = l42;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 45usize, s_i32_2, host);
                    vm.exit_call();
                    let __t171 = __call_res?;
                    s_i32_2 = __t171;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l53;
                    s_i32_1 = 1116648i32;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                    'blk_186: {
                        if s_i32_0 != 0 {
                            s_i32_0 = l5;
                            s_i32_1 = 69i32;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 6usize, s_i32_0, s_i32_1, host);
                            vm.exit_call();
                            __call_res?;
                        }
                    }
                    s_i32_0 = 2i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l53;
            s_i32_1 = 1116656i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_187: {
                if s_i32_0 != 0 {
                    s_i32_0 = l42;
                    s_i32_1 = 8i32;
                    s_i32_0 = ops::i32_lt_u(s_i32_0, s_i32_1);
                    s_i32_1 = l43;
                    s_i32_2 = 7i32;
                    s_i32_1 = ops::i32_ne(s_i32_1, s_i32_2);
                    s_i32_0 = ops::i32_or(s_i32_0, s_i32_1);
                    'blk_188: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = vm.get_i32(*g_map.get(1usize).ok_or(WasmTrap::Unreachable)?)?;
                    s_i32_1 = l42;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 16usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t172 = __call_res?;
                    s_i32_0 = __t172;
                    l24 = s_i32_0;
                    s_i32_0 = l24;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                    l12 = s_i32_0;
                    s_i32_0 = l24;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
                    l15 = s_i32_0;
                    s_i32_0 = l12;
                    s_i32_1 = 8i32;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                    'blk_189: {
                        if s_i32_0 != 0 {
                            s_i32_0 = l5;
                            s_i32_1 = 16i32;
                            s_i32_2 = 1116656i32;
                            s_i32_3 = 1116688i32;
                            s_i32_4 = l15;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 28usize, s_i32_2, s_i32_3, s_i32_4, host);
                            vm.exit_call();
                            let __t173 = __call_res?;
                            s_i32_2 = __t173;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                            vm.exit_call();
                            __call_res?;
                            s_i32_0 = l15;
                            return Ok(s_i32_0);
                        } else {
                            s_i32_0 = l12;
                            s_i32_1 = 9i32;
                            s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
                            'blk_190: {
                                if s_i32_0 != 0 {
                                    return Err(WasmTrap::Unreachable);
                                }
                            }
                            s_i32_0 = l5;
                            s_i32_1 = 16i32;
                            s_i32_2 = 1116656i32;
                            s_i32_3 = 1116689i32;
                            s_i32_4 = l15;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 28usize, s_i32_2, s_i32_3, s_i32_4, host);
                            vm.exit_call();
                            let __t174 = __call_res?;
                            s_i32_2 = __t174;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                            vm.exit_call();
                            __call_res?;
                            s_i32_0 = l15;
                            return Ok(s_i32_0);
                        }
                    }
                }
            }
            s_i32_0 = l53;
            s_i32_1 = 1116658i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_191: {
                if s_i32_0 != 0 {
                    s_i32_0 = l42;
                    s_i32_1 = 8i32;
                    s_i32_0 = ops::i32_lt_u(s_i32_0, s_i32_1);
                    s_i32_1 = l42;
                    s_i32_2 = l43;
                    s_i32_1 = ops::i32_ne(s_i32_1, s_i32_2);
                    s_i32_0 = ops::i32_or(s_i32_0, s_i32_1);
                    'blk_192: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = vm.get_i32(*g_map.get(1usize).ok_or(WasmTrap::Unreachable)?)?;
                    s_i32_1 = l42;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 16usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t175 = __call_res?;
                    s_i32_0 = __t175;
                    l24 = s_i32_0;
                    s_i32_0 = l24;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                    s_i32_1 = 8i32;
                    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
                    'blk_193: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = l5;
                    s_i32_1 = 16i32;
                    s_i32_2 = 1116658i32;
                    s_i32_3 = 1116688i32;
                    s_i32_4 = l24;
                    s_i32_4 = ops::i32_load(vm, 0, s_i32_4, 4u32)?;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 28usize, s_i32_2, s_i32_3, s_i32_4, host);
                    vm.exit_call();
                    let __t176 = __call_res?;
                    s_i32_2 = __t176;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l42;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l53;
            s_i32_1 = 1116693i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_194: {
                if s_i32_0 != 0 {
                    s_i32_0 = l0;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 40u32)?;
                    s_i32_1 = l43;
                    s_i32_2 = 7i32;
                    s_i32_1 = ops::i32_ne(s_i32_1, s_i32_2);
                    s_i32_0 = ops::i32_or(s_i32_0, s_i32_1);
                    'blk_195: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = l5;
                    s_i32_1 = 16i32;
                    s_i32_2 = 1116693i32;
                    s_i32_3 = 1116689i32;
                    s_i32_4 = l42;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 28usize, s_i32_2, s_i32_3, s_i32_4, host);
                    vm.exit_call();
                    let __t177 = __call_res?;
                    s_i32_2 = __t177;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = 9i32;
                    s_i32_1 = l42;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 8usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t178 = __call_res?;
                    s_i32_0 = __t178;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l53;
            s_i32_1 = 1116694i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_196: {
                if s_i32_0 != 0 {
                    s_i32_0 = l0;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 40u32)?;
                    s_i32_1 = l42;
                    s_i32_2 = 8i32;
                    s_i32_1 = ops::i32_lt_u(s_i32_1, s_i32_2);
                    s_i32_0 = ops::i32_or(s_i32_0, s_i32_1);
                    'blk_197: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = vm.get_i32(*g_map.get(1usize).ok_or(WasmTrap::Unreachable)?)?;
                    s_i32_1 = l42;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 16usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t179 = __call_res?;
                    s_i32_0 = __t179;
                    l24 = s_i32_0;
                    s_i32_0 = l24;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                    s_i32_1 = 9i32;
                    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
                    s_i32_1 = l24;
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
                    s_i32_2 = l43;
                    s_i32_1 = ops::i32_ne(s_i32_1, s_i32_2);
                    s_i32_0 = ops::i32_or(s_i32_0, s_i32_1);
                    'blk_198: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = l5;
                    s_i32_1 = 16i32;
                    s_i32_2 = 1116694i32;
                    s_i32_3 = 1116689i32;
                    s_i32_4 = l43;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 28usize, s_i32_2, s_i32_3, s_i32_4, host);
                    vm.exit_call();
                    let __t180 = __call_res?;
                    s_i32_2 = __t180;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = 0i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l53;
            s_i32_1 = 1116695i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_199: {
                if s_i32_0 != 0 {
                    s_i32_0 = l0;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 40u32)?;
                    s_i32_1 = l42;
                    s_i32_2 = 8i32;
                    s_i32_1 = ops::i32_lt_u(s_i32_1, s_i32_2);
                    vm.deduct_1(host)?;
                    s_i32_0 = ops::i32_or(s_i32_0, s_i32_1);
                    'blk_200: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = vm.get_i32(*g_map.get(1usize).ok_or(WasmTrap::Unreachable)?)?;
                    s_i32_1 = l42;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 16usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t181 = __call_res?;
                    s_i32_0 = __t181;
                    l24 = s_i32_0;
                    s_i32_0 = l24;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                    s_i32_1 = 9i32;
                    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
                    'blk_201: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = l24;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
                    l15 = s_i32_0;
                    s_i32_0 = l43;
                    s_i32_1 = 8i32;
                    s_i32_2 = l15;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 8usize, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    let __t182 = __call_res?;
                    s_i32_1 = __t182;
                    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
                    'blk_202: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = l5;
                    s_i32_1 = 16i32;
                    s_i32_2 = 1116695i32;
                    s_i32_3 = 1116689i32;
                    s_i32_4 = l15;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 28usize, s_i32_2, s_i32_3, s_i32_4, host);
                    vm.exit_call();
                    let __t183 = __call_res?;
                    s_i32_2 = __t183;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = 0i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l53;
            s_i32_1 = 1116637i32;
            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
            s_i32_1 = l53;
            s_i32_2 = 1116640i32;
            vm.deduct_1(host)?;
            s_i32_1 = ops::i32_le_u(s_i32_1, s_i32_2);
            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
            'blk_203: {
                if s_i32_0 != 0 {
                    s_i32_0 = l43;
                    s_i32_1 = 5i32;
                    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
                    'blk_204: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = l5;
                    s_i32_1 = 16i32;
                    s_i32_2 = l53;
                    s_i32_3 = l42;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 42usize, s_i32_2, s_i32_3, host);
                    vm.exit_call();
                    let __t184 = __call_res?;
                    s_i32_2 = __t184;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l42;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l42;
            s_i32_1 = l43;
            s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
            'blk_205: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = l5;
            s_i32_1 = 16i32;
            s_i32_2 = l53;
            s_i32_3 = l42;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 42usize, s_i32_2, s_i32_3, host);
            vm.exit_call();
            let __t185 = __call_res?;
            s_i32_2 = __t185;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l53;
            s_i32_1 = 1116649i32;
            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
            s_i32_1 = l53;
            s_i32_2 = 1116652i32;
            s_i32_1 = ops::i32_le_u(s_i32_1, s_i32_2);
            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
            'blk_206: {
                if s_i32_0 != 0 {
                    s_i32_0 = 2i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l42;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l3;
    s_i32_1 = 27i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_207: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
            l53 = s_i32_0;
            s_i32_0 = l0;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 8u32)?;
            s_i32_2 = 0i32;
            vm.enter_call()?;
            let __call_res = Self::f_17fe7444ad16(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            let __t186 = __call_res?;
            s_i32_0 = __t186;
            l42 = s_i32_0;
            s_i32_0 = l42;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_208: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l5;
            s_i32_1 = l42;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_1, host);
            vm.exit_call();
            let __t187 = __call_res?;
            s_i32_1 = __t187;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t188 = __call_res?;
            s_i32_0 = __t188;
            l45 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = 33i32;
            s_i32_2 = l45;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l0;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 12u32)?;
            s_i32_2 = 0i32;
            vm.enter_call()?;
            let __call_res = Self::f_17fe7444ad16(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            let __t189 = __call_res?;
            s_i32_0 = __t189;
            l43 = s_i32_0;
            s_i32_0 = l43;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_209: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l43;
            s_i32_1 = 7i32;
            s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
            'blk_210: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = l5;
            s_i32_1 = 2228434i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t190 = __call_res?;
            s_i32_0 = __t190;
            l46 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = 33i32;
            s_i32_2 = l46;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l0;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 16u32)?;
            s_i32_2 = 0i32;
            vm.enter_call()?;
            let __call_res = Self::f_17fe7444ad16(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            let __t191 = __call_res?;
            s_i32_0 = __t191;
            l44 = s_i32_0;
            s_i32_0 = l44;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_211: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l5;
            s_i32_1 = l44;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_1, host);
            vm.exit_call();
            let __t192 = __call_res?;
            s_i32_1 = __t192;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t193 = __call_res?;
            s_i32_0 = __t193;
            l47 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = 33i32;
            s_i32_2 = l47;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 32i32;
            s_i32_2 = l45;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 32i32;
            s_i32_2 = l46;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l5;
            s_i32_1 = 32i32;
            s_i32_2 = l47;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l42;
            s_i32_1 = 8i32;
            s_i32_0 = ops::i32_lt_u(s_i32_0, s_i32_1);
            'blk_212: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = vm.get_i32(*g_map.get(1usize).ok_or(WasmTrap::Unreachable)?)?;
            s_i32_1 = l42;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 16usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t194 = __call_res?;
            s_i32_0 = __t194;
            l24 = s_i32_0;
            s_i32_0 = l53;
            vm.deduct_1(host)?;
            s_i32_1 = 1116657i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_213: {
                if s_i32_0 != 0 {
                    s_i32_0 = l24;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                    s_i32_1 = 8i32;
                    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
                    s_i32_1 = l44;
                    s_i32_2 = 7i32;
                    s_i32_1 = ops::i32_ne(s_i32_1, s_i32_2);
                    s_i32_0 = ops::i32_or(s_i32_0, s_i32_1);
                    'blk_214: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = l5;
                    s_i32_1 = 16i32;
                    s_i32_2 = 1116657i32;
                    s_i32_3 = 1116688i32;
                    s_i32_4 = l24;
                    s_i32_4 = ops::i32_load(vm, 0, s_i32_4, 4u32)?;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 28usize, s_i32_2, s_i32_3, s_i32_4, host);
                    vm.exit_call();
                    let __t195 = __call_res?;
                    s_i32_2 = __t195;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l42;
                    return Ok(s_i32_0);
                } else {
                    s_i32_0 = l0;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 40u32)?;
                    'blk_215: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = l24;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                    s_i32_1 = 9i32;
                    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
                    s_i32_1 = l24;
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
                    s_i32_2 = l44;
                    s_i32_1 = ops::i32_ne(s_i32_1, s_i32_2);
                    s_i32_0 = ops::i32_or(s_i32_0, s_i32_1);
                    'blk_216: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = l5;
                    s_i32_1 = 16i32;
                    s_i32_2 = 1116696i32;
                    s_i32_3 = 1116689i32;
                    s_i32_4 = l44;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 28usize, s_i32_2, s_i32_3, s_i32_4, host);
                    vm.exit_call();
                    let __t196 = __call_res?;
                    s_i32_2 = __t196;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = 0i32;
                    return Ok(s_i32_0);
                }
            }
        }
    }
    s_i32_0 = l3;
    s_i32_1 = 28i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_217: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
            s_i32_2 = 0i32;
            vm.enter_call()?;
            let __call_res = Self::f_17fe7444ad16(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            let __t197 = __call_res?;
            s_i32_0 = __t197;
            l42 = s_i32_0;
            s_i32_0 = l42;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_218: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l4;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 8u32)?;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 9usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t198 = __call_res?;
            s_i32_0 = __t198;
            l14 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = 16i32;
            s_i32_2 = 1116653i32;
            s_i32_3 = 3i32;
            s_i32_4 = 1114414i32;
            s_i32_5 = l42;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 44usize, s_i32_5, host);
            vm.exit_call();
            let __t199 = __call_res?;
            s_i32_5 = __t199;
            s_i32_6 = l14;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 44usize, s_i32_6, host);
            vm.exit_call();
            let __t200 = __call_res?;
            s_i32_6 = __t200;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_i32_i32_ret_i32(vm, c_map, 41usize, s_i32_2, s_i32_3, s_i32_4, s_i32_5, s_i32_6, host);
            vm.exit_call();
            let __t201 = __call_res?;
            s_i32_2 = __t201;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l14;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l3;
    s_i32_1 = 29i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_219: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
            s_i32_2 = 0i32;
            vm.enter_call()?;
            let __call_res = Self::f_17fe7444ad16(vm, dispatch, g_map, c_map, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            let __t202 = __call_res?;
            s_i32_0 = __t202;
            l42 = s_i32_0;
            s_i32_0 = l42;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_220: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l4;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 8u32)?;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 9usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t203 = __call_res?;
            s_i32_0 = __t203;
            l14 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = 16i32;
            s_i32_2 = 1116654i32;
            s_i32_3 = 3i32;
            s_i32_4 = 1114415i32;
            s_i32_5 = l42;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 44usize, s_i32_5, host);
            vm.exit_call();
            let __t204 = __call_res?;
            s_i32_5 = __t204;
            s_i32_6 = l14;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 44usize, s_i32_6, host);
            vm.exit_call();
            let __t205 = __call_res?;
            s_i32_6 = __t205;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_i32_i32_ret_i32(vm, c_map, 41usize, s_i32_2, s_i32_3, s_i32_4, s_i32_5, s_i32_6, host);
            vm.exit_call();
            let __t206 = __call_res?;
            s_i32_2 = __t206;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l14;
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
const FN_30_GLOBALS: &[usize] = &[4usize, 9usize, 8usize];
const FN_30_CALLEES: &[usize] = &[2usize, 3usize, 1usize, 1usize, 0usize, 5usize, 0usize, 0usize, 0usize, 4usize];
const FN_31_GLOBALS: &[usize] = &[11usize];
const FN_31_CALLEES: &[usize] = &[2usize, 0usize, 0usize];
const FN_32_GLOBALS: &[usize] = &[11usize];
const FN_32_CALLEES: &[usize] = &[6usize, 2usize, 0usize, 2usize];
const FN_33_CALLEES: &[usize] = &[6usize, 1usize, 4usize, 1usize, 0usize];
const FN_34_GLOBALS: &[usize] = &[12usize];
const FN_34_CALLEES: &[usize] = &[2usize, 0usize, 0usize];
const FN_35_GLOBALS: &[usize] = &[12usize];
const FN_35_CALLEES: &[usize] = &[7usize, 2usize, 0usize, 2usize];
const FN_36_GLOBALS: &[usize] = &[12usize];
const FN_36_CALLEES: &[usize] = &[0usize];
const FN_37_GLOBALS: &[usize] = &[10usize, 8usize, 4usize, 7usize, 9usize];
const FN_37_CALLEES: &[usize] = &[2usize, 0usize, 1usize, 2usize, 8usize, 2usize, 1usize, 3usize, 1usize, 0usize, 4usize, 0usize, 2usize, 0usize, 7usize, 2usize];
const FN_38_GLOBALS: &[usize] = &[13usize];
const FN_38_CALLEES: &[usize] = &[0usize, 0usize, 7usize];
const FN_39_GLOBALS: &[usize] = &[14usize];
const FN_39_CALLEES: &[usize] = &[0usize, 2usize];
const FN_42_CALLEES: &[usize] = &[4usize];
const FN_45_GLOBALS: &[usize] = &[14usize];
const FN_45_CALLEES: &[usize] = &[2usize, 0usize, 0usize, 2usize];
const FN_46_GLOBALS: &[usize] = &[14usize];
const FN_46_CALLEES: &[usize] = &[2usize, 0usize, 0usize, 2usize];
const FN_47_CALLEES: &[usize] = &[9usize];
const FN_48_GLOBALS: &[usize] = &[8usize];
const FN_48_CALLEES: &[usize] = &[1usize, 1usize, 1usize, 4usize];
const FN_49_GLOBALS: &[usize] = &[8usize];
const FN_49_CALLEES: &[usize] = &[1usize, 1usize, 1usize, 1usize, 4usize];
const FN_50_GLOBALS: &[usize] = &[6usize, 8usize, 5usize];
const FN_50_CALLEES: &[usize] = &[3usize, 1usize];
const FN_51_GLOBALS: &[usize] = &[8usize];
const FN_51_CALLEES: &[usize] = &[2usize, 3usize, 7usize];
const FN_52_GLOBALS: &[usize] = &[9usize];
const FN_52_CALLEES: &[usize] = &[2usize, 5usize, 7usize];
const FN_53_GLOBALS: &[usize] = &[2usize, 3usize, 9usize];
const FN_53_CALLEES: &[usize] = &[0usize, 5usize];
const FN_54_GLOBALS: &[usize] = &[9usize, 8usize];
const FN_54_CALLEES: &[usize] = &[3usize, 0usize, 1usize, 1usize, 2usize, 2usize, 2usize, 9usize];
const FN_55_CALLEES: &[usize] = &[1usize, 0usize, 0usize];
const FN_56_CALLEES: &[usize] = &[10usize, 1usize, 1usize, 1usize, 0usize];
const FN_57_CALLEES: &[usize] = &[1usize];
const FN_58_GLOBALS: &[usize] = &[8usize];
const FN_58_CALLEES: &[usize] = &[0usize, 3usize, 8usize, 11usize, 5usize, 1usize, 2usize, 2usize, 3usize, 15usize, 1usize];
const FN_61_GLOBALS: &[usize] = &[9usize, 8usize, 17usize];
const FN_61_CALLEES: &[usize] = &[3usize, 0usize, 12usize, 13usize, 11usize, 1usize, 2usize, 2usize, 1usize, 10usize, 9usize, 2usize, 5usize, 8usize, 14usize, 9usize, 10usize];
const FN_62_GLOBALS: &[usize] = &[8usize, 9usize, 15usize, 22usize];
const FN_62_CALLEES: &[usize] = &[1usize, 0usize, 1usize, 3usize, 6usize, 2usize, 9usize, 5usize, 2usize, 8usize, 11usize, 2usize, 15usize, 14usize];
const FN_63_CALLEES: &[usize] = &[1usize];
const FN_64_CALLEES: &[usize] = &[1usize];
const FN_65_GLOBALS: &[usize] = &[14usize, 15usize, 16usize];
const FN_65_CALLEES: &[usize] = &[1usize, 4usize, 0usize, 3usize, 3usize];
const FN_66_GLOBALS: &[usize] = &[16usize];
const FN_66_CALLEES: &[usize] = &[0usize, 0usize];
const FN_67_GLOBALS: &[usize] = &[16usize];
const FN_67_CALLEES: &[usize] = &[0usize, 0usize];
const FN_68_GLOBALS: &[usize] = &[16usize];
const FN_68_CALLEES: &[usize] = &[2usize, 0usize, 0usize];
const FN_69_GLOBALS: &[usize] = &[16usize, 15usize];
const FN_69_CALLEES: &[usize] = &[0usize, 2usize, 0usize];
const FN_70_GLOBALS: &[usize] = &[16usize, 15usize];
const FN_70_CALLEES: &[usize] = &[0usize, 2usize, 0usize];
const FN_71_GLOBALS: &[usize] = &[16usize];
const FN_71_CALLEES: &[usize] = &[3usize, 0usize, 2usize, 0usize, 2usize, 4usize];
const FN_72_GLOBALS: &[usize] = &[16usize];
const FN_72_CALLEES: &[usize] = &[4usize, 0usize, 2usize, 0usize, 2usize];
const FN_73_CALLEES: &[usize] = &[9usize, 0usize, 10usize, 3usize, 8usize, 4usize];
const FN_74_GLOBALS: &[usize] = &[14usize, 15usize];
const FN_74_CALLEES: &[usize] = &[0usize];
const FN_75_GLOBALS: &[usize] = &[14usize, 15usize];
const FN_75_CALLEES: &[usize] = &[0usize];
const FN_76_GLOBALS: &[usize] = &[14usize];
const FN_76_CALLEES: &[usize] = &[0usize, 12usize];
const FN_77_GLOBALS: &[usize] = &[14usize, 15usize];
const FN_77_CALLEES: &[usize] = &[0usize, 0usize, 4usize, 5usize];
const FN_78_CALLEES: &[usize] = &[2usize, 0usize];
const FN_79_GLOBALS: &[usize] = &[16usize];
const FN_79_CALLEES: &[usize] = &[4usize, 3usize, 0usize, 2usize, 0usize];
const FN_80_GLOBALS: &[usize] = &[16usize, 15usize, 8usize];
const FN_80_CALLEES: &[usize] = &[2usize, 0usize, 2usize, 15usize, 11usize, 0usize, 0usize, 13usize, 8usize, 2usize, 3usize, 12usize, 3usize, 1usize, 1usize, 14usize, 1usize, 4usize, 3usize, 1usize, 3usize, 5usize, 8usize];
const FN_81_GLOBALS: &[usize] = &[16usize];
const FN_81_CALLEES: &[usize] = &[0usize, 2usize, 0usize];
const FN_82_GLOBALS: &[usize] = &[16usize];
const FN_82_CALLEES: &[usize] = &[1usize, 0usize, 2usize, 0usize, 2usize];
const FN_83_CALLEES: &[usize] = &[2usize, 0usize];
const FN_84_GLOBALS: &[usize] = &[16usize];
const FN_84_CALLEES: &[usize] = &[0usize, 2usize, 0usize, 5usize];
const FN_85_GLOBALS: &[usize] = &[16usize];
const FN_85_CALLEES: &[usize] = &[0usize, 0usize, 2usize, 0usize, 2usize];
const FN_86_CALLEES: &[usize] = &[0usize, 1usize];
const FN_87_CALLEES: &[usize] = &[2usize, 0usize, 2usize];
const FN_88_CALLEES: &[usize] = &[2usize, 2usize];
const FN_89_CALLEES: &[usize] = &[4usize];
const FN_90_CALLEES: &[usize] = &[4usize, 0usize];
const FN_91_CALLEES: &[usize] = &[4usize];
const FN_92_CALLEES: &[usize] = &[4usize, 0usize];
const FN_93_CALLEES: &[usize] = &[4usize, 1usize];
const FN_94_CALLEES: &[usize] = &[4usize, 1usize];
const FN_95_CALLEES: &[usize] = &[4usize, 0usize];
const FN_96_CALLEES: &[usize] = &[4usize, 0usize];
const FN_97_GLOBALS: &[usize] = &[8usize];
const FN_97_CALLEES: &[usize] = &[2usize, 1usize, 4usize, 0usize, 0usize, 0usize];
const FN_98_GLOBALS: &[usize] = &[11usize, 12usize, 8usize];
const FN_98_CALLEES: &[usize] = &[1usize, 4usize, 2usize, 0usize, 0usize, 0usize];
const FN_99_GLOBALS: &[usize] = &[17usize, 18usize, 19usize, 20usize, 21usize, 22usize, 23usize, 24usize];
const FN_99_CALLEES: &[usize] = &[0usize, 1usize, 8usize];
const FN_100_GLOBALS: &[usize] = &[17usize, 18usize, 20usize, 19usize, 21usize, 8usize];
const FN_100_CALLEES: &[usize] = &[4usize, 13usize, 0usize, 5usize, 16usize, 5usize, 6usize, 6usize, 7usize, 0usize, 14usize, 2usize];
const FN_101_CALLEES: &[usize] = &[7usize, 0usize];
const FN_102_CALLEES: &[usize] = &[6usize, 7usize, 0usize];
const FN_103_CALLEES: &[usize] = &[0usize];
const FN_104_CALLEES: &[usize] = &[0usize];
const FN_105_GLOBALS: &[usize] = &[14usize];
const FN_105_CALLEES: &[usize] = &[12usize, 6usize, 17usize, 5usize, 6usize, 7usize, 0usize, 7usize, 0usize, 15usize];
const FN_106_GLOBALS: &[usize] = &[14usize, 8usize, 15usize];
const FN_106_CALLEES: &[usize] = &[12usize, 0usize, 1usize, 8usize, 13usize, 0usize, 5usize, 5usize, 6usize, 6usize, 7usize, 4usize, 5usize, 16usize, 0usize, 1usize, 8usize, 4usize, 14usize, 2usize];
const FN_107_GLOBALS: &[usize] = &[25usize];
const FN_108_CALLEES: &[usize] = &[2usize, 0usize, 0usize];
const FN_109_CALLEES: &[usize] = &[2usize, 0usize, 0usize];
const FN_110_CALLEES: &[usize] = &[2usize, 0usize, 0usize];
const FN_111_CALLEES: &[usize] = &[9usize, 0usize, 2usize];
const FN_112_CALLEES: &[usize] = &[2usize, 1usize, 0usize, 2usize];
const FN_113_CALLEES: &[usize] = &[9usize, 10usize, 2usize, 0usize, 16usize];
const FN_114_CALLEES: &[usize] = &[0usize, 2usize, 0usize];
const FN_115_CALLEES: &[usize] = &[16usize, 2usize, 0usize, 10usize, 1usize, 3usize, 0usize, 9usize, 8usize];
const FN_116_GLOBALS: &[usize] = &[14usize, 15usize];
const FN_116_CALLEES: &[usize] = &[9usize, 13usize, 2usize, 0usize, 0usize, 10usize, 4usize, 3usize, 8usize, 4usize, 7usize, 1usize, 8usize, 5usize, 2usize, 1usize, 2usize];
const FN_117_GLOBALS: &[usize] = &[17usize, 20usize];
const FN_117_CALLEES: &[usize] = &[5usize];
const FN_118_CALLEES: &[usize] = &[3usize, 5usize, 0usize, 6usize, 6usize];
const FN_119_CALLEES: &[usize] = &[5usize, 3usize, 0usize, 7usize];
const FN_120_GLOBALS: &[usize] = &[14usize];
const FN_120_CALLEES: &[usize] = &[4usize, 0usize, 4usize, 16usize, 7usize, 5usize, 2usize, 2usize];
const FN_121_GLOBALS: &[usize] = &[21usize, 12usize, 8usize];
const FN_121_CALLEES: &[usize] = &[2usize, 1usize, 0usize, 13usize, 3usize, 3usize, 9usize, 10usize, 2usize, 3usize, 8usize, 0usize, 0usize, 0usize, 11usize, 16usize, 7usize, 5usize, 1usize, 8usize, 12usize, 1usize, 13usize, 6usize];
const FN_122_GLOBALS: &[usize] = &[15usize, 14usize, 17usize, 20usize];
const FN_122_CALLEES: &[usize] = &[7usize, 3usize, 2usize, 4usize, 16usize, 5usize, 6usize, 0usize, 9usize, 13usize, 3usize, 12usize, 4usize, 5usize, 2usize, 1usize, 0usize, 2usize, 2usize, 0usize, 10usize, 7usize, 4usize, 3usize, 8usize, 4usize, 1usize, 5usize, 6usize, 7usize, 19usize, 1usize, 0usize, 4usize, 12usize, 18usize, 8usize, 5usize, 2usize, 6usize, 6usize, 0usize, 17usize, 6usize, 7usize, 15usize];
const FN_123_GLOBALS: &[usize] = &[17usize, 8usize, 22usize, 24usize, 20usize];
const FN_123_CALLEES: &[usize] = &[3usize, 13usize, 0usize, 5usize, 1usize, 2usize, 0usize, 1usize, 0usize, 2usize, 10usize, 1usize, 3usize, 5usize, 16usize, 17usize, 13usize, 6usize, 14usize, 2usize];
const FN_124_CALLEES: &[usize] = &[9usize, 13usize, 1usize, 5usize, 16usize, 2usize, 0usize, 10usize];
const FN_125_CALLEES: &[usize] = &[1usize, 1usize, 0usize, 17usize, 13usize, 5usize];
const FN_126_GLOBALS: &[usize] = &[22usize, 23usize, 17usize, 20usize, 16usize, 24usize, 8usize];
const FN_126_CALLEES: &[usize] = &[13usize, 5usize, 6usize, 6usize, 7usize, 16usize, 2usize, 0usize, 5usize, 14usize, 2usize];
const FN_127_GLOBALS: &[usize] = &[16usize];
const FN_127_CALLEES: &[usize] = &[2usize, 0usize, 1usize];
const FN_128_GLOBALS: &[usize] = &[16usize, 22usize, 24usize, 8usize];
const FN_128_CALLEES: &[usize] = &[0usize, 7usize, 2usize, 9usize, 0usize, 0usize, 8usize, 13usize, 5usize, 4usize, 16usize, 7usize, 0usize, 14usize, 2usize];
const FN_129_GLOBALS: &[usize] = &[1usize, 0usize, 2usize, 3usize, 10usize, 11usize, 12usize, 13usize, 14usize, 15usize, 16usize, 17usize, 22usize, 25usize, 8usize];
const FN_129_CALLEES: &[usize] = &[1usize, 2usize, 1usize, 0usize, 11usize, 2usize, 5usize, 4usize, 6usize, 7usize, 8usize, 9usize, 16usize];
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
    crate::Bound_p_i32_i32_i32_i32_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_517bbdea3a61, globals: FN_84_GLOBALS, callees: FN_84_CALLEES },
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
    crate::Bound_p_i32_i32_i32_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_ba80a369cdcb, globals: FN_32_GLOBALS, callees: FN_32_CALLEES },
    crate::Bound_p_i32_i32_i32_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_e778f0ae372c, globals: FN_81_GLOBALS, callees: FN_81_CALLEES },
    crate::Bound_p_i32_i32_i32_i32_i32_i32_ret_i32 { func: crate::c_85899_87792::ChunkFns::f_fb262af913b2, globals: FN_85_GLOBALS, callees: FN_85_CALLEES },
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
    crate::Bound_p_i32_i32_i32_i32_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_e3d239a807a4, globals: &[], callees: FN_115_CALLEES },
    crate::Bound_p_i32_i32_i32_i32_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_0eb69cfbacde, globals: FN_123_GLOBALS, callees: FN_123_CALLEES },
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
    crate::Bound_p_i32_i32_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_aab27a071d0d, globals: FN_37_GLOBALS, callees: FN_37_CALLEES },
    crate::Bound_p_i32_i32_i32_i32_i32_ret_i32 { func: crate::c_85899_87792::ChunkFns::f_78eba914826e, globals: FN_82_GLOBALS, callees: FN_82_CALLEES },
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
    crate::Bound_p_i32_i32_i32_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_07908887d2ca, globals: &[], callees: FN_96_CALLEES },
    crate::Bound_p_i32_i32_i32_i32_i32_ret_void { func: crate::c_85899_79292_1::ChunkFns::f_58deb2a85fe1, globals: &[], callees: FN_124_CALLEES },
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
    crate::Bound_p_i32_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_0ffb3bdf5187, globals: FN_24_GLOBALS, callees: FN_24_CALLEES },
    crate::Bound_p_i32_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_cbc237d053c1, globals: FN_30_GLOBALS, callees: FN_30_CALLEES },
    crate::Bound_p_i32_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_68ca86b09415, globals: FN_69_GLOBALS, callees: FN_69_CALLEES },
    crate::Bound_p_i32_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_152ec9a5d530, globals: FN_70_GLOBALS, callees: FN_70_CALLEES },
    crate::Bound_p_i32_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_d7490f9cf842, globals: &[], callees: FN_87_CALLEES },
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
    crate::Bound_p_i32_i32_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_5d566823fb6c, globals: &[], callees: FN_92_CALLEES },
    crate::Bound_p_i32_i32_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_293c7d635704, globals: &[], callees: FN_103_CALLEES },
    crate::Bound_p_i32_i32_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_ff67f9ea2c5b, globals: &[], callees: FN_104_CALLEES },
    crate::Bound_p_i32_i32_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_1dac86d11457, globals: &[], callees: FN_113_CALLEES },
    crate::Bound_p_i32_i32_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_62dc121ee7e1, globals: &[], callees: FN_119_CALLEES },
    crate::Bound_p_i32_i32_i32_i32_ret_void { func: crate::c_85899_79292_1::ChunkFns::f_23c7b6aa52a7, globals: &[], callees: FN_125_CALLEES },
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
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_31b8ad2e76d3, globals: &[], callees: FN_33_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_87792::ChunkFns::f_7e9d84c9daa2, globals: FN_66_GLOBALS, callees: FN_66_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_87792::ChunkFns::f_c8fc591b4686, globals: FN_71_GLOBALS, callees: FN_71_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_87792::ChunkFns::f_116a2da5e6bc, globals: FN_72_GLOBALS, callees: FN_72_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_87717d28a526, globals: &[], callees: FN_83_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_f5420555aefe, globals: &[], callees: FN_102_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_ed3ff760701d, globals: &[], callees: FN_108_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_004b66504437, globals: &[], callees: FN_109_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_81cc25430500, globals: &[], callees: FN_110_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_a98617ea6ec7, globals: &[], callees: FN_111_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_9a77c7cfc6c6, globals: &[], callees: FN_114_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_cbe6ccd58a27, globals: FN_121_GLOBALS, callees: FN_121_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_92692::ChunkFns::f_17fe7444ad16, globals: FN_122_GLOBALS, callees: FN_122_CALLEES },
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
    crate::Bound_p_i32_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_6569a18ff452, globals: FN_36_GLOBALS, callees: FN_36_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_c60b575467fb, globals: &[], callees: FN_63_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_aca97c761850, globals: &[], callees: FN_64_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_dc65747ad20e, globals: &[], callees: FN_90_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_a562805ce2e0, globals: &[], callees: FN_91_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_534b766828b2, globals: FN_120_GLOBALS, callees: FN_120_CALLEES },
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
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_da1c236549d3, globals: FN_27_GLOBALS, callees: FN_27_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_a657653b64d1, globals: FN_28_GLOBALS, callees: FN_28_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_f8be9dff3f3b, globals: FN_31_GLOBALS, callees: FN_31_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_f8be9dff3f3b, globals: FN_34_GLOBALS, callees: FN_34_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_b2d0372afbef, globals: FN_35_GLOBALS, callees: FN_35_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_98a40a4e126c, globals: FN_45_GLOBALS, callees: FN_45_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_44bf2eebebc5, globals: FN_46_GLOBALS, callees: FN_46_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_87792::ChunkFns::f_d36e1b31003e, globals: FN_67_GLOBALS, callees: FN_67_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_f80c06cb23fe, globals: FN_68_GLOBALS, callees: FN_68_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_ca9b528594d9, globals: &[], callees: FN_73_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_6abe7e1add88, globals: &[], callees: FN_78_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_6f29b496c61b, globals: FN_79_GLOBALS, callees: FN_79_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_2488f523e9ab, globals: &[], callees: FN_88_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_1632f6fcfe85, globals: &[], callees: FN_101_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_92692::ChunkFns::f_ed28f33ecf20, globals: FN_116_GLOBALS, callees: FN_116_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_8ef52de6b9da, globals: &[], callees: FN_118_CALLEES },
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
    crate::Bound_p_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_0c1b387702b2, globals: FN_65_GLOBALS, callees: FN_65_CALLEES },
    crate::Bound_p_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_76a01cf8b94d, globals: &[], callees: FN_89_CALLEES },
    crate::Bound_p_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_090940f804f9, globals: &[], callees: FN_93_CALLEES },
    crate::Bound_p_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_550472efdf87, globals: FN_105_GLOBALS, callees: FN_105_CALLEES },
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
    crate::Bound_p_i32_i64_ret_void { func: crate::c_85899_79292::ChunkFns::f_6ffdd7253944, globals: &[], callees: FN_94_CALLEES },
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
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_25abec857daa, globals: &[], callees: FN_42_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_934dd2b0a015, globals: &[], callees: &[] },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_23ce34ed2575, globals: &[], callees: &[] },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_948b576b5c16, globals: &[], callees: FN_47_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92692::ChunkFns::f_9401d1c479c0, globals: &[], callees: &[] },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_3a2d43ddb2f5, globals: &[], callees: &[] },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_7c241a508314, globals: FN_62_GLOBALS, callees: FN_62_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_621550e77e3b, globals: FN_75_GLOBALS, callees: FN_75_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_c9226330fb94, globals: &[], callees: FN_86_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_b7b055bd5416, globals: FN_97_GLOBALS, callees: FN_97_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_239cb2f96e82, globals: FN_106_GLOBALS, callees: FN_106_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_c0e8f275fac9, globals: &[], callees: FN_112_CALLEES },
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
    crate::Bound_p_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_44d81f681ce4, globals: &[], callees: FN_16_CALLEES },
    crate::Bound_p_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_2640236e5dc3, globals: &[], callees: FN_20_CALLEES },
    crate::Bound_p_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_60b618eb1778, globals: FN_74_GLOBALS, callees: FN_74_CALLEES },
    crate::Bound_p_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_a1d02c5fb9c2, globals: FN_76_GLOBALS, callees: FN_76_CALLEES },
    crate::Bound_p_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_717050e4ee19, globals: FN_77_GLOBALS, callees: FN_77_CALLEES },
    crate::Bound_p_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_29d13b4bd6bc, globals: &[], callees: FN_95_CALLEES },
    crate::Bound_p_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_9386bb430299, globals: FN_117_GLOBALS, callees: FN_117_CALLEES },
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
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_d7a26b86c8c3, globals: FN_38_GLOBALS, callees: FN_38_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_7734d017eaa1, globals: FN_48_GLOBALS, callees: FN_48_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_ed88189ac803, globals: FN_49_GLOBALS, callees: FN_49_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_4b4523d1e0e6, globals: FN_50_GLOBALS, callees: FN_50_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_a802fe84ff33, globals: FN_51_GLOBALS, callees: FN_51_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_8d9629285dfc, globals: FN_52_GLOBALS, callees: FN_52_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_2d0311fa7074, globals: FN_53_GLOBALS, callees: FN_53_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_c4aab8465645, globals: FN_54_GLOBALS, callees: FN_54_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_9d76726450cf, globals: &[], callees: FN_55_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_892d46fb695c, globals: &[], callees: FN_56_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_6154355b5577, globals: FN_58_GLOBALS, callees: FN_58_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_7308eeca408a, globals: FN_61_GLOBALS, callees: FN_61_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_af93137c5c01, globals: FN_98_GLOBALS, callees: FN_98_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_24fd9031937d, globals: FN_107_GLOBALS, callees: &[] },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_79292_1::ChunkFns::f_7ec9b65296e7, globals: FN_129_GLOBALS, callees: FN_129_CALLEES },
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
    crate::Bound_p_unit_ret_void { func: crate::c_85899_79292::ChunkFns::f_b2a9ea0711d2, globals: FN_39_GLOBALS, callees: FN_39_CALLEES },
    crate::Bound_p_unit_ret_void { func: crate::c_85899_79292::ChunkFns::f_8d58d78a7a3b, globals: &[], callees: FN_57_CALLEES },
    crate::Bound_p_unit_ret_void { func: crate::c_85899_87792::ChunkFns::f_fc806df2edb4, globals: FN_80_GLOBALS, callees: FN_80_CALLEES },
    crate::Bound_p_unit_ret_void { func: crate::c_85899_79292::ChunkFns::f_81a252fc3c25, globals: FN_99_GLOBALS, callees: FN_99_CALLEES },
    crate::Bound_p_unit_ret_void { func: crate::c_85899_79292::ChunkFns::f_3d58e151fe65, globals: FN_100_GLOBALS, callees: FN_100_CALLEES },
    crate::Bound_p_unit_ret_void { func: crate::c_85899_79292_1::ChunkFns::f_c71faaa3ebde, globals: FN_126_GLOBALS, callees: FN_126_CALLEES },
    crate::Bound_p_unit_ret_void { func: crate::c_85899_79292_1::ChunkFns::f_9809d582b386, globals: FN_127_GLOBALS, callees: FN_127_CALLEES },
    crate::Bound_p_unit_ret_void { func: crate::c_85899_79292_1::ChunkFns::f_6356e8ed2b9e, globals: FN_128_GLOBALS, callees: FN_128_CALLEES },
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
    sig_p_i32_i32_i32_i32_i32_i32_i32_ret_i32: BOUND_SIG_P_I32_I32_I32_I32_I32_I32_I32_RET_I32,
    sig_p_i32_i32_i32_i32_i32_i32_i32_ret_void: &[],
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
    (2229904u128, [0x1d, 0x3b, 0x24, 0x91, 0x72, 0xd4, 0x96, 0x6d, 0xf6, 0x44, 0x7d, 0x4b, 0x38, 0x92, 0xb3, 0x1b, 0x11, 0x42, 0x1b, 0x61, 0xd7, 0x6c, 0xa2, 0x16, 0x32, 0x44, 0x7c, 0xd7, 0x1f, 0xd3, 0x93, 0x28]),
];

const MODULE_META: SubstituteModuleMeta = SubstituteModuleMeta {
    name: "85899_92692/f1680@l8589992763",
    accounting_version: 2u32,
    wasm_sha256: [0x04, 0xf9, 0xe2, 0x2c, 0x7e, 0x5c, 0xc0, 0xfe, 0xc3, 0x12, 0xf2, 0x4c, 0x46, 0xe2, 0x0b, 0x9e, 0x79, 0x7e, 0xe9, 0xad, 0x2d, 0x62, 0x44, 0xdd, 0xf0, 0x45, 0x75, 0x9e, 0xb1, 0xc5, 0x31, 0x4f],
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
    vm.globals_i32 = vec![g0, g1, g2, g3, g4, g5, g6, g7, g8, g9, g10, g11, g12, g13, g14, g15, g16, g17, g18, g19, g20, g21, g22, g23, g24, g25];
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
        let res = DISPATCH.call_direct_p_unit_ret_i32(vm, 18usize, host);
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

