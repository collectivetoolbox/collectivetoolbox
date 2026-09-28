// SPDX-License-Identifier: AGPL-3.0-or-later AND GPL-2.0-or-later AND BSD-3-Clause AND LGPL-2.1-or-later
// SPDX-License-Identifier for parts derived from rzip: GPL-2.0-or-later
// SPDX-License-Identifier for parts derived from glibc: BSD-3-Clause AND LGPL-2.1-or-later
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

// See license note at end of this file for parts derived from rzip and glibc (the random number generator in test_regenerate_hash_index_table is the part from glibc).

//! Implementation of the `rzip` 2.1 compression format.
//!
//! Faithful safe Rust port from Andrew Tridgell and Rusty Russell's C
//! implementation in `old/unix-tools/rzip-2.1/`.
//!
//! rzip uses a two-stage compression approach:
//! 1. Long-distance redundancy elimination across large chunks (up to 100MB+),
//!    emitting match offsets and literals split across two streams:
//!    - Stream 0: control stream (literal headers, match headers with offsets,
//!      EOF sentinel, and CRC-32 checksum).
//!    - Stream 1: raw literal bytes.
//! 2. Second-stage bzip2 compression and block-level interleaving with
//!    forward-linked 13-byte chunk headers.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;
use anyhow::{bail, ensure, Context, Result};
use std::io::{Cursor, Read, Seek, SeekFrom, Write};

const RZIP_MAGIC: [u8; 4] = *b"RZIP";
const RZIP_MAJOR: u8 = 2;
const RZIP_MINOR: u8 = 1;
const MAGIC_HEADER_SIZE: usize = 24;

const NUM_STREAMS: usize = 2;
const STREAM_BLOCK_HEADER_SIZE: usize = 13;

const MINIMUM_MATCH: usize = 31;
const GREAT_MATCH: usize = 1024;

const CTYPE_NONE: u8 = 3;
const CTYPE_BZIP2: u8 = 4;

const CHUNK_MULTIPLE: usize = 100 * 1024 * 1024;

/// Compression level parameters controlling hashtable size and bzip2 level.
#[derive(Debug, Clone, Copy)]
struct CompressionLevel {
    bzip_level: u32,
    mb_used: usize,
    initial_freq: u32,
    max_chain_len: usize,
}

const LEVELS: [CompressionLevel; 10] = [
    CompressionLevel { bzip_level: 0, mb_used: 1, initial_freq: 4, max_chain_len: 1 },
    CompressionLevel { bzip_level: 1, mb_used: 2, initial_freq: 4, max_chain_len: 2 },
    CompressionLevel { bzip_level: 3, mb_used: 4, initial_freq: 4, max_chain_len: 2 },
    CompressionLevel { bzip_level: 5, mb_used: 8, initial_freq: 4, max_chain_len: 2 },
    CompressionLevel { bzip_level: 7, mb_used: 16, initial_freq: 4, max_chain_len: 3 },
    CompressionLevel { bzip_level: 9, mb_used: 32, initial_freq: 4, max_chain_len: 4 },
    CompressionLevel { bzip_level: 9, mb_used: 32, initial_freq: 2, max_chain_len: 6 },
    CompressionLevel { bzip_level: 9, mb_used: 64, initial_freq: 1, max_chain_len: 16 },
    CompressionLevel { bzip_level: 9, mb_used: 64, initial_freq: 1, max_chain_len: 32 },
    CompressionLevel { bzip_level: 9, mb_used: 64, initial_freq: 1, max_chain_len: 128 },
];

/// Table generated using Mark Adler's CRC-32 polynomial algorithm (0xEDB88320).
pub const CRC_TABLE: [u32; 256] = [
    0x0000_0000, 0x7707_3096, 0xEE0E_612C, 0x9909_51BA, 0x076D_C419,
    0x706A_F48F, 0xE963_A535, 0x9E64_95A3, 0x0EDB_8832, 0x79DC_B8A4,
    0xE0D5_E91E, 0x97D2_D988, 0x09B6_4C2B, 0x7EB1_7CBD, 0xE7B8_2D07,
    0x90BF_1D91, 0x1DB7_1064, 0x6AB0_20F2, 0xF3B9_7148, 0x84BE_41DE,
    0x1ADA_D47D, 0x6DDD_E4EB, 0xF4D4_B551, 0x83D3_85C7, 0x136C_9856,
    0x646B_A8C0, 0xFD62_F97A, 0x8A65_C9EC, 0x1401_5C4F, 0x6306_6CD9,
    0xFA0F_3D63, 0x8D08_0DF5, 0x3B6E_20C8, 0x4C69_105E, 0xD560_41E4,
    0xA267_7172, 0x3C03_E4D1, 0x4B04_D447, 0xD20D_85FD, 0xA50A_B56B,
    0x35B5_A8FA, 0x42B2_986C, 0xDBBB_C9D6, 0xACBC_F940, 0x32D8_6CE3,
    0x45DF_5C75, 0xDCD6_0DCF, 0xABD1_3D59, 0x26D9_30AC, 0x51DE_003A,
    0xC8D7_5180, 0xBFD0_6116, 0x21B4_F4B5, 0x56B3_C423, 0xCFBA_9599,
    0xB8BD_A50F, 0x2802_B89E, 0x5F05_8808, 0xC60C_D9B2, 0xB10B_E924,
    0x2F6F_7C87, 0x5868_4C11, 0xC161_1DAB, 0xB666_2D3D, 0x76DC_4190,
    0x01DB_7106, 0x98D2_20BC, 0xEFD5_102A, 0x71B1_8589, 0x06B6_B51F,
    0x9FBF_E4A5, 0xE8B8_D433, 0x7807_C9A2, 0x0F00_F934, 0x9609_A88E,
    0xE10E_9818, 0x7F6A_0DBB, 0x086D_3D2D, 0x9164_6C97, 0xE663_5C01,
    0x6B6B_51F4, 0x1C6C_6162, 0x8565_30D8, 0xF262_004E, 0x6C06_95ED,
    0x1B01_A57B, 0x8208_F4C1, 0xF50F_C457, 0x65B0_D9C6, 0x12B7_E950,
    0x8BBE_B8EA, 0xFCB9_887C, 0x62DD_1DDF, 0x15DA_2D49, 0x8CD3_7CF3,
    0xFBD4_4C65, 0x4DB2_6158, 0x3AB5_51CE, 0xA3BC_0074, 0xD4BB_30E2,
    0x4ADF_A541, 0x3DD8_95D7, 0xA4D1_C46D, 0xD3D6_F4FB, 0x4369_E96A,
    0x346E_D9FC, 0xAD67_8846, 0xDA60_B8D0, 0x4404_2D73, 0x3303_1DE5,
    0xAA0A_4C5F, 0xDD0D_7CC9, 0x5005_713C, 0x2702_41AA, 0xBE0B_1010,
    0xC90C_2086, 0x5768_B525, 0x206F_85B3, 0xB966_D409, 0xCE61_E49F,
    0x5EDE_F90E, 0x29D9_C998, 0xB0D0_9822, 0xC7D7_A8B4, 0x59B3_3D17,
    0x2EB4_0D81, 0xB7BD_5C3B, 0xC0BA_6CAD, 0xEDB8_8320, 0x9ABF_B3B6,
    0x03B6_E20C, 0x74B1_D29A, 0xEAD5_4739, 0x9DD2_77AF, 0x04DB_2615,
    0x73DC_1683, 0xE363_0B12, 0x9464_3B84, 0x0D6D_6A3E, 0x7A6A_5AA8,
    0xE40E_CF0B, 0x9309_FF9D, 0x0A00_AE27, 0x7D07_9EB1, 0xF00F_9344,
    0x8708_A3D2, 0x1E01_F268, 0x6906_C2FE, 0xF762_575D, 0x8065_67CB,
    0x196C_3671, 0x6E6B_06E7, 0xFED4_1B76, 0x89D3_2BE0, 0x10DA_7A5A,
    0x67DD_4ACC, 0xF9B9_DF6F, 0x8EBE_EFF9, 0x17B7_BE43, 0x60B0_8ED5,
    0xD6D6_A3E8, 0xA1D1_937E, 0x38D8_C2C4, 0x4FDF_F252, 0xD1BB_67F1,
    0xA6BC_5767, 0x3FB5_06DD, 0x48B2_364B, 0xD80D_2BDA, 0xAF0A_1B4C,
    0x3603_4AF6, 0x4104_7A60, 0xDF60_EFC3, 0xA867_DF55, 0x316E_8EEF,
    0x4669_BE79, 0xCB61_B38C, 0xBC66_831A, 0x256F_D2A0, 0x5268_E236,
    0xCC0C_7795, 0xBB0B_4703, 0x2202_16B9, 0x5505_262F, 0xC5BA_3BBE,
    0xB2BD_0B28, 0x2BB4_5A92, 0x5CB3_6A04, 0xC2D7_FFA7, 0xB5D0_CF31,
    0x2CD9_9E8B, 0x5BDE_AE1D, 0x9B64_C2B0, 0xEC63_F226, 0x756A_A39C,
    0x026D_930A, 0x9C09_06A9, 0xEB0E_363F, 0x7207_6785, 0x0500_5713,
    0x95BF_4A82, 0xE2B8_7A14, 0x7BB1_2BAE, 0x0CB6_1B38, 0x92D2_8E9B,
    0xE5D5_BE0D, 0x7CDC_EFB7, 0x0BDB_DF21, 0x86D3_D2D4, 0xF1D4_E242,
    0x68DD_B3F8, 0x1FDA_836E, 0x81BE_16CD, 0xF6B9_265B, 0x6FB0_77E1,
    0x18B7_4777, 0x8808_5AE6, 0xFF0F_6A70, 0x6606_3BCA, 0x1101_0B5C,
    0x8F65_9EFF, 0xF862_AE69, 0x616B_FFD3, 0x166C_CF45, 0xA00A_E278,
    0xD70D_D2EE, 0x4E04_8354, 0x3903_B3C2, 0xA767_2661, 0xD060_16F7,
    0x4969_474D, 0x3E6E_77DB, 0xAED1_6A4A, 0xD9D6_5ADC, 0x40DF_0B66,
    0x37D8_3BF0, 0xA9BC_AE53, 0xDEBB_9EC5, 0x47B2_CF7F, 0x30B5_FFE9,
    0xBDBD_F21C, 0xCABA_C28A, 0x53B3_9330, 0x24B4_A3A6, 0xBAD0_3605,
    0xCDD7_0693, 0x54DE_5729, 0x23D9_67BF, 0xB366_7A2E, 0xC461_4AB8,
    0x5D68_1B02, 0x2A6F_2B94, 0xB40B_BE37, 0xC30C_8EA1, 0x5A05_DF1B,
    0x2D02_EF8D,
];

/// Updates a running CRC-32 with buffer data (initial value 0, no final !).
///
/// Computes Mark Adler's CRC-32 using `crc32fast::Hasher` with `new_with_initial(!crc)`
/// and inverted finalize (`!`), matching rzip 2.1's algorithm with SIMD acceleration.
#[inline]
fn update_crc32(crc: u32, buf: &[u8]) -> u32 {
    let mut hasher = crc32fast::Hasher::new_with_initial(!crc);
    hasher.update(buf);
    !hasher.finalize()
}

