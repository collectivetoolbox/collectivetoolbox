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
use ctb_io_environment::EnvDescription;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek, Write};
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

const JOURNAL_MAGIC: &[u8; 8] = b"CTBCSCJ\x01";

const TAG_SESSION_HEADER: u8 = 1;
const TAG_ENTITY: u8 = 2;
const TAG_BATCH_COMMIT: u8 = 3;
const TAG_JOB_COMPLETED: u8 = 4;
const TAG_SESSION_ENV: u8 = 5;
const TAG_ERROR_RECORD: u8 = 6;
const TAG_WARNING_RECORD: u8 = 7;

pub const PLATFORM_LINUX: u8 = 1;
pub const PLATFORM_MACOS: u8 = 2;
pub const PLATFORM_WINDOWS: u8 = 3;
pub const PLATFORM_OTHER: u8 = 4;

/// Stage in the copy pipeline where an operation or file access error occurred.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum JournalErrorStage {
    /// Directory listing or entry enumeration failed.
    Traversal,
    /// Inspection of file status, flags, xattrs, or streams failed.
    MetadataRead,
    /// Opening source file or device failed.
    PayloadOpen,
    /// Reading file payload or stream data failed.
    PayloadRead,
    /// Writing or flushing destination payload or metadata failed.
    Materialize,
    /// Post-copy verification or concurrent modification check failed.
    PostCheck,
}

/// A persistent record of an item or operation failure stored in the journal.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct JournalErrorRecord {
    pub path: PathBuf,
    pub raw_path: Vec<u8>,
    pub stage: JournalErrorStage,
    pub error_message: String,
    pub os_error: Option<i32>,
    pub timestamp_sec: i64,
}

