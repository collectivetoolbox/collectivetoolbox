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

//! Developer signing tool for release artifacts.
//!
//! This module implements the `--ctb-dev-sign` command which:
//! - Scans a configurable input directory for release artifacts
//! - Chunks each file using content-defined chunking
//! - Writes chunks to `output_dir/bh/{hash}`
//! - Builds a `ReleaseManifest` with all file entries
//! - Signs the manifest using the dev private key from `pc_settings`
//! - Writes the manifest to `output_dir/ctb-{platform}-{datetime}.json`

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use chrono::Utc;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::chunking::{
    chunk_file, compute_file_sha256_hex, write_chunks_to_directory_compressed,
};
use crate::manifest::{FileEntry, Platform, ReleaseManifest};
use crate::signing::{KeyId, private_key_from_base64, sign_manifest};
use ctb_utilities::pc_settings::{PcSettingStrKey, PcSettings};

/// Default input directory for release artifacts.
fn default_input_dir() -> PathBuf {
    // Reason for fallback: home directory resolution failure falls back to relative current directory "."
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ctb_release")
        .join("input")
}

/// Default output directory for signed releases.
fn default_output_dir() -> PathBuf {
    // Reason for fallback: home directory resolution failure falls back to relative current directory "."
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ctb_release")
        .join("releases")
}

/// Detects the current target.
fn current_platform() -> Platform {
    #[cfg(target_os = "linux")]
    {
        if std::env::consts::ARCH == "x86" {
            Platform::LinuxX86
        } else {
            Platform::LinuxX64
        }
    }
    #[cfg(target_os = "windows")]
    {
        Platform::WindowsX64
    }
    #[cfg(target_os = "macos")]
    {
        match std::env::consts::ARCH {
            "aarch64" => Platform::MacArm64,
            _ => Platform::MacX64,
        }
    }
    #[cfg(not(any(
        target_os = "linux",
        target_os = "windows",
        target_os = "macos"
    )))]
    {
        Platform::LinuxX64 // Fallback
    }
}

/// Parses a platform string into a Platform enum.
fn parse_platform(s: &str) -> Result<Platform> {
    match s.to_lowercase().as_str() {
        "linux" | "linux-x64" | "linux-amd64" => Ok(Platform::LinuxX64),
        "linux-x86" | "linux-i386" | "linux-i686" => Ok(Platform::LinuxX86),
        "windows" | "win" | "windows-x64" | "win-x64" | "windows-amd64" => {
            Ok(Platform::WindowsX64)
        }
        "mac" | "macos" | "darwin" | "mac-x64" | "macos-x64" | "darwin-x64" => {
            Ok(Platform::MacX64)
        }
        "mac-arm64" | "macos-arm64" | "darwin-arm64" | "mac-aarch64" => {
            Ok(Platform::MacArm64)
        }
        _ => {
            bail!(
                "Unknown platform: '{s}'. Use e.g. 'linux-x64', 'linux-x86', 'windows-x64', 'mac-x64', or 'mac-arm64'."
            )
        }
    }
}

/// Summary of the dev-sign operation.
#[derive(Debug)]
pub struct DevSignSummary {
    /// Number of files processed.
    pub files_processed: usize,
    /// Total number of chunks created.
    pub total_chunks: usize,
    /// Path to the output manifest.
    pub manifest_path: PathBuf,
    /// Key ID used for signing.
    pub key_id: String,
}

impl std::fmt::Display for DevSignSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Dev-sign completed successfully:")?;
        writeln!(f, "  Files processed: {}", self.files_processed)?;
        writeln!(f, "  Total chunks: {}", self.total_chunks)?;
        writeln!(f, "  Manifest: {}", self.manifest_path.display())?;
        writeln!(f, "  Signing key ID: {}", self.key_id)?;
        Ok(())
    }
}

