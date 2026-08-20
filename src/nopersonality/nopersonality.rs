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

//! `LD_PRELOAD` shared object intercepting `personality()` and `kill()` syscalls.
//!
//! Ignores `EPERM` failures when setting personality in unprivileged build
//! containers during Guix image builds, and guards against `kill(-1, sig)`.

#![no_std]

use core::arch::asm;
use core::ffi::{c_int, c_long, c_ulong};

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[expect(
    unsafe_code,
    clippy::missing_safety_doc,
    reason = "Issuing direct syscall with inline assembly"
)]
#[inline(always)]
unsafe fn syscall1(num: c_long, arg1: c_long) -> c_long {
    let ret: c_long;
    // SAFETY: Issuing 1-argument syscall with clobbered rcx and r11.
    unsafe {
        asm!(
            "syscall",
            inout("rax") num => ret,
            in("rdi") arg1,
            out("rcx") _,
            out("r11") _,
            options(nostack, preserves_flags)
        );
    }
    ret
}

#[expect(
    unsafe_code,
    clippy::missing_safety_doc,
    reason = "Issuing direct syscall with inline assembly"
)]
#[inline(always)]
unsafe fn syscall2(num: c_long, arg1: c_long, arg2: c_long) -> c_long {
    let ret: c_long;
    // SAFETY: Issuing 2-argument syscall with clobbered rcx and r11.
    unsafe {
        asm!(
            "syscall",
            inout("rax") num => ret,
            in("rdi") arg1,
            in("rsi") arg2,
            out("rcx") _,
            out("r11") _,
            options(nostack, preserves_flags)
        );
    }
    ret
}

#[expect(
    unsafe_code,
    clippy::missing_safety_doc,
    reason = "Issuing direct syscall with inline assembly"
)]
#[inline(always)]
unsafe fn syscall3(
    num: c_long,
    arg1: c_long,
    arg2: c_long,
    arg3: c_long,
) -> c_long {
    let ret: c_long;
    // SAFETY: Issuing 3-argument syscall with clobbered rcx and r11.
    unsafe {
        asm!(
            "syscall",
            inout("rax") num => ret,
            in("rdi") arg1,
            in("rsi") arg2,
            in("rdx") arg3,
            out("rcx") _,
            out("r11") _,
            options(nostack, preserves_flags)
        );
    }
    ret
}

const SYS_KILL: c_long = 62;
const SYS_SETUID: c_long = 105;
const SYS_SETGID: c_long = 106;
const SYS_SETREUID: c_long = 113;
const SYS_SETREGID: c_long = 114;
const SYS_SETGROUPS: c_long = 116;
const SYS_SETRESUID: c_long = 117;
const SYS_SETRESGID: c_long = 119;
const SYS_PERSONALITY: c_long = 135;

/// Intercept `personality` syscall to ignore `EPERM` errors.
///
/// # Safety
///
/// Invokes the `personality` syscall directly.
#[unsafe(no_mangle)]
#[expect(
    unsafe_code,
    reason = "LD_PRELOAD shared library export overriding personality syscall"
)]
pub unsafe extern "C" fn personality(persona: c_ulong) -> c_long {
    // Reason for fallback: Out-of-range ulong personas map to invalid -1 arg.
    let persona_long = i64::try_from(persona).unwrap_or(-1);
    // SAFETY: Calling personality syscall.
    let res = unsafe { syscall1(SYS_PERSONALITY, persona_long) };
    if res < 0 {
        return 0;
    }
    res
}

/// Intercept `kill` syscall to guard against `kill(-1, sig)` terminating
/// all processes across the entire container.
///
/// # Safety
///
/// Invokes the `kill` syscall directly.
#[unsafe(no_mangle)]
#[expect(
    unsafe_code,
    reason = "LD_PRELOAD shared library export overriding kill syscall"
)]
pub unsafe extern "C" fn kill(pid: c_int, sig: c_int) -> c_int {
    if pid == -1 {
        return 0;
    }
    // SAFETY: Calling kill syscall.
    let res = unsafe { syscall2(SYS_KILL, c_long::from(pid), c_long::from(sig)) };
    // Reason for fallback: Out-of-range syscall return value defaults to -1 errno.
    c_int::try_from(res).unwrap_or(-1)
}

/// Intercept `setuid` syscall to ignore `EPERM` failures in container builds.
///
/// # Safety
///
/// Invokes the `setuid` syscall directly.
#[unsafe(no_mangle)]
#[expect(
    unsafe_code,
    reason = "LD_PRELOAD shared library export overriding setuid syscall"
)]
pub unsafe extern "C" fn setuid(uid: core::ffi::c_uint) -> c_int {
    // SAFETY: Calling setuid syscall.
    let res = unsafe { syscall1(SYS_SETUID, c_long::from(uid)) };
    if res < 0 {
        return 0;
    }
    0
}

/// Intercept `setgid` syscall to ignore `EPERM` failures in container builds.
///
/// # Safety
///
/// Invokes the `setgid` syscall directly.
#[unsafe(no_mangle)]
#[expect(
    unsafe_code,
    reason = "LD_PRELOAD shared library export overriding setgid syscall"
)]
pub unsafe extern "C" fn setgid(gid: core::ffi::c_uint) -> c_int {
    // SAFETY: Calling setgid syscall.
    let res = unsafe { syscall1(SYS_SETGID, c_long::from(gid)) };
    if res < 0 {
        return 0;
    }
    0
}

