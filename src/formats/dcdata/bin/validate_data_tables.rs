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

use ctb_formats_dcdata::{
    assign_and_update_dc_categories, assign_and_update_format_categories,
    check_category_csvs_need_no_id_updates, find_repository_root,
    generate_merged_csvs, validate_all_data_tables,
    validate_all_data_tables_from_repo,
};
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let mut write_mode = false;
    let mut process_inbox = false;
    let mut dry_run = false;
    let mut plan_reconciliation_manifest: Option<PathBuf> = None;
    let mut require_manifest: Option<PathBuf> = None;

    let mut i = 1usize;
    while i < args.len() {
        let Some(arg) = args.get(i).map(String::as_str) else {
            break;
        };
        match arg {
            "--write" | "-w" => {
                write_mode = true;
            }
            "--process-inbox" => {
                process_inbox = true;
            }
            "--dry-run" => {
                dry_run = true;
            }
            "--plan-reconciliation-manifest" => {
                i = i.saturating_add(1);
                let Some(val) = args.get(i) else {
                    eprintln!(
                        "Error: --plan-reconciliation-manifest requires a <path> argument"
                    );
                    return ExitCode::FAILURE;
                };
                plan_reconciliation_manifest = Some(PathBuf::from(val));
            }
            "--require-manifest" => {
                i = i.saturating_add(1);
                let Some(val) = args.get(i) else {
                    eprintln!(
                        "Error: --require-manifest requires a <path> argument"
                    );
                    return ExitCode::FAILURE;
                };
                require_manifest = Some(PathBuf::from(val));
            }
            other => {
                eprintln!("Error: Unknown flag '{other}'");
                return ExitCode::FAILURE;
            }
        }
        i = i.saturating_add(1);
    }

    if let Some(manifest_out_path) = plan_reconciliation_manifest {
        let repo_root = match find_repository_root() {
            Ok(path) => path,
            Err(e) => {
                eprintln!(
                    "Error: Could not determine repository root for --plan-reconciliation-manifest ({e})"
                );
                return ExitCode::FAILURE;
            }
        };
        if let Err(e) = check_category_csvs_need_no_id_updates(&repo_root) {
            eprintln!("Error checking category CSV IDs: {e:#}");
            return ExitCode::FAILURE;
        }
        let report = validate_all_data_tables_from_repo(&repo_root);
        if report.has_errors() {
            println!("{}", report.format_report());
            return ExitCode::FAILURE;
        }
        // Reason for fallback: manifest output filename stem defaults to "step3-reconcile-csv-graph".
        let manifest_id = manifest_out_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("step3-reconcile-csv-graph")
            .replace('_', "-");
        match ctb_formats_dcdata::graph_ledger::plan_csv_reconciliation_manifest(
            &repo_root,
            &manifest_id,
            &manifest_out_path,
        ) {
            Ok(manifest) => {
                println!(
                    "Planned CSV reconciliation manifest {} ({} items, baseline l{} -> final l{}, {} total nodes).",
                    manifest_out_path.display(),
                    manifest.planned_items.len(),
                    manifest.baseline_last_id,
                    manifest.final_last_id,
                    manifest.final_node_count,
                );
                return ExitCode::SUCCESS;
            }
            Err(e) => {
                eprintln!(
                    "Error planning CSV reconciliation manifest: {e:#}"
                );
                return ExitCode::FAILURE;
            }
        }
    }

    if process_inbox {
        let repo_root = match find_repository_root() {
            Ok(path) => path,
            Err(e) => {
                eprintln!(
                    "Error: Could not determine repository root for --process-inbox ({e})"
                );
                return ExitCode::FAILURE;
            }
        };
        match ctb_formats_dcdata::graph_ledger::process_inbox_nodes(
            &repo_root,
            require_manifest.as_deref(),
            dry_run,
        ) {
            Ok(outcome) => {
                println!(
                    "Inbox Nodes{}: {} appended ({} total nodes, {} chunk files touched).",
                    if dry_run { " (dry-run)" } else { "" },
                    outcome.appended_count,
                    outcome.total_nodes,
                    outcome.files_written,
                );
                return ExitCode::SUCCESS;
            }
            Err(e) => {
                eprintln!("Error processing inbox nodes: {e:#}");
                return ExitCode::FAILURE;
            }
        }
    }

    if !write_mode {
        // Work entirely in memory using the compiled-in asset bundle
        let report = validate_all_data_tables();
        println!("{}", report.format_report());

        if report.has_errors() {
            return ExitCode::FAILURE;
        }

        return ExitCode::SUCCESS;
    }

    // Write mode: Read and write directly from the filesystem
    let repo_root = match find_repository_root() {
        Ok(path) => path,
        Err(e) => {
            eprintln!(
                "Error: Could not determine repository root for --write mode ({e})"
            );
            return ExitCode::FAILURE;
        }
    };

    if require_manifest.is_some() || dry_run {
        if let Err(e) = check_category_csvs_need_no_id_updates(&repo_root) {
            eprintln!("Error checking category CSV IDs: {e:#}");
            return ExitCode::FAILURE;
        }
    } else {
        match assign_and_update_dc_categories(&repo_root) {
            Ok(stats) => {
                if stats.new_ids_assigned > 0 || stats.dc_ids_recalculated > 0 {
                    println!(
                        "Dc Tables: assigned {} new IDs, recalculated {} Dc IDs (max Short ID: {}).",
                        stats.new_ids_assigned,
                        stats.dc_ids_recalculated,
                        stats.max_short_id
                    );
                }
            }
            Err(e) => {
                eprintln!("Error assigning Dc IDs: {e}");
                return ExitCode::FAILURE;
            }
        }

        match assign_and_update_format_categories(&repo_root) {
            Ok(stats) => {
                if stats.new_ids_assigned > 0 || stats.dc_ids_recalculated > 0 {
                    println!(
                        "Format Tables: assigned {} new IDs, recalculated {} Dc IDs (max Short ID: {}).",
                        stats.new_ids_assigned,
                        stats.dc_ids_recalculated,
                        stats.max_short_id
                    );
                }
            }
            Err(e) => {
                eprintln!("Error assigning Format IDs: {e}");
                return ExitCode::FAILURE;
            }
        }
    }

    let report = validate_all_data_tables_from_repo(&repo_root);
    println!("{}", report.format_report());

    if report.has_errors() {
        return ExitCode::FAILURE;
    }

    // Execute or dry-run the durable graph ledger reconciliation BEFORE mutating merged CSVs
    // or .generated.rs files so any manifest mismatch or prevalidation failure leaves zero files modified (R7-F3).
    match ctb_formats_dcdata::graph_ledger::update_system_graph_ledger_with_options(
        &repo_root,
        require_manifest.as_deref(),
        dry_run,
        None,
    ) {
        Ok(stats) => {
            if stats.nodes_appended > 0 || stats.files_written > 0 || dry_run {
                println!(
                    "System Graph Ledger{}: {} total nodes ({} appended, {} superseded, {} retracted, {} chunk files updated).",
                    if dry_run { " (dry-run)" } else { "" },
                    stats.total_nodes,
                    stats.nodes_appended,
                    stats.nodes_superseded,
                    stats.nodes_retracted,
                    stats.files_written
                );
            }
        }
        Err(e) => {
            eprintln!("Error updating System graph ledger (assets/data/graph/*.ctbnodes): {e:#}");
            return ExitCode::FAILURE;
        }
    }

    if dry_run {
        return ExitCode::SUCCESS;
    }

    match generate_merged_csvs(&repo_root) {
        Ok(stats) => {
            println!(
                "Successfully generated DcList.generated.csv ({} records), formats.generated.csv ({} records), unicode.generated.csv ({} records), and all.generated.csv ({} records).",
                stats.dc_records_merged,
                stats.format_records_merged,
                stats.unicode_records_merged,
                stats.total_records_merged
            );
        }
        Err(e) => {
            eprintln!("Error generating merged CSV tables: {e}");
            return ExitCode::FAILURE;
        }
    }

    match ctb_formats_dcdata::updater::generate_format_id_file(&repo_root) {
        Ok(updated) => {
            if updated {
                println!("Successfully updated format_id.generated.rs from system graph.");
            }
        }
        Err(e) => {
            eprintln!("Error generating format_id.generated.rs: {e}");
            return ExitCode::FAILURE;
        }
    }

    match ctb_build_support::dc_codegen::generate_dc_file(&repo_root) {
        Ok(updated) => {
            if updated {
                println!("Successfully updated dcdata/dc.generated.rs from system graph.");
            }
        }
        Err(e) => {
            eprintln!("Error generating dcdata/dc.generated.rs: {e}");
            return ExitCode::FAILURE;
        }
    }

    match ctb_build_support::dc_codegen::generate_role_file(&repo_root) {
        Ok(updated) => {
            if updated {
                println!("Successfully updated dcdata/syntax/role.generated.rs from system graph.");
            }
        }
        Err(e) => {
            eprintln!("Error generating dcdata/syntax/role.generated.rs: {e}");
            return ExitCode::FAILURE;
        }
    }

    match ctb_build_support::extension_codegen::generate_extension_data_file(&repo_root) {
        Ok(updated) => {
            if updated {
                println!("Successfully updated extension_data.generated.rs from system graph.");
            }
        }
        Err(e) => {
            eprintln!("Error generating extension_data.generated.rs: {e}");
            return ExitCode::FAILURE;
        }
    }

    match ctb_build_support::encoding_codegen::generate_encoding_file(&repo_root) {
        Ok(updated) => {
            if updated {
                println!("Successfully updated encoding.generated.rs from system graph.");
            }
        }
        Err(e) => {
            eprintln!("Error generating encoding.generated.rs: {e}");
            return ExitCode::FAILURE;
        }
    }

    match ctb_build_support::format_support_codegen::generate_format_support_file(&repo_root) {
        Ok(updated) => {
            if updated {
                println!("Successfully updated format_support.generated.rs from system graph.");
            }
        }
        Err(e) => {
            eprintln!("Error generating format_support.generated.rs: {e}");
            return ExitCode::FAILURE;
        }
    }

    if repo_root.join("built/ctoolbox.rsrc").is_file() {
        let options = ctb_build_support::asset_packer::PrepareOptions {
            prepare_runtime_bundle: true,
            prepare_minimal_assets: false,
            include_rust_docs: false,
            archive_source: false,
            write_debug_stubs: true,
        };
        if let Err(e) = ctb_build_support::asset_packer::prepare_assets(&repo_root, &options) {
            eprintln!("Error repacking runtime resource bundles (built/ctoolbox.rsrc): {e}");
            return ExitCode::FAILURE;
        }
    }

    ExitCode::SUCCESS
}
