//! Release expiration functionality for garbage-collecting old chunks.
//!
//! This module provides the `run_release_expire` function used by the
//! `--ctb-dev-release-expire` CLI command to delete chunks only referenced
//! by manifests older than a configurable threshold.
//!
//! The expiration process:
//! 1. Scans the releases directory for all manifest JSON files
//! 2. Builds a set of chunk hashes referenced by recent manifests
//! 3. Scans the `bh/` directory and deletes chunks not in the keep set
//! 4. Deletes old manifest files

#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

use chrono::{Duration, Utc};
use ctb_utilities::string::bytes::format_bytes_both;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::manifest::ReleaseManifest;

/// Summary of a release expiration operation.
#[derive(Debug)]
pub struct ExpireSummary {
    /// Number of manifest files scanned.
    pub manifests_scanned: usize,
    /// Number of manifest files deleted (older than threshold).
    pub manifests_deleted: usize,
    /// Number of manifest files kept (newer than threshold).
    pub manifests_kept: usize,
    /// Total number of chunks scanned.
    pub chunks_scanned: usize,
    /// Number of chunks deleted (not referenced by recent manifests).
    pub chunks_deleted: usize,
    /// Number of chunks kept (referenced by recent manifests).
    pub chunks_kept: usize,
    /// Total bytes freed by deletion.
    pub bytes_freed: u64,
    /// Any errors encountered during expiration.
    pub errors: Vec<String>,
}

impl std::fmt::Display for ExpireSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Release Expiration Summary")?;
        writeln!(f, "==========================")?;
        writeln!(
            f,
            "Manifests: {} scanned, {} kept, {} deleted",
            self.manifests_scanned, self.manifests_kept, self.manifests_deleted
        )?;
        writeln!(
            f,
            "Chunks:    {} scanned, {} kept, {} deleted",
            self.chunks_scanned, self.chunks_kept, self.chunks_deleted
        )?;
        writeln!(f)?;
        writeln!(
            f,
            "Deleted {} chunks, {} manifests, freed {} bytes",
            self.chunks_deleted,
            self.manifests_deleted,
            format_bytes_both(self.bytes_freed)
        )?;
        if !self.errors.is_empty() {
            writeln!(f)?;
            writeln!(f, "Errors:")?;
            for err in &self.errors {
                writeln!(f, "  - {err}")?;
            }
        }
        Ok(())
    }
}

/// Checks if a filename looks like a release manifest.
///
/// Manifest files match the pattern `ctb-{platform}-{datetime}.json` or
/// `ctb-{platform}-latest.json`.
fn is_manifest_file(filename: &str) -> bool {
    filename.starts_with("ctb-") && filename.ends_with(".json")
}

/// Extracts all chunk hashes from a manifest.
fn get_chunk_hashes(manifest: &ReleaseManifest) -> HashSet<String> {
    manifest
        .files
        .iter()
        .flat_map(|file| file.chunks.iter().map(|chunk| chunk.hash.clone()))
        .collect()
}

/// Recursively scans a directory for chunk files and returns their paths.
///
/// Chunks are stored in a nested structure: `bh/{prefix1}/{prefix2}/{hash}.br`.
fn scan_chunks_directory(
    chunks_dir: &Path,
) -> Result<Vec<(PathBuf, String, u64)>> {
    let mut chunks = Vec::new();

    if !chunks_dir.exists() {
        return Ok(chunks);
    }

    // Traverse the nested structure
    for prefix1_entry in fs::read_dir(chunks_dir).with_context(|| {
        format!("Failed to read chunks directory: {}", chunks_dir.display())
    })? {
        let prefix1_entry = prefix1_entry?;
        let prefix1_path = prefix1_entry.path();

        if !prefix1_path.is_dir() {
            continue;
        }

        for prefix2_entry in fs::read_dir(&prefix1_path)? {
            let prefix2_entry = prefix2_entry?;
            let prefix2_path = prefix2_entry.path();

            if !prefix2_path.is_dir() {
                continue;
            }

            for chunk_entry in fs::read_dir(&prefix2_path)? {
                let chunk_entry = chunk_entry?;
                let chunk_path = chunk_entry.path();

                if chunk_path.is_file() {
                    // Extract hash from filename, removing .br extension if present
                    let hash = chunk_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|s| s.strip_suffix(".br").unwrap_or(s))
                        .map(String::from)
                        .unwrap_or_default();

                    let size =
                        chunk_entry.metadata().map(|m| m.len()).unwrap_or(0);
                    chunks.push((chunk_path, hash, size));
                }
            }
        }
    }

    Ok(chunks)
}

