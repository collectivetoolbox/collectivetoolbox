//! Background update checking for ctoolbox.
//!
//! This module handles automatic update checking at startup:
//! - Generates a random time-of-day on first run and stores it in `pc_settings`
//! - Spawns a detached task that waits until the scheduled time (or runs
//!   immediately if past) and checks for updates
//! - Stores update status in memory accessible by other parts of the process

#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

use std::path::PathBuf;
use std::sync::LazyLock;
use std::time::Duration;

use chrono::{Local, Timelike, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tokio::time::timeout;

use ctb_installer::download::{
    ChunkDownloader, current_platform, no_progress_callback,
};
use ctb_utilities::json::maybe_value::MaybeOption;
use ctb_utilities::pc_settings::{PcSettingBoolKey, PcSettings};
use ctb_utilities::storage::get_storage_dir;

/// Seconds in a day (24 hours).
const SECONDS_IN_DAY: u32 = 86400;

/// Global update status stored in memory.
static UPDATE_STATUS: LazyLock<RwLock<UpdateStatus>> =
    LazyLock::new(|| RwLock::new(UpdateStatus::new()));

#[derive(Debug, Clone)]
struct BuildIdentity {
    version: semver::Version,
    build_date: chrono::DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct ServerUpdateStatusResponse {
    is_newer: bool,
    server_version: String,
    server_build_date: String,
}

/// Status of available updates.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateStatus {
    /// Whether an update is available.
    pub available: bool,
    /// The new version string (if available).
    pub version: Option<String>,
    /// URL to release notes (if available).
    pub release_notes_url: Option<String>,
    /// Last time we checked for updates.
    pub last_check: Option<chrono::DateTime<Utc>>,
    /// Error message if the last check failed.
    pub last_error: Option<String>,
    /// Whether automatic restart is enabled (no user prompting).
    /// This is true when `serve_public_web_site_only` is on and the domain
    /// is not the official collectivetoolbox.com domain.
    #[serde(default)]
    pub auto_restart_enabled: bool,
}

impl UpdateStatus {
    /// Creates a new empty update status.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Record of a pending update that has been fully downloaded and is ready to
/// install. This is written to disk so it persists across restarts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingUpdate {
    /// Version that will be installed.
    pub version: String,
    /// Path to the downloaded binary.
    pub binary_path: PathBuf,
    /// Timestamp when the download completed.
    pub download_completed: chrono::DateTime<Utc>,
}

impl PendingUpdate {
    /// Path to the pending update file in the storage directory.
    fn file_path() -> Result<PathBuf> {
        Ok(get_storage_dir()?.join("pending_update.json"))
    }

    /// Saves this pending update record to disk.
    ///
    /// # Errors
    /// Returns an error if the file cannot be written.
    pub fn save(&self) -> Result<()> {
        let path = Self::file_path()?;
        let json = serde_json::to_string_pretty(self)
            .context("Failed to serialize pending update")?;
        std::fs::write(&path, json).with_context(|| {
            format!("Failed to write pending update to {path:?}")
        })
    }

    /// Loads a pending update record from disk, if one exists.
    ///
    /// # Returns
    /// `Ok(Some(pending))` if a valid pending update exists
    /// `Ok(None)` if no pending update file exists
    ///
    /// # Errors
    /// Returns an error if the file exists but cannot be read or parsed.
    pub fn load() -> Result<Option<Self>> {
        let path = Self::file_path()?;
        if !path.exists() {
            return Ok(None);
        }

        let json = std::fs::read_to_string(&path).with_context(|| {
            format!("Failed to read pending update from {path:?}")
        })?;
        let pending: Self = serde_json::from_str(&json)
            .context("Failed to parse pending update JSON")?;

        // Verify the binary still exists
        if !pending.binary_path.exists() {
            // Binary was deleted - remove the stale record
            Self::clear()?;
            return Ok(None);
        }

        Ok(Some(pending))
    }

    /// Removes the pending update record from disk.
    ///
    /// # Errors
    /// Returns an error if the file cannot be deleted.
    pub fn clear() -> Result<()> {
        let path = Self::file_path()?;
        if path.exists() {
            std::fs::remove_file(&path).with_context(|| {
                format!("Failed to remove pending update file {path:?}")
            })?;
        }
        Ok(())
    }
}

/// Gets the current update status as a JSON string.
///
/// # Errors
/// Returns an error if serialization fails.
#[ipc_method]
pub async fn get_update_status() -> Result<String> {
    let status = UPDATE_STATUS.read().await;
    serde_json::to_string_pretty(&*status)
        .context("Failed to serialize update status")
}

/// Gets the update check time from settings, generating one if not present.
///
/// The time is stored as seconds since midnight (0-86400).
///
/// # Errors
/// Returns an error if settings cannot be loaded or saved.
pub fn get_or_generate_update_check_time() -> Result<u32> {
    let settings = PcSettings::load()?;

    // Check if we already have a time set
    if let MaybeOption::Value(time) = settings.update_check_time {
        return Ok(time);
    }

    // Generate a random time between 0 and 86400 seconds (24 hours)
    let mut rng = rand::rng();
    let random_time: u32 = rng.random_range(0..SECONDS_IN_DAY);

    // Save to settings
    PcSettings::apply_patch(PcSettings {
        update_check_time: MaybeOption::Value(random_time),
        ..Default::default()
    })?;

    log_fmt!(
        "Generated random update check time: {:02}:{:02}:{:02}",
        random_time / 3600,
        (random_time % 3600) / 60,
        random_time % 60
    );

    Ok(random_time)
}

/// Resolves the version that should be used for update comparisons.
///
/// Prefers the installed version from `installation.json`, but falls back to
/// the currently running build version so web UI update checks still work when
/// ctoolbox is running without a persisted installation record.
fn parse_build_date(build_date: &str) -> Result<chrono::DateTime<Utc>> {
    chrono::DateTime::parse_from_rfc3339(build_date)
        .map(|parsed| parsed.with_timezone(&Utc))
        .with_context(|| {
            format!("Failed to parse build timestamp for update checks: {build_date}")
        })
}

fn current_build_identity_for_update_checks() -> Result<BuildIdentity> {
    let current_build = build_info();
    Ok(BuildIdentity {
        version: semver::Version::parse(&current_build.version).with_context(
            || {
                format!(
                    "Failed to parse running build version for update checks: {}",
                    current_build.version
                )
            },
        )?,
        build_date: parse_build_date(&current_build.build_date)?,
    })
}

async fn query_server_update_status(
    server_url: &str,
    current: &BuildIdentity,
) -> Result<ServerUpdateStatusResponse> {
    let api_url =
        format!("{}/api/update-status", server_url.trim_end_matches('/'));
    let current_version = current.version.to_string();
    let current_build_date = current.build_date.to_rfc3339();

    let mut url = reqwest::Url::parse(&api_url)?;
    url.query_pairs_mut()
        .append_pair("version", &current_version)
        .append_pair("build_date", &current_build_date);

    let url_str = url.to_string();
    let body_bytes =
        utilities::https::get_success_with_backoff(&url_str, 10).await?;
    let body = String::from_utf8(body_bytes)
        .context("Failed to decode response body as UTF-8")?;

    serde_json::from_str(&body)
        .context("Failed to parse update-status API response")
}

/// Calculates how many seconds to wait until the scheduled update check time.
///
/// If the scheduled time has already passed today, returns 0 (check immediately).
fn seconds_until_check_time(scheduled_seconds: u32) -> u64 {
    let now = Local::now();
    let current_seconds = u32::try_from(now.num_seconds_from_midnight())
        .unwrap_or(scheduled_seconds);

    if current_seconds >= scheduled_seconds {
        // Time has passed today, check immediately
        0
    } else {
        // Wait until the scheduled time
        u64::from(scheduled_seconds.saturating_sub(current_seconds))
    }
}

/// Performs the actual update check.
///
/// # Arguments
/// - `server_url`: URL of the update server
/// - `auto_restart_enabled`: If true, the update status will flag that the
///   workspace should restart automatically without prompting
async fn perform_update_check(server_url: &str, auto_restart_enabled: bool) {
    log!("Checking for updates...");

    let current_build = match current_build_identity_for_update_checks() {
        Ok(build) => build,
        Err(e) => {
            let status = UpdateStatus {
                last_check: Some(Utc::now()),
                last_error: Some(format!(
                    "Failed to determine current version for update check: {e}"
                )),
                ..Default::default()
            };
            *UPDATE_STATUS.write().await = status.clone();
            log_fmt!(
                "Update check skipped: {}",
                status.last_error.as_ref().unwrap_or(&String::new())
            );
            return;
        }
    };

    let api_status =
        match query_server_update_status(server_url, &current_build).await {
            Ok(status) => status,
            Err(e) => {
                let status = UpdateStatus {
                    last_check: Some(Utc::now()),
                    last_error: Some(format!(
                        "Update-status API preflight failed: {e}"
                    )),
                    ..Default::default()
                };
                *UPDATE_STATUS.write().await = status.clone();
                warn_fmt!(
                    "Update-status API preflight failed: {}",
                    status.last_error.as_ref().unwrap_or(&String::new())
                );
                return;
            }
        };

    if !api_status.is_newer {
        let status = UpdateStatus {
            available: false,
            version: None,
            release_notes_url: None,
            last_check: Some(Utc::now()),
            last_error: None,
            auto_restart_enabled: false,
        };
        *UPDATE_STATUS.write().await = status;
        log_fmt!(
            "No update available (current: {} @ {})",
            current_build.version,
            current_build.build_date
        );
        return;
    }

    // Download latest manifest from server
    let downloader =
        match ChunkDownloader::new(server_url, no_progress_callback()) {
            Ok(d) => d,
            Err(e) => {
                let status = UpdateStatus {
                    last_check: Some(Utc::now()),
                    last_error: Some(format!(
                        "Failed to create downloader: {e}"
                    )),
                    ..Default::default()
                };
                *UPDATE_STATUS.write().await = status.clone();
                warn_fmt!(
                    "Update check failed: {}",
                    status.last_error.as_ref().unwrap_or(&String::new())
                );
                return;
            }
        };

    let manifest = match downloader
        .download_manifest(&current_platform(), None)
        .await
    {
        Ok(m) => m,
        Err(e) => {
            let status = UpdateStatus {
                available: true,
                version: Some(api_status.server_version.clone()),
                release_notes_url: Some(format!(
                    "{server_url}/releases/notes/{}",
                    api_status.server_version
                )),
                last_check: Some(Utc::now()),
                last_error: Some(format!(
                    "Failed to download manifest after API reported newer build: {e}"
                )),
                auto_restart_enabled,
            };
            *UPDATE_STATUS.write().await = status.clone();
            warn_fmt!(
                "Manifest download failed after API reported newer build: {}",
                status.last_error.as_ref().unwrap_or(&String::new())
            );
            return;
        }
    };

    let latest_version = manifest.ctoolbox_version.clone();
    let available = api_status.is_newer;

    let status = UpdateStatus {
        available,
        version: if available {
            Some(latest_version.to_string())
        } else {
            None
        },
        // Build release notes URL from server URL
        release_notes_url: if available {
            Some(format!("{server_url}/releases/notes/{latest_version}"))
        } else {
            None
        },
        last_check: Some(Utc::now()),
        last_error: None,
        auto_restart_enabled: available && auto_restart_enabled,
    };

    *UPDATE_STATUS.write().await = status.clone();

    if available {
        log_fmt!(
            "Update available: {} ({}) -> {} ({})",
            current_build.version,
            current_build.build_date,
            latest_version,
            manifest.date
        );
        if auto_restart_enabled {
            log!(
                "Auto-restart is enabled - update will be applied automatically"
            );
        }
    } else {
        log_fmt!(
            "No update available (current: {} @ {})",
            current_build.version,
            current_build.build_date
        );
    }
}

/// Spawns the background update checker task.
///
/// This task:
/// 1. Waits until the scheduled time (or runs immediately if past)
/// 2. Checks for updates from the server
/// 3. Updates the global status
///
/// If `serve_public_web_site_only` is enabled and the domain is the official
/// collectivetoolbox.com domain, automatic updates are skipped (deployments are
/// handled separately).
///
/// If `serve_public_web_site_only` is enabled and the domain is NOT the official
/// domain, the `auto_restart_enabled` flag is set to skip interactive prompts.
///
/// # Arguments
/// - `server_url`: URL of the update server
///
/// # Errors
/// Returns an error if the scheduled time cannot be determined.
pub fn spawn_update_checker(server_url: String) -> Result<()> {
    // Check if serve_public_web_site_only mode affects update behavior
    let settings = PcSettings::load()?;
    let serve_public_only =
        settings.get_bool(&PcSettingBoolKey::ServePublicWebSiteOnly);
    let is_ctb = environment::is_official_public_website();

    // If serve_public_web_site_only is on AND domain is CTB, skip updates
    // (deployments are handled by a separate deploy script)
    if serve_public_only && is_ctb {
        log!("Automatic updates disabled for official CTB domain deployment");
        return Ok(());
    }

    // If serve_public_web_site_only is on AND domain is NOT CTB,
    // enable auto-restart (skip interactive prompts)
    let auto_restart_enabled = serve_public_only && !is_ctb;
    if auto_restart_enabled {
        log!("Auto-restart enabled: updates will apply without prompting");
    }

    let scheduled_time = get_or_generate_update_check_time()?;
    let wait_seconds = seconds_until_check_time(scheduled_time);

    log_fmt!(
        "Update checker scheduled for {:02}:{:02}:{:02} (in {} seconds)",
        scheduled_time / 3600,
        (scheduled_time % 3600) / 60,
        scheduled_time % 60,
        wait_seconds
    );

    tokio::spawn(async move {
        // Wait until scheduled time
        if wait_seconds > 0 {
            tokio::time::sleep(std::time::Duration::from_secs(wait_seconds))
                .await;
        }

        // Perform the update check
        perform_update_check(&server_url, auto_restart_enabled).await;
    });

    Ok(())
}

/// Timeout for the startup update check in seconds.
const STARTUP_CHECK_TIMEOUT_SECS: u64 = 15;

/// Result of the startup update check.
#[derive(Debug)]
pub enum StartupUpdateResult {
    /// No pending update was found.
    NoPendingUpdate,
    /// A pending update was found and the upgrade process was started.
    /// The workspace should exit to allow the canary process to take over.
    UpgradeStarted,
    /// The update check timed out. Startup should continue normally.
    TimedOut,
    /// An error occurred during the check.
    Error(String),
}

/// Checks for a pending downloaded update at startup and applies it if ready.
///
/// This function:
/// 1. Checks if there's a `PendingUpdate` record indicating a fully downloaded
///    update
/// 2. If found, starts the atomic upgrade process
/// 3. If not found, quickly checks for updates (with a 15-second timeout)
/// 4. Returns immediately if the timeout is exceeded
///
/// This should be called early in workspace startup. If `UpgradeStarted` is
/// returned, the process should exit to allow the canary upgrade to proceed.
///
/// # Returns
/// - `NoPendingUpdate`: No update ready, continue startup
/// - `UpgradeStarted`: Upgrade initiated, should exit
/// - `TimedOut`: Check took too long, continue startup
/// - `Error`: Something went wrong (logged, continue startup)
pub async fn check_startup_update() -> StartupUpdateResult {
    // First, check for an already-downloaded pending update
    match PendingUpdate::load() {
        Ok(Some(pending)) => {
            log_fmt!(
                "Found pending update to version {} downloaded at {}",
                pending.version,
                pending.download_completed
            );

            // Get the current executable path
            let current_exe = match std::env::current_exe() {
                Ok(p) => p,
                Err(e) => {
                    return StartupUpdateResult::Error(format!(
                        "Failed to get current executable path: {e}"
                    ));
                }
            };

            // Start the atomic upgrade
            if let Err(e) = ctb_installer::upgrade::start_atomic_upgrade(
                &pending.binary_path,
                &current_exe,
                None,
            ) {
                // Clear the pending update since it failed
                let _ = PendingUpdate::clear();
                return StartupUpdateResult::Error(format!(
                    "Failed to start upgrade from pending update: {e}"
                ));
            }

            // Clear the pending update record (canary process will handle the
            // rest)
            let _ = PendingUpdate::clear();

            return StartupUpdateResult::UpgradeStarted;
        }
        Ok(None) => {
            // No pending update, continue to quick check
        }
        Err(e) => {
            warn_fmt!("Error loading pending update record: {e}");
            // Continue to quick check anyway
        }
    }

    // No pending update - do a quick check with timeout
    // This is a fast "is there an update available?" check, not a full download
    let check_future = async { quick_update_check().await };

    if let Ok(result) = timeout(
        Duration::from_secs(STARTUP_CHECK_TIMEOUT_SECS),
        check_future,
    )
    .await
    {
        result
    } else {
        log!("Startup update check timed out, continuing startup");
        StartupUpdateResult::TimedOut
    }
}

/// Performs a quick update check without downloading.
///
/// This just checks if an update is available and logs it. Full download and
/// installation happens via the background checker or manual update command.
async fn quick_update_check() -> StartupUpdateResult {
    let current_build = match current_build_identity_for_update_checks() {
        Ok(build) => build,
        Err(_) => return StartupUpdateResult::NoPendingUpdate,
    };

    // Get server URL from settings
    let server_url = ctb_utilities::pc_settings::get_str_setting(
        ctb_utilities::pc_settings::PcSettingStrKey::ServerUrl,
    )
    .unwrap_or_else(|| default_url());

    let Ok(api_status) =
        query_server_update_status(&server_url, &current_build).await
    else {
        return StartupUpdateResult::NoPendingUpdate;
    };

    if !api_status.is_newer {
        return StartupUpdateResult::NoPendingUpdate;
    }

    // Quick manifest check
    let downloader =
        match ChunkDownloader::new(&server_url, no_progress_callback()) {
            Ok(d) => d,
            Err(_) => return StartupUpdateResult::NoPendingUpdate,
        };

    let manifest = if let Ok(m) = downloader
        .download_manifest(&current_platform(), None)
        .await
    {
        m
    } else {
        log_fmt!(
            "Update available at startup: {} ({}) -> {} ({})",
            current_build.version,
            current_build.build_date,
            api_status.server_version,
            api_status.server_build_date
        );
        return StartupUpdateResult::NoPendingUpdate;
    };

    log_fmt!(
        "Update available at startup: {} ({}) -> {} ({})",
        current_build.version,
        current_build.build_date,
        manifest.ctoolbox_version,
        manifest.date
    );

    StartupUpdateResult::NoPendingUpdate
}

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use super::*;

    #[crate::ctb_test]
    fn test_get_or_generate_update_check_time() -> Result<()> {
        // First call should generate and save a time
        let time1 = get_or_generate_update_check_time()?;
        assert!(time1 < SECONDS_IN_DAY);

        // Second call should return the same time
        let time2 = get_or_generate_update_check_time()?;
        assert_eq!(time1, time2);

        Ok(())
    }

    #[crate::ctb_test]
    fn test_seconds_until_check_time() {
        // Get current seconds since midnight
        let now = Local::now();
        let current_seconds = u32::try_from(now.num_seconds_from_midnight())
            .expect("current seconds should fit in u32");

        // If we schedule for "now", it should be 0 or very small
        let wait = seconds_until_check_time(current_seconds);
        assert_eq!(wait, 0);

        // If we schedule for the past, it should be 0
        if current_seconds > 100 {
            let wait = seconds_until_check_time(current_seconds.saturating_sub(100));
            assert_eq!(wait, 0);
        }

        // If we schedule for the future, it should be positive
        if current_seconds < SECONDS_IN_DAY - 100 {
            let wait = seconds_until_check_time(current_seconds.saturating_add(100));
            assert!(wait > 0);
            assert!(wait <= 100);
        }
    }
}
