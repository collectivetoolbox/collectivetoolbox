//! Implementation of Colin L. `McMaster`'s 1979 Online Adaptive Huffman
//! Coding format (`compact` / `uncompact`, `.C` file format).
//!
//! Specification reference: `data/docs/compact.md`

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;
use std::io::{Read, Write};

/// Magic header bytes for `compact` (`.C` format) (`0xFF`, `0x1F`).
pub const COMPACT_MAGIC: [u8; 2] = [0xFF, 0x1F];

const SYMBOL_EF: u16 = 256;
const SYMBOL_NC: u16 = 257;

/// 16-bit word bit reader matching 4.1cBSD `union cio` I/O.
struct WordReader<R: Read> {
    reader: R,
    bitbuf: u32,
    valid: u32,
}

impl<R: Read> WordReader<R> {
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

    fn read_bit(&mut self) -> Result<u32> {
        if self.valid == 0 {
            let hib = self.read_byte()?;
            let lob = match self.read_byte() {
                Ok(b) => b,
                Err(_) => 0, // Trailing byte padding
            };
            self.bitbuf = (u32::from(hib) << 8) | u32::from(lob);
            self.valid = 16;
        }
        let shift = self.valid.saturating_sub(1);
        let bit = (self.bitbuf >> shift) & 1;
        self.valid = shift;
        Ok(bit)
    }

    fn read_bits(&mut self, count: u32) -> Result<u8> {
        let mut value = 0u8;
        for _ in 0..count {
            let bit = self.read_bit()?;
            let bit_u8 =
                u8::try_from(bit).context("Bit conversion overflow")?;
            value = (value << 1) | bit_u8;
        }
        Ok(value)
    }
}

/// 16-bit word bit writer matching 4.1cBSD `union cio` I/O.
struct WordWriter<W: Write> {
    writer: W,
    bitbuf: u32,
    valid: u32,
    bytes_written: u64,
}

impl<W: Write> WordWriter<W> {
    fn new(writer: W) -> Self {
        Self {
            writer,
            bitbuf: 0,
            valid: 0,
            bytes_written: 0,
        }
    }

    fn write_bit(&mut self, bit: u32) -> Result<()> {
        self.bitbuf = (self.bitbuf << 1) | (bit & 1);
        self.valid = self.valid.saturating_add(1);
        if self.valid == 16 {
            let hib = u8::try_from((self.bitbuf >> 8) & 0xFF)
                .context("Bit buffer conversion failed")?;
            let lob = u8::try_from(self.bitbuf & 0xFF)
                .context("Bit buffer conversion failed")?;
            self.writer
                .write_all(&[hib, lob])
                .context("Failed to write bitstream word")?;
            self.bytes_written = self.bytes_written.saturating_add(2);
            self.bitbuf = 0;
            self.valid = 0;
        }
        Ok(())
    }

    fn write_bits(&mut self, value: u32, count: u32) -> Result<()> {
        let mut i = count;
        while i > 0 {
            i = i.saturating_sub(1);
            let bit = (value >> i) & 1;
            self.write_bit(bit)?;
        }
        Ok(())
    }

    fn flush_padding(&mut self) -> Result<()> {
        if self.valid > 0 {
            let shift = 16u32.saturating_sub(self.valid);
            let padded = self.bitbuf << shift;
            let hib = u8::try_from((padded >> 8) & 0xFF)
                .context("Bit buffer padding conversion failed")?;
            let lob = u8::try_from(padded & 0xFF)
                .context("Bit buffer padding conversion failed")?;
            self.writer
                .write_all(&[hib, lob])
                .context("Failed to write final bitstream padding word")?;
            self.bytes_written = self.bytes_written.saturating_add(2);
            self.bitbuf = 0;
            self.valid = 0;
        }
        self.writer
            .flush()
            .context("Failed to flush underlying writer")?;
        Ok(())
    }
}

/// A child branch in `McMaster`'s adaptive Huffman tree (Internal node vs Leaf symbol).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Child {
    Internal(usize),
    Leaf(u16),
}

/// Instantaneous LLEAF/RLEAF leaf parent lookup.
#[derive(Debug, Clone, Copy)]
struct LeafInfo {
    fp: Option<usize>,
    dir: usize,
}

