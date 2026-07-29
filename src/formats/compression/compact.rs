//! Implementation of Colin L. McMaster's 1979 Online Adaptive Huffman
//! Coding format (`compact` / `uncompact`, `.C` file format).
//!
//! Specification reference: `data/docs/compact.md`

#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;
use std::io::{Read, Write};

/// Magic header bytes for `compact` (`.C` format) (`0xFF`, `0x1F`).
pub const COMPACT_MAGIC: [u8; 2] = [0xFF, 0x1F];

const SYMBOL_EF: u16 = 256;
const SYMBOL_NC: u16 = 257;

/// Helper struct for MSB-to-LSB Bit Reader (8-bit bytes).
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

    fn read_bit(&mut self) -> Result<u32> {
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

    fn read_bits(&mut self, count: u32) -> Result<u8> {
        let mut value = 0u8;
        for _ in 0..count {
            let bit = self.read_bit()?;
            let bit_u8 = u8::try_from(bit).context("Bit conversion overflow")?;
            value = (value << 1) | bit_u8;
        }
        Ok(value)
    }
}

/// Helper struct for MSB-to-LSB Bit Writer (8-bit bytes).
struct BitWriter<W: Write> {
    writer: W,
    bitbuf: u32,
    valid: u32,
    bytes_written: u64,
}

impl<W: Write> BitWriter<W> {
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
        if self.valid == 8 {
            let byte = u8::try_from(self.bitbuf & 0xFF).context("Bit buffer conversion failed")?;
            self.writer
                .write_all(&[byte])
                .context("Failed to write bitstream byte")?;
            self.bytes_written = self.bytes_written.saturating_add(1);
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
            let shift = 8u32.saturating_sub(self.valid);
            let byte = u8::try_from((self.bitbuf << shift) & 0xFF)
                .context("Bit buffer padding conversion failed")?;
            self.writer
                .write_all(&[byte])
                .context("Failed to write final bitstream padding byte")?;
            self.bytes_written = self.bytes_written.saturating_add(1);
            self.bitbuf = 0;
            self.valid = 0;
        }
        self.writer
            .flush()
            .context("Failed to flush underlying writer")?;
        Ok(())
    }
}

/// A node in the McMaster adaptive Huffman tree.
#[derive(Debug, Clone, Copy)]
struct TreeNode {
    rank: i32,
    weight: u32,
    symbol: Option<u16>,
    parent: Option<usize>,
    left: Option<usize>,
    right: Option<usize>,
}

/// McMaster's Online Adaptive Huffman Coding Tree.
struct CompactTree {
    nodes: Vec<TreeNode>,
    symbol_leaf: [Option<usize>; 258],
    root_idx: usize,
    min_rank: i32,
}

impl CompactTree {
    fn new(first_byte: u8) -> Self {
        let first_byte_u16 = u16::from(first_byte);
        let first_byte_idx = usize::from(first_byte);

        let mut symbol_leaf = [None; 258];

        let root = TreeNode {
            rank: 4,
            weight: 2,
            symbol: None,
            parent: None,
            left: Some(1),
            right: Some(2),
        };
        let dict1 = TreeNode {
            rank: 3,
            weight: 2,
            symbol: None,
            parent: Some(0),
            left: Some(4),
            right: Some(3),
        };
        let leaf_c0 = TreeNode {
            rank: 2,
            weight: 1,
            symbol: Some(first_byte_u16),
            parent: Some(0),
            left: None,
            right: None,
        };
        let leaf_ef = TreeNode {
            rank: 1,
            weight: 1,
            symbol: Some(SYMBOL_EF),
            parent: Some(1),
            left: None,
            right: None,
        };
        let leaf_nc = TreeNode {
            rank: 0,
            weight: 1,
            symbol: Some(SYMBOL_NC),
            parent: Some(1),
            left: None,
            right: None,
        };

        if let Some(slot) = symbol_leaf.get_mut(first_byte_idx) {
            *slot = Some(2);
        }
        if let Some(slot) = symbol_leaf.get_mut(usize::from(SYMBOL_EF)) {
            *slot = Some(3);
        }
        if let Some(slot) = symbol_leaf.get_mut(usize::from(SYMBOL_NC)) {
            *slot = Some(4);
        }

        let nodes = vec![root, dict1, leaf_c0, leaf_ef, leaf_nc];

        Self {
            nodes,
            symbol_leaf,
            root_idx: 0,
            min_rank: 0,
        }
    }

