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
    AppleFormat, AppleWriteMode, StreamAdaptationRecord,
    apple_double_companion_path,
    create_apple_archive_from_entity_with_stream_source,
    plan_windows_ads_companions,
    serialize_apple_double_for_entity_with_stream_source,
    write_apple_double_companion_bytes,
    write_apple_double_companion_with_stream_source, write_apple_single_double,
    write_windows_ads_companions_with_warnings,
};
use crate::entity::{FileEntity, FileEntityKind};
use crate::identity::resolve_relative_path_for_os;
use crate::metadata::{FileMetadata, OsFamily};
use crate::path_policy::{
    PathTraversalPolicy, SymlinkValidationPolicy, resolve_and_validate_path,
};
use crate::payload::{Extent, PayloadSource};
use crate::sandboxable_dir::{
    DirHandleRef, ExistingEntryPolicy, SandboxableDir,
};
use crate::streams::{read_and_hash_streams, remove_stream, write_streams};
use crate::sys_flags::{
    apply_file_flags, apply_file_flags_with_source, query_file_flags,
};
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
#[expect(
    clippy::struct_excessive_bools,
    reason = "Configuration flags for materialization fidelity"
)]
pub struct MaterializeOptions {
    /// If true, calculate operations without modifying the filesystem.
    pub dry_run: bool,
    /// If true, enforce strict zero-data-loss for metadata and streams.
    pub strict_lossless: bool,
    /// Require access-time equality after materialization and verification.
    pub check_atime: bool,
    /// Require change-time equality even on platforms that cannot set it.
    pub check_ctime: bool,
    /// Require birth-time equality even on platforms that cannot set it.
    pub check_birthtime: bool,
    /// Policy for validating symlink targets.
    pub symlink_policy: SymlinkValidationPolicy,
    /// Policy for validating relative entry paths.
    pub path_policy: PathTraversalPolicy,
    /// Permit creating special nodes (FIFOs, character and block devices).
    pub copy_specials: bool,
    /// If true, always overwrite the destination by recreating the payload and metadata atomically,
    /// skipping any in-place reuse even if payload checksums match.
    pub force_overwrite: bool,
    /// Authorization to replace existing entries or modify their metadata.
    pub existing_entry_policy: ExistingEntryPolicy,
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
            check_atime: false,
            check_ctime: cfg!(windows),
            check_birthtime: cfg!(windows),
            symlink_policy: SymlinkValidationPolicy::PreserveVerbatim,
            path_policy: PathTraversalPolicy::StrictSandboxed,
            copy_specials: false,
            force_overwrite: false,
            existing_entry_policy: ExistingEntryPolicy::Refuse,
            apple_write_mode: AppleWriteMode::NativeOnly,
            apple_single_write_extension:
                crate::apple_double::AppleSingleExtension::WithoutExtension,
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
    /// Fidelity issues observed while materializing the entity.
    pub warnings: Vec<String>,
    /// Stream name and companion path adaptations applied in best-effort mode.
    pub stream_adaptations: Vec<StreamAdaptationRecord>,
}

impl MaterializeReceipt {
    /// Applies any stream adaptations recorded during materialization to `entity.streams`.
    pub fn apply_stream_adaptations(&self, entity: &mut FileEntity) {
        for adapt in &self.stream_adaptations {
            if let Some(stream) = entity.streams.get_mut(adapt.stream_index) {
                if adapt.source_stream_name != adapt.materialized_stream_name {
                    stream.entity.metadata.set_materialized_stream_name(
                        &adapt.source_stream_name,
                        &adapt.materialized_stream_name,
                    );
                }
                if let (Some(src_rel), Some(mat_rel)) = (
                    adapt.source_relative_path.as_deref(),
                    adapt.materialized_relative_path.as_deref(),
                ) {
                    if src_rel != mat_rel
                        || adapt.source_stream_name
                            != adapt.materialized_stream_name
                    {
                        stream
                            .entity
                            .metadata
                            .set_materialized_relative_path(src_rel, mat_rel);
                    }
                    stream.entity.identity.relative_path =
                        Some(mat_rel.to_path_buf());
                }
            }
        }
    }
}

