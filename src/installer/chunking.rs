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

//! Content-defined chunking for file deduplication in the installer.
//!
//! This module implements file chunking using the `FastCDC` algorithm to split
//! files into variable-sized chunks based on content. This enables efficient
//! delta updates by allowing reuse of identical chunks across file versions.
//!
//! Chunks are stored in compressed (brotli) form on disk to save space, but
//! are served and processed in uncompressed form for deduplication.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use fastcdc::v2020::FastCDC;
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

/// Minimum chunk size in bytes (320KB).
pub const MIN_CHUNK_SIZE: u32 = 32 * 1024 * 10;

/// Average chunk size in bytes (640KB).
pub const AVG_CHUNK_SIZE: u32 = 64 * 1024 * 10;

/// Maximum chunk size in bytes (1280KB).
pub const MAX_CHUNK_SIZE: u32 = 128 * 1024 * 10;

/// A content-defined chunk extracted from a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chunk {
    /// SHA-256 hash of the chunk data (hex-encoded).
    pub hash: String,
    /// Byte offset of this chunk within the original file.
    pub offset: u64,
    /// Length of the chunk in bytes.
    pub length: u64,
    /// The actual chunk data.
    pub data: Vec<u8>,
    /// Compressed size in bytes, if known.
    pub compressed_size: Option<u64>,
}

impl Chunk {
    /// Creates a new chunk from data, computing its hash.
    ///
    /// The offset and length are set based on the provided values.
    pub fn new(data: Vec<u8>, offset: u64) -> Self {
        let hash = compute_sha256_hex(&data);
        // Reason for fallback: chunk data buffer length u64 conversion overflow saturates to u64::MAX
        let length = u64::try_from(data.len()).unwrap_or(u64::MAX);
        Self {
            hash,
            offset,
            length,
            data,
            compressed_size: None,
        }
    }

    /// Creates a chunk info (without data) for use in manifests.
    pub fn to_chunk_info(&self) -> crate::manifest::ChunkInfo {
        crate::manifest::ChunkInfo {
            hash: self.hash.clone(),
            offset: self.offset,
            length: self.length,
            compressed_size: self.compressed_size,
        }
    }
}

/// Computes the SHA-256 hash of data and returns it as a hex string.
pub fn compute_sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    bin2hex(hasher.finalize())
}

/// Computes the SHA-256 hash of a file and returns it as a hex string.
///
/// # Errors
/// Returns an error if the file cannot be read.
pub fn compute_file_sha256_hex(path: &Path) -> Result<String> {
    let mut file = File::open(path).with_context(|| {
        format!("Failed to open file for hashing: {}", path.display())
    })?;

    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 64 * 1024];

    loop {
        let bytes_read = file.read(&mut buffer).with_context(|| {
            format!("Failed to read file for hashing: {}", path.display())
        })?;
        if bytes_read == 0 {
            break;
        }
        let slice = buffer.get(..bytes_read).context(
            "bytes_read returned by std::io::Read exceeded buffer size",
        )?;
        hasher.update(slice);
    }

    Ok(bin2hex(hasher.finalize()))
}

/// Splits a file into content-defined chunks using `FastCDC`.
///
/// Each chunk will have a size between `MIN_CHUNK_SIZE` and `MAX_CHUNK_SIZE`,
/// with an average size around `AVG_CHUNK_SIZE`. The chunking boundaries are
/// determined by content, enabling efficient deduplication.
///
/// # Errors
/// Returns an error if the file cannot be read.
pub fn chunk_file(path: &Path) -> Result<Vec<Chunk>> {
    let data = fs::read(path).with_context(|| {
        format!("Failed to read file for chunking: {}", path.display())
    })?;

    chunk_data(&data)
}

