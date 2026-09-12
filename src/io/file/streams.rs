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
use std::ffi::{OsStr, OsString};
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

/// Native stream name, independent of the journal reader's operating system.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum StreamName {
    Bytes(Vec<u8>),
    WindowsUtf16(Vec<u16>),
}

impl StreamName {
    /// Creates a stream name from raw bytes.
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self::Bytes(bytes.to_vec())
    }

    /// Creates a stream name from a UTF-8 string slice.
    #[must_use]
    pub fn from_str(s: &str) -> Self {
        Self::Bytes(s.as_bytes().to_vec())
    }

    /// Returns canonical raw bytes, using little-endian units for UTF-16.
    #[must_use]
    pub fn as_bytes(&self) -> std::borrow::Cow<'_, [u8]> {
        match self {
            Self::Bytes(bytes) => std::borrow::Cow::Borrowed(bytes),
            Self::WindowsUtf16(units) => std::borrow::Cow::Owned(
                units.iter().flat_map(|unit| unit.to_le_bytes()).collect(),
            ),
        }
    }

    #[must_use]
    pub fn from_windows_utf16(units: &[u16]) -> Self {
        Self::WindowsUtf16(units.to_vec())
    }

    #[must_use]
    pub fn from_os_str(name: &OsStr) -> Self {
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStrExt;
            Self::WindowsUtf16(name.encode_wide().collect())
        }
        #[cfg(not(windows))]
        Self::Bytes(name.as_encoded_bytes().to_vec())
    }

    /// Lossy UTF-8 representation for diagnostics and logging.
    #[must_use]
    pub fn to_string_lossy(&self) -> std::borrow::Cow<'_, str> {
        match self {
            Self::Bytes(bytes) => String::from_utf8_lossy(bytes),
            Self::WindowsUtf16(units) => {
                std::borrow::Cow::Owned(String::from_utf16_lossy(units))
            }
        }
    }

    /// Converts only at the filesystem boundary, rejecting lossy transcoding.
    pub fn to_os_string(&self) -> Result<OsString> {
        match self {
            Self::Bytes(bytes) => {
                #[cfg(unix)]
                {
                    Ok(OsStr::from_bytes(bytes).to_os_string())
                }
                #[cfg(not(unix))]
                {
                    Ok(OsString::from(std::str::from_utf8(bytes).context(
                        "Target cannot represent this byte stream name",
                    )?))
                }
            }
            Self::WindowsUtf16(units) => {
                #[cfg(windows)]
                {
                    use std::os::windows::ffi::OsStringExt;
                    Ok(OsString::from_wide(units))
                }
                #[cfg(not(windows))]
                {
                    Ok(OsString::from(String::from_utf16(units).context(
                        "Target cannot represent this UTF-16 stream name",
                    )?))
                }
            }
        }
    }
}

/// The classification of an attached stream or fork.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
)]
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

/// The platform-canonical extended attribute name used to store/represent
/// a macOS resource fork.
///
/// On Darwin/macOS, writing to `com.apple.ResourceFork` writes directly to the
/// intrinsic resource fork. On other Unix platforms (e.g. Linux), standard
/// user xattrs require the `user.` namespace prefix, so
/// `user.com.apple.ResourceFork` is used.
pub const RESOURCE_FORK_XATTR_NAME: &str = if cfg!(target_os = "macos") {
    "com.apple.ResourceFork"
} else {
    "user.com.apple.ResourceFork"
};

impl StreamKind {
    /// Infers the stream kind from its byte name.
    #[must_use]
    pub fn infer_from_name(name: &[u8]) -> Self {
        if name == b"com.apple.ResourceFork"
            || name == b"user.com.apple.ResourceFork"
        {
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
    /// Original name and encoding, including ill-formed Unicode.
    ///
    /// A `None` value represents a nameless stream, such as a Classic Mac OS
    /// resource fork (which in HFS/MFS was an intrinsic nameless fork rather
    /// than a named stream) or a default unnamed stream.
    pub name: Option<StreamName>,
    /// Classification of the stream.
    pub kind: StreamKind,
    /// The stream represented as a full `FileEntity`.
    pub entity: Box<FileEntity>,
    /// In-memory payload data, if loaded.
    pub data: Option<Vec<u8>>,
}

/// Reads all extended attributes, resource forks, and security labels from `path`.
#[cfg(unix)]
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
            Ok(None) => anyhow::bail!(
                "Stream {:?} disappeared while reading {}",
                name_os,
                path.display()
            ),
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

        let stream_name = StreamName::from_bytes(&name_bytes);
        let kind = StreamKind::infer_from_name(&name_bytes);
        streams.push(AttachedStream::from_data(Some(stream_name), kind, val)?);
    }

    // Sort deterministically by stream name and kind
    streams.sort_by(|a, b| match a.name.cmp(&b.name) {
        std::cmp::Ordering::Equal => a.kind.cmp(&b.kind),
        ord => ord,
    });
    Ok(streams)
}

impl AttachedStream {
    /// Creates an attached stream from raw data and optional name.
    pub fn from_data(
        name: Option<StreamName>,
        kind: StreamKind,
        data: Vec<u8>,
    ) -> Result<Self> {
        let mut hasher = Sha256Stream::new();
        hasher.update(&data);
        let sha256 = hasher.finalize();
        let size = u64::try_from(data.len())?;
        let name_bytes = name
            .as_ref()
            .map(|n| n.as_bytes().into_owned())
            .unwrap_or_default();

        let entity = FileEntity {
            identity: FileIdentity {
                origin: FileOrigin::Synthetic,
                relative_path: PathBuf::new(),
                enclosing_path: None,
                raw_relative_path: Vec::new(),
                raw_filename: name_bytes,
                nlink: 1,
                hardlink_group: None,
            },
            metadata: FileMetadata {
                native: None,
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
                    resolution_nsec: None,
                },
                flags: Vec::new(),
                platform_raw_flags: None,
                read_time: None,
                filesystem_type: None,
            },
            kind: FileEntityKind::Regular {
                size,
                sha256,
                is_sparse: false,
                extents: if size > 0 {
                    vec![Extent::Data {
                        offset: 0,
                        length: size,
                    }]
                } else {
                    Vec::new()
                },
            },
            streams: Vec::new(),
        };

