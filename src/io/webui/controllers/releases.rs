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

//! Controller for serving software release artifacts.
//!
//! Provides routes for the installer/updater to fetch:
//! - Release manifests (latest or by version)
//! - Offline installer tarballs (with Range request support)
//! - Individual chunk data by hash (stored compressed, served decompressed)
//! - Public key for signature verification

use crate::utilities::*;

use anyhow::anyhow;
use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::Response,
};
use base64::Engine;
use ctb_formats_pem::ed25519_base64_to_pem;

use serde::Serialize;
use std::io::Read;
use std::path::{Path as StdPath, PathBuf};
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::utilities::pc_settings::{PcSettingStrKey, get_str_setting};
use crate::utilities::storage::get_storage_dir;
use crate::{AppState, RequestState, error_400, error_404};
use ctb_installer::chunking::{
    compute_sha256_hex, read_chunk_from_directory_compressed,
};
use ctb_installer::manifest::FileEntry;
use ctb_installer::manifest::ReleaseManifest;
use ctb_installer::signing::{KeyId, PUBLIC_KEY_LENGTH, SigningPublicKey};
use ctb_installer::tarball::{StreamingTarball, TarballStream};

/// GET /releases/{platform}/{*artifact}
///
/// Dispatcher used to support URLs like:
/// - `/releases/{platform}/{version}.json`
/// - `/releases/{platform}/{version}.tar`
///
/// Axum's router does not allow `{version}.json` style patterns because that
/// puts both a parameter and a literal in the same path segment.
pub async fn get_platform_dispatch(
    State(state): State<AppState>,
    req: RequestState,
    headers: HeaderMap,
    Path((platform, artifact)): Path<(String, String)>,
) -> Response {
    // The catch-all may include additional slashes. We only support a single
    // segment here.
    if artifact.contains('/') || artifact.contains('\\') {
        return error_404(&state, &req, "Not found".to_string());
    }

    if artifact == "latest.json" {
        return get_latest_manifest(State(state), req, Path(platform)).await;
    }

    if artifact == "latest.json.sig" {
        return get_latest_manifest_sig(State(state), req, Path(platform))
            .await;
    }

    if let Some(version) = artifact.strip_suffix(".json") {
        return get_manifest_by_version(
            State(state),
            req,
            Path((platform, version.to_string())),
        )
        .await;
    }

    if let Some(version) = artifact.strip_suffix(".json.sig") {
        return get_manifest_sig_by_version(
            State(state),
            req,
            Path((platform, version.to_string())),
        )
        .await;
    }

    if let Some(version) = artifact.strip_suffix(".tar") {
        return get_offline_tarball(
            State(state),
            req,
            headers,
            Path((platform, version.to_string())),
        )
        .await;
    }

    error_404(&state, &req, "Not found".to_string())
}

/// GET /releases/chunks/{prefix1}/{prefix2}/{*tail}
///
/// Dispatcher used to support URLs like:
/// - `/releases/chunks/{prefix1}/{prefix2}/{hash}.json`
pub async fn get_chunks_dispatch(
    State(state): State<AppState>,
    req: RequestState,
    Path((prefix1, prefix2, tail)): Path<(String, String, String)>,
) -> Response {
    // The catch-all may include additional slashes. We only support a single
    // segment here.
    if tail.contains('/') || tail.contains('\\') {
        return error_404(&state, &req, "Not found".to_string());
    }

    if tail.ends_with(".json") {
        return get_chunk_json(
            State(state),
            req,
            Path((prefix1, prefix2, tail)),
        )
        .await;
    }

    get_chunk_raw(State(state), req, Path((prefix1, prefix2, tail))).await
}

/// Response structure for chunk data as JSON.
#[derive(Serialize)]
struct ChunkResponse {
    /// SHA-256 hash of the chunk.
    hash: String,
    /// Base64-encoded chunk data.
    data: String,
    /// Length of the chunk in bytes.
    length: u64,
}

/// Response structure for public key endpoint.
#[derive(Serialize)]
struct PublicKeyResponse {
    /// Base64-encoded Ed25519 public key.
    public_key: String,
    /// Key ID (first 8 bytes of public key hash, hex-encoded).
    key_id: String,
}

/// Returns the releases directory within the storage directory.
fn get_releases_dir(
    storage_dir_override: Option<&PathBuf>,
) -> anyhow::Result<PathBuf> {
    let storage = if let Some(path) = storage_dir_override {
        path.clone()
    } else {
        get_storage_dir()?
    };
    Ok(storage.join("releases"))
}

/// Returns the chunks directory (bh/) within the releases directory.
fn get_chunks_dir(
    storage_dir_override: Option<&PathBuf>,
) -> anyhow::Result<PathBuf> {
    Ok(get_releases_dir(storage_dir_override)?.join("bh"))
}

fn find_manifest_file<'a>(
    manifest: &'a ReleaseManifest,
    install_path: &str,
) -> Result<&'a FileEntry, TarballLoadError> {
    manifest
        .files
        .iter()
        .find(|entry| entry.path == install_path)
        .ok_or_else(|| {
            TarballLoadError::BadRequest(format!(
                "Release file not found in manifest: {install_path}"
            ))
        })
}