/// Configuration for artifact signing.
#[derive(Debug, Clone, Deserialize)]
pub struct ArtifactConfig {
    /// Relative path within install directory.
    pub install_path: String,
    /// Feature ID this artifact belongs to.
    pub feature_id: String,
    /// Human-readable feature names by language code.
    pub feature_names: HashMap<String, String>,
    /// Whether to gzip after installation.
    pub gzip_after_install: bool,
    /// Whether this artifact is required.
    #[serde(default)]
    pub required: bool,
    /// Required feature IDs.
    pub requires: Vec<String>,
}

impl Default for ArtifactConfig {
    fn default() -> Self {
        Self {
            install_path: String::new(),
            feature_id: "core".to_string(),
            feature_names: {
                let mut names = HashMap::new();
                names.insert("en".to_string(), "Core".to_string());
                names
            },
            gzip_after_install: false,
            required: false,
            requires: Vec::new(),
        }
    }
}

/// Gets artifact configuration from a config file or generates defaults.
///
/// Looks for a `.ctb-artifact.json` file alongside the artifact, or uses
/// defaults based on the filename.
fn get_artifact_config(artifact_path: &Path) -> ArtifactConfig {
    // Try to load a sidecar config file
    let config_path = artifact_path.with_extension("ctb-artifact.json");
    if config_path.exists() {
        if let Ok(content) = fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str::<ArtifactConfig>(&content)
            {
                return config;
            }
        }
    }

    // Generate defaults based on filename
    let filename = artifact_path
        .file_name()
        .and_then(|n| n.to_str())
        // Reason for fallback: artifact path without file_name component defaults filename to "unknown"
        .unwrap_or("unknown");

    let install_path = filename.to_string();

    // Determine feature based on filename patterns
    let (feature_id, feature_name, gzip_after_install, required) =
        if filename.contains("icecat") || filename.contains("browser") {
            (
            "browser".to_string(),
            "Web Browser (IceCat) (if deselected, system browser will be used)"
                .to_string(),
            true,
            false,
        )
        } else if filename.contains("dependencies") {
            (
                "dependencies".to_string(),
                "Dependency Source Code".to_string(),
                true,
                false,
            )
        } else if filename == "ctoolbox-installer"
            || std::path::Path::new(filename)
                .file_stem()
                .is_some_and(|stem| stem == "ctoolbox-installer")
        {
            (
                "installer".to_string(),
                "Application Installer (for generating offline installers)"
                    .to_string(),
                false,
                false,
            )
        } else if filename == "ctoolbox.rsrc" {
            (
                "assets".to_string(),
                "Application Assets".to_string(),
                false,
                true,
            )
        } else if filename == "v86_images.rsrc" || filename.contains("v86") {
            (
                "v86".to_string(),
                "v86 Linux Emulation (Guix OS Image)".to_string(),
                false,
                false,
            )
        } else if filename.contains("ctoolbox-src")
            || filename.contains("-src-")
            || filename.contains("src.tar")
        {
            (
                "src".to_string(),
                "Application Source Code".to_string(),
                true,
                false,
            )
        } else if filename == "ctoolbox"
            || std::path::Path::new(filename)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("exe"))
        {
            (
                "core".to_string(),
                "Core Application".to_string(),
                false,
                true,
            )
        } else {
            (
                "assets".to_string(),
                "Additional Application Assets".to_string(),
                false,
                false,
            )
        };

    let mut feature_names = HashMap::new();
    feature_names.insert("en".to_string(), feature_name);

    ArtifactConfig {
        install_path,
        feature_id,
        feature_names,
        gzip_after_install,
        required,
        requires: Vec::new(),
    }
}

