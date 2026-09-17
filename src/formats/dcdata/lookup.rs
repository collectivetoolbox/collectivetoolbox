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
use crate::{get_dc_categories_dir, get_dc_data_file};

static ALL_DC_DEFNS: LazyLock<Vec<DcDefn>> = LazyLock::new(|| {
    let mut report = ValidationReport::new();
    let known_format_ids = HashSet::new();
    let mut defns = if let Some(dc_dir) = get_dc_categories_dir() {
        validate_all_dc_files(
            dc_dir,
            &known_format_ids,
            &mut report,
        )
    } else {
        Vec::new()
    };
    defns.sort_by_key(|d| d.dc_id);
    defns
});

static ALL_EITE_ROWS: LazyLock<Vec<Vec<String>>> = LazyLock::new(|| {
    // If DcList.generated.csv is available, parse directly for exact CSV text representation
    if let Some(vec_bytes) = get_dc_data_file("DcList.generated.csv") {
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
                    // Only include rows with a valid Short ID for EITE's DcData dataset
                    let has_short_id = row
                        .get(1)
                        .is_some_and(|s| s.trim().parse::<u32>().is_ok());
                    if !has_short_id {
                        continue;
                    }

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
                    if let Some(slice) = row.get(1..10) {
                        let eite_row: Vec<String> = slice
                            .iter()
                            .map(|s| s.trim().to_string())
                            .collect();
                        rows.push(eite_row);
                    }
                }
            }
            return rows;
        }
    }

    // Reason for fallback: missing or unparseable DcList.generated.csv yields empty rows table
    Vec::new()
});

/// Returns a static slice of all Document Character definitions sorted by long ID (`dc_id`).
pub fn get_all_dc_defns() -> &'static [DcDefn] {
    &ALL_DC_DEFNS
}

/// Looks up a Document Character definition by long (Global Graph) ID.
pub fn get_dc_defn(dc_id: u128) -> Option<&'static DcDefn> {
    let defns = get_all_dc_defns();
    defns
        .binary_search_by_key(&dc_id, |d| d.dc_id)
        .ok()
        .and_then(|idx| defns.get(idx))
}

/// Looks up a Document Character definition by short ID (0-indexed).
pub fn get_short_dc_defn(short_id: usize) -> Option<&'static DcDefn> {
    let short_u32 = u32::try_from(short_id).ok()?;
    get_dc_defn(crate::dc::short_to_long_dc(short_u32))
}

/// Returns the highest known short Document Character ID.
pub fn maximum_known_short_dc() -> usize {
    // Reason for fallback: empty definitions list defaults to maximum ID of 0
    get_all_dc_defns()
        .iter()
        .filter_map(|d| d.short_id)
        .max()
        .unwrap_or(0)
}

/// Returns the total count of registered short Document Characters.
pub fn get_dc_count() -> usize {
    get_eite_dc_data_rows().len()
}

/// Returns the canonical name of a Document Character by long (Global Graph) ID.
pub fn get_dc_name(dc_id: u128) -> Result<String> {
    get_dc_defn(dc_id)
        .map(|d| d.name.clone())
        .ok_or_else(|| anyhow!("Unknown Dc ID: {dc_id}"))
}

/// Returns the canonical name of a short Document Character.
pub fn get_short_dc_name(short_id: u32) -> Result<String> {
    get_dc_name(crate::dc::short_to_long_dc(short_id))
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
        let dc0 = get_short_dc_defn(0).expect("Dc 0 exists");
        assert_eq!(dc0.name, "Null (short Dc)");
        assert_eq!(dc0.category, "controls");

        let dc0_long = get_dc_defn(crate::dc::short_to_long_dc(0)).expect("Dc 0 long ID exists");
        assert_eq!(dc0_long.name, "Null (short Dc)");

        let dc6 = get_short_dc_defn(6).expect("Dc 6 exists");
        assert_eq!(dc6.name, "Begin number");

        let max_dc = maximum_known_short_dc();
        assert!(max_dc >= 308);
        assert_eq!(get_dc_count(), max_dc + 1);

        let dc308 = get_short_dc_defn(308).expect("Dc 308 exists");
        assert_eq!(dc308.name, "Next number is a long (global graph) Dc");
        assert_eq!(
            get_short_dc_name(308).unwrap(),
            "Next number is a long (global graph) Dc"
        );
        assert_eq!(
            get_dc_name(crate::dc::short_to_long_dc(308)).unwrap(),
            "Next number is a long (global graph) Dc"
        );

        let u10 = get_dc_defn(10).expect("Unicode clarification 10 exists");
        assert_eq!(u10.name, "<control, Unicode-semantic-defined> LINE FEED");
        assert_eq!(u10.category, "unicode-clarifications");
        assert_eq!(
            get_dc_name(10).unwrap(),
            "<control, Unicode-semantic-defined> LINE FEED"
        );
    }

    #[crate::ctb_test]
    fn test_eite_dc_data_rows() {
        let rows = get_eite_dc_data_rows();
        assert_eq!(rows.len(), get_dc_count());

        let row0 = &rows[0];
        assert_eq!(row0.len(), 9);
        assert_eq!(row0[0], "0");
        assert_eq!(row0[1], "!Null (short Dc)");

        let row308 = &rows[308];
        assert_eq!(row308.len(), 9);
        assert_eq!(row308[0], "308");
        assert_eq!(row308[1], "Next number is a long (global graph) Dc");
    }
}
