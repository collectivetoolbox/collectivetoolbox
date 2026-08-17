//! `LD_PRELOAD` shared object intercepting `personality()` and `setuid()` syscalls.
//!
//! Ignores `EPERM` failures when setting personality, uid/gid, or chroot
//! in unprivileged build containers during Guix image builds, and prevents
//! `LD_PRELOAD` from leaking into child processes spawned by `guix-daemon`.

use std::ffi::{c_char, c_int, c_long, c_uint, c_ulong, c_void};

const RTLD_NEXT: *mut c_void =
    std::ptr::null_mut::<c_void>().wrapping_offset(-1);

#[expect(
    unsafe_code,
    reason = "LD_PRELOAD shared library interfacing with dlsym C FFI"
)]
unsafe extern "C" {
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
}

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

type OrigUidGidFn = unsafe extern "C" fn(c_uint) -> c_int;
type OrigResUidGidFn =
    unsafe extern "C" fn(c_uint, c_uint, c_uint) -> c_int;
type OrigSetgroupsFn = unsafe extern "C" fn(usize, *const c_uint) -> c_int;

#[unsafe(no_mangle)]
#[expect(
    unsafe_code,
    reason = "LD_PRELOAD shared library export overriding setuid syscall"
)]
pub unsafe extern "C" fn setuid(uid: c_uint) -> c_int {
    let symbol_name = c"setuid";
    let orig_ptr = unsafe { dlsym(RTLD_NEXT, symbol_name.as_ptr()) };
    if !orig_ptr.is_null() {
        let orig: OrigUidGidFn = unsafe { std::mem::transmute(orig_ptr) };
        let res = unsafe { orig(uid) };
        if res < 0 {
            return 0;
        }
        return res;
    }
    0
}

#[unsafe(no_mangle)]
#[expect(
    unsafe_code,
    reason = "LD_PRELOAD shared library export overriding seteuid syscall"
)]
pub unsafe extern "C" fn seteuid(euid: c_uint) -> c_int {
    let symbol_name = c"seteuid";
    let orig_ptr = unsafe { dlsym(RTLD_NEXT, symbol_name.as_ptr()) };
    if !orig_ptr.is_null() {
        let orig: OrigUidGidFn = unsafe { std::mem::transmute(orig_ptr) };
        let res = unsafe { orig(euid) };
        if res < 0 {
            return 0;
        }
        return res;
    }
    0
}

#[unsafe(no_mangle)]
#[expect(
    unsafe_code,
    reason = "LD_PRELOAD shared library export overriding setgid syscall"
)]
pub unsafe extern "C" fn setgid(gid: c_uint) -> c_int {
    let symbol_name = c"setgid";
    let orig_ptr = unsafe { dlsym(RTLD_NEXT, symbol_name.as_ptr()) };
    if !orig_ptr.is_null() {
        let orig: OrigUidGidFn = unsafe { std::mem::transmute(orig_ptr) };
        let res = unsafe { orig(gid) };
        if res < 0 {
            return 0;
        }
        return res;
    }
    0
}

#[unsafe(no_mangle)]
#[expect(
    unsafe_code,
    reason = "LD_PRELOAD shared library export overriding setegid syscall"
)]
pub unsafe extern "C" fn setegid(egid: c_uint) -> c_int {
    let symbol_name = c"setegid";
    let orig_ptr = unsafe { dlsym(RTLD_NEXT, symbol_name.as_ptr()) };
    if !orig_ptr.is_null() {
        let orig: OrigUidGidFn = unsafe { std::mem::transmute(orig_ptr) };
        let res = unsafe { orig(egid) };
        if res < 0 {
            return 0;
        }
        return res;
    }
    0
}

#[unsafe(no_mangle)]
#[expect(
    unsafe_code,
    reason = "LD_PRELOAD shared library export overriding setresuid syscall"
)]
pub unsafe extern "C" fn setresuid(
    ruid: c_uint,
    euid: c_uint,
    suid: c_uint,
) -> c_int {
    let symbol_name = c"setresuid";
    let orig_ptr = unsafe { dlsym(RTLD_NEXT, symbol_name.as_ptr()) };
    if !orig_ptr.is_null() {
        let orig: OrigResUidGidFn = unsafe { std::mem::transmute(orig_ptr) };
        let res = unsafe { orig(ruid, euid, suid) };
        if res < 0 {
            return 0;
        }
        return res;
    }
    0
}

#[unsafe(no_mangle)]
#[expect(
    unsafe_code,
    reason = "LD_PRELOAD shared library export overriding setresgid syscall"
)]
pub unsafe extern "C" fn setresgid(
    rgid: c_uint,
    egid: c_uint,
    sgid: c_uint,
) -> c_int {
    let symbol_name = c"setresgid";
    let orig_ptr = unsafe { dlsym(RTLD_NEXT, symbol_name.as_ptr()) };
    if !orig_ptr.is_null() {
        let orig: OrigResUidGidFn = unsafe { std::mem::transmute(orig_ptr) };
        let res = unsafe { orig(rgid, egid, sgid) };
        if res < 0 {
            return 0;
        }
        return res;
    }
    0
}

#[unsafe(no_mangle)]
#[expect(
    unsafe_code,
    reason = "LD_PRELOAD shared library export overriding setgroups syscall"
)]
pub unsafe extern "C" fn setgroups(size: usize, list: *const c_uint) -> c_int {
    let symbol_name = c"setgroups";
    let orig_ptr = unsafe { dlsym(RTLD_NEXT, symbol_name.as_ptr()) };
    if !orig_ptr.is_null() {
        let orig: OrigSetgroupsFn = unsafe { std::mem::transmute(orig_ptr) };
        let res = unsafe { orig(size, list) };
        if res < 0 {
            return 0;
        }
        return res;
    }
    0
}
