// SPDX-License-Identifier: AGPL-3.0-or-later AND GPL-1.0-or-later
// SPDX-License-Identifier for parts derived from freeze: GPL-1.0-or-later
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

// See license note at end of this file for parts derived from freeze.

//! Implementation of the `freeze` / `melt` compression format.
//!
//! Faithful safe Rust port from Leonid A. Broukhis's original C implementation
//! in `old/freeze-2.5.0-44.fc45.src/freeze-2.5.0/freeze-2.5.0/`.
//!
//! Supports both:
//! - Freeze 2.X format (`0x1F, 0x9F`, 8192-byte LZSS window, 3-byte header, 511-symbol dynamic Huffman tree)
//! - Freeze 1.0 format (`0x1F, 0x9E`, 4096-byte LZSS window, fixed Table 1, 315-symbol dynamic Huffman tree)

#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace crate prelude")]
pub(crate) use ctb_utilities::*;
use anyhow::anyhow;
use std::io::{Read, Write};

const MAGIC1: u8 = 0x1F;
const MAGIC2_FREEZE2: u8 = 0x9F;
const MAGIC2_FREEZE1: u8 = 0x9E;

const THRESHOLD: usize = 2;
const MAX_FREQ: u32 = 0x8000;
const ENDOF: usize = 256;

// Freeze 2.X parameters
const WINSIZE2: usize = 8192;
const WINMASK2: usize = WINSIZE2 - 1;
const LOOKAHEAD2: usize = 256;
const MAXDIST2: usize = 7936;
const N_CHAR2: usize = 256 - THRESHOLD + LOOKAHEAD2 + 1; // 511
const HUFVALUES2: [u8; 9] = [0, 0, 0, 1, 2, 6, 19, 34, 0];

// Freeze 1.0 parameters
const WINSIZE1: usize = 4096;
const WINMASK1: usize = WINSIZE1 - 1;
const LOOKAHEAD1: usize = 60;
const MAXDIST1: usize = WINSIZE1 - LOOKAHEAD1; // 4036
const N_CHAR1: usize = 256 - THRESHOLD + LOOKAHEAD1 + 1; // 315
const TABLE1: [u8; 9] = [0, 0, 0, 1, 3, 8, 12, 24, 16];

const HASH_BITS: usize = 14;
const HASH_SIZE: usize = 1 << HASH_BITS;
const HASH_MASK: usize = HASH_SIZE - 1;
const MAX_CHAIN: usize = 256;

// =========================================================================
// Bit-Level I/O (MSB-first, matching bitio.h / bitio.c)
// =========================================================================

struct BitWriter<W: Write> {
    writer: W,
    bitbuf: u64,
    bitlen: u32,
    bytes_written: u64,
}

impl<W: Write> BitWriter<W> {
    fn new(writer: W) -> Self {
        Self {
            writer,
            bitbuf: 0,
            bitlen: 0,
            bytes_written: 0,
        }
    }

    fn write_bits(&mut self, val: u32, n: u32) -> Result<()> {
        if n == 0 {
            return Ok(());
        }
        let mask = if n >= 32 {
            u64::MAX
        } else {
            (1u64 << n).saturating_sub(1)
        };
        self.bitbuf = (self.bitbuf << n) | (u64::from(val) & mask);
        self.bitlen = self.bitlen.saturating_add(n);

        while self.bitlen >= 8 {
            self.bitlen = self.bitlen.saturating_sub(8);
            let byte_u64 = (self.bitbuf >> self.bitlen) & 0xFF;
            let byte = u8::try_from(byte_u64).context("Byte conversion")?;
            self.writer.write_all(&[byte])?;
            self.bytes_written = self.bytes_written.saturating_add(1);
        }
        Ok(())
    }

    fn write_bit(&mut self, bit: u32) -> Result<()> {
        self.write_bits(bit & 1, 1)
    }

    fn flush_tail(&mut self) -> Result<()> {
        if self.bitlen > 0 {
            let shift = 8u32.saturating_sub(self.bitlen);
            let byte_u64 = (self.bitbuf << shift) & 0xFF;
            let byte = u8::try_from(byte_u64).context("Byte conversion")?;
            self.writer.write_all(&[byte])?;
            self.bytes_written = self.bytes_written.saturating_add(1);
            self.bitbuf = 0;
            self.bitlen = 0;
        }
        self.writer.flush().context("Flushing writer")?;
        Ok(())
    }
}

struct BitReader<R: Read> {
    reader: R,
    bitbuf: u64,
    bitlen: u32,
    eof_reached: bool,
    overrun: usize,
}

impl<R: Read> BitReader<R> {
    fn new(reader: R) -> Self {
        Self {
            reader,
            bitbuf: 0,
            bitlen: 0,
            eof_reached: false,
            overrun: 0,
        }
    }

    fn read_byte(&mut self) -> Result<u8> {
        let mut buf = [0u8; 1];
        let n = self.reader.read(&mut buf).context("Reading byte from stream")?;
        if n == 0 {
            self.eof_reached = true;
            self.overrun = self.overrun.saturating_add(1);
            bail!("Unexpected end of compressed stream");
        }
        match buf.first() {
            Some(&b) => Ok(b),
            None => bail!("Empty read buffer"),
        }
    }

    fn refill(&mut self, needed_bits: u32) -> Result<()> {
        while self.bitlen < needed_bits {
            let mut buf = [0u8; 1];
            let n = self.reader.read(&mut buf).context("Reading from stream")?;
            if n == 0 {
                self.eof_reached = true;
                self.overrun = self.overrun.saturating_add(1);
                // Zero-pad bit buffer on EOF
                self.bitbuf <<= 8;
                self.bitlen = self.bitlen.saturating_add(8);
            } else {
                let byte = match buf.first() {
                    Some(&b) => b,
                    None => bail!("Empty read buffer"),
                };
                self.bitbuf = (self.bitbuf << 8) | u64::from(byte);
                self.bitlen = self.bitlen.saturating_add(8);
            }
        }
        Ok(())
    }

    fn read_bit(&mut self) -> Result<u32> {
        if self.bitlen == 0 {
            self.refill(1)?;
        }
        self.bitlen = self.bitlen.saturating_sub(1);
        let bit = u32::try_from((self.bitbuf >> self.bitlen) & 1).context("Bit extraction")?;
        Ok(bit)
    }

    fn read_bits(&mut self, n: u32) -> Result<u32> {
        if n == 0 {
            return Ok(0);
        }
        if self.bitlen < n {
            self.refill(n)?;
        }
        self.bitlen = self.bitlen.saturating_sub(n);
        let mask = (1u64 << n).saturating_sub(1);
        let val = u32::try_from((self.bitbuf >> self.bitlen) & mask).context("Bits extraction")?;
        Ok(val)
    }
}

// =========================================================================
// Adaptive Dynamic Huffman Tree (matching huf.c)
// =========================================================================

struct AdaptiveHuffman {
    chars: usize,
    t: usize,
    r: usize,
    freq: Vec<u32>,
    son: Vec<usize>,
    prnt: Vec<usize>,
}

