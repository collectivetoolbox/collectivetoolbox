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

//! Transactional, crash-safe state journaling and manifest recording.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use ctb_io::file::entity::{FileEntity, FileEntityKind};
use ctb_io::file::metadata::FileFlag;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::os::unix::ffi::OsStringExt;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const JOURNAL_MAGIC: &[u8; 8] = b"CTBCSC02";

const TAG_SESSION_HEADER: u8 = 1;
const TAG_DIR: u8 = 2;
const TAG_FILE: u8 = 3;
const TAG_SYMLINK: u8 = 4;
const _TAG_SPECIAL: u8 = 5;
const TAG_HARDLINK: u8 = 6;
const TAG_BATCH_COMMIT: u8 = 7;
const TAG_JOB_COMPLETED: u8 = 8;

/// Manifest information for a committed regular file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestFile {
    pub relative_path: PathBuf,
    pub size: u64,
    pub sha256: [u8; 32],
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub mtime_sec: i64,
    pub mtime_nsec: u32,
    pub atime_sec: i64,
    pub atime_nsec: u32,
    pub ctime_sec: i64,
    pub ctime_nsec: u32,
    pub birthtime_sec: Option<i64>,
    pub birthtime_nsec: Option<u32>,
    pub flags: Vec<FileFlag>,
    pub is_sparse: bool,
    pub streams: Vec<(OsString, [u8; 32])>,
}

/// Manifest information for a committed directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestDir {
    pub relative_path: PathBuf,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub mtime_sec: i64,
    pub mtime_nsec: u32,
    pub atime_sec: i64,
    pub atime_nsec: u32,
    pub ctime_sec: i64,
    pub ctime_nsec: u32,
    pub birthtime_sec: Option<i64>,
    pub birthtime_nsec: Option<u32>,
    pub flags: Vec<FileFlag>,
    pub streams: Vec<(OsString, [u8; 32])>,
}

/// Manifest information for a committed symlink.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestSymlink {
    pub relative_path: PathBuf,
    pub target: Vec<u8>,
    pub uid: u32,
    pub gid: u32,
    pub mtime_sec: i64,
    pub mtime_nsec: u32,
    pub atime_sec: i64,
    pub atime_nsec: u32,
    pub ctime_sec: i64,
    pub ctime_nsec: u32,
    pub birthtime_sec: Option<i64>,
    pub birthtime_nsec: Option<u32>,
    pub flags: Vec<FileFlag>,
}

impl ManifestFile {
    /// Constructs a `ManifestFile` from a `FileEntity`.
    #[must_use]
    pub fn from_entity(entity: &FileEntity) -> Option<Self> {
        match &entity.kind {
            FileEntityKind::Regular {
                size,
                sha256,
                is_sparse,
                ..
            } => {
                let streams = entity
                    .streams
                    .iter()
                    .map(|s| {
                        let hash = match &s.entity.kind {
                            FileEntityKind::Regular { sha256, .. } => *sha256,
                            _ => [0_u8; 32],
                        };
                        (OsString::from_vec(s.name.0.clone()), hash)
                    })
                    .collect();
                Some(Self {
                    relative_path: entity.identity.relative_path.clone(),
                    size: *size,
                    sha256: *sha256,
                    mode: entity.metadata.mode,
                    uid: entity.metadata.uid,
                    gid: entity.metadata.gid,
                    mtime_sec: entity.metadata.timestamps.mtime_sec,
                    mtime_nsec: entity.metadata.timestamps.mtime_nsec,
                    atime_sec: entity.metadata.timestamps.atime_sec,
                    atime_nsec: entity.metadata.timestamps.atime_nsec,
                    ctime_sec: entity.metadata.timestamps.ctime_sec,
                    ctime_nsec: entity.metadata.timestamps.ctime_nsec,
                    birthtime_sec: entity.metadata.timestamps.birthtime_sec,
                    birthtime_nsec: entity.metadata.timestamps.birthtime_nsec,
                    flags: entity.metadata.flags.clone(),
                    is_sparse: *is_sparse,
                    streams,
                })
            }
            _ => None,
        }
    }
}

