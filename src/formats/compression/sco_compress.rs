// SPDX-License-Identifier: AGPL-3.0-or-later AND GPL-3.0-or-later
// SPDX-License-Identifier for parts derived from from gzip: GPL-3.0-or-later
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

// Header comment from original unlzh.c:
/* unlzh.c -- decompress files in SCO compress -H (LZH) format.
 * The code in this file is directly derived from the public domain 'ar002'
 * written by Haruhiko Okumura.
 */

// AUTHORS file for gzip overall:
/* gzip was written by Jean-loup Gailly <jloup@gzip.org>,
and Mark Adler for the decompression code. */

//! SCO `compress -H` (LZH / `-lh1-`) compression and decompression format.
//!
//! Specification reference: `data/docs/compress-sco.md`
//! Reference decompressor: `old/unix-tools/gzip-1.14/gzip-1.14/unlzh.c`

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;
use std::io::{Read, Write};

/// Magic header bytes for SCO `compress -H` (`0x1F`, `0xA0`).
pub const MAGIC_BYTES: [u8; 2] = [0x1F, 0xA0];

const DICBIT: usize = 13;
const DICSIZ: usize = 1 << DICBIT; // 8192
const MAXMATCH: usize = 256;
const THRESHOLD: usize = 3;

const NC: usize = 510; // Alphabet = 0..509
const CBIT: usize = 9;
const CODE_BIT: usize = 16;

const NP: usize = DICBIT + 1; // 14
const NT: usize = CODE_BIT + 3; // 19
const PBIT: usize = 4;
const TBIT: usize = 5;

fn safe_shl(val: u32, shift: i32) -> u32 {
    if (0..32).contains(&shift) {
        val << shift
    } else {
        0
    }
}

fn safe_shr(val: u32, shift: i32) -> u32 {
    if (0..32).contains(&shift) {
        val >> shift
    } else {
        0
    }
}

// Bitstream reader for MSB-first bit unpacking (matching unlzh.c io.c).
struct BitReader<'a, R: Read> {
    reader: &'a mut R,
    bitbuf: u16,
    subbitbuf: u32,
    bitcount: i32,
}

impl<'a, R: Read> BitReader<'a, R> {
    fn new(reader: &'a mut R) -> Result<Self> {
        let mut br = Self {
            reader,
            bitbuf: 0,
            subbitbuf: 0,
            bitcount: 0,
        };
        br.fillbuf(16)?;
        Ok(br)
    }

    fn fillbuf(&mut self, mut n: i32) -> Result<()> {
        if self.bitcount <= 0 {
            self.bitcount = 0;
            self.subbitbuf = 0;
        }
        let shifted = safe_shl(u32::from(self.bitbuf), n) & 0xFFFF;
        self.bitbuf = u16::try_from(shifted)?;

        while n > self.bitcount {
            n = n.saturating_sub(self.bitcount);
            if n < 32 {
                let addition =
                    u16::try_from(safe_shl(self.subbitbuf, n) & 0xFFFF)?;
                self.bitbuf |= addition;
            }

            let mut byte_buf = [0u8; 1];
            let read_bytes = self.reader.read(&mut byte_buf)?;
            self.subbitbuf = if read_bytes == 0 {
                0
            } else {
                u32::from(byte_buf[0])
            };
            self.bitcount = 8;
        }

        self.bitcount = self.bitcount.saturating_sub(n);
        let addition = u16::try_from(
            safe_shr(self.subbitbuf, self.bitcount.max(0)) & 0xFFFF,
        )?;
        self.bitbuf |= addition;
        Ok(())
    }

    fn getbits(&mut self, n: u32) -> Result<u16> {
        if n == 0 {
            return Ok(0);
        }
        let shift = i32::try_from(16u32.saturating_sub(n))?;
        let mask = if n >= 16 {
            0xFFFFu16
        } else {
            (1u16 << n).wrapping_sub(1)
        };
        let x = u16::try_from(safe_shr(u32::from(self.bitbuf), shift))? & mask;
        self.fillbuf(i32::try_from(n)?)?;
        Ok(x)
    }

    #[expect(
        clippy::expect_used,
        reason = "Bitwise AND masks guarantee values fit within target integer types"
    )]
    fn peekbits3(&self) -> u16 {
        u16::try_from(safe_shr(u32::from(self.bitbuf), 13)).expect("bitbuf shifted fits in u16") & 7
    }

    #[expect(
        clippy::expect_used,
        reason = "Bitwise AND masks guarantee values fit within target integer types"
    )]
    fn peekbits8(&self) -> usize {
        usize::try_from(safe_shr(u32::from(self.bitbuf), 8) & 0xFF).expect("masked byte fits in usize")
    }

    #[expect(
        clippy::expect_used,
        reason = "Bitwise AND masks guarantee values fit within target integer types"
    )]
    fn peekbits12(&self) -> usize {
        usize::try_from(safe_shr(u32::from(self.bitbuf), 4) & 0xFFF)
            .expect("masked bits fit in usize")
    }

    fn dropbits(&mut self, n: u32) -> Result<()> {
        if n > 0 {
            self.fillbuf(i32::try_from(n)?)?;
        }
        Ok(())
    }
}

// Location pointer target for make_table building.
#[derive(Clone, Copy)]
enum TableLoc {
    Table(usize),
    Left(usize),
    Right(usize),
}