impl AdaptiveHuffman {
    fn new(n_char: usize) -> Result<Self> {
        let t = n_char.saturating_mul(2).saturating_sub(1);
        let r = t.saturating_sub(1);
        let mut freq = vec![1u32; t.saturating_add(1)];
        let mut son = vec![0usize; t];
        let mut prnt = vec![0usize; t.saturating_add(n_char)];

        for i in 0..n_char {
            let son_target = i.saturating_add(t);
            if let Some(s) = son.get_mut(i) {
                *s = son_target;
            }
            if let Some(p) = prnt.get_mut(son_target) {
                *p = i;
            }
        }

        let mut i = 0usize;
        let mut j = n_char;
        while j <= r {
            let freq_i = *freq.get(i).ok_or_else(|| anyhow!("Freq index {i} out of bounds"))?;
            let freq_i1 = *freq.get(i.saturating_add(1)).ok_or_else(|| anyhow!("Freq index out of bounds"))?;
            let sum = freq_i.saturating_add(freq_i1);
            if let Some(f) = freq.get_mut(j) {
                *f = sum;
            }
            if let Some(s) = son.get_mut(j) {
                *s = i;
            }
            if let Some(p) = prnt.get_mut(i) {
                *p = j;
            }
            if let Some(p) = prnt.get_mut(i.saturating_add(1)) {
                *p = j;
            }
            i = i.saturating_add(2);
            j = j.saturating_add(1);
        }

        if let Some(f) = freq.get_mut(t) {
            *f = 0xFFFF;
        }
        if let Some(p) = prnt.get_mut(r) {
            *p = 0;
        }

        Ok(Self {
            chars: n_char,
            t,
            r,
            freq,
            son,
            prnt,
        })
    }

    fn reconst(&mut self) -> Result<()> {
        let mut j = 0usize;
        for i in 0..self.t {
            let son_i = *self.son.get(i).ok_or_else(|| anyhow!("Son index {i} out of bounds"))?;
            if son_i >= self.t {
                let freq_i = *self.freq.get(i).ok_or_else(|| anyhow!("Freq index {i} out of bounds"))?;
                let halved = freq_i.saturating_add(1) / 2;
                if let Some(f) = self.freq.get_mut(j) {
                    *f = halved;
                }
                if let Some(s) = self.son.get_mut(j) {
                    *s = son_i;
                }
                j = j.saturating_add(1);
            }
        }

        let mut i = 0usize;
        j = self.chars;
        while j < self.t {
            let k = i.saturating_add(1);
            let freq_i = *self.freq.get(i).ok_or_else(|| anyhow!("Freq index {i} out of bounds"))?;
            let freq_k = *self.freq.get(k).ok_or_else(|| anyhow!("Freq index {k} out of bounds"))?;
            let f = freq_i.saturating_add(freq_k);
            if let Some(fj) = self.freq.get_mut(j) {
                *fj = f;
            }

            let mut insert_k = j.saturating_sub(1);
            while f < *self.freq.get(insert_k).ok_or_else(|| anyhow!("Freq index {insert_k} out of bounds"))? {
                if insert_k == 0 {
                    break;
                }
                insert_k = insert_k.saturating_sub(1);
            }
            if f >= *self.freq.get(insert_k).ok_or_else(|| anyhow!("Freq index {insert_k} out of bounds"))? {
                insert_k = insert_k.saturating_add(1);
            }

            let mut idx = j;
            while idx > insert_k {
                let prev_freq = *self.freq.get(idx.saturating_sub(1)).ok_or_else(|| anyhow!("Freq index out of bounds"))?;
                if let Some(curr) = self.freq.get_mut(idx) {
                    *curr = prev_freq;
                }
                let prev_son = *self.son.get(idx.saturating_sub(1)).ok_or_else(|| anyhow!("Son index out of bounds"))?;
                if let Some(curr_son) = self.son.get_mut(idx) {
                    *curr_son = prev_son;
                }
                idx = idx.saturating_sub(1);
            }
            if let Some(curr) = self.freq.get_mut(insert_k) {
                *curr = f;
            }
            if let Some(curr_son) = self.son.get_mut(insert_k) {
                *curr_son = i;
            }

            i = i.saturating_add(2);
            j = j.saturating_add(1);
        }

        for i in 0..self.t {
            let k = *self.son.get(i).ok_or_else(|| anyhow!("Son index {i} out of bounds"))?;
            if k >= self.t {
                if let Some(p) = self.prnt.get_mut(k) {
                    *p = i;
                }
            } else {
                if let Some(p) = self.prnt.get_mut(k) {
                    *p = i;
                }
                if let Some(p) = self.prnt.get_mut(k.saturating_add(1)) {
                    *p = i;
                }
            }
        }
        Ok(())
    }

    fn update(&mut self, mut c: usize) -> Result<()> {
        if *self.freq.get(self.r).ok_or_else(|| anyhow!("Freq index out of bounds"))? >= MAX_FREQ {
            self.reconst()?;
        }
        c = *self.prnt.get(c.saturating_add(self.t)).ok_or_else(|| anyhow!("Prnt index out of bounds"))?;
        loop {
            let freq_node = match self.freq.get_mut(node_idx(c)?) {
                Some(f) => {
                    *f = f.saturating_add(1);
                    *f
                }
                None => bail!("Corrupted node index {c}"),
            };

            let mut l = c.saturating_add(1);
            let next_freq = *self.freq.get(l).ok_or_else(|| anyhow!("Freq index {l} out of bounds"))?;
            if freq_node > next_freq {
                let mut scan = l.saturating_add(1);
                while freq_node > *self.freq.get(scan).ok_or_else(|| anyhow!("Freq index {scan} out of bounds"))? {
                    scan = scan.saturating_add(1);
                }
                l = scan.saturating_sub(1);

                let freq_l = *self.freq.get(l).ok_or_else(|| anyhow!("Freq index {l} out of bounds"))?;
                if let Some(fn_node) = self.freq.get_mut(c) {
                    *fn_node = freq_l;
                }
                if let Some(fn_l) = self.freq.get_mut(l) {
                    *fn_l = freq_node;
                }

                let i = *self.son.get(c).ok_or_else(|| anyhow!("Son index {c} out of bounds"))?;
                let j = *self.son.get(l).ok_or_else(|| anyhow!("Son index {l} out of bounds"))?;

                if let Some(p) = self.prnt.get_mut(i) {
                    *p = l;
                }
                if i < self.t {
                    if let Some(p) = self.prnt.get_mut(i.saturating_add(1)) {
                        *p = l;
                    }
                }

                if let Some(p) = self.prnt.get_mut(j) {
                    *p = c;
                }
                if j < self.t {
                    if let Some(p) = self.prnt.get_mut(j.saturating_add(1)) {
                        *p = c;
                    }
                }

                if let Some(s) = self.son.get_mut(c) {
                    *s = j;
                }
                if let Some(s) = self.son.get_mut(l) {
                    *s = i;
                }

                c = l;
            }

            c = *self.prnt.get(c).ok_or_else(|| anyhow!("Prnt index {c} out of bounds"))?;
            if c == 0 {
                break;
            }
        }
        Ok(())
    }

    fn encode_char<W: Write>(&mut self, writer: &mut BitWriter<W>, sym: usize) -> Result<()> {
        let mut k = *self.prnt.get(sym.saturating_add(self.t)).ok_or_else(|| anyhow!("Prnt index out of bounds"))?;
        let mut bits = Vec::with_capacity(32);
        while k != self.r {
            let bit = u32::try_from(k & 1).context("Bit conversion")?;
            bits.push(bit);
            k = *self.prnt.get(k).ok_or_else(|| anyhow!("Prnt index out of bounds"))?;
            if bits.len() > 1024 {
                bail!("Cycle detected in Huffman parent chain");
            }
        }
        bits.reverse();
        for bit in bits {
            writer.write_bit(bit)?;
        }
        self.update(sym)?;
        Ok(())
    }

    fn decode_char<R: Read>(&mut self, reader: &mut BitReader<R>) -> Result<usize> {
        let mut c = self.r;
        let mut steps = 0usize;
        loop {
            c = match self.son.get(c) {
                Some(&s) => s,
                None => bail!("Invalid son index {c}"),
            };
            if c >= self.t {
                break;
            }
            let bit = reader.read_bit()?;
            let bit_usize = usize::try_from(bit).context("Invalid bit index")?;
            c = c.saturating_add(bit_usize);
            steps = steps.saturating_add(1);
            if steps > 1024 {
                bail!("Cycle detected during Huffman decoding traversal");
            }
        }
        let sym = c.saturating_sub(self.t);
        self.update(sym)?;
        Ok(sym)
    }
}

