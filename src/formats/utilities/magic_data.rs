//! Centralized registry of magic header signatures across format types.

use crate::format_id::FormatId;
use crate::magic::MagicPattern;
#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use ctb_utilities::*;

/// An entry associating a `FormatId` with a `MagicPattern`.
#[derive(Debug, Clone)]
pub struct MagicEntry {
    pub format_id: FormatId,
    pub pattern: MagicPattern,
}

static GZIP_MAGIC: MagicPattern = MagicPattern::exact(&[0x1F, 0x8B]);
static BZIP2_MAGIC: MagicPattern = MagicPattern::exact(&[0x42, 0x5A, 0x68]);
static SCO_MAGIC: MagicPattern = MagicPattern::exact(&[0x1F, 0xA0]);
static LZW_MAGIC: MagicPattern =
    MagicPattern::masked(&[0x1F, 0x9D, 0x80], &[0xFF, 0xFF, 0x80], 110);
static LZW2_MAGIC: MagicPattern =
    MagicPattern::masked(&[0x1F, 0x9D, 0x00], &[0xFF, 0xFF, 0x80], 90);
static PACK_MAGIC: MagicPattern = MagicPattern::exact(&[0x1F, 0x1E]);
static OLD_PACK_MAGIC: MagicPattern = MagicPattern::exact(&[0x1F, 0x1F]);
static COMPACT_MAGIC_LE: MagicPattern = MagicPattern::exact(&[0xFF, 0x1F]);
static COMPACT_MAGIC_BE: MagicPattern = MagicPattern::exact(&[0x1F, 0xFF]);
static ZLIB_MAGIC: MagicPattern =
    MagicPattern::masked(&[0x78, 0x00], &[0xFF, 0x00], 80);

/// Global static table of all magic header signatures.
pub static MAGIC_REGISTRY: &[MagicEntry] = &[
    MagicEntry {
        format_id: FormatId::Gzip,
        pattern: GZIP_MAGIC,
    },
    MagicEntry {
        format_id: FormatId::Bzip2,
        pattern: BZIP2_MAGIC,
    },
    MagicEntry {
        format_id: FormatId::ScoCompress,
        pattern: SCO_MAGIC,
    },
    MagicEntry {
        format_id: FormatId::CompressLzw,
        pattern: LZW_MAGIC,
    },
    MagicEntry {
        format_id: FormatId::CompressLzw2,
        pattern: LZW2_MAGIC,
    },
    MagicEntry {
        format_id: FormatId::Pack,
        pattern: PACK_MAGIC,
    },
    MagicEntry {
        format_id: FormatId::OldPack,
        pattern: OLD_PACK_MAGIC,
    },
    MagicEntry {
        format_id: FormatId::Compact,
        pattern: COMPACT_MAGIC_LE,
    },
    MagicEntry {
        format_id: FormatId::Compact,
        pattern: COMPACT_MAGIC_BE,
    },
    MagicEntry {
        format_id: FormatId::Zlib,
        pattern: ZLIB_MAGIC,
    },
];
