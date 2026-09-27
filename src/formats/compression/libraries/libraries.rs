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

//! Compression algorithms provided by external libraries.

#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace crate prelude")]
pub(crate) use ctb_utilities::*;

use std::io::{Read, Write};

/// Compresses a stream from `reader` into `writer` using the specified algorithm format.
pub fn compress_stream(
    reader: &mut impl Read,
    writer: &mut impl Write,
    format: FormatId,
) -> Result<u64> {
    match format {
        FormatId::Brotli => {
            let mut encoder =
                brotli::CompressorWriter::new(writer, 4096, 6, 22);
            let bytes_written = std::io::copy(reader, &mut encoder)
                .context("Failed to write to Brotli encoder")?;
            encoder.flush().context("Failed to flush Brotli encoder")?;
            Ok(bytes_written)
        }
        FormatId::Gzip => {
            let mut encoder = flate2::write::GzEncoder::new(
                writer,
                flate2::Compression::default(),
            );
            let bytes_written = std::io::copy(reader, &mut encoder)
                .context("Failed to write to Gzip encoder")?;
            encoder.finish().context("Failed to finish Gzip encoder")?;
            Ok(bytes_written)
        }
        FormatId::Deflate => {
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
        FormatId::Zlib => {
            let mut encoder = flate2::write::ZlibEncoder::new(
                writer,
                flate2::Compression::default(),
            );
            let bytes_written = std::io::copy(reader, &mut encoder)
                .context("Failed to write to Zlib encoder")?;
            encoder.finish().context("Failed to finish Zlib encoder")?;
            Ok(bytes_written)
        }
        FormatId::Bzip2 => {
            let mut encoder = bzip2::write::BzEncoder::new(
                writer,
                bzip2::Compression::default(),
            );
            let bytes_written = std::io::copy(reader, &mut encoder)
                .context("Failed to write to Bzip2 encoder")?;
            encoder.finish().context("Failed to finish Bzip2 encoder")?;
            Ok(bytes_written)
        }
        FormatId::Lz4 => {
            let mut encoder = lz4_flex::frame::FrameEncoder::new(writer);
            let bytes_written = std::io::copy(reader, &mut encoder)
                .context("Failed to write to LZ4 encoder")?;
            encoder.finish().context("Failed to finish LZ4 encoder")?;
            Ok(bytes_written)
        }
        FormatId::Xz => {
            let mut encoder = lzma_rust2::XzWriter::new(
                writer,
                lzma_rust2::XzOptions::default(),
            )
            .context("Failed to create XZ encoder")?;
            let bytes_written = std::io::copy(reader, &mut encoder)
                .context("Failed to write to XZ encoder")?;
            encoder.finish().context("Failed to finish XZ encoder")?;
            Ok(bytes_written)
        }
        FormatId::Lzip => {
            let mut encoder = lzma_rust2::LzipWriter::new(
                writer,
                lzma_rust2::LzipOptions::default(),
            );
            let bytes_written = std::io::copy(reader, &mut encoder)
                .context("Failed to write to Lzip encoder")?;
            encoder.finish().context("Failed to finish Lzip encoder")?;
            Ok(bytes_written)
        }
        FormatId::Lzma => {
            let options = lzma_rust2::LzmaOptions::default();
            let mut encoder =
                lzma_rust2::LzmaWriter::new(writer, &options, true, true, None)
                    .context("Failed to create LZMA encoder")?;
            let bytes_written = std::io::copy(reader, &mut encoder)
                .context("Failed to write to LZMA encoder")?;
            encoder.finish().context("Failed to finish LZMA encoder")?;
            Ok(bytes_written)
        }
        FormatId::Lzma2 => {
            let mut encoder = lzma_rust2::Lzma2Writer::new(
                writer,
                lzma_rust2::Lzma2Options::default(),
            );
            let bytes_written = std::io::copy(reader, &mut encoder)
                .context("Failed to write to LZMA2 encoder")?;
            encoder.finish().context("Failed to finish LZMA2 encoder")?;
            Ok(bytes_written)
        }
        FormatId::Zstd => {
            let mut encoder = zstd::stream::write::Encoder::new(writer, 0)
                .context("Failed to initialize Zstd encoder")?;
            let bytes_written = std::io::copy(reader, &mut encoder)
                .context("Failed to write to Zstd encoder")?;
            encoder.finish().context("Failed to finish Zstd encoder")?;
            Ok(bytes_written)
        }
        FormatId::Lzo => {
            let mut input = Vec::new();
            reader
                .read_to_end(&mut input)
                .context("Failed to read input for LZO compression")?;
            if input.is_empty() {
                return Ok(0);
            }
            let compressed = lzokay_native::compress(&input).map_err(|e| {
                anyhow::anyhow!("LZO compression failed: {e:?}")
            })?;
            writer
                .write_all(&compressed)
                .context("Failed to write LZO compressed data")?;
            Ok(u64::try_from(input.len())?)
        }
        _ => bail!("Unsupported format for library compression: {format:?}"),
    }
}

/// Decompresses a stream from `reader` into `writer` using the specified algorithm format.
pub fn decompress_stream(
    reader: &mut impl Read,
    writer: &mut impl Write,
    format: FormatId,
) -> Result<u64> {
    match format {
        FormatId::Brotli => {
            let mut decoder = brotli::Decompressor::new(reader, 4096);
            let bytes_written = std::io::copy(&mut decoder, writer)
                .context("Failed to decompress Brotli stream")?;
            Ok(bytes_written)
        }
        FormatId::Gzip => {
            let mut decoder = flate2::read::MultiGzDecoder::new(reader);
            let bytes_written = std::io::copy(&mut decoder, writer)
                .context("Failed to decompress Gzip stream")?;
            Ok(bytes_written)
        }
        FormatId::Deflate => {
            let mut decoder = flate2::read::DeflateDecoder::new(reader);
            let bytes_written = std::io::copy(&mut decoder, writer)
                .context("Failed to decompress Deflate stream")?;
            Ok(bytes_written)
        }
        FormatId::Zlib => {
            let mut decoder = flate2::read::ZlibDecoder::new(reader);
            let bytes_written = std::io::copy(&mut decoder, writer)
                .context("Failed to decompress Zlib stream")?;
            Ok(bytes_written)
        }
        FormatId::Bzip2 => {
            let mut decoder = bzip2::read::BzDecoder::new(reader);
            let bytes_written = std::io::copy(&mut decoder, writer)
                .context("Failed to decompress Bzip2 stream")?;
            Ok(bytes_written)
        }
        FormatId::Lz4 => {
            let mut decoder = lz4_flex::frame::FrameDecoder::new(reader);
            let bytes_written = std::io::copy(&mut decoder, writer)
                .context("Failed to decompress LZ4 stream")?;
            Ok(bytes_written)
        }
        FormatId::Xz => {
            let mut decoder = lzma_rust2::XzReader::new(reader, true);
            let bytes_written = std::io::copy(&mut decoder, writer)
                .context("Failed to decompress XZ stream")?;
            Ok(bytes_written)
        }
        FormatId::Lzip => {
            let mut decoder = lzma_rust2::LzipReader::new(reader);
            let bytes_written = std::io::copy(&mut decoder, writer)
                .context("Failed to decompress Lzip stream")?;
            Ok(bytes_written)
        }
        FormatId::Lzma => {
            let mut decoder =
                lzma_rust2::LzmaReader::new_mem_limit(reader, u32::MAX, None)
                    .context("Failed to create LZMA decoder")?;
            let bytes_written = std::io::copy(&mut decoder, writer)
                .context("Failed to decompress LZMA stream")?;
            Ok(bytes_written)
        }
        FormatId::Lzma2 => {
            let mut decoder = lzma_rust2::Lzma2Reader::new(
                reader,
                lzma_rust2::Lzma2Options::default().lzma_options.dict_size,
                None,
            );
            let bytes_written = std::io::copy(&mut decoder, writer)
                .context("Failed to decompress LZMA2 stream")?;
            Ok(bytes_written)
        }
        FormatId::Zstd => {
            let mut decoder = zstd::stream::read::Decoder::new(reader)
                .context("Failed to initialize Zstd decoder")?;
            let bytes_written = std::io::copy(&mut decoder, writer)
                .context("Failed to decompress Zstd stream")?;
            Ok(bytes_written)
        }
        FormatId::Lzo => {
            let mut input = Vec::new();
            reader
                .read_to_end(&mut input)
                .context("Failed to read input for LZO decompression")?;
            if input.is_empty() {
                return Ok(0);
            }
            let mut cursor = std::io::Cursor::new(input);
            let decompressed = lzokay_native::decompress(&mut cursor, None)
                .map_err(|e| {
                    anyhow::anyhow!("LZO decompression failed: {e:?}")
                })?;
            writer
                .write_all(&decompressed)
                .context("Failed to write LZO decompressed data")?;
            Ok(u64::try_from(decompressed.len())?)
        }
        _ => bail!("Unsupported format for library decompression: {format:?}"),
    }
}
