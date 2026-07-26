//! Streaming tarball generation for offline installers.
//!
//! This module generates tarballs on-the-fly from release manifest chunks,
//! supporting HTTP Range requests for resumable downloads. The tarball contains:
//! - The release manifest JSON
//! - All chunk files referenced by the manifest
//! - A copy of the installer binary (optional)
//!
//! The tarball structure is deterministic, allowing Range requests to resume
//! generation from a specific byte offset by computing where in the tarball
//! we are and regenerating from that point.

#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

use flate2::Compression;
use flate2::write::GzEncoder;
use std::path::Path;
use std::{fs::File, io::Write};

use crate::download::current_platform;
use crate::manifest::{Platform, ReleaseManifest};
use ctb_utilities::pc_settings::{self, PcSettingStrKey};

/// Size of a tar block (512 bytes as per POSIX).
const TAR_BLOCK_SIZE: u64 = 512;

/// Computes the number of padding bytes needed to reach the next block boundary.
fn padding_to_block(size: u64) -> u64 {
    let remainder = size % TAR_BLOCK_SIZE;
    if remainder == 0 {
        0
    } else {
        TAR_BLOCK_SIZE.saturating_sub(remainder)
    }
}

/// A tar entry header for streaming generation.
#[derive(Debug, Clone)]
struct TarHeader {
    /// File path within the archive (limited to 100 chars for basic tar).
    name: String,
    /// File size in bytes.
    size: u64,
    /// File mode (e.g. 0o644 for regular files).
    mode: u32,
    /// Modification time (Unix timestamp).
    mtime: u64,
    /// Whether this is a directory.
    is_dir: bool,
}

impl TarHeader {
    /// Creates a new tar header for a regular file.
    fn file(name: impl Into<String>, size: u64) -> Self {
        Self {
            name: name.into(),
            size,
            mode: 0o644,
            mtime: 0, // Will be set from manifest date
            is_dir: false,
        }
    }

    /// Creates a new tar header for a directory.
    fn directory(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            size: 0,
            mode: 0o755,
            mtime: 0,
            is_dir: true,
        }
    }

    /// Sets the modification time.
    fn with_mtime(mut self, mtime: u64) -> Self {
        self.mtime = mtime;
        self
    }

    /// Sets the file mode (permissions).
    fn with_mode(mut self, mode: u32) -> Self {
        self.mode = mode;
        self
    }

    /// Serializes the header to a 512-byte tar block.
    fn to_bytes(&self) -> [u8; 512] {
        let mut header = [0u8; 512];

        // Name (0-99): Ensure trailing slash for directories
        let name = if self.is_dir && !self.name.ends_with('/') {
            format!("{}/", self.name)
        } else {
            self.name.clone()
        };
        let name_bytes = name.as_bytes();
        let name_len = name_bytes.len().min(100);
        header[0..name_len].copy_from_slice(&name_bytes[..name_len]);

        // Mode (100-107): Octal ASCII with trailing space and null
        let mode_str = format!("{:07o}\0", self.mode);
        header[100..108].copy_from_slice(mode_str.as_bytes());

        // UID (108-115): 0 in octal
        header[108..116].copy_from_slice(b"0000000\0");

        // GID (116-123): 0 in octal
        header[116..124].copy_from_slice(b"0000000\0");

        // Size (124-135): Octal ASCII with trailing space
        let size_str = format!("{:011o} ", self.size);
        header[124..136].copy_from_slice(size_str.as_bytes());

        // Mtime (136-147): Octal ASCII with trailing space
        let mtime_str = format!("{:011o} ", self.mtime);
        header[136..148].copy_from_slice(mtime_str.as_bytes());

        // Checksum placeholder (148-155): 8 spaces for initial calculation
        header[148..156].copy_from_slice(b"        ");

        // Typeflag (156): '5' for directory, '0' for regular file
        header[156] = if self.is_dir { b'5' } else { b'0' };

        // Calculate checksum (sum of all bytes in header, treating checksum as spaces)
        let checksum: u32 = header.iter().map(|&b| u32::from(b)).sum();
        let checksum_str = format!("{checksum:06o}\0 ");
        header[148..156].copy_from_slice(checksum_str.as_bytes());

        header
    }

    /// Returns the total size this entry occupies in the tarball (header +
    /// content + padding).
    fn total_size(&self) -> u64 {
        TAR_BLOCK_SIZE.saturating_add(self.size).saturating_add(padding_to_block(self.size))
    }
}