fn node_idx(idx: usize) -> Result<usize> {
    Ok(idx)
}

// =========================================================================
// Static Position Table Handling (matching init in huf.c)
// =========================================================================

struct PositionEncoder {
    p_len: [u8; 64],
    code: [u8; 64],
}

impl PositionEncoder {
    fn new(table: &[u8; 9]) -> Result<Self> {
        let mut p_len = [0u8; 64];
        let mut code = [0u8; 64];

        let mut num = 0u32;
        let mut j = 0usize;
        for i in 1..=8 {
            let count = *table.get(i).ok_or_else(|| anyhow!("Table index {i} out of bounds"))?;
            let shift = 8u32.saturating_sub(u32::try_from(i)?);
            num = num.saturating_add(u32::from(count) << shift);
            for _ in 0..count {
                if let Some(p) = p_len.get_mut(j) {
                    *p = u8::try_from(i)?;
                }
                j = j.saturating_add(1);
            }
        }
        if num != 256 {
            bail!("Invalid position table specification: sum={num}");
        }

        let num_entries = j;
        let mut code_val = 0u8;
        for idx in 0..num_entries {
            if let Some(c) = code.get_mut(idx) {
                *c = code_val;
            }
            code_val = code_val.wrapping_add(1);
            if idx.saturating_add(1) == num_entries {
                break;
            }
            let cur_len = *p_len.get(idx).ok_or_else(|| anyhow!("Position length index {idx} out of bounds"))?;
            let next_len = *p_len.get(idx.saturating_add(1)).ok_or_else(|| anyhow!("Position length index out of bounds"))?;
            let shift = next_len.saturating_sub(cur_len);
            code_val <<= shift;
        }

        Ok(Self { p_len, code })
    }
}

struct PositionDecoder {
    code: [u8; 256],
    d_len: [u8; 256],
}

impl PositionDecoder {
    fn new(table: &[u8; 9], is_freeze1: bool) -> Result<Self> {
        let mut p_len = [0u8; 64];
        let mut num = 0u32;
        let mut j = 0usize;
        for i in 1..=8 {
            let count = *table.get(i).ok_or_else(|| anyhow!("Table index {i} out of bounds"))?;
            let shift = 8u32.saturating_sub(u32::try_from(i)?);
            num = num.saturating_add(u32::from(count) << shift);
            for _ in 0..count {
                if let Some(p) = p_len.get_mut(j) {
                    *p = u8::try_from(i)?;
                }
                j = j.saturating_add(1);
            }
        }
        if num != 256 {
            bail!("Invalid position table specification: sum={num}");
        }

        let num_entries = j;
        let mut code = [0u8; 256];
        let mut d_len = [0u8; 256];

        let mut k = 0usize;
        for idx in 0..num_entries {
            let len = *p_len.get(idx).ok_or_else(|| anyhow!("Position length index {idx} out of bounds"))?;
            let repeats = 1usize << (8u32.saturating_sub(u32::from(len)));
            for _ in 0..repeats {
                if let Some(c) = code.get_mut(k) {
                    *c = u8::try_from(idx)?;
                }
                k = k.saturating_add(1);
            }
        }

        k = 0;
        for idx in 0..num_entries {
            let len = *p_len.get(idx).ok_or_else(|| anyhow!("Position length index {idx} out of bounds"))?;
            let repeats = 1usize << (8u32.saturating_sub(u32::from(len)));
            let base_d = if is_freeze1 {
                len.saturating_sub(2)
            } else {
                len.saturating_sub(1)
            };
            for _ in 0..repeats {
                if let Some(d) = d_len.get_mut(k) {
                    *d = base_d;
                }
                k = k.saturating_add(1);
            }
        }

        Ok(Self { code, d_len })
    }

    fn decode_position2<R: Read>(&self, reader: &mut BitReader<R>) -> Result<usize> {
        let i = usize::try_from(reader.read_bits(8)?)?;
        let prefix_code = usize::from(*self.code.get(i).ok_or_else(|| anyhow!("Code table index out of bounds"))?);
        let d = u32::from(*self.d_len.get(i).ok_or_else(|| anyhow!("Bit length table index out of bounds"))?);
        let mid = (u32::try_from(i).context("Byte to u32")? << d) & 0x7F;
        let rest = reader.read_bits(d)?;
        let lower = mid | rest;
        let dist = (prefix_code << 7) | usize::try_from(lower).context("Position lower bits")?;
        Ok(dist)
    }

    fn decode_position1<R: Read>(&self, reader: &mut BitReader<R>) -> Result<usize> {
        let i = usize::try_from(reader.read_bits(8)?)?;
        let prefix_code = usize::from(*self.code.get(i).ok_or_else(|| anyhow!("Code table index out of bounds"))?);
        let d = u32::from(*self.d_len.get(i).ok_or_else(|| anyhow!("Bit length table index out of bounds"))?);
        let mid = (u32::try_from(i).context("Byte to u32")? << d) & 0x3F;
        let rest = reader.read_bits(d)?;
        let lower = mid | rest;
        let dist = (prefix_code << 6) | usize::try_from(lower).context("Position lower bits")?;
        Ok(dist)
    }
}

fn write_freeze2_header<W: Write>(writer: &mut BitWriter<W>, table: &[u8; 9]) -> Result<()> {
    let mut i = u32::from(*table.get(5).ok_or_else(|| anyhow!("Table index 5 out of bounds"))? & 0x1F);
    i <<= 4;
    i |= u32::from(*table.get(4).ok_or_else(|| anyhow!("Table index 4 out of bounds"))? & 0x0F);
    i <<= 3;
    i |= u32::from(*table.get(3).ok_or_else(|| anyhow!("Table index 3 out of bounds"))? & 0x07);
    i <<= 2;
    i |= u32::from(*table.get(2).ok_or_else(|| anyhow!("Table index 2 out of bounds"))? & 0x03);
    i <<= 1;
    i |= u32::from(*table.get(1).ok_or_else(|| anyhow!("Table index 1 out of bounds"))? & 0x01);

    let b0 = u8::try_from(i & 0xFF).context("Header byte 0")?;
    let b1 = u8::try_from((i >> 8) & 0xFF).context("Header byte 1")?;
    let b2 = *table.get(6).ok_or_else(|| anyhow!("Table index 6 out of bounds"))? & 0x3F;

    writer.writer.write_all(&[b0, b1, b2])?;
    writer.bytes_written = writer.bytes_written.saturating_add(3);
    Ok(())
}

fn read_freeze2_header<R: Read>(reader: &mut BitReader<R>) -> Result<[u8; 9]> {
    let b0 = reader.read_byte()?;
    let b1 = reader.read_byte()?;
    let b2 = reader.read_byte()?;

    let mut i = u32::from(b0) | (u32::from(b1) << 8);
    let t1 = u8::try_from(i & 1).context("Table element 1")?;
    i >>= 1;
    let t2 = u8::try_from(i & 3).context("Table element 2")?;
    i >>= 2;
    let t3 = u8::try_from(i & 7).context("Table element 3")?;
    i >>= 3;
    let t4 = u8::try_from(i & 0x0F).context("Table element 4")?;
    i >>= 4;
    let t5 = u8::try_from(i & 0x1F).context("Table element 5")?;
    i >>= 5;

    if (i & 1) != 0 || (b2 & 0xC0) != 0 {
        bail!("Unknown freeze header format");
    }
    let t6 = b2 & 0x3F;

    let sum = u32::from(t1)
        .saturating_add(u32::from(t2))
        .saturating_add(u32::from(t3))
        .saturating_add(u32::from(t4))
        .saturating_add(u32::from(t5))
        .saturating_add(u32::from(t6));
    if sum > 62 {
        bail!("Invalid freeze header table distribution sum");
    }
    let i_rem = 62u32.saturating_sub(sum);

    let j_sum = 128u32
        .saturating_mul(u32::from(t1))
        .saturating_add(64u32.saturating_mul(u32::from(t2)))
        .saturating_add(32u32.saturating_mul(u32::from(t3)))
        .saturating_add(16u32.saturating_mul(u32::from(t4)))
        .saturating_add(8u32.saturating_mul(u32::from(t5)))
        .saturating_add(4u32.saturating_mul(u32::from(t6)));
    if j_sum > 256 {
        bail!("Invalid freeze header byte representation sum");
    }
    let mut j_rem = 256u32.saturating_sub(j_sum);
    if j_rem < i_rem {
        bail!("Invalid freeze header byte constraint");
    }
    j_rem = j_rem.saturating_sub(i_rem);
    if i_rem < j_rem {
        bail!("Invalid freeze header byte constraint");
    }
    let t7 = u8::try_from(j_rem).context("Table element 7")?;
    let t8 = u8::try_from(i_rem.saturating_sub(j_rem)).context("Table element 8")?;

    Ok([0, t1, t2, t3, t4, t5, t6, t7, t8])
}