fn assemble_release_file_bytes(
    entry: &FileEntry,
    chunks_dir: &StdPath,
) -> Result<Vec<u8>, TarballLoadError> {
    let total_len = usize::try_from(entry.file_size).map_err(|_| {
        TarballLoadError::BadRequest(
            "Release file size exceeds usize range".to_string(),
        )
    })?;
    let mut bytes = vec![0u8; total_len];

    for info in &entry.chunks {
        let start = usize::try_from(info.offset).map_err(|_| {
            TarballLoadError::BadRequest(
                "Chunk offset exceeds usize range".to_string(),
            )
        })?;
        let length = usize::try_from(info.length).map_err(|_| {
            TarballLoadError::BadRequest(
                "Chunk length exceeds usize range".to_string(),
            )
        })?;
        let end = start.checked_add(length).ok_or_else(|| {
            TarballLoadError::BadRequest(
                "Chunk range overflow while assembling release file"
                    .to_string(),
            )
        })?;
        if end > bytes.len() {
            return Err(TarballLoadError::BadRequest(format!(
                "Chunk range exceeds assembled file size for {}",
                entry.path
            )));
        }

        let chunk = read_chunk_from_directory_compressed(
            &info.hash,
            chunks_dir,
            info.offset,
        )
        .map_err(|error| TarballLoadError::BadRequest(error.to_string()))?;
        if chunk.data.len() != length {
            return Err(TarballLoadError::BadRequest(format!(
                "Chunk length mismatch for {}: expected {}, got {}",
                info.hash,
                info.length,
                chunk.data.len()
            )));
        }
        if let Some(slice) = bytes.get_mut(start..end) {
            slice.copy_from_slice(&chunk.data);
        }
    }

    let assembled_hash = compute_sha256_hex(&bytes);
    if assembled_hash != entry.checksum {
        return Err(TarballLoadError::BadRequest(format!(
            "Assembled file checksum mismatch for {}: expected {}, got {}",
            entry.path, entry.checksum, assembled_hash
        )));
    }

    Ok(bytes)
}

/// Computes the path to a compressed chunk using two-level prefix.
///
/// Format: `{chunks_dir}/{first-2-chars}/{next-2-chars}/{full-hash}.br`
fn compressed_chunk_file_path(chunks_dir: &StdPath, hash: &str) -> PathBuf {
    if hash.len() >= 4 {
        #[allow(
            clippy::expect_used,
            reason = "hash.len() >= 4 checked in if condition"
        )]
        let prefix1 = hash.get(0..2).expect("hash.len() >= 4");
        #[allow(
            clippy::expect_used,
            reason = "hash.len() >= 4 checked in if condition"
        )]
        let prefix2 = hash.get(2..4).expect("hash.len() >= 4");
        chunks_dir
            .join(prefix1)
            .join(prefix2)
            .join(format!("{hash}.br"))
    } else {
        chunks_dir.join(format!("{hash}.br"))
    }
}

/// GET /releases/{platform}/latest.json
///
/// Serves the latest release manifest for a specific platform.
/// The latest manifest is expected to be symlinked or copied to
/// `{storage_dir}/releases/ctb-{platform}-latest.json`.
pub async fn get_latest_manifest(
    State(state): State<AppState>,
    req: RequestState,
    Path(platform): Path<String>,
) -> Response {
    // Validate target
    let platform = platform.to_lowercase();
    if ![
        "linux-x64",
        "linux-x86",
        "windows-x64",
        "mac-x64",
        "mac-arm64",
    ]
    .contains(&platform.as_str())
    {
        return error_400(
            &state,
            &req,
            anyhow!(
                "Invalid platform: {platform}. Use e.g. linux-x64, linux-x86, windows-x64, mac-x64, or mac-arm64."
            ),
        );
    }

    let releases_dir =
        match get_releases_dir(state.storage_dir_override.as_ref()) {
            Ok(d) => d,
            Err(e) => return error_400(&state, &req, e),
        };

    let manifest_path =
        releases_dir.join(format!("ctb-{platform}-latest.json"));
    serve_json_file(&state, &req, &manifest_path).await
}

/// GET /releases/{platform}/{version}.json
///
/// Serves a specific release manifest by platform and version string.
/// The version should match a manifest file named `ctb-{platform}-{version}.json`.
pub async fn get_manifest_by_version(
    State(state): State<AppState>,
    req: RequestState,
    Path((platform, version)): Path<(String, String)>,
) -> Response {
    // Validate target
    let platform = platform.to_lowercase();
    if ![
        "linux-x64",
        "linux-x86",
        "windows-x64",
        "mac-x64",
        "mac-arm64",
    ]
    .contains(&platform.as_str())
    {
        return error_400(
            &state,
            &req,
            anyhow!(
                "Invalid platform: {platform}. Use e.g. linux-x64, linux-x86, windows-x64, mac-x64, or mac-arm64."
            ),
        );
    }

    // Sanitize version string to prevent path traversal
    if version.contains("..") || version.contains('/') || version.contains('\\')
    {
        return error_400(&state, &req, anyhow!("Invalid version string"));
    }

    let releases_dir =
        match get_releases_dir(state.storage_dir_override.as_ref()) {
            Ok(d) => d,
            Err(e) => return error_400(&state, &req, e),
        };

    // Try to find a manifest matching the platform and version
    // Format: ctb-{platform}-{version}.json
    let manifest_path =
        releases_dir.join(format!("ctb-{platform}-{version}.json"));
    serve_json_file(&state, &req, &manifest_path).await
}

