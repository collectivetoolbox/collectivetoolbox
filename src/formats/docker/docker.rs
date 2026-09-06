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

//! Docker image and container archive validation utilities.

#[expect(
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use anyhow::{Context, Result};
use include_dir::{Dir, include_dir};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::io::Read;

pub mod cli;

static DOCKER_DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data");

/// Returns an embedded fixture asset byte vector if present.
pub fn get_docker_data(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&DOCKER_DATA_DIR, key)
}

/// Represents an entry in `manifest.json`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DockerManifestEntry {
    pub config: String,
    #[serde(default)]
    pub repo_tags: Option<Vec<String>>,
    pub layers: Vec<String>,
    #[serde(default)]
    pub layer_sources: Option<serde_json::Value>,
}

/// Represents `index.json`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OciIndex {
    pub schema_version: u32,
    #[serde(default)]
    pub media_type: Option<String>,
    pub manifests: Vec<OciIndexManifest>,
}

/// Manifest descriptor in `index.json`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OciIndexManifest {
    pub media_type: String,
    pub digest: String,
    pub size: u64,
    #[serde(default)]
    pub annotations: Option<serde_json::Value>,
}

/// Represents `oci-layout`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OciLayout {
    pub image_layout_version: String,
}

/// Record of a blob whose computed SHA-256 does not match its filename hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobChecksumMismatch {
    pub blob_path: String,
    pub expected_hash: String,
    pub computed_hash: String,
}

/// Summary report of docker image archive validation.
#[derive(Debug, Clone, Default)]
pub struct ValidationReport {
    pub validated_metadata: HashSet<String>,
    pub total_blobs_checked: usize,
    pub valid_blobs_count: usize,
    pub checksum_mismatches: Vec<BlobChecksumMismatch>,
    pub missing_blobs: Vec<String>,
    pub unreferenced_blobs: Vec<String>,
    pub referenced_blobs_count: usize,
}

impl ValidationReport {
    /// Returns true if validation succeeded according to strictness settings.
    pub fn is_success(&self, strict: bool) -> bool {
        self.validated_metadata.contains("manifest.json")
            && self.checksum_mismatches.is_empty()
            && self.missing_blobs.is_empty()
            && (!strict || self.unreferenced_blobs.is_empty())
    }
}

/// Strips leading `./` or `/` prefixes from tar entry paths.
fn normalize_path(path: &str) -> String {
    let mut p = path;
    while let Some(stripped) = p.strip_prefix("./") {
        p = stripped;
    }
    p.trim_start_matches('/').to_string()
}

/// Extracts a clean hex digest from a blob reference string.
fn extract_blob_hash(ref_str: &str) -> String {
    let mut s = ref_str;
    if let Some(stripped) = s.strip_prefix("blobs/sha256/") {
        s = stripped;
    }
    if let Some(stripped) = s.strip_prefix("sha256:") {
        s = stripped;
    }
    if let Some(stripped) = s.strip_suffix(".json") {
        s = stripped;
    }
    if let Some(stripped) = s.strip_suffix("/layer.tar") {
        s = stripped;
    }
    s.trim_matches('/').to_string()
}

