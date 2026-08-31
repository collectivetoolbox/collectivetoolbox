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

//! Chunk downloading for the installer/updater.
//!
//! This module implements downloading of release manifests and file chunks
//! from a ctoolbox update server. It supports:
//!
//! - Downloading manifests (latest or by version)
//! - Downloading individual chunks with retry logic and exponential backoff
//! - Assembling complete files from chunks
//! - Resuming interrupted downloads via local chunk cache
//! - Progress events for UI feedback

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::chunking::{Chunk, compute_sha256_hex, verify_file};
use crate::manifest::{ChunkInfo, FileEntry, ReleaseManifest};
use crate::signing::{KeyId, SigningPublicKey, public_key_from_base64};

/// Default timeout for HTTP requests in seconds.
const REQUEST_TIMEOUT_SECS: u64 = 30;

/// Number of chunks from a single file to download in parallel.
const FILE_CHUNK_DOWNLOAD_PARALLELISM: usize = 4;

/// Returns the current target string for use in URLs.
///
/// Examples: "linux-x64", "linux-x86", "windows-x64", "mac-x64",
/// "mac-arm64".
#[must_use]
pub fn current_platform() -> String {
    #[cfg(target_os = "linux")]
    {
        if std::env::consts::ARCH == "x86" {
            "linux-x86".to_string()
        } else {
            "linux-x64".to_string()
        }
    }
    #[cfg(target_os = "windows")]
    {
        "windows-x64".to_string()
    }
    #[cfg(target_os = "macos")]
    {
        match std::env::consts::ARCH {
            "aarch64" => "mac-arm64".to_string(),
            _ => "mac-x64".to_string(),
        }
    }
    #[cfg(not(any(
        target_os = "linux",
        target_os = "windows",
        target_os = "macos"
    )))]
    {
        "linux-x64".to_string()
    }
}

/// Events emitted during download for progress tracking.
#[derive(Debug, Clone)]
pub enum DownloadEvent {
    /// Installation plan is ready and includes this many files.
    InstallPlan {
        /// Total number of files that will be installed.
        total_files: usize,
    },
    /// A chunk was successfully downloaded and verified.
    ChunkDownloaded {
        /// Hash of the downloaded chunk.
        hash: String,
        /// Size of the chunk in bytes.
        size: u64,
        /// Current chunk index (1-based).
        current: usize,
        /// Total number of chunks to download.
        total: usize,
    },
    /// A chunk was found in the local cache (resume).
    ChunkCached {
        /// Hash of the cached chunk.
        hash: String,
        /// Current chunk index (1-based).
        current: usize,
        /// Total number of chunks.
        total: usize,
    },
    /// A file was successfully assembled from chunks.
    FileAssembled {
        /// Path to the assembled file.
        path: PathBuf,
        /// Total size of the file in bytes.
        size: u64,
    },
    /// An error occurred during download (non-fatal, will retry).
    RetryError {
        /// Description of the error.
        message: String,
        /// Retry attempt number.
        attempt: u32,
        /// Maximum attempts.
        max_attempts: u32,
    },
    /// Download is starting for a file.
    FileStarted {
        /// Path of the file being downloaded.
        path: String,
        /// Number of chunks to download.
        chunk_count: usize,
    },
    /// Installation completed successfully.
    InstallCompleted {
        /// Number of files installed.
        installed_files: usize,
    },
    /// Installation was cancelled by the user.
    InstallCancelled {
        /// Number of files completed before cancellation.
        completed_files: usize,
    },
    /// Installation failed terminally.
    InstallFailed {
        /// User-visible error message.
        message: String,
    },
    /// Non-fatal warning message for UI display.
    Warning {
        /// Warning message text.
        message: String,
    },
}

/// Locates an offline release manifest JSON file if present alongside the binary or in CWD.
#[must_use]
pub fn find_offline_manifest() -> Option<PathBuf> {
    let cwd_manifest = PathBuf::from("manifest.json");
    if cwd_manifest.is_file() {
        return Some(cwd_manifest);
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            let exe_manifest = parent.join("manifest.json");
            if exe_manifest.is_file() {
                return Some(exe_manifest);
            }
        }
    }

    None
}

/// Locates an offline chunks directory if present alongside the binary or in CWD.
#[must_use]
pub fn find_offline_chunks_dir() -> Option<PathBuf> {
    let cwd_chunks = PathBuf::from("chunks");
    if cwd_chunks.is_dir() {
        return Some(cwd_chunks);
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            let exe_chunks = parent.join("chunks");
            if exe_chunks.is_dir() {
                return Some(exe_chunks);
            }
        }
    }

    None
}

#[derive(Debug, Deserialize)]
struct PublicKeyResponse {
    public_key: String,
    key_id: String,
}

/// Progress callback type for download events.
pub type ProgressCallback = Arc<dyn Fn(DownloadEvent) + Send + Sync>;

/// Shared cancellation flag for long-running installer operations.
pub type CancellationFlag = Arc<AtomicBool>;

/// Error string used when an installer operation is cancelled.
pub const INSTALL_CANCELLED_MESSAGE: &str = "Installation cancelled";

