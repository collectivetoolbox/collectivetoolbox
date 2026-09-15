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

//! AppleSingle and AppleDouble integration for file entities and streams.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::file::entity::{FileEntity, FileEntityKind};
pub use crate::file::metadata::AppleMetadata;
use crate::file::payload::Extent;
use crate::file::streams::{AttachedStream, StreamKind, StreamName};
pub use ctb_formats_apple_single_double::{
    AppleArchive, AppleDatesInfo, AppleExtendedAttribute, AppleFormat,
    AppleArchiveEntry, EntryType, ExtendedFinderInfo, FinderFlags, FinderInfo,
    FinderLabel, read_apple_single_double, write_apple_single_double,
    APPLEDOUBLE_MAGIC_BE, APPLEDOUBLE_MAGIC_LE, APPLESINGLE_MAGIC_BE,
    APPLESINGLE_MAGIC_LE, VERSION_2_0_BE,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// The storage style used for auxiliary AppleDouble companion files.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    clap::ValueEnum,
    Serialize,
    Deserialize,
)]
pub enum AppleDoubleStyle {
    /// Normal alongside companion file prefixed by `._` in the same directory.
    Alongside,
    /// Companion file in top-level `__MACOSX` directory mirroring relative path.
    Zip,
    /// Netatalk `.AppleDouble/<filename>` and `.AppleDouble/.Parent` companion files.
    Netatalk,
}

impl AppleDoubleStyle {
    /// Canonical string identifier for this style.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Alongside => "alongside",
            Self::Zip => "zip",
            Self::Netatalk => "netatalk",
        }
    }
}

/// Extension methods for [`AppleArchive`] bridging to `ctb-io-file` types.
pub trait AppleArchiveExt {
    /// Converts resource fork and extended attributes into [`AttachedStream`]s.
    fn to_attached_streams(&self) -> Result<Vec<AttachedStream>>;

    /// Extracts [`AppleMetadata`] from this archive, if any relevant fields are present.
    fn to_apple_metadata(&self) -> Option<AppleMetadata>;
}

impl AppleArchiveExt for AppleArchive {
    fn to_attached_streams(&self) -> Result<Vec<AttachedStream>> {
        let mut streams = Vec::new();

        if let Some(ref rsrc) = self.resource_fork {
            let stream_name = StreamName::from_str("com.apple.ResourceFork");
            let attached = AttachedStream::from_data(
                Some(stream_name),
                StreamKind::MacOsResourceFork,
                rsrc.clone(),
            )?;
            streams.push(attached);
        }

        for attr in &self.extended_attributes {
            let stream_name = StreamName::from_str(&attr.name);
            let attached = AttachedStream::from_data(
                Some(stream_name),
                StreamKind::ExtendedAttribute,
                attr.data.clone(),
            )?;
            streams.push(attached);
        }

        Ok(streams)
    }

    fn to_apple_metadata(&self) -> Option<AppleMetadata> {
        let backup_timestamp_sec = self
            .timestamps
            .as_ref()
            .and_then(|ts| ts.backup_sec)
            .or(self.backup_timestamp_sec);

        if self.finder_info.is_none()
            && self.real_name.is_none()
            && self.comment.is_none()
            && backup_timestamp_sec.is_none()
        {
            return None;
        }

        Some(AppleMetadata {
            finder_info: self.finder_info.clone(),
            real_name: self.real_name.clone(),
            comment: self.comment.clone(),
            backup_timestamp_sec,
        })
    }
}

