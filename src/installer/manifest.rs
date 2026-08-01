//! Release manifest types for installer and update system.
//!
//! This module defines the `ReleaseManifest` structure used for tracking
//! release artifacts, their checksums, chunk information, and signatures.
//! Manifests are JSON-serializable and support Ed25519 signature verification.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use chrono::{DateTime, Utc};
use ctb_formats_base64::base64_decode;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use math::approx_float::{f64_to_u64_approx, u64_to_f64_approx};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported build targets for a release.
///
/// This is intentionally OS+arch (not just OS), so a server can host multiple
/// artifacts for the same OS (e.g. linux-x86 and linux-x64) at once.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Platform {
    #[serde(rename = "linux-x64")]
    LinuxX64,
    #[serde(rename = "linux-x86")]
    LinuxX86,
    #[serde(rename = "windows-x64")]
    WindowsX64,
    #[serde(rename = "mac-x64")]
    MacX64,
    #[serde(rename = "mac-arm64")]
    MacArm64,
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Platform::LinuxX64 => write!(f, "linux-x64"),
            Platform::LinuxX86 => write!(f, "linux-x86"),
            Platform::WindowsX64 => write!(f, "windows-x64"),
            Platform::MacX64 => write!(f, "mac-x64"),
            Platform::MacArm64 => write!(f, "mac-arm64"),
        }
    }
}

/// Information about a single content-defined chunk within a file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkInfo {
    /// SHA-256 hash of the chunk data (hex-encoded).
    pub hash: String,
    /// Byte offset within the reassembled file.
    pub offset: u64,
    /// Length of the chunk in bytes.
    pub length: u64,
    /// Compressed size in bytes, if known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compressed_size: Option<u64>,
}

/// A single file entry in the release manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileEntry {
    /// Installation path relative to the install directory.
    pub path: String,
    /// SHA-256 checksum of the complete file (hex-encoded).
    pub checksum: String,
    /// Total file size in bytes (uncompressed). This is the sum of all chunk
    /// lengths and is used for displaying required disk space in the installer.
    pub file_size: u64,
    /// Whether to gzip the file after installation (for archives that should
    /// be stored compressed locally but uncompressed on server for dedup).
    pub gzip_after_install: bool,
    /// Unique identifier for the feature this file belongs to.
    pub feature_id: String,
    /// Human-readable feature name in multiple languages (ISO code -> name).
    pub feature_name: HashMap<String, String>,
    /// List of feature IDs that this feature depends on.
    pub requires: Vec<String>,
    /// Whether this feature is required and cannot be deselected.
    #[serde(default)]
    pub required: bool,
    /// Whether this feature is currently unavailable (e.g., not ready for this
    /// platform). Unavailable features are shown in the installer but cannot
    /// be selected.
    #[serde(default)]
    pub unavailable: bool,
    /// Content-defined chunks that make up this file.
    pub chunks: Vec<ChunkInfo>,
}

/// The release manifest describing a complete ctoolbox release.
///
/// This manifest is signed by the developer and verified by installers/updaters
/// before applying updates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseManifest {
    /// Manifest format version for forward compatibility.
    pub format_version: u8,
    /// ctoolbox version for this release.
    pub ctoolbox_version: semver::Version,
    /// Target platform for this release.
    pub platform: Platform,
    /// Build/release date in UTC.
    pub date: DateTime<Utc>,
    /// Ed25519 signature over the manifest (base64-encoded).
    /// This field is excluded when computing the signature.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    /// List of revoked signing key IDs (first 8 bytes of public key hash).
    pub revoked_key_ids: Vec<String>,
    /// Files included in this release.
    pub files: Vec<FileEntry>,
}

/// Helper struct for serializing the manifest without the signature field.
/// Used when computing or verifying signatures.
#[derive(Debug, Clone, Serialize)]
struct ManifestForSigning<'a> {
    format_version: u8,
    ctoolbox_version: &'a semver::Version,
    platform: Platform,
    date: &'a DateTime<Utc>,
    revoked_key_ids: &'a Vec<String>,
    files: &'a Vec<FileEntry>,
}

impl ReleaseManifest {
    /// Current manifest format version.
    pub const CURRENT_FORMAT_VERSION: u8 = 1;

    /// Returns the size of the installer file entry in the manifest, if present.
    pub fn installer_file_size(&self) -> u64 {
        self.files
            .iter()
            .find(|f| f.path == "ctoolbox-installer")
            .map_or(0, |f| f.file_size)
    }

    /// Returns the sum of uncompressed sizes of all files in the manifest.
    pub fn total_installed_size(&self) -> u64 {
        self.files.iter().map(|f| f.file_size).sum()
    }