/// Returns whether a cancellation flag has been set.
#[must_use]
pub fn is_cancellation_requested(flag: Option<&CancellationFlag>) -> bool {
    flag.is_some_and(|cancel_flag| cancel_flag.load(Ordering::Relaxed))
}

/// Boxed future used by the HTTP abstraction.
pub type BoxFuture<'a, T> =
    Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;

/// Minimal HTTP response used by the downloader.
#[derive(Debug, Clone)]
pub struct HttpResponse {
    /// HTTP status code.
    pub status: u16,
    /// Raw response body bytes.
    pub body: Vec<u8>,
}

impl HttpResponse {
    fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    fn body_as_utf8(&self) -> Result<String> {
        String::from_utf8(self.body.clone())
            .context("HTTP response body is not valid UTF-8")
    }
}

/// Abstraction over HTTP so tests can run without binding to a real port.
///
/// The production implementation uses the shared HTTP wrapper, while tests can
/// provide an in-memory implementation.
pub trait HttpClient: Send + Sync {
    /// Performs a GET request and returns the full response.
    fn get<'a>(&'a self, url: &'a str) -> BoxFuture<'a, Result<HttpResponse>>;
}

#[derive(Clone)]
struct SharedHttpClient {
    client: https::AsyncClient,
}

impl SharedHttpClient {
    fn new(client: https::AsyncClient) -> Self {
        Self { client }
    }
}

impl HttpClient for SharedHttpClient {
    fn get<'a>(&'a self, url: &'a str) -> BoxFuture<'a, Result<HttpResponse>> {
        Box::pin(async move {
            let response = self
                .client
                .get_with_backoff(url, 10)
                .await
                .with_context(|| format!("Failed to GET {url}"))?;

            let status = response.status_code();
            let body = response
                .bytes()
                .await
                .context("Failed to read HTTP response body")?;

            Ok(HttpResponse { status, body })
        })
    }
}

/// A no-op progress callback for when progress tracking isn't needed.
pub fn no_progress_callback() -> ProgressCallback {
    Arc::new(|_| {})
}

/// Downloader for release manifests and file chunks.
///
/// Handles all HTTP communication with the update server, including
/// retry logic, verification, and caching.
#[derive(Clone)]
pub struct ChunkDownloader {
    /// Base URL of the update server (e.g., "`https://example.com`").
    server_url: String,
    /// HTTP client for making requests.
    http_client: Arc<dyn HttpClient>,
    /// Callback for progress events.
    progress_callback: ProgressCallback,
}

impl ChunkDownloader {
    /// Creates a new chunk downloader.
    ///
    /// # Arguments
    /// - `server_url`: Base URL of the update server
    /// - `progress_callback`: Callback invoked for download progress events
    ///
    /// # Errors
    /// Returns an error if the HTTP client cannot be created.
    pub fn new(
        server_url: impl Into<String>,
        progress_callback: ProgressCallback,
    ) -> Result<Self> {
        let http_client = https::async_client(https::ClientOptions {
            timeout: Some(Duration::from_secs(REQUEST_TIMEOUT_SECS)),
            user_agent: Some(format!(
                "ctoolbox-installer/{}",
                environment::ctb_version()
            )),
            ..https::ClientOptions::default()
        })
        .context("Failed to create HTTP client")?;

        Ok(Self {
            server_url: server_url.into().trim_end_matches('/').to_string(),
            http_client: Arc::new(SharedHttpClient::new(http_client)),
            progress_callback,
        })
    }

    /// Creates a new chunk downloader with a custom HTTP client.
    ///
    /// This is useful for testing or when custom client configuration is needed.
    pub fn with_client(
        server_url: impl Into<String>,
        http_client: https::AsyncClient,
        progress_callback: ProgressCallback,
    ) -> Self {
        Self {
            server_url: server_url.into().trim_end_matches('/').to_string(),
            http_client: Arc::new(SharedHttpClient::new(http_client)),
            progress_callback,
        }
    }

    /// Creates a new chunk downloader with a custom HTTP implementation.
    ///
    /// This is primarily intended for tests to avoid binding to TCP ports.
    pub fn with_http_client(
        server_url: impl Into<String>,
        http_client: Arc<dyn HttpClient>,
        progress_callback: ProgressCallback,
    ) -> Self {
        Self {
            server_url: server_url.into().trim_end_matches('/').to_string(),
            http_client,
            progress_callback,
        }
    }

    /// Emits a progress event.
    pub fn emit(&self, event: DownloadEvent) {
        (self.progress_callback)(event);
    }

    /// Downloads the release manifest.
    ///
    /// # Arguments
    /// - `platform`: The target platform (linux, windows, mac)
    /// - `version`: Optional version string. If `None`, downloads the latest
    ///   manifest.
    ///
    /// # Returns
    /// The parsed release manifest.
    ///
    /// # Errors
    /// Returns an error if the manifest cannot be downloaded or parsed.
    pub async fn download_manifest(
        &self,
        platform: &str,
        version: Option<&str>,
    ) -> Result<ReleaseManifest> {
        let url = match version {
            Some(v) => {
                format!("{}/releases/{}/{}.json", self.server_url, platform, v)
            }
            None => {
                format!("{}/releases/{}/latest.json", self.server_url, platform)
            }
        };

        // Try to download the manifest.
        let response =
            self.http_client.get(&url).await.with_context(|| {
                format!("Failed to fetch manifest from {url}")
            })?;

        if !response.is_success() {
            bail!("Server returned {} for manifest: {}", response.status, url);
        }

        let body = response
            .body_as_utf8()
            .context("Failed to read manifest response body")?;

        let manifest: ReleaseManifest = serde_json::from_str(&body)
            .context("Failed to parse manifest JSON")?;

        Ok(manifest)
    }

