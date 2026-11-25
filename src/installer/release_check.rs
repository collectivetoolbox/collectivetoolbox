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

//! Release verification functionality for server-side validation.
//!
//! This module provides the `verify_release` function used by the
//! `--ctb-dev-release-check` CLI command to verify that an uploaded release
//! is valid before deployment.
//!
//! Verification includes:
//! - Loading and parsing the release manifest
//! - Verifying the Ed25519 signature against the configured public key
//! - Checking that all referenced chunks exist
//! - Verifying each chunk's hash matches its content
//! - Verifying the platform variants for the release

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::chunking::{compressed_chunk_file_path, compute_sha256_hex};
use crate::manifest::ReleaseManifest;
use crate::signing::{KeyId, public_key_from_base64, verify_manifest};

/// Summary of a release verification operation.
#[derive(Debug)]
pub struct VerificationSummary {
    /// Whether the overall verification succeeded.
    pub success: bool,
    /// ctoolbox version in the manifest.
    pub version: String,
    /// Platform of the release.
    pub platform: String,
    /// Build date of the release.
    pub date: String,
    /// Number of files in the manifest.
    pub file_count: usize,
    /// Total number of chunks across all files.
    pub chunk_count: usize,
    /// Number of chunks that were verified.
    pub chunks_verified: usize,
    /// Number of unique chunks read from disk and Brotli-decompressed in this pass.
    pub unique_chunks_decompressed: usize,
    /// Any errors encountered during verification.
    pub errors: Vec<String>,
}

impl std::fmt::Display for VerificationSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Release Verification Summary")?;
        writeln!(f, "============================")?;
        writeln!(f, "Version:  {}", self.version)?;
        writeln!(f, "Platform: {}", self.platform)?;
        writeln!(f, "Date:     {}", self.date)?;
        writeln!(f, "Files:    {}", self.file_count)?;
        writeln!(
            f,
            "Chunks:   {} total, {} verified ({} unique newly decompressed)",
            self.chunk_count,
            self.chunks_verified,
            self.unique_chunks_decompressed
        )?;
        writeln!(f)?;
        if self.success {
            writeln!(f, "✓ Verification PASSED")?;
        } else {
            writeln!(f, "✗ Verification FAILED")?;
            writeln!(f)?;
            writeln!(f, "Errors:")?;
            for err in &self.errors {
                writeln!(f, "  - {err}")?;
            }
        }
        Ok(())
    }
}

/// Verifies a release manifest and all its chunks.
///
/// This function performs the following checks:
/// 1. Loads the manifest from the specified path
/// 2. Verifies the Ed25519 signature against the provided public key
/// 3. Checks that all chunk files exist in the chunks directory
/// 4. Verifies each chunk's SHA-256 hash matches its content
///
/// # Arguments
/// - `manifest_path`: Path to the manifest JSON file
/// - `chunks_dir`: Path to the directory containing chunk files (bh/)
/// - `public_key_b64`: Base64-encoded Ed25519 public key for verification
///
/// # Returns
/// A `VerificationSummary` containing the results of all checks.
pub fn verify_release(
    manifest_path: &Path,
    chunks_dir: &Path,
    public_key_b64: &str,
) -> Result<VerificationSummary> {
    let mut verified_chunks = HashMap::new();
    verify_release_with_cache(
        manifest_path,
        chunks_dir,
        public_key_b64,
        &mut verified_chunks,
    )
}