/// Splits data into content-defined chunks using `FastCDC`.
///
/// This is the in-memory variant of `chunk_file`.
pub fn chunk_data(data: &[u8]) -> Result<Vec<Chunk>> {
    let chunker =
        FastCDC::new(data, MIN_CHUNK_SIZE, AVG_CHUNK_SIZE, MAX_CHUNK_SIZE);
    let mut chunks = Vec::new();

    for chunk_result in chunker {
        let offset = u64::try_from(chunk_result.offset)
            .context("Chunk offset exceeds u64 range")?;
        let chunk_data = data
            .get(
                chunk_result.offset
                    ..chunk_result.offset.saturating_add(chunk_result.length),
            )
            .context("Chunk range calculated by FastCDC exceeded data bounds")?
            .to_vec();
        chunks.push(Chunk::new(chunk_data, offset));
    }

    Ok(chunks)
}

/// Applies a chunk to an output file at its specified offset.
///
/// The file will be created if it doesn't exist. If the file exists but is
/// smaller than required, it will be extended. The chunk data is written at
/// the chunk's offset within the file.
///
/// # Errors
/// Returns an error if the file cannot be opened or written to.
pub fn apply_chunk_to_file(chunk: &Chunk, output: &Path) -> Result<()> {
    // Create parent directories if needed
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!("Failed to create parent directory: {}", parent.display())
        })?;
    }

    // Open file for writing, creating if needed
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(output)
        .with_context(|| {
            format!("Failed to open output file: {}", output.display())
        })?;

    // Seek to the chunk's offset
    file.seek(SeekFrom::Start(chunk.offset)).with_context(|| {
        format!(
            "Failed to seek to offset {} in file: {}",
            chunk.offset,
            output.display()
        )
    })?;

    // Write the chunk data
    file.write_all(&chunk.data).with_context(|| {
        format!("Failed to write chunk to file: {}", output.display())
    })?;

    Ok(())
}

/// Verifies that a chunk's hash matches its data.
///
/// Returns `true` if the computed SHA-256 hash of the chunk data matches
/// the stored hash, `false` otherwise.
pub fn verify_chunk(chunk: &Chunk) -> bool {
    let computed_hash = compute_sha256_hex(&chunk.data);
    computed_hash == chunk.hash
}

/// Verifies that a file's hash matches the expected checksum.
///
/// Returns `true` if the computed SHA-256 hash of the file matches the
/// expected hash, `false` otherwise.
///
/// # Note
/// Returns `false` if the file cannot be read (instead of propagating error).
pub fn verify_file(path: &Path, expected_hash: &str) -> bool {
    match compute_file_sha256_hex(path) {
        Ok(computed_hash) => computed_hash == expected_hash,
        Err(_) => false,
    }
}

/// Writes chunks to a directory, naming each file by its hash.
///
/// This is the streaming variant that stores chunks on disk for later use
/// in updates. Chunks are written to `{output_dir}/{hash}` (or with a prefix
/// subdirectory structure if `use_prefix_dirs` is true, e.g. `{output_dir}/a0/cd/{hash}`.
///
/// **Note:** Prefer using `write_chunks_to_directory_compressed` for storage
/// efficiency. This function writes uncompressed chunks.
///
/// # Arguments
/// - `chunks`: The chunks to write
/// - `output_dir`: Directory to write chunks to (uses two-level prefix: ab/cd/hash)
///
/// # Errors
/// Returns an error if directories cannot be created or files cannot be written.
pub fn write_chunks_to_directory(
    chunks: &[Chunk],
    output_dir: &Path,
) -> Result<()> {
    fs::create_dir_all(output_dir).with_context(|| {
        format!("Failed to create chunk directory: {}", output_dir.display())
    })?;

    for chunk in chunks {
        let chunk_path = if chunk.hash.len() >= 4 {
            #[allow(
                clippy::expect_used,
                reason = "chunk.hash.len() >= 4 checked in if condition"
            )]
            let prefix1 = chunk.hash.get(0..2).expect("hash.len() >= 4");
            #[allow(
                clippy::expect_used,
                reason = "chunk.hash.len() >= 4 checked in if condition"
            )]
            let prefix2 = chunk.hash.get(2..4).expect("hash.len() >= 4");
            output_dir.join(prefix1).join(prefix2).join(&chunk.hash)
        } else {
            output_dir.join(&chunk.hash)
        };

        // Create parent directories for prefixed paths
        if let Some(parent) = chunk_path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!(
                    "Failed to create chunk subdirectory: {}",
                    parent.display()
                )
            })?;
        }

        // Skip if chunk already exists (deduplication)
        if chunk_path.exists() {
            continue;
        }

        fs::write(&chunk_path, &chunk.data).with_context(|| {
            format!("Failed to write chunk file: {}", chunk_path.display())
        })?;
    }

    Ok(())
}

