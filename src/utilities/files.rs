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

//! File system helpers, safe path resolution, and symlink containment checks.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// If `path` is a symlink, checks that it resolves to a location inside `dir`.
/// Returns the canonical path if it is a symlink, or the original path otherwise.
/// Returns an error if the symlink target is outside `dir` or cannot be resolved.
pub fn symlink_is_in_dir(path: &Path, dir: &Path) -> Result<PathBuf> {
    if let Ok(meta) = std::fs::symlink_metadata(path) {
        if meta.file_type().is_symlink() {
            let canonical = std::fs::canonicalize(path).with_context(|| {
                format!("Failed to resolve symlink {}", path.display())
            })?;
            // Reason for fallback: directory may not exist on disk yet, so fallback to raw PathBuf for prefix checking
            let canonical_dir = std::fs::canonicalize(dir)
                .unwrap_or_else(|_| dir.to_path_buf());
            if !canonical.starts_with(&canonical_dir) {
                anyhow::bail!(
                    "Symlink {} resolves outside directory {}: {}",
                    path.display(),
                    dir.display(),
                    canonical.display()
                );
            }
            return Ok(canonical);
        }
    }
    Ok(path.to_path_buf())
}

/// Checks if `path` is a symlink that resolves to a location inside `dir`.
pub fn is_symlink_in_dir(path: &Path, dir: &Path) -> bool {
    symlink_is_in_dir(path, dir).is_ok()
}

/// Guard that removes a temporary staged file if it is dropped before being
/// committed via rename.
struct StagedFile {
    path: PathBuf,
    active: bool,
}

