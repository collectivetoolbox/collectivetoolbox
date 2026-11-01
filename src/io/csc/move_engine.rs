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
use crate::copy_engine::{
    compute_journal_relative_path, execute_copy_pipeline,
};
use crate::journal::{
    MoveSourceDisposition, MoveSourceRecord, StateJournal,
    read_journal_snapshot, resolve_journal_path,
};
use crate::path_resolution::{ResolvedCopyTask, resolve_tasks};
use ctb_io::file::{
    FileEntity, FileEntityKind, FileOrigin, TraversalOptions,
    is_cross_device_error, traverse_dir, verify_materialized_entity,
};
use ctb_utilities::cli::ToolResult;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::Instant;

fn rename_with_policy(
    source: &Path,
    destination: &Path,
    policy: ctb_io::file::ExistingEntryPolicy,
) -> Result<()> {
    let source_parent =
        match source.parent().filter(|path| !path.as_os_str().is_empty()) {
            Some(parent) => parent,
            None => Path::new("."),
        };
    let destination_parent = match destination
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
    {
        Some(parent) => parent,
        None => Path::new("."),
    };
    let source_name =
        source.file_name().context("Missing source entry name")?;
    let destination_name = destination
        .file_name()
        .context("Missing destination entry name")?;
    let source_dir = ctb_io::file::SandboxableDir::open(source_parent)?;
    let destination_dir =
        ctb_io::file::SandboxableDir::open(destination_parent)?;
    destination_dir.rename_from_parent(
        &source_dir.root_fd(),
        source_name,
        &destination_dir.root_fd(),
        destination_name,
        policy,
    )
}

fn normalize_move_source_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        normalized.push(component.as_os_str());
    }
    if normalized.as_os_str().is_empty() {
        path.to_path_buf()
    } else {
        normalized
    }
}

fn is_within_task_source(candidate: &Path, task_source_root: &Path) -> bool {
    let norm_candidate = normalize_move_source_path(candidate);
    let norm_root = normalize_move_source_path(task_source_root);
    norm_candidate == norm_root || norm_candidate.starts_with(&norm_root)
}

fn open_or_create_move_journal(
    args: &MvArgs,
    temp_state_dir: &mut Option<tempfile::TempDir>,
    sources: &[PathBuf],
    dest_path: &Path,
    initial_task_records: &[MoveSourceRecord],
) -> Result<StateJournal> {
    let mut journal = if let Some(ref custom_jp) = args.journal_path {
        let journal_path = if custom_jp.extension().and_then(|e| e.to_str())
            == Some("cscdesc")
        {
            custom_jp.with_extension("cscjournal")
        } else if custom_jp.extension().and_then(|e| e.to_str())
            == Some("cscjournal")
        {
            custom_jp.clone()
        } else {
            custom_jp.with_extension("cscjournal")
        };
        let desc_path = journal_path.with_extension("cscdesc");
        anyhow::ensure!(
            !journal_path.exists() && !desc_path.exists(),
            "Journal file already exists at {}: will not overwrite existing file (pass --resume to resume an interrupted move)",
            journal_path.display()
        );
        StateJournal::create_at_path(
            &journal_path,
            &desc_path,
            sources,
            dest_path,
        )?
    } else if let Some(ref custom_dir) = args.state_dir {
        StateJournal::create_new(custom_dir, sources, dest_path)?
    } else {
        let mut dir = tempfile::tempdir().context(
            "Failed to create temporary directory for move state journal",
        )?;
        let jw = StateJournal::create_new(dir.path(), sources, dest_path)?;
        dir.disable_cleanup(true);
        *temp_state_dir = Some(dir);
        jw
    };
    journal.set_defer_completion(true);
    if !initial_task_records.is_empty() {
        journal.record_move_sources_durable(
            initial_task_records.iter().cloned(),
        )?;
    }
    Ok(journal)
}

