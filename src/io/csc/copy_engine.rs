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
use crate::fs_strict::{
    FileExtent, apply_metadata, get_file_extents, read_and_hash_streams,
    verify_filename_exact_bytes, write_streams,
};
use crate::journal::{
    JournalSnapshot, JournalWriter, ManifestDir, ManifestFile, ManifestSymlink,
};
use crate::path_resolution::ResolvedCopyTask;
use crate::verify_cache::verify_file_independent;
use ctb_formats_checksum::Sha256Stream;
use nix::sys::stat::{Mode, SFlag, mknod};
use nix::unistd::mkfifo;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
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
    pub files_verified: u64,
}

/// Deferred directory metadata fixup item.
struct DeferredDirFixup {
    dest_path: PathBuf,
    source_meta: std::fs::Metadata,
    streams: Vec<(crate::fs_strict::StreamInfo, Vec<u8>)>,
}

/// Runs the complete copy pipeline across all tasks.
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

    // Map of relative path -> (source_path, dest_path, manifest) for the post-flush verify pass
    let mut files_to_verify: Vec<(PathBuf, PathBuf, ManifestFile)> = Vec::new();

    // Restore hardlinks and files from snapshot if resuming
    if let Some(snap) = snapshot {
        for tgt_rel in snap.committed_hardlinks.values() {
            let full_tgt = snap.destination.join(tgt_rel);
            if let Ok(meta) = full_tgt.metadata() {
                hardlink_map.insert((meta.dev(), meta.ino()), full_tgt);
            }
        }
    }

    let mut uncommitted_count: usize = 0;
    let mut last_progress_render = Instant::now();

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
            // Traverse directory
            let mut dir_queue: Vec<(PathBuf, PathBuf)> = vec![(src_root.clone(), tgt_root.clone())];

            while let Some((curr_src, curr_tgt)) = dir_queue.pop() {
                if !curr_tgt.exists() && !args.dry_run {
                    std::fs::create_dir_all(&curr_tgt).with_context(|| {
                        format!("Failed to create destination dir: {}", curr_tgt.display())
                    })?;
                }

                let src_meta = std::fs::metadata(&curr_src)?;
                let streams = read_and_hash_streams(&curr_src)?;
                deferred_dirs.push(DeferredDirFixup {
                    dest_path: curr_tgt.clone(),
                    source_meta: src_meta.clone(),
                    streams: streams.clone(),
                });

                let dir_mtime = src_meta.mtime();
                let dir_mtime_nsec = u32::try_from(src_meta.mtime_nsec())
                    .context("Failed to convert directory mtime nanoseconds to u32")?;
                journal.record_dir(ManifestDir {
                    relative_path: curr_tgt.clone(),
                    mode: src_meta.mode(),
                    mtime_sec: dir_mtime,
                    mtime_nsec: dir_mtime_nsec,
                    uid: src_meta.uid(),
                    gid: src_meta.gid(),
                    streams: streams.iter().map(|(s, _)| (s.name.clone(), s.sha256)).collect(),
                });

                let read_dir = std::fs::read_dir(&curr_src).with_context(|| {
                    format!("Failed to read source directory: {}", curr_src.display())
                })?;

                for entry in read_dir {
                    let entry = entry?;
                    let entry_src = entry.path();
                    let entry_name = entry.file_name();
                    let entry_tgt = curr_tgt.join(&entry_name);

                    let entry_sym_meta = std::fs::symlink_metadata(&entry_src)?;

                    if entry_sym_meta.is_dir() {
                        dir_queue.push((entry_src, entry_tgt));
                    } else {
                        copy_single_item(
                            &entry_src,
                            &entry_tgt,
                            &entry_sym_meta,
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
                                &format!("[Copying] {} files ({} bytes)", stats.files_copied, stats.bytes_copied),
                                0.0,
                            );
                            last_progress_render = Instant::now();
                        }
                    }
                }
            }
        } else {
            // Single file / node copy
            if let Some(parent) = tgt_root.parent() {
                if !parent.exists() && !args.dry_run {
                    std::fs::create_dir_all(parent)?;
                }
            }
            copy_single_item(
                src_root,
                tgt_root,
                &src_meta,
                args,
                journal,
                snapshot,
                &mut hardlink_map,
                &mut stats,
                &mut files_to_verify,
            )?;
        }
    }

    // Fix up all deferred directory permissions and timestamps in reverse order (leaves first)
    if !args.dry_run {
        while let Some(fixup) = deferred_dirs.pop() {
            if fixup.dest_path.exists() {
                if let Err(e) = write_streams(&fixup.dest_path, &fixup.streams) {
                    log_fmt!("Writing directory streams failed for {}: {e}", fixup.dest_path.display());
                }
                if let Err(e) = apply_metadata(&fixup.dest_path, &fixup.source_meta, false) {
                    log_fmt!("Applying directory metadata failed for {}: {e}", fixup.dest_path.display());
                }
            }
        }
        journal.commit_batch()?;
    }

    // Post-flush verification pass
    if args.should_verify_after() && !args.dry_run {
        progress.message("[Verifying] Flushing caches and verifying checksums...");
        crate::verify_cache::try_drop_system_caches();

        for (src_path, dest_path, manifest) in &files_to_verify {
            verify_file_independent(src_path, dest_path, manifest)?;
            stats.files_verified = stats.files_verified.saturating_add(1);

            if progress.is_enabled() && last_progress_render.elapsed().as_millis() > 100 {
                progress.update_progress(
                    &format!("[Verifying] {}/{} verified", stats.files_verified, files_to_verify.len()),
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

fn remove_if_exists(path: &Path) {
    if path.exists() || path.is_symlink() {
        if let Err(e) = std::fs::remove_file(path) {
            log_fmt!("Could not remove existing file at {}: {e}", path.display());
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Internal copy engine worker separating engine parameters"
)]
fn copy_single_item(
    src_path: &Path,
    dest_path: &Path,
    sym_meta: &std::fs::Metadata,
    args: &CscArgs,
    journal: &mut JournalWriter,
    snapshot: Option<&JournalSnapshot>,
    hardlink_map: &mut HashMap<(u64, u64), PathBuf>,
    stats: &mut CopyStats,
    files_to_verify: &mut Vec<(PathBuf, PathBuf, ManifestFile)>,
) -> Result<()> {
    // 1. Check if already committed in snapshot
    if let Some(snap) = snapshot {
        if snap.committed_files.contains_key(dest_path)
            || snap.committed_symlinks.contains_key(dest_path)
            || snap.committed_hardlinks.contains_key(dest_path)
        {
            return Ok(());
        }
    }

    let file_type = sym_meta.file_type();

    // 2. Symlinks
    if file_type.is_symlink() {
        let target = std::fs::read_link(src_path)?;
        if !args.dry_run {
            remove_if_exists(dest_path);
            std::os::unix::fs::symlink(&target, dest_path).with_context(|| {
                format!("Failed to create symlink: {}", dest_path.display())
            })?;
            apply_metadata(dest_path, sym_meta, true)?;
        }
        stats.symlinks_created = stats.symlinks_created.saturating_add(1);
        journal.record_symlink(ManifestSymlink {
            relative_path: dest_path.to_path_buf(),
            target: target.as_os_str().as_encoded_bytes().to_vec(),
            mtime_sec: sym_meta.mtime(),
            mtime_nsec: u32::try_from(sym_meta.mtime_nsec())
                .context("Failed to convert symlink mtime nanoseconds to u32")?,
            uid: sym_meta.uid(),
            gid: sym_meta.gid(),
        });
        return Ok(());
    }

    // 3. Hardlink detection (st_nlink > 1)
    if sym_meta.nlink() > 1 {
        let key = (sym_meta.dev(), sym_meta.ino());
        if let Some(first_target) = hardlink_map.get(&key) {
            if !args.dry_run {
                remove_if_exists(dest_path);
                std::fs::hard_link(first_target, dest_path).with_context(|| {
                    format!(
                        "Failed to create hardlink from {} to {}",
                        first_target.display(),
                        dest_path.display()
                    )
                })?;
            }
            stats.hardlinks_created = stats.hardlinks_created.saturating_add(1);
            journal.record_hardlink(dest_path.to_path_buf(), first_target.clone());
            return Ok(());
        }
        // First time seeing this inode: record for future hardlinks
        hardlink_map.insert(key, dest_path.to_path_buf());
    }

    // 4. Special files (FIFOs, device nodes, sockets)
    if file_type.is_fifo() {
        if !args.dry_run {
            remove_if_exists(dest_path);
            mkfifo(dest_path, Mode::from_bits_truncate(sym_meta.mode())).with_context(|| {
                format!("Failed to create FIFO: {}", dest_path.display())
            })?;
            apply_metadata(dest_path, sym_meta, false)?;
        }
        stats.special_files_created = stats.special_files_created.saturating_add(1);
        return Ok(());
    }

    if file_type.is_char_device() {
        if !args.dry_run {
            remove_if_exists(dest_path);
            mknod(
                dest_path,
                SFlag::S_IFCHR,
                Mode::from_bits_truncate(sym_meta.mode()),
                sym_meta.rdev(),
            )
            .with_context(|| {
                format!("Failed to create character device node: {}", dest_path.display())
            })?;
            apply_metadata(dest_path, sym_meta, false)?;
        }
        stats.special_files_created = stats.special_files_created.saturating_add(1);
        return Ok(());
    }

    if file_type.is_block_device() {
        anyhow::ensure!(
            args.copy_block_devices,
            "Encountered block device {}. Pass --copy-block-devices to permit copying block device nodes.",
            src_path.display()
        );
        if !args.dry_run {
            remove_if_exists(dest_path);
            mknod(
                dest_path,
                SFlag::S_IFBLK,
                Mode::from_bits_truncate(sym_meta.mode()),
                sym_meta.rdev(),
            )
            .with_context(|| {
                format!("Failed to create block device node: {}", dest_path.display())
            })?;
            apply_metadata(dest_path, sym_meta, false)?;
        }
        stats.special_files_created = stats.special_files_created.saturating_add(1);
        return Ok(());
    }

    if file_type.is_socket() {
        anyhow::bail!(
            "Cannot copy live UNIX socket node: {}. Sockets cannot be cloned across directories.",
            src_path.display()
        );
    }

    // 5. Regular file: Atomic temp write with sparse extents & SHA-256
    let captured_mtime = sym_meta.mtime();
    let captured_ctime = sym_meta.ctime();
    let initial_size = sym_meta.size();

    // Check pre-existing file if --skip-existing-checksum is active
    if args.skip_existing_checksum && dest_path.is_file() {
        if let Ok(dest_meta) = dest_path.metadata() {
            if dest_meta.len() == initial_size {
                let src_streams = read_and_hash_streams(src_path)?;
                let dest_streams = read_and_hash_streams(dest_path)?;

                let mut src_hasher = Sha256Stream::new();
                let mut f_src = File::open(src_path)?;
                let mut buf = vec![0_u8; 64 * 1024];
                while let Ok(n) = f_src.read(&mut buf) {
                    if n == 0 {
                        break;
                    }
                    let slice = buf
                        .get(..n)
                        .context("Source read buffer slice index out of bounds")?;
                    src_hasher.update(slice);
                }
                let src_hash = src_hasher.finalize();

                let mut dest_hasher = Sha256Stream::new();
                let mut f_dest = File::open(dest_path)?;
                while let Ok(n) = f_dest.read(&mut buf) {
                    if n == 0 {
                        break;
                    }
                    let slice = buf
                        .get(..n)
                        .context("Destination read buffer slice index out of bounds")?;
                    dest_hasher.update(slice);
                }
                let dest_hash = dest_hasher.finalize();

                let streams_match = src_streams.len() == dest_streams.len()
                    && src_streams.iter().all(|(s, _)| {
                        dest_streams.iter().any(|(d, _)| d.name == s.name && d.sha256 == s.sha256)
                    });

                if src_hash == dest_hash && streams_match {
                    // Pre-existing file is identical! Sync metadata and skip copy
                    if !args.dry_run {
                        apply_metadata(dest_path, sym_meta, false)?;
                    }
                    stats.files_skipped_identical = stats.files_skipped_identical.saturating_add(1);

                    let manifest = ManifestFile {
                        relative_path: dest_path.to_path_buf(),
                        size: initial_size,
                        sha256: src_hash,
                        mtime_sec: captured_mtime,
                        mtime_nsec: u32::try_from(sym_meta.mtime_nsec())
                            .context("Failed to convert file mtime nanoseconds to u32")?,
                        mode: sym_meta.mode(),
                        uid: sym_meta.uid(),
                        gid: sym_meta.gid(),
                        is_sparse: false,
                        streams: src_streams.into_iter().map(|(s, _)| (s.name, s.sha256)).collect(),
                    };
                    journal.record_file(manifest.clone());
                    files_to_verify.push((src_path.to_path_buf(), dest_path.to_path_buf(), manifest));
                    return Ok(());
                }
            }
        }
    }

    if args.dry_run {
        stats.files_copied = stats.files_copied.saturating_add(1);
        stats.bytes_copied = stats.bytes_copied.saturating_add(initial_size);
        return Ok(());
    }

    // Atomic temp file creation in destination parent directory
    let parent_dir = dest_path.parent().context("Dest path has no parent")?;
    let file_name = dest_path.file_name().context("Dest path has no file name")?;

    let pid = std::process::id();
    let thread_id = std::thread::current().id();
    let nanos = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(dur) => dur.subsec_nanos(),
        Err(_) => 0,
    };
    let temp_name = format!(".csc-tmp.{pid}.{thread_id:?}.{nanos}.{}", file_name.to_string_lossy());
    let temp_path = parent_dir.join(temp_name);

    let mut temp_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp_path)
        .with_context(|| format!("Failed to create atomic temp file: {}", temp_path.display()))?;

    let mut src_file = File::open(src_path).with_context(|| {
        format!("Failed to open source file: {}", src_path.display())
    })?;

    let mut hasher = Sha256Stream::new();
    let extents = get_file_extents(&src_file, initial_size)?;
    let is_sparse = extents.iter().any(|e| matches!(e, FileExtent::Hole { .. }));
    src_file.seek(SeekFrom::Start(0))?;

    if is_sparse {
        // Copy sparse extents preserving holes
        for extent in extents {
            match extent {
                FileExtent::Data { offset, length } => {
                    src_file.seek(SeekFrom::Start(offset))?;
                    temp_file.seek(SeekFrom::Start(offset))?;

                    let mut remaining = length;
                    let mut buf = vec![0_u8; 64 * 1024];
                    while remaining > 0 {
                        let to_read = usize::try_from(remaining.min(64 * 1024))
                            .context("Failed to convert buffer slice length to usize")?;
                        let buf_slice = buf
                            .get_mut(..to_read)
                            .context("Buffer slice index out of bounds for read")?;
                        let n = src_file.read(buf_slice)?;
                        if n == 0 {
                            break;
                        }
                        let write_slice = buf
                            .get(..n)
                            .context("Buffer slice index out of bounds for write")?;
                        temp_file.write_all(write_slice)?;
                        hasher.update(write_slice);
                        let n_u64 = u64::try_from(n)
                            .context("Failed to convert read bytes count to u64")?;
                        remaining = remaining.saturating_sub(n_u64);
                    }
                }
                FileExtent::Hole { length, .. } => {
                    // Holes hash as zeroes
                    let zero_buf = [0_u8; 8 * 1024];
                    let mut remaining = length;
                    while remaining > 0 {
                        let chunk = usize::try_from(remaining.min(8 * 1024))
                            .context("Failed to convert hole chunk size to usize")?;
                        let zero_slice = zero_buf
                            .get(..chunk)
                            .context("Zero buffer slice index out of bounds")?;
                        hasher.update(zero_slice);
                        let chunk_u64 = u64::try_from(chunk)
                            .context("Failed to convert hole chunk size to u64")?;
                        remaining = remaining.saturating_sub(chunk_u64);
                    }
                }
            }
        }
        temp_file.set_len(initial_size)?;
    } else {
        // Standard contiguous copy
        let mut buf = vec![0_u8; 64 * 1024];
        loop {
            let n = src_file.read(&mut buf)?;
            if n == 0 {
                break;
            }
            let slice = buf
                .get(..n)
                .context("Buffer slice index out of bounds for write")?;
            temp_file.write_all(slice)?;
            hasher.update(slice);
        }
    }

    let file_sha256 = hasher.finalize();

    // Copy streams, xattrs, and ACLs
    let streams = read_and_hash_streams(src_path)?;
    write_streams(&temp_path, &streams)?;

    // Apply metadata
    apply_metadata(&temp_path, sym_meta, false)?;

    // Sync temp file and parent directory before atomic rename
    temp_file.sync_data()?;
    drop(temp_file);

    let parent_fd = File::open(parent_dir)?;
    parent_fd.sync_data()?;

    // Atomic rename
    std::fs::rename(&temp_path, dest_path).with_context(|| {
        format!("Failed to rename temp file to final dest: {}", dest_path.display())
    })?;
    parent_fd.sync_data()?;

    // Verify raw bytes in destination directory
    verify_filename_exact_bytes(parent_dir, file_name.as_encoded_bytes())?;

    // Check source file modification during copy
    let after_meta = std::fs::symlink_metadata(src_path)?;
    if after_meta.mtime() != captured_mtime
        || after_meta.ctime() != captured_ctime
        || after_meta.size() != initial_size
    {
        if args.on_source_change == SourceChangePolicy::Error {
            remove_if_exists(dest_path);
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
    stats.bytes_copied = stats.bytes_copied.saturating_add(initial_size);

    let manifest = ManifestFile {
        relative_path: dest_path.to_path_buf(),
        size: initial_size,
        sha256: file_sha256,
        mtime_sec: captured_mtime,
        mtime_nsec: u32::try_from(sym_meta.mtime_nsec())
            .context("Failed to convert file mtime nanoseconds to u32")?,
        mode: sym_meta.mode(),
        uid: sym_meta.uid(),
        gid: sym_meta.gid(),
        is_sparse,
        streams: streams.into_iter().map(|(s, _)| (s.name, s.sha256)).collect(),
    };

    journal.record_file(manifest.clone());
    files_to_verify.push((src_path.to_path_buf(), dest_path.to_path_buf(), manifest));

    Ok(())
}
