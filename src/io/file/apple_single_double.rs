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

use crate::entity::{FileEntity, FileEntityKind};
pub use crate::metadata::AppleMetadata;
use crate::payload::Extent;
use crate::streams::{AttachedStream, StreamKind, StreamName};
pub use ctb_formats_apple_single_double::{
    AppleArchive, AppleArchiveEntry, AppleDatesInfo, AppleDoubleStyle,
    AppleExtendedAttribute, AppleFormat, AppleRawEntry, AppleReadOptions,
    AppleSingleExtension, EntryType, ExtendedFinderInfo, FinderFlags,
    FinderInfo, FinderLabel, get_companion_path, is_apple_double_file,
    is_apple_single_file, read_apple_single_double, write_apple_single_double,
    APPLEDOUBLE_MAGIC_BE, APPLEDOUBLE_MAGIC_LE, APPLESINGLE_MAGIC_BE,
    APPLESINGLE_MAGIC_LE, VERSION_2_0_BE,
};
use std::path::{Path, PathBuf};

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
            && self.unrecognized_entries.is_empty()
        {
            return None;
        }

        Some(AppleMetadata {
            finder_info: self.finder_info.clone(),
            real_name: self.real_name.clone(),
            comment: self.comment.clone(),
            backup_timestamp_sec,
            unrecognized_entries: self.unrecognized_entries.clone(),
        })
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
    let mut finder_info = apple_meta.and_then(|a| a.finder_info.clone());

    // Reason for fallback: absent flags metadata defaults to empty slice
    let entity_flags = entity.metadata.flags.as_deref().unwrap_or(&[]);

    let has_mac_flags = entity_flags.iter().any(|f| matches!(
        f,
        crate::metadata::FileFlag::OnDesk
            | crate::metadata::FileFlag::SharedApp
            | crate::metadata::FileFlag::NoInits
            | crate::metadata::FileFlag::Inited
            | crate::metadata::FileFlag::CustomIcon
            | crate::metadata::FileFlag::Stationery
            | crate::metadata::FileFlag::NameLocked
            | crate::metadata::FileFlag::HasBundle
            | crate::metadata::FileFlag::Invisible
            | crate::metadata::FileFlag::Alias
            | crate::metadata::FileFlag::CustomBadge
            | crate::metadata::FileFlag::RoutingInfo
            | crate::metadata::FileFlag::ExtendedFlagsInvalid
    ));
    if has_mac_flags && finder_info.is_none() {
        finder_info = Some(FinderInfo::default());
    }
    if let Some(ref mut finfo) = finder_info {
        for flag in entity_flags {
            match flag {
                crate::metadata::FileFlag::OnDesk => finfo.flags.is_on_desk = true,
                crate::metadata::FileFlag::SharedApp => finfo.flags.is_shared = true,
                crate::metadata::FileFlag::NoInits => finfo.flags.has_no_inits = true,
                crate::metadata::FileFlag::Inited => finfo.flags.has_been_inited = true,
                crate::metadata::FileFlag::CustomIcon => finfo.flags.has_custom_icon = true,
                crate::metadata::FileFlag::Stationery => finfo.flags.is_stationery = true,
                crate::metadata::FileFlag::NameLocked => finfo.flags.name_locked = true,
                crate::metadata::FileFlag::HasBundle => finfo.flags.has_bundle = true,
                crate::metadata::FileFlag::Invisible => finfo.flags.is_invisible = true,
                crate::metadata::FileFlag::Alias => finfo.flags.is_alias = true,
                crate::metadata::FileFlag::CustomBadge => {
                    let ext = finfo.extended.get_or_insert_with(ExtendedFinderInfo::default);
                    ext.xflags.custom_badge = true;
                }
                crate::metadata::FileFlag::RoutingInfo => {
                    let ext = finfo.extended.get_or_insert_with(ExtendedFinderInfo::default);
                    ext.xflags.routing_info = true;
                }
                crate::metadata::FileFlag::ExtendedFlagsInvalid => {
                    let ext = finfo.extended.get_or_insert_with(ExtendedFinderInfo::default);
                    ext.xflags.extended_flags_invalid = true;
                }
                _ => {}
            }
        }
    }
    let real_name = apple_meta.and_then(|a| a.real_name.clone());
    let comment = apple_meta.and_then(|a| a.comment.clone());
    let backup_timestamp_sec = apple_meta.and_then(|a| a.backup_timestamp_sec);
    // Reason for fallback: When Apple metadata is absent, there are no unrecognized entries to preserve.
    let unrecognized_entries = apple_meta
        .map(|a| a.unrecognized_entries.clone())
        .unwrap_or_default();

    let timestamps = entity.metadata.timestamps.as_ref().map(|ts| AppleDatesInfo {
        birthtime_sec: ts.birthtime_sec,
        mtime_sec: ts.mtime_sec,
        ctime_sec: ts.ctime_sec,
        atime_sec: ts.atime_sec,
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
        unrecognized_entries,
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
    // Reason for fallback: absent AppleMetadata defaults to false (no metadata to preserve)
    let has_apple_meta = entity.metadata.apple.as_ref().map_or(false, |a| !a.is_empty());
    if entity.streams.is_empty() && !has_apple_meta {
        return Ok(None);
    }

    // Reason for fallback: destination path without a parent directory defaults to current working directory "."
    let parent_dir = dest_path.parent().unwrap_or(Path::new("."));
    // Reason for fallback: destination path without a file name component defaults to verbatim OsStr
    let file_name = dest_path.file_name().unwrap_or(dest_path.as_os_str());
    let is_dir = matches!(entity.kind, FileEntityKind::Directory | FileEntityKind::Bundle { .. });

    let companion_path = get_companion_path(
        parent_dir,
        Path::new(file_name),
        is_dir,
        style,
        Some(dest_dir_root),
        entity.identity.relative_path.as_deref(),
    );

    if let Some(parent) = companion_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create parent directory for AppleDouble companion: {}", parent.display()))?;
    }

    let apple_double_bytes = serialize_apple_double_for_entity(entity)?;
    std::fs::write(&companion_path, &apple_double_bytes)
        .with_context(|| format!("Failed to write AppleDouble companion file: {}", companion_path.display()))?;

    // Apply timestamps from entity
    if let Some(ref ts) = entity.metadata.timestamps {
        let atime = filetime::FileTime::from_unix_time(
            ts.atime_sec,
            ts.atime_nsec,
        );
        let mtime = filetime::FileTime::from_unix_time(
            ts.mtime_sec,
            ts.mtime_nsec,
        );
        let _ = filetime::set_file_times(&companion_path, atime, mtime);
    }

    Ok(Some(companion_path))
}