    fn swap_nodes(&mut self, n1_idx: usize, n2_idx: usize) {
        if n1_idx == n2_idx {
            return;
        }

        let p1_opt = self.nodes.get(n1_idx).and_then(|n| n.parent);
        let p2_opt = self.nodes.get(n2_idx).and_then(|n| n.parent);

        if p1_opt == Some(n2_idx) || p2_opt == Some(n1_idx) {
            return;
        }

        let (Some(p1), Some(p2)) = (p1_opt, p2_opt) else {
            return;
        };

        // Update left/right pointers in parent p1
        if let Some(p1_node) = self.nodes.get_mut(p1) {
            if p1_node.left == Some(n1_idx) {
                p1_node.left = Some(n2_idx);
            } else if p1_node.right == Some(n1_idx) {
                p1_node.right = Some(n2_idx);
            }
        }

        // Update left/right pointers in parent p2
        if let Some(p2_node) = self.nodes.get_mut(p2) {
            if p2_node.left == Some(n2_idx) {
                p2_node.left = Some(n1_idx);
            } else if p2_node.right == Some(n2_idx) {
                p2_node.right = Some(n1_idx);
            }
        }

        // Swap parent pointers
        if let Some(n1_node) = self.nodes.get_mut(n1_idx) {
            n1_node.parent = Some(p2);
        }
        if let Some(n2_node) = self.nodes.get_mut(n2_idx) {
            n2_node.parent = Some(p1);
        }

        // Swap ranks
        let r1 = self.nodes.get(n1_idx).map(|n| n.rank).unwrap_or(0);
        let r2 = self.nodes.get(n2_idx).map(|n| n.rank).unwrap_or(0);

        if let Some(n1_node) = self.nodes.get_mut(n1_idx) {
            n1_node.rank = r2;
        }
        if let Some(n2_node) = self.nodes.get_mut(n2_idx) {
            n2_node.rank = r1;
        }
    }

    fn uptree(&mut self, start_idx: usize) {
        let mut curr_idx = start_idx;
        while curr_idx != self.root_idx {
            let curr_weight = match self.nodes.get(curr_idx) {
                Some(n) => n.weight,
                None => break,
            };
            let curr_rank = match self.nodes.get(curr_idx) {
                Some(n) => n.rank,
                None => break,
            };
            let curr_parent = match self.nodes.get(curr_idx) {
                Some(n) => n.parent,
                None => break,
            };

            // Find highest rank node with matching weight
            let mut max_rank = -1;
            let mut max_idx = None;

            for (idx, node) in self.nodes.iter().enumerate() {
                if node.weight == curr_weight && node.rank > max_rank {
                    max_rank = node.rank;
                    max_idx = Some(idx);
                }
            }

            if let Some(highest_idx) = max_idx {
                if max_rank > curr_rank && Some(highest_idx) != curr_parent {
                    self.swap_nodes(curr_idx, highest_idx);
                }
            }

            if let Some(node) = self.nodes.get_mut(curr_idx) {
                node.weight = node.weight.saturating_add(1);
            }

            let next_parent = match self.nodes.get(curr_idx) {
                Some(n) => n.parent,
                None => None,
            };
            match next_parent {
                Some(p) => curr_idx = p,
                None => break,
            }
        }

        if let Some(root_node) = self.nodes.get_mut(self.root_idx) {
            root_node.weight = root_node.weight.saturating_add(1);
        }
    }

