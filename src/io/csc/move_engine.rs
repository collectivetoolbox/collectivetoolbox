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
use crate::journal::StateJournal;
use crate::path_resolution::resolve_tasks;
use ctb_io::file::{FileEntity, FileOrigin, is_cross_device_error, verify_materialized_entity};
use ctb_utilities::cli::ToolResult;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::Instant;

fn rename_with_policy(source: &Path, destination: &Path, policy: ctb_io::file::ExistingEntryPolicy) -> Result<()> {
    let source_parent = match source.parent().filter(|path| !path.as_os_str().is_empty()) {
        Some(parent) => parent,
        None => Path::new("."),
    };
    let destination_parent = match destination.parent().filter(|path| !path.as_os_str().is_empty()) {
        Some(parent) => parent,
        None => Path::new("."),
    };
    let source_name = source.file_name().context("Missing source entry name")?;
    let destination_name = destination.file_name().context("Missing destination entry name")?;
    let source_dir = ctb_io::file::SandboxableDir::open(source_parent)?;
    let destination_dir = ctb_io::file::SandboxableDir::open(destination_parent)?;
    destination_dir.rename_from_parent(
        &source_dir.root_fd(), source_name, &destination_dir.root_fd(), destination_name, policy,
    )
}

/// Executes the `mv` command.
///
/// Attempts an atomic O(1) filesystem rename. When encountering `EXDEV`
/// (cross-filesystem link), falls back to executing the verified checksummed
/// copy pipeline (`csc --delete-manifest-after`) and unlinks source entities
/// only after verification completes successfully.
pub fn run_mv(args: MvArgs) -> Result<ToolResult> {
    let _env_scope = if args.full_provenance {
        ctb_io_environment::GlobalEnvironmentScope::enter_full()
    } else {
        ctb_io_environment::GlobalEnvironmentScope::enter_fresh()
    };
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
    let mut retained_journals = Vec::new();
    let mut warning_count = 0_usize;

    for task in &tasks {
        if !args.best_effort_metadata && !args.allow_unknown_fs {
            if let Ok(src_meta) = std::fs::symlink_metadata(&task.source_root) {
                let src_fs = ctb_io::file::query_filesystem_info(&task.source_root, &src_meta);
                if src_fs.fs_type == "unknown" {
                    anyhow::bail!(
                        "Cannot detect filesystem type for source '{}'. Pass --best-effort-metadata or --allow-unknown-fs to proceed.",
                        task.source_root.display()
                    );
                }
            }
            let tgt_existing = if task.target_root.exists() {
                task.target_root.clone()
            } else {
                match ctb_io::file::resolve_existing_ancestors(&task.target_root) {
                    Ok(ancestor) => ancestor,
                    Err(_) => PathBuf::from("."),
                }
            };
            if let Ok(tgt_meta) = std::fs::symlink_metadata(&tgt_existing) {
                let tgt_fs = ctb_io::file::query_filesystem_info(&tgt_existing, &tgt_meta);
                if tgt_fs.fs_type == "unknown" {
                    anyhow::bail!(
                        "Cannot detect filesystem type for destination '{}'. Pass --best-effort-metadata or --allow-unknown-fs to proceed.",
                        task.target_root.display()
                    );
                }
            }
        }

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
            rename_with_policy(&task.source_root, &task.target_root,
                if args.force { ctb_io::file::ExistingEntryPolicy::Replace }
                else { ctb_io::file::ExistingEntryPolicy::Refuse })
        } else {
            Err(std::io::Error::from(std::io::ErrorKind::CrossesDevices).into())
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
            Err(err) if err.downcast_ref::<std::io::Error>().is_some_and(is_cross_device_error) => {
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
                    verify_after: true,
                    no_verify_after: false,
                    overwrite: args.force,
                    on_source_change: SourceChangePolicy::Error,
                    copy_specials_as_specials: true,
                    copy_block_devices_as_regular_files: false,
                    copy_fifos_as_regular_files: false,
                    one_file_system: false,
                    best_effort_metadata: args.best_effort_metadata,
                    allow_unknown_fs: args.allow_unknown_fs,
                    check_atime: false,
                    check_ctime: false,
                    check_birthtime: false,
                    strict: false,
                    delete_manifest_after: true,
                    recursive: true,
                    archive: true,
                    dry_run: false,
                    read_apple_double: None,
                    read_apple_double_alongside: false,
                    no_read_apple_double_alongside: false,
                    read_apple_double_zip: false,
                    read_apple_double_netatalk: false,
                    read_apple_single: false,
                    read_apple_single_without_extension: false,
                    no_read_apple_single: false,
                    read_apple_single_with_extension: Vec::new(),
                    maybe_write_apple_double: None,
                    force_write_apple_double: None,
                    maybe_write_apple_single: false,
                    force_write_apple_single: false,
                    write_apple_single_with_extension: None,
                    write_apple_single_without_extension: false,
                    full_provenance: args.full_provenance,
                    continue_on_error: false,
                    allow_journal_errors: false,
                };

                let mut temp_state_dir = tempfile::tempdir()
                    .context("Failed to create temporary directory for move state journal")?;
                let mut journal = StateJournal::create_new(
                    temp_state_dir.path(),
                    &[task.source_root.clone()],
                    &task.target_root,
                )?;

                temp_state_dir.disable_cleanup(true);
                let j_path = journal.journal_path().to_path_buf();
                let copy_result = (|| {
                    let copy_stats = execute_copy_pipeline(
                        std::slice::from_ref(task),
                        &csc_args,
                        &mut journal,
                        &progress,
                    )?;

                    anyhow::ensure!(copy_stats.special_files_skipped == 0,
                        "Move refused: some source entries could not be copied; all sources retained");
                    anyhow::ensure!(copy_stats.files_failed == 0 && journal.errors().is_empty(),
                        "Move refused: copy recorded failures; all sources retained");
                    let strict_lossless = !args.best_effort_metadata && !args.allow_unknown_fs;
                    remove_copied_sources(&copy_stats.copied_entities, &task.source_root, task.copy_contents_only, strict_lossless)?;
                    Ok(copy_stats)
                })();
                let copy_stats = match copy_result {
                    Ok(stats) => stats,
                    Err(error) => {
                        if let Err(record_error) = journal.mark_failed(&format!("{error:#}")) {
                            return Err(error).context(format!(
                                "Move journal retained at {}; recording failure also failed: {record_error:#}",
                                j_path.display()
                            ));
                        }
                        return Err(error).context(format!(
                            "Move journal retained at {}", j_path.display()
                        ));
                    }
                };

                // Delete manifest files upon successful copy
                warning_count = warning_count.saturating_add(journal.warnings().len());
                for warning in journal.warnings() {
                    progress.message(&format!("[{}] {}", warning.code, warning.message));
                }
                let retain_metadata = !journal.warnings().is_empty()
                    || copy_stats.copied_entities.iter().any(|(_, _, entity)|
                        crate::journal::entity_requires_metadata_retention(entity));
                drop(journal);
                if retain_metadata {
                    progress.message(&format!("Original metadata retained in {}", j_path.display()));
                    retained_journals.push(j_path);
                } else {
                    temp_state_dir.close().with_context(|| format!(
                        "Failed to remove completed move journal {}", j_path.display()
                    ))?;
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
    for journal in retained_journals {
        writeln!(summary, "Original metadata journal:       {}", journal.display())?;
    }
    if warning_count > 0 {
        writeln!(summary, "Status:                          Move completed with {warning_count} warning record(s); fidelity limitations are retained in the journal.")?;
    } else {
        writeln!(summary, "Status:                          Move completed successfully.")?;
    }

    Ok(ToolResult::immediate_ok(summary.into_bytes()))
}

pub(crate) fn remove_copied_sources(
    entries: &[(PathBuf, PathBuf, FileEntity)],
    source_root: &Path,
    contents_only: bool,
    strict_lossless: bool,
) -> Result<()> {
    for (source, destination, entity) in entries {
        verify_source_identity(source, entity)?;
        verify_materialized_entity(source, entity, strict_lossless)?;
        verify_materialized_entity(destination, entity, strict_lossless)?;
    }
    let mut ordered: Vec<_> = entries.iter().collect();
    ordered.sort_by_key(|(source, _, _)| std::cmp::Reverse(source.components().count()));
    for (source, destination, entity) in ordered {
        verify_source_identity(source, entity)?;
        if entity.is_dir() {
            if !contents_only || source != source_root {
                std::fs::remove_dir(source).with_context(|| format!("Source directory changed or could not be removed: {}", source.display()))?;
            }
        } else {
            verify_materialized_entity(source, entity, strict_lossless)?;
            verify_materialized_entity(destination, entity, strict_lossless)?;
            std::fs::remove_file(source).with_context(|| format!("Failed to remove verified source: {}", source.display()))?;
        }
    }
    Ok(())
}

fn verify_source_identity(source: &Path, expected: &FileEntity) -> Result<()> {
    let actual = FileEntity::from_filesystem_metadata_only(source, None)?;
    match (&actual.identity.origin, &expected.identity.origin) {
        (FileOrigin::Filesystem { key: actual_key, .. }, FileOrigin::Filesystem { key: expected_key, .. }) => {
            anyhow::ensure!(actual_key == expected_key, "Source entry was replaced: {}", source.display());
        }
        _ => anyhow::bail!("Source identity is unavailable: {}", source.display()),
    }
    Ok(())
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
    use crate::args::{MvArgs};
    use crate::move_engine::run_mv;
    use ctb_utilities::cli::ToolResult;
    use std::fs;
    use tempfile::tempdir;

    #[crate::ctb_test]
    fn test_mv_collision_requires_force() -> Result<()> {
        use clap::Parser;
        let temporary = tempdir()?;
        fs::create_dir(temporary.path().join("source-parent"))?;
        fs::create_dir(temporary.path().join("destination-parent"))?;
        let source = temporary.path().join("source-parent/source");
        let destination = temporary.path().join("destination-parent/destination");
        fs::write(&source, b"source payload")?;
        fs::write(&destination, b"destination payload")?;
        let args = MvArgs::try_parse_from([
            std::ffi::OsStr::new("mv"), std::ffi::OsStr::new("--best-effort-metadata"),
            source.as_os_str(), destination.as_os_str(),
        ])?;
        assert!(run_mv(args.clone()).is_err());
        assert_eq!(fs::read(&source)?, b"source payload");
        assert_eq!(fs::read(&destination)?, b"destination payload");
        let mut forced = args;
        forced.force = true;
        run_mv(forced)?;
        assert!(!source.exists());
        assert_eq!(fs::read(destination)?, b"source payload");
        Ok(())
    }

    #[crate::ctb_test]
    #[cfg(unix)]
    fn test_mv_dangling_collision_preserves_entries() -> Result<()> {
        let temporary = tempdir()?;
        let source = temporary.path().join("source");
        let destination = temporary.path().join("destination");
        fs::write(&source, b"source payload")?;
        std::os::unix::fs::symlink("missing", &destination)?;
        assert!(rename_with_policy(&source, &destination, ctb_io::file::ExistingEntryPolicy::Refuse).is_err());
        assert_eq!(fs::read(&source)?, b"source payload");
        assert_eq!(fs::read_link(&destination)?, Path::new("missing"));
        Ok(())
    }

    #[crate::ctb_test]
    fn test_mv_same_filesystem() {
        let _fs_lock = ctb_io::file::FS_CACHE_TEST_MUTEX.lock().unwrap();
        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("mv_src");
        let dest = temp.path().join("mv_dest");
        fs::create_dir_all(&src).expect("create src dir");

        let file_path = src.join("move_me.txt");
        fs::write(&file_path, b"Move this file atomically").expect("write file");

        let target_file = dest.join("move_me.txt");
        let args = MvArgs {
            paths: vec![file_path.clone(), target_file.clone()],
            verbose: true,
            progress: false,
            no_progress: true,
            verify_after: true,
            no_verify_after: false,
            best_effort_metadata: false,
            allow_unknown_fs: false,
            force: false,
            dry_run: false,
            full_provenance: false,
        };

        let res = run_mv(args).expect("run mv");
        match res {
            ToolResult::Immediate { stdout, .. } => {
                let out = String::from_utf8_lossy(&stdout);
                assert!(out.contains("Items renamed (same filesystem): 1"));
                assert!(out.contains("Move completed successfully"));
            }
            _ => panic!("Expected Immediate ToolResult"),
        }

        assert!(!file_path.exists(), "Source file should no longer exist");
        assert!(target_file.exists(), "Destination file should exist");
        let content = fs::read(&target_file).expect("read dest");
        assert_eq!(content, b"Move this file atomically");
    }

    #[crate::ctb_test]
    fn test_mv_directory_tree() {
        let _fs_lock = ctb_io::file::FS_CACHE_TEST_MUTEX.lock().unwrap();
        let temp = tempdir().expect("create tempdir");
        let src_dir = temp.path().join("mv_dir_src");
        let dest_dir = temp.path().join("mv_dir_dest");
        fs::create_dir_all(src_dir.join("subdir")).expect("create src tree");

        fs::write(src_dir.join("file1.txt"), b"Content 1").expect("write file1");
        fs::write(src_dir.join("subdir").join("file2.txt"), b"Content 2").expect("write file2");

        let args = MvArgs {
            paths: vec![src_dir.clone(), dest_dir.clone()],
            verbose: true,
            progress: false,
            no_progress: true,
            verify_after: true,
            no_verify_after: false,
            best_effort_metadata: false,
            allow_unknown_fs: false,
            force: false,
            dry_run: false,
            full_provenance: false,
        };

        let res = run_mv(args).expect("run mv on dir");
        match res {
            ToolResult::Immediate { stdout, .. } => {
                let out = String::from_utf8_lossy(&stdout);
                assert!(out.contains("Move completed successfully"));
            }
            _ => panic!("Expected Immediate ToolResult"),
        }

        assert!(!src_dir.exists(), "Source directory should no longer exist");
        assert!(dest_dir.join("file1.txt").exists());
        assert!(dest_dir.join("subdir").join("file2.txt").exists());
    }

    #[crate::ctb_test]
    fn test_mv_cleanup_retains_uncopied_entries() {
        let temp = tempdir().unwrap();
        let source = temp.path().join("source");
        let destination = temp.path().join("destination");
        fs::create_dir(&source).unwrap();
        fs::create_dir(&destination).unwrap();
        fs::write(source.join("new-data"), b"not copied").unwrap();
        let entity = ctb_io::file::FileEntity::from_filesystem(&source, None).unwrap();
        ctb_io::file::apply_entity_metadata(&destination, None, &entity.metadata, false, true, false, cfg!(windows)).unwrap();
        let entries = vec![(source.clone(), destination, entity)];
        assert!(crate::move_engine::remove_copied_sources(&entries, &source, false, true).is_err());
        assert_eq!(fs::read(source.join("new-data")).unwrap(), b"not copied");
    }

    #[crate::ctb_test]
    fn test_mv_cleanup_retains_changed_source() {
        let temp = tempdir().unwrap();
        let source = temp.path().join("source");
        let destination = temp.path().join("destination");
        fs::write(&source, b"original").unwrap();
        let entity = ctb_io::file::FileEntity::from_filesystem(&source, None).unwrap();
        fs::copy(&source, &destination).unwrap();
        fs::write(&source, b"new data").unwrap();
        let entries = vec![(source.clone(), destination, entity)];
        assert!(crate::move_engine::remove_copied_sources(&entries, &source, false, true).is_err());
        assert_eq!(fs::read(source).unwrap(), b"new data");
    }

    #[crate::ctb_test]
    fn test_mv_contents_fallback() {
        let _fs_lock = ctb_io::file::FS_CACHE_TEST_MUTEX.lock().unwrap();
        let temp = tempdir().unwrap();
        let source = temp.path().join("source");
        let destination = temp.path().join("destination");
        fs::create_dir_all(source.join("sub")).unwrap();
        fs::write(source.join("sub/data"), b"move payload").unwrap();
        let result = run_mv(MvArgs {
            paths: vec![source.join(""), destination.clone()],
            verbose: false, progress: false, no_progress: true,
            verify_after: false, no_verify_after: true,
            best_effort_metadata: true, allow_unknown_fs: false, force: false, dry_run: false,
            full_provenance: false,
        });
        let journal_path = match result {
            Ok(ToolResult::Immediate { stdout, .. }) => {
                assert_eq!(fs::read(destination.join("sub/data")).unwrap(), b"move payload");
                assert_eq!(fs::read_dir(&source).unwrap().count(), 0);
                let summary = String::from_utf8(stdout).unwrap();
                let journal_path = summary.lines().find_map(|line|
                    line.strip_prefix("Original metadata journal:").map(str::trim)
                ).expect("Move must retain original native metadata");
                let journal_path = PathBuf::from(journal_path);
                let snapshot = crate::journal::read_journal_snapshot(&journal_path).unwrap();
                assert!(snapshot.is_completed);
                assert!(snapshot.errors.is_empty());
                if !snapshot.warnings.is_empty() {
                    assert!(summary.contains("fidelity limitations are retained in the journal"));
                }
                journal_path
            }
            Ok(_) => panic!("Expected immediate move result"),
            Err(error) => {
                // FreeBSD cannot assign the source birth time. Strict cleanup
                // must refuse data deletion even after best-effort copying.
                let diagnostic = format!("{error:#}");
                assert!(diagnostic.contains("Birthtime mismatch"), "{diagnostic}");
                assert_eq!(fs::read(source.join("sub/data")).unwrap(), b"move payload");
                assert_eq!(fs::read(destination.join("sub/data")).unwrap(), b"move payload");
                let message = error.to_string();
                let journal_path = PathBuf::from(
                    message.strip_prefix("Move journal retained at ").expect("Retained move journal")
                );
                let snapshot = crate::journal::read_journal_snapshot(&journal_path).unwrap();
                assert!(!snapshot.warnings.is_empty());
                assert!(snapshot.committed_entities.values().any(|entity|
                    entity.metadata.timestamps.as_ref().and_then(|ts| ts.birthtime_sec).is_some()
                ));
                journal_path
            }
        };
        let snapshot = crate::journal::read_journal_snapshot(&journal_path).unwrap();
        assert!(!snapshot.committed_entities.is_empty());
        let journal_path = journal_path.as_path();
        fs::remove_file(journal_path).unwrap();
        fs::remove_file(journal_path.with_extension("cscdesc")).unwrap();
        fs::remove_dir(journal_path.parent().unwrap()).unwrap();
    }

    #[crate::ctb_test]
    fn test_mv_failure_retains_journal_and_source() {
        let _fs_lock = ctb_io::file::FS_CACHE_TEST_MUTEX.lock().unwrap();
        let temp = tempdir().unwrap();
        let source = temp.path().join("source");
        let destination = temp.path().join("destination");
        fs::create_dir_all(source.join("sub")).unwrap();
        fs::write(source.join("sub/data"), b"must survive failed move").unwrap();
        fs::create_dir_all(destination.join("sub/data")).unwrap();
        let error = run_mv(MvArgs {
            paths: vec![source.join(""), destination.clone()],
            verbose: false, progress: false, no_progress: true,
            verify_after: false, no_verify_after: true,
            best_effort_metadata: true, allow_unknown_fs: false, force: true, dry_run: false,
            full_provenance: false,
        }).err().expect("Conflicting destination must refuse the move");
        let message = error.to_string();
        let journal_path = Path::new(
            message.strip_prefix("Move journal retained at ").expect("Retained journal diagnostic")
        );
        let snapshot = crate::journal::read_journal_snapshot(journal_path).unwrap();
        assert!(!snapshot.is_completed);
        assert!(!snapshot.errors.is_empty() || !snapshot.warnings.is_empty());
        assert_eq!(fs::read(source.join("sub/data")).unwrap(), b"must survive failed move");
        fs::remove_file(journal_path).unwrap();
        fs::remove_file(journal_path.with_extension("cscdesc")).unwrap();
        fs::remove_dir(journal_path.parent().unwrap()).unwrap();
    }

    #[crate::ctb_test]
    fn test_journal_retention_includes_semantic_diagnostics() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("file");
        fs::write(&path, b"payload").unwrap();
        let mut entity = FileEntity::from_filesystem(&path, None).unwrap();
        entity.metadata = ctb_io::file::FileMetadata::default();
        assert!(!crate::journal::entity_requires_metadata_retention(&entity));
        entity.metadata.push_warning("Captured fidelity limitation".to_string());
        assert!(crate::journal::entity_requires_metadata_retention(&entity));
        entity.metadata.warnings.clear();
        entity.metadata.push_error("Captured metadata error".to_string());
        assert!(crate::journal::entity_requires_metadata_retention(&entity));
    }
}
