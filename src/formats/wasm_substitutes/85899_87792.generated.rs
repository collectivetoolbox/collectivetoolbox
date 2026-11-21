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

//! Generated native Rust WebAssembly substitute chunk `85899_87792`.
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
    pub(crate) fn f_7e9d84c9daa2(vm: &mut SubstituteVm, dispatch: &'static crate::ModuleDispatch, g_map: &'static [usize], c_map: &'static [u32], p0: i32, p1: i32, p2: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = p2;
    let mut l3: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i32_2: i32 = 0i32;
    let mut s_i32_3: i32 = 0i32;
    vm.deduct(host, 1)?;
    s_i32_0 = l2;
    s_i32_0 = ops::i32_eqz(s_i32_0);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
    s_i32_1 = l0;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l3 = s_i32_0;
    s_i32_0 = l3;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    s_i32_1 = l3;
    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
    s_i32_2 = l1;
    s_i32_3 = l2;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
    vm.exit_call();
    let __t1 = __call_res?;
    s_i32_0 = __t1;
    return Ok(s_i32_0);
    }

    pub(crate) fn f_d36e1b31003e(vm: &mut SubstituteVm, dispatch: &'static crate::ModuleDispatch, g_map: &'static [usize], c_map: &'static [u32], p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i32_2: i32 = 0i32;
    let mut s_i32_3: i32 = 0i32;
    vm.deduct(host, 1)?;
    s_i32_0 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
    s_i32_1 = l0;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l2 = s_i32_0;
    s_i32_0 = l2;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    s_i32_0 = ops::i32_eqz(s_i32_0);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
    s_i32_1 = l1;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t1 = __call_res?;
    s_i32_0 = __t1;
    l3 = s_i32_0;
    s_i32_0 = l2;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    s_i32_1 = l2;
    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
    s_i32_2 = l3;
    s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 0u32)?;
    s_i32_3 = l3;
    s_i32_3 = ops::i32_load(vm, 0, s_i32_3, 4u32)?;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
    vm.exit_call();
    let __t2 = __call_res?;
    s_i32_0 = __t2;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn f_c8fc591b4686(vm: &mut SubstituteVm, dispatch: &'static crate::ModuleDispatch, g_map: &'static [usize], c_map: &'static [u32], p0: i32, p1: i32, p2: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i32_2: i32 = 0i32;
    let mut s_i32_3: i32 = 0i32;
    vm.deduct(host, 1)?;
    s_i32_0 = l0;
    s_i32_1 = 0i32;
    s_i32_2 = l1;
    s_i32_3 = l2;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_i32(vm, c_map, 0usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l3 = s_i32_0;
    s_i32_0 = l3;
    s_i32_1 = 0i32;
    s_i32_0 = ops::i32_ge_s(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = l3;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
    s_i32_1 = l0;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t1 = __call_res?;
    s_i32_0 = __t1;
    l4 = s_i32_0;
    s_i32_0 = l4;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
    l5 = s_i32_0;
    s_i32_0 = l5;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 2usize, s_i32_0, host);
    vm.exit_call();
    let __t2 = __call_res?;
    s_i32_0 = __t2;
    l6 = s_i32_0;
    s_i32_0 = 0i32;
    l7 = s_i32_0;
    'blk_2: {
        'blk_3: loop {
            vm.deduct(host, 1)?;
            s_i32_0 = l7;
            s_i32_1 = l6;
            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
            if s_i32_0 != 0 {
                break 'blk_2;
            }
            s_i32_0 = l5;
            s_i32_1 = l7;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i32_0 = __t3;
            l8 = s_i32_0;
            s_i32_0 = l8;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 20u32)?;
            s_i32_1 = l8;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 24u32)?;
            s_i32_2 = l1;
            s_i32_3 = l2;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_i32(vm, c_map, 3usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
            vm.exit_call();
            let __t4 = __call_res?;
            s_i32_0 = __t4;
            'blk_4: {
                if s_i32_0 != 0 {
                    s_i32_0 = 0i32;
                    l9 = s_i32_0;
                    'blk_5: {
                        'blk_6: loop {
                            vm.deduct(host, 1)?;
                            s_i32_0 = l9;
                            s_i32_1 = l0;
                            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                            if s_i32_0 != 0 {
                                break 'blk_5;
                            }
                            s_i32_0 = l9;
                            s_i32_1 = l8;
                            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 0u32)?;
                            s_i32_2 = l8;
                            s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 4u32)?;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, s_i32_2, host);
                            vm.exit_call();
                            let __t5 = __call_res?;
                            s_i32_0 = __t5;
                            'blk_7: {
                                if s_i32_0 != 0 {
                                    s_i32_0 = l8;
                                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
                                    'blk_8: {
                                        if s_i32_0 != 0 {
                                            s_i32_0 = l9;
                                            s_i32_1 = 1i32;
                                            s_i32_2 = l8;
                                            s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 12u32)?;
                                            s_i32_3 = l8;
                                            s_i32_3 = ops::i32_load(vm, 0, s_i32_3, 16u32)?;
                                            vm.enter_call()?;
                                            let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_i32(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
                                            vm.exit_call();
                                            let __t6 = __call_res?;
                                            s_i32_0 = __t6;
                                        } else {
                                            s_i32_0 = l9;
                                            s_i32_1 = 1i32;
                                            s_i32_2 = l8;
                                            s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 12u32)?;
                                            s_i32_3 = l8;
                                            s_i32_3 = ops::i32_load(vm, 0, s_i32_3, 16u32)?;
                                            vm.enter_call()?;
                                            let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_i32(vm, c_map, 0usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
                                            vm.exit_call();
                                            let __t7 = __call_res?;
                                            s_i32_0 = __t7;
                                        }
                                    }
                                    l3 = s_i32_0;
                                    s_i32_0 = l3;
                                    s_i32_1 = 0i32;
                                    s_i32_0 = ops::i32_ge_s(s_i32_0, s_i32_1);
                                    'blk_9: {
                                        if s_i32_0 != 0 {
                                            s_i32_0 = l3;
                                            return Ok(s_i32_0);
                                        }
                                    }
                                }
                            }
                            s_i32_0 = l9;
                            s_i32_1 = 1i32;
                            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                            l9 = s_i32_0;
                            continue 'blk_6;
                            break 'blk_6;
                        }
                    }
                }
            }
            s_i32_0 = l7;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l7 = s_i32_0;
            continue 'blk_3;
            break 'blk_3;
        }
    }
    s_i32_0 = -1i32;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn f_116a2da5e6bc(vm: &mut SubstituteVm, dispatch: &'static crate::ModuleDispatch, g_map: &'static [usize], c_map: &'static [u32], p0: i32, p1: i32, p2: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i32_2: i32 = 0i32;
    let mut s_i32_3: i32 = 0i32;
    let mut s_i32_4: i32 = 0i32;
    vm.deduct(host, 1)?;
    s_i32_0 = l0;
    s_i32_1 = 0i32;
    s_i32_2 = l1;
    s_i32_3 = l2;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_i32(vm, c_map, 0usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l3 = s_i32_0;
    s_i32_0 = l3;
    s_i32_1 = 0i32;
    s_i32_0 = ops::i32_ge_s(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = l3;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
    s_i32_1 = l0;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t1 = __call_res?;
    s_i32_0 = __t1;
    l4 = s_i32_0;
    s_i32_0 = l4;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
    l5 = s_i32_0;
    s_i32_0 = l5;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 2usize, s_i32_0, host);
    vm.exit_call();
    let __t2 = __call_res?;
    s_i32_0 = __t2;
    l6 = s_i32_0;
    s_i32_0 = 0i32;
    l7 = s_i32_0;
    'blk_2: {
        'blk_3: loop {
            vm.deduct(host, 1)?;
            s_i32_0 = l7;
            s_i32_1 = l6;
            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
            if s_i32_0 != 0 {
                break 'blk_2;
            }
            s_i32_0 = l5;
            s_i32_1 = l7;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i32_0 = __t3;
            l8 = s_i32_0;
            s_i32_0 = l8;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
            s_i32_1 = l8;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 12u32)?;
            s_i32_2 = l8;
            s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 16u32)?;
            s_i32_3 = l1;
            s_i32_4 = l2;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_i32(vm, c_map, 3usize, s_i32_1, s_i32_2, s_i32_3, s_i32_4, host);
            vm.exit_call();
            let __t4 = __call_res?;
            s_i32_1 = __t4;
            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
            'blk_4: {
                if s_i32_0 != 0 {
                    s_i32_0 = 0i32;
                    l9 = s_i32_0;
                    'blk_5: {
                        'blk_6: loop {
                            vm.deduct(host, 1)?;
                            s_i32_0 = l9;
                            s_i32_1 = l0;
                            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                            if s_i32_0 != 0 {
                                break 'blk_5;
                            }
                            s_i32_0 = l9;
                            s_i32_1 = l8;
                            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 0u32)?;
                            s_i32_2 = l8;
                            s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 4u32)?;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, s_i32_2, host);
                            vm.exit_call();
                            let __t5 = __call_res?;
                            s_i32_0 = __t5;
                            'blk_7: {
                                if s_i32_0 != 0 {
                                    s_i32_0 = l9;
                                    s_i32_1 = 1i32;
                                    s_i32_2 = l1;
                                    s_i32_3 = l2;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_i32(vm, c_map, 0usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
                                    vm.exit_call();
                                    let __t6 = __call_res?;
                                    s_i32_0 = __t6;
                                    l3 = s_i32_0;
                                    s_i32_0 = l3;
                                    s_i32_1 = 0i32;
                                    s_i32_0 = ops::i32_ge_s(s_i32_0, s_i32_1);
                                    'blk_8: {
                                        if s_i32_0 != 0 {
                                            s_i32_0 = l3;
                                            return Ok(s_i32_0);
                                        }
                                    }
                                }
                            }
                            s_i32_0 = l9;
                            s_i32_1 = 1i32;
                            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                            l9 = s_i32_0;
                            continue 'blk_6;
                            break 'blk_6;
                        }
                    }
                }
            }
            s_i32_0 = l7;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l7 = s_i32_0;
            continue 'blk_3;
            break 'blk_3;
        }
    }
    s_i32_0 = 0i32;
    l9 = s_i32_0;
    'blk_9: {
        'blk_10: loop {
            vm.deduct(host, 1)?;
            s_i32_0 = l9;
            s_i32_1 = l0;
            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
            if s_i32_0 != 0 {
                break 'blk_9;
            }
            s_i32_0 = l9;
            s_i32_1 = 1i32;
            s_i32_2 = l1;
            s_i32_3 = l2;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_i32(vm, c_map, 0usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
            vm.exit_call();
            let __t7 = __call_res?;
            s_i32_0 = __t7;
            l3 = s_i32_0;
            s_i32_0 = l3;
            s_i32_1 = 0i32;
            s_i32_0 = ops::i32_ge_s(s_i32_0, s_i32_1);
            'blk_11: {
                if s_i32_0 != 0 {
                    s_i32_0 = l3;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l9;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l9 = s_i32_0;
            continue 'blk_10;
            break 'blk_10;
        }
    }
    s_i32_0 = -1i32;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn f_fc806df2edb4(vm: &mut SubstituteVm, dispatch: &'static crate::ModuleDispatch, g_map: &'static [usize], c_map: &'static [u32], host: &mut dyn WasmSubstituteHost) -> std::result::Result<(), WasmTrap> {
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
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i32_2: i32 = 0i32;
    let mut s_i32_3: i32 = 0i32;
    let mut s_i32_4: i32 = 0i32;
    vm.deduct(host, 1)?;
    s_i32_0 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 0usize, s_i32_0, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l0 = s_i32_0;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_eqz(s_i32_0);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = 0i32;
    l1 = s_i32_0;
    'blk_2: {
        'blk_3: loop {
            vm.deduct(host, 1)?;
            s_i32_0 = l1;
            s_i32_1 = l0;
            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
            if s_i32_0 != 0 {
                break 'blk_2;
            }
            s_i32_0 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
            s_i32_1 = l1;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            l3 = s_i32_0;
            s_i32_0 = l3;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
            l15 = s_i32_0;
            s_i32_0 = l15;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 0usize, s_i32_0, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_0 = __t2;
            l16 = s_i32_0;
            s_i32_0 = 0i32;
            l17 = s_i32_0;
            'blk_4: {
                'blk_5: loop {
                    vm.deduct(host, 1)?;
                    s_i32_0 = l17;
                    s_i32_1 = l16;
                    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                    if s_i32_0 != 0 {
                        break 'blk_4;
                    }
                    s_i32_0 = l15;
                    s_i32_1 = l17;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t3 = __call_res?;
                    s_i32_0 = __t3;
                    l18 = s_i32_0;
                    s_i32_0 = 0i32;
                    l43 = s_i32_0;
                    s_i32_0 = 0i32;
                    l19 = s_i32_0;
                    'blk_6: {
                        'blk_7: loop {
                            vm.deduct(host, 1)?;
                            s_i32_0 = l19;
                            s_i32_1 = l1;
                            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                            if s_i32_0 != 0 {
                                break 'blk_6;
                            }
                            s_i32_0 = l19;
                            s_i32_1 = l18;
                            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 0u32)?;
                            s_i32_2 = l18;
                            s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 4u32)?;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 2usize, s_i32_0, s_i32_1, s_i32_2, host);
                            vm.exit_call();
                            let __t4 = __call_res?;
                            s_i32_0 = __t4;
                            'blk_8: {
                                if s_i32_0 != 0 {
                                    s_i32_0 = l19;
                                    s_i32_1 = l18;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 3usize, s_i32_0, s_i32_1, host);
                                    vm.exit_call();
                                    let __t5 = __call_res?;
                                    s_i32_0 = __t5;
                                    'blk_9: {
                                        if s_i32_0 != 0 {
                                            s_i32_0 = 1i32;
                                            l43 = s_i32_0;
                                            break 'blk_6;
                                        }
                                    }
                                }
                            }
                            s_i32_0 = l19;
                            s_i32_1 = 1i32;
                            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                            l19 = s_i32_0;
                            continue 'blk_7;
                            break 'blk_7;
                        }
                    }
                    s_i32_0 = l43;
                    s_i32_0 = ops::i32_eqz(s_i32_0);
                    'blk_10: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = l17;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    l17 = s_i32_0;
                    continue 'blk_5;
                    break 'blk_5;
                }
            }
            s_i32_0 = l1;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l1 = s_i32_0;
            continue 'blk_3;
            break 'blk_3;
        }
    }
    s_i32_0 = vm.get_i32(*g_map.get(1usize).ok_or(WasmTrap::Unreachable)?)?;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 0usize, s_i32_0, host);
    vm.exit_call();
    let __t6 = __call_res?;
    s_i32_0 = __t6;
    l5 = s_i32_0;
    s_i32_0 = 0i32;
    l6 = s_i32_0;
    'blk_11: {
        'blk_12: loop {
            vm.deduct(host, 1)?;
            s_i32_0 = l6;
            s_i32_1 = l5;
            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
            if s_i32_0 != 0 {
                break 'blk_11;
            }
            s_i32_0 = vm.get_i32(*g_map.get(1usize).ok_or(WasmTrap::Unreachable)?)?;
            s_i32_1 = l6;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t7 = __call_res?;
            s_i32_0 = __t7;
            l8 = s_i32_0;
            s_i32_0 = l6;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l7 = s_i32_0;
            'blk_13: {
                'blk_14: loop {
                    vm.deduct(host, 1)?;
                    s_i32_0 = l7;
                    s_i32_1 = l5;
                    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                    if s_i32_0 != 0 {
                        break 'blk_13;
                    }
                    s_i32_0 = vm.get_i32(*g_map.get(1usize).ok_or(WasmTrap::Unreachable)?)?;
                    s_i32_1 = l7;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t8 = __call_res?;
                    s_i32_0 = __t8;
                    l9 = s_i32_0;
                    s_i32_0 = l8;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                    s_i32_1 = l9;
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 0u32)?;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                    s_i32_1 = l8;
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
                    s_i32_2 = l9;
                    s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 4u32)?;
                    s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
                    s_i32_2 = l8;
                    s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 0u32)?;
                    s_i32_3 = l9;
                    s_i32_3 = ops::i32_load(vm, 0, s_i32_3, 0u32)?;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_2, s_i32_3, host);
                    vm.exit_call();
                    let __t9 = __call_res?;
                    s_i32_2 = __t9;
                    s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
                    s_i32_0 = ops::i32_or(s_i32_0, s_i32_1);
                    'blk_15: {
                        if s_i32_0 != 0 {
                            s_i32_0 = l8;
                            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 16u32)?;
                            s_i32_1 = l8;
                            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 20u32)?;
                            s_i32_2 = l9;
                            s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 16u32)?;
                            s_i32_3 = l9;
                            s_i32_3 = ops::i32_load(vm, 0, s_i32_3, 20u32)?;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_i32(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
                            vm.exit_call();
                            let __t10 = __call_res?;
                            s_i32_0 = __t10;
                            'blk_16: {
                                if s_i32_0 != 0 {
                                    return Err(WasmTrap::Unreachable);
                                }
                            }
                        }
                    }
                    s_i32_0 = l7;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    l7 = s_i32_0;
                    continue 'blk_14;
                    break 'blk_14;
                }
            }
            s_i32_0 = l8;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 24u32)?;
            l10 = s_i32_0;
            s_i32_0 = l10;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 0usize, s_i32_0, host);
            vm.exit_call();
            let __t11 = __call_res?;
            s_i32_0 = __t11;
            l11 = s_i32_0;
            s_i32_0 = l11;
            s_i32_1 = 2i32;
            s_i32_0 = ops::i32_shl(s_i32_0, s_i32_1);
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 6usize, s_i32_0, host);
            vm.exit_call();
            let __t12 = __call_res?;
            s_i32_0 = __t12;
            l12 = s_i32_0;
            s_i32_0 = 0i32;
            l13 = s_i32_0;
            'blk_17: {
                'blk_18: loop {
                    vm.deduct(host, 1)?;
                    s_i32_0 = l13;
                    s_i32_1 = l11;
                    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                    if s_i32_0 != 0 {
                        break 'blk_17;
                    }
                    s_i32_0 = l8;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                    s_i32_1 = l10;
                    s_i32_2 = l13;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    let __t13 = __call_res?;
                    s_i32_1 = __t13;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 7usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t14 = __call_res?;
                    s_i32_0 = __t14;
                    l14 = s_i32_0;
                    s_i32_0 = l14;
                    s_i32_1 = 0i32;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                    s_i32_1 = l14;
                    s_i32_2 = 1i32;
                    s_i32_1 = ops::i32_eq(s_i32_1, s_i32_2);
                    s_i32_0 = ops::i32_or(s_i32_0, s_i32_1);
                    'blk_19: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = l12;
                    s_i32_1 = l13;
                    s_i32_2 = 2i32;
                    s_i32_1 = ops::i32_shl(s_i32_1, s_i32_2);
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    s_i32_1 = l14;
                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l13;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    l13 = s_i32_0;
                    continue 'blk_18;
                    break 'blk_18;
                }
            }
            s_i32_0 = l8;
            s_i32_1 = l12;
            ops::i32_store(vm, 0, s_i32_0, 28u32, s_i32_1)?;
            s_i32_0 = l8;
            s_i32_1 = l11;
            ops::i32_store(vm, 0, s_i32_0, 32u32, s_i32_1)?;
            s_i32_0 = l6;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l6 = s_i32_0;
            continue 'blk_12;
            break 'blk_12;
        }
    }
    s_i32_0 = 0i32;
    l6 = s_i32_0;
    'blk_20: {
        'blk_21: loop {
            vm.deduct(host, 1)?;
            s_i32_0 = l6;
            s_i32_1 = l5;
            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
            if s_i32_0 != 0 {
                break 'blk_20;
            }
            s_i32_0 = l6;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 8usize, s_i32_0, host);
            vm.exit_call();
            let __t15 = __call_res?;
            s_i32_0 = __t15;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_void(vm, c_map, 9usize, s_i32_0, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = vm.get_i32(*g_map.get(1usize).ok_or(WasmTrap::Unreachable)?)?;
            s_i32_1 = l6;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t16 = __call_res?;
            s_i32_0 = __t16;
            l8 = s_i32_0;
            s_i32_0 = l8;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 32u32)?;
            l11 = s_i32_0;
            s_i32_0 = l8;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 28u32)?;
            l12 = s_i32_0;
            s_i32_0 = 0i32;
            l13 = s_i32_0;
            'blk_22: {
                'blk_23: loop {
                    vm.deduct(host, 1)?;
                    s_i32_0 = l13;
                    s_i32_1 = l11;
                    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                    if s_i32_0 != 0 {
                        break 'blk_22;
                    }
                    s_i32_0 = l12;
                    s_i32_1 = l13;
                    s_i32_2 = 2i32;
                    s_i32_1 = ops::i32_shl(s_i32_1, s_i32_2);
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                    l14 = s_i32_0;
                    s_i32_0 = l14;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_void(vm, c_map, 10usize, s_i32_0, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l8;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
                    s_i32_1 = l14;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 11usize, s_i32_1, host);
                    vm.exit_call();
                    let __t17 = __call_res?;
                    s_i32_1 = __t17;
                    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                    'blk_24: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = l13;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    l13 = s_i32_0;
                    continue 'blk_23;
                    break 'blk_23;
                }
            }
            s_i32_0 = l6;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l6 = s_i32_0;
            continue 'blk_21;
            break 'blk_21;
        }
    }
    s_i32_0 = 0i32;
    l28 = s_i32_0;
    s_i32_0 = 0i32;
    l1 = s_i32_0;
    'blk_25: {
        'blk_26: loop {
            vm.deduct(host, 1)?;
            s_i32_0 = l1;
            s_i32_1 = l0;
            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
            if s_i32_0 != 0 {
                break 'blk_25;
            }
            s_i32_0 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
            s_i32_1 = l1;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t18 = __call_res?;
            s_i32_0 = __t18;
            l3 = s_i32_0;
            s_i32_0 = l3;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 16u32)?;
            l20 = s_i32_0;
            s_i32_0 = l20;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 0usize, s_i32_0, host);
            vm.exit_call();
            let __t19 = __call_res?;
            s_i32_0 = __t19;
            l21 = s_i32_0;
            s_i32_0 = 0i32;
            l22 = s_i32_0;
            'blk_27: {
                'blk_28: loop {
                    vm.deduct(host, 1)?;
                    s_i32_0 = l22;
                    s_i32_1 = l21;
                    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                    if s_i32_0 != 0 {
                        break 'blk_27;
                    }
                    s_i32_0 = l20;
                    s_i32_1 = l22;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t20 = __call_res?;
                    s_i32_0 = __t20;
                    l24 = s_i32_0;
                    s_i32_0 = l1;
                    s_i32_1 = l24;
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 12u32)?;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 7usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t21 = __call_res?;
                    s_i32_0 = __t21;
                    l26 = s_i32_0;
                    s_i32_0 = l26;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_void(vm, c_map, 10usize, s_i32_0, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l26;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 11usize, s_i32_0, host);
                    vm.exit_call();
                    let __t22 = __call_res?;
                    s_i32_0 = __t22;
                    'blk_29: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = l26;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 12usize, s_i32_0, host);
                    vm.exit_call();
                    let __t23 = __call_res?;
                    s_i32_0 = __t23;
                    l27 = s_i32_0;
                    s_i32_0 = l27;
                    s_i32_0 = ops::i32_eqz(s_i32_0);
                    'blk_30: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = l24;
                    s_i32_1 = l26;
                    ops::i32_store(vm, 0, s_i32_0, 28u32, s_i32_1)?;
                    s_i32_0 = 0i32;
                    l23 = s_i32_0;
                    'blk_31: {
                        'blk_32: loop {
                            vm.deduct(host, 1)?;
                            s_i32_0 = l23;
                            s_i32_1 = l22;
                            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                            if s_i32_0 != 0 {
                                break 'blk_31;
                            }
                            s_i32_0 = l20;
                            s_i32_1 = l23;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_0, s_i32_1, host);
                            vm.exit_call();
                            let __t24 = __call_res?;
                            s_i32_0 = __t24;
                            l25 = s_i32_0;
                            s_i32_0 = l25;
                            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 28u32)?;
                            s_i32_1 = l26;
                            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                            s_i32_1 = l24;
                            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 16u32)?;
                            s_i32_2 = l24;
                            s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 20u32)?;
                            s_i32_3 = l25;
                            s_i32_3 = ops::i32_load(vm, 0, s_i32_3, 16u32)?;
                            s_i32_4 = l25;
                            s_i32_4 = ops::i32_load(vm, 0, s_i32_4, 20u32)?;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_i32(vm, c_map, 5usize, s_i32_1, s_i32_2, s_i32_3, s_i32_4, host);
                            vm.exit_call();
                            let __t25 = __call_res?;
                            s_i32_1 = __t25;
                            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                            'blk_33: {
                                if s_i32_0 != 0 {
                                    return Err(WasmTrap::Unreachable);
                                }
                            }
                            s_i32_0 = l23;
                            s_i32_1 = 1i32;
                            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                            l23 = s_i32_0;
                            continue 'blk_32;
                            break 'blk_32;
                        }
                    }
                    s_i32_0 = 99i32;
                    s_i32_1 = l28;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 13usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t26 = __call_res?;
                    s_i32_0 = __t26;
                    l29 = s_i32_0;
                    s_i32_0 = l28;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    l28 = s_i32_0;
                    s_i32_0 = l24;
                    s_i32_1 = l29;
                    s_i32_2 = vm.get_i32(*g_map.get(2usize).ok_or(WasmTrap::Unreachable)?)?;
                    s_i32_3 = l27;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 14usize, s_i32_1, s_i32_2, s_i32_3, host);
                    vm.exit_call();
                    let __t27 = __call_res?;
                    s_i32_1 = __t27;
                    ops::i32_store(vm, 0, s_i32_0, 32u32, s_i32_1)?;
                    s_i32_0 = l22;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    l22 = s_i32_0;
                    continue 'blk_28;
                    break 'blk_28;
                }
            }
            s_i32_0 = l3;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 20u32)?;
            l30 = s_i32_0;
            s_i32_0 = l30;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 0usize, s_i32_0, host);
            vm.exit_call();
            let __t28 = __call_res?;
            s_i32_0 = __t28;
            l31 = s_i32_0;
            s_i32_0 = 0i32;
            l32 = s_i32_0;
            'blk_34: {
                'blk_35: loop {
                    vm.deduct(host, 1)?;
                    s_i32_0 = l32;
                    s_i32_1 = l31;
                    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                    if s_i32_0 != 0 {
                        break 'blk_34;
                    }
                    s_i32_0 = l30;
                    s_i32_1 = l32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t29 = __call_res?;
                    s_i32_0 = __t29;
                    l34 = s_i32_0;
                    s_i32_0 = l1;
                    s_i32_1 = l34;
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 12u32)?;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 7usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t30 = __call_res?;
                    s_i32_0 = __t30;
                    l26 = s_i32_0;
                    s_i32_0 = l26;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_void(vm, c_map, 10usize, s_i32_0, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l34;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                    s_i32_1 = l26;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 11usize, s_i32_1, host);
                    vm.exit_call();
                    let __t31 = __call_res?;
                    s_i32_1 = __t31;
                    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                    'blk_36: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = l34;
                    s_i32_1 = l26;
                    ops::i32_store(vm, 0, s_i32_0, 32u32, s_i32_1)?;
                    s_i32_0 = l34;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 24u32)?;
                    l36 = s_i32_0;
                    s_i32_0 = l36;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 0usize, s_i32_0, host);
                    vm.exit_call();
                    let __t32 = __call_res?;
                    s_i32_0 = __t32;
                    l37 = s_i32_0;
                    s_i32_0 = 0i32;
                    l38 = s_i32_0;
                    'blk_37: {
                        'blk_38: loop {
                            vm.deduct(host, 1)?;
                            s_i32_0 = l38;
                            s_i32_1 = l37;
                            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                            if s_i32_0 != 0 {
                                break 'blk_37;
                            }
                            s_i32_0 = l36;
                            s_i32_1 = l38;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_0, s_i32_1, host);
                            vm.exit_call();
                            let __t33 = __call_res?;
                            s_i32_0 = __t33;
                            l39 = s_i32_0;
                            s_i32_0 = l1;
                            s_i32_1 = l39;
                            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 0u32)?;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 7usize, s_i32_0, s_i32_1, host);
                            vm.exit_call();
                            let __t34 = __call_res?;
                            s_i32_0 = __t34;
                            l40 = s_i32_0;
                            s_i32_0 = l40;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_ret_void(vm, c_map, 10usize, s_i32_0, host);
                            vm.exit_call();
                            __call_res?;
                            s_i32_0 = l40;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 12usize, s_i32_0, host);
                            vm.exit_call();
                            let __t35 = __call_res?;
                            s_i32_0 = __t35;
                            s_i32_0 = ops::i32_eqz(s_i32_0);
                            'blk_39: {
                                if s_i32_0 != 0 {
                                    return Err(WasmTrap::Unreachable);
                                }
                            }
                            s_i32_0 = l34;
                            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                            s_i32_1 = l40;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 11usize, s_i32_1, host);
                            vm.exit_call();
                            let __t36 = __call_res?;
                            s_i32_1 = __t36;
                            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                            'blk_40: {
                                if s_i32_0 != 0 {
                                    return Err(WasmTrap::Unreachable);
                                }
                            }
                            s_i32_0 = l39;
                            s_i32_1 = l40;
                            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                            s_i32_0 = l38;
                            s_i32_1 = 1i32;
                            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                            l38 = s_i32_0;
                            continue 'blk_38;
                            break 'blk_38;
                        }
                    }
                    s_i32_0 = l32;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    l32 = s_i32_0;
                    continue 'blk_35;
                    break 'blk_35;
                }
            }
            s_i32_0 = 0i32;
            l32 = s_i32_0;
            'blk_41: {
                'blk_42: loop {
                    vm.deduct(host, 1)?;
                    s_i32_0 = l32;
                    s_i32_1 = l31;
                    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                    if s_i32_0 != 0 {
                        break 'blk_41;
                    }
                    s_i32_0 = l30;
                    s_i32_1 = l32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t37 = __call_res?;
                    s_i32_0 = __t37;
                    l34 = s_i32_0;
                    s_i32_0 = 0i32;
                    l41 = s_i32_0;
                    s_i32_0 = 0i32;
                    l33 = s_i32_0;
                    'blk_43: {
                        'blk_44: loop {
                            vm.deduct(host, 1)?;
                            s_i32_0 = l33;
                            s_i32_1 = l31;
                            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                            if s_i32_0 != 0 {
                                break 'blk_43;
                            }
                            s_i32_0 = l30;
                            s_i32_1 = l33;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_0, s_i32_1, host);
                            vm.exit_call();
                            let __t38 = __call_res?;
                            s_i32_0 = __t38;
                            l35 = s_i32_0;
                            s_i32_0 = l34;
                            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 16u32)?;
                            s_i32_1 = l34;
                            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 20u32)?;
                            s_i32_2 = l35;
                            s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 16u32)?;
                            s_i32_3 = l35;
                            s_i32_3 = ops::i32_load(vm, 0, s_i32_3, 20u32)?;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_i32(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
                            vm.exit_call();
                            let __t39 = __call_res?;
                            s_i32_0 = __t39;
                            'blk_45: {
                                if s_i32_0 != 0 {
                                    s_i32_0 = l41;
                                    s_i32_1 = 1i32;
                                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                                    l41 = s_i32_0;
                                    s_i32_0 = l32;
                                    s_i32_1 = l33;
                                    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
                                    s_i32_1 = l34;
                                    s_i32_2 = l35;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 15usize, s_i32_1, s_i32_2, host);
                                    vm.exit_call();
                                    let __t40 = __call_res?;
                                    s_i32_1 = __t40;
                                    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                                    'blk_46: {
                                        if s_i32_0 != 0 {
                                            return Err(WasmTrap::Unreachable);
                                        }
                                    }
                                }
                            }
                            s_i32_0 = l33;
                            s_i32_1 = 1i32;
                            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                            l33 = s_i32_0;
                            continue 'blk_44;
                            break 'blk_44;
                        }
                    }
                    s_i32_0 = 48i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 16usize, s_i32_0, host);
                    vm.exit_call();
                    let __t41 = __call_res?;
                    s_i32_0 = __t41;
                    l42 = s_i32_0;
                    s_i32_0 = l42;
                    s_i32_1 = 4i32;
                    s_i32_2 = 1717529715i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 17usize, s_i32_0, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l42;
                    s_i32_1 = 3i32;
                    s_i32_2 = 7167854i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 17usize, s_i32_0, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l42;
                    s_i32_1 = l1;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 18usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l3;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
                    s_i32_1 = 0i32;
                    s_i32_0 = ops::i32_gt_u(s_i32_0, s_i32_1);
                    'blk_47: {
                        if s_i32_0 != 0 {
                            s_i32_0 = l42;
                            s_i32_1 = 95i32;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 19usize, s_i32_0, s_i32_1, host);
                            vm.exit_call();
                            __call_res?;
                            s_i32_0 = l42;
                            s_i32_1 = l3;
                            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 0u32)?;
                            s_i32_2 = l3;
                            s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 4u32)?;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 20usize, s_i32_0, s_i32_1, s_i32_2, host);
                            vm.exit_call();
                            __call_res?;
                        }
                    }
                    s_i32_0 = l42;
                    s_i32_1 = 95i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 19usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l42;
                    s_i32_1 = l34;
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 16u32)?;
                    s_i32_2 = l34;
                    s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 20u32)?;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 20usize, s_i32_0, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l41;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_gt_u(s_i32_0, s_i32_1);
                    'blk_48: {
                        if s_i32_0 != 0 {
                            s_i32_0 = l42;
                            s_i32_1 = 2i32;
                            s_i32_2 = 24415i32;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 17usize, s_i32_0, s_i32_1, s_i32_2, host);
                            vm.exit_call();
                            __call_res?;
                            s_i32_0 = l34;
                            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 24u32)?;
                            l36 = s_i32_0;
                            s_i32_0 = l36;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 0usize, s_i32_0, host);
                            vm.exit_call();
                            let __t42 = __call_res?;
                            s_i32_0 = __t42;
                            l37 = s_i32_0;
                            s_i32_0 = l37;
                            s_i32_0 = ops::i32_eqz(s_i32_0);
                            'blk_49: {
                                if s_i32_0 != 0 {
                                    s_i32_0 = l42;
                                    s_i32_1 = 48i32;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 19usize, s_i32_0, s_i32_1, host);
                                    vm.exit_call();
                                    __call_res?;
                                } else {
                                    s_i32_0 = 0i32;
                                    l38 = s_i32_0;
                                    'blk_50: {
                                        'blk_51: loop {
                                            vm.deduct(host, 1)?;
                                            s_i32_0 = l38;
                                            s_i32_1 = l37;
                                            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                                            if s_i32_0 != 0 {
                                                break 'blk_50;
                                            }
                                            s_i32_0 = l38;
                                            s_i32_1 = 0i32;
                                            s_i32_0 = ops::i32_gt_u(s_i32_0, s_i32_1);
                                            'blk_52: {
                                                if s_i32_0 != 0 {
                                                    s_i32_0 = l42;
                                                    s_i32_1 = 2i32;
                                                    s_i32_2 = 24415i32;
                                                    vm.enter_call()?;
                                                    let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 17usize, s_i32_0, s_i32_1, s_i32_2, host);
                                                    vm.exit_call();
                                                    __call_res?;
                                                }
                                            }
                                            s_i32_0 = l36;
                                            s_i32_1 = l38;
                                            vm.enter_call()?;
                                            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_0, s_i32_1, host);
                                            vm.exit_call();
                                            let __t43 = __call_res?;
                                            s_i32_0 = __t43;
                                            l39 = s_i32_0;
                                            s_i32_0 = l42;
                                            s_i32_1 = l39;
                                            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 12u32)?;
                                            vm.enter_call()?;
                                            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 21usize, s_i32_0, s_i32_1, host);
                                            vm.exit_call();
                                            __call_res?;
                                            s_i32_0 = l38;
                                            s_i32_1 = 1i32;
                                            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                                            l38 = s_i32_0;
                                            continue 'blk_51;
                                            break 'blk_51;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    s_i32_0 = l34;
                    s_i32_1 = l42;
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 0u32)?;
                    s_i32_2 = l42;
                    s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 4u32)?;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 22usize, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    let __t44 = __call_res?;
                    s_i32_1 = __t44;
                    ops::i32_store(vm, 0, s_i32_0, 36u32, s_i32_1)?;
                    s_i32_0 = l32;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    l32 = s_i32_0;
                    continue 'blk_42;
                    break 'blk_42;
                }
            }
            s_i32_0 = l1;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l1 = s_i32_0;
            continue 'blk_26;
            break 'blk_26;
        }
    }
    s_i32_0 = 0i32;
    l1 = s_i32_0;
    'blk_53: {
        'blk_54: loop {
            vm.deduct(host, 1)?;
            s_i32_0 = l1;
            s_i32_1 = l0;
            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
            if s_i32_0 != 0 {
                break 'blk_53;
            }
            s_i32_0 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
            s_i32_1 = l1;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t45 = __call_res?;
            s_i32_0 = __t45;
            l3 = s_i32_0;
            s_i32_0 = l3;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
            s_i32_1 = 0i32;
            s_i32_0 = ops::i32_gt_u(s_i32_0, s_i32_1);
            'blk_55: {
                if s_i32_0 != 0 {
                    s_i32_0 = l1;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    l2 = s_i32_0;
                    'blk_56: {
                        'blk_57: loop {
                            vm.deduct(host, 1)?;
                            s_i32_0 = l2;
                            s_i32_1 = l0;
                            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                            if s_i32_0 != 0 {
                                break 'blk_56;
                            }
                            s_i32_0 = l1;
                            s_i32_1 = l2;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, host);
                            vm.exit_call();
                            let __t46 = __call_res?;
                            s_i32_0 = __t46;
                            'blk_58: {
                                if s_i32_0 != 0 {
                                    s_i32_0 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
                                    s_i32_1 = l2;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_0, s_i32_1, host);
                                    vm.exit_call();
                                    let __t47 = __call_res?;
                                    s_i32_0 = __t47;
                                    l4 = s_i32_0;
                                    s_i32_0 = l3;
                                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 16u32)?;
                                    l20 = s_i32_0;
                                    s_i32_0 = l20;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 0usize, s_i32_0, host);
                                    vm.exit_call();
                                    let __t48 = __call_res?;
                                    s_i32_0 = __t48;
                                    l21 = s_i32_0;
                                    s_i32_0 = l4;
                                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 16u32)?;
                                    l44 = s_i32_0;
                                    s_i32_0 = l44;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 0usize, s_i32_0, host);
                                    vm.exit_call();
                                    let __t49 = __call_res?;
                                    s_i32_0 = __t49;
                                    l45 = s_i32_0;
                                    s_i32_0 = 0i32;
                                    l22 = s_i32_0;
                                    'blk_59: {
                                        'blk_60: loop {
                                            vm.deduct(host, 1)?;
                                            s_i32_0 = l22;
                                            s_i32_1 = l21;
                                            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                                            if s_i32_0 != 0 {
                                                break 'blk_59;
                                            }
                                            s_i32_0 = l20;
                                            s_i32_1 = l22;
                                            vm.enter_call()?;
                                            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_0, s_i32_1, host);
                                            vm.exit_call();
                                            let __t50 = __call_res?;
                                            s_i32_0 = __t50;
                                            l24 = s_i32_0;
                                            s_i32_0 = l24;
                                            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                                            'blk_61: {
                                                if s_i32_0 != 0 {
                                                    s_i32_0 = 0i32;
                                                    l23 = s_i32_0;
                                                    'blk_62: {
                                                        'blk_63: loop {
                                                            vm.deduct(host, 1)?;
                                                            s_i32_0 = l23;
                                                            s_i32_1 = l45;
                                                            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                                                            if s_i32_0 != 0 {
                                                                break 'blk_62;
                                                            }
                                                            s_i32_0 = l44;
                                                            s_i32_1 = l23;
                                                            vm.enter_call()?;
                                                            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_0, s_i32_1, host);
                                                            vm.exit_call();
                                                            let __t51 = __call_res?;
                                                            s_i32_0 = __t51;
                                                            l25 = s_i32_0;
                                                            s_i32_0 = l25;
                                                            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                                                            s_i32_1 = l24;
                                                            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 16u32)?;
                                                            s_i32_2 = l24;
                                                            s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 20u32)?;
                                                            s_i32_3 = l25;
                                                            s_i32_3 = ops::i32_load(vm, 0, s_i32_3, 16u32)?;
                                                            s_i32_4 = l25;
                                                            s_i32_4 = ops::i32_load(vm, 0, s_i32_4, 20u32)?;
                                                            vm.enter_call()?;
                                                            let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_i32(vm, c_map, 5usize, s_i32_1, s_i32_2, s_i32_3, s_i32_4, host);
                                                            vm.exit_call();
                                                            let __t52 = __call_res?;
                                                            s_i32_1 = __t52;
                                                            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                                                            'blk_64: {
                                                                if s_i32_0 != 0 {
                                                                    return Err(WasmTrap::Unreachable);
                                                                }
                                                            }
                                                            s_i32_0 = l23;
                                                            s_i32_1 = 1i32;
                                                            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                                                            l23 = s_i32_0;
                                                            continue 'blk_63;
                                                            break 'blk_63;
                                                        }
                                                    }
                                                }
                                            }
                                            s_i32_0 = l22;
                                            s_i32_1 = 1i32;
                                            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                                            l22 = s_i32_0;
                                            continue 'blk_60;
                                            break 'blk_60;
                                        }
                                    }
                                    s_i32_0 = l3;
                                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 20u32)?;
                                    l30 = s_i32_0;
                                    s_i32_0 = l30;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 0usize, s_i32_0, host);
                                    vm.exit_call();
                                    let __t53 = __call_res?;
                                    s_i32_0 = __t53;
                                    l31 = s_i32_0;
                                    s_i32_0 = l4;
                                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 20u32)?;
                                    l46 = s_i32_0;
                                    s_i32_0 = l46;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 0usize, s_i32_0, host);
                                    vm.exit_call();
                                    let __t54 = __call_res?;
                                    s_i32_0 = __t54;
                                    l47 = s_i32_0;
                                    s_i32_0 = 0i32;
                                    l32 = s_i32_0;
                                    'blk_65: {
                                        'blk_66: loop {
                                            vm.deduct(host, 1)?;
                                            s_i32_0 = l32;
                                            s_i32_1 = l31;
                                            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                                            if s_i32_0 != 0 {
                                                break 'blk_65;
                                            }
                                            s_i32_0 = l30;
                                            s_i32_1 = l32;
                                            vm.enter_call()?;
                                            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_0, s_i32_1, host);
                                            vm.exit_call();
                                            let __t55 = __call_res?;
                                            s_i32_0 = __t55;
                                            l34 = s_i32_0;
                                            s_i32_0 = l34;
                                            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                                            'blk_67: {
                                                if s_i32_0 != 0 {
                                                    s_i32_0 = 0i32;
                                                    l33 = s_i32_0;
                                                    'blk_68: {
                                                        'blk_69: loop {
                                                            vm.deduct(host, 1)?;
                                                            s_i32_0 = l33;
                                                            s_i32_1 = l47;
                                                            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                                                            if s_i32_0 != 0 {
                                                                break 'blk_68;
                                                            }
                                                            s_i32_0 = l46;
                                                            s_i32_1 = l33;
                                                            vm.enter_call()?;
                                                            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_0, s_i32_1, host);
                                                            vm.exit_call();
                                                            let __t56 = __call_res?;
                                                            s_i32_0 = __t56;
                                                            l35 = s_i32_0;
                                                            s_i32_0 = l35;
                                                            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                                                            s_i32_1 = l34;
                                                            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 16u32)?;
                                                            s_i32_2 = l34;
                                                            s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 20u32)?;
                                                            s_i32_3 = l35;
                                                            s_i32_3 = ops::i32_load(vm, 0, s_i32_3, 16u32)?;
                                                            s_i32_4 = l35;
                                                            s_i32_4 = ops::i32_load(vm, 0, s_i32_4, 20u32)?;
                                                            vm.enter_call()?;
                                                            let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_i32(vm, c_map, 5usize, s_i32_1, s_i32_2, s_i32_3, s_i32_4, host);
                                                            vm.exit_call();
                                                            let __t57 = __call_res?;
                                                            s_i32_1 = __t57;
                                                            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                                                            'blk_70: {
                                                                if s_i32_0 != 0 {
                                                                    s_i32_0 = l34;
                                                                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 24u32)?;
                                                                    vm.enter_call()?;
                                                                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 0usize, s_i32_0, host);
                                                                    vm.exit_call();
                                                                    let __t58 = __call_res?;
                                                                    s_i32_0 = __t58;
                                                                    s_i32_1 = l35;
                                                                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 24u32)?;
                                                                    vm.enter_call()?;
                                                                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 0usize, s_i32_1, host);
                                                                    vm.exit_call();
                                                                    let __t59 = __call_res?;
                                                                    s_i32_1 = __t59;
                                                                    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
                                                                    s_i32_1 = l34;
                                                                    s_i32_2 = l35;
                                                                    vm.enter_call()?;
                                                                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 15usize, s_i32_1, s_i32_2, host);
                                                                    vm.exit_call();
                                                                    let __t60 = __call_res?;
                                                                    s_i32_1 = __t60;
                                                                    s_i32_0 = ops::i32_or(s_i32_0, s_i32_1);
                                                                    'blk_71: {
                                                                        if s_i32_0 != 0 {
                                                                            return Err(WasmTrap::Unreachable);
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                            s_i32_0 = l33;
                                                            s_i32_1 = 1i32;
                                                            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                                                            l33 = s_i32_0;
                                                            continue 'blk_69;
                                                            break 'blk_69;
                                                        }
                                                    }
                                                }
                                            }
                                            s_i32_0 = l32;
                                            s_i32_1 = 1i32;
                                            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                                            l32 = s_i32_0;
                                            continue 'blk_66;
                                            break 'blk_66;
                                        }
                                    }
                                }
                            }
                            s_i32_0 = l2;
                            s_i32_1 = 1i32;
                            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                            l2 = s_i32_0;
                            continue 'blk_57;
                            break 'blk_57;
                        }
                    }
                }
            }
            s_i32_0 = l1;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l1 = s_i32_0;
            continue 'blk_54;
            break 'blk_54;
        }
    }
    return Ok(());
    }

    #[inline(never)]
    pub(crate) fn f_78eba914826e(vm: &mut SubstituteVm, dispatch: &'static crate::ModuleDispatch, g_map: &'static [usize], c_map: &'static [u32], p0: i32, p1: i32, p2: i32, p3: i32, p4: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = p2;
    let mut l3: i32 = p3;
    let mut l4: i32 = p4;
    let mut l5: i32 = 0i32;
    let mut l6: i32 = 0i32;
    let mut l7: i32 = 0i32;
    let mut l8: i32 = 0i32;
    let mut l9: i32 = 0i32;
    let mut l10: i32 = 0i32;
    let mut l11: i32 = 0i32;
    let mut l12: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i32_2: i32 = 0i32;
    let mut s_i32_3: i32 = 0i32;
    let mut s_i32_4: i32 = 0i32;
    let mut s_i32_5: i32 = 0i32;
    vm.deduct(host, 1)?;
    s_i32_0 = l0;
    s_i32_1 = 0i32;
    s_i32_2 = l1;
    s_i32_3 = l2;
    s_i32_4 = l3;
    s_i32_5 = l4;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i32_i32_i32_i32_i32_ret_i32(vm, c_map, 0usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, s_i32_4, s_i32_5, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l5 = s_i32_0;
    s_i32_0 = l5;
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = l5;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
    s_i32_1 = l0;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t1 = __call_res?;
    s_i32_0 = __t1;
    l6 = s_i32_0;
    s_i32_0 = l6;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
    l7 = s_i32_0;
    s_i32_0 = l7;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 2usize, s_i32_0, host);
    vm.exit_call();
    let __t2 = __call_res?;
    s_i32_0 = __t2;
    l8 = s_i32_0;
    s_i32_0 = 0i32;
    l9 = s_i32_0;
    'blk_2: {
        'blk_3: loop {
            vm.deduct(host, 1)?;
            s_i32_0 = l9;
            s_i32_1 = l8;
            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
            if s_i32_0 != 0 {
                break 'blk_2;
            }
            s_i32_0 = l7;
            s_i32_1 = l9;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i32_0 = __t3;
            l10 = s_i32_0;
            s_i32_0 = l2;
            'blk_4: {
                if s_i32_0 != 0 {
                    s_i32_0 = l10;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
                    s_i32_1 = l10;
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 12u32)?;
                    s_i32_2 = l10;
                    s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 16u32)?;
                    s_i32_3 = l3;
                    s_i32_4 = l4;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_i32(vm, c_map, 3usize, s_i32_1, s_i32_2, s_i32_3, s_i32_4, host);
                    vm.exit_call();
                    let __t4 = __call_res?;
                    s_i32_1 = __t4;
                    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                    'blk_5: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 0i32;
                            l11 = s_i32_0;
                            'blk_6: {
                                'blk_7: loop {
                                    vm.deduct(host, 1)?;
                                    s_i32_0 = l11;
                                    s_i32_1 = l0;
                                    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                                    if s_i32_0 != 0 {
                                        break 'blk_6;
                                    }
                                    s_i32_0 = l11;
                                    s_i32_1 = l10;
                                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 0u32)?;
                                    s_i32_2 = l10;
                                    s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 4u32)?;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, s_i32_2, host);
                                    vm.exit_call();
                                    let __t5 = __call_res?;
                                    s_i32_0 = __t5;
                                    'blk_8: {
                                        if s_i32_0 != 0 {
                                            s_i32_0 = l11;
                                            s_i32_1 = 1i32;
                                            s_i32_2 = l1;
                                            s_i32_3 = 1i32;
                                            s_i32_4 = l3;
                                            s_i32_5 = l4;
                                            vm.enter_call()?;
                                            let __call_res = dispatch.call_p_i32_i32_i32_i32_i32_i32_ret_i32(vm, c_map, 0usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, s_i32_4, s_i32_5, host);
                                            vm.exit_call();
                                            let __t6 = __call_res?;
                                            s_i32_0 = __t6;
                                            l12 = s_i32_0;
                                            s_i32_0 = l12;
                                            'blk_9: {
                                                if s_i32_0 != 0 {
                                                    s_i32_0 = l5;
                                                    s_i32_1 = l5;
                                                    s_i32_2 = l12;
                                                    s_i32_1 = ops::i32_ne(s_i32_1, s_i32_2);
                                                    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                                                    'blk_10: {
                                                        if s_i32_0 != 0 {
                                                            return Err(WasmTrap::Unreachable);
                                                        }
                                                    }
                                                    s_i32_0 = l12;
                                                    l5 = s_i32_0;
                                                }
                                            }
                                        }
                                    }
                                    s_i32_0 = l11;
                                    s_i32_1 = 1i32;
                                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                                    l11 = s_i32_0;
                                    continue 'blk_7;
                                    break 'blk_7;
                                }
                            }
                        }
                    }
                } else {
                    s_i32_0 = l10;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 20u32)?;
                    s_i32_1 = l10;
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 24u32)?;
                    s_i32_2 = l3;
                    s_i32_3 = l4;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_i32(vm, c_map, 3usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
                    vm.exit_call();
                    let __t7 = __call_res?;
                    s_i32_0 = __t7;
                    'blk_11: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 0i32;
                            l11 = s_i32_0;
                            'blk_12: {
                                'blk_13: loop {
                                    vm.deduct(host, 1)?;
                                    s_i32_0 = l11;
                                    s_i32_1 = l0;
                                    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                                    if s_i32_0 != 0 {
                                        break 'blk_12;
                                    }
                                    s_i32_0 = l11;
                                    s_i32_1 = l10;
                                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 0u32)?;
                                    s_i32_2 = l10;
                                    s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 4u32)?;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, s_i32_2, host);
                                    vm.exit_call();
                                    let __t8 = __call_res?;
                                    s_i32_0 = __t8;
                                    'blk_14: {
                                        if s_i32_0 != 0 {
                                            s_i32_0 = l11;
                                            s_i32_1 = 1i32;
                                            s_i32_2 = l1;
                                            s_i32_3 = l10;
                                            s_i32_3 = ops::i32_load(vm, 0, s_i32_3, 8u32)?;
                                            s_i32_4 = l10;
                                            s_i32_4 = ops::i32_load(vm, 0, s_i32_4, 12u32)?;
                                            s_i32_5 = l10;
                                            s_i32_5 = ops::i32_load(vm, 0, s_i32_5, 16u32)?;
                                            vm.enter_call()?;
                                            let __call_res = dispatch.call_p_i32_i32_i32_i32_i32_i32_ret_i32(vm, c_map, 0usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, s_i32_4, s_i32_5, host);
                                            vm.exit_call();
                                            let __t9 = __call_res?;
                                            s_i32_0 = __t9;
                                            l12 = s_i32_0;
                                            s_i32_0 = l12;
                                            'blk_15: {
                                                if s_i32_0 != 0 {
                                                    s_i32_0 = l5;
                                                    s_i32_1 = l5;
                                                    s_i32_2 = l12;
                                                    s_i32_1 = ops::i32_ne(s_i32_1, s_i32_2);
                                                    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                                                    'blk_16: {
                                                        if s_i32_0 != 0 {
                                                            return Err(WasmTrap::Unreachable);
                                                        }
                                                    }
                                                    s_i32_0 = l12;
                                                    l5 = s_i32_0;
                                                }
                                            }
                                        }
                                    }
                                    s_i32_0 = l11;
                                    s_i32_1 = 1i32;
                                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                                    l11 = s_i32_0;
                                    continue 'blk_13;
                                    break 'blk_13;
                                }
                            }
                        }
                    }
                }
            }
            s_i32_0 = l9;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l9 = s_i32_0;
            continue 'blk_3;
            break 'blk_3;
        }
    }
    s_i32_0 = l2;
    s_i32_1 = l5;
    s_i32_1 = ops::i32_eqz(s_i32_1);
    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
    'blk_17: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            l11 = s_i32_0;
            'blk_18: {
                'blk_19: loop {
                    vm.deduct(host, 1)?;
                    s_i32_0 = l11;
                    s_i32_1 = l0;
                    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                    if s_i32_0 != 0 {
                        break 'blk_18;
                    }
                    s_i32_0 = l11;
                    s_i32_1 = 1i32;
                    s_i32_2 = l1;
                    s_i32_3 = 1i32;
                    s_i32_4 = l3;
                    s_i32_5 = l4;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_i32_i32_i32_ret_i32(vm, c_map, 0usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, s_i32_4, s_i32_5, host);
                    vm.exit_call();
                    let __t10 = __call_res?;
                    s_i32_0 = __t10;
                    l12 = s_i32_0;
                    s_i32_0 = l12;
                    'blk_20: {
                        if s_i32_0 != 0 {
                            s_i32_0 = l5;
                            s_i32_1 = l5;
                            s_i32_2 = l12;
                            s_i32_1 = ops::i32_ne(s_i32_1, s_i32_2);
                            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                            'blk_21: {
                                if s_i32_0 != 0 {
                                    return Err(WasmTrap::Unreachable);
                                }
                            }
                            s_i32_0 = l12;
                            l5 = s_i32_0;
                        }
                    }
                    s_i32_0 = l11;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    l11 = s_i32_0;
                    continue 'blk_19;
                    break 'blk_19;
                }
            }
        }
    }
    s_i32_0 = l5;
    s_i32_0 = ops::i32_eqz(s_i32_0);
    'blk_22: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = l5;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn f_fb262af913b2(vm: &mut SubstituteVm, dispatch: &'static crate::ModuleDispatch, g_map: &'static [usize], c_map: &'static [u32], p0: i32, p1: i32, p2: i32, p3: i32, p4: i32, p5: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = p2;
    let mut l3: i32 = p3;
    let mut l4: i32 = p4;
    let mut l5: i32 = p5;
    let mut l6: i32 = 0i32;
    let mut l7: i32 = 0i32;
    let mut l8: i32 = 0i32;
    let mut l9: i32 = 0i32;
    let mut l10: i32 = 0i32;
    let mut l11: i32 = 0i32;
    let mut l12: i32 = 0i32;
    let mut l13: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i32_2: i32 = 0i32;
    let mut s_i32_3: i32 = 0i32;
    let mut s_i32_4: i32 = 0i32;
    let mut s_i32_5: i32 = 0i32;
    let mut s_i32_6: i32 = 0i32;
    vm.deduct(host, 1)?;
    s_i32_0 = l0;
    s_i32_1 = 0i32;
    s_i32_2 = l1;
    s_i32_3 = l2;
    s_i32_4 = l3;
    s_i32_5 = l4;
    s_i32_6 = l5;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i32_i32_i32_i32_i32_i32_ret_i32(vm, c_map, 0usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, s_i32_4, s_i32_5, s_i32_6, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l6 = s_i32_0;
    s_i32_0 = l6;
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = l6;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
    s_i32_1 = l0;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t1 = __call_res?;
    s_i32_0 = __t1;
    l7 = s_i32_0;
    s_i32_0 = l7;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
    l8 = s_i32_0;
    s_i32_0 = l8;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 2usize, s_i32_0, host);
    vm.exit_call();
    let __t2 = __call_res?;
    s_i32_0 = __t2;
    l9 = s_i32_0;
    s_i32_0 = 0i32;
    l10 = s_i32_0;
    'blk_2: {
        'blk_3: loop {
            vm.deduct(host, 1)?;
            s_i32_0 = l10;
            s_i32_1 = l9;
            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
            if s_i32_0 != 0 {
                break 'blk_2;
            }
            s_i32_0 = l8;
            s_i32_1 = l10;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i32_0 = __t3;
            l11 = s_i32_0;
            s_i32_0 = l1;
            'blk_4: {
                if s_i32_0 != 0 {
                    s_i32_0 = l11;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
                    s_i32_1 = l11;
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 12u32)?;
                    s_i32_2 = l11;
                    s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 16u32)?;
                    s_i32_3 = l2;
                    s_i32_4 = l3;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_i32(vm, c_map, 3usize, s_i32_1, s_i32_2, s_i32_3, s_i32_4, host);
                    vm.exit_call();
                    let __t4 = __call_res?;
                    s_i32_1 = __t4;
                    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                    'blk_5: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 0i32;
                            l12 = s_i32_0;
                            'blk_6: {
                                'blk_7: loop {
                                    vm.deduct(host, 1)?;
                                    s_i32_0 = l12;
                                    s_i32_1 = l0;
                                    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                                    if s_i32_0 != 0 {
                                        break 'blk_6;
                                    }
                                    s_i32_0 = l12;
                                    s_i32_1 = l11;
                                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 0u32)?;
                                    s_i32_2 = l11;
                                    s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 4u32)?;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, s_i32_2, host);
                                    vm.exit_call();
                                    let __t5 = __call_res?;
                                    s_i32_0 = __t5;
                                    'blk_8: {
                                        if s_i32_0 != 0 {
                                            s_i32_0 = l12;
                                            s_i32_1 = 1i32;
                                            s_i32_2 = 1i32;
                                            s_i32_3 = l2;
                                            s_i32_4 = l3;
                                            s_i32_5 = l4;
                                            s_i32_6 = l5;
                                            vm.enter_call()?;
                                            let __call_res = dispatch.call_p_i32_i32_i32_i32_i32_i32_i32_ret_i32(vm, c_map, 0usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, s_i32_4, s_i32_5, s_i32_6, host);
                                            vm.exit_call();
                                            let __t6 = __call_res?;
                                            s_i32_0 = __t6;
                                            l13 = s_i32_0;
                                            s_i32_0 = l13;
                                            'blk_9: {
                                                if s_i32_0 != 0 {
                                                    s_i32_0 = l6;
                                                    s_i32_1 = l6;
                                                    s_i32_2 = l13;
                                                    s_i32_1 = ops::i32_ne(s_i32_1, s_i32_2);
                                                    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                                                    'blk_10: {
                                                        if s_i32_0 != 0 {
                                                            return Err(WasmTrap::Unreachable);
                                                        }
                                                    }
                                                    s_i32_0 = l13;
                                                    l6 = s_i32_0;
                                                }
                                            }
                                        }
                                    }
                                    s_i32_0 = l12;
                                    s_i32_1 = 1i32;
                                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                                    l12 = s_i32_0;
                                    continue 'blk_7;
                                    break 'blk_7;
                                }
                            }
                        }
                    }
                } else {
                    s_i32_0 = l11;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 20u32)?;
                    s_i32_1 = l11;
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 24u32)?;
                    s_i32_2 = l2;
                    s_i32_3 = l3;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_i32(vm, c_map, 3usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
                    vm.exit_call();
                    let __t7 = __call_res?;
                    s_i32_0 = __t7;
                    'blk_11: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 0i32;
                            l12 = s_i32_0;
                            'blk_12: {
                                'blk_13: loop {
                                    vm.deduct(host, 1)?;
                                    s_i32_0 = l12;
                                    s_i32_1 = l0;
                                    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                                    if s_i32_0 != 0 {
                                        break 'blk_12;
                                    }
                                    s_i32_0 = l12;
                                    s_i32_1 = l11;
                                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 0u32)?;
                                    s_i32_2 = l11;
                                    s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 4u32)?;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, s_i32_2, host);
                                    vm.exit_call();
                                    let __t8 = __call_res?;
                                    s_i32_0 = __t8;
                                    'blk_14: {
                                        if s_i32_0 != 0 {
                                            s_i32_0 = l12;
                                            s_i32_1 = 1i32;
                                            s_i32_2 = l11;
                                            s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 8u32)?;
                                            s_i32_3 = l11;
                                            s_i32_3 = ops::i32_load(vm, 0, s_i32_3, 12u32)?;
                                            s_i32_4 = l11;
                                            s_i32_4 = ops::i32_load(vm, 0, s_i32_4, 16u32)?;
                                            s_i32_5 = l4;
                                            s_i32_6 = l5;
                                            vm.enter_call()?;
                                            let __call_res = dispatch.call_p_i32_i32_i32_i32_i32_i32_i32_ret_i32(vm, c_map, 0usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, s_i32_4, s_i32_5, s_i32_6, host);
                                            vm.exit_call();
                                            let __t9 = __call_res?;
                                            s_i32_0 = __t9;
                                            l13 = s_i32_0;
                                            s_i32_0 = l13;
                                            'blk_15: {
                                                if s_i32_0 != 0 {
                                                    s_i32_0 = l6;
                                                    s_i32_1 = l6;
                                                    s_i32_2 = l13;
                                                    s_i32_1 = ops::i32_ne(s_i32_1, s_i32_2);
                                                    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                                                    'blk_16: {
                                                        if s_i32_0 != 0 {
                                                            return Err(WasmTrap::Unreachable);
                                                        }
                                                    }
                                                    s_i32_0 = l13;
                                                    l6 = s_i32_0;
                                                }
                                            }
                                        }
                                    }
                                    s_i32_0 = l12;
                                    s_i32_1 = 1i32;
                                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                                    l12 = s_i32_0;
                                    continue 'blk_13;
                                    break 'blk_13;
                                }
                            }
                        }
                    }
                }
            }
            s_i32_0 = l10;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l10 = s_i32_0;
            continue 'blk_3;
            break 'blk_3;
        }
    }
    s_i32_0 = l1;
    s_i32_1 = l6;
    s_i32_1 = ops::i32_eqz(s_i32_1);
    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
    'blk_17: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            l12 = s_i32_0;
            'blk_18: {
                'blk_19: loop {
                    vm.deduct(host, 1)?;
                    s_i32_0 = l12;
                    s_i32_1 = l0;
                    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                    if s_i32_0 != 0 {
                        break 'blk_18;
                    }
                    s_i32_0 = l12;
                    s_i32_1 = 1i32;
                    s_i32_2 = 1i32;
                    s_i32_3 = l2;
                    s_i32_4 = l3;
                    s_i32_5 = l4;
                    s_i32_6 = l5;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_i32_i32_i32_i32_ret_i32(vm, c_map, 0usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, s_i32_4, s_i32_5, s_i32_6, host);
                    vm.exit_call();
                    let __t10 = __call_res?;
                    s_i32_0 = __t10;
                    l13 = s_i32_0;
                    s_i32_0 = l13;
                    'blk_20: {
                        if s_i32_0 != 0 {
                            s_i32_0 = l6;
                            s_i32_1 = l6;
                            s_i32_2 = l13;
                            s_i32_1 = ops::i32_ne(s_i32_1, s_i32_2);
                            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                            'blk_21: {
                                if s_i32_0 != 0 {
                                    return Err(WasmTrap::Unreachable);
                                }
                            }
                            s_i32_0 = l13;
                            l6 = s_i32_0;
                        }
                    }
                    s_i32_0 = l12;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    l12 = s_i32_0;
                    continue 'blk_19;
                    break 'blk_19;
                }
            }
        }
    }
    s_i32_0 = l6;
    s_i32_0 = ops::i32_eqz(s_i32_0);
    'blk_22: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = l6;
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

