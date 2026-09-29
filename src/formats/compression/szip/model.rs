// SPDX-License-Identifier: AGPL-3.0-or-later AND Apache-2.0
// SPDX-License-Identifier for parts derived from szip: Apache-2.0
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

// Parts derived from szip:
/*
* Copyright 1997,1998,2021 Michael Schindler michael@compressconsult.com
*
* Licensed under the Apache License, Version 2.0 (the "License");
* you may not use this file except in compliance with the License.
* You may obtain a copy of the License at
*
*     http://www.apache.org/licenses/LICENSE-2.0
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific language governing permissions and
* limitations under the License.
*/
// See end of file for full license text for parts derived from szip.

//! Context probability model (`sz_mod4`) combining cache, MTF, and full fallback models.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;
use anyhow::anyhow;
use std::io::{Read, Write};

use super::bit_model::BitModel;
use super::qs_model::QsModel;
use super::range_coder::{RangeDecoder, RangeEncoder};

pub const ALPHABET_SIZE: usize = 256;
pub const CACHE_SIZE: usize = 32;
pub const MTF_SIZE: usize = 20;
pub const MTF_HIST_SIZE: usize = 4096;
pub const MTF_HIST_MASK: usize = MTF_HIST_SIZE.saturating_sub(1);
const EMPTY_ENTRY: u16 = 0xFFFF;

const RL_SHIFT: u32 = 10;
const MTF_SHIFT: u32 = 10;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum SymbolLocation {
    Cache(usize),
    Mtf,
    Full,
}

#[derive(Clone, Debug)]
struct CacheEntry {
    symbol: u8,
    sy_f: u32,
    weight: u32,
    what: usize,
    next: usize,
    prev: usize,
}

