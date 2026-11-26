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

//! Internet Archive file format, item metadata, and checksum parsing utilities.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

pub mod cli;

use anyhow::{Context, Result, anyhow, bail};
use md5::{Digest as _, Md5};
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, Read, Seek, SeekFrom, Write};
use std::path::{Component, Path, PathBuf};
use xml::reader::{EventReader, XmlEvent};

const PART_FILE_SUFFIX: &str = ".part";
const FILES_XML_SUFFIX: &str = "_files.xml";
const META_XML_SUFFIX: &str = "_meta.xml";
const META_SQLITE_SUFFIX: &str = "_meta.sqlite";
const ARCHIVE_TORRENT_SUFFIX: &str = "_archive.torrent";
const KNOWN_IDENTIFIER_SUFFIXES: [&str; 4] = [
    FILES_XML_SUFFIX,
    META_XML_SUFFIX,
    META_SQLITE_SUFFIX,
    ARCHIVE_TORRENT_SUFFIX,
];

#[derive(Debug, Clone, PartialEq, Eq)]
struct ArchiveTarget {
    identifier: String,
    archive_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ManifestEntry {
    name: String,
    source: Option<String>,
    md5: Option<String>,
    sha1: Option<String>,
    size: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FilesManifest {
    identifier: String,
    entries: BTreeMap<String, ManifestEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct MetadataResponse {
    #[serde(default)]
    files: Vec<MetadataFile>,

    #[serde(flatten)]
    extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct MetadataFile {
    name: String,
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    md5: Option<String>,
    #[serde(default)]
    sha1: Option<String>,
    #[serde(default)]
    size: Option<String>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct VerificationMismatch {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    expected_sha1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    actual_sha1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expected_md5: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    actual_md5: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expected_size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    actual_size: Option<u64>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct VerificationResult {
    valid: bool,
    identifier: String,
    checked_files: Vec<String>,
    missing_files: Vec<String>,
    mismatched_files: Vec<VerificationMismatch>,
    unexpected_files: Vec<String>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct DownloadResult {
    identifier: String,
    output_directory: String,
    downloaded_files: Vec<String>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct DownloadHereResult {
    identifier: String,
    output_path: String,
}

trait ArchiveClient {
    fn fetch_metadata(&self, identifier: &str) -> Result<MetadataResponse>;
    fn fetch_files_xml_bytes(&self, identifier: &str) -> Result<Vec<u8>>;
    fn fetch_meta_xml_bytes(&self, identifier: &str) -> Result<Vec<u8>>;
    fn fetch_download_bytes(
        &self,
        identifier: &str,
        file_name: &str,
    ) -> Result<Vec<u8>>;
    fn download_file(
        &self,
        identifier: &str,
        file_name: &str,
        destination: &Path,
        expected_size: Option<u64>,
        file_progress: Option<&FileItemProgress>,
    ) -> Result<u64> {
        let bytes = self.fetch_download_bytes(identifier, file_name)?;
        let len_u64 = u64::try_from(bytes.len()).map_err(|e| {
            anyhow!("Failed to convert byte length to u64: {e}")
        })?;
        if let Some(expected) = expected_size {
            anyhow::ensure!(
                len_u64 == expected,
                "Downloaded size {len_u64} did not match expected size {expected} for {file_name}"
            );
        }
        if let Some(fp) = file_progress {
            fp.set_total_bytes(len_u64);
            fp.set_bytes_done(len_u64);
        }
        // Reason for fallback: If destination has no parent path (bare filename), stage .part file in current directory.
        let parent = destination.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent).with_context(|| {
            format!("Failed to create parent directory {}", parent.display())
        })?;
        let part_path = part_path_for_destination(destination);
        let mut part_file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&part_path)
            .with_context(|| {
                format!(
                    "Failed to open partial download file {}",
                    part_path.display()
                )
            })?;
        let existing_len = part_file
            .metadata()
            .with_context(|| {
                format!("Failed to read metadata for {}", part_path.display())
            })?
            .len();
        let existing_usize = usize::try_from(existing_len).ok();
        let can_append = existing_usize.is_some_and(|existing| {
            existing <= bytes.len()
                && bytes.get(..existing).is_some_and(|prefix| {
                    let mut current = vec![0_u8; existing];
                    part_file
                        .read_exact(&mut current)
                        .is_ok_and(|()| current.as_slice() == prefix)
                })
        });
        if let Some(existing) = existing_usize.filter(|_| can_append) {
            part_file.seek(SeekFrom::End(0)).with_context(|| {
                format!("Failed to seek to end of {}", part_path.display())
            })?;
            if let Some(tail) = bytes.get(existing..) {
                part_file.write_all(tail).with_context(|| {
                    format!("Failed to append to {}", part_path.display())
                })?;
            }
        } else {
            part_file.set_len(0).with_context(|| {
                format!("Failed to truncate {}", part_path.display())
            })?;
            part_file.seek(SeekFrom::Start(0)).with_context(|| {
                format!("Failed to seek to start of {}", part_path.display())
            })?;
            part_file.write_all(&bytes).with_context(|| {
                format!("Failed to write {}", part_path.display())
            })?;
        }
        part_file.sync_all().with_context(|| {
            format!("Failed to sync {}", part_path.display())
        })?;
        drop(part_file);
        commit_atomic_file(&part_path, destination)?;
        Ok(len_u64)
    }
}

struct LiveArchiveClient;

impl ArchiveClient for LiveArchiveClient {
    fn fetch_metadata(&self, identifier: &str) -> Result<MetadataResponse> {
        fetch_live_metadata(identifier)
    }

    fn fetch_files_xml_bytes(&self, identifier: &str) -> Result<Vec<u8>> {
        fetch_live_files_xml_bytes(identifier)
    }

    fn fetch_meta_xml_bytes(&self, identifier: &str) -> Result<Vec<u8>> {
        fetch_live_meta_xml_bytes(identifier)
    }

    fn fetch_download_bytes(
        &self,
        identifier: &str,
        file_name: &str,
    ) -> Result<Vec<u8>> {
        fetch_download_bytes(identifier, file_name)
    }

    fn download_file(
        &self,
        identifier: &str,
        file_name: &str,
        destination: &Path,
        expected_size: Option<u64>,
        file_progress: Option<&FileItemProgress>,
    ) -> Result<u64> {
        download_live_file_chunked_range(
            identifier,
            file_name,
            destination,
            expected_size,
            file_progress,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VerificationOptions {
    files_xml_live_sha1: Option<String>,
}

pub fn guess_identifier_from_path(path: &Path) -> Option<String> {
    guess_identifer_from_path(path)
}

/// Look for `ident_files.xml`, `ident_meta.xml`, `ident_archive.torrent`,
/// and/or `ident_meta.sqlite`; fall back to a plausible identifier-shaped
/// directory name; otherwise return `None`.
pub fn guess_identifer_from_path(path: &Path) -> Option<String> {
    if let Some(file_name) = path.file_name().and_then(std::ffi::OsStr::to_str)
    {
        if let Some(identifier) = identifier_from_metadata_name(file_name) {
            return Some(identifier);
        }
    }

    if path.is_dir() {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                if let Some(identifier) =
                    name.to_str().and_then(identifier_from_metadata_name)
                {
                    return Some(identifier);
                }
            }
        }
    }

    if let Some(parent_name) = path
        .parent()
        .and_then(Path::file_name)
        .and_then(std::ffi::OsStr::to_str)
        .filter(|name| is_probable_identifier(name))
    {
        return Some(parent_name.to_string());
    }

    path.file_name()
        .and_then(std::ffi::OsStr::to_str)
        .filter(|name| is_probable_identifier(name))
        .map(std::string::ToString::to_string)
}

/// If `check_live` is true, fetch the live files XML and verify the local item
/// against that manifest instead of a local `*_files.xml`.
/// If `original` is true, only verify files with source="original".
pub fn verify(
    item_path: &Path,
    identifier: Option<&str>,
    check_live: bool,
    original: bool,
) -> Result<Vec<u8>> {
    let root = normalize_item_root(item_path);
    let identifier = resolve_identifier(item_path, identifier)?;
    let (manifest, files_xml_live_sha1) = if check_live {
        let live_files_xml_bytes = fetch_live_files_xml_bytes(&identifier)?;
        let manifest =
            parse_files_manifest(&identifier, &live_files_xml_bytes)?;
        let sha1 = Some(sha1_hex_for_bytes(&live_files_xml_bytes));
        (manifest, sha1)
    } else {
        let manifest = load_local_files_manifest(&root, &identifier)?;
        (manifest, None)
    };
    let selected_files = if original {
        Some(
            manifest
                .entries
                .values()
                .filter(|entry| entry.source.as_deref() == Some("original"))
                .map(|entry| entry.name.clone())
                .collect::<BTreeSet<_>>(),
        )
    } else {
        None
    };
    let verification = verify_against_manifest(
        &root,
        &identifier,
        &manifest,
        selected_files.as_ref(),
        true,
        &VerificationOptions {
            files_xml_live_sha1,
        },
        None,
    )?;
    pretty_json_bytes(&verification)
}

pub fn iasha1(
    target: &str,
    identifier: Option<&str>,
    check_live: bool,
) -> Result<Vec<u8>> {
    let resolved_target = resolve_hash_target(target, identifier)?;
    let manifest = if check_live {
        fetch_live_files_manifest(&resolved_target.identifier)?
    } else {
        load_manifest_for_hash_target(&resolved_target)?
    };
    let sha1 =
        sha1_for_manifest_entry(&resolved_target, &manifest, check_live)?;
    let mut output = sha1.into_bytes();
    output.push(b'\n');
    Ok(output)
}

pub fn iamd5(
    target: &str,
    identifier: Option<&str>,
    check_live: bool,
) -> Result<Vec<u8>> {
    let resolved_target = resolve_hash_target(target, identifier)?;
    let manifest = if check_live {
        fetch_live_files_manifest(&resolved_target.identifier)?
    } else {
        load_manifest_for_hash_target(&resolved_target)?
    };
    let md5 = md5_for_manifest_entry(&resolved_target, &manifest, check_live)?;
    let mut output = md5.into_bytes();
    output.push(b'\n');
    Ok(output)
}

pub fn contains(target: &str, desired_file: &str) -> Result<Vec<u8>> {
    contains_with_client(&LiveArchiveClient, target, desired_file)
}

pub fn listplain(target: &str) -> Result<Vec<u8>> {
    listplain_with_client(&LiveArchiveClient, target)
}

pub fn metadata(target: &str) -> Result<Vec<u8>> {
    metadata_with_client(&LiveArchiveClient, target)
}

pub fn filesxml(target: &str) -> Result<Vec<u8>> {
    filesxml_with_client(&LiveArchiveClient, target)
}

pub fn metaxml(target: &str) -> Result<Vec<u8>> {
    metaxml_with_client(&LiveArchiveClient, target)
}

pub fn download(
    target: &str,
    output_dir: Option<&Path>,
    original: bool,
    progress: Progress,
) -> Result<Vec<u8>> {
    download_with_client(
        &LiveArchiveClient,
        target,
        output_dir,
        original,
        progress,
    )
}

pub fn download_as_stream(target: &str) -> Result<Vec<u8>> {
    download_as_stream_with_client(&LiveArchiveClient, target)
}

pub fn download_here(
    target: &str,
    output_dir: Option<&Path>,
    progress: Progress,
) -> Result<Vec<u8>> {
    download_here_with_client(&LiveArchiveClient, target, output_dir, progress)
}

fn resolve_download_file_specs(
    client: &dyn ArchiveClient,
    archive_target: &ArchiveTarget,
    original: bool,
    manifest: Option<&FilesManifest>,
) -> Result<Vec<(String, Option<u64>)>> {
    if original {
        let metadata_files =
            client.fetch_metadata(&archive_target.identifier)?.files;
        let original_files: Vec<(String, Option<u64>)> = metadata_files
            .into_iter()
            .filter(|file| file.source.as_deref() == Some("original"))
            .map(|file| {
                let size =
                    file.size.as_deref().and_then(|s| s.parse().ok()).or_else(
                        || {
                            manifest
                                .and_then(|m| m.entries.get(&file.name))
                                .and_then(|e| e.size)
                        },
                    );
                (file.name, size)
            })
            .collect();
        if let Some(file_name) = &archive_target.archive_path {
            Ok(original_files
                .into_iter()
                .filter(|(name, _)| name == file_name)
                .collect())
        } else {
            Ok(original_files)
        }
    } else if let Some(file_name) = &archive_target.archive_path {
        let known_size = manifest
            .and_then(|m| m.entries.get(file_name))
            .and_then(|e| e.size);
        Ok(vec![(file_name.clone(), known_size)])
    } else {
        Ok(client
            .fetch_metadata(&archive_target.identifier)?
            .files
            .into_iter()
            .map(|file| {
                let size =
                    file.size.as_deref().and_then(|s| s.parse().ok()).or_else(
                        || {
                            manifest
                                .and_then(|m| m.entries.get(&file.name))
                                .and_then(|e| e.size)
                        },
                    );
                (file.name, size)
            })
            .collect())
    }
}

fn start_download_batch(
    progress: &Progress,
    identifier: &str,
    file_specs: &[(String, Option<u64>)],
) -> Result<FileBatchProgress> {
    let total_files_u64 = u64::try_from(file_specs.len())
        .map_err(|e| anyhow!("Failed to convert file count to u64: {e}"))?;
    let initial_known_bytes: u64 = file_specs
        .iter()
        .filter_map(|(_, sz)| *sz)
        .fold(0_u64, u64::saturating_add);
    let any_known_size = file_specs.iter().any(|(_, sz)| sz.is_some());
    Ok(progress.start_file_batch(
        &format!("Overall ({identifier})"),
        Some(total_files_u64),
        any_known_size.then_some(initial_known_bytes),
    ))
}

fn download_with_client(
    client: &dyn ArchiveClient,
    target: &str,
    output_dir: Option<&Path>,
    original: bool,
    progress: Progress,
) -> Result<Vec<u8>> {
    let archive_target = parse_archive_target(target)?;
    let base_output_dir = resolve_output_dir(output_dir)?;
    let item_directory = base_output_dir.join(&archive_target.identifier);
    fs::create_dir_all(&item_directory).with_context(|| {
        format!("Failed to create {}", item_directory.display())
    })?;

    progress.message(&format!(
        "Downloading item '{}' to {}",
        archive_target.identifier,
        item_directory.display()
    ));

    let file_specs =
        resolve_download_file_specs(client, &archive_target, original, None)?;
    let batch = start_download_batch(
        &progress,
        &archive_target.identifier,
        &file_specs,
    )?;

    let mut downloaded_files = Vec::new();

    for (file_name, known_size) in &file_specs {
        let destination = item_directory.join(Path::new(file_name));
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!(
                    "Failed to create parent directory {}",
                    parent.display()
                )
            })?;
        }
        let part_path = part_path_for_destination(&destination);
        if let Some(expected_sz) = *known_size
            && destination.is_file()
            && !part_path.exists()
            && fs::metadata(&destination).is_ok_and(|m| m.len() == expected_sz)
        {
            let fp =
                batch.start_transient_file("Existing", file_name, *known_size);
            fp.finish(expected_sz);
            downloaded_files.push(file_name.clone());
            continue;
        }
        let file_progress = batch.start_file(file_name, *known_size);
        let bytes_written = client.download_file(
            &archive_target.identifier,
            file_name,
            &destination,
            *known_size,
            Some(&file_progress),
        )?;
        file_progress.finish(bytes_written);
        downloaded_files.push(file_name.clone());
    }

    batch.finish();

    let result = DownloadResult {
        identifier: archive_target.identifier,
        output_directory: item_directory.display().to_string(),
        downloaded_files,
    };
    pretty_json_bytes(&result)
}

pub fn checkeddl(
    target: &str,
    output_dir: Option<&Path>,
    original: bool,
    progress: Progress,
) -> Result<Vec<u8>> {
    checkeddl_with_client(
        &LiveArchiveClient,
        target,
        output_dir,
        original,
        progress,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "Internal per-file resume and manifest verification helper"
)]
fn checkeddl_download_or_resume_file(
    client: &dyn ArchiveClient,
    item_directory: &Path,
    identifier: &str,
    file_name: &str,
    known_size: Option<u64>,
    manifest: &FilesManifest,
    verify_options: &VerificationOptions,
    batch: &FileBatchProgress,
    progress: &Progress,
    pre_verified_ok: &mut BTreeSet<String>,
) -> Result<()> {
    let destination = item_directory.join(Path::new(file_name));
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!("Failed to create parent directory {}", parent.display())
        })?;
    }
    let part_path = part_path_for_destination(&destination);
    let entry_opt = manifest.entries.get(file_name);

    if destination.is_file()
        && let Some(entry) = entry_opt
    {
        let check_fp =
            batch.start_transient_file("Check", file_name, known_size);
        let (mismatch_opt, hashed_len) = verify_local_file(
            item_directory,
            identifier,
            &destination,
            entry,
            verify_options,
            Some(&check_fp),
        )?;
        if mismatch_opt.is_none() {
            if part_path.is_file() {
                drop(fs::remove_file(&part_path));
            }
            check_fp.finish(hashed_len);
            pre_verified_ok.insert(file_name.to_string());
            return Ok(());
        }
        drop(check_fp);
        if !part_path.is_file() {
            drop(fs::remove_file(&destination));
        }
    }

    let had_existing_part = part_path.is_file();
    let file_progress = batch.start_file(file_name, known_size);
    let bytes_written = client.download_file(
        identifier,
        file_name,
        &destination,
        known_size,
        Some(&file_progress),
    )?;
    file_progress.finish(bytes_written);

    if had_existing_part && let Some(entry) = entry_opt {
        let (mismatch_opt, _) = verify_local_file(
            item_directory,
            identifier,
            &destination,
            entry,
            verify_options,
            None,
        )?;
        if mismatch_opt.is_none() {
            pre_verified_ok.insert(file_name.to_string());
        } else {
            progress.message(&format!(
                "Resumed partial download for '{file_name}' did not match manifest checksum; re-downloading cleanly..."
            ));
            drop(fs::remove_file(&destination));
            if part_path.is_file() {
                drop(fs::remove_file(&part_path));
            }
            let retry_fp =
                batch.start_transient_file("Retry", file_name, known_size);
            let retried_bytes = client.download_file(
                identifier,
                file_name,
                &destination,
                known_size,
                Some(&retry_fp),
            )?;
            retry_fp.finish(retried_bytes);
        }
    }
    Ok(())
}

fn checkeddl_with_client(
    client: &dyn ArchiveClient,
    target: &str,
    output_dir: Option<&Path>,
    original: bool,
    progress: Progress,
) -> Result<Vec<u8>> {
    let archive_target = parse_archive_target(target)?;
    let base_output_dir = resolve_output_dir(output_dir)?;
    let item_directory = base_output_dir.join(&archive_target.identifier);
    fs::create_dir_all(&item_directory).with_context(|| {
        format!("Failed to create {}", item_directory.display())
    })?;

    progress.message(&format!(
        "Downloading item '{}' to {}",
        archive_target.identifier,
        item_directory.display()
    ));

    let live_files_xml_bytes =
        client.fetch_files_xml_bytes(&archive_target.identifier)?;
    let manifest = parse_files_manifest(
        &archive_target.identifier,
        &live_files_xml_bytes,
    )?;
    let verify_options = VerificationOptions {
        files_xml_live_sha1: Some(sha1_hex_for_bytes(&live_files_xml_bytes)),
    };
    let files_xml_name =
        format!("{}{FILES_XML_SUFFIX}", archive_target.identifier);
    let files_xml_path = item_directory.join(&files_xml_name);

    let file_specs = resolve_download_file_specs(
        client,
        &archive_target,
        original,
        Some(&manifest),
    )?;
    let batch = start_download_batch(
        &progress,
        &archive_target.identifier,
        &file_specs,
    )?;
    let mut pre_verified_ok = BTreeSet::new();

    for (file_name, known_size) in &file_specs {
        if file_name == &files_xml_name {
            write_atomic_bytes(&files_xml_path, &live_files_xml_bytes)?;
            let xml_len = u64::try_from(live_files_xml_bytes.len())
                .map_err(|e| anyhow!("Invalid files.xml length: {e}"))?;
            let fp = batch.start_file(file_name, Some(xml_len));
            fp.finish(xml_len);
            pre_verified_ok.insert(file_name.clone());
            continue;
        }
        checkeddl_download_or_resume_file(
            client,
            &item_directory,
            &archive_target.identifier,
            file_name,
            *known_size,
            &manifest,
            &verify_options,
            &batch,
            &progress,
            &mut pre_verified_ok,
        )?;
    }

    batch.finish();

    if !files_xml_path.exists() || archive_target.archive_path.is_none() {
        write_atomic_bytes(&files_xml_path, &live_files_xml_bytes)?;
    }

    progress.message("Verifying downloaded files...");
    let selected_files =
        if let Some(file_name) = archive_target.archive_path.clone() {
            let mut files = BTreeSet::new();
            if !original
                || manifest
                    .entries
                    .get(&file_name)
                    .and_then(|entry| entry.source.as_deref())
                    == Some("original")
            {
                files.insert(file_name);
            }
            Some(files)
        } else if original {
            Some(
                manifest
                    .entries
                    .values()
                    .filter(|entry| entry.source.as_deref() == Some("original"))
                    .map(|entry| entry.name.clone())
                    .collect(),
            )
        } else {
            None
        };
    let verification = verify_against_manifest_with_preverified(
        &item_directory,
        &archive_target.identifier,
        &manifest,
        selected_files.as_ref(),
        archive_target.archive_path.is_none(),
        &verify_options,
        &pre_verified_ok,
        Some(&progress),
    )?;
    progress.message("Verification complete.");
    pretty_json_bytes(&verification)
}

fn download_as_stream_with_client(
    client: &dyn ArchiveClient,
    target: &str,
) -> Result<Vec<u8>> {
    let archive_target = parse_archive_target(target)?;
    let Some(file_name) = archive_target.archive_path else {
        bail!("downloadAsStream requires an item file path or download URL");
    };
    client.fetch_download_bytes(&archive_target.identifier, &file_name)
}

fn download_here_with_client(
    client: &dyn ArchiveClient,
    target: &str,
    output_dir: Option<&Path>,
    progress: Progress,
) -> Result<Vec<u8>> {
    let archive_target = parse_archive_target(target)?;
    let Some(file_name) = archive_target.archive_path.clone() else {
        bail!("downloadHere requires an item file path or download URL");
    };
    let base_output_dir = resolve_output_dir(output_dir)?;
    let leaf_name = Path::new(&file_name)
        .file_name()
        .and_then(std::ffi::OsStr::to_str)
        .ok_or_else(|| {
            anyhow!("Could not determine local file name for {file_name}")
        })?;
    let output_path = base_output_dir.join(leaf_name);
    let batch = progress.start_file_batch(
        &format!("Overall ({})", archive_target.identifier),
        Some(1),
        None,
    );
    let file_progress = batch.start_file(&file_name, None);
    let bytes_written = client.download_file(
        &archive_target.identifier,
        &file_name,
        &output_path,
        None,
        Some(&file_progress),
    )?;
    file_progress.finish(bytes_written);
    batch.finish();
    let result = DownloadHereResult {
        identifier: archive_target.identifier,
        output_path: output_path.display().to_string(),
    };
    pretty_json_bytes(&result)
}

fn normalize_item_root(item_path: &Path) -> PathBuf {
    if item_path.is_file() {
        item_path
            .parent()
            // Reason for fallback: when item_path is a root file without a parent directory, falling back to item_path itself maintains a valid PathBuf root.
            .map_or_else(|| item_path.to_path_buf(), Path::to_path_buf)
    } else {
        item_path.to_path_buf()
    }
}

fn resolve_identifier(
    item_path: &Path,
    identifier: Option<&str>,
) -> Result<String> {
    if let Some(identifier) = identifier {
        let archive_target = parse_archive_target(identifier)?;
        return Ok(archive_target.identifier);
    }
    guess_identifer_from_path(item_path).ok_or_else(|| {
        anyhow!("Could not determine Internet Archive identifier")
    })
}

fn resolve_output_dir(output_dir: Option<&Path>) -> Result<PathBuf> {
    if let Some(output_dir) = output_dir {
        return Ok(output_dir.to_path_buf());
    }
    std::env::current_dir().context("Failed to determine current directory")
}

fn resolve_hash_target(
    target: &str,
    identifier: Option<&str>,
) -> Result<HashTarget> {
    let target_path = Path::new(target);
    if target_path.exists() {
        let root = normalize_item_root(target_path);
        let identifier = resolve_identifier(target_path, identifier)?;
        let file_name = if target_path.is_file() {
            relative_archive_name(&root, target_path)?
        } else {
            bail!("A file path is required when hashing a local target");
        };
        return Ok(HashTarget {
            identifier,
            file_name,
            local_root: Some(root),
            local_file_path: Some(target_path.to_path_buf()),
        });
    }

    let archive_target = parse_archive_target(target)?;
    let Some(file_name) = archive_target.archive_path else {
        bail!("A file path inside the item is required");
    };
    Ok(HashTarget {
        identifier: archive_target.identifier,
        file_name,
        local_root: None,
        local_file_path: None,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HashTarget {
    identifier: String,
    file_name: String,
    local_root: Option<PathBuf>,
    local_file_path: Option<PathBuf>,
}

fn sha1_for_manifest_entry(
    target: &HashTarget,
    manifest: &FilesManifest,
    check_live: bool,
) -> Result<String> {
    if target.file_name == format!("{}{FILES_XML_SUFFIX}", target.identifier) {
        if check_live {
            let xml_bytes = fetch_live_files_xml_bytes(&target.identifier)?;
            return Ok(sha1_hex_for_bytes(&xml_bytes));
        }

        let local_file_path =
            if let Some(local_file_path) = &target.local_file_path {
                local_file_path.clone()
            } else if let Some(local_root) = &target.local_root {
                local_root.join(&target.file_name)
            } else {
                bail!(
                    "A local files XML copy is required to hash {0}",
                    target.file_name
                );
            };
        return sha1_hex_for_file(&local_file_path);
    }

    if let Some(entry) = manifest.entries.get(&target.file_name) {
        if let Some(sha1) = &entry.sha1 {
            return Ok(sha1.clone());
        }
    }

    if let Some(local_file_path) = &target.local_file_path {
        return sha1_hex_for_file(local_file_path);
    }

    bail!(
        "No sha1 value is available for {} in item {}",
        target.file_name,
        target.identifier
    )
}

fn md5_for_manifest_entry(
    target: &HashTarget,
    manifest: &FilesManifest,
    check_live: bool,
) -> Result<String> {
    if target.file_name == format!("{}{FILES_XML_SUFFIX}", target.identifier) {
        if check_live {
            let xml_bytes = fetch_live_files_xml_bytes(&target.identifier)?;
            return Ok(md5_hex_for_bytes(&xml_bytes));
        }

        let local_file_path =
            if let Some(local_file_path) = &target.local_file_path {
                local_file_path.clone()
            } else if let Some(local_root) = &target.local_root {
                local_root.join(&target.file_name)
            } else {
                bail!(
                    "A local files XML copy is required to hash {0}",
                    target.file_name
                );
            };
        return md5_hex_for_file(&local_file_path);
    }

    if let Some(entry) = manifest.entries.get(&target.file_name) {
        if let Some(md5) = &entry.md5 {
            return Ok(md5.clone());
        }
    }

    if let Some(local_file_path) = &target.local_file_path {
        return md5_hex_for_file(local_file_path);
    }

    bail!(
        "No md5 value is available for {} in item {}",
        target.file_name,
        target.identifier
    )
}

fn load_manifest_for_hash_target(target: &HashTarget) -> Result<FilesManifest> {
    if let Some(local_root) = &target.local_root {
        return load_local_files_manifest(local_root, &target.identifier);
    }
    fetch_live_files_manifest(&target.identifier)
}

fn verify_against_manifest(
    item_root: &Path,
    identifier: &str,
    manifest: &FilesManifest,
    selected_files: Option<&BTreeSet<String>>,
    include_unexpected: bool,
    options: &VerificationOptions,
    progress: Option<&Progress>,
) -> Result<VerificationResult> {
    let empty_preverified = BTreeSet::new();
    verify_against_manifest_with_preverified(
        item_root,
        identifier,
        manifest,
        selected_files,
        include_unexpected,
        options,
        &empty_preverified,
        progress,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "Internal manifest verifier accepting optional pre-verified set from resumed checkeddl"
)]
fn verify_against_manifest_with_preverified(
    item_root: &Path,
    identifier: &str,
    manifest: &FilesManifest,
    selected_files: Option<&BTreeSet<String>>,
    include_unexpected: bool,
    options: &VerificationOptions,
    pre_verified_ok: &BTreeSet<String>,
    progress: Option<&Progress>,
) -> Result<VerificationResult> {
    let selected_names: BTreeSet<String> =
        if let Some(selected_files) = selected_files {
            selected_files.clone()
        } else {
            manifest.entries.keys().cloned().collect()
        };

    let total_files = selected_names.len();
    let total_files_u64 = u64::try_from(total_files)
        .map_err(|e| anyhow!("Failed to convert verify count to u64: {e}"))?;
    let known_verify_bytes: u64 = selected_names
        .iter()
        .filter_map(|name| manifest.entries.get(name).and_then(|e| e.size))
        .fold(0_u64, u64::saturating_add);
    let any_known_size = selected_names
        .iter()
        .any(|name| manifest.entries.get(name).and_then(|e| e.size).is_some());

    let verify_batch = progress.map(|prog| {
        prog.start_file_batch(
            &format!("Verifying ({identifier})"),
            Some(total_files_u64),
            any_known_size.then_some(known_verify_bytes),
        )
    });

    let mut checked_files = Vec::new();
    let mut missing_files = Vec::new();
    let mut mismatched_files = Vec::new();

    for file_name in &selected_names {
        let Some(entry) = manifest.entries.get(file_name) else {
            missing_files.push(file_name.clone());
            if let Some(ref batch) = verify_batch {
                batch.record_skipped_file();
            }
            continue;
        };
        let local_path = item_root.join(Path::new(file_name));
        if !local_path.is_file() {
            missing_files.push(file_name.clone());
            if let Some(ref batch) = verify_batch {
                batch.record_skipped_file();
            }
            continue;
        };

        if pre_verified_ok.contains(file_name)
            && let Ok(meta) = fs::metadata(&local_path)
            && entry.size.is_none_or(|sz| sz == meta.len())
        {
            if let Some(ref batch) = verify_batch {
                let fp = batch.start_transient_file(
                    "Verify",
                    file_name,
                    Some(meta.len()),
                );
                fp.finish(meta.len());
            }
            checked_files.push(file_name.clone());
            continue;
        }

        let file_progress = verify_batch.as_ref().map(|batch| {
            batch.start_transient_file("Verify", file_name, entry.size)
        });

        let (mismatch_opt, file_size_hashed) = verify_local_file(
            item_root,
            identifier,
            &local_path,
            entry,
            options,
            file_progress.as_ref(),
        )?;

        if let Some(fp) = file_progress {
            fp.finish(file_size_hashed);
        }

        if let Some(mismatch) = mismatch_opt {
            mismatched_files.push(mismatch);
        } else {
            checked_files.push(file_name.clone());
        }
    }

    if let Some(batch) = verify_batch {
        batch.finish();
    }

    let unexpected_files = if include_unexpected {
        let local_files = collect_local_files(item_root)?;
        let expected_names: BTreeSet<String> =
            manifest.entries.keys().cloned().collect();
        local_files
            .into_iter()
            .filter(|local_name| !expected_names.contains(local_name))
            .collect()
    } else {
        Vec::new()
    };

    checked_files.sort();
    missing_files.sort();
    mismatched_files.sort_by(|left, right| left.name.cmp(&right.name));

    Ok(VerificationResult {
        valid: missing_files.is_empty()
            && mismatched_files.is_empty()
            && unexpected_files.is_empty(),
        identifier: identifier.to_string(),
        checked_files,
        missing_files,
        mismatched_files,
        unexpected_files,
    })
}

fn verify_local_file(
    item_root: &Path,
    identifier: &str,
    local_path: &Path,
    entry: &ManifestEntry,
    options: &VerificationOptions,
    file_progress: Option<&FileItemProgress>,
) -> Result<(Option<VerificationMismatch>, u64)> {
    let actual_size = fs::metadata(local_path)
        .with_context(|| {
            format!("Failed to read metadata for {}", local_path.display())
        })?
        .len();
    if let Some(fp) = file_progress {
        fp.set_total_bytes(actual_size);
    }
    let relative_name = relative_archive_name(item_root, local_path)?;
    let files_xml_name = format!("{identifier}{FILES_XML_SUFFIX}");
    let is_files_xml = relative_name == files_xml_name;
    if is_files_xml && options.files_xml_live_sha1.is_none() {
        return Ok((None, actual_size));
    }

    let mut mismatch = VerificationMismatch {
        name: entry.name.clone(),
        expected_sha1: if is_files_xml {
            options.files_xml_live_sha1.clone()
        } else {
            entry.sha1.clone()
        },
        actual_sha1: None,
        expected_md5: if is_files_xml {
            None
        } else {
            entry.md5.clone()
        },
        actual_md5: None,
        expected_size: entry.size,
        actual_size: Some(actual_size),
    };
    let mut has_mismatch = false;

    if let Some(expected_size) = entry.size
        && expected_size != actual_size
    {
        has_mismatch = true;
    }
    match (&mismatch.expected_sha1, &mismatch.expected_md5) {
        (Some(expected_sha1), Some(expected_md5)) => {
            let (actual_sha1, actual_md5) =
                sha1_and_md5_hex_for_file(local_path, file_progress)?;
            if &actual_sha1 != expected_sha1 {
                mismatch.actual_sha1 = Some(actual_sha1);
                has_mismatch = true;
            }
            if &actual_md5 != expected_md5 {
                mismatch.actual_md5 = Some(actual_md5);
                has_mismatch = true;
            }
        }
        (Some(expected_sha1), None) => {
            let actual_sha1 =
                sha1_hex_for_file_with_progress(local_path, file_progress)?;
            if &actual_sha1 != expected_sha1 {
                mismatch.actual_sha1 = Some(actual_sha1);
                has_mismatch = true;
            }
        }
        (None, Some(expected_md5)) => {
            let actual_md5 =
                md5_hex_for_file_with_progress(local_path, file_progress)?;
            if &actual_md5 != expected_md5 {
                mismatch.actual_md5 = Some(actual_md5);
                has_mismatch = true;
            }
        }
        (None, None) => {}
    }

    if has_mismatch {
        Ok((Some(mismatch), actual_size))
    } else {
        Ok((None, actual_size))
    }
}

fn collect_local_files(item_root: &Path) -> Result<Vec<String>> {
    let mut local_files = Vec::new();
    collect_local_files_inner(item_root, item_root, &mut local_files)?;
    local_files.sort();
    Ok(local_files)
}

fn collect_local_files_inner(
    root: &Path,
    current: &Path,
    local_files: &mut Vec<String>,
) -> Result<()> {
    for entry in fs::read_dir(current)
        .with_context(|| format!("Failed to list {}", current.display()))?
    {
        let entry = entry
            .with_context(|| format!("Failed to read {}", current.display()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_local_files_inner(root, &path, local_files)?;
        } else if path.is_file() {
            local_files.push(relative_archive_name(root, &path)?);
        }
    }
    Ok(())
}

fn relative_archive_name(root: &Path, path: &Path) -> Result<String> {
    let relative_path = path.strip_prefix(root).with_context(|| {
        format!("{} is not within {}", path.display(), root.display())
    })?;
    let mut parts = Vec::new();
    for component in relative_path.components() {
        if let Component::Normal(part) = component {
            parts.push(part.to_string_lossy().into_owned());
        }
    }
    Ok(parts.join("/"))
}

fn load_local_files_manifest(
    item_root: &Path,
    identifier: &str,
) -> Result<FilesManifest> {
    let files_xml_path =
        item_root.join(format!("{identifier}{FILES_XML_SUFFIX}"));
    let bytes = fs::read(&files_xml_path).with_context(|| {
        format!("Failed to read {}", files_xml_path.display())
    })?;
    parse_files_manifest(identifier, &bytes)
}

const CHUNK_RETRY_BASE_DELAY_MS: u64 = 500;

fn fetch_live_files_manifest(identifier: &str) -> Result<FilesManifest> {
    let bytes = fetch_live_files_xml_bytes(identifier)?;
    parse_files_manifest(identifier, &bytes)
}

fn fetch_live_files_xml_bytes(identifier: &str) -> Result<Vec<u8>> {
    let response = perform_get(&format!(
        "https://archive.org/download/{identifier}/{identifier}{FILES_XML_SUFFIX}"
    ))?;
    response.bytes().context("Failed to read files XML body")
}

fn fetch_live_meta_xml_bytes(identifier: &str) -> Result<Vec<u8>> {
    let response = perform_get(&format!(
        "https://archive.org/download/{identifier}/{identifier}{META_XML_SUFFIX}"
    ))?;
    response.bytes().context("Failed to read meta XML body")
}

fn fetch_live_metadata(identifier: &str) -> Result<MetadataResponse> {
    let response =
        perform_get(&format!("https://archive.org/metadata/{identifier}"))?;
    let body = response.text().context("Failed to read metadata body")?;
    serde_json::from_str(&body)
        .context("Failed to parse Internet Archive metadata JSON")
}

const DOWNLOAD_CHUNK_SIZE: u64 = 8_388_608; // 8 MiB

trait RangeHttpResponse: Read {
    fn status(&self) -> reqwest::StatusCode;
    fn headers(&self) -> &reqwest::header::HeaderMap;
    fn content_length(&self) -> Option<u64>;
}

impl RangeHttpResponse for https::BlockingResponse {
    fn status(&self) -> reqwest::StatusCode {
        https::BlockingResponse::status(self)
    }

    fn headers(&self) -> &reqwest::header::HeaderMap {
        https::BlockingResponse::headers(self)
    }

    fn content_length(&self) -> Option<u64> {
        https::BlockingResponse::content_length(self)
    }
}

trait RangeHttpClient {
    fn get_with_headers(
        &self,
        url: &str,
        headers: Option<reqwest::header::HeaderMap>,
    ) -> Result<Box<dyn RangeHttpResponse>>;
}

struct PooledRangeHttpClient;

impl RangeHttpClient for PooledRangeHttpClient {
    fn get_with_headers(
        &self,
        url: &str,
        headers: Option<reqwest::header::HeaderMap>,
    ) -> Result<Box<dyn RangeHttpResponse>> {
        let resp = https::blocking_get_response_with_headers(url, headers)?;
        Ok(Box::new(resp))
    }
}

fn parse_content_range(val: &str) -> Option<(u64, u64, Option<u64>)> {
    let s = val.trim();
    let s = s.strip_prefix("bytes ")?;
    let (range_part, total_part) = s.split_once('/')?;
    let (start_str, end_str) = range_part.split_once('-')?;
    let start: u64 = start_str.trim().parse().ok()?;
    let end: u64 = end_str.trim().parse().ok()?;
    let total = if total_part.trim() == "*" {
        None
    } else {
        total_part.trim().parse().ok()
    };
    Some((start, end, total))
}

fn parse_unsatisfied_content_range_total(val: &str) -> Option<u64> {
    let s = val.trim().strip_prefix("bytes ")?;
    let (range_part, total_part) = s.split_once('/')?;
    if range_part.trim() != "*" {
        return None;
    }
    total_part.trim().parse().ok()
}

fn part_path_for_destination(destination: &Path) -> PathBuf {
    let mut os_str = destination.as_os_str().to_os_string();
    os_str.push(PART_FILE_SUFFIX);
    PathBuf::from(os_str)
}

fn create_temp_path(parent_dir: &Path, file_name: &str) -> PathBuf {
    static ATOMIC_SEQ: std::sync::atomic::AtomicU64 =
        std::sync::atomic::AtomicU64::new(0);
    let pid = std::process::id();
    let nanos = match std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
    {
        Ok(dur) => dur.subsec_nanos(),
        Err(_) => 0,
    };
    let seq = ATOMIC_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    // Reason for fallback: If file_name has no valid leaf or non-utf8 path, use generic prefix for temporary file.
    let leaf_name = Path::new(file_name)
        .file_name()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or("file");
    let temp_name = format!(".{leaf_name}.ia-tmp.{pid}.{nanos}.{seq}");
    parent_dir.join(temp_name)
}

struct TempFileCleanupGuard {
    temp_path: PathBuf,
    active: bool,
}

impl Drop for TempFileCleanupGuard {
    fn drop(&mut self) {
        if self.active {
            drop(fs::remove_file(&self.temp_path));
        }
    }
}

fn commit_atomic_file(temp_path: &Path, destination: &Path) -> Result<()> {
    #[cfg(windows)]
    if destination.exists() {
        drop(fs::remove_file(destination));
    }
    fs::rename(temp_path, destination).with_context(|| {
        format!(
            "Failed to atomically rename {} to {}",
            temp_path.display(),
            destination.display()
        )
    })
}

fn write_atomic_bytes(destination: &Path, bytes: &[u8]) -> Result<()> {
    // Reason for fallback: If destination is a bare filename without a parent directory, stage in current directory.
    let parent = destination.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).with_context(|| {
        format!("Failed to create parent directory {}", parent.display())
    })?;
    // Reason for fallback: Non-UTF-8 leaf names use a generic prefix for the transient atomic staging file.
    let name_str = destination
        .file_name()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or("file");
    let temp_path = create_temp_path(parent, name_str);
    let mut guard = TempFileCleanupGuard {
        temp_path: temp_path.clone(),
        active: true,
    };
    fs::write(&temp_path, bytes).with_context(|| {
        format!("Failed to write temporary file {}", temp_path.display())
    })?;
    commit_atomic_file(&temp_path, destination)?;
    guard.active = false;
    Ok(())
}

fn fetch_download_bytes(identifier: &str, file_name: &str) -> Result<Vec<u8>> {
    let url = download_url(identifier, file_name)?;
    let mut buffer = Vec::new();
    download_to_writer_chunked_range_with_http(
        &PooledRangeHttpClient,
        url.as_str(),
        &mut buffer,
        None,
        DOWNLOAD_CHUNK_SIZE,
        CHUNK_RETRY_BASE_DELAY_MS,
    )?;
    Ok(buffer)
}

fn download_live_file_chunked_range(
    identifier: &str,
    file_name: &str,
    destination: &Path,
    expected_size: Option<u64>,
    file_progress: Option<&FileItemProgress>,
) -> Result<u64> {
    let url = download_url(identifier, file_name)?;
    download_to_part_file_chunked_range_with_http(
        &PooledRangeHttpClient,
        url.as_str(),
        destination,
        expected_size,
        file_progress,
        DOWNLOAD_CHUNK_SIZE,
        CHUNK_RETRY_BASE_DELAY_MS,
    )
}

enum InitialChunkOutcome {
    FullStream(Box<dyn RangeHttpResponse>),
    EmptyFile,
    Partial {
        bytes: Vec<u8>,
        total_size: Option<u64>,
    },
}

enum ResumeChunkOutcome {
    AlreadyComplete(u64),
    RestartFromZero,
    FullStreamFromZero(Box<dyn RangeHttpResponse>),
    Partial {
        bytes: Vec<u8>,
        total_size: Option<u64>,
    },
}

fn sleep_chunk_retry(attempt: usize, backoff_base_ms: u64) {
    if backoff_base_ms == 0 {
        return;
    }
    // Reason for fallback: If attempt count cannot convert to u32, cap exponent to bound exponential backoff.
    let exponent = u32::try_from(attempt).unwrap_or(4).min(4);
    // Reason for fallback: power computation overflow safely caps multiplier
    let multiplier = 2u64.checked_pow(exponent).unwrap_or(16);
    let delay_ms = backoff_base_ms.saturating_mul(multiplier);
    std::thread::sleep(std::time::Duration::from_millis(delay_ms));
}

fn read_response_to_vec_with_progress(
    response: &mut dyn RangeHttpResponse,
    base_offset: u64,
    file_progress: Option<&FileItemProgress>,
) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    let mut buf = [0_u8; 64 * 1024];
    loop {
        let n = response
            .read(&mut buf)
            .context("Failed to read chunk response body")?;
        if n == 0 {
            break;
        }
        let slice = buf.get(..n).context("Buffer slice index out of bounds")?;
        out.extend_from_slice(slice);
        if let Some(fp) = file_progress
            && let Ok(cur_u64) = u64::try_from(out.len())
        {
            fp.set_bytes_done(base_offset.saturating_add(cur_u64));
        }
    }
    Ok(out)
}

fn fetch_initial_chunk_with_retry(
    http: &dyn RangeHttpClient,
    url: &str,
    chunk_size: u64,
    retry_limit: usize,
    backoff_base_ms: u64,
    file_progress: Option<&FileItemProgress>,
) -> Result<InitialChunkOutcome> {
    let first_end = chunk_size.saturating_sub(1);
    let range_str = format!("bytes=0-{first_end}");
    let mut attempt = 0;

    loop {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::RANGE,
            reqwest::header::HeaderValue::from_str(&range_str)
                .context("Failed to format initial Range header")?,
        );

        let attempt_res: Result<InitialChunkOutcome> = (|| {
            let mut response = http
                .get_with_headers(url, Some(headers))
                .with_context(|| format!("Failed to connect to {url}"))?;
            let status = response.status();
            if status == reqwest::StatusCode::OK {
                return Ok(InitialChunkOutcome::FullStream(response));
            }
            if status == reqwest::StatusCode::RANGE_NOT_SATISFIABLE {
                let unsatisfied_total = response
                    .headers()
                    .get(reqwest::header::CONTENT_RANGE)
                    .and_then(|v| v.to_str().ok())
                    .and_then(parse_unsatisfied_content_range_total);
                if unsatisfied_total == Some(0) {
                    return Ok(InitialChunkOutcome::EmptyFile);
                }
                let fallback = http
                    .get_with_headers(url, None)
                    .with_context(|| format!("Failed to GET {url}"))?;
                if fallback.status() == reqwest::StatusCode::OK {
                    return Ok(InitialChunkOutcome::FullStream(fallback));
                }
                bail!(
                    "Unexpected HTTP response status {} after 416 fallback for {url}",
                    fallback.status()
                );
            }
            if status != reqwest::StatusCode::PARTIAL_CONTENT {
                bail!("Unexpected HTTP response status {status} for {url}");
            }

            let parsed_range = response
                .headers()
                .get(reqwest::header::CONTENT_RANGE)
                .and_then(|v| v.to_str().ok())
                .and_then(parse_content_range);
            if let Some((range_start, _, _)) = parsed_range {
                anyhow::ensure!(
                    range_start == 0,
                    "Unexpected Content-Range start {range_start} for initial chunk of {url}"
                );
            }
            let mut total_size = parsed_range.and_then(|(_, _, total)| total);
            if total_size.is_none()
                && let Some(content_len) = response.content_length()
                && content_len < chunk_size
            {
                total_size = Some(content_len);
            }
            if let (Some(fp), Some(total)) = (file_progress, total_size) {
                fp.set_total_bytes(total);
            }

            let chunk_buf = read_response_to_vec_with_progress(
                response.as_mut(),
                0,
                file_progress,
            )
            .context("Failed to read first chunk body")?;
            if chunk_buf.is_empty() && total_size != Some(0) {
                bail!(
                    "Received empty body for initial byte range 0-{first_end}"
                );
            }
            Ok(InitialChunkOutcome::Partial {
                bytes: chunk_buf,
                total_size,
            })
        })();

        match attempt_res {
            Ok(outcome) => return Ok(outcome),
            Err(err) => {
                if attempt < retry_limit {
                    attempt = attempt.saturating_add(1);
                    warn_fmt!(
                        "Retrying initial chunk 0-{first_end} for {url} (attempt {attempt}/{retry_limit}): {err}"
                    );
                    sleep_chunk_retry(attempt, backoff_base_ms);
                    continue;
                }
                return Err(err);
            }
        }
    }
}

fn stream_full_response_to_writer(
    mut response: Box<dyn RangeHttpResponse>,
    url: &str,
    writer: &mut dyn Write,
    file_progress: Option<&FileItemProgress>,
) -> Result<u64> {
    let expected_len = response.content_length();
    if let (Some(fp), Some(expected)) = (file_progress, expected_len) {
        fp.set_total_bytes(expected);
    }
    let mut total_copied = 0_u64;
    let mut buf = [0_u8; 64 * 1024];
    loop {
        let n = response
            .read(&mut buf)
            .context("Failed to read HTTP response body")?;
        if n == 0 {
            break;
        }
        let n_u64 = u64::try_from(n).map_err(|e| {
            anyhow!("Failed to convert read buffer length: {e}")
        })?;
        let slice = buf.get(..n).context("Buffer slice index out of bounds")?;
        writer
            .write_all(slice)
            .context("Failed to write downloaded bytes to stream")?;
        total_copied = total_copied.saturating_add(n_u64);
        if let Some(fp) = file_progress {
            fp.set_bytes_done(total_copied);
        }
    }
    if let Some(expected) = expected_len {
        anyhow::ensure!(
            total_copied == expected,
            "Incomplete download for {url}: received {total_copied} of {expected} bytes"
        );
    }
    Ok(total_copied)
}

#[expect(
    clippy::too_many_arguments,
    reason = "Internal HTTP range resumption helper with explicit retry and progress parameters"
)]
fn fetch_resume_chunk_with_retry(
    http: &dyn RangeHttpClient,
    url: &str,
    resume_offset: u64,
    resume_end: u64,
    expected_size: Option<u64>,
    retry_limit: usize,
    backoff_base_ms: u64,
    file_progress: Option<&FileItemProgress>,
) -> Result<ResumeChunkOutcome> {
    let range_str = format!("bytes={resume_offset}-{resume_end}");
    let mut attempt = 0;

    loop {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::RANGE,
            reqwest::header::HeaderValue::from_str(&range_str)
                .context("Failed to format resume Range header")?,
        );

        let attempt_res: Result<ResumeChunkOutcome> = (|| {
            let mut response = http
                .get_with_headers(url, Some(headers))
                .with_context(|| format!("Failed to connect to {url}"))?;
            let status = response.status();
            if status == reqwest::StatusCode::OK {
                return Ok(ResumeChunkOutcome::FullStreamFromZero(response));
            }
            if status == reqwest::StatusCode::RANGE_NOT_SATISFIABLE {
                let unsatisfied_total = response
                    .headers()
                    .get(reqwest::header::CONTENT_RANGE)
                    .and_then(|v| v.to_str().ok())
                    .and_then(parse_unsatisfied_content_range_total);
                if unsatisfied_total == Some(resume_offset) {
                    return Ok(ResumeChunkOutcome::AlreadyComplete(
                        resume_offset,
                    ));
                }
                return Ok(ResumeChunkOutcome::RestartFromZero);
            }
            if status != reqwest::StatusCode::PARTIAL_CONTENT {
                bail!(
                    "Unexpected HTTP response status {status} while resuming {url}"
                );
            }

            let parsed_range = response
                .headers()
                .get(reqwest::header::CONTENT_RANGE)
                .and_then(|v| v.to_str().ok())
                .and_then(parse_content_range);
            if let Some((range_start, _, _)) = parsed_range {
                anyhow::ensure!(
                    range_start == resume_offset,
                    "Unexpected Content-Range start {range_start} (expected {resume_offset}) while resuming {url}"
                );
            }
            let total_size = parsed_range
                .and_then(|(_, _, total)| total)
                .or(expected_size);
            if let Some(total) = total_size
                && resume_offset > total
            {
                return Ok(ResumeChunkOutcome::RestartFromZero);
            }
            if let (Some(fp), Some(total)) = (file_progress, total_size) {
                fp.set_total_bytes(total);
            }

            let chunk_buf = read_response_to_vec_with_progress(
                response.as_mut(),
                resume_offset,
                file_progress,
            )
            .context("Failed to read resumed chunk body")?;
            if chunk_buf.is_empty() && total_size != Some(resume_offset) {
                bail!(
                    "Received empty body for resumed byte range {resume_offset}-{resume_end}"
                );
            }
            Ok(ResumeChunkOutcome::Partial {
                bytes: chunk_buf,
                total_size,
            })
        })();

        match attempt_res {
            Ok(outcome) => return Ok(outcome),
            Err(err) => {
                if attempt < retry_limit {
                    attempt = attempt.saturating_add(1);
                    warn_fmt!(
                        "Retrying resumed chunk {resume_offset}-{resume_end} for {url} (attempt {attempt}/{retry_limit}): {err}"
                    );
                    sleep_chunk_retry(attempt, backoff_base_ms);
                    continue;
                }
                return Err(err);
            }
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Shared chunked HTTP Range download loop across file and stream writers"
)]
fn continue_chunked_range_loop(
    http: &dyn RangeHttpClient,
    url: &str,
    writer: &mut dyn Write,
    initial_offset: u64,
    initial_chunk_len: u64,
    total_size: Option<u64>,
    chunk_size: u64,
    retry_limit: usize,
    backoff_base_ms: u64,
    file_progress: Option<&FileItemProgress>,
) -> Result<u64> {
    let mut current_offset = initial_offset;
    if let Some(fp) = file_progress {
        fp.set_bytes_done(current_offset);
    }

    if let Some(total) = total_size {
        if current_offset >= total {
            anyhow::ensure!(
                current_offset == total,
                "Received {current_offset} bytes exceeding expected size {total} for {url}"
            );
            return Ok(current_offset);
        }
    } else if initial_chunk_len < chunk_size {
        if let Some(fp) = file_progress {
            fp.set_total_bytes(current_offset);
        }
        return Ok(current_offset);
    }

    loop {
        if let Some(total) = total_size
            && current_offset >= total
        {
            break;
        }

        let next_end = if let Some(total) = total_size {
            std::cmp::min(
                current_offset.saturating_add(chunk_size).saturating_sub(1),
                total.saturating_sub(1),
            )
        } else {
            current_offset.saturating_add(chunk_size).saturating_sub(1)
        };

        if next_end < current_offset {
            break;
        }

        let chunk_bytes_opt = fetch_chunk_with_retry(
            http,
            url,
            current_offset,
            next_end,
            total_size,
            retry_limit,
            backoff_base_ms,
            file_progress,
        )?;

        let Some(chunk_bytes) = chunk_bytes_opt else {
            break;
        };

        let chunk_len = u64::try_from(chunk_bytes.len()).map_err(|e| {
            anyhow!("Failed to convert chunk bytes length to u64: {e}")
        })?;

        writer
            .write_all(&chunk_bytes)
            .context("Failed to write chunk bytes to destination")?;
        writer
            .flush()
            .context("Failed to flush chunk bytes to destination")?;

        current_offset = current_offset.saturating_add(chunk_len);
        if let Some(fp) = file_progress {
            fp.set_bytes_done(current_offset);
        }

        if chunk_len < chunk_size && total_size.is_none() {
            break;
        }
    }

    if let Some(total) = total_size {
        anyhow::ensure!(
            current_offset == total,
            "Incomplete download for {url}: received {current_offset} of {total} bytes"
        );
    } else if let Some(fp) = file_progress {
        fp.set_total_bytes(current_offset);
    }

    Ok(current_offset)
}

