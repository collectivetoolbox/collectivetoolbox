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
use crate::journal::JournalWriter;
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

/// Summary stats from a copy run.
#[derive(Debug, Clone, Default)]
pub struct CopyStats {
    pub files_copied: u64,
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
    }

    let mut uncommitted_count: usize = 0;
    let copy_task = progress.start_task("Copying", None);

    let strict_lossless = !args.best_effort_metadata && !args.allow_unknown_fs;
    let options = MaterializeOptions {
        dry_run: args.dry_run,
        strict_lossless,
        symlink_policy: SymlinkValidationPolicy::PreserveVerbatim,
        path_policy: PathTraversalPolicy::StrictSandboxed,
        copy_specials: args.copy_specials_as_specials,
        force_overwrite: args.always_overwrite,
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
                let dir_entity = FileEntity::from_filesystem(&curr_src, Some(src_root))?;
                validate_filesystem_known(&curr_src, dir_entity.metadata.filesystem_type.as_deref(), args)?;

                if !args.dry_run {
                    dest_dir.ensure_dir_all(
                        &dir_entity.identity.relative_path,
                        options.path_policy,
                    )?;
                }
                stats.dirs_created = stats.dirs_created.saturating_add(1);

                let mut journal_dir = dir_entity.clone();
                let rel = compute_journal_relative_path(journal.destination(), &curr_tgt);
                journal_dir.identity.relative_path = rel.clone();
                journal_dir.identity.raw_relative_path = rel.as_os_str().as_encoded_bytes().to_vec();
                journal.record_entity(&journal_dir);
                if !args.dry_run {
                    journal.commit_batch()?;
                }

                let traversal_opts = ctb_io::file::TraversalOptions::new()
                    .one_file_system(args.one_file_system)
                    .error_policy(ctb_io::file::OnTraversalError::Bail);

                let dir_entries = ctb_io::file::read_dir_safe(&curr_src, &traversal_opts)?;

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
                        expected_filenames.push(entry_name.as_encoded_bytes().to_vec());
                        let mut sym_entity =
                            FileEntity::from_filesystem(&entry_src, Some(src_root))?;
                        validate_filesystem_known(&entry_src, sym_entity.metadata.filesystem_type.as_deref(), args)?;
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
                        let created = copy_single_item(
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
                        if created {
                            expected_filenames.push(entry_name.as_encoded_bytes().to_vec());
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
                let mut sym_entity = FileEntity::from_filesystem(src_root, None)?;
                validate_filesystem_known(src_root, sym_entity.metadata.filesystem_type.as_deref(), args)?;
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

        materialize_entity(&entity, None, &dest_dir, &options)?;

        if !args.dry_run {
            let dest_target = std::fs::read_link(dest_path)?;
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
                anyhow::ensure!(
                    dest_target.as_os_str().as_encoded_bytes() == target.as_slice(),
                    "Target filesystem altered or normalized symlink target for {}",
                    dest_path.display()
                );
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
                verify_directory_filenames_exact(&fixup.dest_path, &expected_refs)?;

                write_streams(&fixup.dest_path, None, &fixup.dir_entity.streams, strict_lossless)?;
                apply_entity_metadata(
                    &fixup.dest_path,
                    None,
                    &fixup.dir_entity.metadata,
                    false,
                    true,
                    strict_lossless,
                )?;
                verify_materialized_entity_ext(
                    &fixup.dest_path,
                    &fixup.dir_entity,
                    strict_lossless,
                    args.should_check_atime(),
                    args.should_check_ctime(),
                )?;
                files_to_verify.push((fixup.src_path, fixup.dest_path, fixup.dir_entity));
            } else {
                anyhow::bail!("Destination directory disappeared: {}", fixup.dest_path.display());
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
            verify_materialized_entity_ext(
                src_path,
                entity,
                !args.best_effort_metadata,
                args.should_check_atime(),
                args.should_check_ctime(),
            )?;
            verify_materialized_entity_ext(
                dest_path,
                entity,
                strict_lossless,
                args.should_check_atime(),
                args.should_check_ctime(),
            )?;
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
) -> Result<bool> {
    // 2. Discover full entity from filesystem
    let mut entity = FileEntity::from_filesystem(src_path, None)?;
    validate_filesystem_known(src_path, entity.metadata.filesystem_type.as_deref(), args)?;
    entity.identity.relative_path = dest_rel_path.to_path_buf();
    entity.identity.raw_relative_path = dest_rel_path.as_os_str().as_encoded_bytes().to_vec();
    if let Some(fname) = dest_rel_path.file_name() {
        entity.identity.raw_filename = fname.as_encoded_bytes().to_vec();
    }
    if !args.dry_run {
        record_journal_entry(journal, dest_path, &entity)?;
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
            let is_self = dest_path == first_target_rel
                || this_full_path == *first_target_rel
                || std::fs::canonicalize(dest_path).ok().as_deref()
                    == std::fs::canonicalize(first_target_rel).ok().as_deref();
            if !is_self {
                let original_entity = entity.clone();
                entity.kind = FileEntityKind::Hardlink {
                    target_relative_path: first_target_rel.as_os_str().as_encoded_bytes().to_vec(),
                };
                materialize_entity(&entity, None, dest_dir, options)?;
                stats.hardlinks_created = stats.hardlinks_created.saturating_add(1);
                record_journal_entry(journal, dest_path, &entity)?;
                files_to_verify.push((src_path.to_path_buf(), dest_path.to_path_buf(), original_entity));
                return Ok(true);
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
                let dev_file = std::fs::File::open(src_path).with_context(|| {
                    format!("Failed to open block device: {}", src_path.display())
                })?;
                let size = query_block_device_size(&dev_file).with_context(|| {
                    format!(
                        "Failed to determine size of block device: {}",
                        src_path.display()
                    )
                })?;
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
                materialize_entity(&entity, None, dest_dir, options)?;
                stats.special_files_created = stats.special_files_created.saturating_add(1);
                record_journal_entry(journal, dest_path, &entity)?;
                files_to_verify.push((src_path.to_path_buf(), dest_path.to_path_buf(), entity));
                return Ok(true);
            } else {
                stats.special_files_skipped = stats.special_files_skipped.saturating_add(1);
                return Ok(false);
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
    let receipt = if args.dry_run {
        materialize_entity(&entity, None, dest_dir, options)
    } else if matches!(entity.kind, FileEntityKind::Regular { .. }) {
        let mut payload = DiskPayloadSource::open(src_path)?;
        materialize_entity(&entity, Some(&mut payload), dest_dir, options)
    } else {
        materialize_entity(&entity, None, dest_dir, options)
    }
    .with_context(|| {
        format!(
            "copying '{}' -> '{}'",
            src_path.display(),
            dest_path.display()
        )
    })?;

    // Verify source wasn't modified concurrently during copy
    let after_meta = std::fs::symlink_metadata(src_path)?;
    #[cfg(unix)]
    let is_block_device_as_regular = args.copy_block_devices_as_regular_files
        && after_meta.file_type().is_block_device();
    #[cfg(not(unix))]
    let is_block_device_as_regular = false;

    #[cfg(unix)]
    let changed = after_meta.mtime() != captured_mtime
        || after_meta.ctime() != captured_ctime
        || after_meta.mtime_nsec() != i64::from(entity.metadata.timestamps.mtime_nsec)
        || after_meta.ctime_nsec() != i64::from(entity.metadata.timestamps.ctime_nsec)
        || after_meta.len() != initial_size;
    #[cfg(not(unix))]
    let changed = after_meta.len() != initial_size
        || filetime::FileTime::from_last_modification_time(&after_meta)
            != filetime::FileTime::from_unix_time(entity.metadata.timestamps.mtime_sec, entity.metadata.timestamps.mtime_nsec);

    if !is_block_device_as_regular && changed {
        if args.on_source_change == SourceChangePolicy::Error {
            anyhow::bail!(
                "Source file {} was modified concurrently during copy (timestamps or size changed)",
                src_path.display()
            );
        }
        eprintln!(
            "WARNING: Source file {} changed during copy; proceeding best-effort.",
            src_path.display()
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

    record_journal_entry(journal, dest_path, &entity)?;
    files_to_verify.push((src_path.to_path_buf(), dest_path.to_path_buf(), entity));

    Ok(true)
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