/// Precomputed 256-entry hash index table matching glibc default `random()`.
/// Upstream doesn't use a const table. LLM stated that it means it's not
/// consistent across platforms, while using a precomputed table is; which
/// sounds plausible but I haven't verified.
const HASH_INDEX: [u32; 256] = [
    0x771C_23C6, 0xFE5A_4873, 0xC518_5CFF, 0xF61F_58EC, 0x59C1_7CCD,
    0x08C4_D7AB, 0x0045_1EFB, 0xDCA6_E146, 0x5BAC_62C2, 0x45E5_27F8,
    0x3C0D_E9E8, 0xAB08_438D, 0x3C24_255A, 0xF4C3_7263, 0xD9DC_D79F,
    0xAFA1_079A, 0xDE82_5D32, 0x1316_D7B7, 0xC6B2_E458, 0x5218_D95A,
    0x03A9_895D, 0xC318_A317, 0x8545_5AE9, 0xF1FC_A8D4, 0xC428_8CB2,
    0xDBCE_E0C6, 0x227F_9EB4, 0x08FC_8611, 0xC69E_1D82, 0xF067_8641,
    0x891C_BD3D, 0xE136_F087, 0x5F66_DDE9, 0xA971_D4A1, 0x9736_F8E1,
    0xA8FB_2367, 0xF578_5F01, 0x9A36_2A97, 0xE762_4ADC, 0xFE3A_7796,
    0xC2B1_A438, 0x28FA_4E2A, 0x68A4_7CB0, 0xA661_06FB, 0xCDD6_CCAF,
    0xC664_8F54, 0xE405_1B18, 0x099F_A45C, 0x9148_481A, 0x0C64_BB43,
    0x5B5B_26FA, 0xAB6C_C33A, 0xF53D_A529, 0xDA9B_3FE6, 0xD3FC_C13C,
    0x9026_C794, 0x7984_0FD8, 0x36E5_A861, 0xB9C6_E9F9, 0xDB4E_26BB,
    0xA68B_3C99, 0xD758_4095, 0x201F_35EB, 0xD5FD_50B3, 0x5135_5DEF,
    0x164D_BF00, 0x9BA1_EAA1, 0xDF8D_0AE5, 0xEA5A_700B, 0x1595_7FD0,
    0x48D5_0247, 0x05A6_96BD, 0x0CA5_5D23, 0x62D9_9EA8, 0x3CCF_EE7B,
    0xA45A_FDC5, 0x3A50_7B73, 0x3459_82C5, 0xAB6C_234B, 0xBB92_2F63,
    0xE18D_DF70, 0xFA57_0624, 0xD036_709E, 0x0106_59DC, 0xE641_5BD4,
    0xD0EE_11F2, 0x283C_2110, 0xBFB9_703B, 0x4C11_E7CD, 0x6608_C550,
    0x90FF_D447, 0xFAFB_015C, 0x437A_016F, 0xE4CD_0119, 0x5BBB_579B,
    0x4028_A5F5, 0x678B_1EE1, 0xF3A2_011C, 0xF90D_BD23, 0x30A7_7029,
    0x47CD_34A4, 0xFFFD_7713, 0x6A13_2ACA, 0x8D73_D3E8, 0x1B42_F632,
    0x8687_E8E0, 0xD72D_5C4D, 0xC2B2_1A34, 0x4023_6E5F, 0x4A46_8277,
    0x96A2_4BCB, 0x5434_FD05, 0x2F4A_D486, 0xF05E_FA2B, 0xDA72_591A,
    0x53E2_AAA2, 0xE13E_EC70, 0x21DC_E373, 0x5BA0_0904, 0xAAAC_29D3,
    0xC2E7_5094, 0x35BB_C9AF, 0xBBBF_FCF0, 0xE6DF_0A9E, 0x325A_FF32,
    0xDC8B_3149, 0x5D5C_B582, 0x7E08_B5A9, 0xA000_2C70, 0xB636_1BB2,
    0xD22C_1A29, 0xE6BA_1348, 0x0DAD_E80A, 0x86F2_1DD5, 0xFAF1_AE18,
    0x69E0_F044, 0xB542_5A5B, 0x9D05_AB8E, 0x73A5_9DD7, 0xAE64_C29B,
    0x9D7C_4342, 0xD988_E806, 0xCF1B_2233, 0xE830_82CD, 0x17D8_4D84,
    0xC9B8_D42D, 0x04B6_64D4, 0x923E_6E47, 0x56D3_DE32, 0xCA3A_3DEC,
    0xFF9E_D3C4, 0x41EE_8AF6, 0x67E0_E823, 0x158A_856C, 0xBAFD_ECB2,
    0x24AD_2304, 0x0AB5_3BEC, 0x5C98_28B9, 0x4FEC_A8BA, 0x9481_ACC3,
    0xAA8A_4A05, 0x9C98_5DEC, 0xA15C_6867, 0xE211_FBB7, 0x31D1_5850,
    0x7603_D2E3, 0x44AC_67D3, 0x7739_5A34, 0x1BA3_945E, 0xFE83_D5F2,
    0x8859_27A8, 0xAA21_B105, 0x4772_0401, 0xD214_C1B4, 0xDB21_8544,
    0x4F82_A2FA, 0xAEC8_EF69, 0x14C7_7E23, 0x07C5_CD1A, 0x6340_9E69,
    0x13E7_B37E, 0x4337_517E, 0x32EB_CF25, 0xFA0C_6B48, 0x48CF_8B53,
    0x5050_E494, 0xB24A_3A31, 0xF686_1690, 0x309F_6F57, 0x840D_EAEE,
    0xCDE7_46BC, 0xD775_C3E5, 0x0A8D_8ECF, 0x4E0D_8DF5, 0x6AEB_BBE2,
    0x7842_8153, 0x7FBB_9DAA, 0xD404_8AB2, 0x0F23_7E85, 0xCD5F_D054,
    0x42A1_3735, 0xF73B_BCD4, 0x5F35_4A82, 0xA2A5_AF98, 0xAC65_ABA8,
    0xD9CC_AE75, 0xBE35_2870, 0x186D_288A, 0xBB99_B462, 0xA337_1329,
    0x6C98_E2DE, 0xB145_DFA5, 0xF0E2_674E, 0x0219_ED59, 0xE175_6051,
    0xDE30_EFAC, 0xFC83_7295, 0xB0CA_08EC, 0xF373_7FE4, 0xEEDC_76F1,
    0x779B_530C, 0x163C_1DF1, 0x4CF1_97C0, 0x771C_32BB, 0x5FEF_FCFC,
    0x4439_BC66, 0xEF30_DA61, 0xC268_6063, 0x1287_0662, 0x3809_2783,
    0x0113_0B69, 0x30C2_FE3A, 0x7D3B_C3AF, 0x6B22_3B16, 0x11D0_BFAC,
    0x814D_6F1F, 0x4EBB_D76D, 0xDBD3_8E34, 0x6B18_D38D, 0x972A_BB4F,
    0x9FB3_06D4, 0x3F18_B063, 0xE18E_13C1, 0x31BC_22E4, 0x77E0_5E83,
    0x61D6_80D8, 0x196E_6196, 0x5705_E7EC, 0x3292_8639, 0x6373_68D8,
    0x139E_D50A, 0xCF8E_589D, 0xF617_6509, 0xA0C0_1BA5, 0x4FCD_A7C1,
    0xA06E_091F,
];

#[inline]
#[expect(
    clippy::expect_used,
    reason = "u8 cast to usize is always strictly within the 256-element HASH_INDEX table"
)]
fn tag_index(b: u8) -> u32 {
    let idx = usize::from(b);
    HASH_INDEX
        .get(idx)
        .copied()
        .expect("u8 is bounded by 0..=255 for 256-element HASH_INDEX")
}


#[inline]
fn compute_full_tag(slice: &[u8]) -> u32 {
    let mut tag = 0u32;
    for &b in slice {
        tag ^= tag_index(b);
    }
    tag
}

#[inline]
fn compute_next_tag(prev_byte: u8, next_byte: u8, tag: u32) -> u32 {
    tag ^ tag_index(prev_byte) ^ tag_index(next_byte)
}

#[inline]
fn increase_mask(mask: u32) -> u32 {
    (mask.wrapping_shl(1)) | 1
}

#[inline]
fn lesser_bitness(a: u32, b: u32) -> bool {
    let mut mask = 0u32;
    loop {
        if mask == u32::MAX {
            break;
        }
        mask = (mask.wrapping_shl(1)) | 1;
        if (a & b & mask) != mask {
            break;
        }
    }
    (a & mask) < (b & mask)
}

#[derive(Clone, Copy, Default)]
struct HashEntry {
    offset: u32,
    tag: u32,
}

struct MatchFinder {
    hash_bits: u32,
    hash_limit: usize,
    hash_count: usize,
    minimum_tag_mask: u32,
    tag_clean_ptr: usize,
    victim_round: usize,
    max_chain_len: usize,
    table: Vec<HashEntry>,
}

impl MatchFinder {
    fn new(level: &CompressionLevel, chunk_size: usize) -> Result<Self> {
        const ENTRIES_PER_MB: usize = (1024 * 1024) / size_of::<HashEntry>();
        let target_size = level
            .mb_used
            .checked_mul(ENTRIES_PER_MB)
            .context("Target hashtable size calculation overflow")?;

        // Adapt table size for smaller chunks while honoring maximum bits
        let mut bits = 0u32;
        while (1_usize.wrapping_shl(bits)) < target_size {
            bits = bits.saturating_add(1);
        }
        if chunk_size > 0 {
            let mut chunk_bits = 8u32;
            while (1_usize.wrapping_shl(chunk_bits)) < chunk_size && chunk_bits < bits {
                chunk_bits = chunk_bits.saturating_add(1);
            }
            bits = chunk_bits;
        }

        let cap = 1_usize.wrapping_shl(bits);
        let limit = cap
            .checked_div(3)
            .context("Non-zero divisor")?
            .checked_mul(2)
            .context("Hash limit calculation overflow")?;
        let initial_mask = (1_u32.wrapping_shl(level.initial_freq)).wrapping_sub(1);

        Ok(Self {
            hash_bits: bits,
            hash_limit: limit,
            hash_count: 0,
            minimum_tag_mask: initial_mask,
            tag_clean_ptr: 0,
            victim_round: 0,
            max_chain_len: level.max_chain_len,
            table: vec![HashEntry::default(); cap],
        })
    }

    #[inline]
    fn mask(&self) -> usize {
        (1_usize.wrapping_shl(self.hash_bits)).wrapping_sub(1)
    }

    #[inline]
    #[expect(
        clippy::expect_used,
        reason = "u32 fits in usize on supported 32-bit and 64-bit architectures"
    )]
    fn primary_hash(&self, t: u32) -> usize {
        let t_usize = usize::try_from(t)
            .expect("u32 fits in usize on supported 32-bit and 64-bit architectures");
        t_usize & self.mask()
    }

    #[inline]
    fn is_empty(&self, h: usize) -> bool {
        match self.table.get(h) {
            Some(entry) => entry.offset == 0 && entry.tag == 0,
            None => true,
        }
    }

    #[inline]
    fn minimum_bitness(&self, t: u32) -> bool {
        let better = increase_mask(self.minimum_tag_mask);
        (t & better) != better
    }

    fn clean_one_from_hash(&mut self) -> u32 {
        let table_len = self.table.len();
        loop {
            let better = increase_mask(self.minimum_tag_mask);
            while self.tag_clean_ptr < table_len {
                let ptr = self.tag_clean_ptr;
                self.tag_clean_ptr = self.tag_clean_ptr.saturating_add(1);
                if self.is_empty(ptr) {
                    continue;
                }
                let Some(entry) = self.table.get_mut(ptr) else {
                    continue;
                };
                let t = entry.tag;
                if (t & better) != better {
                    entry.offset = 0;
                    entry.tag = 0;
                    self.hash_count = self.hash_count.saturating_sub(1);
                    return better;
                }
            }
            self.minimum_tag_mask = better;
            self.tag_clean_ptr = 0;
        }
    }

    fn insert_hash(&mut self, t: u32, offset: u32) {
        let mut h = self.primary_hash(t);
        let mask = self.mask();
        let mut victim_h = 0usize;
        let mut round = 0usize;

        while !self.is_empty(h) {
            let Some(entry) = self.table.get(h).copied() else {
                break;
            };
            let entry_tag = entry.tag;
            let entry_offset = entry.offset;

            if self.minimum_bitness(entry_tag) {
                self.hash_count = self.hash_count.saturating_sub(1);
                break;
            }

            if lesser_bitness(entry_tag, t) {
                self.insert_hash(entry_tag, entry_offset);
                break;
            }

            if entry_tag == t {
                if round == self.victim_round {
                    victim_h = h;
                }
                round = round.saturating_add(1);
                if round == self.max_chain_len {
                    h = victim_h;
                    self.hash_count = self.hash_count.saturating_sub(1);
                    self.victim_round = self.victim_round.saturating_add(1);
                    if self.victim_round == self.max_chain_len {
                        self.victim_round = 0;
                    }
                    break;
                }
            }

            h = (h.wrapping_add(1)) & mask;
        }

        if let Some(entry) = self.table.get_mut(h) {
            entry.tag = t;
            entry.offset = offset;
        }
    }

    fn match_len(
        buf: &[u8],
        p0: usize,
        op0: usize,
        last_match: usize,
    ) -> (usize, usize) {
        if op0 >= p0 {
            return (0, 0);
        }

        let mut p = p0;
        let mut op = op0;
        let end = buf.len();

        while p < end && op < end && buf.get(p) == buf.get(op) {
            p = p.saturating_add(1);
            op = op.saturating_add(1);
        }
        let mut len = p.saturating_sub(p0);

        let mut p_rev = p0;
        let mut op_rev = op0;
        let min_rev = last_match;

        while p_rev > min_rev && op_rev > 0 {
            let prior_p = p_rev.saturating_sub(1);
            let earlier_target = op_rev.saturating_sub(1);
            if buf.get(prior_p) == buf.get(earlier_target) {
                p_rev = prior_p;
                op_rev = earlier_target;
            } else {
                break;
            }
        }

        let rev = p0.saturating_sub(p_rev);
        len = len.saturating_add(rev);

        if len < MINIMUM_MATCH {
            (0, 0)
        } else {
            (len, rev)
        }
    }

    #[expect(
        clippy::expect_used,
        reason = "u32 fits in usize on supported 32-bit and 64-bit architectures"
    )]
    fn find_best_match(
        &self,
        t: u32,
        p_idx: usize,
        buf: &[u8],
        last_match: usize,
    ) -> (usize, u32, usize) {
        let mut length = 0usize;
        let mut best_offset = 0u32;
        let mut best_rev = 0usize;
        let mask = self.mask();
        let mut h = self.primary_hash(t);

        while !self.is_empty(h) {
            if let Some(entry) = self.table.get(h) {
                if t == entry.tag {
                    let op = usize::try_from(entry.offset)
                        .expect("u32 fits in usize on supported 32-bit and 64-bit architectures");
                    let (mlen, rev) = Self::match_len(buf, p_idx, op, last_match);
                    if mlen >= length && mlen > 0 {
                        length = mlen;
                        // Reason for fallback: on 64-bit systems with huge chunks rev could theoretically exceed u32, saturating offset subtraction to 0
                        let rev_u32 = u32::try_from(rev).unwrap_or(u32::MAX);
                        best_offset = entry.offset.saturating_sub(rev_u32);
                        best_rev = rev;
                    }
                }
            }
            h = (h.wrapping_add(1)) & mask;
        }

        (length, best_offset, best_rev)
    }
}