    /// Downloads the release verification public key from the server.
    pub async fn download_public_key(&self) -> Result<SigningPublicKey> {
        let url = format!("{}/releases/public-key", self.server_url);
        let response = self.http_client.get(&url).await.with_context(|| {
            format!("Failed to fetch public key from {url}")
        })?;

        if !response.is_success() {
            bail!(
                "Server returned {} for public key: {}",
                response.status,
                url
            );
        }

        let body = response
            .body_as_utf8()
            .context("Failed to read public key response body")?;
        let payload: PublicKeyResponse = serde_json::from_str(&body)
            .context("Failed to parse public key response")?;
        let public_key = public_key_from_base64(&payload.public_key)
            .context("Failed to decode installer public key")?;
        let expected_key_id = KeyId::from_public_key(&public_key).to_hex();
        if payload.key_id != expected_key_id {
            bail!(
                "Public key key_id mismatch: server returned {}, computed {}",
                payload.key_id,
                expected_key_id
            );
        }

        Ok(public_key)
    }

    /// Downloads a single chunk by hash with retry logic.
    ///
    /// Uses exponential backoff with jitter for retries.
    ///
    /// # Arguments
    /// - `hash`: SHA-256 hash of the chunk (hex-encoded)
    /// - `expected_length`: Expected length of the chunk for validation
    ///
    /// # Returns
    /// The downloaded chunk with verified data.
    ///
    /// # Errors
    /// Returns an error if all retry attempts fail or the chunk fails
    /// verification.
    pub async fn download_chunk(
        &self,
        hash: &str,
        expected_length: u64,
    ) -> Result<Chunk> {
        if hash.len() < 4 {
            bail!("Invalid chunk hash length: {hash}");
        }
        #[allow(
            clippy::expect_used,
            reason = "hash.len() >= 4 checked above"
        )]
        let prefix1 = hash.get(0..2).expect("hash.len() >= 4");
        #[allow(
            clippy::expect_used,
            reason = "hash.len() >= 4 checked above"
        )]
        let prefix2 = hash.get(2..4).expect("hash.len() >= 4");
        let url = format!(
            "{}/releases/chunks/{}/{}/{}",
            self.server_url, prefix1, prefix2, hash
        );

        let response = self
            .http_client
            .get(&url)
            .await
            .with_context(|| format!("Failed to fetch chunk {hash}"))?;

        if !response.is_success() {
            bail!("Server returned {} for chunk {}", response.status, hash);
        }

        let data = response.body;

        // Verify length
        let actual_length =
            u64::try_from(data.len()).context("Chunk length exceeds u64")?;
        if actual_length != expected_length {
            bail!(
                "Chunk length mismatch for {hash}: expected {expected_length}, got {actual_length}"
            );
        }

        // Verify hash
        let computed_hash = compute_sha256_hex(&data);
        if computed_hash != hash {
            bail!(
                "Chunk hash mismatch: expected {hash}, computed {computed_hash}"
            );
        }

        Ok(Chunk {
            hash: hash.to_string(),
            offset: 0, // Will be set by caller
            length: actual_length,
            data,
            compressed_size: None,
        })
    }

    /// Downloads a file by fetching missing chunks and assembling them.
    ///
    /// Supports resumption by checking the cache directory for existing chunks.
    /// Each chunk is verified on arrival before being written to cache.
    ///
    /// # Arguments
    /// - `entry`: The file entry from the manifest describing the file
    /// - `cache_dir`: Directory to store/read cached chunks
    /// - `output_path`: Where to write the assembled file
    ///
    /// # Returns
    /// Path to the assembled file.
    ///
    /// # Errors
    /// Returns an error if chunks cannot be downloaded or the file cannot be
    /// assembled.
    pub async fn download_file(
        &self,
        entry: &FileEntry,
        cache_dir: &Path,
        output_path: &Path,
        cancel_flag: Option<&CancellationFlag>,
    ) -> Result<PathBuf> {
        // Emit start event
        (self.progress_callback)(DownloadEvent::FileStarted {
            path: entry.path.clone(),
            chunk_count: entry.chunks.len(),
        });

        // Create cache directory if needed
        fs::create_dir_all(cache_dir).await.with_context(|| {
            format!("Failed to create cache directory: {}", cache_dir.display())
        })?;

        // Create output directory if needed
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).await.with_context(|| {
                format!(
                    "Failed to create output directory: {}",
                    parent.display()
                )
            })?;
        }

        // Calculate total file size for pre-allocation
        let total_size: u64 = entry.chunks.iter().map(|c| c.length).sum();

        // Create a sparse file by seeking to the end and truncating
        let file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(output_path)
            .await
            .with_context(|| {
                format!(
                    "Failed to create output file: {}",
                    output_path.display()
                )
            })?;

        file.set_len(total_size).await.with_context(|| {
            format!("Failed to pre-allocate file: {}", output_path.display())
        })?;
        drop(file);

        // Download and write multiple chunks in parallel while keeping the
        // number of in-flight operations bounded.
        let total_chunks = entry.chunks.len();
        let mut pending_chunks = entry.chunks.iter().cloned().enumerate();
        let mut join_set = tokio::task::JoinSet::new();
        let cache_dir_path = cache_dir.to_path_buf();
        let output_file_path = output_path.to_path_buf();
        let cancel_flag = cancel_flag.cloned();

        while join_set.len() < FILE_CHUNK_DOWNLOAD_PARALLELISM {
            let Some((index, chunk_info)) = pending_chunks.next() else {
                break;
            };
            spawn_chunk_download_task(
                self.clone(),
                &mut join_set,
                chunk_info,
                cache_dir_path.clone(),
                output_file_path.clone(),
                index.saturating_add(1),
                total_chunks,
                cancel_flag.clone(),
            );
        }

        while let Some(join_result) = join_set.join_next().await {
            match join_result {
                Ok(Ok(())) => {}
                Ok(Err(error)) => {
                    join_set.abort_all();
                    return Err(error);
                }
                Err(error) => {
                    join_set.abort_all();
                    return Err(anyhow::anyhow!(
                        "Chunk download task failed: {error}"
                    ));
                }
            }

            if is_cancellation_requested(cancel_flag.as_ref()) {
                join_set.abort_all();
                bail!(INSTALL_CANCELLED_MESSAGE);
            }

            let Some((index, chunk_info)) = pending_chunks.next() else {
                continue;
            };

            spawn_chunk_download_task(
                self.clone(),
                &mut join_set,
                chunk_info,
                cache_dir_path.clone(),
                output_file_path.clone(),
                index.saturating_add(1),
                total_chunks,
                cancel_flag.clone(),
            );
        }

        // Verify the assembled file
        if !verify_file(output_file_path.as_path(), &entry.checksum) {
            bail!(
                "Assembled file checksum mismatch for {}. Expected: {}",
                entry.path,
                entry.checksum
            );
        }

        // Emit completion event
        (self.progress_callback)(DownloadEvent::FileAssembled {
            path: output_file_path.clone(),
            size: total_size,
        });

        Ok(output_file_path)
    }

    /// Gets a chunk from cache or downloads it.
    async fn get_or_download_chunk(
        &self,
        info: &ChunkInfo,
        cache_dir: &Path,
        current: usize,
        total: usize,
    ) -> Result<Chunk> {
        let chunk_path = chunk_cache_path(cache_dir, &info.hash);

        // Check if chunk is already cached
        if chunk_path.exists() {
            let data = fs::read(&chunk_path).await.with_context(|| {
                format!("Failed to read cached chunk: {}", chunk_path.display())
            })?;

            // Verify cached chunk
            let computed_hash = compute_sha256_hex(&data);
            if computed_hash == info.hash {
                (self.progress_callback)(DownloadEvent::ChunkCached {
                    hash: info.hash.clone(),
                    current,
                    total,
                });

                return Ok(Chunk {
                    hash: info.hash.clone(),
                    offset: info.offset,
                    length: info.length,
                    data,
                    compressed_size: info.compressed_size,
                });
            }
            // Cached chunk is corrupted, will re-download
            let _ = fs::remove_file(&chunk_path).await;
        }

        // Check if chunk is present in offline bundle chunks directory
        if let Some(offline_dir) = find_offline_chunks_dir() {
            let hash_clone = info.hash.clone();
            let offset = info.offset;
            let read_res = tokio::task::spawn_blocking(move || {
                crate::chunking::read_chunk_from_directory_compressed(
                    &hash_clone,
                    &offline_dir,
                    offset,
                )
            })
            .await;

            if let Ok(Ok(chunk)) = read_res {
                let _ = self.cache_chunk(&chunk, cache_dir).await;

                (self.progress_callback)(DownloadEvent::ChunkCached {
                    hash: info.hash.clone(),
                    current,
                    total,
                });

                return Ok(chunk);
            }
        }

        // Download the chunk
        let mut chunk = self.download_chunk(&info.hash, info.length).await?;
        chunk.offset = info.offset;
        chunk.compressed_size = info.compressed_size;

        // Cache the chunk for resumption
        self.cache_chunk(&chunk, cache_dir).await?;

        (self.progress_callback)(DownloadEvent::ChunkDownloaded {
            hash: chunk.hash.clone(),
            size: chunk.length,
            current,
            total,
        });

        Ok(chunk)
    }

    /// Caches a chunk to disk.
    async fn cache_chunk(&self, chunk: &Chunk, cache_dir: &Path) -> Result<()> {
        let chunk_path = chunk_cache_path(cache_dir, &chunk.hash);

        // Create parent directories if using prefix structure
        if let Some(parent) = chunk_path.parent() {
            fs::create_dir_all(parent).await.with_context(|| {
                format!(
                    "Failed to create chunk cache directory: {}",
                    parent.display()
                )
            })?;
        }

        fs::write(&chunk_path, &chunk.data).await.with_context(|| {
            format!("Failed to cache chunk: {}", chunk_path.display())
        })?;

        Ok(())
    }

    /// Writes a chunk to a file at the specified offset.
    async fn write_chunk_to_file(
        &self,
        chunk: &Chunk,
        offset: u64,
        path: &Path,
    ) -> Result<()> {
        use tokio::io::AsyncSeekExt;

        let mut file = fs::OpenOptions::new()
            .write(true)
            .open(path)
            .await
            .with_context(|| {
                format!("Failed to open file for writing: {}", path.display())
            })?;

        file.seek(std::io::SeekFrom::Start(offset))
            .await
            .with_context(|| {
                format!("Failed to seek to offset {offset} in file")
            })?;

        file.write_all(&chunk.data).await.with_context(|| {
            format!("Failed to write chunk to file at offset {offset}")
        })?;

        file.flush().await.with_context(|| "Failed to flush file")?;

        Ok(())
    }

    /// Downloads all files from a manifest to an output directory.
    ///
    /// Respects feature selections if provided.
    ///
    /// # Arguments
    /// - `manifest`: The release manifest
    /// - `cache_dir`: Directory for caching chunks
    /// - `output_dir`: Base directory for assembled files
    /// - `selected_features`: Optional set of feature IDs to install. If None,
    ///   all files are downloaded.
    ///
    /// # Errors
    /// Returns an error if any file fails to download.
    pub async fn download_all_files(
        &self,
        manifest: &ReleaseManifest,
        cache_dir: &Path,
        output_dir: &Path,
        selected_features: Option<&std::collections::HashSet<String>>,
    ) -> Result<Vec<PathBuf>> {
        let mut downloaded_files = Vec::new();

        for entry in &manifest.files {
            // Skip if feature not selected
            if let Some(features) = selected_features {
                if !features.contains(&entry.feature_id) {
                    continue;
                }
            }

            let output_path = output_dir.join(&entry.path);
            let path = self
                .download_file(entry, cache_dir, &output_path, None)
                .await?;
            downloaded_files.push(path);
        }

        Ok(downloaded_files)
    }
}

