// SPDX-License-Identifier for parts derived from mac_alias: MIT
// Copyright (c) 2014 Alastair Houghton
// Copyright (c) 2022 Russell Keith-Magee
// From https://github.com/dmgbuild/mac_alias

// SPDX-License-Identifier for parts derived from Mac-Alias: Artistic-2.0
// Author: "Arne Johannessen <ajnn@cpan.org>"
// From https://www.cpan.org/authors/id/A/AJ/AJNN/Mac-Alias-1.01.tar.gz

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use anyhow::anyhow;

pub mod bookmark;
mod shared;

#[cfg(test)]
use include_dir::{Dir, include_dir};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use shared::{
    build_posix_path_from_components, carbon_path_to_pathbuf,
    normalize_path_string, read_fixed_bytes, read_i32_le, read_u32_le,
    resolve_posix_target,
};

#[cfg(test)]
static ALIAS_DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data");

// Fixtures available: data/fixtures/{folder,removable,root}.alias
#[cfg(test)]
pub(crate) fn get_alias_data(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&ALIAS_DATA_DIR, key)
}

const ALIAS_FILE_MAGIC: &[u8; 16] = b"book\0\0\0\0mark\0\0\0\0";
const ALIAS_FILE_HEADER_SIZE: usize = 56;
const ALIAS_TOC_CHUNK_TYPE: i32 = -2;

const ALIAS_ITEM_TARGET_PATH_COMPONENTS: u32 = 0x1004;
const ALIAS_ITEM_TARGET_URL: u32 = 0x1003;
const ALIAS_ITEM_TARGET_FILENAME: u32 = 0x1020;
const ALIAS_ITEM_DISPLAY_NAME: u32 = 0xf017;
const ALIAS_ITEM_ALIAS_DATA: u32 = 0xfe00;

const ALIAS_DATA_TYPE_STRING: i32 = 0x0101;
const ALIAS_DATA_TYPE_DATA: i32 = 0x0201;
const ALIAS_DATA_TYPE_ARRAY: i32 = 0x0601;
const ALIAS_DATA_TYPE_URL: i32 = 0x0901;

const ALIAS_VERSION_2: u16 = 2;
const ALIAS_VERSION_3: u16 = 3;

const ALIAS_KIND_FILE: u16 = 0;
const ALIAS_KIND_FOLDER: u16 = 1;

const ALIAS_FIXED_DISK: u16 = 0;

const ALIAS_TAG_END: i16 = -1;
const ALIAS_TAG_CARBON_FOLDER_NAME: i16 = 0;
const ALIAS_TAG_CNID_PATH: i16 = 1;
const ALIAS_TAG_CARBON_PATH: i16 = 2;
const ALIAS_TAG_APPLESHARE_ZONE: i16 = 3;
const ALIAS_TAG_APPLESHARE_SERVER: i16 = 4;
const ALIAS_TAG_APPLESHARE_USERNAME: i16 = 5;
const ALIAS_TAG_DRIVER_NAME: i16 = 6;
const ALIAS_TAG_NETWORK_MOUNT_INFO: i16 = 9;
const ALIAS_TAG_DIALUP_INFO: i16 = 10;
const ALIAS_TAG_UNICODE_FILENAME: i16 = 14;
const ALIAS_TAG_UNICODE_VOLUME_NAME: i16 = 15;
const ALIAS_TAG_HIGH_RES_VOLUME_DATE: i16 = 16;
const ALIAS_TAG_HIGH_RES_CREATION_DATE: i16 = 17;
const ALIAS_TAG_POSIX_PATH: i16 = 18;
const ALIAS_TAG_POSIX_PATH_TO_MOUNTPOINT: i16 = 19;
const ALIAS_TAG_RECURSIVE_ALIAS_DISK_IMAGE: i16 = 20;
const ALIAS_TAG_USER_HOME_PREFIX_LEN: i16 = 21;

/// Creates a basic Mac alias file pointing at a target.
pub fn create_simple_alias<P: AsRef<Path>>(
    target_path: P,
    name: Option<&str>,
) -> Result<Vec<u8>> {
    let target_path = target_path.as_ref();
    let path_string = normalize_path_string(target_path)?;
    // Reason for fallback: when display name is omitted and target_path has no filename component (e.g. root directory "/"), use full normalized path string as display name.
    let display_name = name
        .map(ToString::to_string)
        .or_else(|| {
            target_path
                .file_name()
                .map(|value| value.to_string_lossy().to_string())
        })
        .unwrap_or_else(|| path_string.clone());

    let alias_record = AliasRecord::new_simple(&path_string, name)?;
    let alias_record_bytes = alias_record.to_bytes()?;
    build_alias_file(&path_string, &display_name, &alias_record_bytes)
}

/// Reads the target path from a Mac alias file or alias record.
pub fn read_path_from_alias<P: AsRef<Path>>(alias_path: P) -> Result<PathBuf> {
    let bytes = std::fs::read(alias_path.as_ref()).with_context(|| {
        format!(
            "Failed to read alias file {}",
            alias_path.as_ref().display()
        )
    })?;
    read_path_from_alias_bytes(&bytes)
}

pub fn read_path_from_alias_bytes(bytes: &[u8]) -> Result<PathBuf> {
    if is_alias_file(bytes) {
        return read_path_from_alias_file(bytes);
    }

    let record = AliasRecord::from_bytes(bytes)
        .context("Alias data did not parse as an alias record")?;
    read_path_from_alias_record(&record)
}

fn read_path_from_alias_record(record: &AliasRecord) -> Result<PathBuf> {
    record
        .target
        .posix_path
        .as_ref()
        .map(|value| {
            resolve_posix_target(value, record.volume.posix_path.as_deref())
        })
        .or_else(|| {
            record
                .target
                .carbon_path
                .as_ref()
                .map(|value| carbon_path_to_pathbuf(value.as_slice()))
        })
        .or_else(|| {
            record.target.filename.as_ref().map(|name| {
                resolve_posix_target(name, record.volume.posix_path.as_deref())
            })
        })
        .context("Alias record did not include a path")
}

fn is_alias_file(bytes: &[u8]) -> bool {
    bytes.len() >= ALIAS_FILE_HEADER_SIZE && bytes.starts_with(ALIAS_FILE_MAGIC)
}

#[derive(Debug, Clone)]
struct AliasFileHeader {
    data_start: usize,
    data_length: usize,
}

fn parse_alias_file_header(bytes: &[u8]) -> Result<AliasFileHeader> {
    if !is_alias_file(bytes) {
        bail!("Not an alias file");
    }

    let data_start = read_u32_le(bytes, 16)?;
    let data_start_usize =
        usize::try_from(data_start).context("Alias data start overflow")?;
    if data_start_usize < ALIAS_FILE_HEADER_SIZE {
        bail!("Alias data start offset too small");
    }

    let data_length = read_u32_le(bytes, 24)?;
    let data_length_usize =
        usize::try_from(data_length).context("Alias data length overflow")?;
    let data_end = data_start_usize
        .checked_add(data_length_usize)
        .context("Alias data end overflow")?;
    if data_end > bytes.len() {
        bail!("Alias data section truncated");
    }

    Ok(AliasFileHeader {
        data_start: data_start_usize,
        data_length: data_length_usize,
    })
}

