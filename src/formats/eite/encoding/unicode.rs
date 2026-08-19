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

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::{Result, anyhow};

use crate::formats::{dc_from_format, dc_to_format};
use ctb_formats_utilities::FormatLog;

/// Convert a single Unicode scalar (given as an integer) to Dc array (1 element) using 'unicode' mapping.
/// JS: dcaFromUnicodeChar(intChar)
pub fn dca_from_unicode_char(int_char: u32) -> Result<(Vec<u32>, FormatLog)> {
    // In JS: dcFromFormat('unicode', anFromN(intChar)) then push first if exists.
    // We treat int_char as a Unicode scalar, encode it to UTF-8, then call dc_from_format.
    let ch =
        char::from_u32(int_char).ok_or_else(|| anyhow!("Invalid codepoint"))?;
    let mut buf = [0u8; 4];
    let s = ch.encode_utf8(&mut buf);
    let (dcs, log) = dc_from_format("unicode", s.as_bytes())?;
    if dcs.is_empty() {
        Ok((vec![], log))
    } else {
        Ok((
            vec![*dcs.first().ok_or_else(|| anyhow!("dcs is empty"))?],
            log,
        ))
    }
}

/// Convert a Dc to a Unicode codepoint array (1 element).
/// JS: dcToUnicodeCharArray(intDc)
pub fn dc_to_unicode_char_array(dc: u32) -> Result<(Vec<u32>, FormatLog)> {
    // JS did: dcToFormat('unicode', intDc) -> returns UTF-8 bytes -> take firstCharOfUtf8String
    let (utf8_bytes, log) = dc_to_format("unicode", dc)?;
    if utf8_bytes.is_empty() {
        return Ok((vec![], log));
    }
    // Decode first UTF-8 codepoint
    let ch = std::str::from_utf8(&utf8_bytes)
        .map_err(|e| anyhow!("Invalid UTF-8 from dc_to_format: {e}"))?
        .chars()
        .next()
        .ok_or_else(|| anyhow!("Empty UTF-8 string from dc_to_format"))?;
    Ok((vec![u32::from(ch)], log))
}