/// A node in `McMaster`'s adaptive Huffman tree dict array.
#[derive(Debug, Clone, Copy)]
struct DictNode {
    fp: Option<usize>,
    sp: [Child; 2],
    count: [u32; 2],
}

/// `McMaster`'s 1979 4.1cBSD Online Adaptive Huffman Coding Tree.
struct CompactTree {
    dict: Vec<DictNode>,
    leaf_info: [LeafInfo; 258],
    bottom_idx: usize,
}

impl CompactTree {
    fn new(first_byte: u8) -> Self {
        let first_byte_u16 = u16::from(first_byte);
        let first_byte_idx = usize::from(first_byte_u16);

        let mut dict = vec![
            DictNode {
                fp: None,
                sp: [Child::Leaf(0), Child::Leaf(0)],
                count: [0, 0],
            };
            515
        ];

        let mut leaf_info = [LeafInfo { fp: None, dir: 0 }; 258];

        // Root dict[0]: left = dict[1] (sp[0]=Child::Internal(1)), right = C0 leaf (sp[1]=Child::Leaf(C0))
        if let Some(d0) = dict.get_mut(0) {
            d0.count = [2, 1];
            d0.sp = [Child::Internal(1), Child::Leaf(first_byte_u16)];
        }

        // Dict[1] (bottom): left = NC (257), right = EF (256)
        if let Some(d1) = dict.get_mut(1) {
            d1.fp = Some(0);
            d1.count = [1, 1];
            d1.sp = [Child::Leaf(SYMBOL_NC), Child::Leaf(SYMBOL_EF)];
        }

        if let Some(info) = leaf_info.get_mut(first_byte_idx) {
            *info = LeafInfo {
                fp: Some(0),
                dir: 1,
            };
        }
        if let Some(info) = leaf_info.get_mut(usize::from(SYMBOL_NC)) {
            *info = LeafInfo {
                fp: Some(1),
                dir: 0,
            };
        }
        if let Some(info) = leaf_info.get_mut(usize::from(SYMBOL_EF)) {
            *info = LeafInfo {
                fp: Some(1),
                dir: 1,
            };
        }

        Self {
            dict,
            leaf_info,
            bottom_idx: 1,
        }
    }

    fn exch(&mut self, p1: usize, b1: usize, p2: usize, b2: usize) {
        if p1 == p2 && b1 == b2 {
            return;
        }

        let child1 = match self.dict.get(p1).and_then(|n| n.sp.get(b1)) {
            Some(&c) => c,
            None => return,
        };
        let count1 = match self.dict.get(p1).and_then(|n| n.count.get(b1)) {
            Some(&cnt) => cnt,
            None => return,
        };
        let child2 = match self.dict.get(p2).and_then(|n| n.sp.get(b2)) {
            Some(&c) => c,
            None => return,
        };
        let count2 = match self.dict.get(p2).and_then(|n| n.count.get(b2)) {
            Some(&cnt) => cnt,
            None => return,
        };

        if let Some(n1) = self.dict.get_mut(p1) {
            if let Some(slot) = n1.sp.get_mut(b1) {
                *slot = child2;
            }
            if let Some(slot) = n1.count.get_mut(b1) {
                *slot = count2;
            }
        }

        if let Some(n2) = self.dict.get_mut(p2) {
            if let Some(slot) = n2.sp.get_mut(b2) {
                *slot = child1;
            }
            if let Some(slot) = n2.count.get_mut(b2) {
                *slot = count1;
            }
        }

        match child2 {
            Child::Internal(c2) => {
                if let Some(n) = self.dict.get_mut(c2) {
                    n.fp = Some(p1);
                }
            }
            Child::Leaf(sym2) => {
                let idx = usize::from(sym2);
                if let Some(info) = self.leaf_info.get_mut(idx) {
                    info.fp = Some(p1);
                    info.dir = b1;
                }
            }
        }

        match child1 {
            Child::Internal(c1) => {
                if let Some(n) = self.dict.get_mut(c1) {
                    n.fp = Some(p2);
                }
            }
            Child::Leaf(sym1) => {
                let idx = usize::from(sym1);
                if let Some(info) = self.leaf_info.get_mut(idx) {
                    info.fp = Some(p2);
                    info.dir = b2;
                }
            }
        }
    }

