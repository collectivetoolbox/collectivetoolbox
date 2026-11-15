// SPDX-License-Identifier: AGPL-3.0-or-later
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

//! Mixed-mode (`s` / `DcMst`) and pure-Dc (`dcs`) string semantics for StageR 3.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::dc::{
    DC_BASE64_END, DC_BASE64_PADDING, DC_BASE64_START,
    DC_END_ENCAPSULATION_BINARY, DC_ESCAPE, DC_START_ENCAPSULATION_BINARY,
};
use crate::dc_char::{DcChar, DcCharExt};
use ctb_formats_dctf::len_dctf;

/// Binary encapsulation frame header size without metadata: `DC_203_BYTES` (4)
/// + type byte (1) + 128-bit payload length (16).
const RAW_BINARY_HEADER_BYTES: u64 = 21;

/// Binary encapsulation frame header size with metadata: `DC_203_BYTES` (4) +
/// type byte (1) + 128-bit payload length (16) + 128-bit metadata length (16).
const TYPED_BINARY_HEADER_BYTES: u64 = 37;

/// Returns `true` iff `gid` is a valid Document Character scalar outside the
/// unpaired UTF-16 surrogate range `0xD800..=0xDFFF`.
#[must_use]
pub const fn is_valid_dc_u128(gid: u128) -> bool {
    gid < 0xD800 || gid > 0xDFFF
}

fn is_b64_digit_or_pad(ch: DcChar) -> bool {
    ch == DC_BASE64_PADDING
        || (ch.to_u128() >= DC_BASE64_START.to_u128()
            && ch.to_u128() <= DC_BASE64_END.to_u128())
}

fn decoded_base64_len(tokens: &[DcChar]) -> Result<u64> {
    anyhow::ensure!(
        tokens.len().is_multiple_of(4),
        "Invalid Base64 encapsulation length {} inside binary frame",
        tokens.len()
    );
    if tokens.is_empty() {
        return Ok(0);
    }
    let mut padding = 0u64;
    let last_idx = tokens.len().saturating_sub(1);
    for (idx, &tok) in tokens.iter().enumerate() {
        if tok == DC_BASE64_PADDING {
            anyhow::ensure!(
                idx >= last_idx.saturating_sub(1),
                "Misplaced Base64 padding inside binary encapsulation frame"
            );
            padding = padding.saturating_add(1);
        } else {
            anyhow::ensure!(
                tok.to_u128() >= DC_BASE64_START.to_u128()
                    && tok.to_u128() <= DC_BASE64_END.to_u128(),
                "Non-Base64 token {tok} inside binary encapsulation payload"
            );
        }
    }
    let groups = u64::try_from(tokens.len() >> 2)
        .context("Binary Base64 length overflow")?;
    Ok(groups.saturating_mul(3).saturating_sub(padding))
}

fn parse_binary_frame(chars: &[DcChar], start: usize) -> Result<(usize, u64)> {
    let mut idx = start.saturating_add(1);
    let mut inner: Vec<DcChar> = Vec::new();
    let mut found_end = false;
    while let Some(&ch) = chars.get(idx) {
        idx = idx.saturating_add(1);
        if ch == DC_END_ENCAPSULATION_BINARY {
            found_end = true;
            break;
        }
        anyhow::ensure!(
            is_valid_dc_u128(ch.to_u128()),
            "Surrogate Dc {} inside binary encapsulation frame",
            ch
        );
        inner.push(ch);
    }
    anyhow::ensure!(
        found_end,
        "Unclosed Dc 203 binary encapsulation frame (missing Dc 204)"
    );

    // Reason for fallback: an encapsulation frame with an empty base64 payload has no base64 digit or pad tokens, so all inner tokens are metadata and the payload slice starts at inner.len().
    let first_b64 = inner
        .iter()
        .position(|&c| is_b64_digit_or_pad(c))
        .unwrap_or(inner.len());
    let (meta_tokens, b64_tokens) = inner.split_at(first_b64);
    let b64_decoded_len = decoded_base64_len(b64_tokens)?;
    let frame_len = if meta_tokens.is_empty() {
        RAW_BINARY_HEADER_BYTES.saturating_add(b64_decoded_len)
    } else {
        let mut meta_bytes = 0u64;
        for &m in meta_tokens {
            let l = u64::try_from(len_dctf(m.to_u128()))
                .context("len_dctf overflow")?;
            meta_bytes = meta_bytes.saturating_add(l);
        }
        TYPED_BINARY_HEADER_BYTES
            .saturating_add(meta_bytes)
            .saturating_add(b64_decoded_len)
    };
    Ok((idx, frame_len))
}

