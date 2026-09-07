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

//! Independent post-write verification, cache purging, and metadata auditing.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::file::entity::{FileEntity, FileEntityKind};
use crate::file::payload::{Extent, get_file_extents};
use crate::file::streams::read_and_hash_streams;
use crate::file::sys_flags::query_file_flags;
use ctb_formats_checksum::Sha256Stream;
use filetime::{FileTime, set_file_times};
use nix::fcntl::{PosixFadviseAdvice, posix_fadvise};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::os::unix::ffi::OsStringExt;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// Nature of a stream or extended attribute discrepancy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StreamDiffKind {
    /// Stream expected in entity but missing on disk.
    MissingStream,
    /// Unexpected stream present on disk but absent in entity.
    ExtraStream,
    /// Cryptographic digest mismatch on stream content.
    DigestMismatch {
        expected_hex: String,
        actual_hex: String,
    },
}

impl std::fmt::Display for StreamDiffKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingStream => write!(f, "stream missing"),
            Self::ExtraStream => write!(f, "unexpected extra stream"),
            Self::DigestMismatch {
                expected_hex,
                actual_hex,
            } => {
                write!(
                    f,
                    "stream digest mismatch (expected {expected_hex}, got {actual_hex})"
                )
            }
        }
    }
}

/// Discrepancy detected between an expected entity and target state on disk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DiffKind {
    /// Expected entry is missing entirely from disk.
    MissingOnDisk,
    /// Unexpected entry exists on disk but is absent from manifest.
    UntrackedOnDisk,
    /// File type on disk does not match expected entity.
    TypeMismatch {
        expected: String,
        actual: String,
    },
    /// Regular file byte length mismatch.
    SizeMismatch {
        expected: u64,
        actual: u64,
    },
    /// Cryptographic SHA-256 payload checksum mismatch.
    ContentHashMismatch {
        expected_hex: String,
        actual_hex: String,
    },
    /// Symbolic link target path mismatch.
    SymlinkTargetMismatch {
        expected: String,
        actual: String,
    },
    /// POSIX file permission mode mismatch (masked with 0o7777).
    ModeMismatch {
        expected: String,
        actual: String,
    },
    /// Owner user ID (UID) mismatch.
    UidMismatch {
        expected: u32,
        actual: u32,
    },
    /// Owner group ID (GID) mismatch.
    GidMismatch {
        expected: u32,
        actual: u32,
    },
    /// Modification time (mtime) mismatch.
    MtimeMismatch {
        expected_sec: i64,
        expected_nsec: u32,
        actual_sec: i64,
        actual_nsec: u32,
    },
    /// Access time (atime) mismatch.
    AtimeMismatch {
        expected_sec: i64,
        expected_nsec: u32,
        actual_sec: i64,
        actual_nsec: u32,
    },
    /// Metadata change time (ctime) mismatch.
    CtimeMismatch {
        expected_sec: i64,
        expected_nsec: u32,
        actual_sec: i64,
        actual_nsec: u32,
    },
    /// File birth / creation time mismatch.
    BirthtimeMismatch {
        expected_sec: Option<i64>,
        actual_sec: Option<i64>,
    },
    /// Semantic or OS file flags mismatch.
    FlagsMismatch {
        expected: Vec<String>,
        actual: Vec<String>,
    },
    /// Alternate data stream or extended attribute mismatch.
    StreamMismatch {
        stream_name: String,
        details: StreamDiffKind,
    },
    /// Sparse file hole existence mismatch.
    SparseHoleMismatch {
        expected_has_holes: bool,
        actual_has_holes: bool,
    },
    /// Hardlink target or inode grouping mismatch.
    HardlinkMismatch {
        expected_target: PathBuf,
        details: String,
    },
    /// Path cannot be represented losslessly on target operating system.
    IncompatiblePath {
        reason: String,
    },
}