/// Expires old release chunks and manifests.
///
/// This function performs the following:
/// 1. Scans `releases_dir` for all manifest JSON files
/// 2. Parses each manifest and categorizes as "keep" or "expire" based on date
/// 3. Builds a set of all chunk hashes referenced by kept manifests
/// 4. Scans `releases_dir/bh/` and deletes chunks not in the keep set
/// 5. Deletes expired manifest files
///
/// # Arguments
/// - `releases_dir`: Path to the releases directory (contains manifests and
///   `bh/` subdirectory)
/// - `older_than_days`: Manifests older than this many days will be expired
///
/// # Returns
/// An `ExpireSummary` containing statistics about the operation.
pub fn expire_releases(
    releases_dir: &Path,
    older_than_days: u32,
) -> Result<ExpireSummary> {
    let mut errors = Vec::new();
    let mut manifests_to_delete = Vec::new();
    let mut keep_chunk_hashes: HashSet<String> = HashSet::new();

    let cutoff_date = Utc::now()
        .checked_sub_signed(Duration::days(i64::from(older_than_days)))
        .context("cutoff date calculation overflow")?;

    log_fmt!(
        "Expiring releases older than {} days (before {})",
        older_than_days,
        cutoff_date.format("%Y-%m-%d %H:%M:%S UTC")
    );

    // Scan for manifest files
    let mut manifests_scanned = 0usize;
    let mut manifests_kept = 0usize;

    if !releases_dir.exists() {
        bail!(
            "Releases directory does not exist: {}",
            releases_dir.display()
        );
    }

    for entry in fs::read_dir(releases_dir).with_context(|| {
        format!(
            "Failed to read releases directory: {}",
            releases_dir.display()
        )
    })? {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                errors.push(format!("Failed to read directory entry: {e}"));
                continue;
            }
        };

        let path = entry.path();

        // Skip directories and non-JSON files
        if !path.is_file() {
            continue;
        }

        let filename = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name,
            None => continue,
        };

        if !is_manifest_file(filename) {
            continue;
        }

        // Skip 'latest.json' symlinks - these are managed separately
        if filename.ends_with("-latest.json") {
            continue;
        }

        manifests_scanned = manifests_scanned.saturating_add(1);

        // Parse the manifest
        let manifest_json = match fs::read_to_string(&path) {
            Ok(json) => json,
            Err(e) => {
                errors.push(format!(
                    "Failed to read manifest {}: {e}",
                    path.display()
                ));
                continue;
            }
        };

        let manifest: ReleaseManifest =
            match serde_json::from_str(&manifest_json) {
                Ok(m) => m,
                Err(e) => {
                    errors.push(format!(
                        "Failed to parse manifest {}: {e}",
                        path.display()
                    ));
                    continue;
                }
            };

        // Check if manifest is old enough to expire
        if manifest.date < cutoff_date {
            log_fmt!(
                "Marking for deletion: {} (date: {})",
                filename,
                manifest.date.format("%Y-%m-%d")
            );
            manifests_to_delete.push(path);
        } else {
            log_fmt!(
                "Keeping: {} (date: {})",
                filename,
                manifest.date.format("%Y-%m-%d")
            );
            manifests_kept = manifests_kept.saturating_add(1);

            // Add all chunk hashes from this manifest to the keep set
            let hashes = get_chunk_hashes(&manifest);
            keep_chunk_hashes.extend(hashes);
        }
    }

    log_fmt!(
        "Found {} chunk hashes to keep from {} recent manifests",
        keep_chunk_hashes.len(),
        manifests_kept
    );

    // Scan chunks directory
    let chunks_dir = releases_dir.join("bh");
    let all_chunks = scan_chunks_directory(&chunks_dir)?;

    let chunks_scanned = all_chunks.len();
    let mut chunks_deleted = 0usize;
    let mut chunks_kept = 0usize;
    let mut bytes_freed = 0u64;

    // Delete chunks not in the keep set
    for (chunk_path, hash, size) in all_chunks {
        if keep_chunk_hashes.contains(&hash) {
            chunks_kept = chunks_kept.saturating_add(1);
        } else {
            match fs::remove_file(&chunk_path) {
                Ok(()) => {
                    chunks_deleted = chunks_deleted.saturating_add(1);
                    bytes_freed = bytes_freed.saturating_add(size);
                }
                Err(e) => {
                    errors.push(format!(
                        "Failed to delete chunk {}: {e}",
                        chunk_path.display()
                    ));
                }
            }
        }
    }

    // Delete old manifest files
    let manifests_deleted = manifests_to_delete.len();
    for manifest_path in manifests_to_delete {
        if let Ok(metadata) = fs::metadata(&manifest_path) {
            bytes_freed = bytes_freed.saturating_add(metadata.len());
        }
        if let Err(e) = fs::remove_file(&manifest_path) {
            errors.push(format!(
                "Failed to delete manifest {}: {e}",
                manifest_path.display()
            ));
        }
    }

    // Clean up empty directories in bh/
    cleanup_empty_directories(&chunks_dir);

    Ok(ExpireSummary {
        manifests_scanned,
        manifests_deleted,
        manifests_kept,
        chunks_scanned,
        chunks_deleted,
        chunks_kept,
        bytes_freed,
        errors,
    })
}

