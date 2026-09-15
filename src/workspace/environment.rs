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

//! Environment capture IPC endpoints hosted by the workspace supervisor.
//!
//! Subprocesses query these methods via IPC so that only the main workspace
//! process performs external network discovery and maintains the cache.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

pub use ctb_utilities::environment::EnvDescription;

/// Capture a full snapshot of the execution environment on the workspace supervisor.
///
/// # Errors
/// Returns an error if environment capture or IPC serialization fails.
#[ipc_method]
pub async fn capture() -> Result<EnvDescription> {
    Ok(ctb_utilities::environment::capture())
}

/// Capture a quick snapshot of the execution environment on the workspace supervisor,
/// returning cached IP/time information if populated at boot.
///
/// # Errors
/// Returns an error if environment capture or IPC serialization fails.
#[ipc_method]
pub async fn capture_quick() -> Result<EnvDescription> {
    Ok(ctb_utilities::environment::capture_quick())
}

/// Clear cached IP and timestamp offset values on the workspace supervisor and
/// immediately refill them.
///
/// # Errors
/// Returns an error if cache refresh or background task fails.
#[ipc_method]
pub async fn env_cache_reset() -> Result<()> {
    tokio::task::spawn_blocking(ctb_utilities::environment::env_cache_reset)
        .await
        .context("env_cache_reset worker thread panicked")?;
    Ok(())
}