fn make_table(
    nchar: usize,
    bitlen: &[u8],
    tablebits: usize,
    table: &mut [u16],
    left: &mut [u16],
    right: &mut [u16],
) -> Result<()> {
    let mut count = [0u16; 17];
    let mut weight = [0u16; 17];
    let mut start = [0u16; 18];

    for &len in bitlen.iter().take(nchar) {
        let len_idx = usize::from(len);
        if let Some(c) = count.get_mut(len_idx) {
            *c = c.saturating_add(1);
        }
    }

    if let Some(s) = start.get_mut(1) {
        *s = 0;
    }
    for i in 1..=16 {
        let count_i = u32::from(
            *count
                .get(i)
                .ok_or_else(|| anyhow::anyhow!("count index {i} out of bounds"))?,
        );
        let shift = 16u32.saturating_sub(u32::try_from(i)?);
        let prev_start = u32::from(
            *start
                .get(i)
                .ok_or_else(|| anyhow::anyhow!("start index {i} out of bounds"))?,
        );
        let next_start = prev_start.wrapping_add(count_i << shift);
        let next_i = i.saturating_add(1);
        if let Some(s) = start.get_mut(next_i) {
            let s_val = u16::try_from(next_start & 0xFFFF)
                .context("next_start low 16 bits fit in u16")?;
            *s = s_val;
        }
    }

    let start_17 = *start
        .get(17)
        .ok_or_else(|| anyhow::anyhow!("start index 17 out of bounds"))?;
    if (u32::from(start_17) & 0xFFFF) != 0 {
        bail!("Bad table: sum of code weights invalid");
    }

    let jutbits = 16usize.saturating_sub(tablebits);
    let mut i = 1usize;
    while i <= tablebits {
        if let Some(s) = start.get_mut(i) {
            *s >>= jutbits;
        }
        let w_shift = u32::try_from(tablebits.saturating_sub(i))?;
        if let Some(w) = weight.get_mut(i) {
            *w = 1u16 << w_shift;
        }
        i = i.saturating_add(1);
    }
    while i <= 16 {
        let w_shift = u32::try_from(16usize.saturating_sub(i))?;
        if let Some(w) = weight.get_mut(i) {
            *w = 1u16 << w_shift;
        }
        i = i.saturating_add(1);
    }

    let next_tablebits = tablebits.saturating_add(1);
    let start_tbl = usize::from(
        *start
            .get(next_tablebits)
            .ok_or_else(|| anyhow::anyhow!("start index {next_tablebits} out of bounds"))?
            >> jutbits,
    );
    if start_tbl != 0 {
        let k = 1usize << tablebits;
        let mut idx = start_tbl;
        while idx < k && idx < table.len() {
            if let Some(t) = table.get_mut(idx) {
                *t = 0;
            }
            idx = idx.saturating_add(1);
        }
    }

    let mut avail = u16::try_from(nchar)?;
    let mask_shift = 15usize.saturating_sub(tablebits);
    let mask = 1u16 << mask_shift;

    for ch in 0..nchar {
        let len = usize::from(
            *bitlen
                .get(ch)
                .ok_or_else(|| anyhow::anyhow!("bitlen index {ch} out of bounds"))?,
        );
        if len == 0 {
            continue;
        }
        let ch_u16 = u16::try_from(ch)?;
        let curr_start = *start
            .get(len)
            .ok_or_else(|| anyhow::anyhow!("start index {len} out of bounds"))?;
        let curr_weight = *weight
            .get(len)
            .ok_or_else(|| anyhow::anyhow!("weight index {len} out of bounds"))?;
        let nextcode = curr_start.wrapping_add(curr_weight);

        if len <= tablebits {
            let max_code = 1u16 << tablebits;
            if max_code < nextcode {
                bail!("Bad table: nextcode overflow");
            }
            for tbl_idx in usize::from(curr_start)..usize::from(nextcode) {
                if let Some(t) = table.get_mut(tbl_idx) {
                    *t = ch_u16;
                }
            }
        } else {
            let mut k = curr_start;
            let mut loc = TableLoc::Table(usize::from(k >> jutbits));
            let mut tree_depth = len.saturating_sub(tablebits);

            while tree_depth != 0 {
                let curr_val = match loc {
                    TableLoc::Table(idx) => *table
                        .get(idx)
                        .ok_or_else(|| anyhow::anyhow!("table index {idx} out of bounds"))?,
                    TableLoc::Left(idx) => *left
                        .get(idx)
                        .ok_or_else(|| anyhow::anyhow!("left tree index {idx} out of bounds"))?,
                    TableLoc::Right(idx) => *right
                        .get(idx)
                        .ok_or_else(|| anyhow::anyhow!("right tree index {idx} out of bounds"))?,
                };

                let node_val = if curr_val == 0 {
                    let avail_val = avail;
                    avail = avail.saturating_add(1);
                    let avail_idx = usize::from(avail_val);
                    if let Some(l) = left.get_mut(avail_idx) {
                        *l = 0;
                    }
                    if let Some(r) = right.get_mut(avail_idx) {
                        *r = 0;
                    }
                    match loc {
                        TableLoc::Table(idx) => {
                            if let Some(t) = table.get_mut(idx) {
                                *t = avail_val;
                            }
                        }
                        TableLoc::Left(idx) => {
                            if let Some(l) = left.get_mut(idx) {
                                *l = avail_val;
                            }
                        }
                        TableLoc::Right(idx) => {
                            if let Some(r) = right.get_mut(idx) {
                                *r = avail_val;
                            }
                        }
                    }
                    avail_val
                } else {
                    curr_val
                };

                let node_idx = usize::from(node_val);
                if (k & mask) != 0 {
                    loc = TableLoc::Right(node_idx);
                } else {
                    loc = TableLoc::Left(node_idx);
                }
                k <<= 1;
                tree_depth = tree_depth.saturating_sub(1);
            }

            match loc {
                TableLoc::Table(idx) => {
                    if let Some(t) = table.get_mut(idx) {
                        *t = ch_u16;
                    }
                }
                TableLoc::Left(idx) => {
                    if let Some(l) = left.get_mut(idx) {
                        *l = ch_u16;
                    }
                }
                TableLoc::Right(idx) => {
                    if let Some(r) = right.get_mut(idx) {
                        *r = ch_u16;
                    }
                }
            }
        }
        if let Some(s) = start.get_mut(len) {
            *s = nextcode;
        }
    }
    Ok(())
}