/// Applies ownership, permissions, and timestamps from `meta` to `dest`.
///
/// Returns observed best-effort fidelity warnings for retention in semantic
/// metadata or the operation journal. Strict failures remain errors.
pub fn apply_entity_metadata(
    dest: &Path,
    target_display_path: Option<&Path>,
    meta: &FileMetadata,
    is_symlink: bool,
    apply_flags: bool,
    strict_lossless: bool,
    check_birthtime: bool,
) -> Result<Vec<String>> {
    if let Some(timestamps) = &meta.timestamps {
        timestamps.validate()?;
    }
    let mut warnings = Vec::new();
    #[cfg(not(any(unix, windows)))]
    anyhow::ensure!(
        !strict_lossless,
        "Lossless ownership and permission preservation is not implemented on this platform"
    );
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
                    let warning = format!(
                        "Failed to preserve ownership (uid: {uid}, gid: {gid}) for {}: {err} (proceeding best-effort)",
                        display_target.display()
                    );
                    warn_fmt!("{warning}");
                    warnings.push(warning);
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
                        format!(
                            "Failed to set permissions on {}",
                            display_target.display()
                        )
                    });
                }
                let warning = format!(
                    "Failed to set permissions on {}: {e} (proceeding best-effort)",
                    display_target.display()
                );
                warn_fmt!("{warning}");
                warnings.push(warning);
            }
        }
    }
    #[cfg(windows)]
    if !is_symlink {
        if let Some(mode) = meta.mode {
            if (mode & 0o222) == 0 {
                let permission_result = (|| -> Result<()> {
                    let mut perms = std::fs::metadata(dest)?.permissions();
                    perms.set_readonly(true);
                    std::fs::set_permissions(dest, perms)?;
                    Ok(())
                })();
                if let Err(error) = permission_result {
                    let error = error.context(format!(
                        "Failed to set read-only permissions on {}",
                        display_target.display(),
                    ));
                    if strict_lossless {
                        return Err(error);
                    }
                    let warning = format!("{error:#} (proceeding best-effort)");
                    warn_fmt!("{warning}");
                    warnings.push(warning);
                }
            }
        }
    }

    // 3. Timestamps
    if let Some(ref ts) = meta.timestamps {
        // Reason for fallback: If destination metadata cannot be queried, assume not a special file to proceed with standard timestamp update.
        let is_special = dest_meta
            .as_ref()
            .is_ok_and(|m| !m.is_file() && !m.is_dir());
        let time_res = (|| -> Result<()> {
            let atime = crate::metadata::checked_file_time(
                ts.atime_sec,
                ts.atime_nsec,
            )?;
            let mtime = crate::metadata::checked_file_time(
                ts.mtime_sec,
                ts.mtime_nsec,
            )?;
            if is_symlink || is_special {
                set_symlink_file_times(dest, atime, mtime)?;
            } else {
                set_file_times(dest, atime, mtime)?;
            }
            Ok(())
        })();
        if let Err(e) = time_res {
            if strict_lossless {
                return Err(e).with_context(|| {
                    format!(
                        "Failed to set file timestamps on {}",
                        display_target.display()
                    )
                });
            }
            let warning = format!(
                "Failed to set file timestamps on {}: {e} (proceeding best-effort)",
                display_target.display()
            );
            warn_fmt!("{warning}");
            warnings.push(warning);
        }
    }

    #[cfg(windows)]
    {
        if let Some(ref native) = meta.native {
            crate::metadata::windows::apply_windows_security_metadata(
                dest,
                native,
                strict_lossless,
            )?;
            if is_symlink {
                crate::metadata::windows::apply_windows_reparse_metadata(
                    dest,
                    native,
                    strict_lossless,
                )?;
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
    let has_flags = meta.flags.is_some() || meta.platform_raw_flags.is_some();
    if apply_flags && has_flags {
        let flags_slice = meta.flags.as_deref().unwrap_or(&[]); // Reason for fallback: absent flags metadata defaults to empty slice
        let source_os = meta
            .environment
            .as_ref()
            .map_or(OsFamily::CURRENT, |env| env.os_family());
        apply_file_flags_with_source(
            dest,
            flags_slice,
            meta.platform_raw_flags.as_ref(),
            source_os,
            strict_lossless,
        )?;
    }

    if apply_flags {
        #[cfg(windows)]
        warnings.extend(apply_windows_change_time(
            dest,
            meta,
            strict_lossless,
        )?);
        warnings.extend(crate::metadata::check_metadata_replication(
            dest,
            meta,
            strict_lossless,
            false,
            check_birthtime,
        )?);
    }

    Ok(warnings)
}

#[cfg(windows)]
fn apply_windows_change_time(
    dest: &Path,
    meta: &FileMetadata,
    strict_lossless: bool,
) -> Result<Vec<String>> {
    let Some(timestamps) = &meta.timestamps else {
        return Ok(Vec::new());
    };
    let result = (|| -> Result<()> {
        let time = crate::metadata::checked_file_time(
            timestamps.ctime_sec,
            timestamps.ctime_nsec,
        )?;
        crate::metadata::windows::set_file_change_time(dest, time)
    })();
    if let Err(error) = result {
        let error = error.context(format!(
            "Failed to restore change time on {}",
            dest.display()
        ));
        if strict_lossless {
            return Err(error);
        }
        let warning = format!("{error:#} (proceeding best-effort)");
        warn_fmt!("{warning}");
        return Ok(vec![warning]);
    }
    Ok(Vec::new())
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
                .checked_add(
                    u64::from(nsec)
                        .checked_div(100)
                        .context("Division error")?,
                )
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
            if handle == windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE {
                return Err(std::io::Error::last_os_error()).with_context(
                    || {
                        format!(
                            "Failed to open {} to restore birth time",
                            dest.display()
                        )
                    },
                );
            }
            // SAFETY: handle is live, ft remains initialized, and the handle is closed only after SetFileTime returns.
            let result = unsafe {
                let result =
                    windows_sys::Win32::Storage::FileSystem::SetFileTime(
                        handle,
                        &raw const ft,
                        core::ptr::null(),
                        core::ptr::null(),
                    );
                let error = if result == 0 {
                    Some(std::io::Error::last_os_error())
                } else {
                    None
                };
                windows_sys::Win32::Foundation::CloseHandle(handle);
                error
            };
            if let Some(error) = result {
                return Err(error).with_context(|| {
                    format!(
                        "Failed to restore birth time on {}",
                        dest.display()
                    )
                });
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

    verify_directory_filenames_exact(parent_dir, &[expected_filename_bytes])
}

/// Global sequence generator for atomic temporary files.
static ATOMIC_SEQ: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(1);

/// Materializes a complete `FileEntity` onto the filesystem within a [`SandboxableDir`].
pub fn materialize_entity(
    entity: &FileEntity,
    payload: Option<&mut dyn PayloadSource>,
    dest_dir: &SandboxableDir,
    options: &MaterializeOptions,
) -> Result<MaterializeReceipt> {
    materialize_entity_with_stream_source(
        entity,
        payload,
        dest_dir,
        options,
        &crate::streams::FilesystemStreamSource,
    )
}

/// Materializes primary and attached bodies before publishing a regular file.
pub fn materialize_entity_with_stream_source(
    entity: &FileEntity,
    payload: Option<&mut dyn PayloadSource>,
    dest_dir: &SandboxableDir,
    options: &MaterializeOptions,
    stream_source: &dyn crate::streams::AttachedStreamSource,
) -> Result<MaterializeReceipt> {
    if let Some(timestamps) = &entity.metadata.timestamps {
        timestamps.validate()?;
    }
    #[cfg(not(any(unix, windows)))]
    anyhow::ensure!(
        !options.strict_lossless,
        "Lossless materialization is not implemented on this platform"
    );
    if entity.is_error() {
        let err_details = if entity.metadata.errors.is_empty() {
            "entity is marked inaccessible".to_string()
        } else {
            entity.metadata.errors.join("; ")
        };
        anyhow::bail!(
            "Refusing to materialize inaccessible entity or entity with recorded errors ({}): {}",
            entity.identity.display_path(),
            err_details
        );
    }
    let orig_relative_path = entity
        .identity
        .relative_path
        .as_deref()
        .context("Entity missing relative path for materialization")?;
    let _validated_orig = resolve_and_validate_path(
        dest_dir.root_path(),
        orig_relative_path,
        options.path_policy,
    )?;

    let max_leaf_bytes = if options.strict_lossless {
        crate::clean_name::MAX_FILENAME_BYTES
    } else if options.apple_write_mode.is_windows_ads()
        && !entity.streams.is_empty()
    {
        let max_stream_len: usize = entity
            .streams
            .iter()
            .map(|s| {
                // Reason for fallback: unnamed stream (e.g. Mac resource fork) reserves 7 bytes for companion suffix
                s.name
                    .as_ref()
                    .map_or(7, |n| n.as_bytes().len().clamp(8, 64))
            })
            .max()
            .unwrap_or(16); // Reason for fallback: non-empty streams slice always yields Some
        crate::clean_name::MAX_FILENAME_BYTES
            .saturating_sub(max_stream_len.saturating_add(1))
    } else if options.apple_write_mode.double_style().is_some()
        && (!entity.streams.is_empty()
            // Reason for fallback: absent AppleMetadata defaults to false
            || entity.metadata.apple.as_ref().is_some_and(|a| !a.is_empty()))
    {
        crate::clean_name::MAX_FILENAME_BYTES.saturating_sub(2)
    } else if (options.apple_write_mode == AppleWriteMode::ForceAppleSingle
        || options.apple_write_mode == AppleWriteMode::MaybeAppleSingle)
        && options.apple_single_write_extension
            != crate::apple_double::AppleSingleExtension::WithoutExtension
    {
        crate::clean_name::MAX_FILENAME_BYTES.saturating_sub(4)
    } else {
        crate::clean_name::MAX_FILENAME_BYTES
    };

    let (mut relative_path_buf, mut path_warnings) = if options.strict_lossless
    {
        (orig_relative_path.to_path_buf(), Vec::new())
    } else {
        let (mut adapted, mut warnings) =
            crate::clean_name::adapt_relative_path_for_materialization(
                orig_relative_path,
                cfg!(windows),
            );
        if max_leaf_bytes < crate::clean_name::MAX_FILENAME_BYTES {
            if let Some(leaf) =
                adapted.file_name().map(std::ffi::OsStr::to_os_string)
            {
                if leaf.as_encoded_bytes().len() > max_leaf_bytes {
                    let (short_leaf, reason) =
                        crate::clean_name::adapt_component_with_max_bytes(
                            &leaf,
                            cfg!(windows),
                            max_leaf_bytes,
                        );
                    let new_adapted = match adapted.parent() {
                        Some(p) if !p.as_os_str().is_empty() => {
                            p.join(&short_leaf)
                        }
                        _ => PathBuf::from(&short_leaf),
                    };
                    let msg = format!(
                        "Adapted destination path {:?} -> {:?} to leave room for companion filename (best-effort: {})",
                        adapted.display(),
                        new_adapted.display(),
                        reason.as_deref().unwrap_or("truncated base filename") // Reason for fallback: default description when base filename is shortened
                    );
                    warnings.push(msg);
                    adapted = new_adapted;
                }
            }
        }
        for w in &warnings {
            warn_fmt!("{w}");
        }
        (adapted, warnings)
    };
    let mut relative_path: &Path = &relative_path_buf;
    let mut dest_path = resolve_and_validate_path(
        dest_dir.root_path(),
        relative_path,
        options.path_policy,
    )?;

    if !options.strict_lossless
        && options.existing_entry_policy == ExistingEntryPolicy::Refuse
        && !entity.is_dir()
        && relative_path_buf != orig_relative_path
        && std::fs::symlink_metadata(&dest_path).is_ok()
    {
        if let Some(leaf_os) = relative_path_buf
            .file_name()
            .map(std::ffi::OsStr::to_os_string)
        {
            let leaf_str = leaf_os.to_string_lossy();
            // Reason for fallback: single-component relative path is already the file name
            let orig_leaf_bytes = orig_relative_path
                .file_name()
                .unwrap_or(orig_relative_path.as_os_str())
                .as_encoded_bytes();
            let mut collision_idx = 1_usize;
            loop {
                let disambiguated =
                    crate::clean_name::disambiguate_cleaned_name(
                        &leaf_str,
                        orig_leaf_bytes,
                        collision_idx,
                        max_leaf_bytes,
                    );
                let cand_rel = match relative_path_buf.parent() {
                    Some(p) if !p.as_os_str().is_empty() => {
                        p.join(&disambiguated)
                    }
                    _ => PathBuf::from(&disambiguated),
                };
                let cand_dest = resolve_and_validate_path(
                    dest_dir.root_path(),
                    &cand_rel,
                    options.path_policy,
                )?;
                if std::fs::symlink_metadata(&cand_dest).is_err() {
                    let w = format!(
                        "Disambiguated colliding adapted destination path {:?} -> {:?} (best-effort)",
                        relative_path_buf.display(),
                        cand_rel.display()
                    );
                    warn_fmt!("{w}");
                    path_warnings.push(w);
                    relative_path_buf = cand_rel;
                    relative_path = &relative_path_buf;
                    dest_path = cand_dest;
                    break;
                }
                collision_idx = collision_idx.saturating_add(1);
            }
        }
    }

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
            warnings: path_warnings,
            stream_adaptations: Vec::new(),
        });
    }

    let (mut parent_dir_fd, mut file_name) = match dest_dir
        .ensure_parent_dir(relative_path, options.path_policy)
    {
        Ok(res) => res,
        Err(err)
            if !options.strict_lossless
                && is_filename_rejection_error(&err) =>
        {
            let (win_adapted, win_warnings) =
                crate::clean_name::adapt_relative_path_for_materialization(
                    orig_relative_path,
                    true,
                );
            if win_adapted == relative_path_buf {
                return Err(err);
            }
            for w in &win_warnings {
                warn_fmt!("{w}");
            }
            path_warnings.extend(win_warnings);
            relative_path_buf = win_adapted;
            relative_path = &relative_path_buf;
            dest_path = resolve_and_validate_path(
                dest_dir.root_path(),
                relative_path,
                options.path_policy,
            )?;
            dest_dir.ensure_parent_dir(relative_path, options.path_policy)?
        }
        Err(err) => return Err(err),
    };

    if options.existing_entry_policy == ExistingEntryPolicy::Refuse
        && !entity.is_dir()
    {
        match std::fs::symlink_metadata(&dest_path) {
            Ok(_) => bail!(
                "Destination already exists; explicit overwrite permission required: {}",
                dest_path.display()
            ),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error)
                    .context("Cannot inspect materialization destination");
            }
        }
    }

    match &entity.kind {
        FileEntityKind::Symlink { target } => {
            if options.strict_lossless
                && options.check_birthtime
                && !cfg!(windows)
                && entity
                    .metadata
                    .timestamps
                    .as_ref()
                    .and_then(|t| t.birthtime_sec)
                    .is_some()
            {
                anyhow::bail!(
                    "Cannot reproduce symlink birth time on this platform"
                );
            }
            let (target_path, mut adaptation_warnings) = target
                .to_path_buf_for_materialization_with_warnings(
                    options.strict_lossless,
                )?;
            let create = |path: &Path,
                          p_fd: &crate::sandboxable_dir::DirHandle,
                          f_name: &Path,
                          d_path: &Path| {
                #[cfg(windows)]
                if let Some(native) = &entity.metadata.native {
                    if native.source_os == OsFamily::Windows
                        && native.values.contains_key("reparse.data")
                    {
                        return dest_dir.create_native_reparse_at(
                            d_path,
                            path,
                            native,
                            options.symlink_policy,
                            options.existing_entry_policy,
                        );
                    }
                }
                dest_dir.create_symlink_at(
                    &p_fd.as_fd(),
                    f_name,
                    d_path,
                    path.as_os_str().as_encoded_bytes(),
                    options.symlink_policy,
                    options.existing_entry_policy,
                )
            };
            if let Err(error) =
                create(&target_path, &parent_dir_fd, &file_name, &dest_path)
            {
                if options.strict_lossless {
                    return Err(error);
                }
                let mut handled = false;
                if crate::link_path::is_destination_encoding_error(&error) {
                    let adapted = PathBuf::from(
                        target_path.to_string_lossy().into_owned(),
                    );
                    if adapted != target_path {
                        let warning = format!(
                            "Destination rejected symlink encoding {:?} -> {:?} on {}; retrying with replacement characters (best-effort): {error:#}",
                            target_path.display(),
                            adapted.display(),
                            dest_path.display()
                        );
                        warn_fmt!("{warning}");
                        create(
                            &adapted,
                            &parent_dir_fd,
                            &file_name,
                            &dest_path,
                        )?;
                        adaptation_warnings.push(warning);
                        handled = true;
                    }
                }
                if !handled && is_filename_rejection_error(&error) {
                    let (win_adapted, win_warnings) =
                        crate::clean_name::adapt_relative_path_for_materialization(orig_relative_path, true);
                    if win_adapted != relative_path_buf {
                        for w in &win_warnings {
                            warn_fmt!("{w}");
                        }
                        path_warnings.extend(win_warnings);
                        relative_path_buf = win_adapted;
                        relative_path = &relative_path_buf;
                        dest_path = resolve_and_validate_path(
                            dest_dir.root_path(),
                            relative_path,
                            options.path_policy,
                        )?;
                        let (new_p_fd, new_f_name) = dest_dir
                            .ensure_parent_dir(
                                relative_path,
                                options.path_policy,
                            )?;
                        parent_dir_fd = new_p_fd;
                        file_name = new_f_name;
                        create(
                            &target_path,
                            &parent_dir_fd,
                            &file_name,
                            &dest_path,
                        )?;
                        handled = true;
                    }
                }
                if !handled {
                    return Err(error);
                }
            }
            let stream_warnings =
                crate::streams::write_streams_from_source_with_warnings(
                    &dest_path,
                    None,
                    &entity.streams,
                    options.strict_lossless,
                    stream_source,
                )?;
            let mut warnings = path_warnings;
            warnings.extend(adaptation_warnings);
            warnings.extend(stream_warnings);
            warnings.extend(apply_entity_metadata(
                &dest_path,
                None,
                &entity.metadata,
                true,
                true,
                options.strict_lossless,
                options.check_birthtime,
            )?);

            Ok(MaterializeReceipt {
                destination_path: dest_path,
                bytes_written: 0,
                sha256: None,
                skipped_identical: false,
                warnings,
                stream_adaptations: Vec::new(),
            })
        }
        FileEntityKind::Hardlink {
            target_relative_path,
        } => {
            let (mut target_path, mut warnings) = target_relative_path
                .to_path_buf_for_materialization_with_warnings(
                    options.strict_lossless,
                )?;
            warnings.splice(0..0, path_warnings);
            if options.path_policy == PathTraversalPolicy::StrictSandboxed {
                for comp in target_path.components() {
                    match comp {
                        std::path::Component::Prefix(_)
                        | std::path::Component::RootDir
                        | std::path::Component::ParentDir => {
                            bail!(
                                "Hardlink target path rejected: target escapes destination root or contains root/parent components: {}",
                                target_path.display()
                            );
                        }
                        _ => {}
                    }
                }
            }
            if !options.strict_lossless {
                let (adapted_target, target_warnings) =
                    crate::clean_name::adapt_relative_path_for_materialization(
                        &target_path,
                        cfg!(windows),
                    );
                if adapted_target != target_path {
                    for w in &target_warnings {
                        warn_fmt!("{w}");
                    }
                    warnings.extend(target_warnings);
                    target_path = adapted_target;
                }
            }
            if let Err(err) = dest_dir.create_hardlink(
                &target_path,
                &parent_dir_fd.as_fd(),
                &file_name,
                options.existing_entry_policy,
            ) {
                if options.strict_lossless {
                    return Err(err);
                }
                let mut retried = false;
                let (win_target, win_target_warnings) =
                    crate::clean_name::adapt_relative_path_for_materialization(
                        &target_path,
                        true,
                    );
                if win_target != target_path {
                    for w in &win_target_warnings {
                        warn_fmt!("{w}");
                    }
                    warnings.extend(win_target_warnings);
                    target_path = win_target;
                    retried = true;
                }
                if is_filename_rejection_error(&err) {
                    let (win_adapted, win_warnings) =
                        crate::clean_name::adapt_relative_path_for_materialization(orig_relative_path, true);
                    if win_adapted != relative_path_buf {
                        for w in &win_warnings {
                            warn_fmt!("{w}");
                        }
                        warnings.extend(win_warnings);
                        relative_path_buf = win_adapted;
                        relative_path = &relative_path_buf;
                        dest_path = resolve_and_validate_path(
                            dest_dir.root_path(),
                            relative_path,
                            options.path_policy,
                        )?;
                        let (new_p_fd, new_f_name) = dest_dir
                            .ensure_parent_dir(
                                relative_path,
                                options.path_policy,
                            )?;
                        parent_dir_fd = new_p_fd;
                        file_name = new_f_name;
                        retried = true;
                    }
                }
                if retried {
                    dest_dir.create_hardlink(
                        &target_path,
                        &parent_dir_fd.as_fd(),
                        &file_name,
                        options.existing_entry_policy,
                    )?;
                } else {
                    return Err(err);
                }
            }
            #[cfg(windows)]
            warnings.extend(apply_windows_change_time(
                &dest_path,
                &entity.metadata,
                options.strict_lossless,
            )?);
            Ok(MaterializeReceipt {
                destination_path: dest_path,
                bytes_written: 0,
                sha256: None,
                skipped_identical: false,
                warnings,
                stream_adaptations: Vec::new(),
            })
        }
        FileEntityKind::Directory | FileEntityKind::Bundle { .. } => {
            if let Err(err) =
                dest_dir.ensure_dir_all(relative_path, options.path_policy)
            {
                if !options.strict_lossless && is_filename_rejection_error(&err)
                {
                    let (win_adapted, win_warnings) =
                        crate::clean_name::adapt_relative_path_for_materialization(orig_relative_path, true);
                    if win_adapted == relative_path_buf {
                        return Err(err);
                    } else {
                        for w in &win_warnings {
                            warn_fmt!("{w}");
                        }
                        path_warnings.extend(win_warnings);
                        relative_path_buf = win_adapted;
                        relative_path = &relative_path_buf;
                        dest_path = resolve_and_validate_path(
                            dest_dir.root_path(),
                            relative_path,
                            options.path_policy,
                        )?;
                        let _ = dest_dir.ensure_dir_all(
                            relative_path,
                            options.path_policy,
                        )?;
                    }
                } else {
                    return Err(err);
                }
            }
            dest_dir.authorize_metadata(
                relative_path,
                options.existing_entry_policy,
            )?;
            let (stream_adaptations, stream_warnings) =
                materialize_directory_streams_and_companions(
                    entity,
                    &dest_path,
                    dest_dir,
                    options,
                    stream_source,
                )?;
            let mut warnings = path_warnings;
            warnings.extend(stream_warnings);
            warnings.extend(apply_entity_metadata(
                &dest_path,
                None,
                &entity.metadata,
                false,
                true,
                options.strict_lossless,
                options.check_birthtime,
            )?);
            Ok(MaterializeReceipt {
                destination_path: dest_path,
                bytes_written: 0,
                sha256: None,
                skipped_identical: false,
                warnings,
                stream_adaptations,
            })
        }
        FileEntityKind::Fifo
        | FileEntityKind::CharDevice { .. }
        | FileEntityKind::BlockDevice { .. } => {
            if options.strict_lossless
                && options.check_birthtime
                && !cfg!(windows)
                && entity
                    .metadata
                    .timestamps
                    .as_ref()
                    .and_then(|t| t.birthtime_sec)
                    .is_some()
            {
                anyhow::bail!(
                    "Cannot reproduce special-node birth time on this platform"
                );
            }
            anyhow::ensure!(
                options.copy_specials,
                "Special node creation rejected: {}. Enable copy_specials to permit.",
                dest_path.display()
            );
            // Reason for fallback: absent file mode for special node defaults to standard 0o666 permissions
            let mode = entity.metadata.mode.unwrap_or(0o666);
            if let Err(err) = dest_dir.create_special(
                &parent_dir_fd.as_fd(),
                &file_name,
                &entity.kind,
                mode,
                options.existing_entry_policy,
            ) {
                if !options.strict_lossless && is_filename_rejection_error(&err)
                {
                    let (win_adapted, win_warnings) =
                        crate::clean_name::adapt_relative_path_for_materialization(orig_relative_path, true);
                    if win_adapted == relative_path_buf {
                        return Err(err);
                    } else {
                        for w in &win_warnings {
                            warn_fmt!("{w}");
                        }
                        path_warnings.extend(win_warnings);
                        relative_path_buf = win_adapted;
                        relative_path = &relative_path_buf;
                        dest_path = resolve_and_validate_path(
                            dest_dir.root_path(),
                            relative_path,
                            options.path_policy,
                        )?;
                        let (new_p_fd, new_f_name) = dest_dir
                            .ensure_parent_dir(
                                relative_path,
                                options.path_policy,
                            )?;
                        parent_dir_fd = new_p_fd;
                        file_name = new_f_name;
                        dest_dir.create_special(
                            &parent_dir_fd.as_fd(),
                            &file_name,
                            &entity.kind,
                            mode,
                            options.existing_entry_policy,
                        )?;
                    }
                } else {
                    return Err(err);
                }
            }
            let stream_warnings =
                crate::streams::write_streams_from_source_with_warnings(
                    &dest_path,
                    None,
                    &entity.streams,
                    options.strict_lossless,
                    stream_source,
                )?;
            let mut warnings = path_warnings;
            warnings.extend(stream_warnings);
            warnings.extend(apply_entity_metadata(
                &dest_path,
                None,
                &entity.metadata,
                false,
                true,
                options.strict_lossless,
                options.check_birthtime,
            )?);
            Ok(MaterializeReceipt {
                destination_path: dest_path,
                bytes_written: 0,
                sha256: None,
                skipped_identical: false,
                warnings,
                stream_adaptations: Vec::new(),
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
            if options.existing_entry_policy == ExistingEntryPolicy::Replace
                && !options.force_overwrite
            {
                if let Some(mut receipt) = try_update_existing_regular_entity(
                    &dest_path,
                    dest_dir,
                    entity,
                    *size,
                    expected_sha256,
                    *is_sparse,
                    options,
                    stream_source,
                )? {
                    receipt.warnings.splice(0..0, path_warnings);
                    return Ok(receipt);
                }
            }

            let is_apple_single = options.apple_write_mode
                == AppleWriteMode::ForceAppleSingle
                || (options.apple_write_mode
                    == AppleWriteMode::MaybeAppleSingle
                    && (!entity.streams.is_empty()
                        // Reason for fallback: absent AppleMetadata defaults to false (no metadata to preserve)
                        || entity.metadata.apple.as_ref().is_some_and(|a| !a.is_empty())));

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
                source.seek(SeekFrom::Start(0))?;
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
                anyhow::ensure!(
                    read_bytes == initial_size,
                    "Unexpected EOF in AppleSingle payload for {}",
                    dest_path.display()
                );
                let mut extra = [0_u8; 1];
                anyhow::ensure!(
                    source.read(&mut extra)? == 0,
                    "Payload grew during AppleSingle copy of {}",
                    dest_path.display()
                );
                let mut payload_hasher = Sha256Stream::new();
                payload_hasher.update(&data_fork);
                let payload_sha256 = payload_hasher.finalize();
                anyhow::ensure!(
                    *expected_sha256 == [0_u8; 32]
                        || payload_sha256 == *expected_sha256,
                    "Payload SHA-256 verification failed during AppleSingle write for {}",
                    dest_path.display()
                );
                let source_warnings = source.finish()?;
                anyhow::ensure!(
                    !options.strict_lossless || source_warnings.is_empty(),
                    "Payload source fidelity failed for {}: {:?}",
                    dest_path.display(),
                    source_warnings
                );
                let archive =
                    create_apple_archive_from_entity_with_stream_source(
                        entity,
                        AppleFormat::AppleSingle,
                        Some(data_fork),
                        stream_source,
                    )?;
                let single_bytes = write_apple_single_double(&archive)?;
                let mut hasher = Sha256Stream::new();
                hasher.update(&single_bytes);
                let single_sha256 = hasher.finalize();
                let single_size = u64::try_from(single_bytes.len())?;

                let pid = std::process::id();
                let nanos = match std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                {
                    Ok(dur) => dur.subsec_nanos(),
                    Err(_) => 0,
                };
                let seq = ATOMIC_SEQ
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
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
                let mut warnings = path_warnings;
                warnings.extend(apply_entity_metadata(
                    &temp_path,
                    Some(&dest_path),
                    &entity.metadata,
                    false,
                    false,
                    options.strict_lossless,
                    options.check_birthtime,
                )?);
                warnings.extend(source_warnings);

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
                if actual_dest_path != dest_path {
                    log_fmt!(
                        "Materializing AppleSingle file with extension {:?} -> {:?}",
                        dest_path.display(),
                        actual_dest_path.display()
                    );
                }

                dest_dir.commit_atomic_file(
                    &parent_dir_fd.as_fd(),
                    &temp_name,
                    &actual_file_name,
                    options.existing_entry_policy,
                )?;
                cleanup_guard.active = false;

                #[cfg(windows)]
                apply_windows_birthtime(&actual_dest_path, &entity.metadata)?;

                #[cfg(unix)]
                sync_parent_dir_best_effort(&parent_dir_fd, parent_dir);

                let has_flags = entity.metadata.flags.is_some()
                    || entity.metadata.platform_raw_flags.is_some();
                if has_flags {
                    // Reason for fallback: absent flags metadata defaults to empty slice
                    let flags_slice =
                        entity.metadata.flags.as_deref().unwrap_or(&[]);
                    // Reason for fallback: absent environment metadata defaults to compile-target OS family
                    let source_os = entity
                        .metadata
                        .environment
                        .as_ref()
                        .map_or(OsFamily::CURRENT, |env| env.os_family());
                    apply_file_flags_with_source(
                        &actual_dest_path,
                        flags_slice,
                        entity.metadata.platform_raw_flags.as_ref(),
                        source_os,
                        options.strict_lossless,
                    )?;
                }
                #[cfg(windows)]
                warnings.extend(apply_windows_change_time(
                    &actual_dest_path,
                    &entity.metadata,
                    options.strict_lossless,
                )?);
                warnings.extend(crate::metadata::check_metadata_replication(
                    &actual_dest_path,
                    &entity.metadata,
                    options.strict_lossless,
                    false,
                    options.check_birthtime,
                )?);

                return Ok(MaterializeReceipt {
                    destination_path: actual_dest_path,
                    bytes_written: single_size,
                    sha256: Some(single_sha256),
                    skipped_identical: false,
                    warnings,
                    stream_adaptations: Vec::new(),
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
            let nanos = match std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
            {
                Ok(dur) => dur.subsec_nanos(),
                Err(_) => 0,
            };
            let seq =
                ATOMIC_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
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
            let mut sparse_warnings = Vec::new();
            if can_use_sparse {
                let mut extent_end = 0_u64;
                for extent in extents {
                    let (offset, length) = match extent {
                        Extent::Data { offset, length }
                        | Extent::Hole { offset, length } => (*offset, *length),
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
                    let warning = format!(
                        "Sparse extents layout does not strictly cover payload for {}; falling back to linear stream copying without preserving sparse layout",
                        dest_path.display()
                    );
                    warn_fmt!("Caveat: {warning}");
                    sparse_warnings.push(warning);
                }
            }

            #[cfg(windows)]
            if can_use_sparse {
                use std::os::windows::fs::MetadataExt;
                use std::os::windows::io::AsRawHandle;
                use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_SPARSE_FILE;
                use windows_sys::Win32::System::IO::DeviceIoControl;
                use windows_sys::Win32::System::Ioctl::FSCTL_SET_SPARSE;

                let mut returned = 0_u32;
                // SAFETY: temp_file is a valid live file handle
                #[expect(
                    unsafe_code,
                    reason = "DeviceIoControl requires Win32 FFI with a live file handle and synchronous output storage"
                )]
                let ret = unsafe {
                    DeviceIoControl(
                        temp_file.as_raw_handle(),
                        FSCTL_SET_SPARSE,
                        std::ptr::null(),
                        0,
                        std::ptr::null_mut(),
                        0,
                        &mut returned,
                        std::ptr::null_mut(),
                    )
                };
                let has_sparse_attr = temp_file
                    .metadata()
                    .context("Failed to query destination sparse attributes")?
                    .file_attributes()
                    & FILE_ATTRIBUTE_SPARSE_FILE
                    != 0;
                if ret == 0 || !has_sparse_attr {
                    if options.strict_lossless {
                        anyhow::bail!(
                            "Destination filesystem does not support sparse files for {}",
                            dest_path.display()
                        );
                    }
                    let msg = format!(
                        "Destination filesystem did not retain sparse support for {}",
                        dest_path.display()
                    );
                    warn_fmt!("Caveat: {msg}");
                    sparse_warnings.push(msg);
                    can_use_sparse = false;
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
                                anyhow::ensure!(
                                    n != 0,
                                    "Unexpected EOF in sparse payload for {}",
                                    dest_path.display()
                                );
                                let write_slice = buf
                                    .get(..n)
                                    .context("Buffer slice index out of bounds for write")?;
                                temp_file.write_all(write_slice)?;
                                hasher.update(write_slice);
                                let n_u64 = u64::try_from(n).context(
                                    "Failed to convert read bytes count to u64",
                                )?;
                                remaining = remaining.saturating_sub(n_u64);
                            }
                        }
                        Extent::Hole { length, .. } => {
                            let zero_buf = [0_u8; 8 * 1024];
                            let mut remaining = *length;
                            while remaining > 0 {
                                let chunk = usize::try_from(remaining.min(8 * 1024))
                                    .context("Failed to convert hole chunk size to usize")?;
                                let zero_slice =
                                    zero_buf.get(..chunk).context(
                                        "Zero buffer slice index out of bounds",
                                    )?;
                                hasher.update(zero_slice);
                                let chunk_u64 = u64::try_from(chunk).context(
                                    "Failed to convert hole chunk size to u64",
                                )?;
                                remaining = remaining.saturating_sub(chunk_u64);
                            }
                        }
                    }
                }

                #[cfg(windows)]
                {
                    use std::os::windows::io::AsRawHandle;
                    use windows_sys::Win32::System::IO::DeviceIoControl;
                    use windows_sys::Win32::System::Ioctl::{
                        FILE_ZERO_DATA_INFORMATION, FSCTL_SET_ZERO_DATA,
                    };

                    // Crucial NTFS invariant: flush buffered writes before punching holes,
                    // otherwise deferred dirty cache page writeback on handle close or
                    // sync will reallocate the zeroed extent.
                    temp_file.sync_all()?;

                    for extent in extents {
                        if let Extent::Hole { offset, length } = extent {
                            let start = i64::try_from(*offset)
                                .context("Hole offset out of range for i64")?;
                            let end_u64 = offset
                                .checked_add(*length)
                                .context("Hole end offset overflow")?;
                            let end = i64::try_from(end_u64).context(
                                "Hole end offset out of range for i64",
                            )?;
                            let zero_info = FILE_ZERO_DATA_INFORMATION {
                                FileOffset: start,
                                BeyondFinalZero: end,
                            };
                            let mut returned = 0_u32;
                            // SAFETY: temp_file is a live handle; zero_info is a stack-allocated struct.
                            #[expect(
                                unsafe_code,
                                reason = "DeviceIoControl requires Win32 FFI with live synchronous input and output buffers"
                            )]
                            let ok = unsafe {
                                DeviceIoControl(
                                    temp_file.as_raw_handle(),
                                    FSCTL_SET_ZERO_DATA,
                                    (&raw const zero_info).cast(),
                                    u32::try_from(std::mem::size_of_val(
                                        &zero_info,
                                    ))?,
                                    std::ptr::null_mut(),
                                    0,
                                    &mut returned,
                                    std::ptr::null_mut(),
                                )
                            };
                            if ok == 0 {
                                let err = std::io::Error::last_os_error();
                                if options.strict_lossless {
                                    return Err(err).context("Failed to punch sparse hole on Windows");
                                }
                                let msg = format!(
                                    "Failed to punch sparse hole at offset {offset}: {err}"
                                );
                                warn_fmt!("Caveat: {msg}");
                                sparse_warnings.push(msg);
                            } else if *length < 65_536 {
                                let msg = format!(
                                    "Windows NTFS sparse files allocate at 64KB granularity; sub-64KB hole at offset {offset} (length {length}) cannot be physically deallocated on restoration"
                                );
                                warn_fmt!("Caveat: {msg}");
                                sparse_warnings.push(msg);
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
                    written = written
                        .checked_add(u64::try_from(n)?)
                        .context("Payload size overflow")?;
                    anyhow::ensure!(
                        written <= initial_size,
                        "Payload grew during copy of {}",
                        dest_path.display()
                    );
                    let slice = buf.get(..n).context(
                        "Buffer slice index out of bounds for write",
                    )?;
                    temp_file.write_all(slice)?;
                    hasher.update(slice);
                }
                anyhow::ensure!(
                    written == initial_size,
                    "Unexpected EOF in payload for {}",
                    dest_path.display()
                );
            }

            let source_warnings = source.finish()?;
            anyhow::ensure!(
                !options.strict_lossless || source_warnings.is_empty(),
                "Payload source fidelity failed for {}: {:?}",
                dest_path.display(),
                source_warnings
            );
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
                Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
                _ => PathBuf::from("."),
            };
            let temp_path = parent_dir.join(&temp_name);

            let mut write_companion = false;
            let mut write_windows_ads = false;
            let mut stream_warnings = Vec::new();
            match options.apple_write_mode {
                AppleWriteMode::ForceAppleDouble(_) => {
                    write_companion = true;
                }
                AppleWriteMode::MaybeAppleDouble(_) => {
                    // Reason for fallback: absent AppleMetadata defaults to false (no metadata to preserve)
                    let has_apple_meta = entity
                        .metadata
                        .apple
                        .as_ref()
                        .is_some_and(|a| !a.is_empty());
                    let streams_res =
                        crate::streams::write_streams_from_source_with_warnings(
                            &temp_path,
                            Some(&dest_path),
                            &entity.streams,
                            false,
                            stream_source,
                        );
                    match streams_res {
                        Ok(w) => {
                            stream_warnings.extend(w);
                            if has_apple_meta {
                                write_companion = true;
                            }
                        }
                        Err(err) => {
                            warn_fmt!(
                                "Native stream write failed on {}; falling back to AppleDouble companion: {err:#}",
                                dest_path.display()
                            );
                            write_companion = true;
                        }
                    }
                }
                AppleWriteMode::ForceWindowsAds => {
                    write_windows_ads = !entity.streams.is_empty();
                    // Reason for fallback: absent AppleMetadata defaults to false (no metadata to preserve)
                    if options.strict_lossless
                        && entity
                            .metadata
                            .apple
                            .as_ref()
                            .is_some_and(|a| !a.is_empty())
                    {
                        anyhow::bail!(
                            "Cannot preserve Apple metadata for {} with Windows ADS companions",
                            dest_path.display()
                        );
                    }
                }
                AppleWriteMode::MaybeWindowsAds => {
                    let streams_res =
                        crate::streams::write_streams_from_source_with_warnings(
                            &temp_path,
                            Some(&dest_path),
                            &entity.streams,
                            true,
                            stream_source,
                        );
                    match streams_res {
                        Ok(w) => {
                            stream_warnings.extend(w);
                        }
                        Err(err) => {
                            warn_fmt!(
                                "Native stream write failed on {}; falling back to Windows ADS companions: {err:#}",
                                dest_path.display()
                            );
                            write_windows_ads = !entity.streams.is_empty();
                        }
                    }
                    // Reason for fallback: absent AppleMetadata defaults to false (no metadata to preserve)
                    if options.strict_lossless
                        && entity
                            .metadata
                            .apple
                            .as_ref()
                            .is_some_and(|a| !a.is_empty())
                    {
                        anyhow::bail!(
                            "Cannot preserve Apple metadata for {} with Windows ADS companions",
                            dest_path.display()
                        );
                    }
                }
                _ => {
                    stream_warnings.extend(crate::streams::write_streams_from_source_with_warnings(
                        &temp_path,
                        Some(&dest_path),
                        &entity.streams,
                        options.strict_lossless,
                        stream_source,
                    )?);
                    // Reason for fallback: absent AppleMetadata defaults to false (no metadata to preserve)
                    if options.strict_lossless
                        && entity
                            .metadata
                            .apple
                            .as_ref()
                            .is_some_and(|a| !a.is_empty())
                    {
                        anyhow::bail!(
                            "Cannot preserve Apple metadata for {} natively without AppleDouble or AppleSingle",
                            dest_path.display()
                        );
                    }
                }
            }

            let prebuilt_apple_double = if write_companion {
                if let Some(style) = options.apple_write_mode.double_style() {
                    if let Some(companion) = apple_double_companion_path(
                        entity, &dest_path, dest_dir, style,
                    )? {
                        if options.existing_entry_policy
                            == ExistingEntryPolicy::Refuse
                        {
                            match std::fs::symlink_metadata(&companion) {
                                Ok(_) => bail!("AppleDouble companion already exists; explicit overwrite permission required: {}", companion.display()),
                                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {},
                                Err(error) => return Err(error).context("Cannot inspect AppleDouble companion destination"),
                            }
                        }
                        Some((
                            style,
                            serialize_apple_double_for_entity_with_stream_source(
                                entity,
                                stream_source,
                            )?,
                        ))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };
            if write_windows_ads
                && options.existing_entry_policy == ExistingEntryPolicy::Refuse
            {
                for planned in plan_windows_ads_companions(
                    entity,
                    &dest_path,
                    dest_dir,
                    options.strict_lossless,
                )? {
                    if options.strict_lossless || planned.adaptation.is_none() {
                        match std::fs::symlink_metadata(&planned.companion_path) {
                            Ok(_) => bail!(
                                "Windows ADS companion already exists; explicit overwrite permission required: {}",
                                planned.companion_path.display()
                            ),
                            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                            Err(error) => return Err(error).context("Cannot inspect Windows ADS companion destination"),
                        }
                    }
                }
            }

            // Apply ownership, permissions, and timestamps to temp file (defer flags until after rename)
            #[cfg(windows)]
            let temp_file = {
                temp_file.sync_all()?;
                drop(temp_file);
                std::fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(&temp_path)?
            };
            let mut warnings = path_warnings;
            warnings.extend(stream_warnings);
            warnings.extend(apply_entity_metadata(
                &temp_path,
                Some(&dest_path),
                &entity.metadata,
                false,
                false,
                options.strict_lossless,
                options.check_birthtime,
            )?);
            warnings.extend(sparse_warnings);
            warnings.extend(source_warnings);

            let audited_entity = if options.strict_lossless {
                let mut expected = entity.clone();
                if let FileEntityKind::Regular {
                    sha256, is_sparse, ..
                } = &mut expected.kind
                {
                    *sha256 = computed_sha256;
                    if !can_use_sparse {
                        *is_sparse = false;
                    }
                }
                let audit_options = EntityAuditOptions {
                    ignore_flags: true,
                    ignore_atime: !payload_noatime,
                    check_birthtime: options.check_birthtime,
                    ignore_xattrs: write_companion || write_windows_ads,
                    ..EntityAuditOptions::default()
                };
                let (differences, _) = audit_entity_detailed(
                    &temp_path,
                    &expected,
                    &audit_options,
                )?;
                anyhow::ensure!(
                    differences.is_empty(),
                    "Staged file failed fidelity verification for {}: {:?}",
                    dest_path.display(),
                    differences
                );
                Some(expected)
            } else {
                None
            };

            // Sync temp file and parent directory
            temp_file.sync_all()?;
            drop(temp_file);

            #[cfg(unix)]
            sync_parent_dir_best_effort(&parent_dir_fd, &parent_dir);

            // Atomic rename inside parent directory
            if let Err(err) = dest_dir.commit_atomic_file(
                &parent_dir_fd.as_fd(),
                &temp_name,
                &file_name,
                options.existing_entry_policy,
            ) {
                if !options.strict_lossless && is_filename_rejection_error(&err)
                {
                    if let Some(orig_name) = file_name.file_name() {
                        let (mut adapted_name, reason) =
                            crate::clean_name::adapt_component_with_max_bytes(
                                orig_name,
                                true,
                                max_leaf_bytes,
                            );
                        if adapted_name == orig_name {
                            return Err(err);
                        } else {
                            if options.existing_entry_policy
                                == ExistingEntryPolicy::Refuse
                                && std::fs::symlink_metadata(
                                    parent_dir.join(&adapted_name),
                                )
                                .is_ok()
                            {
                                let base_str = adapted_name.to_string_lossy();
                                let mut collision_idx = 1_usize;
                                loop {
                                    let disambig = crate::clean_name::disambiguate_cleaned_name(
                                        &base_str,
                                        orig_name.as_encoded_bytes(),
                                        collision_idx,
                                        max_leaf_bytes,
                                    );
                                    let cand_os =
                                        std::ffi::OsString::from(&disambig);
                                    if std::fs::symlink_metadata(
                                        parent_dir.join(&cand_os),
                                    )
                                    .is_err()
                                    {
                                        adapted_name = cand_os;
                                        break;
                                    }
                                    collision_idx =
                                        collision_idx.saturating_add(1);
                                }
                            }
                            let new_rel = match relative_path.parent() {
                                Some(p) if !p.as_os_str().is_empty() => {
                                    p.join(&adapted_name)
                                }
                                _ => PathBuf::from(&adapted_name),
                            };
                            let msg = format!(
                                "Adapted destination path {:?} -> {:?} (best-effort: {})",
                                orig_relative_path.display(),
                                new_rel.display(),
                                reason.as_deref().unwrap_or(
                                    "rejected by destination filesystem"
                                ) // Reason for fallback: default description when filesystem rejects component
                            );
                            warn_fmt!("{msg}");
                            warnings.push(msg);
                            file_name = PathBuf::from(&adapted_name);
                            dest_path = parent_dir.join(&adapted_name);
                            dest_dir.commit_atomic_file(
                                &parent_dir_fd.as_fd(),
                                &temp_name,
                                &file_name,
                                options.existing_entry_policy,
                            )?;
                        }
                    } else {
                        return Err(err);
                    }
                } else {
                    return Err(err);
                }
            }
            cleanup_guard.active = false;

            #[cfg(windows)]
            apply_windows_birthtime(&dest_path, &entity.metadata)?;

            if let Some((style, ref apple_double_bytes)) = prebuilt_apple_double
            {
                write_apple_double_companion_bytes(
                    entity,
                    &dest_path,
                    dest_dir,
                    style,
                    options.existing_entry_policy,
                    apple_double_bytes,
                )?;
            }
            let mut stream_adaptations = Vec::new();
            if write_windows_ads {
                let (_paths, adapts, ads_warnings) =
                    write_windows_ads_companions_with_warnings(
                        entity,
                        &dest_path,
                        dest_dir,
                        options.existing_entry_policy,
                        options.strict_lossless,
                        stream_source,
                    )?;
                stream_adaptations = adapts;
                warnings.extend(ads_warnings);
            }

            #[cfg(unix)]
            sync_parent_dir_best_effort(&parent_dir_fd, &parent_dir);

            // Deferred immutability: apply flags after publication, before restoring Windows change time.
            let has_flags = entity.metadata.flags.is_some()
                || entity.metadata.platform_raw_flags.is_some();
            if has_flags {
                let flags_slice =
                    entity.metadata.flags.as_deref().unwrap_or(&[]); // Reason for fallback: absent flags metadata defaults to empty slice
                let source_os = entity
                    .metadata
                    .environment
                    .as_ref()
                    .map_or(OsFamily::CURRENT, |env| env.os_family());
                apply_file_flags_with_source(
                    &dest_path,
                    flags_slice,
                    entity.metadata.platform_raw_flags.as_ref(),
                    source_os,
                    options.strict_lossless,
                )?;
            }
            #[cfg(windows)]
            warnings.extend(apply_windows_change_time(
                &dest_path,
                &entity.metadata,
                options.strict_lossless,
            )?);
            warnings.extend(crate::metadata::check_metadata_replication(
                &dest_path,
                &entity.metadata,
                options.strict_lossless,
                false,
                options.check_birthtime,
            )?);
            if write_companion
                || write_windows_ads
                || options.check_atime
                || options.check_ctime
                || options.check_birthtime
            {
                if let Some(expected) = audited_entity.as_ref() {
                    crate::verifier::verify_materialized_entity_ext(
                        &dest_path,
                        expected,
                        true,
                        options.check_atime,
                        options.check_ctime,
                        options.check_birthtime,
                    )?;
                }
            }

            Ok(MaterializeReceipt {
                destination_path: dest_path,
                bytes_written: initial_size,
                sha256: Some(computed_sha256),
                skipped_identical: false,
                warnings,
                stream_adaptations,
            })
        }
        FileEntityKind::Inaccessible => {
            anyhow::bail!(
                "Cannot materialize inaccessible entity: {}",
                dest_path.display()
            );
        }
    }
}

fn is_filename_rejection_error(error: &anyhow::Error) -> bool {
    if crate::link_path::is_destination_encoding_error(error) {
        return true;
    }
    #[cfg(unix)]
    if error.chain().any(|cause| {
        matches!(
            cause.downcast_ref::<rustix::io::Errno>(),
            Some(
                &rustix::io::Errno::NAMETOOLONG
                    | &rustix::io::Errno::INVAL
                    | &rustix::io::Errno::ILSEQ
            )
        )
    }) {
        return true;
    }
    error.chain().any(|cause| {
        let Some(io_err) = cause.downcast_ref::<std::io::Error>() else {
            return false;
        };
        let Some(raw) = io_err.raw_os_error() else {
            return false;
        };
        #[cfg(unix)]
        {
            raw == nix::libc::ENAMETOOLONG
                || raw == nix::libc::EINVAL
                || raw == nix::libc::EILSEQ
        }
        #[cfg(windows)]
        {
            // ERROR_INVALID_NAME (123), ERROR_FILENAME_EXCED_RANGE (206), ERROR_NO_UNICODE_TRANSLATION (1113)
            raw == 123 || raw == 206 || raw == 1113
        }
        #[cfg(not(any(unix, windows)))]
        {
            let _ = raw;
            false
        }
    })
}

#[cfg(unix)]
fn sync_parent_dir_best_effort(
    parent_fd: &impl rustix::fd::AsFd,
    parent_path: &Path,
) {
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
            log_fmt!(
                "Directory fsync note on {}: {err}",
                parent_path.display()
            );
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
            let _ = rustix::fs::unlinkat(
                self.parent_fd,
                &self.temp_name,
                AtFlags::empty(),
            );
            #[cfg(not(unix))]
            let _ =
                std::fs::remove_file(self.parent_fd.path.join(&self.temp_name));
        }
    }
}

