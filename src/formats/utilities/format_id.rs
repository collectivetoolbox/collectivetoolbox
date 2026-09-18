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

//! Standardized format identifier enum across all workspace format crates.
//! Not all of these are binary file formats, exactly.
//! Something like "`HtmlDceutils` & Html & Utf8 & `Lang_En_Us`" would more thoroughly describe a document format.

use crate::detection::FormatCategory;
#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use ctb_utilities::*;

/// Unified format identifier for compression, archives, documents, images, and encodings.
#[expect(
    non_camel_case_types,
    reason = "Language and format variants use underscores to reflect canonical format abbreviations"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FormatId {
    // Single-stream compression formats
    Brotli,
    Gzip,
    Deflate,
    Zlib,
    Bzip2,
    Bzip,
    ScoCompress,
    CompressLzw,
    CompressLzw2,
    CompressLzw1,
    CompressLzw16,
    Pack,
    OldPack,
    Compact,
    Lz4,
    Lzma,
    Lzma2,
    Lzip,
    Xz,
    Zstd,
    Lzo,

    // Binary Data Encodings
    BaseString,
    Base64, // There are different alphabets that can be used for Base64 and other base strings
    Hexadecimal,
    Hexadecimal0xPrefix,
    HexdumpPlain,
    HexdumpFancy437,
    Base16b,
    Base16b_7,
    Base16b_8,
    Base16b_9,
    Base16b_10,
    Base16b_11,
    Base16b_12,
    Base16b_13,
    Base16b_14,
    Base16b_15,
    Base16b_16,
    Base16b_17,
    BaseNb,
    BaseNb_7,
    BaseNb_8,
    BaseNb_9,
    BaseNb_10,
    BaseNb_11,
    BaseNb_12,
    BaseNb_13,
    BaseNb_14,
    BaseNb_15,
    BaseNb_16,
    BaseNb_17,

    // Text/Document Encodings
    String,
    CString,
    PascalString,
    Unicode, // Not a single format on its own
    Utf8, // Variants: unicode versions, BOM/no BOM, line ending variants (CR/LF/CRLF/the various others floating around), PUA extensions
    Ucs2,
    Wtf8,
    Utf8_32_BE,
    Cp437,
    MacRoman,
    // Line Endings and Separators (variants: v:lineEndings)
    LineEndingLf,
    LineEndingCr,
    LineEndingCrLf,
    LineEndingLfCr,
    LineEndingRs,
    LineEndingNl,
    LineSeparatorLf,
    LineSeparatorCr,
    LineSeparatorCrLf,
    LineSeparatorLfCr,
    LineSeparatorRs,
    LineSeparatorNl,
    // Modern CToolbox:
    DcText,
    DcUtf,
    DcList, // modern equivalent of DcArray, not a data format exactly, it's &[u128], maybe doesn't belong here
    // EITE:
    DcIntegerList, // short
    DcArray, // short, not a data format exactly, it's &[u32], maybe doesn't belong here
    Sems,
    Ascii,
    AsciiSafeSubset,
    EiteColorCoded,
    Elad,
    DcBasenb, // DcBasenb and DcBasenbFragment don't fully define a format on their own; they need a Unicode encoding to be embedded in
    DcBasenbFragment,
    // DCE:
    Cdce,
    CdceLegacy,
    CdceLegacyStrict,
    Dce,
    DceHex,
    Dce_3_0a,
    Dce_3_0a_raw,
    Dce_3_0a_hex,
    Dce_3_0a_raw_hex,
    Dce_3_0a_old,
    Dce2txt,
    Dce2hex,
    Hex2dce,
    Dce_3_01a,
    Dce_3_01a_raw,
    Dce_3_01a_hex,
    Dce_3_01a_raw_hex,
    DcIdList,
    Utf8_Base64,
    Utf8_Dc64,
    Utf8_Dc64_enc,
    Utf8_Dc64_bin,
    Utf8_Dc64_bin_hex,
    Utf8_Dc64_bin_enc,
    Utf8_Dc64_bin_enc_hex,
    HtmlDceutils,
    HtmlSnippetDceutils,
    HtmlLegacyCdce,
    HtmlLegacyCdceSnippet,

    // Programming Languages (can have different source text encodings)
    ActionScript,
    AppleScript,
    JavaScript,
    TypeScript,
    Bash,
    C,
    Cpp,
    Cs, // C#
    Java,
    Perl,
    Perl5,
    Perl6,
    Raku,
    Rust,
    Php,
    Sh, // shell
    StageL,
    StageLParseResult,

    // Software executables and packages
    Elf,
    Pe,
    MachO,
    DebianPackage,

    // Languages
    Lang_Ar,
    Lang_Bn,
    Lang_De,
    Lang_En,
    Lang_En_Gb,
    Lang_En_Us,
    Lang_Es,
    Lang_Fa,
    Lang_Fil,
    Lang_Fr,
    Lang_Hi,
    Lang_Id,
    Lang_It,
    Lang_Ja,
    Lang_Ko,
    Lang_Nl,
    Lang_Pt,
    Lang_Pt_Br,
    Lang_Ru,
    Lang_Tr,
    Lang_Ur,
    Lang_Vi,
    Lang_Zh_Cn,

    // Container / Archive formats
    Tar,
    Zip,
    CtbAssetBundle,
    AppleSingle,
    AppleDouble,

    // Document / Image / Data formats
    Html,
    HtmlFragment,
    Pdf,
    Markdown,
    Troff,
    Rss_09,
    Rss_10,
    Rss_20,
    Rss_091,
    Rss_092,
    ScriptingNews_10,
    ScriptingNews_20,
    Atom,
    HAtom, // a type of HTML document
    JsonFeed11,
    WfscanOutput,
    WfparseOutput,

    // Tables, data, and databases
    Csv,
    Tsv,
    Multipart,
    Json,
    Jsonc,
    Xml,
    Warc,
    Pan,
    Sqlite,
    IaMetaJson,
    IaFilesXml,
    IaMetaXml,
    IaMetaSqlite,
    Clubcard,
    CrliteFilter,
    CrliteFilterDelta,
    CommonLogFormat,
    ExtendedLogFormat,

    // Numbers
    Integer, // Needs an encoding to define a binary format
    Natural,
    Positive,
    Negative,
    U32,
    I32,
    U64,
    I64,
    U128,
    I128,
    Float,
    Fixed,
    Pack32,

    // Dates
    Gregorian, // Calendars, which would indicate the interpretation of other formats
    Julian, // Calendars, which would indicate the interpretation of other formats
    DateString, // There's a pretty much endless variety of these. Needs text encoding to define a binary format
    TimeString,
    DateTimeString,
    PanDate, // Implies Integer; needs some integer encoding to define a binary format
    PanTime,
    PanSuperDate,

    // Mathematics
    MathExpression, // Needs text encoding to define a binary format

    // Certificate
    Pem,

    // Names and Addresses
    IpAddr,
    Uri,
    FileName,
    FilePath,
    Lnk,
    MacAlias,
    MacBookmark,
    Torrent,
    Btih,
    Magnet,
    IaIdentifier,
    IaArchiveTorrent,

    // Checksums and hashes
    XxHash32,
    XxHash64,
    XxHash3_64,
    XxHash3_128,
    Md5,
    Md6,
    Sha1,
    Sha256,
    Sha3,
    Tiger,
    Whirlpool,
    Adler32,
    Fletcher4,
    Fletcher8,
    Fletcher16,
    Fletcher32,
    Crc32,
    Cksum,
    BsdChecksum,
    SysvChecksum,
    Hmac,
    Blake,  // families of checksums
    Blake2, // families of checksums
    Blake3, // families of checksums

    // Identifiers(?) Not sure what to call these
    UserAgent,
    FileExtension,
    UriProtocol,

    // Terminals, teletypes, etc. and their formats and features
    Terminal,
    Teletype,      // can't erase/blank
    Videoterminal, // can erase, blank, etc.
    Vt100,         // tons of different variations between terminals
    TerminalMouse,
    TerminalGraphics,
    TerminalSixelGraphics,
    TerminalIterm2Graphics,
    TerminalKittyGraphics,
    RasterDisplay,
    VectorDisplay,

    // Transformation filters - they're configurations that can be added to a format when exporting it in classic EITE, and I'll probably want to add new ones.
    SemanticToText,
    CodeToText,

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
            | Self::Bzip2
            | Self::Bzip
            | Self::ScoCompress
            | Self::CompressLzw
            | Self::CompressLzw2
            | Self::CompressLzw1
            | Self::CompressLzw16
            | Self::Pack
            | Self::OldPack
            | Self::Compact
            | Self::Lz4
            | Self::Lzma
            | Self::Lzma2
            | Self::Lzip
            | Self::Xz
            | Self::Zstd
            | Self::Lzo => FormatCategory::Compression,

            Self::Tar
            | Self::Zip
            | Self::CtbAssetBundle
            | Self::AppleSingle
            | Self::AppleDouble => FormatCategory::Archive,

            Self::Html
            | Self::Json
            | Self::Markdown
            | Self::Pdf
            | Self::Pem
            | Self::Perl => FormatCategory::Document,

            _ => FormatCategory::Other,
        }
    }

    /// Looks up a `FormatId` variant from its identifier name.
    #[must_use]
    pub fn from_ident(ident: &str) -> Option<Self> {
        let trimmed = ident.trim();
        // Case-insensitive ASCII lookup
        match trimmed.to_ascii_lowercase().as_str() {
            "brotli" => Some(Self::Brotli),
            "gzip" => Some(Self::Gzip),
            "deflate" => Some(Self::Deflate),
            "zlib" => Some(Self::Zlib),
            "bzip2" => Some(Self::Bzip2),
            "bzip" => Some(Self::Bzip),
            "scocompress" | "sco_compress" => Some(Self::ScoCompress),
            "compresslzw" | "compress_lzw" => Some(Self::CompressLzw),
            "compresslzw2" | "compress_lzw2" => Some(Self::CompressLzw2),
            "compresslzw1" | "compress_lzw1" => Some(Self::CompressLzw1),
            "compresslzw16" | "compress_lzw16" => Some(Self::CompressLzw16),
            "pack" => Some(Self::Pack),
            "oldpack" | "old_pack" => Some(Self::OldPack),
            "compact" => Some(Self::Compact),
            "lz4" => Some(Self::Lz4),
            "lzma" => Some(Self::Lzma),
            "lzma2" => Some(Self::Lzma2),
            "lzip" => Some(Self::Lzip),
            "xz" => Some(Self::Xz),
            "zstd" => Some(Self::Zstd),
            "lzo" => Some(Self::Lzo),

            "tar" => Some(Self::Tar),
            "zip" => Some(Self::Zip),
            "ctbassetbundle" | "ctb_asset_bundle" => Some(Self::CtbAssetBundle),
            "applesingle" | "apple_single" => Some(Self::AppleSingle),
            "appledouble" | "apple_double" => Some(Self::AppleDouble),

            "html" => Some(Self::Html),
            "htmlfragment" | "html_fragment" => Some(Self::HtmlFragment),
            "pdf" => Some(Self::Pdf),
            "markdown" => Some(Self::Markdown),
            "troff" => Some(Self::Troff),
            "csv" => Some(Self::Csv),
            "tsv" => Some(Self::Tsv),
            "json" => Some(Self::Json),
            "xml" => Some(Self::Xml),
            "warc" => Some(Self::Warc),
            "pan" => Some(Self::Pan),
            "sqlite" => Some(Self::Sqlite),
            "pem" => Some(Self::Pem),
            "elf" => Some(Self::Elf),
            "pe" => Some(Self::Pe),
            "macho" | "mach_o" | "mach-o" => Some(Self::MachO),
            "debianpackage" | "debian_package" => Some(Self::DebianPackage),
            "lnk" => Some(Self::Lnk),
            "torrent" => Some(Self::Torrent),
            "actionscript" | "action_script" => Some(Self::ActionScript),
            "applescript" => Some(Self::AppleScript),
            "javascript" => Some(Self::JavaScript),
            "typescript" => Some(Self::TypeScript),
            "rust" => Some(Self::Rust),
            "c" => Some(Self::C),
            "cpp" => Some(Self::Cpp),
            "perl" => Some(Self::Perl),
            "sh" => Some(Self::Sh),
            "bash" => Some(Self::Bash),
            "utf8" => Some(Self::Utf8),
            "ascii" => Some(Self::Ascii),
            "unknown" => Some(Self::Unknown),
            _ => None,
        }
    }

    /// Returns the canonical Rust identifier string for this format.
    #[must_use]
    pub const fn ident(&self) -> &'static str {
        match self {
            Self::Brotli => "Brotli",
            Self::Gzip => "Gzip",
            Self::Deflate => "Deflate",
            Self::Zlib => "Zlib",
            Self::Bzip2 => "Bzip2",
            Self::Bzip => "Bzip",
            Self::ScoCompress => "ScoCompress",
            Self::CompressLzw => "CompressLzw",
            Self::CompressLzw2 => "CompressLzw2",
            Self::CompressLzw1 => "CompressLzw1",
            Self::CompressLzw16 => "CompressLzw16",
            Self::Pack => "Pack",
            Self::OldPack => "OldPack",
            Self::Compact => "Compact",
            Self::Lz4 => "Lz4",
            Self::Lzma => "Lzma",
            Self::Lzma2 => "Lzma2",
            Self::Lzip => "Lzip",
            Self::Xz => "Xz",
            Self::Zstd => "Zstd",
            Self::Lzo => "Lzo",

            Self::Tar => "Tar",
            Self::Zip => "Zip",
            Self::CtbAssetBundle => "CtbAssetBundle",
            Self::AppleSingle => "AppleSingle",
            Self::AppleDouble => "AppleDouble",

            Self::Html => "Html",
            Self::HtmlFragment => "HtmlFragment",
            Self::Pdf => "Pdf",
            Self::Markdown => "Markdown",
            Self::Troff => "Troff",
            Self::Csv => "Csv",
            Self::Tsv => "Tsv",
            Self::Json => "Json",
            Self::Xml => "Xml",
            Self::Warc => "Warc",
            Self::Pan => "Pan",
            Self::Sqlite => "Sqlite",
            Self::Pem => "Pem",
            Self::Elf => "Elf",
            Self::Pe => "Pe",
            Self::MachO => "MachO",
            Self::DebianPackage => "DebianPackage",
            Self::Lnk => "Lnk",
            Self::Torrent => "Torrent",
            Self::ActionScript => "ActionScript",
            Self::AppleScript => "AppleScript",
            Self::JavaScript => "JavaScript",
            Self::TypeScript => "TypeScript",
            Self::Rust => "Rust",
            Self::C => "C",
            Self::Cpp => "Cpp",
            Self::Perl => "Perl",
            Self::Sh => "Sh",
            Self::Bash => "Bash",
            Self::Utf8 => "Utf8",
            Self::Ascii => "Ascii",
            _ => "Unknown",
        }
    }
}