fn read_pt_len<R: Read>(
    br: &mut BitReader<R>,
    nn: usize,
    nbit: u32,
    i_special: i32,
    pt_len: &mut [u8],
    pt_table: &mut [u16],
    left: &mut [u16],
    right: &mut [u16],
) -> Result<()> {
    let n = usize::from(br.getbits(nbit)?);
    if n == 0 {
        let c = br.getbits(nbit)?;
        for len in pt_len.iter_mut().take(nn) {
            *len = 0;
        }
        for entry in pt_table.iter_mut().take(256) {
            *entry = c;
        }
    } else {
        let mut i = 0usize;
        while i < n {
            let mut c = br.peekbits3();
            if c == 7 {
                let mut mask = 0x1000u32;
                let bitbuf_u32 = u32::from(br.bitbuf);
                while (mask & bitbuf_u32) != 0 {
                    mask >>= 1;
                    c = c.saturating_add(1);
                }
                if c > 16 {
                    bail!("Bad table: PT bit length exceeds 16");
                }
            }
            let consumed_bits = if c < 7 {
                3
            } else {
                u32::from(c).saturating_sub(3)
            };
            br.dropbits(consumed_bits)?;

            if let Some(l) = pt_len.get_mut(i) {
                *l = u8::try_from(c)?;
            }
            i = i.saturating_add(1);

            if i_special >= 0 && i == usize::try_from(i_special)? {
                let mut zero_count = usize::from(br.getbits(2)?);
                while zero_count > 0 {
                    if let Some(l) = pt_len.get_mut(i) {
                        *l = 0;
                    }
                    i = i.saturating_add(1);
                    zero_count = zero_count.saturating_sub(1);
                }
            }
        }
        while i < nn {
            if let Some(l) = pt_len.get_mut(i) {
                *l = 0;
            }
            i = i.saturating_add(1);
        }
        make_table(nn, pt_len, 8, pt_table, left, right)?;
    }
    Ok(())
}

fn read_c_len<R: Read>(
    br: &mut BitReader<R>,
    c_len: &mut [u8],
    c_table: &mut [u16],
    pt_len: &[u8],
    pt_table: &[u16],
    left: &mut [u16],
    right: &mut [u16],
) -> Result<()> {
    let cbit_u32 = u32::try_from(CBIT)?;
    let n = usize::from(br.getbits(cbit_u32)?);
    if n == 0 {
        let c = br.getbits(cbit_u32)?;
        for len in c_len.iter_mut().take(NC) {
            *len = 0;
        }
        for entry in c_table.iter_mut().take(4096) {
            *entry = c;
        }
    } else {
        let mut i = 0usize;
        while i < n {
            let peek_idx = br.peekbits8();
            let mut c = usize::from(
                *pt_table
                    .get(peek_idx)
                    .ok_or_else(|| anyhow::anyhow!("pt_table index {peek_idx} out of bounds"))?,
            );
            if c >= NT {
                let mut mask = 0x80u16;
                while c >= NT {
                    if (br.bitbuf & mask) != 0 {
                        c = usize::from(
                            *right
                                .get(c)
                                .ok_or_else(|| anyhow::anyhow!("right tree index {c} out of bounds"))?,
                        );
                    } else {
                        c = usize::from(
                            *left
                                .get(c)
                                .ok_or_else(|| anyhow::anyhow!("left tree index {c} out of bounds"))?,
                        );
                    }
                    mask >>= 1;
                }
            }
            let pt_bits = u32::from(
                *pt_len
                    .get(c)
                    .ok_or_else(|| anyhow::anyhow!("pt_len index {c} out of bounds"))?,
            );
            br.dropbits(pt_bits)?;

            if c <= 2 {
                let mut count = if c == 0 {
                    1usize
                } else if c == 1 {
                    usize::from(br.getbits(4)?).saturating_add(3)
                } else {
                    usize::from(br.getbits(cbit_u32)?).saturating_add(20)
                };
                while count > 0 {
                    if let Some(l) = c_len.get_mut(i) {
                        *l = 0;
                    }
                    i = i.saturating_add(1);
                    count = count.saturating_sub(1);
                }
            } else {
                if let Some(l) = c_len.get_mut(i) {
                    *l = u8::try_from(c.saturating_sub(2))?;
                }
                i = i.saturating_add(1);
            }
        }
        while i < NC {
            if let Some(l) = c_len.get_mut(i) {
                *l = 0;
            }
            i = i.saturating_add(1);
        }
        make_table(NC, c_len, 12, c_table, left, right)?;
    }
    Ok(())
}

fn decode_c<R: Read>(
    br: &mut BitReader<R>,
    blocksize: &mut u16,
    c_len: &mut [u8],
    c_table: &mut [u16],
    pt_len: &mut [u8],
    pt_table: &mut [u16],
    left: &mut [u16],
    right: &mut [u16],
) -> Result<usize> {
    if *blocksize == 0 {
        *blocksize = br.getbits(16)?;
        if *blocksize == 0 {
            return Ok(NC); // EOF
        }
        read_pt_len(
            br,
            NT,
            u32::try_from(TBIT)?,
            3,
            pt_len,
            pt_table,
            left,
            right,
        )?;
        read_c_len(br, c_len, c_table, pt_len, pt_table, left, right)?;
        read_pt_len(
            br,
            NP,
            u32::try_from(PBIT)?,
            -1,
            pt_len,
            pt_table,
            left,
            right,
        )?;
    }
    *blocksize = blocksize.saturating_sub(1);

    let peek_idx = br.peekbits12();
    let mut j = usize::from(
        *c_table
            .get(peek_idx)
            .ok_or_else(|| anyhow::anyhow!("c_table index {peek_idx} out of bounds"))?,
    );
    if j >= NC {
        let mut mask = 0x8u16;
        while j >= NC {
            if (br.bitbuf & mask) != 0 {
                j = usize::from(
                    *right
                        .get(j)
                        .ok_or_else(|| anyhow::anyhow!("right tree index {j} out of bounds"))?,
                );
            } else {
                j = usize::from(
                    *left
                        .get(j)
                        .ok_or_else(|| anyhow::anyhow!("left tree index {j} out of bounds"))?,
                );
            }
            mask >>= 1;
        }
    }
    let bits = u32::from(
        *c_len
            .get(j)
            .ok_or_else(|| anyhow::anyhow!("c_len index {j} out of bounds"))?,
    );
    br.dropbits(bits)?;
    Ok(j)
}

