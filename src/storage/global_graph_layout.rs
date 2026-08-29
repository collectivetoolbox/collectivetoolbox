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

//! Structure of identifier allocations within the global graph database.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::Result;

pub const UNICODE_REGION_START: u128 = 0;
pub const UNICODE_REGION_END: u128 = 1_114_111;
pub const DC_REGION_START: u128 = 1_114_112;
pub const DC_REGION_END: u128 = 2_228_223;
pub const FORMAT_REGION_START: u128 = 2_228_224;
pub const FORMAT_REGION_END: u128 = 3_342_335;

/// Converts a short Document Character (Dc) ID to its Global Graph ID.
#[must_use]
pub fn dc_to_gid(dc_id: u64) -> u128 {
    DC_REGION_START.saturating_add(u128::from(dc_id))
}

/// Converts a short Format ID to its Global Graph ID.
#[must_use]
pub fn format_to_gid(fmt_id: u64) -> u128 {
    FORMAT_REGION_START.saturating_add(u128::from(fmt_id))
}

/// Formats a Global Graph ID into its short prefix representation (`dc:N`, `fmt:N`, `uni:N`, `gid:N`).
#[must_use]
pub fn gid_to_short(gid: u128) -> String {
    if gid <= UNICODE_REGION_END {
        format!("uni:{gid}")
    } else if (DC_REGION_START..=DC_REGION_END).contains(&gid) {
        format!("dc:{offset}", offset = gid.saturating_sub(DC_REGION_START))
    } else if (FORMAT_REGION_START..=FORMAT_REGION_END).contains(&gid) {
        format!("fmt:{offset}", offset = gid.saturating_sub(FORMAT_REGION_START))
    } else {
        format!("gid:{gid}")
    }
}

pub struct GraphBlock {
    pub name: String,
    pub first_id: u128,
    pub last_id: u128,
}

impl GraphBlock {
    pub fn contains_id(&self, id: u128) -> bool {
        id >= self.first_id && id <= self.last_id
    }
}

pub fn get_block(name: &str) -> Option<GraphBlock> {
    let table = get_layout_table().ok()?;
    for i in 0..table.row_count() {
        // Reason for fallback: layout table missing expected CSV column cell defaults to empty string
        let block_name = table.cell_by_header(i, "Block name").unwrap_or("");
        if block_name == name {
            // Reason for fallback: layout table missing expected CSV column cell defaults to empty string
            let first_str =
                table.cell_by_header(i, "First ID in region").unwrap_or("");
            // Reason for fallback: layout table missing expected CSV column cell defaults to empty string
            let last_str =
                table.cell_by_header(i, "Last ID in region").unwrap_or("");
            if let (Ok(first), Ok(last)) =
                (first_str.parse::<u128>(), last_str.parse::<u128>())
            {
                return Some(GraphBlock {
                    name: name.to_string(),
                    first_id: first,
                    last_id: last,
                });
            }
        }
    }
    None
}

pub fn get_block_name_for_id(node_id: u128) -> Result<String> {
    let table = get_layout_table()?;
    for i in 0..table.row_count() {
        // Reason for fallback: layout table missing expected CSV column cell defaults to empty string
        let first_str =
            table.cell_by_header(i, "First ID in region").unwrap_or("");
        // Reason for fallback: layout table missing expected CSV column cell defaults to empty string
        let last_str =
            table.cell_by_header(i, "Last ID in region").unwrap_or("");
        // Reason for fallback: layout table missing expected CSV column cell defaults to empty string
        let block_name = table.cell_by_header(i, "Block name").unwrap_or("");

        if let (Ok(first), Ok(last)) =
            (first_str.parse::<u128>(), last_str.parse::<u128>())
        {
            if node_id >= first && node_id <= last {
                if block_name == "(refer to Unicode block names)"
                    || block_name == "Unicode"
                {
                    return Ok("Unicode".to_string());
                }
                return Ok(block_name.to_string());
            }
        }
    }
    Ok("Reserved".to_string())
}

/// Validate whether a node target ID is allowed for publishing.
///
/// Returns an error if the target ID falls within the restricted Unicode range.
pub fn validate_publish_target(target_id: u128) -> Result<()> {
    if let Some(ref block) = get_block("Unicode") {
        if block.contains_id(target_id) {
            anyhow::bail!("Publishing nodes to the Unicode range is disallowed.");
        }
    }
    if get_block_name_for_id(target_id)? == "Unicode" {
        anyhow::bail!("Publishing nodes to the Unicode range is disallowed.");
    }
    Ok(())
}

fn get_layout_table() -> Result<std::sync::Arc<csv_tools::CsvTable>> {
    csv_tools::get_or_load_cached(
        "ctb_storage::data/global-graph-layout.csv",
        || {
            let bytes = crate::get_asset("data/global-graph-layout.csv")
                .ok_or_else(|| {
                    anyhow::anyhow!("global-graph-layout.csv not found")
                })?;
            csv_tools::parse_csv_reader(
                &bytes,
                csv_tools::CsvParseOptions {
                    has_header: true,
                    ..Default::default()
                },
            )
        },
    )
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
    fn test_dc_to_gid_and_gid_to_short() {
        assert_eq!(dc_to_gid(0), 1_114_112);
        assert_eq!(dc_to_gid(296), 1_114_408);
        assert_eq!(gid_to_short(1_114_408), "dc:296");
    }

    #[crate::ctb_test]
    fn test_format_to_gid_and_gid_to_short() {
        assert_eq!(format_to_gid(0), 2_228_224);
        assert_eq!(format_to_gid(80), 2_228_304);
        assert_eq!(gid_to_short(2_228_304), "fmt:80");
    }

    #[crate::ctb_test]
    fn test_unicode_gid_to_short() {
        assert_eq!(gid_to_short(0), "uni:0");
        assert_eq!(gid_to_short(1234), "uni:1234");
        assert_eq!(gid_to_short(1_114_111), "uni:1114111");
    }

    #[crate::ctb_test]
    fn test_non_short_mappable_gid_to_short() {
        assert_eq!(gid_to_short(23_234_234), "gid:23234234");
    }
}
