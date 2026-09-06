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

//! Unified file entities and polymorphic entity kinds.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::file::identity::{FileIdentity, FileOrigin, InodeKey};
use crate::file::metadata::{FileMetadata, FileTimestamps};
use crate::file::payload::{Extent, get_file_extents};
use crate::file::streams::{AttachedStream, read_and_hash_streams};
use crate::file::sys_flags::query_file_flags;
use ctb_formats_checksum::Sha256Stream;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::path::{Path, PathBuf};

/// The concrete filesystem or archive kind of a file entity.
#[derive(Debug, Clone)]
pub enum FileEntityKind {
    /// A regular file with size, extents, and payload digest.
    Regular {
        /// Logical file size in bytes.
        size: u64,
        /// SHA-256 cryptographic digest of payload.
        sha256: [u8; 32],
        /// True if any sparse extents (holes) exist.
        is_sparse: bool,
        /// Discovered data and hole extents.
        extents: Vec<Extent>,
    },
    /// A directory node.
    Directory,
    /// A symbolic link pointing to raw target bytes.
    Symlink {
        /// Exact target bytes (not lossily decoded).
        target: Vec<u8>,
    },
    /// A hardlink to an existing path or inode in the session.
    Hardlink {
        /// Relative path to the original linked file.
        target_relative_path: PathBuf,
    },
    /// A named pipe (FIFO).
    Fifo,
    /// A character device node.
    CharDevice {
        /// Major and minor device numbers.
        rdev: u64,
    },
    /// A block device node.
    BlockDevice {
        /// Major and minor device numbers.
        rdev: u64,
    },
    /// A UNIX domain socket node.
    Socket,
    /// A POSIX or Solaris Door descriptor node.
    Door,
    /// A composite application or document bundle directory (e.g. .app, .pages).
    Bundle {
        /// The recognized bundle extension or type.
        bundle_type: String,
    },
}

/// A complete, self-describing file entity holding identity, metadata, streams,
/// and payload descriptor.
#[derive(Debug, Clone)]
pub struct FileEntity {
    /// Multifaceted identity and origin.
    pub identity: FileIdentity,
    /// Standard POSIX and platform metadata.
    pub metadata: FileMetadata,
    /// Concrete entity kind and payload details.
    pub kind: FileEntityKind,
    /// Alternate data streams, resource forks, and security labels.
    pub streams: Vec<AttachedStream>,
}

impl FileEntity {
    /// Inspects an existing filesystem entry at `path` and builds a full `FileEntity`.
    ///
    /// If `base_dir` is provided, `identity.relative_path` is calculated relative
    /// to `base_dir`. Otherwise, it uses the entry's filename.
    pub fn from_filesystem(path: &Path, base_dir: Option<&Path>) -> Result<Self> {
        let sym_meta = std::fs::symlink_metadata(path)
            .with_context(|| format!("Failed to read metadata for {}", path.display()))?;

        let dev = sym_meta.dev();
        let ino = sym_meta.ino();
        let nlink = sym_meta.nlink();
        let mode = sym_meta.mode();
        let uid = sym_meta.uid();
        let gid = sym_meta.gid();
        let file_type = sym_meta.file_type();
        let is_symlink = file_type.is_symlink();

        let atime_sec = sym_meta.atime();
        let atime_nsec = u32::try_from(sym_meta.atime_nsec())
            .context("Failed to convert atime nanoseconds to u32")?;
        let mtime_sec = sym_meta.mtime();
        let mtime_nsec = u32::try_from(sym_meta.mtime_nsec())
            .context("Failed to convert mtime nanoseconds to u32")?;
        let ctime_sec = sym_meta.ctime();
        let ctime_nsec = u32::try_from(sym_meta.ctime_nsec())
            .context("Failed to convert ctime nanoseconds to u32")?;

        let timestamps = FileTimestamps {
            atime_sec,
            atime_nsec,
            mtime_sec,
            mtime_nsec,
            ctime_sec,
            ctime_nsec,
            birthtime_sec: None,
            birthtime_nsec: None,
        };

        let (flags, platform_raw) = query_file_flags(path, is_symlink)?;

        let filename_bytes = path
            .file_name()
            .map_or_else(Vec::new, |f| f.as_bytes().to_vec());

        let relative_path = if let Some(base) = base_dir {
            match path.strip_prefix(base) {
                Ok(rel) => rel.to_path_buf(),
                Err(_) => path.file_name().map_or_else(PathBuf::new, PathBuf::from),
            }
        } else {
            path.file_name().map_or_else(PathBuf::new, PathBuf::from)
        };

        let canonical = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());

