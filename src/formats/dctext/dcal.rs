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
//! ASCII text containing a list of integers representing new Dcs. Strictly, one
//! space after every int, and no newlines. Loosely, with newlines or multiple
//! spaces, or no end space.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::Result;
use ctb_formats_utilities::{ConversionOutput, FormatLog};

use crate::DcList;

/// Converts a Dcal document (`&[u8]`) into a `DcList` (`Vec<u128>`).
///
/// Strictly: ASCII decimal integers with one space after every integer.
/// Loosely: ASCII text containing decimal integers separated by whitespace.
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
    for token in text.split_ascii_whitespace() {
        if !token.bytes().all(|b| b.is_ascii_digit()) {
            log.warn(&format!("Skipping non-integer Dcal token '{token}'"));
            continue;
        }
        match token.parse::<u128>() {
            Ok(id) => list.push(id),
            Err(e) => {
                log.warn(&format!("Failed to parse integer '{token}': {e}"));
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
        let input = b"65 1114408  2228304\n1234\t5678 ";
        let conv = dcal_to_dclist(input).expect("Parse dcal");
        assert!(!conv.log.has_warnings());
        assert_eq!(conv.result, vec![65, 1114408, 2228304, 1234, 5678]);
    }

    #[crate::ctb_test]
    fn test_dcal_skips_non_digits() {
        let input = b"65 invalid 1114408";
        let conv = dcal_to_dclist(input).expect("Parse dcal");
        assert!(conv.log.has_warnings());
        assert_eq!(conv.result, vec![65, 1114408]);
    }
}
