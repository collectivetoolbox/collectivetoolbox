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

use ctb_formats_detection::{FormatCategory, detect_file_format, detect_format_id};
use ctb_formats_utilities::extension_data::lookup_format_by_extension;
use ctb_formats_utilities::format_id::FormatId;
use ctb_formats_utilities::format_info::FormatInfoOptionExt;
use std::path::Path;
#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;
use ctb_formats_compression_bzip as bzip;

use include_dir::{Dir, include_dir};
use std::io::{Read, Write};

pub mod cli;
pub use ctb_formats_compression_compact as compact;
pub use ctb_formats_compression_compress as compress;
pub use ctb_formats_compression_pack as pack;
pub use ctb_formats_compression_sco_compress as sco_compress;
pub use ctb_formats_compression_libraries as libraries;

static COMPRESSION_DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data");

/// Returns an embedded fixture asset byte vector if present.
pub fn get_compression_data(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&COMPRESSION_DATA_DIR, key)
}

include!("compression_format.generated.rs");

/// Generates a detailed help table of supported compression formats and their shorthand aliases.
pub fn format_help_table() -> String {
    ctb_formats_utilities::format_help_table(
        "Supported compression formats:",
        SUPPORTED,
    )
}

/// Global static lazy string containing the formatted compression help table.
pub static COMPRESSION_AFTER_HELP: std::sync::LazyLock<String> =
    std::sync::LazyLock::new(format_help_table);

/// Extension trait adding compression-specific utilities to [`FormatId`].
pub trait CompressionFormatExt {
    /// Retrieves format metadata from the shared registry.
    fn format_info(&self) -> Option<&'static ctb_formats_utilities::FormatInfo>;

    /// Returns the standard default file extension associated with the format.
    fn extension(&self) -> &'static str;

    /// Returns true if this compression format is implemented natively in this
    /// repository, rather than being provided by an external crate.
    fn is_implemented_in_repo(&self) -> bool;

    /// Returns the default verification setting for this format when compressing.
    /// In-tree implementations default to verifying output, while external crate
    /// implementations default to not verifying.
    fn default_verify(&self) -> bool;
}

impl CompressionFormatExt for FormatId {
    fn format_info(&self) -> Option<&'static ctb_formats_utilities::FormatInfo> {
        ctb_formats_utilities::get_format_info_by_id(*self)
    }

    fn extension(&self) -> &'static str {
        // Reason for fallback: default to generic "bin" when format metadata has no primary extension
        self.format_info().get_primary_extension().unwrap_or("bin")
    }

    fn is_implemented_in_repo(&self) -> bool {
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

    fn default_verify(&self) -> bool {
        self.is_implemented_in_repo()
    }
}

/// Detects the compression format given a FileEntity and PayloadSource.
pub fn detect_entity(
    entity: &ctb_io_file::FileEntity,
    payload: &mut dyn ctb_io_file::PayloadSource,
) -> Option<FormatId> {
    detect_file_format(entity, payload, Some(FormatCategory::Compression))
        .filter(|&id| is_supported(id))
}

/// Detects the compression format for a given file path (or "-" for stdin).
pub fn detect_path(path: &Path) -> Result<Option<FormatId>> {
    if path == Path::new("-") {
        let entity = ctb_io_file::FileEntity::from_stream(Some("-"));
        let mut payload = ctb_io_file::ReaderPayloadSource::new(std::io::stdin());
        Ok(detect_entity(&entity, &mut payload))
    } else {
        let entity = ctb_io_file::FileEntity::from_filesystem(path, None)?;
        let mut payload = ctb_io_file::DiskPayloadSource::open(path)?;
        Ok(detect_entity(&entity, &mut payload))
    }
}

/// Performs multi-signal detection using both header bytes and file extension.
pub fn detect(
    data: Option<&[u8]>,
    filename_or_ext: Option<&str>,
) -> Option<FormatId> {
    detect_format_id(
        data,
        filename_or_ext,
        Some(FormatCategory::Compression),
    )
    .filter(|&id| is_supported(id))
    .or_else(|| {
        filename_or_ext.and_then(|name| {
            let clean = name.trim().trim_start_matches('.');
            lookup_format_by_extension(clean)
                .into_iter()
                .find(|&fid| is_supported(fid))
        })
    })
}