/// Parses and validates one of the image metadata JSON files.
fn process_metadata_entry<R: Read>(
    normalized_path: &str,
    entry: &mut tar::Entry<R>,
    referenced_blobs: &mut HashSet<String>,
    report: &mut ValidationReport,
) -> Result<bool> {
    match normalized_path {
        "oci-layout" => {
            let mut content = Vec::new();
            entry
                .read_to_end(&mut content)
                .context("Failed reading oci-layout")?;
            serde_json::from_slice::<OciLayout>(&content)
                .context("Invalid JSON in oci-layout")?;
            report.validated_metadata.insert("oci-layout".to_string());
            Ok(true)
        }
        "repositories" => {
            let mut content = Vec::new();
            entry
                .read_to_end(&mut content)
                .context("Failed reading repositories")?;
            serde_json::from_slice::<serde_json::Value>(&content)
                .context("Invalid JSON in repositories")?;
            report.validated_metadata.insert("repositories".to_string());
            Ok(true)
        }
        "index.json" => {
            let mut content = Vec::new();
            entry
                .read_to_end(&mut content)
                .context("Failed reading index.json")?;
            let index: OciIndex = serde_json::from_slice(&content)
                .context("Invalid JSON in index.json")?;
            for manifest in &index.manifests {
                let digest = extract_blob_hash(&manifest.digest);
                referenced_blobs.insert(digest.to_ascii_lowercase());
            }
            report.validated_metadata.insert("index.json".to_string());
            Ok(true)
        }
        "manifest.json" => {
            let mut content = Vec::new();
            entry
                .read_to_end(&mut content)
                .context("Failed reading manifest.json")?;
            let entries: Vec<DockerManifestEntry> =
                serde_json::from_slice(&content)
                    .context("Invalid JSON in manifest.json")?;
            for m in &entries {
                let config_hash = extract_blob_hash(&m.config);
                referenced_blobs.insert(config_hash.to_ascii_lowercase());
                for layer in &m.layers {
                    let layer_hash = extract_blob_hash(layer);
                    referenced_blobs.insert(layer_hash.to_ascii_lowercase());
                }
            }
            report.validated_metadata.insert("manifest.json".to_string());
            Ok(true)
        }
        _ => Ok(false),
    }
}

/// Computes SHA-256 for a blob entry and validates it against its filename.
fn process_blob_entry<R: Read>(
    normalized_path: &str,
    expected_hash: &str,
    entry: &mut tar::Entry<R>,
    found_blobs: &mut HashSet<String>,
    report: &mut ValidationReport,
) -> Result<()> {
    let expected_hash = expected_hash.trim_end_matches('/');
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; 65536];

    loop {
        let n = entry
            .read(&mut buffer)
            .context("Failed to read blob chunk")?;
        if n == 0 {
            break;
        }
        let slice = buffer
            .get(..n)
            .ok_or_else(|| anyhow::anyhow!("Buffer slice bounds error"))?;
        hasher.update(slice);
    }

    let computed_digest = ctb_utilities::string::to_hex(&hasher.finalize());
    report.total_blobs_checked = report.total_blobs_checked.saturating_add(1);
    let expected_lower = expected_hash.to_ascii_lowercase();

    if computed_digest.eq_ignore_ascii_case(&expected_lower) {
        report.valid_blobs_count = report.valid_blobs_count.saturating_add(1);
    } else {
        report.checksum_mismatches.push(BlobChecksumMismatch {
            blob_path: normalized_path.to_string(),
            expected_hash: expected_lower.clone(),
            computed_hash: computed_digest,
        });
    }
    found_blobs.insert(expected_lower);
    Ok(())
}

/// Streams a tar archive, verifying JSON metadata and blob checksums.
pub fn validate_docker_archive<R: Read>(
    reader: R,
    _strict: bool,
) -> Result<ValidationReport> {
    let mut report = ValidationReport::default();
    let mut referenced_blobs = HashSet::<String>::new();
    let mut found_blobs = HashSet::<String>::new();

    let mut archive = tar::Archive::new(reader);
    let entries = archive.entries().context("Failed to read tar entries")?;

    for entry_result in entries {
        let mut entry = entry_result.context("Failed to read tar entry")?;
        if entry.header().entry_type().is_dir() {
            continue;
        }

        let raw_path = entry.path().context("Invalid tar entry path")?;
        let path_str = raw_path.to_string_lossy();
        let normalized_path = normalize_path(&path_str);

        let was_metadata = process_metadata_entry(
            &normalized_path,
            &mut entry,
            &mut referenced_blobs,
            &mut report,
        )?;

        if !was_metadata
            && let Some(expected_hash) =
                normalized_path.strip_prefix("blobs/sha256/")
        {
            process_blob_entry(
                &normalized_path,
                expected_hash,
                &mut entry,
                &mut found_blobs,
                &mut report,
            )?;
        }
    }

    if !report.validated_metadata.contains("manifest.json") {
        anyhow::bail!("Archive is missing required manifest.json");
    }

    report.referenced_blobs_count = referenced_blobs.len();

    for ref_blob in &referenced_blobs {
        if !found_blobs.contains(ref_blob) {
            report.missing_blobs.push(ref_blob.clone());
        }
    }
    report.missing_blobs.sort();

    for found_blob in &found_blobs {
        if !referenced_blobs.contains(found_blob) {
            report.unreferenced_blobs.push(found_blob.clone());
        }
    }
    report.unreferenced_blobs.sort();

    Ok(report)
}

