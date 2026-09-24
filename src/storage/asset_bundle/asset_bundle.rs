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
use std::collections::{HashMap, VecDeque};
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

const CHUNK_SIZE: u64 = 128 * 1024 * 1024;
const MAX_CACHED_CHUNKS: usize = 4;

#[derive(Debug, Clone)]
enum EntryLocation {
    Direct {
        mmap_index: usize,
        data_range: Range<usize>,
    },
    Chunked {
        chunked_bundle_index: usize,
        data_offset: u64,
        data_len: u64,
    },
}

#[derive(Debug)]
struct ResourceBundleEntry {
    path: String,
    flags: u32,
    location: EntryLocation,
}

#[derive(Debug)]
struct ChunkCache {
    chunks: HashMap<u64, Arc<Mmap>>,
    order: VecDeque<u64>,
    max_chunks: usize,
}

impl ChunkCache {
    fn new(max_chunks: usize) -> Self {
        Self {
            chunks: HashMap::new(),
            order: VecDeque::new(),
            max_chunks,
        }
    }

    fn get(&mut self, chunk_idx: u64) -> Option<Arc<Mmap>> {
        if let Some(mmap) = self.chunks.get(&chunk_idx) {
            if let Some(pos) = self.order.iter().position(|&k| k == chunk_idx) {
                self.order.remove(pos);
                self.order.push_back(chunk_idx);
            }
            Some(Arc::clone(mmap))
        } else {
            None
        }
    }

    fn insert(&mut self, chunk_idx: u64, mmap: Arc<Mmap>) {
        if self.chunks.contains_key(&chunk_idx) {
            if let Some(pos) = self.order.iter().position(|&k| k == chunk_idx) {
                self.order.remove(pos);
            }
        } else if self.chunks.len() >= self.max_chunks {
            if let Some(oldest) = self.order.pop_front() {
                self.chunks.remove(&oldest);
            }
        }
        self.chunks.insert(chunk_idx, mmap);
        self.order.push_back(chunk_idx);
    }
}

#[derive(Debug)]
struct ChunkedResourceBundle {
    file: Arc<File>,
    file_len: u64,
    cache: RwLock<ChunkCache>,
}

impl ChunkedResourceBundle {
    fn open(
        bundle_path: &Path,
    ) -> Result<(Self, Vec<asset_bundle_format::RawAssetBundleEntry>)> {
        let file = open_resource_bundle_file(bundle_path)?;
        let file_len = file
            .metadata()
            .with_context(|| format!("Failed to stat {}", bundle_path.display()))?
            .len();

        let initial_map_len = std::cmp::min(file_len, CHUNK_SIZE);
        let initial_map_usize = usize::try_from(initial_map_len)
            .context("initial chunk map len overflow usize")?;

        #[expect(unsafe_code, reason = "Mmap requires unsafe")]
        // SAFETY: The resource bundle file is not modified by other processes
        // during read-only mapping.
        let chunk_0 = unsafe {
            memmap2::MmapOptions::new()
                .offset(0)
                .len(initial_map_usize)
                .map(&file)
        }
        .with_context(|| {
            format!(
                "Failed to map header chunk of {}",
                bundle_path.display()
            )
        })?;

        let (header, raw_entries) =
            asset_bundle_format::parse_asset_bundle_metadata(&chunk_0)
                .with_context(|| {
                    format!(
                        "Failed to parse metadata of {}",
                        bundle_path.display()
                    )
                })?;

        verify_bundle_integrity(
            &bundle_path.display().to_string(),
            &header,
            EXPECTED_V86_RESOURCE_BUNDLE_UUID,
            EXPECTED_V86_RESOURCE_BUNDLE_SHA256,
        )?;

        let mut cache = ChunkCache::new(MAX_CACHED_CHUNKS);
        cache.insert(0, Arc::new(chunk_0));

        let bundle = Self {
            file: Arc::new(file),
            file_len,
            cache: RwLock::new(cache),
        };

        Ok((bundle, raw_entries))
    }

