//! Implementation of System III/V `pack` (`0x1F 0x1E`) and Early Unix `old_pack`
//! (`0x1F 0x1F`) compression and decompression formats.
//!
//! Specification reference: `data/docs/pack.md`
//! Reference decompressor: `old/unix-tools/gzip-1.14/gzip-1.14/unpack.c`

// SPDX-License-Identifier for parts derived from from gzip: GPL-3.0-or-later

// Header comment from original unpack.c:
/* unpack.c -- decompress files in pack format.

  Copyright (C) 1997, 1999, 2006, 2009-2025 Free Software Foundation, Inc.
  Copyright (C) 1992-1993 Jean-loup Gailly

  This program is free software; you can redistribute it and/or modify
  it under the terms of the GNU General Public License as published by
  the Free Software Foundation; either version 3, or (at your option)
  any later version.

  This program is distributed in the hope that it will be useful,
  but WITHOUT ANY WARRANTY; without even the implied warranty of
  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
  GNU General Public License for more details.

  You should have received a copy of the GNU General Public License
  along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

// AUTHORS file for gzip overall:
/* gzip was written by Jean-loup Gailly <jloup@gzip.org>,
and Mark Adler for the decompression code. */

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::io::{Read, Write};

/// Magic header bytes for standard System III/V `pack` (`0x1F`, `0x1E`).
pub const PACK_MAGIC: [u8; 2] = [0x1F, 0x1E];

/// Magic header bytes for early Unix `old_pack` (`0x1F`, `0x1F`).
pub const OLD_PACK_MAGIC: [u8; 2] = [0x1F, 0x1F];

const MAX_BITLEN: usize = 24;
const LITERALS: usize = 256;
const END_SYMBOL: usize = 256;

// -----------------------------------------------------------------------------
// Helper struct for MSB-to-LSB Bit Reader (8-bit bytes)
// -----------------------------------------------------------------------------

struct BitReader<R: Read> {
    reader: R,
    bitbuf: u32,
    valid: u32,
}

impl<R: Read> BitReader<R> {
    fn new(reader: R) -> Self {
        Self {
            reader,
            bitbuf: 0,
            valid: 0,
        }
    }

    fn read_byte(&mut self) -> Result<u8> {
        let mut buf = [0u8; 1];
        let n = self
            .reader
            .read(&mut buf)
            .context("Failed to read byte from stream")?;
        if n == 0 {
            bail!("Unexpected end of compressed stream");
        }
        let byte = match buf.first() {
            Some(&b) => b,
            None => bail!("Unexpected empty read buffer"),
        };
        Ok(byte)
    }

    fn get_bit(&mut self) -> Result<u32> {
        if self.valid == 0 {
            let byte = self.read_byte()?;
            self.bitbuf = u32::from(byte);
            self.valid = 8;
        }
        let shift = self.valid.saturating_sub(1);
        let bit = (self.bitbuf >> shift) & 1;
        self.valid = shift;
        Ok(bit)
    }
}

// -----------------------------------------------------------------------------
// Helper struct for MSB-to-LSB Bit Writer (8-bit bytes)
// -----------------------------------------------------------------------------

struct BitWriter<W: Write> {
    writer: W,
    bitbuf: u32,
    valid: u32,
}

impl<W: Write> BitWriter<W> {
    fn new(writer: W) -> Self {
        Self {
            writer,
            bitbuf: 0,
            valid: 0,
        }
    }

    fn write_bits(&mut self, code: u32, bits: u32) -> Result<()> {
        let mut rem_bits = bits;
        while rem_bits > 0 {
            let space = 8u32.saturating_sub(self.valid);
            let take = rem_bits.min(space);
            let shift = rem_bits.saturating_sub(take);
            let mask = (1u32
                .checked_shl(take)
                .context("bit shift out of range")?)
            .saturating_sub(1);
            let val = (code >> shift) & mask;

            self.bitbuf = (self.bitbuf << take) | val;
            self.valid = self.valid.saturating_add(take);
            rem_bits = shift;

            if self.valid == 8 {
                let byte = u8::try_from(self.bitbuf & 0xFF)?;
                self.writer
                    .write_all(&[byte])
                    .context("Failed to write bitstream byte")?;
                self.bitbuf = 0;
                self.valid = 0;
            }
        }
        Ok(())
    }

    fn flush_bits(&mut self) -> Result<()> {
        if self.valid > 0 {
            let shift = 8u32.saturating_sub(self.valid);
            let byte = u8::try_from((self.bitbuf << shift) & 0xFF)?;
            self.writer
                .write_all(&[byte])
                .context("Failed to flush bitstream byte")?;
            self.bitbuf = 0;
            self.valid = 0;
        }
        Ok(())
    }
}

// -----------------------------------------------------------------------------
// System III / System V Pack Decompression
// -----------------------------------------------------------------------------

