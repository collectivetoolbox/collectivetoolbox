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
use ctb_formats_dcstring::{
    DcMixedDecode, DcMixedEncode, DcMstring, SharedJson, SharedValueContext,
};
use ctb_io::file::entity::{FileEntity, FileEntityKind};
use ctb_io::file::identity::{FileIdentity, FileOrigin, InodeKey, resolve_relative_path_for_os};
use ctb_io::file::metadata::{FileFlag, FileMetadata, FileTimestamps};
use ctb_io::file::streams::{AttachedStream, StreamKind, StreamName};
use ctb_io::file::Natural;
use ctb_io_environment::EnvDescription;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek, Write};
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

pub mod legacy;

pub(crate) use legacy::{
    JOURNAL_MAGIC_V1, PreservationRecord, PreservedStream, TAG_BATCH_COMMIT,
    TAG_ENTITY, TAG_ERROR_RECORD, TAG_JOB_COMPLETED, TAG_SESSION_ENV,
    TAG_SESSION_HEADER, TAG_WARNING_RECORD, read_entity_payload,
    write_entity_payload,
};

pub const JOURNAL_MAGIC_V4: &[u8; 8] = b"CTBCSCJ\x04";
pub const JOURNAL_MAGIC: &[u8; 8] = JOURNAL_MAGIC_V4;
pub const SESSION_ENVIRONMENT_BINDING: &str = "csc.session.environment";

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

impl JournalErrorStage {
    /// Encodes the stage as a compact numeric code.
    #[must_use]
    pub const fn to_code(self) -> u8 {
        match self {
            Self::Traversal => 0,
            Self::MetadataRead => 1,
            Self::PayloadOpen => 2,
            Self::PayloadRead => 3,
            Self::Materialize => 4,
            Self::PostCheck => 5,
        }
    }

    /// Decodes a numeric code into the corresponding stage.
    #[must_use]
    pub const fn from_code(code: u8) -> Self {
        match code {
            1 => Self::MetadataRead,
            2 => Self::PayloadOpen,
            3 => Self::PayloadRead,
            4 => Self::Materialize,
            5 => Self::PostCheck,
            _ => Self::Traversal,
        }
    }
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
    shared_context: SharedValueContext,
    errors: Vec<JournalErrorRecord>,
    warnings: Vec<JournalWarningRecord>,
    is_legacy: bool,
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
        writer.write_all(JOURNAL_MAGIC_V4)?;
        let header = DcMstring::new();
        writer.write_all(header.as_bytes())?;

        let env = ctb_io_environment::capture_quick_arc();
        let mut shared_context = SharedValueContext::default();
        shared_context.insert(SESSION_ENVIRONMENT_BINDING, Arc::clone(&env))?;
        shared_context.insert("environment", Arc::clone(&env))?;

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
            shared_context,
            errors: Vec::new(),
            warnings: Vec::new(),
            is_legacy: false,
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
        let mut file = OpenOptions::new()
            .write(true)
            .open(journal_path)
            .with_context(|| {
                format!("Failed to reopen state journal: {}", journal_path.display())
            })?;
        file.set_len(snapshot.valid_length).context("Failed to discard uncommitted journal tail")?;
        file.sync_all()?;
        std::io::Seek::seek(&mut file, std::io::SeekFrom::End(0))?;

        let mut total_bytes = 0_u64;
        let mut total_files = 0_u64;
        for e in snapshot.committed_entities.values() {
            if let FileEntityKind::Regular { size, .. } = e.kind {
                total_bytes = total_bytes.saturating_add(size);
                total_files = total_files.saturating_add(1);
            }
        }

        let mut magic = [0_u8; 8];
        let is_legacy = if let Ok(mut f) = File::open(journal_path) {
            f.read_exact(&mut magic).map_or(false, |_| magic == *JOURNAL_MAGIC_V1)
        } else {
            false
        };

        let writer = BufWriter::new(file);
        let mut shared_context = SharedValueContext::default();
        if let Some(environment) = &snapshot.environment {
            shared_context.insert(SESSION_ENVIRONMENT_BINDING, Arc::clone(environment))?;
            shared_context.insert("environment", Arc::clone(environment))?;
        }
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
            environment: snapshot.environment.clone(),
            snapshot: Some(snapshot.clone()),
            shared_context,
            errors: snapshot.errors.clone(),
            warnings: snapshot.warnings.clone(),
            is_legacy,
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
        let mut mst = DcMstring::new();
        mst.push_char(ctb_formats_dcdata::dc::DC_BEGIN_OPERATION_SESSION);
        for src in sources {
            mst.push_char(ctb_formats_dcdata::dc::DC_SOURCE_PATH);
            src.encode_dc_mixed(&mut mst)?;
        }
        mst.push_char(ctb_formats_dcdata::dc::DC_DESTINATION_PATH);
        destination.encode_dc_mixed(&mut mst)?;
        mst.push_char(ctb_formats_dcdata::dc::DC_OPERATION_STATUS_CODE);
        u32::from(current_platform()).encode_dc_mixed(&mut mst)?;
        if let Some(ref env) = self.environment {
            env.encode_dc_mixed_with_context(&mut mst, &self.shared_context)?;
        }
        mst.push_char(ctb_formats_dcdata::dc::DC_END_OPERATION_SESSION);

