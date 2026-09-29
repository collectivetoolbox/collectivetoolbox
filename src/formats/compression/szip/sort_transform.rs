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

//! Context sorting transformations: Order 0 (BWT), Order 4, and general Order N.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;
use anyhow::anyhow;
use std::cmp::Ordering;

const INDIRECT: u32 = 0x8000_0000;

fn set_bit(flags: &mut [u8], bit: usize) {
    let byte_idx = bit >> 3;
    let bit_idx = bit & 7;
    if let Some(slot) = flags.get_mut(byte_idx) {
        *slot |= 1 << bit_idx;
    }
}

fn get_bit(flags: &[u8], bit: usize) -> bool {
    let byte_idx = bit >> 3;
    let bit_idx = bit & 7;
    flags
        .get(byte_idx)
        .is_some_and(|&b| ((b >> bit_idx) & 1) != 0)
}

// -----------------------------------------------------------------------------
// Order 0: BWT with reverse context
// -----------------------------------------------------------------------------

fn qscompare(a: usize, b: usize, data: &[u8], minmatch: usize) -> Ordering {
    if a == b {
        return Ordering::Equal;
    }
    let val_a = a.saturating_sub(minmatch);
    let val_b = b.saturating_sub(minmatch);
    if val_a == val_b {
        return Ordering::Equal;
    }
    if val_a < val_b {
        let mut a1 = val_a;
        let mut b1 = val_b;
        while a1 > 0 && data.get(a1) == data.get(b1) {
            a1 = a1.saturating_sub(1);
            b1 = b1.saturating_sub(1);
        }
        if data.get(a1) <= data.get(b1) {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    } else {
        let mut a1 = val_b;
        let mut b1 = val_a;
        while a1 > 0 && data.get(a1) == data.get(b1) {
            a1 = a1.saturating_sub(1);
            b1 = b1.saturating_sub(1);
        }
        if data.get(a1) <= data.get(b1) {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    }
}

/// Unlimited order context sorting (BWT with context preceding symbol).
pub fn sort_bwt(inout: &mut [u8]) -> Result<u32> {
    let length = inout.len();
    if length <= 1 {
        return Ok(0);
    }

    let mut counts = [0usize; 256];
    for &b in &*inout {
        if let Some(slot) = counts.get_mut(usize::from(b)) {
            *slot = slot.saturating_add(1);
        }
    }

    let mut counts1 = [0usize; 256];
    for i in 0..255 {
        let cur = *counts1
            .get(i)
            .ok_or_else(|| anyhow!("Frequency count lookup out of bounds at {i}"))?;
        let c_val = *counts
            .get(i)
            .ok_or_else(|| anyhow!("Frequency count lookup out of bounds at {i}"))?;
        if let Some(next) = counts1.get_mut(i.saturating_add(1)) {
            *next = cur.saturating_add(c_val);
        }
    }

    let mut contextp = vec![0usize; length];
    for (i, &b) in inout.iter().enumerate() {
        let b_idx = usize::from(b);
        if let Some(slot) = counts1.get_mut(b_idx) {
            let pos = *slot;
            *slot = slot.saturating_add(1);
            if let Some(cp_slot) = contextp.get_mut(pos) {
                *cp_slot = i;
            }
        }
    }

    let mut indexfirst = 0u32;
    let mut start = 0usize;
    let first_byte = *inout
        .first()
        .ok_or_else(|| anyhow!("Empty buffer during sort"))?;
    let last_byte = *inout
        .last()
        .ok_or_else(|| anyhow!("Empty buffer during sort"))?;

    for i in 0..256 {
        let cnt = *counts
            .get(i)
            .ok_or_else(|| anyhow!("Missing frequency count bucket at {i}"))?;
        if cnt > 0 {
            let i_u8 = u8::try_from(i).map_err(|e| anyhow!("Index conversion: {e}"))?;
            let minmatch = usize::from(i_u8 != first_byte);
            let end = start.saturating_add(cnt);
            if let Some(slice) = contextp.get_mut(start..end) {
                slice.sort_by(|&a, &b| qscompare(a, b, inout, minmatch));
            }
            if i_u8 == last_byte {
                let target = length.saturating_sub(1);
                for (j, &val) in contextp.iter().enumerate().skip(start).take(cnt) {
                    if val == target {
                        indexfirst = u32::try_from(j)
                            .map_err(|e| anyhow!("Indexfirst conversion: {e}"))?;
                        break;
                    }
                }
            }
            start = end;
        }
    }

    let mut transformed = vec![0u8; length];
    let idx_first_usize = usize::try_from(indexfirst)
        .map_err(|e| anyhow!("Indexfirst conversion: {e}"))?;
    for (i, slot) in transformed.iter_mut().enumerate() {
        if i == idx_first_usize {
            *slot = *inout
                .first()
                .ok_or_else(|| anyhow!("Empty buffer during sort"))?;
        } else {
            let next_pos = contextp
                .get(i)
                .copied()
                .ok_or_else(|| anyhow!("Context position missing at {i}"))?
                .saturating_add(1);
            *slot = *inout
                .get(next_pos)
                .ok_or_else(|| anyhow!("Position index out of bounds at {next_pos}"))?;
        }
    }
    inout.copy_from_slice(&transformed);
    Ok(indexfirst)
}

/// Unsorts unlimited order BWT block.
pub fn unsort_bwt(input: &[u8], output: &mut [u8], indexfirst: u32) -> Result<()> {
    let length = input.len();
    if length <= 1 {
        output.copy_from_slice(input);
        return Ok(());
    }
    if output.len() < length {
        bail!("Output buffer smaller than input for unsort_bwt");
    }

    let mut counts = [0usize; 256];
    for &b in input {
        if let Some(slot) = counts.get_mut(usize::from(b)) {
            *slot = slot.saturating_add(1);
        }
    }

    let mut sum = length;
    for i in (0..256).rev() {
        let c = *counts
            .get(i)
            .ok_or_else(|| anyhow!("Missing count bucket {i}"))?;
        sum = sum.saturating_sub(c);
        if let Some(slot) = counts.get_mut(i) {
            *slot = sum;
        }
    }

    let mut transvec = vec![0usize; length];
    let idx_first_usize = usize::try_from(indexfirst)
        .map_err(|e| anyhow!("Invalid indexfirst: {e}"))?;
    if idx_first_usize >= length {
        bail!("indexfirst {indexfirst} exceeds block length {length}");
    }

    let get_and_inc = |counts: &mut [usize; 256], b: u8| -> usize {
        let b_idx = usize::from(b);
        if let Some(slot) = counts.get_mut(b_idx) {
            let val = *slot;
            *slot = slot.saturating_add(1);
            val
        } else {
            0
        }
    };

    if let Some(slot) = transvec.get_mut(idx_first_usize) {
        let in_byte = *input
            .get(idx_first_usize)
            .ok_or_else(|| anyhow!("Indexfirst out of bounds in input"))?;
        *slot = get_and_inc(&mut counts, in_byte);
    }
    for i in 0..idx_first_usize {
        if let Some(slot) = transvec.get_mut(i) {
            let in_byte = *input
                .get(i)
                .ok_or_else(|| anyhow!("Input byte out of bounds at {i}"))?;
            *slot = get_and_inc(&mut counts, in_byte);
        }
    }
    for i in (idx_first_usize.saturating_add(1))..length {
        if let Some(slot) = transvec.get_mut(i) {
            let in_byte = *input
                .get(i)
                .ok_or_else(|| anyhow!("Input byte out of bounds at {i}"))?;
            *slot = get_and_inc(&mut counts, in_byte);
        }
    }

    let mut ic = idx_first_usize;
    for slot in output.iter_mut().take(length) {
        *slot = *input
            .get(ic)
            .ok_or_else(|| anyhow!("Corrupted BWT transform: input index {ic} out of bounds"))?;
        ic = *transvec
            .get(ic)
            .ok_or_else(|| anyhow!("Corrupted BWT transform: transvec index {ic} out of bounds"))?;
    }
    if ic != idx_first_usize {
        bail!("Szip BWT cycle verification failed");
    }
    Ok(())
}

// -----------------------------------------------------------------------------
// Order 4: High-speed two-pass sort
// -----------------------------------------------------------------------------

/// Sorts block using fast specialized order 4 algorithm.
pub fn sort_order4(inout: &mut [u8]) -> Result<u32> {
    let length = inout.len();
    if length <= 4 {
        return Ok(0);
    }

    let mut counters = vec![0usize; 0x10000];
    let last_byte = *inout
        .last()
        .ok_or_else(|| anyhow!("Input buffer empty for order4 sort"))?;
    let mut ctx_u16 = usize::from(last_byte) << 8;
    for &b in &*inout {
        ctx_u16 = (ctx_u16 >> 8) | (usize::from(b) << 8);
        if let Some(slot) = counters.get_mut(ctx_u16) {
            *slot = slot.saturating_add(1);
        }
    }

    let mut sum = length;
    for i in (0..0x10000).rev() {
        let cnt = *counters
            .get(i)
            .ok_or_else(|| anyhow!("Missing counter at {i}"))?;
        sum = sum.saturating_sub(cnt);
        if let Some(slot) = counters.get_mut(i) {
            *slot = sum;
        }
    }

    let mut context = vec![0u16; length];
    let mut symbols = vec![0u8; length];

    let b_len4 = *inout
        .get(length.saturating_sub(4))
        .ok_or_else(|| anyhow!("Input buffer too short for order4 sort"))?;
    let b_len5 = *inout
        .get(length.saturating_sub(5))
        .ok_or_else(|| anyhow!("Input buffer too short for order4 sort"))?;
    let initial_ctx = (usize::from(b_len4) << 8) | usize::from(b_len5);

    let mut indexlast = if initial_ctx == 0xFFFF {
        length.saturating_sub(1)
    } else {
        counters
            .get(initial_ctx.saturating_add(1))
            .copied()
            .ok_or_else(|| anyhow!("Missing counter for initial context {initial_ctx}"))?
            .saturating_sub(1)
    };

    let b_len1 = *inout
        .get(length.saturating_sub(1))
        .ok_or_else(|| anyhow!("Input buffer too short for order4 sort"))?;
    let b_len2 = *inout
        .get(length.saturating_sub(2))
        .ok_or_else(|| anyhow!("Input buffer too short for order4 sort"))?;
    let b_len3 = *inout
        .get(length.saturating_sub(3))
        .ok_or_else(|| anyhow!("Input buffer too short for order4 sort"))?;
    let mut full_ctx = ((((usize::from(b_len1) << 8) | usize::from(b_len2)) << 8)
        | usize::from(b_len3))
        << 8
        | usize::from(b_len4);

    for &b in &*inout {
        let low_ctx = full_ctx & 0xFFFF;
        let x = counters
            .get(low_ctx)
            .copied()
            .ok_or_else(|| anyhow!("Missing counter for low context {low_ctx}"))?;
        if let Some(slot) = counters.get_mut(low_ctx) {
            *slot = slot.saturating_add(1);
        }
        if let Some(c_slot) = context.get_mut(x) {
            *c_slot = u16::try_from((full_ctx >> 16) & 0xFFFF)
                .map_err(|e| anyhow!("Context conversion: {e}"))?;
        }
        if let Some(s_slot) = symbols.get_mut(x) {
            *s_slot = b;
        }
        full_ctx = (full_ctx >> 8) | (usize::from(b) << 24);
    }

    let mut i = length;
    let last_pos = indexlast;
    while i > last_pos {
        i = i.saturating_sub(1);
        let c_val = usize::from(
            *context
                .get(i)
                .ok_or_else(|| anyhow!("Context index out of bounds: {i}"))?,
        );
        let cnt_slot = counters
            .get_mut(c_val)
            .ok_or_else(|| anyhow!("Missing counter for context: {c_val}"))?;
        *cnt_slot = cnt_slot.saturating_sub(1);
        let pos = *cnt_slot;
        let out_slot = inout
            .get_mut(pos)
            .ok_or_else(|| anyhow!("Output buffer position out of bounds: {pos}"))?;
        *out_slot = *symbols
            .get(i)
            .ok_or_else(|| anyhow!("Symbol index out of bounds: {i}"))?;
    }

    let c_val_last = usize::from(
        *context
            .get(i)
            .ok_or_else(|| anyhow!("Context index out of bounds: {i}"))?,
    );
    indexlast = *counters
        .get(c_val_last)
        .ok_or_else(|| anyhow!("Missing counter for context: {c_val_last}"))?;

    while i > 0 {
        i = i.saturating_sub(1);
        let c_val = usize::from(
            *context
                .get(i)
                .ok_or_else(|| anyhow!("Context index out of bounds: {i}"))?,
        );
        let cnt_slot = counters
            .get_mut(c_val)
            .ok_or_else(|| anyhow!("Missing counter for context: {c_val}"))?;
        *cnt_slot = cnt_slot.saturating_sub(1);
        let pos = *cnt_slot;
        let out_slot = inout
            .get_mut(pos)
            .ok_or_else(|| anyhow!("Output buffer position out of bounds: {pos}"))?;
        *out_slot = *symbols
            .get(i)
            .ok_or_else(|| anyhow!("Symbol index out of bounds: {i}"))?;
    }

    u32::try_from(indexlast).map_err(|e| anyhow!("Indexlast conversion: {e}"))
}

// -----------------------------------------------------------------------------
// General Order N (N >= 3)
// -----------------------------------------------------------------------------

fn sort_order2(
    in_ext: &mut [u8],
    length: usize,
    order: usize,
    mut o2counts: Vec<usize>,
) -> Result<(usize, Vec<u32>)> {
    let b_len_off = *in_ext
        .get(length.saturating_sub(order))
        .ok_or_else(|| anyhow!("Input buffer too short for order2 sort"))?;
    let b_len_off_minus_1 = *in_ext
        .get(length.saturating_sub(order).saturating_sub(1))
        .ok_or_else(|| anyhow!("Input buffer too short for order2 sort"))?;
    let init_ctx = (usize::from(b_len_off) << 8) | usize::from(b_len_off_minus_1);

    let indexlast = if init_ctx == 0xFFFF {
        length.saturating_sub(1)
    } else {
        o2counts
            .get(init_ctx.saturating_add(1))
            .copied()
            .ok_or_else(|| anyhow!("Missing counter for initial order2 context {init_ctx}"))?
            .saturating_sub(1)
    };

    let mut context = init_ctx;
    let offset = order.saturating_sub(1);
    let mut ptrs = vec![0u32; length];

    for i in 0..offset {
        let copy_val = *in_ext
            .get(i)
            .ok_or_else(|| anyhow!("Missing prefix byte at index {i}"))?;
        let dest = in_ext
            .get_mut(length.saturating_add(i))
            .ok_or_else(|| anyhow!("Extended buffer index out of bounds"))?;
        *dest = copy_val;
        let sample_idx = length.saturating_add(i).saturating_sub(offset);
        let sample_byte = *in_ext
            .get(sample_idx)
            .ok_or_else(|| anyhow!("Missing sample byte at index {sample_idx}"))?;
        context = (context >> 8) | (usize::from(sample_byte) << 8);

        let dest_pos = *o2counts
            .get(context)
            .ok_or_else(|| anyhow!("Missing context count for {context}"))?;
        if let Some(slot) = o2counts.get_mut(context) {
            *slot = slot.saturating_add(1);
        }
        let ptr_val = u32::try_from(length.saturating_add(i))
            .map_err(|e| anyhow!("Ptr conversion: {e}"))?;
        let slot = ptrs
            .get_mut(dest_pos)
            .ok_or_else(|| anyhow!("Ptr position {dest_pos} out of bounds"))?;
        *slot = ptr_val;
    }

    for i in offset..length {
        let sample_idx = i.saturating_sub(offset);
        let sample_byte = *in_ext
            .get(sample_idx)
            .ok_or_else(|| anyhow!("Missing sample byte at index {sample_idx}"))?;
        context = (context >> 8) | (usize::from(sample_byte) << 8);

        let dest_pos = *o2counts
            .get(context)
            .ok_or_else(|| anyhow!("Missing context count for {context}"))?;
        if let Some(slot) = o2counts.get_mut(context) {
            *slot = slot.saturating_add(1);
        }
        let ptr_val = u32::try_from(i).map_err(|e| anyhow!("Ptr conversion: {e}"))?;
        let slot = ptrs
            .get_mut(dest_pos)
            .ok_or_else(|| anyhow!("Ptr position {dest_pos} out of bounds"))?;
        *slot = ptr_val;
    }

    Ok((indexlast, ptrs))
}

fn inc_sort_order(
    in_ext: &[u8],
    length: usize,
    counts: &[usize; 256],
    off: usize,
    old_idxlast: usize,
    ptrs: &[u32],
) -> Result<(usize, Vec<u32>)> {
    let mut ct = *counts;
    let mut next_ptrs = vec![0u32; length];
    let mut last_ch = 0u8;

    for i in 0..=old_idxlast {
        let tmp = usize::try_from(
            *ptrs
                .get(i)
                .ok_or_else(|| anyhow!("Szip sort missing pointer at index {i}"))?,
        )
        .map_err(|e| anyhow!("Ptr conversion: {e}"))?;
        let ch = *in_ext
            .get(tmp.saturating_sub(off))
            .ok_or_else(|| anyhow!("Szip sort context index out of bounds"))?;
        last_ch = ch;
        let ch_idx = usize::from(ch);
        let pos = *ct
            .get(ch_idx)
            .ok_or_else(|| anyhow!("Invalid character index {ch_idx}"))?;
        if let Some(slot) = ct.get_mut(ch_idx) {
            *slot = slot.saturating_add(1);
        }
        let tmp_u32 = u32::try_from(tmp)
            .map_err(|e| anyhow!("Ptr conversion: {e}"))?;
        let slot = next_ptrs
            .get_mut(pos)
            .ok_or_else(|| anyhow!("Next pointer index {pos} out of bounds"))?;
        *slot = tmp_u32;
    }

    let new_idxlast = ct
        .get(usize::from(last_ch))
        .copied()
        .ok_or_else(|| anyhow!("Invalid character frequency index"))?
        .saturating_sub(1);

    for i in (old_idxlast.saturating_add(1))..length {
        let tmp = usize::try_from(
            *ptrs
                .get(i)
                .ok_or_else(|| anyhow!("Szip sort missing pointer at index {i}"))?,
        )
        .map_err(|e| anyhow!("Ptr conversion: {e}"))?;
        let ch = *in_ext
            .get(tmp.saturating_sub(off))
            .ok_or_else(|| anyhow!("Szip sort context index out of bounds"))?;
        let ch_idx = usize::from(ch);
        let pos = *ct
            .get(ch_idx)
            .ok_or_else(|| anyhow!("Invalid character index {ch_idx}"))?;
        if let Some(slot) = ct.get_mut(ch_idx) {
            *slot = slot.saturating_add(1);
        }
        let tmp_u32 = u32::try_from(tmp)
            .map_err(|e| anyhow!("Ptr conversion: {e}"))?;
        let slot = next_ptrs
            .get_mut(pos)
            .ok_or_else(|| anyhow!("Next pointer index {pos} out of bounds"))?;
        *slot = tmp_u32;
    }

    Ok((new_idxlast, next_ptrs))
}

fn finish_sort(
    in_ext: &[u8],
    length: usize,
    counts: &[usize; 256],
    old_idxlast: usize,
    ptrs: &[u32],
    inout: &mut [u8],
) -> Result<u32> {
    let mut ct = *counts;
    let mut last_ch = 0u8;
    let mut out_bytes = vec![0u8; length];

    for i in 0..=old_idxlast {
        let tmp = usize::try_from(
            *ptrs
                .get(i)
                .ok_or_else(|| anyhow!("Szip sort missing pointer at index {i}"))?,
        )
        .map_err(|e| anyhow!("Ptr conversion: {e}"))?;
        let ch = *in_ext
            .get(tmp.saturating_sub(1))
            .ok_or_else(|| anyhow!("Szip sort context index out of bounds"))?;
        last_ch = ch;
        let ch_idx = usize::from(ch);
        let pos = *ct
            .get(ch_idx)
            .ok_or_else(|| anyhow!("Invalid character index {ch_idx}"))?;
        if let Some(slot) = ct.get_mut(ch_idx) {
            *slot = slot.saturating_add(1);
        }
        let sym = *in_ext
            .get(tmp)
            .ok_or_else(|| anyhow!("Szip sort symbol index out of bounds"))?;
        let slot = out_bytes
            .get_mut(pos)
            .ok_or_else(|| anyhow!("Output position {pos} out of bounds"))?;
        *slot = sym;
    }

    let final_idxlast = ct
        .get(usize::from(last_ch))
        .copied()
        .ok_or_else(|| anyhow!("Invalid character frequency index"))?
        .saturating_sub(1);

    for i in (old_idxlast.saturating_add(1))..length {
        let tmp = usize::try_from(
            *ptrs
                .get(i)
                .ok_or_else(|| anyhow!("Szip sort missing pointer at index {i}"))?,
        )
        .map_err(|e| anyhow!("Ptr conversion: {e}"))?;
        let ch = *in_ext
            .get(tmp.saturating_sub(1))
            .ok_or_else(|| anyhow!("Szip sort context index out of bounds"))?;
        let ch_idx = usize::from(ch);
        let pos = *ct
            .get(ch_idx)
            .ok_or_else(|| anyhow!("Invalid character index {ch_idx}"))?;
        if let Some(slot) = ct.get_mut(ch_idx) {
            *slot = slot.saturating_add(1);
        }
        let sym = *in_ext
            .get(tmp)
            .ok_or_else(|| anyhow!("Szip sort symbol index out of bounds"))?;
        let slot = out_bytes
            .get_mut(pos)
            .ok_or_else(|| anyhow!("Output position {pos} out of bounds"))?;
        *slot = sym;
    }

    inout.copy_from_slice(&out_bytes);
    u32::try_from(final_idxlast).map_err(|e| anyhow!("Indexlast conversion: {e}"))
}

/// Sorts block using general context order N (order >= 3).
pub fn sort_general(inout: &mut [u8], order: usize) -> Result<u32> {
    let length = inout.len();
    if length <= order {
        return Ok(0);
    }

    let mut in_ext = vec![0u8; length.saturating_add(order)];
    let Some(dest) = in_ext.get_mut(..length) else {
        bail!("Failed to slice extended buffer");
    };
    dest.copy_from_slice(inout);

    let mut counts = [0usize; 256];
    let mut o2counts = vec![0usize; 0x10000];

    let last_byte = *inout
        .last()
        .ok_or_else(|| anyhow!("Input buffer empty for general sort"))?;
    let mut context = usize::from(last_byte) << 8;

    for &b in &*inout {
        context = (context >> 8) | (usize::from(b) << 8);
        if let Some(slot) = counts.get_mut(usize::from(b)) {
            *slot = slot.saturating_add(1);
        }
        if let Some(slot) = o2counts.get_mut(context) {
            *slot = slot.saturating_add(1);
        }
    }

    let mut sum = length;
    for i in (0..0x10000).rev() {
        let cnt = *o2counts
            .get(i)
            .ok_or_else(|| anyhow!("Missing o2counts counter at {i}"))?;
        sum = sum.saturating_sub(cnt);
        if let Some(slot) = o2counts.get_mut(i) {
            *slot = sum;
        }
    }

    sum = length;
    for i in (0..256).rev() {
        let cnt = *counts
            .get(i)
            .ok_or_else(|| anyhow!("Missing counts counter at {i}"))?;
        sum = sum.saturating_sub(cnt);
        if let Some(slot) = counts.get_mut(i) {
            *slot = sum;
        }
    }

    let (mut indexlast, mut ptrs) = sort_order2(&mut in_ext, length, order, o2counts)?;

    if order > 3 {
        for off in (2..=(order.saturating_sub(2))).rev() {
            let (next_idx, next_ptrs) =
                inc_sort_order(&in_ext, length, &counts, off, indexlast, &ptrs)?;
            indexlast = next_idx;
            ptrs = next_ptrs;
        }
    }

    finish_sort(&in_ext, length, &counts, indexlast, &ptrs, inout)
}

fn make_order2(flags: &mut [u8], input: &[u8], counts: &[usize; 256]) -> Result<()> {
    let mut ct = *counts;
    for i in 0..256 {
        let pos = *ct
            .get(i)
            .ok_or_else(|| anyhow!("Missing frequency bucket at {i}"))?;
        set_bit(flags, pos);
    }
    let mut j_pos = 0usize;
    for i in 0usize..255 {
        let k_limit = *counts
            .get(i.saturating_add(1))
            .ok_or_else(|| anyhow!("Missing frequency bucket limit"))?;
        while j_pos < k_limit {
            let b = usize::from(
                *input
                    .get(j_pos)
                    .ok_or_else(|| anyhow!("Input byte out of bounds at {j_pos}"))?,
            );
            if let Some(slot) = ct.get_mut(b) {
                *slot = slot.saturating_add(1);
            }
            j_pos = j_pos.saturating_add(1);
        }
        for k in 0..256 {
            let pos = *ct
                .get(k)
                .ok_or_else(|| anyhow!("Missing cumulative count at {k}"))?;
            set_bit(flags, pos);
        }
    }
    Ok(())
}

fn increase_order(
    flags_in: &[u8],
    flags_out: &mut [u8],
    input: &[u8],
    counts: &[usize; 256],
) -> Result<()> {
    flags_out.fill(0);
    let mut ct_inc = *counts;
    let mut context_start = 0usize;
    let mut last_seen = [usize::MAX; 256];
    for (i, &b_val) in input.iter().enumerate() {
        if get_bit(flags_in, i) {
            context_start = i;
        }
        let b = usize::from(b_val);
        if last_seen.get(b) != Some(&context_start) {
            if let Some(ls_slot) = last_seen.get_mut(b) {
                *ls_slot = context_start;
            }
            let pos = *ct_inc
                .get(b)
                .ok_or_else(|| anyhow!("Missing cumulative count for byte {b}"))?;
            set_bit(flags_out, pos);
        }
        if let Some(slot) = ct_inc.get_mut(b) {
            *slot = slot.saturating_add(1);
        }
    }
    Ok(())
}

fn make_table(
    flags: &[u8],
    table: &mut [u32],
    input: &[u8],
    counts: &[usize; 256],
    length: usize,
) -> Result<()> {
    let mut ct_tbl = *counts;
    let mut context_start = 0usize;
    let mut first_seen = [0usize; 256];
    for (i, &b_val) in input.iter().enumerate() {
        if get_bit(flags, i) {
            context_start = i;
        }
        let b = usize::from(b_val);
        let first = *first_seen
            .get(b)
            .ok_or_else(|| anyhow!("First seen index out of bounds"))?;
        if first <= context_start {
            if let Some(slot) = table.get_mut(i) {
                let ct_val = *ct_tbl
                    .get(b)
                    .ok_or_else(|| anyhow!("Count table index out of bounds"))?;
                *slot = u32::try_from(ct_val)
                    .map_err(|e| anyhow!("Table entry conversion: {e}"))?;
            }
            if let Some(slot) = first_seen.get_mut(b) {
                *slot = i.saturating_add(1);
            }
        } else if let Some(slot) = table.get_mut(i) {
            let indirect_val = u32::try_from(first.saturating_sub(1))
                .map_err(|e| anyhow!("Indirect pointer conversion: {e}"))?
                | INDIRECT;
            *slot = indirect_val;
        }
        if let Some(slot) = ct_tbl.get_mut(b) {
            *slot = slot.saturating_add(1);
        }
    }
    if let Some(slot) = table.get_mut(length) {
        *slot = INDIRECT;
    }
    Ok(())
}

fn unsort_traverse(
    input: &[u8],
    output: &mut [u8],
    table: &mut [u32],
    indexlast: u32,
    length: usize,
) -> Result<()> {
    let mut j_idx = usize::try_from(indexlast)
        .map_err(|e| anyhow!("Indexlast conversion: {e}"))?;
    for slot in output.iter_mut().take(length) {
        let tmp = *table
            .get(j_idx)
            .ok_or_else(|| anyhow!("Szip unsort table index out of bounds: {j_idx}"))?;
        if (tmp & INDIRECT) != 0 {
            let target_idx = usize::try_from(tmp & !INDIRECT)
                .map_err(|e| anyhow!("Corrupt indirect index: {e}"))?;
            let next_j = usize::try_from(
                *table
                    .get(target_idx)
                    .ok_or_else(|| anyhow!("Corrupt indirect target index: {target_idx}"))?,
            )
            .map_err(|e| anyhow!("Table entry conversion error: {e}"))?;
            let entry = table
                .get_mut(target_idx)
                .ok_or_else(|| anyhow!("Missing indirect target table entry"))?;
            *entry = entry.saturating_add(1);
            j_idx = next_j;
        } else {
            let entry = table
                .get_mut(j_idx)
                .ok_or_else(|| anyhow!("Missing table entry at {j_idx}"))?;
            *entry = entry.saturating_add(1);
            j_idx = usize::try_from(tmp).map_err(|e| anyhow!("Table entry conversion error: {e}"))?;
        }
        *slot = *input
            .get(j_idx)
            .ok_or_else(|| anyhow!("Szip unsort input index out of bounds: {j_idx}"))?;
    }

    let final_j = u32::try_from(j_idx).map_err(|e| anyhow!("Final index conversion: {e}"))?;
    if final_j != indexlast {
        bail!("Szip general unsort cycle verification failed");
    }

    Ok(())
}

/// Unsorts general context order block (order >= 3).
pub fn unsort_general(
    input: &[u8],
    output: &mut [u8],
    indexlast: u32,
    order: usize,
) -> Result<()> {
    let length = input.len();
    if length <= order {
        output.copy_from_slice(input);
        return Ok(());
    }
    if output.len() < length {
        bail!("Output buffer smaller than input buffer for unsort_general");
    }

    let mut counts = [0usize; 256];
    for &b in input {
        if let Some(slot) = counts.get_mut(usize::from(b)) {
            *slot = slot.saturating_add(1);
        }
    }

    let mut j = length;
    for i in (0..256).rev() {
        let cnt = *counts
            .get(i)
            .ok_or_else(|| anyhow!("Missing frequency count bucket at {i}"))?;
        j = j.saturating_sub(cnt);
        if let Some(slot) = counts.get_mut(i) {
            *slot = j;
        }
    }

    let flags_size = (length.saturating_add(8)) >> 3;
    let mut flags1 = vec![0u8; flags_size];
    make_order2(&mut flags1, input, &counts)?;

    let mut flags2 = vec![0u8; flags_size];
    for _ in 2..(order.saturating_sub(1)) {
        increase_order(&flags1, &mut flags2, input, &counts)?;
        std::mem::swap(&mut flags1, &mut flags2);
    }

    let mut table = vec![0u32; length.saturating_add(1)];
    make_table(&flags1, &mut table, input, &counts, length)?;

    unsort_traverse(input, output, &mut table, indexlast, length)
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
    fn test_sort_order4_basic() {
        let orig = b"Hello, Szip 1.11+ compression!";
        let mut buf = orig.to_vec();
        let idx = sort_order4(&mut buf).unwrap();
        assert_eq!(idx, 20);
        let mut out = vec![0u8; orig.len()];
        unsort_general(&buf, &mut out, idx, 4).unwrap();
        assert_eq!(out.as_slice(), orig.as_slice());
    }

    #[crate::ctb_test]
    fn test_sort_general_all_orders() {
        let orig = b"Hello, Szip 1.11+ compression!";
        for order in [3, 4, 5, 6] {
            let mut buf = orig.to_vec();
            let idx = sort_general(&mut buf, order).unwrap();
            assert_eq!(idx, 20);
            let mut out = vec![0u8; orig.len()];
            unsort_general(&buf, &mut out, idx, order).unwrap();
            assert_eq!(out.as_slice(), orig.as_slice());
        }
    }

    #[crate::ctb_test]
    fn test_sort_bwt_basic() {
        let orig = b"Hello, Szip 1.11+ compression!";
        let mut buf = orig.to_vec();
        let idx = sort_bwt(&mut buf).unwrap();
        assert_eq!(idx, 3);
        let mut out = vec![0u8; orig.len()];
        unsort_bwt(&buf, &mut out, idx).unwrap();
        assert_eq!(out.as_slice(), orig.as_slice());
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