impl StagedFile {
    fn new(path: PathBuf) -> Self {
        Self { path, active: true }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn disarm(&mut self) {
        self.active = false;
    }
}

impl Drop for StagedFile {
    fn drop(&mut self) {
        if self.active {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

/// Resolves the directory where a staging file next to `path` should live.
fn parent_dir_for_staging(path: &Path) -> Result<&Path> {
    match path.parent() {
        Some(parent) if parent.as_os_str().is_empty() => Ok(Path::new(".")),
        Some(parent) => Ok(parent),
        None => {
            anyhow::bail!("Path {} has no parent directory", path.display())
        }
    }
}

/// Copies `source` into a new uniquely named staging file in `dest_dir`, syncs
/// it, sets `0o755` permissions on Unix when `executable` is true, and closes
/// the writable handle before returning.
fn stage_file_in_dir(
    source: &Path,
    dest_dir: &Path,
    label: &str,
    executable: bool,
) -> Result<StagedFile> {
    let mut src_file = std::fs::File::open(source).with_context(|| {
        format!("Failed to open source file {}", source.display())
    })?;
    let staged_path =
        dest_dir.join(format!(".ctb-{label}-{}.tmp", uuid::Uuid::new_v4()));
    let mut dst_file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&staged_path)
        .with_context(|| {
            format!("Failed to create staging file {}", staged_path.display())
        })?;
    // Arm cleanup immediately so any write, sync, or permission error removes
    // the partial staging file.
    let staged = StagedFile::new(staged_path);

    std::io::copy(&mut src_file, &mut dst_file).with_context(|| {
        format!(
            "Failed to copy {} to staging file {}",
            source.display(),
            staged.path().display()
        )
    })?;
    dst_file.sync_all().with_context(|| {
        format!("Failed to sync staging file {}", staged.path().display())
    })?;

    #[cfg(unix)]
    if executable {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = dst_file
            .metadata()
            .with_context(|| {
                format!(
                    "Failed to read metadata for {}",
                    staged.path().display()
                )
            })?
            .permissions();
        perms.set_mode(0o755);
        dst_file.set_permissions(perms).with_context(|| {
            format!(
                "Failed to set executable permissions on {}",
                staged.path().display()
            )
        })?;
    }
    #[cfg(not(unix))]
    let _ = executable;

    // Close the writable descriptor/handle explicitly before any rename or exec.
    drop(dst_file);
    Ok(staged)
}

/// Moves `staged_new` into `target_path` after `target_path` has already been
/// renamed to `displaced_path`, rolling back `displaced_path` if the move
/// fails.
#[cfg(not(unix))]
fn install_over_displaced(
    staged_new: &mut StagedFile,
    displaced_path: &Path,
    target_path: &Path,
) -> Result<()> {
    if let Err(install_err) = std::fs::rename(staged_new.path(), target_path) {
        let rollback_status = match std::fs::rename(displaced_path, target_path)
        {
            Ok(()) => "rolled back previous binary".to_owned(),
            Err(rollback_err) => {
                format!("rollback also failed: {rollback_err}")
            }
        };
        return Err(install_err).with_context(|| {
            format!(
                "Failed to install staged file {} at {} ({rollback_status})",
                staged_new.path().display(),
                target_path.display()
            )
        });
    }
    staged_new.disarm();
    Ok(())
}

/// Renames a running `target_path` to a temporary sibling file, installs
/// `staged_new` in its place, and best-effort deletes the displaced copy.
#[cfg(not(unix))]
fn displace_temp_and_install(
    staged_new: &mut StagedFile,
    target_dir: &Path,
    target_path: &Path,
) -> Result<()> {
    let displaced_path =
        target_dir.join(format!(".ctb-old-{}.tmp", uuid::Uuid::new_v4()));
    std::fs::rename(target_path, &displaced_path).with_context(|| {
        format!(
            "Failed to displace active binary {} to {}",
            target_path.display(),
            displaced_path.display()
        )
    })?;
    install_over_displaced(staged_new, &displaced_path, target_path)?;
    if let Err(e) = std::fs::remove_file(&displaced_path) {
        crate::warn_fmt!(
            "Could not immediately remove displaced binary {}: {}",
            displaced_path.display(),
            e
        );
    }
    Ok(())
}

/// Replaces `target_path` with `source_path`, safe even when `target_path` is
/// currently executing.
///
/// Stages the new binary alongside `target_path`, marks it executable (`0o755`)
/// on Unix, and atomically replaces `target_path`. On non-Unix platforms where
/// a running binary cannot be overwritten directly, the active binary is first
/// renamed to a sibling temporary file before moving the staged file into
/// place.
pub fn replace_executable_file(
    source_path: &Path,
    target_path: &Path,
) -> Result<()> {
    let target_dir = parent_dir_for_staging(target_path)?;
    let mut staged_new =
        stage_file_in_dir(source_path, target_dir, "stage", true)?;

    #[cfg(unix)]
    {
        std::fs::rename(staged_new.path(), target_path).with_context(|| {
            format!(
                "Failed to move staged file {} to {}",
                staged_new.path().display(),
                target_path.display()
            )
        })?;
        staged_new.disarm();
    }

    #[cfg(not(unix))]
    {
        if let Err(rename_err) = std::fs::rename(staged_new.path(), target_path)
        {
            if !target_path.exists() {
                return Err(rename_err).with_context(|| {
                    format!(
                        "Failed to move staged file {} to {}",
                        staged_new.path().display(),
                        target_path.display()
                    )
                });
            }
            displace_temp_and_install(&mut staged_new, target_dir, target_path)
                .with_context(|| {
                    format!(
                        "Direct replacement of {} failed ({rename_err})",
                        target_path.display()
                    )
                })?;
        } else {
            staged_new.disarm();
        }
    }

    Ok(())
}

/// Replaces `target_path` with `source_path` while preserving the existing
/// `target_path` at `backup_path`, safe even while `target_path` is running.
pub fn replace_executable_with_backup(
    source_path: &Path,
    target_path: &Path,
    backup_path: &Path,
) -> Result<()> {
    anyhow::ensure!(
        backup_path != target_path,
        "Backup path {} must differ from target path",
        backup_path.display()
    );
    anyhow::ensure!(
        target_path.exists(),
        "Target binary does not exist at {}",
        target_path.display()
    );

    let target_dir = parent_dir_for_staging(target_path)?;
    // Stage the incoming binary before modifying `backup_path` or `target_path`
    // so unreadable inputs fail with no side effects.
    let mut staged_new =
        stage_file_in_dir(source_path, target_dir, "stage", true)?;

    #[cfg(unix)]
    {
        let backup_dir = parent_dir_for_staging(backup_path)?;
        let mut staged_backup =
            stage_file_in_dir(target_path, backup_dir, "backup", true)?;
        std::fs::rename(staged_backup.path(), backup_path).with_context(
            || {
                format!(
                    "Failed to move staged backup {} to {}",
                    staged_backup.path().display(),
                    backup_path.display()
                )
            },
        )?;
        staged_backup.disarm();

        std::fs::rename(staged_new.path(), target_path).with_context(|| {
            format!(
                "Failed to move staged binary {} to {}",
                staged_new.path().display(),
                target_path.display()
            )
        })?;
        staged_new.disarm();
    }

    #[cfg(not(unix))]
    {
        match std::fs::rename(target_path, backup_path) {
            Ok(()) => {
                install_over_displaced(
                    &mut staged_new,
                    backup_path,
                    target_path,
                )?;
            }
            Err(rename_err)
                if rename_err.kind() == std::io::ErrorKind::CrossesDevices
                    || rename_err.raw_os_error() == Some(17) =>
            {
                let backup_dir = parent_dir_for_staging(backup_path)?;
                let mut staged_backup =
                    stage_file_in_dir(target_path, backup_dir, "backup", true)?;
                std::fs::rename(staged_backup.path(), backup_path)
                    .with_context(|| {
                        format!(
                            "Failed to move staged backup {} to {}",
                            staged_backup.path().display(),
                            backup_path.display()
                        )
                    })?;
                staged_backup.disarm();

                displace_temp_and_install(
                    &mut staged_new,
                    target_dir,
                    target_path,
                )?;
            }
            Err(rename_err) => {
                return Err(rename_err).with_context(|| {
                    format!(
                        "Failed to move running binary {} to backup {}",
                        target_path.display(),
                        backup_path.display()
                    )
                });
            }
        }
    }

    Ok(())
}

/// Marks `file` as sparse on filesystems that require explicit sparse opt-in
/// (such as Windows NTFS) before calling `File::set_len` to create large zero
/// regions without reserving physical disk clusters.
///
/// On Unix, extending a file via `set_len` (`ftruncate`) is sparse by default,
/// so this is a no-op. On Windows filesystems that do not support sparse files
/// (such as FAT32), `ERROR_INVALID_FUNCTION` is ignored so callers can still
/// fall back to regular allocation when space permits.
#[cfg_attr(
    windows,
    expect(
        unsafe_code,
        reason = "DeviceIoControl(FSCTL_SET_SPARSE) requires Win32 FFI with a valid open file handle and null input/output buffers"
    )
)]
pub fn try_set_sparse(file: &std::fs::File) -> std::io::Result<()> {
    #[cfg(windows)]
    {
        use std::os::windows::io::AsRawHandle;

        const FSCTL_SET_SPARSE: u32 = 0x0009_00C4;
        const ERROR_INVALID_FUNCTION: i32 = 1;

        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn DeviceIoControl(
                h_device: *mut std::ffi::c_void,
                dw_io_control_code: u32,
                lp_in_buffer: *const std::ffi::c_void,
                n_in_buffer_size: u32,
                lp_out_buffer: *mut std::ffi::c_void,
                n_out_buffer_size: u32,
                lp_bytes_returned: *mut u32,
                lp_overlapped: *mut std::ffi::c_void,
            ) -> i32;
        }

        let mut bytes_returned: u32 = 0;
        // SAFETY: `file` owns a live OS handle for the duration of the call,
        // `bytes_returned` is a valid stack u32, and FSCTL_SET_SPARSE uses null
        // input/output buffers.
        let ok = unsafe {
            DeviceIoControl(
                file.as_raw_handle(),
                FSCTL_SET_SPARSE,
                std::ptr::null(),
                0,
                std::ptr::null_mut(),
                0,
                &mut bytes_returned,
                std::ptr::null_mut(),
            )
        };
        if ok == 0 {
            let err = std::io::Error::last_os_error();
            if err.raw_os_error() != Some(ERROR_INVALID_FUNCTION) {
                return Err(err);
            }
        }
    }
    #[cfg(not(windows))]
    let _ = file;
    Ok(())
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
    use std::fs;
    use std::process::{Command, Stdio};

    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> Self {
            let path = std::env::temp_dir()
                .join(format!("ctoolbox-test-{}", uuid::Uuid::new_v4()));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn assert_no_staging_temps(dir: &Path) {
        for entry in fs::read_dir(dir).unwrap() {
            let name = entry.unwrap().file_name();
            let name_str = name.to_string_lossy();
            let is_tmp = name_str.ends_with(".tmp");
            assert!(
                !(is_tmp
                    && (name_str.starts_with(".ctb-stage-")
                        || name_str.starts_with(".ctb-backup-"))),
                "Unexpected leftover staging file {name_str}"
            );
            #[cfg(unix)]
            assert!(
                !(is_tmp && name_str.starts_with(".ctb-old-")),
                "Unexpected leftover displaced file {name_str}"
            );
        }
    }

    fn run_copied_executable_test(
        target_exe: &Path,
        exact_test: &str,
        env_key: &str,
        env_val: &Path,
    ) -> std::process::ExitStatus {
        let mut child = crate::spawn_with_etxtbsy_retry(|| {
            Command::new(target_exe)
                .args(["--exact", exact_test, "--nocapture"])
                .env(env_key, env_val)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
        })
        .expect("Failed to spawn copied test executable");
        child.wait().expect("Failed to wait on child process")
    }

    #[crate::ctb_test]
    fn test_symlink_in_dir() {
        let temp_path = std::env::temp_dir()
            .join(format!("ctoolbox-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_path).unwrap();

        let dir1 = temp_path.join("dir1");
        let dir2 = temp_path.join("dir2");
        fs::create_dir_all(&dir1).unwrap();
        fs::create_dir_all(&dir2).unwrap();

        let target_in = dir1.join("target.txt");
        fs::write(&target_in, "hello").unwrap();

        let target_out = dir2.join("target.txt");
        fs::write(&target_out, "world").unwrap();

        // Symlink inside dir1 pointing inside dir1
        let sym_in = dir1.join("sym_in.txt");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&target_in, &sym_in).unwrap();

        // Symlink inside dir1 pointing outside (to dir2)
        let sym_out = dir1.join("sym_out.txt");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&target_out, &sym_out).unwrap();

        #[cfg(unix)]
        {
            // Valid symlink inside dir1 should succeed
            let res = symlink_is_in_dir(&sym_in, &dir1);
            assert!(res.is_ok());
            assert_eq!(res.unwrap(), fs::canonicalize(&target_in).unwrap());

            // Invalid symlink pointing outside dir1 should fail
            let res = symlink_is_in_dir(&sym_out, &dir1);
            res.unwrap_err();
        }

        // Regular file should return path itself
        let res = symlink_is_in_dir(&target_in, &dir1);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), target_in);

