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

//! Schema parser and validator for the global graph layout partition table.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::report::ValidationReport;

pub const EXPECTED_UNICODE_START: u128 = 0;
pub const EXPECTED_UNICODE_END: u128 = 1_114_111;
pub const EXPECTED_DC_START: u128 = 1_114_112;
pub const EXPECTED_DC_END: u128 = 2_228_223;
pub const EXPECTED_FORMAT_START: u128 = 2_228_224;
pub const EXPECTED_FORMAT_END: u128 = 3_342_335;

/// A validated row from `global-graph-layout.csv`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedLayoutRow {
    pub first_id: u128,
    pub last_id: u128,
    pub count: u128,
    pub block_name: String,
    pub description: String,
    pub line_number: usize,
}

/// Validates the global graph layout CSV table.
pub fn validate_layout_table(
    csv_bytes: &[u8],
    file_path: &str,
    report: &mut ValidationReport,
) -> Vec<ParsedLayoutRow> {
    let vec_bytes = csv_bytes.to_vec();
    let table = match csv_tools::parse_csv_reader(
        &vec_bytes,
        csv_tools::CsvParseOptions {
            has_header: true,
            flexible: true,
            ..Default::default()
        },
    ) {
        Ok(t) => t,
        Err(e) => {
            report.add_error(
                file_path,
                None,
                None,
                format!("Failed to parse CSV: {e}"),
                Some("Check CSV syntax and quoting"),
            );
            return Vec::new();
        }
    };

    let mut rows = Vec::new();
    let mut expected_next_first: Option<u128> = Some(0);

    if let Some(header) = table.header() {
        if header.len() != 5 {
            report.add_error(
                file_path,
                Some(1),
                None,
                format!(
                    "CSV header has {} columns, expected 5 columns",
                    header.len()
                ),
                Some("Ensure CSV header has exactly 5 columns matching schema"),
            );
        }
    }

    for i in 0..table.row_count() {
        let line_no = i.saturating_add(2); // 1-indexed header + data row

        if let Some(row_slice) = table.row(i) {
            if row_slice.iter().all(|s| s.trim().is_empty()) {
                continue;
            }
            if row_slice.len() != 5 {
                report.add_error(
                    file_path,
                    Some(line_no),
                    None,
                    format!(
                        "Row has {} columns, expected 5 columns (mismatched field count)",
                        row_slice.len()
                    ),
                    Some("Check for unquoted commas, missing commas, or extra columns to avoid misaligned data"),
                );
            }
        }

        let get_str = |col: usize| -> String {
            match table.cell(i, col) {
                Some(s) => s.trim().to_string(),
                None => String::new(),
            }
        };

        let first_str = get_str(0);
        let last_str = get_str(1);
        let count_str = get_str(2);
        let block_name = get_str(3);
        let description = get_str(4);

        if first_str.is_empty() && last_str.is_empty() && block_name.is_empty()
        {
            continue;
        }

        let first_id = if let Ok(val) = first_str.parse::<u128>() {
            val
        } else {
            report.add_error(
                file_path,
                Some(line_no),
                Some("First ID in region"),
                format!("Invalid integer for First ID: '{first_str}'"),
                Some("Must be a non-negative integer"),
            );
            continue;
        };

        let last_id = if let Ok(val) = last_str.parse::<u128>() {
            val
        } else {
            report.add_error(
                file_path,
                Some(line_no),
                Some("Last ID in region"),
                format!("Invalid integer for Last ID: '{last_str}'"),
                Some("Must be a non-negative integer"),
            );
            continue;
        };

        let count = if let Ok(val) = count_str.parse::<u128>() {
            val
        } else {
            report.add_error(
                file_path,
                Some(line_no),
                Some("Count of IDs in region"),
                format!("Invalid integer for Count: '{count_str}'"),
                Some("Must be a positive integer"),
            );
            continue;
        };

        if block_name.is_empty() {
            report.add_error(
                file_path,
                Some(line_no),
                Some("Block name"),
                "Block name cannot be empty",
                Some(
                    "Provide a descriptive name for the graph partition region",
                ),
            );
        }

        if last_id < first_id {
            report.add_error(
                file_path,
                Some(line_no),
                Some("Last ID in region"),
                format!("Last ID ({last_id}) must be >= First ID ({first_id})"),
                Some("Correct range bounds"),
            );
        }

        let computed_count = last_id.saturating_sub(first_id).saturating_add(1);
        if count != computed_count {
            report.add_error(
                file_path,
                Some(line_no),
                Some("Count of IDs in region"),
                format!(
                    "Region Count ({count}) does not match Last ID - First ID + 1 ({computed_count})"
                ),
                Some("Update Count column to match the arithmetic span"),
            );
        }

        if let Some(expected_first) = expected_next_first {
            if first_id != expected_first {
                report.add_error(
                    file_path,
                    Some(line_no),
                    Some("First ID in region"),
                    format!(
                        "Region gap or overlap detected: expected First ID {expected_first}, but found {first_id}"
                    ),
                    Some("Ensure regions are contiguous with no gaps or overlaps"),
                );
            }
        }

        expected_next_first = Some(last_id.saturating_add(1));

        rows.push(ParsedLayoutRow {
            first_id,
            last_id,
            count,
            block_name,
            description,
            line_number: line_no,
        });
    }

    // Verify key landmark regions
    if let Some(unicode_row) = rows.first() {
        if unicode_row.first_id != EXPECTED_UNICODE_START
            || unicode_row.last_id != EXPECTED_UNICODE_END
        {
            report.add_error(
                file_path,
                Some(unicode_row.line_number),
                Some("Block name"),
                format!(
                    "Unicode region bounds mismatch: expected 0..={EXPECTED_UNICODE_END}, found {}..={}",
                    unicode_row.first_id, unicode_row.last_id
                ),
                Some("Restore canonical Unicode bounds"),
            );
        }
    }

    if let Some(dc_row) = rows.get(1) {
        if dc_row.first_id != EXPECTED_DC_START
            || dc_row.last_id != EXPECTED_DC_END
        {
            report.add_error(
                file_path,
                Some(dc_row.line_number),
                Some("Block name"),
                format!(
                    "Dc region bounds mismatch: expected {EXPECTED_DC_START}..={EXPECTED_DC_END}, found {}..={}",
                    dc_row.first_id, dc_row.last_id
                ),
                Some("Restore canonical Dc region bounds"),
            );
        }
    }

    if let Some(fmt_row) = rows.get(2) {
        if fmt_row.first_id != EXPECTED_FORMAT_START
            || fmt_row.last_id != EXPECTED_FORMAT_END
        {
            report.add_error(
                file_path,
                Some(fmt_row.line_number),
                Some("Block name"),
                format!(
                    "Format region bounds mismatch: expected {EXPECTED_FORMAT_START}..={EXPECTED_FORMAT_END}, found {}..={}",
                    fmt_row.first_id, fmt_row.last_id
                ),
                Some("Restore canonical Formats region bounds"),
            );
        }
    }

    rows
}