/// Atomically replaces a file after syncing its contents and publication.
pub fn write_file_durable(path: &Path, contents: &[u8]) -> Result<()> {
    let parent = match path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        Some(parent) => parent,
        None => Path::new("."),
    };
    let mut staged = tempfile::NamedTempFile::new_in(parent)?;
    std::io::Write::write_all(&mut staged, contents)?;
    staged.as_file().sync_all()?;
    #[cfg(windows)]
    {
        use windows_sys::Win32::Storage::FileSystem::{
            FILE_ATTRIBUTE_NORMAL, MOVEFILE_REPLACE_EXISTING,
            MOVEFILE_WRITE_THROUGH, MoveFileExW, SetFileAttributesW,
        };
        let source = crate::metadata::windows::path_to_wide(staged.path())?;
        let destination = crate::metadata::windows::path_to_wide(path)?;
        #[expect(
            unsafe_code,
            reason = "Both native paths are live NUL-terminated UTF-16 buffers"
        )]
        // SAFETY: Both path buffers remain live and NUL-terminated for these calls.
        unsafe {
            if SetFileAttributesW(source.as_ptr(), FILE_ATTRIBUTE_NORMAL) == 0 {
                return Err(std::io::Error::last_os_error())
                    .context("Failed to prepare durable replacement");
            }
            if MoveFileExW(
                source.as_ptr(),
                destination.as_ptr(),
                MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
            ) == 0
            {
                return Err(std::io::Error::last_os_error())
                    .context("Failed to publish durable replacement");
            }
        }
    }
    #[cfg(unix)]
    {
        let directory = std::fs::File::open(parent)?;
        staged.persist(path)?;
        directory.sync_all()?;
    }
    #[cfg(not(any(unix, windows)))]
    anyhow::bail!("Durable file replacement is unsupported on this platform");
    Ok(())
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
    for entry in
        crate::traversal::read_dir_raw(dir_path).with_context(|| {
            format!("Failed to read directory: {}", dir_path.display())
        })?
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

