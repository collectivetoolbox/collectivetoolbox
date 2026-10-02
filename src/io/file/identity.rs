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

use malachite::Natural;
use serde::{Deserialize, Serialize};
use std::fs::Metadata;
use std::path::{Path, PathBuf};

/// Unique filesystem inode identifier across mounts.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct InodeKey {
    /// Device identifier (`st_dev`).
    pub device_id: u64,
    /// Inode number on the device (`st_ino`).
    #[serde(with = "crate::serde_helpers::natural_flexible")]
    pub inode: Natural,
}

impl InodeKey {
    /// Creates a new `InodeKey`.
    #[must_use]
    pub fn new(device_id: u64, inode: impl Into<Natural>) -> Self {
        Self {
            device_id,
            inode: inode.into(),
        }
    }
}

impl From<(u64, u64)> for InodeKey {
    fn from((device_id, inode): (u64, u64)) -> Self {
        Self {
            device_id,
            inode: Natural::from(inode),
        }
    }
}

impl From<(u64, u128)> for InodeKey {
    fn from((device_id, inode): (u64, u128)) -> Self {
        Self {
            device_id,
            inode: Natural::from(inode),
        }
    }
}

impl From<(u64, Natural)> for InodeKey {
    fn from((device_id, inode): (u64, Natural)) -> Self {
        Self { device_id, inode }
    }
}

impl ctb_formats_dcstring::DcMixedEncode for InodeKey {
    fn encode_dc_mixed(&self, mst: &mut ctb_formats_dcstring::DcMst) -> Result<()> {
        mst.push_char(ctb_formats_dcstring::DcChar::from_short(328));
        ctb_formats_dcstring::DcMixedEncode::encode_dc_mixed(&self.device_id, mst)?;
        mst.push_char(ctb_formats_dcstring::DcChar::from_short(327));
        ctb_formats_dcstring::DcMixedEncode::encode_dc_mixed(&self.inode, mst)?;
        Ok(())
    }
}

impl ctb_formats_dcstring::DcMixedDecode for InodeKey {
    fn decode_dc_mixed(reader: &mut ctb_formats_dcstring::DcMixedReader<'_>) -> Result<Self> {
        reader.expect_short_dc(328)?;
        let device_id = <u64 as ctb_formats_dcstring::DcMixedDecode>::decode_dc_mixed(reader)?;
        reader.expect_short_dc(327)?;
        let inode = <Natural as ctb_formats_dcstring::DcMixedDecode>::decode_dc_mixed(reader)?;
        Ok(Self { device_id, inode })
    }
}

/// The origin source where a file was discovered or extracted from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ctb_formats_dcstring::DcMixed)]
#[dc(begin = 349, end = 350)]
pub enum FileOrigin {
    /// Residing on a local filesystem.
    #[dc(short = 351)]
    Filesystem {
        /// Inode key on the origin filesystem.
        #[dc(nested = 328)]
        key: InodeKey,
        /// Canonical path on the origin filesystem.
        #[dc(short = 337)]
        canonical_path: PathBuf,
    },
    /// An entry residing inside an archive (tar, zip, pax, cpio).
    #[dc(short = 352)]
    Archive {
        /// Path to the archive container file.
        #[dc(short = 337)]
        archive_path: PathBuf,
        /// Sequential index of the entry inside the archive.
        #[dc(short = 327)]
        entry_index: u64,
        /// Format of the archive container (e.g. "tar", "zip").
        #[dc(short = 371)]
        archive_format: String,
    },
    /// Remote network resource.
    #[dc(short = 353)]
    Remote {
        /// Full URI of the resource.
        #[dc(short = 337)]
        uri: String,
        /// Entity tag if supplied by the remote endpoint.
        #[dc(short = 368)]
        etag: Option<String>,
    },
    /// Synthetic in-memory file.
    #[dc(short = 354)]
    Synthetic,
    /// A FIFO such as stdin or a named pipe.
    #[dc(short = 534)]
    Fifo,
}

/// Multifaceted identity information for a file entity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ctb_formats_dcstring::DcMixed)]
#[dc(begin = 319, end = 320)]
pub struct FileIdentity {
    /// Origin locator where the file was discovered.
    #[dc(nested = 349)]
    pub origin: FileOrigin,
    /// Logical relative path within the operation or archive.
    #[dc(short = 337)]
    pub relative_path: Option<PathBuf>,
    /// Original enclosing directory or root path from which relative_path was
    /// resolved, if applicable.
    #[dc(short = 357)]
    pub enclosing_path: Option<PathBuf>,
    /// Exact raw bytes of the full relative path with canonical '/' separator.
    #[serde(default, with = "crate::serde_helpers::opt_text_or_base64")]
    #[dc(skip, reason = "Raw relative path bytes are an alternative representation unified with relative_path (Dc 337)")]
    pub raw_relative_path: Option<Vec<u8>>,
    /// Exact raw bytes of the filename on the origin (avoids lossy Unicode conversions).
    #[serde(default, with = "crate::serde_helpers::opt_text_or_base64")]
    #[dc(skip, reason = "Raw filename bytes are an alternative representation unified with relative_path (Dc 337)")]
    pub raw_filename: Option<Vec<u8>>,
    /// Link count on the source filesystem.
    #[dc(short = 355)]
    pub nlink: u64,
    /// Optional hardlink grouping identifier (e.g. InodeKey or archive linkname).
    #[serde(default, with = "crate::serde_helpers::opt_natural_flexible")]
    #[dc(short = 356)]
    pub hardlink_group: Option<Natural>,
}

