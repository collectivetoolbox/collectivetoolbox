// SPDX-License-Identifier: AGPL-3.0-or-later

//! The original `bzip` 0.21 compression format (Julian Seward, 1996).
//!
//! Specification reference: `data/docs/bzip.md`

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;
use anyhow::anyhow;
use std::io::{Read, Write};

/// Preamble magic bytes for `bzip` 0.21 (`"BZ0"`).
pub const BZIP_MAGIC: [u8; 3] = [0x42, 0x5A, 0x30];

/// Default block size limit multiplier (9 = 900,000 bytes).
pub const DEFAULT_BLOCK_SIZE_100K: u8 = 9;

/// Stream sentinel byte appended to the final block's RLE1 data (`'*'`).
pub const SENTINEL_BYTE: u8 = 0x2A;

// Primary BASIS model token values (1-based indices)
const VAL_RUNA: usize = 1;
const VAL_RUNB: usize = 2;
const VAL_ONE: usize = 3;
const VAL_2_3: usize = 4;
const VAL_4_7: usize = 5;
const VAL_8_15: usize = 6;
const VAL_16_31: usize = 7;
const VAL_32_63: usize = 8;
const VAL_64_127: usize = 9;
const VAL_128_255: usize = 10;
const VAL_EOB: usize = 11;

/// Precomputed IEEE 802.3 left-shifting CRC-32 table using polynomial `0x04C11DB7`.
const CRC_TABLE: [u32; 256] = {
    let mut table = [0u32; 256];
    let mut i: u32 = 0;
    while i < 256 {
        let mut crc = i << 24;
        let mut j = 0;
        while j < 8 {
            if (crc & 0x8000_0000) != 0 {
                crc = (crc << 1) ^ 0x04C1_1DB7;
            } else {
                crc <<= 1;
            }
            j += 1;
        }
        table[i as usize] = crc;
        i += 1;
    }
    table
};

/// 32-bit CRC accumulator for `bzip` stream verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BzipCrc(u32);

impl Default for BzipCrc {
    fn default() -> Self {
        Self::new()
    }
}

impl BzipCrc {
    /// Creates a new CRC accumulator initialized to `0xFFFFFFFF`.
    pub const fn new() -> Self {
        Self(0xFFFF_FFFF)
    }

    /// Updates the CRC state with a single uncompressed byte.
    pub fn update_byte(&mut self, byte: u8) {
        // Reason for fallback: masked with 0xFF, guaranteed to fit in u8.
        let shift_val = u8::try_from((self.0 >> 24) & 0xFF).unwrap_or(0);
        let idx = usize::from(shift_val ^ byte);
        // Reason for fallback: idx is bounded by 0..256 (u8 XOR u8), within CRC_TABLE bounds.
        let entry = CRC_TABLE.get(idx).copied().unwrap_or(0);
        self.0 = (self.0 << 8) ^ entry;
    }

    /// Updates the CRC state with a byte slice.
    pub fn update(&mut self, data: &[u8]) {
        for &b in data {
            self.update_byte(b);
        }
    }

    /// Returns the final 32-bit CRC checksum (`~CRC`).
    pub const fn finish(self) -> u32 {
        !self.0
    }
}

/// Adaptive and static discrete probability distribution model for DCC95 arithmetic coding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Model {
    /// Alphabet cardinality.
    pub num_symbols: usize,
    /// Discrete frequency array (1-indexed, length `num_symbols + 1`).
    pub freq: Vec<u32>,
    /// Total cumulative frequency sum.
    pub tot_freq: u32,
    /// Increment count added on each symbol occurrence (0 for static uniform model).
    pub inc_value: u32,
    /// Maximum permitted total frequency before halving.
    pub no_exceed: u32,
}

impl Model {
    /// Initializes a probability model.
    pub fn new(num_symbols: usize, inc_value: u32, no_exceed: u32) -> Result<Self> {
        let mut freq = vec![0u32; num_symbols.saturating_add(1)];
        let tot_freq = if inc_value == 0 {
            for slot in freq.iter_mut().skip(1) {
                *slot = 1;
            }
            u32::try_from(num_symbols)
                .map_err(|e| anyhow!("num_symbols conversion overflow: {e}"))?
        } else {
            for slot in freq.iter_mut().skip(1) {
                *slot = inc_value;
            }
            let n_u32 = u32::try_from(num_symbols)
                .map_err(|e| anyhow!("num_symbols conversion overflow: {e}"))?;
            n_u32
                .checked_mul(inc_value)
                .ok_or_else(|| anyhow!("Initial total frequency overflow"))?
        };
        Ok(Self {
            num_symbols,
            freq,
            tot_freq,
            inc_value,
            no_exceed,
        })
    }

    /// Creates a static uniform 256-symbol model (`bogusModel`) for raw byte I/O.
    pub fn new_bogus() -> Self {
        // Infallible for num_symbols = 256, inc_value = 0, no_exceed = 256
        Self {
            num_symbols: 256,
            freq: {
                let mut f = vec![1u32; 257];
                if let Some(first) = f.first_mut() {
                    *first = 0;
                }
                f
            },
            tot_freq: 256,
            inc_value: 0,
            no_exceed: 256,
        }
    }

    /// Updates frequency distribution after encoding or decoding symbol `s`.
    pub fn update(&mut self, symbol: usize) {
        if symbol == 0 || symbol > self.num_symbols || self.inc_value == 0 {
            return;
        }
        self.tot_freq = self.tot_freq.saturating_add(self.inc_value);
        if let Some(slot) = self.freq.get_mut(symbol) {
            *slot = slot.saturating_add(self.inc_value);
        }
        if self.tot_freq > self.no_exceed {
            let mut new_tot = 0u32;
            for slot in self.freq.iter_mut().skip(1) {
                *slot = (slot.saturating_add(1)) >> 1;
                new_tot = new_tot.saturating_add(*slot);
            }
            self.tot_freq = new_tot;
        }
    }
}