    fn get_chunk(&self, chunk_idx: u64) -> Result<Arc<Mmap>> {
        if let Ok(mut cache) = self.cache.write() {
            if let Some(cached) = cache.get(chunk_idx) {
                return Ok(cached);
            }

            let chunk_offset = chunk_idx
                .checked_mul(CHUNK_SIZE)
                .context("Chunk offset overflow")?;
            let remaining = self
                .file_len
                .checked_sub(chunk_offset)
                .context("Chunk offset exceeds file length")?;
            let map_len = std::cmp::min(remaining, CHUNK_SIZE);
            let map_len_usize = usize::try_from(map_len)
                .context("Chunk length overflow usize")?;

            #[expect(unsafe_code, reason = "Mmap requires unsafe")]
            // SAFETY: The resource bundle file is not modified by other processes
            // during read-only mapping.
            let mmap = unsafe {
                memmap2::MmapOptions::new()
                    .offset(chunk_offset)
                    .len(map_len_usize)
                    .map(&*self.file)
            }
            .with_context(|| {
                format!(
                    "Failed to map chunk {chunk_idx} at offset {chunk_offset}"
                )
            })?;

            let mmap_arc = Arc::new(mmap);
            cache.insert(chunk_idx, Arc::clone(&mmap_arc));
            Ok(mmap_arc)
        } else {
            bail!("Chunk cache lock poisoned")
        }
    }

    fn get_bytes(&self, data_offset: u64, data_len: u64) -> Option<Vec<u8>> {
        if data_len == 0 {
            return Some(Vec::new());
        }

        let data_end = data_offset.checked_add(data_len)?;
        if data_end > self.file_len {
            return None;
        }
        let data_len_usize = usize::try_from(data_len).ok()?;

        let last_byte_offset = data_end.checked_sub(1)?;
        let start_chunk = data_offset.checked_div(CHUNK_SIZE)?;
        let end_chunk = last_byte_offset.checked_div(CHUNK_SIZE)?;

        if start_chunk == end_chunk {
            let chunk = self.get_chunk(start_chunk).ok()?;
            let chunk_base = start_chunk.checked_mul(CHUNK_SIZE)?;
            let start_in_chunk =
                usize::try_from(data_offset.checked_sub(chunk_base)?).ok()?;
            let end_in_chunk =
                usize::try_from(data_end.checked_sub(chunk_base)?).ok()?;
            let slice = chunk.get(start_in_chunk..end_in_chunk)?;
            return Some(slice.to_vec());
        }

        let mut out = Vec::with_capacity(data_len_usize);
        let mut cur_chunk_idx = start_chunk;
        while cur_chunk_idx <= end_chunk {
            let chunk = self.get_chunk(cur_chunk_idx).ok()?;
            let chunk_base = cur_chunk_idx.checked_mul(CHUNK_SIZE)?;
            let chunk_len = u64::try_from(chunk.len()).ok()?;

            let sub_start_u64 = if cur_chunk_idx == start_chunk {
                data_offset.checked_sub(chunk_base)?
            } else {
                0
            };
            let sub_end_u64 = if cur_chunk_idx == end_chunk {
                data_end.checked_sub(chunk_base)?
            } else {
                chunk_len
            };
            let sub_start = usize::try_from(sub_start_u64).ok()?;
            let sub_end = usize::try_from(sub_end_u64).ok()?;
            let slice = chunk.get(sub_start..sub_end)?;
            out.extend_from_slice(slice);

            cur_chunk_idx = cur_chunk_idx.checked_add(1)?;
        }

        Some(out)
    }
}