/// Writes chunks to a directory in compressed (brotli) form.
///
/// Chunks are stored compressed to save disk space. The hash used for naming
/// is based on the uncompressed data, so deduplication works across
/// uncompressed boundaries.
///
/// Uses two-level prefix subdirectory: `{output_dir}/a0/cd/{hash}.br`
///
/// # Arguments
/// - `chunks`: The chunks to write (uncompressed data)
/// - `output_dir`: Directory to write chunks to
///
/// # Errors
/// Returns an error if directories cannot be created or files cannot be written.
pub fn write_chunks_to_directory_compressed(
    chunks: &mut [Chunk],
    output_dir: &Path,
) -> Result<()> {
    fs::create_dir_all(output_dir).with_context(|| {
        format!("Failed to create chunk directory: {}", output_dir.display())
    })?;

    for chunk in chunks {
        let chunk_path = if chunk.hash.len() >= 4 {
            #[allow(
                clippy::expect_used,
                reason = "chunk.hash.len() >= 4 checked in if condition"
            )]
            let prefix1 = chunk.hash.get(0..2).expect("hash.len() >= 4");
            #[allow(
                clippy::expect_used,
                reason = "chunk.hash.len() >= 4 checked in if condition"
            )]
            let prefix2 = chunk.hash.get(2..4).expect("hash.len() >= 4");
            output_dir
                .join(prefix1)
                .join(prefix2)
                .join(format!("{}.br", &chunk.hash))
        } else {
            output_dir.join(format!("{}.br", &chunk.hash))
        };

        // Create parent directories for prefixed paths
        if let Some(parent) = chunk_path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!(
                    "Failed to create chunk subdirectory: {}",
                    parent.display()
                )
            })?;
        }

        // Compress the chunk data using Brotli
        let mut compressed_data = Vec::new();
        {
            let params = brotli::enc::BrotliEncoderParams {
                quality: 6, // Good balance of speed and compression
                ..Default::default()
            };
            let mut encoder = brotli::CompressorWriter::with_params(
                &mut compressed_data,
                4096, // buffer size
                &params,
            );
            encoder
                .write_all(&chunk.data)
                .context("Failed to write chunk data to brotli encoder")?;
            // CompressorWriter finishes on drop, but we flush to be explicit
            encoder.flush().context("Failed to flush brotli encoder")?;
        }

        // Reason for fallback: compressed chunk data length u64 conversion overflow defaults size to 0
        let compressed_size = u64::try_from(compressed_data.len()).unwrap_or(0);
        chunk.compressed_size = Some(compressed_size);

        // Skip if chunk already exists (deduplication)
        if chunk_path.exists() {
            continue;
        }

        fs::write(&chunk_path, compressed_data).with_context(|| {
            format!(
                "Failed to write compressed chunk file: {}",
                chunk_path.display()
            )
        })?;
    }

    Ok(())
}

/// Reads a chunk from a directory by its hash.
///
/// Uses two-level prefix subdirectory: `{directory}/a0/bc/{hash}`
///
/// # Errors
/// Returns an error if the chunk file cannot be found or read.
pub fn read_chunk_from_directory(
    hash: &str,
    directory: &Path,
    offset: u64,
) -> Result<Chunk> {
    let flat_path = directory.join(hash);
    let nested_path = if hash.len() >= 4 {
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
        directory.join(prefix1).join(prefix2).join(hash)
    } else {
        flat_path.clone()
    };

    let chunk_path = if nested_path.exists() {
        nested_path
    } else if flat_path.exists() {
        flat_path
    } else {
        nested_path
    };

    let data = fs::read(&chunk_path).with_context(|| {
        format!("Failed to read chunk from: {}", chunk_path.display())
    })?;

    let chunk = Chunk {
        hash: hash.to_string(),
        offset,
        length: u64::try_from(data.len())
            .context("Chunk length exceeds u64 range")?,
        data,
        compressed_size: None,
    };

    // Verify the chunk matches its hash
    if !verify_chunk(&chunk) {
        bail!(
            "Chunk hash mismatch: expected {}, data hash is {}",
            hash,
            compute_sha256_hex(&chunk.data)
        );
    }

    Ok(chunk)
}