/// Structured Move-to-Front probability model suite (Fenwick hierarchy).
#[derive(Clone, Debug)]
pub struct MtfModelSuite {
    /// Basis model (11 tokens).
    pub basis: Model,
    /// Ranks 2..3 model (2 tokens).
    pub m2_3: Model,
    /// Ranks 4..7 model (4 tokens).
    pub m4_7: Model,
    /// Ranks 8..15 model (8 tokens).
    pub m8_15: Model,
    /// Ranks 16..31 model (16 tokens).
    pub m16_31: Model,
    /// Ranks 32..63 model (32 tokens).
    pub m32_63: Model,
    /// Ranks 64..127 model (64 tokens).
    pub m64_127: Model,
    /// Ranks 128..255 model (128 tokens).
    pub m128_255: Model,
}

impl MtfModelSuite {
    /// Initializes all 8 structured MTF models to their default starting states.
    pub fn new() -> Result<Self> {
        Ok(Self {
            basis: Model::new(11, 12, 1000)?,
            m2_3: Model::new(2, 4, 1000)?,
            m4_7: Model::new(4, 3, 1000)?,
            m8_15: Model::new(8, 3, 1000)?,
            m16_31: Model::new(16, 3, 1000)?,
            m32_63: Model::new(32, 3, 1000)?,
            m64_127: Model::new(64, 2, 1000)?,
            m128_255: Model::new(128, 1, 1000)?,
        })
    }
}

/// Bit-level stream reader packing MSB first.
pub struct BitReader<R: Read> {
    reader: R,
    current_byte: u8,
    bits_left: u8,
}

impl<R: Read> BitReader<R> {
    /// Creates a new bit reader.
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            current_byte: 0,
            bits_left: 0,
        }
    }

    /// Reads a single bit from the stream (MSB first). Returns 0 on stream EOF.
    pub fn read_bit(&mut self) -> Result<u32> {
        if self.bits_left == 0 {
            let mut buf = [0u8; 1];
            let n = self.reader.read(&mut buf).context("Failed to read from bitstream")?;
            if n == 0 {
                return Ok(0); // Zero-padding on stream termination
            }
            if let Some(&b) = buf.first() {
                self.current_byte = b;
            }
            self.bits_left = 8;
        }
        let shift = self.bits_left.saturating_sub(1);
        let bit = (u32::from(self.current_byte) >> shift) & 1;
        self.bits_left = shift;
        Ok(bit)
    }
}

/// Bit-level stream writer packing MSB first.
pub struct BitWriter<W: Write> {
    writer: W,
    current_byte: u8,
    bits_in_buf: u8,
    bytes_written: u64,
}

impl<W: Write> BitWriter<W> {
    /// Creates a new bit writer.
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            current_byte: 0,
            bits_in_buf: 0,
            bytes_written: 0,
        }
    }

    /// Writes a single bit into the stream.
    pub fn write_bit(&mut self, bit: u32) -> Result<()> {
        // Reason for fallback: bit & 1 is 0 or 1, guaranteed to fit in u8.
        let bit_u8 = u8::try_from(bit & 1).unwrap_or(0);
        self.current_byte = (self.current_byte << 1) | bit_u8;
        self.bits_in_buf = self.bits_in_buf.saturating_add(1);
        if self.bits_in_buf == 8 {
            self.writer
                .write_all(&[self.current_byte])
                .context("Failed to write byte to bitstream")?;
            self.bytes_written = self.bytes_written.saturating_add(1);
            self.current_byte = 0;
            self.bits_in_buf = 0;
        }
        Ok(())
    }

    /// Flushes any remaining unwritten bits padded with zeros.
    pub fn flush(&mut self) -> Result<()> {
        if self.bits_in_buf > 0 {
            let shift = 8u8.saturating_sub(self.bits_in_buf);
            let padded = self.current_byte << shift;
            self.writer
                .write_all(&[padded])
                .context("Failed to flush trailing bits to bitstream")?;
            self.bytes_written = self.bytes_written.saturating_add(1);
            self.current_byte = 0;
            self.bits_in_buf = 0;
        }
        self.writer.flush().context("Failed to flush underlying writer")?;
        Ok(())
    }
}

/// Moffat-Neal-Witten (DCC95) 26-bit finite precision arithmetic encoder.
pub struct ArithEncoder<W: Write> {
    bit_writer: BitWriter<W>,
    low: u32,
    range: u32,
    bits_outstanding: u32,
}

impl<W: Write> ArithEncoder<W> {
    /// Initializes an arithmetic encoder.
    pub fn new(writer: W) -> Self {
        Self {
            bit_writer: BitWriter::new(writer),
            low: 0,
            range: 33_554_432, // 2^25
            bits_outstanding: 0,
        }
    }

    /// Encodes a 1-based symbol index using the provided probability model.
    pub fn encode_symbol(&mut self, model: &mut Model, symbol: usize) -> Result<()> {
        if symbol == 0 || symbol > model.num_symbols {
            bail!("Symbol index {symbol} out of range for model of size {}", model.num_symbols);
        }

        let (l_s, h_s) = if model.inc_value == 0 {
            // Reason for fallback: symbol is bounded by num_symbols <= 256.
            let l = u32::try_from(symbol.saturating_sub(1)).unwrap_or(0);
            (l, l.saturating_add(1))
        } else {
            let mut l = 0u32;
            for i in 1..symbol {
                // Reason for fallback: i is within 1..symbol <= num_symbols bound of model.freq.
                let f = model.freq.get(i).copied().unwrap_or(0);
                l = l.saturating_add(f);
            }
            // Reason for fallback: symbol is within 1..=num_symbols bound of model.freq.
            let freq_s = model.freq.get(symbol).copied().unwrap_or(0);
            (l, l.saturating_add(freq_s))
        };
        let t = model.tot_freq;
        if t == 0 {
            bail!("Arithmetic encoder total frequency is zero");
        }

        // Reason for fallback: t is checked non-zero above.
        let r = self.range.checked_div(t).unwrap_or(0);
        if r == 0 {
            bail!("Arithmetic encoder range underflow");
        }

        self.low = self.low.saturating_add(r.saturating_mul(l_s));
        if h_s < t {
            self.range = r.saturating_mul(h_s.saturating_sub(l_s));
        } else {
            self.range = self.range.saturating_sub(r.saturating_mul(l_s));
        }

        while self.range <= 16_777_216 {
            if self.low.saturating_add(self.range) <= 33_554_432 {
                self.bit_writer.write_bit(0)?;
                for _ in 0..self.bits_outstanding {
                    self.bit_writer.write_bit(1)?;
                }
                self.bits_outstanding = 0;
            } else if self.low >= 33_554_432 {
                self.bit_writer.write_bit(1)?;
                for _ in 0..self.bits_outstanding {
                    self.bit_writer.write_bit(0)?;
                }
                self.bits_outstanding = 0;
                self.low = self.low.saturating_sub(33_554_432);
            } else {
                self.bits_outstanding = self.bits_outstanding.saturating_add(1);
                self.low = self.low.saturating_sub(16_777_216);
            }
            self.low = self.low.saturating_mul(2);
            self.range = self.range.saturating_mul(2);
        }

        model.update(symbol);
        Ok(())
    }