    fn is_in_subtree(
        &self,
        parent: usize,
        branch_dir: usize,
        mut sub: usize,
    ) -> bool {
        if parent == sub {
            return false;
        }
        let mut steps = 0usize;
        while let Some(fp) = self.dict.get(sub).and_then(|n| n.fp) {
            if fp == parent {
                let is_branch = match self
                    .dict
                    .get(fp)
                    .and_then(|n| n.sp.get(branch_dir))
                {
                    Some(&Child::Internal(child_idx)) => child_idx == sub,
                    _ => false,
                };
                return is_branch;
            }
            sub = fp;
            steps = steps.saturating_add(1);
            if steps > 515 {
                break;
            }
        }
        false
    }

    fn uptree(&mut self, symbol: u16) {
        let sym_usize = usize::from(symbol);
        let info = match self.leaf_info.get(sym_usize) {
            Some(i) => *i,
            None => return,
        };
        let mut curr_p = info.fp;
        let mut curr_b = info.dir;

        while let Some(p_idx) = curr_p {
            let w = match self.dict.get(p_idx).and_then(|n| n.count.get(curr_b))
            {
                Some(&cnt) => cnt,
                None => break,
            };

            let mut target_p = None;
            let mut target_b = None;

            let curr_linear = p_idx.saturating_mul(2).saturating_add(curr_b);

            for cand_linear in 0..curr_linear {
                let cand_p = cand_linear >> 1;
                let cand_b = cand_linear & 1;

                let cand_count = match self
                    .dict
                    .get(cand_p)
                    .and_then(|n| n.count.get(cand_b))
                {
                    Some(&cnt) => cnt,
                    None => continue,
                };

                if cand_count == w
                    && !self.is_in_subtree(p_idx, curr_b, cand_p)
                    && !self.is_in_subtree(cand_p, cand_b, p_idx)
                {
                    target_p = Some(cand_p);
                    target_b = Some(cand_b);
                    break;
                }
            }

            if let (Some(tp), Some(tb)) = (target_p, target_b) {
                if tp != p_idx || tb != curr_b {
                    self.exch(p_idx, curr_b, tp, tb);
                    curr_p = Some(tp);
                    curr_b = tb;
                }
            }

            // Reason for fallback: if tree node update loop has not moved to parent (curr_p is None), p_idx serves as active node index.
            let active_p = curr_p.unwrap_or(p_idx);
            if let Some(n) = self.dict.get_mut(active_p) {
                if let Some(slot) = n.count.get_mut(curr_b) {
                    *slot = slot.saturating_add(1);
                }
            }

            let old_p_idx = active_p;
            curr_p = match self.dict.get(active_p) {
                Some(n) => n.fp,
                None => None,
            };
            if let Some(parent_idx) = curr_p {
                let parent_right = match self.dict.get(parent_idx) {
                    Some(n) => n.sp[1],
                    None => Child::Leaf(0),
                };
                curr_b =
                    usize::from(parent_right == Child::Internal(old_p_idx));
            }
        }
    }

    fn insert(&mut self, symbol: u8) -> Result<()> {
        let pp = self.bottom_idx;
        let old_right = match self.dict.get(pp).and_then(|n| n.sp.get(1)) {
            Some(&c) => c,
            None => bail!("Bottom index node out of bounds"),
        };
        let old_count = match self.dict.get(pp).and_then(|n| n.count.get(1)) {
            Some(&cnt) => cnt,
            None => bail!("Bottom index count out of bounds"),
        };

        self.bottom_idx = self.bottom_idx.saturating_add(1);
        let new_bottom_idx = self.bottom_idx;

        if let Some(n) = self.dict.get_mut(pp) {
            if let Some(slot) = n.sp.get_mut(1) {
                *slot = Child::Internal(new_bottom_idx);
            }
        }

        let sym_u16 = u16::from(symbol);
        if let Some(nb) = self.dict.get_mut(new_bottom_idx) {
            nb.fp = Some(pp);
            nb.sp = [old_right, Child::Leaf(sym_u16)];
            nb.count = [old_count, 0];
        }

        match old_right {
            Child::Internal(c) => {
                if let Some(n) = self.dict.get_mut(c) {
                    n.fp = Some(new_bottom_idx);
                }
            }
            Child::Leaf(sym) => {
                let idx = usize::from(sym);
                if let Some(info) = self.leaf_info.get_mut(idx) {
                    info.fp = Some(new_bottom_idx);
                    info.dir = 0;
                }
            }
        }

        let sym_usize = usize::from(symbol);
        if let Some(info) = self.leaf_info.get_mut(sym_usize) {
            info.fp = Some(new_bottom_idx);
            info.dir = 1;
        }

        Ok(())
    }