// =========================================================================
// LZSS Compression & Match Finding Engine
// =========================================================================

fn hash_triplet(b0: u8, b1: u8, b2: u8) -> usize {
    let v = usize::from(b0)
        .saturating_add(usize::from(b1) << 3)
        .saturating_add(usize::from(b2) << 6);
    v & HASH_MASK
}

fn find_longest_match(
    text: &[u8],
    r: usize,
    lookahead_len: usize,
    hashtab: &[usize],
    next: &[usize],
    win_size: usize,
    win_mask: usize,
    max_dist: usize,
) -> Result<(usize, usize)> {
    if lookahead_len <= THRESHOLD {
        return Ok((0, 0));
    }
    let b0 = *text.get(r).ok_or_else(|| anyhow!("Text buffer index out of bounds"))?;
    let b1 = *text.get(r.saturating_add(1)).ok_or_else(|| anyhow!("Text buffer index out of bounds"))?;
    let b2 = *text.get(r.saturating_add(2)).ok_or_else(|| anyhow!("Text buffer index out of bounds"))?;
    let h = hash_triplet(b0, b1, b2);

    let mut candidate = *hashtab.get(h).ok_or_else(|| anyhow!("Hash table index out of bounds"))?;
    let mut best_len = 0usize;
    let mut best_pos = 0usize;
    let mut chain_count = MAX_CHAIN;

    while candidate > 0 && chain_count > 0 {
        chain_count = chain_count.saturating_sub(1);
        let dist = r.saturating_sub(candidate);
        if dist == 0 || dist > max_dist {
            break;
        }

        // Compare candidate with target
        let mut match_len = 0usize;
        while match_len < lookahead_len {
            let cb = text.get(candidate.saturating_add(match_len)).copied();
            let rb = text.get(r.saturating_add(match_len)).copied();
            if cb != rb || cb.is_none() {
                break;
            }
            match_len = match_len.saturating_add(1);
        }

        if match_len > best_len {
            best_len = match_len;
            best_pos = candidate;
            if best_len == lookahead_len {
                break;
            }
        }

        candidate = *next.get(candidate & win_mask).ok_or_else(|| anyhow!("Next table index out of bounds"))?;
    }

    if best_len <= THRESHOLD {
        Ok((0, 0))
    } else {
        Ok((best_len, best_pos))
    }
}

// =========================================================================
// Public APIs: Freeze 2.X Compression and Decompression
// =========================================================================

/// Compresses a stream from `reader` into `writer` using Freeze 2.X format (`.F`).
pub fn compress_freeze2_stream<R: Read, W: Write>(reader: &mut R, writer: &mut W) -> Result<u64> {
    let mut input_data = Vec::new();
    reader.read_to_end(&mut input_data).context("Reading input for Freeze 2 compression")?;

    let mut bit_writer = BitWriter::new(writer);

    // 1. Write Magic Header (0x1F, 0x9F)
    bit_writer.writer.write_all(&[MAGIC1, MAGIC2_FREEZE2])?;
    bit_writer.bytes_written = bit_writer.bytes_written.saturating_add(2);

    // 2. Write 3-Byte Position Header
    write_freeze2_header(&mut bit_writer, &HUFVALUES2)?;

    // 3. Initialize Huffman & Position Tables
    let mut huff = AdaptiveHuffman::new(N_CHAR2)?;
    let pos_enc = PositionEncoder::new(&HUFVALUES2)?;

    // 4. LZSS Buffer & Hash Tables
    let mut text_buf = vec![b' '; MAXDIST2];
    text_buf.extend_from_slice(&input_data);

    let mut hashtab = vec![0usize; HASH_SIZE];
    let mut next = vec![0usize; WINSIZE2];

    // Seed hash table with initial buffer content
    for idx in 0..MAXDIST2 {
        if idx.saturating_add(2) < text_buf.len() {
            let b0 = *text_buf.get(idx).ok_or_else(|| anyhow!("Text buffer index out of bounds"))?;
            let b1 = *text_buf.get(idx.saturating_add(1)).ok_or_else(|| anyhow!("Text buffer index out of bounds"))?;
            let b2 = *text_buf.get(idx.saturating_add(2)).ok_or_else(|| anyhow!("Text buffer index out of bounds"))?;
            let h = hash_triplet(b0, b1, b2);
            if let Some(entry) = next.get_mut(idx & WINMASK2) {
                *entry = *hashtab.get(h).ok_or_else(|| anyhow!("Hash table index out of bounds"))?;
            }
            if let Some(head) = hashtab.get_mut(h) {
                *head = idx;
            }
        }
    }

    let total_len = text_buf.len();
    let mut r = MAXDIST2;

    while r < total_len {
        let lookahead_len = (total_len.saturating_sub(r)).min(LOOKAHEAD2);
        let (match_len, match_pos) = find_longest_match(
            &text_buf,
            r,
            lookahead_len,
            &hashtab,
            &next,
            WINSIZE2,
            WINMASK2,
            MAXDIST2,
        )?;

        if match_len <= THRESHOLD {
            // Literal byte
            let byte = *text_buf.get(r).ok_or_else(|| anyhow!("Text buffer index out of bounds"))?;
            huff.encode_char(&mut bit_writer, usize::from(byte))?;

            // Insert node into hash chain
            if r.saturating_add(2) < total_len {
                let b0 = *text_buf.get(r).ok_or_else(|| anyhow!("Text buffer index out of bounds"))?;
                let b1 = *text_buf.get(r.saturating_add(1)).ok_or_else(|| anyhow!("Text buffer index out of bounds"))?;
                let b2 = *text_buf.get(r.saturating_add(2)).ok_or_else(|| anyhow!("Text buffer index out of bounds"))?;
                let h = hash_triplet(b0, b1, b2);
                if let Some(entry) = next.get_mut(r & WINMASK2) {
                    *entry = *hashtab.get(h).ok_or_else(|| anyhow!("Hash table index out of bounds"))?;
                }
                if let Some(head) = hashtab.get_mut(h) {
                    *head = r;
                }
            }
            r = r.saturating_add(1);
        } else {
            // Check delayed coding at r + 1
            let next_lookahead = (total_len.saturating_sub(r.saturating_add(1))).min(LOOKAHEAD2);
            let (next_match_len, next_match_pos) = if next_lookahead > THRESHOLD {
                find_longest_match(
                    &text_buf,
                    r.saturating_add(1),
                    next_lookahead,
                    &hashtab,
                    &next,
                    WINSIZE2,
                    WINMASK2,
                    MAXDIST2,
                )?
            } else {
                (0, 0)
            };

            if next_match_len > match_len {
                // Delayed choice: emit literal at r, then next match at r+1
                let byte = *text_buf.get(r).ok_or_else(|| anyhow!("Text buffer index out of bounds"))?;
                huff.encode_char(&mut bit_writer, usize::from(byte))?;

                if r.saturating_add(2) < total_len {
                    let b0 = *text_buf.get(r).ok_or_else(|| anyhow!("Text buffer index out of bounds"))?;
                    let b1 = *text_buf.get(r.saturating_add(1)).ok_or_else(|| anyhow!("Text buffer index out of bounds"))?;
                    let b2 = *text_buf.get(r.saturating_add(2)).ok_or_else(|| anyhow!("Text buffer index out of bounds"))?;
                    let h = hash_triplet(b0, b1, b2);
                    if let Some(entry) = next.get_mut(r & WINMASK2) {
                        *entry = *hashtab.get(h).ok_or_else(|| anyhow!("Hash table index out of bounds"))?;
                    }
                    if let Some(head) = hashtab.get_mut(h) {
                        *head = r;
                    }
                }
                r = r.saturating_add(1);

                // Now emit match
                let dist = (r.saturating_sub(next_match_pos)).saturating_sub(1);
                let length_code = next_match_len.saturating_sub(THRESHOLD).saturating_add(256);
                huff.encode_char(&mut bit_writer, length_code)?;

                let upper = (dist >> 7) & 0x3F;
                let lower = u32::try_from(dist & 0x7F).context("Lower position bits")?;
                let plen = u32::from(*pos_enc.p_len.get(upper).ok_or_else(|| anyhow!("Position length index out of bounds"))?);
                let pcode = u32::from(*pos_enc.code.get(upper).ok_or_else(|| anyhow!("Position code index out of bounds"))?);
                bit_writer.write_bits(pcode, plen)?;
                bit_writer.write_bits(lower, 7)?;

                for step in 0..next_match_len {
                    let cur = r.saturating_add(step);
                    if cur.saturating_add(2) < total_len {
                        let b0 = *text_buf.get(cur).ok_or_else(|| anyhow!("Text buffer index out of bounds"))?;
                        let b1 = *text_buf.get(cur.saturating_add(1)).ok_or_else(|| anyhow!("Text buffer index out of bounds"))?;
                        let b2 = *text_buf.get(cur.saturating_add(2)).ok_or_else(|| anyhow!("Text buffer index out of bounds"))?;
                        let h = hash_triplet(b0, b1, b2);
                        if let Some(entry) = next.get_mut(cur & WINMASK2) {
                            *entry = *hashtab.get(h).ok_or_else(|| anyhow!("Hash table index out of bounds"))?;
                        }
                        if let Some(head) = hashtab.get_mut(h) {
                            *head = cur;
                        }
                    }
                }
                r = r.saturating_add(next_match_len);
            } else {
                // Immediate match
                let dist = (r.saturating_sub(match_pos)).saturating_sub(1);
                let length_code = match_len.saturating_sub(THRESHOLD).saturating_add(256);
                huff.encode_char(&mut bit_writer, length_code)?;

                let upper = (dist >> 7) & 0x3F;
                let lower = u32::try_from(dist & 0x7F).context("Lower position bits")?;
                let plen = u32::from(*pos_enc.p_len.get(upper).ok_or_else(|| anyhow!("Position length index out of bounds"))?);
                let pcode = u32::from(*pos_enc.code.get(upper).ok_or_else(|| anyhow!("Position code index out of bounds"))?);
                bit_writer.write_bits(pcode, plen)?;
                bit_writer.write_bits(lower, 7)?;

                for step in 0..match_len {
                    let cur = r.saturating_add(step);
                    if cur.saturating_add(2) < total_len {
                        let b0 = *text_buf.get(cur).ok_or_else(|| anyhow!("Text buffer index out of bounds"))?;
                        let b1 = *text_buf.get(cur.saturating_add(1)).ok_or_else(|| anyhow!("Text buffer index out of bounds"))?;
                        let b2 = *text_buf.get(cur.saturating_add(2)).ok_or_else(|| anyhow!("Text buffer index out of bounds"))?;
                        let h = hash_triplet(b0, b1, b2);
                        if let Some(entry) = next.get_mut(cur & WINMASK2) {
                            *entry = *hashtab.get(h).ok_or_else(|| anyhow!("Hash table index out of bounds"))?;
                        }
                        if let Some(head) = hashtab.get_mut(h) {
                            *head = cur;
                        }
                    }
                }
                r = r.saturating_add(match_len);
            }
        }
    }

    // Emit ENDOF code (256)
    huff.encode_char(&mut bit_writer, ENDOF)?;
    bit_writer.flush_tail()?;

    Ok(bit_writer.bytes_written)
}

