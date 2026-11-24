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

//! Atomic binary upgrade and canary validation.
//!
//! This module implements the "upgrade dance" for safely replacing the running
//! ctoolbox binary:
//!
//! 1. Download new binary to a temp location
//! 2. Copy current binary to a backup location
//! 3. Spawn new binary with `--ctb-upgrade-canary --backup-path {backup}`
//! 4. Exit current process
//! 5. New binary waits 30s; if still alive, delete backup and optionally
//!    restart
//! 6. If new binary crashes, backup can be restored by external watchdog logic

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::download::{
    ChunkDownloader, current_platform, no_progress_callback,
};
use crate::install::InstallationRecord;
use crate::manifest::ReleaseManifest;

/// Duration to wait during canary validation (30 seconds).
const CANARY_WAIT_SECS: u64 = 30;

/// Result of checking for available updates.
#[derive(Debug)]
pub struct UpdateCheckResult {
    /// Whether an update is available.
    pub available: bool,
    /// Current installed version.
    pub current_version: semver::Version,
    /// Latest available version (if update available).
    pub latest_version: Option<semver::Version>,
    /// The manifest for the latest version (if update available).
    pub manifest: Option<ReleaseManifest>,
}

/// Checks whether an update is available from the server.
///
/// Compares the currently installed version (from installation.json) against
/// the latest manifest from the server.
///
/// # Arguments
/// - `server_url`: URL of the update server
///
/// # Returns
/// An `UpdateCheckResult` indicating whether an update is available.
///
/// # Errors
/// Returns an error if the installation record cannot be loaded or the server
/// cannot be reached.
pub async fn check_for_update(server_url: &str) -> Result<UpdateCheckResult> {
    // Load current installation record
    let record = InstallationRecord::load()
        .context("No installation record found. Is ctoolbox installed?")?;
    let current_version = record.ctoolbox_version.clone();

    // Download latest manifest from server
    let downloader = ChunkDownloader::new(server_url, no_progress_callback())?;
    let platform = current_platform();
    let manifest = downloader.download_manifest(&platform, None).await?;
    let latest_version = manifest.ctoolbox_version.clone();

    // Compare versions
    let available = latest_version > current_version;

    Ok(UpdateCheckResult {
        available,
        current_version,
        latest_version: if available {
            Some(latest_version)
        } else {
            None
        },
        manifest: if available { Some(manifest) } else { None },
    })
}

/// Gets the current installed version from the installation record.
///
/// # Errors
/// Returns an error if no installation record exists.
pub fn get_installed_version() -> Result<semver::Version> {
    let record = InstallationRecord::load()
        .context("No installation record found. Is ctoolbox installed?")?;
    Ok(record.ctoolbox_version)
}

#[cfg(windows)]
struct WindowsErrorModeGuard {
    previous: u32,
}

#[cfg(windows)]
impl WindowsErrorModeGuard {
    #[expect(
        unsafe_code,
        reason = "SetErrorMode suppresses modal Windows system error dialogs while spawning an untrusted canary binary"
    )]
    fn suppress_dialogs() -> Self {
        const SEM_FAILCRITICALERRORS: u32 = 0x0001;
        const SEM_NOGPFAULTERRORBOX: u32 = 0x0002;
        const SEM_NOOPENFILEERRORBOX: u32 = 0x8000;
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn SetErrorMode(u_mode: u32) -> u32;
        }
        // SAFETY: SetErrorMode is thread/process-safe FFI taking a bitmask of SEM_* flags and returning the prior mode.
        let previous = unsafe {
            let prev = SetErrorMode(0);
            let _ = SetErrorMode(
                prev | SEM_FAILCRITICALERRORS
                    | SEM_NOGPFAULTERRORBOX
                    | SEM_NOOPENFILEERRORBOX,
            );
            prev
        };
        Self { previous }
    }
}