/// Reads a compressed (brotli) chunk from a directory by its hash.
///
/// Uses two-level prefix subdirectory: `{directory}/a0/bc/{hash}.br`
/// (or flat `{directory}/{hash}.br` as fallback).
///
/// The returned chunk contains the uncompressed data, and the hash and length
/// fields refer to the uncompressed data.
///
/// # Errors
/// Returns an error if the chunk file cannot be found, read, or decompressed.
pub fn read_chunk_from_directory_compressed(
    hash: &str,
    directory: &Path,
    offset: u64,
) -> Result<Chunk> {
    let flat_path = directory.join(format!("{hash}.br"));
    let nested_path = if hash.len() >= 4 {
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
        directory
            .join(prefix1)
            .join(prefix2)
            .join(format!("{hash}.br"))
    } else {
        flat_path.clone()
    };

    let chunk_path = if nested_path.exists() {
        nested_path
    } else if flat_path.exists() {
        flat_path
    } else {
        nested_path
    };

    let compressed_data = fs::read(&chunk_path).with_context(|| {
        format!(
            "Failed to read compressed chunk from: {}",
            chunk_path.display()
        )
    })?;

    // Decompress the data using Brotli
    let mut data = Vec::new();
    {
        let mut decoder = brotli::Decompressor::new(&compressed_data[..], 4096);
        decoder.read_to_end(&mut data).with_context(|| {
            format!("Failed to decompress chunk: {}", chunk_path.display())
        })?;
    }

    let chunk = Chunk {
        hash: hash.to_string(),
        offset,
        length: u64::try_from(data.len())
            .context("Chunk length exceeds u64 range")?,
        data,
        compressed_size: Some(
            // Reason for fallback: compressed chunk data length u64 conversion overflow defaults size to 0
            u64::try_from(compressed_data.len()).unwrap_or(0),
        ),
    };

    // Verify the decompressed chunk matches its hash
    if !verify_chunk(&chunk) {
        bail!(
            "Chunk hash mismatch after decompression: expected {}, data hash is {}",
            hash,
            compute_sha256_hex(&chunk.data)
        );
    }

    Ok(chunk)
}

/// Reads raw compressed chunk bytes from a directory by its hash.
///
/// Unlike `read_chunk_from_directory_compressed`, this returns the raw
/// brotli-compressed bytes without decompression. Used for serving compressed
/// chunks directly to clients.
///
/// Uses two-level prefix subdirectory: `{directory}/a0/bc/{hash}.br`
/// (or flat `{directory}/{hash}.br` as fallback).
///
/// # Errors
/// Returns an error if the chunk file cannot be found or read.
pub fn read_compressed_chunk_bytes(
    hash: &str,
    directory: &Path,
) -> Result<Vec<u8>> {
    let flat_path = directory.join(format!("{hash}.br"));
    let nested_path = if hash.len() >= 4 {
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
        directory
            .join(prefix1)
            .join(prefix2)
            .join(format!("{hash}.br"))
    } else {
        flat_path.clone()
    };

    let chunk_path = if nested_path.exists() {
        nested_path
    } else if flat_path.exists() {
        flat_path
    } else {
        nested_path
    };

    fs::read(&chunk_path).with_context(|| {
        format!(
            "Failed to read compressed chunk from: {}",
            chunk_path.display()
        )
    })
}