    /// Estimates the offline tarball size containing the manifest, chunks, and the installer.
    pub fn estimate_offline_tarball_size(&self) -> u64 {
        let installer_size = self.installer_file_size();

        let mut seen_chunks = std::collections::HashSet::new();
        let mut total_compressed_chunk_bytes: u64 = 0;
        for file in &self.files {
            for chunk in &file.chunks {
                if seen_chunks.insert(&chunk.hash) {
                    if let Some(comp_sz) = chunk.compressed_size {
                        total_compressed_chunk_bytes =
                            total_compressed_chunk_bytes
                                .saturating_add(comp_sz);
                    } else {
                        // Fallback to 45% of uncompressed size
                        total_compressed_chunk_bytes = total_compressed_chunk_bytes.saturating_add(
                            f64_to_u64_approx(u64_to_f64_approx(chunk.length).expect("The filesize is weird; it does not approximate to f64.") * 0.45).expect("The filesize is weird; it does not approximate to u64.")
                        );
                    }
                }
            }
        }

        // Overhead: manifest file size + tar headers (512 bytes per entry) + padding + 1024 bytes trailer
        // Let's assume a generic overhead of 200 KB
        let overhead = 200_000;

        installer_size
            .saturating_add(total_compressed_chunk_bytes)
            .saturating_add(overhead)
    }

    /// Estimates the gzipped size of a file in the manifest.
    pub fn estimate_gzipped_file_size(&self, path: &str) -> u64 {
        let uncompressed = self
            .files
            .iter()
            .find(|f| f.path == path)
            .map_or(0, |f| f.file_size);
        // Assume 29.3% compression ratio for source code/dependencies tarballs
        f64_to_u64_approx(
            u64_to_f64_approx(uncompressed).expect(
                "The filesize is weird; it does not approximate to f64.",
            ) * 0.293,
        )
        .expect("The filesize is weird; it does not approximate to u64.")
    }

    /// Creates a new release manifest with the current format version.
    pub fn new(
        ctoolbox_version: semver::Version,
        platform: Platform,
        date: DateTime<Utc>,
    ) -> Self {
        Self {
            format_version: Self::CURRENT_FORMAT_VERSION,
            ctoolbox_version,
            platform,
            date,
            signature: None,
            revoked_key_ids: Vec::new(),
            files: Vec::new(),
        }
    }

    /// Serializes the manifest to JSON, excluding the signature field.
    ///
    /// This is the canonical representation used for signing and verification.
    /// The JSON is serialized in a deterministic order to ensure consistent
    /// signatures.
    pub fn serialize_for_signing(&self) -> Result<String> {
        let for_signing = ManifestForSigning {
            format_version: self.format_version,
            ctoolbox_version: &self.ctoolbox_version,
            platform: self.platform,
            date: &self.date,
            revoked_key_ids: &self.revoked_key_ids,
            files: &self.files,
        };
        serde_json::to_string(&for_signing)
            .context("Failed to serialize manifest for signing")
    }

    /// Verifies the manifest's signature against a given public key.
    ///
    /// Returns `Ok(true)` if the signature is valid, `Ok(false)` if the
    /// signature is present but invalid, and an error if the signature is
    /// missing or malformed.
    pub fn verify_signature(&self, public_key: &VerifyingKey) -> Result<bool> {
        let Some(sig_b64) = &self.signature else {
            bail!("Manifest has no signature");
        };

        // Decode the base64 signature
        let sig_bytes = base64_decode(sig_b64)?;

        // Parse signature bytes
        let sig_array: [u8; 64] =
            sig_bytes.try_into().map_err(|v: Vec<u8>| {
                anyhow::anyhow!(
                    "Invalid signature length: expected 64 bytes, got {}",
                    v.len()
                )
            })?;
        let signature = Signature::from_bytes(&sig_array);

        // Get canonical JSON for verification
        let message = self.serialize_for_signing()?;

        // Verify signature
        match public_key.verify(message.as_bytes(), &signature) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Adds a file entry to the manifest.
    pub fn add_file(&mut self, entry: FileEntry) {
        self.files.push(entry);
    }

    /// Adds a revoked key ID to the manifest.
    pub fn add_revoked_key_id(&mut self, key_id: String) {
        self.revoked_key_ids.push(key_id);
    }

    /// Checks if a given key ID has been revoked in this manifest.
    pub fn is_key_revoked(&self, key_id: &str) -> bool {
        self.revoked_key_ids.iter().any(|id| id == key_id)
    }
}

impl FileEntry {
    /// Creates a new file entry with the given path and checksum.
    pub fn new(path: String, checksum: String, feature_id: String) -> Self {
        Self {
            path,
            checksum,
            file_size: 0,
            gzip_after_install: false,
            feature_id,
            feature_name: HashMap::new(),
            requires: Vec::new(),
            required: false,
            unavailable: false,
            chunks: Vec::new(),
        }
    }