impl Default for CacheEntry {
    fn default() -> Self {
        Self {
            symbol: 0,
            sy_f: 0,
            weight: 0,
            what: 0,
            next: 0,
            prev: 0,
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct MtfEntry {
    sym: usize,
    next: u16,
}

/// Probability model coordinating cache, MTF, full bitmodel, and RLE models.
pub struct SzModel {
    what_mod: [u32; 3],
    newest: usize,
    last_new: usize,
    cache_tot_f: u32,
    mtf_first: usize,
    mtf_size: usize,
    mtf_size_act: usize,
    last_seen: [SymbolLocation; ALPHABET_SIZE],
    cache: [CacheEntry; CACHE_SIZE],
    mtf_hist: Vec<MtfEntry>,
    full: BitModel,
    mtf_mod: QsModel,
    rle_mod: [QsModel; 5],
}

impl SzModel {
    /// Initializes model for compression or decompression.
    pub fn new(is_compress: bool) -> Result<Self> {
        let full = BitModel::new(ALPHABET_SIZE, 40 * 256, 10 * 256)?;
        let mut last_seen = [SymbolLocation::Full; ALPHABET_SIZE];

        let mut cache: [CacheEntry; CACHE_SIZE] = std::array::from_fn(|_| CacheEntry::default());
        for i in 0..(CACHE_SIZE.saturating_sub(1)) {
            let next_idx = i.saturating_add(1);
            let prev_idx = if i == 0 {
                CACHE_SIZE.saturating_sub(1)
            } else {
                i.saturating_sub(1)
            };
            let sym = u8::try_from(
                CACHE_SIZE.saturating_sub(2).saturating_sub(i),
            )
            .unwrap_or(0);

            if let Some(entry) = cache.get_mut(i) {
                entry.next = next_idx;
                entry.prev = prev_idx;
                entry.symbol = sym;
                entry.sy_f = 1;
                entry.weight = 1;
                entry.what = 0;
            }
            if let Some(loc) = last_seen.get_mut(usize::from(sym)) {
                *loc = SymbolLocation::Cache(i);
            }
        }

        // Final cache entry (index 31)
        let last_idx = CACHE_SIZE.saturating_sub(1);
        if let Some(entry) = cache.get_mut(last_idx) {
            entry.next = 0;
            entry.prev = last_idx.saturating_sub(1);
            entry.sy_f = 0;
            entry.weight = 0;
            entry.what = 0;
        }
        if let Some(entry) = cache.first_mut() {
            entry.prev = last_idx;
        }

        let mut what_mod = [41u32, 8u32, 15u32];
        for i in 0..2 {
            if let Some(entry) = cache.get_mut(i) {
                entry.what = 2;
            }
            let ln_idx = (CACHE_SIZE.saturating_sub(7)).saturating_add(i);
            if let Some(entry) = cache.get_mut(ln_idx) {
                entry.what = 2;
            }
        }
        if let Some(entry) = cache.get_mut(2) {
            entry.what = 1;
        }
        let ln_idx2 = (CACHE_SIZE.saturating_sub(7)).saturating_add(2);
        if let Some(entry) = cache.get_mut(ln_idx2) {
            entry.what = 1;
        }

        let mut mtf_hist = vec![
            MtfEntry {
                sym: 0,
                next: EMPTY_ENTRY,
            };
            MTF_HIST_SIZE
        ];

        let mtf_init_size = MTF_SIZE.saturating_mul(2);
        if let Some(entry) = mtf_hist.first_mut() {
            entry.next = u16::try_from(MTF_HIST_SIZE.saturating_sub(1)).unwrap_or(0);
            entry.sym = CACHE_SIZE;
        }
        for i in 1..mtf_init_size {
            if let Some(entry) = mtf_hist.get_mut(i) {
                entry.next = u16::try_from(i.saturating_sub(1)).unwrap_or(0);
                entry.sym = CACHE_SIZE.saturating_add(i);
            }
        }

        let mtf_mod = QsModel::new(MTF_SIZE, MTF_SHIFT, 400, is_compress)?;

        let rle0 = QsModel::new(7, RL_SHIFT, 150, is_compress)?;
        let rle1 = QsModel::new(7, RL_SHIFT, 150, is_compress)?;
        let rle2 = QsModel::new(7, RL_SHIFT, 150, is_compress)?;
        let rle3 = QsModel::new(7, RL_SHIFT, 150, is_compress)?;
        let rle4 = QsModel::new(7, RL_SHIFT, 150, is_compress)?;

        let mut model = Self {
            what_mod,
            newest: CACHE_SIZE.saturating_sub(2),
            last_new: CACHE_SIZE.saturating_sub(7),
            cache_tot_f: u32::try_from(CACHE_SIZE).unwrap_or(32),
            mtf_first: mtf_init_size.saturating_sub(1),
            mtf_size: mtf_init_size,
            mtf_size_act: 0,
            last_seen,
            cache,
            mtf_hist,
            full,
            mtf_mod,
            rle_mod: [rle0, rle1, rle2, rle3, rle4],
        };

        // Deactivate cache symbols from full fallback model
        for i in 0..(CACHE_SIZE.saturating_sub(1)) {
            let sym = model.cache[i].symbol;
            model.full.deactivate(usize::from(sym));
        }

        Ok(model)
    }

    /// Adjusts cache total frequency count after the very first run.
    pub fn fix_after_first(&mut self) {
        self.cache_tot_f = self.cache_tot_f.saturating_sub(1);
    }

    fn add_to_mtf(&mut self, sym: usize) {
        let i = (self.mtf_first.saturating_add(1)) & MTF_HIST_MASK;
        let entry_next = self.mtf_hist.get(i).map(|e| e.next).unwrap_or(EMPTY_ENTRY);
        if entry_next == EMPTY_ENTRY {
            self.mtf_size = self.mtf_size.saturating_add(1);
            self.mtf_size_act = self.mtf_size_act.saturating_add(1);
        } else if self.mtf_size_act == self.mtf_size {
            let old_sym = self.mtf_hist.get(i).map(|e| e.sym).unwrap_or(0);
            if let Some(loc) = self.last_seen.get_mut(old_sym) {
                *loc = SymbolLocation::Full;
            }
            self.full.reactivate(old_sym);
        } else {
            self.mtf_size_act = self.mtf_size_act.saturating_add(1);
        }

        if let Some(entry) = self.mtf_hist.get_mut(i) {
            entry.next = u16::try_from(self.mtf_first).unwrap_or(0);
            entry.sym = sym;
        }
        self.mtf_first = i;
        if let Some(loc) = self.last_seen.get_mut(sym) {
            *loc = SymbolLocation::Mtf;
        }
    }

    fn finish_update(&mut self, symbol: u8) {
        let sym_idx = usize::from(symbol);
        let newest_idx = self.newest;
        if let Some(loc) = self.last_seen.get_mut(sym_idx) {
            *loc = SymbolLocation::Cache(newest_idx);
        }
        let newest_weight = self.cache.get(newest_idx).map(|e| e.weight).unwrap_or(0);
        if let Some(entry) = self.cache.get_mut(newest_idx) {
            entry.symbol = symbol;
        }
        self.cache_tot_f = self.cache_tot_f.saturating_add(newest_weight);

        let to_clear_idx = self.cache.get(newest_idx).map(|e| e.next).unwrap_or(0);
        let to_clear_what = self.cache.get(to_clear_idx).map(|e| e.what).unwrap_or(0);
        if let Some(val) = self.what_mod.get_mut(to_clear_what) {
            *val = val.saturating_sub(1);
        }

        let to_clear_sy_f = self.cache.get(to_clear_idx).map(|e| e.sy_f).unwrap_or(0);
        let to_clear_sym = self.cache.get(to_clear_idx).map(|e| e.symbol).unwrap_or(0);
        let to_clear_sym_idx = usize::from(to_clear_sym);

        if to_clear_sy_f == 0 {
            if let Some(SymbolLocation::Cache(c_idx)) = self.last_seen.get(to_clear_sym_idx) {
                if let Some(entry) = self.cache.get_mut(*c_idx) {
                    entry.sy_f = entry.sy_f.saturating_sub(1);
                }
            }
        } else {
            self.add_to_mtf(to_clear_sym_idx);
        }

        let last_new_idx = self.last_new;
        let ln_what = self.cache.get(last_new_idx).map(|e| e.what).unwrap_or(0);
        if let Some(val) = self.what_mod.get_mut(ln_what) {
            *val = val.saturating_sub(5);
        }

        let ln_weight = self.cache.get(last_new_idx).map(|e| e.weight).unwrap_or(0);
        self.cache_tot_f = self.cache_tot_f.saturating_sub(ln_weight);

        let ln_sym = self.cache.get(last_new_idx).map(|e| e.symbol).unwrap_or(0);
        let ln_sym_idx = usize::from(ln_sym);
        if let Some(SymbolLocation::Cache(c_idx)) = self.last_seen.get(ln_sym_idx) {
            if let Some(entry) = self.cache.get_mut(*c_idx) {
                entry.sy_f = entry.sy_f.saturating_sub(ln_weight.saturating_sub(1));
            }
        }

        if let Some(entry) = self.cache.get_mut(last_new_idx) {
            entry.weight = 1;
        }
        self.last_new = self.cache.get(last_new_idx).map(|e| e.next).unwrap_or(0);
    }

    fn write_run<W: Write>(
        &mut self,
        encoder: &mut RangeEncoder<W>,
        rl_idx: usize,
        n: u32,
    ) -> Result<u32> {
        let rlmod = self
            .rle_mod
            .get_mut(rl_idx)
            .ok_or_else(|| anyhow!("Invalid RLE model index"))?;
        if n <= 4 {
            let sym = usize::try_from(n.saturating_sub(1)).unwrap_or(0);
            let (sy_f, lt_f) = rlmod.get_freq(sym);
            encoder.encode_shift(sy_f, lt_f, RL_SHIFT)?;
            rlmod.update(sym);
            Ok(1 + (n >> 1))
        } else if n <= 8 {
            let (sy_f, lt_f) = rlmod.get_freq(4);
            encoder.encode_shift(sy_f, lt_f, RL_SHIFT)?;
            encoder.encode_shift(1, n.saturating_sub(5), 2)?;
            rlmod.update(4);
            Ok(3)
        } else if n <= 16 {
            let (sy_f, lt_f) = rlmod.get_freq(5);
            encoder.encode_shift(sy_f, lt_f, RL_SHIFT)?;
            encoder.encode_shift(1, n.saturating_sub(9), 3)?;
            rlmod.update(5);
            Ok(4)
        } else {
            let (sy_f, lt_f) = rlmod.get_freq(6);
            encoder.encode_shift(sy_f, lt_f, RL_SHIFT)?;
            if n < 32 {
                encoder.encode_shift(1, n, 5)?;
            } else {
                let mut i = 5u32;
                while (n >> i) > 1 {
                    i = i.saturating_add(1);
                }
                encoder.encode_shift(1, i.saturating_sub(5), 5)?;
                let rem = n.saturating_sub(1_u32 << i);
                encoder.encode_shift(1, rem, i)?;
            }
            rlmod.update(6);
            Ok(4)
        }
    }

    fn read_run<R: Read>(
        &mut self,
        decoder: &mut RangeDecoder<R>,
        rl_idx: usize,
    ) -> Result<(u32, u32)> {
        let rlmod = self
            .rle_mod
            .get_mut(rl_idx)
            .ok_or_else(|| anyhow!("Invalid RLE model index"))?;
        let shift_val = decoder.decode_culshift(RL_SHIFT)?;
        let rl = rlmod.get_sym(shift_val);
        let (sy_f, lt_f) = rlmod.get_freq(rl);
        decoder.update(sy_f, lt_f);
        rlmod.update(rl);

        if rl <= 3 {
            let run_len = u32::try_from(rl.saturating_add(1)).unwrap_or(1);
            let weight = 1 + (run_len >> 1);
            Ok((weight, run_len))
        } else if rl == 4 {
            let extra = decoder.decode_culshift(2)?;
            decoder.update(1, extra);
            let run_len = extra.saturating_add(5);
            Ok((3, run_len))
        } else if rl == 5 {
            let extra = decoder.decode_culshift(3)?;
            decoder.update(1, extra);
            let run_len = extra.saturating_add(9);
            Ok((4, run_len))
        } else {
            let extra = decoder.decode_culshift(5)?;
            decoder.update(1, extra);
            if extra > 16 {
                Ok((4, extra))
            } else {
                let bits_count = extra.saturating_add(5);
                let bits = decoder.decode_culshift(bits_count)?;
                decoder.update(1, bits);
                let run_len = bits.saturating_add(1_u32 << bits_count);
                Ok((4, run_len))
            }
        }
    }

    fn encode_other<W: Write>(
        &mut self,
        encoder: &mut RangeEncoder<W>,
        sym: usize,
    ) -> Result<usize> {
        let mut i = self.mtf_first;
        let mut last = i;
        if self.mtf_size_act >= MTF_SIZE {
            let mut n = 0usize;
            while n < MTF_SIZE {
                if self.mtf_hist.get(i).map(|e| e.sym) == Some(sym) {
                    return self.finish_mtf_hit(encoder, sym, i, last, n);
                }
                last = i;
                i = usize::from(self.mtf_hist.get(i).map(|e| e.next).unwrap_or(0));
                n = n.saturating_add(1);
            }
            while n < self.mtf_size_act {
                if let Some(entry) = self.mtf_hist.get(i) {
                    let entry_sym = entry.sym;
                    if let Some(loc) = self.last_seen.get_mut(entry_sym) {
                        *loc = SymbolLocation::Full;
                    }
                    self.full.reactivate(entry_sym);
                    i = usize::from(entry.next);
                }
                n = n.saturating_add(1);
            }
            self.mtf_size_act = MTF_SIZE;
        } else {
            let mut n = 0usize;
            while n < self.mtf_size_act {
                if self.mtf_hist.get(i).map(|e| e.sym) == Some(sym) {
                    return self.finish_mtf_hit(encoder, sym, i, last, n);
                }
                last = i;
                i = usize::from(self.mtf_hist.get(i).map(|e| e.next).unwrap_or(0));
                n = n.saturating_add(1);
            }
            self.mtf_size_act = if self.mtf_size > MTF_SIZE {
                MTF_SIZE
            } else {
                self.mtf_size
            };
            while n < self.mtf_size_act {
                let entry_sym = self.mtf_hist.get(i).map(|e| e.sym).unwrap_or(0);
                if self.last_seen.get(entry_sym) == Some(&SymbolLocation::Full) {
                    if let Some(loc) = self.last_seen.get_mut(entry_sym) {
                        *loc = SymbolLocation::Mtf;
                    }
                    self.full.deactivate(entry_sym);
                    if entry_sym == sym {
                        self.mtf_size_act = n.saturating_add(1);
                        return self.finish_mtf_hit(encoder, sym, i, last, n);
                    }
                    last = i;
                    i = usize::from(self.mtf_hist.get(i).map(|e| e.next).unwrap_or(0));
                    n = n.saturating_add(1);
                } else {
                    let next = usize::from(self.mtf_hist.get(i).map(|e| e.next).unwrap_or(0));
                    if n > 0 {
                        if let Some(entry) = self.mtf_hist.get_mut(last) {
                            entry.next = u16::try_from(next).unwrap_or(0);
                        }
                    } else {
                        self.mtf_first = next;
                    }
                    if let Some(entry) = self.mtf_hist.get_mut(i) {
                        entry.next = EMPTY_ENTRY;
                    }
                    i = next;
                    self.mtf_size = self.mtf_size.saturating_sub(1);
                    if self.mtf_size_act > self.mtf_size {
                        self.mtf_size_act = self.mtf_size;
                    }
                }
            }
        }

        // Full model fallback (submodel 2)
        let what2 = self.what_mod[2];
        let lt_what2 = self.what_mod[0].saturating_add(self.what_mod[1]);
        encoder.encode_shift(what2, lt_what2, 6)?;
        self.what_mod[2] = self.what_mod[2].saturating_add(6);

        let (sy_f, lt_f) = self.full.get_freq(sym);
        encoder.encode_freq(sy_f, lt_f, self.full.total_freq())?;
        self.full.update_exclude(sym);
        Ok(2)
    }

    fn finish_mtf_hit<W: Write>(
        &mut self,
        encoder: &mut RangeEncoder<W>,
        _sym: usize,
        i: usize,
        last: usize,
        n: usize,
    ) -> Result<usize> {
        let what1 = self.what_mod[1];
        let lt_what1 = self.what_mod[0];
        encoder.encode_shift(what1, lt_what1, 6)?;
        self.what_mod[1] = self.what_mod[1].saturating_add(6);

        let (sy_f, lt_f) = self.mtf_mod.get_freq(n);
        encoder.encode_shift(sy_f, lt_f, MTF_SHIFT)?;
        self.mtf_mod.update(n);

        let next_idx = usize::from(self.mtf_hist.get(i).map(|e| e.next).unwrap_or(0));
        if n == 0 {
            self.mtf_first = next_idx;
        } else if let Some(entry) = self.mtf_hist.get_mut(last) {
            entry.next = u16::try_from(next_idx).unwrap_or(0);
        }
        if let Some(entry) = self.mtf_hist.get_mut(i) {
            entry.next = EMPTY_ENTRY;
        }
        self.mtf_size = self.mtf_size.saturating_sub(1);
        self.mtf_size_act = self.mtf_size_act.saturating_sub(1);
        Ok(1)
    }

    /// Encodes a run of identical symbols `(symbol, runlength)`.
    pub fn encode<W: Write>(
        &mut self,
        encoder: &mut RangeEncoder<W>,
        symbol: u8,
        runlength: u32,
    ) -> Result<()> {
        let sym_idx = usize::from(symbol);
        if let Some(SymbolLocation::Cache(old_idx)) = self.last_seen.get(sym_idx).copied() {
            let what0 = self.what_mod[0];
            encoder.encode_shift(what0, 0, 6)?;
            self.what_mod[0] = self.what_mod[0].saturating_add(6);

            let old_sy_f = self.cache.get(old_idx).map(|e| e.sy_f).unwrap_or(0);
            let old_weight = self.cache.get(old_idx).map(|e| e.weight).unwrap_or(0);

            let mut lt_f = 0u32;
            let mut curr = self.cache.get(old_idx).map(|e| e.next).unwrap_or(0);
            while curr != self.newest {
                lt_f = lt_f.saturating_add(self.cache.get(curr).map(|e| e.sy_f).unwrap_or(0));
                curr = self.cache.get(curr).map(|e| e.next).unwrap_or(0);
            }

            let newest_sy_f = self.cache.get(self.newest).map(|e| e.sy_f).unwrap_or(0);
            let tot_f = self.cache_tot_f.saturating_sub(newest_sy_f);
            encoder.encode_freq(old_sy_f, lt_f, tot_f)?;

            let free_idx = self.cache.get(self.newest).map(|e| e.next).unwrap_or(0);
            let weight = self.write_run(encoder, usize::try_from(old_weight).unwrap_or(0), runlength)?;

            if let Some(entry) = self.cache.get_mut(free_idx) {
                entry.what = 0;
                entry.weight = weight;
                entry.sy_f = weight.saturating_add(old_sy_f);
            }
            if let Some(entry) = self.cache.get_mut(old_idx) {
                entry.sy_f = 0;
            }
            self.newest = free_idx;
        } else {
            let free_idx = self.cache.get(self.newest).map(|e| e.next).unwrap_or(0);
            let what = self.encode_other(encoder, sym_idx)?;
            let weight = self.write_run(encoder, 0, runlength)?;

            if let Some(entry) = self.cache.get_mut(free_idx) {
                entry.what = what;
                entry.weight = weight;
                entry.sy_f = weight;
            }
            self.newest = free_idx;
        }
        self.finish_update(symbol);
        Ok(())
    }

    fn activate_next(&mut self, next_ref: &mut usize) -> bool {
        while self.mtf_size > self.mtf_size_act {
            let idx = *next_ref;
            let sym = self.mtf_hist.get(idx).map(|e| e.sym).unwrap_or(0);
            if self.last_seen.get(sym) == Some(&SymbolLocation::Full) {
                self.full.deactivate(sym);
                if let Some(loc) = self.last_seen.get_mut(sym) {
                    *loc = SymbolLocation::Mtf;
                }
                self.mtf_size_act = self.mtf_size_act.saturating_add(1);
                return true;
            }
            let next = usize::from(self.mtf_hist.get(idx).map(|e| e.next).unwrap_or(0));
            *next_ref = next;
            if let Some(entry) = self.mtf_hist.get_mut(idx) {
                entry.next = EMPTY_ENTRY;
            }
            self.mtf_size = self.mtf_size.saturating_sub(1);
        }
        false
    }

    /// Decodes next run of identical symbols: returns `(symbol, runlength)`.
    pub fn decode<R: Read>(&mut self, decoder: &mut RangeDecoder<R>) -> Result<(u8, u32)> {
        let sym_choice = decoder.decode_culshift(6)?;
        if sym_choice < self.what_mod[0] {
            // Submodel 0: Cache
            let what0 = self.what_mod[0];
            decoder.update(what0, 0);
            self.what_mod[0] = self.what_mod[0].saturating_add(6);

            let newest_sy_f = self.cache.get(self.newest).map(|e| e.sy_f).unwrap_or(0);
            let tot_f = self.cache_tot_f.saturating_sub(newest_sy_f);
            let target_cul = decoder.decode_culfreq(tot_f)?;

            let mut curr = self.cache.get(self.newest).map(|e| e.prev).unwrap_or(0);
            let mut lt_f = self.cache.get(curr).map(|e| e.sy_f).unwrap_or(0);
            while lt_f <= target_cul {
                curr = self.cache.get(curr).map(|e| e.prev).unwrap_or(0);
                lt_f = lt_f.saturating_add(self.cache.get(curr).map(|e| e.sy_f).unwrap_or(0));
            }

            let curr_sy_f = self.cache.get(curr).map(|e| e.sy_f).unwrap_or(0);
            let curr_weight = self.cache.get(curr).map(|e| e.weight).unwrap_or(0);
            let curr_sym = self.cache.get(curr).map(|e| e.symbol).unwrap_or(0);

            decoder.update(curr_sy_f, lt_f.saturating_sub(curr_sy_f));

            let free_idx = self.cache.get(self.newest).map(|e| e.next).unwrap_or(0);
            self.newest = free_idx;

            let (weight, run_len) = self.read_run(decoder, usize::try_from(curr_weight).unwrap_or(0))?;
            if let Some(entry) = self.cache.get_mut(free_idx) {
                entry.what = 0;
                entry.weight = weight;
                entry.sy_f = weight.saturating_add(curr_sy_f);
            }
            if let Some(entry) = self.cache.get_mut(curr) {
                entry.sy_f = 0;
            }
            self.finish_update(curr_sym);
            Ok((curr_sym, run_len))
        } else if sym_choice < self.what_mod[0].saturating_add(self.what_mod[1]) {
            // Submodel 1: MTF
            let what1 = self.what_mod[1];
            let lt_what1 = self.what_mod[0];
            decoder.update(what1, lt_what1);
            self.what_mod[1] = self.what_mod[1].saturating_add(6);

            let shift_val = decoder.decode_culshift(MTF_SHIFT)?;
            let sym = self.mtf_mod.get_sym(shift_val);
            let (sy_f, lt_f) = self.mtf_mod.get_freq(sym);
            decoder.update(sy_f, lt_f);
            self.mtf_mod.update(sym);

            if self.mtf_size_act == 0 {
                let mut first_ref = self.mtf_first;
                self.activate_next(&mut first_ref);
                self.mtf_first = first_ref;
            }

            let resolved_sym: usize;
            if sym == 0 {
                if self.mtf_size_act == 0 {
                    let mut first_ref = self.mtf_first;
                    self.activate_next(&mut first_ref);
                    self.mtf_first = first_ref;
                }
                let first_idx = self.mtf_first;
                resolved_sym = self.mtf_hist.get(first_idx).map(|e| e.sym).unwrap_or(0);
                let next_idx = usize::from(
                    self.mtf_hist.get(first_idx).map(|e| e.next).unwrap_or(0),
                );
                self.mtf_first = next_idx;
                if let Some(entry) = self.mtf_hist.get_mut(first_idx) {
                    entry.next = EMPTY_ENTRY;
                }
            } else {
                let mut pred = self.mtf_first;
                if sym < self.mtf_size_act {
                    for _ in 0..(sym.saturating_sub(1)) {
                        pred = usize::from(
                            self.mtf_hist.get(pred).map(|e| e.next).unwrap_or(0),
                        );
                    }
                } else {
                    for _ in 0..(self.mtf_size_act.saturating_sub(1)) {
                        pred = usize::from(
                            self.mtf_hist.get(pred).map(|e| e.next).unwrap_or(0),
                        );
                    }
                    while self.mtf_size_act < sym {
                        let mut next_ref = usize::from(
                            self.mtf_hist.get(pred).map(|e| e.next).unwrap_or(0),
                        );
                        self.activate_next(&mut next_ref);
                        if let Some(entry) = self.mtf_hist.get_mut(pred) {
                            entry.next = u16::try_from(next_ref).unwrap_or(0);
                        }
                        pred = next_ref;
                    }
                    let mut next_ref = usize::from(
                        self.mtf_hist.get(pred).map(|e| e.next).unwrap_or(0),
                    );
                    self.activate_next(&mut next_ref);
                    if let Some(entry) = self.mtf_hist.get_mut(pred) {
                        entry.next = u16::try_from(next_ref).unwrap_or(0);
                    }
                }
                let target_idx = usize::from(
                    self.mtf_hist.get(pred).map(|e| e.next).unwrap_or(0),
                );
                resolved_sym = self.mtf_hist.get(target_idx).map(|e| e.sym).unwrap_or(0);
                let target_next = self.mtf_hist.get(target_idx).map(|e| e.next).unwrap_or(0);
                if let Some(entry) = self.mtf_hist.get_mut(pred) {
                    entry.next = target_next;
                }
                if let Some(entry) = self.mtf_hist.get_mut(target_idx) {
                    entry.next = EMPTY_ENTRY;
                }
            }

            self.mtf_size_act = self.mtf_size_act.saturating_sub(1);
            self.mtf_size = self.mtf_size.saturating_sub(1);

            let free_idx = self.cache.get(self.newest).map(|e| e.next).unwrap_or(0);
            self.newest = free_idx;

            let (weight, run_len) = self.read_run(decoder, 0)?;
            if let Some(entry) = self.cache.get_mut(free_idx) {
                entry.what = 1;
                entry.weight = weight;
                entry.sy_f = weight;
            }

            let sym_u8 = u8::try_from(resolved_sym).unwrap_or(0);
            self.finish_update(sym_u8);
            Ok((sym_u8, run_len))
        } else {
            // Submodel 2: Full fallback model
            let what2 = self.what_mod[2];
            let lt_what2 = self.what_mod[0].saturating_add(self.what_mod[1]);
            decoder.update(what2, lt_what2);
            self.what_mod[2] = self.what_mod[2].saturating_add(6);

            if self.mtf_size_act > MTF_SIZE {
                let mut i = self.mtf_first;
                for _ in 0..MTF_SIZE {
                    i = usize::from(self.mtf_hist.get(i).map(|e| e.next).unwrap_or(0));
                }
                let mut n = MTF_SIZE;
                while n < self.mtf_size_act {
                    let entry_sym = self.mtf_hist.get(i).map(|e| e.sym).unwrap_or(0);
                    self.full.reactivate(entry_sym);
                    if let Some(loc) = self.last_seen.get_mut(entry_sym) {
                        *loc = SymbolLocation::Full;
                    }
                    i = usize::from(self.mtf_hist.get(i).map(|e| e.next).unwrap_or(0));
                    n = n.saturating_add(1);
                }
                self.mtf_size_act = MTF_SIZE;
            } else if self.mtf_size_act < MTF_SIZE {
                let mut pred = self.mtf_first;
                if self.mtf_size_act == 0 {
                    let mut first_ref = self.mtf_first;
                    self.activate_next(&mut first_ref);
                    self.mtf_first = first_ref;
                    pred = first_ref;
                } else {
                    for _ in 0..(self.mtf_size_act.saturating_sub(1)) {
                        pred = usize::from(
                            self.mtf_hist.get(pred).map(|e| e.next).unwrap_or(0),
                        );
                    }
                }
                while self.mtf_size_act < MTF_SIZE {
                    let mut next_ref = usize::from(
                        self.mtf_hist.get(pred).map(|e| e.next).unwrap_or(0),
                    );
                    if !self.activate_next(&mut next_ref) {
                        break;
                    }
                    if let Some(entry) = self.mtf_hist.get_mut(pred) {
                        entry.next = u16::try_from(next_ref).unwrap_or(0);
                    }
                    pred = next_ref;
                }
            }

            let sym_choice = decoder.decode_culfreq(self.full.total_freq())?;
            let sym = self.full.get_sym(sym_choice);
            let (sy_f, lt_f) = self.full.get_freq(sym);
            decoder.update(sy_f, lt_f);
            self.full.update_exclude(sym);

            let free_idx = self.cache.get(self.newest).map(|e| e.next).unwrap_or(0);
            self.newest = free_idx;

            let (weight, run_len) = self.read_run(decoder, 0)?;
            if let Some(entry) = self.cache.get_mut(free_idx) {
                entry.what = 2;
                entry.weight = weight;
                entry.sy_f = weight;
            }

            let sym_u8 = u8::try_from(sym).unwrap_or(0);
            self.finish_update(sym_u8);
            Ok((sym_u8, run_len))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[crate::ctb_test]
    fn test_szmodel_roundtrip() {
        let input = b"\x63\x53\x31\x48\x20\x20\x31\x2e\x31\x2b\x65\x7a\x6f\x6c\x73\x6f\x70\x6c\x6f\x70\x21\x6d\x6e\x2c\x20\x72\x65\x73\x69\x69";
        let mut encoded = Vec::new();
        {
            let mut enc = RangeEncoder::new(&mut encoded, 1, 10);
            let mut model = SzModel::new(true).unwrap();
            let mut pos = 0usize;
            let mut is_first = true;
            while pos < input.len() {
                let sym = input[pos];
                let mut run_len = 1u32;
                pos += 1;
                while pos < input.len() && input[pos] == sym {
                    run_len += 1;
                    pos += 1;
                }
                model.encode(&mut enc, sym, run_len).unwrap();
                if is_first {
                    model.fix_after_first();
                    is_first = false;
                }
            }
            enc.finish().unwrap();
        }

        println!("Encoded bytes len: {}", encoded.len());

        let mut decoded = Vec::new();
        {
            let mut cur = std::io::Cursor::new(&encoded);
            let (mut dec, rec_byte) = RangeDecoder::new(&mut cur).unwrap().unwrap();
            assert_eq!(rec_byte, 1);
            let mut model = SzModel::new(false).unwrap();
            let mut bytes_left = input.len();
            let mut is_first = true;
            while bytes_left > 0 {
                let (sym, run_len) = model.decode(&mut dec).unwrap();
                println!("Decoded sym: {sym:02x} ('{}'), run_len: {run_len}", sym as char);
                for _ in 0..run_len {
                    decoded.push(sym);
                }
                bytes_left -= run_len as usize;
                if is_first {
                    model.fix_after_first();
                    is_first = false;
                }
            }
            dec.finish().unwrap();
        }

        assert_eq!(decoded.as_slice(), input.as_slice());
    }
}

/*

From `LICENSE` in szip:

```
                                 Apache License
                           Version 2.0, January 2004
                        http://www.apache.org/licenses/

   TERMS AND CONDITIONS FOR USE, REPRODUCTION, AND DISTRIBUTION

   1. Definitions.

      "License" shall mean the terms and conditions for use, reproduction,
      and distribution as defined by Sections 1 through 9 of this document.

      "Licensor" shall mean the copyright owner or entity authorized by
      the copyright owner that is granting the License.

      "Legal Entity" shall mean the union of the acting entity and all
      other entities that control, are controlled by, or are under common
      control with that entity. For the purposes of this definition,
      "control" means (i) the power, direct or indirect, to cause the
      direction or management of such entity, whether by contract or
      otherwise, or (ii) ownership of fifty percent (50%) or more of the
      outstanding shares, or (iii) beneficial ownership of such entity.

      "You" (or "Your") shall mean an individual or Legal Entity
      exercising permissions granted by this License.

      "Source" form shall mean the preferred form for making modifications,
      including but not limited to software source code, documentation
      source, and configuration files.

      "Object" form shall mean any form resulting from mechanical
      transformation or translation of a Source form, including but
      not limited to compiled object code, generated documentation,
      and conversions to other media types.

      "Work" shall mean the work of authorship, whether in Source or
      Object form, made available under the License, as indicated by a
      copyright notice that is included in or attached to the work
      (an example is provided in the Appendix below).

      "Derivative Works" shall mean any work, whether in Source or Object
      form, that is based on (or derived from) the Work and for which the
      editorial revisions, annotations, elaborations, or other modifications
      represent, as a whole, an original work of authorship. For the purposes
      of this License, Derivative Works shall not include works that remain
      separable from, or merely link (or bind by name) to the interfaces of,
      the Work and Derivative Works thereof.

      "Contribution" shall mean any work of authorship, including
      the original version of the Work and any modifications or additions
      to that Work or Derivative Works thereof, that is intentionally
      submitted to Licensor for inclusion in the Work by the copyright owner
      or by an individual or Legal Entity authorized to submit on behalf of
      the copyright owner. For the purposes of this definition, "submitted"
      means any form of electronic, verbal, or written communication sent
      to the Licensor or its representatives, including but not limited to
      communication on electronic mailing lists, source code control systems,
      and issue tracking systems that are managed by, or on behalf of, the
      Licensor for the purpose of discussing and improving the Work, but
      excluding communication that is conspicuously marked or otherwise
      designated in writing by the copyright owner as "Not a Contribution."

      "Contributor" shall mean Licensor and any individual or Legal Entity
      on behalf of whom a Contribution has been received by Licensor and
      subsequently incorporated within the Work.

   2. Grant of Copyright License. Subject to the terms and conditions of
      this License, each Contributor hereby grants to You a perpetual,
      worldwide, non-exclusive, no-charge, royalty-free, irrevocable
      copyright license to reproduce, prepare Derivative Works of,
      publicly display, publicly perform, sublicense, and distribute the
      Work and such Derivative Works in Source or Object form.

   3. Grant of Patent License. Subject to the terms and conditions of
      this License, each Contributor hereby grants to You a perpetual,
      worldwide, non-exclusive, no-charge, royalty-free, irrevocable
      (except as stated in this section) patent license to make, have made,
      use, offer to sell, sell, import, and otherwise transfer the Work,
      where such license applies only to those patent claims licensable
      by such Contributor that are necessarily infringed by their
      Contribution(s) alone or by combination of their Contribution(s)
      with the Work to which such Contribution(s) was submitted. If You
      institute patent litigation against any entity (including a
      cross-claim or counterclaim in a lawsuit) alleging that the Work
      or a Contribution incorporated within the Work constitutes direct
      or contributory patent infringement, then any patent licenses
      granted to You under this License for that Work shall terminate
      as of the date such litigation is filed.

   4. Redistribution. You may reproduce and distribute copies of the
      Work or Derivative Works thereof in any medium, with or without
      modifications, and in Source or Object form, provided that You
      meet the following conditions:

      (a) You must give any other recipients of the Work or
          Derivative Works a copy of this License; and

      (b) You must cause any modified files to carry prominent notices
          stating that You changed the files; and

      (c) You must retain, in the Source form of any Derivative Works
          that You distribute, all copyright, patent, trademark, and
          attribution notices from the Source form of the Work,
          excluding those notices that do not pertain to any part of
          the Derivative Works; and

      (d) If the Work includes a "NOTICE" text file as part of its
          distribution, then any Derivative Works that You distribute must
          include a readable copy of the attribution notices contained
          within such NOTICE file, excluding those notices that do not
          pertain to any part of the Derivative Works, in at least one
          of the following places: within a NOTICE text file distributed
          as part of the Derivative Works; within the Source form or
          documentation, if provided along with the Derivative Works; or,
          within a display generated by the Derivative Works, if and
          wherever such third-party notices normally appear. The contents
          of the NOTICE file are for informational purposes only and
          do not modify the License. You may add Your own attribution
          notices within Derivative Works that You distribute, alongside
          or as an addendum to the NOTICE text from the Work, provided
          that such additional attribution notices cannot be construed
          as modifying the License.

      You may add Your own copyright statement to Your modifications and
      may provide additional or different license terms and conditions
      for use, reproduction, or distribution of Your modifications, or
      for any such Derivative Works as a whole, provided Your use,
      reproduction, and distribution of the Work otherwise complies with
      the conditions stated in this License.

   5. Submission of Contributions. Unless You explicitly state otherwise,
      any Contribution intentionally submitted for inclusion in the Work
      by You to the Licensor shall be under the terms and conditions of
      this License, without any additional terms or conditions.
      Notwithstanding the above, nothing herein shall supersede or modify
      the terms of any separate license agreement you may have executed
      with Licensor regarding such Contributions.

   6. Trademarks. This License does not grant permission to use the trade
      names, trademarks, service marks, or product names of the Licensor,
      except as required for reasonable and customary use in describing the
      origin of the Work and reproducing the content of the NOTICE file.

   7. Disclaimer of Warranty. Unless required by applicable law or
      agreed to in writing, Licensor provides the Work (and each
      Contributor provides its Contributions) on an "AS IS" BASIS,
      WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
      implied, including, without limitation, any warranties or conditions
      of TITLE, NON-INFRINGEMENT, MERCHANTABILITY, or FITNESS FOR A
      PARTICULAR PURPOSE. You are solely responsible for determining the
      appropriateness of using or redistributing the Work and assume any
      risks associated with Your exercise of permissions under this License.

   8. Limitation of Liability. In no event and under no legal theory,
      whether in tort (including negligence), contract, or otherwise,
      unless required by applicable law (such as deliberate and grossly
      negligent acts) or agreed to in writing, shall any Contributor be
      liable to You for damages, including any direct, indirect, special,
      incidental, or consequential damages of any character arising as a
      result of this License or out of the use or inability to use the
      Work (including but not limited to damages for loss of goodwill,
      work stoppage, computer failure or malfunction, or any and all
      other commercial damages or losses), even if such Contributor
      has been advised of the possibility of such damages.

   9. Accepting Warranty or Additional Liability. While redistributing
      the Work or Derivative Works thereof, You may choose to offer,
      and charge a fee for, acceptance of support, warranty, indemnity,
      or other liability obligations and/or rights consistent with this
      License. However, in accepting such obligations, You may act only
      on Your own behalf and on Your sole responsibility, not on behalf
      of any other Contributor, and only if You agree to indemnify,
      defend, and hold each Contributor harmless for any liability
      incurred by, or claims asserted against, such Contributor by reason
      of your accepting any such warranty or additional liability.

   END OF TERMS AND CONDITIONS

   APPENDIX: How to apply the Apache License to your work.

      To apply the Apache License to your work, attach the following
      boilerplate notice, with the fields enclosed by brackets "[]"
      replaced with your own identifying information. (Don't include
      the brackets!)  The text should be enclosed in the appropriate
      comment syntax for the file format. We also recommend that a
      file or class name and description of purpose be included on the
      same "printed page" as the copyright notice for easier
      identification within third-party archives.

   Copyright [yyyy] [name of copyright owner]

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
```
*/