/// Verifies a release manifest while deduplicating chunk decompression/hashing
/// against `verified_chunks` (`chunk_hash -> verified_uncompressed_len`).
///
/// Manifest signature and revocation checks always run for every manifest, and
/// every referenced chunk's expected uncompressed length is checked against the
/// verified chunk length, while each unique `.br` chunk file on disk is read,
/// Brotli-decompressed, and SHA-256 hashed at most once across platforms.
pub fn verify_release_with_cache(
    manifest_path: &Path,
    chunks_dir: &Path,
    public_key_b64: &str,
    verified_chunks: &mut HashMap<String, u64>,
) -> Result<VerificationSummary> {
    let mut errors = Vec::new();
    let mut chunks_verified = 0usize;
    let mut unique_chunks_decompressed = 0usize;

    // Load the public key
    let public_key = public_key_from_base64(public_key_b64)
        .context("Failed to parse release public key")?;

    let key_id = KeyId::from_public_key(&public_key);
    log_fmt!("Using public key: {}", key_id.to_hex());

    // Load and parse the manifest
    let manifest_json =
        fs::read_to_string(manifest_path).with_context(|| {
            format!("Failed to read manifest: {}", manifest_path.display())
        })?;

    let manifest: ReleaseManifest = serde_json::from_str(&manifest_json)
        .with_context(|| {
            format!("Failed to parse manifest: {}", manifest_path.display())
        })?;

    // Check if key is revoked
    if manifest.is_key_revoked(&key_id.to_hex()) {
        errors.push(format!("Public key {} has been revoked", key_id.to_hex()));
    }

    // Verify signature
    match verify_manifest(&manifest, &public_key) {
        Ok(true) => {
            log!("Signature verification: PASSED");
        }
        Ok(false) => {
            errors.push(
                "Signature verification failed: invalid signature".to_string(),
            );
        }
        Err(e) => {
            errors.push(format!("Signature verification error: {e}"));
        }
    }

    // Count total chunks
    let total_chunks: usize =
        manifest.files.iter().map(|f| f.chunks.len()).sum();

    // Verify each chunk (only decompressing/hashing unique chunks once)
    for file_entry in &manifest.files {
        for chunk_info in &file_entry.chunks {
            // Reason for fallback: slice bounds check for hash prefix formatting defaults to full hash string
            let short_hash =
                chunk_info.hash.get(..16).unwrap_or(&chunk_info.hash);

            if let Some(&verified_len) = verified_chunks.get(&chunk_info.hash) {
                if verified_len != chunk_info.length {
                    errors.push(format!(
                        "Chunk {short_hash} has wrong size: expected {}, got {verified_len}",
                        chunk_info.length
                    ));
                    continue;
                }
                chunks_verified = chunks_verified.saturating_add(1);
                continue;
            }

            let chunk_path =
                compressed_chunk_file_path(chunks_dir, &chunk_info.hash);

            if !chunk_path.exists() {
                errors.push(format!(
                    "Missing chunk {short_hash} for file {}",
                    file_entry.path
                ));
                continue;
            }

            // Read compressed chunk and decompress before verification
            match fs::read(&chunk_path) {
                Ok(compressed_data) => {
                    // Decompress using Brotli
                    let mut decoder = brotli::Decompressor::new(
                        compressed_data.as_slice(),
                        4096,
                    );
                    let mut data = Vec::new();
                    if let Err(e) =
                        std::io::Read::read_to_end(&mut decoder, &mut data)
                    {
                        errors.push(format!(
                            "Failed to decompress chunk {short_hash}: {e}"
                        ));
                        continue;
                    }

                    // Reason for fallback: chunk data buffer length u64 conversion overflow defaults data_len to 0
                    let data_len = u64::try_from(data.len()).unwrap_or(0);
                    if data_len != chunk_info.length {
                        errors.push(format!(
                            "Chunk {short_hash} has wrong size: expected {}, got {data_len}",
                            chunk_info.length
                        ));
                        continue;
                    }

                    let computed_hash = compute_sha256_hex(&data);
                    if computed_hash != chunk_info.hash {
                        // Reason for fallback: slice bounds check for hash prefix formatting defaults to full hash string
                        let short_computed =
                            computed_hash.get(..16).unwrap_or(&computed_hash);
                        errors.push(format!(
                            "Chunk {short_hash} has wrong hash: expected {}, got {short_computed}",
                            chunk_info.hash
                        ));
                        continue;
                    }

                    verified_chunks.insert(chunk_info.hash.clone(), data_len);
                    chunks_verified = chunks_verified.saturating_add(1);
                    unique_chunks_decompressed =
                        unique_chunks_decompressed.saturating_add(1);
                }
                Err(e) => {
                    errors.push(format!(
                        "Failed to read chunk {short_hash}: {e}"
                    ));
                }
            }
        }
    }

    let success = errors.is_empty();

    Ok(VerificationSummary {
        success,
        version: manifest.ctoolbox_version.to_string(),
        platform: manifest.platform.to_string(),
        date: ctb_utilities::time::format_rfc3339_auto_si(manifest.date),
        file_count: manifest.files.len(),
        chunk_count: total_chunks,
        chunks_verified,
        unique_chunks_decompressed,
        errors,
    })
}

