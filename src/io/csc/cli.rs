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
use crate::path_resolution::resolve_tasks;
use crate::verify_cache::check_cache_flush_privileges;
use ctb_utilities::cli::ToolResult;
use std::path::PathBuf;
use std::time::Instant;

/// Main entry point for the `csc` command called by CLI routing.
pub fn run_csc(args: CscArgs) -> Result<ToolResult> {
    let start_time = Instant::now();

    // 1. Startup root / privilege check and warning
    check_cache_flush_privileges();

    let home_dir = if let Some(ref custom_dir) = args.state_dir {
        custom_dir.clone()
    } else {
        match std::env::var("HOME") {
            Ok(h) => PathBuf::from(h),
            Err(_) => PathBuf::from("."),
        }
    };

    let progress = Progress::new(args.should_show_progress());

    let (tasks, mut journal, snapshot) = if let Some(ref resume_path) = args.resume {
        // Resume mode: recover state from journal
        let journal_path = if resume_path.extension().and_then(|e| e.to_str()) == Some("cscdesc") {
            resume_path.with_extension("cscjournal")
        } else {
            resume_path.clone()
        };

        let desc_path = journal_path.with_extension("cscdesc");

        progress.message(&format!(
            "Resuming session from journal: {}",
            journal_path.display()
        ));

        let snap = read_journal_snapshot(&journal_path)?;
        if snap.is_completed {
            progress.message("Session was completed; revalidating source and destination entries.");
        }

        let mut all_paths = snap.sources.clone();
        all_paths.push(snap.destination.clone());
        let (mut resolved, _) = resolve_tasks(&all_paths)?;
        for task in &mut resolved {
            if !std::fs::symlink_metadata(&task.source_root)?.is_dir() {
                continue;
            }
            let source_entity = ctb_io::file::FileEntity::from_filesystem_metadata_only(&task.source_root, None)?;
            let recorded_root = snap.committed_entities.values().find(|entity| {
                entity.is_dir() && matches!((&entity.identity.origin, &source_entity.identity.origin),
                    (ctb_io::file::FileOrigin::Filesystem { key, .. },
                     ctb_io::file::FileOrigin::Filesystem { key: source_key, .. })
                    if key == source_key)
            });
            if let Some(entity) = recorded_root {
                task.target_root = snap.destination.join(&entity.identity.relative_path);
            } else {
                anyhow::ensure!(task.copy_contents_only || !snap.destination.exists(),
                    "Cannot safely recover the original directory destination from this journal");
            }
        }
        crate::path_resolution::validate_task_overlap(&resolved)?;

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

        let jw = if let Some(ref custom_jp) = args.journal_path {
            let journal_path = if custom_jp.extension().and_then(|e| e.to_str()) == Some("cscdesc") {
                custom_jp.with_extension("cscjournal")
            } else if custom_jp.extension().and_then(|e| e.to_str()) == Some("cscjournal") {
                custom_jp.clone()
            } else {
                custom_jp.with_extension("cscjournal")
            };
            let desc_path = journal_path.with_extension("cscdesc");
            anyhow::ensure!(
                !journal_path.exists() && !desc_path.exists(),
                "Journal file already exists at {}: will not overwrite existing file (pass --resume to resume an interrupted run)",
                journal_path.display()
            );
            JournalWriter::create_at_path(&journal_path, &desc_path, &sources, &dest_path)?
        } else {
            JournalWriter::create_new(&home_dir, &sources, &dest_path)?
        };
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

    let journal_path = journal.journal_path().to_path_buf();
    let desc_path = journal.desc_path().to_path_buf();
    drop(journal);

    if args.delete_manifest_after {
        if journal_path.exists() {
            let _ = std::fs::remove_file(&journal_path);
        }
        if desc_path.exists() {
            let _ = std::fs::remove_file(&desc_path);
        }
    }

    let elapsed = start_time.elapsed().as_secs_f64();

    // 3. Format result summary
    let mut summary = String::new();
    use std::fmt::Write;
    writeln!(summary, "--- CSC Summary ---")?;
    writeln!(summary, "Files copied:             {}", stats.files_copied)?;
    if stats.files_skipped_identical > 0 {
        writeln!(
            summary,
            "Files skipped (identical): {}",
            stats.files_skipped_identical
        )?;
    }
    writeln!(summary, "Bytes transferred:        {}", stats.bytes_copied)?;
    writeln!(summary, "Directories created:      {}", stats.dirs_created)?;
    writeln!(summary, "Symlinks created:         {}", stats.symlinks_created)?;
    writeln!(summary, "Hardlinks created:        {}", stats.hardlinks_created)?;
    if stats.special_files_created > 0 {
        writeln!(
            summary,
            "Special nodes created:    {}",
            stats.special_files_created
        )?;
    }
    if stats.special_files_skipped > 0 {
        writeln!(
            summary,
            "Special files skipped:    {}",
            stats.special_files_skipped
        )?;
        writeln!(
            summary,
            "WARNING: {} special file(s) were skipped. Pass --copy-specials-as-specials to preserve them as device nodes, or --copy-block-devices-as-regular-files to copy block devices as disk image files.",
            stats.special_files_skipped
        )?;
    }
    if args.should_verify_after() {
        writeln!(summary, "Files verified:           {}", stats.files_verified)?;
    }
    writeln!(summary, "Duration:                 {elapsed:.2}s")?;
    writeln!(summary, "Status:                   All operations verified successfully.")?;

    Ok(ToolResult::immediate_ok(summary.into_bytes()))
}
