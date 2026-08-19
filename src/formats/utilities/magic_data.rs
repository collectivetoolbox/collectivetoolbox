// SPDX-License-Identifier: AGPL-3.0-or-later
/*
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
static BZIP_MAGIC: MagicPattern = MagicPattern::exact(&[0x42, 0x5A, 0x30]);
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
static XZ_MAGIC: MagicPattern =
    MagicPattern::exact(&[0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00]);
static LZIP_MAGIC: MagicPattern = MagicPattern::exact(&[0x4C, 0x5A, 0x49, 0x50]);
static ZSTD_MAGIC: MagicPattern =
    MagicPattern::exact(&[0x28, 0xB5, 0x2F, 0xFD]);
static LZ4_MAGIC: MagicPattern =
    MagicPattern::exact(&[0x04, 0x22, 0x4D, 0x18]);
static LZO_MAGIC: MagicPattern = MagicPattern::exact(&[
    0x89, 0x4C, 0x5A, 0x4F, 0x00, 0x0D, 0x0A, 0x1A, 0x0A,
]);

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
        format_id: FormatId::Bzip,
        pattern: BZIP_MAGIC,
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
    MagicEntry {
        format_id: FormatId::Xz,
        pattern: XZ_MAGIC,
    },
    MagicEntry {
        format_id: FormatId::Lzip,
        pattern: LZIP_MAGIC,
    },
    MagicEntry {
        format_id: FormatId::Zstd,
        pattern: ZSTD_MAGIC,
    },
    MagicEntry {
        format_id: FormatId::Lz4,
        pattern: LZ4_MAGIC,
    },
    MagicEntry {
        format_id: FormatId::Lzo,
        pattern: LZO_MAGIC,
    },
];
