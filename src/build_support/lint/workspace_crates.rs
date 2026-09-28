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

//! Linter rule checking that all workspace crates in `src/` are registered in
//! the root `Cargo.toml` manifest so they are never missed during workspace
//! checks or lints.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use toml::{Table, Value};

/// A violation representing a crate in `src/` that is not registered in the
/// workspace root `Cargo.toml`.
#[derive(Debug, PartialEq, Eq)]
pub struct WorkspaceCrateViolation {
    pub relative_path: PathBuf,
    pub package_name: String,
}

/// Normalizes a path to a clean relative form without leading `./` or trailing
/// slashes.
fn normalize_rel_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for comp in path.components() {
        match comp {
            std::path::Component::CurDir => {}
            std::path::Component::Normal(c) => normalized.push(c),
            _ => normalized.push(comp.as_os_str()),
        }
    }
    normalized
}

/// Extracts all crate path dependencies declared in a dependencies table.
fn collect_paths_from_deps_table(
    table: &Table,
    declared_paths: &mut BTreeSet<PathBuf>,
) {
    for (_dep_name, dep_val) in table {
        if let Some(dep_table) = dep_val.as_table()
            && let Some(p) = dep_table.get("path").and_then(Value::as_str)
        {
            declared_paths.insert(normalize_rel_path(Path::new(p)));
        }
    }
}

/// Extracts all path dependencies and workspace members declared in root
/// `Cargo.toml`.
pub fn collect_declared_crate_paths(root_table: &Table) -> BTreeSet<PathBuf> {
    let mut declared_paths = BTreeSet::new();

    // Standard dependencies tables
    for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some(deps_table) = root_table.get(section).and_then(Value::as_table) {
            collect_paths_from_deps_table(deps_table, &mut declared_paths);
        }
    }

    // Target-specific dependencies tables
    if let Some(target_table) = root_table.get("target").and_then(Value::as_table) {
        for (_target_cfg, target_val) in target_table {
            if let Some(target_cfg_table) = target_val.as_table() {
                for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
                    if let Some(deps_table) =
                        target_cfg_table.get(section).and_then(Value::as_table)
                    {
                        collect_paths_from_deps_table(deps_table, &mut declared_paths);
                    }
                }
            }
        }
    }

    // Explicit workspace members (if any are defined)
    if let Some(ws_table) = root_table.get("workspace").and_then(Value::as_table)
        && let Some(members) = ws_table.get("members").and_then(Value::as_array)
    {
        for m in members {
            if let Some(member_str) = m.as_str() {
                declared_paths.insert(normalize_rel_path(Path::new(member_str)));
            }
        }
    }

    declared_paths
}

/// Extracts exclusion paths declared under `[workspace.exclude]`.
pub fn collect_excluded_paths(root_table: &Table) -> BTreeSet<PathBuf> {
    let mut excluded_paths = BTreeSet::new();
    if let Some(ws_table) = root_table.get("workspace").and_then(Value::as_table)
        && let Some(excludes) = ws_table.get("exclude").and_then(Value::as_array)
    {
        for e in excludes {
            if let Some(exclude_str) = e.as_str() {
                excluded_paths.insert(normalize_rel_path(Path::new(exclude_str)));
            }
        }
    }
    excluded_paths
}