/// Materializes streams and optional AppleDouble or Windows ADS companion files
/// for a directory entity at `dest_path`, returning `(stream_adaptations, warnings)`.
pub fn materialize_directory_streams_and_companions(
    entity: &FileEntity,
    dest_path: &Path,
    dest_dir: &SandboxableDir,
    options: &MaterializeOptions,
    stream_source: &dyn crate::streams::AttachedStreamSource,
) -> Result<(Vec<StreamAdaptationRecord>, Vec<String>)> {
    let mut write_companion = false;
    let mut write_windows_ads = false;
    let mut stream_warnings = Vec::new();
    match options.apple_write_mode {
        AppleWriteMode::ForceAppleDouble(_) => {
            write_companion = true;
        }
        AppleWriteMode::MaybeAppleDouble(_) => {
            // Reason for fallback: absent AppleMetadata defaults to false (no metadata to preserve)
            let has_apple_meta = entity
                .metadata
                .apple
                .as_ref()
                .is_some_and(|a| !a.is_empty());
            let res = crate::streams::write_streams_from_source_with_warnings(
                dest_path,
                None,
                &entity.streams,
                false,
                stream_source,
            );
            match res {
                Ok(w) => {
                    stream_warnings.extend(w);
                    if has_apple_meta {
                        write_companion = true;
                    }
                }
                Err(err) => {
                    warn_fmt!(
                        "Native stream write failed on directory {}; falling back to AppleDouble companion: {err:#}",
                        dest_path.display()
                    );
                    write_companion = true;
                }
            }
        }
        AppleWriteMode::ForceWindowsAds => {
            write_windows_ads = !entity.streams.is_empty();
            // Reason for fallback: absent AppleMetadata defaults to false (no metadata to preserve)
            if options.strict_lossless
                && entity
                    .metadata
                    .apple
                    .as_ref()
                    .is_some_and(|a| !a.is_empty())
            {
                anyhow::bail!(
                    "Cannot preserve Apple metadata for directory {} with Windows ADS companions",
                    dest_path.display()
                );
            }
        }
        AppleWriteMode::MaybeWindowsAds => {
            let res = crate::streams::write_streams_from_source_with_warnings(
                dest_path,
                None,
                &entity.streams,
                true,
                stream_source,
            );
            match res {
                Ok(w) => {
                    stream_warnings.extend(w);
                }
                Err(err) => {
                    warn_fmt!(
                        "Native stream write failed on directory {}; falling back to Windows ADS companions: {err:#}",
                        dest_path.display()
                    );
                    write_windows_ads = !entity.streams.is_empty();
                }
            }
            // Reason for fallback: absent AppleMetadata defaults to false (no metadata to preserve)
            if options.strict_lossless
                && entity
                    .metadata
                    .apple
                    .as_ref()
                    .is_some_and(|a| !a.is_empty())
            {
                anyhow::bail!(
                    "Cannot preserve Apple metadata for directory {} with Windows ADS companions",
                    dest_path.display()
                );
            }
        }
        _ => {
            stream_warnings.extend(
                crate::streams::write_streams_from_source_with_warnings(
                    dest_path,
                    None,
                    &entity.streams,
                    options.strict_lossless,
                    stream_source,
                )?,
            );
            // Reason for fallback: absent AppleMetadata defaults to false (no metadata to preserve)
            if options.strict_lossless
                && entity
                    .metadata
                    .apple
                    .as_ref()
                    .is_some_and(|a| !a.is_empty())
            {
                anyhow::bail!(
                    "Cannot preserve Apple metadata for directory {} natively without AppleDouble or AppleSingle",
                    dest_path.display()
                );
            }
        }
    }

    let parent_sandbox = if (write_companion || write_windows_ads)
        && dest_path == dest_dir.root_path()
    {
        match dest_path.parent().filter(|p| !p.as_os_str().is_empty()) {
            Some(p) => Some(SandboxableDir::open(p)?),
            None => Some(SandboxableDir::open(".")?),
        }
    } else {
        None
    };
    let companion_sandbox = parent_sandbox.as_ref().unwrap_or(dest_dir); // Reason for fallback: non-root directory uses dest_dir directly

    if write_companion {
        if let Some(style) = options.apple_write_mode.double_style() {
            write_apple_double_companion_with_stream_source(
                entity,
                dest_path,
                companion_sandbox,
                style,
                options.existing_entry_policy,
                stream_source,
            )?;
        }
    }
    let mut stream_adaptations = Vec::new();
    if write_windows_ads {
        let (_paths, adapts, ads_warnings) =
            write_windows_ads_companions_with_warnings(
                entity,
                dest_path,
                companion_sandbox,
                options.existing_entry_policy,
                options.strict_lossless,
                stream_source,
            )?;
        stream_adaptations = adapts;
        stream_warnings.extend(ads_warnings);
    }
    Ok((stream_adaptations, stream_warnings))
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
#[expect(
    clippy::too_many_lines,
    reason = "Comprehensive in-place entity validation and lossless metadata reconciliation"
)]
fn try_update_existing_regular_entity(
    dest_path: &Path,
    dest_dir: &SandboxableDir,
    entity: &FileEntity,
    size: u64,
    expected_sha256: &[u8; 32],
    is_sparse: bool,
    options: &MaterializeOptions,
    stream_source: &dyn crate::streams::AttachedStreamSource,
) -> Result<Option<MaterializeReceipt>> {
    let dest_meta = match std::fs::symlink_metadata(dest_path) {
        Ok(m) => m,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Ok(None);
        }
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
    if dest_meta.nlink() > 1 {
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

    let source_names = entity
        .streams
        .iter()
        .map(|stream| {
            stream
                .name
                .as_ref()
                .map(|name| {
                    name.canonicalize_native_for_materialization(
                        options.strict_lossless,
                    )
                })
                .transpose()
        })
        .collect::<Result<Vec<_>>>()?;
    let disk_names = on_disk_streams
        .iter()
        .map(|stream| {
            stream
                .name
                .as_ref()
                .map(crate::streams::StreamName::canonicalize_native)
                .transpose()
        })
        .collect::<Result<Vec<_>>>()?;

    // Remove any extra streams that do not exist on the source entity
    for (disk_stream, disk_name) in on_disk_streams.iter().zip(&disk_names) {
        let exists_in_source =
            entity
                .streams
                .iter()
                .zip(&source_names)
                .any(|(s, source_name)| match (source_name, disk_name) {
                    (Some(s_name), Some(d_name)) => s_name == d_name,
                    (None, None) => s.kind == disk_stream.kind,
                    _ => false,
                });
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

    let mut stream_adaptations = Vec::new();
    let mut stream_warnings = Vec::new();
    if let Some(style) = options.apple_write_mode.double_style() {
        write_apple_double_companion_with_stream_source(
            entity,
            dest_path,
            dest_dir,
            style,
            options.existing_entry_policy,
            stream_source,
        )?;
    } else if options.apple_write_mode.is_windows_ads() {
        let (_paths, adapts, ads_warnings) =
            write_windows_ads_companions_with_warnings(
                entity,
                dest_path,
                dest_dir,
                options.existing_entry_policy,
                options.strict_lossless,
                stream_source,
            )?;
        stream_adaptations = adapts;
        stream_warnings.extend(ads_warnings);
    } else {
        // Write missing or mismatched streams
        for (stream, source_name) in entity.streams.iter().zip(&source_names) {
            let matches_on_disk = on_disk_streams.iter().zip(&disk_names).any(
                |(d, disk_name)| {
                    let names_match = match (disk_name, source_name) {
                        (Some(d_name), Some(s_name)) => d_name == s_name,
                        (None, None) => d.kind == stream.kind,
                        _ => false,
                    };
                    names_match
                        && match (&d.entity.kind, &stream.entity.kind) {
                            (
                                FileEntityKind::Regular {
                                    sha256: d_hash, ..
                                },
                                FileEntityKind::Regular {
                                    sha256: s_hash, ..
                                },
                            ) => d_hash == s_hash,
                            _ => false,
                        }
                },
            );

            if !matches_on_disk {
                stream_warnings.extend(
                    crate::streams::write_streams_from_source_with_warnings(
                        dest_path,
                        Some(dest_path),
                        std::slice::from_ref(stream),
                        options.strict_lossless,
                        stream_source,
                    )?,
                );
            }
        }
    }

    // 8. File Flags: clear any conflicting flags (e.g. immutable) before updating metadata
    // Reason for fallback: query_file_flags is best-effort when unsupported by the filesystem
    let (dest_flags, dest_raw) =
        query_file_flags(dest_path, false).unwrap_or_default();
    #[cfg(target_os = "linux")]
    if !dest_flags.is_empty() {
        use rustix::fs::{IFlags, ioctl_setflags};
        let f_res = std::fs::OpenOptions::new()
            .write(true)
            .open(dest_path)
            .or_else(|_| {
                std::fs::OpenOptions::new().read(true).open(dest_path)
            });
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
    let mut warnings = stream_warnings;
    warnings.extend(apply_entity_metadata(
        dest_path,
        Some(dest_path),
        &entity.metadata,
        false,
        false,
        options.strict_lossless,
        options.check_birthtime,
    )?);

    // 10. Apply final file flags
    let has_flags = entity.metadata.flags.is_some()
        || entity.metadata.platform_raw_flags.is_some();
    if has_flags || !dest_flags.is_empty() || dest_raw.is_some() {
        let flags_slice = entity.metadata.flags.as_deref().unwrap_or(&[]); // Reason for fallback: absent flags metadata defaults to empty slice
        let source_os = entity
            .metadata
            .environment
            .as_ref()
            .map_or(OsFamily::CURRENT, |env| env.os_family());
        apply_file_flags_with_source(
            dest_path,
            flags_slice,
            entity.metadata.platform_raw_flags.as_ref(),
            source_os,
            options.strict_lossless,
        )?;
    }

    #[cfg(windows)]
    warnings.extend(apply_windows_change_time(
        dest_path,
        &entity.metadata,
        options.strict_lossless,
    )?);
    warnings.extend(crate::metadata::check_metadata_replication(
        dest_path,
        &entity.metadata,
        options.strict_lossless,
        false,
        options.check_birthtime,
    )?);

    // 11. Strict lossless audit
    if options.strict_lossless {
        let audit_opts = EntityAuditOptions {
            ignore_atime: !options.check_atime,
            ignore_mtime: false,
            ignore_ctime: !options.check_ctime,
            check_birthtime: options.check_birthtime,
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
        warnings,
        stream_adaptations,
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
    #[cfg(target_os = "linux")]
    #[crate::ctb_test]
    fn test_default_fidelity_keeps_platform_limits_separate() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let source = temporary.path().join("source");
        std::fs::write(&source, b"payload")?;
        let mut entity = FileEntity::from_filesystem(&source, None)?;
        entity.identity.relative_path = Some("default".into());
        let timestamps = entity
            .metadata
            .timestamps
            .as_mut()
            .context("Missing timestamps")?;
        timestamps.birthtime_sec = Some(-1);
        timestamps.birthtime_nsec = Some(123);
        let sandbox = SandboxableDir::open(temporary.path())?;
        let mut payload = MemoryPayloadSource::new(b"payload".to_vec())?;
        materialize_entity(
            &entity,
            Some(&mut payload),
            &sandbox,
            &MaterializeOptions::default(),
        )?;
        entity.identity.relative_path = Some("strict-birthtime".into());
        let mut payload = MemoryPayloadSource::new(b"payload".to_vec())?;
        materialize_entity(
                &entity,
                Some(&mut payload),
                &sandbox,
                &MaterializeOptions {
                    check_birthtime: true,
                    ..Default::default()
                }
            ).unwrap_err();
        entity.identity.relative_path = Some("unknown-native".into());
        entity
            .metadata
            .native
            .as_mut()
            .context("Missing native metadata")?
            .values
            .insert(
                "future.attribute".to_owned(),
                crate::metadata::NativeMetadataValue::Bytes(vec![1, 2, 3]),
            );
        let mut payload = MemoryPayloadSource::new(b"payload".to_vec())?;
        materialize_entity(
                &entity,
                Some(&mut payload),
                &sandbox,
                &MaterializeOptions::default()
            ).unwrap_err();
        entity.identity.relative_path = Some("best-effort".into());
        let mut payload = MemoryPayloadSource::new(b"payload".to_vec())?;
        let receipt = materialize_entity(
            &entity,
            Some(&mut payload),
            &sandbox,
            &MaterializeOptions {
                strict_lossless: false,
                ..Default::default()
            },
        )?;
        assert!(
            receipt
                .warnings
                .iter()
                .any(|warning| warning.contains("future.attribute"))
        );
        Ok(())
    }

    #[crate::ctb_test]
    fn test_file_backed_stream_is_written_before_publication() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let source = temporary.path().join("stream-source");
        let bytes = vec![0x5a_u8; 128];
        std::fs::write(&source, &bytes)?;
        let mut entity = FileEntity::from_vec(b"primary".to_vec());
        entity.identity.relative_path = Some("destination".into());
        entity
            .streams
            .push(crate::streams::AttachedStream::from_file(
                Some(crate::streams::StreamName::from_str("user.lazy")),
                crate::streams::StreamKind::ExtendedAttribute,
                &source,
            )?);
        let sandbox = SandboxableDir::open(temporary.path())?;
        let options = MaterializeOptions::default();
        let companion_options = MaterializeOptions {
            apple_write_mode: AppleWriteMode::ForceAppleDouble(
                crate::apple_double::AppleDoubleStyle::Alongside,
            ),
            ..options.clone()
        };
        let single_options = MaterializeOptions {
            apple_write_mode: AppleWriteMode::ForceAppleSingle,
            ..options.clone()
        };

        // 1. Native materialization of nonresident file-backed stream succeeds
        let mut payload = crate::MemoryPayloadSource::new(b"primary".to_vec())?;
        materialize_entity(&entity, Some(&mut payload), &sandbox, &options)?;
        let restored =
            read_and_hash_streams(&temporary.path().join("destination"))?;
        assert_eq!(restored.len(), 1);
        assert_eq!(
            restored
                .first()
                .context("Missing restored stream")?
                .entity
                .kind,
            entity
                .streams
                .first()
                .context("Missing expected stream")?
                .entity
                .kind
        );

        // 2. AppleDouble materialization of nonresident file-backed stream succeeds
        entity.identity.relative_path = Some("destination-double".into());
        let mut payload = crate::MemoryPayloadSource::new(b"primary".to_vec())?;
        materialize_entity(
            &entity,
            Some(&mut payload),
            &sandbox,
            &companion_options,
        )?;
        assert!(temporary.path().join("destination-double").is_file());
        assert!(temporary.path().join("._destination-double").is_file());
        let parsed_double = crate::apple_double::read_apple_single_double(
            &std::fs::read(temporary.path().join("._destination-double"))?,
        )?;
        assert_eq!(parsed_double.extended_attributes.len(), 1);
        assert_eq!(parsed_double.extended_attributes[0].name, "user.lazy");
        assert_eq!(parsed_double.extended_attributes[0].data, bytes);

        // 3. AppleSingle materialization of nonresident file-backed stream succeeds
        entity.identity.relative_path = Some("destination-single".into());
        let mut payload = crate::MemoryPayloadSource::new(b"primary".to_vec())?;
        materialize_entity(
            &entity,
            Some(&mut payload),
            &sandbox,
            &single_options,
        )?;
        let parsed_single = crate::apple_double::read_apple_single_double(
            &std::fs::read(temporary.path().join("destination-single"))?,
        )?;
        assert_eq!(parsed_single.data_fork.as_deref(), Some(&b"primary"[..]));
        assert_eq!(parsed_single.extended_attributes.len(), 1);
        assert_eq!(parsed_single.extended_attributes[0].name, "user.lazy");
        assert_eq!(parsed_single.extended_attributes[0].data, bytes);

        // 4. Modification of file-backed stream after capture rejects publication in all modes
        std::fs::write(&source, b"changed after capture")?;
        for (rel_name, mode_opts) in [
            ("must-not-publish", &options),
            ("must-not-publish-double", &companion_options),
            ("must-not-publish-single", &single_options),
        ] {
            entity.identity.relative_path = Some(rel_name.into());
            let mut payload =
                crate::MemoryPayloadSource::new(b"primary".to_vec())?;
            materialize_entity(
                    &entity,
                    Some(&mut payload),
                    &sandbox,
                    mode_opts
                ).unwrap_err();
            assert!(!temporary.path().join(rel_name).try_exists()?);
            assert!(
                !temporary
                    .path()
                    .join("._".to_string() + rel_name)
                    .try_exists()?
            );
        }
        assert_eq!(std::fs::read_dir(temporary.path())?.count(), 5);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_write_file_durable_replacement_and_failure_cleanup() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let destination = temporary.path().join("receipt");
        write_file_durable(&destination, b"first receipt")?;
        write_file_durable(&destination, b"updated receipt")?;
        assert_eq!(std::fs::read(&destination)?, b"updated receipt");
        assert_eq!(std::fs::read_dir(temporary.path())?.count(), 1);
        let blocked = temporary.path().join("blocked");
        std::fs::create_dir(&blocked)?;
        assert!(write_file_durable(&blocked, b"must not publish").is_err());
        assert!(blocked.is_dir());
        assert_eq!(std::fs::read_dir(temporary.path())?.count(), 2);
        assert_eq!(std::fs::read(&destination)?, b"updated receipt");
        Ok(())
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_apple_double_preflight_preserves_primary_and_companion()
    -> Result<()> {
        for mode in [
            AppleWriteMode::ForceAppleDouble(
                crate::apple_double::AppleDoubleStyle::Alongside,
            ),
            AppleWriteMode::MaybeAppleDouble(
                crate::apple_double::AppleDoubleStyle::Alongside,
            ),
        ] {
            for dangling in [false, true] {
                let temp = tempfile::tempdir()?;
                let source = temp.path().join("source");
                let destination = temp.path().join("destination");
                let companion = temp.path().join("._destination");
                std::fs::write(&source, b"replacement")?;
                if dangling {
                    std::os::unix::fs::symlink("missing", &companion)?;
                } else {
                    std::fs::write(&companion, b"original companion")?;
                }
                let mut entity = FileEntity::from_filesystem(&source, None)?;
                entity.identity.relative_path =
                    Some(PathBuf::from("destination"));
                entity.metadata.apple = Some(crate::metadata::AppleMetadata {
                    comment: Some("replacement".into()),
                    ..Default::default()
                });
                let sandbox = SandboxableDir::open(temp.path())?;
                let options = MaterializeOptions {
                    apple_write_mode: mode,
                    ..Default::default()
                };
                let mut payload =
                    crate::MemoryPayloadSource::new(b"replacement".to_vec())?;
                let error = materialize_entity(
                    &entity,
                    Some(&mut payload),
                    &sandbox,
                    &options,
                )
                .unwrap_err();
                assert!(
                    format!("{error:#}").contains("companion already exists")
                );
                assert!(!destination.try_exists()?);
                if dangling {
                    assert_eq!(
                        std::fs::read_link(&companion)?,
                        Path::new("missing")
                    );
                } else {
                    assert_eq!(
                        std::fs::read(&companion)?,
                        b"original companion"
                    );
                }
                assert_eq!(std::fs::read_dir(temp.path())?.count(), 2);
            }
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_apple_double_directory_retains_sandbox_ownership() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let source = temp.path().join("source");
        std::fs::create_dir(&source)?;
        let mut entity = FileEntity::from_filesystem(&source, None)?;
        entity.identity.relative_path = Some(PathBuf::from("destination"));
        entity.metadata.apple = Some(crate::metadata::AppleMetadata {
            comment: Some("directory".into()),
            ..Default::default()
        });
        let sandbox = SandboxableDir::open(temp.path())?;
        let options = MaterializeOptions {
            apple_write_mode: AppleWriteMode::ForceAppleDouble(
                crate::apple_double::AppleDoubleStyle::Netatalk,
            ),
            ..Default::default()
        };
        materialize_entity(&entity, None, &sandbox, &options)?;
        sandbox.authorize_metadata(
            Path::new("destination"),
            ExistingEntryPolicy::Refuse,
        )?;
        let reopened = SandboxableDir::open(temp.path())?;
        assert!(
            reopened
                .authorize_metadata(
                    Path::new("destination"),
                    ExistingEntryPolicy::Refuse
                )
                .is_err()
        );
        assert!(
            temp.path()
                .join("destination/.AppleDouble/.Parent")
                .is_file()
        );
        Ok(())
    }

    #[crate::ctb_test]
    fn test_malformed_timestamps_refuse_replacement_in_all_modes() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        let destination = temp.path().join("destination");
        std::fs::write(&source, b"replacement").unwrap();
        std::fs::write(&destination, b"original destination").unwrap();
        let mut entity =
            crate::FileEntity::from_filesystem(&source, None).unwrap();
        entity.identity.relative_path =
            Some(std::path::PathBuf::from("destination"));
        entity.metadata.timestamps.as_mut().unwrap().mtime_nsec = 1_000_000_000;
        let directory = crate::SandboxableDir::open(temp.path()).unwrap();
        for strict_lossless in [true, false] {
            let mut payload =
                crate::MemoryPayloadSource::new(b"replacement".to_vec())
                    .unwrap();
            let options = MaterializeOptions {
                strict_lossless,
                ..MaterializeOptions::default()
            };
            let error = materialize_entity(
                &entity,
                Some(&mut payload),
                &directory,
                &options,
            )
            .unwrap_err();
            assert!(
                format!("{error:#}")
                    .contains("Invalid file timestamp nanoseconds")
            );
            assert_eq!(
                std::fs::read(&destination).unwrap(),
                b"original destination"
            );
        }
    }

    use super::*;
    use crate::*;
    use ctb_formats_checksum::Sha256Stream;
    use std::fs;
    use std::path::PathBuf;

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_symlink_encoding_depends_on_destination_and_lossless_policy()
    -> Result<()> {
        use std::os::unix::ffi::OsStrExt;
        let temp = tempfile::tempdir()?;
        let directory = SandboxableDir::open(temp.path())?;
        for (bytes, is_windows, strict_lossless, succeeds) in [
            (&b"target\xed\xa0"[..], true, true, true),
            (&b"target\xed\xa0\x80\xed\xb0\x80"[..], true, true, true),
            (&b"target\0name"[..], false, true, false),
            (&b"target\0name"[..], false, false, true),
        ] {
            let mut entity = FileEntity::from_vec(Vec::new());
            let target =
                crate::link_path::LinkPath::from_bytes(bytes, is_windows);
            entity.kind = FileEntityKind::Symlink {
                target: target.clone(),
            };
            entity.identity.relative_path = Some(PathBuf::from("link"));
            let options = MaterializeOptions {
                strict_lossless,
                existing_entry_policy: ExistingEntryPolicy::Replace,
                ..Default::default()
            };
            let result =
                materialize_entity(&entity, None, &directory, &options);
            ensure!(
                result.is_ok() == succeeds,
                "Unexpected symlink result: {result:?}"
            );
            if succeeds {
                let expected =
                    target.to_path_buf_for_materialization(strict_lossless)?;
                ensure!(
                    fs::read_link(temp.path().join("link"))?
                        .as_os_str()
                        .as_bytes()
                        == expected.as_os_str().as_bytes(),
                    "Symlink target changed"
                );
                let audit_options = EntityAuditOptions {
                    best_effort: !strict_lossless,
                    ..Default::default()
                };
                ensure!(
                    !audit_entity(
                        &temp.path().join("link"),
                        &entity,
                        &audit_options
                    )?
                    .iter()
                    .any(|diff| matches!(
                        diff,
                        DiffKind::SymlinkTargetMismatch { .. }
                    )),
                    "Audit did not recognize materialized symlink target"
                );
            }
            ensure!(
                target.to_bytes() == bytes,
                "Source symlink target changed"
            );
        }
        Ok(())
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_destination_encoding_retry_is_limited_to_encoding_errors()
    -> Result<()> {
        let encoding = anyhow::Error::from(std::io::Error::from_raw_os_error(
            libc::EILSEQ,
        ))
        .context("Filesystem rejected encoding");
        ensure!(crate::link_path::is_destination_encoding_error(&encoding));
        ensure!(crate::link_path::is_destination_encoding_error(
            &anyhow::Error::from(rustix::io::Errno::ILSEQ)
                .context("Failed to stage symlink"),
        ));
        for code in [libc::EACCES, libc::EINVAL, libc::ENOSPC, libc::EIO] {
            ensure!(
                !crate::link_path::is_destination_encoding_error(
                    &anyhow::Error::from(std::io::Error::from_raw_os_error(
                        code
                    )),
                ),
                "Non-encoding failure triggered replacement adaptation"
            );
        }
        Ok(())
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_materialize_and_verify_regular_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dest_root = temp_dir.path().join("dest_root");
        fs::create_dir_all(&dest_root).unwrap();

        let payload_bytes =
            b"Testing robust file abstraction pipeline with SHA-256";
        let mut hasher = Sha256Stream::new();
        hasher.update(payload_bytes);
        let sha256 = hasher.finalize();
        let size = u64::try_from(payload_bytes.len()).unwrap();

        let rel_path = PathBuf::from("nested/test_file.bin");
        let baseline_file =
            tempfile::NamedTempFile::new_in(&dest_root).unwrap();
        fs::write(baseline_file.path(), payload_bytes).unwrap();
        let (flags, platform_raw_flags) =
            crate::sys_flags::query_file_flags(baseline_file.path(), false)
                .unwrap();
        baseline_file.close().unwrap();
        let entity = FileEntity {
            identity: FileIdentity {
                origin: FileOrigin::Synthetic,
                relative_path: Some(rel_path.clone()),
                enclosing_path: None,
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
                warnings: Vec::new(),
                errors: Vec::new(),
            },
            kind: FileEntityKind::Regular {
                size,
                sha256,
                is_sparse: false,
                extents: vec![Extent::Data {
                    offset: 0,
                    length: size,
                }],
            },
            streams: Vec::new(),
        };

        let mut payload =
            MemoryPayloadSource::new(payload_bytes.to_vec()).unwrap();
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
            (
                true,
                vec![Extent::Data {
                    offset: 0,
                    length: size,
                }],
                payload_bytes[..4].to_vec(),
            ),
            (
                true,
                vec![Extent::Hole {
                    offset: 1,
                    length: size,
                }],
                payload_bytes.to_vec(),
            ),
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
            materialize_entity(
                    &invalid_entity,
                    Some(&mut invalid_payload),
                    &dest_dir,
                    &options,
                ).unwrap_err();
            assert_eq!(fs::read(&target_path).unwrap(), payload_bytes);
            assert_eq!(
                fs::read_dir(target_path.parent().unwrap()).unwrap().count(),
                1
            );
        }
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_existing_materialization_requires_explicit_permission() -> Result<()>
    {
        let temporary = tempfile::tempdir()?;
        let destination = temporary.path().join("destination");
        fs::write(&destination, b"original")?;
        fs::set_permissions(&destination, fs::Permissions::from_mode(0o600))?;
        let before = fs::metadata(&destination)?;
        let sandbox = SandboxableDir::open(temporary.path())?;
        for bytes in [b"original".as_slice(), b"replacement".as_slice()] {
            let mut entity = FileEntity::from_vec(bytes.to_vec());
            entity.identity.relative_path = Some(PathBuf::from("destination"));
            entity.metadata.mode = Some(0o777);
            for force_overwrite in [false, true] {
                let mut source = MemoryPayloadSource::new(bytes.to_vec())?;
                let result = materialize_entity(
                    &entity,
                    Some(&mut source),
                    &sandbox,
                    &MaterializeOptions {
                        force_overwrite,
                        ..Default::default()
                    },
                );
                ensure!(result.is_err());
                let after = fs::metadata(&destination)?;
                ensure!(
                    before.ino() == after.ino()
                        && before.mode() == after.mode()
                );
                ensure!(
                    before.mtime() == after.mtime()
                        && before.mtime_nsec() == after.mtime_nsec()
                );
                ensure!(
                    before.ctime() == after.ctime()
                        && before.ctime_nsec() == after.ctime_nsec()
                );
                ensure!(fs::read(&destination)? == b"original");
            }
        }
        Ok(())
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_authorized_replacement_preserves_unrelated_hardlink() -> Result<()>
    {
        let temporary = tempfile::tempdir()?;
        let destination = temporary.path().join("destination");
        let alias = temporary.path().join("alias");
        fs::write(&destination, b"payload")?;
        fs::hard_link(&destination, &alias)?;
        fs::set_permissions(&destination, fs::Permissions::from_mode(0o600))?;
        let original = fs::metadata(&alias)?;
        let mut entity = FileEntity::from_string("payload");
        entity.identity.relative_path = Some(PathBuf::from("destination"));
        entity.identity.nlink = 2;
        entity.metadata.mode = Some(0o644);
        let mut source = MemoryPayloadSource::new(b"payload".to_vec())?;
        let sandbox = SandboxableDir::open(temporary.path())?;
        let receipt = materialize_entity(
            &entity,
            Some(&mut source),
            &sandbox,
            &MaterializeOptions {
                existing_entry_policy: ExistingEntryPolicy::Replace,
                ..Default::default()
            },
        )?;
        ensure!(!receipt.skipped_identical);
        ensure!(fs::metadata(&alias)?.ino() == original.ino());
        ensure!(fs::metadata(&alias)?.mode() == original.mode());
        ensure!(fs::metadata(&destination)?.ino() != original.ino());
        ensure!(fs::read(&alias)? == b"payload");
        Ok(())
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
                warnings: Vec::new(),
                errors: Vec::new(),
            },
            kind: FileEntityKind::Regular {
                size,
                sha256,
                is_sparse: false,
                extents: vec![Extent::Data {
                    offset: 0,
                    length: size,
                }],
            },
            streams: Vec::new(),
        };

        let mut payload =
            MemoryPayloadSource::new(payload_bytes.to_vec()).unwrap();
        let dest_dir = SandboxableDir::open(&dest_root).unwrap();
        let options = MaterializeOptions {
            existing_entry_policy: ExistingEntryPolicy::Replace,
            ..Default::default()
        };

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
        let mut payload2 =
            MemoryPayloadSource::new(payload_bytes.to_vec()).unwrap();

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
                warnings: Vec::new(),
                errors: Vec::new(),
            },
            kind: FileEntityKind::Regular {
                size,
                sha256,
                is_sparse: false,
                extents: vec![Extent::Data {
                    offset: 0,
                    length: size,
                }],
            },
            streams: Vec::new(),
        };

        let dest_dir = SandboxableDir::open(&dest_root).unwrap();
        let mut p1 = MemoryPayloadSource::new(payload_bytes.to_vec()).unwrap();
        let r1 = entity
            .materialize(
                Some(&mut p1),
                &dest_dir,
                &MaterializeOptions::default(),
            )
            .expect("initial write");
        assert_eq!(r1.bytes_written, size);
        assert!(!r1.skipped_identical);

        // Overwrite with force_overwrite = true
        let mut forced_options = MaterializeOptions::default();
        forced_options.force_overwrite = true;
        forced_options.existing_entry_policy = ExistingEntryPolicy::Replace;

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
                warnings: Vec::new(),
                errors: Vec::new(),
            },
            kind: FileEntityKind::Regular {
                size,
                sha256,
                is_sparse: false,
                extents: vec![Extent::Data {
                    offset: 0,
                    length: size,
                }],
            },
            streams: Vec::new(),
        };

        let dest_dir = SandboxableDir::open(&dest_root).unwrap();
        let mut payload = MemoryPayloadSource::new(new_bytes.to_vec()).unwrap();
        let receipt = entity
            .materialize(
                Some(&mut payload),
                &dest_dir,
                &MaterializeOptions {
                    existing_entry_policy: ExistingEntryPolicy::Replace,
                    ..Default::default()
                },
            )
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
            existing_entry_policy: ExistingEntryPolicy::Replace,
            apple_write_mode: crate::apple_double::AppleWriteMode::NativeOnly,
            apple_single_write_extension:
                crate::apple_double::AppleSingleExtension::WithoutExtension,
            ..Default::default()
        };

        let dest_dir = SandboxableDir::open(temp.path()).unwrap();
        // Materialization should fall back to linear copy and succeed completely without failing
        let mut strict_payload = DiskPayloadSource::open(&src_path).unwrap();
        materializer::materialize_entity(
                &entity,
                Some(&mut strict_payload),
                &dest_dir,
                &MaterializeOptions::default()
            ).unwrap_err();
        assert!(!dest_path.exists());
        materializer::materialize_entity(
            &entity,
            Some(&mut payload),
            &dest_dir,
            &options,
        )
        .unwrap();

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
        assert!(
            root.create_hardlink(
                PathBuf::from("missing").as_path(),
                &root.root_fd(),
                "destination",
                ExistingEntryPolicy::Replace
            )
            .is_err()
        );
        assert!(
            root.create_symlink(
                &root.root_fd(),
                "destination",
                b"invalid\0target",
                SymlinkValidationPolicy::PreserveVerbatim,
                ExistingEntryPolicy::Replace
            )
            .is_err()
        );
        assert!(
            root.create_special(
                &root.root_fd(),
                "destination",
                &FileEntityKind::Socket,
                0o600,
                ExistingEntryPolicy::Replace
            )
            .is_err()
        );
        assert_eq!(fs::read(path).unwrap(), b"keep this data");
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 1);
    }

    #[crate::ctb_test]
    fn test_materialize_entity_refuses_error_or_inaccessible() {
        let temp = tempfile::tempdir().unwrap();
        let dest_dir = SandboxableDir::open(temp.path()).unwrap();
        let options = MaterializeOptions::default();

        // 1. Inaccessible entity
        let inaccessible = FileEntity::inaccessible(
            Path::new("missing.txt"),
            None,
            "Cannot access file",
        );
        let res1 = materialize_entity(&inaccessible, None, &dest_dir, &options);
        assert!(res1.is_err());
        let err1_str = res1.unwrap_err().to_string();
        assert!(
            err1_str.contains("Refusing to materialize inaccessible entity")
        );

        // 2. Regular entity with error attached
        let mut regular_err = FileEntity::from_string("hello");
        regular_err.identity.relative_path = Some(PathBuf::from("hello.txt"));
        regular_err.metadata.push_error("I/O payload corrupt");
        let res2 = materialize_entity(&regular_err, None, &dest_dir, &options);
        assert!(res2.is_err());
        let err2_str = res2.unwrap_err().to_string();
        assert!(err2_str.contains("Refusing to materialize inaccessible entity or entity with recorded errors"));
    }

    #[crate::ctb_test]
    fn test_payload_completion_before_commit() {
        struct CompletionPayload {
            inner: MemoryPayloadSource,
            fail: bool,
            completed: bool,
        }
        impl Read for CompletionPayload {
            fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
                self.inner.read(buffer)
            }
        }
        impl Seek for CompletionPayload {
            fn seek(&mut self, position: SeekFrom) -> std::io::Result<u64> {
                self.inner.seek(position)
            }
        }
        impl PayloadSource for CompletionPayload {
            fn total_size(&self) -> u64 {
                self.inner.total_size()
            }
            fn extents(&self) -> &[Extent] {
                self.inner.extents()
            }
            fn finish(&mut self) -> Result<Vec<String>> {
                self.completed = true;
                anyhow::ensure!(
                    !self.fail,
                    "Injected source completion failure"
                );
                Ok(vec![
                    "Injected source timestamp fidelity warning".to_owned(),
                ])
            }
        }
        let temp = tempfile::tempdir().unwrap();
        let dest_dir = SandboxableDir::open(temp.path()).unwrap();
        let target = temp.path().join("payload");
        let mut entity = FileEntity::from_string("hello");
        entity.identity.relative_path = Some(PathBuf::from("payload"));
        for mode in [
            MaterializeOptions::default().apple_write_mode,
            AppleWriteMode::ForceAppleSingle,
        ] {
            for (fail, strict) in [(true, false), (false, true), (false, false)]
            {
                fs::write(&target, b"existing destination").unwrap();
                let mut payload = CompletionPayload {
                    inner: MemoryPayloadSource::new(b"hello".to_vec()).unwrap(),
                    fail,
                    completed: false,
                };
                let options = MaterializeOptions {
                    force_overwrite: true,
                    existing_entry_policy: ExistingEntryPolicy::Replace,
                    strict_lossless: strict,
                    apple_write_mode: mode,
                    ..MaterializeOptions::default()
                };
                let result = materialize_entity(
                    &entity,
                    Some(&mut payload),
                    &dest_dir,
                    &options,
                );
                assert!(payload.completed);
                if fail || strict {
                    assert!(result.is_err());
                    assert_eq!(
                        fs::read(&target).unwrap(),
                        b"existing destination"
                    );
                } else {
                    assert!(result.unwrap().warnings.iter().any(|warning| {
                        warning.contains("Injected source timestamp")
                    }));
                }
                assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 1);
            }
        }
    }

    #[crate::ctb_test]
    fn test_apple_single_rejects_invalid_payload_before_commit() {
        let temp = tempfile::tempdir().unwrap();
        let dest_dir = SandboxableDir::open(temp.path()).unwrap();
        let target = temp.path().join("payload");
        let mut entity = FileEntity::from_string("hello");
        entity.identity.relative_path = Some(PathBuf::from("payload"));
        let options = MaterializeOptions {
            force_overwrite: true,
            existing_entry_policy: ExistingEntryPolicy::Replace,
            apple_write_mode: AppleWriteMode::ForceAppleSingle,
            ..MaterializeOptions::default()
        };
        for bytes in [b"hell".to_vec(), b"hello!".to_vec(), b"wrong".to_vec()] {
            fs::write(&target, b"existing destination").unwrap();
            let mut payload = MemoryPayloadSource::new(bytes).unwrap();
            materialize_entity(
                    &entity,
                    Some(&mut payload),
                    &dest_dir,
                    &options
                ).unwrap_err();
            assert_eq!(fs::read(&target).unwrap(), b"existing destination");
            assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 1);
        }
    }

    #[crate::ctb_test]
    fn test_sparse_materialization_and_auditing() {
        let temp = tempfile::tempdir().unwrap();
        let dest_dir = SandboxableDir::open(temp.path()).unwrap();
        let file_name = "test_sparse.bin";
        let dest_path = temp.path().join(file_name);

        let total_size = 655_360_u64; // 5 * 128KB
        let extents = vec![
            Extent::Hole {
                offset: 0,
                length: 131_072,
            },
            Extent::Data {
                offset: 131_072,
                length: 131_072,
            },
            Extent::Hole {
                offset: 262_144,
                length: 131_072,
            },
            Extent::Data {
                offset: 393_216,
                length: 131_072,
            },
            Extent::Hole {
                offset: 524_288,
                length: 131_072,
            },
        ];

        let mut payload_bytes =
            vec![0_u8; usize::try_from(total_size).unwrap()];
        if let Some(slice1) = payload_bytes.get_mut(131_072..262_144) {
            slice1.fill(0xAA);
        }
        if let Some(slice2) = payload_bytes.get_mut(393_216..524_288) {
            slice2.fill(0xBB);
        }

        let mut hasher = ctb_formats_checksum::Sha256Stream::new();
        hasher.update(&payload_bytes);
        let sha256 = hasher.finalize();

        let entity = FileEntity {
            identity: FileIdentity {
                origin: FileOrigin::Synthetic,
                relative_path: Some(PathBuf::from(file_name)),
                enclosing_path: None,
                nlink: 1,
                hardlink_group: None,
            },
            kind: FileEntityKind::Regular {
                size: total_size,
                sha256,
                is_sparse: true,
                extents: extents.clone(),
            },
            metadata: FileMetadata::default(),
            streams: Vec::new(),
        };

        let mut payload =
            crate::MemoryPayloadSource::new(payload_bytes.clone()).unwrap();
        let strict_result = materialize_entity(
            &entity,
            Some(&mut payload),
            &dest_dir,
            &MaterializeOptions::default(),
        );
        if let Err(err) = strict_result {
            assert!(format!("{err:#}").contains("sparse"));
            assert!(!dest_path.exists());

            // Native control/readback, not platform identity, decides capability.
            let mut payload =
                crate::MemoryPayloadSource::new(payload_bytes.clone()).unwrap();
            let options = MaterializeOptions {
                strict_lossless: false,
                ..MaterializeOptions::default()
            };
            let receipt = materialize_entity(
                &entity,
                Some(&mut payload),
                &dest_dir,
                &options,
            )
            .unwrap();
            assert!(
                receipt
                    .warnings
                    .iter()
                    .any(|w| w.contains("did not retain sparse support"))
            );
            assert_eq!(std::fs::read(&dest_path).unwrap(), payload_bytes);

            // An advertised sparse-capable volume must not conceal missing holes.
            let audit_opts = EntityAuditOptions {
                best_effort: true,
                check_sparse: true,
                ..EntityAuditOptions::default()
            };
            let (diffs, ignored) =
                audit_entity_detailed(&dest_path, &entity, &audit_opts)
                    .unwrap();
            let fs_info = crate::filesystem::query_filesystem_info(
                &dest_path,
                &std::fs::metadata(&dest_path).unwrap(),
            );
            if fs_info.supports_sparse() == Some(true) {
                assert_eq!(
                    diffs,
                    vec![crate::verifier::DiffKind::SparseHoleMismatch {
                        expected_has_holes: true,
                        actual_has_holes: false,
                    }]
                );
                assert_eq!(ignored.sparseness, 0);
            } else {
                assert!(diffs.is_empty(), "Unexpected diffs: {diffs:?}");
                assert_eq!(ignored.sparseness, 1);
            }
        } else {
            let receipt = strict_result.unwrap();
            assert_eq!(receipt.bytes_written, total_size);

            let read_back = std::fs::read(&dest_path).unwrap();
            assert_eq!(read_back, payload_bytes);

            let audit_opts = EntityAuditOptions {
                check_sparse: true,
                best_effort: false,
                ..EntityAuditOptions::default()
            };
            let (diffs, _) =
                audit_entity_detailed(&dest_path, &entity, &audit_opts)
                    .unwrap();
            assert!(diffs.is_empty(), "Strict sparse audit failed: {diffs:?}");

            let file = std::fs::File::open(&dest_path).unwrap();
            let actual_extents =
                crate::payload::get_file_extents(&file, total_size).unwrap();
            assert_eq!(actual_extents, extents);
        }
    }

    #[crate::ctb_test]
    fn test_invalid_sparse_layout_retains_fidelity_warning() {
        let temp = tempfile::tempdir().unwrap();
        let dest_dir = SandboxableDir::open(temp.path()).unwrap();
        let payload_bytes = vec![0xCC_u8; 32_768];
        let mut hasher = ctb_formats_checksum::Sha256Stream::new();
        hasher.update(&payload_bytes);
        let entity = FileEntity {
            identity: FileIdentity {
                origin: FileOrigin::Synthetic,
                relative_path: Some(PathBuf::from("invalid_sparse.bin")),
                enclosing_path: None,
                nlink: 1,
                hardlink_group: None,
            },
            kind: FileEntityKind::Regular {
                size: u64::try_from(payload_bytes.len()).unwrap(),
                sha256: hasher.finalize(),
                is_sparse: true,
                extents: vec![Extent::Data {
                    offset: 0,
                    length: 16_384,
                }],
            },
            metadata: FileMetadata::default(),
            streams: Vec::new(),
        };
        let mut strict_payload =
            crate::MemoryPayloadSource::new(payload_bytes.clone()).unwrap();
        materialize_entity(
                &entity,
                Some(&mut strict_payload),
                &dest_dir,
                &MaterializeOptions::default()
            ).unwrap_err();
        assert!(!temp.path().join("invalid_sparse.bin").exists());
        let mut payload =
            crate::MemoryPayloadSource::new(payload_bytes.clone()).unwrap();
        let options = MaterializeOptions {
            strict_lossless: false,
            ..MaterializeOptions::default()
        };
        let receipt = materialize_entity(
            &entity,
            Some(&mut payload),
            &dest_dir,
            &options,
        )
        .unwrap();
        assert_eq!(
            std::fs::read(receipt.destination_path).unwrap(),
            payload_bytes
        );
        assert!(receipt.warnings.iter().any(|warning| {
            warning.contains("without preserving sparse layout")
        }));
    }

    #[crate::ctb_test]
    fn test_sub_64k_sparse_allocation_granularity() {
        let temp = tempfile::tempdir().unwrap();
        let dest_dir = SandboxableDir::open(temp.path()).unwrap();
        let file_name = "test_sub_64k_sparse.bin";
        let dest_path = temp.path().join(file_name);

        let total_size = 32_768_u64; // 32KB
        let extents = vec![
            Extent::Hole {
                offset: 0,
                length: 16_384,
            },
            Extent::Data {
                offset: 16_384,
                length: 16_384,
            },
        ];

        let mut payload_bytes =
            vec![0_u8; usize::try_from(total_size).unwrap()];
        if let Some(slice) = payload_bytes.get_mut(16_384..32_768) {
            slice.fill(0xCC);
        }

        let mut hasher = ctb_formats_checksum::Sha256Stream::new();
        hasher.update(&payload_bytes);
        let sha256 = hasher.finalize();

        let entity = FileEntity {
            identity: FileIdentity {
                origin: FileOrigin::Synthetic,
                relative_path: Some(PathBuf::from(file_name)),
                enclosing_path: None,
                nlink: 1,
                hardlink_group: None,
            },
            kind: FileEntityKind::Regular {
                size: total_size,
                sha256,
                is_sparse: true,
                extents: extents.clone(),
            },
            metadata: FileMetadata::default(),
            streams: Vec::new(),
        };

        let normal_destination =
            SandboxableDir::create_or_open(temp.path().join("normal"))
                .unwrap();
        let mut normal_payload =
            crate::MemoryPayloadSource::new(payload_bytes.clone()).unwrap();
        let normal = materialize_entity(
            &entity,
            Some(&mut normal_payload),
            &normal_destination,
            &MaterializeOptions::default(),
        );
        match normal {
            Ok(receipt) => {
                assert_eq!(
                    std::fs::read(receipt.destination_path).unwrap(),
                    payload_bytes
                );
            }
            Err(err) => {
                // If the underlying filesystem (e.g. NTFS on Windows, or ZFS on FreeBSD)
                // has an allocation granularity >= 64KB, strict lossless mode must reject
                // an unrepresentable sub-64KB sparse layout.
                assert!(
                    format!("{err:#}").contains("SparseHoleMismatch")
                        || format!("{err:#}").contains("sparse")
                );
            }
        }

        // Materialize in best-effort mode
        let mut payload =
            crate::MemoryPayloadSource::new(payload_bytes.clone()).unwrap();
        let options = MaterializeOptions {
            strict_lossless: false,
            ..MaterializeOptions::default()
        };
        let _receipt = materialize_entity(
            &entity,
            Some(&mut payload),
            &dest_dir,
            &options,
        )
        .unwrap();
        assert_eq!(std::fs::read(&dest_path).unwrap(), payload_bytes);

        #[cfg(windows)]
        {
            if !ctb_io_environment::is_wine() {
                // On native Windows NTFS, sub-64KB hole warning is recorded
                assert!(
                    receipt
                        .warnings
                        .iter()
                        .any(|w| w.contains("sub-64KB hole")
                            || w.contains("64KB granularity"))
                );
            }
        }

        // Auditing with best_effort: true tolerates sub-64KB holes / missing sparse support
        let audit_opts_be = EntityAuditOptions {
            best_effort: true,
            check_sparse: true,
            ..EntityAuditOptions::default()
        };
        let (diffs, _ignored) =
            audit_entity_detailed(&dest_path, &entity, &audit_opts_be).unwrap();
        assert!(diffs.is_empty(), "Best effort audit must pass: {diffs:?}");

        #[cfg(windows)]
        {
            // On Windows (both native NTFS and WINE), sub-64KB hole cannot be deallocated, so sparseness is ignored
            assert_eq!(ignored.sparseness, 1);

            if !ctb_io_environment::is_wine() {
                // In strict mode on native Windows NTFS, sub-64KB hole causes verification mismatch
                let audit_opts_strict = EntityAuditOptions {
                    best_effort: false,
                    check_sparse: true,
                    ..EntityAuditOptions::default()
                };
                let (strict_diffs, _) = audit_entity_detailed(
                    &dest_path,
                    &entity,
                    &audit_opts_strict,
                )
                .unwrap();
                assert!(
                    !strict_diffs.is_empty(),
                    "Strict audit must detect un-deallocated sub-64KB hole"
                );
            }
        }
    }

    #[crate::ctb_test]
    fn test_hardlink_rejects_escaping_target() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let dest_dir = SandboxableDir::create_or_open(temp_dir.path())?;

        let mut metadata = FileMetadata::default();
        metadata.native = Some(crate::metadata::NativeMetadata {
            source_os: crate::metadata::OsFamily::Windows,
            values: std::collections::BTreeMap::new(),
        });

        // Windows-style escaping hardlink target
        let entity = FileEntity {
            identity: FileIdentity {
                origin: FileOrigin::Synthetic,
                relative_path: Some(PathBuf::from("link.txt")),
                enclosing_path: None,
                nlink: 2,
                hardlink_group: None,
            },
            kind: FileEntityKind::Hardlink {
                target_relative_path: crate::link_path::LinkPath::from_bytes(
                    b"..\\outside.txt",
                    true,
                ),
            },
            metadata,
            streams: Vec::new(),
        };

        let options = MaterializeOptions {
            path_policy: PathTraversalPolicy::StrictSandboxed,
            ..MaterializeOptions::default()
        };

        let res = materialize_entity(&entity, None, &dest_dir, &options);
        assert!(
            res.is_err(),
            "Escaping hardlink target must be rejected under StrictSandboxed"
        );
        let err_msg = res.unwrap_err().to_string();
        assert!(
            err_msg.contains("target escapes destination root or contains root/parent components"),
            "Error message should mention escaping target: {err_msg}"
        );
        Ok(())
    }

    #[crate::ctb_test]
    fn test_best_effort_materialization_adapts_overlong_names_and_logs_warnings()
    -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let dest_dir = SandboxableDir::create_or_open(temp_dir.path())?;

        let overlong_stem = "a".repeat(260);
        let overlong_name = format!("{overlong_stem}.txt");
        let payload_bytes = b"best-effort rename payload".to_vec();
        let sha256 = ctb_formats_checksum::sha256(&payload_bytes);

        let entity = FileEntity {
            identity: FileIdentity {
                origin: FileOrigin::Synthetic,
                relative_path: Some(PathBuf::from(&overlong_name)),
                enclosing_path: None,
                nlink: 1,
                hardlink_group: None,
            },
            kind: FileEntityKind::Regular {
                size: u64::try_from(payload_bytes.len())?,
                sha256,
                is_sparse: false,
                extents: vec![Extent::Data {
                    offset: 0,
                    length: u64::try_from(payload_bytes.len())?,
                }],
            },
            metadata: FileMetadata::default(),
            streams: Vec::new(),
        };

        // Strict mode must reject the >255-byte component.
        let mut strict_payload =
            crate::MemoryPayloadSource::new(payload_bytes.clone())?;
        let strict_opts = MaterializeOptions {
            strict_lossless: true,
            ..MaterializeOptions::default()
        };
        assert!(
            materialize_entity(
                &entity,
                Some(&mut strict_payload),
                &dest_dir,
                &strict_opts
            )
            .is_err(),
            "Strict mode must fail on >255-byte filename"
        );

        // Best-effort mode must truncate the component to <=255 bytes while preserving `.txt`
        // and record the exact rename warning in `receipt.warnings`.
        let mut be_payload =
            crate::MemoryPayloadSource::new(payload_bytes.clone())?;
        let be_opts = MaterializeOptions {
            strict_lossless: false,
            ..MaterializeOptions::default()
        };
        let receipt = materialize_entity(
            &entity,
            Some(&mut be_payload),
            &dest_dir,
            &be_opts,
        )?;
        let materialized_name = receipt
            .destination_path
            .file_name()
            .and_then(|n| n.to_str())
            .context("Expected UTF-8 materialized filename")?;
        assert!(materialized_name.len() <= 255);
        assert!(materialized_name.ends_with(".txt"));
        assert_eq!(std::fs::read(&receipt.destination_path)?, payload_bytes);
        assert!(
            receipt.warnings.iter().any(|w| {
                w.contains("Adapted destination path")
                    && w.contains(&overlong_name)
                    && w.contains(materialized_name)
            }),
            "Expected rename warning in receipt.warnings, got: {:?}",
            receipt.warnings
        );

        // Also verify Windows-specific filename adaptation rules (illegal chars, reserved names, trailing dots).
        let (win_adapted, win_warnings) =
            crate::clean_name::adapt_relative_path_for_materialization(
                Path::new("sub:dir/CON.txt/file?.log."),
                true,
            );
        assert_eq!(win_adapted, PathBuf::from("sub_dir/unnamed.txt/file_.log"));
        assert_eq!(win_warnings.len(), 1);
        assert!(win_warnings[0].contains("Adapted destination path"));

        Ok(())
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_best_effort_symlink_target_and_stream_name_adaptation_warnings()
    -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let dest_dir = SandboxableDir::create_or_open(temp_dir.path())?;

        // Symlink target containing NUL byte is adapted in best-effort mode and records warning.
        let sym_entity = FileEntity {
            identity: FileIdentity {
                origin: FileOrigin::Synthetic,
                relative_path: Some(PathBuf::from("nul_target_link")),
                enclosing_path: None,
                nlink: 1,
                hardlink_group: None,
            },
            kind: FileEntityKind::Symlink {
                target: crate::link_path::LinkPath::from_bytes(
                    b"target\0name",
                    false,
                ),
            },
            metadata: FileMetadata::default(),
            streams: Vec::new(),
        };
        let be_opts = MaterializeOptions {
            strict_lossless: false,
            symlink_policy: SymlinkValidationPolicy::PreserveVerbatim,
            ..MaterializeOptions::default()
        };
        let sym_receipt =
            materialize_entity(&sym_entity, None, &dest_dir, &be_opts)?;
        assert_eq!(
            std::fs::read_link(&sym_receipt.destination_path)?,
            PathBuf::from("target\u{FFFD}name")
        );
        assert!(
            sym_receipt
                .warnings
                .iter()
                .any(|w| w.contains("Replaced NUL in link target")
                    && w.contains("target\\0name")
                    && w.contains("->")),
            "Expected link target adaptation warning, got: {:?}",
            sym_receipt.warnings
        );

        // Stream name containing NUL byte is adapted in best-effort mode and records warning.
        let stream_name =
            crate::streams::StreamName::from_bytes(b"user.bad\0attr");
        let (adapted_stream, stream_warnings) = stream_name
            .to_os_string_for_materialization_with_warnings(false)?;
        assert_eq!(
            adapted_stream,
            std::ffi::OsString::from("user.bad\u{FFFD}attr")
        );
        assert!(
            stream_warnings.iter().any(|w| w.contains("stream name")
                && w.contains("user.bad\\0attr")
                && w.contains("->")),
            "Expected stream name adaptation warning, got: {stream_warnings:?}"
        );

        Ok(())
    }
}