/// Scans a directory for release artifacts (non-hidden files).
fn scan_input_directory(input_dir: &Path) -> Result<Vec<PathBuf>> {
    if !input_dir.exists() {
        bail!(
            "Input directory does not exist: {}\n\
            Create it and place release artifacts inside.",
            input_dir.display()
        );
    }

    let mut artifacts = Vec::new();

    for entry in fs::read_dir(input_dir).with_context(|| {
        format!("Failed to read input directory: {}", input_dir.display())
    })? {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        // Skip hidden files and directories
        if path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with('.'))
        {
            continue;
        }

        // Skip directories for now (could recursively scan in future)
        if path.is_file() {
            artifacts.push(path);
        }
    }

    if artifacts.is_empty() {
        bail!(
            "No release artifacts found in: {}\n\
            Place files to sign in this directory.",
            input_dir.display()
        );
    }

    // Sort for deterministic output
    artifacts.sort();

    Ok(artifacts)
}

/// Runs the dev-sign command.
///
/// # Arguments
/// - `input_dir`: Directory containing release artifacts (defaults to
///   `~/ctb_release/input`)
/// - `output_dir`: Directory to write chunks and manifest (defaults to
///   `~/ctb_release/releases`)
/// - `platform`: Target platform string (defaults to current platform)
///
/// # Errors
/// Returns an error if:
/// - No dev signing key is configured in `pc_settings`
/// - Input directory doesn't exist or is empty
/// - Any file cannot be read or chunked
/// - Output cannot be written
pub fn run_dev_sign(
    input_dir: Option<&Path>,
    output_dir: Option<&Path>,
    platform: Option<&str>,
) -> Result<DevSignSummary> {
    // Reason for fallback: unconfigured input directory option falls back to default input directory
    let input_dir = input_dir.map_or_else(default_input_dir, PathBuf::from);
    // Reason for fallback: unconfigured output directory option falls back to default output directory
    let output_dir = output_dir.map_or_else(default_output_dir, PathBuf::from);
    let chunk_dir = output_dir.join("bh");

    // Resolve platform
    let platform = match platform {
        Some(p) => parse_platform(p)?,
        None => current_platform(),
    };

    // Load signing key from pc_settings
    // Reason for fallback: unreadable pc_settings file defaults settings struct to default values
    let settings = PcSettings::load().unwrap_or_default();
    let private_key_b64 = settings
        .get_str(&PcSettingStrKey::DevSigningPrivateKey)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "No dev signing key configured.\n\
                Generate a keypair and add dev_signing_private_key to pc_settings.json.\n\
                You can generate a keypair using `ctoolbox ctb-dev-key-create --write`."
            )
        })?;

    let private_key = private_key_from_base64(&private_key_b64)
        .context("Failed to decode dev signing private key from pc_settings")?;

    let public_key = private_key.public_key();
    let key_id = KeyId::from_public_key(&public_key);

    // Get ctoolbox version from environment or default
    let ctoolbox_version = environment::ctb_version_semver();

    // Create manifest
    let now = Utc::now();
    let mut manifest = ReleaseManifest::new(ctoolbox_version, platform, now);

    // Scan input directory
    let artifacts = scan_input_directory(&input_dir)?;

    // Create chunk directory
    fs::create_dir_all(&chunk_dir).with_context(|| {
        format!("Failed to create chunk directory: {}", chunk_dir.display())
    })?;

    let mut total_chunks: usize = 0;

    // Process each artifact
    for artifact_path in &artifacts {
        log_fmt!(
            "Processing: {}",
            artifact_path
                .file_name()
                // Reason for fallback: artifact path without file_name component defaults to empty OsStr
                .unwrap_or_default()
                .to_string_lossy()
        );

        // Get artifact configuration
        let config = get_artifact_config(artifact_path);

        // Compute file checksum
        let checksum =
            compute_file_sha256_hex(artifact_path).with_context(|| {
                format!("Failed to hash file: {}", artifact_path.display())
            })?;

        // Chunk the file
        let mut chunks = chunk_file(artifact_path).with_context(|| {
            format!("Failed to chunk file: {}", artifact_path.display())
        })?;

        // Write chunks to output directory in compressed form (bh/a0/bc/hash.br)
        write_chunks_to_directory_compressed(&mut chunks, &chunk_dir)
            .with_context(|| {
                format!(
                    "Failed to write chunks for: {}",
                    artifact_path.display()
                )
            })?;

        // Build file entry
        let mut entry = FileEntry::new(
            config.install_path,
            checksum,
            config.feature_id.clone(),
        );
        entry.gzip_after_install = config.gzip_after_install;
        entry.feature_name = config.feature_names;
        entry.required = config.required;
        entry.requires = config.requires;

        // Add chunk info to entry
        for chunk in &chunks {
            entry.add_chunk(chunk.to_chunk_info());
        }

        total_chunks = total_chunks.saturating_add(chunks.len());
        manifest.add_file(entry);
    }

    // Sign the manifest
    let signature = sign_manifest(&manifest, &private_key)?;
    manifest.signature = Some(signature);

    // Write manifest
    let manifest_filename =
        format!("ctb-{}-{}.json", platform, now.format("%Y%m%d-%H%M%S"));
    let manifest_path = output_dir.join(&manifest_filename);

    let manifest_json = serde_json::to_string_pretty(&manifest)
        .context("Failed to serialize manifest to JSON")?;
    fs::write(&manifest_path, &manifest_json).with_context(|| {
        format!("Failed to write manifest: {}", manifest_path.display())
    })?;

    // Write detached signature of the manifest file
    let detached_signature = private_key.sign(manifest_json.as_bytes());
    let sig_filename = format!("{manifest_filename}.sig");
    let sig_path = output_dir.join(&sig_filename);
    fs::write(&sig_path, detached_signature.to_bytes()).with_context(|| {
        format!("Failed to write detached signature: {}", sig_path.display())
    })?;

    Ok(DevSignSummary {
        files_processed: artifacts.len(),
        total_chunks,
        manifest_path,
        key_id: key_id.to_hex(),
    })
}

