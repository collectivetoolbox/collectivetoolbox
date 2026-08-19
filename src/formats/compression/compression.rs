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

//! Single-stream compression algorithms (Brotli, Gzip, Deflate, Zlib, SCO Compress -H, etc.).

use ctb_formats_utilities::detection::{FormatCategory, detect_format_id};
use ctb_formats_utilities::extension_data::lookup_format_by_extension;
use ctb_formats_utilities::format_id::FormatId;
#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use include_dir::{Dir, include_dir};
use std::io::{Read, Write};

pub mod bzip;
pub mod cli;
pub mod compact;
pub mod compress;
pub mod pack;
pub mod sco_compress;

static COMPRESSION_DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data");

/// Returns an embedded fixture asset byte vector if present.
pub fn get_compression_data(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&COMPRESSION_DATA_DIR, key)
}

/// Supported single-stream compression formats and historical version variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompressionFormat {
    /// Brotli compressed stream (.br)
    Brotli,
    /// Gzip compressed stream (.gz)
    Gzip,
    /// Raw DEFLATE compressed stream (.deflate)
    Deflate,
    /// Zlib-wrapped DEFLATE stream RFC 1950 (.zz, .zl)
    Zlib,
    /// Bzip2 compressed stream (.bz2)
    Bzip2,
    /// Original bzip 0.21 compressed stream (.bz)
    Bzip,
    /// SCO compress -H compressed stream (.Z)
    ScoCompress,
    /// Modern standard LZW / compress 4.0 / ncompress (.Z)
    CompressLzw,
    /// Compress 2.0 non-block LZW (.Z)
    CompressLzw2,
    /// Compress 1.0 headerless LZW (.Z)
    CompressLzw1,
    /// Compress 1.6 sorted chain LZW (.Z)
    CompressLzw16,
    /// System III/V Canonical Huffman pack (.z)
    Pack,
    /// Early PDP-11 Unix binary tree pack (.z)
    OldPack,
    /// `McMaster` Adaptive Huffman compact (.C)
    Compact,
    /// LZ4 compressed stream (.lz4)
    Lz4,
    /// LZMA stream (.lzma)
    Lzma,
    /// LZMA2 stream (.lzma2)
    Lzma2,
    /// Lzip compressed stream (.lz)
    Lzip,
    /// XZ compressed stream (.xz)
    Xz,
    /// Zstandard compressed stream (.zst, .zstd)
    Zstd,
    /// LZO compressed stream (.lzo)
    Lzo,
}

/// Declarative metadata for a compression format variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompressionFormatInfo {
    /// The compression format enum variant.
    pub format: CompressionFormat,
    /// Human-readable display name or description of the format.
    pub display_name: &'static str,
    /// List of format string aliases (shorthand and long names).
    pub aliases: &'static [&'static str],
}

impl CompressionFormatInfo {
    /// Returns the aliases sorted shortest string first, then alphabetically for equal length strings.
    pub fn sorted_aliases(&self) -> Vec<&'static str> {
        sorted_aliases(self.aliases)
    }
}