/// Describes an entry in the streaming tarball.
#[derive(Debug, Clone)]
pub struct TarballEntry {
    /// Tar header for this entry.
    header: TarHeader,
    /// The data source for this entry.
    data_source: TarballDataSource,
    /// Byte offset where this entry starts in the tarball.
    start_offset: u64,
}

/// Data source for a tarball entry.
#[derive(Debug, Clone)]
pub enum TarballDataSource {
    /// Inline data (for small entries like the manifest JSON).
    Inline(Vec<u8>),
    /// Chunk hash - data will be read from the chunks directory.
    Chunk { hash: String },
    /// File on disk (for the installer binary).
    File { path: String },
}

/// A streaming tarball generator that can resume from any byte offset.
///
/// The tarball layout is computed upfront, allowing Range requests to seek
/// to a specific byte position and resume generating from there.
#[derive(Debug, Clone)]
pub struct StreamingTarball {
    /// Precomputed entries with their byte offsets.
    entries: Vec<TarballEntry>,
    /// Total size of the tarball in bytes.
    total_size: u64,
    /// The release manifest (kept for reference).
    manifest: ReleaseManifest,
    /// Path to the chunks directory.
    chunks_dir: String,
}

impl StreamingTarball {
    fn build_with_installer_source(
        manifest: ReleaseManifest,
        chunks_dir: String,
        installer_source: Option<(TarballDataSource, u64)>,
    ) -> Result<Self> {
        let mtime = u64::try_from(manifest.date.timestamp()).unwrap_or(0);

        let mut entries = Vec::new();
        let mut offset: u64 = 0;

        // Root directory entry
        let root_name = format!(
            "ctoolbox-{}-{}",
            manifest.platform, manifest.ctoolbox_version
        );
        let root_header = TarHeader::directory(&root_name).with_mtime(mtime);
        entries.push(TarballEntry {
            start_offset: offset,
            header: root_header.clone(),
            data_source: TarballDataSource::Inline(Vec::new()),
        });
        offset = offset.saturating_add(root_header.total_size());

        // Manifest JSON entry
        let manifest_json = serde_json::to_vec_pretty(&manifest)
            .context("Failed to serialize manifest to JSON")?;
        let manifest_name = format!("{root_name}/manifest.json");
        let manifest_header = TarHeader::file(
            &manifest_name,
            u64::try_from(manifest_json.len())?,
        )
        .with_mtime(mtime);
        entries.push(TarballEntry {
            start_offset: offset,
            header: manifest_header.clone(),
            data_source: TarballDataSource::Inline(manifest_json),
        });
        offset = offset.saturating_add(manifest_header.total_size());

        // Chunks directory entry
        let chunks_subdir_name = format!("{root_name}/chunks");
        let chunks_dir_header =
            TarHeader::directory(&chunks_subdir_name).with_mtime(mtime);
        entries.push(TarballEntry {
            start_offset: offset,
            header: chunks_dir_header.clone(),
            data_source: TarballDataSource::Inline(Vec::new()),
        });
        offset = offset.saturating_add(chunks_dir_header.total_size());

        // Collect unique chunks across all files
        let mut seen_hashes = std::collections::HashSet::new();
        for file in &manifest.files {
            for chunk in &file.chunks {
                if seen_hashes.insert(chunk.hash.clone()) {
                    let chunk_path =
                        compressed_chunk_path(&chunks_dir, &chunk.hash);
                    let chunk_size = std::fs::metadata(&chunk_path)
                        .with_context(|| {
                            format!(
                                "Failed to get metadata for chunk: {}",
                                chunk.hash
                            )
                        })?
                        .len();

                    let prefix1 = chunk.hash.get(0..2).unwrap_or("");
                    let prefix2 = chunk.hash.get(2..4).unwrap_or("");
                    let chunk_name = format!(
                        "{chunks_subdir_name}/{prefix1}/{prefix2}/{}.br",
                        chunk.hash
                    );
                    let chunk_header = TarHeader::file(&chunk_name, chunk_size)
                        .with_mtime(mtime);
                    entries.push(TarballEntry {
                        start_offset: offset,
                        header: chunk_header.clone(),
                        data_source: TarballDataSource::Chunk {
                            hash: chunk.hash.clone(),
                        },
                    });
                    offset = offset.saturating_add(chunk_header.total_size());
                }
            }
        }

        if let Some((data_source, installer_size)) = installer_source {
            let installer_name = format!("{root_name}/ctoolbox-installer");
            let installer_header =
                TarHeader::file(&installer_name, installer_size)
                    .with_mtime(mtime)
                    .with_mode(0o755);
            entries.push(TarballEntry {
                start_offset: offset,
                header: installer_header.clone(),
                data_source,
            });
            offset = offset.saturating_add(installer_header.total_size());
        }

        let total_size = offset.saturating_add(1024);

        Ok(Self {
            entries,
            total_size,
            manifest,
            chunks_dir,
        })
    }