    /// Encodes a raw 8-bit byte via `bogusModel`.
    pub fn encode_byte(&mut self, byte: u8, bogus_model: &mut Model) -> Result<()> {
        let sym = usize::from(byte).saturating_add(1);
        self.encode_symbol(bogus_model, sym)
    }

    /// Encodes a 32-bit integer as 4 big-endian bytes via `bogusModel`.
    pub fn encode_u32(&mut self, val: u32, bogus_model: &mut Model) -> Result<()> {
        for b in val.to_be_bytes() {
            self.encode_byte(b, bogus_model)?;
        }
        Ok(())
    }

    /// Terminates the arithmetic encoder and flushes the bitstream.
    pub fn finish(mut self) -> Result<u64> {
        for i in (1u32..=26u32).rev() {
            let shift = i.saturating_sub(1);
            let bit = (self.low >> shift) & 1;
            self.bit_writer.write_bit(bit)?;
            let opp_bit = 1u32.saturating_sub(bit);
            for _ in 0..self.bits_outstanding {
                self.bit_writer.write_bit(opp_bit)?;
            }
            self.bits_outstanding = 0;
        }
        self.bit_writer.flush()?;
        Ok(self.bit_writer.bytes_written)
    }
}

/// Moffat-Neal-Witten (DCC95) 26-bit finite precision arithmetic decoder.
pub struct ArithDecoder<R: Read> {
    bit_reader: BitReader<R>,
    range: u32,
    code: u32,
}

impl<R: Read> ArithDecoder<R> {
    /// Initializes an arithmetic decoder by reading the initial 26 bits into register `D`.
    pub fn new(mut bit_reader: BitReader<R>) -> Result<Self> {
        let mut code = 0u32;
        for _ in 0..26 {
            let bit = bit_reader.read_bit()?;
            code = (code << 1) | (bit & 1);
        }
        Ok(Self {
            bit_reader,
            range: 33_554_432, // 2^25
            code,
        })
    }

    /// Decodes a 1-based symbol index using the provided probability model.
    pub fn decode_symbol(&mut self, model: &mut Model) -> Result<usize> {
        let t = model.tot_freq;
        if t == 0 {
            bail!("Arithmetic decoder total frequency is zero");
        }

        // Reason for fallback: t is checked non-zero above.
        let r = self.range.checked_div(t).unwrap_or(0);
        if r == 0 {
            bail!("Arithmetic decoder range underflow");
        }

        // Reason for fallback: r is checked non-zero above.
        let target = (self.code.checked_div(r).unwrap_or(0)).min(t.saturating_sub(1));

        let (symbol, l_s, h_s) = if model.inc_value == 0 {
            // Reason for fallback: target is bounded by t <= 256, fits in usize.
            let sym = usize::try_from(target).unwrap_or(0).saturating_add(1);
            let l = target;
            let h = target.saturating_add(1);
            (sym, l, h)
        } else {
            let mut cum = 0u32;
            let mut sym = 1usize;
            let mut l = 0u32;
            let mut h = 0u32;

            for i in 1..=model.num_symbols {
                // Reason for fallback: i is within 1..=num_symbols bound of model.freq.
                let f = model.freq.get(i).copied().unwrap_or(0);
                let next_cum = cum.saturating_add(f);
                if target < next_cum {
                    sym = i;
                    l = cum;
                    h = next_cum;
                    break;
                }
                cum = next_cum;
            }
            (sym, l, h)
        };

        self.code = self.code.saturating_sub(r.saturating_mul(l_s));
        if h_s < t {
            self.range = r.saturating_mul(h_s.saturating_sub(l_s));
        } else {
            self.range = self.range.saturating_sub(r.saturating_mul(l_s));
        }

        while self.range <= 16_777_216 {
            self.range = self.range.saturating_mul(2);
            let bit = self.bit_reader.read_bit()?;
            self.code = (self.code.saturating_mul(2)).saturating_add(bit & 1);
        }

        model.update(symbol);
        Ok(symbol)
    }

    /// Decodes a raw 8-bit byte via `bogusModel`.
    pub fn decode_byte(&mut self, bogus_model: &mut Model) -> Result<u8> {
        let sym = self.decode_symbol(bogus_model)?;
        u8::try_from(sym.saturating_sub(1)).context("Byte decoding conversion error")
    }

    /// Decodes a 32-bit integer as 4 big-endian bytes via `bogusModel`.
    pub fn decode_u32(&mut self, bogus_model: &mut Model) -> Result<u32> {
        let b0 = self.decode_byte(bogus_model)?;
        let b1 = self.decode_byte(bogus_model)?;
        let b2 = self.decode_byte(bogus_model)?;
        let b3 = self.decode_byte(bogus_model)?;
        Ok(u32::from_be_bytes([b0, b1, b2, b3]))
    }
}

