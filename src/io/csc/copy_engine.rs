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

//! Core file copying engine with sparse support, atomic temp writes, and strict verification.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::args::{CscArgs, SourceChangePolicy};
use crate::journal::{JournalErrorRecord, JournalErrorStage, JournalWriter};
use crate::path_resolution::ResolvedCopyTask;
use ctb_io::file::entity::{FileEntity, FileEntityKind};
use ctb_io::file::identity::FileOrigin;
use ctb_io::file::materializer::{
    MaterializeOptions, apply_entity_metadata, materialize_entity,
};
use ctb_io::file::path_policy::{PathTraversalPolicy, SymlinkValidationPolicy};
use ctb_io::file::payload::DiskPayloadSource;
use ctb_io::file::{query_block_device_size, verify_directory_filenames_exact};
use ctb_io::file::sandboxable_dir::SandboxableDir;
use ctb_io::file::streams::write_streams;
use ctb_io::file::verifier::{try_drop_system_caches, verify_materialized_entity_ext};
use std::collections::HashMap;
#[cfg(unix)]
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Summary stats from a copy run.
#[derive(Debug, Clone, Default)]
pub struct CopyStats {
    pub files_copied: u64,
    pub files_failed: u64,
    pub files_skipped_identical: u64,
    pub bytes_copied: u64,
    pub dirs_created: u64,
    pub symlinks_created: u64,
    pub hardlinks_created: u64,
    pub special_files_created: u64,
    pub special_files_skipped: u64,
    pub files_verified: u64,
    pub(crate) copied_entities: Vec<(PathBuf, PathBuf, FileEntity)>,
}

/// Deferred symlink to materialize in Pass 2.
struct DeferredSymlink {
    src_path: PathBuf,
    dest_path: PathBuf,
    entity: FileEntity,
    dest_dir_root: PathBuf,
}

/// Deferred directory metadata fixup item for Pass 3.
struct DeferredDirFixup {
    src_path: PathBuf,
    dest_path: PathBuf,
    dir_entity: FileEntity,
    expected_filenames: Vec<Vec<u8>>,
}

/// Records an item failure and either swallows it under `--continue-on-error` or
/// marks the journal failed and aborts.
fn handle_item_error(
    src_path: &Path,
    stage: JournalErrorStage,
    err: anyhow::Error,
    args: &CscArgs,
    journal: &mut JournalWriter,
    stats: &mut CopyStats,
) -> Result<Option<PathBuf>> {
    let now_sec = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|d| i64::try_from(d.as_secs()).ok())
        .unwrap_or(0);
    let os_error = err
        .downcast_ref::<std::io::Error>()
        .and_then(std::io::Error::raw_os_error);
    let err_record = JournalErrorRecord {
        path: src_path.to_path_buf(),
        raw_path: src_path.as_os_str().as_encoded_bytes().to_vec(),
        stage,
        error_message: err.to_string(),
        os_error,
        timestamp_sec: now_sec,
    };
    let _ = journal.record_error(&err_record);
    if args.continue_on_error {
        stats.files_failed = stats.files_failed.saturating_add(1);
        Ok(None)
    } else {
        let _ = journal.mark_failed(&err.to_string());
        Err(err)
    }
}

/// Runs the complete copy pipeline across all tasks using descriptor-safe, multi-pass copying.
fn validate_filesystem_known(path: &Path, fs_type: Option<&str>, args: &CscArgs) -> Result<()> {
    if !args.best_effort_metadata && !args.allow_unknown_fs {
        if let Some(fs) = fs_type {
            if fs == "unknown" {
                anyhow::bail!(
                    "Cannot detect filesystem type for '{}'. Pass --best-effort-metadata or --allow-unknown-fs to proceed.",
                    path.display()
                );
            }
        } else {
            anyhow::bail!(
                "Cannot detect filesystem type for '{}'. Pass --best-effort-metadata or --allow-unknown-fs to proceed.",
                path.display()
            );
        }
    }
    Ok(())
}