fn download_to_writer_chunked_range_with_http(
    http: &dyn RangeHttpClient,
    url: &str,
    writer: &mut dyn Write,
    file_progress: Option<&FileItemProgress>,
    chunk_size: u64,
    backoff_base_ms: u64,
) -> Result<u64> {
    anyhow::ensure!(chunk_size > 0, "Download chunk size must be non-zero");
    let retry_limit = invocation_settings::get_settings()
        .retry_on_host_error
        .max(3);

    let (first_chunk, total_size) = match fetch_initial_chunk_with_retry(
        http,
        url,
        chunk_size,
        retry_limit,
        backoff_base_ms,
        file_progress,
    )? {
        InitialChunkOutcome::EmptyFile => {
            if let Some(fp) = file_progress {
                fp.set_total_bytes(0);
                fp.set_bytes_done(0);
            }
            return Ok(0);
        }
        InitialChunkOutcome::FullStream(response) => {
            return stream_full_response_to_writer(
                response,
                url,
                writer,
                file_progress,
            );
        }
        InitialChunkOutcome::Partial { bytes, total_size } => {
            (bytes, total_size)
        }
    };

    let first_chunk_len = u64::try_from(first_chunk.len())
        .map_err(|e| anyhow!("Failed to convert chunk size to u64: {e}"))?;
    writer
        .write_all(&first_chunk)
        .context("Failed to write first chunk to destination")?;
    writer
        .flush()
        .context("Failed to flush first chunk to destination")?;

    continue_chunked_range_loop(
        http,
        url,
        writer,
        first_chunk_len,
        first_chunk_len,
        total_size,
        chunk_size,
        retry_limit,
        backoff_base_ms,
        file_progress,
    )
}