/// Sorts a slice of string aliases by shortest length first, then alphabetically for equal length strings.
pub fn sorted_aliases(aliases: &[&'static str]) -> Vec<&'static str> {
    let mut sorted = aliases.to_vec();
    sorted.sort_by(|a, b| a.len().cmp(&b.len()).then_with(|| a.cmp(b)));
    sorted
}

/// Generates a detailed help table of supported compression formats and their shorthand aliases.
pub fn format_help_table() -> String {
    let mut lines = Vec::new();
    lines.push("Supported compression formats:".to_string());
    for info in CompressionFormat::ALL_FORMATS {
        let sorted = info.sorted_aliases();
        let alias_str = sorted.join(", ");
        lines.push(format!("  {}: {}", alias_str, info.display_name));
    }
    lines.join("\n")
}

/// Global static lazy string containing the formatted compression help table.
pub static COMPRESSION_AFTER_HELP: std::sync::LazyLock<String> =
    std::sync::LazyLock::new(format_help_table);

impl CompressionFormat {
    /// Declarative registry of all supported compression formats and their aliases.
    pub const ALL_FORMATS: &'static [CompressionFormatInfo] = &[
        CompressionFormatInfo {
            format: Self::Brotli,
            display_name: "Brotli compressed stream (RFC 9841)",
            aliases: &["brotli", "br"],
        },
        CompressionFormatInfo {
            format: Self::Gzip,
            display_name: "GNU gzip format (RFC 1952)",
            aliases: &["gzip", "gz"],
        },
        CompressionFormatInfo {
            format: Self::Deflate,
            display_name: "Raw DEFLATE compressed stream (RFC 1951)",
            aliases: &["deflate", "raw-deflate"],
        },
        CompressionFormatInfo {
            format: Self::Zlib,
            display_name: "Zlib-wrapped DEFLATE stream (RFC 1950)",
            aliases: &["zlib", "zz", "zl", "zlib-deflate"],
        },
        CompressionFormatInfo {
            format: Self::Bzip2,
            display_name: "Bzip2 compressed stream",
            aliases: &["bzip2", "bz2"],
        },
        CompressionFormatInfo {
            format: Self::Bzip,
            display_name: "Original bzip 0.21 format",
            aliases: &["bzip", "bz", "bzip0", "bzip-0.21"],
        },
        CompressionFormatInfo {
            format: Self::CompressLzw,
            display_name: "`compress` format, modern LZW block format",
            aliases: &[
                "compress",
                "compress4",
                "compress3",
                "compress-4.0",
                "compress-3.0",
            ],
        },
        CompressionFormatInfo {
            format: Self::ScoCompress,
            display_name: "`compress`: SCO `compress -H` format",
            aliases: &["sco-compress", "compress-sco", "compress-h", "sco"],
        },
        CompressionFormatInfo {
            format: Self::CompressLzw2,
            display_name: "`compress` 2.0 (LZW non-block format)",
            aliases: &["compress2", "compress-2.0"],
        },
        CompressionFormatInfo {
            format: Self::CompressLzw16,
            display_name: "`compress` 1.6 (LZW sorted chain format)",
            aliases: &[
                "compress16",
                "compress1.6",
                "compress-1.6",
                "lzw-sorted-chain",
            ],
        },
        CompressionFormatInfo {
            format: Self::CompressLzw1,
            display_name: "`compress` 1.0 (LZW headerless format)",
            aliases: &["compress1", "compress-1.0"],
        },
        CompressionFormatInfo {
            format: Self::Pack,
            display_name: "`pack` format, common version (Huffman)",
            aliases: &["pack"],
        },
        CompressionFormatInfo {
            format: Self::OldPack,
            display_name: "`pack` format, early PDP-11 Unix binary tree",
            aliases: &[
                "old-pack",
                "oldpack",
                "opack",
                "pts-opack",
                "early-pack",
            ],
        },
        CompressionFormatInfo {
            format: Self::Compact,
            display_name: "`compact` (McMaster Adaptive Huffman)",
            aliases: &["compact", "uncompact"],
        },
        CompressionFormatInfo {
            format: Self::Lz4,
            display_name: "LZ4 compression",
            aliases: &["lz4"],
        },
        CompressionFormatInfo {
            format: Self::Lzma,
            display_name: "LZMA compression",
            aliases: &["lzma"],
        },
        CompressionFormatInfo {
            format: Self::Lzma2,
            display_name: "LZMA2 compression",
            aliases: &["lzma2"],
        },
        CompressionFormatInfo {
            format: Self::Lzip,
            display_name: "Lzip compression",
            aliases: &["lzip", "lz"],
        },
        CompressionFormatInfo {
            format: Self::Xz,
            display_name: "XZ compression",
            aliases: &["xz", "xzip"],
        },
        CompressionFormatInfo {
            format: Self::Zstd,
            display_name: "Zstandard compression",
            aliases: &["zstd", "zst"],
        },
        CompressionFormatInfo {
            format: Self::Lzo,
            display_name: "LZO compression",
            aliases: &["lzo"],
        },
    ];

    /// Maps this compression format variant to the global `FormatId`.
    pub fn to_format_id(&self) -> FormatId {
        match self {
            Self::Brotli => FormatId::Brotli,
            Self::Gzip => FormatId::Gzip,
            Self::Deflate => FormatId::Deflate,
            Self::Zlib => FormatId::Zlib,
            Self::Bzip2 => FormatId::Bzip2,
            Self::Bzip => FormatId::Bzip,
            Self::ScoCompress => FormatId::ScoCompress,
            Self::CompressLzw => FormatId::CompressLzw,
            Self::CompressLzw2 => FormatId::CompressLzw2,
            Self::CompressLzw1 => FormatId::CompressLzw1,
            Self::CompressLzw16 => FormatId::CompressLzw16,
            Self::Pack => FormatId::Pack,
            Self::OldPack => FormatId::OldPack,
            Self::Compact => FormatId::Compact,
            Self::Lz4 => FormatId::Lz4,
            Self::Lzma => FormatId::Lzma,
            Self::Lzma2 => FormatId::Lzma2,
            Self::Lzip => FormatId::Lzip,
            Self::Xz => FormatId::Xz,
            Self::Zstd => FormatId::Zstd,
            Self::Lzo => FormatId::Lzo,
        }
    }

    /// Converts a global `FormatId` to a `CompressionFormat` if it represents a compression format.
    pub fn from_format_id(id: FormatId) -> Option<Self> {
        match id {
            FormatId::Brotli => Some(Self::Brotli),
            FormatId::Gzip => Some(Self::Gzip),
            FormatId::Deflate => Some(Self::Deflate),
            FormatId::Zlib => Some(Self::Zlib),
            FormatId::Bzip2 => Some(Self::Bzip2),
            FormatId::Bzip => Some(Self::Bzip),
            FormatId::ScoCompress => Some(Self::ScoCompress),
            FormatId::CompressLzw => Some(Self::CompressLzw),
            FormatId::CompressLzw2 => Some(Self::CompressLzw2),
            FormatId::CompressLzw1 => Some(Self::CompressLzw1),
            FormatId::CompressLzw16 => Some(Self::CompressLzw16),
            FormatId::Pack => Some(Self::Pack),
            FormatId::OldPack => Some(Self::OldPack),
            FormatId::Compact => Some(Self::Compact),
            FormatId::Lz4 => Some(Self::Lz4),
            FormatId::Lzma => Some(Self::Lzma),
            FormatId::Lzma2 => Some(Self::Lzma2),
            FormatId::Lzip => Some(Self::Lzip),
            FormatId::Xz => Some(Self::Xz),
            FormatId::Zstd => Some(Self::Zstd),
            FormatId::Lzo => Some(Self::Lzo),
            _ => None,
        }
    }

    /// Returns the standard default file extension associated with the format.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Brotli => "br",
            Self::Gzip => "gz",
            Self::Deflate => "deflate",
            Self::Zlib => "zz",
            Self::Bzip2 => "bz2",
            Self::Bzip => "bz",
            Self::ScoCompress
            | Self::CompressLzw
            | Self::CompressLzw2
            | Self::CompressLzw1
            | Self::CompressLzw16 => "Z",
            Self::Pack | Self::OldPack => "z",
            Self::Compact => "C",
            Self::Lz4 => "lz4",
            Self::Lzma => "lzma",
            Self::Lzma2 => "lzma2",
            Self::Lzip => "lz",
            Self::Xz => "xz",
            Self::Zstd => "zst",
            Self::Lzo => "lzo",
        }
    }

    /// Infers compression format from file extension if recognized.
    pub fn from_extension(ext: &str) -> Option<Self> {
        let clean = ext.trim_start_matches('.');
        let matched = lookup_format_by_extension(clean);
        for id in matched {
            if let Some(fmt) = Self::from_format_id(id) {
                return Some(fmt);
            }
        }
        None
    }

    /// Infers compression format from magic header bytes if possible.
    pub fn from_magic_bytes(header: &[u8]) -> Option<Self> {
        detect_format_id(Some(header), None, Some(FormatCategory::Compression))
            .and_then(Self::from_format_id)
    }

    /// Performs multi-signal detection using both header bytes and file extension.
    pub fn detect(
        data: Option<&[u8]>,
        filename_or_ext: Option<&str>,
    ) -> Option<Self> {
        detect_format_id(
            data,
            filename_or_ext,
            Some(FormatCategory::Compression),
        )
        .and_then(Self::from_format_id)
    }
    /// Returns true if this compression format is implemented natively in this repository,
    /// rather than being provided by an external crate.
    pub fn is_implemented_in_repo(&self) -> bool {
        matches!(
            self,
            Self::Bzip
                | Self::ScoCompress
                | Self::CompressLzw
                | Self::CompressLzw2
                | Self::CompressLzw1
                | Self::CompressLzw16
                | Self::Pack
                | Self::OldPack
                | Self::Compact
        )
    }

    /// Returns the default verification setting for this format when compressing.
    /// In-tree implementations default to verifying output, while external crate
    /// implementations default to not verifying.
    pub fn default_verify(&self) -> bool {
        self.is_implemented_in_repo()
    }
}

