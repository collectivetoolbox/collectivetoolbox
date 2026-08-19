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

use std::collections::HashMap;
use std::sync::OnceLock;

pub struct Tables {
    pub dc_map_unicode_lossy: HashMap<String, String>,
    pub dc_map_send_unicode: HashMap<String, String>,
    pub base64_to_dc: HashMap<String, String>,
    pub dc_to_base64: HashMap<String, String>,
    pub cdce_html_legacy: HashMap<String, String>,
    pub dce_versions: Vec<String>,
    pub dc_map_send_dce3_0a: HashMap<String, String>,
    pub dc_map_dce3_0a_core: HashMap<String, String>,
    pub dce3_0a_core: Vec<String>,
    pub dc_map_send_dce3_01a_all: HashMap<String, String>,
    pub dc_map_dce3_01a_core: HashMap<String, String>,
    pub dc_map_dce3_01a_mathematics: HashMap<String, String>,
    pub dc_map_dce3_01a_punctuation_and_whitespace: HashMap<String, String>,
    pub dc_map_dce3_01a_semantic_records: HashMap<String, String>,
    pub dc_map_dce3_01a_variant_selectors: HashMap<String, String>,
}

fn parse_csv_map(csv_bytes: &[u8]) -> HashMap<String, String> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(csv_bytes);
    let mut map = HashMap::new();
    for record in rdr.records().flatten() {
        if record.len() >= 2 {
            if let (Some(k), Some(v)) = (record.get(0), record.get(1)) {
                let _ = map.insert(k.to_string(), v.to_string());
            }
        } else if record.len() == 1 {
            if let Some(k) = record.get(0) {
                let _ = map.insert(k.to_string(), String::new());
            }
        }
    }
    map
}

fn parse_csv_vec(csv_bytes: &[u8]) -> Vec<String> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(csv_bytes);
    let mut vec = vec![String::new(); 256];
    for record in rdr.records().flatten() {
        if record.len() >= 2 {
            if let (Some(k), Some(v)) = (record.get(0), record.get(1)) {
                if let Ok(idx) = k.parse::<usize>() {
                    if let Some(slot) = vec.get_mut(idx) {
                        *slot = v.to_string();
                    }
                }
            }
        }
    }
    vec
}

pub static TABLES: OnceLock<Tables> = OnceLock::new();

pub fn get_tables() -> &'static Tables {
    TABLES.get_or_init(|| {
        Tables {
            dc_map_unicode_lossy: parse_csv_map(include_bytes!("data/csv/dceutils_data/dceutils_data.php-DcMap_Unicode_Lossy.csv")),
            dc_map_send_unicode: parse_csv_map(include_bytes!("data/csv/dceutils_data_dc/dceutils_data_dc.php-DcMapSend_Unicode.csv")),
            base64_to_dc: parse_csv_map(include_bytes!("data/csv/dceutils_data/dceutils_data.php-Base64_to_Dc.csv")),
            dc_to_base64: parse_csv_map(include_bytes!("data/csv/dceutils_data/dceutils_data.php-Dc_to_Base64.csv")),
            cdce_html_legacy: parse_csv_map(include_bytes!("data/csv/dceutils_data/dceutils_data.php-cdce_html_legacy.csv")),
            dce_versions: parse_csv_vec(include_bytes!("data/csv/dceutils_data/dceutils_data.php-dce_versions.csv")),
            dc_map_send_dce3_0a: parse_csv_map(include_bytes!("data/csv/dceutils_data_3_0a/dceutils_data_3_0a.php-DcMapSend_dce3_0a.csv")),
            dc_map_dce3_0a_core: parse_csv_map(include_bytes!("data/csv/dceutils_data_3_0a/dceutils_data_3_0a.php-DcMap_dce3_0a_Core.csv")),
            dce3_0a_core: parse_csv_vec(include_bytes!("data/csv/dceutils_data_3_0a/dceutils_data_3_0a.php-dce3_0a_core.csv")),
            dc_map_send_dce3_01a_all: parse_csv_map(include_bytes!("data/csv/dceutils_data_3_01a/dceutils_data_3_01a.php-DcMapSend_dce3_01a_All.csv")),
            dc_map_dce3_01a_core: parse_csv_map(include_bytes!("data/csv/dceutils_data_3_01a/dceutils_data_3_01a.php-DcMap_dce3_01a_Core.csv")),
            dc_map_dce3_01a_mathematics: parse_csv_map(include_bytes!("data/csv/dceutils_data_3_01a/dceutils_data_3_01a.php-DcMap_dce3_01a_Mathematics.csv")),
            dc_map_dce3_01a_punctuation_and_whitespace: parse_csv_map(include_bytes!("data/csv/dceutils_data_3_01a/dceutils_data_3_01a.php-DcMap_dce3_01a_Punctuation_and_Whitespace.csv")),
            dc_map_dce3_01a_semantic_records: parse_csv_map(include_bytes!("data/csv/dceutils_data_3_01a/dceutils_data_3_01a.php-DcMap_dce3_01a_Semantic_Records.csv")),
            dc_map_dce3_01a_variant_selectors: parse_csv_map(include_bytes!("data/csv/dceutils_data_3_01a/dceutils_data_3_01a.php-DcMap_dce3_01a_Variant_Selectors.csv")),
        }
    })
}
