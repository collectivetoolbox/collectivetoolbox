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

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use cargo_metadata::{Metadata, MetadataCommand, Package};
use toml::{Table, Value};

fn main() -> Result<()> {
    let workspace_root = workspace_root_from_args()?;
    let metadata = MetadataCommand::new()
        .manifest_path(workspace_root.join("Cargo.toml"))
        .exec()
        .context("failed to load cargo metadata")?;
    let root_manifest = metadata.workspace_root.join("Cargo.toml");
    let root_document = parse_manifest(root_manifest.as_std_path())?;
    let patched_crates = patched_crates(&workspace_root, &root_document);

    let mut violations = Vec::new();
    for package in &metadata.packages {
        if !metadata.workspace_members.contains(&package.id) {
            continue;
        }
        if should_skip_manifest(
            &workspace_root,
            package.manifest_path.as_std_path(),
        ) {
            continue;
        }
        collect_manifest_violations(
            &workspace_root,
            root_manifest.as_std_path(),
            package,
            &mut violations,
        )?;
    }

    collect_resolution_violations(
        &workspace_root,
        &metadata,
        &patched_crates,
        &mut violations,
    );

    if violations.is_empty() {
        println!("patch lint passed");
        return Ok(());
    }

    eprintln!("patch lint failed:");
    for violation in violations {
        eprintln!("- {violation}");
    }

    bail!("found patch manifest violations")
}

fn workspace_root_from_args() -> Result<PathBuf> {
    let mut args = env::args_os();
    let _program = args.next();
    let Some(root) = args.next() else {
        bail!("usage: lint-patches <workspace-root>");
    };
    if args.next().is_some() {
        bail!("usage: lint-patches <workspace-root>");
    }
    Ok(PathBuf::from(root))
}

fn collect_manifest_violations(
    workspace_root: &Path,
    root_manifest: &Path,
    package: &Package,
    violations: &mut Vec<String>,
) -> Result<()> {
    let manifest_path = package.manifest_path.as_std_path();
    let manifest = parse_manifest(manifest_path)?;
    let display_path = display_path(workspace_root, manifest_path);

    if manifest_path != root_manifest && has_patch_crates_io(&manifest) {
        violations.push(format!(
            "{display_path}: non-root manifest declares [patch.crates-io], which Cargo ignores"
        ));
    }

    Ok(())
}

fn parse_manifest(path: &Path) -> Result<Table> {
    let text = fs::read_to_string(path).with_context(|| {
        format!("failed to read manifest {}", path.display())
    })?;
    text.parse::<Table>()
        .with_context(|| format!("failed to parse TOML in {}", path.display()))
}

fn patched_crates(
    workspace_root: &Path,
    document: &Table,
) -> BTreeMap<String, PathBuf> {
    let mut crates = BTreeMap::new();
    let Some(patch_table) = document.get("patch").and_then(Value::as_table)
    else {
        return crates;
    };
    let Some(crates_io) =
        patch_table.get("crates-io").and_then(Value::as_table)
    else {
        return crates;
    };

    for (crate_name, patch_value) in crates_io {
        let Some(patch_table) = patch_value.as_table() else {
            continue;
        };
        let Some(path_value) = patch_table.get("path").and_then(Value::as_str)
        else {
            continue;
        };
        crates.insert(crate_name.clone(), workspace_root.join(path_value));
    }

    crates
}

fn has_patch_crates_io(document: &Table) -> bool {
    document
        .get("patch")
        .and_then(Value::as_table)
        .and_then(|patch| patch.get("crates-io"))
        .is_some()
}

fn collect_resolution_violations(
    workspace_root: &Path,
    metadata: &Metadata,
    patched_crates: &BTreeMap<String, PathBuf>,
    violations: &mut Vec<String>,
) {
    for (crate_name, expected_manifest_dir) in patched_crates {
        let resolved_packages: Vec<&Package> = metadata
            .packages
            .iter()
            .filter(|package| package.name == *crate_name)
            .collect();
        if resolved_packages.is_empty() {
            violations.push(format!(
                "root [patch.crates-io] entry {crate_name} at {} is unused by the current resolution",
                display_path(workspace_root, expected_manifest_dir)
            ));
            continue;
        }

        let expected_manifest = expected_manifest_dir.join("Cargo.toml");
        for package in resolved_packages {
            let actual_manifest = package.manifest_path.as_std_path();
            if actual_manifest == expected_manifest {
                continue;
            }

            violations.push(format!(
                "patched crate {crate_name} resolved to {} instead of {}",
                display_path(workspace_root, actual_manifest),
                display_path(workspace_root, &expected_manifest)
            ));
        }
    }
}

fn display_path(workspace_root: &Path, path: &Path) -> String {
    if let Ok(relative) = path.strip_prefix(workspace_root) {
        return relative.display().to_string();
    }
    path.display().to_string()
}

fn should_skip_manifest(workspace_root: &Path, manifest_path: &Path) -> bool {
    let Ok(relative) = manifest_path.strip_prefix(workspace_root) else {
        return false;
    };
    relative
        .components()
        .next()
        .is_some_and(|component| component.as_os_str() == "vendor")
}
