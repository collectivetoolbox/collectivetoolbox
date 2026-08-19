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

//! DCE 3.0a specification character set and codepage conversion.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::{Context, anyhow};
use ctb_formats_hexdump::hex2bin;

use ctb_formats_utilities::FormatLog;

use crate::dce_convert;
use crate::tables::get_tables;
use crate::tools::explode_escaped;

pub fn convert_3_0a_to_dc(data: &[u8], log: &mut FormatLog) -> Result<String> {
    let hex = bin2hex(data);
    if hex.len() < 14 || hex.get(12..14) != Some("01") {
        log.error("This document is not stored using the specified format.");
        bail!("This document is not stored using the specified format.");
    }

    let tables = get_tables();
    let mut txt = String::new();
    let mut counter = 14_usize;

    while counter < hex.len() {
        let next_two = counter.saturating_add(2);
        if next_two <= hex.len() {
            if let Some(slice) = hex.get(counter..next_two) {
                let byte_hex = slice.to_uppercase();
                if let Some(val) = tables.dc_map_dce3_0a_core.get(&byte_hex) {
                    if !val.is_empty() {
                        txt.push_str(val);
                        txt.push(',');
                    }
                }
            }
        }
        let next_four = counter.saturating_add(4);
        if next_four <= hex.len() {
            if let Some(four_slice) = hex.get(counter..next_four) {
                if four_slice == "fd03" {
                    break;
                }
            }
        }
        counter = counter.saturating_add(2);
    }

    if txt.len() >= 6 {
        let sub = txt
            .get(3..txt.len().saturating_sub(3))
            .context("substring slice")?;
        txt = sub.to_string();
    } else {
        txt.clear();
    }

    Ok(txt)
}

pub fn convert_3_0a_raw_to_dc(
    data: &[u8],
    log: &mut FormatLog,
) -> Result<String> {
    let hex = bin2hex(data);
    if hex.len() < 2 || hex.get(0..2) != Some("80") {
        log.error("This document is not stored using the specified format.");
        bail!("This document is not stored using the specified format.");
    }

    let mut wrapped = Vec::with_capacity(data.len().saturating_add(10));
    wrapped
        .extend_from_slice(&[0x44, 0x43, 0x45, 0x65, 0x02, 0x01, 0x01, 0xFD]);
    wrapped.extend_from_slice(data);
    wrapped.extend_from_slice(&[0xFD, 0x03]);

    dce_convert(&wrapped, "3_0a", "dc")
        .map(|b| String::from_utf8_lossy(&b).into_owned())
}

pub fn convert_dc_to_3_0a_output(
    data: &str,
    _log: &mut FormatLog,
) -> Result<Vec<u8>> {
    let dc_array = explode_escaped(',', data);
    let tables = get_tables();
    let mut hex_str = "44434565020101fd80".to_string();

    for dc_id in dc_array {
        if let Some(mapped) = tables.dc_map_send_dce3_0a.get(&dc_id) {
            hex_str.push_str(&mapped.to_lowercase());
        }
    }
    hex_str.push_str("81fd03");

    let bytes = hex2bin(&hex_str).map_err(|e| {
        anyhow!("Failed to encode hex in 3_0a serialization: {e}")
    })?;
    Ok(bytes)
}

pub fn convert_dc_to_3_0a_raw_output(
    data: &str,
    _log: &mut FormatLog,
) -> Result<Vec<u8>> {
    let full = dce_convert(data.as_bytes(), "dc", "3_0a")?;
    if full.len() >= 12 {
        let sub = full
            .get(9..full.len().saturating_sub(3))
            .context("substring slice")?;
        Ok(sub.to_vec())
    } else {
        bail!("Output too short")
    }
}
