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

//! DCE 3.01a specification character set and codepage conversion.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::anyhow;
use ctb_formats_hexdump::hex2bin;

use ctb_formats_utilities::FormatLog;

use crate::dce_convert_internal;
use crate::tables::get_tables;
use crate::tools::explode_escaped;

pub fn convert_3_01a_to_dc(data: &[u8], log: &mut FormatLog) -> Result<String> {
    let hex = bin2hex(data);
    if !hex.starts_with("444345650201") {
        log.error("This document is not stored using the specified format.");
        bail!("This document is not stored using the specified format.");
    }
    if hex.len() < 14 || hex.get(12..14) != Some("02") {
        log.error(
            "This document is not stored using the specified version of DCE.",
        );
        bail!(
            "This document is not stored using the specified version of DCE."
        );
    }

    let tables = get_tables();
    let mut txt = String::new();
    let mut state = "Core".to_string();
    let mut counter = 14_usize;
    let mut append = String::new();

    while counter < hex.len() {
        let end_two = counter.saturating_add(2);
        if end_two <= hex.len() {
            if let Some(slice) = hex.get(counter..end_two) {
                let byte_hex = slice.to_uppercase();
                match state.as_str() {
                    "Core" => {
                        if let Some(val) =
                            tables.dc_map_dce3_01a_core.get(&byte_hex)
                        {
                            if let Some(stripped) = val.strip_prefix('>') {
                                state = stripped.to_string();
                                append.clear();
                            } else {
                                append = format!("{val},");
                            }
                        } else {
                            append.clear();
                        }
                    }
                    "Variant_Selectors" => {
                        if byte_hex == "FD" || byte_hex == "FE" {
                            state = "Core".to_string();
                            append.clear();
                        } else if let Some(val) = tables
                            .dc_map_dce3_01a_variant_selectors
                            .get(&byte_hex)
                        {
                            append = format!("{val},");
                        } else {
                            append.clear();
                        }
                    }
                    "Semantic_Records" => {
                        if byte_hex == "FD" || byte_hex == "FE" {
                            state = "Core".to_string();
                            append.clear();
                        } else if let Some(val) = tables
                            .dc_map_dce3_01a_semantic_records
                            .get(&byte_hex)
                        {
                            append = format!("{val},");
                        } else {
                            append.clear();
                        }
                    }
                    "Mathematics" => {
                        if byte_hex == "FD" || byte_hex == "FE" {
                            state = "Core".to_string();
                            append.clear();
                        } else if let Some(val) =
                            tables.dc_map_dce3_01a_mathematics.get(&byte_hex)
                        {
                            append = format!("{val},");
                        } else {
                            append.clear();
                        }
                    }
                    "Punctuation_and_Whitespace"
                    | "Whitespace_and_Punctuation" => {
                        // Corrected PHP bug: The PHP state name for whitespace & punctuation is defined as
                        // 'Punctuation_and_Whitespace' (C8 -> >Punctuation_and_Whitespace). But the case statement
                        // in PHP's switch was incorrectly written as 'Whitespace_and_Punctuation'.
                        // We have corrected this to support 'Punctuation_and_Whitespace' (retaining 'Whitespace_and_Punctuation'
                        // for safety/compatibility) so that the state transitions correctly instead of being bypassed.
                        if byte_hex == "FD" || byte_hex == "FE" {
                            state = "Core".to_string();
                            append.clear();
                        } else if let Some(val) = tables
                            .dc_map_dce3_01a_punctuation_and_whitespace
                            .get(&byte_hex)
                        {
                            append = format!("{val},");
                        } else {
                            append.clear();
                        }
                    }
                    _ => {}
                }
            }
        }

        let end_four = counter.saturating_add(4);
        if end_four <= hex.len() {
            if let Some(four_hex) = hex.get(counter..end_four) {
                if four_hex == "fd03" {
                    break;
                }
            }
        }

        txt.push_str(&append);
        counter = counter.saturating_add(2);
    }

    if txt.len() >= 4 {
        if let Some(sub) = txt.get(3..txt.len().saturating_sub(1)) {
            txt = sub.to_string();
        } else {
            txt.clear();
        }
    } else {
        txt.clear();
    }

    Ok(txt)
}

pub fn convert_3_01a_raw_to_dc(
    data: &[u8],
    log: &mut FormatLog,
) -> Result<String> {
    let hex = bin2hex(data);
    if !hex.starts_with("80") {
        log.error("This document is not stored using the specified format.");
        bail!("This document is not stored using the specified format.");
    }

    let mut wrapped = Vec::with_capacity(data.len().saturating_add(10));
    wrapped
        .extend_from_slice(&[0x44, 0x43, 0x45, 0x65, 0x02, 0x01, 0x02, 0xFD]);
    wrapped.extend_from_slice(data);
    wrapped.extend_from_slice(&[0xFD, 0x03]);

    dce_convert_internal(&wrapped, "3_01a", "dc", log)
        .map(|b| String::from_utf8_lossy(&b).into_owned())
}