fn discover_task_source_inventory(
    task: &ResolvedCopyTask,
    csc_args: &CscArgs,
    journal: &mut StateJournal,
) -> Result<()> {
    let norm_root = normalize_move_source_path(&task.source_root);
    let src_meta =
        std::fs::symlink_metadata(&task.source_root).with_context(|| {
            format!(
                "Source path does not exist before cross-device move: {}",
                task.source_root.display()
            )
        })?;
    let mut discovered = Vec::new();
    if src_meta.is_dir() {
        let traversal_options = TraversalOptions::new()
            .yield_root(true)
            .one_file_system(csc_args.one_file_system)
            .apple_read_options(csc_args.resolve_apple_read_options());
        for entry in traverse_dir(&task.source_root, traversal_options)? {
            let entry = entry?;
            anyhow::ensure!(
                entry.errors.is_empty(),
                "Source traversal errors: {}",
                entry.errors.join("; ")
            );
            let is_root = entry.relative_path.as_os_str().is_empty();
            let norm_src = if is_root {
                norm_root.clone()
            } else {
                normalize_move_source_path(&entry.path)
            };
            let dest_path = if is_root {
                task.target_root.clone()
            } else {
                task.target_root.join(&entry.relative_path)
            };
            let source_identity =
                ctb_io::file::identity::query_file_identity(&norm_src).ok();
            let committed_relative_path = compute_journal_relative_path(
                journal.destination(),
                &dest_path,
            )
            .ok();
            if let Some(existing) = journal.move_sources().get(&norm_src) {
                if let (Some(expected_id), Some(actual_id)) =
                    (&existing.source_identity, &source_identity)
                {
                    anyhow::ensure!(
                        expected_id == actual_id,
                        "Source entry was replaced before move completed: {}",
                        norm_src.display()
                    );
                }
                let mut updated = existing.clone();
                if is_root {
                    updated.is_task_root = true;
                    updated.contents_only_root = task.copy_contents_only;
                }
                if updated.source_identity.is_none() {
                    updated.source_identity = source_identity;
                }
                if updated.committed_relative_path.is_none() {
                    updated.committed_relative_path = committed_relative_path;
                }
                discovered.push(updated);
            } else {
                discovered.push(MoveSourceRecord {
                    source_path: norm_src,
                    destination_path: dest_path,
                    committed_relative_path,
                    source_identity,
                    is_dir: entry.is_dir,
                    is_task_root: is_root,
                    contents_only_root: is_root && task.copy_contents_only,
                    disposition: MoveSourceDisposition::Pending,
                });
            }
        }
    } else {
        let source_identity =
            ctb_io::file::identity::query_file_identity(&norm_root).ok();
        let committed_relative_path = compute_journal_relative_path(
            journal.destination(),
            &task.target_root,
        )
        .ok();
        if let Some(existing) = journal.move_sources().get(&norm_root) {
            if let (Some(expected_id), Some(actual_id)) =
                (&existing.source_identity, &source_identity)
            {
                anyhow::ensure!(
                    expected_id == actual_id,
                    "Source entry was replaced before move completed: {}",
                    norm_root.display()
                );
            }
            let mut updated = existing.clone();
            updated.is_task_root = true;
            if updated.source_identity.is_none() {
                updated.source_identity = source_identity;
            }
            if updated.committed_relative_path.is_none() {
                updated.committed_relative_path = committed_relative_path;
            }
            discovered.push(updated);
        } else {
            discovered.push(MoveSourceRecord {
                source_path: norm_root,
                destination_path: task.target_root.clone(),
                committed_relative_path,
                source_identity,
                is_dir: false,
                is_task_root: true,
                contents_only_root: false,
                disposition: MoveSourceDisposition::Pending,
            });
        }
    }
    journal.record_move_sources_durable(discovered)
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
    let progress = Progress::new(args.should_show_progress());
    let strict_lossless = !args.best_effort_metadata && !args.allow_unknown_fs;

    let mut temp_state_dir: Option<tempfile::TempDir> = None;
    let (tasks, sources, dest_path, mut initial_task_records, mut journal) =
        if let Some(ref resume_arg) = args.resume {
            let journal_path = resolve_journal_path(resume_arg)?;
            let desc_path = journal_path.with_extension("cscdesc");
            let snap = read_journal_snapshot(&journal_path)?;
            let mut root_records: Vec<MoveSourceRecord> = snap
                .move_sources
                .values()
                .filter(|r| r.is_task_root)
                .cloned()
                .collect();
            root_records.sort_by(|a, b| a.source_path.cmp(&b.source_path));
            let resolved_tasks = if root_records.is_empty() {
                let mut all_paths = snap.sources.clone();
                all_paths.push(snap.destination.clone());
                resolve_tasks(&all_paths)?.0
            } else {
                root_records
                    .iter()
                    .map(|r| ResolvedCopyTask {
                        source_root: r.source_path.clone(),
                        target_root: r.destination_path.clone(),
                        copy_contents_only: r.contents_only_root,
                    })
                    .collect::<Vec<_>>()
            };
            let mut jw = StateJournal::open_for_resume(
                &journal_path,
                &desc_path,
                &snap,
            )?;
            jw.set_defer_completion(true);
            (
                resolved_tasks,
                snap.sources.clone(),
                snap.destination.clone(),
                root_records,
                Some(jw),
            )
        } else {
            anyhow::ensure!(
                args.paths.len() >= 2,
                "Must provide at least one source and one destination path"
            );
            let (resolved_tasks, resolved_dest) = resolve_tasks(&args.paths)?;
            if !args.best_effort_metadata && !args.allow_unknown_fs {
                for task in &resolved_tasks {
                    if let Ok(src_meta) =
                        std::fs::symlink_metadata(&task.source_root)
                    {
                        let src_fs = ctb_io::file::query_filesystem_info(
                            &task.source_root,
                            &src_meta,
                        );
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
                        match ctb_io::file::resolve_existing_ancestors(
                            &task.target_root,
                        ) {
                            Ok(ancestor) => ancestor,
                            Err(_) => PathBuf::from("."),
                        }
                    };
                    if let Ok(tgt_meta) =
                        std::fs::symlink_metadata(&tgt_existing)
                    {
                        let tgt_fs = ctb_io::file::query_filesystem_info(
                            &tgt_existing,
                            &tgt_meta,
                        );
                        if tgt_fs.fs_type == "unknown" {
                            anyhow::bail!(
                                "Cannot detect filesystem type for destination '{}'. Pass --best-effort-metadata or --allow-unknown-fs to proceed.",
                                task.target_root.display()
                            );
                        }
                    }
                }
            }
            let task_sources: Vec<PathBuf> = resolved_tasks
                .iter()
                .map(|t| t.source_root.clone())
                .collect();
            let mut root_records = Vec::with_capacity(resolved_tasks.len());
            for task in &resolved_tasks {
                let norm_src = normalize_move_source_path(&task.source_root);
                let meta = std::fs::symlink_metadata(&task.source_root)?;
                let source_identity =
                    ctb_io::file::identity::query_file_identity(&norm_src).ok();
                let committed_rel = compute_journal_relative_path(
                    &resolved_dest,
                    &task.target_root,
                )
                .ok();
                root_records.push(MoveSourceRecord {
                    source_path: norm_src,
                    destination_path: task.target_root.clone(),
                    committed_relative_path: committed_rel,
                    source_identity,
                    is_dir: meta.is_dir(),
                    is_task_root: true,
                    contents_only_root: task.copy_contents_only,
                    disposition: MoveSourceDisposition::Pending,
                });
            }
            let jw = if !args.dry_run
                && (args.journal_path.is_some() || args.state_dir.is_some())
            {
                Some(open_or_create_move_journal(
                    &args,
                    &mut temp_state_dir,
                    &task_sources,
                    &resolved_dest,
                    &root_records,
                )?)
            } else {
                None
            };
            (
                resolved_tasks,
                task_sources,
                resolved_dest,
                root_records,
                jw,
            )
        };

    let mut moved_renamed: u64 = 0;
    let mut moved_copied: u64 = 0;
    let mut bytes_copied: u64 = 0;
    let mut retained_journals = Vec::new();
    let mut warning_count = 0_usize;

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

        let norm_task_src = normalize_move_source_path(&task.source_root);
        // Reason for fallback: move execution without an active state journal has no recorded source move records
        let existing_task_records: Vec<MoveSourceRecord> = journal
            .as_ref()
            .map(|jw| {
                jw.move_sources()
                    .values()
                    .filter(|r| {
                        is_within_task_source(&r.source_path, &norm_task_src)
                    })
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();

        if existing_task_records.iter().any(|r| {
            r.is_task_root && r.disposition == MoveSourceDisposition::Renamed
        }) {
            let root_rec = existing_task_records
                .iter()
                .find(|r| r.is_task_root)
                .context("Missing renamed task root record")?;
            std::fs::symlink_metadata(&root_rec.destination_path).with_context(|| {
                format!(
                    "Resumed move failed: previously renamed destination missing: {}",
                    root_rec.destination_path.display()
                )
            })?;
            if let Some(ref expected_id) = root_rec.source_identity {
                if let Ok(actual_id) =
                    ctb_io::file::identity::query_file_identity(
                        &root_rec.destination_path,
                    )
                {
                    anyhow::ensure!(
                        actual_id == *expected_id,
                        "Resumed move failed: previously renamed destination identity changed: {}",
                        root_rec.destination_path.display()
                    );
                }
            }
            moved_renamed = moved_renamed.saturating_add(1);
            continue;
        }

        let task_in_deletion_or_done = !existing_task_records.is_empty()
            && existing_task_records.iter().all(|r| {
                matches!(
                    r.disposition,
                    MoveSourceDisposition::CopiedVerified
                        | MoveSourceDisposition::Deleted
                        | MoveSourceDisposition::RetainedRoot
                )
            });

        if task_in_deletion_or_done {
            let jw = journal.as_mut().context(
                "Journal must be open when resuming move source deletion",
            )?;
            let j_path = jw.journal_path().to_path_buf();
            if let Err(error) = execute_journaled_source_deletion(
                jw,
                &norm_task_src,
                strict_lossless,
            ) {
                if let Err(record_error) = jw.mark_failed(&format!("{error:#}"))
                {
                    return Err(error).context(format!(
                        "Move journal retained at {}; recording failure also failed: {record_error:#}",
                        j_path.display()
                    ));
                }
                return Err(error).context(format!(
                    "Move journal retained at {}",
                    j_path.display()
                ));
            }
            for rec in &existing_task_records {
                if let Some(entity) = jw.committed_entity_for_move_source(rec) {
                    if let FileEntityKind::Regular { size, .. } = entity.kind {
                        moved_copied = moved_copied.saturating_add(1);
                        bytes_copied = bytes_copied.saturating_add(size);
                    }
                }
            }
            continue;
        }

        let has_in_progress_copy = existing_task_records.len() > 1
            || journal.as_ref().is_some_and(|jw| {
                !jw.committed_entities().is_empty()
                    && existing_task_records.iter().any(|r| {
                        r.committed_relative_path.as_ref().is_some_and(|rel| {
                            jw.committed_entities().contains_key(
                                rel.as_os_str().as_encoded_bytes(),
                            )
                        })
                    })
            });

        // Fast path: atomic rename on the same filesystem (unless cross-device copy already started)
        let rename_res = if !task.copy_contents_only && !has_in_progress_copy {
            if let Some(parent) = task.target_root.parent() {
                if !parent.as_os_str().is_empty() && !parent.exists() {
                    std::fs::create_dir_all(parent).with_context(|| {
                        format!(
                            "Failed to create destination directory: {}",
                            parent.display()
                        )
                    })?;
                }
            }
            rename_with_policy(
                &task.source_root,
                &task.target_root,
                if args.force {
                    ctb_io::file::ExistingEntryPolicy::Replace
                } else {
                    ctb_io::file::ExistingEntryPolicy::Refuse
                },
            )
        } else {
            Err(std::io::Error::from(std::io::ErrorKind::CrossesDevices).into())
        };

        match rename_res {
            Ok(()) => {
                moved_renamed = moved_renamed.saturating_add(1);
                if let Some(rec) = initial_task_records
                    .iter_mut()
                    .find(|r| r.source_path == norm_task_src)
                {
                    rec.disposition = MoveSourceDisposition::Renamed;
                }
                if let Some(ref mut jw) = journal {
                    jw.update_move_source_disposition_durable(
                        &norm_task_src,
                        MoveSourceDisposition::Renamed,
                    )?;
                }
                if args.verbose {
                    progress.message(&format!(
                        "Renamed {} -> {}",
                        task.source_root.display(),
                        task.target_root.display()
                    ));
                }
            }
            Err(err)
                if err
                    .downcast_ref::<std::io::Error>()
                    .is_some_and(is_cross_device_error) =>
            {
                progress.message(&format!(
                    "Cross-device move detected for {}: copying with verification...",
                    task.source_root.display()
                ));

                let csc_args = CscArgs {
                    paths: vec![
                        task.source_root.clone(),
                        task.target_root.clone(),
                    ],
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
                    read_windows_ads: false,
                    no_read_windows_ads: false,
                    maybe_write_windows_ads: false,
                    force_write_windows_ads: false,
                    full_provenance: args.full_provenance,
                    continue_on_error: false,
                    allow_journal_errors: false,
                };

                if journal.is_none() {
                    journal = Some(open_or_create_move_journal(
                        &args,
                        &mut temp_state_dir,
                        &sources,
                        &dest_path,
                        &initial_task_records,
                    )?);
                }
                let jw = journal
                    .as_mut()
                    .context("Move journal must be initialized")?;
                let j_path = jw.journal_path().to_path_buf();

                let copy_result = (|| {
                    discover_task_source_inventory(task, &csc_args, jw)?;
                    let copy_stats = execute_copy_pipeline(
                        std::slice::from_ref(task),
                        &csc_args,
                        jw,
                        &progress,
                    )?;

                    anyhow::ensure!(
                        copy_stats.special_files_skipped == 0,
                        "Move refused: some source entries could not be copied; all sources retained"
                    );
                    anyhow::ensure!(
                        copy_stats.files_failed == 0 && jw.errors().is_empty(),
                        "Move refused: copy recorded failures; all sources retained"
                    );
                    remove_copied_sources_with_journal(
                        &copy_stats.copied_entities,
                        &task.source_root,
                        task.copy_contents_only,
                        strict_lossless,
                        jw,
                    )?;
                    Ok(copy_stats)
                })();
                let copy_stats = match copy_result {
                    Ok(stats) => stats,
                    Err(error) => {
                        if let Err(record_error) =
                            jw.mark_failed(&format!("{error:#}"))
                        {
                            return Err(error).context(format!(
                                "Move journal retained at {}; recording failure also failed: {record_error:#}",
                                j_path.display()
                            ));
                        }
                        return Err(error).context(format!(
                            "Move journal retained at {}",
                            j_path.display()
                        ));
                    }
                };

                moved_copied = moved_copied
                    .saturating_add(copy_stats.files_copied)
                    .saturating_add(copy_stats.files_skipped_identical);
                bytes_copied =
                    bytes_copied.saturating_add(copy_stats.bytes_copied);
            }
            Err(err) => {
                if journal.is_none() && moved_renamed > 0 {
                    if let Ok(mut jw) = open_or_create_move_journal(
                        &args,
                        &mut temp_state_dir,
                        &sources,
                        &dest_path,
                        &initial_task_records,
                    ) {
                        let _ = jw.mark_failed(&format!("{err:#}"));
                        journal = Some(jw);
                    }
                } else if let Some(ref mut jw) = journal {
                    let _ = jw.mark_failed(&format!("{err:#}"));
                }
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

    if let Some(mut jw) = journal {
        jw.mark_completed()?;
        let j_path = jw.journal_path().to_path_buf();
        warning_count = warning_count.saturating_add(jw.warnings().len());
        for warning in jw.warnings() {
            progress
                .message(&format!("[{}] {}", warning.code, warning.message));
        }
        let retain_metadata = !jw.warnings().is_empty()
            || jw
                .committed_entities()
                .values()
                .any(crate::journal::entity_requires_metadata_retention);
        let explicit_journal_location = args.journal_path.is_some()
            || args.state_dir.is_some()
            || args.resume.is_some();
        drop(jw);
        if retain_metadata {
            progress.message(&format!(
                "Original metadata retained in {}",
                j_path.display()
            ));
            retained_journals.push(j_path);
        } else if !explicit_journal_location {
            if let Some(dir) = temp_state_dir.take() {
                dir.close().with_context(|| {
                    format!(
                        "Failed to remove completed move journal {}",
                        j_path.display()
                    )
                })?;
            }
        }
    }

    let elapsed = start_time.elapsed().as_secs_f64();
    let mut summary = String::new();
    writeln!(summary, "--- MV Summary ---")?;
    writeln!(summary, "Items renamed (same filesystem): {moved_renamed}")?;
    if moved_copied > 0 {
        writeln!(summary, "Files copied (cross-device):     {moved_copied}")?;
        writeln!(summary, "Bytes transferred:               {bytes_copied}")?;
    }
    writeln!(summary, "Duration:                        {elapsed:.2}s")?;
    for journal in retained_journals {
        writeln!(
            summary,
            "Original metadata journal:       {}",
            journal.display()
        )?;
    }
    if warning_count > 0 {
        writeln!(
            summary,
            "Status:                          Move completed with {warning_count} warning record(s); fidelity limitations are retained in the journal."
        )?;
    } else {
        writeln!(
            summary,
            "Status:                          Move completed successfully."
        )?;
    }

    Ok(ToolResult::immediate_ok(summary.into_bytes()))
}

pub(crate) fn remove_copied_sources(
    entries: &[(PathBuf, PathBuf, FileEntity)],
    source_root: &Path,
    contents_only: bool,
    strict_lossless: bool,
) -> Result<()> {
    let temp_dir = tempfile::tempdir().context(
        "Failed to create temporary directory for move source deletion journal",
    )?;
    // Reason for fallback: when entries slice is empty, destination root for temporary move journal defaults to current directory
    let dest_root = entries
        .iter()
        .find(|(src, _, _)| {
            normalize_move_source_path(src)
                == normalize_move_source_path(source_root)
        })
        .map(|(_, dst, _)| dst.clone())
        .or_else(|| entries.first().map(|(_, dst, _)| dst.clone()))
        .unwrap_or_else(|| PathBuf::from("."));
    let mut journal = StateJournal::create_new(
        temp_dir.path(),
        &[source_root.to_path_buf()],
        &dest_root,
    )?;
    journal.set_defer_completion(true);
    remove_copied_sources_with_journal(
        entries,
        source_root,
        contents_only,
        strict_lossless,
        &mut journal,
    )
}

pub(crate) fn remove_copied_sources_with_journal(
    entries: &[(PathBuf, PathBuf, FileEntity)],
    source_root: &Path,
    contents_only: bool,
    strict_lossless: bool,
    journal: &mut StateJournal,
) -> Result<()> {
    record_verified_copied_sources_in_journal(
        entries,
        source_root,
        contents_only,
        journal,
    )?;
    execute_journaled_source_deletion(journal, source_root, strict_lossless)
}

fn record_verified_copied_sources_in_journal(
    entries: &[(PathBuf, PathBuf, FileEntity)],
    source_root: &Path,
    contents_only: bool,
    journal: &mut StateJournal,
) -> Result<()> {
    let norm_root = normalize_move_source_path(source_root);
    let mut records = Vec::with_capacity(entries.len());
    for (source, destination, entity) in entries {
        let norm_src = normalize_move_source_path(source);
        let rel =
            compute_journal_relative_path(journal.destination(), destination)?;
        if !journal
            .committed_entities()
            .contains_key(rel.as_os_str().as_encoded_bytes())
        {
            let mut journal_entity = entity.clone();
            journal_entity.identity.relative_path = Some(rel.clone());
            journal.record_entity(&journal_entity);
        }
        let source_identity = match &entity.identity.origin {
            FileOrigin::Filesystem { key, .. } => Some(key.clone()),
            _ => None,
        };
        let is_root = norm_src == norm_root;
        let existing_is_task_root = journal
            .move_sources()
            .get(&norm_src)
            .is_some_and(|r| r.is_task_root);
        records.push(MoveSourceRecord {
            source_path: norm_src,
            destination_path: destination.clone(),
            committed_relative_path: Some(rel),
            source_identity,
            is_dir: entity.is_dir(),
            is_task_root: is_root || existing_is_task_root,
            contents_only_root: is_root && contents_only,
            disposition: MoveSourceDisposition::CopiedVerified,
        });
    }
    journal.commit_batch()?;
    journal.record_move_sources_durable(records)?;

    for record in journal.move_sources().values() {
        if is_within_task_source(&record.source_path, &norm_root) {
            anyhow::ensure!(
                record.disposition != MoveSourceDisposition::Pending,
                "Move refused: source inventory entry was not copied and verified: {}",
                record.source_path.display()
            );
        }
    }
    Ok(())
}

fn has_deleted_descendant(
    dir_path: &Path,
    records: &[MoveSourceRecord],
) -> bool {
    records.iter().any(|r| {
        r.source_path != dir_path
            && r.source_path.starts_with(dir_path)
            && r.disposition == MoveSourceDisposition::Deleted
    })
}

pub(crate) fn execute_journaled_source_deletion(
    journal: &mut StateJournal,
    source_root: &Path,
    strict_lossless: bool,
) -> Result<()> {
    let norm_root = normalize_move_source_path(source_root);
    let mut task_records: Vec<MoveSourceRecord> = journal
        .move_sources()
        .values()
        .filter(|r| is_within_task_source(&r.source_path, &norm_root))
        .cloned()
        .collect();
    anyhow::ensure!(
        !task_records.is_empty(),
        "No durable move source inventory records found for {}",
        source_root.display()
    );

    for record in &task_records {
        anyhow::ensure!(
            record.disposition != MoveSourceDisposition::Pending,
            "Move refused: source entry was not copied and verified: {}",
            record.source_path.display()
        );
        if record.disposition == MoveSourceDisposition::Renamed {
            std::fs::symlink_metadata(&record.destination_path).with_context(
                || {
                    format!(
                        "Renamed destination missing: {}",
                        record.destination_path.display()
                    )
                },
            )?;
            continue;
        }
        let committed_entity = journal
            .committed_entity_for_move_source(record)
            .with_context(|| {
                format!(
                    "Missing committed journal entity for move source {}",
                    record.source_path.display()
                )
            })?
            .clone();
        verify_materialized_entity(
            &record.destination_path,
            &committed_entity,
            strict_lossless,
        )?;
        match record.disposition {
            MoveSourceDisposition::CopiedVerified => {
                verify_source_identity(&record.source_path, &committed_entity)?;
                if !record.is_dir
                    || !has_deleted_descendant(
                        &record.source_path,
                        &task_records,
                    )
                {
                    verify_materialized_entity(
                        &record.source_path,
                        &committed_entity,
                        strict_lossless,
                    )?;
                }
            }
            MoveSourceDisposition::Deleted => {
                match std::fs::symlink_metadata(&record.source_path) {
                    Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
                    Ok(_) => {
                        verify_source_identity(&record.source_path, &committed_entity).with_context(
                        || {
                            format!(
                                "Source path was recreated with a different identity after deletion: {}",
                                record.source_path.display()
                            )
                        },
                    )?;
                    }
                    Err(err) => return Err(err.into()),
                }
            }
            MoveSourceDisposition::RetainedRoot => {
                verify_source_identity(&record.source_path, &committed_entity)?;
            }
            MoveSourceDisposition::Pending | MoveSourceDisposition::Renamed => {
            }
        }
    }

    task_records.sort_by(|a, b| {
        std::cmp::Reverse(a.source_path.components().count())
            .cmp(&std::cmp::Reverse(b.source_path.components().count()))
            .then_with(|| a.source_path.cmp(&b.source_path))
    });

    for record in task_records {
        if matches!(
            record.disposition,
            MoveSourceDisposition::Renamed
                | MoveSourceDisposition::RetainedRoot
        ) {
            continue;
        }
        if record.disposition == MoveSourceDisposition::Deleted
            && std::fs::symlink_metadata(&record.source_path)
                .is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound)
        {
            continue;
        }
        let committed_entity = journal
            .committed_entity_for_move_source(&record)
            .with_context(|| {
                format!(
                    "Missing committed journal entity for move source {}",
                    record.source_path.display()
                )
            })?
            .clone();
        verify_source_identity(&record.source_path, &committed_entity)?;
        if record.is_dir {
            if record.contents_only_root {
                journal.update_move_source_disposition_durable(
                    &record.source_path,
                    MoveSourceDisposition::RetainedRoot,
                )?;
            } else {
                std::fs::remove_dir(&record.source_path).with_context(|| {
                    format!(
                        "Source directory changed or could not be removed: {}",
                        record.source_path.display()
                    )
                })?;
                journal.update_move_source_disposition_durable(
                    &record.source_path,
                    MoveSourceDisposition::Deleted,
                )?;
            }
        } else {
            verify_materialized_entity(
                &record.source_path,
                &committed_entity,
                strict_lossless,
            )?;
            verify_materialized_entity(
                &record.destination_path,
                &committed_entity,
                strict_lossless,
            )?;
            std::fs::remove_file(&record.source_path).with_context(|| {
                format!(
                    "Failed to remove verified source: {}",
                    record.source_path.display()
                )
            })?;
            journal.update_move_source_disposition_durable(
                &record.source_path,
                MoveSourceDisposition::Deleted,
            )?;
        }
    }
    Ok(())
}

fn verify_source_identity(source: &Path, expected: &FileEntity) -> Result<()> {
    let actual = FileEntity::from_filesystem_metadata_only(source, None)?;
    match (&actual.identity.origin, &expected.identity.origin) {
        (
            FileOrigin::Filesystem {
                key: actual_key, ..
            },
            FileOrigin::Filesystem {
                key: expected_key, ..
            },
        ) => {
            anyhow::ensure!(
                actual_key == expected_key,
                "Source entry was replaced: {}",
                source.display()
            );
        }
        _ => anyhow::bail!(
            "Source identity is unavailable: {}",
            source.display()
        ),
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
    use crate::args::MvArgs;
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
        let destination =
            temporary.path().join("destination-parent/destination");
        fs::write(&source, b"source payload")?;
        fs::write(&destination, b"destination payload")?;
        let args = MvArgs::try_parse_from([
            std::ffi::OsStr::new("mv"),
            std::ffi::OsStr::new("--best-effort-metadata"),
            source.as_os_str(),
            destination.as_os_str(),
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
        assert!(
            rename_with_policy(
                &source,
                &destination,
                ctb_io::file::ExistingEntryPolicy::Refuse
            )
            .is_err()
        );
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
        fs::write(&file_path, b"Move this file atomically")
            .expect("write file");

        let target_file = dest.join("move_me.txt");
        let args = MvArgs {
            paths: vec![file_path.clone(), target_file.clone()],
            resume: None,
            journal_path: None,
            state_dir: None,
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

        fs::write(src_dir.join("file1.txt"), b"Content 1")
            .expect("write file1");
        fs::write(src_dir.join("subdir").join("file2.txt"), b"Content 2")
            .expect("write file2");

        let args = MvArgs {
            paths: vec![src_dir.clone(), dest_dir.clone()],
            resume: None,
            journal_path: None,
            state_dir: None,
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
        let entity =
            ctb_io::file::FileEntity::from_filesystem(&source, None).unwrap();
        ctb_io::file::apply_entity_metadata(
            &destination,
            None,
            &entity.metadata,
            false,
            true,
            false,
            cfg!(windows),
        )
        .unwrap();
        let entries = vec![(source.clone(), destination, entity)];
        assert!(
            crate::move_engine::remove_copied_sources(
                &entries, &source, false, true
            )
            .is_err()
        );
        assert_eq!(fs::read(source.join("new-data")).unwrap(), b"not copied");
    }

    #[crate::ctb_test]
    fn test_mv_cleanup_retains_changed_source() {
        let temp = tempdir().unwrap();
        let source = temp.path().join("source");
        let destination = temp.path().join("destination");
        fs::write(&source, b"original").unwrap();
        let entity =
            ctb_io::file::FileEntity::from_filesystem(&source, None).unwrap();
        fs::copy(&source, &destination).unwrap();
        fs::write(&source, b"new data").unwrap();
        let entries = vec![(source.clone(), destination, entity)];
        assert!(
            crate::move_engine::remove_copied_sources(
                &entries, &source, false, true
            )
            .is_err()
        );
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
            resume: None,
            journal_path: None,
            state_dir: None,
            verbose: false,
            progress: false,
            no_progress: true,
            verify_after: false,
            no_verify_after: true,
            best_effort_metadata: false,
            allow_unknown_fs: true,
            force: false,
            dry_run: false,
            full_provenance: false,
        });
        let journal_path = match result {
            Ok(ToolResult::Immediate { stdout, .. }) => {
                assert_eq!(
                    fs::read(destination.join("sub/data")).unwrap(),
                    b"move payload"
                );
                assert_eq!(fs::read_dir(&source).unwrap().count(), 0);
                let summary = String::from_utf8(stdout).unwrap();
                let journal_path = summary
                    .lines()
                    .find_map(|line| {
                        line.strip_prefix("Original metadata journal:")
                            .map(str::trim)
                    })
                    .expect("Move must retain original native metadata");
                let journal_path = PathBuf::from(journal_path);
                let snapshot =
                    crate::journal::read_journal_snapshot(&journal_path)
                        .unwrap();
                assert!(snapshot.is_completed);
                assert!(snapshot.errors.is_empty());
                if !snapshot.warnings.is_empty() {
                    assert!(summary.contains(
                        "fidelity limitations are retained in the journal"
                    ));
                }
                journal_path
            }
            Ok(_) => panic!("Expected immediate move result"),
            Err(error) => {
                // FreeBSD cannot assign the source birth time. Strict cleanup
                // must refuse data deletion even after best-effort copying.
                let diagnostic = format!("{error:#}");
                assert!(
                    diagnostic.contains("Birthtime mismatch"),
                    "{diagnostic}"
                );
                assert_eq!(
                    fs::read(source.join("sub/data")).unwrap(),
                    b"move payload"
                );
                assert_eq!(
                    fs::read(destination.join("sub/data")).unwrap(),
                    b"move payload"
                );
                let message = error.to_string();
                let journal_path = PathBuf::from(
                    message
                        .strip_prefix("Move journal retained at ")
                        .expect("Retained move journal"),
                );
                let snapshot =
                    crate::journal::read_journal_snapshot(&journal_path)
                        .unwrap();
                assert!(!snapshot.warnings.is_empty());
                assert!(snapshot.committed_entities.values().any(|entity| {
                    entity
                        .metadata
                        .timestamps
                        .as_ref()
                        .and_then(|ts| ts.birthtime_sec)
                        .is_some()
                }));
                journal_path
            }
        };
        let snapshot =
            crate::journal::read_journal_snapshot(&journal_path).unwrap();
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
        fs::write(source.join("sub/data"), b"must survive failed move")
            .unwrap();
        fs::create_dir_all(destination.join("sub/data")).unwrap();
        let error = run_mv(MvArgs {
            paths: vec![source.join(""), destination.clone()],
            resume: None,
            journal_path: None,
            state_dir: None,
            verbose: false,
            progress: false,
            no_progress: true,
            verify_after: false,
            no_verify_after: true,
            best_effort_metadata: false,
            allow_unknown_fs: true,
            force: true,
            dry_run: false,
            full_provenance: false,
        })
        .err()
        .expect("Conflicting destination must refuse the move");
        let message = error.to_string();
        let journal_path = Path::new(
            message
                .strip_prefix("Move journal retained at ")
                .expect("Retained journal diagnostic"),
        );
        let snapshot =
            crate::journal::read_journal_snapshot(journal_path).unwrap();
        assert!(!snapshot.is_completed);
        assert!(!snapshot.errors.is_empty() || !snapshot.warnings.is_empty());
        assert_eq!(
            fs::read(source.join("sub/data")).unwrap(),
            b"must survive failed move"
        );
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
        entity
            .metadata
            .push_warning("Captured fidelity limitation".to_string());
        assert!(crate::journal::entity_requires_metadata_retention(&entity));
        entity.metadata.warnings.clear();
        entity
            .metadata
            .push_error("Captured metadata error".to_string());
        assert!(crate::journal::entity_requires_metadata_retention(&entity));
    }

    #[crate::ctb_test]
    fn test_mv_durable_source_inventory_and_interrupted_deletion_recovery()
    -> Result<()> {
        let _fs_lock = ctb_io::file::FS_CACHE_TEST_MUTEX.lock().unwrap();
        let temp = tempdir()?;
        let src_rename = temp.path().join("src_rename.txt");
        let src_dir = temp.path().join("src_cross");
        let src_later = temp.path().join("src_later.txt");
        let dest_root = temp.path().join("dest");
        let journal_path = temp.path().join("state/move_recovery.cscjournal");
        fs::create_dir_all(journal_path.parent().unwrap())?;
        fs::create_dir_all(src_dir.join("nested"))?;
        fs::create_dir_all(&dest_root)?;

        fs::write(&src_rename, b"renamed task payload")?;
        let file1 = src_dir.join("nested/alpha.txt");
        let file2 = src_dir.join("nested/beta.txt");
        fs::write(&file1, b"alpha payload")?;
        fs::write(&file2, b"beta payload")?;
        fs::write(&src_later, b"later task payload")?;

        // 1. Run a cross-device move of src_dir contents into dest_root/cross_out to populate
        //    a real journal, then simulate an interruption partway through source deletion.
        let cross_dest = dest_root.join("cross_out");
        let rename_dest = dest_root.join("src_rename.txt");
        let later_dest = dest_root.join("src_later.txt");

        // Perform the rename of task 1 (`src_rename -> rename_dest`)
        let rename_id =
            ctb_io::file::identity::query_file_identity(&src_rename).ok();
        fs::rename(&src_rename, &rename_dest)?;

        // Copy `src_dir` to `cross_dest` using execute_copy_pipeline and record verified sources
        // into the journal WITHOUT deleting all sources yet (simulating crash after deleting `alpha.txt`).
        let desc_path = journal_path.with_extension("cscdesc");
        let mut journal = StateJournal::create_at_path(
            &journal_path,
            &desc_path,
            &[src_rename.clone(), src_dir.clone(), src_later.clone()],
            &dest_root,
        )?;
        journal.set_defer_completion(true);

        let later_id =
            ctb_io::file::identity::query_file_identity(&src_later).ok();
        journal.record_move_sources_durable([
            MoveSourceRecord {
                source_path: normalize_move_source_path(&src_rename),
                destination_path: rename_dest.clone(),
                committed_relative_path: Some(PathBuf::from("src_rename.txt")),
                source_identity: rename_id,
                is_dir: false,
                is_task_root: true,
                contents_only_root: false,
                disposition: MoveSourceDisposition::Renamed,
            },
            MoveSourceRecord {
                source_path: normalize_move_source_path(&src_dir),
                destination_path: cross_dest.clone(),
                committed_relative_path: Some(PathBuf::from("cross_out")),
                source_identity: ctb_io::file::identity::query_file_identity(
                    &src_dir,
                )
                .ok(),
                is_dir: true,
                is_task_root: true,
                contents_only_root: false,
                disposition: MoveSourceDisposition::Pending,
            },
            MoveSourceRecord {
                source_path: normalize_move_source_path(&src_later),
                destination_path: later_dest.clone(),
                committed_relative_path: Some(PathBuf::from("src_later.txt")),
                source_identity: later_id,
                is_dir: false,
                is_task_root: true,
                contents_only_root: false,
                disposition: MoveSourceDisposition::Pending,
            },
        ])?;

        let cross_task = ResolvedCopyTask {
            source_root: src_dir.clone(),
            target_root: cross_dest.clone(),
            copy_contents_only: false,
        };
        let csc_args = crate::args::default_test_args(
            vec![src_dir.clone(), cross_dest.clone()],
            journal_path.parent().unwrap().to_path_buf(),
        );
        discover_task_source_inventory(&cross_task, &csc_args, &mut journal)?;
        let progress = Progress::new(false);
        let copy_stats = execute_copy_pipeline(
            std::slice::from_ref(&cross_task),
            &csc_args,
            &mut journal,
            &progress,
        )?;
        record_verified_copied_sources_in_journal(
            &copy_stats.copied_entities,
            &src_dir,
            false,
            &mut journal,
        )?;

        // Simulate partial deletion before crash: `alpha.txt` is unlinked and marked `Deleted`,
        // while `beta.txt`, `nested`, and `src_dir` remain `CopiedVerified` on disk, and
        // `src_later.txt` is still `Pending`.
        fs::remove_file(&file1)?;
        journal.update_move_source_disposition_durable(
            &normalize_move_source_path(&file1),
            MoveSourceDisposition::Deleted,
        )?;
        drop(journal);

        // Sub-test A: If a new unrelated file is created at the already-deleted `alpha.txt` path
        // before resume, recovery must detect the identity mismatch and refuse to delete it!
        fs::write(&file1, b"unrelated replacement file")?;
        let bad_snap = read_journal_snapshot(&journal_path)?;
        let mut bad_journal = StateJournal::open_for_resume(
            &journal_path,
            &desc_path,
            &bad_snap,
        )?;
        let err = execute_journaled_source_deletion(
            &mut bad_journal,
            &src_dir,
            false,
        )
        .expect_err("Recreated source with different identity must be refused");
        assert!(
            format!("{err:#}").contains(
                "Source path was recreated with a different identity"
            ),
            "Unexpected error: {err:#}"
        );
        assert_eq!(fs::read(&file1)?, b"unrelated replacement file");
        fs::remove_file(&file1)?;
        drop(bad_journal);

        // Sub-test B: Resume the interrupted multi-file move via `run_mv` with `--resume`.
        // No in-memory `copied_entities` exists; recovery uses the durable `MoveSources:` log.
        run_mv(MvArgs {
            paths: Vec::new(),
            resume: Some(journal_path.clone()),
            journal_path: None,
            state_dir: None,
            verbose: false,
            progress: false,
            no_progress: true,
            verify_after: true,
            no_verify_after: false,
            best_effort_metadata: true,
            allow_unknown_fs: true,
            force: false,
            dry_run: false,
            full_provenance: false,
        })?;

        // Verify all 3 tasks completed and all sources are gone while destinations are intact.
        assert!(!src_rename.exists());
        assert!(!src_dir.exists());
        assert!(!src_later.exists());
        assert_eq!(fs::read(&rename_dest)?, b"renamed task payload");
        assert_eq!(
            fs::read(cross_dest.join("nested/alpha.txt"))?,
            b"alpha payload"
        );
        assert_eq!(
            fs::read(cross_dest.join("nested/beta.txt"))?,
            b"beta payload"
        );
        assert_eq!(fs::read(&later_dest)?, b"later task payload");

        let final_snap = read_journal_snapshot(&journal_path)?;
        assert!(final_snap.is_completed);
        assert_eq!(
            final_snap
                .move_sources
                .get(&normalize_move_source_path(&src_rename))
                .map(|r| r.disposition),
            Some(MoveSourceDisposition::Renamed)
        );
        assert_eq!(
            final_snap
                .move_sources
                .get(&normalize_move_source_path(&file1))
                .map(|r| r.disposition),
            Some(MoveSourceDisposition::Deleted)
        );
        assert_eq!(
            final_snap
                .move_sources
                .get(&normalize_move_source_path(&file2))
                .map(|r| r.disposition),
            Some(MoveSourceDisposition::Deleted)
        );
        assert_eq!(
            final_snap
                .move_sources
                .get(&normalize_move_source_path(&src_dir))
                .map(|r| r.disposition),
            Some(MoveSourceDisposition::Deleted)
        );
        assert_eq!(
            final_snap
                .move_sources
                .get(&normalize_move_source_path(&src_later))
                .map(|r| r.disposition),
            Some(MoveSourceDisposition::Renamed)
        );
        Ok(())
    }
}