/// Compresses a file using the deterministic gzip settings used by the web server
/// and computes/returns its SHA256 hex checksum.
pub fn compress_gz_sha256(path: &Path) -> Result<String> {
    use flate2::Compression;
    use flate2::GzBuilder;
    use std::io::Write;

    let bytes = fs::read(path).with_context(|| {
        format!("Failed to read file for compression: {}", path.display())
    })?;

    let mut encoder = GzBuilder::new()
        .mtime(0)
        .write(Vec::new(), Compression::best());
    encoder.write_all(&bytes)?;
    let compressed_bytes = encoder.finish()?;

    let sha = crate::chunking::compute_sha256_hex(&compressed_bytes);
    Ok(sha)
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
    use tempfile::TempDir;

    #[crate::ctb_test]
    fn test_default_directories() {
        let input = default_input_dir();
        let output = default_output_dir();

        assert!(input.to_string_lossy().contains("ctb_release"));
        assert!(input.to_string_lossy().contains("input"));
        assert!(output.to_string_lossy().contains("ctb_release"));
        assert!(output.to_string_lossy().contains("releases"));
    }

    #[crate::ctb_test]
    fn test_parse_platform() {
        assert_eq!(parse_platform("linux-x64").unwrap(), Platform::LinuxX64);
        assert_eq!(parse_platform("linux-x86").unwrap(), Platform::LinuxX86);
        assert_eq!(
            parse_platform("windows-x64").unwrap(),
            Platform::WindowsX64
        );
        assert_eq!(parse_platform("mac-x64").unwrap(), Platform::MacX64);
        assert_eq!(parse_platform("mac-arm64").unwrap(), Platform::MacArm64);

        // Convenience aliases.
        assert_eq!(parse_platform("linux").unwrap(), Platform::LinuxX64);
        assert_eq!(parse_platform("Linux").unwrap(), Platform::LinuxX64);
        assert_eq!(parse_platform("win").unwrap(), Platform::WindowsX64);
        assert_eq!(parse_platform("darwin").unwrap(), Platform::MacX64);
        assert!(parse_platform("unknown").is_err());
    }

    #[crate::ctb_test]
    fn test_scan_empty_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let input_dir = temp_dir.path().join("input");
        fs::create_dir_all(&input_dir)
            .expect("Failed to create input directory");

        let result = scan_input_directory(&input_dir);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No release artifacts")
        );
    }

    #[crate::ctb_test]
    fn test_scan_nonexistent_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let input_dir = temp_dir.path().join("nonexistent");

        let result = scan_input_directory(&input_dir);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }

    #[crate::ctb_test]
    fn test_scan_directory_with_files() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let input_dir = temp_dir.path().join("input");
        fs::create_dir_all(&input_dir)
            .expect("Failed to create input directory");

        // Create test files
        fs::write(input_dir.join("file1.bin"), b"content1").unwrap();
        fs::write(input_dir.join("file2.bin"), b"content2").unwrap();
        fs::write(input_dir.join(".hidden"), b"hidden").unwrap();

        let artifacts = scan_input_directory(&input_dir).unwrap();

        assert_eq!(artifacts.len(), 2);
        assert!(
            artifacts
                .iter()
                .any(|p| p.file_name().unwrap() == "file1.bin")
        );
        assert!(
            artifacts
                .iter()
                .any(|p| p.file_name().unwrap() == "file2.bin")
        );
        // Hidden file should be excluded
        assert!(
            !artifacts
                .iter()
                .any(|p| p.file_name().unwrap() == ".hidden")
        );
    }

    #[crate::ctb_test]
    fn test_get_artifact_config_defaults() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let artifact = temp_dir.path().join("ctoolbox");

        let config = get_artifact_config(&artifact);
        assert_eq!(config.feature_id, "core");
        assert_eq!(
            config.feature_names.get("en"),
            Some(&"Core Application".to_string())
        );
        assert!(!config.gzip_after_install);
        assert!(config.required);
    }

    #[crate::ctb_test]
    fn test_get_artifact_config_browser() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let artifact = temp_dir.path().join("icecat.tar");

        let config = get_artifact_config(&artifact);
        assert_eq!(config.feature_id, "browser");
        assert!(config.gzip_after_install);
        assert!(!config.required);
    }

    #[crate::ctb_test]
    fn test_get_artifact_config_installer() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let artifact = temp_dir.path().join("ctoolbox-installer");

        let config = get_artifact_config(&artifact);
        assert_eq!(config.feature_id, "installer");
        assert_eq!(
            config.feature_names.get("en"),
            Some(
                &"Application Installer (for generating offline installers)"
                    .to_string()
            )
        );
        assert!(!config.required);
    }

    #[crate::ctb_test]
    fn test_get_artifact_config_resource_bundle_required() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let artifact = temp_dir.path().join("ctoolbox.rsrc");

        let config = get_artifact_config(&artifact);
        assert_eq!(config.feature_id, "assets");
        assert_eq!(
            config.feature_names.get("en"),
            Some(&"Application Assets".to_string())
        );
        assert!(config.required);
    }

    #[crate::ctb_test]
    fn test_get_artifact_config_v86() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let artifact = temp_dir.path().join("v86_images.rsrc");

        let config = get_artifact_config(&artifact);
        assert_eq!(config.feature_id, "v86");
        assert_eq!(
            config.feature_names.get("en"),
            Some(&"v86 Linux Emulation (Guix OS Image)".to_string())
        );
        assert!(!config.required);
        assert!(!config.gzip_after_install);
    }

    #[crate::ctb_test]
    fn test_dev_sign_summary_display() {
        let summary = DevSignSummary {
            files_processed: 5,
            total_chunks: 42,
            manifest_path: PathBuf::from("/tmp/test.json"),
            key_id: "abc12345".to_string(),
        };

        let display = format!("{summary}");
        assert!(display.contains("Files processed: 5"));
        assert!(display.contains("Total chunks: 42"));
        assert!(display.contains("abc12345"));
    }
}
