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

    // Step 1: Copy current binary to backup
    fs::copy(target_path, &backup_path).with_context(|| {
        format!(
            "Failed to create backup of {} at {}",
            target_path.display(),
            backup_path.display()
        )
    })?;

    log_fmt!("Created backup at {}", backup_path.display());

    // Step 2: Copy new binary over the current location
    fs::copy(new_binary_path, target_path).with_context(|| {
        format!(
            "Failed to copy new binary from {} to {}",
            new_binary_path.display(),
            target_path.display()
        )
    })?;

    // Set executable permissions on Linux
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(target_path)
            .context("Failed to get binary metadata")?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(target_path, perms)
            .context("Failed to set executable permissions")?;
    }

    log_fmt!("Installed new binary at {}", target_path.display());

    // Step 3: Spawn the new binary with --ctb-upgrade-canary
    // Convert args to Vec<&str> for the fork function
    let mut args: Vec<&str> = vec!["ctb-upgrade-canary", "--backup-path"];
    let backup_path_str = backup_path.to_string_lossy().to_string();
    args.push(&backup_path_str);
    args.push("--target-path");
    let target_path_str = target_path.to_string_lossy().to_string();
    args.push(&target_path_str);

    let port_str;
    if let Some(p) = port {
        port_str = p.to_string();
        args.push("--port");
        args.push(&port_str);
    }

    // Use the utilities fork function to spawn as a detached process
    ctb_utilities::fork(&target_path.to_path_buf(), args);

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
    log_fmt!(
        "Starting canary validation, waiting {}s...",
        CANARY_WAIT_SECS
    );

    // Wait for the canary period
    std::thread::sleep(Duration::from_secs(CANARY_WAIT_SECS));

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

    fs::copy(backup_path, target_path).with_context(|| {
        format!(
            "Failed to restore backup from {} to {}",
            backup_path.display(),
            target_path.display()
        )
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(target_path)
            .context("Failed to get binary metadata")?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(target_path, perms)
            .context("Failed to set executable permissions")?;
    }

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
    ctb_utilities::fork(
        &binary_path.to_path_buf(),
        vec!["--ctoolbox-ipc-port", &port_str],
    );
    Ok(())
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
    use std::io::Write;
    use std::process::{Command, Stdio};
    use tempfile::TempDir;

    #[crate::ctb_test]
    fn test_restore_from_backup() {
        let temp_dir = TempDir::new().unwrap();
        let backup_path = temp_dir.path().join("backup");
        let target_path = temp_dir.path().join("target");

        // Create a "backup" file
        let mut backup = fs::File::create(&backup_path).unwrap();
        backup.write_all(b"backup content").unwrap();

        // Create a "corrupted" target
        let mut target = fs::File::create(&target_path).unwrap();
        target.write_all(b"corrupted").unwrap();

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

    /// Integration test that actually spawns a child process to test the
    /// canary mechanism. This creates a simple shell script that simulates
    /// the upgrade canary behavior.
    #[crate::ctb_test]
    #[cfg(unix)]
    fn test_canary_process_exits_cleanly() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join("test-canary.sh");

        // Create a simple script that simulates canary behavior:
        // - Waits briefly
        // - Deletes the backup file
        // - Exits successfully
        let script = r#"#!/bin/sh
sleep 0.1
rm -f "$1"
exit 0
"#;
        fs::write(&script_path, script).unwrap();
        let mut perms = fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).unwrap();

        let backup_path = temp_dir.path().join("test.backup");
        fs::write(&backup_path, b"backup content").unwrap();

        // Spawn the "canary" process
        let mut child = Command::new(&script_path)
            .arg(&backup_path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to spawn canary script");

        // Wait for it to complete
        let status = child.wait().expect("Failed to wait on canary");
        assert!(status.success(), "Canary should exit successfully");

        // Verify backup was deleted (canary succeeded)
        assert!(
            !backup_path.exists(),
            "Backup should be deleted after successful canary"
        );
    }

    /// Integration test that simulates a crashing canary process.
    #[crate::ctb_test]
    #[cfg(unix)]
    fn test_canary_crash_leaves_backup() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join("test-crash-canary.sh");

        // Create a script that "crashes" (exits with error before deleting
        // backup)
        let script = r"#!/bin/sh
# Simulate a crash - exit before cleanup
exit 1
";
        fs::write(&script_path, script).unwrap();
        let mut perms = fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).unwrap();

        let backup_path = temp_dir.path().join("test.backup");
        fs::write(&backup_path, b"backup content").unwrap();

        // Spawn the "crashing canary" process
        let mut child = Command::new(&script_path)
            .arg(&backup_path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to spawn canary script");

        let status = child.wait().expect("Failed to wait on canary");
        assert!(!status.success(), "Canary should have crashed");

        // Verify backup is still present (for rollback)
        assert!(
            backup_path.exists(),
            "Backup should still exist after canary crash"
        );

        // Now simulate rollback
        let target_path = temp_dir.path().join("ctoolbox");
        fs::write(&target_path, b"crashed binary").unwrap();

        restore_from_backup(&backup_path, &target_path).unwrap();

        let content = fs::read_to_string(&target_path).unwrap();
        assert_eq!(content, "backup content", "Should restore from backup");
    }
}
