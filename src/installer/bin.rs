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

#![deny(unused_must_use)]
#![warn(clippy::pedantic)]
#![deny(clippy::as_conversions)]
#![deny(clippy::unnecessary_fallible_conversions)]
#![deny(clippy::try_err)]
#![deny(clippy::ok_expect)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::indexing_slicing)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::unwrap_in_result)]
#![warn(clippy::panic_in_result_fn)]
#![warn(clippy::map_err_ignore)]
#![allow(clippy::collapsible_if, reason = "style preference")]
#![allow(clippy::must_use_candidate, reason = "style preference")]
#![allow(clippy::needless_range_loop, reason = "style preference")]
#![allow(clippy::missing_panics_doc, reason = "style preference")]
#![allow(clippy::missing_errors_doc, reason = "style preference")]

//! Binary entry point for the standalone ctoolbox installer.
//!
//! This launches the graphical installer by default, or falls back to TUI
//! mode when `--no-gui` is passed.

use ctb_utilities::anyhow;

use nix::libc::{
    MCL_CURRENT, MCL_FUTURE, RLIMIT_CORE, mlockall, rlimit, setrlimit, syscall,
};

#[expect(
    unsafe_code,
    reason = "installer process locking syscalls require unsafe"
)]
fn main() -> anyhow::Result<()> {
    // Try to prevent the process from being swapped out (it still might if the computer is suspended or hibernated, if this is running in a VM, or perhaps if the process doesn't have permission to use this syscall).
    // SAFETY: Calling mlockall is standard Unix process lifecycle configuration and safe to execute.
    unsafe {
        syscall(mlockall(MCL_CURRENT | MCL_FUTURE).into());
    }

    // Try to prevent core dumps
    let limit = rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    // SAFETY: setrlimit configuration using standard structs is safe.
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

    ctb_installer::main()
}