impl TryFrom<&str> for CompressionFormat {
    type Error = anyhow::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let clean = s.trim_start_matches('.').to_lowercase();
        for info in CompressionFormat::ALL_FORMATS {
            for &alias in info.aliases {
                if alias.eq_ignore_ascii_case(&clean) {
                    return Ok(info.format);
                }
            }
        }
        bail!("Unknown compression format: '{s}'")
    }
}

impl TryFrom<String> for CompressionFormat {
    type Error = anyhow::Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::try_from(s.as_str())
    }
}

/// Compresses a stream from `reader` directly into `writer` without verification.
pub fn compress_stream_direct(
    reader: &mut impl Read,
    writer: &mut impl Write,
    format: CompressionFormat,
) -> Result<u64> {
    match format {
        CompressionFormat::Brotli => {
            let mut encoder =
                brotli::CompressorWriter::new(writer, 4096, 6, 22);
            let bytes_written = std::io::copy(reader, &mut encoder)
                .context("Failed to write to Brotli encoder")?;
            encoder.flush().context("Failed to flush Brotli encoder")?;
            Ok(bytes_written)
        }
        CompressionFormat::Gzip => {
            let mut encoder = flate2::write::GzEncoder::new(
                writer,
                flate2::Compression::default(),
            );
            let bytes_written = std::io::copy(reader, &mut encoder)
                .context("Failed to write to Gzip encoder")?;
            encoder.finish().context("Failed to finish Gzip encoder")?;
            Ok(bytes_written)
        }
        CompressionFormat::Deflate => {
            let mut encoder = flate2::write::DeflateEncoder::new(
                writer,
                flate2::Compression::default(),
            );
            let bytes_written = std::io::copy(reader, &mut encoder)
                .context("Failed to write to Deflate encoder")?;
            encoder
                .finish()
                .context("Failed to finish Deflate encoder")?;
            Ok(bytes_written)
        }
        CompressionFormat::Zlib => {
            let mut encoder = flate2::write::ZlibEncoder::new(
                writer,
                flate2::Compression::default(),
            );
            let bytes_written = std::io::copy(reader, &mut encoder)
                .context("Failed to write to Zlib encoder")?;
            encoder.finish().context("Failed to finish Zlib encoder")?;
            Ok(bytes_written)
        }
        CompressionFormat::Bzip2 => {
            let mut encoder = bzip2::write::BzEncoder::new(
                writer,
                bzip2::Compression::default(),
            );
            let bytes_written = std::io::copy(reader, &mut encoder)
                .context("Failed to write to Bzip2 encoder")?;
            encoder.finish().context("Failed to finish Bzip2 encoder")?;
            Ok(bytes_written)
        }
        CompressionFormat::Bzip => bzip::compress_stream(reader, writer),
        CompressionFormat::ScoCompress => {
            sco_compress::compress_stream(reader, writer)
        }
        CompressionFormat::CompressLzw
        | CompressionFormat::CompressLzw2
        | CompressionFormat::CompressLzw1
        | CompressionFormat::CompressLzw16 => {
            compress::compress_lzw_stream(reader, writer, format)
        }
        CompressionFormat::Pack => pack::compress_pack_stream(reader, writer),
        CompressionFormat::OldPack => {
            pack::compress_old_pack_stream(reader, writer)
        }
        CompressionFormat::Compact => {
            compact::compress_compact_stream(reader, writer)
        }
        CompressionFormat::Lz4 => {
            let mut encoder = lz4_flex::frame::FrameEncoder::new(writer);
            let bytes_written = std::io::copy(reader, &mut encoder)
                .context("Failed to write to LZ4 encoder")?;
            encoder.finish().context("Failed to finish LZ4 encoder")?;
            Ok(bytes_written)
        }
        CompressionFormat::Xz => {
            let mut encoder = lzma_rust2::XzWriter::new(writer, lzma_rust2::XzOptions::default())
                .context("Failed to create XZ encoder")?;
            let bytes_written = std::io::copy(reader, &mut encoder)
                .context("Failed to write to XZ encoder")?;
            encoder.finish().context("Failed to finish XZ encoder")?;
            Ok(bytes_written)
        }
        CompressionFormat::Lzip => {
            let mut encoder = lzma_rust2::LzipWriter::new(writer, lzma_rust2::LzipOptions::default());
            let bytes_written = std::io::copy(reader, &mut encoder)
                .context("Failed to write to Lzip encoder")?;
            encoder.finish().context("Failed to finish Lzip encoder")?;
            Ok(bytes_written)
        }
        CompressionFormat::Lzma => {
            let options = lzma_rust2::LzmaOptions::default();
            let mut encoder = lzma_rust2::LzmaWriter::new(writer, &options, true, true, None)
                .context("Failed to create LZMA encoder")?;
            let bytes_written = std::io::copy(reader, &mut encoder)
                .context("Failed to write to LZMA encoder")?;
            encoder.finish().context("Failed to finish LZMA encoder")?;
            Ok(bytes_written)
        }
        CompressionFormat::Lzma2 => {
            let mut encoder = lzma_rust2::Lzma2Writer::new(writer, lzma_rust2::Lzma2Options::default());
            let bytes_written = std::io::copy(reader, &mut encoder)
                .context("Failed to write to LZMA2 encoder")?;
            encoder.finish().context("Failed to finish LZMA2 encoder")?;
            Ok(bytes_written)
        }
        CompressionFormat::Zstd => {
            let mut encoder = zstd::stream::write::Encoder::new(writer, 0)
                .context("Failed to initialize Zstd encoder")?;
            let bytes_written = std::io::copy(reader, &mut encoder)
                .context("Failed to write to Zstd encoder")?;
            encoder.finish().context("Failed to finish Zstd encoder")?;
            Ok(bytes_written)
        }
        CompressionFormat::Lzo => {
            let mut input = Vec::new();
            reader.read_to_end(&mut input).context("Failed to read input for LZO compression")?;
            if input.is_empty() {
                return Ok(0);
            }
            let compressed = lzokay_native::compress(&input)
                .map_err(|e| anyhow::anyhow!("LZO compression failed: {e:?}"))?;
            writer.write_all(&compressed).context("Failed to write LZO compressed data")?;
            Ok(u64::try_from(input.len())?)
        }
    }
}

