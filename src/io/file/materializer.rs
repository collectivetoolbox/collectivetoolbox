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
use crate::file::identity::resolve_relative_path_for_os;
use crate::file::metadata::FileMetadata;
use crate::file::path_policy::{
    PathTraversalPolicy, SymlinkValidationPolicy, resolve_and_validate_path,
};
use crate::file::payload::{Extent, PayloadSource};
use crate::file::sandboxable_dir::{DirHandleRef, SandboxableDir};
use crate::file::streams::{read_and_hash_streams, remove_stream, write_streams};
use crate::file::sys_flags::{apply_file_flags, query_file_flags};
use crate::file::verifier::{EntityAuditOptions, audit_entity_detailed};
use ctb_formats_checksum::Sha256Stream;
use filetime::{FileTime, set_file_times, set_symlink_file_times};
#[cfg(unix)]
use nix::fcntl::{AT_FDCWD, AtFlags as NixAtFlags};
#[cfg(unix)]
use nix::unistd::{Gid, Uid, fchownat};
#[cfg(unix)]
use rustix::fd::AsFd;
#[cfg(unix)]
use rustix::fs::AtFlags;
use std::fs::Permissions;
use std::io::{Read, Seek, SeekFrom, Write};
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

/// Execution options for file materialization.
#[derive(Debug, Clone)]
#[expect(clippy::struct_excessive_bools, reason = "Configuration flags for materialization fidelity")]
pub struct MaterializeOptions {
    /// If true, calculate operations without modifying the filesystem.
    pub dry_run: bool,
    /// If true, enforce strict zero-data-loss for metadata and streams.
    pub strict_lossless: bool,
    /// Policy for validating symlink targets.
    pub symlink_policy: SymlinkValidationPolicy,
    /// Policy for validating relative entry paths.
    pub path_policy: PathTraversalPolicy,
    /// Permit creating special nodes (FIFOs, character and block devices).
    pub copy_specials: bool,
    /// If true, always overwrite the destination by recreating the payload and metadata atomically,
    /// skipping any in-place reuse even if payload checksums match.
    pub force_overwrite: bool,
}

