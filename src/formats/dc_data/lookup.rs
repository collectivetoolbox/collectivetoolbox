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

//! In-memory runtime lookup and query services for Document Characters (Dcs).

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::collections::HashSet;
use std::sync::LazyLock;

use anyhow::{Result, anyhow};

use crate::dc_def::DcDefn;
use crate::report::ValidationReport;
use crate::validation::validate_all_dc_files;
use crate::{DC_CATEGORIES_DIR, DC_DATA_DIR};

static ALL_DC_DEFNS: LazyLock<Vec<DcDefn>> = LazyLock::new(|| {
    // 1. Primary: load from embedded DcList.generated.json if present
    if let Some(file) = DC_DATA_DIR.get_file("DcList.generated.json") {
        if let Ok(mut defns) =
            serde_json::from_slice::<Vec<DcDefn>>(file.contents())
        {
            defns.sort_by_key(|d| d.short_id);
            return defns;
        }
    }

    // 2. Fallback: validate and parse directly from embedded categories
    let mut report = ValidationReport::new();
    let known_format_ids = HashSet::new();
    let mut defns = validate_all_dc_files(
        &DC_CATEGORIES_DIR,
        &known_format_ids,
        &mut report,
    );
    defns.sort_by_key(|d| d.short_id);
    defns
});

static ALL_EITE_ROWS: LazyLock<Vec<Vec<String>>> = LazyLock::new(|| {
    // If DcList.generated.csv is available, parse directly for exact CSV text representation
    if let Some(file) = DC_DATA_DIR.get_file("DcList.generated.csv") {
        let vec_bytes = file.contents().to_vec();
        if let Ok(table) = csv_tools::parse_csv_reader(
            &vec_bytes,
            csv_tools::CsvParseOptions {
                has_header: true,
                flexible: true,
                ..Default::default()
            },
        ) {
            let mut rows = Vec::with_capacity(table.row_count());
            for i in 0..table.row_count() {
                if let Some(row) = table.row(i) {
                    // Slicing columns 1..=9 maps:
                    // 1: Short ID (col 0 in EITE)
                    // 2: Name (col 1 in EITE)
                    // 3: Combining class (col 2 in EITE)
                    // 4: Bidi class (col 3 in EITE)
                    // 5: Casing (col 4 in EITE)
                    // 6: Type (col 5 in EITE)
                    // 7: Script (col 6 in EITE)
                    // 8: Aliases/syntax/xref (col 7 in EITE)
                    // 9: Description (col 8 in EITE)
                    if row.len() >= 10 {
                        let eite_row: Vec<String> = row[1..10]
                            .iter()
                            .map(|s| s.trim().to_string())
                            .collect();
                        rows.push(eite_row);
                    }
                }
            }
            if !rows.is_empty() {
                return rows;
            }
        }
    }

    // Fallback: Construct 9-column rows from ALL_DC_DEFNS
    let defns = get_all_dc_defns();
    let mut rows = Vec::with_capacity(defns.len());
    for d in defns {
        let mut aliases_parts = Vec::new();
        if let Some(syntax) = &d.syntax {
            aliases_parts.push(syntax.raw.clone());
        }
        for alias in &d.aliases {
            aliases_parts.push(alias.clone());
        }
        for xref in &d.cross_references {
            aliases_parts.push(xref.clone());
        }
        for decomp in &d.decompositions {
            aliases_parts.push(decomp.clone());
        }
        let aliases_joined = aliases_parts.join(", ");

        let name_field = if d.is_deprecated {
            format!("!{}", d.name)
        } else {
            d.name.clone()
        };

        rows.push(vec![
            d.short_id.to_string(),
            name_field,
            d.combining_class.to_string(),
            d.bidi_class.as_str().to_string(),
            d.casing_partner.map_or(String::new(), |c| c.to_string()),
            d.general_category.as_str().to_string(),
            d.script.clone(),
            aliases_joined,
            d.description.clone(),
        ]);
    }
    rows
});

/// Returns a static slice of all Document Character definitions sorted by short ID.
pub fn get_all_dc_defns() -> &'static [DcDefn] {
    &ALL_DC_DEFNS
}

/// Looks up a Document Character definition by short ID (0-indexed).
pub fn get_dc_defn(short_id: usize) -> Option<&'static DcDefn> {
    get_all_dc_defns().get(short_id)
}

/// Returns the highest known short Document Character ID.
pub fn maximum_known_short_dc() -> usize {
    get_all_dc_defns().last().map_or(0, |d| d.short_id)
}

/// Returns the total count of registered Document Characters.
pub fn get_dc_count() -> usize {
    get_all_dc_defns().len()
}

/// Returns the canonical name of a short Document Character.
pub fn get_dc_name(short_id: u32) -> Result<String> {
    let usize_id = usize::try_from(short_id)
        .map_err(|e| anyhow!("Invalid short Dc ID {short_id}: {e}"))?;
    get_dc_defn(usize_id)
        .map(|d| d.name.clone())
        .ok_or_else(|| anyhow!("Unknown short Dc ID: {short_id}"))
}

/// Returns the pre-formatted 9-column rows expected by EITE's `"DcData"` dataset.
/// Columns: `[ID, Name, ◌, ⇆, Aa, Type, Script, Aliases, Description]`.
pub fn get_eite_dc_data_rows() -> &'static [Vec<String>] {
    &ALL_EITE_ROWS
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
    fn test_lookup_basic() {
        let dc0 = get_dc_defn(0).expect("Dc 0 exists");
        assert_eq!(dc0.name, "Null");
        assert_eq!(dc0.category, "controls");

        let dc6 = get_dc_defn(6).expect("Dc 6 exists");
        assert_eq!(dc6.name, "Begin number");

        let max_dc = maximum_known_short_dc();
        assert!(max_dc >= 308);
        assert_eq!(get_dc_count(), max_dc + 1);

        let dc308 = get_dc_defn(308).expect("Dc 308 exists");
        assert_eq!(dc308.name, "Next number is a long (global graph) Dc");
        assert_eq!(
            get_dc_name(308).unwrap(),
            "Next number is a long (global graph) Dc"
        );
    }

    #[crate::ctb_test]
    fn test_eite_dc_data_rows() {
        let rows = get_eite_dc_data_rows();
        assert_eq!(rows.len(), get_dc_count());

        let row0 = &rows[0];
        assert_eq!(row0.len(), 9);
        assert_eq!(row0[0], "0");
        assert_eq!(row0[1], "Null");

        let row308 = &rows[308];
        assert_eq!(row308.len(), 9);
        assert_eq!(row308[0], "308");
        assert_eq!(row308[1], "Next number is a long (global graph) Dc");
    }
}