        self.writer.write_all(mst.payload_bytes())?;
        self.writer.flush()?;
        self.writer.get_ref().sync_data()?;
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

        if self.is_legacy {
            let mut batch_hasher = Sha256Stream::new();
            for entity in &self.uncommitted_entities {
                let mut payload = Vec::new();
                legacy::write_entity_payload(&mut payload, entity, &self.shared_context)?;
                let payload_len = u32::try_from(payload.len())?;
                let checksum = xxhash::xxhash3_64(&payload);

                batch_hasher.update(&payload);

                self.writer.write_all(&[legacy::TAG_ENTITY])?;
                legacy::write_u32(&mut self.writer, payload_len)?;
                self.writer.write_all(&checksum)?;
                self.writer.write_all(&payload)?;

                if let FileEntityKind::Regular { size, .. } = entity.kind {
                    self.total_committed_files = self.total_committed_files.saturating_add(1);
                    self.total_committed_bytes = self.total_committed_bytes.saturating_add(size);
                }
            }

            self.current_batch_id = self.current_batch_id.saturating_add(1);
            self.writer.write_all(&[legacy::TAG_BATCH_COMMIT])?;
            legacy::write_u64(&mut self.writer, self.current_batch_id)?;
            let batch_sha256 = batch_hasher.finalize();
            self.writer.write_all(&batch_sha256)?;

            self.writer.flush()?;
            self.writer.get_ref().sync_data()?;

            self.uncommitted_entities.clear();
            self.update_desc_file("InProgress")?;
            return Ok(());
        }

        self.current_batch_id = self.current_batch_id.saturating_add(1);

        let mut mst = DcMstring::new();
        mst.push_char(ctb_formats_dcdata::dc::DC_BEGIN_CHECKSUMMED_SECTION);

        let payload_start = mst.payload_bytes().len();
        mst.push_char(ctb_formats_dcdata::dc::DC_SEQUENCE_IDENTIFIER);
        self.current_batch_id.encode_dc_mixed(&mut mst)?;

        for entity in &self.uncommitted_entities {
            if let Some(read_time) = entity.metadata.read_time {
                read_time
                    .duration_since(UNIX_EPOCH)
                    .context("Read time timestamp is before Unix epoch")?;
            }
            entity.encode_dc_mixed_with_context(&mut mst, &self.shared_context)?;

            if let FileEntityKind::Regular { size, .. } = entity.kind {
                self.total_committed_files = self.total_committed_files.saturating_add(1);
                self.total_committed_bytes = self.total_committed_bytes.saturating_add(size);
            }
        }

        let payload_end = mst.payload_bytes().len();
        let section_bytes = &mst.payload_bytes()[payload_start..payload_end];
        let digest = ctb_formats_checksum::sha256(section_bytes);

        mst.push_char(ctb_formats_dcdata::dc::DC_END_CHECKSUMMED_SECTION);
        mst.push_char(ctb_formats_dcdata::dc::DC_SECTION_DIGEST);
        mst.push_binary(&digest);

        self.writer.write_all(mst.payload_bytes())?;
        self.writer.flush()?;
        self.writer.get_ref().sync_data()?;

