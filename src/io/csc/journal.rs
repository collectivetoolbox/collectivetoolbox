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

use ctb_formats_checksum::{Sha256Stream, xxhash};
use ctb_io::file::entity::{FileEntity, FileEntityKind};
use ctb_io::file::identity::{FileIdentity, FileOrigin, InodeKey, resolve_relative_path_for_os};
use ctb_io::file::metadata::{FileFlag, FileMetadata, FileTimestamps};
use ctb_io::file::streams::{AttachedStream, StreamKind, StreamName};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek, Write};
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const JOURNAL_MAGIC: &[u8; 8] = b"CTBCSCJ\x01";

const TAG_SESSION_HEADER: u8 = 1;
const TAG_ENTITY: u8 = 2;
const TAG_BATCH_COMMIT: u8 = 3;
const TAG_JOB_COMPLETED: u8 = 4;

pub const PLATFORM_LINUX: u8 = 1;
pub const PLATFORM_MACOS: u8 = 2;
pub const PLATFORM_WINDOWS: u8 = 3;
pub const PLATFORM_OTHER: u8 = 4;

#[must_use]
pub fn current_platform() -> u8 {
    if cfg!(target_os = "linux") {
        PLATFORM_LINUX
    } else if cfg!(target_os = "macos") {
        PLATFORM_MACOS
    } else if cfg!(target_os = "windows") {
        PLATFORM_WINDOWS
    } else {
        PLATFORM_OTHER
    }
}

/// Summary of an active or completed journal recovered from disk.
#[derive(Debug, Clone)]
pub struct JournalSnapshot {
    pub sources: Vec<PathBuf>,
    pub destination: PathBuf,
    pub origin_platform: u8,
    pub committed_entities: HashMap<Vec<u8>, FileEntity>,
    pub is_completed: bool,
    pub last_batch_id: u64,
    pub valid_length: u64,
}

impl JournalSnapshot {
    /// Returns true if the relative path (in raw bytes) has been committed.
    #[must_use]
    pub fn is_committed(&self, rel_bytes: &[u8]) -> bool {
        self.committed_entities.contains_key(rel_bytes)
    }
}

/// Writer managing the persistent state journal and its companion `.cscdesc` file.
pub struct JournalWriter {
    journal_path: PathBuf,
    desc_path: PathBuf,
    writer: BufWriter<File>,
    sources: Vec<PathBuf>,
    destination: PathBuf,
    current_batch_id: u64,
    uncommitted_entities: Vec<FileEntity>,
    total_committed_files: u64,
    total_committed_bytes: u64,
}

impl JournalWriter {
    /// Creates a new state journal at the specified path.
    /// Fails with an error if either file already exists.
    pub fn create_at_path(
        journal_path: &Path,
        desc_path: &Path,
        sources: &[PathBuf],
        destination: &Path,
    ) -> Result<Self> {
        let mut opts = OpenOptions::new();
        opts.write(true).create_new(true);
        #[cfg(unix)]
        opts.mode(0o600);
        let file = opts
            .open(journal_path)
            .with_context(|| {
                format!(
                    "Failed to create state journal (file already exists or inaccessible): {}",
                    journal_path.display()
                )
            })?;

        let mut writer = BufWriter::new(file);
        writer.write_all(JOURNAL_MAGIC)?;

        let mut jw = Self {
            journal_path: journal_path.to_path_buf(),
            desc_path: desc_path.to_path_buf(),
            writer,
            sources: sources.to_vec(),
            destination: destination.to_path_buf(),
            current_batch_id: 0,
            uncommitted_entities: Vec::new(),
            total_committed_files: 0,
            total_committed_bytes: 0,
        };

        jw.write_session_header(sources, destination)?;
        jw.update_desc_file("InProgress")?;
        Ok(jw)
    }

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
        Self::create_at_path(&journal_path, &desc_path, sources, destination)
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
        file.set_len(snapshot.valid_length).context("Failed to discard uncommitted journal tail")?;
        file.sync_all()?;

        let mut total_bytes = 0_u64;
        let mut total_files = 0_u64;
        for e in snapshot.committed_entities.values() {
            if let FileEntityKind::Regular { size, .. } = e.kind {
                total_bytes = total_bytes.saturating_add(size);
                total_files = total_files.saturating_add(1);
            }
        }