impl Default for MaterializeOptions {
    fn default() -> Self {
        Self {
            dry_run: false,
            strict_lossless: true,
            symlink_policy: SymlinkValidationPolicy::PreserveVerbatim,
            path_policy: PathTraversalPolicy::StrictSandboxed,
            copy_specials: false,
            force_overwrite: false,
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
    /// True if an existing identical destination was preserved and metadata updated in-place
    /// without re-writing payload data.
    pub skipped_identical: bool,
}

/// Applies ownership, permissions, and timestamps from `meta` to `dest`.
pub fn apply_entity_metadata(
    dest: &Path,
    target_display_path: Option<&Path>,
    meta: &FileMetadata,
    is_symlink: bool,
    apply_flags: bool,
    strict_lossless: bool,
) -> Result<()> {
    #[cfg(not(any(unix, windows)))]
    anyhow::ensure!(!strict_lossless, "Lossless ownership and permission preservation is not implemented on this platform");
    // Reason for fallback: error reporting defaults to actual destination path if no alternate display path provided
    let display_target = target_display_path.unwrap_or(dest);
    crate::metadata::check_metadata_replication(dest, meta, strict_lossless, true)?;
    let mode = meta.mode;
    #[cfg(unix)]
    let uid = meta.uid;
    #[cfg(unix)]
    let gid = meta.gid;
    let atime = FileTime::from_unix_time(
        meta.timestamps.atime_sec,
        meta.timestamps.atime_nsec,
    );
    let mtime = FileTime::from_unix_time(
        meta.timestamps.mtime_sec,
        meta.timestamps.mtime_nsec,
    );

    let dest_meta = if is_symlink {
        std::fs::symlink_metadata(dest)
    } else {
        std::fs::metadata(dest)
    };

    // 1. Ownership: check if dest already has the desired UID/GID
    #[cfg(unix)]
    {
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
                    NixAtFlags::AT_SYMLINK_NOFOLLOW,
                )
            } else {
                nix::unistd::chown(dest, uid_obj, gid_obj)
            };
            if let Err(err) = chown_res {
                if strict_lossless {
                    anyhow::bail!(
                        "Failed to preserve ownership (uid: {uid}, gid: {gid}) for {}: {err}",
                        display_target.display()
                    );
                }
                log_fmt!(
                    "Failed to preserve ownership (uid: {uid}, gid: {gid}) for {}: {err} (proceeding best-effort)",
                    display_target.display()
                );
            }
        }
    }

    // 2. Permissions (symlink permissions are fixed on Linux)
    #[cfg(unix)]
    if !is_symlink {
        let perms = Permissions::from_mode(mode);
        if let Err(e) = std::fs::set_permissions(dest, perms) {
            if strict_lossless {
                return Err(e).with_context(|| {
                    format!("Failed to set permissions on {}", display_target.display())
                });
            }
            log_fmt!(
                "Failed to set permissions on {}: {e} (proceeding best-effort)",
                display_target.display()
            );
        }
    }
    #[cfg(windows)]
    if !is_symlink && (mode & 0o222) == 0 {
        if let Ok(mut perms) = std::fs::metadata(dest).map(|m| m.permissions()) {
            perms.set_readonly(true);
            let _ = std::fs::set_permissions(dest, perms);
        }
    }

    // 3. Timestamps
    // Reason for fallback: If destination metadata cannot be queried, assume not a special file to proceed with standard timestamp update.
    let is_special = dest_meta
        .as_ref()
        .map_or(false, |m| !m.is_file() && !m.is_dir());
    let time_res = if is_symlink || is_special {
        set_symlink_file_times(dest, atime, mtime)
    } else {
        set_file_times(dest, atime, mtime)
    };
    if let Err(e) = time_res {
        if strict_lossless {
            return Err(e).with_context(|| {
                format!("Failed to set file timestamps on {}", display_target.display())
            });
        }
        log_fmt!(
            "Failed to set file timestamps on {}: {e} (proceeding best-effort)",
            display_target.display()
        );
    }

    #[cfg(windows)]
    {
        if let Some(ref native) = meta.native {
            crate::metadata::windows::apply_windows_security_metadata(dest, native, strict_lossless)?;
            if is_symlink {
                crate::metadata::windows::apply_windows_reparse_metadata(dest, native, strict_lossless)?;
            }
        }
        apply_windows_birthtime(dest, meta)?;
    }

    crate::metadata::acl::apply_bsd_acl_metadata(
        dest,
        meta.native.as_ref(),
        is_symlink,
        strict_lossless,
    )?;

    // 4. File flags
    if apply_flags && (!meta.flags.is_empty() || meta.platform_raw_flags.is_some()) {
        apply_file_flags(
            dest,
            &meta.flags,
            meta.platform_raw_flags.as_ref(),
            strict_lossless,
        )?;
    }

    if apply_flags {
        crate::metadata::check_metadata_replication(dest, meta, strict_lossless, false)?;
    }

    Ok(())
}

