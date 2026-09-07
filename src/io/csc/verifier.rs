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

//! Manifest verification engine auditing filesystem directory trees against
//! manifests recorded by `csc`. Reports content changes, missing items, untracked
//! files, and metadata differences.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::args::{CscVerifyArgs, VerifyOutputFormat};
use crate::journal::{
    ManifestDir, ManifestFile, ManifestSymlink, read_journal_snapshot, resolve_journal_path,
};
use crate::verify_cache::check_cache_flush_privileges;
use ctb_formats_checksum::Sha256Stream;
use ctb_io::file::entity::FileEntityKind;
use ctb_io::file::streams::read_and_hash_streams;
use ctb_io::file::sys_flags::query_file_flags;
use ctb_io::file::verifier::{evict_fd_cache, try_drop_system_caches};
use ctb_utilities::cli::ToolResult;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
use std::fs::File;
use std::io::Read;
use std::os::unix::ffi::OsStringExt;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// Nature of a stream/xattr discrepancy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum StreamDiffKind {
    /// Stream expected in manifest but missing on disk.
    MissingStream,
    /// Unexpected stream present on disk but absent in manifest.
    ExtraStream,
    /// Cryptographic digest mismatch on stream content.
    DigestMismatch {
        expected_hex: String,
        actual_hex: String,
    },
}

/// Discrepancy detected between manifest record and target directory on disk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type")]
pub enum DiffKind {
    /// Expected entry is missing entirely from disk.
    MissingOnDisk,
    /// Unexpected entry exists on disk but is absent from manifest.
    UntrackedOnDisk,
    /// File type on disk does not match manifest (e.g. file vs dir vs symlink).
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
    /// Hardlink target or inode grouping mismatch.
    HardlinkMismatch {
        expected_target: PathBuf,
        details: String,
    },
}

/// Collection of discrepancies detected for a specific relative file path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EntryDiff {
    pub relative_path: PathBuf,
    pub differences: Vec<DiffKind>,
}

/// Overall verification audit report for a directory tree against a manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VerificationReport {
    pub target_directory: PathBuf,
    pub manifest_path: PathBuf,
    pub total_manifest_entries: usize,
    pub total_disk_entries_scanned: usize,
    pub matched_entries: usize,
    pub changed_entries: Vec<EntryDiff>,
    pub missing_entries: Vec<PathBuf>,
    pub untracked_entries: Vec<PathBuf>,
}