fn spawn_chunk_download_task(
    downloader: ChunkDownloader,
    join_set: &mut tokio::task::JoinSet<Result<()>>,
    chunk_info: ChunkInfo,
    cache_dir: PathBuf,
    output_path: PathBuf,
    current: usize,
    total: usize,
    cancel_flag: Option<CancellationFlag>,
) {
    join_set.spawn(async move {
        if is_cancellation_requested(cancel_flag.as_ref()) {
            bail!(INSTALL_CANCELLED_MESSAGE);
        }

        let chunk = downloader
            .get_or_download_chunk(&chunk_info, &cache_dir, current, total)
            .await?;
        downloader
            .write_chunk_to_file(&chunk, chunk_info.offset, &output_path)
            .await
    });
}

/// Computes the cache path for a chunk, using two-level prefix.
///
/// Format: `{cache_dir}/{first-2-chars}/{next-2-chars}/{full-hash}`
fn chunk_cache_path(cache_dir: &Path, hash: &str) -> PathBuf {
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
        cache_dir.join(prefix1).join(prefix2).join(hash)
    } else {
        cache_dir.join(hash)
    }
}

/// Computes the server path for a chunk, using two-level prefix.
///
/// Format: `bh/{first-2-chars}/{next-2-chars}/{full-hash}`
pub fn chunk_server_path(hash: &str) -> PathBuf {
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
        PathBuf::from("bh").join(prefix1).join(prefix2).join(hash)
    } else {
        PathBuf::from("bh").join(hash)
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
    use crate::chunking::chunk_data;
    use crate::manifest::{FileEntry, Platform, ReleaseManifest};
    use crate::signing::{generate_keypair, sign_manifest};
    use chrono::Utc;
    use std::collections::{HashMap, HashSet};
    use std::sync::Arc;
    use tempfile::TempDir;
    use tokio::sync::Barrier;
    use tokio::sync::RwLock;

    #[derive(Clone, Default)]
    struct InMemoryHttpClient {
        routes: Arc<HashMap<String, HttpResponse>>,
    }

    impl InMemoryHttpClient {
        fn new(routes: HashMap<String, HttpResponse>) -> Self {
            Self {
                routes: Arc::new(routes),
            }
        }

        fn path_from_url(url: &str) -> &str {
            // The downloader always uses fully-qualified URLs like
            // "http://host/releases/...". We only care about the path.
            let Some((_, after_scheme)) = url.split_once("://") else {
                return url;
            };
            let Some((_, path_without_slash)) = after_scheme.split_once('/')
            else {
                return "/";
            };
            let path_start = url
                .len()
                .saturating_sub(path_without_slash.len())
                .saturating_sub(1);
            url.get(path_start..).unwrap_or("")
        }
    }

    fn chunk_route_path(hash: &str) -> Result<String> {
        let prefix1 = hash.get(0..2).ok_or_else(|| {
            anyhow::anyhow!("chunk hash missing first prefix")
        })?;
        let prefix2 = hash.get(2..4).ok_or_else(|| {
            anyhow::anyhow!("chunk hash missing second prefix")
        })?;
        Ok(format!("/releases/chunks/{prefix1}/{prefix2}/{hash}"))
    }

    impl HttpClient for InMemoryHttpClient {
        fn get<'a>(
            &'a self,
            url: &'a str,
        ) -> BoxFuture<'a, Result<HttpResponse>> {
            Box::pin(async move {
                let path = Self::path_from_url(url).to_string();
                if let Some(response) = self.routes.get(&path) {
                    return Ok(response.clone());
                }
                Ok(HttpResponse {
                    status: 404,
                    body: Vec::new(),
                })
            })
        }
    }

    #[derive(Clone)]
    struct BarrierHttpClient {
        routes: Arc<HashMap<String, HttpResponse>>,
        blocked_paths: Arc<HashSet<String>>,
        barrier: Arc<Barrier>,
    }

    impl BarrierHttpClient {
        fn new(
            routes: HashMap<String, HttpResponse>,
            blocked_paths: HashSet<String>,
            barrier: Arc<Barrier>,
        ) -> Self {
            Self {
                routes: Arc::new(routes),
                blocked_paths: Arc::new(blocked_paths),
                barrier,
            }
        }
    }

    impl HttpClient for BarrierHttpClient {
        fn get<'a>(
            &'a self,
            url: &'a str,
        ) -> BoxFuture<'a, Result<HttpResponse>> {
            Box::pin(async move {
                let path = InMemoryHttpClient::path_from_url(url).to_string();
                if self.blocked_paths.contains(&path) {
                    self.barrier.wait().await;
                }

                if let Some(response) = self.routes.get(&path) {
                    return Ok(response.clone());
                }

                Ok(HttpResponse {
                    status: 404,
                    body: Vec::new(),
                })
            })
        }
    }

    #[crate::ctb_test]
    fn test_chunk_cache_path() {
        let cache_dir = Path::new("/tmp/cache");
        let hash =
            "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2";

        let path = chunk_cache_path(cache_dir, hash);
        assert_eq!(
            path,
            PathBuf::from(
                "/tmp/cache/a1/b2/a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2"
            )
        );
    }

    #[crate::ctb_test]
    fn test_chunk_server_path() {
        let hash =
            "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2";

        let path = chunk_server_path(hash);
        assert_eq!(
            path,
            PathBuf::from(
                "bh/a1/b2/a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2"
            )
        );
    }

    #[crate::ctb_test]
    fn test_no_progress_callback() {
        // Should not panic
        let callback = no_progress_callback();
        callback(DownloadEvent::ChunkDownloaded {
            hash: "test".to_string(),
            size: 100,
            current: 1,
            total: 1,
        });
    }

    /// Creates an in-memory HTTP client serving chunks and manifests.
    fn create_mock_http_client(
        chunks: HashMap<String, Vec<u8>>,
        manifest: ReleaseManifest,
    ) -> Arc<dyn HttpClient> {
        let mut routes: HashMap<String, HttpResponse> = HashMap::new();

        // The downloader requests: /releases/{platform}/latest.json
        let manifest_json = serde_json::to_string(&manifest).unwrap();
        routes.insert(
            "/releases/linux-x64/latest.json".to_string(),
            HttpResponse {
                status: 200,
                body: manifest_json.into_bytes(),
            },
        );

        // Chunk endpoints: /releases/chunks/{aa}/{bb}/{hash}
        for (hash, data) in chunks {
            let path = format!(
                "/releases/chunks/{}/{}/{}",
                hash.get(0..2).unwrap_or(""),
                hash.get(2..4).unwrap_or(""),
                hash
            );
            routes.insert(
                path,
                HttpResponse {
                    status: 200,
                    body: data,
                },
            );
        }

        Arc::new(InMemoryHttpClient::new(routes))
    }

    #[crate::ctb_test("tokio")]
    async fn test_download_manifest() {
        let (private_key, _public_key) = generate_keypair();

        let mut manifest = ReleaseManifest::new(
            semver::Version::new(1, 0, 0),
            Platform::LinuxX64,
            Utc::now(),
        );
        let sig = sign_manifest(&manifest, &private_key).unwrap();
        manifest.signature = Some(sig);

        let http = create_mock_http_client(HashMap::new(), manifest.clone());
        let downloader = ChunkDownloader::with_http_client(
            "http://ctb.test",
            http,
            no_progress_callback(),
        );
        let downloaded = downloader
            .download_manifest("linux-x64", None)
            .await
            .unwrap();

        assert_eq!(manifest.ctoolbox_version, downloaded.ctoolbox_version);
        assert_eq!(manifest.platform, downloaded.platform);
    }

    #[crate::ctb_test("tokio")]
    async fn test_download_chunk() {
        let chunk_data = b"Test chunk content for download verification";
        let chunk_hash = compute_sha256_hex(chunk_data);
        let chunk_len = u64::try_from(chunk_data.len()).unwrap();

        let mut chunks = HashMap::new();
        chunks.insert(chunk_hash.clone(), chunk_data.to_vec());

        let manifest = ReleaseManifest::new(
            semver::Version::new(1, 0, 0),
            Platform::LinuxX64,
            Utc::now(),
        );

        let http = create_mock_http_client(chunks, manifest);
        let downloader = ChunkDownloader::with_http_client(
            "http://ctb.test",
            http,
            no_progress_callback(),
        );
        let downloaded_chunk = downloader
            .download_chunk(&chunk_hash, chunk_len)
            .await
            .unwrap();

        assert_eq!(downloaded_chunk.hash, chunk_hash);
        assert_eq!(downloaded_chunk.data, chunk_data.to_vec());
        assert_eq!(downloaded_chunk.length, chunk_len);
    }

    #[crate::ctb_test("tokio")]
    async fn test_download_file_assembles_correctly() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join("cache");
        let output_path = temp_dir.path().join("output.bin");

        // Create test data that will produce multiple chunks
        let file_content: Vec<u8> = (0..100_000)
            .map(|i| u8::try_from(i % 256).unwrap_or(0))
            .collect();
        let file_hash = compute_sha256_hex(&file_content);

        // Chunk the content
        let chunked = chunk_data(&file_content).unwrap();

        // Build chunk map for mock server
        let mut chunk_map: HashMap<String, Vec<u8>> = HashMap::new();
        for chunk in &chunked {
            chunk_map.insert(chunk.hash.clone(), chunk.data.clone());
        }

        // Create file entry for the manifest
        let mut file_entry = FileEntry::new(
            "test.bin".to_string(),
            file_hash.clone(),
            "test".to_string(),
        );
        for chunk in &chunked {
            file_entry.add_chunk(chunk.to_chunk_info());
        }
        file_entry.compute_file_size();

        let manifest = ReleaseManifest::new(
            semver::Version::new(1, 0, 0),
            Platform::LinuxX64,
            Utc::now(),
        );

        let http = create_mock_http_client(chunk_map, manifest);

        // Track progress events
        let events = Arc::new(RwLock::new(Vec::new()));
        let events_clone = Arc::clone(&events);
        let progress_callback: ProgressCallback = Arc::new(move |event| {
            let events = Arc::clone(&events_clone);
            // Use tokio's try_write to avoid blocking in the callback
            if let Ok(mut guard) = events.try_write() {
                guard.push(event);
            }
        });

        let downloader = ChunkDownloader::with_http_client(
            "http://ctb.test",
            http,
            progress_callback,
        );
        let result_path = downloader
            .download_file(&file_entry, &cache_dir, &output_path, None)
            .await
            .unwrap();

        // Verify the file was assembled correctly
        let assembled_content = tokio::fs::read(&result_path).await.unwrap();
        assert_eq!(
            file_content, assembled_content,
            "Assembled file should match original content"
        );

        // Verify the hash
        let assembled_hash = compute_sha256_hex(&assembled_content);
        assert_eq!(file_hash, assembled_hash);
    }

    #[crate::ctb_test("tokio")]
    async fn test_download_file_downloads_multiple_chunks_in_parallel() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join("cache");
        let output_path = temp_dir.path().join("parallel-output.bin");

        let first_chunk = vec![1_u8; 32 * 1024];
        let second_chunk = vec![2_u8; 32 * 1024];
        let mut file_content = first_chunk.clone();
        file_content.extend_from_slice(&second_chunk);
        let file_hash = compute_sha256_hex(&file_content);

        let first_hash = compute_sha256_hex(&first_chunk);
        let second_hash = compute_sha256_hex(&second_chunk);

        let mut file_entry = FileEntry::new(
            "parallel.bin".to_string(),
            file_hash,
            "test".to_string(),
        );
        file_entry.add_chunk(ChunkInfo::new(
            first_hash.clone(),
            0,
            u64::try_from(first_chunk.len()).unwrap(),
        ));
        file_entry.add_chunk(ChunkInfo::new(
            second_hash.clone(),
            u64::try_from(first_chunk.len()).unwrap(),
            u64::try_from(second_chunk.len()).unwrap(),
        ));
        file_entry.compute_file_size();

        let first_path = chunk_route_path(&first_hash).unwrap();
        let second_path = chunk_route_path(&second_hash).unwrap();

        let mut routes = HashMap::new();
        routes.insert(
            first_path.clone(),
            HttpResponse {
                status: 200,
                body: first_chunk,
            },
        );
        routes.insert(
            second_path.clone(),
            HttpResponse {
                status: 200,
                body: second_chunk,
            },
        );

        let blocked_paths = HashSet::from([first_path, second_path]);
        let barrier = Arc::new(Barrier::new(2));
        let http =
            Arc::new(BarrierHttpClient::new(routes, blocked_paths, barrier));
        let downloader = ChunkDownloader::with_http_client(
            "http://ctb.test",
            http,
            no_progress_callback(),
        );

        tokio::time::timeout(
            Duration::from_secs(1),
            downloader.download_file(
                &file_entry,
                &cache_dir,
                &output_path,
                None,
            ),
        )
        .await
        .expect("parallel chunk downloads should not deadlock")
        .unwrap();

        let assembled_content = tokio::fs::read(&output_path).await.unwrap();
        assert_eq!(assembled_content, file_content);
    }

    #[crate::ctb_test("tokio")]
    async fn test_download_uses_cache_for_existing_chunks() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join("cache");
        let output_path = temp_dir.path().join("output.bin");

        // Create simple test data
        let file_content =
            b"This is cached content that should be read from cache";
        let file_hash = compute_sha256_hex(file_content);

        let chunked = chunk_data(file_content).unwrap();
        assert_eq!(chunked.len(), 1, "Small file should be a single chunk");

        // Pre-populate cache with the chunk
        let chunk = &chunked[0];
        let cache_path = chunk_cache_path(&cache_dir, &chunk.hash);
        if let Some(parent) = cache_path.parent() {
            tokio::fs::create_dir_all(parent).await.unwrap();
        }
        tokio::fs::write(&cache_path, &chunk.data).await.unwrap();

        // Create file entry
        let mut file_entry = FileEntry::new(
            "cached.bin".to_string(),
            file_hash.clone(),
            "test".to_string(),
        );
        file_entry.add_chunk(chunk.to_chunk_info());
        file_entry.compute_file_size();

        // Create server with NO chunks (to prove cache is used)
        let empty_chunks: HashMap<String, Vec<u8>> = HashMap::new();
        let manifest = ReleaseManifest::new(
            semver::Version::new(1, 0, 0),
            Platform::LinuxX64,
            Utc::now(),
        );

        let http = create_mock_http_client(empty_chunks, manifest);

        // Track if cache was used
        let cache_used = Arc::new(RwLock::new(false));
        let cache_used_clone = Arc::clone(&cache_used);
        let progress_callback: ProgressCallback = Arc::new(move |event| {
            if matches!(event, DownloadEvent::ChunkCached { .. }) {
                if let Ok(mut guard) = cache_used_clone.try_write() {
                    *guard = true;
                }
            }
        });

        let downloader = ChunkDownloader::with_http_client(
            "http://ctb.test",
            http,
            progress_callback,
        );
        let result = downloader
            .download_file(&file_entry, &cache_dir, &output_path, None)
            .await;

        assert!(result.is_ok(), "Should succeed using cached chunk");

        let assembled = tokio::fs::read(&output_path).await.unwrap();
        assert_eq!(file_content.to_vec(), assembled);

        let was_cached = *cache_used.read().await;
        assert!(was_cached, "Should have used cached chunk");
    }

    #[crate::ctb_test("tokio")]
    async fn test_download_invalid_hash_fails() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join("cache");
        let output_path = temp_dir.path().join("output.bin");

        let correct_content = b"Correct content";
        let wrong_content = b"Wrong content - tampered!";
        let correct_hash = compute_sha256_hex(correct_content);

        // Serve wrong content under the correct hash (simulating tampering)
        let mut chunk_map: HashMap<String, Vec<u8>> = HashMap::new();
        chunk_map.insert(correct_hash.clone(), wrong_content.to_vec());

        let mut file_entry = FileEntry::new(
            "tampered.bin".to_string(),
            correct_hash.clone(),
            "test".to_string(),
        );
        file_entry.add_chunk(crate::manifest::ChunkInfo::new(
            correct_hash.clone(),
            0,
            u64::try_from(wrong_content.len()).unwrap(),
        ));

        let manifest = ReleaseManifest::new(
            semver::Version::new(1, 0, 0),
            Platform::LinuxX64,
            Utc::now(),
        );

        let http = create_mock_http_client(chunk_map, manifest);
        let downloader = ChunkDownloader::with_http_client(
            "http://ctb.test",
            http,
            no_progress_callback(),
        );
        let result = downloader
            .download_file(&file_entry, &cache_dir, &output_path, None)
            .await;

        assert!(result.is_err(), "Should fail when chunk hash doesn't match");

        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("hash mismatch")
                || err_msg.contains("length mismatch"),
            "Error should mention hash or length mismatch: {err_msg}"
        );
    }

    #[crate::ctb_test("tokio")]
    async fn test_download_nonexistent_chunk_fails() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join("cache");
        let output_path = temp_dir.path().join("output.bin");

        let fake_hash = "0".repeat(64);

        let mut file_entry = FileEntry::new(
            "missing.bin".to_string(),
            fake_hash.clone(),
            "test".to_string(),
        );
        file_entry
            .add_chunk(crate::manifest::ChunkInfo::new(fake_hash, 0, 1000));

        let manifest = ReleaseManifest::new(
            semver::Version::new(1, 0, 0),
            Platform::LinuxX64,
            Utc::now(),
        );

        // Server has no chunks
        let http = create_mock_http_client(HashMap::new(), manifest);
        let downloader = ChunkDownloader::with_http_client(
            "http://ctb.test",
            http,
            no_progress_callback(),
        );
        let result = downloader
            .download_file(&file_entry, &cache_dir, &output_path, None)
            .await;

        assert!(result.is_err(), "Should fail when chunk doesn't exist");
    }
}