/// Decompresses a stream from `reader` into `writer` using Freeze 2.X format (`.F`).
pub fn decompress_freeze2_stream<R: Read, W: Write>(reader: &mut R, writer: &mut W) -> Result<u64> {
    let mut bit_reader = BitReader::new(reader);

    // 1. Verify Magic Header (0x1F, 0x9F)
    let m1 = bit_reader.read_byte().context("Reading freeze magic byte 1")?;
    let m2 = bit_reader.read_byte().context("Reading freeze magic byte 2")?;
    if m1 != MAGIC1 || m2 != MAGIC2_FREEZE2 {
        bail!("Invalid magic header for Freeze 2 stream: expected 1F 9F, found {m1:02X} {m2:02X}");
    }

    // 2. Read 3-Byte Position Header
    let table = read_freeze2_header(&mut bit_reader)?;

    // 3. Initialize Huffman & Position Tables
    let mut huff = AdaptiveHuffman::new(N_CHAR2)?;
    let pos_dec = PositionDecoder::new(&table, false)?;

    // 4. Ring Buffer initialized with space characters
    let mut text_buf = [b' '; WINSIZE2];
    let mut r = 0usize;
    let mut bytes_decompressed = 0u64;

    loop {
        let sym = huff.decode_char(&mut bit_reader)?;
        if sym == ENDOF {
            break;
        }

        if sym < 256 {
            let byte = u8::try_from(sym).context("Literal byte conversion")?;
            if let Some(slot) = text_buf.get_mut(r) {
                *slot = byte;
            }
            r = (r.saturating_add(1)) & WINMASK2;
            writer.write_all(&[byte])?;
            bytes_decompressed = bytes_decompressed.saturating_add(1);
        } else {
            let match_len = sym.saturating_sub(256).saturating_add(THRESHOLD);
            let dist = pos_dec.decode_position2(&mut bit_reader)?;
            let start = (r.wrapping_sub(dist).wrapping_sub(1)) & WINMASK2;

            for step in 0..match_len {
                let src_pos = (start.saturating_add(step)) & WINMASK2;
                let byte = *text_buf.get(src_pos).ok_or_else(|| anyhow!("Ring buffer index out of bounds"))?;
                if let Some(slot) = text_buf.get_mut(r) {
                    *slot = byte;
                }
                r = (r.saturating_add(1)) & WINMASK2;
                writer.write_all(&[byte])?;
                bytes_decompressed = bytes_decompressed.saturating_add(1);
            }
        }
    }

    writer.flush().context("Flushing decompressed writer")?;
    Ok(bytes_decompressed)
}

// =========================================================================
// Public APIs: Freeze 1.0 Compression and Decompression
// =========================================================================