        Ok(Self {
            name,
            kind,
            entity: Box::new(entity),
            data: Some(data),
        })
    }

    /// Lossy string name for logging and diagnostics.
    #[must_use]
    pub fn to_string_lossy(&self) -> std::borrow::Cow<'_, str> {
        match &self.name {
            Some(name) => name.to_string_lossy(),
            None => match self.kind {
                StreamKind::MacOsResourceFork => {
                    std::borrow::Cow::Borrowed("(resource fork)")
                }
                _ => std::borrow::Cow::Borrowed("(unnamed stream)"),
            },
        }
    }
}

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::{
    read_and_hash_streams, remove_windows_stream, write_windows_streams,
};

/// Reads all extended attributes, resource forks, and security labels from `path`.
#[cfg(not(any(unix, windows)))]
pub fn read_and_hash_streams(path: &Path) -> Result<Vec<AttachedStream>> {
    anyhow::bail!(
        "Lossless stream enumeration is not implemented on this platform: {}",
        path.display()
    )
}

/// Writes all attached streams (xattrs, resource forks) to `dest`.
///
/// Fails with a hard error if the target filesystem cannot preserve them.
#[cfg(unix)]
pub fn write_streams(
    dest: &Path,
    target_display_path: Option<&Path>,
    streams: &[AttachedStream],
    strict_lossless: bool,
) -> Result<()> {
    // Reason for fallback: error reporting defaults to actual destination path if no alternate display path provided
    let display_target = target_display_path.unwrap_or(dest);
    let mut names = std::collections::HashSet::new();
    let mut validated = Vec::new();
    for stream in streams {
        let name_os = match &stream.name {
            Some(name) => name.to_os_string()?,
            None => match stream.kind {
                // On macOS, com.apple.ResourceFork writes to the resource fork directly.
                // On other Unix platforms, com.apple.ResourceFork stores the resource fork in xattr.
                StreamKind::MacOsResourceFork => {
                    OsString::from(RESOURCE_FORK_XATTR_NAME)
                }
                _ => {
                    if strict_lossless {
                        anyhow::bail!(
                            "Cannot write unnamed non-resource-fork stream to xattr on {}",
                            display_target.display()
                        );
                    }
                    warn_fmt!(
                        "Caveat: Skipping unnamed stream on {} (proceeding best-effort)",
                        display_target.display()
                    );
                    continue;
                }
            },
        };
        anyhow::ensure!(
            names.insert(name_os.clone()),
            "Stream names collide on the destination platform"
        );
        let Some(data) = &stream.data else {
            anyhow::bail!(
                "Stream {:?} on {} has no in-memory payload to write",
                stream.to_string_lossy(),
                display_target.display()
            );
        };

        let FileEntityKind::Regular { size, sha256, .. } = &stream.entity.kind
        else {
            anyhow::bail!("Attached stream is not a regular payload");
        };
        let mut hasher = Sha256Stream::new();
        hasher.update(data);
        anyhow::ensure!(
            u64::try_from(data.len())? == *size && hasher.finalize() == *sha256,
            "Attached stream payload does not match its descriptor: {:?}",
            stream.to_string_lossy()
        );
        validated.push((stream, name_os, data));
    }
    for (stream, name_os, data) in validated {
        if let Err(e) = xattr::set(dest, &name_os, data) {
            if strict_lossless {
                anyhow::bail!(
                    "Target filesystem failed to store stream {:?} on {} (error: {}). Data would be lost.",
                    stream.to_string_lossy(),
                    display_target.display(),
                    e
                );
            }
            warn_fmt!(
                "Caveat: Target filesystem cannot store extended attribute {:?} on {}: {} (proceeding best-effort)",
                stream.to_string_lossy(),
                display_target.display(),
                e
            );
        }
    }
    Ok(())
}

/// Writes all attached streams (NTFS alternate data streams) to `dest`.
#[cfg(windows)]
pub fn write_streams(
    dest: &Path,
    _target_display_path: Option<&Path>,
    streams: &[AttachedStream],
    _strict_lossless: bool,
) -> Result<()> {
    windows::write_windows_streams(dest, streams)
}

/// Writes all attached streams (xattrs, resource forks) to `dest`.
#[cfg(not(any(unix, windows)))]
pub fn write_streams(
    _dest: &Path,
    _target_display_path: Option<&Path>,
    streams: &[AttachedStream],
    strict_lossless: bool,
) -> Result<()> {
    if !streams.is_empty() {
        if strict_lossless {
            anyhow::bail!("Target platform does not support xattrs/streams");
        }
        warn_fmt!("Caveat: Target platform does not support xattrs/streams (proceeding best-effort)");
    }
    Ok(())
}

/// Remove an attached stream/xattr from `path`.
pub fn remove_stream(path: &Path, name: &std::ffi::OsStr) -> Result<()> {
    #[cfg(unix)]
    {
        xattr::remove(path, name)?;
        Ok(())
    }
    #[cfg(windows)]
    {
        windows::remove_windows_stream(path, name)
    }
    #[cfg(not(any(unix, windows)))]
    {
        anyhow::bail!(
            "Removing stream {name:?} is unsupported on this platform: {}",
            path.display()
        )
    }
}
