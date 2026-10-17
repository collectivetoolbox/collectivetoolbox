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

//! Read-only legacy journal parser supporting revisions 1, 2, and 3
//! (`CTBMETA1`, `CTBMETA2`, `CTBMETA3`) with lossless in-memory conversion
//! to standard Document Characters and `FileEntity` records.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use ctb_formats_checksum::{Sha256Stream, xxhash};
use ctb_formats_dcstring::{
    DcMixedEncode, DcMstring, DcProjectionPolicy, SharedJson,
    SharedValueContext,
};
use ctb_io::file::entity::{FileEntity, FileEntityKind};
use ctb_io::file::identity::{
    FileIdentity, FileOrigin, InodeKey, resolve_relative_path_for_os,
};
use ctb_io::file::link_path::LinkPath;
use ctb_io::file::metadata::{FileFlag, FileMetadata, FileTimestamps};
use ctb_io::file::streams::{AttachedStream, StreamKind, StreamName};
use ctb_io::file::Natural;
use ctb_io_environment::EnvDescription;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Cursor, Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::UNIX_EPOCH;

use super::{
    JournalErrorRecord, JournalErrorRecordPayload, JournalSessionRecord,
    JournalSnapshot, JournalWarningRecord, JournalWarningRecordPayload,
    JOURNAL_MAGIC_V4, PLATFORM_WINDOWS, SESSION_ENVIRONMENT_BINDING,
};

pub const JOURNAL_MAGIC_V1: &[u8; 8] = b"CTBCSCJ\x01";

pub const TAG_SESSION_HEADER: u8 = 1;
pub const TAG_ENTITY: u8 = 2;
pub const TAG_BATCH_COMMIT: u8 = 3;
pub const TAG_JOB_COMPLETED: u8 = 4;
pub const TAG_SESSION_ENV: u8 = 5;
pub const TAG_ERROR_RECORD: u8 = 6;
pub const TAG_WARNING_RECORD: u8 = 7;

