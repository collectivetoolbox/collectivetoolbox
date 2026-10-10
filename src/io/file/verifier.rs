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

use crate::entity::{FileEntity, FileEntityKind};
use crate::metadata::OsFamily;
use crate::payload::{Extent, get_file_extents};
use crate::streams::read_and_hash_streams;
use crate::sys_flags::{query_file_flags, raw_flags_match};
use filetime::FileTime;
#[cfg(unix)]
use nix::fcntl::{PosixFadviseAdvice, posix_fadvise};
#[cfg(unix)]
use std::os::unix::fs::{FileTypeExt, MetadataExt};
#[cfg(windows)]
use std::os::windows::fs::MetadataExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
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
        expected_nsec: Option<u32>,
        actual_sec: Option<i64>,
        actual_nsec: Option<u32>,
    },
    /// Semantic or OS file flags mismatch.
    FlagsMismatch {
        expected: Vec<String>,
        actual: Vec<String>,
    },
    NativeMetadataMismatch {
        details: String,
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
                expected_nsec,
                actual_sec,
                actual_nsec,
            } => {
                write!(
                    f,
                    "Birthtime mismatch: expected {expected_sec:?}/{expected_nsec:?}, got {actual_sec:?}/{actual_nsec:?}"
                )
            }
            Self::FlagsMismatch { expected, actual } => {
                write!(f, "Flags mismatch: expected {expected:?}, got {actual:?}")
            }
            Self::NativeMetadataMismatch { details } => write!(f, "Native metadata mismatch: {details}"),
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

/// Summary of differences that were ignored due to best-effort mode or ignore flags.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct IgnoredDifferences {
    pub ownership: usize,
    pub timestamps: usize,
    pub permissions: usize,
    pub flags: usize,
    pub sparseness: usize,
}

impl IgnoredDifferences {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.ownership == 0
            && self.timestamps == 0
            && self.permissions == 0
            && self.flags == 0
            && self.sparseness == 0
    }

    #[must_use]
    pub fn total(&self) -> usize {
        self.ownership
            .saturating_add(self.timestamps)
            .saturating_add(self.permissions)
            .saturating_add(self.flags)
            .saturating_add(self.sparseness)
    }
}

/// Options controlling which attributes and checks are audited by `audit_entity`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityAuditOptions {
    /// Skip checking access times (atime) during verification. Access times are
    /// still copied to the destination during transfer.
    pub ignore_atime: bool,
    /// Skip checking modification times (mtime) during verification.
    /// Modification times are still copied to the destination during transfer.
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
    /// Best effort mode: tolerate timestamp differences up to 2 seconds and
    /// ignore owner UID/GID differences when running unprivileged.
    pub best_effort: bool,
}

