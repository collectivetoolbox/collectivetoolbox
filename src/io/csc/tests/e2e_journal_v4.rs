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

//! Comprehensive Tier 4 Real-World Workload Test Suite for `csc::journal` v4
//! Document Character Format, Multi-Record Streaming, and Forgiving Salvage.
//!
//! Tiers covered:
//! - Tier 4: Real-World Workload Scenarios (`csc::journal` multi-record streams, batch snapshots,
//!   strict v4 roundtrips, forgiving salvage of v4 dev-era fixtures)
//! - Tier 5: Adversarial Hardening (trailing unclosed batch crash rollback, tampered batch digest isolation)

#[expect(unused_imports, clippy::wildcard_imports, reason = "Integration test root")]
use ctb_utilities::*;

use anyhow::{Context, Result, ensure};
use ctb_formats_checksum::sha256;
use ctb_formats_dcdata::dc::{
    DC_BEGIN_CHECKSUMMED_SECTION, DC_BEGIN_OPERATION_SESSION, DC_END_CHECKSUMMED_SECTION,
    DC_SECTION_DIGEST, DC_SEQUENCE_IDENTIFIER, DC_SOURCE_PATH,
};
use ctb_formats_dcstring::{
    DcChar, DcDocumentNavigator, DcFrameHeaderInfo, DcMixedEncode, DcMstr, DcMstring,
    DcProjectionPolicy, validate_dc_mixed_strict,
};
use ctb_io::file::entity::{FileEntity, FileEntityKind};
use ctb_io::file::identity::{FileIdentity, FileOrigin};
use ctb_io::file::link_path::LinkPath;
use ctb_io::file::metadata::{FileMetadata, FileTimestamps};
use ctb_io_csc::journal::{
    JOURNAL_MAGIC_V4, JournalSnapshot, JournalWriter, read_journal_snapshot,
    read_journal_snapshot_with_policy,
};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

/// Returns the absolute path to the authentic fixtures directory.
fn fixtures_path() -> PathBuf {
    ctb_utilities::manifest_fixture_path(env!("CARGO_MANIFEST_DIR"), "fixtures/journals")
}

// ============================================================================
// TIER 4: REAL-WORLD WORKLOAD SCENARIOS
// ============================================================================

/// Verifies strict rejection vs forgiving salvage of the authentic dev-era
/// `v4_dcmixed-20261011.cscjournal` fixture.
///
/// Invariant:
/// - Under `DcProjectionPolicy::strict()`, `read_journal_snapshot` must reject the fixture
///   due to dev-era syntax discrepancies (e.g. unframed bare paths / timestamps).
/// - Under `DcProjectionPolicy::forgiving()`, `read_journal_snapshot_with_policy` must salvage
///   all well-formed batches and return a populated snapshot.
/// - The authentic fixture file on disk must never be modified.
#[crate::ctb_test]
fn test_tier4_v4_fixture_strict_rejection_vs_forgiving_salvage() -> Result<()> {
    let fixture_file = fixtures_path().join("v4_dcmixed-20261011.cscjournal");
    ensure!(fixture_file.exists(), "v4_dcmixed fixture file must exist");

    let initial_bytes = fs::read(&fixture_file).context("Read initial fixture bytes")?;
    let initial_len = initial_bytes.len();

    // 1. Strict document validation must reject the dev-era syntax discrepancies
    let payload_bytes = initial_bytes.get(8..).context("Payload bytes")?;
    let payload_mstr = DcMstr::from_bytes(payload_bytes).context("Parse as DcMstr")?;
    let strict_validation = validate_dc_mixed_strict(payload_mstr);
    ensure!(
        strict_validation.is_err(),
        "v4_dcmixed-20261011.cscjournal payload must fail strict document validation due to unframed dev-era syntax"
    );

    // 2. Strict journal reading rejects the fixture or rolls back the unverified dev-era batch
    let strict_snapshot = read_journal_snapshot(&fixture_file);
    ensure!(
        strict_snapshot.is_err() || strict_snapshot.is_ok_and(|s| s.committed_entities.is_empty()),
        "Authentic v4 fixture under strict mode must reject or roll back unverified entities"
    );

    // 3. Forgiving salvage converts the authentic v4 journal into a populated DcMstring
    let (dcmstring_doc, conversion_log) =
        ctb_io_csc::journal::convert_cscjournal_to_dcmstring(&initial_bytes)
            .context("v4_dcmixed-20261011.cscjournal must be converted to DcMstring")?;
    ensure!(
        dcmstring_doc.len() > ctb_formats_dcstring::DCMM_HEADER_LEN,
        "Converted DcMstring document must contain salvaged payload"
    );
    ensure!(
        !conversion_log.has_errors(),
        "Forgiving conversion of v4_dcmixed journal must not report unrecoverable errors"
    );

    // 4. Asset Integrity Invariant: fixture bytes must remain unmodified
    let current_bytes = fs::read(&fixture_file).context("Read current fixture bytes")?;
    ensure!(
        current_bytes.len() == initial_len && current_bytes == initial_bytes,
        "Authentic v4_dcmixed-20261011.cscjournal fixture must remain strictly unmodified"
    );

    Ok(())
}