        let _ = fs::remove_dir_all(&temp_path);
    }

    #[crate::ctb_test]
    fn test_replace_running_executable_with_backup() {
        const NEW_BIN_ENV: &str = "CTB_TEST_REPLACE_RUNNING_NEW_BIN";
        if let Some(new_binary_os) = std::env::var_os(NEW_BIN_ENV) {
            let current_exe = std::env::current_exe().unwrap();
            let backup_path = current_exe.with_extension("backup");
            replace_executable_with_backup(
                Path::new(&new_binary_os),
                &current_exe,
                &backup_path,
            )
            .unwrap();
            return;
        }

        let temp_dir = TempDir::new();
        let target_exe = temp_dir
            .path()
            .join(format!("test-runner{}", std::env::consts::EXE_SUFFIX));
        let current_exe = std::env::current_exe().unwrap();
        replace_executable_file(&current_exe, &target_exe).unwrap();
        let original_len = fs::metadata(&target_exe).unwrap().len();

        let backup_path = target_exe.with_extension("backup");
        fs::write(&backup_path, b"stale backup").unwrap();

        let new_binary = temp_dir.path().join("new-runner");
        fs::write(&new_binary, b"upgraded-binary-payload-v2").unwrap();

        let status = run_copied_executable_test(
            &target_exe,
            "files::tests::test_replace_running_executable_with_backup",
            NEW_BIN_ENV,
            &new_binary,
        );
        assert!(status.success(), "Child replacement test should succeed");

        assert_eq!(
            fs::read(&target_exe).unwrap(),
            b"upgraded-binary-payload-v2"
        );
        assert_eq!(fs::metadata(&backup_path).unwrap().len(), original_len);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(&target_exe).unwrap().permissions().mode();
            assert!(
                mode & 0o111 != 0,
                "Replaced executable should retain executable bits"
            );
        }
        assert!(fs::remove_file(&backup_path).is_ok());
        assert_no_staging_temps(temp_dir.path());
    }

    #[crate::ctb_test]
    fn test_replace_executable_file_while_running() {
        const RESTORE_BIN_ENV: &str = "CTB_TEST_REPLACE_FILE_NEW_BIN";
        if let Some(new_binary_os) = std::env::var_os(RESTORE_BIN_ENV) {
            let current_exe = std::env::current_exe().unwrap();
            replace_executable_file(Path::new(&new_binary_os), &current_exe)
                .unwrap();
            return;
        }

        let temp_dir = TempDir::new();
        let target_exe = temp_dir
            .path()
            .join(format!("test-runner{}", std::env::consts::EXE_SUFFIX));
        let current_exe = std::env::current_exe().unwrap();
        replace_executable_file(&current_exe, &target_exe).unwrap();

        let replacement = temp_dir.path().join("replacement-runner");
        fs::write(&replacement, b"restored-binary-payload-v1").unwrap();

        let status = run_copied_executable_test(
            &target_exe,
            "files::tests::test_replace_executable_file_while_running",
            RESTORE_BIN_ENV,
            &replacement,
        );
        assert!(
            status.success(),
            "Child file replacement test should succeed"
        );

        assert_eq!(
            fs::read(&target_exe).unwrap(),
            b"restored-binary-payload-v1"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(&target_exe).unwrap().permissions().mode();
            assert!(
                mode & 0o111 != 0,
                "Replaced executable should retain executable bits"
            );
        }
        #[cfg(not(unix))]
        for entry in fs::read_dir(temp_dir.path()).unwrap() {
            let entry = entry.unwrap();
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with(".ctb-old-") && name_str.ends_with(".tmp") {
                fs::remove_file(entry.path()).expect(
                    "Displaced executable should be removable once child exits",
                );
            }
        }
        assert_no_staging_temps(temp_dir.path());
    }

    #[crate::ctb_test]
    fn test_replace_executable_with_backup_validation_and_cleanup() {
        let temp_dir = TempDir::new();
        let src = temp_dir.path().join("new-bin");
        let target = temp_dir.path().join("target-bin");
        let backup = temp_dir.path().join("target-bin.backup");

        fs::write(&src, b"new").unwrap();
        fs::write(&target, b"old").unwrap();

        replace_executable_with_backup(&src, &target, &target).unwrap_err();
        assert_eq!(fs::read(&target).unwrap(), b"old");
        assert_no_staging_temps(temp_dir.path());

        let missing_target = temp_dir.path().join("missing-target");
        replace_executable_with_backup(&src, &missing_target, &backup)
            .unwrap_err();
        assert!(!backup.exists());
        assert_no_staging_temps(temp_dir.path());

        let missing_source = temp_dir.path().join("missing-source");
        replace_executable_with_backup(&missing_source, &target, &backup)
            .unwrap_err();
        assert_eq!(fs::read(&target).unwrap(), b"old");
        assert!(!backup.exists());
        assert_no_staging_temps(temp_dir.path());
    }
}