fn decode_p<R: Read>(
    br: &mut BitReader<R>,
    pt_len: &[u8],
    pt_table: &[u16],
    left: &[u16],
    right: &[u16],
) -> Result<usize> {
    let peek_idx = br.peekbits8();
    let mut j = usize::from(
        *pt_table
            .get(peek_idx)
            .ok_or_else(|| anyhow::anyhow!("pt_table index {peek_idx} out of bounds"))?,
    );
    if j >= NP {
        let mut mask = 0x80u16;
        while j >= NP {
            if (br.bitbuf & mask) != 0 {
                j = usize::from(
                    *right
                        .get(j)
                        .ok_or_else(|| anyhow::anyhow!("right tree index {j} out of bounds"))?,
                );
            } else {
                j = usize::from(
                    *left
                        .get(j)
                        .ok_or_else(|| anyhow::anyhow!("left tree index {j} out of bounds"))?,
                );
            }
            mask >>= 1;
        }
    }
    let bits = u32::from(
        *pt_len
            .get(j)
            .ok_or_else(|| anyhow::anyhow!("pt_len index {j} out of bounds"))?,
    );
    br.dropbits(bits)?;

    if j != 0 {
        let extra_bits = u32::try_from(j.saturating_sub(1))?;
        let base = 1usize << extra_bits;
        let extra_val = usize::from(br.getbits(extra_bits)?);
        j = base.saturating_add(extra_val);
    }
    Ok(j)
}

/// Decompresses an SCO `compress -H` LZH stream from `reader` into `writer`.
pub fn decompress_stream(
    reader: &mut impl Read,
    writer: &mut impl Write,
) -> Result<u64> {
    let mut magic = [0u8; 2];
    reader
        .read_exact(&mut magic)
        .context("Failed to read SCO compress -H magic header")?;
    if magic != MAGIC_BYTES {
        bail!(
            "Invalid magic header for SCO compress -H stream: {:02X} {:02X}",
            magic[0],
            magic[1]
        );
    }

    let mut br = BitReader::new(reader)?;
    let mut blocksize = 0u16;

    let mut pt_len = vec![0u8; 32];
    let mut pt_table = vec![0u16; 256];
    let mut c_len = vec![0u8; NC];
    let mut c_table = vec![0u16; 4096];
    let mut left = vec![0u16; 2 * NC];
    let mut right = vec![0u16; 2 * NC];

    let mut window = vec![0u8; DICSIZ];
    let mut r = 0usize;
    let mut total_bytes_written = 0u64;

    loop {
        let c = decode_c(
            &mut br,
            &mut blocksize,
            &mut c_len,
            &mut c_table,
            &mut pt_len,
            &mut pt_table,
            &mut left,
            &mut right,
        )?;
        if c == NC {
            break; // EOF
        }

        if c <= 255 {
            let byte = u8::try_from(c)?;
            writer.write_all(&[byte])?;
            if let Some(w_byte) = window.get_mut(r) {
                *w_byte = byte;
            }
            r = (r.saturating_add(1)) & (DICSIZ.saturating_sub(1));
            total_bytes_written = total_bytes_written.saturating_add(1);
        } else {
            let match_len = c.saturating_sub(253);
            let dist = decode_p(&mut br, &pt_len, &pt_table, &left, &right)?;
            let mut src_idx = (r.wrapping_sub(dist).wrapping_sub(1))
                & (DICSIZ.saturating_sub(1));

            for _ in 0..match_len {
                let byte = *window
                    .get(src_idx)
                    .ok_or_else(|| anyhow::anyhow!("window index {src_idx} out of bounds"))?;
                writer.write_all(&[byte])?;
                if let Some(w_byte) = window.get_mut(r) {
                    *w_byte = byte;
                }
                r = (r.saturating_add(1)) & (DICSIZ.saturating_sub(1));
                src_idx =
                    (src_idx.saturating_add(1)) & (DICSIZ.saturating_sub(1));
                total_bytes_written = total_bytes_written.saturating_add(1);
            }
        }
    }

    Ok(total_bytes_written)
}

// Bitstream writer for MSB-first bit packing.
struct BitWriter<'a, W: Write> {
    writer: &'a mut W,
    bitbuf: u32,
    bitcount: u32,
}

impl<'a, W: Write> BitWriter<'a, W> {
    fn new(writer: &'a mut W) -> Self {
        Self {
            writer,
            bitbuf: 0,
            bitcount: 0,
        }
    }