pub fn execute_copy_pipeline(
    tasks: &[ResolvedCopyTask],
    args: &CscArgs,
    journal: &mut JournalWriter,
    progress: &Progress,
) -> Result<CopyStats> {
    crate::path_resolution::validate_task_overlap(tasks)?;
    let mut stats = CopyStats::default();
    let mut hardlink_map: HashMap<(u64, u64), PathBuf> = HashMap::new();
    if let Some(snap) = journal.snapshot() {
        for entity in snap.committed_entities.values() {
            if entity.identity.nlink > 1 && entity.is_regular() {
                if let FileOrigin::Filesystem { key, .. } = &entity.identity.origin {
                    let dest_path = journal.destination().join(&entity.identity.relative_path);
                    if dest_path.exists() {
                        hardlink_map.insert((key.device_id, key.inode), dest_path);
                    }
                }
            }
        }
    }
    let mut deferred_dirs: Vec<DeferredDirFixup> = Vec::new();
    let mut deferred_symlinks: Vec<DeferredSymlink> = Vec::new();
    let mut files_to_verify: Vec<(PathBuf, PathBuf, FileEntity)> = Vec::new();

    if !args.copy_specials_as_specials && !args.copy_block_devices_as_regular_files {
        progress.message(
            "Notice: Special files (FIFOs, device nodes, sockets) will be skipped by default. Pass --copy-specials-as-specials to preserve them.",
        );
        let _ = journal.record_warning(
            "SpecialFilesSkippedByDefault",
            "Special files (FIFOs, device nodes, sockets) are skipped by default. Pass --copy-specials-as-specials to preserve them.",
        );
    }
    if args.best_effort_metadata {
        let _ = journal.record_warning(
            "BestEffortMetadata",
            "Best-effort metadata enabled: non-fatal metadata write errors or timestamp precision differences may be tolerated.",
        );
    }

    let mut uncommitted_count: usize = 0;
    let copy_task = progress.start_task("Copying", None);

    let strict_lossless = !args.best_effort_metadata && !args.allow_unknown_fs;
    let (apple_write_mode, apple_single_write_extension) = args.resolve_apple_write_mode()?;
    let apple_read_options = args.resolve_apple_read_options();
    let options = MaterializeOptions {
        dry_run: args.dry_run,
        strict_lossless,
        symlink_policy: SymlinkValidationPolicy::PreserveVerbatim,
        path_policy: PathTraversalPolicy::StrictSandboxed,
        copy_specials: args.copy_specials_as_specials,
        force_overwrite: args.always_overwrite,
        apple_write_mode,
        apple_single_write_extension,
    };

    // =========================================================================
    // PASS 1: Directory traversal & copying regular files, special nodes, hardlinks
    // =========================================================================
    for task in tasks {
        let src_root = &task.source_root;
        let tgt_root = &task.target_root;

        let src_meta = match std::fs::symlink_metadata(src_root) {
            Ok(m) => m,
            Err(e) => {
                anyhow::bail!("Source path does not exist: {}: {e}", src_root.display());
            }
        };

        let src_fs = ctb_io::file::query_filesystem_info(src_root, &src_meta);
        validate_filesystem_known(src_root, Some(&src_fs.fs_type), args)?;

        let tgt_existing = if tgt_root.exists() {
            tgt_root.clone()
        } else {
            match ctb_io::file::resolve_existing_ancestors(tgt_root) {
                Ok(ancestor) => ancestor,
                Err(_) => PathBuf::from("."),
            }
        };
        if let Ok(tgt_meta) = std::fs::symlink_metadata(&tgt_existing) {
            let tgt_fs = ctb_io::file::query_filesystem_info(&tgt_existing, &tgt_meta);
            validate_filesystem_known(tgt_root, Some(&tgt_fs.fs_type), args)?;
        }

        if src_meta.is_dir() {
            let dest_dir = if args.dry_run {
                SandboxableDir::open(".").context("Failed to open current directory in dry run")?
            } else {
                SandboxableDir::create_or_open(tgt_root)?
            };

            let mut dir_queue: Vec<(PathBuf, PathBuf)> =
                vec![(src_root.clone(), tgt_root.clone())];

            while let Some((curr_src, curr_tgt)) = dir_queue.pop() {
                let dir_entity = match FileEntity::from_filesystem_with_apple_options(
                    &curr_src,
                    Some(src_root),
                    &apple_read_options,
                ) {
                    Ok(e) => e,
                    Err(e) => {
                        let _ = handle_item_error(
                            &curr_src,
                            JournalErrorStage::MetadataRead,
                            e,
                            args,
                            journal,
                            &mut stats,
                        )?;
                        continue;
                    }
                };
                if let Err(e) = validate_filesystem_known(
                    &curr_src,
                    dir_entity.metadata.filesystem_type.as_deref(),
                    args,
                ) {
                    let _ = handle_item_error(
                        &curr_src,
                        JournalErrorStage::MetadataRead,
                        e,
                        args,
                        journal,
                        &mut stats,
                    )?;
                    continue;
                }

                if !args.dry_run {
                    if let Err(e) = dest_dir.ensure_dir_all(
                        &dir_entity.identity.relative_path,
                        options.path_policy,
                    ) {
                        let _ = handle_item_error(
                            &curr_src,
                            JournalErrorStage::Materialize,
                            e,
                            args,
                            journal,
                            &mut stats,
                        )?;
                        continue;
                    }
                }
                stats.dirs_created = stats.dirs_created.saturating_add(1);

                let mut journal_dir = dir_entity.clone();
                let rel = compute_journal_relative_path(journal.destination(), &curr_tgt);
                journal_dir.identity.relative_path = rel.clone();
                journal_dir.identity.raw_relative_path =
                    rel.as_os_str().as_encoded_bytes().to_vec();
                journal.record_entity(&journal_dir);
                if !args.dry_run {
                    journal.commit_batch()?;
                }

                let error_policy = if args.continue_on_error {
                    ctb_io::file::OnTraversalError::Skip
                } else {
                    ctb_io::file::OnTraversalError::Bail
                };
                let traversal_opts = ctb_io::file::TraversalOptions::new()
                    .one_file_system(args.one_file_system)
                    .error_policy(error_policy)
                    .apple_read_options(apple_read_options);

                let raw_dir_entries =
                    match ctb_io::file::read_dir_safe(&curr_src, &traversal_opts) {
                        Ok(entries) => entries,
                        Err(e) => {
                            let _ = handle_item_error(
                                &curr_src,
                                JournalErrorStage::Traversal,
                                e,
                                args,
                                journal,
                                &mut stats,
                            )?;
                            continue;
                        }
                    };
                let dir_entries = match ctb_io::file::validate_and_order_directory_entries(
                    raw_dir_entries,
                    &curr_src,
                    &curr_tgt,
                    tgt_root,
                    &dir_entity.identity.relative_path,
                    &apple_read_options,
                    apple_write_mode,
                    apple_single_write_extension,
                ) {
                    Ok(entries) => entries,
                    Err(e) => {
                        let _ = handle_item_error(
                            &curr_src,
                            JournalErrorStage::Traversal,
                            e,
                            args,
                            journal,
                            &mut stats,
                        )?;
                        continue;
                    }
                };

                let mut expected_filenames: Vec<Vec<u8>> = Vec::new();

                for item in dir_entries {
                    let entry_src = item.path;
                    let entry_name = item.file_name;
                    let entry_tgt = curr_tgt.join(&entry_name);

                    let entry_rel = if dir_entity.identity.relative_path.as_os_str().is_empty() {
                        PathBuf::from(&entry_name)
                    } else {
                        dir_entity.identity.relative_path.join(&entry_name)
                    };

                    if item.is_dir {
                        expected_filenames.push(entry_name.as_encoded_bytes().to_vec());
                        dir_queue.push((entry_src, entry_tgt));
                    } else if item.is_symlink {
                        let sym_entity = match FileEntity::from_filesystem_with_apple_options(
                            &entry_src,
                            Some(src_root),
                            &apple_read_options,
                        ) {
                            Ok(e) => e,
                            Err(e) => {
                                let _ = handle_item_error(
                                    &entry_src,
                                    JournalErrorStage::MetadataRead,
                                    e,
                                    args,
                                    journal,
                                    &mut stats,
                                )?;
                                continue;
                            }
                        };
                        if let Err(e) = validate_filesystem_known(
                            &entry_src,
                            sym_entity.metadata.filesystem_type.as_deref(),
                            args,
                        ) {
                            let _ = handle_item_error(
                                &entry_src,
                                JournalErrorStage::MetadataRead,
                                e,
                                args,
                                journal,
                                &mut stats,
                            )?;
                            continue;
                        }
                        expected_filenames.push(entry_name.as_encoded_bytes().to_vec());
                        let mut sym_entity = sym_entity;
                        sym_entity.identity.relative_path = entry_rel;
                        sym_entity.identity.raw_relative_path =
                            sym_entity.identity.relative_path.as_os_str().as_encoded_bytes().to_vec();
                        sym_entity.identity.raw_filename = entry_name.as_encoded_bytes().to_vec();
                        deferred_symlinks.push(DeferredSymlink {
                            src_path: entry_src,
                            dest_path: entry_tgt,
                            entity: sym_entity,
                            dest_dir_root: tgt_root.clone(),
                        });
                    } else {
                        let created_path = copy_single_item(
                            &entry_src,
                            &entry_tgt,
                            &entry_rel,
                            &dest_dir,
                            &options,
                            args,
                            journal,
                            &mut hardlink_map,
                            &mut stats,
                            &mut files_to_verify,
                        )?;
                        if let Some(dest) = created_path {
                            if let Some(fname) = dest.file_name() {
                                expected_filenames.push(fname.as_encoded_bytes().to_vec());
                            }
                        }

                        uncommitted_count = uncommitted_count.saturating_add(1);
                        if uncommitted_count >= 500 && !args.dry_run {
                            journal.commit_batch()?;
                            uncommitted_count = 0;
                        }

                        if progress.is_enabled() {
                            let detail = format!("{} bytes", stats.bytes_copied);
                            progress.update_task(copy_task, stats.files_copied, Some(&detail));
                        }
                    }
                }

                deferred_dirs.push(DeferredDirFixup {
                    src_path: curr_src,
                    dest_path: curr_tgt.clone(),
                    dir_entity: dir_entity.clone(),
                    expected_filenames,
                });
            }
        } else {
            // Reason for fallback: A single-component relative destination (e.g. "output.bin") has no parent path or an empty parent path (""); falling back to current working directory "." correctly targets the local directory.
            let parent_dest = match tgt_root.parent() {
                Some(p) if !p.as_os_str().is_empty() => p,
                _ => Path::new("."),
            };
            let target_file_name = tgt_root
                .file_name()
                .context("Target path has no file name component")?;

            let dest_dir = if args.dry_run {
                SandboxableDir::open(".").context("Failed to open current directory in dry run")?
            } else {
                SandboxableDir::create_or_open(parent_dest)?
            };

            let target_rel_path = Path::new(target_file_name);

            if src_meta.is_symlink() {
                let mut sym_entity = match FileEntity::from_filesystem(src_root, None) {
                    Ok(e) => e,
                    Err(e) => {
                        let _ = handle_item_error(
                            src_root,
                            JournalErrorStage::MetadataRead,
                            e,
                            args,
                            journal,
                            &mut stats,
                        )?;
                        continue;
                    }
                };
                if let Err(e) = validate_filesystem_known(
                    src_root,
                    sym_entity.metadata.filesystem_type.as_deref(),
                    args,
                ) {
                    let _ = handle_item_error(
                        src_root,
                        JournalErrorStage::MetadataRead,
                        e,
                        args,
                        journal,
                        &mut stats,
                    )?;
                    continue;
                }
                sym_entity.identity.relative_path = target_rel_path.to_path_buf();
                sym_entity.identity.raw_relative_path =
                    target_file_name.as_encoded_bytes().to_vec();
                sym_entity.identity.raw_filename =
                    target_file_name.as_encoded_bytes().to_vec();
                deferred_symlinks.push(DeferredSymlink {
                    src_path: src_root.clone(),
                    dest_path: tgt_root.clone(),
                    entity: sym_entity,
                    dest_dir_root: parent_dest.to_path_buf(),
                });
            } else {
                copy_single_item(
                    src_root,
                    tgt_root,
                    target_rel_path,
                    &dest_dir,
                    &options,
                    args,
                    journal,
                    &mut hardlink_map,
                    &mut stats,
                    &mut files_to_verify,
                )?;
            }
        }
    }

    // =========================================================================
    // PASS 2: Defer symlink creation until all regular files & dirs are on disk
    // =========================================================================
    for symlink_item in deferred_symlinks {
        let dest_path = &symlink_item.dest_path;
        let entity = symlink_item.entity;
        if !args.dry_run {
            record_journal_entry(journal, dest_path, &entity)?;
            journal.commit_batch()?;
        }

        let dest_dir = if args.dry_run {
            SandboxableDir::open(".").context("Failed to open current directory in dry run")?
        } else {
            SandboxableDir::create_or_open(&symlink_item.dest_dir_root)?
        };

        if let Err(e) = materialize_entity(&entity, None, &dest_dir, &options) {
            let _ = handle_item_error(
                &symlink_item.src_path,
                JournalErrorStage::Materialize,
                e,
                args,
                journal,
                &mut stats,
            )?;
            continue;
        }

        if !args.dry_run {
            let dest_target = match std::fs::read_link(dest_path) {
                Ok(t) => t,
                Err(e) => {
                    let _ = handle_item_error(
                        &symlink_item.src_path,
                        JournalErrorStage::Materialize,
                        e.into(),
                        args,
                        journal,
                        &mut stats,
                    )?;
                    continue;
                }
            };
            #[cfg(unix)]
            {
                let atime = filetime::FileTime::from_unix_time(
                    entity.metadata.timestamps.atime_sec,
                    entity.metadata.timestamps.atime_nsec,
                );
                let mtime = filetime::FileTime::from_unix_time(
                    entity.metadata.timestamps.mtime_sec,
                    entity.metadata.timestamps.mtime_nsec,
                );
                let _ = filetime::set_symlink_file_times(dest_path, atime, mtime);
            }
            if let FileEntityKind::Symlink { target } = &entity.kind {
                if dest_target.as_os_str().as_encoded_bytes() != target.as_slice() {
                    let err = anyhow::anyhow!(
                        "Target filesystem altered or normalized symlink target for {}",
                        dest_path.display()
                    );
                    let _ = handle_item_error(
                        &symlink_item.src_path,
                        JournalErrorStage::Materialize,
                        err,
                        args,
                        journal,
                        &mut stats,
                    )?;
                    continue;
                }
            }
        }

        stats.symlinks_created = stats.symlinks_created.saturating_add(1);
        record_journal_entry(journal, dest_path, &entity)?;
        files_to_verify.push((symlink_item.src_path, dest_path.clone(), entity));
    }

    // =========================================================================
    // PASS 3: Apply directory metadata in reverse traversal order (leaf-first)
    // =========================================================================
    if !args.dry_run {
        while let Some(fixup) = deferred_dirs.pop() {
            if fixup.dest_path.exists() {
                let expected_refs: Vec<&[u8]> =
                    fixup.expected_filenames.iter().map(Vec::as_slice).collect();
                if let Err(e) = verify_directory_filenames_exact(&fixup.dest_path, &expected_refs) {
                    let _ = handle_item_error(
                        &fixup.src_path,
                        JournalErrorStage::Materialize,
                        e,
                        args,
                        journal,
                        &mut stats,
                    )?;
                    continue;
                }

                if let Err(e) = write_streams(&fixup.dest_path, None, &fixup.dir_entity.streams, strict_lossless) {
                    let _ = handle_item_error(
                        &fixup.src_path,
                        JournalErrorStage::Materialize,
                        e,
                        args,
                        journal,
                        &mut stats,
                    )?;
                    continue;
                }
                if let Err(e) = apply_entity_metadata(
                    &fixup.dest_path,
                    None,
                    &fixup.dir_entity.metadata,
                    false,
                    true,
                    strict_lossless,
                ) {
                    let _ = handle_item_error(
                        &fixup.src_path,
                        JournalErrorStage::Materialize,
                        e,
                        args,
                        journal,
                        &mut stats,
                    )?;
                    continue;
                }
                if let Err(e) = verify_materialized_entity_ext(
                    &fixup.dest_path,
                    &fixup.dir_entity,
                    strict_lossless,
                    args.should_check_atime(),
                    args.should_check_ctime(),
                ) {
                    let _ = handle_item_error(
                        &fixup.src_path,
                        JournalErrorStage::Materialize,
                        e,
                        args,
                        journal,
                        &mut stats,
                    )?;
                    continue;
                }
                files_to_verify.push((fixup.src_path, fixup.dest_path, fixup.dir_entity));
            } else {
                let err = anyhow::anyhow!("Destination directory disappeared: {}", fixup.dest_path.display());
                let _ = handle_item_error(
                    &fixup.src_path,
                    JournalErrorStage::Materialize,
                    err,
                    args,
                    journal,
                    &mut stats,
                )?;
                continue;
            }
        }
        journal.commit_batch()?;
    }

    if progress.is_enabled() && stats.files_copied > 0 {
        let detail = format!("{} files ({} bytes)", stats.files_copied, stats.bytes_copied);
        progress.finish_task(copy_task, Some(&detail));
    }

    // =========================================================================
    // PASS 4: Post-flush independent verification pass
    // =========================================================================
    if args.should_verify_after() && !args.dry_run {
        progress.message("[Verifying] Flushing caches and verifying checksums...");
        try_drop_system_caches();

        let total_to_verify = u64::try_from(files_to_verify.len())?;
        let verify_task = progress.start_task("Verifying", Some(total_to_verify));

        for (src_path, dest_path, entity) in &files_to_verify {
            let check_atime = args.should_check_atime()
                || (entity.is_regular() && entity.metadata.used_noatime() && !args.best_effort_metadata);
            if let Err(e) = verify_materialized_entity_ext(
                src_path,
                entity,
                !args.best_effort_metadata,
                check_atime,
                args.should_check_ctime(),
            ) {
                let _ = handle_item_error(
                    src_path,
                    JournalErrorStage::PostCheck,
                    e,
                    args,
                    journal,
                    &mut stats,
                )?;
                continue;
            }
            if let Err(e) = verify_materialized_entity_ext(
                dest_path,
                entity,
                strict_lossless,
                check_atime,
                args.should_check_ctime(),
            ) {
                let _ = handle_item_error(
                    dest_path,
                    JournalErrorStage::PostCheck,
                    e,
                    args,
                    journal,
                    &mut stats,
                )?;
                continue;
            }
            if entity.is_regular() {
                stats.files_verified = stats.files_verified.saturating_add(1);
            }

            if progress.is_enabled() {
                progress.update_task(verify_task, stats.files_verified, None);
            }
        }

        if progress.is_enabled() {
            progress.finish_task(verify_task, Some("Verification complete"));
        }
    }

    if args.should_verify_after() && !args.dry_run {
        journal.mark_completed()?;
    }

    stats.copied_entities = files_to_verify;
    Ok(stats)
}