/// GET /releases/chunks/{prefix1}/{prefix2}/{hash}
///
/// Serves raw chunk bytes by SHA-256 hash from the bh/ directory.
/// Chunks are stored compressed (.gz) but served decompressed.
/// Uses two-level prefix: bh/{first-2-chars}/{next-2-chars}/{full-hash}.gz
pub async fn get_chunk_raw(
    State(state): State<AppState>,
    req: RequestState,
    Path((prefix1, prefix2, hash)): Path<(String, String, String)>,
) -> Response {
    // Validate hash format (should be 64 hex characters for SHA-256)
    if !is_valid_chunk_hash(&hash) {
        return error_400(&state, &req, anyhow!("Invalid chunk hash format"));
    }

    // Validate prefixes match hash (already checked length via is_valid_chunk_hash)
    if hash.get(0..2) != Some(prefix1.as_str())
        || hash.get(2..4) != Some(prefix2.as_str())
    {
        return error_400(&state, &req, anyhow!("Prefix mismatch"));
    }

    let chunks_dir = match get_chunks_dir(state.storage_dir_override.as_ref()) {
        Ok(d) => d,
        Err(e) => return error_400(&state, &req, e),
    };

    let chunk_path = compressed_chunk_file_path(&chunks_dir, &hash);

    // Check if compressed file exists
    if !chunk_path.exists() {
        return error_404(&state, &req, format!("Chunk '{hash}' not found"));
    }

    // Read and decompress the chunk using Brotli
    let compressed_data = match tokio::fs::read(&chunk_path).await {
        Ok(d) => d,
        Err(e) => return error_400(&state, &req, anyhow!(e)),
    };

    let mut decoder = brotli::Decompressor::new(&compressed_data[..], 4096);
    let mut decompressed = Vec::new();
    if let Err(e) = decoder.read_to_end(&mut decompressed) {
        return error_400(
            &state,
            &req,
            anyhow!("Failed to decompress chunk: {e}"),
        );
    }

    let body = Body::from(decompressed);

    let mut resp = Response::new(body);
    resp.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );
    resp
}

/// GET /releases/chunks/{prefix}/{hash}.json
///
/// Serves chunk data as JSON containing base64-encoded bytes.
/// Chunks are stored compressed but decompressed for the response.
/// This allows adding metadata in the future if needed.
pub async fn get_chunk_json(
    State(state): State<AppState>,
    req: RequestState,
    Path((prefix1, prefix2, hash)): Path<(String, String, String)>,
) -> Response {
    // The path may include .json suffix, so strip it to get the actual hash
    let hash = hash.trim_end_matches(".json");

    // Validate hash format
    if !is_valid_chunk_hash(hash) {
        return error_400(&state, &req, anyhow!("Invalid chunk hash format"));
    }

    // Validate prefixes match hash (already checked length via is_valid_chunk_hash)
    if hash.get(0..2) != Some(prefix1.as_str())
        || hash.get(2..4) != Some(prefix2.as_str())
    {
        return error_400(&state, &req, anyhow!("Prefix mismatch"));
    }

    let chunks_dir = match get_chunks_dir(state.storage_dir_override.as_ref()) {
        Ok(d) => d,
        Err(e) => return error_400(&state, &req, e),
    };

    let chunk_path = compressed_chunk_file_path(&chunks_dir, hash);

    // Check if compressed file exists
    if !chunk_path.exists() {
        return error_404(&state, &req, format!("Chunk '{hash}' not found"));
    }

    // Read and decompress the chunk data using Brotli
    let compressed_data = match tokio::fs::read(&chunk_path).await {
        Ok(d) => d,
        Err(e) => return error_400(&state, &req, anyhow!(e)),
    };

    let mut decoder = brotli::Decompressor::new(&compressed_data[..], 4096);
    let mut data = Vec::new();
    if let Err(e) = decoder.read_to_end(&mut data) {
        return error_400(
            &state,
            &req,
            anyhow!("Failed to decompress chunk: {e}"),
        );
    }

    // Encode as base64 and wrap in JSON
    let data_b64 = base64::engine::general_purpose::STANDARD.encode(&data);

    let response = ChunkResponse {
        hash: hash.to_string(),
        data: data_b64,
        // Reason for fallback: data buffer length u64 conversion overflow defaults length to 0
        length: u64::try_from(data.len()).unwrap_or(0),
    };

    let json = match serde_json::to_string(&response) {
        Ok(j) => j,
        Err(e) => return error_400(&state, &req, anyhow!(e)),
    };

    let mut resp = Response::new(Body::from(json));
    resp.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    resp
}

/// GET /releases/public-key
///
/// Returns JSON with the server's release verification public key and key ID.
/// The public key is loaded from `pc_settings.release_public_key`.
pub async fn get_public_key(
    State(state): State<AppState>,
    req: RequestState,
) -> Response {
    let Some(pubkey_b64) = get_str_setting(PcSettingStrKey::ReleasePublicKey)
    else {
        return error_404(
            &state,
            &req,
            "Release public key not configured".to_string(),
        );
    };

    // Decode the public key to compute the key ID
    let pubkey_bytes =
        match base64::engine::general_purpose::STANDARD.decode(&pubkey_b64) {
            Ok(b) => b,
            Err(e) => {
                return error_400(
                    &state,
                    &req,
                    anyhow!("Invalid public key encoding: {e}"),
                );
            }
        };

    let pubkey_array: [u8; PUBLIC_KEY_LENGTH] = match pubkey_bytes.try_into() {
        Ok(arr) => arr,
        Err(v) => {
            return error_400(
                &state,
                &req,
                anyhow!(
                    "Invalid public key length: expected {}, got {}",
                    PUBLIC_KEY_LENGTH,
                    v.len()
                ),
            );
        }
    };

    let pubkey = match SigningPublicKey::from_bytes(&pubkey_array) {
        Ok(k) => k,
        Err(e) => {
            return error_400(&state, &req, anyhow!("Invalid public key: {e}"));
        }
    };

    let key_id = KeyId::from_public_key(&pubkey);

    let response = PublicKeyResponse {
        public_key: pubkey_b64,
        key_id: key_id.to_hex(),
    };

    let json = match serde_json::to_string(&response) {
        Ok(j) => j,
        Err(e) => return error_400(&state, &req, anyhow!(e)),
    };

    let mut resp = Response::new(Body::from(json));
    resp.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    resp
}

