#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use anyhow::{Context, Result, anyhow, bail};
use md5::{Digest as _, Md5};
use serde::{Deserialize, Serialize};
use sha1::{Digest as _, Sha1};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::{Component, Path, PathBuf};
use xml::reader::{EventReader, XmlEvent};

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
pub fn verify(
    item_path: &Path,
    identifier: Option<&str>,
    check_live: bool,
) -> Result<Vec<u8>> {
    let root = normalize_item_root(item_path);
    let identifier = resolve_identifier(item_path, identifier)?;
    let verification = if check_live {
        let live_files_xml_bytes = fetch_live_files_xml_bytes(&identifier)?;
        let manifest =
            parse_files_manifest(&identifier, &live_files_xml_bytes)?;
        verify_against_manifest(
            &root,
            &identifier,
            &manifest,
            None,
            true,
            &VerificationOptions {
                files_xml_live_sha1: Some(sha1_hex_for_bytes(
                    &live_files_xml_bytes,
                )),
            },
        )?
    } else {
        let manifest = load_local_files_manifest(&root, &identifier)?;
        verify_against_manifest(
            &root,
            &identifier,
            &manifest,
            None,
            true,
            &VerificationOptions {
                files_xml_live_sha1: None,
            },
        )?
    };
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
) -> Result<Vec<u8>> {
    download_with_client(&LiveArchiveClient, target, output_dir, original)
}

pub fn download_as_stream(target: &str) -> Result<Vec<u8>> {
    download_as_stream_with_client(&LiveArchiveClient, target)
}

pub fn download_here(
    target: &str,
    output_dir: Option<&Path>,
) -> Result<Vec<u8>> {
    download_here_with_client(&LiveArchiveClient, target, output_dir)
}

fn download_with_client(
    client: &dyn ArchiveClient,
    target: &str,
    output_dir: Option<&Path>,
    original: bool,
) -> Result<Vec<u8>> {
    let archive_target = parse_archive_target(target)?;
    let base_output_dir = resolve_output_dir(output_dir)?;
    let item_directory = base_output_dir.join(&archive_target.identifier);
    fs::create_dir_all(&item_directory).with_context(|| {
        format!("Failed to create {}", item_directory.display())
    })?;

    let file_names = if original {
        let metadata_files =
            client.fetch_metadata(&archive_target.identifier)?.files;
        let original_files: Vec<String> = metadata_files
            .into_iter()
            .filter(|file| file.source.as_deref() == Some("original"))
            .map(|file| file.name)
            .collect();
        if let Some(file_name) = &archive_target.archive_path {
            original_files
                .into_iter()
                .filter(|name| name == file_name)
                .collect()
        } else {
            original_files
        }
    } else if let Some(file_name) = &archive_target.archive_path {
        vec![file_name.clone()]
    } else {
        client
            .fetch_metadata(&archive_target.identifier)?
            .files
            .into_iter()
            .map(|file| file.name)
            .collect()
    };

    let mut downloaded_files = Vec::new();
    for file_name in file_names {
        let destination = item_directory.join(Path::new(&file_name));
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!(
                    "Failed to create parent directory {}",
                    parent.display()
                )
            })?;
        }
        let bytes = client
            .fetch_download_bytes(&archive_target.identifier, &file_name)?;
        fs::write(&destination, &bytes).with_context(|| {
            format!("Failed to write downloaded file {}", destination.display())
        })?;
        downloaded_files.push(file_name);
    }

    let result = DownloadResult {
        identifier: archive_target.identifier,
        output_directory: item_directory.display().to_string(),
        downloaded_files,
    };
    pretty_json_bytes(&result)
}

pub fn checkeddl(target: &str, output_dir: Option<&Path>) -> Result<Vec<u8>> {
    checkeddl_with_client(&LiveArchiveClient, target, output_dir)
}