#[cfg(windows)]
impl Drop for WindowsErrorModeGuard {
    #[expect(
        unsafe_code,
        reason = "Restores the process error mode saved by WindowsErrorModeGuard::suppress_dialogs"
    )]
    fn drop(&mut self) {
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn SetErrorMode(u_mode: u32) -> u32;
        }
        // SAFETY: Restores the valid bitmask previously returned by SetErrorMode.
        unsafe {
            let _ = SetErrorMode(self.previous);
        }
    }
}

/// Performs the atomic upgrade dance.
///
/// This function:
/// 1. Copies the current binary to a backup location
/// 2. Copies the new binary over the current location
/// 3. Spawns the new binary with `--ctb-upgrade-canary`
/// 4. Returns so the caller can exit
///
/// The caller should exit immediately after this returns successfully.
///
/// # Arguments
/// - `new_binary_path`: Path to the downloaded new binary
/// - `target_path`: Path where the installed binary lives
/// - `port`: Optional port to restart ctoolbox with after validation
///
/// # Errors
/// Returns an error if the upgrade cannot be started.
pub fn start_atomic_upgrade(
    new_binary_path: &Path,
    target_path: &Path,
    port: Option<u16>,
) -> Result<()> {
    // Create backup path next to the target
    let backup_path = target_path.with_extension("backup");

    // Back up the running binary and stage-replace `target_path` safely on
    // both Unix (no ETXTBSY) and Windows (rename active binary aside first).
    ctb_utilities::files::replace_executable_with_backup(
        new_binary_path,
        target_path,
        &backup_path,
    )
    .with_context(|| {
        format!(
            "Failed to replace {} with {} (backup at {})",
            target_path.display(),
            new_binary_path.display(),
            backup_path.display()
        )
    })?;

    log_fmt!(
        "Installed new binary at {} (backup at {})",
        target_path.display(),
        backup_path.display()
    );

    // Spawn the new binary with `ctb-upgrade-canary`; roll back if launch fails.
    let mut args = vec![
        std::ffi::OsString::from("ctb-upgrade-canary"),
        std::ffi::OsString::from("--backup-path"),
        backup_path.as_os_str().to_os_string(),
        std::ffi::OsString::from("--target-path"),
        target_path.as_os_str().to_os_string(),
    ];
    if let Some(p) = port {
        args.push(std::ffi::OsString::from("--port"));
        args.push(std::ffi::OsString::from(p.to_string()));
    }

    let spawn_result = {
        #[cfg(windows)]
        let _err_mode = WindowsErrorModeGuard::suppress_dialogs();
        ctb_utilities::spawn_detached(target_path, &args)
    };
    if let Err(spawn_err) = spawn_result {
        let rollback_status = match fs::rename(&backup_path, target_path) {
            Ok(()) => "restored original binary from backup".to_owned(),
            Err(rollback_err) => {
                format!("rollback also failed: {rollback_err}")
            }
        };
        return Err(spawn_err).with_context(|| {
            format!(
                "Failed to spawn canary process {} ({rollback_status})",
                target_path.display()
            )
        });
    }

    log_fmt!("Spawned canary process; caller should exit now");

    Ok(())
}

/// Runs the canary validation process.
///
/// This is called by the new binary after an upgrade. It:
/// 1. Waits 30 seconds
/// 2. If still alive, deletes the backup
/// 3. Optionally restarts ctoolbox normally
///
/// If this process crashes within 30s, the backup remains available for
/// external watchdog logic to restore.
///
/// # Arguments
/// - `backup_path`: Path to the backup copy of the previous binary
/// - `target_path`: Path to the installed binary
/// - `port`: Optional port to restart ctoolbox with
///
/// # Errors
/// Returns an error if the backup cannot be deleted (non-fatal logged).
pub fn run_canary_validation(
    backup_path: &Path,
    target_path: &Path,
    port: Option<u16>,
) -> Result<()> {
    run_canary_validation_with_delay(
        backup_path,
        target_path,
        port,
        Duration::from_secs(CANARY_WAIT_SECS),
    )
}