    /// Creates a new streaming tarball generator from a release manifest.
    ///
    /// # Arguments
    /// * `manifest` - The release manifest describing the release
    /// * `chunks_dir` - Path to the bh/ directory containing compressed chunks
    /// * `installer_path` - Optional path to the installer binary to include
    ///
    /// # Returns
    /// A new `StreamingTarball` ready to generate bytes.
    pub fn new(
        manifest: ReleaseManifest,
        chunks_dir: impl Into<String>,
        installer_path: Option<&Path>,
    ) -> Result<Self> {
        let chunks_dir = chunks_dir.into();
        let installer_source = installer_path.and_then(|path| {
            if path.exists() {
                let installer_size = std::fs::metadata(path).ok()?.len();
                Some((
                    TarballDataSource::File {
                        path: path.to_string_lossy().into_owned(),
                    },
                    installer_size,
                ))
            } else {
                None
            }
        });

        Self::build_with_installer_source(
            manifest,
            chunks_dir,
            installer_source,
        )
    }

    /// Creates a new tarball generator using installer bytes that were
    /// assembled from the signed release artifacts.
    pub fn new_with_installer_bytes(
        manifest: ReleaseManifest,
        chunks_dir: impl Into<String>,
        installer_bytes: Option<Vec<u8>>,
    ) -> Result<Self> {
        let installer_source = installer_bytes.map(|bytes| {
            (
                TarballDataSource::Inline(bytes.clone()),
                u64::try_from(bytes.len()).unwrap_or(u64::MAX),
            )
        });
        Self::build_with_installer_source(
            manifest,
            chunks_dir.into(),
            installer_source,
        )
    }

    /// Returns the total size of the tarball in bytes.
    pub fn total_size(&self) -> u64 {
        self.total_size
    }

    /// Returns the platform of the manifest.
    pub fn platform(&self) -> Platform {
        self.manifest.platform
    }

    /// Returns the version string for the tarball filename.
    pub fn version(&self) -> String {
        self.manifest.ctoolbox_version.to_string()
    }

    /// Generates tarball bytes for a given range.
    ///
    /// This is the key method for supporting HTTP Range requests. It computes
    /// which entries fall within the requested range and generates only those
    /// bytes.
    ///
    /// # Arguments
    /// * `start` - Starting byte offset (inclusive)
    /// * `end` - Ending byte offset (exclusive), or None for end of tarball
    ///
    /// # Returns
    /// A vector of bytes for the requested range.
    pub fn generate_range(
        &self,
        start: u64,
        end: Option<u64>,
    ) -> Result<Vec<u8>> {
        let end = end.unwrap_or(self.total_size).min(self.total_size);

        if start >= end {
            return Ok(Vec::new());
        }

        let capacity = usize::try_from(end.saturating_sub(start)).unwrap_or(usize::MAX);
        let mut output = Vec::with_capacity(capacity);
        let mut current_pos = start;

        // Process each entry that overlaps with the requested range
        for entry in &self.entries {
            let entry_end = entry.start_offset.saturating_add(entry.header.total_size());

            // Skip entries entirely before our range
            if entry_end <= start {
                continue;
            }

            // Stop if we've passed the end of the range
            if entry.start_offset >= end {
                break;
            }

            // This entry overlaps with our range - generate its bytes
            let entry_bytes = self.generate_entry_bytes(entry)?;

            // Determine which part of this entry we need
            let entry_start_in_range = start.saturating_sub(entry.start_offset);
            let entry_end_in_range = if entry_end > end {
                end.saturating_sub(entry.start_offset)
            } else {
                entry_end.saturating_sub(entry.start_offset)
            };

            let slice_start = usize::try_from(entry_start_in_range)?;
            let slice_end = usize::try_from(entry_end_in_range)?;

            if slice_start < entry_bytes.len() && slice_end <= entry_bytes.len()
            {
                if let Some(slice) = entry_bytes.get(slice_start..slice_end) {
                    output.extend_from_slice(slice);
                }
                current_pos = entry.start_offset.saturating_add(entry_end_in_range);
            }
        }

        // Add trailing null blocks if we're at the end
        let null_block_start = self.total_size.saturating_sub(1024);
        if current_pos < end && current_pos >= null_block_start {
            let null_start = if current_pos > null_block_start {
                usize::try_from(current_pos.saturating_sub(null_block_start))?
            } else {
                0
            };
            let null_end = usize::try_from(end.saturating_sub(null_block_start).min(1024))?;
            let null_bytes = vec![0u8; null_end.saturating_sub(null_start)];
            output.extend_from_slice(&null_bytes);
        }

        Ok(output)
    }