        let identity = FileIdentity {
            origin: FileOrigin::Filesystem {
                key: InodeKey {
                    device_id: dev,
                    inode: ino,
                },
                canonical_path: canonical,
            },
            relative_path,
            raw_filename: filename_bytes,
            nlink,
            hardlink_group: if nlink > 1 { Some(ino) } else { None },
        };

        let metadata = FileMetadata {
            mode,
            uid,
            gid,
            timestamps,
            flags,
            platform_raw_flags: platform_raw,
        };

        // Determine entity kind
        let kind = if file_type.is_symlink() {
            let target = std::fs::read_link(path)
                .with_context(|| format!("Failed to read symlink target for {}", path.display()))?;
            FileEntityKind::Symlink {
                target: target.as_os_str().as_bytes().to_vec(),
            }
        } else if file_type.is_dir() {
            // Check if directory is a macOS bundle (e.g. .app, .framework)
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if ext.eq_ignore_ascii_case("app")
                    || ext.eq_ignore_ascii_case("pages")
                    || ext.eq_ignore_ascii_case("framework")
                    || ext.eq_ignore_ascii_case("bundle")
                    || ext.eq_ignore_ascii_case("rtfd")
                {
                    FileEntityKind::Bundle {
                        bundle_type: ext.to_lowercase(),
                    }
                } else {
                    FileEntityKind::Directory
                }
            } else {
                FileEntityKind::Directory
            }
        } else if file_type.is_fifo() {
            FileEntityKind::Fifo
        } else if file_type.is_char_device() {
            FileEntityKind::CharDevice {
                rdev: sym_meta.rdev(),
            }
        } else if file_type.is_block_device() {
            FileEntityKind::BlockDevice {
                rdev: sym_meta.rdev(),
            }
        } else if file_type.is_socket() {
            FileEntityKind::Socket
        } else if (mode & 0xF000) == 0xD000 {
            // S_IFDOOR
            FileEntityKind::Door
        } else {
            // Regular file: discover extents and compute SHA-256
            let size = sym_meta.size();
            let mut file = File::open(path)
                .with_context(|| format!("Failed to open file for hashing: {}", path.display()))?;
            let extents = get_file_extents(&file, size)?;
            let is_sparse = extents.iter().any(Extent::is_hole);
            file.seek(SeekFrom::Start(0))?;

            let mut hasher = Sha256Stream::new();
            let mut buf = vec![0_u8; 64 * 1024];
            loop {
                let n = file.read(&mut buf)?;
                if n == 0 {
                    break;
                }
                let slice = buf
                    .get(..n)
                    .context("Buffer slice index out of bounds")?;
                hasher.update(slice);
            }
            let sha256 = hasher.finalize();

            FileEntityKind::Regular {
                size,
                sha256,
                is_sparse,
                extents,
            }
        };

        let streams = if is_symlink {
            Vec::new()
        } else {
            read_and_hash_streams(path)?
        };

        Ok(Self {
            identity,
            metadata,
            kind,
            streams,
        })
    }

    /// Whether this entity is a regular file.
    #[must_use]
    pub const fn is_regular(&self) -> bool {
        matches!(self.kind, FileEntityKind::Regular { .. })
    }

    /// Whether this entity is a directory or bundle directory.
    #[must_use]
    pub const fn is_dir(&self) -> bool {
        matches!(
            self.kind,
            FileEntityKind::Directory | FileEntityKind::Bundle { .. }
        )
    }

    /// Whether this entity is a symbolic link.
    #[must_use]
    pub const fn is_symlink(&self) -> bool {
        matches!(self.kind, FileEntityKind::Symlink { .. })
    }
}