/// Applies forward Burrows-Wheeler transform on a byte slice via cyclic prefix-doubling sort.
pub fn forward_bwt(block: &[u8]) -> Result<(Vec<u8>, usize)> {
    let n = block.len();
    if n == 0 {
        return Ok((Vec::new(), 0));
    }
    if n == 1 {
        // Reason for fallback: block is non-empty (len == 1 verified above).
        let b = block.first().copied().unwrap_or(0);
        return Ok((vec![b], 0));
    }

    let mut sa: Vec<usize> = (0..n).collect();
    let mut rank: Vec<u32> = block.iter().map(|&b| u32::from(b)).collect();
    let mut keys = vec![0u64; n];
    let mut new_rank = vec![0u32; n];

    let mut k = 1usize;
    while k < n {
        for i in 0..n {
            // Reason for fallback: i is within 0..n bounds of rank vector.
            let r1 = u64::from(rank.get(i).copied().unwrap_or(0));
            // Reason for fallback: n >= 2 is non-zero.
            let next_i = (i.saturating_add(k)).checked_rem(n).unwrap_or(0);
            // Reason for fallback: next_i is within 0..n bounds of rank vector.
            let r2 = u64::from(rank.get(next_i).copied().unwrap_or(0));
            if let Some(slot) = keys.get_mut(i) {
                *slot = (r1 << 32) | r2;
            }
        }

        // Reason for fallback: idx is an element of sa (0..n), within bounds of keys vector.
        sa.sort_unstable_by_key(|&idx| keys.get(idx).copied().unwrap_or(0));

        let mut cur_rank = 0u32;
        if let Some(&first_idx) = sa.first() {
            if let Some(slot) = new_rank.get_mut(first_idx) {
                *slot = 0;
            }
        }
        for w in sa.windows(2) {
            let prev = w[0];
            let curr = w[1];
            // Reason for fallback: prev is within bounds of keys vector.
            let prev_key = keys.get(prev).copied().unwrap_or(0);
            // Reason for fallback: curr is within bounds of keys vector.
            let curr_key = keys.get(curr).copied().unwrap_or(0);

            if prev_key != curr_key {
                cur_rank = cur_rank.saturating_add(1);
            }
            if let Some(slot) = new_rank.get_mut(curr) {
                *slot = cur_rank;
            }
        }
        rank.copy_from_slice(&new_rank);
        // Reason for fallback: cur_rank <= n, fits in usize.
        if usize::try_from(cur_rank).unwrap_or(0) == n.saturating_sub(1) {
            break;
        }
        k = k.saturating_mul(2);
    }

    let mut l_column = vec![0u8; n];
    let mut orig_ptr = 0usize;
    for (i, &start) in sa.iter().enumerate() {
        if start == 0 {
            orig_ptr = i;
        }
        // Reason for fallback: n >= 2 is non-zero.
        let last_idx = (start.saturating_add(n).saturating_sub(1))
            .checked_rem(n)
            .unwrap_or(0);
        // Reason for fallback: last_idx is within bounds of block.
        let b = block.get(last_idx).copied().unwrap_or(0);
        if let Some(slot) = l_column.get_mut(i) {
            *slot = b;
        }
    }
    Ok((l_column, orig_ptr))
}

/// Inverts the Burrows-Wheeler transform given $L$-column array and `origPtr`.
pub fn inverse_bwt(l: &[u8], orig_ptr: usize) -> Result<Vec<u8>> {
    let n = l.len();
    if n == 0 {
        return Ok(Vec::new());
    }
    if orig_ptr >= n {
        bail!("Inverse BWT origPtr out of bounds: {orig_ptr} >= {n}");
    }

    let mut c = [0usize; 256];
    for &b in l {
        let idx = usize::from(b);
        if let Some(slot) = c.get_mut(idx) {
            *slot = slot.saturating_add(1);
        }
    }

    let mut cc = [0usize; 256];
    let mut sum = 0usize;
    for ch in 0..256 {
        // Reason for fallback: ch is in 0..256, within bounds of c array.
        let count = c.get(ch).copied().unwrap_or(0);
        sum = sum.saturating_add(count);
        if let Some(slot) = cc.get_mut(ch) {
            *slot = sum.saturating_sub(count);
        }
    }

    let mut t = vec![0usize; n];
    for (i, &b) in l.iter().enumerate() {
        let idx = usize::from(b);
        if let Some(pos) = cc.get_mut(idx) {
            if let Some(t_slot) = t.get_mut(i) {
                *t_slot = *pos;
            }
            *pos = pos.saturating_add(1);
        }
    }

    let mut block = vec![0u8; n];
    let mut curr = orig_ptr;
    for j in (0..n).rev() {
        let ch = l
            .get(curr)
            .copied()
            .ok_or_else(|| anyhow!("Index out of bounds in BWT reconstruction"))?;
        if let Some(slot) = block.get_mut(j) {
            *slot = ch;
        }
        curr = t
            .get(curr)
            .copied()
            .ok_or_else(|| anyhow!("Index out of bounds in BWT pointer vector"))?;
    }
    Ok(block)
}

/// Applies forward deterministic block perturbation ("Spotting") for compression.
pub fn apply_spotting_compression(block: &mut [u8]) {
    let n = block.len();
    if n <= 8001 {
        return;
    }
    let mut pos = 8000usize;
    let mut delta = 1i32;
    while pos < n.saturating_sub(1) {
        if let Some(b) = block.get_mut(pos) {
            *b = (*b).wrapping_add(1);
        }
        let newdelta = match delta {
            3 => 1,
            1 => 4,
            4 => 5,
            5 => 9,
            9 => 2,
            2 => 6,
            6 => 7,
            8 => 8,
            7 => 3,
            _ => 1,
        };
        delta = newdelta;
        let step = 8000i32.saturating_add(17i32.saturating_mul(delta.saturating_sub(5)));
        let step_usize = if step <= 0 {
            1usize
        } else {
            // Reason for fallback: step > 0 verified above.
            usize::try_from(step).unwrap_or(1)
        };
        pos = pos.saturating_add(step_usize);
    }
}

