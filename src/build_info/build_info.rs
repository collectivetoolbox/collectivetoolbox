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

//! Build and version metadata for Collective Toolbox.

#[allow(
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct BuildInfo {
    pub name: String,
    pub version: String,
    pub build_id: String,
    pub build_date: String,
    pub commit: String,
}

impl BuildInfo {
    /// Returns the build date timestamp string.
    ///
    /// Note: This timestamp reflects the git commit time, not the true build
    /// time, in order to preserve build caching across incremental builds.
    #[must_use]
    pub fn build_date(&self) -> &str {
        &self.build_date
    }

    /// Returns the build ID string.
    #[must_use]
    pub fn build_id(&self) -> &str {
        &self.build_id
    }
}

/// Returns the build and version metadata for Collective Toolbox.
///
/// Note: The `build_date` field contains the commit timestamp rather than the
/// true build time to ensure reproducible builds and preserve build caching.
#[must_use]
pub fn build_info() -> BuildInfo {
    // Reason for fallback: builds without git repository context (e.g. from
    // a source tarball) lack VERGEN_GIT_COMMIT_TIMESTAMP and fallback to the
    // build timestamp.
    let build_date = option_env!("VERGEN_GIT_COMMIT_TIMESTAMP")
        .unwrap_or(env!("VERGEN_BUILD_TIMESTAMP"))
        .to_string();

    let commit = env!("VERGEN_GIT_SHA").to_string();
    // Reason for fallback: when CTB_BUILD_ID is unset at compile time, fall back
    // to the git commit SHA as the default build identifier.
    let build_id = option_env!("CTB_BUILD_ID")
        .map_or_else(|| commit.clone(), ToString::to_string);

    BuildInfo {
        name: "ctoolbox".to_string(),
        version: env!("CTB_VERSION").to_string(),
        build_id,
        build_date,
        commit,
    }
}

/// Returns a user-friendly version string that includes the build ID for CLI and display.
#[must_use]
pub fn ctb_version_display() -> &'static str {
    const VERSION: &str = env!("CTB_VERSION");
    const BUILD_ID: &str = match option_env!("CTB_BUILD_ID") {
        Some(id) => id,
        None => "dev",
    };
    const VERSION_DISPLAY: &str = constcat::concat!(VERSION, " (build ", BUILD_ID, ")");
    VERSION_DISPLAY
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
    fn test_build_info() {
        let info = build_info();
        assert_eq!(info.name, "ctoolbox");
        assert!(!info.version.is_empty());
        assert!(!info.build_id.is_empty());
        assert!(!info.build_date.is_empty());
        assert!(!info.commit.is_empty());
        assert_eq!(info.build_date(), &info.build_date);
        assert_eq!(info.build_id(), &info.build_id);
    }
}