/// Buffer and block writer for chunk stream multiplexing.
struct ChunkStreamWriter {
    out: Cursor<Vec<u8>>,
    stream_buf: [Vec<u8>; NUM_STREAMS],
    last_head: [usize; NUM_STREAMS],
    bufsize: usize,
    bzip_level: u32,
}

impl ChunkStreamWriter {
    fn new(bzip_level: u32) -> Result<Self> {
        let bufsize = if bzip_level == 0 {
            100_usize
                .checked_mul(1024)
                .context("Buffer size calculation overflow")?
        } else {
            100_usize
                .checked_mul(1024)
                .context("Buffer size calculation overflow")?
                .checked_mul(usize::try_from(bzip_level).context("bzip level overflow")?)
                .context("Buffer size calculation overflow")?
        };

        let mut writer = Self {
            out: Cursor::new(Vec::new()),
            stream_buf: [Vec::new(), Vec::new()],
            last_head: [0, 0],
            bufsize,
            bzip_level,
        };

        // Write initial 13-byte headers for stream 0 and stream 1
        for i in 0..NUM_STREAMS {
            let cur_pos = usize::try_from(writer.out.position())
                .context("Stream position overflow")?;
            let head_ptr = cur_pos.checked_add(9).context("Offset overflow")?;
            if let Some(target_head) = writer.last_head.get_mut(i) {
                *target_head = head_ptr;
            }

            let mut header = [0u8; STREAM_BLOCK_HEADER_SIZE];
            if let Some(first_byte) = header.first_mut() {
                *first_byte = CTYPE_NONE;
            }
            writer.out.write_all(&header)?;
        }

        Ok(writer)
    }

    fn flush_buffer(&mut self, stream: usize) -> Result<()> {
        let u_len_usize = self.stream_buf.get(stream).context("Invalid stream index")?.len();
        if u_len_usize == 0 {
            return Ok(());
        }

        let u_len = u32::try_from(u_len_usize).context("Stream buffer length overflow")?;
        let cur_pos = usize::try_from(self.out.position())
            .context("Stream position overflow")?;
        let cur_pos_u32 = u32::try_from(cur_pos).context("Chunk offset overflow")?;

        // 1. Back-patch previous block header's next_head pointer
        let prev_head = *self.last_head.get(stream).context("Invalid stream index")?;
        self.out.seek(SeekFrom::Start(
            u64::try_from(prev_head).context("Seek offset overflow")?,
        ))?;
        self.out.write_all(&cur_pos_u32.to_le_bytes())?;

        // 2. Update last_head for this new block
        let new_head = cur_pos.checked_add(9).context("Header offset overflow")?;
        if let Some(target_head) = self.last_head.get_mut(stream) {
            *target_head = new_head;
        }
        self.out.seek(SeekFrom::Start(
            u64::try_from(cur_pos).context("Seek offset overflow")?,
        ))?;

        // 3. Compress buffer if bzip_level > 0 and beneficial
        let (c_type, payload) = if self.bzip_level > 0 && u_len_usize > 0 {
            let mut encoder = bzip2::write::BzEncoder::new(
                Vec::new(),
                bzip2::Compression::new(self.bzip_level),
            );
            if let Some(buf) = self.stream_buf.get(stream) {
                encoder.write_all(buf)?;
            }
            let compressed = encoder.finish()?;
            let max_benefit = u_len_usize.saturating_sub(1);
            if compressed.len() < max_benefit {
                (CTYPE_BZIP2, compressed)
            } else {
                let raw = std::mem::take(self.stream_buf.get_mut(stream).context("Invalid stream index")?);
                (CTYPE_NONE, raw)
            }
        } else {
            let raw = std::mem::take(self.stream_buf.get_mut(stream).context("Invalid stream index")?);
            (CTYPE_NONE, raw)
        };

        let c_len = u32::try_from(payload.len()).context("Compressed length overflow")?;

        // 4. Write 13-byte header and payload
        let mut header = [0u8; STREAM_BLOCK_HEADER_SIZE];
        if let Some(first) = header.first_mut() {
            *first = c_type;
        }
        if let Some(c_len_slice) = header.get_mut(1..5) {
            c_len_slice.copy_from_slice(&c_len.to_le_bytes());
        }
        if let Some(u_len_slice) = header.get_mut(5..9) {
            u_len_slice.copy_from_slice(&u_len.to_le_bytes());
        }
        if let Some(next_slice) = header.get_mut(9..13) {
            next_slice.copy_from_slice(&0_u32.to_le_bytes());
        }

        self.out.write_all(&header)?;
        self.out.write_all(&payload)?;

        if let Some(buf) = self.stream_buf.get_mut(stream) {
            buf.clear();
        }
        Ok(())
    }

    fn write_stream(&mut self, stream: usize, data: &[u8]) -> Result<()> {
        let mut remaining = data;
        while !remaining.is_empty() {
            let buf = self.stream_buf.get_mut(stream).context("Invalid stream index")?;
            let space = self.bufsize.saturating_sub(buf.len());
            let n = remaining.len().min(space);
            let slice = remaining.get(..n).context("Slice range error")?;
            buf.extend_from_slice(slice);
            remaining = remaining.get(n..).context("Slice range error")?;

            let updated_len = self.stream_buf.get(stream).context("Invalid stream index")?.len();
            if updated_len >= self.bufsize {
                self.flush_buffer(stream)?;
            }
        }
        Ok(())
    }

    fn finish(mut self) -> Result<Vec<u8>> {
        for i in 0..NUM_STREAMS {
            let is_empty = self.stream_buf.get(i).is_none_or(Vec::is_empty);
            if !is_empty {
                self.flush_buffer(i)?;
            }
        }
        Ok(self.out.into_inner())
    }
}

/// Emits a literal record: Stream 0 receives header (0, len), Stream 1 receives raw bytes.
fn emit_literal(
    sw: &mut ChunkStreamWriter,
    bytes: &[u8],
) -> Result<()> {
    let mut offset = 0usize;
    let total = bytes.len();

    while offset < total {
        let chunk_len = (total.saturating_sub(offset)).min(0xFFFF);
        let len_u16 = u16::try_from(chunk_len).context("Length overflow")?;
        let slice = bytes.get(offset..offset.saturating_add(chunk_len))
            .context("Slice range error")?;

        let mut header = [0u8; 3];
        if let Some(first) = header.first_mut() {
            *first = 0; // head = 0 (literal)
        }
        if let Some(len_slice) = header.get_mut(1..3) {
            len_slice.copy_from_slice(&len_u16.to_le_bytes());
        }

        sw.write_stream(0, &header)?;
        sw.write_stream(1, slice)?;

        offset = offset.saturating_add(chunk_len);
    }
    Ok(())
}

/// Emits a match record: Stream 0 receives header (1, len) followed by little-endian 4-byte offset.
fn emit_match(
    sw: &mut ChunkStreamWriter,
    cur_pos: usize,
    match_pos: usize,
    mut len: usize,
) -> Result<()> {
    let mut p = cur_pos;
    let mut m_pos = match_pos;

    while len > 0 {
        let n = len.min(0xFFFF);
        let n_u16 = u16::try_from(n).context("Match length overflow")?;
        let ofs = u32::try_from(p.saturating_sub(m_pos)).context("Match offset overflow")?;

        let mut header = [0u8; 7];
        if let Some(first) = header.first_mut() {
            *first = 1; // head = 1 (match)
        }
        if let Some(len_slice) = header.get_mut(1..3) {
            len_slice.copy_from_slice(&n_u16.to_le_bytes());
        }
        if let Some(ofs_slice) = header.get_mut(3..7) {
            ofs_slice.copy_from_slice(&ofs.to_le_bytes());
        }

        sw.write_stream(0, &header)?;

        len = len.saturating_sub(n);
        p = p.saturating_add(n);
        m_pos = m_pos.saturating_add(n);
    }
    Ok(())
}

/// Compresses a single chunk of input data and returns the multiplexed chunk bytes.
fn compress_chunk(chunk: &[u8], level: &CompressionLevel) -> Result<Vec<u8>> {
    let mut sw = ChunkStreamWriter::new(level.bzip_level)?;
    let mut finder = MatchFinder::new(level, chunk.len())?;
    let mut tag_mask = (1_u32.wrapping_shl(level.initial_freq)).wrapping_sub(1);

    let mut last_match = 0usize;
    let mut current_len = 0usize;
    let mut current_p = 0usize;
    let mut current_ofs = 0usize;

    let end = chunk.len().saturating_sub(MINIMUM_MATCH);

    if chunk.len() >= MINIMUM_MATCH {
        let mut p = 0usize;
        let mut t = compute_full_tag(chunk.get(..MINIMUM_MATCH).context("Slice range error")?);

        while p < end {
            p = p.saturating_add(1);
            let prev = *chunk.get(p.saturating_sub(1)).context("Prev chunk byte out of bounds")?;
            let next_idx = p.saturating_add(MINIMUM_MATCH.saturating_sub(1));
            let next = *chunk.get(next_idx).context("Next chunk byte out of bounds")?;
            t = compute_next_tag(prev, next, t);

            if (t & finder.minimum_tag_mask) != finder.minimum_tag_mask {
                continue;
            }

            let (mlen, offset, rev) = finder.find_best_match(t, p, chunk, last_match);

            if (t & tag_mask) == tag_mask {
                finder.hash_count = finder.hash_count.saturating_add(1);
                let p_u32 = u32::try_from(p).context("Position overflow")?;
                finder.insert_hash(t, p_u32);
                if finder.hash_count > finder.hash_limit {
                    tag_mask = finder.clean_one_from_hash();
                }
            }

            if mlen > current_len {
                current_p = p.saturating_sub(rev);
                current_len = mlen;
                current_ofs = usize::try_from(offset).context("Offset overflow")?;
            }

            let reached_great = current_len >= GREAT_MATCH;
            let reached_lookahead = p >= current_p.saturating_add(MINIMUM_MATCH);
            if (reached_great || reached_lookahead) && current_len >= MINIMUM_MATCH {
                if last_match < current_p {
                    let lit_slice = chunk.get(last_match..current_p)
                        .context("Slice range error")?;
                    emit_literal(&mut sw, lit_slice)?;
                }
                emit_match(&mut sw, current_p, current_ofs, current_len)?;
                last_match = current_p.saturating_add(current_len);
                p = last_match;
                current_p = p;
                current_len = 0;

                if p < end {
                    let full_slice = chunk.get(p..p.saturating_add(MINIMUM_MATCH))
                        .context("Slice range error")?;
                    t = compute_full_tag(full_slice);
                }
            }
        }
    }

    // Emit trailing literal bytes if any
    if last_match < chunk.len() {
        let trailing = chunk.get(last_match..chunk.len()).context("Slice range error")?;
        emit_literal(&mut sw, trailing)?;
    }

    // Emit EOF marker for Stream 0 (head = 0, len = 0)
    let eof_marker = [0u8, 0, 0];
    sw.write_stream(0, &eof_marker)?;

    // Emit CRC-32 checksum of this chunk in Stream 0
    let crc = update_crc32(0, chunk);
    sw.write_stream(0, &crc.to_le_bytes())?;

    sw.finish()
}

