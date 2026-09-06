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

//! Top-level CLI entry point and orchestration for `csc`.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::args::CscArgs;
use crate::copy_engine::execute_copy_pipeline;
use crate::journal::{JournalWriter, read_journal_snapshot};
use crate::path_resolution::{ResolvedCopyTask, resolve_tasks};
use crate::verify_cache::check_cache_flush_privileges;
use ctb_utilities::cli::ToolResult;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Main entry point for the `csc` command called by CLI routing.
pub fn run_csc(args: CscArgs) -> Result<ToolResult> {
    let start_time = Instant::now();

    // 1. Startup root / privilege check and warning
    check_cache_flush_privileges();

    let home_dir = match std::env::var("HOME") {
        Ok(h) => PathBuf::from(h),
        Err(_) => PathBuf::from("."),
    };

    let progress = Progress::new(args.should_show_progress());

    let (tasks, mut journal, snapshot) = if let Some(ref resume_path) = args.resume {
        // Resume mode: recover state from journal
        let journal_path = if resume_path.extension().and_then(|e| e.to_str()) == Some("desc") {
            resume_path.with_extension("journal")
        } else {
            resume_path.clone()
        };

        let desc_path = journal_path.with_extension("desc");

        progress.message(&format!(
            "Resuming session from journal: {}",
            journal_path.display()
        ));

        let snap = read_journal_snapshot(&journal_path)?;
        if snap.is_completed {
            let msg = format!(
                "Session in {} is already marked Completed. No files remaining to resume.\n",
                journal_path.display()
            );
            return Ok(ToolResult::immediate_ok(msg.into_bytes()));
        }

        let mut all_paths = snap.sources.clone();
        all_paths.push(snap.destination.clone());
        let (resolved, _) = resolve_tasks(&all_paths)?;

        let jw = JournalWriter::open_for_resume(&journal_path, &desc_path, &snap)?;
        (resolved, jw, Some(snap))
    } else {
        // New run: resolve tasks from command-line arguments
        anyhow::ensure!(
            args.paths.len() >= 2,
            "Must provide at least one source and one destination path (or pass --resume)"
        );

        let (resolved, dest_path) = resolve_tasks(&args.paths)?;
        let sources: Vec<PathBuf> = resolved.iter().map(|t| t.source_root.clone()).collect();

        let jw = JournalWriter::create_new(&home_dir, &sources, &dest_path)?;
        progress.message(&format!("State journal: {}", jw.journal_path().display()));
        (resolved, jw, None)
    };

    // 2. Execute pipeline
    let stats = execute_copy_pipeline(
        &tasks,
        &args,
        &mut journal,
        snapshot.as_ref(),
        &progress,
    )?;

    let elapsed = start_time.elapsed().as_secs_f64();

    // 3. Format result summary
    let mut summary = String::new();
    summary.push_str("\n--- Checksummed Copy (csc) Summary ---\n");
    summary.push_str(&format!("Files copied:             {}\n", stats.files_copied));
    if stats.files_skipped_identical > 0 {
        summary.push_str(&format!(
            "Files skipped (identical): {}\n",
            stats.files_skipped_identical
        ));
    }
    summary.push_str(&format!("Bytes transferred:        {}\n", stats.bytes_copied));
    summary.push_str(&format!("Directories created:      {}\n", stats.dirs_created));
    summary.push_str(&format!("Symlinks created:         {}\n", stats.symlinks_created));
    summary.push_str(&format!("Hardlinks created:        {}\n", stats.hardlinks_created));
    if stats.special_files_created > 0 {
        summary.push_str(&format!(
            "Special nodes created:    {}\n",
            stats.special_files_created
        ));
    }
    if args.should_verify_after() {
        summary.push_str(&format!("Files verified:           {}\n", stats.files_verified));
    }
    summary.push_str(&format!("Duration:                 {:.2}s\n", elapsed));
    summary.push_str("Status:                   All operations verified successfully.\n");

    Ok(ToolResult::immediate_ok(summary.into_bytes()))
}
