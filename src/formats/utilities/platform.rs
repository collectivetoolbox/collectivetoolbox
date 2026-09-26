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

//! Platform operating system compatibility helpers and host platform inference.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::format_id::FormatId;

/// Determines whether a format's associated operating system is compatible
/// with a target operating system context.
///
/// Uses format implication hierarchies declared via `@implies(...)` directives
/// in the format database. A candidate OS matches a target OS if they are identical
/// or if either directly or transitively implies the other.
#[must_use]
pub fn is_os_match(candidate_os: FormatId, target_os: FormatId) -> bool {
    candidate_os == target_os
        || candidate_os.implies().contains(&target_os)
        || target_os.implies().contains(&candidate_os)
}

/// Infers the host platform operating system as an authoritative `FormatId`.
#[must_use]
pub fn current_platform_os() -> Option<FormatId> {
    #[cfg(target_os = "macos")]
    {
        Some(FormatId::MacOs)
    }
    #[cfg(target_os = "windows")]
    {
        Some(FormatId::Windows)
    }
    #[cfg(target_os = "linux")]
    {
        Some(FormatId::GnuLinux)
    }
    #[cfg(all(unix, not(any(target_os = "macos", target_os = "linux"))))]
    {
        Some(FormatId::Unix)
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", unix)))]
    {
        None
    }
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
    fn test_os_match_matrix() {
        assert!(is_os_match(FormatId::Linux, FormatId::Unix));
        assert!(is_os_match(FormatId::Unix, FormatId::Linux));
        assert!(is_os_match(FormatId::MacOs, FormatId::MacOsDarwin));
        assert!(is_os_match(FormatId::MacOsDarwin, FormatId::MacOs));
        assert!(is_os_match(FormatId::WinClassic, FormatId::Windows));
        assert!(is_os_match(FormatId::Windows, FormatId::WinClassic));
        assert!(is_os_match(FormatId::WinNt, FormatId::Windows));
        assert!(is_os_match(FormatId::Windows, FormatId::WinNt));
        assert!(is_os_match(FormatId::Linux, FormatId::GnuLinux));
        assert!(is_os_match(FormatId::GnuLinux, FormatId::Linux));
        assert!(is_os_match(FormatId::GnuLinux, FormatId::Unix));
        assert!(is_os_match(FormatId::Unix, FormatId::GnuLinux));
        assert!(is_os_match(FormatId::FreeBsd, FormatId::Unix));
        assert!(is_os_match(FormatId::Unix, FormatId::FreeBsd));
        assert!(is_os_match(FormatId::Debian, FormatId::Linux));
        assert!(is_os_match(FormatId::Debian, FormatId::Unix));
        assert!(!is_os_match(FormatId::Windows, FormatId::Linux));
        assert!(!is_os_match(FormatId::FreeBsd, FormatId::Linux));
        assert!(!is_os_match(FormatId::Linux, FormatId::FreeBsd));
        assert!(!is_os_match(FormatId::Windows, FormatId::Unix));
        assert!(!is_os_match(FormatId::MacOs, FormatId::Linux));
    }

    #[crate::ctb_test]
    fn test_current_platform_os_is_consistent() {
        let os = current_platform_os();
        #[cfg(target_os = "linux")]
        assert_eq!(os, Some(FormatId::GnuLinux));
        #[cfg(target_os = "macos")]
        assert_eq!(os, Some(FormatId::MacOs));
        #[cfg(target_os = "windows")]
        assert_eq!(os, Some(FormatId::Windows));
    }
}