/// Compresses a stream from `reader` into `writer` using the `rzip` 2.1 format.
pub fn compress_stream<R: Read, W: Write>(reader: &mut R, writer: &mut W) -> Result<u64> {
    let level_idx = 6usize; // Default level 6
    let level = LEVELS.get(level_idx).context("Invalid level index")?;

    let mut input_data = Vec::new();
    reader.read_to_end(&mut input_data).context("Failed to read input stream")?;

    let total_uncompressed = input_data.len();
    let total_u64 = u64::try_from(total_uncompressed).context("Size overflow")?;

    // 1. Write 24-byte magic header
    let mut magic_header = [0u8; MAGIC_HEADER_SIZE];
    if let Some(magic_slice) = magic_header.get_mut(0..4) {
        magic_slice.copy_from_slice(&RZIP_MAGIC);
    }
    if let Some(maj) = magic_header.get_mut(4) {
        *maj = RZIP_MAJOR;
    }
    if let Some(min) = magic_header.get_mut(5) {
        *min = RZIP_MINOR;
    }

    let low_32 = u32::try_from(total_u64 & 0xFFFF_FFFF).context("Masked 32 bits")?;
    let high_32 = u32::try_from(total_u64.wrapping_shr(32)).context("Shifted 32 bits")?;

    // Bytes 6..10: low 32 bits (Big Endian), Bytes 10..14: high 32 bits (Big Endian)
    if let Some(low_slice) = magic_header.get_mut(6..10) {
        low_slice.copy_from_slice(&low_32.to_be_bytes());
    }
    if let Some(high_slice) = magic_header.get_mut(10..14) {
        high_slice.copy_from_slice(&high_32.to_be_bytes());
    }

    writer.write_all(&magic_header).context("Failed to write magic header")?;
    let mut total_written = u64::try_from(MAGIC_HEADER_SIZE).context("Header size")?;

    if total_uncompressed == 0 {
        return Ok(total_written);
    }

    // 2. Compress in chunks
    let chunk_size = level_idx
        .max(1)
        .checked_mul(CHUNK_MULTIPLE)
        .context("Chunk size calculation overflow")?;
    let mut offset = 0usize;

    while offset < total_uncompressed {
        let remaining = total_uncompressed.saturating_sub(offset);
        let n = remaining.min(chunk_size);
        let chunk_slice = input_data.get(offset..offset.saturating_add(n))
            .context("Slice range error")?;

        let compressed_chunk = compress_chunk(chunk_slice, level)?;
        writer.write_all(&compressed_chunk).context("Failed to write compressed chunk")?;

        let chunk_len_u64 = u64::try_from(compressed_chunk.len()).context("Length overflow")?;
        total_written = total_written.checked_add(chunk_len_u64).context("Written overflow")?;
        offset = offset.saturating_add(n);
    }

    Ok(total_written)
}

/// Reads multiplexed blocks from `reader` and returns the assembled stream byte vectors.
fn read_demuxed_streams<R: Read>(reader: &mut R) -> Result<[Vec<u8>; NUM_STREAMS]> {
    let mut init_headers = [0u8; 26];
    reader.read_exact(&mut init_headers).context("Failed to read initial stream headers")?;

    let s0_type = *init_headers.first().context("Stream 0 type missing")?;
    ensure!(s0_type == CTYPE_NONE, "Unexpected initial stream 0 type {s0_type}");
    let s0_bytes: [u8; 4] = init_headers
        .get(9..13)
        .context("Stream 0 offset slice")?
        .try_into()
        .context("Stream 0 offset length")?;
    let s0_next = u32::from_le_bytes(s0_bytes);

    let s1_type = *init_headers.get(13).context("Stream 1 type missing")?;
    ensure!(s1_type == CTYPE_NONE, "Unexpected initial stream 1 type {s1_type}");
    let s1_bytes: [u8; 4] = init_headers
        .get(22..26)
        .context("Stream 1 offset slice")?
        .try_into()
        .context("Stream 1 offset length")?;
    let s1_next = u32::from_le_bytes(s1_bytes);

    let mut next_head = [s0_next, s1_next];
    let mut stream_data = [Vec::<u8>::new(), Vec::<u8>::new()];
    let mut cur_offset = 26_u32;

    while next_head.iter().any(|&h| h != 0) {
        let mut block_header = [0u8; STREAM_BLOCK_HEADER_SIZE];
        reader.read_exact(&mut block_header).context("Failed to read stream block header")?;

        let c_type = *block_header.first().context("Missing block c_type")?;
        let c_len_bytes: [u8; 4] = block_header
            .get(1..5)
            .context("c_len slice")?
            .try_into()
            .context("c_len length")?;
        let c_len = u32::from_le_bytes(c_len_bytes);

        let u_len_bytes: [u8; 4] = block_header
            .get(5..9)
            .context("u_len slice")?
            .try_into()
            .context("u_len length")?;
        let u_len = u32::from_le_bytes(u_len_bytes);

        let block_next_bytes: [u8; 4] = block_header
            .get(9..13)
            .context("block_next slice")?
            .try_into()
            .context("block_next length")?;
        let block_next = u32::from_le_bytes(block_next_bytes);

        let c_len_usize = usize::try_from(c_len).context("Block length overflow")?;
        let mut payload = vec![0u8; c_len_usize];
        reader.read_exact(&mut payload).context("Failed to read block payload")?;

        let stream_idx = if next_head.first().is_some_and(|&h| h == cur_offset) {
            if let Some(target) = next_head.first_mut() {
                *target = block_next;
            }
            0
        } else if next_head.get(1).is_some_and(|&h| h == cur_offset) {
            if let Some(target) = next_head.get_mut(1) {
                *target = block_next;
            }
            1
        } else {
            bail!("Unlinked block at chunk offset {cur_offset}");
        };

        let uncompressed = match c_type {
            CTYPE_NONE => payload,
            CTYPE_BZIP2 => {
                let mut decoder = bzip2::read::BzDecoder::new(&payload[..]);
                let mut decomp = Vec::new();
                decoder.read_to_end(&mut decomp).context("Failed to decompress bzip2 block")?;
                decomp
            }
            other => bail!("Invalid stream compression type {other}"),
        };

        let u_len_usize = usize::try_from(u_len).context("Block u_len overflow")?;
        ensure!(
            uncompressed.len() == u_len_usize,
            "Decompressed block length mismatch: got {}, expected {}",
            uncompressed.len(),
            u_len
        );

        if let Some(buf) = stream_data.get_mut(stream_idx) {
            buf.extend_from_slice(&uncompressed);
        }

        let advance = u32::try_from(STREAM_BLOCK_HEADER_SIZE)
            .context("Header size fits u32")?
            .checked_add(c_len)
            .context("Offset overflow")?;
        cur_offset = cur_offset.checked_add(advance).context("Chunk offset overflow")?;
    }

    Ok(stream_data)
}

/// Replays Stream 0 control instructions, pulling literals from Stream 1, and emits output.
fn replay_stream0_instructions<W: Write>(
    s0_bytes: Vec<u8>,
    s1_bytes: Vec<u8>,
    writer: &mut W,
    remaining_file_size: u64,
) -> Result<u64> {
    let mut s0 = Cursor::new(s0_bytes);
    let mut s1 = Cursor::new(s1_bytes);

    let mut chunk_history = Vec::<u8>::new();
    let mut running_crc = 0u32;
    let mut chunk_decompressed = 0u64;

    loop {
        let mut head_buf = [0u8; 1];
        if s0.read_exact(&mut head_buf).is_err() {
            bail!("Unexpected end of stream 0 control data");
        }
        let [head] = head_buf;

        let mut len_buf = [0u8; 2];
        s0.read_exact(&mut len_buf).context("Failed to read record length")?;
        let len = usize::from(u16::from_le_bytes(len_buf));

        if head == 0 && len == 0 {
            let mut crc_buf = [0u8; 4];
            s0.read_exact(&mut crc_buf).context("Failed to read chunk CRC-32")?;
            let stored_crc = u32::from_le_bytes(crc_buf);
            ensure!(
                stored_crc == running_crc,
                "rzip CRC-32 checksum mismatch: stored 0x{stored_crc:08X}, computed 0x{running_crc:08X}"
            );
            break;
        }

        match head {
            0 => {
                let mut lit_buf = vec![0u8; len];
                s1.read_exact(&mut lit_buf).context("Failed to read literal data from stream 1")?;

                writer.write_all(&lit_buf).context("Failed to write decompressed literal")?;
                running_crc = update_crc32(running_crc, &lit_buf);
                chunk_history.extend_from_slice(&lit_buf);

                let len_u64 = u64::try_from(len).context("Length overflow")?;
                chunk_decompressed = chunk_decompressed.checked_add(len_u64)
                    .context("Decompressed length overflow")?;
            }
            1 => {
                let mut ofs_buf = [0u8; 4];
                s0.read_exact(&mut ofs_buf).context("Failed to read match offset")?;
                let offset = usize::try_from(u32::from_le_bytes(ofs_buf))
                    .context("Match offset overflow")?;

                let cur_history_len = chunk_history.len();
                ensure!(
                    offset <= cur_history_len && offset > 0,
                    "Invalid match offset {offset}, current history length is {cur_history_len}"
                );

                let match_start = cur_history_len.checked_sub(offset)
                    .context("Underflow calculating match start")?;

                let old_len = chunk_history.len();
                for i in 0..len {
                    let read_idx = match_start.checked_add(i).context("Index overflow")?;
                    let b = chunk_history.get(read_idx).copied().context("Match index out of bounds")?;
                    chunk_history.push(b);
                }

                let match_slice = chunk_history.get(old_len..old_len.saturating_add(len))
                    .context("Slice range error")?;
                writer.write_all(match_slice).context("Failed to write decompressed match")?;
                running_crc = update_crc32(running_crc, match_slice);

                let len_u64 = u64::try_from(len).context("Length overflow")?;
                chunk_decompressed = chunk_decompressed.checked_add(len_u64)
                    .context("Decompressed length overflow")?;
            }
            other => bail!("Invalid record head type {other} in stream 0"),
        }

        ensure!(
            chunk_decompressed <= remaining_file_size,
            "Decompressed chunk data exceeded expected total file size"
        );
    }

    Ok(chunk_decompressed)
}

/// Decompresses a single rzip chunk from `reader` and writes uncompressed bytes to `writer`.
fn decompress_chunk<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    remaining_file_size: u64,
) -> Result<u64> {
    let [s0, s1] = read_demuxed_streams(reader)?;
    replay_stream0_instructions(s0, s1, writer, remaining_file_size)
}

/// Decompresses a stream from `reader` into `writer` using the `rzip` 2.1 format.
pub fn decompress_stream<R: Read, W: Write>(reader: &mut R, writer: &mut W) -> Result<u64> {
    // 1. Read 24-byte magic header
    let mut magic_header = [0u8; MAGIC_HEADER_SIZE];
    reader.read_exact(&mut magic_header).context("Failed to read 24-byte rzip magic header")?;

    ensure!(
        magic_header.get(0..4) == Some(&RZIP_MAGIC),
        "Not an rzip file: magic mismatch"
    );

    let major = *magic_header.get(4).context("Missing major byte")?;
    let minor = *magic_header.get(5).context("Missing minor byte")?;
    ensure!(
        major == RZIP_MAJOR && minor == RZIP_MINOR,
        "Unsupported rzip version {major}.{minor}, expected {RZIP_MAJOR}.{RZIP_MINOR}"
    );

    let low_bytes: [u8; 4] = magic_header
        .get(6..10)
        .context("Low size bytes")?
        .try_into()
        .context("Low size length")?;
    let low_32 = u32::from_be_bytes(low_bytes);

    let high_bytes: [u8; 4] = magic_header
        .get(10..14)
        .context("High size bytes")?
        .try_into()
        .context("High size length")?;
    let high_32 = u32::from_be_bytes(high_bytes);
    let expected_size = u64::from(low_32) | (u64::from(high_32).wrapping_shl(32));

    if expected_size == 0 {
        return Ok(0);
    }

    // 2. Decompress chunks sequentially until expected_size is reached
    let mut total_decompressed = 0u64;

    while total_decompressed < expected_size {
        let remaining = expected_size.saturating_sub(total_decompressed);
        let decompressed_this_chunk = decompress_chunk(reader, writer, remaining)?;
        total_decompressed = total_decompressed.checked_add(decompressed_this_chunk)
            .context("Total decompressed overflow")?;
    }

    ensure!(
        total_decompressed == expected_size,
        "Decompressed size mismatch: got {total_decompressed}, expected {expected_size}"
    );

    Ok(total_decompressed)
}

