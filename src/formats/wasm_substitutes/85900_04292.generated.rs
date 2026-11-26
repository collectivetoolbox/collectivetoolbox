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

//! Generated native Rust WebAssembly substitute chunk `85900_04292`.
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
    pub(crate) fn f_426fb06d685b(vm: &mut SubstituteVm, dispatch: &'static crate::ModuleDispatch, g_map: &'static [usize], c_map: &'static [usize], host: &mut dyn WasmSubstituteHost) -> std::result::Result<(), WasmTrap> {
    let mut l0: i64 = 0i64;
    let mut l1: i64 = 0i64;
    let mut l2: i32 = 0i32;
    let mut l3: i64 = 0i64;
    let mut l4: i64 = 0i64;
    let mut l5: i32 = 0i32;
    let mut l6: i64 = 0i64;
    let mut l7: i64 = 0i64;
    let mut l8: i32 = 0i32;
    let mut l9: i64 = 0i64;
    let mut l10: i64 = 0i64;
    let mut l11: i32 = 0i32;
    let mut l12: i64 = 0i64;
    let mut l13: i64 = 0i64;
    let mut l14: i32 = 0i32;
    let mut l15: i64 = 0i64;
    let mut l16: i64 = 0i64;
    let mut l17: i32 = 0i32;
    let mut l18: i64 = 0i64;
    let mut l19: i64 = 0i64;
    let mut l20: i32 = 0i32;
    let mut l21: i64 = 0i64;
    let mut l22: i64 = 0i64;
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
    let mut l38: i64 = 0i64;
    let mut l39: i64 = 0i64;
    let mut l40: i32 = 0i32;
    let mut l41: i64 = 0i64;
    let mut l42: i64 = 0i64;
    let mut l43: i32 = 0i32;
    let mut l44: i64 = 0i64;
    let mut l45: i64 = 0i64;
    let mut l46: i32 = 0i32;
    let mut l47: i64 = 0i64;
    let mut l48: i64 = 0i64;
    let mut l49: i32 = 0i32;
    let mut l50: i64 = 0i64;
    let mut l51: i64 = 0i64;
    let mut l52: i32 = 0i32;
    let mut l53: i64 = 0i64;
    let mut l54: i64 = 0i64;
    let mut l55: i32 = 0i32;
    let mut l56: i64 = 0i64;
    let mut l57: i64 = 0i64;
    let mut l58: i32 = 0i32;
    let mut l59: i64 = 0i64;
    let mut l60: i64 = 0i64;
    let mut l61: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i32_2: i32 = 0i32;
    vm.deduct_1(host)?;
    s_i32_0 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
    'blk_1: {
        if s_i32_0 != 0 {
            return Ok(());
        }
    }
    s_i32_0 = 1i32;
    vm.set_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?, s_i32_0)?;
    s_i32_0 = 1i32;
    vm.set_i32(*g_map.get(1usize).ok_or(WasmTrap::Unreachable)?, s_i32_0)?;
    s_i32_0 = 3i32;
    vm.set_i32(*g_map.get(2usize).ok_or(WasmTrap::Unreachable)?, s_i32_0)?;
    s_i32_0 = 4i32;
    vm.set_i32(*g_map.get(3usize).ok_or(WasmTrap::Unreachable)?, s_i32_0)?;
    s_i32_0 = 5i32;
    vm.set_i32(*g_map.get(4usize).ok_or(WasmTrap::Unreachable)?, s_i32_0)?;
    s_i32_0 = 9i32;
    vm.set_i32(*g_map.get(5usize).ok_or(WasmTrap::Unreachable)?, s_i32_0)?;
    s_i32_0 = 8i32;
    vm.set_i32(*g_map.get(6usize).ok_or(WasmTrap::Unreachable)?, s_i32_0)?;
    s_i32_0 = 10i32;
    vm.set_i32(*g_map.get(7usize).ok_or(WasmTrap::Unreachable)?, s_i32_0)?;
    s_i32_0 = 2i32;
    vm.set_i32(*g_map.get(8usize).ok_or(WasmTrap::Unreachable)?, s_i32_0)?;
    s_i32_0 = 0i32;
    vm.set_i32(*g_map.get(9usize).ok_or(WasmTrap::Unreachable)?, s_i32_0)?;
    s_i32_0 = 6i32;
    vm.set_i32(*g_map.get(10usize).ok_or(WasmTrap::Unreachable)?, s_i32_0)?;
    s_i32_0 = 7i32;
    vm.set_i32(*g_map.get(11usize).ok_or(WasmTrap::Unreachable)?, s_i32_0)?;
    s_i64_0 = -1i64;
    l0 = s_i64_0;
    s_i64_0 = -1i64;
    l1 = s_i64_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 0usize, s_i32_0, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l2 = s_i32_0;
    s_i32_0 = l2;
    s_i64_1 = l0;
    ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
    s_i32_0 = l2;
    s_i64_1 = l1;
    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
    s_i32_0 = l2;
    vm.set_i32(*g_map.get(12usize).ok_or(WasmTrap::Unreachable)?, s_i32_0)?;
    s_i64_0 = 1i64;
    l3 = s_i64_0;
    s_i64_0 = 0i64;
    l4 = s_i64_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 0usize, s_i32_0, host);
    vm.exit_call();
    let __t1 = __call_res?;
    s_i32_0 = __t1;
    l5 = s_i32_0;
    s_i32_0 = l5;
    s_i64_1 = l3;
    ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
    s_i32_0 = l5;
    s_i64_1 = l4;
    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
    s_i32_0 = l5;
    vm.set_i32(*g_map.get(13usize).ok_or(WasmTrap::Unreachable)?, s_i32_0)?;
    s_i64_0 = 0i64;
    l6 = s_i64_0;
    s_i64_0 = 0i64;
    l7 = s_i64_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 0usize, s_i32_0, host);
    vm.exit_call();
    let __t2 = __call_res?;
    s_i32_0 = __t2;
    l8 = s_i32_0;
    s_i32_0 = l8;
    s_i64_1 = l6;
    ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
    s_i32_0 = l8;
    s_i64_1 = l7;
    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
    s_i32_0 = l8;
    vm.set_i32(*g_map.get(14usize).ok_or(WasmTrap::Unreachable)?, s_i32_0)?;
    s_i64_0 = -1i64;
    l9 = s_i64_0;
    s_i64_0 = 9223372036854775807i64;
    l10 = s_i64_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 0usize, s_i32_0, host);
    vm.exit_call();
    let __t3 = __call_res?;
    s_i32_0 = __t3;
    l11 = s_i32_0;
    s_i32_0 = l11;
    s_i64_1 = l9;
    ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
    s_i32_0 = l11;
    s_i64_1 = l10;
    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
    s_i32_0 = l11;
    vm.set_i32(*g_map.get(15usize).ok_or(WasmTrap::Unreachable)?, s_i32_0)?;
    s_i64_0 = 0i64;
    l12 = s_i64_0;
    s_i64_0 = i64::MIN;
    l13 = s_i64_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 0usize, s_i32_0, host);
    vm.exit_call();
    let __t4 = __call_res?;
    s_i32_0 = __t4;
    l14 = s_i32_0;
    s_i32_0 = l14;
    s_i64_1 = l12;
    ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
    s_i32_0 = l14;
    s_i64_1 = l13;
    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
    s_i32_0 = l14;
    vm.set_i32(*g_map.get(16usize).ok_or(WasmTrap::Unreachable)?, s_i32_0)?;
    s_i64_0 = -1i64;
    l15 = s_i64_0;
    s_i64_0 = -1i64;
    l16 = s_i64_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 0usize, s_i32_0, host);
    vm.exit_call();
    let __t5 = __call_res?;
    s_i32_0 = __t5;
    l17 = s_i32_0;
    s_i32_0 = l17;
    s_i64_1 = l15;
    ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
    s_i32_0 = l17;
    s_i64_1 = l16;
    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
    s_i32_0 = l17;
    vm.set_i32(*g_map.get(17usize).ok_or(WasmTrap::Unreachable)?, s_i32_0)?;
    s_i64_0 = 1i64;
    l18 = s_i64_0;
    s_i64_0 = 0i64;
    l19 = s_i64_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 0usize, s_i32_0, host);
    vm.exit_call();
    let __t6 = __call_res?;
    s_i32_0 = __t6;
    l20 = s_i32_0;
    s_i32_0 = l20;
    s_i64_1 = l18;
    ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
    s_i32_0 = l20;
    s_i64_1 = l19;
    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
    s_i32_0 = l20;
    vm.set_i32(*g_map.get(18usize).ok_or(WasmTrap::Unreachable)?, s_i32_0)?;
    s_i64_0 = 0i64;
    l21 = s_i64_0;
    s_i64_0 = 0i64;
    l22 = s_i64_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 0usize, s_i32_0, host);
    vm.exit_call();
    let __t7 = __call_res?;
    s_i32_0 = __t7;
    l23 = s_i32_0;
    s_i32_0 = l23;
    s_i64_1 = l21;
    ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
    s_i32_0 = l23;
    s_i64_1 = l22;
    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
    s_i32_0 = l23;
    vm.set_i32(*g_map.get(19usize).ok_or(WasmTrap::Unreachable)?, s_i32_0)?;
    s_i32_0 = 1i32;
    l24 = s_i32_0;
    s_i32_0 = 1i32;
    l25 = s_i32_0;
    s_i32_0 = 20i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 0usize, s_i32_0, host);
    vm.exit_call();
    let __t8 = __call_res?;
    s_i32_0 = __t8;
    l26 = s_i32_0;
    s_i32_0 = l26;
    s_i32_1 = l26;
    s_i32_2 = 16i32;
    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l26;
    s_i32_1 = 1i32;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l26;
    s_i32_1 = 1i32;
    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
    s_i32_0 = l26;
    s_i32_1 = 1i32;
    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
    s_i32_0 = l26;
    s_i32_1 = l25;
    ops::i32_store(vm, 0, s_i32_0, 16u32, s_i32_1)?;
    s_i32_0 = l26;
    l27 = s_i32_0;
    s_i32_0 = 8i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 0usize, s_i32_0, host);
    vm.exit_call();
    let __t9 = __call_res?;
    s_i32_0 = __t9;
    l28 = s_i32_0;
    s_i32_0 = l28;
    s_i32_1 = l24;
    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l28;
    s_i32_1 = l27;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l28;
    vm.set_i32(*g_map.get(20usize).ok_or(WasmTrap::Unreachable)?, s_i32_0)?;
    s_i32_0 = 0i32;
    l29 = s_i32_0;
    s_i32_0 = 1i32;
    l30 = s_i32_0;
    s_i32_0 = 20i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 0usize, s_i32_0, host);
    vm.exit_call();
    let __t10 = __call_res?;
    s_i32_0 = __t10;
    l31 = s_i32_0;
    s_i32_0 = l31;
    s_i32_1 = l31;
    s_i32_2 = 16i32;
    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l31;
    s_i32_1 = 1i32;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l31;
    s_i32_1 = 1i32;
    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
    s_i32_0 = l31;
    s_i32_1 = 1i32;
    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
    s_i32_0 = l31;
    s_i32_1 = l30;
    ops::i32_store(vm, 0, s_i32_0, 16u32, s_i32_1)?;
    s_i32_0 = l31;
    l32 = s_i32_0;
    s_i32_0 = 8i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 0usize, s_i32_0, host);
    vm.exit_call();
    let __t11 = __call_res?;
    s_i32_0 = __t11;
    l33 = s_i32_0;
    s_i32_0 = l33;
    s_i32_1 = l29;
    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l33;
    s_i32_1 = l32;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l33;
    vm.set_i32(*g_map.get(21usize).ok_or(WasmTrap::Unreachable)?, s_i32_0)?;
    s_i32_0 = 0i32;
    l34 = s_i32_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 0usize, s_i32_0, host);
    vm.exit_call();
    let __t12 = __call_res?;
    s_i32_0 = __t12;
    l35 = s_i32_0;
    s_i32_0 = l35;
    s_i32_1 = l35;
    s_i32_2 = 16i32;
    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l35;
    s_i32_1 = 0i32;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l35;
    s_i32_1 = 0i32;
    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
    s_i32_0 = l35;
    s_i32_1 = 1i32;
    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
    s_i32_0 = l35;
    l36 = s_i32_0;
    s_i32_0 = 8i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 0usize, s_i32_0, host);
    vm.exit_call();
    let __t13 = __call_res?;
    s_i32_0 = __t13;
    l37 = s_i32_0;
    s_i32_0 = l37;
    s_i32_1 = l34;
    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l37;
    s_i32_1 = l36;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l37;
    vm.set_i32(*g_map.get(22usize).ok_or(WasmTrap::Unreachable)?, s_i32_0)?;
    s_i32_0 = -1i32;
    vm.set_i32(*g_map.get(23usize).ok_or(WasmTrap::Unreachable)?, s_i32_0)?;
    s_i32_0 = -1i32;
    vm.deduct_1(host)?;
    vm.set_i32(*g_map.get(24usize).ok_or(WasmTrap::Unreachable)?, s_i32_0)?;
    s_i64_0 = 1114112i64;
    l38 = s_i64_0;
    s_i64_0 = 0i64;
    l39 = s_i64_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 0usize, s_i32_0, host);
    vm.exit_call();
    let __t14 = __call_res?;
    s_i32_0 = __t14;
    l40 = s_i32_0;
    s_i32_0 = l40;
    s_i64_1 = l38;
    ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
    s_i32_0 = l40;
    s_i64_1 = l39;
    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
    s_i32_0 = l40;
    vm.set_i32(*g_map.get(25usize).ok_or(WasmTrap::Unreachable)?, s_i32_0)?;
    s_i64_0 = 0i64;
    l41 = s_i64_0;
    s_i64_0 = 0i64;
    l42 = s_i64_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 0usize, s_i32_0, host);
    vm.exit_call();
    let __t15 = __call_res?;
    s_i32_0 = __t15;
    l43 = s_i32_0;
    s_i32_0 = l43;
    s_i64_1 = l41;
    ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
    s_i32_0 = l43;
    s_i64_1 = l42;
    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
    s_i32_0 = l43;
    vm.set_i32(*g_map.get(26usize).ok_or(WasmTrap::Unreachable)?, s_i32_0)?;
    s_i64_0 = 8589934592i64;
    l44 = s_i64_0;
    s_i64_0 = 0i64;
    l45 = s_i64_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 0usize, s_i32_0, host);
    vm.exit_call();
    let __t16 = __call_res?;
    s_i32_0 = __t16;
    l46 = s_i32_0;
    s_i32_0 = l46;
    s_i64_1 = l44;
    ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
    s_i32_0 = l46;
    s_i64_1 = l45;
    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
    s_i32_0 = l46;
    vm.set_i32(*g_map.get(27usize).ok_or(WasmTrap::Unreachable)?, s_i32_0)?;
    s_i64_0 = 12884901887i64;
    l47 = s_i64_0;
    s_i64_0 = 0i64;
    l48 = s_i64_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 0usize, s_i32_0, host);
    vm.exit_call();
    let __t17 = __call_res?;
    s_i32_0 = __t17;
    l49 = s_i32_0;
    s_i32_0 = l49;
    s_i64_1 = l47;
    ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
    s_i32_0 = l49;
    s_i64_1 = l48;
    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
    s_i32_0 = l49;
    vm.set_i32(*g_map.get(28usize).ok_or(WasmTrap::Unreachable)?, s_i32_0)?;
    s_i64_0 = 1114112i64;
    l50 = s_i64_0;
    s_i64_0 = 0i64;
    l51 = s_i64_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 0usize, s_i32_0, host);
    vm.exit_call();
    let __t18 = __call_res?;
    s_i32_0 = __t18;
    l52 = s_i32_0;
    s_i32_0 = l52;
    s_i64_1 = l50;
    ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
    s_i32_0 = l52;
    s_i64_1 = l51;
    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
    s_i32_0 = l52;
    vm.set_i32(*g_map.get(29usize).ok_or(WasmTrap::Unreachable)?, s_i32_0)?;
    s_i64_0 = 1114111i64;
    l53 = s_i64_0;
    s_i64_0 = 0i64;
    l54 = s_i64_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 0usize, s_i32_0, host);
    vm.exit_call();
    let __t19 = __call_res?;
    s_i32_0 = __t19;
    l55 = s_i32_0;
    s_i32_0 = l55;
    s_i64_1 = l53;
    ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
    s_i32_0 = l55;
    s_i64_1 = l54;
    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
    s_i32_0 = l55;
    vm.set_i32(*g_map.get(30usize).ok_or(WasmTrap::Unreachable)?, s_i32_0)?;
    s_i64_0 = 55296i64;
    l56 = s_i64_0;
    s_i64_0 = 0i64;
    l57 = s_i64_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 0usize, s_i32_0, host);
    vm.exit_call();
    let __t20 = __call_res?;
    s_i32_0 = __t20;
    l58 = s_i32_0;
    s_i32_0 = l58;
    s_i64_1 = l56;
    ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
    s_i32_0 = l58;
    s_i64_1 = l57;
    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
    s_i32_0 = l58;
    vm.set_i32(*g_map.get(31usize).ok_or(WasmTrap::Unreachable)?, s_i32_0)?;
    s_i64_0 = 57343i64;
    l59 = s_i64_0;
    s_i64_0 = 0i64;
    l60 = s_i64_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 0usize, s_i32_0, host);
    vm.exit_call();
    let __t21 = __call_res?;
    s_i32_0 = __t21;
    l61 = s_i32_0;
    s_i32_0 = l61;
    s_i64_1 = l59;
    ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
    s_i32_0 = l61;
    s_i64_1 = l60;
    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
    s_i32_0 = l61;
    vm.set_i32(*g_map.get(32usize).ok_or(WasmTrap::Unreachable)?, s_i32_0)?;
    return Ok(());
    }

    pub(crate) fn f_94115c07ddc1(vm: &mut SubstituteVm, dispatch: &'static crate::ModuleDispatch, g_map: &'static [usize], c_map: &'static [usize], p0: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut s_i32_0: i32 = 0i32;
    vm.deduct_1(host)?;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_unit_ret_void(vm, c_map, 0usize, host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_0, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 2usize, s_i32_0, host);
    vm.exit_call();
    let __t1 = __call_res?;
    s_i32_0 = __t1;
    let __t2 = s_i32_0;
    return dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, __t2, host);
    }

    pub(crate) fn f_05be71efd2df(vm: &mut SubstituteVm, dispatch: &'static crate::ModuleDispatch, g_map: &'static [usize], c_map: &'static [usize], p0: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = 0i32;
    let mut l2: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct_1(host)?;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_unit_ret_void(vm, c_map, 0usize, host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_1 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 1usize, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            l1 = s_i32_0;
            s_i32_0 = l1;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i32_0 = l0;
    s_i32_1 = vm.get_i32(*g_map.get(1usize).ok_or(WasmTrap::Unreachable)?)?;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 2usize, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t1 = __call_res?;
    s_i32_0 = __t1;
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i32_1 = vm.get_i32(*g_map.get(2usize).ok_or(WasmTrap::Unreachable)?)?;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 3usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_0 = __t2;
        } else {
            s_i32_0 = 0i32;
        }
    }
    s_i32_0 = ops::i32_eqz(s_i32_0);
    l2 = s_i32_0;
    s_i32_0 = l2;
    return Ok(s_i32_0);
    }

    pub(crate) fn f_e6879a687a42(vm: &mut SubstituteVm, dispatch: &'static crate::ModuleDispatch, g_map: &'static [usize], c_map: &'static [usize], host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = 0i32;
    let mut l1: i32 = 0i32;
    let mut l2: i32 = 0i32;
    let mut l3: i64 = 0i64;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i32 = 0i32;
    let mut l7: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i32_2: i32 = 0i32;
    vm.deduct_1(host)?;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_unit_ret_void(vm, c_map, 0usize, host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_0, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l0 = s_i32_0;
    s_i32_0 = l0;
    s_i32_1 = l0;
    s_i32_2 = 16i32;
    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l0;
    s_i32_1 = 0i32;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l0;
    s_i32_1 = 0i32;
    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
    s_i32_0 = l0;
    s_i32_1 = 1i32;
    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
    s_i32_0 = l0;
    l1 = s_i32_0;
    s_i64_0 = 64i64;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i64_ret_i32(vm, c_map, 2usize, s_i64_0, host);
    vm.exit_call();
    let __t1 = __call_res?;
    s_i32_0 = __t1;
    l2 = s_i32_0;
    s_i64_0 = 0i64;
    l3 = s_i64_0;
    'blk_1: {
        'blk_2: loop {
            vm.deduct_1(host)?;
            s_i64_0 = l3;
            s_i64_1 = 64i64;
            s_i32_0 = ops::i64_lt_u(s_i64_0, s_i64_1);
            s_i32_0 = ops::i32_eqz(s_i32_0);
            if s_i32_0 != 0 {
                break 'blk_1;
            }
            s_i32_0 = l2;
            s_i32_1 = l1;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 3usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i64_0 = l3;
            s_i64_1 = 1i64;
            s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
            l3 = s_i64_0;
            continue 'blk_2;
            break 'blk_2;
        }
    }
    vm.enter_call()?;
    let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 4usize, host);
    vm.exit_call();
    let __t2 = __call_res?;
    s_i32_0 = __t2;
    l4 = s_i32_0;
    s_i32_0 = l2;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 5usize, s_i32_0, host);
    vm.exit_call();
    let __t3 = __call_res?;
    s_i32_0 = __t3;
    l5 = s_i32_0;
    s_i32_0 = 8i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_0, host);
    vm.exit_call();
    let __t4 = __call_res?;
    s_i32_0 = __t4;
    l6 = s_i32_0;
    s_i32_0 = l6;
    s_i32_1 = l4;
    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l6;
    s_i32_1 = l5;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l6;
    l7 = s_i32_0;
    s_i32_0 = l7;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn f_bd36f6dae8b0(vm: &mut SubstituteVm, dispatch: &'static crate::ModuleDispatch, g_map: &'static [usize], c_map: &'static [usize], p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i64 = 0i64;
    let mut l3: i32 = 0i32;
    let mut l4: i64 = 0i64;
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
    let mut l15: i64 = 0i64;
    let mut l16: i64 = 0i64;
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
    let mut l37: i64 = 0i64;
    let mut l38: i64 = 0i64;
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
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i32_2: i32 = 0i32;
    vm.deduct_1(host)?;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_unit_ret_void(vm, c_map, 0usize, host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l1;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i64(vm, c_map, 1usize, s_i32_0, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i64_0 = __t0;
    s_i64_1 = 63i64;
    s_i64_0 = ops::i64_and(s_i64_0, s_i64_1);
    l2 = s_i64_0;
    s_i32_0 = l0;
    l3 = s_i32_0;
    s_i64_0 = l2;
    l4 = s_i64_0;
    s_i64_0 = l4;
    s_i32_1 = l3;
    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
    s_i64_1 = ops::i64_extend_i32_u(s_i32_1);
    s_i32_0 = ops::i64_lt_u(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
            l5 = s_i32_0;
            s_i32_0 = l3;
            s_i64_1 = l4;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i64_ret_i32(vm, c_map, 2usize, s_i32_0, s_i64_1, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            l6 = s_i32_0;
            s_i32_0 = 8i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_0 = __t2;
            l7 = s_i32_0;
            s_i32_0 = l7;
            s_i32_1 = l5;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l7;
            s_i32_1 = l6;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l7;
        } else {
            s_i32_0 = 0i32;
            l8 = s_i32_0;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i32_0 = __t3;
            l9 = s_i32_0;
            s_i32_0 = l9;
            s_i32_1 = l9;
            s_i32_2 = 16i32;
            s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l9;
            s_i32_1 = 0i32;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l9;
            s_i32_1 = 0i32;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l9;
            s_i32_1 = 1i32;
            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
            s_i32_0 = l9;
            l10 = s_i32_0;
            s_i32_0 = 8i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
            vm.exit_call();
            let __t4 = __call_res?;
            s_i32_0 = __t4;
            l11 = s_i32_0;
            s_i32_0 = l11;
            s_i32_1 = l8;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l11;
            s_i32_1 = l10;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l11;
        }
    }
    l12 = s_i32_0;
    s_i32_0 = l12;
    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = l12;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
            l13 = s_i32_0;
            s_i32_0 = l13;
        } else {
            s_i32_0 = 0i32;
            l14 = s_i32_0;
            s_i64_0 = 0i64;
            l15 = s_i64_0;
            s_i64_0 = 0i64;
            l16 = s_i64_0;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
            vm.exit_call();
            let __t5 = __call_res?;
            s_i32_0 = __t5;
            l17 = s_i32_0;
            s_i32_0 = l17;
            s_i64_1 = l15;
            ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
            s_i32_0 = l17;
            s_i64_1 = l16;
            ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
            s_i32_0 = l17;
            l18 = s_i32_0;
            s_i32_0 = 0i32;
            l19 = s_i32_0;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
            vm.exit_call();
            let __t6 = __call_res?;
            s_i32_0 = __t6;
            l20 = s_i32_0;
            s_i32_0 = l20;
            s_i32_1 = l20;
            s_i32_2 = 16i32;
            s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l20;
            s_i32_1 = 0i32;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l20;
            s_i32_1 = 0i32;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l20;
            s_i32_1 = 1i32;
            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
            s_i32_0 = l20;
            l21 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
            vm.exit_call();
            let __t7 = __call_res?;
            s_i32_0 = __t7;
            l22 = s_i32_0;
            s_i32_0 = l22;
            s_i32_1 = l18;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l22;
            s_i32_1 = l19;
            ops::i32_store8(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l22;
            s_i32_1 = l21;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l22;
            l23 = s_i32_0;
            s_i32_0 = 8i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
            vm.exit_call();
            let __t8 = __call_res?;
            s_i32_0 = __t8;
            l24 = s_i32_0;
            s_i32_0 = l24;
            s_i32_1 = l14;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l24;
            s_i32_1 = l23;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l24;
            l25 = s_i32_0;
            s_i32_0 = l25;
            return Ok(s_i32_0);
        }
    }
    l26 = s_i32_0;
    s_i32_0 = l26;
    l27 = s_i32_0;
    s_i32_0 = l27;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    l28 = s_i32_0;
    s_i32_0 = l27;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    l29 = s_i32_0;
    s_i32_0 = 0i32;
    l30 = s_i32_0;
    'blk_3: {
        'blk_4: loop {
            vm.deduct_1(host)?;
            s_i32_0 = l30;
            s_i32_1 = l29;
            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
            if s_i32_0 != 0 {
                break 'blk_3;
            }
            s_i32_0 = l28;
            s_i32_1 = l30;
            s_i32_2 = 4i32;
            s_i32_1 = ops::i32_mul(s_i32_1, s_i32_2);
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
            l31 = s_i32_0;
            'blk_5: {
                s_i32_0 = l31;
                s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                s_i32_1 = l1;
                vm.enter_call()?;
                let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, host);
                vm.exit_call();
                let __t9 = __call_res?;
                s_i32_0 = __t9;
                'blk_6: {
                    if s_i32_0 != 0 {
                        s_i32_0 = 1i32;
                        l32 = s_i32_0;
                        s_i32_0 = l31;
                        l33 = s_i32_0;
                        s_i32_0 = 8i32;
                        vm.enter_call()?;
                        let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
                        vm.exit_call();
                        let __t10 = __call_res?;
                        s_i32_0 = __t10;
                        l34 = s_i32_0;
                        s_i32_0 = l34;
                        s_i32_1 = l32;
                        ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                        s_i32_0 = l34;
                        s_i32_1 = l33;
                        ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                        s_i32_0 = l34;
                        l35 = s_i32_0;
                        s_i32_0 = l35;
                        return Ok(s_i32_0);
                    } else {
                    }
                }
            }
            s_i32_0 = l30;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l30 = s_i32_0;
            continue 'blk_4;
            break 'blk_4;
        }
    }
    s_i32_0 = 0i32;
    l36 = s_i32_0;
    s_i64_0 = 0i64;
    l37 = s_i64_0;
    s_i64_0 = 0i64;
    l38 = s_i64_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
    vm.exit_call();
    let __t11 = __call_res?;
    s_i32_0 = __t11;
    l39 = s_i32_0;
    s_i32_0 = l39;
    s_i64_1 = l37;
    ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
    s_i32_0 = l39;
    s_i64_1 = l38;
    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
    s_i32_0 = l39;
    l40 = s_i32_0;
    s_i32_0 = 0i32;
    l41 = s_i32_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
    vm.exit_call();
    let __t12 = __call_res?;
    s_i32_0 = __t12;
    l42 = s_i32_0;
    s_i32_0 = l42;
    s_i32_1 = l42;
    s_i32_2 = 16i32;
    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l42;
    s_i32_1 = 0i32;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l42;
    s_i32_1 = 0i32;
    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
    s_i32_0 = l42;
    s_i32_1 = 1i32;
    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
    s_i32_0 = l42;
    l43 = s_i32_0;
    s_i32_0 = 12i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
    vm.exit_call();
    let __t13 = __call_res?;
    s_i32_0 = __t13;
    l44 = s_i32_0;
    s_i32_0 = l44;
    s_i32_1 = l40;
    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l44;
    s_i32_1 = l41;
    ops::i32_store8(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l44;
    s_i32_1 = l43;
    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
    s_i32_0 = l44;
    l45 = s_i32_0;
    s_i32_0 = 8i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
    vm.exit_call();
    let __t14 = __call_res?;
    s_i32_0 = __t14;
    l46 = s_i32_0;
    s_i32_0 = l46;
    s_i32_1 = l36;
    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l46;
    s_i32_1 = l45;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l46;
    l47 = s_i32_0;
    s_i32_0 = l47;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn f_37bc21062338(vm: &mut SubstituteVm, dispatch: &'static crate::ModuleDispatch, g_map: &'static [usize], c_map: &'static [usize], p0: i32, p1: i32, p2: i32, p3: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = p2;
    let mut l3: i32 = p3;
    let mut l4: i64 = 0i64;
    let mut l5: i32 = 0i32;
    let mut l6: i64 = 0i64;
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
    let mut l30: i64 = 0i64;
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
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i32_2: i32 = 0i32;
    vm.deduct_1(host)?;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_unit_ret_void(vm, c_map, 0usize, host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l1;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i64(vm, c_map, 1usize, s_i32_0, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i64_0 = __t0;
    s_i64_1 = 63i64;
    s_i64_0 = ops::i64_and(s_i64_0, s_i64_1);
    l4 = s_i64_0;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    l5 = s_i32_0;
    s_i64_0 = l4;
    l6 = s_i64_0;
    s_i64_0 = l6;
    s_i32_1 = l5;
    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
    s_i64_1 = ops::i64_extend_i32_u(s_i32_1);
    s_i32_0 = ops::i64_lt_u(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
            l7 = s_i32_0;
            s_i32_0 = l5;
            s_i64_1 = l6;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i64_ret_i32(vm, c_map, 2usize, s_i32_0, s_i64_1, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            l8 = s_i32_0;
            s_i32_0 = 8i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_0 = __t2;
            l9 = s_i32_0;
            s_i32_0 = l9;
            s_i32_1 = l7;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l9;
            s_i32_1 = l8;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l9;
        } else {
            s_i32_0 = 0i32;
            l10 = s_i32_0;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i32_0 = __t3;
            l11 = s_i32_0;
            s_i32_0 = l11;
            s_i32_1 = l11;
            s_i32_2 = 16i32;
            s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l11;
            s_i32_1 = 0i32;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l11;
            s_i32_1 = 0i32;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l11;
            s_i32_1 = 1i32;
            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
            s_i32_0 = l11;
            l12 = s_i32_0;
            s_i32_0 = 8i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
            vm.exit_call();
            let __t4 = __call_res?;
            s_i32_0 = __t4;
            l13 = s_i32_0;
            s_i32_0 = l13;
            s_i32_1 = l10;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l13;
            s_i32_1 = l12;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l13;
        }
    }
    l14 = s_i32_0;
    s_i32_0 = l14;
    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = l14;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
            l15 = s_i32_0;
            s_i32_0 = l15;
        } else {
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
            vm.exit_call();
            let __t5 = __call_res?;
            s_i32_0 = __t5;
            l16 = s_i32_0;
            s_i32_0 = l16;
            s_i32_1 = l16;
            s_i32_2 = 16i32;
            s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l16;
            s_i32_1 = 0i32;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l16;
            s_i32_1 = 0i32;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l16;
            s_i32_1 = 1i32;
            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
            s_i32_0 = l16;
        }
    }
    l17 = s_i32_0;
    s_i32_0 = l17;
    l18 = s_i32_0;
    s_i32_0 = l18;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    l19 = s_i32_0;
    s_i32_0 = l18;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    l20 = s_i32_0;
    s_i32_0 = 0i32;
    l21 = s_i32_0;
    'blk_3: {
        'blk_4: loop {
            vm.deduct_1(host)?;
            s_i32_0 = l21;
            s_i32_1 = l20;
            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
            if s_i32_0 != 0 {
                break 'blk_3;
            }
            s_i32_0 = l19;
            s_i32_1 = l21;
            s_i32_2 = 4i32;
            s_i32_1 = ops::i32_mul(s_i32_1, s_i32_2);
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
            l22 = s_i32_0;
            'blk_5: {
                s_i32_0 = l22;
                s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                s_i32_1 = l1;
                vm.enter_call()?;
                let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, host);
                vm.exit_call();
                let __t6 = __call_res?;
                s_i32_0 = __t6;
                'blk_6: {
                    if s_i32_0 != 0 {
                        s_i32_0 = l0;
                        l23 = s_i32_0;
                        s_i32_0 = l23;
                        return Ok(s_i32_0);
                    } else {
                    }
                }
            }
            s_i32_0 = l21;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l21 = s_i32_0;
            continue 'blk_4;
            break 'blk_4;
        }
    }
    s_i32_0 = l17;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
    s_i64_1 = 1i64;
    s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i64_ret_i32(vm, c_map, 5usize, s_i64_0, host);
    vm.exit_call();
    let __t7 = __call_res?;
    s_i32_0 = __t7;
    l24 = s_i32_0;
    s_i32_0 = l24;
    s_i32_1 = l17;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 6usize, s_i32_0, s_i32_1, host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l24;
    s_i32_1 = l1;
    l25 = s_i32_1;
    s_i32_1 = l2;
    l26 = s_i32_1;
    s_i32_1 = l3;
    l27 = s_i32_1;
    s_i32_1 = 12i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_1, host);
    vm.exit_call();
    let __t8 = __call_res?;
    s_i32_1 = __t8;
    l28 = s_i32_1;
    s_i32_1 = l28;
    s_i32_2 = l25;
    ops::i32_store(vm, 0, s_i32_1, 0u32, s_i32_2)?;
    s_i32_1 = l28;
    s_i32_2 = l26;
    ops::i32_store8(vm, 0, s_i32_1, 4u32, s_i32_2)?;
    s_i32_1 = l28;
    s_i32_2 = l27;
    ops::i32_store(vm, 0, s_i32_1, 8u32, s_i32_2)?;
    s_i32_1 = l28;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 7usize, s_i32_0, s_i32_1, host);
    vm.exit_call();
    __call_res?;
    s_i64_0 = 64i64;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i64_ret_i32(vm, c_map, 5usize, s_i64_0, host);
    vm.exit_call();
    let __t9 = __call_res?;
    s_i32_0 = __t9;
    l29 = s_i32_0;
    s_i32_0 = l29;
    s_i32_1 = l0;
    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 6usize, s_i32_0, s_i32_1, host);
    vm.exit_call();
    __call_res?;
    s_i64_0 = l4;
    l30 = s_i64_0;
    s_i32_0 = l24;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 8usize, s_i32_0, host);
    vm.exit_call();
    let __t10 = __call_res?;
    s_i32_0 = __t10;
    l31 = s_i32_0;
    s_i64_0 = l30;
    s_i32_1 = l29;
    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
    s_i64_1 = ops::i64_extend_i32_u(s_i32_1);
    s_i32_0 = ops::i64_lt_u(s_i64_0, s_i64_1);
    'blk_7: {
        if s_i32_0 != 0 {
            s_i32_0 = l29;
            s_i64_1 = l30;
            s_i32_2 = l31;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i64_i32_ret_void(vm, c_map, 9usize, s_i32_0, s_i64_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = 1i32;
            l32 = s_i32_0;
            s_i32_0 = 0i32;
            l33 = s_i32_0;
            s_i32_0 = 4i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
            vm.exit_call();
            let __t11 = __call_res?;
            s_i32_0 = __t11;
            l34 = s_i32_0;
            s_i32_0 = l34;
            s_i32_1 = l32;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l34;
            s_i32_1 = l33;
            ops::i32_store8(vm, 0, s_i32_0, 1u32, s_i32_1)?;
            s_i32_0 = l34;
        } else {
            s_i32_0 = 0i32;
            l35 = s_i32_0;
            s_i32_0 = 2i32;
            l36 = s_i32_0;
            s_i32_0 = 4i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
            vm.exit_call();
            let __t12 = __call_res?;
            s_i32_0 = __t12;
            l37 = s_i32_0;
            s_i32_0 = l37;
            s_i32_1 = l35;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l37;
            s_i32_1 = l36;
            ops::i32_store8(vm, 0, s_i32_0, 1u32, s_i32_1)?;
            s_i32_0 = l37;
        }
    }
    l38 = s_i32_0;
    s_i32_0 = l38;
    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
    'blk_8: {
        if s_i32_0 != 0 {
        } else {
            s_i32_0 = l38;
            s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 1u32)?;
            l39 = s_i32_0;
        }
    }
    s_i32_0 = l0;
    l40 = s_i32_0;
    s_i32_0 = l29;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 8usize, s_i32_0, host);
    vm.exit_call();
    let __t13 = __call_res?;
    s_i32_0 = __t13;
    l41 = s_i32_0;
    s_i32_0 = 8i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
    vm.exit_call();
    let __t14 = __call_res?;
    s_i32_0 = __t14;
    l42 = s_i32_0;
    s_i32_0 = l42;
    s_i32_1 = l40;
    s_i32_2 = 8i32;
    vm.memory_copy(0, 0, s_i32_0, s_i32_1, s_i32_2, host)?;
    s_i32_0 = l42;
    s_i32_1 = l41;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l42;
    l43 = s_i32_0;
    s_i32_0 = l43;
    return Ok(s_i32_0);
    }

    pub(crate) fn f_81ab99176846(vm: &mut SubstituteVm, dispatch: &'static crate::ModuleDispatch, g_map: &'static [usize], c_map: &'static [usize], p0: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = 0i32;
    let mut l2: i64 = 0i64;
    let mut l3: i64 = 0i64;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i32 = 0i32;
    let mut l7: i32 = 0i32;
    let mut l8: i64 = 0i64;
    let mut l9: i64 = 0i64;
    let mut l10: i32 = 0i32;
    let mut l11: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i32_2: i32 = 0i32;
    let mut s_i64_2: i64 = 0i64;
    let mut s_i64_3: i64 = 0i64;
    vm.deduct_1(host)?;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_unit_ret_void(vm, c_map, 0usize, host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
    s_i64_1 = 0i64;
    s_i32_0 = ops::i64_eq(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
            l1 = s_i32_0;
            s_i32_0 = l1;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i32_0 = l0;
    s_i64_1 = 0i64;
    s_i64_2 = 1114112i64;
    l2 = s_i64_2;
    s_i64_2 = 0i64;
    l3 = s_i64_2;
    s_i32_2 = 16i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_2, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_2 = __t0;
    l4 = s_i32_2;
    s_i32_2 = l4;
    s_i64_3 = l2;
    ops::i64_store(vm, 0, s_i32_2, 0u32, s_i64_3)?;
    s_i32_2 = l4;
    s_i64_3 = l3;
    ops::i64_store(vm, 0, s_i32_2, 8u32, s_i64_3)?;
    s_i32_2 = l4;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i64_i32_ret_i32(vm, c_map, 2usize, s_i32_0, s_i64_1, s_i32_2, host);
    vm.exit_call();
    let __t1 = __call_res?;
    s_i32_0 = __t1;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
    vm.exit_call();
    let __t2 = __call_res?;
    s_i32_0 = __t2;
    l5 = s_i32_0;
    s_i32_0 = l5;
    s_i32_1 = 1927i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
        } else {
            s_i32_0 = l5;
            s_i32_1 = 2192i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_3: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                } else {
                    s_i32_0 = l5;
                    s_i32_1 = 2221i32;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                    'blk_4: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 1i32;
                        } else {
                            s_i32_0 = l5;
                            s_i32_1 = 2203i32;
                            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                            'blk_5: {
                                if s_i32_0 != 0 {
                                    s_i32_0 = 1i32;
                                } else {
                                    s_i32_0 = l5;
                                    s_i32_1 = 2240i32;
                                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                                    'blk_6: {
                                        if s_i32_0 != 0 {
                                            s_i32_0 = 1i32;
                                        } else {
                                            s_i32_0 = l5;
                                            s_i32_1 = 2206i32;
                                            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
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
    'blk_7: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
            l6 = s_i32_0;
            s_i32_0 = l6;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i32_0 = l5;
    s_i32_1 = 584i32;
    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
    'blk_8: {
        if s_i32_0 != 0 {
            s_i32_0 = l5;
            s_i32_1 = 593i32;
            s_i32_0 = ops::i32_le_u(s_i32_0, s_i32_1);
        } else {
            s_i32_0 = 0i32;
        }
    }
    'blk_9: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
        } else {
            s_i32_0 = l5;
            s_i32_1 = 610i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_10: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                } else {
                    s_i32_0 = l5;
                    s_i32_1 = 638i32;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                    'blk_11: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 1i32;
                        } else {
                            s_i32_0 = l5;
                            s_i32_1 = 646i32;
                            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                        }
                    }
                }
            }
        }
    }
    'blk_12: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
            l7 = s_i32_0;
            s_i32_0 = l7;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i32_0 = l5;
    s_i32_1 = 260i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_13: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
            s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
            s_i64_1 = 3i64;
            s_i32_0 = ops::i64_ge_u(s_i64_0, s_i64_1);
            'blk_14: {
                if s_i32_0 != 0 {
                    s_i32_0 = l0;
                    s_i64_1 = 2i64;
                    s_i64_2 = 1114112i64;
                    l8 = s_i64_2;
                    s_i64_2 = 0i64;
                    l9 = s_i64_2;
                    s_i32_2 = 16i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_2, host);
                    vm.exit_call();
                    let __t3 = __call_res?;
                    s_i32_2 = __t3;
                    l10 = s_i32_2;
                    s_i32_2 = l10;
                    s_i64_3 = l8;
                    ops::i64_store(vm, 0, s_i32_2, 0u32, s_i64_3)?;
                    s_i32_2 = l10;
                    s_i64_3 = l9;
                    ops::i64_store(vm, 0, s_i32_2, 8u32, s_i64_3)?;
                    s_i32_2 = l10;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i64_i32_ret_i32(vm, c_map, 2usize, s_i32_0, s_i64_1, s_i32_2, host);
                    vm.exit_call();
                    let __t4 = __call_res?;
                    s_i32_0 = __t4;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
                    vm.exit_call();
                    let __t5 = __call_res?;
                    s_i32_0 = __t5;
                    s_i32_1 = 280i32;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                } else {
                    s_i32_0 = 0i32;
                }
            }
        } else {
            s_i32_0 = 0i32;
        }
    }
    l11 = s_i32_0;
    s_i32_0 = l11;
    return Ok(s_i32_0);
    }

    pub(crate) fn f_bf98b8c73ee9(vm: &mut SubstituteVm, dispatch: &'static crate::ModuleDispatch, g_map: &'static [usize], c_map: &'static [usize], p0: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = 0i32;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i32 = 0i32;
    let mut l7: i32 = 0i32;
    let mut l8: i64 = 0i64;
    let mut l9: i64 = 0i64;
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
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i32_2: i32 = 0i32;
    let mut s_i64_2: i64 = 0i64;
    vm.deduct_1(host)?;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_unit_ret_void(vm, c_map, 0usize, host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i64_ret_i32(vm, c_map, 1usize, s_i64_0, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l1 = s_i32_0;
    s_i32_0 = l0;
    l2 = s_i32_0;
    s_i32_0 = l2;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    l3 = s_i32_0;
    s_i32_0 = l2;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    l4 = s_i32_0;
    s_i32_0 = 0i32;
    l5 = s_i32_0;
    'blk_1: {
        'blk_2: loop {
            vm.deduct_1(host)?;
            s_i32_0 = l5;
            s_i32_1 = l4;
            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
            if s_i32_0 != 0 {
                break 'blk_1;
            }
            s_i32_0 = l3;
            s_i32_1 = l5;
            s_i32_2 = 4i32;
            s_i32_1 = ops::i32_mul(s_i32_1, s_i32_2);
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
            l6 = s_i32_0;
            'blk_3: {
                s_i32_0 = l6;
                vm.enter_call()?;
                let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 2usize, s_i32_0, host);
                vm.exit_call();
                let __t1 = __call_res?;
                s_i32_0 = __t1;
                l7 = s_i32_0;
                s_i32_0 = l7;
                s_i64_1 = 127i64;
                l8 = s_i64_1;
                s_i64_1 = 0i64;
                l9 = s_i64_1;
                s_i32_1 = 16i32;
                vm.enter_call()?;
                let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_1, host);
                vm.exit_call();
                let __t2 = __call_res?;
                s_i32_1 = __t2;
                l10 = s_i32_1;
                s_i32_1 = l10;
                s_i64_2 = l8;
                ops::i64_store(vm, 0, s_i32_1, 0u32, s_i64_2)?;
                s_i32_1 = l10;
                s_i64_2 = l9;
                ops::i64_store(vm, 0, s_i32_1, 8u32, s_i64_2)?;
                s_i32_1 = l10;
                vm.enter_call()?;
                let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, host);
                vm.exit_call();
                let __t3 = __call_res?;
                s_i32_0 = __t3;
                'blk_4: {
                    if s_i32_0 != 0 {
                        s_i32_0 = 0i32;
                        l11 = s_i32_0;
                        s_i32_0 = 16i32;
                        vm.enter_call()?;
                        let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
                        vm.exit_call();
                        let __t4 = __call_res?;
                        s_i32_0 = __t4;
                        l12 = s_i32_0;
                        s_i32_0 = l12;
                        s_i32_1 = l12;
                        s_i32_2 = 16i32;
                        s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
                        ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                        s_i32_0 = l12;
                        s_i32_1 = 0i32;
                        ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                        s_i32_0 = l12;
                        s_i32_1 = 0i32;
                        ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                        s_i32_0 = l12;
                        s_i32_1 = 1i32;
                        ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                        s_i32_0 = l12;
                        l13 = s_i32_0;
                        s_i32_0 = 8i32;
                        vm.enter_call()?;
                        let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
                        vm.exit_call();
                        let __t5 = __call_res?;
                        s_i32_0 = __t5;
                        l14 = s_i32_0;
                        s_i32_0 = l14;
                        s_i32_1 = l11;
                        ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                        s_i32_0 = l14;
                        s_i32_1 = l13;
                        ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                        s_i32_0 = l14;
                        l15 = s_i32_0;
                        s_i32_0 = l15;
                        return Ok(s_i32_0);
                    } else {
                    }
                }
                s_i32_0 = l7;
                vm.enter_call()?;
                let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 5usize, s_i32_0, host);
                vm.exit_call();
                let __t6 = __call_res?;
                s_i32_0 = __t6;
                l16 = s_i32_0;
                s_i32_0 = l16;
                s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
                'blk_5: {
                    if s_i32_0 != 0 {
                        s_i32_0 = l16;
                        s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 1u32)?;
                        l17 = s_i32_0;
                        s_i32_0 = l1;
                        s_i32_1 = l17;
                        vm.enter_call()?;
                        let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 6usize, s_i32_0, s_i32_1, host);
                        vm.exit_call();
                        __call_res?;
                    } else {
                        s_i32_0 = l16;
                        s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 2u32)?;
                        l18 = s_i32_0;
                        s_i32_0 = 0i32;
                        l19 = s_i32_0;
                        s_i32_0 = 16i32;
                        vm.enter_call()?;
                        let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
                        vm.exit_call();
                        let __t7 = __call_res?;
                        s_i32_0 = __t7;
                        l20 = s_i32_0;
                        s_i32_0 = l20;
                        s_i32_1 = l20;
                        s_i32_2 = 16i32;
                        s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
                        ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                        s_i32_0 = l20;
                        s_i32_1 = 0i32;
                        ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                        s_i32_0 = l20;
                        s_i32_1 = 0i32;
                        ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                        s_i32_0 = l20;
                        s_i32_1 = 1i32;
                        ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                        s_i32_0 = l20;
                        l21 = s_i32_0;
                        s_i32_0 = 8i32;
                        vm.enter_call()?;
                        let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
                        vm.exit_call();
                        let __t8 = __call_res?;
                        s_i32_0 = __t8;
                        l22 = s_i32_0;
                        s_i32_0 = l22;
                        s_i32_1 = l19;
                        ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                        s_i32_0 = l22;
                        s_i32_1 = l21;
                        ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                        s_i32_0 = l22;
                        l23 = s_i32_0;
                        s_i32_0 = l23;
                        return Ok(s_i32_0);
                    }
                }
            }
            s_i32_0 = l5;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l5 = s_i32_0;
            continue 'blk_2;
            break 'blk_2;
        }
    }
    s_i32_0 = 1i32;
    l24 = s_i32_0;
    s_i32_0 = l1;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 7usize, s_i32_0, host);
    vm.exit_call();
    let __t9 = __call_res?;
    s_i32_0 = __t9;
    l25 = s_i32_0;
    s_i32_0 = 8i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
    vm.exit_call();
    let __t10 = __call_res?;
    s_i32_0 = __t10;
    l26 = s_i32_0;
    s_i32_0 = l26;
    s_i32_1 = l24;
    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l26;
    s_i32_1 = l25;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l26;
    l27 = s_i32_0;
    s_i32_0 = l27;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn f_b4a27025f49f(vm: &mut SubstituteVm, dispatch: &'static crate::ModuleDispatch, g_map: &'static [usize], c_map: &'static [usize], p0: i32, p1: i32, p2: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = p2;
    let mut l3: i64 = 0i64;
    let mut l4: i64 = 0i64;
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
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i32_2: i32 = 0i32;
    let mut s_i64_2: i64 = 0i64;
    let mut s_i64_3: i64 = 0i64;
    vm.deduct_1(host)?;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_unit_ret_void(vm, c_map, 0usize, host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_1 = l1;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_1 = __t0;
    s_i64_2 = 1114684i64;
    l3 = s_i64_2;
    s_i64_2 = 0i64;
    l4 = s_i64_2;
    s_i32_2 = 16i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 2usize, s_i32_2, host);
    vm.exit_call();
    let __t1 = __call_res?;
    s_i32_2 = __t1;
    l5 = s_i32_2;
    s_i32_2 = l5;
    s_i64_3 = l3;
    ops::i64_store(vm, 0, s_i32_2, 0u32, s_i64_3)?;
    s_i32_2 = l5;
    s_i64_3 = l4;
    ops::i64_store(vm, 0, s_i32_2, 8u32, s_i64_3)?;
    s_i32_2 = l5;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 3usize, s_i32_0, s_i32_1, s_i32_2, host);
    vm.exit_call();
    let __t2 = __call_res?;
    s_i32_0 = __t2;
    l6 = s_i32_0;
    s_i32_0 = l6;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    l7 = s_i32_0;
    s_i32_0 = l6;
    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 4u32)?;
    s_i32_0 = ops::i32_eqz(s_i32_0);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = l7;
            l8 = s_i32_0;
            s_i32_0 = 0i32;
            l9 = s_i32_0;
            s_i32_0 = 0i32;
            l10 = s_i32_0;
            s_i32_0 = l6;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 12u32)?;
            l11 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 2usize, s_i32_0, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i32_0 = __t3;
            l12 = s_i32_0;
            s_i32_0 = l12;
            s_i32_1 = l8;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l12;
            s_i32_1 = l9;
            ops::i32_store8(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l12;
            s_i32_1 = l10;
            ops::i32_store8(vm, 0, s_i32_0, 5u32, s_i32_1)?;
            s_i32_0 = l12;
            s_i32_1 = l11;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l12;
            l13 = s_i32_0;
            s_i32_0 = l13;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i32_0 = l6;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
    l14 = s_i32_0;
    s_i32_0 = l14;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    l15 = s_i32_0;
    s_i32_0 = l14;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    l16 = s_i32_0;
    s_i32_0 = 0i32;
    l17 = s_i32_0;
    'blk_2: {
        'blk_3: loop {
            vm.deduct_1(host)?;
            s_i32_0 = l17;
            s_i32_1 = l16;
            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
            if s_i32_0 != 0 {
                break 'blk_2;
            }
            s_i32_0 = l15;
            s_i32_1 = l17;
            s_i32_2 = 4i32;
            s_i32_1 = ops::i32_mul(s_i32_1, s_i32_2);
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
            l18 = s_i32_0;
            'blk_4: {
                s_i32_0 = l18;
                s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
                vm.enter_call()?;
                let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 4usize, s_i32_0, host);
                vm.exit_call();
                let __t4 = __call_res?;
                s_i32_0 = __t4;
                l19 = s_i32_0;
                s_i32_0 = l19;
                vm.enter_call()?;
                let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 5usize, s_i32_0, host);
                vm.exit_call();
                let __t5 = __call_res?;
                s_i32_0 = __t5;
                'blk_5: {
                    if s_i32_0 != 0 {
                        s_i32_0 = l7;
                        s_i32_1 = l19;
                        vm.enter_call()?;
                        let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 6usize, s_i32_0, s_i32_1, host);
                        vm.exit_call();
                        let __t6 = __call_res?;
                        s_i32_0 = __t6;
                        l20 = s_i32_0;
                        s_i32_0 = l20;
                        s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                        l7 = s_i32_0;
                        s_i32_0 = l20;
                        s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 4u32)?;
                        s_i32_1 = 2i32;
                        vm.enter_call()?;
                        let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 7usize, s_i32_0, s_i32_1, host);
                        vm.exit_call();
                        let __t7 = __call_res?;
                        s_i32_0 = __t7;
                        'blk_6: {
                            if s_i32_0 != 0 {
                                s_i32_0 = l7;
                                l21 = s_i32_0;
                                s_i32_0 = 0i32;
                                l22 = s_i32_0;
                                s_i32_0 = 0i32;
                                l23 = s_i32_0;
                                s_i32_0 = l20;
                                s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
                                l24 = s_i32_0;
                                s_i32_0 = 12i32;
                                vm.enter_call()?;
                                let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 2usize, s_i32_0, host);
                                vm.exit_call();
                                let __t8 = __call_res?;
                                s_i32_0 = __t8;
                                l25 = s_i32_0;
                                s_i32_0 = l25;
                                s_i32_1 = l21;
                                ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                                s_i32_0 = l25;
                                s_i32_1 = l22;
                                ops::i32_store8(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                                s_i32_0 = l25;
                                s_i32_1 = l23;
                                ops::i32_store8(vm, 0, s_i32_0, 5u32, s_i32_1)?;
                                s_i32_0 = l25;
                                s_i32_1 = l24;
                                ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                                s_i32_0 = l25;
                                l26 = s_i32_0;
                                s_i32_0 = l26;
                                return Ok(s_i32_0);
                            } else {
                            }
                        }
                        s_i32_0 = l20;
                        s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 4u32)?;
                        s_i32_1 = 1i32;
                        vm.enter_call()?;
                        let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 7usize, s_i32_0, s_i32_1, host);
                        vm.exit_call();
                        let __t9 = __call_res?;
                        s_i32_0 = __t9;
                        'blk_7: {
                            if s_i32_0 != 0 {
                                s_i32_0 = l20;
                                s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
                                vm.enter_call()?;
                                let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 8usize, s_i32_0, host);
                                vm.exit_call();
                                let __t10 = __call_res?;
                                s_i32_0 = __t10;
                                l27 = s_i32_0;
                                s_i32_0 = l27;
                                s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
                                'blk_8: {
                                    if s_i32_0 != 0 {
                                        s_i32_0 = l27;
                                        s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
                                        l28 = s_i32_0;
                                        s_i32_0 = l28;
                                        s_i32_1 = l2;
                                        vm.enter_call()?;
                                        let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 9usize, s_i32_0, s_i32_1, host);
                                        vm.exit_call();
                                        let __t11 = __call_res?;
                                        s_i32_0 = __t11;
                                        'blk_9: {
                                            if s_i32_0 != 0 {
                                                s_i32_0 = l7;
                                                l29 = s_i32_0;
                                                s_i32_0 = 1i32;
                                                l30 = s_i32_0;
                                                s_i32_0 = 1i32;
                                                l31 = s_i32_0;
                                                s_i32_0 = 16i32;
                                                vm.enter_call()?;
                                                let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 2usize, s_i32_0, host);
                                                vm.exit_call();
                                                let __t12 = __call_res?;
                                                s_i32_0 = __t12;
                                                l32 = s_i32_0;
                                                s_i32_0 = l32;
                                                s_i32_1 = l32;
                                                s_i32_2 = 16i32;
                                                s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
                                                ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                                                s_i32_0 = l32;
                                                s_i32_1 = 0i32;
                                                ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                                                s_i32_0 = l32;
                                                s_i32_1 = 0i32;
                                                ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                                                s_i32_0 = l32;
                                                s_i32_1 = 1i32;
                                                ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                                                s_i32_0 = l32;
                                                l33 = s_i32_0;
                                                s_i32_0 = 12i32;
                                                vm.enter_call()?;
                                                let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 2usize, s_i32_0, host);
                                                vm.exit_call();
                                                let __t13 = __call_res?;
                                                s_i32_0 = __t13;
                                                l34 = s_i32_0;
                                                s_i32_0 = l34;
                                                s_i32_1 = l29;
                                                ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                                                s_i32_0 = l34;
                                                s_i32_1 = l30;
                                                ops::i32_store8(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                                                s_i32_0 = l34;
                                                s_i32_1 = l31;
                                                ops::i32_store8(vm, 0, s_i32_0, 5u32, s_i32_1)?;
                                                s_i32_0 = l34;
                                                s_i32_1 = l33;
                                                ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                                                s_i32_0 = l34;
                                                l35 = s_i32_0;
                                                s_i32_0 = l35;
                                                return Ok(s_i32_0);
                                            } else {
                                            }
                                        }
                                    } else {
                                    }
                                }
                            } else {
                            }
                        }
                    } else {
                    }
                }
            }
            s_i32_0 = l17;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l17 = s_i32_0;
            continue 'blk_3;
            break 'blk_3;
        }
    }
    s_i32_0 = l7;
    l36 = s_i32_0;
    s_i32_0 = 1i32;
    l37 = s_i32_0;
    s_i32_0 = 0i32;
    l38 = s_i32_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 2usize, s_i32_0, host);
    vm.exit_call();
    let __t14 = __call_res?;
    s_i32_0 = __t14;
    l39 = s_i32_0;
    s_i32_0 = l39;
    s_i32_1 = l39;
    s_i32_2 = 16i32;
    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l39;
    s_i32_1 = 0i32;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l39;
    s_i32_1 = 0i32;
    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
    s_i32_0 = l39;
    s_i32_1 = 1i32;
    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
    s_i32_0 = l39;
    l40 = s_i32_0;
    s_i32_0 = 12i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 2usize, s_i32_0, host);
    vm.exit_call();
    let __t15 = __call_res?;
    s_i32_0 = __t15;
    l41 = s_i32_0;
    s_i32_0 = l41;
    s_i32_1 = l36;
    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l41;
    s_i32_1 = l37;
    ops::i32_store8(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l41;
    s_i32_1 = l38;
    ops::i32_store8(vm, 0, s_i32_0, 5u32, s_i32_1)?;
    s_i32_0 = l41;
    s_i32_1 = l40;
    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
    s_i32_0 = l41;
    l42 = s_i32_0;
    s_i32_0 = l42;
    return Ok(s_i32_0);
    }

    pub(crate) fn f_be76f48db77a(vm: &mut SubstituteVm, dispatch: &'static crate::ModuleDispatch, g_map: &'static [usize], c_map: &'static [usize], p0: i32, p1: i32, p2: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = p2;
    let mut l3: i64 = 0i64;
    let mut l4: i64 = 0i64;
    let mut l5: i32 = 0i32;
    let mut l6: i64 = 0i64;
    let mut l7: i64 = 0i64;
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
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i32_2: i32 = 0i32;
    let mut s_i64_2: i64 = 0i64;
    let mut s_i32_3: i32 = 0i32;
    let mut s_i64_3: i64 = 0i64;
    let mut s_i32_4: i32 = 0i32;
    let mut s_i32_5: i32 = 0i32;
    let mut s_i32_6: i32 = 0i32;
    let mut s_i64_6: i64 = 0i64;
    let mut s_i64_7: i64 = 0i64;
    vm.deduct_1(host)?;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_unit_ret_void(vm, c_map, 0usize, host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_1 = 1i32;
    s_i64_2 = 1114698i64;
    l3 = s_i64_2;
    s_i64_2 = 0i64;
    l4 = s_i64_2;
    s_i32_2 = 16i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_2, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_2 = __t0;
    l5 = s_i32_2;
    s_i32_2 = l5;
    s_i64_3 = l3;
    ops::i64_store(vm, 0, s_i32_2, 0u32, s_i64_3)?;
    s_i32_2 = l5;
    s_i64_3 = l4;
    ops::i64_store(vm, 0, s_i32_2, 8u32, s_i64_3)?;
    s_i32_2 = l5;
    s_i32_3 = 1i32;
    s_i32_4 = l1;
    s_i32_5 = 0i32;
    s_i64_6 = 1114112i64;
    l6 = s_i64_6;
    s_i64_6 = 0i64;
    l7 = s_i64_6;
    s_i32_6 = 16i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_6, host);
    vm.exit_call();
    let __t1 = __call_res?;
    s_i32_6 = __t1;
    l8 = s_i32_6;
    s_i32_6 = l8;
    s_i64_7 = l6;
    ops::i64_store(vm, 0, s_i32_6, 0u32, s_i64_7)?;
    s_i32_6 = l8;
    s_i64_7 = l7;
    ops::i64_store(vm, 0, s_i32_6, 8u32, s_i64_7)?;
    s_i32_6 = l8;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i32_i32_i32_i32_i32_i32_ret_i32(vm, c_map, 2usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, s_i32_4, s_i32_5, s_i32_6, host);
    vm.exit_call();
    let __t2 = __call_res?;
    s_i32_0 = __t2;
    l9 = s_i32_0;
    s_i32_0 = l9;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    l10 = s_i32_0;
    s_i32_0 = l9;
    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 4u32)?;
    s_i32_0 = ops::i32_eqz(s_i32_0);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = l10;
            l11 = s_i32_0;
            s_i32_0 = 0i32;
            l12 = s_i32_0;
            s_i32_0 = 0i32;
            l13 = s_i32_0;
            s_i32_0 = l9;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 12u32)?;
            l14 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_0, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i32_0 = __t3;
            l15 = s_i32_0;
            s_i32_0 = l15;
            s_i32_1 = l11;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l15;
            s_i32_1 = l12;
            ops::i32_store8(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l15;
            s_i32_1 = l13;
            ops::i32_store8(vm, 0, s_i32_0, 5u32, s_i32_1)?;
            s_i32_0 = l15;
            s_i32_1 = l14;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l15;
            l16 = s_i32_0;
            s_i32_0 = l16;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i32_0 = l9;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
    l17 = s_i32_0;
    s_i32_0 = l17;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    l18 = s_i32_0;
    s_i32_0 = l17;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    l19 = s_i32_0;
    s_i32_0 = 0i32;
    l20 = s_i32_0;
    'blk_2: {
        'blk_3: loop {
            vm.deduct_1(host)?;
            s_i32_0 = l20;
            s_i32_1 = l19;
            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
            if s_i32_0 != 0 {
                break 'blk_2;
            }
            s_i32_0 = l18;
            s_i32_1 = l20;
            s_i32_2 = 4i32;
            s_i32_1 = ops::i32_mul(s_i32_1, s_i32_2);
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
            l21 = s_i32_0;
            'blk_4: {
                s_i32_0 = l21;
                s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 12u32)?;
                vm.enter_call()?;
                let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
                vm.exit_call();
                let __t4 = __call_res?;
                s_i32_0 = __t4;
                l22 = s_i32_0;
                s_i32_0 = l22;
                vm.enter_call()?;
                let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 4usize, s_i32_0, host);
                vm.exit_call();
                let __t5 = __call_res?;
                s_i32_0 = __t5;
                'blk_5: {
                    if s_i32_0 != 0 {
                        s_i32_0 = l10;
                        s_i32_1 = l22;
                        s_i32_2 = l2;
                        vm.enter_call()?;
                        let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i32_2, host);
                        vm.exit_call();
                        let __t6 = __call_res?;
                        s_i32_0 = __t6;
                        l23 = s_i32_0;
                        s_i32_0 = l23;
                        s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                        l10 = s_i32_0;
                        s_i32_0 = l23;
                        s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 4u32)?;
                        s_i32_0 = ops::i32_eqz(s_i32_0);
                        'blk_6: {
                            if s_i32_0 != 0 {
                                s_i32_0 = l23;
                                l24 = s_i32_0;
                                s_i32_0 = l24;
                                return Ok(s_i32_0);
                            } else {
                            }
                        }
                        s_i32_0 = l23;
                        s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 5u32)?;
                        'blk_7: {
                            if s_i32_0 != 0 {
                                s_i32_0 = l10;
                                l25 = s_i32_0;
                                s_i32_0 = 1i32;
                                l26 = s_i32_0;
                                s_i32_0 = 1i32;
                                l27 = s_i32_0;
                                s_i32_0 = 16i32;
                                vm.enter_call()?;
                                let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_0, host);
                                vm.exit_call();
                                let __t7 = __call_res?;
                                s_i32_0 = __t7;
                                l28 = s_i32_0;
                                s_i32_0 = l28;
                                s_i32_1 = l28;
                                s_i32_2 = 16i32;
                                s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
                                ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                                s_i32_0 = l28;
                                s_i32_1 = 0i32;
                                ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                                s_i32_0 = l28;
                                s_i32_1 = 0i32;
                                ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                                s_i32_0 = l28;
                                s_i32_1 = 1i32;
                                ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                                s_i32_0 = l28;
                                l29 = s_i32_0;
                                s_i32_0 = 12i32;
                                vm.enter_call()?;
                                let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_0, host);
                                vm.exit_call();
                                let __t8 = __call_res?;
                                s_i32_0 = __t8;
                                l30 = s_i32_0;
                                s_i32_0 = l30;
                                s_i32_1 = l25;
                                ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                                s_i32_0 = l30;
                                s_i32_1 = l26;
                                ops::i32_store8(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                                s_i32_0 = l30;
                                s_i32_1 = l27;
                                ops::i32_store8(vm, 0, s_i32_0, 5u32, s_i32_1)?;
                                s_i32_0 = l30;
                                s_i32_1 = l29;
                                ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                                s_i32_0 = l30;
                                l31 = s_i32_0;
                                s_i32_0 = l31;
                                return Ok(s_i32_0);
                            } else {
                            }
                        }
                    } else {
                    }
                }
            }
            s_i32_0 = l20;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l20 = s_i32_0;
            continue 'blk_3;
            break 'blk_3;
        }
    }
    s_i32_0 = l10;
    l32 = s_i32_0;
    s_i32_0 = 1i32;
    l33 = s_i32_0;
    s_i32_0 = 0i32;
    l34 = s_i32_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_0, host);
    vm.exit_call();
    let __t9 = __call_res?;
    s_i32_0 = __t9;
    l35 = s_i32_0;
    s_i32_0 = l35;
    s_i32_1 = l35;
    s_i32_2 = 16i32;
    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l35;
    s_i32_1 = 0i32;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l35;
    s_i32_1 = 0i32;
    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
    s_i32_0 = l35;
    s_i32_1 = 1i32;
    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
    s_i32_0 = l35;
    l36 = s_i32_0;
    s_i32_0 = 12i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_0, host);
    vm.exit_call();
    let __t10 = __call_res?;
    s_i32_0 = __t10;
    l37 = s_i32_0;
    s_i32_0 = l37;
    s_i32_1 = l32;
    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l37;
    s_i32_1 = l33;
    ops::i32_store8(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l37;
    s_i32_1 = l34;
    ops::i32_store8(vm, 0, s_i32_0, 5u32, s_i32_1)?;
    s_i32_0 = l37;
    s_i32_1 = l36;
    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
    s_i32_0 = l37;
    l38 = s_i32_0;
    s_i32_0 = l38;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn f_eeae7770e899(vm: &mut SubstituteVm, dispatch: &'static crate::ModuleDispatch, g_map: &'static [usize], c_map: &'static [usize], p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut l3: i64 = 0i64;
    let mut l4: i64 = 0i64;
    let mut l5: i32 = 0i32;
    let mut l6: i32 = 0i32;
    let mut l7: i64 = 0i64;
    let mut l8: i64 = 0i64;
    let mut l9: i32 = 0i32;
    let mut l10: i32 = 0i32;
    let mut l11: i64 = 0i64;
    let mut l12: i64 = 0i64;
    let mut l13: i32 = 0i32;
    let mut l14: i32 = 0i32;
    let mut l15: i64 = 0i64;
    let mut l16: i64 = 0i64;
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
    let mut l54: i32 = 0i32;
    let mut l55: i32 = 0i32;
    let mut l56: i32 = 0i32;
    let mut l57: i32 = 0i32;
    let mut l58: i32 = 0i32;
    let mut l59: i32 = 0i32;
    let mut l60: i32 = 0i32;
    let mut l61: i32 = 0i32;
    let mut l62: i32 = 0i32;
    let mut l63: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i32_2: i32 = 0i32;
    vm.deduct_1(host)?;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_unit_ret_void(vm, c_map, 0usize, host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    l2 = s_i32_0;
    s_i64_0 = 1114226i64;
    l3 = s_i64_0;
    s_i64_0 = 0i64;
    l4 = s_i64_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_0, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l5 = s_i32_0;
    s_i32_0 = l5;
    s_i64_1 = l3;
    ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
    s_i32_0 = l5;
    s_i64_1 = l4;
    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
    s_i32_0 = l5;
    l6 = s_i32_0;
    s_i64_0 = 1114227i64;
    l7 = s_i64_0;
    s_i64_0 = 0i64;
    l8 = s_i64_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_0, host);
    vm.exit_call();
    let __t1 = __call_res?;
    s_i32_0 = __t1;
    l9 = s_i32_0;
    s_i32_0 = l9;
    s_i64_1 = l7;
    ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
    s_i32_0 = l9;
    s_i64_1 = l8;
    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
    s_i32_0 = l9;
    l10 = s_i32_0;
    s_i64_0 = 1114347i64;
    l11 = s_i64_0;
    s_i64_0 = 0i64;
    l12 = s_i64_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_0, host);
    vm.exit_call();
    let __t2 = __call_res?;
    s_i32_0 = __t2;
    l13 = s_i32_0;
    s_i32_0 = l13;
    s_i64_1 = l11;
    ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
    s_i32_0 = l13;
    s_i64_1 = l12;
    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
    s_i32_0 = l13;
    l14 = s_i32_0;
    s_i64_0 = 1114348i64;
    l15 = s_i64_0;
    s_i64_0 = 0i64;
    l16 = s_i64_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_0, host);
    vm.exit_call();
    let __t3 = __call_res?;
    s_i32_0 = __t3;
    l17 = s_i32_0;
    s_i32_0 = l17;
    s_i64_1 = l15;
    ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
    s_i32_0 = l17;
    s_i64_1 = l16;
    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
    s_i32_0 = l17;
    l18 = s_i32_0;
    s_i32_0 = 32i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_0, host);
    vm.exit_call();
    let __t4 = __call_res?;
    s_i32_0 = __t4;
    l19 = s_i32_0;
    s_i32_0 = l19;
    s_i32_1 = l19;
    s_i32_2 = 16i32;
    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l19;
    s_i32_1 = 4i32;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l19;
    s_i32_1 = 4i32;
    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
    s_i32_0 = l19;
    s_i32_1 = 1i32;
    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
    s_i32_0 = l19;
    s_i32_1 = l6;
    ops::i32_store(vm, 0, s_i32_0, 16u32, s_i32_1)?;
    s_i32_0 = l19;
    s_i32_1 = l10;
    ops::i32_store(vm, 0, s_i32_0, 20u32, s_i32_1)?;
    s_i32_0 = l19;
    s_i32_1 = l14;
    ops::i32_store(vm, 0, s_i32_0, 24u32, s_i32_1)?;
    s_i32_0 = l19;
    s_i32_1 = l18;
    ops::i32_store(vm, 0, s_i32_0, 28u32, s_i32_1)?;
    s_i32_0 = l19;
    l20 = s_i32_0;
    s_i32_0 = l20;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    l21 = s_i32_0;
    s_i32_0 = l20;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    l22 = s_i32_0;
    s_i32_0 = 0i32;
    l23 = s_i32_0;
    'blk_1: {
        'blk_2: loop {
            vm.deduct_1(host)?;
            s_i32_0 = l23;
            s_i32_1 = l22;
            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
            if s_i32_0 != 0 {
                break 'blk_1;
            }
            s_i32_0 = l21;
            s_i32_1 = l23;
            s_i32_2 = 4i32;
            s_i32_1 = ops::i32_mul(s_i32_1, s_i32_2);
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
            l24 = s_i32_0;
            'blk_3: {
                s_i32_0 = l2;
                s_i32_1 = l1;
                s_i32_2 = l24;
                vm.enter_call()?;
                let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 2usize, s_i32_0, s_i32_1, s_i32_2, host);
                vm.exit_call();
                let __t5 = __call_res?;
                s_i32_0 = __t5;
                l25 = s_i32_0;
                s_i32_0 = l25;
                s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                l2 = s_i32_0;
                s_i32_0 = l25;
                s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 4u32)?;
                s_i32_0 = ops::i32_eqz(s_i32_0);
                'blk_4: {
                    if s_i32_0 != 0 {
                        s_i32_0 = l25;
                        l26 = s_i32_0;
                        s_i32_0 = l26;
                        return Ok(s_i32_0);
                    } else {
                    }
                }
                s_i32_0 = l25;
                s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 5u32)?;
                'blk_5: {
                    if s_i32_0 != 0 {
                        s_i32_0 = l2;
                        l27 = s_i32_0;
                        s_i32_0 = 1i32;
                        l28 = s_i32_0;
                        s_i32_0 = 1i32;
                        l29 = s_i32_0;
                        s_i32_0 = 16i32;
                        vm.enter_call()?;
                        let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_0, host);
                        vm.exit_call();
                        let __t6 = __call_res?;
                        s_i32_0 = __t6;
                        l30 = s_i32_0;
                        s_i32_0 = l30;
                        s_i32_1 = l30;
                        s_i32_2 = 16i32;
                        s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
                        ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                        s_i32_0 = l30;
                        s_i32_1 = 0i32;
                        ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                        s_i32_0 = l30;
                        s_i32_1 = 0i32;
                        ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                        s_i32_0 = l30;
                        s_i32_1 = 1i32;
                        ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                        s_i32_0 = l30;
                        l31 = s_i32_0;
                        s_i32_0 = 12i32;
                        vm.enter_call()?;
                        let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_0, host);
                        vm.exit_call();
                        let __t7 = __call_res?;
                        s_i32_0 = __t7;
                        l32 = s_i32_0;
                        s_i32_0 = l32;
                        s_i32_1 = l27;
                        ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                        s_i32_0 = l32;
                        s_i32_1 = l28;
                        ops::i32_store8(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                        s_i32_0 = l32;
                        s_i32_1 = l29;
                        ops::i32_store8(vm, 0, s_i32_0, 5u32, s_i32_1)?;
                        s_i32_0 = l32;
                        s_i32_1 = l31;
                        ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                        s_i32_0 = l32;
                        l33 = s_i32_0;
                        s_i32_0 = l33;
                        return Ok(s_i32_0);
                    } else {
                    }
                }
            }
            s_i32_0 = l23;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l23 = s_i32_0;
            continue 'blk_2;
            break 'blk_2;
        }
    }
    s_i32_0 = 24i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_0, host);
    vm.exit_call();
    let __t8 = __call_res?;
    s_i32_0 = __t8;
    l34 = s_i32_0;
    s_i32_0 = l34;
    s_i32_1 = l34;
    s_i32_2 = 16i32;
    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l34;
    s_i32_1 = 8i32;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l34;
    s_i32_1 = 8i32;
    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
    s_i32_0 = l34;
    s_i32_1 = 1i32;
    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
    s_i32_0 = l34;
    s_i64_1 = 7453010347959800622i64;
    ops::i64_store(vm, 0, s_i32_0, 16u32, s_i64_1)?;
    s_i32_0 = l34;
    l35 = s_i32_0;
    s_i32_0 = 20i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_0, host);
    vm.exit_call();
    let __t9 = __call_res?;
    s_i32_0 = __t9;
    l36 = s_i32_0;
    s_i32_0 = l36;
    s_i32_1 = l36;
    s_i32_2 = 16i32;
    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l36;
    s_i32_1 = 4i32;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l36;
    s_i32_1 = 4i32;
    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
    s_i32_0 = l36;
    s_i32_1 = 1i32;
    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
    s_i32_0 = l36;
    s_i32_1 = 46i32;
    ops::i32_store8(vm, 0, s_i32_0, 16u32, s_i32_1)?;
    s_i32_0 = l36;
    s_i32_1 = 68i32;
    ops::i32_store8(vm, 0, s_i32_0, 17u32, s_i32_1)?;
    s_i32_0 = l36;
    s_i32_1 = 67i32;
    ops::i32_store8(vm, 0, s_i32_0, 18u32, s_i32_1)?;
    s_i32_0 = l36;
    s_i32_1 = 69i32;
    ops::i32_store8(vm, 0, s_i32_0, 19u32, s_i32_1)?;
    s_i32_0 = l36;
    l37 = s_i32_0;
    s_i32_0 = 27i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_0, host);
    vm.exit_call();
    let __t10 = __call_res?;
    s_i32_0 = __t10;
    l38 = s_i32_0;
    s_i32_0 = l38;
    s_i32_1 = l38;
    s_i32_2 = 16i32;
    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l38;
    s_i32_1 = 11i32;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l38;
    s_i32_1 = 11i32;
    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
    s_i32_0 = l38;
    s_i32_1 = 1i32;
    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
    s_i32_0 = l38;
    s_i64_1 = 7307216977846813742i64;
    ops::i64_store(vm, 0, s_i32_0, 16u32, s_i64_1)?;
    s_i32_0 = l38;
    s_i32_1 = 101i32;
    ops::i32_store8(vm, 0, s_i32_0, 24u32, s_i32_1)?;
    s_i32_0 = l38;
    s_i32_1 = 116i32;
    ops::i32_store8(vm, 0, s_i32_0, 25u32, s_i32_1)?;
    s_i32_0 = l38;
    s_i32_1 = 115i32;
    ops::i32_store8(vm, 0, s_i32_0, 26u32, s_i32_1)?;
    s_i32_0 = l38;
    l39 = s_i32_0;
    s_i32_0 = 29i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_0, host);
    vm.exit_call();
    let __t11 = __call_res?;
    s_i32_0 = __t11;
    l40 = s_i32_0;
    s_i32_0 = l40;
    s_i32_1 = l40;
    s_i32_2 = 16i32;
    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l40;
    s_i32_1 = 13i32;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l40;
    s_i32_1 = 13i32;
    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
    s_i32_0 = l40;
    s_i32_1 = 1i32;
    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
    s_i32_0 = l40;
    s_i64_1 = 8243124573944628270i64;
    ops::i64_store(vm, 0, s_i32_0, 16u32, s_i64_1)?;
    s_i32_0 = l40;
    s_i32_1 = 115i32;
    ops::i32_store8(vm, 0, s_i32_0, 24u32, s_i32_1)?;
    s_i32_0 = l40;
    s_i32_1 = 105i32;
    ops::i32_store8(vm, 0, s_i32_0, 25u32, s_i32_1)?;
    s_i32_0 = l40;
    s_i32_1 = 111i32;
    ops::i32_store8(vm, 0, s_i32_0, 26u32, s_i32_1)?;
    s_i32_0 = l40;
    s_i32_1 = 110i32;
    ops::i32_store8(vm, 0, s_i32_0, 27u32, s_i32_1)?;
    s_i32_0 = l40;
    s_i32_1 = 115i32;
    ops::i32_store8(vm, 0, s_i32_0, 28u32, s_i32_1)?;
    s_i32_0 = l40;
    l41 = s_i32_0;
    s_i32_0 = 32i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_0, host);
    vm.exit_call();
    let __t12 = __call_res?;
    s_i32_0 = __t12;
    l42 = s_i32_0;
    s_i32_0 = l42;
    s_i32_1 = l42;
    s_i32_2 = 16i32;
    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l42;
    s_i32_1 = 4i32;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l42;
    s_i32_1 = 4i32;
    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
    s_i32_0 = l42;
    s_i32_1 = 1i32;
    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
    s_i32_0 = l42;
    s_i32_1 = l35;
    ops::i32_store(vm, 0, s_i32_0, 16u32, s_i32_1)?;
    s_i32_0 = l42;
    s_i32_1 = l37;
    ops::i32_store(vm, 0, s_i32_0, 20u32, s_i32_1)?;
    s_i32_0 = l42;
    s_i32_1 = l39;
    ops::i32_store(vm, 0, s_i32_0, 24u32, s_i32_1)?;
    s_i32_0 = l42;
    s_i32_1 = l41;
    ops::i32_store(vm, 0, s_i32_0, 28u32, s_i32_1)?;
    s_i32_0 = l42;
    l43 = s_i32_0;
    s_i32_0 = l43;
    vm.deduct_1(host)?;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    l44 = s_i32_0;
    s_i32_0 = l43;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    l45 = s_i32_0;
    s_i32_0 = 0i32;
    l46 = s_i32_0;
    'blk_6: {
        'blk_7: loop {
            vm.deduct_1(host)?;
            s_i32_0 = l46;
            s_i32_1 = l45;
            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
            if s_i32_0 != 0 {
                break 'blk_6;
            }
            s_i32_0 = l44;
            s_i32_1 = l46;
            s_i32_2 = 4i32;
            s_i32_1 = ops::i32_mul(s_i32_1, s_i32_2);
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
            l47 = s_i32_0;
            'blk_8: {
                s_i32_0 = l2;
                s_i32_1 = l1;
                s_i32_2 = l47;
                vm.enter_call()?;
                let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 3usize, s_i32_0, s_i32_1, s_i32_2, host);
                vm.exit_call();
                let __t13 = __call_res?;
                s_i32_0 = __t13;
                l48 = s_i32_0;
                s_i32_0 = l48;
                s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                l2 = s_i32_0;
                s_i32_0 = l48;
                s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 4u32)?;
                s_i32_0 = ops::i32_eqz(s_i32_0);
                'blk_9: {
                    if s_i32_0 != 0 {
                        s_i32_0 = l48;
                        l49 = s_i32_0;
                        s_i32_0 = l49;
                        return Ok(s_i32_0);
                    } else {
                    }
                }
                s_i32_0 = l48;
                s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 5u32)?;
                'blk_10: {
                    if s_i32_0 != 0 {
                        s_i32_0 = l2;
                        l50 = s_i32_0;
                        s_i32_0 = 1i32;
                        l51 = s_i32_0;
                        s_i32_0 = 1i32;
                        l52 = s_i32_0;
                        s_i32_0 = 16i32;
                        vm.enter_call()?;
                        let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_0, host);
                        vm.exit_call();
                        let __t14 = __call_res?;
                        s_i32_0 = __t14;
                        l53 = s_i32_0;
                        s_i32_0 = l53;
                        s_i32_1 = l53;
                        s_i32_2 = 16i32;
                        s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
                        ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                        s_i32_0 = l53;
                        s_i32_1 = 0i32;
                        ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                        s_i32_0 = l53;
                        s_i32_1 = 0i32;
                        ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                        s_i32_0 = l53;
                        s_i32_1 = 1i32;
                        ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                        s_i32_0 = l53;
                        l54 = s_i32_0;
                        s_i32_0 = 12i32;
                        vm.enter_call()?;
                        let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_0, host);
                        vm.exit_call();
                        let __t15 = __call_res?;
                        s_i32_0 = __t15;
                        l55 = s_i32_0;
                        s_i32_0 = l55;
                        s_i32_1 = l50;
                        ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                        s_i32_0 = l55;
                        s_i32_1 = l51;
                        ops::i32_store8(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                        s_i32_0 = l55;
                        s_i32_1 = l52;
                        ops::i32_store8(vm, 0, s_i32_0, 5u32, s_i32_1)?;
                        s_i32_0 = l55;
                        s_i32_1 = l54;
                        ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                        s_i32_0 = l55;
                        l56 = s_i32_0;
                        s_i32_0 = l56;
                        return Ok(s_i32_0);
                    } else {
                    }
                }
            }
            s_i32_0 = l46;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l46 = s_i32_0;
            continue 'blk_7;
            break 'blk_7;
        }
    }
    s_i32_0 = l2;
    l57 = s_i32_0;
    s_i32_0 = 1i32;
    l58 = s_i32_0;
    s_i32_0 = 0i32;
    l59 = s_i32_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_0, host);
    vm.exit_call();
    let __t16 = __call_res?;
    s_i32_0 = __t16;
    l60 = s_i32_0;
    s_i32_0 = l60;
    s_i32_1 = l60;
    s_i32_2 = 16i32;
    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l60;
    s_i32_1 = 0i32;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l60;
    s_i32_1 = 0i32;
    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
    s_i32_0 = l60;
    s_i32_1 = 1i32;
    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
    s_i32_0 = l60;
    l61 = s_i32_0;
    s_i32_0 = 12i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_0, host);
    vm.exit_call();
    let __t17 = __call_res?;
    s_i32_0 = __t17;
    l62 = s_i32_0;
    s_i32_0 = l62;
    s_i32_1 = l57;
    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l62;
    s_i32_1 = l58;
    ops::i32_store8(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l62;
    s_i32_1 = l59;
    ops::i32_store8(vm, 0, s_i32_0, 5u32, s_i32_1)?;
    s_i32_0 = l62;
    s_i32_1 = l61;
    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
    s_i32_0 = l62;
    l63 = s_i32_0;
    s_i32_0 = l63;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn f_cc184be07b93(vm: &mut SubstituteVm, dispatch: &'static crate::ModuleDispatch, g_map: &'static [usize], c_map: &'static [usize], p0: i32, p1: i32, p2: i32, p3: i32, p4: i64, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = p2;
    let mut l3: i32 = p3;
    let mut l4: i64 = p4;
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
    let mut l54: i32 = 0i32;
    let mut l55: i32 = 0i32;
    let mut l56: i32 = 0i32;
    let mut l57: i32 = 0i32;
    let mut l58: i32 = 0i32;
    let mut l59: i32 = 0i32;
    let mut l60: i32 = 0i32;
    let mut l61: i32 = 0i32;
    let mut l62: i32 = 0i32;
    let mut l63: i32 = 0i32;
    let mut l64: i32 = 0i32;
    let mut l65: i32 = 0i32;
    let mut l66: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i32_2: i32 = 0i32;
    let mut s_i64_2: i64 = 0i64;
    let mut s_i64_3: i64 = 0i64;
    vm.deduct_1(host)?;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_unit_ret_void(vm, c_map, 0usize, host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    l5 = s_i32_0;
    s_i32_0 = l1;
    l6 = s_i32_0;
    s_i32_0 = l6;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    l7 = s_i32_0;
    s_i32_0 = l6;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    l8 = s_i32_0;
    s_i32_0 = 0i32;
    l9 = s_i32_0;
    'blk_1: {
        'blk_2: loop {
            vm.deduct_1(host)?;
            s_i32_0 = l9;
            s_i32_1 = l8;
            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
            if s_i32_0 != 0 {
                break 'blk_1;
            }
            s_i32_0 = l7;
            s_i32_1 = l9;
            s_i32_2 = 4i32;
            s_i32_1 = ops::i32_mul(s_i32_1, s_i32_2);
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
            l10 = s_i32_0;
            'blk_3: {
                s_i32_0 = l5;
                s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                s_i32_1 = l10;
                vm.enter_call()?;
                let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_1, host);
                vm.exit_call();
                let __t0 = __call_res?;
                s_i32_1 = __t0;
                s_i32_2 = l2;
                vm.enter_call()?;
                let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 2usize, s_i32_0, s_i32_1, s_i32_2, host);
                vm.exit_call();
                let __t1 = __call_res?;
                s_i32_0 = __t1;
                l11 = s_i32_0;
                s_i32_0 = l5;
                l12 = s_i32_0;
                s_i32_0 = l11;
                s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                l13 = s_i32_0;
                s_i32_0 = 8i32;
                vm.enter_call()?;
                let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
                vm.exit_call();
                let __t2 = __call_res?;
                s_i32_0 = __t2;
                l14 = s_i32_0;
                s_i32_0 = l14;
                s_i32_1 = l12;
                s_i32_2 = 8i32;
                vm.memory_copy(0, 0, s_i32_0, s_i32_1, s_i32_2, host)?;
                s_i32_0 = l14;
                s_i32_1 = l13;
                ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                s_i32_0 = l14;
                l5 = s_i32_0;
                s_i32_0 = l11;
                s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 4u32)?;
                s_i32_0 = ops::i32_eqz(s_i32_0);
                'blk_4: {
                    if s_i32_0 != 0 {
                        s_i32_0 = l5;
                        l15 = s_i32_0;
                        s_i32_0 = 0i32;
                        l16 = s_i32_0;
                        s_i32_0 = 2i32;
                        l17 = s_i32_0;
                        s_i32_0 = 16i32;
                        vm.enter_call()?;
                        let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
                        vm.exit_call();
                        let __t3 = __call_res?;
                        s_i32_0 = __t3;
                        l18 = s_i32_0;
                        s_i32_0 = l18;
                        s_i32_1 = l18;
                        s_i32_2 = 16i32;
                        s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
                        ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                        s_i32_0 = l18;
                        s_i32_1 = 0i32;
                        ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                        s_i32_0 = l18;
                        s_i32_1 = 0i32;
                        ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                        s_i32_0 = l18;
                        s_i32_1 = 1i32;
                        ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                        s_i32_0 = l18;
                        l19 = s_i32_0;
                        s_i32_0 = 12i32;
                        vm.enter_call()?;
                        let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
                        vm.exit_call();
                        let __t4 = __call_res?;
                        s_i32_0 = __t4;
                        l20 = s_i32_0;
                        s_i32_0 = l20;
                        s_i32_1 = l15;
                        ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                        s_i32_0 = l20;
                        s_i32_1 = l16;
                        ops::i32_store8(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                        s_i32_0 = l20;
                        s_i32_1 = l17;
                        ops::i32_store8(vm, 0, s_i32_0, 5u32, s_i32_1)?;
                        s_i32_0 = l20;
                        s_i32_1 = l19;
                        ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                        s_i32_0 = l20;
                        l21 = s_i32_0;
                        s_i32_0 = l21;
                        return Ok(s_i32_0);
                    } else {
                    }
                }
                s_i32_0 = l11;
                s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
                l22 = s_i32_0;
                s_i32_0 = l22;
                s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                l23 = s_i32_0;
                s_i32_0 = l22;
                s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
                l24 = s_i32_0;
                s_i32_0 = 0i32;
                l25 = s_i32_0;
                'blk_5: {
                    'blk_6: loop {
                        vm.deduct_1(host)?;
                        s_i32_0 = l25;
                        s_i32_1 = l24;
                        s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                        if s_i32_0 != 0 {
                            break 'blk_5;
                        }
                        s_i32_0 = l23;
                        s_i32_1 = l25;
                        s_i32_2 = 4i32;
                        s_i32_1 = ops::i32_mul(s_i32_1, s_i32_2);
                        s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                        s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                        l26 = s_i32_0;
                        'blk_7: {
                            s_i32_0 = l5;
                            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                            s_i32_1 = l26;
                            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 0u32)?;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, host);
                            vm.exit_call();
                            let __t5 = __call_res?;
                            s_i32_0 = __t5;
                            l27 = s_i32_0;
                            s_i32_0 = l5;
                            l28 = s_i32_0;
                            s_i32_0 = l27;
                            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                            l29 = s_i32_0;
                            s_i32_0 = 8i32;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
                            vm.exit_call();
                            let __t6 = __call_res?;
                            s_i32_0 = __t6;
                            l30 = s_i32_0;
                            s_i32_0 = l30;
                            s_i32_1 = l28;
                            s_i32_2 = 8i32;
                            vm.memory_copy(0, 0, s_i32_0, s_i32_1, s_i32_2, host)?;
                            s_i32_0 = l30;
                            s_i32_1 = l29;
                            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                            s_i32_0 = l30;
                            l5 = s_i32_0;
                            s_i32_0 = l27;
                            s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 4u32)?;
                            s_i32_0 = ops::i32_eqz(s_i32_0);
                            'blk_8: {
                                if s_i32_0 != 0 {
                                    s_i32_0 = l5;
                                    l31 = s_i32_0;
                                    s_i32_0 = 0i32;
                                    l32 = s_i32_0;
                                    s_i32_0 = 2i32;
                                    l33 = s_i32_0;
                                    s_i32_0 = 16i32;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
                                    vm.exit_call();
                                    let __t7 = __call_res?;
                                    s_i32_0 = __t7;
                                    l34 = s_i32_0;
                                    s_i32_0 = l34;
                                    s_i32_1 = l34;
                                    s_i32_2 = 16i32;
                                    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
                                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                                    s_i32_0 = l34;
                                    s_i32_1 = 0i32;
                                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                                    s_i32_0 = l34;
                                    s_i32_1 = 0i32;
                                    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                                    s_i32_0 = l34;
                                    s_i32_1 = 1i32;
                                    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                                    s_i32_0 = l34;
                                    l35 = s_i32_0;
                                    s_i32_0 = 12i32;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
                                    vm.exit_call();
                                    let __t8 = __call_res?;
                                    s_i32_0 = __t8;
                                    l36 = s_i32_0;
                                    s_i32_0 = l36;
                                    s_i32_1 = l31;
                                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                                    s_i32_0 = l36;
                                    s_i32_1 = l32;
                                    ops::i32_store8(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                                    s_i32_0 = l36;
                                    s_i32_1 = l33;
                                    ops::i32_store8(vm, 0, s_i32_0, 5u32, s_i32_1)?;
                                    s_i32_0 = l36;
                                    s_i32_1 = l35;
                                    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                                    s_i32_0 = l36;
                                    l37 = s_i32_0;
                                    s_i32_0 = l37;
                                    return Ok(s_i32_0);
                                } else {
                                }
                            }
                            s_i32_0 = l27;
                            s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 5u32)?;
                            s_i32_0 = ops::i32_eqz(s_i32_0);
                            'blk_9: {
                                if s_i32_0 != 0 {
                                    s_i32_0 = l3;
                                    'blk_10: {
                                        if s_i32_0 != 0 {
                                            s_i32_0 = l5;
                                            s_i32_1 = l26;
                                            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
                                            s_i64_2 = l4;
                                            s_i64_3 = 1i64;
                                            s_i64_2 = ops::i64_add(s_i64_2, s_i64_3);
                                            vm.enter_call()?;
                                            let __call_res = dispatch.call_p_i32_i32_i64_ret_i32(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i64_2, host);
                                            vm.exit_call();
                                            let __t9 = __call_res?;
                                            s_i32_0 = __t9;
                                            l38 = s_i32_0;
                                            s_i32_0 = l38;
                                            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                                            l5 = s_i32_0;
                                            s_i32_0 = l38;
                                            s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 4u32)?;
                                            'blk_11: {
                                                if s_i32_0 != 0 {
                                                    s_i32_0 = l38;
                                                    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 5u32)?;
                                                    s_i32_1 = 0i32;
                                                    vm.enter_call()?;
                                                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 6usize, s_i32_0, s_i32_1, host);
                                                    vm.exit_call();
                                                    let __t10 = __call_res?;
                                                    s_i32_0 = __t10;
                                                } else {
                                                    s_i32_0 = 0i32;
                                                }
                                            }
                                            'blk_12: {
                                                if s_i32_0 != 0 {
                                                    s_i32_0 = l38;
                                                    l39 = s_i32_0;
                                                    s_i32_0 = l39;
                                                    return Ok(s_i32_0);
                                                } else {
                                                }
                                            }
                                        } else {
                                            s_i32_0 = l26;
                                            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
                                            vm.enter_call()?;
                                            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 7usize, s_i32_0, host);
                                            vm.exit_call();
                                            let __t11 = __call_res?;
                                            s_i32_0 = __t11;
                                            l40 = s_i32_0;
                                            s_i32_0 = l40;
                                            vm.enter_call()?;
                                            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 8usize, s_i32_0, host);
                                            vm.exit_call();
                                            let __t12 = __call_res?;
                                            s_i32_0 = __t12;
                                            'blk_13: {
                                                if s_i32_0 != 0 {
                                                    s_i32_0 = l5;
                                                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                                                    s_i32_1 = l40;
                                                    vm.enter_call()?;
                                                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 9usize, s_i32_0, s_i32_1, host);
                                                    vm.exit_call();
                                                    let __t13 = __call_res?;
                                                    s_i32_0 = __t13;
                                                    l41 = s_i32_0;
                                                    s_i32_0 = l5;
                                                    l42 = s_i32_0;
                                                    s_i32_0 = l41;
                                                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                                                    l43 = s_i32_0;
                                                    s_i32_0 = 8i32;
                                                    vm.enter_call()?;
                                                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
                                                    vm.exit_call();
                                                    let __t14 = __call_res?;
                                                    s_i32_0 = __t14;
                                                    l44 = s_i32_0;
                                                    s_i32_0 = l44;
                                                    s_i32_1 = l42;
                                                    s_i32_2 = 8i32;
                                                    vm.memory_copy(0, 0, s_i32_0, s_i32_1, s_i32_2, host)?;
                                                    s_i32_0 = l44;
                                                    s_i32_1 = l43;
                                                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                                                    s_i32_0 = l44;
                                                    l5 = s_i32_0;
                                                    s_i32_0 = l41;
                                                    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 4u32)?;
                                                    s_i32_1 = 1i32;
                                                    vm.enter_call()?;
                                                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 6usize, s_i32_0, s_i32_1, host);
                                                    vm.exit_call();
                                                    let __t15 = __call_res?;
                                                    s_i32_0 = __t15;
                                                    'blk_14: {
                                                        if s_i32_0 != 0 {
                                                            s_i32_0 = l41;
                                                            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
                                                            vm.enter_call()?;
                                                            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 10usize, s_i32_0, host);
                                                            vm.exit_call();
                                                            let __t16 = __call_res?;
                                                            s_i32_0 = __t16;
                                                            s_i32_0 = ops::i32_eqz(s_i32_0);
                                                        } else {
                                                            s_i32_0 = 0i32;
                                                        }
                                                    }
                                                    'blk_15: {
                                                        if s_i32_0 != 0 {
                                                            s_i32_0 = l41;
                                                            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
                                                            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
                                                            s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
                                                            vm.enter_call()?;
                                                            let __call_res = dispatch.call_p_i64_ret_i32(vm, c_map, 11usize, s_i64_0, host);
                                                            vm.exit_call();
                                                            let __t17 = __call_res?;
                                                            s_i32_0 = __t17;
                                                            l45 = s_i32_0;
                                                            s_i32_0 = 1i32;
                                                            l46 = s_i32_0;
                                                            s_i32_0 = l41;
                                                            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
                                                            l47 = s_i32_0;
                                                            s_i32_0 = l47;
                                                            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                                                            l48 = s_i32_0;
                                                            s_i32_0 = l47;
                                                            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
                                                            l49 = s_i32_0;
                                                            s_i32_0 = 0i32;
                                                            l50 = s_i32_0;
                                                            'blk_16: {
                                                                'blk_17: loop {
                                                                    vm.deduct_1(host)?;
                                                                    s_i32_0 = l50;
                                                                    s_i32_1 = l49;
                                                                    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                                                                    if s_i32_0 != 0 {
                                                                        break 'blk_16;
                                                                    }
                                                                    s_i32_0 = l48;
                                                                    s_i32_1 = l50;
                                                                    s_i32_2 = 4i32;
                                                                    s_i32_1 = ops::i32_mul(s_i32_1, s_i32_2);
                                                                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                                                                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                                                                    l51 = s_i32_0;
                                                                    'blk_18: {
                                                                        s_i32_0 = l5;
                                                                        s_i32_1 = l51;
                                                                        s_i64_2 = l4;
                                                                        s_i64_3 = 1i64;
                                                                        s_i64_2 = ops::i64_add(s_i64_2, s_i64_3);
                                                                        vm.enter_call()?;
                                                                        let __call_res = dispatch.call_p_i32_i32_i64_ret_i32(vm, c_map, 5usize, s_i32_0, s_i32_1, s_i64_2, host);
                                                                        vm.exit_call();
                                                                        let __t18 = __call_res?;
                                                                        s_i32_0 = __t18;
                                                                        l52 = s_i32_0;
                                                                        s_i32_0 = l52;
                                                                        s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                                                                        l5 = s_i32_0;
                                                                        s_i32_0 = l52;
                                                                        s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 4u32)?;
                                                                        'blk_19: {
                                                                            if s_i32_0 != 0 {
                                                                                s_i32_0 = l52;
                                                                                s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 5u32)?;
                                                                                s_i32_1 = 0i32;
                                                                                vm.enter_call()?;
                                                                                let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 6usize, s_i32_0, s_i32_1, host);
                                                                                vm.exit_call();
                                                                                let __t19 = __call_res?;
                                                                                s_i32_0 = __t19;
                                                                            } else {
                                                                                s_i32_0 = 0i32;
                                                                            }
                                                                        }
                                                                        'blk_20: {
                                                                            if s_i32_0 != 0 {
                                                                                s_i32_0 = l45;
                                                                                s_i32_1 = l52;
                                                                                s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 8u32)?;
                                                                                vm.enter_call()?;
                                                                                let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 12usize, s_i32_0, s_i32_1, host);
                                                                                vm.exit_call();
                                                                                __call_res?;
                                                                            } else {
                                                                                s_i32_0 = 0i32;
                                                                                l46 = s_i32_0;
                                                                                break 'blk_16;
                                                                            }
                                                                        }
                                                                    }
                                                                    s_i32_0 = l50;
                                                                    s_i32_1 = 1i32;
                                                                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                                                                    l50 = s_i32_0;
                                                                    continue 'blk_17;
                                                                    break 'blk_17;
                                                                }
                                                            }
                                                            s_i32_0 = l45;
                                                            vm.enter_call()?;
                                                            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 13usize, s_i32_0, host);
                                                            vm.exit_call();
                                                            let __t20 = __call_res?;
                                                            s_i32_0 = __t20;
                                                            l53 = s_i32_0;
                                                            s_i32_0 = l46;
                                                            'blk_21: {
                                                                if s_i32_0 != 0 {
                                                                    s_i32_0 = l53;
                                                                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
                                                                    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
                                                                    s_i64_1 = 0i64;
                                                                    s_i32_0 = ops::i64_gt_u(s_i64_0, s_i64_1);
                                                                } else {
                                                                    s_i32_0 = 0i32;
                                                                }
                                                            }
                                                            'blk_22: {
                                                                if s_i32_0 != 0 {
                                                                    s_i32_0 = l5;
                                                                    l54 = s_i32_0;
                                                                    s_i32_0 = 1i32;
                                                                    l55 = s_i32_0;
                                                                    s_i32_0 = 0i32;
                                                                    l56 = s_i32_0;
                                                                    s_i32_0 = l53;
                                                                    l57 = s_i32_0;
                                                                    s_i32_0 = 12i32;
                                                                    vm.enter_call()?;
                                                                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
                                                                    vm.exit_call();
                                                                    let __t21 = __call_res?;
                                                                    s_i32_0 = __t21;
                                                                    l58 = s_i32_0;
                                                                    s_i32_0 = l58;
                                                                    s_i32_1 = l54;
                                                                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                                                                    s_i32_0 = l58;
                                                                    s_i32_1 = l55;
                                                                    ops::i32_store8(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                                                                    s_i32_0 = l58;
                                                                    s_i32_1 = l56;
                                                                    ops::i32_store8(vm, 0, s_i32_0, 5u32, s_i32_1)?;
                                                                    s_i32_0 = l58;
                                                                    s_i32_1 = l57;
                                                                    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                                                                    s_i32_0 = l58;
                                                                    l59 = s_i32_0;
                                                                    s_i32_0 = l59;
                                                                    return Ok(s_i32_0);
                                                                } else {
                                                                }
                                                            }
                                                        } else {
                                                        }
                                                    }
                                                } else {
                                                }
                                            }
                                        }
                                    }
                                } else {
                                }
                            }
                        }
                        s_i32_0 = l25;
                        s_i32_1 = 1i32;
                        s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                        l25 = s_i32_0;
                        continue 'blk_6;
                        break 'blk_6;
                    }
                }
            }
            s_i32_0 = l9;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l9 = s_i32_0;
            continue 'blk_2;
            break 'blk_2;
        }
    }
    s_i32_0 = l5;
    l60 = s_i32_0;
    s_i32_0 = 0i32;
    l61 = s_i32_0;
    s_i32_0 = 2i32;
    l62 = s_i32_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
    vm.exit_call();
    let __t22 = __call_res?;
    s_i32_0 = __t22;
    l63 = s_i32_0;
    s_i32_0 = l63;
    s_i32_1 = l63;
    s_i32_2 = 16i32;
    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l63;
    s_i32_1 = 0i32;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l63;
    s_i32_1 = 0i32;
    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
    s_i32_0 = l63;
    s_i32_1 = 1i32;
    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
    s_i32_0 = l63;
    l64 = s_i32_0;
    s_i32_0 = 12i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
    vm.exit_call();
    let __t23 = __call_res?;
    s_i32_0 = __t23;
    l65 = s_i32_0;
    s_i32_0 = l65;
    s_i32_1 = l60;
    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l65;
    s_i32_1 = l61;
    ops::i32_store8(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l65;
    s_i32_1 = l62;
    ops::i32_store8(vm, 0, s_i32_0, 5u32, s_i32_1)?;
    s_i32_0 = l65;
    s_i32_1 = l64;
    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
    s_i32_0 = l65;
    l66 = s_i32_0;
    s_i32_0 = l66;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn f_f2aa3d0555d6(vm: &mut SubstituteVm, dispatch: &'static crate::ModuleDispatch, g_map: &'static [usize], c_map: &'static [usize], p0: i32, p1: i32, p2: i64, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i64 = p2;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i32 = 0i32;
    let mut l7: i64 = 0i64;
    let mut l8: i64 = 0i64;
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
    let mut l34: i64 = 0i64;
    let mut l35: i64 = 0i64;
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
    let mut l49: i64 = 0i64;
    let mut l50: i64 = 0i64;
    let mut l51: i32 = 0i32;
    let mut l52: i32 = 0i32;
    let mut l53: i32 = 0i32;
    let mut l54: i32 = 0i32;
    let mut l55: i32 = 0i32;
    let mut l56: i32 = 0i32;
    let mut l57: i32 = 0i32;
    let mut l58: i32 = 0i32;
    let mut l59: i32 = 0i32;
    let mut l60: i32 = 0i32;
    let mut l61: i32 = 0i32;
    let mut l62: i32 = 0i32;
    let mut l63: i32 = 0i32;
    let mut l64: i32 = 0i32;
    let mut l65: i32 = 0i32;
    let mut l66: i64 = 0i64;
    let mut l67: i64 = 0i64;
    let mut l68: i32 = 0i32;
    let mut l69: i32 = 0i32;
    let mut l70: i32 = 0i32;
    let mut l71: i32 = 0i32;
    let mut l72: i32 = 0i32;
    let mut l73: i32 = 0i32;
    let mut l74: i32 = 0i32;
    let mut l75: i32 = 0i32;
    let mut l76: i64 = 0i64;
    let mut l77: i64 = 0i64;
    let mut l78: i32 = 0i32;
    let mut l79: i32 = 0i32;
    let mut l80: i32 = 0i32;
    let mut l81: i32 = 0i32;
    let mut l82: i32 = 0i32;
    let mut l83: i32 = 0i32;
    let mut l84: i32 = 0i32;
    let mut l85: i32 = 0i32;
    let mut l86: i64 = 0i64;
    let mut l87: i64 = 0i64;
    let mut l88: i32 = 0i32;
    let mut l89: i32 = 0i32;
    let mut l90: i32 = 0i32;
    let mut l91: i32 = 0i32;
    let mut l92: i32 = 0i32;
    let mut l93: i32 = 0i32;
    let mut l94: i32 = 0i32;
    let mut l95: i32 = 0i32;
    let mut l96: i64 = 0i64;
    let mut l97: i64 = 0i64;
    let mut l98: i32 = 0i32;
    let mut l99: i32 = 0i32;
    let mut l100: i32 = 0i32;
    let mut l101: i32 = 0i32;
    let mut l102: i32 = 0i32;
    let mut l103: i32 = 0i32;
    let mut l104: i32 = 0i32;
    let mut l105: i32 = 0i32;
    let mut l106: i64 = 0i64;
    let mut l107: i64 = 0i64;
    let mut l108: i32 = 0i32;
    let mut l109: i32 = 0i32;
    let mut l110: i32 = 0i32;
    let mut l111: i32 = 0i32;
    let mut l112: i32 = 0i32;
    let mut l113: i32 = 0i32;
    let mut l114: i32 = 0i32;
    let mut l115: i32 = 0i32;
    let mut l116: i32 = 0i32;
    let mut l117: i32 = 0i32;
    let mut l118: i32 = 0i32;
    let mut l119: i32 = 0i32;
    let mut l120: i32 = 0i32;
    let mut l121: i32 = 0i32;
    let mut l122: i32 = 0i32;
    let mut l123: i32 = 0i32;
    let mut l124: i32 = 0i32;
    let mut l125: i32 = 0i32;
    let mut l126: i32 = 0i32;
    let mut l127: i32 = 0i32;
    let mut l128: i32 = 0i32;
    let mut l129: i64 = 0i64;
    let mut l130: i64 = 0i64;
    let mut l131: i32 = 0i32;
    let mut l132: i32 = 0i32;
    let mut l133: i32 = 0i32;
    let mut l134: i32 = 0i32;
    let mut l135: i32 = 0i32;
    let mut l136: i32 = 0i32;
    let mut l137: i32 = 0i32;
    let mut l138: i32 = 0i32;
    let mut l139: i32 = 0i32;
    let mut l140: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i32_2: i32 = 0i32;
    let mut s_i64_2: i64 = 0i64;
    let mut s_i32_3: i32 = 0i32;
    let mut s_i64_3: i64 = 0i64;
    let mut s_i32_4: i32 = 0i32;
    let mut s_i64_4: i64 = 0i64;
    let mut s_i32_5: i32 = 0i32;
    vm.deduct_1(host)?;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_unit_ret_void(vm, c_map, 0usize, host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l1;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_0, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l3 = s_i32_0;
    s_i32_0 = l3;
    s_i32_1 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 2usize, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t1 = __call_res?;
    s_i32_0 = __t1;
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = l3;
            s_i32_1 = vm.get_i32(*g_map.get(1usize).ok_or(WasmTrap::Unreachable)?)?;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 3usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_0 = __t2;
            'blk_2: {
                if s_i32_0 != 0 {
                    s_i32_0 = l3;
                    s_i32_1 = vm.get_i32(*g_map.get(2usize).ok_or(WasmTrap::Unreachable)?)?;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 2usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t3 = __call_res?;
                    s_i32_0 = __t3;
                } else {
                    s_i32_0 = 0i32;
                }
            }
            'blk_3: {
                if s_i32_0 != 0 {
                    s_i32_0 = l0;
                    l4 = s_i32_0;
                    s_i32_0 = 1i32;
                    l5 = s_i32_0;
                    s_i32_0 = 2i32;
                    l6 = s_i32_0;
                    s_i64_0 = 65533i64;
                    l7 = s_i64_0;
                    s_i64_0 = 0i64;
                    l8 = s_i64_0;
                    s_i32_0 = 16i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 4usize, s_i32_0, host);
                    vm.exit_call();
                    let __t4 = __call_res?;
                    s_i32_0 = __t4;
                    l9 = s_i32_0;
                    s_i32_0 = l9;
                    s_i64_1 = l7;
                    ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
                    s_i32_0 = l9;
                    s_i64_1 = l8;
                    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
                    s_i32_0 = l9;
                    l10 = s_i32_0;
                    s_i32_0 = 20i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 4usize, s_i32_0, host);
                    vm.exit_call();
                    let __t5 = __call_res?;
                    s_i32_0 = __t5;
                    l11 = s_i32_0;
                    s_i32_0 = l11;
                    s_i32_1 = l11;
                    s_i32_2 = 16i32;
                    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l11;
                    s_i32_1 = 1i32;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l11;
                    s_i32_1 = 1i32;
                    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                    s_i32_0 = l11;
                    s_i32_1 = 1i32;
                    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                    s_i32_0 = l11;
                    s_i32_1 = l10;
                    ops::i32_store(vm, 0, s_i32_0, 16u32, s_i32_1)?;
                    s_i32_0 = l11;
                    l12 = s_i32_0;
                    s_i32_0 = 12i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 4usize, s_i32_0, host);
                    vm.exit_call();
                    let __t6 = __call_res?;
                    s_i32_0 = __t6;
                    l13 = s_i32_0;
                    s_i32_0 = l13;
                    s_i32_1 = l4;
                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l13;
                    s_i32_1 = l5;
                    ops::i32_store8(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l13;
                    s_i32_1 = l6;
                    ops::i32_store8(vm, 0, s_i32_0, 5u32, s_i32_1)?;
                    s_i32_0 = l13;
                    s_i32_1 = l12;
                    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                    s_i32_0 = l13;
                    l14 = s_i32_0;
                    s_i32_0 = l14;
                    return Ok(s_i32_0);
                } else {
                    s_i32_0 = l0;
                    l15 = s_i32_0;
                    s_i32_0 = 1i32;
                    l16 = s_i32_0;
                    s_i32_0 = 0i32;
                    l17 = s_i32_0;
                    s_i32_0 = l1;
                    l18 = s_i32_0;
                    s_i32_0 = 20i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 4usize, s_i32_0, host);
                    vm.exit_call();
                    let __t7 = __call_res?;
                    s_i32_0 = __t7;
                    l19 = s_i32_0;
                    s_i32_0 = l19;
                    s_i32_1 = l19;
                    s_i32_2 = 16i32;
                    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l19;
                    s_i32_1 = 1i32;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l19;
                    s_i32_1 = 1i32;
                    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                    s_i32_0 = l19;
                    s_i32_1 = 1i32;
                    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                    s_i32_0 = l19;
                    s_i32_1 = l18;
                    ops::i32_store(vm, 0, s_i32_0, 16u32, s_i32_1)?;
                    s_i32_0 = l19;
                    l20 = s_i32_0;
                    s_i32_0 = 12i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 4usize, s_i32_0, host);
                    vm.exit_call();
                    let __t8 = __call_res?;
                    s_i32_0 = __t8;
                    l21 = s_i32_0;
                    s_i32_0 = l21;
                    s_i32_1 = l15;
                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l21;
                    s_i32_1 = l16;
                    ops::i32_store8(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l21;
                    s_i32_1 = l17;
                    ops::i32_store8(vm, 0, s_i32_0, 5u32, s_i32_1)?;
                    s_i32_0 = l21;
                    s_i32_1 = l20;
                    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                    s_i32_0 = l21;
                    l22 = s_i32_0;
                    s_i32_0 = l22;
                    return Ok(s_i32_0);
                }
            }
            return Err(WasmTrap::Unreachable);
        } else {
        }
    }
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    s_i32_1 = l3;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 5usize, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t9 = __call_res?;
    s_i32_0 = __t9;
    l23 = s_i32_0;
    s_i32_0 = l23;
    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
    'blk_4: {
        if s_i32_0 != 0 {
            s_i32_0 = l23;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
            l24 = s_i32_0;
            s_i32_0 = l0;
            l25 = s_i32_0;
            s_i32_0 = 1i32;
            l26 = s_i32_0;
            s_i32_0 = l24;
            s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 4u32)?;
            l27 = s_i32_0;
            s_i32_0 = l24;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
            l28 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 4usize, s_i32_0, host);
            vm.exit_call();
            let __t10 = __call_res?;
            s_i32_0 = __t10;
            l29 = s_i32_0;
            s_i32_0 = l29;
            s_i32_1 = l25;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l29;
            s_i32_1 = l26;
            ops::i32_store8(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l29;
            s_i32_1 = l27;
            ops::i32_store8(vm, 0, s_i32_0, 5u32, s_i32_1)?;
            s_i32_0 = l29;
            s_i32_1 = l28;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l29;
            l30 = s_i32_0;
            s_i32_0 = l30;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i64_0 = l2;
    s_i64_1 = 8i64;
    s_i32_0 = ops::i64_ge_u(s_i64_0, s_i64_1);
    'blk_5: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            l31 = s_i32_0;
            s_i32_0 = 0i32;
            l32 = s_i32_0;
            s_i32_0 = 2i32;
            l33 = s_i32_0;
            s_i64_0 = 65533i64;
            l34 = s_i64_0;
            s_i64_0 = 0i64;
            l35 = s_i64_0;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 4usize, s_i32_0, host);
            vm.exit_call();
            let __t11 = __call_res?;
            s_i32_0 = __t11;
            l36 = s_i32_0;
            s_i32_0 = l36;
            s_i64_1 = l34;
            ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
            s_i32_0 = l36;
            s_i64_1 = l35;
            ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
            s_i32_0 = l36;
            l37 = s_i32_0;
            s_i32_0 = 20i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 4usize, s_i32_0, host);
            vm.exit_call();
            let __t12 = __call_res?;
            s_i32_0 = __t12;
            l38 = s_i32_0;
            s_i32_0 = l38;
            s_i32_1 = l38;
            s_i32_2 = 16i32;
            s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l38;
            s_i32_1 = 1i32;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l38;
            s_i32_1 = 1i32;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l38;
            s_i32_1 = 1i32;
            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
            s_i32_0 = l38;
            s_i32_1 = l37;
            ops::i32_store(vm, 0, s_i32_0, 16u32, s_i32_1)?;
            s_i32_0 = l38;
            l39 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 4usize, s_i32_0, host);
            vm.exit_call();
            let __t13 = __call_res?;
            s_i32_0 = __t13;
            l40 = s_i32_0;
            s_i32_0 = l40;
            s_i32_1 = l31;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l40;
            s_i32_1 = l32;
            ops::i32_store8(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l40;
            s_i32_1 = l33;
            ops::i32_store8(vm, 0, s_i32_0, 5u32, s_i32_1)?;
            s_i32_0 = l40;
            s_i32_1 = l39;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l40;
            l41 = s_i32_0;
            s_i32_0 = l41;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i32_0 = l0;
    l42 = s_i32_0;
    s_i32_0 = l42;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    s_i32_1 = l1;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 6usize, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t14 = __call_res?;
    s_i32_0 = __t14;
    l43 = s_i32_0;
    s_i32_0 = l42;
    l44 = s_i32_0;
    s_i32_0 = l43;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    l45 = s_i32_0;
    s_i32_0 = 8i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 4usize, s_i32_0, host);
    vm.exit_call();
    let __t15 = __call_res?;
    s_i32_0 = __t15;
    l46 = s_i32_0;
    s_i32_0 = l46;
    s_i32_1 = l44;
    s_i32_2 = 8i32;
    vm.memory_copy(0, 0, s_i32_0, s_i32_1, s_i32_2, host)?;
    s_i32_0 = l46;
    s_i32_1 = l45;
    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l46;
    l42 = s_i32_0;
    s_i32_0 = l43;
    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 4u32)?;
    'blk_6: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            l47 = s_i32_0;
            s_i64_0 = 0i64;
            l49 = s_i64_0;
            s_i64_0 = 0i64;
            l50 = s_i64_0;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 4usize, s_i32_0, host);
            vm.exit_call();
            let __t16 = __call_res?;
            s_i32_0 = __t16;
            l51 = s_i32_0;
            s_i32_0 = l51;
            s_i64_1 = l49;
            ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
            s_i32_0 = l51;
            s_i64_1 = l50;
            ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
            s_i32_0 = l51;
            l48 = s_i32_0;
            s_i32_0 = l43;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
            l52 = s_i32_0;
            s_i32_0 = l52;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
            l53 = s_i32_0;
            s_i32_0 = l52;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
            l54 = s_i32_0;
            s_i32_0 = 0i32;
            l55 = s_i32_0;
            'blk_7: {
                'blk_8: loop {
                    vm.deduct_1(host)?;
                    s_i32_0 = l55;
                    s_i32_1 = l54;
                    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                    if s_i32_0 != 0 {
                        break 'blk_7;
                    }
                    s_i32_0 = l53;
                    s_i32_1 = l55;
                    s_i32_2 = 4i32;
                    s_i32_1 = ops::i32_mul(s_i32_1, s_i32_2);
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                    l56 = s_i32_0;
                    'blk_9: {
                        s_i32_0 = l56;
                        vm.enter_call()?;
                        let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 7usize, s_i32_0, host);
                        vm.exit_call();
                        let __t17 = __call_res?;
                        s_i32_0 = __t17;
                        'blk_10: {
                            if s_i32_0 != 0 {
                                s_i32_0 = l47;
                                s_i32_0 = ops::i32_eqz(s_i32_0);
                                'blk_11: {
                                    if s_i32_0 != 0 {
                                        s_i32_0 = 1i32;
                                    } else {
                                        s_i32_0 = l56;
                                        s_i32_1 = l48;
                                        vm.enter_call()?;
                                        let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 8usize, s_i32_0, s_i32_1, host);
                                        vm.exit_call();
                                        let __t18 = __call_res?;
                                        s_i32_0 = __t18;
                                    }
                                }
                                'blk_12: {
                                    if s_i32_0 != 0 {
                                        s_i32_0 = 1i32;
                                        l47 = s_i32_0;
                                        s_i32_0 = l56;
                                        l48 = s_i32_0;
                                    } else {
                                    }
                                }
                            } else {
                            }
                        }
                    }
                    s_i32_0 = l55;
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                    l55 = s_i32_0;
                    continue 'blk_8;
                    break 'blk_8;
                }
            }
            s_i32_0 = l47;
            'blk_13: {
                if s_i32_0 != 0 {
                    s_i32_0 = l48;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 9usize, s_i32_0, host);
                    vm.exit_call();
                    let __t19 = __call_res?;
                    s_i32_0 = __t19;
                    l57 = s_i32_0;
                    s_i32_0 = 20i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 4usize, s_i32_0, host);
                    vm.exit_call();
                    let __t20 = __call_res?;
                    s_i32_0 = __t20;
                    l58 = s_i32_0;
                    s_i32_0 = l58;
                    s_i32_1 = l58;
                    s_i32_2 = 16i32;
                    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l58;
                    s_i32_1 = 1i32;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l58;
                    s_i32_1 = 1i32;
                    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                    s_i32_0 = l58;
                    s_i32_1 = 1i32;
                    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                    s_i32_0 = l58;
                    s_i32_1 = l57;
                    ops::i32_store(vm, 0, s_i32_0, 16u32, s_i32_1)?;
                    s_i32_0 = l58;
                    l59 = s_i32_0;
                    s_i32_0 = l42;
                    s_i32_1 = l3;
                    s_i32_2 = 0i32;
                    s_i32_3 = l59;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_i32(vm, c_map, 10usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
                    vm.exit_call();
                    let __t21 = __call_res?;
                    s_i32_0 = __t21;
                    l42 = s_i32_0;
                    s_i32_0 = l42;
                    l60 = s_i32_0;
                    s_i32_0 = 1i32;
                    l61 = s_i32_0;
                    s_i32_0 = 0i32;
                    l62 = s_i32_0;
                    s_i32_0 = l59;
                    l63 = s_i32_0;
                    s_i32_0 = 12i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 4usize, s_i32_0, host);
                    vm.exit_call();
                    let __t22 = __call_res?;
                    s_i32_0 = __t22;
                    l64 = s_i32_0;
                    s_i32_0 = l64;
                    s_i32_1 = l60;
                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l64;
                    s_i32_1 = l61;
                    ops::i32_store8(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l64;
                    s_i32_1 = l62;
                    ops::i32_store8(vm, 0, s_i32_0, 5u32, s_i32_1)?;
                    s_i32_0 = l64;
                    s_i32_1 = l63;
                    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                    s_i32_0 = l64;
                    l65 = s_i32_0;
                    s_i32_0 = l65;
                    return Ok(s_i32_0);
                } else {
                }
            }
            s_i32_0 = l42;
            s_i32_1 = l43;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 8u32)?;
            s_i64_2 = 1114723i64;
            l66 = s_i64_2;
            s_i64_2 = 0i64;
            l67 = s_i64_2;
            s_i32_2 = 16i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 4usize, s_i32_2, host);
            vm.exit_call();
            let __t23 = __call_res?;
            s_i32_2 = __t23;
            l68 = s_i32_2;
            s_i32_2 = l68;
            s_i64_3 = l66;
            ops::i64_store(vm, 0, s_i32_2, 0u32, s_i64_3)?;
            s_i32_2 = l68;
            s_i64_3 = l67;
            ops::i64_store(vm, 0, s_i32_2, 8u32, s_i64_3)?;
            s_i32_2 = l68;
            s_i32_3 = 0i32;
            s_i64_4 = l2;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_i32_i64_ret_i32(vm, c_map, 11usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, s_i64_4, host);
            vm.exit_call();
            let __t24 = __call_res?;
            s_i32_0 = __t24;
            l69 = s_i32_0;
            s_i32_0 = l69;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
            l42 = s_i32_0;
            s_i32_0 = l69;
            s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 4u32)?;
            'blk_14: {
                if s_i32_0 != 0 {
                    s_i32_0 = l42;
                    s_i32_1 = l3;
                    s_i32_2 = 0i32;
                    s_i32_3 = l69;
                    s_i32_3 = ops::i32_load(vm, 0, s_i32_3, 8u32)?;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_i32(vm, c_map, 10usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
                    vm.exit_call();
                    let __t25 = __call_res?;
                    s_i32_0 = __t25;
                    l42 = s_i32_0;
                    s_i32_0 = l42;
                    l70 = s_i32_0;
                    s_i32_0 = 1i32;
                    l71 = s_i32_0;
                    s_i32_0 = 0i32;
                    l72 = s_i32_0;
                    s_i32_0 = l69;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
                    l73 = s_i32_0;
                    s_i32_0 = 12i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 4usize, s_i32_0, host);
                    vm.exit_call();
                    let __t26 = __call_res?;
                    s_i32_0 = __t26;
                    l74 = s_i32_0;
                    s_i32_0 = l74;
                    s_i32_1 = l70;
                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l74;
                    s_i32_1 = l71;
                    ops::i32_store8(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l74;
                    s_i32_1 = l72;
                    ops::i32_store8(vm, 0, s_i32_0, 5u32, s_i32_1)?;
                    s_i32_0 = l74;
                    s_i32_1 = l73;
                    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                    s_i32_0 = l74;
                    l75 = s_i32_0;
                    s_i32_0 = l75;
                    return Ok(s_i32_0);
                } else {
                }
            }
            s_i32_0 = l42;
            s_i32_1 = l43;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 8u32)?;
            s_i64_2 = 1114681i64;
            l76 = s_i64_2;
            s_i64_2 = 0i64;
            l77 = s_i64_2;
            s_i32_2 = 16i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 4usize, s_i32_2, host);
            vm.exit_call();
            let __t27 = __call_res?;
            s_i32_2 = __t27;
            l78 = s_i32_2;
            s_i32_2 = l78;
            s_i64_3 = l76;
            ops::i64_store(vm, 0, s_i32_2, 0u32, s_i64_3)?;
            s_i32_2 = l78;
            s_i64_3 = l77;
            ops::i64_store(vm, 0, s_i32_2, 8u32, s_i64_3)?;
            s_i32_2 = l78;
            s_i32_3 = 1i32;
            s_i64_4 = l2;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_i32_i64_ret_i32(vm, c_map, 11usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, s_i64_4, host);
            vm.exit_call();
            let __t28 = __call_res?;
            s_i32_0 = __t28;
            l79 = s_i32_0;
            s_i32_0 = l79;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
            l42 = s_i32_0;
            s_i32_0 = l79;
            s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 4u32)?;
            'blk_15: {
                if s_i32_0 != 0 {
                    s_i32_0 = l42;
                    s_i32_1 = l3;
                    s_i32_2 = 0i32;
                    s_i32_3 = l79;
                    s_i32_3 = ops::i32_load(vm, 0, s_i32_3, 8u32)?;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_i32(vm, c_map, 10usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
                    vm.exit_call();
                    let __t29 = __call_res?;
                    s_i32_0 = __t29;
                    l42 = s_i32_0;
                    s_i32_0 = l42;
                    l80 = s_i32_0;
                    s_i32_0 = 1i32;
                    l81 = s_i32_0;
                    s_i32_0 = 0i32;
                    l82 = s_i32_0;
                    s_i32_0 = l79;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
                    l83 = s_i32_0;
                    s_i32_0 = 12i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 4usize, s_i32_0, host);
                    vm.exit_call();
                    let __t30 = __call_res?;
                    s_i32_0 = __t30;
                    l84 = s_i32_0;
                    s_i32_0 = l84;
                    s_i32_1 = l80;
                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l84;
                    s_i32_1 = l81;
                    ops::i32_store8(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l84;
                    s_i32_1 = l82;
                    ops::i32_store8(vm, 0, s_i32_0, 5u32, s_i32_1)?;
                    s_i32_0 = l84;
                    s_i32_1 = l83;
                    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                    s_i32_0 = l84;
                    l85 = s_i32_0;
                    s_i32_0 = l85;
                    return Ok(s_i32_0);
                } else {
                }
            }
            s_i32_0 = l42;
            s_i32_1 = l43;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 8u32)?;
            s_i64_2 = 1114724i64;
            l86 = s_i64_2;
            s_i64_2 = 0i64;
            l87 = s_i64_2;
            s_i32_2 = 16i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 4usize, s_i32_2, host);
            vm.exit_call();
            let __t31 = __call_res?;
            s_i32_2 = __t31;
            l88 = s_i32_2;
            s_i32_2 = l88;
            s_i64_3 = l86;
            ops::i64_store(vm, 0, s_i32_2, 0u32, s_i64_3)?;
            s_i32_2 = l88;
            s_i64_3 = l87;
            ops::i64_store(vm, 0, s_i32_2, 8u32, s_i64_3)?;
            s_i32_2 = l88;
            s_i32_3 = 0i32;
            s_i64_4 = l2;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_i32_i64_ret_i32(vm, c_map, 11usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, s_i64_4, host);
            vm.exit_call();
            let __t32 = __call_res?;
            s_i32_0 = __t32;
            l89 = s_i32_0;
            s_i32_0 = l89;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
            l42 = s_i32_0;
            s_i32_0 = l89;
            s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 4u32)?;
            'blk_16: {
                if s_i32_0 != 0 {
                    s_i32_0 = l42;
                    s_i32_1 = l3;
                    s_i32_2 = 0i32;
                    s_i32_3 = l89;
                    s_i32_3 = ops::i32_load(vm, 0, s_i32_3, 8u32)?;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_i32(vm, c_map, 10usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
                    vm.exit_call();
                    let __t33 = __call_res?;
                    s_i32_0 = __t33;
                    l42 = s_i32_0;
                    s_i32_0 = l42;
                    vm.deduct_1(host)?;
                    l90 = s_i32_0;
                    s_i32_0 = 1i32;
                    l91 = s_i32_0;
                    s_i32_0 = 0i32;
                    l92 = s_i32_0;
                    s_i32_0 = l89;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
                    l93 = s_i32_0;
                    s_i32_0 = 12i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 4usize, s_i32_0, host);
                    vm.exit_call();
                    let __t34 = __call_res?;
                    s_i32_0 = __t34;
                    l94 = s_i32_0;
                    s_i32_0 = l94;
                    s_i32_1 = l90;
                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l94;
                    s_i32_1 = l91;
                    ops::i32_store8(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l94;
                    s_i32_1 = l92;
                    ops::i32_store8(vm, 0, s_i32_0, 5u32, s_i32_1)?;
                    s_i32_0 = l94;
                    s_i32_1 = l93;
                    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                    s_i32_0 = l94;
                    l95 = s_i32_0;
                    s_i32_0 = l95;
                    return Ok(s_i32_0);
                } else {
                }
            }
            s_i32_0 = l42;
            s_i32_1 = l43;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 8u32)?;
            s_i64_2 = 1114725i64;
            l96 = s_i64_2;
            s_i64_2 = 0i64;
            l97 = s_i64_2;
            s_i32_2 = 16i32;
            vm.deduct_1(host)?;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 4usize, s_i32_2, host);
            vm.exit_call();
            let __t35 = __call_res?;
            s_i32_2 = __t35;
            l98 = s_i32_2;
            s_i32_2 = l98;
            s_i64_3 = l96;
            ops::i64_store(vm, 0, s_i32_2, 0u32, s_i64_3)?;
            s_i32_2 = l98;
            s_i64_3 = l97;
            ops::i64_store(vm, 0, s_i32_2, 8u32, s_i64_3)?;
            s_i32_2 = l98;
            s_i32_3 = 0i32;
            s_i64_4 = l2;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i32_i32_i64_ret_i32(vm, c_map, 11usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, s_i64_4, host);
            vm.exit_call();
            let __t36 = __call_res?;
            s_i32_0 = __t36;
            l99 = s_i32_0;
            s_i32_0 = l99;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
            l42 = s_i32_0;
            s_i32_0 = l99;
            s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 4u32)?;
            'blk_17: {
                if s_i32_0 != 0 {
                    s_i32_0 = l42;
                    s_i32_1 = l3;
                    s_i32_2 = 0i32;
                    s_i32_3 = l99;
                    s_i32_3 = ops::i32_load(vm, 0, s_i32_3, 8u32)?;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_i32(vm, c_map, 10usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
                    vm.exit_call();
                    let __t37 = __call_res?;
                    s_i32_0 = __t37;
                    l42 = s_i32_0;
                    s_i32_0 = l42;
                    l100 = s_i32_0;
                    s_i32_0 = 1i32;
                    l101 = s_i32_0;
                    s_i32_0 = 0i32;
                    l102 = s_i32_0;
                    s_i32_0 = l99;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
                    l103 = s_i32_0;
                    s_i32_0 = 12i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 4usize, s_i32_0, host);
                    vm.exit_call();
                    let __t38 = __call_res?;
                    s_i32_0 = __t38;
                    l104 = s_i32_0;
                    s_i32_0 = l104;
                    s_i32_1 = l100;
                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l104;
                    s_i32_1 = l101;
                    ops::i32_store8(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l104;
                    s_i32_1 = l102;
                    ops::i32_store8(vm, 0, s_i32_0, 5u32, s_i32_1)?;
                    s_i32_0 = l104;
                    s_i32_1 = l103;
                    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                    s_i32_0 = l104;
                    l105 = s_i32_0;
                    s_i32_0 = l105;
                    return Ok(s_i32_0);
                } else {
                }
            }
            s_i32_0 = l43;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
            s_i64_1 = 1114416i64;
            l106 = s_i64_1;
            s_i64_1 = 0i64;
            l107 = s_i64_1;
            s_i32_1 = 16i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 4usize, s_i32_1, host);
            vm.exit_call();
            let __t39 = __call_res?;
            s_i32_1 = __t39;
            l108 = s_i32_1;
            s_i32_1 = l108;
            s_i64_2 = l106;
            ops::i64_store(vm, 0, s_i32_1, 0u32, s_i64_2)?;
            s_i32_1 = l108;
            s_i64_2 = l107;
            ops::i64_store(vm, 0, s_i32_1, 8u32, s_i64_2)?;
            s_i32_1 = l108;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_1, host);
            vm.exit_call();
            let __t40 = __call_res?;
            s_i32_1 = __t40;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 12usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t41 = __call_res?;
            s_i32_0 = __t41;
            'blk_18: {
                if s_i32_0 != 0 {
                    s_i32_0 = l42;
                    s_i32_1 = l3;
                    s_i32_2 = 3i32;
                    s_i32_3 = 16i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 4usize, s_i32_3, host);
                    vm.exit_call();
                    let __t42 = __call_res?;
                    s_i32_3 = __t42;
                    l109 = s_i32_3;
                    s_i32_3 = l109;
                    s_i32_4 = l109;
                    s_i32_5 = 16i32;
                    s_i32_4 = ops::i32_add(s_i32_4, s_i32_5);
                    ops::i32_store(vm, 0, s_i32_3, 0u32, s_i32_4)?;
                    s_i32_3 = l109;
                    s_i32_4 = 0i32;
                    ops::i32_store(vm, 0, s_i32_3, 4u32, s_i32_4)?;
                    s_i32_3 = l109;
                    s_i32_4 = 0i32;
                    ops::i32_store(vm, 0, s_i32_3, 8u32, s_i32_4)?;
                    s_i32_3 = l109;
                    s_i32_4 = 1i32;
                    ops::i32_store(vm, 0, s_i32_3, 12u32, s_i32_4)?;
                    s_i32_3 = l109;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_i32(vm, c_map, 10usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
                    vm.exit_call();
                    let __t43 = __call_res?;
                    s_i32_0 = __t43;
                    l42 = s_i32_0;
                    s_i32_0 = l42;
                    l110 = s_i32_0;
                    s_i32_0 = 1i32;
                    l111 = s_i32_0;
                    s_i32_0 = 3i32;
                    l112 = s_i32_0;
                    s_i32_0 = 16i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 4usize, s_i32_0, host);
                    vm.exit_call();
                    let __t44 = __call_res?;
                    s_i32_0 = __t44;
                    l113 = s_i32_0;
                    s_i32_0 = l113;
                    s_i32_1 = l113;
                    s_i32_2 = 16i32;
                    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l113;
                    s_i32_1 = 0i32;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l113;
                    s_i32_1 = 0i32;
                    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                    s_i32_0 = l113;
                    s_i32_1 = 1i32;
                    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                    s_i32_0 = l113;
                    l114 = s_i32_0;
                    s_i32_0 = 12i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 4usize, s_i32_0, host);
                    vm.exit_call();
                    let __t45 = __call_res?;
                    s_i32_0 = __t45;
                    l115 = s_i32_0;
                    s_i32_0 = l115;
                    s_i32_1 = l110;
                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l115;
                    s_i32_1 = l111;
                    ops::i32_store8(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l115;
                    s_i32_1 = l112;
                    ops::i32_store8(vm, 0, s_i32_0, 5u32, s_i32_1)?;
                    s_i32_0 = l115;
                    s_i32_1 = l114;
                    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                    s_i32_0 = l115;
                    l116 = s_i32_0;
                    s_i32_0 = l116;
                    return Ok(s_i32_0);
                } else {
                }
            }
            s_i32_0 = l42;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
            s_i32_1 = l1;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 13usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t46 = __call_res?;
            s_i32_0 = __t46;
            l117 = s_i32_0;
            s_i32_0 = l42;
            l118 = s_i32_0;
            s_i32_0 = l117;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
            l119 = s_i32_0;
            s_i32_0 = 8i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 4usize, s_i32_0, host);
            vm.exit_call();
            let __t47 = __call_res?;
            s_i32_0 = __t47;
            l120 = s_i32_0;
            s_i32_0 = l120;
            s_i32_1 = l118;
            s_i32_2 = 8i32;
            vm.memory_copy(0, 0, s_i32_0, s_i32_1, s_i32_2, host)?;
            s_i32_0 = l120;
            s_i32_1 = l119;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l120;
            l42 = s_i32_0;
            s_i32_0 = l117;
            s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 4u32)?;
            'blk_19: {
                if s_i32_0 != 0 {
                    s_i32_0 = l117;
                    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 5u32)?;
                } else {
                    s_i32_0 = 0i32;
                }
            }
            'blk_20: {
                if s_i32_0 != 0 {
                    s_i32_0 = l42;
                    s_i32_1 = l3;
                    s_i32_2 = 1i32;
                    s_i32_3 = 16i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 4usize, s_i32_3, host);
                    vm.exit_call();
                    let __t48 = __call_res?;
                    s_i32_3 = __t48;
                    l121 = s_i32_3;
                    s_i32_3 = l121;
                    s_i32_4 = l121;
                    s_i32_5 = 16i32;
                    s_i32_4 = ops::i32_add(s_i32_4, s_i32_5);
                    ops::i32_store(vm, 0, s_i32_3, 0u32, s_i32_4)?;
                    s_i32_3 = l121;
                    s_i32_4 = 0i32;
                    ops::i32_store(vm, 0, s_i32_3, 4u32, s_i32_4)?;
                    s_i32_3 = l121;
                    s_i32_4 = 0i32;
                    ops::i32_store(vm, 0, s_i32_3, 8u32, s_i32_4)?;
                    s_i32_3 = l121;
                    s_i32_4 = 1i32;
                    ops::i32_store(vm, 0, s_i32_3, 12u32, s_i32_4)?;
                    s_i32_3 = l121;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_i32(vm, c_map, 10usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
                    vm.exit_call();
                    let __t49 = __call_res?;
                    s_i32_0 = __t49;
                    l42 = s_i32_0;
                    s_i32_0 = l42;
                    l122 = s_i32_0;
                    s_i32_0 = 1i32;
                    l123 = s_i32_0;
                    s_i32_0 = 1i32;
                    l124 = s_i32_0;
                    s_i32_0 = 16i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 4usize, s_i32_0, host);
                    vm.exit_call();
                    let __t50 = __call_res?;
                    s_i32_0 = __t50;
                    l125 = s_i32_0;
                    s_i32_0 = l125;
                    s_i32_1 = l125;
                    s_i32_2 = 16i32;
                    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l125;
                    s_i32_1 = 0i32;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l125;
                    s_i32_1 = 0i32;
                    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                    s_i32_0 = l125;
                    s_i32_1 = 1i32;
                    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                    s_i32_0 = l125;
                    l126 = s_i32_0;
                    s_i32_0 = 12i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 4usize, s_i32_0, host);
                    vm.exit_call();
                    let __t51 = __call_res?;
                    s_i32_0 = __t51;
                    l127 = s_i32_0;
                    s_i32_0 = l127;
                    s_i32_1 = l122;
                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l127;
                    s_i32_1 = l123;
                    ops::i32_store8(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l127;
                    s_i32_1 = l124;
                    ops::i32_store8(vm, 0, s_i32_0, 5u32, s_i32_1)?;
                    s_i32_0 = l127;
                    s_i32_1 = l126;
                    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                    s_i32_0 = l127;
                    l128 = s_i32_0;
                    s_i32_0 = l128;
                    return Ok(s_i32_0);
                } else {
                }
            }
        } else {
        }
    }
    s_i64_0 = 65533i64;
    l129 = s_i64_0;
    s_i64_0 = 0i64;
    l130 = s_i64_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 4usize, s_i32_0, host);
    vm.exit_call();
    let __t52 = __call_res?;
    s_i32_0 = __t52;
    l131 = s_i32_0;
    s_i32_0 = l131;
    s_i64_1 = l129;
    ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
    s_i32_0 = l131;
    s_i64_1 = l130;
    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
    s_i32_0 = l131;
    l132 = s_i32_0;
    s_i32_0 = 20i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 4usize, s_i32_0, host);
    vm.exit_call();
    let __t53 = __call_res?;
    s_i32_0 = __t53;
    l133 = s_i32_0;
    s_i32_0 = l133;
    s_i32_1 = l133;
    s_i32_2 = 16i32;
    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l133;
    s_i32_1 = 1i32;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l133;
    s_i32_1 = 1i32;
    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
    s_i32_0 = l133;
    s_i32_1 = 1i32;
    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
    s_i32_0 = l133;
    s_i32_1 = l132;
    ops::i32_store(vm, 0, s_i32_0, 16u32, s_i32_1)?;
    s_i32_0 = l133;
    l134 = s_i32_0;
    s_i32_0 = l42;
    s_i32_1 = l3;
    s_i32_2 = 2i32;
    s_i32_3 = l134;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i32_i32_i32_ret_i32(vm, c_map, 10usize, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
    vm.exit_call();
    let __t54 = __call_res?;
    s_i32_0 = __t54;
    l42 = s_i32_0;
    s_i32_0 = l42;
    l135 = s_i32_0;
    s_i32_0 = 1i32;
    l136 = s_i32_0;
    s_i32_0 = 2i32;
    l137 = s_i32_0;
    s_i32_0 = l134;
    l138 = s_i32_0;
    s_i32_0 = 12i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 4usize, s_i32_0, host);
    vm.exit_call();
    let __t55 = __call_res?;
    s_i32_0 = __t55;
    l139 = s_i32_0;
    s_i32_0 = l139;
    s_i32_1 = l135;
    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l139;
    s_i32_1 = l136;
    ops::i32_store8(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l139;
    s_i32_1 = l137;
    ops::i32_store8(vm, 0, s_i32_0, 5u32, s_i32_1)?;
    s_i32_0 = l139;
    s_i32_1 = l138;
    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
    s_i32_0 = l139;
    l140 = s_i32_0;
    s_i32_0 = l140;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn f_e759dd99c9c3(vm: &mut SubstituteVm, dispatch: &'static crate::ModuleDispatch, g_map: &'static [usize], c_map: &'static [usize], p0: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = 0i32;
    let mut l2: i64 = 0i64;
    let mut l3: i64 = 0i64;
    let mut l4: i64 = 0i64;
    let mut l5: i64 = 0i64;
    let mut l6: i32 = 0i32;
    let mut l7: i32 = 0i32;
    let mut l8: i32 = 0i32;
    let mut l9: i64 = 0i64;
    let mut l10: i64 = 0i64;
    let mut l11: i32 = 0i32;
    let mut l12: i64 = 0i64;
    let mut l13: i64 = 0i64;
    let mut l14: i32 = 0i32;
    let mut l15: i32 = 0i32;
    let mut l16: i32 = 0i32;
    let mut l17: i32 = 0i32;
    let mut l18: i32 = 0i32;
    let mut l19: i32 = 0i32;
    let mut l20: i64 = 0i64;
    let mut l21: i64 = 0i64;
    let mut l22: i32 = 0i32;
    let mut l23: i32 = 0i32;
    let mut l24: i64 = 0i64;
    let mut l25: i64 = 0i64;
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
    let mut l39: i64 = 0i64;
    let mut l40: i64 = 0i64;
    let mut l41: i32 = 0i32;
    let mut l42: i32 = 0i32;
    let mut l43: i32 = 0i32;
    let mut l44: i64 = 0i64;
    let mut l45: i64 = 0i64;
    let mut l46: i64 = 0i64;
    let mut l47: i32 = 0i32;
    let mut l48: i64 = 0i64;
    let mut l49: i64 = 0i64;
    let mut l50: i32 = 0i32;
    let mut l51: i32 = 0i32;
    let mut l52: i32 = 0i32;
    let mut l53: i64 = 0i64;
    let mut l54: i64 = 0i64;
    let mut l55: i64 = 0i64;
    let mut l56: i64 = 0i64;
    let mut l57: i32 = 0i32;
    let mut l58: i32 = 0i32;
    let mut l59: i32 = 0i32;
    let mut l60: i64 = 0i64;
    let mut l61: i64 = 0i64;
    let mut l62: i64 = 0i64;
    let mut l63: i32 = 0i32;
    let mut l64: i32 = 0i32;
    let mut l65: i32 = 0i32;
    let mut l66: i32 = 0i32;
    let mut l67: i64 = 0i64;
    let mut l68: i64 = 0i64;
    let mut l69: i64 = 0i64;
    let mut l70: i32 = 0i32;
    let mut l71: i32 = 0i32;
    let mut l72: i32 = 0i32;
    let mut l73: i32 = 0i32;
    let mut l74: i64 = 0i64;
    let mut l75: i64 = 0i64;
    let mut l76: i64 = 0i64;
    let mut l77: i32 = 0i32;
    let mut l78: i32 = 0i32;
    let mut l79: i32 = 0i32;
    let mut l80: i32 = 0i32;
    let mut l81: i32 = 0i32;
    let mut l82: i32 = 0i32;
    let mut l83: i32 = 0i32;
    let mut l84: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i32_2: i32 = 0i32;
    let mut s_i64_2: i64 = 0i64;
    let mut s_i32_3: i32 = 0i32;
    let mut s_i64_3: i64 = 0i64;
    let mut s_i32_4: i32 = 0i32;
    vm.deduct_1(host)?;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_unit_ret_void(vm, c_map, 0usize, host);
    vm.exit_call();
    __call_res?;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_unit_ret_i32(vm, c_map, 1usize, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l1 = s_i32_0;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
    l2 = s_i64_0;
    s_i32_0 = l0;
    s_i64_1 = 0i64;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_i64_ret_i64(vm, c_map, 2usize, s_i32_0, s_i64_1, host);
    vm.exit_call();
    let __t1 = __call_res?;
    s_i64_0 = __t1;
    l3 = s_i64_0;
    s_i64_0 = l3;
    s_i64_1 = l2;
    s_i32_0 = ops::i64_lt_u(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i64_1 = l3;
            s_i64_2 = 1114112i64;
            l4 = s_i64_2;
            s_i64_2 = 0i64;
            l5 = s_i64_2;
            s_i32_2 = 16i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_2, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_2 = __t2;
            l6 = s_i32_2;
            s_i32_2 = l6;
            s_i64_3 = l4;
            ops::i64_store(vm, 0, s_i32_2, 0u32, s_i64_3)?;
            s_i32_2 = l6;
            s_i64_3 = l5;
            ops::i64_store(vm, 0, s_i32_2, 8u32, s_i64_3)?;
            s_i32_2 = l6;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i64_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i64_1, s_i32_2, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i32_0 = __t3;
            l7 = s_i32_0;
            s_i32_0 = l7;
            s_i64_1 = 1114753i64;
            l9 = s_i64_1;
            s_i64_1 = 0i64;
            l10 = s_i64_1;
            s_i32_1 = 16i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_1, host);
            vm.exit_call();
            let __t4 = __call_res?;
            s_i32_1 = __t4;
            l11 = s_i32_1;
            s_i32_1 = l11;
            s_i64_2 = l9;
            ops::i64_store(vm, 0, s_i32_1, 0u32, s_i64_2)?;
            s_i32_1 = l11;
            s_i64_2 = l10;
            ops::i64_store(vm, 0, s_i32_1, 8u32, s_i64_2)?;
            s_i32_1 = l11;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 5usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t5 = __call_res?;
            s_i32_0 = __t5;
            l8 = s_i32_0;
            s_i32_0 = l8;
            s_i32_0 = ops::i32_eqz(s_i32_0);
            'blk_2: {
                if s_i32_0 != 0 {
                    s_i32_0 = l7;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 6usize, s_i32_0, host);
                    vm.exit_call();
                    let __t6 = __call_res?;
                    s_i32_0 = __t6;
                    s_i32_1 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 7usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t7 = __call_res?;
                    s_i32_0 = __t7;
                } else {
                    s_i32_0 = 0i32;
                }
            }
            'blk_3: {
                if s_i32_0 != 0 {
                    s_i32_0 = l1;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                    s_i32_1 = l7;
                    s_i64_2 = 1114753i64;
                    l12 = s_i64_2;
                    s_i64_2 = 0i64;
                    l13 = s_i64_2;
                    s_i32_2 = 16i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_2, host);
                    vm.exit_call();
                    let __t8 = __call_res?;
                    s_i32_2 = __t8;
                    l14 = s_i32_2;
                    s_i32_2 = l14;
                    s_i64_3 = l12;
                    ops::i64_store(vm, 0, s_i32_2, 0u32, s_i64_3)?;
                    s_i32_2 = l14;
                    s_i64_3 = l13;
                    ops::i64_store(vm, 0, s_i32_2, 8u32, s_i64_3)?;
                    s_i32_2 = l14;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 8usize, s_i32_0, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    let __t9 = __call_res?;
                    s_i32_0 = __t9;
                    l15 = s_i32_0;
                    s_i32_0 = l1;
                    l16 = s_i32_0;
                    s_i32_0 = l15;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                    l17 = s_i32_0;
                    s_i32_0 = 8i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
                    vm.exit_call();
                    let __t10 = __call_res?;
                    s_i32_0 = __t10;
                    l18 = s_i32_0;
                    s_i32_0 = l18;
                    s_i32_1 = l16;
                    s_i32_2 = 8i32;
                    vm.memory_copy(0, 0, s_i32_0, s_i32_1, s_i32_2, host)?;
                    s_i32_0 = l18;
                    s_i32_1 = l17;
                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l18;
                    l1 = s_i32_0;
                    s_i32_0 = l15;
                    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 4u32)?;
                    'blk_4: {
                        if s_i32_0 != 0 {
                            s_i32_0 = l15;
                            s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 5u32)?;
                            l8 = s_i32_0;
                        } else {
                        }
                    }
                } else {
                }
            }
            s_i32_0 = l8;
            'blk_5: {
                if s_i32_0 != 0 {
                    s_i64_0 = l2;
                    s_i64_1 = l3;
                    s_i64_0 = ops::i64_sub(s_i64_0, s_i64_1);
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i64_ret_i32(vm, c_map, 9usize, s_i64_0, host);
                    vm.exit_call();
                    let __t11 = __call_res?;
                    s_i32_0 = __t11;
                    l19 = s_i32_0;
                    s_i32_0 = l19;
                    s_i64_1 = 1114753i64;
                    l20 = s_i64_1;
                    s_i64_1 = 0i64;
                    l21 = s_i64_1;
                    s_i32_1 = 16i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_1, host);
                    vm.exit_call();
                    let __t12 = __call_res?;
                    s_i32_1 = __t12;
                    l22 = s_i32_1;
                    s_i32_1 = l22;
                    s_i64_2 = l20;
                    ops::i64_store(vm, 0, s_i32_1, 0u32, s_i64_2)?;
                    s_i32_1 = l22;
                    s_i64_2 = l21;
                    ops::i64_store(vm, 0, s_i32_1, 8u32, s_i64_2)?;
                    s_i32_1 = l22;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 10usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l0;
                    l23 = s_i32_0;
                    s_i64_0 = l3;
                    s_i64_1 = 1i64;
                    s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
                    l24 = s_i64_0;
                    s_i64_0 = l2;
                    l25 = s_i64_0;
                    s_i64_0 = l24;
                    s_i64_1 = l25;
                    s_i32_0 = ops::i64_le_u(s_i64_0, s_i64_1);
                    'blk_6: {
                        if s_i32_0 != 0 {
                            s_i64_0 = l25;
                            s_i32_1 = l23;
                            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
                            s_i64_1 = ops::i64_extend_i32_u(s_i32_1);
                            s_i32_0 = ops::i64_le_u(s_i64_0, s_i64_1);
                        } else {
                            s_i32_0 = 0i32;
                        }
                    }
                    'blk_7: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 1i32;
                            l26 = s_i32_0;
                            s_i32_0 = l23;
                            s_i64_1 = l24;
                            s_i64_2 = l25;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i64_i64_ret_i32(vm, c_map, 11usize, s_i32_0, s_i64_1, s_i64_2, host);
                            vm.exit_call();
                            let __t13 = __call_res?;
                            s_i32_0 = __t13;
                            l27 = s_i32_0;
                            s_i32_0 = 8i32;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
                            vm.exit_call();
                            let __t14 = __call_res?;
                            s_i32_0 = __t14;
                            l28 = s_i32_0;
                            s_i32_0 = l28;
                            s_i32_1 = l26;
                            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                            s_i32_0 = l28;
                            s_i32_1 = l27;
                            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                            s_i32_0 = l28;
                        } else {
                            s_i32_0 = 0i32;
                            l29 = s_i32_0;
                            s_i32_0 = 16i32;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
                            vm.exit_call();
                            let __t15 = __call_res?;
                            s_i32_0 = __t15;
                            l30 = s_i32_0;
                            s_i32_0 = l30;
                            s_i32_1 = l30;
                            s_i32_2 = 16i32;
                            s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
                            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                            s_i32_0 = l30;
                            s_i32_1 = 0i32;
                            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                            s_i32_0 = l30;
                            s_i32_1 = 0i32;
                            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                            s_i32_0 = l30;
                            s_i32_1 = 1i32;
                            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                            s_i32_0 = l30;
                            l31 = s_i32_0;
                            s_i32_0 = 8i32;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
                            vm.exit_call();
                            let __t16 = __call_res?;
                            s_i32_0 = __t16;
                            l32 = s_i32_0;
                            s_i32_0 = l32;
                            s_i32_1 = l29;
                            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                            s_i32_0 = l32;
                            s_i32_1 = l31;
                            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                            s_i32_0 = l32;
                        }
                    }
                    l33 = s_i32_0;
                    s_i32_0 = l33;
                    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
                    'blk_8: {
                        if s_i32_0 != 0 {
                            s_i32_0 = l33;
                            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
                            l34 = s_i32_0;
                            s_i32_0 = l19;
                            s_i32_1 = l34;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 12usize, s_i32_0, s_i32_1, host);
                            vm.exit_call();
                            __call_res?;
                        } else {
                        }
                    }
                    s_i32_0 = l19;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 13usize, s_i32_0, host);
                    vm.exit_call();
                    let __t17 = __call_res?;
                    s_i32_0 = __t17;
                    s_i64_1 = 0i64;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i64_ret_i32(vm, c_map, 14usize, s_i32_0, s_i64_1, host);
                    vm.exit_call();
                    let __t18 = __call_res?;
                    s_i32_0 = __t18;
                    l35 = s_i32_0;
                    s_i32_0 = l35;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                    s_i32_1 = 0i32;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                    'blk_9: {
                        if s_i32_0 != 0 {
                            s_i32_0 = l35;
                            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
                            l36 = s_i32_0;
                            s_i64_0 = l3;
                            s_i32_1 = l36;
                            s_i64_1 = ops::i64_load(vm, 0, s_i32_1, 8u32)?;
                            s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
                            l3 = s_i64_0;
                        } else {
                            s_i32_0 = l35;
                            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                            s_i32_1 = 1i32;
                            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                            'blk_10: {
                                if s_i32_0 != 0 {
                                } else {
                                    s_i32_0 = l35;
                                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
                                    l37 = s_i32_0;
                                }
                            }
                        }
                    }
                } else {
                }
            }
        } else {
        }
    }
    s_i64_0 = l2;
    s_i64_1 = l3;
    s_i64_0 = ops::i64_sub(s_i64_0, s_i64_1);
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i64_ret_i32(vm, c_map, 9usize, s_i64_0, host);
    vm.exit_call();
    let __t19 = __call_res?;
    s_i32_0 = __t19;
    l38 = s_i32_0;
    'blk_11: {
        'blk_12: loop {
            vm.deduct_1(host)?;
            s_i64_0 = l3;
            s_i64_1 = l2;
            s_i32_0 = ops::i64_lt_u(s_i64_0, s_i64_1);
            s_i32_0 = ops::i32_eqz(s_i32_0);
            if s_i32_0 != 0 {
                break 'blk_11;
            }
            s_i32_0 = l0;
            s_i64_1 = l3;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i64_ret_i64(vm, c_map, 2usize, s_i32_0, s_i64_1, host);
            vm.exit_call();
            let __t20 = __call_res?;
            s_i64_0 = __t20;
            l3 = s_i64_0;
            s_i64_0 = l3;
            s_i64_1 = l2;
            s_i32_0 = ops::i64_ge_u(s_i64_0, s_i64_1);
            'blk_13: {
                if s_i32_0 != 0 {
                    break 'blk_11;
                } else {
                }
            }
            s_i32_0 = l0;
            s_i64_1 = l3;
            s_i64_2 = 1114112i64;
            l39 = s_i64_2;
            s_i64_2 = 0i64;
            l40 = s_i64_2;
            s_i32_2 = 16i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_2, host);
            vm.exit_call();
            let __t21 = __call_res?;
            s_i32_2 = __t21;
            l41 = s_i32_2;
            s_i32_2 = l41;
            s_i64_3 = l39;
            ops::i64_store(vm, 0, s_i32_2, 0u32, s_i64_3)?;
            s_i32_2 = l41;
            s_i64_3 = l40;
            ops::i64_store(vm, 0, s_i32_2, 8u32, s_i64_3)?;
            s_i32_2 = l41;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i64_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i64_1, s_i32_2, host);
            vm.exit_call();
            let __t22 = __call_res?;
            s_i32_0 = __t22;
            l42 = s_i32_0;
            s_i32_0 = l42;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 15usize, s_i32_0, host);
            vm.exit_call();
            let __t23 = __call_res?;
            s_i32_0 = __t23;
            l43 = s_i32_0;
            s_i32_0 = l43;
            s_i32_1 = 255i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_14: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                } else {
                    s_i32_0 = l43;
                    s_i32_1 = 2239i32;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                }
            }
            'blk_15: {
                if s_i32_0 != 0 {
                    s_i64_0 = l3;
                    s_i64_1 = 1i64;
                    s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
                    l44 = s_i64_0;
                    s_i64_0 = l44;
                    s_i64_1 = l2;
                    s_i32_0 = ops::i64_ge_u(s_i64_0, s_i64_1);
                    'blk_16: {
                        if s_i32_0 != 0 {
                            s_i32_0 = l38;
                            s_i64_1 = 65533i64;
                            l45 = s_i64_1;
                            s_i64_1 = 0i64;
                            l46 = s_i64_1;
                            s_i32_1 = 16i32;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_1, host);
                            vm.exit_call();
                            let __t24 = __call_res?;
                            s_i32_1 = __t24;
                            l47 = s_i32_1;
                            s_i32_1 = l47;
                            s_i64_2 = l45;
                            ops::i64_store(vm, 0, s_i32_1, 0u32, s_i64_2)?;
                            s_i32_1 = l47;
                            s_i64_2 = l46;
                            ops::i64_store(vm, 0, s_i32_1, 8u32, s_i64_2)?;
                            s_i32_1 = l47;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 10usize, s_i32_0, s_i32_1, host);
                            vm.exit_call();
                            __call_res?;
                            s_i64_0 = l2;
                            l3 = s_i64_0;
                            continue 'blk_12;
                        } else {
                        }
                    }
                    s_i32_0 = l0;
                    s_i64_1 = l44;
                    s_i64_2 = 1114112i64;
                    l48 = s_i64_2;
                    s_i64_2 = 0i64;
                    l49 = s_i64_2;
                    s_i32_2 = 16i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_2, host);
                    vm.exit_call();
                    let __t25 = __call_res?;
                    s_i32_2 = __t25;
                    l50 = s_i32_2;
                    s_i32_2 = l50;
                    s_i64_3 = l48;
                    ops::i64_store(vm, 0, s_i32_2, 0u32, s_i64_3)?;
                    s_i32_2 = l50;
                    s_i64_3 = l49;
                    ops::i64_store(vm, 0, s_i32_2, 8u32, s_i64_3)?;
                    s_i32_2 = l50;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i64_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i64_1, s_i32_2, host);
                    vm.exit_call();
                    let __t26 = __call_res?;
                    s_i32_0 = __t26;
                    l51 = s_i32_0;
                    s_i32_0 = l51;
                    l52 = s_i32_0;
                    s_i64_0 = l44;
                    s_i64_1 = 1i64;
                    s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
                    l53 = s_i64_0;
                    s_i32_0 = l43;
                    s_i32_1 = 2239i32;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                    'blk_17: {
                        if s_i32_0 != 0 {
                            s_i32_0 = l51;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 15usize, s_i32_0, host);
                            vm.exit_call();
                            let __t27 = __call_res?;
                            s_i32_0 = __t27;
                            s_i32_1 = 255i32;
                            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                        } else {
                            s_i32_0 = 0i32;
                        }
                    }
                    'blk_18: {
                        if s_i32_0 != 0 {
                            s_i64_0 = l3;
                            s_i64_1 = 2i64;
                            s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
                            l54 = s_i64_0;
                            s_i64_0 = l54;
                            s_i64_1 = l2;
                            s_i32_0 = ops::i64_lt_u(s_i64_0, s_i64_1);
                            'blk_19: {
                                if s_i32_0 != 0 {
                                    s_i32_0 = l0;
                                    s_i64_1 = l54;
                                    s_i64_2 = 1114112i64;
                                    l55 = s_i64_2;
                                    s_i64_2 = 0i64;
                                    l56 = s_i64_2;
                                    s_i32_2 = 16i32;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_2, host);
                                    vm.exit_call();
                                    let __t28 = __call_res?;
                                    s_i32_2 = __t28;
                                    l57 = s_i32_2;
                                    s_i32_2 = l57;
                                    s_i64_3 = l55;
                                    ops::i64_store(vm, 0, s_i32_2, 0u32, s_i64_3)?;
                                    s_i32_2 = l57;
                                    s_i64_3 = l56;
                                    ops::i64_store(vm, 0, s_i32_2, 8u32, s_i64_3)?;
                                    s_i32_2 = l57;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_i64_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i64_1, s_i32_2, host);
                                    vm.exit_call();
                                    let __t29 = __call_res?;
                                    s_i32_0 = __t29;
                                    l52 = s_i32_0;
                                    s_i64_0 = l54;
                                    s_i64_1 = 1i64;
                                    s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
                                    l53 = s_i64_0;
                                } else {
                                }
                            }
                        } else {
                        }
                    }
                    s_i32_0 = l1;
                    s_i32_1 = l52;
                    s_i64_2 = 0i64;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_i64_ret_i32(vm, c_map, 16usize, s_i32_0, s_i32_1, s_i64_2, host);
                    vm.exit_call();
                    let __t30 = __call_res?;
                    s_i32_0 = __t30;
                    l58 = s_i32_0;
                    s_i32_0 = l58;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                    l1 = s_i32_0;
                    s_i32_0 = l38;
                    s_i32_1 = l58;
                    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 8u32)?;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 12usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                    s_i64_0 = l53;
                    l3 = s_i64_0;
                    continue 'blk_12;
                } else {
                }
            }
            s_i32_0 = l43;
            s_i32_1 = 552i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_20: {
                if s_i32_0 != 0 {
                    s_i32_0 = l38;
                    s_i32_1 = 24i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_1, host);
                    vm.exit_call();
                    let __t31 = __call_res?;
                    s_i32_1 = __t31;
                    l59 = s_i32_1;
                    s_i32_1 = l59;
                    s_i32_2 = l59;
                    s_i32_3 = 16i32;
                    s_i32_2 = ops::i32_add(s_i32_2, s_i32_3);
                    ops::i32_store(vm, 0, s_i32_1, 0u32, s_i32_2)?;
                    s_i32_1 = l59;
                    s_i32_2 = 8i32;
                    ops::i32_store(vm, 0, s_i32_1, 4u32, s_i32_2)?;
                    s_i32_1 = l59;
                    s_i32_2 = 8i32;
                    ops::i32_store(vm, 0, s_i32_1, 8u32, s_i32_2)?;
                    s_i32_1 = l59;
                    s_i32_2 = 1i32;
                    ops::i32_store(vm, 0, s_i32_1, 12u32, s_i32_2)?;
                    s_i32_1 = l59;
                    s_i64_2 = 2322294380849947995i64;
                    ops::i64_store(vm, 0, s_i32_1, 16u32, s_i64_2)?;
                    s_i32_1 = l59;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 17usize, s_i32_1, host);
                    vm.exit_call();
                    let __t32 = __call_res?;
                    s_i32_1 = __t32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 12usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                    s_i64_0 = l3;
                    s_i64_1 = 1i64;
                    s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
                    l60 = s_i64_0;
                    'blk_21: {
                        'blk_22: loop {
                            vm.deduct_1(host)?;
                            s_i64_0 = l60;
                            s_i64_1 = l2;
                            s_i32_0 = ops::i64_lt_u(s_i64_0, s_i64_1);
                            s_i32_0 = ops::i32_eqz(s_i32_0);
                            if s_i32_0 != 0 {
                                break 'blk_21;
                            }
                            s_i32_0 = l0;
                            s_i64_1 = l60;
                            s_i64_2 = 1114112i64;
                            l61 = s_i64_2;
                            s_i64_2 = 0i64;
                            l62 = s_i64_2;
                            s_i32_2 = 16i32;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_2, host);
                            vm.exit_call();
                            let __t33 = __call_res?;
                            s_i32_2 = __t33;
                            l63 = s_i32_2;
                            s_i32_2 = l63;
                            s_i64_3 = l61;
                            ops::i64_store(vm, 0, s_i32_2, 0u32, s_i64_3)?;
                            s_i32_2 = l63;
                            s_i64_3 = l62;
                            ops::i64_store(vm, 0, s_i32_2, 8u32, s_i64_3)?;
                            s_i32_2 = l63;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i64_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i64_1, s_i32_2, host);
                            vm.exit_call();
                            let __t34 = __call_res?;
                            s_i32_0 = __t34;
                            l64 = s_i32_0;
                            s_i32_0 = l64;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 15usize, s_i32_0, host);
                            vm.exit_call();
                            let __t35 = __call_res?;
                            s_i32_0 = __t35;
                            l65 = s_i32_0;
                            s_i32_0 = l65;
                            s_i32_1 = 553i32;
                            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                            'blk_23: {
                                if s_i32_0 != 0 {
                                    s_i64_0 = l60;
                                    s_i64_1 = 1i64;
                                    s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
                                    l60 = s_i64_0;
                                    break 'blk_21;
                                } else {
                                }
                            }
                            s_i32_0 = l64;
                            l66 = s_i32_0;
                            s_i32_0 = l65;
                            s_i32_1 = 255i32;
                            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                            'blk_24: {
                                if s_i32_0 != 0 {
                                    s_i64_0 = l60;
                                    s_i64_1 = 1i64;
                                    s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
                                    l67 = s_i64_0;
                                    s_i64_0 = l67;
                                    s_i64_1 = l2;
                                    s_i32_0 = ops::i64_lt_u(s_i64_0, s_i64_1);
                                    'blk_25: {
                                        if s_i32_0 != 0 {
                                            s_i32_0 = l0;
                                            s_i64_1 = l67;
                                            s_i64_2 = 1114112i64;
                                            l68 = s_i64_2;
                                            s_i64_2 = 0i64;
                                            l69 = s_i64_2;
                                            s_i32_2 = 16i32;
                                            vm.enter_call()?;
                                            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_2, host);
                                            vm.exit_call();
                                            let __t36 = __call_res?;
                                            s_i32_2 = __t36;
                                            l70 = s_i32_2;
                                            s_i32_2 = l70;
                                            s_i64_3 = l68;
                                            ops::i64_store(vm, 0, s_i32_2, 0u32, s_i64_3)?;
                                            s_i32_2 = l70;
                                            s_i64_3 = l69;
                                            ops::i64_store(vm, 0, s_i32_2, 8u32, s_i64_3)?;
                                            s_i32_2 = l70;
                                            vm.enter_call()?;
                                            let __call_res = dispatch.call_p_i32_i64_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i64_1, s_i32_2, host);
                                            vm.exit_call();
                                            let __t37 = __call_res?;
                                            s_i32_0 = __t37;
                                            l66 = s_i32_0;
                                            s_i64_0 = l67;
                                            s_i64_1 = 1i64;
                                            s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
                                            l60 = s_i64_0;
                                        } else {
                                            s_i64_0 = l2;
                                            l60 = s_i64_0;
                                        }
                                    }
                                } else {
                                    s_i64_0 = l60;
                                    s_i64_1 = 1i64;
                                    s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
                                    l60 = s_i64_0;
                                }
                            }
                            s_i32_0 = l1;
                            s_i32_1 = l66;
                            s_i64_2 = 0i64;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_i64_ret_i32(vm, c_map, 16usize, s_i32_0, s_i32_1, s_i64_2, host);
                            vm.exit_call();
                            let __t38 = __call_res?;
                            s_i32_0 = __t38;
                            l71 = s_i32_0;
                            s_i32_0 = l71;
                            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                            l1 = s_i32_0;
                            s_i32_0 = l38;
                            s_i32_1 = l71;
                            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 8u32)?;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 12usize, s_i32_0, s_i32_1, host);
                            vm.exit_call();
                            __call_res?;
                            continue 'blk_22;
                            break 'blk_22;
                        }
                    }
                    s_i32_0 = l38;
                    s_i32_1 = 18i32;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_1, host);
                    vm.exit_call();
                    let __t39 = __call_res?;
                    s_i32_1 = __t39;
                    l72 = s_i32_1;
                    s_i32_1 = l72;
                    s_i32_2 = l72;
                    s_i32_3 = 16i32;
                    s_i32_2 = ops::i32_add(s_i32_2, s_i32_3);
                    ops::i32_store(vm, 0, s_i32_1, 0u32, s_i32_2)?;
                    s_i32_1 = l72;
                    s_i32_2 = 2i32;
                    ops::i32_store(vm, 0, s_i32_1, 4u32, s_i32_2)?;
                    s_i32_1 = l72;
                    s_i32_2 = 2i32;
                    ops::i32_store(vm, 0, s_i32_1, 8u32, s_i32_2)?;
                    s_i32_1 = l72;
                    s_i32_2 = 1i32;
                    ops::i32_store(vm, 0, s_i32_1, 12u32, s_i32_2)?;
                    s_i32_1 = l72;
                    s_i32_2 = 93i32;
                    ops::i32_store8(vm, 0, s_i32_1, 16u32, s_i32_2)?;
                    s_i32_1 = l72;
                    s_i32_2 = 10i32;
                    ops::i32_store8(vm, 0, s_i32_1, 17u32, s_i32_2)?;
                    s_i32_1 = l72;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 17usize, s_i32_1, host);
                    vm.exit_call();
                    let __t40 = __call_res?;
                    s_i32_1 = __t40;
                    vm.enter_call()?;
                    let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 12usize, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                    s_i64_0 = l60;
                    l3 = s_i64_0;
                    continue 'blk_12;
                } else {
                }
            }
            s_i32_0 = l1;
            s_i32_1 = l42;
            s_i64_2 = 0i64;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_i64_ret_i32(vm, c_map, 16usize, s_i32_0, s_i32_1, s_i64_2, host);
            vm.exit_call();
            let __t41 = __call_res?;
            s_i32_0 = __t41;
            l73 = s_i32_0;
            s_i32_0 = l73;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
            l1 = s_i32_0;
            s_i32_0 = l73;
            s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 5u32)?;
            s_i32_1 = 3i32;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 18usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t42 = __call_res?;
            s_i32_0 = __t42;
            'blk_26: {
                if s_i32_0 != 0 {
                    s_i64_0 = l3;
                    s_i64_1 = 1i64;
                    s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
                    l74 = s_i64_0;
                    s_i64_0 = l74;
                    s_i64_1 = l2;
                    s_i32_0 = ops::i64_lt_u(s_i64_0, s_i64_1);
                    'blk_27: {
                        if s_i32_0 != 0 {
                            s_i32_0 = l0;
                            s_i64_1 = l74;
                            s_i64_2 = 1114112i64;
                            l75 = s_i64_2;
                            s_i64_2 = 0i64;
                            l76 = s_i64_2;
                            s_i32_2 = 16i32;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_2, host);
                            vm.exit_call();
                            let __t43 = __call_res?;
                            s_i32_2 = __t43;
                            l77 = s_i32_2;
                            s_i32_2 = l77;
                            s_i64_3 = l75;
                            ops::i64_store(vm, 0, s_i32_2, 0u32, s_i64_3)?;
                            s_i32_2 = l77;
                            s_i64_3 = l76;
                            ops::i64_store(vm, 0, s_i32_2, 8u32, s_i64_3)?;
                            s_i32_2 = l77;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i64_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i64_1, s_i32_2, host);
                            vm.exit_call();
                            let __t44 = __call_res?;
                            s_i32_0 = __t44;
                            l78 = s_i32_0;
                            s_i32_0 = l78;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 6usize, s_i32_0, host);
                            vm.exit_call();
                            let __t45 = __call_res?;
                            s_i32_0 = __t45;
                            s_i32_1 = vm.get_i32(*g_map.get(0usize).ok_or(WasmTrap::Unreachable)?)?;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 7usize, s_i32_0, s_i32_1, host);
                            vm.exit_call();
                            let __t46 = __call_res?;
                            s_i32_0 = __t46;
                            'blk_28: {
                                if s_i32_0 != 0 {
                                    s_i32_0 = l1;
                                    vm.deduct_1(host)?;
                                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                                    s_i32_1 = l78;
                                    s_i32_2 = 23i32;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_2, host);
                                    vm.exit_call();
                                    let __t47 = __call_res?;
                                    s_i32_2 = __t47;
                                    l79 = s_i32_2;
                                    s_i32_2 = l79;
                                    s_i32_3 = l79;
                                    s_i32_4 = 16i32;
                                    s_i32_3 = ops::i32_add(s_i32_3, s_i32_4);
                                    ops::i32_store(vm, 0, s_i32_2, 0u32, s_i32_3)?;
                                    s_i32_2 = l79;
                                    s_i32_3 = 7i32;
                                    ops::i32_store(vm, 0, s_i32_2, 4u32, s_i32_3)?;
                                    s_i32_2 = l79;
                                    s_i32_3 = 7i32;
                                    ops::i32_store(vm, 0, s_i32_2, 8u32, s_i32_3)?;
                                    s_i32_2 = l79;
                                    s_i32_3 = 1i32;
                                    ops::i32_store(vm, 0, s_i32_2, 12u32, s_i32_3)?;
                                    s_i32_2 = l79;
                                    s_i32_3 = 46i32;
                                    ops::i32_store8(vm, 0, s_i32_2, 16u32, s_i32_3)?;
                                    s_i32_2 = l79;
                                    s_i32_3 = 67i32;
                                    ops::i32_store8(vm, 0, s_i32_2, 17u32, s_i32_3)?;
                                    s_i32_2 = l79;
                                    s_i32_3 = 111i32;
                                    ops::i32_store8(vm, 0, s_i32_2, 18u32, s_i32_3)?;
                                    s_i32_2 = l79;
                                    s_i32_3 = 108i32;
                                    ops::i32_store8(vm, 0, s_i32_2, 19u32, s_i32_3)?;
                                    s_i32_2 = l79;
                                    s_i32_3 = 111i32;
                                    ops::i32_store8(vm, 0, s_i32_2, 20u32, s_i32_3)?;
                                    s_i32_2 = l79;
                                    s_i32_3 = 114i32;
                                    ops::i32_store8(vm, 0, s_i32_2, 21u32, s_i32_3)?;
                                    s_i32_2 = l79;
                                    s_i32_3 = 115i32;
                                    ops::i32_store8(vm, 0, s_i32_2, 22u32, s_i32_3)?;
                                    s_i32_2 = l79;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 19usize, s_i32_0, s_i32_1, s_i32_2, host);
                                    vm.exit_call();
                                    let __t48 = __call_res?;
                                    s_i32_0 = __t48;
                                    l80 = s_i32_0;
                                    s_i32_0 = l1;
                                    l81 = s_i32_0;
                                    s_i32_0 = l80;
                                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                                    l82 = s_i32_0;
                                    s_i32_0 = 8i32;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
                                    vm.exit_call();
                                    let __t49 = __call_res?;
                                    s_i32_0 = __t49;
                                    l83 = s_i32_0;
                                    s_i32_0 = l83;
                                    s_i32_1 = l81;
                                    s_i32_2 = 8i32;
                                    vm.memory_copy(0, 0, s_i32_0, s_i32_1, s_i32_2, host)?;
                                    s_i32_0 = l83;
                                    s_i32_1 = l82;
                                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                                    s_i32_0 = l83;
                                    l1 = s_i32_0;
                                    s_i32_0 = l80;
                                    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 4u32)?;
                                    'blk_29: {
                                        if s_i32_0 != 0 {
                                            s_i32_0 = l80;
                                            s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 5u32)?;
                                        } else {
                                            s_i32_0 = 0i32;
                                        }
                                    }
                                    'blk_30: {
                                        if s_i32_0 != 0 {
                                            s_i64_0 = l74;
                                            s_i64_1 = 1i64;
                                            s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
                                            l74 = s_i64_0;
                                        } else {
                                        }
                                    }
                                } else {
                                }
                            }
                        } else {
                        }
                    }
                    s_i64_0 = l74;
                    vm.deduct_1(host)?;
                    l3 = s_i64_0;
                    continue 'blk_12;
                } else {
                }
            }
            s_i32_0 = l38;
            s_i32_1 = l73;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 8u32)?;
            vm.enter_call()?;
            let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 12usize, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i64_0 = l3;
            s_i64_1 = 1i64;
            s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
            l3 = s_i64_0;
            continue 'blk_12;
            break 'blk_12;
        }
    }
    s_i32_0 = l38;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 13usize, s_i32_0, host);
    vm.exit_call();
    let __t50 = __call_res?;
    s_i32_0 = __t50;
    l84 = s_i32_0;
    s_i32_0 = l84;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn f_1cf4620fa532(vm: &mut SubstituteVm, dispatch: &'static crate::ModuleDispatch, g_map: &'static [usize], c_map: &'static [usize], p0: i32, p1: i32, p2: i32, p3: i32, p4: i64, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = p2;
    let mut l3: i32 = p3;
    let mut l4: i64 = p4;
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
    let mut l54: i32 = 0i32;
    let mut l55: i32 = 0i32;
    let mut l56: i32 = 0i32;
    let mut l57: i32 = 0i32;
    let mut l58: i32 = 0i32;
    let mut l59: i32 = 0i32;
    let mut l60: i32 = 0i32;
    let mut l61: i32 = 0i32;
    let mut l62: i32 = 0i32;
    let mut l63: i32 = 0i32;
    let mut l64: i32 = 0i32;
    let mut l65: i32 = 0i32;
    let mut l66: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i32_2: i32 = 0i32;
    let mut s_i64_2: i64 = 0i64;
    let mut s_i64_3: i64 = 0i64;
    vm.deduct_1(host)?;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_unit_ret_void(vm, c_map, 0usize, host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    l5 = s_i32_0;
    s_i32_0 = l1;
    l6 = s_i32_0;
    s_i32_0 = l6;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    l7 = s_i32_0;
    s_i32_0 = l6;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    l8 = s_i32_0;
    s_i32_0 = 0i32;
    l9 = s_i32_0;
    'blk_1: {
        'blk_2: loop {
            vm.deduct_1(host)?;
            s_i32_0 = l9;
            s_i32_1 = l8;
            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
            if s_i32_0 != 0 {
                break 'blk_1;
            }
            s_i32_0 = l7;
            s_i32_1 = l9;
            s_i32_2 = 4i32;
            s_i32_1 = ops::i32_mul(s_i32_1, s_i32_2);
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
            l10 = s_i32_0;
            'blk_3: {
                s_i32_0 = l5;
                s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                s_i32_1 = l10;
                vm.enter_call()?;
                let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 1usize, s_i32_1, host);
                vm.exit_call();
                let __t0 = __call_res?;
                s_i32_1 = __t0;
                s_i32_2 = l2;
                vm.enter_call()?;
                let __call_res = dispatch.call_p_i32_i32_i32_ret_i32(vm, c_map, 2usize, s_i32_0, s_i32_1, s_i32_2, host);
                vm.exit_call();
                let __t1 = __call_res?;
                s_i32_0 = __t1;
                l11 = s_i32_0;
                s_i32_0 = l5;
                l12 = s_i32_0;
                s_i32_0 = l11;
                s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                l13 = s_i32_0;
                s_i32_0 = 8i32;
                vm.enter_call()?;
                let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
                vm.exit_call();
                let __t2 = __call_res?;
                s_i32_0 = __t2;
                l14 = s_i32_0;
                s_i32_0 = l14;
                s_i32_1 = l12;
                s_i32_2 = 8i32;
                vm.memory_copy(0, 0, s_i32_0, s_i32_1, s_i32_2, host)?;
                s_i32_0 = l14;
                s_i32_1 = l13;
                ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                s_i32_0 = l14;
                l5 = s_i32_0;
                s_i32_0 = l11;
                s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 4u32)?;
                s_i32_0 = ops::i32_eqz(s_i32_0);
                'blk_4: {
                    if s_i32_0 != 0 {
                        s_i32_0 = l5;
                        l15 = s_i32_0;
                        s_i32_0 = 0i32;
                        l16 = s_i32_0;
                        s_i32_0 = 2i32;
                        l17 = s_i32_0;
                        s_i32_0 = 16i32;
                        vm.enter_call()?;
                        let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
                        vm.exit_call();
                        let __t3 = __call_res?;
                        s_i32_0 = __t3;
                        l18 = s_i32_0;
                        s_i32_0 = l18;
                        s_i32_1 = l18;
                        s_i32_2 = 16i32;
                        s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
                        ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                        s_i32_0 = l18;
                        s_i32_1 = 0i32;
                        ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                        s_i32_0 = l18;
                        s_i32_1 = 0i32;
                        ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                        s_i32_0 = l18;
                        s_i32_1 = 1i32;
                        ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                        s_i32_0 = l18;
                        l19 = s_i32_0;
                        s_i32_0 = 12i32;
                        vm.enter_call()?;
                        let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
                        vm.exit_call();
                        let __t4 = __call_res?;
                        s_i32_0 = __t4;
                        l20 = s_i32_0;
                        s_i32_0 = l20;
                        s_i32_1 = l15;
                        ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                        s_i32_0 = l20;
                        s_i32_1 = l16;
                        ops::i32_store8(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                        s_i32_0 = l20;
                        s_i32_1 = l17;
                        ops::i32_store8(vm, 0, s_i32_0, 5u32, s_i32_1)?;
                        s_i32_0 = l20;
                        s_i32_1 = l19;
                        ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                        s_i32_0 = l20;
                        l21 = s_i32_0;
                        s_i32_0 = l21;
                        return Ok(s_i32_0);
                    } else {
                    }
                }
                s_i32_0 = l11;
                s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
                l22 = s_i32_0;
                s_i32_0 = l22;
                s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                l23 = s_i32_0;
                s_i32_0 = l22;
                s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
                l24 = s_i32_0;
                s_i32_0 = 0i32;
                l25 = s_i32_0;
                'blk_5: {
                    'blk_6: loop {
                        vm.deduct_1(host)?;
                        s_i32_0 = l25;
                        s_i32_1 = l24;
                        s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                        if s_i32_0 != 0 {
                            break 'blk_5;
                        }
                        s_i32_0 = l23;
                        s_i32_1 = l25;
                        s_i32_2 = 4i32;
                        s_i32_1 = ops::i32_mul(s_i32_1, s_i32_2);
                        s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                        s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                        l26 = s_i32_0;
                        'blk_7: {
                            s_i32_0 = l5;
                            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                            s_i32_1 = l26;
                            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 0u32)?;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 4usize, s_i32_0, s_i32_1, host);
                            vm.exit_call();
                            let __t5 = __call_res?;
                            s_i32_0 = __t5;
                            l27 = s_i32_0;
                            s_i32_0 = l5;
                            l28 = s_i32_0;
                            s_i32_0 = l27;
                            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                            l29 = s_i32_0;
                            s_i32_0 = 8i32;
                            vm.enter_call()?;
                            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
                            vm.exit_call();
                            let __t6 = __call_res?;
                            s_i32_0 = __t6;
                            l30 = s_i32_0;
                            s_i32_0 = l30;
                            s_i32_1 = l28;
                            s_i32_2 = 8i32;
                            vm.memory_copy(0, 0, s_i32_0, s_i32_1, s_i32_2, host)?;
                            s_i32_0 = l30;
                            s_i32_1 = l29;
                            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                            s_i32_0 = l30;
                            l5 = s_i32_0;
                            s_i32_0 = l27;
                            s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 4u32)?;
                            s_i32_0 = ops::i32_eqz(s_i32_0);
                            'blk_8: {
                                if s_i32_0 != 0 {
                                    s_i32_0 = l5;
                                    l31 = s_i32_0;
                                    s_i32_0 = 0i32;
                                    l32 = s_i32_0;
                                    s_i32_0 = 2i32;
                                    l33 = s_i32_0;
                                    s_i32_0 = 16i32;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
                                    vm.exit_call();
                                    let __t7 = __call_res?;
                                    s_i32_0 = __t7;
                                    l34 = s_i32_0;
                                    s_i32_0 = l34;
                                    s_i32_1 = l34;
                                    s_i32_2 = 16i32;
                                    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
                                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                                    s_i32_0 = l34;
                                    s_i32_1 = 0i32;
                                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                                    s_i32_0 = l34;
                                    s_i32_1 = 0i32;
                                    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                                    s_i32_0 = l34;
                                    s_i32_1 = 1i32;
                                    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                                    s_i32_0 = l34;
                                    l35 = s_i32_0;
                                    s_i32_0 = 12i32;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
                                    vm.exit_call();
                                    let __t8 = __call_res?;
                                    s_i32_0 = __t8;
                                    l36 = s_i32_0;
                                    s_i32_0 = l36;
                                    s_i32_1 = l31;
                                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                                    s_i32_0 = l36;
                                    s_i32_1 = l32;
                                    ops::i32_store8(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                                    s_i32_0 = l36;
                                    s_i32_1 = l33;
                                    ops::i32_store8(vm, 0, s_i32_0, 5u32, s_i32_1)?;
                                    s_i32_0 = l36;
                                    s_i32_1 = l35;
                                    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                                    s_i32_0 = l36;
                                    l37 = s_i32_0;
                                    s_i32_0 = l37;
                                    return Ok(s_i32_0);
                                } else {
                                }
                            }
                            s_i32_0 = l27;
                            s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 5u32)?;
                            s_i32_0 = ops::i32_eqz(s_i32_0);
                            'blk_9: {
                                if s_i32_0 != 0 {
                                    s_i32_0 = l26;
                                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 5usize, s_i32_0, host);
                                    vm.exit_call();
                                    let __t9 = __call_res?;
                                    s_i32_0 = __t9;
                                    l38 = s_i32_0;
                                    s_i32_0 = l38;
                                    vm.enter_call()?;
                                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 6usize, s_i32_0, host);
                                    vm.exit_call();
                                    let __t10 = __call_res?;
                                    s_i32_0 = __t10;
                                    'blk_10: {
                                        if s_i32_0 != 0 {
                                            s_i32_0 = l5;
                                            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                                            s_i32_1 = l38;
                                            vm.enter_call()?;
                                            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 7usize, s_i32_0, s_i32_1, host);
                                            vm.exit_call();
                                            let __t11 = __call_res?;
                                            s_i32_0 = __t11;
                                            l39 = s_i32_0;
                                            s_i32_0 = l5;
                                            l40 = s_i32_0;
                                            s_i32_0 = l39;
                                            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                                            l41 = s_i32_0;
                                            s_i32_0 = 8i32;
                                            vm.enter_call()?;
                                            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
                                            vm.exit_call();
                                            let __t12 = __call_res?;
                                            s_i32_0 = __t12;
                                            l42 = s_i32_0;
                                            s_i32_0 = l42;
                                            s_i32_1 = l40;
                                            s_i32_2 = 8i32;
                                            vm.memory_copy(0, 0, s_i32_0, s_i32_1, s_i32_2, host)?;
                                            s_i32_0 = l42;
                                            s_i32_1 = l41;
                                            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                                            s_i32_0 = l42;
                                            l5 = s_i32_0;
                                            s_i32_0 = l39;
                                            s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 4u32)?;
                                            s_i32_1 = 1i32;
                                            vm.enter_call()?;
                                            let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 8usize, s_i32_0, s_i32_1, host);
                                            vm.exit_call();
                                            let __t13 = __call_res?;
                                            s_i32_0 = __t13;
                                            'blk_11: {
                                                if s_i32_0 != 0 {
                                                    s_i32_0 = l39;
                                                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
                                                    vm.enter_call()?;
                                                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 9usize, s_i32_0, host);
                                                    vm.exit_call();
                                                    let __t14 = __call_res?;
                                                    s_i32_0 = __t14;
                                                    s_i32_0 = ops::i32_eqz(s_i32_0);
                                                } else {
                                                    s_i32_0 = 0i32;
                                                }
                                            }
                                            'blk_12: {
                                                if s_i32_0 != 0 {
                                                    s_i32_0 = l39;
                                                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
                                                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
                                                    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
                                                    vm.enter_call()?;
                                                    let __call_res = dispatch.call_p_i64_ret_i32(vm, c_map, 10usize, s_i64_0, host);
                                                    vm.exit_call();
                                                    let __t15 = __call_res?;
                                                    s_i32_0 = __t15;
                                                    l43 = s_i32_0;
                                                    s_i32_0 = 1i32;
                                                    l44 = s_i32_0;
                                                    s_i32_0 = l39;
                                                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
                                                    l45 = s_i32_0;
                                                    s_i32_0 = l45;
                                                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                                                    l46 = s_i32_0;
                                                    s_i32_0 = l45;
                                                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
                                                    l47 = s_i32_0;
                                                    s_i32_0 = 0i32;
                                                    l48 = s_i32_0;
                                                    'blk_13: {
                                                        'blk_14: loop {
                                                            vm.deduct_1(host)?;
                                                            s_i32_0 = l48;
                                                            s_i32_1 = l47;
                                                            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
                                                            if s_i32_0 != 0 {
                                                                break 'blk_13;
                                                            }
                                                            s_i32_0 = l46;
                                                            s_i32_1 = l48;
                                                            s_i32_2 = 4i32;
                                                            s_i32_1 = ops::i32_mul(s_i32_1, s_i32_2);
                                                            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                                                            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                                                            l49 = s_i32_0;
                                                            'blk_15: {
                                                                s_i32_0 = l5;
                                                                s_i32_1 = l49;
                                                                s_i64_2 = l4;
                                                                s_i64_3 = 1i64;
                                                                s_i64_2 = ops::i64_add(s_i64_2, s_i64_3);
                                                                vm.enter_call()?;
                                                                let __call_res = dispatch.call_p_i32_i32_i64_ret_i32(vm, c_map, 11usize, s_i32_0, s_i32_1, s_i64_2, host);
                                                                vm.exit_call();
                                                                let __t16 = __call_res?;
                                                                s_i32_0 = __t16;
                                                                l50 = s_i32_0;
                                                                s_i32_0 = l50;
                                                                s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                                                                l5 = s_i32_0;
                                                                s_i32_0 = l50;
                                                                s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 4u32)?;
                                                                'blk_16: {
                                                                    if s_i32_0 != 0 {
                                                                        s_i32_0 = l50;
                                                                        s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 5u32)?;
                                                                        s_i32_1 = 0i32;
                                                                        vm.enter_call()?;
                                                                        let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 8usize, s_i32_0, s_i32_1, host);
                                                                        vm.exit_call();
                                                                        let __t17 = __call_res?;
                                                                        s_i32_0 = __t17;
                                                                    } else {
                                                                        s_i32_0 = 0i32;
                                                                    }
                                                                }
                                                                'blk_17: {
                                                                    if s_i32_0 != 0 {
                                                                        s_i32_0 = l43;
                                                                        s_i32_1 = l50;
                                                                        s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 8u32)?;
                                                                        vm.enter_call()?;
                                                                        let __call_res = dispatch.call_p_i32_i32_ret_void(vm, c_map, 12usize, s_i32_0, s_i32_1, host);
                                                                        vm.exit_call();
                                                                        __call_res?;
                                                                    } else {
                                                                        s_i32_0 = 0i32;
                                                                        l44 = s_i32_0;
                                                                        break 'blk_13;
                                                                    }
                                                                }
                                                            }
                                                            s_i32_0 = l48;
                                                            s_i32_1 = 1i32;
                                                            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                                                            l48 = s_i32_0;
                                                            continue 'blk_14;
                                                            break 'blk_14;
                                                        }
                                                    }
                                                    s_i32_0 = l43;
                                                    vm.enter_call()?;
                                                    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 13usize, s_i32_0, host);
                                                    vm.exit_call();
                                                    let __t18 = __call_res?;
                                                    s_i32_0 = __t18;
                                                    l51 = s_i32_0;
                                                    s_i32_0 = l44;
                                                    'blk_18: {
                                                        if s_i32_0 != 0 {
                                                            s_i32_0 = l51;
                                                            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
                                                            s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
                                                            s_i64_1 = 0i64;
                                                            s_i32_0 = ops::i64_gt_u(s_i64_0, s_i64_1);
                                                        } else {
                                                            s_i32_0 = 0i32;
                                                        }
                                                    }
                                                    'blk_19: {
                                                        if s_i32_0 != 0 {
                                                            s_i32_0 = l5;
                                                            l52 = s_i32_0;
                                                            s_i32_0 = 1i32;
                                                            l53 = s_i32_0;
                                                            s_i32_0 = 0i32;
                                                            l54 = s_i32_0;
                                                            s_i32_0 = l51;
                                                            l55 = s_i32_0;
                                                            s_i32_0 = 12i32;
                                                            vm.enter_call()?;
                                                            let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
                                                            vm.exit_call();
                                                            let __t19 = __call_res?;
                                                            s_i32_0 = __t19;
                                                            l56 = s_i32_0;
                                                            s_i32_0 = l56;
                                                            s_i32_1 = l52;
                                                            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                                                            s_i32_0 = l56;
                                                            s_i32_1 = l53;
                                                            ops::i32_store8(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                                                            s_i32_0 = l56;
                                                            s_i32_1 = l54;
                                                            ops::i32_store8(vm, 0, s_i32_0, 5u32, s_i32_1)?;
                                                            s_i32_0 = l56;
                                                            s_i32_1 = l55;
                                                            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                                                            s_i32_0 = l56;
                                                            l57 = s_i32_0;
                                                            s_i32_0 = l57;
                                                            return Ok(s_i32_0);
                                                        } else {
                                                        }
                                                    }
                                                } else {
                                                }
                                            }
                                        } else {
                                            s_i32_0 = l5;
                                            s_i32_1 = l26;
                                            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
                                            s_i64_2 = l4;
                                            s_i64_3 = 1i64;
                                            s_i64_2 = ops::i64_add(s_i64_2, s_i64_3);
                                            vm.enter_call()?;
                                            let __call_res = dispatch.call_p_i32_i32_i64_ret_i32(vm, c_map, 11usize, s_i32_0, s_i32_1, s_i64_2, host);
                                            vm.exit_call();
                                            let __t20 = __call_res?;
                                            s_i32_0 = __t20;
                                            l58 = s_i32_0;
                                            s_i32_0 = l58;
                                            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
                                            l5 = s_i32_0;
                                            s_i32_0 = l58;
                                            s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 4u32)?;
                                            'blk_20: {
                                                if s_i32_0 != 0 {
                                                    s_i32_0 = l58;
                                                    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 5u32)?;
                                                    s_i32_1 = 0i32;
                                                    vm.enter_call()?;
                                                    let __call_res = dispatch.call_p_i32_i32_ret_i32(vm, c_map, 8usize, s_i32_0, s_i32_1, host);
                                                    vm.exit_call();
                                                    let __t21 = __call_res?;
                                                    s_i32_0 = __t21;
                                                } else {
                                                    s_i32_0 = 0i32;
                                                }
                                            }
                                            'blk_21: {
                                                if s_i32_0 != 0 {
                                                    s_i32_0 = l58;
                                                    l59 = s_i32_0;
                                                    s_i32_0 = l59;
                                                    return Ok(s_i32_0);
                                                } else {
                                                }
                                            }
                                        }
                                    }
                                } else {
                                }
                            }
                        }
                        s_i32_0 = l25;
                        s_i32_1 = 1i32;
                        s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
                        l25 = s_i32_0;
                        continue 'blk_6;
                        break 'blk_6;
                    }
                }
            }
            s_i32_0 = l9;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l9 = s_i32_0;
            continue 'blk_2;
            break 'blk_2;
        }
    }
    s_i32_0 = l5;
    l60 = s_i32_0;
    s_i32_0 = 0i32;
    l61 = s_i32_0;
    s_i32_0 = 2i32;
    l62 = s_i32_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
    vm.exit_call();
    let __t22 = __call_res?;
    s_i32_0 = __t22;
    l63 = s_i32_0;
    s_i32_0 = l63;
    s_i32_1 = l63;
    s_i32_2 = 16i32;
    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l63;
    s_i32_1 = 0i32;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l63;
    s_i32_1 = 0i32;
    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
    s_i32_0 = l63;
    s_i32_1 = 1i32;
    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
    s_i32_0 = l63;
    l64 = s_i32_0;
    s_i32_0 = 12i32;
    vm.enter_call()?;
    let __call_res = dispatch.call_p_i32_ret_i32(vm, c_map, 3usize, s_i32_0, host);
    vm.exit_call();
    let __t23 = __call_res?;
    s_i32_0 = __t23;
    l65 = s_i32_0;
    s_i32_0 = l65;
    s_i32_1 = l60;
    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l65;
    s_i32_1 = l61;
    ops::i32_store8(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l65;
    s_i32_1 = l62;
    ops::i32_store8(vm, 0, s_i32_0, 5u32, s_i32_1)?;
    s_i32_0 = l65;
    s_i32_1 = l64;
    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
    s_i32_0 = l65;
    l66 = s_i32_0;
    s_i32_0 = l66;
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

const FN_0_GLOBALS: &[usize] = &[0usize, 1usize, 2usize, 3usize, 4usize, 5usize, 6usize, 7usize, 8usize, 9usize, 10usize, 11usize, 12usize, 13usize, 14usize, 15usize, 16usize, 17usize, 18usize, 19usize, 20usize, 21usize, 22usize, 23usize, 24usize, 25usize, 26usize, 27usize, 28usize, 29usize, 30usize, 31usize, 32usize];
const FN_0_CALLEES: &[usize] = &[124usize];
const FN_1_CALLEES: &[usize] = &[0usize, 24usize];
const FN_2_CALLEES: &[usize] = &[0usize, 124usize];
const FN_3_CALLEES: &[usize] = &[0usize, 124usize];
const FN_4_CALLEES: &[usize] = &[0usize, 124usize];
const FN_5_CALLEES: &[usize] = &[0usize, 124usize];
const FN_6_CALLEES: &[usize] = &[0usize, 34usize, 10usize];
const FN_7_GLOBALS: &[usize] = &[14usize];
const FN_7_CALLEES: &[usize] = &[0usize, 120usize];
const FN_8_CALLEES: &[usize] = &[0usize, 0usize];
const FN_9_CALLEES: &[usize] = &[0usize, 10usize];
const FN_10_CALLEES: &[usize] = &[0usize, 121usize];
const FN_11_CALLEES: &[usize] = &[0usize, 121usize];
const FN_12_CALLEES: &[usize] = &[0usize, 25usize];
const FN_13_CALLEES: &[usize] = &[0usize, 120usize];
const FN_14_CALLEES: &[usize] = &[0usize, 124usize];
const FN_15_CALLEES: &[usize] = &[0usize, 124usize];
const FN_16_CALLEES: &[usize] = &[0usize, 124usize];
const FN_17_CALLEES: &[usize] = &[0usize, 13usize];
const FN_18_CALLEES: &[usize] = &[0usize, 13usize];
const FN_19_CALLEES: &[usize] = &[0usize, 13usize];
const FN_20_CALLEES: &[usize] = &[0usize];
const FN_21_CALLEES: &[usize] = &[0usize, 26usize];
const FN_22_CALLEES: &[usize] = &[0usize, 27usize];
const FN_23_CALLEES: &[usize] = &[0usize, 0usize, 0usize, 1usize, 1usize, 124usize];
const FN_24_CALLEES: &[usize] = &[0usize, 120usize];
const FN_25_CALLEES: &[usize] = &[0usize, 121usize];
const FN_26_CALLEES: &[usize] = &[0usize, 26usize];
const FN_27_CALLEES: &[usize] = &[0usize, 32usize, 122usize, 33usize, 2usize];
const FN_28_CALLEES: &[usize] = &[0usize, 33usize, 122usize, 32usize, 2usize];
const FN_29_CALLEES: &[usize] = &[0usize, 28usize];
const FN_30_CALLEES: &[usize] = &[0usize, 29usize];
const FN_31_CALLEES: &[usize] = &[0usize, 30usize];
const FN_32_CALLEES: &[usize] = &[0usize, 26usize];
const FN_33_GLOBALS: &[usize] = &[14usize, 9usize];
const FN_33_CALLEES: &[usize] = &[0usize, 34usize, 10usize, 124usize];
const FN_34_CALLEES: &[usize] = &[0usize, 31usize, 124usize];
const FN_35_GLOBALS: &[usize] = &[14usize, 9usize];
const FN_35_CALLEES: &[usize] = &[0usize, 4usize, 124usize];
const FN_36_GLOBALS: &[usize] = &[14usize, 9usize];
const FN_36_CALLEES: &[usize] = &[0usize, 0usize, 124usize];
const FN_37_CALLEES: &[usize] = &[0usize, 31usize, 124usize];
const FN_38_GLOBALS: &[usize] = &[14usize, 9usize];
const FN_38_CALLEES: &[usize] = &[0usize, 6usize, 35usize, 124usize];
const FN_39_GLOBALS: &[usize] = &[14usize, 9usize];
const FN_39_CALLEES: &[usize] = &[0usize, 124usize, 32usize];
const FN_40_GLOBALS: &[usize] = &[14usize, 9usize];
const FN_40_CALLEES: &[usize] = &[0usize, 124usize, 33usize];
const FN_41_GLOBALS: &[usize] = &[14usize, 9usize];
const FN_41_CALLEES: &[usize] = &[0usize, 10usize, 36usize, 124usize];
const FN_42_GLOBALS: &[usize] = &[9usize];
const FN_42_CALLEES: &[usize] = &[0usize, 8usize, 124usize];
const FN_43_GLOBALS: &[usize] = &[9usize];
const FN_43_CALLEES: &[usize] = &[0usize, 1usize, 124usize];
const FN_44_GLOBALS: &[usize] = &[9usize];
const FN_44_CALLEES: &[usize] = &[0usize, 9usize, 124usize];
const FN_45_GLOBALS: &[usize] = &[9usize];
const FN_45_CALLEES: &[usize] = &[0usize, 124usize];
const FN_46_GLOBALS: &[usize] = &[9usize];
const FN_46_CALLEES: &[usize] = &[0usize, 10usize, 124usize];
const FN_47_GLOBALS: &[usize] = &[14usize, 1usize];
const FN_47_CALLEES: &[usize] = &[0usize, 120usize, 124usize, 13usize, 125usize, 0usize, 32usize, 2usize, 10usize, 36usize, 37usize, 123usize];
const FN_48_CALLEES: &[usize] = &[0usize];
const FN_49_CALLEES: &[usize] = &[0usize, 124usize];
const FN_50_CALLEES: &[usize] = &[0usize, 1usize, 122usize, 0usize, 124usize];
const FN_51_CALLEES: &[usize] = &[0usize, 0usize, 122usize, 1usize, 124usize];
const FN_52_CALLEES: &[usize] = &[0usize, 124usize];
const FN_53_CALLEES: &[usize] = &[0usize, 0usize, 124usize];
const FN_54_CALLEES: &[usize] = &[0usize, 124usize];
const FN_55_CALLEES: &[usize] = &[0usize, 0usize, 122usize];
const FN_56_CALLEES: &[usize] = &[0usize, 1usize, 124usize, 122usize];
const FN_57_GLOBALS: &[usize] = &[14usize];
const FN_57_CALLEES: &[usize] = &[0usize, 21usize, 24usize, 36usize];
const FN_58_CALLEES: &[usize] = &[0usize, 60usize];
const FN_59_CALLEES: &[usize] = &[0usize, 124usize];
const FN_60_CALLEES: &[usize] = &[0usize, 124usize];
const FN_61_CALLEES: &[usize] = &[0usize, 124usize];
const FN_62_CALLEES: &[usize] = &[0usize, 124usize];
const FN_63_CALLEES: &[usize] = &[0usize, 68usize];
const FN_64_GLOBALS: &[usize] = &[19usize, 16usize, 17usize];
const FN_64_CALLEES: &[usize] = &[0usize, 124usize];
const FN_65_GLOBALS: &[usize] = &[19usize];
const FN_65_CALLEES: &[usize] = &[0usize, 124usize, 12usize, 6usize, 35usize, 21usize, 1usize, 12usize];
const FN_66_GLOBALS: &[usize] = &[16usize];
const FN_66_CALLEES: &[usize] = &[0usize, 124usize];
const FN_67_CALLEES: &[usize] = &[0usize, 70usize];
const FN_68_CALLEES: &[usize] = &[0usize, 24usize, 1usize];
const FN_69_CALLEES: &[usize] = &[0usize, 24usize, 2usize];
const FN_70_CALLEES: &[usize] = &[0usize, 61usize];
const FN_71_CALLEES: &[usize] = &[0usize, 124usize];
const FN_72_CALLEES: &[usize] = &[0usize, 124usize];
const FN_73_CALLEES: &[usize] = &[0usize, 2usize];
const FN_74_CALLEES: &[usize] = &[0usize, 124usize];
const FN_75_CALLEES: &[usize] = &[0usize, 124usize];
const FN_76_CALLEES: &[usize] = &[0usize, 124usize];
const FN_77_CALLEES: &[usize] = &[0usize, 124usize];
const FN_78_CALLEES: &[usize] = &[0usize, 51usize];
const FN_79_CALLEES: &[usize] = &[0usize, 51usize];
const FN_80_CALLEES: &[usize] = &[0usize, 124usize];
const FN_81_CALLEES: &[usize] = &[0usize];
const FN_82_CALLEES: &[usize] = &[0usize, 51usize];
const FN_83_CALLEES: &[usize] = &[0usize];
const FN_84_CALLEES: &[usize] = &[0usize, 62usize];
const FN_85_CALLEES: &[usize] = &[0usize, 63usize];
const FN_86_CALLEES: &[usize] = &[0usize, 124usize];
const FN_87_CALLEES: &[usize] = &[0usize, 26usize];
const FN_88_CALLEES: &[usize] = &[0usize, 24usize, 5usize];
const FN_89_CALLEES: &[usize] = &[0usize, 43usize, 67usize];
const FN_90_CALLEES: &[usize] = &[0usize, 64usize];
const FN_91_CALLEES: &[usize] = &[0usize, 65usize];
const FN_92_CALLEES: &[usize] = &[0usize, 66usize];
const FN_93_CALLEES: &[usize] = &[0usize, 1usize];
const FN_94_CALLEES: &[usize] = &[0usize, 67usize];
const FN_95_GLOBALS: &[usize] = &[19usize, 9usize];
const FN_95_CALLEES: &[usize] = &[0usize, 42usize, 68usize, 124usize];
const FN_96_GLOBALS: &[usize] = &[19usize, 1usize, 16usize, 17usize, 9usize];
const FN_96_CALLEES: &[usize] = &[0usize, 124usize, 124usize, 12usize, 31usize, 20usize, 21usize, 33usize];
const FN_97_GLOBALS: &[usize] = &[19usize, 9usize];
const FN_97_CALLEES: &[usize] = &[0usize, 20usize, 124usize];
const FN_98_CALLEES: &[usize] = &[0usize, 67usize, 21usize, 124usize, 68usize, 70usize];
const FN_99_GLOBALS: &[usize] = &[19usize, 9usize];
const FN_99_CALLEES: &[usize] = &[0usize, 44usize, 69usize, 124usize];
const FN_100_GLOBALS: &[usize] = &[16usize, 19usize, 9usize];
const FN_100_CALLEES: &[usize] = &[0usize, 124usize, 124usize, 33usize];
const FN_101_GLOBALS: &[usize] = &[19usize, 9usize];
const FN_101_CALLEES: &[usize] = &[0usize, 124usize, 24usize, 32usize, 20usize];
const FN_102_GLOBALS: &[usize] = &[19usize, 9usize];
const FN_102_CALLEES: &[usize] = &[0usize, 124usize, 0usize, 122usize, 1usize, 2usize];
const FN_103_GLOBALS: &[usize] = &[19usize, 9usize];
const FN_103_CALLEES: &[usize] = &[0usize, 45usize, 70usize, 124usize];
const FN_104_GLOBALS: &[usize] = &[9usize];
const FN_104_CALLEES: &[usize] = &[0usize, 28usize, 124usize, 11usize];
const FN_105_GLOBALS: &[usize] = &[9usize];
const FN_105_CALLEES: &[usize] = &[0usize, 124usize];
const FN_106_GLOBALS: &[usize] = &[14usize, 9usize];
const FN_106_CALLEES: &[usize] = &[0usize, 21usize, 124usize, 24usize];
const FN_107_GLOBALS: &[usize] = &[9usize];
const FN_107_CALLEES: &[usize] = &[0usize, 9usize, 124usize];
const FN_108_GLOBALS: &[usize] = &[9usize];
const FN_108_CALLEES: &[usize] = &[0usize, 124usize];
const FN_109_GLOBALS: &[usize] = &[9usize];
const FN_109_CALLEES: &[usize] = &[0usize, 10usize, 124usize];
const FN_110_GLOBALS: &[usize] = &[19usize, 1usize, 16usize, 17usize];
const FN_110_CALLEES: &[usize] = &[0usize, 124usize, 124usize, 12usize, 31usize, 20usize, 21usize, 33usize];
const FN_111_CALLEES: &[usize] = &[0usize, 24usize, 34usize, 20usize];
const FN_112_CALLEES: &[usize] = &[0usize, 24usize, 35usize, 20usize];
const FN_113_GLOBALS: &[usize] = &[19usize];
const FN_113_CALLEES: &[usize] = &[0usize, 70usize];
const FN_114_CALLEES: &[usize] = &[0usize, 24usize, 36usize, 20usize];
const FN_115_CALLEES: &[usize] = &[0usize, 124usize];
const FN_116_GLOBALS: &[usize] = &[22usize];
const FN_116_CALLEES: &[usize] = &[0usize, 96usize, 85usize, 95usize, 97usize];
const FN_117_CALLEES: &[usize] = &[0usize];
const FN_118_CALLEES: &[usize] = &[0usize, 41usize];
const FN_119_CALLEES: &[usize] = &[0usize];
const FN_120_CALLEES: &[usize] = &[0usize];
const FN_121_CALLEES: &[usize] = &[0usize];
const FN_122_CALLEES: &[usize] = &[0usize, 95usize, 126usize];
const FN_123_CALLEES: &[usize] = &[0usize, 92usize];
const FN_124_CALLEES: &[usize] = &[0usize, 125usize];
const FN_125_GLOBALS: &[usize] = &[22usize];
const FN_125_CALLEES: &[usize] = &[0usize, 9usize, 0usize, 124usize];
const FN_126_GLOBALS: &[usize] = &[22usize];
const FN_126_CALLEES: &[usize] = &[0usize, 124usize];
const FN_127_CALLEES: &[usize] = &[0usize, 36usize];
const FN_128_CALLEES: &[usize] = &[0usize, 9usize, 0usize, 124usize, 85usize];
const FN_129_CALLEES: &[usize] = &[0usize, 12usize, 38usize, 21usize, 85usize];
const FN_130_CALLEES: &[usize] = &[0usize, 4usize, 85usize, 0usize];
const FN_131_CALLEES: &[usize] = &[0usize, 5usize];
const FN_132_CALLEES: &[usize] = &[0usize, 100usize, 85usize];
const FN_133_CALLEES: &[usize] = &[0usize, 76usize];
const FN_134_CALLEES: &[usize] = &[0usize, 76usize];
const FN_135_CALLEES: &[usize] = &[0usize, 1usize, 12usize, 14usize, 1usize, 9usize, 0usize, 0usize, 1usize, 127usize, 85usize];
const FN_136_CALLEES: &[usize] = &[0usize];
const FN_137_CALLEES: &[usize] = &[0usize, 76usize];
const FN_138_CALLEES: &[usize] = &[0usize, 76usize];
const FN_139_GLOBALS: &[usize] = &[22usize];
const FN_139_CALLEES: &[usize] = &[0usize, 50usize, 124usize];
const FN_140_CALLEES: &[usize] = &[0usize, 93usize];
const FN_141_CALLEES: &[usize] = &[0usize, 98usize, 85usize];
const FN_142_CALLEES: &[usize] = &[0usize, 125usize];
const FN_143_GLOBALS: &[usize] = &[22usize];
const FN_143_CALLEES: &[usize] = &[0usize, 41usize, 124usize];
const FN_144_CALLEES: &[usize] = &[0usize, 94usize];
const FN_145_CALLEES: &[usize] = &[0usize, 42usize, 71usize];
const FN_146_CALLEES: &[usize] = &[0usize, 94usize];
const FN_147_GLOBALS: &[usize] = &[22usize, 1usize];
const FN_147_CALLEES: &[usize] = &[0usize, 41usize, 124usize, 100usize, 85usize];
const FN_148_CALLEES: &[usize] = &[0usize, 94usize, 34usize, 71usize, 124usize];
const FN_149_GLOBALS: &[usize] = &[19usize, 9usize, 16usize];
const FN_149_CALLEES: &[usize] = &[0usize, 51usize, 124usize, 1usize, 120usize, 25usize, 26usize];
const FN_150_GLOBALS: &[usize] = &[9usize];
const FN_150_CALLEES: &[usize] = &[0usize, 43usize, 124usize, 27usize];
const FN_151_GLOBALS: &[usize] = &[9usize];
const FN_151_CALLEES: &[usize] = &[0usize, 43usize, 124usize, 28usize];
const FN_152_GLOBALS: &[usize] = &[14usize, 9usize];
const FN_152_CALLEES: &[usize] = &[0usize, 124usize, 51usize];
const FN_153_GLOBALS: &[usize] = &[9usize];
const FN_153_CALLEES: &[usize] = &[0usize, 46usize, 124usize, 9usize];
const FN_154_GLOBALS: &[usize] = &[9usize];
const FN_154_CALLEES: &[usize] = &[0usize, 46usize, 124usize, 10usize];
const FN_155_GLOBALS: &[usize] = &[9usize];
const FN_155_CALLEES: &[usize] = &[0usize, 46usize, 124usize, 11usize];
const FN_156_GLOBALS: &[usize] = &[22usize, 1usize];
const FN_156_CALLEES: &[usize] = &[0usize, 41usize, 124usize, 100usize, 85usize];
const FN_157_CALLEES: &[usize] = &[0usize, 2usize, 15usize, 2usize];
const FN_158_GLOBALS: &[usize] = &[14usize];
const FN_158_CALLEES: &[usize] = &[0usize, 124usize, 15usize, 1usize, 1usize];
const FN_159_CALLEES: &[usize] = &[0usize, 2usize, 15usize];
const FN_160_CALLEES: &[usize] = &[0usize, 1usize, 12usize, 15usize, 9usize, 1usize, 0usize, 127usize];
const FN_161_CALLEES: &[usize] = &[0usize, 12usize, 15usize, 1usize, 2usize, 9usize, 1usize, 127usize, 50usize];
const FN_162_CALLEES: &[usize] = &[0usize, 124usize, 1usize, 16usize, 15usize, 14usize, 0usize, 9usize, 0usize, 0usize, 127usize, 50usize];
const FN_163_CALLEES: &[usize] = &[0usize, 14usize, 2usize, 15usize];
const FN_164_CALLEES: &[usize] = &[0usize, 95usize, 124usize, 15usize, 16usize, 2usize, 1usize, 9usize, 0usize, 127usize, 50usize, 1usize, 122usize, 0usize, 0usize, 127usize, 14usize, 128usize, 99usize];
const FN_165_CALLEES: &[usize] = &[0usize, 58usize];
const FN_166_CALLEES: &[usize] = &[0usize, 1usize, 124usize];
const FN_167_GLOBALS: &[usize] = &[23usize];
const FN_167_CALLEES: &[usize] = &[0usize, 56usize, 2usize, 9usize];
const FN_168_CALLEES: &[usize] = &[0usize, 124usize];
const FN_169_CALLEES: &[usize] = &[0usize];
const FN_170_CALLEES: &[usize] = &[0usize];
const FN_171_CALLEES: &[usize] = &[0usize, 124usize];
const FN_172_CALLEES: &[usize] = &[0usize, 1usize, 124usize];
const FN_173_GLOBALS: &[usize] = &[23usize];
const FN_173_CALLEES: &[usize] = &[0usize, 57usize, 2usize, 9usize];
const FN_174_CALLEES: &[usize] = &[0usize, 1usize];
const FN_175_CALLEES: &[usize] = &[0usize, 124usize];
const FN_176_CALLEES: &[usize] = &[0usize, 0usize, 13usize, 17usize, 129usize, 124usize, 2usize, 130usize, 131usize, 132usize, 1usize, 0usize, 127usize];
const FN_177_CALLEES: &[usize] = &[0usize, 3usize, 124usize, 17usize, 129usize, 1usize, 133usize, 132usize, 134usize, 0usize, 32usize, 2usize, 135usize, 121usize, 13usize, 55usize, 1usize];
const FN_178_GLOBALS: &[usize] = &[14usize];
const FN_178_CALLEES: &[usize] = &[0usize, 1usize, 10usize, 124usize, 32usize, 0usize, 2usize];
const FN_179_CALLEES: &[usize] = &[0usize, 7usize, 10usize, 3usize, 0usize, 0usize, 127usize];
const FN_180_CALLEES: &[usize] = &[0usize, 1usize, 7usize, 0usize, 127usize];
const FN_181_CALLEES: &[usize] = &[0usize, 59usize];
const FN_182_CALLEES: &[usize] = &[0usize, 60usize, 1usize, 122usize, 121usize, 1usize];
const FN_183_CALLEES: &[usize] = &[0usize, 3usize, 124usize, 18usize, 59usize, 1usize];
const FN_184_CALLEES: &[usize] = &[0usize, 18usize, 59usize, 124usize, 1usize, 5usize, 63usize, 7usize, 127usize, 122usize, 128usize, 136usize, 134usize, 3usize];
const FN_185_CALLEES: &[usize] = &[0usize, 1usize, 3usize, 124usize, 18usize, 59usize, 13usize, 127usize, 2usize];
const FN_186_CALLEES: &[usize] = &[0usize, 18usize, 59usize, 124usize, 7usize, 1usize, 127usize, 10usize, 3usize];
const FN_187_CALLEES: &[usize] = &[0usize, 1usize, 124usize, 18usize, 59usize, 56usize, 54usize, 7usize, 127usize, 122usize, 128usize, 3usize];
const FN_188_CALLEES: &[usize] = &[0usize, 18usize, 66usize, 1usize];
const FN_189_CALLEES: &[usize] = &[0usize, 18usize, 59usize, 1usize];
const FN_190_CALLEES: &[usize] = &[0usize, 59usize, 58usize, 124usize];
const FN_191_CALLEES: &[usize] = &[0usize, 1usize, 0usize, 13usize, 58usize, 2usize, 6usize, 4usize, 17usize, 134usize, 121usize, 136usize, 2usize, 127usize];
const FN_192_CALLEES: &[usize] = &[0usize, 0usize, 0usize];
const FN_193_CALLEES: &[usize] = &[0usize, 1usize, 13usize, 58usize, 2usize, 59usize, 127usize];
const FN_194_CALLEES: &[usize] = &[0usize, 1usize, 13usize, 58usize, 2usize, 17usize, 61usize, 59usize, 56usize, 127usize];
const FN_195_CALLEES: &[usize] = &[0usize, 67usize, 58usize, 124usize];
const FN_196_GLOBALS: &[usize] = &[14usize];
const FN_196_CALLEES: &[usize] = &[0usize, 120usize, 58usize, 53usize, 124usize, 13usize, 2usize, 12usize, 9usize, 121usize, 1usize, 33usize, 2usize, 14usize, 127usize];
const FN_197_CALLEES: &[usize] = &[0usize, 0usize, 0usize];
const FN_198_CALLEES: &[usize] = &[0usize, 124usize, 3usize, 137usize, 7usize, 17usize, 5usize, 0usize, 127usize];
const FN_199_CALLEES: &[usize] = &[0usize, 10usize, 3usize, 135usize, 0usize, 122usize, 121usize, 60usize, 1usize, 33usize, 128usize, 136usize];
const FN_200_CALLEES: &[usize] = &[0usize, 17usize, 129usize, 133usize, 132usize, 134usize];
const FN_201_CALLEES: &[usize] = &[0usize, 55usize];
const FN_202_CALLEES: &[usize] = &[0usize, 124usize, 13usize, 18usize, 2usize, 1usize, 127usize, 2usize, 4usize, 138usize];
const FN_203_CALLEES: &[usize] = &[0usize, 124usize, 2usize, 0usize, 1usize, 13usize, 4usize, 18usize, 127usize];
const FN_204_CALLEES: &[usize] = &[0usize, 18usize, 124usize, 3usize];
const FN_205_CALLEES: &[usize] = &[0usize, 4usize, 124usize];
const FN_206_CALLEES: &[usize] = &[0usize, 60usize];
const FN_207_CALLEES: &[usize] = &[0usize, 18usize, 59usize, 1usize, 1usize, 8usize];
const FN_208_CALLEES: &[usize] = &[0usize, 77usize];
const FN_209_CALLEES: &[usize] = &[0usize, 79usize, 13usize, 2usize, 1usize, 2usize, 127usize, 1usize, 4usize, 140usize, 124usize];
const FN_210_CALLEES: &[usize] = &[0usize, 104usize];
const FN_211_CALLEES: &[usize] = &[0usize, 129usize];
const FN_212_GLOBALS: &[usize] = &[3usize];
const FN_212_CALLEES: &[usize] = &[0usize, 64usize, 129usize, 62usize, 124usize];
const FN_213_CALLEES: &[usize] = &[0usize, 13usize, 59usize, 58usize, 2usize, 127usize];
const FN_214_GLOBALS: &[usize] = &[3usize];
const FN_214_CALLEES: &[usize] = &[0usize, 13usize, 18usize, 59usize, 1usize, 124usize, 2usize, 127usize];
const FN_215_CALLEES: &[usize] = &[0usize, 59usize];
const FN_216_CALLEES: &[usize] = &[0usize, 1usize, 18usize, 59usize, 78usize, 2usize, 0usize, 0usize, 1usize, 124usize];
const FN_217_CALLEES: &[usize] = &[0usize, 13usize, 18usize, 59usize, 1usize, 4usize, 2usize, 8usize, 124usize, 127usize];
const FN_218_CALLEES: &[usize] = &[0usize, 18usize, 1usize, 13usize, 4usize, 127usize];
const FN_219_CALLEES: &[usize] = &[0usize, 72usize, 124usize, 12usize, 9usize, 30usize];
const FN_220_CALLEES: &[usize] = &[0usize, 80usize, 124usize];
const FN_221_CALLEES: &[usize] = &[0usize, 124usize, 3usize, 24usize, 71usize];
const FN_222_CALLEES: &[usize] = &[0usize, 46usize, 71usize, 124usize];
const FN_223_CALLEES: &[usize] = &[0usize, 124usize, 3usize, 24usize, 71usize];
const FN_224_CALLEES: &[usize] = &[0usize, 124usize, 3usize, 71usize];
const FN_225_CALLEES: &[usize] = &[0usize, 72usize, 124usize, 12usize];
const FN_226_CALLEES: &[usize] = &[0usize, 80usize, 124usize];
const FN_227_CALLEES: &[usize] = &[0usize, 80usize, 124usize];
const FN_228_CALLEES: &[usize] = &[0usize, 80usize, 124usize];
const FN_229_CALLEES: &[usize] = &[0usize];
const FN_230_CALLEES: &[usize] = &[0usize, 124usize];
const FN_231_CALLEES: &[usize] = &[0usize];
const FN_232_CALLEES: &[usize] = &[0usize, 62usize, 124usize];
const FN_233_CALLEES: &[usize] = &[0usize, 64usize];
const FN_234_CALLEES: &[usize] = &[0usize, 80usize, 124usize, 101usize, 141usize, 3usize, 100usize, 76usize];
const FN_235_CALLEES: &[usize] = &[0usize, 77usize, 124usize, 102usize];
const FN_236_CALLEES: &[usize] = &[0usize, 124usize, 3usize, 100usize, 141usize, 76usize];
const FN_237_CALLEES: &[usize] = &[0usize, 77usize, 124usize, 103usize];
const FN_238_CALLEES: &[usize] = &[0usize, 124usize, 7usize, 72usize, 13usize, 11usize, 30usize, 27usize, 24usize];
const FN_239_CALLEES: &[usize] = &[0usize, 124usize, 7usize, 72usize, 10usize, 12usize, 30usize, 9usize, 0usize, 9usize];
const FN_240_CALLEES: &[usize] = &[0usize, 13usize, 124usize, 18usize, 71usize, 2usize, 127usize];
const FN_241_CALLEES: &[usize] = &[0usize, 124usize, 120usize, 71usize, 13usize, 11usize, 26usize, 24usize, 2usize, 25usize, 127usize, 99usize];
const FN_242_CALLEES: &[usize] = &[0usize, 124usize, 71usize, 13usize, 3usize, 2usize, 127usize, 99usize];
const FN_243_CALLEES: &[usize] = &[0usize, 7usize, 124usize, 72usize, 120usize, 9usize, 11usize, 9usize];
const FN_244_CALLEES: &[usize] = &[0usize, 7usize, 124usize, 72usize, 120usize, 9usize, 11usize, 24usize, 71usize, 10usize, 3usize];
const FN_245_CALLEES: &[usize] = &[0usize];
const FN_246_CALLEES: &[usize] = &[0usize, 130usize, 131usize, 133usize];
const FN_247_CALLEES: &[usize] = &[0usize, 1usize, 132usize];
const FN_248_CALLEES: &[usize] = &[0usize, 13usize, 72usize, 2usize, 19usize, 124usize, 71usize, 0usize, 120usize, 2usize, 111usize, 108usize, 113usize, 133usize, 0usize, 115usize, 139usize, 127usize];
const FN_249_CALLEES: &[usize] = &[0usize, 18usize, 124usize];
const FN_250_CALLEES: &[usize] = &[0usize, 139usize, 124usize, 72usize, 13usize, 2usize, 19usize, 71usize, 0usize, 120usize, 2usize, 111usize, 108usize, 113usize, 133usize, 0usize, 115usize];
const FN_251_CALLEES: &[usize] = &[0usize, 124usize, 0usize, 139usize];
const FN_252_GLOBALS: &[usize] = &[26usize];
const FN_252_CALLEES: &[usize] = &[0usize, 124usize, 13usize, 2usize, 127usize];
const FN_253_CALLEES: &[usize] = &[0usize, 110usize, 13usize, 124usize, 2usize, 139usize, 127usize];
const FN_254_CALLEES: &[usize] = &[0usize, 124usize, 71usize, 0usize];
const FN_255_CALLEES: &[usize] = &[0usize, 0usize, 124usize];
const FN_256_CALLEES: &[usize] = &[0usize, 124usize, 0usize, 139usize];
const FN_257_GLOBALS: &[usize] = &[24usize];
const FN_257_CALLEES: &[usize] = &[0usize, 59usize, 54usize, 72usize, 108usize];
const FN_258_GLOBALS: &[usize] = &[27usize, 28usize];
const FN_258_CALLEES: &[usize] = &[0usize, 10usize, 12usize];
const FN_259_CALLEES: &[usize] = &[0usize, 120usize];
const FN_260_CALLEES: &[usize] = &[0usize, 124usize, 0usize, 139usize, 105usize, 72usize, 108usize];
const FN_261_CALLEES: &[usize] = &[0usize, 124usize, 0usize, 13usize, 71usize, 2usize, 127usize];
const FN_262_CALLEES: &[usize] = &[0usize, 114usize, 132usize, 124usize, 4usize];
const FN_263_CALLEES: &[usize] = &[0usize, 13usize, 124usize, 2usize, 71usize, 139usize, 127usize, 117usize, 0usize, 3usize, 105usize];
const FN_264_CALLEES: &[usize] = &[0usize, 124usize, 13usize, 2usize, 4usize, 127usize, 117usize, 0usize, 139usize, 105usize, 72usize];
const FN_265_CALLEES: &[usize] = &[0usize, 13usize, 124usize, 2usize, 3usize, 4usize, 127usize, 117usize, 0usize, 139usize, 105usize, 72usize, 13usize];
const FN_266_GLOBALS: &[usize] = &[25usize];
const FN_266_CALLEES: &[usize] = &[0usize, 0usize, 59usize];
const FN_267_CALLEES: &[usize] = &[0usize, 2usize, 4usize, 124usize];
const FN_268_CALLEES: &[usize] = &[0usize, 17usize, 124usize];
const FN_269_CALLEES: &[usize] = &[0usize, 13usize, 85usize, 2usize, 127usize];
const FN_270_CALLEES: &[usize] = &[0usize, 112usize, 13usize, 4usize, 2usize, 127usize];
const FN_271_CALLEES: &[usize] = &[0usize, 124usize, 105usize, 0usize, 139usize, 3usize, 13usize, 2usize, 127usize];
const FN_272_GLOBALS: &[usize] = &[25usize];
const FN_272_CALLEES: &[usize] = &[0usize, 0usize, 59usize];
const FN_273_GLOBALS: &[usize] = &[24usize, 29usize];
const FN_273_CALLEES: &[usize] = &[0usize, 59usize, 54usize, 72usize, 10usize];
const FN_274_GLOBALS: &[usize] = &[25usize];
const FN_274_CALLEES: &[usize] = &[0usize, 0usize, 59usize];
const FN_275_CALLEES: &[usize] = &[0usize, 13usize, 4usize, 124usize, 139usize, 2usize, 127usize];
const FN_276_CALLEES: &[usize] = &[0usize, 124usize, 0usize, 139usize];
const FN_277_CALLEES: &[usize] = &[0usize, 18usize, 124usize, 110usize];
const FN_278_CALLEES: &[usize] = &[0usize, 18usize, 124usize, 13usize, 4usize, 2usize, 127usize, 1usize];
const FN_279_CALLEES: &[usize] = &[0usize, 3usize, 2usize, 124usize, 64usize, 74usize, 142usize, 105usize, 62usize, 106usize, 0usize];
const FN_280_GLOBALS: &[usize] = &[24usize, 25usize];
const FN_280_CALLEES: &[usize] = &[0usize, 115usize, 124usize, 13usize, 0usize, 59usize, 2usize, 107usize, 72usize, 112usize, 114usize, 133usize, 114usize, 111usize, 116usize, 117usize, 116usize, 12usize, 4usize, 127usize, 54usize, 104usize, 112usize];
const FN_281_GLOBALS: &[usize] = &[25usize];
const FN_281_CALLEES: &[usize] = &[0usize, 0usize, 59usize, 113usize];
const FN_282_CALLEES: &[usize] = &[0usize, 72usize, 0usize];
const FN_283_GLOBALS: &[usize] = &[27usize, 28usize];
const FN_283_CALLEES: &[usize] = &[0usize, 72usize, 10usize, 12usize];
const FN_284_GLOBALS: &[usize] = &[25usize];
const FN_284_CALLEES: &[usize] = &[0usize, 0usize, 59usize];
const FN_285_GLOBALS: &[usize] = &[25usize];
const FN_285_CALLEES: &[usize] = &[0usize, 0usize, 59usize];
const FN_286_CALLEES: &[usize] = &[0usize, 92usize, 122usize, 90usize];
const FN_287_GLOBALS: &[usize] = &[30usize, 31usize, 32usize];
const FN_287_CALLEES: &[usize] = &[0usize, 11usize, 10usize, 12usize];
const FN_288_CALLEES: &[usize] = &[0usize, 124usize, 13usize, 2usize, 0usize, 127usize];
const FN_289_CALLEES: &[usize] = &[0usize, 0usize, 18usize, 124usize, 120usize];
const FN_290_CALLEES: &[usize] = &[0usize, 0usize, 18usize, 124usize, 120usize, 13usize, 4usize, 2usize, 127usize, 1usize];
const FN_291_CALLEES: &[usize] = &[0usize, 124usize, 0usize, 59usize];
const FN_292_CALLEES: &[usize] = &[0usize, 7usize, 72usize, 124usize, 11usize, 11usize, 3usize, 127usize];
const FN_293_CALLEES: &[usize] = &[0usize, 71usize, 124usize, 2usize, 72usize, 108usize, 113usize, 133usize, 121usize, 108usize];
const FN_294_CALLEES: &[usize] = &[0usize, 124usize, 0usize, 72usize, 108usize, 5usize];
const FN_295_CALLEES: &[usize] = &[0usize, 124usize, 1usize, 6usize];
const FN_296_CALLEES: &[usize] = &[0usize, 71usize, 2usize, 124usize, 111usize, 0usize, 133usize, 72usize, 108usize, 113usize, 120usize, 13usize, 4usize, 127usize];
const FN_297_GLOBALS: &[usize] = &[30usize, 31usize, 32usize];
const FN_297_CALLEES: &[usize] = &[0usize, 72usize, 12usize, 10usize, 124usize, 118usize, 109usize, 119usize, 13usize, 71usize, 1usize, 0usize, 112usize, 119usize];
const FN_298_GLOBALS: &[usize] = &[30usize];
const FN_298_CALLEES: &[usize] = &[0usize, 1usize, 2usize, 124usize, 0usize, 139usize, 72usize, 11usize, 1usize, 13usize, 2usize, 4usize, 4usize, 127usize, 11usize, 59usize, 0usize, 110usize, 133usize, 6usize];
const FN_299_CALLEES: &[usize] = &[0usize, 93usize, 118usize, 94usize, 124usize];
const FN_300_CALLEES: &[usize] = &[124usize];
const FN_321_CALLEES: &[usize] = &[126usize];
const FN_323_CALLEES: &[usize] = &[124usize];
const FN_326_CALLEES: &[usize] = &[124usize];
const FN_330_CALLEES: &[usize] = &[124usize];
const FN_331_CALLEES: &[usize] = &[124usize];
const FN_334_CALLEES: &[usize] = &[124usize];
const FN_337_CALLEES: &[usize] = &[124usize];
const FN_344_CALLEES: &[usize] = &[124usize];
const FN_348_CALLEES: &[usize] = &[124usize];
const FN_349_CALLEES: &[usize] = &[124usize];
const FN_351_CALLEES: &[usize] = &[124usize];
const FN_352_CALLEES: &[usize] = &[124usize];
const FN_353_CALLEES: &[usize] = &[139usize];
const FN_355_CALLEES: &[usize] = &[138usize];
const FN_356_CALLEES: &[usize] = &[124usize];
const FN_358_CALLEES: &[usize] = &[124usize];
const FN_362_CALLEES: &[usize] = &[0usize, 124usize, 123usize];
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
    crate::Bound_p_i32_i32_i32_i32_i32_i32_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_174dfb33e5ba, globals: &[], callees: FN_265_CALLEES },
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
static BOUND_SIG_P_I32_I32_I32_I32_I64_RET_I32: &[crate::Bound_p_i32_i32_i32_i32_i64_ret_i32] = &[
    crate::Bound_p_i32_i32_i32_i32_i64_ret_i32 { func: crate::c_85900_04292::ChunkFns::f_cc184be07b93, globals: &[], callees: FN_296_CALLEES },
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
    crate::Bound_p_i32_i32_i32_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_186d1c0e26c0, globals: &[], callees: FN_255_CALLEES },
    crate::Bound_p_i32_i32_i32_i32_ret_i32 { func: crate::c_85900_04292::ChunkFns::f_37bc21062338, globals: &[], callees: FN_290_CALLEES },
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
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_0ba96dccaf47, globals: FN_196_GLOBALS, callees: FN_196_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_52b047a3c674, globals: &[], callees: FN_250_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_e8fc0c2f6c4c, globals: &[], callees: FN_261_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_20ed2f71bb81, globals: &[], callees: FN_275_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_99792::ChunkFns::f_8cad486356a1, globals: FN_280_GLOBALS, callees: FN_280_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85900_04292::ChunkFns::f_b4a27025f49f, globals: &[], callees: FN_293_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85900_04292::ChunkFns::f_be76f48db77a, globals: &[], callees: FN_294_CALLEES },
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
static BOUND_SIG_P_I32_I32_I64_RET_I32: &[crate::Bound_p_i32_i32_i64_ret_i32] = &[
    crate::Bound_p_i32_i32_i64_ret_i32 { func: crate::c_85900_04292::ChunkFns::f_f2aa3d0555d6, globals: FN_297_GLOBALS, callees: FN_297_CALLEES },
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
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_1_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_27aaa3cb9114, globals: &[], callees: FN_2_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f2340190950c, globals: &[], callees: FN_4_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_0ece3b1734d5, globals: &[], callees: FN_5_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_803fb230173c, globals: &[], callees: FN_6_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_8ed247cfa309, globals: FN_7_GLOBALS, callees: FN_7_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_a9acae168912, globals: &[], callees: FN_8_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_dcce91d2e133, globals: &[], callees: FN_9_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_12_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_1e6ff4c83b62, globals: &[], callees: FN_13_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_da0e89bc9783, globals: &[], callees: FN_17_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_6df8689f059b, globals: &[], callees: FN_18_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f9ce687d90ae, globals: &[], callees: FN_19_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_30ec60d4302a, globals: &[], callees: FN_20_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_21_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_22_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_da0e89bc9783, globals: &[], callees: FN_24_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_26_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7e58af41e27d, globals: &[], callees: FN_27_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7e58af41e27d, globals: &[], callees: FN_28_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_29_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_30_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_31_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_32_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_ace0563243a6, globals: FN_33_GLOBALS, callees: FN_33_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_c69686cacf82, globals: &[], callees: FN_34_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_1a5f3f7571e2, globals: &[], callees: FN_37_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f1c0a657afaa, globals: FN_38_GLOBALS, callees: FN_38_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_acb2a186ecef, globals: FN_39_GLOBALS, callees: FN_39_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_acb2a186ecef, globals: FN_40_GLOBALS, callees: FN_40_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f1c0a657afaa, globals: FN_41_GLOBALS, callees: FN_41_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_5cfd1c966ee0, globals: FN_47_GLOBALS, callees: FN_47_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_c1a365c57784, globals: &[], callees: FN_50_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f15f52fd10ac, globals: &[], callees: FN_51_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_b9ec3478b1be, globals: &[], callees: FN_52_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f5ab0dd84882, globals: &[], callees: FN_53_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_a07d59a0459c, globals: &[], callees: FN_54_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_998c31a576d2, globals: &[], callees: FN_56_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_58_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_27aaa3cb9114, globals: &[], callees: FN_59_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f2340190950c, globals: &[], callees: FN_61_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_0ece3b1734d5, globals: &[], callees: FN_62_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f8850b1df958, globals: &[], callees: FN_63_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_609b70a572c5, globals: FN_64_GLOBALS, callees: FN_64_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f148b5f5c14f, globals: FN_65_GLOBALS, callees: FN_65_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_e23aae24b862, globals: &[], callees: FN_67_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_70_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_1e6ff4c83b62, globals: &[], callees: FN_71_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_da0e89bc9783, globals: &[], callees: FN_78_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_6df8689f059b, globals: &[], callees: FN_79_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f9ce687d90ae, globals: &[], callees: FN_82_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7ec8bf846782, globals: &[], callees: FN_83_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_84_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_85_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_da0e89bc9783, globals: &[], callees: FN_86_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_293ae2a09ea3, globals: &[], callees: FN_89_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_90_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_91_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_92_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_94_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f1c0a657afaa, globals: FN_95_GLOBALS, callees: FN_95_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_28a9f52b5bce, globals: FN_96_GLOBALS, callees: FN_96_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_4cf93470be08, globals: &[], callees: FN_98_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f1c0a657afaa, globals: FN_99_GLOBALS, callees: FN_99_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_1acc39445a44, globals: FN_101_GLOBALS, callees: FN_101_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_5230ac976967, globals: FN_102_GLOBALS, callees: FN_102_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f1c0a657afaa, globals: FN_103_GLOBALS, callees: FN_103_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_3c62ec56fda8, globals: FN_110_GLOBALS, callees: FN_110_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f90a3e72a759, globals: &[], callees: FN_111_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f90a3e72a759, globals: &[], callees: FN_112_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f90a3e72a759, globals: &[], callees: FN_114_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_40871f360722, globals: FN_116_GLOBALS, callees: FN_116_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_84192::ChunkFns::f_4f573e6f88cb, globals: &[], callees: FN_117_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_67329667d7fd, globals: &[], callees: FN_118_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_84192::ChunkFns::f_4f573e6f88cb, globals: &[], callees: FN_119_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_84192::ChunkFns::f_4f573e6f88cb, globals: &[], callees: FN_121_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_ca8d871e077c, globals: &[], callees: FN_122_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_123_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_1e6ff4c83b62, globals: &[], callees: FN_124_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_3594571ef794, globals: &[], callees: FN_132_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_e02bb7a5ce3b, globals: &[], callees: FN_133_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_868a5fcb144d, globals: &[], callees: FN_134_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_94c31aaa12fa, globals: &[], callees: FN_135_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_c56c009f41de, globals: &[], callees: FN_137_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_cfe54f4f1606, globals: &[], callees: FN_138_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_da603e0969cb, globals: FN_139_GLOBALS, callees: FN_139_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_140_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_0364226cb1e8, globals: &[], callees: FN_141_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_da0e89bc9783, globals: &[], callees: FN_142_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_144_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_194f062f76fc, globals: &[], callees: FN_145_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_146_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_4a4ae773c3d4, globals: FN_147_GLOBALS, callees: FN_147_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_1592393fe606, globals: &[], callees: FN_148_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_04c43103713a, globals: FN_156_GLOBALS, callees: FN_156_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_2af4fe9e7d26, globals: &[], callees: FN_159_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_8a603a474721, globals: &[], callees: FN_160_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_08e07943209e, globals: &[], callees: FN_161_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_188a11564f14, globals: &[], callees: FN_162_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_d3a85f461ba8, globals: &[], callees: FN_163_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_a70135e4cf19, globals: &[], callees: FN_164_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_b238ded6c3fe, globals: &[], callees: FN_193_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_fbaa521b0850, globals: &[], callees: FN_195_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_f008f64e784a, globals: &[], callees: FN_202_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_9b9e4d79c2a8, globals: &[], callees: FN_203_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_59a0410eeb26, globals: &[], callees: FN_209_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_dcce91d2e133, globals: &[], callees: FN_210_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_17e4d117a9e2, globals: &[], callees: FN_246_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_c83374c783a1, globals: &[], callees: FN_247_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_cb52f1351618, globals: &[], callees: FN_248_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_f9611536c652, globals: &[], callees: FN_251_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_b732433177f0, globals: &[], callees: FN_254_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_0f8beff50967, globals: &[], callees: FN_259_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_426d859a938f, globals: &[], callees: FN_262_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_2d42b20c8b6d, globals: &[], callees: FN_263_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_bbee40c093b4, globals: &[], callees: FN_264_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_027e6ffc1a1a, globals: &[], callees: FN_270_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_92e33a06770e, globals: &[], callees: FN_279_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85900_04292::ChunkFns::f_bd36f6dae8b0, globals: &[], callees: FN_289_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85900_04292::ChunkFns::f_eeae7770e899, globals: &[], callees: FN_295_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_2::ChunkFns::f_03e481579af4, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_1::ChunkFns::f_c6e97ee70046, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_5015758bb709, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_3948a84700b8, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_2::ChunkFns::f_03e481579af4, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_2::ChunkFns::f_162b60a1a7e7, globals: &[], callees: FN_321_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_11c79f5620b0, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_ea4faf22326d, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_a43a5ae7b21f, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_2c680f88091b, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_140fb33a5afe, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_32917fc773d9, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_7d1f4d7fcad4, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_fa57b0c2b6ab, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_a74c85b451e0, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_3f75c31e0aa0, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_aba70ab0b976, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_deffe42432b7, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_f96a9e02e90e, globals: &[], callees: FN_353_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_2::ChunkFns::f_03e481579af4, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_f96a9e02e90e, globals: &[], callees: FN_355_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_ec7fee50bb79, globals: &[], callees: FN_358_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_5591fa286389, globals: &[], callees: &[] },
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
static BOUND_SIG_P_I32_I32_RET_I64: &[crate::Bound_p_i32_i32_ret_i64] = &[
    crate::Bound_p_i32_i32_ret_i64 { func: crate::c_85899_88792::ChunkFns::f_3abc88984867, globals: &[], callees: FN_55_CALLEES },
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
    crate::Bound_p_i32_i32_ret_void { func: crate::c_85899_88792::ChunkFns::f_c55926dd782d, globals: &[], callees: FN_199_CALLEES },
    crate::Bound_p_i32_i32_ret_void { func: crate::c_85899_83092_2::ChunkFns::f_71d8a2d6235d, globals: &[], callees: FN_326_CALLEES },
    crate::Bound_p_i32_i32_ret_void { func: crate::c_85899_83092_2::ChunkFns::f_71d8a2d6235d, globals: &[], callees: FN_337_CALLEES },
    crate::Bound_p_i32_i32_ret_void { func: crate::c_85899_83092_2::ChunkFns::f_e79c4e116bfd, globals: &[], callees: FN_344_CALLEES },
    crate::Bound_p_i32_i32_ret_void { func: crate::c_85899_83092_2::ChunkFns::f_bc6c808270d1, globals: &[], callees: FN_348_CALLEES },
    crate::Bound_p_i32_i32_ret_void { func: crate::c_85899_88792::ChunkFns::f_8fa562cbc853, globals: &[], callees: FN_351_CALLEES },
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
static BOUND_SIG_P_I32_I64_I32_I32_RET_I32: &[crate::Bound_p_i32_i64_i32_i32_ret_i32] = &[
    crate::Bound_p_i32_i64_i32_i32_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_c3c4818e5a41, globals: &[], callees: FN_278_CALLEES },
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
static BOUND_SIG_P_I32_I64_I32_RET_I32: &[crate::Bound_p_i32_i64_i32_ret_i32] = &[
    crate::Bound_p_i32_i64_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_b953838600c1, globals: &[], callees: FN_249_CALLEES },
    crate::Bound_p_i32_i64_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_6cb216adc685, globals: &[], callees: FN_268_CALLEES },
    crate::Bound_p_i32_i64_i32_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_c7ad525e5a02, globals: &[], callees: FN_277_CALLEES },
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
static BOUND_SIG_P_I32_I64_I32_RET_VOID: &[crate::Bound_p_i32_i64_i32_ret_void] = &[
    crate::Bound_p_i32_i64_i32_ret_void { func: crate::c_85899_83092_2::ChunkFns::f_0a0fecb79fc7, globals: &[], callees: &[] },
    crate::Bound_p_i32_i64_i32_ret_void { func: crate::c_85899_83092_2::ChunkFns::f_0a0fecb79fc7, globals: &[], callees: &[] },
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
    crate::Bound_p_i32_i64_i64_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_b6cf541461dc, globals: &[], callees: FN_205_CALLEES },
    crate::Bound_p_i32_i64_i64_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_a4773ed743de, globals: &[], callees: FN_218_CALLEES },
    crate::Bound_p_i32_i64_i64_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_9adecba56606, globals: &[], callees: FN_330_CALLEES },
    crate::Bound_p_i32_i64_i64_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_9607d677c9db, globals: &[], callees: FN_349_CALLEES },
    crate::Bound_p_i32_i64_i64_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_9adecba56606, globals: &[], callees: FN_352_CALLEES },
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
static BOUND_SIG_P_I32_I64_I64_RET_I64: &[crate::Bound_p_i32_i64_i64_ret_i64] = &[
    crate::Bound_p_i32_i64_i64_ret_i64 { func: crate::c_85899_88792::ChunkFns::f_31af32cb558c, globals: &[], callees: FN_200_CALLEES },
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
static BOUND_SIG_P_I32_I64_RET_I32: &[crate::Bound_p_i32_i64_ret_i32] = &[
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f0da2366c908, globals: &[], callees: FN_177_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_88792_2::ChunkFns::f_7503dfb81265, globals: &[], callees: FN_183_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_deaf01e451f9, globals: &[], callees: FN_184_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_88792_2::ChunkFns::f_ea9ae49cf4e8, globals: &[], callees: FN_185_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_88792_2::ChunkFns::f_a73ab5eebd75, globals: &[], callees: FN_186_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_b96282108807, globals: &[], callees: FN_187_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_419c292e1d24, globals: &[], callees: FN_197_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_a996482bc1ef, globals: &[], callees: FN_204_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_3de4de76d7bc, globals: &[], callees: FN_216_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_7a27ed8e21d5, globals: &[], callees: FN_238_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_ee91944eafa8, globals: &[], callees: FN_239_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_f2d9b324c003, globals: &[], callees: FN_260_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_94f57c3a085c, globals: &[], callees: FN_271_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_5ec036ffae9b, globals: &[], callees: FN_276_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_a4027a2778a0, globals: &[], callees: &[] },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_a4027a2778a0, globals: &[], callees: &[] },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_e849c87b3008, globals: &[], callees: FN_331_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_87279bd968a2, globals: &[], callees: &[] },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_a4027a2778a0, globals: &[], callees: &[] },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_a4027a2778a0, globals: &[], callees: &[] },
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
static BOUND_SIG_P_I32_I64_RET_I64: &[crate::Bound_p_i32_i64_ret_i64] = &[
    crate::Bound_p_i32_i64_ret_i64 { func: crate::c_85899_88792::ChunkFns::f_2bae3ae8abdb, globals: &[], callees: FN_188_CALLEES },
    crate::Bound_p_i32_i64_ret_i64 { func: crate::c_85899_88792::ChunkFns::f_241fa345f31b, globals: &[], callees: FN_189_CALLEES },
    crate::Bound_p_i32_i64_ret_i64 { func: crate::c_85899_92792_3::ChunkFns::f_112be49a4c6a, globals: FN_266_GLOBALS, callees: FN_266_CALLEES },
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
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_1d111f94255c, globals: &[], callees: FN_3_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7eb29a59d915, globals: &[], callees: FN_10_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_fab6c7792bec, globals: &[], callees: FN_11_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_a45a26c14c17, globals: &[], callees: FN_15_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_8f0ec661a804, globals: &[], callees: FN_16_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_3979fdeb44f5, globals: &[], callees: FN_25_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_de050b4b81f1, globals: FN_35_GLOBALS, callees: FN_35_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_327504b23599, globals: FN_42_GLOBALS, callees: FN_42_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_703887b4ca0f, globals: FN_43_GLOBALS, callees: FN_43_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_671a96cacef8, globals: FN_44_GLOBALS, callees: FN_44_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f1674ef8dccc, globals: FN_45_GLOBALS, callees: FN_45_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_e75c24ce5706, globals: FN_46_GLOBALS, callees: FN_46_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7d101c2d913a, globals: FN_57_GLOBALS, callees: FN_57_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_1d111f94255c, globals: &[], callees: FN_60_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_c65ede4802f8, globals: FN_66_GLOBALS, callees: FN_66_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_005520f212f5, globals: &[], callees: FN_68_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_005520f212f5, globals: &[], callees: FN_69_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_1e881702eb44, globals: &[], callees: FN_73_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_a45a26c14c17, globals: &[], callees: FN_75_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_8f0ec661a804, globals: &[], callees: FN_76_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_83092::ChunkFns::f_0e5bb44a1e78, globals: &[], callees: FN_77_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_0cbcbc1ef76e, globals: &[], callees: FN_81_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_9d92c7701c67, globals: &[], callees: FN_87_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_005520f212f5, globals: &[], callees: FN_88_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_49de3a4260ff, globals: &[], callees: FN_93_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_e32d590b954e, globals: FN_97_GLOBALS, callees: FN_97_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_329f5ce01511, globals: FN_100_GLOBALS, callees: FN_100_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_dbe6bf1cc46b, globals: FN_104_GLOBALS, callees: FN_104_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_0fabceb05d99, globals: FN_105_GLOBALS, callees: FN_105_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_b6a8cfd6ecb8, globals: FN_106_GLOBALS, callees: FN_106_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_671a96cacef8, globals: FN_107_GLOBALS, callees: FN_107_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f1674ef8dccc, globals: FN_108_GLOBALS, callees: FN_108_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_e75c24ce5706, globals: FN_109_GLOBALS, callees: FN_109_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_b828c3bb90f0, globals: FN_113_GLOBALS, callees: FN_113_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_83092::ChunkFns::f_70644cf2f7cf, globals: &[], callees: FN_115_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_84192::ChunkFns::f_7931de08c6d9, globals: &[], callees: FN_120_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_64d7099dbc77, globals: FN_126_GLOBALS, callees: FN_126_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_358231fcf579, globals: &[], callees: FN_127_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_d421c25ba8cd, globals: &[], callees: FN_128_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_22fbbfe4115b, globals: &[], callees: FN_129_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_1e881702eb44, globals: &[], callees: FN_131_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_f1d61f915b91, globals: &[], callees: FN_136_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_f6699ee6758d, globals: FN_143_GLOBALS, callees: FN_143_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_56b1ab258317, globals: FN_149_GLOBALS, callees: FN_149_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_e8394947df8a, globals: FN_150_GLOBALS, callees: FN_150_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_a01f25005cd0, globals: FN_151_GLOBALS, callees: FN_151_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_ac9ed96b8296, globals: FN_152_GLOBALS, callees: FN_152_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_e8394947df8a, globals: FN_153_GLOBALS, callees: FN_153_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_a01f25005cd0, globals: FN_154_GLOBALS, callees: FN_154_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_3f199480863a, globals: FN_155_GLOBALS, callees: FN_155_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_4374122c356c, globals: &[], callees: FN_157_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_93af815d1803, globals: FN_158_GLOBALS, callees: FN_158_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_2a1df693f599, globals: &[], callees: FN_165_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_428301bbe702, globals: &[], callees: FN_166_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_16b21026a7f2, globals: FN_167_GLOBALS, callees: FN_167_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_83092::ChunkFns::f_0e5bb44a1e78, globals: &[], callees: FN_168_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_ffdee7c93cbe, globals: &[], callees: FN_169_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_3eb654e927a1, globals: &[], callees: FN_170_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_3927290801c7, globals: &[], callees: FN_172_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_d89853f7c7c5, globals: FN_173_GLOBALS, callees: FN_173_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_49de3a4260ff, globals: &[], callees: FN_174_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_a45a26c14c17, globals: &[], callees: FN_175_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_5e048dbe7c1f, globals: &[], callees: FN_176_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f26f1b9b1ce2, globals: FN_178_GLOBALS, callees: FN_178_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_a4b8b05c5fad, globals: &[], callees: FN_179_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_45817b7043dc, globals: &[], callees: FN_180_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_fdb0906f56bb, globals: &[], callees: FN_181_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_bdc4fb2aaab1, globals: &[], callees: FN_190_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_990600c4a836, globals: &[], callees: FN_191_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_34a49f345dca, globals: &[], callees: FN_194_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_2::ChunkFns::f_638e3fd61a80, globals: &[], callees: FN_198_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_2::ChunkFns::f_61a16d9614ba, globals: &[], callees: FN_201_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_2::ChunkFns::f_61a16d9614ba, globals: &[], callees: FN_206_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_9d92c7701c67, globals: &[], callees: FN_208_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_1::ChunkFns::f_db0ffa8054bf, globals: &[], callees: FN_211_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_cd58da62b9e3, globals: FN_212_GLOBALS, callees: FN_212_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_a2186d19e2c0, globals: &[], callees: FN_213_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_aa32df195e2f, globals: FN_214_GLOBALS, callees: FN_214_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_5e4fbe80a2d9, globals: &[], callees: FN_215_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_e4afd653eea2, globals: &[], callees: FN_217_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_03c6e9c99b64, globals: &[], callees: FN_219_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_038735e9e9c7, globals: &[], callees: FN_220_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_7425067462bd, globals: &[], callees: FN_221_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_a4564f09a357, globals: &[], callees: FN_222_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_975343290a87, globals: &[], callees: FN_223_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_2e9ae6967fad, globals: &[], callees: FN_224_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_fec92329e980, globals: &[], callees: FN_225_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_87a4332dcdea, globals: &[], callees: FN_226_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_d725ff05410f, globals: &[], callees: FN_227_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_3505cdd4951f, globals: &[], callees: FN_228_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792::ChunkFns::f_7c75ee00bddb, globals: &[], callees: FN_229_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792::ChunkFns::f_49eed9e9e18f, globals: &[], callees: FN_230_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792::ChunkFns::f_7c75ee00bddb, globals: &[], callees: FN_231_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_97d4a47a22ec, globals: &[], callees: FN_232_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_2::ChunkFns::f_61a16d9614ba, globals: &[], callees: FN_233_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_9b4a1c598462, globals: &[], callees: FN_234_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_e3f6ab9ba400, globals: &[], callees: FN_235_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_a2a7ff38a2b7, globals: &[], callees: FN_236_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_d9ea1df92c5e, globals: &[], callees: FN_237_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_9c15bf600887, globals: &[], callees: FN_240_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_47fe2304968c, globals: &[], callees: FN_241_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_94d168266957, globals: &[], callees: FN_242_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_093751662340, globals: &[], callees: FN_243_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_e59a8e4dd5a8, globals: &[], callees: FN_244_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_6da45e6623e3, globals: &[], callees: FN_245_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_d53d5875197a, globals: &[], callees: FN_253_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_dd64d7da387f, globals: &[], callees: FN_256_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_ef0263fa8fe1, globals: FN_257_GLOBALS, callees: FN_257_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_e35c1d781ec7, globals: FN_258_GLOBALS, callees: FN_258_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_47d0a89ab3c3, globals: &[], callees: FN_267_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_fef4b833a275, globals: &[], callees: FN_269_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_df0fa22086ad, globals: FN_272_GLOBALS, callees: FN_272_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_02bd3f85ecb6, globals: FN_273_GLOBALS, callees: FN_273_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_3f42f48f24b8, globals: FN_274_GLOBALS, callees: FN_274_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_9704072dbea0, globals: FN_281_GLOBALS, callees: FN_281_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_9c0d878b1a6c, globals: FN_283_GLOBALS, callees: FN_283_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_cec45d7b411d, globals: FN_284_GLOBALS, callees: FN_284_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_ea528067816b, globals: FN_285_GLOBALS, callees: FN_285_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85900_04292::ChunkFns::f_94115c07ddc1, globals: &[], callees: FN_286_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85900_04292::ChunkFns::f_05be71efd2df, globals: FN_287_GLOBALS, callees: FN_287_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85900_04292::ChunkFns::f_81ab99176846, globals: &[], callees: FN_291_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85900_04292::ChunkFns::f_bf98b8c73ee9, globals: &[], callees: FN_292_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85900_04292::ChunkFns::f_e759dd99c9c3, globals: FN_298_GLOBALS, callees: FN_298_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_38535251f258, globals: &[], callees: FN_299_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_83092_1::ChunkFns::f_5e1271fe89af, globals: &[], callees: &[] },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_3dd4bf2481ba, globals: &[], callees: &[] },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_c47bb8df9be0, globals: &[], callees: &[] },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_cbe0184bd9b5, globals: &[], callees: &[] },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_1a71d29336df, globals: &[], callees: &[] },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_1::ChunkFns::f_8b3529200404, globals: &[], callees: FN_356_CALLEES },
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
static BOUND_SIG_P_I32_RET_I64: &[crate::Bound_p_i32_ret_i64] = &[
    crate::Bound_p_i32_ret_i64 { func: crate::c_85899_88792_2::ChunkFns::f_cb1aa58541c6, globals: &[], callees: FN_48_CALLEES },
    crate::Bound_p_i32_ret_i64 { func: crate::c_85899_88792::ChunkFns::f_6f07359304f0, globals: &[], callees: FN_182_CALLEES },
    crate::Bound_p_i32_ret_i64 { func: crate::c_85899_92792_2::ChunkFns::f_c1c69edd038d, globals: &[], callees: FN_207_CALLEES },
    crate::Bound_p_i32_ret_i64 { func: crate::c_85899_92792_3::ChunkFns::f_7b2b3771f8f7, globals: &[], callees: FN_282_CALLEES },
    crate::Bound_p_i32_ret_i64 { func: crate::c_85899_83092_2::ChunkFns::f_c239f066ccbc, globals: &[], callees: &[] },
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
static BOUND_SIG_P_I64_I32_RET_I64: &[crate::Bound_p_i64_i32_ret_i64] = &[
    crate::Bound_p_i64_i32_ret_i64 { func: crate::c_85899_83092_1::ChunkFns::f_79815729aec3, globals: &[], callees: &[] },
    crate::Bound_p_i64_i32_ret_i64 { func: crate::c_85899_83092_1::ChunkFns::f_16e5e69025c5, globals: &[], callees: &[] },
    crate::Bound_p_i64_i32_ret_i64 { func: crate::c_85899_83092_2::ChunkFns::f_53b864d8a1de, globals: &[], callees: &[] },
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
static BOUND_SIG_P_I64_I64_RET_I32: &[crate::Bound_p_i64_i64_ret_i32] = &[
    crate::Bound_p_i64_i64_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_470053343055, globals: &[], callees: FN_23_CALLEES },
    crate::Bound_p_i64_i64_ret_i32 { func: crate::c_85899_83092::ChunkFns::f_bb85b13eb8ff, globals: &[], callees: FN_49_CALLEES },
    crate::Bound_p_i64_i64_ret_i32 { func: crate::c_85899_83092::ChunkFns::f_bb85b13eb8ff, globals: &[], callees: FN_80_CALLEES },
    crate::Bound_p_i64_i64_ret_i32 { func: crate::c_85899_83092::ChunkFns::f_bb85b13eb8ff, globals: &[], callees: FN_171_CALLEES },
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
static BOUND_SIG_P_I64_I64_RET_I64: &[crate::Bound_p_i64_i64_ret_i64] = &[
    crate::Bound_p_i64_i64_ret_i64 { func: crate::c_85899_83092_1::ChunkFns::f_2b59315b7818, globals: &[], callees: &[] },
    crate::Bound_p_i64_i64_ret_i64 { func: crate::c_85899_83092_1::ChunkFns::f_0de929be56dc, globals: &[], callees: &[] },
    crate::Bound_p_i64_i64_ret_i64 { func: crate::c_85899_83092_2::ChunkFns::f_a5a79dab7242, globals: &[], callees: &[] },
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
static BOUND_SIG_P_I64_RET_I32: &[crate::Bound_p_i64_ret_i32] = &[
    crate::Bound_p_i64_ret_i32 { func: crate::c_85899_83092::ChunkFns::f_3002f0cd1024, globals: &[], callees: FN_14_CALLEES },
    crate::Bound_p_i64_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_8d4ef5bbec94, globals: FN_36_GLOBALS, callees: FN_36_CALLEES },
    crate::Bound_p_i64_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_16a5c648c247, globals: &[], callees: FN_72_CALLEES },
    crate::Bound_p_i64_ret_i32 { func: crate::c_85899_83092::ChunkFns::f_3002f0cd1024, globals: &[], callees: FN_74_CALLEES },
    crate::Bound_p_i64_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_956ac04c06f0, globals: FN_125_GLOBALS, callees: FN_125_CALLEES },
    crate::Bound_p_i64_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_a80be209ca01, globals: &[], callees: FN_130_CALLEES },
    crate::Bound_p_i64_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_4ce1c3955399, globals: &[], callees: FN_192_CALLEES },
    crate::Bound_p_i64_ret_i32 { func: crate::c_85899_83092_1::ChunkFns::f_ee190614c423, globals: &[], callees: FN_300_CALLEES },
    crate::Bound_p_i64_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_15ed6333638e, globals: &[], callees: &[] },
    crate::Bound_p_i64_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_a95199099e27, globals: &[], callees: &[] },
    crate::Bound_p_i64_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_609444ff4048, globals: &[], callees: &[] },
    crate::Bound_p_i64_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_f36d2b83b7ff, globals: &[], callees: &[] },
    crate::Bound_p_i64_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_3961cde2a530, globals: &[], callees: FN_323_CALLEES },
    crate::Bound_p_i64_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_3961cde2a530, globals: &[], callees: FN_334_CALLEES },
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
static BOUND_SIG_P_I64_RET_I64: &[crate::Bound_p_i64_ret_i64] = &[
    crate::Bound_p_i64_ret_i64 { func: crate::c_85899_83092_2::ChunkFns::f_554f71c39a7a, globals: &[], callees: &[] },
    crate::Bound_p_i64_ret_i64 { func: crate::c_85899_83092_2::ChunkFns::f_554f71c39a7a, globals: &[], callees: &[] },
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
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_ce4a4f4eaaf0, globals: FN_252_GLOBALS, callees: FN_252_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85900_04292::ChunkFns::f_e6879a687a42, globals: &[], callees: FN_288_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_a6a8842e4b04, globals: &[], callees: FN_362_CALLEES },
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
    crate::Bound_p_unit_ret_void { func: crate::c_85900_04292::ChunkFns::f_426fb06d685b, globals: FN_0_GLOBALS, callees: FN_0_CALLEES },
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
    sig_p_i32_i32_i32_i32_i32_i32_ret_i32: &[],
    sig_p_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i64_i32_i32_i32_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i64_i32_i32_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i64_i32_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i64_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_ret_i32: &[],
    sig_p_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i64_ret_i32: BOUND_SIG_P_I32_I32_I32_I32_I64_RET_I32,
    sig_p_i32_i32_i32_i32_ret_i32: BOUND_SIG_P_I32_I32_I32_I32_RET_I32,
    sig_p_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i64_ret_void: &[],
    sig_p_i32_i32_i32_ret_i32: BOUND_SIG_P_I32_I32_I32_RET_I32,
    sig_p_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i64_i32_i32_i64_i32_ret_i32: &[],
    sig_p_i32_i32_i64_i32_i32_ret_i32: &[],
    sig_p_i32_i32_i64_i32_ret_i32: &[],
    sig_p_i32_i32_i64_ret_i32: BOUND_SIG_P_I32_I32_I64_RET_I32,
    sig_p_i32_i32_i64_ret_void: &[],
    sig_p_i32_i32_ret_i32: BOUND_SIG_P_I32_I32_RET_I32,
    sig_p_i32_i32_ret_i64: BOUND_SIG_P_I32_I32_RET_I64,
    sig_p_i32_i32_ret_void: BOUND_SIG_P_I32_I32_RET_VOID,
    sig_p_i32_i64_i32_i32_ret_i32: BOUND_SIG_P_I32_I64_I32_I32_RET_I32,
    sig_p_i32_i64_i32_i32_ret_i64: &[],
    sig_p_i32_i64_i32_ret_i32: BOUND_SIG_P_I32_I64_I32_RET_I32,
    sig_p_i32_i64_i32_ret_i64: &[],
    sig_p_i32_i64_i32_ret_void: BOUND_SIG_P_I32_I64_I32_RET_VOID,
    sig_p_i32_i64_i64_ret_i32: BOUND_SIG_P_I32_I64_I64_RET_I32,
    sig_p_i32_i64_i64_ret_i64: BOUND_SIG_P_I32_I64_I64_RET_I64,
    sig_p_i32_i64_i64_ret_void: &[],
    sig_p_i32_i64_ret_i32: BOUND_SIG_P_I32_I64_RET_I32,
    sig_p_i32_i64_ret_i64: BOUND_SIG_P_I32_I64_RET_I64,
    sig_p_i32_i64_ret_void: &[],
    sig_p_i32_ret_i32: BOUND_SIG_P_I32_RET_I32,
    sig_p_i32_ret_i64: BOUND_SIG_P_I32_RET_I64,
    sig_p_i32_ret_void: &[],
    sig_p_i64_i32_ret_i64: BOUND_SIG_P_I64_I32_RET_I64,
    sig_p_i64_i64_ret_i32: BOUND_SIG_P_I64_I64_RET_I32,
    sig_p_i64_i64_ret_i64: BOUND_SIG_P_I64_I64_RET_I64,
    sig_p_i64_ret_i32: BOUND_SIG_P_I64_RET_I32,
    sig_p_i64_ret_i64: BOUND_SIG_P_I64_RET_I64,
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
    (2228243u128, [0xb2, 0x27, 0xd6, 0x94, 0xe5, 0x8a, 0x37, 0xc8, 0x99, 0xa0, 0xf1, 0xe1, 0x81, 0x98, 0xa8, 0xc5, 0x91, 0xfc, 0x1a, 0x7e, 0x5a, 0xb2, 0xef, 0xc7, 0xae, 0xd0, 0x00, 0x34, 0xe3, 0xe9, 0x10, 0x26]),
];

const MODULE_META: SubstituteModuleMeta = SubstituteModuleMeta {
    name: "85900_04292/f19@l8590004760",
    accounting_version: 2u32,
    wasm_sha256: [0x13, 0xba, 0x00, 0xdf, 0x61, 0x64, 0x83, 0x6b, 0x1e, 0xcc, 0x1b, 0xfb, 0x12, 0xb8, 0xae, 0xea, 0xbc, 0x18, 0x1a, 0xb5, 0xec, 0x69, 0x11, 0x2a, 0x1c, 0x6d, 0x0f, 0x2a, 0x71, 0x47, 0xfc, 0x91],
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
    let g25: i32 = 0i32;
    let g26: i32 = 0i32;
    let g27: i32 = 0i32;
    let g28: i32 = 0i32;
    let g29: i32 = 0i32;
    let g30: i32 = 0i32;
    let g31: i32 = 0i32;
    let g32: i32 = 0i32;
    let g33: i32 = 256i32;
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
    vm.heap_ptr = g33;
    vm.has_heap_ptr = true;
    vm.globals_i32 = vec![g0, g1, g2, g3, g4, g5, g6, g7, g8, g9, g10, g11, g12, g13, g14, g15, g16, g17, g18, g19, g20, g21, g22, g23, g24, g25, g26, g27, g28, g29, g30, g31, g32];
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
            "__sl_inited" => self.vm.globals_i32.get(0usize).copied(),
            "__c_0" => self.vm.globals_i32.get(1usize).copied(),
            "__c_1" => self.vm.globals_i32.get(2usize).copied(),
            "__c_2" => self.vm.globals_i32.get(3usize).copied(),
            "__c_3" => self.vm.globals_i32.get(4usize).copied(),
            "__c_4" => self.vm.globals_i32.get(5usize).copied(),
            "__c_5" => self.vm.globals_i32.get(6usize).copied(),
            "__c_6" => self.vm.globals_i32.get(7usize).copied(),
            "__c_7" => self.vm.globals_i32.get(8usize).copied(),
            "__c_8" => self.vm.globals_i32.get(9usize).copied(),
            "__c_9" => self.vm.globals_i32.get(10usize).copied(),
            "__c_10" => self.vm.globals_i32.get(11usize).copied(),
            "__c_11" => self.vm.globals_i32.get(12usize).copied(),
            "__c_12" => self.vm.globals_i32.get(13usize).copied(),
            "__c_13" => self.vm.globals_i32.get(14usize).copied(),
            "__c_14" => self.vm.globals_i32.get(15usize).copied(),
            "__c_15" => self.vm.globals_i32.get(16usize).copied(),
            "__c_16" => self.vm.globals_i32.get(17usize).copied(),
            "__c_17" => self.vm.globals_i32.get(18usize).copied(),
            "__c_18" => self.vm.globals_i32.get(19usize).copied(),
            "__c_19" => self.vm.globals_i32.get(20usize).copied(),
            "__c_20" => self.vm.globals_i32.get(21usize).copied(),
            "__c_21" => self.vm.globals_i32.get(22usize).copied(),
            "__c_22" => self.vm.globals_i32.get(23usize).copied(),
            "__c_23" => self.vm.globals_i32.get(24usize).copied(),
            "__c_24" => self.vm.globals_i32.get(25usize).copied(),
            "__c_25" => self.vm.globals_i32.get(26usize).copied(),
            "__c_26" => self.vm.globals_i32.get(27usize).copied(),
            "__c_27" => self.vm.globals_i32.get(28usize).copied(),
            "__c_28" => self.vm.globals_i32.get(29usize).copied(),
            "__c_29" => self.vm.globals_i32.get(30usize).copied(),
            "__c_30" => self.vm.globals_i32.get(31usize).copied(),
            "__c_31" => self.vm.globals_i32.get(32usize).copied(),
            "__heap_ptr" => Some(self.vm.heap_ptr),
            _ => None,
        }
    }

    fn exported_global_i64(&self, name: &str) -> Option<i64> {
        match name {
            _ => None,
        }
    }

    fn set_heap_ptr(&mut self, val: i32) -> bool {
        self.vm.heap_ptr = val;
        true
    }

    fn call_start(&mut self, _host: &mut dyn WasmSubstituteHost) -> std::result::Result<(), WasmTrap> {
        Ok(())
    }

    fn call_entry(&mut self, in_ptr: i32, in_len: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
        let _ = (in_ptr, in_len);
        let vm = &mut self.vm;
        vm.enter_call()?;
        vm.begin_fast_epoch();
        let res = DISPATCH.call_direct_p_unit_ret_i32(vm, 2usize, host);
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

const FN_0_GLOBALS: &[usize] = &[0usize, 1usize, 2usize, 3usize, 4usize, 5usize, 6usize, 7usize, 8usize, 9usize, 10usize, 11usize, 12usize, 13usize, 14usize, 15usize, 16usize, 17usize, 18usize, 19usize, 20usize, 21usize, 22usize, 23usize, 24usize, 25usize, 26usize, 27usize, 28usize, 29usize, 30usize, 31usize, 32usize];
const FN_0_CALLEES: &[usize] = &[124usize];
const FN_1_CALLEES: &[usize] = &[0usize, 24usize];
const FN_2_CALLEES: &[usize] = &[0usize, 124usize];
const FN_3_CALLEES: &[usize] = &[0usize, 124usize];
const FN_4_CALLEES: &[usize] = &[0usize, 124usize];
const FN_5_CALLEES: &[usize] = &[0usize, 124usize];
const FN_6_CALLEES: &[usize] = &[0usize, 34usize, 10usize];
const FN_7_GLOBALS: &[usize] = &[14usize];
const FN_7_CALLEES: &[usize] = &[0usize, 120usize];
const FN_8_CALLEES: &[usize] = &[0usize, 0usize];
const FN_9_CALLEES: &[usize] = &[0usize, 10usize];
const FN_10_CALLEES: &[usize] = &[0usize, 121usize];
const FN_11_CALLEES: &[usize] = &[0usize, 121usize];
const FN_12_CALLEES: &[usize] = &[0usize, 25usize];
const FN_13_CALLEES: &[usize] = &[0usize, 120usize];
const FN_14_CALLEES: &[usize] = &[0usize, 124usize];
const FN_15_CALLEES: &[usize] = &[0usize, 124usize];
const FN_16_CALLEES: &[usize] = &[0usize, 124usize];
const FN_17_CALLEES: &[usize] = &[0usize, 13usize];
const FN_18_CALLEES: &[usize] = &[0usize, 13usize];
const FN_19_CALLEES: &[usize] = &[0usize, 13usize];
const FN_20_CALLEES: &[usize] = &[0usize];
const FN_21_CALLEES: &[usize] = &[0usize, 26usize];
const FN_22_CALLEES: &[usize] = &[0usize, 27usize];
const FN_23_CALLEES: &[usize] = &[0usize, 0usize, 0usize, 1usize, 1usize, 124usize];
const FN_24_CALLEES: &[usize] = &[0usize, 120usize];
const FN_25_CALLEES: &[usize] = &[0usize, 121usize];
const FN_26_CALLEES: &[usize] = &[0usize, 26usize];
const FN_27_CALLEES: &[usize] = &[0usize, 32usize, 122usize, 33usize, 2usize];
const FN_28_CALLEES: &[usize] = &[0usize, 33usize, 122usize, 32usize, 2usize];
const FN_29_CALLEES: &[usize] = &[0usize, 28usize];
const FN_30_CALLEES: &[usize] = &[0usize, 29usize];
const FN_31_CALLEES: &[usize] = &[0usize, 30usize];
const FN_32_CALLEES: &[usize] = &[0usize, 26usize];
const FN_33_GLOBALS: &[usize] = &[14usize, 9usize];
const FN_33_CALLEES: &[usize] = &[0usize, 34usize, 10usize, 124usize];
const FN_34_CALLEES: &[usize] = &[0usize, 31usize, 124usize];
const FN_35_GLOBALS: &[usize] = &[14usize, 9usize];
const FN_35_CALLEES: &[usize] = &[0usize, 4usize, 124usize];
const FN_36_GLOBALS: &[usize] = &[14usize, 9usize];
const FN_36_CALLEES: &[usize] = &[0usize, 0usize, 124usize];
const FN_37_CALLEES: &[usize] = &[0usize, 31usize, 124usize];
const FN_38_GLOBALS: &[usize] = &[14usize, 9usize];
const FN_38_CALLEES: &[usize] = &[0usize, 6usize, 35usize, 124usize];
const FN_39_GLOBALS: &[usize] = &[14usize, 9usize];
const FN_39_CALLEES: &[usize] = &[0usize, 124usize, 32usize];
const FN_40_GLOBALS: &[usize] = &[14usize, 9usize];
const FN_40_CALLEES: &[usize] = &[0usize, 124usize, 33usize];
const FN_41_GLOBALS: &[usize] = &[14usize, 9usize];
const FN_41_CALLEES: &[usize] = &[0usize, 10usize, 36usize, 124usize];
const FN_42_GLOBALS: &[usize] = &[9usize];
const FN_42_CALLEES: &[usize] = &[0usize, 8usize, 124usize];
const FN_43_GLOBALS: &[usize] = &[9usize];
const FN_43_CALLEES: &[usize] = &[0usize, 1usize, 124usize];
const FN_44_GLOBALS: &[usize] = &[9usize];
const FN_44_CALLEES: &[usize] = &[0usize, 9usize, 124usize];
const FN_45_GLOBALS: &[usize] = &[9usize];
const FN_45_CALLEES: &[usize] = &[0usize, 124usize];
const FN_46_GLOBALS: &[usize] = &[9usize];
const FN_46_CALLEES: &[usize] = &[0usize, 10usize, 124usize];
const FN_47_GLOBALS: &[usize] = &[14usize, 1usize];
const FN_47_CALLEES: &[usize] = &[0usize, 120usize, 124usize, 13usize, 125usize, 0usize, 32usize, 2usize, 10usize, 36usize, 37usize, 123usize];
const FN_48_CALLEES: &[usize] = &[0usize];
const FN_49_CALLEES: &[usize] = &[0usize, 124usize];
const FN_50_CALLEES: &[usize] = &[0usize, 1usize, 122usize, 0usize, 124usize];
const FN_51_CALLEES: &[usize] = &[0usize, 0usize, 122usize, 1usize, 124usize];
const FN_52_CALLEES: &[usize] = &[0usize, 124usize];
const FN_53_CALLEES: &[usize] = &[0usize, 0usize, 124usize];
const FN_54_CALLEES: &[usize] = &[0usize, 124usize];
const FN_55_CALLEES: &[usize] = &[0usize, 0usize, 122usize];
const FN_56_CALLEES: &[usize] = &[0usize, 1usize, 124usize, 122usize];
const FN_57_GLOBALS: &[usize] = &[14usize];
const FN_57_CALLEES: &[usize] = &[0usize, 21usize, 24usize, 36usize];
const FN_58_CALLEES: &[usize] = &[0usize, 60usize];
const FN_59_CALLEES: &[usize] = &[0usize, 124usize];
const FN_60_CALLEES: &[usize] = &[0usize, 124usize];
const FN_61_CALLEES: &[usize] = &[0usize, 124usize];
const FN_62_CALLEES: &[usize] = &[0usize, 124usize];
const FN_63_CALLEES: &[usize] = &[0usize, 68usize];
const FN_64_GLOBALS: &[usize] = &[19usize, 16usize, 17usize];
const FN_64_CALLEES: &[usize] = &[0usize, 124usize];
const FN_65_GLOBALS: &[usize] = &[19usize];
const FN_65_CALLEES: &[usize] = &[0usize, 124usize, 12usize, 6usize, 35usize, 21usize, 1usize, 12usize];
const FN_66_GLOBALS: &[usize] = &[16usize];
const FN_66_CALLEES: &[usize] = &[0usize, 124usize];
const FN_67_CALLEES: &[usize] = &[0usize, 70usize];
const FN_68_CALLEES: &[usize] = &[0usize, 24usize, 1usize];
const FN_69_CALLEES: &[usize] = &[0usize, 24usize, 2usize];
const FN_70_CALLEES: &[usize] = &[0usize, 61usize];
const FN_71_CALLEES: &[usize] = &[0usize, 124usize];
const FN_72_CALLEES: &[usize] = &[0usize, 124usize];
const FN_73_CALLEES: &[usize] = &[0usize, 2usize];
const FN_74_CALLEES: &[usize] = &[0usize, 124usize];
const FN_75_CALLEES: &[usize] = &[0usize, 124usize];
const FN_76_CALLEES: &[usize] = &[0usize, 124usize];
const FN_77_CALLEES: &[usize] = &[0usize, 124usize];
const FN_78_CALLEES: &[usize] = &[0usize, 51usize];
const FN_79_CALLEES: &[usize] = &[0usize, 51usize];
const FN_80_CALLEES: &[usize] = &[0usize, 124usize];
const FN_81_CALLEES: &[usize] = &[0usize];
const FN_82_CALLEES: &[usize] = &[0usize, 51usize];
const FN_83_CALLEES: &[usize] = &[0usize];
const FN_84_CALLEES: &[usize] = &[0usize, 62usize];
const FN_85_CALLEES: &[usize] = &[0usize, 63usize];
const FN_86_CALLEES: &[usize] = &[0usize, 124usize];
const FN_87_CALLEES: &[usize] = &[0usize, 26usize];
const FN_88_CALLEES: &[usize] = &[0usize, 24usize, 5usize];
const FN_89_CALLEES: &[usize] = &[0usize, 43usize, 67usize];
const FN_90_CALLEES: &[usize] = &[0usize, 64usize];
const FN_91_CALLEES: &[usize] = &[0usize, 65usize];
const FN_92_CALLEES: &[usize] = &[0usize, 66usize];
const FN_93_CALLEES: &[usize] = &[0usize, 1usize];
const FN_94_CALLEES: &[usize] = &[0usize, 67usize];
const FN_95_GLOBALS: &[usize] = &[19usize, 9usize];
const FN_95_CALLEES: &[usize] = &[0usize, 42usize, 68usize, 124usize];
const FN_96_GLOBALS: &[usize] = &[19usize, 1usize, 16usize, 17usize, 9usize];
const FN_96_CALLEES: &[usize] = &[0usize, 124usize, 124usize, 12usize, 31usize, 20usize, 21usize, 33usize];
const FN_97_GLOBALS: &[usize] = &[19usize, 9usize];
const FN_97_CALLEES: &[usize] = &[0usize, 20usize, 124usize];
const FN_98_CALLEES: &[usize] = &[0usize, 67usize, 21usize, 124usize, 68usize, 70usize];
const FN_99_GLOBALS: &[usize] = &[19usize, 9usize];
const FN_99_CALLEES: &[usize] = &[0usize, 44usize, 69usize, 124usize];
const FN_100_GLOBALS: &[usize] = &[16usize, 19usize, 9usize];
const FN_100_CALLEES: &[usize] = &[0usize, 124usize, 124usize, 33usize];
const FN_101_GLOBALS: &[usize] = &[19usize, 9usize];
const FN_101_CALLEES: &[usize] = &[0usize, 124usize, 24usize, 32usize, 20usize];
const FN_102_GLOBALS: &[usize] = &[19usize, 9usize];
const FN_102_CALLEES: &[usize] = &[0usize, 124usize, 0usize, 122usize, 1usize, 2usize];
const FN_103_GLOBALS: &[usize] = &[19usize, 9usize];
const FN_103_CALLEES: &[usize] = &[0usize, 45usize, 70usize, 124usize];
const FN_104_GLOBALS: &[usize] = &[9usize];
const FN_104_CALLEES: &[usize] = &[0usize, 28usize, 124usize, 11usize];
const FN_105_GLOBALS: &[usize] = &[9usize];
const FN_105_CALLEES: &[usize] = &[0usize, 124usize];
const FN_106_GLOBALS: &[usize] = &[14usize, 9usize];
const FN_106_CALLEES: &[usize] = &[0usize, 21usize, 124usize, 24usize];
const FN_107_GLOBALS: &[usize] = &[9usize];
const FN_107_CALLEES: &[usize] = &[0usize, 9usize, 124usize];
const FN_108_GLOBALS: &[usize] = &[9usize];
const FN_108_CALLEES: &[usize] = &[0usize, 124usize];
const FN_109_GLOBALS: &[usize] = &[9usize];
const FN_109_CALLEES: &[usize] = &[0usize, 10usize, 124usize];
const FN_110_GLOBALS: &[usize] = &[19usize, 1usize, 16usize, 17usize];
const FN_110_CALLEES: &[usize] = &[0usize, 124usize, 124usize, 12usize, 31usize, 20usize, 21usize, 33usize];
const FN_111_CALLEES: &[usize] = &[0usize, 24usize, 34usize, 20usize];
const FN_112_CALLEES: &[usize] = &[0usize, 24usize, 35usize, 20usize];
const FN_113_GLOBALS: &[usize] = &[19usize];
const FN_113_CALLEES: &[usize] = &[0usize, 70usize];
const FN_114_CALLEES: &[usize] = &[0usize, 24usize, 36usize, 20usize];
const FN_115_CALLEES: &[usize] = &[0usize, 124usize];
const FN_116_GLOBALS: &[usize] = &[22usize];
const FN_116_CALLEES: &[usize] = &[0usize, 96usize, 85usize, 95usize, 97usize];
const FN_117_CALLEES: &[usize] = &[0usize];
const FN_118_CALLEES: &[usize] = &[0usize, 41usize];
const FN_119_CALLEES: &[usize] = &[0usize];
const FN_120_CALLEES: &[usize] = &[0usize];
const FN_121_CALLEES: &[usize] = &[0usize];
const FN_122_CALLEES: &[usize] = &[0usize, 95usize, 126usize];
const FN_123_CALLEES: &[usize] = &[0usize, 92usize];
const FN_124_CALLEES: &[usize] = &[0usize, 125usize];
const FN_125_GLOBALS: &[usize] = &[22usize];
const FN_125_CALLEES: &[usize] = &[0usize, 9usize, 0usize, 124usize];
const FN_126_GLOBALS: &[usize] = &[22usize];
const FN_126_CALLEES: &[usize] = &[0usize, 124usize];
const FN_127_CALLEES: &[usize] = &[0usize, 36usize];
const FN_128_CALLEES: &[usize] = &[0usize, 9usize, 0usize, 124usize, 85usize];
const FN_129_CALLEES: &[usize] = &[0usize, 12usize, 38usize, 21usize, 85usize];
const FN_130_CALLEES: &[usize] = &[0usize, 4usize, 85usize, 0usize];
const FN_131_CALLEES: &[usize] = &[0usize, 5usize];
const FN_132_CALLEES: &[usize] = &[0usize, 100usize, 85usize];
const FN_133_CALLEES: &[usize] = &[0usize, 76usize];
const FN_134_CALLEES: &[usize] = &[0usize, 76usize];
const FN_135_CALLEES: &[usize] = &[0usize, 1usize, 12usize, 14usize, 1usize, 9usize, 0usize, 0usize, 1usize, 127usize, 85usize];
const FN_136_CALLEES: &[usize] = &[0usize];
const FN_137_CALLEES: &[usize] = &[0usize, 76usize];
const FN_138_CALLEES: &[usize] = &[0usize, 76usize];
const FN_139_GLOBALS: &[usize] = &[22usize];
const FN_139_CALLEES: &[usize] = &[0usize, 50usize, 124usize];
const FN_140_CALLEES: &[usize] = &[0usize, 93usize];
const FN_141_CALLEES: &[usize] = &[0usize, 98usize, 85usize];
const FN_142_CALLEES: &[usize] = &[0usize, 125usize];
const FN_143_GLOBALS: &[usize] = &[22usize];
const FN_143_CALLEES: &[usize] = &[0usize, 41usize, 124usize];
const FN_144_CALLEES: &[usize] = &[0usize, 94usize];
const FN_145_CALLEES: &[usize] = &[0usize, 42usize, 71usize];
const FN_146_CALLEES: &[usize] = &[0usize, 94usize];
const FN_147_GLOBALS: &[usize] = &[22usize, 1usize];
const FN_147_CALLEES: &[usize] = &[0usize, 41usize, 124usize, 100usize, 85usize];
const FN_148_CALLEES: &[usize] = &[0usize, 94usize, 34usize, 71usize, 124usize];
const FN_149_GLOBALS: &[usize] = &[19usize, 9usize, 16usize];
const FN_149_CALLEES: &[usize] = &[0usize, 51usize, 124usize, 1usize, 120usize, 25usize, 26usize];
const FN_150_GLOBALS: &[usize] = &[9usize];
const FN_150_CALLEES: &[usize] = &[0usize, 43usize, 124usize, 27usize];
const FN_151_GLOBALS: &[usize] = &[9usize];
const FN_151_CALLEES: &[usize] = &[0usize, 43usize, 124usize, 28usize];
const FN_152_GLOBALS: &[usize] = &[14usize, 9usize];
const FN_152_CALLEES: &[usize] = &[0usize, 124usize, 51usize];
const FN_153_GLOBALS: &[usize] = &[9usize];
const FN_153_CALLEES: &[usize] = &[0usize, 46usize, 124usize, 9usize];
const FN_154_GLOBALS: &[usize] = &[9usize];
const FN_154_CALLEES: &[usize] = &[0usize, 46usize, 124usize, 10usize];
const FN_155_GLOBALS: &[usize] = &[9usize];
const FN_155_CALLEES: &[usize] = &[0usize, 46usize, 124usize, 11usize];
const FN_156_GLOBALS: &[usize] = &[22usize, 1usize];
const FN_156_CALLEES: &[usize] = &[0usize, 41usize, 124usize, 100usize, 85usize];
const FN_157_CALLEES: &[usize] = &[0usize, 2usize, 15usize, 2usize];
const FN_158_GLOBALS: &[usize] = &[14usize];
const FN_158_CALLEES: &[usize] = &[0usize, 124usize, 15usize, 1usize, 1usize];
const FN_159_CALLEES: &[usize] = &[0usize, 2usize, 15usize];
const FN_160_CALLEES: &[usize] = &[0usize, 1usize, 12usize, 15usize, 9usize, 1usize, 0usize, 127usize];
const FN_161_CALLEES: &[usize] = &[0usize, 12usize, 15usize, 1usize, 2usize, 9usize, 1usize, 127usize, 50usize];
const FN_162_CALLEES: &[usize] = &[0usize, 124usize, 1usize, 16usize, 15usize, 14usize, 0usize, 9usize, 0usize, 0usize, 127usize, 50usize];
const FN_163_CALLEES: &[usize] = &[0usize, 14usize, 2usize, 15usize];
const FN_164_CALLEES: &[usize] = &[0usize, 95usize, 124usize, 15usize, 16usize, 2usize, 1usize, 9usize, 0usize, 127usize, 50usize, 1usize, 122usize, 0usize, 0usize, 127usize, 14usize, 128usize, 99usize];
const FN_165_CALLEES: &[usize] = &[0usize, 58usize];
const FN_166_CALLEES: &[usize] = &[0usize, 1usize, 124usize];
const FN_167_GLOBALS: &[usize] = &[23usize];
const FN_167_CALLEES: &[usize] = &[0usize, 56usize, 2usize, 9usize];
const FN_168_CALLEES: &[usize] = &[0usize, 124usize];
const FN_169_CALLEES: &[usize] = &[0usize];
const FN_170_CALLEES: &[usize] = &[0usize];
const FN_171_CALLEES: &[usize] = &[0usize, 124usize];
const FN_172_CALLEES: &[usize] = &[0usize, 1usize, 124usize];
const FN_173_GLOBALS: &[usize] = &[23usize];
const FN_173_CALLEES: &[usize] = &[0usize, 57usize, 2usize, 9usize];
const FN_174_CALLEES: &[usize] = &[0usize, 1usize];
const FN_175_CALLEES: &[usize] = &[0usize, 124usize];
const FN_176_CALLEES: &[usize] = &[0usize, 0usize, 13usize, 17usize, 129usize, 124usize, 2usize, 130usize, 131usize, 132usize, 1usize, 0usize, 127usize];
const FN_177_CALLEES: &[usize] = &[0usize, 3usize, 124usize, 17usize, 129usize, 1usize, 133usize, 132usize, 134usize, 0usize, 32usize, 2usize, 135usize, 121usize, 13usize, 55usize, 1usize];
const FN_178_GLOBALS: &[usize] = &[14usize];
const FN_178_CALLEES: &[usize] = &[0usize, 1usize, 10usize, 124usize, 32usize, 0usize, 2usize];
const FN_179_CALLEES: &[usize] = &[0usize, 7usize, 10usize, 3usize, 0usize, 0usize, 127usize];
const FN_180_CALLEES: &[usize] = &[0usize, 1usize, 7usize, 0usize, 127usize];
const FN_181_CALLEES: &[usize] = &[0usize, 59usize];
const FN_182_CALLEES: &[usize] = &[0usize, 60usize, 1usize, 122usize, 121usize, 1usize];
const FN_183_CALLEES: &[usize] = &[0usize, 3usize, 124usize, 18usize, 59usize, 1usize];
const FN_184_CALLEES: &[usize] = &[0usize, 18usize, 59usize, 124usize, 1usize, 5usize, 63usize, 7usize, 127usize, 122usize, 128usize, 136usize, 134usize, 3usize];
const FN_185_CALLEES: &[usize] = &[0usize, 1usize, 3usize, 124usize, 18usize, 59usize, 13usize, 127usize, 2usize];
const FN_186_CALLEES: &[usize] = &[0usize, 18usize, 59usize, 124usize, 7usize, 1usize, 127usize, 10usize, 3usize];
const FN_187_CALLEES: &[usize] = &[0usize, 1usize, 124usize, 18usize, 59usize, 56usize, 54usize, 7usize, 127usize, 122usize, 128usize, 3usize];
const FN_188_CALLEES: &[usize] = &[0usize, 18usize, 66usize, 1usize];
const FN_189_CALLEES: &[usize] = &[0usize, 18usize, 59usize, 1usize];
const FN_190_CALLEES: &[usize] = &[0usize, 59usize, 58usize, 124usize];
const FN_191_CALLEES: &[usize] = &[0usize, 1usize, 0usize, 13usize, 58usize, 2usize, 6usize, 4usize, 17usize, 134usize, 121usize, 136usize, 2usize, 127usize];
const FN_192_CALLEES: &[usize] = &[0usize, 0usize, 0usize];
const FN_193_CALLEES: &[usize] = &[0usize, 1usize, 13usize, 58usize, 2usize, 59usize, 127usize];
const FN_194_CALLEES: &[usize] = &[0usize, 1usize, 13usize, 58usize, 2usize, 17usize, 61usize, 59usize, 56usize, 127usize];
const FN_195_CALLEES: &[usize] = &[0usize, 67usize, 58usize, 124usize];
const FN_196_GLOBALS: &[usize] = &[14usize];
const FN_196_CALLEES: &[usize] = &[0usize, 120usize, 58usize, 53usize, 124usize, 13usize, 2usize, 12usize, 9usize, 121usize, 1usize, 33usize, 2usize, 14usize, 127usize];
const FN_197_CALLEES: &[usize] = &[0usize, 0usize, 0usize];
const FN_198_CALLEES: &[usize] = &[0usize, 124usize, 3usize, 137usize, 7usize, 17usize, 5usize, 0usize, 127usize];
const FN_199_CALLEES: &[usize] = &[0usize, 10usize, 3usize, 135usize, 0usize, 122usize, 121usize, 60usize, 1usize, 33usize, 128usize, 136usize];
const FN_200_CALLEES: &[usize] = &[0usize, 17usize, 129usize, 133usize, 132usize, 134usize];
const FN_201_CALLEES: &[usize] = &[0usize, 55usize];
const FN_202_CALLEES: &[usize] = &[0usize, 124usize, 13usize, 18usize, 2usize, 1usize, 127usize, 2usize, 4usize, 138usize];
const FN_203_CALLEES: &[usize] = &[0usize, 124usize, 2usize, 0usize, 1usize, 13usize, 4usize, 18usize, 127usize];
const FN_204_CALLEES: &[usize] = &[0usize, 18usize, 124usize, 3usize];
const FN_205_CALLEES: &[usize] = &[0usize, 4usize, 124usize];
const FN_206_CALLEES: &[usize] = &[0usize, 60usize];
const FN_207_CALLEES: &[usize] = &[0usize, 18usize, 59usize, 1usize, 1usize, 8usize];
const FN_208_CALLEES: &[usize] = &[0usize, 77usize];
const FN_209_CALLEES: &[usize] = &[0usize, 79usize, 13usize, 2usize, 1usize, 2usize, 127usize, 1usize, 4usize, 140usize, 124usize];
const FN_210_CALLEES: &[usize] = &[0usize, 104usize];
const FN_211_CALLEES: &[usize] = &[0usize, 129usize];
const FN_212_GLOBALS: &[usize] = &[3usize];
const FN_212_CALLEES: &[usize] = &[0usize, 64usize, 129usize, 62usize, 124usize];
const FN_213_CALLEES: &[usize] = &[0usize, 13usize, 59usize, 58usize, 2usize, 127usize];
const FN_214_GLOBALS: &[usize] = &[3usize];
const FN_214_CALLEES: &[usize] = &[0usize, 13usize, 18usize, 59usize, 1usize, 124usize, 2usize, 127usize];
const FN_215_CALLEES: &[usize] = &[0usize, 59usize];
const FN_216_CALLEES: &[usize] = &[0usize, 1usize, 18usize, 59usize, 78usize, 2usize, 0usize, 0usize, 1usize, 124usize];
const FN_217_CALLEES: &[usize] = &[0usize, 13usize, 18usize, 59usize, 1usize, 4usize, 2usize, 8usize, 124usize, 127usize];
const FN_218_CALLEES: &[usize] = &[0usize, 18usize, 1usize, 13usize, 4usize, 127usize];
const FN_219_CALLEES: &[usize] = &[0usize, 72usize, 124usize, 12usize, 9usize, 30usize];
const FN_220_CALLEES: &[usize] = &[0usize, 80usize, 124usize];
const FN_221_CALLEES: &[usize] = &[0usize, 124usize, 3usize, 24usize, 71usize];
const FN_222_CALLEES: &[usize] = &[0usize, 46usize, 71usize, 124usize];
const FN_223_CALLEES: &[usize] = &[0usize, 124usize, 3usize, 24usize, 71usize];
const FN_224_CALLEES: &[usize] = &[0usize, 124usize, 3usize, 71usize];
const FN_225_CALLEES: &[usize] = &[0usize, 72usize, 124usize, 12usize];
const FN_226_CALLEES: &[usize] = &[0usize, 80usize, 124usize];
const FN_227_CALLEES: &[usize] = &[0usize, 80usize, 124usize];
const FN_228_CALLEES: &[usize] = &[0usize, 80usize, 124usize];
const FN_229_CALLEES: &[usize] = &[0usize];
const FN_230_CALLEES: &[usize] = &[0usize, 124usize];
const FN_231_CALLEES: &[usize] = &[0usize];
const FN_232_CALLEES: &[usize] = &[0usize, 62usize, 124usize];
const FN_233_CALLEES: &[usize] = &[0usize, 64usize];
const FN_234_CALLEES: &[usize] = &[0usize, 80usize, 124usize, 101usize, 141usize, 3usize, 100usize, 76usize];
const FN_235_CALLEES: &[usize] = &[0usize, 77usize, 124usize, 102usize];
const FN_236_CALLEES: &[usize] = &[0usize, 124usize, 3usize, 100usize, 141usize, 76usize];
const FN_237_CALLEES: &[usize] = &[0usize, 77usize, 124usize, 103usize];
const FN_238_CALLEES: &[usize] = &[0usize, 124usize, 7usize, 72usize, 13usize, 11usize, 30usize, 27usize, 24usize];
const FN_239_CALLEES: &[usize] = &[0usize, 124usize, 7usize, 72usize, 10usize, 12usize, 30usize, 9usize, 0usize, 9usize];
const FN_240_CALLEES: &[usize] = &[0usize, 13usize, 124usize, 18usize, 71usize, 2usize, 127usize];
const FN_241_CALLEES: &[usize] = &[0usize, 124usize, 120usize, 71usize, 13usize, 11usize, 26usize, 24usize, 2usize, 25usize, 127usize, 99usize];
const FN_242_CALLEES: &[usize] = &[0usize, 124usize, 71usize, 13usize, 3usize, 2usize, 127usize, 99usize];
const FN_243_CALLEES: &[usize] = &[0usize, 7usize, 124usize, 72usize, 120usize, 9usize, 11usize, 9usize];
const FN_244_CALLEES: &[usize] = &[0usize, 7usize, 124usize, 72usize, 120usize, 9usize, 11usize, 24usize, 71usize, 10usize, 3usize];
const FN_245_CALLEES: &[usize] = &[0usize];
const FN_246_CALLEES: &[usize] = &[0usize, 130usize, 131usize, 133usize];
const FN_247_CALLEES: &[usize] = &[0usize, 1usize, 132usize];
const FN_248_CALLEES: &[usize] = &[0usize, 13usize, 72usize, 2usize, 19usize, 124usize, 71usize, 0usize, 120usize, 2usize, 111usize, 108usize, 113usize, 133usize, 0usize, 115usize, 116usize, 139usize, 127usize];
const FN_249_CALLEES: &[usize] = &[0usize, 18usize, 124usize];
const FN_250_CALLEES: &[usize] = &[0usize, 139usize, 124usize, 72usize, 13usize, 2usize, 19usize, 71usize, 0usize, 120usize, 2usize, 111usize, 108usize, 113usize, 133usize, 0usize, 115usize, 116usize];
const FN_251_CALLEES: &[usize] = &[0usize, 124usize, 0usize, 139usize];
const FN_252_GLOBALS: &[usize] = &[26usize];
const FN_252_CALLEES: &[usize] = &[0usize, 124usize, 13usize, 2usize, 127usize];
const FN_253_CALLEES: &[usize] = &[0usize, 110usize, 13usize, 124usize, 2usize, 139usize, 127usize];
const FN_254_CALLEES: &[usize] = &[0usize, 124usize, 71usize, 0usize];
const FN_255_CALLEES: &[usize] = &[0usize, 0usize, 124usize];
const FN_256_CALLEES: &[usize] = &[0usize, 124usize, 0usize, 139usize];
const FN_257_GLOBALS: &[usize] = &[24usize];
const FN_257_CALLEES: &[usize] = &[0usize, 59usize, 54usize, 72usize, 108usize];
const FN_258_GLOBALS: &[usize] = &[27usize, 28usize];
const FN_258_CALLEES: &[usize] = &[0usize, 10usize, 12usize];
const FN_259_CALLEES: &[usize] = &[0usize, 120usize];
const FN_260_CALLEES: &[usize] = &[0usize, 124usize, 0usize, 139usize, 105usize, 72usize, 108usize];
const FN_261_CALLEES: &[usize] = &[0usize, 124usize, 0usize, 13usize, 71usize, 2usize, 127usize];
const FN_262_CALLEES: &[usize] = &[0usize, 114usize, 132usize, 124usize, 4usize];
const FN_263_CALLEES: &[usize] = &[0usize, 13usize, 124usize, 2usize, 71usize, 139usize, 127usize, 117usize, 0usize, 3usize, 105usize];
const FN_264_CALLEES: &[usize] = &[0usize, 124usize, 13usize, 2usize, 4usize, 127usize, 117usize, 0usize, 139usize, 105usize, 72usize];
const FN_265_CALLEES: &[usize] = &[0usize, 13usize, 124usize, 2usize, 3usize, 4usize, 127usize, 117usize, 0usize, 139usize, 105usize, 72usize, 13usize];
const FN_266_GLOBALS: &[usize] = &[25usize];
const FN_266_CALLEES: &[usize] = &[0usize, 0usize, 59usize];
const FN_267_CALLEES: &[usize] = &[0usize, 2usize, 4usize, 124usize];
const FN_268_CALLEES: &[usize] = &[0usize, 17usize, 124usize];
const FN_269_CALLEES: &[usize] = &[0usize, 13usize, 85usize, 2usize, 127usize];
const FN_270_CALLEES: &[usize] = &[0usize, 112usize, 13usize, 4usize, 2usize, 127usize];
const FN_271_CALLEES: &[usize] = &[0usize, 124usize, 105usize, 0usize, 139usize, 3usize, 13usize, 2usize, 127usize];
const FN_272_GLOBALS: &[usize] = &[25usize];
const FN_272_CALLEES: &[usize] = &[0usize, 0usize, 59usize];
const FN_273_GLOBALS: &[usize] = &[24usize, 29usize];
const FN_273_CALLEES: &[usize] = &[0usize, 59usize, 54usize, 72usize, 10usize];
const FN_274_GLOBALS: &[usize] = &[25usize];
const FN_274_CALLEES: &[usize] = &[0usize, 0usize, 59usize];
const FN_275_CALLEES: &[usize] = &[0usize, 13usize, 4usize, 124usize, 139usize, 2usize, 127usize];
const FN_276_CALLEES: &[usize] = &[0usize, 124usize, 0usize, 139usize];
const FN_277_CALLEES: &[usize] = &[0usize, 18usize, 124usize, 110usize];
const FN_278_CALLEES: &[usize] = &[0usize, 18usize, 124usize, 13usize, 4usize, 2usize, 127usize, 1usize];
const FN_279_CALLEES: &[usize] = &[0usize, 3usize, 2usize, 124usize, 64usize, 74usize, 142usize, 105usize, 62usize, 106usize, 0usize];
const FN_280_GLOBALS: &[usize] = &[24usize, 25usize];
const FN_280_CALLEES: &[usize] = &[0usize, 115usize, 124usize, 13usize, 0usize, 59usize, 2usize, 107usize, 72usize, 112usize, 114usize, 133usize, 114usize, 111usize, 116usize, 117usize, 116usize, 12usize, 4usize, 127usize, 54usize, 104usize, 112usize];
const FN_281_GLOBALS: &[usize] = &[25usize];
const FN_281_CALLEES: &[usize] = &[0usize, 0usize, 59usize, 113usize];
const FN_282_CALLEES: &[usize] = &[0usize, 72usize, 0usize];
const FN_283_GLOBALS: &[usize] = &[27usize, 28usize];
const FN_283_CALLEES: &[usize] = &[0usize, 72usize, 10usize, 12usize];
const FN_284_GLOBALS: &[usize] = &[25usize];
const FN_284_CALLEES: &[usize] = &[0usize, 0usize, 59usize];
const FN_285_GLOBALS: &[usize] = &[25usize];
const FN_285_CALLEES: &[usize] = &[0usize, 0usize, 59usize];
const FN_286_CALLEES: &[usize] = &[0usize, 92usize, 122usize, 90usize];
const FN_287_GLOBALS: &[usize] = &[30usize, 31usize, 32usize];
const FN_287_CALLEES: &[usize] = &[0usize, 11usize, 10usize, 12usize];
const FN_288_CALLEES: &[usize] = &[0usize, 124usize, 13usize, 2usize, 0usize, 127usize];
const FN_289_CALLEES: &[usize] = &[0usize, 0usize, 18usize, 124usize, 120usize];
const FN_290_CALLEES: &[usize] = &[0usize, 0usize, 18usize, 124usize, 120usize, 13usize, 4usize, 2usize, 127usize, 1usize];
const FN_291_CALLEES: &[usize] = &[0usize, 124usize, 0usize, 59usize];
const FN_292_CALLEES: &[usize] = &[0usize, 7usize, 72usize, 124usize, 11usize, 11usize, 3usize, 127usize];
const FN_293_CALLEES: &[usize] = &[0usize, 71usize, 124usize, 2usize, 72usize, 108usize, 113usize, 133usize, 121usize, 108usize];
const FN_294_CALLEES: &[usize] = &[0usize, 124usize, 0usize, 72usize, 108usize, 5usize];
const FN_295_CALLEES: &[usize] = &[0usize, 124usize, 1usize, 6usize];
const FN_296_CALLEES: &[usize] = &[0usize, 71usize, 2usize, 124usize, 111usize, 72usize, 108usize, 113usize, 133usize, 120usize, 13usize, 0usize, 4usize, 127usize];
const FN_297_GLOBALS: &[usize] = &[30usize, 31usize, 32usize];
const FN_297_CALLEES: &[usize] = &[0usize, 72usize, 12usize, 10usize, 124usize, 118usize, 109usize, 119usize, 13usize, 71usize, 1usize, 0usize, 112usize, 119usize];
const FN_298_GLOBALS: &[usize] = &[30usize];
const FN_298_CALLEES: &[usize] = &[0usize, 1usize, 2usize, 124usize, 0usize, 139usize, 72usize, 11usize, 1usize, 13usize, 2usize, 4usize, 4usize, 127usize, 11usize, 59usize, 0usize, 110usize, 133usize, 6usize];
const FN_299_CALLEES: &[usize] = &[0usize, 93usize, 118usize, 94usize, 124usize];
const FN_300_CALLEES: &[usize] = &[124usize];
const FN_321_CALLEES: &[usize] = &[126usize];
const FN_323_CALLEES: &[usize] = &[124usize];
const FN_326_CALLEES: &[usize] = &[124usize];
const FN_330_CALLEES: &[usize] = &[124usize];
const FN_331_CALLEES: &[usize] = &[124usize];
const FN_334_CALLEES: &[usize] = &[124usize];
const FN_337_CALLEES: &[usize] = &[124usize];
const FN_344_CALLEES: &[usize] = &[124usize];
const FN_348_CALLEES: &[usize] = &[124usize];
const FN_349_CALLEES: &[usize] = &[124usize];
const FN_351_CALLEES: &[usize] = &[124usize];
const FN_352_CALLEES: &[usize] = &[124usize];
const FN_353_CALLEES: &[usize] = &[139usize];
const FN_355_CALLEES: &[usize] = &[138usize];
const FN_356_CALLEES: &[usize] = &[124usize];
const FN_358_CALLEES: &[usize] = &[124usize];
const FN_362_CALLEES: &[usize] = &[0usize, 124usize, 123usize];
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
    crate::Bound_p_i32_i32_i32_i32_i32_i32_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_174dfb33e5ba, globals: &[], callees: FN_265_CALLEES },
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
static BOUND_SIG_P_I32_I32_I32_I32_I64_RET_I32: &[crate::Bound_p_i32_i32_i32_i32_i64_ret_i32] = &[
    crate::Bound_p_i32_i32_i32_i32_i64_ret_i32 { func: crate::c_85900_04292::ChunkFns::f_1cf4620fa532, globals: &[], callees: FN_296_CALLEES },
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
    crate::Bound_p_i32_i32_i32_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_186d1c0e26c0, globals: &[], callees: FN_255_CALLEES },
    crate::Bound_p_i32_i32_i32_i32_ret_i32 { func: crate::c_85900_04292::ChunkFns::f_37bc21062338, globals: &[], callees: FN_290_CALLEES },
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
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_0ba96dccaf47, globals: FN_196_GLOBALS, callees: FN_196_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_99792::ChunkFns::f_5cbcd199a7c2, globals: &[], callees: FN_250_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_e8fc0c2f6c4c, globals: &[], callees: FN_261_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_20ed2f71bb81, globals: &[], callees: FN_275_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_99792::ChunkFns::f_8cad486356a1, globals: FN_280_GLOBALS, callees: FN_280_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85900_04292::ChunkFns::f_b4a27025f49f, globals: &[], callees: FN_293_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85900_04292::ChunkFns::f_be76f48db77a, globals: &[], callees: FN_294_CALLEES },
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
static BOUND_SIG_P_I32_I32_I64_RET_I32: &[crate::Bound_p_i32_i32_i64_ret_i32] = &[
    crate::Bound_p_i32_i32_i64_ret_i32 { func: crate::c_85900_04292::ChunkFns::f_f2aa3d0555d6, globals: FN_297_GLOBALS, callees: FN_297_CALLEES },
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
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_1_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_27aaa3cb9114, globals: &[], callees: FN_2_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f2340190950c, globals: &[], callees: FN_4_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_0ece3b1734d5, globals: &[], callees: FN_5_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_803fb230173c, globals: &[], callees: FN_6_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_8ed247cfa309, globals: FN_7_GLOBALS, callees: FN_7_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_a9acae168912, globals: &[], callees: FN_8_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_dcce91d2e133, globals: &[], callees: FN_9_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_12_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_1e6ff4c83b62, globals: &[], callees: FN_13_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_da0e89bc9783, globals: &[], callees: FN_17_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_6df8689f059b, globals: &[], callees: FN_18_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f9ce687d90ae, globals: &[], callees: FN_19_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_30ec60d4302a, globals: &[], callees: FN_20_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_21_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_22_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_da0e89bc9783, globals: &[], callees: FN_24_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_26_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7e58af41e27d, globals: &[], callees: FN_27_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7e58af41e27d, globals: &[], callees: FN_28_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_29_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_30_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_31_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_32_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_ace0563243a6, globals: FN_33_GLOBALS, callees: FN_33_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_c69686cacf82, globals: &[], callees: FN_34_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_1a5f3f7571e2, globals: &[], callees: FN_37_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f1c0a657afaa, globals: FN_38_GLOBALS, callees: FN_38_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_acb2a186ecef, globals: FN_39_GLOBALS, callees: FN_39_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_acb2a186ecef, globals: FN_40_GLOBALS, callees: FN_40_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f1c0a657afaa, globals: FN_41_GLOBALS, callees: FN_41_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_5cfd1c966ee0, globals: FN_47_GLOBALS, callees: FN_47_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_c1a365c57784, globals: &[], callees: FN_50_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f15f52fd10ac, globals: &[], callees: FN_51_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_b9ec3478b1be, globals: &[], callees: FN_52_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f5ab0dd84882, globals: &[], callees: FN_53_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_a07d59a0459c, globals: &[], callees: FN_54_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_998c31a576d2, globals: &[], callees: FN_56_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_58_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_27aaa3cb9114, globals: &[], callees: FN_59_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f2340190950c, globals: &[], callees: FN_61_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_0ece3b1734d5, globals: &[], callees: FN_62_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f8850b1df958, globals: &[], callees: FN_63_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_609b70a572c5, globals: FN_64_GLOBALS, callees: FN_64_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f148b5f5c14f, globals: FN_65_GLOBALS, callees: FN_65_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_e23aae24b862, globals: &[], callees: FN_67_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_70_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_1e6ff4c83b62, globals: &[], callees: FN_71_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_da0e89bc9783, globals: &[], callees: FN_78_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_6df8689f059b, globals: &[], callees: FN_79_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f9ce687d90ae, globals: &[], callees: FN_82_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7ec8bf846782, globals: &[], callees: FN_83_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_84_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_85_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_da0e89bc9783, globals: &[], callees: FN_86_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_293ae2a09ea3, globals: &[], callees: FN_89_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_90_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_91_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_92_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_94_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f1c0a657afaa, globals: FN_95_GLOBALS, callees: FN_95_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_28a9f52b5bce, globals: FN_96_GLOBALS, callees: FN_96_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_4cf93470be08, globals: &[], callees: FN_98_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f1c0a657afaa, globals: FN_99_GLOBALS, callees: FN_99_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_1acc39445a44, globals: FN_101_GLOBALS, callees: FN_101_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_5230ac976967, globals: FN_102_GLOBALS, callees: FN_102_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f1c0a657afaa, globals: FN_103_GLOBALS, callees: FN_103_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_3c62ec56fda8, globals: FN_110_GLOBALS, callees: FN_110_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f90a3e72a759, globals: &[], callees: FN_111_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f90a3e72a759, globals: &[], callees: FN_112_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f90a3e72a759, globals: &[], callees: FN_114_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_40871f360722, globals: FN_116_GLOBALS, callees: FN_116_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_84192::ChunkFns::f_4f573e6f88cb, globals: &[], callees: FN_117_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_67329667d7fd, globals: &[], callees: FN_118_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_84192::ChunkFns::f_4f573e6f88cb, globals: &[], callees: FN_119_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_84192::ChunkFns::f_4f573e6f88cb, globals: &[], callees: FN_121_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_ca8d871e077c, globals: &[], callees: FN_122_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_123_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_1e6ff4c83b62, globals: &[], callees: FN_124_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_3594571ef794, globals: &[], callees: FN_132_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_e02bb7a5ce3b, globals: &[], callees: FN_133_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_868a5fcb144d, globals: &[], callees: FN_134_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_94c31aaa12fa, globals: &[], callees: FN_135_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_c56c009f41de, globals: &[], callees: FN_137_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_cfe54f4f1606, globals: &[], callees: FN_138_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_da603e0969cb, globals: FN_139_GLOBALS, callees: FN_139_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_140_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_0364226cb1e8, globals: &[], callees: FN_141_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_da0e89bc9783, globals: &[], callees: FN_142_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_144_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_194f062f76fc, globals: &[], callees: FN_145_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_146_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_4a4ae773c3d4, globals: FN_147_GLOBALS, callees: FN_147_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_1592393fe606, globals: &[], callees: FN_148_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_04c43103713a, globals: FN_156_GLOBALS, callees: FN_156_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_2af4fe9e7d26, globals: &[], callees: FN_159_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_8a603a474721, globals: &[], callees: FN_160_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_08e07943209e, globals: &[], callees: FN_161_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_188a11564f14, globals: &[], callees: FN_162_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_d3a85f461ba8, globals: &[], callees: FN_163_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_a70135e4cf19, globals: &[], callees: FN_164_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_b238ded6c3fe, globals: &[], callees: FN_193_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_fbaa521b0850, globals: &[], callees: FN_195_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_f008f64e784a, globals: &[], callees: FN_202_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_9b9e4d79c2a8, globals: &[], callees: FN_203_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_59a0410eeb26, globals: &[], callees: FN_209_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_dcce91d2e133, globals: &[], callees: FN_210_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_17e4d117a9e2, globals: &[], callees: FN_246_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_c83374c783a1, globals: &[], callees: FN_247_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_99792::ChunkFns::f_5450ef596911, globals: &[], callees: FN_248_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_f9611536c652, globals: &[], callees: FN_251_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_b732433177f0, globals: &[], callees: FN_254_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_0f8beff50967, globals: &[], callees: FN_259_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_426d859a938f, globals: &[], callees: FN_262_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_2d42b20c8b6d, globals: &[], callees: FN_263_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_bbee40c093b4, globals: &[], callees: FN_264_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_027e6ffc1a1a, globals: &[], callees: FN_270_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_92e33a06770e, globals: &[], callees: FN_279_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85900_04292::ChunkFns::f_bd36f6dae8b0, globals: &[], callees: FN_289_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85900_04292::ChunkFns::f_eeae7770e899, globals: &[], callees: FN_295_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_2::ChunkFns::f_03e481579af4, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_1::ChunkFns::f_c6e97ee70046, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_5015758bb709, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_3948a84700b8, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_2::ChunkFns::f_03e481579af4, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_2::ChunkFns::f_162b60a1a7e7, globals: &[], callees: FN_321_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_11c79f5620b0, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_ea4faf22326d, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_a43a5ae7b21f, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_2c680f88091b, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_140fb33a5afe, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_32917fc773d9, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_7d1f4d7fcad4, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_fa57b0c2b6ab, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_a74c85b451e0, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_3f75c31e0aa0, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_aba70ab0b976, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_deffe42432b7, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_f96a9e02e90e, globals: &[], callees: FN_353_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_2::ChunkFns::f_03e481579af4, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_f96a9e02e90e, globals: &[], callees: FN_355_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_ec7fee50bb79, globals: &[], callees: FN_358_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_5591fa286389, globals: &[], callees: &[] },
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
static BOUND_SIG_P_I32_I32_RET_I64: &[crate::Bound_p_i32_i32_ret_i64] = &[
    crate::Bound_p_i32_i32_ret_i64 { func: crate::c_85899_88792::ChunkFns::f_3abc88984867, globals: &[], callees: FN_55_CALLEES },
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
    crate::Bound_p_i32_i32_ret_void { func: crate::c_85899_88792::ChunkFns::f_c55926dd782d, globals: &[], callees: FN_199_CALLEES },
    crate::Bound_p_i32_i32_ret_void { func: crate::c_85899_83092_2::ChunkFns::f_71d8a2d6235d, globals: &[], callees: FN_326_CALLEES },
    crate::Bound_p_i32_i32_ret_void { func: crate::c_85899_83092_2::ChunkFns::f_71d8a2d6235d, globals: &[], callees: FN_337_CALLEES },
    crate::Bound_p_i32_i32_ret_void { func: crate::c_85899_83092_2::ChunkFns::f_e79c4e116bfd, globals: &[], callees: FN_344_CALLEES },
    crate::Bound_p_i32_i32_ret_void { func: crate::c_85899_83092_2::ChunkFns::f_bc6c808270d1, globals: &[], callees: FN_348_CALLEES },
    crate::Bound_p_i32_i32_ret_void { func: crate::c_85899_88792::ChunkFns::f_8fa562cbc853, globals: &[], callees: FN_351_CALLEES },
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
static BOUND_SIG_P_I32_I64_I32_I32_RET_I32: &[crate::Bound_p_i32_i64_i32_i32_ret_i32] = &[
    crate::Bound_p_i32_i64_i32_i32_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_c3c4818e5a41, globals: &[], callees: FN_278_CALLEES },
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
static BOUND_SIG_P_I32_I64_I32_RET_I32: &[crate::Bound_p_i32_i64_i32_ret_i32] = &[
    crate::Bound_p_i32_i64_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_b953838600c1, globals: &[], callees: FN_249_CALLEES },
    crate::Bound_p_i32_i64_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_6cb216adc685, globals: &[], callees: FN_268_CALLEES },
    crate::Bound_p_i32_i64_i32_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_c7ad525e5a02, globals: &[], callees: FN_277_CALLEES },
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
static BOUND_SIG_P_I32_I64_I32_RET_VOID: &[crate::Bound_p_i32_i64_i32_ret_void] = &[
    crate::Bound_p_i32_i64_i32_ret_void { func: crate::c_85899_83092_2::ChunkFns::f_0a0fecb79fc7, globals: &[], callees: &[] },
    crate::Bound_p_i32_i64_i32_ret_void { func: crate::c_85899_83092_2::ChunkFns::f_0a0fecb79fc7, globals: &[], callees: &[] },
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
    crate::Bound_p_i32_i64_i64_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_b6cf541461dc, globals: &[], callees: FN_205_CALLEES },
    crate::Bound_p_i32_i64_i64_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_a4773ed743de, globals: &[], callees: FN_218_CALLEES },
    crate::Bound_p_i32_i64_i64_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_9adecba56606, globals: &[], callees: FN_330_CALLEES },
    crate::Bound_p_i32_i64_i64_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_9607d677c9db, globals: &[], callees: FN_349_CALLEES },
    crate::Bound_p_i32_i64_i64_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_9adecba56606, globals: &[], callees: FN_352_CALLEES },
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
static BOUND_SIG_P_I32_I64_I64_RET_I64: &[crate::Bound_p_i32_i64_i64_ret_i64] = &[
    crate::Bound_p_i32_i64_i64_ret_i64 { func: crate::c_85899_88792::ChunkFns::f_31af32cb558c, globals: &[], callees: FN_200_CALLEES },
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
static BOUND_SIG_P_I32_I64_RET_I32: &[crate::Bound_p_i32_i64_ret_i32] = &[
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f0da2366c908, globals: &[], callees: FN_177_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_88792_2::ChunkFns::f_7503dfb81265, globals: &[], callees: FN_183_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_deaf01e451f9, globals: &[], callees: FN_184_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_88792_2::ChunkFns::f_ea9ae49cf4e8, globals: &[], callees: FN_185_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_88792_2::ChunkFns::f_a73ab5eebd75, globals: &[], callees: FN_186_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_b96282108807, globals: &[], callees: FN_187_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_419c292e1d24, globals: &[], callees: FN_197_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_a996482bc1ef, globals: &[], callees: FN_204_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_3de4de76d7bc, globals: &[], callees: FN_216_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_7a27ed8e21d5, globals: &[], callees: FN_238_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_ee91944eafa8, globals: &[], callees: FN_239_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_f2d9b324c003, globals: &[], callees: FN_260_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_94f57c3a085c, globals: &[], callees: FN_271_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_5ec036ffae9b, globals: &[], callees: FN_276_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_a4027a2778a0, globals: &[], callees: &[] },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_a4027a2778a0, globals: &[], callees: &[] },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_e849c87b3008, globals: &[], callees: FN_331_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_87279bd968a2, globals: &[], callees: &[] },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_a4027a2778a0, globals: &[], callees: &[] },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_a4027a2778a0, globals: &[], callees: &[] },
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
static BOUND_SIG_P_I32_I64_RET_I64: &[crate::Bound_p_i32_i64_ret_i64] = &[
    crate::Bound_p_i32_i64_ret_i64 { func: crate::c_85899_88792::ChunkFns::f_2bae3ae8abdb, globals: &[], callees: FN_188_CALLEES },
    crate::Bound_p_i32_i64_ret_i64 { func: crate::c_85899_88792::ChunkFns::f_241fa345f31b, globals: &[], callees: FN_189_CALLEES },
    crate::Bound_p_i32_i64_ret_i64 { func: crate::c_85899_92792_3::ChunkFns::f_112be49a4c6a, globals: FN_266_GLOBALS, callees: FN_266_CALLEES },
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
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_1d111f94255c, globals: &[], callees: FN_3_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7eb29a59d915, globals: &[], callees: FN_10_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_fab6c7792bec, globals: &[], callees: FN_11_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_a45a26c14c17, globals: &[], callees: FN_15_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_8f0ec661a804, globals: &[], callees: FN_16_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_3979fdeb44f5, globals: &[], callees: FN_25_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_de050b4b81f1, globals: FN_35_GLOBALS, callees: FN_35_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_327504b23599, globals: FN_42_GLOBALS, callees: FN_42_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_703887b4ca0f, globals: FN_43_GLOBALS, callees: FN_43_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_671a96cacef8, globals: FN_44_GLOBALS, callees: FN_44_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f1674ef8dccc, globals: FN_45_GLOBALS, callees: FN_45_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_e75c24ce5706, globals: FN_46_GLOBALS, callees: FN_46_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7d101c2d913a, globals: FN_57_GLOBALS, callees: FN_57_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_1d111f94255c, globals: &[], callees: FN_60_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_c65ede4802f8, globals: FN_66_GLOBALS, callees: FN_66_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_005520f212f5, globals: &[], callees: FN_68_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_005520f212f5, globals: &[], callees: FN_69_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_1e881702eb44, globals: &[], callees: FN_73_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_a45a26c14c17, globals: &[], callees: FN_75_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_8f0ec661a804, globals: &[], callees: FN_76_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_83092::ChunkFns::f_0e5bb44a1e78, globals: &[], callees: FN_77_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_0cbcbc1ef76e, globals: &[], callees: FN_81_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_9d92c7701c67, globals: &[], callees: FN_87_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_005520f212f5, globals: &[], callees: FN_88_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_49de3a4260ff, globals: &[], callees: FN_93_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_e32d590b954e, globals: FN_97_GLOBALS, callees: FN_97_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_329f5ce01511, globals: FN_100_GLOBALS, callees: FN_100_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_dbe6bf1cc46b, globals: FN_104_GLOBALS, callees: FN_104_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_0fabceb05d99, globals: FN_105_GLOBALS, callees: FN_105_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_b6a8cfd6ecb8, globals: FN_106_GLOBALS, callees: FN_106_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_671a96cacef8, globals: FN_107_GLOBALS, callees: FN_107_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f1674ef8dccc, globals: FN_108_GLOBALS, callees: FN_108_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_e75c24ce5706, globals: FN_109_GLOBALS, callees: FN_109_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_b828c3bb90f0, globals: FN_113_GLOBALS, callees: FN_113_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_83092::ChunkFns::f_70644cf2f7cf, globals: &[], callees: FN_115_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_84192::ChunkFns::f_7931de08c6d9, globals: &[], callees: FN_120_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_64d7099dbc77, globals: FN_126_GLOBALS, callees: FN_126_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_358231fcf579, globals: &[], callees: FN_127_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_d421c25ba8cd, globals: &[], callees: FN_128_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_22fbbfe4115b, globals: &[], callees: FN_129_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_1e881702eb44, globals: &[], callees: FN_131_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_f1d61f915b91, globals: &[], callees: FN_136_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_f6699ee6758d, globals: FN_143_GLOBALS, callees: FN_143_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_56b1ab258317, globals: FN_149_GLOBALS, callees: FN_149_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_e8394947df8a, globals: FN_150_GLOBALS, callees: FN_150_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_a01f25005cd0, globals: FN_151_GLOBALS, callees: FN_151_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_ac9ed96b8296, globals: FN_152_GLOBALS, callees: FN_152_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_e8394947df8a, globals: FN_153_GLOBALS, callees: FN_153_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_a01f25005cd0, globals: FN_154_GLOBALS, callees: FN_154_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_3f199480863a, globals: FN_155_GLOBALS, callees: FN_155_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_4374122c356c, globals: &[], callees: FN_157_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_93af815d1803, globals: FN_158_GLOBALS, callees: FN_158_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_2a1df693f599, globals: &[], callees: FN_165_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_428301bbe702, globals: &[], callees: FN_166_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_16b21026a7f2, globals: FN_167_GLOBALS, callees: FN_167_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_83092::ChunkFns::f_0e5bb44a1e78, globals: &[], callees: FN_168_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_ffdee7c93cbe, globals: &[], callees: FN_169_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_3eb654e927a1, globals: &[], callees: FN_170_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_3927290801c7, globals: &[], callees: FN_172_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_d89853f7c7c5, globals: FN_173_GLOBALS, callees: FN_173_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_49de3a4260ff, globals: &[], callees: FN_174_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_a45a26c14c17, globals: &[], callees: FN_175_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_5e048dbe7c1f, globals: &[], callees: FN_176_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f26f1b9b1ce2, globals: FN_178_GLOBALS, callees: FN_178_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_a4b8b05c5fad, globals: &[], callees: FN_179_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_45817b7043dc, globals: &[], callees: FN_180_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_fdb0906f56bb, globals: &[], callees: FN_181_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_bdc4fb2aaab1, globals: &[], callees: FN_190_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_990600c4a836, globals: &[], callees: FN_191_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_34a49f345dca, globals: &[], callees: FN_194_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_2::ChunkFns::f_638e3fd61a80, globals: &[], callees: FN_198_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_2::ChunkFns::f_61a16d9614ba, globals: &[], callees: FN_201_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_2::ChunkFns::f_61a16d9614ba, globals: &[], callees: FN_206_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_9d92c7701c67, globals: &[], callees: FN_208_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_1::ChunkFns::f_db0ffa8054bf, globals: &[], callees: FN_211_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_cd58da62b9e3, globals: FN_212_GLOBALS, callees: FN_212_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_a2186d19e2c0, globals: &[], callees: FN_213_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_aa32df195e2f, globals: FN_214_GLOBALS, callees: FN_214_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_5e4fbe80a2d9, globals: &[], callees: FN_215_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_e4afd653eea2, globals: &[], callees: FN_217_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_03c6e9c99b64, globals: &[], callees: FN_219_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_038735e9e9c7, globals: &[], callees: FN_220_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_7425067462bd, globals: &[], callees: FN_221_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_a4564f09a357, globals: &[], callees: FN_222_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_975343290a87, globals: &[], callees: FN_223_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_2e9ae6967fad, globals: &[], callees: FN_224_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_fec92329e980, globals: &[], callees: FN_225_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_87a4332dcdea, globals: &[], callees: FN_226_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_d725ff05410f, globals: &[], callees: FN_227_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_3505cdd4951f, globals: &[], callees: FN_228_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792::ChunkFns::f_7c75ee00bddb, globals: &[], callees: FN_229_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792::ChunkFns::f_49eed9e9e18f, globals: &[], callees: FN_230_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792::ChunkFns::f_7c75ee00bddb, globals: &[], callees: FN_231_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_97d4a47a22ec, globals: &[], callees: FN_232_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_2::ChunkFns::f_61a16d9614ba, globals: &[], callees: FN_233_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_9b4a1c598462, globals: &[], callees: FN_234_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_e3f6ab9ba400, globals: &[], callees: FN_235_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_a2a7ff38a2b7, globals: &[], callees: FN_236_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_d9ea1df92c5e, globals: &[], callees: FN_237_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_9c15bf600887, globals: &[], callees: FN_240_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_47fe2304968c, globals: &[], callees: FN_241_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_94d168266957, globals: &[], callees: FN_242_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_093751662340, globals: &[], callees: FN_243_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_e59a8e4dd5a8, globals: &[], callees: FN_244_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_6da45e6623e3, globals: &[], callees: FN_245_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_d53d5875197a, globals: &[], callees: FN_253_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_dd64d7da387f, globals: &[], callees: FN_256_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_ef0263fa8fe1, globals: FN_257_GLOBALS, callees: FN_257_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_e35c1d781ec7, globals: FN_258_GLOBALS, callees: FN_258_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_47d0a89ab3c3, globals: &[], callees: FN_267_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_fef4b833a275, globals: &[], callees: FN_269_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_df0fa22086ad, globals: FN_272_GLOBALS, callees: FN_272_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_02bd3f85ecb6, globals: FN_273_GLOBALS, callees: FN_273_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_3f42f48f24b8, globals: FN_274_GLOBALS, callees: FN_274_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_9704072dbea0, globals: FN_281_GLOBALS, callees: FN_281_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_9c0d878b1a6c, globals: FN_283_GLOBALS, callees: FN_283_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_cec45d7b411d, globals: FN_284_GLOBALS, callees: FN_284_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_ea528067816b, globals: FN_285_GLOBALS, callees: FN_285_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85900_04292::ChunkFns::f_94115c07ddc1, globals: &[], callees: FN_286_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85900_04292::ChunkFns::f_05be71efd2df, globals: FN_287_GLOBALS, callees: FN_287_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85900_04292::ChunkFns::f_81ab99176846, globals: &[], callees: FN_291_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85900_04292::ChunkFns::f_bf98b8c73ee9, globals: &[], callees: FN_292_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85900_04292::ChunkFns::f_e759dd99c9c3, globals: FN_298_GLOBALS, callees: FN_298_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_38535251f258, globals: &[], callees: FN_299_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_83092_1::ChunkFns::f_5e1271fe89af, globals: &[], callees: &[] },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_3dd4bf2481ba, globals: &[], callees: &[] },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_c47bb8df9be0, globals: &[], callees: &[] },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_cbe0184bd9b5, globals: &[], callees: &[] },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_1a71d29336df, globals: &[], callees: &[] },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_1::ChunkFns::f_8b3529200404, globals: &[], callees: FN_356_CALLEES },
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
static BOUND_SIG_P_I32_RET_I64: &[crate::Bound_p_i32_ret_i64] = &[
    crate::Bound_p_i32_ret_i64 { func: crate::c_85899_88792_2::ChunkFns::f_cb1aa58541c6, globals: &[], callees: FN_48_CALLEES },
    crate::Bound_p_i32_ret_i64 { func: crate::c_85899_88792::ChunkFns::f_6f07359304f0, globals: &[], callees: FN_182_CALLEES },
    crate::Bound_p_i32_ret_i64 { func: crate::c_85899_92792_2::ChunkFns::f_c1c69edd038d, globals: &[], callees: FN_207_CALLEES },
    crate::Bound_p_i32_ret_i64 { func: crate::c_85899_92792_3::ChunkFns::f_7b2b3771f8f7, globals: &[], callees: FN_282_CALLEES },
    crate::Bound_p_i32_ret_i64 { func: crate::c_85899_83092_2::ChunkFns::f_c239f066ccbc, globals: &[], callees: &[] },
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
static BOUND_SIG_P_I64_I32_RET_I64: &[crate::Bound_p_i64_i32_ret_i64] = &[
    crate::Bound_p_i64_i32_ret_i64 { func: crate::c_85899_83092_1::ChunkFns::f_79815729aec3, globals: &[], callees: &[] },
    crate::Bound_p_i64_i32_ret_i64 { func: crate::c_85899_83092_1::ChunkFns::f_16e5e69025c5, globals: &[], callees: &[] },
    crate::Bound_p_i64_i32_ret_i64 { func: crate::c_85899_83092_2::ChunkFns::f_53b864d8a1de, globals: &[], callees: &[] },
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
static BOUND_SIG_P_I64_I64_RET_I32: &[crate::Bound_p_i64_i64_ret_i32] = &[
    crate::Bound_p_i64_i64_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_470053343055, globals: &[], callees: FN_23_CALLEES },
    crate::Bound_p_i64_i64_ret_i32 { func: crate::c_85899_83092::ChunkFns::f_bb85b13eb8ff, globals: &[], callees: FN_49_CALLEES },
    crate::Bound_p_i64_i64_ret_i32 { func: crate::c_85899_83092::ChunkFns::f_bb85b13eb8ff, globals: &[], callees: FN_80_CALLEES },
    crate::Bound_p_i64_i64_ret_i32 { func: crate::c_85899_83092::ChunkFns::f_bb85b13eb8ff, globals: &[], callees: FN_171_CALLEES },
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
static BOUND_SIG_P_I64_I64_RET_I64: &[crate::Bound_p_i64_i64_ret_i64] = &[
    crate::Bound_p_i64_i64_ret_i64 { func: crate::c_85899_83092_1::ChunkFns::f_2b59315b7818, globals: &[], callees: &[] },
    crate::Bound_p_i64_i64_ret_i64 { func: crate::c_85899_83092_1::ChunkFns::f_0de929be56dc, globals: &[], callees: &[] },
    crate::Bound_p_i64_i64_ret_i64 { func: crate::c_85899_83092_2::ChunkFns::f_a5a79dab7242, globals: &[], callees: &[] },
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
static BOUND_SIG_P_I64_RET_I32: &[crate::Bound_p_i64_ret_i32] = &[
    crate::Bound_p_i64_ret_i32 { func: crate::c_85899_83092::ChunkFns::f_3002f0cd1024, globals: &[], callees: FN_14_CALLEES },
    crate::Bound_p_i64_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_8d4ef5bbec94, globals: FN_36_GLOBALS, callees: FN_36_CALLEES },
    crate::Bound_p_i64_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_16a5c648c247, globals: &[], callees: FN_72_CALLEES },
    crate::Bound_p_i64_ret_i32 { func: crate::c_85899_83092::ChunkFns::f_3002f0cd1024, globals: &[], callees: FN_74_CALLEES },
    crate::Bound_p_i64_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_956ac04c06f0, globals: FN_125_GLOBALS, callees: FN_125_CALLEES },
    crate::Bound_p_i64_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_a80be209ca01, globals: &[], callees: FN_130_CALLEES },
    crate::Bound_p_i64_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_4ce1c3955399, globals: &[], callees: FN_192_CALLEES },
    crate::Bound_p_i64_ret_i32 { func: crate::c_85899_83092_1::ChunkFns::f_ee190614c423, globals: &[], callees: FN_300_CALLEES },
    crate::Bound_p_i64_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_15ed6333638e, globals: &[], callees: &[] },
    crate::Bound_p_i64_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_a95199099e27, globals: &[], callees: &[] },
    crate::Bound_p_i64_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_609444ff4048, globals: &[], callees: &[] },
    crate::Bound_p_i64_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_f36d2b83b7ff, globals: &[], callees: &[] },
    crate::Bound_p_i64_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_3961cde2a530, globals: &[], callees: FN_323_CALLEES },
    crate::Bound_p_i64_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_3961cde2a530, globals: &[], callees: FN_334_CALLEES },
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
static BOUND_SIG_P_I64_RET_I64: &[crate::Bound_p_i64_ret_i64] = &[
    crate::Bound_p_i64_ret_i64 { func: crate::c_85899_83092_2::ChunkFns::f_554f71c39a7a, globals: &[], callees: &[] },
    crate::Bound_p_i64_ret_i64 { func: crate::c_85899_83092_2::ChunkFns::f_554f71c39a7a, globals: &[], callees: &[] },
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
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_ce4a4f4eaaf0, globals: FN_252_GLOBALS, callees: FN_252_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85900_04292::ChunkFns::f_e6879a687a42, globals: &[], callees: FN_288_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_a6a8842e4b04, globals: &[], callees: FN_362_CALLEES },
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
    crate::Bound_p_unit_ret_void { func: crate::c_85900_04292::ChunkFns::f_426fb06d685b, globals: FN_0_GLOBALS, callees: FN_0_CALLEES },
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
    sig_p_i32_i32_i32_i32_i32_i32_ret_i32: &[],
    sig_p_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i64_i32_i32_i32_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i64_i32_i32_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i64_i32_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i64_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_ret_i32: &[],
    sig_p_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i64_ret_i32: BOUND_SIG_P_I32_I32_I32_I32_I64_RET_I32,
    sig_p_i32_i32_i32_i32_ret_i32: BOUND_SIG_P_I32_I32_I32_I32_RET_I32,
    sig_p_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i64_ret_void: &[],
    sig_p_i32_i32_i32_ret_i32: BOUND_SIG_P_I32_I32_I32_RET_I32,
    sig_p_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i64_i32_i32_i64_i32_ret_i32: &[],
    sig_p_i32_i32_i64_i32_i32_ret_i32: &[],
    sig_p_i32_i32_i64_i32_ret_i32: &[],
    sig_p_i32_i32_i64_ret_i32: BOUND_SIG_P_I32_I32_I64_RET_I32,
    sig_p_i32_i32_i64_ret_void: &[],
    sig_p_i32_i32_ret_i32: BOUND_SIG_P_I32_I32_RET_I32,
    sig_p_i32_i32_ret_i64: BOUND_SIG_P_I32_I32_RET_I64,
    sig_p_i32_i32_ret_void: BOUND_SIG_P_I32_I32_RET_VOID,
    sig_p_i32_i64_i32_i32_ret_i32: BOUND_SIG_P_I32_I64_I32_I32_RET_I32,
    sig_p_i32_i64_i32_i32_ret_i64: &[],
    sig_p_i32_i64_i32_ret_i32: BOUND_SIG_P_I32_I64_I32_RET_I32,
    sig_p_i32_i64_i32_ret_i64: &[],
    sig_p_i32_i64_i32_ret_void: BOUND_SIG_P_I32_I64_I32_RET_VOID,
    sig_p_i32_i64_i64_ret_i32: BOUND_SIG_P_I32_I64_I64_RET_I32,
    sig_p_i32_i64_i64_ret_i64: BOUND_SIG_P_I32_I64_I64_RET_I64,
    sig_p_i32_i64_i64_ret_void: &[],
    sig_p_i32_i64_ret_i32: BOUND_SIG_P_I32_I64_RET_I32,
    sig_p_i32_i64_ret_i64: BOUND_SIG_P_I32_I64_RET_I64,
    sig_p_i32_i64_ret_void: &[],
    sig_p_i32_ret_i32: BOUND_SIG_P_I32_RET_I32,
    sig_p_i32_ret_i64: BOUND_SIG_P_I32_RET_I64,
    sig_p_i32_ret_void: &[],
    sig_p_i64_i32_ret_i64: BOUND_SIG_P_I64_I32_RET_I64,
    sig_p_i64_i64_ret_i32: BOUND_SIG_P_I64_I64_RET_I32,
    sig_p_i64_i64_ret_i64: BOUND_SIG_P_I64_I64_RET_I64,
    sig_p_i64_ret_i32: BOUND_SIG_P_I64_RET_I32,
    sig_p_i64_ret_i64: BOUND_SIG_P_I64_RET_I64,
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
    (2228243u128, [0x7e, 0x90, 0x3b, 0x63, 0x39, 0x6f, 0x1a, 0x12, 0xd5, 0x1d, 0xe5, 0x96, 0x15, 0x48, 0x05, 0x76, 0x88, 0xa7, 0xfa, 0x68, 0x80, 0x44, 0x04, 0xe1, 0x6c, 0xc3, 0xa1, 0xce, 0x3b, 0x75, 0x5f, 0xc4]),
];

const MODULE_META: SubstituteModuleMeta = SubstituteModuleMeta {
    name: "85900_04292/f19@l8590004775",
    accounting_version: 2u32,
    wasm_sha256: [0xf6, 0xf7, 0x62, 0x3d, 0xcd, 0xf4, 0xed, 0x49, 0xbe, 0x3c, 0xdb, 0xdb, 0xe6, 0xf2, 0xca, 0x07, 0xb6, 0x6b, 0x12, 0x8b, 0x9a, 0x27, 0x1d, 0x52, 0xe4, 0x38, 0xd9, 0x2d, 0x13, 0xdf, 0x55, 0xda],
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
    let g25: i32 = 0i32;
    let g26: i32 = 0i32;
    let g27: i32 = 0i32;
    let g28: i32 = 0i32;
    let g29: i32 = 0i32;
    let g30: i32 = 0i32;
    let g31: i32 = 0i32;
    let g32: i32 = 0i32;
    let g33: i32 = 256i32;
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
    vm.heap_ptr = g33;
    vm.has_heap_ptr = true;
    vm.globals_i32 = vec![g0, g1, g2, g3, g4, g5, g6, g7, g8, g9, g10, g11, g12, g13, g14, g15, g16, g17, g18, g19, g20, g21, g22, g23, g24, g25, g26, g27, g28, g29, g30, g31, g32];
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
            "__sl_inited" => self.vm.globals_i32.get(0usize).copied(),
            "__c_0" => self.vm.globals_i32.get(1usize).copied(),
            "__c_1" => self.vm.globals_i32.get(2usize).copied(),
            "__c_2" => self.vm.globals_i32.get(3usize).copied(),
            "__c_3" => self.vm.globals_i32.get(4usize).copied(),
            "__c_4" => self.vm.globals_i32.get(5usize).copied(),
            "__c_5" => self.vm.globals_i32.get(6usize).copied(),
            "__c_6" => self.vm.globals_i32.get(7usize).copied(),
            "__c_7" => self.vm.globals_i32.get(8usize).copied(),
            "__c_8" => self.vm.globals_i32.get(9usize).copied(),
            "__c_9" => self.vm.globals_i32.get(10usize).copied(),
            "__c_10" => self.vm.globals_i32.get(11usize).copied(),
            "__c_11" => self.vm.globals_i32.get(12usize).copied(),
            "__c_12" => self.vm.globals_i32.get(13usize).copied(),
            "__c_13" => self.vm.globals_i32.get(14usize).copied(),
            "__c_14" => self.vm.globals_i32.get(15usize).copied(),
            "__c_15" => self.vm.globals_i32.get(16usize).copied(),
            "__c_16" => self.vm.globals_i32.get(17usize).copied(),
            "__c_17" => self.vm.globals_i32.get(18usize).copied(),
            "__c_18" => self.vm.globals_i32.get(19usize).copied(),
            "__c_19" => self.vm.globals_i32.get(20usize).copied(),
            "__c_20" => self.vm.globals_i32.get(21usize).copied(),
            "__c_21" => self.vm.globals_i32.get(22usize).copied(),
            "__c_22" => self.vm.globals_i32.get(23usize).copied(),
            "__c_23" => self.vm.globals_i32.get(24usize).copied(),
            "__c_24" => self.vm.globals_i32.get(25usize).copied(),
            "__c_25" => self.vm.globals_i32.get(26usize).copied(),
            "__c_26" => self.vm.globals_i32.get(27usize).copied(),
            "__c_27" => self.vm.globals_i32.get(28usize).copied(),
            "__c_28" => self.vm.globals_i32.get(29usize).copied(),
            "__c_29" => self.vm.globals_i32.get(30usize).copied(),
            "__c_30" => self.vm.globals_i32.get(31usize).copied(),
            "__c_31" => self.vm.globals_i32.get(32usize).copied(),
            "__heap_ptr" => Some(self.vm.heap_ptr),
            _ => None,
        }
    }

    fn exported_global_i64(&self, name: &str) -> Option<i64> {
        match name {
            _ => None,
        }
    }

    fn set_heap_ptr(&mut self, val: i32) -> bool {
        self.vm.heap_ptr = val;
        true
    }

    fn call_start(&mut self, _host: &mut dyn WasmSubstituteHost) -> std::result::Result<(), WasmTrap> {
        Ok(())
    }

    fn call_entry(&mut self, in_ptr: i32, in_len: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
        let _ = (in_ptr, in_len);
        let vm = &mut self.vm;
        vm.enter_call()?;
        vm.begin_fast_epoch();
        let res = DISPATCH.call_direct_p_unit_ret_i32(vm, 2usize, host);
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

const FN_0_GLOBALS: &[usize] = &[0usize, 1usize, 2usize, 3usize, 4usize, 5usize, 6usize, 7usize, 8usize, 9usize, 10usize, 11usize, 12usize, 13usize, 14usize, 15usize, 16usize, 17usize, 18usize, 19usize, 20usize, 21usize, 22usize, 23usize, 24usize, 25usize, 26usize, 27usize, 28usize, 29usize, 30usize, 31usize, 32usize];
const FN_0_CALLEES: &[usize] = &[124usize];
const FN_1_CALLEES: &[usize] = &[0usize, 124usize];
const FN_2_CALLEES: &[usize] = &[0usize, 124usize];
const FN_3_CALLEES: &[usize] = &[0usize, 124usize];
const FN_4_CALLEES: &[usize] = &[0usize, 124usize];
const FN_5_CALLEES: &[usize] = &[0usize, 36usize, 9usize];
const FN_6_GLOBALS: &[usize] = &[14usize];
const FN_6_CALLEES: &[usize] = &[0usize, 120usize];
const FN_7_CALLEES: &[usize] = &[0usize, 0usize];
const FN_8_CALLEES: &[usize] = &[0usize, 9usize];
const FN_9_CALLEES: &[usize] = &[0usize, 121usize];
const FN_10_CALLEES: &[usize] = &[0usize, 121usize];
const FN_11_CALLEES: &[usize] = &[0usize, 30usize];
const FN_12_CALLEES: &[usize] = &[0usize, 120usize];
const FN_13_CALLEES: &[usize] = &[0usize, 124usize];
const FN_14_CALLEES: &[usize] = &[0usize, 124usize];
const FN_15_CALLEES: &[usize] = &[0usize, 124usize];
const FN_16_CALLEES: &[usize] = &[0usize, 12usize];
const FN_17_CALLEES: &[usize] = &[0usize, 12usize];
const FN_18_CALLEES: &[usize] = &[0usize, 12usize];
const FN_19_CALLEES: &[usize] = &[0usize];
const FN_20_CALLEES: &[usize] = &[0usize, 26usize];
const FN_21_CALLEES: &[usize] = &[0usize, 0usize, 0usize, 1usize, 1usize, 124usize];
const FN_22_CALLEES: &[usize] = &[0usize, 120usize];
const FN_23_CALLEES: &[usize] = &[0usize, 121usize];
const FN_24_CALLEES: &[usize] = &[0usize, 27usize];
const FN_25_CALLEES: &[usize] = &[0usize, 25usize];
const FN_26_CALLEES: &[usize] = &[0usize, 26usize];
const FN_27_CALLEES: &[usize] = &[0usize, 32usize, 122usize, 33usize, 1usize];
const FN_28_CALLEES: &[usize] = &[0usize, 33usize, 122usize, 32usize, 1usize];
const FN_29_CALLEES: &[usize] = &[0usize, 28usize];
const FN_30_CALLEES: &[usize] = &[0usize, 29usize];
const FN_31_CALLEES: &[usize] = &[0usize, 24usize];
const FN_32_CALLEES: &[usize] = &[0usize, 26usize];
const FN_33_GLOBALS: &[usize] = &[14usize, 9usize];
const FN_33_CALLEES: &[usize] = &[0usize, 36usize, 9usize, 124usize];
const FN_34_CALLEES: &[usize] = &[0usize, 31usize, 124usize];
const FN_35_GLOBALS: &[usize] = &[14usize, 9usize];
const FN_35_CALLEES: &[usize] = &[0usize, 4usize, 124usize];
const FN_36_GLOBALS: &[usize] = &[14usize, 9usize];
const FN_36_CALLEES: &[usize] = &[0usize, 0usize, 124usize];
const FN_37_CALLEES: &[usize] = &[0usize, 31usize, 124usize];
const FN_38_GLOBALS: &[usize] = &[14usize, 9usize];
const FN_38_CALLEES: &[usize] = &[0usize, 5usize, 35usize, 124usize];
const FN_39_GLOBALS: &[usize] = &[14usize, 9usize];
const FN_39_CALLEES: &[usize] = &[0usize, 124usize, 32usize];
const FN_40_GLOBALS: &[usize] = &[14usize, 9usize];
const FN_40_CALLEES: &[usize] = &[0usize, 124usize, 33usize];
const FN_41_GLOBALS: &[usize] = &[14usize, 9usize];
const FN_41_CALLEES: &[usize] = &[0usize, 9usize, 34usize, 124usize];
const FN_42_GLOBALS: &[usize] = &[9usize];
const FN_42_CALLEES: &[usize] = &[0usize, 8usize, 124usize];
const FN_43_GLOBALS: &[usize] = &[9usize];
const FN_43_CALLEES: &[usize] = &[0usize, 1usize, 124usize];
const FN_44_GLOBALS: &[usize] = &[9usize];
const FN_44_CALLEES: &[usize] = &[0usize, 9usize, 124usize];
const FN_45_GLOBALS: &[usize] = &[9usize];
const FN_45_CALLEES: &[usize] = &[0usize, 124usize];
const FN_46_GLOBALS: &[usize] = &[9usize];
const FN_46_CALLEES: &[usize] = &[0usize, 10usize, 124usize];
const FN_47_GLOBALS: &[usize] = &[14usize, 1usize];
const FN_47_CALLEES: &[usize] = &[0usize, 120usize, 124usize, 12usize, 125usize, 0usize, 32usize, 1usize, 9usize, 34usize, 37usize, 123usize];
const FN_48_CALLEES: &[usize] = &[0usize];
const FN_49_CALLEES: &[usize] = &[0usize, 124usize];
const FN_50_CALLEES: &[usize] = &[0usize, 1usize, 122usize, 0usize, 124usize];
const FN_51_CALLEES: &[usize] = &[0usize, 0usize, 122usize, 1usize, 124usize];
const FN_52_CALLEES: &[usize] = &[0usize, 124usize];
const FN_53_CALLEES: &[usize] = &[0usize, 0usize, 124usize];
const FN_54_CALLEES: &[usize] = &[0usize, 124usize];
const FN_55_CALLEES: &[usize] = &[0usize, 0usize, 122usize];
const FN_56_CALLEES: &[usize] = &[0usize, 1usize, 124usize, 122usize];
const FN_57_GLOBALS: &[usize] = &[14usize];
const FN_57_CALLEES: &[usize] = &[0usize, 21usize, 24usize, 34usize];
const FN_58_CALLEES: &[usize] = &[0usize, 124usize];
const FN_59_CALLEES: &[usize] = &[0usize, 124usize];
const FN_60_CALLEES: &[usize] = &[0usize, 124usize];
const FN_61_CALLEES: &[usize] = &[0usize, 124usize];
const FN_62_CALLEES: &[usize] = &[0usize, 70usize];
const FN_63_GLOBALS: &[usize] = &[19usize, 16usize, 17usize];
const FN_63_CALLEES: &[usize] = &[0usize, 124usize];
const FN_64_GLOBALS: &[usize] = &[19usize];
const FN_64_CALLEES: &[usize] = &[0usize, 124usize, 12usize, 5usize, 35usize, 21usize, 1usize, 11usize];
const FN_65_GLOBALS: &[usize] = &[16usize];
const FN_65_CALLEES: &[usize] = &[0usize, 124usize];
const FN_66_CALLEES: &[usize] = &[0usize, 68usize];
const FN_67_CALLEES: &[usize] = &[0usize, 24usize, 1usize];
const FN_68_CALLEES: &[usize] = &[0usize, 24usize, 2usize];
const FN_69_CALLEES: &[usize] = &[0usize, 66usize];
const FN_70_CALLEES: &[usize] = &[0usize, 124usize];
const FN_71_CALLEES: &[usize] = &[0usize, 124usize];
const FN_72_CALLEES: &[usize] = &[0usize, 2usize];
const FN_73_CALLEES: &[usize] = &[0usize, 124usize];
const FN_74_CALLEES: &[usize] = &[0usize, 124usize];
const FN_75_CALLEES: &[usize] = &[0usize, 124usize];
const FN_76_CALLEES: &[usize] = &[0usize, 124usize];
const FN_77_CALLEES: &[usize] = &[0usize, 50usize];
const FN_78_CALLEES: &[usize] = &[0usize, 50usize];
const FN_79_CALLEES: &[usize] = &[0usize, 124usize];
const FN_80_CALLEES: &[usize] = &[0usize];
const FN_81_CALLEES: &[usize] = &[0usize, 50usize];
const FN_82_CALLEES: &[usize] = &[0usize];
const FN_83_CALLEES: &[usize] = &[0usize, 62usize];
const FN_84_CALLEES: &[usize] = &[0usize, 124usize];
const FN_85_CALLEES: &[usize] = &[0usize, 26usize];
const FN_86_CALLEES: &[usize] = &[0usize, 24usize, 5usize];
const FN_87_CALLEES: &[usize] = &[0usize, 63usize];
const FN_88_CALLEES: &[usize] = &[0usize, 61usize];
const FN_89_CALLEES: &[usize] = &[0usize, 42usize, 67usize];
const FN_90_CALLEES: &[usize] = &[0usize, 64usize];
const FN_91_CALLEES: &[usize] = &[0usize, 65usize];
const FN_92_CALLEES: &[usize] = &[0usize, 60usize];
const FN_93_CALLEES: &[usize] = &[0usize, 1usize];
const FN_94_CALLEES: &[usize] = &[0usize, 67usize];
const FN_95_GLOBALS: &[usize] = &[19usize, 9usize];
const FN_95_CALLEES: &[usize] = &[0usize, 41usize, 70usize, 124usize];
const FN_96_GLOBALS: &[usize] = &[19usize, 1usize, 16usize, 17usize, 9usize];
const FN_96_CALLEES: &[usize] = &[0usize, 124usize, 124usize, 12usize, 31usize, 20usize, 21usize, 33usize];
const FN_97_GLOBALS: &[usize] = &[19usize, 9usize];
const FN_97_CALLEES: &[usize] = &[0usize, 20usize, 124usize];
const FN_98_CALLEES: &[usize] = &[0usize, 67usize, 21usize, 124usize, 70usize, 68usize];
const FN_99_GLOBALS: &[usize] = &[19usize, 9usize];
const FN_99_CALLEES: &[usize] = &[0usize, 43usize, 69usize, 124usize];
const FN_100_GLOBALS: &[usize] = &[16usize, 19usize, 9usize];
const FN_100_CALLEES: &[usize] = &[0usize, 124usize, 124usize, 33usize];
const FN_101_GLOBALS: &[usize] = &[19usize, 9usize];
const FN_101_CALLEES: &[usize] = &[0usize, 124usize, 24usize, 32usize, 20usize];
const FN_102_GLOBALS: &[usize] = &[19usize, 9usize];
const FN_102_CALLEES: &[usize] = &[0usize, 124usize, 0usize, 122usize, 1usize, 2usize];
const FN_103_GLOBALS: &[usize] = &[19usize, 9usize];
const FN_103_CALLEES: &[usize] = &[0usize, 44usize, 68usize, 124usize];
const FN_104_GLOBALS: &[usize] = &[9usize];
const FN_104_CALLEES: &[usize] = &[0usize, 28usize, 124usize, 11usize];
const FN_105_GLOBALS: &[usize] = &[9usize];
const FN_105_CALLEES: &[usize] = &[0usize, 124usize];
const FN_106_GLOBALS: &[usize] = &[14usize, 9usize];
const FN_106_CALLEES: &[usize] = &[0usize, 21usize, 124usize, 24usize];
const FN_107_GLOBALS: &[usize] = &[9usize];
const FN_107_CALLEES: &[usize] = &[0usize, 9usize, 124usize];
const FN_108_GLOBALS: &[usize] = &[9usize];
const FN_108_CALLEES: &[usize] = &[0usize, 124usize];
const FN_109_GLOBALS: &[usize] = &[9usize];
const FN_109_CALLEES: &[usize] = &[0usize, 10usize, 124usize];
const FN_110_GLOBALS: &[usize] = &[19usize, 1usize, 16usize, 17usize];
const FN_110_CALLEES: &[usize] = &[0usize, 124usize, 124usize, 12usize, 31usize, 20usize, 21usize, 33usize];
const FN_111_CALLEES: &[usize] = &[0usize, 24usize, 34usize, 20usize];
const FN_112_GLOBALS: &[usize] = &[19usize];
const FN_112_CALLEES: &[usize] = &[0usize, 68usize];
const FN_113_CALLEES: &[usize] = &[0usize, 24usize, 35usize, 20usize];
const FN_114_CALLEES: &[usize] = &[0usize, 24usize, 36usize, 20usize];
const FN_115_CALLEES: &[usize] = &[0usize, 124usize];
const FN_116_CALLEES: &[usize] = &[0usize];
const FN_117_CALLEES: &[usize] = &[0usize, 41usize];
const FN_118_CALLEES: &[usize] = &[0usize];
const FN_119_CALLEES: &[usize] = &[0usize];
const FN_120_CALLEES: &[usize] = &[0usize];
const FN_121_CALLEES: &[usize] = &[0usize, 95usize, 126usize];
const FN_122_CALLEES: &[usize] = &[0usize, 42usize, 90usize];
const FN_123_CALLEES: &[usize] = &[0usize, 125usize];
const FN_124_GLOBALS: &[usize] = &[22usize];
const FN_124_CALLEES: &[usize] = &[0usize, 9usize, 0usize, 124usize];
const FN_125_GLOBALS: &[usize] = &[22usize];
const FN_125_CALLEES: &[usize] = &[0usize, 124usize];
const FN_126_CALLEES: &[usize] = &[0usize, 36usize];
const FN_127_CALLEES: &[usize] = &[0usize, 9usize, 0usize, 124usize, 84usize];
const FN_128_CALLEES: &[usize] = &[0usize, 12usize, 38usize, 21usize, 84usize];
const FN_129_CALLEES: &[usize] = &[0usize, 4usize, 84usize, 0usize];
const FN_130_CALLEES: &[usize] = &[0usize, 5usize];
const FN_131_CALLEES: &[usize] = &[0usize, 100usize, 84usize];
const FN_132_CALLEES: &[usize] = &[0usize, 75usize];
const FN_133_CALLEES: &[usize] = &[0usize, 75usize];
const FN_134_CALLEES: &[usize] = &[0usize, 1usize, 12usize, 14usize, 1usize, 9usize, 0usize, 0usize, 1usize, 127usize, 84usize];
const FN_135_CALLEES: &[usize] = &[0usize];
const FN_136_CALLEES: &[usize] = &[0usize, 75usize];
const FN_137_CALLEES: &[usize] = &[0usize, 75usize];
const FN_138_GLOBALS: &[usize] = &[22usize];
const FN_138_CALLEES: &[usize] = &[0usize, 50usize, 124usize];
const FN_139_CALLEES: &[usize] = &[0usize, 93usize];
const FN_140_CALLEES: &[usize] = &[0usize, 125usize];
const FN_141_GLOBALS: &[usize] = &[22usize];
const FN_141_CALLEES: &[usize] = &[0usize, 41usize, 124usize];
const FN_142_CALLEES: &[usize] = &[0usize, 98usize, 84usize];
const FN_143_CALLEES: &[usize] = &[0usize, 92usize];
const FN_144_CALLEES: &[usize] = &[0usize, 94usize];
const FN_145_GLOBALS: &[usize] = &[22usize];
const FN_145_CALLEES: &[usize] = &[0usize, 96usize, 84usize, 95usize, 97usize];
const FN_146_CALLEES: &[usize] = &[0usize, 94usize];
const FN_147_GLOBALS: &[usize] = &[22usize, 1usize];
const FN_147_CALLEES: &[usize] = &[0usize, 41usize, 124usize, 100usize, 84usize];
const FN_148_CALLEES: &[usize] = &[0usize, 94usize, 34usize, 90usize, 124usize];
const FN_149_GLOBALS: &[usize] = &[19usize, 9usize, 16usize];
const FN_149_CALLEES: &[usize] = &[0usize, 51usize, 124usize, 1usize, 120usize, 25usize, 26usize];
const FN_150_GLOBALS: &[usize] = &[9usize];
const FN_150_CALLEES: &[usize] = &[0usize, 43usize, 124usize, 27usize];
const FN_151_GLOBALS: &[usize] = &[9usize];
const FN_151_CALLEES: &[usize] = &[0usize, 43usize, 124usize, 28usize];
const FN_152_GLOBALS: &[usize] = &[14usize, 9usize];
const FN_152_CALLEES: &[usize] = &[0usize, 124usize, 51usize];
const FN_153_GLOBALS: &[usize] = &[9usize];
const FN_153_CALLEES: &[usize] = &[0usize, 46usize, 124usize, 9usize];
const FN_154_GLOBALS: &[usize] = &[9usize];
const FN_154_CALLEES: &[usize] = &[0usize, 46usize, 124usize, 10usize];
const FN_155_GLOBALS: &[usize] = &[9usize];
const FN_155_CALLEES: &[usize] = &[0usize, 46usize, 124usize, 11usize];
const FN_156_GLOBALS: &[usize] = &[22usize, 1usize];
const FN_156_CALLEES: &[usize] = &[0usize, 41usize, 124usize, 100usize, 84usize];
const FN_157_CALLEES: &[usize] = &[0usize, 2usize, 15usize, 2usize];
const FN_158_GLOBALS: &[usize] = &[14usize];
const FN_158_CALLEES: &[usize] = &[0usize, 124usize, 15usize, 1usize, 1usize];
const FN_159_CALLEES: &[usize] = &[0usize, 2usize, 15usize];
const FN_160_CALLEES: &[usize] = &[0usize, 1usize, 12usize, 15usize, 9usize, 1usize, 0usize, 127usize];
const FN_161_CALLEES: &[usize] = &[0usize, 12usize, 15usize, 1usize, 2usize, 9usize, 1usize, 127usize, 50usize];
const FN_162_CALLEES: &[usize] = &[0usize, 124usize, 1usize, 16usize, 15usize, 14usize, 0usize, 9usize, 0usize, 0usize, 127usize, 50usize];
const FN_163_CALLEES: &[usize] = &[0usize, 14usize, 2usize, 15usize];
const FN_164_CALLEES: &[usize] = &[0usize, 95usize, 124usize, 15usize, 16usize, 2usize, 1usize, 9usize, 0usize, 127usize, 50usize, 1usize, 122usize, 0usize, 0usize, 127usize, 14usize, 128usize, 99usize];
const FN_165_CALLEES: &[usize] = &[0usize, 58usize];
const FN_166_CALLEES: &[usize] = &[0usize, 1usize, 124usize];
const FN_167_GLOBALS: &[usize] = &[23usize];
const FN_167_CALLEES: &[usize] = &[0usize, 56usize, 2usize, 9usize];
const FN_168_CALLEES: &[usize] = &[0usize, 124usize];
const FN_169_CALLEES: &[usize] = &[0usize];
const FN_170_CALLEES: &[usize] = &[0usize];
const FN_171_CALLEES: &[usize] = &[0usize, 124usize];
const FN_172_CALLEES: &[usize] = &[0usize, 1usize, 124usize];
const FN_173_GLOBALS: &[usize] = &[23usize];
const FN_173_CALLEES: &[usize] = &[0usize, 57usize, 2usize, 9usize];
const FN_174_CALLEES: &[usize] = &[0usize, 1usize];
const FN_175_CALLEES: &[usize] = &[0usize, 124usize];
const FN_176_CALLEES: &[usize] = &[0usize, 0usize, 13usize, 17usize, 129usize, 124usize, 2usize, 130usize, 131usize, 132usize, 1usize, 0usize, 127usize];
const FN_177_CALLEES: &[usize] = &[0usize, 3usize, 124usize, 17usize, 129usize, 1usize, 133usize, 132usize, 134usize, 0usize, 32usize, 1usize, 135usize, 121usize, 12usize, 55usize, 1usize];
const FN_178_GLOBALS: &[usize] = &[14usize];
const FN_178_CALLEES: &[usize] = &[0usize, 1usize, 9usize, 124usize, 32usize, 0usize, 1usize];
const FN_179_CALLEES: &[usize] = &[0usize, 7usize, 10usize, 3usize, 0usize, 0usize, 127usize];
const FN_180_CALLEES: &[usize] = &[0usize, 1usize, 7usize, 0usize, 127usize];
const FN_181_CALLEES: &[usize] = &[0usize, 59usize];
const FN_182_CALLEES: &[usize] = &[0usize, 60usize, 1usize, 122usize, 121usize, 1usize];
const FN_183_CALLEES: &[usize] = &[0usize, 3usize, 124usize, 18usize, 59usize, 1usize];
const FN_184_CALLEES: &[usize] = &[0usize, 18usize, 59usize, 124usize, 1usize, 5usize, 63usize, 7usize, 127usize, 122usize, 128usize, 136usize, 134usize, 3usize];
const FN_185_CALLEES: &[usize] = &[0usize, 1usize, 3usize, 124usize, 18usize, 59usize, 13usize, 127usize, 2usize];
const FN_186_CALLEES: &[usize] = &[0usize, 18usize, 59usize, 124usize, 7usize, 1usize, 127usize, 10usize, 3usize];
const FN_187_CALLEES: &[usize] = &[0usize, 1usize, 124usize, 18usize, 59usize, 56usize, 54usize, 7usize, 127usize, 122usize, 128usize, 3usize];
const FN_188_CALLEES: &[usize] = &[0usize, 18usize, 66usize, 1usize];
const FN_189_CALLEES: &[usize] = &[0usize, 18usize, 59usize, 1usize];
const FN_190_CALLEES: &[usize] = &[0usize, 59usize, 58usize, 124usize];
const FN_191_CALLEES: &[usize] = &[0usize, 1usize, 0usize, 13usize, 58usize, 2usize, 6usize, 4usize, 17usize, 134usize, 121usize, 136usize, 2usize, 127usize];
const FN_192_CALLEES: &[usize] = &[0usize, 0usize, 0usize];
const FN_193_CALLEES: &[usize] = &[0usize, 1usize, 13usize, 58usize, 2usize, 59usize, 127usize];
const FN_194_CALLEES: &[usize] = &[0usize, 1usize, 13usize, 58usize, 2usize, 17usize, 61usize, 59usize, 56usize, 127usize];
const FN_195_CALLEES: &[usize] = &[0usize, 67usize, 58usize, 124usize];
const FN_196_GLOBALS: &[usize] = &[14usize];
const FN_196_CALLEES: &[usize] = &[0usize, 120usize, 58usize, 53usize, 124usize, 13usize, 2usize, 12usize, 9usize, 121usize, 1usize, 33usize, 2usize, 14usize, 127usize];
const FN_197_CALLEES: &[usize] = &[0usize, 0usize, 0usize];
const FN_198_CALLEES: &[usize] = &[0usize, 124usize, 3usize, 137usize, 7usize, 17usize, 5usize, 0usize, 127usize];
const FN_199_CALLEES: &[usize] = &[0usize, 10usize, 3usize, 135usize, 0usize, 122usize, 121usize, 60usize, 1usize, 33usize, 128usize, 136usize];
const FN_200_CALLEES: &[usize] = &[0usize, 17usize, 129usize, 133usize, 132usize, 134usize];
const FN_201_CALLEES: &[usize] = &[0usize, 55usize];
const FN_202_CALLEES: &[usize] = &[0usize, 124usize, 2usize, 0usize, 1usize, 13usize, 4usize, 18usize, 127usize];
const FN_203_CALLEES: &[usize] = &[0usize, 124usize, 13usize, 18usize, 2usize, 1usize, 127usize, 2usize, 4usize, 138usize];
const FN_204_CALLEES: &[usize] = &[0usize, 18usize, 124usize, 3usize];
const FN_205_CALLEES: &[usize] = &[0usize, 4usize, 124usize];
const FN_206_CALLEES: &[usize] = &[0usize, 60usize];
const FN_207_CALLEES: &[usize] = &[0usize, 18usize, 59usize, 1usize, 1usize, 8usize];
const FN_208_CALLEES: &[usize] = &[0usize, 77usize];
const FN_209_CALLEES: &[usize] = &[0usize, 103usize];
const FN_210_CALLEES: &[usize] = &[0usize, 79usize, 13usize, 2usize, 1usize, 2usize, 127usize, 1usize, 4usize, 140usize, 124usize];
const FN_211_CALLEES: &[usize] = &[0usize, 129usize];
const FN_212_GLOBALS: &[usize] = &[3usize];
const FN_212_CALLEES: &[usize] = &[0usize, 64usize, 129usize, 62usize, 124usize];
const FN_213_CALLEES: &[usize] = &[0usize, 13usize, 59usize, 58usize, 2usize, 127usize];
const FN_214_GLOBALS: &[usize] = &[3usize];
const FN_214_CALLEES: &[usize] = &[0usize, 13usize, 18usize, 59usize, 1usize, 124usize, 2usize, 127usize];
const FN_215_CALLEES: &[usize] = &[0usize, 59usize];
const FN_216_CALLEES: &[usize] = &[0usize, 1usize, 18usize, 59usize, 78usize, 2usize, 0usize, 0usize, 1usize, 124usize];
const FN_217_CALLEES: &[usize] = &[0usize, 13usize, 18usize, 59usize, 1usize, 4usize, 2usize, 8usize, 124usize, 127usize];
const FN_218_CALLEES: &[usize] = &[0usize, 18usize, 1usize, 13usize, 4usize, 127usize];
const FN_219_CALLEES: &[usize] = &[0usize, 72usize, 124usize, 11usize, 9usize, 30usize];
const FN_220_CALLEES: &[usize] = &[0usize, 80usize, 124usize];
const FN_221_CALLEES: &[usize] = &[0usize, 124usize, 3usize, 24usize, 71usize];
const FN_222_CALLEES: &[usize] = &[0usize, 46usize, 71usize, 124usize];
const FN_223_CALLEES: &[usize] = &[0usize, 124usize, 3usize, 24usize, 71usize];
const FN_224_CALLEES: &[usize] = &[0usize, 124usize, 3usize, 71usize];
const FN_225_CALLEES: &[usize] = &[0usize, 72usize, 124usize, 11usize];
const FN_226_CALLEES: &[usize] = &[0usize, 80usize, 124usize];
const FN_227_CALLEES: &[usize] = &[0usize, 80usize, 124usize];
const FN_228_CALLEES: &[usize] = &[0usize, 80usize, 124usize];
const FN_229_CALLEES: &[usize] = &[0usize];
const FN_230_CALLEES: &[usize] = &[0usize, 124usize];
const FN_231_CALLEES: &[usize] = &[0usize];
const FN_232_CALLEES: &[usize] = &[0usize, 62usize, 124usize];
const FN_233_CALLEES: &[usize] = &[0usize, 64usize];
const FN_234_CALLEES: &[usize] = &[0usize, 80usize, 124usize, 101usize, 141usize, 3usize, 100usize, 76usize];
const FN_235_CALLEES: &[usize] = &[0usize, 77usize, 124usize, 102usize];
const FN_236_CALLEES: &[usize] = &[0usize, 124usize, 3usize, 100usize, 141usize, 76usize];
const FN_237_CALLEES: &[usize] = &[0usize, 77usize, 124usize, 103usize];
const FN_238_CALLEES: &[usize] = &[0usize, 124usize, 7usize, 72usize, 12usize, 10usize, 30usize, 27usize, 24usize];
const FN_239_CALLEES: &[usize] = &[0usize, 124usize, 7usize, 72usize, 9usize, 11usize, 30usize, 9usize, 0usize, 9usize];
const FN_240_CALLEES: &[usize] = &[0usize, 13usize, 124usize, 18usize, 71usize, 2usize, 127usize];
const FN_241_CALLEES: &[usize] = &[0usize, 124usize, 120usize, 71usize, 13usize, 10usize, 26usize, 24usize, 2usize, 25usize, 127usize, 99usize];
const FN_242_CALLEES: &[usize] = &[0usize, 124usize, 71usize, 13usize, 3usize, 2usize, 127usize, 99usize];
const FN_243_CALLEES: &[usize] = &[0usize, 7usize, 124usize, 72usize, 120usize, 9usize, 10usize, 9usize];
const FN_244_CALLEES: &[usize] = &[0usize, 7usize, 124usize, 72usize, 120usize, 9usize, 10usize, 24usize, 71usize, 10usize, 3usize];
const FN_245_CALLEES: &[usize] = &[0usize];
const FN_246_CALLEES: &[usize] = &[0usize, 130usize, 131usize, 133usize];
const FN_247_CALLEES: &[usize] = &[0usize, 1usize, 132usize];
const FN_248_CALLEES: &[usize] = &[0usize, 13usize, 72usize, 2usize, 19usize, 124usize, 71usize, 0usize, 120usize, 2usize, 111usize, 108usize, 113usize, 133usize, 0usize, 115usize, 116usize, 139usize, 127usize];
const FN_249_CALLEES: &[usize] = &[0usize, 18usize, 124usize];
const FN_250_CALLEES: &[usize] = &[0usize, 139usize, 124usize, 72usize, 13usize, 2usize, 19usize, 71usize, 0usize, 120usize, 2usize, 111usize, 108usize, 113usize, 133usize, 0usize, 115usize, 116usize];
const FN_251_CALLEES: &[usize] = &[0usize, 124usize, 0usize, 139usize];
const FN_252_GLOBALS: &[usize] = &[26usize];
const FN_252_CALLEES: &[usize] = &[0usize, 124usize, 13usize, 2usize, 127usize];
const FN_253_CALLEES: &[usize] = &[0usize, 110usize, 13usize, 124usize, 2usize, 139usize, 127usize];
const FN_254_CALLEES: &[usize] = &[0usize, 124usize, 71usize, 0usize];
const FN_255_CALLEES: &[usize] = &[0usize, 0usize, 124usize];
const FN_256_CALLEES: &[usize] = &[0usize, 124usize, 0usize, 139usize];
const FN_257_GLOBALS: &[usize] = &[24usize];
const FN_257_CALLEES: &[usize] = &[0usize, 59usize, 54usize, 72usize, 108usize];
const FN_258_GLOBALS: &[usize] = &[27usize, 28usize];
const FN_258_CALLEES: &[usize] = &[0usize, 9usize, 11usize];
const FN_259_CALLEES: &[usize] = &[0usize, 120usize];
const FN_260_CALLEES: &[usize] = &[0usize, 124usize, 0usize, 139usize, 105usize, 72usize, 108usize];
const FN_261_CALLEES: &[usize] = &[0usize, 124usize, 0usize, 13usize, 71usize, 2usize, 127usize];
const FN_262_CALLEES: &[usize] = &[0usize, 114usize, 132usize, 124usize, 4usize];
const FN_263_CALLEES: &[usize] = &[0usize, 13usize, 124usize, 2usize, 71usize, 139usize, 127usize, 117usize, 0usize, 3usize, 105usize];
const FN_264_CALLEES: &[usize] = &[0usize, 124usize, 13usize, 2usize, 4usize, 127usize, 117usize, 0usize, 139usize, 105usize, 72usize];
const FN_265_CALLEES: &[usize] = &[0usize, 13usize, 124usize, 2usize, 3usize, 4usize, 127usize, 117usize, 0usize, 139usize, 105usize, 72usize, 13usize];
const FN_266_GLOBALS: &[usize] = &[25usize];
const FN_266_CALLEES: &[usize] = &[0usize, 0usize, 59usize];
const FN_267_CALLEES: &[usize] = &[0usize, 2usize, 4usize, 124usize];
const FN_268_CALLEES: &[usize] = &[0usize, 17usize, 124usize];
const FN_269_CALLEES: &[usize] = &[0usize, 13usize, 85usize, 2usize, 127usize];
const FN_270_CALLEES: &[usize] = &[0usize, 112usize, 13usize, 4usize, 2usize, 127usize];
const FN_271_CALLEES: &[usize] = &[0usize, 124usize, 105usize, 0usize, 139usize, 3usize, 13usize, 2usize, 127usize];
const FN_272_GLOBALS: &[usize] = &[25usize];
const FN_272_CALLEES: &[usize] = &[0usize, 0usize, 59usize];
const FN_273_GLOBALS: &[usize] = &[24usize, 29usize];
const FN_273_CALLEES: &[usize] = &[0usize, 59usize, 54usize, 72usize, 9usize];
const FN_274_GLOBALS: &[usize] = &[25usize];
const FN_274_CALLEES: &[usize] = &[0usize, 0usize, 59usize];
const FN_275_CALLEES: &[usize] = &[0usize, 13usize, 4usize, 124usize, 139usize, 2usize, 127usize];
const FN_276_CALLEES: &[usize] = &[0usize, 124usize, 0usize, 139usize];
const FN_277_CALLEES: &[usize] = &[0usize, 18usize, 124usize, 110usize];
const FN_278_CALLEES: &[usize] = &[0usize, 18usize, 124usize, 13usize, 4usize, 2usize, 127usize, 1usize];
const FN_279_CALLEES: &[usize] = &[0usize, 3usize, 2usize, 124usize, 64usize, 74usize, 142usize, 105usize, 62usize, 106usize, 0usize];
const FN_280_GLOBALS: &[usize] = &[24usize, 25usize];
const FN_280_CALLEES: &[usize] = &[0usize, 115usize, 124usize, 13usize, 0usize, 59usize, 2usize, 107usize, 72usize, 112usize, 114usize, 133usize, 114usize, 111usize, 116usize, 117usize, 116usize, 12usize, 4usize, 127usize, 54usize, 104usize, 112usize];
const FN_281_GLOBALS: &[usize] = &[25usize];
const FN_281_CALLEES: &[usize] = &[0usize, 0usize, 59usize, 113usize];
const FN_282_CALLEES: &[usize] = &[0usize, 72usize, 0usize];
const FN_283_GLOBALS: &[usize] = &[27usize, 28usize];
const FN_283_CALLEES: &[usize] = &[0usize, 72usize, 9usize, 11usize];
const FN_284_GLOBALS: &[usize] = &[25usize];
const FN_284_CALLEES: &[usize] = &[0usize, 0usize, 59usize];
const FN_285_GLOBALS: &[usize] = &[25usize];
const FN_285_CALLEES: &[usize] = &[0usize, 0usize, 59usize];
const FN_286_CALLEES: &[usize] = &[0usize, 92usize, 122usize, 90usize];
const FN_287_GLOBALS: &[usize] = &[30usize, 31usize, 32usize];
const FN_287_CALLEES: &[usize] = &[0usize, 10usize, 9usize, 11usize];
const FN_288_CALLEES: &[usize] = &[0usize, 124usize, 13usize, 2usize, 0usize, 127usize];
const FN_289_CALLEES: &[usize] = &[0usize, 0usize, 18usize, 124usize, 120usize];
const FN_290_CALLEES: &[usize] = &[0usize, 0usize, 18usize, 124usize, 120usize, 13usize, 4usize, 2usize, 127usize, 1usize];
const FN_291_CALLEES: &[usize] = &[0usize, 124usize, 0usize, 59usize];
const FN_292_CALLEES: &[usize] = &[0usize, 7usize, 72usize, 124usize, 10usize, 11usize, 3usize, 127usize];
const FN_293_CALLEES: &[usize] = &[0usize, 71usize, 124usize, 2usize, 72usize, 108usize, 113usize, 133usize, 121usize, 108usize];
const FN_294_CALLEES: &[usize] = &[0usize, 124usize, 0usize, 72usize, 108usize, 5usize];
const FN_295_CALLEES: &[usize] = &[0usize, 124usize, 1usize, 6usize];
const FN_296_CALLEES: &[usize] = &[0usize, 71usize, 2usize, 124usize, 111usize, 72usize, 108usize, 113usize, 133usize, 120usize, 13usize, 0usize, 4usize, 127usize];
const FN_297_GLOBALS: &[usize] = &[30usize, 31usize, 32usize];
const FN_297_CALLEES: &[usize] = &[0usize, 72usize, 11usize, 9usize, 124usize, 118usize, 109usize, 119usize, 12usize, 71usize, 1usize, 0usize, 112usize, 119usize];
const FN_298_GLOBALS: &[usize] = &[30usize];
const FN_298_CALLEES: &[usize] = &[0usize, 1usize, 2usize, 124usize, 0usize, 139usize, 72usize, 10usize, 1usize, 13usize, 2usize, 4usize, 4usize, 127usize, 11usize, 59usize, 0usize, 110usize, 133usize, 6usize];
const FN_299_CALLEES: &[usize] = &[0usize, 93usize, 118usize, 94usize, 124usize];
const FN_300_CALLEES: &[usize] = &[124usize];
const FN_321_CALLEES: &[usize] = &[126usize];
const FN_323_CALLEES: &[usize] = &[124usize];
const FN_326_CALLEES: &[usize] = &[124usize];
const FN_330_CALLEES: &[usize] = &[124usize];
const FN_331_CALLEES: &[usize] = &[124usize];
const FN_334_CALLEES: &[usize] = &[124usize];
const FN_337_CALLEES: &[usize] = &[124usize];
const FN_344_CALLEES: &[usize] = &[124usize];
const FN_348_CALLEES: &[usize] = &[124usize];
const FN_349_CALLEES: &[usize] = &[124usize];
const FN_351_CALLEES: &[usize] = &[124usize];
const FN_352_CALLEES: &[usize] = &[124usize];
const FN_353_CALLEES: &[usize] = &[139usize];
const FN_355_CALLEES: &[usize] = &[138usize];
const FN_356_CALLEES: &[usize] = &[124usize];
const FN_358_CALLEES: &[usize] = &[124usize];
const FN_362_CALLEES: &[usize] = &[0usize, 124usize, 123usize];
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
    crate::Bound_p_i32_i32_i32_i32_i32_i32_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_174dfb33e5ba, globals: &[], callees: FN_265_CALLEES },
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
static BOUND_SIG_P_I32_I32_I32_I32_I64_RET_I32: &[crate::Bound_p_i32_i32_i32_i32_i64_ret_i32] = &[
    crate::Bound_p_i32_i32_i32_i32_i64_ret_i32 { func: crate::c_85900_04292::ChunkFns::f_1cf4620fa532, globals: &[], callees: FN_296_CALLEES },
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
    crate::Bound_p_i32_i32_i32_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_186d1c0e26c0, globals: &[], callees: FN_255_CALLEES },
    crate::Bound_p_i32_i32_i32_i32_ret_i32 { func: crate::c_85900_04292::ChunkFns::f_37bc21062338, globals: &[], callees: FN_290_CALLEES },
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
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_0ba96dccaf47, globals: FN_196_GLOBALS, callees: FN_196_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_99792::ChunkFns::f_5cbcd199a7c2, globals: &[], callees: FN_250_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_e8fc0c2f6c4c, globals: &[], callees: FN_261_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_20ed2f71bb81, globals: &[], callees: FN_275_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85899_99792::ChunkFns::f_8cad486356a1, globals: FN_280_GLOBALS, callees: FN_280_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85900_04292::ChunkFns::f_b4a27025f49f, globals: &[], callees: FN_293_CALLEES },
    crate::Bound_p_i32_i32_i32_ret_i32 { func: crate::c_85900_04292::ChunkFns::f_be76f48db77a, globals: &[], callees: FN_294_CALLEES },
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
static BOUND_SIG_P_I32_I32_I64_RET_I32: &[crate::Bound_p_i32_i32_i64_ret_i32] = &[
    crate::Bound_p_i32_i32_i64_ret_i32 { func: crate::c_85900_04292::ChunkFns::f_f2aa3d0555d6, globals: FN_297_GLOBALS, callees: FN_297_CALLEES },
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
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_27aaa3cb9114, globals: &[], callees: FN_1_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f2340190950c, globals: &[], callees: FN_3_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_0ece3b1734d5, globals: &[], callees: FN_4_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_803fb230173c, globals: &[], callees: FN_5_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_8ed247cfa309, globals: FN_6_GLOBALS, callees: FN_6_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_a9acae168912, globals: &[], callees: FN_7_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_dcce91d2e133, globals: &[], callees: FN_8_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_11_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_1e6ff4c83b62, globals: &[], callees: FN_12_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_da0e89bc9783, globals: &[], callees: FN_16_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_6df8689f059b, globals: &[], callees: FN_17_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f9ce687d90ae, globals: &[], callees: FN_18_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_30ec60d4302a, globals: &[], callees: FN_19_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_20_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_da0e89bc9783, globals: &[], callees: FN_22_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_24_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_25_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_26_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7e58af41e27d, globals: &[], callees: FN_27_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7e58af41e27d, globals: &[], callees: FN_28_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_29_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_30_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_31_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_32_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_ace0563243a6, globals: FN_33_GLOBALS, callees: FN_33_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_c69686cacf82, globals: &[], callees: FN_34_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_1a5f3f7571e2, globals: &[], callees: FN_37_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f1c0a657afaa, globals: FN_38_GLOBALS, callees: FN_38_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_acb2a186ecef, globals: FN_39_GLOBALS, callees: FN_39_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_acb2a186ecef, globals: FN_40_GLOBALS, callees: FN_40_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f1c0a657afaa, globals: FN_41_GLOBALS, callees: FN_41_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_5cfd1c966ee0, globals: FN_47_GLOBALS, callees: FN_47_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_c1a365c57784, globals: &[], callees: FN_50_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f15f52fd10ac, globals: &[], callees: FN_51_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_a07d59a0459c, globals: &[], callees: FN_52_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f5ab0dd84882, globals: &[], callees: FN_53_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_b9ec3478b1be, globals: &[], callees: FN_54_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_998c31a576d2, globals: &[], callees: FN_56_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_27aaa3cb9114, globals: &[], callees: FN_58_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f2340190950c, globals: &[], callees: FN_60_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_0ece3b1734d5, globals: &[], callees: FN_61_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f8850b1df958, globals: &[], callees: FN_62_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_609b70a572c5, globals: FN_63_GLOBALS, callees: FN_63_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f148b5f5c14f, globals: FN_64_GLOBALS, callees: FN_64_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_e23aae24b862, globals: &[], callees: FN_66_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_69_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_1e6ff4c83b62, globals: &[], callees: FN_70_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_da0e89bc9783, globals: &[], callees: FN_77_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_6df8689f059b, globals: &[], callees: FN_78_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f9ce687d90ae, globals: &[], callees: FN_81_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7ec8bf846782, globals: &[], callees: FN_82_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_83_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_da0e89bc9783, globals: &[], callees: FN_84_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_87_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_88_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_293ae2a09ea3, globals: &[], callees: FN_89_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_90_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_91_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_92_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_94_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f1c0a657afaa, globals: FN_95_GLOBALS, callees: FN_95_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_28a9f52b5bce, globals: FN_96_GLOBALS, callees: FN_96_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_4cf93470be08, globals: &[], callees: FN_98_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f1c0a657afaa, globals: FN_99_GLOBALS, callees: FN_99_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_1acc39445a44, globals: FN_101_GLOBALS, callees: FN_101_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_5230ac976967, globals: FN_102_GLOBALS, callees: FN_102_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f1c0a657afaa, globals: FN_103_GLOBALS, callees: FN_103_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_3c62ec56fda8, globals: FN_110_GLOBALS, callees: FN_110_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f90a3e72a759, globals: &[], callees: FN_111_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f90a3e72a759, globals: &[], callees: FN_113_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f90a3e72a759, globals: &[], callees: FN_114_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_84192::ChunkFns::f_4f573e6f88cb, globals: &[], callees: FN_116_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_67329667d7fd, globals: &[], callees: FN_117_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_84192::ChunkFns::f_4f573e6f88cb, globals: &[], callees: FN_118_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_84192::ChunkFns::f_4f573e6f88cb, globals: &[], callees: FN_120_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_ca8d871e077c, globals: &[], callees: FN_121_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_194f062f76fc, globals: &[], callees: FN_122_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_1e6ff4c83b62, globals: &[], callees: FN_123_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_3594571ef794, globals: &[], callees: FN_131_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_e02bb7a5ce3b, globals: &[], callees: FN_132_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_868a5fcb144d, globals: &[], callees: FN_133_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_94c31aaa12fa, globals: &[], callees: FN_134_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_c56c009f41de, globals: &[], callees: FN_136_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_cfe54f4f1606, globals: &[], callees: FN_137_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_da603e0969cb, globals: FN_138_GLOBALS, callees: FN_138_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_139_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_da0e89bc9783, globals: &[], callees: FN_140_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_0364226cb1e8, globals: &[], callees: FN_142_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_143_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_144_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_40871f360722, globals: FN_145_GLOBALS, callees: FN_145_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7c1b3db89f36, globals: &[], callees: FN_146_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_4a4ae773c3d4, globals: FN_147_GLOBALS, callees: FN_147_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_1592393fe606, globals: &[], callees: FN_148_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_04c43103713a, globals: FN_156_GLOBALS, callees: FN_156_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_2af4fe9e7d26, globals: &[], callees: FN_159_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_8a603a474721, globals: &[], callees: FN_160_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_08e07943209e, globals: &[], callees: FN_161_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_188a11564f14, globals: &[], callees: FN_162_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_d3a85f461ba8, globals: &[], callees: FN_163_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_a70135e4cf19, globals: &[], callees: FN_164_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_b238ded6c3fe, globals: &[], callees: FN_193_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_fbaa521b0850, globals: &[], callees: FN_195_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_9b9e4d79c2a8, globals: &[], callees: FN_202_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_f008f64e784a, globals: &[], callees: FN_203_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_dcce91d2e133, globals: &[], callees: FN_209_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_59a0410eeb26, globals: &[], callees: FN_210_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_17e4d117a9e2, globals: &[], callees: FN_246_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_c83374c783a1, globals: &[], callees: FN_247_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_99792::ChunkFns::f_5450ef596911, globals: &[], callees: FN_248_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_f9611536c652, globals: &[], callees: FN_251_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_b732433177f0, globals: &[], callees: FN_254_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_0f8beff50967, globals: &[], callees: FN_259_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_426d859a938f, globals: &[], callees: FN_262_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_2d42b20c8b6d, globals: &[], callees: FN_263_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_bbee40c093b4, globals: &[], callees: FN_264_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_027e6ffc1a1a, globals: &[], callees: FN_270_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_92e33a06770e, globals: &[], callees: FN_279_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85900_04292::ChunkFns::f_bd36f6dae8b0, globals: &[], callees: FN_289_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85900_04292::ChunkFns::f_eeae7770e899, globals: &[], callees: FN_295_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_2::ChunkFns::f_03e481579af4, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_1::ChunkFns::f_c6e97ee70046, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_5015758bb709, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_3948a84700b8, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_2::ChunkFns::f_03e481579af4, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_2::ChunkFns::f_162b60a1a7e7, globals: &[], callees: FN_321_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_11c79f5620b0, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_ea4faf22326d, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_a43a5ae7b21f, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_2c680f88091b, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_140fb33a5afe, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_32917fc773d9, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_7d1f4d7fcad4, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_fa57b0c2b6ab, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_a74c85b451e0, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_3f75c31e0aa0, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_aba70ab0b976, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_deffe42432b7, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_f96a9e02e90e, globals: &[], callees: FN_353_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_88792_2::ChunkFns::f_03e481579af4, globals: &[], callees: &[] },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_f96a9e02e90e, globals: &[], callees: FN_355_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_ec7fee50bb79, globals: &[], callees: FN_358_CALLEES },
    crate::Bound_p_i32_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_5591fa286389, globals: &[], callees: &[] },
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
static BOUND_SIG_P_I32_I32_RET_I64: &[crate::Bound_p_i32_i32_ret_i64] = &[
    crate::Bound_p_i32_i32_ret_i64 { func: crate::c_85899_88792::ChunkFns::f_3abc88984867, globals: &[], callees: FN_55_CALLEES },
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
    crate::Bound_p_i32_i32_ret_void { func: crate::c_85899_88792::ChunkFns::f_c55926dd782d, globals: &[], callees: FN_199_CALLEES },
    crate::Bound_p_i32_i32_ret_void { func: crate::c_85899_83092_2::ChunkFns::f_71d8a2d6235d, globals: &[], callees: FN_326_CALLEES },
    crate::Bound_p_i32_i32_ret_void { func: crate::c_85899_83092_2::ChunkFns::f_71d8a2d6235d, globals: &[], callees: FN_337_CALLEES },
    crate::Bound_p_i32_i32_ret_void { func: crate::c_85899_83092_2::ChunkFns::f_e79c4e116bfd, globals: &[], callees: FN_344_CALLEES },
    crate::Bound_p_i32_i32_ret_void { func: crate::c_85899_83092_2::ChunkFns::f_bc6c808270d1, globals: &[], callees: FN_348_CALLEES },
    crate::Bound_p_i32_i32_ret_void { func: crate::c_85899_88792::ChunkFns::f_8fa562cbc853, globals: &[], callees: FN_351_CALLEES },
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
static BOUND_SIG_P_I32_I64_I32_I32_RET_I32: &[crate::Bound_p_i32_i64_i32_i32_ret_i32] = &[
    crate::Bound_p_i32_i64_i32_i32_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_c3c4818e5a41, globals: &[], callees: FN_278_CALLEES },
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
static BOUND_SIG_P_I32_I64_I32_RET_I32: &[crate::Bound_p_i32_i64_i32_ret_i32] = &[
    crate::Bound_p_i32_i64_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_b953838600c1, globals: &[], callees: FN_249_CALLEES },
    crate::Bound_p_i32_i64_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_6cb216adc685, globals: &[], callees: FN_268_CALLEES },
    crate::Bound_p_i32_i64_i32_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_c7ad525e5a02, globals: &[], callees: FN_277_CALLEES },
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
static BOUND_SIG_P_I32_I64_I32_RET_VOID: &[crate::Bound_p_i32_i64_i32_ret_void] = &[
    crate::Bound_p_i32_i64_i32_ret_void { func: crate::c_85899_83092_2::ChunkFns::f_0a0fecb79fc7, globals: &[], callees: &[] },
    crate::Bound_p_i32_i64_i32_ret_void { func: crate::c_85899_83092_2::ChunkFns::f_0a0fecb79fc7, globals: &[], callees: &[] },
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
    crate::Bound_p_i32_i64_i64_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_b6cf541461dc, globals: &[], callees: FN_205_CALLEES },
    crate::Bound_p_i32_i64_i64_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_a4773ed743de, globals: &[], callees: FN_218_CALLEES },
    crate::Bound_p_i32_i64_i64_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_9adecba56606, globals: &[], callees: FN_330_CALLEES },
    crate::Bound_p_i32_i64_i64_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_9607d677c9db, globals: &[], callees: FN_349_CALLEES },
    crate::Bound_p_i32_i64_i64_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_9adecba56606, globals: &[], callees: FN_352_CALLEES },
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
static BOUND_SIG_P_I32_I64_I64_RET_I64: &[crate::Bound_p_i32_i64_i64_ret_i64] = &[
    crate::Bound_p_i32_i64_i64_ret_i64 { func: crate::c_85899_88792::ChunkFns::f_31af32cb558c, globals: &[], callees: FN_200_CALLEES },
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
static BOUND_SIG_P_I32_I64_RET_I32: &[crate::Bound_p_i32_i64_ret_i32] = &[
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f0da2366c908, globals: &[], callees: FN_177_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_88792_2::ChunkFns::f_7503dfb81265, globals: &[], callees: FN_183_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_deaf01e451f9, globals: &[], callees: FN_184_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_88792_2::ChunkFns::f_ea9ae49cf4e8, globals: &[], callees: FN_185_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_88792_2::ChunkFns::f_a73ab5eebd75, globals: &[], callees: FN_186_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_b96282108807, globals: &[], callees: FN_187_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_419c292e1d24, globals: &[], callees: FN_197_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_a996482bc1ef, globals: &[], callees: FN_204_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_3de4de76d7bc, globals: &[], callees: FN_216_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_7a27ed8e21d5, globals: &[], callees: FN_238_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_ee91944eafa8, globals: &[], callees: FN_239_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_f2d9b324c003, globals: &[], callees: FN_260_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_94f57c3a085c, globals: &[], callees: FN_271_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_5ec036ffae9b, globals: &[], callees: FN_276_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_a4027a2778a0, globals: &[], callees: &[] },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_a4027a2778a0, globals: &[], callees: &[] },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_e849c87b3008, globals: &[], callees: FN_331_CALLEES },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_87279bd968a2, globals: &[], callees: &[] },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_a4027a2778a0, globals: &[], callees: &[] },
    crate::Bound_p_i32_i64_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_a4027a2778a0, globals: &[], callees: &[] },
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
static BOUND_SIG_P_I32_I64_RET_I64: &[crate::Bound_p_i32_i64_ret_i64] = &[
    crate::Bound_p_i32_i64_ret_i64 { func: crate::c_85899_88792::ChunkFns::f_2bae3ae8abdb, globals: &[], callees: FN_188_CALLEES },
    crate::Bound_p_i32_i64_ret_i64 { func: crate::c_85899_88792::ChunkFns::f_241fa345f31b, globals: &[], callees: FN_189_CALLEES },
    crate::Bound_p_i32_i64_ret_i64 { func: crate::c_85899_92792_3::ChunkFns::f_112be49a4c6a, globals: FN_266_GLOBALS, callees: FN_266_CALLEES },
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
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_1d111f94255c, globals: &[], callees: FN_2_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7eb29a59d915, globals: &[], callees: FN_9_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_fab6c7792bec, globals: &[], callees: FN_10_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_a45a26c14c17, globals: &[], callees: FN_14_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_8f0ec661a804, globals: &[], callees: FN_15_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_3979fdeb44f5, globals: &[], callees: FN_23_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_de050b4b81f1, globals: FN_35_GLOBALS, callees: FN_35_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_327504b23599, globals: FN_42_GLOBALS, callees: FN_42_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_703887b4ca0f, globals: FN_43_GLOBALS, callees: FN_43_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_671a96cacef8, globals: FN_44_GLOBALS, callees: FN_44_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f1674ef8dccc, globals: FN_45_GLOBALS, callees: FN_45_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_e75c24ce5706, globals: FN_46_GLOBALS, callees: FN_46_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_7d101c2d913a, globals: FN_57_GLOBALS, callees: FN_57_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_1d111f94255c, globals: &[], callees: FN_59_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_c65ede4802f8, globals: FN_65_GLOBALS, callees: FN_65_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_005520f212f5, globals: &[], callees: FN_67_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_005520f212f5, globals: &[], callees: FN_68_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_1e881702eb44, globals: &[], callees: FN_72_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_a45a26c14c17, globals: &[], callees: FN_74_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_8f0ec661a804, globals: &[], callees: FN_75_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_83092::ChunkFns::f_0e5bb44a1e78, globals: &[], callees: FN_76_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_0cbcbc1ef76e, globals: &[], callees: FN_80_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_9d92c7701c67, globals: &[], callees: FN_85_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_005520f212f5, globals: &[], callees: FN_86_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_49de3a4260ff, globals: &[], callees: FN_93_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_e32d590b954e, globals: FN_97_GLOBALS, callees: FN_97_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_329f5ce01511, globals: FN_100_GLOBALS, callees: FN_100_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_dbe6bf1cc46b, globals: FN_104_GLOBALS, callees: FN_104_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_0fabceb05d99, globals: FN_105_GLOBALS, callees: FN_105_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_b6a8cfd6ecb8, globals: FN_106_GLOBALS, callees: FN_106_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_671a96cacef8, globals: FN_107_GLOBALS, callees: FN_107_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f1674ef8dccc, globals: FN_108_GLOBALS, callees: FN_108_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_e75c24ce5706, globals: FN_109_GLOBALS, callees: FN_109_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_b828c3bb90f0, globals: FN_112_GLOBALS, callees: FN_112_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_83092::ChunkFns::f_70644cf2f7cf, globals: &[], callees: FN_115_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_84192::ChunkFns::f_7931de08c6d9, globals: &[], callees: FN_119_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_64d7099dbc77, globals: FN_125_GLOBALS, callees: FN_125_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_358231fcf579, globals: &[], callees: FN_126_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_d421c25ba8cd, globals: &[], callees: FN_127_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_22fbbfe4115b, globals: &[], callees: FN_128_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_1e881702eb44, globals: &[], callees: FN_130_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_f1d61f915b91, globals: &[], callees: FN_135_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_f6699ee6758d, globals: FN_141_GLOBALS, callees: FN_141_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_56b1ab258317, globals: FN_149_GLOBALS, callees: FN_149_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_e8394947df8a, globals: FN_150_GLOBALS, callees: FN_150_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_a01f25005cd0, globals: FN_151_GLOBALS, callees: FN_151_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_ac9ed96b8296, globals: FN_152_GLOBALS, callees: FN_152_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_e8394947df8a, globals: FN_153_GLOBALS, callees: FN_153_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_a01f25005cd0, globals: FN_154_GLOBALS, callees: FN_154_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_3f199480863a, globals: FN_155_GLOBALS, callees: FN_155_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_4374122c356c, globals: &[], callees: FN_157_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_93af815d1803, globals: FN_158_GLOBALS, callees: FN_158_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_2a1df693f599, globals: &[], callees: FN_165_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_428301bbe702, globals: &[], callees: FN_166_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_16b21026a7f2, globals: FN_167_GLOBALS, callees: FN_167_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_83092::ChunkFns::f_0e5bb44a1e78, globals: &[], callees: FN_168_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_ffdee7c93cbe, globals: &[], callees: FN_169_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_3eb654e927a1, globals: &[], callees: FN_170_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_3927290801c7, globals: &[], callees: FN_172_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_d89853f7c7c5, globals: FN_173_GLOBALS, callees: FN_173_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_49de3a4260ff, globals: &[], callees: FN_174_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_a45a26c14c17, globals: &[], callees: FN_175_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_5e048dbe7c1f, globals: &[], callees: FN_176_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_f26f1b9b1ce2, globals: FN_178_GLOBALS, callees: FN_178_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_a4b8b05c5fad, globals: &[], callees: FN_179_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_45817b7043dc, globals: &[], callees: FN_180_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_fdb0906f56bb, globals: &[], callees: FN_181_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_bdc4fb2aaab1, globals: &[], callees: FN_190_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_990600c4a836, globals: &[], callees: FN_191_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_34a49f345dca, globals: &[], callees: FN_194_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_2::ChunkFns::f_638e3fd61a80, globals: &[], callees: FN_198_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_2::ChunkFns::f_61a16d9614ba, globals: &[], callees: FN_201_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_2::ChunkFns::f_61a16d9614ba, globals: &[], callees: FN_206_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_9d92c7701c67, globals: &[], callees: FN_208_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_1::ChunkFns::f_db0ffa8054bf, globals: &[], callees: FN_211_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_cd58da62b9e3, globals: FN_212_GLOBALS, callees: FN_212_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_a2186d19e2c0, globals: &[], callees: FN_213_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_aa32df195e2f, globals: FN_214_GLOBALS, callees: FN_214_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_5e4fbe80a2d9, globals: &[], callees: FN_215_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_e4afd653eea2, globals: &[], callees: FN_217_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_03c6e9c99b64, globals: &[], callees: FN_219_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_038735e9e9c7, globals: &[], callees: FN_220_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_7425067462bd, globals: &[], callees: FN_221_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_a4564f09a357, globals: &[], callees: FN_222_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_975343290a87, globals: &[], callees: FN_223_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_2e9ae6967fad, globals: &[], callees: FN_224_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_fec92329e980, globals: &[], callees: FN_225_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_87a4332dcdea, globals: &[], callees: FN_226_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_d725ff05410f, globals: &[], callees: FN_227_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_3505cdd4951f, globals: &[], callees: FN_228_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792::ChunkFns::f_7c75ee00bddb, globals: &[], callees: FN_229_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792::ChunkFns::f_49eed9e9e18f, globals: &[], callees: FN_230_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792::ChunkFns::f_7c75ee00bddb, globals: &[], callees: FN_231_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_97d4a47a22ec, globals: &[], callees: FN_232_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_88792_2::ChunkFns::f_61a16d9614ba, globals: &[], callees: FN_233_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_9b4a1c598462, globals: &[], callees: FN_234_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_e3f6ab9ba400, globals: &[], callees: FN_235_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_a2a7ff38a2b7, globals: &[], callees: FN_236_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_d9ea1df92c5e, globals: &[], callees: FN_237_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_9c15bf600887, globals: &[], callees: FN_240_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_47fe2304968c, globals: &[], callees: FN_241_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_94d168266957, globals: &[], callees: FN_242_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_093751662340, globals: &[], callees: FN_243_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_e59a8e4dd5a8, globals: &[], callees: FN_244_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_6da45e6623e3, globals: &[], callees: FN_245_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_d53d5875197a, globals: &[], callees: FN_253_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_dd64d7da387f, globals: &[], callees: FN_256_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_ef0263fa8fe1, globals: FN_257_GLOBALS, callees: FN_257_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_e35c1d781ec7, globals: FN_258_GLOBALS, callees: FN_258_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_47d0a89ab3c3, globals: &[], callees: FN_267_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_fef4b833a275, globals: &[], callees: FN_269_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_df0fa22086ad, globals: FN_272_GLOBALS, callees: FN_272_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_02bd3f85ecb6, globals: FN_273_GLOBALS, callees: FN_273_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_3f42f48f24b8, globals: FN_274_GLOBALS, callees: FN_274_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_2::ChunkFns::f_9704072dbea0, globals: FN_281_GLOBALS, callees: FN_281_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_9c0d878b1a6c, globals: FN_283_GLOBALS, callees: FN_283_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_cec45d7b411d, globals: FN_284_GLOBALS, callees: FN_284_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_ea528067816b, globals: FN_285_GLOBALS, callees: FN_285_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85900_04292::ChunkFns::f_94115c07ddc1, globals: &[], callees: FN_286_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85900_04292::ChunkFns::f_05be71efd2df, globals: FN_287_GLOBALS, callees: FN_287_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85900_04292::ChunkFns::f_81ab99176846, globals: &[], callees: FN_291_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85900_04292::ChunkFns::f_bf98b8c73ee9, globals: &[], callees: FN_292_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85900_04292::ChunkFns::f_e759dd99c9c3, globals: FN_298_GLOBALS, callees: FN_298_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_38535251f258, globals: &[], callees: FN_299_CALLEES },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_83092_1::ChunkFns::f_5e1271fe89af, globals: &[], callees: &[] },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_3dd4bf2481ba, globals: &[], callees: &[] },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_c47bb8df9be0, globals: &[], callees: &[] },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_cbe0184bd9b5, globals: &[], callees: &[] },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_1a71d29336df, globals: &[], callees: &[] },
    crate::Bound_p_i32_ret_i32 { func: crate::c_85899_92792_1::ChunkFns::f_8b3529200404, globals: &[], callees: FN_356_CALLEES },
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
static BOUND_SIG_P_I32_RET_I64: &[crate::Bound_p_i32_ret_i64] = &[
    crate::Bound_p_i32_ret_i64 { func: crate::c_85899_88792_2::ChunkFns::f_cb1aa58541c6, globals: &[], callees: FN_48_CALLEES },
    crate::Bound_p_i32_ret_i64 { func: crate::c_85899_88792::ChunkFns::f_6f07359304f0, globals: &[], callees: FN_182_CALLEES },
    crate::Bound_p_i32_ret_i64 { func: crate::c_85899_92792_2::ChunkFns::f_c1c69edd038d, globals: &[], callees: FN_207_CALLEES },
    crate::Bound_p_i32_ret_i64 { func: crate::c_85899_92792_3::ChunkFns::f_7b2b3771f8f7, globals: &[], callees: FN_282_CALLEES },
    crate::Bound_p_i32_ret_i64 { func: crate::c_85899_83092_2::ChunkFns::f_c239f066ccbc, globals: &[], callees: &[] },
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
static BOUND_SIG_P_I64_I32_RET_I64: &[crate::Bound_p_i64_i32_ret_i64] = &[
    crate::Bound_p_i64_i32_ret_i64 { func: crate::c_85899_83092_1::ChunkFns::f_79815729aec3, globals: &[], callees: &[] },
    crate::Bound_p_i64_i32_ret_i64 { func: crate::c_85899_83092_1::ChunkFns::f_16e5e69025c5, globals: &[], callees: &[] },
    crate::Bound_p_i64_i32_ret_i64 { func: crate::c_85899_83092_2::ChunkFns::f_53b864d8a1de, globals: &[], callees: &[] },
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
static BOUND_SIG_P_I64_I64_RET_I32: &[crate::Bound_p_i64_i64_ret_i32] = &[
    crate::Bound_p_i64_i64_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_470053343055, globals: &[], callees: FN_21_CALLEES },
    crate::Bound_p_i64_i64_ret_i32 { func: crate::c_85899_83092::ChunkFns::f_bb85b13eb8ff, globals: &[], callees: FN_49_CALLEES },
    crate::Bound_p_i64_i64_ret_i32 { func: crate::c_85899_83092::ChunkFns::f_bb85b13eb8ff, globals: &[], callees: FN_79_CALLEES },
    crate::Bound_p_i64_i64_ret_i32 { func: crate::c_85899_83092::ChunkFns::f_bb85b13eb8ff, globals: &[], callees: FN_171_CALLEES },
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
static BOUND_SIG_P_I64_I64_RET_I64: &[crate::Bound_p_i64_i64_ret_i64] = &[
    crate::Bound_p_i64_i64_ret_i64 { func: crate::c_85899_83092_1::ChunkFns::f_2b59315b7818, globals: &[], callees: &[] },
    crate::Bound_p_i64_i64_ret_i64 { func: crate::c_85899_83092_1::ChunkFns::f_0de929be56dc, globals: &[], callees: &[] },
    crate::Bound_p_i64_i64_ret_i64 { func: crate::c_85899_83092_2::ChunkFns::f_a5a79dab7242, globals: &[], callees: &[] },
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
static BOUND_SIG_P_I64_RET_I32: &[crate::Bound_p_i64_ret_i32] = &[
    crate::Bound_p_i64_ret_i32 { func: crate::c_85899_83092::ChunkFns::f_3002f0cd1024, globals: &[], callees: FN_13_CALLEES },
    crate::Bound_p_i64_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_8d4ef5bbec94, globals: FN_36_GLOBALS, callees: FN_36_CALLEES },
    crate::Bound_p_i64_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_16a5c648c247, globals: &[], callees: FN_71_CALLEES },
    crate::Bound_p_i64_ret_i32 { func: crate::c_85899_83092::ChunkFns::f_3002f0cd1024, globals: &[], callees: FN_73_CALLEES },
    crate::Bound_p_i64_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_956ac04c06f0, globals: FN_124_GLOBALS, callees: FN_124_CALLEES },
    crate::Bound_p_i64_ret_i32 { func: crate::c_85899_88792_1::ChunkFns::f_a80be209ca01, globals: &[], callees: FN_129_CALLEES },
    crate::Bound_p_i64_ret_i32 { func: crate::c_85899_88792::ChunkFns::f_4ce1c3955399, globals: &[], callees: FN_192_CALLEES },
    crate::Bound_p_i64_ret_i32 { func: crate::c_85899_83092_1::ChunkFns::f_ee190614c423, globals: &[], callees: FN_300_CALLEES },
    crate::Bound_p_i64_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_15ed6333638e, globals: &[], callees: &[] },
    crate::Bound_p_i64_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_a95199099e27, globals: &[], callees: &[] },
    crate::Bound_p_i64_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_609444ff4048, globals: &[], callees: &[] },
    crate::Bound_p_i64_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_f36d2b83b7ff, globals: &[], callees: &[] },
    crate::Bound_p_i64_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_3961cde2a530, globals: &[], callees: FN_323_CALLEES },
    crate::Bound_p_i64_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_3961cde2a530, globals: &[], callees: FN_334_CALLEES },
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
static BOUND_SIG_P_I64_RET_I64: &[crate::Bound_p_i64_ret_i64] = &[
    crate::Bound_p_i64_ret_i64 { func: crate::c_85899_83092_2::ChunkFns::f_554f71c39a7a, globals: &[], callees: &[] },
    crate::Bound_p_i64_ret_i64 { func: crate::c_85899_83092_2::ChunkFns::f_554f71c39a7a, globals: &[], callees: &[] },
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
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_92792_3::ChunkFns::f_ce4a4f4eaaf0, globals: FN_252_GLOBALS, callees: FN_252_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85900_04292::ChunkFns::f_e6879a687a42, globals: &[], callees: FN_288_CALLEES },
    crate::Bound_p_unit_ret_i32 { func: crate::c_85899_83092_2::ChunkFns::f_a6a8842e4b04, globals: &[], callees: FN_362_CALLEES },
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
    crate::Bound_p_unit_ret_void { func: crate::c_85900_04292::ChunkFns::f_426fb06d685b, globals: FN_0_GLOBALS, callees: FN_0_CALLEES },
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
    sig_p_i32_i32_i32_i32_i32_i32_ret_i32: &[],
    sig_p_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i64_i32_i32_i32_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i64_i32_i32_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i64_i32_i32_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_i64_ret_void: &[],
    sig_p_i32_i32_i32_i32_i32_ret_i32: &[],
    sig_p_i32_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i32_i64_ret_i32: BOUND_SIG_P_I32_I32_I32_I32_I64_RET_I32,
    sig_p_i32_i32_i32_i32_ret_i32: BOUND_SIG_P_I32_I32_I32_I32_RET_I32,
    sig_p_i32_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i32_i64_ret_void: &[],
    sig_p_i32_i32_i32_ret_i32: BOUND_SIG_P_I32_I32_I32_RET_I32,
    sig_p_i32_i32_i32_ret_void: &[],
    sig_p_i32_i32_i64_i32_i32_i64_i32_ret_i32: &[],
    sig_p_i32_i32_i64_i32_i32_ret_i32: &[],
    sig_p_i32_i32_i64_i32_ret_i32: &[],
    sig_p_i32_i32_i64_ret_i32: BOUND_SIG_P_I32_I32_I64_RET_I32,
    sig_p_i32_i32_i64_ret_void: &[],
    sig_p_i32_i32_ret_i32: BOUND_SIG_P_I32_I32_RET_I32,
    sig_p_i32_i32_ret_i64: BOUND_SIG_P_I32_I32_RET_I64,
    sig_p_i32_i32_ret_void: BOUND_SIG_P_I32_I32_RET_VOID,
    sig_p_i32_i64_i32_i32_ret_i32: BOUND_SIG_P_I32_I64_I32_I32_RET_I32,
    sig_p_i32_i64_i32_i32_ret_i64: &[],
    sig_p_i32_i64_i32_ret_i32: BOUND_SIG_P_I32_I64_I32_RET_I32,
    sig_p_i32_i64_i32_ret_i64: &[],
    sig_p_i32_i64_i32_ret_void: BOUND_SIG_P_I32_I64_I32_RET_VOID,
    sig_p_i32_i64_i64_ret_i32: BOUND_SIG_P_I32_I64_I64_RET_I32,
    sig_p_i32_i64_i64_ret_i64: BOUND_SIG_P_I32_I64_I64_RET_I64,
    sig_p_i32_i64_i64_ret_void: &[],
    sig_p_i32_i64_ret_i32: BOUND_SIG_P_I32_I64_RET_I32,
    sig_p_i32_i64_ret_i64: BOUND_SIG_P_I32_I64_RET_I64,
    sig_p_i32_i64_ret_void: &[],
    sig_p_i32_ret_i32: BOUND_SIG_P_I32_RET_I32,
    sig_p_i32_ret_i64: BOUND_SIG_P_I32_RET_I64,
    sig_p_i32_ret_void: &[],
    sig_p_i64_i32_ret_i64: BOUND_SIG_P_I64_I32_RET_I64,
    sig_p_i64_i64_ret_i32: BOUND_SIG_P_I64_I64_RET_I32,
    sig_p_i64_i64_ret_i64: BOUND_SIG_P_I64_I64_RET_I64,
    sig_p_i64_ret_i32: BOUND_SIG_P_I64_RET_I32,
    sig_p_i64_ret_i64: BOUND_SIG_P_I64_RET_I64,
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
    (2228243u128, [0x3d, 0xdf, 0xdb, 0x53, 0x17, 0xf8, 0xe0, 0xf9, 0x51, 0x5f, 0xa6, 0x31, 0x0a, 0x58, 0xa4, 0x1f, 0x84, 0xdb, 0x96, 0x46, 0x80, 0xb0, 0x1f, 0x6c, 0xc3, 0x46, 0xbc, 0xab, 0x18, 0xc5, 0xb7, 0x24]),
];

const MODULE_META: SubstituteModuleMeta = SubstituteModuleMeta {
    name: "85900_04292/f19@l8590004986",
    accounting_version: 2u32,
    wasm_sha256: [0xcc, 0x57, 0x40, 0x22, 0xf8, 0xf7, 0xb7, 0xb0, 0x36, 0x82, 0x38, 0xd3, 0xeb, 0x48, 0x8d, 0x40, 0xf0, 0x14, 0xb6, 0x5b, 0x19, 0x9a, 0xe9, 0x1b, 0x82, 0x3f, 0x42, 0x03, 0x1d, 0xc4, 0x75, 0x90],
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
    let g25: i32 = 0i32;
    let g26: i32 = 0i32;
    let g27: i32 = 0i32;
    let g28: i32 = 0i32;
    let g29: i32 = 0i32;
    let g30: i32 = 0i32;
    let g31: i32 = 0i32;
    let g32: i32 = 0i32;
    let g33: i32 = 256i32;
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
    vm.heap_ptr = g33;
    vm.has_heap_ptr = true;
    vm.globals_i32 = vec![g0, g1, g2, g3, g4, g5, g6, g7, g8, g9, g10, g11, g12, g13, g14, g15, g16, g17, g18, g19, g20, g21, g22, g23, g24, g25, g26, g27, g28, g29, g30, g31, g32];
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
            "__sl_inited" => self.vm.globals_i32.get(0usize).copied(),
            "__c_0" => self.vm.globals_i32.get(1usize).copied(),
            "__c_1" => self.vm.globals_i32.get(2usize).copied(),
            "__c_2" => self.vm.globals_i32.get(3usize).copied(),
            "__c_3" => self.vm.globals_i32.get(4usize).copied(),
            "__c_4" => self.vm.globals_i32.get(5usize).copied(),
            "__c_5" => self.vm.globals_i32.get(6usize).copied(),
            "__c_6" => self.vm.globals_i32.get(7usize).copied(),
            "__c_7" => self.vm.globals_i32.get(8usize).copied(),
            "__c_8" => self.vm.globals_i32.get(9usize).copied(),
            "__c_9" => self.vm.globals_i32.get(10usize).copied(),
            "__c_10" => self.vm.globals_i32.get(11usize).copied(),
            "__c_11" => self.vm.globals_i32.get(12usize).copied(),
            "__c_12" => self.vm.globals_i32.get(13usize).copied(),
            "__c_13" => self.vm.globals_i32.get(14usize).copied(),
            "__c_14" => self.vm.globals_i32.get(15usize).copied(),
            "__c_15" => self.vm.globals_i32.get(16usize).copied(),
            "__c_16" => self.vm.globals_i32.get(17usize).copied(),
            "__c_17" => self.vm.globals_i32.get(18usize).copied(),
            "__c_18" => self.vm.globals_i32.get(19usize).copied(),
            "__c_19" => self.vm.globals_i32.get(20usize).copied(),
            "__c_20" => self.vm.globals_i32.get(21usize).copied(),
            "__c_21" => self.vm.globals_i32.get(22usize).copied(),
            "__c_22" => self.vm.globals_i32.get(23usize).copied(),
            "__c_23" => self.vm.globals_i32.get(24usize).copied(),
            "__c_24" => self.vm.globals_i32.get(25usize).copied(),
            "__c_25" => self.vm.globals_i32.get(26usize).copied(),
            "__c_26" => self.vm.globals_i32.get(27usize).copied(),
            "__c_27" => self.vm.globals_i32.get(28usize).copied(),
            "__c_28" => self.vm.globals_i32.get(29usize).copied(),
            "__c_29" => self.vm.globals_i32.get(30usize).copied(),
            "__c_30" => self.vm.globals_i32.get(31usize).copied(),
            "__c_31" => self.vm.globals_i32.get(32usize).copied(),
            "__heap_ptr" => Some(self.vm.heap_ptr),
            _ => None,
        }
    }

    fn exported_global_i64(&self, name: &str) -> Option<i64> {
        match name {
            _ => None,
        }
    }

    fn set_heap_ptr(&mut self, val: i32) -> bool {
        self.vm.heap_ptr = val;
        true
    }

    fn call_start(&mut self, _host: &mut dyn WasmSubstituteHost) -> std::result::Result<(), WasmTrap> {
        Ok(())
    }

    fn call_entry(&mut self, in_ptr: i32, in_len: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
        let _ = (in_ptr, in_len);
        let vm = &mut self.vm;
        vm.enter_call()?;
        vm.begin_fast_epoch();
        let res = DISPATCH.call_direct_p_unit_ret_i32(vm, 2usize, host);
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