/// A persistent record of an operational warning or caveat stored in the journal.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct JournalWarningRecord {
    pub code: String,
    pub message: String,
    pub timestamp_sec: i64,
}

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
    pub noatime_used: bool,
    pub environment: Option<Arc<EnvDescription>>,
    pub errors: Vec<JournalErrorRecord>,
    pub warnings: Vec<JournalWarningRecord>,
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
    noatime_used: bool,
    pub environment: Option<Arc<EnvDescription>>,
    snapshot: Option<JournalSnapshot>,
    errors: Vec<JournalErrorRecord>,
    warnings: Vec<JournalWarningRecord>,
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

        let env = ctb_io_environment::capture_quick_arc();
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
            noatime_used: false,
            environment: Some(env),
            snapshot: None,
            errors: Vec::new(),
            warnings: Vec::new(),
        };

        jw.write_session_header(sources, destination)?;
        jw.write_session_environment()?;
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
            noatime_used: snapshot.noatime_used,
            environment: snapshot
                .environment
                .clone()
                .or_else(|| Some(ctb_io_environment::capture_quick_arc())),
            snapshot: Some(snapshot.clone()),
            errors: snapshot.errors.clone(),
            warnings: snapshot.warnings.clone(),
        })
    }

    /// Records whether `O_NOATIME` was successfully used for payload operations.
    pub fn record_noatime_used(&mut self, used: bool) {
        if used {
            self.noatime_used = true;
        }
    }

    /// Whether `O_NOATIME` was used during this journal session.
    #[must_use]
    pub const fn noatime_used(&self) -> bool {
        self.noatime_used
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

    fn write_session_environment(&mut self) -> Result<()> {
        if let Some(ref env) = self.environment {
            let json = env.to_json()?;
            self.writer.write_all(&[TAG_SESSION_ENV])?;
            write_bytes(&mut self.writer, json.as_bytes())?;
            self.writer.flush()?;
        }
        Ok(())
    }

    /// Queues a `FileEntity` for transactional commit.
    pub fn record_entity(&mut self, entity: &FileEntity) {
        let mut entity = entity.clone();
        if let Some(ref env) = self.environment {
            attach_session_environment(&mut entity, env);
        }
        self.uncommitted_entities.push(entity);
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
        if self.errors.is_empty() {
            self.update_desc_file("Completed")?;
        } else {
            self.update_desc_file(&format!("Completed with {} error(s)", self.errors.len()))?;
        }
        Ok(())
    }

    /// Records an item or operational error directly into the state journal and descriptor.
    pub fn record_error(&mut self, error: &JournalErrorRecord) -> Result<()> {
        let json = serde_json::to_string(error)?;
        self.writer.write_all(&[TAG_ERROR_RECORD])?;
        write_bytes(&mut self.writer, json.as_bytes())?;
        self.writer.flush()?;
        self.writer.get_ref().sync_data()?;
        self.errors.push(error.clone());
        self.update_desc_file("InProgress")?;
        Ok(())
    }

    /// Records a caveat or operational warning directly into the state journal and descriptor.
    pub fn record_warning(&mut self, code: &str, message: &str) -> Result<()> {
        // Reason for fallback: pre-epoch system clock failure defaults warning timestamp to 0
        let now_sec = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()
            .and_then(|d| i64::try_from(d.as_secs()).ok())
            .unwrap_or(0);
        let record = JournalWarningRecord {
            code: code.to_string(),
            message: message.to_string(),
            timestamp_sec: now_sec,
        };
        let json = serde_json::to_string(&record)?;
        self.writer.write_all(&[TAG_WARNING_RECORD])?;
        write_bytes(&mut self.writer, json.as_bytes())?;
        self.writer.flush()?;
        self.writer.get_ref().sync_data()?;
        self.warnings.push(record);
        self.update_desc_file("InProgress")?;
        Ok(())
    }

    /// Marks the job as failed with a specified reason and updates `.cscdesc`.
    pub fn mark_failed(&mut self, reason: &str) -> Result<()> {
        let _ = self.commit_batch();
        self.writer.flush()?;
        self.writer.get_ref().sync_data()?;
        let status = format!("Failed: {reason}");
        self.update_desc_file(&status)?;
        Ok(())
    }

    /// Slice of all errors recorded during this journal session.
    #[must_use]
    pub fn errors(&self) -> &[JournalErrorRecord] {
        &self.errors
    }

    /// Slice of all warnings and caveats recorded during this journal session.
    #[must_use]
    pub fn warnings(&self) -> &[JournalWarningRecord] {
        &self.warnings
    }

    fn update_desc_file(&self, status: &str) -> Result<()> {
        use std::fmt::Write;
        let mut desc_text = String::new();
        writeln!(desc_text, "CSC State File")?;
        writeln!(desc_text, "Status: {status}")?;
        writeln!(desc_text, "FilesCommitted: {}", self.total_committed_files)?;
        writeln!(desc_text, "BytesCommitted: {}", self.total_committed_bytes)?;
        writeln!(desc_text, "ErrorsCount: {}", self.errors.len())?;
        if !self.errors.is_empty() {
            writeln!(desc_text, "Errors:")?;
            for err in &self.errors {
                writeln!(
                    desc_text,
                    "  - [{:?}] {}: {}",
                    err.stage,
                    err.path.display(),
                    err.error_message
                )?;
            }
        }
        writeln!(desc_text, "WarningsCount: {}", self.warnings.len())?;
        if !self.warnings.is_empty() {
            writeln!(desc_text, "Warnings:")?;
            for warn in &self.warnings {
                writeln!(desc_text, "  - [{}]: {}", warn.code, warn.message)?;
            }
        }
        writeln!(desc_text, "Sources:")?;
        for s in &self.sources {
            writeln!(desc_text, "  - {}", s.display())?;
        }
        writeln!(desc_text, "Destination: {}", self.destination.display())?;
        writeln!(desc_text, "NoatimeUsed: {}", self.noatime_used)?;

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

    #[must_use]
    pub fn snapshot(&self) -> Option<&JournalSnapshot> {
        self.snapshot.as_ref()
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
    let mut environment: Option<Arc<EnvDescription>> = None;
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

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
            TAG_SESSION_ENV => {
                let Ok(env_bytes) = read_bytes(&mut reader) else {
                    break;
                };
                if let Ok(json_str) = std::str::from_utf8(&env_bytes) {
                    if let Ok(env_desc) = EnvDescription::from_json(json_str) {
                        environment = Some(Arc::new(env_desc));
                    }
                }
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
                        let path_key = entity.identity.path_bytes().map_or_else(Vec::new, |b| b.to_vec());
                        pending_entities.insert(path_key, entity);
                    }
                    Err(error) => return Err(error).context("Checksummed journal entity cannot be decoded without losing metadata"),
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
            TAG_ERROR_RECORD => {
                let Ok(err_bytes) = read_bytes(&mut reader) else {
                    break;
                };
                if let Ok(json_str) = std::str::from_utf8(&err_bytes) {
                    if let Ok(record) = serde_json::from_str::<JournalErrorRecord>(json_str) {
                        errors.push(record);
                    }
                }
                valid_length = reader.stream_position()?;
            }
            TAG_WARNING_RECORD => {
                let Ok(warn_bytes) = read_bytes(&mut reader) else {
                    break;
                };
                if let Ok(json_str) = std::str::from_utf8(&warn_bytes) {
                    if let Ok(record) = serde_json::from_str::<JournalWarningRecord>(json_str) {
                        warnings.push(record);
                    }
                }
                valid_length = reader.stream_position()?;
            }
            _ => {
                break;
            }
        }
    }

    anyhow::ensure!(valid_length > 0, "Journal session header is incomplete");
    let desc_path = path.with_extension("cscdesc");
    let noatime_used = if desc_path.exists() {
        if let Ok(desc_content) = std::fs::read_to_string(&desc_path) {
            desc_content.lines().any(|l| l.trim() == "NoatimeUsed: true")
        } else {
            false
        }
    } else {
        false
    };

    if let Some(ref env) = environment {
        for entity in committed_entities.values_mut() {
            attach_session_environment(entity, env);
        }
    }

    Ok(JournalSnapshot {
        sources,
        destination,
        origin_platform,
        committed_entities,
        is_completed,
        last_batch_id,
        valid_length,
        noatime_used,
        environment,
        errors,
        warnings,
    })
}

