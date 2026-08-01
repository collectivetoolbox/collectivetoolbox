#![deny(unused_must_use)]
#![warn(clippy::pedantic)]
#![deny(clippy::as_conversions)]
#![deny(clippy::unnecessary_fallible_conversions)]
#![deny(clippy::try_err)]
#![deny(clippy::ok_expect)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![cfg_attr(test, warn(clippy::indexing_slicing))]
#![cfg_attr(not(test), deny(clippy::indexing_slicing))]
#![warn(clippy::unwrap_used)]
#![warn(clippy::unwrap_in_result)]
#![warn(clippy::panic_in_result_fn)]
#![warn(clippy::map_err_ignore)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]

pub(crate) use ctb_utilities::Result;
use ctb_utilities::anyhow;

use nix::libc::{
    MCL_CURRENT, MCL_FUTURE, RLIMIT_CORE, mlockall, rlimit, setrlimit, syscall,
};

#[tokio::main]
pub async fn main() -> Result<()> {
    // Try to prevent the process from being swapped out (it still might if the computer is suspended or hibernated, if this is running in a VM, or perhaps if the process doesn't have permission to use this syscall).
    unsafe {
        syscall(mlockall(MCL_CURRENT | MCL_FUTURE).into());
    }

    // Try to prevent core dumps
    let limit = rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    unsafe {
        setrlimit(RLIMIT_CORE, &raw const limit);
    }

    let result =
        rustls::crypto::aws_lc_rs::default_provider().install_default();
    anyhow::ensure!(
        result.is_ok(),
        "Failed to initialize rustls aws-lc-rs provider: {:?}",
        result.err()
    );

    // Ensure XKB data is available before any winit/X11 initialization.
    ctb_storage_minimal::xkb::ensure_xkb_config_root()?;
    ctb_storage_minimal::xkb::ensure_x11_locale_root()?;

    ctoolbox::workspace::entry().await
}
