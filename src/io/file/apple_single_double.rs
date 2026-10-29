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
#[cfg(unix)]
use rustix::fd::AsFd;

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


fn read_stream_bytes_from_source(
    stream: &AttachedStream,
    stream_source: &dyn crate::streams::AttachedStreamSource,
) -> Result<Vec<u8>> {
    let FileEntityKind::Regular {
        size: expected_size,
        sha256: expected_sha256,
        ..
    } = &stream.entity.kind
    else {
        bail!("Attached stream is not a regular payload");
    };
    let mut reader = stream_source.open(stream)?;
    let mut hasher = ctb_formats_checksum::Sha256Stream::new();
    let mut data = Vec::new();
    data.try_reserve_exact(usize::try_from(*expected_size)?)?;
    let mut buf = [0_u8; 64 * 1024];
    let mut total_read = 0_u64;
    loop {
        let n = std::io::Read::read(&mut reader, &mut buf)
            .context("Failed to read attached stream source")?;
        if n == 0 {
            break;
        }
        let chunk = buf.get(..n).context("Invalid slice range")?;
        hasher.update(chunk);
        total_read = total_read
            .checked_add(u64::try_from(n)?)
            .context("Stream size overflow")?;
        ensure!(total_read <= *expected_size, "Stream grew during restoration");
        data.extend_from_slice(chunk);
    }
    ensure!(
        total_read == *expected_size && hasher.finalize() == *expected_sha256,
        "Attached stream payload does not match its descriptor: {:?}",
        stream.to_string_lossy()
    );
    Ok(data)
}

/// Constructs an [`AppleArchive`] representing the metadata, resource fork, and
/// extended attributes attached to `entity`.
pub fn create_apple_archive_from_entity(
    entity: &FileEntity,
    format: AppleFormat,
    data_fork: Option<Vec<u8>>,
) -> Result<AppleArchive> {
    create_apple_archive_from_entity_with_stream_source(
        entity,
        format,
        data_fork,
        &crate::streams::FilesystemStreamSource,
    )
}