    /// Generates all bytes for a single tarball entry (header + content +
    /// padding).
    fn generate_entry_bytes(&self, entry: &TarballEntry) -> Result<Vec<u8>> {
        let total_size = usize::try_from(entry.header.total_size())?;
        let mut bytes = Vec::with_capacity(total_size);

        // Write header
        bytes.extend_from_slice(&entry.header.to_bytes());

        // Write content based on data source
        match &entry.data_source {
            TarballDataSource::Inline(data) => {
                bytes.extend_from_slice(data);
            }
            TarballDataSource::Chunk { hash } => {
                let chunk_path = compressed_chunk_path(&self.chunks_dir, hash);
                let data = std::fs::read(&chunk_path)
                    .with_context(|| format!("Failed to read chunk: {hash}"))?;
                bytes.extend_from_slice(&data);
            }
            TarballDataSource::File { path } => {
                let data = std::fs::read(path)
                    .with_context(|| format!("Failed to read file: {path}"))?;
                bytes.extend_from_slice(&data);
            }
        }

        // Add padding to reach block boundary
        let content_size = bytes.len().saturating_sub(512); // Subtract header size
        let padding = padding_to_block(u64::try_from(content_size)?);
        bytes.resize(bytes.len().saturating_add(usize::try_from(padding)?), 0);

        Ok(bytes)
    }

    /// Finds the entry index and relative offset for a given tarball position.
    ///
    /// This is useful for understanding where in the tarball a byte offset
    /// falls.
    pub fn locate_offset(&self, offset: u64) -> Option<(usize, u64)> {
        for (idx, entry) in self.entries.iter().enumerate() {
            let entry_end = entry.start_offset.saturating_add(entry.header.total_size());
            if offset >= entry.start_offset && offset < entry_end {
                return Some((idx, offset.saturating_sub(entry.start_offset)));
            }
        }

        // Check if it's in the trailing null blocks
        let null_start = self.total_size.saturating_sub(1024);
        if offset >= null_start && offset < self.total_size {
            return Some((self.entries.len(), offset.saturating_sub(null_start)));
        }

        None
    }
}

/// Computes the path to a compressed chunk file using the two-level prefix
/// scheme.
fn compressed_chunk_path(chunks_dir: &str, hash: &str) -> String {
    if hash.len() >= 4 {
        let prefix1 = hash.get(0..2).unwrap_or("");
        let prefix2 = hash.get(2..4).unwrap_or("");
        format!("{chunks_dir}/{prefix1}/{prefix2}/{hash}.br")
    } else {
        format!("{chunks_dir}/{hash}.br")
    }
}

/// A streaming iterator that yields tarball chunks for use with Axum's
/// streaming response.
///
/// This allows generating the tarball in chunks without loading everything
/// into memory at once.
pub struct TarballStream {
    /// The underlying tarball generator.
    tarball: StreamingTarball,
    /// Current byte position in the tarball.
    position: u64,
    /// End position for range requests.
    end_position: u64,
    /// Chunk size for streaming (default 64KB).
    chunk_size: usize,
}

impl TarballStream {
    /// Creates a new tarball stream for the full tarball.
    pub fn new(tarball: StreamingTarball) -> Self {
        let total = tarball.total_size();
        Self {
            tarball,
            position: 0,
            end_position: total,
            chunk_size: 64 * 1024,
        }
    }

