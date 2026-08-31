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

//! Command-line runner for data table validation, automatic ID assignment,
//! and merged dataset generation.

use std::process::ExitCode;
use ctb_formats_data_validator::{
    assign_and_update_dc_categories, assign_and_update_format_categories,
    find_repository_root, generate_merged_csvs, validate_all_data_tables,
};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let check_only = args.iter().any(|arg| arg == "--check" || arg == "--verify");

    let repo_root = match find_repository_root() {
        Ok(path) => Some(path),
        Err(e) => {
            eprintln!("Warning: Could not determine repository root ({e})");
            None
        }
    };

    if !check_only {
        if let Some(ref root) = repo_root {
            match assign_and_update_dc_categories(root) {
                Ok(stats) => {
                    if stats.new_ids_assigned > 0 || stats.dc_ids_recalculated > 0 {
                        println!(
                            "Dc Tables: assigned {} new IDs, recalculated {} Dc IDs (max Short ID: {}).",
                            stats.new_ids_assigned, stats.dc_ids_recalculated, stats.max_short_id
                        );
                    }
                }
                Err(e) => {
                    eprintln!("Error assigning Dc IDs: {e}");
                    return ExitCode::FAILURE;
                }
            }

            match assign_and_update_format_categories(root) {
                Ok(stats) => {
                    if stats.new_ids_assigned > 0 || stats.dc_ids_recalculated > 0 {
                        println!(
                            "Format Tables: assigned {} new IDs, recalculated {} Dc IDs (max Short ID: {}).",
                            stats.new_ids_assigned, stats.dc_ids_recalculated, stats.max_short_id
                        );
                    }
                }
                Err(e) => {
                    eprintln!("Error assigning Format IDs: {e}");
                    return ExitCode::FAILURE;
                }
            }
        }
    }

    let report = validate_all_data_tables();
    println!("{}", report.format_report());

    if report.has_errors() {
        return ExitCode::FAILURE;
    }

    if !check_only {
        if let Some(ref root) = repo_root {
            match generate_merged_csvs(root) {
                Ok(stats) => {
                    println!(
                        "Successfully generated DcList.generated.csv ({} records) and formats.generated.csv ({} records).",
                        stats.dc_records_merged, stats.format_records_merged
                    );
                }
                Err(e) => {
                    eprintln!("Error generating merged CSV tables: {e}");
                    return ExitCode::FAILURE;
                }
            }
        }
    }

    ExitCode::SUCCESS
}