#[derive(Debug)]
struct ResourceBundle {
    entries: Vec<ResourceBundleEntry>,
    entry_by_path: HashMap<String, usize>,
    mmaps: Vec<Mmap>,
    chunked_bundles: Vec<ChunkedResourceBundle>,
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
        if let Some(grandparent) = parent.parent() {
            if grandparent.file_name() == Some(std::ffi::OsStr::new("built")) {
                candidates.push(grandparent.join("ctoolbox.rsrc"));
            }
        }
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
                                location: EntryLocation::Direct {
                                    mmap_index: 0,
                                    data_range: abs_start..abs_end,
                                },
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
                location: EntryLocation::Direct {
                    mmap_index: 0,
                    data_range: parsed_entry.data_range,
                },
            });
            entry_by_path.insert(parsed_entry.path, entry_index);
        }

        let mmaps = vec![main_mmap];
        let mut chunked_bundles = Vec::new();

        // Try loading separate v86_images.rsrc if present
        let v86_path = bundle_path.with_file_name("v86_images.rsrc");
        if v86_path.is_file() {
            let load_result = (|| -> Result<()> {
                let (chunked_bundle, raw_entries) =
                    ChunkedResourceBundle::open(&v86_path)?;
                let chunked_idx = chunked_bundles.len();
                chunked_bundles.push(chunked_bundle);

                for parsed_entry in raw_entries {
                    let entry_index = entries.len();
                    entries.push(ResourceBundleEntry {
                        path: parsed_entry.path.clone(),
                        flags: parsed_entry.flags,
                        location: EntryLocation::Chunked {
                            chunked_bundle_index: chunked_idx,
                            data_offset: parsed_entry.data_offset,
                            data_len: parsed_entry.data_len,
                        },
                    });
                    entry_by_path.insert(parsed_entry.path, entry_index);
                }
                Ok(())
            })();

            if let Err(err) = load_result {
                warn_fmt!(
                    "v86 resource bundle at {} could not be loaded: {err:#}; v86 VM assets will not be loaded",
                    v86_path.display()
                );
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
            chunked_bundles,
            delta_cache: RwLock::new(HashMap::new()),
        })
    }

    fn get_asset_vec(&self, key: &str) -> Option<Vec<u8>> {
        let normalized = normalize_asset_key(key);
        let index = self.entry_by_path.get(normalized).or_else(|| {
            if let Some(tail) = normalized
                .strip_prefix("vendor/v86_images/")
                .or_else(|| normalized.strip_prefix("v86_images/"))
            {
                let alias = format!("images/{tail}");
                self.entry_by_path.get(&alias)
            } else {
                None
            }
        })?;
        let entry = self.entries.get(*index)?;

        let raw_vec = match &entry.location {
            EntryLocation::Direct {
                mmap_index,
                data_range,
            } => {
                let mmap = self.mmaps.get(*mmap_index)?;
                let raw_slice = mmap.get(data_range.clone())?;
                raw_slice.to_vec()
            }
            EntryLocation::Chunked {
                chunked_bundle_index,
                data_offset,
                data_len,
            } => {
                let chunked = self.chunked_bundles.get(*chunked_bundle_index)?;
                chunked.get_bytes(*data_offset, *data_len)?
            }
        };

        if entry.flags & asset_bundle_format::ASSET_FLAG_DELTA == 0 {
            return Some(raw_vec);
        }

        if let Ok(cache) = self.delta_cache.read() {
            if let Some(cached) = cache.get(normalized) {
                return Some((**cached).clone());
            }
        }

        let (base_path, delta_bytes) =
            asset_bundle_format::delta::decode_delta_payload(&raw_vec).ok()?;
        let base_bytes = self.get_asset_vec(base_path)?;
        let target_bytes =
            asset_bundle_format::delta::decode_delta(&base_bytes, delta_bytes)
                .ok()?;

        if let Ok(mut cache) = self.delta_cache.write() {
            cache
                .insert(normalized.to_string(), Arc::new(target_bytes.clone()));
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

    #[expect(unsafe_code, reason = "Mmap in test requires unsafe")]
    #[crate::ctb_test]
    fn test_chunk_cache_eviction() {
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new().expect("create temp file");
        // SAFETY: Temporary file is private and not modified concurrently during read-only mapping.
        let mmap = unsafe { Mmap::map(temp_file.as_file()).expect("mmap temp file") };
        let arc_mmap = Arc::new(mmap);

        let mut cache = ChunkCache::new(2);
        cache.insert(0, Arc::clone(&arc_mmap));
        cache.insert(1, Arc::clone(&arc_mmap));
        assert!(cache.get(0).is_some());

        // Inserting chunk 2 should evict chunk 1 (since chunk 0 was recently accessed)
        cache.insert(2, Arc::clone(&arc_mmap));
        assert!(cache.get(0).is_some());
        assert!(cache.get(1).is_none());
        assert!(cache.get(2).is_some());
    }

    #[crate::ctb_test]
    fn test_v86_images_chunked_loading_if_present() {
        if let Ok(v86_path) =
            find_resource_bundle_path().map(|p| p.with_file_name("v86_images.rsrc"))
        {
            if v86_path.is_file() {
                let asset = get_asset("vendor/v86_images/guix/guix-fs.json");
                assert!(
                    asset.is_some(),
                    "Expected vendor/v86_images/guix/guix-fs.json to be loaded via chunked bundle"
                );
                let bytes = asset.unwrap();
                assert!(
                    !bytes.is_empty() && bytes[0] == b'{',
                    "Expected guix-fs.json to be non-empty JSON"
                );
            }
        }
    }

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
    fn built_platform_binary_uses_workspace_built_bundle() {
        let exe_path = Path::new("/repo/built/linux-x64/ctoolbox");

        assert_eq!(workspace_root_for_cargo_target_exe(exe_path), None);
        assert_eq!(
            resource_bundle_candidates_for_exe(exe_path),
            vec![
                PathBuf::from("/repo/built/linux-x64/ctoolbox.rsrc"),
                PathBuf::from("/repo/built/ctoolbox.rsrc"),
            ]
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