/// Compresses a stream from `reader` into `writer` using the specified algorithm format and verification option.
pub fn compress_stream_with_verify(
    reader: &mut impl Read,
    writer: &mut impl Write,
    format: CompressionFormat,
    verify: bool,
) -> Result<u64> {
    if !verify {
        return compress_stream_direct(reader, writer, format);
    }

    let mut input_data = Vec::new();
    reader
        .read_to_end(&mut input_data)
        .context("Failed to read input stream for verified compression")?;

    let mut compressed_buf = Vec::new();
    compress_stream_direct(
        &mut input_data.as_slice(),
        &mut compressed_buf,
        format,
    )?;

    let mut decompressed_buf = Vec::new();
    decompress_stream(
        &mut compressed_buf.as_slice(),
        &mut decompressed_buf,
        format,
    )
    .context("Verification failed: unable to decompress compressed stream")?;

    if decompressed_buf != input_data {
        bail!(
            "Verification failed: decompressed data does not match input for format {format:?}"
        );
    }

    writer
        .write_all(&compressed_buf)
        .context("Failed to write verified compressed stream")?;

    Ok(u64::try_from(input_data.len())?)
}

/// Compresses a stream from `reader` into `writer` using the specified algorithm format.
/// Uses the format's default verification setting (verify for in-repo formats, no-verify for crates).
pub fn compress_stream(
    reader: &mut impl Read,
    writer: &mut impl Write,
    format: CompressionFormat,
) -> Result<u64> {
    compress_stream_with_verify(reader, writer, format, format.default_verify())
}