fn strip_apple_single_extension_from_entity(entity: &mut FileEntity, ext_suffix: &str) {
    let Some(ref rel) = entity.identity.relative_path else {
        return;
    };
    let file_name_opt = rel.file_name().and_then(|n| n.to_str()).map(ToString::to_string);
    if let Some(file_name) = file_name_opt {
        let should_strip = if file_name.len() >= ext_suffix.len() {
            let suffix_start = file_name.len().saturating_sub(ext_suffix.len());
            // Reason for fallback: slice guaranteed in bounds by length check
            file_name.get(suffix_start..).is_some_and(|s| s.eq_ignore_ascii_case(ext_suffix))
        } else {
            false
        };
        if should_strip {
            let stripped_len = file_name.len().saturating_sub(ext_suffix.len());
            if let Some(stripped_name) = file_name.get(..stripped_len) {
                let parent = rel.parent();
                // Reason for fallback: empty or absent parent defaults to relative file name
                let new_rel = match parent {
                    Some(p) if !p.as_os_str().is_empty() => p.join(stripped_name),
                    _ => PathBuf::from(stripped_name),
                };
                entity.identity.relative_path = Some(new_rel.clone());
                entity.identity.raw_relative_path = Some(new_rel.as_os_str().as_encoded_bytes().to_vec());
                entity.identity.raw_filename = Some(stripped_name.as_bytes().to_vec());
            }
        }
    }
}

