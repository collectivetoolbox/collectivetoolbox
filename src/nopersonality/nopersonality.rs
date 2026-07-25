//! `LD_PRELOAD` shared object intercepting `personality()` syscalls.
//!
//! Ignores `EPERM` failure when setting personality in unprivileged
//! build containers during Guix image builds.

use std::ffi::{c_char, c_long, c_ulong, c_void};
use std::io::Error;

const RTLD_NEXT: *mut c_void = -1_isize as *mut c_void;
const EPERM: i32 = 1;

#[allow(unsafe_code)]
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
#[allow(unsafe_code, clippy::multiple_unsafe_ops_per_block)]
pub unsafe extern "C" fn personality(persona: c_ulong) -> c_long {
    let symbol_name = c"personality";
    let orig_ptr = unsafe { dlsym(RTLD_NEXT, symbol_name.as_ptr()) };
    if !orig_ptr.is_null() {
        let orig: OrigPersonalityFn = unsafe { std::mem::transmute(orig_ptr) };
        let res = unsafe { orig(persona) };
        if res < 0 && Error::last_os_error().raw_os_error() == Some(EPERM) {
            return 0;
        }
        return res;
    }
    0
}