    fn get_code_path(&self, symbol: u16) -> Result<Vec<u32>> {
        let sym_usize = usize::from(symbol);
        let info = self
            .leaf_info
            .get(sym_usize)
            .copied()
            .context("Invalid symbol index")?;
        let parent_idx = info.fp.context("Symbol not present in tree")?;

        let mut path = Vec::new();
        let mut curr_dir = info.dir;
        let mut curr_p = parent_idx;

        while path.len() <= 515 {
            let bit =
                u32::try_from(curr_dir).context("Invalid branch direction")?;
            path.push(bit);

            let node = match self.dict.get(curr_p) {
                Some(n) => n,
                None => bail!("Corrupted node index {curr_p}"),
            };

            let fp = match node.fp {
                Some(p) => p,
                None => break,
            };

            let fp_node = match self.dict.get(fp) {
                Some(n) => n,
                None => bail!("Corrupted parent node index {fp}"),
            };

            curr_dir = usize::from(fp_node.sp[1] == Child::Internal(curr_p));
            curr_p = fp;
        }

        if path.len() > 515 {
            bail!("Cycle detected in Huffman tree code path");
        }

        path.reverse();
        Ok(path)
    }

    fn encode_symbol<W: Write>(
        &mut self,
        writer: &mut WordWriter<W>,
        symbol: u16,
    ) -> Result<()> {
        let path = self.get_code_path(symbol)?;
        for bit in path {
            writer.write_bit(bit)?;
        }
        Ok(())
    }
}

/// Compresses raw binary data from `reader` into `writer` using `compact` format (`.C`).
pub fn compress_compact_stream<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
) -> Result<u64> {
    let mut initial_byte_buf = [0u8; 1];
    let n = reader
        .read(&mut initial_byte_buf)
        .context("Failed to read initial byte for compact compression")?;
    if n == 0 {
        bail!(
            "Input stream is empty; compact format requires at least 1 raw seed byte"
        );
    }

    let first_byte = initial_byte_buf[0];
    let mut word_writer = WordWriter::new(writer);

    // Write Magic Header 0xFF 0x1F and First Literal Byte C0
    word_writer
        .writer
        .write_all(&COMPACT_MAGIC)
        .context("Failed to write compact magic header")?;
    word_writer.bytes_written = word_writer.bytes_written.saturating_add(2);

    word_writer
        .writer
        .write_all(&[first_byte])
        .context("Failed to write first raw seed byte")?;
    word_writer.bytes_written = word_writer.bytes_written.saturating_add(1);

    let mut tree = CompactTree::new(first_byte);

    let mut buf = [0u8; 4096];
    loop {
        let bytes_read = reader
            .read(&mut buf)
            .context("Failed to read block from input stream")?;
        if bytes_read == 0 {
            break;
        }

        // Reason for fallback: bytes_read is bounded by 4096-byte buffer capacity, so ..bytes_read is in bounds; fallback to empty slice safely handles out-of-bounds.
        let slice = buf.get(..bytes_read).unwrap_or(&[]);
        for &byte in slice {
            let byte_u16 = u16::from(byte);
            let byte_usize = usize::from(byte);

            if tree
                .leaf_info
                .get(byte_usize)
                .and_then(|info| info.fp)
                .is_some()
            {
                // Symbol already seen
                tree.encode_symbol(&mut word_writer, byte_u16)?;
                tree.uptree(byte_u16);
            } else {
                // Unseen symbol -> send NC escape, uptree(NC), insert(c), write_bits(c, 8), uptree(ch)
                tree.encode_symbol(&mut word_writer, SYMBOL_NC)?;
                tree.uptree(SYMBOL_NC);
                tree.insert(byte)?;
                word_writer.write_bits(u32::from(byte), 8)?;
                tree.uptree(byte_u16);
            }
        }
    }

    // Write End-of-File marker
    tree.encode_symbol(&mut word_writer, SYMBOL_EF)?;
    word_writer.flush_padding()?;

    Ok(word_writer.bytes_written)
}

