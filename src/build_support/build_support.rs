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

//! Build support library for ctoolbox build process.

pub mod asset_packer;
pub mod dc_codegen;
pub mod encoding_codegen;
pub mod extension_codegen;
pub mod fnv;
pub mod format_id_codegen;
pub mod ipc_codegen;
pub mod license_consts;
pub mod seabios_builder;
pub mod standard_boilerplate;
pub mod v86_generator;
pub mod v86_packer;

use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};

/// Checks whether a directory appears to be the root of the ctoolbox repository.
#[must_use]
pub fn is_workspace_root(dir: &Path) -> bool {
    dir.join(".ctoolbox_workspace_root").is_file()
        && (dir.join("Cargo.toml").is_file()
            && dir.join("src").join("formats").is_dir())
}

/// Discovers the root directory of the ctoolbox repository.
///
/// # Errors
/// Returns an error if directory traversal fails or the repository root
/// cannot be found.
pub fn find_repository_root() -> Result<PathBuf> {
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let manifest_path = PathBuf::from(manifest_dir);
        let mut cur = manifest_path.as_path();
        while let Some(parent) = cur.parent() {
            if is_workspace_root(parent) {
                return Ok(parent.to_path_buf());
            }
            cur = parent;
        }
    }

    let mut cur =
        std::env::current_dir().context("Failed to get current dir")?;
    loop {
        if is_workspace_root(&cur) {
            return Ok(cur);
        }
        let Some(parent) = cur.parent() else {
            break;
        };
        cur = parent.to_path_buf();
    }

    bail!("Could not locate repository root containing src/formats/")
}

/// Discovers the root directory of the ctoolbox repository starting from a path.
///
/// # Errors
/// Returns an error if repository root cannot be determined.
pub fn find_repository_root_from(start: &Path) -> Result<PathBuf> {
    let mut cur = Some(start);
    while let Some(dir) = cur {
        if is_workspace_root(dir) {
            return Ok(dir.to_path_buf());
        }
        cur = dir.parent();
    }
    find_repository_root()
}