/// Computes the path to an AppleDouble companion file according to `style`.
#[must_use]
pub fn get_companion_path(
    parent_dir: &Path,
    file_name: &Path,
    is_dir: bool,
    style: AppleDoubleStyle,
    root_dest: Option<&Path>,
    relative_path: Option<&Path>,
) -> PathBuf {
    match style {
        AppleDoubleStyle::Alongside => {
            let name_str = file_name.to_string_lossy();
            parent_dir.join(format!("._{}", name_str))
        }
        AppleDoubleStyle::Zip => {
            let base = root_dest.unwrap_or(parent_dir);
            let rel = relative_path.unwrap_or(file_name);
            let rel_parent = rel.parent().unwrap_or_else(|| Path::new(""));
            let name_str = rel.file_name().unwrap_or(file_name.as_os_str()).to_string_lossy();
            base.join("__MACOSX").join(rel_parent).join(format!("._{}", name_str))
        }
        AppleDoubleStyle::Netatalk => {
            if is_dir {
                parent_dir.join(file_name).join(".AppleDouble").join(".Parent")
            } else {
                parent_dir.join(".AppleDouble").join(file_name)
            }
        }
    }
}

/// Constructs an [`AppleArchive`] representing the metadata, resource fork, and
/// extended attributes attached to `entity`.
pub fn create_apple_archive_from_entity(
    entity: &FileEntity,
    format: AppleFormat,
    data_fork: Option<Vec<u8>>,
) -> Result<AppleArchive> {
    let mut resource_fork = None;
    let mut extended_attributes = Vec::new();

    for stream in &entity.streams {
        match stream.kind {
            StreamKind::MacOsResourceFork => {
                if let Some(ref data) = stream.data {
                    resource_fork = Some(data.clone());
                }
            }
            StreamKind::ExtendedAttribute => {
                let name = match &stream.name {
                    Some(n) => n.to_string_lossy().into_owned(),
                    None => continue,
                };
                if let Some(ref data) = stream.data {
                    extended_attributes.push(AppleExtendedAttribute {
                        name,
                        size: data.len(),
                        data: data.clone(),
                    });
                }
            }
            _ => {}
        }
    }

    let apple_meta = entity.metadata.apple.as_ref();
    let finder_info = apple_meta.and_then(|a| a.finder_info.clone());
    let real_name = apple_meta.and_then(|a| a.real_name.clone());
    let comment = apple_meta.and_then(|a| a.comment.clone());
    let backup_timestamp_sec = apple_meta.and_then(|a| a.backup_timestamp_sec);

    let timestamps = Some(AppleDatesInfo {
        birthtime_sec: entity.metadata.timestamps.birthtime_sec,
        mtime_sec: entity.metadata.timestamps.mtime_sec,
        ctime_sec: entity.metadata.timestamps.ctime_sec,
        atime_sec: entity.metadata.timestamps.atime_sec,
        backup_sec: backup_timestamp_sec,
    });

    let data_fork_size = data_fork.as_ref().map(Vec::len);
    let resource_fork_size = resource_fork.as_ref().map(Vec::len);

    Ok(AppleArchive {
        format,
        version: VERSION_2_0_BE,
        real_name,
        comment,
        timestamps,
        backup_timestamp_sec,
        finder_info,
        extended_attributes,
        data_fork,
        resource_fork,
        data_fork_size,
        resource_fork_size,
        entries: Vec::new(),
    })
}

/// Serializes an AppleDouble companion payload for `entity`.
pub fn serialize_apple_double_for_entity(entity: &FileEntity) -> Result<Vec<u8>> {
    let archive = create_apple_archive_from_entity(entity, AppleFormat::AppleDouble, None)?;
    write_apple_single_double(&archive)
}

/// Selection for writing AppleSingle, AppleDouble, or native filesystem streams.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppleWriteMode {
    /// Only native streams. If streams or Apple metadata cannot be written natively on destination filesystem, fails in strict lossless mode.
    #[default]
    NativeOnly,
    /// Try native streams first. If not supported or if non-resource-fork Apple metadata cannot be stored natively, fall back to creating companion AppleDouble file according to style.
    MaybeAppleDouble(AppleDoubleStyle),
    /// Always write AppleDouble companion file according to style for any file having streams or Apple metadata.
    ForceAppleDouble(AppleDoubleStyle),
    /// Try native streams first. If not supported, write as AppleSingle file.
    MaybeAppleSingle,
    /// Always write file as AppleSingle archive.
    ForceAppleSingle,
}