fn read_path_from_alias_file(bytes: &[u8]) -> Result<PathBuf> {
    let header = parse_alias_file_header(bytes)?;
    let toc_entries = parse_alias_toc_entries(bytes, &header)?;

    if let Some(offset) = toc_entries.get(&ALIAS_ITEM_TARGET_PATH_COMPONENTS) {
        let components = read_alias_path_components(bytes, &header, *offset)?;
        return Ok(build_posix_path_from_components(&components));
    }

    if let Some(offset) = toc_entries.get(&ALIAS_ITEM_TARGET_URL) {
        if let Some(path) = read_alias_url_path(bytes, &header, *offset)? {
            return Ok(path);
        }
    }

    if let Some(offset) = toc_entries.get(&ALIAS_ITEM_ALIAS_DATA) {
        let record_chunk = read_alias_chunk(bytes, &header, *offset)?;
        if record_chunk.chunk_type == ALIAS_DATA_TYPE_DATA {
            let record = AliasRecord::from_bytes(&record_chunk.data)
                .context("Alias data chunk did not parse as an alias record")?;
            return read_path_from_alias_record(&record);
        }
    }

    bail!("Alias file did not contain a target path")
}

fn parse_alias_toc_entries(
    bytes: &[u8],
    header: &AliasFileHeader,
) -> Result<BTreeMap<u32, u32>> {
    let mut entries = BTreeMap::new();
    let mut next_offset = read_u32_le(bytes, header.data_start)?;

    while next_offset != 0 {
        let toc_chunk = read_alias_chunk(bytes, header, next_offset)?;
        if toc_chunk.chunk_type != ALIAS_TOC_CHUNK_TYPE {
            bail!("Alias TOC chunk had invalid type");
        }
        if toc_chunk.data.len() < 12 {
            bail!("Alias TOC chunk too short");
        }

        let next = read_u32_le(&toc_chunk.data, 4)?;
        let count = read_u32_le(&toc_chunk.data, 8)?;
        let count_usize =
            usize::try_from(count).context("Alias TOC count overflow")?;
        let items_size = count_usize
            .checked_mul(12)
            .context("Alias TOC size overflow")?;
        let items_end = 12usize
            .checked_add(items_size)
            .context("Alias TOC size overflow")?;
        if items_end > toc_chunk.data.len() {
            bail!("Alias TOC data truncated");
        }

        let mut entry_offset = 12usize;
        for _ in 0..count_usize {
            let item_type = read_u32_le(&toc_chunk.data, entry_offset)?;
            let item_offset =
                read_u32_le(&toc_chunk.data, entry_offset.saturating_add(4))?;
            entries.insert(item_type, item_offset);
            entry_offset = entry_offset
                .checked_add(12)
                .context("Alias TOC entry overflow")?;
        }

        next_offset = next;
    }

    Ok(entries)
}

#[derive(Debug)]
struct AliasChunk {
    chunk_type: i32,
    data: Vec<u8>,
}

fn read_alias_chunk(
    bytes: &[u8],
    header: &AliasFileHeader,
    offset: u32,
) -> Result<AliasChunk> {
    let offset_usize =
        usize::try_from(offset).context("Alias chunk offset overflow")?;
    let chunk_start = header
        .data_start
        .checked_add(offset_usize)
        .context("Alias chunk offset overflow")?;
    let data_section_end = header
        .data_start
        .checked_add(header.data_length)
        .context("Alias data section overflow")?;
    if chunk_start.saturating_add(8) > data_section_end {
        bail!("Alias chunk header out of bounds");
    }

    let length = read_u32_le(bytes, chunk_start)?;
    let chunk_type = read_i32_le(bytes, chunk_start.saturating_add(4))?;
    let length_usize =
        usize::try_from(length).context("Alias chunk size overflow")?;
    let data_start = chunk_start
        .checked_add(8)
        .context("Alias chunk data overflow")?;
    let data_end = data_start
        .checked_add(length_usize)
        .context("Alias chunk data overflow")?;
    if data_end > data_section_end {
        bail!("Alias chunk data out of bounds");
    }
    let data = bytes
        .get(data_start..data_end)
        .context("Alias chunk data out of bounds")?
        .to_vec();

    Ok(AliasChunk { chunk_type, data })
}

fn read_alias_path_components(
    bytes: &[u8],
    header: &AliasFileHeader,
    offset: u32,
) -> Result<Vec<String>> {
    let chunk = read_alias_chunk(bytes, header, offset)?;
    if chunk.chunk_type != ALIAS_DATA_TYPE_ARRAY {
        bail!("Alias path components were not stored as an array");
    }

    if chunk.data.len() % 4 != 0 {
        bail!("Alias path component array had invalid length");
    }

    let mut components = Vec::new();
    let mut offset_index = 0usize;
    while offset_index < chunk.data.len() {
        let end = offset_index
            .checked_add(4)
            .context("Alias path component offset overflow")?;
        let slice = chunk
            .data
            .get(offset_index..end)
            .context("Alias path component offset out of range")?;
        let arr: [u8; 4] =
            slice.try_into().context("invalid offset slice size")?;
        let offset_value = u32::from_le_bytes(arr);
        let component_chunk = read_alias_chunk(bytes, header, offset_value)?;
        if component_chunk.chunk_type != ALIAS_DATA_TYPE_STRING {
            bail!("Alias path component was not a string");
        }
        let component = String::from_utf8(component_chunk.data)
            .context("Alias path component was not UTF-8")?;
        components.push(component);
        offset_index = end;
    }

    Ok(components)
}

fn read_alias_url_path(
    bytes: &[u8],
    header: &AliasFileHeader,
    offset: u32,
) -> Result<Option<PathBuf>> {
    let chunk = read_alias_chunk(bytes, header, offset)?;
    if chunk.chunk_type != ALIAS_DATA_TYPE_URL
        && chunk.chunk_type != ALIAS_DATA_TYPE_STRING
    {
        return Ok(None);
    }

    let url =
        String::from_utf8(chunk.data).context("Alias URL was not UTF-8")?;
    if let Some(path) = url.strip_prefix("file://") {
        let path = path.trim_start_matches('/');
        return Ok(Some(PathBuf::from("/").join(path)));
    }

    Ok(None)
}

