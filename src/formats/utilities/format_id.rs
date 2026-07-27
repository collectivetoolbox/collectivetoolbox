//! Standardized format identifier enum across all workspace format crates.

#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use ctb_utilities::*;
use crate::detection::FormatCategory;

/// Unified format identifier for compression, archives, documents, images, and encodings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FormatId {
    // Single-stream compression formats
    Brotli,
    Gzip,
    Deflate,
    Zlib,
    ScoCompress,
    CompressLzw,
    CompressLzw2,
    CompressLzw1,
    Pack,
    OldPack,
    Compact,

    // Container / Archive formats
    Tar,
    Zip,

    // Document / Image / Data formats
    Html,
    Pan,
    Json,
    Markdown,
    Pdf,
    Pem,

    /// Unrecognized format.
    Unknown,
}

impl FormatId {
    /// Returns the primary format category for this format ID.
    pub fn category(&self) -> FormatCategory {
        match self {
            Self::Brotli
            | Self::Gzip
            | Self::Deflate
            | Self::Zlib
            | Self::ScoCompress
            | Self::CompressLzw
            | Self::CompressLzw2
            | Self::CompressLzw1
            | Self::Pack
            | Self::OldPack
            | Self::Compact => FormatCategory::Compression,

            Self::Tar | Self::Zip => FormatCategory::Archive,

            Self::Html | Self::Json | Self::Markdown | Self::Pdf | Self::Pem => FormatCategory::Document,

            Self::Pan => FormatCategory::Image,

            Self::Unknown => FormatCategory::Other,
        }
    }
}
