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

//! Move engine providing atomic zero-copy rename on the same filesystem and
//! verified checksummed copy fallback across filesystem boundaries.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::args::{CscArgs, MvArgs, SourceChangePolicy};
use crate::copy_engine::execute_copy_pipeline;
use crate::journal::JournalWriter;
use crate::path_resolution::resolve_tasks;
use ctb_utilities::cli::ToolResult;
use std::fmt::Write as _;
use std::path::PathBuf;
use std::time::Instant;

/// Executes the `mv` command.
///
/// Attempts an atomic O(1) filesystem rename. When encountering `EXDEV`
/// (cross-filesystem link), falls back to executing the verified checksummed
/// copy pipeline (`csc --delete-manifest-after`) and unlinks source entities
/// only after verification completes successfully.
pub fn run_mv(args: MvArgs) -> Result<ToolResult> {
    let start_time = Instant::now();
    anyhow::ensure!(
        args.paths.len() >= 2,
        "Must provide at least one source and one destination path"
    );

    let progress = Progress::new(args.should_show_progress());
    let (tasks, _dest_path) = resolve_tasks(&args.paths)?;

    let mut moved_renamed: u64 = 0;
    let mut moved_copied: u64 = 0;
    let mut bytes_copied: u64 = 0;

    let home_dir = match std::env::var("HOME") {
        Ok(h) => PathBuf::from(h),
        Err(_) => PathBuf::from("."),
    };

    for task in &tasks {
        if args.dry_run {
            progress.message(&format!(
                "[Dry run] Move {} -> {}",
                task.source_root.display(),
                task.target_root.display()
            ));
            moved_renamed = moved_renamed.saturating_add(1);
            continue;
        }

        // Fast path: atomic rename on the same filesystem
        let rename_res = if !task.copy_contents_only {
            if let Some(parent) = task.target_root.parent() {
                if !parent.as_os_str().is_empty() && !parent.exists() {
                    std::fs::create_dir_all(parent).with_context(|| {
                        format!("Failed to create destination directory: {}", parent.display())
                    })?;
                }
            }
            std::fs::rename(&task.source_root, &task.target_root)
        } else {
            Err(std::io::Error::from_raw_os_error(nix::libc::EXDEV))
        };

        match rename_res {
            Ok(()) => {
                moved_renamed = moved_renamed.saturating_add(1);
                if args.verbose {
                    progress.message(&format!(
                        "Renamed {} -> {}",
                        task.source_root.display(),
                        task.target_root.display()
                    ));
                }
            }
            Err(err) if err.raw_os_error() == Some(nix::libc::EXDEV) => {
                progress.message(&format!(
                    "Cross-device move detected for {}: copying with verification...",
                    task.source_root.display()
                ));

                let csc_args = CscArgs {
                    paths: vec![task.source_root.clone(), task.target_root.clone()],
                    state_dir: None,
                    resume: None,
                    journal_path: None,
                    verbose: args.verbose,
                    progress: args.progress,
                    no_progress: args.no_progress,
                    verify_after: args.verify_after,
                    no_verify_after: args.no_verify_after,
                    always_overwrite: args.force,
                    skip_existing_checksum: true,
                    on_source_change: SourceChangePolicy::Error,
                    copy_specials_as_specials: true,
                    copy_block_devices_as_regular_files: false,
                    one_file_system: false,
                    best_effort_metadata: args.best_effort_metadata,
                    check_atime: false,
                    delete_manifest_after: true,
                    recursive: true,
                    archive: true,
                    dry_run: false,
                };

                let mut journal = JournalWriter::create_new(
                    &home_dir,
                    &[task.source_root.clone()],
                    &task.target_root,
                )?;

                let copy_stats = execute_copy_pipeline(
                    std::slice::from_ref(task),
                    &csc_args,
                    &mut journal,
                    None,
                    &progress,
                )?;

                // Delete manifest files upon successful copy
                let j_path = journal.journal_path().to_path_buf();
                let d_path = journal.desc_path().to_path_buf();
                drop(journal);
                if j_path.exists() {
                    let _ = std::fs::remove_file(&j_path);
                }
                if d_path.exists() {
                    let _ = std::fs::remove_file(&d_path);
                }

                // Copy and verification succeeded; unlink source
                if task.source_root.is_dir() {
                    if task.copy_contents_only {
                        for entry in std::fs::read_dir(&task.source_root)? {
                            let entry = entry?;
                            let path = entry.path();
                            if path.is_dir() {
                                std::fs::remove_dir_all(&path)?;
                            } else {
                                std::fs::remove_file(&path)?;
                            }
                        }
                    } else {
                        std::fs::remove_dir_all(&task.source_root).with_context(|| {
                            format!(
                                "Failed to remove source directory after copy: {}",
                                task.source_root.display()
                            )
                        })?;
                    }
                } else {
                    std::fs::remove_file(&task.source_root).with_context(|| {
                        format!(
                            "Failed to remove source file after copy: {}",
                            task.source_root.display()
                        )
                    })?;
                }

                moved_copied = moved_copied.saturating_add(copy_stats.files_copied);
                bytes_copied = bytes_copied.saturating_add(copy_stats.bytes_copied);
            }
            Err(err) => {
                return Err(err).with_context(|| {
                    format!(
                        "Failed to move '{}' to '{}'",
                        task.source_root.display(),
                        task.target_root.display()
                    )
                });
            }
        }
    }

    let elapsed = start_time.elapsed().as_secs_f64();
    let mut summary = String::new();
    writeln!(summary, "--- MV Summary ---")?;
    writeln!(summary, "Items renamed (same filesystem): {}", moved_renamed)?;
    if moved_copied > 0 {
        writeln!(summary, "Files copied (cross-device):     {}", moved_copied)?;
        writeln!(summary, "Bytes transferred:               {}", bytes_copied)?;
    }
    writeln!(summary, "Duration:                        {elapsed:.2}s")?;
    writeln!(summary, "Status:                          Move completed successfully.")?;

    Ok(ToolResult::immediate_ok(summary.into_bytes()))
}