/// Validates a mixed-mode `s` (`DcMst`) `DcChar` sequence and computes its
/// encoded byte length (`byteLen`).
///
/// Plain scalars and escaped literals are measured using `DcTF` (`UTF-8e-128`)
/// byte lengths, while unescaped `Dc 203..Dc 204` binary encapsulation frames
/// are validated and measured using `DcMst` binary chunk header + payload
/// sizes (`0` for empty strings).
pub fn validate_and_measure_s_byte_len(chars: &[DcChar]) -> Result<u64> {
    let mut idx = 0usize;
    let mut total = 0u64;
    while let Some(&ch) = chars.get(idx) {
        anyhow::ensure!(
            is_valid_dc_u128(ch.to_u128()),
            "Invalid surrogate Dc {} in mixed-mode string",
            ch
        );
        if ch == DC_ESCAPE {
            let next_idx = idx.saturating_add(1);
            let escaped = *chars
                .get(next_idx)
                .context("Trailing Dc 255 escape at end of mixed-mode string")?;
            anyhow::ensure!(
                is_valid_dc_u128(escaped.to_u128()),
                "Invalid surrogate Dc {} after Dc 255 escape",
                escaped
            );
            let esc_len = u64::try_from(len_dctf(DC_ESCAPE.to_u128()))
                .context("len_dctf overflow")?;
            let val_len = u64::try_from(len_dctf(escaped.to_u128()))
                .context("len_dctf overflow")?;
            total = total.saturating_add(esc_len).saturating_add(val_len);
            idx = idx.saturating_add(2);
            continue;
        }
        if ch == DC_START_ENCAPSULATION_BINARY {
            let (next_idx, frame_bytes) = parse_binary_frame(chars, idx)?;
            total = total.saturating_add(frame_bytes);
            idx = next_idx;
            continue;
        }
        anyhow::ensure!(
            ch != DC_END_ENCAPSULATION_BINARY,
            "Unmatched Dc 204 (End binary encapsulation) in mixed-mode string"
        );
        let ch_len = u64::try_from(len_dctf(ch.to_u128()))
            .context("len_dctf overflow")?;
        total = total.saturating_add(ch_len);
        idx = idx.saturating_add(1);
    }
    Ok(total)
}

/// Validates and converts an `s` (`DcMst`) `DcChar` slice into a pure `dcs`
/// sequence (`Err(())` corresponds to `error:invalidEncoding` when binary
/// encapsulation frames or invalid escapes are present).
pub fn dcs_from_s(chars: &[DcChar]) -> std::result::Result<Vec<DcChar>, ()> {
    let mut out = Vec::with_capacity(chars.len());
    let mut idx = 0usize;
    while let Some(&ch) = chars.get(idx) {
        if !is_valid_dc_u128(ch.to_u128()) {
            return Err(());
        }
        if ch == DC_ESCAPE {
            let next_idx = idx.saturating_add(1);
            let Some(&escaped) = chars.get(next_idx) else {
                return Err(());
            };
            if !is_valid_dc_u128(escaped.to_u128()) {
                return Err(());
            }
            out.push(escaped);
            idx = idx.saturating_add(2);
            continue;
        }
        if ch == DC_START_ENCAPSULATION_BINARY
            || ch == DC_END_ENCAPSULATION_BINARY
        {
            return Err(());
        }
        out.push(ch);
        idx = idx.saturating_add(1);
    }
    Ok(out)
}

/// Widens a pure `dcs` `DcChar` slice into an `s` (`DcMst`) sequence, escaping
/// any structural `Dc 255` / `Dc 203` / `Dc 204` markers so `dcsFromS(sFromDcs(x)) == ok x`.
#[must_use]
pub fn s_from_dcs(chars: &[DcChar]) -> Vec<DcChar> {
    let mut out = Vec::with_capacity(chars.len());
    for &ch in chars {
        if ch == DC_ESCAPE
            || ch == DC_START_ENCAPSULATION_BINARY
            || ch == DC_END_ENCAPSULATION_BINARY
        {
            out.push(DC_ESCAPE);
        }
        out.push(ch);
    }
    out
}