/// Constructs an [`AppleArchive`] representing the metadata, resource fork, and
/// extended attributes attached to `entity`, reading resident or nonresident
/// attached stream payloads from `stream_source`.
pub fn create_apple_archive_from_entity_with_stream_source(
    entity: &FileEntity,
    format: AppleFormat,
    data_fork: Option<Vec<u8>>,
    stream_source: &dyn crate::streams::AttachedStreamSource,
) -> Result<AppleArchive> {
    let mut resource_fork = None;
    let mut extended_attributes = Vec::new();

    for stream in &entity.streams {
        match stream.kind {
            StreamKind::MacOsResourceFork => {
                let data = read_stream_bytes_from_source(stream, stream_source)?;
                resource_fork = Some(data);
            }
            StreamKind::ExtendedAttribute | StreamKind::NtfsAlternateDataStream => {
                let name = match &stream.name {
                    Some(n) => n.to_string_lossy().into_owned(),
                    None => continue,
                };
                let data = read_stream_bytes_from_source(stream, stream_source)?;
                extended_attributes.push(AppleExtendedAttribute {
                    name,
                    size: data.len(),
                    data,
                });
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
    if entity.is_dir() {
        if finder_info.is_some() || !extended_attributes.is_empty() {
            let finfo = finder_info.get_or_insert_with(FinderInfo::default);
            if finfo.window_bounds.is_none() {
                finfo.window_bounds = Some([50, 50, 500, 700]);
            }
            if let Some(ref mut ext) = finfo.extended {
                ext.icon_id = 0;
            }
        }
    } else if let Some(ref mut finfo) = finder_info {
        finfo.window_bounds = None;
        if let Some(ref mut ext) = finfo.extended {
            ext.scroll_position = None;
        }
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
    serialize_apple_double_for_entity_with_stream_source(
        entity,
        &crate::streams::FilesystemStreamSource,
    )
}

/// Serializes an AppleDouble companion payload for `entity`, reading attached
/// streams from `stream_source`.
pub fn serialize_apple_double_for_entity_with_stream_source(
    entity: &FileEntity,
    stream_source: &dyn crate::streams::AttachedStreamSource,
) -> Result<Vec<u8>> {
    let archive = create_apple_archive_from_entity_with_stream_source(
        entity,
        AppleFormat::AppleDouble,
        None,
        stream_source,
    )?;
    write_apple_single_double(&archive)
}

/// Selection for writing AppleSingle, AppleDouble, Windows ADS companions, or native filesystem streams.
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
    /// Try native streams first. If not supported, fall back to creating alongside `<filename>:<stream_name>` Windows ADS companion files.
    MaybeWindowsAds,
    /// Always write attached streams as alongside `<filename>:<stream_name>` Windows ADS companion files.
    ForceWindowsAds,
}

impl AppleWriteMode {
    /// Returns true if this write mode forces AppleSingle, AppleDouble, or Windows ADS companions.
    #[must_use]
    pub fn is_force(&self) -> bool {
        matches!(
            self,
            Self::ForceAppleDouble(_) | Self::ForceAppleSingle | Self::ForceWindowsAds
        )
    }

    /// Returns the associated [`AppleDoubleStyle`] if this mode writes AppleDouble.
    #[must_use]
    pub fn double_style(&self) -> Option<AppleDoubleStyle> {
        match self {
            Self::MaybeAppleDouble(s) | Self::ForceAppleDouble(s) => Some(*s),
            _ => None,
        }
    }

    /// Returns true if this mode writes Windows ADS `<filename>:<stream_name>` companions.
    #[must_use]
    pub const fn is_windows_ads(&self) -> bool {
        matches!(self, Self::MaybeWindowsAds | Self::ForceWindowsAds)
    }
}


/// Writes an AppleDouble companion file for `entity` to `dest_path` according to `style`.
///
/// Returns the path to the created companion file, or `None` if the entity has neither
/// attached streams nor Apple metadata.
pub fn write_apple_double_companion(
    entity: &FileEntity,
    dest_path: &Path,
    dest_dir: &crate::sandboxable_dir::SandboxableDir,
    style: AppleDoubleStyle,
    existing: crate::sandboxable_dir::ExistingEntryPolicy,
) -> Result<Option<PathBuf>> {
    write_apple_double_companion_with_stream_source(
        entity,
        dest_path,
        dest_dir,
        style,
        existing,
        &crate::streams::FilesystemStreamSource,
    )
}

/// Writes an AppleDouble companion file for `entity` to `dest_path` according to `style`,
/// reading attached streams from `stream_source`.
pub fn write_apple_double_companion_with_stream_source(
    entity: &FileEntity,
    dest_path: &Path,
    dest_dir: &crate::sandboxable_dir::SandboxableDir,
    style: AppleDoubleStyle,
    existing: crate::sandboxable_dir::ExistingEntryPolicy,
    stream_source: &dyn crate::streams::AttachedStreamSource,
) -> Result<Option<PathBuf>> {
    if apple_double_companion_path(entity, dest_path, dest_dir, style)?.is_none() {
        return Ok(None);
    }
    let apple_double_bytes =
        serialize_apple_double_for_entity_with_stream_source(entity, stream_source)?;
    write_apple_double_companion_bytes(
        entity,
        dest_path,
        dest_dir,
        style,
        existing,
        &apple_double_bytes,
    )
}

/// Writes pre-serialized AppleDouble companion bytes for `entity` to `dest_path` according to `style`.
pub(crate) fn write_apple_double_companion_bytes(
    entity: &FileEntity,
    dest_path: &Path,
    dest_dir: &crate::sandboxable_dir::SandboxableDir,
    style: AppleDoubleStyle,
    existing: crate::sandboxable_dir::ExistingEntryPolicy,
    apple_double_bytes: &[u8],
) -> Result<Option<PathBuf>> {
    let times = if let Some(timestamps) = &entity.metadata.timestamps {
        timestamps.validate()?;
        Some((
            crate::metadata::checked_file_time(timestamps.atime_sec, timestamps.atime_nsec)
                .context("Invalid native AppleDouble companion access time")?,
            crate::metadata::checked_file_time(timestamps.mtime_sec, timestamps.mtime_nsec)
                .context("Invalid native AppleDouble companion modification time")?,
        ))
    } else {
        None
    };
    let Some(companion_path) = apple_double_companion_path(entity, dest_path, dest_dir, style)? else {
        return Ok(None);
    };
    let relative = companion_path.strip_prefix(dest_dir.root_path())
        .context("AppleDouble companion is outside the destination sandbox")?;
    let (parent, final_name) = dest_dir.ensure_parent_dir(
        relative, crate::path_policy::PathTraversalPolicy::StrictSandboxed,
    )?;
    let temp_name = format!(".ctb-apple-double-{}", uuid());
    let mut staged = dest_dir.create_temp_file(&parent.as_fd(), &temp_name, 0o600)?;
    let result = (|| -> Result<()> {
        std::io::Write::write_all(&mut staged, apple_double_bytes)
            .context("Failed to stage AppleDouble companion bytes")?;
        if let Some((atime, mtime)) = times {
            filetime::set_file_handle_times(&staged, Some(atime), Some(mtime))
                .context("Failed to stage AppleDouble companion timestamps")?;
        }
        staged.sync_all().context("Failed to sync staged AppleDouble companion")?;
        dest_dir.commit_atomic_file(&parent.as_fd(), &temp_name, &final_name, existing)
    })();
    drop(staged);
    if let Err(error) = result {
        #[cfg(unix)]
        let cleanup = rustix::fs::unlinkat(parent.as_fd(), temp_name.as_str(), rustix::fs::AtFlags::empty());
        #[cfg(not(unix))]
        let cleanup = std::fs::remove_file(
            companion_path.parent().context("AppleDouble companion has no parent")?.join(&temp_name),
        );
        if let Err(cleanup) = cleanup {
            return Err(error).context(format!("Failed to remove staged AppleDouble companion: {cleanup}"));
        }
        return Err(error);
    }

    Ok(Some(companion_path))
}

/// Returns whether the entity has Apple metadata or Mac-specific file flags
/// that require preservation in an AppleDouble companion file.
pub(crate) fn has_apple_metadata(entity: &FileEntity) -> bool {
    // Reason for fallback: absent AppleMetadata defaults to false (no metadata to preserve)
    let has_apple_meta = entity.metadata.apple.as_ref().map_or(false, |a| !a.is_empty());
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
    has_apple_meta || has_mac_flags
}

pub(crate) fn apple_double_companion_path(
    entity: &FileEntity,
    dest_path: &Path,
    dest_dir: &crate::sandboxable_dir::SandboxableDir,
    style: AppleDoubleStyle,
) -> Result<Option<PathBuf>> {
    if entity.streams.is_empty() && !has_apple_metadata(entity) {
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
        Some(dest_dir.root_path()),
        entity.identity.relative_path.as_deref(),
    );

    let relative = companion_path.strip_prefix(dest_dir.root_path())
        .context("AppleDouble companion is outside the destination sandbox")?;
    ensure!(relative.components().all(|component| matches!(
        component, std::path::Component::Normal(_) | std::path::Component::CurDir
    )), "AppleDouble companion path is not contained in the destination sandbox");
    Ok(Some(companion_path))
}


fn strip_apple_single_extension_from_entity(entity: &mut FileEntity, extension: AppleSingleExtension) {
    let Some(ref rel) = entity.identity.relative_path else {
        return;
    };
    let Some(stem) = rel.file_name().and_then(|name| extension.strip_from_name(name)) else {
        return;
    };
    entity.identity.relative_path = Some(rel.with_file_name(stem));
}

/// Checks for and joins AppleDouble companion files or AppleSingle archive into `entity`.
pub fn join_apple_double_or_single(
    entity: &mut FileEntity,
    file_path: &Path,
    base_dir: Option<&Path>,
    options: &AppleReadOptions,
) -> Result<()> {
    join_apple_double_or_single_captured(entity, file_path, base_dir, options, None)
}

pub(crate) fn join_apple_double_or_single_captured(
    entity: &mut FileEntity,
    file_path: &Path,
    base_dir: Option<&Path>,
    options: &AppleReadOptions,
    snapshot: Option<&crate::payload::SourceSnapshot>,
) -> Result<()> {
    // 1. If any AppleSingle read style is enabled and this is a regular file, check if it's AppleSingle
    if options.any_apple_single() && entity.is_regular() {
        // Reason for fallback: file path without a file name component defaults to verbatim OsStr
        let file_name_os = file_path.file_name().unwrap_or(file_path.as_os_str());

        let single_match = if options.read_apple_single_as
            && AppleSingleExtension::As.strip_from_name(file_name_os).is_some()
        {
            Some(AppleSingleExtension::As)
        } else if options.read_apple_single_asf
            && AppleSingleExtension::Asf.strip_from_name(file_name_os).is_some()
        {
            Some(AppleSingleExtension::Asf)
        } else if options.read_apple_single_without_extension || options.read_apple_single {
            Some(AppleSingleExtension::WithoutExtension)
        } else {
            None
        };

        if let Some(matched_ext) = single_match {
            if let Some(data) = read_captured_apple_input(file_path, snapshot, true)? {
                let archive = read_apple_single_double(&data)?;
                ensure!(archive.format == AppleFormat::AppleSingle, "Recognized container is not AppleSingle");
                unpack_apple_single_into_entity(entity, &archive)?;
                if let Some(snapshot) = snapshot {
                    // Reason for fallback: empty data fork represents empty file payload
                    snapshot.set_interpreted_payload(archive.data_fork.clone().unwrap_or_default())?;
                }
                match matched_ext {
                    AppleSingleExtension::As => {
                        strip_apple_single_extension_from_entity(entity, AppleSingleExtension::As);
                    }
                    AppleSingleExtension::Asf => {
                        strip_apple_single_extension_from_entity(entity, AppleSingleExtension::Asf);
                    }
                    AppleSingleExtension::WithoutExtension => {}
                }
                return Ok(());
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

    let mut existing = Vec::new();
    for companion_path in companion_candidates {
        match std::fs::symlink_metadata(&companion_path) {
            Ok(metadata) => {
                ensure!(metadata.is_file(), "AppleDouble companion is not a regular file: {}", companion_path.display());
                if !existing.contains(&companion_path) { existing.push(companion_path); }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {},
            Err(error) => return Err(error).context("Cannot inspect AppleDouble companion"),
        }
    }
    ensure!(existing.len() <= 1, "Multiple AppleDouble companions for {}", file_path.display());
    if let Some(companion_path) = existing.first() {
        let dependency = snapshot.map(|snapshot| snapshot.capture_dependency(companion_path)).transpose()?;
        let data = read_captured_apple_input(companion_path, dependency.as_ref(), false)?.context("AppleDouble input omitted")?;
        let archive = read_apple_single_double(&data).with_context(|| format!("Invalid AppleDouble companion: {}", companion_path.display()))?;
        ensure!(archive.format == AppleFormat::AppleDouble, "Companion is not AppleDouble: {}", companion_path.display());
        join_apple_archive_into_entity(entity, &archive)?;
        if let Some(snapshot) = snapshot { snapshot.retain_changes(snapshot.validate()?)?; }
    }

    // 3. Check for alongside Windows ADS companion files (`<filename>:<stream_name>`)
    if options.read_windows_ads_alongside {
        join_windows_ads_companions_captured(entity, parent_dir, file_name, snapshot)?;
    }

    Ok(())
}

/// Splits a raw filename byte slice into `(base_name, stream_suffix)` if it matches
/// the Windows-style alternate data stream companion convention `<base_name>:<stream_suffix>`,
/// where both `<base_name>` and `<stream_suffix>` are non-empty and `<stream_suffix>` is not
/// solely `:$DATA`.
#[must_use]
pub fn split_windows_ads_companion_bytes(name_bytes: &[u8]) -> Option<(&[u8], &[u8])> {
    let colon_idx = name_bytes.iter().position(|&b| b == b':')?;
    let base = name_bytes.get(..colon_idx)?;
    let suffix = name_bytes.get(colon_idx.checked_add(1)?..)?;
    if base.is_empty() || suffix.is_empty() {
        return None;
    }
    let normalized_suffix = strip_ads_data_type_suffix(suffix);
    if normalized_suffix.is_empty() || normalized_suffix.starts_with(b":") {
        return None;
    }
    Some((base, normalized_suffix))
}

#[must_use]
fn strip_ads_data_type_suffix(suffix: &[u8]) -> &[u8] {
    const DATA_SUFFIX: &[u8] = b":$DATA";
    if suffix.len() >= DATA_SUFFIX.len() {
        let split_at = suffix.len().saturating_sub(DATA_SUFFIX.len());
        if let Some(tail) = suffix.get(split_at..) {
            if tail.eq_ignore_ascii_case(DATA_SUFFIX) {
                // Reason for fallback: empty slice when suffix equals DATA_SUFFIX
                return suffix.get(..split_at).unwrap_or(&[]);
            }
        }
    }
    suffix
}

fn join_windows_ads_companions_captured(
    entity: &mut FileEntity,
    parent_dir: &Path,
    file_name: &std::ffi::OsStr,
    snapshot: Option<&crate::payload::SourceSnapshot>,
) -> Result<()> {
    let entries = match crate::traversal::read_dir_raw(parent_dir) {
        Ok(entries) => entries,
        Err(error)
            if error
                .downcast_ref::<std::io::Error>()
                .is_some_and(|e| e.kind() == std::io::ErrorKind::NotFound) =>
        {
            return Ok(());
        }
        Err(error) => {
            return Err(error).with_context(|| {
                format!(
                    "Failed to inspect directory for Windows ADS companions: {}",
                    parent_dir.display()
                )
            });
        }
    };

    let base_bytes = file_name.as_encoded_bytes();
    let mut matches: Vec<(Vec<u8>, PathBuf)> = Vec::new();

    for entry_res in entries {
        let entry = entry_res.with_context(|| {
            format!(
                "Failed to read directory entry while scanning for Windows ADS companions in {}",
                parent_dir.display()
            )
        })?;
        let candidate_name = entry.file_name();
        let candidate_bytes = candidate_name.as_encoded_bytes();
        let Some((cand_base, stream_suffix)) = split_windows_ads_companion_bytes(candidate_bytes)
        else {
            continue;
        };
        if cand_base != base_bytes {
            continue;
        }
        let companion_path = parent_dir.join(&candidate_name);
        let metadata = match std::fs::symlink_metadata(&companion_path) {
            Ok(meta) => meta,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(error).with_context(|| {
                    format!(
                        "Cannot inspect Windows ADS companion: {}",
                        companion_path.display()
                    )
                });
            }
        };
        if !metadata.is_file() {
            continue;
        }
        matches.push((stream_suffix.to_vec(), companion_path));
    }

    if matches.is_empty() {
        return Ok(());
    }

    matches.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

    for (stream_suffix_bytes, companion_path) in matches {
        let dependency = snapshot
            .map(|snap| snap.capture_dependency(&companion_path))
            .transpose()?;
        let data = read_captured_apple_input(&companion_path, dependency.as_ref(), false)?
            .context("Windows ADS companion input omitted")?;
        let stream_name = crate::streams::StreamName::from_bytes(&stream_suffix_bytes);
        let stream = AttachedStream::from_data(
            Some(stream_name),
            StreamKind::NtfsAlternateDataStream,
            data,
        )?;
        if !entity.streams.iter().any(|s| s.name == stream.name) {
            entity.streams.push(stream);
        }
    }

    entity.streams.sort_by(|a, b| match a.name.cmp(&b.name) {
        std::cmp::Ordering::Equal => a.kind.cmp(&b.kind),
        ord => ord,
    });

    if let Some(snapshot) = snapshot {
        snapshot.retain_changes(snapshot.validate()?)?;
    }

    Ok(())
}

/// Record of a stream name and/or companion relative path adapted during
/// best-effort materialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamAdaptationRecord {
    /// Index of the stream in `entity.streams`.
    pub stream_index: usize,
    /// Original stream name bytes before adaptation.
    pub source_stream_name: Vec<u8>,
    /// Materialized stream name bytes written on disk.
    pub materialized_stream_name: Vec<u8>,
    /// Original relative path of the companion before adaptation, if applicable.
    pub source_relative_path: Option<PathBuf>,
    /// Materialized relative path of the companion written on disk, if applicable.
    pub materialized_relative_path: Option<PathBuf>,
}

/// Planned Windows ADS `<filename>:<stream_name>` companion file for a stream.
#[derive(Debug, Clone)]
pub struct PlannedWindowsAdsCompanion {
    /// Index of the stream in `entity.streams`.
    pub stream_index: usize,
    /// Destination path of the companion file.
    pub companion_path: PathBuf,
    /// Attached stream descriptor.
    pub stream: AttachedStream,
    /// Adaptation metadata if the stream name or companion path was modified in best-effort mode.
    pub adaptation: Option<StreamAdaptationRecord>,
    /// Materialization warning emitted if the stream name was adapted.
    pub warning: Option<String>,
}

/// Computes the planned Windows ADS `<filename>:<stream_name>` companion files for `entity`
/// at `dest_path`, including any best-effort stream-name adaptations and warnings.
pub fn plan_windows_ads_companions(
    entity: &FileEntity,
    dest_path: &Path,
    dest_dir: &crate::sandboxable_dir::SandboxableDir,
    strict_lossless: bool,
) -> Result<Vec<PlannedWindowsAdsCompanion>> {
    if entity.streams.is_empty() {
        return Ok(Vec::new());
    }

    // Reason for fallback: destination path without a parent directory defaults to current working directory "."
    let parent_dir = dest_path.parent().unwrap_or(Path::new("."));
    // Reason for fallback: destination path without a file name component defaults to verbatim OsStr
    let file_name = dest_path.file_name().unwrap_or(dest_path.as_os_str());
    let base_len = file_name.as_encoded_bytes().len();

    let orig_base_rel = entity
        .metadata
        .source_relative_path()
        .or_else(|| entity.identity.relative_path.clone());

    let mut out = Vec::with_capacity(entity.streams.len());
    let mut seen_names = std::collections::HashSet::new();

    for (stream_index, stream) in entity.streams.iter().enumerate() {
        let raw_stream_bytes: std::borrow::Cow<'_, [u8]> = match &stream.name {
            Some(name) => {
                if strict_lossless {
                    let os = name.to_os_string_for_materialization(true)?;
                    std::borrow::Cow::Owned(os.as_encoded_bytes().to_vec())
                } else {
                    name.as_bytes()
                }
            }
            None => match stream.kind {
                StreamKind::MacOsResourceFork => {
                    std::borrow::Cow::Borrowed(crate::streams::RESOURCE_FORK_XATTR_NAME.as_bytes())
                }
                _ => {
                    if strict_lossless {
                        bail!(
                            "Cannot write unnamed stream as Windows ADS companion for {}",
                            dest_path.display()
                        );
                    }
                    std::borrow::Cow::Borrowed(b"")
                }
            },
        };
        let stream_bytes = raw_stream_bytes.as_ref();
        let trimmed_leading = stream_bytes.strip_prefix(b":").unwrap_or(stream_bytes); // Reason for fallback: stream name without leading ':' is already unprefixed
        let orig_suffix = strip_ads_data_type_suffix(trimmed_leading);

        let (final_suffix_bytes, adaptation_warning) = if strict_lossless {
            ensure!(
                !orig_suffix.is_empty(),
                "Windows ADS companion stream name cannot be empty for {}",
                dest_path.display()
            );
            ensure!(
                orig_suffix != b"." && orig_suffix != b"..",
                "Windows ADS companion stream name cannot be '.' or '..' for {}",
                dest_path.display()
            );
            ensure!(
                !orig_suffix.contains(&b'/')
                    && !orig_suffix.contains(&b'\\')
                    && !orig_suffix.contains(&b':')
                    && !orig_suffix.contains(&0),
                "Windows ADS companion stream name contains path separator, colon, or NUL for {}",
                dest_path.display()
            );
            ensure!(
                base_len
                    .saturating_add(1)
                    .saturating_add(orig_suffix.len())
                    <= crate::clean_name::MAX_FILENAME_BYTES,
                "Windows ADS companion filename exceeds maximum filename length ({} bytes) for {}",
                crate::clean_name::MAX_FILENAME_BYTES,
                dest_path.display()
            );
            if cfg!(windows) {
                let s = std::str::from_utf8(orig_suffix).with_context(|| {
                    format!(
                        "Non-UTF-8 stream name cannot be materialized as companion filename on Windows for {}",
                        dest_path.display()
                    )
                })?;
                ensure!(
                    !s.chars().any(|c| matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') || c.is_control()),
                    "Windows ADS companion stream name contains illegal characters for {}",
                    dest_path.display()
                );
            }
            (orig_suffix.to_vec(), None)
        } else {
            let max_suffix_bytes = crate::clean_name::MAX_FILENAME_BYTES
                .saturating_sub(base_len.saturating_add(1));
            ensure!(
                max_suffix_bytes > 0,
                "Base filename {:?} leaves no room for Windows ADS companion suffix on {}",
                file_name,
                dest_path.display()
            );
            let (mut cleaned_str, mut reason) =
                crate::clean_name::clean_stream_name_for_companion(orig_suffix, max_suffix_bytes);

            let mut candidate_companion =
                build_windows_ads_companion_os_string(file_name, cleaned_str.as_bytes())?;
            if seen_names.contains(&candidate_companion) {
                let base_cleaned = cleaned_str.clone();
                let mut collision_idx = 1_usize;
                loop {
                    let disambiguated = crate::clean_name::disambiguate_cleaned_name(
                        &base_cleaned,
                        orig_suffix,
                        collision_idx,
                        max_suffix_bytes,
                    );
                    let cand_os = build_windows_ads_companion_os_string(
                        file_name,
                        disambiguated.as_bytes(),
                    )?;
                    if !seen_names.contains(&cand_os) {
                        cleaned_str = disambiguated;
                        candidate_companion = cand_os;
                        break;
                    }
                    collision_idx = collision_idx.saturating_add(1);
                }
                let disambig_msg = "disambiguated colliding stream name";
                reason = Some(match reason {
                    Some(prev) => format!("{prev}, {disambig_msg}"),
                    None => disambig_msg.to_string(),
                });
            }
            let _ = candidate_companion;
            let warning = if cleaned_str.as_bytes() != orig_suffix {
                let orig_lossy = String::from_utf8_lossy(orig_suffix);
                Some(format!(
                    "Adapted Windows ADS companion stream name {:?} -> {:?} on {} (best-effort: {})",
                    orig_lossy,
                    cleaned_str,
                    dest_path.display(),
                    reason.as_deref().unwrap_or("cleaned stream name") // Reason for fallback: default description when stream name was modified
                ))
            } else {
                None
            };
            (cleaned_str.into_bytes(), warning)
        };

        let companion_file_name =
            build_windows_ads_companion_os_string(file_name, &final_suffix_bytes)?;
        ensure!(
            seen_names.insert(companion_file_name.clone()),
            "Windows ADS companion names collide on {}",
            dest_path.display()
        );
        let companion_path = parent_dir.join(&companion_file_name);
        let relative = companion_path
            .strip_prefix(dest_dir.root_path())
            .context("Windows ADS companion is outside the destination sandbox")?;
        ensure!(
            relative.components().all(|component| matches!(
                component,
                std::path::Component::Normal(_) | std::path::Component::CurDir
            )),
            "Windows ADS companion path is not contained in the destination sandbox"
        );

        let mat_rel = relative.to_path_buf();
        let src_rel = orig_base_rel.as_ref().map(|base_rel| {
            let base_file = base_rel.file_name().unwrap_or(base_rel.as_os_str()); // Reason for fallback: single-component relative path is already the file name
            let orig_comp_os = build_windows_ads_companion_os_string(base_file, orig_suffix)
                .unwrap_or_else(|_| {
                    // Reason for fallback: non-UTF-8 suffix on non-Unix falls back to lossy string display for source relative path record
                    std::ffi::OsString::from(format!(
                        "{}:{}",
                        base_file.to_string_lossy(),
                        String::from_utf8_lossy(orig_suffix)
                    ))
                });
            match base_rel.parent() {
                Some(p) if !p.as_os_str().is_empty() => p.join(orig_comp_os),
                _ => PathBuf::from(orig_comp_os),
            }
        });

        let path_changed = src_rel.as_ref().is_some_and(|s| *s != mat_rel);
        let adaptation = if final_suffix_bytes != orig_suffix || path_changed {
            Some(StreamAdaptationRecord {
                stream_index,
                source_stream_name: orig_suffix.to_vec(),
                materialized_stream_name: final_suffix_bytes,
                source_relative_path: src_rel,
                materialized_relative_path: Some(mat_rel),
            })
        } else {
            None
        };

        out.push(PlannedWindowsAdsCompanion {
            stream_index,
            companion_path,
            stream: stream.clone(),
            adaptation,
            warning: adaptation_warning,
        });
    }

    Ok(out)
}

/// Computes the destination paths of all Windows ADS `<filename>:<stream_name>` companion
/// files that would be written for `entity` at `dest_path`.
pub fn windows_ads_companion_paths(
    entity: &FileEntity,
    dest_path: &Path,
    dest_dir: &crate::sandboxable_dir::SandboxableDir,
    strict_lossless: bool,
) -> Result<Vec<(PathBuf, AttachedStream)>> {
    let planned = plan_windows_ads_companions(entity, dest_path, dest_dir, strict_lossless)?;
    Ok(planned
        .into_iter()
        .map(|p| (p.companion_path, p.stream))
        .collect())
}

fn build_windows_ads_companion_os_string(
    base_name: &std::ffi::OsStr,
    stream_suffix: &[u8],
) -> Result<std::ffi::OsString> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::{OsStrExt, OsStringExt};
        let mut combined = Vec::with_capacity(
            base_name
                .as_bytes()
                .len()
                .saturating_add(1)
                .saturating_add(stream_suffix.len()),
        );
        combined.extend_from_slice(base_name.as_bytes());
        combined.push(b':');
        combined.extend_from_slice(stream_suffix);
        Ok(std::ffi::OsString::from_vec(combined))
    }
    #[cfg(not(unix))]
    {
        let suffix_str = std::str::from_utf8(stream_suffix)
            .context("Non-UTF-8 stream name cannot be materialized as companion filename on this platform")?;
        let mut combined = base_name.to_os_string();
        combined.push(":");
        combined.push(suffix_str);
        Ok(combined)
    }
}

/// Writes alongside `<filename>:<stream_name>` Windows ADS companion files for `entity`.
pub fn write_windows_ads_companions(
    entity: &FileEntity,
    dest_path: &Path,
    dest_dir: &crate::sandboxable_dir::SandboxableDir,
    existing: crate::sandboxable_dir::ExistingEntryPolicy,
    strict_lossless: bool,
    stream_source: &dyn crate::streams::AttachedStreamSource,
) -> Result<Vec<PathBuf>> {
    let (paths, _adaptations, _warnings) = write_windows_ads_companions_with_warnings(
        entity,
        dest_path,
        dest_dir,
        existing,
        strict_lossless,
        stream_source,
    )?;
    Ok(paths)
}

/// Writes alongside `<filename>:<stream_name>` Windows ADS companion files for `entity`,
/// returning `(written_paths, stream_adaptations, warnings)`.
pub fn write_windows_ads_companions_with_warnings(
    entity: &FileEntity,
    dest_path: &Path,
    dest_dir: &crate::sandboxable_dir::SandboxableDir,
    existing: crate::sandboxable_dir::ExistingEntryPolicy,
    strict_lossless: bool,
    stream_source: &dyn crate::streams::AttachedStreamSource,
) -> Result<(Vec<PathBuf>, Vec<StreamAdaptationRecord>, Vec<String>)> {
    let times = if let Some(timestamps) = &entity.metadata.timestamps {
        timestamps.validate()?;
        Some((
            crate::metadata::checked_file_time(timestamps.atime_sec, timestamps.atime_nsec)
                .context("Invalid native Windows ADS companion access time")?,
            crate::metadata::checked_file_time(timestamps.mtime_sec, timestamps.mtime_nsec)
                .context("Invalid native Windows ADS companion modification time")?,
        ))
    } else {
        None
    };

    let planned = plan_windows_ads_companions(entity, dest_path, dest_dir, strict_lossless)?;
    if planned.is_empty() {
        return Ok((Vec::new(), Vec::new(), Vec::new()));
    }

    let mut written_paths = Vec::with_capacity(planned.len());
    let mut adaptations = Vec::new();
    let mut warnings = Vec::new();
    // Reason for fallback: destination path without a parent directory defaults to current working directory "."
    let parent_dir = dest_path.parent().unwrap_or(Path::new("."));
    // Reason for fallback: destination path without a file name component defaults to verbatim OsStr
    let file_name = dest_path.file_name().unwrap_or(dest_path.as_os_str());
    let max_suffix_bytes = crate::clean_name::MAX_FILENAME_BYTES
        .saturating_sub(file_name.as_encoded_bytes().len().saturating_add(1))
        .max(1);
    for mut item in planned {
        if !strict_lossless
            && existing == crate::sandboxable_dir::ExistingEntryPolicy::Refuse
            && std::fs::symlink_metadata(&item.companion_path).is_ok()
        {
            // Reason for fallback: companion without an adaptation has no recorded original source stream name
            let orig_suffix = item
                .adaptation
                .as_ref()
                .map_or_else(Vec::new, |a| a.source_stream_name.clone());
            // Reason for fallback: companion without an adaptation uses lossy original suffix as disambiguation baseline
            let base_cleaned = item
                .adaptation
                .as_ref()
                .map_or_else(
                    || String::from_utf8_lossy(&orig_suffix).into_owned(),
                    |a| String::from_utf8_lossy(&a.materialized_stream_name).into_owned(),
                );
            let mut collision_idx = 1_usize;
            loop {
                let disambig = crate::clean_name::disambiguate_cleaned_name(
                    &base_cleaned,
                    &orig_suffix,
                    collision_idx,
                    max_suffix_bytes,
                );
                let cand_os = build_windows_ads_companion_os_string(file_name, disambig.as_bytes())?;
                let cand_path = parent_dir.join(&cand_os);
                if std::fs::symlink_metadata(&cand_path).is_err() {
                    let rel = cand_path
                        .strip_prefix(dest_dir.root_path())
                        .context("Windows ADS companion is outside the destination sandbox")?
                        .to_path_buf();
                    let w = format!(
                        "Disambiguated colliding Windows ADS companion {:?} -> {:?} on {} (best-effort)",
                        item.companion_path.display(),
                        cand_path.display(),
                        dest_path.display()
                    );
                    item.companion_path = cand_path;
                    item.warning = Some(match item.warning {
                        Some(prev) => format!("{prev}; {w}"),
                        None => w,
                    });
                    item.adaptation = Some(StreamAdaptationRecord {
                        stream_index: item.stream_index,
                        source_stream_name: orig_suffix,
                        materialized_stream_name: disambig.into_bytes(),
                        source_relative_path: item
                            .adaptation
                            .and_then(|a| a.source_relative_path),
                        materialized_relative_path: Some(rel),
                    });
                    break;
                }
                collision_idx = collision_idx.saturating_add(1);
            }
        }
        let companion_path = item.companion_path;
        let stream = item.stream;
        if let Some(w) = item.warning {
            warn_fmt!("{w}");
            warnings.push(w);
        }
        if let Some(adapt) = item.adaptation {
            adaptations.push(adapt);
        }
        let FileEntityKind::Regular {
            size: expected_size,
            sha256: expected_sha256,
            ..
        } = &stream.entity.kind
        else {
            bail!("Attached stream is not a regular payload");
        };

        let relative = companion_path
            .strip_prefix(dest_dir.root_path())
            .context("Windows ADS companion is outside the destination sandbox")?;
        let (parent, final_name) = dest_dir.ensure_parent_dir(
            relative,
            crate::path_policy::PathTraversalPolicy::StrictSandboxed,
        )?;
        let temp_name = format!(".ctb-windows-ads-{}", uuid());
        let mut staged = dest_dir.create_temp_file(&parent.as_fd(), &temp_name, 0o600)?;
        let result = (|| -> Result<()> {
            let mut reader = stream_source.open(&stream)?;
            let mut hasher = ctb_formats_checksum::Sha256Stream::new();
            let mut total_written = 0_u64;
            let mut buf = [0_u8; 64 * 1024];
            loop {
                let n = std::io::Read::read(&mut reader, &mut buf)
                    .context("Failed to read Windows ADS stream source")?;
                if n == 0 {
                    break;
                }
                let chunk = buf.get(..n).context("Invalid slice range")?;
                hasher.update(chunk);
                total_written = total_written
                    .checked_add(u64::try_from(n)?)
                    .context("Stream size overflow")?;
                std::io::Write::write_all(&mut staged, chunk)
                    .context("Failed to stage Windows ADS companion bytes")?;
            }
            ensure!(
                total_written == *expected_size && hasher.finalize() == *expected_sha256,
                "Attached stream payload does not match its descriptor: {:?}",
                stream.to_string_lossy()
            );
            if let Some((atime, mtime)) = times {
                filetime::set_file_handle_times(&staged, Some(atime), Some(mtime))
                    .context("Failed to stage Windows ADS companion timestamps")?;
            }
            staged
                .sync_all()
                .context("Failed to sync staged Windows ADS companion")?;
            dest_dir.commit_atomic_file(&parent.as_fd(), &temp_name, &final_name, existing)
        })();
        drop(staged);
        if let Err(error) = result {
            #[cfg(unix)]
            let cleanup = rustix::fs::unlinkat(
                parent.as_fd(),
                temp_name.as_str(),
                rustix::fs::AtFlags::empty(),
            );
            #[cfg(not(unix))]
            let cleanup = std::fs::remove_file(
                companion_path
                    .parent()
                    .context("Windows ADS companion has no parent")?
                    .join(&temp_name),
            );
            if let Err(cleanup) = cleanup {
                return Err(error).context(format!(
                    "Failed to remove staged Windows ADS companion: {cleanup}"
                ));
            }
            return Err(error);
        }
        written_paths.push(companion_path);
    }

    Ok((written_paths, adaptations, warnings))
}

fn read_captured_apple_input(path: &Path, snapshot: Option<&crate::payload::SourceSnapshot>, single_only: bool) -> Result<Option<Vec<u8>>> {
    use crate::payload::PayloadSource;
    use std::io::Read;
    let mut payload = crate::payload::DiskPayloadSource::open(path)?;
    if let Some(snapshot) = snapshot { payload.bind_snapshot(snapshot)?; }
    let mut data = Vec::new();
    let selected = if single_only {
        (&mut payload).take(4).read_to_end(&mut data)?;
        matches!(<[u8; 4]>::try_from(data.as_slice()).map(u32::from_be_bytes), Ok(APPLESINGLE_MAGIC_BE | APPLESINGLE_MAGIC_LE))
    } else { true };
    if selected { payload.read_to_end(&mut data)?; }
    let warnings = payload.finish()?;
    if let Some(snapshot) = snapshot { snapshot.retain_cleanup_warnings(warnings)?; }
    else { ensure!(warnings.is_empty(), "Apple input cleanup diagnostics: {}", warnings.join("; ")); }
    Ok(if selected { Some(data) } else { None })
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
    if let Some(ref finfo) = archive.finder_info {
        if finfo.window_bounds.is_some() && archive.data_fork.is_none() {
            if let FileEntityKind::Regular { size: 0, .. } = entity.kind {
                entity.kind = FileEntityKind::Directory;
            }
        }
    }
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
    use crate::sandboxable_dir::{ExistingEntryPolicy, SandboxableDir};

    #[crate::ctb_test]
    fn test_apple_double_companion_publication_policies() -> Result<()> {
        for style in [AppleDoubleStyle::Alongside, AppleDoubleStyle::Zip, AppleDoubleStyle::Netatalk] {
            let temp = tempfile::tempdir()?;
            let path = temp.path().join("file");
            std::fs::write(&path, b"primary")?;
            let sandbox = SandboxableDir::open(temp.path())?;
            let mut entity = FileEntity::from_filesystem(&path, None)?;
            entity.metadata.apple = Some(AppleMetadata { comment: Some("replacement".into()), ..Default::default() });
            let timestamps = entity.metadata.timestamps.as_mut().unwrap();
            timestamps.atime_sec = 1_700_000_000;
            timestamps.atime_nsec = 123_456_789;
            timestamps.mtime_sec = 1_700_000_001;
            timestamps.mtime_nsec = 987_654_321;
            let companion = apple_double_companion_path(&entity, &path, &sandbox, style)?.unwrap();
            std::fs::create_dir_all(companion.parent().unwrap())?;
            std::fs::write(&companion, b"original")?;
            let before = std::fs::symlink_metadata(&companion)?;
            assert!(write_apple_double_companion(&entity, &path, &sandbox, style, ExistingEntryPolicy::Refuse).is_err());
            assert_eq!(std::fs::read(&companion)?, b"original");
            assert_eq!(std::fs::metadata(&companion)?.modified()?, before.modified()?);
            assert_eq!(std::fs::read_dir(companion.parent().unwrap())?.count(),
                if style == AppleDoubleStyle::Alongside { 2 } else { 1 });
            let expected = serialize_apple_double_for_entity(&entity)?;
            assert_eq!(write_apple_double_companion(&entity, &path, &sandbox, style, ExistingEntryPolicy::Replace)?, Some(companion.clone()));
            let after = std::fs::metadata(&companion)?;
            let (expected_atime_nsec, expected_mtime_nsec) = if cfg!(windows) {
                ((123_456_789 / 100) * 100, (987_654_321 / 100) * 100)
            } else {
                (123_456_789, 987_654_321)
            };
            assert_eq!(filetime::FileTime::from_last_access_time(&after), filetime::FileTime::from_unix_time(1_700_000_000, expected_atime_nsec));
            assert_eq!(filetime::FileTime::from_last_modification_time(&after), filetime::FileTime::from_unix_time(1_700_000_001, expected_mtime_nsec));
            assert_eq!(std::fs::read(&companion)?, expected);
            assert_eq!(std::fs::read(&path)?, b"primary");
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                assert_ne!(before.ino(), after.ino());
            }
        }
        Ok(())
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_apple_double_companion_symlink_publication() -> Result<()> {
        for dangling in [false, true] {
            let temp = tempfile::tempdir()?;
            let path = temp.path().join("file");
            let target = temp.path().join("target");
            let companion = temp.path().join("._file");
            std::fs::write(&path, b"primary")?;
            if !dangling { std::fs::write(&target, b"target original")?; }
            std::os::unix::fs::symlink(&target, &companion)?;
            let sandbox = SandboxableDir::open(temp.path())?;
            let options = AppleReadOptions { read_apple_double_alongside: false, ..Default::default() };
            let mut entity = FileEntity::from_filesystem_with_apple_options(&path, None, &options)?;
            entity.metadata.apple = Some(AppleMetadata { comment: Some("replacement".into()), ..Default::default() });
            assert!(write_apple_double_companion(&entity, &path, &sandbox, AppleDoubleStyle::Alongside, ExistingEntryPolicy::Refuse).is_err());
            assert_eq!(std::fs::read_link(&companion)?, target);
            assert_eq!(std::fs::read(&path)?, b"primary");
            if dangling { assert!(!target.try_exists()?); }
            else { assert_eq!(std::fs::read(&target)?, b"target original"); }
            write_apple_double_companion(&entity, &path, &sandbox, AppleDoubleStyle::Alongside, ExistingEntryPolicy::Replace)?;
            assert!(!std::fs::symlink_metadata(&companion)?.file_type().is_symlink());
            assert_eq!(std::fs::read(&companion)?, serialize_apple_double_for_entity(&entity)?);
            assert_eq!(std::fs::read(&path)?, b"primary");
            if dangling { assert!(!target.try_exists()?); }
            else { assert_eq!(std::fs::read(&target)?, b"target original"); }
            assert_eq!(std::fs::read_dir(temp.path())?.count(), if dangling { 2 } else { 3 });
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_apple_double_invalid_metadata_preserves_originals() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let path = temp.path().join("file");
        let companion = temp.path().join("._file");
        std::fs::write(&path, b"primary")?;
        std::fs::write(&companion, b"original")?;
        let sandbox = SandboxableDir::open(temp.path())?;
        let options = AppleReadOptions { read_apple_double_alongside: false, ..Default::default() };
        let mut entity = FileEntity::from_filesystem_with_apple_options(&path, None, &options)?;
        entity.metadata.apple = Some(AppleMetadata { comment: Some("replacement".into()), ..Default::default() });
        for existing in [ExistingEntryPolicy::Refuse, ExistingEntryPolicy::Replace] {
            for overflow in [false, true] {
                let mut invalid = entity.clone();
                let timestamps = invalid.metadata.timestamps.as_mut().unwrap();
                if overflow { timestamps.mtime_sec = i64::MIN; }
                else { timestamps.mtime_nsec = 1_000_000_000; }
                assert!(write_apple_double_companion(&invalid, &path, &sandbox, AppleDoubleStyle::Alongside, existing).is_err());
                assert_eq!(std::fs::read(&companion)?, b"original");
                assert_eq!(std::fs::read(&path)?, b"primary");
                assert_eq!(std::fs::read_dir(temp.path())?.count(), 2);
            }
        }
        entity.identity.relative_path = Some(PathBuf::from("../../file"));
        assert!(write_apple_double_companion(&entity, &path, &sandbox, AppleDoubleStyle::Zip, ExistingEntryPolicy::Replace).is_err());
        assert_eq!(std::fs::read(&companion)?, b"original");
        assert_eq!(std::fs::read(&path)?, b"primary");
        assert_eq!(std::fs::read_dir(temp.path())?.count(), 2);
        Ok(())
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_apple_double_fresh_raw_companion_names() -> Result<()> {
        use std::os::unix::ffi::{OsStrExt, OsStringExt};
        for (style, expected) in [
            (AppleDoubleStyle::Alongside, b"._raw\xff".as_slice()),
            (AppleDoubleStyle::Zip, b"__MACOSX/._raw\xff".as_slice()),
            (AppleDoubleStyle::Netatalk, b".AppleDouble/raw\xff".as_slice()),
        ] {
            let temp = tempfile::tempdir()?;
            let name = std::ffi::OsString::from_vec(b"raw\xff".to_vec());
            let path = temp.path().join(&name);
            std::fs::write(&path, b"primary")?;
            let sandbox = SandboxableDir::open(temp.path())?;
            let mut entity = FileEntity::from_filesystem(&path, None)?;
            entity.identity.relative_path = Some(PathBuf::from(&name));
            entity.metadata.apple = Some(AppleMetadata { comment: Some("raw".into()), ..Default::default() });
            let companion = write_apple_double_companion(&entity, &path, &sandbox, style, ExistingEntryPolicy::Refuse)?.unwrap();
            assert_eq!(companion.strip_prefix(temp.path())?.as_os_str().as_bytes(), expected);
            assert_eq!(std::fs::read(&companion)?, serialize_apple_double_for_entity(&entity)?);
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_apple_double_rejects_native_timestamp_overflow_before_write() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("file");
        std::fs::write(&path, b"payload").unwrap();
        let mut entity = FileEntity::from_filesystem(&path, None).unwrap();
        entity.metadata.flags = Some(vec![FileFlag::Invisible]);
        entity.metadata.timestamps.as_mut().unwrap().atime_sec = i64::MIN;
        let sandbox = crate::sandboxable_dir::SandboxableDir::open(temp.path()).unwrap();
        let error = write_apple_double_companion(
            &entity, &path, &sandbox, AppleDoubleStyle::Alongside,
            crate::sandboxable_dir::ExistingEntryPolicy::Replace,
        ).unwrap_err();
        assert!(format!("{error:#}").contains("native conversion range"));
        assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 1);
        assert_eq!(std::fs::read(&path).unwrap(), b"payload");
    }

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

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_windows_ads_fixture_read_and_write_roundtrip() -> Result<()> {
        let fixture_dir = ctb_utilities::manifest_fixture_path(
            env!("CARGO_MANIFEST_DIR"),
            "windows_ads",
        );
        let pan_path = fixture_dir.join("example2 with lemurs.pan");
        let pan_zone_path = fixture_dir.join("example2 with lemurs.pan:Zone.Identifier");
        let bin_path = fixture_dir.join("example2 with lemurs-edited.bin");
        let bin_zone_path = fixture_dir.join("example2 with lemurs-edited.bin:Zone.Identifier");

        let expected_pan_zone = std::fs::read(&pan_zone_path)?;
        let expected_bin_zone = std::fs::read(&bin_zone_path)?;

        let ads_opts = AppleReadOptions {
            read_windows_ads_alongside: true,
            ..Default::default()
        };
        let entity_pan = FileEntity::from_filesystem_with_apple_options(
            &pan_path,
            Some(&fixture_dir),
            &ads_opts,
        )?;
        let zone_stream = entity_pan
            .streams
            .iter()
            .find(|s| {
                s.name
                    .as_ref()
                    .is_some_and(|n| n.as_bytes().as_ref() == b"Zone.Identifier")
            })
            .context("Missing Zone.Identifier stream on example2 with lemurs.pan")?;
        ensure!(zone_stream.kind == StreamKind::NtfsAlternateDataStream);
        ensure!(zone_stream.data.as_deref() == Some(expected_pan_zone.as_slice()));

        let entity_bin = FileEntity::from_filesystem_with_apple_options(
            &bin_path,
            Some(&fixture_dir),
            &ads_opts,
        )?;
        let bin_stream = entity_bin
            .streams
            .iter()
            .find(|s| {
                s.name
                    .as_ref()
                    .is_some_and(|n| n.as_bytes().as_ref() == b"Zone.Identifier")
            })
            .context("Missing Zone.Identifier stream on example2 with lemurs-edited.bin")?;
        ensure!(bin_stream.kind == StreamKind::NtfsAlternateDataStream);
        ensure!(bin_stream.data.as_deref() == Some(expected_bin_zone.as_slice()));

        // Verify default AppleReadOptions (read_windows_ads_alongside: false) does not join the companion
        let entity_no_ads = FileEntity::from_filesystem(&pan_path, Some(&fixture_dir))?;
        ensure!(
            !entity_no_ads.streams.iter().any(|s| {
                s.name
                    .as_ref()
                    .is_some_and(|n| n.as_bytes().as_ref() == b"Zone.Identifier")
            }),
            "Zone.Identifier should not be joined when read_windows_ads_alongside is false"
        );

        // Materialize with ForceWindowsAds into a fresh directory and verify companion creation
        let temp = tempfile::tempdir()?;
        let dest_pan = temp.path().join("example2 with lemurs.pan");
        std::fs::write(&dest_pan, std::fs::read(&pan_path)?)?;
        let sandbox = SandboxableDir::open(temp.path())?;
        let written = write_windows_ads_companions(
            &entity_pan,
            &dest_pan,
            &sandbox,
            ExistingEntryPolicy::Refuse,
            true,
            &crate::streams::FilesystemStreamSource,
        )?;
        let expected_companion = temp.path().join("example2 with lemurs.pan:Zone.Identifier");
        ensure!(written == vec![expected_companion.clone()]);
        ensure!(std::fs::read(&expected_companion)? == expected_pan_zone);
        ensure!(
            write_windows_ads_companions(
                &entity_pan,
                &dest_pan,
                &sandbox,
                ExistingEntryPolicy::Refuse,
                true,
                &crate::streams::FilesystemStreamSource,
            )
            .is_err(),
            "Refuse policy must reject overwriting existing Windows ADS companion"
        );
        write_windows_ads_companions(
            &entity_pan,
            &dest_pan,
            &sandbox,
            ExistingEntryPolicy::Replace,
            true,
            &crate::streams::FilesystemStreamSource,
        )?;

        Ok(())
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_windows_ads_stream_name_cleaning_collisions_and_directory_companions() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let src_dir = temp.path().join("src");
        let dest_dir_path = temp.path().join("dest");
        std::fs::create_dir_all(&src_dir)?;
        std::fs::create_dir_all(&dest_dir_path)?;

        // 1. Directory with an alongside `<dirname>:Zone.Identifier` companion
        let sub_dir = src_dir.join("my_folder");
        std::fs::create_dir(&sub_dir)?;
        let dir_zone_bytes = b"[ZoneTransfer]\r\nZoneId=3\r\n";
        std::fs::write(
            src_dir.join("my_folder:Zone.Identifier"),
            dir_zone_bytes,
        )?;

        let ads_opts = AppleReadOptions {
            read_windows_ads_alongside: true,
            ..Default::default()
        };
        let dir_entity = FileEntity::from_filesystem_with_apple_options(
            &sub_dir,
            Some(&src_dir),
            &ads_opts,
        )?;
        ensure!(dir_entity.is_dir());
        let dir_stream = dir_entity
            .streams
            .iter()
            .find(|s| {
                s.name
                    .as_ref()
                    .is_some_and(|n| n.as_bytes().as_ref() == b"Zone.Identifier")
            })
            .context("Directory must join my_folder:Zone.Identifier stream")?;
        ensure!(dir_stream.data.as_deref() == Some(dir_zone_bytes.as_slice()));

        // Materialize directory with ForceWindowsAds
        let dest_sandbox = SandboxableDir::open(&dest_dir_path)?;
        let mat_opts = crate::materializer::MaterializeOptions {
            strict_lossless: false,
            apple_write_mode: AppleWriteMode::ForceWindowsAds,
            ..Default::default()
        };
        let dir_receipt = crate::materializer::materialize_entity(
            &dir_entity,
            None,
            &dest_sandbox,
            &mat_opts,
        )?;
        ensure!(dir_receipt.destination_path == dest_dir_path.join("my_folder"));
        ensure!(
            std::fs::read(dest_dir_path.join("my_folder:Zone.Identifier"))?
                == dir_zone_bytes
        );
        crate::verifier::verify_materialized_entity(
            &dir_receipt.destination_path,
            &dir_entity,
            false,
        )?;

        // 2. Unrepresentable and colliding stream names:
        //    - slash/backslash (`a/b`, `a\b` -> collide on `a⌿b`!)
        //    - NUL and colon (`bad\0:name`)
        //    - invalid UTF-8 binary bytes (`bin\xff\xfe`)
        //    - unnamed non-resource-fork stream (`None`)
        let src_file = src_dir.join("sample.txt");
        std::fs::write(&src_file, b"hello")?;
        let mut entity = FileEntity::from_filesystem(&src_file, Some(&src_dir))?;
        entity.streams = vec![
            AttachedStream::from_data(
                Some(crate::streams::StreamName::from_bytes(b"a/b")),
                StreamKind::NtfsAlternateDataStream,
                b"slash-stream".to_vec(),
            )?,
            AttachedStream::from_data(
                Some(crate::streams::StreamName::from_bytes(b"a\\b")),
                StreamKind::NtfsAlternateDataStream,
                b"backslash-stream".to_vec(),
            )?,
            AttachedStream::from_data(
                Some(crate::streams::StreamName::from_bytes(b"bad\0:name")),
                StreamKind::NtfsAlternateDataStream,
                b"nul-colon-stream".to_vec(),
            )?,
            AttachedStream::from_data(
                Some(crate::streams::StreamName::from_bytes(b"bin\xff\xfe")),
                StreamKind::NtfsAlternateDataStream,
                b"binary-name-stream".to_vec(),
            )?,
            AttachedStream::from_data(
                None,
                StreamKind::NtfsAlternateDataStream,
                b"unnamed-stream".to_vec(),
            )?,
        ];

        // In strict_lossless mode, materialization MUST fail on each unrepresentable stream
        let strict_opts = crate::materializer::MaterializeOptions {
            strict_lossless: true,
            apple_write_mode: AppleWriteMode::ForceWindowsAds,
            ..Default::default()
        };
        let mut payload = crate::payload::MemoryPayloadSource::new(b"hello".to_vec())?;
        ensure!(
            crate::materializer::materialize_entity(
                &entity,
                Some(&mut payload),
                &dest_sandbox,
                &strict_opts,
            )
            .is_err(),
            "Strict lossless mode must reject unrepresentable Windows ADS stream names"
        );

        // In best-effort mode (`strict_lossless: false`), materialization MUST clean & disambiguate
        // all 5 streams, record adaptations, and pass verification!
        let mut payload = crate::payload::MemoryPayloadSource::new(b"hello".to_vec())?;
        let receipt = crate::materializer::materialize_entity(
            &entity,
            Some(&mut payload),
            &dest_sandbox,
            &mat_opts,
        )?;
        ensure!(receipt.stream_adaptations.len() == 5);
        ensure!(!receipt.warnings.is_empty());

        let mut recorded_entity = entity.clone();
        receipt.apply_stream_adaptations(&mut recorded_entity);
        for stream in &recorded_entity.streams {
            ensure!(
                stream.entity.metadata.materialized_stream_name().is_some(),
                "Adapted stream must record materialized.stream_name in metadata"
            );
            ensure!(
                stream.entity.metadata.materialized_relative_path().is_some(),
                "Adapted stream must record materialized.relative_path in metadata"
            );
        }

        // Verify both with recorded_entity (as stored in journal) and raw entity (best-effort)
        crate::verifier::verify_materialized_entity(
            &receipt.destination_path,
            &recorded_entity,
            false,
        )?;
        crate::verifier::verify_materialized_entity(
            &receipt.destination_path,
            &entity,
            false,
        )?;

        // 3. Overlong base filename (255 bytes) + stream in best-effort mode must truncate base
        //    filename so `<base>:<stream>` fits within 255 bytes.
        let mut long_entity = entity.clone();
        let long_name = "L".repeat(255);
        long_entity.identity.relative_path = Some(PathBuf::from(&long_name));
        long_entity.streams = vec![AttachedStream::from_data(
            Some(crate::streams::StreamName::from_bytes(b"Zone.Identifier")),
            StreamKind::NtfsAlternateDataStream,
            b"zone-for-long".to_vec(),
        )?];
        let mut payload = crate::payload::MemoryPayloadSource::new(b"hello".to_vec())?;
        ensure!(
            crate::materializer::materialize_entity(
                &long_entity,
                Some(&mut payload),
                &dest_sandbox,
                &strict_opts,
            )
            .is_err(),
            "Strict lossless mode must reject >255 byte combined companion filename"
        );
        let mut payload = crate::payload::MemoryPayloadSource::new(b"hello".to_vec())?;
        let long_receipt = crate::materializer::materialize_entity(
            &long_entity,
            Some(&mut payload),
            &dest_sandbox,
            &mat_opts,
        )?;
        let mat_base_len = long_receipt
            .destination_path
            .file_name()
            .unwrap()
            .as_encoded_bytes()
            .len();
        ensure!(mat_base_len + 1 + "Zone.Identifier".len() <= crate::clean_name::MAX_FILENAME_BYTES);
        crate::verifier::verify_materialized_entity(
            &long_receipt.destination_path,
            &long_entity,
            false,
        )?;

        Ok(())
    }
}
