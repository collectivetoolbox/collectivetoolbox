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

//! Implementation of Format 340 (Dcal: Dc ASCII List).
//!
//! Dcal is an ASCII serialization format containing a space-delimited list of integers
//! representing Document Characters and Global Graph IDs. Strictly, one space follows
//! every integer with no newlines. Loosely, newlines, multiple spaces, commas,
//! hex literals, or prefixed IDs (`dc:N`, `fmt:N`, `uni:N`, `U+XXXX`) are supported on import.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::{Result, anyhow, bail};
use ctb_formats_utilities::{ConversionOutput, FormatLog};

use crate::DcList;

/// Parses a token representing a single Global Graph ID or short ID into a `u128`.
///
/// Supported formats:
/// - Decimal integers: `1114408`, `65`
/// - Hexadecimal literals: `0x110128`, `0x41`
/// - Prefixed short Dc IDs: `dc:296`, `dc:0x128` -> mapped to `1114112 + N`
/// - Prefixed format IDs: `fmt:80`, `fmt:0x50` -> mapped to `2228224 + N`
/// - Prefixed Unicode codepoints: `uni:65`, `U+0041`, `u+1f602` -> `65`, `0x41`, `0x1F602`
/// - Embedded tokens: `@1114408@`, `@L296@` -> mapped to global graph IDs
pub fn parse_graph_token(token: &str) -> Result<u128> {
    let trimmed = token.trim();
    if trimmed.is_empty() {
        bail!("Empty ID token");
    }

    // Strip wrapping '@' if present (e.g. "@1114408@")
    let inner = if trimmed.starts_with('@') && trimmed.ends_with('@') && trimmed.len() >= 2 {
        &trimmed[1..trimmed.len().saturating_sub(1)]
    } else {
        trimmed
    };

    if let Some(rest) = inner.strip_prefix("dc:")
        .or_else(|| inner.strip_prefix("DC:"))
        .or_else(|| inner.strip_prefix("Dc:"))
    {
        let short_id = parse_u128_literal(rest)?;
        return Ok(1_114_112_u128.saturating_add(short_id));
    }

    if let Some(rest) = inner.strip_prefix("fmt:")
        .or_else(|| inner.strip_prefix("FMT:"))
        .or_else(|| inner.strip_prefix("Fmt:"))
    {
        let short_fmt = parse_u128_literal(rest)?;
        return Ok(2_228_224_u128.saturating_add(short_fmt));
    }

    if let Some(rest) = inner.strip_prefix("uni:")
        .or_else(|| inner.strip_prefix("UNI:"))
        .or_else(|| inner.strip_prefix("Uni:"))
    {
        return parse_u128_literal(rest);
    }

    if let Some(rest) = inner.strip_prefix("U+").or_else(|| inner.strip_prefix("u+")) {
        let cp = u128::from_str_radix(rest, 16)
            .map_err(|e| anyhow!("Invalid Unicode hex codepoint '{trimmed}': {e}"))?;
        return Ok(cp);
    }

    if let Some(rest) = inner.strip_prefix('L') {
        // @L<id>@ token: 1114408 followed by local id
        let local_id = parse_u128_literal(rest)?;
        return Ok(local_id);
    }

    parse_u128_literal(inner)
}

fn parse_u128_literal(s: &str) -> Result<u128> {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u128::from_str_radix(hex, 16)
            .map_err(|e| anyhow!("Invalid hex literal '{s}': {e}"))
    } else {
        s.parse::<u128>()
            .map_err(|e| anyhow!("Invalid integer literal '{s}': {e}"))
    }
}

/// Converts a Dcal document (`&[u8]`) into a `DcList` (`Vec<u128>`).
///
/// Parses whitespace- and comma-delimited tokens into Global Graph IDs.
pub fn dcal_to_dclist(document: &[u8]) -> Result<ConversionOutput<DcList>> {
    let mut log = FormatLog::default();
    let text = match std::str::from_utf8(document) {
        Ok(s) => s,
        Err(e) => {
            log.warn(&format!("Invalid UTF-8 in Dcal input: {e}"));
            return Ok(ConversionOutput::new(Vec::new(), log));
        }
    };

    let mut list = Vec::new();
    for line in text.lines() {
        for part in line.split(|c: char| c.is_ascii_whitespace() || c == ',') {
            let token = part.trim();
            if token.is_empty() {
                continue;
            }
            match parse_graph_token(token) {
                Ok(id) => list.push(id),
                Err(e) => {
                    log.warn(&format!("Skipping invalid Dcal token '{token}': {e}"));
                }
            }
        }
    }

    Ok(ConversionOutput::new(list, log))
}

/// Serializes a `DcList` (`&[u128]`) to Dcal format bytes (`Vec<u8>`).
///
/// Strictly outputs one space after every integer in standard decimal format.
pub fn dclist_to_dcal(dclist: &[u128]) -> Vec<u8> {
    let mut output = String::new();
    for &id in dclist {
        output.push_str(&format!("{id} "));
    }
    output.into_bytes()
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
    fn test_dcal_serialization() {
        let list = vec![65, 1114408, 2228304];
        let bytes = dclist_to_dcal(&list);
        assert_eq!(std::str::from_utf8(&bytes).unwrap(), "65 1114408 2228304 ");
    }

    #[crate::ctb_test]
    fn test_dcal_parsing_loose() {
        let input = b"65 1114408, 2228304\n0x41 dc:296 fmt:80 U+0041 uni:65 @1114408@";
        let conv = dcal_to_dclist(input).expect("Parse dcal");
        assert!(!conv.log.has_warnings());
        assert_eq!(
            conv.result,
            vec![
                65, 1114408, 2228304,
                65, 1114408, 2228304,
                65, 65, 1114408
            ]
        );
    }

    #[crate::ctb_test]
    fn test_parse_graph_token() {
        assert_eq!(parse_graph_token("1114408").unwrap(), 1114408);
        assert_eq!(parse_graph_token("0x110128").unwrap(), 1114408);
        assert_eq!(parse_graph_token("dc:296").unwrap(), 1114408);
        assert_eq!(parse_graph_token("fmt:80").unwrap(), 2228304);
        assert_eq!(parse_graph_token("U+0041").unwrap(), 65);
        assert_eq!(parse_graph_token("uni:65").unwrap(), 65);
        assert_eq!(parse_graph_token("@1114408@").unwrap(), 1114408);
    }
}