/// GET /releases/{platform}/{version}.tar
///
/// Serves an offline installer tarball containing the manifest and all chunks.
/// Supports HTTP Range requests for resumable downloads.
///
/// The tarball structure is:
/// ```text
/// ctoolbox-{platform}-{version}/
/// ├── manifest.json
/// ├── chunks/
/// │   ├── {hash1}.br
/// │   ├── {hash2}.br
/// │   └── ...
/// └── ctoolbox-installer (optional, if installer binary exists)
/// ```
///
/// # Range Request Support
///
/// This endpoint supports the `Range` header for resumable downloads:
/// - `Range: bytes=0-` - Start from beginning
/// - `Range: bytes=1000-` - Resume from byte 1000
/// - `Range: bytes=1000-2000` - Request specific range
///
/// Range requests return 206 Partial Content with appropriate headers.
pub async fn get_offline_tarball(
    State(state): State<AppState>,
    req: RequestState,
    headers: HeaderMap,
    Path((platform, version)): Path<(String, String)>,
) -> Response {
    use crate::controllers::base::SizedStreamBody;
    use std::io::Write;
    use tokio::io::{AsyncReadExt, AsyncSeekExt};

    // Validate and load tarball
    let tarball = match load_tarball_for_version(
        state.storage_dir_override.as_ref(),
        &platform,
        &version,
    ) {
        Ok(t) => t,
        Err(TarballLoadError::BadRequest(msg)) => {
            return error_400(&state, &req, anyhow!("{msg}"));
        }
        Err(TarballLoadError::NotFound(msg)) => {
            let fallback_url =
                format!("{}/releases/{platform}/{version}.tar", default_url());
            return if environment::is_official_public_website() {
                error_404(&state, &req, msg)
            } else {
                crate::controllers::base::redirect_temporary(
                    req.is_js_request,
                    &fallback_url,
                )
            };
        }
    };

    let releases_dir =
        match get_releases_dir(state.storage_dir_override.as_ref()) {
            Ok(d) => d,
            Err(e) => return error_400(&state, &req, e),
        };
    let cache_dir = if state.storage_dir_override.is_some() {
        releases_dir.join("downloads_cache")
    } else {
        match ctb_utilities::storage::get_cache_dir() {
            Ok(d) => d.join("downloads_cache"),
            Err(e) => return error_400(&state, &req, e),
        }
    };

    let resolved_version = tarball.version();
    let download_file_name =
        format!("ctoolbox-{platform}-{resolved_version}.tar");
    let cached_file_path = cache_dir.join(&download_file_name);

    // Coordinate concurrent generation
    let mut is_generator = false;

    loop {
        if cached_file_path.exists() {
            break;
        }

        let mut generating = state.generating_downloads.lock().await;
        if cached_file_path.exists() {
            break;
        }

        if generating.contains(&download_file_name) {
            // Someone else is generating it, drop lock and wait
            drop(generating);
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        } else {
            // We are the generator!
            generating.insert(download_file_name.clone());
            is_generator = true;
            break;
        }
    }

    if is_generator {
        let cached_file_path_clone = cached_file_path.clone();
        let tarball_clone = tarball.clone();

        let generate_res =
            tokio::task::spawn_blocking(move || -> Result<()> {
                let temp_file_path =
                    cached_file_path_clone.with_extension("tmp");
                if let Some(parent) = temp_file_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut file = std::fs::File::create(&temp_file_path)?;
                let mut stream = TarballStream::new(tarball_clone);
                while let Some(chunk) = stream.next_chunk()? {
                    file.write_all(&chunk)?;
                }
                file.flush()?;
                std::fs::rename(&temp_file_path, &cached_file_path_clone)?;
                Ok(())
            })
            .await;

        // Clean up from the generating set
        let mut generating = state.generating_downloads.lock().await;
        generating.remove(&download_file_name);
        drop(generating);

        match generate_res {
            Ok(Ok(())) => {}
            Ok(Err(error)) => {
                let fallback_url = format!(
                    "{}/releases/{platform}/{version}.tar",
                    default_url()
                );
                return if environment::is_official_public_website() {
                    error_404(&state, &req, error.to_string())
                } else {
                    crate::controllers::base::redirect_temporary(
                        req.is_js_request,
                        &fallback_url,
                    )
                };
            }
            Err(error) => return error_400(&state, &req, anyhow!(error)),
        }
    }

    // Serve the cached file
    let file = match tokio::fs::File::open(&cached_file_path).await {
        Ok(f) => f,
        Err(e) => {
            return error_400(
                &state,
                &req,
                anyhow!("Failed to open cached tarball file: {e}"),
            );
        }
    };
    let metadata = match file.metadata().await {
        Ok(m) => m,
        Err(e) => {
            return error_400(
                &state,
                &req,
                anyhow!("Failed to read cached tarball metadata: {e}"),
            );
        }
    };
    let total_size = metadata.len();

    let range = parse_range_header(&headers, total_size);

    if let Some((start, end)) = range {
        let mut file = file;
        if let Err(e) = file.seek(std::io::SeekFrom::Start(start)).await {
            return error_400(
                &state,
                &req,
                anyhow!("Failed to seek cached file: {e}"),
            );
        }
        let content_length = end.saturating_sub(start);
        let stream = ReaderStream::new(file.take(content_length));
        let body = Body::new(SizedStreamBody {
            stream,
            size: content_length,
        });

        let mut resp = Response::new(body);
        *resp.status_mut() = StatusCode::PARTIAL_CONTENT;

        let headers_mut = resp.headers_mut();
        headers_mut.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/x-tar"),
        );
        headers_mut
            .insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));

        if let Ok(len_val) = HeaderValue::from_str(&content_length.to_string())
        {
            headers_mut.insert(header::CONTENT_LENGTH, len_val);
        }

        let range_str =
            format!("bytes {start}-{}/{total_size}", end.saturating_sub(1));
        if let Ok(range_val) = HeaderValue::from_str(&range_str) {
            headers_mut.insert(header::CONTENT_RANGE, range_val);
        }

        if let Ok(disposition) = HeaderValue::from_str(&format!(
            "attachment; filename=\"{download_file_name}\""
        )) {
            headers_mut.insert(header::CONTENT_DISPOSITION, disposition);
        }
        headers_mut.insert(
            header::SET_COOKIE,
            HeaderValue::from_static(
                "ctb_download_started=1; Path=/; SameSite=Lax",
            ),
        );
        resp
    } else {
        let stream = ReaderStream::new(file);
        let body = Body::new(SizedStreamBody {
            stream,
            size: total_size,
        });

        let mut resp = Response::new(body);
        let headers_mut = resp.headers_mut();
        headers_mut.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/x-tar"),
        );
        headers_mut
            .insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));

        if let Ok(len_val) = HeaderValue::from_str(&total_size.to_string()) {
            headers_mut.insert(header::CONTENT_LENGTH, len_val);
        }
        if let Ok(disposition) = HeaderValue::from_str(&format!(
            "attachment; filename=\"{download_file_name}\""
        )) {
            headers_mut.insert(header::CONTENT_DISPOSITION, disposition);
        }
        headers_mut.insert(
            header::SET_COOKIE,
            HeaderValue::from_static(
                "ctb_download_started=1; Path=/; SameSite=Lax",
            ),
        );
        resp
    }
}

