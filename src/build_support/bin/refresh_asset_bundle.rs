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

//! CLI tool to refresh and stage static asset bundles for the ctoolbox build process.

use std::env;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use ctb_build_support::asset_packer::{PrepareOptions, prepare_assets};

use ctb_build_support::v86_packer;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if let (Some(cmd), Some(arg1), Some(arg2)) =
        (args.get(1), args.get(2), args.get(3))
    {
        if cmd == "--pack-v86-rsrc" {
            let input_dir = Path::new(arg1);
            let output_rsrc = Path::new(arg2);
            return v86_packer::pack_v86_rsrc(input_dir, output_rsrc);
        }
        if cmd == "--build-v86-initrd" {
            let fs_json = Path::new(arg1);
            let out_initrd = Path::new(arg2);
            return v86_packer::build_custom_initrd(fs_json, out_initrd);
        }
        if let Some(arg3) = args.get(4) {
            if cmd == "--pack-v86-dir" {
                let input_dir = Path::new(arg1);
                let output_dir = Path::new(arg2);
                let output_fs_json = Path::new(arg3);
                v86_packer::pack_rootfs_dir(
                    input_dir,
                    output_dir,
                    output_fs_json,
                    true,
                    &["/boot/"],
                )?;
                let initrd_path = output_dir
                    .parent()
                    .ok_or_else(|| {
                        anyhow::anyhow!("output_dir has no parent directory")
                    })?
                    .join("guix_posix_initrd.cpio.gz");
                return v86_packer::build_custom_initrd(
                    output_fs_json,
                    &initrd_path,
                );
            }
            if cmd == "--pack-v86-tar" {
                let tar_path = Path::new(arg1);
                let output_dir = Path::new(arg2);
                let output_fs_json = Path::new(arg3);
                v86_packer::pack_rootfs_tar(
                    tar_path,
                    output_dir,
                    output_fs_json,
                    true,
                )?;
                let initrd_path = output_dir
                    .parent()
                    .ok_or_else(|| {
                        anyhow::anyhow!("output_dir has no parent directory")
                    })?
                    .join("guix_posix_initrd.cpio.gz");
                return v86_packer::build_custom_initrd(
                    output_fs_json,
                    &initrd_path,
                );
            }
        }
    }

    let workspace_root = workspace_root_from_args()?;
    let options = PrepareOptions {
        prepare_runtime_bundle: true,
        prepare_minimal_assets: false,
        include_rust_docs: false,
        archive_source: false,
        write_debug_stubs: true,
    };

    prepare_assets(&workspace_root, &options).with_context(|| {
        format!(
            "Failed to refresh runtime asset bundle in {}",
            workspace_root.display()
        )
    })?;

    println!(
        "Refreshed runtime asset bundle at {}",
        workspace_root.join("built/ctoolbox.rsrc").display()
    );
    Ok(())
}

fn workspace_root_from_args() -> Result<PathBuf> {
    let mut args = env::args_os();
    let _program = args.next();
    let Some(root) = args.next() else {
        bail!(
            "usage: refresh-asset-bundle <workspace-root> OR --pack-v86-dir/tar <input> <out-flat-dir> <out-fs-json>"
        );
    };
    if args.next().is_some() {
        bail!("usage: refresh-asset-bundle <workspace-root>");
    }

    let root = PathBuf::from(root);
    ensure_workspace_root(&root)?;
    Ok(root)
}

fn ensure_workspace_root(root: &Path) -> Result<()> {
    let manifest = root.join("Cargo.toml");
    if !manifest.is_file() {
        bail!(
            "workspace root must contain Cargo.toml: {}",
            manifest.display()
        );
    }

    Ok(())
}