impl AppleWriteMode {
    /// Returns true if this write mode forces AppleSingle or AppleDouble.
    #[must_use]
    pub fn is_force(&self) -> bool {
        matches!(self, Self::ForceAppleDouble(_) | Self::ForceAppleSingle)
    }

    /// Returns the associated [`AppleDoubleStyle`] if this mode writes AppleDouble.
    #[must_use]
    pub fn double_style(&self) -> Option<AppleDoubleStyle> {
        match self {
            Self::MaybeAppleDouble(s) | Self::ForceAppleDouble(s) => Some(*s),
            _ => None,
        }
    }
}

/// Options controlling which AppleSingle and AppleDouble styles to detect and unpack on read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppleReadOptions {
    /// Read alongside ._<filename> companion files. Defaults to true.
    pub read_apple_double_alongside: bool,
    /// Read __MACOSX/ companion files. Defaults to false.
    pub read_apple_double_zip: bool,
    /// Read Netatalk .AppleDouble/<filename> and .AppleDouble/.Parent companion files. Defaults to false.
    pub read_apple_double_netatalk: bool,
    /// Read AppleSingle files and decode data fork, resource fork, and Apple metadata. Defaults to false.
    pub read_apple_single: bool,
}

impl Default for AppleReadOptions {
    fn default() -> Self {
        Self {
            read_apple_double_alongside: true,
            read_apple_double_zip: false,
            read_apple_double_netatalk: false,
            read_apple_single: false,
        }
    }
}

/// Writes an AppleDouble companion file for `entity` to `dest_path` according to `style`.
///
/// Returns the path to the created companion file, or `None` if the entity has neither
/// attached streams nor Apple metadata.
pub fn write_apple_double_companion(
    entity: &FileEntity,
    dest_path: &Path,
    dest_dir_root: &Path,
    style: AppleDoubleStyle,
) -> Result<Option<PathBuf>> {
    let has_apple_meta = entity.metadata.apple.as_ref().map_or(false, |a| !a.is_empty());
    if entity.streams.is_empty() && !has_apple_meta {
        return Ok(None);
    }

    let parent_dir = dest_path.parent().unwrap_or(Path::new("."));
    let file_name = dest_path.file_name().unwrap_or(dest_path.as_os_str());
    let is_dir = matches!(entity.kind, FileEntityKind::Directory | FileEntityKind::Bundle { .. });

    let companion_path = get_companion_path(
        parent_dir,
        Path::new(file_name),
        is_dir,
        style,
        Some(dest_dir_root),
        Some(&entity.identity.relative_path),
    );

    if let Some(parent) = companion_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create parent directory for AppleDouble companion: {}", parent.display()))?;
    }

    let apple_double_bytes = serialize_apple_double_for_entity(entity)?;
    std::fs::write(&companion_path, &apple_double_bytes)
        .with_context(|| format!("Failed to write AppleDouble companion file: {}", companion_path.display()))?;

    // Apply timestamps from entity
    let atime = filetime::FileTime::from_unix_time(
        entity.metadata.timestamps.atime_sec,
        entity.metadata.timestamps.atime_nsec,
    );
    let mtime = filetime::FileTime::from_unix_time(
        entity.metadata.timestamps.mtime_sec,
        entity.metadata.timestamps.mtime_nsec,
    );
    let _ = filetime::set_file_times(&companion_path, atime, mtime);

    Ok(Some(companion_path))
}

