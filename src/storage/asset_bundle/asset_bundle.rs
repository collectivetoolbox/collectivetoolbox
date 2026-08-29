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

//! Embedded and external resource bundle loading, path location, and memory-mapped access.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace prelude"
)]
pub(crate) use ctb_utilities::utilities::*;

use ctb_formats_ctb_asset_bundle as asset_bundle_format;
use glob::Pattern;
use memmap2::Mmap;
use std::collections::HashMap;
use std::fs::{self, File};
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock, RwLock};
use uuid::Uuid;

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

const EXPECTED_RESOURCE_BUNDLE_UUID: Option<&str> =
    option_env!("CTB_ASSET_PACK_UUID");
const EXPECTED_RESOURCE_BUNDLE_SHA256: Option<&str> =
    option_env!("CTB_ASSET_PACK_SHA256");

const EXPECTED_V86_RESOURCE_BUNDLE_UUID: Option<&str> =
    option_env!("CTB_V86_ASSET_PACK_UUID");
const EXPECTED_V86_RESOURCE_BUNDLE_SHA256: Option<&str> =
    option_env!("CTB_V86_ASSET_PACK_SHA256");

static PROJECT_ASSETS: OnceLock<Result<ResourceBundle, String>> =
    OnceLock::new();

#[derive(Debug)]
struct ResourceBundleEntry {
    path: String,
    flags: u32,
    mmap_index: usize,
    data_range: Range<usize>,
}

#[derive(Debug)]
struct ResourceBundle {
    entries: Vec<ResourceBundleEntry>,
    entry_by_path: HashMap<String, usize>,
    mmaps: Vec<Mmap>,
    delta_cache: RwLock<HashMap<String, Arc<Vec<u8>>>>,
}

/// Retrieves the raw bytes of an embedded or bundled asset by key.
pub fn get_asset(key: &str) -> Option<Vec<u8>> {
    let bundle = project_assets().ok()?;
    bundle.get_asset_vec(key)
}

/// Retrieves the UTF-8 text of an embedded or bundled asset by key.
pub fn get_asset_utf8(key: &str) -> Result<String> {
    let bytes = get_asset(key)
        .ok_or_else(|| anyhow::anyhow!("Failed to load asset {key}"))?;
    String::from_utf8(bytes)
        .with_context(|| format!("Failed to decode UTF-8 asset {key}"))
}

/// Finds asset paths matching a glob pattern.
pub fn find_assets(glob: &str) -> Result<Vec<String>> {
    project_assets()?.find_paths(glob)
}

/// Validates that the resource bundle can be located and opened.
pub fn validate_resource_bundle() -> Result<()> {
    let _bundle = project_assets()?;
    Ok(())
}

fn project_assets() -> Result<&'static ResourceBundle> {
    let bundle = PROJECT_ASSETS
        .get_or_init(|| ResourceBundle::load().map_err(|err| err.to_string()));
    match bundle {
        Ok(bundle) => Ok(bundle),
        Err(err) => Err(anyhow::anyhow!(err.clone())),
    }
}

/// Discovers the path to the primary resource bundle (`ctoolbox.rsrc`).
pub fn find_resource_bundle_path() -> Result<PathBuf> {
    let mut candidates = Vec::new();
    let mut exe_dir = None;

    if let Ok(exe_path) = std::env::current_exe() {
        // Reason for fallback: if binary executable path cannot be canonicalized, use raw std::env::current_exe path
        let exe_path = fs::canonicalize(&exe_path).unwrap_or(exe_path);
        if let Some(parent) = exe_path.parent() {
            exe_dir = Some(parent.to_path_buf());
        }
        candidates.extend(resource_bundle_candidates_for_exe(&exe_path));
    }

    if utilities::testing::is_in_test() {
        candidates.push(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../../built/ctoolbox.rsrc"),
        );
    }

    for candidate in &candidates {
        if let Some(resolved) =
            resolve_allowed_candidate(candidate, exe_dir.as_deref())
        {
            return Ok(resolved);
        }
    }

    let searched = candidates
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    bail!("Could not find ctoolbox.rsrc in: {searched}")
}

fn resource_bundle_candidates_for_exe(exe_path: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(parent) = exe_path.parent() {
        candidates.push(parent.join("ctoolbox.rsrc"));
    }

    if let Some(workspace_root) = workspace_root_for_cargo_target_exe(exe_path)
    {
        candidates.push(workspace_root.join("built/ctoolbox.rsrc"));
    }

    candidates
}

