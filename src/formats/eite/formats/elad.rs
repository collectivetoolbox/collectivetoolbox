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

//! ELAD (`.elad` / `.emd`) short-Dc ASCII document conversions.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::Result;

use crate::dc::DC_REPLACEMENT_UNAVAIL_DC;
use crate::formats::FormatLog;
use crate::formats::ascii::dca_from_ascii;

/// Parses an ELAD (`.elad` / `.emd`) byte slice into a short-Dc `u32` array:
/// printable ASCII characters other than `@` (`0x40`) map via [`dca_from_ascii`],
/// while `@<decimal_short_dc> ` (terminated by a single space `0x20`) decodes to
/// the specified short Dc ID.
pub fn dca_from_elad(content: &[u8]) -> Result<(Vec<u32>, FormatLog)> {
    let mut out = Vec::with_capacity(content.len());
    let mut log = FormatLog::default();
    let mut i = 0usize;
    while let Some(&b) = content.get(i) {
        if b == b'@' {
            let digits_start = i.saturating_add(1);
            let mut j = digits_start;
            while content.get(j).is_some_and(u8::is_ascii_digit) {
                j = j.saturating_add(1);
            }
            if j > digits_start && content.get(j) == Some(&b' ') {
                let digit_slice = content
                    .get(digits_start..j)
                    .context("ELAD short Dc digit span out of bounds")?;
                let digit_str = std::str::from_utf8(digit_slice)?;
                match digit_str.parse::<u32>() {
                    Ok(short_id) if short_id <= 1_114_111 => {
                        out.push(short_id);
                        i = j.saturating_add(1);
                        continue;
                    }
                    _ => {
                        log.error(&format!(
                            "Invalid ELAD short Dc tag '@{digit_str} ' at byte {i}"
                        ));
                    }
                }
            } else {
                log.error(&format!(
                    "Malformed ELAD '@' escape at byte {i}: expected '@<short_dc> '"
                ));
            }
            out.push(DC_REPLACEMENT_UNAVAIL_DC.to_short()?);
            i = i.saturating_add(1);
            continue;
        }
        let (chunk_dca, chunk_log) = dca_from_ascii(&[b])?;
        log.merge(&chunk_log);
        out.extend(chunk_dca);
        i = i.saturating_add(1);
    }
    Ok((out, log))
}

/// Convert to “Elad” – placeholder (returns empty per original stub).
pub fn dca_to_elad(_dc_array: &[u32]) -> Result<Vec<u8>> {
    // Original had:
    // intArrayRes = [];
    // assertByteArray(intArrayRes);
    Ok(Vec::new())
}
