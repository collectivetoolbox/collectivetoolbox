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
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::ffi::OsString;
use std::fs;
use std::ops::Range;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub const RESOURCE_BUNDLE_MAGIC: &[u8; 8] = b"CTBRSRC\0";
pub const RESOURCE_BUNDLE_MAGIC_LEN: usize = 8;
const ENTRY_COUNT_OFFSET: usize = 12;
const UUID_OFFSET: usize = 16;
const SHA256_OFFSET: usize = 32;
const TIMESTAMP_OFFSET: usize = 64;
pub const RESOURCE_BUNDLE_VERSION_V1: u32 = 1;
pub const RESOURCE_BUNDLE_VERSION_V2: u32 = 2;
pub const RESOURCE_BUNDLE_VERSION: u32 = 3;
pub const RESOURCE_BUNDLE_UUID_SIZE: usize = 16;
pub const RESOURCE_BUNDLE_SHA256_SIZE: usize = 32;
pub const RESOURCE_BUNDLE_TIMESTAMP_SIZE: usize = 8;
pub const RESOURCE_ENTRY_SIZE: usize = 32;
pub const RESOURCE_BUNDLE_V1_HEADER_SIZE: usize = 16;
pub const RESOURCE_BUNDLE_V2_HEADER_SIZE: usize = 32;
pub const RESOURCE_BUNDLE_HEADER_SIZE: usize = 72;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetBundleSourceEntry {
    pub path: String,
    pub contents: Vec<u8>,
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

pub fn build_asset_bundle(
    entries: &[AssetBundleSourceEntry],
) -> Result<(Vec<u8>, AssetBundleHeader)> {
    let entries = normalized_source_entries(entries)?;
    let content_sha256 = hash_normalized_entries(&entries)?;
    let bundle_uuid = Uuid::new_v4();
    let created_at_unix_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("System clock is before UNIX_EPOCH")?
        .as_secs();

    let entry_count = u32::try_from(entries.len())
        .context("Too many resource bundle entries")?;
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

    let paths_start = u64::try_from(RESOURCE_BUNDLE_HEADER_SIZE)
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
        append_u32(&mut index, 0);
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

    let total_capacity = RESOURCE_BUNDLE_HEADER_SIZE
        .checked_add(index.len())
        .and_then(|size| size.checked_add(path_bytes.len()))
        .and_then(|size| size.checked_add(data_bytes.len()))
        .context("Resource bundle too large")?;
    let header = AssetBundleHeader {
        version: RESOURCE_BUNDLE_VERSION,
        entry_count,
        bundle_uuid,
        content_sha256,
        created_at_unix_secs,
    };

    let mut bundle = Vec::with_capacity(total_capacity);
    bundle.extend_from_slice(RESOURCE_BUNDLE_MAGIC);
    append_u32(&mut bundle, header.version);
    append_u32(&mut bundle, header.entry_count);
    bundle.extend_from_slice(header.bundle_uuid.as_bytes());
    bundle.extend_from_slice(&header.content_sha256);
    append_u64(&mut bundle, header.created_at_unix_secs);
    bundle.extend_from_slice(&index);
    bundle.extend_from_slice(&path_bytes);
    bundle.extend_from_slice(&data_bytes);

    Ok((bundle, header))
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
        let _flags = read_u32(
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
        RESOURCE_BUNDLE_VERSION => ParsedAssetBundleHeader {
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
        },
        _ => bail!("Unsupported resource bundle version {version}"),
    };

    Ok(parsed_header)
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
        let data =
            bundle_bytes.get(entry.data_range.clone()).ok_or_else(|| {
                anyhow::anyhow!("Invalid data range for {}", entry.path)
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
    if header.version >= RESOURCE_BUNDLE_VERSION {
        return Some(format_sha256_hex(&header.content_sha256));
    }
    None
}

fn created_at_metadata_value(header: &AssetBundleHeader) -> Option<u64> {
    if header.version >= RESOURCE_BUNDLE_VERSION {
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
