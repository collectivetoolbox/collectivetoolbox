//! Single-stream compression algorithms (Brotli, Gzip, Deflate, Zlib, SCO Compress -H, etc.).

#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace crate prelude")]
pub(crate) use ctb_utilities::*;
use ctb_formats_utilities::detection::{FormatCategory, detect_format_id};
use ctb_formats_utilities::extension_data::lookup_format_by_extension;
use ctb_formats_utilities::format_id::FormatId;

use include_dir::{Dir, include_dir};
use std::io::{Read, Write};

pub mod cli;
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
    /// SCO compress -H compressed stream (.Z)
    ScoCompress,
    /// Modern standard LZW / compress 4.0 / ncompress (.Z)
    CompressLzw,
    /// Compress 2.0 non-block LZW (.Z)
    CompressLzw2,
    /// Compress 1.0 headerless LZW (.Z)
    CompressLzw1,
    /// System III/V Canonical Huffman pack (.z)
    Pack,
    /// Early PDP-11 Unix binary tree pack (.z)
    OldPack,
    /// McMaster Adaptive Huffman compact (.C)
    Compact,
}

impl CompressionFormat {
    /// Maps this compression format variant to the global `FormatId`.
    pub fn to_format_id(&self) -> FormatId {
        match self {
            Self::Brotli => FormatId::Brotli,
            Self::Gzip => FormatId::Gzip,
            Self::Deflate => FormatId::Deflate,
            Self::Zlib => FormatId::Zlib,
            Self::ScoCompress => FormatId::ScoCompress,
            Self::CompressLzw => FormatId::CompressLzw,
            Self::CompressLzw2 => FormatId::CompressLzw2,
            Self::CompressLzw1 => FormatId::CompressLzw1,
            Self::Pack => FormatId::Pack,
            Self::OldPack => FormatId::OldPack,
            Self::Compact => FormatId::Compact,
        }
    }

    /// Converts a global `FormatId` to a `CompressionFormat` if it represents a compression format.
    pub fn from_format_id(id: FormatId) -> Option<Self> {
        match id {
            FormatId::Brotli => Some(Self::Brotli),
            FormatId::Gzip => Some(Self::Gzip),
            FormatId::Deflate => Some(Self::Deflate),
            FormatId::Zlib => Some(Self::Zlib),
            FormatId::ScoCompress => Some(Self::ScoCompress),
            FormatId::CompressLzw => Some(Self::CompressLzw),
            FormatId::CompressLzw2 => Some(Self::CompressLzw2),
            FormatId::CompressLzw1 => Some(Self::CompressLzw1),
            FormatId::Pack => Some(Self::Pack),
            FormatId::OldPack => Some(Self::OldPack),
            FormatId::Compact => Some(Self::Compact),
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
            Self::ScoCompress | Self::CompressLzw | Self::CompressLzw2 | Self::CompressLzw1 => "Z",
            Self::Pack | Self::OldPack => "z",
            Self::Compact => "C",
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
        detect_format_id(
            Some(header),
            None,
            Some(FormatCategory::Compression),
        )
        .and_then(Self::from_format_id)
    }

    /// Performs multi-signal detection using both header bytes and file extension.
    pub fn detect(data: Option<&[u8]>, filename_or_ext: Option<&str>) -> Option<Self> {
        detect_format_id(
            data,
            filename_or_ext,
            Some(FormatCategory::Compression),
        )
        .and_then(Self::from_format_id)
    }
}

impl TryFrom<&str> for CompressionFormat {
    type Error = anyhow::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.trim().to_ascii_lowercase().as_str() {
            "brotli" | "br" => Ok(Self::Brotli),
            "gzip" | "gz" => Ok(Self::Gzip),
            "deflate" | "raw-deflate" => Ok(Self::Deflate),
            "zlib" | "zz" | "zl" | "zlib-deflate" => Ok(Self::Zlib),
            "sco-compress" | "compress-sco" | "compress-h" => Ok(Self::ScoCompress),
            "compress" | "sco" | "ncompress" | "lzw" => Ok(Self::ScoCompress),
            "compress4" | "compress-4.0" | "compress-3.0" | "lzw-block" => Ok(Self::CompressLzw),
            "compress2" | "compress-2.0" | "lzw-nonblock" => Ok(Self::CompressLzw2),
            "compress1" | "compress-1.0" | "lzw-headerless" => Ok(Self::CompressLzw1),
            "pack" | "sys3-pack" | "sys5-pack" => Ok(Self::Pack),
            "old-pack" | "opack" | "pts-opack" | "early-pack" => Ok(Self::OldPack),
            "compact" | "uncompact" => Ok(Self::Compact),
            _ => bail!("Unknown compression format: '{s}'"),
        }
    }
}