/// Convenience helper to compress an in-memory byte slice.
pub fn compress_bytes(input: &[u8]) -> Result<Vec<u8>> {
    let mut reader = input;
    let mut output = Vec::new();
    compress_stream(&mut reader, &mut output)?;
    Ok(output)
}

/// Convenience helper to decompress an in-memory byte slice.
pub fn decompress_bytes(input: &[u8]) -> Result<Vec<u8>> {
    let mut reader = input;
    let mut output = Vec::new();
    decompress_stream(&mut reader, &mut output)?;
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
    fn test_rzip_crc() {
        assert_eq!(update_crc32(0, b""), 0);
        let data = b"Hello, world!";
        assert_eq!(update_crc32(0, data), 0xE492_8064);
    }

    #[crate::ctb_test]
    fn test_regenerate_crc_table() {
        // Regenerates Mark Adler's CRC-32 table (polynomial 0xEDB8_8320)
        // as described in rzip-2.1/crc32.c and the PNG specification (ISO/IEC 15948:2004).
        let mut generated_table = [0u32; 256];
        for (i, entry) in generated_table.iter_mut().enumerate() {
            let mut c = u32::try_from(i).unwrap_or(0);
            for _ in 0..8 {
                if (c & 1) != 0 {
                    c = 0xEDB8_8320 ^ (c.wrapping_shr(1));
                } else {
                    c = c.wrapping_shr(1);
                }
            }
            *entry = c;
        }

        assert_eq!(
            generated_table, CRC_TABLE,
            "Generated CRC-32 table must match CRC_TABLE"
        );

        // Also verify against crc32fast::Hasher for every single byte
        for (i, &expected) in CRC_TABLE.iter().enumerate() {
            let byte = u8::try_from(i).unwrap_or(0);
            let mut hasher = crc32fast::Hasher::new_with_initial(!0);
            hasher.update(&[byte]);
            let computed = !hasher.finalize();
            assert_eq!(
                computed, expected,
                "crc32fast single-byte hash must match table at index {i}"
            );
        }
    }

    #[crate::ctb_test]
    fn test_regenerate_hash_index_table() {
        // Regenerates HASH_INDEX matching old/unix-tools/rzip-2.1/rzip.c init_hash_indexes():
        //   for (i = 0; i < 256; i++) {
        //       hash_index[i] = ((random() << 16) ^ random());
        //   }
        // where random() is the default glibc additive lagged Fibonacci PRNG (TYPE_3, seed 1).
        let mut state = [0u32; 31];
        state[0] = 1;
        for i in 1_usize..31_usize {
            let prev = state.get(i.saturating_sub(1)).copied().unwrap_or(0);
            let prod = u64::from(prev).wrapping_mul(16807);
            let val = prod.wrapping_rem(0x7FFF_FFFF);
            if let Some(slot) = state.get_mut(i) {
                *slot = u32::try_from(val).unwrap_or(0);
            }
        }

        let mut fptr = 3usize;
        let mut rptr = 0usize;

        let mut glibc_random = || -> u32 {
            let s_f = state[fptr];
            let s_r = state[rptr];
            let val = s_f.wrapping_add(s_r);
            state[fptr] = val;
            let res = (val.wrapping_shr(1)) & 0x7FFF_FFFF;
            fptr = (fptr.wrapping_add(1)).wrapping_rem(31);
            rptr = (rptr.wrapping_add(1)).wrapping_rem(31);
            res
        };

        // Glibc srandom() warms up the state with 10 * 31 = 310 calls to random().
        for _ in 0..310 {
            glibc_random();
        }

        let mut generated_hash_index = [0u32; 256];
        for entry in &mut generated_hash_index {
            let r1 = glibc_random();
            let r2 = glibc_random();
            *entry = (r1.wrapping_shl(16)) ^ r2;
        }

        assert_eq!(
            generated_hash_index, HASH_INDEX,
            "Generated HASH_INDEX must match precomputed HASH_INDEX"
        );
    }

    #[crate::ctb_test]
    fn test_rzip_roundtrip_empty() {
        let compressed = compress_bytes(b"").unwrap();
        assert_eq!(compressed.len(), 24);
        assert_eq!(&compressed[0..4], &RZIP_MAGIC);
        assert_eq!(compressed[4], RZIP_MAJOR);
        assert_eq!(compressed[5], RZIP_MINOR);
        let decompressed = decompress_bytes(&compressed).unwrap();
        assert!(decompressed.is_empty());
    }

    #[crate::ctb_test]
    fn test_rzip_corrupted_magic() {
        let mut compressed = compress_bytes(b"Test data").unwrap();
        if let Some(first) = compressed.first_mut() {
            *first = b'X';
        }
        let _ = decompress_bytes(&compressed).unwrap_err();
    }

    #[crate::ctb_test]
    fn test_rzip_corrupted_crc() {
        let mut compressed = compress_bytes(b"Test data with enough length to test").unwrap();
        let last = compressed.len().saturating_sub(1);
        if let Some(target) = compressed.get_mut(last) {
            *target ^= 0xFF;
        }
        let _ = decompress_bytes(&compressed).unwrap_err();
    }
}