/// Reassembles a file from chunks.
///
/// Given a list of chunk info (hash, offset, length) and a directory containing
/// the chunk files, reads each chunk and writes it to the output file at the
/// correct offset.
///
/// # Errors
/// Returns an error if chunks cannot be read or the file cannot be written.
pub fn reassemble_file_from_chunks(
    chunk_infos: &[crate::manifest::ChunkInfo],
    chunk_dir: &Path,
    output_path: &Path,
) -> Result<()> {
    // Remove existing file if present
    if output_path.exists() {
        fs::remove_file(output_path).with_context(|| {
            format!("Failed to remove existing file: {}", output_path.display())
        })?;
    }

    for info in chunk_infos {
        let chunk =
            read_chunk_from_directory(&info.hash, chunk_dir, info.offset)?;
        apply_chunk_to_file(&chunk, output_path)?;
    }

    Ok(())
}

/// Streams a file's chunks to a directory, processing the file in a
/// memory-efficient way.
///
/// Unlike `chunk_file` which loads the entire file into memory, this function
/// processes the file in streaming fashion and writes chunks directly to disk.
/// However, due to `FastCDC`'s requirements, it still needs to buffer data.
/// Uses two-level prefix subdirectory structure: ab/cd/hash
///
/// Returns the list of chunk infos (without data) for use in manifests.
///
/// # Errors
/// Returns an error if the file cannot be read or chunks cannot be written.
pub fn stream_chunk_file_to_directory(
    input_path: &Path,
    output_dir: &Path,
) -> Result<Vec<crate::manifest::ChunkInfo>> {
    // FastCDC requires all data in memory, so we read the file fully
    // In the future, we could implement a streaming CDC algorithm
    let chunks = chunk_file(input_path)?;

    write_chunks_to_directory(&chunks, output_dir)?;

    Ok(chunks.iter().map(Chunk::to_chunk_info).collect())
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
    use std::io::Write;

    use tempfile::TempDir;

    fn create_test_file(
        dir: &Path,
        name: &str,
        content: &[u8],
    ) -> std::path::PathBuf {
        let path = dir.join(name);
        let mut file = File::create(&path).unwrap();
        file.write_all(content).unwrap();
        path
    }

    #[crate::ctb_test]
    fn test_compute_sha256_hex() {
        let data = b"hello world";
        let hash = compute_sha256_hex(data);
        // Known SHA-256 hash of "hello world"
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[crate::ctb_test]
    fn test_chunk_new() {
        let data = vec![1, 2, 3, 4, 5];
        let chunk = Chunk::new(data.clone(), 100);

        assert_eq!(chunk.offset, 100);
        assert_eq!(chunk.length, 5);
        assert_eq!(chunk.data, data);
        assert!(!chunk.hash.is_empty());
        assert!(verify_chunk(&chunk));
    }

    #[crate::ctb_test]
    fn test_chunk_data_small_file() {
        // Small file should produce a single chunk
        let data = vec![42u8; 1000];
        let chunks = chunk_data(&data).unwrap();

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].offset, 0);
        assert_eq!(chunks[0].data, data);
        assert!(verify_chunk(&chunks[0]));
    }

    #[crate::ctb_test]
    fn test_chunk_data_large_file_dedup() -> Result<()> {
        // Large file should produce multiple chunks. Use data that is comfortably larger than the configured chunk sizes.
        let target_len = usize::try_from(MAX_CHUNK_SIZE)
            .unwrap_or(0)
            .saturating_mul(3);
        let data: Vec<u8> = (0..target_len)
            .map(|i: usize| {
                let v = (i32::try_from(i)
                    .unwrap()
                    .wrapping_mul(17)
                    .wrapping_add(31))
                    % 256;
                u8::try_from(v).unwrap()
            })
            .collect();
        let chunks = chunk_data(&data).unwrap();

        // Should have multiple chunks once the input exceeds the configured
        // chunker bounds.
        assert!(
            chunks.len() > 1,
            "Expected multiple chunks, got {}",
            chunks.len()
        );

        // Verify all chunks
        for chunk in &chunks {
            assert!(verify_chunk(chunk));
        }

        // Reassembling should give original data
        let mut reassembled = vec![0u8; data.len()];
        for chunk in &chunks {
            let start = usize::try_from(chunk.offset).unwrap();
            let end = start.saturating_add(chunk.data.len());
            reassembled[start..end].copy_from_slice(&chunk.data);
        }
        assert_eq!(reassembled, data);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_chunk_data_large_file_random() -> Result<()> {
        // Large file should produce multiple chunks
        let target_len = usize::try_from(MAX_CHUNK_SIZE)
            .unwrap_or(0)
            .saturating_mul(3);
        let buf = rand_bytes_fast_insecure(target_len)?;
        let data: Vec<u8> = buf;
        let chunks = chunk_data(&data).unwrap();

        assert!(
            chunks.len() > 1,
            "Expected multiple chunks, got {}",
            chunks.len()
        );
        assert!(
            bail_if_none!(chunks.first()).data.len()
                != bail_if_none!(chunks.get(1)).data.len(),
            "Expected variable chunk sizes, {chunks:?}"
        );
        assert!(
            bail_if_none!(chunks.first()).hash
                != bail_if_none!(chunks.get(1)).hash,
            "The chunks should be different with more random data, {chunks:?}"
        );

        // Verify all chunks
        for chunk in &chunks {
            assert!(verify_chunk(chunk));
        }

        // Reassembling should give original data
        let mut reassembled = vec![0u8; data.len()];
        for chunk in &chunks {
            let start = usize::try_from(chunk.offset).unwrap();
            let end = start.saturating_add(chunk.data.len());
            reassembled[start..end].copy_from_slice(&chunk.data);
        }
        assert_eq!(reassembled, data);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_chunk_file() {
        let temp_dir = TempDir::new().unwrap();
        let data: Vec<u8> = (0..100 * 1024)
            .map(|i| u8::try_from(i % 256).expect("It should fit"))
            .collect();
        let file_path = create_test_file(temp_dir.path(), "test.bin", &data);

        let chunks = chunk_file(&file_path).unwrap();

        // Should have at least one chunk
        assert!(!chunks.is_empty());

        // First chunk should start at offset 0
        assert_eq!(chunks[0].offset, 0);
    }

    #[crate::ctb_test]
    fn test_apply_chunk_to_file() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("output.bin");

        let chunk1 = Chunk::new(vec![1, 2, 3, 4, 5], 0);
        let chunk2 = Chunk::new(vec![6, 7, 8, 9, 10], 5);

        apply_chunk_to_file(&chunk1, &output_path).unwrap();
        apply_chunk_to_file(&chunk2, &output_path).unwrap();

        let result = fs::read(&output_path).unwrap();
        assert_eq!(result, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    }

    #[crate::ctb_test]
    fn test_verify_chunk_valid() {
        let chunk = Chunk::new(vec![1, 2, 3], 0);
        assert!(verify_chunk(&chunk));
    }

    #[crate::ctb_test]
    fn test_verify_chunk_invalid() {
        let mut chunk = Chunk::new(vec![1, 2, 3], 0);
        chunk.data = vec![4, 5, 6]; // Corrupt the data
        assert!(!verify_chunk(&chunk));
    }

    #[crate::ctb_test]
    fn test_verify_file() {
        let temp_dir = TempDir::new().unwrap();
        let data = b"test content for hashing";
        let file_path = create_test_file(temp_dir.path(), "test.txt", data);

        let expected_hash = compute_sha256_hex(data);
        assert!(verify_file(&file_path, &expected_hash));
        assert!(!verify_file(&file_path, "wrong_hash"));
    }

    #[crate::ctb_test]
    fn test_write_and_read_chunks_flat() {
        let temp_dir = TempDir::new().unwrap();
        let chunk_dir = temp_dir.path().join("chunks");

        let chunks =
            vec![Chunk::new(vec![1, 2, 3], 0), Chunk::new(vec![4, 5, 6], 3)];

        write_chunks_to_directory(&chunks, &chunk_dir).unwrap();

        // Verify chunks were written with two-level prefix
        let prefix1 = chunks[0].hash.get(0..2).unwrap_or("");
        let prefix2 = chunks[0].hash.get(2..4).unwrap_or("");
        assert!(
            chunk_dir
                .join(prefix1)
                .join(prefix2)
                .join(&chunks[0].hash)
                .exists()
        );

        let prefix1 = chunks[1].hash.get(0..2).unwrap_or("");
        let prefix2 = chunks[1].hash.get(2..4).unwrap_or("");
        assert!(
            chunk_dir
                .join(prefix1)
                .join(prefix2)
                .join(&chunks[1].hash)
                .exists()
        );

        // Read back and verify
        let read_chunk =
            read_chunk_from_directory(&chunks[0].hash, &chunk_dir, 0).unwrap();
        assert_eq!(read_chunk.data, chunks[0].data);
    }

    #[crate::ctb_test]
    fn test_write_and_read_chunks_with_prefix() {
        let temp_dir = TempDir::new().unwrap();
        let chunk_dir = temp_dir.path().join("chunks");

        let chunks = vec![Chunk::new(vec![1, 2, 3], 0)];

        write_chunks_to_directory(&chunks, &chunk_dir).unwrap();

        // Verify chunk was written with two-level prefix structure
        let prefix1 = chunks[0].hash.get(0..2).unwrap_or("");
        let prefix2 = chunks[0].hash.get(2..4).unwrap_or("");
        let expected_path =
            chunk_dir.join(prefix1).join(prefix2).join(&chunks[0].hash);
        assert!(expected_path.exists());

        // Read back
        let read_chunk =
            read_chunk_from_directory(&chunks[0].hash, &chunk_dir, 0).unwrap();
        assert_eq!(read_chunk.data, chunks[0].data);
    }

    #[crate::ctb_test]
    fn test_reassemble_file_from_chunks() {
        let temp_dir = TempDir::new().unwrap();
        let chunk_dir = temp_dir.path().join("chunks");
        let output_path = temp_dir.path().join("reassembled.bin");

        // Create original data and chunk it
        let original_data: Vec<u8> = (0..100 * 1024)
            .map(|i| u8::try_from(i % 256).expect("It should fit"))
            .collect();
        let chunks = chunk_data(&original_data).unwrap();

        // Write chunks to directory
        write_chunks_to_directory(&chunks, &chunk_dir).unwrap();

        // Get chunk infos
        let chunk_infos: Vec<_> =
            chunks.iter().map(Chunk::to_chunk_info).collect();

        // Reassemble
        reassemble_file_from_chunks(&chunk_infos, &chunk_dir, &output_path)
            .unwrap();

        // Verify result matches original
        let reassembled = fs::read(&output_path).unwrap();
        assert_eq!(reassembled, original_data);
    }

    #[crate::ctb_test]
    fn test_stream_chunk_file_to_directory() {
        let temp_dir = TempDir::new().unwrap();
        let original_data: Vec<u8> = (0..100 * 1024)
            .map(|i| u8::try_from(i % 256).expect("It should fit"))
            .collect();
        let input_path =
            create_test_file(temp_dir.path(), "input.bin", &original_data);
        let chunk_dir = temp_dir.path().join("chunks");

        let chunk_infos =
            stream_chunk_file_to_directory(&input_path, &chunk_dir).unwrap();

        // Should have chunk info
        assert!(!chunk_infos.is_empty());

        // Chunks should exist on disk with two-level prefix
        for info in &chunk_infos {
            let prefix1 = info.hash.get(0..2).unwrap_or("");
            let prefix2 = info.hash.get(2..4).unwrap_or("");
            assert!(
                chunk_dir
                    .join(prefix1)
                    .join(prefix2)
                    .join(&info.hash)
                    .exists()
            );
        }
    }

    #[crate::ctb_test]
    fn test_deduplication() {
        let temp_dir = TempDir::new().unwrap();
        let chunk_dir = temp_dir.path().join("chunks");

        // Same content produces same hash
        let chunk1 = Chunk::new(vec![1, 2, 3], 0);
        let chunk2 = Chunk::new(vec![1, 2, 3], 100); // Different offset, same data

        assert_eq!(chunk1.hash, chunk2.hash);

        // Writing twice should not create duplicate files
        write_chunks_to_directory(&[chunk1.clone()], &chunk_dir).unwrap();
        write_chunks_to_directory(&[chunk2], &chunk_dir).unwrap();

        // Only one file should exist
        let entries: Vec<_> = fs::read_dir(&chunk_dir).unwrap().collect();
        assert_eq!(entries.len(), 1);
    }
}