#[expect(
    clippy::too_many_arguments,
    reason = "Internal worker separating pipeline state from configuration"
)]
fn copy_single_item(
    src_path: &Path,
    dest_path: &Path,
    dest_rel_path: &Path,
    dest_dir: &SandboxableDir,
    options: &MaterializeOptions,
    args: &CscArgs,
    journal: &mut JournalWriter,
    hardlink_map: &mut HashMap<(u64, u64), PathBuf>,
    stats: &mut CopyStats,
    files_to_verify: &mut Vec<(PathBuf, PathBuf, FileEntity)>,
) -> Result<Option<PathBuf>> {
    // 2. Discover full entity from filesystem
    let apple_read_options = args.resolve_apple_read_options();
    let mut entity = match FileEntity::from_filesystem_with_apple_options(
        src_path,
        None,
        &apple_read_options,
    ) {
        Ok(e) => e,
        Err(err) => {
            return handle_item_error(
                src_path,
                JournalErrorStage::MetadataRead,
                err,
                args,
                journal,
                stats,
            );
        }
    };
    if let Err(err) = validate_filesystem_known(
        src_path,
        entity.metadata.filesystem_type.as_deref(),
        args,
    ) {
        return handle_item_error(
            src_path,
            JournalErrorStage::MetadataRead,
            err,
            args,
            journal,
            stats,
        );
    }

    // If an AppleSingle extension was stripped on read, update destination relative path
    let effective_dest_rel = if let Some(dest_fname) = dest_rel_path.file_name().and_then(|f| f.to_str()) {
        let is_stripped = if apple_read_options.read_apple_single_as
            && (dest_fname.ends_with(".as") || dest_fname.ends_with(".AS"))
        {
            let stripped_len = dest_fname.len().saturating_sub(".as".len());
            dest_fname.get(..stripped_len).is_some_and(|s| entity.identity.raw_filename == s.as_bytes())
        } else if apple_read_options.read_apple_single_asf
            && (dest_fname.ends_with(".asf") || dest_fname.ends_with(".ASF"))
        {
            let stripped_len = dest_fname.len().saturating_sub(".asf".len());
            dest_fname.get(..stripped_len).is_some_and(|s| entity.identity.raw_filename == s.as_bytes())
        } else {
            false
        };

        if is_stripped {
            if let Ok(fname_str) = std::str::from_utf8(&entity.identity.raw_filename) {
                match dest_rel_path.parent() {
                    Some(p) if !p.as_os_str().is_empty() => p.join(fname_str),
                    _ => PathBuf::from(fname_str),
                }
            } else {
                dest_rel_path.to_path_buf()
            }
        } else {
            dest_rel_path.to_path_buf()
        }
    } else {
        dest_rel_path.to_path_buf()
    };
    // Reason for fallback: parent path defaults to current directory
    let parent_path = dest_path.parent().unwrap_or(Path::new("."));
    // Reason for fallback: file name defaults to verbatim OsStr
    let file_name = effective_dest_rel.file_name().unwrap_or(effective_dest_rel.as_os_str());
    let effective_dest_path = parent_path.join(file_name);

    entity.identity.relative_path = effective_dest_rel;
    entity.identity.raw_relative_path = entity.identity.relative_path.as_os_str().as_encoded_bytes().to_vec();
    if let Some(fname) = entity.identity.relative_path.file_name() {
        entity.identity.raw_filename = fname.as_encoded_bytes().to_vec();
    }
    if !args.dry_run {
        record_journal_entry(journal, &effective_dest_path, &entity)?;
        journal.commit_batch()?;
    }

    // 3. Hardlink detection (nlink > 1)
    if entity.identity.nlink > 1 && entity.is_regular() {
        let key = match &entity.identity.origin {
            FileOrigin::Filesystem { key, .. } => (key.device_id, key.inode),
            _ => anyhow::bail!("Filesystem entity has no inode identity"),
        };

        if let Some(first_target_rel) = hardlink_map.get(&key) {
            let this_full_path = dest_dir.root_path().join(&entity.identity.relative_path);
            let is_self = effective_dest_path == *first_target_rel
                || this_full_path == *first_target_rel
                || std::fs::canonicalize(&effective_dest_path).ok().as_deref()
                    == std::fs::canonicalize(first_target_rel).ok().as_deref();
            if !is_self {
                let original_entity = entity.clone();
                entity.kind = FileEntityKind::Hardlink {
                    target_relative_path: first_target_rel.as_os_str().as_encoded_bytes().to_vec(),
                };
                if let Err(err) = materialize_entity(&entity, None, dest_dir, options) {
                    return handle_item_error(
                        src_path,
                        JournalErrorStage::Materialize,
                        err,
                        args,
                        journal,
                        stats,
                    );
                }
                stats.hardlinks_created = stats.hardlinks_created.saturating_add(1);
                record_journal_entry(journal, &effective_dest_path, &entity)?;
                files_to_verify.push((src_path.to_path_buf(), effective_dest_path.clone(), original_entity));
                return Ok(Some(effective_dest_path));
            }
        }
    }

    // 4. Special files (FIFOs, device nodes, sockets, doors)
    match &entity.kind {
        FileEntityKind::Fifo
        | FileEntityKind::CharDevice { .. }
        | FileEntityKind::BlockDevice { .. }
        | FileEntityKind::Socket
        | FileEntityKind::Door => {
            if args.copy_block_devices_as_regular_files
                && matches!(entity.kind, FileEntityKind::BlockDevice { .. })
            {
                let dev_file = match std::fs::File::open(src_path).with_context(|| {
                    format!("Failed to open block device: {}", src_path.display())
                }) {
                    Ok(f) => f,
                    Err(err) => {
                        return handle_item_error(
                            src_path,
                            JournalErrorStage::PayloadOpen,
                            err,
                            args,
                            journal,
                            stats,
                        );
                    }
                };
                let size = match query_block_device_size(&dev_file).with_context(|| {
                    format!(
                        "Failed to determine size of block device: {}",
                        src_path.display()
                    )
                }) {
                    Ok(s) => s,
                    Err(err) => {
                        return handle_item_error(
                            src_path,
                            JournalErrorStage::MetadataRead,
                            err,
                            args,
                            journal,
                            stats,
                        );
                    }
                };
                entity.kind = FileEntityKind::Regular {
                    size,
                    sha256: [0_u8; 32],
                    is_sparse: false,
                    extents: vec![ctb_io::file::payload::Extent::Data {
                        offset: 0,
                        length: size,
                    }],
                };
            } else if args.copy_specials_as_specials
                && !matches!(entity.kind, FileEntityKind::Socket | FileEntityKind::Door)
            {
                if let Err(err) = materialize_entity(&entity, None, dest_dir, options) {
                    return handle_item_error(
                        src_path,
                        JournalErrorStage::Materialize,
                        err,
                        args,
                        journal,
                        stats,
                    );
                }
                stats.special_files_created = stats.special_files_created.saturating_add(1);
                record_journal_entry(journal, dest_path, &entity)?;
                files_to_verify.push((src_path.to_path_buf(), dest_path.to_path_buf(), entity));
                return Ok(Some(dest_path.to_path_buf()));
            } else {
                stats.special_files_skipped = stats.special_files_skipped.saturating_add(1);
                return Ok(None);
            }
        }
        _ => {}
    }

    // 5. Regular files: Sparse support, in-flight SHA-256, atomic rename
    #[cfg(unix)]
    let captured_mtime = entity.metadata.timestamps.mtime_sec;
    #[cfg(unix)]
    let captured_ctime = entity.metadata.timestamps.ctime_sec;
    let initial_size = match &entity.kind {
        FileEntityKind::Regular { size, .. } => *size,
        _ => 0,
    };

    // Materialize payload
    let apple_single_data = if apple_read_options.any_apple_single() && entity.is_regular() {
        if let Ok(data) = std::fs::read(src_path) {
            if let Ok(archive) = ctb_io::file::read_apple_single_double(&data) {
                if archive.format == ctb_io::file::AppleFormat::AppleSingle {
                    // Reason for fallback: empty data fork represents empty file payload
                    Some(archive.data_fork.unwrap_or_default())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let mut entity = entity;
    let (receipt, file_used_noatime) = if args.dry_run {
        (materialize_entity(&entity, None, dest_dir, options), false)
    } else if let Some(mem_bytes) = apple_single_data.as_ref() {
        let mut mem_payload = match ctb_io::file::MemoryPayloadSource::new(mem_bytes.clone()) {
            Ok(p) => p,
            Err(err) => {
                return handle_item_error(
                    src_path,
                    JournalErrorStage::PayloadOpen,
                    err,
                    args,
                    journal,
                    stats,
                );
            }
        };
        (
            materialize_entity(&entity, Some(&mut mem_payload), dest_dir, options),
            false,
        )
    } else if matches!(entity.kind, FileEntityKind::Regular { .. }) {
        let mut payload = match DiskPayloadSource::open(src_path) {
            Ok(p) => p,
            Err(err) => {
                return handle_item_error(
                    src_path,
                    JournalErrorStage::PayloadOpen,
                    err,
                    args,
                    journal,
                    stats,
                );
            }
        };
        let noatime = payload.opened_with_noatime();
        (
            materialize_entity(&entity, Some(&mut payload), dest_dir, options),
            noatime,
        )
    } else {
        (materialize_entity(&entity, None, dest_dir, options), false)
    };
    let receipt = match receipt.with_context(|| {
        format!(
            "copying '{}' -> '{}'",
            src_path.display(),
            effective_dest_path.display()
        )
    }) {
        Ok(r) => r,
        Err(err) => {
            return handle_item_error(
                src_path,
                JournalErrorStage::Materialize,
                err,
                args,
                journal,
                stats,
            );
        }
    };

    if file_used_noatime {
        entity.metadata.set_used_noatime(true);
        journal.record_noatime_used(true);
    }

    // Verify source wasn't modified concurrently during copy
    let after_meta = match std::fs::symlink_metadata(src_path) {
        Ok(m) => m,
        Err(err) => {
            return handle_item_error(
                src_path,
                JournalErrorStage::PostCheck,
                err.into(),
                args,
                journal,
                stats,
            );
        }
    };
    #[cfg(unix)]
    let is_block_device_as_regular = args.copy_block_devices_as_regular_files
        && after_meta.file_type().is_block_device();
    #[cfg(not(unix))]
    let is_block_device_as_regular = false;

    #[cfg(unix)]
    let changed = after_meta.mtime() != captured_mtime
        || after_meta.ctime() != captured_ctime
        || after_meta.mtime_nsec() != i64::from(entity.metadata.timestamps.mtime_nsec)
        || (apple_single_data.is_none() && after_meta.len() != initial_size);
    #[cfg(not(unix))]
    let changed = (apple_single_data.is_none() && after_meta.len() != initial_size)
        || filetime::FileTime::from_last_modification_time(&after_meta)
            != filetime::FileTime::from_unix_time(entity.metadata.timestamps.mtime_sec, entity.metadata.timestamps.mtime_nsec);

    if !is_block_device_as_regular && changed {
        if args.on_source_change == SourceChangePolicy::Error {
            let err = anyhow::anyhow!(
                "Source file {} was modified concurrently during copy (timestamps or size changed)",
                src_path.display()
            );
            return handle_item_error(
                src_path,
                JournalErrorStage::PostCheck,
                err,
                args,
                journal,
                stats,
            );
        }
        eprintln!(
            "WARNING: Source file {} changed during copy; proceeding best-effort.",
            src_path.display()
        );
        let _ = journal.record_warning(
            "SourceModifiedDuringCopy",
            &format!(
                "Source file {} changed during copy; proceeding best-effort.",
                src_path.display()
            ),
        );
    }

    if receipt.skipped_identical {
        stats.files_skipped_identical = stats.files_skipped_identical.saturating_add(1);
    } else {
        stats.files_copied = stats.files_copied.saturating_add(1);
    }
    stats.bytes_copied = stats.bytes_copied.saturating_add(receipt.bytes_written);

    if entity.identity.nlink > 1 && entity.is_regular() {
        if let FileOrigin::Filesystem { key, .. } = &entity.identity.origin {
            hardlink_map.insert(
                (key.device_id, key.inode),
                dest_dir.root_path().join(&entity.identity.relative_path),
            );
        }
    }

    if let FileEntityKind::Regular { ref mut sha256, .. } = entity.kind {
        if *sha256 == [0_u8; 32] {
            if let Some(h) = receipt.sha256 {
                *sha256 = h;
            }
        }
    }

    record_journal_entry(journal, &receipt.destination_path, &entity)?;
    let final_dest = receipt.destination_path;
    files_to_verify.push((src_path.to_path_buf(), final_dest.clone(), entity));

    Ok(Some(final_dest))
}

fn compute_journal_relative_path(journal_dest: &Path, dest_path: &Path) -> PathBuf {
    // Reason for fallback: path not prefixed with "./" retains original path unchanged
    let norm_dest = dest_path.strip_prefix("./").unwrap_or(dest_path);
    // Reason for fallback: path not prefixed with "./" retains original path unchanged
    let norm_journal = journal_dest.strip_prefix("./").unwrap_or(journal_dest);

    if norm_journal.as_os_str().is_empty() || norm_journal == Path::new(".") {
        return norm_dest.to_path_buf();
    }

    if let Ok(rel) = norm_dest.strip_prefix(norm_journal) {
        // Reason for fallback: path not prefixed with "./" retains original path unchanged
        return rel.strip_prefix("./").unwrap_or(rel).to_path_buf();
    }
    if let Ok(rel) = dest_path.strip_prefix(journal_dest) {
        return rel.to_path_buf();
    }

    norm_dest.to_path_buf()
}

fn record_journal_entry(
    journal: &mut JournalWriter,
    dest_path: &Path,
    entity: &FileEntity,
) -> Result<()> {
    let mut journal_entity = entity.clone();
    let rel = compute_journal_relative_path(journal.destination(), dest_path);
    journal_entity.identity.relative_path = rel.clone();
    journal_entity.identity.raw_relative_path = rel.as_os_str().as_encoded_bytes().to_vec();
    if let FileEntityKind::Hardlink { target_relative_path } = &mut journal_entity.kind {
        let target = ctb_io::file::resolve_relative_path_for_os(target_relative_path, cfg!(windows))?;
        // Reason for fallback: if destination cannot be canonicalized, use destination path directly
        let journal_root = std::fs::canonicalize(journal.destination())
            .unwrap_or_else(|_| journal.destination().to_path_buf());
        *target_relative_path = compute_journal_relative_path(&journal_root, &target)
            .as_os_str().as_encoded_bytes().to_vec();
    }
    journal.record_entity(&journal_entity);
    Ok(())
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
    use crate::args::{AppleDoubleStyle, default_test_args, default_verify_args};
    use crate::cli::run_csc;
    use crate::verifier::run_csc_verify;
    use crate::journal::find_cscjournal;
    use ctb_utilities::cli::ToolResult;
    use std::fs;
    use std::os::unix::fs::{PermissionsExt};
    use std::path::{PathBuf};
    use tempfile::tempdir;

    #[crate::ctb_test]
    fn test_copy_tree_with_verify() {
        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_dir");
        let dest = temp.path().join("dest_dir");
        let state = temp.path().join("state_dir");
        fs::create_dir_all(src.join("sub")).expect("create src sub");
        fs::create_dir_all(&state).expect("create state dir");

        fs::write(src.join("file1.txt"), b"Hello, World!").expect("write file1");
        fs::write(src.join("sub").join("file2.bin"), vec![42_u8; 1000]).expect("write file2");

        let args = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state,
        );

        let res = run_csc(args).expect("run csc");
        match res {
            ctb_utilities::cli::ToolResult::Immediate { stdout, exit_code, .. } => {
                assert_eq!(exit_code, 0);
                let out = String::from_utf8_lossy(&stdout);
                assert!(out.contains("Files copied:             2"));
                assert!(out.contains("Files verified:           2"));
            }
            _ => panic!("Expected Immediate ToolResult"),
        }

        assert_eq!(
            fs::read(dest.join("file1.txt")).expect("read file1"),
            b"Hello, World!"
        );
        assert_eq!(
            fs::read(dest.join("sub").join("file2.bin")).expect("read file2"),
            vec![42_u8; 1000]
        );
    }

    #[crate::ctb_test]
    fn test_hardlinks_preserved() {
        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_links");
        let dest = temp.path().join("dest_links");
        let state = temp.path().join("state_dir");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&state).expect("create state");

        let f1 = src.join("orig.txt");
        fs::write(&f1, b"Shared content across hardlinks").expect("write orig");

        let f2 = src.join("link.txt");
        fs::hard_link(&f1, &f2).expect("create hardlink");

        let args = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state,
        );

        run_csc(args).expect("run csc");

        let dest_f1 = dest.join("orig.txt");
        let dest_f2 = dest.join("link.txt");

        let m1 = fs::metadata(&dest_f1).expect("meta1");
        let m2 = fs::metadata(&dest_f2).expect("meta2");

        assert_eq!(m1.ino(), m2.ino());
        assert_eq!(m1.dev(), m2.dev());
        assert_eq!(m1.nlink(), 2);
    }

    #[crate::ctb_test]
    fn test_symlinks_preserved() {
        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_sym");
        let dest = temp.path().join("dest_sym");
        let state = temp.path().join("state_dir");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&state).expect("create state");

        let target_file = src.join("target.txt");
        fs::write(&target_file, b"Symlink target data").expect("write target");

        let sym = src.join("sym.txt");
        std::os::unix::fs::symlink("target.txt", &sym).expect("create symlink");

        let args = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state,
        );

        run_csc(args).expect("run csc");

        let dest_sym = dest.join("sym.txt");
        assert!(dest_sym.is_symlink());
        let read = fs::read_link(&dest_sym).expect("read symlink");
        assert_eq!(read, PathBuf::from("target.txt"));
    }

    #[crate::ctb_test]
    fn test_hardlinks_across_destination_roots() {
        let temp = tempdir().unwrap();
        let first = temp.path().join("first");
        let second = temp.path().join("second");
        let dest = temp.path().join("dest");
        let state = temp.path().join("state");
        fs::create_dir(&first).unwrap();
        fs::create_dir(&second).unwrap();
        fs::create_dir(&state).unwrap();
        fs::write(first.join("shared"), b"shared data").unwrap();
        fs::write(second.join("shared"), b"unrelated data").unwrap();
        fs::hard_link(first.join("shared"), second.join("linked")).unwrap();
        run_csc(default_test_args(vec![first, second, dest.clone()], state.clone())).unwrap();
        assert_eq!(fs::read(dest.join("second/linked")).unwrap(), b"shared data");
        assert_eq!(fs::read(dest.join("second/shared")).unwrap(), b"unrelated data");
        assert_eq!(fs::metadata(dest.join("first/shared")).unwrap().ino(), fs::metadata(dest.join("second/linked")).unwrap().ino());
        let result = run_csc_verify(&default_verify_args(find_cscjournal(&state), Some(dest))).unwrap();
        match result {
            ToolResult::Immediate { exit_code, .. } => assert_eq!(exit_code, 0),
            _ => panic!("Expected immediate verification result"),
        }
    }

    #[crate::ctb_test]
    fn test_skip_existing_checksum() {
        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_skip");
        let dest = temp.path().join("dest_skip");
        let state = temp.path().join("state_dir");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&dest).expect("create dest");
        fs::create_dir_all(&state).expect("create state");

        fs::write(src.join("file.txt"), b"Same data").expect("write src");
        fs::write(dest.join("file.txt"), b"Same data").expect("write dest");

        let args = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state,
        );

        let res = run_csc(args).expect("run csc");
        match res {
            ctb_utilities::cli::ToolResult::Immediate { stdout, .. } => {
                let out = String::from_utf8_lossy(&stdout);
                assert!(out.contains("Files skipped (identical): 1"));
                assert!(out.contains("Files copied:             0"));
            }
            _ => panic!("Expected Immediate ToolResult"),
        }
    }

    #[crate::ctb_test]
    fn test_resume_mode() {
        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_res");
        let dest = temp.path().join("dest_res");
        let state = temp.path().join("state_res");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&state).expect("create state");

        fs::write(src.join("first.txt"), b"First payload").expect("write first");

        let args1 = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state.clone(),
        );

        run_csc(args1).expect("run initial csc");

        // Locate created journal
        let mut journal_file = None;
        for entry in fs::read_dir(&state).expect("read state dir") {
            let entry = entry.expect("entry");
            if entry.path().extension().and_then(|e| e.to_str()) == Some("cscjournal") {
                journal_file = Some(entry.path());
                break;
            }
        }
        let journal_path = journal_file.expect("found journal");

        // Now resume without specifying source or dest!
        let mut resume_args = default_test_args(Vec::new(), state.clone());
        resume_args.resume = Some(journal_path);

        let res = run_csc(resume_args).expect("run resume csc");
        match res {
            ctb_utilities::cli::ToolResult::Immediate { stdout, .. } => {
                let out = String::from_utf8_lossy(&stdout);
                assert!(out.contains("already marked Completed") || out.contains("Summary"));
            }
            _ => panic!("Expected Immediate ToolResult"),
        }
    }

    #[cfg(unix)]

    #[crate::ctb_test]
    fn test_xattrs_and_streams_preserved() {
        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_xattrs");
        let dest = temp.path().join("dest_xattrs");
        let state = temp.path().join("state_dir");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&state).expect("create state");

        let file = src.join("data.bin");
        fs::write(&file, b"Main stream payload").expect("write file");

        xattr::set(&file, "user.test_attr", b"Value of attr 1").expect("set xattr 1");
        xattr::set(
            &file,
            "user.complex_stream",
            b"Multi-line\nstream\x00with\x01binary\xFFdata",
        )
        .expect("set xattr 2");

        let args = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state,
        );

        run_csc(args).expect("run csc with xattrs");

        let dest_file = dest.join("data.bin");
        assert_eq!(
            fs::read(&dest_file).expect("read dest"),
            b"Main stream payload"
        );

        let val1 = xattr::get(&dest_file, "user.test_attr")
            .expect("get xattr 1")
            .expect("attr 1 exists");
        assert_eq!(val1, b"Value of attr 1");

        let val2 = xattr::get(&dest_file, "user.complex_stream")
            .expect("get xattr 2")
            .expect("attr 2 exists");
        assert_eq!(val2, b"Multi-line\nstream\x00with\x01binary\xFFdata");
    }

    #[crate::ctb_test]
    fn test_resume_repairs_committed_destination_without_nesting() {
        let temp = tempdir().unwrap();
        let source = temp.path().join("source");
        let destination = temp.path().join("destination");
        let state = temp.path().join("state");
        fs::create_dir(&source).unwrap();
        fs::create_dir(&state).unwrap();
        fs::write(source.join("data"), b"original").unwrap();
        let mut args = default_test_args(vec![source, destination.clone()], state.clone());
        args.no_verify_after = true;
        args.verify_after = false;
        run_csc(args).unwrap();
        fs::write(destination.join("data"), b"damaged!").unwrap();
        let journal_path = find_cscjournal(&state);
        {
            use std::io::Write;
            let mut journal = fs::OpenOptions::new().append(true).open(&journal_path).unwrap();
            journal.write_all(b"damaged tail").unwrap();
        }
        let mut resume = default_test_args(Vec::new(), state.clone());
        resume.resume = Some(journal_path.clone());
        run_csc(resume).unwrap();
        assert_eq!(fs::read(destination.join("data")).unwrap(), b"original");
        assert!(!destination.join("source").exists());
        assert!(crate::journal::read_journal_snapshot(&journal_path).unwrap().is_completed);
    }

    #[crate::ctb_test]
    fn test_unusual_filenames_and_stream_names() {
        use std::os::unix::ffi::OsStrExt;

        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_unusual");
        let dest = temp.path().join("dest_unusual");
        let state = temp.path().join("state_dir");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&state).expect("create state");

        // 1. Filename with newlines
        let nl_file = src.join("file\nwith\nnewlines.txt");
        fs::write(&nl_file, b"Newlines in filename content").expect("write nl_file");

        // 2. Filename with unusual punctuation and spaces
        let punc_file = src.join("special !@#$%^&*()_+-=[]{}|;',.<>?~.txt");
        fs::write(&punc_file, b"Punctuation content").expect("write punc_file");

        // 3. Misencoded non-UTF-8 raw byte filename
        let misencoded_name = std::ffi::OsStr::from_bytes(b"misencoded_\xFF\xFE_\x80_junk.dat");
        let mis_file = src.join(misencoded_name);
        fs::write(&mis_file, b"Misencoded filename payload").expect("write mis_file");

        // 4. Stream name with binary junk
        let misencoded_stream =
            std::ffi::OsStr::from_bytes(b"user.stream_\xFF\xFE_\x80_junk");
        xattr::set(&mis_file, misencoded_stream, b"Stream with binary name")
            .expect("set binary stream");

        let args = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state,
        );

        run_csc(args).expect("run csc with unusual names");

        assert_eq!(
            fs::read(dest.join("file\nwith\nnewlines.txt")).expect("read nl"),
            b"Newlines in filename content"
        );
        assert_eq!(
            fs::read(dest.join("special !@#$%^&*()_+-=[]{}|;',.<>?~.txt"))
                .expect("read punc"),
            b"Punctuation content"
        );

        let dest_mis = dest.join(misencoded_name);
        assert_eq!(
            fs::read(&dest_mis).expect("read mis"),
            b"Misencoded filename payload"
        );

        let stream_val = xattr::get(&dest_mis, misencoded_stream)
            .expect("get binary stream")
            .expect("stream exists");
        assert_eq!(stream_val, b"Stream with binary name");
    }

    #[crate::ctb_test]
    fn test_long_path_names_and_deep_nesting() {
        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_long");
        let dest = temp.path().join("dest_long");
        let state = temp.path().join("state_dir");
        fs::create_dir_all(&state).expect("create state");

        // Long filename of 240 bytes (close to NAME_MAX = 255)
        let long_filename = "a".repeat(240);
        let long_file = src.join(&long_filename);
        fs::create_dir_all(&src).expect("create src");
        fs::write(&long_file, b"Payload in file with 240-byte name")
            .expect("write long file");

        // Deep directory nesting (15 levels deep)
        let mut deep_dir = src.clone();
        for i in 0..15 {
            deep_dir = deep_dir.join(format!("level_{i}"));
        }
        fs::create_dir_all(&deep_dir).expect("create deep dir");
        let nested_file = deep_dir.join("deep_nested.txt");
        fs::write(&nested_file, b"Nested file content").expect("write nested file");

        let args = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state,
        );

        run_csc(args).expect("run csc with long paths");

        assert_eq!(
            fs::read(dest.join(&long_filename)).expect("read long file"),
            b"Payload in file with 240-byte name"
        );

        let mut dest_deep = dest.clone();
        for i in 0..15 {
            dest_deep = dest_deep.join(format!("level_{i}"));
        }
        assert_eq!(
            fs::read(dest_deep.join("deep_nested.txt")).expect("read nested file"),
            b"Nested file content"
        );
    }

    #[crate::ctb_test]
    fn test_sparse_file_preservation() {
        use std::io::Seek;
        use std::io::SeekFrom;
        use std::io::Write;
        use ctb_io::file::{get_file_extents, Extent as FileExtent};

        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_sparse");
        let dest = temp.path().join("dest_sparse");
        let state = temp.path().join("state_dir");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&state).expect("create state");

        let sparse_src = src.join("sparse.img");
        let mut f = fs::File::create(&sparse_src).expect("create sparse file");
        // Write header
        f.write_all(b"HeaderData").expect("write header");
        // Seek forward 10MB
        f.seek(SeekFrom::Start(10 * 1024 * 1024)).expect("seek 10MB");
        // Write tail
        f.write_all(b"TailData").expect("write tail");
        f.sync_data().expect("sync sparse src");
        drop(f);

        let src_meta = fs::metadata(&sparse_src).expect("src meta");
        let expected_len = src_meta.len();

        let args = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state,
        );

        run_csc(args).expect("run csc with sparse file");

        let dest_sparse = dest.join("sparse.img");
        let dest_meta = fs::metadata(&dest_sparse).expect("dest meta");
        assert_eq!(dest_meta.len(), expected_len);

        // Verify extents on dest contain a hole
        let dest_file = fs::File::open(&dest_sparse).expect("open dest sparse");
        let extents = get_file_extents(&dest_file, expected_len).expect("get extents");
        let has_hole = extents.iter().any(|e| matches!(e, FileExtent::Hole { .. }));
        assert!(has_hole, "Expected destination file to preserve sparse hole");

        // Verify content integrity
        let mut dest_f = dest_file;
        let mut header = [0_u8; 10];
        dest_f.seek(SeekFrom::Start(0)).expect("seek to 0");
        std::io::Read::read_exact(&mut dest_f, &mut header).expect("read header");
        assert_eq!(&header, b"HeaderData");

        dest_f.seek(SeekFrom::Start(10 * 1024 * 1024)).expect("seek to tail");
        let mut tail = [0_u8; 8];
        std::io::Read::read_exact(&mut dest_f, &mut tail).expect("read tail");
        assert_eq!(&tail, b"TailData");
    }

    #[crate::ctb_test]
    fn test_timestamps_and_metadata_preserved() {
        use std::os::unix::fs::PermissionsExt;
        use filetime::{FileTime, set_file_times};

        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_meta");
        let dest = temp.path().join("dest_meta");
        let state = temp.path().join("state_dir");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&state).expect("create state");

        let file = src.join("custom_meta.txt");
        fs::write(&file, b"Testing metadata and timestamps").expect("write file");

        // Set permissions 0o640 (rw-r-----)
        fs::set_permissions(&file, fs::Permissions::from_mode(0o640))
            .expect("set permissions");

        // Set directory permissions 0o750 (rwxr-x---)
        fs::set_permissions(&src, fs::Permissions::from_mode(0o750))
            .expect("set dir perms");

        // Set specific mtime timestamp
        let custom_time = FileTime::from_unix_time(1_400_000_000, 500_000_000);
        set_file_times(&file, custom_time, custom_time).expect("set file times");

        let args = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state,
        );

        run_csc(args).expect("run csc");

        let dest_file = dest.join("custom_meta.txt");
        let dm = fs::metadata(&dest_file).expect("dest meta");
        assert_eq!(dm.permissions().mode() & 0o777, 0o640);
        assert_eq!(dm.mtime(), 1_400_000_000);

        let d_dir = fs::metadata(&dest).expect("dest dir meta");
        assert_eq!(d_dir.permissions().mode() & 0o777, 0o750);
    }

    #[crate::ctb_test]
    fn test_fifo_special_file_preserved() {
        use nix::sys::stat::Mode;
        use nix::unistd::mkfifo;
        use std::os::unix::fs::FileTypeExt;

        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_fifo");
        let dest = temp.path().join("dest_fifo");
        let state = temp.path().join("state_dir");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&state).expect("create state");

        let fifo_path = src.join("test.fifo");
        mkfifo(&fifo_path, Mode::from_bits_truncate(0o660)).expect("mkfifo");

        let args = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state.clone(),
        );

        let res = run_csc(args).expect("run csc with fifo default skip");
        if let ToolResult::Immediate { stdout, .. } = res {
            let out_str = String::from_utf8_lossy(&stdout);
            assert!(out_str.contains("Special files skipped:    1"));
        }
        let dest_fifo = dest.join("test.fifo");
        assert!(!dest_fifo.exists(), "Default should skip FIFO");

        // Now test with copy_specials_as_specials = true
        let dest_copy = temp.path().join("dest_fifo_copy");
        let state_copy = temp.path().join("state_dir_copy");
        fs::create_dir_all(&state_copy).expect("create state copy");
        let mut args_copy = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest_copy.clone(),
            ],
            state_copy,
        );
        args_copy.copy_specials_as_specials = true;
        run_csc(args_copy).expect("run csc with copy_specials_as_specials");

        let dest_fifo_node = dest_copy.join("test.fifo");
        let meta = fs::symlink_metadata(&dest_fifo_node).expect("fifo metadata");
        assert!(meta.file_type().is_fifo(), "Expected created node to be a FIFO");
    }

    #[crate::ctb_test]
    fn test_symlink_dangling_and_relative_target_preserved() {
        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_dangling");
        let dest = temp.path().join("dest_dangling");
        let state = temp.path().join("state_dir");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&state).expect("create state");

        let sym = src.join("dangling.sym");
        std::os::unix::fs::symlink("../non_existent_folder/absent.txt", &sym)
            .expect("create dangling symlink");

        let args = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state,
        );

        run_csc(args).expect("run csc");

        let dest_sym = dest.join("dangling.sym");
        assert!(dest_sym.is_symlink());
        let read = fs::read_link(&dest_sym).expect("read dangling symlink");
        assert_eq!(read, PathBuf::from("../non_existent_folder/absent.txt"));
    }

    #[crate::ctb_test]
    fn test_hardlinks_preserved_across_resume() {
        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_hl_res");
        let dest = temp.path().join("dest_hl_res");
        let state = temp.path().join("state_hl_res");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&state).expect("create state");

        let f1 = src.join("orig.txt");
        fs::write(&f1, b"Shared payload across resumed hardlinks").expect("write orig");
        let anchor = temp.path().join("anchor.tmp");
        fs::hard_link(&f1, &anchor).expect("create anchor link");

        let args1 = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state.clone(),
        );
        run_csc(args1).expect("initial run");

        let journal_path = find_cscjournal(&state);

        // Simulate an interrupted run by stripping the 1-byte TAG_JOB_COMPLETED marker
        let f = fs::OpenOptions::new()
            .write(true)
            .open(&journal_path)
            .expect("open journal");
        let len = f.metadata().expect("meta").len();
        f.set_len(len.saturating_sub(1)).expect("truncate completion tag");
        drop(f);
        let desc_path = journal_path.with_extension("cscdesc");
        let _ = fs::remove_file(&desc_path);

        let f2 = src.join("link.txt");
        fs::hard_link(&f1, &f2).expect("create link");
        let _ = fs::remove_file(&anchor);

        let mut resume_args = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state,
        );
        resume_args.resume = Some(journal_path);
        run_csc(resume_args).expect("resume run");

        let dest_f1 = dest.join("orig.txt");
        let dest_f2 = dest.join("link.txt");

        let m1 = fs::metadata(&dest_f1).expect("meta1");
        let m2 = fs::metadata(&dest_f2).expect("meta2");

        assert_eq!(m1.ino(), m2.ino(), "Inodes must match across resume");
        assert_eq!(m1.nlink(), 2, "Link count must be 2");
    }

    #[crate::ctb_test]
    fn test_best_effort_metadata_timestamp_tolerance() {
        use filetime::{FileTime, set_file_times};

        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_be");
        let dest = temp.path().join("dest_be");
        let state = temp.path().join("state_be");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&state).expect("create state");

        let file = src.join("file.txt");
        fs::write(&file, b"Timestamp drift test").expect("write file");

        let mut args = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state.clone(),
        );
        args.best_effort_metadata = true;
        run_csc(args).expect("run csc");

        let journal_path = find_cscjournal(&state);

        let dest_file = dest.join("file.txt");
        let dest_meta = fs::metadata(&dest_file).expect("dest meta");
        let orig_mtime = dest_meta.mtime();
        set_file_times(
            &dest_file,
            FileTime::from_unix_time(dest_meta.atime(), 0),
            FileTime::from_unix_time(orig_mtime.saturating_add(1), 0),
        )
        .expect("set drifted mtime");

        let mut strict_vargs = default_verify_args(journal_path.clone(), Some(dest.clone()));
        strict_vargs.best_effort = false;
        let res_strict = run_csc_verify(&strict_vargs).expect("verify strict");
        if let ToolResult::Immediate { stdout, .. } = res_strict {
            let out = String::from_utf8_lossy(&stdout);
            assert!(
                out.contains("discrepancies detected") || out.contains("mismatch"),
                "Strict mode must report timestamp mismatch: {out}"
            );
        }

        let mut be_vargs = default_verify_args(journal_path, Some(dest));
        be_vargs.best_effort = true;
        let res_be = run_csc_verify(&be_vargs).expect("verify best_effort");
        if let ToolResult::Immediate { stdout, .. } = res_be {
            let out = String::from_utf8_lossy(&stdout);
            assert!(
                out.contains("OK") || out.contains("0 discrepancies"),
                "Best effort mode must tolerate 1s timestamp difference: {out}"
            );
        }
    }

    #[crate::ctb_test]
    fn test_csc_default_skips_identical_file_and_updates_metadata() {
        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_skip");
        let dest = temp.path().join("dest_skip");
        let state1 = temp.path().join("state_dir1");
        let state2 = temp.path().join("state_dir2");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&state1).expect("create state1");
        fs::create_dir_all(&state2).expect("create state2");

        let file_path = src.join("doc.txt");
        let payload = b"Lossless smart skipping integration test payload";
        fs::write(&file_path, payload).expect("write initial file");
        fs::set_permissions(&file_path, fs::Permissions::from_mode(0o600)).expect("chmod initial");

        // First copy: fresh write of 1 file
        let args1 = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state1,
        );
        let res1 = run_csc(args1).expect("first copy");
        match res1 {
            ToolResult::Immediate { stdout, exit_code, .. } => {
                assert_eq!(exit_code, 0);
                let out = String::from_utf8_lossy(&stdout);
                assert!(out.contains("Files copied:             1"));
            }
            _ => panic!("Expected Immediate ToolResult"),
        }

        let dest_file = dest.join("doc.txt");
        assert_eq!(fs::read(&dest_file).expect("read dest"), payload);
        let meta1 = fs::metadata(&dest_file).expect("dest meta");
        assert_eq!(meta1.permissions().mode() & 0o7777, 0o600);

        // Update metadata on source file (change mode to 0o644)
        fs::set_permissions(&file_path, fs::Permissions::from_mode(0o644)).expect("chmod updated");

        // Second copy: default behavior should skip payload rewrite and losslessly update metadata in-place
        let args2 = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state2,
        );
        let res2 = run_csc(args2).expect("second copy with smart skip");
        match res2 {
            ToolResult::Immediate { stdout, exit_code, .. } => {
                assert_eq!(exit_code, 0);
                let out = String::from_utf8_lossy(&stdout);
                assert!(out.contains("Files copied:             0"));
                assert!(out.contains("Files skipped (identical): 1"));
                assert!(out.contains("Bytes transferred:        0"));
                assert!(out.contains("Files verified:           1"));
            }
            _ => panic!("Expected Immediate ToolResult"),
        }

        // Verify that destination metadata was updated in-place to 0o644
        let meta2 = fs::metadata(&dest_file).expect("dest meta after update");
        assert_eq!(meta2.permissions().mode() & 0o7777, 0o644);
        assert_eq!(fs::read(&dest_file).expect("read dest after update"), payload);
    }

    #[crate::ctb_test]
    fn test_csc_always_overwrite_rewrites_identical_file() {
        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_always");
        let dest = temp.path().join("dest_always");
        let state1 = temp.path().join("state_dir1");
        let state2 = temp.path().join("state_dir2");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&state1).expect("create state1");
        fs::create_dir_all(&state2).expect("create state2");

        let file_path = src.join("data.bin");
        let payload = b"Always overwrite payload verification";
        fs::write(&file_path, payload).expect("write data");

        // First copy
        let args1 = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state1,
        );
        run_csc(args1).expect("first copy");

        // Second copy with always_overwrite = true
        let mut args2 = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state2,
        );
        args2.always_overwrite = true;

        let res2 = run_csc(args2).expect("always overwrite copy");
        match res2 {
            ToolResult::Immediate { stdout, exit_code, .. } => {
                assert_eq!(exit_code, 0);
                let out = String::from_utf8_lossy(&stdout);
                assert!(out.contains("Files copied:             1"));
                let payload_len = payload.len();
                assert!(out.contains(&format!("Bytes transferred:        {payload_len}")));
            }
            _ => panic!("Expected Immediate ToolResult"),
        }
    }

    #[crate::ctb_test]
    fn test_unknown_filesystem_error_and_allow_flags() {
        use ctb_io::file::{clear_filesystem_cache, extract_device_id, set_cached_filesystem_info, FilesystemInfo};

        let _fs_lock = ctb_io::file::FS_CACHE_TEST_MUTEX.lock().unwrap();
        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src");
        let dest = temp.path().join("dest");
        let state = temp.path().join("state");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&state).expect("create state");
        fs::write(src.join("file.txt"), b"data").expect("write file");

        let meta = fs::metadata(&src).expect("read src metadata");
        let dev_id = extract_device_id(&meta).expect("dev id");

        // Simulate unknown filesystem
        set_cached_filesystem_info(dev_id, FilesystemInfo {
            fs_type: "unknown".to_string(),
            resolution_nsec: 1,
        });

        // 1. Without best-effort-metadata and without allow-unknown-fs -> should error
        let mut args = default_test_args(vec![src.clone(), dest.clone()], state.clone());
        args.best_effort_metadata = false;
        args.allow_unknown_fs = false;
        match crate::cli::run_csc(args) {
            Err(err) => {
                assert!(err.to_string().contains("Cannot detect filesystem type"));
            }
            Ok(_) => panic!("Expected error when filesystem is unknown"),
        }

        // 2. With allow-unknown-fs -> should succeed
        let dest2 = temp.path().join("dest2");
        let state2 = temp.path().join("state2");
        fs::create_dir_all(&state2).expect("create state2");
        let mut args2 = default_test_args(vec![src.clone(), dest2], state2);
        args2.best_effort_metadata = false;
        args2.allow_unknown_fs = true;
        match crate::cli::run_csc(args2) {
            Ok(_) => {}
            Err(e) => panic!("run_csc failed with allow_unknown_fs: {e:?}"),
        }

        // 3. With best-effort-metadata -> should succeed
        let dest3 = temp.path().join("dest3");
        let state3 = temp.path().join("state3");
        fs::create_dir_all(&state3).expect("create state3");
        let mut args3 = default_test_args(vec![src.clone(), dest3], state3);
        args3.best_effort_metadata = true;
        args3.allow_unknown_fs = false;
        match crate::cli::run_csc(args3) {
            Ok(_) => {}
            Err(e) => panic!("run_csc failed with best_effort_metadata: {e:?}"),
        }

        // Clean up cache
        clear_filesystem_cache();
    }

    #[crate::ctb_test]
    fn test_csc_noatime_recorded_and_verified() {
        let temp = tempdir().expect("tempdir");
        let src = temp.path().join("src");
        let dest = temp.path().join("dest");
        let state = temp.path().join("state");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&dest).expect("create dest");
        fs::create_dir_all(&state).expect("create state");

        let f = src.join("test_file.txt");
        fs::write(&f, b"hello noatime test").expect("write file");

        let mut args = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state.clone(),
        );
        args.verify_after = true;
        run_csc(args).expect("run csc");

        let journal_path = find_cscjournal(&state);
        let desc_path = journal_path.with_extension("cscdesc");
        assert!(desc_path.exists(), ".cscdesc must exist");
        let desc_content = fs::read_to_string(&desc_path).expect("read desc");
        assert!(
            desc_content.contains("NoatimeUsed: true"),
            "Descriptor should record NoatimeUsed: true when file owner copies file"
        );
    }

    #[crate::ctb_test]
    fn test_csc_copy_apple_double_alongside_roundtrip() {
        let temp = tempdir().expect("tempdir");
        let src = temp.path().join("src");
        let dest = temp.path().join("dest");
        let state = temp.path().join("state");
        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&state).unwrap();
        fs::write(src.join("hello.txt"), b"Hello Payload").unwrap();

        let archive = ctb_io::file::AppleArchive {
            format: ctb_io::file::AppleFormat::AppleDouble,
            version: ctb_io::file::VERSION_2_0_BE,
            real_name: Some("hello.txt".to_string()),
            comment: Some("Test Comment".to_string()),
            timestamps: None,
            backup_timestamp_sec: None,
            finder_info: Some(ctb_io::file::FinderInfo {
                file_type: "TEXT".to_string(),
                file_creator: "ttxt".to_string(),
                raw_flags: 0,
                label: ctb_io::file::FinderLabel::from_index(0),
                flags: ctb_io::file::FinderFlags::default(),
                location: (0, 0),
                folder_id: 0,
                extended: None,
            }),
            extended_attributes: Vec::new(),
            data_fork: None,
            resource_fork: Some(b"Resource Fork Bytes".to_vec()),
            data_fork_size: None,
            resource_fork_size: Some(19),
            entries: Vec::new(),
        };
        let companion_bytes = ctb_io::file::write_apple_single_double(&archive).unwrap();
        fs::write(src.join("._hello.txt"), companion_bytes).unwrap();

        let mut args = default_test_args(
            vec![PathBuf::from(format!("{}/", src.display())), dest.clone()],
            state,
        );
        args.read_apple_double_alongside = true;
        args.force_write_apple_double = Some(AppleDoubleStyle::Alongside);

        let res = run_csc(args).expect("run csc");
        match res {
            ctb_utilities::cli::ToolResult::Immediate { exit_code, .. } => {
                assert_eq!(exit_code, 0);
            }
            _ => panic!("Expected immediate result"),
        }

        assert_eq!(fs::read(dest.join("hello.txt")).unwrap(), b"Hello Payload");
        assert!(dest.join("._hello.txt").exists(), "Companion ._hello.txt must exist on dest");

        // Inspect entity with apple read options enabled
        let opts = ctb_io::file::AppleReadOptions {
            read_apple_double_alongside: true,
            ..ctb_io::file::AppleReadOptions::default()
        };
        let entity = ctb_io::file::FileEntity::from_filesystem_with_apple_options(
            &dest.join("hello.txt"),
            Some(&dest),
            &opts,
        ).unwrap();

        let rsrc_stream = entity.streams.iter().find(|s| s.kind == ctb_io::file::StreamKind::MacOsResourceFork);
        assert!(rsrc_stream.is_some(), "Resource fork must be preserved");
        assert_eq!(rsrc_stream.unwrap().data.as_deref(), Some(&b"Resource Fork Bytes"[..]));
        assert_eq!(
            entity.metadata.apple.as_ref().and_then(|a| a.finder_info.as_ref()).map(|f| f.file_type.as_str()),
            Some("TEXT")
        );
    }

    #[crate::ctb_test]
    fn test_csc_copy_apple_double_zip_roundtrip() {
        let temp = tempdir().expect("tempdir");
        let src = temp.path().join("src");
        let dest = temp.path().join("dest");
        let state = temp.path().join("state");
        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&state).unwrap();
        fs::write(src.join("zipdoc.txt"), b"Zip Payload").unwrap();

        let archive = ctb_io::file::AppleArchive {
            format: ctb_io::file::AppleFormat::AppleDouble,
            version: ctb_io::file::VERSION_2_0_BE,
            real_name: Some("zipdoc.txt".to_string()),
            comment: None,
            timestamps: None,
            backup_timestamp_sec: None,
            finder_info: Some(ctb_io::file::FinderInfo {
                file_type: "DOCU".to_string(),
                file_creator: "docu".to_string(),
                raw_flags: 0,
                label: ctb_io::file::FinderLabel::from_index(0),
                flags: ctb_io::file::FinderFlags::default(),
                location: (0, 0),
                folder_id: 0,
                extended: None,
            }),
            extended_attributes: Vec::new(),
            data_fork: None,
            resource_fork: Some(b"Zip Resource Fork".to_vec()),
            data_fork_size: None,
            resource_fork_size: Some(17),
            entries: Vec::new(),
        };
        let companion_bytes = ctb_io::file::write_apple_single_double(&archive).unwrap();
        fs::write(src.join("._zipdoc.txt"), companion_bytes).unwrap();

        let mut args = default_test_args(
            vec![PathBuf::from(format!("{}/", src.display())), dest.clone()],
            state,
        );
        args.read_apple_double_alongside = true;
        args.force_write_apple_double = Some(AppleDoubleStyle::Zip);

        let res = run_csc(args).expect("run csc");
        match res {
            ctb_utilities::cli::ToolResult::Immediate { exit_code, .. } => {
                assert_eq!(exit_code, 0);
            }
            _ => panic!("Expected immediate result"),
        }

        assert_eq!(fs::read(dest.join("zipdoc.txt")).unwrap(), b"Zip Payload");
        let macosx_companion = dest.join("__MACOSX").join("._zipdoc.txt");
        assert!(macosx_companion.exists(), "__MACOSX/._zipdoc.txt must exist on dest");

        let opts = ctb_io::file::AppleReadOptions {
            read_apple_double_zip: true,
            ..ctb_io::file::AppleReadOptions::default()
        };
        let entity = ctb_io::file::FileEntity::from_filesystem_with_apple_options(
            &dest.join("zipdoc.txt"),
            Some(&dest),
            &opts,
        ).unwrap();

        let rsrc_stream = entity.streams.iter().find(|s| s.kind == ctb_io::file::StreamKind::MacOsResourceFork);
        assert!(rsrc_stream.is_some(), "Resource fork must be preserved in zip style");
        assert_eq!(rsrc_stream.unwrap().data.as_deref(), Some(&b"Zip Resource Fork"[..]));
    }

    #[crate::ctb_test]
    fn test_csc_copy_apple_double_netatalk_roundtrip() {
        let temp = tempdir().expect("tempdir");
        let src = temp.path().join("src");
        let dest = temp.path().join("dest");
        let state = temp.path().join("state");
        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&state).unwrap();
        fs::write(src.join("netadoc.bin"), b"Netatalk Payload").unwrap();

        let archive = ctb_io::file::AppleArchive {
            format: ctb_io::file::AppleFormat::AppleDouble,
            version: ctb_io::file::VERSION_2_0_BE,
            real_name: Some("netadoc.bin".to_string()),
            comment: None,
            timestamps: None,
            backup_timestamp_sec: None,
            finder_info: Some(ctb_io::file::FinderInfo {
                file_type: "BINA".to_string(),
                file_creator: "bina".to_string(),
                raw_flags: 0,
                label: ctb_io::file::FinderLabel::from_index(0),
                flags: ctb_io::file::FinderFlags::default(),
                location: (0, 0),
                folder_id: 0,
                extended: None,
            }),
            extended_attributes: Vec::new(),
            data_fork: None,
            resource_fork: Some(b"Netatalk Resource Data".to_vec()),
            data_fork_size: None,
            resource_fork_size: Some(22),
            entries: Vec::new(),
        };
        let companion_bytes = ctb_io::file::write_apple_single_double(&archive).unwrap();
        fs::write(src.join("._netadoc.bin"), companion_bytes).unwrap();

        let mut args = default_test_args(
            vec![PathBuf::from(format!("{}/", src.display())), dest.clone()],
            state,
        );
        args.read_apple_double_alongside = true;
        args.force_write_apple_double = Some(AppleDoubleStyle::Netatalk);

        let res = run_csc(args).expect("run csc");
        match res {
            ctb_utilities::cli::ToolResult::Immediate { exit_code, .. } => {
                assert_eq!(exit_code, 0);
            }
            _ => panic!("Expected immediate result"),
        }

        assert_eq!(fs::read(dest.join("netadoc.bin")).unwrap(), b"Netatalk Payload");
        let netatalk_companion = dest.join(".AppleDouble").join("netadoc.bin");
        assert!(netatalk_companion.exists(), ".AppleDouble/netadoc.bin must exist on dest");

        let opts = ctb_io::file::AppleReadOptions {
            read_apple_double_netatalk: true,
            ..ctb_io::file::AppleReadOptions::default()
        };
        let entity = ctb_io::file::FileEntity::from_filesystem_with_apple_options(
            &dest.join("netadoc.bin"),
            Some(&dest),
            &opts,
        ).unwrap();

        let rsrc_stream = entity.streams.iter().find(|s| s.kind == ctb_io::file::StreamKind::MacOsResourceFork);
        assert!(rsrc_stream.is_some(), "Resource fork must be preserved in netatalk style");
        assert_eq!(rsrc_stream.unwrap().data.as_deref(), Some(&b"Netatalk Resource Data"[..]));
    }

    #[crate::ctb_test]
    fn test_csc_copy_apple_single_roundtrip_and_no_as_extension() {
        let temp = tempdir().expect("tempdir");
        let src = temp.path().join("src");
        let dest = temp.path().join("dest");
        let state = temp.path().join("state");
        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&state).unwrap();
        fs::write(src.join("my_file.txt"), b"AppleSingle Plain Data").unwrap();

        let archive = ctb_io::file::AppleArchive {
            format: ctb_io::file::AppleFormat::AppleDouble,
            version: ctb_io::file::VERSION_2_0_BE,
            real_name: Some("my_file.txt".to_string()),
            comment: Some("Single Test Comment".to_string()),
            timestamps: None,
            backup_timestamp_sec: None,
            finder_info: Some(ctb_io::file::FinderInfo {
                file_type: "APPL".to_string(),
                file_creator: "macs".to_string(),
                raw_flags: 0,
                label: ctb_io::file::FinderLabel::from_index(0),
                flags: ctb_io::file::FinderFlags::default(),
                location: (0, 0),
                folder_id: 0,
                extended: None,
            }),
            extended_attributes: Vec::new(),
            data_fork: None,
            resource_fork: Some(b"Single Resource Fork Data".to_vec()),
            data_fork_size: None,
            resource_fork_size: Some(25),
            entries: Vec::new(),
        };
        let companion_bytes = ctb_io::file::write_apple_single_double(&archive).unwrap();
        fs::write(src.join("._my_file.txt"), companion_bytes).unwrap();

        let mut args = default_test_args(
            vec![PathBuf::from(format!("{}/", src.display())), dest.clone()],
            state,
        );
        args.read_apple_double_alongside = true;
        args.force_write_apple_single = true;

        let res = run_csc(args).expect("run csc");
        match res {
            ctb_utilities::cli::ToolResult::Immediate { exit_code, .. } => {
                assert_eq!(exit_code, 0);
            }
            _ => panic!("Expected immediate result"),
        }

        // AppleSingle file MUST NOT get a .as extension appended!
        assert!(dest.join("my_file.txt").exists(), "Destination file must be my_file.txt");
        assert!(!dest.join("my_file.txt.as").exists(), "Destination file must NOT have .as appended");

        // Verify magic bytes on disk are AppleSingle
        let disk_bytes = fs::read(dest.join("my_file.txt")).unwrap();
        let magic = u32::from_be_bytes(disk_bytes[..4].try_into().unwrap());
        assert_eq!(magic, ctb_io::file::APPLESINGLE_MAGIC_BE);

        // Read and decode with read_apple_single
        let opts = ctb_io::file::AppleReadOptions {
            read_apple_single: true,
            ..ctb_io::file::AppleReadOptions::default()
        };
        let entity = ctb_io::file::FileEntity::from_filesystem_with_apple_options(
            &dest.join("my_file.txt"),
            Some(&dest),
            &opts,
        ).unwrap();

        assert!(entity.is_regular());
        if let ctb_io::file::FileEntityKind::Regular { size, .. } = entity.kind {
            assert_eq!(size, 22); // length of b"AppleSingle Plain Data"
        } else {
            panic!("Expected regular entity");
        }

        let rsrc_stream = entity.streams.iter().find(|s| s.kind == ctb_io::file::StreamKind::MacOsResourceFork);
        assert!(rsrc_stream.is_some(), "Resource fork must be preserved in AppleSingle");
        assert_eq!(rsrc_stream.unwrap().data.as_deref(), Some(&b"Single Resource Fork Data"[..]));
        assert_eq!(
            entity.metadata.apple.as_ref().and_then(|a| a.finder_info.as_ref()).map(|f| f.file_type.as_str()),
            Some("APPL")
        );
    }

    #[crate::ctb_test]
    fn test_csc_orphan_apple_double_preserved_during_traversal() {
        let temp = tempdir().expect("tempdir");
        let src = temp.path().join("src");
        let dest = temp.path().join("dest");
        let state = temp.path().join("state");
        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&state).unwrap();

        fs::write(src.join("valid.txt"), b"Valid base file").unwrap();

        let archive = ctb_io::file::AppleArchive {
            format: ctb_io::file::AppleFormat::AppleDouble,
            version: ctb_io::file::VERSION_2_0_BE,
            real_name: Some("valid.txt".to_string()),
            comment: None,
            timestamps: None,
            backup_timestamp_sec: None,
            finder_info: None,
            extended_attributes: Vec::new(),
            data_fork: None,
            resource_fork: Some(b"valid rsrc".to_vec()),
            data_fork_size: None,
            resource_fork_size: Some(10),
            entries: Vec::new(),
        };
        let valid_companion_bytes = ctb_io::file::write_apple_single_double(&archive).unwrap();
        fs::write(src.join("._valid.txt"), valid_companion_bytes).unwrap();

        // Orphaned AppleDouble companion (no sibling "orphan.txt")
        let orphan_archive = ctb_io::file::AppleArchive {
            format: ctb_io::file::AppleFormat::AppleDouble,
            version: ctb_io::file::VERSION_2_0_BE,
            real_name: Some("orphan.txt".to_string()),
            comment: None,
            timestamps: None,
            backup_timestamp_sec: None,
            finder_info: None,
            extended_attributes: Vec::new(),
            data_fork: None,
            resource_fork: Some(b"orphan rsrc".to_vec()),
            data_fork_size: None,
            resource_fork_size: Some(11),
            entries: Vec::new(),
        };
        let orphan_bytes = ctb_io::file::write_apple_single_double(&orphan_archive).unwrap();
        fs::write(src.join("._orphan.txt"), orphan_bytes).unwrap();

        // Plain text file starting with "._" that is NOT an AppleDouble file
        fs::write(src.join("._fake.txt"), b"plain text not appledouble").unwrap();

        let mut args = default_test_args(
            vec![PathBuf::from(format!("{}/", src.display())), dest.clone()],
            state,
        );
        args.read_apple_double_alongside = true;

        let res = run_csc(args).expect("run csc");
        match res {
            ctb_utilities::cli::ToolResult::Immediate { exit_code, .. } => {
                assert_eq!(exit_code, 0);
            }
            _ => panic!("Expected immediate result"),
        }

        // 1. valid.txt must exist on dest
        assert!(dest.join("valid.txt").exists());

        // 2. Orphaned ._orphan.txt MUST NOT be omitted; it must be copied as an ordinary file
        assert!(dest.join("._orphan.txt").exists(), "Orphaned ._orphan.txt must be preserved on dest");

        // 3. Fake ._fake.txt MUST NOT be omitted; it must be copied as an ordinary file
        assert!(dest.join("._fake.txt").exists(), "Fake ._fake.txt must be preserved on dest");
        assert_eq!(fs::read(dest.join("._fake.txt")).unwrap(), b"plain text not appledouble");
    }

    #[crate::ctb_test]
    fn test_csc_read_apple_single_with_extension_strips() {
        let temp = tempdir().expect("tempdir");
        let src = temp.path().join("src");
        let dest = temp.path().join("dest");
        let state = temp.path().join("state");
        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&state).unwrap();

        let archive = ctb_io::file::AppleArchive {
            format: ctb_io::file::AppleFormat::AppleSingle,
            version: ctb_io::file::VERSION_2_0_BE,
            real_name: Some("doc".to_string()),
            comment: None,
            timestamps: None,
            backup_timestamp_sec: None,
            finder_info: None,
            extended_attributes: Vec::new(),
            data_fork: Some(b"Unpacked doc payload".to_vec()),
            resource_fork: None,
            data_fork_size: Some(20),
            resource_fork_size: None,
            entries: Vec::new(),
        };
        let single_bytes = ctb_io::file::write_apple_single_double(&archive).unwrap();
        fs::write(src.join("doc.as"), single_bytes).unwrap();

        let mut args = default_test_args(
            vec![PathBuf::from(format!("{}/", src.display())), dest.clone()],
            state,
        );
        args.read_apple_single_with_extension = vec![ctb_io::file::AppleSingleExtension::As];

        let res = run_csc(args).expect("run csc");
        match res {
            ctb_utilities::cli::ToolResult::Immediate { exit_code, .. } => {
                assert_eq!(exit_code, 0);
            }
            _ => panic!("Expected immediate result"),
        }

        // Extension .as must be stripped automatically upon valid AppleSingle decode
        assert!(dest.join("doc").exists(), "Stripped destination 'doc' must exist");
        assert_eq!(fs::read(dest.join("doc")).unwrap(), b"Unpacked doc payload");
        assert!(!dest.join("doc.as").exists(), "doc.as must not exist on dest");
    }

    #[crate::ctb_test]
    fn test_csc_read_apple_single_non_applesingle_preserves_extension() {
        let temp = tempdir().expect("tempdir");
        let src = temp.path().join("src");
        let dest = temp.path().join("dest");
        let state = temp.path().join("state");
        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&state).unwrap();

        // Ordinary ActionScript source file (not AppleSingle)
        fs::write(src.join("script.as"), b"package { class Main {} }").unwrap();

        let mut args = default_test_args(
            vec![PathBuf::from(format!("{}/", src.display())), dest.clone()],
            state,
        );
        args.read_apple_single_with_extension = vec![ctb_io::file::AppleSingleExtension::As];

        let res = run_csc(args).expect("run csc");
        match res {
            ctb_utilities::cli::ToolResult::Immediate { exit_code, .. } => {
                assert_eq!(exit_code, 0);
            }
            _ => panic!("Expected immediate result"),
        }

        // Non-AppleSingle file must preserve its .as extension intact
        assert!(dest.join("script.as").exists(), "script.as must remain intact");
        assert_eq!(fs::read(dest.join("script.as")).unwrap(), b"package { class Main {} }");
        assert!(!dest.join("script").exists(), "script must not exist");
    }

    #[crate::ctb_test]
    fn test_csc_apple_single_dependency_ordering() {
        let temp = tempdir().expect("tempdir");
        let src = temp.path().join("src");
        let dest = temp.path().join("dest");
        let state = temp.path().join("state");
        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&state).unwrap();

        // Foo.as and Foo.as.as
        fs::write(src.join("Foo.as"), b"content of Foo.as").unwrap();
        fs::write(src.join("Foo.as.as"), b"content of Foo.as.as").unwrap();

        let mut args = default_test_args(
            vec![PathBuf::from(format!("{}/", src.display())), dest.clone()],
            state,
        );
        args.force_write_apple_single = true;
        args.write_apple_single_with_extension = Some(ctb_io::file::AppleSingleExtension::As);

        let res = run_csc(args).expect("run csc");
        match res {
            ctb_utilities::cli::ToolResult::Immediate { exit_code, .. } => {
                assert_eq!(exit_code, 0);
            }
            _ => panic!("Expected immediate result"),
        }

        // Foo.as -> Foo.as.as and Foo.as.as -> Foo.as.as.as
        assert!(dest.join("Foo.as.as").exists(), "Foo.as.as must exist");
        assert!(dest.join("Foo.as.as.as").exists(), "Foo.as.as.as must exist");

        let d1 = fs::read(dest.join("Foo.as.as")).unwrap();
        let a1 = ctb_io::file::read_apple_single_double(&d1).unwrap();
        assert_eq!(a1.data_fork.as_deref(), Some(&b"content of Foo.as"[..]));

        let d2 = fs::read(dest.join("Foo.as.as.as")).unwrap();
        let a2 = ctb_io::file::read_apple_single_double(&d2).unwrap();
        assert_eq!(a2.data_fork.as_deref(), Some(&b"content of Foo.as.as"[..]));
    }

    #[crate::ctb_test]
    fn test_csc_apple_single_name_collision_preservation() {
        let temp = tempdir().expect("tempdir");
        let src = temp.path().join("src");
        let dest = temp.path().join("dest");
        let state = temp.path().join("state");
        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&state).unwrap();

        // Three files with potential collision: Foo, Foo.as, Foo.as.as
        fs::write(src.join("Foo"), b"content of Foo").unwrap();
        fs::write(src.join("Foo.as"), b"content of Foo.as").unwrap();
        fs::write(src.join("Foo.as.as"), b"content of Foo.as.as").unwrap();

        let mut args = default_test_args(
            vec![PathBuf::from(format!("{}/", src.display())), dest.clone()],
            state,
        );
        args.force_write_apple_single = true;

        let res = run_csc(args).expect("run csc");
        match res {
            ctb_utilities::cli::ToolResult::Immediate { exit_code, .. } => {
                assert_eq!(exit_code, 0);
            }
            _ => panic!("Expected immediate result"),
        }

        // All three files must exist independently and not clobber each other
        assert!(dest.join("Foo").exists());
        assert!(dest.join("Foo.as").exists());
        assert!(dest.join("Foo.as.as").exists());

        let opts = ctb_io::file::AppleReadOptions {
            read_apple_single: true,
            ..ctb_io::file::AppleReadOptions::default()
        };

        let e1 = ctb_io::file::FileEntity::from_filesystem_with_apple_options(&dest.join("Foo"), Some(&dest), &opts).unwrap();
        let e2 = ctb_io::file::FileEntity::from_filesystem_with_apple_options(&dest.join("Foo.as"), Some(&dest), &opts).unwrap();
        let e3 = ctb_io::file::FileEntity::from_filesystem_with_apple_options(&dest.join("Foo.as.as"), Some(&dest), &opts).unwrap();

        if let ctb_io::file::FileEntityKind::Regular { size, .. } = e1.kind {
            assert_eq!(size, 14); // "content of Foo"
        }
        if let ctb_io::file::FileEntityKind::Regular { size, .. } = e2.kind {
            assert_eq!(size, 17); // "content of Foo.as"
        }
        if let ctb_io::file::FileEntityKind::Regular { size, .. } = e3.kind {
            assert_eq!(size, 20); // "content of Foo.as.as"
        }
    }

    #[crate::ctb_test]
    fn test_csc_read_apple_single_collision_bailout() {
        let temp = tempdir().expect("tempdir");
        let src = temp.path().join("src");
        let dest = temp.path().join("dest");
        let state = temp.path().join("state");
        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&state).unwrap();

        // Sibling files: foo and foo.as
        fs::write(src.join("foo"), b"regular file foo").unwrap();

        let archive = ctb_io::file::AppleArchive {
            format: ctb_io::file::AppleFormat::AppleSingle,
            version: ctb_io::file::VERSION_2_0_BE,
            real_name: Some("foo".to_string()),
            comment: None,
            timestamps: None,
            backup_timestamp_sec: None,
            finder_info: None,
            extended_attributes: Vec::new(),
            data_fork: Some(b"applesingle foo".to_vec()),
            resource_fork: None,
            data_fork_size: Some(15),
            resource_fork_size: None,
            entries: Vec::new(),
        };
        let single_bytes = ctb_io::file::write_apple_single_double(&archive).unwrap();
        fs::write(src.join("foo.as"), single_bytes).unwrap();

        let mut args = default_test_args(
            vec![PathBuf::from(format!("{}/", src.display())), dest],
            state,
        );
        args.read_apple_single_with_extension = vec![ctb_io::file::AppleSingleExtension::As];

        // Must bail out because both unpack/target 'foo'
        match run_csc(args) {
            Err(err) => {
                let err_msg = err.to_string();
                assert!(err_msg.contains("collision") || err_msg.contains("Target collision"), "Error message must report collision: {err_msg}");
            }
            Ok(_) => panic!("Must error on target collision between foo and foo.as"),
        }
    }

    #[crate::ctb_test]
    fn test_csc_apple_double_companion_collision_bailout() {
        let temp = tempdir().expect("tempdir");
        let src = temp.path().join("src");
        let dest = temp.path().join("dest");
        let state = temp.path().join("state");
        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&state).unwrap();

        fs::write(src.join("file.txt"), b"base file").unwrap();
        // Independent file that happens to match the companion path
        fs::write(src.join("._file.txt"), b"independent not appledouble").unwrap();

        let mut args = default_test_args(
            vec![PathBuf::from(format!("{}/", src.display())), dest],
            state,
        );
        args.force_write_apple_double = Some(AppleDoubleStyle::Alongside);

        match run_csc(args) {
            Err(err) => {
                let err_msg = err.to_string();
                assert!(err_msg.contains("collision") || err_msg.contains("Companion collision"), "Error message must report collision: {err_msg}");
            }
            Ok(_) => panic!("Must error on companion collision with independent ._file.txt"),
        }
    }

    #[crate::ctb_test]
    fn test_csc_netatalk_companion_collision_bailout() {
        let temp = tempdir().expect("tempdir");
        let src = temp.path().join("src");
        let dest = temp.path().join("dest");
        let state = temp.path().join("state");
        fs::create_dir_all(src.join(".AppleDouble")).unwrap();
        fs::create_dir_all(&state).unwrap();

        fs::write(src.join("file.txt"), b"base file").unwrap();
        // Preexisting independent file in .AppleDouble
        fs::write(src.join(".AppleDouble").join("file.txt"), b"independent netatalk").unwrap();

        let mut args = default_test_args(
            vec![PathBuf::from(format!("{}/", src.display())), dest],
            state,
        );
        args.force_write_apple_double = Some(AppleDoubleStyle::Netatalk);

        let res = run_csc(args);
        assert!(res.is_err(), "Must error on companion collision with independent .AppleDouble/file.txt");
    }

    #[crate::ctb_test]
    fn test_csc_zip_apple_double_macosx_file_collision_bailout() {
        let temp = tempdir().expect("tempdir");
        let src = temp.path().join("src");
        let dest = temp.path().join("dest");
        let state = temp.path().join("state");
        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&state).unwrap();

        fs::write(src.join("__MACOSX"), b"regular file not dir").unwrap();
        fs::write(src.join("foo.txt"), b"some content").unwrap();

        let mut args = default_test_args(
            vec![PathBuf::from(format!("{}/", src.display())), dest],
            state,
        );
        args.force_write_apple_double = Some(AppleDoubleStyle::Zip);

        match run_csc(args) {
            Err(err) => {
                let msg = err.to_string();
                assert!(msg.contains("collision") || msg.contains("__MACOSX"), "Error must report __MACOSX collision: {msg}");
            }
            Ok(_) => panic!("Must bail on regular file __MACOSX collision with AppleDouble zip"),
        }
    }

    #[crate::ctb_test]
    fn test_csc_netatalk_dot_appledouble_file_collision_bailout() {
        let temp = tempdir().expect("tempdir");
        let src = temp.path().join("src");
        let dest = temp.path().join("dest");
        let state = temp.path().join("state");
        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&state).unwrap();

        fs::write(src.join(".AppleDouble"), b"regular file not dir").unwrap();
        fs::write(src.join("foo.txt"), b"some content").unwrap();

        let mut args = default_test_args(
            vec![PathBuf::from(format!("{}/", src.display())), dest],
            state,
        );
        args.force_write_apple_double = Some(AppleDoubleStyle::Netatalk);

        match run_csc(args) {
            Err(err) => {
                let msg = err.to_string();
                assert!(msg.contains("collision") || msg.contains(".AppleDouble"), "Error must report .AppleDouble collision: {msg}");
            }
            Ok(_) => panic!("Must bail on regular file .AppleDouble collision with Netatalk"),
        }
    }

    #[crate::ctb_test]
    fn test_csc_netatalk_parent_file_collision_bailout() {
        let temp = tempdir().expect("tempdir");
        let src = temp.path().join("src");
        let dest = temp.path().join("dest");
        let state = temp.path().join("state");
        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&state).unwrap();

        fs::write(src.join(".Parent"), b"regular file named .Parent").unwrap();

        let mut args = default_test_args(
            vec![PathBuf::from(format!("{}/", src.display())), dest],
            state,
        );
        args.force_write_apple_double = Some(AppleDoubleStyle::Netatalk);

        match run_csc(args) {
            Err(err) => {
                let msg = err.to_string();
                assert!(msg.contains("collision") || msg.contains(".Parent"), "Error must report .Parent collision: {msg}");
            }
            Ok(_) => panic!("Must bail on regular file .Parent collision with Netatalk metadata"),
        }
    }

    #[crate::ctb_test]
    fn test_csc_foo_as_and_foo_as_as_target_collision_bailout() {
        let temp = tempdir().expect("tempdir");
        let src = temp.path().join("src");
        let dest = temp.path().join("dest");
        let state = temp.path().join("state");
        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&state).unwrap();

        // foo.as is a regular file (not AppleSingle)
        fs::write(src.join("foo.as"), b"regular actionscript").unwrap();

        // foo.as.as is AppleSingle
        let archive = ctb_io::file::AppleArchive {
            format: ctb_io::file::AppleFormat::AppleSingle,
            version: ctb_io::file::VERSION_2_0_BE,
            real_name: Some("foo.as".to_string()),
            comment: None,
            timestamps: None,
            backup_timestamp_sec: None,
            finder_info: None,
            extended_attributes: Vec::new(),
            data_fork: Some(b"nested applesingle".to_vec()),
            resource_fork: None,
            data_fork_size: Some(18),
            resource_fork_size: None,
            entries: Vec::new(),
        };
        let single_bytes = ctb_io::file::write_apple_single_double(&archive).unwrap();
        fs::write(src.join("foo.as.as"), single_bytes).unwrap();

        let mut args = default_test_args(
            vec![PathBuf::from(format!("{}/", src.display())), dest],
            state,
        );
        args.read_apple_single_with_extension = vec![ctb_io::file::AppleSingleExtension::As];

        // foo.as targets dest/foo.as; foo.as.as unpacks to dest/foo.as -> Collision!
        match run_csc(args) {
            Err(err) => {
                let msg = err.to_string();
                assert!(msg.contains("collision") || msg.contains("Target collision"), "Error must report collision: {msg}");
            }
            Ok(_) => panic!("Must bail on target collision between foo.as and foo.as.as"),
        }
    }

    #[crate::ctb_test]
    fn test_csc_as_and_asf_target_collision_bailout() {
        let temp = tempdir().expect("tempdir");
        let src = temp.path().join("src");
        let dest = temp.path().join("dest");
        let state = temp.path().join("state");
        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&state).unwrap();

        let archive = ctb_io::file::AppleArchive {
            format: ctb_io::file::AppleFormat::AppleSingle,
            version: ctb_io::file::VERSION_2_0_BE,
            real_name: Some("doc".to_string()),
            comment: None,
            timestamps: None,
            backup_timestamp_sec: None,
            finder_info: None,
            extended_attributes: Vec::new(),
            data_fork: Some(b"doc content".to_vec()),
            resource_fork: None,
            data_fork_size: Some(11),
            resource_fork_size: None,
            entries: Vec::new(),
        };
        let single_bytes = ctb_io::file::write_apple_single_double(&archive).unwrap();
        fs::write(src.join("doc.as"), &single_bytes).unwrap();
        fs::write(src.join("doc.asf"), &single_bytes).unwrap();

        let mut args = default_test_args(
            vec![PathBuf::from(format!("{}/", src.display())), dest],
            state,
        );
        args.read_apple_single_with_extension = vec![
            ctb_io::file::AppleSingleExtension::As,
            ctb_io::file::AppleSingleExtension::Asf,
        ];

        // Both doc.as and doc.asf strip to dest/doc -> Collision!
        match run_csc(args) {
            Err(err) => {
                let msg = err.to_string();
                assert!(msg.contains("collision") || msg.contains("Target collision"), "Error must report collision: {msg}");
            }
            Ok(_) => panic!("Must bail on target collision between doc.as and doc.asf"),
        }
    }

    #[crate::ctb_test]
    fn test_csc_double_dot_underscore_companion_collision_bailout() {
        let temp = tempdir().expect("tempdir");
        let src = temp.path().join("src");
        let dest = temp.path().join("dest");
        let state = temp.path().join("state");
        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&state).unwrap();

        // Independent ._foo and ._._foo files
        fs::write(src.join("._foo"), b"independent file 1").unwrap();
        fs::write(src.join("._._foo"), b"independent file 2").unwrap();

        let mut args = default_test_args(
            vec![PathBuf::from(format!("{}/", src.display())), dest],
            state,
        );
        // Turn off reading apple double alongside so both are treated as independent files
        args.no_read_apple_double_alongside = true;
        args.force_write_apple_double = Some(AppleDoubleStyle::Alongside);

        // Writing alongside for ._foo produces companion ._._foo, which collides with destination for ._._foo
        match run_csc(args) {
            Err(err) => {
                let msg = err.to_string();
                assert!(msg.contains("collision") || msg.contains("Companion collision"), "Error must report collision: {msg}");
            }
            Ok(_) => panic!("Must bail on companion collision for ._foo and ._._foo"),
        }
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_csc_continue_on_error_copies_remaining_and_records_errors_and_warnings() {
        if nix::unistd::geteuid().as_raw() == 0 {
            return;
        }
        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src");
        let dest = temp.path().join("dest");
        let state = temp.path().join("state");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&state).expect("create state");

        let readable_file = src.join("readable.txt");
        let unreadable_file = src.join("unreadable.txt");
        fs::write(&readable_file, b"readable content").expect("write readable");
        fs::write(&unreadable_file, b"unreadable content").expect("write unreadable");

        fs::set_permissions(&unreadable_file, fs::Permissions::from_mode(0o000)).expect("chmod 000");

        let mut args = default_test_args(
            vec![PathBuf::from(format!("{}/", src.display())), dest.clone()],
            state.clone(),
        );
        args.continue_on_error = true;

        let res = run_csc(args);
        let _ = fs::set_permissions(&unreadable_file, fs::Permissions::from_mode(0o644));

        let res = res.expect("run_csc with continue_on_error must succeed");
        match res {
            ctb_utilities::cli::ToolResult::Immediate { stdout, exit_code, .. } => {
                assert_eq!(exit_code, 0);
                let out = String::from_utf8_lossy(&stdout);
                assert!(out.contains("Errors recorded:"));
            }
            _ => panic!("Expected Immediate ToolResult"),
        }

        assert_eq!(
            fs::read(dest.join("readable.txt")).expect("read copied file"),
            b"readable content"
        );
        assert!(!dest.join("unreadable.txt").exists(), "unreadable file must not be copied");

        let journal_path = crate::journal::find_cscjournal(&state);
        let snapshot = crate::journal::read_journal_snapshot(&journal_path).expect("read journal snapshot");
        assert_eq!(snapshot.errors.len(), 1);
        let err = &snapshot.errors[0];
        assert_eq!(err.stage, crate::journal::JournalErrorStage::PayloadOpen);
        assert!(err.path.to_string_lossy().contains("unreadable.txt"));

        assert!(!snapshot.warnings.is_empty(), "Warnings must be recorded in journal");

        let desc_path = journal_path.with_extension("cscdesc");
        let desc_content = fs::read_to_string(&desc_path).expect("read desc");
        assert!(desc_content.contains("ErrorsCount: 1"), "desc must report ErrorsCount: 1: {desc_content}");
        assert!(desc_content.contains("unreadable.txt"), "desc must contain error path");
        assert!(desc_content.contains("WarningsCount:"), "desc must report WarningsCount");
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_csc_aborts_and_records_error_without_continue_on_error() {
        if nix::unistd::geteuid().as_raw() == 0 {
            return;
        }
        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src");
        let dest = temp.path().join("dest");
        let state = temp.path().join("state");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&state).expect("create state");

        let unreadable_file = src.join("bad.txt");
        fs::write(&unreadable_file, b"cannot read").expect("write bad");
        fs::set_permissions(&unreadable_file, fs::Permissions::from_mode(0o000)).expect("chmod 000");

        let mut args = default_test_args(
            vec![PathBuf::from(format!("{}/", src.display())), dest.clone()],
            state.clone(),
        );
        args.continue_on_error = false;

        let res = run_csc(args);
        let _ = fs::set_permissions(&unreadable_file, fs::Permissions::from_mode(0o644));

        assert!(res.is_err(), "Must abort on unreadable file when continue_on_error is false");

        let journal_path = crate::journal::find_cscjournal(&state);
        let snapshot = crate::journal::read_journal_snapshot(&journal_path).expect("read journal snapshot");
        assert_eq!(snapshot.errors.len(), 1);
        assert_eq!(snapshot.errors[0].stage, crate::journal::JournalErrorStage::PayloadOpen);

        let desc_path = journal_path.with_extension("cscdesc");
        let desc_content = fs::read_to_string(&desc_path).expect("read desc");
        assert!(desc_content.contains("Status: Failed:"), "desc must record Failed status: {desc_content}");
        assert!(desc_content.contains("ErrorsCount: 1"), "desc must record ErrorsCount: 1");
    }
}