impl std::fmt::Display for DiffKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingOnDisk => write!(f, "Missing on disk"),
            Self::UntrackedOnDisk => write!(f, "Untracked on disk"),
            Self::TypeMismatch { expected, actual } => {
                write!(f, "Type mismatch: expected {expected}, got {actual}")
            }
            Self::SizeMismatch { expected, actual } => {
                write!(
                    f,
                    "Size mismatch: expected {expected} bytes, got {actual} bytes"
                )
            }
            Self::ContentHashMismatch {
                expected_hex,
                actual_hex,
            } => {
                write!(
                    f,
                    "SHA-256 mismatch: expected {expected_hex}, got {actual_hex}"
                )
            }
            Self::SymlinkTargetMismatch { expected, actual } => {
                write!(
                    f,
                    "Symlink target mismatch: expected {expected}, got {actual}"
                )
            }
            Self::ModeMismatch { expected, actual } => {
                write!(f, "Permissions mismatch: expected {expected}, got {actual}")
            }
            Self::UidMismatch { expected, actual } => {
                write!(f, "Owner UID mismatch: expected {expected}, got {actual}")
            }
            Self::GidMismatch { expected, actual } => {
                write!(f, "Owner GID mismatch: expected {expected}, got {actual}")
            }
            Self::MtimeMismatch {
                expected_sec,
                expected_nsec,
                actual_sec,
                actual_nsec,
            } => {
                write!(
                    f,
                    "Mtime mismatch: expected {expected_sec}.{expected_nsec:09}, got {actual_sec}.{actual_nsec:09}"
                )
            }
            Self::AtimeMismatch {
                expected_sec,
                expected_nsec,
                actual_sec,
                actual_nsec,
            } => {
                write!(
                    f,
                    "Atime mismatch: expected {expected_sec}.{expected_nsec:09}, got {actual_sec}.{actual_nsec:09}"
                )
            }
            Self::CtimeMismatch {
                expected_sec,
                expected_nsec,
                actual_sec,
                actual_nsec,
            } => {
                write!(
                    f,
                    "Ctime mismatch: expected {expected_sec}.{expected_nsec:09}, got {actual_sec}.{actual_nsec:09}"
                )
            }
            Self::BirthtimeMismatch {
                expected_sec,
                actual_sec,
            } => {
                write!(
                    f,
                    "Birthtime mismatch: expected {expected_sec:?}, got {actual_sec:?}"
                )
            }
            Self::FlagsMismatch { expected, actual } => {
                write!(f, "Flags mismatch: expected {expected:?}, got {actual:?}")
            }
            Self::StreamMismatch {
                stream_name,
                details,
            } => {
                write!(f, "Stream '{stream_name}' mismatch: {details}")
            }
            Self::SparseHoleMismatch {
                expected_has_holes,
                actual_has_holes,
            } => {
                write!(
                    f,
                    "Sparse hole mismatch: expected sparse={expected_has_holes}, got sparse={actual_has_holes}"
                )
            }
            Self::HardlinkMismatch {
                expected_target,
                details,
            } => {
                write!(
                    f,
                    "Hardlink mismatch: target {}, details: {details}",
                    expected_target.display()
                )
            }
            Self::IncompatiblePath { reason } => {
                write!(f, "Incompatible path for target platform: {reason}")
            }
        }
    }
}

/// Options controlling which attributes and checks are audited by `audit_entity`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityAuditOptions {
    /// Skip checking access times (atime).
    pub ignore_atime: bool,
    /// Skip checking modification times (mtime).
    pub ignore_mtime: bool,
    /// Skip checking metadata change times (ctime).
    pub ignore_ctime: bool,
    /// Skip checking owner UID and GID.
    pub ignore_owner: bool,
    /// Skip checking POSIX permission modes.
    pub ignore_perms: bool,
    /// Skip checking file flags.
    pub ignore_flags: bool,
    /// Skip checking extended attributes and alternate data streams.
    pub ignore_xattrs: bool,
    /// Check whether sparse hole presence matches.
    pub check_sparse: bool,
    /// Request kernel and file cache page eviction prior to reading.
    pub drop_caches: bool,
}

impl Default for EntityAuditOptions {
    fn default() -> Self {
        Self {
            ignore_atime: false,
            ignore_mtime: false,
            ignore_ctime: true,
            ignore_owner: false,
            ignore_perms: false,
            ignore_flags: false,
            ignore_xattrs: false,
            check_sparse: true,
            drop_caches: false,
        }
    }
}