impl FileIdentity {
    /// Returns the raw byte representation of the relative path, if present.
    #[must_use]
    pub fn path_bytes(&self) -> Option<&[u8]> {
        if let Some(ref raw) = self.raw_relative_path {
            Some(raw.as_slice())
        } else {
            self.relative_path
                .as_deref()
                .map(|p| p.as_os_str().as_encoded_bytes())
        }
    }

    /// Returns the full original path if `enclosing_path` and `relative_path` are present,
    /// otherwise falls back to origin's canonical path if available.
    #[must_use]
    pub fn full_original_path(&self) -> Option<PathBuf> {
        if let Some(ref base) = self.enclosing_path {
            self.relative_path.as_deref().map(|rel| base.join(rel))
        } else {
            match &self.origin {
                FileOrigin::Filesystem { canonical_path, .. } => {
                    Some(canonical_path.clone())
                }
                _ => None,
            }
        }
    }

    /// Returns a human-readable display path for error reporting and diagnostics.
    #[must_use]
    pub fn display_path(&self) -> String {
        if let Some(ref rel) = self.relative_path {
            rel.display().to_string()
        } else if let Some(path) = self.full_original_path() {
            path.display().to_string()
        } else {
            "<unknown>".to_string()
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

    #[cfg(windows)]
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

    #[cfg(all(not(unix), not(windows)))]
    {
        let _ = origin_is_windows;
        let _ = raw_bytes;
        anyhow::bail!("Path resolution is unimplemented for this platform");
    }
}

/// Queries the unique file identity ([`InodeKey`]) of a filesystem entry.
///
/// On Unix, uses `symlink_metadata` to retrieve device (`st_dev`) and inode
/// (`st_ino`). On Windows, queries file identity using the `file-id` crate
/// (`file_id::get_file_id`), resolving volume serial number and 64-bit or
/// 128-bit file index without modifying the target or following reparse points.
pub fn query_file_identity(path: &Path) -> Result<InodeKey> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let meta = std::fs::symlink_metadata(path)
            .with_context(|| format!("Failed to read metadata for {}", path.display()))?;
        Ok(InodeKey {
            device_id: meta.dev(),
            inode: Natural::from(meta.ino()),
        })
    }
    #[cfg(windows)]
    {
        let fid = file_id::get_file_id(path)
            .with_context(|| format!("Failed to read file ID for {}", path.display()))?;
        match fid {
            file_id::FileId::LowRes {
                volume_serial_number,
                file_index,
            } => Ok(InodeKey {
                device_id: u64::from(volume_serial_number),
                inode: Natural::from(file_index),
            }),
            file_id::FileId::HighRes {
                volume_serial_number,
                file_id,
            } => Ok(InodeKey {
                device_id: volume_serial_number,
                inode: Natural::from(file_id),
            }),
            file_id::FileId::Inode {
                device_id,
                inode_number,
            } => Ok(InodeKey {
                device_id,
                inode: Natural::from(inode_number),
            }),
        }
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = path;
        anyhow::bail!("query_file_identity is not supported on this platform");
    }
}

/// Extracts or queries the [`InodeKey`] identity for a path and its metadata.
///
/// On Unix, extracts `st_dev` and `st_ino` directly from `meta`.
/// On Windows, queries the file identity using [`query_file_identity`].
pub fn extract_file_identity(path: &Path, meta: &Metadata) -> Result<InodeKey> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let _ = path;
        Ok(InodeKey {
            device_id: meta.dev(),
            inode: Natural::from(meta.ino()),
        })
    }
    #[cfg(windows)]
    {
        let _ = meta;
        query_file_identity(path)
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = (path, meta);
        anyhow::bail!("extract_file_identity is not supported on this platform");
    }
}

/// Extracts or queries the device or volume ID for a path and its metadata.
pub fn extract_device_id_for_path(path: &Path, meta: &Metadata) -> Result<u64> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let _ = path;
        Ok(meta.dev())
    }
    #[cfg(windows)]
    {
        let _ = meta;
        query_file_identity(path).map(|key| key.device_id)
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = (path, meta);
        anyhow::bail!("extract_device_id_for_path is not supported on this platform");
    }
}