/// Intercept `setreuid` syscall to ignore `EPERM` failures in container builds.
///
/// # Safety
///
/// Invokes the `setreuid` syscall directly.
#[unsafe(no_mangle)]
#[expect(
    unsafe_code,
    reason = "LD_PRELOAD shared library export overriding setreuid syscall"
)]
pub unsafe extern "C" fn setreuid(
    ruid: core::ffi::c_uint,
    euid: core::ffi::c_uint,
) -> c_int {
    // SAFETY: Calling setreuid syscall.
    let res = unsafe {
        syscall2(SYS_SETREUID, c_long::from(ruid), c_long::from(euid))
    };
    if res < 0 {
        return 0;
    }
    0
}

/// Intercept `setregid` syscall to ignore `EPERM` failures in container builds.
///
/// # Safety
///
/// Invokes the `setregid` syscall directly.
#[unsafe(no_mangle)]
#[expect(
    unsafe_code,
    reason = "LD_PRELOAD shared library export overriding setregid syscall"
)]
pub unsafe extern "C" fn setregid(
    rgid: core::ffi::c_uint,
    egid: core::ffi::c_uint,
) -> c_int {
    // SAFETY: Calling setregid syscall.
    let res = unsafe {
        syscall2(SYS_SETREGID, c_long::from(rgid), c_long::from(egid))
    };
    if res < 0 {
        return 0;
    }
    0
}

/// Intercept `setresuid` syscall to ignore `EPERM` failures in container builds.
///
/// # Safety
///
/// Invokes the `setresuid` syscall directly.
#[unsafe(no_mangle)]
#[expect(
    unsafe_code,
    reason = "LD_PRELOAD shared library export overriding setresuid syscall"
)]
pub unsafe extern "C" fn setresuid(
    ruid: core::ffi::c_uint,
    euid: core::ffi::c_uint,
    suid: core::ffi::c_uint,
) -> c_int {
    // SAFETY: Calling setresuid syscall.
    let res = unsafe {
        syscall3(
            SYS_SETRESUID,
            c_long::from(ruid),
            c_long::from(euid),
            c_long::from(suid),
        )
    };
    if res < 0 {
        return 0;
    }
    0
}

/// Intercept `setresgid` syscall to ignore `EPERM` failures in container builds.
///
/// # Safety
///
/// Invokes the `setresgid` syscall directly.
#[unsafe(no_mangle)]
#[expect(
    unsafe_code,
    reason = "LD_PRELOAD shared library export overriding setresgid syscall"
)]
pub unsafe extern "C" fn setresgid(
    rgid: core::ffi::c_uint,
    egid: core::ffi::c_uint,
    sgid: core::ffi::c_uint,
) -> c_int {
    // SAFETY: Calling setresgid syscall.
    let res = unsafe {
        syscall3(
            SYS_SETRESGID,
            c_long::from(rgid),
            c_long::from(egid),
            c_long::from(sgid),
        )
    };
    if res < 0 {
        return 0;
    }
    0
}

/// Intercept `seteuid` syscall to ignore `EPERM` failures in container builds.
///
/// # Safety
///
/// Invokes the `setresuid` syscall directly.
#[unsafe(no_mangle)]
#[expect(
    unsafe_code,
    reason = "LD_PRELOAD shared library export overriding seteuid syscall"
)]
pub unsafe extern "C" fn seteuid(euid: core::ffi::c_uint) -> c_int {
    // SAFETY: Calling setresuid syscall with -1 for ruid and suid.
    let res = unsafe {
        syscall3(SYS_SETRESUID, -1, c_long::from(euid), -1)
    };
    if res < 0 {
        return 0;
    }
    0
}

/// Intercept `setegid` syscall to ignore `EPERM` failures in container builds.
///
/// # Safety
///
/// Invokes the `setresgid` syscall directly.
#[unsafe(no_mangle)]
#[expect(
    unsafe_code,
    reason = "LD_PRELOAD shared library export overriding setegid syscall"
)]
pub unsafe extern "C" fn setegid(egid: core::ffi::c_uint) -> c_int {
    // SAFETY: Calling setresgid syscall with -1 for rgid and sgid.
    let res = unsafe {
        syscall3(SYS_SETRESGID, -1, c_long::from(egid), -1)
    };
    if res < 0 {
        return 0;
    }
    0
}

/// Intercept `setgroups` syscall to ignore `EPERM` failures in container builds.
///
/// # Safety
///
/// Invokes the `setgroups` syscall directly.
#[unsafe(no_mangle)]
#[expect(
    unsafe_code,
    reason = "LD_PRELOAD shared library export overriding setgroups syscall"
)]
pub unsafe extern "C" fn setgroups(
    size: usize,
    list: *const core::ffi::c_uint,
) -> c_int {
    // Reason for fallback: Out-of-range size value maps to invalid -1 arg.
    let size_long = i64::try_from(size).unwrap_or(-1);
    // Reason for fallback: Out-of-range pointer address maps to 0.
    let list_addr = i64::try_from(list.addr()).unwrap_or(0);
    // SAFETY: Calling setgroups syscall.
    let res = unsafe { syscall2(SYS_SETGROUPS, size_long, list_addr) };
    if res < 0 {
        return 0;
    }
    0
}
