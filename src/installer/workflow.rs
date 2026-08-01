//! Shared installer workflow for manifest loading and real installation.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::collections::HashSet;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::download::{
    CancellationFlag, ChunkDownloader, DownloadEvent,
    INSTALL_CANCELLED_MESSAGE, ProgressCallback, current_platform,
    is_cancellation_requested, no_progress_callback,
};
use crate::feature::{Feature, features_from_manifest};
use crate::install::{
    InstallConfig, InstallationRecord, add_to_path, create_desktop_entry,
    create_desktop_icon, install_file, rollback_installation,
};
use crate::manifest::{FileEntry, ReleaseManifest};
use crate::signing::{public_key_from_base64, verify_manifest};
use ctb_utilities::pc_settings::{self, PcSettingStrKey};

/// Loads the latest manifest and converts it into installer features.
pub fn load_manifest_and_features(
    lang_code: &str,
) -> Result<(ReleaseManifest, Vec<Feature>)> {
    let runtime = build_runtime()?;
    let server_url = resolved_server_url();

    let manifest = runtime.block_on(async {
        let downloader =
            ChunkDownloader::new(&server_url, no_progress_callback())?;
        let manifest = if let Some(offline_path) =
            crate::download::find_offline_manifest()
        {
            let data = std::fs::read(&offline_path).with_context(|| {
                format!(
                    "Failed to read offline manifest at {}",
                    offline_path.display()
                )
            })?;
            serde_json::from_slice::<ReleaseManifest>(&data).with_context(
                || {
                    format!(
                        "Failed to parse offline manifest at {}",
                        offline_path.display()
                    )
                },
            )?
        } else {
            downloader
                .download_manifest(&current_platform(), None)
                .await
                .with_context(|| {
                    format!(
                        "Failed to download release manifest from {server_url}"
                    )
                })?
        };

        verify_downloaded_manifest(&downloader, &manifest).await?;
        Ok::<ReleaseManifest, anyhow::Error>(manifest)
    })?;

    let features = features_from_manifest(&manifest, lang_code);
    if features.is_empty() {
        bail!("Release manifest does not contain any installer features");
    }

    Ok((manifest, features))
}

/// Runs the real installation flow using an already-loaded manifest.
pub fn run_installation(
    config: &InstallConfig,
    manifest: &ReleaseManifest,
    progress_callback: ProgressCallback,
    cancel_flag: Option<&CancellationFlag>,
) -> Result<InstallationRecord> {
    let selected_features =
        resolve_selected_feature_ids(manifest, &config.selected_features);
    let files = selected_manifest_files(manifest, &selected_features);
    if files.is_empty() {
        bail!("No files were selected for installation");
    }

    progress_callback(DownloadEvent::InstallPlan {
        total_files: files.len(),
    });

    let runtime = build_runtime()?;
    let server_url = resolved_server_url();
    let cache_dir = temp_work_dir("cache")?;
    let stage_dir = temp_work_dir("stage")?;

    let mut record = InstallationRecord::new(
        manifest.ctoolbox_version.clone(),
        config.clone(),
    );

    let cache_dir_clone = cache_dir.clone();
    let stage_dir_clone = stage_dir.clone();
    let progress_callback_clone = progress_callback.clone();

    let result = runtime.block_on({
        let record = &mut record;
        async move {
            let downloader = ChunkDownloader::new(
                &server_url,
                progress_callback_clone.clone(),
            )?;
            verify_downloaded_manifest(&downloader, manifest).await?;

            for entry in &files {
                if is_cancellation_requested(cancel_flag) {
                    progress_callback_clone(DownloadEvent::InstallCancelled {
                        completed_files: record.installed_files.len(),
                    });
                    bail!(INSTALL_CANCELLED_MESSAGE);
                }

                let staged_path = stage_dir_clone.join(&entry.path);
                let assembled_path = downloader
                    .download_file(
                        entry,
                        &cache_dir_clone,
                        &staged_path,
                        cancel_flag,
                    )
                    .await
                    .with_context(|| {
                        format!(
                            "Failed to download installation file {}",
                            entry.path
                        )
                    })?;

                let installed_path =
                    install_file(entry, &assembled_path, config).with_context(
                        || format!("Failed to install {}", entry.path),
                    )?;

                let recorded_path = if let Ok(path) =
                    installed_path.strip_prefix(&config.install_dir)
                {
                    path.to_string_lossy().to_string()
                } else {
                    installed_path.to_string_lossy().to_string()
                };
                record.add_file(recorded_path);
            }

            if config.add_to_start_menu {
                let _ = create_desktop_entry(config)?;
            }
            if config.add_desktop_shortcut {
                let _ = create_desktop_icon(config)?;
            }
            if config.add_to_path {
                add_to_path(config)?;
            }

            record.save()?;
            Ok::<(), anyhow::Error>(())
        }
    });

    let _ = std::fs::remove_dir_all(&cache_dir);
    let _ = std::fs::remove_dir_all(&stage_dir);

    if let Err(err) = result {
        if let Err(rollback_err) = rollback_installation(&record) {
            warn_fmt!("Failed to rollback installation: {}", rollback_err);
        }
        return Err(err);
    }

    progress_callback(DownloadEvent::InstallCompleted {
        installed_files: record.installed_files.len(),
    });
    Ok(record)
}