impl ManifestDir {
    /// Constructs a `ManifestDir` from a `FileEntity`.
    #[must_use]
    pub fn from_entity(entity: &FileEntity) -> Self {
        let streams = entity
            .streams
            .iter()
            .map(|s| {
                let hash = match &s.entity.kind {
                    FileEntityKind::Regular { sha256, .. } => *sha256,
                    _ => [0_u8; 32],
                };
                (OsString::from_vec(s.name.0.clone()), hash)
            })
            .collect();
        Self {
            relative_path: entity.identity.relative_path.clone(),
            mode: entity.metadata.mode,
            uid: entity.metadata.uid,
            gid: entity.metadata.gid,
            mtime_sec: entity.metadata.timestamps.mtime_sec,
            mtime_nsec: entity.metadata.timestamps.mtime_nsec,
            atime_sec: entity.metadata.timestamps.atime_sec,
            atime_nsec: entity.metadata.timestamps.atime_nsec,
            ctime_sec: entity.metadata.timestamps.ctime_sec,
            ctime_nsec: entity.metadata.timestamps.ctime_nsec,
            birthtime_sec: entity.metadata.timestamps.birthtime_sec,
            birthtime_nsec: entity.metadata.timestamps.birthtime_nsec,
            flags: entity.metadata.flags.clone(),
            streams,
        }
    }
}

impl ManifestSymlink {
    /// Constructs a `ManifestSymlink` from a `FileEntity`.
    #[must_use]
    pub fn from_entity(entity: &FileEntity) -> Option<Self> {
        match &entity.kind {
            FileEntityKind::Symlink { target } => Some(Self {
                relative_path: entity.identity.relative_path.clone(),
                target: target.clone(),
                uid: entity.metadata.uid,
                gid: entity.metadata.gid,
                mtime_sec: entity.metadata.timestamps.mtime_sec,
                mtime_nsec: entity.metadata.timestamps.mtime_nsec,
                atime_sec: entity.metadata.timestamps.atime_sec,
                atime_nsec: entity.metadata.timestamps.atime_nsec,
                ctime_sec: entity.metadata.timestamps.ctime_sec,
                ctime_nsec: entity.metadata.timestamps.ctime_nsec,
                birthtime_sec: entity.metadata.timestamps.birthtime_sec,
                birthtime_nsec: entity.metadata.timestamps.birthtime_nsec,
                flags: entity.metadata.flags.clone(),
            }),
            _ => None,
        }
    }
}

/// Summary of an active or completed journal recovered from disk.
#[derive(Debug, Clone)]
pub struct JournalSnapshot {
    pub sources: Vec<PathBuf>,
    pub destination: PathBuf,
    pub committed_files: HashMap<PathBuf, ManifestFile>,
    pub committed_dirs: HashMap<PathBuf, ManifestDir>,
    pub committed_symlinks: HashMap<PathBuf, ManifestSymlink>,
    pub committed_hardlinks: HashMap<PathBuf, PathBuf>,
    pub is_completed: bool,
    pub last_batch_id: u64,
}

/// Writer managing the persistent state journal and its companion `.cscdesc` file.
pub struct JournalWriter {
    journal_path: PathBuf,
    desc_path: PathBuf,
    writer: BufWriter<File>,
    sources: Vec<PathBuf>,
    destination: PathBuf,
    current_batch_id: u64,
    uncommitted_files: Vec<ManifestFile>,
    uncommitted_dirs: Vec<ManifestDir>,
    uncommitted_symlinks: Vec<ManifestSymlink>,
    uncommitted_hardlinks: Vec<(PathBuf, PathBuf)>,
    total_committed_files: u64,
    total_committed_bytes: u64,
}

