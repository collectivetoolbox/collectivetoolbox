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

//! Generated shared Merkle-deduplicated WASM substitute functions (shard 1).
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
    pub(crate) fn s_837205300e16(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l1;
    s_i32_1 = l0;
    let __t0 = s_i32_0;
    let __t1 = s_i32_1;
    return crate::shared_0::Shared::s_559745ed6085(vm, __t0, __t1, host);
    }

    pub(crate) fn s_84059c53f331(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
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
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 0u32)?;
    s_i32_1 = l1;
    s_i64_1 = ops::i64_load(vm, 0, s_i32_1, 0u32)?;
    s_i32_0 = ops::i64_eq(s_i64_0, s_i64_1);
    s_i32_0 = ops::i32_eqz(s_i32_0);
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l0;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
    s_i32_1 = l1;
    s_i64_1 = ops::i64_load(vm, 0, s_i32_1, 8u32)?;
    s_i32_0 = ops::i64_eq(s_i64_0, s_i64_1);
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

    pub(crate) fn s_846bfd421390(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, g1: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let __call_res = crate::shared_0::Shared::s_259f493e8e91(vm, s_i32_0, s_i32_1, host);
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
            let __call_res = Self::s_d0df0f4c4018(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            l3 = s_i32_0;
            s_i32_0 = 0i32;
            l4 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
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
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
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

    pub(crate) fn s_848c8312f309(vm: &mut SubstituteVm, p0: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut s_i32_0: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    vm.enter_call()?;
    let __call_res = Self::s_f76e9849d691(vm, s_i32_0, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    let __t1 = s_i32_0;
    return Self::s_9ebceca0bdb5(vm, __t1, host);
    }

    #[inline(never)]
    pub(crate) fn s_86188d572530(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i64 = 0i64;
    let mut l3: i64 = 0i64;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i32 = 0i32;
    let mut l7: i32 = 0i32;
    let mut l8: i64 = 0i64;
    let mut l9: i32 = 0i32;
    let mut l10: i64 = 0i64;
    let mut l11: i64 = 0i64;
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
    s_i32_0 = l0;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
    l2 = s_i64_0;
    s_i32_0 = l1;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
    l3 = s_i64_0;
    s_i64_0 = l2;
    s_i64_1 = 0i64;
    s_i32_0 = ops::i64_ne(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i64_0 = l3;
            s_i64_1 = 0i64;
            s_i32_0 = ops::i64_ne(s_i64_0, s_i64_1);
        } else {
            s_i32_0 = 0i32;
        }
    }
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            l4 = s_i32_0;
            s_i32_0 = l4;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i32_0 = l0;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 0u32)?;
    s_i32_1 = l1;
    s_i64_1 = ops::i64_load(vm, 0, s_i32_1, 0u32)?;
    vm.enter_call()?;
    let __call_res = Self::s_fbbf699c9d44(vm, s_i64_0, s_i64_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l5 = s_i32_0;
    s_i32_0 = l0;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 0u32)?;
    s_i64_1 = l3;
    vm.enter_call()?;
    let __call_res = Self::s_fbbf699c9d44(vm, s_i64_0, s_i64_1, host);
    vm.exit_call();
    let __t1 = __call_res?;
    s_i32_0 = __t1;
    l6 = s_i32_0;
    s_i64_0 = l2;
    s_i32_1 = l1;
    s_i64_1 = ops::i64_load(vm, 0, s_i32_1, 0u32)?;
    vm.enter_call()?;
    let __call_res = Self::s_fbbf699c9d44(vm, s_i64_0, s_i64_1, host);
    vm.exit_call();
    let __t2 = __call_res?;
    s_i32_0 = __t2;
    l7 = s_i32_0;
    s_i32_0 = l6;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
    s_i32_1 = l7;
    s_i64_1 = ops::i64_load(vm, 0, s_i32_1, 8u32)?;
    s_i64_0 = ops::i64_or(s_i64_0, s_i64_1);
    l8 = s_i64_0;
    s_i64_0 = l8;
    s_i64_1 = 0i64;
    s_i32_0 = ops::i64_ne(s_i64_0, s_i64_1);
    'blk_3: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            l9 = s_i32_0;
            s_i32_0 = l9;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i32_0 = l6;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 0u32)?;
    s_i32_1 = l7;
    s_i64_1 = ops::i64_load(vm, 0, s_i32_1, 0u32)?;
    s_i64_0 = ops::i64_or(s_i64_0, s_i64_1);
    l10 = s_i64_0;
    s_i32_0 = l5;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
    s_i64_1 = l10;
    s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
    l11 = s_i64_0;
    s_i64_0 = l11;
    s_i32_1 = l5;
    s_i64_1 = ops::i64_load(vm, 0, s_i32_1, 8u32)?;
    s_i32_0 = ops::i64_ge_u(s_i64_0, s_i64_1);
    l12 = s_i32_0;
    s_i32_0 = l12;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_862b469aad5b(vm: &mut SubstituteVm, p0: i64, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i64 = p0;
    let mut l1: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i64_1: i64 = 0i64;
    vm.deduct(host, 1)?;
    s_i64_0 = l0;
    s_i32_0 = ops::i32_wrap_i64(s_i64_0);
    l1 = s_i32_0;
    s_i32_0 = l1;
    s_i64_0 = ops::i64_extend_i32_s(s_i32_0);
    s_i64_1 = l0;
    s_i32_0 = ops::i64_ne(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = l1;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn s_86622d9568ec(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i32_2: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l1;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
    s_i64_1 = 5i64;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i64_0 = __t0;
    vm.enter_call()?;
    let __call_res = Self::s_c5da14f612f0(vm, s_i64_0, host);
    vm.exit_call();
    let __t1 = __call_res?;
    s_i32_0 = __t1;
    l2 = s_i32_0;
    s_i32_0 = l2;
    s_i32_1 = 260i32;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_1232b68aede8(vm, s_i32_1, host);
    vm.exit_call();
    let __t2 = __call_res?;
    s_i32_1 = __t2;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_697cb42b9ce7(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l2;
    s_i32_1 = 262i32;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_1232b68aede8(vm, s_i32_1, host);
    vm.exit_call();
    let __t3 = __call_res?;
    s_i32_1 = __t3;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_697cb42b9ce7(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l2;
    s_i32_1 = l0;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_697cb42b9ce7(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l2;
    s_i32_1 = 263i32;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_1232b68aede8(vm, s_i32_1, host);
    vm.exit_call();
    let __t4 = __call_res?;
    s_i32_1 = __t4;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_697cb42b9ce7(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    __call_res?;
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
            s_i32_2 = 4i32;
            s_i32_1 = ops::i32_mul(s_i32_1, s_i32_2);
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
            l7 = s_i32_0;
            'blk_3: {
                s_i32_0 = l7;
                vm.enter_call()?;
                let __call_res = crate::shared_0::Shared::s_62439e6f6ae7(vm, s_i32_0, g0, host);
                vm.exit_call();
                let __t5 = __call_res?;
                s_i32_0 = __t5;
                l8 = s_i32_0;
                s_i32_0 = l8;
                s_i32_1 = 261i32;
                s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                'blk_4: {
                    if s_i32_0 != 0 {
                        s_i32_0 = 1i32;
                    } else {
                        s_i32_0 = l8;
                        s_i32_1 = 255i32;
                        s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                    }
                }
                'blk_5: {
                    if s_i32_0 != 0 {
                        s_i32_0 = l2;
                        s_i32_1 = 255i32;
                        vm.enter_call()?;
                        let __call_res = crate::shared_0::Shared::s_1232b68aede8(vm, s_i32_1, host);
                        vm.exit_call();
                        let __t6 = __call_res?;
                        s_i32_1 = __t6;
                        vm.enter_call()?;
                        let __call_res = crate::shared_0::Shared::s_697cb42b9ce7(vm, s_i32_0, s_i32_1, host);
                        vm.exit_call();
                        __call_res?;
                    } else {
                    }
                }
                s_i32_0 = l2;
                s_i32_1 = l7;
                vm.enter_call()?;
                let __call_res = crate::shared_0::Shared::s_697cb42b9ce7(vm, s_i32_0, s_i32_1, host);
                vm.exit_call();
                __call_res?;
            }
            s_i32_0 = l6;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l6 = s_i32_0;
            continue 'blk_2;
            break 'blk_2;
        }
    }
    s_i32_0 = l2;
    s_i32_1 = 261i32;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_1232b68aede8(vm, s_i32_1, host);
    vm.exit_call();
    let __t7 = __call_res?;
    s_i32_1 = __t7;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_697cb42b9ce7(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l2;
    vm.enter_call()?;
    let __call_res = Self::s_df3679579582(vm, s_i32_0, host);
    vm.exit_call();
    let __t8 = __call_res?;
    s_i32_0 = __t8;
    l9 = s_i32_0;
    s_i32_0 = l9;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_86635ee877c5(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i32_2: i32 = 0i32;
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
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    l2 = s_i32_0;
    s_i32_0 = l2;
    s_i32_1 = l1;
    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    l3 = s_i32_0;
    s_i32_0 = l1;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    l4 = s_i32_0;
    s_i32_0 = 0i32;
    l5 = s_i32_0;
    'blk_3: {
        'blk_4: loop {
            vm.deduct(host, 1)?;
            s_i32_0 = l5;
            s_i32_1 = l2;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            if s_i32_0 != 0 {
                break 'blk_3;
            }
            s_i32_0 = l3;
            s_i32_1 = l5;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
            s_i32_1 = l4;
            s_i32_2 = l5;
            s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
            s_i32_1 = ops::i32_load8_u(vm, 0, s_i32_1, 0u32)?;
            s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
            'blk_5: {
                if s_i32_0 != 0 {
                    s_i32_0 = 0i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l5;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l5 = s_i32_0;
            continue 'blk_4;
            break 'blk_4;
        }
    }
    s_i32_0 = 1i32;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn s_86b08f7a2a43(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i64 = 0i64;
    let mut l3: i64 = 0i64;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i64 = 0i64;
    let mut l7: i32 = 0i32;
    let mut l8: i32 = 0i32;
    let mut l9: i32 = 0i32;
    let mut l10: i32 = 0i32;
    let mut l11: i32 = 0i32;
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
    s_i32_0 = l1;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
    l3 = s_i64_0;
    s_i64_0 = l2;
    s_i64_1 = l3;
    s_i32_0 = ops::i64_lt_u(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = -1i32;
            l4 = s_i32_0;
            s_i32_0 = l4;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i64_0 = l2;
    s_i64_1 = l3;
    s_i32_0 = ops::i64_gt_u(s_i64_0, s_i64_1);
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
            l5 = s_i32_0;
            s_i32_0 = l5;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i64_0 = l2;
    l6 = s_i64_0;
    'blk_3: {
        'blk_4: loop {
            vm.deduct(host, 1)?;
            s_i64_0 = l6;
            s_i64_1 = 0i64;
            s_i32_0 = ops::i64_gt_u(s_i64_0, s_i64_1);
            s_i32_0 = ops::i32_eqz(s_i32_0);
            if s_i32_0 != 0 {
                break 'blk_3;
            }
            s_i64_0 = l6;
            s_i64_1 = 1i64;
            vm.enter_call()?;
            let __call_res = Self::s_aa4577c560e8(vm, s_i64_0, s_i64_1, host);
            vm.exit_call();
            let __t0 = __call_res?;
            s_i64_0 = __t0;
            l6 = s_i64_0;
            s_i32_0 = l0;
            s_i64_1 = l6;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            l7 = s_i32_0;
            s_i32_0 = l1;
            s_i64_1 = l6;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_0 = __t2;
            l8 = s_i32_0;
            s_i32_0 = l7;
            s_i32_1 = l8;
            s_i32_0 = ops::i32_lt_u(s_i32_0, s_i32_1);
            'blk_5: {
                if s_i32_0 != 0 {
                    s_i32_0 = -1i32;
                    l9 = s_i32_0;
                    s_i32_0 = l9;
                    return Ok(s_i32_0);
                } else {
                }
            }
            s_i32_0 = l7;
            s_i32_1 = l8;
            s_i32_0 = ops::i32_gt_u(s_i32_0, s_i32_1);
            'blk_6: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                    l10 = s_i32_0;
                    s_i32_0 = l10;
                    return Ok(s_i32_0);
                } else {
                }
            }
            continue 'blk_4;
            break 'blk_4;
        }
    }
    s_i32_0 = 0i32;
    l11 = s_i32_0;
    s_i32_0 = l11;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_8a79d3bd42f3(vm: &mut SubstituteVm, p0: i32, g0: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let __call_res = crate::shared_0::Shared::s_3ac41cc6c31c(vm, s_i32_0, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 0u32)?;
            s_i64_1 = 2228224i64;
            vm.enter_call()?;
            let __call_res = Self::s_aa4577c560e8(vm, s_i64_0, s_i64_1, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i64_0 = __t1;
            vm.enter_call()?;
            let __call_res = Self::s_d5f50d7e651d(vm, s_i64_0, host);
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

    pub(crate) fn s_8b082d5ac52c(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    s_i32_0 = ops::i32_lt_u(s_i32_0, s_i32_1);
    return Ok(s_i32_0);
    }

    pub(crate) fn s_8bb0e1cac787(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, g1: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let __call_res = Self::s_accc7a26e8a3(vm, s_i32_0, s_i32_1, host);
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
            let __call_res = Self::s_a6d2220b3671(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            l3 = s_i32_0;
            s_i32_0 = 0i32;
            l4 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
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
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
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

    pub(crate) fn s_8cfc64e181cb(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut l3: i64 = 0i64;
    let mut l4: i64 = 0i64;
    let mut l5: i64 = 0i64;
    let mut l6: i32 = 0i32;
    let mut l7: i32 = 0i32;
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
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 0u32)?;
    s_i32_1 = l1;
    s_i64_1 = ops::i64_load(vm, 0, s_i32_1, 0u32)?;
    vm.enter_call()?;
    let __call_res = Self::s_fbbf699c9d44(vm, s_i64_0, s_i64_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l2 = s_i32_0;
    s_i32_0 = l0;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 0u32)?;
    s_i32_1 = l1;
    s_i64_1 = ops::i64_load(vm, 0, s_i32_1, 8u32)?;
    s_i64_0 = ops::i64_mul(s_i64_0, s_i64_1);
    s_i32_1 = l0;
    s_i64_1 = ops::i64_load(vm, 0, s_i32_1, 8u32)?;
    s_i32_2 = l1;
    s_i64_2 = ops::i64_load(vm, 0, s_i32_2, 0u32)?;
    s_i64_1 = ops::i64_mul(s_i64_1, s_i64_2);
    s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
    l3 = s_i64_0;
    s_i32_0 = l2;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 0u32)?;
    l4 = s_i64_0;
    s_i32_0 = l2;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
    s_i64_1 = l3;
    s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
    l5 = s_i64_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
    vm.exit_call();
    let __t1 = __call_res?;
    s_i32_0 = __t1;
    l6 = s_i32_0;
    s_i32_0 = l6;
    s_i64_1 = l4;
    ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
    s_i32_0 = l6;
    s_i64_1 = l5;
    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
    s_i32_0 = l6;
    l7 = s_i32_0;
    s_i32_0 = l7;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_8dcb06005c72(vm: &mut SubstituteVm, p0: i64, p1: i64, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i64, WasmTrap> {
    let mut l0: i64 = p0;
    let mut l1: i64 = p1;
    let mut l2: i64 = 0i64;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i64_1: i64 = 0i64;
    vm.deduct(host, 1)?;
    s_i64_0 = l0;
    s_i64_1 = l1;
    s_i64_0 = ops::i64_mul(s_i64_0, s_i64_1);
    l2 = s_i64_0;
    s_i64_0 = l0;
    s_i64_1 = 0i64;
    s_i32_0 = ops::i64_ne(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i64_0 = l2;
            s_i64_1 = l0;
            s_i64_0 = ops::i64_div_u(s_i64_0, s_i64_1)?;
            s_i64_1 = l1;
            s_i32_0 = ops::i64_ne(s_i64_0, s_i64_1);
            'blk_2: {
                if s_i32_0 != 0 {
                    return Err(WasmTrap::Unreachable);
                }
            }
        }
    }
    s_i64_0 = l2;
    return Ok(s_i64_0);
    }

    pub(crate) fn s_91b7532ed2b6(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    s_i32_0 = l0;
    s_i32_1 = l1;
    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
    l2 = s_i32_0;
    s_i32_0 = l2;
    s_i32_1 = l0;
    s_i32_0 = ops::i32_lt_u(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = l2;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_91c5b1d25086(vm: &mut SubstituteVm, p0: i32, g0: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_1 = 255i32;
    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
    let __t0 = s_i32_0;
    return Self::s_f71eec8c32bf(vm, __t0, g0, host);
    }

    pub(crate) fn s_91e1eeb1b0ab(vm: &mut SubstituteVm, p0: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = 0i32;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = 0i32;
    l1 = s_i32_0;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    l2 = s_i32_0;
    s_i32_0 = 8i32;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l3 = s_i32_0;
    s_i32_0 = l3;
    s_i32_1 = l1;
    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l3;
    s_i32_1 = l2;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l3;
    l4 = s_i32_0;
    s_i32_0 = l4;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_94ee13bac3fe(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i64, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i64 = 0i64;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i32_2: i32 = 0i32;
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
            s_i32_1 = l1;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_73a866edda27(vm, s_i64_0, s_i32_1, host);
            vm.exit_call();
            let __t0 = __call_res?;
            s_i64_0 = __t0;
            s_i64_1 = 1i64;
            s_i64_0 = ops::i64_and(s_i64_0, s_i64_1);
        } else {
            s_i32_0 = l0;
            s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
            s_i32_1 = l1;
            s_i32_2 = 64i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_304f22c4ab8c(vm, s_i32_1, s_i32_2, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_1 = __t1;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_73a866edda27(vm, s_i64_0, s_i32_1, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i64_0 = __t2;
            s_i64_1 = 1i64;
            s_i64_0 = ops::i64_and(s_i64_0, s_i64_1);
        }
    }
    l2 = s_i64_0;
    s_i64_0 = l2;
    return Ok(s_i64_0);
    }

    pub(crate) fn s_94f7298892ed(vm: &mut SubstituteVm, p0: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    l1 = s_i32_0;
    s_i32_0 = l1;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_958e9a31e156(vm: &mut SubstituteVm, p0: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
    l1 = s_i64_0;
    s_i64_0 = l1;
    s_i64_1 = 0i64;
    s_i32_0 = ops::i64_ne(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i64_0 = l1;
            s_i64_0 = ops::i64_clz(s_i64_0);
            s_i32_0 = ops::i32_wrap_i64(s_i64_0);
        } else {
            s_i32_0 = 64i32;
            s_i32_1 = l0;
            s_i64_1 = ops::i64_load(vm, 0, s_i32_1, 0u32)?;
            s_i64_1 = ops::i64_clz(s_i64_1);
            s_i32_1 = ops::i32_wrap_i64(s_i64_1);
            vm.enter_call()?;
            let __call_res = Self::s_91b7532ed2b6(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t0 = __call_res?;
            s_i32_0 = __t0;
        }
    }
    l2 = s_i32_0;
    s_i32_0 = l2;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_95ed5cb908ad(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i64 = 0i64;
    let mut l3: i64 = 0i64;
    let mut l4: i64 = 0i64;
    let mut l5: i64 = 0i64;
    let mut l6: i64 = 0i64;
    let mut l7: i32 = 0i32;
    let mut l8: i32 = 0i32;
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
    s_i64_0 = ops::i64_sub(s_i64_0, s_i64_1);
    l2 = s_i64_0;
    s_i32_0 = l0;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 0u32)?;
    s_i32_1 = l1;
    s_i64_1 = ops::i64_load(vm, 0, s_i32_1, 0u32)?;
    s_i32_0 = ops::i64_lt_u(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i64_0 = 1i64;
        } else {
            s_i64_0 = 0i64;
        }
    }
    l3 = s_i64_0;
    s_i32_0 = l0;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
    s_i32_1 = l1;
    s_i64_1 = ops::i64_load(vm, 0, s_i32_1, 8u32)?;
    s_i64_0 = ops::i64_sub(s_i64_0, s_i64_1);
    s_i64_1 = l3;
    s_i64_0 = ops::i64_sub(s_i64_0, s_i64_1);
    l4 = s_i64_0;
    s_i64_0 = l2;
    l5 = s_i64_0;
    s_i64_0 = l4;
    l6 = s_i64_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l7 = s_i32_0;
    s_i32_0 = l7;
    s_i64_1 = l5;
    ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
    s_i32_0 = l7;
    s_i64_1 = l6;
    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
    s_i32_0 = l7;
    l8 = s_i32_0;
    s_i32_0 = l8;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_95ed62e31677(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, g1: usize, g2: usize, g3: usize, g4: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let __call_res = Self::s_dbc761d7befd(vm, s_i32_0, s_i32_1, g0, g1, g2, g3, g4, host);
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

    pub(crate) fn s_9942fd50e8d2(vm: &mut SubstituteVm, p0: i64, g0: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i64 = p0;
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
    s_i64_0 = l0;
    s_i64_1 = 0i64;
    s_i32_0 = ops::i64_lt_s(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i64_0 = 0i64;
            s_i64_1 = l0;
            s_i64_0 = ops::i64_sub(s_i64_0, s_i64_1);
            l1 = s_i64_0;
            s_i64_0 = l1;
            vm.enter_call()?;
            let __call_res = Self::s_ac5f71241803(vm, s_i64_0, g0, host);
            vm.exit_call();
            let __t0 = __call_res?;
            s_i32_0 = __t0;
            l2 = s_i32_0;
            s_i32_0 = 1i32;
            s_i32_1 = l2;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
            let __t1 = s_i32_0;
            let __t2 = s_i32_1;
            return crate::shared_0::Shared::s_482c16ff60d0(vm, __t1, __t2, g0, host);
        } else {
            s_i64_0 = l0;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_7b03233bc608(vm, s_i64_0, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i64_0 = __t3;
            let __t4 = s_i64_0;
            return Self::s_ac5f71241803(vm, __t4, g0, host);
        }
    }
    return Err(WasmTrap::Unreachable);
    }

    pub(crate) fn s_9c006b2946a7(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    s_i64_0 = ops::i64_or(s_i64_0, s_i64_1);
    l2 = s_i64_0;
    s_i32_0 = l0;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
    s_i32_1 = l1;
    s_i64_1 = ops::i64_load(vm, 0, s_i32_1, 8u32)?;
    s_i64_0 = ops::i64_or(s_i64_0, s_i64_1);
    l3 = s_i64_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
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

    pub(crate) fn s_9c3d0ffe567d(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, g1: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let __call_res = crate::shared_0::Shared::s_7e5ac0c660db(vm, s_i32_0, s_i32_1, g0, g1, host);
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

    pub(crate) fn s_9c6c8b521a4a(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i32_2: i32 = 0i32;
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
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    l2 = s_i32_0;
    s_i32_0 = l2;
    s_i32_1 = l1;
    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l2;
    s_i32_1 = 2i32;
    s_i32_0 = ops::i32_shl(s_i32_0, s_i32_1);
    l2 = s_i32_0;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    l3 = s_i32_0;
    s_i32_0 = l1;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    l4 = s_i32_0;
    s_i32_0 = 0i32;
    l5 = s_i32_0;
    'blk_3: {
        'blk_4: loop {
            vm.deduct(host, 1)?;
            s_i32_0 = l5;
            s_i32_1 = l2;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            if s_i32_0 != 0 {
                break 'blk_3;
            }
            s_i32_0 = l3;
            s_i32_1 = l5;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
            s_i32_1 = l4;
            s_i32_2 = l5;
            s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 0u32)?;
            s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
            'blk_5: {
                if s_i32_0 != 0 {
                    s_i32_0 = 0i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l5;
            s_i32_1 = 4i32;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l5 = s_i32_0;
            continue 'blk_4;
            break 'blk_4;
        }
    }
    s_i32_0 = 1i32;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn s_9d5206c8376a(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, g1: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let __call_res = crate::shared_0::Shared::s_21f47e1623f0(vm, s_i32_0, s_i32_1, g0, g1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l2 = s_i32_0;
    s_i32_0 = l2;
    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
    s_i32_0 = ops::i32_eqz(s_i32_0);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = l2;
            l3 = s_i32_0;
            s_i32_0 = l3;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i32_0 = l2;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    l4 = s_i32_0;
    s_i32_0 = l4;
    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
            l5 = s_i32_0;
            s_i32_0 = l4;
            s_i32_1 = l1;
            vm.enter_call()?;
            let __call_res = Self::s_91e1eeb1b0ab(vm, s_i32_1, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_1 = __t1;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_1cbb4a12599c(vm, s_i32_0, s_i32_1, g0, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_0 = __t2;
            l6 = s_i32_0;
            s_i32_0 = 0i32;
            l7 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
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
        } else {
            s_i32_0 = 1i32;
            l9 = s_i32_0;
            s_i32_0 = l4;
            l10 = s_i32_0;
            s_i32_0 = 0i32;
            l11 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t4 = __call_res?;
            s_i32_0 = __t4;
            l12 = s_i32_0;
            s_i32_0 = l12;
            s_i32_1 = l9;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l12;
            s_i32_1 = l10;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l12;
            s_i32_1 = l11;
            ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l12;
        }
    }
    l13 = s_i32_0;
    s_i32_0 = l13;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_9e8b02290b48(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i32_2: i32 = 0i32;
    vm.deduct(host, 1)?;
    s_i32_0 = l0;
    s_i32_1 = l1;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    l2 = s_i32_0;
    s_i32_0 = l2;
    s_i32_1 = l1;
    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
            return Ok(s_i32_0);
        }
    }
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    l3 = s_i32_0;
    s_i32_0 = l1;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    l4 = s_i32_0;
    s_i32_0 = 0i32;
    l5 = s_i32_0;
    'blk_3: {
        'blk_4: loop {
            vm.deduct(host, 1)?;
            s_i32_0 = l5;
            s_i32_1 = l2;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            if s_i32_0 != 0 {
                break 'blk_3;
            }
            s_i32_0 = l3;
            s_i32_1 = l5;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
            s_i32_1 = l4;
            s_i32_2 = l5;
            s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
            s_i32_1 = ops::i32_load8_u(vm, 0, s_i32_1, 0u32)?;
            s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
            'blk_5: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                    return Ok(s_i32_0);
                }
            }
            s_i32_0 = l5;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l5 = s_i32_0;
            continue 'blk_4;
            break 'blk_4;
        }
    }
    s_i32_0 = 0i32;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_9ebceca0bdb5(vm: &mut SubstituteVm, p0: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = 0i32;
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
    s_i64_0 = ops::i64_popcnt(s_i64_0);
    s_i32_0 = ops::i32_wrap_i64(s_i64_0);
    s_i32_1 = l0;
    s_i64_1 = ops::i64_load(vm, 0, s_i32_1, 8u32)?;
    s_i64_1 = ops::i64_popcnt(s_i64_1);
    s_i32_1 = ops::i32_wrap_i64(s_i64_1);
    vm.enter_call()?;
    let __call_res = Self::s_91b7532ed2b6(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l1 = s_i32_0;
    s_i32_0 = l1;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_a024cedc19c6(vm: &mut SubstituteVm, p0: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    s_i32_0 = l0;
    s_i32_1 = 255i32;
    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
    l1 = s_i64_0;
    s_i64_0 = 0i64;
    l2 = s_i64_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
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

    pub(crate) fn s_a173f4736176(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let __call_res = Self::s_84059c53f331(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    s_i32_0 = ops::i32_eqz(s_i32_0);
    l2 = s_i32_0;
    s_i32_0 = l2;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_a30b95af5241(vm: &mut SubstituteVm, p0: i32, g0: usize, g1: usize, g2: usize, g3: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let __call_res = Self::s_ffcc61d1c500(vm, s_i32_0, g0, g1, g2, g3, host);
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
            s_i32_0 = vm.get_i32(g2)?;
            l4 = s_i32_0;
            s_i32_0 = 20i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
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
            return Self::s_b4afe44dab61(vm, __t2, g2, host);
        }
    }
    l6 = s_i32_0;
    s_i32_0 = l6;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_a6d2220b3671(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let __call_res = Self::s_f76e9849d691(vm, s_i32_0, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    s_i32_1 = l1;
    vm.enter_call()?;
    let __call_res = Self::s_f76e9849d691(vm, s_i32_1, host);
    vm.exit_call();
    let __t1 = __call_res?;
    s_i32_1 = __t1;
    vm.enter_call()?;
    let __call_res = Self::s_95ed5cb908ad(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t2 = __call_res?;
    s_i32_0 = __t2;
    let __t3 = s_i32_0;
    return crate::shared_0::Shared::s_623491ac0af2(vm, __t3, host);
    }

    pub(crate) fn s_a810243eab94(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, g1: usize, g2: usize, g3: usize, g4: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let __call_res = Self::s_a82938aed706(vm, s_i32_0, s_i32_1, g0, g1, g2, g3, g4, host);
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
    pub(crate) fn s_a82938aed706(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, g1: usize, g2: usize, g3: usize, g4: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let __call_res = Self::s_84059c53f331(vm, s_i32_0, s_i32_1, host);
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
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
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
    let __call_res = Self::s_84059c53f331(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t2 = __call_res?;
    s_i32_0 = __t2;
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            s_i32_1 = vm.get_i32(g3)?;
            vm.enter_call()?;
            let __call_res = Self::s_84059c53f331(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i32_0 = __t3;
        } else {
            s_i32_0 = 0i32;
        }
    }
    'blk_3: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
            l7 = s_i32_0;
            s_i32_0 = vm.get_i32(g0)?;
            l8 = s_i32_0;
            s_i32_0 = 0i32;
            l9 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
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
    let __call_res = Self::s_f7d297dbbc03(vm, s_i32_0, g4, host);
    vm.exit_call();
    let __t5 = __call_res?;
    s_i32_0 = __t5;
    s_i32_1 = l1;
    vm.enter_call()?;
    let __call_res = Self::s_f7d297dbbc03(vm, s_i32_1, g4, host);
    vm.exit_call();
    let __t6 = __call_res?;
    s_i32_1 = __t6;
    vm.enter_call()?;
    let __call_res = Self::s_bd22d8f9c2fc(vm, s_i32_0, s_i32_1, g4, g1, host);
    vm.exit_call();
    let __t7 = __call_res?;
    s_i32_0 = __t7;
    l12 = s_i32_0;
    s_i32_0 = l12;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 8u32)?;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_623491ac0af2(vm, s_i32_0, host);
    vm.exit_call();
    let __t8 = __call_res?;
    s_i32_0 = __t8;
    l13 = s_i32_0;
    s_i32_0 = l0;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_51f0c0b4c723(vm, s_i32_0, host);
    vm.exit_call();
    let __t9 = __call_res?;
    s_i32_0 = __t9;
    'blk_4: {
        if s_i32_0 != 0 {
            s_i32_0 = l13;
            vm.enter_call()?;
            let __call_res = Self::s_d0a938e2c1d3(vm, s_i32_0, g0, host);
            vm.exit_call();
            let __t10 = __call_res?;
            s_i32_0 = __t10;
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
    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
    vm.exit_call();
    let __t11 = __call_res?;
    s_i32_0 = __t11;
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

    pub(crate) fn s_aa31c70be032(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i32_2: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            let __t0 = s_i32_0;
            return crate::shared_0::Shared::s_1676a7fbbb39(vm, __t0, g0, host);
        } else {
            s_i32_0 = 316i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_1232b68aede8(vm, s_i32_0, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            l2 = s_i32_0;
            s_i32_0 = 20i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_0 = __t2;
            l3 = s_i32_0;
            s_i32_0 = l3;
            s_i32_1 = l3;
            s_i32_2 = 16i32;
            s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l3;
            s_i32_1 = 1i32;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l3;
            s_i32_1 = 1i32;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l3;
            s_i32_1 = 1i32;
            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
            s_i32_0 = l3;
            s_i32_1 = l2;
            ops::i32_store(vm, 0, s_i32_0, 16u32, s_i32_1)?;
            s_i32_0 = l3;
        }
    }
    l4 = s_i32_0;
    s_i32_0 = l4;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_aa4577c560e8(vm: &mut SubstituteVm, p0: i64, p1: i64, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i64, WasmTrap> {
    let mut l0: i64 = p0;
    let mut l1: i64 = p1;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i64_1: i64 = 0i64;
    vm.deduct(host, 1)?;
    s_i64_0 = l0;
    s_i64_1 = l1;
    s_i32_0 = ops::i64_lt_u(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i64_0 = l0;
    s_i64_1 = l1;
    s_i64_0 = ops::i64_sub(s_i64_0, s_i64_1);
    return Ok(s_i64_0);
    }

    #[inline(never)]
    pub(crate) fn s_aa514e5e691b(vm: &mut SubstituteVm, p0: i32, p1: i64, g0: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i64 = p1;
    let mut l2: i64 = 0i64;
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
    let mut l14: i64 = 0i64;
    let mut l15: i32 = 0i32;
    let mut l16: i32 = 0i32;
    let mut l17: i32 = 0i32;
    let mut l18: i32 = 0i32;
    let mut l19: i64 = 0i64;
    let mut l20: i32 = 0i32;
    let mut l21: i32 = 0i32;
    let mut l22: i32 = 0i32;
    let mut l23: i32 = 0i32;
    let mut l24: i32 = 0i32;
    let mut l25: i64 = 0i64;
    let mut l26: i32 = 0i32;
    let mut l27: i32 = 0i32;
    let mut l28: i64 = 0i64;
    let mut l29: i32 = 0i32;
    let mut l30: i32 = 0i32;
    let mut l31: i32 = 0i32;
    let mut l32: i32 = 0i32;
    let mut l33: i64 = 0i64;
    let mut l34: i32 = 0i32;
    let mut l35: i32 = 0i32;
    let mut l36: i32 = 0i32;
    let mut l37: i32 = 0i32;
    let mut l38: i32 = 0i32;
    let mut l39: i32 = 0i32;
    let mut l40: i64 = 0i64;
    let mut l41: i32 = 0i32;
    let mut l42: i32 = 0i32;
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
    s_i64_1 = 5i64;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i64_0 = __t0;
    s_i64_1 = l2;
    s_i32_0 = ops::i64_gt_u(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            l3 = s_i32_0;
            s_i64_0 = 0i64;
            s_i64_1 = 0i64;
            vm.enter_call()?;
            let __call_res = Self::s_b3e6ca84b2de(vm, s_i64_0, s_i64_1, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            l4 = s_i32_0;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_0 = __t2;
            l5 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = l5;
            s_i32_2 = 16i32;
            s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l5;
            s_i32_1 = 0i32;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l5;
            s_i32_1 = 0i32;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l5;
            s_i32_1 = 1i32;
            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
            s_i32_0 = l5;
            l6 = s_i32_0;
            s_i64_0 = l1;
            l7 = s_i64_0;
            s_i32_0 = 24i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i32_0 = __t3;
            l8 = s_i32_0;
            s_i32_0 = l8;
            s_i32_1 = l3;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l8;
            s_i32_1 = l4;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l8;
            s_i32_1 = l6;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l8;
            s_i64_1 = l7;
            ops::i64_store(vm, 0, s_i32_0, 16u32, s_i64_1)?;
            s_i32_0 = l8;
            l9 = s_i32_0;
            s_i32_0 = l9;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i32_0 = l0;
    s_i64_1 = l1;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
    vm.exit_call();
    let __t4 = __call_res?;
    s_i32_0 = __t4;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_62439e6f6ae7(vm, s_i32_0, g0, host);
    vm.exit_call();
    let __t5 = __call_res?;
    s_i32_0 = __t5;
    s_i32_1 = 260i32;
    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
        } else {
            s_i32_0 = l0;
            s_i64_1 = l1;
            s_i64_2 = 1i64;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_1c917789866c(vm, s_i64_1, s_i64_2, host);
            vm.exit_call();
            let __t6 = __call_res?;
            s_i64_1 = __t6;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
            vm.exit_call();
            let __t7 = __call_res?;
            s_i32_0 = __t7;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_62439e6f6ae7(vm, s_i32_0, g0, host);
            vm.exit_call();
            let __t8 = __call_res?;
            s_i32_0 = __t8;
            s_i32_1 = 262i32;
            s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
            'blk_3: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                } else {
                    s_i32_0 = l0;
                    s_i64_1 = l1;
                    s_i64_2 = 3i64;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_1c917789866c(vm, s_i64_1, s_i64_2, host);
                    vm.exit_call();
                    let __t9 = __call_res?;
                    s_i64_1 = __t9;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
                    vm.exit_call();
                    let __t10 = __call_res?;
                    s_i32_0 = __t10;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_62439e6f6ae7(vm, s_i32_0, g0, host);
                    vm.exit_call();
                    let __t11 = __call_res?;
                    s_i32_0 = __t11;
                    s_i32_1 = 263i32;
                    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
                }
            }
        }
    }
    'blk_4: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            l10 = s_i32_0;
            s_i64_0 = 0i64;
            s_i64_1 = 0i64;
            vm.enter_call()?;
            let __call_res = Self::s_b3e6ca84b2de(vm, s_i64_0, s_i64_1, host);
            vm.exit_call();
            let __t12 = __call_res?;
            s_i32_0 = __t12;
            l11 = s_i32_0;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t13 = __call_res?;
            s_i32_0 = __t13;
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
            s_i64_0 = l1;
            l14 = s_i64_0;
            s_i32_0 = 24i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t14 = __call_res?;
            s_i32_0 = __t14;
            l15 = s_i32_0;
            s_i32_0 = l15;
            s_i32_1 = l10;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l15;
            s_i32_1 = l11;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l15;
            s_i32_1 = l13;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l15;
            s_i64_1 = l14;
            ops::i64_store(vm, 0, s_i32_0, 16u32, s_i64_1)?;
            s_i32_0 = l15;
            l16 = s_i32_0;
            s_i32_0 = l16;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i32_0 = l0;
    s_i64_1 = l1;
    s_i64_2 = 2i64;
    s_i64_1 = ops::i64_add(s_i64_1, s_i64_2);
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
    vm.exit_call();
    let __t15 = __call_res?;
    s_i32_0 = __t15;
    l17 = s_i32_0;
    s_i64_0 = l2;
    s_i64_1 = l1;
    s_i64_0 = ops::i64_sub(s_i64_0, s_i64_1);
    vm.enter_call()?;
    let __call_res = Self::s_c5da14f612f0(vm, s_i64_0, host);
    vm.exit_call();
    let __t16 = __call_res?;
    s_i32_0 = __t16;
    l18 = s_i32_0;
    s_i64_0 = l1;
    s_i64_1 = 4i64;
    s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
    l19 = s_i64_0;
    'blk_5: {
        'blk_6: loop {
            vm.deduct(host, 1)?;
            s_i64_0 = l19;
            s_i64_1 = l2;
            s_i32_0 = ops::i64_lt_u(s_i64_0, s_i64_1);
            s_i32_0 = ops::i32_eqz(s_i32_0);
            if s_i32_0 != 0 {
                break 'blk_5;
            }
            s_i32_0 = l0;
            s_i64_1 = l19;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
            vm.exit_call();
            let __t17 = __call_res?;
            s_i32_0 = __t17;
            l20 = s_i32_0;
            s_i32_0 = l20;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_62439e6f6ae7(vm, s_i32_0, g0, host);
            vm.exit_call();
            let __t18 = __call_res?;
            s_i32_0 = __t18;
            l21 = s_i32_0;
            s_i32_0 = l21;
            s_i32_1 = 261i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_7: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                    l22 = s_i32_0;
                    s_i32_0 = l17;
                    l23 = s_i32_0;
                    s_i32_0 = l18;
                    vm.enter_call()?;
                    let __call_res = Self::s_df3679579582(vm, s_i32_0, host);
                    vm.exit_call();
                    let __t19 = __call_res?;
                    s_i32_0 = __t19;
                    l24 = s_i32_0;
                    s_i64_0 = l19;
                    s_i64_1 = 1i64;
                    s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
                    l25 = s_i64_0;
                    s_i32_0 = 24i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                    vm.exit_call();
                    let __t20 = __call_res?;
                    s_i32_0 = __t20;
                    l26 = s_i32_0;
                    s_i32_0 = l26;
                    s_i32_1 = l22;
                    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l26;
                    s_i32_1 = l23;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l26;
                    s_i32_1 = l24;
                    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
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
            s_i32_0 = l21;
            s_i32_1 = 255i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_8: {
                if s_i32_0 != 0 {
                    s_i64_0 = l19;
                    s_i64_1 = 1i64;
                    s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
                    l28 = s_i64_0;
                    s_i64_0 = l28;
                    s_i64_1 = l2;
                    s_i32_0 = ops::i64_ge_u(s_i64_0, s_i64_1);
                    'blk_9: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 0i32;
                            l29 = s_i32_0;
                            s_i64_0 = 0i64;
                            s_i64_1 = 0i64;
                            vm.enter_call()?;
                            let __call_res = Self::s_b3e6ca84b2de(vm, s_i64_0, s_i64_1, host);
                            vm.exit_call();
                            let __t21 = __call_res?;
                            s_i32_0 = __t21;
                            l30 = s_i32_0;
                            s_i32_0 = 16i32;
                            vm.enter_call()?;
                            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                            vm.exit_call();
                            let __t22 = __call_res?;
                            s_i32_0 = __t22;
                            l31 = s_i32_0;
                            s_i32_0 = l31;
                            s_i32_1 = l31;
                            s_i32_2 = 16i32;
                            s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
                            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                            s_i32_0 = l31;
                            s_i32_1 = 0i32;
                            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                            s_i32_0 = l31;
                            s_i32_1 = 0i32;
                            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                            s_i32_0 = l31;
                            s_i32_1 = 1i32;
                            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                            s_i32_0 = l31;
                            l32 = s_i32_0;
                            s_i64_0 = l1;
                            l33 = s_i64_0;
                            s_i32_0 = 24i32;
                            vm.enter_call()?;
                            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                            vm.exit_call();
                            let __t23 = __call_res?;
                            s_i32_0 = __t23;
                            l34 = s_i32_0;
                            s_i32_0 = l34;
                            s_i32_1 = l29;
                            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                            s_i32_0 = l34;
                            s_i32_1 = l30;
                            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                            s_i32_0 = l34;
                            s_i32_1 = l32;
                            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                            s_i32_0 = l34;
                            s_i64_1 = l33;
                            ops::i64_store(vm, 0, s_i32_0, 16u32, s_i64_1)?;
                            s_i32_0 = l34;
                            l35 = s_i32_0;
                            s_i32_0 = l35;
                            return Ok(s_i32_0);
                        } else {
                        }
                    }
                    s_i32_0 = l18;
                    s_i32_1 = l0;
                    s_i64_2 = l28;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_17df4637c35a(vm, s_i32_1, s_i64_2, host);
                    vm.exit_call();
                    let __t24 = __call_res?;
                    s_i32_1 = __t24;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_697cb42b9ce7(vm, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                    s_i64_0 = l19;
                    s_i64_1 = 2i64;
                    s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
                    l19 = s_i64_0;
                } else {
                    s_i32_0 = l18;
                    s_i32_1 = l20;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_697cb42b9ce7(vm, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                    s_i64_0 = l19;
                    s_i64_1 = 1i64;
                    s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
                    l19 = s_i64_0;
                }
            }
            continue 'blk_6;
            break 'blk_6;
        }
    }
    s_i32_0 = 0i32;
    l36 = s_i32_0;
    s_i64_0 = 0i64;
    s_i64_1 = 0i64;
    vm.enter_call()?;
    let __call_res = Self::s_b3e6ca84b2de(vm, s_i64_0, s_i64_1, host);
    vm.exit_call();
    let __t25 = __call_res?;
    s_i32_0 = __t25;
    l37 = s_i32_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
    vm.exit_call();
    let __t26 = __call_res?;
    s_i32_0 = __t26;
    l38 = s_i32_0;
    s_i32_0 = l38;
    s_i32_1 = l38;
    s_i32_2 = 16i32;
    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l38;
    s_i32_1 = 0i32;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l38;
    s_i32_1 = 0i32;
    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
    s_i32_0 = l38;
    s_i32_1 = 1i32;
    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
    s_i32_0 = l38;
    l39 = s_i32_0;
    s_i64_0 = l1;
    l40 = s_i64_0;
    s_i32_0 = 24i32;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
    vm.exit_call();
    let __t27 = __call_res?;
    s_i32_0 = __t27;
    l41 = s_i32_0;
    s_i32_0 = l41;
    s_i32_1 = l36;
    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l41;
    s_i32_1 = l37;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l41;
    s_i32_1 = l39;
    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
    s_i32_0 = l41;
    s_i64_1 = l40;
    ops::i64_store(vm, 0, s_i32_0, 16u32, s_i64_1)?;
    s_i32_0 = l41;
    l42 = s_i32_0;
    s_i32_0 = l42;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_ab8268aad544(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, g1: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i64_2: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_1 = vm.get_i32(g0)?;
    vm.enter_call()?;
    let __call_res = Self::s_84059c53f331(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
        } else {
            s_i32_0 = l1;
            s_i32_1 = vm.get_i32(g0)?;
            vm.enter_call()?;
            let __call_res = Self::s_84059c53f331(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
        }
    }
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
            l2 = s_i32_0;
            s_i32_0 = l2;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i32_0 = l0;
    vm.enter_call()?;
    let __call_res = Self::s_f7d297dbbc03(vm, s_i32_0, g1, host);
    vm.exit_call();
    let __t2 = __call_res?;
    s_i32_0 = __t2;
    l3 = s_i32_0;
    s_i32_0 = l1;
    vm.enter_call()?;
    let __call_res = Self::s_f7d297dbbc03(vm, s_i32_0, g1, host);
    vm.exit_call();
    let __t3 = __call_res?;
    s_i32_0 = __t3;
    l4 = s_i32_0;
    s_i32_0 = l3;
    s_i32_1 = l4;
    vm.enter_call()?;
    let __call_res = Self::s_86188d572530(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t4 = __call_res?;
    s_i32_0 = __t4;
    s_i32_0 = ops::i32_eqz(s_i32_0);
    'blk_3: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            l5 = s_i32_0;
            s_i32_0 = l5;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i32_0 = l3;
    s_i32_1 = l4;
    vm.enter_call()?;
    let __call_res = Self::s_8cfc64e181cb(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t5 = __call_res?;
    s_i32_0 = __t5;
    l6 = s_i32_0;
    s_i32_0 = l0;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_51f0c0b4c723(vm, s_i32_0, host);
    vm.exit_call();
    let __t6 = __call_res?;
    s_i32_0 = __t6;
    s_i32_1 = l1;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_51f0c0b4c723(vm, s_i32_1, host);
    vm.exit_call();
    let __t7 = __call_res?;
    s_i32_1 = __t7;
    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
    'blk_4: {
        if s_i32_0 != 0 {
            s_i32_0 = l6;
            s_i64_1 = 0i64;
            s_i64_2 = i64::MIN;
            vm.enter_call()?;
            let __call_res = Self::s_b3e6ca84b2de(vm, s_i64_1, s_i64_2, host);
            vm.exit_call();
            let __t8 = __call_res?;
            s_i32_1 = __t8;
            let __t9 = s_i32_0;
            let __t10 = s_i32_1;
            return Self::s_f507e9fb18e1(vm, __t9, __t10, host);
        } else {
            s_i32_0 = l6;
            s_i64_1 = -1i64;
            s_i64_2 = 9223372036854775807i64;
            vm.enter_call()?;
            let __call_res = Self::s_b3e6ca84b2de(vm, s_i64_1, s_i64_2, host);
            vm.exit_call();
            let __t11 = __call_res?;
            s_i32_1 = __t11;
            let __t12 = s_i32_0;
            let __t13 = s_i32_1;
            return Self::s_f507e9fb18e1(vm, __t12, __t13, host);
        }
    }
    return Err(WasmTrap::Unreachable);
    }

    pub(crate) fn s_abb2b8d6397f(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i32_2: i32 = 0i32;
    vm.deduct(host, 1)?;
    s_i32_0 = l0;
    s_i32_1 = l1;
    s_i32_0 = ops::i32_sub(s_i32_0, s_i32_1);
    l2 = s_i32_0;
    s_i32_0 = l0;
    s_i32_1 = l1;
    s_i32_0 = ops::i32_xor(s_i32_0, s_i32_1);
    s_i32_1 = l2;
    s_i32_2 = l0;
    s_i32_1 = ops::i32_xor(s_i32_1, s_i32_2);
    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
    s_i32_1 = 0i32;
    s_i32_0 = ops::i32_lt_s(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = l2;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn s_ac5f71241803(vm: &mut SubstituteVm, p0: i64, g0: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i64 = p0;
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
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i32_2: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i64_0 = l0;
    s_i64_1 = 0i64;
    s_i32_0 = ops::i64_eq(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = vm.get_i32(g0)?;
        } else {
            s_i64_0 = l0;
            s_i64_1 = 4294967295i64;
            s_i64_0 = ops::i64_and(s_i64_0, s_i64_1);
            vm.enter_call()?;
            let __call_res = Self::s_d5f50d7e651d(vm, s_i64_0, host);
            vm.exit_call();
            let __t0 = __call_res?;
            s_i32_0 = __t0;
            l1 = s_i32_0;
            s_i64_0 = l0;
            s_i32_1 = 32i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_73a866edda27(vm, s_i64_0, s_i32_1, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i64_0 = __t1;
            vm.enter_call()?;
            let __call_res = Self::s_d5f50d7e651d(vm, s_i64_0, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_0 = __t2;
            l2 = s_i32_0;
            s_i32_0 = l2;
            s_i32_1 = 0i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_2: {
                if s_i32_0 != 0 {
                    s_i32_0 = 0i32;
                    l3 = s_i32_0;
                    s_i32_0 = l1;
                    l4 = s_i32_0;
                    s_i32_0 = 20i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                    vm.exit_call();
                    let __t3 = __call_res?;
                    s_i32_0 = __t3;
                    l5 = s_i32_0;
                    s_i32_0 = l5;
                    s_i32_1 = l5;
                    s_i32_2 = 16i32;
                    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l5;
                    s_i32_1 = 1i32;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l5;
                    s_i32_1 = 1i32;
                    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                    s_i32_0 = l5;
                    s_i32_1 = 1i32;
                    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                    s_i32_0 = l5;
                    s_i32_1 = l4;
                    ops::i32_store(vm, 0, s_i32_0, 16u32, s_i32_1)?;
                    s_i32_0 = l5;
                    l6 = s_i32_0;
                    s_i32_0 = 8i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                    vm.exit_call();
                    let __t4 = __call_res?;
                    s_i32_0 = __t4;
                    l7 = s_i32_0;
                    s_i32_0 = l7;
                    s_i32_1 = l3;
                    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l7;
                    s_i32_1 = l6;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l7;
                } else {
                    s_i32_0 = 0i32;
                    l8 = s_i32_0;
                    s_i32_0 = l1;
                    l9 = s_i32_0;
                    s_i32_0 = l2;
                    l10 = s_i32_0;
                    s_i32_0 = 24i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
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
                    s_i32_1 = 2i32;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l11;
                    s_i32_1 = 2i32;
                    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                    s_i32_0 = l11;
                    s_i32_1 = 1i32;
                    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                    s_i32_0 = l11;
                    s_i32_1 = l9;
                    ops::i32_store(vm, 0, s_i32_0, 16u32, s_i32_1)?;
                    s_i32_0 = l11;
                    s_i32_1 = l10;
                    ops::i32_store(vm, 0, s_i32_0, 20u32, s_i32_1)?;
                    s_i32_0 = l11;
                    l12 = s_i32_0;
                    s_i32_0 = 8i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                    vm.exit_call();
                    let __t6 = __call_res?;
                    s_i32_0 = __t6;
                    l13 = s_i32_0;
                    s_i32_0 = l13;
                    s_i32_1 = l8;
                    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l13;
                    s_i32_1 = l12;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l13;
                }
            }
        }
    }
    l14 = s_i32_0;
    s_i32_0 = l14;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_accc7a26e8a3(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let __call_res = Self::s_a6d2220b3671(vm, s_i32_0, s_i32_1, host);
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
    s_i64_0 = l4;
    s_i64_1 = l5;
    s_i64_0 = ops::i64_xor(s_i64_0, s_i64_1);
    s_i64_1 = l3;
    s_i64_2 = l4;
    s_i64_1 = ops::i64_xor(s_i64_1, s_i64_2);
    s_i64_0 = ops::i64_and(s_i64_0, s_i64_1);
    s_i64_1 = 0i64;
    s_i32_0 = ops::i64_ge_s(s_i64_0, s_i64_1);
    l6 = s_i32_0;
    s_i32_0 = l6;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn s_adfbc42be29b(vm: &mut SubstituteVm, p0: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = 0i32;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i32 = 0i32;
    let mut l7: i64 = 0i64;
    let mut l8: i64 = 0i64;
    let mut l9: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i32_2: i32 = 0i32;
    let mut s_i64_2: i64 = 0i64;
    let mut s_i32_3: i32 = 0i32;
    let mut s_i64_3: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
    s_i64_1 = 4i64;
    s_i64_0 = ops::i64_mul(s_i64_0, s_i64_1);
    s_i64_1 = 64i64;
    s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_61878e7d5ba7(vm, s_i64_0, host);
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
            vm.deduct(host, 1)?;
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
                s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 0u32)?;
                l7 = s_i64_0;
                s_i32_0 = l6;
                s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
                l8 = s_i64_0;
                s_i64_0 = l8;
                s_i64_1 = 0i64;
                s_i32_0 = ops::i64_eq(s_i64_0, s_i64_1);
                'blk_4: {
                    if s_i32_0 != 0 {
                        s_i64_0 = l7;
                        s_i64_1 = 127i64;
                        s_i32_0 = ops::i64_le_u(s_i64_0, s_i64_1);
                        'blk_5: {
                            if s_i32_0 != 0 {
                                s_i32_0 = l1;
                                s_i64_1 = l7;
                                vm.enter_call()?;
                                let __call_res = Self::s_b5672010fcc9(vm, s_i64_1, host);
                                vm.exit_call();
                                let __t1 = __call_res?;
                                s_i32_1 = __t1;
                                vm.enter_call()?;
                                let __call_res = crate::shared_0::Shared::s_5daf123ed54b(vm, s_i32_0, s_i32_1, host);
                                vm.exit_call();
                                __call_res?;
                            } else {
                                s_i64_0 = l7;
                                s_i64_1 = 65536i64;
                                s_i32_0 = ops::i64_ge_u(s_i64_0, s_i64_1);
                                'blk_6: {
                                    if s_i32_0 != 0 {
                                        s_i64_0 = l7;
                                        s_i64_1 = 2097151i64;
                                        s_i32_0 = ops::i64_le_u(s_i64_0, s_i64_1);
                                    } else {
                                        s_i32_0 = 0i32;
                                    }
                                }
                                'blk_7: {
                                    if s_i32_0 != 0 {
                                        s_i32_0 = l1;
                                        s_i32_1 = 240i32;
                                        s_i64_2 = l7;
                                        s_i32_3 = 18i32;
                                        vm.enter_call()?;
                                        let __call_res = crate::shared_0::Shared::s_73a866edda27(vm, s_i64_2, s_i32_3, host);
                                        vm.exit_call();
                                        let __t2 = __call_res?;
                                        s_i64_2 = __t2;
                                        vm.enter_call()?;
                                        let __call_res = Self::s_b5672010fcc9(vm, s_i64_2, host);
                                        vm.exit_call();
                                        let __t3 = __call_res?;
                                        s_i32_2 = __t3;
                                        s_i32_1 = ops::i32_or(s_i32_1, s_i32_2);
                                        s_i32_2 = 255i32;
                                        s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
                                        vm.enter_call()?;
                                        let __call_res = crate::shared_0::Shared::s_5daf123ed54b(vm, s_i32_0, s_i32_1, host);
                                        vm.exit_call();
                                        __call_res?;
                                        s_i32_0 = l1;
                                        s_i32_1 = 128i32;
                                        s_i64_2 = l7;
                                        s_i32_3 = 12i32;
                                        vm.enter_call()?;
                                        let __call_res = crate::shared_0::Shared::s_73a866edda27(vm, s_i64_2, s_i32_3, host);
                                        vm.exit_call();
                                        let __t4 = __call_res?;
                                        s_i64_2 = __t4;
                                        s_i64_3 = 63i64;
                                        s_i64_2 = ops::i64_and(s_i64_2, s_i64_3);
                                        vm.enter_call()?;
                                        let __call_res = Self::s_b5672010fcc9(vm, s_i64_2, host);
                                        vm.exit_call();
                                        let __t5 = __call_res?;
                                        s_i32_2 = __t5;
                                        s_i32_1 = ops::i32_or(s_i32_1, s_i32_2);
                                        s_i32_2 = 255i32;
                                        s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
                                        vm.enter_call()?;
                                        let __call_res = crate::shared_0::Shared::s_5daf123ed54b(vm, s_i32_0, s_i32_1, host);
                                        vm.exit_call();
                                        __call_res?;
                                        s_i32_0 = l1;
                                        s_i32_1 = 128i32;
                                        s_i64_2 = l7;
                                        s_i32_3 = 6i32;
                                        vm.enter_call()?;
                                        let __call_res = crate::shared_0::Shared::s_73a866edda27(vm, s_i64_2, s_i32_3, host);
                                        vm.exit_call();
                                        let __t6 = __call_res?;
                                        s_i64_2 = __t6;
                                        s_i64_3 = 63i64;
                                        s_i64_2 = ops::i64_and(s_i64_2, s_i64_3);
                                        vm.enter_call()?;
                                        let __call_res = Self::s_b5672010fcc9(vm, s_i64_2, host);
                                        vm.exit_call();
                                        let __t7 = __call_res?;
                                        s_i32_2 = __t7;
                                        s_i32_1 = ops::i32_or(s_i32_1, s_i32_2);
                                        s_i32_2 = 255i32;
                                        s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
                                        vm.enter_call()?;
                                        let __call_res = crate::shared_0::Shared::s_5daf123ed54b(vm, s_i32_0, s_i32_1, host);
                                        vm.exit_call();
                                        __call_res?;
                                        s_i32_0 = l1;
                                        s_i32_1 = 128i32;
                                        s_i64_2 = l7;
                                        s_i64_3 = 63i64;
                                        s_i64_2 = ops::i64_and(s_i64_2, s_i64_3);
                                        vm.enter_call()?;
                                        let __call_res = Self::s_b5672010fcc9(vm, s_i64_2, host);
                                        vm.exit_call();
                                        let __t8 = __call_res?;
                                        s_i32_2 = __t8;
                                        s_i32_1 = ops::i32_or(s_i32_1, s_i32_2);
                                        s_i32_2 = 255i32;
                                        s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
                                        vm.enter_call()?;
                                        let __call_res = crate::shared_0::Shared::s_5daf123ed54b(vm, s_i32_0, s_i32_1, host);
                                        vm.exit_call();
                                        __call_res?;
                                    } else {
                                        s_i64_0 = l7;
                                        s_i64_1 = 2097152i64;
                                        s_i32_0 = ops::i64_ge_u(s_i64_0, s_i64_1);
                                        'blk_8: {
                                            if s_i32_0 != 0 {
                                                s_i64_0 = l7;
                                                s_i64_1 = 67108863i64;
                                                s_i32_0 = ops::i64_le_u(s_i64_0, s_i64_1);
                                            } else {
                                                s_i32_0 = 0i32;
                                            }
                                        }
                                        'blk_9: {
                                            if s_i32_0 != 0 {
                                                s_i32_0 = l1;
                                                s_i32_1 = 248i32;
                                                s_i64_2 = l7;
                                                s_i32_3 = 24i32;
                                                vm.enter_call()?;
                                                let __call_res = crate::shared_0::Shared::s_73a866edda27(vm, s_i64_2, s_i32_3, host);
                                                vm.exit_call();
                                                let __t9 = __call_res?;
                                                s_i64_2 = __t9;
                                                vm.enter_call()?;
                                                let __call_res = Self::s_b5672010fcc9(vm, s_i64_2, host);
                                                vm.exit_call();
                                                let __t10 = __call_res?;
                                                s_i32_2 = __t10;
                                                s_i32_1 = ops::i32_or(s_i32_1, s_i32_2);
                                                s_i32_2 = 255i32;
                                                s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
                                                vm.enter_call()?;
                                                let __call_res = crate::shared_0::Shared::s_5daf123ed54b(vm, s_i32_0, s_i32_1, host);
                                                vm.exit_call();
                                                __call_res?;
                                                s_i32_0 = l1;
                                                s_i32_1 = 128i32;
                                                s_i64_2 = l7;
                                                s_i32_3 = 18i32;
                                                vm.enter_call()?;
                                                let __call_res = crate::shared_0::Shared::s_73a866edda27(vm, s_i64_2, s_i32_3, host);
                                                vm.exit_call();
                                                let __t11 = __call_res?;
                                                s_i64_2 = __t11;
                                                s_i64_3 = 63i64;
                                                s_i64_2 = ops::i64_and(s_i64_2, s_i64_3);
                                                vm.enter_call()?;
                                                let __call_res = Self::s_b5672010fcc9(vm, s_i64_2, host);
                                                vm.exit_call();
                                                let __t12 = __call_res?;
                                                s_i32_2 = __t12;
                                                s_i32_1 = ops::i32_or(s_i32_1, s_i32_2);
                                                s_i32_2 = 255i32;
                                                s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
                                                vm.enter_call()?;
                                                let __call_res = crate::shared_0::Shared::s_5daf123ed54b(vm, s_i32_0, s_i32_1, host);
                                                vm.exit_call();
                                                __call_res?;
                                                s_i32_0 = l1;
                                                s_i32_1 = 128i32;
                                                s_i64_2 = l7;
                                                s_i32_3 = 12i32;
                                                vm.enter_call()?;
                                                let __call_res = crate::shared_0::Shared::s_73a866edda27(vm, s_i64_2, s_i32_3, host);
                                                vm.exit_call();
                                                let __t13 = __call_res?;
                                                s_i64_2 = __t13;
                                                s_i64_3 = 63i64;
                                                s_i64_2 = ops::i64_and(s_i64_2, s_i64_3);
                                                vm.enter_call()?;
                                                let __call_res = Self::s_b5672010fcc9(vm, s_i64_2, host);
                                                vm.exit_call();
                                                let __t14 = __call_res?;
                                                s_i32_2 = __t14;
                                                s_i32_1 = ops::i32_or(s_i32_1, s_i32_2);
                                                s_i32_2 = 255i32;
                                                s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
                                                vm.enter_call()?;
                                                let __call_res = crate::shared_0::Shared::s_5daf123ed54b(vm, s_i32_0, s_i32_1, host);
                                                vm.exit_call();
                                                __call_res?;
                                                s_i32_0 = l1;
                                                s_i32_1 = 128i32;
                                                s_i64_2 = l7;
                                                s_i32_3 = 6i32;
                                                vm.enter_call()?;
                                                let __call_res = crate::shared_0::Shared::s_73a866edda27(vm, s_i64_2, s_i32_3, host);
                                                vm.exit_call();
                                                let __t15 = __call_res?;
                                                s_i64_2 = __t15;
                                                s_i64_3 = 63i64;
                                                s_i64_2 = ops::i64_and(s_i64_2, s_i64_3);
                                                vm.enter_call()?;
                                                let __call_res = Self::s_b5672010fcc9(vm, s_i64_2, host);
                                                vm.exit_call();
                                                let __t16 = __call_res?;
                                                s_i32_2 = __t16;
                                                s_i32_1 = ops::i32_or(s_i32_1, s_i32_2);
                                                s_i32_2 = 255i32;
                                                s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
                                                vm.enter_call()?;
                                                let __call_res = crate::shared_0::Shared::s_5daf123ed54b(vm, s_i32_0, s_i32_1, host);
                                                vm.exit_call();
                                                __call_res?;
                                                s_i32_0 = l1;
                                                s_i32_1 = 128i32;
                                                s_i64_2 = l7;
                                                s_i64_3 = 63i64;
                                                s_i64_2 = ops::i64_and(s_i64_2, s_i64_3);
                                                vm.enter_call()?;
                                                let __call_res = Self::s_b5672010fcc9(vm, s_i64_2, host);
                                                vm.exit_call();
                                                let __t17 = __call_res?;
                                                s_i32_2 = __t17;
                                                s_i32_1 = ops::i32_or(s_i32_1, s_i32_2);
                                                s_i32_2 = 255i32;
                                                s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
                                                vm.enter_call()?;
                                                let __call_res = crate::shared_0::Shared::s_5daf123ed54b(vm, s_i32_0, s_i32_1, host);
                                                vm.exit_call();
                                                __call_res?;
                                            } else {
                                                s_i32_0 = l1;
                                                s_i32_1 = l6;
                                                vm.enter_call()?;
                                                let __call_res = Self::s_eb4cabbb1a10(vm, s_i32_0, s_i32_1, host);
                                                vm.exit_call();
                                                __call_res?;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        s_i32_0 = l1;
                        s_i32_1 = l6;
                        vm.enter_call()?;
                        let __call_res = Self::s_eb4cabbb1a10(vm, s_i32_0, s_i32_1, host);
                        vm.exit_call();
                        __call_res?;
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
    s_i32_0 = l1;
    vm.enter_call()?;
    let __call_res = Self::s_df3679579582(vm, s_i32_0, host);
    vm.exit_call();
    let __t18 = __call_res?;
    s_i32_0 = __t18;
    l9 = s_i32_0;
    s_i32_0 = l9;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_b1342c1ced52(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    return Ok(s_i32_0);
    }

    pub(crate) fn s_b3e6ca84b2de(vm: &mut SubstituteVm, p0: i64, p1: i64, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i64 = p0;
    let mut l1: i64 = p1;
    let mut l2: i64 = 0i64;
    let mut l3: i64 = 0i64;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i64_1: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i64_0 = l0;
    l2 = s_i64_0;
    s_i64_0 = l1;
    l3 = s_i64_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
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

    pub(crate) fn s_b3f54762a190(vm: &mut SubstituteVm, p0: i32, p1: i64, p2: i64, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let mut s_i32_3: i32 = 0i32;
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
    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l4 = s_i32_0;
    s_i32_0 = l4;
    s_i32_1 = l0;
    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 0u32)?;
    s_i64_2 = l1;
    s_i32_2 = ops::i32_wrap_i64(s_i64_2);
    s_i32_3 = 2i32;
    s_i32_2 = ops::i32_shl(s_i32_2, s_i32_3);
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

    pub(crate) fn s_b4afe44dab61(vm: &mut SubstituteVm, p0: i32, g0: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i64 = 0i64;
    let mut l2: i64 = 0i64;
    let mut l3: i32 = 0i32;
    let mut l4: i64 = 0i64;
    let mut l5: i32 = 0i32;
    let mut l6: i32 = 0i32;
    let mut l7: i32 = 0i32;
    let mut l8: i64 = 0i64;
    let mut l9: i32 = 0i32;
    let mut l10: i32 = 0i32;
    let mut l11: i32 = 0i32;
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
    s_i32_0 = ops::i64_lt_s(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i64_0 = -1i64;
        } else {
            s_i64_0 = 0i64;
        }
    }
    l2 = s_i64_0;
    s_i32_0 = l0;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
    s_i64_1 = l2;
    s_i32_0 = ops::i64_eq(s_i64_0, s_i64_1);
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
            l3 = s_i32_0;
            s_i64_0 = l1;
            l4 = s_i64_0;
            s_i32_0 = 0i32;
            l5 = s_i32_0;
            s_i32_0 = 20i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t0 = __call_res?;
            s_i32_0 = __t0;
            l6 = s_i32_0;
            s_i32_0 = l6;
            s_i32_1 = l3;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l6;
            s_i64_1 = l4;
            ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
            s_i32_0 = l6;
            s_i32_1 = l5;
            ops::i32_store8(vm, 0, s_i32_0, 16u32, s_i32_1)?;
            s_i32_0 = l6;
        } else {
            s_i32_0 = 0i32;
            l7 = s_i32_0;
            s_i64_0 = 0i64;
            l8 = s_i64_0;
            s_i32_0 = vm.get_i32(g0)?;
            l9 = s_i32_0;
            s_i32_0 = 20i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            l10 = s_i32_0;
            s_i32_0 = l10;
            s_i32_1 = l7;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l10;
            s_i64_1 = l8;
            ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
            s_i32_0 = l10;
            s_i32_1 = l9;
            ops::i32_store8(vm, 0, s_i32_0, 16u32, s_i32_1)?;
            s_i32_0 = l10;
        }
    }
    l11 = s_i32_0;
    s_i32_0 = l11;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_b5672010fcc9(vm: &mut SubstituteVm, p0: i64, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i64 = p0;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i64_1: i64 = 0i64;
    vm.deduct(host, 1)?;
    s_i64_0 = l0;
    s_i64_1 = 255i64;
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

    #[inline(never)]
    pub(crate) fn s_b75f4039fb33(vm: &mut SubstituteVm, p0: i32, p1: i64, p2: i64, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i64, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i64 = p1;
    let mut l2: i64 = p2;
    let mut l3: i32 = 0i32;
    let mut l4: i64 = 0i64;
    let mut l5: i64 = 0i64;
    let mut l6: i64 = 0i64;
    let mut l7: i64 = 0i64;
    let mut l8: i64 = 0i64;
    let mut l9: i64 = 0i64;
    let mut l10: i64 = 0i64;
    let mut l11: i64 = 0i64;
    let mut l12: i64 = 0i64;
    let mut l13: i64 = 0i64;
    let mut l14: i32 = 0i32;
    let mut l15: i64 = 0i64;
    let mut l16: i64 = 0i64;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i32_2: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i64_1 = l1;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_4e6fb2273c0f(vm, s_i32_0, s_i64_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l3 = s_i32_0;
    s_i32_0 = l3;
    s_i32_1 = 128i32;
    vm.enter_call()?;
    let __call_res = Self::s_8b082d5ac52c(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t1 = __call_res?;
    s_i32_0 = __t1;
    'blk_1: {
        if s_i32_0 != 0 {
            s_i64_0 = 1i64;
            l4 = s_i64_0;
            s_i64_0 = l4;
            return Ok(s_i64_0);
        } else {
        }
    }
    s_i32_0 = l3;
    s_i32_1 = 194i32;
    vm.enter_call()?;
    let __call_res = Self::s_8b082d5ac52c(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t2 = __call_res?;
    s_i32_0 = __t2;
    'blk_2: {
        if s_i32_0 != 0 {
            s_i64_0 = 0i64;
            l5 = s_i64_0;
            s_i64_0 = l5;
            return Ok(s_i64_0);
        } else {
        }
    }
    s_i32_0 = l3;
    s_i32_1 = 224i32;
    vm.enter_call()?;
    let __call_res = Self::s_8b082d5ac52c(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t3 = __call_res?;
    s_i32_0 = __t3;
    'blk_3: {
        if s_i32_0 != 0 {
            s_i64_0 = 2i64;
            l6 = s_i64_0;
            s_i64_0 = l6;
            return Ok(s_i64_0);
        } else {
        }
    }
    s_i32_0 = l3;
    s_i32_1 = 240i32;
    vm.enter_call()?;
    let __call_res = Self::s_8b082d5ac52c(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t4 = __call_res?;
    s_i32_0 = __t4;
    'blk_4: {
        if s_i32_0 != 0 {
            s_i64_0 = 3i64;
            l7 = s_i64_0;
            s_i64_0 = l7;
            return Ok(s_i64_0);
        } else {
        }
    }
    s_i32_0 = l3;
    s_i32_1 = 248i32;
    vm.enter_call()?;
    let __call_res = Self::s_8b082d5ac52c(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t5 = __call_res?;
    s_i32_0 = __t5;
    'blk_5: {
        if s_i32_0 != 0 {
            s_i64_0 = 4i64;
            l8 = s_i64_0;
            s_i64_0 = l8;
            return Ok(s_i64_0);
        } else {
        }
    }
    s_i32_0 = l3;
    s_i32_1 = 252i32;
    vm.enter_call()?;
    let __call_res = Self::s_8b082d5ac52c(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t6 = __call_res?;
    s_i32_0 = __t6;
    'blk_6: {
        if s_i32_0 != 0 {
            s_i64_0 = 5i64;
            l9 = s_i64_0;
            s_i64_0 = l9;
            return Ok(s_i64_0);
        } else {
        }
    }
    s_i32_0 = l3;
    s_i32_1 = 254i32;
    vm.enter_call()?;
    let __call_res = Self::s_8b082d5ac52c(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t7 = __call_res?;
    s_i32_0 = __t7;
    'blk_7: {
        if s_i32_0 != 0 {
            s_i64_0 = 6i64;
            l10 = s_i64_0;
            s_i64_0 = l10;
            return Ok(s_i64_0);
        } else {
        }
    }
    s_i32_0 = l3;
    s_i32_1 = 254i32;
    vm.enter_call()?;
    let __call_res = Self::s_b1342c1ced52(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t8 = __call_res?;
    s_i32_0 = __t8;
    'blk_8: {
        if s_i32_0 != 0 {
            s_i64_0 = 7i64;
            l11 = s_i64_0;
            s_i64_0 = l11;
            return Ok(s_i64_0);
        } else {
        }
    }
    s_i64_0 = l1;
    s_i64_1 = 1i64;
    s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
    l12 = s_i64_0;
    s_i64_0 = l12;
    s_i64_1 = l2;
    s_i32_0 = ops::i64_ge_u(s_i64_0, s_i64_1);
    'blk_9: {
        if s_i32_0 != 0 {
            s_i64_0 = 0i64;
            l13 = s_i64_0;
            s_i64_0 = l13;
            return Ok(s_i64_0);
        } else {
        }
    }
    s_i32_0 = l0;
    s_i64_1 = l12;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_4e6fb2273c0f(vm, s_i32_0, s_i64_1, host);
    vm.exit_call();
    let __t9 = __call_res?;
    s_i32_0 = __t9;
    l14 = s_i32_0;
    s_i32_0 = l14;
    s_i32_1 = 192i32;
    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
    s_i32_1 = 255i32;
    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
    s_i32_1 = 128i32;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_3e7647da0841(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t10 = __call_res?;
    s_i32_0 = __t10;
    'blk_10: {
        if s_i32_0 != 0 {
            s_i64_0 = 0i64;
            l15 = s_i64_0;
            s_i64_0 = l15;
            return Ok(s_i64_0);
        } else {
        }
    }
    s_i64_0 = 8i64;
    s_i32_1 = l14;
    s_i32_2 = 2i32;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_708c3053a6ba(vm, s_i32_1, s_i32_2, host);
    vm.exit_call();
    let __t11 = __call_res?;
    s_i32_1 = __t11;
    s_i32_2 = 15i32;
    s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
    s_i32_2 = 255i32;
    s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
    s_i32_2 = 255i32;
    s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
    s_i64_1 = ops::i64_extend_i32_u(s_i32_1);
    s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
    l16 = s_i64_0;
    s_i64_0 = l16;
    return Ok(s_i64_0);
    }

    pub(crate) fn s_b84f0030ee1e(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l1;
    s_i32_1 = l0;
    let __t0 = s_i32_0;
    let __t1 = s_i32_1;
    return crate::shared_0::Shared::s_745aa11cfb74(vm, __t0, __t1, host);
    }

    pub(crate) fn s_b8acef2c4d16(vm: &mut SubstituteVm, p0: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    s_i32_0 = l0;
    s_i32_1 = 255i32;
    s_i32_0 = ops::i32_gt_u(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = l0;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_b92cf85b6153(vm: &mut SubstituteVm, p0: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i64 = 0i64;
    let mut l2: i32 = 0i32;
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
    l1 = s_i64_0;
    'blk_1: {
        'blk_2: loop {
            vm.deduct(host, 1)?;
            s_i64_0 = l1;
            s_i64_1 = 0i64;
            s_i32_0 = ops::i64_gt_u(s_i64_0, s_i64_1);
            'blk_3: {
                if s_i32_0 != 0 {
                    s_i32_0 = l0;
                    s_i64_1 = l1;
                    s_i64_2 = 1i64;
                    vm.enter_call()?;
                    let __call_res = Self::s_aa4577c560e8(vm, s_i64_1, s_i64_2, host);
                    vm.exit_call();
                    let __t0 = __call_res?;
                    s_i64_1 = __t0;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
                    vm.exit_call();
                    let __t1 = __call_res?;
                    s_i32_0 = __t1;
                    s_i32_1 = 0i32;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                } else {
                    s_i32_0 = 0i32;
                }
            }
            s_i32_0 = ops::i32_eqz(s_i32_0);
            if s_i32_0 != 0 {
                break 'blk_1;
            }
            s_i64_0 = l1;
            s_i64_1 = 1i64;
            vm.enter_call()?;
            let __call_res = Self::s_aa4577c560e8(vm, s_i64_0, s_i64_1, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i64_0 = __t2;
            l1 = s_i64_0;
            continue 'blk_2;
            break 'blk_2;
        }
    }
    s_i64_0 = l1;
    s_i32_1 = l0;
    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
    s_i64_1 = ops::i64_extend_i32_u(s_i32_1);
    s_i32_0 = ops::i64_eq(s_i64_0, s_i64_1);
    'blk_4: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
        } else {
            s_i32_0 = l0;
            s_i64_1 = 0i64;
            s_i64_2 = l1;
            vm.enter_call()?;
            let __call_res = Self::s_b3f54762a190(vm, s_i32_0, s_i64_1, s_i64_2, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i32_0 = __t3;
        }
    }
    l2 = s_i32_0;
    s_i32_0 = l2;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_b9e770999dbb(vm: &mut SubstituteVm, p0: i32, p1: i64, p2: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<(), WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i64 = p1;
    let mut l2: i32 = p2;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
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
    s_i64_0 = l1;
    s_i32_1 = l0;
    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
    s_i64_1 = ops::i64_extend_i32_u(s_i32_1);
    s_i32_0 = ops::i64_ge_u(s_i64_0, s_i64_1);
    'blk_2: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    s_i64_1 = l1;
    s_i32_1 = ops::i32_wrap_i64(s_i64_1);
    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
    s_i32_1 = l2;
    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    return Ok(());
    }

    pub(crate) fn s_b9f91f9d99e8(vm: &mut SubstituteVm, p0: i32, g0: usize, g1: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let __call_res = crate::shared_0::Shared::s_4200fb7655b9(vm, s_i32_0, g0, g1, host);
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
            s_i32_0 = vm.get_i32(g1)?;
            l4 = s_i32_0;
            s_i32_0 = 4i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            l5 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = l2;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l5;
            s_i32_1 = l3;
            ops::i32_store8(vm, 0, s_i32_0, 1u32, s_i32_1)?;
            s_i32_0 = l5;
            s_i32_1 = l4;
            ops::i32_store8(vm, 0, s_i32_0, 2u32, s_i32_1)?;
            s_i32_0 = l5;
        } else {
            s_i32_0 = l1;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
            let __t2 = s_i32_0;
            return Self::s_ffe97171ec9d(vm, __t2, g1, host);
        }
    }
    l6 = s_i32_0;
    s_i32_0 = l6;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_b9fd3c36babe(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, g1: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let __call_res = crate::shared_0::Shared::s_1d06de4c220c(vm, s_i32_0, s_i32_1, g0, g1, host);
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
    pub(crate) fn s_ba1f75ca3190(vm: &mut SubstituteVm, p0: i32, g0: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
            s_i32_0 = l0;
            s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 0u32)?;
            s_i64_1 = 9223372036854775807i64;
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
            let __call_res = crate::shared_0::Shared::s_7b03233bc608(vm, s_i64_0, host);
            vm.exit_call();
            let __t0 = __call_res?;
            s_i64_0 = __t0;
            l2 = s_i64_0;
            s_i32_0 = 0i32;
            l3 = s_i32_0;
            s_i32_0 = 20i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
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
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_0 = __t2;
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

    #[inline(never)]
    pub(crate) fn s_bac47b6bd83a(vm: &mut SubstituteVm, p0: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i64 = 0i64;
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
    let mut l16: i64 = 0i64;
    let mut l17: i32 = 0i32;
    let mut l18: i64 = 0i64;
    let mut l19: i64 = 0i64;
    let mut l20: i32 = 0i32;
    let mut l21: i32 = 0i32;
    let mut l22: i32 = 0i32;
    let mut l23: i64 = 0i64;
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
    let mut l35: i32 = 0i32;
    let mut l36: i32 = 0i32;
    let mut l37: i32 = 0i32;
    let mut l38: i32 = 0i32;
    let mut l39: i32 = 0i32;
    let mut l40: i32 = 0i32;
    let mut l41: i64 = 0i64;
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
    let mut l54: i64 = 0i64;
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
    let mut l65: i64 = 0i64;
    let mut l66: i32 = 0i32;
    let mut l67: i32 = 0i32;
    let mut l68: i32 = 0i32;
    let mut l69: i32 = 0i32;
    let mut l70: i32 = 0i32;
    let mut l71: i64 = 0i64;
    let mut l72: i32 = 0i32;
    let mut l73: i32 = 0i32;
    let mut l74: i32 = 0i32;
    let mut l75: i32 = 0i32;
    let mut l76: i32 = 0i32;
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
    s_i64_1 = 20i64;
    s_i32_0 = ops::i64_lt_u(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            l2 = s_i32_0;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t0 = __call_res?;
            s_i32_0 = __t0;
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
            s_i32_0 = 8i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            l5 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = l2;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l5;
            s_i32_1 = l4;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l5;
            l6 = s_i32_0;
            s_i32_0 = l6;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i64_0 = l1;
    s_i64_1 = 4i64;
    s_i64_0 = ops::i64_sub(s_i64_0, s_i64_1);
    l7 = s_i64_0;
    s_i32_0 = l0;
    s_i64_1 = 0i64;
    s_i64_2 = 16i64;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_61029c4c8df8(vm, s_i32_0, s_i64_1, s_i64_2, host);
    vm.exit_call();
    let __t2 = __call_res?;
    s_i32_0 = __t2;
    s_i32_1 = 32i32;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_1, host);
    vm.exit_call();
    let __t3 = __call_res?;
    s_i32_1 = __t3;
    l8 = s_i32_1;
    s_i32_1 = l8;
    s_i32_2 = l8;
    s_i32_3 = 16i32;
    s_i32_2 = ops::i32_add(s_i32_2, s_i32_3);
    ops::i32_store(vm, 0, s_i32_1, 0u32, s_i32_2)?;
    s_i32_1 = l8;
    s_i32_2 = 16i32;
    ops::i32_store(vm, 0, s_i32_1, 4u32, s_i32_2)?;
    s_i32_1 = l8;
    s_i32_2 = 16i32;
    ops::i32_store(vm, 0, s_i32_1, 8u32, s_i32_2)?;
    s_i32_1 = l8;
    s_i32_2 = 1i32;
    ops::i32_store(vm, 0, s_i32_1, 12u32, s_i32_2)?;
    s_i32_1 = l8;
    s_i64_2 = -8753712395831701260i64;
    ops::i64_store(vm, 0, s_i32_1, 16u32, s_i64_2)?;
    s_i32_1 = l8;
    s_i64_2 = -8681654801726664460i64;
    ops::i64_store(vm, 0, s_i32_1, 24u32, s_i64_2)?;
    s_i32_1 = l8;
    vm.enter_call()?;
    let __call_res = Self::s_9e8b02290b48(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t4 = __call_res?;
    s_i32_0 = __t4;
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
        } else {
            s_i32_0 = l0;
            s_i64_1 = l7;
            s_i64_2 = l1;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_61029c4c8df8(vm, s_i32_0, s_i64_1, s_i64_2, host);
            vm.exit_call();
            let __t5 = __call_res?;
            s_i32_0 = __t5;
            s_i32_1 = 20i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_1, host);
            vm.exit_call();
            let __t6 = __call_res?;
            s_i32_1 = __t6;
            l9 = s_i32_1;
            s_i32_1 = l9;
            s_i32_2 = l9;
            s_i32_3 = 16i32;
            s_i32_2 = ops::i32_add(s_i32_2, s_i32_3);
            ops::i32_store(vm, 0, s_i32_1, 0u32, s_i32_2)?;
            s_i32_1 = l9;
            s_i32_2 = 4i32;
            ops::i32_store(vm, 0, s_i32_1, 4u32, s_i32_2)?;
            s_i32_1 = l9;
            s_i32_2 = 4i32;
            ops::i32_store(vm, 0, s_i32_1, 8u32, s_i32_2)?;
            s_i32_1 = l9;
            s_i32_2 = 1i32;
            ops::i32_store(vm, 0, s_i32_1, 12u32, s_i32_2)?;
            s_i32_1 = l9;
            s_i32_2 = 244i32;
            ops::i32_store8(vm, 0, s_i32_1, 16u32, s_i32_2)?;
            s_i32_1 = l9;
            s_i32_2 = 144i32;
            ops::i32_store8(vm, 0, s_i32_1, 17u32, s_i32_2)?;
            s_i32_1 = l9;
            s_i32_2 = 132i32;
            ops::i32_store8(vm, 0, s_i32_1, 18u32, s_i32_2)?;
            s_i32_1 = l9;
            s_i32_2 = 133i32;
            ops::i32_store8(vm, 0, s_i32_1, 19u32, s_i32_2)?;
            s_i32_1 = l9;
            vm.enter_call()?;
            let __call_res = Self::s_9e8b02290b48(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t7 = __call_res?;
            s_i32_0 = __t7;
        }
    }
    'blk_3: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            l10 = s_i32_0;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t8 = __call_res?;
            s_i32_0 = __t8;
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
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t9 = __call_res?;
            s_i32_0 = __t9;
            l13 = s_i32_0;
            s_i32_0 = l13;
            s_i32_1 = l10;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
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
    s_i64_0 = l7;
    s_i64_1 = 16i64;
    s_i64_0 = ops::i64_sub(s_i64_0, s_i64_1);
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_61878e7d5ba7(vm, s_i64_0, host);
    vm.exit_call();
    let __t10 = __call_res?;
    s_i32_0 = __t10;
    l15 = s_i32_0;
    s_i64_0 = 16i64;
    l16 = s_i64_0;
    s_i32_0 = 0i32;
    l17 = s_i32_0;
    s_i64_0 = 16i64;
    l18 = s_i64_0;
    s_i64_0 = l7;
    s_i64_1 = 5i64;
    s_i32_0 = ops::i64_ge_u(s_i64_0, s_i64_1);
    'blk_4: {
        if s_i32_0 != 0 {
            s_i64_0 = l7;
            s_i64_1 = 5i64;
            s_i64_0 = ops::i64_sub(s_i64_0, s_i64_1);
        } else {
            s_i64_0 = 0i64;
        }
    }
    l19 = s_i64_0;
    'blk_5: {
        'blk_6: loop {
            vm.deduct(host, 1)?;
            s_i64_0 = l18;
            s_i64_1 = l19;
            s_i32_0 = ops::i64_lt_u(s_i64_0, s_i64_1);
            s_i32_0 = ops::i32_eqz(s_i32_0);
            if s_i32_0 != 0 {
                break 'blk_5;
            }
            s_i32_0 = l0;
            s_i64_1 = l18;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_4e6fb2273c0f(vm, s_i32_0, s_i64_1, host);
            vm.exit_call();
            let __t11 = __call_res?;
            s_i32_0 = __t11;
            s_i32_1 = 255i32;
            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
            l20 = s_i32_0;
            s_i32_0 = l20;
            s_i32_1 = 244i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_7: {
                if s_i32_0 != 0 {
                    s_i32_0 = l0;
                    s_i64_1 = l18;
                    s_i64_2 = 2i64;
                    s_i64_1 = ops::i64_add(s_i64_1, s_i64_2);
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_4e6fb2273c0f(vm, s_i32_0, s_i64_1, host);
                    vm.exit_call();
                    let __t12 = __call_res?;
                    s_i32_0 = __t12;
                    s_i32_1 = 255i32;
                    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                    l21 = s_i32_0;
                    s_i32_0 = l21;
                    s_i32_1 = 131i32;
                    s_i32_0 = ops::i32_sub(s_i32_0, s_i32_1);
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_le_u(s_i32_0, s_i32_1);
                    'blk_8: {
                        if s_i32_0 != 0 {
                            s_i32_0 = l0;
                            s_i64_1 = l18;
                            s_i64_2 = 1i64;
                            s_i64_1 = ops::i64_add(s_i64_1, s_i64_2);
                            vm.enter_call()?;
                            let __call_res = crate::shared_0::Shared::s_4e6fb2273c0f(vm, s_i32_0, s_i64_1, host);
                            vm.exit_call();
                            let __t13 = __call_res?;
                            s_i32_0 = __t13;
                            s_i32_1 = 255i32;
                            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                            s_i32_1 = 144i32;
                            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                            'blk_9: {
                                if s_i32_0 != 0 {
                                    s_i32_0 = l0;
                                    s_i64_1 = l18;
                                    s_i64_2 = 3i64;
                                    s_i64_1 = ops::i64_add(s_i64_1, s_i64_2);
                                    vm.enter_call()?;
                                    let __call_res = crate::shared_0::Shared::s_4e6fb2273c0f(vm, s_i32_0, s_i64_1, host);
                                    vm.exit_call();
                                    let __t14 = __call_res?;
                                    s_i32_0 = __t14;
                                    s_i32_1 = 255i32;
                                    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                                    l22 = s_i32_0;
                                    s_i32_0 = l21;
                                    s_i32_1 = 131i32;
                                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                                    'blk_10: {
                                        if s_i32_0 != 0 {
                                            s_i32_0 = l22;
                                            s_i32_1 = 191i32;
                                            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                                        } else {
                                            s_i32_0 = 0i32;
                                        }
                                    }
                                    'blk_11: {
                                        if s_i32_0 != 0 {
                                            s_i32_0 = 1i32;
                                            l17 = s_i32_0;
                                            s_i64_0 = l18;
                                            s_i64_1 = l16;
                                            s_i32_0 = ops::i64_gt_u(s_i64_0, s_i64_1);
                                            'blk_12: {
                                                if s_i32_0 != 0 {
                                                    s_i32_0 = l15;
                                                    s_i32_1 = l0;
                                                    s_i64_2 = l16;
                                                    s_i64_3 = l18;
                                                    vm.enter_call()?;
                                                    let __call_res = crate::shared_0::Shared::s_61029c4c8df8(vm, s_i32_1, s_i64_2, s_i64_3, host);
                                                    vm.exit_call();
                                                    let __t15 = __call_res?;
                                                    s_i32_1 = __t15;
                                                    vm.enter_call()?;
                                                    let __call_res = Self::s_de64faaada56(vm, s_i32_0, s_i32_1, host);
                                                    vm.exit_call();
                                                    __call_res?;
                                                } else {
                                                }
                                            }
                                            s_i64_0 = l18;
                                            s_i64_1 = 4i64;
                                            s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
                                            l18 = s_i64_0;
                                            s_i64_0 = l18;
                                            l16 = s_i64_0;
                                            s_i32_0 = l0;
                                            s_i64_1 = l18;
                                            s_i64_2 = l7;
                                            vm.enter_call()?;
                                            let __call_res = Self::s_b75f4039fb33(vm, s_i32_0, s_i64_1, s_i64_2, host);
                                            vm.exit_call();
                                            let __t16 = __call_res?;
                                            s_i64_0 = __t16;
                                            l23 = s_i64_0;
                                            s_i64_0 = l23;
                                            s_i64_1 = 0i64;
                                            s_i32_0 = ops::i64_eq(s_i64_0, s_i64_1);
                                            'blk_13: {
                                                if s_i32_0 != 0 {
                                                    s_i32_0 = 1i32;
                                                } else {
                                                    s_i64_0 = l18;
                                                    s_i64_1 = l23;
                                                    s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
                                                    s_i64_1 = l7;
                                                    s_i32_0 = ops::i64_gt_u(s_i64_0, s_i64_1);
                                                }
                                            }
                                            'blk_14: {
                                                if s_i32_0 != 0 {
                                                    s_i32_0 = 0i32;
                                                    l24 = s_i32_0;
                                                    s_i32_0 = 16i32;
                                                    vm.enter_call()?;
                                                    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                                                    vm.exit_call();
                                                    let __t17 = __call_res?;
                                                    s_i32_0 = __t17;
                                                    l25 = s_i32_0;
                                                    s_i32_0 = l25;
                                                    s_i32_1 = l25;
                                                    s_i32_2 = 16i32;
                                                    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
                                                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                                                    s_i32_0 = l25;
                                                    s_i32_1 = 0i32;
                                                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                                                    s_i32_0 = l25;
                                                    s_i32_1 = 0i32;
                                                    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                                                    s_i32_0 = l25;
                                                    s_i32_1 = 1i32;
                                                    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                                                    s_i32_0 = l25;
                                                    l26 = s_i32_0;
                                                    s_i32_0 = 8i32;
                                                    vm.enter_call()?;
                                                    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                                                    vm.exit_call();
                                                    let __t18 = __call_res?;
                                                    s_i32_0 = __t18;
                                                    l27 = s_i32_0;
                                                    s_i32_0 = l27;
                                                    s_i32_1 = l24;
                                                    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                                                    s_i32_0 = l27;
                                                    s_i32_1 = l26;
                                                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                                                    s_i32_0 = l27;
                                                    l28 = s_i32_0;
                                                    s_i32_0 = l28;
                                                    return Ok(s_i32_0);
                                                } else {
                                                }
                                            }
                                            s_i64_0 = l18;
                                            s_i64_1 = l23;
                                            s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
                                            l18 = s_i64_0;
                                            continue 'blk_6;
                                        } else {
                                        }
                                    }
                                    s_i32_0 = l21;
                                    s_i32_1 = 132i32;
                                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                                    'blk_15: {
                                        if s_i32_0 != 0 {
                                            s_i32_0 = l22;
                                            s_i32_1 = 133i32;
                                            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                                        } else {
                                            s_i32_0 = 0i32;
                                        }
                                    }
                                    'blk_16: {
                                        if s_i32_0 != 0 {
                                            s_i32_0 = 0i32;
                                            l29 = s_i32_0;
                                            s_i32_0 = 16i32;
                                            vm.enter_call()?;
                                            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                                            vm.exit_call();
                                            let __t19 = __call_res?;
                                            s_i32_0 = __t19;
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
                                            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                                            vm.exit_call();
                                            let __t20 = __call_res?;
                                            s_i32_0 = __t20;
                                            l32 = s_i32_0;
                                            s_i32_0 = l32;
                                            s_i32_1 = l29;
                                            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                                            s_i32_0 = l32;
                                            s_i32_1 = l31;
                                            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                                            s_i32_0 = l32;
                                            l33 = s_i32_0;
                                            s_i32_0 = l33;
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
                    s_i64_0 = l18;
                    s_i64_1 = 4i64;
                    s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
                    l18 = s_i64_0;
                } else {
                    s_i32_0 = l20;
                    s_i32_1 = 248i32;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                    'blk_17: {
                        if s_i32_0 != 0 {
                            s_i64_0 = l18;
                            s_i64_1 = 5i64;
                            s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
                            l18 = s_i64_0;
                        } else {
                            s_i32_0 = l20;
                            s_i32_1 = 128i32;
                            s_i32_0 = ops::i32_lt_u(s_i32_0, s_i32_1);
                            'blk_18: {
                                if s_i32_0 != 0 {
                                    s_i64_0 = l18;
                                    s_i64_1 = 1i64;
                                    s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
                                    l18 = s_i64_0;
                                } else {
                                    s_i32_0 = l0;
                                    s_i64_1 = l18;
                                    s_i64_2 = l7;
                                    vm.enter_call()?;
                                    let __call_res = Self::s_b75f4039fb33(vm, s_i32_0, s_i64_1, s_i64_2, host);
                                    vm.exit_call();
                                    let __t21 = __call_res?;
                                    s_i64_0 = __t21;
                                    l34 = s_i64_0;
                                    s_i64_0 = l34;
                                    s_i64_1 = 0i64;
                                    s_i32_0 = ops::i64_eq(s_i64_0, s_i64_1);
                                    'blk_19: {
                                        if s_i32_0 != 0 {
                                            s_i32_0 = 1i32;
                                        } else {
                                            s_i64_0 = l18;
                                            s_i64_1 = l34;
                                            s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
                                            s_i64_1 = l7;
                                            s_i32_0 = ops::i64_gt_u(s_i64_0, s_i64_1);
                                        }
                                    }
                                    'blk_20: {
                                        if s_i32_0 != 0 {
                                            s_i32_0 = 0i32;
                                            l35 = s_i32_0;
                                            s_i32_0 = 16i32;
                                            vm.enter_call()?;
                                            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                                            vm.exit_call();
                                            let __t22 = __call_res?;
                                            s_i32_0 = __t22;
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
                                            s_i32_0 = 8i32;
                                            vm.enter_call()?;
                                            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                                            vm.exit_call();
                                            let __t23 = __call_res?;
                                            s_i32_0 = __t23;
                                            l38 = s_i32_0;
                                            s_i32_0 = l38;
                                            s_i32_1 = l35;
                                            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                                            s_i32_0 = l38;
                                            s_i32_1 = l37;
                                            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                                            s_i32_0 = l38;
                                            l39 = s_i32_0;
                                            s_i32_0 = l39;
                                            return Ok(s_i32_0);
                                        } else {
                                        }
                                    }
                                    s_i64_0 = l18;
                                    s_i64_1 = l34;
                                    s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
                                    l18 = s_i64_0;
                                }
                            }
                        }
                    }
                }
            }
            continue 'blk_6;
            break 'blk_6;
        }
    }
    'blk_21: {
        'blk_22: loop {
            vm.deduct(host, 1)?;
            s_i64_0 = l18;
            s_i64_1 = l7;
            s_i32_0 = ops::i64_lt_u(s_i64_0, s_i64_1);
            s_i32_0 = ops::i32_eqz(s_i32_0);
            if s_i32_0 != 0 {
                break 'blk_21;
            }
            s_i32_0 = l0;
            s_i64_1 = l18;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_4e6fb2273c0f(vm, s_i32_0, s_i64_1, host);
            vm.exit_call();
            let __t24 = __call_res?;
            s_i32_0 = __t24;
            s_i32_1 = 255i32;
            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
            l40 = s_i32_0;
            s_i32_0 = l40;
            s_i32_1 = 244i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_23: {
                if s_i32_0 != 0 {
                    s_i64_0 = l18;
                    s_i64_1 = 4i64;
                    s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
                    l41 = s_i64_0;
                    s_i64_0 = l41;
                    s_i64_1 = l7;
                    s_i32_0 = ops::i64_gt_u(s_i64_0, s_i64_1);
                    'blk_24: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 0i32;
                            l42 = s_i32_0;
                            s_i32_0 = 16i32;
                            vm.enter_call()?;
                            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                            vm.exit_call();
                            let __t25 = __call_res?;
                            s_i32_0 = __t25;
                            l43 = s_i32_0;
                            s_i32_0 = l43;
                            s_i32_1 = l43;
                            s_i32_2 = 16i32;
                            s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
                            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                            s_i32_0 = l43;
                            s_i32_1 = 0i32;
                            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                            s_i32_0 = l43;
                            s_i32_1 = 0i32;
                            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                            s_i32_0 = l43;
                            s_i32_1 = 1i32;
                            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                            s_i32_0 = l43;
                            l44 = s_i32_0;
                            s_i32_0 = 8i32;
                            vm.enter_call()?;
                            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                            vm.exit_call();
                            let __t26 = __call_res?;
                            s_i32_0 = __t26;
                            l45 = s_i32_0;
                            s_i32_0 = l45;
                            s_i32_1 = l42;
                            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                            s_i32_0 = l45;
                            s_i32_1 = l44;
                            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                            s_i32_0 = l45;
                            l46 = s_i32_0;
                            s_i32_0 = l46;
                            return Ok(s_i32_0);
                        } else {
                        }
                    }
                    s_i32_0 = l0;
                    s_i64_1 = l18;
                    s_i64_2 = 2i64;
                    s_i64_1 = ops::i64_add(s_i64_1, s_i64_2);
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_4e6fb2273c0f(vm, s_i32_0, s_i64_1, host);
                    vm.exit_call();
                    let __t27 = __call_res?;
                    s_i32_0 = __t27;
                    s_i32_1 = 255i32;
                    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                    l47 = s_i32_0;
                    s_i32_0 = l47;
                    s_i32_1 = 131i32;
                    s_i32_0 = ops::i32_sub(s_i32_0, s_i32_1);
                    s_i32_1 = 1i32;
                    s_i32_0 = ops::i32_le_u(s_i32_0, s_i32_1);
                    'blk_25: {
                        if s_i32_0 != 0 {
                            s_i32_0 = l0;
                            s_i64_1 = l18;
                            s_i64_2 = 1i64;
                            s_i64_1 = ops::i64_add(s_i64_1, s_i64_2);
                            vm.enter_call()?;
                            let __call_res = crate::shared_0::Shared::s_4e6fb2273c0f(vm, s_i32_0, s_i64_1, host);
                            vm.exit_call();
                            let __t28 = __call_res?;
                            s_i32_0 = __t28;
                            s_i32_1 = 255i32;
                            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                            s_i32_1 = 144i32;
                            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                            'blk_26: {
                                if s_i32_0 != 0 {
                                    s_i32_0 = l0;
                                    s_i64_1 = l18;
                                    s_i64_2 = 3i64;
                                    s_i64_1 = ops::i64_add(s_i64_1, s_i64_2);
                                    vm.enter_call()?;
                                    let __call_res = crate::shared_0::Shared::s_4e6fb2273c0f(vm, s_i32_0, s_i64_1, host);
                                    vm.exit_call();
                                    let __t29 = __call_res?;
                                    s_i32_0 = __t29;
                                    s_i32_1 = 255i32;
                                    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                                    l48 = s_i32_0;
                                    s_i32_0 = l47;
                                    s_i32_1 = 131i32;
                                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                                    'blk_27: {
                                        if s_i32_0 != 0 {
                                            s_i32_0 = l48;
                                            s_i32_1 = 191i32;
                                            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                                        } else {
                                            s_i32_0 = 0i32;
                                        }
                                    }
                                    'blk_28: {
                                        if s_i32_0 != 0 {
                                            s_i32_0 = 1i32;
                                            l17 = s_i32_0;
                                            s_i64_0 = l18;
                                            s_i64_1 = l16;
                                            s_i32_0 = ops::i64_gt_u(s_i64_0, s_i64_1);
                                            'blk_29: {
                                                if s_i32_0 != 0 {
                                                    s_i32_0 = l15;
                                                    s_i32_1 = l0;
                                                    s_i64_2 = l16;
                                                    s_i64_3 = l18;
                                                    vm.enter_call()?;
                                                    let __call_res = crate::shared_0::Shared::s_61029c4c8df8(vm, s_i32_1, s_i64_2, s_i64_3, host);
                                                    vm.exit_call();
                                                    let __t30 = __call_res?;
                                                    s_i32_1 = __t30;
                                                    vm.enter_call()?;
                                                    let __call_res = Self::s_de64faaada56(vm, s_i32_0, s_i32_1, host);
                                                    vm.exit_call();
                                                    __call_res?;
                                                } else {
                                                }
                                            }
                                            s_i64_0 = l41;
                                            l18 = s_i64_0;
                                            s_i64_0 = l18;
                                            l16 = s_i64_0;
                                            s_i64_0 = l18;
                                            s_i64_1 = l7;
                                            s_i32_0 = ops::i64_ge_u(s_i64_0, s_i64_1);
                                            'blk_30: {
                                                if s_i32_0 != 0 {
                                                    s_i32_0 = 0i32;
                                                    l49 = s_i32_0;
                                                    s_i32_0 = 16i32;
                                                    vm.enter_call()?;
                                                    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                                                    vm.exit_call();
                                                    let __t31 = __call_res?;
                                                    s_i32_0 = __t31;
                                                    l50 = s_i32_0;
                                                    s_i32_0 = l50;
                                                    s_i32_1 = l50;
                                                    s_i32_2 = 16i32;
                                                    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
                                                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                                                    s_i32_0 = l50;
                                                    s_i32_1 = 0i32;
                                                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                                                    s_i32_0 = l50;
                                                    s_i32_1 = 0i32;
                                                    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                                                    s_i32_0 = l50;
                                                    s_i32_1 = 1i32;
                                                    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                                                    s_i32_0 = l50;
                                                    l51 = s_i32_0;
                                                    s_i32_0 = 8i32;
                                                    vm.enter_call()?;
                                                    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                                                    vm.exit_call();
                                                    let __t32 = __call_res?;
                                                    s_i32_0 = __t32;
                                                    l52 = s_i32_0;
                                                    s_i32_0 = l52;
                                                    s_i32_1 = l49;
                                                    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                                                    s_i32_0 = l52;
                                                    s_i32_1 = l51;
                                                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                                                    s_i32_0 = l52;
                                                    l53 = s_i32_0;
                                                    s_i32_0 = l53;
                                                    return Ok(s_i32_0);
                                                } else {
                                                }
                                            }
                                            s_i32_0 = l0;
                                            s_i64_1 = l18;
                                            s_i64_2 = l7;
                                            vm.enter_call()?;
                                            let __call_res = Self::s_b75f4039fb33(vm, s_i32_0, s_i64_1, s_i64_2, host);
                                            vm.exit_call();
                                            let __t33 = __call_res?;
                                            s_i64_0 = __t33;
                                            l54 = s_i64_0;
                                            s_i64_0 = l54;
                                            s_i64_1 = 0i64;
                                            s_i32_0 = ops::i64_eq(s_i64_0, s_i64_1);
                                            'blk_31: {
                                                if s_i32_0 != 0 {
                                                    s_i32_0 = 1i32;
                                                } else {
                                                    s_i64_0 = l18;
                                                    s_i64_1 = l54;
                                                    s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
                                                    s_i64_1 = l7;
                                                    s_i32_0 = ops::i64_gt_u(s_i64_0, s_i64_1);
                                                }
                                            }
                                            'blk_32: {
                                                if s_i32_0 != 0 {
                                                    s_i32_0 = 0i32;
                                                    l55 = s_i32_0;
                                                    s_i32_0 = 16i32;
                                                    vm.enter_call()?;
                                                    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                                                    vm.exit_call();
                                                    let __t34 = __call_res?;
                                                    s_i32_0 = __t34;
                                                    l56 = s_i32_0;
                                                    s_i32_0 = l56;
                                                    s_i32_1 = l56;
                                                    s_i32_2 = 16i32;
                                                    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
                                                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                                                    s_i32_0 = l56;
                                                    s_i32_1 = 0i32;
                                                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                                                    s_i32_0 = l56;
                                                    s_i32_1 = 0i32;
                                                    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                                                    s_i32_0 = l56;
                                                    s_i32_1 = 1i32;
                                                    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                                                    s_i32_0 = l56;
                                                    l57 = s_i32_0;
                                                    s_i32_0 = 8i32;
                                                    vm.enter_call()?;
                                                    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                                                    vm.exit_call();
                                                    let __t35 = __call_res?;
                                                    s_i32_0 = __t35;
                                                    l58 = s_i32_0;
                                                    s_i32_0 = l58;
                                                    s_i32_1 = l55;
                                                    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                                                    s_i32_0 = l58;
                                                    s_i32_1 = l57;
                                                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                                                    s_i32_0 = l58;
                                                    l59 = s_i32_0;
                                                    s_i32_0 = l59;
                                                    return Ok(s_i32_0);
                                                } else {
                                                }
                                            }
                                            s_i64_0 = l18;
                                            s_i64_1 = l54;
                                            s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
                                            l18 = s_i64_0;
                                            continue 'blk_22;
                                        } else {
                                        }
                                    }
                                    s_i32_0 = l47;
                                    s_i32_1 = 132i32;
                                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                                    'blk_33: {
                                        if s_i32_0 != 0 {
                                            s_i32_0 = l48;
                                            s_i32_1 = 133i32;
                                            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                                        } else {
                                            s_i32_0 = 0i32;
                                        }
                                    }
                                    'blk_34: {
                                        if s_i32_0 != 0 {
                                            s_i32_0 = 0i32;
                                            l60 = s_i32_0;
                                            s_i32_0 = 16i32;
                                            vm.enter_call()?;
                                            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                                            vm.exit_call();
                                            let __t36 = __call_res?;
                                            s_i32_0 = __t36;
                                            l61 = s_i32_0;
                                            s_i32_0 = l61;
                                            s_i32_1 = l61;
                                            s_i32_2 = 16i32;
                                            s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
                                            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                                            s_i32_0 = l61;
                                            s_i32_1 = 0i32;
                                            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                                            s_i32_0 = l61;
                                            s_i32_1 = 0i32;
                                            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                                            s_i32_0 = l61;
                                            s_i32_1 = 1i32;
                                            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                                            s_i32_0 = l61;
                                            l62 = s_i32_0;
                                            s_i32_0 = 8i32;
                                            vm.enter_call()?;
                                            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                                            vm.exit_call();
                                            let __t37 = __call_res?;
                                            s_i32_0 = __t37;
                                            l63 = s_i32_0;
                                            s_i32_0 = l63;
                                            s_i32_1 = l60;
                                            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                                            s_i32_0 = l63;
                                            s_i32_1 = l62;
                                            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                                            s_i32_0 = l63;
                                            l64 = s_i32_0;
                                            s_i32_0 = l64;
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
                    s_i64_0 = l41;
                    l18 = s_i64_0;
                } else {
                    s_i32_0 = l40;
                    s_i32_1 = 248i32;
                    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
                    'blk_35: {
                        if s_i32_0 != 0 {
                            s_i64_0 = l18;
                            s_i64_1 = 5i64;
                            s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
                            l65 = s_i64_0;
                            s_i64_0 = l65;
                            s_i64_1 = l7;
                            s_i32_0 = ops::i64_gt_u(s_i64_0, s_i64_1);
                            'blk_36: {
                                if s_i32_0 != 0 {
                                    s_i32_0 = 0i32;
                                    l66 = s_i32_0;
                                    s_i32_0 = 16i32;
                                    vm.enter_call()?;
                                    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                                    vm.exit_call();
                                    let __t38 = __call_res?;
                                    s_i32_0 = __t38;
                                    l67 = s_i32_0;
                                    s_i32_0 = l67;
                                    s_i32_1 = l67;
                                    s_i32_2 = 16i32;
                                    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
                                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                                    s_i32_0 = l67;
                                    s_i32_1 = 0i32;
                                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                                    s_i32_0 = l67;
                                    s_i32_1 = 0i32;
                                    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                                    s_i32_0 = l67;
                                    s_i32_1 = 1i32;
                                    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                                    s_i32_0 = l67;
                                    l68 = s_i32_0;
                                    s_i32_0 = 8i32;
                                    vm.enter_call()?;
                                    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                                    vm.exit_call();
                                    let __t39 = __call_res?;
                                    s_i32_0 = __t39;
                                    l69 = s_i32_0;
                                    s_i32_0 = l69;
                                    s_i32_1 = l66;
                                    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                                    s_i32_0 = l69;
                                    s_i32_1 = l68;
                                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                                    s_i32_0 = l69;
                                    l70 = s_i32_0;
                                    s_i32_0 = l70;
                                    return Ok(s_i32_0);
                                } else {
                                }
                            }
                            s_i64_0 = l65;
                            l18 = s_i64_0;
                        } else {
                            s_i32_0 = l40;
                            s_i32_1 = 128i32;
                            s_i32_0 = ops::i32_lt_u(s_i32_0, s_i32_1);
                            'blk_37: {
                                if s_i32_0 != 0 {
                                    s_i64_0 = l18;
                                    s_i64_1 = 1i64;
                                    s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
                                    l18 = s_i64_0;
                                } else {
                                    s_i32_0 = l0;
                                    s_i64_1 = l18;
                                    s_i64_2 = l7;
                                    vm.enter_call()?;
                                    let __call_res = Self::s_b75f4039fb33(vm, s_i32_0, s_i64_1, s_i64_2, host);
                                    vm.exit_call();
                                    let __t40 = __call_res?;
                                    s_i64_0 = __t40;
                                    l71 = s_i64_0;
                                    s_i64_0 = l71;
                                    s_i64_1 = 0i64;
                                    s_i32_0 = ops::i64_eq(s_i64_0, s_i64_1);
                                    'blk_38: {
                                        if s_i32_0 != 0 {
                                            s_i32_0 = 1i32;
                                        } else {
                                            s_i64_0 = l18;
                                            s_i64_1 = l71;
                                            s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
                                            s_i64_1 = l7;
                                            s_i32_0 = ops::i64_gt_u(s_i64_0, s_i64_1);
                                        }
                                    }
                                    'blk_39: {
                                        if s_i32_0 != 0 {
                                            s_i32_0 = 0i32;
                                            l72 = s_i32_0;
                                            s_i32_0 = 16i32;
                                            vm.enter_call()?;
                                            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                                            vm.exit_call();
                                            let __t41 = __call_res?;
                                            s_i32_0 = __t41;
                                            l73 = s_i32_0;
                                            s_i32_0 = l73;
                                            s_i32_1 = l73;
                                            s_i32_2 = 16i32;
                                            s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
                                            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                                            s_i32_0 = l73;
                                            s_i32_1 = 0i32;
                                            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                                            s_i32_0 = l73;
                                            s_i32_1 = 0i32;
                                            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                                            s_i32_0 = l73;
                                            s_i32_1 = 1i32;
                                            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                                            s_i32_0 = l73;
                                            l74 = s_i32_0;
                                            s_i32_0 = 8i32;
                                            vm.enter_call()?;
                                            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                                            vm.exit_call();
                                            let __t42 = __call_res?;
                                            s_i32_0 = __t42;
                                            l75 = s_i32_0;
                                            s_i32_0 = l75;
                                            s_i32_1 = l72;
                                            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                                            s_i32_0 = l75;
                                            s_i32_1 = l74;
                                            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                                            s_i32_0 = l75;
                                            l76 = s_i32_0;
                                            s_i32_0 = l76;
                                            return Ok(s_i32_0);
                                        } else {
                                        }
                                    }
                                    s_i64_0 = l18;
                                    s_i64_1 = l71;
                                    s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
                                    l18 = s_i64_0;
                                }
                            }
                        }
                    }
                }
            }
            continue 'blk_22;
            break 'blk_22;
        }
    }
    s_i32_0 = l17;
    s_i32_0 = ops::i32_eqz(s_i32_0);
    'blk_40: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
            l77 = s_i32_0;
            s_i32_0 = l0;
            s_i64_1 = 16i64;
            s_i64_2 = l7;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_61029c4c8df8(vm, s_i32_0, s_i64_1, s_i64_2, host);
            vm.exit_call();
            let __t43 = __call_res?;
            s_i32_0 = __t43;
            l78 = s_i32_0;
            s_i32_0 = 8i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t44 = __call_res?;
            s_i32_0 = __t44;
            l79 = s_i32_0;
            s_i32_0 = l79;
            s_i32_1 = l77;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l79;
            s_i32_1 = l78;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l79;
            l80 = s_i32_0;
            s_i32_0 = l80;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i64_0 = l7;
    s_i64_1 = l16;
    s_i32_0 = ops::i64_gt_u(s_i64_0, s_i64_1);
    'blk_41: {
        if s_i32_0 != 0 {
            s_i32_0 = l15;
            s_i32_1 = l0;
            s_i64_2 = l16;
            s_i64_3 = l7;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_61029c4c8df8(vm, s_i32_1, s_i64_2, s_i64_3, host);
            vm.exit_call();
            let __t45 = __call_res?;
            s_i32_1 = __t45;
            vm.enter_call()?;
            let __call_res = Self::s_de64faaada56(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
        } else {
        }
    }
    s_i32_0 = 1i32;
    l81 = s_i32_0;
    s_i32_0 = l15;
    vm.enter_call()?;
    let __call_res = Self::s_df3679579582(vm, s_i32_0, host);
    vm.exit_call();
    let __t46 = __call_res?;
    s_i32_0 = __t46;
    l82 = s_i32_0;
    s_i32_0 = 8i32;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
    vm.exit_call();
    let __t47 = __call_res?;
    s_i32_0 = __t47;
    l83 = s_i32_0;
    s_i32_0 = l83;
    s_i32_1 = l81;
    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l83;
    s_i32_1 = l82;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l83;
    l84 = s_i32_0;
    s_i32_0 = l84;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_bb055f113709(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, g1: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let __call_res = Self::s_c9242759e3db(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l2 = s_i32_0;
    s_i32_0 = l2;
    s_i32_1 = l0;
    vm.enter_call()?;
    let __call_res = Self::s_c92ee68c0844(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t1 = __call_res?;
    s_i32_0 = __t1;
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
            l3 = s_i32_0;
            s_i32_0 = l2;
            l4 = s_i32_0;
            s_i32_0 = 0i32;
            l5 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_0 = __t2;
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
        } else {
            s_i32_0 = 0i32;
            l7 = s_i32_0;
            s_i32_0 = vm.get_i32(g0)?;
            l8 = s_i32_0;
            s_i32_0 = vm.get_i32(g1)?;
            l9 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i32_0 = __t3;
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
        }
    }
    l11 = s_i32_0;
    s_i32_0 = l11;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_bcd9b7adc78b(vm: &mut SubstituteVm, p0: i32, p1: i64, g0: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i64 = p1;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i32_2: i32 = 0i32;
    let mut s_i64_2: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_1 = 0i32;
    s_i64_2 = l1;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_41e9bacc9fe3(vm, s_i64_2, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_2 = __t0;
    let __t1 = s_i32_0;
    let __t2 = s_i32_1;
    let __t3 = s_i32_2;
    return crate::shared_0::Shared::s_6e6223e30e05(vm, __t1, __t2, __t3, g0, host);
    }

    #[inline(never)]
    pub(crate) fn s_bd22d8f9c2fc(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, g1: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let mut l14: i64 = 0i64;
    let mut l15: i64 = 0i64;
    let mut l16: i32 = 0i32;
    let mut l17: i64 = 0i64;
    let mut l18: i64 = 0i64;
    let mut l19: i32 = 0i32;
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
    let mut l31: i64 = 0i64;
    let mut l32: i32 = 0i32;
    let mut l33: i32 = 0i32;
    let mut l34: i64 = 0i64;
    let mut l35: i64 = 0i64;
    let mut l36: i64 = 0i64;
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
    let mut s_i64_2: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l1;
    s_i32_1 = vm.get_i32(g0)?;
    vm.enter_call()?;
    let __call_res = Self::s_84059c53f331(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            l2 = s_i32_0;
            s_i32_0 = vm.get_i32(g0)?;
            l3 = s_i32_0;
            s_i32_0 = vm.get_i32(g0)?;
            l4 = s_i32_0;
            s_i32_0 = vm.get_i32(g1)?;
            l5 = s_i32_0;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            l6 = s_i32_0;
            s_i32_0 = l6;
            s_i32_1 = l2;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l6;
            s_i32_1 = l3;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l6;
            s_i32_1 = l4;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l6;
            s_i32_1 = l5;
            ops::i32_store8(vm, 0, s_i32_0, 12u32, s_i32_1)?;
            s_i32_0 = l6;
            l7 = s_i32_0;
            s_i32_0 = l7;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i32_0 = l0;
    s_i32_1 = l1;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_559745ed6085(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t2 = __call_res?;
    s_i32_0 = __t2;
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
            l8 = s_i32_0;
            s_i32_0 = vm.get_i32(g0)?;
            l9 = s_i32_0;
            s_i32_0 = l0;
            l10 = s_i32_0;
            s_i32_0 = 0i32;
            l11 = s_i32_0;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i32_0 = __t3;
            l12 = s_i32_0;
            s_i32_0 = l12;
            s_i32_1 = l8;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l12;
            s_i32_1 = l9;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l12;
            s_i32_1 = l10;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l12;
            s_i32_1 = l11;
            ops::i32_store8(vm, 0, s_i32_0, 12u32, s_i32_1)?;
            s_i32_0 = l12;
            l13 = s_i32_0;
            s_i32_0 = l13;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i32_0 = l0;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
    s_i64_1 = 0i64;
    s_i32_0 = ops::i64_eq(s_i64_0, s_i64_1);
    'blk_3: {
        if s_i32_0 != 0 {
            s_i32_0 = l1;
            s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
            s_i64_1 = 0i64;
            s_i32_0 = ops::i64_eq(s_i64_0, s_i64_1);
        } else {
            s_i32_0 = 0i32;
        }
    }
    'blk_4: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 0u32)?;
            l14 = s_i64_0;
            s_i32_0 = l1;
            s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 0u32)?;
            l15 = s_i64_0;
            s_i32_0 = 1i32;
            l16 = s_i32_0;
            s_i64_0 = l14;
            s_i64_1 = l15;
            s_i64_0 = ops::i64_div_u(s_i64_0, s_i64_1)?;
            l17 = s_i64_0;
            s_i64_0 = 0i64;
            l18 = s_i64_0;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t4 = __call_res?;
            s_i32_0 = __t4;
            l19 = s_i32_0;
            s_i32_0 = l19;
            s_i64_1 = l17;
            ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
            s_i32_0 = l19;
            s_i64_1 = l18;
            ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
            s_i32_0 = l19;
            l20 = s_i32_0;
            s_i64_0 = l14;
            s_i64_1 = l15;
            s_i64_0 = ops::i64_rem_u(s_i64_0, s_i64_1)?;
            l21 = s_i64_0;
            s_i64_0 = 0i64;
            l22 = s_i64_0;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t5 = __call_res?;
            s_i32_0 = __t5;
            l23 = s_i32_0;
            s_i32_0 = l23;
            s_i64_1 = l21;
            ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
            s_i32_0 = l23;
            s_i64_1 = l22;
            ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
            s_i32_0 = l23;
            l24 = s_i32_0;
            s_i32_0 = 0i32;
            l25 = s_i32_0;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t6 = __call_res?;
            s_i32_0 = __t6;
            l26 = s_i32_0;
            s_i32_0 = l26;
            s_i32_1 = l16;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l26;
            s_i32_1 = l20;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l26;
            s_i32_1 = l24;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l26;
            s_i32_1 = l25;
            ops::i32_store8(vm, 0, s_i32_0, 12u32, s_i32_1)?;
            s_i32_0 = l26;
            l27 = s_i32_0;
            s_i32_0 = l27;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i32_0 = vm.get_i32(g0)?;
    l28 = s_i32_0;
    s_i32_0 = vm.get_i32(g0)?;
    l29 = s_i32_0;
    s_i32_0 = 127i32;
    l30 = s_i32_0;
    s_i64_0 = i64::MIN;
    l31 = s_i64_0;
    'blk_5: {
        'blk_6: loop {
            vm.deduct(host, 1)?;
            s_i32_0 = l30;
            s_i32_1 = 0i32;
            s_i32_0 = ops::i32_ge_s(s_i32_0, s_i32_1);
            s_i32_0 = ops::i32_eqz(s_i32_0);
            if s_i32_0 != 0 {
                break 'blk_5;
            }
            s_i32_0 = l30;
            vm.enter_call()?;
            let __call_res = Self::s_d41cf468c215(vm, s_i32_0, host);
            vm.exit_call();
            let __t7 = __call_res?;
            s_i32_0 = __t7;
            l32 = s_i32_0;
            s_i32_0 = l29;
            s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
            s_i64_1 = l31;
            s_i64_0 = ops::i64_and(s_i64_0, s_i64_1);
            s_i64_1 = 0i64;
            s_i32_0 = ops::i64_ne(s_i64_0, s_i64_1);
            l33 = s_i32_0;
            s_i32_0 = l0;
            s_i32_1 = l32;
            vm.enter_call()?;
            let __call_res = Self::s_94ee13bac3fe(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t8 = __call_res?;
            s_i64_0 = __t8;
            l34 = s_i64_0;
            s_i32_0 = l29;
            s_i32_1 = 1i32;
            vm.enter_call()?;
            let __call_res = Self::s_fa6af9f06252(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t9 = __call_res?;
            s_i32_0 = __t9;
            s_i64_1 = l34;
            l35 = s_i64_1;
            s_i64_1 = 0i64;
            l36 = s_i64_1;
            s_i32_1 = 16i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_1, host);
            vm.exit_call();
            let __t10 = __call_res?;
            s_i32_1 = __t10;
            l37 = s_i32_1;
            s_i32_1 = l37;
            s_i64_2 = l35;
            ops::i64_store(vm, 0, s_i32_1, 0u32, s_i64_2)?;
            s_i32_1 = l37;
            s_i64_2 = l36;
            ops::i64_store(vm, 0, s_i32_1, 8u32, s_i64_2)?;
            s_i32_1 = l37;
            vm.enter_call()?;
            let __call_res = Self::s_9c006b2946a7(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t11 = __call_res?;
            s_i32_0 = __t11;
            l29 = s_i32_0;
            s_i32_0 = l33;
            'blk_7: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                } else {
                    s_i32_0 = l29;
                    s_i32_1 = l1;
                    vm.enter_call()?;
                    let __call_res = Self::s_c92ee68c0844(vm, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t12 = __call_res?;
                    s_i32_0 = __t12;
                }
            }
            'blk_8: {
                if s_i32_0 != 0 {
                    s_i32_0 = l29;
                    s_i32_1 = l1;
                    vm.enter_call()?;
                    let __call_res = Self::s_95ed5cb908ad(vm, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t13 = __call_res?;
                    s_i32_0 = __t13;
                    l29 = s_i32_0;
                    s_i32_0 = l28;
                    s_i32_1 = l32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_7a74d4a23901(vm, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t14 = __call_res?;
                    s_i32_0 = __t14;
                    l28 = s_i32_0;
                } else {
                }
            }
            s_i32_0 = l30;
            s_i32_1 = 1i32;
            vm.enter_call()?;
            let __call_res = Self::s_abb2b8d6397f(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t15 = __call_res?;
            s_i32_0 = __t15;
            l30 = s_i32_0;
            continue 'blk_6;
            break 'blk_6;
        }
    }
    s_i32_0 = 1i32;
    l38 = s_i32_0;
    s_i32_0 = l28;
    l39 = s_i32_0;
    s_i32_0 = l29;
    l40 = s_i32_0;
    s_i32_0 = 0i32;
    l41 = s_i32_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
    vm.exit_call();
    let __t16 = __call_res?;
    s_i32_0 = __t16;
    l42 = s_i32_0;
    s_i32_0 = l42;
    s_i32_1 = l38;
    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l42;
    s_i32_1 = l39;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l42;
    s_i32_1 = l40;
    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
    s_i32_0 = l42;
    s_i32_1 = l41;
    ops::i32_store8(vm, 0, s_i32_0, 12u32, s_i32_1)?;
    s_i32_0 = l42;
    l43 = s_i32_0;
    s_i32_0 = l43;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_c0dc152a2cfa(vm: &mut SubstituteVm, p0: i64, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i64 = p0;
    let mut l1: i32 = 0i32;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i32_2: i32 = 0i32;
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
    s_i32_2 = 3i32;
    s_i32_1 = ops::i32_shl(s_i32_1, s_i32_2);
    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
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

    pub(crate) fn s_c31be4b7b29d(vm: &mut SubstituteVm, p0: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
    s_i64_1 = 0i64;
    s_i32_0 = ops::i64_eq(s_i64_0, s_i64_1);
    l1 = s_i32_0;
    s_i32_0 = l1;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_c463b55dc5fa(vm: &mut SubstituteVm, p0: i64, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i64, WasmTrap> {
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
    s_i64_0 = ops::i64_shr_s(s_i64_0, s_i64_1);
    return Ok(s_i64_0);
    }

    pub(crate) fn s_c55df26c3956(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, g1: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let __call_res = Self::s_bb055f113709(vm, s_i32_0, s_i32_1, g0, g1, host);
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

    pub(crate) fn s_c5da14f612f0(vm: &mut SubstituteVm, p0: i64, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i64 = p0;
    let mut l1: i32 = 0i32;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i32_2: i32 = 0i32;
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
    s_i32_2 = 2i32;
    s_i32_1 = ops::i32_shl(s_i32_1, s_i32_2);
    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
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

    pub(crate) fn s_c6ab0a288667(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    s_i32_0 = l1;
    s_i32_1 = 32i32;
    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = l0;
    s_i32_1 = l1;
    s_i32_0 = ops::i32_shr_u(s_i32_0, s_i32_1);
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn s_c6c34807855c(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, g1: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let __call_res = Self::s_c31be4b7b29d(vm, s_i32_0, host);
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
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
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
    let __call_res = crate::shared_0::Shared::s_4552c7e0e83b(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t2 = __call_res?;
    s_i32_0 = __t2;
    l7 = s_i32_0;
    s_i32_0 = 1i32;
    l8 = s_i32_0;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
    s_i32_1 = l1;
    s_i32_1 = ops::i32_load8_u(vm, 0, s_i32_1, 0u32)?;
    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
    s_i32_1 = l7;
    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 0u32)?;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_482c16ff60d0(vm, s_i32_0, s_i32_1, g0, host);
    vm.exit_call();
    let __t3 = __call_res?;
    s_i32_0 = __t3;
    l9 = s_i32_0;
    s_i32_0 = 0i32;
    l10 = s_i32_0;
    s_i32_0 = 12i32;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
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

    pub(crate) fn s_c7105d8549b1(vm: &mut SubstituteVm, p0: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    s_i64_0 = 2228224i64;
    s_i32_1 = l0;
    s_i64_1 = ops::i64_extend_i32_u(s_i32_1);
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i64_0 = __t0;
    l1 = s_i64_0;
    s_i64_0 = 0i64;
    l2 = s_i64_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
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

    pub(crate) fn s_c7674591188c(vm: &mut SubstituteVm, p0: i32, p1: i64, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i64, WasmTrap> {
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
    s_i32_2 = 3i32;
    s_i32_1 = ops::i32_shl(s_i32_1, s_i32_2);
    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 0u32)?;
    return Ok(s_i64_0);
    }

    pub(crate) fn s_c9242759e3db(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i64 = 0i64;
    let mut l3: i64 = 0i64;
    let mut l4: i64 = 0i64;
    let mut l5: i64 = 0i64;
    let mut l6: i64 = 0i64;
    let mut l7: i32 = 0i32;
    let mut l8: i32 = 0i32;
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
    s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
    l2 = s_i64_0;
    s_i64_0 = l2;
    s_i32_1 = l0;
    s_i64_1 = ops::i64_load(vm, 0, s_i32_1, 0u32)?;
    s_i32_0 = ops::i64_lt_u(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i64_0 = 1i64;
        } else {
            s_i64_0 = 0i64;
        }
    }
    l3 = s_i64_0;
    s_i32_0 = l0;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
    s_i32_1 = l1;
    s_i64_1 = ops::i64_load(vm, 0, s_i32_1, 8u32)?;
    s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
    s_i64_1 = l3;
    s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
    l4 = s_i64_0;
    s_i64_0 = l2;
    l5 = s_i64_0;
    s_i64_0 = l4;
    l6 = s_i64_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l7 = s_i32_0;
    s_i32_0 = l7;
    s_i64_1 = l5;
    ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
    s_i32_0 = l7;
    s_i64_1 = l6;
    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
    s_i32_0 = l7;
    l8 = s_i32_0;
    s_i32_0 = l8;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_c92ee68c0844(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let __call_res = crate::shared_0::Shared::s_559745ed6085(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    s_i32_0 = ops::i32_eqz(s_i32_0);
    l2 = s_i32_0;
    s_i32_0 = l2;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn s_ca373e3834fd(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut l2: i32 = 0i32;
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
    let mut s_i32_2: i32 = 0i32;
    vm.deduct(host, 1)?;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    l2 = s_i32_0;
    s_i32_0 = l1;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    l3 = s_i32_0;
    s_i32_0 = l2;
    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
    s_i32_1 = l3;
    s_i64_1 = ops::i64_extend_i32_u(s_i32_1);
    s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
    s_i64_1 = 2i64;
    s_i64_0 = ops::i64_shl(s_i64_0, s_i64_1);
    l4 = s_i64_0;
    s_i64_0 = l4;
    s_i64_1 = 2147417856i64;
    s_i32_0 = ops::i64_gt_u(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = l2;
    s_i32_1 = l3;
    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
    l5 = s_i32_0;
    s_i32_0 = l2;
    s_i32_1 = 2i32;
    s_i32_0 = ops::i32_shl(s_i32_0, s_i32_1);
    l6 = s_i32_0;
    s_i32_0 = l3;
    s_i32_1 = 2i32;
    s_i32_0 = ops::i32_shl(s_i32_0, s_i32_1);
    l7 = s_i32_0;
    s_i32_0 = 16i32;
    s_i64_1 = l4;
    s_i32_1 = ops::i32_wrap_i64(s_i64_1);
    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l8 = s_i32_0;
    s_i32_0 = l8;
    s_i32_1 = 16i32;
    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
    l9 = s_i32_0;
    s_i32_0 = l6;
    s_i32_1 = 0i32;
    s_i32_0 = ops::i32_gt_u(s_i32_0, s_i32_1);
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = l9;
            s_i32_1 = l0;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 0u32)?;
            s_i32_2 = l6;
            vm.memory_copy(0, 0, s_i32_0, s_i32_1, s_i32_2, host)?;
        }
    }
    s_i32_0 = l7;
    s_i32_1 = 0i32;
    s_i32_0 = ops::i32_gt_u(s_i32_0, s_i32_1);
    'blk_3: {
        if s_i32_0 != 0 {
            s_i32_0 = l9;
            s_i32_1 = l6;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 0u32)?;
            s_i32_2 = l7;
            vm.memory_copy(0, 0, s_i32_0, s_i32_1, s_i32_2, host)?;
        }
    }
    s_i32_0 = l8;
    s_i32_1 = l9;
    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l8;
    s_i32_1 = l5;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l8;
    s_i32_1 = l5;
    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
    s_i32_0 = l8;
    s_i32_1 = 1i32;
    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
    s_i32_0 = l8;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_cc2d1c45c42a(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    s_i32_0 = l1;
    s_i32_1 = 32i32;
    s_i32_0 = ops::i32_ge_u(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = l0;
    s_i32_1 = l1;
    s_i32_0 = ops::i32_shl(s_i32_0, s_i32_1);
    return Ok(s_i32_0);
    }

    pub(crate) fn s_cead6caa4c39(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
            let __call_res = Self::s_fa6af9f06252(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t0 = __call_res?;
            s_i32_0 = __t0;
            s_i32_1 = l0;
            s_i32_2 = 128i32;
            s_i32_3 = l2;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_304f22c4ab8c(vm, s_i32_2, s_i32_3, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_2 = __t1;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_7009a9e7c9fa(vm, s_i32_1, s_i32_2, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_1 = __t2;
            let __t3 = s_i32_0;
            let __t4 = s_i32_1;
            return Self::s_9c006b2946a7(vm, __t3, __t4, host);
        }
    }
    l3 = s_i32_0;
    s_i32_0 = l3;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_d06a332d0c37(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    l2 = s_i32_0;
    s_i32_0 = l1;
    s_i32_1 = vm.get_i32(g0)?;
    vm.enter_call()?;
    let __call_res = Self::s_84059c53f331(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    s_i32_0 = ops::i32_eqz(s_i32_0);
    l3 = s_i32_0;
    s_i32_0 = l3;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_d0a938e2c1d3(vm: &mut SubstituteVm, p0: i32, g0: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = vm.get_i32(g0)?;
    s_i32_1 = l0;
    let __t0 = s_i32_0;
    let __t1 = s_i32_1;
    return Self::s_a6d2220b3671(vm, __t0, __t1, host);
    }

    pub(crate) fn s_d0df0f4c4018(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let __call_res = Self::s_f76e9849d691(vm, s_i32_0, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    s_i32_1 = l1;
    vm.enter_call()?;
    let __call_res = Self::s_f76e9849d691(vm, s_i32_1, host);
    vm.exit_call();
    let __t1 = __call_res?;
    s_i32_1 = __t1;
    vm.enter_call()?;
    let __call_res = Self::s_c9242759e3db(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t2 = __call_res?;
    s_i32_0 = __t2;
    let __t3 = s_i32_0;
    return crate::shared_0::Shared::s_623491ac0af2(vm, __t3, host);
    }

    pub(crate) fn s_d1e390df4346(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let __call_res = crate::shared_0::Shared::s_745aa11cfb74(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    s_i32_0 = ops::i32_eqz(s_i32_0);
    l2 = s_i32_0;
    s_i32_0 = l2;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_d344c04b540c(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let __call_res = Self::s_84059c53f331(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l2 = s_i32_0;
    s_i32_0 = l2;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_d41cf468c215(vm: &mut SubstituteVm, p0: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut s_i32_0: i32 = 0i32;
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
    return Ok(s_i32_0);
    }

    pub(crate) fn s_d5f50d7e651d(vm: &mut SubstituteVm, p0: i64, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i64 = p0;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i64_1: i64 = 0i64;
    vm.deduct(host, 1)?;
    s_i64_0 = l0;
    s_i64_1 = 4294967295i64;
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

    #[inline(never)]
    pub(crate) fn s_d7b267099233(vm: &mut SubstituteVm, p0: i32, g0: usize, g1: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
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
    s_i32_0 = l0;
    s_i32_1 = 0i32;
    s_i32_0 = ops::i32_ge_s(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
            l1 = s_i32_0;
            s_i32_0 = l0;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_1596f30da51c(vm, s_i32_0, host);
            vm.exit_call();
            let __t0 = __call_res?;
            s_i64_0 = __t0;
            l2 = s_i64_0;
            s_i64_0 = 0i64;
            l3 = s_i64_0;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
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
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
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
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
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

    pub(crate) fn s_d7d900550897(vm: &mut SubstituteVm, p0: i32, p1: i64, p2: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<(), WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i64 = p1;
    let mut l2: i32 = p2;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
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
    s_i64_0 = l1;
    s_i32_1 = l0;
    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
    s_i64_1 = ops::i64_extend_i32_u(s_i32_1);
    s_i32_0 = ops::i64_ge_u(s_i64_0, s_i64_1);
    'blk_2: {
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
    s_i32_1 = l2;
    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    return Ok(());
    }

    #[inline(never)]
    pub(crate) fn s_d8316e1823a7(vm: &mut SubstituteVm, p0: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i64 = 0i64;
    let mut l2: i32 = 0i32;
    let mut l3: i64 = 0i64;
    let mut l4: i32 = 0i32;
    let mut l5: i64 = 0i64;
    let mut l6: i64 = 0i64;
    let mut l7: i32 = 0i32;
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
    let mut l22: i64 = 0i64;
    let mut l23: i32 = 0i32;
    let mut l24: i32 = 0i32;
    let mut l25: i32 = 0i32;
    let mut l26: i32 = 0i32;
    let mut l27: i32 = 0i32;
    let mut l28: i64 = 0i64;
    let mut l29: i64 = 0i64;
    let mut l30: i32 = 0i32;
    let mut l31: i64 = 0i64;
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
    let mut l46: i64 = 0i64;
    let mut l47: i32 = 0i32;
    let mut l48: i32 = 0i32;
    let mut l49: i32 = 0i32;
    let mut l50: i32 = 0i32;
    let mut l51: i32 = 0i32;
    let mut l52: i64 = 0i64;
    let mut l53: i64 = 0i64;
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
    s_i32_1 = 2i32;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_73a866edda27(vm, s_i64_0, s_i32_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i64_0 = __t0;
    vm.enter_call()?;
    let __call_res = Self::s_c5da14f612f0(vm, s_i64_0, host);
    vm.exit_call();
    let __t1 = __call_res?;
    s_i32_0 = __t1;
    l2 = s_i32_0;
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
            let __call_res = crate::shared_0::Shared::s_4e6fb2273c0f(vm, s_i32_0, s_i64_1, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_0 = __t2;
            l4 = s_i32_0;
            s_i32_0 = l4;
            s_i32_1 = 128i32;
            vm.enter_call()?;
            let __call_res = Self::s_8b082d5ac52c(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i32_0 = __t3;
            'blk_3: {
                if s_i32_0 != 0 {
                    s_i32_0 = l2;
                    s_i32_1 = l4;
                    s_i32_2 = 255i32;
                    s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
                    s_i64_1 = ops::i64_extend_i32_u(s_i32_1);
                    l5 = s_i64_1;
                    s_i64_1 = 0i64;
                    l6 = s_i64_1;
                    s_i32_1 = 16i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_1, host);
                    vm.exit_call();
                    let __t4 = __call_res?;
                    s_i32_1 = __t4;
                    l7 = s_i32_1;
                    s_i32_1 = l7;
                    s_i64_2 = l5;
                    ops::i64_store(vm, 0, s_i32_1, 0u32, s_i64_2)?;
                    s_i32_1 = l7;
                    s_i64_2 = l6;
                    ops::i64_store(vm, 0, s_i32_1, 8u32, s_i64_2)?;
                    s_i32_1 = l7;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_697cb42b9ce7(vm, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                    s_i64_0 = l3;
                    s_i64_1 = 1i64;
                    s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
                    l3 = s_i64_0;
                    continue 'blk_2;
                } else {
                }
            }
            s_i32_0 = l4;
            s_i32_1 = 240i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_7972a1a3c721(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t5 = __call_res?;
            s_i32_0 = __t5;
            'blk_4: {
                if s_i32_0 != 0 {
                    s_i32_0 = l4;
                    s_i32_1 = 247i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_3693c5d73f07(vm, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t6 = __call_res?;
                    s_i32_0 = __t6;
                } else {
                    s_i32_0 = 0i32;
                }
            }
            'blk_5: {
                if s_i32_0 != 0 {
                    s_i64_0 = l3;
                    s_i64_1 = 4i64;
                    s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
                    l8 = s_i64_0;
                    s_i64_0 = l8;
                    s_i64_1 = l1;
                    s_i32_0 = ops::i64_gt_u(s_i64_0, s_i64_1);
                    'blk_6: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 0i32;
                            l9 = s_i32_0;
                            s_i32_0 = 16i32;
                            vm.enter_call()?;
                            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                            vm.exit_call();
                            let __t7 = __call_res?;
                            s_i32_0 = __t7;
                            l10 = s_i32_0;
                            s_i32_0 = l10;
                            s_i32_1 = l10;
                            s_i32_2 = 16i32;
                            s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
                            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                            s_i32_0 = l10;
                            s_i32_1 = 0i32;
                            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                            s_i32_0 = l10;
                            s_i32_1 = 0i32;
                            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                            s_i32_0 = l10;
                            s_i32_1 = 1i32;
                            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                            s_i32_0 = l10;
                            l11 = s_i32_0;
                            s_i32_0 = 8i32;
                            vm.enter_call()?;
                            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                            vm.exit_call();
                            let __t8 = __call_res?;
                            s_i32_0 = __t8;
                            l12 = s_i32_0;
                            s_i32_0 = l12;
                            s_i32_1 = l9;
                            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                            s_i32_0 = l12;
                            s_i32_1 = l11;
                            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                            s_i32_0 = l12;
                            l13 = s_i32_0;
                            s_i32_0 = l13;
                            return Ok(s_i32_0);
                        } else {
                        }
                    }
                    s_i32_0 = l0;
                    s_i64_1 = l3;
                    s_i64_2 = 1i64;
                    s_i64_1 = ops::i64_add(s_i64_1, s_i64_2);
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_4e6fb2273c0f(vm, s_i32_0, s_i64_1, host);
                    vm.exit_call();
                    let __t9 = __call_res?;
                    s_i32_0 = __t9;
                    l14 = s_i32_0;
                    s_i32_0 = l0;
                    s_i64_1 = l3;
                    s_i64_2 = 2i64;
                    s_i64_1 = ops::i64_add(s_i64_1, s_i64_2);
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_4e6fb2273c0f(vm, s_i32_0, s_i64_1, host);
                    vm.exit_call();
                    let __t10 = __call_res?;
                    s_i32_0 = __t10;
                    l15 = s_i32_0;
                    s_i32_0 = l0;
                    s_i64_1 = l3;
                    s_i64_2 = 3i64;
                    s_i64_1 = ops::i64_add(s_i64_1, s_i64_2);
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_4e6fb2273c0f(vm, s_i32_0, s_i64_1, host);
                    vm.exit_call();
                    let __t11 = __call_res?;
                    s_i32_0 = __t11;
                    l16 = s_i32_0;
                    s_i32_0 = l14;
                    s_i32_1 = 192i32;
                    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                    s_i32_1 = 255i32;
                    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                    s_i32_1 = 128i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_3e7647da0841(vm, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t12 = __call_res?;
                    s_i32_0 = __t12;
                    'blk_7: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 1i32;
                        } else {
                            s_i32_0 = l15;
                            s_i32_1 = 192i32;
                            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                            s_i32_1 = 255i32;
                            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                            s_i32_1 = 128i32;
                            vm.enter_call()?;
                            let __call_res = crate::shared_0::Shared::s_3e7647da0841(vm, s_i32_0, s_i32_1, host);
                            vm.exit_call();
                            let __t13 = __call_res?;
                            s_i32_0 = __t13;
                            'blk_8: {
                                if s_i32_0 != 0 {
                                    s_i32_0 = 1i32;
                                } else {
                                    s_i32_0 = l16;
                                    s_i32_1 = 192i32;
                                    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                                    s_i32_1 = 255i32;
                                    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                                    s_i32_1 = 128i32;
                                    vm.enter_call()?;
                                    let __call_res = crate::shared_0::Shared::s_3e7647da0841(vm, s_i32_0, s_i32_1, host);
                                    vm.exit_call();
                                    let __t14 = __call_res?;
                                    s_i32_0 = __t14;
                                }
                            }
                        }
                    }
                    'blk_9: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 0i32;
                            l17 = s_i32_0;
                            s_i32_0 = 16i32;
                            vm.enter_call()?;
                            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                            vm.exit_call();
                            let __t15 = __call_res?;
                            s_i32_0 = __t15;
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
                            s_i32_0 = 8i32;
                            vm.enter_call()?;
                            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                            vm.exit_call();
                            let __t16 = __call_res?;
                            s_i32_0 = __t16;
                            l20 = s_i32_0;
                            s_i32_0 = l20;
                            s_i32_1 = l17;
                            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                            s_i32_0 = l20;
                            s_i32_1 = l19;
                            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                            s_i32_0 = l20;
                            l21 = s_i32_0;
                            s_i32_0 = l21;
                            return Ok(s_i32_0);
                        } else {
                        }
                    }
                    s_i32_0 = l4;
                    s_i32_1 = 7i32;
                    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                    s_i32_1 = 255i32;
                    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                    s_i32_1 = 255i32;
                    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
                    s_i32_1 = 18i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_52dcb6d97135(vm, s_i64_0, s_i32_1, host);
                    vm.exit_call();
                    let __t17 = __call_res?;
                    s_i64_0 = __t17;
                    s_i32_1 = l14;
                    s_i32_2 = 63i32;
                    s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
                    s_i32_2 = 255i32;
                    s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
                    s_i32_2 = 255i32;
                    s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
                    s_i64_1 = ops::i64_extend_i32_u(s_i32_1);
                    s_i32_2 = 12i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_52dcb6d97135(vm, s_i64_1, s_i32_2, host);
                    vm.exit_call();
                    let __t18 = __call_res?;
                    s_i64_1 = __t18;
                    s_i32_2 = l15;
                    s_i32_3 = 63i32;
                    s_i32_2 = ops::i32_and(s_i32_2, s_i32_3);
                    s_i32_3 = 255i32;
                    s_i32_2 = ops::i32_and(s_i32_2, s_i32_3);
                    s_i32_3 = 255i32;
                    s_i32_2 = ops::i32_and(s_i32_2, s_i32_3);
                    s_i64_2 = ops::i64_extend_i32_u(s_i32_2);
                    s_i32_3 = 6i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_52dcb6d97135(vm, s_i64_2, s_i32_3, host);
                    vm.exit_call();
                    let __t19 = __call_res?;
                    s_i64_2 = __t19;
                    s_i32_3 = l16;
                    s_i32_4 = 63i32;
                    s_i32_3 = ops::i32_and(s_i32_3, s_i32_4);
                    s_i32_4 = 255i32;
                    s_i32_3 = ops::i32_and(s_i32_3, s_i32_4);
                    s_i32_4 = 255i32;
                    s_i32_3 = ops::i32_and(s_i32_3, s_i32_4);
                    s_i64_3 = ops::i64_extend_i32_u(s_i32_3);
                    s_i64_2 = ops::i64_or(s_i64_2, s_i64_3);
                    s_i64_1 = ops::i64_or(s_i64_1, s_i64_2);
                    s_i64_0 = ops::i64_or(s_i64_0, s_i64_1);
                    l22 = s_i64_0;
                    s_i64_0 = l22;
                    s_i64_1 = 65536i64;
                    s_i32_0 = ops::i64_lt_u(s_i64_0, s_i64_1);
                    'blk_10: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 0i32;
                            l23 = s_i32_0;
                            s_i32_0 = 16i32;
                            vm.enter_call()?;
                            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                            vm.exit_call();
                            let __t20 = __call_res?;
                            s_i32_0 = __t20;
                            l24 = s_i32_0;
                            s_i32_0 = l24;
                            s_i32_1 = l24;
                            s_i32_2 = 16i32;
                            s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
                            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                            s_i32_0 = l24;
                            s_i32_1 = 0i32;
                            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                            s_i32_0 = l24;
                            s_i32_1 = 0i32;
                            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                            s_i32_0 = l24;
                            s_i32_1 = 1i32;
                            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                            s_i32_0 = l24;
                            l25 = s_i32_0;
                            s_i32_0 = 8i32;
                            vm.enter_call()?;
                            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                            vm.exit_call();
                            let __t21 = __call_res?;
                            s_i32_0 = __t21;
                            l26 = s_i32_0;
                            s_i32_0 = l26;
                            s_i32_1 = l23;
                            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                            s_i32_0 = l26;
                            s_i32_1 = l25;
                            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                            s_i32_0 = l26;
                            l27 = s_i32_0;
                            s_i32_0 = l27;
                            return Ok(s_i32_0);
                        } else {
                        }
                    }
                    s_i32_0 = l2;
                    s_i64_1 = l22;
                    l28 = s_i64_1;
                    s_i64_1 = 0i64;
                    l29 = s_i64_1;
                    s_i32_1 = 16i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_1, host);
                    vm.exit_call();
                    let __t22 = __call_res?;
                    s_i32_1 = __t22;
                    l30 = s_i32_1;
                    s_i32_1 = l30;
                    s_i64_2 = l28;
                    ops::i64_store(vm, 0, s_i32_1, 0u32, s_i64_2)?;
                    s_i32_1 = l30;
                    s_i64_2 = l29;
                    vm.deduct(host, 1)?;
                    ops::i64_store(vm, 0, s_i32_1, 8u32, s_i64_2)?;
                    s_i32_1 = l30;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_697cb42b9ce7(vm, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                    s_i64_0 = l8;
                    l3 = s_i64_0;
                    continue 'blk_2;
                } else {
                }
            }
            s_i32_0 = l4;
            s_i32_1 = 248i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_7972a1a3c721(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t23 = __call_res?;
            s_i32_0 = __t23;
            'blk_11: {
                if s_i32_0 != 0 {
                    s_i32_0 = l4;
                    s_i32_1 = 251i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_3693c5d73f07(vm, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t24 = __call_res?;
                    s_i32_0 = __t24;
                } else {
                    s_i32_0 = 0i32;
                }
            }
            'blk_12: {
                if s_i32_0 != 0 {
                    s_i64_0 = l3;
                    s_i64_1 = 5i64;
                    s_i64_0 = ops::i64_add(s_i64_0, s_i64_1);
                    l31 = s_i64_0;
                    s_i64_0 = l31;
                    s_i64_1 = l1;
                    s_i32_0 = ops::i64_gt_u(s_i64_0, s_i64_1);
                    'blk_13: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 0i32;
                            l32 = s_i32_0;
                            s_i32_0 = 16i32;
                            vm.enter_call()?;
                            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                            vm.exit_call();
                            let __t25 = __call_res?;
                            s_i32_0 = __t25;
                            l33 = s_i32_0;
                            s_i32_0 = l33;
                            s_i32_1 = l33;
                            s_i32_2 = 16i32;
                            s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
                            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                            s_i32_0 = l33;
                            s_i32_1 = 0i32;
                            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                            s_i32_0 = l33;
                            s_i32_1 = 0i32;
                            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                            s_i32_0 = l33;
                            s_i32_1 = 1i32;
                            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                            s_i32_0 = l33;
                            l34 = s_i32_0;
                            s_i32_0 = 8i32;
                            vm.enter_call()?;
                            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                            vm.exit_call();
                            let __t26 = __call_res?;
                            s_i32_0 = __t26;
                            l35 = s_i32_0;
                            s_i32_0 = l35;
                            s_i32_1 = l32;
                            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                            s_i32_0 = l35;
                            s_i32_1 = l34;
                            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                            s_i32_0 = l35;
                            l36 = s_i32_0;
                            s_i32_0 = l36;
                            return Ok(s_i32_0);
                        } else {
                        }
                    }
                    s_i32_0 = l0;
                    s_i64_1 = l3;
                    s_i64_2 = 1i64;
                    s_i64_1 = ops::i64_add(s_i64_1, s_i64_2);
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_4e6fb2273c0f(vm, s_i32_0, s_i64_1, host);
                    vm.exit_call();
                    let __t27 = __call_res?;
                    s_i32_0 = __t27;
                    l37 = s_i32_0;
                    s_i32_0 = l0;
                    s_i64_1 = l3;
                    s_i64_2 = 2i64;
                    s_i64_1 = ops::i64_add(s_i64_1, s_i64_2);
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_4e6fb2273c0f(vm, s_i32_0, s_i64_1, host);
                    vm.exit_call();
                    let __t28 = __call_res?;
                    s_i32_0 = __t28;
                    l38 = s_i32_0;
                    s_i32_0 = l0;
                    s_i64_1 = l3;
                    s_i64_2 = 3i64;
                    s_i64_1 = ops::i64_add(s_i64_1, s_i64_2);
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_4e6fb2273c0f(vm, s_i32_0, s_i64_1, host);
                    vm.exit_call();
                    let __t29 = __call_res?;
                    s_i32_0 = __t29;
                    l39 = s_i32_0;
                    s_i32_0 = l0;
                    s_i64_1 = l3;
                    s_i64_2 = 4i64;
                    s_i64_1 = ops::i64_add(s_i64_1, s_i64_2);
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_4e6fb2273c0f(vm, s_i32_0, s_i64_1, host);
                    vm.exit_call();
                    let __t30 = __call_res?;
                    s_i32_0 = __t30;
                    l40 = s_i32_0;
                    s_i32_0 = l37;
                    s_i32_1 = 192i32;
                    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                    s_i32_1 = 255i32;
                    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                    s_i32_1 = 128i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_3e7647da0841(vm, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t31 = __call_res?;
                    s_i32_0 = __t31;
                    'blk_14: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 1i32;
                        } else {
                            s_i32_0 = l38;
                            s_i32_1 = 192i32;
                            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                            s_i32_1 = 255i32;
                            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                            s_i32_1 = 128i32;
                            vm.enter_call()?;
                            let __call_res = crate::shared_0::Shared::s_3e7647da0841(vm, s_i32_0, s_i32_1, host);
                            vm.exit_call();
                            let __t32 = __call_res?;
                            s_i32_0 = __t32;
                            'blk_15: {
                                if s_i32_0 != 0 {
                                    s_i32_0 = 1i32;
                                } else {
                                    s_i32_0 = l39;
                                    s_i32_1 = 192i32;
                                    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                                    s_i32_1 = 255i32;
                                    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                                    s_i32_1 = 128i32;
                                    vm.enter_call()?;
                                    let __call_res = crate::shared_0::Shared::s_3e7647da0841(vm, s_i32_0, s_i32_1, host);
                                    vm.exit_call();
                                    let __t33 = __call_res?;
                                    s_i32_0 = __t33;
                                    'blk_16: {
                                        if s_i32_0 != 0 {
                                            s_i32_0 = 1i32;
                                        } else {
                                            s_i32_0 = l40;
                                            s_i32_1 = 192i32;
                                            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                                            s_i32_1 = 255i32;
                                            s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                                            s_i32_1 = 128i32;
                                            vm.enter_call()?;
                                            let __call_res = crate::shared_0::Shared::s_3e7647da0841(vm, s_i32_0, s_i32_1, host);
                                            vm.exit_call();
                                            let __t34 = __call_res?;
                                            s_i32_0 = __t34;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    'blk_17: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 0i32;
                            l41 = s_i32_0;
                            s_i32_0 = 16i32;
                            vm.enter_call()?;
                            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                            vm.exit_call();
                            let __t35 = __call_res?;
                            s_i32_0 = __t35;
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
                            s_i32_0 = 8i32;
                            vm.enter_call()?;
                            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                            vm.exit_call();
                            let __t36 = __call_res?;
                            s_i32_0 = __t36;
                            l44 = s_i32_0;
                            s_i32_0 = l44;
                            s_i32_1 = l41;
                            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                            s_i32_0 = l44;
                            s_i32_1 = l43;
                            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                            s_i32_0 = l44;
                            l45 = s_i32_0;
                            s_i32_0 = l45;
                            return Ok(s_i32_0);
                        } else {
                        }
                    }
                    s_i32_0 = l4;
                    s_i32_1 = 3i32;
                    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                    s_i32_1 = 255i32;
                    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                    s_i32_1 = 255i32;
                    s_i32_0 = ops::i32_and(s_i32_0, s_i32_1);
                    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
                    s_i32_1 = 24i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_52dcb6d97135(vm, s_i64_0, s_i32_1, host);
                    vm.exit_call();
                    let __t37 = __call_res?;
                    s_i64_0 = __t37;
                    s_i32_1 = l37;
                    s_i32_2 = 63i32;
                    s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
                    s_i32_2 = 255i32;
                    s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
                    s_i32_2 = 255i32;
                    s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
                    s_i64_1 = ops::i64_extend_i32_u(s_i32_1);
                    s_i32_2 = 18i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_52dcb6d97135(vm, s_i64_1, s_i32_2, host);
                    vm.exit_call();
                    let __t38 = __call_res?;
                    s_i64_1 = __t38;
                    s_i32_2 = l38;
                    s_i32_3 = 63i32;
                    s_i32_2 = ops::i32_and(s_i32_2, s_i32_3);
                    s_i32_3 = 255i32;
                    s_i32_2 = ops::i32_and(s_i32_2, s_i32_3);
                    s_i32_3 = 255i32;
                    s_i32_2 = ops::i32_and(s_i32_2, s_i32_3);
                    s_i64_2 = ops::i64_extend_i32_u(s_i32_2);
                    s_i32_3 = 12i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_52dcb6d97135(vm, s_i64_2, s_i32_3, host);
                    vm.exit_call();
                    let __t39 = __call_res?;
                    s_i64_2 = __t39;
                    s_i32_3 = l39;
                    s_i32_4 = 63i32;
                    s_i32_3 = ops::i32_and(s_i32_3, s_i32_4);
                    s_i32_4 = 255i32;
                    s_i32_3 = ops::i32_and(s_i32_3, s_i32_4);
                    s_i32_4 = 255i32;
                    s_i32_3 = ops::i32_and(s_i32_3, s_i32_4);
                    s_i64_3 = ops::i64_extend_i32_u(s_i32_3);
                    s_i32_4 = 6i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_52dcb6d97135(vm, s_i64_3, s_i32_4, host);
                    vm.exit_call();
                    let __t40 = __call_res?;
                    s_i64_3 = __t40;
                    s_i32_4 = l40;
                    s_i32_5 = 63i32;
                    s_i32_4 = ops::i32_and(s_i32_4, s_i32_5);
                    s_i32_5 = 255i32;
                    s_i32_4 = ops::i32_and(s_i32_4, s_i32_5);
                    s_i32_5 = 255i32;
                    s_i32_4 = ops::i32_and(s_i32_4, s_i32_5);
                    s_i64_4 = ops::i64_extend_i32_u(s_i32_4);
                    s_i64_3 = ops::i64_or(s_i64_3, s_i64_4);
                    s_i64_2 = ops::i64_or(s_i64_2, s_i64_3);
                    s_i64_1 = ops::i64_or(s_i64_1, s_i64_2);
                    s_i64_0 = ops::i64_or(s_i64_0, s_i64_1);
                    l46 = s_i64_0;
                    s_i64_0 = l46;
                    s_i64_1 = 2097152i64;
                    s_i32_0 = ops::i64_lt_u(s_i64_0, s_i64_1);
                    'blk_18: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 0i32;
                            l47 = s_i32_0;
                            s_i32_0 = 16i32;
                            vm.enter_call()?;
                            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                            vm.exit_call();
                            let __t41 = __call_res?;
                            s_i32_0 = __t41;
                            l48 = s_i32_0;
                            s_i32_0 = l48;
                            s_i32_1 = l48;
                            s_i32_2 = 16i32;
                            s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
                            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                            s_i32_0 = l48;
                            s_i32_1 = 0i32;
                            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                            s_i32_0 = l48;
                            vm.deduct(host, 1)?;
                            s_i32_1 = 0i32;
                            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                            s_i32_0 = l48;
                            s_i32_1 = 1i32;
                            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                            s_i32_0 = l48;
                            l49 = s_i32_0;
                            s_i32_0 = 8i32;
                            vm.enter_call()?;
                            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                            vm.exit_call();
                            let __t42 = __call_res?;
                            s_i32_0 = __t42;
                            l50 = s_i32_0;
                            s_i32_0 = l50;
                            s_i32_1 = l47;
                            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                            s_i32_0 = l50;
                            s_i32_1 = l49;
                            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                            s_i32_0 = l50;
                            l51 = s_i32_0;
                            s_i32_0 = l51;
                            return Ok(s_i32_0);
                        } else {
                        }
                    }
                    s_i32_0 = l2;
                    s_i64_1 = l46;
                    l52 = s_i64_1;
                    s_i64_1 = 0i64;
                    l53 = s_i64_1;
                    s_i32_1 = 16i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_1, host);
                    vm.exit_call();
                    let __t43 = __call_res?;
                    s_i32_1 = __t43;
                    l54 = s_i32_1;
                    s_i32_1 = l54;
                    s_i64_2 = l52;
                    ops::i64_store(vm, 0, s_i32_1, 0u32, s_i64_2)?;
                    s_i32_1 = l54;
                    s_i64_2 = l53;
                    ops::i64_store(vm, 0, s_i32_1, 8u32, s_i64_2)?;
                    vm.deduct(host, 1)?;
                    s_i32_1 = l54;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_697cb42b9ce7(vm, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                    s_i64_0 = l31;
                    l3 = s_i64_0;
                    continue 'blk_2;
                } else {
                }
            }
            s_i32_0 = l0;
            s_i64_1 = l3;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_5d431d0f77d6(vm, s_i32_0, s_i64_1, host);
            vm.exit_call();
            let __t44 = __call_res?;
            s_i32_0 = __t44;
            l55 = s_i32_0;
            s_i32_0 = l55;
            s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
            s_i32_0 = ops::i32_eqz(s_i32_0);
            'blk_19: {
                if s_i32_0 != 0 {
                    s_i32_0 = 0i32;
                    l56 = s_i32_0;
                    s_i32_0 = 16i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                    vm.exit_call();
                    let __t45 = __call_res?;
                    s_i32_0 = __t45;
                    l57 = s_i32_0;
                    s_i32_0 = l57;
                    s_i32_1 = l57;
                    s_i32_2 = 16i32;
                    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l57;
                    s_i32_1 = 0i32;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l57;
                    s_i32_1 = 0i32;
                    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                    s_i32_0 = l57;
                    s_i32_1 = 1i32;
                    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                    s_i32_0 = l57;
                    l58 = s_i32_0;
                    s_i32_0 = 8i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                    vm.exit_call();
                    let __t46 = __call_res?;
                    s_i32_0 = __t46;
                    l59 = s_i32_0;
                    s_i32_0 = l59;
                    s_i32_1 = l56;
                    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l59;
                    s_i32_1 = l58;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l59;
                    l60 = s_i32_0;
                    s_i32_0 = l60;
                    return Ok(s_i32_0);
                } else {
                }
            }
            s_i32_0 = l2;
            s_i32_1 = l55;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_697cb42b9ce7(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l55;
            s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
            l3 = s_i64_0;
            continue 'blk_2;
            break 'blk_2;
        }
    }
    s_i32_0 = 1i32;
    l61 = s_i32_0;
    s_i32_0 = l2;
    vm.enter_call()?;
    let __call_res = Self::s_df3679579582(vm, s_i32_0, host);
    vm.exit_call();
    let __t47 = __call_res?;
    s_i32_0 = __t47;
    l62 = s_i32_0;
    s_i32_0 = 8i32;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
    vm.exit_call();
    let __t48 = __call_res?;
    s_i32_0 = __t48;
    l63 = s_i32_0;
    s_i32_0 = l63;
    s_i32_1 = l61;
    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l63;
    s_i32_1 = l62;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l63;
    l64 = s_i32_0;
    s_i32_0 = l64;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_d8f7034c6f81(vm: &mut SubstituteVm, p0: i32, g0: usize, g1: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let __call_res = crate::shared_0::Shared::s_4200fb7655b9(vm, s_i32_0, g0, g1, host);
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
            s_i32_0 = vm.get_i32(g1)?;
            l4 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
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
            return crate::shared_0::Shared::s_289c1ec0e82a(vm, __t2, g1, host);
        }
    }
    l6 = s_i32_0;
    s_i32_0 = l6;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_d9ebd9898b89(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let __t0 = s_i32_0;
    let __t1 = s_i32_1;
    return Self::s_c92ee68c0844(vm, __t0, __t1, host);
    }

    #[inline(never)]
    pub(crate) fn s_dbc761d7befd(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, g1: usize, g2: usize, g3: usize, g4: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let __call_res = Self::s_a82938aed706(vm, s_i32_0, s_i32_1, g0, g1, g2, g3, g4, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l2 = s_i32_0;
    s_i32_0 = l2;
    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
    s_i32_0 = ops::i32_eqz(s_i32_0);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = l2;
            l3 = s_i32_0;
            s_i32_0 = l3;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i32_0 = l2;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    l4 = s_i32_0;
    s_i32_0 = l4;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_51f0c0b4c723(vm, s_i32_0, host);
    vm.exit_call();
    let __t1 = __call_res?;
    s_i32_0 = __t1;
    s_i32_0 = ops::i32_eqz(s_i32_0);
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
            l5 = s_i32_0;
            s_i32_0 = l4;
            l6 = s_i32_0;
            s_i32_0 = 0i32;
            l7 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
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
        } else {
            s_i32_0 = l1;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_51f0c0b4c723(vm, s_i32_0, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i32_0 = __t3;
            s_i32_0 = ops::i32_eqz(s_i32_0);
            'blk_3: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                    l9 = s_i32_0;
                    s_i32_0 = l4;
                    s_i32_1 = l1;
                    vm.enter_call()?;
                    let __call_res = Self::s_d0df0f4c4018(vm, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t4 = __call_res?;
                    s_i32_0 = __t4;
                    l10 = s_i32_0;
                    s_i32_0 = 0i32;
                    l11 = s_i32_0;
                    s_i32_0 = 12i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                    vm.exit_call();
                    let __t5 = __call_res?;
                    s_i32_0 = __t5;
                    l12 = s_i32_0;
                    s_i32_0 = l12;
                    s_i32_1 = l9;
                    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l12;
                    s_i32_1 = l10;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l12;
                    s_i32_1 = l11;
                    ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                    s_i32_0 = l12;
                } else {
                    s_i32_0 = 1i32;
                    l13 = s_i32_0;
                    s_i32_0 = l4;
                    s_i32_1 = l1;
                    vm.enter_call()?;
                    let __call_res = Self::s_a6d2220b3671(vm, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t6 = __call_res?;
                    s_i32_0 = __t6;
                    l14 = s_i32_0;
                    s_i32_0 = 0i32;
                    l15 = s_i32_0;
                    s_i32_0 = 12i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                    vm.exit_call();
                    let __t7 = __call_res?;
                    s_i32_0 = __t7;
                    l16 = s_i32_0;
                    s_i32_0 = l16;
                    s_i32_1 = l13;
                    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l16;
                    s_i32_1 = l14;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l16;
                    s_i32_1 = l15;
                    ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                    s_i32_0 = l16;
                }
            }
        }
    }
    l17 = s_i32_0;
    s_i32_0 = l17;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn s_de0eb096e7a6(vm: &mut SubstituteVm, p0: i32, p1: i64, g0: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i64 = p1;
    let mut l2: i64 = 0i64;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i64 = 0i64;
    let mut l7: i32 = 0i32;
    let mut l8: i32 = 0i32;
    let mut l9: i32 = 0i32;
    let mut l10: i64 = 0i64;
    let mut l11: i32 = 0i32;
    let mut l12: i32 = 0i32;
    let mut l13: i32 = 0i32;
    let mut l14: i32 = 0i32;
    let mut l15: i64 = 0i64;
    let mut l16: i32 = 0i32;
    let mut l17: i32 = 0i32;
    let mut l18: i32 = 0i32;
    let mut l19: i32 = 0i32;
    let mut l20: i32 = 0i32;
    let mut l21: i32 = 0i32;
    let mut l22: i64 = 0i64;
    let mut l23: i32 = 0i32;
    let mut l24: i32 = 0i32;
    let mut l25: i32 = 0i32;
    let mut l26: i32 = 0i32;
    let mut l27: i32 = 0i32;
    let mut l28: i64 = 0i64;
    let mut l29: i32 = 0i32;
    let mut l30: i32 = 0i32;
    let mut l31: i32 = 0i32;
    let mut l32: i32 = 0i32;
    let mut l33: i32 = 0i32;
    let mut l34: i64 = 0i64;
    let mut l35: i32 = 0i32;
    let mut l36: i32 = 0i32;
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
            s_i32_0 = 1i32;
        } else {
            s_i32_0 = l0;
            s_i64_1 = l1;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
            vm.exit_call();
            let __t0 = __call_res?;
            s_i32_0 = __t0;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_62439e6f6ae7(vm, s_i32_0, g0, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            s_i32_1 = 270i32;
            s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
        }
    }
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            l3 = s_i32_0;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
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
            s_i64_0 = l1;
            l6 = s_i64_0;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i32_0 = __t3;
            l7 = s_i32_0;
            s_i32_0 = l7;
            s_i32_1 = l3;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l7;
            s_i32_1 = l5;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l7;
            s_i64_1 = l6;
            ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
            s_i32_0 = l7;
            l8 = s_i32_0;
            s_i32_0 = l8;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i64_0 = 16i64;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_61878e7d5ba7(vm, s_i64_0, host);
    vm.exit_call();
    let __t4 = __call_res?;
    s_i32_0 = __t4;
    l9 = s_i32_0;
    s_i64_0 = l1;
    s_i64_1 = 1i64;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
    vm.exit_call();
    let __t5 = __call_res?;
    s_i64_0 = __t5;
    l10 = s_i64_0;
    'blk_3: {
        'blk_4: loop {
            vm.deduct(host, 1)?;
            s_i64_0 = l10;
            s_i64_1 = l2;
            s_i32_0 = ops::i64_lt_u(s_i64_0, s_i64_1);
            s_i32_0 = ops::i32_eqz(s_i32_0);
            if s_i32_0 != 0 {
                break 'blk_3;
            }
            s_i32_0 = l0;
            s_i64_1 = l10;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
            vm.exit_call();
            let __t6 = __call_res?;
            s_i32_0 = __t6;
            l11 = s_i32_0;
            s_i32_0 = l11;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_62439e6f6ae7(vm, s_i32_0, g0, host);
            vm.exit_call();
            let __t7 = __call_res?;
            s_i32_0 = __t7;
            l12 = s_i32_0;
            s_i32_0 = l12;
            s_i32_1 = 271i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_5: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                    l13 = s_i32_0;
                    s_i32_0 = l9;
                    vm.enter_call()?;
                    let __call_res = Self::s_df3679579582(vm, s_i32_0, host);
                    vm.exit_call();
                    let __t8 = __call_res?;
                    s_i32_0 = __t8;
                    l14 = s_i32_0;
                    s_i64_0 = l10;
                    s_i64_1 = 1i64;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
                    vm.exit_call();
                    let __t9 = __call_res?;
                    s_i64_0 = __t9;
                    l15 = s_i64_0;
                    s_i32_0 = 16i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
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
                    l17 = s_i32_0;
                    s_i32_0 = l17;
                    return Ok(s_i32_0);
                } else {
                }
            }
            s_i32_0 = l11;
            l18 = s_i32_0;
            s_i32_0 = l12;
            s_i32_1 = 255i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_6: {
                if s_i32_0 != 0 {
                    s_i64_0 = l10;
                    s_i64_1 = 1i64;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
                    vm.exit_call();
                    let __t11 = __call_res?;
                    s_i64_0 = __t11;
                    s_i64_1 = l2;
                    s_i32_0 = ops::i64_ge_u(s_i64_0, s_i64_1);
                    'blk_7: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 0i32;
                            l19 = s_i32_0;
                            s_i32_0 = 16i32;
                            vm.enter_call()?;
                            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                            vm.exit_call();
                            let __t12 = __call_res?;
                            s_i32_0 = __t12;
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
                            s_i64_0 = l1;
                            l22 = s_i64_0;
                            s_i32_0 = 16i32;
                            vm.enter_call()?;
                            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                            vm.exit_call();
                            let __t13 = __call_res?;
                            s_i32_0 = __t13;
                            l23 = s_i32_0;
                            s_i32_0 = l23;
                            s_i32_1 = l19;
                            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                            s_i32_0 = l23;
                            s_i32_1 = l21;
                            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                            s_i32_0 = l23;
                            s_i64_1 = l22;
                            ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
                            s_i32_0 = l23;
                            l24 = s_i32_0;
                            s_i32_0 = l24;
                            return Ok(s_i32_0);
                        } else {
                        }
                    }
                    s_i32_0 = l0;
                    s_i64_1 = l10;
                    s_i64_2 = 1i64;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_1c917789866c(vm, s_i64_1, s_i64_2, host);
                    vm.exit_call();
                    let __t14 = __call_res?;
                    s_i64_1 = __t14;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
                    vm.exit_call();
                    let __t15 = __call_res?;
                    s_i32_0 = __t15;
                    l18 = s_i32_0;
                    s_i64_0 = l10;
                    s_i64_1 = 2i64;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
                    vm.exit_call();
                    let __t16 = __call_res?;
                    s_i64_0 = __t16;
                    l10 = s_i64_0;
                } else {
                    s_i64_0 = l10;
                    s_i64_1 = 1i64;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
                    vm.exit_call();
                    let __t17 = __call_res?;
                    s_i64_0 = __t17;
                    l10 = s_i64_0;
                }
            }
            s_i32_0 = l18;
            s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
            s_i64_1 = 0i64;
            s_i32_0 = ops::i64_ne(s_i64_0, s_i64_1);
            'blk_8: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                } else {
                    s_i32_0 = l18;
                    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 0u32)?;
                    s_i64_1 = 127i64;
                    s_i32_0 = ops::i64_gt_u(s_i64_0, s_i64_1);
                }
            }
            'blk_9: {
                if s_i32_0 != 0 {
                    s_i32_0 = 0i32;
                    l25 = s_i32_0;
                    s_i32_0 = 16i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                    vm.exit_call();
                    let __t18 = __call_res?;
                    s_i32_0 = __t18;
                    l26 = s_i32_0;
                    s_i32_0 = l26;
                    s_i32_1 = l26;
                    s_i32_2 = 16i32;
                    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l26;
                    s_i32_1 = 0i32;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l26;
                    s_i32_1 = 0i32;
                    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                    s_i32_0 = l26;
                    s_i32_1 = 1i32;
                    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                    s_i32_0 = l26;
                    l27 = s_i32_0;
                    s_i64_0 = l1;
                    l28 = s_i64_0;
                    s_i32_0 = 16i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                    vm.exit_call();
                    let __t19 = __call_res?;
                    s_i32_0 = __t19;
                    l29 = s_i32_0;
                    s_i32_0 = l29;
                    s_i32_1 = l25;
                    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l29;
                    s_i32_1 = l27;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l29;
                    s_i64_1 = l28;
                    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
                    s_i32_0 = l29;
                    l30 = s_i32_0;
                    s_i32_0 = l30;
                    return Ok(s_i32_0);
                } else {
                }
            }
            s_i32_0 = l9;
            s_i32_1 = l18;
            s_i64_1 = ops::i64_load(vm, 0, s_i32_1, 0u32)?;
            vm.enter_call()?;
            let __call_res = Self::s_b5672010fcc9(vm, s_i64_1, host);
            vm.exit_call();
            let __t20 = __call_res?;
            s_i32_1 = __t20;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_5daf123ed54b(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            continue 'blk_4;
            break 'blk_4;
        }
    }
    s_i32_0 = 0i32;
    l31 = s_i32_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
    vm.exit_call();
    let __t21 = __call_res?;
    s_i32_0 = __t21;
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
    s_i64_0 = l1;
    l34 = s_i64_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
    vm.exit_call();
    let __t22 = __call_res?;
    s_i32_0 = __t22;
    l35 = s_i32_0;
    s_i32_0 = l35;
    s_i32_1 = l31;
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
    }

    pub(crate) fn s_de56ae80f317(vm: &mut SubstituteVm, p0: i32, p1: i64, p2: i64, host: &mut dyn WasmSubstituteHost) -> std::result::Result<(), WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i64 = p1;
    let mut l2: i64 = p2;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
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
    s_i64_0 = l1;
    s_i32_1 = l0;
    s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 4u32)?;
    s_i64_1 = ops::i64_extend_i32_u(s_i32_1);
    s_i32_0 = ops::i64_ge_u(s_i64_0, s_i64_1);
    'blk_2: {
        if s_i32_0 != 0 {
            return Err(WasmTrap::Unreachable);
        }
    }
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 0u32)?;
    s_i64_1 = l1;
    s_i32_1 = ops::i32_wrap_i64(s_i64_1);
    s_i32_2 = 3i32;
    s_i32_1 = ops::i32_shl(s_i32_1, s_i32_2);
    s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
    s_i64_1 = l2;
    ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
    return Ok(());
    }

    #[inline(never)]
    pub(crate) fn s_de64faaada56(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<(), WasmTrap> {
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
            s_i64_1 = 2147417856i64;
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
                    s_i32_1 = 1073708928i32;
                    s_i32_0 = ops::i32_gt_u(s_i32_0, s_i32_1);
                    'blk_5: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 2147417856i32;
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
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
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
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            s_i32_1 = l1;
            s_i32_1 = ops::i32_load(vm, 0, s_i32_1, 0u32)?;
            s_i32_2 = l2;
            vm.memory_copy(0, 0, s_i32_0, s_i32_1, s_i32_2, host)?;
            s_i32_0 = l0;
            s_i32_1 = l7;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
        }
    }
    return Ok(());
    }

    pub(crate) fn s_df3679579582(vm: &mut SubstituteVm, p0: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
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
    s_i32_1 = 1i32;
    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
    s_i32_0 = l0;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_e0004f9831a9(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, g1: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
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
            s_i32_1 = l1;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_7009a9e7c9fa(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            l7 = s_i32_0;
            s_i32_0 = 0i32;
            l8 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_0 = __t2;
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

    pub(crate) fn s_e04446e5e582(vm: &mut SubstituteVm, p0: i64, g0: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i64 = p0;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i32_2: i32 = 0i32;
    let mut s_i64_2: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = 199i32;
    s_i32_1 = 0i32;
    s_i64_2 = l0;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_41e9bacc9fe3(vm, s_i64_2, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_2 = __t0;
    let __t1 = s_i32_0;
    let __t2 = s_i32_1;
    let __t3 = s_i32_2;
    return crate::shared_0::Shared::s_6e6223e30e05(vm, __t1, __t2, __t3, g0, host);
    }

    pub(crate) fn s_e1d06f25ed5a(vm: &mut SubstituteVm, p0: i32, g0: usize, g1: usize, g2: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    s_i32_1 = vm.get_i32(g0)?;
    vm.enter_call()?;
    let __call_res = Self::s_84059c53f331(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            l1 = s_i32_0;
            s_i32_0 = vm.get_i32(g1)?;
            l2 = s_i32_0;
            s_i32_0 = vm.get_i32(g2)?;
            l3 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
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
            let __call_res = Self::s_d0a938e2c1d3(vm, s_i32_0, g1, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_0 = __t2;
            l6 = s_i32_0;
            s_i32_0 = 0i32;
            l7 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
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

    #[inline(never)]
    pub(crate) fn s_e208ae0aad56(vm: &mut SubstituteVm, p0: i32, g0: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i32_2: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i64_0 = -1i64;
    s_i64_1 = 288230376151711743i64;
    vm.enter_call()?;
    let __call_res = Self::s_b3e6ca84b2de(vm, s_i64_0, s_i64_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l1 = s_i32_0;
    s_i32_0 = vm.get_i32(g0)?;
    l2 = s_i32_0;
    s_i32_0 = l0;
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
                s_i32_0 = l1;
                s_i32_1 = l2;
                vm.enter_call()?;
                let __call_res = Self::s_c92ee68c0844(vm, s_i32_0, s_i32_1, host);
                vm.exit_call();
                let __t1 = __call_res?;
                s_i32_0 = __t1;
                s_i32_0 = ops::i32_eqz(s_i32_0);
                'blk_4: {
                    if s_i32_0 != 0 {
                        s_i32_0 = 0i32;
                        l8 = s_i32_0;
                        s_i32_0 = vm.get_i32(g0)?;
                        l9 = s_i32_0;
                        s_i32_0 = 8i32;
                        vm.enter_call()?;
                        let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                        vm.exit_call();
                        let __t2 = __call_res?;
                        s_i32_0 = __t2;
                        l10 = s_i32_0;
                        s_i32_0 = l10;
                        s_i32_1 = l8;
                        ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                        s_i32_0 = l10;
                        s_i32_1 = l9;
                        ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                        s_i32_0 = l10;
                        l11 = s_i32_0;
                        s_i32_0 = l11;
                        return Ok(s_i32_0);
                    } else {
                    }
                }
                s_i32_0 = l2;
                s_i32_1 = 6i32;
                vm.enter_call()?;
                let __call_res = Self::s_fa6af9f06252(vm, s_i32_0, s_i32_1, host);
                vm.exit_call();
                let __t3 = __call_res?;
                s_i32_0 = __t3;
                s_i32_1 = l7;
                s_i32_2 = 255i32;
                s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
                s_i64_1 = ops::i64_extend_i32_u(s_i32_1);
                vm.enter_call()?;
                let __call_res = crate::shared_0::Shared::s_41e9bacc9fe3(vm, s_i64_1, host);
                vm.exit_call();
                let __t4 = __call_res?;
                s_i32_1 = __t4;
                vm.enter_call()?;
                let __call_res = Self::s_9c006b2946a7(vm, s_i32_0, s_i32_1, host);
                vm.exit_call();
                let __t5 = __call_res?;
                s_i32_0 = __t5;
                l2 = s_i32_0;
            }
            s_i32_0 = l6;
            s_i32_1 = 1i32;
            s_i32_0 = ops::i32_add(s_i32_0, s_i32_1);
            l6 = s_i32_0;
            continue 'blk_2;
            break 'blk_2;
        }
    }
    s_i32_0 = 1i32;
    l12 = s_i32_0;
    s_i32_0 = l2;
    l13 = s_i32_0;
    s_i32_0 = 8i32;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
    vm.exit_call();
    let __t6 = __call_res?;
    s_i32_0 = __t6;
    l14 = s_i32_0;
    s_i32_0 = l14;
    s_i32_1 = l12;
    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l14;
    s_i32_1 = l13;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l14;
    l15 = s_i32_0;
    s_i32_0 = l15;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_e4620c669b0f(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let __call_res = crate::shared_0::Shared::s_14dd5b913b03(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    l2 = s_i32_0;
    s_i32_0 = l2;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn s_e4bf017d6693(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, g1: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let mut l12: i64 = 0i64;
    let mut l13: i64 = 0i64;
    let mut l14: i64 = 0i64;
    let mut l15: i32 = 0i32;
    let mut l16: i64 = 0i64;
    let mut l17: i64 = 0i64;
    let mut l18: i32 = 0i32;
    let mut l19: i32 = 0i32;
    let mut l20: i32 = 0i32;
    let mut l21: i32 = 0i32;
    let mut l22: i64 = 0i64;
    let mut l23: i32 = 0i32;
    let mut l24: i64 = 0i64;
    let mut l25: i64 = 0i64;
    let mut l26: i32 = 0i32;
    let mut l27: i32 = 0i32;
    let mut l28: i32 = 0i32;
    let mut l29: i32 = 0i32;
    let mut l30: i32 = 0i32;
    let mut l31: i64 = 0i64;
    let mut l32: i64 = 0i64;
    let mut l33: i32 = 0i32;
    let mut l34: i32 = 0i32;
    let mut l35: i32 = 0i32;
    let mut l36: i32 = 0i32;
    let mut l37: i32 = 0i32;
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
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
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
            l6 = s_i32_0;
            s_i32_0 = l6;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i32_0 = l1;
    s_i32_1 = 0i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
            l7 = s_i32_0;
            s_i32_0 = l0;
            l8 = s_i32_0;
            s_i32_0 = 0i32;
            l9 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
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
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 0u32)?;
    l12 = s_i64_0;
    s_i32_0 = l0;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
    l13 = s_i64_0;
    s_i64_0 = l13;
    l14 = s_i64_0;
    s_i32_0 = l1;
    s_i32_1 = 64i32;
    s_i32_0 = ops::i32_lt_u(s_i32_0, s_i32_1);
    'blk_3: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
            l15 = s_i32_0;
            s_i64_0 = l12;
            s_i32_1 = l1;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_73a866edda27(vm, s_i64_0, s_i32_1, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i64_0 = __t2;
            s_i64_1 = l13;
            s_i32_2 = 64i32;
            s_i32_3 = l1;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_304f22c4ab8c(vm, s_i32_2, s_i32_3, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i32_2 = __t3;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_52dcb6d97135(vm, s_i64_1, s_i32_2, host);
            vm.exit_call();
            let __t4 = __call_res?;
            s_i64_1 = __t4;
            s_i64_0 = ops::i64_or(s_i64_0, s_i64_1);
            l16 = s_i64_0;
            s_i64_0 = l14;
            s_i32_1 = l1;
            vm.enter_call()?;
            let __call_res = Self::s_c463b55dc5fa(vm, s_i64_0, s_i32_1, host);
            vm.exit_call();
            let __t5 = __call_res?;
            s_i64_0 = __t5;
            l17 = s_i64_0;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t6 = __call_res?;
            s_i32_0 = __t6;
            l18 = s_i32_0;
            s_i32_0 = l18;
            s_i64_1 = l16;
            ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
            s_i32_0 = l18;
            s_i64_1 = l17;
            ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
            s_i32_0 = l18;
            l19 = s_i32_0;
            s_i32_0 = 0i32;
            l20 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t7 = __call_res?;
            s_i32_0 = __t7;
            l21 = s_i32_0;
            s_i32_0 = l21;
            s_i32_1 = l15;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l21;
            s_i32_1 = l19;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l21;
            s_i32_1 = l20;
            ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l21;
        } else {
            s_i64_0 = l14;
            s_i32_1 = 63i32;
            vm.enter_call()?;
            let __call_res = Self::s_c463b55dc5fa(vm, s_i64_0, s_i32_1, host);
            vm.exit_call();
            let __t8 = __call_res?;
            s_i64_0 = __t8;
            l22 = s_i64_0;
            s_i32_0 = l1;
            s_i32_1 = 64i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_4: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                    l23 = s_i32_0;
                    s_i64_0 = l13;
                    l24 = s_i64_0;
                    s_i64_0 = l22;
                    l25 = s_i64_0;
                    s_i32_0 = 16i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                    vm.exit_call();
                    let __t9 = __call_res?;
                    s_i32_0 = __t9;
                    l26 = s_i32_0;
                    s_i32_0 = l26;
                    s_i64_1 = l24;
                    ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
                    s_i32_0 = l26;
                    s_i64_1 = l25;
                    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
                    s_i32_0 = l26;
                    l27 = s_i32_0;
                    s_i32_0 = 0i32;
                    l28 = s_i32_0;
                    s_i32_0 = 12i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                    vm.exit_call();
                    let __t10 = __call_res?;
                    s_i32_0 = __t10;
                    l29 = s_i32_0;
                    s_i32_0 = l29;
                    s_i32_1 = l23;
                    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l29;
                    s_i32_1 = l27;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l29;
                    s_i32_1 = l28;
                    ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                    s_i32_0 = l29;
                } else {
                    s_i32_0 = 1i32;
                    l30 = s_i32_0;
                    s_i64_0 = l14;
                    s_i32_1 = l1;
                    s_i32_2 = 64i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_304f22c4ab8c(vm, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    let __t11 = __call_res?;
                    s_i32_1 = __t11;
                    vm.enter_call()?;
                    let __call_res = Self::s_c463b55dc5fa(vm, s_i64_0, s_i32_1, host);
                    vm.exit_call();
                    let __t12 = __call_res?;
                    s_i64_0 = __t12;
                    l31 = s_i64_0;
                    s_i64_0 = l22;
                    l32 = s_i64_0;
                    s_i32_0 = 16i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                    vm.exit_call();
                    let __t13 = __call_res?;
                    s_i32_0 = __t13;
                    l33 = s_i32_0;
                    s_i32_0 = l33;
                    s_i64_1 = l31;
                    ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
                    s_i32_0 = l33;
                    s_i64_1 = l32;
                    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
                    s_i32_0 = l33;
                    l34 = s_i32_0;
                    s_i32_0 = 0i32;
                    l35 = s_i32_0;
                    s_i32_0 = 12i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                    vm.exit_call();
                    let __t14 = __call_res?;
                    s_i32_0 = __t14;
                    l36 = s_i32_0;
                    s_i32_0 = l36;
                    s_i32_1 = l30;
                    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l36;
                    s_i32_1 = l34;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l36;
                    s_i32_1 = l35;
                    ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                    s_i32_0 = l36;
                }
            }
        }
    }
    l37 = s_i32_0;
    s_i32_0 = l37;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_e55b9d2a287d(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, g1: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let __call_res = crate::shared_0::Shared::s_21f47e1623f0(vm, s_i32_0, s_i32_1, g0, g1, host);
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

    pub(crate) fn s_e6100fabeea2(vm: &mut SubstituteVm, p0: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    return crate::shared_0::Shared::s_3c36851dd02b(vm, __t0, host);
    }

    pub(crate) fn s_e67a785b002f(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let __call_res = Self::s_c9242759e3db(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    s_i32_1 = l0;
    let __t1 = s_i32_0;
    let __t2 = s_i32_1;
    return Self::s_c92ee68c0844(vm, __t1, __t2, host);
    }

    pub(crate) fn s_e68f8dc79e1d(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, g1: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
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
            s_i32_1 = l1;
            vm.enter_call()?;
            let __call_res = Self::s_fa6af9f06252(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            l7 = s_i32_0;
            s_i32_0 = 0i32;
            l8 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_0 = __t2;
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

    pub(crate) fn s_ea37e84be7cc(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    s_i64_0 = ops::i64_and(s_i64_0, s_i64_1);
    l2 = s_i64_0;
    s_i32_0 = l0;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
    s_i32_1 = l1;
    s_i64_1 = ops::i64_load(vm, 0, s_i32_1, 8u32)?;
    s_i64_0 = ops::i64_and(s_i64_0, s_i64_1);
    l3 = s_i64_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
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

    #[inline(never)]
    pub(crate) fn s_eb4cabbb1a10(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<(), WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = p1;
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
    let mut l13: i32 = 0i32;
    let mut l14: i32 = 0i32;
    let mut l15: i32 = 0i32;
    let mut l16: i32 = 0i32;
    let mut l17: i32 = 0i32;
    let mut l18: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i64_0: i64 = 0i64;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i64_1: i64 = 0i64;
    let mut s_i32_2: i32 = 0i32;
    let mut s_i64_2: i64 = 0i64;
    let mut s_i32_3: i32 = 0i32;
    let mut s_i64_3: i64 = 0i64;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l1;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 0u32)?;
    l2 = s_i64_0;
    s_i32_0 = l1;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
    l3 = s_i64_0;
    s_i64_0 = l3;
    s_i64_1 = 0i64;
    s_i32_0 = ops::i64_eq(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i64_0 = l2;
            s_i64_1 = 127i64;
            s_i32_0 = ops::i64_le_u(s_i64_0, s_i64_1);
        } else {
            s_i32_0 = 0i32;
        }
    }
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = l0;
            s_i64_1 = l2;
            vm.enter_call()?;
            let __call_res = Self::s_b5672010fcc9(vm, s_i64_1, host);
            vm.exit_call();
            let __t0 = __call_res?;
            s_i32_1 = __t0;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_5daf123ed54b(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            return Ok(());
        } else {
        }
    }
    s_i64_0 = l3;
    s_i64_1 = 0i64;
    s_i32_0 = ops::i64_eq(s_i64_0, s_i64_1);
    'blk_3: {
        if s_i32_0 != 0 {
            s_i64_0 = l2;
            s_i64_1 = 68719476735i64;
            s_i32_0 = ops::i64_le_u(s_i64_0, s_i64_1);
        } else {
            s_i32_0 = 0i32;
        }
    }
    'blk_4: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
            l4 = s_i32_0;
            s_i32_0 = 192i32;
            l5 = s_i32_0;
            s_i64_0 = l2;
            s_i64_1 = 2047i64;
            s_i32_0 = ops::i64_le_u(s_i64_0, s_i64_1);
            'blk_5: {
                if s_i32_0 != 0 {
                } else {
                    s_i64_0 = l2;
                    s_i64_1 = 65535i64;
                    s_i32_0 = ops::i64_le_u(s_i64_0, s_i64_1);
                    'blk_6: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 2i32;
                            l4 = s_i32_0;
                            s_i32_0 = 224i32;
                            l5 = s_i32_0;
                        } else {
                            s_i64_0 = l2;
                            s_i64_1 = 2097151i64;
                            s_i32_0 = ops::i64_le_u(s_i64_0, s_i64_1);
                            'blk_7: {
                                if s_i32_0 != 0 {
                                    s_i32_0 = 3i32;
                                    l4 = s_i32_0;
                                    s_i32_0 = 240i32;
                                    l5 = s_i32_0;
                                } else {
                                    s_i64_0 = l2;
                                    s_i64_1 = 67108863i64;
                                    s_i32_0 = ops::i64_le_u(s_i64_0, s_i64_1);
                                    'blk_8: {
                                        if s_i32_0 != 0 {
                                            s_i32_0 = 4i32;
                                            l4 = s_i32_0;
                                            s_i32_0 = 248i32;
                                            l5 = s_i32_0;
                                        } else {
                                            s_i64_0 = l2;
                                            s_i64_1 = 2147483647i64;
                                            s_i32_0 = ops::i64_le_u(s_i64_0, s_i64_1);
                                            'blk_9: {
                                                if s_i32_0 != 0 {
                                                    s_i32_0 = 5i32;
                                                    l4 = s_i32_0;
                                                    s_i32_0 = 252i32;
                                                    l5 = s_i32_0;
                                                } else {
                                                    s_i32_0 = 6i32;
                                                    l4 = s_i32_0;
                                                    s_i32_0 = 254i32;
                                                    l5 = s_i32_0;
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
            s_i32_0 = l4;
            s_i32_1 = 6i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_19ca9f9d3f2f(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            l6 = s_i32_0;
            s_i32_0 = l0;
            s_i32_1 = l5;
            s_i64_2 = l2;
            s_i32_3 = l6;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_73a866edda27(vm, s_i64_2, s_i32_3, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i64_2 = __t2;
            vm.enter_call()?;
            let __call_res = Self::s_b5672010fcc9(vm, s_i64_2, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i32_2 = __t3;
            s_i32_1 = ops::i32_or(s_i32_1, s_i32_2);
            s_i32_2 = 255i32;
            s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_5daf123ed54b(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = 0i32;
            l7 = s_i32_0;
            'blk_10: {
                'blk_11: loop {
                    vm.deduct(host, 1)?;
                    s_i32_0 = l7;
                    s_i32_1 = l4;
                    s_i32_0 = ops::i32_lt_u(s_i32_0, s_i32_1);
                    s_i32_0 = ops::i32_eqz(s_i32_0);
                    if s_i32_0 != 0 {
                        break 'blk_10;
                    }
                    s_i32_0 = l4;
                    s_i32_1 = 1i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_304f22c4ab8c(vm, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t4 = __call_res?;
                    s_i32_0 = __t4;
                    s_i32_1 = l7;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_304f22c4ab8c(vm, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t5 = __call_res?;
                    s_i32_0 = __t5;
                    s_i32_1 = 6i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_19ca9f9d3f2f(vm, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t6 = __call_res?;
                    s_i32_0 = __t6;
                    l8 = s_i32_0;
                    s_i32_0 = l0;
                    s_i32_1 = 128i32;
                    s_i64_2 = l2;
                    s_i32_3 = l8;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_73a866edda27(vm, s_i64_2, s_i32_3, host);
                    vm.exit_call();
                    let __t7 = __call_res?;
                    s_i64_2 = __t7;
                    s_i64_3 = 63i64;
                    s_i64_2 = ops::i64_and(s_i64_2, s_i64_3);
                    vm.enter_call()?;
                    let __call_res = Self::s_b5672010fcc9(vm, s_i64_2, host);
                    vm.exit_call();
                    let __t8 = __call_res?;
                    s_i32_2 = __t8;
                    s_i32_1 = ops::i32_or(s_i32_1, s_i32_2);
                    s_i32_2 = 255i32;
                    s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_5daf123ed54b(vm, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l7;
                    s_i32_1 = 1i32;
                    vm.enter_call()?;
                    let __call_res = Self::s_91b7532ed2b6(vm, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t9 = __call_res?;
                    s_i32_0 = __t9;
                    l7 = s_i32_0;
                    continue 'blk_11;
                    break 'blk_11;
                }
            }
            return Ok(());
        } else {
        }
    }
    s_i32_0 = l1;
    vm.enter_call()?;
    let __call_res = Self::s_f76e9849d691(vm, s_i32_0, host);
    vm.exit_call();
    let __t10 = __call_res?;
    s_i32_0 = __t10;
    l9 = s_i32_0;
    s_i32_0 = l9;
    vm.enter_call()?;
    let __call_res = Self::s_958e9a31e156(vm, s_i32_0, host);
    vm.exit_call();
    let __t11 = __call_res?;
    s_i32_0 = __t11;
    l10 = s_i32_0;
    s_i32_0 = 128i32;
    s_i32_1 = l10;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_304f22c4ab8c(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t12 = __call_res?;
    s_i32_0 = __t12;
    l11 = s_i32_0;
    s_i32_0 = l11;
    s_i32_1 = 38i32;
    s_i32_0 = ops::i32_le_u(s_i32_0, s_i32_1);
    'blk_12: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
        } else {
            s_i32_0 = l11;
            s_i32_1 = 38i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_304f22c4ab8c(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t13 = __call_res?;
            s_i32_0 = __t13;
            s_i32_1 = 5i32;
            vm.enter_call()?;
            let __call_res = Self::s_91b7532ed2b6(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t14 = __call_res?;
            s_i32_0 = __t14;
            s_i32_1 = 6i32;
            s_i32_0 = ops::i32_div_u(s_i32_0, s_i32_1)?;
        }
    }
    l12 = s_i32_0;
    s_i32_0 = l12;
    s_i32_1 = 6i32;
    vm.enter_call()?;
    let __call_res = Self::s_91b7532ed2b6(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t15 = __call_res?;
    s_i32_0 = __t15;
    l13 = s_i32_0;
    s_i32_0 = l13;
    s_i32_1 = 6i32;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_19ca9f9d3f2f(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t16 = __call_res?;
    s_i32_0 = __t16;
    l14 = s_i32_0;
    s_i32_0 = l14;
    s_i32_1 = 128i32;
    s_i32_0 = ops::i32_lt_u(s_i32_0, s_i32_1);
    'blk_13: {
        if s_i32_0 != 0 {
            s_i32_0 = l9;
            s_i32_1 = l14;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_7009a9e7c9fa(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t17 = __call_res?;
            s_i32_0 = __t17;
            s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 0u32)?;
            s_i64_1 = 3i64;
            s_i64_0 = ops::i64_and(s_i64_0, s_i64_1);
            vm.enter_call()?;
            let __call_res = Self::s_b5672010fcc9(vm, s_i64_0, host);
            vm.exit_call();
            let __t18 = __call_res?;
            s_i32_0 = __t18;
        } else {
            s_i32_0 = 0i32;
        }
    }
    l15 = s_i32_0;
    s_i32_0 = l0;
    s_i32_1 = 255i32;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_5daf123ed54b(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_1 = 128i32;
    s_i32_2 = l12;
    vm.enter_call()?;
    let __call_res = Self::s_b8acef2c4d16(vm, s_i32_2, host);
    vm.exit_call();
    let __t19 = __call_res?;
    s_i32_2 = __t19;
    s_i32_3 = 2i32;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_1e670f572c93(vm, s_i32_2, s_i32_3, host);
    vm.exit_call();
    let __t20 = __call_res?;
    s_i32_2 = __t20;
    s_i32_3 = l15;
    s_i32_2 = ops::i32_or(s_i32_2, s_i32_3);
    s_i32_3 = 255i32;
    s_i32_2 = ops::i32_and(s_i32_2, s_i32_3);
    s_i32_1 = ops::i32_or(s_i32_1, s_i32_2);
    s_i32_2 = 255i32;
    s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_5daf123ed54b(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = 0i32;
    l16 = s_i32_0;
    'blk_14: {
        'blk_15: loop {
            vm.deduct(host, 1)?;
            s_i32_0 = l16;
            s_i32_1 = l13;
            s_i32_0 = ops::i32_lt_u(s_i32_0, s_i32_1);
            s_i32_0 = ops::i32_eqz(s_i32_0);
            if s_i32_0 != 0 {
                break 'blk_14;
            }
            s_i32_0 = l13;
            s_i32_1 = 1i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_304f22c4ab8c(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t21 = __call_res?;
            s_i32_0 = __t21;
            s_i32_1 = l16;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_304f22c4ab8c(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t22 = __call_res?;
            s_i32_0 = __t22;
            s_i32_1 = 6i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_19ca9f9d3f2f(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t23 = __call_res?;
            s_i32_0 = __t23;
            l17 = s_i32_0;
            s_i32_0 = l9;
            s_i32_1 = l17;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_7009a9e7c9fa(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t24 = __call_res?;
            s_i32_0 = __t24;
            l18 = s_i32_0;
            s_i32_0 = l0;
            s_i32_1 = 128i32;
            s_i32_2 = l18;
            s_i64_2 = ops::i64_load(vm, 0, s_i32_2, 0u32)?;
            s_i64_3 = 63i64;
            s_i64_2 = ops::i64_and(s_i64_2, s_i64_3);
            vm.enter_call()?;
            let __call_res = Self::s_b5672010fcc9(vm, s_i64_2, host);
            vm.exit_call();
            let __t25 = __call_res?;
            s_i32_2 = __t25;
            s_i32_1 = ops::i32_or(s_i32_1, s_i32_2);
            s_i32_2 = 255i32;
            s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_5daf123ed54b(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l16;
            s_i32_1 = 1i32;
            vm.enter_call()?;
            let __call_res = Self::s_91b7532ed2b6(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t26 = __call_res?;
            s_i32_0 = __t26;
            l16 = s_i32_0;
            continue 'blk_15;
            break 'blk_15;
        }
    }
    return Ok(());
    }

    pub(crate) fn s_ec608351a53d(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, g1: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let __call_res = Self::s_8bb0e1cac787(vm, s_i32_0, s_i32_1, g0, g1, host);
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
    pub(crate) fn s_f2fcb0a8824e(vm: &mut SubstituteVm, p0: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i64, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i64 = 0i64;
    let mut l2: i64 = 0i64;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i32 = 0i32;
    let mut l7: i32 = 0i32;
    let mut l8: i32 = 0i32;
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
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 0u32)?;
    l1 = s_i64_0;
    s_i32_0 = l0;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
    l2 = s_i64_0;
    s_i64_0 = l2;
    s_i64_1 = 0i64;
    s_i32_0 = ops::i64_eq(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i64_0 = l1;
            s_i64_1 = 127i64;
            s_i32_0 = ops::i64_le_u(s_i64_0, s_i64_1);
            'blk_2: {
                if s_i32_0 != 0 {
                    s_i64_0 = 1i64;
                } else {
                    s_i64_0 = l1;
                    s_i64_1 = 2047i64;
                    s_i32_0 = ops::i64_le_u(s_i64_0, s_i64_1);
                    'blk_3: {
                        if s_i32_0 != 0 {
                            s_i64_0 = 2i64;
                        } else {
                            s_i64_0 = l1;
                            s_i64_1 = 65535i64;
                            s_i32_0 = ops::i64_le_u(s_i64_0, s_i64_1);
                            'blk_4: {
                                if s_i32_0 != 0 {
                                    s_i64_0 = 3i64;
                                } else {
                                    s_i64_0 = l1;
                                    s_i64_1 = 2097151i64;
                                    s_i32_0 = ops::i64_le_u(s_i64_0, s_i64_1);
                                    'blk_5: {
                                        if s_i32_0 != 0 {
                                            s_i64_0 = 4i64;
                                        } else {
                                            s_i64_0 = l1;
                                            s_i64_1 = 67108863i64;
                                            s_i32_0 = ops::i64_le_u(s_i64_0, s_i64_1);
                                            'blk_6: {
                                                if s_i32_0 != 0 {
                                                    s_i64_0 = 5i64;
                                                } else {
                                                    s_i64_0 = l1;
                                                    s_i64_1 = 2147483647i64;
                                                    s_i32_0 = ops::i64_le_u(s_i64_0, s_i64_1);
                                                    'blk_7: {
                                                        if s_i32_0 != 0 {
                                                            s_i64_0 = 6i64;
                                                        } else {
                                                            s_i64_0 = l1;
                                                            s_i64_1 = 68719476735i64;
                                                            s_i32_0 = ops::i64_le_u(s_i64_0, s_i64_1);
                                                            'blk_8: {
                                                                if s_i32_0 != 0 {
                                                                    s_i64_0 = 7i64;
                                                                } else {
                                                                    s_i32_0 = l0;
                                                                    vm.enter_call()?;
                                                                    let __call_res = Self::s_f76e9849d691(vm, s_i32_0, host);
                                                                    vm.exit_call();
                                                                    let __t0 = __call_res?;
                                                                    s_i32_0 = __t0;
                                                                    vm.enter_call()?;
                                                                    let __call_res = Self::s_958e9a31e156(vm, s_i32_0, host);
                                                                    vm.exit_call();
                                                                    let __t1 = __call_res?;
                                                                    s_i32_0 = __t1;
                                                                    l3 = s_i32_0;
                                                                    s_i32_0 = 128i32;
                                                                    s_i32_1 = l3;
                                                                    vm.enter_call()?;
                                                                    let __call_res = crate::shared_0::Shared::s_304f22c4ab8c(vm, s_i32_0, s_i32_1, host);
                                                                    vm.exit_call();
                                                                    let __t2 = __call_res?;
                                                                    s_i32_0 = __t2;
                                                                    l4 = s_i32_0;
                                                                    s_i32_0 = l4;
                                                                    s_i32_1 = 38i32;
                                                                    s_i32_0 = ops::i32_le_u(s_i32_0, s_i32_1);
                                                                    'blk_9: {
                                                                        if s_i32_0 != 0 {
                                                                            s_i32_0 = 0i32;
                                                                        } else {
                                                                            s_i32_0 = l4;
                                                                            s_i32_1 = 38i32;
                                                                            vm.enter_call()?;
                                                                            let __call_res = crate::shared_0::Shared::s_304f22c4ab8c(vm, s_i32_0, s_i32_1, host);
                                                                            vm.exit_call();
                                                                            let __t3 = __call_res?;
                                                                            s_i32_0 = __t3;
                                                                            s_i32_1 = 5i32;
                                                                            vm.enter_call()?;
                                                                            let __call_res = Self::s_91b7532ed2b6(vm, s_i32_0, s_i32_1, host);
                                                                            vm.exit_call();
                                                                            let __t4 = __call_res?;
                                                                            s_i32_0 = __t4;
                                                                            s_i32_1 = 6i32;
                                                                            s_i32_0 = ops::i32_div_u(s_i32_0, s_i32_1)?;
                                                                        }
                                                                    }
                                                                    l5 = s_i32_0;
                                                                    s_i64_0 = 8i64;
                                                                    s_i32_1 = l5;
                                                                    s_i64_1 = ops::i64_extend_i32_u(s_i32_1);
                                                                    vm.enter_call()?;
                                                                    let __call_res = crate::shared_0::Shared::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
                                                                    vm.exit_call();
                                                                    let __t5 = __call_res?;
                                                                    s_i64_0 = __t5;
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
            s_i32_0 = l0;
            vm.enter_call()?;
            let __call_res = Self::s_f76e9849d691(vm, s_i32_0, host);
            vm.exit_call();
            let __t6 = __call_res?;
            s_i32_0 = __t6;
            vm.enter_call()?;
            let __call_res = Self::s_958e9a31e156(vm, s_i32_0, host);
            vm.exit_call();
            let __t7 = __call_res?;
            s_i32_0 = __t7;
            l6 = s_i32_0;
            s_i32_0 = 128i32;
            s_i32_1 = l6;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_304f22c4ab8c(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t8 = __call_res?;
            s_i32_0 = __t8;
            l7 = s_i32_0;
            s_i32_0 = l7;
            s_i32_1 = 38i32;
            s_i32_0 = ops::i32_le_u(s_i32_0, s_i32_1);
            'blk_10: {
                if s_i32_0 != 0 {
                    s_i32_0 = 0i32;
                } else {
                    s_i32_0 = l7;
                    s_i32_1 = 38i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_304f22c4ab8c(vm, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t9 = __call_res?;
                    s_i32_0 = __t9;
                    s_i32_1 = 5i32;
                    vm.enter_call()?;
                    let __call_res = Self::s_91b7532ed2b6(vm, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t10 = __call_res?;
                    s_i32_0 = __t10;
                    s_i32_1 = 6i32;
                    s_i32_0 = ops::i32_div_u(s_i32_0, s_i32_1)?;
                }
            }
            l8 = s_i32_0;
            s_i64_0 = 8i64;
            s_i32_1 = l8;
            s_i64_1 = ops::i64_extend_i32_u(s_i32_1);
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
            vm.exit_call();
            let __t11 = __call_res?;
            s_i64_0 = __t11;
        }
    }
    l9 = s_i64_0;
    s_i64_0 = l9;
    return Ok(s_i64_0);
    }

    pub(crate) fn s_f507e9fb18e1(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let __call_res = crate::shared_0::Shared::s_559745ed6085(vm, s_i32_0, s_i32_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    s_i32_0 = ops::i32_eqz(s_i32_0);
    l2 = s_i32_0;
    s_i32_0 = l2;
    return Ok(s_i32_0);
    }

    #[inline(never)]
    pub(crate) fn s_f55569016827(vm: &mut SubstituteVm, p0: i32, p1: i64, g0: usize, g1: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i64 = p1;
    let mut l2: i64 = 0i64;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i64 = 0i64;
    let mut l7: i32 = 0i32;
    let mut l8: i32 = 0i32;
    let mut l9: i32 = 0i32;
    let mut l10: i32 = 0i32;
    let mut l11: i32 = 0i32;
    let mut l12: i32 = 0i32;
    let mut l13: i64 = 0i64;
    let mut l14: i32 = 0i32;
    let mut l15: i32 = 0i32;
    let mut l16: i32 = 0i32;
    let mut l17: i32 = 0i32;
    let mut l18: i32 = 0i32;
    let mut l19: i32 = 0i32;
    let mut l20: i64 = 0i64;
    let mut l21: i32 = 0i32;
    let mut l22: i32 = 0i32;
    let mut l23: i64 = 0i64;
    let mut l24: i32 = 0i32;
    let mut l25: i64 = 0i64;
    let mut l26: i32 = 0i32;
    let mut l27: i32 = 0i32;
    let mut l28: i64 = 0i64;
    let mut l29: i32 = 0i32;
    let mut l30: i32 = 0i32;
    let mut l31: i32 = 0i32;
    let mut l32: i32 = 0i32;
    let mut l33: i32 = 0i32;
    let mut l34: i64 = 0i64;
    let mut l35: i32 = 0i32;
    let mut l36: i32 = 0i32;
    let mut l37: i32 = 0i32;
    let mut l38: i32 = 0i32;
    let mut l39: i32 = 0i32;
    let mut l40: i64 = 0i64;
    let mut l41: i32 = 0i32;
    let mut l42: i32 = 0i32;
    let mut l43: i32 = 0i32;
    let mut l44: i32 = 0i32;
    let mut l45: i32 = 0i32;
    let mut l46: i32 = 0i32;
    let mut l47: i32 = 0i32;
    let mut l48: i32 = 0i32;
    let mut l49: i32 = 0i32;
    let mut l50: i64 = 0i64;
    let mut l51: i32 = 0i32;
    let mut l52: i32 = 0i32;
    let mut l53: i32 = 0i32;
    let mut l54: i32 = 0i32;
    let mut l55: i32 = 0i32;
    let mut l56: i32 = 0i32;
    let mut l57: i32 = 0i32;
    let mut l58: i64 = 0i64;
    let mut l59: i32 = 0i32;
    let mut l60: i32 = 0i32;
    let mut l61: i32 = 0i32;
    let mut l62: i32 = 0i32;
    let mut l63: i32 = 0i32;
    let mut l64: i64 = 0i64;
    let mut l65: i32 = 0i32;
    let mut l66: i32 = 0i32;
    let mut l67: i32 = 0i32;
    let mut l68: i32 = 0i32;
    let mut l69: i32 = 0i32;
    let mut l70: i32 = 0i32;
    let mut l71: i64 = 0i64;
    let mut l72: i32 = 0i32;
    let mut l73: i32 = 0i32;
    let mut l74: i32 = 0i32;
    let mut l75: i32 = 0i32;
    let mut l76: i32 = 0i32;
    let mut l77: i32 = 0i32;
    let mut l78: i64 = 0i64;
    let mut l79: i32 = 0i32;
    let mut l80: i32 = 0i32;
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
    l2 = s_i64_0;
    s_i64_0 = l1;
    s_i64_1 = l2;
    s_i32_0 = ops::i64_ge_u(s_i64_0, s_i64_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
        } else {
            s_i32_0 = l0;
            s_i64_1 = l1;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
            vm.exit_call();
            let __t0 = __call_res?;
            s_i32_0 = __t0;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_62439e6f6ae7(vm, s_i32_0, g0, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            s_i32_1 = 203i32;
            s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
        }
    }
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            l3 = s_i32_0;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
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
            s_i64_0 = l1;
            l6 = s_i64_0;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i32_0 = __t3;
            l7 = s_i32_0;
            s_i32_0 = l7;
            s_i32_1 = l3;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l7;
            s_i32_1 = l5;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l7;
            s_i64_1 = l6;
            ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
            s_i32_0 = l7;
            l8 = s_i32_0;
            s_i32_0 = l8;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i32_0 = l0;
    s_i64_1 = l1;
    s_i64_2 = 1i64;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_1c917789866c(vm, s_i64_1, s_i64_2, host);
    vm.exit_call();
    let __t4 = __call_res?;
    s_i64_1 = __t4;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_2570bfaef612(vm, s_i32_0, s_i64_1, g0, host);
    vm.exit_call();
    let __t5 = __call_res?;
    s_i32_0 = __t5;
    l9 = s_i32_0;
    s_i32_0 = l9;
    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
    s_i32_0 = ops::i32_eqz(s_i32_0);
    'blk_3: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
        } else {
            s_i32_0 = l9;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
            s_i32_1 = 199i32;
            s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
            'blk_4: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                } else {
                    s_i32_0 = l9;
                    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 8u32)?;
                }
            }
        }
    }
    'blk_5: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            l10 = s_i32_0;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t6 = __call_res?;
            s_i32_0 = __t6;
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
            s_i64_0 = l1;
            l13 = s_i64_0;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t7 = __call_res?;
            s_i32_0 = __t7;
            l14 = s_i32_0;
            s_i32_0 = l14;
            s_i32_1 = l10;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l14;
            s_i32_1 = l12;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l14;
            s_i64_1 = l13;
            ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
            s_i32_0 = l14;
            l15 = s_i32_0;
            s_i32_0 = l15;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i32_0 = l9;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 12u32)?;
    vm.enter_call()?;
    let __call_res = Self::s_e208ae0aad56(vm, s_i32_0, g1, host);
    vm.exit_call();
    let __t8 = __call_res?;
    s_i32_0 = __t8;
    l16 = s_i32_0;
    s_i32_0 = l16;
    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
    s_i32_0 = ops::i32_eqz(s_i32_0);
    'blk_6: {
        if s_i32_0 != 0 {
            s_i32_0 = 1i32;
        } else {
            s_i32_0 = l16;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
            s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 8u32)?;
            s_i64_1 = 0i64;
            s_i32_0 = ops::i64_ne(s_i64_0, s_i64_1);
        }
    }
    'blk_7: {
        if s_i32_0 != 0 {
            s_i32_0 = 0i32;
            l17 = s_i32_0;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t9 = __call_res?;
            s_i32_0 = __t9;
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
            s_i64_0 = l1;
            l20 = s_i64_0;
            s_i32_0 = 16i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t10 = __call_res?;
            s_i32_0 = __t10;
            l21 = s_i32_0;
            s_i32_0 = l21;
            s_i32_1 = l17;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l21;
            s_i32_1 = l19;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l21;
            s_i64_1 = l20;
            ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
            s_i32_0 = l21;
            l22 = s_i32_0;
            s_i32_0 = l22;
            return Ok(s_i32_0);
        } else {
        }
    }
    s_i32_0 = l16;
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 0u32)?;
    l23 = s_i64_0;
    s_i64_0 = l23;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_61878e7d5ba7(vm, s_i64_0, host);
    vm.exit_call();
    let __t11 = __call_res?;
    s_i32_0 = __t11;
    l24 = s_i32_0;
    s_i32_0 = l9;
    s_i64_0 = ops::i64_load(vm, 0, s_i32_0, 16u32)?;
    l25 = s_i64_0;
    'blk_8: {
        'blk_9: loop {
            vm.deduct(host, 1)?;
            s_i64_0 = l25;
            s_i64_1 = l2;
            s_i32_0 = ops::i64_lt_u(s_i64_0, s_i64_1);
            s_i32_0 = ops::i32_eqz(s_i32_0);
            if s_i32_0 != 0 {
                break 'blk_8;
            }
            s_i32_0 = l0;
            s_i64_1 = l25;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
            vm.exit_call();
            let __t12 = __call_res?;
            s_i32_0 = __t12;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_62439e6f6ae7(vm, s_i32_0, g0, host);
            vm.exit_call();
            let __t13 = __call_res?;
            s_i32_0 = __t13;
            s_i32_1 = 204i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_10: {
                if s_i32_0 != 0 {
                    s_i32_0 = l24;
                    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
                    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
                    s_i64_1 = l23;
                    s_i32_0 = ops::i64_eq(s_i64_0, s_i64_1);
                    'blk_11: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 1i32;
                            l26 = s_i32_0;
                            s_i32_0 = l24;
                            vm.enter_call()?;
                            let __call_res = Self::s_df3679579582(vm, s_i32_0, host);
                            vm.exit_call();
                            let __t14 = __call_res?;
                            s_i32_0 = __t14;
                            l27 = s_i32_0;
                            s_i64_0 = l25;
                            s_i64_1 = 1i64;
                            vm.enter_call()?;
                            let __call_res = crate::shared_0::Shared::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
                            vm.exit_call();
                            let __t15 = __call_res?;
                            s_i64_0 = __t15;
                            l28 = s_i64_0;
                            s_i32_0 = 16i32;
                            vm.enter_call()?;
                            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                            vm.exit_call();
                            let __t16 = __call_res?;
                            s_i32_0 = __t16;
                            l29 = s_i32_0;
                            s_i32_0 = l29;
                            s_i32_1 = l26;
                            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                            s_i32_0 = l29;
                            s_i32_1 = l27;
                            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                            s_i32_0 = l29;
                            s_i64_1 = l28;
                            ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
                            s_i32_0 = l29;
                            l30 = s_i32_0;
                            s_i32_0 = l30;
                            return Ok(s_i32_0);
                        } else {
                            s_i32_0 = 0i32;
                            l31 = s_i32_0;
                            s_i32_0 = 16i32;
                            vm.enter_call()?;
                            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                            vm.exit_call();
                            let __t17 = __call_res?;
                            s_i32_0 = __t17;
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
                            s_i64_0 = l1;
                            l34 = s_i64_0;
                            s_i32_0 = 16i32;
                            vm.enter_call()?;
                            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                            vm.exit_call();
                            let __t18 = __call_res?;
                            s_i32_0 = __t18;
                            l35 = s_i32_0;
                            s_i32_0 = l35;
                            s_i32_1 = l31;
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
                        }
                    }
                    return Err(WasmTrap::Unreachable);
                } else {
                }
            }
            s_i64_0 = l25;
            s_i64_1 = 4i64;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
            vm.exit_call();
            let __t19 = __call_res?;
            s_i64_0 = __t19;
            s_i64_1 = l2;
            s_i32_0 = ops::i64_gt_u(s_i64_0, s_i64_1);
            'blk_12: {
                if s_i32_0 != 0 {
                    s_i32_0 = 0i32;
                    l37 = s_i32_0;
                    s_i32_0 = 16i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                    vm.exit_call();
                    let __t20 = __call_res?;
                    s_i32_0 = __t20;
                    l38 = s_i32_0;
                    s_i32_0 = l38;
                    s_i32_1 = l38;
                    s_i32_2 = 16i32;
                    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l38;
                    s_i32_1 = 0i32;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l38;
                    s_i32_1 = 0i32;
                    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                    s_i32_0 = l38;
                    s_i32_1 = 1i32;
                    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                    s_i32_0 = l38;
                    l39 = s_i32_0;
                    s_i64_0 = l1;
                    l40 = s_i64_0;
                    s_i32_0 = 16i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                    vm.exit_call();
                    let __t21 = __call_res?;
                    s_i32_0 = __t21;
                    l41 = s_i32_0;
                    s_i32_0 = l41;
                    s_i32_1 = l37;
                    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l41;
                    s_i32_1 = l39;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l41;
                    s_i64_1 = l40;
                    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
                    s_i32_0 = l41;
                    l42 = s_i32_0;
                    s_i32_0 = l42;
                    return Ok(s_i32_0);
                } else {
                }
            }
            s_i32_0 = l0;
            s_i64_1 = l25;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
            vm.exit_call();
            let __t22 = __call_res?;
            s_i32_0 = __t22;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_62439e6f6ae7(vm, s_i32_0, g0, host);
            vm.exit_call();
            let __t23 = __call_res?;
            s_i32_0 = __t23;
            l43 = s_i32_0;
            s_i32_0 = l0;
            s_i64_1 = l25;
            s_i64_2 = 1i64;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_1c917789866c(vm, s_i64_1, s_i64_2, host);
            vm.exit_call();
            let __t24 = __call_res?;
            s_i64_1 = __t24;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
            vm.exit_call();
            let __t25 = __call_res?;
            s_i32_0 = __t25;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_62439e6f6ae7(vm, s_i32_0, g0, host);
            vm.exit_call();
            let __t26 = __call_res?;
            s_i32_0 = __t26;
            l44 = s_i32_0;
            s_i32_0 = l0;
            s_i64_1 = l25;
            s_i64_2 = 2i64;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_1c917789866c(vm, s_i64_1, s_i64_2, host);
            vm.exit_call();
            let __t27 = __call_res?;
            s_i64_1 = __t27;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
            vm.exit_call();
            let __t28 = __call_res?;
            s_i32_0 = __t28;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_62439e6f6ae7(vm, s_i32_0, g0, host);
            vm.exit_call();
            let __t29 = __call_res?;
            s_i32_0 = __t29;
            l45 = s_i32_0;
            s_i32_0 = l0;
            s_i64_1 = l25;
            s_i64_2 = 3i64;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_1c917789866c(vm, s_i64_1, s_i64_2, host);
            vm.exit_call();
            let __t30 = __call_res?;
            s_i64_1 = __t30;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_17df4637c35a(vm, s_i32_0, s_i64_1, host);
            vm.exit_call();
            let __t31 = __call_res?;
            s_i32_0 = __t31;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_62439e6f6ae7(vm, s_i32_0, g0, host);
            vm.exit_call();
            let __t32 = __call_res?;
            s_i32_0 = __t32;
            l46 = s_i32_0;
            s_i32_0 = l43;
            s_i32_1 = 127i32;
            s_i32_0 = ops::i32_lt_u(s_i32_0, s_i32_1);
            'blk_13: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                } else {
                    s_i32_0 = l43;
                    s_i32_1 = 190i32;
                    s_i32_0 = ops::i32_gt_u(s_i32_0, s_i32_1);
                    'blk_14: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 1i32;
                        } else {
                            s_i32_0 = l44;
                            s_i32_1 = 127i32;
                            s_i32_0 = ops::i32_lt_u(s_i32_0, s_i32_1);
                            'blk_15: {
                                if s_i32_0 != 0 {
                                    s_i32_0 = 1i32;
                                } else {
                                    s_i32_0 = l44;
                                    s_i32_1 = 190i32;
                                    s_i32_0 = ops::i32_gt_u(s_i32_0, s_i32_1);
                                }
                            }
                        }
                    }
                }
            }
            'blk_16: {
                if s_i32_0 != 0 {
                    s_i32_0 = 0i32;
                    l47 = s_i32_0;
                    s_i32_0 = 16i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                    vm.exit_call();
                    let __t33 = __call_res?;
                    s_i32_0 = __t33;
                    l48 = s_i32_0;
                    s_i32_0 = l48;
                    s_i32_1 = l48;
                    s_i32_2 = 16i32;
                    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l48;
                    s_i32_1 = 0i32;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l48;
                    s_i32_1 = 0i32;
                    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                    s_i32_0 = l48;
                    s_i32_1 = 1i32;
                    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                    s_i32_0 = l48;
                    l49 = s_i32_0;
                    s_i64_0 = l1;
                    l50 = s_i64_0;
                    s_i32_0 = 16i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                    vm.exit_call();
                    let __t34 = __call_res?;
                    s_i32_0 = __t34;
                    l51 = s_i32_0;
                    s_i32_0 = l51;
                    s_i32_1 = l47;
                    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                    s_i32_0 = l51;
                    s_i32_1 = l49;
                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                    s_i32_0 = l51;
                    s_i64_1 = l50;
                    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
                    s_i32_0 = l51;
                    l52 = s_i32_0;
                    s_i32_0 = l52;
                    return Ok(s_i32_0);
                } else {
                }
            }
            s_i32_0 = l43;
            s_i32_1 = 127i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_304f22c4ab8c(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t35 = __call_res?;
            s_i32_0 = __t35;
            vm.enter_call()?;
            let __call_res = Self::s_b8acef2c4d16(vm, s_i32_0, host);
            vm.exit_call();
            let __t36 = __call_res?;
            s_i32_0 = __t36;
            l53 = s_i32_0;
            s_i32_0 = l44;
            s_i32_1 = 127i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_304f22c4ab8c(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t37 = __call_res?;
            s_i32_0 = __t37;
            vm.enter_call()?;
            let __call_res = Self::s_b8acef2c4d16(vm, s_i32_0, host);
            vm.exit_call();
            let __t38 = __call_res?;
            s_i32_0 = __t38;
            l54 = s_i32_0;
            s_i32_0 = l24;
            s_i32_1 = l53;
            s_i32_2 = 2i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_1e670f572c93(vm, s_i32_1, s_i32_2, host);
            vm.exit_call();
            let __t39 = __call_res?;
            s_i32_1 = __t39;
            s_i32_2 = l54;
            s_i32_3 = 4i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_708c3053a6ba(vm, s_i32_2, s_i32_3, host);
            vm.exit_call();
            let __t40 = __call_res?;
            s_i32_2 = __t40;
            s_i32_1 = ops::i32_or(s_i32_1, s_i32_2);
            s_i32_2 = 255i32;
            s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_5daf123ed54b(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            __call_res?;
            s_i32_0 = l45;
            s_i32_1 = 195i32;
            s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
            'blk_17: {
                if s_i32_0 != 0 {
                    s_i32_0 = l46;
                    s_i32_1 = 195i32;
                    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
                    'blk_18: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 0i32;
                            l55 = s_i32_0;
                            s_i32_0 = 16i32;
                            vm.enter_call()?;
                            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                            vm.exit_call();
                            let __t41 = __call_res?;
                            s_i32_0 = __t41;
                            l56 = s_i32_0;
                            s_i32_0 = l56;
                            s_i32_1 = l56;
                            s_i32_2 = 16i32;
                            s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
                            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                            s_i32_0 = l56;
                            s_i32_1 = 0i32;
                            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                            s_i32_0 = l56;
                            s_i32_1 = 0i32;
                            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                            s_i32_0 = l56;
                            s_i32_1 = 1i32;
                            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                            s_i32_0 = l56;
                            l57 = s_i32_0;
                            s_i64_0 = l1;
                            l58 = s_i64_0;
                            s_i32_0 = 16i32;
                            vm.enter_call()?;
                            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                            vm.exit_call();
                            let __t42 = __call_res?;
                            s_i32_0 = __t42;
                            l59 = s_i32_0;
                            s_i32_0 = l59;
                            s_i32_1 = l55;
                            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                            s_i32_0 = l59;
                            s_i32_1 = l57;
                            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                            s_i32_0 = l59;
                            s_i64_1 = l58;
                            ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
                            s_i32_0 = l59;
                            l60 = s_i32_0;
                            s_i32_0 = l60;
                            return Ok(s_i32_0);
                        } else {
                        }
                    }
                } else {
                    s_i32_0 = l45;
                    s_i32_1 = 127i32;
                    s_i32_0 = ops::i32_lt_u(s_i32_0, s_i32_1);
                    'blk_19: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 1i32;
                        } else {
                            s_i32_0 = l45;
                            s_i32_1 = 190i32;
                            s_i32_0 = ops::i32_gt_u(s_i32_0, s_i32_1);
                        }
                    }
                    'blk_20: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 0i32;
                            l61 = s_i32_0;
                            s_i32_0 = 16i32;
                            vm.enter_call()?;
                            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                            vm.exit_call();
                            let __t43 = __call_res?;
                            s_i32_0 = __t43;
                            l62 = s_i32_0;
                            s_i32_0 = l62;
                            s_i32_1 = l62;
                            s_i32_2 = 16i32;
                            s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
                            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                            s_i32_0 = l62;
                            s_i32_1 = 0i32;
                            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                            s_i32_0 = l62;
                            s_i32_1 = 0i32;
                            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                            s_i32_0 = l62;
                            s_i32_1 = 1i32;
                            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                            s_i32_0 = l62;
                            l63 = s_i32_0;
                            s_i64_0 = l1;
                            l64 = s_i64_0;
                            s_i32_0 = 16i32;
                            vm.enter_call()?;
                            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                            vm.exit_call();
                            let __t44 = __call_res?;
                            s_i32_0 = __t44;
                            l65 = s_i32_0;
                            s_i32_0 = l65;
                            s_i32_1 = l61;
                            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                            s_i32_0 = l65;
                            s_i32_1 = l63;
                            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                            s_i32_0 = l65;
                            s_i64_1 = l64;
                            ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
                            s_i32_0 = l65;
                            l66 = s_i32_0;
                            s_i32_0 = l66;
                            return Ok(s_i32_0);
                        } else {
                        }
                    }
                    vm.deduct(host, 1)?;
                    s_i32_0 = l45;
                    s_i32_1 = 127i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_304f22c4ab8c(vm, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    let __t45 = __call_res?;
                    s_i32_0 = __t45;
                    vm.enter_call()?;
                    let __call_res = Self::s_b8acef2c4d16(vm, s_i32_0, host);
                    vm.exit_call();
                    let __t46 = __call_res?;
                    s_i32_0 = __t46;
                    l67 = s_i32_0;
                    s_i32_0 = l24;
                    s_i32_1 = l54;
                    s_i32_2 = 15i32;
                    s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
                    s_i32_2 = 255i32;
                    s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
                    s_i32_2 = 4i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_1e670f572c93(vm, s_i32_1, s_i32_2, host);
                    vm.exit_call();
                    let __t47 = __call_res?;
                    s_i32_1 = __t47;
                    s_i32_2 = l67;
                    s_i32_3 = 2i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_708c3053a6ba(vm, s_i32_2, s_i32_3, host);
                    vm.exit_call();
                    let __t48 = __call_res?;
                    s_i32_2 = __t48;
                    s_i32_1 = ops::i32_or(s_i32_1, s_i32_2);
                    s_i32_2 = 255i32;
                    s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_5daf123ed54b(vm, s_i32_0, s_i32_1, host);
                    vm.exit_call();
                    __call_res?;
                    s_i32_0 = l46;
                    s_i32_1 = 195i32;
                    s_i32_0 = ops::i32_ne(s_i32_0, s_i32_1);
                    'blk_21: {
                        if s_i32_0 != 0 {
                            s_i32_0 = l46;
                            s_i32_1 = 127i32;
                            s_i32_0 = ops::i32_lt_u(s_i32_0, s_i32_1);
                            'blk_22: {
                                if s_i32_0 != 0 {
                                    s_i32_0 = 1i32;
                                } else {
                                    s_i32_0 = l46;
                                    s_i32_1 = 190i32;
                                    s_i32_0 = ops::i32_gt_u(s_i32_0, s_i32_1);
                                }
                            }
                            'blk_23: {
                                if s_i32_0 != 0 {
                                    s_i32_0 = 0i32;
                                    l68 = s_i32_0;
                                    s_i32_0 = 16i32;
                                    vm.enter_call()?;
                                    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                                    vm.exit_call();
                                    let __t49 = __call_res?;
                                    s_i32_0 = __t49;
                                    l69 = s_i32_0;
                                    s_i32_0 = l69;
                                    s_i32_1 = l69;
                                    s_i32_2 = 16i32;
                                    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
                                    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                                    s_i32_0 = l69;
                                    s_i32_1 = 0i32;
                                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                                    s_i32_0 = l69;
                                    s_i32_1 = 0i32;
                                    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                                    s_i32_0 = l69;
                                    s_i32_1 = 1i32;
                                    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
                                    s_i32_0 = l69;
                                    l70 = s_i32_0;
                                    s_i64_0 = l1;
                                    l71 = s_i64_0;
                                    s_i32_0 = 16i32;
                                    vm.enter_call()?;
                                    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                                    vm.exit_call();
                                    let __t50 = __call_res?;
                                    s_i32_0 = __t50;
                                    l72 = s_i32_0;
                                    s_i32_0 = l72;
                                    s_i32_1 = l68;
                                    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                                    s_i32_0 = l72;
                                    s_i32_1 = l70;
                                    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                                    s_i32_0 = l72;
                                    s_i64_1 = l71;
                                    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
                                    s_i32_0 = l72;
                                    l73 = s_i32_0;
                                    s_i32_0 = l73;
                                    return Ok(s_i32_0);
                                } else {
                                }
                            }
                            s_i32_0 = l46;
                            s_i32_1 = 127i32;
                            vm.enter_call()?;
                            let __call_res = crate::shared_0::Shared::s_304f22c4ab8c(vm, s_i32_0, s_i32_1, host);
                            vm.exit_call();
                            let __t51 = __call_res?;
                            s_i32_0 = __t51;
                            vm.enter_call()?;
                            let __call_res = Self::s_b8acef2c4d16(vm, s_i32_0, host);
                            vm.exit_call();
                            let __t52 = __call_res?;
                            s_i32_0 = __t52;
                            l74 = s_i32_0;
                            s_i32_0 = l24;
                            s_i32_1 = l67;
                            s_i32_2 = 3i32;
                            s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
                            s_i32_2 = 255i32;
                            s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
                            s_i32_2 = 6i32;
                            vm.enter_call()?;
                            let __call_res = crate::shared_0::Shared::s_1e670f572c93(vm, s_i32_1, s_i32_2, host);
                            vm.exit_call();
                            let __t53 = __call_res?;
                            s_i32_1 = __t53;
                            s_i32_2 = l74;
                            s_i32_1 = ops::i32_or(s_i32_1, s_i32_2);
                            s_i32_2 = 255i32;
                            s_i32_1 = ops::i32_and(s_i32_1, s_i32_2);
                            vm.enter_call()?;
                            let __call_res = crate::shared_0::Shared::s_5daf123ed54b(vm, s_i32_0, s_i32_1, host);
                            vm.exit_call();
                            __call_res?;
                        } else {
                        }
                    }
                }
            }
            s_i64_0 = l25;
            s_i64_1 = 4i64;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
            vm.exit_call();
            let __t54 = __call_res?;
            s_i64_0 = __t54;
            l25 = s_i64_0;
            vm.deduct(host, 1)?;
            continue 'blk_9;
            break 'blk_9;
        }
    }
    s_i32_0 = 0i32;
    l75 = s_i32_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
    vm.exit_call();
    let __t55 = __call_res?;
    s_i32_0 = __t55;
    l76 = s_i32_0;
    s_i32_0 = l76;
    s_i32_1 = l76;
    s_i32_2 = 16i32;
    s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
    ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l76;
    s_i32_1 = 0i32;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l76;
    s_i32_1 = 0i32;
    ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
    s_i32_0 = l76;
    s_i32_1 = 1i32;
    ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
    s_i32_0 = l76;
    l77 = s_i32_0;
    s_i64_0 = l1;
    l78 = s_i64_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
    vm.exit_call();
    let __t56 = __call_res?;
    s_i32_0 = __t56;
    l79 = s_i32_0;
    s_i32_0 = l79;
    s_i32_1 = l75;
    ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
    s_i32_0 = l79;
    s_i32_1 = l77;
    ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
    s_i32_0 = l79;
    s_i64_1 = l78;
    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
    s_i32_0 = l79;
    l80 = s_i32_0;
    s_i32_0 = l80;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_f6eb7e45de1d(vm: &mut SubstituteVm, p0: i32, g0: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = 0i32;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    vm.enter_call()?;
    let __call_res = Self::s_c31be4b7b29d(vm, s_i32_0, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = vm.get_i32(g0)?;
        } else {
            s_i32_0 = l0;
            s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
            s_i32_0 = ops::i32_eqz(s_i32_0);
            l1 = s_i32_0;
            s_i32_0 = l0;
            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
            l2 = s_i32_0;
            s_i32_0 = 8i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            l3 = s_i32_0;
            s_i32_0 = l3;
            s_i32_1 = l1;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l3;
            s_i32_1 = l2;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l3;
        }
    }
    l4 = s_i32_0;
    s_i32_0 = l4;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_f71eec8c32bf(vm: &mut SubstituteVm, p0: i32, g0: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut l1: i32 = 0i32;
    let mut l2: i32 = 0i32;
    let mut l3: i32 = 0i32;
    let mut l4: i32 = 0i32;
    let mut l5: i32 = 0i32;
    let mut l6: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    let mut s_i32_2: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    s_i32_1 = 0i32;
    s_i32_0 = ops::i32_eq(s_i32_0, s_i32_1);
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = vm.get_i32(g0)?;
        } else {
            s_i32_0 = 0i32;
            l1 = s_i32_0;
            s_i32_0 = l0;
            l2 = s_i32_0;
            s_i32_0 = 20i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t0 = __call_res?;
            s_i32_0 = __t0;
            l3 = s_i32_0;
            s_i32_0 = l3;
            s_i32_1 = l3;
            s_i32_2 = 16i32;
            s_i32_1 = ops::i32_add(s_i32_1, s_i32_2);
            ops::i32_store(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l3;
            s_i32_1 = 1i32;
            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
            s_i32_0 = l3;
            s_i32_1 = 1i32;
            ops::i32_store(vm, 0, s_i32_0, 8u32, s_i32_1)?;
            s_i32_0 = l3;
            s_i32_1 = 1i32;
            ops::i32_store(vm, 0, s_i32_0, 12u32, s_i32_1)?;
            s_i32_0 = l3;
            s_i32_1 = l2;
            ops::i32_store(vm, 0, s_i32_0, 16u32, s_i32_1)?;
            s_i32_0 = l3;
            l4 = s_i32_0;
            s_i32_0 = 8i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            l5 = s_i32_0;
            s_i32_0 = l5;
            s_i32_1 = l1;
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

    pub(crate) fn s_f76e9849d691(vm: &mut SubstituteVm, p0: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
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
    s_i32_1 = l0;
    s_i64_1 = ops::i64_load(vm, 0, s_i32_1, 8u32)?;
    let __t0 = s_i64_0;
    let __t1 = s_i64_1;
    return Self::s_b3e6ca84b2de(vm, __t0, __t1, host);
    }

    pub(crate) fn s_f7d297dbbc03(vm: &mut SubstituteVm, p0: i32, g0: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i32 = p0;
    let mut s_i32_0: i32 = 0i32;
    let mut s_i32_1: i32 = 0i32;
    vm.deduct(host, 1)?;
    vm.enter_call()?;
    let __call_res = vm.call_init(host);
    vm.exit_call();
    __call_res?;
    s_i32_0 = l0;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_51f0c0b4c723(vm, s_i32_0, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i32_0 = __t0;
    'blk_1: {
        if s_i32_0 != 0 {
            s_i32_0 = vm.get_i32(g0)?;
            s_i32_1 = l0;
            vm.enter_call()?;
            let __call_res = Self::s_f76e9849d691(vm, s_i32_1, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_1 = __t1;
            let __t2 = s_i32_0;
            let __t3 = s_i32_1;
            return Self::s_95ed5cb908ad(vm, __t2, __t3, host);
        } else {
            s_i32_0 = l0;
            let __t4 = s_i32_0;
            return Self::s_f76e9849d691(vm, __t4, host);
        }
    }
    return Err(WasmTrap::Unreachable);
    }

    #[inline(never)]
    pub(crate) fn s_fa6af9f06252(vm: &mut SubstituteVm, p0: i32, p1: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
                    let __call_res = crate::shared_0::Shared::s_52dcb6d97135(vm, s_i64_0, s_i32_1, host);
                    vm.exit_call();
                    let __t0 = __call_res?;
                    s_i64_0 = __t0;
                    l4 = s_i64_0;
                    s_i64_0 = l3;
                    s_i32_1 = l1;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_52dcb6d97135(vm, s_i64_0, s_i32_1, host);
                    vm.exit_call();
                    let __t1 = __call_res?;
                    s_i64_0 = __t1;
                    s_i64_1 = l2;
                    s_i32_2 = 64i32;
                    s_i32_3 = l1;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_304f22c4ab8c(vm, s_i32_2, s_i32_3, host);
                    vm.exit_call();
                    let __t2 = __call_res?;
                    s_i32_2 = __t2;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_73a866edda27(vm, s_i64_1, s_i32_2, host);
                    vm.exit_call();
                    let __t3 = __call_res?;
                    s_i64_1 = __t3;
                    s_i64_0 = ops::i64_or(s_i64_0, s_i64_1);
                    l5 = s_i64_0;
                    s_i32_0 = 16i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
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
                            s_i64_0 = 0i64;
                            l7 = s_i64_0;
                            s_i64_0 = l2;
                            l8 = s_i64_0;
                            s_i32_0 = 16i32;
                            vm.enter_call()?;
                            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
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
                            s_i64_0 = 0i64;
                            l10 = s_i64_0;
                            s_i64_0 = l2;
                            s_i32_1 = l1;
                            s_i32_2 = 64i32;
                            vm.enter_call()?;
                            let __call_res = crate::shared_0::Shared::s_304f22c4ab8c(vm, s_i32_1, s_i32_2, host);
                            vm.exit_call();
                            let __t6 = __call_res?;
                            s_i32_1 = __t6;
                            vm.enter_call()?;
                            let __call_res = crate::shared_0::Shared::s_52dcb6d97135(vm, s_i64_0, s_i32_1, host);
                            vm.exit_call();
                            let __t7 = __call_res?;
                            s_i64_0 = __t7;
                            l11 = s_i64_0;
                            s_i32_0 = 16i32;
                            vm.enter_call()?;
                            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
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

    #[inline(never)]
    pub(crate) fn s_fbbf699c9d44(vm: &mut SubstituteVm, p0: i64, p1: i64, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
    let mut l0: i64 = p0;
    let mut l1: i64 = p1;
    let mut l2: i64 = 0i64;
    let mut l3: i64 = 0i64;
    let mut l4: i64 = 0i64;
    let mut l5: i64 = 0i64;
    let mut l6: i64 = 0i64;
    let mut l7: i64 = 0i64;
    let mut l8: i64 = 0i64;
    let mut l9: i64 = 0i64;
    let mut l10: i64 = 0i64;
    let mut l11: i64 = 0i64;
    let mut l12: i64 = 0i64;
    let mut l13: i64 = 0i64;
    let mut l14: i64 = 0i64;
    let mut l15: i64 = 0i64;
    let mut l16: i32 = 0i32;
    let mut l17: i32 = 0i32;
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
    s_i64_0 = 4294967295i64;
    l2 = s_i64_0;
    s_i64_0 = l0;
    s_i64_1 = l2;
    s_i64_0 = ops::i64_and(s_i64_0, s_i64_1);
    l3 = s_i64_0;
    s_i64_0 = l0;
    s_i32_1 = 32i32;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_73a866edda27(vm, s_i64_0, s_i32_1, host);
    vm.exit_call();
    let __t0 = __call_res?;
    s_i64_0 = __t0;
    l4 = s_i64_0;
    s_i64_0 = l1;
    s_i64_1 = l2;
    s_i64_0 = ops::i64_and(s_i64_0, s_i64_1);
    l5 = s_i64_0;
    s_i64_0 = l1;
    s_i32_1 = 32i32;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_73a866edda27(vm, s_i64_0, s_i32_1, host);
    vm.exit_call();
    let __t1 = __call_res?;
    s_i64_0 = __t1;
    l6 = s_i64_0;
    s_i64_0 = l3;
    s_i64_1 = l5;
    vm.enter_call()?;
    let __call_res = Self::s_8dcb06005c72(vm, s_i64_0, s_i64_1, host);
    vm.exit_call();
    let __t2 = __call_res?;
    s_i64_0 = __t2;
    l7 = s_i64_0;
    s_i64_0 = l3;
    s_i64_1 = l6;
    vm.enter_call()?;
    let __call_res = Self::s_8dcb06005c72(vm, s_i64_0, s_i64_1, host);
    vm.exit_call();
    let __t3 = __call_res?;
    s_i64_0 = __t3;
    l8 = s_i64_0;
    s_i64_0 = l4;
    s_i64_1 = l5;
    vm.enter_call()?;
    let __call_res = Self::s_8dcb06005c72(vm, s_i64_0, s_i64_1, host);
    vm.exit_call();
    let __t4 = __call_res?;
    s_i64_0 = __t4;
    l9 = s_i64_0;
    s_i64_0 = l4;
    s_i64_1 = l6;
    vm.enter_call()?;
    let __call_res = Self::s_8dcb06005c72(vm, s_i64_0, s_i64_1, host);
    vm.exit_call();
    let __t5 = __call_res?;
    s_i64_0 = __t5;
    l10 = s_i64_0;
    s_i64_0 = l7;
    s_i32_1 = 32i32;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_73a866edda27(vm, s_i64_0, s_i32_1, host);
    vm.exit_call();
    let __t6 = __call_res?;
    s_i64_0 = __t6;
    s_i64_1 = l8;
    s_i64_2 = l2;
    s_i64_1 = ops::i64_and(s_i64_1, s_i64_2);
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
    vm.exit_call();
    let __t7 = __call_res?;
    s_i64_0 = __t7;
    s_i64_1 = l9;
    s_i64_2 = l2;
    s_i64_1 = ops::i64_and(s_i64_1, s_i64_2);
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
    vm.exit_call();
    let __t8 = __call_res?;
    s_i64_0 = __t8;
    l11 = s_i64_0;
    s_i64_0 = l7;
    s_i64_1 = l2;
    s_i64_0 = ops::i64_and(s_i64_0, s_i64_1);
    s_i64_1 = l11;
    s_i64_2 = l2;
    s_i64_1 = ops::i64_and(s_i64_1, s_i64_2);
    s_i32_2 = 32i32;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_52dcb6d97135(vm, s_i64_1, s_i32_2, host);
    vm.exit_call();
    let __t9 = __call_res?;
    s_i64_1 = __t9;
    s_i64_0 = ops::i64_or(s_i64_0, s_i64_1);
    l12 = s_i64_0;
    s_i64_0 = l10;
    s_i64_1 = l8;
    s_i32_2 = 32i32;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_73a866edda27(vm, s_i64_1, s_i32_2, host);
    vm.exit_call();
    let __t10 = __call_res?;
    s_i64_1 = __t10;
    s_i64_2 = l9;
    s_i32_3 = 32i32;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_73a866edda27(vm, s_i64_2, s_i32_3, host);
    vm.exit_call();
    let __t11 = __call_res?;
    s_i64_2 = __t11;
    s_i64_3 = l11;
    s_i32_4 = 32i32;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_73a866edda27(vm, s_i64_3, s_i32_4, host);
    vm.exit_call();
    let __t12 = __call_res?;
    s_i64_3 = __t12;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_1c917789866c(vm, s_i64_2, s_i64_3, host);
    vm.exit_call();
    let __t13 = __call_res?;
    s_i64_2 = __t13;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_1c917789866c(vm, s_i64_1, s_i64_2, host);
    vm.exit_call();
    let __t14 = __call_res?;
    s_i64_1 = __t14;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_1c917789866c(vm, s_i64_0, s_i64_1, host);
    vm.exit_call();
    let __t15 = __call_res?;
    s_i64_0 = __t15;
    l13 = s_i64_0;
    s_i64_0 = l12;
    l14 = s_i64_0;
    s_i64_0 = l13;
    l15 = s_i64_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
    vm.exit_call();
    let __t16 = __call_res?;
    s_i32_0 = __t16;
    l16 = s_i32_0;
    s_i32_0 = l16;
    s_i64_1 = l14;
    ops::i64_store(vm, 0, s_i32_0, 0u32, s_i64_1)?;
    s_i32_0 = l16;
    s_i64_1 = l15;
    ops::i64_store(vm, 0, s_i32_0, 8u32, s_i64_1)?;
    s_i32_0 = l16;
    l17 = s_i32_0;
    s_i32_0 = l17;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_fc6a710e7a01(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, g1: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let __call_res = crate::shared_0::Shared::s_3cfae8586ad0(vm, s_i32_0, s_i32_1, g0, g1, host);
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

    pub(crate) fn s_fca97fe5a062(vm: &mut SubstituteVm, p0: i32, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    s_i64_0 = ops::i64_extend_i32_u(s_i32_0);
    l1 = s_i64_0;
    s_i64_0 = 0i64;
    l2 = s_i64_0;
    s_i32_0 = 16i32;
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
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

    pub(crate) fn s_fd100d57cfdc(vm: &mut SubstituteVm, p0: i32, p1: i32, g0: usize, g1: usize, g2: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let __call_res = crate::shared_0::Shared::s_1010eaf60e55(vm, s_i32_0, s_i32_1, g0, g1, g2, host);
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
    pub(crate) fn s_ffcc61d1c500(vm: &mut SubstituteVm, p0: i32, g0: usize, g1: usize, g2: usize, g3: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
    let mut l16: i32 = 0i32;
    let mut l17: i32 = 0i32;
    let mut s_i32_0: i32 = 0i32;
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
    vm.enter_call()?;
    let __call_res = crate::shared_0::Shared::s_0f7538eaa7db(vm, s_i32_0, g0, host);
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
            s_i32_0 = vm.get_i32(g1)?;
            l3 = s_i32_0;
            s_i32_0 = vm.get_i32(g2)?;
            l4 = s_i32_0;
            s_i32_0 = 12i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
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
    s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
    l7 = s_i32_0;
    s_i32_0 = l0;
    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
    'blk_2: {
        if s_i32_0 != 0 {
            s_i32_0 = l7;
            s_i64_1 = 0i64;
            s_i64_2 = i64::MIN;
            vm.enter_call()?;
            let __call_res = Self::s_b3e6ca84b2de(vm, s_i64_1, s_i64_2, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_1 = __t2;
            vm.enter_call()?;
            let __call_res = Self::s_84059c53f331(vm, s_i32_0, s_i32_1, host);
            vm.exit_call();
            let __t3 = __call_res?;
            s_i32_0 = __t3;
            'blk_3: {
                if s_i32_0 != 0 {
                    s_i32_0 = 1i32;
                    l8 = s_i32_0;
                    s_i32_0 = vm.get_i32(g3)?;
                    l9 = s_i32_0;
                    s_i32_0 = 0i32;
                    l10 = s_i32_0;
                    s_i32_0 = 12i32;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
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
                } else {
                    s_i32_0 = l7;
                    vm.enter_call()?;
                    let __call_res = crate::shared_0::Shared::s_14853a432ff7(vm, s_i32_0, g1, g2, host);
                    vm.exit_call();
                    let __t5 = __call_res?;
                    s_i32_0 = __t5;
                    l12 = s_i32_0;
                    s_i32_0 = l12;
                    s_i32_0 = ops::i32_load8_u(vm, 0, s_i32_0, 0u32)?;
                    s_i32_0 = ops::i32_eqz(s_i32_0);
                    'blk_4: {
                        if s_i32_0 != 0 {
                            s_i32_0 = 0i32;
                            l13 = s_i32_0;
                            s_i32_0 = vm.get_i32(g1)?;
                            l14 = s_i32_0;
                            s_i32_0 = vm.get_i32(g2)?;
                            l15 = s_i32_0;
                            s_i32_0 = 12i32;
                            vm.enter_call()?;
                            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
                            vm.exit_call();
                            let __t6 = __call_res?;
                            s_i32_0 = __t6;
                            l16 = s_i32_0;
                            s_i32_0 = l16;
                            s_i32_1 = l13;
                            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
                            s_i32_0 = l16;
                            s_i32_1 = l14;
                            ops::i32_store(vm, 0, s_i32_0, 4u32, s_i32_1)?;
                            s_i32_0 = l16;
                            s_i32_1 = l15;
                            ops::i32_store8(vm, 0, s_i32_0, 8u32, s_i32_1)?;
                            s_i32_0 = l16;
                        } else {
                            s_i32_0 = l12;
                            s_i32_0 = ops::i32_load(vm, 0, s_i32_0, 4u32)?;
                            let __t7 = s_i32_0;
                            return Self::s_e1d06f25ed5a(vm, __t7, g3, g1, g2, host);
                        }
                    }
                }
            }
        } else {
            s_i32_0 = l7;
            let __t8 = s_i32_0;
            return crate::shared_0::Shared::s_14853a432ff7(vm, __t8, g1, g2, host);
        }
    }
    l17 = s_i32_0;
    s_i32_0 = l17;
    return Ok(s_i32_0);
    }

    pub(crate) fn s_ffe97171ec9d(vm: &mut SubstituteVm, p0: i32, g0: usize, host: &mut dyn WasmSubstituteHost) -> std::result::Result<i32, WasmTrap> {
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
            s_i64_1 = 255i64;
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
            let __call_res = Self::s_b5672010fcc9(vm, s_i64_0, host);
            vm.exit_call();
            let __t0 = __call_res?;
            s_i32_0 = __t0;
            l2 = s_i32_0;
            s_i32_0 = 0i32;
            l3 = s_i32_0;
            s_i32_0 = 4i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t1 = __call_res?;
            s_i32_0 = __t1;
            l4 = s_i32_0;
            s_i32_0 = l4;
            s_i32_1 = l1;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l4;
            s_i32_1 = l2;
            ops::i32_store8(vm, 0, s_i32_0, 1u32, s_i32_1)?;
            s_i32_0 = l4;
            s_i32_1 = l3;
            ops::i32_store8(vm, 0, s_i32_0, 2u32, s_i32_1)?;
            s_i32_0 = l4;
        } else {
            s_i32_0 = 0i32;
            l5 = s_i32_0;
            s_i32_0 = 0i32;
            l6 = s_i32_0;
            s_i32_0 = vm.get_i32(g0)?;
            l7 = s_i32_0;
            s_i32_0 = 4i32;
            vm.enter_call()?;
            let __call_res = crate::shared_0::Shared::s_498abb70f200(vm, s_i32_0, host);
            vm.exit_call();
            let __t2 = __call_res?;
            s_i32_0 = __t2;
            l8 = s_i32_0;
            s_i32_0 = l8;
            s_i32_1 = l5;
            ops::i32_store8(vm, 0, s_i32_0, 0u32, s_i32_1)?;
            s_i32_0 = l8;
            s_i32_1 = l6;
            ops::i32_store8(vm, 0, s_i32_0, 1u32, s_i32_1)?;
            s_i32_0 = l8;
            s_i32_1 = l7;
            ops::i32_store8(vm, 0, s_i32_0, 2u32, s_i32_1)?;
            s_i32_0 = l8;
        }
    }
    l9 = s_i32_0;
    s_i32_0 = l9;
    return Ok(s_i32_0);
    }

}
