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

//! Generated shared Merkle-deduplicated WASM substitute functions (shard 0).
//! Do not edit by hand; regenerate via `./scripts/generate-wasm-substitutes`.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

#[allow(
    unused_imports,
    reason = "Generated shared substitute runtime imports"
)]
use ctb_wasm_substitute_rt::{
    SubstituteVm, WasmSubstituteHost, WasmTrap, ops,
};

/// Shared Merkle-deduplicated WebAssembly function container.
pub(crate) struct Shared;

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
impl Shared {
    #[inline(never)]
    pub(crate) fn s_04f5a9f59b67(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i32 = 0i32;
    let mut l7: i32 = 0i32;
    let mut l8: i64 = 0i64;
    let mut l9: i64 = 0i64;
    let mut l10: i64 = 0i64;
    let mut l11: i64 = 0i64;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i32_2: i32 = 0i32;
    let mut s_i64_2: i64 = 0i64;
    let mut s_i64_3: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l1;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
    s_i64_1 = 5i64;
    s_i64_0 = ops::i64_div_u(s_i64_0, s_i64_1)?;
    s_i64_1 = 2i64;
    vm.enter_call()?;
    let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i64_0 = __t0;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_c5da14f612f0(vm, s_i64_0, host);
    vm.exit_call();
    let __t1 = __call_res?;
    s_i32_0 = __t1;
    l2 = s_i32_0;
    s_i32_0 = l1;
    l3 = s_i32_0;
    s_i32_0 = l3;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    l4 = s_i32_0;
    s_i32_0 = l3;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    l5 = s_i32_0;
    s_i32_0 = 0i32;
    l6 = s_i32_0;
    'blk_1: {
        'blk_2: loop {
            vm.deduct(host, 1)?;
            s_i32_0 = l6;
            s_i32_1 = l5;
            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
            if s_i32_0 != 0 {
                break 'blk_1;
            }
            s_i32_0 = l4;
            s_i32_1 = l6;
            s_i32_2 = 1i32;
            s_i32_1 = ops::i32_mul(s_i32_1, s_i32_2);
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
            l7 = s_i32_0;
            'blk_3: {
                s_i32_0 = l7;
                s_i32_1 = 255i32;
                s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
                l8 = s_i64_0;
                s_i64_0 = 0i64;
                l9 = s_i64_0;
                s_i32_0 = l2;
                s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
                s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
                l10 = s_i64_0;
                'blk_4: {
                    'blk_5: loop {
                        vm.deduct(host, 1)?;
                        s_i64_0 = l9;
                        s_i64_1 = l10;
                        s_i32_0 = ops::i64_lt_u(s_i64_0, s_i64_1);
                        s_i32_0 = ops::i32_eqz(s_i32_0);
                        if s_i32_0 != 0 {
                            break 'blk_4;
                        }
                        s_i32_0 = l2;
                        s_i64_1 = l9;
                        vm.enter_call()?;
                        let __call_res = Self::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
                        vm.exit_call();
                        let __t2 = __call_res?;
                        s_i32_0 = __t2;
                        s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
                        s_i32_1 = 6i32;
                        vm.enter_call()?;
                        let __call_res = Self::s_52dcb6d97135(vm, s_i64_0, s_i32_1, host);
                        vm.exit_call();
                        let __t3 = __call_res?;
                        s_i64_0 = __t3;
                        s_i64_1 = l8;
                        vm.enter_call()?;
                        let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
                        vm.exit_call();
                        let __t4 = __call_res?;
                        s_i64_0 = __t4;
                        l11 = s_i64_0;
                        s_i32_0 = l2;
                        s_i64_1 = l9;
                        s_i64_2 = l11;
                        s_i64_3 = 4294967295i64;
                        s_i64_2 = ops::i64_and(s_i64_2, s_i64_3);
                        vm.enter_call()?;
                        let __call_res = crate::shared_1::Shared::s_d5f50d7e651d(vm, s_i64_2, host);
                        vm.exit_call();
                        let __t5 = __call_res?;
                        s_i32_2 = __t5;
                        vm.enter_call()?;
                        let __call_res = crate::shared_1::Shared::s_d7d900550897(vm, s_i32_0, s_i64_1, s_i32_2, host);
                        vm.exit_call();
                        __call_res?;
                        s_i64_0 = l11;
                        s_i32_1 = 32i32;
                        vm.enter_call()?;
                        let __call_res = Self::s_73a866edda27(vm, s_i64_0, s_i32_1, host);
                        vm.exit_call();
                        let __t6 = __call_res?;
                        s_i64_0 = __t6;
                        l8 = s_i64_0;
                        s_i64_0 = l9;
                        s_i64_1 = 1i64;
                        vm.enter_call()?;
                        let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
                        vm.exit_call();
                        let __t7 = __call_res?;
                        s_i64_0 = __t7;
                        l9 = s_i64_0;
                        continue 'blk_5;
                        break 'blk_5;
                    }
                }
                s_i64_0 = l8;
                s_i64_1 = 0i64;
                s_i32_0 = ops::i64_ne(s_i64_0, s_i64_1);
                'blk_6: {
                    if s_i32_0 != 0 {
                        s_i32_0 = l2;
                        s_i64_1 = l8;
                        vm.enter_call()?;
                        let __call_res = crate::shared_1::Shared::s_d5f50d7e651d(vm, s_i64_1, host);
                        vm.exit_call();
                        let __t8 = __call_res?;
                        s_i32_1 = __t8;
                        vm.enter_call()?;
                        let __call_res = Self::s_697cb42b9ce7(vm, s_i32_0, s_i32_1, host);
                        vm.exit_call();
                        __call_res?;
                    } else {
                    }
                }
            }
            s_i32_0 = l6;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l6 = s_i32_0;
            continue 'blk_2;
            break 'blk_2;
        }
    }
    s_i32_0 = l0;
    s_i32_1 = l2;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_df3679579582(vm, s_i32_1, host);
    vm.exit_call();
    let __t9 = __call_res?;
    s_i32_1 = __t9;
    let __t10 = s_i32_0;
    let __t11 = s_i32_1;
    return Self::s_482c16ff60d0(vm, __t10, __t11, g0, host);
    }

    pub(crate) fn s_062991301687(vm: &mut SubstituteVm, p0: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    s_i32_0 = l0;
    s_i32_1 = i32::MIN;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = 0i32;
    s_i32_1 = l0;
    s_i32_0 = ops::i32_sub(s_i32_0, s_i32_1);
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn s_0e12a2af4587(vm: &mut SubstituteVm, p0: i32, g0: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = 0i32;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i32 = 0i32;
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
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_b4afe44dab61(vm, s_i32_0, g0, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l1 = s_i32_0;
    s_i32_0 = l1;
    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
    s_i32_0 = ops::i32_eqz(s_i32_0);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            l2 = s_i32_0;
            s_i32_0 = 0i32;
            l3 = s_i32_0;
            s_i32_0 = vm.get_i32(g0)?;
            l4 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            l5 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = l2;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l5;
            s_i32_1 = l3;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l5;
            s_i32_1 = l4;
            ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l5;
            l6 = s_i32_0;
            s_i32_0 = l6;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i32_0 = l1;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
    l7 = s_i64_0;
    s_i64_0 = l7;
    s_i64_1 = -2147483648i64;
    s_i32_0 = ops::i64_ge_s(s_i64_0, s_i64_1);
    'blk_2: {
        if s_i32_0 != 0 {
            s_i64_0 = l7;
            s_i64_1 = 2147483647i64;
            s_i32_0 = ops::i64_le_s(s_i64_0, s_i64_1);
        } else {
            s_i32_0 = 0i32;
        }
    }
    'blk_3: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
            l8 = s_i32_0;
            s_i64_0 = l7;
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_862b469aad5b(vm, s_i64_0, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_0 = __t2;
            l9 = s_i32_0;
            s_i32_0 = 0i32;
            l10 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i32_0 = __t3;
            l11 = s_i32_0;
            s_i32_0 = l11;
            s_i32_1 = l8;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l11;
            s_i32_1 = l9;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l11;
            s_i32_1 = l10;
            ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l11;
        } else {
            s_i32_0 = 0i32;
            l12 = s_i32_0;
            s_i32_0 = 0i32;
            l13 = s_i32_0;
            s_i32_0 = vm.get_i32(g0)?;
            l14 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t4 = __call_res?;
            s_i32_0 = __t4;
            l15 = s_i32_0;
            s_i32_0 = l15;
            s_i32_1 = l12;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l15;
            s_i32_1 = l13;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l15;
            s_i32_1 = l14;
            ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l15;
        }
    }
    l16 = s_i32_0;
    s_i32_0 = l16;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn s_0f7538eaa7db(vm: &mut SubstituteVm, p0: i32, g0: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i64 = 0i64;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i64 = 0i64;
    let mut l7: i64 = 0i64;
    let mut l8: i64 = 0i64;
    let mut l9: i64 = 0i64;
    let mut l10: i32 = 0i32;
    let mut l11: i32 = 0i32;
    let mut l12: i32 = 0i32;
    let mut l13: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i32_2: i32 = 0i32;
    let mut s_i64_2: i64 = 0i64;
    let mut s_i32_3: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
    l1 = s_i64_0;
    s_i64_0 = l1;
    s_i64_1 = 4i64;
    s_i32_0 = ops::i64_gt_u(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            l2 = s_i32_0;
            s_i32_0 = vm.get_i32(g0)?;
            l3 = s_i32_0;
            s_i32_0 = 8i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t0 = __call_res?;
            s_i32_0 = __t0;
            l4 = s_i32_0;
            s_i32_0 = l4;
            s_i32_1 = l2;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l4;
            s_i32_1 = l3;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l4;
            l5 = s_i32_0;
            s_i32_0 = l5;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i64_0 = l1;
    s_i64_1 = 0i64;
    s_i32_0 = ops::i64_gt_u(s_i64_0, s_i64_1);
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i64_1 = 0i64;
            vm.enter_call()?;
            let __call_res = Self::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
        } else {
            s_i64_0 = 0i64;
        }
    }
    l6 = s_i64_0;
    s_i64_0 = l1;
    s_i64_1 = 1i64;
    s_i32_0 = ops::i64_gt_u(s_i64_0, s_i64_1);
    'blk_3: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i64_1 = 1i64;
            vm.enter_call()?;
            let __call_res = Self::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_0 = __t2;
            s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
        } else {
            s_i64_0 = 0i64;
        }
    }
    l7 = s_i64_0;
    s_i64_0 = l1;
    s_i64_1 = 2i64;
    s_i32_0 = ops::i64_gt_u(s_i64_0, s_i64_1);
    'blk_4: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i64_1 = 2i64;
            vm.enter_call()?;
            let __call_res = Self::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i32_0 = __t3;
            s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
        } else {
            s_i64_0 = 0i64;
        }
    }
    l8 = s_i64_0;
    s_i64_0 = l1;
    s_i64_1 = 3i64;
    s_i32_0 = ops::i64_gt_u(s_i64_0, s_i64_1);
    'blk_5: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i64_1 = 3i64;
            vm.enter_call()?;
            let __call_res = Self::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
            vm.exit_call();
            let __t4 = __call_res?;
            s_i32_0 = __t4;
            s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
        } else {
            s_i64_0 = 0i64;
        }
    }
    l9 = s_i64_0;
    s_i32_0 = 1i32;
    l10 = s_i32_0;
    s_i64_0 = l6;
    s_i64_1 = l7;
    s_i32_2 = 32i32;
    vm.enter_call()?;
    let __call_res = Self::s_52dcb6d97135(vm, s_i64_1, s_i32_2, host);
    vm.exit_call();
    let __t5 = __call_res?;
    s_i64_1 = __t5;
    s_i64_0 = ops::i64_or(s_i64_0, s_i64_1);
    s_i64_1 = l8;
    s_i64_2 = l9;
    s_i32_3 = 32i32;
    vm.enter_call()?;
    let __call_res = Self::s_52dcb6d97135(vm, s_i64_2, s_i32_3, host);
    vm.exit_call();
    let __t6 = __call_res?;
    s_i64_2 = __t6;
    s_i64_1 = ops::i64_or(s_i64_1, s_i64_2);
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_b3e6ca84b2de(vm, s_i64_0, s_i64_1, host);
    vm.exit_call();
    let __t7 = __call_res?;
    s_i32_0 = __t7;
    l11 = s_i32_0;
    s_i32_0 = 8i32;
    vm.enter_call()?;
    let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
    vm.exit_call();
    let __t8 = __call_res?;
    s_i32_0 = __t8;
    l12 = s_i32_0;
    s_i32_0 = l12;
    s_i32_1 = l10;
    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l12;
    s_i32_1 = l11;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l12;
    l13 = s_i32_0;
    s_i32_0 = l13;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_1010eaf60e55(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, g1: usize, g2: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_1 = l1;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_ab8268aad544(vm, s_i32_0, s_i32_1, g0, g1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
            l2 = s_i32_0;
            s_i32_0 = l0;
            s_i32_1 = l1;
            vm.enter_call()?;
            let __call_res = Self::s_69a15786a590(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            l3 = s_i32_0;
            s_i32_0 = 0i32;
            l4 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_0 = __t2;
            l5 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = l2;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l5;
            s_i32_1 = l3;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l5;
            s_i32_1 = l4;
            ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l5;
        } else {
            s_i32_0 = 0i32;
            l6 = s_i32_0;
            s_i32_0 = vm.get_i32(g0)?;
            l7 = s_i32_0;
            s_i32_0 = vm.get_i32(g2)?;
            l8 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i32_0 = __t3;
            l9 = s_i32_0;
            s_i32_0 = l9;
            s_i32_1 = l6;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l9;
            s_i32_1 = l7;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l9;
            s_i32_1 = l8;
            ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l9;
        }
    }
    l10 = s_i32_0;
    s_i32_0 = l10;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_118c96141dcc(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = 1i32;
    l2 = s_i32_0;
    s_i32_0 = l2;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn s_1230bc006319(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<(), WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i64 = 0i64;
    let mut l7: i32 = 0i32;
    let mut l8: i32 = 0i32;
    let mut l9: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i32_2: i32 = 0i32;
    let mut s_i32_3: i32 = 0i32;
    vm.deduct(host, 1)?;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 12u32)?;
    s_i32_1 = 0i32;
    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = l1;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    l2 = s_i32_0;
    s_i32_0 = l2;
    s_i32_1 = 0i32;
    s_i32_0 = ops::i32_gt_u(s_i32_0, s_i32_1);
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
            l3 = s_i32_0;
            s_i32_0 = l0;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
            l4 = s_i32_0;
            s_i32_0 = l0;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
            l5 = s_i32_0;
            s_i32_0 = l3;
            s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
            s_i32_1 = l2;
            s_i64_1 = ops::i64_extend_i32_u(s_i32_1);
            s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
            l6 = s_i64_0;
            s_i64_0 = l6;
            s_i64_1 = 536854464i64;
            s_i32_0 = ops::i64_gt_u(s_i64_0, s_i64_1);
            'blk_3: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i64_0 = l6;
            s_i32_0 = ops::i32_wrap_i64(s_i64_0);
            l7 = s_i32_0;
            s_i32_0 = l7;
            s_i32_1 = l4;
            s_i32_0 = ops::i32_gt_u(s_i32_0, s_i32_1);
            'blk_4: {
                if s_i32_0 != 0 {
                    s_i32_0 = l4;
                    s_i32_1 = 268427232i32;
                    s_i32_0 = ops::i32_gt_u(s_i32_0, s_i32_1);
                    'blk_5: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 536854464i32;
                            l8 = s_i32_0;
                        } else {
                            s_i32_0 = l4;
                            s_i32_1 = 1i32;
                            s_i32_0 = ops::i32_shl(s_i32_0, s_i32_1);
                            l8 = s_i32_0;
                        }
                    }
                    s_i32_0 = l8;
                    s_i32_1 = l7;
                    s_i32_0 = ops::i32_lt_u(s_i32_0, s_i32_1);
                    'blk_6: {
                        if s_i32_0 != 0 {
                            s_i32_0 = l7;
                            l8 = s_i32_0;
                        }
                    }
                    s_i32_0 = l8;
                    s_i32_1 = 2i32;
                    s_i32_0 = ops::i32_shl(s_i32_0, s_i32_1);
                    vm.enter_call()?;
                    let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
                    vm.exit_call();
                    let __t0 = __call_res?;
                    s_i32_0 = __t0;
                    l9 = s_i32_0;
                    s_i32_0 = l3;
                    s_i32_1 = 0i32;
                    s_i32_0 = ops::i32_gt_u(s_i32_0, s_i32_1);
                    'blk_7: {
                        if s_i32_0 != 0 {
                            s_i32_0 = l9;
                            s_i32_1 = l5;
                            s_i32_2 = l3;
                            s_i32_3 = 2i32;
                            s_i32_2 = ops::i32_shl(s_i32_2, s_i32_3);
                            vm.memory_copy(0, 0, s_i32_0, s_i32_1, s_i32_2, host)?;
                        }
                    }
                    s_i32_0 = l0;
                    s_i32_1 = l9;
                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l0;
                    s_i32_1 = l8;
                    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                    s_i32_0 = l9;
                    l5 = s_i32_0;
                }
            }
            s_i32_0 = l5;
            s_i32_1 = l3;
            s_i32_2 = 2i32;
            s_i32_1 = ops::i32_shl(s_i32_1, s_i32_2);
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 0u32)?;
            s_i32_2 = l2;
            s_i32_3 = 2i32;
            s_i32_2 = ops::i32_shl(s_i32_2, s_i32_3);
            vm.memory_copy(0, 0, s_i32_0, s_i32_1, s_i32_2, host)?;
            s_i32_0 = l0;
            s_i32_1 = l7;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
        }
    }
    return Ok(());
    }

    pub(crate) fn s_1232b68aede8(vm: &mut SubstituteVm, p0: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i64 = 0i64;
    let mut l2: i64 = 0i64;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i64_0 = 1114112i64;
    s_i32_1 = l0;
    s_i64_1 = ops::i64_extend_i32_u(s_i32_1);
    vm.enter_call()?;
    let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i64_0 = __t0;
    l1 = s_i64_0;
    s_i64_0 = 0i64;
    l2 = s_i64_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
    vm.exit_call();
    let __t1 = __call_res?;
    s_i32_0 = __t1;
    l3 = s_i32_0;
    s_i32_0 = l3;
    s_i64_1 = l1;
    ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
    s_i32_0 = l3;
    s_i64_1 = l2;
    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
    s_i32_0 = l3;
    l4 = s_i32_0;
    s_i32_0 = l4;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn s_13309633dc1d(vm: &mut SubstituteVm, p0: i32, p1: i64, g0: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i64 = p1;
    let mut l2: i64 = 0i64;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut l5: i64 = 0i64;
    let mut l6: i32 = 0i32;
    let mut l7: i32 = 0i32;
    let mut l8: i32 = 0i32;
    let mut l9: i32 = 0i32;
    let mut l10: i32 = 0i32;
    let mut l11: i64 = 0i64;
    let mut l12: i32 = 0i32;
    let mut l13: i32 = 0i32;
    let mut l14: i32 = 0i32;
    let mut l15: i64 = 0i64;
    let mut l16: i32 = 0i32;
    let mut l17: i32 = 0i32;
    let mut l18: i32 = 0i32;
    let mut l19: i64 = 0i64;
    let mut l20: i32 = 0i32;
    let mut l21: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i64_2: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
    l2 = s_i64_0;
    s_i64_0 = l1;
    s_i64_1 = l2;
    s_i32_0 = ops::i64_ge_u(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            l3 = s_i32_0;
            s_i64_0 = 0i64;
            s_i64_1 = 0i64;
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_b3e6ca84b2de(vm, s_i64_0, s_i64_1, host);
            vm.exit_call();
            let __t0 = __call_res?;
            s_i32_0 = __t0;
            l4 = s_i32_0;
            s_i64_0 = l1;
            l5 = s_i64_0;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            l6 = s_i32_0;
            s_i32_0 = l6;
            s_i32_1 = l3;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l6;
            s_i32_1 = l4;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l6;
            s_i64_1 = l5;
            ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
            s_i32_0 = l6;
            l7 = s_i32_0;
            s_i32_0 = l7;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i32_0 = l0;
    s_i64_1 = l1;
    vm.enter_call()?;
    let __call_res = Self::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
    vm.exit_call();
    let __t2 = __call_res?;
    s_i32_0 = __t2;
    l8 = s_i32_0;
    s_i32_0 = l8;
    vm.enter_call()?;
    let __call_res = Self::s_62439e6f6ae7(vm, s_i32_0, g0, host);
    vm.exit_call();
    let __t3 = __call_res?;
    s_i32_0 = __t3;
    s_i32_1 = 255i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_2: {
        if s_i32_0 != 0 {
            s_i64_0 = l1;
            s_i64_1 = 1i64;
            vm.enter_call()?;
            let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
            vm.exit_call();
            let __t4 = __call_res?;
            s_i64_0 = __t4;
            s_i64_1 = l2;
            s_i32_0 = ops::i64_ge_u(s_i64_0, s_i64_1);
            'blk_3: {
                if s_i32_0 != 0 {
                    s_i32_0 = 0i32;
                    l9 = s_i32_0;
                    s_i64_0 = 0i64;
                    s_i64_1 = 0i64;
                    vm.enter_call()?;
                    let __call_res = crate::shared_1::Shared::s_b3e6ca84b2de(vm, s_i64_0, s_i64_1, host);
                    vm.exit_call();
                    let __t5 = __call_res?;
                    s_i32_0 = __t5;
                    l10 = s_i32_0;
                    s_i64_0 = l1;
                    l11 = s_i64_0;
                    s_i32_0 = 16i32;
                    vm.enter_call()?;
                    let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
                    vm.exit_call();
                    let __t6 = __call_res?;
                    s_i32_0 = __t6;
                    l12 = s_i32_0;
                    s_i32_0 = l12;
                    s_i32_1 = l9;
                    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l12;
                    s_i32_1 = l10;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l12;
                    s_i64_1 = l11;
                    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
                    s_i32_0 = l12;
                } else {
                    s_i32_0 = 1i32;
                    l13 = s_i32_0;
                    s_i32_0 = l0;
                    s_i64_1 = l1;
                    s_i64_2 = 1i64;
                    vm.enter_call()?;
                    let __call_res = Self::s_1c917789866c(vm, s_i64_1, s_i64_2, host);
                    vm.exit_call();
                    let __t7 = __call_res?;
                    s_i64_1 = __t7;
                    vm.enter_call()?;
                    let __call_res = Self::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
                    vm.exit_call();
                    let __t8 = __call_res?;
                    s_i32_0 = __t8;
                    l14 = s_i32_0;
                    s_i64_0 = l1;
                    s_i64_1 = 2i64;
                    vm.enter_call()?;
                    let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
                    vm.exit_call();
                    let __t9 = __call_res?;
                    s_i64_0 = __t9;
                    l15 = s_i64_0;
                    s_i32_0 = 16i32;
                    vm.enter_call()?;
                    let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
                    vm.exit_call();
                    let __t10 = __call_res?;
                    s_i32_0 = __t10;
                    l16 = s_i32_0;
                    s_i32_0 = l16;
                    s_i32_1 = l13;
                    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l16;
                    s_i32_1 = l14;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l16;
                    s_i64_1 = l15;
                    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
                    s_i32_0 = l16;
                }
            }
        } else {
            s_i32_0 = 1i32;
            l17 = s_i32_0;
            s_i32_0 = l8;
            l18 = s_i32_0;
            s_i64_0 = l1;
            s_i64_1 = 1i64;
            vm.enter_call()?;
            let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
            vm.exit_call();
            let __t11 = __call_res?;
            s_i64_0 = __t11;
            l19 = s_i64_0;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t12 = __call_res?;
            s_i32_0 = __t12;
            l20 = s_i32_0;
            s_i32_0 = l20;
            s_i32_1 = l17;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l20;
            s_i32_1 = l18;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l20;
            s_i64_1 = l19;
            ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
            s_i32_0 = l20;
        }
    }
    l21 = s_i32_0;
    s_i32_0 = l21;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_133f9dfb2fbd(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i64 = 0i64;
    let mut l3: i64 = 0i64;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 0u32)?;
    s_i32_1 = l1;
    s_i64_1 = ops::i64_load(vm, 0, s_i32_1, 0u32)?;
    s_i64_0 = ops::i64_xor(s_i64_0, s_i64_1);
    l2 = s_i64_0;
    s_i32_0 = l0;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
    s_i32_1 = l1;
    s_i64_1 = ops::i64_load(vm, 0, s_i32_1, 8u32)?;
    s_i64_0 = ops::i64_xor(s_i64_0, s_i64_1);
    l3 = s_i64_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l4 = s_i32_0;
    s_i32_0 = l4;
    s_i64_1 = l2;
    ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
    s_i32_0 = l4;
    s_i64_1 = l3;
    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
    s_i32_0 = l4;
    l5 = s_i32_0;
    s_i32_0 = l5;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_1446c357ee24(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, g1: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_1 = l1;
    vm.enter_call()?;
    let __call_res = Self::s_62932ccbf570(vm, s_i32_0, s_i32_1, g0, g1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l2 = s_i32_0;
    s_i32_0 = l2;
    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
    s_i32_0 = ops::i32_eqz(s_i32_0);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        } else {
            s_i32_0 = l2;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
        }
    }
    l3 = s_i32_0;
    s_i32_0 = l3;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_14853a432ff7(vm: &mut SubstituteVm, p0: i32, g0: usize, g1: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = 0i32;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i32 = 0i32;
    let mut l7: i32 = 0i32;
    let mut l8: i32 = 0i32;
    let mut l9: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
    s_i64_1 = 0i64;
    s_i32_0 = ops::i64_ge_s(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
            l1 = s_i32_0;
            s_i32_0 = l0;
            vm.enter_call()?;
            let __call_res = Self::s_623491ac0af2(vm, s_i32_0, host);
            vm.exit_call();
            let __t0 = __call_res?;
            s_i32_0 = __t0;
            l2 = s_i32_0;
            s_i32_0 = 0i32;
            l3 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            l4 = s_i32_0;
            s_i32_0 = l4;
            s_i32_1 = l1;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l4;
            s_i32_1 = l2;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l4;
            s_i32_1 = l3;
            ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l4;
        } else {
            s_i32_0 = 0i32;
            l5 = s_i32_0;
            s_i32_0 = vm.get_i32(g0)?;
            l6 = s_i32_0;
            s_i32_0 = vm.get_i32(g1)?;
            l7 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_0 = __t2;
            l8 = s_i32_0;
            s_i32_0 = l8;
            s_i32_1 = l5;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l8;
            s_i32_1 = l6;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l8;
            s_i32_1 = l7;
            ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l8;
        }
    }
    l9 = s_i32_0;
    s_i32_0 = l9;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_14dd5b913b03(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    s_i32_0 = l0;
    s_i32_1 = l1;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
    s_i32_1 = l1;
    s_i32_1 = ops::i32_load8_u(vm, 0, s_i32_1, 0u32)?;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    s_i32_0 = ops::i32_eqz(s_i32_0);
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    s_i32_1 = l1;
    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_9c6c8b521a4a(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    s_i32_0 = ops::i32_eqz(s_i32_0);
    'blk_3: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = 1i32;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_1596f30da51c(vm: &mut SubstituteVm, p0: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i64, WasmTrap> {
    let mut l0: i32 = p0;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    s_i32_0 = l0;
    s_i32_1 = 0i32;
    s_i32_0 = ops::i32_lt_s(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = l0;
    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
    return Ok(s_i64_0);
    }

    pub(crate) fn s_1613a7276793(vm: &mut SubstituteVm, p0: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut s_i32_0: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_f76e9849d691(vm, s_i32_0, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    let __t1 = s_i32_0;
    return crate::shared_1::Shared::s_958e9a31e156(vm, __t1, host);
    }

    #[inline(never)]
    pub(crate) fn s_1676a7fbbb39(vm: &mut SubstituteVm, p0: i32, g0: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = 0i32;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i32 = 0i32;
    let mut l7: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i32_2: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    vm.enter_call()?;
    let __call_res = Self::s_62439e6f6ae7(vm, s_i32_0, g0, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l1 = s_i32_0;
    s_i32_0 = l1;
    s_i32_1 = 203i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
        } else {
            s_i32_0 = l1;
            s_i32_1 = 255i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_2: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                } else {
                    s_i32_0 = l1;
                    s_i32_1 = 616i32;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                }
            }
        }
    }
    'blk_3: {
        if s_i32_0 != 0 {
            s_i32_0 = 255i32;
            vm.enter_call()?;
            let __call_res = Self::s_1232b68aede8(vm, s_i32_0, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            l2 = s_i32_0;
            s_i32_0 = l0;
            l3 = s_i32_0;
            s_i32_0 = 24i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_0 = __t2;
            l4 = s_i32_0;
            s_i32_0 = l4;
            s_i32_1 = l4;
            s_i32_2 = 16i32;
            s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l4;
            s_i32_1 = 2i32;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l4;
            s_i32_1 = 2i32;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l4;
            s_i32_1 = 1i32;
            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
            s_i32_0 = l4;
            s_i32_1 = l2;
            ops::i32_store(vm, 0, s_i32_0, 16u32, s_i32_1)?;
            s_i32_0 = l4;
            s_i32_1 = l3;
            ops::i32_store(vm, 0, s_i32_0, 20u32, s_i32_1)?;
            s_i32_0 = l4;
        } else {
            s_i32_0 = l0;
            l5 = s_i32_0;
            s_i32_0 = 20i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i32_0 = __t3;
            l6 = s_i32_0;
            s_i32_0 = l6;
            s_i32_1 = l6;
            s_i32_2 = 16i32;
            s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l6;
            s_i32_1 = 1i32;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l6;
            s_i32_1 = 1i32;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l6;
            s_i32_1 = 1i32;
            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
            s_i32_0 = l6;
            s_i32_1 = l5;
            ops::i32_store(vm, 0, s_i32_0, 16u32, s_i32_1)?;
            s_i32_0 = l6;
        }
    }
    l7 = s_i32_0;
    s_i32_0 = l7;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_17df4637c35a(vm: &mut SubstituteVm, p0: i32, p1: i64, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i64 = p1;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i32_2: i32 = 0i32;
    vm.deduct(host, 1)?;
    s_i64_0 = l1;
    s_i32_1 = l0;
    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
    s_i64_1 = ops::i64_extend_i32_u(s_i32_1);
    s_i32_0 = ops::i64_ge_u(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    s_i64_1 = l1;
    s_i32_1 = ops::i32_wrap_i64(s_i64_1);
    s_i32_2 = 2i32;
    s_i32_1 = ops::i32_shl(s_i32_1, s_i32_2);
    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn s_17f5562cc011(vm: &mut SubstituteVm, p0: i32, g0: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i64 = 0i64;
    let mut l2: i32 = 0i32;
    let mut l3: i64 = 0i64;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
    l1 = s_i64_0;
    s_i64_0 = l1;
    s_i64_1 = 2i64;
    vm.enter_call()?;
    let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i64_0 = __t0;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_c5da14f612f0(vm, s_i64_0, host);
    vm.exit_call();
    let __t1 = __call_res?;
    s_i32_0 = __t1;
    l2 = s_i32_0;
    s_i32_0 = l2;
    s_i32_1 = 270i32;
    vm.enter_call()?;
    let __call_res = Self::s_1232b68aede8(vm, s_i32_1, host);
    vm.exit_call();
    let __t2 = __call_res?;
    s_i32_1 = __t2;
    vm.enter_call()?;
    let __call_res = Self::s_697cb42b9ce7(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    __call_res?;
    s_i64_0 = 0i64;
    l3 = s_i64_0;
    'blk_1: {
        'blk_2: loop {
            vm.deduct(host, 1)?;
            s_i64_0 = l3;
            s_i64_1 = l1;
            s_i32_0 = ops::i64_lt_u(s_i64_0, s_i64_1);
            s_i32_0 = ops::i32_eqz(s_i32_0);
            if s_i32_0 != 0 {
                break 'blk_1;
            }
            s_i32_0 = l0;
            s_i64_1 = l3;
            vm.enter_call()?;
            let __call_res = Self::s_4e6fb2273c0f(vm, s_i32_0, s_i64_1, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i32_0 = __t3;
            s_i32_1 = 255i32;
            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_fca97fe5a062(vm, s_i32_0, host);
            vm.exit_call();
            let __t4 = __call_res?;
            s_i32_0 = __t4;
            l4 = s_i32_0;
            s_i32_0 = l4;
            vm.enter_call()?;
            let __call_res = Self::s_62439e6f6ae7(vm, s_i32_0, g0, host);
            vm.exit_call();
            let __t5 = __call_res?;
            s_i32_0 = __t5;
            l5 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = 255i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_3: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                } else {
                    s_i32_0 = l5;
                    s_i32_1 = 271i32;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                    'blk_4: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 1i32;
                        } else {
                            s_i64_0 = l3;
                            s_i64_1 = 0i64;
                            s_i32_0 = ops::i64_eq(s_i64_0, s_i64_1);
                            'blk_5: {
                                if s_i32_0 != 0 {
                                    s_i32_0 = l4;
                                    vm.enter_call()?;
                                    let __call_res = Self::s_3ac41cc6c31c(vm, s_i32_0, host);
                                    vm.exit_call();
                                    let __t6 = __call_res?;
                                    s_i32_0 = __t6;
                                    'blk_6: {
                                        if s_i32_0 != 0 {
                                            s_i32_0 = 1i32;
                                        } else {
                                            s_i32_0 = l5;
                                            s_i32_1 = 262i32;
                                            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                                        }
                                    }
                                } else {
                                    s_i32_0 = 0i32;
                                }
                            }
                        }
                    }
                }
            }
            'blk_7: {
                if s_i32_0 != 0 {
                    s_i32_0 = l2;
                    s_i32_1 = 255i32;
                    vm.enter_call()?;
                    let __call_res = Self::s_1232b68aede8(vm, s_i32_1, host);
                    vm.exit_call();
                    let __t7 = __call_res?;
                    s_i32_1 = __t7;
                    vm.enter_call()?;
                    let __call_res = Self::s_697cb42b9ce7(vm, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                } else {
                }
            }
            s_i32_0 = l2;
            s_i32_1 = l4;
            vm.enter_call()?;
            let __call_res = Self::s_697cb42b9ce7(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i64_0 = l3;
            s_i64_1 = 1i64;
            vm.enter_call()?;
            let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
            vm.exit_call();
            let __t8 = __call_res?;
            s_i64_0 = __t8;
            l3 = s_i64_0;
            continue 'blk_2;
            break 'blk_2;
        }
    }
    s_i32_0 = l2;
    s_i32_1 = 271i32;
    vm.enter_call()?;
    let __call_res = Self::s_1232b68aede8(vm, s_i32_1, host);
    vm.exit_call();
    let __t9 = __call_res?;
    s_i32_1 = __t9;
    vm.enter_call()?;
    let __call_res = Self::s_697cb42b9ce7(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l2;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_df3679579582(vm, s_i32_0, host);
    vm.exit_call();
    let __t10 = __call_res?;
    s_i32_0 = __t10;
    l6 = s_i32_0;
    s_i32_0 = l6;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_18330dd1a568(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, g1: usize, g2: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l1;
    s_i32_1 = vm.get_i32(g0)?;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_84059c53f331(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    s_i32_0 = ops::i32_eqz(s_i32_0);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i32_1 = vm.get_i32(g1)?;
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_84059c53f331(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            'blk_2: {
                if s_i32_0 != 0 {
                    s_i32_0 = l1;
                    s_i32_1 = vm.get_i32(g2)?;
                    vm.enter_call()?;
                    let __call_res = crate::shared_1::Shared::s_84059c53f331(vm, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t2 = __call_res?;
                    s_i32_0 = __t2;
                } else {
                    s_i32_0 = 0i32;
                }
            }
            s_i32_0 = ops::i32_eqz(s_i32_0);
        } else {
            s_i32_0 = 0i32;
        }
    }
    l2 = s_i32_0;
    s_i32_0 = l2;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn s_18758f469c25(vm: &mut SubstituteVm, p0: i32, p1: i64, host: &mut dyn WasmSubstituteHost) -> std::result::Result<(), WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i64 = p1;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i32_2: i32 = 0i32;
    let mut s_i32_3: i32 = 0i32;
    vm.deduct(host, 1)?;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 12u32)?;
    s_i32_1 = 0i32;
    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    l2 = s_i32_0;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
    l3 = s_i32_0;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    l4 = s_i32_0;
    s_i32_0 = l2;
    s_i32_1 = l3;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = l2;
            s_i32_1 = 268427232i32;
            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
            'blk_3: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = l3;
            s_i32_1 = 8i32;
            s_i32_0 = ops::i32_lt_u(s_i32_0, s_i32_1);
            'blk_4: {
                if s_i32_0 != 0 {
                    s_i32_0 = 8i32;
                    l5 = s_i32_0;
                } else {
                    s_i32_0 = l3;
                    s_i32_1 = 134213376i32;
                    s_i32_0 = ops::i32_gt_u(s_i32_0, s_i32_1);
                    'blk_5: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 268427232i32;
                            l5 = s_i32_0;
                        } else {
                            s_i32_0 = l3;
                            s_i32_1 = 1i32;
                            s_i32_0 = ops::i32_shl(s_i32_0, s_i32_1);
                            l5 = s_i32_0;
                        }
                    }
                }
            }
            s_i32_0 = l5;
            s_i32_1 = 3i32;
            s_i32_0 = ops::i32_shl(s_i32_0, s_i32_1);
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t0 = __call_res?;
            s_i32_0 = __t0;
            l6 = s_i32_0;
            s_i32_0 = l2;
            s_i32_1 = 0i32;
            s_i32_0 = ops::i32_gt_u(s_i32_0, s_i32_1);
            'blk_6: {
                if s_i32_0 != 0 {
                    s_i32_0 = l6;
                    s_i32_1 = l4;
                    s_i32_2 = l2;
                    s_i32_3 = 3i32;
                    s_i32_2 = ops::i32_shl(s_i32_2, s_i32_3);
                    vm.memory_copy(0, 0, s_i32_0, s_i32_1, s_i32_2, host)?;
                }
            }
            s_i32_0 = l0;
            s_i32_1 = l6;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l0;
            s_i32_1 = l5;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l6;
            l4 = s_i32_0;
        }
    }
    s_i32_0 = l4;
    s_i32_1 = l2;
    s_i32_2 = 3i32;
    s_i32_1 = ops::i32_shl(s_i32_1, s_i32_2);
    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
    s_i64_1 = l1;
    ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
    s_i32_0 = l0;
    s_i32_1 = l2;
    s_i32_2 = 1i32;
    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    return Ok(());
    }

    pub(crate) fn s_18f4f42cbdc4(vm: &mut SubstituteVm, p0: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = 1i32;
    l1 = s_i32_0;
    s_i32_0 = l1;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_19ca9f9d3f2f(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i64 = 0i64;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    vm.deduct(host, 1)?;
    s_i32_0 = l0;
    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
    s_i32_1 = l1;
    s_i64_1 = ops::i64_extend_i32_u(s_i32_1);
    s_i64_0 = ops::i64_mul(s_i64_0, s_i64_1);
    l2 = s_i64_0;
    s_i64_0 = l2;
    s_i64_1 = 4294967295i64;
    s_i32_0 = ops::i64_gt_u(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i64_0 = l2;
    s_i32_0 = ops::i32_wrap_i64(s_i64_0);
    return Ok(s_i32_0);
    }

    pub(crate) fn s_1c37980bc1a5(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l1;
    s_i32_1 = l0;
    vm.enter_call()?;
    let __call_res = Self::s_745aa11cfb74(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    s_i32_0 = ops::i32_eqz(s_i32_0);
    l2 = s_i32_0;
    s_i32_0 = l2;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_1c917789866c(vm: &mut SubstituteVm, p0: i64, p1: i64, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i64, WasmTrap> {
    let mut l0: i64 = p0;
    let mut l1: i64 = p1;
    let mut l2: i64 = 0i64;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i64_1: i64 = 0i64;
    vm.deduct(host, 1)?;
    s_i64_0 = l0;
    s_i64_1 = l1;
    s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
    l2 = s_i64_0;
    s_i64_0 = l2;
    s_i64_1 = l0;
    s_i32_0 = ops::i64_lt_u(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i64_0 = l2;
    return Ok(s_i64_0);
    }

    pub(crate) fn s_1cbb4a12599c(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i32 = 0i32;
    let mut l7: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i32_2: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
    l2 = s_i32_0;
    s_i32_0 = l1;
    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
    l3 = s_i32_0;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    l4 = s_i32_0;
    s_i32_0 = l1;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    l5 = s_i32_0;
    s_i32_0 = l2;
    s_i32_1 = l3;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = l2;
            s_i32_1 = l4;
            s_i32_2 = l5;
            vm.enter_call()?;
            let __call_res = Self::s_5af5ebb1dc24(vm, s_i32_1, s_i32_2, host);
            vm.exit_call();
            let __t0 = __call_res?;
            s_i32_1 = __t0;
            let __t1 = s_i32_0;
            let __t2 = s_i32_1;
            return Self::s_482c16ff60d0(vm, __t1, __t2, g0, host);
        } else {
            s_i32_0 = l4;
            s_i32_1 = l5;
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_86b08f7a2a43(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i32_0 = __t3;
            l6 = s_i32_0;
            s_i32_0 = l6;
            s_i32_1 = 0i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_2: {
                if s_i32_0 != 0 {
                    s_i32_0 = vm.get_i32(g0)?;
                } else {
                    s_i32_0 = l6;
                    s_i32_1 = 0i32;
                    s_i32_0 = ops::i32_gt_s(s_i32_0, s_i32_1);
                    'blk_3: {
                        if s_i32_0 != 0 {
                            s_i32_0 = l2;
                            s_i32_1 = l4;
                            s_i32_2 = l5;
                            vm.enter_call()?;
                            let __call_res = Self::s_66cfb899ddc3(vm, s_i32_1, s_i32_2, host);
                            vm.exit_call();
                            let __t4 = __call_res?;
                            s_i32_1 = __t4;
                            let __t5 = s_i32_0;
                            let __t6 = s_i32_1;
                            return Self::s_482c16ff60d0(vm, __t5, __t6, g0, host);
                        } else {
                            s_i32_0 = l3;
                            s_i32_1 = l5;
                            s_i32_2 = l4;
                            vm.enter_call()?;
                            let __call_res = Self::s_66cfb899ddc3(vm, s_i32_1, s_i32_2, host);
                            vm.exit_call();
                            let __t7 = __call_res?;
                            s_i32_1 = __t7;
                            let __t8 = s_i32_0;
                            let __t9 = s_i32_1;
                            return Self::s_482c16ff60d0(vm, __t8, __t9, g0, host);
                        }
                    }
                    return Err(WasmTrap::Unreachable);
                }
            }
        }
    }
    l7 = s_i32_0;
    s_i32_0 = l7;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_1cc9fee33150(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    l2 = s_i32_0;
    s_i32_0 = l1;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    l3 = s_i32_0;
    'blk_1: {
        'blk_2: loop {
            vm.deduct(host, 1)?;
            s_i32_0 = l3;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
            s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
            s_i64_1 = 0i64;
            s_i32_0 = ops::i64_gt_u(s_i64_0, s_i64_1);
            s_i32_0 = ops::i32_eqz(s_i32_0);
            if s_i32_0 != 0 {
                break 'blk_1;
            }
            s_i32_0 = l2;
            s_i32_1 = l3;
            vm.enter_call()?;
            let __call_res = Self::s_4552c7e0e83b(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t0 = __call_res?;
            s_i32_0 = __t0;
            l4 = s_i32_0;
            s_i32_0 = l3;
            l2 = s_i32_0;
            s_i32_0 = l4;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
            l3 = s_i32_0;
            continue 'blk_2;
            break 'blk_2;
        }
    }
    s_i32_0 = 0i32;
    s_i32_1 = l2;
    let __t1 = s_i32_0;
    let __t2 = s_i32_1;
    return Self::s_482c16ff60d0(vm, __t1, __t2, g0, host);
    }

    pub(crate) fn s_1d06de4c220c(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, g1: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i32 = 0i32;
    let mut l7: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_1 = l1;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_bd22d8f9c2fc(vm, s_i32_0, s_i32_1, g0, g1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l2 = s_i32_0;
    s_i32_0 = l2;
    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
    l3 = s_i32_0;
    s_i32_0 = l2;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    l4 = s_i32_0;
    s_i32_0 = l2;
    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 12u32)?;
    l5 = s_i32_0;
    s_i32_0 = 12i32;
    vm.enter_call()?;
    let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
    vm.exit_call();
    let __t1 = __call_res?;
    s_i32_0 = __t1;
    l6 = s_i32_0;
    s_i32_0 = l6;
    s_i32_1 = l3;
    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l6;
    s_i32_1 = l4;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l6;
    s_i32_1 = l5;
    ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
    s_i32_0 = l6;
    l7 = s_i32_0;
    s_i32_0 = l7;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_1db1a11a3e81(vm: &mut SubstituteVm, p0: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut s_i32_0: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 517i32;
        } else {
            s_i32_0 = 518i32;
        }
    }
    let __t0 = s_i32_0;
    return Self::s_1232b68aede8(vm, __t0, host);
    }

    pub(crate) fn s_1dfcf0885d9c(vm: &mut SubstituteVm, p0: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i64 = 0i64;
    let mut l2: i64 = 0i64;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i64_1: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 0u32)?;
    s_i64_1 = -1i64;
    s_i64_0 = ops::i64_xor(s_i64_0, s_i64_1);
    l1 = s_i64_0;
    s_i32_0 = l0;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
    s_i64_1 = -1i64;
    s_i64_0 = ops::i64_xor(s_i64_0, s_i64_1);
    l2 = s_i64_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l3 = s_i32_0;
    s_i32_0 = l3;
    s_i64_1 = l1;
    ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
    s_i32_0 = l3;
    s_i64_1 = l2;
    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
    s_i32_0 = l3;
    l4 = s_i32_0;
    s_i32_0 = l4;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn s_1e656cbc7955(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, g1: usize, g2: usize, g3: usize, g4: usize, g5: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l1;
    s_i32_1 = vm.get_i32(g0)?;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_84059c53f331(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            l2 = s_i32_0;
            s_i32_0 = vm.get_i32(g0)?;
            l3 = s_i32_0;
            s_i32_0 = vm.get_i32(g1)?;
            l4 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            l5 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = l2;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l5;
            s_i32_1 = l3;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l5;
            s_i32_1 = l4;
            ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l5;
            l6 = s_i32_0;
            s_i32_0 = l6;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i32_0 = l0;
    s_i32_1 = vm.get_i32(g2)?;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_84059c53f331(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t2 = __call_res?;
    s_i32_0 = __t2;
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            s_i32_1 = vm.get_i32(g3)?;
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_84059c53f331(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i32_0 = __t3;
        } else {
            s_i32_0 = 0i32;
        }
    }
    'blk_3: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            l7 = s_i32_0;
            s_i32_0 = vm.get_i32(g0)?;
            l8 = s_i32_0;
            s_i32_0 = vm.get_i32(g4)?;
            l9 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t4 = __call_res?;
            s_i32_0 = __t4;
            l10 = s_i32_0;
            s_i32_0 = l10;
            s_i32_1 = l7;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l10;
            s_i32_1 = l8;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l10;
            s_i32_1 = l9;
            ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l10;
            l11 = s_i32_0;
            s_i32_0 = l11;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i32_0 = l0;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_f7d297dbbc03(vm, s_i32_0, g5, host);
    vm.exit_call();
    let __t5 = __call_res?;
    s_i32_0 = __t5;
    s_i32_1 = l1;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_f7d297dbbc03(vm, s_i32_1, g5, host);
    vm.exit_call();
    let __t6 = __call_res?;
    s_i32_1 = __t6;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_bd22d8f9c2fc(vm, s_i32_0, s_i32_1, g5, g1, host);
    vm.exit_call();
    let __t7 = __call_res?;
    s_i32_0 = __t7;
    l12 = s_i32_0;
    s_i32_0 = l12;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    vm.enter_call()?;
    let __call_res = Self::s_623491ac0af2(vm, s_i32_0, host);
    vm.exit_call();
    let __t8 = __call_res?;
    s_i32_0 = __t8;
    l13 = s_i32_0;
    s_i32_0 = l0;
    vm.enter_call()?;
    let __call_res = Self::s_51f0c0b4c723(vm, s_i32_0, host);
    vm.exit_call();
    let __t9 = __call_res?;
    s_i32_0 = __t9;
    s_i32_1 = l1;
    vm.enter_call()?;
    let __call_res = Self::s_51f0c0b4c723(vm, s_i32_1, host);
    vm.exit_call();
    let __t10 = __call_res?;
    s_i32_1 = __t10;
    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
    'blk_4: {
        if s_i32_0 != 0 {
            s_i32_0 = l13;
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_d0a938e2c1d3(vm, s_i32_0, g0, host);
            vm.exit_call();
            let __t11 = __call_res?;
            s_i32_0 = __t11;
        } else {
            s_i32_0 = l13;
        }
    }
    l14 = s_i32_0;
    s_i32_0 = 1i32;
    l15 = s_i32_0;
    s_i32_0 = l14;
    l16 = s_i32_0;
    s_i32_0 = 0i32;
    l17 = s_i32_0;
    s_i32_0 = 12i32;
    vm.enter_call()?;
    let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
    vm.exit_call();
    let __t12 = __call_res?;
    s_i32_0 = __t12;
    l18 = s_i32_0;
    s_i32_0 = l18;
    s_i32_1 = l15;
    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l18;
    s_i32_1 = l16;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l18;
    s_i32_1 = l17;
    ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
    s_i32_0 = l18;
    l19 = s_i32_0;
    s_i32_0 = l19;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_1e670f572c93(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    s_i32_0 = l1;
    s_i32_1 = 8i32;
    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = l0;
    s_i32_1 = l1;
    s_i32_0 = ops::i32_shl(s_i32_0, s_i32_1);
    s_i32_1 = 255i32;
    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn s_21f47e1623f0(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, g1: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l1;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_c31be4b7b29d(vm, s_i32_0, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            l2 = s_i32_0;
            s_i32_0 = vm.get_i32(g0)?;
            l3 = s_i32_0;
            s_i32_0 = vm.get_i32(g1)?;
            l4 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            l5 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = l2;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l5;
            s_i32_1 = l3;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l5;
            s_i32_1 = l4;
            ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l5;
            l6 = s_i32_0;
            s_i32_0 = l6;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    s_i32_1 = l1;
    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
    vm.enter_call()?;
    let __call_res = Self::s_4552c7e0e83b(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t2 = __call_res?;
    s_i32_0 = __t2;
    l7 = s_i32_0;
    s_i32_0 = 1i32;
    l8 = s_i32_0;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
    s_i32_1 = l7;
    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
    vm.enter_call()?;
    let __call_res = Self::s_482c16ff60d0(vm, s_i32_0, s_i32_1, g0, host);
    vm.exit_call();
    let __t3 = __call_res?;
    s_i32_0 = __t3;
    l9 = s_i32_0;
    s_i32_0 = 0i32;
    l10 = s_i32_0;
    s_i32_0 = 12i32;
    vm.enter_call()?;
    let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
    vm.exit_call();
    let __t4 = __call_res?;
    s_i32_0 = __t4;
    l11 = s_i32_0;
    s_i32_0 = l11;
    s_i32_1 = l8;
    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l11;
    s_i32_1 = l9;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l11;
    s_i32_1 = l10;
    ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
    s_i32_0 = l11;
    l12 = s_i32_0;
    s_i32_0 = l12;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_251f4cc0bb92(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_1 = l1;
    vm.enter_call()?;
    let __call_res = Self::s_5b883f251b47(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    s_i32_1 = 0i32;
    s_i32_0 = ops::i32_lt_s(s_i32_0, s_i32_1);
    l2 = s_i32_0;
    s_i32_0 = l2;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn s_2570bfaef612(vm: &mut SubstituteVm, p0: i32, p1: i64, g0: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i64 = p1;
    let mut l2: i64 = 0i64;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i32 = 0i32;
    let mut l7: i32 = 0i32;
    let mut l8: i64 = 0i64;
    let mut l9: i32 = 0i32;
    let mut l10: i32 = 0i32;
    let mut l11: i32 = 0i32;
    let mut l12: i32 = 0i32;
    let mut l13: i32 = 0i32;
    let mut l14: i32 = 0i32;
    let mut l15: i32 = 0i32;
    let mut l16: i64 = 0i64;
    let mut l17: i32 = 0i32;
    let mut l18: i32 = 0i32;
    let mut l19: i32 = 0i32;
    let mut l20: i32 = 0i32;
    let mut l21: i32 = 0i32;
    let mut l22: i32 = 0i32;
    let mut l23: i32 = 0i32;
    let mut l24: i32 = 0i32;
    let mut l25: i64 = 0i64;
    let mut l26: i32 = 0i32;
    let mut l27: i32 = 0i32;
    let mut l28: i32 = 0i32;
    let mut l29: i64 = 0i64;
    let mut l30: i32 = 0i32;
    let mut l31: i32 = 0i32;
    let mut l32: i32 = 0i32;
    let mut l33: i32 = 0i32;
    let mut l34: i32 = 0i32;
    let mut l35: i32 = 0i32;
    let mut l36: i32 = 0i32;
    let mut l37: i32 = 0i32;
    let mut l38: i64 = 0i64;
    let mut l39: i32 = 0i32;
    let mut l40: i32 = 0i32;
    let mut l41: i32 = 0i32;
    let mut l42: i32 = 0i32;
    let mut l43: i32 = 0i32;
    let mut l44: i32 = 0i32;
    let mut l45: i64 = 0i64;
    let mut l46: i32 = 0i32;
    let mut l47: i32 = 0i32;
    let mut l48: i32 = 0i32;
    let mut l49: i32 = 0i32;
    let mut l50: i32 = 0i32;
    let mut l51: i32 = 0i32;
    let mut l52: i32 = 0i32;
    let mut l53: i64 = 0i64;
    let mut l54: i32 = 0i32;
    let mut l55: i32 = 0i32;
    let mut l56: i32 = 0i32;
    let mut l57: i32 = 0i32;
    let mut l58: i32 = 0i32;
    let mut l59: i32 = 0i32;
    let mut l60: i32 = 0i32;
    let mut l61: i64 = 0i64;
    let mut l62: i32 = 0i32;
    let mut l63: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i32_2: i32 = 0i32;
    let mut s_i64_2: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
    l2 = s_i64_0;
    s_i64_0 = l1;
    s_i64_1 = 4i64;
    vm.enter_call()?;
    let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i64_0 = __t0;
    s_i64_1 = l2;
    s_i32_0 = ops::i64_gt_u(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            l3 = s_i32_0;
            s_i32_0 = 0i32;
            l4 = s_i32_0;
            s_i32_0 = 0i32;
            l5 = s_i32_0;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            l6 = s_i32_0;
            s_i32_0 = l6;
            s_i32_1 = l6;
            s_i32_2 = 16i32;
            s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l6;
            s_i32_1 = 0i32;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l6;
            s_i32_1 = 0i32;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l6;
            s_i32_1 = 1i32;
            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
            s_i32_0 = l6;
            l7 = s_i32_0;
            s_i64_0 = l1;
            l8 = s_i64_0;
            s_i32_0 = 24i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_0 = __t2;
            l9 = s_i32_0;
            s_i32_0 = l9;
            s_i32_1 = l3;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l9;
            s_i32_1 = l4;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l9;
            s_i32_1 = l5;
            ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l9;
            s_i32_1 = l7;
            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
            s_i32_0 = l9;
            s_i64_1 = l8;
            ops::i64_store(vm, 0, s_i32_0, 16u32, s_i64_1)?;
            s_i32_0 = l9;
            l10 = s_i32_0;
            s_i32_0 = l10;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i32_0 = l0;
    s_i64_1 = l1;
    vm.enter_call()?;
    let __call_res = Self::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
    vm.exit_call();
    let __t3 = __call_res?;
    s_i32_0 = __t3;
    vm.enter_call()?;
    let __call_res = Self::s_62439e6f6ae7(vm, s_i32_0, g0, host);
    vm.exit_call();
    let __t4 = __call_res?;
    s_i32_0 = __t4;
    s_i32_1 = 6i32;
    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            l11 = s_i32_0;
            s_i32_0 = 0i32;
            l12 = s_i32_0;
            s_i32_0 = 0i32;
            l13 = s_i32_0;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t5 = __call_res?;
            s_i32_0 = __t5;
            l14 = s_i32_0;
            s_i32_0 = l14;
            s_i32_1 = l14;
            s_i32_2 = 16i32;
            s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l14;
            s_i32_1 = 0i32;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l14;
            s_i32_1 = 0i32;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l14;
            s_i32_1 = 1i32;
            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
            s_i32_0 = l14;
            l15 = s_i32_0;
            s_i64_0 = l1;
            l16 = s_i64_0;
            s_i32_0 = 24i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t6 = __call_res?;
            s_i32_0 = __t6;
            l17 = s_i32_0;
            s_i32_0 = l17;
            s_i32_1 = l11;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l17;
            s_i32_1 = l12;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l17;
            s_i32_1 = l13;
            ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l17;
            s_i32_1 = l15;
            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
            s_i32_0 = l17;
            s_i64_1 = l16;
            ops::i64_store(vm, 0, s_i32_0, 16u32, s_i64_1)?;
            s_i32_0 = l17;
            l18 = s_i32_0;
            s_i32_0 = l18;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i32_0 = l0;
    s_i64_1 = l1;
    s_i64_2 = 1i64;
    vm.enter_call()?;
    let __call_res = Self::s_1c917789866c(vm, s_i64_1, s_i64_2, host);
    vm.exit_call();
    let __t7 = __call_res?;
    s_i64_1 = __t7;
    vm.enter_call()?;
    let __call_res = Self::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
    vm.exit_call();
    let __t8 = __call_res?;
    s_i32_0 = __t8;
    l19 = s_i32_0;
    s_i32_0 = l19;
    vm.enter_call()?;
    let __call_res = Self::s_3ac41cc6c31c(vm, s_i32_0, host);
    vm.exit_call();
    let __t9 = __call_res?;
    s_i32_0 = __t9;
    s_i32_0 = ops::i32_eqz(s_i32_0);
    'blk_3: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            l20 = s_i32_0;
            s_i32_0 = 0i32;
            l21 = s_i32_0;
            s_i32_0 = 0i32;
            l22 = s_i32_0;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t10 = __call_res?;
            s_i32_0 = __t10;
            l23 = s_i32_0;
            s_i32_0 = l23;
            s_i32_1 = l23;
            s_i32_2 = 16i32;
            s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l23;
            s_i32_1 = 0i32;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l23;
            s_i32_1 = 0i32;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l23;
            s_i32_1 = 1i32;
            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
            s_i32_0 = l23;
            l24 = s_i32_0;
            s_i64_0 = l1;
            l25 = s_i64_0;
            s_i32_0 = 24i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t11 = __call_res?;
            s_i32_0 = __t11;
            l26 = s_i32_0;
            s_i32_0 = l26;
            s_i32_1 = l20;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l26;
            s_i32_1 = l21;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l26;
            s_i32_1 = l22;
            ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l26;
            s_i32_1 = l24;
            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
            s_i32_0 = l26;
            s_i64_1 = l25;
            ops::i64_store(vm, 0, s_i32_0, 16u32, s_i64_1)?;
            s_i32_0 = l26;
            l27 = s_i32_0;
            s_i32_0 = l27;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i32_0 = l19;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_8a79d3bd42f3(vm, s_i32_0, g0, host);
    vm.exit_call();
    let __t12 = __call_res?;
    s_i32_0 = __t12;
    l28 = s_i32_0;
    s_i64_0 = l1;
    s_i64_1 = 2i64;
    vm.enter_call()?;
    let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
    vm.exit_call();
    let __t13 = __call_res?;
    s_i64_0 = __t13;
    l29 = s_i64_0;
    s_i32_0 = 0i32;
    l30 = s_i32_0;
    s_i32_0 = l0;
    s_i64_1 = l29;
    vm.enter_call()?;
    let __call_res = Self::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
    vm.exit_call();
    let __t14 = __call_res?;
    s_i32_0 = __t14;
    vm.enter_call()?;
    let __call_res = Self::s_62439e6f6ae7(vm, s_i32_0, g0, host);
    vm.exit_call();
    let __t15 = __call_res?;
    s_i32_0 = __t15;
    s_i32_1 = 11i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_4: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
            l30 = s_i32_0;
            s_i64_0 = l29;
            s_i64_1 = 1i64;
            vm.enter_call()?;
            let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
            vm.exit_call();
            let __t16 = __call_res?;
            s_i64_0 = __t16;
            l29 = s_i64_0;
        } else {
        }
    }
    s_i64_0 = 22i64;
    vm.enter_call()?;
    let __call_res = Self::s_61878e7d5ba7(vm, s_i64_0, host);
    vm.exit_call();
    let __t17 = __call_res?;
    s_i32_0 = __t17;
    l31 = s_i32_0;
    'blk_5: {
        'blk_6: loop {
            vm.deduct(host, 1)?;
            s_i64_0 = l29;
            s_i64_1 = l2;
            s_i32_0 = ops::i64_lt_u(s_i64_0, s_i64_1);
            s_i32_0 = ops::i32_eqz(s_i32_0);
            if s_i32_0 != 0 {
                break 'blk_5;
            }
            s_i32_0 = l0;
            s_i64_1 = l29;
            vm.enter_call()?;
            let __call_res = Self::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
            vm.exit_call();
            let __t18 = __call_res?;
            s_i32_0 = __t18;
            vm.enter_call()?;
            let __call_res = Self::s_62439e6f6ae7(vm, s_i32_0, g0, host);
            vm.exit_call();
            let __t19 = __call_res?;
            s_i32_0 = __t19;
            l32 = s_i32_0;
            s_i32_0 = l32;
            s_i32_1 = 7i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_7: {
                if s_i32_0 != 0 {
                    s_i32_0 = l31;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
                    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
                    s_i64_1 = 0i64;
                    s_i32_0 = ops::i64_eq(s_i64_0, s_i64_1);
                    'blk_8: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 0i32;
                            l33 = s_i32_0;
                            s_i32_0 = 0i32;
                            l34 = s_i32_0;
                            s_i32_0 = 0i32;
                            l35 = s_i32_0;
                            s_i32_0 = 16i32;
                            vm.enter_call()?;
                            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
                            vm.exit_call();
                            let __t20 = __call_res?;
                            s_i32_0 = __t20;
                            l36 = s_i32_0;
                            s_i32_0 = l36;
                            s_i32_1 = l36;
                            s_i32_2 = 16i32;
                            s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
                            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                            s_i32_0 = l36;
                            s_i32_1 = 0i32;
                            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                            s_i32_0 = l36;
                            s_i32_1 = 0i32;
                            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                            s_i32_0 = l36;
                            s_i32_1 = 1i32;
                            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                            s_i32_0 = l36;
                            l37 = s_i32_0;
                            s_i64_0 = l1;
                            l38 = s_i64_0;
                            s_i32_0 = 24i32;
                            vm.enter_call()?;
                            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
                            vm.exit_call();
                            let __t21 = __call_res?;
                            s_i32_0 = __t21;
                            l39 = s_i32_0;
                            s_i32_0 = l39;
                            s_i32_1 = l33;
                            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                            s_i32_0 = l39;
                            s_i32_1 = l34;
                            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                            s_i32_0 = l39;
                            s_i32_1 = l35;
                            ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                            s_i32_0 = l39;
                            s_i32_1 = l37;
                            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                            s_i32_0 = l39;
                            s_i64_1 = l38;
                            ops::i64_store(vm, 0, s_i32_0, 16u32, s_i64_1)?;
                            s_i32_0 = l39;
                            l40 = s_i32_0;
                            s_i32_0 = l40;
                            return Ok(s_i32_0);
                        } else {
                            s_i32_0 = 1i32;
                            l41 = s_i32_0;
                            s_i32_0 = l28;
                            l42 = s_i32_0;
                            s_i32_0 = l30;
                            l43 = s_i32_0;
                            s_i32_0 = l31;
                            vm.enter_call()?;
                            let __call_res = crate::shared_1::Shared::s_df3679579582(vm, s_i32_0, host);
                            vm.exit_call();
                            let __t22 = __call_res?;
                            s_i32_0 = __t22;
                            l44 = s_i32_0;
                            s_i64_0 = l29;
                            s_i64_1 = 1i64;
                            vm.enter_call()?;
                            let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
                            vm.exit_call();
                            let __t23 = __call_res?;
                            s_i64_0 = __t23;
                            l45 = s_i64_0;
                            s_i32_0 = 24i32;
                            vm.enter_call()?;
                            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
                            vm.exit_call();
                            let __t24 = __call_res?;
                            s_i32_0 = __t24;
                            l46 = s_i32_0;
                            s_i32_0 = l46;
                            s_i32_1 = l41;
                            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                            s_i32_0 = l46;
                            s_i32_1 = l42;
                            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                            s_i32_0 = l46;
                            s_i32_1 = l43;
                            ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                            s_i32_0 = l46;
                            s_i32_1 = l44;
                            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                            s_i32_0 = l46;
                            s_i64_1 = l45;
                            ops::i64_store(vm, 0, s_i32_0, 16u32, s_i64_1)?;
                            s_i32_0 = l46;
                            l47 = s_i32_0;
                            s_i32_0 = l47;
                            return Ok(s_i32_0);
                        }
                    }
                    return Err(WasmTrap::Unreachable);
                } else {
                }
            }
            s_i32_0 = l32;
            s_i32_1 = 127i32;
            s_i32_0 = ops::i32_lt_u(s_i32_0, s_i32_1);
            'blk_9: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                } else {
                    s_i32_0 = l32;
                    s_i32_1 = 190i32;
                    s_i32_0 = ops::i32_gt_u(s_i32_0, s_i32_1);
                }
            }
            'blk_10: {
                if s_i32_0 != 0 {
                    s_i32_0 = 0i32;
                    l48 = s_i32_0;
                    s_i32_0 = 0i32;
                    l49 = s_i32_0;
                    s_i32_0 = 0i32;
                    l50 = s_i32_0;
                    s_i32_0 = 16i32;
                    vm.enter_call()?;
                    let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
                    vm.exit_call();
                    let __t25 = __call_res?;
                    s_i32_0 = __t25;
                    l51 = s_i32_0;
                    s_i32_0 = l51;
                    s_i32_1 = l51;
                    s_i32_2 = 16i32;
                    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l51;
                    s_i32_1 = 0i32;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l51;
                    s_i32_1 = 0i32;
                    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                    s_i32_0 = l51;
                    s_i32_1 = 1i32;
                    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                    s_i32_0 = l51;
                    l52 = s_i32_0;
                    s_i64_0 = l1;
                    l53 = s_i64_0;
                    s_i32_0 = 24i32;
                    vm.enter_call()?;
                    let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
                    vm.exit_call();
                    let __t26 = __call_res?;
                    s_i32_0 = __t26;
                    l54 = s_i32_0;
                    s_i32_0 = l54;
                    s_i32_1 = l48;
                    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l54;
                    s_i32_1 = l49;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l54;
                    s_i32_1 = l50;
                    ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                    s_i32_0 = l54;
                    s_i32_1 = l52;
                    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                    s_i32_0 = l54;
                    s_i64_1 = l53;
                    ops::i64_store(vm, 0, s_i32_0, 16u32, s_i64_1)?;
                    s_i32_0 = l54;
                    l55 = s_i32_0;
                    s_i32_0 = l55;
                    return Ok(s_i32_0);
                } else {
                }
            }
            s_i32_0 = l31;
            s_i32_1 = l32;
            s_i32_2 = 127i32;
            vm.enter_call()?;
            let __call_res = Self::s_304f22c4ab8c(vm, s_i32_1, s_i32_2, host);
            vm.exit_call();
            let __t27 = __call_res?;
            s_i32_1 = __t27;
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_b8acef2c4d16(vm, s_i32_1, host);
            vm.exit_call();
            let __t28 = __call_res?;
            s_i32_1 = __t28;
            vm.enter_call()?;
            let __call_res = Self::s_5daf123ed54b(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i64_0 = l29;
            s_i64_1 = 1i64;
            vm.enter_call()?;
            let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
            vm.exit_call();
            let __t29 = __call_res?;
            s_i64_0 = __t29;
            l29 = s_i64_0;
            continue 'blk_6;
            break 'blk_6;
        }
    }
    s_i32_0 = 0i32;
    l56 = s_i32_0;
    s_i32_0 = 0i32;
    l57 = s_i32_0;
    s_i32_0 = 0i32;
    l58 = s_i32_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
    vm.exit_call();
    let __t30 = __call_res?;
    s_i32_0 = __t30;
    l59 = s_i32_0;
    s_i32_0 = l59;
    s_i32_1 = l59;
    s_i32_2 = 16i32;
    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l59;
    s_i32_1 = 0i32;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l59;
    s_i32_1 = 0i32;
    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
    s_i32_0 = l59;
    s_i32_1 = 1i32;
    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
    s_i32_0 = l59;
    l60 = s_i32_0;
    s_i64_0 = l1;
    l61 = s_i64_0;
    s_i32_0 = 24i32;
    vm.enter_call()?;
    let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
    vm.exit_call();
    let __t31 = __call_res?;
    s_i32_0 = __t31;
    l62 = s_i32_0;
    s_i32_0 = l62;
    s_i32_1 = l56;
    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l62;
    s_i32_1 = l57;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l62;
    s_i32_1 = l58;
    ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
    s_i32_0 = l62;
    s_i32_1 = l60;
    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
    s_i32_0 = l62;
    s_i64_1 = l61;
    ops::i64_store(vm, 0, s_i32_0, 16u32, s_i64_1)?;
    s_i32_0 = l62;
    l63 = s_i32_0;
    s_i32_0 = l63;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_259f493e8e91(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut l3: i64 = 0i64;
    let mut l4: i64 = 0i64;
    let mut l5: i64 = 0i64;
    let mut l6: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i64_2: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_1 = l1;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_d0df0f4c4018(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l2 = s_i32_0;
    s_i32_0 = l2;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
    l3 = s_i64_0;
    s_i32_0 = l0;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
    l4 = s_i64_0;
    s_i32_0 = l1;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
    l5 = s_i64_0;
    s_i64_0 = l3;
    s_i64_1 = l4;
    s_i64_0 = ops::i64_xor(s_i64_0, s_i64_1);
    s_i64_1 = l3;
    s_i64_2 = l5;
    s_i64_1 = ops::i64_xor(s_i64_1, s_i64_2);
    s_i64_0 = ops::i64_and(s_i64_0, s_i64_1);
    s_i64_1 = 0i64;
    s_i32_0 = ops::i64_ge_s(s_i64_0, s_i64_1);
    l6 = s_i32_0;
    s_i32_0 = l6;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_2604e97131e1(vm: &mut SubstituteVm, p0: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i64, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i64 = 0i64;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 0u32)?;
    l1 = s_i64_0;
    s_i64_0 = l1;
    return Ok(s_i64_0);
    }

    pub(crate) fn s_2769f9b32b2b(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_1 = l1;
    vm.enter_call()?;
    let __call_res = Self::s_5b883f251b47(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    s_i32_1 = 0i32;
    s_i32_0 = ops::i32_le_s(s_i32_0, s_i32_1);
    l2 = s_i32_0;
    s_i32_0 = l2;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn s_27df125e1a57(vm: &mut SubstituteVm, p0: i32, g0: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i64 = 0i64;
    let mut l2: i32 = 0i32;
    let mut l3: i64 = 0i64;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i32 = 0i32;
    let mut l7: i64 = 0i64;
    let mut l8: i32 = 0i32;
    let mut l9: i32 = 0i32;
    let mut l10: i32 = 0i32;
    let mut l11: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i32_2: i32 = 0i32;
    let mut s_i64_2: i64 = 0i64;
    let mut s_i32_3: i32 = 0i32;
    let mut s_i32_4: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
    l1 = s_i64_0;
    s_i64_0 = 10i64;
    s_i64_1 = l1;
    s_i64_2 = 2i64;
    vm.enter_call()?;
    let __call_res = Self::s_1c917789866c(vm, s_i64_1, s_i64_2, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i64_1 = __t0;
    s_i64_2 = 3i64;
    s_i64_1 = ops::i64_div_u(s_i64_1, s_i64_2)?;
    s_i64_2 = 4i64;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_8dcb06005c72(vm, s_i64_1, s_i64_2, host);
    vm.exit_call();
    let __t1 = __call_res?;
    s_i64_1 = __t1;
    vm.enter_call()?;
    let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
    vm.exit_call();
    let __t2 = __call_res?;
    s_i64_0 = __t2;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_c5da14f612f0(vm, s_i64_0, host);
    vm.exit_call();
    let __t3 = __call_res?;
    s_i32_0 = __t3;
    l2 = s_i32_0;
    s_i32_0 = l2;
    s_i32_1 = 203i32;
    vm.enter_call()?;
    let __call_res = Self::s_1232b68aede8(vm, s_i32_1, host);
    vm.exit_call();
    let __t4 = __call_res?;
    s_i32_1 = __t4;
    vm.enter_call()?;
    let __call_res = Self::s_697cb42b9ce7(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l2;
    s_i64_1 = l1;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_e04446e5e582(vm, s_i64_1, g0, host);
    vm.exit_call();
    let __t5 = __call_res?;
    s_i32_1 = __t5;
    vm.enter_call()?;
    let __call_res = Self::s_1230bc006319(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    __call_res?;
    s_i64_0 = 0i64;
    l3 = s_i64_0;
    'blk_1: {
        'blk_2: loop {
            vm.deduct(host, 1)?;
            s_i64_0 = l3;
            s_i64_1 = 3i64;
            vm.enter_call()?;
            let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
            vm.exit_call();
            let __t6 = __call_res?;
            s_i64_0 = __t6;
            s_i64_1 = l1;
            s_i32_0 = ops::i64_le_u(s_i64_0, s_i64_1);
            s_i32_0 = ops::i32_eqz(s_i32_0);
            if s_i32_0 != 0 {
                break 'blk_1;
            }
            s_i32_0 = l0;
            s_i64_1 = l3;
            vm.enter_call()?;
            let __call_res = Self::s_4e6fb2273c0f(vm, s_i32_0, s_i64_1, host);
            vm.exit_call();
            let __t7 = __call_res?;
            s_i32_0 = __t7;
            l4 = s_i32_0;
            s_i32_0 = l0;
            s_i64_1 = l3;
            s_i64_2 = 1i64;
            vm.enter_call()?;
            let __call_res = Self::s_1c917789866c(vm, s_i64_1, s_i64_2, host);
            vm.exit_call();
            let __t8 = __call_res?;
            s_i64_1 = __t8;
            vm.enter_call()?;
            let __call_res = Self::s_4e6fb2273c0f(vm, s_i32_0, s_i64_1, host);
            vm.exit_call();
            let __t9 = __call_res?;
            s_i32_0 = __t9;
            l5 = s_i32_0;
            s_i32_0 = l0;
            s_i64_1 = l3;
            s_i64_2 = 2i64;
            vm.enter_call()?;
            let __call_res = Self::s_1c917789866c(vm, s_i64_1, s_i64_2, host);
            vm.exit_call();
            let __t10 = __call_res?;
            s_i64_1 = __t10;
            vm.enter_call()?;
            let __call_res = Self::s_4e6fb2273c0f(vm, s_i32_0, s_i64_1, host);
            vm.exit_call();
            let __t11 = __call_res?;
            s_i32_0 = __t11;
            l6 = s_i32_0;
            s_i32_0 = l2;
            s_i32_1 = 127i32;
            s_i32_2 = l4;
            s_i32_3 = 2i32;
            vm.enter_call()?;
            let __call_res = Self::s_708c3053a6ba(vm, s_i32_2, s_i32_3, host);
            vm.exit_call();
            let __t12 = __call_res?;
            s_i32_2 = __t12;
            s_i32_3 = 255i32;
            s_i32_2 = ops::i32_and(s_i32_2, s_i32_3);
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_91b7532ed2b6(vm, s_i32_1, s_i32_2, host);
            vm.exit_call();
            let __t13 = __call_res?;
            s_i32_1 = __t13;
            vm.enter_call()?;
            let __call_res = Self::s_1232b68aede8(vm, s_i32_1, host);
            vm.exit_call();
            let __t14 = __call_res?;
            s_i32_1 = __t14;
            vm.enter_call()?;
            let __call_res = Self::s_697cb42b9ce7(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l2;
            s_i32_1 = 127i32;
            s_i32_2 = l4;
            s_i32_3 = 3i32;
            s_i32_2 = ops::i32_and(s_i32_2, s_i32_3);
            s_i32_3 = 255i32;
            s_i32_2 = ops::i32_and(s_i32_2, s_i32_3);
            s_i32_3 = 4i32;
            vm.enter_call()?;
            let __call_res = Self::s_1e670f572c93(vm, s_i32_2, s_i32_3, host);
            vm.exit_call();
            let __t15 = __call_res?;
            s_i32_2 = __t15;
            s_i32_3 = l5;
            s_i32_4 = 4i32;
            vm.enter_call()?;
            let __call_res = Self::s_708c3053a6ba(vm, s_i32_3, s_i32_4, host);
            vm.exit_call();
            let __t16 = __call_res?;
            s_i32_3 = __t16;
            s_i32_2 = ops::i32_or(s_i32_2, s_i32_3);
            s_i32_3 = 255i32;
            s_i32_2 = ops::i32_and(s_i32_2, s_i32_3);
            s_i32_3 = 255i32;
            s_i32_2 = ops::i32_and(s_i32_2, s_i32_3);
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_91b7532ed2b6(vm, s_i32_1, s_i32_2, host);
            vm.exit_call();
            let __t17 = __call_res?;
            s_i32_1 = __t17;
            vm.enter_call()?;
            let __call_res = Self::s_1232b68aede8(vm, s_i32_1, host);
            vm.exit_call();
            let __t18 = __call_res?;
            s_i32_1 = __t18;
            vm.enter_call()?;
            let __call_res = Self::s_697cb42b9ce7(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l2;
            s_i32_1 = 127i32;
            s_i32_2 = l5;
            s_i32_3 = 15i32;
            s_i32_2 = ops::i32_and(s_i32_2, s_i32_3);
            s_i32_3 = 255i32;
            s_i32_2 = ops::i32_and(s_i32_2, s_i32_3);
            s_i32_3 = 2i32;
            vm.enter_call()?;
            let __call_res = Self::s_1e670f572c93(vm, s_i32_2, s_i32_3, host);
            vm.exit_call();
            let __t19 = __call_res?;
            s_i32_2 = __t19;
            s_i32_3 = l6;
            s_i32_4 = 6i32;
            vm.enter_call()?;
            let __call_res = Self::s_708c3053a6ba(vm, s_i32_3, s_i32_4, host);
            vm.exit_call();
            let __t20 = __call_res?;
            s_i32_3 = __t20;
            s_i32_2 = ops::i32_or(s_i32_2, s_i32_3);
            s_i32_3 = 255i32;
            s_i32_2 = ops::i32_and(s_i32_2, s_i32_3);
            s_i32_3 = 255i32;
            s_i32_2 = ops::i32_and(s_i32_2, s_i32_3);
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_91b7532ed2b6(vm, s_i32_1, s_i32_2, host);
            vm.exit_call();
            let __t21 = __call_res?;
            s_i32_1 = __t21;
            vm.enter_call()?;
            let __call_res = Self::s_1232b68aede8(vm, s_i32_1, host);
            vm.exit_call();
            let __t22 = __call_res?;
            s_i32_1 = __t22;
            vm.enter_call()?;
            let __call_res = Self::s_697cb42b9ce7(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l2;
            s_i32_1 = 127i32;
            s_i32_2 = l6;
            s_i32_3 = 63i32;
            s_i32_2 = ops::i32_and(s_i32_2, s_i32_3);
            s_i32_3 = 255i32;
            s_i32_2 = ops::i32_and(s_i32_2, s_i32_3);
            s_i32_3 = 255i32;
            s_i32_2 = ops::i32_and(s_i32_2, s_i32_3);
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_91b7532ed2b6(vm, s_i32_1, s_i32_2, host);
            vm.exit_call();
            let __t23 = __call_res?;
            s_i32_1 = __t23;
            vm.enter_call()?;
            let __call_res = Self::s_1232b68aede8(vm, s_i32_1, host);
            vm.exit_call();
            let __t24 = __call_res?;
            s_i32_1 = __t24;
            vm.enter_call()?;
            let __call_res = Self::s_697cb42b9ce7(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i64_0 = l3;
            s_i64_1 = 3i64;
            vm.enter_call()?;
            let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
            vm.exit_call();
            let __t25 = __call_res?;
            s_i64_0 = __t25;
            l3 = s_i64_0;
            continue 'blk_2;
            break 'blk_2;
        }
    }
    s_i64_0 = l1;
    s_i64_1 = l3;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_aa4577c560e8(vm, s_i64_0, s_i64_1, host);
    vm.exit_call();
    let __t26 = __call_res?;
    s_i64_0 = __t26;
    l7 = s_i64_0;
    s_i64_0 = l7;
    s_i64_1 = 1i64;
    s_i32_0 = ops::i64_eq(s_i64_0, s_i64_1);
    'blk_3: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i64_1 = l3;
            vm.enter_call()?;
            let __call_res = Self::s_4e6fb2273c0f(vm, s_i32_0, s_i64_1, host);
            vm.exit_call();
            let __t27 = __call_res?;
            s_i32_0 = __t27;
            l8 = s_i32_0;
            s_i32_0 = l2;
            s_i32_1 = 127i32;
            s_i32_2 = l8;
            s_i32_3 = 2i32;
            vm.enter_call()?;
            let __call_res = Self::s_708c3053a6ba(vm, s_i32_2, s_i32_3, host);
            vm.exit_call();
            let __t28 = __call_res?;
            s_i32_2 = __t28;
            s_i32_3 = 255i32;
            s_i32_2 = ops::i32_and(s_i32_2, s_i32_3);
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_91b7532ed2b6(vm, s_i32_1, s_i32_2, host);
            vm.exit_call();
            let __t29 = __call_res?;
            s_i32_1 = __t29;
            vm.enter_call()?;
            let __call_res = Self::s_1232b68aede8(vm, s_i32_1, host);
            vm.exit_call();
            let __t30 = __call_res?;
            s_i32_1 = __t30;
            vm.enter_call()?;
            let __call_res = Self::s_697cb42b9ce7(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l2;
            s_i32_1 = 127i32;
            s_i32_2 = l8;
            s_i32_3 = 3i32;
            s_i32_2 = ops::i32_and(s_i32_2, s_i32_3);
            s_i32_3 = 255i32;
            s_i32_2 = ops::i32_and(s_i32_2, s_i32_3);
            s_i32_3 = 4i32;
            vm.enter_call()?;
            let __call_res = Self::s_1e670f572c93(vm, s_i32_2, s_i32_3, host);
            vm.exit_call();
            let __t31 = __call_res?;
            s_i32_2 = __t31;
            s_i32_3 = 255i32;
            s_i32_2 = ops::i32_and(s_i32_2, s_i32_3);
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_91b7532ed2b6(vm, s_i32_1, s_i32_2, host);
            vm.exit_call();
            let __t32 = __call_res?;
            s_i32_1 = __t32;
            vm.enter_call()?;
            let __call_res = Self::s_1232b68aede8(vm, s_i32_1, host);
            vm.exit_call();
            let __t33 = __call_res?;
            s_i32_1 = __t33;
            vm.enter_call()?;
            let __call_res = Self::s_697cb42b9ce7(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l2;
            s_i32_1 = 195i32;
            vm.enter_call()?;
            let __call_res = Self::s_1232b68aede8(vm, s_i32_1, host);
            vm.exit_call();
            let __t34 = __call_res?;
            s_i32_1 = __t34;
            vm.enter_call()?;
            let __call_res = Self::s_697cb42b9ce7(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l2;
            s_i32_1 = 195i32;
            vm.enter_call()?;
            let __call_res = Self::s_1232b68aede8(vm, s_i32_1, host);
            vm.exit_call();
            let __t35 = __call_res?;
            s_i32_1 = __t35;
            vm.enter_call()?;
            let __call_res = Self::s_697cb42b9ce7(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
        } else {
            s_i64_0 = l7;
            s_i64_1 = 2i64;
            s_i32_0 = ops::i64_eq(s_i64_0, s_i64_1);
            'blk_4: {
                if s_i32_0 != 0 {
                    s_i32_0 = l0;
                    s_i64_1 = l3;
                    vm.enter_call()?;
                    let __call_res = Self::s_4e6fb2273c0f(vm, s_i32_0, s_i64_1, host);
                    vm.exit_call();
                    let __t36 = __call_res?;
                    s_i32_0 = __t36;
                    l9 = s_i32_0;
                    s_i32_0 = l0;
                    s_i64_1 = l3;
                    s_i64_2 = 1i64;
                    vm.enter_call()?;
                    let __call_res = Self::s_1c917789866c(vm, s_i64_1, s_i64_2, host);
                    vm.exit_call();
                    let __t37 = __call_res?;
                    s_i64_1 = __t37;
                    vm.enter_call()?;
                    let __call_res = Self::s_4e6fb2273c0f(vm, s_i32_0, s_i64_1, host);
                    vm.exit_call();
                    let __t38 = __call_res?;
                    s_i32_0 = __t38;
                    l10 = s_i32_0;
                    s_i32_0 = l2;
                    s_i32_1 = 127i32;
                    s_i32_2 = l9;
                    s_i32_3 = 2i32;
                    vm.enter_call()?;
                    let __call_res = Self::s_708c3053a6ba(vm, s_i32_2, s_i32_3, host);
                    vm.exit_call();
                    let __t39 = __call_res?;
                    s_i32_2 = __t39;
                    s_i32_3 = 255i32;
                    s_i32_2 = ops::i32_and(s_i32_2, s_i32_3);
                    vm.enter_call()?;
                    let __call_res = crate::shared_1::Shared::s_91b7532ed2b6(vm, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    let __t40 = __call_res?;
                    s_i32_1 = __t40;
                    vm.enter_call()?;
                    let __call_res = Self::s_1232b68aede8(vm, s_i32_1, host);
                    vm.exit_call();
                    let __t41 = __call_res?;
                    s_i32_1 = __t41;
                    vm.enter_call()?;
                    let __call_res = Self::s_697cb42b9ce7(vm, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l2;
                    s_i32_1 = 127i32;
                    s_i32_2 = l9;
                    s_i32_3 = 3i32;
                    s_i32_2 = ops::i32_and(s_i32_2, s_i32_3);
                    s_i32_3 = 255i32;
                    s_i32_2 = ops::i32_and(s_i32_2, s_i32_3);
                    s_i32_3 = 4i32;
                    vm.enter_call()?;
                    let __call_res = Self::s_1e670f572c93(vm, s_i32_2, s_i32_3, host);
                    vm.exit_call();
                    let __t42 = __call_res?;
                    s_i32_2 = __t42;
                    s_i32_3 = l10;
                    s_i32_4 = 4i32;
                    vm.enter_call()?;
                    let __call_res = Self::s_708c3053a6ba(vm, s_i32_3, s_i32_4, host);
                    vm.exit_call();
                    let __t43 = __call_res?;
                    s_i32_3 = __t43;
                    s_i32_2 = ops::i32_or(s_i32_2, s_i32_3);
                    s_i32_3 = 255i32;
                    s_i32_2 = ops::i32_and(s_i32_2, s_i32_3);
                    s_i32_3 = 255i32;
                    s_i32_2 = ops::i32_and(s_i32_2, s_i32_3);
                    vm.enter_call()?;
                    let __call_res = crate::shared_1::Shared::s_91b7532ed2b6(vm, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    let __t44 = __call_res?;
                    s_i32_1 = __t44;
                    vm.enter_call()?;
                    let __call_res = Self::s_1232b68aede8(vm, s_i32_1, host);
                    vm.exit_call();
                    let __t45 = __call_res?;
                    s_i32_1 = __t45;
                    vm.enter_call()?;
                    let __call_res = Self::s_697cb42b9ce7(vm, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l2;
                    s_i32_1 = 127i32;
                    s_i32_2 = l10;
                    s_i32_3 = 15i32;
                    s_i32_2 = ops::i32_and(s_i32_2, s_i32_3);
                    s_i32_3 = 255i32;
                    s_i32_2 = ops::i32_and(s_i32_2, s_i32_3);
                    s_i32_3 = 2i32;
                    vm.enter_call()?;
                    let __call_res = Self::s_1e670f572c93(vm, s_i32_2, s_i32_3, host);
                    vm.exit_call();
                    let __t46 = __call_res?;
                    s_i32_2 = __t46;
                    s_i32_3 = 255i32;
                    s_i32_2 = ops::i32_and(s_i32_2, s_i32_3);
                    vm.enter_call()?;
                    let __call_res = crate::shared_1::Shared::s_91b7532ed2b6(vm, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    let __t47 = __call_res?;
                    s_i32_1 = __t47;
                    vm.enter_call()?;
                    let __call_res = Self::s_1232b68aede8(vm, s_i32_1, host);
                    vm.exit_call();
                    let __t48 = __call_res?;
                    s_i32_1 = __t48;
                    vm.enter_call()?;
                    let __call_res = Self::s_697cb42b9ce7(vm, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l2;
                    s_i32_1 = 195i32;
                    vm.enter_call()?;
                    let __call_res = Self::s_1232b68aede8(vm, s_i32_1, host);
                    vm.exit_call();
                    let __t49 = __call_res?;
                    s_i32_1 = __t49;
                    vm.enter_call()?;
                    let __call_res = Self::s_697cb42b9ce7(vm, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                } else {
                }
            }
        }
    }
    s_i32_0 = l2;
    s_i32_1 = 204i32;
    vm.enter_call()?;
    let __call_res = Self::s_1232b68aede8(vm, s_i32_1, host);
    vm.exit_call();
    let __t50 = __call_res?;
    s_i32_1 = __t50;
    vm.enter_call()?;
    let __call_res = Self::s_697cb42b9ce7(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l2;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_df3679579582(vm, s_i32_0, host);
    vm.exit_call();
    let __t51 = __call_res?;
    s_i32_0 = __t51;
    l11 = s_i32_0;
    s_i32_0 = l11;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_289c1ec0e82a(vm: &mut SubstituteVm, p0: i32, g0: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = 0i32;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i32 = 0i32;
    let mut l7: i32 = 0i32;
    let mut l8: i32 = 0i32;
    let mut l9: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
    s_i64_1 = 0i64;
    s_i32_0 = ops::i64_eq(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 0u32)?;
            s_i64_1 = 4294967295i64;
            s_i32_0 = ops::i64_le_u(s_i64_0, s_i64_1);
        } else {
            s_i32_0 = 0i32;
        }
    }
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
            l1 = s_i32_0;
            s_i32_0 = l0;
            s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 0u32)?;
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_d5f50d7e651d(vm, s_i64_0, host);
            vm.exit_call();
            let __t0 = __call_res?;
            s_i32_0 = __t0;
            l2 = s_i32_0;
            s_i32_0 = 0i32;
            l3 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            l4 = s_i32_0;
            s_i32_0 = l4;
            s_i32_1 = l1;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l4;
            s_i32_1 = l2;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l4;
            s_i32_1 = l3;
            ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l4;
        } else {
            s_i32_0 = 0i32;
            l5 = s_i32_0;
            s_i32_0 = 0i32;
            l6 = s_i32_0;
            s_i32_0 = vm.get_i32(g0)?;
            l7 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_0 = __t2;
            l8 = s_i32_0;
            s_i32_0 = l8;
            s_i32_1 = l5;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l8;
            s_i32_1 = l6;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l8;
            s_i32_1 = l7;
            ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l8;
        }
    }
    l9 = s_i32_0;
    s_i32_0 = l9;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn s_2ddc63e083bc(vm: &mut SubstituteVm, p0: i32, p1: i64, g0: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i64, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i64 = p1;
    let mut l2: i64 = 0i64;
    let mut l3: i64 = 0i64;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i64 = 0i64;
    let mut l7: i32 = 0i32;
    let mut l8: i64 = 0i64;
    let mut l9: i64 = 0i64;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
    l2 = s_i64_0;
    s_i64_0 = l1;
    s_i64_1 = l2;
    s_i32_0 = ops::i64_ge_u(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i64_0 = l1;
            l3 = s_i64_0;
            s_i64_0 = l3;
            return Ok(s_i64_0);
        } else {
        }
    }
    s_i32_0 = l0;
    s_i64_1 = l1;
    vm.enter_call()?;
    let __call_res = Self::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    vm.enter_call()?;
    let __call_res = Self::s_62439e6f6ae7(vm, s_i32_0, g0, host);
    vm.exit_call();
    let __t1 = __call_res?;
    s_i32_0 = __t1;
    l4 = s_i32_0;
    s_i32_0 = l4;
    s_i32_1 = 246i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
        } else {
            s_i32_0 = l4;
            s_i32_1 = 247i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
        }
    }
    'blk_3: {
        if s_i32_0 != 0 {
            s_i32_0 = 248i32;
        } else {
            s_i32_0 = 251i32;
        }
    }
    l5 = s_i32_0;
    s_i64_0 = l1;
    s_i64_1 = 1i64;
    vm.enter_call()?;
    let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
    vm.exit_call();
    let __t2 = __call_res?;
    s_i64_0 = __t2;
    l6 = s_i64_0;
    'blk_4: {
        'blk_5: loop {
            vm.deduct(host, 1)?;
            s_i64_0 = l6;
            s_i64_1 = l2;
            s_i32_0 = ops::i64_lt_u(s_i64_0, s_i64_1);
            s_i32_0 = ops::i32_eqz(s_i32_0);
            if s_i32_0 != 0 {
                break 'blk_4;
            }
            s_i32_0 = l0;
            s_i64_1 = l6;
            vm.enter_call()?;
            let __call_res = Self::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i32_0 = __t3;
            vm.enter_call()?;
            let __call_res = Self::s_62439e6f6ae7(vm, s_i32_0, g0, host);
            vm.exit_call();
            let __t4 = __call_res?;
            s_i32_0 = __t4;
            l7 = s_i32_0;
            s_i32_0 = l7;
            s_i32_1 = 255i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_6: {
                if s_i32_0 != 0 {
                    s_i64_0 = l6;
                    s_i64_1 = 2i64;
                    vm.enter_call()?;
                    let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
                    vm.exit_call();
                    let __t5 = __call_res?;
                    s_i64_0 = __t5;
                    l6 = s_i64_0;
                } else {
                    s_i32_0 = l7;
                    s_i32_1 = l5;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                    'blk_7: {
                        if s_i32_0 != 0 {
                            s_i64_0 = l6;
                            s_i64_1 = 1i64;
                            vm.enter_call()?;
                            let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
                            vm.exit_call();
                            let __t6 = __call_res?;
                            s_i64_0 = __t6;
                            l8 = s_i64_0;
                            s_i64_0 = l8;
                            return Ok(s_i64_0);
                        } else {
                            s_i64_0 = l6;
                            s_i64_1 = 1i64;
                            vm.enter_call()?;
                            let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
                            vm.exit_call();
                            let __t7 = __call_res?;
                            s_i64_0 = __t7;
                            l6 = s_i64_0;
                        }
                    }
                }
            }
            continue 'blk_5;
            break 'blk_5;
        }
    }
    s_i64_0 = l2;
    l9 = s_i64_0;
    s_i64_0 = l9;
    return Ok(s_i64_0);
    }

    pub(crate) fn s_2f291b8074cc(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_1 = l1;
    vm.enter_call()?;
    let __call_res = Self::s_5b883f251b47(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    s_i32_1 = 0i32;
    s_i32_0 = ops::i32_ge_s(s_i32_0, s_i32_1);
    l2 = s_i32_0;
    s_i32_0 = l2;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_2f75c7656227(vm: &mut SubstituteVm, p0: i32, p1: i64, g0: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i64, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i64 = p1;
    let mut l2: i64 = 0i64;
    let mut l3: i64 = 0i64;
    let mut l4: i64 = 0i64;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i64_1: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
    l2 = s_i64_0;
    s_i64_0 = l1;
    l3 = s_i64_0;
    'blk_1: {
        'blk_2: loop {
            vm.deduct(host, 1)?;
            s_i64_0 = l3;
            s_i64_1 = l2;
            s_i32_0 = ops::i64_lt_u(s_i64_0, s_i64_1);
            'blk_3: {
                if s_i32_0 != 0 {
                    s_i32_0 = l0;
                    s_i64_1 = l3;
                    vm.enter_call()?;
                    let __call_res = Self::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
                    vm.exit_call();
                    let __t0 = __call_res?;
                    s_i32_0 = __t0;
                    vm.enter_call()?;
                    let __call_res = Self::s_3b56e5b236ab(vm, s_i32_0, g0, host);
                    vm.exit_call();
                    let __t1 = __call_res?;
                    s_i32_0 = __t1;
                } else {
                    s_i32_0 = 0i32;
                }
            }
            s_i32_0 = ops::i32_eqz(s_i32_0);
            if s_i32_0 != 0 {
                break 'blk_1;
            }
            s_i32_0 = l0;
            s_i64_1 = l3;
            vm.enter_call()?;
            let __call_res = Self::s_2ddc63e083bc(vm, s_i32_0, s_i64_1, g0, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i64_0 = __t2;
            l3 = s_i64_0;
            continue 'blk_2;
            break 'blk_2;
        }
    }
    s_i64_0 = l3;
    l4 = s_i64_0;
    s_i64_0 = l4;
    return Ok(s_i64_0);
    }

    pub(crate) fn s_30171acb6f3a(vm: &mut SubstituteVm, p0: i32, p1: i64, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i64 = p1;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    vm.deduct(host, 1)?;
    s_i64_0 = l1;
    s_i64_1 = 536854464i64;
    s_i32_0 = ops::i64_gt_u(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i64_0 = l1;
    s_i32_0 = ops::i32_wrap_i64(s_i64_0);
    l2 = s_i32_0;
    s_i32_0 = l2;
    s_i32_1 = 2i32;
    s_i32_0 = ops::i32_shl(s_i32_0, s_i32_1);
    l3 = s_i32_0;
    s_i32_0 = 16i32;
    s_i32_1 = l3;
    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
    vm.enter_call()?;
    let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l4 = s_i32_0;
    s_i32_0 = l4;
    s_i32_1 = 16i32;
    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
    l5 = s_i32_0;
    s_i32_0 = 0i32;
    l6 = s_i32_0;
    'blk_2: {
        'blk_3: loop {
            vm.deduct(host, 1)?;
            s_i32_0 = l6;
            s_i32_1 = l3;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            if s_i32_0 != 0 {
                break 'blk_2;
            }
            s_i32_0 = l5;
            s_i32_1 = l6;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            s_i32_1 = l0;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l6;
            s_i32_1 = 4i32;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l6 = s_i32_0;
            continue 'blk_3;
            break 'blk_3;
        }
    }
    s_i32_0 = l4;
    s_i32_1 = l5;
    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l4;
    s_i32_1 = l2;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l4;
    s_i32_1 = l2;
    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
    s_i32_0 = l4;
    s_i32_1 = 0i32;
    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
    s_i32_0 = l4;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_304f22c4ab8c(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    s_i32_0 = l0;
    s_i32_1 = l1;
    s_i32_0 = ops::i32_lt_u(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = l0;
    s_i32_1 = l1;
    s_i32_0 = ops::i32_sub(s_i32_0, s_i32_1);
    return Ok(s_i32_0);
    }

    pub(crate) fn s_31be37cebf03(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, g1: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_1 = l1;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_846bfd421390(vm, s_i32_0, s_i32_1, g0, g1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l2 = s_i32_0;
    s_i32_0 = l2;
    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
    s_i32_0 = ops::i32_eqz(s_i32_0);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        } else {
            s_i32_0 = l2;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
        }
    }
    l3 = s_i32_0;
    s_i32_0 = l3;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_32c1e1d33325(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, g1: usize, g2: usize, g3: usize, g4: usize, g5: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_1 = l1;
    vm.enter_call()?;
    let __call_res = Self::s_1e656cbc7955(vm, s_i32_0, s_i32_1, g0, g1, g2, g3, g4, g5, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l2 = s_i32_0;
    s_i32_0 = l2;
    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
    s_i32_0 = ops::i32_eqz(s_i32_0);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        } else {
            s_i32_0 = l2;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
        }
    }
    l3 = s_i32_0;
    s_i32_0 = l3;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_33259fb77008(vm: &mut SubstituteVm, p0: i32, g0: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_1 = vm.get_i32(g0)?;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_84059c53f331(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    s_i32_0 = ops::i32_eqz(s_i32_0);
    l1 = s_i32_0;
    s_i32_0 = l1;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_34c286d1358b(vm: &mut SubstituteVm, p0: i64, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i64 = p0;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i64_1: i64 = 0i64;
    vm.deduct(host, 1)?;
    s_i64_0 = l0;
    s_i64_1 = 2147483647i64;
    s_i32_0 = ops::i64_gt_u(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i64_0 = l0;
    s_i32_0 = ops::i32_wrap_i64(s_i64_0);
    return Ok(s_i32_0);
    }

    pub(crate) fn s_3693c5d73f07(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i32_2: i32 = 0i32;
    vm.deduct(host, 1)?;
    s_i32_0 = l0;
    s_i32_1 = 255i32;
    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
    s_i32_1 = l1;
    s_i32_2 = 255i32;
    s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
    s_i32_0 = ops::i32_le_u(s_i32_0, s_i32_1);
    return Ok(s_i32_0);
    }

    pub(crate) fn s_375a3135d758(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, g1: usize, g2: usize, g3: usize, g4: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_1 = l1;
    vm.enter_call()?;
    let __call_res = Self::s_18330dd1a568(vm, s_i32_0, s_i32_1, g0, g1, g2, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    s_i32_0 = ops::i32_eqz(s_i32_0);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        } else {
            s_i32_0 = l0;
            s_i32_1 = l1;
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_a82938aed706(vm, s_i32_0, s_i32_1, g0, g3, g1, g2, g4, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
        }
    }
    l2 = s_i32_0;
    s_i32_0 = l2;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_37ab7a008474(vm: &mut SubstituteVm, p0: i32, g0: usize, g1: usize, g2: usize, g3: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = 0i32;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_ffcc61d1c500(vm, s_i32_0, g0, g1, g2, g3, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l1 = s_i32_0;
    s_i32_0 = l1;
    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
    s_i32_0 = ops::i32_eqz(s_i32_0);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            l2 = s_i32_0;
            s_i32_0 = 0i32;
            l3 = s_i32_0;
            s_i32_0 = vm.get_i32(g2)?;
            l4 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            l5 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = l2;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l5;
            s_i32_1 = l3;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l5;
            s_i32_1 = l4;
            ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l5;
        } else {
            s_i32_0 = l1;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
            let __t2 = s_i32_0;
            return Self::s_0e12a2af4587(vm, __t2, g2, host);
        }
    }
    l6 = s_i32_0;
    s_i32_0 = l6;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_383cd8b1812f(vm: &mut SubstituteVm, p0: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i64_1: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
    s_i64_1 = 0i64;
    s_i32_0 = ops::i64_eq(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 0u32)?;
            s_i64_1 = 1114112i64;
            s_i32_0 = ops::i64_ge_u(s_i64_0, s_i64_1);
            'blk_2: {
                if s_i32_0 != 0 {
                    s_i32_0 = l0;
                    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 0u32)?;
                    s_i64_1 = 2228223i64;
                    s_i32_0 = ops::i64_le_u(s_i64_0, s_i64_1);
                } else {
                    s_i32_0 = 0i32;
                }
            }
        } else {
            s_i32_0 = 0i32;
        }
    }
    l1 = s_i32_0;
    s_i32_0 = l1;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_3ac41cc6c31c(vm: &mut SubstituteVm, p0: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i64_1: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
    s_i64_1 = 0i64;
    s_i32_0 = ops::i64_eq(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 0u32)?;
            s_i64_1 = 2228224i64;
            s_i32_0 = ops::i64_ge_u(s_i64_0, s_i64_1);
            'blk_2: {
                if s_i32_0 != 0 {
                    s_i32_0 = l0;
                    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 0u32)?;
                    s_i64_1 = 3342335i64;
                    s_i32_0 = ops::i64_le_u(s_i64_0, s_i64_1);
                } else {
                    s_i32_0 = 0i32;
                }
            }
        } else {
            s_i32_0 = 0i32;
        }
    }
    l1 = s_i32_0;
    s_i32_0 = l1;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_3b56e5b236ab(vm: &mut SubstituteVm, p0: i32, g0: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = 0i32;
    let mut l2: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    vm.enter_call()?;
    let __call_res = Self::s_62439e6f6ae7(vm, s_i32_0, g0, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l1 = s_i32_0;
    s_i32_0 = l1;
    s_i32_1 = 246i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
        } else {
            s_i32_0 = l1;
            s_i32_1 = 247i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_2: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                } else {
                    s_i32_0 = l1;
                    s_i32_1 = 249i32;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                    'blk_3: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 1i32;
                        } else {
                            s_i32_0 = l1;
                            s_i32_1 = 250i32;
                            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                        }
                    }
                }
            }
        }
    }
    l2 = s_i32_0;
    s_i32_0 = l2;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_3c2ef182f08d(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, g1: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_1 = l1;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_9d5206c8376a(vm, s_i32_0, s_i32_1, g0, g1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l2 = s_i32_0;
    s_i32_0 = l2;
    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
    s_i32_0 = ops::i32_eqz(s_i32_0);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        } else {
            s_i32_0 = l2;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
        }
    }
    l3 = s_i32_0;
    s_i32_0 = l3;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_3c36851dd02b(vm: &mut SubstituteVm, p0: i64, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i64 = p0;
    let mut l1: i64 = 0i64;
    let mut l2: i64 = 0i64;
    let mut l3: i64 = 0i64;
    let mut l4: i64 = 0i64;
    let mut l5: i32 = 0i32;
    let mut l6: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i64_1: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i64_0 = l0;
    l1 = s_i64_0;
    s_i64_0 = l0;
    s_i64_1 = 0i64;
    s_i32_0 = ops::i64_lt_s(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i64_0 = -1i64;
        } else {
            s_i64_0 = 0i64;
        }
    }
    l2 = s_i64_0;
    s_i64_0 = l1;
    l3 = s_i64_0;
    s_i64_0 = l2;
    l4 = s_i64_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
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
    s_i32_0 = l6;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_3cfae8586ad0(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, g1: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i32 = 0i32;
    let mut l7: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_1 = l1;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_bd22d8f9c2fc(vm, s_i32_0, s_i32_1, g0, g1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l2 = s_i32_0;
    s_i32_0 = l2;
    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
    l3 = s_i32_0;
    s_i32_0 = l2;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
    l4 = s_i32_0;
    s_i32_0 = l2;
    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 12u32)?;
    l5 = s_i32_0;
    s_i32_0 = 12i32;
    vm.enter_call()?;
    let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
    vm.exit_call();
    let __t1 = __call_res?;
    s_i32_0 = __t1;
    l6 = s_i32_0;
    s_i32_0 = l6;
    s_i32_1 = l3;
    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l6;
    s_i32_1 = l4;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l6;
    s_i32_1 = l5;
    ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
    s_i32_0 = l6;
    l7 = s_i32_0;
    s_i32_0 = l7;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn s_3dbd6afaf7f2(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i64 = 0i64;
    let mut l3: i64 = 0i64;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i32 = 0i32;
    let mut l7: i64 = 0i64;
    let mut l8: i64 = 0i64;
    let mut l9: i64 = 0i64;
    let mut l10: i64 = 0i64;
    let mut l11: i64 = 0i64;
    let mut l12: i64 = 0i64;
    let mut l13: i64 = 0i64;
    let mut l14: i64 = 0i64;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i32_2: i32 = 0i32;
    let mut s_i64_2: i64 = 0i64;
    let mut s_i64_3: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
    l2 = s_i64_0;
    s_i32_0 = l1;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
    l3 = s_i64_0;
    s_i64_0 = l2;
    s_i64_1 = 0i64;
    s_i32_0 = ops::i64_eq(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
        } else {
            s_i64_0 = l3;
            s_i64_1 = 0i64;
            s_i32_0 = ops::i64_eq(s_i64_0, s_i64_1);
        }
    }
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t0 = __call_res?;
            s_i32_0 = __t0;
            l4 = s_i32_0;
            s_i32_0 = l4;
            s_i32_1 = l4;
            s_i32_2 = 16i32;
            s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l4;
            s_i32_1 = 0i32;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l4;
            s_i32_1 = 0i32;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l4;
            s_i32_1 = 1i32;
            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
            s_i32_0 = l4;
            l5 = s_i32_0;
            s_i32_0 = l5;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i32_0 = 0i32;
    s_i64_1 = l2;
    s_i64_2 = l3;
    vm.enter_call()?;
    let __call_res = Self::s_1c917789866c(vm, s_i64_1, s_i64_2, host);
    vm.exit_call();
    let __t1 = __call_res?;
    s_i64_1 = __t1;
    vm.enter_call()?;
    let __call_res = Self::s_30171acb6f3a(vm, s_i32_0, s_i64_1, host);
    vm.exit_call();
    let __t2 = __call_res?;
    s_i32_0 = __t2;
    l6 = s_i32_0;
    s_i64_0 = 0i64;
    l7 = s_i64_0;
    'blk_3: {
        'blk_4: loop {
            vm.deduct(host, 1)?;
            s_i64_0 = l7;
            s_i64_1 = l2;
            s_i32_0 = ops::i64_lt_u(s_i64_0, s_i64_1);
            s_i32_0 = ops::i32_eqz(s_i32_0);
            if s_i32_0 != 0 {
                break 'blk_3;
            }
            s_i32_0 = l0;
            s_i64_1 = l7;
            vm.enter_call()?;
            let __call_res = Self::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i32_0 = __t3;
            s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
            l8 = s_i64_0;
            s_i64_0 = 0i64;
            l9 = s_i64_0;
            s_i64_0 = 0i64;
            l10 = s_i64_0;
            'blk_5: {
                'blk_6: loop {
                    vm.deduct(host, 1)?;
                    s_i64_0 = l10;
                    s_i64_1 = l3;
                    s_i32_0 = ops::i64_lt_u(s_i64_0, s_i64_1);
                    s_i32_0 = ops::i32_eqz(s_i32_0);
                    if s_i32_0 != 0 {
                        break 'blk_5;
                    }
                    s_i64_0 = l7;
                    s_i64_1 = l10;
                    vm.enter_call()?;
                    let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
                    vm.exit_call();
                    let __t4 = __call_res?;
                    s_i64_0 = __t4;
                    l11 = s_i64_0;
                    s_i32_0 = l6;
                    s_i64_1 = l11;
                    vm.enter_call()?;
                    let __call_res = Self::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
                    vm.exit_call();
                    let __t5 = __call_res?;
                    s_i32_0 = __t5;
                    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
                    l12 = s_i64_0;
                    s_i64_0 = l12;
                    s_i64_1 = l8;
                    s_i32_2 = l1;
                    s_i64_3 = l10;
                    vm.enter_call()?;
                    let __call_res = Self::s_17df4637c35a(vm, s_i32_2, s_i64_3, host);
                    vm.exit_call();
                    let __t6 = __call_res?;
                    s_i32_2 = __t6;
                    s_i64_2 = ops::i64_extend_i32_u(s_i32_2);
                    vm.enter_call()?;
                    let __call_res = crate::shared_1::Shared::s_8dcb06005c72(vm, s_i64_1, s_i64_2, host);
                    vm.exit_call();
                    let __t7 = __call_res?;
                    s_i64_1 = __t7;
                    s_i64_2 = l9;
                    vm.enter_call()?;
                    let __call_res = Self::s_1c917789866c(vm, s_i64_1, s_i64_2, host);
                    vm.exit_call();
                    let __t8 = __call_res?;
                    s_i64_1 = __t8;
                    vm.enter_call()?;
                    let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
                    vm.exit_call();
                    let __t9 = __call_res?;
                    s_i64_0 = __t9;
                    l13 = s_i64_0;
                    s_i32_0 = l6;
                    s_i64_1 = l11;
                    s_i64_2 = l13;
                    s_i64_3 = 4294967295i64;
                    s_i64_2 = ops::i64_and(s_i64_2, s_i64_3);
                    vm.enter_call()?;
                    let __call_res = crate::shared_1::Shared::s_d5f50d7e651d(vm, s_i64_2, host);
                    vm.exit_call();
                    let __t10 = __call_res?;
                    s_i32_2 = __t10;
                    vm.enter_call()?;
                    let __call_res = crate::shared_1::Shared::s_d7d900550897(vm, s_i32_0, s_i64_1, s_i32_2, host);
                    vm.exit_call();
                    __call_res?;
                    s_i64_0 = l13;
                    s_i32_1 = 32i32;
                    vm.enter_call()?;
                    let __call_res = Self::s_73a866edda27(vm, s_i64_0, s_i32_1, host);
                    vm.exit_call();
                    let __t11 = __call_res?;
                    s_i64_0 = __t11;
                    l9 = s_i64_0;
                    s_i64_0 = l10;
                    s_i64_1 = 1i64;
                    vm.enter_call()?;
                    let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
                    vm.exit_call();
                    let __t12 = __call_res?;
                    s_i64_0 = __t12;
                    l10 = s_i64_0;
                    continue 'blk_6;
                    break 'blk_6;
                }
            }
            s_i64_0 = l7;
            s_i64_1 = l3;
            vm.enter_call()?;
            let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
            vm.exit_call();
            let __t13 = __call_res?;
            s_i64_0 = __t13;
            l14 = s_i64_0;
            s_i32_0 = l6;
            s_i64_1 = l14;
            s_i32_2 = l6;
            s_i64_3 = l14;
            vm.enter_call()?;
            let __call_res = Self::s_17df4637c35a(vm, s_i32_2, s_i64_3, host);
            vm.exit_call();
            let __t14 = __call_res?;
            s_i32_2 = __t14;
            s_i64_2 = ops::i64_extend_i32_u(s_i32_2);
            s_i64_3 = l9;
            vm.enter_call()?;
            let __call_res = Self::s_1c917789866c(vm, s_i64_2, s_i64_3, host);
            vm.exit_call();
            let __t15 = __call_res?;
            s_i64_2 = __t15;
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_d5f50d7e651d(vm, s_i64_2, host);
            vm.exit_call();
            let __t16 = __call_res?;
            s_i32_2 = __t16;
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_d7d900550897(vm, s_i32_0, s_i64_1, s_i32_2, host);
            vm.exit_call();
            __call_res?;
            s_i64_0 = l7;
            s_i64_1 = 1i64;
            vm.enter_call()?;
            let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
            vm.exit_call();
            let __t17 = __call_res?;
            s_i64_0 = __t17;
            l7 = s_i64_0;
            continue 'blk_4;
            break 'blk_4;
        }
    }
    s_i32_0 = l6;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_df3679579582(vm, s_i32_0, host);
    vm.exit_call();
    let __t18 = __call_res?;
    s_i32_0 = __t18;
    let __t19 = s_i32_0;
    return crate::shared_1::Shared::s_b92cf85b6153(vm, __t19, host);
    }

    pub(crate) fn s_3e7647da0841(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i32_2: i32 = 0i32;
    vm.deduct(host, 1)?;
    s_i32_0 = l0;
    s_i32_1 = 255i32;
    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
    s_i32_1 = l1;
    s_i32_2 = 255i32;
    s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
    return Ok(s_i32_0);
    }

    pub(crate) fn s_3eaf9106858e(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, g1: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_1 = l1;
    vm.enter_call()?;
    let __call_res = Self::s_82d03d870b76(vm, s_i32_0, s_i32_1, g0, g1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l2 = s_i32_0;
    s_i32_0 = l2;
    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
    s_i32_0 = ops::i32_eqz(s_i32_0);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        } else {
            s_i32_0 = l2;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
        }
    }
    l3 = s_i32_0;
    s_i32_0 = l3;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_404f103259e0(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_1 = l1;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_f6eb7e45de1d(vm, s_i32_1, g0, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_1 = __t0;
    let __t1 = s_i32_0;
    let __t2 = s_i32_1;
    return Self::s_1cbb4a12599c(vm, __t1, __t2, g0, host);
    }

    pub(crate) fn s_40c2979f136e(vm: &mut SubstituteVm, p0: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    s_i32_1 = 2i32;
    s_i32_0 = ops::i32_shr_u(s_i32_0, s_i32_1);
    return Ok(s_i32_0);
    }

    pub(crate) fn s_41e9bacc9fe3(vm: &mut SubstituteVm, p0: i64, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i64 = p0;
    let mut l1: i64 = 0i64;
    let mut l2: i64 = 0i64;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i64_1: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i64_0 = l0;
    l1 = s_i64_0;
    s_i64_0 = 0i64;
    l2 = s_i64_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l3 = s_i32_0;
    s_i32_0 = l3;
    s_i64_1 = l1;
    ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
    s_i32_0 = l3;
    s_i64_1 = l2;
    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
    s_i32_0 = l3;
    l4 = s_i32_0;
    s_i32_0 = l4;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn s_4200fb7655b9(vm: &mut SubstituteVm, p0: i32, g0: usize, g1: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
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
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            l1 = s_i32_0;
            s_i32_0 = vm.get_i32(g0)?;
            l2 = s_i32_0;
            s_i32_0 = vm.get_i32(g1)?;
            l3 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t0 = __call_res?;
            s_i32_0 = __t0;
            l4 = s_i32_0;
            s_i32_0 = l4;
            s_i32_1 = l1;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l4;
            s_i32_1 = l2;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l4;
            s_i32_1 = l3;
            ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l4;
            l5 = s_i32_0;
            s_i32_0 = l5;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    vm.enter_call()?;
    let __call_res = Self::s_0f7538eaa7db(vm, s_i32_0, g0, host);
    vm.exit_call();
    let __t1 = __call_res?;
    s_i32_0 = __t1;
    l6 = s_i32_0;
    s_i32_0 = l6;
    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
            l7 = s_i32_0;
            s_i32_0 = l6;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
            l8 = s_i32_0;
            s_i32_0 = 0i32;
            l9 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_0 = __t2;
            l10 = s_i32_0;
            s_i32_0 = l10;
            s_i32_1 = l7;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l10;
            s_i32_1 = l8;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l10;
            s_i32_1 = l9;
            ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l10;
        } else {
            s_i32_0 = 0i32;
            l11 = s_i32_0;
            s_i32_0 = vm.get_i32(g0)?;
            l12 = s_i32_0;
            s_i32_0 = vm.get_i32(g1)?;
            l13 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i32_0 = __t3;
            l14 = s_i32_0;
            s_i32_0 = l14;
            s_i32_1 = l11;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l14;
            s_i32_1 = l12;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l14;
            s_i32_1 = l13;
            ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l14;
        }
    }
    l15 = s_i32_0;
    s_i32_0 = l15;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_42e84b32c5de(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, g1: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_1 = l1;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_e0004f9831a9(vm, s_i32_0, s_i32_1, g0, g1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l2 = s_i32_0;
    s_i32_0 = l2;
    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
    s_i32_0 = ops::i32_eqz(s_i32_0);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        } else {
            s_i32_0 = l2;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
        }
    }
    l3 = s_i32_0;
    s_i32_0 = l3;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn s_4552c7e0e83b(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let mut l15: i64 = 0i64;
    let mut l16: i64 = 0i64;
    let mut l17: i64 = 0i64;
    let mut l18: i32 = 0i32;
    let mut l19: i64 = 0i64;
    let mut l20: i64 = 0i64;
    let mut l21: i64 = 0i64;
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
    let mut l33: i64 = 0i64;
    let mut l34: i64 = 0i64;
    let mut l35: i64 = 0i64;
    let mut l36: i32 = 0i32;
    let mut l37: i32 = 0i32;
    let mut l38: i32 = 0i32;
    let mut l39: i64 = 0i64;
    let mut l40: i32 = 0i32;
    let mut l41: i32 = 0i32;
    let mut l42: i64 = 0i64;
    let mut l43: i64 = 0i64;
    let mut l44: i64 = 0i64;
    let mut l45: i64 = 0i64;
    let mut l46: i32 = 0i32;
    let mut l47: i32 = 0i32;
    let mut l48: i32 = 0i32;
    let mut l49: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i32_2: i32 = 0i32;
    let mut s_i64_2: i64 = 0i64;
    let mut s_i32_3: i32 = 0i32;
    let mut s_i64_3: i64 = 0i64;
    let mut s_i32_4: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_1 = l1;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_86b08f7a2a43(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l2 = s_i32_0;
    s_i32_0 = l2;
    s_i32_1 = 0i32;
    s_i32_0 = ops::i32_lt_s(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            l3 = s_i32_0;
            s_i32_0 = l3;
            s_i32_1 = l3;
            s_i32_2 = 16i32;
            s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l3;
            s_i32_1 = 0i32;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l3;
            s_i32_1 = 0i32;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l3;
            s_i32_1 = 1i32;
            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
            s_i32_0 = l3;
            l4 = s_i32_0;
            s_i32_0 = l0;
            l5 = s_i32_0;
            s_i32_0 = 8i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_0 = __t2;
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
        } else {
        }
    }
    s_i32_0 = l2;
    s_i32_1 = 0i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
            l8 = s_i32_0;
            s_i32_0 = 20i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
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
            s_i32_1 = 1i32;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l9;
            s_i32_1 = 1i32;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l9;
            s_i32_1 = 1i32;
            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
            s_i32_0 = l9;
            s_i32_1 = l8;
            ops::i32_store(vm, 0, s_i32_0, 16u32, s_i32_1)?;
            s_i32_0 = l9;
            l10 = s_i32_0;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t4 = __call_res?;
            s_i32_0 = __t4;
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
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t5 = __call_res?;
            s_i32_0 = __t5;
            l13 = s_i32_0;
            s_i32_0 = l13;
            s_i32_1 = l10;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l13;
            s_i32_1 = l12;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l13;
            l14 = s_i32_0;
            s_i32_0 = l14;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
    l15 = s_i64_0;
    s_i32_0 = l1;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
    l16 = s_i64_0;
    s_i64_0 = l16;
    s_i64_1 = 1i64;
    s_i32_0 = ops::i64_eq(s_i64_0, s_i64_1);
    'blk_3: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            s_i64_1 = 0i64;
            vm.enter_call()?;
            let __call_res = Self::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
            vm.exit_call();
            let __t6 = __call_res?;
            s_i32_0 = __t6;
            s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
            l17 = s_i64_0;
            s_i32_0 = 0i32;
            s_i64_1 = l15;
            vm.enter_call()?;
            let __call_res = Self::s_30171acb6f3a(vm, s_i32_0, s_i64_1, host);
            vm.exit_call();
            let __t7 = __call_res?;
            s_i32_0 = __t7;
            l18 = s_i32_0;
            s_i64_0 = 0i64;
            l19 = s_i64_0;
            s_i64_0 = l15;
            l20 = s_i64_0;
            'blk_4: {
                'blk_5: loop {
                    vm.deduct(host, 1)?;
                    s_i64_0 = l20;
                    s_i64_1 = 0i64;
                    s_i32_0 = ops::i64_gt_u(s_i64_0, s_i64_1);
                    s_i32_0 = ops::i32_eqz(s_i32_0);
                    if s_i32_0 != 0 {
                        break 'blk_4;
                    }
                    s_i64_0 = l20;
                    s_i64_1 = 1i64;
                    vm.enter_call()?;
                    let __call_res = crate::shared_1::Shared::s_aa4577c560e8(vm, s_i64_0, s_i64_1, host);
                    vm.exit_call();
                    let __t8 = __call_res?;
                    s_i64_0 = __t8;
                    l20 = s_i64_0;
                    s_i64_0 = l19;
                    s_i32_1 = 32i32;
                    vm.enter_call()?;
                    let __call_res = Self::s_52dcb6d97135(vm, s_i64_0, s_i32_1, host);
                    vm.exit_call();
                    let __t9 = __call_res?;
                    s_i64_0 = __t9;
                    s_i32_1 = l0;
                    s_i64_2 = l20;
                    vm.enter_call()?;
                    let __call_res = Self::s_17df4637c35a(vm, s_i32_1, s_i64_2, host);
                    vm.exit_call();
                    let __t10 = __call_res?;
                    s_i32_1 = __t10;
                    s_i64_1 = ops::i64_extend_i32_u(s_i32_1);
                    s_i64_0 = ops::i64_or(s_i64_0, s_i64_1);
                    l21 = s_i64_0;
                    s_i32_0 = l18;
                    s_i64_1 = l20;
                    s_i64_2 = l21;
                    s_i64_3 = l17;
                    s_i64_2 = ops::i64_div_u(s_i64_2, s_i64_3)?;
                    vm.enter_call()?;
                    let __call_res = crate::shared_1::Shared::s_d5f50d7e651d(vm, s_i64_2, host);
                    vm.exit_call();
                    let __t11 = __call_res?;
                    s_i32_2 = __t11;
                    vm.enter_call()?;
                    let __call_res = crate::shared_1::Shared::s_d7d900550897(vm, s_i32_0, s_i64_1, s_i32_2, host);
                    vm.exit_call();
                    __call_res?;
                    s_i64_0 = l21;
                    s_i64_1 = l17;
                    s_i64_0 = ops::i64_rem_u(s_i64_0, s_i64_1)?;
                    l19 = s_i64_0;
                    continue 'blk_5;
                    break 'blk_5;
                }
            }
            s_i64_0 = l19;
            s_i64_1 = 0i64;
            s_i32_0 = ops::i64_eq(s_i64_0, s_i64_1);
            'blk_6: {
                if s_i32_0 != 0 {
                    s_i32_0 = 16i32;
                    vm.enter_call()?;
                    let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
                    vm.exit_call();
                    let __t12 = __call_res?;
                    s_i32_0 = __t12;
                    l22 = s_i32_0;
                    s_i32_0 = l22;
                    s_i32_1 = l22;
                    s_i32_2 = 16i32;
                    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l22;
                    s_i32_1 = 0i32;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l22;
                    s_i32_1 = 0i32;
                    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                    s_i32_0 = l22;
                    s_i32_1 = 1i32;
                    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                    s_i32_0 = l22;
                } else {
                    s_i64_0 = l19;
                    vm.enter_call()?;
                    let __call_res = crate::shared_1::Shared::s_d5f50d7e651d(vm, s_i64_0, host);
                    vm.exit_call();
                    let __t13 = __call_res?;
                    s_i32_0 = __t13;
                    l23 = s_i32_0;
                    s_i32_0 = 20i32;
                    vm.enter_call()?;
                    let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
                    vm.exit_call();
                    let __t14 = __call_res?;
                    s_i32_0 = __t14;
                    l24 = s_i32_0;
                    s_i32_0 = l24;
                    s_i32_1 = l24;
                    s_i32_2 = 16i32;
                    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l24;
                    s_i32_1 = 1i32;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l24;
                    s_i32_1 = 1i32;
                    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                    s_i32_0 = l24;
                    s_i32_1 = 1i32;
                    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                    s_i32_0 = l24;
                    s_i32_1 = l23;
                    ops::i32_store(vm, 0, s_i32_0, 16u32, s_i32_1)?;
                    s_i32_0 = l24;
                }
            }
            l25 = s_i32_0;
            s_i32_0 = l18;
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_df3679579582(vm, s_i32_0, host);
            vm.exit_call();
            let __t15 = __call_res?;
            s_i32_0 = __t15;
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_b92cf85b6153(vm, s_i32_0, host);
            vm.exit_call();
            let __t16 = __call_res?;
            s_i32_0 = __t16;
            l26 = s_i32_0;
            s_i32_0 = l25;
            l27 = s_i32_0;
            s_i32_0 = 8i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t17 = __call_res?;
            s_i32_0 = __t17;
            l28 = s_i32_0;
            s_i32_0 = l28;
            s_i32_1 = l26;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l28;
            s_i32_1 = l27;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l28;
            l29 = s_i32_0;
            s_i32_0 = l29;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i32_0 = 0i32;
    s_i64_1 = l15;
    vm.enter_call()?;
    let __call_res = Self::s_30171acb6f3a(vm, s_i32_0, s_i64_1, host);
    vm.exit_call();
    let __t18 = __call_res?;
    s_i32_0 = __t18;
    l30 = s_i32_0;
    s_i32_0 = 0i32;
    s_i64_1 = l16;
    s_i64_2 = 1i64;
    vm.enter_call()?;
    let __call_res = Self::s_1c917789866c(vm, s_i64_1, s_i64_2, host);
    vm.exit_call();
    let __t19 = __call_res?;
    s_i64_1 = __t19;
    vm.enter_call()?;
    let __call_res = Self::s_30171acb6f3a(vm, s_i32_0, s_i64_1, host);
    vm.exit_call();
    let __t20 = __call_res?;
    s_i32_0 = __t20;
    l31 = s_i32_0;
    s_i32_0 = l0;
    s_i64_1 = l15;
    s_i64_2 = 1i64;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_aa4577c560e8(vm, s_i64_1, s_i64_2, host);
    vm.exit_call();
    let __t21 = __call_res?;
    s_i64_1 = __t21;
    vm.enter_call()?;
    let __call_res = Self::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
    vm.exit_call();
    let __t22 = __call_res?;
    s_i32_0 = __t22;
    l32 = s_i32_0;
    s_i32_0 = 32i32;
    s_i32_1 = l32;
    s_i32_1 = ops::i32_clz(s_i32_1);
    vm.enter_call()?;
    let __call_res = Self::s_304f22c4ab8c(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t23 = __call_res?;
    s_i32_0 = __t23;
    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
    l33 = s_i64_0;
    s_i64_0 = l15;
    s_i64_1 = 1i64;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_aa4577c560e8(vm, s_i64_0, s_i64_1, host);
    vm.exit_call();
    let __t24 = __call_res?;
    s_i64_0 = __t24;
    s_i64_1 = 32i64;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_8dcb06005c72(vm, s_i64_0, s_i64_1, host);
    vm.exit_call();
    let __t25 = __call_res?;
    s_i64_0 = __t25;
    s_i64_1 = l33;
    vm.enter_call()?;
    let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
    vm.exit_call();
    let __t26 = __call_res?;
    s_i64_0 = __t26;
    l34 = s_i64_0;
    'blk_7: {
        'blk_8: loop {
            vm.deduct(host, 1)?;
            s_i64_0 = l34;
            s_i64_1 = 0i64;
            s_i32_0 = ops::i64_gt_u(s_i64_0, s_i64_1);
            s_i32_0 = ops::i32_eqz(s_i32_0);
            if s_i32_0 != 0 {
                break 'blk_7;
            }
            s_i64_0 = l34;
            s_i64_1 = 1i64;
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_aa4577c560e8(vm, s_i64_0, s_i64_1, host);
            vm.exit_call();
            let __t27 = __call_res?;
            s_i64_0 = __t27;
            l34 = s_i64_0;
            s_i64_0 = l34;
            s_i32_1 = 5i32;
            vm.enter_call()?;
            let __call_res = Self::s_73a866edda27(vm, s_i64_0, s_i32_1, host);
            vm.exit_call();
            let __t28 = __call_res?;
            s_i64_0 = __t28;
            l35 = s_i64_0;
            s_i64_0 = l34;
            s_i64_1 = 31i64;
            s_i64_0 = ops::i64_and(s_i64_0, s_i64_1);
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_d5f50d7e651d(vm, s_i64_0, host);
            vm.exit_call();
            let __t29 = __call_res?;
            s_i32_0 = __t29;
            l36 = s_i32_0;
            s_i32_0 = l0;
            s_i64_1 = l35;
            vm.enter_call()?;
            let __call_res = Self::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
            vm.exit_call();
            let __t30 = __call_res?;
            s_i32_0 = __t30;
            s_i32_1 = l36;
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_c6ab0a288667(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t31 = __call_res?;
            s_i32_0 = __t31;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
            l37 = s_i32_0;
            s_i32_0 = l37;
            l38 = s_i32_0;
            s_i64_0 = 0i64;
            l39 = s_i64_0;
            'blk_9: {
                'blk_10: loop {
                    vm.deduct(host, 1)?;
                    s_i64_0 = l39;
                    s_i64_1 = l16;
                    s_i32_0 = ops::i64_le_u(s_i64_0, s_i64_1);
                    s_i32_0 = ops::i32_eqz(s_i32_0);
                    if s_i32_0 != 0 {
                        break 'blk_9;
                    }
                    s_i32_0 = l31;
                    s_i64_1 = l39;
                    vm.enter_call()?;
                    let __call_res = Self::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
                    vm.exit_call();
                    let __t32 = __call_res?;
                    s_i32_0 = __t32;
                    l40 = s_i32_0;
                    s_i32_0 = l40;
                    s_i32_1 = 31i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_1::Shared::s_c6ab0a288667(vm, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t33 = __call_res?;
                    s_i32_0 = __t33;
                    l41 = s_i32_0;
                    s_i32_0 = l31;
                    s_i64_1 = l39;
                    s_i32_2 = l40;
                    s_i32_3 = 2147483647i32;
                    s_i32_2 = ops::i32_and(s_i32_2, s_i32_3);
                    s_i32_3 = 1i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_1::Shared::s_cc2d1c45c42a(vm, s_i32_2, s_i32_3, host);
                    vm.exit_call();
                    let __t34 = __call_res?;
                    s_i32_2 = __t34;
                    s_i32_3 = l38;
                    s_i32_2 = ops::i32_or(s_i32_2, s_i32_3);
                    vm.enter_call()?;
                    let __call_res = crate::shared_1::Shared::s_d7d900550897(vm, s_i32_0, s_i64_1, s_i32_2, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l41;
                    l38 = s_i32_0;
                    s_i64_0 = l39;
                    s_i64_1 = 1i64;
                    vm.enter_call()?;
                    let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
                    vm.exit_call();
                    let __t35 = __call_res?;
                    s_i64_0 = __t35;
                    l39 = s_i64_0;
                    continue 'blk_10;
                    break 'blk_10;
                }
            }
            s_i32_0 = l31;
            s_i32_1 = l1;
            vm.enter_call()?;
            let __call_res = Self::s_72a31b5ae287(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t36 = __call_res?;
            s_i32_0 = __t36;
            'blk_11: {
                if s_i32_0 != 0 {
                    s_i64_0 = 0i64;
                    l42 = s_i64_0;
                    s_i64_0 = 0i64;
                    l43 = s_i64_0;
                    'blk_12: {
                        'blk_13: loop {
                            vm.deduct(host, 1)?;
                            s_i64_0 = l43;
                            s_i64_1 = l16;
                            s_i32_0 = ops::i64_lt_u(s_i64_0, s_i64_1);
                            s_i32_0 = ops::i32_eqz(s_i32_0);
                            if s_i32_0 != 0 {
                                break 'blk_12;
                            }
                            s_i32_0 = l31;
                            s_i64_1 = l43;
                            vm.enter_call()?;
                            let __call_res = Self::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
                            vm.exit_call();
                            let __t37 = __call_res?;
                            s_i32_0 = __t37;
                            s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
                            l44 = s_i64_0;
                            s_i32_0 = l1;
                            s_i64_1 = l43;
                            vm.enter_call()?;
                            let __call_res = Self::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
                            vm.exit_call();
                            let __t38 = __call_res?;
                            s_i32_0 = __t38;
                            s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
                            s_i64_1 = l42;
                            vm.enter_call()?;
                            let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
                            vm.exit_call();
                            let __t39 = __call_res?;
                            s_i64_0 = __t39;
                            l45 = s_i64_0;
                            s_i64_0 = l44;
                            s_i64_1 = l45;
                            s_i32_0 = ops::i64_ge_u(s_i64_0, s_i64_1);
                            'blk_14: {
                                if s_i32_0 != 0 {
                                    s_i32_0 = l31;
                                    s_i64_1 = l43;
                                    s_i64_2 = l44;
                                    s_i64_3 = l45;
                                    vm.enter_call()?;
                                    let __call_res = crate::shared_1::Shared::s_aa4577c560e8(vm, s_i64_2, s_i64_3, host);
                                    vm.exit_call();
                                    let __t40 = __call_res?;
                                    s_i64_2 = __t40;
                                    vm.enter_call()?;
                                    let __call_res = crate::shared_1::Shared::s_d5f50d7e651d(vm, s_i64_2, host);
                                    vm.exit_call();
                                    let __t41 = __call_res?;
                                    s_i32_2 = __t41;
                                    vm.enter_call()?;
                                    let __call_res = crate::shared_1::Shared::s_d7d900550897(vm, s_i32_0, s_i64_1, s_i32_2, host);
                                    vm.exit_call();
                                    __call_res?;
                                    s_i64_0 = 0i64;
                                    l42 = s_i64_0;
                                } else {
                                    s_i32_0 = l31;
                                    s_i64_1 = l43;
                                    s_i64_2 = l44;
                                    s_i64_3 = 4294967296i64;
                                    vm.enter_call()?;
                                    let __call_res = Self::s_1c917789866c(vm, s_i64_2, s_i64_3, host);
                                    vm.exit_call();
                                    let __t42 = __call_res?;
                                    s_i64_2 = __t42;
                                    s_i64_3 = l45;
                                    vm.enter_call()?;
                                    let __call_res = crate::shared_1::Shared::s_aa4577c560e8(vm, s_i64_2, s_i64_3, host);
                                    vm.exit_call();
                                    let __t43 = __call_res?;
                                    s_i64_2 = __t43;
                                    vm.enter_call()?;
                                    let __call_res = crate::shared_1::Shared::s_d5f50d7e651d(vm, s_i64_2, host);
                                    vm.exit_call();
                                    let __t44 = __call_res?;
                                    s_i32_2 = __t44;
                                    vm.enter_call()?;
                                    let __call_res = crate::shared_1::Shared::s_d7d900550897(vm, s_i32_0, s_i64_1, s_i32_2, host);
                                    vm.exit_call();
                                    __call_res?;
                                    s_i64_0 = 1i64;
                                    l42 = s_i64_0;
                                }
                            }
                            s_i64_0 = l43;
                            s_i64_1 = 1i64;
                            vm.enter_call()?;
                            let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
                            vm.exit_call();
                            let __t45 = __call_res?;
                            s_i64_0 = __t45;
                            l43 = s_i64_0;
                            continue 'blk_13;
                            break 'blk_13;
                        }
                    }
                    s_i32_0 = l31;
                    s_i64_1 = l16;
                    s_i32_2 = l31;
                    s_i64_3 = l16;
                    vm.enter_call()?;
                    let __call_res = Self::s_17df4637c35a(vm, s_i32_2, s_i64_3, host);
                    vm.exit_call();
                    let __t46 = __call_res?;
                    s_i32_2 = __t46;
                    s_i64_2 = ops::i64_extend_i32_u(s_i32_2);
                    s_i64_3 = l42;
                    vm.enter_call()?;
                    let __call_res = crate::shared_1::Shared::s_aa4577c560e8(vm, s_i64_2, s_i64_3, host);
                    vm.exit_call();
                    let __t47 = __call_res?;
                    s_i64_2 = __t47;
                    vm.enter_call()?;
                    let __call_res = crate::shared_1::Shared::s_d5f50d7e651d(vm, s_i64_2, host);
                    vm.exit_call();
                    let __t48 = __call_res?;
                    s_i32_2 = __t48;
                    vm.enter_call()?;
                    let __call_res = crate::shared_1::Shared::s_d7d900550897(vm, s_i32_0, s_i64_1, s_i32_2, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l30;
                    s_i64_1 = l35;
                    s_i32_2 = l30;
                    s_i64_3 = l35;
                    vm.enter_call()?;
                    let __call_res = Self::s_17df4637c35a(vm, s_i32_2, s_i64_3, host);
                    vm.exit_call();
                    let __t49 = __call_res?;
                    s_i32_2 = __t49;
                    s_i32_3 = 1i32;
                    s_i32_4 = l36;
                    vm.enter_call()?;
                    let __call_res = crate::shared_1::Shared::s_cc2d1c45c42a(vm, s_i32_3, s_i32_4, host);
                    vm.exit_call();
                    let __t50 = __call_res?;
                    s_i32_3 = __t50;
                    s_i32_2 = ops::i32_or(s_i32_2, s_i32_3);
                    vm.enter_call()?;
                    let __call_res = crate::shared_1::Shared::s_d7d900550897(vm, s_i32_0, s_i64_1, s_i32_2, host);
                    vm.exit_call();
                    __call_res?;
                } else {
                }
            }
            continue 'blk_8;
            break 'blk_8;
        }
    }
    s_i32_0 = l30;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_df3679579582(vm, s_i32_0, host);
    vm.exit_call();
    let __t51 = __call_res?;
    s_i32_0 = __t51;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_b92cf85b6153(vm, s_i32_0, host);
    vm.exit_call();
    let __t52 = __call_res?;
    s_i32_0 = __t52;
    l46 = s_i32_0;
    s_i32_0 = l31;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_df3679579582(vm, s_i32_0, host);
    vm.exit_call();
    let __t53 = __call_res?;
    s_i32_0 = __t53;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_b92cf85b6153(vm, s_i32_0, host);
    vm.exit_call();
    let __t54 = __call_res?;
    s_i32_0 = __t54;
    l47 = s_i32_0;
    s_i32_0 = 8i32;
    vm.enter_call()?;
    let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
    vm.exit_call();
    let __t55 = __call_res?;
    s_i32_0 = __t55;
    l48 = s_i32_0;
    s_i32_0 = l48;
    s_i32_1 = l46;
    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l48;
    s_i32_1 = l47;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l48;
    l49 = s_i32_0;
    s_i32_0 = l49;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_482c16ff60d0(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l1;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_b92cf85b6153(vm, s_i32_0, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l2 = s_i32_0;
    s_i32_0 = l2;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
    s_i64_1 = 0i64;
    s_i32_0 = ops::i64_eq(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = vm.get_i32(g0)?;
        } else {
            s_i32_0 = l0;
            l3 = s_i32_0;
            s_i32_0 = l2;
            l4 = s_i32_0;
            s_i32_0 = 8i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            l5 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = l3;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l5;
            s_i32_1 = l4;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l5;
        }
    }
    l6 = s_i32_0;
    s_i32_0 = l6;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn s_498abb70f200(vm: &mut SubstituteVm, p0: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = 0i32;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i32_2: i32 = 0i32;
    vm.deduct(host, 1)?;
    s_i32_0 = l0;
    s_i32_1 = 0i32;
    s_i32_0 = ops::i32_lt_s(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = 0i32;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    s_i32_1 = 4i32;
    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 0u32)?;
    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
    l1 = s_i32_0;
    s_i32_0 = l1;
    s_i32_1 = 256i32;
    s_i32_0 = ops::i32_lt_u(s_i32_0, s_i32_1);
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = 256i32;
            l1 = s_i32_0;
        }
    }
    s_i32_0 = vm.heap_ptr;
    l2 = s_i32_0;
    s_i32_0 = l2;
    s_i32_1 = l1;
    s_i32_0 = ops::i32_lt_u(s_i32_0, s_i32_1);
    'blk_3: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            l2 = s_i32_0;
        }
    }
    s_i32_0 = l2;
    s_i32_1 = 7i32;
    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
    s_i32_1 = -8i32;
    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
    l2 = s_i32_0;
    s_i32_0 = l0;
    s_i32_1 = 2147418112i32;
    s_i32_2 = l2;
    s_i32_1 = ops::i32_sub(s_i32_1, s_i32_2);
    s_i32_0 = ops::i32_gt_u(s_i32_0, s_i32_1);
    'blk_4: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = l2;
    s_i32_1 = l0;
    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
    s_i32_1 = 7i32;
    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
    s_i32_1 = -8i32;
    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
    l3 = s_i32_0;
    s_i32_0 = vm.memory_size(0)?;
    s_i32_1 = 16i32;
    s_i32_0 = ops::i32_shl(s_i32_0, s_i32_1);
    l4 = s_i32_0;
    s_i32_0 = l3;
    s_i32_1 = l4;
    s_i32_0 = ops::i32_gt_u(s_i32_0, s_i32_1);
    'blk_5: {
        if s_i32_0 != 0 {
            s_i32_0 = l3;
            s_i32_1 = l4;
            s_i32_0 = ops::i32_sub(s_i32_0, s_i32_1);
            s_i32_1 = 65535i32;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            s_i32_1 = 16i32;
            s_i32_0 = ops::i32_shr_u(s_i32_0, s_i32_1);
            s_i32_0 = vm.memory_grow(0, s_i32_0, host)?;
            s_i32_1 = -1i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_6: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
        }
    }
    s_i32_0 = l3;
    vm.heap_ptr = s_i32_0;
    s_i32_0 = l2;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_4adad98baaec(vm: &mut SubstituteVm, p0: i32, g0: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = 0i32;
    let mut l2: i64 = 0i64;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i64 = 0i64;
    let mut l7: i32 = 0i32;
    let mut l8: i32 = 0i32;
    let mut l9: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
    s_i64_1 = 0i64;
    s_i32_0 = ops::i64_eq(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
            l1 = s_i32_0;
            s_i32_0 = l0;
            s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 0u32)?;
            l2 = s_i64_0;
            s_i32_0 = 0i32;
            l3 = s_i32_0;
            s_i32_0 = 20i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t0 = __call_res?;
            s_i32_0 = __t0;
            l4 = s_i32_0;
            s_i32_0 = l4;
            s_i32_1 = l1;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l4;
            s_i64_1 = l2;
            ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
            s_i32_0 = l4;
            s_i32_1 = l3;
            ops::i32_store8(vm, 0, s_i32_0, 16u32, s_i32_1)?;
            s_i32_0 = l4;
        } else {
            s_i32_0 = 0i32;
            l5 = s_i32_0;
            s_i64_0 = 0i64;
            l6 = s_i64_0;
            s_i32_0 = vm.get_i32(g0)?;
            l7 = s_i32_0;
            s_i32_0 = 20i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            l8 = s_i32_0;
            s_i32_0 = l8;
            s_i32_1 = l5;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l8;
            s_i64_1 = l6;
            ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
            s_i32_0 = l8;
            s_i32_1 = l7;
            ops::i32_store8(vm, 0, s_i32_0, 16u32, s_i32_1)?;
            s_i32_0 = l8;
        }
    }
    l9 = s_i32_0;
    s_i32_0 = l9;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_4b9016d16cf3(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l1;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_c31be4b7b29d(vm, s_i32_0, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    s_i32_0 = ops::i32_eqz(s_i32_0);
    l2 = s_i32_0;
    s_i32_0 = l2;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_4e6fb2273c0f(vm: &mut SubstituteVm, p0: i32, p1: i64, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i64 = p1;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    vm.deduct(host, 1)?;
    s_i64_0 = l1;
    s_i32_1 = l0;
    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
    s_i64_1 = ops::i64_extend_i32_u(s_i32_1);
    s_i32_0 = ops::i64_ge_u(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    s_i64_1 = l1;
    s_i32_1 = ops::i32_wrap_i64(s_i64_1);
    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_4f12b3118ae1(vm: &mut SubstituteVm, p0: i32, g0: usize, g1: usize, g2: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = 0i32;
    let mut l2: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_e1d06f25ed5a(vm, s_i32_0, g0, g1, g2, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l1 = s_i32_0;
    s_i32_0 = l1;
    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
    s_i32_0 = ops::i32_eqz(s_i32_0);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        } else {
            s_i32_0 = l1;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
        }
    }
    l2 = s_i32_0;
    s_i32_0 = l2;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_51f0c0b4c723(vm: &mut SubstituteVm, p0: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i64_1: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
    s_i64_1 = 0i64;
    s_i32_0 = ops::i64_lt_s(s_i64_0, s_i64_1);
    l1 = s_i32_0;
    s_i32_0 = l1;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_52dcb6d97135(vm: &mut SubstituteVm, p0: i64, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i64, WasmTrap> {
    let mut l0: i64 = p0;
    let mut l1: i32 = p1;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    vm.deduct(host, 1)?;
    s_i32_0 = l1;
    s_i32_1 = 64i32;
    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i64_0 = l0;
    s_i32_1 = l1;
    s_i64_1 = ops::i64_extend_i32_u(s_i32_1);
    s_i64_0 = ops::i64_shl(s_i64_0, s_i64_1);
    return Ok(s_i64_0);
    }

    pub(crate) fn s_534ed9a12d15(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i64, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut l3: i64 = 0i64;
    let mut l4: i64 = 0i64;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 16u32)?;
    l2 = s_i32_0;
    s_i64_0 = 0i64;
    l3 = s_i64_0;
    'blk_1: {
        'blk_2: loop {
            vm.deduct(host, 1)?;
            s_i64_0 = l3;
            s_i32_1 = l2;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
            s_i64_1 = ops::i64_extend_i32_u(s_i32_1);
            s_i32_0 = ops::i64_lt_u(s_i64_0, s_i64_1);
            s_i32_0 = ops::i32_eqz(s_i32_0);
            if s_i32_0 != 0 {
                break 'blk_1;
            }
            s_i32_0 = l2;
            s_i64_1 = l3;
            vm.enter_call()?;
            let __call_res = Self::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
            vm.exit_call();
            let __t0 = __call_res?;
            s_i32_0 = __t0;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
            s_i32_1 = l1;
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_86635ee877c5(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            'blk_3: {
                if s_i32_0 != 0 {
                    s_i64_0 = l3;
                    l4 = s_i64_0;
                    s_i64_0 = l4;
                    return Ok(s_i64_0);
                } else {
                }
            }
            s_i64_0 = l3;
            s_i64_1 = 1i64;
            vm.enter_call()?;
            let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i64_0 = __t2;
            l3 = s_i64_0;
            continue 'blk_2;
            break 'blk_2;
        }
    }
    return Err(WasmTrap::Unreachable);
    }

    pub(crate) fn s_559745ed6085(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
    s_i32_1 = l1;
    s_i64_1 = ops::i64_load(vm, 0, s_i32_1, 8u32)?;
    s_i32_0 = ops::i64_ne(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
            s_i32_1 = l1;
            s_i64_1 = ops::i64_load(vm, 0, s_i32_1, 8u32)?;
            s_i32_0 = ops::i64_lt_u(s_i64_0, s_i64_1);
        } else {
            s_i32_0 = l0;
            s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 0u32)?;
            s_i32_1 = l1;
            s_i64_1 = ops::i64_load(vm, 0, s_i32_1, 0u32)?;
            s_i32_0 = ops::i64_lt_u(s_i64_0, s_i64_1);
        }
    }
    l2 = s_i32_0;
    s_i32_0 = l2;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_595014f04f46(vm: &mut SubstituteVm, p0: i64, p1: i64, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i64 = p0;
    let mut l1: i64 = p1;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    vm.deduct(host, 1)?;
    s_i64_0 = l1;
    s_i64_1 = 268427232i64;
    s_i32_0 = ops::i64_gt_u(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i64_0 = l1;
    s_i32_0 = ops::i32_wrap_i64(s_i64_0);
    l2 = s_i32_0;
    s_i32_0 = l2;
    s_i32_1 = 3i32;
    s_i32_0 = ops::i32_shl(s_i32_0, s_i32_1);
    l3 = s_i32_0;
    s_i32_0 = 16i32;
    s_i32_1 = l3;
    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
    vm.enter_call()?;
    let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l4 = s_i32_0;
    s_i32_0 = l4;
    s_i32_1 = 16i32;
    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
    l5 = s_i32_0;
    s_i32_0 = 0i32;
    l6 = s_i32_0;
    'blk_2: {
        'blk_3: loop {
            vm.deduct(host, 1)?;
            s_i32_0 = l6;
            s_i32_1 = l3;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            if s_i32_0 != 0 {
                break 'blk_2;
            }
            s_i32_0 = l5;
            s_i32_1 = l6;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            s_i64_1 = l0;
            ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
            s_i32_0 = l6;
            s_i32_1 = 8i32;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l6 = s_i32_0;
            continue 'blk_3;
            break 'blk_3;
        }
    }
    s_i32_0 = l4;
    s_i32_1 = l5;
    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l4;
    s_i32_1 = l2;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l4;
    s_i32_1 = l2;
    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
    s_i32_0 = l4;
    s_i32_1 = 0i32;
    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
    s_i32_0 = l4;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_59d2e4c57622(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, g1: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_1 = l1;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_e4bf017d6693(vm, s_i32_0, s_i32_1, g0, g1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l2 = s_i32_0;
    s_i32_0 = l2;
    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
    s_i32_0 = ops::i32_eqz(s_i32_0);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        } else {
            s_i32_0 = l2;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
        }
    }
    l3 = s_i32_0;
    s_i32_0 = l3;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn s_5af5ebb1dc24(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i64 = 0i64;
    let mut l3: i64 = 0i64;
    let mut l4: i64 = 0i64;
    let mut l5: i32 = 0i32;
    let mut l6: i64 = 0i64;
    let mut l7: i64 = 0i64;
    let mut l8: i64 = 0i64;
    let mut l9: i64 = 0i64;
    let mut l10: i64 = 0i64;
    let mut l11: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i64_2: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
    l2 = s_i64_0;
    s_i32_0 = l1;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
    l3 = s_i64_0;
    s_i64_0 = l2;
    s_i64_1 = l3;
    s_i32_0 = ops::i64_gt_u(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i64_0 = l2;
        } else {
            s_i64_0 = l3;
        }
    }
    l4 = s_i64_0;
    s_i64_0 = l4;
    s_i64_1 = 1i64;
    vm.enter_call()?;
    let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i64_0 = __t0;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_c5da14f612f0(vm, s_i64_0, host);
    vm.exit_call();
    let __t1 = __call_res?;
    s_i32_0 = __t1;
    l5 = s_i32_0;
    s_i64_0 = 0i64;
    l6 = s_i64_0;
    s_i64_0 = 0i64;
    l7 = s_i64_0;
    'blk_2: {
        'blk_3: loop {
            vm.deduct(host, 1)?;
            s_i64_0 = l7;
            s_i64_1 = l4;
            s_i32_0 = ops::i64_lt_u(s_i64_0, s_i64_1);
            s_i32_0 = ops::i32_eqz(s_i32_0);
            if s_i32_0 != 0 {
                break 'blk_2;
            }
            s_i64_0 = l7;
            s_i64_1 = l2;
            s_i32_0 = ops::i64_lt_u(s_i64_0, s_i64_1);
            'blk_4: {
                if s_i32_0 != 0 {
                    s_i32_0 = l0;
                    s_i64_1 = l7;
                    vm.enter_call()?;
                    let __call_res = Self::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
                    vm.exit_call();
                    let __t2 = __call_res?;
                    s_i32_0 = __t2;
                    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
                } else {
                    s_i64_0 = 0i64;
                }
            }
            l8 = s_i64_0;
            s_i64_0 = l7;
            s_i64_1 = l3;
            s_i32_0 = ops::i64_lt_u(s_i64_0, s_i64_1);
            'blk_5: {
                if s_i32_0 != 0 {
                    s_i32_0 = l1;
                    s_i64_1 = l7;
                    vm.enter_call()?;
                    let __call_res = Self::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
                    vm.exit_call();
                    let __t3 = __call_res?;
                    s_i32_0 = __t3;
                    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
                } else {
                    s_i64_0 = 0i64;
                }
            }
            l9 = s_i64_0;
            s_i64_0 = l8;
            s_i64_1 = l9;
            vm.enter_call()?;
            let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
            vm.exit_call();
            let __t4 = __call_res?;
            s_i64_0 = __t4;
            s_i64_1 = l6;
            vm.enter_call()?;
            let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
            vm.exit_call();
            let __t5 = __call_res?;
            s_i64_0 = __t5;
            l10 = s_i64_0;
            s_i32_0 = l5;
            s_i64_1 = l10;
            s_i64_2 = 4294967295i64;
            s_i64_1 = ops::i64_and(s_i64_1, s_i64_2);
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_d5f50d7e651d(vm, s_i64_1, host);
            vm.exit_call();
            let __t6 = __call_res?;
            s_i32_1 = __t6;
            vm.enter_call()?;
            let __call_res = Self::s_697cb42b9ce7(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i64_0 = l10;
            s_i32_1 = 32i32;
            vm.enter_call()?;
            let __call_res = Self::s_73a866edda27(vm, s_i64_0, s_i32_1, host);
            vm.exit_call();
            let __t7 = __call_res?;
            s_i64_0 = __t7;
            l6 = s_i64_0;
            s_i64_0 = l7;
            s_i64_1 = 1i64;
            vm.enter_call()?;
            let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
            vm.exit_call();
            let __t8 = __call_res?;
            s_i64_0 = __t8;
            l7 = s_i64_0;
            continue 'blk_3;
            break 'blk_3;
        }
    }
    s_i64_0 = l6;
    s_i64_1 = 0i64;
    s_i32_0 = ops::i64_ne(s_i64_0, s_i64_1);
    'blk_6: {
        if s_i32_0 != 0 {
            s_i32_0 = l5;
            s_i64_1 = l6;
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_d5f50d7e651d(vm, s_i64_1, host);
            vm.exit_call();
            let __t9 = __call_res?;
            s_i32_1 = __t9;
            vm.enter_call()?;
            let __call_res = Self::s_697cb42b9ce7(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
        } else {
        }
    }
    s_i32_0 = l5;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_df3679579582(vm, s_i32_0, host);
    vm.exit_call();
    let __t10 = __call_res?;
    s_i32_0 = __t10;
    l11 = s_i32_0;
    s_i32_0 = l11;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_5b883f251b47(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
    l2 = s_i32_0;
    s_i32_0 = l1;
    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
    l3 = s_i32_0;
    s_i32_0 = l2;
    s_i32_1 = l3;
    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = l2;
            'blk_2: {
                if s_i32_0 != 0 {
                    s_i32_0 = -1i32;
                } else {
                    s_i32_0 = 1i32;
                }
            }
        } else {
            s_i32_0 = l0;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_86b08f7a2a43(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t0 = __call_res?;
            s_i32_0 = __t0;
            l4 = s_i32_0;
            s_i32_0 = l2;
            'blk_3: {
                if s_i32_0 != 0 {
                    s_i32_0 = l4;
                    vm.enter_call()?;
                    let __call_res = Self::s_062991301687(vm, s_i32_0, host);
                    vm.exit_call();
                    let __t1 = __call_res?;
                    s_i32_0 = __t1;
                } else {
                    s_i32_0 = l4;
                }
            }
        }
    }
    l5 = s_i32_0;
    s_i32_0 = l5;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn s_5d431d0f77d6(vm: &mut SubstituteVm, p0: i32, p1: i64, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i64 = p1;
    let mut l2: i64 = 0i64;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut l5: i64 = 0i64;
    let mut l6: i32 = 0i32;
    let mut l7: i32 = 0i32;
    let mut l8: i32 = 0i32;
    let mut l9: i32 = 0i32;
    let mut l10: i32 = 0i32;
    let mut l11: i64 = 0i64;
    let mut l12: i32 = 0i32;
    let mut l13: i32 = 0i32;
    let mut l14: i32 = 0i32;
    let mut l15: i32 = 0i32;
    let mut l16: i64 = 0i64;
    let mut l17: i32 = 0i32;
    let mut l18: i32 = 0i32;
    let mut l19: i32 = 0i32;
    let mut l20: i32 = 0i32;
    let mut l21: i64 = 0i64;
    let mut l22: i32 = 0i32;
    let mut l23: i32 = 0i32;
    let mut l24: i32 = 0i32;
    let mut l25: i32 = 0i32;
    let mut l26: i32 = 0i32;
    let mut l27: i64 = 0i64;
    let mut l28: i32 = 0i32;
    let mut l29: i32 = 0i32;
    let mut l30: i32 = 0i32;
    let mut l31: i64 = 0i64;
    let mut l32: i32 = 0i32;
    let mut l33: i32 = 0i32;
    let mut l34: i64 = 0i64;
    let mut l35: i32 = 0i32;
    let mut l36: i32 = 0i32;
    let mut l37: i32 = 0i32;
    let mut l38: i64 = 0i64;
    let mut l39: i32 = 0i32;
    let mut l40: i32 = 0i32;
    let mut l41: i32 = 0i32;
    let mut l42: i64 = 0i64;
    let mut l43: i32 = 0i32;
    let mut l44: i32 = 0i32;
    let mut l45: i32 = 0i32;
    let mut l46: i32 = 0i32;
    let mut l47: i32 = 0i32;
    let mut l48: i32 = 0i32;
    let mut l49: i64 = 0i64;
    let mut l50: i32 = 0i32;
    let mut l51: i32 = 0i32;
    let mut l52: i32 = 0i32;
    let mut l53: i32 = 0i32;
    let mut l54: i64 = 0i64;
    let mut l55: i32 = 0i32;
    let mut l56: i32 = 0i32;
    let mut l57: i64 = 0i64;
    let mut l58: i64 = 0i64;
    let mut l59: i32 = 0i32;
    let mut l60: i32 = 0i32;
    let mut l61: i32 = 0i32;
    let mut l62: i64 = 0i64;
    let mut l63: i32 = 0i32;
    let mut l64: i32 = 0i32;
    let mut l65: i64 = 0i64;
    let mut l66: i64 = 0i64;
    let mut l67: i32 = 0i32;
    let mut l68: i32 = 0i32;
    let mut l69: i32 = 0i32;
    let mut l70: i64 = 0i64;
    let mut l71: i32 = 0i32;
    let mut l72: i32 = 0i32;
    let mut l73: i32 = 0i32;
    let mut l74: i32 = 0i32;
    let mut l75: i64 = 0i64;
    let mut l76: i32 = 0i32;
    let mut l77: i32 = 0i32;
    let mut l78: i32 = 0i32;
    let mut l79: i32 = 0i32;
    let mut l80: i64 = 0i64;
    let mut l81: i32 = 0i32;
    let mut l82: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i32_2: i32 = 0i32;
    let mut s_i64_2: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
    l2 = s_i64_0;
    s_i64_0 = l1;
    s_i64_1 = l2;
    s_i32_0 = ops::i64_ge_u(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            l3 = s_i32_0;
            s_i64_0 = 0i64;
            s_i64_1 = 0i64;
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_b3e6ca84b2de(vm, s_i64_0, s_i64_1, host);
            vm.exit_call();
            let __t0 = __call_res?;
            s_i32_0 = __t0;
            l4 = s_i32_0;
            s_i64_0 = l1;
            l5 = s_i64_0;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            l6 = s_i32_0;
            s_i32_0 = l6;
            s_i32_1 = l3;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l6;
            s_i32_1 = l4;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l6;
            s_i64_1 = l5;
            ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
            s_i32_0 = l6;
            l7 = s_i32_0;
            s_i32_0 = l7;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i32_0 = l0;
    s_i64_1 = l1;
    vm.enter_call()?;
    let __call_res = Self::s_4e6fb2273c0f(vm, s_i32_0, s_i64_1, host);
    vm.exit_call();
    let __t2 = __call_res?;
    s_i32_0 = __t2;
    l8 = s_i32_0;
    s_i32_0 = l8;
    s_i32_1 = 128i32;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_8b082d5ac52c(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t3 = __call_res?;
    s_i32_0 = __t3;
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
            l9 = s_i32_0;
            s_i32_0 = l8;
            s_i32_1 = 255i32;
            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
            s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
            s_i64_1 = 0i64;
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_b3e6ca84b2de(vm, s_i64_0, s_i64_1, host);
            vm.exit_call();
            let __t4 = __call_res?;
            s_i32_0 = __t4;
            l10 = s_i32_0;
            s_i64_0 = l1;
            s_i64_1 = 1i64;
            vm.enter_call()?;
            let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
            vm.exit_call();
            let __t5 = __call_res?;
            s_i64_0 = __t5;
            l11 = s_i64_0;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t6 = __call_res?;
            s_i32_0 = __t6;
            l12 = s_i32_0;
            s_i32_0 = l12;
            s_i32_1 = l9;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l12;
            s_i32_1 = l10;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l12;
            s_i64_1 = l11;
            ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
            s_i32_0 = l12;
            l13 = s_i32_0;
            s_i32_0 = l13;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i32_0 = l8;
    s_i32_1 = 194i32;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_8b082d5ac52c(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t7 = __call_res?;
    s_i32_0 = __t7;
    'blk_3: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            l14 = s_i32_0;
            s_i64_0 = 0i64;
            s_i64_1 = 0i64;
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_b3e6ca84b2de(vm, s_i64_0, s_i64_1, host);
            vm.exit_call();
            let __t8 = __call_res?;
            s_i32_0 = __t8;
            l15 = s_i32_0;
            s_i64_0 = l1;
            l16 = s_i64_0;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t9 = __call_res?;
            s_i32_0 = __t9;
            l17 = s_i32_0;
            s_i32_0 = l17;
            s_i32_1 = l14;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l17;
            s_i32_1 = l15;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l17;
            s_i64_1 = l16;
            ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
            s_i32_0 = l17;
            l18 = s_i32_0;
            s_i32_0 = l18;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i32_0 = l8;
    s_i32_1 = 255i32;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_b1342c1ced52(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t10 = __call_res?;
    s_i32_0 = __t10;
    'blk_4: {
        if s_i32_0 != 0 {
            s_i64_0 = l1;
            s_i64_1 = 1i64;
            vm.enter_call()?;
            let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
            vm.exit_call();
            let __t11 = __call_res?;
            s_i64_0 = __t11;
            s_i64_1 = l2;
            s_i32_0 = ops::i64_ge_u(s_i64_0, s_i64_1);
            'blk_5: {
                if s_i32_0 != 0 {
                    s_i32_0 = 0i32;
                    l19 = s_i32_0;
                    s_i64_0 = 0i64;
                    s_i64_1 = 0i64;
                    vm.enter_call()?;
                    let __call_res = crate::shared_1::Shared::s_b3e6ca84b2de(vm, s_i64_0, s_i64_1, host);
                    vm.exit_call();
                    let __t12 = __call_res?;
                    s_i32_0 = __t12;
                    l20 = s_i32_0;
                    s_i64_0 = l1;
                    l21 = s_i64_0;
                    s_i32_0 = 16i32;
                    vm.enter_call()?;
                    let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
                    vm.exit_call();
                    let __t13 = __call_res?;
                    s_i32_0 = __t13;
                    l22 = s_i32_0;
                    s_i32_0 = l22;
                    s_i32_1 = l19;
                    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l22;
                    s_i32_1 = l20;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l22;
                    s_i64_1 = l21;
                    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
                    s_i32_0 = l22;
                    l23 = s_i32_0;
                    s_i32_0 = l23;
                    return Ok(s_i32_0);
                } else {
                }
            }
            s_i32_0 = l0;
            s_i64_1 = l1;
            s_i64_2 = 1i64;
            vm.enter_call()?;
            let __call_res = Self::s_1c917789866c(vm, s_i64_1, s_i64_2, host);
            vm.exit_call();
            let __t14 = __call_res?;
            s_i64_1 = __t14;
            vm.enter_call()?;
            let __call_res = Self::s_4e6fb2273c0f(vm, s_i32_0, s_i64_1, host);
            vm.exit_call();
            let __t15 = __call_res?;
            s_i32_0 = __t15;
            l24 = s_i32_0;
            s_i32_0 = l24;
            s_i32_1 = 192i32;
            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
            s_i32_1 = 255i32;
            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
            s_i32_1 = 128i32;
            vm.enter_call()?;
            let __call_res = Self::s_3e7647da0841(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t16 = __call_res?;
            s_i32_0 = __t16;
            'blk_6: {
                if s_i32_0 != 0 {
                    s_i32_0 = 0i32;
                    l25 = s_i32_0;
                    s_i64_0 = 0i64;
                    s_i64_1 = 0i64;
                    vm.enter_call()?;
                    let __call_res = crate::shared_1::Shared::s_b3e6ca84b2de(vm, s_i64_0, s_i64_1, host);
                    vm.exit_call();
                    let __t17 = __call_res?;
                    s_i32_0 = __t17;
                    l26 = s_i32_0;
                    s_i64_0 = l1;
                    l27 = s_i64_0;
                    s_i32_0 = 16i32;
                    vm.enter_call()?;
                    let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
                    vm.exit_call();
                    let __t18 = __call_res?;
                    s_i32_0 = __t18;
                    l28 = s_i32_0;
                    s_i32_0 = l28;
                    s_i32_1 = l25;
                    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l28;
                    s_i32_1 = l26;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l28;
                    s_i64_1 = l27;
                    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
                    s_i32_0 = l28;
                    l29 = s_i32_0;
                    s_i32_0 = l29;
                    return Ok(s_i32_0);
                } else {
                }
            }
            s_i32_0 = l24;
            s_i32_1 = 2i32;
            vm.enter_call()?;
            let __call_res = Self::s_708c3053a6ba(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t19 = __call_res?;
            s_i32_0 = __t19;
            s_i32_1 = 15i32;
            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
            s_i32_1 = 255i32;
            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
            s_i32_1 = 255i32;
            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
            l30 = s_i32_0;
            s_i64_0 = 8i64;
            s_i32_1 = l30;
            s_i64_1 = ops::i64_extend_i32_u(s_i32_1);
            vm.enter_call()?;
            let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
            vm.exit_call();
            let __t20 = __call_res?;
            s_i64_0 = __t20;
            l31 = s_i64_0;
            s_i64_0 = l1;
            s_i64_1 = l31;
            vm.enter_call()?;
            let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
            vm.exit_call();
            let __t21 = __call_res?;
            s_i64_0 = __t21;
            s_i64_1 = l2;
            s_i32_0 = ops::i64_gt_u(s_i64_0, s_i64_1);
            'blk_7: {
                if s_i32_0 != 0 {
                    s_i32_0 = 0i32;
                    l32 = s_i32_0;
                    s_i64_0 = 0i64;
                    s_i64_1 = 0i64;
                    vm.enter_call()?;
                    let __call_res = crate::shared_1::Shared::s_b3e6ca84b2de(vm, s_i64_0, s_i64_1, host);
                    vm.exit_call();
                    let __t22 = __call_res?;
                    s_i32_0 = __t22;
                    l33 = s_i32_0;
                    s_i64_0 = l1;
                    l34 = s_i64_0;
                    s_i32_0 = 16i32;
                    vm.enter_call()?;
                    let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
                    vm.exit_call();
                    let __t23 = __call_res?;
                    s_i32_0 = __t23;
                    l35 = s_i32_0;
                    s_i32_0 = l35;
                    s_i32_1 = l32;
                    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l35;
                    s_i32_1 = l33;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l35;
                    s_i64_1 = l34;
                    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
                    s_i32_0 = l35;
                    l36 = s_i32_0;
                    s_i32_0 = l36;
                    return Ok(s_i32_0);
                } else {
                }
            }
            s_i32_0 = l24;
            s_i32_1 = 3i32;
            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
            s_i32_1 = 255i32;
            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
            s_i32_1 = 255i32;
            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
            s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
            vm.enter_call()?;
            let __call_res = Self::s_41e9bacc9fe3(vm, s_i64_0, host);
            vm.exit_call();
            let __t24 = __call_res?;
            s_i32_0 = __t24;
            l37 = s_i32_0;
            s_i64_0 = 2i64;
            l38 = s_i64_0;
            'blk_8: {
                'blk_9: loop {
                    vm.deduct(host, 1)?;
                    s_i64_0 = l38;
                    s_i64_1 = l31;
                    s_i32_0 = ops::i64_lt_u(s_i64_0, s_i64_1);
                    s_i32_0 = ops::i32_eqz(s_i32_0);
                    if s_i32_0 != 0 {
                        break 'blk_8;
                    }
                    s_i32_0 = l0;
                    s_i64_1 = l1;
                    s_i64_2 = l38;
                    vm.enter_call()?;
                    let __call_res = Self::s_1c917789866c(vm, s_i64_1, s_i64_2, host);
                    vm.exit_call();
                    let __t25 = __call_res?;
                    s_i64_1 = __t25;
                    vm.enter_call()?;
                    let __call_res = Self::s_4e6fb2273c0f(vm, s_i32_0, s_i64_1, host);
                    vm.exit_call();
                    let __t26 = __call_res?;
                    s_i32_0 = __t26;
                    l39 = s_i32_0;
                    s_i32_0 = l39;
                    s_i32_1 = 192i32;
                    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                    s_i32_1 = 255i32;
                    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                    s_i32_1 = 128i32;
                    vm.enter_call()?;
                    let __call_res = Self::s_3e7647da0841(vm, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t27 = __call_res?;
                    s_i32_0 = __t27;
                    'blk_10: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 0i32;
                            l40 = s_i32_0;
                            s_i64_0 = 0i64;
                            s_i64_1 = 0i64;
                            vm.enter_call()?;
                            let __call_res = crate::shared_1::Shared::s_b3e6ca84b2de(vm, s_i64_0, s_i64_1, host);
                            vm.exit_call();
                            let __t28 = __call_res?;
                            s_i32_0 = __t28;
                            l41 = s_i32_0;
                            s_i64_0 = l1;
                            l42 = s_i64_0;
                            s_i32_0 = 16i32;
                            vm.enter_call()?;
                            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
                            vm.exit_call();
                            let __t29 = __call_res?;
                            s_i32_0 = __t29;
                            l43 = s_i32_0;
                            s_i32_0 = l43;
                            s_i32_1 = l40;
                            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                            s_i32_0 = l43;
                            s_i32_1 = l41;
                            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                            s_i32_0 = l43;
                            s_i64_1 = l42;
                            ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
                            s_i32_0 = l43;
                            l44 = s_i32_0;
                            s_i32_0 = l44;
                            return Ok(s_i32_0);
                        } else {
                        }
                    }
                    s_i32_0 = l37;
                    s_i32_1 = 6i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_1::Shared::s_fa6af9f06252(vm, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t30 = __call_res?;
                    s_i32_0 = __t30;
                    s_i32_1 = l39;
                    s_i32_2 = 63i32;
                    s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
                    s_i32_2 = 255i32;
                    s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
                    s_i32_2 = 255i32;
                    s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
                    s_i64_1 = ops::i64_extend_i32_u(s_i32_1);
                    vm.enter_call()?;
                    let __call_res = Self::s_41e9bacc9fe3(vm, s_i64_1, host);
                    vm.exit_call();
                    let __t31 = __call_res?;
                    s_i32_1 = __t31;
                    vm.enter_call()?;
                    let __call_res = crate::shared_1::Shared::s_9c006b2946a7(vm, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t32 = __call_res?;
                    s_i32_0 = __t32;
                    l37 = s_i32_0;
                    s_i64_0 = l38;
                    s_i64_1 = 1i64;
                    vm.enter_call()?;
                    let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
                    vm.exit_call();
                    let __t33 = __call_res?;
                    s_i64_0 = __t33;
                    l38 = s_i64_0;
                    continue 'blk_9;
                    break 'blk_9;
                }
            }
            s_i32_0 = l30;
            s_i32_1 = 0i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_11: {
                if s_i32_0 != 0 {
                    s_i32_0 = 36i32;
                } else {
                    s_i32_0 = 32i32;
                    s_i32_1 = 6i32;
                    s_i32_2 = l30;
                    vm.enter_call()?;
                    let __call_res = Self::s_19ca9f9d3f2f(vm, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    let __t34 = __call_res?;
                    s_i32_1 = __t34;
                    vm.enter_call()?;
                    let __call_res = crate::shared_1::Shared::s_91b7532ed2b6(vm, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t35 = __call_res?;
                    s_i32_0 = __t35;
                }
            }
            l45 = s_i32_0;
            s_i64_0 = 1i64;
            vm.enter_call()?;
            let __call_res = Self::s_41e9bacc9fe3(vm, s_i64_0, host);
            vm.exit_call();
            let __t36 = __call_res?;
            s_i32_0 = __t36;
            s_i32_1 = l45;
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_fa6af9f06252(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t37 = __call_res?;
            s_i32_0 = __t37;
            l46 = s_i32_0;
            s_i32_0 = l37;
            s_i32_1 = l46;
            vm.enter_call()?;
            let __call_res = Self::s_559745ed6085(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t38 = __call_res?;
            s_i32_0 = __t38;
            'blk_12: {
                if s_i32_0 != 0 {
                    s_i32_0 = 0i32;
                    l47 = s_i32_0;
                    s_i64_0 = 0i64;
                    s_i64_1 = 0i64;
                    vm.enter_call()?;
                    let __call_res = crate::shared_1::Shared::s_b3e6ca84b2de(vm, s_i64_0, s_i64_1, host);
                    vm.exit_call();
                    let __t39 = __call_res?;
                    s_i32_0 = __t39;
                    l48 = s_i32_0;
                    s_i64_0 = l1;
                    l49 = s_i64_0;
                    s_i32_0 = 16i32;
                    vm.deduct(host, 1)?;
                    vm.enter_call()?;
                    let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
                    vm.exit_call();
                    let __t40 = __call_res?;
                    s_i32_0 = __t40;
                    l50 = s_i32_0;
                    s_i32_0 = l50;
                    s_i32_1 = l47;
                    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l50;
                    s_i32_1 = l48;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l50;
                    s_i64_1 = l49;
                    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
                    s_i32_0 = l50;
                    l51 = s_i32_0;
                    s_i32_0 = l51;
                    return Ok(s_i32_0);
                } else {
                }
            }
            s_i32_0 = 1i32;
            l52 = s_i32_0;
            s_i32_0 = l37;
            vm.enter_call()?;
            let __call_res = Self::s_623491ac0af2(vm, s_i32_0, host);
            vm.exit_call();
            let __t41 = __call_res?;
            s_i32_0 = __t41;
            l53 = s_i32_0;
            s_i64_0 = l1;
            s_i64_1 = l31;
            vm.enter_call()?;
            let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
            vm.exit_call();
            let __t42 = __call_res?;
            s_i64_0 = __t42;
            l54 = s_i64_0;
            vm.deduct(host, 1)?;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t43 = __call_res?;
            s_i32_0 = __t43;
            l55 = s_i32_0;
            s_i32_0 = l55;
            s_i32_1 = l52;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l55;
            s_i32_1 = l53;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l55;
            s_i64_1 = l54;
            ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
            s_i32_0 = l55;
            l56 = s_i32_0;
            s_i32_0 = l56;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i64_0 = 7i64;
    l57 = s_i64_0;
    s_i64_0 = 2147483648i64;
    l58 = s_i64_0;
    s_i32_0 = 0i32;
    l59 = s_i32_0;
    s_i32_0 = l8;
    s_i32_1 = 224i32;
    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
    s_i32_1 = 255i32;
    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
    s_i32_1 = 192i32;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_b1342c1ced52(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t44 = __call_res?;
    s_i32_0 = __t44;
    'blk_13: {
        if s_i32_0 != 0 {
            s_i64_0 = 2i64;
            l57 = s_i64_0;
            s_i64_0 = 128i64;
            l58 = s_i64_0;
            s_i32_0 = 31i32;
            l59 = s_i32_0;
        } else {
            s_i32_0 = l8;
            s_i32_1 = 240i32;
            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
            s_i32_1 = 255i32;
            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
            s_i32_1 = 224i32;
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_b1342c1ced52(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t45 = __call_res?;
            s_i32_0 = __t45;
            'blk_14: {
                if s_i32_0 != 0 {
                    s_i64_0 = 3i64;
                    l57 = s_i64_0;
                    s_i64_0 = 2048i64;
                    l58 = s_i64_0;
                    s_i32_0 = 15i32;
                    l59 = s_i32_0;
                } else {
                    s_i32_0 = l8;
                    s_i32_1 = 248i32;
                    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                    s_i32_1 = 255i32;
                    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                    s_i32_1 = 240i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_1::Shared::s_b1342c1ced52(vm, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t46 = __call_res?;
                    s_i32_0 = __t46;
                    'blk_15: {
                        if s_i32_0 != 0 {
                            s_i64_0 = 4i64;
                            l57 = s_i64_0;
                            s_i64_0 = 65536i64;
                            l58 = s_i64_0;
                            s_i32_0 = 7i32;
                            l59 = s_i32_0;
                        } else {
                            s_i32_0 = l8;
                            s_i32_1 = 252i32;
                            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                            s_i32_1 = 255i32;
                            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                            s_i32_1 = 248i32;
                            vm.enter_call()?;
                            let __call_res = crate::shared_1::Shared::s_b1342c1ced52(vm, s_i32_0, s_i32_1, host);
                            vm.exit_call();
                            let __t47 = __call_res?;
                            s_i32_0 = __t47;
                            'blk_16: {
                                if s_i32_0 != 0 {
                                    s_i64_0 = 5i64;
                                    l57 = s_i64_0;
                                    s_i64_0 = 2097152i64;
                                    l58 = s_i64_0;
                                    s_i32_0 = 3i32;
                                    l59 = s_i32_0;
                                } else {
                                    s_i32_0 = l8;
                                    s_i32_1 = 254i32;
                                    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                                    s_i32_1 = 255i32;
                                    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                                    s_i32_1 = 252i32;
                                    vm.enter_call()?;
                                    let __call_res = crate::shared_1::Shared::s_b1342c1ced52(vm, s_i32_0, s_i32_1, host);
                                    vm.exit_call();
                                    let __t48 = __call_res?;
                                    s_i32_0 = __t48;
                                    'blk_17: {
                                        if s_i32_0 != 0 {
                                            s_i64_0 = 6i64;
                                            l57 = s_i64_0;
                                            s_i64_0 = 67108864i64;
                                            l58 = s_i64_0;
                                            s_i32_0 = 1i32;
                                            l59 = s_i32_0;
                                        } else {
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
    s_i64_0 = l1;
    s_i64_1 = l57;
    vm.enter_call()?;
    let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
    vm.exit_call();
    let __t49 = __call_res?;
    s_i64_0 = __t49;
    s_i64_1 = l2;
    s_i32_0 = ops::i64_gt_u(s_i64_0, s_i64_1);
    'blk_18: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            l60 = s_i32_0;
            s_i64_0 = 0i64;
            s_i64_1 = 0i64;
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_b3e6ca84b2de(vm, s_i64_0, s_i64_1, host);
            vm.exit_call();
            let __t50 = __call_res?;
            s_i32_0 = __t50;
            l61 = s_i32_0;
            s_i64_0 = l1;
            l62 = s_i64_0;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t51 = __call_res?;
            s_i32_0 = __t51;
            l63 = s_i32_0;
            s_i32_0 = l63;
            s_i32_1 = l60;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l63;
            s_i32_1 = l61;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l63;
            s_i64_1 = l62;
            ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
            s_i32_0 = l63;
            l64 = s_i32_0;
            s_i32_0 = l64;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i32_0 = l8;
    s_i32_1 = l59;
    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
    s_i32_1 = 255i32;
    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
    s_i32_1 = 255i32;
    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
    l65 = s_i64_0;
    s_i64_0 = 1i64;
    l66 = s_i64_0;
    'blk_19: {
        'blk_20: loop {
            vm.deduct(host, 1)?;
            s_i64_0 = l66;
            s_i64_1 = l57;
            s_i32_0 = ops::i64_lt_u(s_i64_0, s_i64_1);
            s_i32_0 = ops::i32_eqz(s_i32_0);
            if s_i32_0 != 0 {
                break 'blk_19;
            }
            s_i32_0 = l0;
            s_i64_1 = l1;
            s_i64_2 = l66;
            vm.enter_call()?;
            let __call_res = Self::s_1c917789866c(vm, s_i64_1, s_i64_2, host);
            vm.exit_call();
            let __t52 = __call_res?;
            s_i64_1 = __t52;
            vm.enter_call()?;
            let __call_res = Self::s_4e6fb2273c0f(vm, s_i32_0, s_i64_1, host);
            vm.exit_call();
            let __t53 = __call_res?;
            s_i32_0 = __t53;
            l67 = s_i32_0;
            s_i32_0 = l67;
            s_i32_1 = 192i32;
            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
            s_i32_1 = 255i32;
            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
            s_i32_1 = 128i32;
            vm.enter_call()?;
            let __call_res = Self::s_3e7647da0841(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t54 = __call_res?;
            s_i32_0 = __t54;
            'blk_21: {
                if s_i32_0 != 0 {
                    s_i32_0 = 0i32;
                    l68 = s_i32_0;
                    s_i64_0 = 0i64;
                    s_i64_1 = 0i64;
                    vm.enter_call()?;
                    let __call_res = crate::shared_1::Shared::s_b3e6ca84b2de(vm, s_i64_0, s_i64_1, host);
                    vm.exit_call();
                    let __t55 = __call_res?;
                    s_i32_0 = __t55;
                    l69 = s_i32_0;
                    s_i64_0 = l1;
                    l70 = s_i64_0;
                    s_i32_0 = 16i32;
                    vm.enter_call()?;
                    let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
                    vm.exit_call();
                    let __t56 = __call_res?;
                    s_i32_0 = __t56;
                    l71 = s_i32_0;
                    s_i32_0 = l71;
                    s_i32_1 = l68;
                    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l71;
                    s_i32_1 = l69;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l71;
                    s_i64_1 = l70;
                    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
                    s_i32_0 = l71;
                    l72 = s_i32_0;
                    s_i32_0 = l72;
                    return Ok(s_i32_0);
                } else {
                }
            }
            s_i64_0 = l65;
            s_i32_1 = 6i32;
            vm.enter_call()?;
            let __call_res = Self::s_52dcb6d97135(vm, s_i64_0, s_i32_1, host);
            vm.exit_call();
            let __t57 = __call_res?;
            s_i64_0 = __t57;
            s_i32_1 = l67;
            s_i32_2 = 63i32;
            s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
            s_i32_2 = 255i32;
            s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
            s_i32_2 = 255i32;
            s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
            s_i64_1 = ops::i64_extend_i32_u(s_i32_1);
            s_i64_0 = ops::i64_or(s_i64_0, s_i64_1);
            l65 = s_i64_0;
            s_i64_0 = l66;
            s_i64_1 = 1i64;
            vm.enter_call()?;
            let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
            vm.exit_call();
            let __t58 = __call_res?;
            s_i64_0 = __t58;
            l66 = s_i64_0;
            continue 'blk_20;
            break 'blk_20;
        }
    }
    s_i64_0 = l65;
    s_i64_1 = l58;
    s_i32_0 = ops::i64_lt_u(s_i64_0, s_i64_1);
    'blk_22: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            l73 = s_i32_0;
            s_i64_0 = 0i64;
            s_i64_1 = 0i64;
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_b3e6ca84b2de(vm, s_i64_0, s_i64_1, host);
            vm.exit_call();
            let __t59 = __call_res?;
            s_i32_0 = __t59;
            l74 = s_i32_0;
            s_i64_0 = l1;
            l75 = s_i64_0;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t60 = __call_res?;
            s_i32_0 = __t60;
            l76 = s_i32_0;
            s_i32_0 = l76;
            s_i32_1 = l73;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l76;
            s_i32_1 = l74;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l76;
            s_i64_1 = l75;
            ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
            s_i32_0 = l76;
            l77 = s_i32_0;
            s_i32_0 = l77;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i32_0 = 1i32;
    l78 = s_i32_0;
    s_i64_0 = l65;
    s_i64_1 = 0i64;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_b3e6ca84b2de(vm, s_i64_0, s_i64_1, host);
    vm.exit_call();
    let __t61 = __call_res?;
    s_i32_0 = __t61;
    l79 = s_i32_0;
    s_i64_0 = l1;
    s_i64_1 = l57;
    vm.enter_call()?;
    let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
    vm.exit_call();
    let __t62 = __call_res?;
    s_i64_0 = __t62;
    l80 = s_i64_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
    vm.exit_call();
    let __t63 = __call_res?;
    s_i32_0 = __t63;
    l81 = s_i32_0;
    s_i32_0 = l81;
    s_i32_1 = l78;
    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l81;
    s_i32_1 = l79;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l81;
    s_i64_1 = l80;
    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
    s_i32_0 = l81;
    l82 = s_i32_0;
    s_i32_0 = l82;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn s_5daf123ed54b(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<(), WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i32_2: i32 = 0i32;
    vm.deduct(host, 1)?;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 12u32)?;
    s_i32_1 = 0i32;
    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    l2 = s_i32_0;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
    l3 = s_i32_0;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    l4 = s_i32_0;
    s_i32_0 = l2;
    s_i32_1 = l3;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = l2;
            s_i32_1 = 2147417856i32;
            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
            'blk_3: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = l3;
            s_i32_1 = 8i32;
            s_i32_0 = ops::i32_lt_u(s_i32_0, s_i32_1);
            'blk_4: {
                if s_i32_0 != 0 {
                    s_i32_0 = 8i32;
                    l5 = s_i32_0;
                } else {
                    s_i32_0 = l3;
                    s_i32_1 = 1073708928i32;
                    s_i32_0 = ops::i32_gt_u(s_i32_0, s_i32_1);
                    'blk_5: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 2147417856i32;
                            l5 = s_i32_0;
                        } else {
                            s_i32_0 = l3;
                            s_i32_1 = 1i32;
                            s_i32_0 = ops::i32_shl(s_i32_0, s_i32_1);
                            l5 = s_i32_0;
                        }
                    }
                }
            }
            s_i32_0 = l5;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t0 = __call_res?;
            s_i32_0 = __t0;
            l6 = s_i32_0;
            s_i32_0 = l2;
            s_i32_1 = 0i32;
            s_i32_0 = ops::i32_gt_u(s_i32_0, s_i32_1);
            'blk_6: {
                if s_i32_0 != 0 {
                    s_i32_0 = l6;
                    s_i32_1 = l4;
                    s_i32_2 = l2;
                    vm.memory_copy(0, 0, s_i32_0, s_i32_1, s_i32_2, host)?;
                }
            }
            s_i32_0 = l0;
            s_i32_1 = l6;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l0;
            s_i32_1 = l5;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l6;
            l4 = s_i32_0;
        }
    }
    s_i32_0 = l4;
    s_i32_1 = l2;
    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
    s_i32_1 = l1;
    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l0;
    s_i32_1 = l2;
    s_i32_2 = 1i32;
    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    return Ok(());
    }

    pub(crate) fn s_5e3d0c000a09(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_1 = l1;
    vm.enter_call()?;
    let __call_res = Self::s_5b883f251b47(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    s_i32_1 = 0i32;
    s_i32_0 = ops::i32_gt_s(s_i32_0, s_i32_1);
    l2 = s_i32_0;
    s_i32_0 = l2;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn s_5f2ed55f6d0e(vm: &mut SubstituteVm, p0: i32, g0: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i64 = 0i64;
    let mut l2: i64 = 0i64;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i32 = 0i32;
    let mut l7: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i32_2: i32 = 0i32;
    let mut s_i64_2: i64 = 0i64;
    let mut s_i32_3: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 0u32)?;
    l1 = s_i64_0;
    s_i32_0 = l0;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
    l2 = s_i64_0;
    s_i32_0 = 0i32;
    s_i64_1 = l1;
    s_i64_2 = 4294967295i64;
    s_i64_1 = ops::i64_and(s_i64_1, s_i64_2);
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_d5f50d7e651d(vm, s_i64_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_1 = __t0;
    l3 = s_i32_1;
    s_i64_1 = l1;
    s_i32_2 = 32i32;
    vm.enter_call()?;
    let __call_res = Self::s_73a866edda27(vm, s_i64_1, s_i32_2, host);
    vm.exit_call();
    let __t1 = __call_res?;
    s_i64_1 = __t1;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_d5f50d7e651d(vm, s_i64_1, host);
    vm.exit_call();
    let __t2 = __call_res?;
    s_i32_1 = __t2;
    l4 = s_i32_1;
    s_i64_1 = l2;
    s_i64_2 = 4294967295i64;
    s_i64_1 = ops::i64_and(s_i64_1, s_i64_2);
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_d5f50d7e651d(vm, s_i64_1, host);
    vm.exit_call();
    let __t3 = __call_res?;
    s_i32_1 = __t3;
    l5 = s_i32_1;
    s_i64_1 = l2;
    s_i32_2 = 32i32;
    vm.enter_call()?;
    let __call_res = Self::s_73a866edda27(vm, s_i64_1, s_i32_2, host);
    vm.exit_call();
    let __t4 = __call_res?;
    s_i64_1 = __t4;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_d5f50d7e651d(vm, s_i64_1, host);
    vm.exit_call();
    let __t5 = __call_res?;
    s_i32_1 = __t5;
    l6 = s_i32_1;
    s_i32_1 = 32i32;
    vm.enter_call()?;
    let __call_res = Self::s_498abb70f200(vm, s_i32_1, host);
    vm.exit_call();
    let __t6 = __call_res?;
    s_i32_1 = __t6;
    l7 = s_i32_1;
    s_i32_1 = l7;
    s_i32_2 = l7;
    s_i32_3 = 16i32;
    s_i32_2 = ops::i32_add(s_i32_2, s_i32_3);
    ops::i32_store(vm, 0, s_i32_1, 0u32, s_i32_2)?;
    s_i32_1 = l7;
    s_i32_2 = 4i32;
    ops::i32_store(vm, 0, s_i32_1, 4u32, s_i32_2)?;
    s_i32_1 = l7;
    s_i32_2 = 4i32;
    ops::i32_store(vm, 0, s_i32_1, 8u32, s_i32_2)?;
    s_i32_1 = l7;
    s_i32_2 = 1i32;
    ops::i32_store(vm, 0, s_i32_1, 12u32, s_i32_2)?;
    s_i32_1 = l7;
    s_i32_2 = l3;
    ops::i32_store(vm, 0, s_i32_1, 16u32, s_i32_2)?;
    s_i32_1 = l7;
    s_i32_2 = l4;
    ops::i32_store(vm, 0, s_i32_1, 20u32, s_i32_2)?;
    s_i32_1 = l7;
    s_i32_2 = l5;
    ops::i32_store(vm, 0, s_i32_1, 24u32, s_i32_2)?;
    s_i32_1 = l7;
    s_i32_2 = l6;
    ops::i32_store(vm, 0, s_i32_1, 28u32, s_i32_2)?;
    s_i32_1 = l7;
    let __t7 = s_i32_0;
    let __t8 = s_i32_1;
    return Self::s_482c16ff60d0(vm, __t7, __t8, g0, host);
    }

    pub(crate) fn s_60a8e37a9726(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i32_2: i32 = 0i32;
    let mut s_i32_3: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l1;
    s_i32_1 = 127i32;
    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
    l2 = s_i32_0;
    s_i32_0 = l2;
    s_i32_1 = 0i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
        } else {
            s_i32_0 = l0;
            s_i32_1 = l2;
            vm.enter_call()?;
            let __call_res = Self::s_7009a9e7c9fa(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t0 = __call_res?;
            s_i32_0 = __t0;
            s_i32_1 = l0;
            s_i32_2 = 128i32;
            s_i32_3 = l2;
            vm.enter_call()?;
            let __call_res = Self::s_304f22c4ab8c(vm, s_i32_2, s_i32_3, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_2 = __t1;
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_fa6af9f06252(vm, s_i32_1, s_i32_2, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_1 = __t2;
            let __t3 = s_i32_0;
            let __t4 = s_i32_1;
            return crate::shared_1::Shared::s_9c006b2946a7(vm, __t3, __t4, host);
        }
    }
    l3 = s_i32_0;
    s_i32_0 = l3;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_60e5b4741e78(vm: &mut SubstituteVm, p0: i32, g0: usize, g1: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = 0i32;
    let mut l2: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_f7d297dbbc03(vm, s_i32_0, g0, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l1 = s_i32_0;
    s_i32_0 = l1;
    vm.enter_call()?;
    let __call_res = Self::s_5f2ed55f6d0e(vm, s_i32_0, g1, host);
    vm.exit_call();
    let __t1 = __call_res?;
    s_i32_0 = __t1;
    l2 = s_i32_0;
    s_i32_0 = l0;
    vm.enter_call()?;
    let __call_res = Self::s_51f0c0b4c723(vm, s_i32_0, host);
    vm.exit_call();
    let __t2 = __call_res?;
    s_i32_0 = __t2;
    s_i32_1 = l2;
    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
    let __t3 = s_i32_0;
    let __t4 = s_i32_1;
    return Self::s_482c16ff60d0(vm, __t3, __t4, g1, host);
    }

    pub(crate) fn s_61029c4c8df8(vm: &mut SubstituteVm, p0: i32, p1: i64, p2: i64, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i64 = p1;
    let mut l2: i64 = p2;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i32_2: i32 = 0i32;
    let mut s_i64_2: i64 = 0i64;
    vm.deduct(host, 1)?;
    s_i64_0 = l1;
    s_i64_1 = l2;
    s_i32_0 = ops::i64_gt_u(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i64_0 = l2;
    s_i32_1 = l0;
    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
    s_i64_1 = ops::i64_extend_i32_u(s_i32_1);
    s_i32_0 = ops::i64_gt_u(s_i64_0, s_i64_1);
    'blk_2: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i64_0 = l2;
    s_i64_1 = l1;
    s_i64_0 = ops::i64_sub(s_i64_0, s_i64_1);
    s_i32_0 = ops::i32_wrap_i64(s_i64_0);
    l3 = s_i32_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l4 = s_i32_0;
    s_i32_0 = l4;
    s_i32_1 = l0;
    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 0u32)?;
    s_i64_2 = l1;
    s_i32_2 = ops::i32_wrap_i64(s_i64_2);
    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l4;
    s_i32_1 = l3;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l4;
    s_i32_1 = l3;
    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
    s_i32_0 = l4;
    s_i32_1 = 1i32;
    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
    s_i32_0 = l4;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_61878e7d5ba7(vm: &mut SubstituteVm, p0: i64, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i64 = p0;
    let mut l1: i32 = 0i32;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    vm.deduct(host, 1)?;
    s_i64_0 = l0;
    s_i64_1 = 65536i64;
    s_i32_0 = ops::i64_gt_u(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 65536i32;
            l1 = s_i32_0;
        } else {
            s_i64_0 = l0;
            s_i32_0 = ops::i32_wrap_i64(s_i64_0);
            l1 = s_i32_0;
        }
    }
    s_i32_0 = 16i32;
    s_i32_1 = l1;
    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
    vm.enter_call()?;
    let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l2 = s_i32_0;
    s_i32_0 = l2;
    s_i32_1 = 16i32;
    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
    l3 = s_i32_0;
    s_i32_0 = l2;
    s_i32_1 = l3;
    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l2;
    s_i32_1 = 0i32;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l2;
    s_i32_1 = l1;
    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
    s_i32_0 = l2;
    s_i32_1 = 0i32;
    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
    s_i32_0 = l2;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_623491ac0af2(vm: &mut SubstituteVm, p0: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i64 = 0i64;
    let mut l2: i64 = 0i64;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i64_1: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 0u32)?;
    l1 = s_i64_0;
    s_i32_0 = l0;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
    l2 = s_i64_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l3 = s_i32_0;
    s_i32_0 = l3;
    s_i64_1 = l1;
    ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
    s_i32_0 = l3;
    s_i64_1 = l2;
    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
    s_i32_0 = l3;
    l4 = s_i32_0;
    s_i32_0 = l4;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_62439e6f6ae7(vm: &mut SubstituteVm, p0: i32, g0: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i64_1: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    vm.enter_call()?;
    let __call_res = Self::s_383cd8b1812f(vm, s_i32_0, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 0u32)?;
            s_i64_1 = 1114112i64;
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_aa4577c560e8(vm, s_i64_0, s_i64_1, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i64_0 = __t1;
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_d5f50d7e651d(vm, s_i64_0, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_0 = __t2;
        } else {
            s_i32_0 = vm.get_i32(g0)?;
        }
    }
    l1 = s_i32_0;
    s_i32_0 = l1;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_62932ccbf570(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, g1: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_1 = l1;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_c92ee68c0844(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
            l2 = s_i32_0;
            s_i32_0 = l0;
            s_i32_1 = l1;
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_95ed5cb908ad(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            l3 = s_i32_0;
            s_i32_0 = 0i32;
            l4 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_0 = __t2;
            l5 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = l2;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l5;
            s_i32_1 = l3;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l5;
            s_i32_1 = l4;
            ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l5;
        } else {
            s_i32_0 = 0i32;
            l6 = s_i32_0;
            s_i32_0 = vm.get_i32(g0)?;
            l7 = s_i32_0;
            s_i32_0 = vm.get_i32(g1)?;
            l8 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i32_0 = __t3;
            l9 = s_i32_0;
            s_i32_0 = l9;
            s_i32_1 = l6;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l9;
            s_i32_1 = l7;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l9;
            s_i32_1 = l8;
            ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l9;
        }
    }
    l10 = s_i32_0;
    s_i32_0 = l10;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_6294292a1d8b(vm: &mut SubstituteVm, p0: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = 0i32;
    let mut l2: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_f2fcb0a8824e(vm, s_i32_0, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i64_0 = __t0;
    vm.enter_call()?;
    let __call_res = Self::s_61878e7d5ba7(vm, s_i64_0, host);
    vm.exit_call();
    let __t1 = __call_res?;
    s_i32_0 = __t1;
    l1 = s_i32_0;
    s_i32_0 = l1;
    s_i32_1 = l0;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_eb4cabbb1a10(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l1;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_df3679579582(vm, s_i32_0, host);
    vm.exit_call();
    let __t2 = __call_res?;
    s_i32_0 = __t2;
    l2 = s_i32_0;
    s_i32_0 = l2;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_6309b496bb23(vm: &mut SubstituteVm, p0: i32, g0: usize, g1: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = 0i32;
    let mut l2: i32 = 0i32;
    let mut l3: i64 = 0i64;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    vm.enter_call()?;
    let __call_res = Self::s_4200fb7655b9(vm, s_i32_0, g0, g1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l1 = s_i32_0;
    s_i32_0 = l1;
    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
    s_i32_0 = ops::i32_eqz(s_i32_0);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            l2 = s_i32_0;
            s_i64_0 = 0i64;
            l3 = s_i64_0;
            s_i32_0 = vm.get_i32(g1)?;
            l4 = s_i32_0;
            s_i32_0 = 20i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            l5 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = l2;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l5;
            s_i64_1 = l3;
            ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
            s_i32_0 = l5;
            s_i32_1 = l4;
            ops::i32_store8(vm, 0, s_i32_0, 16u32, s_i32_1)?;
            s_i32_0 = l5;
        } else {
            s_i32_0 = l1;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
            let __t2 = s_i32_0;
            return Self::s_4adad98baaec(vm, __t2, g1, host);
        }
    }
    l6 = s_i32_0;
    s_i32_0 = l6;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn s_66cfb899ddc3(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i64 = 0i64;
    let mut l3: i64 = 0i64;
    let mut l4: i32 = 0i32;
    let mut l5: i64 = 0i64;
    let mut l6: i64 = 0i64;
    let mut l7: i64 = 0i64;
    let mut l8: i64 = 0i64;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i64_2: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
    l2 = s_i64_0;
    s_i32_0 = l1;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
    l3 = s_i64_0;
    s_i64_0 = l2;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_c5da14f612f0(vm, s_i64_0, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l4 = s_i32_0;
    s_i64_0 = 0i64;
    l5 = s_i64_0;
    s_i64_0 = 0i64;
    l6 = s_i64_0;
    'blk_1: {
        'blk_2: loop {
            vm.deduct(host, 1)?;
            s_i64_0 = l6;
            s_i64_1 = l2;
            s_i32_0 = ops::i64_lt_u(s_i64_0, s_i64_1);
            s_i32_0 = ops::i32_eqz(s_i32_0);
            if s_i32_0 != 0 {
                break 'blk_1;
            }
            s_i32_0 = l0;
            s_i64_1 = l6;
            vm.enter_call()?;
            let __call_res = Self::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
            l7 = s_i64_0;
            s_i64_0 = l6;
            s_i64_1 = l3;
            s_i32_0 = ops::i64_lt_u(s_i64_0, s_i64_1);
            'blk_3: {
                if s_i32_0 != 0 {
                    s_i32_0 = l1;
                    s_i64_1 = l6;
                    vm.enter_call()?;
                    let __call_res = Self::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
                    vm.exit_call();
                    let __t2 = __call_res?;
                    s_i32_0 = __t2;
                    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
                } else {
                    s_i64_0 = 0i64;
                }
            }
            s_i64_1 = l5;
            vm.enter_call()?;
            let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i64_0 = __t3;
            l8 = s_i64_0;
            s_i64_0 = l7;
            s_i64_1 = l8;
            s_i32_0 = ops::i64_ge_u(s_i64_0, s_i64_1);
            'blk_4: {
                if s_i32_0 != 0 {
                    s_i32_0 = l4;
                    s_i64_1 = l7;
                    s_i64_2 = l8;
                    vm.enter_call()?;
                    let __call_res = crate::shared_1::Shared::s_aa4577c560e8(vm, s_i64_1, s_i64_2, host);
                    vm.exit_call();
                    let __t4 = __call_res?;
                    s_i64_1 = __t4;
                    vm.enter_call()?;
                    let __call_res = crate::shared_1::Shared::s_d5f50d7e651d(vm, s_i64_1, host);
                    vm.exit_call();
                    let __t5 = __call_res?;
                    s_i32_1 = __t5;
                    vm.enter_call()?;
                    let __call_res = Self::s_697cb42b9ce7(vm, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                    s_i64_0 = 0i64;
                    l5 = s_i64_0;
                } else {
                    s_i32_0 = l4;
                    s_i64_1 = l7;
                    s_i64_2 = 4294967296i64;
                    vm.enter_call()?;
                    let __call_res = Self::s_1c917789866c(vm, s_i64_1, s_i64_2, host);
                    vm.exit_call();
                    let __t6 = __call_res?;
                    s_i64_1 = __t6;
                    s_i64_2 = l8;
                    vm.enter_call()?;
                    let __call_res = crate::shared_1::Shared::s_aa4577c560e8(vm, s_i64_1, s_i64_2, host);
                    vm.exit_call();
                    let __t7 = __call_res?;
                    s_i64_1 = __t7;
                    vm.enter_call()?;
                    let __call_res = crate::shared_1::Shared::s_d5f50d7e651d(vm, s_i64_1, host);
                    vm.exit_call();
                    let __t8 = __call_res?;
                    s_i32_1 = __t8;
                    vm.enter_call()?;
                    let __call_res = Self::s_697cb42b9ce7(vm, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                    s_i64_0 = 1i64;
                    l5 = s_i64_0;
                }
            }
            s_i64_0 = l6;
            s_i64_1 = 1i64;
            vm.enter_call()?;
            let __call_res = Self::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
            vm.exit_call();
            let __t9 = __call_res?;
            s_i64_0 = __t9;
            l6 = s_i64_0;
            continue 'blk_2;
            break 'blk_2;
        }
    }
    s_i32_0 = l4;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_df3679579582(vm, s_i32_0, host);
    vm.exit_call();
    let __t10 = __call_res?;
    s_i32_0 = __t10;
    let __t11 = s_i32_0;
    return crate::shared_1::Shared::s_b92cf85b6153(vm, __t11, host);
    }

    pub(crate) fn s_695eab7c815a(vm: &mut SubstituteVm, p0: i32, p1: i32, p2: i32, p3: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = p2;
    let mut l3: i32 = p3;
    let mut l4: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i32_2: i32 = 0i32;
    vm.deduct(host, 1)?;
    s_i32_0 = l1;
    s_i32_1 = l3;
    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = 0i32;
    l4 = s_i32_0;
    'blk_2: {
        'blk_3: loop {
            vm.deduct(host, 1)?;
            s_i32_0 = l4;
            s_i32_1 = l1;
            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
            if s_i32_0 != 0 {
                break 'blk_2;
            }
            s_i32_0 = l0;
            s_i32_1 = l4;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
            s_i32_1 = l2;
            s_i32_2 = l4;
            s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
            s_i32_1 = ops::i32_load8_u(vm, 0, s_i32_1, 0u32)?;
            s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
            'blk_4: {
                if s_i32_0 != 0 {
                    s_i32_0 = 0i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l4;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l4 = s_i32_0;
            continue 'blk_3;
            break 'blk_3;
        }
    }
    s_i32_0 = 1i32;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn s_697cb42b9ce7(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<(), WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
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
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 12u32)?;
    s_i32_1 = 0i32;
    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    l2 = s_i32_0;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
    l3 = s_i32_0;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    l4 = s_i32_0;
    s_i32_0 = l2;
    s_i32_1 = l3;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = l2;
            s_i32_1 = 536854464i32;
            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
            'blk_3: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
            s_i32_0 = l3;
            s_i32_1 = 8i32;
            s_i32_0 = ops::i32_lt_u(s_i32_0, s_i32_1);
            'blk_4: {
                if s_i32_0 != 0 {
                    s_i32_0 = 8i32;
                    l5 = s_i32_0;
                } else {
                    s_i32_0 = l3;
                    s_i32_1 = 268427232i32;
                    s_i32_0 = ops::i32_gt_u(s_i32_0, s_i32_1);
                    'blk_5: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 536854464i32;
                            l5 = s_i32_0;
                        } else {
                            s_i32_0 = l3;
                            s_i32_1 = 1i32;
                            s_i32_0 = ops::i32_shl(s_i32_0, s_i32_1);
                            l5 = s_i32_0;
                        }
                    }
                }
            }
            s_i32_0 = l5;
            s_i32_1 = 2i32;
            s_i32_0 = ops::i32_shl(s_i32_0, s_i32_1);
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t0 = __call_res?;
            s_i32_0 = __t0;
            l6 = s_i32_0;
            s_i32_0 = l2;
            s_i32_1 = 0i32;
            s_i32_0 = ops::i32_gt_u(s_i32_0, s_i32_1);
            'blk_6: {
                if s_i32_0 != 0 {
                    s_i32_0 = l6;
                    s_i32_1 = l4;
                    s_i32_2 = l2;
                    s_i32_3 = 2i32;
                    s_i32_2 = ops::i32_shl(s_i32_2, s_i32_3);
                    vm.memory_copy(0, 0, s_i32_0, s_i32_1, s_i32_2, host)?;
                }
            }
            s_i32_0 = l0;
            s_i32_1 = l6;
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l0;
            s_i32_1 = l5;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l6;
            l4 = s_i32_0;
        }
    }
    s_i32_0 = l4;
    s_i32_1 = l2;
    s_i32_2 = 2i32;
    s_i32_1 = ops::i32_shl(s_i32_1, s_i32_2);
    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
    s_i32_1 = l1;
    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l0;
    s_i32_1 = l2;
    s_i32_2 = 1i32;
    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    return Ok(());
    }

    pub(crate) fn s_69a15786a590(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_f76e9849d691(vm, s_i32_0, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    s_i32_1 = l1;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_f76e9849d691(vm, s_i32_1, host);
    vm.exit_call();
    let __t1 = __call_res?;
    s_i32_1 = __t1;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_8cfc64e181cb(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t2 = __call_res?;
    s_i32_0 = __t2;
    let __t3 = s_i32_0;
    return Self::s_623491ac0af2(vm, __t3, host);
    }

    pub(crate) fn s_6d1067ace5f6(vm: &mut SubstituteVm, p0: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i64 = 0i64;
    let mut l2: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 0u32)?;
    l1 = s_i64_0;
    s_i64_0 = l1;
    s_i64_1 = 0i64;
    s_i32_0 = ops::i64_ne(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i64_0 = l1;
            s_i64_0 = ops::i64_ctz(s_i64_0);
            s_i32_0 = ops::i32_wrap_i64(s_i64_0);
        } else {
            s_i32_0 = 64i32;
            s_i32_1 = l0;
            s_i64_1 = ops::i64_load(vm, 0, s_i32_1, 8u32)?;
            s_i64_1 = ops::i64_ctz(s_i64_1);
            s_i32_1 = ops::i32_wrap_i64(s_i64_1);
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_91b7532ed2b6(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t0 = __call_res?;
            s_i32_0 = __t0;
        }
    }
    l2 = s_i32_0;
    s_i32_0 = l2;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_6da36f629329(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, g1: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_1 = l1;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_e68f8dc79e1d(vm, s_i32_0, s_i32_1, g0, g1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l2 = s_i32_0;
    s_i32_0 = l2;
    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
    s_i32_0 = ops::i32_eqz(s_i32_0);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        } else {
            s_i32_0 = l2;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
        }
    }
    l3 = s_i32_0;
    s_i32_0 = l3;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn s_6e6223e30e05(vm: &mut SubstituteVm, p0: i32, p1: i32, p2: i32, g0: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let mut l13: i64 = 0i64;
    let mut l14: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i32_2: i32 = 0i32;
    let mut s_i64_2: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l2;
    s_i32_1 = vm.get_i32(g0)?;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_84059c53f331(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 6i32;
            vm.enter_call()?;
            let __call_res = Self::s_1232b68aede8(vm, s_i32_0, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            l3 = s_i32_0;
            s_i32_0 = l0;
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_c7105d8549b1(vm, s_i32_0, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_0 = __t2;
            l4 = s_i32_0;
            s_i32_0 = 127i32;
            vm.enter_call()?;
            let __call_res = Self::s_1232b68aede8(vm, s_i32_0, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i32_0 = __t3;
            l5 = s_i32_0;
            s_i32_0 = 7i32;
            vm.enter_call()?;
            let __call_res = Self::s_1232b68aede8(vm, s_i32_0, host);
            vm.exit_call();
            let __t4 = __call_res?;
            s_i32_0 = __t4;
            l6 = s_i32_0;
            s_i32_0 = 32i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t5 = __call_res?;
            s_i32_0 = __t5;
            l7 = s_i32_0;
            s_i32_0 = l7;
            s_i32_1 = l7;
            s_i32_2 = 16i32;
            s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l7;
            s_i32_1 = 4i32;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l7;
            s_i32_1 = 4i32;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l7;
            s_i32_1 = 1i32;
            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
            s_i32_0 = l7;
            s_i32_1 = l3;
            ops::i32_store(vm, 0, s_i32_0, 16u32, s_i32_1)?;
            s_i32_0 = l7;
            s_i32_1 = l4;
            ops::i32_store(vm, 0, s_i32_0, 20u32, s_i32_1)?;
            s_i32_0 = l7;
            s_i32_1 = l5;
            ops::i32_store(vm, 0, s_i32_0, 24u32, s_i32_1)?;
            s_i32_0 = l7;
            s_i32_1 = l6;
            ops::i32_store(vm, 0, s_i32_0, 28u32, s_i32_1)?;
            s_i32_0 = l7;
            l8 = s_i32_0;
            s_i32_0 = l8;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i64_0 = 26i64;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_c5da14f612f0(vm, s_i64_0, host);
    vm.exit_call();
    let __t6 = __call_res?;
    s_i32_0 = __t6;
    l9 = s_i32_0;
    s_i32_0 = l9;
    s_i32_1 = 6i32;
    vm.enter_call()?;
    let __call_res = Self::s_1232b68aede8(vm, s_i32_1, host);
    vm.exit_call();
    let __t7 = __call_res?;
    s_i32_1 = __t7;
    vm.enter_call()?;
    let __call_res = Self::s_697cb42b9ce7(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l9;
    s_i32_1 = l0;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_c7105d8549b1(vm, s_i32_1, host);
    vm.exit_call();
    let __t8 = __call_res?;
    s_i32_1 = __t8;
    vm.enter_call()?;
    let __call_res = Self::s_697cb42b9ce7(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l1;
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = l9;
            s_i32_1 = 11i32;
            vm.enter_call()?;
            let __call_res = Self::s_1232b68aede8(vm, s_i32_1, host);
            vm.exit_call();
            let __t9 = __call_res?;
            s_i32_1 = __t9;
            vm.enter_call()?;
            let __call_res = Self::s_697cb42b9ce7(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
        } else {
        }
    }
    s_i64_0 = 22i64;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_c5da14f612f0(vm, s_i64_0, host);
    vm.exit_call();
    let __t10 = __call_res?;
    s_i32_0 = __t10;
    l10 = s_i32_0;
    s_i32_0 = l2;
    l11 = s_i32_0;
    'blk_3: {
        'blk_4: loop {
            vm.deduct(host, 1)?;
            s_i32_0 = l11;
            s_i32_1 = vm.get_i32(g0)?;
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_84059c53f331(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t11 = __call_res?;
            s_i32_0 = __t11;
            s_i32_0 = ops::i32_eqz(s_i32_0);
            s_i32_0 = ops::i32_eqz(s_i32_0);
            if s_i32_0 != 0 {
                break 'blk_3;
            }
            s_i32_0 = l11;
            s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 0u32)?;
            s_i64_1 = 63i64;
            s_i64_0 = ops::i64_and(s_i64_0, s_i64_1);
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_d5f50d7e651d(vm, s_i64_0, host);
            vm.exit_call();
            let __t12 = __call_res?;
            s_i32_0 = __t12;
            l12 = s_i32_0;
            s_i32_0 = l10;
            s_i32_1 = 127i32;
            s_i32_2 = l12;
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_91b7532ed2b6(vm, s_i32_1, s_i32_2, host);
            vm.exit_call();
            let __t13 = __call_res?;
            s_i32_1 = __t13;
            vm.enter_call()?;
            let __call_res = Self::s_697cb42b9ce7(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l11;
            s_i32_1 = 6i32;
            vm.enter_call()?;
            let __call_res = Self::s_7009a9e7c9fa(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t14 = __call_res?;
            s_i32_0 = __t14;
            l11 = s_i32_0;
            continue 'blk_4;
            break 'blk_4;
        }
    }
    s_i32_0 = l10;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
    l13 = s_i64_0;
    'blk_5: {
        'blk_6: loop {
            vm.deduct(host, 1)?;
            s_i64_0 = l13;
            s_i64_1 = 0i64;
            s_i32_0 = ops::i64_gt_u(s_i64_0, s_i64_1);
            s_i32_0 = ops::i32_eqz(s_i32_0);
            if s_i32_0 != 0 {
                break 'blk_5;
            }
            s_i64_0 = l13;
            s_i64_1 = 1i64;
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_aa4577c560e8(vm, s_i64_0, s_i64_1, host);
            vm.exit_call();
            let __t15 = __call_res?;
            s_i64_0 = __t15;
            l13 = s_i64_0;
            s_i32_0 = l9;
            s_i32_1 = l10;
            s_i64_2 = l13;
            vm.enter_call()?;
            let __call_res = Self::s_17df4637c35a(vm, s_i32_1, s_i64_2, host);
            vm.exit_call();
            let __t16 = __call_res?;
            s_i32_1 = __t16;
            vm.enter_call()?;
            let __call_res = Self::s_1232b68aede8(vm, s_i32_1, host);
            vm.exit_call();
            let __t17 = __call_res?;
            s_i32_1 = __t17;
            vm.enter_call()?;
            let __call_res = Self::s_697cb42b9ce7(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            continue 'blk_6;
            break 'blk_6;
        }
    }
    s_i32_0 = l9;
    s_i32_1 = 7i32;
    vm.enter_call()?;
    let __call_res = Self::s_1232b68aede8(vm, s_i32_1, host);
    vm.exit_call();
    let __t18 = __call_res?;
    s_i32_1 = __t18;
    vm.enter_call()?;
    let __call_res = Self::s_697cb42b9ce7(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l9;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_df3679579582(vm, s_i32_0, host);
    vm.exit_call();
    let __t19 = __call_res?;
    s_i32_0 = __t19;
    l14 = s_i32_0;
    s_i32_0 = l14;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_6ecb218e2168(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i32_2: i32 = 0i32;
    vm.deduct(host, 1)?;
    s_i32_0 = l1;
    s_i32_1 = l0;
    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
    s_i32_2 = 2i32;
    s_i32_1 = ops::i32_shr_u(s_i32_1, s_i32_2);
    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    s_i32_1 = l1;
    s_i32_2 = 2i32;
    s_i32_1 = ops::i32_shl(s_i32_1, s_i32_2);
    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn s_7009a9e7c9fa(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i64 = 0i64;
    let mut l3: i64 = 0i64;
    let mut l4: i64 = 0i64;
    let mut l5: i64 = 0i64;
    let mut l6: i32 = 0i32;
    let mut l7: i64 = 0i64;
    let mut l8: i64 = 0i64;
    let mut l9: i32 = 0i32;
    let mut l10: i64 = 0i64;
    let mut l11: i64 = 0i64;
    let mut l12: i32 = 0i32;
    let mut l13: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i32_2: i32 = 0i32;
    let mut s_i32_3: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 0u32)?;
    l2 = s_i64_0;
    s_i32_0 = l0;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
    l3 = s_i64_0;
    s_i32_0 = l1;
    s_i32_1 = 0i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
        } else {
            s_i32_0 = l1;
            s_i32_1 = 64i32;
            s_i32_0 = ops::i32_lt_u(s_i32_0, s_i32_1);
            'blk_2: {
                if s_i32_0 != 0 {
                    s_i64_0 = l2;
                    s_i32_1 = l1;
                    vm.enter_call()?;
                    let __call_res = Self::s_73a866edda27(vm, s_i64_0, s_i32_1, host);
                    vm.exit_call();
                    let __t0 = __call_res?;
                    s_i64_0 = __t0;
                    s_i64_1 = l3;
                    s_i32_2 = 64i32;
                    s_i32_3 = l1;
                    vm.enter_call()?;
                    let __call_res = Self::s_304f22c4ab8c(vm, s_i32_2, s_i32_3, host);
                    vm.exit_call();
                    let __t1 = __call_res?;
                    s_i32_2 = __t1;
                    vm.enter_call()?;
                    let __call_res = Self::s_52dcb6d97135(vm, s_i64_1, s_i32_2, host);
                    vm.exit_call();
                    let __t2 = __call_res?;
                    s_i64_1 = __t2;
                    s_i64_0 = ops::i64_or(s_i64_0, s_i64_1);
                    l4 = s_i64_0;
                    s_i64_0 = l3;
                    s_i32_1 = l1;
                    vm.enter_call()?;
                    let __call_res = Self::s_73a866edda27(vm, s_i64_0, s_i32_1, host);
                    vm.exit_call();
                    let __t3 = __call_res?;
                    s_i64_0 = __t3;
                    l5 = s_i64_0;
                    s_i32_0 = 16i32;
                    vm.enter_call()?;
                    let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
                    vm.exit_call();
                    let __t4 = __call_res?;
                    s_i32_0 = __t4;
                    l6 = s_i32_0;
                    s_i32_0 = l6;
                    s_i64_1 = l4;
                    ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
                    s_i32_0 = l6;
                    s_i64_1 = l5;
                    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
                    s_i32_0 = l6;
                } else {
                    s_i32_0 = l1;
                    s_i32_1 = 64i32;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                    'blk_3: {
                        if s_i32_0 != 0 {
                            s_i64_0 = l3;
                            l7 = s_i64_0;
                            s_i64_0 = 0i64;
                            l8 = s_i64_0;
                            s_i32_0 = 16i32;
                            vm.enter_call()?;
                            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
                            vm.exit_call();
                            let __t5 = __call_res?;
                            s_i32_0 = __t5;
                            l9 = s_i32_0;
                            s_i32_0 = l9;
                            s_i64_1 = l7;
                            ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
                            s_i32_0 = l9;
                            s_i64_1 = l8;
                            ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
                            s_i32_0 = l9;
                        } else {
                            s_i64_0 = l3;
                            s_i32_1 = l1;
                            s_i32_2 = 64i32;
                            vm.enter_call()?;
                            let __call_res = Self::s_304f22c4ab8c(vm, s_i32_1, s_i32_2, host);
                            vm.exit_call();
                            let __t6 = __call_res?;
                            s_i32_1 = __t6;
                            vm.enter_call()?;
                            let __call_res = Self::s_73a866edda27(vm, s_i64_0, s_i32_1, host);
                            vm.exit_call();
                            let __t7 = __call_res?;
                            s_i64_0 = __t7;
                            l10 = s_i64_0;
                            s_i64_0 = 0i64;
                            l11 = s_i64_0;
                            s_i32_0 = 16i32;
                            vm.enter_call()?;
                            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
                            vm.exit_call();
                            let __t8 = __call_res?;
                            s_i32_0 = __t8;
                            l12 = s_i32_0;
                            s_i32_0 = l12;
                            s_i64_1 = l10;
                            ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
                            s_i32_0 = l12;
                            s_i64_1 = l11;
                            ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
                            s_i32_0 = l12;
                        }
                    }
                }
            }
        }
    }
    l13 = s_i32_0;
    s_i32_0 = l13;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_702be4e99395(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i32_2: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
    s_i32_1 = l1;
    s_i32_1 = ops::i32_load8_u(vm, 0, s_i32_1, 0u32)?;
    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
    s_i32_1 = l0;
    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
    s_i32_2 = l1;
    s_i32_2 = ops::i32_load(vm, 0, s_i32_2, 4u32)?;
    vm.enter_call()?;
    let __call_res = Self::s_3dbd6afaf7f2(vm, s_i32_1, s_i32_2, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_1 = __t0;
    let __t1 = s_i32_0;
    let __t2 = s_i32_1;
    return Self::s_482c16ff60d0(vm, __t1, __t2, g0, host);
    }

    pub(crate) fn s_708c3053a6ba(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    s_i32_0 = l1;
    s_i32_1 = 8i32;
    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = l0;
    s_i32_1 = 255i32;
    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
    s_i32_1 = l1;
    s_i32_0 = ops::i32_shr_u(s_i32_0, s_i32_1);
    return Ok(s_i32_0);
    }

    pub(crate) fn s_72a31b5ae287(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l1;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
    l2 = s_i64_0;
    s_i32_0 = l0;
    s_i64_1 = l2;
    vm.enter_call()?;
    let __call_res = Self::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    s_i32_1 = 0i32;
    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
            l3 = s_i32_0;
            s_i32_0 = l3;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i64_0 = l2;
    l4 = s_i64_0;
    'blk_2: {
        'blk_3: loop {
            vm.deduct(host, 1)?;
            s_i64_0 = l4;
            s_i64_1 = 0i64;
            s_i32_0 = ops::i64_gt_u(s_i64_0, s_i64_1);
            s_i32_0 = ops::i32_eqz(s_i32_0);
            if s_i32_0 != 0 {
                break 'blk_2;
            }
            s_i64_0 = l4;
            s_i64_1 = 1i64;
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_aa4577c560e8(vm, s_i64_0, s_i64_1, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i64_0 = __t1;
            l4 = s_i64_0;
            s_i32_0 = l0;
            s_i64_1 = l4;
            vm.enter_call()?;
            let __call_res = Self::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_0 = __t2;
            l5 = s_i32_0;
            s_i32_0 = l1;
            s_i64_1 = l4;
            vm.enter_call()?;
            let __call_res = Self::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i32_0 = __t3;
            l6 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = l6;
            s_i32_0 = ops::i32_gt_u(s_i32_0, s_i32_1);
            'blk_4: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                    l7 = s_i32_0;
                    s_i32_0 = l7;
                    return Ok(s_i32_0);
                } else {
                }
            }
            s_i32_0 = l5;
            s_i32_1 = l6;
            s_i32_0 = ops::i32_lt_u(s_i32_0, s_i32_1);
            'blk_5: {
                if s_i32_0 != 0 {
                    s_i32_0 = 0i32;
                    l8 = s_i32_0;
                    s_i32_0 = l8;
                    return Ok(s_i32_0);
                } else {
                }
            }
            continue 'blk_3;
            break 'blk_3;
        }
    }
    s_i32_0 = 1i32;
    l9 = s_i32_0;
    s_i32_0 = l9;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_73a866edda27(vm: &mut SubstituteVm, p0: i64, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i64, WasmTrap> {
    let mut l0: i64 = p0;
    let mut l1: i32 = p1;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    vm.deduct(host, 1)?;
    s_i32_0 = l1;
    s_i32_1 = 64i32;
    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i64_0 = l0;
    s_i32_1 = l1;
    s_i64_1 = ops::i64_extend_i32_u(s_i32_1);
    s_i64_0 = ops::i64_shr_u(s_i64_0, s_i64_1);
    return Ok(s_i64_0);
    }

    pub(crate) fn s_745aa11cfb74(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i64 = 0i64;
    let mut l3: i64 = 0i64;
    let mut l4: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
    l2 = s_i64_0;
    s_i32_0 = l1;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
    l3 = s_i64_0;
    s_i64_0 = l2;
    s_i64_1 = l3;
    s_i32_0 = ops::i64_ne(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i64_0 = l2;
            s_i64_1 = l3;
            s_i32_0 = ops::i64_lt_s(s_i64_0, s_i64_1);
        } else {
            s_i32_0 = l0;
            s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 0u32)?;
            s_i32_1 = l1;
            s_i64_1 = ops::i64_load(vm, 0, s_i32_1, 0u32)?;
            s_i32_0 = ops::i64_lt_u(s_i64_0, s_i64_1);
        }
    }
    l4 = s_i32_0;
    s_i32_0 = l4;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_74954132e4ef(vm: &mut SubstituteVm, p0: i32, g0: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i64_0 = ops::i64_extend_i32_s(s_i32_0);
    let __t0 = s_i64_0;
    return crate::shared_1::Shared::s_9942fd50e8d2(vm, __t0, g0, host);
    }

    pub(crate) fn s_770d71d6937d(vm: &mut SubstituteVm, p0: i32, g0: usize, g1: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = 0i32;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i32 = 0i32;
    let mut l7: i32 = 0i32;
    let mut l8: i32 = 0i32;
    let mut l9: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    vm.enter_call()?;
    let __call_res = Self::s_51f0c0b4c723(vm, s_i32_0, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            l1 = s_i32_0;
            s_i32_0 = vm.get_i32(g0)?;
            l2 = s_i32_0;
            s_i32_0 = vm.get_i32(g1)?;
            l3 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            l4 = s_i32_0;
            s_i32_0 = l4;
            s_i32_1 = l1;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l4;
            s_i32_1 = l2;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l4;
            s_i32_1 = l3;
            ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l4;
        } else {
            s_i32_0 = 1i32;
            l5 = s_i32_0;
            s_i32_0 = l0;
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_f76e9849d691(vm, s_i32_0, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_0 = __t2;
            l6 = s_i32_0;
            s_i32_0 = 0i32;
            l7 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i32_0 = __t3;
            l8 = s_i32_0;
            s_i32_0 = l8;
            s_i32_1 = l5;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l8;
            s_i32_1 = l6;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l8;
            s_i32_1 = l7;
            ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l8;
        }
    }
    l9 = s_i32_0;
    s_i32_0 = l9;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_7732a9e441f7(vm: &mut SubstituteVm, p0: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut s_i32_0: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_f76e9849d691(vm, s_i32_0, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    let __t1 = s_i32_0;
    return Self::s_6d1067ace5f6(vm, __t1, host);
    }

    pub(crate) fn s_778b8df33b33(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, g1: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_1 = l1;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_c6c34807855c(vm, s_i32_0, s_i32_1, g0, g1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l2 = s_i32_0;
    s_i32_0 = l2;
    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
    s_i32_0 = ops::i32_eqz(s_i32_0);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        } else {
            s_i32_0 = l2;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
        }
    }
    l3 = s_i32_0;
    s_i32_0 = l3;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_7972a1a3c721(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i32_2: i32 = 0i32;
    vm.deduct(host, 1)?;
    s_i32_0 = l0;
    s_i32_1 = 255i32;
    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
    s_i32_1 = l1;
    s_i32_2 = 255i32;
    s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
    return Ok(s_i32_0);
    }

    pub(crate) fn s_7a311ae8b3ee(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i32_2: i32 = 0i32;
    let mut s_i32_3: i32 = 0i32;
    vm.deduct(host, 1)?;
    s_i32_0 = vm.get_i32(g0)?;
    vm.enter_call()?;
    let __call_res = Self::s_40c2979f136e(vm, s_i32_0, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l2 = s_i32_0;
    s_i32_0 = 0i32;
    l3 = s_i32_0;
    'blk_1: {
        'blk_2: loop {
            vm.deduct(host, 1)?;
            s_i32_0 = l3;
            s_i32_1 = l2;
            s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
            if s_i32_0 != 0 {
                break 'blk_1;
            }
            s_i32_0 = vm.get_i32(g0)?;
            s_i32_1 = l3;
            vm.enter_call()?;
            let __call_res = Self::s_6ecb218e2168(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            l4 = s_i32_0;
            s_i32_0 = l4;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
            s_i32_1 = l4;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
            s_i32_2 = l0;
            s_i32_3 = l1;
            vm.enter_call()?;
            let __call_res = Self::s_695eab7c815a(vm, s_i32_0, s_i32_1, s_i32_2, s_i32_3, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_0 = __t2;
            'blk_3: {
                if s_i32_0 != 0 {
                    s_i32_0 = l3;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l3;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l3 = s_i32_0;
            continue 'blk_2;
            break 'blk_2;
        }
    }
    s_i32_0 = -1i32;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_7a74d4a23901(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i64 = 0i64;
    let mut l3: i64 = 0i64;
    let mut l4: i32 = 0i32;
    let mut l5: i64 = 0i64;
    let mut l6: i64 = 0i64;
    let mut l7: i32 = 0i32;
    let mut l8: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i32_2: i32 = 0i32;
    let mut s_i32_3: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l1;
    s_i32_1 = 64i32;
    s_i32_0 = ops::i32_lt_u(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 0u32)?;
            s_i64_1 = 1i64;
            s_i32_2 = l1;
            vm.enter_call()?;
            let __call_res = Self::s_52dcb6d97135(vm, s_i64_1, s_i32_2, host);
            vm.exit_call();
            let __t0 = __call_res?;
            s_i64_1 = __t0;
            s_i64_0 = ops::i64_or(s_i64_0, s_i64_1);
            l2 = s_i64_0;
            s_i32_0 = l0;
            s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
            l3 = s_i64_0;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            l4 = s_i32_0;
            s_i32_0 = l4;
            s_i64_1 = l2;
            ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
            s_i32_0 = l4;
            s_i64_1 = l3;
            ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
            s_i32_0 = l4;
        } else {
            s_i32_0 = l0;
            s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 0u32)?;
            l5 = s_i64_0;
            s_i32_0 = l0;
            s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
            s_i64_1 = 1i64;
            s_i32_2 = l1;
            s_i32_3 = 64i32;
            vm.enter_call()?;
            let __call_res = Self::s_304f22c4ab8c(vm, s_i32_2, s_i32_3, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_2 = __t2;
            vm.enter_call()?;
            let __call_res = Self::s_52dcb6d97135(vm, s_i64_1, s_i32_2, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i64_1 = __t3;
            s_i64_0 = ops::i64_or(s_i64_0, s_i64_1);
            l6 = s_i64_0;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t4 = __call_res?;
            s_i32_0 = __t4;
            l7 = s_i32_0;
            s_i32_0 = l7;
            s_i64_1 = l5;
            ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
            s_i32_0 = l7;
            s_i64_1 = l6;
            ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
            s_i32_0 = l7;
        }
    }
    l8 = s_i32_0;
    s_i32_0 = l8;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_7b03233bc608(vm: &mut SubstituteVm, p0: i64, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i64, WasmTrap> {
    let mut l0: i64 = p0;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i64_1: i64 = 0i64;
    vm.deduct(host, 1)?;
    s_i64_0 = l0;
    s_i64_1 = 0i64;
    s_i32_0 = ops::i64_lt_s(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i64_0 = l0;
    return Ok(s_i64_0);
    }

    pub(crate) fn s_7e5ac0c660db(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, g1: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l1;
    s_i32_1 = 128i32;
    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            l2 = s_i32_0;
            s_i32_0 = vm.get_i32(g0)?;
            l3 = s_i32_0;
            s_i32_0 = vm.get_i32(g1)?;
            l4 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t0 = __call_res?;
            s_i32_0 = __t0;
            l5 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = l2;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l5;
            s_i32_1 = l3;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l5;
            s_i32_1 = l4;
            ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l5;
        } else {
            s_i32_0 = 1i32;
            l6 = s_i32_0;
            s_i32_0 = l0;
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_f76e9849d691(vm, s_i32_0, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            s_i32_1 = l1;
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_fa6af9f06252(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_0 = __t2;
            vm.enter_call()?;
            let __call_res = Self::s_623491ac0af2(vm, s_i32_0, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i32_0 = __t3;
            l7 = s_i32_0;
            s_i32_0 = 0i32;
            l8 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t4 = __call_res?;
            s_i32_0 = __t4;
            l9 = s_i32_0;
            s_i32_0 = l9;
            s_i32_1 = l6;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l9;
            s_i32_1 = l7;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l9;
            s_i32_1 = l8;
            ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l9;
        }
    }
    l10 = s_i32_0;
    s_i32_0 = l10;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_7fca186a8986(vm: &mut SubstituteVm, p0: i32, g0: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = 0i32;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i32 = 0i32;
    let mut l7: i32 = 0i32;
    let mut l8: i32 = 0i32;
    let mut l9: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
    s_i64_1 = 0i64;
    s_i32_0 = ops::i64_eq(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 0u32)?;
            s_i64_1 = 2147483647i64;
            s_i32_0 = ops::i64_le_u(s_i64_0, s_i64_1);
        } else {
            s_i32_0 = 0i32;
        }
    }
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
            l1 = s_i32_0;
            s_i32_0 = l0;
            s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 0u32)?;
            vm.enter_call()?;
            let __call_res = Self::s_34c286d1358b(vm, s_i64_0, host);
            vm.exit_call();
            let __t0 = __call_res?;
            s_i32_0 = __t0;
            l2 = s_i32_0;
            s_i32_0 = 0i32;
            l3 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            l4 = s_i32_0;
            s_i32_0 = l4;
            s_i32_1 = l1;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l4;
            s_i32_1 = l2;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l4;
            s_i32_1 = l3;
            ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l4;
        } else {
            s_i32_0 = 0i32;
            l5 = s_i32_0;
            s_i32_0 = 0i32;
            l6 = s_i32_0;
            s_i32_0 = vm.get_i32(g0)?;
            l7 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_0 = __t2;
            l8 = s_i32_0;
            s_i32_0 = l8;
            s_i32_1 = l5;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l8;
            s_i32_1 = l6;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l8;
            s_i32_1 = l7;
            ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l8;
        }
    }
    l9 = s_i32_0;
    s_i32_0 = l9;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_7ff1ca352828(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_1 = l1;
    vm.enter_call()?;
    let __call_res = Self::s_14dd5b913b03(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    s_i32_0 = ops::i32_eqz(s_i32_0);
    l2 = s_i32_0;
    s_i32_0 = l2;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn s_801c67a6e55b(vm: &mut SubstituteVm, p0: i64, g0: usize, g1: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i64 = p0;
    let mut l1: i32 = 0i32;
    let mut l2: i64 = 0i64;
    let mut l3: i64 = 0i64;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i32 = 0i32;
    let mut l7: i32 = 0i32;
    let mut l8: i32 = 0i32;
    let mut l9: i32 = 0i32;
    let mut l10: i32 = 0i32;
    let mut l11: i32 = 0i32;
    let mut l12: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i64_0 = l0;
    s_i64_1 = 0i64;
    s_i32_0 = ops::i64_ge_s(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
            l1 = s_i32_0;
            s_i64_0 = l0;
            vm.enter_call()?;
            let __call_res = Self::s_7b03233bc608(vm, s_i64_0, host);
            vm.exit_call();
            let __t0 = __call_res?;
            s_i64_0 = __t0;
            l2 = s_i64_0;
            s_i64_0 = 0i64;
            l3 = s_i64_0;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            l4 = s_i32_0;
            s_i32_0 = l4;
            s_i64_1 = l2;
            ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
            s_i32_0 = l4;
            s_i64_1 = l3;
            ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
            s_i32_0 = l4;
            l5 = s_i32_0;
            s_i32_0 = 0i32;
            l6 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_0 = __t2;
            l7 = s_i32_0;
            s_i32_0 = l7;
            s_i32_1 = l1;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l7;
            s_i32_1 = l5;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l7;
            s_i32_1 = l6;
            ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l7;
        } else {
            s_i32_0 = 0i32;
            l8 = s_i32_0;
            s_i32_0 = vm.get_i32(g0)?;
            l9 = s_i32_0;
            s_i32_0 = vm.get_i32(g1)?;
            l10 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i32_0 = __t3;
            l11 = s_i32_0;
            s_i32_0 = l11;
            s_i32_1 = l8;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l11;
            s_i32_1 = l9;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l11;
            s_i32_1 = l10;
            ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l11;
        }
    }
    l12 = s_i32_0;
    s_i32_0 = l12;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_82d03d870b76(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, g1: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_1 = l1;
    vm.enter_call()?;
    let __call_res = crate::shared_1::Shared::s_86188d572530(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
            l2 = s_i32_0;
            s_i32_0 = l0;
            s_i32_1 = l1;
            vm.enter_call()?;
            let __call_res = crate::shared_1::Shared::s_8cfc64e181cb(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            l3 = s_i32_0;
            s_i32_0 = 0i32;
            l4 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_0 = __t2;
            l5 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = l2;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l5;
            s_i32_1 = l3;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l5;
            s_i32_1 = l4;
            ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l5;
        } else {
            s_i32_0 = 0i32;
            l6 = s_i32_0;
            s_i32_0 = vm.get_i32(g0)?;
            l7 = s_i32_0;
            s_i32_0 = vm.get_i32(g1)?;
            l8 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = Self::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i32_0 = __t3;
            l9 = s_i32_0;
            s_i32_0 = l9;
            s_i32_1 = l6;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l9;
            s_i32_1 = l7;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l9;
            s_i32_1 = l8;
            ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l9;
        }
    }
    l10 = s_i32_0;
    s_i32_0 = l10;
    return Ok(s_i32_0);
    }

}