impl JournalWriter {
    /// Creates a new timestamped state journal directly in the user's home directory.
    /// Fails with an error if either file already exists.
    pub fn create_new(
        home_dir: &Path,
        sources: &[PathBuf],
        destination: &Path,
    ) -> Result<Self> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("System clock before Unix epoch")?;
        let secs = now.as_secs();
        let nanos = now.subsec_nanos();
        let pid = std::process::id();
        let base_name = format!("csc_state_{secs}_{nanos}_{pid}");

        let journal_path = home_dir.join(format!("{base_name}.cscjournal"));
        let desc_path = home_dir.join(format!("{base_name}.cscdesc"));

        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&journal_path)
            .with_context(|| {
                format!(
                    "Failed to create state journal (file already exists or inaccessible): {}",
                    journal_path.display()
                )
            })?;

        let mut writer = BufWriter::new(file);
        writer.write_all(JOURNAL_MAGIC)?;

        let mut jw = Self {
            journal_path,
            desc_path,
            writer,
            sources: sources.to_vec(),
            destination: destination.to_path_buf(),
            current_batch_id: 0,
            uncommitted_files: Vec::new(),
            uncommitted_dirs: Vec::new(),
            uncommitted_symlinks: Vec::new(),
            uncommitted_hardlinks: Vec::new(),
            total_committed_files: 0,
            total_committed_bytes: 0,
        };

        jw.write_session_header(sources, destination)?;
        jw.update_desc_file("InProgress")?;
        Ok(jw)
    }

    /// Reopens an existing journal for resuming, seeking to the end.
    pub fn open_for_resume(
        journal_path: &Path,
        desc_path: &Path,
        snapshot: &JournalSnapshot,
    ) -> Result<Self> {
        let file = OpenOptions::new()
            .append(true)
            .open(journal_path)
            .with_context(|| {
                format!("Failed to reopen state journal: {}", journal_path.display())
            })?;

        let mut total_bytes = 0_u64;
        for f in snapshot.committed_files.values() {
            total_bytes = total_bytes.saturating_add(f.size);
        }

        let writer = BufWriter::new(file);
        Ok(Self {
            journal_path: journal_path.to_path_buf(),
            desc_path: desc_path.to_path_buf(),
            writer,
            sources: snapshot.sources.clone(),
            destination: snapshot.destination.clone(),
            current_batch_id: snapshot.last_batch_id,
            uncommitted_files: Vec::new(),
            uncommitted_dirs: Vec::new(),
            uncommitted_symlinks: Vec::new(),
            uncommitted_hardlinks: Vec::new(),
            total_committed_files: u64::try_from(snapshot.committed_files.len())?,
            total_committed_bytes: total_bytes,
        })
    }

    fn write_session_header(&mut self, sources: &[PathBuf], destination: &Path) -> Result<()> {
        self.writer.write_all(&[TAG_SESSION_HEADER])?;
        write_u32(&mut self.writer, u32::try_from(sources.len())?)?;
        for src in sources {
            write_bytes(&mut self.writer, src.as_os_str().as_encoded_bytes())?;
        }
        write_bytes(&mut self.writer, destination.as_os_str().as_encoded_bytes())?;
        self.writer.flush()?;
        Ok(())
    }

    pub fn record_file(&mut self, file: ManifestFile) {
        self.uncommitted_files.push(file);
    }

    pub fn record_dir(&mut self, dir: ManifestDir) {
        self.uncommitted_dirs.push(dir);
    }

    pub fn record_symlink(&mut self, symlink: ManifestSymlink) {
        self.uncommitted_symlinks.push(symlink);
    }

    pub fn record_hardlink(&mut self, source_rel: PathBuf, target_rel: PathBuf) {
        self.uncommitted_hardlinks.push((source_rel, target_rel));
    }

    /// Records any `FileEntity` by dispatching to the appropriate manifest recorder.
    pub fn record_entity(&mut self, entity: &FileEntity) {
        match &entity.kind {
            FileEntityKind::Directory | FileEntityKind::Bundle { .. } => {
                self.record_dir(ManifestDir::from_entity(entity));
            }
            FileEntityKind::Regular { .. } => {
                if let Some(mf) = ManifestFile::from_entity(entity) {
                    self.record_file(mf);
                }
            }
            FileEntityKind::Symlink { .. } => {
                if let Some(ms) = ManifestSymlink::from_entity(entity) {
                    self.record_symlink(ms);
                }
            }
            FileEntityKind::Hardlink {
                target_relative_path,
            } => {
                self.record_hardlink(
                    entity.identity.relative_path.clone(),
                    target_relative_path.clone(),
                );
            }
            _ => {}
        }
    }

    /// Commits all pending items to disk with an explicit fsync, advancing the transaction.
    pub fn commit_batch(&mut self) -> Result<()> {
        if self.uncommitted_files.is_empty()
            && self.uncommitted_dirs.is_empty()
            && self.uncommitted_symlinks.is_empty()
            && self.uncommitted_hardlinks.is_empty()
        {
            return Ok(());
        }

        // 1. Write directory entries
        for d in &self.uncommitted_dirs {
            self.writer.write_all(&[TAG_DIR])?;
            write_bytes(&mut self.writer, d.relative_path.as_os_str().as_encoded_bytes())?;
            write_u32(&mut self.writer, d.mode)?;
            write_u32(&mut self.writer, d.uid)?;
            write_u32(&mut self.writer, d.gid)?;
            write_i64(&mut self.writer, d.mtime_sec)?;
            write_u32(&mut self.writer, d.mtime_nsec)?;
            write_i64(&mut self.writer, d.atime_sec)?;
            write_u32(&mut self.writer, d.atime_nsec)?;
            write_i64(&mut self.writer, d.ctime_sec)?;
            write_u32(&mut self.writer, d.ctime_nsec)?;
            write_opt_timestamp(&mut self.writer, d.birthtime_sec, d.birthtime_nsec)?;
            write_u32(&mut self.writer, u32::try_from(d.flags.len())?)?;
            for flag in &d.flags {
                write_flag(&mut self.writer, *flag)?;
            }
            write_u32(&mut self.writer, u32::try_from(d.streams.len())?)?;
            for (sname, shash) in &d.streams {
                write_bytes(&mut self.writer, sname.as_encoded_bytes())?;
                self.writer.write_all(shash)?;
            }
        }

        // 2. Write file entries
        for f in &self.uncommitted_files {
            self.writer.write_all(&[TAG_FILE])?;
            write_bytes(&mut self.writer, f.relative_path.as_os_str().as_encoded_bytes())?;
            write_u64(&mut self.writer, f.size)?;
            self.writer.write_all(&f.sha256)?;
            write_u32(&mut self.writer, f.mode)?;
            write_u32(&mut self.writer, f.uid)?;
            write_u32(&mut self.writer, f.gid)?;
            write_i64(&mut self.writer, f.mtime_sec)?;
            write_u32(&mut self.writer, f.mtime_nsec)?;
            write_i64(&mut self.writer, f.atime_sec)?;
            write_u32(&mut self.writer, f.atime_nsec)?;
            write_i64(&mut self.writer, f.ctime_sec)?;
            write_u32(&mut self.writer, f.ctime_nsec)?;
            write_opt_timestamp(&mut self.writer, f.birthtime_sec, f.birthtime_nsec)?;
            write_u32(&mut self.writer, u32::try_from(f.flags.len())?)?;
            for flag in &f.flags {
                write_flag(&mut self.writer, *flag)?;
            }
            self.writer.write_all(&[u8::from(f.is_sparse)])?;
            write_u32(&mut self.writer, u32::try_from(f.streams.len())?)?;
            for (sname, shash) in &f.streams {
                write_bytes(&mut self.writer, sname.as_encoded_bytes())?;
                self.writer.write_all(shash)?;
            }

            self.total_committed_files = self.total_committed_files.saturating_add(1);
            self.total_committed_bytes = self.total_committed_bytes.saturating_add(f.size);
        }

        // 3. Write symlink entries
        for s in &self.uncommitted_symlinks {
            self.writer.write_all(&[TAG_SYMLINK])?;
            write_bytes(&mut self.writer, s.relative_path.as_os_str().as_encoded_bytes())?;
            write_bytes(&mut self.writer, &s.target)?;
            write_u32(&mut self.writer, s.uid)?;
            write_u32(&mut self.writer, s.gid)?;
            write_i64(&mut self.writer, s.mtime_sec)?;
            write_u32(&mut self.writer, s.mtime_nsec)?;
            write_i64(&mut self.writer, s.atime_sec)?;
            write_u32(&mut self.writer, s.atime_nsec)?;
            write_i64(&mut self.writer, s.ctime_sec)?;
            write_u32(&mut self.writer, s.ctime_nsec)?;
            write_opt_timestamp(&mut self.writer, s.birthtime_sec, s.birthtime_nsec)?;
            write_u32(&mut self.writer, u32::try_from(s.flags.len())?)?;
            for flag in &s.flags {
                write_flag(&mut self.writer, *flag)?;
            }
        }

        // 4. Write hardlink entries
        for (src_rel, tgt_rel) in &self.uncommitted_hardlinks {
            self.writer.write_all(&[TAG_HARDLINK])?;
            write_bytes(&mut self.writer, src_rel.as_os_str().as_encoded_bytes())?;
            write_bytes(&mut self.writer, tgt_rel.as_os_str().as_encoded_bytes())?;
        }

        // 5. Write BATCH_COMMIT record
        self.current_batch_id = self.current_batch_id.saturating_add(1);
        self.writer.write_all(&[TAG_BATCH_COMMIT])?;
        write_u64(&mut self.writer, self.current_batch_id)?;

        // 6. Flush buffer and fsync journal
        self.writer.flush()?;
        self.writer.get_ref().sync_data()?;

        self.uncommitted_files.clear();
        self.uncommitted_dirs.clear();
        self.uncommitted_symlinks.clear();
        self.uncommitted_hardlinks.clear();

        self.update_desc_file("InProgress")?;
        Ok(())
    }

    /// Marks the job as completely finished and updates the `.cscdesc` companion file.
    pub fn mark_completed(&mut self) -> Result<()> {
        self.commit_batch()?;
        self.writer.write_all(&[TAG_JOB_COMPLETED])?;
        self.writer.flush()?;
        self.writer.get_ref().sync_data()?;
        self.update_desc_file("Completed")?;
        Ok(())
    }

    fn update_desc_file(&self, status: &str) -> Result<()> {
        use std::fmt::Write;
        let mut desc_text = String::new();
        writeln!(desc_text, "CSC State File")?;
        writeln!(desc_text, "Status: {status}")?;
        writeln!(desc_text, "FilesCommitted: {}", self.total_committed_files)?;
        writeln!(desc_text, "BytesCommitted: {}", self.total_committed_bytes)?;
        writeln!(desc_text, "Sources:")?;
        for s in &self.sources {
            writeln!(desc_text, "  - {}", s.display())?;
        }
        writeln!(desc_text, "Destination: {}", self.destination.display())?;

        let temp_desc = format!("{}.tmp", self.desc_path.display());
        std::fs::write(&temp_desc, desc_text.as_bytes())?;
        std::fs::rename(&temp_desc, &self.desc_path)?;
        Ok(())
    }

    #[must_use]
    pub fn journal_path(&self) -> &Path {
        &self.journal_path
    }

    #[must_use]
    pub fn destination(&self) -> &Path {
        &self.destination
    }
}

