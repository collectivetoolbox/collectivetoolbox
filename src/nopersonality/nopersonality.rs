//! `LD_PRELOAD` shared object intercepting `personality()` syscalls.
//!
//! Ignores `EPERM` failure when setting personality in unprivileged
//! build containers during Guix image builds.

use std::ffi::{c_char, c_long, c_ulong, c_void};
use std::io::Error;

const RTLD_NEXT: *mut c_void = std::ptr::null_mut::<c_void>().wrapping_offset(-1);
const EPERM: i32 = 1;

#[expect(unsafe_code, reason = "LD_PRELOAD shared library interfacing with dlsym C FFI")]
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
#[expect(unsafe_code, reason = "LD_PRELOAD shared library export overriding personality syscall")]
pub unsafe extern "C" fn personality(persona: c_ulong) -> c_long {
    let symbol_name = c"personality";
    // SAFETY: Resolving personality symbol from RTLD_NEXT via dlsym.
    let orig_ptr = unsafe { dlsym(RTLD_NEXT, symbol_name.as_ptr()) };
    if !orig_ptr.is_null() {
        // SAFETY: dlsym returned a non-null pointer to the C personality function matching OrigPersonalityFn signature.
        let orig: OrigPersonalityFn = unsafe { std::mem::transmute(orig_ptr) };
        // SAFETY: Invoking the resolved original personality function with the given persona.
        let res = unsafe { orig(persona) };
        if res < 0 && Error::last_os_error().raw_os_error() == Some(EPERM) {
            return 0;
        }
        return res;
    }
    0
}

