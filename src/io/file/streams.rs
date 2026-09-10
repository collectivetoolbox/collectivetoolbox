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

//! Alternate data streams, resource forks, and extended attributes.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::file::entity::{FileEntity, FileEntityKind};
use crate::file::identity::{FileIdentity, FileOrigin};
use crate::file::metadata::{FileMetadata, FileTimestamps};
use crate::file::payload::Extent;
use ctb_formats_checksum::Sha256Stream;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

/// Arbitrary byte stream or extended attribute name.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StreamName(pub Vec<u8>);

impl StreamName {
    /// Creates a stream name from raw bytes.
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self(bytes.to_vec())
    }

    /// Creates a stream name from a UTF-8 string slice.
    #[must_use]
    pub fn from_str(s: &str) -> Self {
        Self(s.as_bytes().to_vec())
    }

    /// Accesses the underlying byte slice.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Lossy UTF-8 representation for diagnostics and logging.
    #[must_use]
    pub fn to_string_lossy(&self) -> std::borrow::Cow<'_, str> {
        String::from_utf8_lossy(&self.0)
    }

    /// Converts to an `OsStr` reference.
    #[must_use]
    pub fn as_os_str(&self) -> &OsStr {
        OsStr::from_bytes(&self.0)
    }
}

/// The classification of an attached stream or fork.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamKind {
    /// Standard extended attribute (`user.*`).
    ExtendedAttribute,
    /// Apple macOS resource fork (`com.apple.ResourceFork` or `..namedfork/rsrc`).
    MacOsResourceFork,
    /// NTFS alternate data stream (`:stream`).
    NtfsAlternateDataStream,
    /// POSIX access control list.
    PosixAclAccess,
    /// POSIX default access control list.
    PosixAclDefault,
    /// Security label (SELinux, AppArmor, MAC).
    SecurityLabel,
}

impl StreamKind {
    /// Infers the stream kind from its byte name.
    #[must_use]
    pub fn infer_from_name(name: &[u8]) -> Self {
        if name == b"com.apple.ResourceFork" {
            Self::MacOsResourceFork
        } else if name.starts_with(b"system.posix_acl_access") {
            Self::PosixAclAccess
        } else if name.starts_with(b"system.posix_acl_default") {
            Self::PosixAclDefault
        } else if name.starts_with(b"security.") {
            Self::SecurityLabel
        } else {
            Self::ExtendedAttribute
        }
    }
}

/// An alternate stream, resource fork, or extended attribute attached to a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachedStream {
    /// Raw byte name of the stream.
    pub name: StreamName,
    /// Classification of the stream.
    pub kind: StreamKind,
    /// The stream represented as a full `FileEntity`.
    pub entity: Box<FileEntity>,
    /// In-memory payload data, if loaded.
    pub data: Option<Vec<u8>>,
}

/// Reads all extended attributes, resource forks, and security labels from `path`.
pub fn read_and_hash_streams(path: &Path) -> Result<Vec<AttachedStream>> {
    let mut streams = Vec::new();

    let xattr_names = match xattr::list(path) {
        Ok(iter) => iter,
        Err(e) => {
            if e.raw_os_error() == Some(nix::libc::ENOTSUP)
                || e.raw_os_error() == Some(nix::libc::EOPNOTSUPP)
            {
                return Ok(streams);
            }
            return Err(e).with_context(|| {
                format!("Failed to list xattrs/streams for {}", path.display())
            });
        }
    };

    for name_os in xattr_names {
        let name_bytes = name_os.as_bytes().to_vec();
        let val = match xattr::get(path, &name_os) {
            Ok(Some(v)) => v,
            Ok(None) => continue,
            Err(e) => {
                return Err(e).with_context(|| {
                    format!(
                        "Failed to read stream {:?} on {}",
                        String::from_utf8_lossy(&name_bytes),
                        path.display()
                    )
                });
            }
        };

        let mut hasher = Sha256Stream::new();
        hasher.update(&val);
        let sha256 = hasher.finalize();
        let size = u64::try_from(val.len())?;

        let stream_name = StreamName(name_bytes.clone());
        let kind = StreamKind::infer_from_name(&name_bytes);

        let entity = FileEntity {
            identity: FileIdentity {
                origin: FileOrigin::Synthetic,
                relative_path: PathBuf::from(stream_name.to_string_lossy().as_ref()),
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
                sha256,
                is_sparse: false,
                extents: if size > 0 {
                    vec![Extent::Data { offset: 0, length: size }]
                } else {
                    Vec::new()
                },
            },
            streams: Vec::new(),
        };

        streams.push(AttachedStream {
            name: stream_name,
            kind,
            entity: Box::new(entity),
            data: Some(val),
        });
    }

    // Sort deterministically by byte name
    streams.sort_by(|a, b| a.name.0.cmp(&b.name.0));
    Ok(streams)
}

/// Writes all attached streams (xattrs, resource forks) to `dest`.
///
/// Fails with a hard error if the target filesystem cannot preserve them.
pub fn write_streams(
    dest: &Path,
    target_display_path: Option<&Path>,
    streams: &[AttachedStream],
    strict_lossless: bool,
) -> Result<()> {
    let display_target = target_display_path.unwrap_or(dest);
    for stream in streams {
        let name_os = stream.name.as_os_str();
        let Some(data) = &stream.data else {
            anyhow::bail!(
                "Stream {:?} on {} has no in-memory payload to write",
                stream.name.to_string_lossy(),
                display_target.display()
            );
        };

        if let Err(e) = xattr::set(dest, name_os, data) {
            if strict_lossless {
                anyhow::bail!(
                    "Target filesystem failed to store stream {:?} on {} (error: {}). Data would be lost.",
                    stream.name.to_string_lossy(),
                    display_target.display(),
                    e
                );
            }
        }
    }
    Ok(())
}