/// Error type for tarball loading.
enum TarballLoadError {
    BadRequest(String),
    NotFound(String),
}

/// Loads and validates a streaming tarball for the given platform and version.
fn load_tarball_for_version(
    storage_dir_override: Option<&PathBuf>,
    platform: &str,
    version: &str,
) -> Result<StreamingTarball, TarballLoadError> {
    let platform_lower = platform.to_lowercase();
    if ![
        "linux-x86",
        "linux-x64",
        "windows-x64",
        "mac-x64",
        "mac-arm64",
    ]
    .contains(&platform_lower.as_str())
    {
        return Err(TarballLoadError::BadRequest(format!(
            "Invalid platform: {platform}. Use linux-x86, linux-x64, windows-x64, mac-x64, or mac-arm64."
        )));
    }

    if version.contains("..") || version.contains('/') || version.contains('\\')
    {
        return Err(TarballLoadError::BadRequest(
            "Invalid version string".into(),
        ));
    }

    let version = version.trim_end_matches(".tar");

    let releases_dir = get_releases_dir(storage_dir_override)
        .map_err(|e| TarballLoadError::BadRequest(e.to_string()))?;

    let manifest_path = if version == "latest" {
        releases_dir.join(format!("ctb-{platform_lower}-latest.json"))
    } else {
        releases_dir.join(format!("ctb-{platform_lower}-{version}.json"))
    };

    if !manifest_path.exists() {
        return Err(TarballLoadError::NotFound(format!(
            "Release manifest for {platform}/{version} not found"
        )));
    }

    let manifest_content =
        std::fs::read_to_string(&manifest_path).map_err(|e| {
            TarballLoadError::BadRequest(format!(
                "Failed to read manifest: {e}"
            ))
        })?;

    let manifest: ReleaseManifest = serde_json::from_str(&manifest_content)
        .map_err(|e| {
            TarballLoadError::BadRequest(format!(
                "Failed to parse manifest: {e}"
            ))
        })?;

    let chunks_dir = get_chunks_dir(storage_dir_override)
        .map_err(|e| TarballLoadError::BadRequest(e.to_string()))?;

    let installer_bytes = find_manifest_file(&manifest, "ctoolbox-installer")
        .and_then(|entry| assemble_release_file_bytes(entry, &chunks_dir))
        .ok();

    StreamingTarball::new_with_installer_bytes(
        manifest,
        chunks_dir.to_string_lossy().to_string(),
        installer_bytes,
    )
    .map_err(|e| {
        TarballLoadError::BadRequest(format!("Failed to create tarball: {e}"))
    })
}