/// Inverts deterministic block perturbation ("Spotting") for decompression.
pub fn apply_spotting_decompression(block: &mut [u8]) {
    let n = block.len();
    if n <= 8001 {
        return;
    }
    let mut pos = 8000usize;
    let mut delta = 1i32;
    while pos < n.saturating_sub(1) {
        if let Some(b) = block.get_mut(pos) {
            *b = (*b).wrapping_sub(1);
        }
        let newdelta = match delta {
            3 => 1,
            1 => 4,
            4 => 5,
            5 => 9,
            9 => 2,
            2 => 6,
            6 => 7,
            8 => 8,
            7 => 3,
            _ => 1,
        };
        delta = newdelta;
        let step = 8000i32.saturating_add(17i32.saturating_mul(delta.saturating_sub(5)));
        let step_usize = if step <= 0 {
            1usize
        } else {
            // Reason for fallback: step > 0 verified above.
            usize::try_from(step).unwrap_or(1)
        };
        pos = pos.saturating_add(step_usize);
    }
}

/// Encodes a zero-run length of $K \ge 1$ using Wheeler bijective base-2 tokens (`RUNA`/`RUNB`).
fn encode_zero_run<W: Write>(
    encoder: &mut ArithEncoder<W>,
    models: &mut MtfModelSuite,
    count: usize,
) -> Result<()> {
    if count == 0 {
        return Ok(());
    }
    let mut k = count;
    let mut bits = 0u32;
    let mut num_tokens = 0u32;
    while k != 0 {
        num_tokens = num_tokens.saturating_add(1);
        bits = bits << 1;
        k = k.saturating_sub(1);
        if (k & 1) == 1 {
            bits |= 1;
        }
        k >>= 1;
    }
    while num_tokens > 0 {
        if (bits & 1) == 1 {
            encoder.encode_symbol(&mut models.basis, VAL_RUNA)?;
        } else {
            encoder.encode_symbol(&mut models.basis, VAL_RUNB)?;
        }
        bits >>= 1;
        num_tokens = num_tokens.saturating_sub(1);
    }
    Ok(())
}

/// Encodes an MTF transformed block into the arithmetic bitstream.
pub fn encode_mtf_block<W: Write>(
    encoder: &mut ArithEncoder<W>,
    models: &mut MtfModelSuite,
    l_column: &[u8],
) -> Result<()> {
    let mut yy = [0u8; 256];
    for (i, slot) in yy.iter_mut().enumerate() {
        // Reason for fallback: i is bounded by 0..256, fits in u8.
        *slot = u8::try_from(i).unwrap_or(0);
    }

    let mut zero_run_count = 0usize;

    for &c in l_column {
        let mut rank = 0usize;
        if yy.first().copied() == Some(c) {
            rank = 0;
        } else {
            for (i, &entry) in yy.iter().enumerate() {
                if entry == c {
                    rank = i;
                    break;
                }
            }
        }

        if rank == 0 {
            zero_run_count = zero_run_count.saturating_add(1);
        } else {
            encode_zero_run(encoder, models, zero_run_count)?;
            zero_run_count = 0;

            match rank {
                1 => {
                    encoder.encode_symbol(&mut models.basis, VAL_ONE)?;
                }
                2..=3 => {
                    encoder.encode_symbol(&mut models.basis, VAL_2_3)?;
                    let sub = rank.saturating_sub(2).saturating_add(1);
                    encoder.encode_symbol(&mut models.m2_3, sub)?;
                }
                4..=7 => {
                    encoder.encode_symbol(&mut models.basis, VAL_4_7)?;
                    let sub = rank.saturating_sub(4).saturating_add(1);
                    encoder.encode_symbol(&mut models.m4_7, sub)?;
                }
                8..=15 => {
                    encoder.encode_symbol(&mut models.basis, VAL_8_15)?;
                    let sub = rank.saturating_sub(8).saturating_add(1);
                    encoder.encode_symbol(&mut models.m8_15, sub)?;
                }
                16..=31 => {
                    encoder.encode_symbol(&mut models.basis, VAL_16_31)?;
                    let sub = rank.saturating_sub(16).saturating_add(1);
                    encoder.encode_symbol(&mut models.m16_31, sub)?;
                }
                32..=63 => {
                    encoder.encode_symbol(&mut models.basis, VAL_32_63)?;
                    let sub = rank.saturating_sub(32).saturating_add(1);
                    encoder.encode_symbol(&mut models.m32_63, sub)?;
                }
                64..=127 => {
                    encoder.encode_symbol(&mut models.basis, VAL_64_127)?;
                    let sub = rank.saturating_sub(64).saturating_add(1);
                    encoder.encode_symbol(&mut models.m64_127, sub)?;
                }
                128..=255 => {
                    encoder.encode_symbol(&mut models.basis, VAL_128_255)?;
                    let sub = rank.saturating_sub(128).saturating_add(1);
                    encoder.encode_symbol(&mut models.m128_255, sub)?;
                }
                _ => bail!("MTF rank {rank} out of bounds"),
            }

            yy.copy_within(0..rank, 1);
            if let Some(first) = yy.first_mut() {
                *first = c;
            }
        }
    }

    encode_zero_run(encoder, models, zero_run_count)?;
    encoder.encode_symbol(&mut models.basis, VAL_EOB)?;
    Ok(())
}

