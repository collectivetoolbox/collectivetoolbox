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

//! File origin, identity, and hardlink tracking identifiers.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::path::PathBuf;

/// Unique filesystem inode identifier across mounts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InodeKey {
    /// Device identifier (`st_dev`).
    pub device_id: u64,
    /// Inode number on the device (`st_ino`).
    pub inode: u64,
}

/// The origin source where a file was discovered or extracted from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileOrigin {
    /// Residing on a local filesystem.
    Filesystem {
        /// Inode key on the origin filesystem.
        key: InodeKey,
        /// Canonical path on the origin filesystem.
        canonical_path: PathBuf,
    },
    /// An entry residing inside an archive (tar, zip, pax, cpio).
    Archive {
        /// Path to the archive container file.
        archive_path: PathBuf,
        /// Sequential index of the entry inside the archive.
        entry_index: u64,
        /// Format of the archive container (e.g. "tar", "zip").
        archive_format: String,
    },
    /// Remote network resource.
    Remote {
        /// Full URI of the resource.
        uri: String,
        /// Entity tag if supplied by the remote endpoint.
        etag: Option<String>,
    },
    /// Synthetic in-memory file.
    Synthetic,
}

/// Multifaceted identity information for a file entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileIdentity {
    /// Origin locator where the file was discovered.
    pub origin: FileOrigin,
    /// Logical relative path within the operation or archive.
    pub relative_path: PathBuf,
    /// Exact raw bytes of the full relative path with canonical '/' separator.
    pub raw_relative_path: Vec<u8>,
    /// Exact raw bytes of the filename on the origin (avoids lossy Unicode conversions).
    pub raw_filename: Vec<u8>,
    /// Link count on the source filesystem.
    pub nlink: u64,
    /// Optional hardlink grouping identifier (e.g. InodeKey or archive linkname).
    pub hardlink_group: Option<u64>,
}

impl FileIdentity {
    /// Returns the raw byte representation of the relative path.
    #[must_use]
    pub fn path_bytes(&self) -> &[u8] {
        if !self.raw_relative_path.is_empty() {
            &self.raw_relative_path
        } else {
            self.relative_path.as_os_str().as_encoded_bytes()
        }
    }
}

/// Resolves raw relative path bytes to a local OS `PathBuf`.
///
/// Converts canonical forward-slash separated path bytes to the current platform's
/// path representation. On POSIX/Linux, this is lossless for any byte sequence.
/// On Windows, decodes UTF-8 and maps `/` to `\`.
pub fn resolve_relative_path_for_os(raw_bytes: &[u8], origin_is_windows: bool) -> Result<PathBuf> {
    #[cfg(unix)]
    {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let bytes = if origin_is_windows && raw_bytes.contains(&b'\\') {
            let mut normalized = raw_bytes.to_vec();
            for b in &mut normalized {
                if *b == b'\\' {
                    *b = b'/';
                }
            }
            normalized
        } else {
            raw_bytes.to_vec()
        };

        Ok(PathBuf::from(OsStr::from_bytes(&bytes)))
    }

    #[cfg(not(unix))]
    {
        let s = std::str::from_utf8(raw_bytes)
            .context("Relative path bytes are not valid UTF-8 for current platform")?;

        for c in s.chars() {
            if matches!(c, ':' | '*' | '?' | '"' | '<' | '>' | '|') {
                anyhow::bail!("Path contains character '{c}' which is incompatible with target platform");
            }
        }

        let normalized = s.replace('/', "\\");
        Ok(PathBuf::from(normalized))
    }
}