/// Decompresses a standard System III/V `pack` stream (`0x1F 0x1E`).
pub fn decompress_pack_stream(
    reader: &mut impl Read,
    writer: &mut impl Write,
) -> Result<u64> {
    let mut br = BitReader::new(reader);

    // Read and verify magic header bytes (0x1F 0x1E)
    let magic0 = br.read_byte()?;
    let magic1 = br.read_byte()?;
    if magic0 != PACK_MAGIC[0] || magic1 != PACK_MAGIC[1] {
        bail!(
            "Invalid Pack magic header: expected 0x1F 0x1E, got 0x{magic0:02X} 0x{magic1:02X}"
        );
    }

    // Read 32-bit Big-Endian original uncompressed size
    let b2 = u32::from(br.read_byte()?);
    let b3 = u32::from(br.read_byte()?);
    let b4 = u32::from(br.read_byte()?);
    let b5 = u32::from(br.read_byte()?);
    let orig_size = (b2 << 24) | (b3 << 16) | (b4 << 8) | b5;

    // Read maximum tree depth (maxlev)
    let max_len_byte = br.read_byte()?;
    let max_len = usize::from(max_len_byte);
    if max_len == 0 || max_len > MAX_BITLEN {
        bail!(
            "Invalid Pack tree depth: maxlev {max_len} is out of range 1..={MAX_BITLEN}"
        );
    }

    // Read leaf counts for each level 1..=max_len
    let mut leaves = vec![0i32; max_len.saturating_add(1)];
    let mut total_leaves = 0usize;
    for len in 1..=max_len {
        let count = usize::from(br.read_byte()?);
        let count_i32 = i32::try_from(count)?;
        let leaf_slot = leaves.get_mut(len).context("Invalid leaves index")?;
        *leaf_slot = count_i32;
        total_leaves = total_leaves.saturating_add(count);
    }

    // Adjust stored levcount[max_len]: stored as actual - 2 + 1 (implicit EOB offset)
    let max_len_count =
        leaves.get_mut(max_len).context("Invalid max_len index")?;
    *max_len_count = max_len_count.saturating_add(1);

    let explicit_leaf_count = total_leaves.saturating_add(1);
    let mut literal = vec![0u8; explicit_leaf_count];
    let mut base = 0usize;
    let mut lit_base = vec![0i32; max_len.saturating_add(1)];

    for len in 1..=max_len {
        let l_base = lit_base.get_mut(len).context("Invalid lit_base index")?;
        *l_base = i32::try_from(base)?;
        let count =
            usize::try_from(*leaves.get(len).context("Invalid leaves index")?)?;
        for _ in 0..count {
            let sym = br.read_byte()?;
            if base < literal.len() {
                let slot =
                    literal.get_mut(base).context("Invalid literal slot")?;
                *slot = sym;
                base = base.saturating_add(1);
            }
        }
    }

    // Include implicit EOB leaf at maximum level
    let max_len_count_final =
        leaves.get_mut(max_len).context("Invalid max_len index")?;
    *max_len_count_final = max_len_count_final.saturating_add(1);

    // Build internal nodes count table and adjust lit_base
    let mut parents = vec![0i32; max_len.saturating_add(1)];
    let mut nodes = 0i32;
    for len in (1..=max_len).rev() {
        nodes >>= 1;
        let p_slot = parents.get_mut(len).context("Invalid parents slot")?;
        *p_slot = nodes;
        let l_slot = lit_base.get_mut(len).context("Invalid lit_base slot")?;
        *l_slot = l_slot.saturating_sub(nodes);
        let l_count = *leaves.get(len).context("Invalid leaves count")?;
        nodes = nodes.saturating_add(l_count);
    }

    let eob_code = (*leaves.get(max_len).context("Invalid max_len leaves")?)
        .saturating_sub(1);

    let mut bytes_emitted = 0u64;
    let orig_size_u64 = u64::from(orig_size);

    if orig_size_u64 == 0 {
        return Ok(0);
    }

    let mut code = 0i32;
    let mut len = 1usize;

    loop {
        let bit = i32::try_from(br.get_bit()?)?;
        code = (code << 1) | bit;

        let p_val = *parents.get(len).context("Invalid parents len")?;
        if code >= p_val {
            if len == max_len && code == eob_code {
                break;
            }
            let l_base = *lit_base.get(len).context("Invalid lit_base len")?;
            let idx = usize::try_from(code.saturating_add(l_base))?;
            let sym =
                *literal.get(idx).context("Literal index out of bounds")?;

            writer
                .write_all(&[sym])
                .context("Failed to write decompressed byte")?;
            bytes_emitted = bytes_emitted.saturating_add(1);

            if bytes_emitted == orig_size_u64 {
                break;
            }

            code = 0;
            len = 1;
        } else {
            len = len.saturating_add(1);
            if len > max_len {
                bail!("Huffman code length exceeded maximum level {max_len}");
            }
        }
    }

    if bytes_emitted != orig_size_u64 {
        bail!(
            "Decompressed byte count mismatch: expected {orig_size}, got {bytes_emitted}"
        );
    }

    Ok(bytes_emitted)
}

// -----------------------------------------------------------------------------
// System III / System V Pack Compression
// -----------------------------------------------------------------------------

#[derive(Eq, PartialEq)]
struct HuffmanNode {
    freq: u64,
    symbol: Option<usize>,
    left: Option<Box<HuffmanNode>>,
    right: Option<Box<HuffmanNode>>,
}

impl Ord for HuffmanNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other.freq.cmp(&self.freq)
    }
}

impl PartialOrd for HuffmanNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn collect_depths(node: &HuffmanNode, depth: usize, depths: &mut [usize; 257]) {
    if let Some(sym) = node.symbol {
            if let Some(slot) = depths.get_mut(sym) {
                *slot = depth;
            }
    } else {
        if let Some(ref l) = node.left {
            collect_depths(l, depth.saturating_add(1), depths);
        }
        if let Some(ref r) = node.right {
            collect_depths(r, depth.saturating_add(1), depths);
        }
    }
}