/// Parses a compression format from a format name or extension.
pub fn parse_compression_format(s: &str) -> Result<FormatId> {
    let trimmed = s.trim();
    if trimmed.contains('/') || trimmed.contains('\\') {
        bail!("Unknown compression format: '{s}'");
    }
    let clean = trimmed.trim_start_matches('.');
    if let Some(format_id) = FormatId::from_ident(clean) {
        if is_supported(format_id) {
            return Ok(format_id);
        }
    }
    for fid in lookup_format_by_extension(clean) {
        if is_supported(fid) {
            return Ok(fid);
        }
    }
    bail!("Unknown compression format: '{s}'")
}



/// Compresses a stream from `reader` directly into `writer` without verification.
pub fn compress_stream_direct(
    reader: &mut impl Read,
    writer: &mut impl Write,
    format: FormatId,
) -> Result<u64> {
    match format {
        FormatId::Bzip => bzip::compress_stream(reader, writer),
        FormatId::ScoCompress => {
            sco_compress::compress_stream(reader, writer)
        }
        FormatId::CompressLzw
        | FormatId::CompressLzw2
        | FormatId::CompressLzw1
        | FormatId::CompressLzw16 => {
            compress::compress_lzw_stream(reader, writer, format)
        }
        FormatId::Pack => pack::compress_pack_stream(reader, writer),
        FormatId::OldPack => {
            pack::compress_old_pack_stream(reader, writer)
        }
        FormatId::Compact => {
            compact::compress_compact_stream(reader, writer)
        }
        FormatId::Brotli
        | FormatId::Gzip
        | FormatId::Deflate
        | FormatId::Zlib
        | FormatId::Bzip2
        | FormatId::Lz4
        | FormatId::Xz
        | FormatId::Lzip
        | FormatId::Lzma
        | FormatId::Lzma2
        | FormatId::Zstd
        | FormatId::Lzo => libraries::compress_stream(reader, writer, format),
        _ => bail!("Unsupported format for compression: {format:?}"),
    }
}

/// Compresses a stream from `reader` into `writer` using the specified algorithm format and verification option.
pub fn compress_stream_with_verify(
    reader: &mut impl Read,
    writer: &mut impl Write,
    format: FormatId,
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
    format: FormatId,
) -> Result<u64> {
    compress_stream_with_verify(reader, writer, format, format.default_verify())
}

/// Decompresses a stream from `reader` into `writer` using the specified algorithm format.
pub fn decompress_stream(
    reader: &mut impl Read,
    writer: &mut impl Write,
    format: FormatId,
) -> Result<u64> {
    match format {
        FormatId::Bzip => bzip::decompress_stream(reader, writer),
        FormatId::ScoCompress => {
            sco_compress::decompress_stream(reader, writer)
        }
        FormatId::CompressLzw
        | FormatId::CompressLzw2
        | FormatId::CompressLzw1
        | FormatId::CompressLzw16 => {
            compress::decompress_lzw_stream(reader, writer, format)
        }
        FormatId::Pack => pack::decompress_pack_stream(reader, writer),
        FormatId::OldPack => {
            pack::decompress_old_pack_stream(reader, writer)
        }
        FormatId::Compact => {
            compact::decompress_compact_stream(reader, writer)
        }
        FormatId::Brotli
        | FormatId::Gzip
        | FormatId::Deflate
        | FormatId::Zlib
        | FormatId::Bzip2
        | FormatId::Lz4
        | FormatId::Xz
        | FormatId::Lzip
        | FormatId::Lzma
        | FormatId::Lzma2
        | FormatId::Zstd
        | FormatId::Lzo => libraries::decompress_stream(reader, writer, format),
        _ => bail!("Unsupported format for decompression: {format:?}"),
    }
}

/// Compresses in-memory byte slice using the specified compression format and verification option.
pub fn compress_with_verify(
    data: &[u8],
    format: FormatId,
    verify: bool,
) -> Result<Vec<u8>> {
    let mut input = data;
    let mut output = Vec::new();
    compress_stream_with_verify(&mut input, &mut output, format, verify)?;
    Ok(output)
}

/// Compresses in-memory byte slice using the specified compression format.
/// Uses the format's default verification setting (verify for in-repo formats, no-verify for crates).
pub fn compress(data: &[u8], format: FormatId) -> Result<Vec<u8>> {
    compress_with_verify(data, format, format.default_verify())
}