/// Decompresses `compact` compressed binary data from `reader` into `writer`.
pub fn decompress_compact_stream<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
) -> Result<u64> {
    let mut word_reader = WordReader::new(reader);

    // Read and verify magic header (0xFF 0x1F)
    let magic_b0 = word_reader
        .read_byte()
        .context("Failed to read compact magic byte 0")?;
    let magic_b1 = word_reader
        .read_byte()
        .context("Failed to read compact magic byte 1")?;
    if [magic_b0, magic_b1] != COMPACT_MAGIC {
        bail!("Invalid magic header for compact stream");
    }

    let first_byte = word_reader
        .read_byte()
        .context("Failed to read first raw seed byte")?;

    writer
        .write_all(&[first_byte])
        .context("Failed to write first decompressed byte")?;
    let mut bytes_written: u64 = 1;

    let mut tree = CompactTree::new(first_byte);

    loop {
        let mut curr_node_idx = 0usize;
        let mut steps = 0usize;
        let symbol: u16 = loop {
            steps = steps.saturating_add(1);
            if steps > 515 {
                bail!(
                    "Cycle detected during compact decompression tree traversal"
                );
            }
            let bit = word_reader.read_bit()?;
            let bit_usize =
                usize::try_from(bit).context("Invalid bit index")?;

            let node = match tree.dict.get(curr_node_idx) {
                Some(n) => n,
                None => bail!("Corrupted node index {curr_node_idx}"),
            };

            let next_sp = match node.sp.get(bit_usize) {
                Some(&child) => child,
                None => bail!("Invalid branch index {bit_usize}"),
            };

            match next_sp {
                Child::Internal(child_idx) => curr_node_idx = child_idx,
                Child::Leaf(sym) => break sym,
            }
        };

        if symbol == SYMBOL_EF {
            break;
        } else if symbol == SYMBOL_NC {
            tree.uptree(SYMBOL_NC);
            let raw_byte = word_reader.read_bits(8)?;
            tree.insert(raw_byte)?;
            tree.uptree(u16::from(raw_byte));
            writer
                .write_all(&[raw_byte])
                .context("Failed to write decompressed byte")?;
            bytes_written = bytes_written.saturating_add(1);
        } else {
            let byte =
                u8::try_from(symbol).context("Invalid data symbol value")?;
            writer
                .write_all(&[byte])
                .context("Failed to write decompressed byte")?;
            bytes_written = bytes_written.saturating_add(1);
            let sym_u16 = u16::from(byte);
            tree.uptree(sym_u16);
        }
    }

    Ok(bytes_written)
}

#[cfg(test)]
#[expect(
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
    fn test_compact_roundtrip_simple() {
        let input = b"Hello, world! This is a test for compact adaptive Huffman coding.";
        let mut compressed = Vec::new();
        let mut reader = &input[..];
        compress_compact_stream(&mut reader, &mut compressed)
            .expect("Compression failed");

        assert!(
            compressed.len() >= 3,
            "Compressed stream must have at least header"
        );
        assert_eq!(compressed.get(0..2), Some(&COMPACT_MAGIC[..]));
        assert_eq!(compressed.get(2), Some(&b'H'));

        let mut decompressed = Vec::new();
        let mut comp_reader = &compressed[..];
        decompress_compact_stream(&mut comp_reader, &mut decompressed)
            .expect("Decompression failed");

        assert_eq!(decompressed, input);
    }

    #[crate::ctb_test]
    fn test_compact_invalid_magic() {
        let bad_input = [0x00, 0x00, 0x41];
        let mut out = Vec::new();
        let mut reader = &bad_input[..];
        let res = decompress_compact_stream(&mut reader, &mut out);
        res.unwrap_err();
    }

    #[crate::ctb_test]
    fn test_compact_empty_input() {
        let empty: &[u8] = &[];
        let mut out = Vec::new();
        let mut reader = empty;
        let res = compress_compact_stream(&mut reader, &mut out);
        res.unwrap_err();
    }
}