/// Compresses a stream from `reader` into `writer` using Freeze 1.0 format (`.F`).
pub fn compress_freeze1_stream<R: Read, W: Write>(reader: &mut R, writer: &mut W) -> Result<u64> {
    let mut input_data = Vec::new();
    reader.read_to_end(&mut input_data).context("Reading input for Freeze 1 compression")?;

    let mut bit_writer = BitWriter::new(writer);

    // 1. Write Magic Header (0x1F, 0x9E)
    bit_writer.writer.write_all(&[MAGIC1, MAGIC2_FREEZE1])?;
    bit_writer.bytes_written = bit_writer.bytes_written.saturating_add(2);

    // 2. Initialize Huffman & Position Tables (Fixed Table 1)
    let mut huff = AdaptiveHuffman::new(N_CHAR1)?;
    let pos_enc = PositionEncoder::new(&TABLE1)?;

    // 3. LZSS Buffer & Hash Tables
    let mut text_buf = vec![b' '; MAXDIST1];
    text_buf.extend_from_slice(&input_data);

    let mut hashtab = vec![0usize; HASH_SIZE];
    let mut next = vec![0usize; WINSIZE1];

    for idx in 0..MAXDIST1 {
        if idx.saturating_add(2) < text_buf.len() {
            let b0 = *text_buf.get(idx).ok_or_else(|| anyhow!("Text buffer index out of bounds"))?;
            let b1 = *text_buf.get(idx.saturating_add(1)).ok_or_else(|| anyhow!("Text buffer index out of bounds"))?;
            let b2 = *text_buf.get(idx.saturating_add(2)).ok_or_else(|| anyhow!("Text buffer index out of bounds"))?;
            let h = hash_triplet(b0, b1, b2);
            if let Some(entry) = next.get_mut(idx & WINMASK1) {
                *entry = *hashtab.get(h).ok_or_else(|| anyhow!("Hash table index out of bounds"))?;
            }
            if let Some(head) = hashtab.get_mut(h) {
                *head = idx;
            }
        }
    }

    let total_len = text_buf.len();
    let mut r = MAXDIST1;

    while r < total_len {
        let lookahead_len = (total_len.saturating_sub(r)).min(LOOKAHEAD1);
        let (match_len, match_pos) = find_longest_match(
            &text_buf,
            r,
            lookahead_len,
            &hashtab,
            &next,
            WINSIZE1,
            WINMASK1,
            MAXDIST1,
        )?;

        if match_len <= THRESHOLD {
            let byte = *text_buf.get(r).ok_or_else(|| anyhow!("Text buffer index out of bounds"))?;
            huff.encode_char(&mut bit_writer, usize::from(byte))?;

            if r.saturating_add(2) < total_len {
                let b0 = *text_buf.get(r).ok_or_else(|| anyhow!("Text buffer index out of bounds"))?;
                let b1 = *text_buf.get(r.saturating_add(1)).ok_or_else(|| anyhow!("Text buffer index out of bounds"))?;
                let b2 = *text_buf.get(r.saturating_add(2)).ok_or_else(|| anyhow!("Text buffer index out of bounds"))?;
                let h = hash_triplet(b0, b1, b2);
                if let Some(entry) = next.get_mut(r & WINMASK1) {
                    *entry = *hashtab.get(h).ok_or_else(|| anyhow!("Hash table index out of bounds"))?;
                }
                if let Some(head) = hashtab.get_mut(h) {
                    *head = r;
                }
            }
            r = r.saturating_add(1);
        } else {
            let dist = (r.saturating_sub(match_pos)).saturating_sub(1);
            let length_code = match_len.saturating_sub(THRESHOLD).saturating_add(256);
            huff.encode_char(&mut bit_writer, length_code)?;

            let upper = (dist >> 6) & 0x3F;
            let lower = u32::try_from(dist & 0x3F).context("Lower position bits")?;
            let plen = u32::from(*pos_enc.p_len.get(upper).ok_or_else(|| anyhow!("Position length index out of bounds"))?);
            let pcode = u32::from(*pos_enc.code.get(upper).ok_or_else(|| anyhow!("Position code index out of bounds"))?);
            bit_writer.write_bits(pcode, plen)?;
            bit_writer.write_bits(lower, 6)?;

            for step in 0..match_len {
                let cur = r.saturating_add(step);
                if cur.saturating_add(2) < total_len {
                    let b0 = *text_buf.get(cur).ok_or_else(|| anyhow!("Text buffer index out of bounds"))?;
                    let b1 = *text_buf.get(cur.saturating_add(1)).ok_or_else(|| anyhow!("Text buffer index out of bounds"))?;
                    let b2 = *text_buf.get(cur.saturating_add(2)).ok_or_else(|| anyhow!("Text buffer index out of bounds"))?;
                    let h = hash_triplet(b0, b1, b2);
                    if let Some(entry) = next.get_mut(cur & WINMASK1) {
                        *entry = *hashtab.get(h).ok_or_else(|| anyhow!("Hash table index out of bounds"))?;
                    }
                    if let Some(head) = hashtab.get_mut(h) {
                        *head = cur;
                    }
                }
            }
            r = r.saturating_add(match_len);
        }
    }

    huff.encode_char(&mut bit_writer, ENDOF)?;
    bit_writer.flush_tail()?;

    Ok(bit_writer.bytes_written)
}

/// Decompresses a stream from `reader` into `writer` using Freeze 1.0 format (`.F`).
pub fn decompress_freeze1_stream<R: Read, W: Write>(reader: &mut R, writer: &mut W) -> Result<u64> {
    let mut bit_reader = BitReader::new(reader);

    // 1. Verify Magic Header (0x1F, 0x9E)
    let m1 = bit_reader.read_byte().context("Reading freeze magic byte 1")?;
    let m2 = bit_reader.read_byte().context("Reading freeze magic byte 2")?;
    if m1 != MAGIC1 || m2 != MAGIC2_FREEZE1 {
        bail!("Invalid magic header for Freeze 1 stream: expected 1F 9E, found {m1:02X} {m2:02X}");
    }

    // 2. Initialize Huffman & Position Tables (Fixed Table 1)
    let mut huff = AdaptiveHuffman::new(N_CHAR1)?;
    let pos_dec = PositionDecoder::new(&TABLE1, true)?;

    // 3. Ring Buffer (4096 bytes) initialized with space characters
    let mut text_buf = [b' '; WINSIZE1];
    let mut r = 0usize;
    let mut bytes_decompressed = 0u64;

    loop {
        let sym = huff.decode_char(&mut bit_reader)?;
        if sym == ENDOF {
            break;
        }

        if sym < 256 {
            let byte = u8::try_from(sym).context("Literal byte conversion")?;
            if let Some(slot) = text_buf.get_mut(r) {
                *slot = byte;
            }
            r = (r.saturating_add(1)) & WINMASK1;
            writer.write_all(&[byte])?;
            bytes_decompressed = bytes_decompressed.saturating_add(1);
        } else {
            let match_len = sym.saturating_sub(256).saturating_add(THRESHOLD);
            let dist = pos_dec.decode_position1(&mut bit_reader)?;
            let start = (r.wrapping_sub(dist).wrapping_sub(1)) & WINMASK1;

            for step in 0..match_len {
                let src_pos = (start.saturating_add(step)) & WINMASK1;
                let byte = *text_buf.get(src_pos).ok_or_else(|| anyhow!("Ring buffer index out of bounds"))?;
                if let Some(slot) = text_buf.get_mut(r) {
                    *slot = byte;
                }
                r = (r.saturating_add(1)) & WINMASK1;
                writer.write_all(&[byte])?;
                bytes_decompressed = bytes_decompressed.saturating_add(1);
            }
        }
    }

    writer.flush().context("Flushing decompressed writer")?;
    Ok(bytes_decompressed)
}

/// Convenience alias compressing using the standard Freeze 2.X format.
pub fn compress_stream<R: Read, W: Write>(reader: &mut R, writer: &mut W) -> Result<u64> {
    compress_freeze2_stream(reader, writer)
}