/// Decompresses a stream from `reader` into `writer` using the specified algorithm format.
pub fn decompress_stream(
    reader: &mut impl Read,
    writer: &mut impl Write,
    format: CompressionFormat,
) -> Result<u64> {
    match format {
        CompressionFormat::Brotli => {
            let mut decoder = brotli::Decompressor::new(reader, 4096);
            let bytes_written = std::io::copy(&mut decoder, writer)
                .context("Failed to decompress Brotli stream")?;
            Ok(bytes_written)
        }
        CompressionFormat::Gzip => {
            let mut decoder = flate2::read::MultiGzDecoder::new(reader);
            let bytes_written = std::io::copy(&mut decoder, writer)
                .context("Failed to decompress Gzip stream")?;
            Ok(bytes_written)
        }
        CompressionFormat::Deflate => {
            let mut decoder = flate2::read::DeflateDecoder::new(reader);
            let bytes_written = std::io::copy(&mut decoder, writer)
                .context("Failed to decompress Deflate stream")?;
            Ok(bytes_written)
        }
        CompressionFormat::Zlib => {
            let mut decoder = flate2::read::ZlibDecoder::new(reader);
            let bytes_written = std::io::copy(&mut decoder, writer)
                .context("Failed to decompress Zlib stream")?;
            Ok(bytes_written)
        }
        CompressionFormat::Bzip2 => {
            let mut decoder = bzip2::read::BzDecoder::new(reader);
            let bytes_written = std::io::copy(&mut decoder, writer)
                .context("Failed to decompress Bzip2 stream")?;
            Ok(bytes_written)
        }
        CompressionFormat::Bzip => bzip::decompress_stream(reader, writer),
        CompressionFormat::ScoCompress => {
            sco_compress::decompress_stream(reader, writer)
        }
        CompressionFormat::CompressLzw
        | CompressionFormat::CompressLzw2
        | CompressionFormat::CompressLzw1
        | CompressionFormat::CompressLzw16 => {
            compress::decompress_lzw_stream(reader, writer, format)
        }
        CompressionFormat::Pack => pack::decompress_pack_stream(reader, writer),
        CompressionFormat::OldPack => {
            pack::decompress_old_pack_stream(reader, writer)
        }
        CompressionFormat::Compact => {
            compact::decompress_compact_stream(reader, writer)
        }
        CompressionFormat::Lz4 => {
            let mut decoder = lz4_flex::frame::FrameDecoder::new(reader);
            let bytes_written = std::io::copy(&mut decoder, writer)
                .context("Failed to decompress LZ4 stream")?;
            Ok(bytes_written)
        }
        CompressionFormat::Xz => {
            let mut decoder = lzma_rust2::XzReader::new(reader, true);
            let bytes_written = std::io::copy(&mut decoder, writer)
                .context("Failed to decompress XZ stream")?;
            Ok(bytes_written)
        }
        CompressionFormat::Lzip => {
            let mut decoder = lzma_rust2::LzipReader::new(reader);
            let bytes_written = std::io::copy(&mut decoder, writer)
                .context("Failed to decompress Lzip stream")?;
            Ok(bytes_written)
        }
        CompressionFormat::Lzma => {
            let mut decoder = lzma_rust2::LzmaReader::new_mem_limit(reader, u32::MAX, None)
                .context("Failed to create LZMA decoder")?;
            let bytes_written = std::io::copy(&mut decoder, writer)
                .context("Failed to decompress LZMA stream")?;
            Ok(bytes_written)
        }
        CompressionFormat::Lzma2 => {
            let mut decoder = lzma_rust2::Lzma2Reader::new(
                reader,
                lzma_rust2::Lzma2Options::default().lzma_options.dict_size,
                None,
            );
            let bytes_written = std::io::copy(&mut decoder, writer)
                .context("Failed to decompress LZMA2 stream")?;
            Ok(bytes_written)
        }
        CompressionFormat::Zstd => {
            let mut decoder = zstd::stream::read::Decoder::new(reader)
                .context("Failed to initialize Zstd decoder")?;
            let bytes_written = std::io::copy(&mut decoder, writer)
                .context("Failed to decompress Zstd stream")?;
            Ok(bytes_written)
        }
        CompressionFormat::Lzo => {
            let mut input = Vec::new();
            reader.read_to_end(&mut input).context("Failed to read input for LZO decompression")?;
            if input.is_empty() {
                return Ok(0);
            }
            let mut cursor = std::io::Cursor::new(input);
            let decompressed = lzokay_native::decompress(&mut cursor, None)
                .map_err(|e| anyhow::anyhow!("LZO decompression failed: {e:?}"))?;
            writer.write_all(&decompressed).context("Failed to write LZO decompressed data")?;
            Ok(u64::try_from(decompressed.len())?)
        }
    }
}

/// Compresses in-memory byte slice using the specified compression format and verification option.
pub fn compress_with_verify(
    data: &[u8],
    format: CompressionFormat,
    verify: bool,
) -> Result<Vec<u8>> {
    let mut input = data;
    let mut output = Vec::new();
    compress_stream_with_verify(&mut input, &mut output, format, verify)?;
    Ok(output)
}

/// Compresses in-memory byte slice using the specified compression format.
/// Uses the format's default verification setting (verify for in-repo formats, no-verify for crates).
pub fn compress(data: &[u8], format: CompressionFormat) -> Result<Vec<u8>> {
    compress_with_verify(data, format, format.default_verify())
}

