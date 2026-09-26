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

use crate::apple_double::{
    AppleFormat, AppleWriteMode, create_apple_archive_from_entity,
    write_apple_double_companion, write_apple_single_double,
};
use crate::entity::{FileEntity, FileEntityKind};
use crate::identity::resolve_relative_path_for_os;
use crate::metadata::FileMetadata;
use crate::path_policy::{
    PathTraversalPolicy, SymlinkValidationPolicy, resolve_and_validate_path,
};
use crate::payload::{Extent, PayloadSource};
use crate::sandboxable_dir::{DirHandleRef, SandboxableDir};
use crate::streams::{read_and_hash_streams, remove_stream, write_streams};
use crate::sys_flags::{apply_file_flags, query_file_flags};
use crate::verifier::{EntityAuditOptions, audit_entity_detailed};
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
    /// Policy for writing AppleDouble, AppleSingle, or native streams.
    pub apple_write_mode: AppleWriteMode,
    /// Extension style when writing AppleSingle archives.
    pub apple_single_write_extension: crate::apple_double::AppleSingleExtension,
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
            apple_write_mode: AppleWriteMode::NativeOnly,
            apple_single_write_extension: crate::apple_double::AppleSingleExtension::WithoutExtension,
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

    let dest_meta = if is_symlink {
        std::fs::symlink_metadata(dest)
    } else {
        std::fs::metadata(dest)
    };

    // 1. Ownership: check if dest already has the desired UID/GID
    #[cfg(unix)]
    {
        if let (Some(uid), Some(gid)) = (meta.uid, meta.gid) {
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
    }

    // 2. Permissions (symlink permissions are fixed on Linux)
    #[cfg(unix)]
    if !is_symlink {
        if let Some(mode) = meta.mode {
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
    }
    #[cfg(windows)]
    if !is_symlink {
        if let Some(mode) = meta.mode {
            if (mode & 0o222) == 0 {
                if let Ok(mut perms) = std::fs::metadata(dest).map(|m| m.permissions()) {
                    perms.set_readonly(true);
                    let _ = std::fs::set_permissions(dest, perms);
                }
            }
        }
    }

    // 3. Timestamps
    if let Some(ref ts) = meta.timestamps {
        let atime = FileTime::from_unix_time(
            ts.atime_sec,
            ts.atime_nsec,
        );
        let mtime = FileTime::from_unix_time(
            ts.mtime_sec,
            ts.mtime_nsec,
        );

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
    let has_flags = meta.flags.as_ref().map_or(false, |f| !f.is_empty());
    if apply_flags && (has_flags || meta.platform_raw_flags.is_some()) {
        let flags_slice = meta.flags.as_deref().unwrap_or(&[]); // Reason for fallback: absent flags metadata defaults to empty slice
        apply_file_flags(
            dest,
            flags_slice,
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
    if let Some(ref ts) = meta.timestamps {
        if let Some(sec) = ts.birthtime_sec {
            // Reason for fallback: Sub-second nanoseconds default to 0 when unrecorded in timestamp metadata.
            let nsec = ts.birthtime_nsec.unwrap_or(0);
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
    let relative_path = entity
        .identity
        .relative_path
        .as_deref()
        .context("Entity missing relative path for materialization")?;
    let dest_path = resolve_and_validate_path(
        dest_dir.root_path(),
        relative_path,
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
        .ensure_parent_dir(relative_path, options.path_policy)?;

    match &entity.kind {
        FileEntityKind::Symlink { target } => {
            if options.strict_lossless && entity.metadata.timestamps.as_ref().and_then(|t| t.birthtime_sec).is_some() {
                anyhow::bail!("Cannot reproduce symlink birth time on this platform");
            }
            dest_dir.create_symlink_at(
                &parent_dir_fd.as_fd(),
                &file_name,
                &dest_path,
                target,
                options.symlink_policy,
            )?;
            write_streams(&dest_path, None, &entity.streams, options.strict_lossless)?;
            apply_entity_metadata(
                &dest_path,
                None,
                &entity.metadata,
                true,
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
                dest_dir.ensure_dir_all(relative_path, options.path_policy)?;
            let mut write_companion = false;
            match options.apple_write_mode {
                AppleWriteMode::ForceAppleDouble(_) => {
                    write_companion = true;
                }
                AppleWriteMode::MaybeAppleDouble(_) => {
                    // Reason for fallback: absent AppleMetadata defaults to false (no metadata to preserve)
                    let has_apple_meta = entity.metadata.apple.as_ref().map_or(false, |a| !a.is_empty());
                    let res = write_streams(&dest_path, None, &entity.streams, false);
                    if res.is_err() || has_apple_meta {
                        write_companion = true;
                    }
                }
                _ => {
                    write_streams(&dest_path, None, &entity.streams, options.strict_lossless)?;
                    // Reason for fallback: absent AppleMetadata defaults to false (no metadata to preserve)
                    if options.strict_lossless && entity.metadata.apple.as_ref().map_or(false, |a| !a.is_empty()) {
                        anyhow::bail!(
                            "Cannot preserve Apple metadata for directory {} natively without AppleDouble or AppleSingle",
                            dest_path.display()
                        );
                    }
                }
            }
            if write_companion {
                if let Some(style) = options.apple_write_mode.double_style() {
                    write_apple_double_companion(entity, &dest_path, dest_dir.root_path(), style)?;
                }
            }
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
            if options.strict_lossless && entity.metadata.timestamps.as_ref().and_then(|t| t.birthtime_sec).is_some() {
                anyhow::bail!("Cannot reproduce special-node birth time on this platform");
            }
            anyhow::ensure!(
                options.copy_specials,
                "Special node creation rejected: {}. Enable copy_specials to permit.",
                dest_path.display()
            );
            // Reason for fallback: absent file mode for special node defaults to standard 0o666 permissions
            let mode = entity.metadata.mode.unwrap_or(0o666);
            dest_dir.create_special(
                &parent_dir_fd.as_fd(),
                &file_name,
                &entity.kind,
                mode,
            )?;
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

            let is_apple_single = options.apple_write_mode == AppleWriteMode::ForceAppleSingle
                || (options.apple_write_mode == AppleWriteMode::MaybeAppleSingle
                    && (!entity.streams.is_empty()
                        // Reason for fallback: absent AppleMetadata defaults to false (no metadata to preserve)
                        || entity.metadata.apple.as_ref().map_or(false, |a| !a.is_empty())));

            if is_apple_single {
                let Some(source) = payload else {
                    anyhow::bail!(
                        "PayloadSource required to materialize AppleSingle file: {}",
                        dest_path.display()
                    );
                };
                let mut data_fork = Vec::new();
                let initial_size = *size;
                let mut buf = vec![0_u8; 65536];
                let mut read_bytes = 0_u64;
                while read_bytes < initial_size {
                    let to_read = std::cmp::min(
                        u64::try_from(buf.len())?,
                        initial_size.saturating_sub(read_bytes),
                    );
                    let read_len = usize::try_from(to_read)?;
                    let Some(buf_slice) = buf.get_mut(..read_len) else {
                        anyhow::bail!("Buffer slice out of bounds");
                    };
                    let n = source.read(buf_slice)?;
                    if n == 0 {
                        break;
                    }
                    read_bytes = read_bytes.saturating_add(u64::try_from(n)?);
                    let Some(read_slice) = buf.get(..n) else {
                        anyhow::bail!("Read buffer slice out of bounds");
                    };
                    data_fork.extend_from_slice(read_slice);
                }
                let archive = create_apple_archive_from_entity(
                    entity,
                    AppleFormat::AppleSingle,
                    Some(data_fork),
                )?;
                let single_bytes = write_apple_single_double(&archive)?;
                let mut hasher = Sha256Stream::new();
                hasher.update(&single_bytes);
                let single_sha256 = hasher.finalize();
                let single_size = u64::try_from(single_bytes.len())?;

                let pid = std::process::id();
                let nanos = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
                    Ok(dur) => dur.subsec_nanos(),
                    Err(_) => 0,
                };
                let seq = ATOMIC_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let temp_name = format!(".csc-tmp.{pid}.{nanos}.{seq}");

                // Reason for fallback: absent file mode for AppleSingle temp file defaults to standard 0o644 permissions
                let mode = entity.metadata.mode.unwrap_or(0o644);
                let mut temp_file = dest_dir.create_temp_file(
                    &parent_dir_fd.as_fd(),
                    &temp_name,
                    mode,
                )?;
                let mut cleanup_guard = TempFileCleanupGuard {
                    parent_fd: parent_dir_fd.as_fd(),
                    temp_name: temp_name.clone(),
                    active: true,
                };
                temp_file.write_all(&single_bytes)?;
                temp_file.sync_all()?;
                drop(temp_file);

                let parent_dir = match dest_path.parent() {
                    Some(p) if !p.as_os_str().is_empty() => p,
                    _ => Path::new("."),
                };
                let temp_path = parent_dir.join(&temp_name);
                apply_entity_metadata(
                    &temp_path,
                    Some(&dest_path),
                    &entity.metadata,
                    false,
                    false,
                    options.strict_lossless,
                )?;

                #[cfg(unix)]
                sync_parent_dir_best_effort(&parent_dir_fd, parent_dir);

                let actual_file_name = match options.apple_single_write_extension {
                    crate::apple_double::AppleSingleExtension::WithoutExtension => {
                        file_name.as_os_str().to_os_string()
                    }
                    crate::apple_double::AppleSingleExtension::As => {
                        let mut s = file_name.as_os_str().to_os_string();
                        s.push(".as");
                        s
                    }
                    crate::apple_double::AppleSingleExtension::Asf => {
                        let mut s = file_name.as_os_str().to_os_string();
                        s.push(".asf");
                        s
                    }
                };
                let actual_dest_path = if options.apple_single_write_extension
                    == crate::apple_double::AppleSingleExtension::WithoutExtension
                {
                    dest_path.clone()
                } else {
                    parent_dir.join(&actual_file_name)
                };

                dest_dir.commit_atomic_file(&parent_dir_fd.as_fd(), &temp_name, &actual_file_name)?;
                cleanup_guard.active = false;

                #[cfg(unix)]
                sync_parent_dir_best_effort(&parent_dir_fd, parent_dir);

                return Ok(MaterializeReceipt {
                    destination_path: actual_dest_path,
                    bytes_written: single_size,
                    sha256: Some(single_sha256),
                    skipped_identical: false,
                });
            }

            let Some(source) = payload else {
                anyhow::bail!(
                    "PayloadSource required to materialize regular file: {}",
                    dest_path.display()
                );
            };
            let payload_noatime = source.opened_with_noatime();

            let pid = std::process::id();
            let nanos = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
                Ok(dur) => dur.subsec_nanos(),
                Err(_) => 0,
            };
            let seq = ATOMIC_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let temp_name = format!(".csc-tmp.{pid}.{nanos}.{seq}");

            // Reason for fallback: absent file mode for regular temp file defaults to standard 0o644 permissions
            let mode = entity.metadata.mode.unwrap_or(0o644);
            let mut temp_file = dest_dir.create_temp_file(
                &parent_dir_fd.as_fd(),
                &temp_name,
                mode,
            )?;

            // RAII guard unlinking temp_file if an error occurs before commit_atomic_file
            let mut cleanup_guard = TempFileCleanupGuard {
                parent_fd: parent_dir_fd.as_fd(),
                temp_name: temp_name.clone(),
                active: true,
            };

            let mut hasher = Sha256Stream::new();
            let initial_size = *size;

            let mut can_use_sparse = *is_sparse;
            if can_use_sparse {
                let mut extent_end = 0_u64;
                for extent in extents {
                    let (offset, length) = match extent {
                        Extent::Data { offset, length } | Extent::Hole { offset, length } => {
                            (*offset, *length)
                        }
                    };
                    if offset != extent_end || length == 0 {
                        can_use_sparse = false;
                        break;
                    }
                    if let Some(next_end) = offset.checked_add(length) {
                        extent_end = next_end;
                        if extent_end > initial_size {
                            can_use_sparse = false;
                            break;
                        }
                    } else {
                        can_use_sparse = false;
                        break;
                    }
                }
                if extent_end != initial_size {
                    can_use_sparse = false;
                }
                if !can_use_sparse {
                    if options.strict_lossless {
                        anyhow::bail!(
                            "Invalid sparse extents layout for {}: extents do not strictly cover initial size {} (extent end: {})",
                            dest_path.display(),
                            initial_size,
                            extent_end
                        );
                    }
                    warn_fmt!(
                        "Caveat: Sparse extents layout does not strictly cover payload for {}; falling back to lossless linear stream copying",
                        dest_path.display()
                    );
                }
            }

            if can_use_sparse {
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

            let mut write_companion = false;
            match options.apple_write_mode {
                AppleWriteMode::ForceAppleDouble(_) => {
                    write_companion = true;
                }
                AppleWriteMode::MaybeAppleDouble(_) => {
                    // Reason for fallback: absent AppleMetadata defaults to false (no metadata to preserve)
                    let has_apple_meta = entity.metadata.apple.as_ref().map_or(false, |a| !a.is_empty());
                    let streams_res = write_streams(&temp_path, Some(&dest_path), &entity.streams, false);
                    if streams_res.is_err() || has_apple_meta {
                        write_companion = true;
                    }
                }
                _ => {
                    write_streams(&temp_path, Some(&dest_path), &entity.streams, options.strict_lossless)?;
                    // Reason for fallback: absent AppleMetadata defaults to false (no metadata to preserve)
                    if options.strict_lossless && entity.metadata.apple.as_ref().map_or(false, |a| !a.is_empty()) {
                        anyhow::bail!(
                            "Cannot preserve Apple metadata for {} natively without AppleDouble or AppleSingle",
                            dest_path.display()
                        );
                    }
                }
            }

            // Apply ownership, permissions, and timestamps to temp file (defer flags until after rename)
            apply_entity_metadata(
                &temp_path,
                Some(&dest_path),
                &entity.metadata,
                false,
                false,
                options.strict_lossless,
            )?;

            if options.strict_lossless {
                let mut expected = entity.clone();
                if let FileEntityKind::Regular { sha256, is_sparse, .. } = &mut expected.kind {
                    *sha256 = computed_sha256;
                    if !can_use_sparse {
                        *is_sparse = false;
                    }
                }
                let audit_options = EntityAuditOptions {
                    ignore_flags: true,
                    ignore_atime: !payload_noatime,
                    ..EntityAuditOptions::default()
                };
                let (differences, _) = audit_entity_detailed(&temp_path, &expected, &audit_options)?;
                anyhow::ensure!(differences.is_empty(), "Staged file failed fidelity verification for {}: {:?}", dest_path.display(), differences);
            }

            // Sync temp file and parent directory
            temp_file.sync_all()?;
            drop(temp_file);

            #[cfg(unix)]
            sync_parent_dir_best_effort(&parent_dir_fd, parent_dir);

            // Atomic rename inside parent directory
            dest_dir.commit_atomic_file(&parent_dir_fd.as_fd(), &temp_name, &file_name)?;
            cleanup_guard.active = false;

            if write_companion {
                if let Some(style) = options.apple_write_mode.double_style() {
                    write_apple_double_companion(entity, &dest_path, dest_dir.root_path(), style)?;
                }
            }

            #[cfg(unix)]
            sync_parent_dir_best_effort(&parent_dir_fd, parent_dir);

            // Deferred immutability: apply flags as the very last step!
            let has_flags = entity.metadata.flags.as_ref().map_or(false, |f| !f.is_empty());
            if has_flags
                || entity.metadata.platform_raw_flags.is_some()
            {
                let flags_slice = entity.metadata.flags.as_deref().unwrap_or(&[]); // Reason for fallback: absent flags metadata defaults to empty slice
                apply_file_flags(
                    &dest_path,
                    flags_slice,
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

#[cfg(unix)]
fn sync_parent_dir_best_effort(parent_fd: &impl rustix::fd::AsFd, parent_path: &Path) {
    if let Err(err) = rustix::fs::fsync(parent_fd) {
        let raw = err.raw_os_error();
        if raw == nix::libc::EINVAL
            || raw == nix::libc::ENOTSUP
            || raw == nix::libc::EOPNOTSUPP
            || raw == nix::libc::EBADF
        {
            warn_fmt!(
                "Caveat: Destination filesystem does not support directory fsync on {} ({err}); proceeding without barrier",
                parent_path.display()
            );
        } else {
            log_fmt!("Directory fsync note on {}: {err}", parent_path.display());
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
            let res = match &disk_stream.name {
                Some(name) => remove_stream(dest_path, &name.to_os_string()?),
                None => match disk_stream.kind {
                    crate::streams::StreamKind::MacOsResourceFork => {
                        remove_stream(
                            dest_path,
                            std::ffi::OsStr::new(
                                crate::streams::RESOURCE_FORK_XATTR_NAME,
                            ),
                        )
                    }
                    _ => Ok(()),
                },
            };
            if let Err(e) = res {
                if options.strict_lossless {
                    log_fmt!(
                        "Could not remove extra stream {:?} in-place from {}: {e}; falling through to atomic replacement",
                        disk_stream.to_string_lossy(),
                        dest_path.display()
                    );
                    return Ok(None);
                }
            }
        }
    }

    if let Some(style) = options.apple_write_mode.double_style() {
        // Reason for fallback: destination path without a parent directory defaults to current working directory "."
        let parent_dir = dest_path.parent().unwrap_or(Path::new("."));
        write_apple_double_companion(entity, dest_path, parent_dir, style)?;
    } else {
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
    }

    // 8. File Flags: clear any conflicting flags (e.g. immutable) before updating metadata
    // Reason for fallback: query_file_flags is best-effort when unsupported by the filesystem
    let (dest_flags, dest_raw) = query_file_flags(dest_path, false).unwrap_or_default();
    #[cfg(target_os = "linux")]
    if !dest_flags.is_empty() {
        use rustix::fs::{IFlags, ioctl_setflags};
        let f_res = std::fs::OpenOptions::new()
            .write(true)
            .open(dest_path)
            .or_else(|_| std::fs::OpenOptions::new().read(true).open(dest_path));
        if let Ok(f) = f_res {
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
    let has_flags = entity.metadata.flags.as_ref().map_or(false, |f| !f.is_empty());
    if has_flags
        || entity.metadata.platform_raw_flags.is_some()
        || !dest_flags.is_empty()
        || dest_raw.is_some()
    {
        let flags_slice = entity.metadata.flags.as_deref().unwrap_or(&[]); // Reason for fallback: absent flags metadata defaults to empty slice
        apply_file_flags(
            dest_path,
            flags_slice,
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
    use ctb_formats_checksum::Sha256Stream;
    use std::fs;
    use std::path::{PathBuf};

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_materialize_and_verify_regular_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dest_root = temp_dir.path().join("dest_root");
        fs::create_dir_all(&dest_root).unwrap();

        let payload_bytes = b"Testing robust file abstraction pipeline with SHA-256";
        let mut hasher = Sha256Stream::new();
        hasher.update(payload_bytes);
        let sha256 = hasher.finalize();
        let size = u64::try_from(payload_bytes.len()).unwrap();

        let rel_path = PathBuf::from("nested/test_file.bin");
        let baseline_file = tempfile::NamedTempFile::new_in(&dest_root).unwrap();
        fs::write(baseline_file.path(), payload_bytes).unwrap();
        let (flags, platform_raw_flags) =
            crate::sys_flags::query_file_flags(baseline_file.path(), false).unwrap();
        baseline_file.close().unwrap();
        let entity = FileEntity {
            identity: FileIdentity {
                origin: FileOrigin::Synthetic,
                relative_path: Some(rel_path.clone()),
                enclosing_path: None,
                raw_relative_path: Some(rel_path.as_os_str().as_encoded_bytes().to_vec()),
                raw_filename: Some(b"test_file.bin".to_vec()),
                nlink: 1,
                hardlink_group: None,
            },
            metadata: FileMetadata {
                native: None,
                mode: Some(0o644),
                uid: Some(nix::unistd::getuid().as_raw()),
                gid: Some(nix::unistd::getgid().as_raw()),
                timestamps: Some(FileTimestamps {
                    atime_sec: 1_700_000_000,
                    atime_nsec: 0,
                    mtime_sec: 1_700_000_000,
                    mtime_nsec: 0,
                    ctime_sec: 1_700_000_000,
                    ctime_nsec: 0,
                    birthtime_sec: None,
                    birthtime_nsec: None,
                    resolution_nsec: None,
                }),
                flags: Some(flags),
                platform_raw_flags,
                read_time: None,
                filesystem_type: None,
                environment: None,
                apple: None,
            },
            kind: FileEntityKind::Regular {
                size,
                sha256,
                is_sparse: false,
                extents: vec![Extent::Data { offset: 0, length: size }],
            },
            streams: Vec::new(),
        };

        let mut payload = MemoryPayloadSource::new(payload_bytes.to_vec()).unwrap();
        let options = MaterializeOptions::default();

        let dest_dir = SandboxableDir::open(&dest_root).unwrap();
        let receipt = materialize_entity(
            &entity,
            Some(&mut payload),
            &dest_dir,
            &options,
        )
        .expect("materialize regular file");

        assert_eq!(receipt.bytes_written, size);
        assert_eq!(receipt.sha256, Some(sha256));

        // Independent verify pass
        let target_path = dest_root.join(&rel_path);
        verify_materialized_entity(&target_path, &entity, true)
            .expect("independent verification of materialized entity");

        for (is_sparse, extents, bytes) in [
            (false, Vec::new(), payload_bytes[..4].to_vec()),
            (false, Vec::new(), vec![1_u8; payload_bytes.len() + 1]),
            (true, vec![Extent::Data { offset: 0, length: size }], payload_bytes[..4].to_vec()),
            (true, vec![Extent::Hole { offset: 1, length: size }], payload_bytes.to_vec()),
            (true, Vec::new(), payload_bytes.to_vec()),
        ] {
            let mut invalid_entity = entity.clone();
            invalid_entity.kind = FileEntityKind::Regular {
                size,
                sha256: [0_u8; 32],
                is_sparse,
                extents,
            };
            let mut invalid_payload = MemoryPayloadSource::new(bytes).unwrap();
            assert!(materialize_entity(
                &invalid_entity,
                Some(&mut invalid_payload),
                &dest_dir,
                &options,
            ).is_err());
            assert_eq!(fs::read(&target_path).unwrap(), payload_bytes);
            assert_eq!(fs::read_dir(target_path.parent().unwrap()).unwrap().count(), 1);
        }
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_materialize_entity_skips_identical_payload_and_updates_metadata() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dest_root = temp_dir.path().join("dest");
        fs::create_dir_all(&dest_root).unwrap();

        let payload_bytes = b"Antigravity smart materialization payload";
        let size = u64::try_from(payload_bytes.len()).unwrap();
        let mut hasher = Sha256Stream::new();
        hasher.update(payload_bytes);
        let sha256 = hasher.finalize();

        let rel_path = PathBuf::from("docs/report.txt");

        // 1. Initial materialization: writes payload to disk
        let mut entity = FileEntity {
            identity: FileIdentity {
                origin: FileOrigin::Synthetic,
                relative_path: Some(rel_path.clone()),
                enclosing_path: None,
                raw_relative_path: Some(rel_path.as_os_str().as_encoded_bytes().to_vec()),
                raw_filename: Some(b"report.txt".to_vec()),
                nlink: 1,
                hardlink_group: None,
            },
            metadata: FileMetadata {
                native: None,
                mode: Some(0o600),
                uid: Some(nix::unistd::getuid().as_raw()),
                gid: Some(nix::unistd::getgid().as_raw()),
                timestamps: Some(FileTimestamps {
                    atime_sec: 1_700_000_000,
                    atime_nsec: 0,
                    mtime_sec: 1_700_000_000,
                    mtime_nsec: 0,
                    ctime_sec: 1_700_000_000,
                    ctime_nsec: 0,
                    birthtime_sec: None,
                    birthtime_nsec: None,
                    resolution_nsec: None,
                }),
                flags: Some(Vec::new()),
                platform_raw_flags: None,
                read_time: None,
                filesystem_type: None,
                environment: None,
                apple: None,
            },
            kind: FileEntityKind::Regular {
                size,
                sha256,
                is_sparse: false,
                extents: vec![Extent::Data { offset: 0, length: size }],
            },
            streams: Vec::new(),
        };

        let mut payload = MemoryPayloadSource::new(payload_bytes.to_vec()).unwrap();
        let dest_dir = SandboxableDir::open(&dest_root).unwrap();
        let options = MaterializeOptions::default();

        let receipt1 = entity
            .materialize(Some(&mut payload), &dest_dir, &options)
            .expect("first materialization");
        assert_eq!(receipt1.bytes_written, size);
        assert!(!receipt1.skipped_identical);

        let target_path = dest_root.join(&rel_path);
        let meta1 = fs::metadata(&target_path).unwrap();
        assert_eq!(meta1.permissions().mode() & 0o7777, 0o600);

        // 2. Second materialization with updated permissions (0o644) and mtime
        entity.metadata.mode = Some(0o644);
        if let Some(ref mut ts) = entity.metadata.timestamps {
            ts.mtime_sec = 1_700_050_000;
        }
        let mut payload2 = MemoryPayloadSource::new(payload_bytes.to_vec()).unwrap();

        let receipt2 = entity
            .materialize(Some(&mut payload2), &dest_dir, &options)
            .expect("second smart materialization");
        assert_eq!(receipt2.bytes_written, 0);
        assert!(receipt2.skipped_identical);

        let meta2 = fs::metadata(&target_path).unwrap();
        assert_eq!(meta2.permissions().mode() & 0o7777, 0o644);
        assert_eq!(meta2.mtime(), 1_700_050_000);

        // Independent verification of the in-place updated entity
        verify_materialized_entity(&target_path, &entity, true)
            .expect("verification of smart-updated entity");
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_materialize_entity_force_overwrite() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dest_root = temp_dir.path().join("dest");
        fs::create_dir_all(&dest_root).unwrap();

        let payload_bytes = b"Payload for force overwrite test";
        let size = u64::try_from(payload_bytes.len()).unwrap();
        let mut hasher = Sha256Stream::new();
        hasher.update(payload_bytes);
        let sha256 = hasher.finalize();

        let rel_path = PathBuf::from("forced.bin");
        let entity = FileEntity {
            identity: FileIdentity {
                origin: FileOrigin::Synthetic,
                relative_path: Some(rel_path.clone()),
                enclosing_path: None,
                raw_relative_path: Some(rel_path.as_os_str().as_encoded_bytes().to_vec()),
                raw_filename: Some(b"forced.bin".to_vec()),
                nlink: 1,
                hardlink_group: None,
            },
            metadata: FileMetadata {
                native: None,
                mode: Some(0o644),
                uid: Some(nix::unistd::getuid().as_raw()),
                gid: Some(nix::unistd::getgid().as_raw()),
                timestamps: Some(FileTimestamps {
                    atime_sec: 1_700_000_000,
                    atime_nsec: 0,
                    mtime_sec: 1_700_000_000,
                    mtime_nsec: 0,
                    ctime_sec: 1_700_000_000,
                    ctime_nsec: 0,
                    birthtime_sec: None,
                    birthtime_nsec: None,
                    resolution_nsec: None,
                }),
                flags: Some(Vec::new()),
                platform_raw_flags: None,
                read_time: None,
                filesystem_type: None,
                environment: None,
                apple: None,
            },
            kind: FileEntityKind::Regular {
                size,
                sha256,
                is_sparse: false,
                extents: vec![Extent::Data { offset: 0, length: size }],
            },
            streams: Vec::new(),
        };

        let dest_dir = SandboxableDir::open(&dest_root).unwrap();
        let mut p1 = MemoryPayloadSource::new(payload_bytes.to_vec()).unwrap();
        let r1 = entity
            .materialize(Some(&mut p1), &dest_dir, &MaterializeOptions::default())
            .expect("initial write");
        assert_eq!(r1.bytes_written, size);
        assert!(!r1.skipped_identical);

        // Overwrite with force_overwrite = true
        let mut forced_options = MaterializeOptions::default();
        forced_options.force_overwrite = true;

        let mut p2 = MemoryPayloadSource::new(payload_bytes.to_vec()).unwrap();
        let r2 = entity
            .materialize(Some(&mut p2), &dest_dir, &forced_options)
            .expect("forced rewrite");
        assert_eq!(r2.bytes_written, size);
        assert!(!r2.skipped_identical);
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_materialize_entity_payload_mismatch_rewrites_atomic() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dest_root = temp_dir.path().join("dest");
        fs::create_dir_all(&dest_root).unwrap();

        let rel_path = PathBuf::from("mismatch.txt");
        let dest_file = dest_root.join(&rel_path);
        // Pre-create destination with different content of the same length
        fs::write(&dest_file, b"Old contentAAAA").unwrap();

        let new_bytes = b"New contentBBBB";
        let size = u64::try_from(new_bytes.len()).unwrap();
        let mut hasher = Sha256Stream::new();
        hasher.update(new_bytes);
        let sha256 = hasher.finalize();

        let entity = FileEntity {
            identity: FileIdentity {
                origin: FileOrigin::Synthetic,
                relative_path: Some(rel_path.clone()),
                enclosing_path: None,
                raw_relative_path: Some(rel_path.as_os_str().as_encoded_bytes().to_vec()),
                raw_filename: Some(b"mismatch.txt".to_vec()),
                nlink: 1,
                hardlink_group: None,
            },
            metadata: FileMetadata {
                native: None,
                mode: Some(0o644),
                uid: Some(nix::unistd::getuid().as_raw()),
                gid: Some(nix::unistd::getgid().as_raw()),
                timestamps: Some(FileTimestamps {
                    atime_sec: 1_700_000_000,
                    atime_nsec: 0,
                    mtime_sec: 1_700_000_000,
                    mtime_nsec: 0,
                    ctime_sec: 1_700_000_000,
                    ctime_nsec: 0,
                    birthtime_sec: None,
                    birthtime_nsec: None,
                    resolution_nsec: None,
                }),
                flags: Some(Vec::new()),
                platform_raw_flags: None,
                read_time: None,
                filesystem_type: None,
                environment: None,
                apple: None,
            },
            kind: FileEntityKind::Regular {
                size,
                sha256,
                is_sparse: false,
                extents: vec![Extent::Data { offset: 0, length: size }],
            },
            streams: Vec::new(),
        };

        let dest_dir = SandboxableDir::open(&dest_root).unwrap();
        let mut payload = MemoryPayloadSource::new(new_bytes.to_vec()).unwrap();
        let receipt = entity
            .materialize(Some(&mut payload), &dest_dir, &MaterializeOptions::default())
            .expect("materialize on mismatch");

        assert_eq!(receipt.bytes_written, size);
        assert!(!receipt.skipped_identical);
        assert_eq!(fs::read(&dest_file).unwrap(), new_bytes);
    }

    #[crate::ctb_test]
    fn test_materialize_entity_sparse_extent_fallback() {
        let temp = tempfile::tempdir().unwrap();
        let src_path = temp.path().join("sparse_src.bin");
        let dest_path = temp.path().join("sparse_dest.bin");

        // Create a 100-byte test file
        let mut data = vec![0_u8; 100];
        for (i, byte) in data.iter_mut().enumerate() {
            // Safe truncated u8 value for test pattern
            *byte = u8::try_from(i % 251).unwrap_or(0);
        }
        std::fs::write(&src_path, &data).unwrap();

        let mut entity = FileEntity::from_filesystem(&src_path, None).unwrap();
        // Artificially corrupt extent map so it fails validation (claims file is sparse, but extents total only 50 bytes)
        if let FileEntityKind::Regular {
            ref mut is_sparse,
            ref mut extents,
            ..
        } = entity.kind
        {
            *is_sparse = true;
            *extents = vec![
                Extent::Data {
                    offset: 0,
                    length: 30,
                },
                Extent::Hole {
                    offset: 30,
                    length: 20,
                },
            ];
        }
        entity.identity.relative_path = Some(PathBuf::from("sparse_dest.bin"));

        let mut payload = DiskPayloadSource::open(&src_path).unwrap();
        let options = MaterializeOptions {
            dry_run: false,
            strict_lossless: false,
            symlink_policy: SymlinkValidationPolicy::PreserveVerbatim,
            path_policy: PathTraversalPolicy::StrictSandboxed,
            copy_specials: false,
            force_overwrite: true,
            apple_write_mode: crate::apple_double::AppleWriteMode::NativeOnly,
            apple_single_write_extension: crate::apple_double::AppleSingleExtension::WithoutExtension,
        };

        let dest_dir = SandboxableDir::open(temp.path()).unwrap();
        // Materialization should fall back to linear copy and succeed completely without failing
        materializer::materialize_entity(&entity, Some(&mut payload), &dest_dir, &options).unwrap();

        let read_back = std::fs::read(&dest_path).unwrap();
        assert_eq!(read_back, data);
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_failed_node_replacement_preserves_destination() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("destination");
        fs::write(&path, b"keep this data").unwrap();
        let root = SandboxableDir::open(temp.path()).unwrap();
        assert!(root.create_hardlink(PathBuf::from("missing").as_path(), &root.root_fd(), "destination").is_err());
        assert!(root.create_symlink(&root.root_fd(), "destination", b"invalid\0target", SymlinkValidationPolicy::PreserveVerbatim).is_err());
        assert!(root.create_special(&root.root_fd(), "destination", &FileEntityKind::Socket, 0o600).is_err());
        assert_eq!(fs::read(path).unwrap(), b"keep this data");
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 1);
    }
}