    /// Sets the file size in bytes.
    #[must_use]
    pub fn with_file_size(mut self, size: u64) -> Self {
        self.file_size = size;
        self
    }

    /// Marks this feature as required.
    #[must_use]
    pub fn with_required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Marks this feature as unavailable.
    #[must_use]
    pub fn with_unavailable(mut self, unavailable: bool) -> Self {
        self.unavailable = unavailable;
        self
    }

    /// Calculates the file size from chunk lengths.
    ///
    /// This can be called after adding chunks to compute the total size.
    pub fn compute_file_size(&mut self) {
        self.file_size = self
            .chunks
            .iter()
            .map(|chunk| chunk.offset.saturating_add(chunk.length))
            .max()
            .unwrap_or(0);
    }

    /// Sets whether the file should be gzipped after installation.
    #[must_use]
    pub fn with_gzip_after_install(mut self, gzip: bool) -> Self {
        self.gzip_after_install = gzip;
        self
    }

    /// Adds a localized feature name.
    #[must_use]
    pub fn with_feature_name(
        mut self,
        lang_code: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        self.feature_name.insert(lang_code.into(), name.into());
        self
    }

    /// Adds a required feature dependency.
    #[must_use]
    pub fn with_requires(mut self, feature_id: impl Into<String>) -> Self {
        self.requires.push(feature_id.into());
        self
    }

    /// Adds a chunk to the file entry.
    pub fn add_chunk(&mut self, chunk: ChunkInfo) {
        self.file_size = self
            .file_size
            .max(chunk.offset.saturating_add(chunk.length));
        self.chunks.push(chunk);
    }

    /// Gets the feature name for a given language, falling back to English.
    pub fn get_feature_name(&self, lang_code: &str) -> Option<&str> {
        self.feature_name
            .get(lang_code)
            .or_else(|| self.feature_name.get("en"))
            .map(String::as_str)
    }
}

impl ChunkInfo {
    /// Creates a new chunk info entry.
    pub fn new(hash: String, offset: u64, length: u64) -> Self {
        Self {
            hash,
            offset,
            length,
            compressed_size: None,
        }
    }
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
    use crate::signing::{generate_keypair, sign_manifest};

    #[crate::ctb_test]
    fn test_add_chunk_updates_file_size_from_chunk_end() {
        let mut entry = FileEntry::new(
            "ctoolbox-installer".to_string(),
            "sum".to_string(),
            "core".to_string(),
        );

        entry.add_chunk(ChunkInfo::new("a".to_string(), 0, 5));
        entry.add_chunk(ChunkInfo::new("b".to_string(), 5, 3));

        assert_eq!(entry.file_size, 8);
    }

    #[crate::ctb_test]
    fn test_compute_file_size_uses_highest_chunk_end() {
        let mut entry = FileEntry::new(
            "ctoolbox-installer".to_string(),
            "sum".to_string(),
            "core".to_string(),
        );
        entry.chunks = vec![
            ChunkInfo::new("a".to_string(), 0, 5),
            ChunkInfo::new("b".to_string(), 10, 3),
        ];

        entry.compute_file_size();

        assert_eq!(entry.file_size, 13);
    }

    #[crate::ctb_test]
    fn test_manifest_serialization() {
        let manifest = ReleaseManifest::new(
            semver::Version::new(1, 0, 0),
            Platform::LinuxX64,
            Utc::now(),
        );

        let json = serde_json::to_string(&manifest).unwrap();
        let parsed: ReleaseManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(manifest, parsed);
    }

    #[crate::ctb_test]
    fn test_serialize_for_signing_excludes_signature() {
        let mut manifest = ReleaseManifest::new(
            semver::Version::new(1, 0, 0),
            Platform::LinuxX64,
            Utc::now(),
        );
        manifest.signature = Some("fake_signature".to_string());

        let for_signing = manifest.serialize_for_signing().unwrap();
        assert!(!for_signing.contains("signature"));
        assert!(!for_signing.contains("fake_signature"));
    }

    #[crate::ctb_test]
    fn test_file_entry_builder() {
        let entry = FileEntry::new(
            "bin/ctoolbox".to_string(),
            "abc123".to_string(),
            "core".to_string(),
        )
        .with_gzip_after_install(false)
        .with_feature_name("en", "Core Application")
        .with_feature_name("de", "Kernanwendung")
        .with_requires("runtime");

        assert_eq!(entry.path, "bin/ctoolbox");
        assert_eq!(entry.get_feature_name("en"), Some("Core Application"));
        assert_eq!(entry.get_feature_name("de"), Some("Kernanwendung"));
        assert_eq!(entry.get_feature_name("fr"), Some("Core Application")); // Falls back to en
        assert_eq!(entry.requires, vec!["runtime"]);
    }