impl Default for EntityAuditOptions {
    fn default() -> Self {
        Self {
            ignore_atime: true,
            ignore_mtime: false,
            ignore_ctime: true,
            ignore_owner: false,
            ignore_perms: false,
            ignore_flags: false,
            ignore_xattrs: false,
            check_sparse: true,
            drop_caches: false,
            best_effort: false,
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
    #[cfg(unix)]
    {
        let _ = posix_fadvise(file, 0, 0, PosixFadviseAdvice::POSIX_FADV_DONTNEED);
    }
    #[cfg(not(unix))]
    {
        let _ = file;
    }
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

/// Returns true if global kernel cache dropping (`/proc/sys/vm/drop_caches`) is available.
#[must_use]
pub fn has_cache_flush_privileges() -> bool {
    #[cfg(unix)]
    {
        nix::unistd::geteuid().is_root()
            || std::fs::OpenOptions::new()
                .write(true)
                .open("/proc/sys/vm/drop_caches")
                .is_ok()
    }
    #[cfg(not(unix))]
    {
        true
    }
}

/// Audits an on-disk filesystem entry at `path` against an `expected` entity.
///
/// Returns a list of detected discrepancies (`Vec<DiffKind>`). An empty return
/// indicates the entry on disk completely matches the expected entity.
///
/// Regular payload reads attempt `O_NOATIME` on Linux. Otherwise, changed
/// access times are restored through the open handle, including on read errors.
/// Restoration never writes mtime. Unix cannot restore the resulting ctime
/// change; stream and symlink reads have separate timestamp limitations.
/// Audits a materialized filesystem entry at `path` against an `expected` FileEntity definition,
/// returning both detected discrepancies and a count of differences that were ignored due to
/// best-effort tolerance or ignore options.
#[expect(clippy::too_many_lines, reason = "Comprehensive audit of all entity attributes, hashes, streams, and extents")]
pub fn audit_entity_detailed(
    path: &Path,
    expected: &FileEntity,
    options: &EntityAuditOptions,
) -> Result<(Vec<DiffKind>, IgnoredDifferences)> {
    #[cfg(not(any(unix, windows)))]
    anyhow::ensure!(options.ignore_owner && options.ignore_perms && options.ignore_flags
        && options.ignore_atime && options.ignore_mtime && options.ignore_ctime && !options.check_sparse,
        "Native metadata and sparse auditing is not implemented on this platform");
    let mut diffs = Vec::new();
    let mut ignored = IgnoredDifferences::default();

    let dest_meta = match std::fs::symlink_metadata(path) {
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Ok((vec![DiffKind::MissingOnDisk], ignored));
        }
        Err(err) => {
            return Err(err).with_context(|| {
                format!("Failed to read metadata for {}", path.display())
            });
        }
        Ok(m) => m,
    };

    // 1. File Type Check
    #[cfg(unix)]
    let special_matches = match &expected.kind {
        FileEntityKind::Fifo => dest_meta.file_type().is_fifo(),
        FileEntityKind::CharDevice { rdev } => dest_meta.file_type().is_char_device() && dest_meta.rdev() == *rdev,
        FileEntityKind::BlockDevice { rdev } => dest_meta.file_type().is_block_device() && dest_meta.rdev() == *rdev,
        FileEntityKind::Socket => dest_meta.file_type().is_socket(),
        FileEntityKind::Door => dest_meta.mode() & 0xF000 == 0xD000,
        _ => true,
    };
    #[cfg(not(unix))]
    let special_matches = expected.is_regular() || expected.is_dir() || expected.is_symlink();
    anyhow::ensure!(!matches!(expected.kind, FileEntityKind::Hardlink { .. }),
        "Hardlink auditing requires the manifest root to resolve the target");
    let type_matches = if expected.is_symlink() {
        dest_meta.is_symlink()
    } else if expected.is_dir() {
        dest_meta.is_dir()
    } else if matches!(expected.kind, FileEntityKind::Regular { .. }) {
        dest_meta.is_file()
    } else {
        special_matches
    };

    if !type_matches {
        let expected_str = if expected.is_symlink() {
            "symlink".to_string()
        } else if expected.is_dir() {
            "directory".to_string()
        } else {
            expected.kind.kind_str().to_string()
        };
        diffs.push(DiffKind::TypeMismatch {
            expected: expected_str,
            actual: describe_file_type(&dest_meta),
        });
        return Ok((diffs, ignored));
    }

    #[cfg(windows)]
    {
        anyhow::ensure!(
            options.ignore_owner
                || (expected.metadata.uid.is_none() && expected.metadata.gid.is_none()),
            "Windows cannot audit POSIX ownership"
        );
        anyhow::ensure!(
            options.ignore_perms || expected.metadata.mode.is_none(),
            "Windows cannot audit POSIX permission modes"
        );
    }

    // 2. Mode / Permissions (skip symlinks as their permissions are not meaningful on Linux)
    #[cfg(unix)]
    if !expected.is_symlink() {
        if let Some(mode) = expected.metadata.mode {
            let expected_mode = mode & 0o7777;
            let actual_mode = dest_meta.mode() & 0o7777;
            if expected_mode != actual_mode {
                if options.ignore_perms {
                    ignored.permissions = ignored.permissions.saturating_add(1);
                } else {
                    diffs.push(DiffKind::ModeMismatch {
                        expected: format!("{expected_mode:#05o}"),
                        actual: format!("{actual_mode:#05o}"),
                    });
                }
            }
        }
    }

    // 3. Ownership
    #[cfg(unix)]
    {
        if let (Some(exp_uid), Some(exp_gid)) = (expected.metadata.uid, expected.metadata.gid) {
            let uid_diff = dest_meta.uid() != exp_uid;
            let gid_diff = dest_meta.gid() != exp_gid;
            if uid_diff || gid_diff {
                let is_root = nix::unistd::getuid().is_root();

                let ignore_owner = options.ignore_owner
                    || (options.best_effort && !is_root);
                if ignore_owner {
                    ignored.ownership = ignored.ownership.saturating_add(1);
                } else {
                    if uid_diff {
                        diffs.push(DiffKind::UidMismatch {
                            expected: exp_uid,
                            actual: dest_meta.uid(),
                        });
                    }
                    if gid_diff {
                        diffs.push(DiffKind::GidMismatch {
                            expected: exp_gid,
                            actual: dest_meta.gid(),
                        });
                    }
                }
            }
        }
    }

    // 4. Timestamps
    #[cfg(unix)]
    let actual_mtime_sec = dest_meta.mtime();
    #[cfg(unix)]
    let actual_mtime_nsec = u32::try_from(dest_meta.mtime_nsec())
        .context("Failed to convert mtime nanoseconds to u32")?;
    #[cfg(unix)]
    let actual_atime_sec = dest_meta.atime();
    #[cfg(unix)]
    let actual_atime_nsec = u32::try_from(dest_meta.atime_nsec())
        .context("Failed to convert atime nanoseconds to u32")?;
    #[cfg(unix)]
    let actual_ctime_sec = dest_meta.ctime();
    #[cfg(unix)]
    let actual_ctime_nsec = u32::try_from(dest_meta.ctime_nsec())
        .context("Failed to convert ctime nanoseconds to u32")?;

    #[cfg(windows)]
    let actual_mtime = FileTime::from_last_modification_time(&dest_meta);
    #[cfg(windows)]
    let actual_mtime_sec = actual_mtime.unix_seconds();
    #[cfg(windows)]
    let actual_mtime_nsec = actual_mtime.nanoseconds();
    #[cfg(windows)]
    let actual_atime = FileTime::from_last_access_time(&dest_meta);
    #[cfg(windows)]
    let actual_atime_sec = actual_atime.unix_seconds();
    #[cfg(windows)]
    let actual_atime_nsec = actual_atime.nanoseconds();
    #[cfg(windows)]
    let actual_ctime = crate::metadata::windows::query_file_change_time(path)?;
    #[cfg(windows)]
    let actual_ctime_sec = actual_ctime.unix_seconds();
    #[cfg(windows)]
    let actual_ctime_nsec = actual_ctime.nanoseconds();
    #[cfg(any(unix, windows))]
    if let Some(ref exp_ts) = expected.metadata.timestamps {
        let dest_res = crate::filesystem::query_filesystem_resolution(path, &dest_meta);
        // Reason for fallback: unspecified timestamp resolution defaults to 0 nanosecond tolerance
        let tolerance_nsec = dest_res.max(exp_ts.resolution_nsec.unwrap_or(0));

        let mtime_mismatch = actual_mtime_sec != exp_ts.mtime_sec
            || actual_mtime_nsec != exp_ts.mtime_nsec;
        if mtime_mismatch {
            if options.ignore_mtime {
                ignored.timestamps = ignored.timestamps.saturating_add(1);
            } else if options.best_effort
                && is_timestamp_acceptable_best_effort(
                    actual_mtime_sec,
                    actual_mtime_nsec,
                    exp_ts.mtime_sec,
                    exp_ts.mtime_nsec,
                    tolerance_nsec,
                )
            {
                ignored.timestamps = ignored.timestamps.saturating_add(1);
            } else {
                diffs.push(DiffKind::MtimeMismatch {
                    expected_sec: exp_ts.mtime_sec,
                    expected_nsec: exp_ts.mtime_nsec,
                    actual_sec: actual_mtime_sec,
                    actual_nsec: actual_mtime_nsec,
                });
            }
        }

        let atime_mismatch = actual_atime_sec != exp_ts.atime_sec
            || actual_atime_nsec != exp_ts.atime_nsec;
        if atime_mismatch {
            if options.ignore_atime {
                if options.best_effort {
                    ignored.timestamps = ignored.timestamps.saturating_add(1);
                }
            } else if options.best_effort
                && is_timestamp_acceptable_best_effort(
                    actual_atime_sec,
                    actual_atime_nsec,
                    exp_ts.atime_sec,
                    exp_ts.atime_nsec,
                    tolerance_nsec,
                )
            {
                ignored.timestamps = ignored.timestamps.saturating_add(1);
            } else {
                diffs.push(DiffKind::AtimeMismatch {
                    expected_sec: exp_ts.atime_sec,
                    expected_nsec: exp_ts.atime_nsec,
                    actual_sec: actual_atime_sec,
                    actual_nsec: actual_atime_nsec,
                });
            }
        }

        {
            let ctime_mismatch = actual_ctime_sec != exp_ts.ctime_sec
                || actual_ctime_nsec != exp_ts.ctime_nsec;
            if ctime_mismatch && !options.ignore_ctime {
                if options.best_effort
                    && is_timestamp_acceptable_best_effort(
                        actual_ctime_sec,
                        actual_ctime_nsec,
                        exp_ts.ctime_sec,
                        exp_ts.ctime_nsec,
                        tolerance_nsec,
                    )
                {
                    ignored.timestamps = ignored.timestamps.saturating_add(1);
                } else {
                    diffs.push(DiffKind::CtimeMismatch {
                        expected_sec: exp_ts.ctime_sec,
                        expected_nsec: exp_ts.ctime_nsec,
                        actual_sec: actual_ctime_sec,
                        actual_nsec: actual_ctime_nsec,
                    });
                }
            }
        }

        if let Some(expected_sec) = exp_ts.birthtime_sec {
            let actual = crate::metadata::capture_birthtime(&dest_meta)?;
            let actual_sec = actual.map(|time| time.unix_seconds());
            let actual_nsec = actual.map(|time| time.nanoseconds());
            if actual_sec != Some(expected_sec) || actual_nsec != exp_ts.birthtime_nsec {
                if options.best_effort {
                    ignored.timestamps = ignored.timestamps.saturating_add(1);
                    warn_fmt!("Birth time differs on {}; original retained in source metadata", path.display());
                } else {
                    diffs.push(DiffKind::BirthtimeMismatch {
                        expected_sec: Some(expected_sec),
                        expected_nsec: exp_ts.birthtime_nsec,
                        actual_sec, actual_nsec,
                    });
                }
            }
        }
    }

    if expected.metadata.native.is_some() {
        let differences = crate::metadata::native_metadata_differences(path, &expected.metadata, options.ignore_flags)?;
        if !differences.is_empty() {
            if options.best_effort {
                warn_fmt!("Native metadata differs on {}: {differences:?}", path.display());
                ignored.flags = ignored.flags.saturating_add(1);
            } else {
                diffs.push(DiffKind::NativeMetadataMismatch { details: differences.join("; ") });
            }
        }
    }

    // 5. File Flags
    if !options.ignore_flags
        && (expected.metadata.flags.is_some()
            || expected.metadata.platform_raw_flags.is_some())
    {
        let (actual_flags, actual_raw) = query_file_flags(path, expected.is_symlink())?;
        if let Some(expected_raw) = &expected.metadata.platform_raw_flags {
            let matches_raw = actual_raw
                .as_ref()
                .is_some_and(|actual| raw_flags_match(expected_raw, actual));
            if !matches_raw {
                let details = format!("Raw flags expected {expected_raw:?}, got {actual_raw:?}");
                if options.best_effort {
                    warn_fmt!("{}: {details}", path.display());
                    ignored.flags = ignored.flags.saturating_add(1);
                } else {
                    diffs.push(DiffKind::NativeMetadataMismatch { details });
                }
            }
        }
        // Reason for fallback: absent flags metadata defaults to empty slice for verification comparison
        let exp_flags = expected.metadata.flags.as_deref().unwrap_or(&[]);
        let mut exp_names: Vec<String> = exp_flags
            .iter()
            .map(|f| f.name().to_string())
            .collect();
        let mut act_names: Vec<String> = actual_flags
            .iter()
            .filter(|f| exp_flags.contains(f) || f.settability(OsFamily::CURRENT).is_settable())
            .map(|f| f.name().to_string())
            .collect();
        exp_names.sort();
        act_names.sort();
        if exp_names != act_names {
            #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
            let is_default_bsd_uarch = exp_names.is_empty()
                && act_names.as_slice() == ["uarch"];
            #[cfg(not(any(target_os = "freebsd", target_os = "dragonfly")))]
            let is_default_bsd_uarch = false;

            if options.ignore_flags || is_default_bsd_uarch {
                ignored.flags = ignored.flags.saturating_add(1);
            } else {
                diffs.push(DiffKind::FlagsMismatch {
                    expected: exp_names,
                    actual: act_names,
                });
            }
        }
    }

    // 6. Streams and Extended Attributes
    if !options.ignore_xattrs {
        let mut on_disk_streams = read_and_hash_streams(path)?;
        if on_disk_streams.is_empty() && !expected.streams.is_empty() {
            let mut check_entity = expected.clone();
            check_entity.streams.clear();
            let all_apple_opts = crate::apple_double::AppleReadOptions {
                read_apple_double_alongside: true,
                read_apple_double_zip: true,
                read_apple_double_netatalk: true,
                read_apple_single_without_extension: true,
                read_apple_single: true,
                read_apple_single_as: true,
                read_apple_single_asf: true,
            };
            if crate::apple_double::join_apple_double_or_single(&mut check_entity, path, None, &all_apple_opts).is_ok() {
                on_disk_streams = check_entity.streams;
            }
        }
        let mut expected_map = HashMap::new();
        for s in &expected.streams {
            let hash = match &s.entity.kind {
                FileEntityKind::Regular { sha256, .. } => *sha256,
                _ => [0_u8; 32],
            };
            let native_name = match &s.name {
                Some(name) => name.canonicalize_native()?,
                None => match s.kind {
                    crate::streams::StreamKind::MacOsResourceFork => {
                        crate::streams::StreamName::from_str(
                            crate::streams::RESOURCE_FORK_XATTR_NAME,
                        ).canonicalize_native()?
                    }
                    _ => continue,
                },
            };
            anyhow::ensure!(
                expected_map.insert(native_name, hash).is_none(),
                "Stream names collide on the destination platform"
            );
        }

        let mut disk_map = HashMap::new();
        for s in &on_disk_streams {
            let hash = match &s.entity.kind {
                FileEntityKind::Regular { sha256, .. } => *sha256,
                _ => [0_u8; 32],
            };
            let native_name = match &s.name {
                Some(name) => name.canonicalize_native()?,
                None => match s.kind {
                    crate::streams::StreamKind::MacOsResourceFork => {
                        crate::streams::StreamName::from_str(
                            crate::streams::RESOURCE_FORK_XATTR_NAME,
                        ).canonicalize_native()?
                    }
                    _ => continue,
                },
            };
            disk_map.insert(native_name, hash);
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
        let actual_target = crate::metadata::read_symlink_target(path, &dest_meta)?;
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
        ref extents,
    } = expected.kind
    {
        audit_file_contents(path, |file, opened_meta| {
            anyhow::ensure!(
                opened_meta.len() == dest_meta.len()
                    && opened_meta.modified()? == dest_meta.modified()?,
                "File changed before payload verification: {}", path.display()
            );
            #[cfg(unix)]
            anyhow::ensure!(
                opened_meta.dev() == dest_meta.dev() && opened_meta.ino() == dest_meta.ino(),
                "File replaced before payload verification: {}", path.display()
            );
            let apple_single_archive = if dest_meta.len() == expected_size {
                None
            } else {
                let mut magic = [0u8; 4];
                let is_single = match file.read_exact(&mut magic) {
                    Ok(()) => {
                        let m = u32::from_be_bytes(magic);
                        m == crate::apple_double::APPLESINGLE_MAGIC_BE
                            || m == crate::apple_double::APPLESINGLE_MAGIC_LE
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => false,
                    Err(error) => return Err(error).context("Failed to read audit payload magic"),
                };
                file.seek(SeekFrom::Start(0))?;
                if is_single {
                    let mut data = Vec::new();
                    file.read_to_end(&mut data)?;
                    let archive = ctb_formats_apple_single_double::read_apple_single_double(&data)
                        .context("Invalid AppleSingle audit payload")?;
                    if archive.format == ctb_formats_apple_single_double::AppleFormat::AppleSingle {
                        Some(archive)
                    } else {
                        None
                    }
                } else {
                    None
                }
            };

            if let Some(ref archive) = apple_single_archive {
                // Reason for fallback: AppleSingle archive without a data fork entry represents an empty 0-byte file payload
                let data_fork = archive.data_fork.as_deref().unwrap_or(&[]);
                let actual_size = u64::try_from(data_fork.len())?;
                if actual_size != expected_size {
                    diffs.push(DiffKind::SizeMismatch {
                        expected: expected_size,
                        actual: actual_size,
                    });
                }
                let mut hasher = ctb_formats_checksum::Sha256Stream::new();
                hasher.update(data_fork);
                let actual_sha256 = hasher.finalize();
                if actual_sha256 != expected_sha256 {
                    diffs.push(DiffKind::ContentHashMismatch {
                        expected_hex: hex_encode(&expected_sha256),
                        actual_hex: hex_encode(&actual_sha256),
                    });
                }
            } else {
                file.seek(SeekFrom::Start(0))?;
                let actual_size = dest_meta.len();
                if actual_size != expected_size {
                    diffs.push(DiffKind::SizeMismatch {
                        expected: expected_size,
                        actual: actual_size,
                    });
                }

                if options.drop_caches {
                    evict_fd_cache(file);
                }

                if options.check_sparse {
                    let actual_extents = get_file_extents(file, actual_size)?;
                    let actual_has_holes = actual_extents.iter().any(Extent::is_hole);
                    let expected_has_holes = extents.iter().any(Extent::is_hole);
                    #[cfg(windows)]
                    let actual_sparse_attr = dest_meta.file_attributes()
                        & windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_SPARSE_FILE != 0;

                    let expected_sparse = is_sparse || expected_has_holes;
                    let actual_sparse = {
                        #[cfg(windows)]
                        {
                            actual_has_holes || actual_sparse_attr
                        }
                        #[cfg(not(windows))]
                        {
                            actual_has_holes
                        }
                    };

                    let mut missing_holes = Vec::new();
                    for extent in extents {
                        if let Extent::Hole { offset, length } = extent {
                            let is_covered = actual_extents.iter().any(|actual| match actual {
                                Extent::Hole { offset: a_off, length: a_len } => {
                                    *a_off <= *offset && a_off.saturating_add(*a_len) >= offset.saturating_add(*length)
                                }
                                Extent::Data { .. } => false,
                            });
                            if !is_covered {
                                missing_holes.push((*offset, *length));
                            }
                        }
                    }

                    let sparse_mismatch = expected_sparse != actual_sparse || !missing_holes.is_empty();
                    if sparse_mismatch {
                        let mut tolerate = false;
                        if options.best_effort {
                            let fs_info = crate::filesystem::query_filesystem_info(path, &dest_meta);
                            let fs_supports_sparse = fs_info.supports_sparse() == Some(true);

                            let sub_allocation_unit_only = {
                                #[cfg(windows)]
                                {
                                    !missing_holes.is_empty()
                                        && missing_holes.iter().all(|(_, len)| *len < 65_536)
                                }
                                #[cfg(not(windows))]
                                {
                                    #[cfg(unix)]
                                    {
                                        use std::os::unix::fs::MetadataExt;
                                        let min_granularity = dest_meta.blksize().max(4096);
                                        !missing_holes.is_empty()
                                            && missing_holes.iter().all(|(_, len)| *len < min_granularity)
                                    }
                                    #[cfg(not(unix))]
                                    {
                                        false
                                    }
                                }
                            };

                            if !fs_supports_sparse || sub_allocation_unit_only {
                                tolerate = true;
                            }
                        }

                        if tolerate {
                            ignored.sparseness = ignored.sparseness.saturating_add(1);
                        } else {
                            diffs.push(DiffKind::SparseHoleMismatch {
                                expected_has_holes: expected_sparse,
                                actual_has_holes: actual_sparse,
                            });
                        }
                    }
                    file.seek(SeekFrom::Start(0))?;
                }

                let actual_extents = if is_sparse {
                    get_file_extents(file, actual_size)?
                } else {
                    Vec::new()
                };
                let actual_sha256 = crate::payload::hash_payload_stream(
                    file,
                    &actual_extents,
                    is_sparse,
                    path,
                )?;

                if actual_sha256 != expected_sha256 {
                    diffs.push(DiffKind::ContentHashMismatch {
                        expected_hex: hex_encode(&expected_sha256),
                        actual_hex: hex_encode(&actual_sha256),
                    });
                }

            }
            Ok(())
        })?;
    }

    Ok((diffs, ignored))
}

/// Audits an on-disk filesystem entry at `path` against an `expected` entity.
///
/// Returns a list of detected discrepancies (`Vec<DiffKind>`). An empty return
/// indicates the entry on disk completely matches the expected entity.
///
/// Regular payload reads preserve access time where supported, or restore it
/// through the open handle. Restoration errors are returned. Unix ctime and
/// separate stream/symlink reads prevent a universal non-invasive guarantee.
pub fn audit_entity(
    path: &Path,
    expected: &FileEntity,
    options: &EntityAuditOptions,
) -> Result<Vec<DiffKind>> {
    let (diffs, _) = audit_entity_detailed(path, expected, options)?;
    Ok(diffs)
}

fn audit_file_contents<T>(
    path: &Path,
    read: impl FnOnce(&mut File, &std::fs::Metadata) -> Result<T>,
) -> Result<T> {
    #[cfg(target_os = "linux")]
    let mut file = {
        use std::os::unix::fs::OpenOptionsExt;
        match File::options().read(true).custom_flags(nix::libc::O_NOATIME).open(path) {
            Ok(file) => file,
            Err(_) => File::open(path)?,
        }
    };
    #[cfg(windows)]
    let mut file = {
        use std::os::windows::fs::OpenOptionsExt;
        use windows_sys::Win32::Foundation::GENERIC_READ;
        use windows_sys::Win32::Storage::FileSystem::FILE_WRITE_ATTRIBUTES;
        match File::options().read(true).access_mode(GENERIC_READ | FILE_WRITE_ATTRIBUTES).open(path) {
            Ok(file) => file,
            Err(_) => File::open(path)?,
        }
    };
    #[cfg(not(any(target_os = "linux", windows)))]
    let mut file = File::open(path)?;
    let original = file.metadata()?;
    let result = read(&mut file, &original);
    let completion = complete_audit_read(&file, &original);
    match (result, completion) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), Ok(())) => Err(error),
        (Ok(_), Err(error)) => Err(error)
            .with_context(|| format!("Failed to complete audit read on {}", path.display())),
        (Err(error), Err(cleanup)) => Err(error).context(format!(
            "Audit read cleanup also failed on {}: {cleanup:#}", path.display()
        )),
    }
}

fn complete_audit_read(file: &File, original: &std::fs::Metadata) -> Result<()> {
    let current = file.metadata()?;
    anyhow::ensure!(
        current.len() == original.len() && current.modified()? == original.modified()?,
        "File changed during payload verification; refusing timestamp restoration"
    );
    let atime = FileTime::from_last_access_time(original);
    if FileTime::from_last_access_time(&current) != atime {
        filetime::set_file_handle_times(file, Some(atime), None)
            .context("Failed to restore audit access time")?;
    }
    Ok(())
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
    verify_materialized_entity_ext(dest_path, entity, strict_lossless, false, false)
}

/// Verifies a materialized entity with opt-in control over access time (atime) and change time (ctime) checking.
pub fn verify_materialized_entity_ext(
    dest_path: &Path,
    entity: &FileEntity,
    strict_lossless: bool,
    check_atime: bool,
    check_ctime: bool,
) -> Result<()> {
    let mut options = EntityAuditOptions::default();
    options.drop_caches = true;
    options.check_sparse = true;
    options.ignore_ctime = !check_ctime;
    options.ignore_atime = !check_atime;
    options.best_effort = !strict_lossless;

    let diffs = audit_entity(dest_path, entity, &options)?;
    for difference in &diffs {
        if strict_lossless {
            anyhow::bail!(
                "Verification failure on {}: {difference}",
                dest_path.display()
            );
        } else {
            match difference {
                DiffKind::ContentHashMismatch { .. }
                | DiffKind::SizeMismatch { .. }
                | DiffKind::MissingOnDisk
                | DiffKind::TypeMismatch { .. }
                | DiffKind::SymlinkTargetMismatch { .. }
                | DiffKind::StreamMismatch { .. }
                | DiffKind::HardlinkMismatch { .. } => {
                    anyhow::bail!(
                        "Verification payload integrity failure on {}: {difference}",
                        dest_path.display()
                    );
                }
                _ => {
                    warn_fmt!(
                        "Verification metadata warning on {}: {difference}",
                        dest_path.display()
                    );
                }
            }
        }
    }
    Ok(())
}

/// Determines whether a destination timestamp difference is acceptable under
/// best-effort mode, taking into account filesystem timestamp resolution.
pub(crate) fn is_timestamp_acceptable_best_effort(
    actual_sec: i64,
    _actual_nsec: u32,
    expected_sec: i64,
    _expected_nsec: u32,
    tolerance_nsec: u32,
) -> bool {
    let sec_diff = actual_sec.saturating_sub(expected_sec);
    // Reason for fallback: division calculation overflow fallback defaults to 2 seconds
    let sec_tol = i64::from(
        tolerance_nsec
            .saturating_add(999_999_999)
            .checked_div(1_000_000_000)
            .unwrap_or(2)
            .max(2),
    );
    sec_diff.abs() <= sec_tol
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
mod tests {
    use super::*;
    use crate::*;
    use std::fs;
    use std::fs::File;
    #[cfg(unix)]
    use std::os::unix::fs::{PermissionsExt};

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_symlink_audit_preserves_observed_not_expected_timestamps() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("link");
        std::os::unix::fs::symlink("target", &path).unwrap();
        let entity = FileEntity::from_filesystem(&path, None).unwrap();
        let observed = FileTime::from_unix_time(1_000_000_000, 0);
        filetime::set_symlink_file_times(&path, observed, observed).unwrap();
        let differences = audit_entity(&path, &entity, &EntityAuditOptions::default()).unwrap();
        assert!(differences.iter().any(|diff| matches!(diff, DiffKind::MtimeMismatch { .. })));
        let after = fs::symlink_metadata(&path).unwrap();
        assert_eq!(FileTime::from_last_modification_time(&after), observed);
        assert_eq!(FileTime::from_last_access_time(&after), observed);
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_symlink_cleanup_rejects_changes_and_replacement() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("link");
        std::os::unix::fs::symlink("original", &path).unwrap();
        let original = fs::symlink_metadata(&path).unwrap();
        let changed = FileTime::from_unix_time(1_000_000_000, 0);
        filetime::set_symlink_file_times(&path, changed, changed).unwrap();
        assert!(crate::metadata::restore_symlink_access_time(&path, &original).is_err());
        assert_eq!(FileTime::from_last_modification_time(&fs::symlink_metadata(&path).unwrap()), changed);
        let changed_metadata = fs::symlink_metadata(&path).unwrap();
        let replacement = temp.path().join("replacement");
        std::os::unix::fs::symlink("replacement-target", &replacement).unwrap();
        fs::rename(&replacement, &path).unwrap();
        let replacement_metadata = fs::symlink_metadata(&path).unwrap();
        assert!(crate::metadata::restore_symlink_access_time(&path, &changed_metadata).is_err());
        let after = fs::symlink_metadata(&path).unwrap();
        assert_eq!(FileTime::from_last_access_time(&after), FileTime::from_last_access_time(&replacement_metadata));
        assert_eq!(FileTime::from_last_modification_time(&after), FileTime::from_last_modification_time(&replacement_metadata));
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_symlink_read_reports_read_and_cleanup_failures() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("link");
        std::os::unix::fs::symlink("target", &path).unwrap();
        let observed = fs::symlink_metadata(&path).unwrap();
        fs::remove_file(&path).unwrap();
        let error = crate::metadata::read_symlink_target(&path, &observed).unwrap_err();
        let message = format!("{error:#}");
        assert!(message.contains("Failed to read symlink target"));
        assert!(message.contains("Symlink read cleanup also failed"));
    }

    #[crate::ctb_test]
    fn test_audit_completion_preserves_concurrent_mtime() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("modified");
        fs::write(&path, b"original").unwrap();
        let file = File::options().read(true).write(true).open(&path).unwrap();
        let original = file.metadata().unwrap();
        let changed = FileTime::from_unix_time(1_000_000_000, 0);
        filetime::set_file_handle_times(&file, None, Some(changed)).unwrap();
        let error = complete_audit_read(&file, &original).unwrap_err();
        assert!(error.to_string().contains("changed during payload verification"));
        assert_eq!(FileTime::from_last_modification_time(&file.metadata().unwrap()), changed);
    }

