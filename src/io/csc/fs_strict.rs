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

//! Strict zero-data-loss validation, metadata preservation, and stream handling.
//!
//! Delegates low-level filesystem representation and operations to `ctb_io::file`.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

pub use ctb_io::file::Extent as FileExtent;
pub use ctb_io::file::get_file_extents;
pub use ctb_io::file::verify_filename_exact_bytes;

use ctb_io::file::entity::{FileEntity, FileEntityKind};
use ctb_io::file::identity::{FileIdentity, FileOrigin};
use ctb_io::file::materializer::apply_entity_metadata;
use ctb_io::file::metadata::{FileMetadata, FileTimestamps};
use ctb_io::file::streams::{AttachedStream, StreamKind, StreamName};
use std::ffi::OsString;
use std::fs::Metadata;
use std::os::unix::ffi::OsStringExt;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

/// Information about an extended attribute, ACL, or alternate stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamInfo {
    /// The name of the stream or extended attribute (e.g. `user.DosStream.foo`).
    pub name: OsString,
    /// Size of the stream payload in bytes.
    pub size: u64,
    /// Cryptographic SHA-256 digest of the stream payload.
    pub sha256: [u8; 32],
}

/// Reads all extended attributes, ACLs, security labels, and alternate streams
/// from `path`, computing their SHA-256 checksums.
pub fn read_and_hash_streams(path: &Path) -> Result<Vec<(StreamInfo, Vec<u8>)>> {
    let attached = ctb_io::file::read_and_hash_streams(path)?;
    let mut streams = Vec::with_capacity(attached.len());

    for s in attached {
        let (size, sha256) = match &s.entity.kind {
            FileEntityKind::Regular { size, sha256, .. } => (*size, *sha256),
            _ => (0, [0_u8; 32]),
        };
        // Reason for fallback: AttachedStream in-memory data payload is optional; an absent buffer represents an empty byte payload.
        let data = s.data.unwrap_or_default();
        streams.push((
            StreamInfo {
                name: OsString::from_vec(s.name.0),
                size,
                sha256,
            },
            data,
        ));
    }

    Ok(streams)
}

/// Writes all streams (xattrs, ACLs, security labels) to `dest`.
/// If the destination filesystem cannot store them, fails with a hard error.
pub fn write_streams(dest: &Path, streams: &[(StreamInfo, Vec<u8>)]) -> Result<()> {
    let mut attached = Vec::with_capacity(streams.len());
    for (info, val) in streams {
        let name_bytes = info.name.as_encoded_bytes().to_vec();
        let stream_name = StreamName(name_bytes.clone());
        let kind = StreamKind::infer_from_name(&name_bytes);
        let size = u64::try_from(val.len())?;

        let entity = FileEntity {
            identity: FileIdentity {
                origin: FileOrigin::Synthetic,
                relative_path: PathBuf::from(&info.name),
                enclosing_path: None,
                raw_relative_path: name_bytes.clone(),
                raw_filename: name_bytes,
                nlink: 1,
                hardlink_group: None,
            },
            metadata: FileMetadata {
                mode: 0o644,
                uid: 0,
                gid: 0,
                timestamps: FileTimestamps {
                    atime_sec: 0,
                    atime_nsec: 0,
                    mtime_sec: 0,
                    mtime_nsec: 0,
                    ctime_sec: 0,
                    ctime_nsec: 0,
                    birthtime_sec: None,
                    birthtime_nsec: None,
                },
                flags: Vec::new(),
                platform_raw_flags: None,
                read_time: None,
            },
            kind: FileEntityKind::Regular {
                size,
                sha256: info.sha256,
                is_sparse: false,
                extents: Vec::new(),
            },
            streams: Vec::new(),
        };

        attached.push(AttachedStream {
            name: stream_name,
            kind,
            entity: Box::new(entity),
            data: Some(val.clone()),
        });
    }

    ctb_io::file::write_streams(dest, &attached, true)
}

/// Applies permissions, ownership, and timestamps from `source_meta` to `dest`.
/// Fails with a hard error if ownership or permissions cannot be preserved.
pub fn apply_metadata(dest: &Path, source_meta: &Metadata, is_symlink: bool) -> Result<()> {
    let mode = source_meta.permissions().mode();
    let uid = source_meta.uid();
    let gid = source_meta.gid();
    let atime_sec = source_meta.atime();
    let atime_nsec = u32::try_from(source_meta.atime_nsec())
        .context("Failed to convert atime_nsec to u32")?;
    let mtime_sec = source_meta.mtime();
    let mtime_nsec = u32::try_from(source_meta.mtime_nsec())
        .context("Failed to convert mtime_nsec to u32")?;
    let ctime_sec = source_meta.ctime();
    let ctime_nsec = u32::try_from(source_meta.ctime_nsec())
        .context("Failed to convert ctime_nsec to u32")?;

    let meta = FileMetadata {
        mode,
        uid,
        gid,
        timestamps: FileTimestamps {
            atime_sec,
            atime_nsec,
            mtime_sec,
            mtime_nsec,
            ctime_sec,
            ctime_nsec,
            birthtime_sec: None,
            birthtime_nsec: None,
        },
        flags: Vec::new(),
        platform_raw_flags: None,
        read_time: None,
    };

    apply_entity_metadata(dest, &meta, is_symlink, true, true)
}