/// Recursively removes empty directories under the given path.
fn cleanup_empty_directories(dir: &Path) {
    if !dir.is_dir() {
        return;
    }

    // First, recurse into subdirectories
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                cleanup_empty_directories(&path);
            }
        }
    }

    // Then try to remove this directory if empty (ignore errors)
    let _ = fs::remove_dir(dir);
}

/// Runs the dev-release-expire command.
///
/// Expires old releases from the given directory (or default location).
///
/// # Arguments
/// - `older_than_days`: Expire releases older than this many days
/// - `releases_dir`: Optional path to releases directory; defaults to
///   `{storage_dir}/releases/`
///
/// # Returns
/// A string summary of the expiration results.
pub fn run_release_expire(
    older_than_days: u32,
    releases_dir: Option<&Path>,
) -> Result<String> {
    use ctb_utilities::storage::get_storage_dir;

    // Determine releases directory
    let releases_dir = match releases_dir {
        Some(p) => p.to_path_buf(),
        None => get_storage_dir()?.join("releases"),
    };

    log_fmt!("Expiring releases in: {}", releases_dir.display());

    let summary = expire_releases(&releases_dir, older_than_days)?;

    Ok(summary.to_string())
}

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use super::*;
    use crate::manifest::{ChunkInfo, FileEntry, Platform, ReleaseManifest};
    use crate::signing::{generate_keypair, sign_manifest};
    use chrono::{DateTime, Duration, Utc};
    use std::fs;
    use tempfile::TempDir;

    fn create_test_chunk(
        chunks_dir: &Path,
        hash: &str,
        data: &[u8],
    ) -> Result<()> {
        let prefix1 = hash.get(0..2).unwrap_or("");
        let prefix2 = hash.get(2..4).unwrap_or("");
        let chunk_dir = chunks_dir.join(prefix1).join(prefix2);
        fs::create_dir_all(&chunk_dir)?;
        fs::write(chunk_dir.join(hash), data)?;
        Ok(())
    }

    fn create_test_manifest(
        releases_dir: &Path,
        filename: &str,
        date: DateTime<Utc>,
        chunk_hashes: &[&str],
    ) -> Result<()> {
        let (private_key, _) = generate_keypair();

        let mut manifest = ReleaseManifest::new(
            semver::Version::new(1, 0, 0),
            Platform::LinuxX64,
            date,
        );

        let mut file_entry = FileEntry::new(
            "test.bin".to_string(),
            "checksum".to_string(),
            "core".to_string(),
        );

        for (i, hash) in chunk_hashes.iter().enumerate() {
            file_entry.add_chunk(ChunkInfo::new(
                (*hash).to_string(),
                u64::try_from(i.saturating_mul(1000)).unwrap_or(0),
                1000,
            ));
        }
        manifest.add_file(file_entry);

        let signature = sign_manifest(&manifest, &private_key)?;
        manifest.signature = Some(signature);

        let manifest_json = serde_json::to_string_pretty(&manifest)?;
        fs::write(releases_dir.join(filename), manifest_json)?;

        Ok(())
    }

    #[crate::ctb_test]
    fn test_expire_releases_basic() {
        let temp = TempDir::new().unwrap();
        let releases_dir = temp.path();
        let chunks_dir = releases_dir.join("bh");
        fs::create_dir_all(&chunks_dir).unwrap();

        // Create chunks
        let recent_hash = "aa11223344556677889900aabbccddeeff00112233445566778899aabbccddeeff";
        let old_hash = "bb11223344556677889900aabbccddeeff00112233445566778899aabbccddeeff";
        let orphan_hash = "cc11223344556677889900aabbccddeeff00112233445566778899aabbccddeeff";

        create_test_chunk(&chunks_dir, recent_hash, b"recent chunk data")
            .unwrap();
        create_test_chunk(&chunks_dir, old_hash, b"old chunk data").unwrap();
        create_test_chunk(&chunks_dir, orphan_hash, b"orphan chunk data")
            .unwrap();

        // Create manifests
        let recent_date = Utc::now().checked_sub_signed(Duration::days(5)).unwrap_or_else(Utc::now);
        let old_date = Utc::now().checked_sub_signed(Duration::days(60)).unwrap_or_else(Utc::now);

        create_test_manifest(
            releases_dir,
            "ctb-linux-2025-01-10.json",
            recent_date,
            &[recent_hash],
        )
        .unwrap();

        create_test_manifest(
            releases_dir,
            "ctb-linux-2024-11-01.json",
            old_date,
            &[old_hash],
        )
        .unwrap();

        // Run expiration with 30-day threshold
        let summary = expire_releases(releases_dir, 30).unwrap();

        // Check results
        assert_eq!(summary.manifests_scanned, 2);
        assert_eq!(summary.manifests_kept, 1);
        assert_eq!(summary.manifests_deleted, 1);
        assert_eq!(summary.chunks_scanned, 3);
        assert_eq!(summary.chunks_kept, 1);
        assert_eq!(summary.chunks_deleted, 2); // old + orphan

        // Verify recent chunk still exists
        let recent_chunk_path = chunks_dir
            .join(recent_hash.get(0..2).unwrap_or(""))
            .join(recent_hash.get(2..4).unwrap_or(""))
            .join(recent_hash);
        assert!(recent_chunk_path.exists(), "Recent chunk should be kept");

        // Verify old chunk was deleted
        let old_chunk_path = chunks_dir
            .join(old_hash.get(0..2).unwrap_or(""))
            .join(old_hash.get(2..4).unwrap_or(""))
            .join(old_hash);
        assert!(!old_chunk_path.exists(), "Old chunk should be deleted");

        // Verify orphan chunk was deleted
        let orphan_chunk_path = chunks_dir
            .join(orphan_hash.get(0..2).unwrap_or(""))
            .join(orphan_hash.get(2..4).unwrap_or(""))
            .join(orphan_hash);
        assert!(
            !orphan_chunk_path.exists(),
            "Orphan chunk should be deleted"
        );

        // Verify recent manifest still exists
        assert!(
            releases_dir.join("ctb-linux-2025-01-10.json").exists(),
            "Recent manifest should be kept"
        );

        // Verify old manifest was deleted
        assert!(
            !releases_dir.join("ctb-linux-2024-11-01.json").exists(),
            "Old manifest should be deleted"
        );
    }

    #[crate::ctb_test]
    fn test_expire_releases_shared_chunks() {
        let temp = TempDir::new().unwrap();
        let releases_dir = temp.path();
        let chunks_dir = releases_dir.join("bh");
        fs::create_dir_all(&chunks_dir).unwrap();

        // Create a chunk shared by both old and new manifests
        let shared_hash = "dd11223344556677889900aabbccddeeff00112233445566778899aabbccddeeff";
        let old_only_hash = "ee11223344556677889900aabbccddeeff00112233445566778899aabbccddeeff";

        create_test_chunk(&chunks_dir, shared_hash, b"shared chunk").unwrap();
        create_test_chunk(&chunks_dir, old_only_hash, b"old only chunk")
            .unwrap();

        let recent_date = Utc::now().checked_sub_signed(Duration::days(5)).unwrap_or_else(Utc::now);
        let old_date = Utc::now().checked_sub_signed(Duration::days(60)).unwrap_or_else(Utc::now);

        // Recent manifest references shared chunk
        create_test_manifest(
            releases_dir,
            "ctb-linux-recent.json",
            recent_date,
            &[shared_hash],
        )
        .unwrap();

        // Old manifest references both shared and old-only chunks
        create_test_manifest(
            releases_dir,
            "ctb-linux-old.json",
            old_date,
            &[shared_hash, old_only_hash],
        )
        .unwrap();

        let summary = expire_releases(releases_dir, 30).unwrap();

        // Shared chunk should be kept
        assert_eq!(summary.chunks_kept, 1);
        assert_eq!(summary.chunks_deleted, 1);

        let shared_chunk_path = chunks_dir
            .join(shared_hash.get(0..2).unwrap_or(""))
            .join(shared_hash.get(2..4).unwrap_or(""))
            .join(shared_hash);
        assert!(shared_chunk_path.exists(), "Shared chunk should be kept");

        let old_only_chunk_path = chunks_dir
            .join(old_only_hash.get(0..2).unwrap_or(""))
            .join(old_only_hash.get(2..4).unwrap_or(""))
            .join(old_only_hash);
        assert!(
            !old_only_chunk_path.exists(),
            "Old-only chunk should be deleted"
        );
    }

    #[crate::ctb_test]
    fn test_expire_releases_skips_latest() {
        let temp = TempDir::new().unwrap();
        let releases_dir = temp.path();
        let chunks_dir = releases_dir.join("bh");
        fs::create_dir_all(&chunks_dir).unwrap();

        // Create a chunk
        let hash = "ff11223344556677889900aabbccddeeff00112233445566778899aabbccddeeff";
        create_test_chunk(&chunks_dir, hash, b"chunk data").unwrap();

        // Create a manifest
        let date = Utc::now().checked_sub_signed(Duration::days(5)).unwrap_or_else(Utc::now);
        create_test_manifest(
            releases_dir,
            "ctb-linux-2025-01-10.json",
            date,
            &[hash],
        )
        .unwrap();

        // Create a latest.json symlink (or file for testing)
        fs::write(releases_dir.join("ctb-linux-x64-latest.json"), "{}")
            .unwrap();

        let summary = expire_releases(releases_dir, 30).unwrap();

        // Should only scan the dated manifest, not the -latest.json file
        assert_eq!(summary.manifests_scanned, 1);
    }

    #[crate::ctb_test]
    fn test_expire_empty_directory() {
        let temp = TempDir::new().unwrap();
        let releases_dir = temp.path();
        let chunks_dir = releases_dir.join("bh");
        fs::create_dir_all(&chunks_dir).unwrap();

        // No manifests, no chunks
        let summary = expire_releases(releases_dir, 30).unwrap();

        assert_eq!(summary.manifests_scanned, 0);
        assert_eq!(summary.chunks_scanned, 0);
        assert_eq!(summary.bytes_freed, 0);
    }

    #[crate::ctb_test]
    fn test_is_manifest_file() {
        assert!(is_manifest_file("ctb-linux-x64-2025-01-10.json"));
        assert!(is_manifest_file("ctb-windows-x64-latest.json"));
        assert!(is_manifest_file("ctb-mac-2024-12-25.json"));
        assert!(!is_manifest_file("ctb-mac-2024-12-25.json.sig"));
        assert!(!is_manifest_file("readme.txt"));
        assert!(!is_manifest_file("manifest.json"));
        assert!(!is_manifest_file("ctb-linux.tar.gz"));
    }
}
