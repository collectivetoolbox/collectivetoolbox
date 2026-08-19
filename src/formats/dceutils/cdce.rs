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

//! CDCE character set decoder and data conversion utilities.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::anyhow;

use ctb_formats_utilities::FormatLog;

use crate::dce_convert;
use crate::tables::get_tables;
use crate::tools::explode_escaped;

pub fn convert_legacy_cdce_to_dc(
    data: &[u8],
    strict: bool,
    log: &mut FormatLog,
) -> Result<String> {
    let utf32_bytes = ctb_formats_encoding::unicode::utf8_to_utf32be(data)
        .map_err(|e| {
            log.error(&format!("iconv conversion failed: {e}"));
            anyhow!("iconv conversion failed: {e}")
        })?;

    let chunks = utf32_bytes.chunks_exact(4);
    if !chunks.remainder().is_empty() {
        log.error("UTF-32BE payload length is not a multiple of 4");
        bail!("UTF-32BE payload length is not a multiple of 4");
    }

    let codepoints: Vec<u32> = chunks
        .map(|chunk| {
            let Ok(bytes) = <[u8; 4]>::try_from(chunk) else {
                unreachable!("chunks_exact(4) guarantees 4-byte chunks");
            };
            u32::from_be_bytes(bytes)
        })
        .collect();

    let tables = get_tables();
    let mut txt = String::new();
    let mut i = 0_usize;

    while i < codepoints.len() {
        let cp = *codepoints
            .get(i)
            .context("codepoints index out of bounds")?;
        if cp == 0x40 {
            // '@'
            // 1-character CDCE condition
            let mut matched_1char = false;
            let i_plus_2 = i.saturating_add(2);
            if i_plus_2 < codepoints.len()
                && codepoints.get(i_plus_2) == Some(&0x40)
            {
                let next_cp = *codepoints
                    .get(i.saturating_add(1))
                    .context("codepoints index out of bounds")?;
                if (0x31..=0x39).contains(&next_cp) {
                    // '1'..='9'
                    let digit_char = char::from_u32(next_cp)
                        .ok_or_else(|| anyhow::anyhow!("Invalid codepoint 0x{next_cp:X}"))?;
                    txt.push_str(&format!("{digit_char},"));
                    i = i.saturating_add(3);
                    matched_1char = true;
                }
            }

            if !matched_1char {
                // 2-character CDCE condition
                let mut matched_2char = false;
                let i_plus_3 = i.saturating_add(3);
                if i_plus_3 < codepoints.len()
                    && codepoints.get(i_plus_3) == Some(&0x40)
                {
                    let cp1 = *codepoints
                        .get(i.saturating_add(1))
                        .context("codepoints index out of bounds")?;
                    let cp2 = *codepoints
                        .get(i.saturating_add(2))
                        .context("codepoints index out of bounds")?;
                    if (0x30..=0x39).contains(&cp1)
                        && (0x30..=0x39).contains(&cp2)
                    {
                        let d1 = u8::try_from(cp1.saturating_sub(0x30))?;
                        let d2 = u8::try_from(cp2.saturating_sub(0x30))?;
                        let val = d1.saturating_mul(10).saturating_add(d2);
                        if val > 0 && val < 13 {
                            txt.push_str(&format!("{val},"));
                            i = i.saturating_add(4);
                            matched_2char = true;
                        }
                    }
                }

                if !matched_2char {
                    if strict {
                        log.error(&format!(
                            "CDCE decoding error: invalid escape sequence at codepoint index {i}"
                        ));
                        // format txt like PHP did, then return decoding error
                        let mut cleaned = txt.trim_end_matches(',').to_string();
                        while cleaned.contains(",,") {
                            cleaned = cleaned.replace(",,", ",0,");
                        }
                        return Ok(format!("{cleaned}… CDCE decoding error!"));
                    }
                    log.warn(&format!(
                        "Warning: Unicode recovery attempted for corrupted CDCE sequence at codepoint index {i}"
                    ));
                    // Recover using Lossy Unicode map for '@' (0x40)
                    if let Some(val) = tables.dc_map_unicode_lossy.get("40") {
                        txt.push_str(&format!("{val},"));
                    }
                    i = i.saturating_add(1);
                }
            }
        } else {
            let key = format!("{cp:X}");
            if let Some(val) = tables.dc_map_unicode_lossy.get(&key) {
                if !val.is_empty() {
                    txt.push_str(&format!("{val},"));
                }
            }
            i = i.saturating_add(1);
        }
    }

    Ok(txt)
}

pub fn convert_dc_to_legacy_cdce_output(
    data: &str,
    _log: &mut FormatLog,
) -> Result<Vec<u8>> {
    let dc_array = explode_escaped(',', data);
    let tables = get_tables();
    let mut txt = String::new();

    for dc_id in dc_array {
        if tables.cdce_html_legacy.contains_key(&dc_id) {
            txt.push_str(&format!("@{dc_id}@"));
        } else {
            let utf8_res = dce_convert(dc_id.as_bytes(), "dc", "utf8")?;
            let utf8_str = String::from_utf8(utf8_res)
                .context("invalid UTF-8 from conversion")?;
            txt.push_str(&utf8_str);
        }
    }

    Ok(txt.into_bytes())
}