        let writer = BufWriter::new(file);
        Ok(Self {
            journal_path: journal_path.to_path_buf(),
            desc_path: desc_path.to_path_buf(),
            writer,
            sources: snapshot.sources.clone(),
            destination: snapshot.destination.clone(),
            current_batch_id: snapshot.last_batch_id,
            uncommitted_entities: Vec::new(),
            total_committed_files: total_files,
            total_committed_bytes: total_bytes,
        })
    }

    fn write_session_header(&mut self, sources: &[PathBuf], destination: &Path) -> Result<()> {
        self.writer.write_all(&[TAG_SESSION_HEADER])?;
        self.writer.write_all(&[current_platform()])?;
        write_u32(&mut self.writer, u32::try_from(sources.len())?)?;
        for src in sources {
            write_bytes(&mut self.writer, src.as_os_str().as_encoded_bytes())?;
        }
        write_bytes(&mut self.writer, destination.as_os_str().as_encoded_bytes())?;
        self.writer.flush()?;
        Ok(())
    }

    /// Queues a `FileEntity` for transactional commit.
    pub fn record_entity(&mut self, entity: &FileEntity) {
        self.uncommitted_entities.push(entity.clone());
    }

    /// Commits all pending items to disk with an explicit fsync, advancing the transaction.
    pub fn commit_batch(&mut self) -> Result<()> {
        if self.uncommitted_entities.is_empty() {
            return Ok(());
        }

        let mut batch_hasher = Sha256Stream::new();

        for entity in &self.uncommitted_entities {
            let mut payload = Vec::new();
            write_entity_payload(&mut payload, entity)?;
            let payload_len = u32::try_from(payload.len())?;
            let checksum = xxhash::xxhash3_64(&payload);

            batch_hasher.update(&payload);

            self.writer.write_all(&[TAG_ENTITY])?;
            write_u32(&mut self.writer, payload_len)?;
            self.writer.write_all(&checksum)?;
            self.writer.write_all(&payload)?;

            if let FileEntityKind::Regular { size, .. } = entity.kind {
                self.total_committed_files = self.total_committed_files.saturating_add(1);
                self.total_committed_bytes = self.total_committed_bytes.saturating_add(size);
            }
        }

        // Write BATCH_COMMIT record with batch SHA-256 hash
        self.current_batch_id = self.current_batch_id.saturating_add(1);
        self.writer.write_all(&[TAG_BATCH_COMMIT])?;
        write_u64(&mut self.writer, self.current_batch_id)?;
        let batch_sha256 = batch_hasher.finalize();
        self.writer.write_all(&batch_sha256)?;

        // Flush buffer and fsync journal
        self.writer.flush()?;
        self.writer.get_ref().sync_data()?;

        self.uncommitted_entities.clear();
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
    pub fn desc_path(&self) -> &Path {
        &self.desc_path
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
                    // Reason for fallback: filesystems lacking mtime support default to UNIX_EPOCH so missing timestamps sort oldest
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
    let mut origin_platform = 0_u8;

    let mut committed_entities = HashMap::new();
    let mut pending_entities = HashMap::new();
    let mut batch_hasher = Sha256Stream::new();

    let mut last_batch_id = 0_u64;
    let mut is_completed = false;
    let mut valid_length = 0_u64;

    let mut tag_buf = [0_u8; 1];
    while reader.read_exact(&mut tag_buf).is_ok() {
        match tag_buf[0] {
            TAG_SESSION_HEADER => {
                let Ok(plat) = read_u8(&mut reader) else {
                    break;
                };
                origin_platform = plat;

                let Ok(src_count) = read_u32(&mut reader) else {
                    break;
                };
                let mut valid_sources = true;
                for _ in 0..src_count {
                    if let Ok(bytes) = read_bytes(&mut reader) {
                        let is_windows = origin_platform == PLATFORM_WINDOWS;
                        let src_path = resolve_relative_path_for_os(&bytes, is_windows)?;
                        sources.push(src_path);
                    } else {
                        valid_sources = false;
                        break;
                    }
                }
                if !valid_sources {
                    break;
                }
                let Ok(dest_bytes) = read_bytes(&mut reader) else {
                    break;
                };
                let is_windows = origin_platform == PLATFORM_WINDOWS;
                destination = resolve_relative_path_for_os(&dest_bytes, is_windows)?;
                valid_length = reader.stream_position()?;
            }
            TAG_ENTITY => {
                is_completed = false;
                let Ok(payload_len) = read_u32(&mut reader) else {
                    break;
                };
                let mut expected_checksum = [0_u8; 8];
                if reader.read_exact(&mut expected_checksum).is_err() {
                    break;
                }
                let Ok(payload_len_usize) = usize::try_from(payload_len) else {
                    break;
                };
                let mut payload = vec![0_u8; payload_len_usize];
                if reader.read_exact(&mut payload).is_err() {
                    break;
                }
                let actual_checksum = xxhash::xxhash3_64(&payload);
                if actual_checksum != expected_checksum {
                    break;
                }

                batch_hasher.update(&payload);

                match read_entity_payload(&payload[..], origin_platform) {
                    Ok(entity) => {
                        let path_key = entity.identity.path_bytes().to_vec();
                        pending_entities.insert(path_key, entity);
                    }
                    Err(_) => {
                        break;
                    }
                }
            }
            TAG_BATCH_COMMIT => {
                let Ok(batch_id) = read_u64(&mut reader) else {
                    break;
                };
                let mut expected_batch_hash = [0_u8; 32];
                if reader.read_exact(&mut expected_batch_hash).is_err() {
                    break;
                }
                let actual_batch_hash = batch_hasher.finalize();
                if actual_batch_hash != expected_batch_hash {
                    break;
                }

                last_batch_id = batch_id;
                committed_entities.extend(pending_entities.drain());
                batch_hasher = Sha256Stream::new();
                valid_length = reader.stream_position()?;
            }
            TAG_JOB_COMPLETED => {
                anyhow::ensure!(pending_entities.is_empty(), "Completion marker precedes batch commit");
                is_completed = true;
                valid_length = reader.stream_position()?;
            }
            _ => {
                break;
            }
        }
    }

    anyhow::ensure!(valid_length > 0, "Journal session header is incomplete");
    Ok(JournalSnapshot {
        sources,
        destination,
        origin_platform,
        committed_entities,
        is_completed,
        last_batch_id,
        valid_length,
    })
}

// Low-level entity serialization

fn write_entity_payload(w: &mut impl Write, entity: &FileEntity) -> Result<()> {
    write_bytes(w, entity.identity.path_bytes())?;
    write_bytes(w, &entity.identity.raw_filename)?;
    match &entity.identity.enclosing_path {
        Some(p) => {
            w.write_all(&[1])?;
            write_bytes(w, p.as_os_str().as_encoded_bytes())?;
        }
        None => {
            w.write_all(&[0])?;
        }
    }
    write_u64(w, entity.identity.nlink)?;
    let (device_id, _inode) = match &entity.identity.origin {
        FileOrigin::Filesystem { key, .. } => (key.device_id, key.inode),
        _ => (0, 0),
    };
    match entity.identity.hardlink_group {
        Some(grp) => {
            w.write_all(&[1])?;
            write_u64(w, grp)?;
            write_u64(w, device_id)?;
        }
        None => {
            w.write_all(&[0])?;
        }
    }
    write_u32(w, entity.metadata.mode)?;
    write_u32(w, entity.metadata.uid)?;
    write_u32(w, entity.metadata.gid)?;
    write_i64(w, entity.metadata.timestamps.mtime_sec)?;
    write_u32(w, entity.metadata.timestamps.mtime_nsec)?;
    write_i64(w, entity.metadata.timestamps.atime_sec)?;
    write_u32(w, entity.metadata.timestamps.atime_nsec)?;
    write_i64(w, entity.metadata.timestamps.ctime_sec)?;
    write_u32(w, entity.metadata.timestamps.ctime_nsec)?;
    write_opt_timestamp(
        w,
        entity.metadata.timestamps.birthtime_sec,
        entity.metadata.timestamps.birthtime_nsec,
    )?;
    let (read_sec, read_nsec) = match entity.metadata.read_time {
        Some(t) => {
            let d = t
                .duration_since(UNIX_EPOCH)
                .context("Read time timestamp is before Unix epoch")?;
            let s = i64::try_from(d.as_secs())
                .context("Read time seconds exceed i64::MAX")?;
            (Some(s), Some(d.subsec_nanos()))
        }
        None => (None, None),
    };
    write_opt_timestamp(w, read_sec, read_nsec)?;
    write_u32(w, u32::try_from(entity.metadata.flags.len())?)?;
    for flag in &entity.metadata.flags {
        write_flag(w, *flag)?;
    }

    match &entity.kind {
        FileEntityKind::Regular {
            size,
            sha256,
            is_sparse,
            ..
        } => {
            w.write_all(&[0])?;
            write_u64(w, *size)?;
            w.write_all(sha256)?;
            w.write_all(&[u8::from(*is_sparse)])?;
        }
        FileEntityKind::Directory => {
            w.write_all(&[1])?;
        }
        FileEntityKind::Symlink { target } => {
            w.write_all(&[2])?;
            write_bytes(w, target)?;
        }
        FileEntityKind::Hardlink {
            target_relative_path,
        } => {
            w.write_all(&[3])?;
            write_bytes(w, target_relative_path)?;
        }
        FileEntityKind::Fifo => {
            w.write_all(&[4])?;
        }
        FileEntityKind::CharDevice { rdev } => {
            w.write_all(&[5])?;
            write_u64(w, *rdev)?;
        }
        FileEntityKind::BlockDevice { rdev } => {
            w.write_all(&[6])?;
            write_u64(w, *rdev)?;
        }
        FileEntityKind::Socket => {
            w.write_all(&[7])?;
        }
        FileEntityKind::Door => {
            w.write_all(&[8])?;
        }
        FileEntityKind::Bundle { bundle_type } => {
            w.write_all(&[9])?;
            write_bytes(w, bundle_type.as_bytes())?;
        }
    }

    write_u32(w, u32::try_from(entity.streams.len())?)?;
    for s in &entity.streams {
        write_bytes(w, &s.name.0)?;
        let hash = match &s.entity.kind {
            FileEntityKind::Regular { sha256, .. } => *sha256,
            _ => [0_u8; 32],
        };
        w.write_all(&hash)?;
    }
    match &entity.identity.origin {
        FileOrigin::Filesystem { key, canonical_path } => {
            w.write_all(&[1])?;
            write_u64(w, key.device_id)?;
            write_u64(w, key.inode)?;
            write_bytes(w, canonical_path.as_os_str().as_encoded_bytes())?;
        }
        _ => w.write_all(&[0])?,
    }
    Ok(())
}

fn read_entity_payload(mut r: &[u8], origin_platform: u8) -> Result<FileEntity> {
    let raw_rel_path = read_bytes(&mut r)?;
    let raw_filename = read_bytes(&mut r)?;
    let has_enc = read_u8(&mut r)?;
    let enclosing_path = if has_enc == 1 {
        let enc_bytes = read_bytes(&mut r)?;
        let is_windows = origin_platform == PLATFORM_WINDOWS;
        Some(resolve_relative_path_for_os(&enc_bytes, is_windows)?)
    } else {
        None
    };
    let nlink = read_u64(&mut r)?;
    let has_grp = read_u8(&mut r)?;
    let (hardlink_group, origin_device_id) = if has_grp == 1 {
        let grp = read_u64(&mut r)?;
        let dev = read_u64(&mut r)?;
        (Some(grp), dev)
    } else {
        (None, 0)
    };

    let mode = read_u32(&mut r)?;
    let uid = read_u32(&mut r)?;
    let gid = read_u32(&mut r)?;
    let mtime_sec = read_i64(&mut r)?;
    let mtime_nsec = read_u32(&mut r)?;
    let atime_sec = read_i64(&mut r)?;
    let atime_nsec = read_u32(&mut r)?;
    let ctime_sec = read_i64(&mut r)?;
    let ctime_nsec = read_u32(&mut r)?;
    let (birthtime_sec, birthtime_nsec) = read_opt_timestamp(&mut r)?;
    let (read_sec, read_nsec) = read_opt_timestamp(&mut r)?;
    let read_time = match (read_sec, read_nsec) {
        (Some(sec), Some(nsec)) => {
            let u_sec = u64::try_from(sec)
                .context("Journal read_time seconds is negative")?;
            anyhow::ensure!(
                nsec < 1_000_000_000,
                "Journal read_time nanoseconds out of range: {nsec}"
            );
            let duration = std::time::Duration::new(u_sec, nsec);
            let time = UNIX_EPOCH
                .checked_add(duration)
                .context("Journal read_time timestamp overflow")?;
            Some(time)
        }
        (None, None) => None,
        _ => anyhow::bail!("Journal read_time has mismatched sec/nsec components"),
    };

    let flag_count = read_u32(&mut r)?;
    let mut flags = Vec::with_capacity(usize::try_from(flag_count)?);
    for _ in 0..flag_count {
        if let Some(flag) = read_flag(&mut r)? {
            flags.push(flag);
        }
    }

    let kind_tag = read_u8(&mut r)?;
    let kind = match kind_tag {
        0 => {
            let size = read_u64(&mut r)?;
            let mut sha256 = [0_u8; 32];
            r.read_exact(&mut sha256)?;
            let is_sparse = read_u8(&mut r)? != 0;
            FileEntityKind::Regular {
                size,
                sha256,
                is_sparse,
                extents: Vec::new(),
            }
        }
        1 => FileEntityKind::Directory,
        2 => {
            let target = read_bytes(&mut r)?;
            FileEntityKind::Symlink { target }
        }
        3 => {
            let target_relative_path = read_bytes(&mut r)?;
            FileEntityKind::Hardlink {
                target_relative_path,
            }
        }
        4 => FileEntityKind::Fifo,
        5 => {
            let rdev = read_u64(&mut r)?;
            FileEntityKind::CharDevice { rdev }
        }
        6 => {
            let rdev = read_u64(&mut r)?;
            FileEntityKind::BlockDevice { rdev }
        }
        7 => FileEntityKind::Socket,
        8 => FileEntityKind::Door,
        9 => {
            let btype_bytes = read_bytes(&mut r)?;
            let bundle_type = String::from_utf8_lossy(&btype_bytes).to_string();
            FileEntityKind::Bundle { bundle_type }
        }
        _ => bail!("Unknown file type tag"),
    };

    let stream_count = read_u32(&mut r)?;
    let mut streams = Vec::with_capacity(usize::try_from(stream_count)?);
    for _ in 0..stream_count {
        let sname_bytes = read_bytes(&mut r)?;
        let mut shash = [0_u8; 32];
        r.read_exact(&mut shash)?;

        let stream_name = StreamName(sname_bytes.clone());
        let skind = StreamKind::infer_from_name(&sname_bytes);
        let s_entity = FileEntity {
            identity: FileIdentity {
                origin: FileOrigin::Synthetic,
                relative_path: PathBuf::from(stream_name.to_string_lossy().as_ref()),
                enclosing_path: None,
                raw_relative_path: sname_bytes.clone(),
                raw_filename: sname_bytes,
                nlink: 1,
                hardlink_group: None,
            },
            metadata: FileMetadata {
                mode: 0o644,
                uid: 0,
                gid: 0,
                timestamps: FileTimestamps {
                    atime_sec: 0,
                    atime_nsec: 0,
                    mtime_sec: 0,
                    mtime_nsec: 0,
                    ctime_sec: 0,
                    ctime_nsec: 0,
                    birthtime_sec: None,
                    birthtime_nsec: None,
                },
                flags: Vec::new(),
                platform_raw_flags: None,
                read_time: None,
            },
            kind: FileEntityKind::Regular {
                size: 0,
                sha256: shash,
                is_sparse: false,
                extents: Vec::new(),
            },
            streams: Vec::new(),
        };

        streams.push(AttachedStream {
            name: stream_name,
            kind: skind,
            entity: Box::new(s_entity),
            data: None,
        });
    }

    let is_windows = origin_platform == PLATFORM_WINDOWS;
    let relative_path = resolve_relative_path_for_os(&raw_rel_path, is_windows)?;

    let mut origin = if let Some(grp) = hardlink_group {
        FileOrigin::Filesystem {
            key: InodeKey {
                device_id: origin_device_id,
                inode: grp,
            },
            canonical_path: relative_path.clone(),
        }
    } else {
        FileOrigin::Synthetic
    };

    let mut origin_tag = [0_u8; 1];
    if r.read(&mut origin_tag)? != 0 {
        match origin_tag {
            [0] => {}
            [1] => {
                let device_id = read_u64(&mut r)?;
                let inode = read_u64(&mut r)?;
                let canonical_path = resolve_relative_path_for_os(&read_bytes(&mut r)?, is_windows)?;
                origin = FileOrigin::Filesystem { key: InodeKey { device_id, inode }, canonical_path };
            }
            _ => anyhow::bail!("Invalid journal source identity tag"),
        }
    }

    Ok(FileEntity {
        identity: FileIdentity {
            origin,
            relative_path,
            enclosing_path,
            raw_relative_path: raw_rel_path,
            raw_filename,
            nlink,
            hardlink_group,
        },
        metadata: FileMetadata {
            mode,
            uid,
            gid,
            timestamps: FileTimestamps {
                atime_sec,
                atime_nsec,
                mtime_sec,
                mtime_nsec,
                ctime_sec,
                ctime_nsec,
                birthtime_sec,
                birthtime_nsec,
            },
            flags,
            platform_raw_flags: None,
            read_time,
        },
        kind,
        streams,
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

fn read_u8(r: &mut impl Read) -> Result<u8> {
    let mut buf = [0_u8; 1];
    r.read_exact(&mut buf)?;
    Ok(buf[0])
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