fn build_runtime() -> Result<tokio::runtime::Runtime> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("Failed to create installer async runtime")
}

fn resolved_server_url() -> String {
    pc_settings::get_str_setting(PcSettingStrKey::ServerUrl)
        .unwrap_or_else(default_url)
}

async fn verify_downloaded_manifest(
    downloader: &ChunkDownloader,
    manifest: &ReleaseManifest,
) -> Result<()> {
    let public_key = if let Some(encoded) =
        pc_settings::get_str_setting(PcSettingStrKey::ReleasePublicKey)
    {
        Some(
            public_key_from_base64(&encoded)
                .context("Failed to decode configured release public key")?,
        )
    } else {
        match downloader.download_public_key().await {
            Ok(key) => Some(key),
            Err(err) => {
                if crate::download::find_offline_manifest().is_some() {
                    warn_fmt!(
                        "Offline installation: could not fetch release verification public key from server ({err})"
                    );
                    downloader.emit(DownloadEvent::Warning {
                        message: format!(
                            "Offline installation: could not fetch public key to verify manifest ({err})"
                        ),
                    });
                    None
                } else {
                    return Err(err).context(
                        "Failed to download release verification public key",
                    );
                }
            }
        }
    };

    if let Some(public_key) = public_key {
        let verified = verify_manifest(manifest, &public_key)
            .context("Failed to verify release manifest signature")?;
        if !verified {
            bail!("Release manifest signature verification failed");
        }
    }

    Ok(())
}

fn resolve_selected_feature_ids(
    manifest: &ReleaseManifest,
    requested_features: &HashSet<String>,
) -> HashSet<String> {
    let mut selected = if requested_features.is_empty() {
        manifest
            .files
            .iter()
            .filter(|file| !file.unavailable)
            .map(|file| file.feature_id.clone())
            .collect()
    } else {
        requested_features.clone()
    };

    let mut changed = true;
    while changed {
        changed = false;
        for file in &manifest.files {
            if file.required && selected.insert(file.feature_id.clone()) {
                changed = true;
            }

            if selected.contains(&file.feature_id) {
                for dependency in &file.requires {
                    if selected.insert(dependency.clone()) {
                        changed = true;
                    }
                }
            }
        }
    }

    selected
}

fn selected_manifest_files(
    manifest: &ReleaseManifest,
    selected_features: &HashSet<String>,
) -> Vec<FileEntry> {
    manifest
        .files
        .iter()
        .filter(|file| {
            !file.unavailable && selected_features.contains(&file.feature_id)
        })
        .cloned()
        .collect()
}

fn temp_work_dir(kind: &str) -> Result<PathBuf> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("System clock is before UNIX_EPOCH")?
        .as_millis();
    let dir = std::env::temp_dir().join(format!(
        "ctoolbox-installer-{}-{}-{}",
        kind,
        std::process::id(),
        timestamp
    ));
    std::fs::create_dir_all(&dir).with_context(|| {
        format!("Failed to create temp dir {}", dir.display())
    })?;
    Ok(dir)
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
    use crate::manifest::{ChunkInfo, FileEntry, Platform};
    use chrono::Utc;

    fn test_manifest() -> ReleaseManifest {
        let mut manifest = ReleaseManifest::new(
            semver::Version::new(1, 0, 0),
            Platform::LinuxX64,
            Utc::now(),
        );

        let mut core = FileEntry::new(
            "bin/ctoolbox".to_string(),
            "a".repeat(64),
            "core".to_string(),
        )
        .with_required(true)
        .with_file_size(10);
        core.add_chunk(ChunkInfo::new("b".repeat(64), 0, 10));
        manifest.add_file(core);

        let mut docs = FileEntry::new(
            "share/docs.txt".to_string(),
            "c".repeat(64),
            "docs".to_string(),
        )
        .with_requires("core")
        .with_file_size(5);
        docs.add_chunk(ChunkInfo::new("d".repeat(64), 0, 5));
        manifest.add_file(docs);

        manifest
    }

    #[crate::ctb_test]
    fn selected_features_include_required_dependencies() {
        let manifest = test_manifest();
        let requested = HashSet::from(["docs".to_string()]);

        let selected = resolve_selected_feature_ids(&manifest, &requested);

        assert!(selected.contains("docs"));
        assert!(selected.contains("core"));
    }

    #[crate::ctb_test]
    fn empty_requested_features_install_all_available_features() {
        let manifest = test_manifest();
        let selected = resolve_selected_feature_ids(&manifest, &HashSet::new());

        assert!(selected.contains("core"));
        assert!(selected.contains("docs"));
    }
}