fn run_canary_validation_with_delay(
    backup_path: &Path,
    target_path: &Path,
    port: Option<u16>,
    wait: Duration,
) -> Result<()> {
    log_fmt!("Starting canary validation, waiting {}s...", wait.as_secs());

    // Wait for the canary period
    std::thread::sleep(wait);

    log_fmt!("Canary validation passed, deleting backup");

    // Delete the backup - the upgrade is considered successful
    if backup_path.exists() {
        if let Err(e) = fs::remove_file(backup_path) {
            warn_fmt!(
                "Failed to delete backup at {}: {}",
                backup_path.display(),
                e
            );
        } else {
            log_fmt!("Deleted backup at {}", backup_path.display());
        }
    }

    // Optionally restart ctoolbox normally
    if let Some(p) = port {
        log_fmt!("Restarting ctoolbox on port {}", p);
        restart_ctoolbox(target_path, p)?;
    }

    Ok(())
}

/// Restores from backup after a failed upgrade.
///
/// This can be called by external watchdog logic if the canary process crashes.
///
/// # Arguments
/// - `backup_path`: Path to the backup copy
/// - `target_path`: Path to restore to
///
/// # Errors
/// Returns an error if the restore fails.
pub fn restore_from_backup(
    backup_path: &Path,
    target_path: &Path,
) -> Result<()> {
    if !backup_path.exists() {
        bail!(
            "Backup does not exist at {}, cannot restore",
            backup_path.display()
        );
    }

    ctb_utilities::files::replace_executable_file(backup_path, target_path)
        .with_context(|| {
            format!(
                "Failed to restore backup from {} to {}",
                backup_path.display(),
                target_path.display()
            )
        })?;

    // Delete the backup after successful restore
    fs::remove_file(backup_path).with_context(|| {
        format!("Failed to delete backup at {}", backup_path.display())
    })?;

    log_fmt!("Restored {} from backup", target_path.display());

    Ok(())
}

/// Restarts ctoolbox with the given port.
fn restart_ctoolbox(binary_path: &Path, port: u16) -> Result<()> {
    let port_str = port.to_string();
    ctb_utilities::spawn_detached(
        binary_path,
        ["--ctoolbox-ipc-port", port_str.as_str()],
    )
}