/*
Licensing details for rzip, from rzip-2.1/COPYING:

```
		    GNU GENERAL PUBLIC LICENSE
		       Version 2, June 1991

 Copyright (C) 1989, 1991 Free Software Foundation, Inc.
                          675 Mass Ave, Cambridge, MA 02139, USA
 Everyone is permitted to copy and distribute verbatim copies
 of this license document, but changing it is not allowed.

			    Preamble

  The licenses for most software are designed to take away your
freedom to share and change it.  By contrast, the GNU General Public
License is intended to guarantee your freedom to share and change free
software--to make sure the software is free for all its users.  This
General Public License applies to most of the Free Software
Foundation's software and to any other program whose authors commit to
using it.  (Some other Free Software Foundation software is covered by
the GNU Library General Public License instead.)  You can apply it to
your programs, too.

  When we speak of free software, we are referring to freedom, not
price.  Our General Public Licenses are designed to make sure that you
have the freedom to distribute copies of free software (and charge for
this service if you wish), that you receive source code or can get it
if you want it, that you can change the software or use pieces of it
in new free programs; and that you know you can do these things.

  To protect your rights, we need to make restrictions that forbid
anyone to deny you these rights or to ask you to surrender the rights.
These restrictions translate to certain responsibilities for you if you
distribute copies of the software, or if you modify it.

  For example, if you distribute copies of such a program, whether
gratis or for a fee, you must give the recipients all the rights that
you have.  You must make sure that they, too, receive or can get the
source code.  And you must show them these terms so they know their
rights.

  We protect your rights with two steps: (1) copyright the software, and
(2) offer you this license which gives you legal permission to copy,
distribute and/or modify the software.

  Also, for each author's protection and ours, we want to make certain
that everyone understands that there is no warranty for this free
software.  If the software is modified by someone else and passed on, we
want its recipients to know that what they have is not the original, so
that any problems introduced by others will not reflect on the original
authors' reputations.

  Finally, any free program is threatened constantly by software
patents.  We wish to avoid the danger that redistributors of a free
program will individually obtain patent licenses, in effect making the
program proprietary.  To prevent this, we have made it clear that any
patent must be licensed for everyone's free use or not licensed at all.

  The precise terms and conditions for copying, distribution and
modification follow.

		    GNU GENERAL PUBLIC LICENSE
   TERMS AND CONDITIONS FOR COPYING, DISTRIBUTION AND MODIFICATION

  0. This License applies to any program or other work which contains
a notice placed by the copyright holder saying it may be distributed
under the terms of this General Public License.  The "Program", below,
refers to any such program or work, and a "work based on the Program"
means either the Program or any derivative work under copyright law:
that is to say, a work containing the Program or a portion of it,
either verbatim or with modifications and/or translated into another
language.  (Hereinafter, translation is included without limitation in
the term "modification".)  Each licensee is addressed as "you".

Activities other than copying, distribution and modification are not
covered by this License; they are outside its scope.  The act of
running the Program is not restricted, and the output from the Program
is covered only if its contents constitute a work based on the
Program (independent of having been made by running the Program).
Whether that is true depends on what the Program does.

  1. You may copy and distribute verbatim copies of the Program's
source code as you receive it, in any medium, provided that you
conspicuously and appropriately publish on each copy an appropriate
copyright notice and disclaimer of warranty; keep intact all the
notices that refer to this License and to the absence of any warranty;
and give any other recipients of the Program a copy of this License
along with the Program.

You may charge a fee for the physical act of transferring a copy, and
you may at your option offer warranty protection in exchange for a fee.

  2. You may modify your copy or copies of the Program or any portion
of it, thus forming a work based on the Program, and copy and
distribute such modifications or work under the terms of Section 1
above, provided that you also meet all of these conditions:

    a) You must cause the modified files to carry prominent notices
    stating that you changed the files and the date of any change.

    b) You must cause any work that you distribute or publish, that in
    whole or in part contains or is derived from the Program or any
    part thereof, to be licensed as a whole at no charge to all third
    parties under the terms of this License.

    c) If the modified program normally reads commands interactively
    when run, you must cause it, when started running for such
    interactive use in the most ordinary way, to print or display an
    announcement including an appropriate copyright notice and a
    notice that there is no warranty (or else, saying that you provide
    a warranty) and that users may redistribute the program under
    these conditions, and telling the user how to view a copy of this
    License.  (Exception: if the Program itself is interactive but
    does not normally print such an announcement, your work based on
    the Program is not required to print an announcement.)

These requirements apply to the modified work as a whole.  If
identifiable sections of that work are not derived from the Program,
and can be reasonably considered independent and separate works in
themselves, then this License, and its terms, do not apply to those
sections when you distribute them as separate works.  But when you
distribute the same sections as part of a whole which is a work based
on the Program, the distribution of the whole must be on the terms of
this License, whose permissions for other licensees extend to the
entire whole, and thus to each and every part regardless of who wrote it.

Thus, it is not the intent of this section to claim rights or contest
your rights to work written entirely by you; rather, the intent is to
exercise the right to control the distribution of derivative or
collective works based on the Program.

In addition, mere aggregation of another work not based on the Program
with the Program (or with a work based on the Program) on a volume of
a storage or distribution medium does not bring the other work under
the scope of this License.

  3. You may copy and distribute the Program (or a work based on it,
under Section 2) in object code or executable form under the terms of
Sections 1 and 2 above provided that you also do one of the following:

    a) Accompany it with the complete corresponding machine-readable
    source code, which must be distributed under the terms of Sections
    1 and 2 above on a medium customarily used for software interchange; or,

    b) Accompany it with a written offer, valid for at least three
    years, to give any third party, for a charge no more than your
    cost of physically performing source distribution, a complete
    machine-readable copy of the corresponding source code, to be
    distributed under the terms of Sections 1 and 2 above on a medium
    customarily used for software interchange; or,

    c) Accompany it with the information you received as to the offer
    to distribute corresponding source code.  (This alternative is
    allowed only for noncommercial distribution and only if you
    received the program in object code or executable form with such
    an offer, in accord with Subsection b above.)

The source code for a work means the preferred form of the work for
making modifications to it.  For an executable work, complete source
code means all the source code for all modules it contains, plus any
associated interface definition files, plus the scripts used to
control compilation and installation of the executable.  However, as a
special exception, the source code distributed need not include
anything that is normally distributed (in either source or binary
form) with the major components (compiler, kernel, and so on) of the
operating system on which the executable runs, unless that component
itself accompanies the executable.

If distribution of executable or object code is made by offering
access to copy from a designated place, then offering equivalent
access to copy the source code from the same place counts as
distribution of the source code, even though third parties are not
compelled to copy the source along with the object code.

  4. You may not copy, modify, sublicense, or distribute the Program
except as expressly provided under this License.  Any attempt
otherwise to copy, modify, sublicense or distribute the Program is
void, and will automatically terminate your rights under this License.
However, parties who have received copies, or rights, from you under
this License will not have their licenses terminated so long as such
parties remain in full compliance.

  5. You are not required to accept this License, since you have not
signed it.  However, nothing else grants you permission to modify or
distribute the Program or its derivative works.  These actions are
prohibited by law if you do not accept this License.  Therefore, by
modifying or distributing the Program (or any work based on the
Program), you indicate your acceptance of this License to do so, and
all its terms and conditions for copying, distributing or modifying
the Program or works based on it.

  6. Each time you redistribute the Program (or any work based on the
Program), the recipient automatically receives a license from the
original licensor to copy, distribute or modify the Program subject to
these terms and conditions.  You may not impose any further
restrictions on the recipients' exercise of the rights granted herein.
You are not responsible for enforcing compliance by third parties to
this License.

  7. If, as a consequence of a court judgment or allegation of patent
infringement or for any other reason (not limited to patent issues),
conditions are imposed on you (whether by court order, agreement or
otherwise) that contradict the conditions of this License, they do not
excuse you from the conditions of this License.  If you cannot
distribute so as to satisfy simultaneously your obligations under this
License and any other pertinent obligations, then as a consequence you
may not distribute the Program at all.  For example, if a patent
license would not permit royalty-free redistribution of the Program by
all those who receive copies directly or indirectly through you, then
the only way you could satisfy both it and this License would be to
refrain entirely from distribution of the Program.

If any portion of this section is held invalid or unenforceable under
any particular circumstance, the balance of the section is intended to
apply and the section as a whole is intended to apply in other
circumstances.

It is not the purpose of this section to induce you to infringe any
patents or other property right claims or to contest validity of any
such claims; this section has the sole purpose of protecting the
integrity of the free software distribution system, which is
implemented by public license practices.  Many people have made
generous contributions to the wide range of software distributed
through that system in reliance on consistent application of that
system; it is up to the author/donor to decide if he or she is willing
to distribute software through any other system and a licensee cannot
impose that choice.

This section is intended to make thoroughly clear what is believed to
be a consequence of the rest of this License.

  8. If the distribution and/or use of the Program is restricted in
certain countries either by patents or by copyrighted interfaces, the
original copyright holder who places the Program under this License
may add an explicit geographical distribution limitation excluding
those countries, so that distribution is permitted only in or among
countries not thus excluded.  In such case, this License incorporates
the limitation as if written in the body of this License.

  9. The Free Software Foundation may publish revised and/or new versions
of the General Public License from time to time.  Such new versions will
be similar in spirit to the present version, but may differ in detail to
address new problems or concerns.

Each version is given a distinguishing version number.  If the Program
specifies a version number of this License which applies to it and "any
later version", you have the option of following the terms and conditions
either of that version or of any later version published by the Free
Software Foundation.  If the Program does not specify a version number of
this License, you may choose any version ever published by the Free Software
Foundation.

  10. If you wish to incorporate parts of the Program into other free
programs whose distribution conditions are different, write to the author
to ask for permission.  For software which is copyrighted by the Free
Software Foundation, write to the Free Software Foundation; we sometimes
make exceptions for this.  Our decision will be guided by the two goals
of preserving the free status of all derivatives of our free software and
of promoting the sharing and reuse of software generally.

			    NO WARRANTY

  11. BECAUSE THE PROGRAM IS LICENSED FREE OF CHARGE, THERE IS NO WARRANTY
FOR THE PROGRAM, TO THE EXTENT PERMITTED BY APPLICABLE LAW.  EXCEPT WHEN
OTHERWISE STATED IN WRITING THE COPYRIGHT HOLDERS AND/OR OTHER PARTIES
PROVIDE THE PROGRAM "AS IS" WITHOUT WARRANTY OF ANY KIND, EITHER EXPRESSED
OR IMPLIED, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF
MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE.  THE ENTIRE RISK AS
TO THE QUALITY AND PERFORMANCE OF THE PROGRAM IS WITH YOU.  SHOULD THE
PROGRAM PROVE DEFECTIVE, YOU ASSUME THE COST OF ALL NECESSARY SERVICING,
REPAIR OR CORRECTION.

  12. IN NO EVENT UNLESS REQUIRED BY APPLICABLE LAW OR AGREED TO IN WRITING
WILL ANY COPYRIGHT HOLDER, OR ANY OTHER PARTY WHO MAY MODIFY AND/OR
REDISTRIBUTE THE PROGRAM AS PERMITTED ABOVE, BE LIABLE TO YOU FOR DAMAGES,
INCLUDING ANY GENERAL, SPECIAL, INCIDENTAL OR CONSEQUENTIAL DAMAGES ARISING
OUT OF THE USE OR INABILITY TO USE THE PROGRAM (INCLUDING BUT NOT LIMITED
TO LOSS OF DATA OR DATA BEING RENDERED INACCURATE OR LOSSES SUSTAINED BY
YOU OR THIRD PARTIES OR A FAILURE OF THE PROGRAM TO OPERATE WITH ANY OTHER
PROGRAMS), EVEN IF SUCH HOLDER OR OTHER PARTY HAS BEEN ADVISED OF THE
POSSIBILITY OF SUCH DAMAGES.

		     END OF TERMS AND CONDITIONS

	Appendix: How to Apply These Terms to Your New Programs

  If you develop a new program, and you want it to be of the greatest
possible use to the public, the best way to achieve this is to make it
free software which everyone can redistribute and change under these terms.

  To do so, attach the following notices to the program.  It is safest
to attach them to the start of each source file to most effectively
convey the exclusion of warranty; and each file should have at least
the "copyright" line and a pointer to where the full notice is found.

    <one line to give the program's name and a brief idea of what it does.>
    Copyright (C) 19yy  <name of author>

    This program is free software; you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation; either version 2 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program; if not, write to the Free Software
    Foundation, Inc., 675 Mass Ave, Cambridge, MA 02139, USA.

Also add information on how to contact you by electronic and paper mail.

If the program is interactive, make it output a short notice like this
when it starts in an interactive mode:

    Gnomovision version 69, Copyright (C) 19yy name of author
    Gnomovision comes with ABSOLUTELY NO WARRANTY; for details type `show w'.
    This is free software, and you are welcome to redistribute it
    under certain conditions; type `show c' for details.

The hypothetical commands `show w' and `show c' should show the appropriate
parts of the General Public License.  Of course, the commands you use may
be called something other than `show w' and `show c'; they could even be
mouse-clicks or menu items--whatever suits your program.

You should also get your employer (if you work as a programmer) or your
school, if any, to sign a "copyright disclaimer" for the program, if
necessary.  Here is a sample; alter the names:

  Yoyodyne, Inc., hereby disclaims all copyright interest in the program
  `Gnomovision' (which makes passes at compilers) written by James Hacker.

  <signature of Ty Coon>, 1 April 1989
  Ty Coon, President of Vice

This General Public License does not permit incorporating your program into
proprietary programs.  If your program is a subroutine library, you may
consider it more useful to permit linking proprietary applications with the
library.  If this is what you want to do, use the GNU Library General
Public License instead of this License.
```

Text of the GPL 3, the license Collective Toolbox reuses rzip under per the or-later option:

```
# GNU GENERAL PUBLIC LICENSE

Version 3, 29 June 2007

Copyright (C) 2007 Free Software Foundation, Inc.
<https://fsf.org/>

Everyone is permitted to copy and distribute verbatim copies of this
license document, but changing it is not allowed.

## Preamble

The GNU General Public License is a free, copyleft license for
software and other kinds of works.

The licenses for most software and other practical works are designed
to take away your freedom to share and change the works. By contrast,
the GNU General Public License is intended to guarantee your freedom
to share and change all versions of a program--to make sure it remains
free software for all its users. We, the Free Software Foundation, use
the GNU General Public License for most of our software; it applies
also to any other work released this way by its authors. You can apply
it to your programs, too.

When we speak of free software, we are referring to freedom, not
price. Our General Public Licenses are designed to make sure that you
have the freedom to distribute copies of free software (and charge for
them if you wish), that you receive source code or can get it if you
want it, that you can change the software or use pieces of it in new
free programs, and that you know you can do these things.

To protect your rights, we need to prevent others from denying you
these rights or asking you to surrender the rights. Therefore, you
have certain responsibilities if you distribute copies of the
software, or if you modify it: responsibilities to respect the freedom
of others.

For example, if you distribute copies of such a program, whether
gratis or for a fee, you must pass on to the recipients the same
freedoms that you received. You must make sure that they, too, receive
or can get the source code. And you must show them these terms so they
know their rights.

Developers that use the GNU GPL protect your rights with two steps:
(1) assert copyright on the software, and (2) offer you this License
giving you legal permission to copy, distribute and/or modify it.

For the developers' and authors' protection, the GPL clearly explains
that there is no warranty for this free software. For both users' and
authors' sake, the GPL requires that modified versions be marked as
changed, so that their problems will not be attributed erroneously to
authors of previous versions.

Some devices are designed to deny users access to install or run
modified versions of the software inside them, although the
manufacturer can do so. This is fundamentally incompatible with the
aim of protecting users' freedom to change the software. The
systematic pattern of such abuse occurs in the area of products for
individuals to use, which is precisely where it is most unacceptable.
Therefore, we have designed this version of the GPL to prohibit the
practice for those products. If such problems arise substantially in
other domains, we stand ready to extend this provision to those
domains in future versions of the GPL, as needed to protect the
freedom of users.

Finally, every program is threatened constantly by software patents.
States should not allow patents to restrict development and use of
software on general-purpose computers, but in those that do, we wish
to avoid the special danger that patents applied to a free program
could make it effectively proprietary. To prevent this, the GPL
assures that patents cannot be used to render the program non-free.

The precise terms and conditions for copying, distribution and
modification follow.

## TERMS AND CONDITIONS

### 0. Definitions.

"This License" refers to version 3 of the GNU General Public License.

"Copyright" also means copyright-like laws that apply to other kinds
of works, such as semiconductor masks.

"The Program" refers to any copyrightable work licensed under this
License. Each licensee is addressed as "you". "Licensees" and
"recipients" may be individuals or organizations.

To "modify" a work means to copy from or adapt all or part of the work
in a fashion requiring copyright permission, other than the making of
an exact copy. The resulting work is called a "modified version" of
the earlier work or a work "based on" the earlier work.

A "covered work" means either the unmodified Program or a work based
on the Program.

To "propagate" a work means to do anything with it that, without
permission, would make you directly or secondarily liable for
infringement under applicable copyright law, except executing it on a
computer or modifying a private copy. Propagation includes copying,
distribution (with or without modification), making available to the
public, and in some countries other activities as well.

To "convey" a work means any kind of propagation that enables other
parties to make or receive copies. Mere interaction with a user
through a computer network, with no transfer of a copy, is not
conveying.

An interactive user interface displays "Appropriate Legal Notices" to
the extent that it includes a convenient and prominently visible
feature that (1) displays an appropriate copyright notice, and (2)
tells the user that there is no warranty for the work (except to the
extent that warranties are provided), that licensees may convey the
work under this License, and how to view a copy of this License. If
the interface presents a list of user commands or options, such as a
menu, a prominent item in the list meets this criterion.

### 1. Source Code.

The "source code" for a work means the preferred form of the work for
making modifications to it. "Object code" means any non-source form of
a work.

A "Standard Interface" means an interface that either is an official
standard defined by a recognized standards body, or, in the case of
interfaces specified for a particular programming language, one that
is widely used among developers working in that language.

The "System Libraries" of an executable work include anything, other
than the work as a whole, that (a) is included in the normal form of
packaging a Major Component, but which is not part of that Major
Component, and (b) serves only to enable use of the work with that
Major Component, or to implement a Standard Interface for which an
implementation is available to the public in source code form. A
"Major Component", in this context, means a major essential component
(kernel, window system, and so on) of the specific operating system
(if any) on which the executable work runs, or a compiler used to
produce the work, or an object code interpreter used to run it.

The "Corresponding Source" for a work in object code form means all
the source code needed to generate, install, and (for an executable
work) run the object code and to modify the work, including scripts to
control those activities. However, it does not include the work's
System Libraries, or general-purpose tools or generally available free
programs which are used unmodified in performing those activities but
which are not part of the work. For example, Corresponding Source
includes interface definition files associated with source files for
the work, and the source code for shared libraries and dynamically
linked subprograms that the work is specifically designed to require,
such as by intimate data communication or control flow between those
subprograms and other parts of the work.

The Corresponding Source need not include anything that users can
regenerate automatically from other parts of the Corresponding Source.

The Corresponding Source for a work in source code form is that same
work.

### 2. Basic Permissions.

All rights granted under this License are granted for the term of
copyright on the Program, and are irrevocable provided the stated
conditions are met. This License explicitly affirms your unlimited
permission to run the unmodified Program. The output from running a
covered work is covered by this License only if the output, given its
content, constitutes a covered work. This License acknowledges your
rights of fair use or other equivalent, as provided by copyright law.

You may make, run and propagate covered works that you do not convey,
without conditions so long as your license otherwise remains in force.
You may convey covered works to others for the sole purpose of having
them make modifications exclusively for you, or provide you with
facilities for running those works, provided that you comply with the
terms of this License in conveying all material for which you do not
control copyright. Those thus making or running the covered works for
you must do so exclusively on your behalf, under your direction and
control, on terms that prohibit them from making any copies of your
copyrighted material outside their relationship with you.

Conveying under any other circumstances is permitted solely under the
conditions stated below. Sublicensing is not allowed; section 10 makes
it unnecessary.

### 3. Protecting Users' Legal Rights From Anti-Circumvention Law.

No covered work shall be deemed part of an effective technological
measure under any applicable law fulfilling obligations under article
11 of the WIPO copyright treaty adopted on 20 December 1996, or
similar laws prohibiting or restricting circumvention of such
measures.

When you convey a covered work, you waive any legal power to forbid
circumvention of technological measures to the extent such
circumvention is effected by exercising rights under this License with
respect to the covered work, and you disclaim any intention to limit
operation or modification of the work as a means of enforcing, against
the work's users, your or third parties' legal rights to forbid
circumvention of technological measures.

### 4. Conveying Verbatim Copies.

You may convey verbatim copies of the Program's source code as you
receive it, in any medium, provided that you conspicuously and
appropriately publish on each copy an appropriate copyright notice;
keep intact all notices stating that this License and any
non-permissive terms added in accord with section 7 apply to the code;
keep intact all notices of the absence of any warranty; and give all
recipients a copy of this License along with the Program.

You may charge any price or no price for each copy that you convey,
and you may offer support or warranty protection for a fee.

### 5. Conveying Modified Source Versions.

You may convey a work based on the Program, or the modifications to
produce it from the Program, in the form of source code under the
terms of section 4, provided that you also meet all of these
conditions:

-   a) The work must carry prominent notices stating that you modified
    it, and giving a relevant date.
-   b) The work must carry prominent notices stating that it is
    released under this License and any conditions added under
    section 7. This requirement modifies the requirement in section 4
    to "keep intact all notices".
-   c) You must license the entire work, as a whole, under this
    License to anyone who comes into possession of a copy. This
    License will therefore apply, along with any applicable section 7
    additional terms, to the whole of the work, and all its parts,
    regardless of how they are packaged. This License gives no
    permission to license the work in any other way, but it does not
    invalidate such permission if you have separately received it.
-   d) If the work has interactive user interfaces, each must display
    Appropriate Legal Notices; however, if the Program has interactive
    interfaces that do not display Appropriate Legal Notices, your
    work need not make them do so.

A compilation of a covered work with other separate and independent
works, which are not by their nature extensions of the covered work,
and which are not combined with it such as to form a larger program,
in or on a volume of a storage or distribution medium, is called an
"aggregate" if the compilation and its resulting copyright are not
used to limit the access or legal rights of the compilation's users
beyond what the individual works permit. Inclusion of a covered work
in an aggregate does not cause this License to apply to the other
parts of the aggregate.

### 6. Conveying Non-Source Forms.

You may convey a covered work in object code form under the terms of
sections 4 and 5, provided that you also convey the machine-readable
Corresponding Source under the terms of this License, in one of these
ways:

-   a) Convey the object code in, or embodied in, a physical product
    (including a physical distribution medium), accompanied by the
    Corresponding Source fixed on a durable physical medium
    customarily used for software interchange.
-   b) Convey the object code in, or embodied in, a physical product
    (including a physical distribution medium), accompanied by a
    written offer, valid for at least three years and valid for as
    long as you offer spare parts or customer support for that product
    model, to give anyone who possesses the object code either (1) a
    copy of the Corresponding Source for all the software in the
    product that is covered by this License, on a durable physical
    medium customarily used for software interchange, for a price no
    more than your reasonable cost of physically performing this
    conveying of source, or (2) access to copy the Corresponding
    Source from a network server at no charge.
-   c) Convey individual copies of the object code with a copy of the
    written offer to provide the Corresponding Source. This
    alternative is allowed only occasionally and noncommercially, and
    only if you received the object code with such an offer, in accord
    with subsection 6b.
-   d) Convey the object code by offering access from a designated
    place (gratis or for a charge), and offer equivalent access to the
    Corresponding Source in the same way through the same place at no
    further charge. You need not require recipients to copy the
    Corresponding Source along with the object code. If the place to
    copy the object code is a network server, the Corresponding Source
    may be on a different server (operated by you or a third party)
    that supports equivalent copying facilities, provided you maintain
    clear directions next to the object code saying where to find the
    Corresponding Source. Regardless of what server hosts the
    Corresponding Source, you remain obligated to ensure that it is
    available for as long as needed to satisfy these requirements.
-   e) Convey the object code using peer-to-peer transmission,
    provided you inform other peers where the object code and
    Corresponding Source of the work are being offered to the general
    public at no charge under subsection 6d.

A separable portion of the object code, whose source code is excluded
from the Corresponding Source as a System Library, need not be
included in conveying the object code work.

A "User Product" is either (1) a "consumer product", which means any
tangible personal property which is normally used for personal,
family, or household purposes, or (2) anything designed or sold for
incorporation into a dwelling. In determining whether a product is a
consumer product, doubtful cases shall be resolved in favor of
coverage. For a particular product received by a particular user,
"normally used" refers to a typical or common use of that class of
product, regardless of the status of the particular user or of the way
in which the particular user actually uses, or expects or is expected
to use, the product. A product is a consumer product regardless of
whether the product has substantial commercial, industrial or
non-consumer uses, unless such uses represent the only significant
mode of use of the product.

"Installation Information" for a User Product means any methods,
procedures, authorization keys, or other information required to
install and execute modified versions of a covered work in that User
Product from a modified version of its Corresponding Source. The
information must suffice to ensure that the continued functioning of
the modified object code is in no case prevented or interfered with
solely because modification has been made.

If you convey an object code work under this section in, or with, or
specifically for use in, a User Product, and the conveying occurs as
part of a transaction in which the right of possession and use of the
User Product is transferred to the recipient in perpetuity or for a
fixed term (regardless of how the transaction is characterized), the
Corresponding Source conveyed under this section must be accompanied
by the Installation Information. But this requirement does not apply
if neither you nor any third party retains the ability to install
modified object code on the User Product (for example, the work has
been installed in ROM).

The requirement to provide Installation Information does not include a
requirement to continue to provide support service, warranty, or
updates for a work that has been modified or installed by the
recipient, or for the User Product in which it has been modified or
installed. Access to a network may be denied when the modification
itself materially and adversely affects the operation of the network
or violates the rules and protocols for communication across the
network.

Corresponding Source conveyed, and Installation Information provided,
in accord with this section must be in a format that is publicly
documented (and with an implementation available to the public in
source code form), and must require no special password or key for
unpacking, reading or copying.

### 7. Additional Terms.

"Additional permissions" are terms that supplement the terms of this
License by making exceptions from one or more of its conditions.
Additional permissions that are applicable to the entire Program shall
be treated as though they were included in this License, to the extent
that they are valid under applicable law. If additional permissions
apply only to part of the Program, that part may be used separately
under those permissions, but the entire Program remains governed by
this License without regard to the additional permissions.

When you convey a copy of a covered work, you may at your option
remove any additional permissions from that copy, or from any part of
it. (Additional permissions may be written to require their own
removal in certain cases when you modify the work.) You may place
additional permissions on material, added by you to a covered work,
for which you have or can give appropriate copyright permission.

Notwithstanding any other provision of this License, for material you
add to a covered work, you may (if authorized by the copyright holders
of that material) supplement the terms of this License with terms:

-   a) Disclaiming warranty or limiting liability differently from the
    terms of sections 15 and 16 of this License; or
-   b) Requiring preservation of specified reasonable legal notices or
    author attributions in that material or in the Appropriate Legal
    Notices displayed by works containing it; or
-   c) Prohibiting misrepresentation of the origin of that material,
    or requiring that modified versions of such material be marked in
    reasonable ways as different from the original version; or
-   d) Limiting the use for publicity purposes of names of licensors
    or authors of the material; or
-   e) Declining to grant rights under trademark law for use of some
    trade names, trademarks, or service marks; or
-   f) Requiring indemnification of licensors and authors of that
    material by anyone who conveys the material (or modified versions
    of it) with contractual assumptions of liability to the recipient,
    for any liability that these contractual assumptions directly
    impose on those licensors and authors.

All other non-permissive additional terms are considered "further
restrictions" within the meaning of section 10. If the Program as you
received it, or any part of it, contains a notice stating that it is
governed by this License along with a term that is a further
restriction, you may remove that term. If a license document contains
a further restriction but permits relicensing or conveying under this
License, you may add to a covered work material governed by the terms
of that license document, provided that the further restriction does
not survive such relicensing or conveying.

If you add terms to a covered work in accord with this section, you
must place, in the relevant source files, a statement of the
additional terms that apply to those files, or a notice indicating
where to find the applicable terms.

Additional terms, permissive or non-permissive, may be stated in the
form of a separately written license, or stated as exceptions; the
above requirements apply either way.

### 8. Termination.

You may not propagate or modify a covered work except as expressly
provided under this License. Any attempt otherwise to propagate or
modify it is void, and will automatically terminate your rights under
this License (including any patent licenses granted under the third
paragraph of section 11).

However, if you cease all violation of this License, then your license
from a particular copyright holder is reinstated (a) provisionally,
unless and until the copyright holder explicitly and finally
terminates your license, and (b) permanently, if the copyright holder
fails to notify you of the violation by some reasonable means prior to
60 days after the cessation.

Moreover, your license from a particular copyright holder is
reinstated permanently if the copyright holder notifies you of the
violation by some reasonable means, this is the first time you have
received notice of violation of this License (for any work) from that
copyright holder, and you cure the violation prior to 30 days after
your receipt of the notice.

Termination of your rights under this section does not terminate the
licenses of parties who have received copies or rights from you under
this License. If your rights have been terminated and not permanently
reinstated, you do not qualify to receive new licenses for the same
material under section 10.

### 9. Acceptance Not Required for Having Copies.

You are not required to accept this License in order to receive or run
a copy of the Program. Ancillary propagation of a covered work
occurring solely as a consequence of using peer-to-peer transmission
to receive a copy likewise does not require acceptance. However,
nothing other than this License grants you permission to propagate or
modify any covered work. These actions infringe copyright if you do
not accept this License. Therefore, by modifying or propagating a
covered work, you indicate your acceptance of this License to do so.

### 10. Automatic Licensing of Downstream Recipients.

Each time you convey a covered work, the recipient automatically
receives a license from the original licensors, to run, modify and
propagate that work, subject to this License. You are not responsible
for enforcing compliance by third parties with this License.

An "entity transaction" is a transaction transferring control of an
organization, or substantially all assets of one, or subdividing an
organization, or merging organizations. If propagation of a covered
work results from an entity transaction, each party to that
transaction who receives a copy of the work also receives whatever
licenses to the work the party's predecessor in interest had or could
give under the previous paragraph, plus a right to possession of the
Corresponding Source of the work from the predecessor in interest, if
the predecessor has it or can get it with reasonable efforts.

You may not impose any further restrictions on the exercise of the
rights granted or affirmed under this License. For example, you may
not impose a license fee, royalty, or other charge for exercise of
rights granted under this License, and you may not initiate litigation
(including a cross-claim or counterclaim in a lawsuit) alleging that
any patent claim is infringed by making, using, selling, offering for
sale, or importing the Program or any portion of it.

### 11. Patents.

A "contributor" is a copyright holder who authorizes use under this
License of the Program or a work on which the Program is based. The
work thus licensed is called the contributor's "contributor version".

A contributor's "essential patent claims" are all patent claims owned
or controlled by the contributor, whether already acquired or
hereafter acquired, that would be infringed by some manner, permitted
by this License, of making, using, or selling its contributor version,
but do not include claims that would be infringed only as a
consequence of further modification of the contributor version. For
purposes of this definition, "control" includes the right to grant
patent sublicenses in a manner consistent with the requirements of
this License.

Each contributor grants you a non-exclusive, worldwide, royalty-free
patent license under the contributor's essential patent claims, to
make, use, sell, offer for sale, import and otherwise run, modify and
propagate the contents of its contributor version.

In the following three paragraphs, a "patent license" is any express
agreement or commitment, however denominated, not to enforce a patent
(such as an express permission to practice a patent or covenant not to
sue for patent infringement). To "grant" such a patent license to a
party means to make such an agreement or commitment not to enforce a
patent against the party.

If you convey a covered work, knowingly relying on a patent license,
and the Corresponding Source of the work is not available for anyone
to copy, free of charge and under the terms of this License, through a
publicly available network server or other readily accessible means,
then you must either (1) cause the Corresponding Source to be so
available, or (2) arrange to deprive yourself of the benefit of the
patent license for this particular work, or (3) arrange, in a manner
consistent with the requirements of this License, to extend the patent
license to downstream recipients. "Knowingly relying" means you have
actual knowledge that, but for the patent license, your conveying the
covered work in a country, or your recipient's use of the covered work
in a country, would infringe one or more identifiable patents in that
country that you have reason to believe are valid.

If, pursuant to or in connection with a single transaction or
arrangement, you convey, or propagate by procuring conveyance of, a
covered work, and grant a patent license to some of the parties
receiving the covered work authorizing them to use, propagate, modify
or convey a specific copy of the covered work, then the patent license
you grant is automatically extended to all recipients of the covered
work and works based on it.

A patent license is "discriminatory" if it does not include within the
scope of its coverage, prohibits the exercise of, or is conditioned on
the non-exercise of one or more of the rights that are specifically
granted under this License. You may not convey a covered work if you
are a party to an arrangement with a third party that is in the
business of distributing software, under which you make payment to the
third party based on the extent of your activity of conveying the
work, and under which the third party grants, to any of the parties
who would receive the covered work from you, a discriminatory patent
license (a) in connection with copies of the covered work conveyed by
you (or copies made from those copies), or (b) primarily for and in
connection with specific products or compilations that contain the
covered work, unless you entered into that arrangement, or that patent
license was granted, prior to 28 March 2007.

Nothing in this License shall be construed as excluding or limiting
any implied license or other defenses to infringement that may
otherwise be available to you under applicable patent law.

### 12. No Surrender of Others' Freedom.

If conditions are imposed on you (whether by court order, agreement or
otherwise) that contradict the conditions of this License, they do not
excuse you from the conditions of this License. If you cannot convey a
covered work so as to satisfy simultaneously your obligations under
this License and any other pertinent obligations, then as a
consequence you may not convey it at all. For example, if you agree to
terms that obligate you to collect a royalty for further conveying
from those to whom you convey the Program, the only way you could
satisfy both those terms and this License would be to refrain entirely
from conveying the Program.

### 13. Use with the GNU Affero General Public License.

Notwithstanding any other provision of this License, you have
permission to link or combine any covered work with a work licensed
under version 3 of the GNU Affero General Public License into a single
combined work, and to convey the resulting work. The terms of this
License will continue to apply to the part which is the covered work,
but the special requirements of the GNU Affero General Public License,
section 13, concerning interaction through a network will apply to the
combination as such.

### 14. Revised Versions of this License.

The Free Software Foundation may publish revised and/or new versions
of the GNU General Public License from time to time. Such new versions
will be similar in spirit to the present version, but may differ in
detail to address new problems or concerns.

Each version is given a distinguishing version number. If the Program
specifies that a certain numbered version of the GNU General Public
License "or any later version" applies to it, you have the option of
following the terms and conditions either of that numbered version or
of any later version published by the Free Software Foundation. If the
Program does not specify a version number of the GNU General Public
License, you may choose any version ever published by the Free
Software Foundation.

If the Program specifies that a proxy can decide which future versions
of the GNU General Public License can be used, that proxy's public
statement of acceptance of a version permanently authorizes you to
choose that version for the Program.

Later license versions may give you additional or different
permissions. However, no additional obligations are imposed on any
author or copyright holder as a result of your choosing to follow a
later version.

### 15. Disclaimer of Warranty.

THERE IS NO WARRANTY FOR THE PROGRAM, TO THE EXTENT PERMITTED BY
APPLICABLE LAW. EXCEPT WHEN OTHERWISE STATED IN WRITING THE COPYRIGHT
HOLDERS AND/OR OTHER PARTIES PROVIDE THE PROGRAM "AS IS" WITHOUT
WARRANTY OF ANY KIND, EITHER EXPRESSED OR IMPLIED, INCLUDING, BUT NOT
LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
A PARTICULAR PURPOSE. THE ENTIRE RISK AS TO THE QUALITY AND
PERFORMANCE OF THE PROGRAM IS WITH YOU. SHOULD THE PROGRAM PROVE
DEFECTIVE, YOU ASSUME THE COST OF ALL NECESSARY SERVICING, REPAIR OR
CORRECTION.

### 16. Limitation of Liability.

IN NO EVENT UNLESS REQUIRED BY APPLICABLE LAW OR AGREED TO IN WRITING
WILL ANY COPYRIGHT HOLDER, OR ANY OTHER PARTY WHO MODIFIES AND/OR
CONVEYS THE PROGRAM AS PERMITTED ABOVE, BE LIABLE TO YOU FOR DAMAGES,
INCLUDING ANY GENERAL, SPECIAL, INCIDENTAL OR CONSEQUENTIAL DAMAGES
ARISING OUT OF THE USE OR INABILITY TO USE THE PROGRAM (INCLUDING BUT
NOT LIMITED TO LOSS OF DATA OR DATA BEING RENDERED INACCURATE OR
LOSSES SUSTAINED BY YOU OR THIRD PARTIES OR A FAILURE OF THE PROGRAM
TO OPERATE WITH ANY OTHER PROGRAMS), EVEN IF SUCH HOLDER OR OTHER
PARTY HAS BEEN ADVISED OF THE POSSIBILITY OF SUCH DAMAGES.

### 17. Interpretation of Sections 15 and 16.

If the disclaimer of warranty and limitation of liability provided
above cannot be given local legal effect according to their terms,
reviewing courts shall apply local law that most closely approximates
an absolute waiver of all civil liability in connection with the
Program, unless a warranty or assumption of liability accompanies a
copy of the Program in return for a fee.

END OF TERMS AND CONDITIONS

## How to Apply These Terms to Your New Programs

If you develop a new program, and you want it to be of the greatest
possible use to the public, the best way to achieve this is to make it
free software which everyone can redistribute and change under these
terms.

To do so, attach the following notices to the program. It is safest to
attach them to the start of each source file to most effectively state
the exclusion of warranty; and each file should have at least the
"copyright" line and a pointer to where the full notice is found.

        <one line to give the program's name and a brief idea of what it does.>
        Copyright (C) <year>  <name of author>

        This program is free software: you can redistribute it and/or modify
        it under the terms of the GNU General Public License as published by
        the Free Software Foundation, either version 3 of the License, or
        (at your option) any later version.

        This program is distributed in the hope that it will be useful,
        but WITHOUT ANY WARRANTY; without even the implied warranty of
        MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
        GNU General Public License for more details.

        You should have received a copy of the GNU General Public License
        along with this program.  If not, see <https://www.gnu.org/licenses/>.

Also add information on how to contact you by electronic and paper
mail.

If the program does terminal interaction, make it output a short
notice like this when it starts in an interactive mode:

        <program>  Copyright (C) <year>  <name of author>
        This program comes with ABSOLUTELY NO WARRANTY; for details type `show w'.
        This is free software, and you are welcome to redistribute it
        under certain conditions; type `show c' for details.

The hypothetical commands \`show w' and \`show c' should show the
appropriate parts of the General Public License. Of course, your
program's commands might be different; for a GUI interface, you would
use an "about box".

You should also get your employer (if you work as a programmer) or
school, if any, to sign a "copyright disclaimer" for the program, if
necessary. For more information on this, and how to apply and follow
the GNU GPL, see <https://www.gnu.org/licenses/>.

The GNU General Public License does not permit incorporating your
program into proprietary programs. If your program is a subroutine
library, you may consider it more useful to permit linking proprietary
applications with the library. If this is what you want to do, use the
GNU Lesser General Public License instead of this License. But first,
please read <https://www.gnu.org/licenses/why-not-lgpl.html>.
```

*/