#[cfg(windows)]
#[expect(
    unsafe_code,
    reason = "Win32 CreateFileW and SetFileTime require FFI to set file creation timestamp"
)]
fn apply_windows_birthtime(dest: &Path, meta: &FileMetadata) -> Result<()> {
    if let Some(sec) = meta.timestamps.birthtime_sec {
        // Reason for fallback: Sub-second nanoseconds default to 0 when unrecorded in timestamp metadata.
        let nsec = meta.timestamps.birthtime_nsec.unwrap_or(0);
        let total_secs = u64::try_from(sec.saturating_add(11_644_473_600))
            .context("Invalid epoch conversion")?;
        let intervals = total_secs
            .checked_mul(10_000_000)
            .context("Overflow in birth time calculation")?
            .checked_add(u64::from(nsec).checked_div(100).context("Division error")?)
            .context("Overflow adding nanoseconds")?;
        let low = u32::try_from(intervals & 0xFFFF_FFFF)?;
        let high = u32::try_from(intervals >> 32)?;
        let ft = windows_sys::Win32::Foundation::FILETIME {
            dwLowDateTime: low,
            dwHighDateTime: high,
        };
        let wide = crate::metadata::windows::path_to_wide(dest)?;
        // SAFETY: wide is a null-terminated path and handle is properly closed if valid.
        let handle = unsafe {
            windows_sys::Win32::Storage::FileSystem::CreateFileW(
                wide.as_ptr(),
                windows_sys::Win32::Storage::FileSystem::FILE_WRITE_ATTRIBUTES,
                windows_sys::Win32::Storage::FileSystem::FILE_SHARE_READ
                    | windows_sys::Win32::Storage::FileSystem::FILE_SHARE_WRITE
                    | windows_sys::Win32::Storage::FileSystem::FILE_SHARE_DELETE,
                core::ptr::null(),
                windows_sys::Win32::Storage::FileSystem::OPEN_EXISTING,
                windows_sys::Win32::Storage::FileSystem::FILE_FLAG_BACKUP_SEMANTICS
                    | windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OPEN_REPARSE_POINT,
                core::ptr::null_mut(),
            )
        };
        if handle != windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE {
            unsafe {
                windows_sys::Win32::Storage::FileSystem::SetFileTime(
                    handle,
                    &raw const ft,
                    core::ptr::null(),
                    core::ptr::null(),
                );
                windows_sys::Win32::Foundation::CloseHandle(handle);
            }
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
        #[cfg(unix)]
        let entry_matches = entry_name.as_bytes() == expected_filename_bytes;
        #[cfg(not(unix))]
        let entry_matches = entry_name.as_encoded_bytes() == expected_filename_bytes;
        if entry_matches {
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

/// Global sequence generator for atomic temporary files.
static ATOMIC_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

/// Materializes a complete `FileEntity` onto the filesystem within a [`SandboxableDir`].
pub fn materialize_entity(
    entity: &FileEntity,
    payload: Option<&mut dyn PayloadSource>,
    dest_dir: &SandboxableDir,
    options: &MaterializeOptions,
) -> Result<MaterializeReceipt> {
    #[cfg(not(any(unix, windows)))]
    anyhow::ensure!(!options.strict_lossless, "Lossless materialization is not implemented on this platform");
    let dest_path = resolve_and_validate_path(
        dest_dir.root_path(),
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
            skipped_identical: false,
        });
    }

    let (parent_dir_fd, file_name) = dest_dir
        .ensure_parent_dir(&entity.identity.relative_path, options.path_policy)?;

    match &entity.kind {
        FileEntityKind::Symlink { target } => {
            if options.strict_lossless && entity.metadata.timestamps.birthtime_sec.is_some() {
                anyhow::bail!("Cannot reproduce symlink birth time on this platform");
            }
            dest_dir.create_symlink_at(
                &parent_dir_fd.as_fd(),
                &file_name,
                &dest_path,
                target,
                options.symlink_policy,
            )?;
            apply_entity_metadata(
                &dest_path,
                None,
                &entity.metadata,
                true,
                true,
                options.strict_lossless,
            )?;
            write_streams(&dest_path, None, &entity.streams, options.strict_lossless)?;

            Ok(MaterializeReceipt {
                destination_path: dest_path,
                bytes_written: 0,
                sha256: None,
                skipped_identical: false,
            })
        }
        FileEntityKind::Hardlink {
            target_relative_path,
        } => {
            let target_path = resolve_relative_path_for_os(target_relative_path, false)?;
            dest_dir.create_hardlink(
                &target_path,
                &parent_dir_fd.as_fd(),
                &file_name,
            )?;
            Ok(MaterializeReceipt {
                destination_path: dest_path,
                bytes_written: 0,
                sha256: None,
                skipped_identical: false,
            })
        }
        FileEntityKind::Directory | FileEntityKind::Bundle { .. } => {
            let _dir_fd =
                dest_dir.ensure_dir_all(&entity.identity.relative_path, options.path_policy)?;
            write_streams(&dest_path, None, &entity.streams, options.strict_lossless)?;
            apply_entity_metadata(
                &dest_path,
                None,
                &entity.metadata,
                false,
                true,
                options.strict_lossless,
            )?;
            Ok(MaterializeReceipt {
                destination_path: dest_path,
                bytes_written: 0,
                sha256: None,
                skipped_identical: false,
            })
        }
        FileEntityKind::Fifo
        | FileEntityKind::CharDevice { .. }
        | FileEntityKind::BlockDevice { .. } => {
            if options.strict_lossless && entity.metadata.timestamps.birthtime_sec.is_some() {
                anyhow::bail!("Cannot reproduce special-node birth time on this platform");
            }
            anyhow::ensure!(
                options.copy_specials,
                "Special node creation rejected: {}. Enable copy_specials to permit.",
                dest_path.display()
            );
            dest_dir.create_special(
                &parent_dir_fd.as_fd(),
                &file_name,
                &entity.kind,
                entity.metadata.mode,
            )?;
            apply_entity_metadata(
                &dest_path,
                None,
                &entity.metadata,
                false,
                true,
                options.strict_lossless,
            )?;
            write_streams(&dest_path, None, &entity.streams, options.strict_lossless)?;
            Ok(MaterializeReceipt {
                destination_path: dest_path,
                bytes_written: 0,
                sha256: None,
                skipped_identical: false,
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
            if !options.force_overwrite {
                if let Some(receipt) = try_update_existing_regular_entity(
                    &dest_path,
                    entity,
                    *size,
                    expected_sha256,
                    *is_sparse,
                    options,
                )? {
                    return Ok(receipt);
                }
            }

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
            let seq = ATOMIC_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let temp_name = format!(".csc-tmp.{pid}.{nanos}.{seq}");

            let mut temp_file = dest_dir.create_temp_file(
                &parent_dir_fd.as_fd(),
                &temp_name,
                entity.metadata.mode,
            )?;

            // RAII guard unlinking temp_file if an error occurs before commit_atomic_file
            let mut cleanup_guard = TempFileCleanupGuard {
                parent_fd: parent_dir_fd.as_fd(),
                temp_name: temp_name.clone(),
                active: true,
            };

            let mut hasher = Sha256Stream::new();
            let initial_size = *size;

            if *is_sparse {
                let mut extent_end = 0_u64;
                for extent in extents {
                    let (offset, length) = match extent {
                        Extent::Data { offset, length } | Extent::Hole { offset, length } => {
                            (*offset, *length)
                        }
                    };
                    anyhow::ensure!(offset == extent_end && length > 0, "Invalid sparse extent layout");
                    extent_end = offset.checked_add(length).context("Sparse extent overflow")?;
                    anyhow::ensure!(extent_end <= initial_size, "Sparse extent exceeds payload size");
                }
                anyhow::ensure!(extent_end == initial_size, "Sparse extents do not cover payload");
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
                                anyhow::ensure!(n != 0, "Unexpected EOF in sparse payload for {}", dest_path.display());
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
                let mut written = 0_u64;
                loop {
                    let n = source.read(&mut buf)?;
                    if n == 0 {
                        break;
                    }
                    written = written.checked_add(u64::try_from(n)?).context("Payload size overflow")?;
                    anyhow::ensure!(written <= initial_size, "Payload grew during copy of {}", dest_path.display());
                    let slice = buf
                        .get(..n)
                        .context("Buffer slice index out of bounds for write")?;
                    temp_file.write_all(slice)?;
                    hasher.update(slice);
                }
                anyhow::ensure!(written == initial_size, "Unexpected EOF in payload for {}", dest_path.display());
            }

            let computed_sha256 = hasher.finalize();
            let is_sentinel = *expected_sha256 == [0_u8; 32];
            if !is_sentinel && computed_sha256 != *expected_sha256 {
                anyhow::bail!(
                    "Payload SHA-256 verification failed during write for {}: expected {:02x?}, got {:02x?}",
                    dest_path.display(),
                    expected_sha256,
                    computed_sha256
                );
            }

            let parent_dir = match dest_path.parent() {
                Some(p) if !p.as_os_str().is_empty() => p,
                _ => Path::new("."),
            };
            let temp_path = parent_dir.join(&temp_name);

            // Apply ownership, permissions, and timestamps to temp file (defer flags until after rename)
            apply_entity_metadata(
                &temp_path,
                Some(&dest_path),
                &entity.metadata,
                false,
                false,
                options.strict_lossless,
            )?;

            // Write attached streams (xattrs, resource forks)
            write_streams(&temp_path, Some(&dest_path), &entity.streams, options.strict_lossless)?;

            if options.strict_lossless {
                let mut expected = entity.clone();
                if let FileEntityKind::Regular { sha256, .. } = &mut expected.kind {
                    *sha256 = computed_sha256;
                }
                let audit_options = EntityAuditOptions {
                    ignore_flags: true,
                    ..EntityAuditOptions::default()
                };
                let (differences, _) = audit_entity_detailed(&temp_path, &expected, &audit_options)?;
                anyhow::ensure!(differences.is_empty(), "Staged file failed fidelity verification for {}: {:?}", dest_path.display(), differences);
            }

            // Sync temp file and parent directory
            temp_file.sync_all()?;
            drop(temp_file);

            #[cfg(unix)]
            rustix::fs::fsync(&parent_dir_fd).with_context(|| {
                format!("Failed to sync parent directory: {}", parent_dir.display())
            })?;

            // Atomic rename inside parent directory
            dest_dir.commit_atomic_file(&parent_dir_fd.as_fd(), &temp_name, &file_name)?;
            cleanup_guard.active = false;

            #[cfg(unix)]
            rustix::fs::fsync(&parent_dir_fd).with_context(|| {
                format!("Failed to sync parent directory: {}", parent_dir.display())
            })?;

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
            crate::metadata::check_metadata_replication(
                &dest_path, &entity.metadata, options.strict_lossless, false,
            )?;

            Ok(MaterializeReceipt {
                destination_path: dest_path,
                bytes_written: initial_size,
                sha256: Some(computed_sha256),
                skipped_identical: false,
            })
        }
    }
}

struct TempFileCleanupGuard<'a> {
    parent_fd: DirHandleRef<'a>,
    temp_name: String,
    active: bool,
}

impl Drop for TempFileCleanupGuard<'_> {
    fn drop(&mut self) {
        if self.active {
            #[cfg(unix)]
            let _ = rustix::fs::unlinkat(&self.parent_fd, &self.temp_name, AtFlags::empty());
            #[cfg(not(unix))]
            let _ = std::fs::remove_file(self.parent_fd.path.join(&self.temp_name));
        }
    }
}

/// Verifies a collection of filenames within a directory in a single pass.
pub fn verify_directory_filenames_exact(
    dir: &Path,
    expected_filenames: &[&[u8]],
) -> Result<()> {
    if expected_filenames.is_empty() {
        return Ok(());
    }

    let dir_path = match dir.as_os_str() {
        s if s.is_empty() => Path::new("."),
        _ => dir,
    };

    let mut found = std::collections::HashSet::new();
    for entry in std::fs::read_dir(dir_path)
        .with_context(|| format!("Failed to read directory: {}", dir_path.display()))?
    {
        let entry = entry.with_context(|| {
            format!("Error reading entry in directory: {}", dir_path.display())
        })?;
        #[cfg(unix)]
        found.insert(entry.file_name().as_bytes().to_vec());
        #[cfg(not(unix))]
        found.insert(entry.file_name().as_encoded_bytes().to_vec());
    }

    for expected in expected_filenames {
        anyhow::ensure!(
            found.contains(*expected),
            "Target filesystem altered, normalized, or discarded filename bytes for {:?}",
            String::from_utf8_lossy(expected)
        );
    }
    Ok(())
}

/// Convenience wrapper to materialize an entity given a destination root path.
pub fn materialize_entity_at_path(
    entity: &FileEntity,
    payload: Option<&mut dyn PayloadSource>,
    dest_root: &Path,
    options: &MaterializeOptions,
) -> Result<MaterializeReceipt> {
    let dest_dir = SandboxableDir::create_or_open(dest_root)?;
    materialize_entity(entity, payload, &dest_dir, options)
}

/// Attempts to validate and losslessly update an existing regular file in-place
/// without re-writing payload bytes if the content hash and structure match.
#[expect(clippy::too_many_lines, reason = "Comprehensive in-place entity validation and lossless metadata reconciliation")]
fn try_update_existing_regular_entity(
    dest_path: &Path,
    entity: &FileEntity,
    size: u64,
    expected_sha256: &[u8; 32],
    is_sparse: bool,
    options: &MaterializeOptions,
) -> Result<Option<MaterializeReceipt>> {
    let dest_meta = match std::fs::symlink_metadata(dest_path) {
        Ok(m) => m,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => {
            return Err(err).with_context(|| {
                format!("Failed to query metadata for {}", dest_path.display())
            });
        }
    };

    // 1. Must be a regular file
    if !dest_meta.is_file() {
        return Ok(None);
    }

    // 2. Size must match
    if dest_meta.len() != size {
        return Ok(None);
    }

    // 3. Inode must not be hardlinked to other files
    #[cfg(not(unix))]
    return Ok(None);
    #[cfg(unix)]
    if dest_meta.nlink() > entity.identity.nlink {
        return Ok(None);
    }

    // 4. Sparseness check: verify allocated blocks vs hole structure
    #[cfg(unix)]
    {
        let dest_is_sparse = dest_meta.blocks().saturating_mul(512) < size;
        if is_sparse != dest_is_sparse {
            return Ok(None);
        }
    }

    // 5. If expected hash is sentinel [0; 32], we cannot safely skip without payload verification
    if *expected_sha256 == [0_u8; 32] {
        return Ok(None);
    }

    // 6. Compute SHA-256 digest of dest_path non-invasively
    #[cfg(target_os = "linux")]
    let mut file = {
        use std::os::unix::fs::OpenOptionsExt;
        let mut opts = std::fs::File::options();
        opts.read(true);
        opts.custom_flags(nix::libc::O_NOATIME);
        match opts.open(dest_path) {
            Ok(f) => f,
            Err(_) => match std::fs::File::open(dest_path) {
                Ok(f) => f,
                Err(_) => return Ok(None),
            },
        }
    };
    #[cfg(not(target_os = "linux"))]
    let mut file = match std::fs::File::open(dest_path) {
        Ok(f) => f,
        Err(_) => return Ok(None),
    };

    let mut hasher = Sha256Stream::new();
    let mut buf = vec![0_u8; 64 * 1024];
    loop {
        let n = match file.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => return Ok(None),
        };
        let slice = buf
            .get(..n)
            .context("Buffer slice index out of bounds during hash check")?;
        hasher.update(slice);
    }
    drop(file);

    let computed_sha256 = hasher.finalize();
    if computed_sha256 != *expected_sha256 {
        return Ok(None);
    }

    // 7. Reconcile extended attributes (streams)
    let on_disk_streams = match read_and_hash_streams(dest_path) {
        Ok(s) => s,
        Err(e) => {
            if options.strict_lossless {
                return Err(e);
            }
            Vec::new()
        }
    };

    // Remove any extra streams that do not exist on the source entity
    for disk_stream in &on_disk_streams {
        let exists_in_source = entity
            .streams
            .iter()
            .any(|s| s.name == disk_stream.name);
        if !exists_in_source {
            let res = remove_stream(dest_path, &disk_stream.name.to_os_string()?);
            if let Err(e) = res {
                if options.strict_lossless {
                    log_fmt!(
                        "Could not remove extra stream {:?} in-place from {}: {e}; falling through to atomic replacement",
                        disk_stream.name.to_string_lossy(),
                        dest_path.display()
                    );
                    return Ok(None);
                }
            }
        }
    }

    // Write missing or mismatched streams
    for stream in &entity.streams {
        let matches_on_disk = on_disk_streams.iter().any(|d| {
            d.name == stream.name
                && match (&d.entity.kind, &stream.entity.kind) {
                    (
                        FileEntityKind::Regular { sha256: d_hash, .. },
                        FileEntityKind::Regular { sha256: s_hash, .. },
                    ) => d_hash == s_hash,
                    _ => false,
                }
        });

        if !matches_on_disk {
            write_streams(
                dest_path,
                Some(dest_path),
                std::slice::from_ref(stream),
                options.strict_lossless,
            )?;
        }
    }

    // 8. File Flags: clear any conflicting flags (e.g. immutable) before updating metadata
    let (dest_flags, dest_raw) = query_file_flags(dest_path, false).unwrap_or_default();
    #[cfg(target_os = "linux")]
    if !dest_flags.is_empty() {
        use rustix::fs::{IFlags, ioctl_setflags};
        if let Ok(f) = std::fs::OpenOptions::new().write(true).open(dest_path) {
            if let Err(err) = ioctl_setflags(&f, IFlags::empty()) {
                if options.strict_lossless {
                    log_fmt!(
                        "Warning: could not clear file flags for {}: {err}",
                        dest_path.display()
                    );
                }
            }
        }
    }

    // 9. Apply ownership, permissions, and timestamps (defer flags to next step)
    apply_entity_metadata(
        dest_path,
        Some(dest_path),
        &entity.metadata,
        false,
        false,
        options.strict_lossless,
    )?;

    // 10. Apply final file flags
    if !entity.metadata.flags.is_empty()
        || entity.metadata.platform_raw_flags.is_some()
        || !dest_flags.is_empty()
        || dest_raw.is_some()
    {
        apply_file_flags(
            dest_path,
            &entity.metadata.flags,
            entity.metadata.platform_raw_flags.as_ref(),
            options.strict_lossless,
        )?;
    }

    // 11. Strict lossless audit
    if options.strict_lossless {
        let audit_opts = EntityAuditOptions {
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
        };
        let (diffs, _) = audit_entity_detailed(dest_path, entity, &audit_opts)?;
        if !diffs.is_empty() {
            log_fmt!(
                "In-place metadata update on {} had discrepancies ({:?}); falling through to atomic replacement",
                dest_path.display(),
                diffs
            );
            return Ok(None);
        }
    }

    Ok(Some(MaterializeReceipt {
        destination_path: dest_path.to_path_buf(),
        bytes_written: 0,
        sha256: Some(computed_sha256),
        skipped_identical: true,
    }))
}