/// Resolves a manifest path or directory into a `.cscjournal` file path.
pub fn resolve_journal_path(path: &Path) -> Result<PathBuf> {
    if path.is_file() {
        if path.extension().and_then(|e| e.to_str()) == Some("cscdesc") {
            return Ok(path.with_extension("cscjournal"));
        }
        return Ok(path.to_path_buf());
    }

    if path.is_dir() {
        let mut candidates = Vec::new();
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) == Some("cscjournal") {
                if let Ok(meta) = p.metadata() {
                    let mtime = meta.modified().unwrap_or(UNIX_EPOCH);
                    candidates.push((mtime, p));
                }
            }
        }
        candidates.sort_by(|a, b| b.0.cmp(&a.0));
        if let Some((_, newest)) = candidates.into_iter().next() {
            return Ok(newest);
        }
    }

    anyhow::bail!(
        "Could not find valid .cscjournal file from path: {}",
        path.display()
    )
}

/// Reads a state journal, rolling back to the last valid `BATCH_COMMIT` marker.
pub fn read_journal_snapshot(path: &Path) -> Result<JournalSnapshot> {
    let journal_path = resolve_journal_path(path)?;
    let file = File::open(&journal_path).with_context(|| {
        format!("Failed to open journal file: {}", journal_path.display())
    })?;
    let mut reader = BufReader::new(file);

    let mut magic = [0_u8; 8];
    reader.read_exact(&mut magic).context("Incomplete journal magic")?;
    anyhow::ensure!(
        &magic == JOURNAL_MAGIC,
        "Invalid or unrecognized journal magic in {}",
        journal_path.display()
    );

    let mut sources = Vec::new();
    let mut destination = PathBuf::new();

    // Committed maps (persisted across batch commits)
    let mut committed_files = HashMap::new();
    let mut committed_dirs = HashMap::new();
    let mut committed_symlinks = HashMap::new();
    let mut committed_hardlinks = HashMap::new();

    // In-flight batch buffers (discarded if not followed by TAG_BATCH_COMMIT)
    let mut pending_files = HashMap::new();
    let mut pending_dirs = HashMap::new();
    let mut pending_symlinks = HashMap::new();
    let mut pending_hardlinks = HashMap::new();

    let mut last_batch_id = 0_u64;
    let mut is_completed = false;

    let mut tag_buf = [0_u8; 1];
    while reader.read_exact(&mut tag_buf).is_ok() {
        match tag_buf[0] {
            TAG_SESSION_HEADER => {
                let src_count = read_u32(&mut reader)?;
                for _ in 0..src_count {
                    let bytes = read_bytes(&mut reader)?;
                    sources.push(PathBuf::from(std::ffi::OsString::from_vec(bytes)));
                }
                let dest_bytes = read_bytes(&mut reader)?;
                destination = PathBuf::from(std::ffi::OsString::from_vec(dest_bytes));
            }
            TAG_DIR => {
                let rel_bytes = read_bytes(&mut reader)?;
                let rel_path = PathBuf::from(std::ffi::OsString::from_vec(rel_bytes));
                let mode = read_u32(&mut reader)?;
                let uid = read_u32(&mut reader)?;
                let gid = read_u32(&mut reader)?;
                let mtime_sec = read_i64(&mut reader)?;
                let mtime_nsec = read_u32(&mut reader)?;
                let atime_sec = read_i64(&mut reader)?;
                let atime_nsec = read_u32(&mut reader)?;
                let ctime_sec = read_i64(&mut reader)?;
                let ctime_nsec = read_u32(&mut reader)?;
                let (birthtime_sec, birthtime_nsec) = read_opt_timestamp(&mut reader)?;
                let flag_count = read_u32(&mut reader)?;
                let mut flags = Vec::with_capacity(usize::try_from(flag_count)?);
                for _ in 0..flag_count {
                    if let Some(flag) = read_flag(&mut reader)? {
                        flags.push(flag);
                    }
                }
                let stream_count = read_u32(&mut reader)?;
                let mut streams = Vec::with_capacity(usize::try_from(stream_count)?);
                for _ in 0..stream_count {
                    let sname_bytes = read_bytes(&mut reader)?;
                    let sname = OsString::from_vec(sname_bytes);
                    let mut shash = [0_u8; 32];
                    reader.read_exact(&mut shash)?;
                    streams.push((sname, shash));
                }
                pending_dirs.insert(
                    rel_path.clone(),
                    ManifestDir {
                        relative_path: rel_path,
                        mode,
                        uid,
                        gid,
                        mtime_sec,
                        mtime_nsec,
                        atime_sec,
                        atime_nsec,
                        ctime_sec,
                        ctime_nsec,
                        birthtime_sec,
                        birthtime_nsec,
                        flags,
                        streams,
                    },
                );
            }
            TAG_FILE => {
                let rel_bytes = read_bytes(&mut reader)?;
                let rel_path = PathBuf::from(std::ffi::OsString::from_vec(rel_bytes));
                let size = read_u64(&mut reader)?;
                let mut sha256 = [0_u8; 32];
                reader.read_exact(&mut sha256)?;
                let mode = read_u32(&mut reader)?;
                let uid = read_u32(&mut reader)?;
                let gid = read_u32(&mut reader)?;
                let mtime_sec = read_i64(&mut reader)?;
                let mtime_nsec = read_u32(&mut reader)?;
                let atime_sec = read_i64(&mut reader)?;
                let atime_nsec = read_u32(&mut reader)?;
                let ctime_sec = read_i64(&mut reader)?;
                let ctime_nsec = read_u32(&mut reader)?;
                let (birthtime_sec, birthtime_nsec) = read_opt_timestamp(&mut reader)?;
                let flag_count = read_u32(&mut reader)?;
                let mut flags = Vec::with_capacity(usize::try_from(flag_count)?);
                for _ in 0..flag_count {
                    if let Some(flag) = read_flag(&mut reader)? {
                        flags.push(flag);
                    }
                }
                let mut sparse_byte = [0_u8; 1];
                reader.read_exact(&mut sparse_byte)?;
                let is_sparse = sparse_byte[0] != 0;
                let stream_count = read_u32(&mut reader)?;
                let mut streams = Vec::with_capacity(usize::try_from(stream_count)?);
                for _ in 0..stream_count {
                    let sname_bytes = read_bytes(&mut reader)?;
                    let sname = OsString::from_vec(sname_bytes);
                    let mut shash = [0_u8; 32];
                    reader.read_exact(&mut shash)?;
                    streams.push((sname, shash));
                }
                pending_files.insert(
                    rel_path.clone(),
                    ManifestFile {
                        relative_path: rel_path,
                        size,
                        sha256,
                        mode,
                        uid,
                        gid,
                        mtime_sec,
                        mtime_nsec,
                        atime_sec,
                        atime_nsec,
                        ctime_sec,
                        ctime_nsec,
                        birthtime_sec,
                        birthtime_nsec,
                        flags,
                        is_sparse,
                        streams,
                    },
                );
            }
            TAG_SYMLINK => {
                let rel_bytes = read_bytes(&mut reader)?;
                let rel_path = PathBuf::from(std::ffi::OsString::from_vec(rel_bytes));
                let target = read_bytes(&mut reader)?;
                let uid = read_u32(&mut reader)?;
                let gid = read_u32(&mut reader)?;
                let mtime_sec = read_i64(&mut reader)?;
                let mtime_nsec = read_u32(&mut reader)?;
                let atime_sec = read_i64(&mut reader)?;
                let atime_nsec = read_u32(&mut reader)?;
                let ctime_sec = read_i64(&mut reader)?;
                let ctime_nsec = read_u32(&mut reader)?;
                let (birthtime_sec, birthtime_nsec) = read_opt_timestamp(&mut reader)?;
                let flag_count = read_u32(&mut reader)?;
                let mut flags = Vec::with_capacity(usize::try_from(flag_count)?);
                for _ in 0..flag_count {
                    if let Some(flag) = read_flag(&mut reader)? {
                        flags.push(flag);
                    }
                }
                pending_symlinks.insert(
                    rel_path.clone(),
                    ManifestSymlink {
                        relative_path: rel_path,
                        target,
                        uid,
                        gid,
                        mtime_sec,
                        mtime_nsec,
                        atime_sec,
                        atime_nsec,
                        ctime_sec,
                        ctime_nsec,
                        birthtime_sec,
                        birthtime_nsec,
                        flags,
                    },
                );
            }
            TAG_HARDLINK => {
                let src_bytes = read_bytes(&mut reader)?;
                let tgt_bytes = read_bytes(&mut reader)?;
                let src_rel = PathBuf::from(std::ffi::OsString::from_vec(src_bytes));
                let tgt_rel = PathBuf::from(std::ffi::OsString::from_vec(tgt_bytes));
                pending_hardlinks.insert(src_rel, tgt_rel);
            }
            TAG_BATCH_COMMIT => {
                let batch_id = read_u64(&mut reader)?;
                last_batch_id = batch_id;
                // Commit in-flight items into confirmed maps
                committed_files.extend(pending_files.drain());
                committed_dirs.extend(pending_dirs.drain());
                committed_symlinks.extend(pending_symlinks.drain());
                committed_hardlinks.extend(pending_hardlinks.drain());
            }
            TAG_JOB_COMPLETED => {
                is_completed = true;
            }
            _ => {
                // Unknown tag or truncated record; stop reading and keep valid committed state
                break;
            }
        }
    }

    Ok(JournalSnapshot {
        sources,
        destination,
        committed_files,
        committed_dirs,
        committed_symlinks,
        committed_hardlinks,
        is_completed,
        last_batch_id,
    })
}