fn finalize_part_file(
    mut part_file: File,
    part_path: &Path,
    destination: &Path,
    total_bytes: u64,
) -> Result<u64> {
    part_file.sync_all().with_context(|| {
        format!(
            "Failed to sync partial download file {}",
            part_path.display()
        )
    })?;
    drop(part_file);
    commit_atomic_file(part_path, destination)?;
    Ok(total_bytes)
}

fn reset_part_file_to_zero(
    part_file: &mut File,
    part_path: &Path,
) -> Result<()> {
    part_file.set_len(0).with_context(|| {
        format!("Failed to truncate {}", part_path.display())
    })?;
    let _ = part_file.seek(SeekFrom::Start(0)).with_context(|| {
        format!("Failed to seek to start of {}", part_path.display())
    })?;
    Ok(())
}

#[expect(
    clippy::too_many_arguments,
    reason = "Internal .part resumption helper taking explicit file handle, destination paths, and range parameters"
)]
fn resume_existing_part_file(
    http: &dyn RangeHttpClient,
    url: &str,
    mut part_file: File,
    part_path: &Path,
    destination: &Path,
    existing_len: u64,
    expected_size: Option<u64>,
    file_progress: Option<&FileItemProgress>,
    chunk_size: u64,
    retry_limit: usize,
    backoff_base_ms: u64,
) -> Result<u64> {
    part_file.seek(SeekFrom::End(0)).with_context(|| {
        format!("Failed to seek to end of {}", part_path.display())
    })?;
    if let Some(fp) = file_progress {
        if let Some(expected) = expected_size {
            fp.set_total_bytes(expected);
        }
        fp.set_bytes_done(existing_len);
    }
    let resume_end = if let Some(expected) = expected_size {
        std::cmp::min(
            existing_len.saturating_add(chunk_size).saturating_sub(1),
            expected.saturating_sub(1),
        )
    } else {
        existing_len.saturating_add(chunk_size).saturating_sub(1)
    };
    match fetch_resume_chunk_with_retry(
        http,
        url,
        existing_len,
        resume_end,
        expected_size,
        retry_limit,
        backoff_base_ms,
        file_progress,
    )? {
        ResumeChunkOutcome::AlreadyComplete(total) => {
            if let Some(fp) = file_progress {
                fp.set_total_bytes(total);
                fp.set_bytes_done(total);
            }
            finalize_part_file(part_file, part_path, destination, total)
        }
        ResumeChunkOutcome::FullStreamFromZero(response) => {
            reset_part_file_to_zero(&mut part_file, part_path)?;
            let written = stream_full_response_to_writer(
                response,
                url,
                &mut part_file,
                file_progress,
            )?;
            finalize_part_file(part_file, part_path, destination, written)
        }
        ResumeChunkOutcome::RestartFromZero => {
            reset_part_file_to_zero(&mut part_file, part_path)?;
            let written = download_to_writer_chunked_range_with_http(
                http,
                url,
                &mut part_file,
                file_progress,
                chunk_size,
                backoff_base_ms,
            )?;
            finalize_part_file(part_file, part_path, destination, written)
        }
        ResumeChunkOutcome::Partial { bytes, total_size } => {
            let chunk_len = u64::try_from(bytes.len()).map_err(|e| {
                anyhow!("Failed to convert chunk size to u64: {e}")
            })?;
            part_file.write_all(&bytes).with_context(|| {
                format!("Failed to append to {}", part_path.display())
            })?;
            part_file.flush().with_context(|| {
                format!("Failed to flush {}", part_path.display())
            })?;
            let new_offset = existing_len.saturating_add(chunk_len);
            let total_written = continue_chunked_range_loop(
                http,
                url,
                &mut part_file,
                new_offset,
                chunk_len,
                total_size,
                chunk_size,
                retry_limit,
                backoff_base_ms,
                file_progress,
            )?;
            finalize_part_file(part_file, part_path, destination, total_written)
        }
    }
}