    fn insert(&mut self, symbol: u8) -> Result<()> {
        let nc_sym_idx = usize::from(SYMBOL_NC);
        let nc_node_idx = match self.symbol_leaf.get(nc_sym_idx) {
            Some(&Some(idx)) => idx,
            _ => bail!("NC leaf symbol not present in tree"),
        };

        let nc_weight = match self.nodes.get(nc_node_idx) {
            Some(n) => n.weight,
            None => bail!("Invalid NC node index"),
        };

        if let Some(nc_node) = self.nodes.get_mut(nc_node_idx) {
            nc_node.symbol = None;
        }

        let new_nc_rank = self.min_rank.saturating_sub(1);
        let new_sym_rank = self.min_rank.saturating_sub(2);
        self.min_rank = self.min_rank.saturating_sub(2);

        let new_nc_idx = self.nodes.len();
        let new_nc_node = TreeNode {
            rank: new_nc_rank,
            weight: nc_weight,
            symbol: Some(SYMBOL_NC),
            parent: Some(nc_node_idx),
            left: None,
            right: None,
        };
        self.nodes.push(new_nc_node);

        let sym_u16 = u16::from(symbol);
        let sym_usize = usize::from(symbol);
        let new_sym_idx = self.nodes.len();
        let new_sym_node = TreeNode {
            rank: new_sym_rank,
            weight: 0,
            symbol: Some(sym_u16),
            parent: Some(nc_node_idx),
            left: None,
            right: None,
        };
        self.nodes.push(new_sym_node);

        if let Some(nc_node) = self.nodes.get_mut(nc_node_idx) {
            nc_node.left = Some(new_nc_idx);
            nc_node.right = Some(new_sym_idx);
        }

        if let Some(slot) = self.symbol_leaf.get_mut(nc_sym_idx) {
            *slot = Some(new_nc_idx);
        }
        if let Some(slot) = self.symbol_leaf.get_mut(sym_usize) {
            *slot = Some(new_sym_idx);
        }

        Ok(())
    }

    fn get_code_path(&self, symbol: u16) -> Result<Vec<u32>> {
        let sym_usize = usize::from(symbol);
        let leaf_idx = match self.symbol_leaf.get(sym_usize) {
            Some(&Some(idx)) => idx,
            _ => bail!("Symbol {symbol} not present in Huffman tree"),
        };

        let mut path = Vec::new();
        let mut curr = leaf_idx;
        while curr != self.root_idx {
            let parent_idx = match self.nodes.get(curr).and_then(|n| n.parent) {
                Some(p) => p,
                None => bail!("Node {curr} has missing parent before root"),
            };
            let parent_node = match self.nodes.get(parent_idx) {
                Some(p) => p,
                None => bail!("Parent node {parent_idx} not found"),
            };

            if parent_node.left == Some(curr) {
                path.push(0);
            } else if parent_node.right == Some(curr) {
                path.push(1);
            } else {
                bail!("Inconsistent tree parent-child relationship");
            }
            curr = parent_idx;
        }

        path.reverse();
        Ok(path)
    }

    fn encode_symbol<W: Write>(
        &mut self,
        writer: &mut BitWriter<W>,
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
        bail!("Input stream is empty; compact format requires at least 1 raw seed byte");
    }

    let first_byte = initial_byte_buf[0];
    let mut bit_writer = BitWriter::new(writer);

    // Write Magic Header 0xFF 0x1F and First Literal Byte C0
    bit_writer
        .writer
        .write_all(&COMPACT_MAGIC)
        .context("Failed to write compact magic header")?;
    bit_writer.bytes_written = bit_writer.bytes_written.saturating_add(2);

    bit_writer
        .writer
        .write_all(&[first_byte])
        .context("Failed to write first raw seed byte")?;
    bit_writer.bytes_written = bit_writer.bytes_written.saturating_add(1);

    let mut tree = CompactTree::new(first_byte);