/// Compresses a stream into standard System III/V `pack` format (`0x1F 0x1E`). Not an actual streaming compressor; requires sufficient memory.
pub fn compress_pack_stream(
    reader: &mut impl Read,
    writer: &mut impl Write,
) -> Result<u64> {
    let mut input_bytes = Vec::new();
    reader
        .read_to_end(&mut input_bytes)
        .context("Failed to read input data for Pack compression")?;

    let orig_size = input_bytes.len();
    let orig_size_u32 = u32::try_from(orig_size).context(
        "Input data size exceeds 32-bit uint capacity for Pack header",
    )?;

    // Frequency counting
    let mut freqs = [0u64; 257];
    for &b in &input_bytes {
        let idx = usize::from(b);
        let count = freqs.get_mut(idx).context("Invalid frequency index")?;
        *count = count.saturating_add(1);
    }
    // EOB symbol (256) always present with frequency 1
    let eob_slot = freqs.get_mut(END_SYMBOL).context("Invalid EOB slot")?;
    *eob_slot = 1;

    // Build min-heap for Huffman tree
    let mut heap = BinaryHeap::new();
    for (sym, &freq) in freqs.iter().enumerate() {
        if freq > 0 {
            heap.push(HuffmanNode {
                freq,
                symbol: Some(sym),
                left: None,
                right: None,
            });
        }
    }

    // Ensure at least 2 leaves in tree
    if heap.len() < 2 {
        for dummy_sym in 0..256 {
            // Reason for fallback: dummy_sym is bounded by 0..256 so index is within 256-element freqs array bounds.
            if freqs.get(dummy_sym).copied().unwrap_or(0) == 0 {
                heap.push(HuffmanNode {
                    freq: 1,
                    symbol: Some(dummy_sym),
                    left: None,
                    right: None,
                });
                if heap.len() >= 2 {
                    break;
                }
            }
        }
    }

    while heap.len() > 1 {
        let left = heap.pop().context("Heap underflow")?;
        let right = heap.pop().context("Heap underflow")?;
        let parent_freq = left.freq.saturating_add(right.freq);
        heap.push(HuffmanNode {
            freq: parent_freq,
            symbol: None,
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
        });
    }

    let root = heap.pop().context("Huffman tree root missing")?;
    let mut depths = [0usize; 257];
    collect_depths(&root, 0, &mut depths);

    // Reason for fallback: depths is a non-empty 257-element array so max() is guaranteed to return Some.
    let max_len = depths.iter().copied().max().unwrap_or(1).max(1);
    if max_len > MAX_BITLEN {
        bail!(
            "Generated Huffman code length {max_len} exceeds maximum allowed depth {MAX_BITLEN}"
        );
    }

    // Count leaves per level
    let mut leaf_counts = vec![0usize; max_len.saturating_add(1)];
    for &d in &depths {
        if d > 0 {
            let slot = leaf_counts
                .get_mut(d)
                .context("Invalid leaf_counts index")?;
            *slot = slot.saturating_add(1);
        }
    }

    // Group symbols by depth
    // Workaround for gzip bug #28861: symbol 256 (END) must be ordered last within its depth level
    let mut level_symbols =
        vec![Vec::<usize>::new(); max_len.saturating_add(1)];
    for (sym, &d) in depths.iter().enumerate() {
        if d > 0 {
            let vec = level_symbols
                .get_mut(d)
                .context("Invalid level_symbols level")?;
            vec.push(sym);
        }
    }

    for (d, syms) in level_symbols.iter_mut().enumerate() {
        if d > 0 {
            syms.sort_by(|&a, &b| {
                if a == END_SYMBOL {
                    Ordering::Greater
                } else if b == END_SYMBOL {
                    Ordering::Less
                } else {
                    a.cmp(&b)
                }
            });
        }
    }

    // Calculate canonical parents count per level
    let mut parents = vec![0i32; max_len.saturating_add(1)];
    let p_max = parents
        .get_mut(max_len)
        .context("Invalid parents max_len")?;
    *p_max = 0;
    for len in (1..max_len).rev() {
        let p_next = *parents
            .get(len.saturating_add(1))
            .context("Invalid parent next")?;
        let l_next = i32::try_from(
            *leaf_counts
                .get(len.saturating_add(1))
                .context("Invalid leaf next")?,
        )?;
        let p_curr = parents.get_mut(len).context("Invalid parent curr")?;
        *p_curr = p_next.saturating_add(l_next) / 2;
    }

    // Assign codes to symbols
    let mut code_map = [(0u32, 0u32); 257];
    for len in 1..=max_len {
        let base_code =
            u32::try_from(*parents.get(len).context("Invalid parents len")?)?;
        let syms = level_symbols
            .get(len)
            .context("Invalid level_symbols len")?;
        for (idx, &sym) in syms.iter().enumerate() {
            let sym_code = base_code.saturating_add(u32::try_from(idx)?);
            let len_u32 = u32::try_from(len)?;
            let map_slot =
                code_map.get_mut(sym).context("Invalid code_map slot")?;
            *map_slot = (sym_code, len_u32);
        }
    }

    // Write header
    writer
        .write_all(&PACK_MAGIC)
        .context("Failed to write Pack magic")?;
    let orig_be = orig_size_u32.to_be_bytes();
    writer
        .write_all(&orig_be)
        .context("Failed to write orig_size")?;

    let max_len_u8 = u8::try_from(max_len)?;
    writer
        .write_all(&[max_len_u8])
        .context("Failed to write maxlev")?;

    // Write leaf counts array
    for len in 1..=max_len {
        let count = *leaf_counts.get(len).context("Invalid leaf_counts len")?;
        let store_val = if len == max_len {
            count.saturating_sub(2)
        } else {
            count
        };
        let val_u8 = u8::try_from(store_val)?;
        writer
            .write_all(&[val_u8])
            .context("Failed to write level count")?;
    }

    // Write literal symbol bytes (excluding EOB 256)
    for len in 1..=max_len {
        let syms = level_symbols
            .get(len)
            .context("Invalid level_symbols len")?;
        for &sym in syms {
            if sym != END_SYMBOL {
                let byte = u8::try_from(sym)?;
                writer
                    .write_all(&[byte])
                    .context("Failed to write symbol byte")?;
            }
        }
    }

    // Write MSB-to-LSB bitstream
    let mut bw = BitWriter::new(writer);
    for &b in &input_bytes {
        let sym = usize::from(b);
        let &(code, bits) =
            code_map.get(sym).context("Symbol missing from code map")?;
        bw.write_bits(code, bits)?;
    }
    // Write EOB symbol
    let &(eob_code, eob_bits) = code_map
        .get(END_SYMBOL)
        .context("EOB missing from code map")?;
    bw.write_bits(eob_code, eob_bits)?;
    bw.flush_bits()?;

    Ok(u64::try_from(orig_size)?)
}

// -----------------------------------------------------------------------------
// Early PDP-11 OldPack Decompression (`0x1F 0x1F`)
// -----------------------------------------------------------------------------
/// Decompresses an early PDP-11 `old_pack` stream (`0x1F 0x1F`).
pub fn decompress_old_pack_stream(
    reader: &mut impl Read,
    writer: &mut impl Write,
) -> Result<u64> {
    let mut magic = [0u8; 2];
    reader
        .read_exact(&mut magic)
        .context("Failed to read OldPack magic header")?;
    if magic[0] != OLD_PACK_MAGIC[0] || magic[1] != OLD_PACK_MAGIC[1] {
        bail!(
            "Invalid OldPack magic header: expected 0x1F 0x1F, got 0x{:02X} 0x{:02X}",
            magic[0],
            magic[1]
        );
    }

    // Read 32-bit PDP-11 Middle-Endian uncompressed size
    let mut size_buf = [0u8; 4];
    reader
        .read_exact(&mut size_buf)
        .context("Failed to read OldPack origsize")?;
    // Reason for fallback: size_buf is a 4-byte array so indices 0..4 are guaranteed to be in bounds.
    let hi_b0 = size_buf.first().copied().unwrap_or(0);
    // Reason for fallback: size_buf is a 4-byte array so indices 0..4 are guaranteed to be in bounds.
    let hi_b1 = size_buf.get(1).copied().unwrap_or(0);
    // Reason for fallback: size_buf is a 4-byte array so indices 0..4 are guaranteed to be in bounds.
    let lo_b0 = size_buf.get(2).copied().unwrap_or(0);
    // Reason for fallback: size_buf is a 4-byte array so indices 0..4 are guaranteed to be in bounds.
    let lo_b1 = size_buf.get(3).copied().unwrap_or(0);

    let hi = u32::from(u16::from_le_bytes([hi_b0, hi_b1]));
    let lo = u32::from(u16::from_le_bytes([lo_b0, lo_b1]));
    let orig_size = (hi << 16) | lo;
    let orig_size_u64 = u64::from(orig_size);

    if orig_size_u64 == 0 {
        return Ok(0);
    }

    // Read 16-bit LE keysize
    let mut key_buf = [0u8; 2];
    reader
        .read_exact(&mut key_buf)
        .context("Failed to read OldPack keysize")?;
    let keysize = usize::from(u16::from_le_bytes(key_buf));

    if keysize == 0 || keysize > 4096 {
        bail!("Invalid OldPack keysize: {keysize}");
    }

    // Decode compressed tree dictionary into tree_words[]
    let mut tree_words = vec![0u16; keysize];
    let mut t = 0usize;
    while t < keysize {
        let mut b_buf = [0u8; 1];
        reader
            .read_exact(&mut b_buf)
            .context("Failed to read OldPack dictionary byte")?;
        // Reason for fallback: b_buf is a 1-byte array so index 0 is guaranteed to be in bounds.
        let b = b_buf.first().copied().unwrap_or(0);
        if b == 0xFF {
            let mut w_buf = [0u8; 2];
            reader
                .read_exact(&mut w_buf)
                .context("Failed to read OldPack dictionary word")?;
            let word = u16::from_le_bytes(w_buf);
            let slot =
                tree_words.get_mut(t).context("Invalid tree_words index")?;
            *slot = word;
        } else {
            let slot =
                tree_words.get_mut(t).context("Invalid tree_words index")?;
            *slot = u16::from(b);
        }
        t = t.saturating_add(1);
    }

    // Decompress bitstream reading 16-bit LE words (MSB to LSB bit order)
    let mut bytes_emitted = 0u64;
    let mut tp = 0usize;

    let mut word_buf = 0u16;
    let mut bits_left = 0u32;

    loop {
        let left_off =
            *tree_words.get(tp).context("Tree index tp out of bounds")?;
        let right_val = *tree_words
            .get(tp.saturating_add(1))
            .context("Tree index tp+1 out of bounds")?;

        if left_off == 0 {
            // Leaf node! right_val is literal character
            let sym = u8::try_from(right_val & 0xFF)?;
            writer
                .write_all(&[sym])
                .context("Failed to write decompressed byte")?;
            bytes_emitted = bytes_emitted.saturating_add(1);
            tp = 0;

            if bytes_emitted == orig_size_u64 {
                break;
            }
        } else {
            // Internal node! Fetch next bit
            if bits_left == 0 {
                let mut w_buf = [0u8; 2];
                let n = reader
                    .read(&mut w_buf)
                    .context("Failed to read word from bitstream")?;
                if n < 2 {
                    break;
                }
                word_buf = u16::from_le_bytes(w_buf);
                bits_left = 16;
            }

            let shift = bits_left.saturating_sub(1);
            let bit = (word_buf >> shift) & 1;
            bits_left = shift;

            let offset = if bit == 0 { left_off } else { right_val };
            tp = tp.saturating_add(usize::from(offset));
        }
    }

    if bytes_emitted != orig_size_u64 {
        bail!(
            "OldPack decompressed size mismatch: expected {orig_size}, got {bytes_emitted}"
        );
    }

    Ok(bytes_emitted)
}