fn download_to_part_file_chunked_range_with_http(
    http: &dyn RangeHttpClient,
    url: &str,
    destination: &Path,
    expected_size: Option<u64>,
    file_progress: Option<&FileItemProgress>,
    chunk_size: u64,
    backoff_base_ms: u64,
) -> Result<u64> {
    anyhow::ensure!(chunk_size > 0, "Download chunk size must be non-zero");
    // Reason for fallback: If destination is a relative file path without an explicit parent directory, use current directory.
    let parent = destination.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).with_context(|| {
        format!("Failed to create parent directory {}", parent.display())
    })?;

    let part_path = part_path_for_destination(destination);
    let mut part_file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(&part_path)
        .with_context(|| {
            format!(
                "Failed to open partial download file {}",
                part_path.display()
            )
        })?;
    let mut existing_len = part_file
        .metadata()
        .with_context(|| {
            format!("Failed to read metadata for {}", part_path.display())
        })?
        .len();

    if let Some(expected) = expected_size {
        if existing_len > expected {
            reset_part_file_to_zero(&mut part_file, &part_path)?;
            existing_len = 0;
        } else if existing_len == expected && expected > 0 {
            if let Some(fp) = file_progress {
                fp.set_total_bytes(expected);
                fp.set_bytes_done(expected);
            }
            return finalize_part_file(
                part_file,
                &part_path,
                destination,
                expected,
            );
        }
    }

    let retry_limit = invocation_settings::get_settings()
        .retry_on_host_error
        .max(3);

    if existing_len > 0 {
        return resume_existing_part_file(
            http,
            url,
            part_file,
            &part_path,
            destination,
            existing_len,
            expected_size,
            file_progress,
            chunk_size,
            retry_limit,
            backoff_base_ms,
        );
    }

    let total_written = download_to_writer_chunked_range_with_http(
        http,
        url,
        &mut part_file,
        file_progress,
        chunk_size,
        backoff_base_ms,
    )?;
    finalize_part_file(part_file, &part_path, destination, total_written)
}

