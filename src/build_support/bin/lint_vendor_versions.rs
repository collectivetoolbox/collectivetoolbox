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

//! Lint tool to check if vendored packages in `vendor/` (excluding
//! `vendor/ctb-vendored/` and `vendor/upstream-for-reference/`) have newer
//! versions available on crates.io than the versions the patched packages are
//! based on.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use semver::Version;
use serde::Deserialize;
use toml::{Table, Value};

#[derive(Deserialize)]
struct CrateResponse {
    #[serde(rename = "crate")]
    crate_info: CrateInfo,
}

#[derive(Deserialize)]
struct CrateInfo {
    max_version: Option<String>,
    max_stable_version: Option<String>,
}

struct OutdatedPackage {
    crate_name: String,
    local_version: String,
    crates_io_version: String,
}

fn main() -> Result<()> {
    let (workspace_root, offline) = parse_args()?;

    if offline {
        println!(
            "================================================================================"
        );
        println!(
            "==================== VENDORED PACKAGE VERSION CHECK SKIPPED ===================="
        );
        println!(
            "================================================================================"
        );
        println!(
            "Skipping crates.io version check for vendored packages (--offline specified)."
        );
        println!(
            "================================================================================"
        );
        println!(
            "================================================================================"
        );
        return Ok(());
    }

    let vendor_dir = workspace_root.join("vendor");
    if !vendor_dir.is_dir() {
        return Ok(());
    }

    let vendored_crates = find_vendored_crates(&vendor_dir)?;
    let mut outdated = Vec::new();

    for (crate_name, local_ver_str) in vendored_crates {
        let Some(crates_io_ver_str) = fetch_crates_io_version(&crate_name)
        else {
            continue;
        };

        let Ok(local_semver) = Version::parse(&local_ver_str) else {
            continue;
        };
        let Ok(crates_io_semver) = Version::parse(&crates_io_ver_str) else {
            continue;
        };

        if crates_io_semver > local_semver {
            outdated.push(OutdatedPackage {
                crate_name,
                local_version: local_ver_str,
                crates_io_version: crates_io_ver_str,
            });
        }
    }

    if outdated.is_empty() {
        println!(
            "vendored package version lint passed (all vendored packages up to date)"
        );
        return Ok(());
    }

    println!(
        "================================================================================"
    );
    println!(
        "==================== VENDORED PACKAGE VERSION CHECK WARNING ===================="
    );
    println!(
        "================================================================================"
    );
    println!(
        "WARNING: The following vendored packages in vendor/ have newer versions available on crates.io:\n"
    );
    for pkg in &outdated {
        println!(
            "  - {}: local version {} (crates.io latest stable: {})",
            pkg.crate_name, pkg.local_version, pkg.crates_io_version
        );
    }
    println!();
    println!("Please consider updating these patched dependencies when appropriate.");
    println!(
        "================================================================================"
    );
    println!(
        "================================================================================"
    );

    Ok(())
}

fn parse_args() -> Result<(PathBuf, bool)> {
    let args = env::args_os().skip(1);
    let mut offline = false;
    let mut root_path = None;

    for arg in args {
        let arg_str = arg.to_string_lossy();
        if arg_str == "--offline" {
            offline = true;
        } else if root_path.is_none() {
            root_path = Some(PathBuf::from(arg));
        } else {
            bail!("usage: lint-vendor-versions [--offline] <workspace-root>");
        }
    }

    let Some(root) = root_path else {
        bail!("usage: lint-vendor-versions [--offline] <workspace-root>");
    };

    Ok((root, offline))
}

/// Discovers vendored crates directly inside `vendor/`, excluding
/// `ctb-vendored` and `upstream-for-reference`.
fn find_vendored_crates(vendor_dir: &Path) -> Result<Vec<(String, String)>> {
    let mut results = Vec::new();
    let entries = fs::read_dir(vendor_dir)
        .with_context(|| format!("failed to read directory {}", vendor_dir.display()))?;

    for entry in entries {
        let entry = entry?;
        let file_name = entry.file_name();
        let name_str = file_name.to_string_lossy();

        if name_str == "ctb-vendored" || name_str == "upstream-for-reference" {
            continue;
        }

        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let cargo_toml = path.join("Cargo.toml");
        if !cargo_toml.is_file() {
            continue;
        }

        if let Ok(manifest) = parse_manifest(&cargo_toml) {
            if let Some(pkg_table) = manifest.get("package").and_then(Value::as_table) {
                let name = pkg_table
                    .get("name")
                    .and_then(Value::as_str)
                    .map(ToString::to_string);
                let version = pkg_table
                    .get("version")
                    .and_then(Value::as_str)
                    .map(ToString::to_string);

                if let (Some(name), Some(version)) = (name, version) {
                    results.push((name, version));
                }
            }
        }
    }

    results.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(results)
}

fn parse_manifest(path: &Path) -> Result<Table> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read manifest {}", path.display()))?;
    text.parse::<Table>()
        .with_context(|| format!("failed to parse TOML in {}", path.display()))
}

/// Fetches the latest stable version for `crate_name` from crates.io API.
fn fetch_crates_io_version(crate_name: &str) -> Option<String> {
    let url = format!("https://crates.io/api/v1/crates/{crate_name}");
    let output = Command::new("curl")
        .args([
            "-s",
            "-f",
            "-H",
            "User-Agent: ctoolbox-lint (vendor-version-check)",
            &url,
        ])
        .output();

    let Ok(output) = output else {
        return None;
    };

    if !output.status.success() {
        return None;
    }

    let Ok(resp) = serde_json::from_slice::<CrateResponse>(&output.stdout) else {
        return None;
    };

    resp.crate_info
        .max_stable_version
        .or(resp.crate_info.max_version)
}
