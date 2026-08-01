//! Standardized format identifier enum across all workspace format crates.
//! Not all of these are binary file formats, exactly.
//! Something like "`HtmlDceutils` & Html & Utf8 & `Lang_En_Us`" would more thoroughly describe a document format.

use crate::detection::FormatCategory;
#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use ctb_utilities::*;

/// Unified format identifier for compression, archives, documents, images, and encodings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FormatId {
    // Single-stream compression formats
    Brotli,
    Gzip,
    Deflate,
    Zlib,
    Bzip2,
    ScoCompress,
    CompressLzw,
    CompressLzw2,
    CompressLzw1,
    CompressLzw16,
    Pack,
    OldPack,
    Compact,

    // Binary Data Encodings
    BaseString,
    Base64, // There are different alphabets that can be used for Base64 and other base strings
    Hexadecimal,
    Hexadecimal0xPrefix,
    HexdumpClassic,
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
    // Modern CToolbox:
    DcText,
    DcUtf,
    DcList, // modern equivalent of DcArray, not a data format exactly, it's &[u128], maybe doesn't belong here
    // EITE:
    DcIntegerList, // classic
    DcArray, // classic, not a data format exactly, it's &[u32], maybe doesn't belong here
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
            | Self::ScoCompress
            | Self::CompressLzw
            | Self::CompressLzw2
            | Self::CompressLzw1
            | Self::Pack
            | Self::OldPack
            | Self::Compact => FormatCategory::Compression,

            Self::Tar | Self::Zip => FormatCategory::Archive,

            Self::Html
            | Self::Json
            | Self::Markdown
            | Self::Pdf
            | Self::Pem
            | Self::Perl => FormatCategory::Document,

            _ => FormatCategory::Other,
        }
    }
}
