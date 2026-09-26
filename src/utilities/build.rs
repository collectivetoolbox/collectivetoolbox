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

//! Build script for ctb-utilities code generation and version metadata.

use anyhow::{Result, bail};
use cargo_metadata::MetadataCommand;

fn main() -> Result<()> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let manifest_dir = std::path::PathBuf::from(manifest_dir);
    ctb_build_support::ipc_codegen::generate_workspace_ipc_methods(
        &manifest_dir,
    )?;
    ctb_build_support::ipc_codegen::generate_ipc_service_boilerplate(
        &manifest_dir,
    )?;

    println!("cargo:rerun-if-changed=../formats/dcdata/data/categories");
    ctb_build_support::format_id_codegen::generate_format_id_file(
        &manifest_dir,
    )?;
    ctb_build_support::dc_codegen::generate_dc_file(
        &manifest_dir,
    )?;
    ctb_build_support::extension_codegen::generate_extension_data_file(
        &manifest_dir,
    )?;
    ctb_build_support::encoding_codegen::generate_encoding_file(
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
        .unwrap_or_else(|| "dev".to_string());

    println!("cargo:rustc-env=CTB_BUILD_ID={build_id}");

    Ok(())
}

/*

// From workspace-filter:

Copyright 2025 Ossian Mapes

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/