/// Reads a legacy state journal (revision 1..=3) from disk into a `JournalSnapshot`.
pub fn read_legacy_journal_snapshot<R: Read + Seek>(
    reader: &mut BufReader<R>,
    journal_path: &Path,
    policy: &DcProjectionPolicy,
) -> Result<JournalSnapshot> {
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
    let mut shared_context = SharedValueContext::default();
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    let mut tag_buf = [0_u8; 1];
    while reader.read_exact(&mut tag_buf).is_ok() {
        match tag_buf[0] {
            TAG_SESSION_HEADER => {
                let Ok(plat) = read_u8(reader) else {
                    break;
                };
                origin_platform = plat;

                let Ok(src_count) = read_u32(reader) else {
                    break;
                };
                let mut valid_sources = true;
                for _ in 0..src_count {
                    if let Ok(bytes) = read_bytes(reader) {
                        let is_windows = origin_platform == PLATFORM_WINDOWS;
                        let src_path =
                            resolve_relative_path_for_os(&bytes, is_windows)?;
                        sources.push(src_path);
                    } else {
                        valid_sources = false;
                        break;
                    }
                }
                if !valid_sources {
                    break;
                }
                let Ok(dest_bytes) = read_bytes(reader) else {
                    break;
                };
                let is_windows = origin_platform == PLATFORM_WINDOWS;
                destination =
                    resolve_relative_path_for_os(&dest_bytes, is_windows)?;
                valid_length = reader.stream_position()?;
            }
            TAG_SESSION_ENV => {
                let Ok(env_bytes) = read_bytes(reader) else {
                    break;
                };
                let json_str = std::str::from_utf8(&env_bytes)
                    .context("Invalid journal environment UTF-8")?;
                let env_desc = EnvDescription::from_json(json_str)
                    .context("Invalid journal environment JSON")?;
                let definition = Arc::new(env_desc);
                shared_context.insert(
                    SESSION_ENVIRONMENT_BINDING,
                    Arc::clone(&definition),
                )?;
                environment = Some(definition);
                valid_length = reader.stream_position()?;
            }
            TAG_ENTITY => {
                is_completed = false;
                let Ok(payload_len) = read_u32(reader) else {
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

                match read_entity_payload(
                    &payload[..],
                    origin_platform,
                    &shared_context,
                    policy,
                ) {
                    Ok(entity) => {
                        // Reason for fallback: streams or entities without relative path use empty key in journal table
                        let path_key = entity
                            .identity
                            .path_bytes()
                            .map_or_else(Vec::new, |b| b.to_vec());
                        pending_entities.insert(path_key, entity);
                    }
                    Err(error) => {
                        return Err(error).context(
                            "Checksummed journal entity cannot be decoded without losing metadata",
                        );
                    }
                }
            }
            TAG_BATCH_COMMIT => {
                let Ok(batch_id) = read_u64(reader) else {
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
                anyhow::ensure!(
                    pending_entities.is_empty(),
                    "Completion marker precedes batch commit"
                );
                is_completed = true;
                valid_length = reader.stream_position()?;
            }
            TAG_ERROR_RECORD => {
                let Ok(err_bytes) = read_bytes(reader) else {
                    break;
                };
                if let Ok(json_str) = std::str::from_utf8(&err_bytes) {
                    if let Ok(record) =
                        serde_json::from_str::<JournalErrorRecord>(json_str)
                    {
                        errors.push(record);
                    }
                }
                valid_length = reader.stream_position()?;
            }
            TAG_WARNING_RECORD => {
                let Ok(warn_bytes) = read_bytes(reader) else {
                    break;
                };
                if let Ok(json_str) = std::str::from_utf8(&warn_bytes) {
                    if let Ok(record) =
                        serde_json::from_str::<JournalWarningRecord>(json_str)
                    {
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

/// Converts a legacy state journal (revisions 1..=3) to the v4 Dc format (`CTBCSCJ\x04`).
/// Embedded Dc identities are decoded strictly.
pub fn convert_legacy_journal_to_v4<R: Read + Seek, W: Write>(
    reader: &mut BufReader<R>,
    writer: &mut W,
) -> Result<()> {
    convert_legacy_journal_to_v4_with_policy(reader, writer, &DcProjectionPolicy::strict())
}

/// Converts a legacy journal with an explicit embedded Dc projection policy.
pub fn convert_legacy_journal_to_v4_with_policy<R: Read + Seek, W: Write>(
    reader: &mut BufReader<R>,
    writer: &mut W,
    policy: &DcProjectionPolicy,
) -> Result<()> {
    // If the reader is at the start of the stream, consume and verify legacy magic
    if reader.stream_position()? == 0 {
        let mut magic = [0_u8; 8];
        reader.read_exact(&mut magic).context("Incomplete journal magic")?;
        anyhow::ensure!(
            &magic == JOURNAL_MAGIC_V1,
            "Expected legacy journal magic CTBCSCJ\\x01"
        );
    }

    writer.write_all(JOURNAL_MAGIC_V4)?;
    let header = DcMstring::new();
    writer.write_all(header.as_bytes())?;

    let mut origin_platform = 0_u8;
    let mut sources = Vec::new();
    let mut destination = PathBuf::new();
    let mut environment: Option<Arc<EnvDescription>> = None;
    let mut shared_context = SharedValueContext::default();

    let mut pending_entities = Vec::new();
    let mut batch_hasher = Sha256Stream::new();
    let mut header_written = false;

    let mut tag_buf = [0_u8; 1];
    while reader.read_exact(&mut tag_buf).is_ok() {
        match tag_buf[0] {
            TAG_SESSION_HEADER => {
                let Ok(plat) = read_u8(reader) else {
                    break;
                };
                origin_platform = plat;

                let Ok(src_count) = read_u32(reader) else {
                    break;
                };
                let mut valid_sources = true;
                for _ in 0..src_count {
                    if let Ok(bytes) = read_bytes(reader) {
                        let is_windows = origin_platform == PLATFORM_WINDOWS;
                        let src_path =
                            resolve_relative_path_for_os(&bytes, is_windows)?;
                        sources.push(src_path);
                    } else {
                        valid_sources = false;
                        break;
                    }
                }
                if !valid_sources {
                    break;
                }
                let Ok(dest_bytes) = read_bytes(reader) else {
                    break;
                };
                let is_windows = origin_platform == PLATFORM_WINDOWS;
                destination =
                    resolve_relative_path_for_os(&dest_bytes, is_windows)?;

                // Peek if TAG_SESSION_ENV immediately follows
                let has_env = if let Ok(buf) = reader.fill_buf() {
                    buf.first().copied() == Some(TAG_SESSION_ENV)
                } else {
                    false
                };
                if has_env {
                    let mut env_tag = [0_u8; 1];
                    let _ = reader.read_exact(&mut env_tag);
                    if let Ok(env_bytes) = read_bytes(reader) {
                        if let Ok(json_str) = std::str::from_utf8(&env_bytes) {
                            if let Ok(env_desc) = EnvDescription::from_json(json_str) {
                                let definition = Arc::new(env_desc);
                                let _ = shared_context.insert(
                                    SESSION_ENVIRONMENT_BINDING,
                                    Arc::clone(&definition),
                                );
                                let _ = shared_context
                                    .insert("environment", Arc::clone(&definition));
                                environment = Some(definition);
                            }
                        }
                    }
                }

                let record = JournalSessionRecord {
                    sources: sources.clone(),
                    destination: destination.clone(),
                    origin_platform: u32::from(origin_platform),
                    environment: environment.clone(),
                };
                let mut mst = DcMstring::new();
                record.encode_dc_mixed(&mut mst)?;
                writer.write_all(mst.payload_bytes())?;
                header_written = true;
            }
            TAG_SESSION_ENV => {
                let Ok(env_bytes) = read_bytes(reader) else {
                    break;
                };
                if let Ok(json_str) = std::str::from_utf8(&env_bytes) {
                    if let Ok(env_desc) = EnvDescription::from_json(json_str) {
                        let definition = Arc::new(env_desc);
                        let _ = shared_context.insert(
                            SESSION_ENVIRONMENT_BINDING,
                            Arc::clone(&definition),
                        );
                        let _ = shared_context
                            .insert("environment", Arc::clone(&definition));
                        environment = Some(definition);
                    }
                }
            }
            TAG_ENTITY => {
                let Ok(payload_len) = read_u32(reader) else {
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

                match read_entity_payload(
                    &payload[..],
                    origin_platform,
                    &shared_context,
                    policy,
                ) {
                    Ok(entity) => {
                        pending_entities.push(entity);
                    }
                    Err(error) => {
                        return Err(error).context(
                            "Checksummed journal entity cannot be decoded without losing metadata",
                        );
                    }
                }
            }
            TAG_BATCH_COMMIT => {
                let Ok(batch_id) = read_u64(reader) else {
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
                batch_hasher = Sha256Stream::new();

                let mut mst = DcMstring::new();
                mst.push_char(ctb_formats_dcdata::dc::DC_BEGIN_CHECKSUMMED_SECTION);

                let payload_start = mst.payload_bytes().len();
                mst.push_char(ctb_formats_dcdata::dc::DC_SEQUENCE_IDENTIFIER);
                batch_id.encode_dc_mixed(&mut mst)?;

                for entity in &pending_entities {
                    if let Some(read_time) = entity.metadata.read_time {
                        read_time
                            .duration_since(UNIX_EPOCH)
                            .context("Read time timestamp is before Unix epoch")?;
                    }
                    entity.encode_dc_mixed_with_context(&mut mst, &shared_context)?;
                }

                let payload_end = mst.payload_bytes().len();
                let section_bytes = mst
                    .payload_bytes()
                    .get(payload_start..payload_end)
                    .context("Invalid payload slice range for journal section")?;
                let digest = ctb_formats_checksum::sha256(section_bytes);

                mst.push_char(ctb_formats_dcdata::dc::DC_END_CHECKSUMMED_SECTION);
                mst.push_char(ctb_formats_dcdata::dc::DC_SECTION_DIGEST);
                mst.push_checksum(ctb_utilities::format_id::DC_SHA256, &digest)?;

                writer.write_all(mst.payload_bytes())?;
                pending_entities.clear();
            }
            TAG_JOB_COMPLETED => {
                anyhow::ensure!(
                    pending_entities.is_empty(),
                    "Completion marker precedes batch commit"
                );
                let mut mst = DcMstring::new();
                mst.push_char(ctb_formats_dcdata::dc::DC_OPERATION_COMPLETED);
                writer.write_all(mst.payload_bytes())?;
            }
            TAG_ERROR_RECORD => {
                let Ok(err_bytes) = read_bytes(reader) else {
                    break;
                };
                if let Ok(json_str) = std::str::from_utf8(&err_bytes) {
                    if let Ok(error) =
                        serde_json::from_str::<JournalErrorRecord>(json_str)
                    {
                        let stage_code = u64::from(error.stage.to_code());
                        let os_code = match error.os_error {
                            Some(error_code) => {
                                anyhow::ensure!(
                                    error_code != 0,
                                    "Zero OS error code cannot be distinguished from an absent error in this journal format"
                                );
                                u64::from(u32::from_le_bytes(error_code.to_le_bytes()))
                            }
                            None => 0_u64,
                        };
                        let packed = (stage_code << 32) | os_code;
                        let payload = JournalErrorRecordPayload::new(
                            error.path,
                            packed,
                            error.error_message,
                            error.timestamp_sec,
                        );
                        let mut mst = DcMstring::new();
                        payload.encode_dc_mixed(&mut mst)?;
                        writer.write_all(mst.payload_bytes())?;
                    }
                }
            }
            TAG_WARNING_RECORD => {
                let Ok(warn_bytes) = read_bytes(reader) else {
                    break;
                };
                if let Ok(json_str) = std::str::from_utf8(&warn_bytes) {
                    if let Ok(record) =
                        serde_json::from_str::<JournalWarningRecord>(json_str)
                    {
                        let payload = JournalWarningRecordPayload::new(
                            &record.code,
                            &record.message,
                            record.timestamp_sec,
                        );
                        let mut mst = DcMstring::new();
                        payload.encode_dc_mixed(&mut mst)?;
                        writer.write_all(mst.payload_bytes())?;
                    }
                }
            }
            _ => {
                break;
            }
        }
    }

    anyhow::ensure!(header_written, "Journal session header is incomplete");
    writer.flush()?;
    Ok(())
}

/// Converts in-memory legacy journal bytes to v4 journal bytes (`CTBCSCJ\x04`).
pub fn convert_legacy_journal_bytes_to_v4(bytes: &[u8]) -> Result<Vec<u8>> {
    convert_legacy_journal_bytes_to_v4_with_policy(bytes, &DcProjectionPolicy::strict())
}

/// Converts legacy journal bytes using an explicit embedded Dc projection policy.
pub fn convert_legacy_journal_bytes_to_v4_with_policy(
    bytes: &[u8],
    policy: &DcProjectionPolicy,
) -> Result<Vec<u8>> {
    let mut cursor = Cursor::new(bytes);
    let mut reader = BufReader::new(&mut cursor);
    let mut out = Vec::new();
    convert_legacy_journal_to_v4_with_policy(&mut reader, &mut out, policy)?;
    Ok(out)
}

fn attach_legacy_session_environment(
    entity: &mut FileEntity,
    env: &Arc<EnvDescription>,
) {
    if entity.metadata.environment.is_none()
        || entity
            .metadata
            .environment
            .as_ref()
            .is_some_and(|existing| existing == env)
    {
        entity.metadata.environment = Some(Arc::clone(env));
    }
    for stream in &mut entity.streams {
        attach_legacy_session_environment(&mut stream.entity, env);
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct PreservationRecord {
    #[serde(default)]
    pub(crate) identity_dc: Option<Vec<u8>>,
    pub(crate) metadata: serde_json::Value,
    pub(crate) extents: Vec<ctb_io::file::Extent>,
    pub(crate) streams: Vec<PreservedStream>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct PreservedStream {
    #[serde(default)]
    pub(crate) name: Option<StreamName>,
    pub(crate) kind: StreamKind,
    pub(crate) descriptor: Vec<u8>,
    pub(crate) data: Option<Vec<u8>>,
}

// Low-level entity serialization for legacy compatibility testing

pub(crate) fn write_entity_payload(
    w: &mut impl Write,
    entity: &FileEntity,
    context: &SharedValueContext,
) -> Result<()> {
    // Reason for fallback: streams or entities without relative path write empty path bytes to wire format
    write_bytes(w, entity.identity.path_bytes().unwrap_or(&[]))?;
    // Reason for fallback: entities without filename write empty slice to wire format
    write_bytes(w, entity.identity.filename_bytes().unwrap_or(&[]))?;
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
    let device_id = match &entity.identity.origin {
        FileOrigin::Filesystem { key, .. } => key.device_id,
        _ => 0,
    };
    match &entity.identity.hardlink_group {
        Some(grp) => {
            w.write_all(&[1])?;
            write_natural(w, grp)?;
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
    let (
        mtime_sec,
        mtime_nsec,
        atime_sec,
        atime_nsec,
        ctime_sec,
        ctime_nsec,
        birthtime_sec,
        birthtime_nsec,
    ) = match &entity.metadata.timestamps {
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
            write_bytes(w, &target.to_bytes())?;
        }
        FileEntityKind::Hardlink {
            target_relative_path,
        } => {
            w.write_all(&[3])?;
            write_bytes(w, &target_relative_path.to_bytes())?;
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
        FileEntityKind::Inaccessible => {
            w.write_all(&[10])?;
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
            write_natural(w, &key.inode)?;
            write_bytes(w, canonical_path.as_os_str().as_encoded_bytes())?;
        }
        _ => w.write_all(&[0])?,
    }
    let mut streams = Vec::with_capacity(entity.streams.len());
    for stream in &entity.streams {
        let mut descriptor = Vec::new();
        write_entity_payload(&mut descriptor, &stream.entity, context)?;
        streams.push(PreservedStream {
            name: stream.name.clone(),
            kind: stream.kind,
            descriptor,
            data: stream.data.clone(),
        });
    }
    let record = PreservationRecord {
        identity_dc: Some(entity.identity.to_dc_mixed()?.as_bytes().to_vec()),
        metadata: entity.metadata.encode_json_with_context(context)?,
        extents: match &entity.kind {
            FileEntityKind::Regular { extents, .. } => extents.clone(),
            _ => Vec::new(),
        },
        streams,
    };
    w.write_all(b"CTBMETA3")?;
    write_bytes(w, &serde_json::to_vec(&record)?)?;
    Ok(())
}

pub fn read_entity_payload(
    mut r: &[u8],
    origin_platform: u8,
    context: &SharedValueContext,
    policy: &DcProjectionPolicy,
) -> Result<FileEntity> {
    let raw_rel_path = read_bytes(&mut r)?;
    let raw_filename = read_bytes(&mut r)?;
    let has_enc = read_u8(&mut r)?;
    let is_windows = origin_platform == PLATFORM_WINDOWS;
    let enclosing_path = if has_enc == 1 {
        let enc_bytes = read_bytes(&mut r)?;
        Some(resolve_relative_path_for_os(&enc_bytes, is_windows)?)
    } else {
        None
    };
    let nlink = read_u64(&mut r)?;
    let has_grp = read_u8(&mut r)?;
    let (hardlink_group, origin_device_id) = if has_grp == 1 {
        let grp = read_natural(&mut r)?;
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
                .context("Journal read_time overflow")?;
            Some(time)
        }
        _ => None,
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
            FileEntityKind::Symlink {
                target: LinkPath::from_bytes(&target, is_windows),
            }
        }
        3 => {
            let target_relative_path = read_bytes(&mut r)?;
            FileEntityKind::Hardlink {
                target_relative_path: LinkPath::from_bytes(&target_relative_path, is_windows),
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
            let bundle_bytes = read_bytes(&mut r)?;
            let bundle_type = String::from_utf8(bundle_bytes)?;
            FileEntityKind::Bundle { bundle_type }
        }
        10 => FileEntityKind::Inaccessible,
        other => anyhow::bail!("Unknown entity kind in journal: {other}"),
    };

    let stream_count = read_u32(&mut r)?;
    let mut streams = Vec::with_capacity(usize::try_from(stream_count)?);
    for _ in 0..stream_count {
        let name_bytes = read_bytes(&mut r)?;
        let name = if name_bytes.is_empty() {
            None
        } else {
            Some(StreamName::from_bytes(&name_bytes))
        };
        let mut sha256 = [0_u8; 32];
        r.read_exact(&mut sha256)?;
        streams.push(AttachedStream {
            name,
            kind: StreamKind::ExtendedAttribute,
            entity: Box::new(FileEntity {
                identity: FileIdentity {
                    origin: FileOrigin::Synthetic,
                    relative_path: None,
                    enclosing_path: None,
                    nlink: 1,
                    hardlink_group: None,
                },
                metadata: FileMetadata::empty(),
                kind: FileEntityKind::Regular {
                    size: 0,
                    sha256,
                    is_sparse: false,
                    extents: Vec::new(),
                },
                streams: Vec::new(),
            }),
            data: None,
        });
    }

    let origin = if read_u8(&mut r)? == 1 {
        let dev = read_u64(&mut r)?;
        let ino = read_natural(&mut r)?;
        let canon = read_bytes(&mut r)?;
        let is_windows = origin_platform == PLATFORM_WINDOWS;
        FileOrigin::Filesystem {
            key: InodeKey {
                device_id: dev,
                inode: ino,
            },
            canonical_path: resolve_relative_path_for_os(&canon, is_windows)?,
        }
    } else {
        FileOrigin::Synthetic
    };

    let is_windows = origin_platform == PLATFORM_WINDOWS;
    let relative_path = if raw_rel_path.is_empty() {
        None
    } else {
        Some(resolve_relative_path_for_os(&raw_rel_path, is_windows)?)
    };

    if !raw_filename.is_empty() {
        let legacy_filename =
            resolve_relative_path_for_os(&raw_filename, is_windows)?;
        let derived_filename = relative_path
            .as_deref()
            .and_then(Path::file_name)
            .map(PathBuf::from);
        if let Some(derived) = derived_filename {
            anyhow::ensure!(
                derived == legacy_filename,
                "Journal entity filename disagrees with relative path: {:?} vs {:?}",
                legacy_filename,
                derived
            );
        }
    }

    let hardlink_group = if hardlink_group.is_some() {
        hardlink_group
    } else if nlink > 1 {
        match &origin {
            FileOrigin::Filesystem { key, .. } => {
                let dev = if origin_device_id != 0 {
                    origin_device_id
                } else {
                    key.device_id
                };
                if dev != 0 {
                    Some(key.inode.clone())
                } else {
                    None
                }
            }
            _ => None,
        }
    } else {
        None
    };

    let mut entity = FileEntity {
        identity: FileIdentity {
            origin,
            relative_path,
            enclosing_path,
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
            warnings: Vec::new(),
            errors: Vec::new(),
        },
        kind,
        streams,
    };
    let mut legacy = true;
    let mut has_semantic_identity = false;
    if !r.is_empty() {
        let mut magic = [0_u8; 8];
        r.read_exact(&mut magic)?;
        anyhow::ensure!(
            &magic == b"CTBMETA1" || &magic == b"CTBMETA2" || &magic == b"CTBMETA3",
            "Unknown preservation record version"
        );
        legacy = &magic == b"CTBMETA1";
        let mut record: PreservationRecord =
            serde_json::from_slice(&read_bytes(&mut r)?)?;
        anyhow::ensure!(
            record.streams.len() == entity.streams.len(),
            "Stream descriptor count mismatch"
        );
        has_semantic_identity = record.identity_dc.is_some();
        anyhow::ensure!(
            &magic != b"CTBMETA3" || has_semantic_identity,
            "Semantic identity is missing from CTBMETA3 preservation record"
        );
        if let Some(identity_dc) = record.identity_dc {
            let identity = DcMstring::from_bytes(identity_dc)?
                .decode::<FileIdentity>(policy)?;
            let semantic_path = identity
                .relative_path
                .as_deref()
                .filter(|value| !value.as_os_str().is_empty());
            let legacy_path = entity
                .identity
                .relative_path
                .as_deref()
                .filter(|value| !value.as_os_str().is_empty());
            anyhow::ensure!(
                semantic_path == legacy_path,
                "Semantic journal identity disagrees with its primary path"
            );
            entity.identity = identity;
        }
        if let Some(flags) = record.metadata.get_mut("platform_raw_flags") {
            if flags.is_object() {
                let raw = flags.get("raw_value").and_then(serde_json::Value::as_u64)
                    .context("Legacy platform_raw_flags must contain an unsigned raw_value")?;
                *flags = serde_json::Value::from(raw);
            }
        }
        entity.metadata = if legacy {
            serde_json::from_value(record.metadata)?
        } else {
            FileMetadata::decode_json_with_context(record.metadata, context)?
        };
        if let FileEntityKind::Regular { extents, .. } = &mut entity.kind {
            *extents = record.extents;
        }
        for (stream, preserved) in entity.streams.iter_mut().zip(record.streams) {
            stream.name = preserved.name;
            stream.kind = preserved.kind;
            stream.entity = Box::new(read_entity_payload(
                &preserved.descriptor,
                origin_platform,
                context,
                policy,
            )?);
            stream.data = preserved.data;
        }
        anyhow::ensure!(r.is_empty(), "Unexpected preservation record data");
    }
    if legacy && context.contains(SESSION_ENVIRONMENT_BINDING) {
        let environment =
            context.resolve::<EnvDescription>(SESSION_ENVIRONMENT_BINDING)?;
        attach_legacy_session_environment(&mut entity, &environment);
    }
    if !has_semantic_identity {
        let warning = "Legacy journal lacks semantic identity; absent versus empty paths and non-filesystem origin provenance may not be recoverable";
        entity.metadata.push_warning(warning);
        warn_fmt!("{}: {warning}", entity.identity.display_path());
    }
    Ok(entity)
}

// Low-level serialization helpers

pub(crate) fn write_bytes(w: &mut impl Write, bytes: &[u8]) -> Result<()> {
    write_u32(w, u32::try_from(bytes.len())?)?;
    w.write_all(bytes)?;
    Ok(())
}

pub(crate) fn read_bytes(r: &mut impl Read) -> Result<Vec<u8>> {
    let len = usize::try_from(read_u32(r)?)?;
    let mut buf = vec![0_u8; len];
    r.read_exact(&mut buf)?;
    Ok(buf)
}

pub(crate) fn write_u32(w: &mut impl Write, v: u32) -> Result<()> {
    w.write_all(&v.to_le_bytes())?;
    Ok(())
}

pub(crate) fn read_u32(r: &mut impl Read) -> Result<u32> {
    let mut buf = [0_u8; 4];
    r.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

pub(crate) fn write_u64(w: &mut impl Write, v: u64) -> Result<()> {
    w.write_all(&v.to_le_bytes())?;
    Ok(())
}

pub(crate) fn read_u64(r: &mut impl Read) -> Result<u64> {
    let mut buf = [0_u8; 8];
    r.read_exact(&mut buf)?;
    Ok(u64::from_le_bytes(buf))
}

pub(crate) fn write_natural(w: &mut impl Write, val: &Natural) -> Result<()> {
    write_bytes(w, val.to_string().as_bytes())
}

pub(crate) fn read_natural(r: &mut impl Read) -> Result<Natural> {
    let bytes = read_bytes(r)?;
    let s = std::str::from_utf8(&bytes)?;
    use std::str::FromStr;
    Natural::from_str(s)
        .map_err(|()| anyhow::anyhow!("Invalid natural in journal: {s}"))
}

pub(crate) fn write_i64(w: &mut impl Write, v: i64) -> Result<()> {
    w.write_all(&v.to_le_bytes())?;
    Ok(())
}

pub(crate) fn read_i64(r: &mut impl Read) -> Result<i64> {
    let mut buf = [0_u8; 8];
    r.read_exact(&mut buf)?;
    Ok(i64::from_le_bytes(buf))
}

pub(crate) fn read_u8(r: &mut impl Read) -> Result<u8> {
    let mut buf = [0_u8; 1];
    r.read_exact(&mut buf)?;
    Ok(buf[0])
}

pub(crate) fn write_flag(w: &mut impl Write, flag: FileFlag) -> Result<()> {
    write_bytes(w, flag.name().as_bytes())
}

pub(crate) fn read_flag(r: &mut impl Read) -> Result<Option<FileFlag>> {
    let bytes = read_bytes(r)?;
    let s = std::str::from_utf8(&bytes)?;
    Ok(FileFlag::from_name(s))
}

pub(crate) fn write_opt_timestamp(
    w: &mut impl Write,
    sec: Option<i64>,
    nsec: Option<u32>,
) -> Result<()> {
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

pub(crate) fn read_opt_timestamp(
    r: &mut impl Read,
) -> Result<(Option<i64>, Option<u32>)> {
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
