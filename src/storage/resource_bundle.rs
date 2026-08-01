#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use ctb_formats_ctb_asset_bundle as asset_bundle_format;
use ctb_storage_asset_bundle_locator::find_resource_bundle_path;
use glob::Pattern;
use memmap2::Mmap;
use std::collections::HashMap;
use std::fs::File;
use std::ops::Range;
use std::path::Path;
use std::sync::OnceLock;
use uuid::Uuid;

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

const EXPECTED_RESOURCE_BUNDLE_UUID: &str = env!("CTB_ASSET_PACK_UUID");
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
    mmap_index: usize,
    data_range: Range<usize>,
}

#[derive(Debug)]
struct ResourceBundle {
    entries: Vec<ResourceBundleEntry>,
    entry_by_path: HashMap<String, usize>,
    mmaps: Vec<Mmap>,
}

pub(crate) fn get_asset(key: &str) -> Option<Vec<u8>> {
    project_assets().ok()?.get_bytes(key).map(ToOwned::to_owned)
}

pub(crate) fn get_asset_utf8(key: &str) -> Result<String> {
    let bytes = get_asset(key)
        .ok_or_else(|| anyhow::anyhow!("Failed to load asset {key}"))?;
    String::from_utf8(bytes)
        .with_context(|| format!("Failed to decode UTF-8 asset {key}"))
}

pub(crate) fn find_assets(glob: &str) -> Result<Vec<String>> {
    project_assets()?.find_paths(glob)
}

pub(crate) fn validate_resource_bundle() -> Result<()> {
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
            Some(EXPECTED_RESOURCE_BUNDLE_UUID),
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
                mmap_index: 0,
                data_range: parsed_entry.data_range,
            });
            entry_by_path.insert(parsed_entry.path, entry_index);
        }

        let mut mmaps = vec![main_mmap];

        // Try loading separate v86_images.rsrc if present
        let v86_path = bundle_path.with_file_name("v86_images.rsrc");
        if v86_path.is_file() {
            if let Ok(v86_file) = open_resource_bundle_file(&v86_path) {
                #[expect(unsafe_code, reason = "Mmap requires unsafe")]
                // SAFETY: The resource bundle file is not modified by other processes during read-only mapping.
                if let Ok(v86_mmap) = unsafe { Mmap::map(&v86_file) } {
                    if let Ok(parsed_v86) =
                        asset_bundle_format::parse_asset_bundle(&v86_mmap)
                    {
                        let verify_res = verify_bundle_integrity(
                            &v86_path.display().to_string(),
                            &parsed_v86.header,
                            EXPECTED_V86_RESOURCE_BUNDLE_UUID,
                            EXPECTED_V86_RESOURCE_BUNDLE_SHA256,
                        );
                        if verify_res.is_ok() {
                            let mmap_idx = mmaps.len();
                            mmaps.push(v86_mmap);
                            for parsed_entry in parsed_v86.entries {
                                let entry_index = entries.len();
                                entries.push(ResourceBundleEntry {
                                    path: parsed_entry.path.clone(),
                                    mmap_index: mmap_idx,
                                    data_range: parsed_entry.data_range,
                                });
                                entry_by_path
                                    .insert(parsed_entry.path, entry_index);
                            }
                        }
                    }
                }
            }
        }

        Ok(Self {
            entries,
            entry_by_path,
            mmaps,
        })
    }

    fn get_bytes(&self, key: &str) -> Option<&[u8]> {
        let normalized = normalize_asset_key(key);
        let index = self.entry_by_path.get(normalized)?;
        let entry = self.entries.get(*index)?;
        let mmap = self.mmaps.get(entry.mmap_index)?;
        mmap.get(entry.data_range.clone())
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
    key.strip_prefix('/').unwrap_or(key)
}
