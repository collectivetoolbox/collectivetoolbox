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

//! Build script for ctoolbox workspace assets and version metadata.

use anyhow::Result;
use ctb_build_support::asset_packer::{
    PrepareOptions, prepare_assets, print_rerun_directives,
};
use std::env;
use std::path::Path;
use vergen_gix::{
    BuildBuilder, CargoBuilder, Emitter, GixBuilder, RustcBuilder,
    SysinfoBuilder,
};

/// Build script to prepare assets and metadata for the project.
fn main() -> Result<()> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
    let manifest_path = Path::new(&manifest_dir);
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_owned());
    let is_release = profile == "release";
    let skip_docs = env::var_os("CTB_SKIP_DOCS").is_some();
    let include_rust_docs = is_release && !skip_docs;

    let build_rs_debug = env::var_os("CTB_BUILD_RS_DEBUG").is_some();
    if build_rs_debug {
        println!(
            "cargo:warning=ctoolbox/build.rs: CARGO_MANIFEST_DIR={}",
            manifest_path.display()
        );
    }

    // Print rerun-if-changed directives for asset paths
    print_rerun_directives(manifest_path)?;
    println!("cargo:rerun-if-changed=./Cargo.lock");

    // Prepare assets (Cargo controls re-running build scripts via the
    // `rerun-if-changed` directives above).
    println!("cargo:warning=Preparing build output directory...");
    let options = PrepareOptions {
        prepare_runtime_bundle: true,
        prepare_minimal_assets: true,
        include_rust_docs,
        archive_source: false,
        write_debug_stubs: !include_rust_docs,
    };
    if build_rs_debug {
        println!("cargo:warning=ctoolbox/build.rs: PrepareOptions={options:?}");
        let target_doc_dir =
            manifest_path.join("target/x86_64-unknown-linux-musl/doc");
        let built_doc_dir = manifest_path.join("built/docs");
        println!(
            "cargo:warning=ctoolbox/build.rs: target-docs exists={} built-docs exists={}",
            target_doc_dir.is_dir(),
            built_doc_dir.is_dir()
        );
    }
    let prepared_assets = prepare_assets(manifest_path, &options)?;
    if let Some(asset_pack_uuid) = prepared_assets.asset_pack_uuid {
        println!("cargo:rustc-env=CTB_ASSET_PACK_UUID={asset_pack_uuid}");
    }

    if std::env::var("CARGO_CFG_TARGET_OS").expect("Could not get target OS")
        == "windows"
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("../assets/web/favicon.ico");
        res.compile()?;
    }

    // Emit vergen metadata
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