/// Downloads the new binary for an update.
///
/// Downloads the main ctoolbox binary from the manifest to a temporary
/// location.
///
/// # Arguments
/// - `server_url`: URL of the update server
/// - `manifest`: The release manifest containing file information
/// - `cache_dir`: Directory to cache chunks during download
///
/// # Returns
/// Path to the downloaded binary.
///
/// # Errors
/// Returns an error if the download fails.
pub async fn download_new_binary(
    server_url: &str,
    manifest: &ReleaseManifest,
    cache_dir: &Path,
) -> Result<PathBuf> {
    // Find the main binary in the manifest
    let binary_entry = manifest
        .files
        .iter()
        .find(|f| f.path == "ctoolbox" || f.path == "bin/ctoolbox")
        .ok_or_else(|| {
            anyhow::anyhow!("No ctoolbox binary found in manifest")
        })?;

    // Create a temporary file for the downloaded binary
    let temp_dir = std::env::temp_dir();
    let temp_binary = temp_dir.join("ctoolbox-update");

    // Download the binary
    let downloader = ChunkDownloader::new(server_url, no_progress_callback())?;
    downloader
        .download_file(binary_entry, cache_dir, &temp_binary, None)
        .await?;

    Ok(temp_binary)
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
    use std::process::{Command, Stdio};
    use tempfile::TempDir;

    #[crate::ctb_test]
    fn test_restore_from_backup() {
        let temp_dir = TempDir::new().unwrap();
        let backup_path = temp_dir.path().join("backup");
        let target_path = temp_dir.path().join("target");

        // Create a "backup" file
        fs::write(&backup_path, b"backup content").unwrap();

        // Create a "corrupted" target
        fs::write(&target_path, b"corrupted").unwrap();

        // Restore
        restore_from_backup(&backup_path, &target_path).unwrap();

        // Verify
        let content = fs::read_to_string(&target_path).unwrap();
        assert_eq!(content, "backup content");
        assert!(!backup_path.exists());
    }

    #[crate::ctb_test]
    fn test_restore_missing_backup() {
        let temp_dir = TempDir::new().unwrap();
        let backup_path = temp_dir.path().join("nonexistent");
        let target_path = temp_dir.path().join("target");

        let result = restore_from_backup(&backup_path, &target_path);
        assert!(result.is_err());
    }

    #[crate::ctb_test]
    fn test_backup_creation_during_upgrade_setup() {
        let temp_dir = TempDir::new().unwrap();
        let original_binary = temp_dir.path().join("ctoolbox");
        let new_binary = temp_dir.path().join("ctoolbox-new");

        // Create the "original" binary
        fs::write(&original_binary, b"original binary content v1.0").unwrap();

        // Create the "new" binary
        fs::write(&new_binary, b"new binary content v2.0").unwrap();

        // We can't fully test start_atomic_upgrade since it spawns a process,
        // but we can test the backup path logic
        let backup_path = original_binary.with_extension("backup");
        assert_eq!(
            backup_path,
            temp_dir.path().join("ctoolbox.backup"),
            "Backup path should use .backup extension"
        );

        // Simulate what start_atomic_upgrade does (without spawning)
        fs::copy(&original_binary, &backup_path).unwrap();
        fs::copy(&new_binary, &original_binary).unwrap();

        // Verify backup was created
        assert!(backup_path.exists(), "Backup should exist");
        let backup_content = fs::read_to_string(&backup_path).unwrap();
        assert_eq!(backup_content, "original binary content v1.0");

        // Verify new binary is in place
        let current_content = fs::read_to_string(&original_binary).unwrap();
        assert_eq!(current_content, "new binary content v2.0");
    }

    #[crate::ctb_test]
    fn test_rollback_restores_original() {
        let temp_dir = TempDir::new().unwrap();
        let target_path = temp_dir.path().join("ctoolbox");
        let backup_path = temp_dir.path().join("ctoolbox.backup");

        // Set up initial state: "bad" binary in place, "good" backup exists
        fs::write(&target_path, b"bad binary that crashes").unwrap();
        fs::write(&backup_path, b"good original binary").unwrap();

        // Simulate rollback
        restore_from_backup(&backup_path, &target_path).unwrap();

        // Verify original is restored
        let content = fs::read_to_string(&target_path).unwrap();
        assert_eq!(content, "good original binary");

        // Verify backup is cleaned up
        assert!(
            !backup_path.exists(),
            "Backup should be deleted after restore"
        );
    }

    #[crate::ctb_test]
    fn test_upgrade_preserves_executable_permissions() {
        let temp_dir = TempDir::new().unwrap();
        let target_path = temp_dir.path().join("ctoolbox");
        let backup_path = temp_dir.path().join("ctoolbox.backup");

        // Create backup and target
        fs::write(&backup_path, b"backup content").unwrap();
        fs::write(&target_path, b"target content").unwrap();

        // Restore from backup
        restore_from_backup(&backup_path, &target_path).unwrap();

        // On Unix, verify executable permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::metadata(&target_path).unwrap().permissions();
            assert!(
                perms.mode() & 0o111 != 0,
                "Binary should have executable permissions"
            );
        }
    }

    #[crate::ctb_test]
    fn test_atomic_upgrade_simulation() {
        // This test simulates the full atomic upgrade dance without actually
        // spawning processes. It verifies the file operations work correctly.
        let temp_dir = TempDir::new().unwrap();

        let original_binary = temp_dir.path().join("ctoolbox");
        let new_binary = temp_dir.path().join("ctoolbox-update");
        let backup_path = original_binary.with_extension("backup");

        // Phase 1: Initial state
        fs::write(&original_binary, b"#!/bin/sh\necho v1.0").unwrap();
        fs::write(&new_binary, b"#!/bin/sh\necho v2.0").unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms =
                fs::metadata(&original_binary).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&original_binary, perms).unwrap();
        }

        // Phase 2: Create backup and install new binary
        fs::copy(&original_binary, &backup_path).unwrap();
        fs::copy(&new_binary, &original_binary).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms =
                fs::metadata(&original_binary).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&original_binary, perms).unwrap();
        }

        // Verify state after "upgrade"
        assert!(backup_path.exists(), "Backup should exist after upgrade");
        let current = fs::read_to_string(&original_binary).unwrap();
        assert!(current.contains("v2.0"), "New version should be installed");

        // Phase 3a: Simulate successful canary - delete backup
        fs::remove_file(&backup_path).unwrap();
        assert!(
            !backup_path.exists(),
            "Backup should be deleted after successful canary"
        );

        // Reset for rollback test
        fs::write(&backup_path, b"#!/bin/sh\necho v1.0").unwrap();
        fs::write(&original_binary, b"#!/bin/sh\nexit 1").unwrap(); // "crashed" binary

        // Phase 3b: Simulate failed canary - restore from backup
        restore_from_backup(&backup_path, &original_binary).unwrap();

        let restored = fs::read_to_string(&original_binary).unwrap();
        assert!(
            restored.contains("v1.0"),
            "Original version should be restored after failed canary"
        );
    }

    #[crate::ctb_test]
    fn test_concurrent_backup_restore_safety() {
        // Test that backup/restore operations are safe when files are being
        // accessed
        let temp_dir = TempDir::new().unwrap();
        let target_path = temp_dir.path().join("ctoolbox");
        let backup_path = temp_dir.path().join("ctoolbox.backup");

        // Create files
        fs::write(&backup_path, b"backup data").unwrap();
        fs::write(&target_path, b"original data").unwrap();

        // Multiple sequential restores should work
        for i in 0..3 {
            // Re-create backup for next iteration
            if i > 0 {
                fs::write(&backup_path, format!("backup data {i}").as_bytes())
                    .unwrap();
            }

            restore_from_backup(&backup_path, &target_path).unwrap();

            // Verify restore worked
            let content = fs::read_to_string(&target_path).unwrap();
            if i == 0 {
                assert_eq!(content, "backup data");
            } else {
                assert_eq!(content, format!("backup data {i}"));
            }
        }
    }

    /// Integration test that spawns a self-re-exec child running the canary
    /// validation flow and verifies it cleans up the backup and exits cleanly.
    #[crate::ctb_test]
    fn test_canary_process_exits_cleanly() {
        const BACKUP_ENV: &str = "CTB_TEST_CANARY_BACKUP_PATH";
        const TARGET_ENV: &str = "CTB_TEST_CANARY_TARGET_PATH";
        if let (Some(backup_path), Some(target_path)) =
            (std::env::var_os(BACKUP_ENV), std::env::var_os(TARGET_ENV))
        {
            run_canary_validation_with_delay(
                Path::new(&backup_path),
                Path::new(&target_path),
                None,
                Duration::from_millis(100),
            )
            .unwrap();
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        let backup_path = temp_dir.path().join("test.backup");
        let target_path = temp_dir
            .path()
            .join(format!("ctoolbox{}", std::env::consts::EXE_SUFFIX));
        fs::write(&backup_path, b"backup content").unwrap();

        let current_exe = std::env::current_exe().unwrap();
        let mut child = Command::new(&current_exe)
            .args([
                "--exact",
                "upgrade::tests::test_canary_process_exits_cleanly",
                "--nocapture",
            ])
            .env(BACKUP_ENV, &backup_path)
            .env(TARGET_ENV, &target_path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to spawn canary process");

        let status = child.wait().expect("Failed to wait on canary");
        assert!(status.success(), "Canary should exit successfully");

        assert!(
            !backup_path.exists(),
            "Backup should be deleted after successful canary"
        );
    }

    /// Integration test that simulates a crashing canary process via
    /// self-re-exec and verifies rollback restores the backup.
    #[crate::ctb_test]
    fn test_canary_crash_leaves_backup() {
        const CRASH_ENV: &str = "CTB_TEST_CANARY_CRASH_CHILD";
        if std::env::var_os(CRASH_ENV).is_some() {
            std::process::exit(1);
        }

        let temp_dir = TempDir::new().unwrap();
        let backup_path = temp_dir.path().join("test.backup");
        fs::write(&backup_path, b"backup content").unwrap();

        let current_exe = std::env::current_exe().unwrap();
        let mut child = Command::new(&current_exe)
            .args([
                "--exact",
                "upgrade::tests::test_canary_crash_leaves_backup",
                "--nocapture",
            ])
            .env(CRASH_ENV, "1")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to spawn crashing canary process");

        let status = child.wait().expect("Failed to wait on canary");
        assert!(!status.success(), "Canary should have crashed");

        assert!(
            backup_path.exists(),
            "Backup should still exist after canary crash"
        );

        let target_path = temp_dir
            .path()
            .join(format!("ctoolbox{}", std::env::consts::EXE_SUFFIX));
        fs::write(&target_path, b"crashed binary").unwrap();

        restore_from_backup(&backup_path, &target_path).unwrap();

        let content = fs::read_to_string(&target_path).unwrap();
        assert_eq!(content, "backup content", "Should restore from backup");
        assert!(
            !backup_path.exists(),
            "Backup should be removed after rollback restore"
        );
    }

    /// Exercises `start_atomic_upgrade` from a running copy of the test
    /// binary when the replacement binary fails to launch, verifying rollback.
    #[crate::ctb_test]
    fn test_start_atomic_upgrade_rollback_on_unexecutable_binary() {
        const NEW_BIN_ENV: &str = "CTB_TEST_UPGRADE_ROLLBACK_NEW_BIN";
        if let Some(new_binary_os) = std::env::var_os(NEW_BIN_ENV) {
            let current_exe = std::env::current_exe().unwrap();
            let err = start_atomic_upgrade(
                Path::new(&new_binary_os),
                &current_exe,
                None,
            )
            .expect_err("Unexecutable replacement should fail to spawn");
            assert!(
                !err.to_string().is_empty(),
                "Error message should describe spawn failure"
            );
            assert!(
                current_exe.exists(),
                "Original binary should be restored after rollback"
            );
            assert!(
                !current_exe.with_extension("backup").exists(),
                "Backup should be consumed by rollback"
            );
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        let target_exe = temp_dir
            .path()
            .join(format!("ctoolbox{}", std::env::consts::EXE_SUFFIX));
        let current_exe = std::env::current_exe().unwrap();
        ctb_utilities::files::replace_executable_file(
            &current_exe,
            &target_exe,
        )
        .unwrap();
        let original_len = fs::metadata(&target_exe).unwrap().len();

        let bad_new_binary = temp_dir.path().join("bad-update");
        // Nonexistent shebang interpreter fails with ENOENT on Unix without
        // triggering execvp's `/bin/sh` fallback; truncated MZ header fails
        // CreateProcessW with ERROR_BAD_EXE_FORMAT on Windows without invoking
        // the legacy 16-bit DOS NTVDM compatibility dialog on Windows 7.
        #[cfg(not(windows))]
        let bad_bytes: &[u8] = b"#!/ctb-nonexistent-interpreter\n";
        #[cfg(windows)]
        let bad_bytes: &[u8] = b"MZ\x00\x00corrupt-pe-image";
        fs::write(&bad_new_binary, bad_bytes).unwrap();

        let mut child = ctb_utilities::spawn_with_etxtbsy_retry(|| {
            Command::new(&target_exe)
                .args([
                    "--exact",
                    "upgrade::tests::test_start_atomic_upgrade_rollback_on_unexecutable_binary",
                    "--nocapture",
                ])
                .env(NEW_BIN_ENV, &bad_new_binary)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
        })
        .expect("Failed to spawn copied test executable");

        let status = child.wait().expect("Failed to wait on child test");
        assert!(status.success(), "Rollback verification child should pass");
        assert_eq!(
            fs::metadata(&target_exe).unwrap().len(),
            original_len,
            "Original binary size should be restored after rollback"
        );
        assert!(
            !target_exe.with_extension("backup").exists(),
            "Backup file should not remain after rollback"
        );
    }
}