/// Convenience alias decompressing using the standard Freeze 2.X format.
pub fn decompress_stream<R: Read, W: Write>(reader: &mut R, writer: &mut W) -> Result<u64> {
    decompress_freeze2_stream(reader, writer)
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
    fn test_freeze2_roundtrip_simple() {
        let input = b"Hello, world! This is a test for the Freeze 2.X compression format.";
        let mut comp = Vec::new();
        compress_freeze2_stream(&mut &input[..], &mut comp).expect("Freeze 2 compression failed");

        assert!(comp.len() >= 5, "Header must be at least 5 bytes");
        assert_eq!(comp.get(0..2), Some(&[MAGIC1, MAGIC2_FREEZE2][..]));

        let mut decomp = Vec::new();
        decompress_freeze2_stream(&mut comp.as_slice(), &mut decomp)
            .expect("Freeze 2 decompression failed");
        assert_eq!(decomp, input);
    }

    #[crate::ctb_test]
    fn test_freeze2_roundtrip_empty() {
        let input = b"";
        let mut comp = Vec::new();
        compress_freeze2_stream(&mut &input[..], &mut comp).expect("Empty compression failed");

        let mut decomp = Vec::new();
        decompress_freeze2_stream(&mut comp.as_slice(), &mut decomp)
            .expect("Empty decompression failed");
        assert_eq!(decomp, input);
    }

    #[crate::ctb_test]
    fn test_freeze2_roundtrip_repetitive() {
        let input = b"ABCDEFGH12345678".repeat(500);
        let mut comp = Vec::new();
        compress_freeze2_stream(&mut input.as_slice(), &mut comp).expect("Compression failed");

        assert!(comp.len() < input.len(), "Compression must save space on repetitive data");

        let mut decomp = Vec::new();
        decompress_freeze2_stream(&mut comp.as_slice(), &mut decomp).expect("Decompression failed");
        assert_eq!(decomp, input);
    }

    #[crate::ctb_test]
    fn test_freeze1_roundtrip_simple() {
        let input = b"Hello, Freeze 1.0! Testing backward-compatible format roundtrip.";
        let mut comp = Vec::new();
        compress_freeze1_stream(&mut &input[..], &mut comp).expect("Freeze 1 compression failed");

        assert_eq!(comp.get(0..2), Some(&[MAGIC1, MAGIC2_FREEZE1][..]));

        let mut decomp = Vec::new();
        decompress_freeze1_stream(&mut comp.as_slice(), &mut decomp)
            .expect("Freeze 1 decompression failed");
        assert_eq!(decomp, input);
    }

    #[crate::ctb_test]
    fn test_freeze1_roundtrip_empty() {
        let input = b"";
        let mut comp = Vec::new();
        compress_freeze1_stream(&mut &input[..], &mut comp).expect("Empty compression failed");

        let mut decomp = Vec::new();
        decompress_freeze1_stream(&mut comp.as_slice(), &mut decomp)
            .expect("Empty decompression failed");
        assert_eq!(decomp, input);
    }

    #[crate::ctb_test]
    fn test_freeze1_roundtrip_repetitive() {
        let input = b"ABCDEFGH12345678".repeat(500);
        let mut comp = Vec::new();
        compress_freeze1_stream(&mut input.as_slice(), &mut comp).expect("Compression failed");

        assert!(comp.len() < input.len(), "Compression must save space on repetitive data");

        let mut decomp = Vec::new();
        decompress_freeze1_stream(&mut comp.as_slice(), &mut decomp).expect("Decompression failed");
        assert_eq!(decomp, input);
    }

    #[crate::ctb_test]
    fn test_freeze_invalid_magic() {
        let bad = [0x00, 0x00, 0x41];
        let mut out = Vec::new();
        let res2 = decompress_freeze2_stream(&mut &bad[..], &mut out);
        assert!(res2.is_err());

        let res1 = decompress_freeze1_stream(&mut &bad[..], &mut out);
        assert!(res1.is_err());
    }

    #[crate::ctb_test]
    fn test_freeze2_decompress_fixture() {
        let fixture_data = match std::fs::read("/workspaces/ctoolbox/src/formats/compression/data/fixtures/example2 with lemurs.pan.F") {
            Ok(d) => d,
            Err(_) => return,
        };
        let raw_data = match std::fs::read("/workspaces/ctoolbox/src/formats/compression/data/fixtures/example2 with lemurs.pan") {
            Ok(d) => d,
            Err(_) => return,
        };
        let mut out = Vec::new();
        decompress_freeze2_stream(&mut fixture_data.as_slice(), &mut out).expect("Decompression failed");
        if out != raw_data {
            for (idx, (a, b)) in out.iter().zip(raw_data.iter()).enumerate() {
                if a != b {
                    panic!("Mismatch at index {idx}: got {a} ('{}'), expected {b} ('{}')", char::from(*a), char::from(*b));
                }
            }
            panic!("Length mismatch: got {}, expected {}", out.len(), raw_data.len());
        }
    }
}