/// Checks for and joins AppleDouble companion files or AppleSingle archive into `entity`.
pub fn join_apple_double_or_single(
    entity: &mut FileEntity,
    file_path: &Path,
    base_dir: Option<&Path>,
    options: &AppleReadOptions,
) -> Result<()> {
    // 1. If read_apple_single is enabled and this is a regular file, check if it's AppleSingle
    if options.read_apple_single && entity.is_regular() {
        if let Ok(data) = std::fs::read(file_path) {
            if let Some(magic_slice) = data.get(..4) {
                if let Ok(magic_arr) = <[u8; 4]>::try_from(magic_slice) {
                    let magic = u32::from_be_bytes(magic_arr);
                    if magic == APPLESINGLE_MAGIC_BE || magic == APPLESINGLE_MAGIC_LE {
                        if let Ok(archive) = read_apple_single_double(&data) {
                            if archive.format == AppleFormat::AppleSingle {
                                unpack_apple_single_into_entity(entity, &archive)?;
                                return Ok(());
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Check for AppleDouble companion files according to enabled read options
    let parent_dir = file_path.parent().unwrap_or(Path::new("."));
    let file_name = file_path.file_name().unwrap_or(file_path.as_os_str());
    let is_dir = matches!(entity.kind, FileEntityKind::Directory | FileEntityKind::Bundle { .. });

    let mut companion_candidates = Vec::new();

    if options.read_apple_double_alongside {
        let path = get_companion_path(
            parent_dir,
            Path::new(file_name),
            is_dir,
            AppleDoubleStyle::Alongside,
            base_dir,
            Some(&entity.identity.relative_path),
        );
        companion_candidates.push(path);
    }
    if options.read_apple_double_zip {
        let path = get_companion_path(
            parent_dir,
            Path::new(file_name),
            is_dir,
            AppleDoubleStyle::Zip,
            base_dir,
            Some(&entity.identity.relative_path),
        );
        companion_candidates.push(path);
    }
    if options.read_apple_double_netatalk {
        let path = get_companion_path(
            parent_dir,
            Path::new(file_name),
            is_dir,
            AppleDoubleStyle::Netatalk,
            base_dir,
            Some(&entity.identity.relative_path),
        );
        companion_candidates.push(path);
    }

    for companion_path in companion_candidates {
        if companion_path.is_file() {
            if let Ok(data) = std::fs::read(&companion_path) {
                if let Ok(archive) = read_apple_single_double(&data) {
                    if archive.format == AppleFormat::AppleDouble {
                        join_apple_archive_into_entity(entity, &archive)?;
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

fn unpack_apple_single_into_entity(entity: &mut FileEntity, archive: &AppleArchive) -> Result<()> {
    if let Some(ref data) = archive.data_fork {
        let size = u64::try_from(data.len())?;
        let mut hasher = ctb_formats_checksum::Sha256Stream::new();
        hasher.update(data);
        let sha256 = hasher.finalize();
        entity.kind = FileEntityKind::Regular {
            size,
            sha256,
            is_sparse: false,
            extents: vec![Extent::Data {
                offset: 0,
                length: size,
            }],
        };
    }
    join_apple_archive_into_entity(entity, archive)
}

fn join_apple_archive_into_entity(entity: &mut FileEntity, archive: &AppleArchive) -> Result<()> {
    let streams = archive.to_attached_streams()?;
    for stream in streams {
        if !entity.streams.iter().any(|s| s.name == stream.name) {
            entity.streams.push(stream);
        }
    }
    if let Some(apple_meta) = archive.to_apple_metadata() {
        if entity.metadata.apple.is_none() {
            entity.metadata.apple = Some(apple_meta);
        } else if let Some(ref mut existing) = entity.metadata.apple {
            if existing.finder_info.is_none() {
                existing.finder_info = apple_meta.finder_info;
            }
            if existing.real_name.is_none() {
                existing.real_name = apple_meta.real_name;
            }
            if existing.comment.is_none() {
                existing.comment = apple_meta.comment;
            }
            if existing.backup_timestamp_sec.is_none() {
                existing.backup_timestamp_sec = apple_meta.backup_timestamp_sec;
            }
        }
    }
    Ok(())
}