fn build_alias_file(
    target_path: &str,
    display_name: &str,
    alias_record: &[u8],
) -> Result<Vec<u8>> {
    let components = target_path
        .trim_start_matches('/')
        .split('/')
        .filter(|component| !component.is_empty())
        .map(ToString::to_string)
        .collect::<Vec<_>>();

    // Reason for fallback: when normalized path components vector is empty (e.g. root directory "/"), file_name defaults to display_name string.
    let file_name = components
        .last()
        .cloned()
        .unwrap_or_else(|| display_name.to_string());
    let url = build_file_url(target_path);

    let mut data_section = Vec::new();
    data_section.extend_from_slice(&0u32.to_le_bytes());

    let mut component_offsets = Vec::new();
    for component in &components {
        let offset = append_alias_chunk(
            &mut data_section,
            ALIAS_DATA_TYPE_STRING,
            component.as_bytes(),
        )?;
        component_offsets.push(offset);
    }

    let file_name_offset = if file_name.is_empty() {
        None
    } else {
        Some(append_alias_chunk(
            &mut data_section,
            ALIAS_DATA_TYPE_STRING,
            file_name.as_bytes(),
        )?)
    };

    let display_name_offset = if display_name.is_empty() {
        None
    } else {
        Some(append_alias_chunk(
            &mut data_section,
            ALIAS_DATA_TYPE_STRING,
            display_name.as_bytes(),
        )?)
    };

    let url_offset = if url.is_empty() {
        None
    } else {
        Some(append_alias_chunk(
            &mut data_section,
            ALIAS_DATA_TYPE_URL,
            url.as_bytes(),
        )?)
    };

    let alias_data_offset = append_alias_chunk(
        &mut data_section,
        ALIAS_DATA_TYPE_DATA,
        alias_record,
    )?;

    let mut path_array_data = Vec::new();
    for offset in &component_offsets {
        path_array_data.extend_from_slice(&offset.to_le_bytes());
    }
    let path_array_offset = append_alias_chunk(
        &mut data_section,
        ALIAS_DATA_TYPE_ARRAY,
        &path_array_data,
    )?;

    let mut toc_entries = Vec::new();
    toc_entries.push((ALIAS_ITEM_TARGET_PATH_COMPONENTS, path_array_offset));
    if let Some(offset) = url_offset {
        toc_entries.push((ALIAS_ITEM_TARGET_URL, offset));
    }
    if let Some(offset) = file_name_offset {
        toc_entries.push((ALIAS_ITEM_TARGET_FILENAME, offset));
    }
    if let Some(offset) = display_name_offset {
        toc_entries.push((ALIAS_ITEM_DISPLAY_NAME, offset));
    }
    toc_entries.push((ALIAS_ITEM_ALIAS_DATA, alias_data_offset));

    let toc_data = build_alias_toc_data(&toc_entries)?;
    let toc_offset =
        append_alias_chunk(&mut data_section, ALIAS_TOC_CHUNK_TYPE, &toc_data)?;
    data_section
        .get_mut(0..4)
        .context("invalid alias data section size")?
        .copy_from_slice(&toc_offset.to_le_bytes());

    let data_length = u32::try_from(data_section.len())
        .context("Alias data section too large")?;
    let data_offset = u32::try_from(ALIAS_FILE_HEADER_SIZE)
        .context("Alias header size overflow")?;

    let mut bytes = Vec::new();
    bytes.extend_from_slice(ALIAS_FILE_MAGIC);
    bytes.extend_from_slice(&data_offset.to_le_bytes());
    bytes.extend_from_slice(&data_offset.to_le_bytes());
    bytes.extend_from_slice(&data_length.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&0u64.to_le_bytes());
    bytes.extend_from_slice(&0u64.to_le_bytes());
    bytes.extend_from_slice(&data_section);

    Ok(bytes)
}

fn append_alias_chunk(
    data_section: &mut Vec<u8>,
    chunk_type: i32,
    data: &[u8],
) -> Result<u32> {
    let offset = u32::try_from(data_section.len())
        .context("Alias chunk offset overflow")?;
    let size =
        u32::try_from(data.len()).context("Alias chunk size overflow")?;
    data_section.extend_from_slice(&size.to_le_bytes());
    data_section.extend_from_slice(&chunk_type.to_le_bytes());
    data_section.extend_from_slice(data);
    let padding = (4usize.saturating_sub(data.len() % 4)) % 4;
    let new_len = data_section
        .len()
        .checked_add(padding)
        .context("Alias chunk padding overflow")?;
    data_section.resize(new_len, 0);
    Ok(offset)
}

fn build_alias_toc_data(entries: &[(u32, u32)]) -> Result<Vec<u8>> {
    let count =
        u32::try_from(entries.len()).context("Alias TOC count overflow")?;
    let mut data = Vec::new();
    data.extend_from_slice(&1u32.to_le_bytes());
    data.extend_from_slice(&0u32.to_le_bytes());
    data.extend_from_slice(&count.to_le_bytes());
    for (item_type, offset) in entries {
        data.extend_from_slice(&item_type.to_le_bytes());
        data.extend_from_slice(&offset.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());
    }
    Ok(data)
}