/// Parses the Range header and returns the start and end byte positions.
///
/// Returns `Some((start, end))` if a valid range was specified, where `end`
/// is exclusive. Returns `None` if no Range header or invalid format.
pub(crate) fn parse_range_header(
    headers: &HeaderMap,
    total_size: u64,
) -> Option<(u64, u64)> {
    let range_header = headers.get(header::RANGE)?;
    let range_str = range_header.to_str().ok()?;

    // Format: "bytes=start-end" or "bytes=start-"
    let range_str = range_str.strip_prefix("bytes=")?;

    let (start_str, end_str) = range_str.split_once('-')?;

    let start: u64 = start_str.parse().ok()?;

    let end: u64 = if end_str.is_empty() {
        total_size
    } else {
        // HTTP ranges are inclusive, so add 1 for exclusive end
        end_str
            .parse::<u64>()
            .ok()?
            .saturating_add(1)
            .min(total_size)
    };

    if start >= end || start >= total_size {
        return None;
    }

    Some((start, end))
}

/// Validates that a string is a valid SHA-256 chunk hash (64 hex characters).
fn is_valid_chunk_hash(hash: &str) -> bool {
    hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit())
}

/// Helper to serve a JSON file from disk.
async fn serve_json_file(
    state: &AppState,
    req: &RequestState,
    path: &PathBuf,
) -> Response {
    // Check if file exists
    if !path.exists() {
        return error_404(
            state,
            req,
            format!(
                "Manifest '{}' not found",
                // Reason for fallback: file path without file_name component formats name as "unknown"
                path.file_name().map_or_else(
                    || "unknown".to_string(),
                    |s| s.to_string_lossy().into_owned()
                )
            ),
        );
    }

    // Open and stream the file
    let file = match File::open(path).await {
        Ok(f) => f,
        Err(e) => return error_400(state, req, anyhow!(e)),
    };

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let mut resp = Response::new(body);
    resp.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    resp
}

/// Helper to serve a signature file from disk.
async fn serve_sig_file(
    state: &AppState,
    req: &RequestState,
    path: &PathBuf,
) -> Response {
    if !path.exists() {
        return error_404(
            state,
            req,
            format!(
                "Signature '{}' not found",
                // Reason for fallback: file path without file_name component formats name as "unknown"
                path.file_name().map_or_else(
                    || "unknown".to_string(),
                    |s| s.to_string_lossy().into_owned()
                )
            ),
        );
    }

    let file = match File::open(path).await {
        Ok(f) => f,
        Err(e) => return error_400(state, req, anyhow!(e)),
    };

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let mut resp = Response::new(body);
    resp.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );
    resp
}

/// GET /releases/{platform}/latest.json.sig
pub async fn get_latest_manifest_sig(
    State(state): State<AppState>,
    req: RequestState,
    Path(platform): Path<String>,
) -> Response {
    let platform = platform.to_lowercase();
    if ![
        "linux-x64",
        "linux-x86",
        "windows-x64",
        "mac-x64",
        "mac-arm64",
    ]
    .contains(&platform.as_str())
    {
        return error_400(&state, &req, anyhow!("Invalid platform"));
    }

    let releases_dir =
        match get_releases_dir(state.storage_dir_override.as_ref()) {
            Ok(d) => d,
            Err(e) => return error_400(&state, &req, e),
        };

    let mut sig_path =
        releases_dir.join(format!("ctb-{platform}-latest.json.sig"));
    if !sig_path.exists() {
        let manifest_path =
            releases_dir.join(format!("ctb-{platform}-latest.json"));
        if let Ok(resolved_manifest) =
            files::symlink_is_in_dir(&manifest_path, &releases_dir)
        {
            if resolved_manifest != manifest_path {
                let mut sig_file = resolved_manifest.clone();
                if let Some(ext) = resolved_manifest.extension() {
                    let mut new_ext = ext.to_os_string();
                    new_ext.push(".sig");
                    sig_file.set_extension(new_ext);
                }
                if sig_file.exists() {
                    sig_path = sig_file;
                }
            }
        }
    }
    serve_sig_file(&state, &req, &sig_path).await
}

/// GET /releases/{platform}/{version}.json.sig
pub async fn get_manifest_sig_by_version(
    State(state): State<AppState>,
    req: RequestState,
    Path((platform, version)): Path<(String, String)>,
) -> Response {
    let platform = platform.to_lowercase();
    if ![
        "linux-x64",
        "linux-x86",
        "windows-x64",
        "mac-x64",
        "mac-arm64",
    ]
    .contains(&platform.as_str())
    {
        return error_400(&state, &req, anyhow!("Invalid platform"));
    }

    if version.contains("..") || version.contains('/') || version.contains('\\')
    {
        return error_400(&state, &req, anyhow!("Invalid version string"));
    }

    let releases_dir =
        match get_releases_dir(state.storage_dir_override.as_ref()) {
            Ok(d) => d,
            Err(e) => return error_400(&state, &req, e),
        };

    let sig_path =
        releases_dir.join(format!("ctb-{platform}-{version}.json.sig"));
    serve_sig_file(&state, &req, &sig_path).await
}