fn fetch_chunk_with_retry(
    http: &dyn RangeHttpClient,
    url: &str,
    start: u64,
    end: u64,
    expected_total: Option<u64>,
    retry_limit: usize,
    backoff_base_ms: u64,
    file_progress: Option<&FileItemProgress>,
) -> Result<Option<Vec<u8>>> {
    let range_val = format!("bytes={start}-{end}");
    let mut attempt = 0;
    loop {
        let mut chunk_headers = reqwest::header::HeaderMap::new();
        chunk_headers.insert(
            reqwest::header::RANGE,
            reqwest::header::HeaderValue::from_str(&range_val)
                .context("Invalid Range header value")?,
        );

        let chunk_read_res: Result<Option<Vec<u8>>> = (|| {
            let mut chunk_resp =
                http.get_with_headers(url, Some(chunk_headers))?;
            let status = chunk_resp.status();
            if status == reqwest::StatusCode::RANGE_NOT_SATISFIABLE {
                if expected_total.is_none() {
                    return Ok(None);
                }
                bail!(
                    "Unexpected HTTP 416 Range Not Satisfiable for chunk {start}-{end}"
                );
            }
            if status != reqwest::StatusCode::PARTIAL_CONTENT {
                bail!(
                    "Unexpected HTTP status {status} for chunk {start}-{end}"
                );
            }

            if let Some((range_start, _, range_total)) = chunk_resp
                .headers()
                .get(reqwest::header::CONTENT_RANGE)
                .and_then(|v| v.to_str().ok())
                .and_then(parse_content_range)
            {
                anyhow::ensure!(
                    range_start == start,
                    "Unexpected Content-Range start {range_start} (expected {start}) for {url}"
                );
                if let (Some(expected), Some(actual)) =
                    (expected_total, range_total)
                {
                    anyhow::ensure!(
                        actual == expected,
                        "Remote file size changed during download of {url}: expected {expected}, got {actual}"
                    );
                }
            }

            let buf = read_response_to_vec_with_progress(
                chunk_resp.as_mut(),
                start,
                file_progress,
            )
            .context("Failed to read chunk response body")?;
            if buf.is_empty() {
                bail!("Received empty body for chunk {start}-{end}");
            }
            Ok(Some(buf))
        })();

        match chunk_read_res {
            Ok(data) => return Ok(data),
            Err(err) => {
                if attempt < retry_limit {
                    attempt = attempt.saturating_add(1);
                    warn_fmt!(
                        "Retrying chunk {start}-{end} for {url} (attempt {attempt}/{retry_limit}): {err}"
                    );
                    sleep_chunk_retry(attempt, backoff_base_ms);
                    continue;
                }
                return Err(err.context(format!(
                    "Failed to download chunk {start}-{end} from {url}"
                )));
            }
        }
    }
}

fn perform_get(url: &str) -> Result<https::BlockingResponse> {
    let response = https::blocking_get_response_with_backoff(url, 3)
        .with_context(|| format!("Failed to GET {url}"))?;
    if !response.is_success() {
        bail!(
            "Unexpected response status {} for {url}",
            response.status_code()
        );
    }
    Ok(response)
}

fn contains_with_client(
    client: &dyn ArchiveClient,
    target: &str,
    desired_file: &str,
) -> Result<Vec<u8>> {
    let archive_target = parse_archive_target(target)?;
    let metadata = client.fetch_metadata(&archive_target.identifier)?;
    let result = metadata.files.iter().any(|file| file.name == desired_file);
    let mut output = if result {
        b"true".to_vec()
    } else {
        b"false".to_vec()
    };
    output.push(b'\n');
    Ok(output)
}

fn listplain_with_client(
    client: &dyn ArchiveClient,
    target: &str,
) -> Result<Vec<u8>> {
    let archive_target = parse_archive_target(target)?;
    let metadata = client.fetch_metadata(&archive_target.identifier)?;
    let mut names: Vec<String> =
        metadata.files.into_iter().map(|file| file.name).collect();
    names.sort();
    let mut output = names.join("\n").into_bytes();
    if !output.is_empty() {
        output.push(b'\n');
    }
    Ok(output)
}

fn metadata_with_client(
    client: &dyn ArchiveClient,
    target: &str,
) -> Result<Vec<u8>> {
    let archive_target = parse_archive_target(target)?;
    let metadata = client.fetch_metadata(&archive_target.identifier)?;
    pretty_json_bytes(&metadata)
}

fn filesxml_with_client(
    client: &dyn ArchiveClient,
    target: &str,
) -> Result<Vec<u8>> {
    let archive_target = parse_archive_target(target)?;
    client.fetch_files_xml_bytes(&archive_target.identifier)
}

fn metaxml_with_client(
    client: &dyn ArchiveClient,
    target: &str,
) -> Result<Vec<u8>> {
    let archive_target = parse_archive_target(target)?;
    client.fetch_meta_xml_bytes(&archive_target.identifier)
}

fn download_url(identifier: &str, file_name: &str) -> Result<reqwest::Url> {
    let mut url = reqwest::Url::parse("https://archive.org/download/")
        .context("Failed to construct download URL")?;
    {
        let mut segments = url
            .path_segments_mut()
            .map_err(|()| anyhow!("Failed to modify download URL segments"))?;
        segments.push(identifier);
        for segment in file_name.split('/') {
            segments.push(segment);
        }
    }
    Ok(url)
}

fn parse_files_manifest(
    identifier: &str,
    bytes: &[u8],
) -> Result<FilesManifest> {
    let cursor = std::io::Cursor::new(bytes);
    let parser = EventReader::new(BufReader::new(cursor));
    let mut entries = BTreeMap::new();
    let mut current_entry: Option<ManifestEntry> = None;
    let mut current_tag: Option<String> = None;
    let mut current_text = String::new();

    for event in parser {
        match event.context("Failed to parse files XML")? {
            XmlEvent::StartElement {
                name, attributes, ..
            } => {
                if name.local_name == "file" {
                    let file_name = attributes
                        .iter()
                        .find(|attribute| attribute.name.local_name == "name")
                        .map(|attribute| attribute.value.clone())
                        .ok_or_else(|| {
                            anyhow!(
                                "Encountered <file> without a name attribute"
                            )
                        })?;
                    let source = attributes
                        .iter()
                        .find(|attribute| attribute.name.local_name == "source")
                        .map(|attribute| attribute.value.clone());
                    current_entry = Some(ManifestEntry {
                        name: file_name,
                        source,
                        md5: None,
                        sha1: None,
                        size: None,
                    });
                    current_tag = None;
                    current_text.clear();
                } else if current_entry.is_some() {
                    current_tag = Some(name.local_name);
                    current_text.clear();
                }
            }
            XmlEvent::Characters(text) | XmlEvent::CData(text) => {
                if current_entry.is_some() && current_tag.is_some() {
                    current_text.push_str(&text);
                }
            }
            XmlEvent::EndElement { name } => {
                if name.local_name == "file" {
                    if let Some(entry) = current_entry.take() {
                        entries.insert(entry.name.clone(), entry);
                    }
                    current_tag = None;
                    current_text.clear();
                    continue;
                }

                if let Some(entry) = current_entry.as_mut() {
                    if let Some(tag) = &current_tag {
                        let value = current_text.trim();
                        match tag.as_str() {
                            "md5" if !value.is_empty() => {
                                entry.md5 = Some(value.to_string());
                            }
                            "sha1" if !value.is_empty() => {
                                entry.sha1 = Some(value.to_string());
                            }
                            "size" if !value.is_empty() => {
                                entry.size = value.parse::<u64>().ok();
                            }
                            _ => {}
                        }
                    }
                }

                current_tag = None;
                current_text.clear();
            }
            XmlEvent::EndDocument => break,
            _ => {}
        }
    }

    Ok(FilesManifest {
        identifier: identifier.to_string(),
        entries,
    })
}

fn is_archive_org_host(host: &str) -> bool {
    // Reason for fallback: if host has no colon separator, use host as-is.
    let host_no_port = host.split(':').next().unwrap_or(host);
    // Reason for fallback: if host has no trailing dot, use host_no_port as-is.
    let host_clean = host_no_port.strip_suffix('.').unwrap_or(host_no_port);
    host_clean.eq_ignore_ascii_case("archive.org")
        || host_clean.to_ascii_lowercase().ends_with(".archive.org")
}

fn is_wayback_host(host: &str) -> bool {
    // Reason for fallback: if host has no colon separator, use host as-is.
    let host_no_port = host.split(':').next().unwrap_or(host);
    // Reason for fallback: if host has no trailing dot, use host_no_port as-is.
    let host_clean = host_no_port.strip_suffix('.').unwrap_or(host_no_port);
    host_clean.eq_ignore_ascii_case("web.archive.org")
}

fn parse_archive_target(target: &str) -> Result<ArchiveTarget> {
    let trimmed = target.trim();
    if trimmed.is_empty() {
        bail!("An Internet Archive identifier is required");
    }

    let url_candidate =
        if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            Some(trimmed.to_string())
        } else if let Some((host_candidate, _)) = trimmed.split_once('/') {
            if is_archive_org_host(host_candidate) {
                Some(format!("https://{trimmed}"))
            } else {
                None
            }
        } else {
            None
        };

    if is_archive_org_host(trimmed) {
        bail!("An Internet Archive identifier is required");
    }

    if let Some(url_str) = url_candidate {
        let url = reqwest::Url::parse(&url_str)
            .map_err(|e| anyhow!("Failed to parse URL '{url_str}': {e}"))?;
        let Some(host) = url.host_str() else {
            bail!("URL has no host: {url_str}");
        };
        if is_wayback_host(host) {
            bail!(
                "Wayback Machine URLs (web.archive.org) are not supported as item targets"
            );
        }
        if !is_archive_org_host(host) {
            bail!("'{host}' is not an Internet Archive host");
        }

        let segments: Vec<&str> = url
            .path_segments()
            // Reason for fallback: URL without a path produces empty segments; empty list safely avoids panic.
            .map(|segs| segs.filter(|segment| !segment.is_empty()).collect())
            .unwrap_or_default();

        let keyword_pos = segments.iter().position(|segment| {
            matches!(
                *segment,
                "details"
                    | "download"
                    | "metadata"
                    | "stream"
                    | "embed"
                    | "items"
            )
        });

        let (ident_segment, path_slice) = if let Some(idx) = keyword_pos {
            let Some(ident_raw) = segments.get(idx.saturating_add(1)) else {
                bail!("An Internet Archive identifier is required");
            };
            let path_slice = match segments.get(idx.saturating_add(2)..) {
                Some(slice) => slice,
                None => &[],
            };
            (*ident_raw, path_slice)
        } else {
            let Some(first) = segments.first() else {
                bail!("An Internet Archive identifier is required");
            };
            let path_slice = match segments.get(1..) {
                Some(slice) => slice,
                None => &[],
            };
            (*first, path_slice)
        };

        let ident = percent_encoding::percent_decode_str(ident_segment)
            .decode_utf8()
            .context("Invalid UTF-8 in identifier")?;
        if !is_probable_identifier(&ident) {
            bail!("{ident} is not a plausible Internet Archive identifier");
        }

        let archive_path = if path_slice.is_empty() {
            None
        } else {
            let mut parts = Vec::new();
            for part in path_slice {
                let decoded = percent_encoding::percent_decode_str(part)
                    .decode_utf8()
                    .context("Invalid UTF-8 in archive path")?;
                parts.push(decoded.into_owned());
            }
            let path_str = parts.join("/");
            if path_str.is_empty() {
                None
            } else {
                Some(path_str)
            }
        };

        return Ok(ArchiveTarget {
            identifier: ident.into_owned(),
            archive_path,
        });
    }

    parse_archive_target_path(trimmed)
}

fn parse_archive_target_path(target: &str) -> Result<ArchiveTarget> {
    let trimmed = target.trim_matches('/');
    if trimmed.is_empty() {
        bail!("An Internet Archive identifier is required");
    }
    let mut parts = trimmed.splitn(2, '/');
    // Reason for fallback: non-empty check on line 1069 guarantees splitn iterator yields at least one item; empty string fallback safely avoids panic.
    let identifier = parts.next().unwrap_or_default();
    if !is_probable_identifier(identifier) {
        bail!("{identifier} is not a plausible Internet Archive identifier");
    }
    let archive_path = parts
        .next()
        .filter(|path| !path.is_empty())
        .map(std::string::ToString::to_string);
    Ok(ArchiveTarget {
        identifier: identifier.to_string(),
        archive_path,
    })
}

fn identifier_from_metadata_name(name: &str) -> Option<String> {
    for suffix in KNOWN_IDENTIFIER_SUFFIXES {
        if let Some(identifier) = name.strip_suffix(suffix)
            && is_probable_identifier(identifier)
        {
            return Some(identifier.to_string());
        }
    }
    None
}

fn is_probable_identifier(candidate: &str) -> bool {
    !candidate.is_empty()
        && candidate.chars().all(|character| {
            character.is_ascii_alphanumeric()
                || matches!(character, '-' | '_' | '.')
        })
}

const FILE_HASH_BUFFER_SIZE: usize = 65536;

fn sha1_and_md5_hex_for_file(
    path: &Path,
    file_progress: Option<&FileItemProgress>,
) -> Result<(String, String)> {
    let mut file = File::open(path)
        .with_context(|| format!("Failed to open {}", path.display()))?;
    let mut sha1_hasher = Sha1::new();
    let mut md5_hasher = Md5::new();
    let mut buffer = [0_u8; FILE_HASH_BUFFER_SIZE];
    let mut total_read = 0_u64;
    loop {
        let bytes_read = file
            .read(&mut buffer)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        if bytes_read == 0 {
            break;
        }
        let slice = buffer
            .get(..bytes_read)
            .context("Invalid bytes read length")?;
        sha1_hasher.update(slice);
        md5_hasher.update(slice);
        if let Ok(n_u64) = u64::try_from(bytes_read) {
            total_read = total_read.saturating_add(n_u64);
            if let Some(fp) = file_progress {
                fp.set_bytes_done(total_read);
            }
        }
    }
    Ok((
        format!("{:x}", sha1_hasher.finalize()),
        format!("{:x}", md5_hasher.finalize()),
    ))
}

fn sha1_hex_for_file(path: &Path) -> Result<String> {
    sha1_hex_for_file_with_progress(path, None)
}