impl VerificationReport {
    /// Returns true if no discrepancies of any kind were detected.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.changed_entries.is_empty()
            && self.missing_entries.is_empty()
            && self.untracked_entries.is_empty()
    }

    /// Formats the audit report into a human-readable text document.
    #[must_use]
    pub fn format_human_report(&self) -> String {
        use std::fmt::Write;
        let mut out = String::new();
        let _ = writeln!(out, "--- CSC Manifest Verification Report ---");
        let _ = writeln!(out, "Manifest:         {}", self.manifest_path.display());
        let _ = writeln!(out, "Target Directory: {}", self.target_directory.display());

        if self.is_clean() {
            let _ = writeln!(
                out,
                "Status:           OK - Directory matches manifest perfectly."
            );
            let _ = writeln!(
                out,
                "Verified Entries: {} (0 discrepancies)",
                self.matched_entries
            );
            return out;
        }

        let _ = writeln!(
            out,
            "Status:           FAILED - Discrepancies detected between directory and manifest.\n"
        );

        if !self.missing_entries.is_empty() {
            let _ = writeln!(out, "Missing Entries ({}):", self.missing_entries.len());
            for m in &self.missing_entries {
                let _ = writeln!(out, "  [MISSING] {}", m.display());
            }
            let _ = writeln!(out);
        }

        if !self.untracked_entries.is_empty() {
            let _ = writeln!(out, "Untracked Entries ({}):", self.untracked_entries.len());
            for u in &self.untracked_entries {
                let _ = writeln!(out, "  [UNTRACKED] {}", u.display());
            }
            let _ = writeln!(out);
        }

        if !self.changed_entries.is_empty() {
            let _ = writeln!(out, "Changed Entries ({}):", self.changed_entries.len());
            for entry in &self.changed_entries {
                let _ = writeln!(out, "  [CHANGED] {}", entry.relative_path.display());
                for diff in &entry.differences {
                    match diff {
                        DiffKind::TypeMismatch { expected, actual } => {
                            let _ = writeln!(
                                out,
                                "    - Type mismatch: expected {expected}, got {actual}"
                            );
                        }
                        DiffKind::SizeMismatch { expected, actual } => {
                            let _ = writeln!(
                                out,
                                "    - Size mismatch: expected {expected} bytes, got {actual} bytes"
                            );
                        }
                        DiffKind::ContentHashMismatch {
                            expected_hex,
                            actual_hex,
                        } => {
                            let _ = writeln!(
                                out,
                                "    - SHA-256 mismatch: expected {expected_hex}, got {actual_hex}"
                            );
                        }
                        DiffKind::SymlinkTargetMismatch { expected, actual } => {
                            let _ = writeln!(
                                out,
                                "    - Symlink target mismatch: expected {expected}, got {actual}"
                            );
                        }
                        DiffKind::ModeMismatch { expected, actual } => {
                            let _ = writeln!(
                                out,
                                "    - Permissions mismatch: expected {expected}, got {actual}"
                            );
                        }
                        DiffKind::UidMismatch { expected, actual } => {
                            let _ = writeln!(
                                out,
                                "    - Owner UID mismatch: expected {expected}, got {actual}"
                            );
                        }
                        DiffKind::GidMismatch { expected, actual } => {
                            let _ = writeln!(
                                out,
                                "    - Owner GID mismatch: expected {expected}, got {actual}"
                            );
                        }
                        DiffKind::MtimeMismatch {
                            expected_sec,
                            expected_nsec,
                            actual_sec,
                            actual_nsec,
                        } => {
                            let _ = writeln!(
                                out,
                                "    - Mtime mismatch: expected {expected_sec}.{expected_nsec:09}, got {actual_sec}.{actual_nsec:09}"
                            );
                        }
                        DiffKind::AtimeMismatch {
                            expected_sec,
                            expected_nsec,
                            actual_sec,
                            actual_nsec,
                        } => {
                            let _ = writeln!(
                                out,
                                "    - Atime mismatch: expected {expected_sec}.{expected_nsec:09}, got {actual_sec}.{actual_nsec:09}"
                            );
                        }
                        DiffKind::CtimeMismatch {
                            expected_sec,
                            expected_nsec,
                            actual_sec,
                            actual_nsec,
                        } => {
                            let _ = writeln!(
                                out,
                                "    - Ctime mismatch: expected {expected_sec}.{expected_nsec:09}, got {actual_sec}.{actual_nsec:09}"
                            );
                        }
                        DiffKind::BirthtimeMismatch {
                            expected_sec,
                            actual_sec,
                        } => {
                            let _ = writeln!(
                                out,
                                "    - Birthtime mismatch: expected {expected_sec:?}, got {actual_sec:?}"
                            );
                        }
                        DiffKind::FlagsMismatch { expected, actual } => {
                            let _ = writeln!(
                                out,
                                "    - Flags mismatch: expected {expected:?}, got {actual:?}"
                            );
                        }
                        DiffKind::StreamMismatch {
                            stream_name,
                            details,
                        } => match details {
                            StreamDiffKind::MissingStream => {
                                let _ = writeln!(out, "    - Stream missing: {stream_name}");
                            }
                            StreamDiffKind::ExtraStream => {
                                let _ = writeln!(out, "    - Unexpected extra stream: {stream_name}");
                            }
                            StreamDiffKind::DigestMismatch {
                                expected_hex,
                                actual_hex,
                            } => {
                                let _ = writeln!(
                                    out,
                                    "    - Stream '{stream_name}' hash mismatch: expected {expected_hex}, got {actual_hex}"
                                );
                            }
                        },
                        DiffKind::HardlinkMismatch {
                            expected_target,
                            details,
                        } => {
                            let _ = writeln!(
                                out,
                                "    - Hardlink mismatch: target {}, details: {details}",
                                expected_target.display()
                            );
                        }
                        DiffKind::MissingOnDisk => {
                            let _ = writeln!(out, "    - Missing on disk");
                        }
                        DiffKind::UntrackedOnDisk => {
                            let _ = writeln!(out, "    - Untracked on disk");
                        }
                    }
                }
            }
            let _ = writeln!(out);
        }

        let _ = writeln!(out, "Summary:");
        let _ = writeln!(out, "  Manifest entries:  {}", self.total_manifest_entries);
        let _ = writeln!(out, "  Clean matches:     {}", self.matched_entries);
        let _ = writeln!(out, "  Changed entries:   {}", self.changed_entries.len());
        let _ = writeln!(out, "  Missing entries:   {}", self.missing_entries.len());
        let _ = writeln!(
            out,
            "  Untracked entries: {}",
            self.untracked_entries.len()
        );

        out
    }

    /// Formats the audit report as JSON.
    pub fn format_json_report(&self) -> Result<String> {
        serde_json::to_string_pretty(self).context("Failed to serialize verification report to JSON")
    }
}