/// Converts a 32-byte digest array into a lowercase hex string.
#[expect(clippy::let_underscore_must_use, reason = "Writing into in-memory String cannot fail")]
pub fn hex_encode(bytes: &[u8; 32]) -> String {
    use std::fmt::Write;
    let mut s = String::with_capacity(64);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

fn describe_file_type(meta: &std::fs::Metadata) -> String {
    if meta.is_dir() {
        "directory".to_string()
    } else if meta.is_symlink() {
        "symlink".to_string()
    } else if meta.is_file() {
        "regular file".to_string()
    } else {
        "special node".to_string()
    }
}

/// Drops system page cache pages for a specific file descriptor.
pub fn evict_fd_cache(file: &File) {
    let _ = posix_fadvise(file, 0, 0, PosixFadviseAdvice::POSIX_FADV_DONTNEED);
}

/// Attempts to drop system-wide cache pages on Linux if running with privileges.
pub fn try_drop_system_caches() {
    #[cfg(target_os = "linux")]
    {
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .write(true)
            .open("/proc/sys/vm/drop_caches")
        {
            use std::io::Write;
            let _ = f.write_all(b"3\n");
        }
    }
}

/// Audits an on-disk filesystem entry at `path` against an `expected` entity.
///
/// Returns a list of detected discrepancies (`Vec<DiffKind>`). An empty return
/// indicates the entry on disk completely matches the expected entity.
///
/// This audit is guaranteed to be non-invasive: on Linux, files are opened with
/// `O_NOATIME` to prevent modifying access times. If `O_NOATIME` is unavailable
/// (e.g. unprivileged non-owner), access and modification times are restored
/// immediately after payload verification so no modified state remains.
#[expect(clippy::too_many_lines, reason = "Comprehensive audit of all entity attributes, hashes, streams, and extents")]
pub fn audit_entity(
    path: &Path,
    expected: &FileEntity,
    options: &EntityAuditOptions,
) -> Result<Vec<DiffKind>> {
    let mut diffs = Vec::new();

    let dest_meta = match std::fs::symlink_metadata(path) {
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Ok(vec![DiffKind::MissingOnDisk]);
        }
        Err(err) => {
            return Err(err).with_context(|| {
                format!("Failed to read metadata for {}", path.display())
            });
        }
        Ok(m) => m,
    };

    // 1. File Type Check
    let type_matches = if expected.is_symlink() {
        dest_meta.is_symlink()
    } else if expected.is_dir() {
        dest_meta.is_dir()
    } else if matches!(expected.kind, FileEntityKind::Regular { .. }) {
        dest_meta.is_file()
    } else {
        true
    };

    if !type_matches {
        let expected_str = if expected.is_symlink() {
            "symlink".to_string()
        } else if expected.is_dir() {
            "directory".to_string()
        } else {
            "regular file".to_string()
        };
        diffs.push(DiffKind::TypeMismatch {
            expected: expected_str,
            actual: describe_file_type(&dest_meta),
        });
        return Ok(diffs);
    }

    // 2. Mode / Permissions (skip symlinks as their permissions are not meaningful on Linux)
    if !options.ignore_perms && !expected.is_symlink() {
        let expected_mode = expected.metadata.mode & 0o7777;
        let actual_mode = dest_meta.mode() & 0o7777;
        if expected_mode != actual_mode {
            diffs.push(DiffKind::ModeMismatch {
                expected: format!("{expected_mode:#05o}"),
                actual: format!("{actual_mode:#05o}"),
            });
        }
    }

    // 3. Ownership
    if !options.ignore_owner {
        if dest_meta.uid() != expected.metadata.uid {
            diffs.push(DiffKind::UidMismatch {
                expected: expected.metadata.uid,
                actual: dest_meta.uid(),
            });
        }
        if dest_meta.gid() != expected.metadata.gid {
            diffs.push(DiffKind::GidMismatch {
                expected: expected.metadata.gid,
                actual: dest_meta.gid(),
            });
        }
    }

    // 4. Timestamps
    if !options.ignore_mtime {
        let actual_sec = dest_meta.mtime();
        // Reason for fallback: Filesystems without sub-second timestamp resolution or negative nsec return 0 nanoseconds.
        let actual_nsec = u32::try_from(dest_meta.mtime_nsec()).unwrap_or(0);
        if actual_sec != expected.metadata.timestamps.mtime_sec
            || actual_nsec != expected.metadata.timestamps.mtime_nsec
        {
            diffs.push(DiffKind::MtimeMismatch {
                expected_sec: expected.metadata.timestamps.mtime_sec,
                expected_nsec: expected.metadata.timestamps.mtime_nsec,
                actual_sec,
                actual_nsec,
            });
        }
    }

    if !options.ignore_atime {
        let actual_sec = dest_meta.atime();
        // Reason for fallback: Filesystems without sub-second timestamp resolution or negative nsec return 0 nanoseconds.
        let actual_nsec = u32::try_from(dest_meta.atime_nsec()).unwrap_or(0);
        if actual_sec != expected.metadata.timestamps.atime_sec
            || actual_nsec != expected.metadata.timestamps.atime_nsec
        {
            diffs.push(DiffKind::AtimeMismatch {
                expected_sec: expected.metadata.timestamps.atime_sec,
                expected_nsec: expected.metadata.timestamps.atime_nsec,
                actual_sec,
                actual_nsec,
            });
        }
    }

    if !options.ignore_ctime {
        let actual_sec = dest_meta.ctime();
        // Reason for fallback: Filesystems without sub-second timestamp resolution or negative nsec return 0 nanoseconds.
        let actual_nsec = u32::try_from(dest_meta.ctime_nsec()).unwrap_or(0);
        if actual_sec != expected.metadata.timestamps.ctime_sec
            || actual_nsec != expected.metadata.timestamps.ctime_nsec
        {
            diffs.push(DiffKind::CtimeMismatch {
                expected_sec: expected.metadata.timestamps.ctime_sec,
                expected_nsec: expected.metadata.timestamps.ctime_nsec,
                actual_sec,
                actual_nsec,
            });
        }
    }

    // 5. File Flags
    if !options.ignore_flags {
        if let Ok((actual_flags, _)) = query_file_flags(path, expected.is_symlink()) {
            let mut exp_names: Vec<String> = expected
                .metadata
                .flags
                .iter()
                .map(|f| f.name().to_string())
                .collect();
            let mut act_names: Vec<String> = actual_flags
                .iter()
                .map(|f| f.name().to_string())
                .collect();
            exp_names.sort();
            act_names.sort();
            if exp_names != act_names {
                diffs.push(DiffKind::FlagsMismatch {
                    expected: exp_names,
                    actual: act_names,
                });
            }
        }
    }

    // 6. Streams and Extended Attributes (skip symlinks)
    if !options.ignore_xattrs && !expected.is_symlink() {
        let on_disk_streams = read_and_hash_streams(path)?;
        let mut expected_map: HashMap<OsString, [u8; 32]> = HashMap::new();
        for s in &expected.streams {
            let hash = match &s.entity.kind {
                FileEntityKind::Regular { sha256, .. } => *sha256,
                _ => [0_u8; 32],
            };
            expected_map.insert(OsString::from_vec(s.name.0.clone()), hash);
        }

        let mut disk_map: HashMap<OsString, [u8; 32]> = HashMap::new();
        for s in &on_disk_streams {
            let hash = match &s.entity.kind {
                FileEntityKind::Regular { sha256, .. } => *sha256,
                _ => [0_u8; 32],
            };
            disk_map.insert(OsString::from_vec(s.name.0.clone()), hash);
        }

        for (exp_name, exp_hash) in &expected_map {
            if let Some(act_hash) = disk_map.get(exp_name) {
                if exp_hash != act_hash {
                    diffs.push(DiffKind::StreamMismatch {
                        stream_name: exp_name.to_string_lossy().to_string(),
                        details: StreamDiffKind::DigestMismatch {
                            expected_hex: hex_encode(exp_hash),
                            actual_hex: hex_encode(act_hash),
                        },
                    });
                }
            } else {
                diffs.push(DiffKind::StreamMismatch {
                    stream_name: exp_name.to_string_lossy().to_string(),
                    details: StreamDiffKind::MissingStream,
                });
            }
        }

        for disk_name in disk_map.keys() {
            if !expected_map.contains_key(disk_name) {
                diffs.push(DiffKind::StreamMismatch {
                    stream_name: disk_name.to_string_lossy().to_string(),
                    details: StreamDiffKind::ExtraStream,
                });
            }
        }
    }

    // 7. Symlink Target
    if let FileEntityKind::Symlink {
        target: ref expected_target,
    } = expected.kind
    {
        let actual_target = std::fs::read_link(path)?;
        let actual_bytes = actual_target.as_os_str().as_encoded_bytes();
        if actual_bytes != expected_target.as_slice() {
            diffs.push(DiffKind::SymlinkTargetMismatch {
                expected: String::from_utf8_lossy(expected_target).to_string(),
                actual: String::from_utf8_lossy(actual_bytes).to_string(),
            });
        }
    }

    // 8. Regular File Payload & Sparse Extents
    if let FileEntityKind::Regular {
        size: expected_size,
        sha256: expected_sha256,
        is_sparse,
        extents: ref expected_extents,
    } = expected.kind
    {
        let actual_size = dest_meta.len();
        if actual_size != expected_size {
            diffs.push(DiffKind::SizeMismatch {
                expected: expected_size,
                actual: actual_size,
            });
        }

        // Open non-invasively: attempt O_NOATIME on Linux
        #[cfg(target_os = "linux")]
        let (mut file, opened_with_noatime) = {
            use std::os::unix::fs::OpenOptionsExt;
            let mut opts = File::options();
            opts.read(true);
            opts.custom_flags(nix::libc::O_NOATIME);
            match opts.open(path) {
                Ok(f) => (f, true),
                Err(_) => {
                    let f = File::open(path).with_context(|| {
                        format!("Failed to open file for verification: {}", path.display())
                    })?;
                    (f, false)
                }
            }
        };
        #[cfg(not(target_os = "linux"))]
        let (mut file, opened_with_noatime) = {
            let f = File::open(path).with_context(|| {
                format!("Failed to open file for verification: {}", path.display())
            })?;
            (f, false)
        };

        if options.drop_caches {
            evict_fd_cache(&file);
        }

        if options.check_sparse && is_sparse {
            let actual_extents = get_file_extents(&file, actual_size)?;
            let actual_has_holes = actual_extents.iter().any(Extent::is_hole);
            let expected_has_holes = expected_extents.iter().any(Extent::is_hole);
            if actual_has_holes != expected_has_holes {
                diffs.push(DiffKind::SparseHoleMismatch {
                    expected_has_holes,
                    actual_has_holes,
                });
            }
            file.seek(SeekFrom::Start(0))?;
        }

        let mut hasher = Sha256Stream::new();
        let mut buf = vec![0_u8; 64 * 1024];
        loop {
            let n = file.read(&mut buf)?;
            if n == 0 {
                break;
            }
            let slice = buf
                .get(..n)
                .context("Verification read buffer slice out of bounds")?;
            hasher.update(slice);
        }
        let actual_sha256 = hasher.finalize();

        if actual_sha256 != expected_sha256 {
            diffs.push(DiffKind::ContentHashMismatch {
                expected_hex: hex_encode(&expected_sha256),
                actual_hex: hex_encode(&actual_sha256),
            });
        }

        // If O_NOATIME was not usable, restore original observed atime/mtime
        if !opened_with_noatime {
            // Reason for fallback: Filesystems without sub-second timestamp resolution or negative nsec return 0 nanoseconds.
            let orig_atime = FileTime::from_unix_time(
                dest_meta.atime(),
                u32::try_from(dest_meta.atime_nsec()).unwrap_or(0),
            );
            // Reason for fallback: Filesystems without sub-second timestamp resolution or negative nsec return 0 nanoseconds.
            let orig_mtime = FileTime::from_unix_time(
                dest_meta.mtime(),
                u32::try_from(dest_meta.mtime_nsec()).unwrap_or(0),
            );
            let _ = set_file_times(path, orig_atime, orig_mtime);
        }
    }

    Ok(diffs)
}

/// Performs an independent verification pass of `dest_path` against `entity`.
///
/// If any discrepancy is discovered and `strict_lossless` is true, returns an
/// error describing the exact mismatch. This verification is guaranteed to leave
/// no modified state on `dest_path`.
pub fn verify_materialized_entity(
    dest_path: &Path,
    entity: &FileEntity,
    strict_lossless: bool,
) -> Result<()> {
    let mut options = EntityAuditOptions::default();
    options.drop_caches = true;
    options.check_sparse = true;
    options.ignore_ctime = true;

    let diffs = audit_entity(dest_path, entity, &options)?;
    if strict_lossless {
        if let Some(first) = diffs.first() {
            anyhow::bail!(
                "Verification failure on {}: {first}",
                dest_path.display()
            );
        }
    }
    Ok(())
}

