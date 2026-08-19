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

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use include_dir::{Dir, include_dir};

pub mod deno_config;
pub mod diagnostics;
pub mod js_lint;
pub mod js_test;
pub mod jsdoc;
pub mod lint;
pub mod project_files_resolver;
pub mod string;
pub mod ts_check;
pub mod tsconfig;
pub mod typescript;

static JS_DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data");

pub fn get_js_data(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&JS_DATA_DIR, key)
}

#[cfg(test)]
#[expect(
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
    #[crate::ctb_test]
    fn test_get_bootstrapped_compiler() {
        // let repo_path = super::typescript::get_default_ts_repo_path();
        // if let Some(parent) = repo_path.parent() {
        //     let cached_tar = parent.join("TypeScript-built.tar");
        //     if cached_tar.exists() {
        //         println!("Skipping test_bootstrap_typescript because vendor/TypeScript-built.tar exists.");
        //         return;
        //     }
        // }
        // assert!(repo_path.exists(), "TypeScript repo path does not exist: {}", repo_path.display());
        // let tarball = super::typescript::get_bootstrapped_compiler(&repo_path).unwrap();
        let tarball = super::typescript::get_bootstrapped_compiler().unwrap();
        assert!(!tarball.is_empty(), "Bootstrapped tarball is empty!");

        let mut archive = tar::Archive::new(&tarball[..]);
        let entries = archive.entries().unwrap();
        let mut found_tsc = false;
        let mut found_ts = false;
        let mut found_lib = false;
        for entry in entries {
            let entry: tar::Entry<'_, &[u8]> = entry.unwrap();
            let path = entry
                .path()
                .unwrap()
                .to_string_lossy()
                .into_owned()
                .replace('\\', "/");
            if path == "tsc/tsc.js" {
                found_tsc = true;
            }
            if path == "typescript/typescript.js" {
                found_ts = true;
            }
            if path == "tsc/lib.es5.d.ts" {
                found_lib = true;
            }
        }
        assert!(found_tsc, "tsc/tsc.js not found in tarball");
        assert!(found_ts, "typescript/typescript.js not found in tarball");
        assert!(found_lib, "tsc/lib.es5.d.ts not found in tarball");
    }
}