    /// Creates a new tarball stream for a specific byte range.
    pub fn with_range(tarball: StreamingTarball, start: u64, end: u64) -> Self {
        Self {
            tarball,
            position: start,
            end_position: end,
            chunk_size: 64 * 1024,
        }
    }

    /// Sets the chunk size for streaming.
    #[must_use]
    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size;
        self
    }

    /// Returns the next chunk of bytes, or None if we're done.
    pub fn next_chunk(&mut self) -> Result<Option<Vec<u8>>> {
        if self.position >= self.end_position {
            return Ok(None);
        }

        let chunk_end = self.position.saturating_add(u64::try_from(self.chunk_size)?).min(self.end_position);
        let bytes = self
            .tarball
            .generate_range(self.position, Some(chunk_end))?;

        if bytes.is_empty() {
            return Ok(None);
        }

        self.position = self.position.saturating_add(u64::try_from(bytes.len())?);
        Ok(Some(bytes))
    }

    /// Returns the remaining bytes to be generated.
    pub fn remaining(&self) -> u64 {
        self.end_position.saturating_sub(self.position)
    }

    /// Returns the total size being streamed.
    pub fn total_size(&self) -> u64 {
        let start_offset = if let Some(entry) = self.tarball.entries.first() {
            entry.start_offset
        } else {
            0
        };
        self.end_position.saturating_sub(start_offset)
    }

    /// Returns the starting position.
    pub fn start_position(&self) -> u64 {
        self.position
    }

    /// Returns the end position.
    pub fn end_position(&self) -> u64 {
        self.end_position
    }
}

