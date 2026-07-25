#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
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
        let block_name = table.cell_by_header(i, "Block name").unwrap_or("");
        if block_name == name {
            let first_str = table.cell_by_header(i, "First ID in region").unwrap_or("");
            let last_str = table.cell_by_header(i, "Last ID in region").unwrap_or("");
            if let (Ok(first), Ok(last)) = (first_str.parse::<u128>(), last_str.parse::<u128>()) {
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
        let first_str = table.cell_by_header(i, "First ID in region").unwrap_or("");
        let last_str = table.cell_by_header(i, "Last ID in region").unwrap_or("");
        let block_name = table.cell_by_header(i, "Block name").unwrap_or("");

        if let (Ok(first), Ok(last)) = (first_str.parse::<u128>(), last_str.parse::<u128>()) {
            if node_id >= first && node_id <= last {
                if block_name == "(refer to Unicode block names)" || block_name == "Unicode" {
                    return Ok("Unicode".to_string());
                }
                return Ok(block_name.to_string());
            }
        }
    }
    Ok("Reserved".to_string())
}

fn get_layout_table() -> Result<std::sync::Arc<csv_tools::CsvTable>> {
    csv_tools::get_or_load_cached(
        "ctb_storage::data/global-graph-layout.csv",
        || {
            let bytes = crate::get_asset("data/global-graph-layout.csv")
                .ok_or_else(|| anyhow::anyhow!("global-graph-layout.csv not found"))?;
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