// Low-level serialization helpers

fn write_bytes(w: &mut impl Write, bytes: &[u8]) -> Result<()> {
    write_u32(w, u32::try_from(bytes.len())?)?;
    w.write_all(bytes)?;
    Ok(())
}

fn read_bytes(r: &mut impl Read) -> Result<Vec<u8>> {
    let len = usize::try_from(read_u32(r)?)?;
    let mut buf = vec![0_u8; len];
    r.read_exact(&mut buf)?;
    Ok(buf)
}

fn write_u32(w: &mut impl Write, v: u32) -> Result<()> {
    w.write_all(&v.to_le_bytes())?;
    Ok(())
}

fn read_u32(r: &mut impl Read) -> Result<u32> {
    let mut buf = [0_u8; 4];
    r.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

fn write_u64(w: &mut impl Write, v: u64) -> Result<()> {
    w.write_all(&v.to_le_bytes())?;
    Ok(())
}

fn read_u64(r: &mut impl Read) -> Result<u64> {
    let mut buf = [0_u8; 8];
    r.read_exact(&mut buf)?;
    Ok(u64::from_le_bytes(buf))
}

fn write_i64(w: &mut impl Write, v: i64) -> Result<()> {
    w.write_all(&v.to_le_bytes())?;
    Ok(())
}

fn read_i64(r: &mut impl Read) -> Result<i64> {
    let mut buf = [0_u8; 8];
    r.read_exact(&mut buf)?;
    Ok(i64::from_le_bytes(buf))
}

fn write_flag(w: &mut impl Write, flag: FileFlag) -> Result<()> {
    write_bytes(w, flag.name().as_bytes())
}

fn read_flag(r: &mut impl Read) -> Result<Option<FileFlag>> {
    let bytes = read_bytes(r)?;
    let s = std::str::from_utf8(&bytes)?;
    Ok(FileFlag::from_name(s))
}

fn write_opt_timestamp(w: &mut impl Write, sec: Option<i64>, nsec: Option<u32>) -> Result<()> {
    match (sec, nsec) {
        (Some(s), Some(ns)) => {
            w.write_all(&[1])?;
            write_i64(w, s)?;
            write_u32(w, ns)?;
        }
        _ => {
            w.write_all(&[0])?;
        }
    }
    Ok(())
}

fn read_opt_timestamp(r: &mut impl Read) -> Result<(Option<i64>, Option<u32>)> {
    let mut tag = [0_u8; 1];
    r.read_exact(&mut tag)?;
    if tag[0] == 1 {
        let sec = read_i64(r)?;
        let nsec = read_u32(r)?;
        Ok((Some(sec), Some(nsec)))
    } else {
        Ok((None, None))
    }
}