fn build_file_url(target_path: &str) -> String {
    let trimmed = target_path.trim_start_matches('/');
    if trimmed.is_empty() {
        String::from("file:///")
    } else {
        format!("file:///{trimmed}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AliasRecord {
    pub app_info: [u8; 4],
    pub version: u16,
    pub volume: VolumeInfo,
    pub target: TargetInfo,
    pub extra: Vec<AliasExtra>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VolumeInfo {
    pub name: String,
    pub creation_date: MacTimestamp,
    pub fs_type: [u8; 4],
    pub disk_type: u16,
    pub attribute_flags: u32,
    pub fs_id: [u8; 2],
    pub appleshare_info: Option<AppleShareInfo>,
    pub driver_name: Option<Vec<u8>>,
    pub posix_path: Option<String>,
    pub disk_image_alias: Option<Box<AliasRecord>>,
    pub dialup_info: Option<Vec<u8>>,
    pub network_mount_info: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppleShareInfo {
    pub zone: Option<Vec<u8>>,
    pub server: Option<Vec<u8>>,
    pub user: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetInfo {
    pub kind: u16,
    pub filename: Option<String>,
    pub folder_cnid: u32,
    pub cnid: u32,
    pub creation_date: MacTimestamp,
    pub creator_code: [u8; 4],
    pub type_code: [u8; 4],
    pub levels_from: i16,
    pub levels_to: i16,
    pub folder_name: Option<String>,
    pub cnid_path: Option<Vec<u32>>,
    pub carbon_path: Option<Vec<u8>>,
    pub posix_path: Option<String>,
    pub user_home_prefix_len: Option<i16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AliasExtra {
    pub tag: i16,
    pub value: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacTimestamp {
    pub seconds: u64,
    pub fraction: u16,
}

impl MacTimestamp {
    fn from_seconds(seconds: u64) -> Self {
        Self {
            seconds,
            fraction: 0,
        }
    }

    fn from_fixed_point(value: u64) -> Result<Self> {
        let seconds = value / 65536;
        let fraction = u16::try_from(value % 65536)
            .context("Fixed-point timestamp overflow")?;
        Ok(Self { seconds, fraction })
    }

    fn to_fixed_point(self) -> u64 {
        (self.seconds.saturating_mul(65536))
            .saturating_add(u64::from(self.fraction))
    }
}

impl AliasRecord {
    pub fn new_simple(target_path: &str, name: Option<&str>) -> Result<Self> {
        let filename = name.map(ToString::to_string).or_else(|| {
            Path::new(target_path)
                .file_name()
                .map(|value| value.to_string_lossy().to_string())
        });
        let folder_name = Path::new(target_path)
            .parent()
            .and_then(|parent| parent.file_name())
            .map(|value| value.to_string_lossy().to_string());
        let kind = if Path::new(target_path).extension().is_none() {
            ALIAS_KIND_FOLDER
        } else {
            ALIAS_KIND_FILE
        };

        Ok(Self {
            app_info: [0, 0, 0, 0],
            version: ALIAS_VERSION_2,
            volume: VolumeInfo {
                name: String::from("/"),
                creation_date: MacTimestamp::from_seconds(0),
                fs_type: [b'H', b'+', 0, 0],
                disk_type: ALIAS_FIXED_DISK,
                attribute_flags: 0,
                fs_id: [0, 0],
                appleshare_info: None,
                driver_name: None,
                posix_path: Some(String::from("/")),
                disk_image_alias: None,
                dialup_info: None,
                network_mount_info: None,
            },
            target: TargetInfo {
                kind,
                filename,
                folder_cnid: 0,
                cnid: 0,
                creation_date: MacTimestamp::from_seconds(0),
                creator_code: [0, 0, 0, 0],
                type_code: [0, 0, 0, 0],
                levels_from: -1,
                levels_to: -1,
                folder_name,
                cnid_path: None,
                carbon_path: None,
                posix_path: Some(target_path.to_string()),
                user_home_prefix_len: None,
            },
            extra: Vec::new(),
        })
    }

    #[allow(
        clippy::too_many_lines,
        reason = "from_bytes is a complex parsing routine for Mac OS alias records"
    )]
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 8 {
            bail!("Alias record too short");
        }

        let app_info: [u8; 4] = bytes
            .get(0..4)
            .context("Alias record too short for app info")?
            .try_into()
            .context("Invalid app info size")?;
        let record_size = read_u16_be(bytes, 4)?;
        let version = read_u16_be(bytes, 6)?;
        if record_size < 150 {
            bail!("Alias record size too small");
        }
        let record_size_usize = usize::from(record_size);
        if bytes.len() < record_size_usize {
            bail!("Alias record truncated");
        }
        if version != ALIAS_VERSION_2 && version != ALIAS_VERSION_3 {
            bail!("Unsupported alias record version");
        }

        let mut offset = 8usize;
        let (volume, target) = if version == ALIAS_VERSION_2 {
            let kind = read_u16_be(bytes, offset)?;
            offset = offset.checked_add(2).context("Alias header overflow")?;
            let volname = read_pascal_string(bytes, offset, 28)?;
            offset = offset.checked_add(28).context("Alias header overflow")?;
            let volume_date = read_u32_be(bytes, offset)?;
            offset = offset.checked_add(4).context("Alias header overflow")?;
            let fs_type = read_fixed_bytes(bytes, offset, 2)?;
            offset = offset.checked_add(2).context("Alias header overflow")?;
            let disk_type = read_u16_be(bytes, offset)?;
            offset = offset.checked_add(2).context("Alias header overflow")?;
            let folder_cnid = read_u32_be(bytes, offset)?;
            offset = offset.checked_add(4).context("Alias header overflow")?;
            let filename = read_pascal_string(bytes, offset, 64)?;
            offset = offset.checked_add(64).context("Alias header overflow")?;
            let cnid = read_u32_be(bytes, offset)?;
            offset = offset.checked_add(4).context("Alias header overflow")?;
            let creation_date = read_u32_be(bytes, offset)?;
            offset = offset.checked_add(4).context("Alias header overflow")?;
            let creator_code = read_fixed_bytes(bytes, offset, 4)?;
            offset = offset.checked_add(4).context("Alias header overflow")?;
            let type_code = read_fixed_bytes(bytes, offset, 4)?;
            offset = offset.checked_add(4).context("Alias header overflow")?;
            let levels_from = read_i16_be(bytes, offset)?;
            offset = offset.checked_add(2).context("Alias header overflow")?;
            let levels_to = read_i16_be(bytes, offset)?;
            offset = offset.checked_add(2).context("Alias header overflow")?;
            let attribute_flags = read_u32_be(bytes, offset)?;
            offset = offset.checked_add(4).context("Alias header overflow")?;
            let fs_id = read_fixed_bytes(bytes, offset, 2)?;
            offset = offset.checked_add(2).context("Alias header overflow")?;
            offset = offset.checked_add(10).context("Alias header overflow")?;

            let &[fst0, fst1] = fs_type.as_slice() else {
                bail!("invalid fs_type size");
            };
            let &[fid0, fid1] = fs_id.as_slice() else {
                bail!("invalid fs_id size");
            };
            let &[cc0, cc1, cc2, cc3] = creator_code.as_slice() else {
                bail!("invalid creator_code size");
            };
            let &[tc0, tc1, tc2, tc3] = type_code.as_slice() else {
                bail!("invalid type_code size");
            };

            (
                VolumeInfo {
                    name: volname,
                    creation_date: MacTimestamp::from_seconds(u64::from(
                        volume_date,
                    )),
                    fs_type: [fst0, fst1, 0, 0],
                    disk_type,
                    attribute_flags,
                    fs_id: [fid0, fid1],
                    appleshare_info: None,
                    driver_name: None,
                    posix_path: None,
                    disk_image_alias: None,
                    dialup_info: None,
                    network_mount_info: None,
                },
                TargetInfo {
                    kind,
                    filename: Some(filename),
                    folder_cnid,
                    cnid,
                    creation_date: MacTimestamp::from_seconds(u64::from(
                        creation_date,
                    )),
                    creator_code: [cc0, cc1, cc2, cc3],
                    type_code: [tc0, tc1, tc2, tc3],
                    levels_from,
                    levels_to,
                    folder_name: None,
                    cnid_path: None,
                    carbon_path: None,
                    posix_path: None,
                    user_home_prefix_len: None,
                },
            )
        } else {
            let kind = read_u16_be(bytes, offset)?;
            offset = offset.checked_add(2).context("Alias header overflow")?;
            let volume_date = read_u64_be(bytes, offset)?;
            offset = offset.checked_add(8).context("Alias header overflow")?;
            let fs_type = read_fixed_bytes(bytes, offset, 4)?;
            offset = offset.checked_add(4).context("Alias header overflow")?;
            let disk_type = read_u16_be(bytes, offset)?;
            offset = offset.checked_add(2).context("Alias header overflow")?;
            let folder_cnid = read_u32_be(bytes, offset)?;
            offset = offset.checked_add(4).context("Alias header overflow")?;
            let cnid = read_u32_be(bytes, offset)?;
            offset = offset.checked_add(4).context("Alias header overflow")?;
            let creation_date = read_u64_be(bytes, offset)?;
            offset = offset.checked_add(8).context("Alias header overflow")?;
            let attribute_flags = read_u32_be(bytes, offset)?;
            offset = offset.checked_add(4).context("Alias header overflow")?;
            offset = offset.checked_add(14).context("Alias header overflow")?;

            let &[fst0, fst1, fst2, fst3] = fs_type.as_slice() else {
                bail!("invalid fs_type size");
            };

            (
                VolumeInfo {
                    name: String::new(),
                    creation_date: MacTimestamp::from_fixed_point(volume_date)?,
                    fs_type: [fst0, fst1, fst2, fst3],
                    disk_type,
                    attribute_flags,
                    fs_id: [0, 0],
                    appleshare_info: None,
                    driver_name: None,
                    posix_path: None,
                    disk_image_alias: None,
                    dialup_info: None,
                    network_mount_info: None,
                },
                TargetInfo {
                    kind,
                    filename: None,
                    folder_cnid,
                    cnid,
                    creation_date: MacTimestamp::from_fixed_point(
                        creation_date,
                    )?,
                    creator_code: [0, 0, 0, 0],
                    type_code: [0, 0, 0, 0],
                    levels_from: -1,
                    levels_to: -1,
                    folder_name: None,
                    cnid_path: None,
                    carbon_path: None,
                    posix_path: None,
                    user_home_prefix_len: None,
                },
            )
        };

        let mut record = Self {
            app_info,
            version,
            volume,
            target,
            extra: Vec::new(),
        };

        while offset.saturating_add(4) <= record_size_usize {
            let tag = read_i16_be(bytes, offset)?;
            offset = offset.checked_add(2).context("Alias tag overflow")?;
            if tag == ALIAS_TAG_END {
                break;
            }
            let length = read_u16_be(bytes, offset)?;
            offset = offset.checked_add(2).context("Alias tag overflow")?;
            let length_usize = usize::from(length);
            let data_end = offset
                .checked_add(length_usize)
                .context("Alias tag overflow")?;
            if data_end > record_size_usize {
                bail!("Alias tag exceeded record size");
            }
            let value = bytes
                .get(offset..data_end)
                .context("Alias tag out of bounds")?
                .to_vec();
            offset = data_end;
            if length % 2 != 0 {
                offset = offset
                    .checked_add(1)
                    .context("Alias tag padding overflow")?;
            }

            match tag {
                ALIAS_TAG_CARBON_FOLDER_NAME => {
                    record.target.folder_name =
                        Some(String::from_utf8_lossy(&value).to_string());
                }
                ALIAS_TAG_CNID_PATH => {
                    let mut cnids = Vec::new();
                    let mut value_offset = 0usize;
                    while value_offset < value.len() {
                        let end = value_offset
                            .checked_add(4)
                            .context("Alias CNID path overflow")?;
                        let slice = value
                            .get(value_offset..end)
                            .context("Alias CNID path out of bounds")?;
                        let arr: [u8; 4] = slice
                            .try_into()
                            .context("Alias CNID path chunk invalid size")?;
                        let cnid = u32::from_be_bytes(arr);
                        cnids.push(cnid);
                        value_offset = end;
                    }
                    record.target.cnid_path = Some(cnids);
                }
                ALIAS_TAG_CARBON_PATH => {
                    record.target.carbon_path = Some(value);
                }
                ALIAS_TAG_APPLESHARE_ZONE => {
                    let appleshare = record
                        .volume
                        .appleshare_info
                        .get_or_insert(AppleShareInfo {
                            zone: None,
                            server: None,
                            user: None,
                        });
                    appleshare.zone = Some(value);
                }
                ALIAS_TAG_APPLESHARE_SERVER => {
                    let appleshare = record
                        .volume
                        .appleshare_info
                        .get_or_insert(AppleShareInfo {
                            zone: None,
                            server: None,
                            user: None,
                        });
                    appleshare.server = Some(value);
                }
                ALIAS_TAG_APPLESHARE_USERNAME => {
                    let appleshare = record
                        .volume
                        .appleshare_info
                        .get_or_insert(AppleShareInfo {
                            zone: None,
                            server: None,
                            user: None,
                        });
                    appleshare.user = Some(value);
                }
                ALIAS_TAG_DRIVER_NAME => {
                    record.volume.driver_name = Some(value);
                }
                ALIAS_TAG_NETWORK_MOUNT_INFO => {
                    record.volume.network_mount_info = Some(value);
                }
                ALIAS_TAG_DIALUP_INFO => {
                    record.volume.dialup_info = Some(value);
                }
                ALIAS_TAG_UNICODE_FILENAME => {
                    record.target.filename = Some(decode_utf16_be(&value)?);
                }
                ALIAS_TAG_UNICODE_VOLUME_NAME => {
                    record.volume.name = decode_utf16_be(&value)?;
                }
                ALIAS_TAG_HIGH_RES_VOLUME_DATE => {
                    if let Ok(arr) = value.as_slice().try_into() {
                        let value = u64::from_be_bytes(arr);
                        record.volume.creation_date =
                            MacTimestamp::from_fixed_point(value)?;
                    }
                }
                ALIAS_TAG_HIGH_RES_CREATION_DATE => {
                    if let Ok(arr) = value.as_slice().try_into() {
                        let value = u64::from_be_bytes(arr);
                        record.target.creation_date =
                            MacTimestamp::from_fixed_point(value)?;
                    }
                }
                ALIAS_TAG_POSIX_PATH => {
                    record.target.posix_path =
                        Some(String::from_utf8_lossy(&value).to_string());
                }
                ALIAS_TAG_POSIX_PATH_TO_MOUNTPOINT => {
                    record.volume.posix_path =
                        Some(String::from_utf8_lossy(&value).to_string());
                }
                ALIAS_TAG_RECURSIVE_ALIAS_DISK_IMAGE => {
                    record.volume.disk_image_alias =
                        Some(Box::new(AliasRecord::from_bytes(&value)?));
                }
                ALIAS_TAG_USER_HOME_PREFIX_LEN => {
                    if let Some(slice_prefix) = value.get(0..2) {
                        if let Ok(arr) = slice_prefix.try_into() {
                            let length = i16::from_be_bytes(arr);
                            record.target.user_home_prefix_len = Some(length);
                        }
                    }
                }
                _ => record.extra.push(AliasExtra { tag, value }),
            }
        }

        Ok(record)
    }

    #[allow(
        clippy::too_many_lines,
        reason = "to_bytes is a complex serialization routine for Mac OS alias records"
    )]
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.app_info);
        bytes.extend_from_slice(&0u16.to_be_bytes());
        bytes.extend_from_slice(&self.version.to_be_bytes());

        if self.version == ALIAS_VERSION_2 {
            let volume_name = self.volume.name.replace(':', "/");
            // Reason for fallback: v2 Mac alias record binary format uses empty Pascal string (len 0) for target filename when target has no filename component (e.g. volume root).
            let filename = self
                .target
                .filename
                .clone()
                .unwrap_or_default()
                .replace(':', "/");
            bytes.extend_from_slice(&self.target.kind.to_be_bytes());
            write_pascal_string(&mut bytes, &volume_name, 28)?;
            bytes.extend_from_slice(
                &u32::try_from(self.volume.creation_date.seconds)?
                    .to_be_bytes(),
            );
            bytes.extend_from_slice(&self.volume.fs_type[0..2]);
            bytes.extend_from_slice(&self.volume.disk_type.to_be_bytes());
            bytes.extend_from_slice(&self.target.folder_cnid.to_be_bytes());
            write_pascal_string(&mut bytes, &filename, 64)?;
            bytes.extend_from_slice(&self.target.cnid.to_be_bytes());
            bytes.extend_from_slice(
                &u32::try_from(self.target.creation_date.seconds)?
                    .to_be_bytes(),
            );
            bytes.extend_from_slice(&self.target.creator_code);
            bytes.extend_from_slice(&self.target.type_code);
            bytes.extend_from_slice(&self.target.levels_from.to_be_bytes());
            bytes.extend_from_slice(&self.target.levels_to.to_be_bytes());
            bytes.extend_from_slice(&self.volume.attribute_flags.to_be_bytes());
            bytes.extend_from_slice(&self.volume.fs_id);
            bytes.extend_from_slice(&[0u8; 10]);
        } else {
            bytes.extend_from_slice(&self.target.kind.to_be_bytes());
            bytes.extend_from_slice(
                &self.volume.creation_date.to_fixed_point().to_be_bytes(),
            );
            bytes.extend_from_slice(&self.volume.fs_type);
            bytes.extend_from_slice(&self.volume.disk_type.to_be_bytes());
            bytes.extend_from_slice(&self.target.folder_cnid.to_be_bytes());
            bytes.extend_from_slice(&self.target.cnid.to_be_bytes());
            bytes.extend_from_slice(
                &self.target.creation_date.to_fixed_point().to_be_bytes(),
            );
            bytes.extend_from_slice(&self.volume.attribute_flags.to_be_bytes());
            bytes.extend_from_slice(&[0u8; 14]);
        }

        if let Some(folder_name) = &self.target.folder_name {
            let name = folder_name.replace(':', "/");
            write_tag_bytes(
                &mut bytes,
                ALIAS_TAG_CARBON_FOLDER_NAME,
                name.as_bytes(),
            )?;
        }

        write_tag_u64(
            &mut bytes,
            ALIAS_TAG_HIGH_RES_VOLUME_DATE,
            self.volume.creation_date.to_fixed_point(),
        )?;
        write_tag_u64(
            &mut bytes,
            ALIAS_TAG_HIGH_RES_CREATION_DATE,
            self.target.creation_date.to_fixed_point(),
        )?;

        if let Some(cnid_path) = &self.target.cnid_path {
            let mut data = Vec::new();
            for cnid in cnid_path {
                data.extend_from_slice(&cnid.to_be_bytes());
            }
            write_tag_bytes(&mut bytes, ALIAS_TAG_CNID_PATH, &data)?;
        }

        if let Some(carbon_path) = &self.target.carbon_path {
            write_tag_bytes(&mut bytes, ALIAS_TAG_CARBON_PATH, carbon_path)?;
        }

        if let Some(appleshare) = &self.volume.appleshare_info {
            if let Some(zone) = appleshare.zone.as_ref() {
                write_tag_bytes(&mut bytes, ALIAS_TAG_APPLESHARE_ZONE, zone)?;
            }
            if let Some(server) = appleshare.server.as_ref() {
                write_tag_bytes(
                    &mut bytes,
                    ALIAS_TAG_APPLESHARE_SERVER,
                    server,
                )?;
            }
            if let Some(user) = appleshare.user.as_ref() {
                write_tag_bytes(
                    &mut bytes,
                    ALIAS_TAG_APPLESHARE_USERNAME,
                    user,
                )?;
            }
        }

        if let Some(driver_name) = &self.volume.driver_name {
            write_tag_bytes(&mut bytes, ALIAS_TAG_DRIVER_NAME, driver_name)?;
        }

        if let Some(network_mount) = &self.volume.network_mount_info {
            write_tag_bytes(
                &mut bytes,
                ALIAS_TAG_NETWORK_MOUNT_INFO,
                network_mount,
            )?;
        }

        if let Some(dialup_info) = &self.volume.dialup_info {
            write_tag_bytes(&mut bytes, ALIAS_TAG_DIALUP_INFO, dialup_info)?;
        }

        if let Some(filename) = &self.target.filename {
            write_tag_utf16(&mut bytes, ALIAS_TAG_UNICODE_FILENAME, filename)?;
        }

        write_tag_utf16(
            &mut bytes,
            ALIAS_TAG_UNICODE_VOLUME_NAME,
            &self.volume.name,
        )?;

        if let Some(posix_path) = &self.target.posix_path {
            write_tag_bytes(
                &mut bytes,
                ALIAS_TAG_POSIX_PATH,
                posix_path.as_bytes(),
            )?;
        }

        if let Some(posix_path) = &self.volume.posix_path {
            write_tag_bytes(
                &mut bytes,
                ALIAS_TAG_POSIX_PATH_TO_MOUNTPOINT,
                posix_path.as_bytes(),
            )?;
        }

        if let Some(disk_alias) = &self.volume.disk_image_alias {
            let alias_bytes = disk_alias.to_bytes()?;
            write_tag_bytes(
                &mut bytes,
                ALIAS_TAG_RECURSIVE_ALIAS_DISK_IMAGE,
                &alias_bytes,
            )?;
        }

        if let Some(prefix_len) = self.target.user_home_prefix_len {
            let mut data = Vec::new();
            data.extend_from_slice(&prefix_len.to_be_bytes());
            write_tag_bytes(&mut bytes, ALIAS_TAG_USER_HOME_PREFIX_LEN, &data)?;
        }

        for extra in &self.extra {
            write_tag_bytes(&mut bytes, extra.tag, &extra.value)?;
        }

        bytes.extend_from_slice(&ALIAS_TAG_END.to_be_bytes());
        bytes.extend_from_slice(&0u16.to_be_bytes());

        let record_size =
            u16::try_from(bytes.len()).context("Alias record too large")?;
        bytes
            .get_mut(4..6)
            .context("invalid alias header size")?
            .copy_from_slice(&record_size.to_be_bytes());
        Ok(bytes)
    }
}

