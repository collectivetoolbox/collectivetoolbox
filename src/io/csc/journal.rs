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
use ctb_io::file::link_path::LinkPath;
use ctb_io::file::metadata::{FileFlag, FileMetadata, FileTimestamps};
use ctb_io::file::streams::{AttachedStream, StreamKind, StreamName};
use ctb_io::file::Natural;
use ctb_io_environment::EnvDescription;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) mod capture;
pub mod legacy;
pub(crate) mod scanner;
mod reader;
pub(crate) mod stream;

pub use reader::{JournalReader, JournalPayload};
pub use scanner::ReadMode;
pub use stream::{ARCHIVE_MAGIC, ARCHIVE_MAGIC_V1, ARCHIVE_MAGIC_V2, JournalMagic, JournalWriter};

pub use legacy::{
    convert_legacy_journal_bytes_to_v4, convert_legacy_journal_bytes_to_v4_with_policy,
    convert_legacy_journal_to_v4, convert_legacy_journal_to_v4_with_policy,
};

pub(crate) use legacy::{
    JOURNAL_MAGIC_V1, PreservationRecord, PreservedStream, TAG_BATCH_COMMIT,
    TAG_ENTITY, TAG_ERROR_RECORD, TAG_JOB_COMPLETED, TAG_SESSION_ENV,
    TAG_SESSION_HEADER, TAG_WARNING_RECORD, read_entity_payload,
    write_entity_payload,
};

pub const JOURNAL_MAGIC_V4: &[u8; 8] = b"CTBCSCJ\x04";
pub const JOURNAL_MAGIC_V5: &[u8; 8] = b"CTBCSCJ\x05";
pub const JOURNAL_MAGIC: &[u8; 8] = JOURNAL_MAGIC_V5;
pub const SESSION_ENVIRONMENT_BINDING: &str = "csc.session.environment";

pub(crate) fn session_context(session: &JournalSessionRecord) -> Result<SharedValueContext> {
    let mut context = SharedValueContext::default();
    if let Some(environment) = &session.environment {
        context.insert(SESSION_ENVIRONMENT_BINDING, Arc::clone(environment))?;
        context.insert("environment", Arc::clone(environment))?;
    }
    Ok(context)
}

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

impl JournalErrorRecord {
    /// Captures an operational failure without discarding its native cause.
    #[must_use]
    pub fn from_error(path: &Path, stage: JournalErrorStage, error: &anyhow::Error) -> Self {
        Self {
            path: path.to_path_buf(),
            raw_path: path.as_os_str().as_encoded_bytes().to_vec(),
            stage,
            error_message: format!("{error:#}"),
            os_error: error.downcast_ref::<std::io::Error>().and_then(std::io::Error::raw_os_error),
            timestamp_sec: sec_from_system_time(SystemTime::now()),
        }
    }
}

/// A persistent record of an operational warning or caveat stored in the journal.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct JournalWarningRecord {
    pub code: String,
    pub message: String,
    pub timestamp_sec: i64,
}

/// Persistent operation session record bounded by Dc 547..548.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    ctb_formats_dcstring::DcMixed,
)]
#[dc(begin = 547, end = 548)]
pub struct JournalSessionRecord {
    #[dc(549)]
    pub sources: Vec<PathBuf>,
    #[dc(550)]
    pub destination: PathBuf,
    #[dc(558)]
    pub origin_platform: u32,
    #[dc(nested = 340, shared = "environment")]
    pub environment: Option<Arc<EnvDescription>>,
}

fn system_time_from_sec(timestamp_sec: i64) -> std::time::SystemTime {
    let dur = std::time::Duration::from_secs(timestamp_sec.unsigned_abs());
    if timestamp_sec < 0 {
        match UNIX_EPOCH.checked_sub(dur) {
            Some(t) => t,
            // Reason for fallback: Pre-epoch timestamp underflowing SystemTime limits falls back to UNIX_EPOCH.
            None => UNIX_EPOCH,
        }
    } else {
        match UNIX_EPOCH.checked_add(dur) {
            Some(t) => t,
            // Reason for fallback: Post-epoch timestamp overflowing SystemTime limits falls back to UNIX_EPOCH.
            None => UNIX_EPOCH,
        }
    }
}

fn sec_from_system_time(time: std::time::SystemTime) -> i64 {
    match time.duration_since(UNIX_EPOCH) {
        Ok(dur) => match i64::try_from(dur.as_secs()) {
            Ok(s) => s,
            // Reason for fallback: Duration exceeding i64::MAX seconds clamps to i64::MAX.
            Err(_) => i64::MAX,
        },
        Err(err) => {
            let dur = err.duration();
            let secs = match i64::try_from(dur.as_secs()) {
                Ok(s) => s,
                // Reason for fallback: Duration before epoch exceeding i64::MAX seconds clamps to i64::MIN.
                Err(_) => return i64::MIN,
            };
            let neg_secs = secs.saturating_neg();
            if dur.subsec_nanos() == 0 {
                neg_secs
            } else {
                neg_secs.saturating_sub(1)
            }
        }
    }
}

/// Persistent error incident payload bounded by Dc 552..553.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    ctb_formats_dcstring::DcMixed,
)]
#[dc(begin = 552, end = 553)]
pub struct JournalErrorRecordPayload {
    #[dc(549)]
    pub path: PathBuf,
    #[dc(558)]
    pub packed_status: u64,
    #[dc(556)]
    pub error_message: String,
    #[dc(nested = 390)]
    pub timestamp: std::time::SystemTime,
}

impl JournalErrorRecordPayload {
    /// Uses the record's typed path and whole-second timestamp for v4 encoding.
    /// The separate `raw_path` field is not serialized by this format.
    ///
    /// # Errors
    /// Rejects `Some(0)`, which is indistinguishable from no native OS error.
    pub fn from_record(record: &JournalErrorRecord) -> Result<Self> {
        let stage_code = u64::from(record.stage.to_code());
        let os_code = match record.os_error {
            Some(error_code) => {
                anyhow::ensure!(error_code != 0, "Zero OS error code cannot be distinguished from an absent error in this journal format");
                u64::from(u32::from_le_bytes(error_code.to_le_bytes()))
            }
            None => 0_u64,
        };
        Ok(Self::new(
            record.path.clone(),
            (stage_code << 32) | os_code,
            record.error_message.clone(),
            record.timestamp_sec,
        ))
    }

    /// Reconstructs raw bytes from the decoded host path and floors the timestamp
    /// to whole seconds. Unknown byte-sized stages retain the traversal fallback.
    ///
    /// # Errors
    /// Rejects stages larger than one byte under the strict journal policy.
    pub fn to_record(
        &self,
        policy: &ctb_formats_dcstring::DcProjectionPolicy,
    ) -> Result<JournalErrorRecord> {
        let stage_code = match u8::try_from(self.packed_status >> 32) {
            Ok(code) => code,
            Err(_) => {
                anyhow::ensure!(!policy.is_strict(), "Invalid stage code in error record");
                0
            }
        };
        let os_code = u32::try_from(self.packed_status & 0xFFFF_FFFF)?;
        Ok(JournalErrorRecord {
            path: self.path.clone(),
            raw_path: self.path.as_os_str().as_encoded_bytes().to_vec(),
            stage: JournalErrorStage::from_code(stage_code),
            error_message: self.error_message.clone(),
            os_error: if os_code == 0 {
                None
            } else {
                Some(i32::from_le_bytes(os_code.to_le_bytes()))
            },
            timestamp_sec: self.timestamp_sec(),
        })
    }

    #[must_use]
    pub fn new(
        path: PathBuf,
        packed_status: u64,
        error_message: String,
        timestamp_sec: i64,
    ) -> Self {
        Self {
            path,
            packed_status,
            error_message,
            timestamp: system_time_from_sec(timestamp_sec),
        }
    }

    #[must_use]
    pub fn error_message(&self) -> &str {
        &self.error_message
    }

    #[must_use]
    pub fn timestamp_sec(&self) -> i64 {
        sec_from_system_time(self.timestamp)
    }
}

/// Persistent warning incident payload bounded by Dc 554..555.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    ctb_formats_dcstring::DcMixed,
)]
#[dc(begin = 554, end = 555)]
pub struct JournalWarningRecordPayload {
    #[dc(556)]
    pub message: String,
    #[dc(nested = 390)]
    pub timestamp: std::time::SystemTime,
}

impl JournalWarningRecordPayload {
    /// Combines the warning code and message using the journal's `": "` delimiter.
    #[must_use]
    pub fn from_record(record: &JournalWarningRecord) -> Self {
        Self::new(&record.code, &record.message, record.timestamp_sec)
    }

    /// Splits at the first `": "`, retaining later delimiters in the message.
    /// Warnings without a delimiter have an empty code.
    #[must_use]
    pub fn to_record(&self) -> JournalWarningRecord {
        let (code, message) = match self.message.split_once(": ") {
            Some((code, message)) => (code.to_string(), message.to_string()),
            None => (String::new(), self.message.clone()),
        };
        JournalWarningRecord {
            code,
            message,
            timestamp_sec: self.timestamp_sec(),
        }
    }

    #[must_use]
    pub fn new(code: &str, message: &str, timestamp_sec: i64) -> Self {
        Self {
            message: format!("{code}: {message}"),
            timestamp: system_time_from_sec(timestamp_sec),
        }
    }

    #[must_use]
    pub fn combined_message(&self) -> &str {
        &self.message
    }

    #[must_use]
    pub fn timestamp_sec(&self) -> i64 {
        sec_from_system_time(self.timestamp)
    }
}

/// Persistent warning incident payload with string-only log bounded by Dc 554..555.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    ctb_formats_dcstring::DcMixed,
)]
#[dc(begin = 554, end = 555)]
pub struct JournalWarningRecordStringPayload {
    #[dc(556)]
    pub message: String,
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

/// Durable identity receipt proving exclusive creation and authorization of a
/// destination directory across crash and resume sessions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppliedDirectoryReceipt {
    pub relative: PathBuf,
    pub identity: InodeKey,
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
    pub applied_directories: HashMap<PathBuf, InodeKey>,
}

impl JournalSnapshot {
    /// Returns true if the relative path (in raw bytes) has been committed.
    #[must_use]
    pub fn is_committed(&self, rel_bytes: &[u8]) -> bool {
        self.committed_entities.contains_key(rel_bytes)
    }
}