/// Decodes an MTF transformed block from the arithmetic bitstream until `VAL_EOB`.
pub fn decode_mtf_block<R: Read>(
    decoder: &mut ArithDecoder<R>,
    models: &mut MtfModelSuite,
    block_limit: usize,
) -> Result<Vec<u8>> {
    let mut yy = [0u8; 256];
    for (i, slot) in yy.iter_mut().enumerate() {
        // Reason for fallback: i is bounded by 0..256, fits in u8.
        *slot = u8::try_from(i).unwrap_or(0);
    }

    let mut l_column = Vec::new();

    let mut next_sym = decoder.decode_symbol(&mut models.basis)?;
    while next_sym != VAL_EOB {
        if next_sym == VAL_RUNA || next_sym == VAL_RUNB {
            let mut n = 0usize;
            let mut token = next_sym;
            loop {
                n = n.saturating_mul(2);
                if token == VAL_RUNA {
                    n = n | 1;
                }
                n = n.saturating_add(1);

                let peek = decoder.decode_symbol(&mut models.basis)?;
                if peek == VAL_RUNA || peek == VAL_RUNB {
                    token = peek;
                } else {
                    // Reason for fallback: yy is a fixed-size 256-byte array, first byte always exists.
                    let head_byte = yy.first().copied().unwrap_or(0);
                    if l_column.len().saturating_add(n) > block_limit {
                        bail!("Decoded block size exceeded limit {block_limit}");
                    }
                    l_column.resize(l_column.len().saturating_add(n), head_byte);
                    next_sym = peek;
                    break;
                }
            }
            continue;
        }

        let rank = match next_sym {
            VAL_ONE => 1usize,
            VAL_2_3 => {
                let sub = decoder.decode_symbol(&mut models.m2_3)?;
                2usize.saturating_add(sub).saturating_sub(1)
            }
            VAL_4_7 => {
                let sub = decoder.decode_symbol(&mut models.m4_7)?;
                4usize.saturating_add(sub).saturating_sub(1)
            }
            VAL_8_15 => {
                let sub = decoder.decode_symbol(&mut models.m8_15)?;
                8usize.saturating_add(sub).saturating_sub(1)
            }
            VAL_16_31 => {
                let sub = decoder.decode_symbol(&mut models.m16_31)?;
                16usize.saturating_add(sub).saturating_sub(1)
            }
            VAL_32_63 => {
                let sub = decoder.decode_symbol(&mut models.m32_63)?;
                32usize.saturating_add(sub).saturating_sub(1)
            }
            VAL_64_127 => {
                let sub = decoder.decode_symbol(&mut models.m64_127)?;
                64usize.saturating_add(sub).saturating_sub(1)
            }
            VAL_128_255 => {
                let sub = decoder.decode_symbol(&mut models.m128_255)?;
                128usize.saturating_add(sub).saturating_sub(1)
            }
            _ => bail!("Unexpected basis token {next_sym}"),
        };

        if rank >= 256 {
            bail!("Decoded MTF rank {rank} exceeds 255");
        }

        // Reason for fallback: rank is validated < 256 above, within bounds of yy.
        let c = yy.get(rank).copied().unwrap_or(0);
        yy.copy_within(0..rank, 1);
        if let Some(first) = yy.first_mut() {
            *first = c;
        }

        if l_column.len().saturating_add(1) > block_limit {
            bail!("Decoded block size exceeded limit {block_limit}");
        }
        l_column.push(c);

        next_sym = decoder.decode_symbol(&mut models.basis)?;
    }

    Ok(l_column)
}

/// Applies RLE1 folding on raw uncompressed input bytes into RLE1 chunks.
pub fn rle1_encode(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut i = 0usize;
    let n = data.len();
    while i < n {
        // Reason for fallback: i < n verified by loop condition.
        let b = data.get(i).copied().unwrap_or(0);
        let mut run_len = 1usize;
        while i.saturating_add(run_len) < n
            && data.get(i.saturating_add(run_len)).copied() == Some(b)
            && run_len < 255
        {
            run_len = run_len.saturating_add(1);
        }

        if run_len < 4 {
            for _ in 0..run_len {
                out.push(b);
            }
        } else {
            out.push(b);
            out.push(b);
            out.push(b);
            out.push(b);
            // Reason for fallback: run_len is bounded by 4..=255, so run_len - 4 fits in u8.
            let count_byte = u8::try_from(run_len.saturating_sub(4)).unwrap_or(0);
            out.push(count_byte);
        }
        i = i.saturating_add(run_len);
    }
    out
}

/// Inverts RLE1 decoding across un-spotted block data, emitting raw bytes and updating CRC.
pub fn rle1_decode(
    block: &[u8],
    is_last_block: bool,
    crc: &mut BzipCrc,
    output: &mut impl Write,
) -> Result<u64> {
    let n = block.len();
    if is_last_block {
        if n == 0 {
            bail!("Last block is unexpectedly empty");
        }
        // Reason for fallback: n > 0 verified by empty check above.
        let last_byte = block.get(n.saturating_sub(1)).copied().unwrap_or(0);
        if last_byte != SENTINEL_BYTE {
            bail!(
                "Corrupted stream: expected sentinel byte 0x2A ('*'), got 0x{last_byte:02X}"
            );
        }
    }

    let limit = if is_last_block {
        n.saturating_sub(1)
    } else {
        n
    };

    let mut bytes_emitted = 0u64;
    let mut count = 0usize;
    let mut ch_prev: Option<u8> = None;
    let mut i = 0usize;

    while i < limit {
        let ch = block
            .get(i)
            .copied()
            .ok_or_else(|| anyhow!("Index out of bounds in RLE1 decode"))?;
        output.write_all(&[ch]).context("Failed to write decompressed byte")?;
        crc.update_byte(ch);
        bytes_emitted = bytes_emitted.saturating_add(1);

        if ch_prev != Some(ch) {
            count = 1;
            ch_prev = Some(ch);
        } else {
            count = count.saturating_add(1);
            if count == 4 {
                let rep_idx = i.saturating_add(1);
                if rep_idx >= limit {
                    bail!("Truncated RLE1 repetition count in block");
                }
                let rep = usize::from(
                    block
                        .get(rep_idx)
                        .copied()
                        .ok_or_else(|| anyhow!("Missing RLE1 repetition byte"))?,
                );
                i = rep_idx;
                if rep > 0 {
                    let fill = vec![ch; rep];
                    output
                        .write_all(&fill)
                        .context("Failed to write repeated bytes")?;
                    crc.update(&fill);
                    // Reason for fallback: rep is at most 255 (single byte count), fits in u64.
                    bytes_emitted =
                        bytes_emitted.saturating_add(u64::try_from(rep).unwrap_or(0));
                }
                count = 0;
            }
        }
        i = i.saturating_add(1);
    }

    Ok(bytes_emitted)
}

