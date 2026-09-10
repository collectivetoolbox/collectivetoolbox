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
use crate::file::materializer::{MaterializeOptions, MaterializeReceipt};
use crate::file::metadata::{FileMetadata, FileTimestamps};
use crate::file::payload::{Extent, PayloadSource, get_file_extents};
use crate::file::sandboxable_dir::SandboxableDir;
use crate::file::streams::{AttachedStream, read_and_hash_streams};
use crate::file::sys_flags::query_file_flags;
use ctb_formats_checksum::Sha256Stream;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// The concrete filesystem or archive kind of a file entity.
#[derive(Debug, Clone, PartialEq, Eq)]
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
        /// Relative path to the original linked file in raw bytes.
        target_relative_path: Vec<u8>,
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

/// Canonical categorical type of a file entity without payload details.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FileEntityType {
    /// Regular file.
    Regular,
    /// Directory node.
    Directory,
    /// Symbolic link.
    Symlink,
    /// Hardlink to an existing path or inode.
    Hardlink,
    /// Named pipe (FIFO).
    Fifo,
    /// Character device node.
    CharDevice,
    /// Block device node.
    BlockDevice,
    /// UNIX domain socket node.
    Socket,
    /// Door descriptor node.
    Door,
    /// Composite bundle directory.
    Bundle,
}

impl FileEntityType {
    /// Returns the canonical string representation matching the SQLite `kind` column.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Regular => "regular",
            Self::Directory => "dir",
            Self::Symlink => "symlink",
            Self::Hardlink => "hardlink",
            Self::Fifo => "fifo",
            Self::CharDevice => "chardev",
            Self::BlockDevice => "blockdev",
            Self::Socket => "socket",
            Self::Door => "door",
            Self::Bundle => "bundle",
        }
    }

    /// Parses a string specifier or alias into a `FileEntityType`.
    ///
    /// Supports find-style shorthand ("f", "d", "l", "h", "p", "c", "b", "s")
    /// as well as full type names and canonical names.
    pub fn parse(s: &str) -> Result<Self> {
        let trimmed = s.trim().to_lowercase();
        match trimmed.as_str() {
            "f" | "file" | "regular" => Ok(Self::Regular),
            "d" | "dir" | "directory" => Ok(Self::Directory),
            "l" | "symlink" | "link" => Ok(Self::Symlink),
            "h" | "hardlink" => Ok(Self::Hardlink),
            "p" | "fifo" | "pipe" => Ok(Self::Fifo),
            "c" | "char" | "chardev" | "character" => Ok(Self::CharDevice),
            "b" | "block" | "blockdev" => Ok(Self::BlockDevice),
            "s" | "socket" => Ok(Self::Socket),
            "door" => Ok(Self::Door),
            "bundle" => Ok(Self::Bundle),
            other => anyhow::bail!(
                "Unknown file entity type '{other}'. Expected one of: file/f, \
                 dir/d, symlink/l, hardlink/h, fifo/p, chardev/c, blockdev/b, \
                 socket/s, door, bundle"
            ),
        }
    }
}

impl std::fmt::Display for FileEntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for FileEntityType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl FileEntityKind {
    /// Returns the categorical entity type of this kind.
    #[must_use]
    pub const fn entity_type(&self) -> FileEntityType {
        match self {
            Self::Regular { .. } => FileEntityType::Regular,
            Self::Directory => FileEntityType::Directory,
            Self::Symlink { .. } => FileEntityType::Symlink,
            Self::Hardlink { .. } => FileEntityType::Hardlink,
            Self::Fifo => FileEntityType::Fifo,
            Self::CharDevice { .. } => FileEntityType::CharDevice,
            Self::BlockDevice { .. } => FileEntityType::BlockDevice,
            Self::Socket => FileEntityType::Socket,
            Self::Door => FileEntityType::Door,
            Self::Bundle { .. } => FileEntityType::Bundle,
        }
    }

    /// Returns the canonical string representation for this kind.
    #[must_use]
    pub const fn kind_str(&self) -> &'static str {
        self.entity_type().as_str()
    }
}


