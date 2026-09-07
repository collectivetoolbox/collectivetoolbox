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
use crate::journal::{JournalSnapshot, JournalWriter, PLATFORM_WINDOWS};
use crate::path_resolution::ResolvedCopyTask;
use ctb_io::file::entity::{FileEntity, FileEntityKind};
use ctb_io::file::identity::{FileOrigin, resolve_relative_path_for_os};
use ctb_io::file::materializer::{
    MaterializeOptions, apply_entity_metadata, materialize_entity,
};
use ctb_io::file::path_policy::{PathTraversalPolicy, SymlinkValidationPolicy};
use ctb_io::file::payload::DiskPayloadSource;
use ctb_io::file::sandboxable_dir::SandboxableDir;
use ctb_io::file::streams::write_streams;
use ctb_io::file::verifier::{try_drop_system_caches, verify_materialized_entity};
use std::collections::HashMap;
use std::io::{Seek, SeekFrom};
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::path::{Path, PathBuf};
use std::time::Instant;

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
}

/// Deferred symlink to materialize in Pass 2.
struct DeferredSymlink {
    dest_path: PathBuf,
    entity: FileEntity,
    dest_dir_root: PathBuf,
}

/// Deferred directory metadata fixup item for Pass 3.
struct DeferredDirFixup {
    dest_path: PathBuf,
    dir_entity: FileEntity,
}