    #[crate::ctb_test]
    fn test_audit_completion_restores_handle_not_replacement() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("original");
        fs::write(&path, b"original").unwrap();
        let file = File::options().read(true).write(true).open(&path).unwrap();
        let original = file.metadata().unwrap();
        fs::rename(&path, temp.path().join("moved")).unwrap();
        fs::write(&path, b"replacement").unwrap();
        let replacement_time = FileTime::from_unix_time(1_100_000_000, 0);
        filetime::set_file_times(&path, replacement_time, replacement_time).unwrap();
        filetime::set_file_handle_times(
            &file, Some(FileTime::from_unix_time(1_000_000_000, 0)), None,
        ).unwrap();
        complete_audit_read(&file, &original).unwrap();
        assert_eq!(
            FileTime::from_last_access_time(&file.metadata().unwrap()),
            FileTime::from_last_access_time(&original)
        );
        let replacement = fs::metadata(&path).unwrap();
        assert_eq!(FileTime::from_last_access_time(&replacement), replacement_time);
        assert_eq!(FileTime::from_last_modification_time(&replacement), replacement_time);
    }

    #[crate::ctb_test]
    fn test_audit_read_error_still_completes() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("read-error");
        fs::write(&path, b"payload").unwrap();
        let original = fs::metadata(&path).unwrap();
        let error = audit_file_contents(&path, |_, _| -> Result<()> {
            filetime::set_file_atime(&path, FileTime::from_unix_time(1_000_000_000, 0))?;
            anyhow::bail!("injected read failure")
        }).unwrap_err();
        assert_eq!(error.to_string(), "injected read failure");
        assert_eq!(
            FileTime::from_last_access_time(&fs::metadata(&path).unwrap()),
            FileTime::from_last_access_time(&original)
        );
    }

    #[crate::ctb_test]
    fn test_audit_read_and_completion_errors_are_both_reported() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("read-and-change");
        fs::write(&path, b"payload").unwrap();
        let changed = FileTime::from_unix_time(1_000_000_000, 0);
        let error = audit_file_contents(&path, |_, _| -> Result<()> {
            filetime::set_file_mtime(&path, changed)?;
            anyhow::bail!("injected read failure")
        }).unwrap_err();
        let diagnostic = format!("{error:#}");
        assert!(diagnostic.contains("injected read failure"));
        assert!(diagnostic.contains("changed during payload verification"));
        assert_eq!(
            FileTime::from_last_modification_time(&fs::metadata(&path).unwrap()), changed
        );
    }

    #[cfg(windows)]
    #[crate::ctb_test]
    fn test_windows_change_time_capture_and_audit() {
        let epoch = 116_444_736_000_000_000_i64;
        for (ticks, seconds, nanos) in [
            (0, -11_644_473_600, 0),
            (epoch.saturating_sub(1), -1, 999_999_900),
            (epoch, 0, 0),
            (epoch.saturating_add(12_345_678), 1, 234_567_800),
        ] {
            let time = crate::metadata::windows::filetime_from_windows_ticks(ticks).unwrap();
            assert_eq!((time.unix_seconds(), time.nanoseconds()), (seconds, nanos));
        }
        assert!(crate::metadata::windows::filetime_from_windows_ticks(-1).is_err());
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("change_time");
        fs::write(&path, b"native timestamp payload").unwrap();
        filetime::set_file_mtime(&path, FileTime::from_unix_time(1_000_000_000, 0)).unwrap();
        let mut entity = FileEntity::from_filesystem(&path, None).unwrap();
        let actual = crate::metadata::windows::query_file_change_time(&path).unwrap();
        let timestamps = entity.metadata.timestamps.as_ref().unwrap();
        assert_eq!((timestamps.ctime_sec, timestamps.ctime_nsec), (actual.unix_seconds(), actual.nanoseconds()));
        assert_ne!(timestamps.ctime_sec, timestamps.mtime_sec);
        assert!(entity.metadata.warnings.iter().any(|warning| warning.contains("change-time restoration is not implemented")));
        entity.metadata.environment = None;
        entity.metadata.read_time = None;
        ctb_formats_dcstring::assert_dc_roundtrip(&entity.metadata).unwrap();
        let options = EntityAuditOptions { ignore_ctime: false, ..EntityAuditOptions::default() };
        assert!(audit_entity(&path, &entity, &options).unwrap().is_empty());
        entity.metadata.timestamps.as_mut().unwrap().ctime_sec = actual.unix_seconds().saturating_sub(100);
        let differences = audit_entity(&path, &entity, &options).unwrap();
        assert!(differences.iter().any(|difference| matches!(difference, DiffKind::CtimeMismatch { .. })));
        assert!(!differences.iter().any(|difference| matches!(difference, DiffKind::ContentHashMismatch { .. })));
        let ignored = EntityAuditOptions { ignore_ctime: true, ..options };
        assert!(audit_entity(&path, &entity, &ignored).unwrap().is_empty());
    }

    #[cfg(windows)]
    #[crate::ctb_test]
    fn test_windows_default_materialization_audits_dense_payload() {
        let temp = tempfile::tempdir().unwrap();
        let data = b"verified payload".to_vec();
        let mut entity = FileEntity::from_vec(data.clone());
        entity.identity.relative_path = Some(PathBuf::from("payload"));
        let mut payload = MemoryPayloadSource::new(data).unwrap();
        let receipt = entity.materialize_at_path(
            Some(&mut payload), temp.path(), &MaterializeOptions::default(),
        ).unwrap();
        let options = EntityAuditOptions::default();
        assert!(audit_entity(&receipt.destination_path, &entity, &options)
            .unwrap().is_empty());
        let mut captured = FileEntity::from_filesystem(&receipt.destination_path, None).unwrap();
        assert_eq!(
            (captured.metadata.uid, captured.metadata.gid, captured.metadata.mode),
            (None, None, None),
        );
        let mut timestamp_entity = FileEntity::from_vec(b"old timestamp".to_vec());
        timestamp_entity.identity.relative_path = Some(PathBuf::from("restored_mtime"));
        let mut timestamps = captured.metadata.timestamps.clone().unwrap();
        timestamps.mtime_sec = 1_000_000_000;
        timestamps.mtime_nsec = 123_456_700;
        timestamps.birthtime_sec = None;
        timestamps.birthtime_nsec = None;
        timestamp_entity.metadata.timestamps = Some(timestamps);
        let mut timestamp_payload = MemoryPayloadSource::new(b"old timestamp".to_vec()).unwrap();
        let timestamp_receipt = timestamp_entity.materialize_at_path(
            Some(&mut timestamp_payload), temp.path(), &MaterializeOptions::default(),
        ).unwrap();
        let restored = FileTime::from_last_modification_time(
            &fs::metadata(&timestamp_receipt.destination_path).unwrap(),
        );
        assert_eq!((restored.unix_seconds(), restored.nanoseconds()), (1_000_000_000, 123_456_700));
        assert!(audit_entity(&timestamp_receipt.destination_path, &timestamp_entity, &options)
            .unwrap().is_empty());
        if let Some(ref mut timestamps) = captured.metadata.timestamps {
            timestamps.mtime_sec = timestamps.mtime_sec.saturating_add(1);
        } else {
            panic!("Windows capture omitted timestamps");
        }
        assert!(audit_entity(&receipt.destination_path, &captured, &options)
            .unwrap().iter()
            .any(|diff| matches!(diff, DiffKind::MtimeMismatch { .. })));
        fs::write(&receipt.destination_path, b"tampered payload").unwrap();
        assert!(audit_entity(&receipt.destination_path, &entity, &options)
            .unwrap().iter()
            .any(|diff| matches!(diff, DiffKind::ContentHashMismatch { .. })));
        entity.metadata.mode = Some(0o600);
        assert!(audit_entity(&receipt.destination_path, &entity, &options)
            .unwrap_err().to_string().contains("POSIX permission modes"));
        entity.metadata.mode = None;
        if let FileEntityKind::Regular { ref mut is_sparse, .. } = entity.kind {
            *is_sparse = true;
        }
        assert!(audit_entity(&receipt.destination_path, &entity, &options)
            .unwrap()
            .iter()
            .any(|diff| matches!(diff, DiffKind::SparseHoleMismatch { expected_has_holes: true, actual_has_holes: false })));
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_best_effort_never_hides_payload_corruption() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("payload");
        fs::write(&path, b"original").unwrap();
        let expected = FileEntity::from_filesystem(&path, None).unwrap();
        fs::write(&path, b"tampered").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
        assert!(verify_materialized_entity(&path, &expected, false).is_err());
    }

    #[crate::ctb_test]
    fn test_is_timestamp_acceptable_best_effort() {
        // Tolerates up to 2 seconds drift by default in best effort mode
        assert!(verifier::is_timestamp_acceptable_best_effort(
            1_000, 0, 1_002, 0, 1_000
        ));
        assert!(verifier::is_timestamp_acceptable_best_effort(
            1_000, 100, 1_000, 150, 100
        ));
        assert!(!verifier::is_timestamp_acceptable_best_effort(
            1_000, 0, 1_003, 0, 1_000
        ));

        // Coarser resolution volumes (e.g. 3s) allow larger tolerance
        assert!(verifier::is_timestamp_acceptable_best_effort(
            1_000, 0, 1_003, 0, 3_000_000_000
        ));
        assert!(!verifier::is_timestamp_acceptable_best_effort(
            1_000, 0, 1_004, 0, 3_000_000_000
        ));
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_strict_verification_fails_on_sub100ns_timestamp_difference() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("sub100ns_test");
        fs::write(&path, b"timestamp data").unwrap();

        let mut entity = FileEntity::from_filesystem(&path, None).unwrap();
        // Shift mtime by 15 nanoseconds (sub-100ns difference)
        if let Some(ref mut ts) = entity.metadata.timestamps {
            ts.mtime_nsec = ts.mtime_nsec.saturating_add(15);
        }

        // Strict verification must fail on any unpreserved nanosecond difference
        let strict_options = EntityAuditOptions {
            best_effort: false,
            ..Default::default()
        };
        let diffs = audit_entity(&path, &entity, &strict_options).unwrap();
        assert!(
            diffs
                .iter()
                .any(|d| matches!(d, DiffKind::MtimeMismatch { .. }))
        );
    }

    #[crate::ctb_test]
    fn test_noatime_tracked_and_verified() {
        let temp = tempfile::tempdir().unwrap();
        let src_path = temp.path().join("noatime_src.txt");
        let dest_path = temp.path().join("noatime_dest.txt");
        std::fs::write(&src_path, b"test noatime data").unwrap();

        #[cfg(target_os = "linux")]
        {
            use filetime::{FileTime, set_file_times};
            let atime = FileTime::from_unix_time(1_650_000_000, 0);
            let mtime = FileTime::from_unix_time(1_650_000_100, 0);
            set_file_times(&src_path, atime, mtime).unwrap();

            let payload = DiskPayloadSource::open(&src_path).unwrap();
            assert!(payload.opened_with_noatime());

            let mut entity = FileEntity::from_filesystem(&src_path, None).unwrap();
            assert!(entity.used_noatime());
            entity.identity.relative_path = Some(std::path::PathBuf::from("noatime_dest.txt"));
            if let Some(ref mut ts) = entity.metadata.timestamps {
                ts.birthtime_sec = None;
                ts.birthtime_nsec = None;
            }

            let mut payload = DiskPayloadSource::open(&src_path).unwrap();
            let dest_dir = sandboxable_dir::SandboxableDir::open(temp.path()).unwrap();
            let options = MaterializeOptions {
                dry_run: false,
                strict_lossless: true,
                symlink_policy: SymlinkValidationPolicy::PreserveVerbatim,
                path_policy: PathTraversalPolicy::StrictSandboxed,
                copy_specials: false,
                force_overwrite: true,
                apple_write_mode: crate::apple_double::AppleWriteMode::NativeOnly,
                apple_single_write_extension: crate::apple_double::AppleSingleExtension::WithoutExtension,
            };

            materializer::materialize_entity(&entity, Some(&mut payload), &dest_dir, &options).unwrap();

            // Destination atime must match and pass verification when checked
            verifier::verify_materialized_entity_ext(&dest_path, &entity, true, true, false).unwrap();

            // When destination atime is modified, verification with check_atime must fail
            let bad_atime = FileTime::from_unix_time(1_700_000_000, 0);
            set_file_times(&dest_path, bad_atime, mtime).unwrap();
            assert!(verifier::verify_materialized_entity_ext(&dest_path, &entity, true, true, false).is_err());
        }
    }
}
