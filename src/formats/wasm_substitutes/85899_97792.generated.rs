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

//! Generated native Rust WebAssembly substitute chunk `85899_97792`.
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
    pub(crate) fn f_e1ee448cdcd2(vm: &mut SubstituteVm, dispatch: &'static crate::ModuleDispatch, g_map: &'static [usize], c_map: &'static [u32], p0: i32, p1: i32, p2: i32, p3: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = p2;
    let mut l3: i32 = p3;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i32 = 0i32;
    let mut l7: i32 = 0i32;
    let mut l8: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i32_2: i32 = 0i32;
    let mut s_i32_3: i32 = 0i32;
    let mut s_i32_4: i32 = 0i32;
    vm.deduct(host, 1)?;
    s_i32_0 = l0;
    s_i32_1 = l1;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    s_i32_0 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
    s_i32_1 = 1114382i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i32_1 = l1;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            l4 = s_i32_0;
            s_i32_0 = vm.get_i32(*g_map.get(1usize).ok_or(WasmTrap::Unreachable)?)?;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
            s_i32_1 = 0i32;
            s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
            'blk_2: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = vm.get_i32(*g_map.get(2usize).ok_or(WasmTrap::Unreachable)?)?;
            l5 = s_i32_0;
            s_i32_0 = vm.get_i32(*g_map.get(3usize).ok_or(WasmTrap::Unreachable)?)?;
            l6 = s_i32_0;
            s_i32_0 = l2;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 2usize, s_i32_0, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_0 = __t2;
            l7 = s_i32_0;
            s_i32_0 = 0i32;
            l8 = s_i32_0;
            'blk_3: {
                'blk_4: loop {
                    vm.deduct(host, 1)?;
                    s_i32_0 = l8;
                    s_i32_1 = l7;
                    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                    'blk_5: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = l5;
                    s_i32_1 = l6;
                    s_i32_2 = l2;
                    s_i32_3 = l8;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 3usize, s_i32_2, s_i32_3, host);
                    vm.exit_call();
                    let __t3 = __call_res?;
                    s_i32_2 = __t3;
                    s_i32_3 = l3;
                    s_i32_4 = l8;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 3usize, s_i32_3, s_i32_4, host);
                    vm.exit_call();
                    let __t4 = __call_res?;
                    s_i32_3 = __t4;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
                    vm.exit_call();
                    let __t5 = __call_res?;
                    s_i32_0 = __t5;
                    'blk_6: {
                        if s_i32_0 != 0 {
                            break 'blk_3;
                        }
                    }
                    s_i32_0 = l8;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    l8 = s_i32_0;
                    continue 'blk_4;
                    break 'blk_4;
                }
            }
            s_i32_0 = l4;
            vm.set_i32(*g_map.get(1usize).ok_or(WasmTrap::Unreachable)?, s_i32_0)?;
            s_i32_0 = l8;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i32_0 = l0;
    s_i32_1 = l1;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 5usize, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t6 = __call_res?;
    s_i32_0 = __t6;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn f_b218de90020d(vm: &mut SubstituteVm, dispatch: &'static crate::ModuleDispatch, g_map: &'static [usize], c_map: &'static [u32], p0: i32, p1: i32, p2: i32, p3: i32, p4: i32, p5: i32, p6: i32, p7: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = p2;
    let mut l3: i32 = p3;
    let mut l4: i32 = p4;
    let mut l5: i32 = p5;
    let mut l6: i32 = p6;
    let mut l7: i32 = p7;
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
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i32_2: i32 = 0i32;
    let mut s_i32_3: i32 = 0i32;
    let mut s_i32_4: i32 = 0i32;
    vm.deduct(host, 1)?;
    s_i32_0 = l0;
    s_i32_1 = l1;
    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
    l8 = s_i32_0;
    s_i32_0 = l0;
    s_i32_1 = l8;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l12 = s_i32_0;
    s_i32_0 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
    s_i32_1 = 1114372i32;
    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = l0;
    s_i32_1 = l12;
    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
    l10 = s_i32_0;
    s_i32_0 = l10;
    s_i32_1 = l8;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t1 = __call_res?;
    s_i32_0 = __t1;
    l10 = s_i32_0;
    s_i32_0 = l10;
    s_i32_1 = l8;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t2 = __call_res?;
    s_i32_0 = __t2;
    l12 = s_i32_0;
    s_i32_0 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
    s_i32_1 = 1114422i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = l10;
            s_i32_1 = l8;
            s_i32_2 = 1114422i32;
            s_i32_3 = 1114423i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_i32(vm, c_map, 2usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i32_0 = __t3;
            l10 = s_i32_0;
        }
    }
    s_i32_0 = l1;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
    vm.exit_call();
    let __t4 = __call_res?;
    s_i32_0 = __t4;
    l17 = s_i32_0;
    s_i32_0 = l17;
    s_i32_1 = l0;
    s_i32_2 = l10;
    s_i32_3 = l0;
    s_i32_2 = ops::i32_sub(s_i32_2, s_i32_3);
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 4usize, s_i32_0, s_i32_1, s_i32_2, host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l8;
    s_i32_1 = 4i32;
    s_i32_0 = ops::i32_sub(s_i32_0, s_i32_1);
    l9 = s_i32_0;
    s_i32_0 = l9;
    s_i32_1 = l8;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t5 = __call_res?;
    s_i32_0 = __t5;
    l12 = s_i32_0;
    s_i32_0 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
    s_i32_1 = 1114373i32;
    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
    'blk_3: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    'blk_4: {
        'blk_5: loop {
            vm.deduct(host, 1)?;
            s_i32_0 = l10;
            s_i32_1 = l9;
            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
            if s_i32_0 != 0 {
                break 'blk_4;
            }
            s_i32_0 = l10;
            s_i32_1 = l9;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t6 = __call_res?;
            s_i32_0 = __t6;
            l12 = s_i32_0;
            s_i32_0 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
            l13 = s_i32_0;
            s_i32_0 = l17;
            s_i32_1 = l10;
            s_i32_2 = l12;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 4usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l10;
            s_i32_1 = l12;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l10 = s_i32_0;
            s_i32_0 = l13;
            s_i32_1 = 1114759i32;
            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
            s_i32_1 = l13;
            s_i32_2 = 1115015i32;
            s_i32_1 = ops::i32_lt_u(s_i32_1, s_i32_2);
            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
            'blk_6: {
                if s_i32_0 != 0 {
                    s_i32_0 = l13;
                    s_i32_1 = 1114759i32;
                    s_i32_0 = ops::i32_sub(s_i32_0, s_i32_1);
                    l14 = s_i32_0;
                    s_i32_0 = l14;
                    s_i32_1 = 2i32;
                    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                    s_i32_1 = l14;
                    s_i32_2 = 4i32;
                    s_i32_1 = ops::i32_le_u(s_i32_1, s_i32_2);
                    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                    'blk_7: {
                        if s_i32_0 != 0 {
                            s_i32_0 = l10;
                            s_i32_1 = l9;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_0, s_i32_1, host);
                            vm.exit_call();
                            let __t7 = __call_res?;
                            s_i32_0 = __t7;
                            l11 = s_i32_0;
                            s_i32_0 = l17;
                            s_i32_1 = l10;
                            s_i32_2 = l11;
                            s_i32_3 = l10;
                            s_i32_2 = ops::i32_sub(s_i32_2, s_i32_3);
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 4usize, s_i32_0, s_i32_1, s_i32_2, host);
                            vm.exit_call();
                            __call_res?;
                            s_i32_0 = l11;
                            l10 = s_i32_0;
                        } else {
                            s_i32_0 = l14;
                            s_i32_1 = 16i32;
                            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                            s_i32_1 = l14;
                            s_i32_2 = 18i32;
                            s_i32_1 = ops::i32_eq(s_i32_1, s_i32_2);
                            s_i32_0 = ops::i32_or(s_i32_0, s_i32_1);
                            s_i32_1 = l14;
                            s_i32_2 = 210i32;
                            s_i32_1 = ops::i32_eq(s_i32_1, s_i32_2);
                            s_i32_0 = ops::i32_or(s_i32_0, s_i32_1);
                            'blk_8: {
                                if s_i32_0 != 0 {
                                    s_i32_0 = l10;
                                    s_i32_1 = l9;
                                    s_i32_2 = l3;
                                    s_i32_3 = l4;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_i32(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
                                    vm.exit_call();
                                    let __t8 = __call_res?;
                                    s_i32_0 = __t8;
                                    l15 = s_i32_0;
                                    s_i32_0 = vm.get_i32(*g_map.get(1usize).ok_or(WasmTrap::Unreachable)?)?;
                                    l10 = s_i32_0;
                                    s_i32_0 = l2;
                                    s_i32_1 = l15;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 6usize, s_i32_0, s_i32_1, host);
                                    vm.exit_call();
                                    let __t9 = __call_res?;
                                    s_i32_0 = __t9;
                                    l16 = s_i32_0;
                                    s_i32_0 = l17;
                                    s_i32_1 = l16;
                                    s_i64_1 = ops::i64_extend_i32_u(s_i32_1);
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_i64_ret_void(vm, c_map, 7usize, s_i32_0, s_i64_1, host);
                                    vm.exit_call();
                                    __call_res?;
                                } else {
                                    s_i32_0 = l14;
                                    s_i32_1 = 35i32;
                                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                                    s_i32_1 = l14;
                                    s_i32_2 = 36i32;
                                    s_i32_1 = ops::i32_eq(s_i32_1, s_i32_2);
                                    s_i32_0 = ops::i32_or(s_i32_0, s_i32_1);
                                    'blk_9: {
                                        if s_i32_0 != 0 {
                                            s_i32_0 = l10;
                                            s_i32_1 = l9;
                                            s_i32_2 = l6;
                                            s_i32_3 = l7;
                                            vm.enter_call()?;
                                            let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_i32(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
                                            vm.exit_call();
                                            let __t10 = __call_res?;
                                            s_i32_0 = __t10;
                                            l15 = s_i32_0;
                                            s_i32_0 = vm.get_i32(*g_map.get(1usize).ok_or(WasmTrap::Unreachable)?)?;
                                            l10 = s_i32_0;
                                            s_i32_0 = l5;
                                            s_i32_1 = l15;
                                            vm.enter_call()?;
                                            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 6usize, s_i32_0, s_i32_1, host);
                                            vm.exit_call();
                                            let __t11 = __call_res?;
                                            s_i32_0 = __t11;
                                            l16 = s_i32_0;
                                            s_i32_0 = l17;
                                            s_i32_1 = l16;
                                            s_i64_1 = ops::i64_extend_i32_u(s_i32_1);
                                            vm.enter_call()?;
                                            let __call_res = dispatch.call_p_i32_i64_ret_void(vm, c_map, 7usize, s_i32_0, s_i64_1, host);
                                            vm.exit_call();
                                            __call_res?;
                                        } else {
                                            s_i32_0 = l14;
                                            s_i32_1 = 12i32;
                                            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                                            s_i32_1 = l14;
                                            s_i32_2 = 13i32;
                                            s_i32_1 = ops::i32_eq(s_i32_1, s_i32_2);
                                            s_i32_0 = ops::i32_or(s_i32_0, s_i32_1);
                                            s_i32_1 = l14;
                                            s_i32_2 = 32i32;
                                            s_i32_1 = ops::i32_ge_u(s_i32_1, s_i32_2);
                                            s_i32_2 = l14;
                                            s_i32_3 = 34i32;
                                            s_i32_2 = ops::i32_le_u(s_i32_2, s_i32_3);
                                            s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
                                            s_i32_2 = l14;
                                            s_i32_3 = 63i32;
                                            s_i32_2 = ops::i32_ge_u(s_i32_2, s_i32_3);
                                            s_i32_3 = l14;
                                            s_i32_4 = 68i32;
                                            s_i32_3 = ops::i32_le_u(s_i32_3, s_i32_4);
                                            s_i32_2 = ops::i32_and(s_i32_2, s_i32_3);
                                            s_i32_1 = ops::i32_or(s_i32_1, s_i32_2);
                                            s_i32_0 = ops::i32_or(s_i32_0, s_i32_1);
                                            'blk_10: {
                                                if s_i32_0 != 0 {
                                                    s_i32_0 = l10;
                                                    s_i32_1 = l9;
                                                    s_i32_2 = l17;
                                                    vm.enter_call()?;
                                                    let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 8usize, s_i32_0, s_i32_1, s_i32_2, host);
                                                    vm.exit_call();
                                                    let __t12 = __call_res?;
                                                    s_i32_0 = __t12;
                                                    l10 = s_i32_0;
                                                } else {
                                                    s_i32_0 = l14;
                                                    s_i32_1 = 14i32;
                                                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                                                    'blk_11: {
                                                        if s_i32_0 != 0 {
                                                            s_i32_0 = l10;
                                                            s_i32_1 = l9;
                                                            s_i32_2 = 1114422i32;
                                                            s_i32_3 = 1114423i32;
                                                            vm.enter_call()?;
                                                            let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_i32(vm, c_map, 2usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
                                                            vm.exit_call();
                                                            let __t13 = __call_res?;
                                                            s_i32_0 = __t13;
                                                            l11 = s_i32_0;
                                                            s_i32_0 = l17;
                                                            s_i32_1 = l10;
                                                            s_i32_2 = l11;
                                                            s_i32_3 = l10;
                                                            s_i32_2 = ops::i32_sub(s_i32_2, s_i32_3);
                                                            vm.enter_call()?;
                                                            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 4usize, s_i32_0, s_i32_1, s_i32_2, host);
                                                            vm.exit_call();
                                                            __call_res?;
                                                            s_i32_0 = l11;
                                                            s_i32_1 = l9;
                                                            s_i32_2 = l17;
                                                            vm.enter_call()?;
                                                            let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 8usize, s_i32_0, s_i32_1, s_i32_2, host);
                                                            vm.exit_call();
                                                            let __t14 = __call_res?;
                                                            s_i32_0 = __t14;
                                                            l10 = s_i32_0;
                                                        } else {
                                                            s_i32_0 = l14;
                                                            s_i32_1 = 17i32;
                                                            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                                                            s_i32_1 = l14;
                                                            s_i32_2 = 19i32;
                                                            s_i32_1 = ops::i32_eq(s_i32_1, s_i32_2);
                                                            s_i32_0 = ops::i32_or(s_i32_0, s_i32_1);
                                                            'blk_12: {
                                                                if s_i32_0 != 0 {
                                                                    s_i32_0 = l10;
                                                                    s_i32_1 = l9;
                                                                    s_i32_2 = l17;
                                                                    vm.enter_call()?;
                                                                    let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 8usize, s_i32_0, s_i32_1, s_i32_2, host);
                                                                    vm.exit_call();
                                                                    let __t15 = __call_res?;
                                                                    s_i32_0 = __t15;
                                                                    l10 = s_i32_0;
                                                                    s_i32_0 = l10;
                                                                    s_i32_1 = l9;
                                                                    s_i32_2 = l17;
                                                                    vm.enter_call()?;
                                                                    let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 8usize, s_i32_0, s_i32_1, s_i32_2, host);
                                                                    vm.exit_call();
                                                                    let __t16 = __call_res?;
                                                                    s_i32_0 = __t16;
                                                                    l10 = s_i32_0;
                                                                } else {
                                                                    s_i32_0 = l14;
                                                                    s_i32_1 = 28i32;
                                                                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                                                                    'blk_13: {
                                                                        if s_i32_0 != 0 {
                                                                            s_i32_0 = l10;
                                                                            s_i32_1 = l9;
                                                                            vm.enter_call()?;
                                                                            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_0, s_i32_1, host);
                                                                            vm.exit_call();
                                                                            let __t17 = __call_res?;
                                                                            s_i32_0 = __t17;
                                                                            l11 = s_i32_0;
                                                                            s_i32_0 = l17;
                                                                            s_i32_1 = l10;
                                                                            s_i32_2 = l11;
                                                                            s_i32_3 = l10;
                                                                            s_i32_2 = ops::i32_sub(s_i32_2, s_i32_3);
                                                                            vm.enter_call()?;
                                                                            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 4usize, s_i32_0, s_i32_1, s_i32_2, host);
                                                                            vm.exit_call();
                                                                            __call_res?;
                                                                            s_i32_0 = l11;
                                                                            l10 = s_i32_0;
                                                                        } else {
                                                                            s_i32_0 = l14;
                                                                            s_i32_1 = 40i32;
                                                                            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                                                                            s_i32_1 = l14;
                                                                            s_i32_2 = 62i32;
                                                                            s_i32_1 = ops::i32_le_u(s_i32_1, s_i32_2);
                                                                            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                                                                            'blk_14: {
                                                                                if s_i32_0 != 0 {
                                                                                    s_i32_0 = l10;
                                                                                    s_i32_1 = l9;
                                                                                    s_i32_2 = l17;
                                                                                    vm.enter_call()?;
                                                                                    let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 8usize, s_i32_0, s_i32_1, s_i32_2, host);
                                                                                    vm.exit_call();
                                                                                    let __t18 = __call_res?;
                                                                                    s_i32_0 = __t18;
                                                                                    l10 = s_i32_0;
                                                                                    s_i32_0 = l10;
                                                                                    s_i32_1 = l9;
                                                                                    s_i32_2 = l17;
                                                                                    vm.enter_call()?;
                                                                                    let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 8usize, s_i32_0, s_i32_1, s_i32_2, host);
                                                                                    vm.exit_call();
                                                                                    let __t19 = __call_res?;
                                                                                    s_i32_0 = __t19;
                                                                                    l10 = s_i32_0;
                                                                                    s_i32_0 = l10;
                                                                                    s_i32_1 = l9;
                                                                                    s_i32_2 = l17;
                                                                                    vm.enter_call()?;
                                                                                    let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 8usize, s_i32_0, s_i32_1, s_i32_2, host);
                                                                                    vm.exit_call();
                                                                                    let __t20 = __call_res?;
                                                                                    s_i32_0 = __t20;
                                                                                    l10 = s_i32_0;
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
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    s_i32_0 = l13;
                    s_i32_1 = 1115271i32;
                    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                    s_i32_1 = l13;
                    s_i32_2 = 1115527i32;
                    s_i32_1 = ops::i32_lt_u(s_i32_1, s_i32_2);
                    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                    'blk_15: {
                        if s_i32_0 != 0 {
                            s_i32_0 = l13;
                            s_i32_1 = 1115271i32;
                            s_i32_0 = ops::i32_sub(s_i32_0, s_i32_1);
                            l14 = s_i32_0;
                            s_i32_0 = l14;
                            s_i32_1 = 8i32;
                            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                            s_i32_1 = l14;
                            s_i32_2 = 10i32;
                            s_i32_1 = ops::i32_eq(s_i32_1, s_i32_2);
                            s_i32_0 = ops::i32_or(s_i32_0, s_i32_1);
                            s_i32_1 = l14;
                            s_i32_2 = 12i32;
                            s_i32_1 = ops::i32_eq(s_i32_1, s_i32_2);
                            s_i32_2 = l14;
                            s_i32_3 = 14i32;
                            s_i32_2 = ops::i32_eq(s_i32_2, s_i32_3);
                            s_i32_1 = ops::i32_or(s_i32_1, s_i32_2);
                            s_i32_0 = ops::i32_or(s_i32_0, s_i32_1);
                            'blk_16: {
                                if s_i32_0 != 0 {
                                    s_i32_0 = l10;
                                    s_i32_1 = l9;
                                    s_i32_2 = l17;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 8usize, s_i32_0, s_i32_1, s_i32_2, host);
                                    vm.exit_call();
                                    let __t21 = __call_res?;
                                    s_i32_0 = __t21;
                                    l10 = s_i32_0;
                                    s_i32_0 = l10;
                                    s_i32_1 = l9;
                                    s_i32_2 = l17;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 8usize, s_i32_0, s_i32_1, s_i32_2, host);
                                    vm.exit_call();
                                    let __t22 = __call_res?;
                                    s_i32_0 = __t22;
                                    l10 = s_i32_0;
                                } else {
                                    s_i32_0 = l14;
                                    s_i32_1 = 9i32;
                                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                                    s_i32_1 = l14;
                                    s_i32_2 = 11i32;
                                    s_i32_1 = ops::i32_eq(s_i32_1, s_i32_2);
                                    s_i32_0 = ops::i32_or(s_i32_0, s_i32_1);
                                    s_i32_1 = l14;
                                    s_i32_2 = 13i32;
                                    s_i32_1 = ops::i32_eq(s_i32_1, s_i32_2);
                                    s_i32_2 = l14;
                                    s_i32_3 = 15i32;
                                    s_i32_2 = ops::i32_ge_u(s_i32_2, s_i32_3);
                                    s_i32_3 = l14;
                                    s_i32_4 = 17i32;
                                    s_i32_3 = ops::i32_le_u(s_i32_3, s_i32_4);
                                    s_i32_2 = ops::i32_and(s_i32_2, s_i32_3);
                                    s_i32_1 = ops::i32_or(s_i32_1, s_i32_2);
                                    s_i32_0 = ops::i32_or(s_i32_0, s_i32_1);
                                    'blk_17: {
                                        if s_i32_0 != 0 {
                                            s_i32_0 = l10;
                                            s_i32_1 = l9;
                                            s_i32_2 = l17;
                                            vm.enter_call()?;
                                            let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 8usize, s_i32_0, s_i32_1, s_i32_2, host);
                                            vm.exit_call();
                                            let __t23 = __call_res?;
                                            s_i32_0 = __t23;
                                            l10 = s_i32_0;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            continue 'blk_5;
            break 'blk_5;
        }
    }
    s_i32_0 = l17;
    s_i32_1 = 1114373i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 9usize, s_i32_0, s_i32_1, host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l17;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    vm.set_i32(*g_map.get(2usize).ok_or(WasmTrap::Unreachable)?, s_i32_0)?;
    s_i32_0 = l17;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn f_8e3a351d9bba(vm: &mut SubstituteVm, dispatch: &'static crate::ModuleDispatch, g_map: &'static [usize], c_map: &'static [u32], p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let mut l26: i32 = 0i32;
    let mut l27: i32 = 0i32;
    let mut l28: i32 = 0i32;
    let mut l29: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i32_2: i32 = 0i32;
    let mut s_i64_2: i64 = 0i64;
    let mut s_i32_3: i32 = 0i32;
    let mut s_i32_4: i32 = 0i32;
    let mut s_i32_5: i32 = 0i32;
    let mut s_i32_6: i32 = 0i32;
    let mut s_i32_7: i32 = 0i32;
    vm.deduct(host, 1)?;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 28u32)?;
    s_i32_1 = 0i32;
    s_i32_0 = ops::i32_gt_u(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 28u32)?;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_sub(s_i32_0, s_i32_1);
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = 98i32;
    s_i32_1 = l1;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 0usize, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l21 = s_i32_0;
    s_i32_0 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
    l22 = s_i32_0;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 20u32)?;
    l2 = s_i32_0;
    s_i32_0 = l2;
    s_i32_1 = l0;
    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 24u32)?;
    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
    l3 = s_i32_0;
    s_i32_0 = l2;
    s_i32_1 = l3;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t1 = __call_res?;
    s_i32_0 = __t1;
    l6 = s_i32_0;
    s_i32_0 = vm.get_i32(*g_map.get(1usize).ok_or(WasmTrap::Unreachable)?)?;
    s_i32_1 = 1116039i32;
    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
    'blk_2: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = l2;
    s_i32_1 = l6;
    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
    l4 = s_i32_0;
    s_i32_0 = l3;
    s_i32_1 = 4i32;
    s_i32_0 = ops::i32_sub(s_i32_0, s_i32_1);
    l5 = s_i32_0;
    s_i32_0 = l5;
    s_i32_1 = l3;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t2 = __call_res?;
    s_i32_0 = __t2;
    l6 = s_i32_0;
    s_i32_0 = vm.get_i32(*g_map.get(1usize).ok_or(WasmTrap::Unreachable)?)?;
    s_i32_1 = 1116040i32;
    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
    'blk_3: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = l4;
    s_i32_1 = l5;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t3 = __call_res?;
    s_i32_0 = __t3;
    l6 = s_i32_0;
    s_i32_0 = vm.get_i32(*g_map.get(1usize).ok_or(WasmTrap::Unreachable)?)?;
    s_i32_1 = 1114372i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_4: {
        if s_i32_0 != 0 {
            s_i32_0 = l21;
            s_i32_1 = l22;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 2usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t4 = __call_res?;
            s_i32_0 = __t4;
            l25 = s_i32_0;
            s_i32_0 = l25;
            s_i32_1 = l4;
            s_i32_2 = l5;
            s_i32_3 = l4;
            s_i32_2 = ops::i32_sub(s_i32_2, s_i32_3);
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 3usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l0;
            s_i32_1 = l25;
            s_i32_2 = 1i32;
            s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
            ops::i32_store(vm, 0, s_i32_0, 28u32, s_i32_1)?;
            s_i32_0 = l25;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = vm.get_i32(*g_map.get(1usize).ok_or(WasmTrap::Unreachable)?)?;
    s_i32_1 = 1114671i32;
    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
    'blk_5: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = l4;
    s_i32_1 = l6;
    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
    l4 = s_i32_0;
    s_i32_0 = l5;
    s_i32_1 = 4i32;
    s_i32_0 = ops::i32_sub(s_i32_0, s_i32_1);
    l5 = s_i32_0;
    s_i32_0 = l5;
    s_i32_1 = l5;
    s_i32_2 = 4i32;
    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t5 = __call_res?;
    s_i32_0 = __t5;
    l6 = s_i32_0;
    s_i32_0 = vm.get_i32(*g_map.get(1usize).ok_or(WasmTrap::Unreachable)?)?;
    s_i32_1 = 1114672i32;
    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
    'blk_6: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = l4;
    s_i32_1 = l5;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t6 = __call_res?;
    s_i32_0 = __t6;
    l6 = s_i32_0;
    s_i32_0 = vm.get_i32(*g_map.get(1usize).ok_or(WasmTrap::Unreachable)?)?;
    s_i32_1 = 1114118i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_7: {
        if s_i32_0 != 0 {
            s_i32_0 = l4;
            s_i32_1 = l5;
            s_i32_2 = 1114118i32;
            s_i32_3 = 1114119i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
            vm.exit_call();
            let __t7 = __call_res?;
            s_i32_0 = __t7;
            l4 = s_i32_0;
        }
    }
    s_i32_0 = l4;
    s_i32_1 = l5;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 5usize, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t8 = __call_res?;
    s_i32_0 = __t8;
    l4 = s_i32_0;
    s_i32_0 = l4;
    s_i32_1 = l5;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 5usize, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t9 = __call_res?;
    s_i32_0 = __t9;
    l4 = s_i32_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 6usize, s_i32_0, host);
    vm.exit_call();
    let __t10 = __call_res?;
    s_i32_0 = __t10;
    l8 = s_i32_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 6usize, s_i32_0, host);
    vm.exit_call();
    let __t11 = __call_res?;
    s_i32_0 = __t11;
    l9 = s_i32_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 6usize, s_i32_0, host);
    vm.exit_call();
    let __t12 = __call_res?;
    s_i32_0 = __t12;
    l10 = s_i32_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 6usize, s_i32_0, host);
    vm.exit_call();
    let __t13 = __call_res?;
    s_i32_0 = __t13;
    l11 = s_i32_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 6usize, s_i32_0, host);
    vm.exit_call();
    let __t14 = __call_res?;
    s_i32_0 = __t14;
    l12 = s_i32_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 6usize, s_i32_0, host);
    vm.exit_call();
    let __t15 = __call_res?;
    s_i32_0 = __t15;
    l13 = s_i32_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 6usize, s_i32_0, host);
    vm.exit_call();
    let __t16 = __call_res?;
    s_i32_0 = __t16;
    l14 = s_i32_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 6usize, s_i32_0, host);
    vm.exit_call();
    let __t17 = __call_res?;
    s_i32_0 = __t17;
    l15 = s_i32_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 6usize, s_i32_0, host);
    vm.exit_call();
    let __t18 = __call_res?;
    s_i32_0 = __t18;
    l16 = s_i32_0;
    s_i32_0 = 4i32;
    s_i64_1 = 1852399981i64;
    s_i64_2 = 0i64;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i64_i64_ret_i32(vm, c_map, 7usize, s_i32_0, s_i64_1, s_i64_2, host);
    vm.exit_call();
    let __t19 = __call_res?;
    s_i32_0 = __t19;
    l23 = s_i32_0;
    s_i32_0 = -1i32;
    l26 = s_i32_0;
    'blk_8: {
        'blk_9: loop {
            vm.deduct(host, 1)?;
            s_i32_0 = l4;
            s_i32_1 = l5;
            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
            if s_i32_0 != 0 {
                break 'blk_8;
            }
            s_i32_0 = l4;
            s_i32_1 = l5;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t20 = __call_res?;
            s_i32_0 = __t20;
            l6 = s_i32_0;
            s_i32_0 = vm.get_i32(*g_map.get(1usize).ok_or(WasmTrap::Unreachable)?)?;
            s_i32_1 = 1114626i32;
            s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
            'blk_10: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = l4;
            s_i32_1 = l6;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l4 = s_i32_0;
            s_i32_0 = l4;
            s_i32_1 = l5;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 8usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t21 = __call_res?;
            s_i32_0 = __t21;
            l4 = s_i32_0;
            s_i32_0 = vm.get_i32(*g_map.get(2usize).ok_or(WasmTrap::Unreachable)?)?;
            l17 = s_i32_0;
            s_i32_0 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
            l18 = s_i32_0;
            s_i32_0 = vm.get_i32(*g_map.get(3usize).ok_or(WasmTrap::Unreachable)?)?;
            l19 = s_i32_0;
            s_i32_0 = l4;
            s_i32_1 = l5;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t22 = __call_res?;
            s_i32_0 = __t22;
            l6 = s_i32_0;
            s_i32_0 = vm.get_i32(*g_map.get(1usize).ok_or(WasmTrap::Unreachable)?)?;
            s_i32_1 = 1114424i32;
            s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
            'blk_11: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = l4;
            s_i32_1 = l6;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l4 = s_i32_0;
            s_i32_0 = l4;
            s_i32_1 = l5;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t23 = __call_res?;
            s_i32_0 = __t23;
            l6 = s_i32_0;
            s_i32_0 = vm.get_i32(*g_map.get(1usize).ok_or(WasmTrap::Unreachable)?)?;
            l7 = s_i32_0;
            s_i32_0 = l7;
            s_i32_1 = 1114118i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_12: {
                if s_i32_0 != 0 {
                    s_i32_0 = l4;
                    s_i32_1 = l5;
                    s_i32_2 = 1114118i32;
                    s_i32_3 = 1114119i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
                    vm.exit_call();
                    let __t24 = __call_res?;
                    s_i32_0 = __t24;
                    l20 = s_i32_0;
                    s_i32_0 = l9;
                    s_i32_1 = l17;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 9usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l10;
                    s_i32_1 = l18;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 9usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l8;
                    s_i32_1 = l17;
                    s_i32_2 = l18;
                    s_i32_3 = l19;
                    s_i32_3 = ops::i32_load(vm, 0, s_i32_3, 0u32)?;
                    s_i32_4 = l19;
                    s_i32_4 = ops::i32_load(vm, 0, s_i32_4, 4u32)?;
                    s_i32_5 = l4;
                    s_i32_6 = l20;
                    s_i32_7 = l4;
                    s_i32_6 = ops::i32_sub(s_i32_6, s_i32_7);
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_i32_i32_i32_ret_i32(vm, c_map, 10usize, s_i32_1, s_i32_2, s_i32_3, s_i32_4, s_i32_5, s_i32_6, host);
                    vm.exit_call();
                    let __t25 = __call_res?;
                    s_i32_1 = __t25;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 9usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l20;
                    l4 = s_i32_0;
                } else {
                    s_i32_0 = l7;
                    s_i32_1 = 1114372i32;
                    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
                    'blk_13: {
                        if s_i32_0 != 0 {
                            return Err(WasmTrap::Unreachable);
                        }
                    }
                    s_i32_0 = l4;
                    s_i32_1 = l5;
                    s_i32_2 = 1114372i32;
                    s_i32_3 = 1114373i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
                    vm.exit_call();
                    let __t26 = __call_res?;
                    s_i32_0 = __t26;
                    l20 = s_i32_0;
                    s_i32_0 = l12;
                    s_i32_1 = l17;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 9usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l13;
                    s_i32_1 = l18;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 9usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l17;
                    s_i32_1 = l18;
                    s_i32_2 = l23;
                    s_i32_3 = 4i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_i32(vm, c_map, 11usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
                    vm.exit_call();
                    let __t27 = __call_res?;
                    s_i32_0 = __t27;
                    l24 = s_i32_0;
                    s_i32_0 = l24;
                    'blk_14: {
                        if s_i32_0 != 0 {
                            s_i32_0 = l21;
                            l17 = s_i32_0;
                            s_i32_0 = l22;
                            l18 = s_i32_0;
                        } else {
                        }
                    }
                    s_i32_0 = l17;
                    s_i32_1 = l18;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 12usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t28 = __call_res?;
                    s_i32_0 = __t28;
                    l25 = s_i32_0;
                    s_i32_0 = l25;
                    s_i32_1 = 0i32;
                    s_i32_0 = ops::i32_lt_s(s_i32_0, s_i32_1);
                    'blk_15: {
                        if s_i32_0 != 0 {
                            s_i32_0 = l17;
                            s_i32_1 = l18;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 2usize, s_i32_0, s_i32_1, host);
                            vm.exit_call();
                            let __t29 = __call_res?;
                            s_i32_0 = __t29;
                            l25 = s_i32_0;
                            s_i32_0 = l14;
                            s_i32_1 = l25;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 9usize, s_i32_0, s_i32_1, host);
                            vm.exit_call();
                            __call_res?;
                            s_i32_0 = l15;
                            s_i32_1 = l4;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 9usize, s_i32_0, s_i32_1, host);
                            vm.exit_call();
                            __call_res?;
                            s_i32_0 = l16;
                            s_i32_1 = l20;
                            s_i32_2 = l4;
                            s_i32_1 = ops::i32_sub(s_i32_1, s_i32_2);
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 9usize, s_i32_0, s_i32_1, host);
                            vm.exit_call();
                            __call_res?;
                        }
                    }
                    s_i32_0 = l24;
                    'blk_16: {
                        if s_i32_0 != 0 {
                            s_i32_0 = l25;
                            l26 = s_i32_0;
                        }
                    }
                    s_i32_0 = l11;
                    s_i32_1 = l25;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 9usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l20;
                    l4 = s_i32_0;
                }
            }
            s_i32_0 = l4;
            s_i32_1 = l5;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t30 = __call_res?;
            s_i32_0 = __t30;
            l6 = s_i32_0;
            s_i32_0 = vm.get_i32(*g_map.get(1usize).ok_or(WasmTrap::Unreachable)?)?;
            s_i32_1 = 1114627i32;
            s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
            'blk_17: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = l4;
            s_i32_1 = l6;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l4 = s_i32_0;
            continue 'blk_9;
            break 'blk_9;
        }
    }
    s_i32_0 = l14;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 13usize, s_i32_0, host);
    vm.exit_call();
    let __t31 = __call_res?;
    s_i32_0 = __t31;
    l28 = s_i32_0;
    s_i32_0 = 0i32;
    l29 = s_i32_0;
    'blk_18: {
        'blk_19: loop {
            vm.deduct(host, 1)?;
            s_i32_0 = l29;
            s_i32_1 = l28;
            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
            if s_i32_0 != 0 {
                break 'blk_18;
            }
            s_i32_0 = l15;
            s_i32_1 = l29;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 14usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t32 = __call_res?;
            s_i32_0 = __t32;
            s_i32_1 = l16;
            s_i32_2 = l29;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 14usize, s_i32_1, s_i32_2, host);
            vm.exit_call();
            let __t33 = __call_res?;
            s_i32_1 = __t33;
            s_i32_2 = l11;
            s_i32_3 = l12;
            s_i32_4 = l13;
            s_i32_5 = l8;
            s_i32_6 = l9;
            s_i32_7 = l10;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_i32_i32_i32_i32_i32_ret_i32(vm, c_map, 15usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, s_i32_4, s_i32_5, s_i32_6, s_i32_7, host);
            vm.exit_call();
            let __t34 = __call_res?;
            s_i32_0 = __t34;
            l27 = s_i32_0;
            s_i32_0 = l14;
            s_i32_1 = l29;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 14usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t35 = __call_res?;
            s_i32_0 = __t35;
            s_i32_1 = l27;
            s_i32_2 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_ret_void(vm, c_map, 3usize, s_i32_0, s_i32_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l29;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l29 = s_i32_0;
            continue 'blk_19;
            break 'blk_19;
        }
    }
    s_i32_0 = l26;
    s_i32_1 = 0i32;
    s_i32_0 = ops::i32_lt_s(s_i32_0, s_i32_1);
    'blk_20: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = l0;
    s_i32_1 = l26;
    s_i32_2 = 1i32;
    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
    ops::i32_store(vm, 0, s_i32_0, 28u32, s_i32_1)?;
    s_i32_0 = l26;
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
const FN_30_GLOBALS: &[usize] = &[4usize, 9usize, 7usize, 8usize];
const FN_30_CALLEES: &[u32] = &[2u32, 4u32, 2u32, 0u32, 0u32, 5u32];
const FN_31_GLOBALS: &[usize] = &[4usize, 9usize, 8usize];
const FN_31_CALLEES: &[u32] = &[2u32, 3u32, 1u32, 1u32, 0u32, 2u32, 0u32, 0u32, 0u32, 4u32];
const FN_32_GLOBALS: &[usize] = &[11usize];
const FN_32_CALLEES: &[u32] = &[2u32, 0u32, 0u32];
const FN_33_GLOBALS: &[usize] = &[11usize];
const FN_33_CALLEES: &[u32] = &[6u32, 2u32, 0u32, 2u32];
const FN_34_CALLEES: &[u32] = &[6u32, 1u32, 4u32, 1u32, 0u32];
const FN_35_GLOBALS: &[usize] = &[12usize];
const FN_35_CALLEES: &[u32] = &[2u32, 0u32, 0u32];
const FN_36_GLOBALS: &[usize] = &[12usize];
const FN_36_CALLEES: &[u32] = &[7u32, 2u32, 0u32, 2u32];
const FN_37_GLOBALS: &[usize] = &[12usize];
const FN_37_CALLEES: &[u32] = &[0u32];
const FN_38_GLOBALS: &[usize] = &[10usize, 9usize];
const FN_38_CALLEES: &[u32] = &[2u32, 0u32];
const FN_39_GLOBALS: &[usize] = &[8usize, 4usize, 7usize, 9usize];
const FN_39_CALLEES: &[u32] = &[1u32, 2u32, 8u32, 2u32, 1u32, 3u32, 1u32, 0u32, 4u32, 2u32, 0u32, 0u32, 7u32, 2u32, 0u32, 0u32];
const FN_40_GLOBALS: &[usize] = &[9usize];
const FN_40_CALLEES: &[u32] = &[0u32, 9u32];
const FN_41_GLOBALS: &[usize] = &[4usize, 9usize];
const FN_41_CALLEES: &[u32] = &[2u32, 3u32, 5u32, 1u32, 0u32];
const FN_42_GLOBALS: &[usize] = &[9usize];
const FN_42_CALLEES: &[u32] = &[0u32, 2u32, 9u32, 5u32];
const FN_43_CALLEES: &[u32] = &[7u32, 0u32];
const FN_44_GLOBALS: &[usize] = &[13usize];
const FN_44_CALLEES: &[u32] = &[1u32, 0u32, 7u32];
const FN_45_GLOBALS: &[usize] = &[14usize];
const FN_45_CALLEES: &[u32] = &[0u32, 2u32];
const FN_48_CALLEES: &[u32] = &[4u32];
const FN_51_GLOBALS: &[usize] = &[14usize];
const FN_51_CALLEES: &[u32] = &[2u32, 0u32, 0u32, 2u32];
const FN_52_GLOBALS: &[usize] = &[14usize];
const FN_52_CALLEES: &[u32] = &[2u32, 0u32, 0u32, 2u32];
const FN_53_CALLEES: &[u32] = &[10u32];
const FN_54_GLOBALS: &[usize] = &[8usize];
const FN_54_CALLEES: &[u32] = &[1u32, 1u32, 1u32, 4u32];
const FN_55_GLOBALS: &[usize] = &[8usize];
const FN_55_CALLEES: &[u32] = &[1u32, 1u32, 1u32, 1u32, 4u32];
const FN_56_GLOBALS: &[usize] = &[6usize, 8usize, 5usize];
const FN_56_CALLEES: &[u32] = &[3u32, 1u32];
const FN_57_GLOBALS: &[usize] = &[8usize];
const FN_57_CALLEES: &[u32] = &[2u32, 3u32, 7u32];
const FN_58_GLOBALS: &[usize] = &[9usize];
const FN_58_CALLEES: &[u32] = &[2u32, 5u32, 7u32];
const FN_59_GLOBALS: &[usize] = &[2usize, 3usize, 9usize];
const FN_59_CALLEES: &[u32] = &[0u32, 5u32];
const FN_60_GLOBALS: &[usize] = &[9usize, 8usize];
const FN_60_CALLEES: &[u32] = &[3u32, 0u32, 1u32, 1u32, 2u32, 2u32, 2u32, 9u32];
const FN_61_CALLEES: &[u32] = &[1u32, 0u32, 0u32];
const FN_62_CALLEES: &[u32] = &[10u32, 1u32, 1u32, 1u32, 0u32];
const FN_63_CALLEES: &[u32] = &[1u32];
const FN_64_GLOBALS: &[usize] = &[8usize];
const FN_64_CALLEES: &[u32] = &[0u32, 3u32, 8u32, 11u32, 5u32, 1u32, 2u32, 2u32, 3u32, 15u32, 1u32];
const FN_67_GLOBALS: &[usize] = &[9usize, 8usize, 17usize];
const FN_67_CALLEES: &[u32] = &[3u32, 0u32, 12u32, 13u32, 11u32, 1u32, 2u32, 2u32, 1u32, 10u32, 9u32, 2u32, 5u32, 8u32, 14u32, 9u32, 10u32];
const FN_68_GLOBALS: &[usize] = &[8usize, 9usize, 15usize, 22usize];
const FN_68_CALLEES: &[u32] = &[1u32, 0u32, 1u32, 3u32, 6u32, 2u32, 9u32, 5u32, 2u32, 8u32, 11u32, 2u32, 15u32, 14u32];
const FN_69_CALLEES: &[u32] = &[1u32];
const FN_70_CALLEES: &[u32] = &[1u32];
const FN_71_GLOBALS: &[usize] = &[14usize, 15usize, 16usize];
const FN_71_CALLEES: &[u32] = &[1u32, 4u32, 0u32, 3u32, 3u32];
const FN_72_GLOBALS: &[usize] = &[16usize];
const FN_72_CALLEES: &[u32] = &[0u32, 0u32];
const FN_73_GLOBALS: &[usize] = &[16usize];
const FN_73_CALLEES: &[u32] = &[0u32, 0u32];
const FN_74_GLOBALS: &[usize] = &[16usize];
const FN_74_CALLEES: &[u32] = &[2u32, 0u32, 0u32];
const FN_75_GLOBALS: &[usize] = &[16usize, 15usize];
const FN_75_CALLEES: &[u32] = &[0u32, 2u32, 0u32];
const FN_76_GLOBALS: &[usize] = &[16usize, 15usize];
const FN_76_CALLEES: &[u32] = &[0u32, 2u32, 0u32];
const FN_77_GLOBALS: &[usize] = &[16usize];
const FN_77_CALLEES: &[u32] = &[3u32, 0u32, 2u32, 0u32, 3u32, 4u32];
const FN_78_GLOBALS: &[usize] = &[16usize];
const FN_78_CALLEES: &[u32] = &[4u32, 0u32, 2u32, 0u32, 3u32];
const FN_79_CALLEES: &[u32] = &[10u32, 0u32, 11u32, 4u32, 8u32, 5u32];
const FN_80_GLOBALS: &[usize] = &[14usize, 15usize];
const FN_80_CALLEES: &[u32] = &[0u32];
const FN_81_GLOBALS: &[usize] = &[14usize, 15usize];
const FN_81_CALLEES: &[u32] = &[0u32];
const FN_82_GLOBALS: &[usize] = &[14usize];
const FN_82_CALLEES: &[u32] = &[0u32, 12u32];
const FN_83_GLOBALS: &[usize] = &[14usize, 15usize];
const FN_83_CALLEES: &[u32] = &[0u32, 0u32, 4u32, 5u32];
const FN_84_CALLEES: &[u32] = &[2u32, 0u32];
const FN_85_GLOBALS: &[usize] = &[16usize];
const FN_85_CALLEES: &[u32] = &[4u32, 3u32, 0u32, 2u32, 0u32];
const FN_86_GLOBALS: &[usize] = &[16usize, 15usize, 8usize];
const FN_86_CALLEES: &[u32] = &[2u32, 0u32, 3u32, 16u32, 12u32, 0u32, 0u32, 14u32, 8u32, 2u32, 3u32, 12u32, 3u32, 1u32, 1u32, 15u32, 1u32, 4u32, 3u32, 1u32, 3u32, 5u32, 8u32];
const FN_87_GLOBALS: &[usize] = &[16usize];
const FN_87_CALLEES: &[u32] = &[0u32, 2u32, 0u32];
const FN_88_GLOBALS: &[usize] = &[16usize];
const FN_88_CALLEES: &[u32] = &[1u32, 0u32, 2u32, 0u32, 3u32];
const FN_89_CALLEES: &[u32] = &[2u32, 0u32];
const FN_90_GLOBALS: &[usize] = &[16usize];
const FN_90_CALLEES: &[u32] = &[0u32, 2u32, 0u32, 6u32];
const FN_91_GLOBALS: &[usize] = &[16usize];
const FN_91_CALLEES: &[u32] = &[0u32, 0u32, 2u32, 0u32, 3u32];
const FN_92_CALLEES: &[u32] = &[0u32, 1u32];
const FN_93_CALLEES: &[u32] = &[2u32, 0u32, 2u32];
const FN_94_CALLEES: &[u32] = &[2u32, 2u32];
const FN_95_CALLEES: &[u32] = &[4u32];
const FN_96_CALLEES: &[u32] = &[4u32, 0u32];
const FN_97_CALLEES: &[u32] = &[4u32];
const FN_98_CALLEES: &[u32] = &[4u32, 0u32];
const FN_99_CALLEES: &[u32] = &[4u32, 1u32];
const FN_100_CALLEES: &[u32] = &[4u32, 1u32];
const FN_101_CALLEES: &[u32] = &[4u32, 0u32];
const FN_102_CALLEES: &[u32] = &[4u32, 0u32];
const FN_103_GLOBALS: &[usize] = &[8usize];
const FN_103_CALLEES: &[u32] = &[2u32, 1u32, 4u32, 0u32, 0u32, 0u32];
const FN_104_GLOBALS: &[usize] = &[11usize, 12usize, 8usize];
const FN_104_CALLEES: &[u32] = &[1u32, 4u32, 2u32, 0u32, 0u32, 0u32];
const FN_105_GLOBALS: &[usize] = &[17usize, 18usize, 19usize, 20usize, 21usize, 22usize, 23usize, 24usize];
const FN_105_CALLEES: &[u32] = &[0u32, 1u32, 8u32];
const FN_106_GLOBALS: &[usize] = &[17usize, 18usize, 20usize, 19usize, 21usize, 8usize];
const FN_106_CALLEES: &[u32] = &[4u32, 13u32, 0u32, 5u32, 17u32, 5u32, 6u32, 6u32, 7u32, 1u32, 14u32, 2u32];
const FN_107_CALLEES: &[u32] = &[7u32, 1u32];
const FN_108_CALLEES: &[u32] = &[6u32, 7u32, 1u32];
const FN_109_CALLEES: &[u32] = &[1u32];
const FN_110_CALLEES: &[u32] = &[1u32];
const FN_111_GLOBALS: &[usize] = &[14usize];
const FN_111_CALLEES: &[u32] = &[12u32, 6u32, 0u32, 6u32, 7u32, 0u32, 7u32, 1u32, 5u32, 15u32];
const FN_112_GLOBALS: &[usize] = &[14usize, 8usize, 15usize];
const FN_112_CALLEES: &[u32] = &[12u32, 0u32, 1u32, 8u32, 13u32, 0u32, 5u32, 5u32, 6u32, 6u32, 7u32, 4u32, 5u32, 17u32, 1u32, 2u32, 8u32, 4u32, 14u32, 2u32];
const FN_113_GLOBALS: &[usize] = &[25usize];
const FN_114_CALLEES: &[u32] = &[2u32, 0u32, 0u32];
const FN_115_CALLEES: &[u32] = &[2u32, 0u32, 0u32];
const FN_116_CALLEES: &[u32] = &[2u32, 0u32, 0u32];
const FN_117_CALLEES: &[u32] = &[10u32, 0u32, 2u32];
const FN_118_CALLEES: &[u32] = &[2u32, 1u32, 0u32, 2u32];
const FN_119_CALLEES: &[u32] = &[10u32, 11u32, 2u32, 0u32, 16u32];
const FN_120_CALLEES: &[u32] = &[0u32, 2u32, 0u32];
const FN_121_CALLEES: &[u32] = &[16u32, 2u32, 0u32, 11u32, 1u32, 4u32, 0u32, 10u32, 9u32];
const FN_122_GLOBALS: &[usize] = &[14usize, 15usize];
const FN_122_CALLEES: &[u32] = &[10u32, 14u32, 2u32, 0u32, 0u32, 11u32, 5u32, 4u32, 8u32, 4u32, 8u32, 2u32, 9u32, 6u32, 2u32, 1u32, 2u32];
const FN_123_GLOBALS: &[usize] = &[17usize, 20usize];
const FN_123_CALLEES: &[u32] = &[5u32];
const FN_124_CALLEES: &[u32] = &[3u32, 5u32, 1u32, 6u32, 6u32];
const FN_125_CALLEES: &[u32] = &[5u32, 3u32, 1u32, 7u32];
const FN_126_GLOBALS: &[usize] = &[14usize];
const FN_126_CALLEES: &[u32] = &[4u32, 0u32, 4u32, 17u32, 7u32, 5u32, 2u32, 3u32];
const FN_127_GLOBALS: &[usize] = &[21usize, 12usize, 8usize];
const FN_127_CALLEES: &[u32] = &[2u32, 1u32, 0u32, 14u32, 3u32, 3u32, 10u32, 11u32, 2u32, 4u32, 9u32, 0u32, 0u32, 0u32, 12u32, 17u32, 7u32, 5u32, 1u32, 8u32, 12u32, 1u32, 14u32, 6u32];
const FN_128_GLOBALS: &[usize] = &[15usize, 14usize, 17usize, 20usize];
const FN_128_CALLEES: &[u32] = &[7u32, 3u32, 2u32, 4u32, 17u32, 5u32, 6u32, 1u32, 10u32, 14u32, 3u32, 12u32, 4u32, 5u32, 2u32, 1u32, 0u32, 2u32, 3u32, 0u32, 11u32, 7u32, 5u32, 4u32, 8u32, 4u32, 2u32, 5u32, 7u32, 8u32, 20u32, 2u32, 0u32, 5u32, 13u32, 19u32, 9u32, 6u32, 2u32, 6u32, 6u32, 0u32, 1u32, 0u32, 6u32, 7u32, 15u32];
const FN_129_GLOBALS: &[usize] = &[17usize, 8usize, 22usize, 24usize, 20usize];
const FN_129_CALLEES: &[u32] = &[3u32, 13u32, 0u32, 5u32, 1u32, 2u32, 0u32, 1u32, 0u32, 2u32, 11u32, 1u32, 3u32, 5u32, 17u32, 17u32, 14u32, 6u32, 14u32, 2u32];
const FN_130_CALLEES: &[u32] = &[10u32, 14u32, 1u32, 6u32, 16u32, 2u32, 0u32, 11u32];
const FN_131_CALLEES: &[u32] = &[1u32, 1u32, 0u32, 17u32, 14u32, 5u32];
const FN_132_GLOBALS: &[usize] = &[22usize, 23usize, 17usize, 20usize, 16usize, 24usize, 8usize];
const FN_132_CALLEES: &[u32] = &[13u32, 5u32, 6u32, 6u32, 7u32, 17u32, 2u32, 0u32, 6u32, 14u32, 2u32];
const FN_133_GLOBALS: &[usize] = &[16usize];
const FN_133_CALLEES: &[u32] = &[2u32, 0u32, 1u32];
const FN_134_GLOBALS: &[usize] = &[16usize, 22usize, 24usize, 8usize];
const FN_134_CALLEES: &[u32] = &[0u32, 7u32, 2u32, 10u32, 0u32, 0u32, 8u32, 13u32, 5u32, 4u32, 17u32, 7u32, 1u32, 14u32, 2u32];
const FN_135_GLOBALS: &[usize] = &[1usize, 0usize, 2usize, 3usize, 10usize, 11usize, 12usize, 13usize, 14usize, 15usize, 16usize, 17usize, 22usize, 25usize, 8usize];
const FN_135_CALLEES: &[u32] = &[1u32, 2u32, 1u32, 0u32, 11u32, 2u32, 5u32, 4u32, 6u32, 7u32, 8u32, 9u32, 16u32];
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
    crate::Bound_p_i32_i32_i32_i32_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_517bbdea3a61, globals: FN_90_GLOBALS, callees: FN_90_CALLEES },
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
    crate::Bound_p_i32_i32_i32_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_e778f0ae372c, globals: FN_87_GLOBALS, callees: FN_87_CALLEES },
    crate::Bound_p_i32_i32_i32_i32_i32_i32_ret_i32 { func: crate::c_85899_87792::ChunkFns::f_fb262af913b2, globals: FN_91_GLOBALS, callees: FN_91_CALLEES },
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
    crate::Bound_p_i32_i32_i32_i32_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_e3d239a807a4, globals: &[], callees: FN_121_CALLEES },
    crate::Bound_p_i32_i32_i32_i32_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_0eb69cfbacde, globals: FN_129_GLOBALS, callees: FN_129_CALLEES },
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
    crate::Bound_p_i32_i32_i32_i32_i32_ret_i32 { func: crate::c_85899_87792::ChunkFns::f_78eba914826e, globals: FN_88_GLOBALS, callees: FN_88_CALLEES },
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
    crate::Bound_p_i32_i32_i32_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_07908887d2ca, globals: &[], callees: FN_102_CALLEES },
    crate::Bound_p_i32_i32_i32_i32_i32_ret_void { func: crate::c_85899_79292_1::ChunkFns::f_58deb2a85fe1, globals: &[], callees: FN_130_CALLEES },
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
    crate::Bound_p_i32_i32_i32_i32_ret_i32 { func: crate::c_85899_97792::ChunkFns::f_e1ee448cdcd2, globals: FN_30_GLOBALS, callees: FN_30_CALLEES },
    crate::Bound_p_i32_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_68ca86b09415, globals: FN_75_GLOBALS, callees: FN_75_CALLEES },
    crate::Bound_p_i32_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_152ec9a5d530, globals: FN_76_GLOBALS, callees: FN_76_CALLEES },
    crate::Bound_p_i32_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_d7490f9cf842, globals: &[], callees: FN_93_CALLEES },
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
    crate::Bound_p_i32_i32_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_5d566823fb6c, globals: &[], callees: FN_98_CALLEES },
    crate::Bound_p_i32_i32_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_293c7d635704, globals: &[], callees: FN_109_CALLEES },
    crate::Bound_p_i32_i32_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_ff67f9ea2c5b, globals: &[], callees: FN_110_CALLEES },
    crate::Bound_p_i32_i32_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_1dac86d11457, globals: &[], callees: FN_119_CALLEES },
    crate::Bound_p_i32_i32_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_62dc121ee7e1, globals: &[], callees: FN_125_CALLEES },
    crate::Bound_p_i32_i32_i32_i32_ret_void { func: crate::c_85899_79292_1::ChunkFns::f_23c7b6aa52a7, globals: &[], callees: FN_131_CALLEES },
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
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_87792::ChunkFns::f_7e9d84c9daa2, globals: FN_72_GLOBALS, callees: FN_72_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_87792::ChunkFns::f_c8fc591b4686, globals: FN_77_GLOBALS, callees: FN_77_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_87792::ChunkFns::f_116a2da5e6bc, globals: FN_78_GLOBALS, callees: FN_78_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_87717d28a526, globals: &[], callees: FN_89_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_f5420555aefe, globals: &[], callees: FN_108_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_ed3ff760701d, globals: &[], callees: FN_114_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_004b66504437, globals: &[], callees: FN_115_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_81cc25430500, globals: &[], callees: FN_116_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_a98617ea6ec7, globals: &[], callees: FN_117_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_9a77c7cfc6c6, globals: &[], callees: FN_120_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_cbe6ccd58a27, globals: FN_127_GLOBALS, callees: FN_127_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_93892::ChunkFns::f_e4738ced9eb5, globals: FN_128_GLOBALS, callees: FN_128_CALLEES },
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
    crate::Bound_p_i32_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_c60b575467fb, globals: &[], callees: FN_69_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_aca97c761850, globals: &[], callees: FN_70_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_dc65747ad20e, globals: &[], callees: FN_96_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_a562805ce2e0, globals: &[], callees: FN_97_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_534b766828b2, globals: FN_126_GLOBALS, callees: FN_126_CALLEES },
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
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_f8be9dff3f3b, globals: FN_32_GLOBALS, callees: FN_32_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_f8be9dff3f3b, globals: FN_35_GLOBALS, callees: FN_35_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_b2d0372afbef, globals: FN_36_GLOBALS, callees: FN_36_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_97792::ChunkFns::f_8e3a351d9bba, globals: FN_39_GLOBALS, callees: FN_39_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_98a40a4e126c, globals: FN_51_GLOBALS, callees: FN_51_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_44bf2eebebc5, globals: FN_52_GLOBALS, callees: FN_52_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_87792::ChunkFns::f_d36e1b31003e, globals: FN_73_GLOBALS, callees: FN_73_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_f80c06cb23fe, globals: FN_74_GLOBALS, callees: FN_74_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_ca9b528594d9, globals: &[], callees: FN_79_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_6abe7e1add88, globals: &[], callees: FN_84_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_6f29b496c61b, globals: FN_85_GLOBALS, callees: FN_85_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_2488f523e9ab, globals: &[], callees: FN_94_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_1632f6fcfe85, globals: &[], callees: FN_107_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_92692::ChunkFns::f_ed28f33ecf20, globals: FN_122_GLOBALS, callees: FN_122_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_8ef52de6b9da, globals: &[], callees: FN_124_CALLEES },
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
    crate::Bound_p_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_0c1b387702b2, globals: FN_71_GLOBALS, callees: FN_71_CALLEES },
    crate::Bound_p_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_76a01cf8b94d, globals: &[], callees: FN_95_CALLEES },
    crate::Bound_p_i32_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_090940f804f9, globals: &[], callees: FN_99_CALLEES },
    crate::Bound_p_i32_i32_ret_void { func: crate::c_85899_93892::ChunkFns::f_a90324b40919, globals: FN_111_GLOBALS, callees: FN_111_CALLEES },
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
    crate::Bound_p_i32_i64_ret_void { func: crate::c_85899_79292::ChunkFns::f_6ffdd7253944, globals: &[], callees: FN_100_CALLEES },
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
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_7c241a508314, globals: FN_68_GLOBALS, callees: FN_68_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_621550e77e3b, globals: FN_81_GLOBALS, callees: FN_81_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_c9226330fb94, globals: &[], callees: FN_92_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_b7b055bd5416, globals: FN_103_GLOBALS, callees: FN_103_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_239cb2f96e82, globals: FN_112_GLOBALS, callees: FN_112_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_c0e8f275fac9, globals: &[], callees: FN_118_CALLEES },
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
    crate::Bound_p_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_60b618eb1778, globals: FN_80_GLOBALS, callees: FN_80_CALLEES },
    crate::Bound_p_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_a1d02c5fb9c2, globals: FN_82_GLOBALS, callees: FN_82_CALLEES },
    crate::Bound_p_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_717050e4ee19, globals: FN_83_GLOBALS, callees: FN_83_CALLEES },
    crate::Bound_p_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_29d13b4bd6bc, globals: &[], callees: FN_101_CALLEES },
    crate::Bound_p_i32_ret_void { func: crate::c_85899_79292::ChunkFns::f_9386bb430299, globals: FN_123_GLOBALS, callees: FN_123_CALLEES },
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
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_7734d017eaa1, globals: FN_54_GLOBALS, callees: FN_54_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_ed88189ac803, globals: FN_55_GLOBALS, callees: FN_55_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_4b4523d1e0e6, globals: FN_56_GLOBALS, callees: FN_56_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_a802fe84ff33, globals: FN_57_GLOBALS, callees: FN_57_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_8d9629285dfc, globals: FN_58_GLOBALS, callees: FN_58_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_2d0311fa7074, globals: FN_59_GLOBALS, callees: FN_59_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_c4aab8465645, globals: FN_60_GLOBALS, callees: FN_60_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_9d76726450cf, globals: &[], callees: FN_61_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_892d46fb695c, globals: &[], callees: FN_62_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_6154355b5577, globals: FN_64_GLOBALS, callees: FN_64_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_7308eeca408a, globals: FN_67_GLOBALS, callees: FN_67_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_af93137c5c01, globals: FN_104_GLOBALS, callees: FN_104_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_79292::ChunkFns::f_24fd9031937d, globals: FN_113_GLOBALS, callees: &[] },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_79292_1::ChunkFns::f_7ec9b65296e7, globals: FN_135_GLOBALS, callees: FN_135_CALLEES },
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
    crate::Bound_p_unit_ret_void { func: crate::c_85899_79292::ChunkFns::f_8d58d78a7a3b, globals: &[], callees: FN_63_CALLEES },
    crate::Bound_p_unit_ret_void { func: crate::c_85899_87792::ChunkFns::f_fc806df2edb4, globals: FN_86_GLOBALS, callees: FN_86_CALLEES },
    crate::Bound_p_unit_ret_void { func: crate::c_85899_79292::ChunkFns::f_81a252fc3c25, globals: FN_105_GLOBALS, callees: FN_105_CALLEES },
    crate::Bound_p_unit_ret_void { func: crate::c_85899_79292::ChunkFns::f_3d58e151fe65, globals: FN_106_GLOBALS, callees: FN_106_CALLEES },
    crate::Bound_p_unit_ret_void { func: crate::c_85899_79292_1::ChunkFns::f_c71faaa3ebde, globals: FN_132_GLOBALS, callees: FN_132_CALLEES },
    crate::Bound_p_unit_ret_void { func: crate::c_85899_79292_1::ChunkFns::f_9809d582b386, globals: FN_133_GLOBALS, callees: FN_133_CALLEES },
    crate::Bound_p_unit_ret_void { func: crate::c_85899_79292_1::ChunkFns::f_6356e8ed2b9e, globals: FN_134_GLOBALS, callees: FN_134_CALLEES },
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

const MODULE_META: SubstituteModuleMeta = SubstituteModuleMeta {
    name: "85899_97792/f1680@l8589997868",
    accounting_version: 2u32,
    wasm_sha256: [0xa4, 0x3f, 0xca, 0x3c, 0x93, 0xa0, 0x31, 0xab, 0x7e, 0x35, 0x1c, 0x88, 0x78, 0xcf, 0x54, 0x0e, 0x9f, 0xdb, 0x9f, 0xa9, 0x93, 0xa7, 0x80, 0xa8, 0xc9, 0x3f, 0xd6, 0x9e, 0x20, 0x00, 0x19, 0xcf],
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