/// Extracts the package name from a crate's `Cargo.toml`.
fn extract_package_name(manifest_path: &Path) -> Result<String> {
    let text = fs::read_to_string(manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;
    let table: Table = text
        .parse()
        .with_context(|| format!("failed to parse {}", manifest_path.display()))?;
    let Some(package_table) = table.get("package").and_then(Value::as_table) else {
        return Ok(String::from("<unknown>"));
    };
    let Some(name_val) = package_table.get("name").and_then(Value::as_str) else {
        return Ok(String::from("<unknown>"));
    };
    Ok(name_val.to_string())
}

/// Recursively traverses a directory, discovering crates and verifying
/// their presence in `declared_paths`.
pub fn scan_src_crates(
    current_dir: &Path,
    workspace_root: &Path,
    declared_paths: &BTreeSet<PathBuf>,
    excluded_paths: &BTreeSet<PathBuf>,
    violations: &mut Vec<WorkspaceCrateViolation>,
) -> Result<()> {
    if !current_dir.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(current_dir)
        .with_context(|| format!("failed to read dir {}", current_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        if path.is_dir() {
            // Skip hidden directories and template directories starting with `_`
            if name.starts_with('.') || name.starts_with('_') {
                continue;
            }

            // Skip standard non-source / vendor directories
            if name == "target"
                || name == "vendor"
                || name == "built"
                || name == "old"
                || name == "node_modules"
            {
                continue;
            }

            let rel_path = path
                .strip_prefix(workspace_root)
                .map(normalize_rel_path)
                .unwrap_or_else(|_| normalize_rel_path(&path));

            // Skip paths matching workspace.exclude
            if excluded_paths.contains(&rel_path) {
                continue;
            }

            // If this directory contains a Cargo.toml, verify it
            let manifest_path = path.join("Cargo.toml");
            if manifest_path.is_file() {
                if !declared_paths.contains(&rel_path) {
                    let package_name = extract_package_name(&manifest_path)
                        .unwrap_or_else(|_| String::from("<unknown>"));
                    violations.push(WorkspaceCrateViolation {
                        relative_path: rel_path,
                        package_name,
                    });
                }
            }

            // Continue recursion to discover any nested subcrates
            scan_src_crates(
                &path,
                workspace_root,
                declared_paths,
                excluded_paths,
                violations,
            )?;
        }
    }

    Ok(())
}

/// Validates that all crates under `src/` are declared in the root `Cargo.toml`.
pub fn check_workspace_crates(
    workspace_root: &Path,
) -> Result<Vec<WorkspaceCrateViolation>> {
    let root_manifest_path = workspace_root.join("Cargo.toml");
    let content = fs::read_to_string(&root_manifest_path)
        .with_context(|| format!("failed to read {}", root_manifest_path.display()))?;
    let root_table: Table = content
        .parse()
        .with_context(|| "failed to parse root Cargo.toml")?;

    let declared_paths = collect_declared_crate_paths(&root_table);
    let excluded_paths = collect_excluded_paths(&root_table);

    let mut violations = Vec::new();
    let src_dir = workspace_root.join("src");
    scan_src_crates(
        &src_dir,
        workspace_root,
        &declared_paths,
        &excluded_paths,
        &mut violations,
    )?;

    violations.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    Ok(violations)
}

#[cfg(test)]
#[allow(
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
    use super::*;

    #[test]
    fn test_collect_declared_crate_paths() {
        let toml_str = r#"
[package]
name = "root"

[dependencies]
crate-a = { path = "src/crate_a" }
crate-b = { path = "./src/crate_b/" }
external-dep = "1.0.0"

[build-dependencies]
build-crate = { path = "src/build_crate" }

[target.'cfg(unix)'.dependencies]
unix-crate = { path = "src/unix_crate" }

[workspace]
members = ["src/member_crate"]
exclude = ["src/excluded_crate"]
"#;
        let table: Table = toml_str.parse().unwrap();
        let paths = collect_declared_crate_paths(&table);
        assert!(paths.contains(Path::new("src/crate_a")));
        assert!(paths.contains(Path::new("src/crate_b")));
        assert!(paths.contains(Path::new("src/build_crate")));
        assert!(paths.contains(Path::new("src/unix_crate")));
        assert!(paths.contains(Path::new("src/member_crate")));
        assert_eq!(paths.len(), 5);

        let excluded = collect_excluded_paths(&table);
        assert!(excluded.contains(Path::new("src/excluded_crate")));
        assert_eq!(excluded.len(), 1);
    }

    #[test]
    fn test_normalize_rel_path() {
        assert_eq!(
            normalize_rel_path(Path::new("./src/formats/../formats/bzip")),
            PathBuf::from("src/formats/../formats/bzip")
        );
        assert_eq!(
            normalize_rel_path(Path::new("./src/foo/bar/")),
            PathBuf::from("src/foo/bar")
        );
    }
}