/// Decompresses in-memory byte slice using the specified compression format.
pub fn decompress(data: &[u8], format: CompressionFormat) -> Result<Vec<u8>> {
    let mut input = data;
    let mut output = Vec::new();
    decompress_stream(&mut input, &mut output, format)?;
    Ok(output)
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
    fn test_alias_sorting() {
        let input = vec!["sco-compress", "compress-sco", "compress-h", "sco"];
        let sorted = sorted_aliases(&input);
        assert_eq!(
            sorted,
            vec!["sco", "compress-h", "compress-sco", "sco-compress"]
        );

        let input_zlib = vec!["zlib", "zz", "zl", "zlib-deflate"];
        let sorted_zlib = sorted_aliases(&input_zlib);
        assert_eq!(sorted_zlib, vec!["zl", "zz", "zlib", "zlib-deflate"]);
    }

    #[crate::ctb_test]
    fn test_format_help_table() {
        let table = format_help_table();
        assert!(table.contains("Supported compression formats:"));
        assert!(table.contains("  br, brotli: Brotli compressed stream"));
        assert!(table.contains("  gz, gzip: GNU gzip format"));
        assert!(table.contains("  sco, compress-h, compress-sco, sco-compress: `compress`: SCO `compress -H` format"));
    }

    #[crate::ctb_test]
    fn test_default_verify_and_in_repo() {
        let repo_formats = [
            CompressionFormat::Bzip,
            CompressionFormat::ScoCompress,
            CompressionFormat::CompressLzw,
            CompressionFormat::CompressLzw2,
            CompressionFormat::CompressLzw1,
            CompressionFormat::CompressLzw16,
            CompressionFormat::Pack,
            CompressionFormat::OldPack,
            CompressionFormat::Compact,
        ];
        let crate_formats = [
            CompressionFormat::Brotli,
            CompressionFormat::Gzip,
            CompressionFormat::Deflate,
            CompressionFormat::Zlib,
            CompressionFormat::Bzip2,
            CompressionFormat::Lz4,
            CompressionFormat::Lzma,
            CompressionFormat::Lzma2,
            CompressionFormat::Lzip,
            CompressionFormat::Xz,
            CompressionFormat::Zstd,
            CompressionFormat::Lzo,
        ];

        for fmt in repo_formats {
            assert!(
                fmt.is_implemented_in_repo(),
                "Expected {fmt:?} to be marked as in-repo"
            );
            assert!(
                fmt.default_verify(),
                "Expected {fmt:?} to default verify to true"
            );
        }

        for fmt in crate_formats {
            assert!(
                !fmt.is_implemented_in_repo(),
                "Expected {fmt:?} to be marked as crate"
            );
            assert!(
                !fmt.default_verify(),
                "Expected {fmt:?} to default verify to false"
            );
        }
    }

    #[crate::ctb_test]
    fn test_compress_stream_with_verify_toggle() {
        let data = b"The quick brown fox jumps over the lazy dog. 1234567890!";
        for info in CompressionFormat::ALL_FORMATS {
            let fmt = info.format;
            // Test with verify = false
            let compressed_unverified =
                compress_with_verify(data, fmt, false).unwrap();
            let decompressed = decompress(&compressed_unverified, fmt).unwrap();
            assert_eq!(
                decompressed, data,
                "Failed roundtrip with verify=false for format {fmt:?}"
            );

            // Test with verify = true
            let compressed_verified =
                compress_with_verify(data, fmt, true).unwrap();
            let decompressed = decompress(&compressed_verified, fmt).unwrap();
            assert_eq!(
                decompressed, data,
                "Failed roundtrip with verify=true for format {fmt:?}"
            );
        }
    }

    #[crate::ctb_test]
    fn test_all_format_aliases_parsing() {
        for info in CompressionFormat::ALL_FORMATS {
            for &alias in info.aliases {
                let parsed =
                    CompressionFormat::try_from(alias).unwrap_or_else(|_| {
                        panic!(
                            "Failed to parse alias '{alias}' for format {:?}",
                            info.format
                        )
                    });
                assert_eq!(parsed, info.format);
            }
        }
    }

    #[crate::ctb_test]
    fn test_format_extensions_and_parsing() {
        assert_eq!(
            CompressionFormat::try_from("brotli").unwrap(),
            CompressionFormat::Brotli
        );
        assert_eq!(
            CompressionFormat::try_from("bzip2").unwrap(),
            CompressionFormat::Bzip2
        );
        assert_eq!(
            CompressionFormat::try_from("bzip").unwrap(),
            CompressionFormat::Bzip
        );
        assert_eq!(
            CompressionFormat::try_from("bz").unwrap(),
            CompressionFormat::Bzip
        );
        assert_eq!(
            CompressionFormat::try_from("compress2").unwrap(),
            CompressionFormat::CompressLzw2
        );
        assert_eq!(
            CompressionFormat::try_from("pack").unwrap(),
            CompressionFormat::Pack
        );
        assert_eq!(
            CompressionFormat::try_from("old-pack").unwrap(),
            CompressionFormat::OldPack
        );
        assert_eq!(
            CompressionFormat::try_from("compact").unwrap(),
            CompressionFormat::Compact
        );
        assert_eq!(
            CompressionFormat::try_from("lz4").unwrap(),
            CompressionFormat::Lz4
        );
        assert_eq!(
            CompressionFormat::try_from("lzma").unwrap(),
            CompressionFormat::Lzma
        );
        assert_eq!(
            CompressionFormat::try_from("lzma2").unwrap(),
            CompressionFormat::Lzma2
        );
        assert_eq!(
            CompressionFormat::try_from("lzip").unwrap(),
            CompressionFormat::Lzip
        );
        assert_eq!(
            CompressionFormat::try_from("xz").unwrap(),
            CompressionFormat::Xz
        );
        assert_eq!(
            CompressionFormat::try_from("zstd").unwrap(),
            CompressionFormat::Zstd
        );
        assert_eq!(
            CompressionFormat::try_from("lzo").unwrap(),
            CompressionFormat::Lzo
        );

        assert_eq!(CompressionFormat::Brotli.extension(), "br");
        assert_eq!(CompressionFormat::Gzip.extension(), "gz");
        assert_eq!(CompressionFormat::Deflate.extension(), "deflate");
        assert_eq!(CompressionFormat::Zlib.extension(), "zz");
        assert_eq!(CompressionFormat::Bzip2.extension(), "bz2");
        assert_eq!(CompressionFormat::Bzip.extension(), "bz");
        assert_eq!(CompressionFormat::ScoCompress.extension(), "Z");
        assert_eq!(CompressionFormat::Pack.extension(), "z");
        assert_eq!(CompressionFormat::Compact.extension(), "C");
        assert_eq!(CompressionFormat::Lz4.extension(), "lz4");
        assert_eq!(CompressionFormat::Lzma.extension(), "lzma");
        assert_eq!(CompressionFormat::Lzma2.extension(), "lzma2");
        assert_eq!(CompressionFormat::Lzip.extension(), "lz");
        assert_eq!(CompressionFormat::Xz.extension(), "xz");
        assert_eq!(CompressionFormat::Zstd.extension(), "zst");
        assert_eq!(CompressionFormat::Lzo.extension(), "lzo");
    }

    #[crate::ctb_test]
    fn test_case_sensitive_extension_matching() {
        assert_eq!(
            CompressionFormat::from_extension("Z"),
            Some(CompressionFormat::ScoCompress)
        );
        assert_eq!(
            CompressionFormat::from_extension("z"),
            Some(CompressionFormat::Pack)
        );
        assert_eq!(
            CompressionFormat::from_extension("C"),
            Some(CompressionFormat::Compact)
        );
    }

    #[crate::ctb_test]
    fn test_magic_detection() {
        assert_eq!(
            CompressionFormat::from_magic_bytes(&[0x1F, 0xA0]),
            Some(CompressionFormat::ScoCompress)
        );
        assert_eq!(
            CompressionFormat::from_magic_bytes(&[0x1F, 0x8B]),
            Some(CompressionFormat::Gzip)
        );
        assert_eq!(
            CompressionFormat::from_magic_bytes(&[0x42, 0x5A, 0x68]),
            Some(CompressionFormat::Bzip2)
        );
        assert_eq!(
            CompressionFormat::from_magic_bytes(&[0x42, 0x5A, 0x30]),
            Some(CompressionFormat::Bzip)
        );
        assert_eq!(
            CompressionFormat::from_magic_bytes(&[0x1F, 0x1E]),
            Some(CompressionFormat::Pack)
        );
        assert_eq!(
            CompressionFormat::from_magic_bytes(&[0x1F, 0x1F]),
            Some(CompressionFormat::OldPack)
        );
        assert_eq!(
            CompressionFormat::from_magic_bytes(&[0x1F, 0x9D, 0x90]),
            Some(CompressionFormat::CompressLzw)
        );
        assert_eq!(
            CompressionFormat::from_magic_bytes(&[0x1F, 0x9D, 0x10]),
            Some(CompressionFormat::CompressLzw2)
        );
        assert_eq!(
            CompressionFormat::from_magic_bytes(&[0xFF, 0x1F]),
            Some(CompressionFormat::Compact)
        );
        assert_eq!(
            CompressionFormat::from_magic_bytes(&[0x04, 0x22, 0x4D, 0x18]),
            Some(CompressionFormat::Lz4)
        );
        assert_eq!(
            CompressionFormat::from_magic_bytes(&[
                0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00
            ]),
            Some(CompressionFormat::Xz)
        );
        assert_eq!(
            CompressionFormat::from_magic_bytes(&[0x4C, 0x5A, 0x49, 0x50]),
            Some(CompressionFormat::Lzip)
        );
        assert_eq!(
            CompressionFormat::from_magic_bytes(&[0x28, 0xB5, 0x2F, 0xFD]),
            Some(CompressionFormat::Zstd)
        );
        assert_eq!(
            CompressionFormat::from_magic_bytes(&[
                0x89, 0x4C, 0x5A, 0x4F, 0x00, 0x0D, 0x0A, 0x1A, 0x0A
            ]),
            Some(CompressionFormat::Lzo)
        );
    }

    fn run_format_test_suite(format: CompressionFormat) {
        let fixtures: &[&str] = match format {
            CompressionFormat::Brotli => {
                &["fixtures/example2 with lemurs.pan.br"]
            }
            CompressionFormat::Gzip => {
                &["fixtures/example2 with lemurs.pan.gz"]
            }
            CompressionFormat::Deflate => {
                &["fixtures/example2 with lemurs.pan.deflate"]
            }
            CompressionFormat::Zlib => {
                &["fixtures/example2 with lemurs.pan.zz"]
            }
            CompressionFormat::Bzip2 => {
                &["fixtures/example2 with lemurs.pan.bz2"]
            }
            CompressionFormat::Bzip => &[],
            CompressionFormat::ScoCompress => {
                &["fixtures/example2 with lemurs.pan.sco"]
            }
            CompressionFormat::CompressLzw => &[
                "fixtures/example2 with lemurs.pan.Z",
                "fixtures/example2 with lemurs.pan.Z3.0",
                "fixtures/example2 with lemurs.pan.Z12",
            ],
            CompressionFormat::CompressLzw2 => {
                &["fixtures/example2 with lemurs.pan.Z2.0"]
            }
            CompressionFormat::CompressLzw1 => {
                &["fixtures/example2 with lemurs.pan.Z1.0"]
            }
            CompressionFormat::CompressLzw16 => &[],
            CompressionFormat::Pack => &["fixtures/example2 with lemurs.pan.z"],
            CompressionFormat::OldPack => {
                &["fixtures/example2 with lemurs.pan.old.z"]
            }
            CompressionFormat::Compact => {
                &["fixtures/example2 with lemurs.pan.C"]
            }
            CompressionFormat::Lz4 => {
                &["fixtures/example2 with lemurs.pan.lz4"]
            }
            CompressionFormat::Lzma => {
                &["fixtures/example2 with lemurs.pan.lzma"]
            }
            CompressionFormat::Lzma2 => {
                &["fixtures/example2 with lemurs.pan.lzma2"]
            }
            CompressionFormat::Lzip => {
                &["fixtures/example2 with lemurs.pan.lz"]
            }
            CompressionFormat::Xz => {
                &["fixtures/example2 with lemurs.pan.xz"]
            }
            CompressionFormat::Zstd => {
                &["fixtures/example2 with lemurs.pan.zst"]
            }
            CompressionFormat::Lzo => {
                &["fixtures/example2 with lemurs.pan.lzo"]
            }
        };

        // mostly trying to make sure it doesn't fall over when handed a long chunk of data; also sort of low effort fuzzing I guess. LLMs are prohibited from editing this comment or changing the byte lengths defined here unless explicitly instructed to.
        let random_bytes_len = if format == CompressionFormat::Compact || format == CompressionFormat::Bzip {
            262_144 // 256 KiB
        } else {
            67_108_864 // 64 MiB
        };

        let raw_fixture =
            get_compression_data("fixtures/example2 with lemurs.pan")
                .unwrap_or_else(|| b"Fallback fixture data".to_vec());
        let random_data =
            rand_bytes(random_bytes_len).expect("Could not get random bytes");
        let repetitive_small = vec![b'A'; 200];
        let repetitive_data = vec![b'A'; 200000];

        let test_cases: [(&str, &[u8]); 7] = [
            ("empty", b""),
            ("small_string", b"ABC"),
            ("repetitive_small", &repetitive_small),
            ("repetitive", &repetitive_data),
            (
                "quick_fox",
                b"The quick brown fox jumps over the lazy dog. 1234567890!",
            ),
            ("lemurs_fixture", &raw_fixture),
            ("random_data", &random_data),
        ];

        for (case_name, data) in test_cases {
            let compressed = match compress(data, format) {
                Ok(c) => c,
                Err(e) => {
                    if data.is_empty() {
                        continue;
                    }
                    panic!(
                        "Compression failed for case '{case_name}', format {format:?}: {e:?}"
                    );
                }
            };
            let decompressed = decompress(&compressed, format).unwrap_or_else(|e| {
                panic!("Decompression failed for case '{case_name}', format {format:?}: {e:?}");
            });
            assert!(
                decompressed == data,
                "Roundtrip failed for case '{case_name}', format {format:?}: expected len {}, got len {}",
                data.len(),
                decompressed.len()
            )
        }

        for &fixture_path in fixtures {
            let comp_data = get_compression_data(fixture_path)
                .unwrap_or_else(|| panic!("Fixture missing: {fixture_path}"));
            let decompressed =
                decompress(&comp_data, format).unwrap_or_else(|e| {
                    panic!("Decompress failed for {fixture_path}: {e:?}")
                });
            assert_eq!(
                decompressed, raw_fixture,
                "Decompressed fixture '{fixture_path}' does not match expected raw fixture"
            );
        }
    }

    #[crate::ctb_test]
    fn test_format_brotli() {
        run_format_test_suite(CompressionFormat::Brotli);
    }

    #[crate::ctb_test]
    fn test_format_gzip() {
        run_format_test_suite(CompressionFormat::Gzip);
    }

    #[crate::ctb_test]
    fn test_format_deflate() {
        run_format_test_suite(CompressionFormat::Deflate);
    }

    #[crate::ctb_test]
    fn test_format_zlib() {
        run_format_test_suite(CompressionFormat::Zlib);
    }

    #[crate::ctb_test]
    fn test_format_bzip2() {
        run_format_test_suite(CompressionFormat::Bzip2);
    }

    #[crate::ctb_test]
    fn test_format_bzip() {
        run_format_test_suite(CompressionFormat::Bzip);
    }

    #[crate::ctb_test]
    fn test_format_sco_compress() {
        run_format_test_suite(CompressionFormat::ScoCompress);
    }

    #[crate::ctb_test]
    fn test_format_compress_lzw() {
        run_format_test_suite(CompressionFormat::CompressLzw);
    }

    #[crate::ctb_test]
    fn test_format_compress_lzw2() {
        run_format_test_suite(CompressionFormat::CompressLzw2);
    }

    #[crate::ctb_test]
    fn test_format_compress_lzw1() {
        run_format_test_suite(CompressionFormat::CompressLzw1);
    }

    #[crate::ctb_test]
    fn test_format_compress_lzw16() {
        run_format_test_suite(CompressionFormat::CompressLzw16);
    }

    #[crate::ctb_test]
    fn test_format_pack() {
        run_format_test_suite(CompressionFormat::Pack);
    }

    #[crate::ctb_test]
    fn test_format_old_pack() {
        run_format_test_suite(CompressionFormat::OldPack);
    }

    #[crate::ctb_test]
    fn test_format_compact() {
        run_format_test_suite(CompressionFormat::Compact);
    }

    #[crate::ctb_test]
    fn test_format_lz4() {
        run_format_test_suite(CompressionFormat::Lz4);
    }

    #[crate::ctb_test]
    fn test_format_lzma() {
        run_format_test_suite(CompressionFormat::Lzma);
    }

    #[crate::ctb_test]
    fn test_format_lzma2() {
        run_format_test_suite(CompressionFormat::Lzma2);
    }

    #[crate::ctb_test]
    fn test_format_lzip() {
        run_format_test_suite(CompressionFormat::Lzip);
    }

    #[crate::ctb_test]
    fn test_format_xz() {
        run_format_test_suite(CompressionFormat::Xz);
    }

    #[crate::ctb_test]
    fn test_format_zstd() {
        run_format_test_suite(CompressionFormat::Zstd);
    }

    #[crate::ctb_test]
    fn test_format_lzo() {
        run_format_test_suite(CompressionFormat::Lzo);
    }
}