/// A complete, self-describing file entity holding identity, metadata, streams,
/// and payload descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// Materializes this entity onto the filesystem within a [`SandboxableDir`].
    pub fn materialize(
        &self,
        payload: Option<&mut dyn PayloadSource>,
        dest_dir: &SandboxableDir,
        options: &MaterializeOptions,
    ) -> Result<MaterializeReceipt> {
        crate::file::materializer::materialize_entity(self, payload, dest_dir, options)
    }

    /// Materializes this entity onto the filesystem given a destination root path.
    pub fn materialize_at_path(
        &self,
        payload: Option<&mut dyn PayloadSource>,
        dest_root: &Path,
        options: &MaterializeOptions,
    ) -> Result<MaterializeReceipt> {
        crate::file::materializer::materialize_entity_at_path(self, payload, dest_root, options)
    }

    /// Inspects an existing filesystem entry at `path` and builds a full `FileEntity`.
    ///
    /// If `base_dir` is provided, `identity.relative_path` is calculated relative
    /// to `base_dir`. Otherwise, it uses the entry's filename.
    pub fn from_filesystem(path: &Path, base_dir: Option<&Path>) -> Result<Self> {
        Self::from_filesystem_internal(path, base_dir, true)
    }

    /// Inspects an existing filesystem entry at `path` and builds a `FileEntity`
    /// without reading file payload bytes or computing cryptographic hashes.
    pub fn from_filesystem_metadata_only(path: &Path, base_dir: Option<&Path>) -> Result<Self> {
        Self::from_filesystem_internal(path, base_dir, false)
    }

    fn from_filesystem_internal(
        path: &Path,
        base_dir: Option<&Path>,
        compute_hash: bool,
    ) -> Result<Self> {
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

        // Reason for fallback: Root or empty paths have no trailing filename component, represented by empty raw filename bytes.
        let filename_bytes = path
            .file_name()
            .map_or_else(Vec::new, |f| f.as_bytes().to_vec());

        // Reason for fallback: When path cannot be stripped of base_dir prefix or is root, fall back to file name or empty PathBuf.
        let relative_path = if let Some(base) = base_dir {
            match path.strip_prefix(base) {
                Ok(rel) if !rel.as_os_str().is_empty() => rel.to_path_buf(),
                Ok(_) if sym_meta.is_dir() => PathBuf::new(),
                _ => path.file_name().map_or_else(PathBuf::new, PathBuf::from),
            }
        } else {
            path.file_name().map_or_else(PathBuf::new, PathBuf::from)
        };

        let read_time = Some(SystemTime::now());

        // Reason for fallback: Dangling symlinks or special pseudo-paths cannot be canonicalized by the OS; fall back to verbatim path.
        let canonical = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());

        let raw_relative_path = relative_path.as_os_str().as_encoded_bytes().to_vec();

        let enclosing_path = match base_dir {
            Some(base) => Some(base.to_path_buf()),
            None => path.parent().map(Path::to_path_buf),
        };

        let identity = FileIdentity {
            origin: FileOrigin::Filesystem {
                key: InodeKey {
                    device_id: dev,
                    inode: ino,
                },
                canonical_path: canonical,
            },
            relative_path,
            enclosing_path,
            raw_relative_path,
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
            read_time,
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
        } else if !compute_hash {
            FileEntityKind::Regular {
                size: sym_meta.size(),
                sha256: [0_u8; 32],
                is_sparse: false,
                extents: Vec::new(),
            }
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

        let streams = if is_symlink || !compute_hash {
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

    /// Returns the original enclosing directory if available.
    #[must_use]
    pub fn enclosing_path(&self) -> Option<&Path> {
        self.identity.enclosing_path.as_deref()
    }

    /// Returns the timestamp documenting when this file record was
    /// read/inspected from the filesystem (current as of).
    #[must_use]
    pub const fn is_current_as_of(&self) -> Option<SystemTime> {
        self.metadata.read_time
    }

    /// Returns the time when this file entity was read from the filesystem, if
    /// captured.
    #[must_use]
    pub const fn read_time(&self) -> Option<SystemTime> {
        self.metadata.read_time
    }
}