/// Verifies strict validation vs forgiving inspection of the authentic dev-era
/// `v4_dcmixed_only-20261010.dcmm` fixture.
///
/// Invariant:
/// - Under `DcProjectionPolicy::strict()`, the fixture must fail strict document validation.
/// - Under `DcProjectionPolicy::forgiving()`, document navigation must discover core session
///   constructs (`DC_BEGIN_OPERATION_SESSION` and `DC_SOURCE_PATH`).
/// - The authentic fixture file on disk must never be modified.
#[crate::ctb_test]
fn test_tier4_dcmm_fixture_strict_vs_forgiving() -> Result<()> {
    let fixture_file = fixtures_path().join("v4_dcmixed_only-20261010.dcmm");
    ensure!(fixture_file.exists(), "v4_dcmixed_only.dcmm fixture file must exist");

    let initial_bytes = fs::read(&fixture_file).context("Read initial dcmm bytes")?;
    let initial_len = initial_bytes.len();

    let mstr = DcMstr::from_bytes(&initial_bytes).context("Parse dcmm as DcMstr")?;

    // 1. Strict document validation must reject legacy unframed constructs
    let strict_res = validate_dc_mixed_strict(mstr);
    ensure!(
        strict_res.is_err(),
        "v4_dcmixed_only.dcmm must fail strict document validation"
    );

    // 2. Forgiving navigation must successfully navigate tokens
    let forgiving_policy = DcProjectionPolicy::forgiving();
    let mut nav = DcDocumentNavigator::from_mstr(mstr);
    let mut found_session_opener = false;

    while let Ok(Some(stub)) = nav.next_sibling_stub(&forgiving_policy) {
        if stub.head_dc == DC_BEGIN_OPERATION_SESSION {
            found_session_opener = true;
            break;
        }
    }

    ensure!(
        found_session_opener,
        "Forgiving navigation must discover DC_BEGIN_OPERATION_SESSION in dcmm fixture"
    );

    // 3. Asset Integrity Invariant: fixture bytes must remain unmodified
    let current_bytes = fs::read(&fixture_file).context("Read current dcmm bytes")?;
    ensure!(
        current_bytes.len() == initial_len && current_bytes == initial_bytes,
        "Authentic v4_dcmixed_only.dcmm fixture must remain strictly unmodified"
    );

    Ok(())
}