// -----------------------------------------------------------------------------
// Early PDP-11 OldPack Compression (`0x1F 0x1F`)
// -----------------------------------------------------------------------------

/// Compresses a stream into early PDP-11 `old_pack` format (`0x1F 0x1F`).
pub fn compress_old_pack_stream(
    reader: &mut impl Read,
    writer: &mut impl Write,
) -> Result<u64> {
    let mut input_bytes = Vec::new();
    reader
        .read_to_end(&mut input_bytes)
        .context("Failed to read input data for OldPack compression")?;

    let orig_size = input_bytes.len();
    let orig_size_u32 = u32::try_from(orig_size).context(
        "Input data size exceeds 32-bit uint capacity for OldPack header",
    )?;

    // Frequency counting
    let mut freqs = [0u64; 256];
    for &b in &input_bytes {
        let idx = usize::from(b);
        let count = freqs.get_mut(idx).context("Invalid frequency index")?;
        *count = count.saturating_add(1);
    }

    // Build min-heap for explicit binary tree
    let mut heap = BinaryHeap::new();
    for (sym, &freq) in freqs.iter().enumerate() {
        if freq > 0 {
            heap.push(HuffmanNode {
                freq,
                symbol: Some(sym),
                left: None,
                right: None,
            });
        }
    }

    // Ensure at least 2 leaves in tree
    if heap.len() < 2 {
        for dummy_sym in 0..256 {
            // Reason for fallback: dummy_sym is bounded by 0..256 so index is within 256-element freqs array bounds.
            if freqs.get(dummy_sym).copied().unwrap_or(0) == 0 {
                heap.push(HuffmanNode {
                    freq: 1,
                    symbol: Some(dummy_sym),
                    left: None,
                    right: None,
                });
                if heap.len() >= 2 {
                    break;
                }
            }
        }
    }

    while heap.len() > 1 {
        let left = heap.pop().context("Heap underflow")?;
        let right = heap.pop().context("Heap underflow")?;
        let parent_freq = left.freq.saturating_add(right.freq);
        heap.push(HuffmanNode {
            freq: parent_freq,
            symbol: None,
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
        });
    }

    let root = heap.pop().context("Huffman tree root missing")?;

    let mut tree_words = Vec::<u16>::new();

    fn serialize_node(
        node: &HuffmanNode,
        tree_words: &mut Vec<u16>,
    ) -> Result<usize> {
        let tp = tree_words.len();
        tree_words.push(0);
        tree_words.push(0);

        if let Some(sym) = node.symbol {
            if let Some(slot_right) = tree_words.get_mut(tp.saturating_add(1))
            {
                *slot_right = u16::try_from(sym)
                    .context("Huffman symbol out of range for u16")?;
            }
        } else {
            let (Some(left_node), Some(right_node)) =
                (node.left.as_ref(), node.right.as_ref())
            else {
                return Ok(tp);
            };

            let left_tp = serialize_node(left_node, tree_words)?;
            let right_tp = serialize_node(right_node, tree_words)?;

            let left_offset = u16::try_from(left_tp.saturating_sub(tp))
                .context("Left node offset out of range for u16")?;
            let right_offset = u16::try_from(right_tp.saturating_sub(tp))
                .context("Right node offset out of range for u16")?;

            if let Some(slot_left) = tree_words.get_mut(tp) {
                *slot_left = left_offset;
            }
            if let Some(slot_right) = tree_words.get_mut(tp.saturating_add(1))
            {
                *slot_right = right_offset;
            }
        }
        Ok(tp)
    }

    serialize_node(&root, &mut tree_words)?;

    let keysize = u16::try_from(tree_words.len())
        .context("OldPack tree size exceeds 16-bit capacity")?;

    let mut symbol_bits = vec![(0u32, 0u32); 256];
    fn build_paths(
        node: &HuffmanNode,
        code: u32,
        len: u32,
        symbol_bits: &mut [(u32, u32)],
    ) {
        if let Some(sym) = node.symbol {
            if sym < 256 {
                if let Some(slot) = symbol_bits.get_mut(sym) {
                    *slot = (code, len);
                }
            }
        } else {
            if let Some(ref l) = node.left {
                build_paths(l, code << 1, len.saturating_add(1), symbol_bits);
            }
            if let Some(ref r) = node.right {
                build_paths(
                    r,
                    (code << 1) | 1,
                    len.saturating_add(1),
                    symbol_bits,
                );
            }
        }
    }
    build_paths(&root, 0, 0, &mut symbol_bits);

    // Write header
    writer
        .write_all(&OLD_PACK_MAGIC)
        .context("Failed to write OldPack magic")?;

    let hi = u16::try_from((orig_size_u32 >> 16) & 0xFFFF)?;
    let lo = u16::try_from(orig_size_u32 & 0xFFFF)?;
    writer
        .write_all(&hi.to_le_bytes())
        .context("Failed to write origsize hi")?;
    writer
        .write_all(&lo.to_le_bytes())
        .context("Failed to write origsize lo")?;

    writer
        .write_all(&keysize.to_le_bytes())
        .context("Failed to write keysize")?;

    // Write compressed tree dictionary array
    for &w in &tree_words {
        if w < 0xFF {
            let byte = u8::try_from(w)?;
            writer
                .write_all(&[byte])
                .context("Failed to write dictionary byte")?;
        } else {
            writer
                .write_all(&[0xFF])
                .context("Failed to write 0xFF escape")?;
            writer
                .write_all(&w.to_le_bytes())
                .context("Failed to write dictionary word")?;
        }
    }

    // Write bitstream in 16-bit LE words (MSB to LSB bit order)
    let mut word_buf = 0u16;
    let mut valid_bits = 0u32;

    for &b in &input_bytes {
        let sym = usize::from(b);
        let &(code, bits) = symbol_bits
            .get(sym)
            .context("Symbol missing in path table")?;

        let mut rem_bits = bits;
        while rem_bits > 0 {
            let space = 16u32.saturating_sub(valid_bits);
            let take = rem_bits.min(space);
            let shift = rem_bits.saturating_sub(take);
            let mask = (1u32
                .checked_shl(take)
                .context("bit shift out of range")?)
            .saturating_sub(1);
            let val = u16::try_from((code >> shift) & mask)?;

            word_buf = (word_buf << take) | val;
            valid_bits = valid_bits.saturating_add(take);
            rem_bits = shift;

            if valid_bits == 16 {
                writer
                    .write_all(&word_buf.to_le_bytes())
                    .context("Failed to write bitstream word")?;
                word_buf = 0;
                valid_bits = 0;
            }
        }
    }

    if valid_bits > 0 {
        let shift = 16u32.saturating_sub(valid_bits);
        word_buf <<= shift;
        writer
            .write_all(&word_buf.to_le_bytes())
            .context("Failed to flush bitstream word")?;
    }

    Ok(u64::try_from(orig_size)?)
}