/// Runs the complete copy pipeline across all tasks using descriptor-safe, multi-pass copying.
pub fn execute_copy_pipeline(
    tasks: &[ResolvedCopyTask],
    args: &CscArgs,
    journal: &mut JournalWriter,
    snapshot: Option<&JournalSnapshot>,
    progress: &Progress,
) -> Result<CopyStats> {
    let mut stats = CopyStats::default();
    let mut hardlink_map: HashMap<(u64, u64), PathBuf> = HashMap::new();
    let mut deferred_dirs: Vec<DeferredDirFixup> = Vec::new();
    let mut deferred_symlinks: Vec<DeferredSymlink> = Vec::new();
    let mut files_to_verify: Vec<(PathBuf, PathBuf, FileEntity)> = Vec::new();

    // Restore hardlinks from snapshot if resuming (go through already-copied items and store their inodes in memory so we know we've already seen them)
    if let Some(snap) = snapshot {
        let is_windows = snap.origin_platform == PLATFORM_WINDOWS;
        for (raw_rel, entity) in &snap.committed_entities {
            if entity.identity.nlink > 1 {
                if let FileOrigin::Filesystem { key, .. } = &entity.identity.origin {
                    if let Ok(rel_path) = resolve_relative_path_for_os(raw_rel, is_windows) {
                        hardlink_map.entry((key.device_id, key.inode)).or_insert(rel_path);
                    }
                }
            }
        }
    }

    if !args.copy_specials_as_specials && !args.copy_block_devices_as_regular_files {
        progress.message(
            "Notice: Special files (FIFOs, device nodes, sockets) will be skipped by default. Pass --copy-specials-as-specials to preserve them.",
        );
    }

    let mut uncommitted_count: usize = 0;
    let mut last_progress_render = Instant::now();

    let options = MaterializeOptions {
        dry_run: args.dry_run,
        strict_lossless: true,
        symlink_policy: SymlinkValidationPolicy::PreserveVerbatim,
        path_policy: PathTraversalPolicy::StrictSandboxed,
        copy_specials: args.copy_specials_as_specials,
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

                if !args.dry_run {
                    dest_dir.ensure_dir_all(
                        &dir_entity.identity.relative_path,
                        options.path_policy,
                    )?;
                }
                stats.dirs_created = stats.dirs_created.saturating_add(1);

                deferred_dirs.push(DeferredDirFixup {
                    dest_path: curr_tgt.clone(),
                    dir_entity: dir_entity.clone(),
                });

                let mut journal_dir = dir_entity.clone();
                if let Ok(rel) = curr_tgt.strip_prefix(journal.destination()) {
                    journal_dir.identity.relative_path = rel.to_path_buf();
                    journal_dir.identity.raw_relative_path = rel.as_os_str().as_encoded_bytes().to_vec();
                }
                journal.record_entity(&journal_dir);

                let read_dir = std::fs::read_dir(&curr_src).with_context(|| {
                    format!("Failed to read source directory: {}", curr_src.display())
                })?;

                for entry in read_dir {
                    let entry = entry?;
                    let entry_src = entry.path();
                    let entry_name = entry.file_name();
                    let entry_tgt = curr_tgt.join(&entry_name);

                    let entry_sym_meta = std::fs::symlink_metadata(&entry_src)?;

                    if args.one_file_system && entry_sym_meta.dev() != src_meta.dev() {
                        continue;
                    }

                    if entry_sym_meta.is_dir() {
                        dir_queue.push((entry_src, entry_tgt));
                    } else if entry_sym_meta.is_symlink() {
                        let sym_entity =
                            FileEntity::from_filesystem(&entry_src, Some(src_root))?;
                        deferred_symlinks.push(DeferredSymlink {
                            dest_path: entry_tgt,
                            entity: sym_entity,
                            dest_dir_root: tgt_root.clone(),
                        });
                    } else {
                        copy_single_item(
                            &entry_src,
                            &entry_tgt,
                            src_root,
                            &dest_dir,
                            &options,
                            args,
                            journal,
                            snapshot,
                            &mut hardlink_map,
                            &mut stats,
                            &mut files_to_verify,
                        )?;

                        uncommitted_count = uncommitted_count.saturating_add(1);
                        if uncommitted_count >= 500 && !args.dry_run {
                            journal.commit_batch()?;
                            uncommitted_count = 0;
                        }

                        if progress.is_enabled()
                            && last_progress_render.elapsed().as_millis() > 100
                        {
                            progress.update_progress(
                                &format!(
                                    "[Copying] {} files ({} bytes)",
                                    stats.files_copied, stats.bytes_copied
                                ),
                                0.0,
                            );
                            last_progress_render = Instant::now();
                        }
                    }
                }
            }
        } else {
            // Reason for fallback: A single-component relative destination (e.g. "output.bin") has no parent path; falling back to current working directory "." correctly targets the local directory.
            let parent_dest = tgt_root.parent().unwrap_or(Path::new("."));
            let dest_dir = if args.dry_run {
                SandboxableDir::open(".").context("Failed to open current directory in dry run")?
            } else {
                SandboxableDir::create_or_open(parent_dest)?
            };

            if src_meta.is_symlink() {
                let sym_entity = FileEntity::from_filesystem(src_root, None)?;
                deferred_symlinks.push(DeferredSymlink {
                    dest_path: tgt_root.clone(),
                    entity: sym_entity,
                    dest_dir_root: parent_dest.to_path_buf(),
                });
            } else {
                copy_single_item(
                    src_root,
                    tgt_root,
                    src_root,
                    &dest_dir,
                    &options,
                    args,
                    journal,
                    snapshot,
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
        let mut entity = symlink_item.entity;

        if let Some(snap) = snapshot {
            let rel_bytes = if let Ok(rel) = dest_path.strip_prefix(journal.destination()) {
                rel.as_os_str().as_encoded_bytes()
            } else {
                dest_path.as_os_str().as_encoded_bytes()
            };
            if snap.is_committed(rel_bytes) {
                continue;
            }
        }

        let dest_dir = if args.dry_run {
            SandboxableDir::open(".").context("Failed to open current directory in dry run")?
        } else {
            SandboxableDir::create_or_open(&symlink_item.dest_dir_root)?
        };

        if let Ok(rel) = dest_path.strip_prefix(dest_dir.root_path()) {
            entity.identity.relative_path = rel.to_path_buf();
            entity.identity.raw_relative_path = rel.as_os_str().as_encoded_bytes().to_vec();
        }

        materialize_entity(&entity, None, &dest_dir, &options)?;

        if !args.dry_run {
            let dest_target = std::fs::read_link(dest_path)?;
            if let FileEntityKind::Symlink { target } = &entity.kind {
                anyhow::ensure!(
                    dest_target.as_os_str().as_encoded_bytes() == target.as_slice(),
                    "Target filesystem altered or normalized symlink target for {}",
                    dest_path.display()
                );
            }
        }

        stats.symlinks_created = stats.symlinks_created.saturating_add(1);
        let mut journal_symlink = entity.clone();
        if let Ok(rel) = dest_path.strip_prefix(journal.destination()) {
            journal_symlink.identity.relative_path = rel.to_path_buf();
            journal_symlink.identity.raw_relative_path = rel.as_os_str().as_encoded_bytes().to_vec();
        }
        journal.record_entity(&journal_symlink);
    }

    // =========================================================================
    // PASS 3: Apply directory metadata in reverse traversal order (leaf-first)
    // =========================================================================
    if !args.dry_run {
        while let Some(fixup) = deferred_dirs.pop() {
            if fixup.dest_path.exists() {
                if let Err(e) = write_streams(&fixup.dest_path, &fixup.dir_entity.streams, true) {
                    log_fmt!(
                        "Writing directory streams failed for {}: {e}",
                        fixup.dest_path.display()
                    );
                }
                if let Err(e) = apply_entity_metadata(
                    &fixup.dest_path,
                    &fixup.dir_entity.metadata,
                    false,
                    true,
                ) {
                    log_fmt!(
                        "Applying directory metadata failed for {}: {e}",
                        fixup.dest_path.display()
                    );
                }
            }
        }
        journal.commit_batch()?;
    }

    // =========================================================================
    // PASS 4: Post-flush independent verification pass
    // =========================================================================
    if args.should_verify_after() && !args.dry_run {
        progress.message("[Verifying] Flushing caches and verifying checksums...");
        try_drop_system_caches();

        for (_src_path, dest_path, entity) in &files_to_verify {
            verify_materialized_entity(dest_path, entity, true)?;
            stats.files_verified = stats.files_verified.saturating_add(1);

            if progress.is_enabled() && last_progress_render.elapsed().as_millis() > 100 {
                progress.update_progress(
                    &format!(
                        "[Verifying] {}/{} verified",
                        stats.files_verified,
                        files_to_verify.len()
                    ),
                    0.0,
                );
                last_progress_render = Instant::now();
            }
        }
    }

    if !args.dry_run {
        journal.mark_completed()?;
    }

    Ok(stats)
}

#[expect(
    clippy::too_many_arguments,
    reason = "Internal worker separating pipeline state from configuration"
)]
fn copy_single_item(
    src_path: &Path,
    dest_path: &Path,
    src_root: &Path,
    dest_dir: &SandboxableDir,
    options: &MaterializeOptions,
    args: &CscArgs,
    journal: &mut JournalWriter,
    snapshot: Option<&JournalSnapshot>,
    hardlink_map: &mut HashMap<(u64, u64), PathBuf>,
    stats: &mut CopyStats,
    files_to_verify: &mut Vec<(PathBuf, PathBuf, FileEntity)>,
) -> Result<()> {
    // 1. Check if already committed in snapshot
    if let Some(snap) = snapshot {
        // Reason for fallback: If dest_path is outside or equal to destination root, retain dest_path as relative lookup key.
        let rel_dest = dest_path
            .strip_prefix(journal.destination())
            .unwrap_or(dest_path);
        let rel_bytes = rel_dest.as_os_str().as_encoded_bytes();
        let dest_bytes = dest_path.as_os_str().as_encoded_bytes();
        if snap.is_committed(rel_bytes) || snap.is_committed(dest_bytes) {
            return Ok(());
        }
    }

    // 2. Discover full entity from filesystem
    let mut entity = FileEntity::from_filesystem(src_path, Some(src_root))?;
    if let Ok(rel) = dest_path.strip_prefix(dest_dir.root_path()) {
        entity.identity.relative_path = rel.to_path_buf();
        entity.identity.raw_relative_path = rel.as_os_str().as_encoded_bytes().to_vec();
    }

    // 3. Hardlink detection (nlink > 1)
    if entity.identity.nlink > 1 {
        let key = match &entity.identity.origin {
            FileOrigin::Filesystem { key, .. } => (key.device_id, key.inode),
            _ => (0, 0),
        };

        if let Some(first_target_rel) = hardlink_map.get(&key) {
            entity.kind = FileEntityKind::Hardlink {
                target_relative_path: first_target_rel.as_os_str().as_encoded_bytes().to_vec(),
            };
            materialize_entity(&entity, None, dest_dir, options)?;
            stats.hardlinks_created = stats.hardlinks_created.saturating_add(1);
            record_journal_entry(journal, dest_path, &entity);
            return Ok(());
        }

        hardlink_map.insert(key, entity.identity.relative_path.clone());
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
                let mut dev_file = std::fs::File::open(src_path).with_context(|| {
                    format!("Failed to open block device: {}", src_path.display())
                })?;
                let size = dev_file.seek(SeekFrom::End(0)).with_context(|| {
                    format!(
                        "Failed to determine size of block device: {}",
                        src_path.display()
                    )
                })?;
                dev_file.seek(SeekFrom::Start(0))?;
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
                record_journal_entry(journal, dest_path, &entity);
                return Ok(());
            } else {
                stats.special_files_skipped = stats.special_files_skipped.saturating_add(1);
                return Ok(());
            }
        }
        _ => {}
    }

    // 5. Regular files: Sparse support, in-flight SHA-256, atomic rename
    let captured_mtime = entity.metadata.timestamps.mtime_sec;
    let captured_ctime = entity.metadata.timestamps.ctime_sec;
    let initial_size = match &entity.kind {
        FileEntityKind::Regular { size, .. } => *size,
        _ => 0,
    };

    // Check pre-existing identical file if --skip-existing-checksum is enabled
    if args.skip_existing_checksum && dest_path.is_file() {
        if let Ok(dest_meta) = dest_path.metadata() {
            if dest_meta.len() == initial_size {
                if let Ok(dest_entity) = FileEntity::from_filesystem(dest_path, None) {
                    let hashes_match = match (&entity.kind, &dest_entity.kind) {
                        (
                            FileEntityKind::Regular { sha256: s, .. },
                            FileEntityKind::Regular { sha256: d, .. },
                        ) => s == d,
                        _ => false,
                    };

                    let streams_match = entity.streams.len() == dest_entity.streams.len()
                        && entity.streams.iter().all(|s| {
                            dest_entity
                                .streams
                                .iter()
                                .any(|d| d.name == s.name && d.entity.kind.kind_sha256() == s.entity.kind.kind_sha256())
                        });

                    if hashes_match && streams_match {
                        if !args.dry_run {
                            apply_entity_metadata(
                                dest_path,
                                &entity.metadata,
                                false,
                                options.strict_lossless,
                            )?;
                        }
                        stats.files_skipped_identical =
                            stats.files_skipped_identical.saturating_add(1);
                        record_journal_entry(journal, dest_path, &entity);
                        files_to_verify.push((
                            src_path.to_path_buf(),
                            dest_path.to_path_buf(),
                            entity,
                        ));
                        return Ok(());
                    }
                }
            }
        }
    }

    // Materialize payload
    let receipt = if args.dry_run {
        materialize_entity(&entity, None, dest_dir, options)?
    } else if matches!(entity.kind, FileEntityKind::Regular { .. }) {
        let mut payload = DiskPayloadSource::open(src_path)?;
        materialize_entity(&entity, Some(&mut payload), dest_dir, options)?
    } else {
        materialize_entity(&entity, None, dest_dir, options)?
    };

    // Verify source wasn't modified concurrently during copy
    let after_meta = std::fs::symlink_metadata(src_path)?;
    let is_block_device_as_regular = args.copy_block_devices_as_regular_files
        && after_meta.file_type().is_block_device();
    if !is_block_device_as_regular
        && (after_meta.mtime() != captured_mtime
            || after_meta.ctime() != captured_ctime
            || after_meta.size() != initial_size)
    {
        if args.on_source_change == SourceChangePolicy::Error {
            let _ = std::fs::remove_file(dest_path);
            anyhow::bail!(
                "Source file {} was modified concurrently during copy (mtime/ctime/size changed)",
                src_path.display()
            );
        }
        eprintln!(
            "WARNING: Source file {} changed during copy; proceeding best-effort.",
            src_path.display()
        );
    }

    stats.files_copied = stats.files_copied.saturating_add(1);
    stats.bytes_copied = stats.bytes_copied.saturating_add(receipt.bytes_written);

    record_journal_entry(journal, dest_path, &entity);
    files_to_verify.push((src_path.to_path_buf(), dest_path.to_path_buf(), entity));

    Ok(())
}

fn record_journal_entry(
    journal: &mut JournalWriter,
    dest_path: &Path,
    entity: &FileEntity,
) {
    let mut journal_entity = entity.clone();
    if let Ok(rel) = dest_path.strip_prefix(journal.destination()) {
        journal_entity.identity.relative_path = rel.to_path_buf();
        journal_entity.identity.raw_relative_path = rel.as_os_str().as_encoded_bytes().to_vec();
    }
    journal.record_entity(&journal_entity);
}

trait FileEntityKindExt {
    fn kind_sha256(&self) -> Option<[u8; 32]>;
}

impl FileEntityKindExt for FileEntityKind {
    fn kind_sha256(&self) -> Option<[u8; 32]> {
        match self {
            Self::Regular { sha256, .. } => Some(*sha256),
            _ => None,
        }
    }
}