fn attach_session_environment(entity: &mut FileEntity, env: &Arc<EnvDescription>) {
    if !entity.metadata.environment.as_ref().is_some_and(|e| Arc::ptr_eq(e, env)) {
        entity.metadata.environment = Some(Arc::clone(env));
    }
    for stream in &mut entity.streams {
        attach_session_environment(&mut stream.entity, env);
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct PreservationRecord {
    metadata: FileMetadata,
    extents: Vec<ctb_io::file::Extent>,
    streams: Vec<PreservedStream>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct PreservedStream {
    #[serde(default)]
    name: Option<StreamName>,
    kind: StreamKind,
    descriptor: Vec<u8>,
    data: Option<Vec<u8>>,
}

// Low-level entity serialization

fn write_entity_payload(w: &mut impl Write, entity: &FileEntity) -> Result<()> {
    // Reason for fallback: streams or entities without relative path write empty path bytes to wire format
    write_bytes(w, entity.identity.path_bytes().unwrap_or(&[]))?;
    // Reason for fallback: entities without filename write empty slice to wire format
    write_bytes(w, entity.identity.raw_filename.as_deref().unwrap_or(&[]))?;
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
    // Reason for fallback: default permissions 0o644 written when mode is absent in stream or non-posix entity
    write_u32(w, entity.metadata.mode.unwrap_or(0o644))?;
    // Reason for fallback: root uid 0 written when uid is unrecorded in wire format
    write_u32(w, entity.metadata.uid.unwrap_or(0))?;
    // Reason for fallback: root gid 0 written when gid is unrecorded in wire format
    write_u32(w, entity.metadata.gid.unwrap_or(0))?;
    let (mtime_sec, mtime_nsec, atime_sec, atime_nsec, ctime_sec, ctime_nsec, birthtime_sec, birthtime_nsec) =
        match &entity.metadata.timestamps {
            Some(ts) => (
                ts.mtime_sec,
                ts.mtime_nsec,
                ts.atime_sec,
                ts.atime_nsec,
                ts.ctime_sec,
                ts.ctime_nsec,
                ts.birthtime_sec,
                ts.birthtime_nsec,
            ),
            None => (0, 0, 0, 0, 0, 0, None, None),
        };
    write_i64(w, mtime_sec)?;
    write_u32(w, mtime_nsec)?;
    write_i64(w, atime_sec)?;
    write_u32(w, atime_nsec)?;
    write_i64(w, ctime_sec)?;
    write_u32(w, ctime_nsec)?;
    write_opt_timestamp(w, birthtime_sec, birthtime_nsec)?;
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
    // Reason for fallback: empty flags written when flags are absent
    let flags_slice = entity.metadata.flags.as_deref().unwrap_or(&[]);
    write_u32(w, u32::try_from(flags_slice.len())?)?;
    for flag in flags_slice {
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
        // Reason for fallback: Nameless streams have no stream name, so serialized stream name bytes default to empty.
        let name_bytes = s
            .name
            .as_ref()
            .map(|n| n.as_bytes().into_owned())
            .unwrap_or_default();
        write_bytes(w, &name_bytes)?;
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
    let mut streams = Vec::with_capacity(entity.streams.len());
    for stream in &entity.streams {
        let mut descriptor = Vec::new();
        write_entity_payload(&mut descriptor, &stream.entity)?;
        streams.push(PreservedStream {
            name: stream.name.clone(),
            kind: stream.kind,
            descriptor,
            data: stream.data.clone(),
        });
    }
    let record = PreservationRecord {
        metadata: entity.metadata.clone(),
        extents: match &entity.kind {
            FileEntityKind::Regular { extents, .. } => extents.clone(),
            _ => Vec::new(),
        },
        streams,
    };
    w.write_all(b"CTBMETA1")?;
    write_bytes(w, &serde_json::to_vec(&record)?)?;
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

        let stream_name = if sname_bytes.is_empty() {
            None
        } else {
            Some(StreamName::from_bytes(&sname_bytes))
        };
        // Reason for fallback: Nameless streams represent macOS resource forks whose kind cannot be inferred from a filename.
        let skind = stream_name
            .as_ref()
            .map(|n| StreamKind::infer_from_name(&n.as_bytes()))
            .unwrap_or(StreamKind::MacOsResourceFork);
        // Reason for fallback: Nameless streams lack a filename, so synthetic relative path defaults to empty.
        let relative_path = stream_name
            .as_ref()
            .map(|n| PathBuf::from(n.to_string_lossy().as_ref()))
            .unwrap_or_default();
        let s_entity = FileEntity {
            identity: FileIdentity {
                origin: FileOrigin::Synthetic,
                relative_path: Some(relative_path),
                enclosing_path: None,
                raw_relative_path: Some(sname_bytes.clone()),
                raw_filename: Some(sname_bytes),
                nlink: 1,
                hardlink_group: None,
            },
            metadata: FileMetadata {
                native: None,
                mode: Some(0o644),
                uid: Some(0),
                gid: Some(0),
                timestamps: Some(FileTimestamps {
                    atime_sec: 0,
                    atime_nsec: 0,
                    mtime_sec: 0,
                    mtime_nsec: 0,
                    ctime_sec: 0,
                    ctime_nsec: 0,
                    birthtime_sec: None,
                    birthtime_nsec: None,
                    resolution_nsec: None,
                }),
                flags: Some(Vec::new()),
                platform_raw_flags: None,
                read_time: None,
                filesystem_type: None,
                environment: None,
                apple: None,
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

    let mut entity = FileEntity {
        identity: FileIdentity {
            origin,
            relative_path: Some(relative_path),
            enclosing_path,
            raw_relative_path: Some(raw_rel_path),
            raw_filename: Some(raw_filename),
            nlink,
            hardlink_group,
        },
        metadata: FileMetadata {
            native: None,
            mode: Some(mode),
            uid: Some(uid),
            gid: Some(gid),
            timestamps: Some(FileTimestamps {
                atime_sec,
                atime_nsec,
                mtime_sec,
                mtime_nsec,
                ctime_sec,
                ctime_nsec,
                birthtime_sec,
                birthtime_nsec,
                resolution_nsec: None,
            }),
            flags: Some(flags),
            platform_raw_flags: None,
            read_time,
            filesystem_type: None,
            environment: None,
            apple: None,
        },
        kind,
        streams,
    };
    if !r.is_empty() {
        let mut magic = [0_u8; 8];
        r.read_exact(&mut magic)?;
        anyhow::ensure!(&magic == b"CTBMETA1", "Unknown preservation record version");
        let record: PreservationRecord = serde_json::from_slice(&read_bytes(&mut r)?)?;
        anyhow::ensure!(record.streams.len() == entity.streams.len(), "Stream descriptor count mismatch");
        entity.metadata = record.metadata;
        if let FileEntityKind::Regular { extents, .. } = &mut entity.kind {
            *extents = record.extents;
        }
        for (stream, preserved) in entity.streams.iter_mut().zip(record.streams) {
            stream.name = preserved.name;
            stream.kind = preserved.kind;
            stream.entity = Box::new(read_entity_payload(&preserved.descriptor, origin_platform)?);
            stream.data = preserved.data;
        }
        anyhow::ensure!(r.is_empty(), "Unexpected preservation record data");
    }
    Ok(entity)
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
#[cfg(test)]
pub(crate) fn find_cscjournal(state_dir: &Path) -> PathBuf {
    for entry in std::fs::read_dir(state_dir).expect("read state dir") {
        let entry = entry.expect("entry");
        if entry.path().extension().and_then(|e| e.to_str()) == Some("cscjournal") {
            return entry.path();
        }
    }
    panic!("No .cscjournal found in {}", state_dir.display());
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
    use std::fs;
    use std::path::{PathBuf};
    use tempfile::tempdir;

    #[crate::ctb_test]
    fn test_journal_entity_format_and_checksum_resilience() {
        use crate::journal::{JournalWriter, read_journal_snapshot};
        use ctb_io::file::entity::{FileEntity, FileEntityKind};
        use ctb_io::file::identity::{FileIdentity, FileOrigin};
        use ctb_io::file::metadata::{FileMetadata, FileTimestamps};
        use std::io::Write;

        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src");
        let dest = temp.path().join("dest");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&dest).expect("create dest");

        let mut writer = JournalWriter::create_new(temp.path(), &[src.clone()], &dest).expect("create journal");

        let read_time_expected = std::time::SystemTime::UNIX_EPOCH
            .checked_add(std::time::Duration::from_secs(1_700_000_000))
            .expect("valid timestamp");

        let mut file_entity = FileEntity {
            identity: FileIdentity {
                origin: FileOrigin::Synthetic,
                relative_path: Some(PathBuf::from("hello.txt")),
                enclosing_path: Some(src.clone()),
                raw_relative_path: Some(b"hello.txt".to_vec()),
                raw_filename: Some(b"hello.txt".to_vec()),
                nlink: 1,
                hardlink_group: None,
            },
            metadata: FileMetadata {
                native: None,
                mode: Some(0o644),
                uid: Some(1000),
                gid: Some(1000),
                timestamps: Some(FileTimestamps {
                    atime_sec: 1_700_000_000,
                    atime_nsec: 100,
                    mtime_sec: 1_700_000_001,
                    mtime_nsec: 200,
                    ctime_sec: 1_700_000_002,
                    ctime_nsec: 300,
                    birthtime_sec: None,
                    birthtime_nsec: None,
                    resolution_nsec: None,
                }),
                flags: Some(Vec::new()),
                platform_raw_flags: None,
                read_time: Some(read_time_expected),
                filesystem_type: None,
                environment: None,
                apple: None,
            },
            kind: FileEntityKind::Regular {
                size: 42,
                sha256: [0xAB; 32],
                is_sparse: false,
                extents: Vec::new(),
            },
            streams: Vec::new(),
        };

        let link_entity = FileEntity {
            identity: FileIdentity {
                origin: FileOrigin::Synthetic,
                relative_path: Some(PathBuf::from("link.txt")),
                enclosing_path: None,
                raw_relative_path: Some(b"link.txt".to_vec()),
                raw_filename: Some(b"link.txt".to_vec()),
                nlink: 2,
                hardlink_group: Some(12345),
            },
            metadata: FileMetadata {
                native: None,
                mode: Some(0o644),
                uid: Some(1000),
                gid: Some(1000),
                timestamps: Some(FileTimestamps {
                    atime_sec: 1_700_000_000,
                    atime_nsec: 0,
                    mtime_sec: 1_700_000_000,
                    mtime_nsec: 0,
                    ctime_sec: 1_700_000_000,
                    ctime_nsec: 0,
                    birthtime_sec: None,
                    birthtime_nsec: None,
                    resolution_nsec: None,
                }),
                flags: Some(Vec::new()),
                platform_raw_flags: None,
                read_time: None,
                filesystem_type: None,
                environment: None,
                apple: None,
            },
            kind: FileEntityKind::Hardlink {
                target_relative_path: b"hello.txt".to_vec(),
            },
            streams: Vec::new(),
        };

        if let Some(ref mut ts) = file_entity.metadata.timestamps {
            ts.birthtime_sec = Some(-123);
            ts.birthtime_nsec = Some(987_654_321);
        }
        file_entity.metadata.native = Some(ctb_io::file::metadata::NativeMetadata {
            source_os: ctb_io::file::OsFamily::Darwin,
            values: std::collections::BTreeMap::from([
                ("opaque".to_owned(), ctb_io::file::metadata::NativeMetadataValue::Bytes(vec![0, 255, 128])),
                ("unsigned".to_owned(), ctb_io::file::metadata::NativeMetadataValue::Unsigned(u64::MAX)),
                ("signed".to_owned(), ctb_io::file::metadata::NativeMetadataValue::Signed(i64::MIN)),
            ]),
        });
        file_entity.metadata.platform_raw_flags = Some(ctb_io::file::PlatformRawFlags {
            source_os: ctb_io::file::OsFamily::Darwin,
            raw_value: u64::MAX,
            has_unparsed_flags: true,
        });
        if let FileEntityKind::Regular { extents, .. } = &mut file_entity.kind {
            extents.push(ctb_io::file::Extent::Data { offset: 0, length: 42 });
        }
        let mut stream_entity = file_entity.clone();
        stream_entity.streams.clear();
        file_entity.streams.push(ctb_io::file::AttachedStream {
            name: Some(ctb_io::file::StreamName::from_bytes(b"user.raw\xff")),
            kind: ctb_io::file::StreamKind::SecurityLabel,
            entity: Box::new(stream_entity),
            data: Some(vec![0, 255, 128]),
        });
        let mut wide_stream = file_entity.streams[0].clone();
        wide_stream.name = Some(ctb_io::file::StreamName::from_windows_utf16(&[0x003a, 0xd800, 0x0061]));
        file_entity.streams.push(wide_stream);
        for name in [
            ctb_io::file::StreamName::from_bytes(b"user.binary\xff"),
            ctb_io::file::StreamName::from_windows_utf16(&[0x003a, 0xd800, 0x0061]),
        ] {
            file_entity.streams.push(ctb_io::file::AttachedStream::from_data(
                Some(name), ctb_io::file::StreamKind::NtfsAlternateDataStream, vec![0, 255, 128],
            ).unwrap());
        }
        file_entity.streams.push(ctb_io::file::AttachedStream::from_data(
            None, ctb_io::file::StreamKind::MacOsResourceFork, vec![1, 2, 3, 4],
        ).unwrap());
        file_entity.metadata.environment = writer.environment.clone();
        for stream in &mut file_entity.streams {
            stream.entity.metadata.environment = writer.environment.clone();
        }
        writer.record_entity(&file_entity);
        writer.record_entity(&link_entity);
        writer.commit_batch().expect("commit batch");

        let snap = read_journal_snapshot(writer.journal_path()).expect("read snapshot");
        assert_eq!(snap.committed_entities.len(), 2);
        assert!(snap.is_committed(b"hello.txt"));
        assert!(snap.is_committed(b"link.txt"));

        let read_file = snap.committed_entities.get(b"hello.txt".as_slice()).expect("get hello.txt");
        assert_eq!(read_file, &file_entity);
        assert_eq!(read_file.identity.enclosing_path, Some(src.clone()));
        assert_eq!(read_file.metadata.read_time, Some(read_time_expected));
        if let FileEntityKind::Regular { size, sha256, .. } = &read_file.kind {
            assert_eq!(*size, 42);
            assert_eq!(*sha256, [0xAB; 32]);
        } else {
            panic!("Expected regular file kind");
        }

        let read_link = snap.committed_entities.get(b"link.txt".as_slice()).expect("get link.txt");
        if let FileEntityKind::Hardlink { target_relative_path } = &read_link.kind {
            assert_eq!(target_relative_path, b"hello.txt");
        } else {
            panic!("Expected hardlink kind");
        }

        // Test corruption resilience: Append corrupted bytes (bad checksum)
        {
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .open(writer.journal_path())
                .expect("open for append");
            // TAG_ENTITY (2) + len (10) + checksum (0) + 10 junk bytes
            file.write_all(&[2, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]).expect("write junk");
            file.write_all(b"badpayload").expect("write junk payload");
        }

        // Snapshot should cleanly recover up to the last valid batch and discard corrupted bytes
        let recovered = read_journal_snapshot(writer.journal_path()).expect("recover after corruption");
        assert_eq!(recovered.committed_entities.len(), 2);
        assert_eq!(recovered.last_batch_id, snap.last_batch_id);
    }

    #[crate::ctb_test]
    fn test_journal_pre_epoch_read_time_rejected() {
        use crate::journal::JournalWriter;
        use ctb_io::file::entity::{FileEntity, FileEntityKind};
        use ctb_io::file::identity::{FileIdentity, FileOrigin};
        use ctb_io::file::metadata::{FileMetadata, FileTimestamps};

        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src");
        let dest = temp.path().join("dest");
        std::fs::create_dir_all(&src).expect("create src");
        std::fs::create_dir_all(&dest).expect("create dest");

        let mut writer = JournalWriter::create_new(temp.path(), &[src.clone()], &dest).expect("create journal");

        let pre_epoch_time = std::time::SystemTime::UNIX_EPOCH
            .checked_sub(std::time::Duration::from_secs(10))
            .expect("valid pre-epoch time");

        let file_entity = FileEntity {
            identity: FileIdentity {
                origin: FileOrigin::Synthetic,
                relative_path: Some(PathBuf::from("pre_epoch.txt")),
                enclosing_path: Some(src),
                raw_relative_path: Some(b"pre_epoch.txt".to_vec()),
                raw_filename: Some(b"pre_epoch.txt".to_vec()),
                nlink: 1,
                hardlink_group: None,
            },
            metadata: FileMetadata {
                native: None,
                mode: Some(0o644),
                uid: Some(1000),
                gid: Some(1000),
                timestamps: Some(FileTimestamps {
                    atime_sec: 1_700_000_000,
                    atime_nsec: 0,
                    mtime_sec: 1_700_000_000,
                    mtime_nsec: 0,
                    ctime_sec: 1_700_000_000,
                    ctime_nsec: 0,
                    birthtime_sec: None,
                    birthtime_nsec: None,
                    resolution_nsec: None,
                }),
                flags: Some(Vec::new()),
                platform_raw_flags: None,
                read_time: Some(pre_epoch_time),
                filesystem_type: None,
                environment: None,
                apple: None,
            },
            kind: FileEntityKind::Regular {
                size: 0,
                sha256: [0; 32],
                is_sparse: false,
                extents: Vec::new(),
            },
            streams: Vec::new(),
        };

        writer.record_entity(&file_entity);
        assert!(writer.commit_batch().is_err());
    }

    #[crate::ctb_test]
    fn test_failed_copy_retains_captured_metadata_and_destination() {
        let _fs_lock = ctb_io::file::FS_CACHE_TEST_MUTEX.lock().unwrap();
        ctb_io::file::clear_filesystem_cache();
        let temp = tempdir().unwrap();
        let source = temp.path().join("link");
        let destination = temp.path().join("destination");
        let state = temp.path().join("state");
        fs::create_dir(&state).unwrap();
        std::os::unix::fs::symlink("missing-target", &source).unwrap();
        fs::write(&destination, b"old destination").unwrap();
        let _ = std::fs::read_link(&source);
        let original = ctb_io::file::FileEntity::from_filesystem(&source, None).unwrap();
        assert!(original.metadata.timestamps.as_ref().and_then(|ts| ts.birthtime_sec).is_some());
        let mut args = default_test_args(vec![source.clone(), destination.clone()], state.clone());
        args.best_effort_metadata = false;
        assert!(run_csc(args).is_err());
        assert_eq!(fs::read(&destination).unwrap(), b"old destination");
        assert!(fs::symlink_metadata(&source).is_ok());
        let journal = find_cscjournal(&state);
        let snapshot = crate::journal::read_journal_snapshot(&journal).unwrap();
        assert!(!snapshot.is_completed);
        let recorded = snapshot.committed_entities.values().next().unwrap();
        let mut expected_native = original.metadata.native.clone();
        let mut actual_native = recorded.metadata.native.clone();
        if let (Some(exp), Some(act)) = (&mut expected_native, &mut actual_native) {
            exp.values.remove("statx.atime.nsec");
            act.values.remove("statx.atime.nsec");
            exp.values.remove("statx.ctime.nsec");
            act.values.remove("statx.ctime.nsec");
        }
        assert_eq!(actual_native, expected_native);
        let mut expected_ts = original.metadata.timestamps;
        let mut actual_ts = recorded.metadata.timestamps;
        if let Some(ref mut ts) = expected_ts {
            ts.atime_sec = 0;
            ts.atime_nsec = 0;
            ts.ctime_nsec = 0;
        }
        if let Some(ref mut ts) = actual_ts {
            ts.atime_sec = 0;
            ts.atime_nsec = 0;
            ts.ctime_nsec = 0;
        }
        assert_eq!(actual_ts, expected_ts);
        assert_eq!(recorded.metadata.platform_raw_flags, original.metadata.platform_raw_flags);
        assert_eq!(recorded.streams, original.streams);
    }

    #[crate::ctb_test]
    fn test_target_error_reporting_shows_target_file() {
        let temp = tempdir().expect("create tempdir");
        let scratch_file = temp.path().join("scratch.csc-tmp.12345");
        fs::write(&scratch_file, b"data").expect("write scratch");

        let intended_target = temp.path().join("my_real_file.bin");

        // Non-root user cannot chown to root (uid 0) unless running as root
        if nix::unistd::geteuid().as_raw() != 0 {
            let meta = ctb_io::file::FileMetadata {
                native: None,
                mode: Some(0o644),
                uid: Some(0),
                gid: Some(0),
                timestamps: Some(ctb_io::file::FileTimestamps {
                    atime_sec: 1_000_000,
                    atime_nsec: 0,
                    mtime_sec: 1_000_000,
                    mtime_nsec: 0,
                    ctime_sec: 1_000_000,
                    ctime_nsec: 0,
                    birthtime_sec: None,
                    birthtime_nsec: None,
                    resolution_nsec: None,
                }),
                flags: Some(Vec::new()),
                platform_raw_flags: None,
                read_time: None,
                filesystem_type: None,
                environment: None,
                apple: None,
            };

            let err = ctb_io::file::apply_entity_metadata(
                &scratch_file,
                Some(&intended_target),
                &meta,
                false,
                false,
                true, // strict_lossless
            )
            .unwrap_err();

            let err_msg = format!("{err:#}");
            // The error MUST mention the intended destination path, and NOT the scratch temp file!
            assert!(
                err_msg.contains("my_real_file.bin"),
                "Error message should mention the intended target file: {err_msg}"
            );
            assert!(
                !err_msg.contains("scratch.csc-tmp.12345"),
                "Error message should NOT mention the scratch temp path: {err_msg}"
            );
        }
    }

    #[crate::ctb_test("tokio")]
    async fn test_journal_and_index_environment_metadata() {
        use crate::journal::{JournalWriter, read_journal_snapshot};
        use ctb_io::file::entity::{FileEntity, FileEntityKind};
        use ctb_io::file::identity::{FileIdentity, FileOrigin};
        use ctb_io::file::metadata::{FileMetadata, FileTimestamps};
        use std::sync::Arc;
        use turso::{Builder, Value};

        let temp = tempdir().expect("tempdir");
        let journal_path = temp.path().join("test_env.cscjournal");
        let desc_path = temp.path().join("test_env.cscdesc");
        let src = temp.path().join("src");
        let dest = temp.path().join("dest");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&dest).expect("create dest");

        // 1. Create journal and verify environment is initialized
        let mut writer = JournalWriter::create_at_path(
            &journal_path,
            &desc_path,
            &[src.clone()],
            &dest,
        ).expect("create_at_path");

        assert!(writer.environment.is_some(), "JournalWriter should initialize environment");

        let rel_path = PathBuf::from("file_a.txt");
        let entity = FileEntity {
            identity: FileIdentity {
                origin: FileOrigin::Synthetic,
                relative_path: Some(rel_path.clone()),
                enclosing_path: None,
                raw_relative_path: Some(b"file_a.txt".to_vec()),
                raw_filename: Some(b"file_a.txt".to_vec()),
                nlink: 1,
                hardlink_group: None,
            },
            metadata: FileMetadata {
                native: None,
                mode: Some(0o644),
                uid: Some(1000),
                gid: Some(1000),
                timestamps: Some(FileTimestamps {
                    atime_sec: 1_700_000_000,
                    atime_nsec: 0,
                    mtime_sec: 1_700_000_000,
                    mtime_nsec: 0,
                    ctime_sec: 1_700_000_000,
                    ctime_nsec: 0,
                    birthtime_sec: None,
                    birthtime_nsec: None,
                    resolution_nsec: None,
                }),
                flags: Some(Vec::new()),
                platform_raw_flags: None,
                read_time: None,
                filesystem_type: None,
                environment: None,
                apple: None,
            },
            kind: FileEntityKind::Regular {
                size: 25,
                sha256: [0x11; 32],
                is_sparse: false,
                extents: Vec::new(),
            },
            streams: Vec::new(),
        };

        writer.record_entity(&entity);
        writer.commit_batch().expect("commit_batch");
        writer.mark_completed().expect("mark_completed");

        // 2. Read snapshot and verify environment attached to entity deduplicated
        let snapshot = read_journal_snapshot(&journal_path).expect("read snapshot");
        assert!(snapshot.environment.is_some(), "Snapshot should have decoded environment");
        let snap_env = snapshot.environment.as_ref().unwrap();

        let committed = snapshot.committed_entities.get(b"file_a.txt".as_slice())
            .expect("committed entity");
        assert!(committed.metadata.environment.is_some(), "Entity should inherit deduplicated environment");
        assert!(
            Arc::ptr_eq(snap_env, committed.metadata.environment.as_ref().unwrap()),
            "Arc pointer should be shared (deduplicated)"
        );

        // 3. Backward compatibility: reading a synthetic journal without TAG_SESSION_ENV
        let legacy_journal_path = temp.path().join("legacy.cscjournal");
        let mut legacy_bytes = Vec::new();
        legacy_bytes.extend_from_slice(b"CTBCSCJ\x01");
        // TAG_SESSION_HEADER = 1, current_platform = 1, sources count = 0, destination = ""
        legacy_bytes.push(1); // TAG_SESSION_HEADER
        legacy_bytes.push(1); // platform
        legacy_bytes.extend_from_slice(&0_u32.to_le_bytes()); // src_count = 0
        legacy_bytes.extend_from_slice(&0_u32.to_le_bytes()); // dest_len = 0
        legacy_bytes.push(4); // TAG_JOB_COMPLETED
        fs::write(&legacy_journal_path, legacy_bytes).expect("write legacy journal");

        let legacy_snapshot = read_journal_snapshot(&legacy_journal_path).expect("read legacy snapshot");
        assert!(legacy_snapshot.environment.is_none(), "Legacy journal should yield None environment");

        // 4. Test SQLite database schema initialization, migration, and sources environment column
        let db_path = temp.path().join("env_test.cscindex.sqlite");
        let db = Builder::new_local(db_path.to_str().expect("valid path"))
            .experimental_index_method(true)
            .build()
            .await
            .expect("open db");
        let conn = db.connect().expect("connect db");

        crate::index_engine::init_database_schema(&conn).await.expect("init database schema");

        // Verify environment column exists in sources table
        let mut pragma_stmt = conn.prepare("PRAGMA table_info(sources)").await.expect("prepare pragma");
        let mut pragma_rows = pragma_stmt.query(()).await.expect("query pragma");
        let mut has_env_col = false;
        while let Some(row) = pragma_rows.next().await.expect("row") {
            if let Ok(Value::Text(col)) = row.get_value(1) {
                if col == "environment" {
                    has_env_col = true;
                    break;
                }
            }
        }
        assert!(has_env_col, "sources table must have environment column");

        // Ingest snapshot into database
        let progress = ctb_utilities::Progress::new(false);
        let env_json = snapshot.environment.as_ref().and_then(|e| e.to_json().ok());
        let src_id = crate::index_engine::get_or_create_source(
            &conn,
            "test_source",
            &journal_path,
            env_json.as_deref(),
        ).await.expect("get_or_create_source");

        let ingested = crate::index_engine::ingest_journal_snapshot(
            &conn,
            src_id,
            &snapshot,
            100,
            &progress,
            None,
        ).await.expect("ingest");
        assert_eq!(ingested, 1);

        // Verify the environment JSON in sources row
        let mut select_stmt = conn.prepare("SELECT environment FROM sources WHERE id = ?").await.expect("prepare");
        let mut select_rows = select_stmt.query(vec![Value::Integer(src_id)]).await.expect("query");
        let row = select_rows.next().await.expect("next").expect("row present");
        let stored_env = row.get_value(0).expect("get value");
        if let Value::Text(json_str) = stored_env {
            assert!(json_str.contains("\"os\":"), "Stored JSON should contain os");
            let decoded = ctb_io_environment::EnvDescription::from_json(&json_str)
                .expect("valid environment JSON");
            assert_eq!(decoded.os, snap_env.os);
            assert_eq!(decoded.is_linux(), snap_env.is_linux());
            assert_eq!(decoded.is_windows(), snap_env.is_windows());
        } else {
            panic!("Expected Value::Text for sources.environment");
        }
    }

    #[crate::ctb_test]
    fn test_environment_metadata_auto_population_and_operation_isolation() {
        let temp = tempdir().unwrap();
        let file1 = temp.path().join("file1.txt");
        let file2 = temp.path().join("file2.txt");
        fs::write(&file1, b"hello file 1").unwrap();
        fs::write(&file2, b"hello file 2").unwrap();

        // 1. FileEntity::from_filesystem automatically populates environment metadata
        let mut entity1 = ctb_io::file::FileEntity::from_filesystem(&file1, None).unwrap();
        assert!(entity1.metadata.environment.is_some());
        assert!(entity1.environment().is_some());

        // 2. Attached streams also automatically populate environment metadata
        let stream = ctb_io::file::AttachedStream::from_data(
            Some(ctb_io::file::StreamName::from_bytes(b"stream1")),
            ctb_io::file::StreamKind::NtfsAlternateDataStream,
            vec![1, 2, 3],
        ).unwrap();
        assert!(stream.entity.metadata.environment.is_some());
        entity1.streams.push(stream);

        // 3. Unscoped calls within the same operation epoch share the exact same Arc (pointer equality)
        let entity2 = ctb_io::file::FileEntity::from_filesystem(&file2, None).unwrap();
        let env1 = entity1.metadata.environment.as_ref().unwrap();
        let env2 = entity2.metadata.environment.as_ref().unwrap();
        assert!(std::sync::Arc::ptr_eq(env1, env2));

        // 4. FileEntity::set_environment propagates to all attached streams
        let custom_env = std::sync::Arc::new(ctb_io_environment::capture_quick());
        entity1.set_environment(std::sync::Arc::clone(&custom_env));
        assert!(std::sync::Arc::ptr_eq(
            entity1.metadata.environment.as_ref().unwrap(),
            &custom_env
        ));
        assert!(std::sync::Arc::ptr_eq(
            entity1.streams[0].entity.metadata.environment.as_ref().unwrap(),
            &custom_env
        ));

        // 5. Simulate two separate operations within a single process (e.g. successive csc runs):
        // Each operation enters its own GlobalEnvironmentScope, getting a distinct Arc snapshot.
        let arc_op1 = {
            let _scope1 = ctb_io_environment::GlobalEnvironmentScope::enter_fresh();
            let e = ctb_io::file::FileEntity::from_filesystem(&file1, None).unwrap();
            e.metadata.environment.unwrap()
        };

        let arc_op2 = {
            let _scope2 = ctb_io_environment::GlobalEnvironmentScope::enter_fresh();
            let e = ctb_io::file::FileEntity::from_filesystem(&file1, None).unwrap();
            e.metadata.environment.unwrap()
        };

        // Independent operations get distinct Arc instances (not inadvertently cached together)
        assert!(!std::sync::Arc::ptr_eq(&arc_op1, &arc_op2));

        // Global network caches remain intact across operations
        if let Ok(ip) = ctb_io_environment::local_ipv4() {
            let cached = ctb_io_environment::cached_local_ipv4();
            assert_eq!(cached.ok(), Some(ip));
        }
    }

    #[crate::ctb_test("tokio")]
    async fn test_full_provenance_csc_and_related_commands() {
        use crate::args::{default_test_args, default_verify_args, default_fsindex_args, MvArgs};
        use crate::cli::run_csc;
        use crate::verifier::run_csc_verify;
        use crate::index_engine::run_fsindex;
        use crate::move_engine::run_mv;
        use crate::journal::{read_journal_snapshot, find_cscjournal};
        use turso::Builder;

        let temp = tempdir().unwrap();
        let src = temp.path().join("src");
        let dest = temp.path().join("dest");
        let state = temp.path().join("state");
        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&dest).unwrap();
        fs::create_dir_all(&state).unwrap();

        fs::write(src.join("test.txt"), b"full provenance test content").unwrap();

        // 1. Run csc with full_provenance: true
        let mut csc_args = default_test_args(
            vec![PathBuf::from(format!("{}/", src.display())), dest.clone()],
            state.clone(),
        );
        csc_args.full_provenance = true;
        run_csc(csc_args).expect("run csc with full provenance");

        let journal_path = find_cscjournal(&state);
        let snapshot = read_journal_snapshot(&journal_path).expect("read journal snapshot");
        assert!(snapshot.environment.is_some(), "Journal snapshot must have environment");
        let env = snapshot.environment.as_ref().unwrap();
        assert!(!env.ctb_version.is_empty());

        // Verify entities committed in snapshot have matching environment
        for entity in snapshot.committed_entities.values() {
            assert!(entity.metadata.environment.is_some());
        }

        // 2. Run csc-verify with full_provenance: true
        let mut verify_args = default_verify_args(journal_path.clone(), None);
        verify_args.full_provenance = true;
        let v_res = run_csc_verify(&verify_args).expect("run csc-verify with full provenance");
        match v_res {
            ctb_utilities::cli::ToolResult::Immediate { exit_code, .. } => {
                assert_eq!(exit_code, 0);
            }
            _ => panic!("Expected Immediate ToolResult from csc-verify"),
        }

        // 3. Run fsindex with full_provenance: true
        let db_path = temp.path().join("provenance.cscindex.sqlite");
        let mut fsindex_args = default_fsindex_args(vec![dest.clone()], Some(db_path.clone()));
        fsindex_args.full_provenance = true;
        run_fsindex(fsindex_args).await.expect("run fsindex with full provenance");

        let db = Builder::new_local(db_path.to_str().unwrap()).build().await.expect("open db");
        let conn = db.connect().expect("connect db");
        let mut select_stmt = conn.prepare("SELECT environment FROM sources").await.expect("prepare");
        let mut rows = select_stmt.query(()).await.expect("query");
        let row = rows.next().await.expect("next").expect("row exists");
        let env_val = row.get_value(0).expect("get_value");
        match env_val {
            turso::Value::Text(json_str) => {
                let parsed = ctb_io_environment::EnvDescription::from_json(&json_str)
                    .expect("parse environment JSON");
                assert_eq!(parsed.ctb_version, env.ctb_version);
            }
            _ => panic!("Expected Value::Text for sources.environment"),
        }

        // 4. Run mv with full_provenance: true
        let mv_src = temp.path().join("mv_src");
        let mv_dest = temp.path().join("mv_dest");
        fs::create_dir_all(&mv_src).unwrap();
        fs::write(mv_src.join("mv_test.txt"), b"move with provenance").unwrap();
        let mv_args = MvArgs {
            paths: vec![mv_src.clone(), mv_dest.clone()],
            verbose: false,
            progress: false,
            no_progress: true,
            verify_after: true,
            no_verify_after: false,
            best_effort_metadata: false,
            allow_unknown_fs: false,
            force: false,
            dry_run: false,
            full_provenance: true,
        };
        run_mv(mv_args).expect("run mv with full provenance");
        assert!(mv_dest.join("mv_test.txt").exists());
    }
}