pub(crate) fn entity_requires_metadata_retention(entity: &FileEntity) -> bool {
    entity.metadata.native.is_some()
        || entity.metadata.timestamps.as_ref().and_then(|ts| ts.birthtime_sec).is_some()
        || entity.metadata.platform_raw_flags.is_some()
        || !entity.streams.is_empty()
        || !entity.metadata.warnings.is_empty()
        || !entity.metadata.errors.is_empty()
}

/// Writer managing the persistent state journal and its companion `.cscdesc` file.
pub struct StateJournal {
    journal_path: PathBuf,
    desc_path: PathBuf,
    writer: stream::JournalWriter<BufWriter<File>>,
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
    pending_descriptor_status: Option<String>,
    applied_directories: HashMap<PathBuf, InodeKey>,
}

impl StateJournal {
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

        let env = ctb_io_environment::capture_quick_arc();
        let session = JournalSessionRecord {
            sources: sources.to_vec(), destination: destination.to_path_buf(),
            origin_platform: u32::from(current_platform()), environment: Some(Arc::clone(&env)),
        };
        let mut writer = stream::JournalWriter::new(BufWriter::new(file), &session, stream::JournalMagic::JournalV5)?;
        Self::sync_output(writer.output_mut())?;

        let jw = Self {
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
            pending_descriptor_status: None,
            applied_directories: HashMap::new(),
        };

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