fn decode_utf16_be(bytes: &[u8]) -> Result<String> {
    if bytes.len() < 2 {
        return Ok(String::new());
    }
    let mut data = Vec::new();
    let mut offset = 2usize;
    while offset.saturating_add(1) < bytes.len() {
        let end = offset.checked_add(2).context("UTF-16 data overflow")?;
        let slice = bytes.get(offset..end).context("UTF-16 out of bounds")?;
        let arr: [u8; 2] =
            slice.try_into().context("UTF-16 invalid chunk size")?;
        let value = u16::from_be_bytes(arr);
        data.push(value);
        offset = end;
    }
    String::from_utf16(&data).context("UTF-16 string decode failed")
}

fn write_tag_bytes(buffer: &mut Vec<u8>, tag: i16, value: &[u8]) -> Result<()> {
    buffer.extend_from_slice(&tag.to_be_bytes());
    let length = u16::try_from(value.len()).context("Alias tag too large")?;
    buffer.extend_from_slice(&length.to_be_bytes());
    buffer.extend_from_slice(value);
    if length % 2 != 0 {
        buffer.push(0);
    }
    Ok(())
}

fn write_tag_u64(buffer: &mut Vec<u8>, tag: i16, value: u64) -> Result<()> {
    write_tag_bytes(buffer, tag, &value.to_be_bytes())
}

