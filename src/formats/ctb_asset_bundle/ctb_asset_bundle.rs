//! Asset bundle file format support.
//!
//! This crate owns the on-disk `.rsrc` file format: header layout, index
//! parsing, serialization, and includes a utility for extraction back into a
//! directory tree (useful for debugging; in normal use it's memory-mapped).
//!
//! V1 headers:
//!
//! ```ignore
//! const RESOURCE_BUNDLE_MAGIC: &[u8; 8] = b"CTBRSRC\0";
//! const RESOURCE_BUNDLE_VERSION: u32 = 1;
//! const RESOURCE_ENTRY_SIZE: usize = 32;
//! ```
//!
//! V2 headers add a bundle UUID based on the content:
//! ```ignore
//! const RESOURCE_BUNDLE_MAGIC: &[u8; 8] = b"CTBRSRC\0";
//! const RESOURCE_BUNDLE_VERSION: u32 = 2;
//! const RESOURCE_BUNDLE_UUID_SIZE: usize = 16;
//! const RESOURCE_ENTRY_SIZE: usize = 32;
//! const EXPECTED_RESOURCE_BUNDLE_UUID: &str = env!("CTB_ASSET_PACK_UUID");
//! ```
//!
//! V2 UUID was implemented as follows:
//!
//! ```ignore
//! fn compute_resource_bundle_uuid(
//!     entries: &[ResourceBundleEntry],
//! ) -> Result<[u8; RESOURCE_BUNDLE_UUID_SIZE]> {
//!     let mut hasher = Sha256::new();
//!     for entry in entries {
//!         let path_len = u64::try_from(entry.path.len())
//!             .context("resource bundle path length overflow")?;
//!         hasher.update(path_len.to_le_bytes());
//!         hasher.update(entry.path.as_bytes());
//!
//!         let contents_len = u64::try_from(entry.contents.len())
//!             .context("resource bundle contents length overflow")?;
//!         hasher.update(contents_len.to_le_bytes());
//!         hasher.update(&entry.contents);
//!     }
//!
//!     let digest = hasher.finalize();
//!     let mut uuid_bytes = [0_u8; RESOURCE_BUNDLE_UUID_SIZE];
//!     for (dst, src) in uuid_bytes.iter_mut().zip(digest.iter()) {
//!         *dst = *src;
//!     }
//!
//!     // Mark this as an RFC 4122 variant UUID with a custom-content version.
//!     uuid_bytes[6] = (uuid_bytes[6] & 0x0f) | 0x80;
//!     uuid_bytes[8] = (uuid_bytes[8] & 0x3f) | 0x80;
//!
//!     Ok(uuid_bytes)
//! }
//! ```
//!
//! V3 headers add content SHA256 and creation timestamp, and switch to a random
//! UUID.

use anyhow::{Context, Result, bail, ensure};
use include_dir::{Dir, include_dir};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::ffi::OsString;
use std::fs;
use std::ops::Range;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

static CTB_ASSET_BUNDLE_DATA_DIR: Dir =
    include_dir!("$CARGO_MANIFEST_DIR/data");

pub fn get_embedded_asset(dir: &Dir, key: &str) -> Option<Vec<u8>> {
    // Reason for fallback: keys without leading slash are looked up directly.
    let key = key.strip_prefix('/').unwrap_or(key);
    let file = dir.get_file(key);
    Some(file?.contents().to_vec())
}

pub mod delta;

pub fn get_ctb_asset_bundle_data(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&CTB_ASSET_BUNDLE_DATA_DIR, key)
}

pub const RESOURCE_BUNDLE_MAGIC: &[u8; 8] = b"CTBRSRC\0";
pub const RESOURCE_BUNDLE_MAGIC_LEN: usize = 8;
const ENTRY_COUNT_OFFSET: usize = 12;
const UUID_OFFSET: usize = 16;
const SHA256_OFFSET: usize = 32;
const TIMESTAMP_OFFSET: usize = 64;
pub const RESOURCE_BUNDLE_VERSION_V1: u32 = 1;
pub const RESOURCE_BUNDLE_VERSION_V2: u32 = 2;
pub const RESOURCE_BUNDLE_VERSION_V3: u32 = 3;
pub const RESOURCE_BUNDLE_VERSION_V4: u32 = 4;
pub const RESOURCE_BUNDLE_VERSION: u32 = 4;
pub const RESOURCE_BUNDLE_UUID_SIZE: usize = 16;
pub const RESOURCE_BUNDLE_SHA256_SIZE: usize = 32;
pub const RESOURCE_BUNDLE_TIMESTAMP_SIZE: usize = 8;
pub const RESOURCE_ENTRY_SIZE: usize = 32;
pub const RESOURCE_BUNDLE_V1_HEADER_SIZE: usize = 16;
pub const RESOURCE_BUNDLE_V2_HEADER_SIZE: usize = 32;
pub const RESOURCE_BUNDLE_HEADER_SIZE: usize = 72;

pub const ASSET_FLAG_RAW: u32 = 0;
pub const ASSET_FLAG_DELTA: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetBundleSourceEntry {
    pub path: String,
    pub contents: Vec<u8>,
    pub flags: u32,
}

impl AssetBundleSourceEntry {
    #[must_use]
    pub fn raw(path: impl Into<String>, contents: Vec<u8>) -> Self {
        Self {
            path: path.into(),
            contents,
            flags: ASSET_FLAG_RAW,
        }
    }

