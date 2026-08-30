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

//! Main executable entry point for the Collective Toolbox application.

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

/// Helper to check if an error represents an OS Error 2 (ENOENT / Not Found).
fn is_not_found_error(err: &anyhow::Error) -> bool {
    for cause in err.chain() {
        if let Some(io_err) = cause.downcast_ref::<std::io::Error>() {
            if io_err.kind() == std::io::ErrorKind::NotFound
                || io_err.raw_os_error() == Some(2)
            {
                return true;
            }
        }
    }
    let msg = format!("{err:?}");
    msg.contains("os error 2") || msg.contains("No such file or directory")
}

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

    if let Err(err) = ctoolbox::workspace::entry().await {
        if is_not_found_error(&err) {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            if let Err(retry_err) = ctoolbox::workspace::entry().await {
                eprintln!(
                    "Error: A required file or directory was not found during startup (os error 2).\n\
                     Details: {retry_err:?}\n\n\
                     Please retry running the command again by hand."
                );
                std::process::exit(1);
            }
            return Ok(());
        }
        return Err(err);
    }

    Ok(())
}