fn checkeddl_with_client(
    client: &dyn ArchiveClient,
    target: &str,
    output_dir: Option<&Path>,
) -> Result<Vec<u8>> {
    let archive_target = parse_archive_target(target)?;
    let base_output_dir = resolve_output_dir(output_dir)?;
    let item_directory = base_output_dir.join(&archive_target.identifier);

    let _ = download_with_client(client, target, Some(&base_output_dir), false)?;
    let live_files_xml_bytes =
        client.fetch_files_xml_bytes(&archive_target.identifier)?;
    let manifest = parse_files_manifest(
        &archive_target.identifier,
        &live_files_xml_bytes,
    )?;
    let selected_files = archive_target.archive_path.clone().map(|file_name| {
        let mut files = BTreeSet::new();
        files.insert(file_name);
        files
    });
    let verification = verify_against_manifest(
        &item_directory,
        &archive_target.identifier,
        &manifest,
        selected_files.as_ref(),
        archive_target.archive_path.is_none(),
        &VerificationOptions {
            files_xml_live_sha1: Some(sha1_hex_for_bytes(
                &live_files_xml_bytes,
            )),
        },
    )?;
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
    let bytes =
        client.fetch_download_bytes(&archive_target.identifier, &file_name)?;
    fs::write(&output_path, &bytes).with_context(|| {
        format!("Failed to write {}", output_path.display())
    })?;
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
        return Ok(identifier.to_string());
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
) -> Result<VerificationResult> {
    let selected_names: BTreeSet<String> =
        if let Some(selected_files) = selected_files {
            selected_files.clone()
        } else {
            manifest.entries.keys().cloned().collect()
        };

    let mut checked_files = Vec::new();
    let mut missing_files = Vec::new();
    let mut mismatched_files = Vec::new();

    for file_name in &selected_names {
        let Some(entry) = manifest.entries.get(file_name) else {
            missing_files.push(file_name.clone());
            continue;
        };
        let local_path = item_root.join(Path::new(file_name));
        if !local_path.is_file() {
            missing_files.push(file_name.clone());
            continue;
        }

        if let Some(mismatch) = verify_local_file(
            item_root,
            identifier,
            &local_path,
            entry,
            options,
        )? {
            mismatched_files.push(mismatch);
        } else {
            checked_files.push(file_name.clone());
        }
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
) -> Result<Option<VerificationMismatch>> {
    let actual_size = fs::metadata(local_path)
        .with_context(|| {
            format!("Failed to read metadata for {}", local_path.display())
        })?
        .len();
    let relative_name = relative_archive_name(item_root, local_path)?;
    let files_xml_name = format!("{identifier}{FILES_XML_SUFFIX}");
    let is_files_xml = relative_name == files_xml_name;
    if is_files_xml && options.files_xml_live_sha1.is_none() {
        return Ok(None);
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
    if let Some(expected_sha1) = &mismatch.expected_sha1 {
        let actual_sha1 = sha1_hex_for_file(local_path)?;
        if &actual_sha1 != expected_sha1 {
            mismatch.actual_sha1 = Some(actual_sha1);
            has_mismatch = true;
        }
    }
    if let Some(expected_md5) = &mismatch.expected_md5 {
        let actual_md5 = md5_hex_for_file(local_path)?;
        if &actual_md5 != expected_md5 {
            mismatch.actual_md5 = Some(actual_md5);
            has_mismatch = true;
        }
    }

    if has_mismatch {
        Ok(Some(mismatch))
    } else {
        Ok(None)
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

fn fetch_download_bytes(identifier: &str, file_name: &str) -> Result<Vec<u8>> {
    let url = download_url(identifier, file_name)?;
    let response = perform_get(url.as_str())?;
    response.bytes().context("Failed to read download body")
}

fn perform_get(url: &str) -> Result<https::BlockingResponse> {
    let response = https::blocking_get_response(url)
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

fn parse_archive_target(target: &str) -> Result<ArchiveTarget> {
    if let Some(stripped) = target
        .strip_prefix("https://archive.org/download/")
        .or_else(|| target.strip_prefix("http://archive.org/download/"))
    {
        return parse_archive_target_path(stripped);
    }
    if let Some(stripped) = target
        .strip_prefix("https://archive.org/metadata/")
        .or_else(|| target.strip_prefix("http://archive.org/metadata/"))
    {
        return parse_archive_target_path(stripped);
    }
    parse_archive_target_path(target)
}

fn parse_archive_target_path(target: &str) -> Result<ArchiveTarget> {
    let trimmed = target.trim_matches('/');
    if trimmed.is_empty() {
        bail!("An Internet Archive identifier is required");
    }
    let mut parts = trimmed.splitn(2, '/');
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

fn sha1_hex_for_file(path: &Path) -> Result<String> {
    let mut file = File::open(path)
        .with_context(|| format!("Failed to open {}", path.display()))?;
    let mut hasher = Sha1::new();
    let mut buffer = [0_u8; 8192];
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
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn md5_hex_for_file(path: &Path) -> Result<String> {
    let mut file = File::open(path)
        .with_context(|| format!("Failed to open {}", path.display()))?;
    let mut hasher = Md5::new();
    let mut buffer = [0_u8; 8192];
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
#[expect(
    clippy::unwrap_in_result,
    clippy::panic_in_result_fn,
    clippy::indexing_slicing,
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
        let result = verify(&item_path, None, false).unwrap();
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
    fn test_verify_reports_mismatch() {
        let temp_dir = tempfile::tempdir().unwrap();
        let item_path = write_basic_item(&temp_dir);
        fs::write(item_path.join(AUDIO_FILE_NAME), b"HELLO").unwrap();

        let result = verify(&item_path, None, false).unwrap();
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

        let result = verify(&item_path, None, false).unwrap();
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
}