    #[must_use]
    pub fn delta(path: impl Into<String>, delta_payload: Vec<u8>) -> Self {
        Self {
            path: path.into(),
            contents: delta_payload,
            flags: ASSET_FLAG_DELTA,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetBundleHeader {
    pub version: u32,
    pub entry_count: u32,
    pub bundle_uuid: Uuid,
    pub content_sha256: [u8; RESOURCE_BUNDLE_SHA256_SIZE],
    pub created_at_unix_secs: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetBundleEntry {
    pub path: String,
    pub flags: u32,
    pub data_range: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedAssetBundle {
    pub header: AssetBundleHeader,
    pub entries: Vec<AssetBundleEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AssetBundleMetadata {
    pub version: u32,
    pub entry_count: u32,
    pub bundle_uuid: Option<String>,
    pub content_sha256: Option<String>,
    pub created_at_unix_secs: Option<u64>,
}

impl From<&AssetBundleHeader> for AssetBundleMetadata {
    fn from(header: &AssetBundleHeader) -> Self {
        Self {
            version: header.version,
            entry_count: header.entry_count,
            bundle_uuid: bundle_uuid_metadata_value(header),
            content_sha256: content_sha256_metadata_value(header),
            created_at_unix_secs: created_at_metadata_value(header),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedAssetBundleHeader {
    header: AssetBundleHeader,
    header_size: usize,
}

pub fn compute_asset_bundle_content_sha256(
    entries: &[AssetBundleSourceEntry],
) -> Result<[u8; RESOURCE_BUNDLE_SHA256_SIZE]> {
    let entries = normalized_source_entries(entries)?;
    hash_normalized_entries(&entries)
}

pub fn compute_v2_resource_bundle_uuid(
    entries: &[AssetBundleSourceEntry],
) -> Result<Uuid> {
    let entries = normalized_source_entries(entries)?;
    let mut hasher = Sha256::new();
    for entry in &entries {
        let path_len = u64::try_from(entry.path.len())
            .context("resource bundle path length overflow")?;
        hasher.update(path_len.to_le_bytes());
        hasher.update(entry.path.as_bytes());

        let contents_len = u64::try_from(entry.contents.len())
            .context("resource bundle contents length overflow")?;
        hasher.update(contents_len.to_le_bytes());
        hasher.update(&entry.contents);
    }

    let digest = hasher.finalize();
    let mut uuid_bytes = [0_u8; RESOURCE_BUNDLE_UUID_SIZE];
    for (dst, src) in uuid_bytes.iter_mut().zip(digest.iter()) {
        *dst = *src;
    }

    // Mark this as an RFC 4122 variant UUID with a custom-content version.
    let byte6 = uuid_bytes.get_mut(6).context("uuid byte 6 missing")?;
    *byte6 = (*byte6 & 0x0f) | 0x80;
    let byte8 = uuid_bytes.get_mut(8).context("uuid byte 8 missing")?;
    *byte8 = (*byte8 & 0x3f) | 0x80;

    Ok(Uuid::from_bytes(uuid_bytes))
}

fn build_asset_bundle_internal(
    entries: &[AssetBundleSourceEntry],
    version: u32,
    custom_uuid: Option<Uuid>,
    custom_created_at: Option<u64>,
) -> Result<(Vec<u8>, AssetBundleHeader)> {
    let entries = normalized_source_entries(entries)?;
    let content_sha256 = hash_normalized_entries(&entries)?;
    let entry_count = u32::try_from(entries.len())
        .context("Too many resource bundle entries")?;

    let header_size = match version {
        RESOURCE_BUNDLE_VERSION_V1 => RESOURCE_BUNDLE_V1_HEADER_SIZE,
        RESOURCE_BUNDLE_VERSION_V2 => RESOURCE_BUNDLE_V2_HEADER_SIZE,
        RESOURCE_BUNDLE_VERSION_V3 | RESOURCE_BUNDLE_VERSION_V4 => {
            RESOURCE_BUNDLE_HEADER_SIZE
        }
        _ => bail!("Unsupported resource bundle version {version}"),
    };

    let bundle_uuid = match version {
        RESOURCE_BUNDLE_VERSION_V1 => Uuid::nil(),
        RESOURCE_BUNDLE_VERSION_V2 => match custom_uuid {
            Some(uuid) => uuid,
            None => compute_v2_resource_bundle_uuid(&entries)?,
        },
        RESOURCE_BUNDLE_VERSION_V3 | RESOURCE_BUNDLE_VERSION_V4 => {
            // Reason for fallback: when custom UUID is not provided for v3/v4 bundle, generate a random UUID.
            custom_uuid.unwrap_or_else(Uuid::new_v4)
        }
        _ => bail!("Unsupported resource bundle version {version}"),
    };

    let created_at_unix_secs = match version {
        RESOURCE_BUNDLE_VERSION_V1 | RESOURCE_BUNDLE_VERSION_V2 => 0,
        RESOURCE_BUNDLE_VERSION_V3 | RESOURCE_BUNDLE_VERSION_V4 => {
            match custom_created_at {
                Some(ts) => ts,
                None => SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .context("System clock is before UNIX_EPOCH")?
                    .as_secs(),
            }
        }
        _ => bail!("Unsupported resource bundle version {version}"),
    };

    let index_bytes = RESOURCE_ENTRY_SIZE
        .checked_mul(usize::try_from(entry_count).context("entry count")?)
        .context("Resource bundle index too large")?;

    let mut path_bytes = Vec::new();
    let mut data_bytes = Vec::new();
    let mut index = Vec::with_capacity(index_bytes);
    let mut path_offsets = Vec::with_capacity(entries.len());
    let mut data_offsets = Vec::with_capacity(entries.len());

    for entry in &entries {
        let path_offset = u64::try_from(path_bytes.len())
            .context("Resource bundle path table too large")?;
        path_offsets.push(path_offset);
        path_bytes.extend_from_slice(entry.path.as_bytes());

        let data_offset = u64::try_from(data_bytes.len())
            .context("Resource bundle data section too large")?;
        data_offsets.push(data_offset);
        data_bytes.extend_from_slice(&entry.contents);
    }

    let paths_start = u64::try_from(header_size)
        .context("header size overflow")?
        .checked_add(u64::try_from(index_bytes).context("index size overflow")?)
        .context("paths start overflow")?;
    let data_start = paths_start
        .checked_add(u64::try_from(path_bytes.len()).context("paths len")?)
        .context("data start overflow")?;

    for (entry, (path_offset, data_offset)) in entries
        .iter()
        .zip(path_offsets.into_iter().zip(data_offsets.into_iter()))
    {
        append_u64(
            &mut index,
            paths_start
                .checked_add(path_offset)
                .context("paths start offset overflow")?,
        );
        append_u32(
            &mut index,
            u32::try_from(entry.path.len()).context("path length overflow")?,
        );
        append_u32(&mut index, entry.flags);
        append_u64(
            &mut index,
            data_start
                .checked_add(data_offset)
                .context("data start offset overflow")?,
        );
        append_u64(
            &mut index,
            u64::try_from(entry.contents.len()).context("content length")?,
        );
    }

    let total_capacity = header_size
        .checked_add(index.len())
        .and_then(|size| size.checked_add(path_bytes.len()))
        .and_then(|size| size.checked_add(data_bytes.len()))
        .context("Resource bundle too large")?;

    let header = AssetBundleHeader {
        version,
        entry_count,
        bundle_uuid,
        content_sha256,
        created_at_unix_secs,
    };

    let mut bundle = Vec::with_capacity(total_capacity);
    bundle.extend_from_slice(RESOURCE_BUNDLE_MAGIC);
    append_u32(&mut bundle, header.version);
    append_u32(&mut bundle, header.entry_count);

    if version >= RESOURCE_BUNDLE_VERSION_V2 {
        bundle.extend_from_slice(header.bundle_uuid.as_bytes());
    }
    if version >= RESOURCE_BUNDLE_VERSION_V3 {
        bundle.extend_from_slice(&header.content_sha256);
        append_u64(&mut bundle, header.created_at_unix_secs);
    }

    bundle.extend_from_slice(&index);
    bundle.extend_from_slice(&path_bytes);
    bundle.extend_from_slice(&data_bytes);

    Ok((bundle, header))
}

pub fn build_asset_bundle(
    entries: &[AssetBundleSourceEntry],
) -> Result<(Vec<u8>, AssetBundleHeader)> {
    build_asset_bundle_internal(entries, RESOURCE_BUNDLE_VERSION, None, None)
}

pub fn build_asset_bundle_v1(
    entries: &[AssetBundleSourceEntry],
) -> Result<(Vec<u8>, AssetBundleHeader)> {
    build_asset_bundle_internal(entries, RESOURCE_BUNDLE_VERSION_V1, None, None)
}

pub fn build_asset_bundle_v2(
    entries: &[AssetBundleSourceEntry],
) -> Result<(Vec<u8>, AssetBundleHeader)> {
    build_asset_bundle_internal(entries, RESOURCE_BUNDLE_VERSION_V2, None, None)
}

pub fn build_asset_bundle_v2_with_uuid(
    entries: &[AssetBundleSourceEntry],
    bundle_uuid: Uuid,
) -> Result<(Vec<u8>, AssetBundleHeader)> {
    build_asset_bundle_internal(
        entries,
        RESOURCE_BUNDLE_VERSION_V2,
        Some(bundle_uuid),
        None,
    )
}

pub fn build_asset_bundle_v3(
    entries: &[AssetBundleSourceEntry],
) -> Result<(Vec<u8>, AssetBundleHeader)> {
    build_asset_bundle_internal(entries, RESOURCE_BUNDLE_VERSION_V3, None, None)
}

pub fn build_asset_bundle_v3_with_details(
    entries: &[AssetBundleSourceEntry],
    bundle_uuid: Uuid,
    created_at_unix_secs: u64,
) -> Result<(Vec<u8>, AssetBundleHeader)> {
    build_asset_bundle_internal(
        entries,
        RESOURCE_BUNDLE_VERSION_V3,
        Some(bundle_uuid),
        Some(created_at_unix_secs),
    )
}

pub fn build_asset_bundle_v4(
    entries: &[AssetBundleSourceEntry],
) -> Result<(Vec<u8>, AssetBundleHeader)> {
    build_asset_bundle_internal(entries, RESOURCE_BUNDLE_VERSION_V4, None, None)
}

pub fn build_asset_bundle_v4_with_details(
    entries: &[AssetBundleSourceEntry],
    bundle_uuid: Uuid,
    created_at_unix_secs: u64,
) -> Result<(Vec<u8>, AssetBundleHeader)> {
    build_asset_bundle_internal(
        entries,
        RESOURCE_BUNDLE_VERSION_V4,
        Some(bundle_uuid),
        Some(created_at_unix_secs),
    )
}

pub fn create_test_fixture_entries(
    pan_file: &[u8],
    lzma2_file: &[u8],
) -> Vec<AssetBundleSourceEntry> {
    vec![
        AssetBundleSourceEntry::raw("example2 with lemurs.pan", pan_file.to_vec()),
        AssetBundleSourceEntry::raw(
            "test directory/test directory 2/example2 with lemurs.pan.lzma2",
            lzma2_file.to_vec(),
        ),
    ]
}

pub fn create_test_fixture_entries_v4(
    pan_file: &[u8],
    lzma2_file: &[u8],
    edited_bin_file: &[u8],
) -> Result<Vec<AssetBundleSourceEntry>> {
    let delta_payload = delta::encode_delta_payload(
        "example2 with lemurs.pan",
        pan_file,
        edited_bin_file,
    )?;
    Ok(vec![
        AssetBundleSourceEntry::raw("example2 with lemurs.pan", pan_file.to_vec()),
        AssetBundleSourceEntry::delta(
            "example2 with lemurs-edited.bin",
            delta_payload,
        ),
        AssetBundleSourceEntry::raw(
            "test directory/test directory 2/example2 with lemurs.pan.lzma2",
            lzma2_file.to_vec(),
        ),
    ])
}

pub fn parse_asset_bundle_header(bytes: &[u8]) -> Result<AssetBundleHeader> {
    Ok(parse_asset_bundle_header_inner(bytes)?.header)
}

pub fn parse_asset_bundle(bytes: &[u8]) -> Result<ParsedAssetBundle> {
    let parsed_header = parse_asset_bundle_header_inner(bytes)?;
    let header = parsed_header.header;
    let header_size = parsed_header.header_size;
    let entry_count =
        usize::try_from(header.entry_count).context("entry count overflow")?;
    let index_bytes = RESOURCE_ENTRY_SIZE
        .checked_mul(entry_count)
        .context("resource bundle index too large")?;
    let required_len = header_size
        .checked_add(index_bytes)
        .context("header size and index size overflow")?;
    ensure!(
        bytes.len() >= required_len,
        "Resource bundle index truncated"
    );

    let mut seen_paths = HashSet::<String>::new();
    let mut entries = Vec::with_capacity(entry_count);
    for index in 0..entry_count {
        let entry_offset = RESOURCE_ENTRY_SIZE
            .checked_mul(index)
            .and_then(|offset| header_size.checked_add(offset))
            .context("entry offset overflow")?;
        let path_offset = usize::try_from(read_u64(bytes, entry_offset)?)
            .context("path offset overflow")?;
        let path_len = usize::try_from(read_u32(
            bytes,
            entry_offset
                .checked_add(8)
                .context("entry path len offset overflow")?,
        )?)
        .context("path length overflow")?;
        let flags = read_u32(
            bytes,
            entry_offset
                .checked_add(12)
                .context("entry flags offset overflow")?,
        )?;
        let data_offset = usize::try_from(read_u64(
            bytes,
            entry_offset
                .checked_add(16)
                .context("entry data offset overflow")?,
        )?)
        .context("data offset overflow")?;
        let data_len = usize::try_from(read_u64(
            bytes,
            entry_offset
                .checked_add(24)
                .context("entry data len offset overflow")?,
        )?)
        .context("data length overflow")?;

        let path_end = path_offset
            .checked_add(path_len)
            .context("path range overflow")?;
        let path_bytes = bytes
            .get(path_offset..path_end)
            .ok_or_else(|| anyhow::anyhow!("Path range out of bounds"))?;
        let path = std::str::from_utf8(path_bytes)
            .context("Invalid path encoding in resource bundle")?
            .to_owned();
        let path = normalize_bundle_path(&path)?;
        ensure!(
            seen_paths.insert(path.clone()),
            "Duplicate bundle path {path}"
        );

        let data_end = data_offset
            .checked_add(data_len)
            .context("data range overflow")?;
        ensure!(
            bytes.get(data_offset..data_end).is_some(),
            "Data range out of bounds for {path}"
        );

        entries.push(AssetBundleEntry {
            path,
            flags,
            data_range: data_offset..data_end,
        });
    }

    Ok(ParsedAssetBundle { header, entries })
}

fn parse_asset_bundle_header_inner(
    bytes: &[u8],
) -> Result<ParsedAssetBundleHeader> {
    let magic = bytes
        .get(..RESOURCE_BUNDLE_MAGIC.len())
        .ok_or_else(|| anyhow::anyhow!("Resource bundle header missing"))?;
    ensure!(
        magic == RESOURCE_BUNDLE_MAGIC,
        "Unsupported resource bundle magic"
    );

    let version = read_u32(bytes, RESOURCE_BUNDLE_MAGIC_LEN)?;
    let entry_count = read_u32(bytes, ENTRY_COUNT_OFFSET)?;
    let parsed_header = match version {
        RESOURCE_BUNDLE_VERSION_V1 => ParsedAssetBundleHeader {
            header: AssetBundleHeader {
                version,
                entry_count,
                bundle_uuid: Uuid::nil(),
                content_sha256: [0_u8; RESOURCE_BUNDLE_SHA256_SIZE],
                created_at_unix_secs: 0,
            },
            header_size: RESOURCE_BUNDLE_V1_HEADER_SIZE,
        },
        RESOURCE_BUNDLE_VERSION_V2 => ParsedAssetBundleHeader {
            header: AssetBundleHeader {
                version,
                entry_count,
                bundle_uuid: read_uuid(bytes, UUID_OFFSET)?,
                content_sha256: [0_u8; RESOURCE_BUNDLE_SHA256_SIZE],
                created_at_unix_secs: 0,
            },
            header_size: RESOURCE_BUNDLE_V2_HEADER_SIZE,
        },
        RESOURCE_BUNDLE_VERSION_V3 | RESOURCE_BUNDLE_VERSION_V4 => {
            ParsedAssetBundleHeader {
                header: AssetBundleHeader {
                    version,
                    entry_count,
                    bundle_uuid: read_uuid(bytes, UUID_OFFSET)?,
                    content_sha256: read_fixed_bytes::<RESOURCE_BUNDLE_SHA256_SIZE>(
                        bytes,
                        SHA256_OFFSET,
                        "sha256",
                    )?,
                    created_at_unix_secs: read_u64(bytes, TIMESTAMP_OFFSET)?,
                },
                header_size: RESOURCE_BUNDLE_HEADER_SIZE,
            }
        }
        _ => bail!("Unsupported resource bundle version {version}"),
    };

    Ok(parsed_header)
}

/// Retrieves an asset's decompressed contents from a parsed asset bundle.
#[must_use]
pub fn get_bundle_asset(
    bytes: &[u8],
    entries: &[AssetBundleEntry],
    key: &str,
) -> Option<Vec<u8>> {
    // Reason for fallback: keys without leading slash are looked up directly.
    let normalized = key.strip_prefix('/').unwrap_or(key);
    let entry = entries.iter().find(|e| e.path == normalized)?;
    let raw_slice = bytes.get(entry.data_range.clone())?;
    if entry.flags & ASSET_FLAG_DELTA == 0 {
        return Some(raw_slice.to_vec());
    }

    let (base_path, delta_bytes) = delta::decode_delta_payload(raw_slice).ok()?;
    let base_data = get_bundle_asset(bytes, entries, base_path)?;
    delta::decode_delta(&base_data, delta_bytes).ok()
}

pub fn extract_asset_bundle(
    bundle_path: &Path,
    output_dir: &Path,
) -> Result<PathBuf> {
    let bundle_bytes = fs::read(bundle_path)
        .with_context(|| format!("Failed to read {}", bundle_path.display()))?;
    let bundle = parse_asset_bundle(&bundle_bytes).with_context(|| {
        format!("Failed to parse {}", bundle_path.display())
    })?;

    let output_dir = output_dir.join(format!(
        "{}-extracted",
        bundle_path
            .file_stem()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Failed to get file stem for {}",
                    bundle_path.display()
                )
            })?
            .to_string_lossy()
    ));
    ensure!(
        !output_dir.exists(),
        "Extraction output already exists: {}",
        output_dir.display()
    );

    let assets_dir = output_dir.join("assets");
    fs::create_dir_all(&assets_dir).with_context(|| {
        format!("Failed to create {}", assets_dir.display())
    })?;

    let metadata = AssetBundleMetadata::from(&bundle.header);
    let metadata_json = serde_json::to_vec_pretty(&metadata)
        .context("Failed to encode bundle metadata as JSON")?;
    fs::write(output_dir.join("metadata.json"), metadata_json).with_context(
        || {
            format!(
                "Failed to write {}",
                output_dir.join("metadata.json").display()
            )
        },
    )?;

    for entry in &bundle.entries {
        let relative_path = bundle_entry_pathbuf(&entry.path)?;
        let asset_path = assets_dir.join(relative_path);
        let parent = asset_path.parent().ok_or_else(|| {
            anyhow::anyhow!("No parent directory for {}", asset_path.display())
        })?;
        fs::create_dir_all(parent).with_context(|| {
            format!("Failed to create {}", parent.display())
        })?;
        let data = get_bundle_asset(&bundle_bytes, &bundle.entries, &entry.path)
            .ok_or_else(|| {
                anyhow::anyhow!("Failed to retrieve asset data for {}", entry.path)
            })?;
        fs::write(&asset_path, data).with_context(|| {
            format!("Failed to write {}", asset_path.display())
        })?;
    }

    Ok(output_dir)
}

/// Copy of method from `utilities/string.rs`.
fn to_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().saturating_mul(2));
    let chars = b"0123456789abcdef";
    for &byte in bytes {
        let hi = usize::from(byte >> 4);
        let lo = usize::from(byte & 0xf);
        if let (Some(&c1), Some(&c2)) = (chars.get(hi), chars.get(lo)) {
            out.push(char::from(c1));
            out.push(char::from(c2));
        }
    }
    out
}

pub fn format_sha256_hex(
    content_sha256: &[u8; RESOURCE_BUNDLE_SHA256_SIZE],
) -> String {
    to_hex(content_sha256)
}

fn bundle_uuid_metadata_value(header: &AssetBundleHeader) -> Option<String> {
    if header.version >= RESOURCE_BUNDLE_VERSION_V2 {
        return Some(header.bundle_uuid.to_string());
    }
    None
}

fn content_sha256_metadata_value(header: &AssetBundleHeader) -> Option<String> {
    if header.version >= RESOURCE_BUNDLE_VERSION_V3 {
        return Some(format_sha256_hex(&header.content_sha256));
    }
    None
}

fn created_at_metadata_value(header: &AssetBundleHeader) -> Option<u64> {
    if header.version >= RESOURCE_BUNDLE_VERSION_V3 {
        return Some(header.created_at_unix_secs);
    }
    None
}

fn normalized_source_entries(
    entries: &[AssetBundleSourceEntry],
) -> Result<Vec<AssetBundleSourceEntry>> {
    let mut normalized = Vec::with_capacity(entries.len());
    let mut seen_paths = HashSet::<String>::new();
    for entry in entries {
        let path = normalize_bundle_path(&entry.path)?;
        ensure!(
            seen_paths.insert(path.clone()),
            "Duplicate bundle path {path}"
        );
        normalized.push(AssetBundleSourceEntry {
            path,
            contents: entry.contents.clone(),
            flags: entry.flags,
        });
    }
    normalized.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(normalized)
}

fn hash_normalized_entries(
    entries: &[AssetBundleSourceEntry],
) -> Result<[u8; RESOURCE_BUNDLE_SHA256_SIZE]> {
    let mut hasher = Sha256::new();
    for entry in entries {
        let path_len = u64::try_from(entry.path.len())
            .context("resource bundle path length overflow")?;
        hasher.update(path_len.to_le_bytes());
        hasher.update(entry.path.as_bytes());

        let contents_len = u64::try_from(entry.contents.len())
            .context("resource bundle contents length overflow")?;
        hasher.update(contents_len.to_le_bytes());
        hasher.update(&entry.contents);
    }
    let digest = hasher.finalize();
    let mut out = [0_u8; RESOURCE_BUNDLE_SHA256_SIZE];
    for (dst, src) in out.iter_mut().zip(digest.iter()) {
        *dst = *src;
    }
    Ok(out)
}

fn normalize_bundle_path(path: &str) -> Result<String> {
    let mut normalized = PathBuf::new();
    for component in Path::new(path).components() {
        match component {
            Component::Normal(part) => normalized.push(part),
            Component::CurDir => {}
            other => bail!("Unsupported bundle path component: {other:?}"),
        }
    }
    Ok(normalized.to_string_lossy().replace('\\', "/"))
}

fn bundle_entry_pathbuf(path: &str) -> Result<PathBuf> {
    let mut out = PathBuf::new();
    for part in path.split('/') {
        if part.is_empty() || part == "." || part == ".." {
            bail!("Unsupported bundle extraction path component: {part}");
        }
        out.push(OsString::from(part));
    }
    Ok(out)
}

fn append_u32(buf: &mut Vec<u8>, value: u32) {
    buf.extend_from_slice(&value.to_le_bytes());
}

fn append_u64(buf: &mut Vec<u8>, value: u64) {
    buf.extend_from_slice(&value.to_le_bytes());
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32> {
    let end = offset.checked_add(4).context("u32 offset overflow")?;
    let slice = bytes
        .get(offset..end)
        .ok_or_else(|| anyhow::anyhow!("u32 out of bounds at {offset}"))?;
    let array: [u8; 4] = slice
        .try_into()
        .map_err(|_err| anyhow::anyhow!("u32 size mismatch"))?;
    Ok(u32::from_le_bytes(array))
}

fn read_u64(bytes: &[u8], offset: usize) -> Result<u64> {
    let end = offset.checked_add(8).context("u64 offset overflow")?;
    let slice = bytes
        .get(offset..end)
        .ok_or_else(|| anyhow::anyhow!("u64 out of bounds at {offset}"))?;
    let array: [u8; 8] = slice
        .try_into()
        .map_err(|_err| anyhow::anyhow!("u64 size mismatch"))?;
    Ok(u64::from_le_bytes(array))
}

fn read_uuid(bytes: &[u8], offset: usize) -> Result<Uuid> {
    let array =
        read_fixed_bytes::<RESOURCE_BUNDLE_UUID_SIZE>(bytes, offset, "uuid")?;
    Ok(Uuid::from_bytes(array))
}

fn read_fixed_bytes<const N: usize>(
    bytes: &[u8],
    offset: usize,
    label: &str,
) -> Result<[u8; N]> {
    let end = offset
        .checked_add(N)
        .context("fixed byte offset overflow")?;
    let slice = bytes
        .get(offset..end)
        .ok_or_else(|| anyhow::anyhow!("{label} out of bounds at {offset}"))?;
    let array: [u8; N] = slice
        .try_into()
        .map_err(|_err| anyhow::anyhow!("{label} size mismatch"))?;
    Ok(array)
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

    fn raw_test_files() -> (Vec<u8>, Vec<u8>, Vec<u8>) {
        let pan_bytes =
            get_ctb_asset_bundle_data("fixtures/example2 with lemurs.pan")
                .expect("fixtures/example2 with lemurs.pan missing from embedded assets");
        let lzma2_bytes = get_ctb_asset_bundle_data(
            "fixtures/example2 with lemurs.pan.lzma2",
        )
        .expect("fixtures/example2 with lemurs.pan.lzma2 missing from embedded assets");
        let edited_bytes = get_ctb_asset_bundle_data(
            "fixtures/example2 with lemurs-edited.bin",
        )
        .expect("fixtures/example2 with lemurs-edited.bin missing from embedded assets");
        (pan_bytes, lzma2_bytes, edited_bytes)
    }

    #[test]
    fn test_embedded_fixtures_exist() {
        assert!(get_ctb_asset_bundle_data("fixtures/bundle_v1.rsrc").is_some());
        assert!(get_ctb_asset_bundle_data("fixtures/bundle_v2.rsrc").is_some());
        assert!(get_ctb_asset_bundle_data("fixtures/bundle_v3.rsrc").is_some());
        assert!(get_ctb_asset_bundle_data("fixtures/bundle_v4.rsrc").is_some());
        assert!(get_ctb_asset_bundle_data("fixtures/example2 with lemurs.pan").is_some());
        assert!(get_ctb_asset_bundle_data("fixtures/example2 with lemurs.pan.lzma2").is_some());
        assert!(get_ctb_asset_bundle_data("fixtures/example2 with lemurs-edited.bin").is_some());
    }

    #[test]
    fn test_unpack_v1_bundle() -> Result<()> {
        let (raw_pan, raw_lzma2, _) = raw_test_files();
        let bundle_bytes =
            get_ctb_asset_bundle_data("fixtures/bundle_v1.rsrc")
                .context("bundle_v1.rsrc missing from embedded assets")?;

        let parsed = parse_asset_bundle(&bundle_bytes)?;
        assert_eq!(parsed.header.version, RESOURCE_BUNDLE_VERSION_V1);
        assert_eq!(parsed.header.entry_count, 2);
        assert_eq!(parsed.header.bundle_uuid, Uuid::nil());
        assert_eq!(parsed.header.content_sha256, [0_u8; 32]);
        assert_eq!(parsed.header.created_at_unix_secs, 0);

        assert_eq!(parsed.entries.len(), 2);
        assert_eq!(parsed.entries[0].path, "example2 with lemurs.pan");
        assert_eq!(
            &bundle_bytes[parsed.entries[0].data_range.clone()],
            &raw_pan[..]
        );
        assert_eq!(
            parsed.entries[1].path,
            "test directory/test directory 2/example2 with lemurs.pan.lzma2"
        );
        assert_eq!(
            &bundle_bytes[parsed.entries[1].data_range.clone()],
            &raw_lzma2[..]
        );

        Ok(())
    }

    #[test]
    fn test_unpack_v2_bundle() -> Result<()> {
        let (raw_pan, raw_lzma2, _) = raw_test_files();
        let bundle_bytes =
            get_ctb_asset_bundle_data("fixtures/bundle_v2.rsrc")
                .context("bundle_v2.rsrc missing from embedded assets")?;

        let entries = create_test_fixture_entries(&raw_pan, &raw_lzma2);
        let expected_uuid = compute_v2_resource_bundle_uuid(&entries)?;

        let parsed = parse_asset_bundle(&bundle_bytes)?;
        assert_eq!(parsed.header.version, RESOURCE_BUNDLE_VERSION_V2);
        assert_eq!(parsed.header.entry_count, 2);
        assert_eq!(parsed.header.bundle_uuid, expected_uuid);
        assert_eq!(parsed.header.content_sha256, [0_u8; 32]);
        assert_eq!(parsed.header.created_at_unix_secs, 0);

        assert_eq!(parsed.entries.len(), 2);
        assert_eq!(parsed.entries[0].path, "example2 with lemurs.pan");
        assert_eq!(
            &bundle_bytes[parsed.entries[0].data_range.clone()],
            &raw_pan[..]
        );
        assert_eq!(
            parsed.entries[1].path,
            "test directory/test directory 2/example2 with lemurs.pan.lzma2"
        );
        assert_eq!(
            &bundle_bytes[parsed.entries[1].data_range.clone()],
            &raw_lzma2[..]
        );

        Ok(())
    }

    #[test]
    fn test_unpack_v3_bundle() -> Result<()> {
        let (raw_pan, raw_lzma2, _) = raw_test_files();
        let bundle_bytes =
            get_ctb_asset_bundle_data("fixtures/bundle_v3.rsrc")
                .context("bundle_v3.rsrc missing from embedded assets")?;

        let entries = create_test_fixture_entries(&raw_pan, &raw_lzma2);
        let expected_sha = compute_asset_bundle_content_sha256(&entries)?;

        let parsed = parse_asset_bundle(&bundle_bytes)?;
        assert_eq!(parsed.header.version, RESOURCE_BUNDLE_VERSION_V3);
        assert_eq!(parsed.header.entry_count, 2);
        assert_ne!(parsed.header.bundle_uuid, Uuid::nil());
        assert_eq!(parsed.header.content_sha256, expected_sha);
        assert!(parsed.header.created_at_unix_secs > 0);

        assert_eq!(parsed.entries.len(), 2);
        assert_eq!(parsed.entries[0].path, "example2 with lemurs.pan");
        assert_eq!(
            &bundle_bytes[parsed.entries[0].data_range.clone()],
            &raw_pan[..]
        );
        assert_eq!(
            parsed.entries[1].path,
            "test directory/test directory 2/example2 with lemurs.pan.lzma2"
        );
        assert_eq!(
            &bundle_bytes[parsed.entries[1].data_range.clone()],
            &raw_lzma2[..]
        );

        Ok(())
    }

    #[test]
    fn test_unpack_v4_bundle_with_delta() -> Result<()> {
        let (raw_pan, raw_lzma2, raw_edited) = raw_test_files();
        let bundle_bytes =
            get_ctb_asset_bundle_data("fixtures/bundle_v4.rsrc")
                .context("bundle_v4.rsrc missing from embedded assets")?;

        let entries = create_test_fixture_entries_v4(
            &raw_pan,
            &raw_lzma2,
            &raw_edited,
        )?;
        let expected_sha = compute_asset_bundle_content_sha256(&entries)?;

        let parsed = parse_asset_bundle(&bundle_bytes)?;
        assert_eq!(parsed.header.version, RESOURCE_BUNDLE_VERSION_V4);
        assert_eq!(parsed.header.entry_count, 3);
        assert_ne!(parsed.header.bundle_uuid, Uuid::nil());
        assert_eq!(parsed.header.content_sha256, expected_sha);
        assert!(parsed.header.created_at_unix_secs > 0);

        assert_eq!(parsed.entries.len(), 3);
        assert_eq!(parsed.entries[0].path, "example2 with lemurs-edited.bin");
        assert_eq!(parsed.entries[0].flags, ASSET_FLAG_DELTA);

        assert_eq!(parsed.entries[1].path, "example2 with lemurs.pan");
        assert_eq!(parsed.entries[1].flags, ASSET_FLAG_RAW);

        assert_eq!(
            parsed.entries[2].path,
            "test directory/test directory 2/example2 with lemurs.pan.lzma2"
        );
        assert_eq!(parsed.entries[2].flags, ASSET_FLAG_RAW);

        // Verify transparent retrieval of delta-encoded and raw entries
        let extracted_pan =
            get_bundle_asset(&bundle_bytes, &parsed.entries, "example2 with lemurs.pan")
                .context("get_bundle_asset failed for pan")?;
        assert_eq!(extracted_pan, raw_pan);

        let extracted_edited = get_bundle_asset(
            &bundle_bytes,
            &parsed.entries,
            "example2 with lemurs-edited.bin",
        )
        .context("get_bundle_asset failed for edited bin")?;
        assert_eq!(extracted_edited, raw_edited);

        let extracted_lzma2 = get_bundle_asset(
            &bundle_bytes,
            &parsed.entries,
            "test directory/test directory 2/example2 with lemurs.pan.lzma2",
        )
        .context("get_bundle_asset failed for lzma2")?;
        assert_eq!(extracted_lzma2, raw_lzma2);

        Ok(())
    }

    #[test]
    fn test_extract_asset_bundle_all_versions() -> Result<()> {
        let (raw_pan, raw_lzma2, raw_edited) = raw_test_files();
        let tmp_root = std::env::temp_dir().join(format!(
            "ctb_asset_bundle_test_{}",
            Uuid::new_v4()
        ));
        fs::create_dir_all(&tmp_root)?;

        let versions = [
            ("bundle_v1.rsrc", RESOURCE_BUNDLE_VERSION_V1),
            ("bundle_v2.rsrc", RESOURCE_BUNDLE_VERSION_V2),
            ("bundle_v3.rsrc", RESOURCE_BUNDLE_VERSION_V3),
            ("bundle_v4.rsrc", RESOURCE_BUNDLE_VERSION_V4),
        ];

        for (fixture_name, version) in versions {
            let bundle_bytes = get_ctb_asset_bundle_data(&format!(
                "fixtures/{fixture_name}"
            ))
            .context("fixture missing")?;
            let bundle_path = tmp_root.join(fixture_name);
            fs::write(&bundle_path, &bundle_bytes)?;

            let extracted_dir =
                extract_asset_bundle(&bundle_path, &tmp_root)?;
            let assets_dir = extracted_dir.join("assets");
            let pan_extracted =
                fs::read(assets_dir.join("example2 with lemurs.pan"))?;
            assert_eq!(pan_extracted, raw_pan);

            let lzma2_extracted = fs::read(assets_dir.join(
                "test directory/test directory 2/example2 with lemurs.pan.lzma2",
            ))?;
            assert_eq!(lzma2_extracted, raw_lzma2);

            let expected_count = if version == RESOURCE_BUNDLE_VERSION_V4 {
                let edited_extracted =
                    fs::read(assets_dir.join("example2 with lemurs-edited.bin"))?;
                assert_eq!(edited_extracted, raw_edited);
                3
            } else {
                2
            };

            let metadata_bytes =
                fs::read(extracted_dir.join("metadata.json"))?;
            let metadata: serde_json::Value =
                serde_json::from_slice(&metadata_bytes)?;
            assert_eq!(metadata["version"], version);
            assert_eq!(metadata["entry_count"], expected_count);

            if version >= RESOURCE_BUNDLE_VERSION_V2 {
                assert!(metadata["bundle_uuid"].is_string());
            } else {
                assert!(metadata["bundle_uuid"].is_null());
            }

            if version >= RESOURCE_BUNDLE_VERSION_V3 {
                assert!(metadata["content_sha256"].is_string());
                assert!(metadata["created_at_unix_secs"].is_number());
            } else {
                assert!(metadata["content_sha256"].is_null());
                assert!(metadata["created_at_unix_secs"].is_null());
            }
        }

        let _ = fs::remove_dir_all(&tmp_root);
        Ok(())
    }

    /// Generator code used to produce the fixtures in `data/fixtures`.
    /// This is preserved so that fixture creation can be verified or re-run.
    #[test]
    fn test_generate_and_verify_fixture_generation() -> Result<()> {
        let (raw_pan, raw_lzma2, raw_edited) = raw_test_files();
        let entries = create_test_fixture_entries(&raw_pan, &raw_lzma2);

        let (v1_bytes, v1_header) = build_asset_bundle_v1(&entries)?;
        assert_eq!(v1_header.version, 1);
        let parsed_v1 = parse_asset_bundle(&v1_bytes)?;
        assert_eq!(parsed_v1.entries.len(), 2);

        let (v2_bytes, v2_header) = build_asset_bundle_v2(&entries)?;
        assert_eq!(v2_header.version, 2);
        let parsed_v2 = parse_asset_bundle(&v2_bytes)?;
        assert_eq!(parsed_v2.entries.len(), 2);

        let fixed_v3_uuid =
            Uuid::parse_str("3c059cbb-98f6-4ef1-a4b7-db80efd12345")?;
        let fixed_v3_ts = 1_700_000_000_u64;
        let (v3_bytes, v3_header) = build_asset_bundle_v3_with_details(
            &entries,
            fixed_v3_uuid,
            fixed_v3_ts,
        )?;
        assert_eq!(v3_header.version, 3);
        let parsed_v3 = parse_asset_bundle(&v3_bytes)?;
        assert_eq!(parsed_v3.entries.len(), 2);

        let fixed_v4_uuid =
            Uuid::parse_str("4c059cbb-98f6-4ef1-a4b7-db80efd12345")?;
        let fixed_v4_ts = 1_700_000_000_u64;
        let entries_v4 = create_test_fixture_entries_v4(
            &raw_pan,
            &raw_lzma2,
            &raw_edited,
        )?;
        let (v4_bytes, v4_header) = build_asset_bundle_v4_with_details(
            &entries_v4,
            fixed_v4_uuid,
            fixed_v4_ts,
        )?;
        assert_eq!(v4_header.version, 4);
        let parsed_v4 = parse_asset_bundle(&v4_bytes)?;
        assert_eq!(parsed_v4.entries.len(), 3);

        // Verify that the static fixtures match what our builder produces
        let fixture_v1 =
            get_ctb_asset_bundle_data("fixtures/bundle_v1.rsrc").unwrap();
        assert_eq!(v1_bytes, fixture_v1);

        let fixture_v2 =
            get_ctb_asset_bundle_data("fixtures/bundle_v2.rsrc").unwrap();
        assert_eq!(v2_bytes, fixture_v2);

        let fixture_v3 =
            get_ctb_asset_bundle_data("fixtures/bundle_v3.rsrc").unwrap();
        assert_eq!(v3_bytes, fixture_v3);

        let fixture_v4 =
            get_ctb_asset_bundle_data("fixtures/bundle_v4.rsrc").unwrap();
        assert_eq!(v4_bytes, fixture_v4);

        Ok(())
    }
}
