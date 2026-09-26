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

//! Host system management and hardware/platform control APIs.
//!
//! Provides capabilities for managing the host system, such as restarting or
//! shutting down the computer, configuring network and Wi-Fi connectivity, and
//! interacting with system services when booting directly into CTB.

#[allow(
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

/// Check whether the current platform and permissions allow restarting the
/// host computer.
///
/// Returns `false` by default on web, desktop, and unprivileged platforms.
/// Systems booted directly into CTB with administrative control may return
/// `true` once system shutdown/reboot controls are enabled.
#[must_use]
pub fn can_restart_pc() -> bool {
    false
}

/// Request a restart of the host computer if supported.
///
/// Returns an error if restarting the computer is not supported or not
/// permitted in the current environment.
pub fn restart_pc() -> Result<()> {
    bail!("restarting the host computer is not supported in this environment");
}

#[cfg(test)]
#[allow(
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
    use super::*;

    #[crate::ctb_test]
    fn test_can_restart_pc_default() {
        assert!(!can_restart_pc());
    }

    #[crate::ctb_test]
    fn test_restart_pc_unsupported() {
        let result = restart_pc();
        assert!(result.is_err());
    }
}