// Corrected PHP bug: The original PHP function used the undefined variable `$dc`
// instead of the function parameter `$data` (which was called `$dc` on the call to DcMapSendSimple).
// Thus, DcMapSendSimple was always called with an empty string, returning an empty string.
// We have corrected this to use the passed `data` parameter to correctly encode the output format.
pub fn convert_dc_to_3_01a_output(
    data: &str,
    log: &mut FormatLog,
) -> Result<Vec<u8>> {
    let bounded = crate::convert_dc_to_dc_output(data, log);
    let encoded = dc_map_send_simple_with_log(&bounded, log);
    let hex_str = format!("44434565020102fd{encoded}fd03");
    let bytes = hex2bin(&hex_str).map_err(|e| {
        log.error(&format!("invalid hex string generated: {e}"));
        anyhow!("invalid hex string generated: {e}")
    })?;
    Ok(bytes)
}

pub fn convert_dc_to_3_01a_raw_output(
    data: &str,
    log: &mut FormatLog,
) -> Result<Vec<u8>> {
    let full = dce_convert_internal(data.as_bytes(), "dc", "3_01a", log)?;
    if full.len() >= 10 {
        let sub = full
            .get(8..full.len().saturating_sub(2))
            .context("substring slice")?;
        Ok(sub.to_vec())
    } else {
        bail!("Output too short")
    }
}

pub fn dc_map_send_simple(dc_list: &str) -> String {
    let mut log = FormatLog::default();
    dc_map_send_simple_with_log(dc_list, &mut log)
}

pub fn dc_map_send_simple_with_log(
    dc_list: &str,
    log: &mut FormatLog,
) -> String {
    if dc_list.is_empty() {
        return String::new();
    }
    let dc_array = explode_escaped(',', dc_list);
    let tables = get_tables();
    let mut mapped = String::new();
    let mut state = "Core".to_string();

    for (idx, dc_id) in dc_array.into_iter().enumerate() {
        if dc_id.is_empty() {
            continue;
        }
        let idx_u64 = u64::try_from(idx).unwrap_or(u64::MAX);
        match dc_id.as_str() {
            "13" => {
                mapped.push_str("FD");
                state = "Core".to_string();
                continue;
            }
            "14" => {
                mapped.push_str("FE");
                state = "Core".to_string();
                continue;
            }
            "16" => {
                if state != "Core" {
                    mapped.push_str("FD");
                }
                mapped.push_str("01");
                state = "DCE_Version_Selectors".to_string();
                continue;
            }
            "196" => {
                if state != "Core" {
                    mapped.push_str("FD");
                }
                mapped.push_str("C5");
                state = "Variant_Selectors".to_string();
                continue;
            }
            "197" => {
                if state != "Core" {
                    mapped.push_str("FD");
                }
                mapped.push_str("C6");
                state = "Semantic_Records".to_string();
                continue;
            }
            "198" => {
                if state != "Core" {
                    mapped.push_str("FD");
                }
                mapped.push_str("C7");
                state = "Mathematics".to_string();
                continue;
            }
            "199" => {
                if state != "Core" {
                    mapped.push_str("FD");
                }
                mapped.push_str("C8");
                state = "Punctuation_and_Whitespace".to_string();
                continue;
            }
            "205" => {
                if state != "DCE_Version_Selectors" {
                    if state != "Core" {
                        mapped.push_str("FD");
                    }
                    mapped.push_str("01");
                    state = "DCE_Version_Selectors".to_string();
                }
                mapped.push_str("03");
                continue;
            }
            _ => {}
        }
        if let Some(record) = tables.dc_map_send_dce3_01a_all.get(&dc_id)
            && !record.is_empty()
        {
            if let Some((state_needed, append)) = record.split_once(':') {
                if state_needed == state {
                    mapped.push_str(append);
                } else if state_needed == "Core" {
                    mapped.push_str("FD");
                    mapped.push_str(append);
                    state = "Core".to_string();
                } else if state_needed == "DCE_Version_Selectors" {
                    if state != "Core" {
                        mapped.push_str("FD");
                    }
                    mapped.push_str("01");
                    mapped.push_str(append);
                    state = "DCE_Version_Selectors".to_string();
                } else {
                    let switch_query = format!(">{state_needed}");
                    let mut state_switch = None;
                    for val in tables.dc_map_send_dce3_01a_all.values() {
                        if val.contains(&switch_query)
                            && let Some((_, switch_hex)) = val.split_once(':')
                        {
                            state_switch = Some(switch_hex.to_string());
                            break;
                        }
                    }
                    if let Some(switch_hex) = state_switch {
                        if state != "Core" {
                            mapped.push_str("FD");
                        }
                        mapped.push_str(&switch_hex);
                        mapped.push_str(append);
                        state = state_needed.to_string();
                    } else {
                        log.export_warning(
                            idx_u64,
                            &format!(
                                "Short Dc {dc_id} (sheet {state_needed}) has no sheet switch in DCE 3.01a and was omitted"
                            ),
                        );
                    }
                }
                continue;
            }
        }
        log.export_warning(
            idx_u64,
            &format!(
                "Short Dc {dc_id} cannot be represented in DCE 3.01a and was omitted"
            ),
        );
    }

    mapped
}