/// Compresses a stream from `reader` into `writer` using the original `bzip` 0.21 format.
pub fn compress_stream(reader: &mut impl Read, writer: &mut impl Write) -> Result<u64> {
    compress_stream_with_block_size(reader, writer, DEFAULT_BLOCK_SIZE_100K)
}

/// Compresses a stream with a configurable block size indicator (1..=9).
pub fn compress_stream_with_block_size(
    reader: &mut impl Read,
    writer: &mut impl Write,
    block_size_100k: u8,
) -> Result<u64> {
    if !(1..=9).contains(&block_size_100k) {
        bail!("Invalid bzip block size indicator: {block_size_100k}, must be 1..=9");
    }

    // Write 4-byte plaintext header: "BZ0" + ('0' + block_size_100k)
    let block_char = b'0'.saturating_add(block_size_100k);
    let header = [BZIP_MAGIC[0], BZIP_MAGIC[1], BZIP_MAGIC[2], block_char];
    writer
        .write_all(&header)
        .context("Failed to write bzip header")?;

    let block_limit = usize::from(block_size_100k).saturating_mul(100_000);
    let allowable_block_size = block_limit.saturating_sub(19);

    let mut uncompressed_data = Vec::new();
    reader
        .read_to_end(&mut uncompressed_data)
        .context("Failed to read uncompressed input data")?;

    let mut global_crc = BzipCrc::new();
    global_crc.update(&uncompressed_data);
    let final_crc = global_crc.finish();

    let mut encoder = ArithEncoder::new(writer);
    let mut bogus_model = Model::new_bogus();

    fn encode_block<W: Write>(
        block: &mut Vec<u8>,
        is_final: bool,
        encoder: &mut ArithEncoder<W>,
        bogus_model: &mut Model,
    ) -> Result<()> {
        if is_final {
            block.push(SENTINEL_BYTE);
        }
        apply_spotting_compression(block);
        let (l_col, orig_ptr) = forward_bwt(block)?;

        let orig_i32 = i32::try_from(orig_ptr).context("origPtr conversion overflow")?;
        let v = if is_final {
            -(orig_i32.saturating_add(1))
        } else {
            orig_i32.saturating_add(1)
        };
        let v_u32 = u32::from_ne_bytes(v.to_ne_bytes());
        encoder.encode_u32(v_u32, bogus_model)?;

        let mut models = MtfModelSuite::new()?;
        encode_mtf_block(encoder, &mut models, &l_col)?;
        Ok(())
    }

    let mut current_block = Vec::new();
    let n = uncompressed_data.len();
    let mut i = 0usize;

    while i < n {
        // Reason for fallback: i < n verified by loop condition.
        let b = uncompressed_data.get(i).copied().unwrap_or(0);
        let mut run_len = 1usize;
        while i.saturating_add(run_len) < n
            && uncompressed_data.get(i.saturating_add(run_len)).copied() == Some(b)
            && run_len < 255
        {
            run_len = run_len.saturating_add(1);
        }

        let run_bytes = if run_len < 4 {
            run_len
        } else {
            5
        };

        if !current_block.is_empty()
            && current_block.len().saturating_add(run_bytes) > allowable_block_size
        {
            encode_block(
                &mut current_block,
                false,
                &mut encoder,
                &mut bogus_model,
            )?;
            current_block.clear();
        }

        if run_len < 4 {
            for _ in 0..run_len {
                current_block.push(b);
            }
        } else {
            current_block.push(b);
            current_block.push(b);
            current_block.push(b);
            current_block.push(b);
            // Reason for fallback: run_len is in 4..=255, fits in u8.
            let count_byte = u8::try_from(run_len.saturating_sub(4)).unwrap_or(0);
            current_block.push(count_byte);
        }

        i = i.saturating_add(run_len);
    }

    encode_block(
        &mut current_block,
        true,
        &mut encoder,
        &mut bogus_model,
    )?;

    // Stream CRC-32 written via bogusModel
    encoder.encode_u32(final_crc, &mut bogus_model)?;
    encoder.finish()?;

    u64::try_from(uncompressed_data.len()).map_err(|e| anyhow!("Data length overflow: {e}"))
}

/// Decompresses a stream from `reader` into `writer` using the original `bzip` 0.21 format.
pub fn decompress_stream(reader: &mut impl Read, writer: &mut impl Write) -> Result<u64> {
    let mut header = [0u8; 4];
    reader
        .read_exact(&mut header)
        .context("Failed to read 4-byte bzip header")?;

    let h0 = match header.first() {
        Some(&b) => b,
        None => 0,
    };
    let h1 = match header.get(1) {
        Some(&b) => b,
        None => 0,
    };
    let h2 = match header.get(2) {
        Some(&b) => b,
        None => 0,
    };
    if h0 != BZIP_MAGIC[0] || h1 != BZIP_MAGIC[1] || h2 != BZIP_MAGIC[2] {
        bail!(
            "Invalid bzip magic header: expected 'BZ0', got {:02X}{:02X}{:02X}",
            h0,
            h1,
            h2
        );
    }

    let block_char = match header.get(3) {
        Some(&b) => b,
        None => 0,
    };
    if !(b'1'..=b'9').contains(&block_char) {
        bail!("Invalid bzip block size indicator: ASCII '{block_char}'");
    }
    let block_size_100k = block_char.saturating_sub(b'0');
    let block_limit = usize::from(block_size_100k).saturating_mul(100_000);

    let bit_reader = BitReader::new(reader);
    let mut decoder = ArithDecoder::new(bit_reader)?;
    let mut bogus_model = Model::new_bogus();
    let mut global_crc = BzipCrc::new();
    let mut total_decompressed = 0u64;

    loop {
        let v_u32 = decoder.decode_u32(&mut bogus_model)?;
        let v = i32::from_ne_bytes(v_u32.to_ne_bytes());
        let is_last_block = v < 0;
        let orig_ptr = usize::try_from(v.unsigned_abs().saturating_sub(1))
            .context("origPtr calculation overflow")?;

        let mut models = MtfModelSuite::new()?;
        let l_column = decode_mtf_block(&mut decoder, &mut models, block_limit)?;
        let n = l_column.len();

        if n == 0 {
            bail!("Decoded empty L-column block in bzip stream");
        }
        if orig_ptr >= n {
            bail!("bzip origPtr {orig_ptr} exceeds block size {n}");
        }

        let mut block = inverse_bwt(&l_column, orig_ptr)?;
        apply_spotting_decompression(&mut block);

        let bytes_out =
            rle1_decode(&block, is_last_block, &mut global_crc, writer)?;
        total_decompressed = total_decompressed.saturating_add(bytes_out);

        if is_last_block {
            break;
        }
    }

    let stored_crc = decoder.decode_u32(&mut bogus_model)?;
    let computed_crc = global_crc.finish();
    if stored_crc != computed_crc {
        bail!(
            "bzip CRC-32 checksum mismatch: stored 0x{stored_crc:08X}, computed 0x{computed_crc:08X}"
        );
    }

    Ok(total_decompressed)
}