/// GET /releases/public-key.pem
pub async fn get_public_key_pem(
    State(state): State<AppState>,
    req: RequestState,
) -> Response {
    let Some(pubkey_b64) = get_str_setting(PcSettingStrKey::ReleasePublicKey)
    else {
        return error_404(
            &state,
            &req,
            "Release public key not configured".to_string(),
        );
    };

    let pem = ed25519_base64_to_pem(&pubkey_b64);

    let mut resp = Response::new(Body::from(pem));
    resp.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/x-pem-file"),
    );
    resp
}

/// Calculates download filesizes for the homepage based on latest manifests.
pub async fn calculate_download_sizes(
    storage_dir_override: Option<PathBuf>,
) -> std::collections::HashMap<String, String> {
    tokio::task::spawn_blocking(move || {
        let mut sizes = std::collections::HashMap::new();

        let releases_dir = match get_releases_dir(storage_dir_override.as_ref())
        {
            Ok(d) => d,
            Err(_) => return sizes,
        };

        // Load x64 manifest
        let x64_path = releases_dir.join("ctb-linux-x64-latest.json");
        if let Ok(content) = std::fs::read_to_string(&x64_path) {
            if let Ok(manifest) =
                serde_json::from_str::<ReleaseManifest>(&content)
            {
                let installer_size = manifest.installer_file_size();
                if installer_size > 0 {
                    sizes.insert(
                        "linux_x64_installer".to_string(),
                        ctb_utilities::string::bytes::format_bytes_decimal(
                            installer_size,
                        ),
                    );
                }
                if let Some(installer_file) = manifest
                    .files
                    .iter()
                    .find(|f| f.path == "ctoolbox-installer")
                {
                    sizes.insert(
                        "linux_x64_installer_checksum".to_string(),
                        installer_file.checksum.clone(),
                    );
                }

                if let Ok(offline_size) = manifest.estimate_offline_tarball_size() {
                    sizes.insert(
                        "linux_x64_offline".to_string(),
                        format!(
                            "approx. {}",
                            ctb_utilities::string::bytes::format_bytes_decimal(
                                offline_size
                            )
                        ),
                    );
                }

                let build_info = crate::utilities::build_info();
                let src_name = format!(
                    "ctoolbox-src-{}-{}.tar",
                    manifest.ctoolbox_version, build_info.commit
                );
                if let Ok(src_size) = manifest.estimate_gzipped_file_size(&src_name) {
                    if src_size > 0 {
                        sizes.insert(
                            "src_tar_gz".to_string(),
                            format!(
                                "approx. {}",
                                ctb_utilities::string::bytes::format_bytes_decimal(
                                    src_size
                                )
                            ),
                        );
                    }
                }

                let dep_name = format!(
                    "ctoolbox-dependencies-{}-{}.tar",
                    manifest.ctoolbox_version, build_info.commit
                );
                if let Ok(dep_size) = manifest.estimate_gzipped_file_size(&dep_name) {
                    if dep_size > 0 {
                        sizes.insert(
                            "dependencies_tar_gz".to_string(),
                            format!(
                                "approx. {}",
                                ctb_utilities::string::bytes::format_bytes_decimal(
                                    dep_size
                                )
                            ),
                        );
                    }
                }
            }
        }

        // Load x86 manifest
        let x86_path = releases_dir.join("ctb-linux-x86-latest.json");
        if let Ok(content) = std::fs::read_to_string(&x86_path) {
            if let Ok(manifest) =
                serde_json::from_str::<ReleaseManifest>(&content)
            {
                let installer_size = manifest.installer_file_size();
                if installer_size > 0 {
                    sizes.insert(
                        "linux_x86_installer".to_string(),
                        ctb_utilities::string::bytes::format_bytes_decimal(
                            installer_size,
                        ),
                    );
                }
                if let Some(installer_file) = manifest
                    .files
                    .iter()
                    .find(|f| f.path == "ctoolbox-installer")
                {
                    sizes.insert(
                        "linux_x86_installer_checksum".to_string(),
                        installer_file.checksum.clone(),
                    );
                }

                if let Ok(offline_size) = manifest.estimate_offline_tarball_size() {
                    sizes.insert(
                        "linux_x86_offline".to_string(),
                        format!(
                            "approx. {}",
                            ctb_utilities::string::bytes::format_bytes_decimal(
                                offline_size
                            )
                        ),
                    );
                }
            }
        }

        sizes
    })
    .await
    // Reason for fallback: tokio spawn_blocking join error defaults sizes map to empty HashMap
    .unwrap_or_default()
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

    #[crate::ctb_test]
    fn test_is_valid_chunk_hash() {
        // Valid SHA-256 hash (64 hex chars)
        assert!(is_valid_chunk_hash(
            "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2"
        ));

        // Too short
        assert!(!is_valid_chunk_hash("a1b2c3d4"));

        // Too long
        assert!(!is_valid_chunk_hash(
            "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2a1b2"
        ));

        // Invalid characters
        assert!(!is_valid_chunk_hash(
            "g1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2"
        ));

        // Empty
        assert!(!is_valid_chunk_hash(""));
    }

    #[crate::ctb_test]
    fn test_parse_range_header() {
        use axum::http::HeaderMap;

        let mut headers = HeaderMap::new();

        // No Range header
        assert_eq!(parse_range_header(&headers, 100), None);

        // Invalid format
        headers.insert(header::RANGE, "bytes=invalid".parse().unwrap());
        assert_eq!(parse_range_header(&headers, 100), None);

        // Valid full range from start
        headers.insert(header::RANGE, "bytes=0-".parse().unwrap());
        assert_eq!(parse_range_header(&headers, 100), Some((0, 100)));

        // Valid mid range to end
        headers.insert(header::RANGE, "bytes=50-".parse().unwrap());
        assert_eq!(parse_range_header(&headers, 100), Some((50, 100)));

        // Valid specific range
        headers.insert(header::RANGE, "bytes=10-49".parse().unwrap());
        assert_eq!(parse_range_header(&headers, 100), Some((10, 50)));

        // Unsatisfiable range (start >= total_size)
        headers.insert(header::RANGE, "bytes=100-".parse().unwrap());
        assert_eq!(parse_range_header(&headers, 100), None);

        // Unsatisfiable range (start >= end)
        headers.insert(header::RANGE, "bytes=50-40".parse().unwrap());
        assert_eq!(parse_range_header(&headers, 100), None);
    }

    #[crate::ctb_test("tokio")]
    async fn test_get_latest_manifest_sig_fallback() {
        //bypass-tempdir-lint
        use crate::test_helpers::TestApp;
        use axum::http::{Method, StatusCode};
        use std::fs;

        let test_app = TestApp::new();
        let releases_dir = test_app
            .state
            .storage_dir_override
            .as_ref()
            .unwrap()
            .join("releases");
        fs::create_dir_all(&releases_dir).unwrap();

        // Create a dummy manifest file
        let manifest_filename = "ctb-linux-x64-20260617-120000.json";
        let manifest_path = releases_dir.join(manifest_filename);
        fs::write(&manifest_path, "{}").unwrap();

        // Create the corresponding signature file
        let sig_filename = "ctb-linux-x64-20260617-120000.json.sig";
        let sig_path = releases_dir.join(sig_filename);
        fs::write(&sig_path, "signature-bytes").unwrap();

        // Create a symlink ctb-linux-x64-latest.json pointing to the manifest file
        let latest_manifest_path =
            releases_dir.join("ctb-linux-x64-latest.json");
        #[cfg(unix)]
        std::os::unix::fs::symlink(manifest_filename, &latest_manifest_path)
            .unwrap();
        #[cfg(not(unix))]
        fs::write(&latest_manifest_path, "{}").unwrap();

        // Request latest.json.sig
        let resp = test_app
            .request_get_response::<()>(
                Method::GET,
                "/releases/linux-x64/latest.json.sig",
                None,
                None,
                None,
                None,
            )
            .await;

        assert_eq!(resp.status(), StatusCode::OK);
        let body = crate::test_helpers::body_to_text(resp).await;
        assert_eq!(body, "signature-bytes");
    }

    #[crate::ctb_test("tokio")]
    async fn test_offline_tarball_caching_and_range_requests() {
        //bypass-tempdir-lint
        use crate::test_helpers::TestApp;
        use axum::http::{HeaderMap, Method, StatusCode};
        use std::fs;

        let test_app = TestApp::new();
        let releases_dir = test_app
            .state
            .storage_dir_override
            .as_ref()
            .unwrap()
            .join("releases");
        fs::create_dir_all(&releases_dir).unwrap();

        let chunks_dir = releases_dir.join("bh");
        fs::create_dir_all(&chunks_dir).unwrap();

        // Write a mock manifest ctb-linux-x64-latest.json
        let manifest_content = r#"{
            "format_version": 1,
            "ctoolbox_version": "0.1.0",
            "platform": "linux-x64",
            "date": "2026-06-16T22:43:00Z",
            "revoked_key_ids": [],
            "files": []
        }"#;

        fs::write(
            releases_dir.join("ctb-linux-x64-latest.json"),
            manifest_content,
        )
        .unwrap();

        // 1. Request full tarball
        let resp = test_app
            .request_get_response::<()>(
                Method::GET,
                "/releases/linux-x64/latest.tar",
                None,
                None,
                None,
                None,
            )
            .await;

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers().get(header::CONTENT_TYPE).unwrap(),
            "application/x-tar"
        );

        let full_body = crate::test_helpers::body_to_bytes(resp).await;
        assert!(full_body.len() > 1024); // Must have at least the directory header, manifest, and 1024 trailing null bytes

        // Check that the cached file was created on disk
        let cache_file_path = releases_dir
            .join("downloads_cache")
            .join("ctoolbox-linux-x64-0.1.0.tar");
        assert!(cache_file_path.exists());
        let cached_metadata = fs::metadata(&cache_file_path).unwrap();
        assert_eq!(
            cached_metadata.len(),
            u64::try_from(full_body.len()).unwrap()
        );

        // 2. Request range from cached file
        let mut headers = HeaderMap::new();
        headers.insert(header::RANGE, "bytes=10-20".parse().unwrap());

        let resp_range = test_app
            .request_get_response::<()>(
                Method::GET,
                "/releases/linux-x64/latest.tar",
                Some(headers),
                None,
                None,
                None,
            )
            .await;

        assert_eq!(resp_range.status(), StatusCode::PARTIAL_CONTENT);
        assert_eq!(
            resp_range.headers().get(header::CONTENT_RANGE).unwrap(),
            &format!("bytes 10-20/{}", full_body.len())
        );
        assert_eq!(
            resp_range.headers().get(header::CONTENT_LENGTH).unwrap(),
            "11"
        );

        let range_body = crate::test_helpers::body_to_bytes(resp_range).await;
        assert_eq!(range_body.len(), 11);
        assert_eq!(&range_body[..], &full_body[10..21]);
    }
}