/// Segments a validated mixed-mode `s` slice into atomic units (single `DcChar`
/// or `Dc 255` escape pair or `Dc 203..Dc 204` binary frame) so `explode` never
/// splits inside an escape pair or encapsulated binary chunk.
pub fn s_atomic_units(chars: &[DcChar]) -> Result<Vec<Vec<DcChar>>> {
    let mut units = Vec::new();
    let mut idx = 0usize;
    while let Some(&ch) = chars.get(idx) {
        anyhow::ensure!(
            is_valid_dc_u128(ch.to_u128()),
            "Invalid surrogate Dc {} in mixed-mode string",
            ch
        );
        if ch == DC_ESCAPE {
            let next_idx = idx.saturating_add(1);
            let escaped = *chars
                .get(next_idx)
                .context("Trailing Dc 255 escape at end of mixed-mode string")?;
            units.push(vec![DC_ESCAPE, escaped]);
            idx = idx.saturating_add(2);
        } else if ch == DC_START_ENCAPSULATION_BINARY {
            let (next_idx, _) = parse_binary_frame(chars, idx)?;
            let slice = chars
                .get(idx..next_idx)
                .context("Binary frame slice bounds error")?;
            units.push(slice.to_vec());
            idx = next_idx;
        } else {
            anyhow::ensure!(
                ch != DC_END_ENCAPSULATION_BINARY,
                "Unmatched Dc 204 (End binary encapsulation) in mixed-mode string"
            );
            units.push(vec![ch]);
            idx = idx.saturating_add(1);
        }
    }
    Ok(units)
}

/// Splits a slice of elements around non-overlapping occurrences of `sep`.
///
/// When `sep` is empty, each individual element becomes a singleton sequence,
/// returning an empty `Vec` when `input` itself is empty.
#[must_use]
pub fn explode_slice<T: Clone + PartialEq>(
    sep: &[T],
    input: &[T],
) -> Vec<Vec<T>> {
    if sep.is_empty() {
        if input.is_empty() {
            return Vec::new();
        }
        return input.iter().map(|el| vec![el.clone()]).collect();
    }
    let mut out = Vec::new();
    let mut cursor = 0usize;
    let sep_len = sep.len();
    while let Some(tail) = input.get(cursor..) {
        if tail.len() < sep_len {
            out.push(tail.to_vec());
            return out;
        }
        let mut found_at = None;
        for offset in 0..=tail.len().saturating_sub(sep_len) {
            if tail.get(offset..offset.saturating_add(sep_len)) == Some(sep) {
                found_at = Some(offset);
                break;
            }
        }
        if let Some(offset) = found_at {
            if let Some(chunk) = tail.get(..offset) {
                out.push(chunk.to_vec());
            }
            cursor = cursor.saturating_add(offset).saturating_add(sep_len);
        } else {
            out.push(tail.to_vec());
            return out;
        }
    }
    out.push(Vec::new());
    out
}

/// Concatenates slices in `parts` interspersed with `sep`.
#[must_use]
pub fn implode_slices<T: Clone>(sep: &[T], parts: &[Vec<T>]) -> Vec<T> {
    let mut out = Vec::new();
    for (idx, part) in parts.iter().enumerate() {
        if idx > 0 {
            out.extend_from_slice(sep);
        }
        out.extend_from_slice(part);
    }
    out
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
    fn test_s_from_dcs_roundtrip_with_binary_marker() {
        let pure = vec![
            DcChar::from(b'a'),
            DC_START_ENCAPSULATION_BINARY,
            DC_ESCAPE,
            DC_END_ENCAPSULATION_BINARY,
            DcChar::from(b'z'),
        ];
        let widened = s_from_dcs(&pure);
        assert!(validate_and_measure_s_byte_len(&widened).unwrap() > 0);
        assert_eq!(dcs_from_s(&widened), Ok(pure));
    }

    #[crate::ctb_test]
    fn test_explode_and_implode_roundtrip() {
        let sep = vec![b':'];
        let input = b"alpha:beta:".to_vec();
        let parts = explode_slice(&sep, &input);
        assert_eq!(
            parts,
            vec![b"alpha".to_vec(), b"beta".to_vec(), Vec::new()]
        );
        assert_eq!(implode_slices(&sep, &parts), input);
    }
}
