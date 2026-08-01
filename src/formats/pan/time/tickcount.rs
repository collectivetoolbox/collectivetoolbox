/* SPDX-License-Identifier: MIT */
// See full license details in COPYING in the `ctb-formats-pan` crate source directory.

//! Cross-platform tick count helper.

#![allow(
    clippy::module_name_repetitions,
    reason = "idiomatic module structure names"
)]

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

const TICKS_PER_SECOND: u128 = 60;
const NANOS_PER_SECOND: u128 = 1_000_000_000;

/// Returns a monotonic tick count in 60 Hz ticks.
pub fn tickcount() -> Result<i64> {
    let nanos = nanos_since_boot()?;
    let ticks = nanos
        .checked_mul(TICKS_PER_SECOND)
        .context("tickcount overflow")?
        / NANOS_PER_SECOND;

    i64::try_from(ticks).context("tickcount did not fit into i64")
}

#[allow(unsafe_code)]
#[cfg(target_os = "windows")]
fn nanos_since_boot() -> Result<u128> {
    #[link(name = "Kernel32")]
    extern "system" {
        fn GetTickCount64() -> u64;
    }

    let ms = unsafe { GetTickCount64() };
    let ms_u = u128::from(ms);
    ms_u.checked_mul(1_000_000u128).context("nanos overflow")
}

#[allow(unsafe_code)]
#[cfg(all(unix, target_os = "macos"))]
fn nanos_since_boot() -> Result<u128> {
    #[repr(C)]
    struct MachTimebaseInfo {
        numer: u32,
        denom: u32,
    }

    extern "C" {
        fn mach_absolute_time() -> u64;
        fn mach_timebase_info(info: *mut MachTimebaseInfo) -> i32;
    }

    let t = u128::from(unsafe { mach_absolute_time() });

    let mut info = MachTimebaseInfo { numer: 0, denom: 0 };
    let rc = unsafe { mach_timebase_info(&mut info) };
    if rc != 0 {
        bail!("mach_timebase_info failed with code {rc}");
    }

    let numer = u128::from(info.numer);
    let denom = u128::from(info.denom);
    if denom == 0 {
        bail!("mach_timebase_info returned denom=0");
    }

    t.checked_mul(numer).context("nanos overflow")? / denom
}

#[cfg(all(unix, not(target_os = "macos")))]
fn nanos_since_boot() -> Result<u128> {
    #[cfg(any(target_os = "linux", target_os = "android"))]
    use nix::libc::CLOCK_BOOTTIME;
    #[cfg(not(any(target_os = "linux", target_os = "android")))]
    use nix::libc::CLOCK_MONOTONIC;
    use nix::time::clock_gettime;

    #[repr(C)]
    struct Timespec {
        tv_sec: i64,
        tv_nsec: i64,
    }

    #[cfg(any(target_os = "linux", target_os = "android"))]
    const CLOCK_ID: i32 = CLOCK_BOOTTIME;

    #[cfg(not(any(target_os = "linux", target_os = "android")))]
    const CLOCK_ID: i32 = CLOCK_MONOTONIC;

    let _ts = Timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    let ts = clock_gettime(CLOCK_ID.into())?;

    let sec = u128::try_from(ts.tv_sec()).context("tv_sec was negative")?;
    let nsec = u128::try_from(ts.tv_nsec()).context("tv_nsec was negative")?;
    sec.checked_mul(NANOS_PER_SECOND)
        .context("nanos overflow")?
        .checked_add(nsec)
        .context("nanos overflow")
}

#[cfg(test)]
#[expect(
    clippy::panic,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::unwrap_in_result,
    clippy::panic_in_result_fn,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    reason = "Standard repository test boilerplate"
)]
mod tests {
    use ctb_utilities::anyhow::ensure;

    use super::*;

    #[crate::ctb_test]
    fn tickcount_is_monotonic() -> Result<()> {
        let a = tickcount()?;
        let b = tickcount()?;
        ensure!(a >= 0, "got {a}");
        ensure!(b >= a, "tickcount went backwards: {a} -> {b}");
        Ok(())
    }
}