/// Verifies strict v4 journal serialization roundtrip:
/// Newly generated v4 journals must pass strict document validation and decode
/// with zero loss under `DcProjectionPolicy::strict()`.
#[crate::ctb_test]
fn test_tier4_strict_v4_journal_lossless_roundtrip() -> Result<()> {
    let temp = tempdir()?;
    let journal_path = temp.path().join("strict_roundtrip.cscjournal");
    let desc_path = temp.path().join("strict_roundtrip.cscdesc");
    let src = temp.path().join("src");
    let dest = temp.path().join("dest");
    fs::create_dir_all(&src)?;
    fs::create_dir_all(&dest)?;

    let mut writer = JournalWriter::create_at_path(
        &journal_path,
        &desc_path,
        &[src.clone()],
        &dest,
    )?;

    // Construct a canonical FileEntity with full metadata
    let rel_path = PathBuf::from("docs/report.txt");
    let mut regular_entity = FileEntity {
        identity: FileIdentity {
            origin: FileOrigin::Synthetic,
            relative_path: Some(rel_path.clone()),
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
                mtime_sec: 1_700_000_000,
                mtime_nsec: 200,
                ctime_sec: 1_700_000_000,
                ctime_nsec: 300,
                birthtime_sec: Some(1_699_000_000),
                birthtime_nsec: Some(400),
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
            size: 2048,
            sha256: [0x77; 32],
            is_sparse: false,
            extents: Vec::new(),
        },
        streams: Vec::new(),
    };

    // Construct a hardlink pointing to the regular entity
    let link_entity = FileEntity {
        identity: FileIdentity {
            origin: FileOrigin::Synthetic,
            relative_path: Some(PathBuf::from("docs/hardlink.txt")),
            enclosing_path: Some(src.clone()),
            nlink: 2,
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
                mtime_sec: 1_700_000_000,
                mtime_nsec: 200,
                ctime_sec: 1_700_000_000,
                ctime_nsec: 300,
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
        kind: FileEntityKind::Hardlink {
            target_relative_path: LinkPath::from_bytes(b"docs/report.txt", false),
        },
        streams: Vec::new(),
    };

    writer.record_entity(&regular_entity);
    writer.record_entity(&link_entity);
    writer.commit_batch()?;
    writer.mark_completed()?;

    // Read back snapshot under strict policy
    let snapshot = read_journal_snapshot(&journal_path)
        .context("Strict read of newly generated v4 journal must succeed")?;

    ensure!(snapshot.is_completed, "Snapshot must be marked completed");
    ensure!(snapshot.last_batch_id == 1, "Snapshot last_batch_id must be 1");
    ensure!(
        snapshot.committed_entities.len() == 2,
        "Snapshot must contain 2 committed entities"
    );

    let committed_regular = snapshot
        .committed_entities
        .get(b"docs/report.txt".as_slice())
        .context("Committed regular entity must be present")?;
    ensure!(
        committed_regular.metadata.mode == Some(0o644),
        "Mode metadata must match"
    );
    ensure!(
        committed_regular.metadata.uid == Some(1000),
        "UID metadata must match"
    );
    ensure!(
        committed_regular.metadata.gid == Some(1000),
        "GID metadata must match"
    );

    let committed_link = snapshot
        .committed_entities
        .get(b"docs/hardlink.txt".as_slice())
        .context("Committed hardlink entity must be present")?;
    ensure!(
        matches!(
            committed_link.kind,
            FileEntityKind::Hardlink { ref target_relative_path } if target_relative_path == b"docs/report.txt"
        ),
        "Hardlink target relative path must match"
    );

    // Check that the v4 journal data (after the 8-byte magic) is strictly valid
    let file_bytes = fs::read(&journal_path)?;
    ensure!(
        file_bytes.starts_with(JOURNAL_MAGIC_V4),
        "Journal must start with JOURNAL_MAGIC_V4"
    );
    let payload_bytes = file_bytes
        .get(8..)
        .context("Payload after 8-byte magic")?;
    let payload_mstr = DcMstr::from_bytes(payload_bytes)
        .context("Payload must be valid DcMstr")?;
    validate_dc_mixed_strict(payload_mstr)
        .context("Newly serialized v4 journal payload must pass validate_dc_mixed_strict")?;

    Ok(())
}

/// Verifies multi-record streaming across sequential batches in v4 journals.
#[crate::ctb_test]
fn test_tier4_multi_record_journal_streaming_monotonic_batches() -> Result<()> {
    let temp = tempdir()?;
    let journal_path = temp.path().join("streaming.cscjournal");
    let desc_path = temp.path().join("streaming.cscdesc");
    let src = temp.path().join("src");
    let dest = temp.path().join("dest");
    fs::create_dir_all(&src)?;
    fs::create_dir_all(&dest)?;

    let mut writer = JournalWriter::create_at_path(
        &journal_path,
        &desc_path,
        &[src.clone()],
        &dest,
    )?;

    // Commit 3 sequential batches
    for batch_num in 1_u64..=3 {
        let name = format!("batch_{batch_num}.txt");
        let entity = FileEntity {
            identity: FileIdentity {
                origin: FileOrigin::Synthetic,
                relative_path: Some(PathBuf::from(&name)),
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
                size: 100,
                sha256: [0x11; 32],
                is_sparse: false,
                extents: Vec::new(),
            },
            streams: Vec::new(),
        };

        writer.record_entity(&entity);
        writer.commit_batch()?;
        if batch_num == 2 {
            writer.mark_completed()?;
            let snapshot = read_journal_snapshot(&journal_path)?;
            ensure!(snapshot.is_completed);
            ensure!(writer.mark_completed().is_err());
            drop(writer);
            writer = JournalWriter::open_for_resume(&journal_path, &desc_path, &snapshot)?;
        }
    }

    ensure!(
        !read_journal_snapshot(&journal_path)?.is_completed,
        "An explicitly reopened journal must remain unfinished until completed again"
    );
    writer.mark_completed()?;

    let snapshot = read_journal_snapshot(&journal_path)?;
    ensure!(snapshot.is_completed, "Journal must be completed");
    ensure!(
        snapshot.last_batch_id == 3,
        "Snapshot last_batch_id must be 3"
    );
    ensure!(
        snapshot.committed_entities.len() == 3,
        "Snapshot must have 3 committed entities"
    );

    for batch_num in 1_u64..=3 {
        let key = format!("batch_{batch_num}.txt");
        ensure!(
            snapshot.committed_entities.contains_key(key.as_bytes()),
            "Entity {key} must be committed in snapshot"
        );
    }

    Ok(())
}

// ============================================================================
// TIER 5: ADVERSARIAL HARDENING (JOURNAL ENGINE)
// ============================================================================

#[crate::ctb_test]
fn test_journal_rejects_closed_invalid_records_without_truncating() -> Result<()> {
    use ctb_formats_dcdata::dc::{
        DC_BEGIN_ERROR_RECORD, DC_BEGIN_WARNING_RECORD, DC_END_ERROR_RECORD,
        DC_END_WARNING_RECORD, DC_ESCAPE, DC_OPERATION_COMPLETED,
    };

    let temp = tempdir()?;
    let journal_path = temp.path().join("invalid_records.cscjournal");
    let mut writer = JournalWriter::create_at_path(
        &journal_path,
        &journal_path.with_extension("cscdesc"),
        &[temp.path().join("src")],
        &temp.path().join("dest"),
    )?;
    writer.mark_completed()?;
    let original = fs::read(&journal_path)?;
    ensure!(read_journal_snapshot(&journal_path)?.is_completed);

    for (name, tokens) in [
        ("unknown", vec![DcChar::from_short(517)]),
        ("error", vec![DC_BEGIN_ERROR_RECORD, DC_END_ERROR_RECORD]),
        ("warning", vec![DC_BEGIN_WARNING_RECORD, DC_END_WARNING_RECORD]),
        ("escaped_completion", vec![DC_ESCAPE, DC_OPERATION_COMPLETED]),
    ] {
        let test_path = temp.path().join(format!("{name}.cscjournal"));
        let mut fragment = DcMstring::new();
        for token in tokens {
            fragment.push_char(token);
        }
        fragment.push_char(DC_OPERATION_COMPLETED);
        let mut bytes = original.clone();
        bytes.extend_from_slice(fragment.payload_bytes());
        fs::write(&test_path, &bytes)?;
        ensure!(read_journal_snapshot(&test_path).is_err(), "Strict replay accepted {name}");
        if name == "unknown" || name == "escaped_completion" {
            ensure!(
                read_journal_snapshot_with_policy(&test_path, &DcProjectionPolicy::forgiving()).is_err(),
                "Forgiving replay silently discarded {name}"
            );
        }
        ensure!(fs::read(&test_path)? == bytes, "Replay modified {name}");
    }

    for (name, tail, incomplete) in [
        ("incomplete_token", &[0xC2_u8][..], true),
        ("invalid_token", &[0x80_u8][..], false),
    ] {
        let test_path = temp.path().join(format!("{name}.cscjournal"));
        let mut bytes = original.clone();
        bytes.extend_from_slice(tail);
        fs::write(&test_path, bytes)?;
        let result = read_journal_snapshot(&test_path);
        if incomplete {
            let snapshot = result?;
            ensure!(snapshot.is_completed);
            ensure!(snapshot.valid_length == u64::try_from(original.len())?);
        } else {
            ensure!(result.is_err(), "Malformed token must not be mistaken for EOF");
        }
    }
    Ok(())
}

/// Verifies that crash truncation mid-batch results in clean rollback of the trailing
/// unclosed checksummed section without losing previously committed batches.
#[crate::ctb_test]
fn test_tier4_unclosed_trailing_batch_crash_rollback() -> Result<()> {
    let temp = tempdir()?;
    let journal_path = temp.path().join("crash_rollback.cscjournal");
    let desc_path = temp.path().join("crash_rollback.cscdesc");
    let src = temp.path().join("src");
    let dest = temp.path().join("dest");
    fs::create_dir_all(&src)?;
    fs::create_dir_all(&dest)?;

    let mut writer = JournalWriter::create_at_path(
        &journal_path,
        &desc_path,
        &[src.clone()],
        &dest,
    )?;

    // Commit 1 valid batch
    let entity = FileEntity {
        identity: FileIdentity {
            origin: FileOrigin::Synthetic,
            relative_path: Some(PathBuf::from("valid_file.txt")),
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
            size: 50,
            sha256: [0x22; 32],
            is_sparse: false,
            extents: Vec::new(),
        },
        streams: Vec::new(),
    };

    writer.record_entity(&entity);
    writer.commit_batch()?;

    // Simulate a crash during batch 2 write:
    // Append an unclosed DC_BEGIN_CHECKSUMMED_SECTION (Dc 543) without closer
    {
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(&journal_path)?;

        let mut unclosed_mstr = DcMstring::new();
        unclosed_mstr.push_char(DC_BEGIN_CHECKSUMMED_SECTION);
        let body_start = unclosed_mstr.payload_bytes().len();
        unclosed_mstr.push_char(DC_SEQUENCE_IDENTIFIER);
        2u64.encode_dc_mixed(&mut unclosed_mstr)?;
        let valid_digest = sha256(unclosed_mstr.payload_bytes().get(body_start..).context("Batch body")?);
        let mut complete = unclosed_mstr.clone();
        complete.push_char(DC_END_CHECKSUMMED_SECTION);
        complete.push_char(DC_SECTION_DIGEST);
        complete.push_checksum(ctb_utilities::format_id::DC_SHA256, &valid_digest)?;
        let control_path = temp.path().join("complete_crash_batch.cscjournal");
        let mut control_bytes = fs::read(&journal_path)?;
        control_bytes.extend_from_slice(complete.payload_bytes());
        fs::write(&control_path, control_bytes)?;
        let control = read_journal_snapshot(&control_path)?;
        ensure!(control.last_batch_id == 2 && control.is_committed(b"valid_file.txt"));
        // Cut off mid-stream without DC_END_CHECKSUMMED_SECTION or digest

        file.write_all(unclosed_mstr.payload_bytes())?;
    }

    // Reading snapshot must successfully recover batch 1 and roll back the unclosed tail
    let snapshot = read_journal_snapshot(&journal_path)
        .context("Snapshot recovery after crash truncation must succeed")?;

    ensure!(
        snapshot.last_batch_id == 1,
        "Snapshot must roll back to last valid batch (batch 1)"
    );
    ensure!(
        snapshot.committed_entities.len() == 1,
        "Only entities from valid batch 1 must be present"
    );
    ensure!(
        snapshot.is_committed(b"valid_file.txt"),
        "Entity from batch 1 must be committed"
    );

    Ok(())
}

/// Verifies that a batch with a tampered digest is rejected and rolled back,
/// preventing corrupted data from entering the snapshot.
#[crate::ctb_test]
fn test_tier4_tampered_batch_digest_rejection_and_isolation() -> Result<()> {
    let temp = tempdir()?;
    let journal_path = temp.path().join("tampered_batch.cscjournal");
    let desc_path = temp.path().join("tampered_batch.cscdesc");
    let src = temp.path().join("src");
    let dest = temp.path().join("dest");
    fs::create_dir_all(&src)?;
    fs::create_dir_all(&dest)?;

    let mut writer = JournalWriter::create_at_path(
        &journal_path,
        &desc_path,
        &[src.clone()],
        &dest,
    )?;

    // Commit batch 1
    let entity = FileEntity {
        identity: FileIdentity {
            origin: FileOrigin::Synthetic,
            relative_path: Some(PathBuf::from("safe_record.txt")),
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
            size: 50,
            sha256: [0x33; 32],
            is_sparse: false,
            extents: Vec::new(),
        },
        streams: Vec::new(),
    };

    writer.record_entity(&entity);
    writer.commit_batch()?;

    // Append batch 2 with an intentionally bogus SHA-256 digest
    {
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(&journal_path)?;

        let mut tampered_mst = DcMstring::new();
        tampered_mst.push_char(DC_BEGIN_CHECKSUMMED_SECTION);
        let body_start = tampered_mst.payload_bytes().len();
        tampered_mst.push_char(DC_SEQUENCE_IDENTIFIER);
        2u64.encode_dc_mixed(&mut tampered_mst)?;

        // Fake entity in batch
        let fake_entity = FileEntity {
            identity: FileIdentity {
                origin: FileOrigin::Synthetic,
                relative_path: Some(PathBuf::from("tampered_file.txt")),
                enclosing_path: Some(src),
                nlink: 1,
                hardlink_group: None,
            },
            metadata: FileMetadata {
                native: None,
                mode: Some(0o644),
                uid: Some(1000),
                gid: Some(1000),
                timestamps: None,
                flags: None,
                platform_raw_flags: None,
                read_time: None,
                filesystem_type: None,
                environment: None,
                apple: None,
                warnings: Vec::new(),
                errors: Vec::new(),
            },
            kind: FileEntityKind::Regular {
                size: 10,
                sha256: [0; 32],
                is_sparse: false,
                extents: Vec::new(),
            },
            streams: Vec::new(),
        };
        fake_entity.encode_dc_mixed(&mut tampered_mst)?;
        let valid_digest = sha256(tampered_mst.payload_bytes().get(body_start..).context("Batch body")?);
        tampered_mst.push_char(DC_END_CHECKSUMMED_SECTION);

        // Append SectionDigest (545) with invalid digest
        tampered_mst.push_char(DC_SECTION_DIGEST);
        let mut valid_mst = tampered_mst.clone();
        valid_mst.push_checksum(ctb_utilities::format_id::DC_SHA256, &valid_digest)?;
        let control_path = temp.path().join("untampered_batch.cscjournal");
        let mut control_bytes = fs::read(&journal_path)?;
        control_bytes.extend_from_slice(valid_mst.payload_bytes());
        fs::write(&control_path, control_bytes)?;
        let control = read_journal_snapshot(&control_path)?;
        ensure!(control.last_batch_id == 2 && control.committed_entities.len() == 2);
        ensure!(control.is_committed(b"tampered_file.txt"));
        let bogus_digest = [0xFF_u8; 32];
        tampered_mst.push_checksum(ctb_utilities::format_id::DC_SHA256, &bogus_digest)?;

        file.write_all(tampered_mst.payload_bytes())?;
    }

    // Read snapshot: the journal streaming engine must detect the invalid digest,
    // reject batch 2, and roll back to batch 1.
    let snapshot = read_journal_snapshot(&journal_path)?;

    ensure!(
        snapshot.last_batch_id == 1,
        "Snapshot must roll back to batch 1 on tampered digest"
    );
    ensure!(
        snapshot.committed_entities.len() == 1,
        "Tampered batch entities must not be committed"
    );
    ensure!(
        snapshot.is_committed(b"safe_record.txt"),
        "Batch 1 entity must remain committed"
    );
    ensure!(
        !snapshot.is_committed(b"tampered_file.txt"),
        "Tampered entity must NOT be committed"
    );

    let salvaged = read_journal_snapshot_with_policy(&journal_path, &DcProjectionPolicy::forgiving())?;
    ensure!(salvaged.last_batch_id == 2 && salvaged.is_committed(b"tampered_file.txt"));
    ensure!(salvaged.warnings.iter().any(|warning| warning.code == "journal-digest-mismatch"));

    let mut suffix = DcMstring::new();
    suffix.push_char(ctb_formats_dcdata::dc::DC_OPERATION_COMPLETED);
    fs::OpenOptions::new().append(true).open(&journal_path)?.write_all(suffix.payload_bytes())?;
    ensure!(read_journal_snapshot(&journal_path).is_err(), "Interior digest mismatch must not authorize suffix truncation");

    Ok(())
}

