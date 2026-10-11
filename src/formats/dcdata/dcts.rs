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

//! Implementation of Dcts (DcText short) serialization.
//!
//! Encodes sequences of Document Character (Dc) IDs to canonical Dcts text
//! representation:
//! - Plain text characters become literal Unicode (except `@` which becomes `@@`).
//! - Short Dcs are formatted as `@{id}@`.
//! - Formats are formatted as `@f{id}@`.
//! - Unicode escapes outside standard scalar characters are `@u{hex}@`.
//! - Local node references (Dc 296 followed by a numeral) are `@L{number}@`.
//! - Long Dcs are `@l{id}@`.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use ctb_storage_minimal::global_graph_layout::{
    FORMAT_REGION_END, FORMAT_REGION_START, SHORT_DC_REGION_END,
    SHORT_DC_REGION_START, UNICODE_REGION_END,
};

use crate::dc_number_minimal::read_dc_number_u128;

/// Options for Dcts serialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DctsOptions {
    /// When true, format Unicode control characters (e.g. 0..=0x1F, 0x7F..=0x9F)
    /// using `@u{hex}@` syntax instead of emitting raw control characters.
    pub escape_control_chars: bool,
}

/// Serializes a sequence of Document Character IDs (`&[u128]`) to a Dcts `String`
/// using the specified formatting options.
#[must_use]
pub fn dclist_to_dcts_string_with_options(
    dclist: &[u128],
    options: DctsOptions,
) -> String {
    let mut output = String::new();
    let mut i = 0usize;

    while i < dclist.len() {
        let Some(&dcid) = dclist.get(i) else { break };

        if dcid == 64 {
            output.push_str("@@");
        } else if dcid <= UNICODE_REGION_END {
            if let Ok(cp) = u32::try_from(dcid) {
                if let Some(ch) = char::from_u32(cp) {
                    if options.escape_control_chars && ch.is_control() {
                        output.push_str(&format!("@u{cp:x}@"));
                    } else {
                        output.push(ch);
                    }
                } else {
                    output.push_str(&format!("@u{cp:x}@"));
                }
            } else {
                output.push_str(&format!("@u{dcid:x}@"));
            }
        } else if dcid == 1_114_408 {
            if let Some(rest) = dclist.get(i.saturating_add(1)..) {
                if let Ok((int_val, consumed)) = read_dc_number_u128(rest) {
                    output.push_str(&format!("@L{int_val}@"));
                    i = i.saturating_add(1).saturating_add(consumed);
                    continue;
                }
            }
            output.push_str("@296@");
        } else if (SHORT_DC_REGION_START..=SHORT_DC_REGION_END).contains(&dcid) {
            let short_id = dcid.saturating_sub(SHORT_DC_REGION_START);
            output.push_str(&format!("@{short_id}@"));
        } else if (FORMAT_REGION_START..=FORMAT_REGION_END).contains(&dcid) {
            let fmt_id = dcid.saturating_sub(FORMAT_REGION_START);
            output.push_str(&format!("@f{fmt_id}@"));
        } else {
            output.push_str(&format!("@l{dcid}@"));
        }
        i = i.saturating_add(1);
    }
    output
}

/// Serializes a sequence of Document Character IDs (`&[u128]`) to Dcts format
/// bytes using the specified options.
#[must_use]
pub fn dclist_to_dcts_with_options(
    dclist: &[u128],
    options: DctsOptions,
) -> Vec<u8> {
    dclist_to_dcts_string_with_options(dclist, options).into_bytes()
}

/// Serializes a sequence of Document Character IDs (`&[u128]`) to Dcts format
/// bytes using default options.
#[must_use]
pub fn dclist_to_dcts(dclist: &[u128]) -> Vec<u8> {
    dclist_to_dcts_with_options(dclist, DctsOptions::default())
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
    fn test_dclist_to_dcts_plain_and_escapes() {
        // Plain text: "hello"
        let stream = vec![104, 101, 108, 108, 111];
        assert_eq!(dclist_to_dcts_string_with_options(&stream, DctsOptions::default()), "hello");

        // Literal '@'
        let at_stream = vec![64];
        assert_eq!(dclist_to_dcts_string_with_options(&at_stream, DctsOptions::default()), "@@");

        // Short Dc 123
        let short_dc = SHORT_DC_REGION_START.saturating_add(123);
        assert_eq!(dclist_to_dcts_string_with_options(&[short_dc], DctsOptions::default()), "@123@");

        // Format 80
        let fmt = FORMAT_REGION_START.saturating_add(80);
        assert_eq!(dclist_to_dcts_string_with_options(&[fmt], DctsOptions::default()), "@f80@");
    }

    #[crate::ctb_test]
    fn test_dclist_to_dcts_control_characters() {
        let stream = vec![10, 0, 9]; // '\n', '\0', '\t'

        // Default: raw control chars
        let raw = dclist_to_dcts_string_with_options(&stream, DctsOptions::default());
        assert_eq!(raw, "\n\0\t");

        // Escaped: control chars converted to @u...
        let escaped = dclist_to_dcts_string_with_options(
            &stream,
            DctsOptions { escape_control_chars: true },
        );
        assert_eq!(escaped, "@ua@@u0@@u9@");
    }
}