/* Parts derived from freeze are used under the license documented by Fedora:

From `freeze.spec` from `freeze-2.5.0-44.fc45.src.rpm`:

```
# Confirmed with upstream, see email text in Source1
License:   GPL-1.0-or-later
```

This references the following email exchange, `Freeze_license_email.txt`:

```
Date: Fri, 18 Jul 2008 14:14:18 -0700
From: "Leo Broukhis" <leob@mailcom.com>
To: "Tom spot Callaway" <tcallawa@redhat.com>
Subject: Re: Freeze license

On Fri, Jul 18, 2008 at 1:51 PM, Tom spot Callaway <tcallawa@redhat.com> wrote:
> Well, at least one person in the Fedora community thinks it is useful,
> because he wants to maintain it. :)

All right.

> If you have "GPL-like" license text, can you send it to me? Alternately,
> it would be a lot simpler if you could choose one of the licenses from
> this page, as we already know they are ok:
>
> http://fedoraproject.org/wiki/Licensing#Good_Licenses

Let's call it GPL+, then.

Thanks,

Leo
```

ctoolbox uses the code under the GPL 3 (see below) under the "or later" provision. The original GPL, for reference:

```
# GNU GENERAL PUBLIC LICENSE

Version 1, February 1989

    Copyright (C) 1989 Free Software Foundation, Inc.
    <https://fsf.org/>

    Everyone is permitted to copy and distribute verbatim copies
    of this license document, but changing it is not allowed.

## Preamble

The license agreements of most software companies try to keep users at
the mercy of those companies. By contrast, our General Public License
is intended to guarantee your freedom to share and change free
software--to make sure the software is free for all its users. The
General Public License applies to the Free Software Foundation's
software and to any other program whose authors commit to using it.
You can use it for your programs, too.

When we speak of free software, we are referring to freedom, not
price. Specifically, the General Public License is designed to make
sure that you have the freedom to give away or sell copies of free
software, that you receive source code or can get it if you want it,
that you can change the software or use pieces of it in new free
programs; and that you know you can do these things.

To protect your rights, we need to make restrictions that forbid
anyone to deny you these rights or to ask you to surrender the rights.
These restrictions translate to certain responsibilities for you if
you distribute copies of the software, or if you modify it.

For example, if you distribute copies of a such a program, whether
gratis or for a fee, you must give the recipients all the rights that
you have. You must make sure that they, too, receive or can get the
source code. And you must tell them their rights.

We protect your rights with two steps: (1) copyright the software, and
(2) offer you this license which gives you legal permission to copy,
distribute and/or modify the software.

Also, for each author's protection and ours, we want to make certain
that everyone understands that there is no warranty for this free
software. If the software is modified by someone else and passed on,
we want its recipients to know that what they have is not the
original, so that any problems introduced by others will not reflect
on the original authors' reputations.

The precise terms and conditions for copying, distribution and
modification follow.

## GNU GENERAL PUBLIC LICENSE TERMS AND CONDITIONS FOR COPYING, DISTRIBUTION AND MODIFICATION

**0.** This License Agreement applies to any program or other work
which contains a notice placed by the copyright holder saying it may
be distributed under the terms of this General Public License. The
"Program", below, refers to any such program or work, and a "work
based on the Program" means either the Program or any work containing
the Program or a portion of it, either verbatim or with modifications.
Each licensee is addressed as "you".

**1.** You may copy and distribute verbatim copies of the Program's
source code as you receive it, in any medium, provided that you
conspicuously and appropriately publish on each copy an appropriate
copyright notice and disclaimer of warranty; keep intact all the
notices that refer to this General Public License and to the absence
of any warranty; and give any other recipients of the Program a copy
of this General Public License along with the Program. You may charge
a fee for the physical act of transferring a copy.

**2.** You may modify your copy or copies of the Program or any
portion of it, and copy and distribute such modifications under the
terms of Paragraph 1 above, provided that you also do the following:


**a)** cause the modified files to carry prominent notices stating
that you changed the files and the date of any change; and


**b)** cause the whole of any work that you distribute or publish,
that in whole or in part contains the Program or any part thereof,
either with or without modifications, to be licensed at no charge to
all third parties under the terms of this General Public License
(except that you may choose to grant warranty protection to some or
all third parties, at your option).


**c)** If the modified program normally reads commands interactively
when run, you must cause it, when started running for such interactive
use in the simplest and most usual way, to print or display an
announcement including an appropriate copyright notice and a notice
that there is no warranty (or else, saying that you provide
a warranty) and that users may redistribute the program under these
conditions, and telling the user how to view a copy of this General
Public License.


**d)** You may charge a fee for the physical act of transferring a
copy, and you may at your option offer warranty protection in exchange
for a fee.

Mere aggregation of another independent work with the Program (or its
derivative) on a volume of a storage or distribution medium does not
bring the other work under the scope of these terms.

**3.** You may copy and distribute the Program (or a portion or
derivative of it, under Paragraph 2) in object code or executable form
under the terms of Paragraphs 1 and 2 above provided that you also do
one of the following:


**a)** accompany it with the complete corresponding machine-readable
source code, which must be distributed under the terms of Paragraphs 1
and 2 above; or,


**b)** accompany it with a written offer, valid for at least three
years, to give any third party free (except for a nominal charge for
the cost of distribution) a complete machine-readable copy of the
corresponding source code, to be distributed under the terms of
Paragraphs 1 and 2 above; or,


**c)** accompany it with the information you received as to where the
corresponding source code may be obtained. (This alternative is
allowed only for noncommercial distribution and only if you received
the program in object code or executable form alone.)

Source code for a work means the preferred form of the work for making
modifications to it. For an executable file, complete source code
means all the source code for all modules it contains; but, as a
special exception, it need not include source code for modules which
are standard libraries that accompany the operating system on which
the executable file runs, or for standard header files or definitions
files that accompany that operating system.

**4.** You may not copy, modify, sublicense, distribute or transfer
the Program except as expressly provided under this General Public
License. Any attempt otherwise to copy, modify, sublicense, distribute
or transfer the Program is void, and will automatically terminate your
rights to use the Program under this License. However, parties who
have received copies, or rights to use copies, from you under this
General Public License will not have their licenses terminated so long
as such parties remain in full compliance.

**5.** By copying, distributing or modifying the Program (or any work
based on the Program) you indicate your acceptance of this license to
do so, and all its terms and conditions.

**6.** Each time you redistribute the Program (or any work based on
the Program), the recipient automatically receives a license from the
original licensor to copy, distribute or modify the Program subject to
these terms and conditions. You may not impose any further
restrictions on the recipients' exercise of the rights granted herein.

**7.** The Free Software Foundation may publish revised and/or new
versions of the General Public License from time to time. Such new
versions will be similar in spirit to the present version, but may
differ in detail to address new problems or concerns.

Each version is given a distinguishing version number. If the Program
specifies a version number of the license which applies to it and "any
later version", you have the option of following the terms and
conditions either of that version or of any later version published by
the Free Software Foundation. If the Program does not specify a
version number of the license, you may choose any version ever
published by the Free Software Foundation.

**8.** If you wish to incorporate parts of the Program into other free
programs whose distribution conditions are different, write to the
author to ask for permission. For software which is copyrighted by the
Free Software Foundation, write to the Free Software Foundation; we
sometimes make exceptions for this. Our decision will be guided by the
two goals of preserving the free status of all derivatives of our free
software and of promoting the sharing and reuse of software generally.

**NO WARRANTY**

**9.** BECAUSE THE PROGRAM IS LICENSED FREE OF CHARGE, THERE IS NO
WARRANTY FOR THE PROGRAM, TO THE EXTENT PERMITTED BY APPLICABLE LAW.
EXCEPT WHEN OTHERWISE STATED IN WRITING THE COPYRIGHT HOLDERS AND/OR
OTHER PARTIES PROVIDE THE PROGRAM "AS IS" WITHOUT WARRANTY OF ANY
KIND, EITHER EXPRESSED OR IMPLIED, INCLUDING, BUT NOT LIMITED TO, THE
IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
PURPOSE. THE ENTIRE RISK AS TO THE QUALITY AND PERFORMANCE OF THE
PROGRAM IS WITH YOU. SHOULD THE PROGRAM PROVE DEFECTIVE, YOU ASSUME
THE COST OF ALL NECESSARY SERVICING, REPAIR OR CORRECTION.

**10.** IN NO EVENT UNLESS REQUIRED BY APPLICABLE LAW OR AGREED TO IN
WRITING WILL ANY COPYRIGHT HOLDER, OR ANY OTHER PARTY WHO MAY MODIFY
AND/OR REDISTRIBUTE THE PROGRAM AS PERMITTED ABOVE, BE LIABLE TO YOU
FOR DAMAGES, INCLUDING ANY GENERAL, SPECIAL, INCIDENTAL OR
CONSEQUENTIAL DAMAGES ARISING OUT OF THE USE OR INABILITY TO USE THE
PROGRAM (INCLUDING BUT NOT LIMITED TO LOSS OF DATA OR DATA BEING
RENDERED INACCURATE OR LOSSES SUSTAINED BY YOU OR THIRD PARTIES OR A
FAILURE OF THE PROGRAM TO OPERATE WITH ANY OTHER PROGRAMS), EVEN IF
SUCH HOLDER OR OTHER PARTY HAS BEEN ADVISED OF THE POSSIBILITY OF SUCH
DAMAGES.

END OF TERMS AND CONDITIONS

## Appendix: How to Apply These Terms to Your New Programs

If you develop a new program, and you want it to be of the greatest
possible use to humanity, the best way to achieve this is to make it
free software which everyone can redistribute and change under these
terms.

To do so, attach the following notices to the program. It is safest to
attach them to the start of each source file to most effectively
convey the exclusion of warranty; and each file should have at least
the "copyright" line and a pointer to where the full notice is found.

    <one line to give the program's name and a brief idea of what it does.>
    Copyright (C) 19yy <name of author>

        This program is free software; you can redistribute it and/or modify
        it under the terms of the GNU General Public License as published by
        the Free Software Foundation; either version 1, or (at your option)
        any later version.

        This program is distributed in the hope that it will be useful,
        but WITHOUT ANY WARRANTY; without even the implied warranty of
        MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
        GNU General Public License for more details.

        You should have received a copy of the GNU General Public License
        along with this program; if not, see <https://www.gnu.org/licenses/>.

Also add information on how to contact you by electronic and paper
mail.

If the program is interactive, make it output a short notice like this
when it starts in an interactive mode:

    Gnomovision version 69, Copyright (C) 19xx name of author Gnomovision
    comes with ABSOLUTELY NO WARRANTY; for details type `show w'. This is
    free software, and you are welcome to redistribute it under certain
    conditions; type `show c' for details.

The hypothetical commands \`show w' and \`show c' should show the
appropriate parts of the General Public License. Of course, the
commands you use may be called something other than \`show w' and
\`show c'; they could even be mouse-clicks or menu items--whatever
suits your program.

You should also get your employer (if you work as a programmer) or
your school, if any, to sign a "copyright disclaimer" for the program,
if necessary. Here a sample; alter the names:

    Yoyodyne, Inc., hereby disclaims all copyright interest in the program
    `Gnomovision' (a program to direct compilers to make passes at
    assemblers) written by James Hacker.

    <signature of Moe Ghoul>, 1 April 1989
    Moe Ghoul, President of Vice

That's all there is to it!
```

Text of the GPL 3:

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