/*

License for parts derived from gzip 1.14:

```
                    GNU GENERAL PUBLIC LICENSE
                       Version 3, 29 June 2007

 Copyright (C) 2007 Free Software Foundation, Inc. <https://fsf.org/>
 Everyone is permitted to copy and distribute verbatim copies
 of this license document, but changing it is not allowed.

                            Preamble

  The GNU General Public License is a free, copyleft license for
software and other kinds of works.

  The licenses for most software and other practical works are designed
to take away your freedom to share and change the works.  By contrast,
the GNU General Public License is intended to guarantee your freedom to
share and change all versions of a program--to make sure it remains free
software for all its users.  We, the Free Software Foundation, use the
GNU General Public License for most of our software; it applies also to
any other work released this way by its authors.  You can apply it to
your programs, too.

  When we speak of free software, we are referring to freedom, not
price.  Our General Public Licenses are designed to make sure that you
have the freedom to distribute copies of free software (and charge for
them if you wish), that you receive source code or can get it if you
want it, that you can change the software or use pieces of it in new
free programs, and that you know you can do these things.

  To protect your rights, we need to prevent others from denying you
these rights or asking you to surrender the rights.  Therefore, you have
certain responsibilities if you distribute copies of the software, or if
you modify it: responsibilities to respect the freedom of others.

  For example, if you distribute copies of such a program, whether
gratis or for a fee, you must pass on to the recipients the same
freedoms that you received.  You must make sure that they, too, receive
or can get the source code.  And you must show them these terms so they
know their rights.

  Developers that use the GNU GPL protect your rights with two steps:
(1) assert copyright on the software, and (2) offer you this License
giving you legal permission to copy, distribute and/or modify it.

  For the developers' and authors' protection, the GPL clearly explains
that there is no warranty for this free software.  For both users' and
authors' sake, the GPL requires that modified versions be marked as
changed, so that their problems will not be attributed erroneously to
authors of previous versions.

  Some devices are designed to deny users access to install or run
modified versions of the software inside them, although the manufacturer
can do so.  This is fundamentally incompatible with the aim of
protecting users' freedom to change the software.  The systematic
pattern of such abuse occurs in the area of products for individuals to
use, which is precisely where it is most unacceptable.  Therefore, we
have designed this version of the GPL to prohibit the practice for those
products.  If such problems arise substantially in other domains, we
stand ready to extend this provision to those domains in future versions
of the GPL, as needed to protect the freedom of users.

  Finally, every program is threatened constantly by software patents.
States should not allow patents to restrict development and use of
software on general-purpose computers, but in those that do, we wish to
avoid the special danger that patents applied to a free program could
make it effectively proprietary.  To prevent this, the GPL assures that
patents cannot be used to render the program non-free.

  The precise terms and conditions for copying, distribution and
modification follow.

                       TERMS AND CONDITIONS

  0. Definitions.

  "This License" refers to version 3 of the GNU General Public License.

  "Copyright" also means copyright-like laws that apply to other kinds of
works, such as semiconductor masks.

  "The Program" refers to any copyrightable work licensed under this
License.  Each licensee is addressed as "you".  "Licensees" and
"recipients" may be individuals or organizations.

  To "modify" a work means to copy from or adapt all or part of the work
in a fashion requiring copyright permission, other than the making of an
exact copy.  The resulting work is called a "modified version" of the
earlier work or a work "based on" the earlier work.

  A "covered work" means either the unmodified Program or a work based
on the Program.

  To "propagate" a work means to do anything with it that, without
permission, would make you directly or secondarily liable for
infringement under applicable copyright law, except executing it on a
computer or modifying a private copy.  Propagation includes copying,
distribution (with or without modification), making available to the
public, and in some countries other activities as well.

  To "convey" a work means any kind of propagation that enables other
parties to make or receive copies.  Mere interaction with a user through
a computer network, with no transfer of a copy, is not conveying.

  An interactive user interface displays "Appropriate Legal Notices"
to the extent that it includes a convenient and prominently visible
feature that (1) displays an appropriate copyright notice, and (2)
tells the user that there is no warranty for the work (except to the
extent that warranties are provided), that licensees may convey the
work under this License, and how to view a copy of this License.  If
the interface presents a list of user commands or options, such as a
menu, a prominent item in the list meets this criterion.

  1. Source Code.

  The "source code" for a work means the preferred form of the work
for making modifications to it.  "Object code" means any non-source
form of a work.

  A "Standard Interface" means an interface that either is an official
standard defined by a recognized standards body, or, in the case of
interfaces specified for a particular programming language, one that
is widely used among developers working in that language.

  The "System Libraries" of an executable work include anything, other
than the work as a whole, that (a) is included in the normal form of
packaging a Major Component, but which is not part of that Major
Component, and (b) serves only to enable use of the work with that
Major Component, or to implement a Standard Interface for which an
implementation is available to the public in source code form.  A
"Major Component", in this context, means a major essential component
(kernel, window system, and so on) of the specific operating system
(if any) on which the executable work runs, or a compiler used to
produce the work, or an object code interpreter used to run it.

  The "Corresponding Source" for a work in object code form means all
the source code needed to generate, install, and (for an executable
work) run the object code and to modify the work, including scripts to
control those activities.  However, it does not include the work's
System Libraries, or general-purpose tools or generally available free
programs which are used unmodified in performing those activities but
which are not part of the work.  For example, Corresponding Source
includes interface definition files associated with source files for
the work, and the source code for shared libraries and dynamically
linked subprograms that the work is specifically designed to require,
such as by intimate data communication or control flow between those
subprograms and other parts of the work.

  The Corresponding Source need not include anything that users
can regenerate automatically from other parts of the Corresponding
Source.

  The Corresponding Source for a work in source code form is that
same work.

  2. Basic Permissions.

  All rights granted under this License are granted for the term of
copyright on the Program, and are irrevocable provided the stated
conditions are met.  This License explicitly affirms your unlimited
permission to run the unmodified Program.  The output from running a
covered work is covered by this License only if the output, given its
content, constitutes a covered work.  This License acknowledges your
rights of fair use or other equivalent, as provided by copyright law.

  You may make, run and propagate covered works that you do not
convey, without conditions so long as your license otherwise remains
in force.  You may convey covered works to others for the sole purpose
of having them make modifications exclusively for you, or provide you
with facilities for running those works, provided that you comply with
the terms of this License in conveying all material for which you do
not control copyright.  Those thus making or running the covered works
for you must do so exclusively on your behalf, under your direction
and control, on terms that prohibit them from making any copies of
your copyrighted material outside their relationship with you.

  Conveying under any other circumstances is permitted solely under
the conditions stated below.  Sublicensing is not allowed; section 10
makes it unnecessary.

  3. Protecting Users' Legal Rights From Anti-Circumvention Law.

  No covered work shall be deemed part of an effective technological
measure under any applicable law fulfilling obligations under article
11 of the WIPO copyright treaty adopted on 20 December 1996, or
similar laws prohibiting or restricting circumvention of such
measures.

  When you convey a covered work, you waive any legal power to forbid
circumvention of technological measures to the extent such circumvention
is effected by exercising rights under this License with respect to
the covered work, and you disclaim any intention to limit operation or
modification of the work as a means of enforcing, against the work's
users, your or third parties' legal rights to forbid circumvention of
technological measures.

  4. Conveying Verbatim Copies.

  You may convey verbatim copies of the Program's source code as you
receive it, in any medium, provided that you conspicuously and
appropriately publish on each copy an appropriate copyright notice;
keep intact all notices stating that this License and any
non-permissive terms added in accord with section 7 apply to the code;
keep intact all notices of the absence of any warranty; and give all
recipients a copy of this License along with the Program.

  You may charge any price or no price for each copy that you convey,
and you may offer support or warranty protection for a fee.

  5. Conveying Modified Source Versions.

  You may convey a work based on the Program, or the modifications to
produce it from the Program, in the form of source code under the
terms of section 4, provided that you also meet all of these conditions:

    a) The work must carry prominent notices stating that you modified
    it, and giving a relevant date.

    b) The work must carry prominent notices stating that it is
    released under this License and any conditions added under section
    7.  This requirement modifies the requirement in section 4 to
    "keep intact all notices".

    c) You must license the entire work, as a whole, under this
    License to anyone who comes into possession of a copy.  This
    License will therefore apply, along with any applicable section 7
    additional terms, to the whole of the work, and all its parts,
    regardless of how they are packaged.  This License gives no
    permission to license the work in any other way, but it does not
    invalidate such permission if you have separately received it.

    d) If the work has interactive user interfaces, each must display
    Appropriate Legal Notices; however, if the Program has interactive
    interfaces that do not display Appropriate Legal Notices, your
    work need not make them do so.

  A compilation of a covered work with other separate and independent
works, which are not by their nature extensions of the covered work,
and which are not combined with it such as to form a larger program,
in or on a volume of a storage or distribution medium, is called an
"aggregate" if the compilation and its resulting copyright are not
used to limit the access or legal rights of the compilation's users
beyond what the individual works permit.  Inclusion of a covered work
in an aggregate does not cause this License to apply to the other
parts of the aggregate.

  6. Conveying Non-Source Forms.

  You may convey a covered work in object code form under the terms
of sections 4 and 5, provided that you also convey the
machine-readable Corresponding Source under the terms of this License,
in one of these ways:

    a) Convey the object code in, or embodied in, a physical product
    (including a physical distribution medium), accompanied by the
    Corresponding Source fixed on a durable physical medium
    customarily used for software interchange.

    b) Convey the object code in, or embodied in, a physical product
    (including a physical distribution medium), accompanied by a
    written offer, valid for at least three years and valid for as
    long as you offer spare parts or customer support for that product
    model, to give anyone who possesses the object code either (1) a
    copy of the Corresponding Source for all the software in the
    product that is covered by this License, on a durable physical
    medium customarily used for software interchange, for a price no
    more than your reasonable cost of physically performing this
    conveying of source, or (2) access to copy the
    Corresponding Source from a network server at no charge.

    c) Convey individual copies of the object code with a copy of the
    written offer to provide the Corresponding Source.  This
    alternative is allowed only occasionally and noncommercially, and
    only if you received the object code with such an offer, in accord
    with subsection 6b.

    d) Convey the object code by offering access from a designated
    place (gratis or for a charge), and offer equivalent access to the
    Corresponding Source in the same way through the same place at no
    further charge.  You need not require recipients to copy the
    Corresponding Source along with the object code.  If the place to
    copy the object code is a network server, the Corresponding Source
    may be on a different server (operated by you or a third party)
    that supports equivalent copying facilities, provided you maintain
    clear directions next to the object code saying where to find the
    Corresponding Source.  Regardless of what server hosts the
    Corresponding Source, you remain obligated to ensure that it is
    available for as long as needed to satisfy these requirements.

    e) Convey the object code using peer-to-peer transmission, provided
    you inform other peers where the object code and Corresponding
    Source of the work are being offered to the general public at no
    charge under subsection 6d.

  A separable portion of the object code, whose source code is excluded
from the Corresponding Source as a System Library, need not be
included in conveying the object code work.

  A "User Product" is either (1) a "consumer product", which means any
tangible personal property which is normally used for personal, family,
or household purposes, or (2) anything designed or sold for incorporation
into a dwelling.  In determining whether a product is a consumer product,
doubtful cases shall be resolved in favor of coverage.  For a particular
product received by a particular user, "normally used" refers to a
typical or common use of that class of product, regardless of the status
of the particular user or of the way in which the particular user
actually uses, or expects or is expected to use, the product.  A product
is a consumer product regardless of whether the product has substantial
commercial, industrial or non-consumer uses, unless such uses represent
the only significant mode of use of the product.

  "Installation Information" for a User Product means any methods,
procedures, authorization keys, or other information required to install
and execute modified versions of a covered work in that User Product from
a modified version of its Corresponding Source.  The information must
suffice to ensure that the continued functioning of the modified object
code is in no case prevented or interfered with solely because
modification has been made.

  If you convey an object code work under this section in, or with, or
specifically for use in, a User Product, and the conveying occurs as
part of a transaction in which the right of possession and use of the
User Product is transferred to the recipient in perpetuity or for a
fixed term (regardless of how the transaction is characterized), the
Corresponding Source conveyed under this section must be accompanied
by the Installation Information.  But this requirement does not apply
if neither you nor any third party retains the ability to install
modified object code on the User Product (for example, the work has
been installed in ROM).

  The requirement to provide Installation Information does not include a
requirement to continue to provide support service, warranty, or updates
for a work that has been modified or installed by the recipient, or for
the User Product in which it has been modified or installed.  Access to a
network may be denied when the modification itself materially and
adversely affects the operation of the network or violates the rules and
protocols for communication across the network.

  Corresponding Source conveyed, and Installation Information provided,
in accord with this section must be in a format that is publicly
documented (and with an implementation available to the public in
source code form), and must require no special password or key for
unpacking, reading or copying.

  7. Additional Terms.

  "Additional permissions" are terms that supplement the terms of this
License by making exceptions from one or more of its conditions.
Additional permissions that are applicable to the entire Program shall
be treated as though they were included in this License, to the extent
that they are valid under applicable law.  If additional permissions
apply only to part of the Program, that part may be used separately
under those permissions, but the entire Program remains governed by
this License without regard to the additional permissions.

  When you convey a copy of a covered work, you may at your option
remove any additional permissions from that copy, or from any part of
it.  (Additional permissions may be written to require their own
removal in certain cases when you modify the work.)  You may place
additional permissions on material, added by you to a covered work,
for which you have or can give appropriate copyright permission.

  Notwithstanding any other provision of this License, for material you
add to a covered work, you may (if authorized by the copyright holders of
that material) supplement the terms of this License with terms:

    a) Disclaiming warranty or limiting liability differently from the
    terms of sections 15 and 16 of this License; or

    b) Requiring preservation of specified reasonable legal notices or
    author attributions in that material or in the Appropriate Legal
    Notices displayed by works containing it; or

    c) Prohibiting misrepresentation of the origin of that material, or
    requiring that modified versions of such material be marked in
    reasonable ways as different from the original version; or

    d) Limiting the use for publicity purposes of names of licensors or
    authors of the material; or

    e) Declining to grant rights under trademark law for use of some
    trade names, trademarks, or service marks; or

    f) Requiring indemnification of licensors and authors of that
    material by anyone who conveys the material (or modified versions of
    it) with contractual assumptions of liability to the recipient, for
    any liability that these contractual assumptions directly impose on
    those licensors and authors.

  All other non-permissive additional terms are considered "further
restrictions" within the meaning of section 10.  If the Program as you
received it, or any part of it, contains a notice stating that it is
governed by this License along with a term that is a further
restriction, you may remove that term.  If a license document contains
a further restriction but permits relicensing or conveying under this
License, you may add to a covered work material governed by the terms
of that license document, provided that the further restriction does
not survive such relicensing or conveying.

  If you add terms to a covered work in accord with this section, you
must place, in the relevant source files, a statement of the
additional terms that apply to those files, or a notice indicating
where to find the applicable terms.

  Additional terms, permissive or non-permissive, may be stated in the
form of a separately written license, or stated as exceptions;
the above requirements apply either way.

  8. Termination.

  You may not propagate or modify a covered work except as expressly
provided under this License.  Any attempt otherwise to propagate or
modify it is void, and will automatically terminate your rights under
this License (including any patent licenses granted under the third
paragraph of section 11).

  However, if you cease all violation of this License, then your
license from a particular copyright holder is reinstated (a)
provisionally, unless and until the copyright holder explicitly and
finally terminates your license, and (b) permanently, if the copyright
holder fails to notify you of the violation by some reasonable means
prior to 60 days after the cessation.

  Moreover, your license from a particular copyright holder is
reinstated permanently if the copyright holder notifies you of the
violation by some reasonable means, this is the first time you have
received notice of violation of this License (for any work) from that
copyright holder, and you cure the violation prior to 30 days after
your receipt of the notice.

  Termination of your rights under this section does not terminate the
licenses of parties who have received copies or rights from you under
this License.  If your rights have been terminated and not permanently
reinstated, you do not qualify to receive new licenses for the same
material under section 10.

  9. Acceptance Not Required for Having Copies.

  You are not required to accept this License in order to receive or
run a copy of the Program.  Ancillary propagation of a covered work
occurring solely as a consequence of using peer-to-peer transmission
to receive a copy likewise does not require acceptance.  However,
nothing other than this License grants you permission to propagate or
modify any covered work.  These actions infringe copyright if you do
not accept this License.  Therefore, by modifying or propagating a
covered work, you indicate your acceptance of this License to do so.

  10. Automatic Licensing of Downstream Recipients.

  Each time you convey a covered work, the recipient automatically
receives a license from the original licensors, to run, modify and
propagate that work, subject to this License.  You are not responsible
for enforcing compliance by third parties with this License.

  An "entity transaction" is a transaction transferring control of an
organization, or substantially all assets of one, or subdividing an
organization, or merging organizations.  If propagation of a covered
work results from an entity transaction, each party to that
transaction who receives a copy of the work also receives whatever
licenses to the work the party's predecessor in interest had or could
give under the previous paragraph, plus a right to possession of the
Corresponding Source of the work from the predecessor in interest, if
the predecessor has it or can get it with reasonable efforts.

  You may not impose any further restrictions on the exercise of the
rights granted or affirmed under this License.  For example, you may
not impose a license fee, royalty, or other charge for exercise of
rights granted under this License, and you may not initiate litigation
(including a cross-claim or counterclaim in a lawsuit) alleging that
any patent claim is infringed by making, using, selling, offering for
sale, or importing the Program or any portion of it.

  11. Patents.

  A "contributor" is a copyright holder who authorizes use under this
License of the Program or a work on which the Program is based.  The
work thus licensed is called the contributor's "contributor version".

  A contributor's "essential patent claims" are all patent claims
owned or controlled by the contributor, whether already acquired or
hereafter acquired, that would be infringed by some manner, permitted
by this License, of making, using, or selling its contributor version,
but do not include claims that would be infringed only as a
consequence of further modification of the contributor version.  For
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
sue for patent infringement).  To "grant" such a patent license to a
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
license to downstream recipients.  "Knowingly relying" means you have
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

  A patent license is "discriminatory" if it does not include within
the scope of its coverage, prohibits the exercise of, or is
conditioned on the non-exercise of one or more of the rights that are
specifically granted under this License.  You may not convey a covered
work if you are a party to an arrangement with a third party that is
in the business of distributing software, under which you make payment
to the third party based on the extent of your activity of conveying
the work, and under which the third party grants, to any of the
parties who would receive the covered work from you, a discriminatory
patent license (a) in connection with copies of the covered work
conveyed by you (or copies made from those copies), or (b) primarily
for and in connection with specific products or compilations that
contain the covered work, unless you entered into that arrangement,
or that patent license was granted, prior to 28 March 2007.

  Nothing in this License shall be construed as excluding or limiting
any implied license or other defenses to infringement that may
otherwise be available to you under applicable patent law.

  12. No Surrender of Others' Freedom.

  If conditions are imposed on you (whether by court order, agreement or
otherwise) that contradict the conditions of this License, they do not
excuse you from the conditions of this License.  If you cannot convey a
covered work so as to satisfy simultaneously your obligations under this
License and any other pertinent obligations, then as a consequence you may
not convey it at all.  For example, if you agree to terms that obligate you
to collect a royalty for further conveying from those to whom you convey
the Program, the only way you could satisfy both those terms and this
License would be to refrain entirely from conveying the Program.

  13. Use with the GNU Affero General Public License.

  Notwithstanding any other provision of this License, you have
permission to link or combine any covered work with a work licensed
under version 3 of the GNU Affero General Public License into a single
combined work, and to convey the resulting work.  The terms of this
License will continue to apply to the part which is the covered work,
but the special requirements of the GNU Affero General Public License,
section 13, concerning interaction through a network will apply to the
combination as such.

  14. Revised Versions of this License.

  The Free Software Foundation may publish revised and/or new versions of
the GNU General Public License from time to time.  Such new versions will
be similar in spirit to the present version, but may differ in detail to
address new problems or concerns.

  Each version is given a distinguishing version number.  If the
Program specifies that a certain numbered version of the GNU General
Public License "or any later version" applies to it, you have the
option of following the terms and conditions either of that numbered
version or of any later version published by the Free Software
Foundation.  If the Program does not specify a version number of the
GNU General Public License, you may choose any version ever published
by the Free Software Foundation.

  If the Program specifies that a proxy can decide which future
versions of the GNU General Public License can be used, that proxy's
public statement of acceptance of a version permanently authorizes you
to choose that version for the Program.

  Later license versions may give you additional or different
permissions.  However, no additional obligations are imposed on any
author or copyright holder as a result of your choosing to follow a
later version.

  15. Disclaimer of Warranty.

  THERE IS NO WARRANTY FOR THE PROGRAM, TO THE EXTENT PERMITTED BY
APPLICABLE LAW.  EXCEPT WHEN OTHERWISE STATED IN WRITING THE COPYRIGHT
HOLDERS AND/OR OTHER PARTIES PROVIDE THE PROGRAM "AS IS" WITHOUT WARRANTY
OF ANY KIND, EITHER EXPRESSED OR IMPLIED, INCLUDING, BUT NOT LIMITED TO,
THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
PURPOSE.  THE ENTIRE RISK AS TO THE QUALITY AND PERFORMANCE OF THE PROGRAM
IS WITH YOU.  SHOULD THE PROGRAM PROVE DEFECTIVE, YOU ASSUME THE COST OF
ALL NECESSARY SERVICING, REPAIR OR CORRECTION.

  16. Limitation of Liability.

  IN NO EVENT UNLESS REQUIRED BY APPLICABLE LAW OR AGREED TO IN WRITING
WILL ANY COPYRIGHT HOLDER, OR ANY OTHER PARTY WHO MODIFIES AND/OR CONVEYS
THE PROGRAM AS PERMITTED ABOVE, BE LIABLE TO YOU FOR DAMAGES, INCLUDING ANY
GENERAL, SPECIAL, INCIDENTAL OR CONSEQUENTIAL DAMAGES ARISING OUT OF THE
USE OR INABILITY TO USE THE PROGRAM (INCLUDING BUT NOT LIMITED TO LOSS OF
DATA OR DATA BEING RENDERED INACCURATE OR LOSSES SUSTAINED BY YOU OR THIRD
PARTIES OR A FAILURE OF THE PROGRAM TO OPERATE WITH ANY OTHER PROGRAMS),
EVEN IF SUCH HOLDER OR OTHER PARTY HAS BEEN ADVISED OF THE POSSIBILITY OF
SUCH DAMAGES.

  17. Interpretation of Sections 15 and 16.

  If the disclaimer of warranty and limitation of liability provided
above cannot be given local legal effect according to their terms,
reviewing courts shall apply local law that most closely approximates
an absolute waiver of all civil liability in connection with the
Program, unless a warranty or assumption of liability accompanies a
copy of the Program in return for a fee.

                     END OF TERMS AND CONDITIONS

            How to Apply These Terms to Your New Programs

  If you develop a new program, and you want it to be of the greatest
possible use to the public, the best way to achieve this is to make it
free software which everyone can redistribute and change under these terms.

  To do so, attach the following notices to the program.  It is safest
to attach them to the start of each source file to most effectively
state the exclusion of warranty; and each file should have at least
the "copyright" line and a pointer to where the full notice is found.

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

Also add information on how to contact you by electronic and paper mail.

  If the program does terminal interaction, make it output a short
notice like this when it starts in an interactive mode:

    <program>  Copyright (C) <year>  <name of author>
    This program comes with ABSOLUTELY NO WARRANTY; for details type `show w'.
    This is free software, and you are welcome to redistribute it
    under certain conditions; type `show c' for details.

The hypothetical commands `show w' and `show c' should show the appropriate
parts of the General Public License.  Of course, your program's commands
might be different; for a GUI interface, you would use an "about box".

  You should also get your employer (if you work as a programmer) or school,
if any, to sign a "copyright disclaimer" for the program, if necessary.
For more information on this, and how to apply and follow the GNU GPL, see
<https://www.gnu.org/licenses/>.

  The GNU General Public License does not permit incorporating your program
into proprietary programs.  If your program is a subroutine library, you
may consider it more useful to permit linking proprietary applications with
the library.  If this is what you want to do, use the GNU Lesser General
Public License instead of this License.  But first, please read
<https://www.gnu.org/licenses/why-not-lgpl.html>.
```
*/
