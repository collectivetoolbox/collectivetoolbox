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

//! Build script for embedding license text and minimal asset dependencies.

use std::env;
use std::path::PathBuf;

#[expect(
    clippy::expect_used,
    clippy::panic,
    clippy::panic_used,
    reason = "It is a build script, so panicking seems like an OK way to handle errors."
)]
fn main() {
    let manifest = PathBuf::from(
        env::var("CARGO_MANIFEST_DIR")
            .expect("CARGO_MANIFEST_DIR environment variable must be set by Cargo"),
    );
    let workspace = manifest
        .join("../../../")
        .canonicalize()
        .expect("Failed to resolve workspace root from manifest dir");

    assert!(
        !(!workspace.join("assets").is_dir()
            || !workspace.join("vendor").is_dir()),
        "Resolved workspace root does not look like the ctoolbox root: {}",
        workspace.display()
    );

    // Use the shared build-support helper to ensure minimal assets exist.
    if let Err(err) =
        ctb_build_support::asset_packer::ensure_minimal_assets_for_build_rs(
            &workspace,
        )
    {
        eprintln!("Failed to prepare minimal assets: {err}");
        std::process::exit(1);
    }
}