fn write_tag_utf16(buffer: &mut Vec<u8>, tag: i16, value: &str) -> Result<()> {
    let mut data = Vec::new();
    let utf16 = value.encode_utf16().collect::<Vec<_>>();
    let len = u16::try_from(utf16.len()).context("UTF-16 string too large")?;
    data.extend_from_slice(&len.to_be_bytes());
    for value in utf16 {
        data.extend_from_slice(&value.to_be_bytes());
    }
    write_tag_bytes(buffer, tag, &data)
}

fn write_pascal_string(
    buffer: &mut Vec<u8>,
    value: &str,
    size: usize,
) -> Result<()> {
    let bytes = value.as_bytes();
    let max_len = size
        .checked_sub(1)
        .context("Pascal string size too small")?;
    if bytes.len() > max_len {
        bail!("Pascal string too long");
    }
    let length = u8::try_from(bytes.len()).context("Pascal string too long")?;
    buffer.push(length);
    buffer.extend_from_slice(bytes);
    let pad_len = max_len
        .checked_sub(bytes.len())
        .context("Pascal string padding overflow")?;
    let new_len = buffer
        .len()
        .checked_add(pad_len)
        .context("Pascal string padding overflow")?;
    buffer.resize(new_len, 0);
    Ok(())
}

#[allow(
    clippy::range_plus_one,
    reason = "Pascal string length calculations naturally fit range_plus_one"
)]
fn read_pascal_string(
    bytes: &[u8],
    offset: usize,
    size: usize,
) -> Result<String> {
    let slice = read_fixed_bytes(bytes, offset, size)?;
    if slice.is_empty() {
        return Ok(String::new());
    }
    let len = usize::from(*slice.get(0).context("Pascal string empty")?);
    if len > slice.len().saturating_sub(1) {
        bail!("Pascal string length invalid");
    }
    let value = slice
        .get(1..1_usize.saturating_add(len))
        .context("Pascal string out of bounds")?;
    Ok(String::from_utf8_lossy(value).to_string())
}

