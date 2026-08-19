// SPDX-License-Identifier: AGPL-3.0-or-later
/*
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

//! `LD_PRELOAD` shared object intercepting `personality()` and `setuid()` syscalls.
//!
//! Ignores `EPERM` failures when setting personality, uid/gid, or chroot
//! in unprivileged build containers during Guix image builds, and prevents
//! `LD_PRELOAD` from leaking into child processes spawned by `guix-daemon`.

use std::ffi::{c_char, c_int, c_long, c_ulong, c_void};

const RTLD_NEXT: *mut c_void =
    std::ptr::null_mut::<c_void>().wrapping_offset(-1);

#[expect(
    unsafe_code,
    reason = "LD_PRELOAD shared library interfacing with dlsym C FFI"
)]
unsafe extern "C" {
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    fn unsetenv(name: *const c_char) -> c_int;
}

#[expect(
    unsafe_code,
    reason = "ELF .init_array constructor to clear LD_PRELOAD from process environment"
)]
#[unsafe(link_section = ".init_array")]
#[used]
static INIT: unsafe extern "C" fn() = {
    unsafe extern "C" fn init() {
        let name = c"LD_PRELOAD";
        // SAFETY: Calling C library unsetenv with valid null-terminated string
        // so that spawned child processes do not inherit LD_PRELOAD.
        unsafe {
            unsetenv(name.as_ptr());
        }
    }
    init
};

type OrigPersonalityFn = unsafe extern "C" fn(c_ulong) -> c_long;

/// Intercept `personality` syscall to ignore `EPERM` errors.
///
/// # Safety
///
/// Calls `dlsym` to resolve `personality` in `RTLD_NEXT` and invokes it.
#[unsafe(no_mangle)]
#[expect(
    unsafe_code,
    reason = "LD_PRELOAD shared library export overriding personality syscall"
)]
pub unsafe extern "C" fn personality(persona: c_ulong) -> c_long {
    let symbol_name = c"personality";
    // SAFETY: Resolving personality symbol from RTLD_NEXT via dlsym.
    let orig_ptr = unsafe { dlsym(RTLD_NEXT, symbol_name.as_ptr()) };
    if !orig_ptr.is_null() {
        // SAFETY: dlsym returned a non-null pointer matching OrigPersonalityFn.
        let orig: OrigPersonalityFn = unsafe { std::mem::transmute(orig_ptr) };
        // SAFETY: Invoking the resolved original personality function.
        let res = unsafe { orig(persona) };
        if res < 0 {
            return 0;
        }
        return res;
    }
    0
}

type OrigKillFn = unsafe extern "C" fn(c_int, c_int) -> c_int;

/// Intercept `kill` syscall to guard against `kill(-1, sig)` terminating
/// all processes across the entire container.
///
/// # Safety
///
/// Calls `dlsym` to resolve `kill` in `RTLD_NEXT` and invokes it.
#[unsafe(no_mangle)]
#[expect(
    unsafe_code,
    reason = "LD_PRELOAD shared library export overriding kill syscall"
)]
pub unsafe extern "C" fn kill(pid: c_int, sig: c_int) -> c_int {
    if pid == -1 {
        return 0;
    }
    let symbol_name = c"kill";
    // SAFETY: Resolving kill symbol from RTLD_NEXT via dlsym.
    let orig_ptr = unsafe { dlsym(RTLD_NEXT, symbol_name.as_ptr()) };
    if !orig_ptr.is_null() {
        // SAFETY: dlsym returned a non-null pointer matching OrigKillFn.
        let orig: OrigKillFn = unsafe { std::mem::transmute(orig_ptr) };
        // SAFETY: Invoking the resolved original kill function.
        return unsafe { orig(pid, sig) };
    }
    0
}