    #[crate::ctb_test]
    fn test_signature_verification() {
        let (private_key, public_key) = generate_keypair();

        let mut manifest = ReleaseManifest::new(
            semver::Version::new(1, 2, 3),
            Platform::LinuxX64,
            Utc::now(),
        );

        // Sign the manifest
        manifest.signature =
            Some(sign_manifest(&manifest, &private_key).unwrap());

        // Verify the signature
        assert!(
            manifest
                .verify_signature(public_key.as_verifying_key())
                .unwrap()
        );
    }

    #[crate::ctb_test]
    fn test_invalid_signature() {
        use base64::Engine;

        let (_private_key, public_key) = generate_keypair();

        let mut manifest = ReleaseManifest::new(
            semver::Version::new(1, 0, 0),
            Platform::LinuxX64,
            Utc::now(),
        );

        // Set an invalid signature
        manifest.signature =
            Some(base64::engine::general_purpose::STANDARD.encode([0u8; 64]));

        // Should return false for invalid signature
        assert!(
            !manifest
                .verify_signature(public_key.as_verifying_key())
                .unwrap()
        );
    }

    #[crate::ctb_test]
    fn test_missing_signature() {
        let (_private_key, public_key) = generate_keypair();

        let manifest = ReleaseManifest::new(
            semver::Version::new(1, 0, 0),
            Platform::LinuxX64,
            Utc::now(),
        );

        // Should error on missing signature
        assert!(
            manifest
                .verify_signature(public_key.as_verifying_key())
                .is_err()
        );
    }

    #[crate::ctb_test]
    fn test_revoked_key_check() {
        let mut manifest = ReleaseManifest::new(
            semver::Version::new(1, 0, 0),
            Platform::LinuxX64,
            Utc::now(),
        );

        manifest.add_revoked_key_id("abc12345".to_string());

        assert!(manifest.is_key_revoked("abc12345"));
        assert!(!manifest.is_key_revoked("other_key"));
    }

    #[crate::ctb_test]
    fn test_platform_serialization() {
        assert_eq!(
            serde_json::to_string(&Platform::LinuxX64).unwrap(),
            "\"linux-x64\""
        );
        assert_eq!(
            serde_json::to_string(&Platform::LinuxX86).unwrap(),
            "\"linux-x86\""
        );
        assert_eq!(
            serde_json::to_string(&Platform::WindowsX64).unwrap(),
            "\"windows-x64\""
        );
        assert_eq!(
            serde_json::to_string(&Platform::MacX64).unwrap(),
            "\"mac-x64\""
        );
        assert_eq!(
            serde_json::to_string(&Platform::MacArm64).unwrap(),
            "\"mac-arm64\""
        );
    }

    #[crate::ctb_test]
    fn test_manifest_filesizes() {
        let mut manifest = ReleaseManifest::new(
            semver::Version::new(1, 0, 0),
            Platform::LinuxX64,
            Utc::now(),
        );

        let mut installer = FileEntry::new(
            "ctoolbox-installer".to_string(),
            "abc".to_string(),
            "installer".to_string(),
        )
        .with_file_size(10_000_000);
        installer.add_chunk(ChunkInfo::new(
            "chunk1".to_string(),
            0,
            10_000_000,
        ));
        manifest.add_file(installer);

        let mut other_file = FileEntry::new(
            "bin/ctoolbox".to_string(),
            "def".to_string(),
            "core".to_string(),
        )
        .with_file_size(20_000_000);
        let mut chunk2 = ChunkInfo::new("chunk2".to_string(), 0, 20_000_000);
        chunk2.compressed_size = Some(8_000_000);
        other_file.add_chunk(chunk2);
        manifest.add_file(other_file);

        assert_eq!(manifest.installer_file_size(), 10_000_000);
        assert_eq!(manifest.total_installed_size(), 30_000_000);

        // chunk1 (fallback to 45% of 10M = 4.5M) + chunk2 (8M) + installer (10M) + overhead (200k)
        // = 4.5M + 8M + 10M + 200k = 22.7M
        let estimated_offline = manifest.estimate_offline_tarball_size();
        assert_eq!(
            estimated_offline,
            4_500_000 + 8_000_000 + 10_000_000 + 200_000
        );

        assert_eq!(
            manifest.estimate_gzipped_file_size("bin/ctoolbox"),
            5_860_000
        );
    }
}