fn read_u16_be(bytes: &[u8], offset: usize) -> Result<u16> {
    let slice = read_fixed_bytes(bytes, offset, 2)?;
    let arr: [u8; 2] = slice
        .try_into()
        .map_err(|_| anyhow!("invalid u16 slice size"))?;
    Ok(u16::from_be_bytes(arr))
}

fn read_i16_be(bytes: &[u8], offset: usize) -> Result<i16> {
    let slice = read_fixed_bytes(bytes, offset, 2)?;
    let arr: [u8; 2] = slice
        .try_into()
        .map_err(|_| anyhow!("invalid i16 slice size"))?;
    Ok(i16::from_be_bytes(arr))
}

fn read_u32_be(bytes: &[u8], offset: usize) -> Result<u32> {
    let slice = read_fixed_bytes(bytes, offset, 4)?;
    let arr: [u8; 4] = slice
        .try_into()
        .map_err(|_| anyhow!("invalid u32 slice size"))?;
    Ok(u32::from_be_bytes(arr))
}

fn read_u64_be(bytes: &[u8], offset: usize) -> Result<u64> {
    let slice = read_fixed_bytes(bytes, offset, 8)?;
    let arr: [u8; 8] = slice
        .try_into()
        .map_err(|_| anyhow!("invalid u64 slice size"))?;
    Ok(u64::from_be_bytes(arr))
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
    use std::fs;
    use std::path::PathBuf;

    use tempfile::TempDir;

    use super::*;

    #[crate::ctb_test]
    fn test_create_alias_round_trip() {
        let dir = TempDir::new().unwrap();
        let alias_bytes = create_simple_alias(
            "/tmp/ctb/alias-target.txt",
            Some("alias-target"),
        )
        .unwrap();
        let path = dir.path().join("alias");
        fs::write(&path, alias_bytes).unwrap();
        let target = read_path_from_alias(&path).unwrap();
        assert_eq!(target, PathBuf::from("/tmp/ctb/alias-target.txt"));
    }

    #[crate::ctb_test]
    fn test_read_reference_alias_files() {
        let dir = TempDir::new().unwrap();
        let fixtures = [
            (
                get_alias_data("fixtures/folder.alias")
                    .expect("Missing folder.alias fixture"),
                PathBuf::from("/System/Library/Perl"),
            ),
            (
                get_alias_data("fixtures/removable.alias")
                    .expect("Missing removable.alias fixture"),
                PathBuf::from("/Volumes/SANDISK/untitled"),
            ),
            (
                get_alias_data("fixtures/root.alias")
                    .expect("Missing root.alias fixture"),
                PathBuf::from("/"),
            ),
        ];

        for (index, (bytes, expected)) in fixtures.iter().enumerate() {
            let path = dir.path().join(format!("fixture-{index}.alias"));
            fs::write(&path, bytes).unwrap();
            let target = read_path_from_alias(&path).unwrap();
            assert_eq!(target, *expected);
        }
    }

    #[crate::ctb_test]
    fn test_alias_record_round_trip() {
        let record =
            AliasRecord::new_simple("/tmp/alias-record.txt", Some("alias.txt"))
                .unwrap();
        let bytes = record.to_bytes().unwrap();
        let parsed = AliasRecord::from_bytes(&bytes).unwrap();
        assert_eq!(
            parsed.target.posix_path,
            Some("/tmp/alias-record.txt".to_string())
        );
        assert_eq!(parsed.target.filename, Some("alias.txt".to_string()));
    }
}