    /// Reopens a journal, preserving all committed bytes and completion markers.
    /// Previously completed journals explicitly begin a new writable session.
    /// Legacy journals (prior to `CSCJRNL5` / `DcMixed2`) are rejected with an error.
    pub fn open_for_resume(
        journal_path: &Path,
        desc_path: &Path,
        snapshot: &JournalSnapshot,
    ) -> Result<Self> {
        let mut header = [0_u8; 8 + ctb_formats_dcstring::DCMM_HEADER_LEN];
        {
            let mut f = File::open(journal_path).with_context(|| {
                format!("Failed to open state journal: {}", journal_path.display())
            })?;
            f.read_exact(&mut header).with_context(|| {
                format!("State journal header is too short to resume: {}", journal_path.display())
            })?;
        }
        ensure!(
            header.get(..8) == Some(JOURNAL_MAGIC_V5.as_slice())
                && header.get(8..13) == Some(&ctb_formats_dcstring::DCMM_MAGIC)
                && header.get(13..29) == Some(&ctb_formats_dcstring::DCMM_UUID),
            "Cannot resume a legacy CSC journal (expected CSCJRNL5 / DcMixed2): {}",
            journal_path.display()
        );

        let mut file = OpenOptions::new()
            .write(true)
            .open(journal_path)
            .with_context(|| {
                format!("Failed to reopen state journal: {}", journal_path.display())
            })?;
        let mut total_bytes = 0_u64;
        let mut total_files = 0_u64;
        for e in snapshot.committed_entities.values() {
            if let FileEntityKind::Regular { size, .. } = e.kind {
                total_bytes = total_bytes.saturating_add(size);
                total_files = total_files.saturating_add(1);
            }
        }

        file.set_len(snapshot.valid_length).context("Failed to discard uncommitted journal tail")?;
        file.sync_all()?;
        std::io::Seek::seek(&mut file, std::io::SeekFrom::End(0))?;
        let session = JournalSessionRecord {
            sources: snapshot.sources.clone(), destination: snapshot.destination.clone(),
            origin_platform: u32::from(snapshot.origin_platform), environment: snapshot.environment.clone(),
        };
        let writer = stream::JournalWriter::resume_output(BufWriter::new(file), &session,
            snapshot.last_batch_id, false)?;
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
            errors: snapshot.errors.clone(),
            warnings: snapshot.warnings.clone(),
            pending_descriptor_status: None,
            applied_directories: snapshot.applied_directories.clone(),
        })
    }

    /// Records an applied destination directory identity receipt for crash recovery.
    pub fn record_applied_directory(&mut self, relative: PathBuf, identity: InodeKey) {
        self.applied_directories.insert(relative, identity);
    }

    /// Read-only map of all applied destination directory identity receipts.
    #[must_use]
    pub fn applied_directories(&self) -> &HashMap<PathBuf, InodeKey> {
        &self.applied_directories
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

    fn sync_output(writer: &mut BufWriter<File>) -> Result<()> {
        writer.flush()?;
        writer.get_ref().sync_data()?;
        Ok(())
    }

    /// Queues a `FileEntity` for transactional commit.
    /// In copy mode, stream payloads are stripped so that `.cscjournal` records
    /// only stream metadata and checksums rather than duplicating stream bodies.
    /// A poisoned or completed writer retains the item; the next commit fails.
    pub fn record_entity(&mut self, entity: &FileEntity) {
        let mut entity = entity.clone();
        entity.strip_stream_bodies();
        self.uncommitted_entities.push(entity);
    }

    /// Commits all pending items to disk with an explicit fsync, advancing the transaction.
    pub fn commit_batch(&mut self) -> Result<()> {
        self.ensure_ready()?;
        if self.uncommitted_entities.is_empty() {
            return Ok(());
        }

        self.writer.write_batch(&self.uncommitted_entities, Self::sync_output)?;
        self.current_batch_id = self.writer.sequence();
        for entity in &self.uncommitted_entities {
            if let FileEntityKind::Regular { size, .. } = entity.kind {
                self.total_committed_files = self.total_committed_files.saturating_add(1);
                self.total_committed_bytes = self.total_committed_bytes.saturating_add(size);
            }
        }

        self.uncommitted_entities.clear();
        self.persist_descriptor("InProgress".to_owned())?;
        Ok(())
    }

    /// Marks the job as completely finished and updates the `.cscdesc` companion file.
    pub fn mark_completed(&mut self) -> Result<()> {
        self.commit_batch()?;
        self.writer.complete(Self::sync_output)?;
        if self.errors.is_empty() && !self.warnings.is_empty() {
            self.persist_descriptor(format!("Completed with {} warning(s)", self.warnings.len()))?;
        } else if self.errors.is_empty() {
            self.persist_descriptor("Completed".to_owned())?;
        } else {
            self.persist_descriptor(format!("Completed with {} error(s)", self.errors.len()))?;
        }
        Ok(())
    }

    /// Records an item or operational error directly into the state journal and descriptor.
    pub fn record_error(&mut self, error: &JournalErrorRecord) -> Result<()> {
        self.ensure_ready()?;
        self.writer.record_error_durable(error, Self::sync_output)?;
        self.errors.push(error.clone());
        self.persist_descriptor("InProgress".to_owned())?;
        Ok(())
    }

    /// Records a caveat or operational warning directly into the state journal and descriptor.
    pub fn record_warning(&mut self, code: &str, message: &str) -> Result<()> {
        self.ensure_ready()?;
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
        self.writer.record_warning_durable(&record, Self::sync_output)?;
        self.warnings.push(record);
        self.persist_descriptor("InProgress".to_owned())?;
        Ok(())
    }

    /// Marks the job as failed with a specified reason and updates `.cscdesc`.
    pub fn mark_failed(&mut self, reason: &str) -> Result<()> {
        self.commit_batch().with_context(|| format!("Failed to commit pending journal records while marking failure: {reason}"))?;
        self.writer.transaction(|_, _| Ok(()), Self::sync_output)?;
        let status = format!("Failed: {reason}");
        self.persist_descriptor(status)?;
        Ok(())
    }

    fn ensure_ready(&self) -> Result<()> {
        ensure!(self.pending_descriptor_status.is_none(),
            "Journal metadata repair pending; call retry_descriptor before writing");
        self.writer.ensure_ready()
    }

    /// Returns the descriptor status awaiting metadata-only repair.
    #[must_use]
    pub fn pending_descriptor_status(&self) -> Option<&str> {
        self.pending_descriptor_status.as_deref()
    }

    fn persist_descriptor(&mut self, status: String) -> Result<()> {
        self.pending_descriptor_status = Some(status);
        self.retry_descriptor()
    }

    /// Retries only a pending descriptor update, without writing journal bytes.
    /// Failure retains the pending status and blocks further journal writes.
    /// Success does not reopen a completed writer or permit duplicate completion.
    pub fn retry_descriptor(&mut self) -> Result<()> {
        let status = self.pending_descriptor_status.as_deref()
            .context("No journal descriptor repair is pending")?;
        self.update_desc_file(status)
            .context("Journal operation is durable but descriptor update failed; retry_descriptor is required")?;
        self.pending_descriptor_status = None;
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
                    ctb_formats_string::escape_path_human_readable(&err.path),
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
            writeln!(
                desc_text,
                "  - {}",
                ctb_formats_string::escape_path_human_readable(s)
            )?;
        }
        writeln!(
            desc_text,
            "Destination: {}",
            ctb_formats_string::escape_path_human_readable(&self.destination)
        )?;
        writeln!(desc_text, "NoatimeUsed: {}", self.noatime_used)?;
        if !self.applied_directories.is_empty() {
            writeln!(desc_text, "AppliedDirectories:")?;
            let mut sorted: Vec<_> = self.applied_directories.iter().collect();
            sorted.sort_by(|a, b| a.0.cmp(b.0));
            for (rel, id) in sorted {
                let receipt = AppliedDirectoryReceipt {
                    relative: rel.clone(),
                    identity: id.clone(),
                };
                let line = serde_json::to_string(&receipt)?;
                writeln!(desc_text, "  - {line}")?;
            }
        }

        ctb_io::file::write_file_durable(&self.desc_path, desc_text.as_bytes())
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
        for entry in ctb_io::file::read_dir_raw(path)? {
            let entry = entry?;
            let p = entry.path();
            let ext = p.extension().and_then(|e| e.to_str());
            if ext == Some("cscjournal") || ext == Some("cscarchive") {
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
        "Could not find valid .cscjournal or .cscarchive file from path: {}",
        path.display()
    )
}

/// Reads a state journal in strict mode, rolling back to the last valid batch or record.
pub fn read_journal_snapshot(path: &Path) -> Result<JournalSnapshot> {
    read_journal_snapshot_with_policy(
        path,
        &ctb_formats_dcstring::DcProjectionPolicy::strict(),
    )
}

/// Reads a state journal using the specified [`ctb_formats_dcstring::DcProjectionPolicy`],
/// rolling back to the last valid batch or record on crash truncation.
pub fn read_journal_snapshot_with_policy(
    path: &Path,
    policy: &ctb_formats_dcstring::DcProjectionPolicy,
) -> Result<JournalSnapshot> {
    let journal_path = resolve_journal_path(path)?;
    let mut file = File::open(&journal_path).with_context(|| {
        format!("Failed to open journal file: {}", journal_path.display())
    })?;

    let mut magic = [0_u8; 8];
    file.read_exact(&mut magic).context("Incomplete journal magic")?;

    let desc_path = journal_path.with_extension("cscdesc");
    let (noatime_used, applied_directories) = if desc_path.exists() {
        if let Ok(desc_content) = std::fs::read_to_string(&desc_path) {
            let noatime = desc_content.lines().any(|l| l.trim() == "NoatimeUsed: true");
            let mut dirs = HashMap::new();
            let mut in_applied = false;
            for line in desc_content.lines() {
                let trimmed = line.trim();
                if trimmed == "AppliedDirectories:" {
                    in_applied = true;
                    continue;
                }
                if in_applied {
                    if let Some(entry) = trimmed.strip_prefix("- ") {
                        if let Ok(receipt) = serde_json::from_str::<AppliedDirectoryReceipt>(entry) {
                            dirs.insert(receipt.relative, receipt.identity);
                        }
                    } else if !trimmed.is_empty() && !line.starts_with("  ") {
                        in_applied = false;
                    }
                }
            }
            (noatime, dirs)
        } else {
            (false, HashMap::new())
        }
    } else {
        (false, HashMap::new())
    };

    let mut snapshot = match &magic {
        m if m == JOURNAL_MAGIC_V5
            || m == JOURNAL_MAGIC_V4
            || m == ARCHIVE_MAGIC_V2
            || m == ARCHIVE_MAGIC_V1 =>
        {
            read_v4_journal_snapshot(&mut file, &journal_path, noatime_used, policy)?
        }
        m if m == JOURNAL_MAGIC_V1 => {
            let mut reader = BufReader::new(file);
            let mut snapshot = legacy::read_legacy_journal_snapshot(&mut reader, &journal_path, policy)?;
            for entity in snapshot.committed_entities.values() {
                warn_unexpected_stream_bodies(entity, &mut snapshot.warnings);
            }
            snapshot
        }
        _ => {
            anyhow::bail!(
                "Invalid or unrecognized journal magic in {}",
                journal_path.display()
            );
        }
    };
    snapshot.applied_directories = applied_directories;
    Ok(snapshot)
}

fn warn_unexpected_stream_bodies(entity: &FileEntity, warnings: &mut Vec<JournalWarningRecord>) {
    let mut pending = vec![entity];
    while let Some(current) = pending.pop() {
        for stream in &current.streams {
            if stream.data.is_some() || stream.payload.is_some() {
                let msg = format!(
                    "Journal snapshot for {:?} contains unexpected embedded stream body: {:?}",
                    entity.identity.relative_path,
                    stream.to_string_lossy()
                );
                warn_fmt!("{msg}");
                warnings.push(JournalWarningRecord {
                    code: "unexpected-journal-stream-body".into(),
                    message: msg,
                    timestamp_sec: 0,
                });
            }
            pending.push(&stream.entity);
        }
    }
}

/// Converts any in-memory cscjournal bytes (legacy or v4+) to a `DcMstring` document.
///
/// Legacy journal versions (revisions 1..=3) are converted to the Dc format (version 4+),
/// and then the 8 leading magic bytes are removed from the beginning of the version 4 data,
/// yielding a native `DcMixed` document.
pub fn convert_cscjournal_to_dcmstring(
    bytes: &[u8],
) -> Result<(DcMstring, ctb_formats_utilities::FormatLog)> {
    convert_cscjournal_to_dcmstring_with_policy(bytes, &ctb_formats_dcstring::DcProjectionPolicy::strict())
}

/// Converts journal bytes, using `policy` when projecting legacy Dc identities.
pub fn convert_cscjournal_to_dcmstring_with_policy(
    bytes: &[u8],
    policy: &ctb_formats_dcstring::DcProjectionPolicy,
) -> Result<(DcMstring, ctb_formats_utilities::FormatLog)> {
    anyhow::ensure!(bytes.len() >= 8, "CscJournal data is too short (< 8 bytes)");
    let magic = bytes
        .get(..8)
        .context("Slice bounds invalid for journal magic")?;

    let is_v4_or_newer = magic == JOURNAL_MAGIC_V5
        || magic == JOURNAL_MAGIC_V4
        || (magic.starts_with(b"CTBCSCJ") && magic.get(7).is_some_and(|&v| v >= 4))
        || magic == ARCHIVE_MAGIC_V2
        || magic == ARCHIVE_MAGIC_V1;

    let dcmm_bytes = if is_v4_or_newer {
        bytes
            .get(8..)
            .context("Slice bounds invalid for journal payload")?
    } else {
        let is_legacy = magic == JOURNAL_MAGIC_V1
            || (magic.starts_with(b"CTBCSCJ") && magic.get(7).is_some_and(|&v| v < 4));
        anyhow::ensure!(
            is_legacy,
            "Invalid or unrecognized cscjournal magic bytes: {magic:?}"
        );
        let v4_bytes = legacy::convert_legacy_journal_bytes_to_v4_with_policy(bytes, policy)?;
        return ctb_formats_dcstring::dc_mixed::dcmm_to_dcmstring(
            v4_bytes
                .get(8..)
                .context("Slice bounds invalid for converted journal payload")?,
        );
    };

    ctb_formats_dcstring::dc_mixed::dcmm_to_dcmstring(dcmm_bytes)
}

/// Converts in-memory cscjournal bytes to current DcMixed journal bytes (`CSCJRNL5`),
/// or returns version 4+ journal bytes unmodified.
pub fn convert_cscjournal_to_v4_bytes(bytes: &[u8]) -> Result<Vec<u8>> {
    convert_cscjournal_to_v4_bytes_with_policy(bytes, &ctb_formats_dcstring::DcProjectionPolicy::strict())
}

/// Converts journal bytes, using `policy` when projecting legacy Dc identities.
pub fn convert_cscjournal_to_v4_bytes_with_policy(
    bytes: &[u8],
    policy: &ctb_formats_dcstring::DcProjectionPolicy,
) -> Result<Vec<u8>> {
    anyhow::ensure!(bytes.len() >= 8, "CscJournal data is too short (< 8 bytes)");
    let magic = bytes
        .get(..8)
        .context("Slice bounds invalid for journal magic")?;
    if magic == JOURNAL_MAGIC_V5
        || magic == JOURNAL_MAGIC_V4
        || (magic.starts_with(b"CTBCSCJ") && magic.get(7).is_some_and(|&v| v >= 4))
        || magic == ARCHIVE_MAGIC_V2
        || magic == ARCHIVE_MAGIC_V1
    {
        Ok(bytes.to_vec())
    } else if magic == JOURNAL_MAGIC_V1
        || (magic.starts_with(b"CTBCSCJ") && magic.get(7).is_some_and(|&v| v < 4))
    {
        legacy::convert_legacy_journal_bytes_to_v4_with_policy(bytes, policy)
    } else {
        anyhow::bail!("Invalid or unrecognized cscjournal magic bytes: {magic:?}");
    }
}

/// Reads a v4 native DcMixed state journal using [`ctb_formats_dcstring::DcDocumentNavigator`]
/// without buffering file bodies. Member metadata is retained for projection.
pub fn read_v4_journal_snapshot(
    file: &mut File,
    journal_path: &Path,
    noatime_used: bool,
    policy: &ctb_formats_dcstring::DcProjectionPolicy,
) -> Result<JournalSnapshot> {
    let mut nav_file = file
        .try_clone()
        .with_context(|| format!("Failed to clone file handle for {}", journal_path.display()))?;
    nav_file.seek(SeekFrom::Start(0))?;
    let mut magic = [0_u8; 8];
    nav_file.read_exact(&mut magic)?;
    let is_archive = &magic == ARCHIVE_MAGIC_V2 || &magic == ARCHIVE_MAGIC_V1;

    let scanned = JournalReader::read(nav_file, scanner::ReadMode::Recover, policy)
        .with_context(|| format!("Invalid DcMixed journal in {}", journal_path.display()))?;
    let origin_platform = match u8::try_from(scanned.session.origin_platform) {
        Ok(platform) => platform,
        Err(_) => {
            anyhow::ensure!(!policy.is_strict(), "Invalid origin platform code in session header");
            current_platform()
        }
    };
    let mut committed_entities = HashMap::new();
    let mut warnings = scanned.warnings;
    for member in scanned.members {
        if !is_archive && member.payload.is_some() {
            let msg = format!(
                "Journal snapshot for {:?} contains unexpected embedded file body",
                member.entity.identity.relative_path
            );
            warn_fmt!("{msg}");
            warnings.push(JournalWarningRecord {
                code: "unexpected-journal-file-body".into(),
                message: msg,
                timestamp_sec: 0,
            });
        }
        let mut entity = member.entity;
        if !is_archive {
            warn_unexpected_stream_bodies(&entity, &mut warnings);
        } else {
            entity.strip_stream_bodies();
        }
        let path_key = match entity.identity.path_bytes() {
            Some(bytes) => bytes.to_vec(),
            None => Vec::new(),
        };
        committed_entities.insert(path_key, entity);
    }
    Ok(JournalSnapshot {
        sources: scanned.session.sources,
        destination: scanned.session.destination,
        origin_platform,
        committed_entities,
        is_completed: scanned.completed,
        last_batch_id: scanned.sequence,
        valid_length: scanned.valid_length,
        noatime_used,
        environment: scanned.session.environment,
        errors: scanned.errors,
        warnings,
        applied_directories: HashMap::new(),
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

    #[cfg(unix)]
    #[crate::ctb_test]
    fn journal_failed_commit_preserves_counters_queue_and_refuses_retry() -> Result<()> {
        for fault in ["encoding", "write", "flush", "sync"] {
            let temporary = tempdir()?;
            let journal = temporary.path().join("copy.cscjournal");
            let description = temporary.path().join("copy.cscdesc");
            let mut writer = StateJournal::create_at_path(&journal, &description, &[], temporary.path())?;
            let entity = FileEntity::from_vec(b"payload".to_vec());
            writer.record_entity(&entity);
            writer.commit_batch()?;
            writer.record_entity(&entity);
            let mut second = entity.clone();
            if fault == "encoding" {
                second.metadata.read_time = Some(UNIX_EPOCH.checked_sub(std::time::Duration::from_secs(1)).context("Test time underflow")?);
            }
            writer.record_entity(&second);
            match fault {
                "write" => {
                    let output = File::open(&journal)?;
                    writer.writer = writer.writer.map_output(|_| BufWriter::with_capacity(0, output));
                }
                "flush" => {
                    let output = File::open(&journal)?;
                    writer.writer = writer.writer.map_output(|_| BufWriter::new(output));
                }
                "sync" => {
                    let output = OpenOptions::new().write(true).open("/dev/null")?;
                    writer.writer = writer.writer.map_output(|_| BufWriter::new(output));
                }
                _ => {}
            }
            let description_before = fs::read(&description)?;
            ensure!(writer.commit_batch().is_err(), "Accepted {fault} failure");
            ensure!(writer.current_batch_id == 1 && writer.total_committed_files == 1 && writer.total_committed_bytes == 7);
            ensure!(writer.uncommitted_entities == vec![entity, second]);
            ensure!(writer.writer.is_failed());
            let bytes_after_failure = fs::read(&journal)?;
            ensure!(writer.commit_batch().is_err());
            ensure!(writer.mark_completed().is_err());
            ensure!(writer.record_warning("late", "must fail").is_err());
            ensure!(writer.mark_failed("failed commit").is_err());
            ensure!(writer.current_batch_id == 1 && writer.total_committed_files == 1 && writer.total_committed_bytes == 7);
            ensure!(writer.uncommitted_entities.len() == 2);
            ensure!(fs::read(&journal)? == bytes_after_failure && fs::read(&description)? == description_before);
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn journal_descriptor_failure_requires_metadata_only_repair() -> Result<()> {
        for operation in ["batch", "completion", "warning", "error", "failure"] {
            let temporary = tempdir()?;
            let journal = temporary.path().join("copy.cscjournal");
            let description = temporary.path().join("copy.cscdesc");
            let mut writer = StateJournal::create_at_path(&journal, &description, &[], Path::new(""))?;
            let mut entity = FileEntity::from_vec(b"payload".to_vec());
            entity.identity.relative_path = Some("payload".into());
            writer.record_entity(&entity);
            writer.commit_batch()?;
            let error = JournalErrorRecord::from_error(temporary.path(), JournalErrorStage::PayloadRead, &anyhow::anyhow!("injected"));
            if operation == "completion" {
                writer.record_error(&error)?;
            }
            fs::remove_file(&description)?;
            fs::create_dir(&description)?;
            let result = match operation {
                "batch" => {
                    entity.identity.relative_path = Some("second".into());
                    writer.record_entity(&entity);
                    writer.commit_batch()
                }
                "completion" => writer.mark_completed(),
                "warning" => writer.record_warning("injected", "warning"),
                "error" => writer.record_error(&error),
                _ => writer.mark_failed("injected"),
            };
            ensure!(format!("{:#}", result.expect_err("Descriptor failure must propagate")).contains("operation is durable"));
            let status = writer.pending_descriptor_status().context("Missing pending descriptor")?.to_owned();
            ensure!(status == match operation {
                "completion" => "Completed with 1 error(s)",
                "failure" => "Failed: injected",
                _ => "InProgress",
            });
            let bytes = fs::read(&journal)?;
            let counters = (writer.current_batch_id, writer.total_committed_files, writer.total_committed_bytes);
            ensure!(writer.uncommitted_entities.is_empty());
            ensure!(!writer.writer.is_failed());
            ensure!(writer.retry_descriptor().is_err());
            ensure!(writer.commit_batch().is_err());
            ensure!(writer.mark_completed().is_err());
            ensure!(writer.record_warning("blocked", "blocked").is_err());
            ensure!(writer.record_error(&error).is_err());
            ensure!(writer.mark_failed("blocked").is_err());
            ensure!(writer.pending_descriptor_status() == Some(status.as_str()));
            ensure!(fs::read(&journal)? == bytes);
            fs::remove_dir(&description)?;
            writer.retry_descriptor()?;
            ensure!(writer.pending_descriptor_status().is_none());
            ensure!(fs::read(&journal)? == bytes);
            ensure!((writer.current_batch_id, writer.total_committed_files, writer.total_committed_bytes) == counters);
            let descriptor = fs::read_to_string(&description)?;
            ensure!(descriptor.contains(&format!("Status: {status}\n")));
            ensure!(descriptor.contains(&format!("FilesCommitted: {}\nBytesCommitted: {}\n", counters.1, counters.2)));
            ensure!(descriptor.contains(&format!("ErrorsCount: {}\n", writer.errors().len())));
            ensure!(descriptor.contains(&format!("WarningsCount: {}\n", writer.warnings().len())));
            ensure!(writer.retry_descriptor().is_err());
            if operation == "completion" {
                ensure!(writer.mark_completed().is_err());
                ensure!(writer.record_warning("late", "late").is_err());
                ensure!(fs::read(&journal)? == bytes);
            } else {
                writer.mark_completed()?;
            }
            let snapshot = read_journal_snapshot(&journal)?;
            ensure!(snapshot.is_completed && snapshot.last_batch_id == counters.0);
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn journal_resume_preserves_completed_diagnostics_and_prefix() -> Result<()> {
        let temporary = tempdir()?;
        let journal = temporary.path().join("copy.cscjournal");
        let description = temporary.path().join("copy.cscdesc");
        let mut writer = StateJournal::create_at_path(&journal, &description, &[], Path::new(""))?;
        let mut entity = FileEntity::from_vec(b"original".to_vec());
        entity.identity.relative_path = Some("original".into());
        writer.record_entity(&entity);
        writer.mark_completed()?;
        drop(writer);
        let warning = JournalWarningRecord { code: "historical".into(), message: "warning".into(), timestamp_sec: 1 };
        let error = JournalErrorRecord::from_error(temporary.path(), JournalErrorStage::PayloadRead, &anyhow::anyhow!("historical"));
        let mut output = OpenOptions::new().append(true).open(&journal)?;
        let mut diagnostics = DcMstring::new();
        JournalWarningRecordPayload::from_record(&warning).encode_dc_mixed(&mut diagnostics)?;
        JournalErrorRecordPayload::from_record(&error)?.encode_dc_mixed(&mut diagnostics)?;
        output.write_all(diagnostics.payload_bytes())?;
        output.sync_all()?;
        drop(output);
        let prefix = fs::read(&journal)?;
        let snapshot = read_journal_snapshot(&journal)?;
        ensure!(snapshot.is_completed && snapshot.valid_length == u64::try_from(prefix.len())?);
        let mut resumed = StateJournal::open_for_resume(&journal, &description, &snapshot)?;
        ensure!(fs::read(&journal)? == prefix);
        entity.identity.relative_path = Some("resumed".into());
        resumed.record_entity(&entity);
        resumed.commit_batch()?;
        let snapshot = read_journal_snapshot(&journal)?;
        ensure!(!snapshot.is_completed && snapshot.last_batch_id == 2);
        ensure!(snapshot.is_committed(b"original") && snapshot.is_committed(b"resumed"));
        ensure!(snapshot.warnings.len() == 1 && snapshot.warnings.first() == Some(&warning));
        ensure!(snapshot.errors.len() == 1 && snapshot.errors.first() == Some(&error));
        resumed.mark_completed()?;
        ensure!(fs::read(&journal)?.starts_with(&prefix));
        let snapshot = read_journal_snapshot(&journal)?;
        ensure!(snapshot.is_completed && snapshot.errors.len() == 1 && snapshot.warnings.len() == 1);
        Ok(())
    }

    #[crate::ctb_test]
    fn journal_completion_refuses_reuse_and_explicit_resume_reopens() -> Result<()> {
        let temporary = tempdir()?;
        let journal = temporary.path().join("copy.cscjournal");
        let description = temporary.path().join("copy.cscdesc");
        let mut writer = StateJournal::create_at_path(&journal, &description, &[], temporary.path())?;
        writer.mark_completed()?;
        let completed = fs::read(&journal)?;
        ensure!(writer.mark_completed().is_err());
        ensure!(writer.record_warning("late", "must fail").is_err());
        let error = JournalErrorRecord::from_error(temporary.path(), JournalErrorStage::PayloadRead, &anyhow::anyhow!("late"));
        ensure!(writer.record_error(&error).is_err());
        let mut entity = FileEntity::from_vec(b"resumed".to_vec());
        entity.identity.relative_path = Some("resumed".into());
        writer.record_entity(&entity);
        ensure!(writer.commit_batch().is_err() && writer.mark_completed().is_err());
        ensure!(writer.uncommitted_entities.len() == 1 && writer.current_batch_id == 0);
        ensure!(fs::read(&journal)? == completed);
        drop(writer);
        let snapshot = read_journal_snapshot(&journal)?;
        ensure!(snapshot.is_completed);
        let mut resumed = StateJournal::open_for_resume(&journal, &description, &snapshot)?;
        ensure!(fs::read(&journal)? == completed);
        ensure!(read_journal_snapshot(&journal)?.is_completed);
        resumed.record_entity(&entity);
        resumed.commit_batch()?;
        ensure!(!read_journal_snapshot(&journal)?.is_completed);
        resumed.mark_completed()?;
        ensure!(fs::read(&journal)?.starts_with(&completed));
        let snapshot = read_journal_snapshot(&journal)?;
        ensure!(snapshot.is_completed && snapshot.last_batch_id == 1 && snapshot.is_committed(b"resumed"));
        Ok(())
    }

    #[crate::ctb_test]
    fn diagnostic_mark_failed_propagates_pending_encoding_failure() -> Result<()> {
        let temporary = tempdir()?;
        let source = temporary.path().join("source");
        let journal = temporary.path().join("copy.cscjournal");
        let description = temporary.path().join("copy.cscdesc");
        fs::write(&source, b"payload")?;
        let mut entity = FileEntity::from_filesystem(&source, None)?;
        entity.metadata.timestamps.as_mut().context("Missing captured timestamps")?.mtime_nsec = 1_000_000_000;
        let mut writer = StateJournal::create_at_path(&journal, &description,
            std::slice::from_ref(&source), temporary.path())?;
        let before = fs::read(&description)?;
        writer.record_entity(&entity);
        let error = writer.mark_failed("Injected operation failure").expect_err("Invalid pending record must fail");
        ensure!(format!("{error:#}").contains("Invalid file timestamp nanoseconds"));
        ensure!(writer.uncommitted_entities.len() == 1);
        ensure!(fs::read(&description)? == before, "Failed commit rewrote the descriptor as persisted status");
        Ok(())
    }

    #[crate::ctb_test]
    fn diagnostic_error_conversion() -> Result<()> {
        let policy = ctb_formats_dcstring::DcProjectionPolicy::strict();
        for stage in [
            JournalErrorStage::Traversal,
            JournalErrorStage::MetadataRead,
            JournalErrorStage::PayloadOpen,
            JournalErrorStage::PayloadRead,
            JournalErrorStage::Materialize,
            JournalErrorStage::PostCheck,
        ] {
            for os_error in [None, Some(13), Some(-1), Some(i32::MIN), Some(i32::MAX)] {
                for timestamp_sec in [-1, 0, 1] {
                    let record = JournalErrorRecord {
                        path: PathBuf::from("source"),
                        raw_path: b"source".to_vec(),
                        stage,
                        error_message: "OS error fidelity".to_owned(),
                        os_error,
                        timestamp_sec,
                    };
                    let payload = JournalErrorRecordPayload::from_record(&record)?;
                    assert_eq!(payload.to_record(&policy)?, record);
                    let os_code = match os_error {
                        Some(code) => u64::from(u32::from_le_bytes(code.to_le_bytes())),
                        None => 0,
                    };
                    let expected = JournalErrorRecordPayload::new(
                        record.path.clone(),
                        (u64::from(stage.to_code()) << 32) | os_code,
                        record.error_message.clone(),
                        timestamp_sec,
                    );
                    let mut actual_bytes = DcMstring::new();
                    payload.encode_dc_mixed(&mut actual_bytes)?;
                    let mut expected_bytes = DcMstring::new();
                    expected.encode_dc_mixed(&mut expected_bytes)?;
                    assert_eq!(actual_bytes.as_bytes(), expected_bytes.as_bytes());
                }
            }
        }
        let zero = JournalErrorRecord {
            path: PathBuf::from("source"),
            raw_path: b"source".to_vec(),
            stage: JournalErrorStage::Traversal,
            error_message: String::new(),
            os_error: Some(0),
            timestamp_sec: 0,
        };
        assert!(JournalErrorRecordPayload::from_record(&zero).is_err());
        Ok(())
    }

    #[crate::ctb_test]
    fn diagnostic_status_validation() -> Result<()> {
        let strict = ctb_formats_dcstring::DcProjectionPolicy::strict();
        let forgiving = ctb_formats_dcstring::DcProjectionPolicy::forgiving();
        for stage_code in [6_u64, 255, 256, u64::from(u32::MAX)] {
            let payload = JournalErrorRecordPayload::new(
                PathBuf::from("source"), stage_code << 32, String::new(), 0,
            );
            assert_eq!(payload.to_record(&strict).is_err(), stage_code > 255);
            assert_eq!(payload.to_record(&forgiving)?.stage, JournalErrorStage::Traversal);
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn diagnostic_warning_conversion() {
        for timestamp_sec in [-1, 0, 1] {
            let record = JournalWarningRecord {
                code: "metadata".to_owned(),
                message: "first: second: third".to_owned(),
                timestamp_sec,
            };
            let payload = JournalWarningRecordPayload::from_record(&record);
            assert_eq!(payload.message, "metadata: first: second: third");
            assert_eq!(payload.to_record(), record);
        }
        let payload = JournalWarningRecordPayload {
            message: "unsplit warning".to_owned(),
            timestamp: UNIX_EPOCH,
        };
        assert_eq!(payload.to_record(), JournalWarningRecord {
            code: String::new(),
            message: payload.message.clone(),
            timestamp_sec: 0,
        });
    }

    #[crate::ctb_test]
    fn diagnostic_timestamp_conversion() -> Result<()> {
        for (timestamp, timestamp_sec) in [
            (UNIX_EPOCH.checked_sub(std::time::Duration::from_millis(1)), -1),
            (UNIX_EPOCH.checked_add(std::time::Duration::from_millis(999)), 0),
        ] {
            let timestamp = timestamp.context("Test timestamp out of range")?;
            let error = JournalErrorRecordPayload {
                path: PathBuf::from("source"),
                packed_status: 0,
                error_message: String::new(),
                timestamp,
            };
            assert_eq!(error.to_record(&ctb_formats_dcstring::DcProjectionPolicy::strict())?.timestamp_sec, timestamp_sec);
            let warning = JournalWarningRecordPayload {
                message: "code: message".to_owned(),
                timestamp,
            };
            assert_eq!(warning.to_record().timestamp_sec, timestamp_sec);
        }
        Ok(())
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn diagnostic_non_utf8_path_conversion() -> Result<()> {
        use std::os::unix::ffi::OsStrExt;

        let raw_path = b"source/\xff".to_vec();
        let record = JournalErrorRecord {
            path: PathBuf::from(std::ffi::OsStr::from_bytes(&raw_path)),
            raw_path,
            stage: JournalErrorStage::PayloadOpen,
            error_message: "non-UTF-8 path".to_owned(),
            os_error: Some(-1),
            timestamp_sec: 0,
        };
        let payload = JournalErrorRecordPayload::from_record(&record)?;
        assert_eq!(payload.to_record(&ctb_formats_dcstring::DcProjectionPolicy::strict())?, record);
        let temp = tempdir()?;
        let journal_path = temp.path().join("raw-path.cscjournal");
        let mut writer = StateJournal::create_at_path(
            &journal_path, &journal_path.with_extension("cscdesc"),
            &[temp.path().join("source")], &temp.path().join("destination"),
        )?;
        writer.record_error(&record)?;
        assert_eq!(read_journal_snapshot(&journal_path)?.errors, vec![record]);
        Ok(())
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn diagnostic_cross_platform_path_conversion() -> Result<()> {
        use std::os::unix::ffi::OsStrExt;

        let raw_path = b"C:/source/\xed\xa0\x80".to_vec();
        let record = JournalErrorRecord {
            path: PathBuf::from(std::ffi::OsStr::from_bytes(&raw_path)),
            raw_path,
            stage: JournalErrorStage::PayloadRead,
            error_message: "Windows WTF-8 filename".to_owned(),
            os_error: Some(-1),
            timestamp_sec: -1,
        };
        let session = JournalSessionRecord {
            sources: vec![PathBuf::from("C:/source")],
            destination: PathBuf::from("D:/destination"),
            origin_platform: u32::from(PLATFORM_WINDOWS),
            environment: None,
        };
        let mut document = DcMstring::new();
        session.encode_dc_mixed(&mut document)?;
        JournalErrorRecordPayload::from_record(&record)?.encode_dc_mixed(&mut document)?;
        let (dcmm1_bytes, _) = ctb_formats_dcstring::dc_mixed::dcmstring_to_dcmm1(&document)?;
        for (magic, payload) in [
            (JOURNAL_MAGIC_V5, document.as_bytes()),
            (JOURNAL_MAGIC_V4, dcmm1_bytes.as_slice()),
        ] {
            let mut bytes = magic.to_vec();
            bytes.extend_from_slice(payload);
            let temp = tempdir()?;
            let journal_path = temp.path().join("windows-path.cscjournal");
            fs::write(&journal_path, bytes)?;
            let snapshot = read_journal_snapshot(&journal_path)?;
            assert_eq!(snapshot.origin_platform, PLATFORM_WINDOWS);
            assert_eq!(snapshot.errors, vec![record.clone()]);
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_journal_signed_os_errors_roundtrip() -> Result<()> {
        let temp = tempdir()?;
        let journal_path = temp.path().join("signed-errors.cscjournal");
        let mut writer = StateJournal::create_at_path(
            &journal_path, &journal_path.with_extension("cscdesc"),
            &[temp.path().join("source")], &temp.path().join("destination"),
        )?;
        for os_error in [None, Some(13), Some(-1), Some(i32::MIN), Some(i32::MAX)] {
            let record = JournalErrorRecord {
                path: PathBuf::from("source"),
                raw_path: b"source".to_vec(),
                stage: JournalErrorStage::MetadataRead,
                error_message: "OS error fidelity".to_owned(),
                os_error,
                timestamp_sec: 0,
            };
            writer.record_error(&record)?;
            let snapshot = read_journal_snapshot(&journal_path)?;
            assert_eq!(snapshot.errors.last(), Some(&record));
        }
        let mut zero_record = writer.errors().last().context("Missing recorded errors")?.clone();
        zero_record.os_error = Some(0);
        assert!(writer.record_error(&zero_record).is_err());
        assert_eq!(read_journal_snapshot(&journal_path)?.errors.len(), 5);
        Ok(())
    }

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
    fn test_journal_legacy_embedded_identity_obeys_policy() -> Result<()> {
        use ctb_formats_dcstring::DcProjectionPolicy;

        let context = SharedValueContext::default();
        let mut entity = FileEntity::from_vec(b"payload".to_vec());
        entity.identity.relative_path = Some(PathBuf::from("entry"));
        let mut encoded = Vec::new();
        write_entity_payload(&mut encoded, &entity, &context)?;
        let positive = read_entity_payload(&encoded, current_platform(), &context, &DcProjectionPolicy::strict())?;
        assert_eq!(positive.identity, entity.identity);

        let marker = encoded.windows(8).position(|window| window == b"CTBMETA3")
            .context("Missing preservation record marker")?;
        let record_start = marker.checked_add(8).context("Preservation record offset overflow")?;
        let mut cursor = encoded.get(record_start..).context("Invalid preservation record offset")?;
        let mut record: PreservationRecord = serde_json::from_slice(&read_bytes(&mut cursor)?)?;
        let mut missing_nlink = DcMstring::new();
        missing_nlink.push_char(ctb_formats_dcstring::DcChar::from_short(319));
        entity.identity.origin.encode_dc_mixed(&mut missing_nlink)?;
        missing_nlink.push_char(ctb_formats_dcstring::DcChar::from_short(337));
        entity.identity.relative_path.as_ref().context("Missing test path")?.encode_dc_mixed(&mut missing_nlink)?;
        missing_nlink.push_char(ctb_formats_dcstring::DcChar::from_short(320));
        record.identity_dc = Some(missing_nlink.as_bytes().to_vec());
        let mut malformed = encoded.get(..record_start).context("Invalid preservation prefix")?.to_vec();
        write_bytes(&mut malformed, &serde_json::to_vec(&record)?)?;
        assert!(read_entity_payload(&malformed, current_platform(), &context, &DcProjectionPolicy::strict()).is_err());
        let salvaged = read_entity_payload(&malformed, current_platform(), &context, &DcProjectionPolicy::forgiving())?;
        assert_eq!(salvaged.identity.relative_path, entity.identity.relative_path);
        assert_eq!(salvaged.identity.nlink, 0);

        let mut journal = JOURNAL_MAGIC_V1.to_vec();
        journal.extend_from_slice(&[TAG_SESSION_HEADER, current_platform()]);
        write_u32(&mut journal, 0)?;
        write_bytes(&mut journal, b"destination")?;
        journal.push(TAG_ENTITY);
        write_u32(&mut journal, u32::try_from(malformed.len())?)?;
        journal.extend_from_slice(&xxhash::xxhash3_64(&malformed));
        journal.extend_from_slice(&malformed);
        journal.push(TAG_BATCH_COMMIT);
        legacy::write_u64(&mut journal, 1)?;
        journal.extend_from_slice(&ctb_formats_checksum::sha256(&malformed));
        journal.push(TAG_JOB_COMPLETED);
        assert!(convert_cscjournal_to_v4_bytes(&journal).is_err());
        assert!(convert_cscjournal_to_dcmstring(&journal).is_err());
        assert!(convert_cscjournal_to_v4_bytes_with_policy(&journal, &DcProjectionPolicy::strict()).is_err());
        assert!(convert_cscjournal_to_dcmstring_with_policy(&journal, &DcProjectionPolicy::strict()).is_err());
        let converted = convert_cscjournal_to_v4_bytes_with_policy(&journal, &DcProjectionPolicy::forgiving())?;
        let temp = tempdir()?;
        let path = temp.path().join("salvaged.cscjournal");
        fs::write(&path, converted)?;
        let snapshot = read_journal_snapshot(&path)?;
        assert!(snapshot.is_completed);
        assert_eq!(snapshot.last_batch_id, 1);
        assert_eq!(snapshot.committed_entities.get(b"entry".as_slice()).context("Missing salvaged entity")?.identity, salvaged.identity);
        let (document, log) = convert_cscjournal_to_dcmstring_with_policy(&journal, &DcProjectionPolicy::forgiving())?;
        assert!(document.len() > ctb_formats_dcstring::DCMM_HEADER_LEN);
        assert!(!log.has_errors());
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
        let oldest = read_entity_payload(prefix, current_platform(), &context, &ctb_formats_dcstring::DcProjectionPolicy::strict())?;
        assert!(oldest.metadata.warnings.iter().any(|warning| warning.contains("Legacy journal lacks semantic identity")));
        let mut legacy = prefix.to_vec();
        legacy.extend_from_slice(b"CTBMETA2");
        write_bytes(&mut legacy, &serde_json::to_vec(&record)?)?;
        let decoded = read_entity_payload(&legacy, current_platform(), &context, &ctb_formats_dcstring::DcProjectionPolicy::strict())?;
        assert!(decoded.metadata.warnings.iter().any(|warning| warning.contains("Legacy journal lacks semantic identity")));
        let mut invalid = prefix.to_vec();
        invalid.extend_from_slice(b"CTBMETA3");
        write_bytes(&mut invalid, &serde_json::to_vec(&record)?)?;
        assert!(read_entity_payload(&invalid, current_platform(), &context, &ctb_formats_dcstring::DcProjectionPolicy::strict()).is_err());
        Ok(())
    }

    #[crate::ctb_test]
    fn test_legacy_raw_flags_normalized_only_at_journal_boundary() -> Result<()> {
        let context = SharedValueContext::default();
        let entity = FileEntity::from_vec(b"payload".to_vec());
        let mut encoded = Vec::new();
        write_entity_payload(&mut encoded, &entity, &context)?;
        let marker = encoded.windows(8).position(|window| window == b"CTBMETA3")
            .context("Missing preservation record marker")?;
        let record_start = marker.checked_add(8).context("Preservation record offset overflow")?;
        let mut cursor = encoded.get(record_start..).context("Invalid preservation record offset")?;
        let mut record: PreservationRecord = serde_json::from_slice(&read_bytes(&mut cursor)?)?;
        let flags = serde_json::json!({
            "source_os": "linux", "raw_value": u64::MAX, "has_unparsed_flags": true
        });
        record.metadata["platform_raw_flags"] = flags.clone();
        assert!(serde_json::from_value::<ctb_io::file::PlatformRawFlags>(flags).is_err());
        for magic in [b"CTBMETA1", b"CTBMETA2", b"CTBMETA3"] {
            let mut legacy = encoded.get(..marker).context("Invalid record prefix")?.to_vec();
            legacy.extend_from_slice(magic);
            write_bytes(&mut legacy, &serde_json::to_vec(&record)?)?;
            let decoded = read_entity_payload(&legacy, current_platform(), &context, &ctb_formats_dcstring::DcProjectionPolicy::strict())?;
            assert_eq!(decoded.metadata.platform_raw_flags,
                Some(ctb_io::file::PlatformRawFlags::from_raw(u64::MAX)));
        }
        record.metadata["platform_raw_flags"] = serde_json::json!({"raw_value": -1});
        let mut legacy = encoded.get(..marker).context("Invalid record prefix")?.to_vec();
        legacy.extend_from_slice(b"CTBMETA3");
        write_bytes(&mut legacy, &serde_json::to_vec(&record)?)?;
        assert!(read_entity_payload(&legacy, current_platform(), &context, &ctb_formats_dcstring::DcProjectionPolicy::strict()).is_err());
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
                archive_format: FormatId::Tar,
            };
            let mut encoded = Vec::new();
            write_entity_payload(&mut encoded, &entity, &context)?;
            let decoded = read_entity_payload(&encoded, current_platform(), &context, &ctb_formats_dcstring::DcProjectionPolicy::strict())?;
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
        let error = read_entity_payload(&encoded, current_platform(), &context, &ctb_formats_dcstring::DcProjectionPolicy::strict())
            .expect_err("Conflicting identity fields must not be discarded");
        assert!(format!("{error:#}").contains("filename disagrees"));
        Ok(())
    }

    #[crate::ctb_test]
    fn test_journal_shared_environment_provenance() -> Result<()> {
        let temp = tempdir()?;
        let journal_path = temp.path().join("sharing.cscjournal");
        let mut writer = StateJournal::create_at_path(&journal_path, &temp.path().join("sharing.cscdesc"), &[], temp.path())?;
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
        entity.streams.push(AttachedStream { name: None, kind: StreamKind::MacOsResourceFork, entity: Box::new(child), data: None, payload: None });
        entity.streams.push(AttachedStream { name: None, kind: StreamKind::MacOsResourceFork, entity: Box::new(FileEntity::from_vec(b"unknown".to_vec())), data: None, payload: None });
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
        write_entity_payload(&mut payload, &entity, writer.writer.context())?;
        assert!(read_entity_payload(&payload, current_platform(), &SharedValueContext::default(), &ctb_formats_dcstring::DcProjectionPolicy::strict()).is_err());
        Ok(())
    }

    #[crate::ctb_test]
    fn test_journal_legacy_resume_refused_and_v5_unknown_environment_preserved() -> Result<()> {
        let temp = tempdir()?;

        // 1. Legacy v1 journal must be refused by open_for_resume without modifying disk bytes
        let v1_path = temp.path().join("legacy_v1.cscjournal");
        let mut v1_bytes = JOURNAL_MAGIC_V1.to_vec();
        v1_bytes.extend_from_slice(&[TAG_SESSION_HEADER, current_platform()]);
        write_u32(&mut v1_bytes, 0)?;
        write_bytes(&mut v1_bytes, b"")?;
        v1_bytes.push(TAG_JOB_COMPLETED);
        fs::write(&v1_path, &v1_bytes)?;
        let v1_snapshot = read_journal_snapshot(&v1_path)?;
        assert!(
            StateJournal::open_for_resume(&v1_path, &temp.path().join("legacy_v1.cscdesc"), &v1_snapshot).is_err(),
            "Resuming a legacy v1 journal must fail with an error"
        );
        assert_eq!(fs::read(&v1_path)?, v1_bytes, "Refused v1 journal must remain unmodified");

        // 2. Legacy v4 (DcMixed1) journal must be refused by open_for_resume without modifying disk bytes
        let session = JournalSessionRecord {
            sources: Vec::new(),
            destination: PathBuf::new(),
            origin_platform: u32::from(current_platform()),
            environment: None,
        };
        let v5_bytes = stream::JournalWriter::new(Vec::new(), &session, stream::JournalMagic::JournalV5)?.finish()?;
        let v5_mstr = DcMstring::from_slice(v5_bytes.get(8..).context("Missing v5 payload")?)?;
        let (dcmm1_payload, _) = ctb_formats_dcstring::dc_mixed::dcmstring_to_dcmm1(&v5_mstr)?;
        let mut v4_bytes = JOURNAL_MAGIC_V4.to_vec();
        v4_bytes.extend_from_slice(&dcmm1_payload);
        let v4_path = temp.path().join("legacy_v4.cscjournal");
        fs::write(&v4_path, &v4_bytes)?;
        let v4_snapshot = read_journal_snapshot(&v4_path)?;
        assert!(
            StateJournal::open_for_resume(&v4_path, &temp.path().join("legacy_v4.cscdesc"), &v4_snapshot).is_err(),
            "Resuming a legacy v4 (DcMixed1) journal must fail with an error"
        );
        assert_eq!(fs::read(&v4_path)?, v4_bytes, "Refused v4 journal must remain unmodified");
        assert!(read_journal_snapshot(&v4_path)?.is_completed, "Refused v4 journal must remain readable");

        // Mismatched outer magic and inner DcMixed UUID must be rejected
        let mut mismatched_v4 = JOURNAL_MAGIC_V4.to_vec();
        mismatched_v4.extend_from_slice(v5_mstr.as_bytes());
        let mismatched_v4_path = temp.path().join("mismatched_v4.cscjournal");
        fs::write(&mismatched_v4_path, &mismatched_v4)?;
        assert!(read_journal_snapshot(&mismatched_v4_path).is_err());

        // 3. Modern v5 (DcMixed2) journal with environment: None preserves unknown environment across resume
        let v5_path = temp.path().join("modern_v5.cscjournal");
        fs::write(&v5_path, v5_bytes)?;
        let snapshot = read_journal_snapshot(&v5_path)?;
        let mut writer = StateJournal::open_for_resume(&v5_path, &temp.path().join("modern_v5.cscdesc"), &snapshot)?;
        assert!(writer.environment.is_none());
        let mut entity = FileEntity::from_vec(b"unknown".to_vec());
        entity.identity.relative_path = Some(PathBuf::from("unknown"));
        writer.record_entity(&entity);
        writer.commit_batch()?;
        let snapshot = read_journal_snapshot(&v5_path)?;
        assert!(snapshot.environment.is_none());
        assert!(snapshot.committed_entities.get(b"unknown".as_slice()).context("Missing resumed entity")?.metadata.environment.is_none());
        Ok(())
    }

    #[crate::ctb_test]
    fn test_journal_entity_format_and_checksum_resilience() {
        use crate::journal::{StateJournal, read_journal_snapshot};
        use ctb_io::file::entity::{FileEntity, FileEntityKind};
        use ctb_io::file::identity::{FileIdentity, FileOrigin};
        use ctb_io::file::metadata::{FileMetadata, FileTimestamps};
        use std::io::Write;

        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src");
        let dest = temp.path().join("dest");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&dest).expect("create dest");

        let mut writer = StateJournal::create_new(temp.path(), &[src.clone()], &dest).expect("create journal");

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
                target_relative_path: LinkPath::from_bytes(b"hello.txt", cfg!(windows)),
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
        file_entity.metadata.platform_raw_flags = Some(ctb_io::file::PlatformRawFlags::from_raw(u64::MAX));
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
            payload: None,
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
        let mut expected_file = file_entity.clone();
        expected_file.strip_stream_bodies();
        assert_eq!(read_file, &expected_file);
        for stream in &read_file.streams {
            assert_eq!(stream.data, None);
        }
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

        let malformed_journal = fs::read(writer.journal_path()).expect("read malformed journal");
        for policy in [
            ctb_formats_dcstring::DcProjectionPolicy::strict(),
            ctb_formats_dcstring::DcProjectionPolicy::forgiving(),
        ] {
            let error = read_journal_snapshot_with_policy(writer.journal_path(), &policy)
                .expect_err("Malformed closed records must not be silently discarded");
            assert!(format!("{error:#}").contains("Unsupported journal record"));
        }
        assert_eq!(fs::read(writer.journal_path()).expect("reread journal"), malformed_journal);
    }

    #[crate::ctb_test]
    fn test_journal_pre_epoch_read_time_rejected() {
        use crate::journal::StateJournal;
        use ctb_io::file::entity::{FileEntity, FileEntityKind};
        use ctb_io::file::identity::{FileIdentity, FileOrigin};
        use ctb_io::file::metadata::{FileMetadata, FileTimestamps};

        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src");
        let dest = temp.path().join("dest");
        std::fs::create_dir_all(&src).expect("create src");
        std::fs::create_dir_all(&dest).expect("create dest");

        let mut writer = StateJournal::create_new(temp.path(), &[src.clone()], &dest).expect("create journal");

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
            // Separate captures can cross a second boundary; readlink and
            // timestamp cleanup affect both components of atime/ctime.
            for name in ["statx.atime.sec", "statx.atime.nsec", "statx.ctime.sec", "statx.ctime.nsec"] {
                exp.values.remove(name);
                act.values.remove(name);
            }
        }
        assert_eq!(actual_native, expected_native);
        let mut expected_ts = original.metadata.timestamps;
        let mut actual_ts = recorded.metadata.timestamps;
        if let Some(ref mut ts) = expected_ts {
            ts.atime_sec = 0;
            ts.atime_nsec = 0;
            ts.ctime_sec = 0;
            ts.ctime_nsec = 0;
        }
        if let Some(ref mut ts) = actual_ts {
            ts.atime_sec = 0;
            ts.atime_nsec = 0;
            ts.ctime_sec = 0;
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
                true,
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
        use crate::journal::{StateJournal, read_journal_snapshot};
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
        let mut writer = StateJournal::create_at_path(
            &journal_path,
            &desc_path,
            &[src.clone()],
            &dest,
        ).expect("create_at_path");

        assert!(writer.environment.is_some(), "StateJournal should initialize environment");

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
        csc_args.overwrite = true;
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
    /// - Revision 1 (CTBMETA1): Commit `d90071e25` (last buildable commit before CTBMETA2 in 8a1f3786d)
    /// - Revision 2 (CTBMETA2): Commit `02070a495` (last buildable commit before CTBMETA3 in 14737998c)
    /// - Revision 3 (CTBMETA3): Commit `14737998c` (last buildable commit before DcMixed in a48081c47)
    /// - Revision 4 (DcMixed): Commit `HEAD` / `main` (CTBCSCJ\x04 with DcMM)
    #[crate::ctb_test]
    fn test_all_journal_revision_fixtures() -> Result<()> {
        let fixtures_dir = ctb_utilities::manifest_fixture_path(env!("CARGO_MANIFEST_DIR"), "fixtures/journals");

        let s1 = read_journal_snapshot(&fixtures_dir.join("v1_ctbmeta1-20261011.cscjournal"))
            .context("Failed to read revision 1 fixture")?;
        let s2 = read_journal_snapshot(&fixtures_dir.join("v2_ctbmeta2-20261011.cscjournal"))
            .context("Failed to read revision 2 fixture")?;
        let v3_fixture = fixtures_dir.join("v3_ctbmeta3-20261011.cscjournal");
        let strict_error = read_journal_snapshot(&v3_fixture)
            .expect_err("Historical untyped path literals must be rejected strictly");
        assert!(format!("{strict_error:#}").contains("missing Dc 262..263 type header"));
        let s3 = read_journal_snapshot_with_policy(
            &v3_fixture,
            &ctb_formats_dcstring::DcProjectionPolicy::forgiving(),
        ).context("Failed to recover revision 3 fixture with explicit forgiving policy")?;
        let v4_fixture = fixtures_dir.join("v4_dcmixed-20261011.cscjournal");
        assert!(
            read_journal_snapshot(&v4_fixture).is_err(),
            "Development-era malformed closed records must be rejected in strict mode"
        );
        let s4 = read_journal_snapshot_with_policy(
            &v4_fixture,
            &ctb_formats_dcstring::DcProjectionPolicy::forgiving(),
        )?;

        // Verify snapshots across all four format revisions
        for (rev_name, snap) in [
            ("v1_ctbmeta1", &s1),
            ("v2_ctbmeta2", &s2),
            ("v3_ctbmeta3", &s3),
            ("v4_dcmixed", &s4),
        ] {
            assert!(snap.is_completed, "{rev_name} must be completed");
            assert!(!snap.committed_entities.is_empty(), "{rev_name} must have entities");
            assert!(
                snap.committed_entities.contains_key(b"src/fixture_info.txt".as_slice())
                    || snap.committed_entities.contains_key(b"src\\fixture_info.txt".as_slice())
                    || snap.committed_entities.contains_key(b"src/main.rs".as_slice())
                    || snap.committed_entities.contains_key(b"src\\main.rs".as_slice()),
                "{rev_name} must contain payload files"
            );
        }

        assert!(s4.environment.is_some(), "v4_dcmixed must record environment");

        Ok(())
    }

    #[crate::ctb_test]
    fn test_journal_revision_rejects_retired_dcmm_types() -> Result<()> {
        let temp = tempdir()?;
        let path = temp.path().join("retired.cscjournal");
        for type_byte in [1, 4] {
            let mut payload = DcMstring::new().into_bytes();
            payload.extend_from_slice(&ctb_formats_dcstring::dc_mixed::DC_203_BYTES);
            payload.push(type_byte);
            let expected_offset = payload.len().saturating_sub(1);
            payload.extend_from_slice(&0u128.to_be_bytes());
            if type_byte == 4 {
                payload.extend_from_slice(&ctb_formats_checksum::sha256(b""));
            }
            let mut bytes = JOURNAL_MAGIC_V4.to_vec();
            bytes.extend_from_slice(&payload);
            fs::write(&path, &bytes)?;
            for error in [
                read_journal_snapshot(&path)
                    .err()
                    .context("Retired types must not be recovered as a truncated journal")?,
                convert_cscjournal_to_dcmstring(&bytes)
                    .err()
                    .context("Retired types must not be converted")?,
            ] {
                let cause = error
                    .downcast_ref::<ctb_formats_dcstring::DcMixedError>()
                    .context("Missing DcMixed error")?;
                assert_eq!(
                    cause,
                    &ctb_formats_dcstring::DcMixedError::UnsupportedLegacyType {
                        offset: expected_offset,
                        type_byte,
                    },
                );
            }
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_journal_conversion_all_revisions_to_v4_and_dcmstring() -> Result<()> {
        let fixtures_dir = ctb_utilities::manifest_fixture_path(
            env!("CARGO_MANIFEST_DIR"),
            "fixtures/journals",
        );

        let temp = tempfile::tempdir()?;

        // Test converting legacy revisions 1..=3 to v4 bytes and validating snapshot parity
        for (rev_name, filename, policy) in [
            ("v1_ctbmeta1", "v1_ctbmeta1-20261011.cscjournal", ctb_formats_dcstring::DcProjectionPolicy::strict()),
            ("v2_ctbmeta2", "v2_ctbmeta2-20261011.cscjournal", ctb_formats_dcstring::DcProjectionPolicy::strict()),
            ("v3_ctbmeta3", "v3_ctbmeta3-20261011.cscjournal", ctb_formats_dcstring::DcProjectionPolicy::forgiving()),
        ] {
            let legacy_path = fixtures_dir.join(filename);
            let legacy_bytes = fs::read(&legacy_path)?;
            let orig_snap = read_journal_snapshot_with_policy(&legacy_path, &policy)
                .with_context(|| format!("Failed to read {rev_name} fixture before conversion"))?;

            let v4_bytes = convert_cscjournal_to_v4_bytes_with_policy(&legacy_bytes, &policy)?;
            assert!(
                v4_bytes.starts_with(JOURNAL_MAGIC),
                "{rev_name} converted bytes must start with JOURNAL_MAGIC"
            );

            let converted_v4_path = temp.path().join(format!("{rev_name}_converted.cscjournal"));
            fs::write(&converted_v4_path, &v4_bytes)?;

            let v4_snap = read_journal_snapshot(&converted_v4_path)?;
            assert_eq!(
                v4_snap.is_completed, orig_snap.is_completed,
                "{rev_name}: completed status must match"
            );
            assert_eq!(
                v4_snap.committed_entities.len(),
                orig_snap.committed_entities.len(),
                "{rev_name}: entity count must match"
            );
            assert_eq!(
                v4_snap.origin_platform, orig_snap.origin_platform,
                "{rev_name}: origin platform must match"
            );
            assert_eq!(
                v4_snap.destination, orig_snap.destination,
                "{rev_name}: destination must match"
            );
            for key in orig_snap.committed_entities.keys() {
                assert!(
                    v4_snap.committed_entities.contains_key(key),
                    "{rev_name}: converted v4 journal must retain entity {key:?}"
                );
            }
        }

        // Test converting all 4 revisions directly to DcMstring
        for (filename, policy) in [
            ("v1_ctbmeta1-20261011.cscjournal", ctb_formats_dcstring::DcProjectionPolicy::strict()),
            ("v2_ctbmeta2-20261011.cscjournal", ctb_formats_dcstring::DcProjectionPolicy::strict()),
            ("v3_ctbmeta3-20261011.cscjournal", ctb_formats_dcstring::DcProjectionPolicy::forgiving()),
            ("v4_dcmixed-20261011.cscjournal", ctb_formats_dcstring::DcProjectionPolicy::forgiving()),
        ] {
            let bytes = fs::read(fixtures_dir.join(filename))?;
            let (doc, log) = convert_cscjournal_to_dcmstring_with_policy(&bytes, &policy)?;
            assert!(
                doc.len() > ctb_formats_dcstring::DCMM_HEADER_LEN,
                "{filename}: DcMstring document must have payload beyond 21-byte header"
            );
            assert!(!log.has_errors(), "{filename}: conversion should not log errors");
        }

        Ok(())
    }

    #[crate::ctb_test]
    fn test_copy_journal_strips_attached_stream_bodies() -> Result<()> {
        let temp = tempdir()?;
        let journal_path = temp.path().join("copy.cscjournal");
        let desc_path = temp.path().join("copy.cscdesc");
        let source = temp.path().join("source.bin");
        let destination = temp.path().join("dest.bin");

        let mut journal = StateJournal::create_at_path(
            &journal_path,
            &desc_path,
            std::slice::from_ref(&source),
            &destination,
        )?;

        let mut entity = FileEntity::from_vec(b"primary content".to_vec());
        entity.identity.relative_path = Some("source.bin".into());

        let stream_payload = b"large stream payload that should not be in journal".to_vec();
        let expected_size = u64::try_from(stream_payload.len())?;
        let expected_hash = ctb_formats_checksum::sha256(&stream_payload);

        let stream = AttachedStream::from_data(
            Some(StreamName::from_str("user.test_stream")),
            StreamKind::ExtendedAttribute,
            stream_payload.clone(),
        )?;
        entity.streams.push(stream);

        journal.record_entity(&entity);
        journal.commit_batch()?;

        // Verify on-disk .cscjournal does not contain the stream body bytes
        let journal_bytes = fs::read(&journal_path)?;
        assert!(
            !journal_bytes.windows(stream_payload.len()).any(|w| w == stream_payload.as_slice()),
            "Copy mode .cscjournal must not contain stream payload bodies"
        );

        // Verify reading snapshot restores metadata and digests but stream.data is None
        let snapshot = read_journal_snapshot(&journal_path)?;
        let recorded = snapshot.committed_entities.values().next().context("Missing committed entity")?;
        assert_eq!(recorded.streams.len(), 1);
        assert_eq!(recorded.streams[0].name, Some(StreamName::from_str("user.test_stream")));
        assert_eq!(recorded.streams[0].kind, StreamKind::ExtendedAttribute);
        assert_eq!(recorded.streams[0].data, None, "Stream data must be stripped in journal snapshot");

        // Metadata and checksums are preserved on stream.entity
        if let FileEntityKind::Regular { size, sha256, .. } = &recorded.streams[0].entity.kind {
            assert_eq!(*size, expected_size);
            assert_eq!(*sha256, expected_hash);
        } else {
            panic!("Expected regular stream entity kind");
        }

        Ok(())
    }

    #[crate::ctb_test]
    fn test_journal_durable_applied_directory_receipts() -> Result<()> {
        let temp = tempdir()?;
        let journal_path = temp.path().join("applied.cscjournal");
        let desc_path = temp.path().join("applied.cscdesc");
        let source = temp.path().join("src");
        let destination = temp.path().join("dst");

        let mut journal = StateJournal::create_at_path(
            &journal_path,
            &desc_path,
            std::slice::from_ref(&source),
            &destination,
        )?;

        let rel1 = PathBuf::from("dir1");
        let id1 = InodeKey::new(12345, 67890_u64);
        let rel2 = PathBuf::from("dir1/nested");
        let id2 = InodeKey::new(12345, 67891_u64);

        journal.record_applied_directory(rel1.clone(), id1.clone());
        journal.record_applied_directory(rel2.clone(), id2.clone());

        let mut dummy = FileEntity::from_vec(b"dummy".to_vec());
        dummy.identity.relative_path = Some("dir1/file.txt".into());
        journal.record_entity(&dummy);
        journal.commit_batch()?;

        // Verify .cscdesc contains AppliedDirectories section
        let desc_content = fs::read_to_string(&desc_path)?;
        assert!(desc_content.contains("AppliedDirectories:"));

        // Snapshot reading restores applied_directories
        let snapshot = read_journal_snapshot(&journal_path)?;
        assert_eq!(snapshot.applied_directories.len(), 2);
        assert_eq!(snapshot.applied_directories.get(&rel1), Some(&id1));
        assert_eq!(snapshot.applied_directories.get(&rel2), Some(&id2));

        // Reopening journal for resume retains applied_directories
        let resumed = StateJournal::open_for_resume(&journal_path, &desc_path, &snapshot)?;
        assert_eq!(resumed.applied_directories().len(), 2);
        assert_eq!(resumed.applied_directories().get(&rel1), Some(&id1));
        assert_eq!(resumed.applied_directories().get(&rel2), Some(&id2));

        Ok(())
    }

    #[crate::ctb_test]
    fn test_128_bit_file_id_applied_directory_receipt() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let journal_path = temp_dir.path().join("test_128.cscjournal");
        let desc_path = temp_dir.path().join("test_128.cscdesc");
        let source = temp_dir.path().join("source");
        let destination = temp_dir.path().join("destination");

        let mut journal = StateJournal::create_at_path(
            &journal_path,
            &desc_path,
            std::slice::from_ref(&source),
            &destination,
        )?;

        // 128-bit file ID (e.g. ReFS or 128-bit NTFS FILE_ID_128)
        let file_id_128: u128 = 0x0123_4567_89ab_cdef_fedc_ba98_7654_3210;
        let volume_64: u64 = 0xfeed_cafe_dead_beef;
        let id_128 = InodeKey::from_file_id_128(volume_64, file_id_128);
        assert_eq!(id_128.as_u128(), Some(file_id_128));

        let rel = PathBuf::from("refs_dir");
        journal.record_applied_directory(rel.clone(), id_128.clone());

        let mut dummy = FileEntity::from_vec(b"test".to_vec());
        dummy.identity.relative_path = Some("refs_dir/file.txt".into());
        journal.record_entity(&dummy);
        journal.commit_batch()?;

        // Verify .cscdesc contains 128-bit number
        let desc_content = fs::read_to_string(&desc_path)?;
        assert!(desc_content.contains("AppliedDirectories:"));

        // Snapshot reading restores exact 128-bit identity
        let snapshot = read_journal_snapshot(&journal_path)?;
        let restored_identity = snapshot.applied_directories.get(&rel)
            .context("Missing restored directory receipt")?;
        assert_eq!(restored_identity, &id_128);
        assert_eq!(restored_identity.as_u128(), Some(file_id_128));

        Ok(())
    }

    #[crate::ctb_test]
    fn test_copy_journal_warns_on_unexpected_stream_body() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let journal_path = temp_dir.path().join("warn_test.cscjournal");
        let desc_path = temp_dir.path().join("warn_test.cscdesc");
        let source = temp_dir.path().join("source");
        let destination = temp_dir.path().join("destination");

        let mut journal = StateJournal::create_at_path(
            &journal_path,
            &desc_path,
            std::slice::from_ref(&source),
            &destination,
        )?;

        // Manually inject an entity that retains an in-memory stream body into the record queue
        let mut entity = FileEntity::from_vec(b"hello".to_vec());
        entity.identity.relative_path = Some("file_with_stream.txt".into());
        let mut stream = AttachedStream::from_data(
            Some(StreamName::from_str("user.stream")),
            StreamKind::ExtendedAttribute,
            b"embedded_stream_body".to_vec(),
        )?;
        stream.entity.streams.push(AttachedStream::from_data(
            Some(StreamName::from_str("user.nested")),
            StreamKind::ExtendedAttribute,
            b"nested_stream_body".to_vec(),
        )?);
        entity.streams.push(stream);

        // Bypass copy-mode stripping to exercise both primary and stream bodies.
        let mut payload = ctb_io::file::MemoryPayloadSource::new(b"hello".to_vec())?;
        journal.writer.write_entity(&entity, Some(&mut payload))?;
        StateJournal::sync_output(journal.writer.output_mut())?;

        // Reading snapshot must emit and record a warning for the unexpected stream body
        let snapshot = read_journal_snapshot(&journal_path)?;
        assert!(
            snapshot.warnings.iter().any(|w| w.code == "unexpected-journal-stream-body"),
            "Expected unexpected-journal-stream-body warning in copy journal snapshot"
        );
        assert_eq!(snapshot.warnings.iter().filter(|warning| warning.code == "unexpected-journal-stream-body").count(), 2);
        assert_eq!(snapshot.warnings.iter().filter(|warning| warning.code == "unexpected-journal-file-body").count(), 1);
        let direct = read_v4_journal_snapshot(
            &mut File::open(&journal_path)?,
            &journal_path,
            false,
            &ctb_formats_dcstring::DcProjectionPolicy::strict(),
        )?;
        assert_eq!(direct.warnings, snapshot.warnings);

        Ok(())
    }
}
