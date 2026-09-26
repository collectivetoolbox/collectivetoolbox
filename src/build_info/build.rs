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

//! Build script emitting version metadata and git build information.

use anyhow::Result;
use cargo_metadata::MetadataCommand;
use vergen_gix::{BuildBuilder, CargoBuilder, Emitter, GixBuilder, RustcBuilder};

fn main() -> Result<()> {
    let metadata = MetadataCommand::new().exec()?;
    // Get the main ctoolbox package version.
    let ctb_version = metadata
        .workspace_packages()
        .into_iter()
        .find(|p| p.name == "ctoolbox");
    let ctb_version = match ctb_version {
        Some(pkg) => pkg.version.to_string(),
        None => "ERROR GETTING CTOOLBOX VERSION".to_string(),
    };

    println!("cargo:rustc-env=CTB_VERSION={ctb_version}");

    let build_id = std::env::var("CTB_BUILD_ID")
        .ok()
        .or_else(|| {
            std::process::Command::new("git")
                .args(["rev-parse", "HEAD"])
                .output()
                .ok()
                .and_then(|output| {
                    if output.status.success() {
                        String::from_utf8(output.stdout)
                            .ok()
                            .map(|s| s.trim().to_string())
                    } else {
                        None
                    }
                })
        })
        // Reason for fallback: builds outside a git worktree or without CTB_BUILD_ID
        // default to "dev" as the build identifier.
        .unwrap_or_else(|| "dev".to_string());

    println!("cargo:rustc-env=CTB_BUILD_ID={build_id}");

    let build = BuildBuilder::all_build()?;
    let cargo = CargoBuilder::all_cargo()?;
    let gix = GixBuilder::all_git()?;
    let rustc = RustcBuilder::all_rustc()?;

    Emitter::default()
        .add_instructions(&build)?
        .add_instructions(&cargo)?
        .add_instructions(&gix)?
        .add_instructions(&rustc)?
        .emit()?;

    Ok(())
}