fn sha1_hex_for_file_with_progress(
    path: &Path,
    file_progress: Option<&FileItemProgress>,
) -> Result<String> {
    let mut file = File::open(path)
        .with_context(|| format!("Failed to open {}", path.display()))?;
    let mut hasher = Sha1::new();
    let mut buffer = [0_u8; FILE_HASH_BUFFER_SIZE];
    let mut total_read = 0_u64;
    loop {
        let bytes_read = file
            .read(&mut buffer)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(
            buffer
                .get(..bytes_read)
                .context("Invalid bytes read length")?,
        );
        if let Ok(n_u64) = u64::try_from(bytes_read) {
            total_read = total_read.saturating_add(n_u64);
            if let Some(fp) = file_progress {
                fp.set_bytes_done(total_read);
            }
        }
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn md5_hex_for_file(path: &Path) -> Result<String> {
    md5_hex_for_file_with_progress(path, None)
}

fn md5_hex_for_file_with_progress(
    path: &Path,
    file_progress: Option<&FileItemProgress>,
) -> Result<String> {
    let mut file = File::open(path)
        .with_context(|| format!("Failed to open {}", path.display()))?;
    let mut hasher = Md5::new();
    let mut buffer = [0_u8; FILE_HASH_BUFFER_SIZE];
    let mut total_read = 0_u64;
    loop {
        let bytes_read = file
            .read(&mut buffer)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(
            buffer
                .get(..bytes_read)
                .context("Invalid bytes read length")?,
        );
        if let Ok(n_u64) = u64::try_from(bytes_read) {
            total_read = total_read.saturating_add(n_u64);
            if let Some(fp) = file_progress {
                fp.set_bytes_done(total_read);
            }
        }
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn sha1_hex_for_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn md5_hex_for_bytes(bytes: &[u8]) -> String {
    let mut hasher = Md5::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn pretty_json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let compact =
        serde_json::to_string(value).context("Failed to serialize JSON")?;
    let formatted = json::jq_formatted(".", &compact)
        .context("Failed to format Internet Archive JSON output")?;
    Ok(formatted.into_bytes())
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

    const TEST_IDENTIFIER: &str = "1234-some-test-item-identifier";
    const AUDIO_FILE_NAME: &str = "01 Track 01.m4a";
    const IMAGE_FILE_NAME: &str = "01 Track 01.png";
    const META_XML: &str = "<metadata><title>Some Test Item</title></metadata>";

    #[derive(Default)]
    struct FixtureArchiveClient {
        metadata: Option<MetadataResponse>,
        files_xml_bytes: Option<Vec<u8>>,
        meta_xml_bytes: Option<Vec<u8>>,
        download_bytes: BTreeMap<String, Vec<u8>>,
    }

    impl ArchiveClient for FixtureArchiveClient {
        fn fetch_metadata(
            &self,
            _identifier: &str,
        ) -> Result<MetadataResponse> {
            self.metadata
                .clone()
                .ok_or_else(|| anyhow!("missing fixture metadata"))
        }

        fn fetch_files_xml_bytes(&self, _identifier: &str) -> Result<Vec<u8>> {
            self.files_xml_bytes
                .clone()
                .ok_or_else(|| anyhow!("missing fixture files xml"))
        }

        fn fetch_meta_xml_bytes(&self, _identifier: &str) -> Result<Vec<u8>> {
            self.meta_xml_bytes
                .clone()
                .ok_or_else(|| anyhow!("missing fixture meta xml"))
        }

        fn fetch_download_bytes(
            &self,
            _identifier: &str,
            file_name: &str,
        ) -> Result<Vec<u8>> {
            self.download_bytes.get(file_name).cloned().ok_or_else(|| {
                anyhow!("missing fixture download bytes for {file_name}")
            })
        }
    }

    fn fixture_metadata() -> MetadataResponse {
        MetadataResponse {
            files: vec![
                MetadataFile {
                    name: AUDIO_FILE_NAME.to_string(),
                    source: Some("original".to_string()),
                    md5: Some("5d41402abc4b2a76b9719d911017c592".to_string()),
                    sha1: Some(
                        "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d".to_string(),
                    ),
                    size: Some("5".to_string()),
                },
                MetadataFile {
                    name: IMAGE_FILE_NAME.to_string(),
                    source: Some("derivative".to_string()),
                    md5: Some("7d793037a0760186574b0282f2f435e7".to_string()),
                    sha1: Some(
                        "7c211433f02071597741e6ff5a8ea34789abbf43".to_string(),
                    ),
                    size: Some("5".to_string()),
                },
            ],
            extra: BTreeMap::from([(
                "metadata".to_string(),
                serde_json::json!({"identifier": TEST_IDENTIFIER}),
            )]),
        }
    }

    fn fixture_client() -> FixtureArchiveClient {
        FixtureArchiveClient {
            metadata: Some(fixture_metadata()),
            files_xml_bytes: Some(fixture_xml().into_bytes()),
            meta_xml_bytes: Some(META_XML.as_bytes().to_vec()),
            download_bytes: BTreeMap::from([
                (AUDIO_FILE_NAME.to_string(), b"hello".to_vec()),
                (IMAGE_FILE_NAME.to_string(), b"world".to_vec()),
            ]),
        }
    }

    fn fixture_xml() -> String {
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<files>\
<file name=\"{AUDIO_FILE_NAME}\" source=\"original\">\
<size>5</size>\
<md5>5d41402abc4b2a76b9719d911017c592</md5>\
<sha1>aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d</sha1>\
</file>\
<file name=\"{IMAGE_FILE_NAME}\" source=\"derivative\">\
<size>5</size>\
<md5>7d793037a0760186574b0282f2f435e7</md5>\
<sha1>7c211433f02071597741e6ff5a8ea34789abbf43</sha1>\
</file>\
<file name=\"{TEST_IDENTIFIER}_files.xml\" source=\"original\">\
<md5>placeholder</md5>\
</file>\
</files>"
        )
    }

    fn write_basic_item(temp_dir: &tempfile::TempDir) -> PathBuf {
        let item_path = temp_dir.path().join(TEST_IDENTIFIER);
        fs::create_dir_all(&item_path).unwrap();
        fs::write(item_path.join(AUDIO_FILE_NAME), b"hello").unwrap();
        fs::write(item_path.join(IMAGE_FILE_NAME), b"world").unwrap();
        fs::write(
            item_path.join(format!("{TEST_IDENTIFIER}{FILES_XML_SUFFIX}")),
            fixture_xml(),
        )
        .unwrap();
        item_path
    }

    #[crate::ctb_test]
    fn test_verify() {
        let temp_dir = tempfile::tempdir().unwrap();
        let item_path = write_basic_item(&temp_dir);
        let result = verify(&item_path, None, false, false).unwrap();
        let result_str = String::from_utf8(result).unwrap();
        assert!(result_str.starts_with("{\n"));
        let result_json: serde_json::Value =
            serde_json::from_str(&result_str).unwrap();
        assert_eq!(result_json["valid"], true);
        assert_eq!(result_json["identifier"], TEST_IDENTIFIER);
        assert_eq!(result_json["missing_files"], serde_json::json!([]));
        assert_eq!(result_json["unexpected_files"], serde_json::json!([]));
    }

    #[crate::ctb_test]
    fn test_verify_original() {
        let temp_dir = tempfile::tempdir().unwrap();
        let item_path = temp_dir.path().join(TEST_IDENTIFIER);
        fs::create_dir_all(&item_path).unwrap();
        fs::write(item_path.join(AUDIO_FILE_NAME), b"hello").unwrap();
        fs::write(
            item_path.join(format!("{TEST_IDENTIFIER}{FILES_XML_SUFFIX}")),
            fixture_xml(),
        )
        .unwrap();

        let result = verify(&item_path, None, false, true).unwrap();
        let result_json: serde_json::Value =
            serde_json::from_slice(&result).unwrap();
        assert_eq!(result_json["valid"], true);
        assert_eq!(result_json["missing_files"], serde_json::json!([]));
    }

    #[crate::ctb_test]
    fn test_verify_reports_mismatch() {
        let temp_dir = tempfile::tempdir().unwrap();
        let item_path = write_basic_item(&temp_dir);
        fs::write(item_path.join(AUDIO_FILE_NAME), b"HELLO").unwrap();

        let result = verify(&item_path, None, false, false).unwrap();
        let result_json: serde_json::Value =
            serde_json::from_slice(&result).unwrap();

        assert_eq!(result_json["valid"], false);
        assert_eq!(
            result_json["mismatched_files"].as_array().unwrap().len(),
            1
        );
        assert_eq!(
            result_json["mismatched_files"][0]["name"],
            serde_json::json!(AUDIO_FILE_NAME)
        );
    }

    #[crate::ctb_test]
    fn test_guess_identifier_from_metadata_path() {
        let path = Path::new("/tmp/1234-some-test-item-identifier_meta.sqlite");
        assert_eq!(
            guess_identifer_from_path(path),
            Some(TEST_IDENTIFIER.to_string())
        );
    }

    #[crate::ctb_test]
    fn test_guess_identifier_from_item_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let item_path = write_basic_item(&temp_dir);

        assert_eq!(
            guess_identifier_from_path(&item_path),
            Some(TEST_IDENTIFIER.to_string())
        );
    }

    #[crate::ctb_test]
    fn test_iasha1_hashes_local_files_xml_when_manifest_has_no_sha1() {
        let temp_dir = tempfile::tempdir().unwrap();
        let item_path = write_basic_item(&temp_dir);
        let files_xml_path =
            item_path.join(format!("{TEST_IDENTIFIER}{FILES_XML_SUFFIX}"));

        let expected_sha1 = sha1_hex_for_file(&files_xml_path).unwrap();
        let actual_sha1 = String::from_utf8(
            iasha1(files_xml_path.to_string_lossy().as_ref(), None, false)
                .unwrap(),
        )
        .unwrap();

        assert_eq!(actual_sha1.trim(), expected_sha1);
    }

    #[crate::ctb_test]
    fn test_iamd5_hashes_local_files_xml_when_manifest_has_no_md5() {
        let temp_dir = tempfile::tempdir().unwrap();
        let item_path = write_basic_item(&temp_dir);
        let files_xml_path =
            item_path.join(format!("{TEST_IDENTIFIER}{FILES_XML_SUFFIX}"));

        let expected_md5 = md5_hex_for_file(&files_xml_path).unwrap();
        let actual_md5 = String::from_utf8(
            iamd5(files_xml_path.to_string_lossy().as_ref(), None, false)
                .unwrap(),
        )
        .unwrap();

        assert_eq!(actual_md5.trim(), expected_md5);
    }

    #[crate::ctb_test]
    fn test_verify_ignores_local_files_xml_checksum_mismatch() {
        let temp_dir = tempfile::tempdir().unwrap();
        let item_path = write_basic_item(&temp_dir);

        let result = verify(&item_path, None, false, false).unwrap();
        let result_json: serde_json::Value =
            serde_json::from_slice(&result).unwrap();

        assert_eq!(result_json["valid"], true);
        assert_eq!(result_json["mismatched_files"], serde_json::json!([]));
    }

    #[crate::ctb_test]
    fn test_verify_live_detects_files_xml_checksum_mismatch() {
        let temp_dir = tempfile::tempdir().unwrap();
        let item_path = write_basic_item(&temp_dir);
        let local_manifest =
            load_local_files_manifest(&item_path, TEST_IDENTIFIER).unwrap();
        let live_files_xml_bytes = b"<files></files>".to_vec();
        let verification = verify_against_manifest(
            &item_path,
            TEST_IDENTIFIER,
            &local_manifest,
            None,
            true,
            &VerificationOptions {
                files_xml_live_sha1: Some(sha1_hex_for_bytes(
                    &live_files_xml_bytes,
                )),
            },
            None,
        )
        .unwrap();

        assert!(!verification.valid);
        assert_eq!(verification.mismatched_files.len(), 1);
        assert_eq!(
            verification.mismatched_files[0].name,
            format!("{TEST_IDENTIFIER}{FILES_XML_SUFFIX}")
        );
    }

    #[crate::ctb_test]
    fn test_metadata_with_client_is_pretty_printed() {
        let result =
            metadata_with_client(&fixture_client(), TEST_IDENTIFIER).unwrap();
        let result_str = String::from_utf8(result).unwrap();

        assert!(result_str.starts_with("{\n"));
        assert!(result_str.contains("\n  \"files\":"));
    }

    #[crate::ctb_test]
    fn test_listplain_with_client() {
        let result = String::from_utf8(
            listplain_with_client(&fixture_client(), TEST_IDENTIFIER).unwrap(),
        )
        .unwrap();

        assert_eq!(result, format!("{AUDIO_FILE_NAME}\n{IMAGE_FILE_NAME}\n"));
    }

    #[crate::ctb_test]
    fn test_contains_with_client() {
        let result = String::from_utf8(
            contains_with_client(
                &fixture_client(),
                TEST_IDENTIFIER,
                IMAGE_FILE_NAME,
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(result, "true\n");
    }

    #[crate::ctb_test]
    fn test_filesxml_with_client() {
        let result = String::from_utf8(
            filesxml_with_client(&fixture_client(), TEST_IDENTIFIER).unwrap(),
        )
        .unwrap();

        assert_eq!(result, fixture_xml());
    }

    #[crate::ctb_test]
    fn test_metaxml_with_client() {
        let result = String::from_utf8(
            metaxml_with_client(&fixture_client(), TEST_IDENTIFIER).unwrap(),
        )
        .unwrap();

        assert_eq!(result, META_XML);
    }

    #[crate::ctb_test]
    fn test_download_as_stream_with_client() {
        let result = String::from_utf8(
            download_as_stream_with_client(
                &fixture_client(),
                &format!("{TEST_IDENTIFIER}/{AUDIO_FILE_NAME}"),
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(result, "hello");
    }

    #[crate::ctb_test]
    fn test_download_here_with_client() {
        let temp_dir = tempfile::tempdir().unwrap();
        let result = download_here_with_client(
            &fixture_client(),
            &format!("{TEST_IDENTIFIER}/{AUDIO_FILE_NAME}"),
            Some(temp_dir.path()),
            Progress::new(false),
        )
        .unwrap();
        let result_json: serde_json::Value =
            serde_json::from_slice(&result).unwrap();
        let written =
            fs::read_to_string(temp_dir.path().join(AUDIO_FILE_NAME)).unwrap();

        assert_eq!(written, "hello");
        assert_eq!(result_json["identifier"], TEST_IDENTIFIER);
    }

    #[crate::ctb_test]
    fn test_download_with_client() {
        let temp_dir = tempfile::tempdir().unwrap();
        let result = download_with_client(
            &fixture_client(),
            TEST_IDENTIFIER,
            Some(temp_dir.path()),
            false,
            Progress::new(false),
        )
        .unwrap();
        let result_json: serde_json::Value =
            serde_json::from_slice(&result).unwrap();

        assert_eq!(
            fs::read_to_string(
                temp_dir.path().join(TEST_IDENTIFIER).join(AUDIO_FILE_NAME)
            )
            .unwrap(),
            "hello"
        );
        assert_eq!(
            result_json["downloaded_files"].as_array().unwrap().len(),
            2
        );
    }

    #[crate::ctb_test]
    fn test_download_with_client_original() {
        let temp_dir = tempfile::tempdir().unwrap();
        let result = download_with_client(
            &fixture_client(),
            TEST_IDENTIFIER,
            Some(temp_dir.path()),
            true,
            Progress::new(false),
        )
        .unwrap();
        let result_json: serde_json::Value =
            serde_json::from_slice(&result).unwrap();

        assert_eq!(
            fs::read_to_string(
                temp_dir.path().join(TEST_IDENTIFIER).join(AUDIO_FILE_NAME)
            )
            .unwrap(),
            "hello"
        );
        let downloaded = result_json["downloaded_files"].as_array().unwrap();
        assert_eq!(downloaded.len(), 1);
        assert_eq!(downloaded[0], AUDIO_FILE_NAME);
    }

    #[crate::ctb_test]
    fn test_checkeddl_with_client() {
        let temp_dir = tempfile::tempdir().unwrap();
        let result = checkeddl_with_client(
            &fixture_client(),
            &format!("{TEST_IDENTIFIER}/{AUDIO_FILE_NAME}"),
            Some(temp_dir.path()),
            false,
            Progress::new(false),
        )
        .unwrap();
        let result_json: serde_json::Value =
            serde_json::from_slice(&result).unwrap();

        assert_eq!(result_json["valid"], true);
        assert_eq!(
            result_json["checked_files"],
            serde_json::json!([AUDIO_FILE_NAME])
        );
    }

    #[crate::ctb_test]
    fn test_checkeddl_with_client_data_node_url() {
        let temp_dir = tempfile::tempdir().unwrap();
        let data_node_url = format!(
            "https://dn711000.ca.archive.org/0/items/{TEST_IDENTIFIER}/{AUDIO_FILE_NAME}"
        );
        let result = checkeddl_with_client(
            &fixture_client(),
            &data_node_url,
            Some(temp_dir.path()),
            false,
            Progress::new(false),
        )
        .unwrap();
        let result_json: serde_json::Value =
            serde_json::from_slice(&result).unwrap();

        assert_eq!(result_json["valid"], true);
        assert_eq!(
            result_json["checked_files"],
            serde_json::json!([AUDIO_FILE_NAME])
        );
    }

    #[crate::ctb_test]
    fn test_checkeddl_with_client_original() {
        let temp_dir = tempfile::tempdir().unwrap();
        let result = checkeddl_with_client(
            &fixture_client(),
            TEST_IDENTIFIER,
            Some(temp_dir.path()),
            true,
            Progress::new(false),
        )
        .unwrap();
        let result_json: serde_json::Value =
            serde_json::from_slice(&result).unwrap();

        assert_eq!(result_json["valid"], true);
        let checked = result_json["checked_files"].as_array().unwrap();
        assert_eq!(checked.len(), 2);
        assert!(checked.contains(&serde_json::json!(AUDIO_FILE_NAME)));
        assert!(checked.contains(&serde_json::json!(format!(
            "{TEST_IDENTIFIER}{FILES_XML_SUFFIX}"
        ))));
    }

    #[crate::ctb_test]
    fn test_parse_archive_target_urls() {
        let details_url =
            "https://archive.org/details/populationschedu1331unix";
        let target = parse_archive_target(details_url).unwrap();
        assert_eq!(target.identifier, "populationschedu1331unix");
        assert_eq!(target.archive_path, None);

        let details_slash =
            "https://archive.org/details/populationschedu1331unix/";
        let target = parse_archive_target(details_slash).unwrap();
        assert_eq!(target.identifier, "populationschedu1331unix");
        assert_eq!(target.archive_path, None);

        let details_with_file =
            "https://archive.org/details/populationschedu1331unix/sub/file.pdf";
        let target = parse_archive_target(details_with_file).unwrap();
        assert_eq!(target.identifier, "populationschedu1331unix");
        assert_eq!(target.archive_path, Some("sub/file.pdf".to_string()));

        let download_url = "https://archive.org/download/populationschedu1331unix/01%20Track%2001.m4a";
        let target = parse_archive_target(download_url).unwrap();
        assert_eq!(target.identifier, "populationschedu1331unix");
        assert_eq!(target.archive_path, Some("01 Track 01.m4a".to_string()));

        let metadata_url =
            "https://archive.org/metadata/populationschedu1331unix";
        let target = parse_archive_target(metadata_url).unwrap();
        assert_eq!(target.identifier, "populationschedu1331unix");
        assert_eq!(target.archive_path, None);

        let stream_url = "https://archive.org/stream/populationschedu1331unix";
        let target = parse_archive_target(stream_url).unwrap();
        assert_eq!(target.identifier, "populationschedu1331unix");
        assert_eq!(target.archive_path, None);

        let embed_url = "https://archive.org/embed/populationschedu1331unix";
        let target = parse_archive_target(embed_url).unwrap();
        assert_eq!(target.identifier, "populationschedu1331unix");
        assert_eq!(target.archive_path, None);

        let items_url = "https://archive.org/items/populationschedu1331unix";
        let target = parse_archive_target(items_url).unwrap();
        assert_eq!(target.identifier, "populationschedu1331unix");
        assert_eq!(target.archive_path, None);

        let www_http_query = "http://www.archive.org/details/populationschedu1331unix?query=1#frag";
        let target = parse_archive_target(www_http_query).unwrap();
        assert_eq!(target.identifier, "populationschedu1331unix");
        assert_eq!(target.archive_path, None);

        let no_scheme = "archive.org/details/populationschedu1331unix";
        let target = parse_archive_target(no_scheme).unwrap();
        assert_eq!(target.identifier, "populationschedu1331unix");
        assert_eq!(target.archive_path, None);

        let plain_ident = "populationschedu1331unix";
        let target = parse_archive_target(plain_ident).unwrap();
        assert_eq!(target.identifier, "populationschedu1331unix");
        assert_eq!(target.archive_path, None);

        let plain_with_path = "populationschedu1331unix/01 Track 01.m4a";
        let target = parse_archive_target(plain_with_path).unwrap();
        assert_eq!(target.identifier, "populationschedu1331unix");
        assert_eq!(target.archive_path, Some("01 Track 01.m4a".to_string()));

        let data_node_url = "https://dn711000.ca.archive.org/0/items/prideandprejudice_1005_librivox/prideandprejudice_01_austen.ogg";
        let target = parse_archive_target(data_node_url).unwrap();
        assert_eq!(target.identifier, "prideandprejudice_1005_librivox");
        assert_eq!(
            target.archive_path,
            Some("prideandprejudice_01_austen.ogg".to_string())
        );

        let data_node_no_scheme = "dn711000.ca.archive.org/0/items/prideandprejudice_1005_librivox/prideandprejudice_01_austen.ogg";
        let target = parse_archive_target(data_node_no_scheme).unwrap();
        assert_eq!(target.identifier, "prideandprejudice_1005_librivox");
        assert_eq!(
            target.archive_path,
            Some("prideandprejudice_01_austen.ogg".to_string())
        );

        let us_node_url = "https://ia800100.us.archive.org/3/items/prideandprejudice_1005_librivox/prideandprejudice_01_austen.ogg";
        let target = parse_archive_target(us_node_url).unwrap();
        assert_eq!(target.identifier, "prideandprejudice_1005_librivox");
        assert_eq!(
            target.archive_path,
            Some("prideandprejudice_01_austen.ogg".to_string())
        );

        let node_no_disk_url = "https://ia600201.us.archive.org/items/prideandprejudice_1005_librivox/prideandprejudice_01_austen.ogg";
        let target = parse_archive_target(node_no_disk_url).unwrap();
        assert_eq!(target.identifier, "prideandprejudice_1005_librivox");
        assert_eq!(
            target.archive_path,
            Some("prideandprejudice_01_austen.ogg".to_string())
        );

        let node_item_only = "https://dn711000.ca.archive.org/0/items/prideandprejudice_1005_librivox";
        let target = parse_archive_target(node_item_only).unwrap();
        assert_eq!(target.identifier, "prideandprejudice_1005_librivox");
        assert_eq!(target.archive_path, None);

        parse_archive_target("").unwrap_err();
        parse_archive_target("https://archive.org/details/").unwrap_err();
        parse_archive_target("https://archive.org/").unwrap_err();
        parse_archive_target("https://dn711000.ca.archive.org/0/items/")
            .unwrap_err();
        parse_archive_target("https://dn711000.ca.archive.org/").unwrap_err();
        parse_archive_target(
            "https://web.archive.org/web/20261018/https://example.com",
        )
        .unwrap_err();
        parse_archive_target("https://example.com/items/test/file.ogg")
            .unwrap_err();
    }

    #[crate::ctb_test]
    fn test_parse_content_range() {
        assert_eq!(
            parse_content_range("bytes 0-8388607/52428800"),
            Some((0, 8388607, Some(52428800)))
        );
        assert_eq!(
            parse_content_range("bytes 100-200/*"),
            Some((100, 200, None))
        );
        assert_eq!(parse_content_range("bytes 0-0/1"), Some((0, 0, Some(1))));
        assert_eq!(parse_content_range("invalid"), None);
        assert_eq!(parse_content_range("items 0-10/20"), None);
    }

    #[crate::ctb_test]
    fn test_create_temp_path_and_atomic_commit() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dest = temp_dir.path().join("final_file.bin");
        let temp_path = create_temp_path(temp_dir.path(), "final_file.bin");
        assert!(temp_path.starts_with(temp_dir.path()));
        assert_ne!(temp_path, dest);

        let mut guard = TempFileCleanupGuard {
            temp_path: temp_path.clone(),
            active: true,
        };
        fs::write(&temp_path, b"atomic test payload").unwrap();
        assert!(temp_path.exists());

        commit_atomic_file(&temp_path, &dest).unwrap();
        guard.active = false;

        assert!(!temp_path.exists());
        assert!(dest.exists());
        assert_eq!(fs::read(&dest).unwrap(), b"atomic test payload");
    }

    #[crate::ctb_test]
    fn test_temp_file_cleanup_guard_removes_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().join(".test_cleanup_temp");
        fs::write(&temp_path, b"uncommitted partial data").unwrap();
        assert!(temp_path.exists());

        {
            let _guard = TempFileCleanupGuard {
                temp_path: temp_path.clone(),
                active: true,
            };
        }

        assert!(!temp_path.exists());
    }

    struct MockRangeResponse {
        status: reqwest::StatusCode,
        headers: reqwest::header::HeaderMap,
        content_length: Option<u64>,
        body: std::io::Cursor<Vec<u8>>,
        fail_after_bytes: Option<usize>,
        read_so_far: usize,
    }

    impl Read for MockRangeResponse {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            if let Some(limit) = self.fail_after_bytes
                && self.read_so_far >= limit
            {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "simulated stalled range stream",
                ));
            }
            let max_len = if let Some(limit) = self.fail_after_bytes {
                buf.len().min(limit - self.read_so_far)
            } else {
                buf.len()
            };
            let n = self.body.read(&mut buf[..max_len])?;
            self.read_so_far += n;
            Ok(n)
        }
    }

    impl RangeHttpResponse for MockRangeResponse {
        fn status(&self) -> reqwest::StatusCode {
            self.status
        }

        fn headers(&self) -> &reqwest::header::HeaderMap {
            &self.headers
        }

        fn content_length(&self) -> Option<u64> {
            self.content_length
        }
    }

    struct MockRangeClient<F> {
        handler: F,
    }

    impl<F> MockRangeClient<F>
    where
        F: Fn(
            &str,
            Option<&reqwest::header::HeaderMap>,
        ) -> Result<MockRangeResponse>,
    {
        fn new(handler: F) -> Self {
            Self { handler }
        }
    }

    impl<F> RangeHttpClient for MockRangeClient<F>
    where
        F: Fn(
            &str,
            Option<&reqwest::header::HeaderMap>,
        ) -> Result<MockRangeResponse>,
    {
        fn get_with_headers(
            &self,
            url: &str,
            headers: Option<reqwest::header::HeaderMap>,
        ) -> Result<Box<dyn RangeHttpResponse>> {
            Ok(Box::new((self.handler)(url, headers.as_ref())?))
        }
    }

    #[crate::ctb_test]
    fn test_chunked_range_download_many_chunks_and_retries() {
        let payload: Vec<u8> = (0..2500_u32)
            .map(|i| u8::try_from(i % 251).unwrap())
            .collect();
        let total_len = u64::try_from(payload.len()).unwrap();
        let attempts = std::sync::atomic::AtomicUsize::new(0);

        let client = MockRangeClient::new(|_url, headers| {
            let call_idx =
                attempts.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let range_str = headers
                .unwrap()
                .get(reqwest::header::RANGE)
                .unwrap()
                .to_str()
                .unwrap();
            let (start_s, end_s) = range_str
                .strip_prefix("bytes=")
                .unwrap()
                .split_once('-')
                .unwrap();
            let start: usize = start_s.parse().unwrap();
            let end: usize =
                end_s.parse::<usize>().unwrap().min(payload.len() - 1);

            // Inject transient connection failure on chunk 0 and mid-read
            // timeout on a later chunk to test retry recovery.
            if call_idx == 0 {
                bail!("simulated transient connection reset on chunk 0");
            }

            let mut resp_headers = reqwest::header::HeaderMap::new();
            resp_headers.insert(
                reqwest::header::CONTENT_RANGE,
                reqwest::header::HeaderValue::from_str(&format!(
                    "bytes {start}-{end}/{total_len}"
                ))
                .unwrap(),
            );
            let slice = payload[start..=end].to_vec();
            let slice_len = u64::try_from(slice.len()).unwrap();
            Ok(MockRangeResponse {
                status: reqwest::StatusCode::PARTIAL_CONTENT,
                headers: resp_headers,
                content_length: Some(slice_len),
                body: std::io::Cursor::new(slice),
                fail_after_bytes: if call_idx == 70 { Some(4) } else { None },
                read_so_far: 0,
            })
        });

        let progress = Progress::new(false);
        let batch = progress.start_file_batch("Overall", Some(1), None);
        let fp = batch.start_file("big.bin", None);
        let mut out = Vec::new();
        let written = download_to_writer_chunked_range_with_http(
            &client,
            "https://archive.org/download/test/big.bin",
            &mut out,
            Some(&fp),
            16,
            0,
        )
        .unwrap();
        fp.finish(written);
        batch.finish();

        assert_eq!(written, total_len);
        assert_eq!(out, payload);
        // 2500 / 16 = 157 chunks + 2 retried attempts = 159 requests
        assert_eq!(attempts.load(std::sync::atomic::Ordering::SeqCst), 159);
    }

    #[crate::ctb_test]
    fn test_chunked_range_download_rejects_200_ok_on_nonzero_offset() {
        let client = MockRangeClient::new(|_url, headers| {
            let range_str = headers
                .unwrap()
                .get(reqwest::header::RANGE)
                .unwrap()
                .to_str()
                .unwrap();
            if range_str.starts_with("bytes=0-") {
                let mut resp_headers = reqwest::header::HeaderMap::new();
                resp_headers.insert(
                    reqwest::header::CONTENT_RANGE,
                    reqwest::header::HeaderValue::from_static("bytes 0-3/8"),
                );
                Ok(MockRangeResponse {
                    status: reqwest::StatusCode::PARTIAL_CONTENT,
                    headers: resp_headers,
                    content_length: Some(4),
                    body: std::io::Cursor::new(b"abcd".to_vec()),
                    fail_after_bytes: None,
                    read_so_far: 0,
                })
            } else {
                // Misbehaving server ignores Range on second chunk and returns 200 OK
                Ok(MockRangeResponse {
                    status: reqwest::StatusCode::OK,
                    headers: reqwest::header::HeaderMap::new(),
                    content_length: Some(8),
                    body: std::io::Cursor::new(b"abcdefgh".to_vec()),
                    fail_after_bytes: None,
                    read_so_far: 0,
                })
            }
        });

        let mut out = Vec::new();
        let err = download_to_writer_chunked_range_with_http(
            &client,
            "https://archive.org/download/test/broken.bin",
            &mut out,
            None,
            4,
            0,
        )
        .unwrap_err();
        assert!(format!("{err:#}").contains("Unexpected HTTP status 200"));
    }

    #[crate::ctb_test]
    fn test_chunked_range_download_empty_file_416_and_truncated_error() {
        let empty_client = MockRangeClient::new(|_url, _headers| {
            let mut resp_headers = reqwest::header::HeaderMap::new();
            resp_headers.insert(
                reqwest::header::CONTENT_RANGE,
                reqwest::header::HeaderValue::from_static("bytes */0"),
            );
            Ok(MockRangeResponse {
                status: reqwest::StatusCode::RANGE_NOT_SATISFIABLE,
                headers: resp_headers,
                content_length: Some(0),
                body: std::io::Cursor::new(Vec::new()),
                fail_after_bytes: None,
                read_so_far: 0,
            })
        });
        let mut empty_out = Vec::new();
        let written = download_to_writer_chunked_range_with_http(
            &empty_client,
            "https://archive.org/download/test/empty.bin",
            &mut empty_out,
            None,
            16,
            0,
        )
        .unwrap();
        assert_eq!(written, 0);
        assert!(empty_out.is_empty());

        let truncated_client = MockRangeClient::new(|_url, headers| {
            let range_str = headers
                .unwrap()
                .get(reqwest::header::RANGE)
                .unwrap()
                .to_str()
                .unwrap();
            if range_str.starts_with("bytes=0-") {
                let mut resp_headers = reqwest::header::HeaderMap::new();
                resp_headers.insert(
                    reqwest::header::CONTENT_RANGE,
                    reqwest::header::HeaderValue::from_static("bytes 0-3/8"),
                );
                Ok(MockRangeResponse {
                    status: reqwest::StatusCode::PARTIAL_CONTENT,
                    headers: resp_headers,
                    content_length: Some(4),
                    body: std::io::Cursor::new(b"abcd".to_vec()),
                    fail_after_bytes: None,
                    read_so_far: 0,
                })
            } else {
                Ok(MockRangeResponse {
                    status: reqwest::StatusCode::RANGE_NOT_SATISFIABLE,
                    headers: reqwest::header::HeaderMap::new(),
                    content_length: Some(0),
                    body: std::io::Cursor::new(Vec::new()),
                    fail_after_bytes: None,
                    read_so_far: 0,
                })
            }
        });
        let mut trunc_out = Vec::new();
        let err = download_to_writer_chunked_range_with_http(
            &truncated_client,
            "https://archive.org/download/test/trunc.bin",
            &mut trunc_out,
            None,
            4,
            0,
        )
        .unwrap_err();
        assert!(format!("{err:#}").contains("416"));
    }

    #[crate::ctb_test]
    fn test_part_file_interrupted_download_resumes_from_exact_offset() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dest = temp_dir.path().join("movie.iso");
        let part = part_path_for_destination(&dest);
        assert_eq!(part, temp_dir.path().join("movie.iso.part"));

        let payload: Vec<u8> = (0..120_u32)
            .map(|i| u8::try_from(i % 251).unwrap())
            .collect();
        let total_len = u64::try_from(payload.len()).unwrap();

        // Run 1: fail persistently once offset reaches 48 (3 chunks of 16 bytes),
        // simulating an interrupted download leaving 48 bytes in movie.iso.part.
        let run1_client = MockRangeClient::new(|_url, headers| {
            let range_str = headers
                .unwrap()
                .get(reqwest::header::RANGE)
                .unwrap()
                .to_str()
                .unwrap();
            let (start_s, end_s) = range_str
                .strip_prefix("bytes=")
                .unwrap()
                .split_once('-')
                .unwrap();
            let start: usize = start_s.parse().unwrap();
            let end: usize =
                end_s.parse::<usize>().unwrap().min(payload.len() - 1);
            if start >= 48 {
                bail!("simulated interruption at byte 48");
            }
            let mut resp_headers = reqwest::header::HeaderMap::new();
            resp_headers.insert(
                reqwest::header::CONTENT_RANGE,
                reqwest::header::HeaderValue::from_str(&format!(
                    "bytes {start}-{end}/{total_len}"
                ))
                .unwrap(),
            );
            let slice = payload[start..=end].to_vec();
            let slice_len = u64::try_from(slice.len()).unwrap();
            Ok(MockRangeResponse {
                status: reqwest::StatusCode::PARTIAL_CONTENT,
                headers: resp_headers,
                content_length: Some(slice_len),
                body: std::io::Cursor::new(slice),
                fail_after_bytes: None,
                read_so_far: 0,
            })
        });

        download_to_part_file_chunked_range_with_http(
            &run1_client,
            "https://archive.org/download/test/movie.iso",
            &dest,
            Some(total_len),
            None,
            16,
            0,
        )
        .unwrap_err();

        assert!(!dest.exists());
        assert!(part.is_file());
        assert_eq!(fs::read(&part).unwrap(), &payload[..48]);

        // Run 2: resume from movie.iso.part; assert NO request starts below byte 48!
        let requested_starts = std::sync::Mutex::new(Vec::new());
        let run2_client = MockRangeClient::new(|_url, headers| {
            let range_str = headers
                .unwrap()
                .get(reqwest::header::RANGE)
                .unwrap()
                .to_str()
                .unwrap();
            let (start_s, end_s) = range_str
                .strip_prefix("bytes=")
                .unwrap()
                .split_once('-')
                .unwrap();
            let start: usize = start_s.parse().unwrap();
            let end: usize =
                end_s.parse::<usize>().unwrap().min(payload.len() - 1);
            requested_starts.lock().unwrap().push(start);

            let mut resp_headers = reqwest::header::HeaderMap::new();
            resp_headers.insert(
                reqwest::header::CONTENT_RANGE,
                reqwest::header::HeaderValue::from_str(&format!(
                    "bytes {start}-{end}/{total_len}"
                ))
                .unwrap(),
            );
            let slice = payload[start..=end].to_vec();
            let slice_len = u64::try_from(slice.len()).unwrap();
            Ok(MockRangeResponse {
                status: reqwest::StatusCode::PARTIAL_CONTENT,
                headers: resp_headers,
                content_length: Some(slice_len),
                body: std::io::Cursor::new(slice),
                fail_after_bytes: None,
                read_so_far: 0,
            })
        });

        let final_len = download_to_part_file_chunked_range_with_http(
            &run2_client,
            "https://archive.org/download/test/movie.iso",
            &dest,
            Some(total_len),
            None,
            16,
            0,
        )
        .unwrap();

        assert_eq!(final_len, total_len);
        assert!(!part.exists());
        assert_eq!(fs::read(&dest).unwrap(), payload);
        let starts = requested_starts.lock().unwrap().clone();
        assert_eq!(starts, vec![48, 64, 80, 96, 112]);
    }

    #[crate::ctb_test]
    fn test_part_file_handles_already_complete_416_and_200_ok_fallback() {
        let temp_dir = tempfile::tempdir().unwrap();

        // Case A: .part already has full 32 bytes, expected_size is None, server returns 416 bytes */32
        let dest_a = temp_dir.path().join("complete.bin");
        let part_a = part_path_for_destination(&dest_a);
        let payload_a = b"0123456789abcdef0123456789abcdef".to_vec();
        fs::write(&part_a, &payload_a).unwrap();

        let client_416 = MockRangeClient::new(|_url, _headers| {
            let mut resp_headers = reqwest::header::HeaderMap::new();
            resp_headers.insert(
                reqwest::header::CONTENT_RANGE,
                reqwest::header::HeaderValue::from_static("bytes */32"),
            );
            Ok(MockRangeResponse {
                status: reqwest::StatusCode::RANGE_NOT_SATISFIABLE,
                headers: resp_headers,
                content_length: Some(0),
                body: std::io::Cursor::new(Vec::new()),
                fail_after_bytes: None,
                read_so_far: 0,
            })
        });
        let written_a = download_to_part_file_chunked_range_with_http(
            &client_416,
            "https://archive.org/download/test/complete.bin",
            &dest_a,
            None,
            None,
            16,
            0,
        )
        .unwrap();
        assert_eq!(written_a, 32);
        assert!(!part_a.exists());
        assert_eq!(fs::read(&dest_a).unwrap(), payload_a);

        // Case B: .part has 10 bytes, server ignores Range and responds 200 OK with full stream
        let dest_b = temp_dir.path().join("fallback200.bin");
        let part_b = part_path_for_destination(&dest_b);
        fs::write(&part_b, b"0123456789").unwrap();
        let payload_b = b"0123456789ABCDEFGHIJKLMNOPQRSTUV".to_vec();
        let payload_b_clone = payload_b.clone();

        let client_200 = MockRangeClient::new(move |_url, _headers| {
            Ok(MockRangeResponse {
                status: reqwest::StatusCode::OK,
                headers: reqwest::header::HeaderMap::new(),
                content_length: Some(32),
                body: std::io::Cursor::new(payload_b_clone.clone()),
                fail_after_bytes: None,
                read_so_far: 0,
            })
        });
        let written_b = download_to_part_file_chunked_range_with_http(
            &client_200,
            "https://archive.org/download/test/fallback200.bin",
            &dest_b,
            Some(32),
            None,
            16,
            0,
        )
        .unwrap();
        assert_eq!(written_b, 32);
        assert!(!part_b.exists());
        assert_eq!(fs::read(&dest_b).unwrap(), payload_b);
    }

    #[crate::ctb_test]
    fn test_checkeddl_resumes_interrupted_batch_skipping_verified_and_healing_corrupt_part()
     {
        struct RangeBackedArchiveClient<'a> {
            metadata: MetadataResponse,
            files_xml: Vec<u8>,
            files: BTreeMap<String, Vec<u8>>,
            range_log: &'a std::sync::Mutex<Vec<(String, usize)>>,
        }

        impl ArchiveClient for RangeBackedArchiveClient<'_> {
            fn fetch_metadata(
                &self,
                _identifier: &str,
            ) -> Result<MetadataResponse> {
                Ok(self.metadata.clone())
            }

            fn fetch_files_xml_bytes(
                &self,
                _identifier: &str,
            ) -> Result<Vec<u8>> {
                Ok(self.files_xml.clone())
            }

            fn fetch_meta_xml_bytes(
                &self,
                _identifier: &str,
            ) -> Result<Vec<u8>> {
                Ok(META_XML.as_bytes().to_vec())
            }

            fn fetch_download_bytes(
                &self,
                _identifier: &str,
                file_name: &str,
            ) -> Result<Vec<u8>> {
                self.files
                    .get(file_name)
                    .cloned()
                    .ok_or_else(|| anyhow!("missing {file_name}"))
            }

            fn download_file(
                &self,
                identifier: &str,
                file_name: &str,
                destination: &Path,
                expected_size: Option<u64>,
                file_progress: Option<&FileItemProgress>,
            ) -> Result<u64> {
                let data = self
                    .files
                    .get(file_name)
                    .cloned()
                    .ok_or_else(|| anyhow!("missing {file_name}"))?;
                let total = u64::try_from(data.len()).unwrap();
                let fname = file_name.to_string();
                let log_ref = self.range_log;
                let mock_http = MockRangeClient::new(move |_url, headers| {
                    let range_str = headers
                        .unwrap()
                        .get(reqwest::header::RANGE)
                        .unwrap()
                        .to_str()
                        .unwrap();
                    let (start_s, end_s) = range_str
                        .strip_prefix("bytes=")
                        .unwrap()
                        .split_once('-')
                        .unwrap();
                    let start: usize = start_s.parse().unwrap();
                    let end: usize =
                        end_s.parse::<usize>().unwrap().min(data.len() - 1);
                    log_ref.lock().unwrap().push((fname.clone(), start));
                    let mut resp_headers = reqwest::header::HeaderMap::new();
                    resp_headers.insert(
                        reqwest::header::CONTENT_RANGE,
                        reqwest::header::HeaderValue::from_str(&format!(
                            "bytes {start}-{end}/{total}"
                        ))
                        .unwrap(),
                    );
                    let slice = data[start..=end].to_vec();
                    let slice_len = u64::try_from(slice.len()).unwrap();
                    Ok(MockRangeResponse {
                        status: reqwest::StatusCode::PARTIAL_CONTENT,
                        headers: resp_headers,
                        content_length: Some(slice_len),
                        body: std::io::Cursor::new(slice),
                        fail_after_bytes: None,
                        read_so_far: 0,
                    })
                });
                let url = download_url(identifier, file_name)?;
                download_to_part_file_chunked_range_with_http(
                    &mock_http,
                    url.as_str(),
                    destination,
                    expected_size,
                    file_progress,
                    16,
                    0,
                )
            }
        }

        let file_a = b"file-a-already-complete-and-verified-bytes!".to_vec();
        let file_b =
            b"file-b-resumable-payload-across-several-16-byte-chunks!!"
                .to_vec();
        let file_c =
            b"file-c-clean-remote-contents-after-corrupt-part".to_vec();

        let files_map = BTreeMap::from([
            ("a.bin".to_string(), file_a.clone()),
            ("b.bin".to_string(), file_b.clone()),
            ("c.bin".to_string(), file_c.clone()),
        ]);

        let mut xml =
            String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?><files>");
        let mut meta_files = Vec::new();
        for (name, bytes) in &files_map {
            let md5 = md5_hex_for_bytes(bytes);
            let sha1 = sha1_hex_for_bytes(bytes);
            let sz = bytes.len().to_string();
            xml.push_str(&format!(
                "<file name=\"{name}\" source=\"original\"><size>{sz}</size><md5>{md5}</md5><sha1>{sha1}</sha1></file>"
            ));
            meta_files.push(MetadataFile {
                name: name.clone(),
                source: Some("original".to_string()),
                md5: Some(md5),
                sha1: Some(sha1),
                size: Some(sz),
            });
        }
        xml.push_str(&format!(
            "<file name=\"{TEST_IDENTIFIER}{FILES_XML_SUFFIX}\" source=\"original\"><md5>placeholder</md5></file></files>"
        ));

        let temp_dir = tempfile::tempdir().unwrap();
        let item_dir = temp_dir.path().join(TEST_IDENTIFIER);
        fs::create_dir_all(&item_dir).unwrap();

        // Seed interrupted state on disk:
        // 1) a.bin is complete & valid (plus a stale a.bin.part that should be cleaned up)
        fs::write(item_dir.join("a.bin"), &file_a).unwrap();
        fs::write(item_dir.join("a.bin.part"), b"stale").unwrap();
        // 2) b.bin was interrupted after 24 bytes -> b.bin.part exists
        fs::write(item_dir.join("b.bin.part"), &file_b[..24]).unwrap();
        // 3) c.bin.part has 16 corrupted bytes -> must detect mismatch after resume and re-download cleanly
        fs::write(item_dir.join("c.bin.part"), b"CORRUPTED_PREFIX").unwrap();

        let range_log = std::sync::Mutex::new(Vec::new());
        let client = RangeBackedArchiveClient {
            metadata: MetadataResponse {
                files: meta_files,
                extra: BTreeMap::new(),
            },
            files_xml: xml.into_bytes(),
            files: files_map,
            range_log: &range_log,
        };

        let out = checkeddl_with_client(
            &client,
            TEST_IDENTIFIER,
            Some(temp_dir.path()),
            false,
            Progress::new(false),
        )
        .unwrap();
        let json_res: serde_json::Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(json_res["valid"], true, "{json_res:#}");
        assert_eq!(json_res["mismatched_files"], serde_json::json!([]));
        assert_eq!(json_res["unexpected_files"], serde_json::json!([]));

        assert_eq!(fs::read(item_dir.join("a.bin")).unwrap(), file_a);
        assert_eq!(fs::read(item_dir.join("b.bin")).unwrap(), file_b);
        assert_eq!(fs::read(item_dir.join("c.bin")).unwrap(), file_c);
        assert!(!item_dir.join("a.bin.part").exists());
        assert!(!item_dir.join("b.bin.part").exists());
        assert!(!item_dir.join("c.bin.part").exists());

        let logged = range_log.lock().unwrap().clone();
        // a.bin must NOT be downloaded at all (0 HTTP requests)!
        assert!(!logged.iter().any(|(f, _)| f == "a.bin"), "{logged:?}");
        // b.bin must ONLY be requested starting at offset 24 (24, 40)!
        let b_starts: Vec<usize> = logged
            .iter()
            .filter(|(f, _)| f == "b.bin")
            .map(|(_, s)| *s)
            .collect();
        assert_eq!(b_starts, vec![24, 40]);
        // c.bin resumed at 16 first, failed SHA1/MD5 check, then re-downloaded cleanly from 0!
        let c_starts: Vec<usize> = logged
            .iter()
            .filter(|(f, _)| f == "c.bin")
            .map(|(_, s)| *s)
            .collect();
        assert_eq!(c_starts.first().copied(), Some(16));
        assert!(c_starts.contains(&0), "{c_starts:?}");
    }
}
