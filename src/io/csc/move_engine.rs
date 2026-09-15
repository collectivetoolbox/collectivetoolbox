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
use ctb_io::file::{FileEntity, FileOrigin, is_cross_device_error, verify_materialized_entity};
use ctb_utilities::cli::ToolResult;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Executes the `mv` command.
///
/// Attempts an atomic O(1) filesystem rename. When encountering `EXDEV`
/// (cross-filesystem link), falls back to executing the verified checksummed
/// copy pipeline (`csc --delete-manifest-after`) and unlinks source entities
/// only after verification completes successfully.
pub fn run_mv(args: MvArgs) -> Result<ToolResult> {
    let _env_scope =
        ctb_utilities::environment::GlobalEnvironmentScope::enter_fresh();
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
            std::fs::rename(&task.source_root, &task.target_root)
        } else {
            Err(std::io::Error::from(std::io::ErrorKind::CrossesDevices))
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
            Err(err) if is_cross_device_error(&err) => {
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
                    always_overwrite: args.force,
                    on_source_change: SourceChangePolicy::Error,
                    copy_specials_as_specials: true,
                    copy_block_devices_as_regular_files: false,
                    one_file_system: false,
                    best_effort_metadata: args.best_effort_metadata,
                    allow_unknown_fs: args.allow_unknown_fs,
                    check_atime: false,
                    check_ctime: false,
                    strict: false,
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
                    &progress,
                )?;

                anyhow::ensure!(copy_stats.special_files_skipped == 0,
                    "Move refused: some source entries could not be copied; all sources retained");
                remove_copied_sources(&copy_stats.copied_entities, &task.source_root, task.copy_contents_only)?;

                // Delete manifest files upon successful copy
                let j_path = journal.journal_path().to_path_buf();
                let d_path = journal.desc_path().to_path_buf();
                drop(journal);
                let retain_metadata = copy_stats.copied_entities.iter().any(|(_, _, entity)|
                    entity.metadata.native.is_some() || entity.metadata.timestamps.birthtime_sec.is_some()
                    || entity.metadata.platform_raw_flags.is_some() || !entity.streams.is_empty());
                if retain_metadata {
                    progress.message(&format!("Original metadata retained in {}", j_path.display()));
                }
                if !retain_metadata && j_path.exists() {
                    let _ = std::fs::remove_file(&j_path);
                }
                if !retain_metadata && d_path.exists() {
                    let _ = std::fs::remove_file(&d_path);
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

pub(crate) fn remove_copied_sources(
    entries: &[(PathBuf, PathBuf, FileEntity)],
    source_root: &Path,
    contents_only: bool,
) -> Result<()> {
    for (source, destination, entity) in entries {
        verify_source_identity(source, entity)?;
        verify_materialized_entity(source, entity, true)?;
        verify_materialized_entity(destination, entity, true)?;
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
            verify_materialized_entity(source, entity, true)?;
            verify_materialized_entity(destination, entity, true)?;
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