    fn putbits(&mut self, n: u32, val: u32) -> Result<()> {
        if n == 0 {
            return Ok(());
        }
        let mask = (1u32 << n).wrapping_sub(1);
        let clean_val = val & mask;
        self.bitbuf = (self.bitbuf << n) | clean_val;
        self.bitcount = self.bitcount.saturating_add(n);

        while self.bitcount >= 8 {
            let shift = self.bitcount.saturating_sub(8);
            let byte = u8::try_from((self.bitbuf >> shift) & 0xFF)?;
            self.writer.write_all(&[byte])?;
            self.bitcount = shift;
            let keep_mask = (1u32 << shift).wrapping_sub(1);
            self.bitbuf &= keep_mask;
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        if self.bitcount > 0 {
            let shift = 8u32.saturating_sub(self.bitcount);
            let byte = u8::try_from((self.bitbuf << shift) & 0xFF)?;
            self.writer.write_all(&[byte])?;
            self.bitbuf = 0;
            self.bitcount = 0;
        }
        self.writer.flush()?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
enum LzhSymbol {
    Literal(u8),
    Match { length: usize, distance: usize },
}

#[derive(Clone)]
struct PackageItem {
    weight: u64,
    leaves: Vec<usize>,
}

fn build_huffman_lengths(freqs: &[u32], max_bits: u8, bitlen: &mut [u8]) -> Result<()> {
    for len in bitlen.iter_mut() {
        *len = 0;
    }

    let active: Vec<(usize, u32)> = freqs
        .iter()
        .enumerate()
        .filter(|&(_, &f)| f > 0)
        .map(|(i, &f)| (i, f))
        .collect();

    let num_active = active.len();
    if num_active == 0 {
        return Ok(());
    }
    if num_active == 1 {
        if let Some(&(sym, _)) = active.first() {
            if let Some(l) = bitlen.get_mut(sym) {
                *l = 1;
            }
        }
        return Ok(());
    }

    let num_levels = usize::from(max_bits);
    if num_levels == 0 {
        bail!("max_bits must be greater than 0");
    }

    let mut current_level: Vec<PackageItem> = active
        .iter()
        .map(|&(sym, weight)| PackageItem {
            weight: u64::from(weight),
            leaves: vec![sym],
        })
        .collect();

    current_level.sort_by(|a, b| a.weight.cmp(&b.weight));

    for _ in 1..num_levels {
        let mut packages = Vec::new();
        let mut idx = 0usize;
        while idx.saturating_add(1) < current_level.len() {
            let item1 = current_level
                .get(idx)
                .ok_or_else(|| anyhow::anyhow!("Package-Merge level index {idx} out of bounds"))?;
            let next_idx = idx.saturating_add(1);
            let item2 = current_level
                .get(next_idx)
                .ok_or_else(|| anyhow::anyhow!("Package-Merge level index {next_idx} out of bounds"))?;

            let mut leaves = Vec::with_capacity(
                item1.leaves.len().saturating_add(item2.leaves.len()),
            );
            leaves.extend_from_slice(&item1.leaves);
            leaves.extend_from_slice(&item2.leaves);

            packages.push(PackageItem {
                weight: item1.weight.saturating_add(item2.weight),
                leaves,
            });
            idx = idx.saturating_add(2);
        }

        let mut next_level: Vec<PackageItem> = active
            .iter()
            .map(|&(sym, weight)| PackageItem {
                weight: u64::from(weight),
                leaves: vec![sym],
            })
            .collect();
        next_level.extend(packages);
        next_level.sort_by(|a, b| a.weight.cmp(&b.weight));
        current_level = next_level;
    }

    let target_items = num_active.saturating_mul(2).saturating_sub(2);

    if current_level.len() < target_items {
        bail!(
            "Package-Merge: insufficient items ({}) to satisfy target ({target_items}) at max_bits {max_bits}",
            current_level.len()
        );
    }

    for item in current_level.iter().take(target_items) {
        for &sym in &item.leaves {
            if let Some(l) = bitlen.get_mut(sym) {
                *l = (*l).saturating_add(1);
            }
        }
    }

    for &(sym, _) in &active {
        let len = bitlen
            .get(sym)
            .copied()
            .ok_or_else(|| anyhow::anyhow!("bitlen symbol index {sym} out of bounds"))?;
        if len == 0 || len > max_bits {
            bail!("Package-Merge produced invalid length {len} for symbol {sym} (max {max_bits})");
        }
    }

    Ok(())
}

fn build_canonical_codes(bitlen: &[u8], codes: &mut [u16]) -> Result<()> {
    for code in codes.iter_mut() {
        *code = 0;
    }
    let mut count = [0u16; 17];
    let mut start = [0u16; 18];

    for &len in bitlen {
        let l = usize::from(len);
        if l <= 16 {
            if let Some(c) = count.get_mut(l) {
                *c = c.saturating_add(1);
            }
        }
    }

    if let Some(s) = start.get_mut(1) {
        *s = 0;
    }
    for i in 1..=16 {
        let count_i = u32::from(
            *count
                .get(i)
                .ok_or_else(|| anyhow::anyhow!("count index {i} out of bounds"))?,
        );
        let shift = 16u32.saturating_sub(u32::try_from(i)?);
        let prev_start = u32::from(
            *start
                .get(i)
                .ok_or_else(|| anyhow::anyhow!("start index {i} out of bounds"))?,
        );
        let next_start = prev_start.wrapping_add(count_i << shift);
        let next_i = i.saturating_add(1);
        if let Some(s) = start.get_mut(next_i) {
            let s_val = u16::try_from(next_start & 0xFFFF)
                .context("next_start low 16 bits fit in u16")?;
            *s = s_val;
        }
    }

    for (ch, &len) in bitlen.iter().enumerate() {
        let l = usize::from(len);
        if l == 0 {
            continue;
        }
        let curr_start = *start
            .get(l)
            .ok_or_else(|| anyhow::anyhow!("start index {l} out of bounds"))?;
        let code_shift = 16usize.saturating_sub(l);
        let code = curr_start >> code_shift;
        if let Some(c) = codes.get_mut(ch) {
            *c = code;
        }
        let add_shift = 16usize.saturating_sub(l);
        if let Some(s) = start.get_mut(l) {
            *s = s.wrapping_add(1u16 << add_shift);
        }
    }
    Ok(())
}

fn write_pt_len<W: Write>(
    bw: &mut BitWriter<W>,
    pt_len: &mut [u8],
    nn: usize,
    nbit: u32,
    i_special: i32,
) -> Result<()> {
    let mut max_non_zero = 0usize;
    let mut count_non_zero = 0usize;

    for (idx, &l) in pt_len.iter().take(nn).enumerate() {
        if l > 0 {
            max_non_zero = idx.saturating_add(1);
            count_non_zero = count_non_zero.saturating_add(1);
        }
    }

    if count_non_zero == 0 {
        bw.putbits(nbit, 0)?;
        bw.putbits(nbit, 0)?;
        return Ok(());
    }

    if count_non_zero == 1 {
        let single_sym = pt_len
            .iter()
            .take(nn)
            .position(|&l| l > 0)
            .ok_or_else(|| anyhow::anyhow!("count_non_zero == 1 guarantees non-zero symbol exists"))?;
        bw.putbits(nbit, 0)?;
        bw.putbits(nbit, u32::try_from(single_sym)?)?;
        if let Some(l) = pt_len.get_mut(single_sym) {
            *l = 0;
        }
        return Ok(());
    }

    bw.putbits(nbit, u32::try_from(max_non_zero)?)?;

    let mut i = 0usize;
    while i < max_non_zero {
        let c = *pt_len
            .get(i)
            .ok_or_else(|| anyhow::anyhow!("pt_len index {i} out of bounds"))?;
        if c < 7 {
            bw.putbits(3, u32::from(c))?;
        } else {
            bw.putbits(3, 7)?;
            let extra = u32::from(c).saturating_sub(7);
            for _ in 0..extra {
                bw.putbits(1, 1)?;
            }
            bw.putbits(1, 0)?;
        }
        i = i.saturating_add(1);

        if i_special >= 0 && i == usize::try_from(i_special)? {
            let mut zero_count = 0u32;
            while zero_count < 3
                && i < max_non_zero
                && pt_len.get(i).copied() == Some(0)
            {
                zero_count = zero_count.saturating_add(1);
                i = i.saturating_add(1);
            }
            bw.putbits(2, zero_count)?;
        }
    }

    Ok(())
}

#[expect(
    clippy::expect_used,
    clippy::unwrap_in_result,
    reason = "Non-empty block invariants guarantee at least one non-zero frequency symbol"
)]
fn compress_block<W: Write>(
    bw: &mut BitWriter<W>,
    symbols: &[LzhSymbol],
) -> Result<()> {
    let blocksize = u16::try_from(symbols.len())
        .context("Block symbol count exceeds 65535")?;
    if blocksize == 0 {
        return Ok(());
    }

    let mut c_freqs = vec![0u32; NC];
    let mut p_freqs = vec![0u32; NP];

    for &sym in symbols {
        match sym {
            LzhSymbol::Literal(b) => {
                let idx = usize::from(b);
                if let Some(f) = c_freqs.get_mut(idx) {
                    *f = f.saturating_add(1);
                }
            }
            LzhSymbol::Match { length, distance } => {
                let c_sym =
                    length.saturating_sub(THRESHOLD).saturating_add(256);
                if let Some(f) = c_freqs.get_mut(c_sym) {
                    *f = f.saturating_add(1);
                }

                let p_sym = if distance == 0 {
                    0usize
                } else {
                    let d_u32 = u32::try_from(distance)?;
                    let leading = d_u32.leading_zeros();
                    usize::try_from(32u32.saturating_sub(leading))?
                };
                if let Some(f) = p_freqs.get_mut(p_sym) {
                    *f = f.saturating_add(1);
                }
            }
        }
    }

    let count_distinct_c = c_freqs.iter().filter(|&&f| f > 0).count();
    if count_distinct_c == 1 {
        let single_c = c_freqs
            .iter()
            .position(|&f| f > 0)
            .expect("count_distinct_c == 1 guarantees non-zero frequency exists");
        // 1. Write blocksize
        bw.putbits(16, u32::from(blocksize))?;
        // 2. Write empty PT tree
        let tbit_u32 = u32::try_from(TBIT)?;
        bw.putbits(tbit_u32, 0)?;
        bw.putbits(tbit_u32, 0)?;
        // 3. Write single C symbol
        let cbit_u32 = u32::try_from(CBIT)?;
        bw.putbits(cbit_u32, 0)?;
        bw.putbits(cbit_u32, u32::try_from(single_c)?)?;
        // 4. Write empty P tree
        let pbit_u32 = u32::try_from(PBIT)?;
        bw.putbits(pbit_u32, 0)?;
        bw.putbits(pbit_u32, 0)?;
        // 5. Single symbol consumes 0 bits
        return Ok(());
    }

    let mut c_len = vec![0u8; NC];
    build_huffman_lengths(&c_freqs, 16, &mut c_len)?;

    let mut p_len = vec![0u8; NP];
    build_huffman_lengths(&p_freqs, 16, &mut p_len)?;

    enum PtEntry {
        Len(u8),
        ZeroRun4(u32),
        ZeroRunCbit(u32),
    }

    let mut pt_entries = Vec::new();
    let mut pt_freqs = vec![0u32; NT];

    let mut i = 0usize;
    let mut max_c_idx = 0usize;
    for (idx, &l) in c_len.iter().enumerate() {
        if l > 0 {
            max_c_idx = idx.saturating_add(1);
        }
    }

    while i < max_c_idx {
        let l = *c_len
            .get(i)
            .ok_or_else(|| anyhow::anyhow!("c_len index {i} out of bounds"))?;
        if l == 0 {
            let mut run = 0u32;
            while i < max_c_idx && c_len.get(i).copied() == Some(0) {
                run = run.saturating_add(1);
                i = i.saturating_add(1);
            }
            let mut rem = run;
            while rem > 0 {
                if rem >= 20 {
                    let count = rem.min(531);
                    let val = count.saturating_sub(20);
                    pt_entries.push(PtEntry::ZeroRunCbit(val));
                    if let Some(f) = pt_freqs.get_mut(2) {
                        *f = f.saturating_add(1);
                    }
                    rem = rem.saturating_sub(count);
                } else if rem >= 3 {
                    let count = rem.min(18);
                    let val = count.saturating_sub(3);
                    pt_entries.push(PtEntry::ZeroRun4(val));
                    if let Some(f) = pt_freqs.get_mut(1) {
                        *f = f.saturating_add(1);
                    }
                    rem = rem.saturating_sub(count);
                } else {
                    pt_entries.push(PtEntry::Len(0));
                    if let Some(f) = pt_freqs.get_mut(0) {
                        *f = f.saturating_add(1);
                    }
                    rem = rem.saturating_sub(1);
                }
            }
        } else {
            let sym = l.saturating_add(2);
            pt_entries.push(PtEntry::Len(sym));
            let sym_idx = usize::from(sym);
            if let Some(f) = pt_freqs.get_mut(sym_idx) {
                *f = f.saturating_add(1);
            }
            i = i.saturating_add(1);
        }
    }

    let mut pt_len = vec![0u8; NT];
    build_huffman_lengths(&pt_freqs, 7, &mut pt_len)?;

    let mut pt_codes = vec![0u16; NT];
    build_canonical_codes(&pt_len, &mut pt_codes)?;

    let mut c_codes = vec![0u16; NC];
    build_canonical_codes(&c_len, &mut c_codes)?;

    let mut p_codes = vec![0u16; NP];
    build_canonical_codes(&p_len, &mut p_codes)?;

    // 1. Write blocksize
    bw.putbits(16, u32::from(blocksize))?;

    // 2. Write PT tree
    write_pt_len(bw, &mut pt_len, NT, u32::try_from(TBIT)?, 3)?;

    // 3. Write C-tree bit lengths
    let cbit_u32 = u32::try_from(CBIT)?;
    if max_c_idx == 0 {
        let single_c = c_freqs
            .iter()
            .position(|&f| f > 0)
            .expect("non-empty block guarantees non-zero frequency exists");
        bw.putbits(cbit_u32, 0)?;
        bw.putbits(cbit_u32, u32::try_from(single_c)?)?;
    } else {
        bw.putbits(cbit_u32, u32::try_from(max_c_idx)?)?;
        for entry in pt_entries {
            match entry {
                PtEntry::Len(s) => {
                    let sym_idx = usize::from(s);
                    let len = u32::from(
                        *pt_len
                            .get(sym_idx)
                            .ok_or_else(|| anyhow::anyhow!("pt_len index {sym_idx} out of bounds"))?,
                    );
                    let code = u32::from(
                        *pt_codes
                            .get(sym_idx)
                            .ok_or_else(|| anyhow::anyhow!("pt_codes index {sym_idx} out of bounds"))?,
                    );
                    bw.putbits(len, code)?;
                }
                PtEntry::ZeroRun4(val) => {
                    let len = u32::from(
                        *pt_len
                            .get(1)
                            .ok_or_else(|| anyhow::anyhow!("pt_len index 1 out of bounds"))?,
                    );
                    let code = u32::from(
                        *pt_codes
                            .get(1)
                            .ok_or_else(|| anyhow::anyhow!("pt_codes index 1 out of bounds"))?,
                    );
                    bw.putbits(len, code)?;
                    bw.putbits(4, val)?;
                }
                PtEntry::ZeroRunCbit(val) => {
                    let len = u32::from(
                        *pt_len
                            .get(2)
                            .ok_or_else(|| anyhow::anyhow!("pt_len index 2 out of bounds"))?,
                    );
                    let code = u32::from(
                        *pt_codes
                            .get(2)
                            .ok_or_else(|| anyhow::anyhow!("pt_codes index 2 out of bounds"))?,
                    );
                    bw.putbits(len, code)?;
                    bw.putbits(cbit_u32, val)?;
                }
            }
        }
    }

    // 4. Write P-tree
    write_pt_len(bw, &mut p_len, NP, u32::try_from(PBIT)?, -1)?;

    // 5. Write symbols
    for &sym in symbols {
        match sym {
            LzhSymbol::Literal(b) => {
                let idx = usize::from(b);
                let len = u32::from(
                    *c_len
                        .get(idx)
                        .ok_or_else(|| anyhow::anyhow!("c_len index {idx} out of bounds"))?,
                );
                let code = u32::from(
                    *c_codes
                        .get(idx)
                        .ok_or_else(|| anyhow::anyhow!("c_codes index {idx} out of bounds"))?,
                );
                bw.putbits(len, code)?;
            }
            LzhSymbol::Match { length, distance } => {
                let c_sym =
                    length.saturating_sub(THRESHOLD).saturating_add(256);
                let c_len_bits = u32::from(
                    *c_len
                        .get(c_sym)
                        .ok_or_else(|| anyhow::anyhow!("c_len index {c_sym} out of bounds"))?,
                );
                let c_code = u32::from(
                    *c_codes
                        .get(c_sym)
                        .ok_or_else(|| anyhow::anyhow!("c_codes index {c_sym} out of bounds"))?,
                );
                bw.putbits(c_len_bits, c_code)?;

                let (p_sym, extra_bits, extra_val) = if distance == 0 {
                    (0usize, 0u32, 0u32)
                } else {
                    let d_u32 = u32::try_from(distance)?;
                    let leading = d_u32.leading_zeros();
                    let p = usize::try_from(32u32.saturating_sub(leading))?;
                    let eb = u32::try_from(p.saturating_sub(1))?;
                    let ev = d_u32.saturating_sub(1u32 << eb);
                    (p, eb, ev)
                };

                let p_len_bits = u32::from(
                    *p_len
                        .get(p_sym)
                        .ok_or_else(|| anyhow::anyhow!("p_len index {p_sym} out of bounds"))?,
                );
                let p_code = u32::from(
                    *p_codes
                        .get(p_sym)
                        .ok_or_else(|| anyhow::anyhow!("p_codes index {p_sym} out of bounds"))?,
                );
                bw.putbits(p_len_bits, p_code)?;

                if extra_bits > 0 {
                    bw.putbits(extra_bits, extra_val)?;
                }
            }
        }
    }

    Ok(())
}

/// Compresses an input stream to an SCO `compress -H` LZH stream into `writer`.
pub fn compress_stream(
    reader: &mut impl Read,
    writer: &mut impl Write,
) -> Result<u64> {
    let mut input_data = Vec::new();
    reader.read_to_end(&mut input_data)?;
    let total_uncompressed_bytes = u64::try_from(input_data.len())?;

    writer
        .write_all(&MAGIC_BYTES)
        .context("Failed to write SCO compress magic header")?;

    let mut bw = BitWriter::new(writer);
    let mut symbols = Vec::new();

    let len = input_data.len();
    let mut pos = 0usize;

    let mut head = vec![None::<usize>; 65536];
    let mut prev = vec![None::<usize>; DICSIZ];

    while pos < len {
        let mut best_len = 0usize;
        let mut best_dist = 0usize;

        if pos.saturating_add(THRESHOLD) <= len {
            let b0 = usize::from(
                *input_data
                    .get(pos)
                    .ok_or_else(|| anyhow::anyhow!("input_data index {pos} out of bounds"))?,
            );
            let b1 = usize::from(
                *input_data
                    .get(pos.saturating_add(1))
                    .ok_or_else(|| anyhow::anyhow!("input_data index out of bounds"))?,
            );
            let b2 = usize::from(
                *input_data
                    .get(pos.saturating_add(2))
                    .ok_or_else(|| anyhow::anyhow!("input_data index out of bounds"))?,
            );
            let h = (b0 << 8) ^ (b1 << 4) ^ b2;

            let mut m_pos = head.get(h).copied().flatten();
            let limit = pos.saturating_sub(DICSIZ);

            let mut chain = 0usize;
            while let Some(match_idx) = m_pos {
                if match_idx < limit || chain >= 128 {
                    break;
                }
                chain = chain.saturating_add(1);

                let mut match_len = 0usize;
                let max_possible = (len.saturating_sub(pos)).min(MAXMATCH);

                while match_len < max_possible
                    && input_data.get(pos.saturating_add(match_len))
                        == input_data.get(match_idx.saturating_add(match_len))
                {
                    match_len = match_len.saturating_add(1);
                }

                if match_len > best_len {
                    best_len = match_len;
                    best_dist = pos.saturating_sub(match_idx).saturating_sub(1);
                    if best_len == max_possible {
                        break;
                    }
                }

                m_pos = prev
                    .get(match_idx & (DICSIZ.saturating_sub(1)))
                    .copied()
                    .flatten();
            }

            if let Some(p_slot) = prev.get_mut(pos & (DICSIZ.saturating_sub(1)))
            {
                *p_slot = head.get(h).copied().flatten();
            }
            if let Some(h_slot) = head.get_mut(h) {
                *h_slot = Some(pos);
            }
        }

        if best_len >= THRESHOLD {
            symbols.push(LzhSymbol::Match {
                length: best_len,
                distance: best_dist,
            });

            for offset in 1..best_len {
                let p = pos.saturating_add(offset);
                if p.saturating_add(THRESHOLD) <= len {
                    let b0 = usize::from(
                        *input_data
                            .get(p)
                            .ok_or_else(|| anyhow::anyhow!("input_data index {p} out of bounds"))?,
                    );
                    let b1 = usize::from(
                        *input_data
                            .get(p.saturating_add(1))
                            .ok_or_else(|| anyhow::anyhow!("input_data index out of bounds"))?,
                    );
                    let b2 = usize::from(
                        *input_data
                            .get(p.saturating_add(2))
                            .ok_or_else(|| anyhow::anyhow!("input_data index out of bounds"))?,
                    );
                    let h = (b0 << 8) ^ (b1 << 4) ^ b2;

                    if let Some(p_slot) =
                        prev.get_mut(p & (DICSIZ.saturating_sub(1)))
                    {
                        *p_slot = head.get(h).copied().flatten();
                    }
                    if let Some(h_slot) = head.get_mut(h) {
                        *h_slot = Some(p);
                    }
                }
            }

            pos = pos.saturating_add(best_len);
        } else {
            let b = *input_data
                .get(pos)
                .ok_or_else(|| anyhow::anyhow!("input_data index {pos} out of bounds"))?;
            symbols.push(LzhSymbol::Literal(b));
            pos = pos.saturating_add(1);
        }

        if symbols.len() >= 16384 {
            compress_block(&mut bw, &symbols)?;
            symbols.clear();
        }
    }

    if !symbols.is_empty() || len == 0 {
        compress_block(&mut bw, &symbols)?;
    }

    bw.putbits(16, 0)?;
    bw.flush()?;

    Ok(total_uncompressed_bytes)
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
    fn test_sco_compress_empty() {
        let raw = b"";
        let mut compressed = Vec::new();
        compress_stream(&mut &raw[..], &mut compressed).unwrap();

        assert_eq!(&compressed[0..2], &MAGIC_BYTES);

        let mut decompressed = Vec::new();
        decompress_stream(&mut &compressed[..], &mut decompressed).unwrap();
        assert_eq!(decompressed, raw);
    }

    #[crate::ctb_test]
    fn test_sco_compress_small_string() {
        let raw = b"Hello SCO compress -H format!";
        let mut compressed = Vec::new();
        compress_stream(&mut &raw[..], &mut compressed).unwrap();

        let mut decompressed = Vec::new();
        decompress_stream(&mut &compressed[..], &mut decompressed).unwrap();
        assert_eq!(decompressed, raw);
    }

    #[crate::ctb_test]
    fn test_sco_compress_repetitive_data() {
        let raw = vec![b'A'; 5000];
        let mut compressed = Vec::new();
        compress_stream(&mut &raw[..], &mut compressed).unwrap();

        assert!(
            compressed.len() < raw.len(),
            "Repetitive data should compress well"
        );

        let mut decompressed = Vec::new();
        decompress_stream(&mut &compressed[..], &mut decompressed).unwrap();
        assert_eq!(
            decompressed.len(),
            raw.len(),
            "Decompressed length {} != raw length {}",
            decompressed.len(),
            raw.len()
        );
        assert_eq!(decompressed, raw);
    }

    #[crate::ctb_test]
    fn test_package_merge_kraft_equality() {
        // Test various frequency distributions to ensure Kraft equality is strictly preserved
        let mut freqs = vec![0u32; 510];
        for (i, f) in freqs.iter_mut().enumerate() {
            *f = u32::try_from(i.saturating_mul(13).saturating_add(7)).unwrap_or(1);
        }

        let mut bitlen = vec![0u8; 510];
        build_huffman_lengths(&freqs, 16, &mut bitlen).unwrap();

        let mut kraft_sum = 0u32;
        for &len in &bitlen {
            if len > 0 {
                assert!(len <= 16, "Length {len} exceeded max_bits 16");
                let shift = 16u32.saturating_sub(u32::from(len));
                kraft_sum = kraft_sum.saturating_add(1u32 << shift);
            }
        }
        assert_eq!(kraft_sum, 65536, "Kraft equality violated: sum * 2^16 = {kraft_sum} != 65536");
    }
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