/* License details for parts derived from glibc:

From stdlib/random.c:

```
/* Copyright (C) 1995-2026 Free Software Foundation, Inc.

   The GNU C Library is free software; you can redistribute it and/or
   modify it under the terms of the GNU Lesser General Public
   License as published by the Free Software Foundation; either
   version 2.1 of the License, or (at your option) any later version.

   The GNU C Library is distributed in the hope that it will be useful,
   but WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
   Lesser General Public License for more details.

   You should have received a copy of the GNU Lesser General Public
   License along with the GNU C Library; if not, see
   <https://www.gnu.org/licenses/>.  */

/*
 * This is derived from the Berkeley source:
 *	@(#)random.c	5.5 (Berkeley) 7/6/88
 * It was reworked for the GNU C Library by Roland McGrath.
 * Rewritten to use reentrant functions by Ulrich Drepper, 1995.
 */

/*
   Copyright (C) 1983 Regents of the University of California.
   All rights reserved.

   Redistribution and use in source and binary forms, with or without
   modification, are permitted provided that the following conditions
   are met:

   1. Redistributions of source code must retain the above copyright
      notice, this list of conditions and the following disclaimer.
   2. Redistributions in binary form must reproduce the above copyright
      notice, this list of conditions and the following disclaimer in the
      documentation and/or other materials provided with the distribution.
   4. Neither the name of the University nor the names of its contributors
      may be used to endorse or promote products derived from this software
      without specific prior written permission.

   THIS SOFTWARE IS PROVIDED BY THE REGENTS AND CONTRIBUTORS ``AS IS'' AND
   ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
   IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
   ARE DISCLAIMED.  IN NO EVENT SHALL THE REGENTS OR CONTRIBUTORS BE LIABLE
   FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
   DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS
   OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
   HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT
   LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY
   OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
   SUCH DAMAGE.*/
