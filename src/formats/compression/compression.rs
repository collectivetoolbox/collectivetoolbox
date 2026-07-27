//! Single-stream compression algorithms (Brotli, Gzip, Deflate, Zlib).

#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace crate prelude")]
pub(crate) use ctb_utilities::*;

use include_dir::{Dir, include_dir};
use std::io::{Read, Write};

pub mod sco_compress;

static COMPRESSION_DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data");

/// Returns an embedded fixture asset byte vector if present.
pub fn get_compression_data(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&COMPRESSION_DATA_DIR, key)
}

/// Supported single-stream compression formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionFormat {
    /// Brotli compressed stream (.br)
    Brotli,
    /// Gzip compressed stream (.gz)
    Gzip,
    /// Raw deflate compressed stream (.deflate)
    Deflate,
    /// Zlib-wrapped deflate compressed stream RFC 1950 (.zz or .zl)
    Zlib,
    /// SCO compress -H compressed stream (.sco or .Z)
    ScoCompress,
}

impl CompressionFormat {
    /// Returns standard extension associated with the format.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Brotli => "br",
            Self::Gzip => "gz",
            Self::Deflate => "deflate",
            Self::Zlib => "zz",
            Self::ScoCompress => "sco",
        }
    }

    /// Infers compression format from file extension if recognized.
    pub fn from_extension(ext: &str) -> Option<Self> {
        let clean_ext = ext.trim_start_matches('.').to_ascii_lowercase();
        match clean_ext.as_str() {
            "br" | "brotli" => Some(Self::Brotli),
            "gz" | "gzip" => Some(Self::Gzip),
            "deflate" | "raw-deflate" => Some(Self::Deflate),
            "zz" | "zl" | "zlib" => Some(Self::Zlib),
            "sco" | "compress-sco" | "sco-compress" | "lzh" | "compress-h" => {
                Some(Self::ScoCompress)
            }
            _ => None,
        }
    }

    /// Infers compression format from magic header bytes if possible.
    pub fn from_magic_bytes(header: &[u8]) -> Option<Self> {
        if let (Some(&b0), Some(&b1)) = (header.first(), header.get(1)) {
            if b0 == 0x1f && b1 == 0x8b {
                return Some(Self::Gzip);
            }
            if b0 == 0x1f && b1 == 0xa0 {
                return Some(Self::ScoCompress);
            }
            if b0 == 0x78 && matches!(b1, 0x01 | 0x9c | 0xda) {
                return Some(Self::Zlib);
            }
        }
        None
    }
}

impl TryFrom<&str> for CompressionFormat {
    type Error = anyhow::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_ascii_lowercase().as_str() {
            "brotli" | "br" => Ok(Self::Brotli),
            "gzip" | "gz" => Ok(Self::Gzip),
            "deflate" | "raw-deflate" => Ok(Self::Deflate),
            "zlib" | "zz" | "zl" | "zlib-deflate" => Ok(Self::Zlib),
            "sco-compress" | "compress-sco" | "sco" | "lzh" | "compress-h" => {
                Ok(Self::ScoCompress)
            }
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
            CompressionFormat::try_from("br").unwrap(),
            CompressionFormat::Brotli
        );
        assert_eq!(
            CompressionFormat::try_from("gzip").unwrap(),
            CompressionFormat::Gzip
        );
        assert_eq!(
            CompressionFormat::try_from("gz").unwrap(),
            CompressionFormat::Gzip
        );
        assert_eq!(
            CompressionFormat::try_from("deflate").unwrap(),
            CompressionFormat::Deflate
        );
        assert_eq!(
            CompressionFormat::try_from("zlib").unwrap(),
            CompressionFormat::Zlib
        );
        assert_eq!(
            CompressionFormat::try_from("zz").unwrap(),
            CompressionFormat::Zlib
        );
        assert_eq!(
            CompressionFormat::try_from("zl").unwrap(),
            CompressionFormat::Zlib
        );
        assert_eq!(
            CompressionFormat::try_from("sco-compress").unwrap(),
            CompressionFormat::ScoCompress
        );
        assert_eq!(
            CompressionFormat::try_from("sco").unwrap(),
            CompressionFormat::ScoCompress
        );
        assert_eq!(
            CompressionFormat::try_from("lzh").unwrap(),
            CompressionFormat::ScoCompress
        );

        assert_eq!(CompressionFormat::Brotli.extension(), "br");
        assert_eq!(CompressionFormat::Gzip.extension(), "gz");
        assert_eq!(CompressionFormat::Deflate.extension(), "deflate");
        assert_eq!(CompressionFormat::Zlib.extension(), "zz");
        assert_eq!(CompressionFormat::ScoCompress.extension(), "sco");
    }

    #[crate::ctb_test]
    fn test_round_trip_all_formats() {
        let sample = b"The quick brown fox jumps over the lazy dog. 1234567890!";
        let formats = [
            CompressionFormat::Brotli,
            CompressionFormat::Gzip,
            CompressionFormat::Deflate,
            CompressionFormat::Zlib,
            CompressionFormat::ScoCompress,
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

        assert!(!raw.is_empty(), "Raw fixture must not be empty");

        // Byte-by-byte comparisons against raw fixture
        let decomp_gz =
            decompress(&gz, CompressionFormat::Gzip).expect("Gz decompress failed");
        assert_eq!(decomp_gz.len(), raw.len());
        assert_eq!(decomp_gz, raw, "Gz decompressed content byte-by-byte mismatch");

        let decomp_br =
            decompress(&br, CompressionFormat::Brotli).expect("Brotli decompress failed");
        assert_eq!(decomp_br.len(), raw.len());
        assert_eq!(decomp_br, raw, "Brotli decompressed content byte-by-byte mismatch");

        let decomp_deflate =
            decompress(&deflate, CompressionFormat::Deflate).expect("Deflate decompress failed");
        assert_eq!(decomp_deflate.len(), raw.len());
        assert_eq!(decomp_deflate, raw, "Deflate decompressed content byte-by-byte mismatch");

        let decomp_zz =
            decompress(&zz, CompressionFormat::Zlib).expect("Zlib decompress failed");
        assert_eq!(decomp_zz.len(), raw.len());
        assert_eq!(decomp_zz, raw, "Zlib decompressed content byte-by-byte mismatch");

        let decomp_sco =
            decompress(&sco, CompressionFormat::ScoCompress).expect("SCO compress decompress failed");
        assert_eq!(decomp_sco.len(), raw.len());
        assert_eq!(decomp_sco, raw, "SCO compress decompressed content byte-by-byte mismatch");
    }
}