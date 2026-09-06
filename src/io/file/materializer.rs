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

//! Atomic file materialization, metadata sequencing, and fidelity verification.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::file::entity::{FileEntity, FileEntityKind};
use crate::file::metadata::FileMetadata;
use crate::file::path_policy::{
    PathTraversalPolicy, SymlinkValidationPolicy, ensure_sandboxed_dir_all,
    resolve_and_validate_path, validate_symlink_target,
};
use crate::file::payload::{Extent, PayloadSource};
use crate::file::streams::write_streams;
use crate::file::sys_flags::apply_file_flags;
use ctb_formats_checksum::Sha256Stream;
use filetime::{FileTime, set_file_times, set_symlink_file_times};
use nix::fcntl::{AT_FDCWD, AtFlags};
use nix::sys::stat::{Mode, SFlag, mknod};
use nix::unistd::{Gid, Uid, fchownat, mkfifo};
use std::ffi::OsStr;
use std::fs::{File, OpenOptions, Permissions};
use std::io::{Seek, SeekFrom, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

/// Execution options for file materialization.
#[derive(Debug, Clone)]
pub struct MaterializeOptions {
    /// If true, calculate operations without modifying the filesystem.
    pub dry_run: bool,
    /// If true, enforce strict zero-data-loss for metadata and streams.
    pub strict_lossless: bool,
    /// Policy for validating symlink targets.
    pub symlink_policy: SymlinkValidationPolicy,
    /// Policy for validating relative entry paths.
    pub path_policy: PathTraversalPolicy,
    /// Permit creating block device special nodes.
    pub copy_block_devices: bool,
}

impl Default for MaterializeOptions {
    fn default() -> Self {
        Self {
            dry_run: false,
            strict_lossless: true,
            symlink_policy: SymlinkValidationPolicy::PreserveVerbatim,
            path_policy: PathTraversalPolicy::StrictSandboxed,
            copy_block_devices: false,
        }
    }
}

/// A receipt summarizing the outcome of materializing a file entity.
#[derive(Debug, Clone)]
pub struct MaterializeReceipt {
    /// Final materialized path on the target filesystem.
    pub destination_path: PathBuf,
    /// Number of bytes written to the data fork.
    pub bytes_written: u64,
    /// Computed cryptographic SHA-256 digest of the data fork.
    pub sha256: Option<[u8; 32]>,
}

/// Applies ownership, permissions, and timestamps from `meta` to `dest`.
pub fn apply_entity_metadata(
    dest: &Path,
    meta: &FileMetadata,
    is_symlink: bool,
    strict_lossless: bool,
) -> Result<()> {
    let mode = meta.mode;
    let uid = meta.uid;
    let gid = meta.gid;
    let atime = FileTime::from_unix_time(
        meta.timestamps.atime_sec,
        meta.timestamps.atime_nsec,
    );
    let mtime = FileTime::from_unix_time(
        meta.timestamps.mtime_sec,
        meta.timestamps.mtime_nsec,
    );

    // 1. Ownership: check if dest already has the desired UID/GID
    let dest_meta = if is_symlink {
        std::fs::symlink_metadata(dest)
    } else {
        std::fs::metadata(dest)
    };

    let needs_chown = match dest_meta {
        Ok(ref dm) => dm.uid() != uid || dm.gid() != gid,
        Err(_) => true,
    };

    if needs_chown {
        let uid_obj = Some(Uid::from_raw(uid));
        let gid_obj = Some(Gid::from_raw(gid));
        let chown_res = if is_symlink {
            fchownat(
                AT_FDCWD,
                dest,
                uid_obj,
                gid_obj,
                AtFlags::AT_SYMLINK_NOFOLLOW,
            )
        } else {
            nix::unistd::chown(dest, uid_obj, gid_obj)
        };
        if let Err(err) = chown_res {
            if strict_lossless {
                anyhow::bail!(
                    "Failed to preserve ownership (uid: {uid}, gid: {gid}) for {}: {err}",
                    dest.display()
                );
            }
        }
    }

    // 2. Permissions (symlink permissions are fixed on Linux)
    if !is_symlink {
        let perms = Permissions::from_mode(mode);
        if let Err(e) = std::fs::set_permissions(dest, perms) {
            if strict_lossless {
                return Err(e).with_context(|| {
                    format!("Failed to set permissions on {}", dest.display())
                });
            }
        }
    }

    // 3. Timestamps
    let time_res = if is_symlink {
        set_symlink_file_times(dest, atime, mtime)
    } else {
        set_file_times(dest, atime, mtime)
    };
    if let Err(e) = time_res {
        if strict_lossless {
            return Err(e).with_context(|| {
                format!("Failed to set file timestamps on {}", dest.display())
            });
        }
    }

    Ok(())
}

/// Verifies that the destination filesystem did not alter, normalize, or
/// truncate the filename bytes.
pub fn verify_filename_exact_bytes(
    parent_dir: &Path,
    expected_filename_bytes: &[u8],
) -> Result<()> {
    if expected_filename_bytes.is_empty() {
        return Ok(());
    }

    let mut matched = false;
    for entry in std::fs::read_dir(parent_dir)
        .with_context(|| format!("Failed to read directory: {}", parent_dir.display()))?
    {
        let entry = entry.with_context(|| {
            format!("Error reading entry in directory: {}", parent_dir.display())
        })?;
        let entry_name = entry.file_name();
        let entry_bytes = entry_name.as_bytes();
        if entry_bytes == expected_filename_bytes {
            matched = true;
            break;
        }
    }

    anyhow::ensure!(
        matched,
        "Target filesystem altered, normalized, or discarded filename bytes for {:?}",
        String::from_utf8_lossy(expected_filename_bytes)
    );
    Ok(())
}

fn remove_if_exists(path: &Path) {
    if path.exists() || path.is_symlink() {
        let _ = std::fs::remove_file(path);
    }
}

/// Materializes a complete `FileEntity` onto the filesystem at `dest_root`.
pub fn materialize_entity(
    entity: &FileEntity,
    payload: Option<&mut dyn PayloadSource>,
    dest_root: &Path,
    options: &MaterializeOptions,
) -> Result<MaterializeReceipt> {
    let dest_path = resolve_and_validate_path(
        dest_root,
        &entity.identity.relative_path,
        options.path_policy,
    )?;

    if options.dry_run {
        let size = match &entity.kind {
            FileEntityKind::Regular { size, .. } => *size,
            _ => 0,
        };
        return Ok(MaterializeReceipt {
            destination_path: dest_path,
            bytes_written: size,
            sha256: match &entity.kind {
                FileEntityKind::Regular { sha256, .. } => Some(*sha256),
                _ => None,
            },
        });
    }

    let parent_dir = dest_path
        .parent()
        .context("Destination path has no parent directory")?;
    if options.path_policy == PathTraversalPolicy::StrictSandboxed {
        ensure_sandboxed_dir_all(dest_root, parent_dir)?;
    } else if !parent_dir.exists() {
        std::fs::create_dir_all(parent_dir).with_context(|| {
            format!("Failed to create directory tree: {}", parent_dir.display())
        })?;
    }

    match &entity.kind {
        FileEntityKind::Symlink { target } => {
            validate_symlink_target(
                dest_root,
                &dest_path,
                target,
                options.symlink_policy,
            )?;
            remove_if_exists(&dest_path);
            let target_os = OsStr::from_bytes(target);
            std::os::unix::fs::symlink(target_os, &dest_path).with_context(|| {
                format!("Failed to create symlink: {}", dest_path.display())
            })?;
            apply_entity_metadata(&dest_path, &entity.metadata, true, options.strict_lossless)?;

            Ok(MaterializeReceipt {
                destination_path: dest_path,
                bytes_written: 0,
                sha256: None,
            })
        }
        FileEntityKind::Hardlink {
            target_relative_path,
        } => {
            let target_path = dest_root.join(target_relative_path);
            remove_if_exists(&dest_path);
            std::fs::hard_link(&target_path, &dest_path).with_context(|| {
                format!(
                    "Failed to create hardlink from {} to {}",
                    target_path.display(),
                    dest_path.display()
                )
            })?;
            Ok(MaterializeReceipt {
                destination_path: dest_path,
                bytes_written: 0,
                sha256: None,
            })
        }
        FileEntityKind::Directory | FileEntityKind::Bundle { .. } => {
            if options.path_policy == PathTraversalPolicy::StrictSandboxed {
                ensure_sandboxed_dir_all(dest_root, &dest_path)?;
            } else if !dest_path.exists() {
                std::fs::create_dir_all(&dest_path).with_context(|| {
                    format!("Failed to create directory: {}", dest_path.display())
                })?;
            }
            write_streams(&dest_path, &entity.streams, options.strict_lossless)?;
            apply_entity_metadata(
                &dest_path,
                &entity.metadata,
                false,
                options.strict_lossless,
            )?;
            Ok(MaterializeReceipt {
                destination_path: dest_path,
                bytes_written: 0,
                sha256: None,
            })
        }
        FileEntityKind::Fifo => {
            remove_if_exists(&dest_path);
            mkfifo(
                &dest_path,
                Mode::from_bits_truncate(entity.metadata.mode),
            )
            .with_context(|| format!("Failed to create FIFO: {}", dest_path.display()))?;
            apply_entity_metadata(
                &dest_path,
                &entity.metadata,
                false,
                options.strict_lossless,
            )?;
            Ok(MaterializeReceipt {
                destination_path: dest_path,
                bytes_written: 0,
                sha256: None,
            })
        }
        FileEntityKind::CharDevice { rdev } => {
            remove_if_exists(&dest_path);
            mknod(
                &dest_path,
                SFlag::S_IFCHR,
                Mode::from_bits_truncate(entity.metadata.mode),
                *rdev,
            )
            .with_context(|| {
                format!(
                    "Failed to create character device node: {}",
                    dest_path.display()
                )
            })?;
            apply_entity_metadata(
                &dest_path,
                &entity.metadata,
                false,
                options.strict_lossless,
            )?;
            Ok(MaterializeReceipt {
                destination_path: dest_path,
                bytes_written: 0,
                sha256: None,
            })
        }
        FileEntityKind::BlockDevice { rdev } => {
            anyhow::ensure!(
                options.copy_block_devices,
                "Block device node creation rejected: {}. Enable copy_block_devices to permit.",
                dest_path.display()
            );
            remove_if_exists(&dest_path);
            mknod(
                &dest_path,
                SFlag::S_IFBLK,
                Mode::from_bits_truncate(entity.metadata.mode),
                *rdev,
            )
            .with_context(|| {
                format!(
                    "Failed to create block device node: {}",
                    dest_path.display()
                )
            })?;
            apply_entity_metadata(
                &dest_path,
                &entity.metadata,
                false,
                options.strict_lossless,
            )?;
            Ok(MaterializeReceipt {
                destination_path: dest_path,
                bytes_written: 0,
                sha256: None,
            })
        }
        FileEntityKind::Socket => {
            anyhow::bail!(
                "Cannot materialize UNIX domain socket {}. Sockets cannot be cloned across directories.",
                dest_path.display()
            );
        }
        FileEntityKind::Door => {
            anyhow::bail!(
                "Cannot materialize Door IPC descriptor {}. Doors cannot be cloned across filesystems.",
                dest_path.display()
            );
        }
        FileEntityKind::Regular {
            size,
            sha256: expected_sha256,
            is_sparse,
            extents,
        } => {
            let Some(source) = payload else {
                anyhow::bail!(
                    "PayloadSource required to materialize regular file: {}",
                    dest_path.display()
                );
            };

            let pid = std::process::id();
            let nanos = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
                Ok(dur) => dur.subsec_nanos(),
                Err(_) => 0,
            };
            static ATOMIC_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
            let seq = ATOMIC_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let temp_name = format!(".csc-tmp.{pid}.{nanos}.{seq}");
            let temp_path = parent_dir.join(temp_name);

            let mut temp_file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temp_path)
                .with_context(|| {
                    format!("Failed to create atomic temp file: {}", temp_path.display())
                })?;

            let mut hasher = Sha256Stream::new();
            let initial_size = *size;

            if *is_sparse {
                for extent in extents {
                    match extent {
                        Extent::Data { offset, length } => {
                            source.seek(SeekFrom::Start(*offset))?;
                            temp_file.seek(SeekFrom::Start(*offset))?;

                            let mut remaining = *length;
                            let mut buf = vec![0_u8; 64 * 1024];
                            while remaining > 0 {
                                let to_read = usize::try_from(remaining.min(64 * 1024))
                                    .context("Failed to convert buffer slice length to usize")?;
                                let buf_slice = buf
                                    .get_mut(..to_read)
                                    .context("Buffer slice index out of bounds for read")?;
                                let n = source.read(buf_slice)?;
                                if n == 0 {
                                    break;
                                }
                                let write_slice = buf
                                    .get(..n)
                                    .context("Buffer slice index out of bounds for write")?;
                                temp_file.write_all(write_slice)?;
                                hasher.update(write_slice);
                                let n_u64 = u64::try_from(n)
                                    .context("Failed to convert read bytes count to u64")?;
                                remaining = remaining.saturating_sub(n_u64);
                            }
                        }
                        Extent::Hole { length, .. } => {
                            let zero_buf = [0_u8; 8 * 1024];
                            let mut remaining = *length;
                            while remaining > 0 {
                                let chunk = usize::try_from(remaining.min(8 * 1024))
                                    .context("Failed to convert hole chunk size to usize")?;
                                let zero_slice = zero_buf
                                    .get(..chunk)
                                    .context("Zero buffer slice index out of bounds")?;
                                hasher.update(zero_slice);
                                let chunk_u64 = u64::try_from(chunk)
                                    .context("Failed to convert hole chunk size to u64")?;
                                remaining = remaining.saturating_sub(chunk_u64);
                            }
                        }
                    }
                }
                temp_file.set_len(initial_size)?;
            } else {
                source.seek(SeekFrom::Start(0))?;
                let mut buf = vec![0_u8; 64 * 1024];
                loop {
                    let n = source.read(&mut buf)?;
                    if n == 0 {
                        break;
                    }
                    let slice = buf
                        .get(..n)
                        .context("Buffer slice index out of bounds for write")?;
                    temp_file.write_all(slice)?;
                    hasher.update(slice);
                }
            }

            let computed_sha256 = hasher.finalize();
            if computed_sha256 != *expected_sha256 {
                let _ = std::fs::remove_file(&temp_path);
                anyhow::bail!(
                    "Payload SHA-256 verification failed during write for {}: expected {:02x?}, got {:02x?}",
                    dest_path.display(),
                    expected_sha256,
                    computed_sha256
                );
            }

            // Write attached streams (xattrs, resource forks)
            write_streams(&temp_path, &entity.streams, options.strict_lossless)?;

            // Apply ownership, permissions, and timestamps to temp file
            apply_entity_metadata(
                &temp_path,
                &entity.metadata,
                false,
                options.strict_lossless,
            )?;

            // Sync temp file and parent directory
            temp_file.sync_data()?;
            drop(temp_file);

            let parent_fd = File::open(parent_dir)?;
            parent_fd.sync_data()?;

            // Atomic rename
            std::fs::rename(&temp_path, &dest_path).with_context(|| {
                format!(
                    "Failed to rename temp file to destination: {}",
                    dest_path.display()
                )
            })?;
            parent_fd.sync_data()?;

            // Verify raw filename bytes on destination filesystem
            verify_filename_exact_bytes(parent_dir, &entity.identity.raw_filename)?;

            // Deferred immutability: apply flags as the very last step!
            if !entity.metadata.flags.is_empty()
                || entity.metadata.platform_raw_flags.is_some()
            {
                apply_file_flags(
                    &dest_path,
                    &entity.metadata.flags,
                    entity.metadata.platform_raw_flags.as_ref(),
                    options.strict_lossless,
                )?;
            }

            Ok(MaterializeReceipt {
                destination_path: dest_path,
                bytes_written: initial_size,
                sha256: Some(computed_sha256),
            })
        }
    }
}