impl TryFrom<String> for CompressionFormat {
    type Error = anyhow::Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::try_from(s.as_str())
    }
}

/// Compresses a stream from `reader` into `writer` using the specified algorithm format.
pub fn compress_stream(
    reader: &mut impl Read,
    writer: &mut impl Write,
    format: CompressionFormat,
) -> Result<u64> {
    match format {
        CompressionFormat::Brotli => {
            let mut encoder = brotli::CompressorWriter::new(writer, 4096, 6, 22);
            let bytes_written = std::io::copy(reader, &mut encoder)
                .context("Failed to write to Brotli encoder")?;
            encoder
                .flush()
                .context("Failed to flush Brotli encoder")?;
            Ok(bytes_written)
        }
        CompressionFormat::Gzip => {
            let mut encoder =
                flate2::write::GzEncoder::new(writer, flate2::Compression::default());
            let bytes_written = std::io::copy(reader, &mut encoder)
                .context("Failed to write to Gzip encoder")?;
            encoder.finish().context("Failed to finish Gzip encoder")?;
            Ok(bytes_written)
        }
        CompressionFormat::Deflate => {
            let mut encoder =
                flate2::write::DeflateEncoder::new(writer, flate2::Compression::default());
            let bytes_written = std::io::copy(reader, &mut encoder)
                .context("Failed to write to Deflate encoder")?;
            encoder
                .finish()
                .context("Failed to finish Deflate encoder")?;
            Ok(bytes_written)
        }
        CompressionFormat::Zlib => {
            let mut encoder =
                flate2::write::ZlibEncoder::new(writer, flate2::Compression::default());
            let bytes_written = std::io::copy(reader, &mut encoder)
                .context("Failed to write to Zlib encoder")?;
            encoder.finish().context("Failed to finish Zlib encoder")?;
            Ok(bytes_written)
        }
        CompressionFormat::ScoCompress => sco_compress::compress_stream(reader, writer),
        CompressionFormat::CompressLzw
        | CompressionFormat::CompressLzw2
        | CompressionFormat::CompressLzw1 => compress::compress_lzw_stream(reader, writer, format),
        CompressionFormat::Pack => pack::compress_pack_stream(reader, writer),
        CompressionFormat::OldPack => pack::compress_old_pack_stream(reader, writer),
        other => bail!("Compression for format '{other:?}' is not yet implemented"),
    }
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
            let mut decoder = flate2::read::GzDecoder::new(reader);
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
        CompressionFormat::ScoCompress => sco_compress::decompress_stream(reader, writer),
        CompressionFormat::CompressLzw
        | CompressionFormat::CompressLzw2
        | CompressionFormat::CompressLzw1 => {
            compress::decompress_lzw_stream(reader, writer, format)
        }
        CompressionFormat::Pack => pack::decompress_pack_stream(reader, writer),
        CompressionFormat::OldPack => pack::decompress_old_pack_stream(reader, writer),
        other => bail!("Decompression for format '{other:?}' is not yet implemented"),
    }
}

/// Compresses in-memory byte slice using the specified compression format.
pub fn compress(data: &[u8], format: CompressionFormat) -> Result<Vec<u8>> {
    let mut input = data;
    let mut output = Vec::new();
    compress_stream(&mut input, &mut output, format)?;
    Ok(output)
}

