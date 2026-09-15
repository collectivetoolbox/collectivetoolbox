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
use crate::verifier::check_cache_flush_privileges;
use ctb_utilities::cli::ToolResult;
use std::path::PathBuf;
use std::time::Instant;

/// Main entry point for the `csc` command called by CLI routing.
pub fn run_csc(args: CscArgs) -> Result<ToolResult> {
    let _env_scope =
        ctb_utilities::environment::GlobalEnvironmentScope::enter_fresh();
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

    let (tasks, mut journal) = if let Some(ref resume_path) = args.resume {
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
        (resolved, jw)
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
        (resolved, jw)
    };

    // 2. Execute pipeline
    let stats = execute_copy_pipeline(
        &tasks,
        &args,
        &mut journal,
        &progress,
    )?;

    let journal_path = journal.journal_path().to_path_buf();
    let desc_path = journal.desc_path().to_path_buf();
    drop(journal);

    let retain_metadata_journal = stats.copied_entities.iter().any(|(_, _, entity)|
        entity.metadata.native.is_some() || entity.metadata.timestamps.birthtime_sec.is_some()
        || entity.metadata.platform_raw_flags.is_some() || !entity.streams.is_empty());
    if args.delete_manifest_after && retain_metadata_journal {
        warn_fmt!("Retaining {}: it contains original metadata that is not guaranteed to be reproducible", journal_path.display());
    }
    if args.delete_manifest_after && !retain_metadata_journal {
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
    if retain_metadata_journal {
        writeln!(summary, "Original metadata journal: {}", journal_path.display())?;
    }
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
#[cfg(all(test, unix))]
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
    use crate::args::{default_test_args};
    use crate::cli::run_csc;
    use crate::journal::find_cscjournal;
    use ctb_utilities::cli::ToolResult;
    use std::fs;
    use std::path::{PathBuf};
    use tempfile::tempdir;

    #[crate::ctb_test]
    fn test_multiple_sources_to_dest_dir() {
        let temp = tempdir().expect("create tempdir");
        let src1 = temp.path().join("src1");
        let src2 = temp.path().join("src2");
        let dest = temp.path().join("dest_multi");
        let state = temp.path().join("state_dir");
        fs::create_dir_all(&src1).expect("create src1");
        fs::create_dir_all(&src2).expect("create src2");
        fs::create_dir_all(&state).expect("create state");

        fs::write(src1.join("a.txt"), b"Content A").expect("write a");
        fs::write(src2.join("b.txt"), b"Content B").expect("write b");

        let args = default_test_args(
            vec![src1.clone(), src2.clone(), dest.clone()],
            state,
        );

        run_csc(args).expect("run csc multiple sources");

        assert_eq!(
            fs::read(dest.join("src1").join("a.txt")).expect("read a"),
            b"Content A"
        );
        assert_eq!(
            fs::read(dest.join("src2").join("b.txt")).expect("read b"),
            b"Content B"
        );
    }

    #[crate::ctb_test]
    fn test_copy_single_file_to_directory() {
        let temp = tempdir().expect("create tempdir");
        let src_file = temp.path().join(".face");
        let dest_dir = temp.path().join("dest_dir");
        let state = temp.path().join("state_dir");
        fs::create_dir_all(&dest_dir).expect("create dest");
        fs::create_dir_all(&state).expect("create state");

        fs::write(&src_file, b"face icon bytes").expect("write face");

        // Test case 1: dest with trailing slash (e.g. csc ~/.face ./)
        let dest_with_slash = PathBuf::from(format!("{}/", dest_dir.display()));
        let args = default_test_args(
            vec![src_file.clone(), dest_with_slash],
            state.clone(),
        );

        run_csc(args).expect("run csc single file with trailing slash");
        assert_eq!(
            fs::read(dest_dir.join(".face")).expect("read copied file"),
            b"face icon bytes"
        );

        // Test case 2: dest without trailing slash (e.g. csc ~/.face .)
        let dest_dir2 = temp.path().join("dest_dir2");
        let state2 = temp.path().join("state_dir2");
        fs::create_dir_all(&dest_dir2).expect("create dest2");
        fs::create_dir_all(&state2).expect("create state2");

        let args2 = default_test_args(
            vec![src_file.clone(), dest_dir2.clone()],
            state2,
        );
        run_csc(args2).expect("run csc single file to directory without trailing slash");
        assert_eq!(
            fs::read(dest_dir2.join(".face")).expect("read copied file 2"),
            b"face icon bytes"
        );

        // Test case 3: direct file rename (e.g. csc ~/.face renamed.face)
        let renamed_file = temp.path().join("renamed.face");
        let state3 = temp.path().join("state_dir3");
        fs::create_dir_all(&state3).expect("create state3");

        let args3 = default_test_args(
            vec![src_file.clone(), renamed_file.clone()],
            state3,
        );
        run_csc(args3).expect("run csc single file rename");
        assert_eq!(
            fs::read(&renamed_file).expect("read renamed file"),
            b"face icon bytes"
        );
    }

    #[crate::ctb_test]
    fn test_csc_configurable_journal_path() {
        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src");
        let dest = temp.path().join("dest");
        let state = temp.path().join("state");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&state).expect("create state");
        fs::write(src.join("test.txt"), b"Configurable journal test").expect("write test.txt");

        let custom_journal = state.join("my_backup.cscjournal");
        let custom_desc = state.join("my_backup.cscdesc");

        let mut args = default_test_args(vec![src.clone(), dest.clone()], state.clone());
        args.journal_path = Some(custom_journal.clone());

        let res = run_csc(args.clone()).expect("run csc with explicit journal path");
        match res {
            ToolResult::Immediate { exit_code, .. } => assert_eq!(exit_code, 0),
            _ => panic!("Expected immediate tool result"),
        }

        assert!(custom_journal.exists(), "Explicit .cscjournal must be created");
        assert!(custom_desc.exists(), "Explicit .cscdesc must be created");
        assert!(dest.join("test.txt").exists(), "Destination file must exist");

        // Attempting to run again with the same journal path without --resume must fail
        let err = match run_csc(args) {
            Err(e) => e,
            Ok(_) => panic!("Expected error when journal already exists"),
        };
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("already exists"),
            "Must reject overwriting existing journal file: {err_msg}"
        );
    }

    #[cfg(target_os = "linux")]

    #[crate::ctb_test]
    fn test_csc_delete_manifest_after() {
        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_del_manifest");
        let dest = temp.path().join("dest_del_manifest");
        let state = temp.path().join("state_del_manifest");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&state).expect("create state dir");

        fs::write(src.join("test.txt"), b"Manifest should be deleted after copy").expect("write file");

        let mut args = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state.clone(),
        );
        args.delete_manifest_after = true;

        let res = run_csc(args).expect("run csc with delete_manifest_after");
        match res {
            ToolResult::Immediate { stdout, .. } => {
                let out = String::from_utf8_lossy(&stdout);
                assert!(out.contains("Files copied:             1"));
            }
            _ => panic!("Expected Immediate ToolResult"),
        }

        assert!(dest.join("test.txt").exists());

        let journal = find_cscjournal(&state);
        let snapshot = crate::journal::read_journal_snapshot(&journal).unwrap();
        assert!(snapshot.committed_entities.values().any(|entity| entity.metadata.native.is_some()));
        assert!(journal.with_extension("cscdesc").exists());
    }
}