        self.uncommitted_entities.clear();
        self.update_desc_file("InProgress")?;
        Ok(())
    }

    /// Marks the job as completely finished and updates the `.cscdesc` companion file.
    pub fn mark_completed(&mut self) -> Result<()> {
        self.commit_batch()?;
        if self.is_legacy {
            self.writer.write_all(&[legacy::TAG_JOB_COMPLETED])?;
            self.writer.flush()?;
            self.writer.get_ref().sync_data()?;
            if self.errors.is_empty() {
                self.update_desc_file("Completed")?;
            } else {
                self.update_desc_file(&format!("Completed with {} error(s)", self.errors.len()))?;
            }
            return Ok(());
        }
        let mut mst = DcMstring::new();
        mst.push_char(ctb_formats_dcdata::dc::DC_OPERATION_COMPLETED);
        self.writer.write_all(mst.payload_bytes())?;
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
        if self.is_legacy {
            let json = serde_json::to_vec(error)?;
            self.writer.write_all(&[legacy::TAG_ERROR_RECORD])?;
            legacy::write_bytes(&mut self.writer, &json)?;
            self.writer.flush()?;
            self.writer.get_ref().sync_data()?;
            self.errors.push(error.clone());
            self.update_desc_file("InProgress")?;
            return Ok(());
        }
        let mut mst = DcMstring::new();
        mst.push_char(ctb_formats_dcdata::dc::DC_BEGIN_ERROR_RECORD);
        mst.push_char(ctb_formats_dcdata::dc::DC_SOURCE_PATH);
        error.path.encode_dc_mixed(&mut mst)?;
        mst.push_char(ctb_formats_dcdata::dc::DC_OPERATION_STATUS_CODE);
        let stage_code = u64::from(error.stage.to_code());
        let os_code = u64::from(
            error
                .os_error
                .and_then(|e| u32::try_from(e).ok())
                .unwrap_or(0),
        );
        let packed = (stage_code << 32) | os_code;
        packed.encode_dc_mixed(&mut mst)?;
        mst.push_char(ctb_formats_dcdata::dc::DC_LOG_MESSAGE);
        error.error_message.encode_dc_mixed(&mut mst)?;
        error.timestamp_sec.encode_dc_mixed(&mut mst)?;
        mst.push_char(ctb_formats_dcdata::dc::DC_END_ERROR_RECORD);

        self.writer.write_all(mst.payload_bytes())?;
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
        if self.is_legacy {
            let json = serde_json::to_vec(&record)?;
            self.writer.write_all(&[legacy::TAG_WARNING_RECORD])?;
            legacy::write_bytes(&mut self.writer, &json)?;
            self.writer.flush()?;
            self.writer.get_ref().sync_data()?;
            self.warnings.push(record);
            self.update_desc_file("InProgress")?;
            return Ok(());
        }
        let mut mst = DcMstring::new();
        mst.push_char(ctb_formats_dcdata::dc::DC_BEGIN_WARNING_RECORD);
        mst.push_char(ctb_formats_dcdata::dc::DC_LOG_MESSAGE);
        format!("{code}: {message}").encode_dc_mixed(&mut mst)?;
        now_sec.encode_dc_mixed(&mut mst)?;
        mst.push_char(ctb_formats_dcdata::dc::DC_END_WARNING_RECORD);

        self.writer.write_all(mst.payload_bytes())?;
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

/// Reads a state journal, rolling back to the last valid batch or record.
pub fn read_journal_snapshot(path: &Path) -> Result<JournalSnapshot> {
    let journal_path = resolve_journal_path(path)?;
    let mut file = File::open(&journal_path).with_context(|| {
        format!("Failed to open journal file: {}", journal_path.display())
    })?;

    let mut magic = [0_u8; 8];
    file.read_exact(&mut magic).context("Incomplete journal magic")?;

    let desc_path = journal_path.with_extension("cscdesc");
    let noatime_used = if desc_path.exists() {
        if let Ok(desc_content) = std::fs::read_to_string(&desc_path) {
            desc_content.lines().any(|l| l.trim() == "NoatimeUsed: true")
        } else {
            false
        }
    } else {
        false
    };

    match &magic {
        m if m == JOURNAL_MAGIC_V4 => {
            read_v4_journal_snapshot(&mut file, &journal_path, noatime_used)
        }
        m if m == JOURNAL_MAGIC_V1 => {
            let mut reader = BufReader::new(file);
            legacy::read_legacy_journal_snapshot(&mut reader, &journal_path)
        }
        _ => {
            anyhow::bail!(
                "Invalid or unrecognized journal magic in {}",
                journal_path.display()
            );
        }
    }
}

/// Reads a v4 native DcMixed state journal.
pub fn read_v4_journal_snapshot(
    file: &mut File,
    journal_path: &Path,
    noatime_used: bool,
) -> Result<JournalSnapshot> {
    file.seek(std::io::SeekFrom::Start(8))
        .context("Failed to seek past journal magic")?;
    let mut file_payload = Vec::new();
    file.read_to_end(&mut file_payload)
        .context("Failed to read journal payload")?;

    let mut valid_slice_len = file_payload.len();
    loop {
        if valid_slice_len < ctb_formats_dcstring::DCMX_HEADER_LEN {
            anyhow::bail!(
                "Journal {} is too short to contain a valid DcMixed header",
                journal_path.display()
            );
        }
        match ctb_formats_dcstring::validate_dcmst(&file_payload[..valid_slice_len]) {
            Ok(()) => break,
            Err(ctb_formats_dcstring::DcMixedError::TruncatedEncapsulation { offset, .. })
            | Err(ctb_formats_dcstring::DcMixedError::InvalidUtf8e128 { offset, .. })
            | Err(ctb_formats_dcstring::DcMixedError::InvalidEncapsulationType { offset, .. })
            | Err(ctb_formats_dcstring::DcMixedError::SizeOverflow { offset, .. })
            | Err(ctb_formats_dcstring::DcMixedError::ChecksumMismatch { offset, .. }) => {
                if offset >= valid_slice_len {
                    valid_slice_len = valid_slice_len.saturating_sub(1);
                } else {
                    valid_slice_len = offset;
                }
            }
            Err(err) => return Err(err).context("Invalid DcMixed journal structure"),
        }
    }

    let mstr = ctb_formats_dcstring::DcMstr::from_bytes(&file_payload[..valid_slice_len])?;
    let mut reader = ctb_formats_dcstring::DcMixedReader::new(mstr);

    let mut sources = Vec::new();
    let mut destination = PathBuf::new();
    let mut origin_platform = 0_u8;
    let mut environment: Option<Arc<EnvDescription>> = None;
    let mut shared_context = SharedValueContext::default();
    let mut committed_entities = HashMap::new();
    let mut pending_entities = HashMap::new();
    let mut is_completed = false;
    let mut last_batch_id = 0_u64;
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut valid_length = 0_u64;

    // Decode session header
    if reader.peek_char()? == Some(ctb_formats_dcdata::dc::DC_BEGIN_OPERATION_SESSION) {
        let _ = reader.next_char();
        let mut header_ok = true;
        while let Ok(Some(inner_ch)) = reader.peek_char() {
            if inner_ch == ctb_formats_dcdata::dc::DC_END_OPERATION_SESSION {
                break;
            }
            if inner_ch == ctb_formats_dcdata::dc::DC_SOURCE_PATH {
                let _ = reader.next_char();
                match PathBuf::decode_dc_mixed(&mut reader) {
                    Ok(src) => sources.push(src),
                    Err(_) => {
                        header_ok = false;
                        break;
                    }
                }
            } else if inner_ch == ctb_formats_dcdata::dc::DC_DESTINATION_PATH {
                let _ = reader.next_char();
                match PathBuf::decode_dc_mixed(&mut reader) {
                    Ok(dest) => destination = dest,
                    Err(_) => {
                        header_ok = false;
                        break;
                    }
                }
            } else if inner_ch == ctb_formats_dcdata::dc::DC_OPERATION_STATUS_CODE {
                let _ = reader.next_char();
                match u32::decode_dc_mixed(&mut reader) {
                    Ok(code) => {
                        origin_platform = u8::try_from(code).unwrap_or(0);
                    }
                    Err(_) => {
                        header_ok = false;
                        break;
                    }
                }
            } else if inner_ch == ctb_formats_dcdata::dc::DC_BEGIN_ENVIRONMENT_DESCRIPTION {
                match EnvDescription::decode_dc_mixed(&mut reader) {
                    Ok(env_desc) => {
                        environment = Some(Arc::new(env_desc));
                    }
                    Err(_) => {
                        header_ok = false;
                        break;
                    }
                }
            } else {
                let _ = reader.next_char();
            }
        }
        if header_ok
            && reader
                .expect_char(ctb_formats_dcdata::dc::DC_END_OPERATION_SESSION)
                .is_ok()
        {
            if let Some(ref env_arc) = environment {
                let _ = shared_context
                    .insert(SESSION_ENVIRONMENT_BINDING, Arc::clone(env_arc));
                let _ = shared_context.insert("environment", Arc::clone(env_arc));
            }
            reader.set_context(&shared_context);
            valid_length = 8_u64.saturating_add(u64::try_from(reader.position()).unwrap_or(0));
        }
    }

    anyhow::ensure!(valid_length > 0, "Journal session header is incomplete");

    while let Ok(Some(ch)) = reader.peek_char() {
        match ch {
            ctb_formats_dcdata::dc::DC_BEGIN_CHECKSUMMED_SECTION => {
                let _ = reader.next_char();
                let section_start = reader.position();
                if reader
                    .expect_char(ctb_formats_dcdata::dc::DC_SEQUENCE_IDENTIFIER)
                    .is_err()
                {
                    break;
                }
                let Ok(batch_id) = u64::decode_dc_mixed(&mut reader) else {
                    break;
                };
                let mut batch_ok = true;
                while let Ok(Some(inner_ch)) = reader.peek_char() {
                    if inner_ch == ctb_formats_dcdata::dc::DC_END_CHECKSUMMED_SECTION {
                        break;
                    }
                    match FileEntity::decode_dc_mixed(&mut reader) {
                        Ok(entity) => {
                            let path_key = entity
                                .identity
                                .path_bytes()
                                .map_or_else(Vec::new, |b| b.to_vec());
                            pending_entities.insert(path_key, entity);
                        }
                        Err(_) => {
                            batch_ok = false;
                            break;
                        }
                    }
                }
                if !batch_ok {
                    break;
                }
                let section_end = reader.position();
                if reader
                    .expect_char(ctb_formats_dcdata::dc::DC_END_CHECKSUMMED_SECTION)
                    .is_err()
                {
                    break;
                }
                if reader
                    .expect_char(ctb_formats_dcdata::dc::DC_SECTION_DIGEST)
                    .is_err()
                {
                    break;
                }
                let Ok(expected_digest) = reader.read_binary_payload() else {
                    break;
                };
                let Some(section_bytes) = mstr.as_bytes().get(section_start..section_end) else {
                    break;
                };
                let actual_digest = ctb_formats_checksum::sha256(section_bytes);
                if actual_digest.as_slice() != expected_digest.as_ref() {
                    break;
                }

                last_batch_id = batch_id;
                committed_entities.extend(pending_entities.drain());
                valid_length = 8_u64.saturating_add(u64::try_from(reader.position()).unwrap_or(0));
            }
            ctb_formats_dcdata::dc::DC_OPERATION_COMPLETED => {
                let _ = reader.next_char();
                if !pending_entities.is_empty() {
                    break;
                }
                is_completed = true;
                valid_length = 8_u64.saturating_add(u64::try_from(reader.position()).unwrap_or(0));
            }
            ctb_formats_dcdata::dc::DC_BEGIN_ERROR_RECORD => {
                let _ = reader.next_char();
                let Ok(_) = reader.expect_char(ctb_formats_dcdata::dc::DC_SOURCE_PATH) else {
                    break;
                };
                let Ok(path) = PathBuf::decode_dc_mixed(&mut reader) else {
                    break;
                };
                let Ok(_) = reader.expect_char(ctb_formats_dcdata::dc::DC_OPERATION_STATUS_CODE) else {
                    break;
                };
                let Ok(packed) = u64::decode_dc_mixed(&mut reader) else {
                    break;
                };
                let stage_code = u8::try_from(packed >> 32).unwrap_or(0);
                let os_code = u32::try_from(packed & 0xFFFF_FFFF).unwrap_or(0);
                let Ok(_) = reader.expect_char(ctb_formats_dcdata::dc::DC_LOG_MESSAGE) else {
                    break;
                };
                let Ok(error_message) = String::decode_dc_mixed(&mut reader) else {
                    break;
                };
                let Ok(timestamp_sec) = i64::decode_dc_mixed(&mut reader) else {
                    break;
                };
                if reader
                    .expect_char(ctb_formats_dcdata::dc::DC_END_ERROR_RECORD)
                    .is_err()
                {
                    break;
                }
                let raw_path = path.as_os_str().as_encoded_bytes().to_vec();
                errors.push(JournalErrorRecord {
                    path,
                    raw_path,
                    stage: JournalErrorStage::from_code(stage_code),
                    error_message,
                    os_error: if os_code == 0 {
                        None
                    } else {
                        i32::try_from(os_code).ok()
                    },
                    timestamp_sec,
                });
                valid_length = 8_u64.saturating_add(u64::try_from(reader.position()).unwrap_or(0));
            }
            ctb_formats_dcdata::dc::DC_BEGIN_WARNING_RECORD => {
                let _ = reader.next_char();
                let Ok(_) = reader.expect_char(ctb_formats_dcdata::dc::DC_LOG_MESSAGE) else {
                    break;
                };
                let Ok(combined) = String::decode_dc_mixed(&mut reader) else {
                    break;
                };
                let Ok(timestamp_sec) = i64::decode_dc_mixed(&mut reader) else {
                    break;
                };
                if reader
                    .expect_char(ctb_formats_dcdata::dc::DC_END_WARNING_RECORD)
                    .is_err()
                {
                    break;
                }
                let (code, message) = match combined.split_once(": ") {
                    Some((c, m)) => (c.to_string(), m.to_string()),
                    None => (String::new(), combined),
                };
                warnings.push(JournalWarningRecord {
                    code,
                    message,
                    timestamp_sec,
                });
                valid_length = 8_u64.saturating_add(u64::try_from(reader.position()).unwrap_or(0));
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
        noatime_used,
        environment,
        errors,
        warnings,
    })
}

fn attach_legacy_session_environment(entity: &mut FileEntity, env: &Arc<EnvDescription>) {
    if entity.metadata.environment.is_none()
        || entity.metadata.environment.as_ref().is_some_and(|existing| existing == env)
    {
        entity.metadata.environment = Some(Arc::clone(env));
    }
    for stream in &mut entity.streams {
        attach_legacy_session_environment(&mut stream.entity, env);
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
    use crate::args::{default_test_args};
    use crate::cli::run_csc;
    use crate::journal::find_cscjournal;
    use crate::journal::legacy::{read_bytes, read_u32, write_bytes, write_u32};
    use std::fs;
    use std::path::{PathBuf};
    use tempfile::tempdir;

    #[crate::ctb_test]
    fn test_journal_rejects_unknown_environment_json() -> Result<()> {
        let temp = tempdir()?;
        let journal_path = temp.path().join("unknown-environment.cscjournal");
        let mut bytes = JOURNAL_MAGIC_V1.to_vec();
        bytes.extend_from_slice(&[TAG_SESSION_HEADER, current_platform()]);
        write_u32(&mut bytes, 0)?;
        write_bytes(&mut bytes, b"")?;
        bytes.push(TAG_SESSION_ENV);
        write_bytes(&mut bytes, br#"{"os":"linux","future_flag":true}"#)?;
        fs::write(&journal_path, bytes)?;
        let error = read_journal_snapshot(&journal_path)
            .expect_err("unknown environment fields must not be silently discarded");
        assert!(format!("{error:#}").contains("unknown field"));
        Ok(())
    }

    #[crate::ctb_test]
    fn test_journal_legacy_identity_fidelity_warning() -> Result<()> {
        let context = SharedValueContext::default();
        let mut entity = FileEntity::from_vec(b"payload".to_vec());
        entity.identity.relative_path = Some(PathBuf::from("entry"));
        let mut encoded = Vec::new();
        write_entity_payload(&mut encoded, &entity, &context)?;
        let marker = encoded.windows(8).position(|window| window == b"CTBMETA3")
            .context("Missing preservation record marker")?;
        let record_start = marker.checked_add(8).context("Preservation record offset overflow")?;
        let mut cursor = encoded.get(record_start..).context("Invalid preservation record offset")?;
        let mut record: PreservationRecord = serde_json::from_slice(&read_bytes(&mut cursor)?)?;
        record.identity_dc = None;
        let prefix = encoded.get(..marker).context("Invalid preservation record prefix")?;
        let oldest = read_entity_payload(prefix, current_platform(), &context)?;
        assert!(oldest.metadata.warnings.iter().any(|warning| warning.contains("Legacy journal lacks semantic identity")));
        let mut legacy = prefix.to_vec();
        legacy.extend_from_slice(b"CTBMETA2");
        write_bytes(&mut legacy, &serde_json::to_vec(&record)?)?;
        let decoded = read_entity_payload(&legacy, current_platform(), &context)?;
        assert!(decoded.metadata.warnings.iter().any(|warning| warning.contains("Legacy journal lacks semantic identity")));
        let mut invalid = prefix.to_vec();
        invalid.extend_from_slice(b"CTBMETA3");
        write_bytes(&mut invalid, &serde_json::to_vec(&record)?)?;
        assert!(read_entity_payload(&invalid, current_platform(), &context).is_err());
        Ok(())
    }

    #[crate::ctb_test]
    fn test_journal_semantic_identity_roundtrip() -> Result<()> {
        let context = SharedValueContext::default();
        for path in [None, Some(PathBuf::new()), Some(PathBuf::from("entry"))] {
            let mut entity = FileEntity::from_vec(b"payload".to_vec());
            entity.identity.relative_path = path;
            entity.identity.origin = FileOrigin::Archive {
                archive_path: PathBuf::from("source.tar"),
                entry_index: 3,
                archive_format: "tar".to_owned(),
            };
            let mut encoded = Vec::new();
            write_entity_payload(&mut encoded, &entity, &context)?;
            let decoded = read_entity_payload(&encoded, current_platform(), &context)?;
            assert_eq!(decoded.identity, entity.identity);
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_journal_conflicting_filename_rejected() -> Result<()> {
        let mut entity = FileEntity::from_vec(b"payload".to_vec());
        entity.identity.relative_path = Some(PathBuf::from("name"));
        let context = SharedValueContext::default();
        let mut encoded = Vec::new();
        write_entity_payload(&mut encoded, &entity, &context)?;
        let mut cursor = encoded.as_slice();
        assert_eq!(read_bytes(&mut cursor)?, b"name");
        let filename_length = usize::try_from(read_u32(&mut cursor)?)?;
        let filename_offset = encoded.len().checked_sub(cursor.len()).context("Invalid journal filename offset")?;
        for byte in encoded.iter_mut().skip(filename_offset).take(filename_length) {
            *byte = b'x';
        }
        let error = read_entity_payload(&encoded, current_platform(), &context)
            .expect_err("Conflicting identity fields must not be discarded");
        assert!(format!("{error:#}").contains("filename disagrees"));
        Ok(())
    }

    #[crate::ctb_test]
    fn test_journal_shared_environment_provenance() -> Result<()> {
        let temp = tempdir()?;
        let journal_path = temp.path().join("sharing.cscjournal");
        let mut writer = JournalWriter::create_at_path(&journal_path, &temp.path().join("sharing.cscdesc"), &[], temp.path())?;
        let environment = writer.environment.clone().context("Missing writer environment")?;
        let distinct = Arc::new(EnvDescription {
            os: "original-foreign-environment".to_owned(),
            ..EnvDescription::default()
        });
        let mut entity = FileEntity::from_vec(b"parent".to_vec());
        entity.identity.relative_path = Some(PathBuf::from("parent"));
        entity.metadata.environment = Some(Arc::clone(&environment));
        let mut child = FileEntity::from_vec(b"child".to_vec());
        child.metadata.environment = Some(Arc::clone(&distinct));
        entity.streams.push(AttachedStream { name: None, kind: StreamKind::MacOsResourceFork, entity: Box::new(child), data: None });
        entity.streams.push(AttachedStream { name: None, kind: StreamKind::MacOsResourceFork, entity: Box::new(FileEntity::from_vec(b"unknown".to_vec())), data: None });
        writer.record_entity(&entity);
        let mut unknown = FileEntity::from_vec(b"unknown".to_vec());
        unknown.identity.relative_path = Some(PathBuf::from("unknown"));
        writer.record_entity(&unknown);
        let mut equal = FileEntity::from_vec(b"equal".to_vec());
        equal.identity.relative_path = Some(PathBuf::from("equal"));
        equal.metadata.environment = Some(Arc::new((*environment).clone()));
        writer.record_entity(&equal);
        writer.commit_batch()?;
        let snapshot = read_journal_snapshot(&journal_path)?;
        let stored = snapshot.environment.as_ref().context("Missing persisted environment")?;
        let parent = snapshot.committed_entities.get(b"parent".as_slice()).context("Missing parent")?;
        assert!(parent.metadata.environment.as_ref().is_some_and(|value| Arc::ptr_eq(value, stored)));
        assert_eq!(parent.streams[0].entity.metadata.environment.as_ref(), Some(&distinct));
        assert!(parent.streams[1].entity.metadata.environment.is_none());
        assert!(snapshot.committed_entities.get(b"unknown".as_slice()).context("Missing unknown")?.metadata.environment.is_none());
        assert!(snapshot.committed_entities.get(b"equal".as_slice()).context("Missing equal")?.metadata.environment.as_ref().is_some_and(|value| Arc::ptr_eq(value, stored)));
        let mut payload = Vec::new();
        write_entity_payload(&mut payload, &entity, &writer.shared_context)?;
        assert!(read_entity_payload(&payload, current_platform(), &SharedValueContext::default()).is_err());
        Ok(())
    }

    #[crate::ctb_test]
    fn test_journal_legacy_resume_keeps_unknown_environment() -> Result<()> {
        let temp = tempdir()?;
        let journal_path = temp.path().join("legacy.cscjournal");
        let mut bytes = JOURNAL_MAGIC_V1.to_vec();
        bytes.extend_from_slice(&[TAG_SESSION_HEADER, current_platform()]);
        write_u32(&mut bytes, 0)?;
        write_bytes(&mut bytes, b"")?;
        bytes.push(TAG_JOB_COMPLETED);
        fs::write(&journal_path, bytes)?;
        let snapshot = read_journal_snapshot(&journal_path)?;
        let mut writer = JournalWriter::open_for_resume(&journal_path, &temp.path().join("legacy.cscdesc"), &snapshot)?;
        assert!(writer.environment.is_none());
        let mut entity = FileEntity::from_vec(b"unknown".to_vec());
        entity.identity.relative_path = Some(PathBuf::from("unknown"));
        writer.record_entity(&entity);
        writer.commit_batch()?;
        let snapshot = read_journal_snapshot(&journal_path)?;
        assert!(snapshot.environment.is_none());
        assert!(snapshot.committed_entities.get(b"unknown".as_slice()).context("Missing resumed entity")?.metadata.environment.is_none());
        Ok(())
    }

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
                warnings: Vec::new(),
                errors: Vec::new(),
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
                nlink: 2,
                hardlink_group: Some(Natural::from(12345_u64)),
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
                warnings: Vec::new(),
                errors: Vec::new(),
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
                warnings: Vec::new(),
                errors: Vec::new(),
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

    #[cfg(unix)]
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
        assert!(recorded.metadata.errors.iter().any(|error| error.starts_with("Materialization failed:")));
        assert!(snapshot.errors.iter().any(|error| matches!(error.stage, JournalErrorStage::Materialize)));
    }

    #[cfg(unix)]
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
                warnings: Vec::new(),
                errors: Vec::new(),
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
                environment: writer.environment.clone(),
                apple: None,
                warnings: Vec::new(),
                errors: Vec::new(),
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

        // 2. Read snapshot and verify the explicit environment reference is shared.
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

        let scope = ctb_io_environment::EnvironmentScope::enter_fresh();

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

        // 3. Calls within the same operation scope share the exact same Arc (pointer equality)
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
        drop(scope);

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

        let download_attempts = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let attempted = Arc::clone(&download_attempts);
        let _offline = ctb_utilities::https::crlite::set_test_download_override(move |_| {
            attempted.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            anyhow::bail!("Provenance test is deterministically offline")
        }).unwrap();
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
        assert!(download_attempts.load(std::sync::atomic::Ordering::Relaxed) > 0,
            "Full provenance must exercise network discovery through the offline transport");
    }

    /// Verifies authentic multi-revision journal test fixtures:
    /// - Revision 1 (CTBMETA1): Commit `795dfba07`
    /// - Revision 2 (CTBMETA2): Commit `15ce50fc5` / `c0bc7ec50` / `99d5548c4`
    /// - Revision 3 (CTBMETA3): Commit `bed55a2ba` / `14737998c`
    /// - Revision 4 (DcMixed): Live JournalWriter engine (CTBCSCJ\x04)
    #[crate::ctb_test]
    fn test_all_journal_revision_fixtures() -> Result<()> {
        let fixtures_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/journals");
        fs::create_dir_all(&fixtures_dir)?;

        let v4_journal = fixtures_dir.join("v4_dcmixed.cscjournal");
        let v4_desc = fixtures_dir.join("v4_dcmixed.cscdesc");
        if !v4_journal.exists() {
            let src = PathBuf::from("/source/project");
            let dest = PathBuf::from("/backup/project");
            let mut writer = JournalWriter::create_at_path(&v4_journal, &v4_desc, &[src], &dest)?;
            let mut dir_entity = FileEntity::from_vec(b"src".to_vec());
            dir_entity.kind = FileEntityKind::Directory;
            dir_entity.identity.relative_path = Some(PathBuf::from("src"));
            writer.record_entity(&dir_entity);

            let mut file_entity = FileEntity::from_vec(b"src/main.rs".to_vec());
            file_entity.kind = FileEntityKind::Regular {
                size: 1024,
                sha256: [0_u8; 32],
                is_sparse: false,
                extents: Vec::new(),
            };
            file_entity.identity.relative_path = Some(PathBuf::from("src/main.rs"));
            writer.record_entity(&file_entity);

            writer.commit_batch()?;
            writer.mark_completed()?;
        }

        // 1. Revision 1 (CTBMETA1)
        let s1 = read_journal_snapshot(&fixtures_dir.join("v1_ctbmeta1.cscjournal"))?;
        assert!(s1.is_completed);
        assert!(!s1.committed_entities.is_empty());

        // 2. Revision 2 (CTBMETA2)
        let s2 = read_journal_snapshot(&fixtures_dir.join("v2_ctbmeta2.cscjournal"))?;
        assert!(s2.is_completed);
        assert!(!s2.committed_entities.is_empty());

        // 3. Revision 3 (CTBMETA3)
        let s3 = read_journal_snapshot(&fixtures_dir.join("v3_ctbmeta3.cscjournal"))?;
        assert!(s3.is_completed);
        assert!(!s3.committed_entities.is_empty());

        // 4. Revision 4 (DcMixed)
        let s4 = read_journal_snapshot(&v4_journal)?;
        assert!(s4.is_completed);
        assert_eq!(s4.committed_entities.len(), 2);
        assert!(s4.environment.is_some());
        assert!(s4.committed_entities.contains_key(b"src".as_slice()));
        assert!(s4.committed_entities.contains_key(b"src/main.rs".as_slice()));

        Ok(())
    }
}