/// Decompresses in-memory byte slice using the specified compression format.
pub fn decompress(data: &[u8], format: CompressionFormat) -> Result<Vec<u8>> {
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
    fn test_format_extensions_and_parsing() {
        assert_eq!(
            CompressionFormat::try_from("brotli").unwrap(),
            CompressionFormat::Brotli
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

        assert_eq!(CompressionFormat::Brotli.extension(), "br");
        assert_eq!(CompressionFormat::Gzip.extension(), "gz");
        assert_eq!(CompressionFormat::Deflate.extension(), "deflate");
        assert_eq!(CompressionFormat::Zlib.extension(), "zz");
        assert_eq!(CompressionFormat::ScoCompress.extension(), "Z");
        assert_eq!(CompressionFormat::Pack.extension(), "z");
        assert_eq!(CompressionFormat::Compact.extension(), "C");
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
    }

    #[crate::ctb_test]
    fn test_round_trip_all_implemented_formats() {
        let sample = b"The quick brown fox jumps over the lazy dog. 1234567890!";
        let formats = [
            CompressionFormat::Brotli,
            CompressionFormat::Gzip,
            CompressionFormat::Deflate,
            CompressionFormat::Zlib,
            CompressionFormat::ScoCompress,
            CompressionFormat::CompressLzw,
            CompressionFormat::CompressLzw2,
            CompressionFormat::CompressLzw1,
            CompressionFormat::Pack,
            CompressionFormat::OldPack,
        ];

        for format in formats {
            let compressed = compress(sample, format).unwrap();
            let decompressed = decompress(&compressed, format).unwrap();
            assert_eq!(
                decompressed,
                sample,
                "Roundtrip failed for format {:?}\nDecompressed ({}) : {:?}\nExpected     ({}) : {:?}",
                format,
                decompressed.len(),
                String::from_utf8_lossy(&decompressed),
                sample.len(),
                String::from_utf8_lossy(sample)
            );
        }
    }

    #[crate::ctb_test]
    fn test_embedded_fixtures() {
        let raw = get_compression_data("fixtures/example2 with lemurs.pan")
            .expect("Raw fixture missing");
        let gz = get_compression_data("fixtures/example2 with lemurs.pan.gz")
            .expect("Gz fixture missing");
        let br = get_compression_data("fixtures/example2 with lemurs.pan.br")
            .expect("Brotli fixture missing");
        let deflate = get_compression_data("fixtures/example2 with lemurs.pan.deflate")
            .expect("Deflate fixture missing");
        let zz = get_compression_data("fixtures/example2 with lemurs.pan.zz")
            .expect("Zlib fixture missing");
        let sco = get_compression_data("fixtures/example2 with lemurs.pan.sco")
            .expect("SCO compress fixture missing");
        let pack_z = get_compression_data("fixtures/example2 with lemurs.pan.z")
            .expect("Pack fixture missing");
        let old_pack_z = get_compression_data("fixtures/example2 with lemurs.pan.old.z")
            .expect("OldPack fixture missing");

        assert!(!raw.is_empty(), "Raw fixture must not be empty");

        let decomp_gz =
            decompress(&gz, CompressionFormat::Gzip).expect("Gz decompress failed");
        assert_eq!(decomp_gz, raw);

        let decomp_br =
            decompress(&br, CompressionFormat::Brotli).expect("Brotli decompress failed");
        assert_eq!(decomp_br, raw);

        let decomp_deflate =
            decompress(&deflate, CompressionFormat::Deflate).expect("Deflate decompress failed");
        assert_eq!(decomp_deflate, raw);

        let decomp_zz =
            decompress(&zz, CompressionFormat::Zlib).expect("Zlib decompress failed");
        assert_eq!(decomp_zz, raw);

        let decomp_sco =
            decompress(&sco, CompressionFormat::ScoCompress).expect("SCO compress decompress failed");
        assert_eq!(decomp_sco, raw);

        let decomp_pack =
            decompress(&pack_z, CompressionFormat::Pack).expect("Pack decompress failed");
        assert_eq!(decomp_pack, raw);

        let decomp_old_pack =
            decompress(&old_pack_z, CompressionFormat::OldPack).expect("OldPack decompress failed");
        assert_eq!(decomp_old_pack, raw);
    }
}