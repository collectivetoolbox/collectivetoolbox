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

//! Legacy process-management utilities.
//!
//! The old IPC stack tracked subprocesses and per-process HTTP channels in this
//! crate. The new IPC stack owns process lifecycle inside
//! `ctb_workspace_ipc::process_manager` and `ctb_workspace_ipc::workspace_runner`.
//!
//! This module remains as a small compatibility layer for any remaining callers
//! during migration, but it intentionally avoids re-introducing the legacy IPC
//! channel abstractions.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace prelude"
)]
use crate::utilities::*;

/// Best-effort emergency exit for unrecoverable situations.
///
/// The IPC runner will attempt to clean up subprocesses via OS-level process
/// groups / parent-death semantics. This function is a last resort.
pub fn emergency_exit(exit_code: i32) -> ! {
    std::process::exit(exit_code)
}