/// Checks for and joins AppleDouble companion files or AppleSingle archive into `entity`.
pub fn join_apple_double_or_single(
    entity: &mut FileEntity,
    file_path: &Path,
    base_dir: Option<&Path>,
    options: &AppleReadOptions,
) -> Result<()> {
    // 1. If any AppleSingle read style is enabled and this is a regular file, check if it's AppleSingle
    if options.any_apple_single() && entity.is_regular() {
        // Reason for fallback: file path without a file name component defaults to verbatim OsStr
        let file_name_os = file_path.file_name().unwrap_or(file_path.as_os_str());
        let file_name_str = file_name_os.to_string_lossy();

        let single_match = if options.read_apple_single_as
            && (file_name_str.ends_with(".as") || file_name_str.ends_with(".AS"))
        {
            Some(AppleSingleExtension::As)
        } else if options.read_apple_single_asf
            && (file_name_str.ends_with(".asf") || file_name_str.ends_with(".ASF"))
        {
            Some(AppleSingleExtension::Asf)
        } else if options.read_apple_single_without_extension || options.read_apple_single {
            Some(AppleSingleExtension::WithoutExtension)
        } else {
            None
        };

        if let Some(matched_ext) = single_match {
            let is_single_magic = match std::fs::File::open(file_path) {
                Ok(mut f) => {
                    use std::io::Read;
                    let mut magic = [0u8; 4];
                    if f.read_exact(&mut magic).is_ok() {
                        let m = u32::from_be_bytes(magic);
                        m == APPLESINGLE_MAGIC_BE || m == APPLESINGLE_MAGIC_LE
                    } else {
                        false
                    }
                }
                Err(_) => false,
            };

            if is_single_magic {
                if let Ok(data) = std::fs::read(file_path) {
                    if let Ok(archive) = read_apple_single_double(&data) {
                        if archive.format == AppleFormat::AppleSingle {
                            unpack_apple_single_into_entity(entity, &archive)?;
                            match matched_ext {
                                AppleSingleExtension::As => {
                                    strip_apple_single_extension_from_entity(entity, ".as");
                                }
                                AppleSingleExtension::Asf => {
                                    strip_apple_single_extension_from_entity(entity, ".asf");
                                }
                                AppleSingleExtension::WithoutExtension => {}
                            }
                            return Ok(());
                        }
                    }
                }
            }
        }
    }

    // 2. Check for AppleDouble companion files according to enabled read options
    // Reason for fallback: file path without a parent directory defaults to current working directory "."
    let parent_dir = file_path.parent().unwrap_or(Path::new("."));
    // Reason for fallback: file path without a file name component defaults to verbatim OsStr
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
            entity.identity.relative_path.as_deref(),
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
            entity.identity.relative_path.as_deref(),
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
            entity.identity.relative_path.as_deref(),
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
    if let Some(ref finfo) = archive.finder_info {
        let flags_to_add = [
            (finfo.flags.is_on_desk, crate::metadata::FileFlag::OnDesk),
            (finfo.flags.is_shared, crate::metadata::FileFlag::SharedApp),
            (finfo.flags.has_no_inits, crate::metadata::FileFlag::NoInits),
            (finfo.flags.has_been_inited, crate::metadata::FileFlag::Inited),
            (finfo.flags.has_custom_icon, crate::metadata::FileFlag::CustomIcon),
            (finfo.flags.is_stationery, crate::metadata::FileFlag::Stationery),
            (finfo.flags.name_locked, crate::metadata::FileFlag::NameLocked),
            (finfo.flags.has_bundle, crate::metadata::FileFlag::HasBundle),
            (finfo.flags.is_invisible, crate::metadata::FileFlag::Invisible),
            (finfo.flags.is_alias, crate::metadata::FileFlag::Alias),
        ];
        let flags = entity.metadata.flags.get_or_insert_with(Vec::new);
        for (is_set, flag) in flags_to_add {
            if is_set && !flags.contains(&flag) {
                flags.push(flag);
            }
        }
        if let Some(ref ext) = finfo.extended {
            if ext.xflags.custom_badge && !flags.contains(&crate::metadata::FileFlag::CustomBadge) {
                flags.push(crate::metadata::FileFlag::CustomBadge);
            }
            if ext.xflags.routing_info && !flags.contains(&crate::metadata::FileFlag::RoutingInfo) {
                flags.push(crate::metadata::FileFlag::RoutingInfo);
            }
            if ext.xflags.extended_flags_invalid && !flags.contains(&crate::metadata::FileFlag::ExtendedFlagsInvalid) {
                flags.push(crate::metadata::FileFlag::ExtendedFlagsInvalid);
            }
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
            for unrec in apple_meta.unrecognized_entries {
                if !existing
                    .unrecognized_entries
                    .iter()
                    .any(|e| e.entry_id == unrec.entry_id)
                {
                    existing.unrecognized_entries.push(unrec);
                }
            }
        }
    }
    Ok(())
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
#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace test prelude"
)]
mod tests {
    use super::*;
    use crate::metadata::FileFlag;

    #[crate::ctb_test]
    fn test_mac_file_flags_sync() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let file_path = temp.path().join("test_file.txt");
        std::fs::write(&file_path, b"payload")?;

        let mut entity = FileEntity::from_filesystem(&file_path, None)?;
        entity.metadata.flags = Some(vec![
            FileFlag::OnDesk,
            FileFlag::NoInits,
            FileFlag::Invisible,
            FileFlag::CustomBadge,
        ]);

        let archive = create_apple_archive_from_entity(&entity, AppleFormat::AppleDouble, None)?;
        let finfo = archive.finder_info.as_ref().context("missing finder_info")?;
        ensure!(finfo.flags.is_on_desk);
        ensure!(finfo.flags.has_no_inits);
        ensure!(finfo.flags.is_invisible);
        let ext = finfo.extended.as_ref().context("missing extended finder info")?;
        ensure!(ext.xflags.custom_badge);

        let mut roundtrip_entity = FileEntity::from_filesystem(&file_path, None)?;
        roundtrip_entity.metadata.flags = Some(Vec::new());
        join_apple_archive_into_entity(&mut roundtrip_entity, &archive)?;
        let rt_flags = roundtrip_entity.metadata.flags.as_ref().context("missing flags")?;
        ensure!(rt_flags.contains(&FileFlag::OnDesk));
        ensure!(rt_flags.contains(&FileFlag::NoInits));
        ensure!(rt_flags.contains(&FileFlag::Invisible));
        ensure!(rt_flags.contains(&FileFlag::CustomBadge));

        Ok(())
    }
}