```

From stdlib/random_r.c:

```
/*
   Copyright (C) 1995-2026 Free Software Foundation, Inc.

   The GNU C Library is free software; you can redistribute it and/or
   modify it under the terms of the GNU Lesser General Public
   License as published by the Free Software Foundation; either
   version 2.1 of the License, or (at your option) any later version.

   The GNU C Library is distributed in the hope that it will be useful,
   but WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
   Lesser General Public License for more details.

   You should have received a copy of the GNU Lesser General Public
   License along with the GNU C Library; if not, see
   <https://www.gnu.org/licenses/>.  */

/*
   Copyright (C) 1983 Regents of the University of California.
   All rights reserved.

   Redistribution and use in source and binary forms, with or without
   modification, are permitted provided that the following conditions
   are met:

   1. Redistributions of source code must retain the above copyright
      notice, this list of conditions and the following disclaimer.
   2. Redistributions in binary form must reproduce the above copyright
      notice, this list of conditions and the following disclaimer in the
      documentation and/or other materials provided with the distribution.
   4. Neither the name of the University nor the names of its contributors
      may be used to endorse or promote products derived from this software
      without specific prior written permission.

   THIS SOFTWARE IS PROVIDED BY THE REGENTS AND CONTRIBUTORS ``AS IS'' AND
   ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
   IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
   ARE DISCLAIMED.  IN NO EVENT SHALL THE REGENTS OR CONTRIBUTORS BE LIABLE
   FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
   DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS
   OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
   HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT
   LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY
   OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
   SUCH DAMAGE.*/

/*
 * This is derived from the Berkeley source:
 *	@(#)random.c	5.5 (Berkeley) 7/6/88
 * It was reworked for the GNU C Library by Roland McGrath.
 * Rewritten to be reentrant by Ulrich Drepper, 1995
 */
```
*/