    let mut buf = [0u8; 4096];
    loop {
        let bytes_read = reader
            .read(&mut buf)
            .context("Failed to read block from input stream")?;
        if bytes_read == 0 {
            break;
        }

        let slice = buf.get(..bytes_read).unwrap_or(&[]);
        for &byte in slice {
            let byte_u16 = u16::from(byte);
            let byte_usize = usize::from(byte);

            if let Some(&Some(leaf_idx)) = tree.symbol_leaf.get(byte_usize) {
                // Symbol already seen
                tree.encode_symbol(&mut bit_writer, byte_u16)?;
                tree.uptree(leaf_idx);
            } else {
                // Unseen symbol -> send NC escape, raw byte, insert into tree
                let nc_leaf_idx = match tree.symbol_leaf.get(usize::from(SYMBOL_NC)) {
                    Some(&Some(idx)) => idx,
                    _ => bail!("NC escape symbol missing from tree"),
                };
                tree.encode_symbol(&mut bit_writer, SYMBOL_NC)?;
                tree.uptree(nc_leaf_idx);

                bit_writer.write_bits(u32::from(byte), 8)?;
                tree.insert(byte)?;

                let new_leaf_idx = match tree.symbol_leaf.get(byte_usize) {
                    Some(&Some(idx)) => idx,
                    _ => bail!("Inserted symbol leaf missing from tree"),
                };
                tree.uptree(new_leaf_idx);
            }
        }
    }

    // Write End-of-File marker
    tree.encode_symbol(&mut bit_writer, SYMBOL_EF)?;
    bit_writer.flush_padding()?;

    Ok(bit_writer.bytes_written)
}

/// Decompresses `compact` compressed binary data from `reader` into `writer`.
pub fn decompress_compact_stream<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
) -> Result<u64> {
    let mut bit_reader = BitReader::new(reader);

    // Read and verify magic header (0xFF 0x1F)
    let magic_b0 = bit_reader
        .read_byte()
        .context("Failed to read magic byte 0")?;
    let magic_b1 = bit_reader
        .read_byte()
        .context("Failed to read magic byte 1")?;

    if [magic_b0, magic_b1] != COMPACT_MAGIC {
        bail!(
            "Invalid compact magic header: expected [0xFF, 0x1F], got [0x{magic_b0:02X}, 0x{magic_b1:02X}]"
        );
    }

    let first_byte = bit_reader
        .read_byte()
        .context("Failed to read first raw seed byte")?;

    writer
        .write_all(&[first_byte])
        .context("Failed to write first decompressed byte")?;
    let mut bytes_written: u64 = 1;

    let mut tree = CompactTree::new(first_byte);

    loop {
        let mut curr = tree.root_idx;
        while tree.nodes.get(curr).and_then(|n| n.symbol).is_none() {
            let bit = bit_reader.read_bit()?;
            let node = match tree.nodes.get(curr) {
                Some(n) => n,
                None => bail!("Corrupted node index {curr}"),
            };
            let next_node = if bit == 0 { node.left } else { node.right };
            match next_node {
                Some(idx) => curr = idx,
                None => bail!("Corrupted compact bitstream: reached internal node with missing child"),
            }
        }

        let symbol = match tree.nodes.get(curr).and_then(|n| n.symbol) {
            Some(sym) => sym,
            None => bail!("Internal error: leaf node missing symbol"),
        };

        if symbol == SYMBOL_EF {
            break;
        } else if symbol == SYMBOL_NC {
            tree.uptree(curr);
            let raw_byte = bit_reader.read_bits(8)?;
            tree.insert(raw_byte)?;
            writer
                .write_all(&[raw_byte])
                .context("Failed to write decompressed byte")?;
            bytes_written = bytes_written.saturating_add(1);

            let new_sym_idx = usize::from(raw_byte);
            let leaf_idx = match tree.symbol_leaf.get(new_sym_idx) {
                Some(&Some(idx)) => idx,
                _ => bail!("Newly inserted symbol leaf missing"),
            };
            tree.uptree(leaf_idx);
        } else {
            let byte = u8::try_from(symbol).context("Invalid data symbol value")?;
            writer
                .write_all(&[byte])
                .context("Failed to write decompressed byte")?;
            bytes_written = bytes_written.saturating_add(1);
            tree.uptree(curr);
        }
    }

    Ok(bytes_written)
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
        assert!(res.is_err());
    }

    #[crate::ctb_test]
    fn test_compact_empty_input() {
        let empty: &[u8] = &[];
        let mut out = Vec::new();
        let mut reader = empty;
        let res = compress_compact_stream(&mut reader, &mut out);
        assert!(res.is_err());
    }
}