/// Runs the dev-release-check command across one or more target platforms.
///
/// Loads the release manifest(s) from the given path (or resolves each
/// platform-specific latest manifest), verifies each manifest's Ed25519
/// signature against the public key from `pc_settings`, and verifies all unique
/// chunk hashes once across the run.
///
/// # Arguments
/// - `manifest_path`: Optional path to a single manifest; defaults to
///   `{storage_dir}/releases/ctb-{platform}-latest.json` for each platform
/// - `chunks_dir`: Optional path to chunks directory; defaults to
///   `{storage_dir}/releases/bh/`
/// - `platforms`: Target strings (e.g. `["linux-x64", "freebsd-x64"]`).
///   Defaults to the current platform when empty and `manifest_path` is `None`.
///
/// # Returns
/// A string summary of the verification results.
pub fn run_dev_release_check(
    manifest_path: Option<&Path>,
    chunks_dir: Option<&Path>,
    platforms: &[String],
) -> Result<String> {
    use ctb_utilities::pc_settings::{PcSettingStrKey, get_str_setting};
    use ctb_utilities::storage::get_storage_dir;

    // Get the public key from settings
    let Some(public_key_b64) =
        get_str_setting(PcSettingStrKey::ReleasePublicKey)
    else {
        bail!(
            "No release_public_key configured in pc_settings. \
               Please set it before verifying releases."
        );
    };

    let storage_dir = get_storage_dir()?;
    let releases_dir = storage_dir.join("releases");

    // Determine chunks directory
    let chunks_dir = match chunks_dir {
        Some(p) => p.to_path_buf(),
        None => releases_dir.join("bh"),
    };

    let manifest_paths = if let Some(explicit_manifest) = manifest_path {
        if platforms.len() > 1 {
            bail!(
                "Cannot combine --manifest with multiple --platform values; omit --manifest to check multiple platforms."
            );
        }
        vec![explicit_manifest.to_path_buf()]
    } else {
        let resolved_platforms: Vec<String> = if platforms.is_empty() {
            vec![crate::download::current_platform()]
        } else {
            platforms
                .iter()
                .map(|p| p.trim())
                .filter(|p| !p.is_empty())
                .map(str::to_string)
                .collect()
        };
        if resolved_platforms.is_empty() {
            bail!("No valid platform specified for ctb-dev-release-check");
        }
        resolved_platforms
            .into_iter()
            .map(|plat| releases_dir.join(format!("ctb-{plat}-latest.json")))
            .collect()
    };

    log_fmt!("Chunks directory: {}", chunks_dir.display());

    let mut verified_chunks = HashMap::new();
    let mut formatted_summaries = Vec::with_capacity(manifest_paths.len());
    let mut any_failed = false;

    for path in &manifest_paths {
        log_fmt!("Verifying release manifest: {}", path.display());
        let summary = verify_release_with_cache(
            path,
            &chunks_dir,
            &public_key_b64,
            &mut verified_chunks,
        )?;
        if !summary.success {
            any_failed = true;
        }
        formatted_summaries.push(summary.to_string());
    }

    let combined = formatted_summaries.join("\n");
    if any_failed {
        bail!("{combined}");
    }

    Ok(combined)
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
    use crate::manifest::{ChunkInfo, FileEntry, Platform, ReleaseManifest};
    use crate::signing::{
        generate_keypair, public_key_to_base64, sign_manifest,
    };
    use jiff::Timestamp;
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// Helper to write a Brotli-compressed chunk to the chunks directory.
    fn write_compressed_chunk(
        chunks_dir: &std::path::Path,
        hash: &str,
        data: &[u8],
    ) {
        let chunk_path: PathBuf = if hash.len() >= 4 {
            #[expect(
                clippy::expect_used,
                reason = "hash.len() >= 4 is guaranteed by preceding condition"
            )]
            let prefix1 = hash.get(0..2).expect("hash.len() >= 4");
            #[expect(
                clippy::expect_used,
                reason = "hash.len() >= 4 is guaranteed by preceding condition"
            )]
            let prefix2 = hash.get(2..4).expect("hash.len() >= 4");
            chunks_dir
                .join(prefix1)
                .join(prefix2)
                .join(format!("{hash}.br"))
        } else {
            chunks_dir.join(format!("{hash}.br"))
        };

        if let Some(parent) = chunk_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }

        let mut compressed = Vec::new();
        {
            let params = brotli::enc::BrotliEncoderParams {
                quality: 6,
                ..Default::default()
            };
            let mut encoder = brotli::CompressorWriter::with_params(
                &mut compressed,
                4096,
                &params,
            );
            encoder.write_all(data).unwrap();
            encoder.flush().unwrap();
        }
        fs::write(chunk_path, compressed).unwrap();
    }

    #[crate::ctb_test]
    fn test_verify_release_valid() {
        // Create temp directories
        let temp = TempDir::new().unwrap();
        let releases_dir = temp.path();
        let chunks_dir = releases_dir.join("bh");
        fs::create_dir_all(&chunks_dir).unwrap();

        // Generate keypair
        let (private_key, public_key) = generate_keypair();
        let public_key_b64 = public_key_to_base64(&public_key);

        // Create a test chunk
        let chunk_data = b"Hello, world!";
        let chunk_hash = compute_sha256_hex(chunk_data);
        let chunk_len = u64::try_from(chunk_data.len()).unwrap();

        // Write compressed chunk to disk
        write_compressed_chunk(&chunks_dir, &chunk_hash, chunk_data);

        // Create manifest
        let mut manifest = ReleaseManifest::new(
            semver::Version::new(1, 0, 0),
            Platform::LinuxX64,
            Timestamp::now(),
        );

        let mut file_entry = FileEntry::new(
            "test.txt".to_string(),
            chunk_hash.clone(),
            "test_feature".to_string(),
        );
        file_entry.add_chunk(ChunkInfo::new(chunk_hash, 0, chunk_len));
        manifest.add_file(file_entry);

        // Sign manifest
        let signature = sign_manifest(&manifest, &private_key).unwrap();
        manifest.signature = Some(signature);

        // Write manifest
        let manifest_path = releases_dir.join("latest.json");
        let manifest_json = serde_json::to_string_pretty(&manifest).unwrap();
        fs::write(&manifest_path, manifest_json).unwrap();

        // Verify
        let summary =
            verify_release(&manifest_path, &chunks_dir, &public_key_b64)
                .unwrap();

        assert!(
            summary.success,
            "Verification should pass: {:?}",
            summary.errors
        );
        assert_eq!(summary.file_count, 1);
        assert_eq!(summary.chunk_count, 1);
        assert_eq!(summary.chunks_verified, 1);
        assert!(summary.errors.is_empty());
    }

    #[crate::ctb_test]
    fn test_verify_release_missing_chunk() {
        let temp = TempDir::new().unwrap();
        let releases_dir = temp.path();
        let chunks_dir = releases_dir.join("bh");
        fs::create_dir_all(&chunks_dir).unwrap();

        let (private_key, public_key) = generate_keypair();
        let public_key_b64 = public_key_to_base64(&public_key);

        // Create manifest with a reference to a non-existent chunk
        let mut manifest = ReleaseManifest::new(
            semver::Version::new(1, 0, 0),
            Platform::LinuxX64,
            Timestamp::now(),
        );

        let mut file_entry = FileEntry::new(
            "missing.txt".to_string(),
            "0000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
            "test_feature".to_string(),
        );
        file_entry.add_chunk(ChunkInfo::new(
            "0000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
            0,
            100,
        ));
        manifest.add_file(file_entry);

        let signature = sign_manifest(&manifest, &private_key).unwrap();
        manifest.signature = Some(signature);

        let manifest_path = releases_dir.join("latest.json");
        fs::write(
            &manifest_path,
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();

        let summary =
            verify_release(&manifest_path, &chunks_dir, &public_key_b64)
                .unwrap();

        assert!(!summary.success);
        assert_eq!(summary.errors.len(), 1);
        assert!(summary.errors[0].contains("Missing chunk"));
    }

    #[crate::ctb_test]
    fn test_verify_release_invalid_signature() {
        let temp = TempDir::new().unwrap();
        let releases_dir = temp.path();
        let chunks_dir = releases_dir.join("bh");
        fs::create_dir_all(&chunks_dir).unwrap();

        // Generate two keypairs - sign with one, verify with other
        let (private_key, _) = generate_keypair();
        let (_, wrong_public_key) = generate_keypair();
        let wrong_public_key_b64 = public_key_to_base64(&wrong_public_key);

        let mut manifest = ReleaseManifest::new(
            semver::Version::new(1, 0, 0),
            Platform::LinuxX64,
            Timestamp::now(),
        );

        let signature = sign_manifest(&manifest, &private_key).unwrap();
        manifest.signature = Some(signature);

        let manifest_path = releases_dir.join("latest.json");
        fs::write(
            &manifest_path,
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();

        let summary =
            verify_release(&manifest_path, &chunks_dir, &wrong_public_key_b64)
                .unwrap();

        assert!(!summary.success);
        assert!(
            summary
                .errors
                .iter()
                .any(|e| e.contains("invalid signature"))
        );
    }

    #[crate::ctb_test]
    fn test_run_dev_release_check_returns_err_on_failed_verification() {
        use ctb_utilities::json::maybe_value::MaybeOption;
        use ctb_utilities::pc_settings::PcSettings;

        let temp = TempDir::new().unwrap();
        let releases_dir = temp.path().join("releases");
        let chunks_dir = releases_dir.join("bh");
        fs::create_dir_all(&chunks_dir).unwrap();

        let (private_key, _) = generate_keypair();
        let (_, wrong_public_key) = generate_keypair();
        let wrong_public_key_b64 = public_key_to_base64(&wrong_public_key);

        PcSettings::apply_patch(PcSettings {
            release_public_key: MaybeOption::Value(wrong_public_key_b64),
            ..Default::default()
        })
        .unwrap();

        let mut manifest = ReleaseManifest::new(
            semver::Version::new(1, 0, 0),
            Platform::LinuxX64,
            Timestamp::now(),
        );
        manifest.signature =
            Some(sign_manifest(&manifest, &private_key).unwrap());

        let manifest_path = releases_dir.join("ctb-linux-x64-latest.json");
        fs::write(
            &manifest_path,
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();

        let err = run_dev_release_check(
            Some(&manifest_path),
            Some(&chunks_dir),
            &["linux-x64".to_string()],
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("Verification FAILED"),
            "expected failure summary in error, got: {err}"
        );
    }

    #[crate::ctb_test]
    fn test_verify_release_with_cache_deduplicates_shared_chunks_across_platforms()
     {
        let temp = TempDir::new().unwrap();
        let releases_dir = temp.path();
        let chunks_dir = releases_dir.join("bh");
        fs::create_dir_all(&chunks_dir).unwrap();

        let (private_key, public_key) = generate_keypair();
        let public_key_b64 = public_key_to_base64(&public_key);

        let shared_data = b"shared v86_images.rsrc content across all targets";
        let shared_hash = compute_sha256_hex(shared_data);
        let shared_len = u64::try_from(shared_data.len()).unwrap();
        write_compressed_chunk(&chunks_dir, &shared_hash, shared_data);

        let linux_bin = b"linux-x64-ctoolbox-elf";
        let linux_hash = compute_sha256_hex(linux_bin);
        let linux_len = u64::try_from(linux_bin.len()).unwrap();
        write_compressed_chunk(&chunks_dir, &linux_hash, linux_bin);

        let freebsd_bin = b"freebsd-x64-ctoolbox-elf";
        let freebsd_hash = compute_sha256_hex(freebsd_bin);
        let freebsd_len = u64::try_from(freebsd_bin.len()).unwrap();
        write_compressed_chunk(&chunks_dir, &freebsd_hash, freebsd_bin);

        // Build linux-x64 manifest (shared + linux binary)
        let mut linux_manifest = ReleaseManifest::new(
            semver::Version::new(1, 0, 0),
            Platform::LinuxX64,
            Timestamp::now(),
        );
        let mut shared_entry = FileEntry::new(
            "v86_images.rsrc".to_string(),
            shared_hash.clone(),
            "v86".to_string(),
        );
        shared_entry.add_chunk(ChunkInfo::new(
            shared_hash.clone(),
            0,
            shared_len,
        ));
        linux_manifest.add_file(shared_entry.clone());

        let mut linux_entry = FileEntry::new(
            "ctoolbox".to_string(),
            linux_hash.clone(),
            "core".to_string(),
        );
        linux_entry.add_chunk(ChunkInfo::new(linux_hash, 0, linux_len));
        linux_manifest.add_file(linux_entry);
        linux_manifest.signature =
            Some(sign_manifest(&linux_manifest, &private_key).unwrap());
        let linux_manifest_path =
            releases_dir.join("ctb-linux-x64-latest.json");
        fs::write(
            &linux_manifest_path,
            serde_json::to_string_pretty(&linux_manifest).unwrap(),
        )
        .unwrap();

        // Build freebsd-x64 manifest (same shared_entry + freebsd binary)
        let mut freebsd_manifest = ReleaseManifest::new(
            semver::Version::new(1, 0, 0),
            Platform::FreebsdX64,
            Timestamp::now(),
        );
        freebsd_manifest.add_file(shared_entry);
        let mut freebsd_entry = FileEntry::new(
            "ctoolbox".to_string(),
            freebsd_hash.clone(),
            "core".to_string(),
        );
        freebsd_entry.add_chunk(ChunkInfo::new(freebsd_hash, 0, freebsd_len));
        freebsd_manifest.add_file(freebsd_entry);
        freebsd_manifest.signature =
            Some(sign_manifest(&freebsd_manifest, &private_key).unwrap());
        let freebsd_manifest_path =
            releases_dir.join("ctb-freebsd-x64-latest.json");
        fs::write(
            &freebsd_manifest_path,
            serde_json::to_string_pretty(&freebsd_manifest).unwrap(),
        )
        .unwrap();

        let mut cache = HashMap::new();
        let sum_linux = verify_release_with_cache(
            &linux_manifest_path,
            &chunks_dir,
            &public_key_b64,
            &mut cache,
        )
        .unwrap();
        assert!(sum_linux.success);
        assert_eq!(sum_linux.chunk_count, 2);
        assert_eq!(sum_linux.chunks_verified, 2);
        assert_eq!(sum_linux.unique_chunks_decompressed, 2);

        // Even if the shared chunk file were removed now, we'd know it wasn't re-read;
        // verify_release_with_cache skips decompressing shared_hash and only decompresses freebsd_hash!
        let sum_freebsd = verify_release_with_cache(
            &freebsd_manifest_path,
            &chunks_dir,
            &public_key_b64,
            &mut cache,
        )
        .unwrap();
        assert!(sum_freebsd.success);
        assert_eq!(sum_freebsd.chunk_count, 2);
        assert_eq!(sum_freebsd.chunks_verified, 2);
        assert_eq!(sum_freebsd.unique_chunks_decompressed, 1);

        // Verify a manifest claiming a wrong length for an already-verified chunk hash still fails
        let mut tampered_manifest = ReleaseManifest::new(
            semver::Version::new(1, 0, 0),
            Platform::WindowsX64,
            Timestamp::now(),
        );
        let mut bad_shared = FileEntry::new(
            "v86_images.rsrc".to_string(),
            shared_hash.clone(),
            "v86".to_string(),
        );
        bad_shared.add_chunk(ChunkInfo::new(
            shared_hash,
            0,
            shared_len.saturating_add(1),
        ));
        tampered_manifest.add_file(bad_shared);
        tampered_manifest.signature =
            Some(sign_manifest(&tampered_manifest, &private_key).unwrap());
        let tampered_path = releases_dir.join("ctb-windows-x64-latest.json");
        fs::write(
            &tampered_path,
            serde_json::to_string_pretty(&tampered_manifest).unwrap(),
        )
        .unwrap();

        let sum_tampered = verify_release_with_cache(
            &tampered_path,
            &chunks_dir,
            &public_key_b64,
            &mut cache,
        )
        .unwrap();
        assert!(!sum_tampered.success);
        assert!(
            sum_tampered
                .errors
                .iter()
                .any(|e| e.contains("wrong size"))
        );
    }
}
