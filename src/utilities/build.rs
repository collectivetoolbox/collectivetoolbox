// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT
// SPDX-License-Identifier for parts derived from workspace-filter: MIT
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

// Parts of this code are adapted from the workspace-filter crate:

// Copyright 2025 Ossian Mapes

// See additional licensing details at end of file.

//! Build script emitting version metadata and git build information.

use anyhow::{Result, bail};
use cargo_metadata::MetadataCommand;
use vergen_gix::{
    BuildBuilder, CargoBuilder, Emitter, GixBuilder, RustcBuilder,
    SysinfoBuilder,
};

fn main() -> Result<()> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let manifest_dir = std::path::PathBuf::from(manifest_dir);
    ctb_build_support::ipc_codegen::generate_workspace_ipc_methods(
        &manifest_dir,
    )?;
    ctb_build_support::ipc_codegen::generate_ipc_service_boilerplate(
        &manifest_dir,
    )?;

    let filter = workspace_filter_build::build();
    if filter.is_err() {
        let Some(err) = filter.err() else {
            bail!("Failed to build workspace filter: unknown error");
        };
        bail!("Failed to build workspace filter: {err}");
    }

    let metadata = MetadataCommand::new().exec()?;
    // get the main ctoolbox package version
    let ctb_version = metadata
        .workspace_packages()
        .into_iter()
        .find(|p| p.name == "ctoolbox");
    let ctb_version = match ctb_version {
        Some(pkg) => pkg.version.to_string(),
        None => "ERROR GETTING CTOOLBOX VERSION".to_string(),
    };

    println!("cargo:rustc-env=CTB_VERSION={ctb_version}");

    // NOTE: This will output everything, and requires all features enabled.
    // NOTE: See the specific builder documentation for configuration options.
    let build = BuildBuilder::all_build()?;
    let cargo = CargoBuilder::all_cargo()?;
    let gix = GixBuilder::all_git()?;
    let rustc = RustcBuilder::all_rustc()?;
    let si = SysinfoBuilder::all_sysinfo()?;

    Emitter::default()
        .add_instructions(&build)?
        .add_instructions(&cargo)?
        .add_instructions(&gix)?
        .add_instructions(&rustc)?
        .add_instructions(&si)?
        .emit()?;

    Ok(())
}

/*

// From workspace-filter:

Copyright 2025 Ossian Mapes

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/
