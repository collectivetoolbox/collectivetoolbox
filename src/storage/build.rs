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

//! Build script for ctb-storage that ensures assets are prepared.

use anyhow::Result;
use ctb_build_support::asset_packer::{
    PrepareOptions, prepare_assets, print_rerun_directives,
};
use std::env;
use std::path::Path;

fn main() -> Result<()> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
    let manifest_path = Path::new(&manifest_dir);
    // Reason for fallback: builds executed outside cargo environment default profile string to "debug"
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_owned());
    let is_release = profile == "release";
    let skip_docs = env::var_os("CTB_SKIP_DOCS").is_some();
    let include_rust_docs = is_release && !skip_docs;

    let build_rs_debug = env::var_os("CTB_BUILD_RS_DEBUG").is_some();
    if build_rs_debug {
        println!(
            "cargo:warning=ctb-storage/build.rs: CARGO_MANIFEST_DIR={}",
            manifest_path.display()
        );
    }

    // Navigate up to the ctoolbox crate directory
    let ctoolbox_dir = manifest_path
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| anyhow::anyhow!("Could not find ctoolbox directory"))?;

    // Print rerun-if-changed directives for asset paths
    print_rerun_directives(ctoolbox_dir)?;

    // Prepare assets (Cargo controls re-running build scripts via the
    // `rerun-if-changed` directives above).
    println!("cargo:warning=ctb-storage: Preparing build output directory...");
    let options = PrepareOptions {
        prepare_runtime_bundle: true,
        prepare_minimal_assets: true,
        include_rust_docs,
        archive_source: false,
        write_debug_stubs: !include_rust_docs,
    };
    if build_rs_debug {
        println!(
            "cargo:warning=ctb-storage/build.rs: PrepareOptions={options:?}"
        );
        let target_doc_dir =
            ctoolbox_dir.join("target/x86_64-unknown-linux-musl/doc");
        let built_doc_dir = ctoolbox_dir.join("built/docs");
        println!(
            "cargo:warning=ctb-storage/build.rs: target-docs exists={} built-docs exists={}",
            target_doc_dir.is_dir(),
            built_doc_dir.is_dir()
        );
    }
    let prepared_assets = prepare_assets(ctoolbox_dir, &options)?;
    if let Some(asset_pack_uuid) = prepared_assets.asset_pack_uuid {
        println!("cargo:rustc-env=CTB_ASSET_PACK_UUID={asset_pack_uuid}");
    }
    if let Some(asset_pack_sha256) = prepared_assets.asset_pack_sha256 {
        println!("cargo:rustc-env=CTB_ASSET_PACK_SHA256={asset_pack_sha256}");
    }
    if let Some(v86_uuid) = prepared_assets.v86_asset_pack_uuid {
        println!("cargo:rustc-env=CTB_V86_ASSET_PACK_UUID={v86_uuid}");
    }
    if let Some(v86_sha) = prepared_assets.v86_asset_pack_sha256 {
        println!("cargo:rustc-env=CTB_V86_ASSET_PACK_SHA256={v86_sha}");
    }

    Ok(())
}