/// Downloads an offline installer tarball from the configured update server
/// and writes it to a local path.
///
/// If the output path ends with `.gz`, the streamed tarball is gzip-compressed
/// locally while writing.
pub fn download_offline_bundle_to_path(
    output_path: &Path,
    server_url: Option<&str>,
    platform: Option<&str>,
    version: Option<&str>,
) -> Result<()> {
    let server_url = if let Some(server_url) = server_url {
        server_url.to_string()
    } else if let Some(server_url) =
        pc_settings::get_str_setting(PcSettingStrKey::ServerUrl)
    {
        server_url
    } else {
        default_url()
    };
    let server_url = server_url.trim_end_matches('/').to_string();
    let platform = if let Some(platform) = platform {
        platform.to_string()
    } else {
        current_platform()
    };
    let version = version.unwrap_or("latest");
    let url = format!("{server_url}/releases/{platform}/{version}.tar");

    if let Some(parent) = output_path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent).with_context(|| {
            format!("Failed to create output directory {}", parent.display())
        })?;
    }

    let client = https::blocking_client(https::ClientOptions::default())
        .context(
            "Failed to build HTTP client for offline installer download",
        )?;
    let mut response =
        client.get_with_backoff(&url, 10).with_context(|| {
            format!("Failed to download offline installer from {url}")
        })?;

    if !response.is_success() {
        let status = response.status_code();
        let body = response
            .text()
            .unwrap_or_else(|_| String::from("<body unavailable>"));
        bail!("Offline installer download failed: HTTP {status}\n{body}");
    }

    let file = File::create(output_path).with_context(|| {
        format!("Failed to create output file {}", output_path.display())
    })?;

    if output_path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("gz"))
    {
        let mut encoder = GzEncoder::new(file, Compression::default());
        response
            .copy_to(&mut encoder)
            .context("Failed to write gzip-compressed offline installer")?;
        encoder
            .finish()
            .context("Failed to finalize gzip-compressed offline installer")?;
    } else {
        let mut file = file;
        response
            .copy_to(&mut file)
            .context("Failed to write offline installer")?;
        file.flush()
            .context("Failed to flush offline installer output file")?;
    }

    Ok(())
}

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use super::*;
    use chrono::Utc;
    use tempfile::TempDir;

    fn create_test_manifest() -> ReleaseManifest {
        ReleaseManifest {
            format_version: 1,
            ctoolbox_version: semver::Version::new(0, 1, 0),
            platform: Platform::LinuxX64,
            date: Utc::now(),
            signature: None,
            revoked_key_ids: Vec::new(),
            files: vec![],
        }
    }

    #[crate::ctb_test]
    fn test_tar_header_file() {
        let header = TarHeader::file("test.txt", 100);
        let bytes = header.to_bytes();

        // Verify name
        assert_eq!(&bytes[0..8], b"test.txt");

        // Verify typeflag is '0' for regular file
        assert_eq!(bytes[156], b'0');

        // Verify size is encoded
        assert!(!bytes[124..136].iter().all(|&b| b == 0));
    }

    #[crate::ctb_test]
    fn test_tar_header_directory() {
        let header = TarHeader::directory("mydir");
        let bytes = header.to_bytes();

        // Name should have trailing slash
        assert_eq!(&bytes[0..6], b"mydir/");

        // Typeflag should be '5' for directory
        assert_eq!(bytes[156], b'5');

        // Size should be 0
        let size_field = std::str::from_utf8(&bytes[124..136]).unwrap();
        let size_field = size_field.trim_matches('\0').trim();
        let size_digits = size_field.trim_start_matches('0');
        let size_digits = if size_digits.is_empty() {
            "0"
        } else {
            size_digits
        };
        let size = u64::from_str_radix(size_digits, 8).unwrap();
        assert_eq!(size, 0);
    }

    #[crate::ctb_test]
    fn test_padding_to_block() {
        assert_eq!(padding_to_block(0), 0);
        assert_eq!(padding_to_block(512), 0);
        assert_eq!(padding_to_block(1024), 0);
        assert_eq!(padding_to_block(1), 511);
        assert_eq!(padding_to_block(511), 1);
        assert_eq!(padding_to_block(513), 511);
    }

    #[crate::ctb_test]
    fn test_streaming_tarball_empty_manifest() {
        let temp = TempDir::new().unwrap();
        let chunks_dir = temp.path().join("bh");
        std::fs::create_dir_all(&chunks_dir).unwrap();

        let manifest = create_test_manifest();
        let tarball = StreamingTarball::new(
            manifest,
            chunks_dir.to_string_lossy().to_string(),
            None,
        )
        .unwrap();

        // Should have root dir, manifest.json, and chunks dir entries
        assert_eq!(tarball.entries.len(), 3);
        assert!(tarball.total_size() > 0);
    }

    #[crate::ctb_test]
    fn test_generate_range_full() {
        let temp = TempDir::new().unwrap();
        let chunks_dir = temp.path().join("bh");
        std::fs::create_dir_all(&chunks_dir).unwrap();

        let manifest = create_test_manifest();
        let tarball = StreamingTarball::new(
            manifest,
            chunks_dir.to_string_lossy().to_string(),
            None,
        )
        .unwrap();

        let full = tarball.generate_range(0, None).unwrap();
        assert_eq!(u64::try_from(full.len()).unwrap(), tarball.total_size());

        // Verify it ends with 1024 null bytes
        let end = &full[full.len().saturating_sub(1024)..];
        assert!(end.iter().all(|&b| b == 0));
    }

    #[crate::ctb_test]
    fn test_generate_range_partial() {
        let temp = TempDir::new().unwrap();
        let chunks_dir = temp.path().join("bh");
        std::fs::create_dir_all(&chunks_dir).unwrap();

        let manifest = create_test_manifest();
        let tarball = StreamingTarball::new(
            manifest,
            chunks_dir.to_string_lossy().to_string(),
            None,
        )
        .unwrap();

        // Request first 512 bytes (first block)
        let partial = tarball.generate_range(0, Some(512)).unwrap();
        assert_eq!(partial.len(), 512);

        // Should be a valid tar header
        let full = tarball.generate_range(0, None).unwrap();
        assert_eq!(&partial[..], &full[..512]);
    }

    #[crate::ctb_test]
    fn test_tarball_stream() {
        let temp = TempDir::new().unwrap();
        let chunks_dir = temp.path().join("bh");
        std::fs::create_dir_all(&chunks_dir).unwrap();

        let manifest = create_test_manifest();
        let tarball = StreamingTarball::new(
            manifest,
            chunks_dir.to_string_lossy().to_string(),
            None,
        )
        .unwrap();

        let total_size = tarball.total_size();
        let mut stream = TarballStream::new(tarball).with_chunk_size(256);

        let mut collected = Vec::new();
        while let Some(chunk) = stream.next_chunk().unwrap() {
            collected.extend_from_slice(&chunk);
        }

        assert_eq!(u64::try_from(collected.len()).unwrap(), total_size);
    }
}
