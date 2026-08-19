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

use anyhow::Result;

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