/// Compresses a byte slice using the original `bzip` 0.21 format.
pub fn compress(data: &[u8]) -> Result<Vec<u8>> {
    let mut input = data;
    let mut output = Vec::new();
    compress_stream(&mut input, &mut output)?;
    Ok(output)
}

/// Decompresses a byte slice using the original `bzip` 0.21 format.
pub fn decompress(data: &[u8]) -> Result<Vec<u8>> {
    let mut input = data;
    let mut output = Vec::new();
    decompress_stream(&mut input, &mut output)?;
    Ok(output)
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
    fn test_bzip_crc() {
        let mut crc = BzipCrc::new();
        crc.update(b"123456789");
        let result = crc.finish();
        assert_eq!(result, 0xFC89_1918);
    }

    #[crate::ctb_test]
    fn test_rle1_roundtrip() {
        let input = b"AAAAABBBCCCCCCDDDD";
        let rle = rle1_encode(input);
        assert_eq!(
            rle,
            vec![
                b'A', b'A', b'A', b'A', 1, // 5 'A's -> 4 'A's + 1
                b'B', b'B', b'B', // 3 'B's
                b'C', b'C', b'C', b'C', 2, // 6 'C's -> 4 'C's + 2
                b'D', b'D', b'D', b'D', 0, // 4 'D's -> 4 'D's + 0
            ]
        );

        let mut block = rle.clone();
        block.push(SENTINEL_BYTE);
        let mut crc = BzipCrc::new();
        let mut decoded = Vec::new();
        rle1_decode(&block, true, &mut crc, &mut decoded).unwrap();
        assert_eq!(decoded, input);
    }

    #[crate::ctb_test]
    fn test_bwt_roundtrip() {
        let text = b"banana";
        let (l, orig_ptr) = forward_bwt(text).unwrap();
        let reconstructed = inverse_bwt(&l, orig_ptr).unwrap();
        assert_eq!(reconstructed, text);
    }

    #[crate::ctb_test]
    fn test_spotting_roundtrip() {
        let mut data = vec![0u8; 20000];
        for (i, b) in data.iter_mut().enumerate() {
            *b = u8::try_from(i.checked_rem(256).unwrap_or(0)).unwrap_or(0);
        }
        let original = data.clone();
        apply_spotting_compression(&mut data);
        assert_ne!(data, original);
        apply_spotting_decompression(&mut data);
        assert_eq!(data, original);
    }

    #[crate::ctb_test]
    fn test_roundtrip_empty() {
        let data = b"";
        let compressed = compress(data).unwrap();
        let decompressed = decompress(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[crate::ctb_test]
    fn test_roundtrip_short() {
        let data = b"Hello, bzip 0.21 world!";
        let compressed = compress(data).unwrap();
        let decompressed = decompress(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[crate::ctb_test]
    fn test_roundtrip_repetitive() {
        let data = vec![b'A'; 50000];
        let compressed = compress(&data).unwrap();
        let decompressed = decompress(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[crate::ctb_test]
    fn test_roundtrip_multiblock() {
        // Test with block size 1 (100,000 bytes limit) and 250,000 bytes input
        let mut data = Vec::new();
        for i in 0usize..250_000usize {
            data.push(
                u8::try_from(
                    (i.saturating_mul(37).saturating_add(13))
                        .checked_rem(256)
                        .unwrap_or(0),
                )
                .unwrap_or(0),
            );
        }
        let mut compressed = Vec::new();
        compress_stream_with_block_size(
            &mut data.as_slice(),
            &mut compressed,
            1,
        )
        .unwrap();
        let mut decompressed = Vec::new();
        decompress_stream(&mut compressed.as_slice(), &mut decompressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[crate::ctb_test]
    fn test_roundtrip_rle_at_block_boundary() {
        // Construct an input where repeated sequences hit near allowable_block_size
        // (99,981 bytes) for block size 1, testing runs of >= 4 bytes spanning the boundary.
        let mut data = Vec::new();
        // Pad data to just before the block boundary
        data.resize(99_975, b'X');
        // Add a run of 10 'Y's which would straddle the boundary if sliced naively
        data.extend(vec![b'Y'; 10]);
        // Add another 50,000 bytes
        data.extend(vec![b'Z'; 50_000]);

        let mut compressed = Vec::new();
        compress_stream_with_block_size(
            &mut data.as_slice(),
            &mut compressed,
            1,
        )
        .unwrap();

        let mut decompressed = Vec::new();
        decompress_stream(&mut compressed.as_slice(), &mut decompressed).unwrap();
        assert_eq!(decompressed, data);
    }
}