/*

// From mac_alias:

MIT License

Copyright (c) 2014 Alastair Houghton
Copyright (c) 2022 Russell Keith-Magee

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
THE SOFTWARE.


// From Mac-Alias:

               The Artistic License 2.0

        Copyright (c) 2000-2006, The Perl Foundation.

     Everyone is permitted to copy and distribute verbatim copies
      of this license document, but changing it is not allowed.

Preamble

This license establishes the terms under which a given free software
Package may be copied, modified, distributed, and/or redistributed.
The intent is that the Copyright Holder maintains some artistic
control over the development of that Package while still keeping the
Package available as open source and free software.

You are always permitted to make arrangements wholly outside of this
license directly with the Copyright Holder of a given Package.  If the
terms of this license do not permit the full use that you propose to
make of the Package, you should contact the Copyright Holder and seek
a different licensing arrangement.

Definitions

    "Copyright Holder" means the individual(s) or organization(s)
    named in the copyright notice for the entire Package.

    "Contributor" means any party that has contributed code or other
    material to the Package, in accordance with the Copyright Holder's
    procedures.

    "You" and "your" means any person who would like to copy,
    distribute, or modify the Package.

    "Package" means the collection of files distributed by the
    Copyright Holder, and derivatives of that collection and/or of
    those files. A given Package may consist of either the Standard
    Version, or a Modified Version.

    "Distribute" means providing a copy of the Package or making it
    accessible to anyone else, or in the case of a company or
    organization, to others outside of your company or organization.

    "Distributor Fee" means any fee that you charge for Distributing
    this Package or providing support for this Package to another
    party.  It does not mean licensing fees.

    "Standard Version" refers to the Package if it has not been
    modified, or has been modified only in ways explicitly requested
    by the Copyright Holder.

    "Modified Version" means the Package, if it has been changed, and
    such changes were not explicitly requested by the Copyright
    Holder.

    "Original License" means this Artistic License as Distributed with
    the Standard Version of the Package, in its current version or as
    it may be modified by The Perl Foundation in the future.

    "Source" form means the source code, documentation source, and
    configuration files for the Package.

    "Compiled" form means the compiled bytecode, object code, binary,
    or any other form resulting from mechanical transformation or
    translation of the Source form.


Permission for Use and Modification Without Distribution

(1)  You are permitted to use the Standard Version and create and use
Modified Versions for any purpose without restriction, provided that
you do not Distribute the Modified Version.


Permissions for Redistribution of the Standard Version

(2)  You may Distribute verbatim copies of the Source form of the
Standard Version of this Package in any medium without restriction,
either gratis or for a Distributor Fee, provided that you duplicate
all of the original copyright notices and associated disclaimers.  At
your discretion, such verbatim copies may or may not include a
Compiled form of the Package.

(3)  You may apply any bug fixes, portability changes, and other
modifications made available from the Copyright Holder.  The resulting
Package will still be considered the Standard Version, and as such
will be subject to the Original License.


Distribution of Modified Versions of the Package as Source

(4)  You may Distribute your Modified Version as Source (either gratis
or for a Distributor Fee, and with or without a Compiled form of the
Modified Version) provided that you clearly document how it differs
from the Standard Version, including, but not limited to, documenting
any non-standard features, executables, or modules, and provided that
you do at least ONE of the following:

    (a)  make the Modified Version available to the Copyright Holder
    of the Standard Version, under the Original License, so that the
    Copyright Holder may include your modifications in the Standard
    Version.

    (b)  ensure that installation of your Modified Version does not
    prevent the user installing or running the Standard Version. In
    addition, the Modified Version must bear a name that is different
    from the name of the Standard Version.

    (c)  allow anyone who receives a copy of the Modified Version to
    make the Source form of the Modified Version available to others
    under

    (i)  the Original License or

    (ii)  a license that permits the licensee to freely copy,
    modify and redistribute the Modified Version using the same
    licensing terms that apply to the copy that the licensee
    received, and requires that the Source form of the Modified
    Version, and of any works derived from it, be made freely
    available in that license fees are prohibited but Distributor
    Fees are allowed.


Distribution of Compiled Forms of the Standard Version
or Modified Versions without the Source

(5)  You may Distribute Compiled forms of the Standard Version without
the Source, provided that you include complete instructions on how to
get the Source of the Standard Version.  Such instructions must be
valid at the time of your distribution.  If these instructions, at any
time while you are carrying out such distribution, become invalid, you
must provide new instructions on demand or cease further distribution.
If you provide valid instructions or cease distribution within thirty
days after you become aware that the instructions are invalid, then
you do not forfeit any of your rights under this license.

(6)  You may Distribute a Modified Version in Compiled form without
the Source, provided that you comply with Section 4 with respect to
the Source of the Modified Version.


Aggregating or Linking the Package

(7)  You may aggregate the Package (either the Standard Version or
Modified Version) with other packages and Distribute the resulting
aggregation provided that you do not charge a licensing fee for the
Package.  Distributor Fees are permitted, and licensing fees for other
components in the aggregation are permitted. The terms of this license
apply to the use and Distribution of the Standard or Modified Versions
as included in the aggregation.

(8) You are permitted to link Modified and Standard Versions with
other works, to embed the Package in a larger work of your own, or to
build stand-alone binary or bytecode versions of applications that
include the Package, and Distribute the result without restriction,
provided the result does not expose a direct interface to the Package.


Items That are Not Considered Part of a Modified Version

(9) Works (including, but not limited to, modules and scripts) that
merely extend or make use of the Package, do not, by themselves, cause
the Package to be a Modified Version.  In addition, such works are not
considered parts of the Package itself, and are not subject to the
terms of this license.


General Provisions

(10)  Any use, modification, and distribution of the Standard or
Modified Versions is governed by this Artistic License. By using,
modifying or distributing the Package, you accept this license. Do not
use, modify, or distribute the Package, if you do not accept this
license.

(11)  If your Modified Version has been derived from a Modified
Version made by someone other than you, you are nevertheless required
to ensure that your Modified Version complies with the requirements of
this license.

(12)  This license does not grant you the right to use any trademark,
service mark, tradename, or logo of the Copyright Holder.

(13)  This license includes the non-exclusive, worldwide,
free-of-charge patent license to make, have made, use, offer to sell,
sell, import and otherwise transfer the Package with respect to any
patent claims licensable by the Copyright Holder that are necessarily
infringed by the Package. If you institute patent litigation
(including a cross-claim or counterclaim) against any party alleging
that the Package constitutes direct or contributory patent
infringement, then this Artistic License to you shall terminate on the
date that such litigation is filed.

(14)  Disclaimer of Warranty:
THE PACKAGE IS PROVIDED BY THE COPYRIGHT HOLDER AND CONTRIBUTORS "AS
IS" AND WITHOUT ANY EXPRESS OR IMPLIED WARRANTIES. THE IMPLIED
WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, OR
NON-INFRINGEMENT ARE DISCLAIMED TO THE EXTENT PERMITTED BY YOUR LOCAL
LAW. UNLESS REQUIRED BY LAW, NO COPYRIGHT HOLDER OR CONTRIBUTOR WILL
BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, OR CONSEQUENTIAL
DAMAGES ARISING IN ANY WAY OUT OF THE USE OF THE PACKAGE, EVEN IF
ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

*/
