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

use crate::file::entity::FileEntity;
pub use crate::file::metadata::AppleMetadata;
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