pub fn is_cargo_target_binary() -> bool {
    ctb_utilities::environment::is_cargo_target_binary()
}

fn workspace_root_for_cargo_target_exe(exe_path: &Path) -> Option<PathBuf> {
    ctb_utilities::workspace_path_resolution::workspace_root_for_cargo_target_exe(exe_path)
}

fn resolve_allowed_candidate(
    candidate: &Path,
    exe_dir: Option<&Path>,
) -> Option<PathBuf> {
    match fs::symlink_metadata(candidate) {
        Ok(metadata) => {
            let file_type = metadata.file_type();
            if file_type.is_file() && !file_type.is_symlink() {
                Some(candidate.to_path_buf())
            } else if file_type.is_symlink() {
                let Some(exe_dir) = exe_dir else {
                    return None;
                };
                let Ok(resolved) = fs::canonicalize(candidate) else {
                    return None;
                };
                if resolved.is_file() {
                    let Some(resolved_parent) = resolved.parent() else {
                        return None;
                    };
                    if resolved_parent == exe_dir {
                        return Some(resolved);
                    }
                }
                None
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

fn verify_bundle_integrity(
    bundle_name: &str,
    header: &asset_bundle_format::AssetBundleHeader,
    expected_uuid: Option<&str>,
    expected_sha256: Option<&str>,
) -> Result<()> {
    let is_dev = environment::is_cargo_target_binary()
        || environment::is_in_test()
        || environment::is_debug_build();

    if let Some(exp_uuid_str) = expected_uuid {
        if let Ok(exp_uuid) = Uuid::parse_str(exp_uuid_str) {
            if header.bundle_uuid != exp_uuid {
                let msg = format!(
                    "Resource bundle UUID mismatch in {bundle_name}: expected {exp_uuid_str}, found {}",
                    header.bundle_uuid
                );
                if is_dev {
                    warn!(msg);
                } else {
                    bail!(msg);
                }
            }
        }
    }

    if let Some(exp_sha) = expected_sha256 {
        let found_sha =
            asset_bundle_format::format_sha256_hex(&header.content_sha256);
        if found_sha != exp_sha {
            let msg = format!(
                "Resource bundle SHA256 mismatch in {bundle_name}: expected {exp_sha}, found {found_sha}"
            );
            if is_dev {
                warn!(msg);
            } else {
                bail!(msg);
            }
        }
    }

    Ok(())
}

impl ResourceBundle {
    fn load() -> Result<Self> {
        let bundle_path = find_resource_bundle_path()?;
        let file = open_resource_bundle_file(&bundle_path)?;
        #[expect(unsafe_code, reason = "Mmap requires unsafe")]
        // SAFETY: The resource bundle file is not modified by other processes during read-only mapping.
        let main_mmap = unsafe { Mmap::map(&file) }.with_context(|| {
            format!("Failed to map {}", bundle_path.display())
        })?;

        let parsed = asset_bundle_format::parse_asset_bundle(&main_mmap)
            .with_context(|| {
                format!("Failed to parse {}", bundle_path.display())
            })?;

        verify_bundle_integrity(
            &bundle_path.display().to_string(),
            &parsed.header,
            EXPECTED_RESOURCE_BUNDLE_UUID,
            EXPECTED_RESOURCE_BUNDLE_SHA256,
        )?;

        let main_mmap_ref = &main_mmap;
        let mut entries = Vec::with_capacity(parsed.entries.len());
        let mut entry_by_path = HashMap::with_capacity(parsed.entries.len());

        for parsed_entry in parsed.entries {
            if parsed_entry.path.ends_with(".rsrc") {
                if let Some(inner_bytes) =
                    main_mmap_ref.get(parsed_entry.data_range.clone())
                {
                    if let Ok(inner_bundle) =
                        asset_bundle_format::parse_asset_bundle(inner_bytes)
                    {
                        let offset = parsed_entry.data_range.start;
                        for inner_entry in inner_bundle.entries {
                            let abs_start = offset
                                .saturating_add(inner_entry.data_range.start);
                            let abs_end = offset
                                .saturating_add(inner_entry.data_range.end);
                            let entry_index = entries.len();
                            entries.push(ResourceBundleEntry {
                                path: inner_entry.path.clone(),
                                flags: inner_entry.flags,
                                mmap_index: 0,
                                data_range: abs_start..abs_end,
                            });
                            entry_by_path.insert(inner_entry.path, entry_index);
                        }
                        continue;
                    }
                }
            }

            let entry_index = entries.len();
            entries.push(ResourceBundleEntry {
                path: parsed_entry.path.clone(),
                flags: parsed_entry.flags,
                mmap_index: 0,
                data_range: parsed_entry.data_range,
            });
            entry_by_path.insert(parsed_entry.path, entry_index);
        }

        let mut mmaps = vec![main_mmap];

        // Try loading separate v86_images.rsrc if present
        let v86_path = bundle_path.with_file_name("v86_images.rsrc");
        if v86_path.is_file() {
            let v86_file = open_resource_bundle_file(&v86_path).with_context(|| {
                format!("Failed to open v86 resource bundle {}", v86_path.display())
            })?;
            #[expect(unsafe_code, reason = "Mmap requires unsafe")]
            // SAFETY: The resource bundle file is not modified by other processes during read-only mapping.
            let v86_mmap = unsafe { Mmap::map(&v86_file) }.with_context(|| {
                format!("Failed to map v86 resource bundle {}", v86_path.display())
            })?;
            let parsed_v86 = asset_bundle_format::parse_asset_bundle(&v86_mmap)
                .with_context(|| {
                    format!(
                        "Failed to parse v86 resource bundle {}",
                        v86_path.display()
                    )
                })?;
            verify_bundle_integrity(
                &v86_path.display().to_string(),
                &parsed_v86.header,
                EXPECTED_V86_RESOURCE_BUNDLE_UUID,
                EXPECTED_V86_RESOURCE_BUNDLE_SHA256,
            )?;
            let mmap_idx = mmaps.len();
            mmaps.push(v86_mmap);
            for parsed_entry in parsed_v86.entries {
                let entry_index = entries.len();
                entries.push(ResourceBundleEntry {
                    path: parsed_entry.path.clone(),
                    flags: parsed_entry.flags,
                    mmap_index: mmap_idx,
                    data_range: parsed_entry.data_range,
                });
                entry_by_path.insert(parsed_entry.path, entry_index);
            }
        } else {
            warn!(
                "v86 resource bundle not found at {}; v86 VM assets will not be loaded",
                v86_path.display()
            );
        }

        Ok(Self {
            entries,
            entry_by_path,
            mmaps,
            delta_cache: RwLock::new(HashMap::new()),
        })
    }

    fn get_asset_vec(&self, key: &str) -> Option<Vec<u8>> {
        let normalized = normalize_asset_key(key);
        let index = self.entry_by_path.get(normalized)?;
        let entry = self.entries.get(*index)?;
        let mmap = self.mmaps.get(entry.mmap_index)?;
        let raw_slice = mmap.get(entry.data_range.clone())?;

        if entry.flags & asset_bundle_format::ASSET_FLAG_DELTA == 0 {
            return Some(raw_slice.to_vec());
        }

        if let Ok(cache) = self.delta_cache.read() {
            if let Some(cached) = cache.get(normalized) {
                return Some((**cached).clone());
            }
        }

        let (base_path, delta_bytes) =
            asset_bundle_format::delta::decode_delta_payload(raw_slice).ok()?;
        let base_bytes = self.get_asset_vec(base_path)?;
        let target_bytes =
            asset_bundle_format::delta::decode_delta(&base_bytes, delta_bytes).ok()?;

        if let Ok(mut cache) = self.delta_cache.write() {
            cache.insert(normalized.to_string(), Arc::new(target_bytes.clone()));
        }

        Some(target_bytes)
    }

    fn find_paths(&self, glob: &str) -> Result<Vec<String>> {
        let pattern = Pattern::new(glob)
            .with_context(|| format!("Failed to parse asset glob {glob}"))?;
        let mut matches = Vec::new();
        for entry in &self.entries {
            if pattern.matches(&entry.path) {
                matches.push(entry.path.clone());
            }
        }
        matches.sort();
        Ok(matches)
    }
}

fn open_resource_bundle_file(bundle_path: &Path) -> Result<File> {
    #[cfg(unix)]
    let file = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(bundle_path)
        .with_context(|| format!("Failed to open {}", bundle_path.display()))?;

    #[cfg(not(unix))]
    let file = File::open(bundle_path)
        .with_context(|| format!("Failed to open {}", bundle_path.display()))?;

    let metadata = file
        .metadata()
        .with_context(|| format!("Failed to stat {}", bundle_path.display()))?;
    ensure!(
        metadata.is_file(),
        "Resource bundle path is not a regular file: {}",
        bundle_path.display()
    );

    Ok(file)
}

fn normalize_asset_key(key: &str) -> &str {
    // Reason for fallback: asset key without leading slash retains original relative key path
    key.strip_prefix('/').unwrap_or(key)
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
    use super::*;

    #[crate::ctb_test]
    fn cargo_release_binary_uses_workspace_built_bundle() {
        let exe_path = Path::new("/repo/target/release/js-lint");

        assert_eq!(
            workspace_root_for_cargo_target_exe(exe_path),
            Some(PathBuf::from("/repo"))
        );
        assert_eq!(
            resource_bundle_candidates_for_exe(exe_path),
            vec![
                PathBuf::from("/repo/target/release/ctoolbox.rsrc"),
                PathBuf::from("/repo/built/ctoolbox.rsrc"),
            ]
        );
    }

    #[crate::ctb_test]
    fn cargo_deps_binary_uses_workspace_built_bundle() {
        let exe_path = Path::new("/repo/target/debug/deps/locator-tests");

        assert_eq!(
            workspace_root_for_cargo_target_exe(exe_path),
            Some(PathBuf::from("/repo"))
        );
        assert_eq!(
            resource_bundle_candidates_for_exe(exe_path),
            vec![
                PathBuf::from("/repo/target/debug/deps/ctoolbox.rsrc"),
                PathBuf::from("/repo/built/ctoolbox.rsrc"),
            ]
        );
    }

    #[crate::ctb_test]
    fn cargo_target_triple_release_binary_uses_workspace_built_bundle() {
        let exe_path =
            Path::new("/repo/target/x86_64-unknown-linux-musl/release/js-lint");

        assert_eq!(
            workspace_root_for_cargo_target_exe(exe_path),
            Some(PathBuf::from("/repo"))
        );
        assert_eq!(
            resource_bundle_candidates_for_exe(exe_path),
            vec![
                PathBuf::from(
                    "/repo/target/x86_64-unknown-linux-musl/release/ctoolbox.rsrc"
                ),
                PathBuf::from("/repo/built/ctoolbox.rsrc"),
            ]
        );
    }

    #[crate::ctb_test]
    fn cargo_target_triple_deps_binary_uses_workspace_built_bundle() {
        let exe_path = Path::new(
            "/repo/target/x86_64-unknown-linux-musl/debug/deps/locator-tests",
        );

        assert_eq!(
            workspace_root_for_cargo_target_exe(exe_path),
            Some(PathBuf::from("/repo"))
        );
        assert_eq!(
            resource_bundle_candidates_for_exe(exe_path),
            vec![
                PathBuf::from(
                    "/repo/target/x86_64-unknown-linux-musl/debug/deps/ctoolbox.rsrc"
                ),
                PathBuf::from("/repo/built/ctoolbox.rsrc"),
            ]
        );
    }

    #[crate::ctb_test]
    fn non_cargo_binary_does_not_use_workspace_built_bundle() {
        let exe_path = Path::new("/opt/ctoolbox/bin/js-lint");

        assert_eq!(workspace_root_for_cargo_target_exe(exe_path), None);
        assert_eq!(
            resource_bundle_candidates_for_exe(exe_path),
            vec![PathBuf::from("/opt/ctoolbox/bin/ctoolbox.rsrc")]
        );
    }

    #[crate::ctb_test]
    fn test_candidate_is_allowed_file_symlink() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let dir = temp_dir.path();

        let file_path = dir.join("ctoolbox-0.1.5.rsrc");
        fs::write(&file_path, b"test").expect("Failed to write test file");

        let symlink_path = dir.join("ctoolbox.rsrc");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&file_path, &symlink_path)
            .expect("Failed to create symlink");

        // If candidate is a regular file, it should be allowed (even without exe_dir)
        let resolved = resolve_allowed_candidate(&file_path, None);
        assert_eq!(resolved, Some(file_path.clone()));

        // If candidate is a symlink, and exe_dir matches resolved_parent, it should be allowed and return resolved path
        #[cfg(unix)]
        {
            let resolved = resolve_allowed_candidate(&symlink_path, Some(dir));
            assert_eq!(resolved, Some(file_path.clone()));

            // If candidate is a symlink, and exe_dir does not match resolved_parent, it should be rejected
            let other_dir = Path::new("/other/dir");
            let resolved =
                resolve_allowed_candidate(&symlink_path, Some(other_dir));
            assert_eq!(resolved, None);
        }
    }
}