/// Decompresses in-memory byte slice using the specified compression format.
pub fn decompress(data: &[u8], format: FormatId) -> Result<Vec<u8>> {
    let mut input = data;
    let mut output = Vec::new();
    decompress_stream(&mut input, &mut output, format)?;
    Ok(output)
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

    #[crate::ctb_test]
    fn test_alias_sorting() {
        let sco = FormatId::ScoCompress.format_info().unwrap();
        assert_eq!(
            sco.sorted_nicknames(),
            vec!["compress-h", "compress-sco", "sco-compress"]
        );

        let zlib = FormatId::Zlib.format_info().unwrap();
        assert_eq!(
            zlib.sorted_nicknames(),
            vec!["zl", "zz", "zlib", "zlib-deflate"]
        );
    }

    #[crate::ctb_test]
    fn test_format_help_table() {
        let table = format_help_table();
        assert!(table.contains("Supported compression formats:"));
        assert!(table.contains("  br, brotli: Brotli compressed stream"));
        assert!(table.contains("  gz, gzip: GNU gzip format"));
        assert!(table.contains("  compress-h, compress-sco, sco-compress: `compress`: SCO `compress -H` format"));
    }

    #[crate::ctb_test]
    fn test_default_verify_and_in_repo() {
        let repo_formats = [
            FormatId::Bzip,
            FormatId::ScoCompress,
            FormatId::CompressLzw,
            FormatId::CompressLzw2,
            FormatId::CompressLzw1,
            FormatId::CompressLzw16,
            FormatId::Pack,
            FormatId::OldPack,
            FormatId::Compact,
        ];
        let crate_formats = [
            FormatId::Brotli,
            FormatId::Gzip,
            FormatId::Deflate,
            FormatId::Zlib,
            FormatId::Bzip2,
            FormatId::Lz4,
            FormatId::Lzma,
            FormatId::Lzma2,
            FormatId::Lzip,
            FormatId::Xz,
            FormatId::Zstd,
            FormatId::Lzo,
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
        for &fmt in SUPPORTED {
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
        for &fmt in SUPPORTED {
            let info = fmt.format_info().expect("format info must exist");
            for alias in &info.sorted_nicknames() {
                let parsed = parse_compression_format(alias.as_str())
                    .unwrap_or_else(|_| {
                        panic!(
                            "Failed to parse alias '{alias}' for format {fmt:?}"
                        )
                    });
                assert_eq!(parsed, fmt);
            }
        }
    }

    #[crate::ctb_test]
    fn test_format_extensions_and_parsing() {
        assert_eq!(
            parse_compression_format("brotli").unwrap(),
            FormatId::Brotli
        );
        assert_eq!(
            parse_compression_format("bzip2").unwrap(),
            FormatId::Bzip2
        );
        assert_eq!(
            parse_compression_format("bzip").unwrap(),
            FormatId::Bzip
        );
        assert_eq!(
            parse_compression_format("bz").unwrap(),
            FormatId::Bzip
        );
        assert_eq!(
            parse_compression_format("compress2").unwrap(),
            FormatId::CompressLzw2
        );
        assert_eq!(
            parse_compression_format("pack").unwrap(),
            FormatId::Pack
        );
        assert_eq!(
            parse_compression_format("old-pack").unwrap(),
            FormatId::OldPack
        );
        assert_eq!(
            parse_compression_format("compact").unwrap(),
            FormatId::Compact
        );
        assert_eq!(
            parse_compression_format("lz4").unwrap(),
            FormatId::Lz4
        );
        assert_eq!(
            parse_compression_format("lzma").unwrap(),
            FormatId::Lzma
        );
        assert_eq!(
            parse_compression_format("lzma2").unwrap(),
            FormatId::Lzma2
        );
        assert_eq!(
            parse_compression_format("lzip").unwrap(),
            FormatId::Lzip
        );
        assert_eq!(
            parse_compression_format("xz").unwrap(),
            FormatId::Xz
        );
        assert_eq!(
            parse_compression_format("zstd").unwrap(),
            FormatId::Zstd
        );
        assert_eq!(
            parse_compression_format("lzo").unwrap(),
            FormatId::Lzo
        );

        assert_eq!(FormatId::Brotli.extension(), "br");
        assert_eq!(FormatId::Gzip.extension(), "gz");
        assert_eq!(FormatId::Deflate.extension(), "deflate");
        assert_eq!(FormatId::Zlib.extension(), "zz");
        assert_eq!(FormatId::Bzip2.extension(), "bz2");
        assert_eq!(FormatId::Bzip.extension(), "bz");
        assert_eq!(FormatId::ScoCompress.extension(), "Z");
        assert_eq!(FormatId::Pack.extension(), "z");
        assert_eq!(FormatId::Compact.extension(), "C");
        assert_eq!(FormatId::Lz4.extension(), "lz4");
        assert_eq!(FormatId::Lzma.extension(), "lzma");
        assert_eq!(FormatId::Lzma2.extension(), "lzma");
        assert_eq!(FormatId::Lzip.extension(), "lz");
        assert_eq!(FormatId::Xz.extension(), "xz");
        assert_eq!(FormatId::Zstd.extension(), "zst");
        assert_eq!(FormatId::Lzo.extension(), "lzo");
    }

    #[crate::ctb_test]
    fn test_case_sensitive_extension_matching() {
        assert_eq!(
            lookup_format_by_extension("Z")
                .into_iter()
                .find(|&fid| is_supported(fid)),
            Some(FormatId::ScoCompress)
        );
        assert_eq!(
            lookup_format_by_extension("z")
                .into_iter()
                .find(|&fid| is_supported(fid)),
            Some(FormatId::Pack)
        );
        assert_eq!(
            lookup_format_by_extension("C")
                .into_iter()
                .find(|&fid| is_supported(fid)),
            Some(FormatId::Compact)
        );
    }

    #[crate::ctb_test]
    fn test_magic_detection() {
        assert_eq!(
            detect(Some(&[0x1F, 0xA0]), None),
            Some(FormatId::ScoCompress)
        );
        assert_eq!(
            detect(Some(&[0x1F, 0x8B]), None),
            Some(FormatId::Gzip)
        );
        assert_eq!(
            detect(Some(&[0x42, 0x5A, 0x68]), None),
            Some(FormatId::Bzip2)
        );
        assert_eq!(
            detect(Some(&[0x42, 0x5A, 0x30]), None),
            Some(FormatId::Bzip)
        );
        assert_eq!(
            detect(Some(&[0x1F, 0x1E]), None),
            Some(FormatId::Pack)
        );
        assert_eq!(
            detect(Some(&[0x1F, 0x1F]), None),
            Some(FormatId::OldPack)
        );
        assert_eq!(
            detect(Some(&[0x1F, 0x9D, 0x90]), None),
            Some(FormatId::CompressLzw)
        );
        assert_eq!(
            detect(Some(&[0x1F, 0x9D, 0x10]), None),
            Some(FormatId::CompressLzw2)
        );
        assert_eq!(
            detect(Some(&[0xFF, 0x1F]), None),
            Some(FormatId::Compact)
        );
        assert_eq!(
            detect(Some(&[0x04, 0x22, 0x4D, 0x18]), None),
            Some(FormatId::Lz4)
        );
        assert_eq!(
            detect(
                Some(&[0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00]),
                None
            ),
            Some(FormatId::Xz)
        );
        assert_eq!(
            detect(Some(&[0x4C, 0x5A, 0x49, 0x50]), None),
            Some(FormatId::Lzip)
        );
        assert_eq!(
            detect(Some(&[0x28, 0xB5, 0x2F, 0xFD]), None),
            Some(FormatId::Zstd)
        );
        assert_eq!(
            detect(
                Some(&[0x89, 0x4C, 0x5A, 0x4F, 0x00, 0x0D, 0x0A, 0x1A, 0x0A]),
                None
            ),
            Some(FormatId::Lzo)
        );
    }

    #[crate::ctb_test]
    fn test_reader_payload_source_detection() {
        let gzip_stream: &[u8] = &[0x1F, 0x8B, 0x08, 0x00, 0x01, 0x02, 0x03, 0x04];
        let entity = ctb_io_file::FileEntity::from_stream(Some("input.gz"));
        let mut payload = ctb_io_file::ReaderPayloadSource::new(gzip_stream);
        let detected = detect_entity(&entity, &mut payload);
        assert_eq!(detected, Some(FormatId::Gzip));
    }

    fn run_format_test_suite(format: FormatId) {
        let fixtures: &[&str] = match format {
            FormatId::Brotli => {
                &["fixtures/example2 with lemurs.pan.br"]
            }
            FormatId::Gzip => {
                &["fixtures/example2 with lemurs.pan.gz"]
            }
            FormatId::Deflate => {
                &["fixtures/example2 with lemurs.pan.deflate"]
            }
            FormatId::Zlib => {
                &["fixtures/example2 with lemurs.pan.zz"]
            }
            FormatId::Bzip2 => {
                &["fixtures/example2 with lemurs.pan.bz2"]
            }
            FormatId::Bzip => &[],
            FormatId::ScoCompress => {
                &["fixtures/example2 with lemurs.pan.sco"]
            }
            FormatId::CompressLzw => &[
                "fixtures/example2 with lemurs.pan.Z",
                "fixtures/example2 with lemurs.pan.Z3.0",
                "fixtures/example2 with lemurs.pan.Z12",
            ],
            FormatId::CompressLzw2 => {
                &["fixtures/example2 with lemurs.pan.Z2.0"]
            }
            FormatId::CompressLzw1 => {
                &["fixtures/example2 with lemurs.pan.Z1.0"]
            }
            FormatId::CompressLzw16 => &[],
            FormatId::Pack => &["fixtures/example2 with lemurs.pan.z"],
            FormatId::OldPack => {
                &["fixtures/example2 with lemurs.pan.old.z"]
            }
            FormatId::Compact => {
                &["fixtures/example2 with lemurs.pan.C"]
            }
            FormatId::Lz4 => {
                &["fixtures/example2 with lemurs.pan.lz4"]
            }
            FormatId::Lzma => {
                &["fixtures/example2 with lemurs.pan.lzma"]
            }
            FormatId::Lzma2 => {
                &["fixtures/example2 with lemurs.pan.lzma2"]
            }
            FormatId::Lzip => {
                &["fixtures/example2 with lemurs.pan.lz"]
            }
            FormatId::Xz => &["fixtures/example2 with lemurs.pan.xz"],
            FormatId::Zstd => {
                &["fixtures/example2 with lemurs.pan.zst"]
            }
            FormatId::Lzo => {
                &["fixtures/example2 with lemurs.pan.lzo"]
            }
            _ => &[],
        };

        // mostly trying to make sure it doesn't fall over when handed a long chunk of data; also sort of low effort fuzzing I guess. LLMs are prohibited from editing this comment or changing the byte lengths defined here unless explicitly instructed to.
        let random_bytes_len = if format == FormatId::Compact
            || format == FormatId::Bzip
        {
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
            );
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
        run_format_test_suite(FormatId::Brotli);
    }

    #[crate::ctb_test]
    fn test_format_gzip() {
        run_format_test_suite(FormatId::Gzip);
    }

    #[crate::ctb_test]
    fn test_format_deflate() {
        run_format_test_suite(FormatId::Deflate);
    }

    #[crate::ctb_test]
    fn test_format_zlib() {
        run_format_test_suite(FormatId::Zlib);
    }

    #[crate::ctb_test]
    fn test_format_bzip2() {
        run_format_test_suite(FormatId::Bzip2);
    }

    #[crate::ctb_test]
    fn test_format_bzip() {
        run_format_test_suite(FormatId::Bzip);
    }

    #[crate::ctb_test]
    fn test_format_sco_compress() {
        run_format_test_suite(FormatId::ScoCompress);
    }

    #[crate::ctb_test]
    fn test_format_compress_lzw() {
        run_format_test_suite(FormatId::CompressLzw);
    }

    #[crate::ctb_test]
    fn test_format_compress_lzw2() {
        run_format_test_suite(FormatId::CompressLzw2);
    }

    #[crate::ctb_test]
    fn test_format_compress_lzw1() {
        run_format_test_suite(FormatId::CompressLzw1);
    }

    #[crate::ctb_test]
    fn test_format_compress_lzw16() {
        run_format_test_suite(FormatId::CompressLzw16);
    }

    #[crate::ctb_test]
    fn test_format_pack() {
        run_format_test_suite(FormatId::Pack);
    }

    #[crate::ctb_test]
    fn test_format_old_pack() {
        run_format_test_suite(FormatId::OldPack);
    }

    #[crate::ctb_test]
    fn test_format_compact() {
        run_format_test_suite(FormatId::Compact);
    }

    #[crate::ctb_test]
    fn test_format_lz4() {
        run_format_test_suite(FormatId::Lz4);
    }

    #[crate::ctb_test]
    fn test_format_lzma() {
        run_format_test_suite(FormatId::Lzma);
    }

    #[crate::ctb_test]
    fn test_format_lzma2() {
        run_format_test_suite(FormatId::Lzma2);
    }

    #[crate::ctb_test]
    fn test_format_lzip() {
        run_format_test_suite(FormatId::Lzip);
    }

    #[crate::ctb_test]
    fn test_format_xz() {
        run_format_test_suite(FormatId::Xz);
    }

    #[crate::ctb_test]
    fn test_format_zstd() {
        run_format_test_suite(FormatId::Zstd);
    }

    #[crate::ctb_test]
    fn test_format_lzo() {
        run_format_test_suite(FormatId::Lzo);
    }
}