const FN_0_GLOBALS: &[usize] = &[0usize, 1usize];
const FN_2_CALLEES: &[u32] = &[0u32];
const FN_3_CALLEES: &[u32] = &[0u32];
const FN_4_CALLEES: &[u32] = &[0u32];
const FN_5_CALLEES: &[u32] = &[0u32];
const FN_6_CALLEES: &[u32] = &[0u32];
const FN_10_CALLEES: &[u32] = &[1u32, 0u32];
const FN_11_GLOBALS: &[usize] = &[8usize];
const FN_11_CALLEES: &[u32] = &[1u32, 1u32, 3u32];
const FN_12_GLOBALS: &[usize] = &[8usize];
const FN_12_CALLEES: &[u32] = &[0u32];
const FN_13_GLOBALS: &[usize] = &[5usize, 4usize, 6usize];
const FN_14_GLOBALS: &[usize] = &[2usize, 3usize, 4usize];
const FN_14_CALLEES: &[u32] = &[2u32];
const FN_15_GLOBALS: &[usize] = &[2usize, 3usize, 4usize];
const FN_15_CALLEES: &[u32] = &[2u32];
const FN_16_CALLEES: &[u32] = &[1u32];
const FN_17_GLOBALS: &[usize] = &[2usize, 3usize, 4usize];
const FN_17_CALLEES: &[u32] = &[2u32, 0u32];
const FN_18_CALLEES: &[u32] = &[0u32, 0u32];
const FN_19_CALLEES: &[u32] = &[0u32, 1u32];
const FN_20_CALLEES: &[u32] = &[3u32];
const FN_21_CALLEES: &[u32] = &[1u32];
const FN_22_CALLEES: &[u32] = &[4u32, 0u32];
const FN_23_CALLEES: &[u32] = &[0u32, 4u32, 0u32];
const FN_24_GLOBALS: &[usize] = &[4usize];
const FN_24_CALLEES: &[u32] = &[2u32];
const FN_25_GLOBALS: &[usize] = &[4usize];
const FN_25_CALLEES: &[u32] = &[2u32, 1u32];
const FN_26_GLOBALS: &[usize] = &[2usize, 3usize, 10usize];
const FN_26_CALLEES: &[u32] = &[2u32, 1u32, 1u32, 0u32, 2u32];
const FN_27_GLOBALS: &[usize] = &[4usize, 7usize, 8usize, 9usize];
const FN_27_CALLEES: &[u32] = &[2u32, 3u32, 1u32, 4u32, 0u32];
const FN_28_GLOBALS: &[usize] = &[4usize, 9usize];
const FN_28_CALLEES: &[u32] = &[2u32];
const FN_29_CALLEES: &[u32] = &[1u32, 0u32];
const FN_30_GLOBALS: &[usize] = &[4usize, 9usize, 8usize];
const FN_30_CALLEES: &[u32] = &[2u32, 3u32, 1u32, 1u32, 0u32, 5u32, 0u32, 0u32, 0u32, 4u32];
const FN_31_GLOBALS: &[usize] = &[11usize];
const FN_31_CALLEES: &[u32] = &[2u32, 0u32, 0u32];
const FN_32_GLOBALS: &[usize] = &[11usize];
const FN_32_CALLEES: &[u32] = &[6u32, 2u32, 0u32, 2u32];
const FN_33_CALLEES: &[u32] = &[6u32, 1u32, 4u32, 1u32, 0u32];
const FN_34_GLOBALS: &[usize] = &[12usize];
const FN_34_CALLEES: &[u32] = &[2u32, 0u32, 0u32];
const FN_35_GLOBALS: &[usize] = &[12usize];
const FN_35_CALLEES: &[u32] = &[7u32, 2u32, 0u32, 2u32];
const FN_36_GLOBALS: &[usize] = &[12usize];
const FN_36_CALLEES: &[u32] = &[0u32];
const FN_37_GLOBALS: &[usize] = &[10usize, 8usize, 4usize, 7usize, 9usize];
const FN_37_CALLEES: &[u32] = &[2u32, 0u32, 1u32, 2u32, 8u32, 2u32, 1u32, 3u32, 1u32, 0u32, 4u32, 0u32, 2u32, 0u32, 7u32, 2u32];
const FN_38_GLOBALS: &[usize] = &[13usize];
const FN_38_CALLEES: &[u32] = &[0u32, 0u32, 7u32];
const FN_39_GLOBALS: &[usize] = &[14usize];
const FN_39_CALLEES: &[u32] = &[0u32, 2u32];
const FN_42_CALLEES: &[u32] = &[4u32];
const FN_45_GLOBALS: &[usize] = &[14usize];
const FN_45_CALLEES: &[u32] = &[2u32, 0u32, 0u32, 2u32];
const FN_46_GLOBALS: &[usize] = &[14usize];
const FN_46_CALLEES: &[u32] = &[2u32, 0u32, 0u32, 2u32];
const FN_47_CALLEES: &[u32] = &[9u32];
const FN_48_GLOBALS: &[usize] = &[8usize];
const FN_48_CALLEES: &[u32] = &[1u32, 1u32, 1u32, 4u32];
const FN_49_GLOBALS: &[usize] = &[8usize];
const FN_49_CALLEES: &[u32] = &[1u32, 1u32, 1u32, 1u32, 4u32];
const FN_50_GLOBALS: &[usize] = &[6usize, 8usize, 5usize];
const FN_50_CALLEES: &[u32] = &[3u32, 1u32];
const FN_51_GLOBALS: &[usize] = &[8usize];
const FN_51_CALLEES: &[u32] = &[2u32, 3u32, 7u32];
const FN_52_GLOBALS: &[usize] = &[9usize];
const FN_52_CALLEES: &[u32] = &[2u32, 5u32, 7u32];
const FN_53_GLOBALS: &[usize] = &[2usize, 3usize, 9usize];
const FN_53_CALLEES: &[u32] = &[0u32, 5u32];
const FN_54_GLOBALS: &[usize] = &[9usize, 8usize];
const FN_54_CALLEES: &[u32] = &[3u32, 0u32, 1u32, 1u32, 2u32, 2u32, 2u32, 9u32];
const FN_55_CALLEES: &[u32] = &[1u32, 0u32, 0u32];
const FN_56_CALLEES: &[u32] = &[10u32, 1u32, 1u32, 1u32, 0u32];
const FN_57_CALLEES: &[u32] = &[1u32];
const FN_58_GLOBALS: &[usize] = &[8usize];
const FN_58_CALLEES: &[u32] = &[0u32, 3u32, 8u32, 11u32, 5u32, 1u32, 2u32, 2u32, 3u32, 15u32, 1u32];
const FN_61_GLOBALS: &[usize] = &[9usize, 8usize, 17usize];
const FN_61_CALLEES: &[u32] = &[3u32, 0u32, 12u32, 13u32, 11u32, 1u32, 2u32, 2u32, 1u32, 10u32, 9u32, 2u32, 5u32, 8u32, 14u32, 9u32, 10u32];
const FN_62_GLOBALS: &[usize] = &[8usize, 9usize, 15usize, 22usize];
const FN_62_CALLEES: &[u32] = &[1u32, 0u32, 1u32, 3u32, 6u32, 2u32, 9u32, 5u32, 2u32, 8u32, 11u32, 2u32, 15u32, 14u32];
const FN_63_CALLEES: &[u32] = &[1u32];
const FN_64_CALLEES: &[u32] = &[1u32];
const FN_65_GLOBALS: &[usize] = &[14usize, 15usize, 16usize];
const FN_65_CALLEES: &[u32] = &[1u32, 4u32, 0u32, 3u32, 3u32];
const FN_66_GLOBALS: &[usize] = &[16usize];
const FN_66_CALLEES: &[u32] = &[0u32, 0u32];
const FN_67_GLOBALS: &[usize] = &[16usize];
const FN_67_CALLEES: &[u32] = &[0u32, 0u32];
const FN_68_GLOBALS: &[usize] = &[16usize];
const FN_68_CALLEES: &[u32] = &[2u32, 0u32, 0u32];
const FN_69_GLOBALS: &[usize] = &[16usize, 15usize];
const FN_69_CALLEES: &[u32] = &[0u32, 2u32, 0u32];
const FN_70_GLOBALS: &[usize] = &[16usize, 15usize];
const FN_70_CALLEES: &[u32] = &[0u32, 2u32, 0u32];
const FN_71_GLOBALS: &[usize] = &[16usize];
const FN_71_CALLEES: &[u32] = &[3u32, 0u32, 2u32, 0u32, 2u32, 4u32];
const FN_72_GLOBALS: &[usize] = &[16usize];
const FN_72_CALLEES: &[u32] = &[4u32, 0u32, 2u32, 0u32, 2u32];
const FN_73_CALLEES: &[u32] = &[9u32, 0u32, 10u32, 3u32, 8u32, 4u32];
const FN_74_GLOBALS: &[usize] = &[14usize, 15usize];
const FN_74_CALLEES: &[u32] = &[0u32];
const FN_75_GLOBALS: &[usize] = &[14usize, 15usize];
const FN_75_CALLEES: &[u32] = &[0u32];
const FN_76_GLOBALS: &[usize] = &[14usize];
const FN_76_CALLEES: &[u32] = &[0u32, 12u32];
const FN_77_GLOBALS: &[usize] = &[14usize, 15usize];
const FN_77_CALLEES: &[u32] = &[0u32, 0u32, 4u32, 5u32];
const FN_78_CALLEES: &[u32] = &[2u32, 0u32];
const FN_79_GLOBALS: &[usize] = &[16usize];
const FN_79_CALLEES: &[u32] = &[4u32, 3u32, 0u32, 2u32, 0u32];
const FN_80_GLOBALS: &[usize] = &[16usize, 15usize, 8usize];
const FN_80_CALLEES: &[u32] = &[2u32, 0u32, 2u32, 15u32, 11u32, 0u32, 0u32, 13u32, 8u32, 2u32, 3u32, 12u32, 3u32, 1u32, 1u32, 14u32, 1u32, 4u32, 3u32, 1u32, 3u32, 5u32, 8u32];
const FN_81_GLOBALS: &[usize] = &[16usize];
const FN_81_CALLEES: &[u32] = &[0u32, 2u32, 0u32];
const FN_82_GLOBALS: &[usize] = &[16usize];
const FN_82_CALLEES: &[u32] = &[1u32, 0u32, 2u32, 0u32, 2u32];
const FN_83_CALLEES: &[u32] = &[2u32, 0u32];
const FN_84_GLOBALS: &[usize] = &[16usize];
const FN_84_CALLEES: &[u32] = &[0u32, 2u32, 0u32, 5u32];
const FN_85_GLOBALS: &[usize] = &[16usize];
const FN_85_CALLEES: &[u32] = &[0u32, 0u32, 2u32, 0u32, 2u32];
const FN_86_CALLEES: &[u32] = &[0u32, 1u32];
const FN_87_CALLEES: &[u32] = &[2u32, 0u32, 2u32];
const FN_88_CALLEES: &[u32] = &[2u32, 2u32];
const FN_89_CALLEES: &[u32] = &[4u32];
const FN_90_CALLEES: &[u32] = &[4u32, 0u32];
const FN_91_CALLEES: &[u32] = &[4u32];
const FN_92_CALLEES: &[u32] = &[4u32, 0u32];
const FN_93_CALLEES: &[u32] = &[4u32, 1u32];
const FN_94_CALLEES: &[u32] = &[4u32, 1u32];
const FN_95_CALLEES: &[u32] = &[4u32, 0u32];
const FN_96_CALLEES: &[u32] = &[4u32, 0u32];
const FN_97_GLOBALS: &[usize] = &[8usize];
const FN_97_CALLEES: &[u32] = &[2u32, 1u32, 4u32, 0u32, 0u32, 0u32];
const FN_98_GLOBALS: &[usize] = &[11usize, 12usize, 8usize];
const FN_98_CALLEES: &[u32] = &[1u32, 4u32, 2u32, 0u32, 0u32, 0u32];
const FN_99_GLOBALS: &[usize] = &[17usize, 18usize, 19usize, 20usize, 21usize, 22usize, 23usize, 24usize];
const FN_99_CALLEES: &[u32] = &[0u32, 1u32, 8u32];
const FN_100_GLOBALS: &[usize] = &[17usize, 18usize, 20usize, 19usize, 21usize, 8usize];
const FN_100_CALLEES: &[u32] = &[4u32, 13u32, 0u32, 5u32, 16u32, 5u32, 6u32, 6u32, 7u32, 0u32, 14u32, 2u32];
const FN_101_CALLEES: &[u32] = &[7u32, 0u32];
const FN_102_CALLEES: &[u32] = &[6u32, 7u32, 0u32];
const FN_103_CALLEES: &[u32] = &[0u32];
const FN_104_CALLEES: &[u32] = &[0u32];
const FN_105_GLOBALS: &[usize] = &[14usize];
const FN_105_CALLEES: &[u32] = &[12u32, 6u32, 17u32, 5u32, 6u32, 7u32, 0u32, 7u32, 0u32, 15u32];
const FN_106_GLOBALS: &[usize] = &[14usize, 8usize, 15usize];
const FN_106_CALLEES: &[u32] = &[12u32, 0u32, 1u32, 8u32, 13u32, 0u32, 5u32, 5u32, 6u32, 6u32, 7u32, 4u32, 5u32, 16u32, 0u32, 1u32, 8u32, 4u32, 14u32, 2u32];
const FN_107_GLOBALS: &[usize] = &[25usize];
const FN_108_CALLEES: &[u32] = &[2u32, 0u32, 0u32];
const FN_109_CALLEES: &[u32] = &[2u32, 0u32, 0u32];
const FN_110_CALLEES: &[u32] = &[2u32, 0u32, 0u32];
const FN_111_CALLEES: &[u32] = &[9u32, 0u32, 2u32];
const FN_112_CALLEES: &[u32] = &[2u32, 1u32, 0u32, 2u32];
const FN_113_CALLEES: &[u32] = &[9u32, 10u32, 2u32, 0u32, 16u32];
const FN_114_CALLEES: &[u32] = &[0u32, 2u32, 0u32];
const FN_115_CALLEES: &[u32] = &[16u32, 2u32, 0u32, 10u32, 1u32, 3u32, 0u32, 9u32, 8u32];
const FN_116_GLOBALS: &[usize] = &[14usize, 15usize];
const FN_116_CALLEES: &[u32] = &[9u32, 13u32, 2u32, 0u32, 0u32, 10u32, 4u32, 3u32, 8u32, 4u32, 7u32, 1u32, 8u32, 5u32, 2u32, 1u32, 2u32];
const FN_117_GLOBALS: &[usize] = &[17usize, 20usize];
const FN_117_CALLEES: &[u32] = &[5u32];
const FN_118_CALLEES: &[u32] = &[3u32, 5u32, 0u32, 6u32, 6u32];
const FN_119_CALLEES: &[u32] = &[5u32, 3u32, 0u32, 7u32];
const FN_120_GLOBALS: &[usize] = &[14usize];
const FN_120_CALLEES: &[u32] = &[4u32, 0u32, 4u32, 16u32, 7u32, 5u32, 2u32, 2u32];
const FN_121_GLOBALS: &[usize] = &[21usize, 12usize, 8usize];
const FN_121_CALLEES: &[u32] = &[2u32, 1u32, 0u32, 13u32, 3u32, 3u32, 9u32, 10u32, 2u32, 3u32, 8u32, 0u32, 0u32, 0u32, 11u32, 16u32, 7u32, 5u32, 1u32, 8u32, 12u32, 1u32, 13u32, 6u32];
const FN_122_GLOBALS: &[usize] = &[15usize, 14usize, 17usize, 20usize];
const FN_122_CALLEES: &[u32] = &[7u32, 3u32, 2u32, 4u32, 16u32, 5u32, 6u32, 0u32, 9u32, 13u32, 3u32, 12u32, 4u32, 5u32, 2u32, 1u32, 0u32, 2u32, 2u32, 0u32, 10u32, 7u32, 4u32, 3u32, 8u32, 4u32, 1u32, 5u32, 6u32, 7u32, 19u32, 1u32, 0u32, 4u32, 12u32, 18u32, 8u32, 5u32, 2u32, 6u32, 6u32, 0u32, 17u32, 6u32, 7u32, 15u32];
const FN_123_GLOBALS: &[usize] = &[17usize, 8usize, 22usize, 24usize, 20usize];
const FN_123_CALLEES: &[u32] = &[3u32, 13u32, 0u32, 5u32, 1u32, 2u32, 0u32, 1u32, 0u32, 2u32, 10u32, 1u32, 3u32, 5u32, 16u32, 17u32, 13u32, 6u32, 14u32, 2u32];
const FN_124_CALLEES: &[u32] = &[9u32, 13u32, 1u32, 5u32, 16u32, 2u32, 0u32, 10u32];
const FN_125_CALLEES: &[u32] = &[1u32, 1u32, 0u32, 17u32, 13u32, 5u32];
const FN_126_GLOBALS: &[usize] = &[22usize, 23usize, 17usize, 20usize, 16usize, 24usize, 8usize];
const FN_126_CALLEES: &[u32] = &[13u32, 5u32, 6u32, 6u32, 7u32, 16u32, 2u32, 0u32, 5u32, 14u32, 2u32];
const FN_127_GLOBALS: &[usize] = &[16usize];
const FN_127_CALLEES: &[u32] = &[2u32, 0u32, 1u32];
const FN_128_GLOBALS: &[usize] = &[16usize, 22usize, 24usize, 8usize];
const FN_128_CALLEES: &[u32] = &[0u32, 7u32, 2u32, 9u32, 0u32, 0u32, 8u32, 13u32, 5u32, 4u32, 16u32, 7u32, 0u32, 14u32, 2u32];
const FN_129_GLOBALS: &[usize] = &[1usize, 0usize, 2usize, 3usize, 10usize, 11usize, 12usize, 13usize, 14usize, 15usize, 16usize, 17usize, 22usize, 25usize, 8usize];
const FN_129_CALLEES: &[u32] = &[1u32, 2u32, 1u32, 0u32, 11u32, 2u32, 5u32, 4u32, 6u32, 7u32, 8u32, 9u32, 16u32];
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
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_ca95e99a0f69, globals: FN_122_GLOBALS, callees: FN_122_CALLEES },
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
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_9fa2e2ee3bf1, globals: FN_116_GLOBALS, callees: FN_116_CALLEES },
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
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_1bbfa2fe2e70, globals: &[], callees: &[] },
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
    sig_p_i32_i32_i32_i32_ret_i32: BOUND_SIG_P_I32_I32_I32_I32_RET_I32,
    sig_p_i32_i32_i32_i32_ret_void: BOUND_SIG_P_I32_I32_I32_I32_RET_VOID,
    sig_p_i32_i32_i32_i64_ret_void: &[],
    sig_p_i32_i32_i32_ret_i32: BOUND_SIG_P_I32_I32_I32_RET_I32,
    sig_p_i32_i32_i32_ret_void: BOUND_SIG_P_I32_I32_I32_RET_VOID,
    sig_p_i32_i32_i64_i32_i32_ret_i32: &[],
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

const MODULE_META: SubstituteModuleMeta = SubstituteModuleMeta {
    name: "85899_87792/f1680@l8589988341",
    accounting_version: 2u32,
    wasm_sha256: [0x61, 0x32, 0xea, 0xc4, 0xb1, 0xdc, 0xb3, 0x59, 0x31, 0x5f, 0x98, 0x22, 0x67, 0x42, 0xae, 0x68, 0x5a, 0x63, 0x49, 0x0d, 0x94, 0xd3, 0xf0, 0x42, 0x90, 0x6c, 0x3f, 0xc9, 0x68, 0x5c, 0xae, 0x7c],
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
        let res = DISPATCH.call_direct_p_unit_ret_i32(vm, 18usize, host);
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

