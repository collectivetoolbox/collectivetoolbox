//! Centralized registry of file extension rules across format types.

use crate::extension::ExtensionRule;
use crate::format_id::FormatId;
#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use ctb_utilities::*;

/// An entry associating a `FormatId` with an `ExtensionRule`.
#[derive(Debug, Clone)]
pub struct ExtensionEntry {
    pub format_id: FormatId,
    pub rule: ExtensionRule,
}

static EXT_BR: ExtensionRule = ExtensionRule::insensitive("br");
static EXT_GZ: ExtensionRule = ExtensionRule::insensitive("gz");
static EXT_GZIP: ExtensionRule = ExtensionRule::insensitive("gzip");
static EXT_DEFLATE: ExtensionRule = ExtensionRule::insensitive("deflate");
static EXT_ZZ: ExtensionRule = ExtensionRule::insensitive("zz");
static EXT_ZL: ExtensionRule = ExtensionRule::insensitive("zl");
static EXT_BZ2: ExtensionRule = ExtensionRule::insensitive("bz2");
static EXT_BZIP2: ExtensionRule = ExtensionRule::insensitive("bzip2");
static EXT_BZ: ExtensionRule = ExtensionRule::insensitive("bz");
static EXT_BZIP: ExtensionRule = ExtensionRule::insensitive("bzip");
static EXT_LZ4: ExtensionRule = ExtensionRule::insensitive("lz4");
static EXT_LZMA: ExtensionRule = ExtensionRule::insensitive("lzma");
static EXT_LZMA2: ExtensionRule = ExtensionRule::insensitive("lzma2");
static EXT_LZ: ExtensionRule = ExtensionRule::insensitive("lz");
static EXT_LZIP: ExtensionRule = ExtensionRule::insensitive("lzip");
static EXT_XZ: ExtensionRule = ExtensionRule::insensitive("xz");
static EXT_XZIP: ExtensionRule = ExtensionRule::insensitive("xzip");
static EXT_ZST: ExtensionRule = ExtensionRule::insensitive("zst");
static EXT_ZSTD: ExtensionRule = ExtensionRule::insensitive("zstd");
static EXT_LZO: ExtensionRule = ExtensionRule::insensitive("lzo");

static EXT_UPPER_Z: ExtensionRule = ExtensionRule::sensitive("Z");
static EXT_LOWER_Z: ExtensionRule = ExtensionRule::sensitive("z");
static EXT_UPPER_C: ExtensionRule = ExtensionRule::sensitive("C");

static EXT_HTML: ExtensionRule = ExtensionRule::insensitive("html");
static EXT_HTM: ExtensionRule = ExtensionRule::insensitive("htm");
static EXT_PAN: ExtensionRule = ExtensionRule::insensitive("pan");
static EXT_JSON: ExtensionRule = ExtensionRule::insensitive("json");
static EXT_MD: ExtensionRule = ExtensionRule::insensitive("md");
static EXT_TAR: ExtensionRule = ExtensionRule::insensitive("tar");

/// Global static registry of single file extension mappings.
pub static EXTENSION_REGISTRY: &[ExtensionEntry] = &[
    ExtensionEntry {
        format_id: FormatId::Gzip,
        rule: EXT_GZ,
    },
    ExtensionEntry {
        format_id: FormatId::Gzip,
        rule: EXT_GZIP,
    },
    ExtensionEntry {
        format_id: FormatId::Brotli,
        rule: EXT_BR,
    },
    ExtensionEntry {
        format_id: FormatId::Deflate,
        rule: EXT_DEFLATE,
    },
    ExtensionEntry {
        format_id: FormatId::Zlib,
        rule: EXT_ZZ,
    },
    ExtensionEntry {
        format_id: FormatId::Zlib,
        rule: EXT_ZL,
    },
    ExtensionEntry {
        format_id: FormatId::Bzip2,
        rule: EXT_BZ2,
    },
    ExtensionEntry {
        format_id: FormatId::Bzip2,
        rule: EXT_BZIP2,
    },
    ExtensionEntry {
        format_id: FormatId::Bzip,
        rule: EXT_BZ,
    },
    ExtensionEntry {
        format_id: FormatId::Bzip,
        rule: EXT_BZIP,
    },
    ExtensionEntry {
        format_id: FormatId::ScoCompress,
        rule: EXT_UPPER_Z,
    },
    ExtensionEntry {
        format_id: FormatId::CompressLzw,
        rule: EXT_UPPER_Z,
    },
    ExtensionEntry {
        format_id: FormatId::Pack,
        rule: EXT_LOWER_Z,
    },
    ExtensionEntry {
        format_id: FormatId::OldPack,
        rule: EXT_LOWER_Z,
    },
    ExtensionEntry {
        format_id: FormatId::Compact,
        rule: EXT_UPPER_C,
    },
    ExtensionEntry {
        format_id: FormatId::Lz4,
        rule: EXT_LZ4,
    },
    ExtensionEntry {
        format_id: FormatId::Lzma,
        rule: EXT_LZMA,
    },
    ExtensionEntry {
        format_id: FormatId::Lzma2,
        rule: EXT_LZMA,
    },
    ExtensionEntry {
        format_id: FormatId::Lzma2,
        rule: EXT_LZMA2,
    },
    ExtensionEntry {
        format_id: FormatId::Lzip,
        rule: EXT_LZ,
    },
    ExtensionEntry {
        format_id: FormatId::Lzip,
        rule: EXT_LZIP,
    },
    ExtensionEntry {
        format_id: FormatId::Xz,
        rule: EXT_XZ,
    },
    ExtensionEntry {
        format_id: FormatId::Xz,
        rule: EXT_XZIP,
    },
    ExtensionEntry {
        format_id: FormatId::Zstd,
        rule: EXT_ZST,
    },
    ExtensionEntry {
        format_id: FormatId::Zstd,
        rule: EXT_ZSTD,
    },
    ExtensionEntry {
        format_id: FormatId::Lzo,
        rule: EXT_LZO,
    },
    ExtensionEntry {
        format_id: FormatId::Html,
        rule: EXT_HTML,
    },
    ExtensionEntry {
        format_id: FormatId::Html,
        rule: EXT_HTM,
    },
    ExtensionEntry {
        format_id: FormatId::Pan,
        rule: EXT_PAN,
    },
    ExtensionEntry {
        format_id: FormatId::Json,
        rule: EXT_JSON,
    },
    ExtensionEntry {
        format_id: FormatId::Markdown,
        rule: EXT_MD,
    },
    ExtensionEntry {
        format_id: FormatId::Tar,
        rule: EXT_TAR,
    },
];

/// Helper to lookup `FormatId` from a single extension string.
pub fn lookup_format_by_extension(ext: &str) -> Vec<FormatId> {
    let mut matches = Vec::new();
    for entry in EXTENSION_REGISTRY {
        if entry.rule.matches(ext) && !matches.contains(&entry.format_id) {
            matches.push(entry.format_id);
        }
    }
    matches
}