#[cfg(test)]
#[expect(
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
    use std::io::Cursor;

    #[crate::ctb_test]
    fn test_validate_hello_world_fixture() {
        let fixture_bytes = get_docker_data(
            "fixtures/docker-hello-world/hello-world.tar",
        )
        .expect("Load hello-world.tar fixture");

        let report = validate_docker_archive(Cursor::new(&fixture_bytes), false)
            .expect("Validate hello-world.tar");

        assert!(report.validated_metadata.contains("manifest.json"));
        assert!(report.validated_metadata.contains("index.json"));
        assert!(report.validated_metadata.contains("oci-layout"));
        assert!(report.validated_metadata.contains("repositories"));
        assert_eq!(report.missing_blobs.len(), 0);
        assert_eq!(report.checksum_mismatches.len(), 0);
        assert_eq!(report.valid_blobs_count, 4);
        assert!(report.is_success(false));
    }

    #[crate::ctb_test]
    fn test_validate_synthetic_valid_archive() {
        let mut builder = tar::Builder::new(Vec::new());

        let oci_layout = b"{\"imageLayoutVersion\": \"1.0.0\"}";
        let mut header = tar::Header::new_gnu();
        header.set_size(u64::try_from(oci_layout.len()).unwrap());
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(&mut header, "oci-layout", &oci_layout[..])
            .unwrap();

        let repos = b"{\"repo\":{\"latest\":\"layer1\"}}";
        let mut header = tar::Header::new_gnu();
        header.set_size(u64::try_from(repos.len()).unwrap());
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(&mut header, "repositories", &repos[..])
            .unwrap();

        let layer_content = b"fake layer content 123";
        let layer_hash =
            ctb_formats_checksum::sha256_hex(layer_content);

        let config_content = b"{\"architecture\":\"amd64\"}";
        let config_hash =
            ctb_formats_checksum::sha256_hex(config_content);

        let manifest_json = format!(
            "[{{\"Config\":\"blobs/sha256/{config_hash}\",\"Layers\":[\"blobs/sha256/{layer_hash}\"]}}]"
        );
        let mut header = tar::Header::new_gnu();
        header.set_size(u64::try_from(manifest_json.len()).unwrap());
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(
                &mut header,
                "manifest.json",
                manifest_json.as_bytes(),
            )
            .unwrap();

        let mut header = tar::Header::new_gnu();
        header.set_size(u64::try_from(config_content.len()).unwrap());
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(
                &mut header,
                format!("blobs/sha256/{config_hash}"),
                &config_content[..],
            )
            .unwrap();

        let mut header = tar::Header::new_gnu();
        header.set_size(u64::try_from(layer_content.len()).unwrap());
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(
                &mut header,
                format!("blobs/sha256/{layer_hash}"),
                &layer_content[..],
            )
            .unwrap();

        let tar_bytes = builder.into_inner().unwrap();
        let report =
            validate_docker_archive(Cursor::new(&tar_bytes), true).unwrap();

        assert!(report.is_success(true));
        assert_eq!(report.valid_blobs_count, 2);
        assert_eq!(report.checksum_mismatches.len(), 0);
        assert_eq!(report.missing_blobs.len(), 0);
        assert_eq!(report.unreferenced_blobs.len(), 0);
    }

    #[crate::ctb_test]
    fn test_validate_checksum_mismatch() {
        let mut builder = tar::Builder::new(Vec::new());

        let fake_hash =
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let manifest_json = format!(
            "[{{\"Config\":\"blobs/sha256/{fake_hash}\",\"Layers\":[]}}]"
        );
        let mut header = tar::Header::new_gnu();
        header.set_size(u64::try_from(manifest_json.len()).unwrap());
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(
                &mut header,
                "manifest.json",
                manifest_json.as_bytes(),
            )
            .unwrap();

        let corrupt_content = b"not matching hash";
        let mut header = tar::Header::new_gnu();
        header.set_size(u64::try_from(corrupt_content.len()).unwrap());
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(
                &mut header,
                format!("blobs/sha256/{fake_hash}"),
                &corrupt_content[..],
            )
            .unwrap();

        let tar_bytes = builder.into_inner().unwrap();
        let report =
            validate_docker_archive(Cursor::new(&tar_bytes), false).unwrap();

        assert!(!report.is_success(false));
        assert_eq!(report.checksum_mismatches.len(), 1);
        assert_eq!(
            report.checksum_mismatches[0].expected_hash,
            fake_hash
        );
    }

    #[crate::ctb_test]
    fn test_validate_missing_blob() {
        let mut builder = tar::Builder::new(Vec::new());

        let missing_hash =
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        let manifest_json = format!(
            "[{{\"Config\":\"blobs/sha256/{missing_hash}\",\"Layers\":[]}}]"
        );
        let mut header = tar::Header::new_gnu();
        header.set_size(u64::try_from(manifest_json.len()).unwrap());
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(
                &mut header,
                "manifest.json",
                manifest_json.as_bytes(),
            )
            .unwrap();

        let tar_bytes = builder.into_inner().unwrap();
        let report =
            validate_docker_archive(Cursor::new(&tar_bytes), false).unwrap();

        assert!(!report.is_success(false));
        assert_eq!(report.missing_blobs.len(), 1);
        assert_eq!(report.missing_blobs[0], missing_hash);
    }

    #[crate::ctb_test]
    fn test_validate_strict_mode() {
        let fixture_bytes = get_docker_data(
            "fixtures/docker-hello-world/hello-world.tar",
        )
        .expect("Load hello-world.tar fixture");

        // Without strict: passes even with unreferenced blob
        let report_non_strict = validate_docker_archive(
            Cursor::new(&fixture_bytes),
            false,
        )
        .expect("Validate without strict");
        assert!(report_non_strict.is_success(false));
        assert_eq!(report_non_strict.unreferenced_blobs.len(), 1);

        // With strict: fails due to unreferenced blob
        assert!(!report_non_strict.is_success(true));
    }

    #[crate::ctb_test]
    fn test_validate_invalid_json() {
        let mut builder = tar::Builder::new(Vec::new());

        let bad_json = b"not valid json {{{";
        let mut header = tar::Header::new_gnu();
        header.set_size(u64::try_from(bad_json.len()).unwrap());
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(&mut header, "manifest.json", &bad_json[..])
            .unwrap();

        let tar_bytes = builder.into_inner().unwrap();
        let result = validate_docker_archive(Cursor::new(&tar_bytes), false);
        assert!(result.is_err());
    }

    #[crate::ctb_test]
    fn test_cli_run_validate_docker_image() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let fixture_path = std::path::Path::new(manifest_dir)
            .join("data/fixtures/docker-hello-world/hello-world.tar");
        let res = cli::run_validate_docker_image(Some(&fixture_path), false)
            .expect("Run CLI validation");

        match res {
            ToolResult::Immediate { exit_code, stdout, .. } => {
                assert_eq!(exit_code, 0);
                let text = String::from_utf8_lossy(&stdout);
                assert!(text.contains("Docker image validation successful:"));
                assert!(text.contains("manifest.json (valid JSON)"));
                assert!(text.contains("4 blob checksum(s) verified (SHA-256)"));
            }
            _ => panic!("Expected Immediate ToolResult"),
        }

        // Test strict failure on hello-world fixture (unreferenced blob present)
        let res_strict =
            cli::run_validate_docker_image(Some(&fixture_path), true)
                .expect("Run CLI validation strict");
        match res_strict {
            ToolResult::Immediate { exit_code, stderr, .. } => {
                assert_eq!(exit_code, 1);
                let text = String::from_utf8_lossy(&stderr);
                assert!(text.contains("Docker image validation failed:"));
                assert!(text.contains("unreferenced blob(s) found (strict mode)"));
            }
            _ => panic!("Expected Immediate ToolResult"),
        }
    }
}