/// Helper converting a 32-byte digest array into a lowercase hex string.
fn hex_encode(bytes: &[u8; 32]) -> String {
    use std::fmt::Write;
    let mut s = String::with_capacity(64);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

/// Performs verification of target directory against manifest according to options.
pub fn verify_directory_against_manifest(args: &CscVerifyArgs) -> Result<VerificationReport> {
    let journal_path = resolve_journal_path(&args.manifest)?;
    let snapshot = read_journal_snapshot(&journal_path)?;

    let target_dir = if let Some(ref custom_dir) = args.dir {
        custom_dir.clone()
    } else {
        snapshot.destination.clone()
    };

    anyhow::ensure!(
        target_dir.exists(),
        "Target directory to verify does not exist: {}",
        target_dir.display()
    );
    anyhow::ensure!(
        target_dir.is_dir(),
        "Target path to verify is not a directory: {}",
        target_dir.display()
    );

    // Drop OS page cache and attempt kernel drop_caches if enabled
    if args.should_drop_caches() {
        check_cache_flush_privileges();
        try_drop_system_caches();
    }

    let mut changed_entries = Vec::new();
    let mut missing_entries = Vec::new();
    let mut matched_entries = 0_usize;
    let mut verified_paths = HashSet::new();

    let total_manifest_entries = snapshot
        .committed_files
        .len()
        .saturating_add(snapshot.committed_dirs.len())
        .saturating_add(snapshot.committed_symlinks.len())
        .saturating_add(snapshot.committed_hardlinks.len());

    // 1. Verify Regular Files
    for (rel_path, mf) in &snapshot.committed_files {
        verified_paths.insert(normalize_rel_path(rel_path));
        let full_path = target_dir.join(rel_path);

        match std::fs::symlink_metadata(&full_path) {
            Err(_) => {
                missing_entries.push(rel_path.clone());
            }
            Ok(meta) => {
                let diffs = audit_file(&full_path, mf, &meta, args)?;
                if diffs.is_empty() {
                    matched_entries = matched_entries.saturating_add(1);
                } else {
                    changed_entries.push(EntryDiff {
                        relative_path: rel_path.clone(),
                        differences: diffs,
                    });
                }
            }
        }
    }

    // 2. Verify Directories
    for (rel_path, md) in &snapshot.committed_dirs {
        if rel_path.as_os_str().is_empty() || rel_path == Path::new(".") {
            continue;
        }
        verified_paths.insert(normalize_rel_path(rel_path));
        let full_path = target_dir.join(rel_path);

        match std::fs::symlink_metadata(&full_path) {
            Err(_) => {
                missing_entries.push(rel_path.clone());
            }
            Ok(meta) => {
                let diffs = audit_directory(&full_path, md, &meta, args)?;
                if diffs.is_empty() {
                    matched_entries = matched_entries.saturating_add(1);
                } else {
                    changed_entries.push(EntryDiff {
                        relative_path: rel_path.clone(),
                        differences: diffs,
                    });
                }
            }
        }
    }

    // 3. Verify Symlinks
    for (rel_path, ms) in &snapshot.committed_symlinks {
        verified_paths.insert(normalize_rel_path(rel_path));
        let full_path = target_dir.join(rel_path);

        match std::fs::symlink_metadata(&full_path) {
            Err(_) => {
                missing_entries.push(rel_path.clone());
            }
            Ok(meta) => {
                let diffs = audit_symlink(&full_path, ms, &meta, args)?;
                if diffs.is_empty() {
                    matched_entries = matched_entries.saturating_add(1);
                } else {
                    changed_entries.push(EntryDiff {
                        relative_path: rel_path.clone(),
                        differences: diffs,
                    });
                }
            }
        }
    }

    // 4. Verify Hardlinks
    for (src_rel, tgt_rel) in &snapshot.committed_hardlinks {
        verified_paths.insert(normalize_rel_path(src_rel));
        let src_full = target_dir.join(src_rel);
        let tgt_full = target_dir.join(tgt_rel);

        match (
            std::fs::symlink_metadata(&src_full),
            std::fs::symlink_metadata(&tgt_full),
        ) {
            (Ok(m1), Ok(m2)) => {
                if m1.ino() != m2.ino() || m1.dev() != m2.dev() {
                    changed_entries.push(EntryDiff {
                        relative_path: src_rel.clone(),
                        differences: vec![DiffKind::HardlinkMismatch {
                            expected_target: tgt_rel.clone(),
                            details: format!(
                                "Inodes differ: ({}:{}) vs ({}:{})",
                                m1.dev(),
                                m1.ino(),
                                m2.dev(),
                                m2.ino()
                            ),
                        }],
                    });
                } else {
                    matched_entries = matched_entries.saturating_add(1);
                }
            }
            (Err(_), _) => {
                missing_entries.push(src_rel.clone());
            }
            (_, Err(_)) => {
                changed_entries.push(EntryDiff {
                    relative_path: src_rel.clone(),
                    differences: vec![DiffKind::HardlinkMismatch {
                        expected_target: tgt_rel.clone(),
                        details: format!("Target hardlink file missing: {}", tgt_full.display()),
                    }],
                });
            }
        }
    }

    // 5. Discover untracked entries on disk unless ignored
    let mut untracked_entries = Vec::new();
    let mut total_disk_scanned = 0_usize;

    if !args.should_ignore_untracked() {
        let disk_entries = scan_disk_entries(&target_dir)?;
        total_disk_scanned = disk_entries.len();

        for disk_rel in disk_entries {
            let norm = normalize_rel_path(&disk_rel);
            if !verified_paths.contains(&norm) {
                untracked_entries.push(disk_rel);
            }
        }
    }

    // Sort entries for deterministic output
    changed_entries.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    missing_entries.sort();
    untracked_entries.sort();

    Ok(VerificationReport {
        target_directory: target_dir,
        manifest_path: journal_path,
        total_manifest_entries,
        total_disk_entries_scanned: total_disk_scanned,
        matched_entries,
        changed_entries,
        missing_entries,
        untracked_entries,
    })
}

fn normalize_rel_path(p: &Path) -> PathBuf {
    p.strip_prefix("./").unwrap_or(p).to_path_buf()
}

fn audit_file(
    full_path: &Path,
    mf: &ManifestFile,
    meta: &std::fs::Metadata,
    args: &CscVerifyArgs,
) -> Result<Vec<DiffKind>> {
    let mut diffs = Vec::new();

    if !meta.is_file() {
        diffs.push(DiffKind::TypeMismatch {
            expected: "regular file".into(),
            actual: if meta.is_dir() {
                "directory".into()
            } else if meta.is_symlink() {
                "symlink".into()
            } else {
                "special node".into()
            },
        });
        return Ok(diffs);
    }

    // Permissions
    if !args.should_ignore_perms() {
        let expected_mode = mf.mode & 0o7777;
        let actual_mode = meta.mode() & 0o7777;
        if expected_mode != actual_mode {
            diffs.push(DiffKind::ModeMismatch {
                expected: format!("{expected_mode:#05o}"),
                actual: format!("{actual_mode:#05o}"),
            });
        }
    }

    // Ownership
    if !args.should_ignore_owner() {
        if meta.uid() != mf.uid {
            diffs.push(DiffKind::UidMismatch {
                expected: mf.uid,
                actual: meta.uid(),
            });
        }
        if meta.gid() != mf.gid {
            diffs.push(DiffKind::GidMismatch {
                expected: mf.gid,
                actual: meta.gid(),
            });
        }
    }

    // Modification time
    if !args.should_ignore_mtime() {
        let actual_sec = meta.mtime();
        let actual_nsec = u32::try_from(meta.mtime_nsec()).unwrap_or(0);
        if actual_sec != mf.mtime_sec || actual_nsec != mf.mtime_nsec {
            diffs.push(DiffKind::MtimeMismatch {
                expected_sec: mf.mtime_sec,
                expected_nsec: mf.mtime_nsec,
                actual_sec,
                actual_nsec,
            });
        }
    }

    // Access time
    if !args.should_ignore_atime() {
        let actual_sec = meta.atime();
        let actual_nsec = u32::try_from(meta.atime_nsec()).unwrap_or(0);
        if actual_sec != mf.atime_sec || actual_nsec != mf.atime_nsec {
            diffs.push(DiffKind::AtimeMismatch {
                expected_sec: mf.atime_sec,
                expected_nsec: mf.atime_nsec,
                actual_sec,
                actual_nsec,
            });
        }
    }

    // Metadata change time
    if !args.should_ignore_ctime() {
        let actual_sec = meta.ctime();
        let actual_nsec = u32::try_from(meta.ctime_nsec()).unwrap_or(0);
        if actual_sec != mf.ctime_sec || actual_nsec != mf.ctime_nsec {
            diffs.push(DiffKind::CtimeMismatch {
                expected_sec: mf.ctime_sec,
                expected_nsec: mf.ctime_nsec,
                actual_sec,
                actual_nsec,
            });
        }
    }

    // Flags
    if !args.should_ignore_flags() {
        if let Ok((actual_flags, _)) = query_file_flags(full_path, false) {
            let mut exp_names: Vec<String> = mf.flags.iter().map(|f| f.name().to_string()).collect();
            let mut act_names: Vec<String> = actual_flags.iter().map(|f| f.name().to_string()).collect();
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

    // Size
    let actual_size = meta.len();
    if actual_size != mf.size {
        diffs.push(DiffKind::SizeMismatch {
            expected: mf.size,
            actual: actual_size,
        });
    }

    // Payload cryptographic checksum
    let mut file = File::open(full_path).with_context(|| {
        format!("Failed to open file for verification: {}", full_path.display())
    })?;
    if args.should_drop_caches() {
        evict_fd_cache(&file);
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
            .context("Verification buffer slice out of bounds")?;
        hasher.update(slice);
    }
    let actual_sha256 = hasher.finalize();

    if actual_sha256 != mf.sha256 {
        diffs.push(DiffKind::ContentHashMismatch {
            expected_hex: hex_encode(&mf.sha256),
            actual_hex: hex_encode(&actual_sha256),
        });
    }

    // Extended attributes and streams
    if !args.should_ignore_xattrs() {
        let on_disk_streams = read_and_hash_streams(full_path)?;
        let mut expected_map: HashMap<OsString, [u8; 32]> = HashMap::new();
        for (sname, shash) in &mf.streams {
            expected_map.insert(sname.clone(), *shash);
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

    Ok(diffs)
}

fn audit_directory(
    full_path: &Path,
    md: &ManifestDir,
    meta: &std::fs::Metadata,
    args: &CscVerifyArgs,
) -> Result<Vec<DiffKind>> {
    let mut diffs = Vec::new();

    if !meta.is_dir() {
        diffs.push(DiffKind::TypeMismatch {
            expected: "directory".into(),
            actual: if meta.is_file() {
                "regular file".into()
            } else if meta.is_symlink() {
                "symlink".into()
            } else {
                "special node".into()
            },
        });
        return Ok(diffs);
    }

    if !args.should_ignore_perms() {
        let expected_mode = md.mode & 0o7777;
        let actual_mode = meta.mode() & 0o7777;
        if expected_mode != actual_mode {
            diffs.push(DiffKind::ModeMismatch {
                expected: format!("{expected_mode:#05o}"),
                actual: format!("{actual_mode:#05o}"),
            });
        }
    }

    if !args.should_ignore_owner() {
        if meta.uid() != md.uid {
            diffs.push(DiffKind::UidMismatch {
                expected: md.uid,
                actual: meta.uid(),
            });
        }
        if meta.gid() != md.gid {
            diffs.push(DiffKind::GidMismatch {
                expected: md.gid,
                actual: meta.gid(),
            });
        }
    }

    if !args.should_ignore_mtime() {
        let actual_sec = meta.mtime();
        let actual_nsec = u32::try_from(meta.mtime_nsec()).unwrap_or(0);
        if actual_sec != md.mtime_sec || actual_nsec != md.mtime_nsec {
            diffs.push(DiffKind::MtimeMismatch {
                expected_sec: md.mtime_sec,
                expected_nsec: md.mtime_nsec,
                actual_sec,
                actual_nsec,
            });
        }
    }

    if !args.should_ignore_atime() {
        let actual_sec = meta.atime();
        let actual_nsec = u32::try_from(meta.atime_nsec()).unwrap_or(0);
        if actual_sec != md.atime_sec || actual_nsec != md.atime_nsec {
            diffs.push(DiffKind::AtimeMismatch {
                expected_sec: md.atime_sec,
                expected_nsec: md.atime_nsec,
                actual_sec,
                actual_nsec,
            });
        }
    }

    if !args.should_ignore_ctime() {
        let actual_sec = meta.ctime();
        let actual_nsec = u32::try_from(meta.ctime_nsec()).unwrap_or(0);
        if actual_sec != md.ctime_sec || actual_nsec != md.ctime_nsec {
            diffs.push(DiffKind::CtimeMismatch {
                expected_sec: md.ctime_sec,
                expected_nsec: md.ctime_nsec,
                actual_sec,
                actual_nsec,
            });
        }
    }

    if !args.should_ignore_flags() {
        if let Ok((actual_flags, _)) = query_file_flags(full_path, false) {
            let mut exp_names: Vec<String> = md.flags.iter().map(|f| f.name().to_string()).collect();
            let mut act_names: Vec<String> = actual_flags.iter().map(|f| f.name().to_string()).collect();
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

    Ok(diffs)
}

fn audit_symlink(
    full_path: &Path,
    ms: &ManifestSymlink,
    meta: &std::fs::Metadata,
    args: &CscVerifyArgs,
) -> Result<Vec<DiffKind>> {
    let mut diffs = Vec::new();

    if !meta.is_symlink() {
        diffs.push(DiffKind::TypeMismatch {
            expected: "symlink".into(),
            actual: if meta.is_dir() {
                "directory".into()
            } else if meta.is_file() {
                "regular file".into()
            } else {
                "special node".into()
            },
        });
        return Ok(diffs);
    }

    let actual_target = std::fs::read_link(full_path)?;
    let actual_bytes = actual_target.as_os_str().as_encoded_bytes();
    if actual_bytes != ms.target.as_slice() {
        diffs.push(DiffKind::SymlinkTargetMismatch {
            expected: String::from_utf8_lossy(&ms.target).to_string(),
            actual: String::from_utf8_lossy(actual_bytes).to_string(),
        });
    }

    if !args.should_ignore_owner() {
        if meta.uid() != ms.uid {
            diffs.push(DiffKind::UidMismatch {
                expected: ms.uid,
                actual: meta.uid(),
            });
        }
        if meta.gid() != ms.gid {
            diffs.push(DiffKind::GidMismatch {
                expected: ms.gid,
                actual: meta.gid(),
            });
        }
    }

    if !args.should_ignore_mtime() {
        let actual_sec = meta.mtime();
        let actual_nsec = u32::try_from(meta.mtime_nsec()).unwrap_or(0);
        if actual_sec != ms.mtime_sec || actual_nsec != ms.mtime_nsec {
            diffs.push(DiffKind::MtimeMismatch {
                expected_sec: ms.mtime_sec,
                expected_nsec: ms.mtime_nsec,
                actual_sec,
                actual_nsec,
            });
        }
    }

    if !args.should_ignore_atime() {
        let actual_sec = meta.atime();
        let actual_nsec = u32::try_from(meta.atime_nsec()).unwrap_or(0);
        if actual_sec != ms.atime_sec || actual_nsec != ms.atime_nsec {
            diffs.push(DiffKind::AtimeMismatch {
                expected_sec: ms.atime_sec,
                expected_nsec: ms.atime_nsec,
                actual_sec,
                actual_nsec,
            });
        }
    }

    Ok(diffs)
}

fn scan_disk_entries(root: &Path) -> Result<HashSet<PathBuf>> {
    let mut result = HashSet::new();
    let mut dir_queue = vec![root.to_path_buf()];

    while let Some(current_dir) = dir_queue.pop() {
        let entries = match std::fs::read_dir(&current_dir) {
            Ok(e) => e,
            Err(err) => {
                log_fmt!("Failed to read directory {}: {err}", current_dir.display());
                continue;
            }
        };

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if let Ok(rel) = path.strip_prefix(root) {
                if !rel.as_os_str().is_empty() {
                    result.insert(rel.to_path_buf());
                }
            }

            if let Ok(sym_meta) = std::fs::symlink_metadata(&path) {
                if sym_meta.is_dir() {
                    dir_queue.push(path);
                }
            }
        }
    }

    Ok(result)
}

/// Main orchestration entry point for the `csc-verify` CLI command.
pub fn run_csc_verify(args: CscVerifyArgs) -> Result<ToolResult> {
    let report = verify_directory_against_manifest(&args)?;

    let output_str = match args.format {
        VerifyOutputFormat::Text => report.format_human_report(),
        VerifyOutputFormat::Json => report.format_json_report()?,
    };

    let exit_code = if report.is_clean() { 0 } else { 1 };

    Ok(ToolResult::Immediate {
        stdout: output_str.into_bytes(),
        stderr: Vec::new(),
        exit_code,
    })
}
