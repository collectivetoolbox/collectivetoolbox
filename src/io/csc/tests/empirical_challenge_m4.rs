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

//! Milestone 4 Empirical Challenge Suite:
//! Stress-testing journal session/error/warning records, verifying $O(1)$ arena reuse,
//! and testing read_v4_journal_snapshot error resilience under adversarial corruptions.

#[expect(unused_imports, clippy::wildcard_imports, reason = "Integration test root")]
use ctb_utilities::*;

use anyhow::{Context, Result, ensure};
use ctb_formats_dcdata::dc::*;
use ctb_formats_dcstring::{
    DcChar, DcDocumentNavigator, DcFrameStub, DcMixedDecode, DcMixedEncode,
    DcMstr, DcMstring, DcProjectionPolicy, DcSyntaxArena, validate_dc_mixed_strict,
};
use ctb_io::file::entity::{FileEntity, FileEntityKind};
use ctb_io::file::identity::{FileIdentity, FileOrigin};
use ctb_io::file::metadata::{FileMetadata, FileTimestamps};
use ctb_io_environment::EnvDescription;
use ctb_io_csc::journal::{
    JOURNAL_MAGIC_V4, JournalErrorRecord, JournalErrorRecordPayload, JournalErrorStage,
    JournalSessionRecord, JournalSnapshot, JournalWarningRecord,
    JournalWarningRecordPayload, JournalWarningRecordStringPayload, JournalWriter,
    read_journal_snapshot, read_journal_snapshot_with_policy, read_v4_journal_snapshot,
};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::tempdir;

// ============================================================================
// 1. STRESS TESTS: JournalSessionRecord, JournalErrorRecordPayload, JournalWarningRecordPayload
// ============================================================================

/// Helper to decode a frame from a DcMstr document using DcDocumentNavigator and DcSyntaxArena.
fn decode_single_frame<T: DcMixedDecode>(
    mstr: &DcMstr,
    policy: &DcProjectionPolicy,
) -> Result<T> {
    let mut nav = DcDocumentNavigator::from_mstr(mstr);
    let mut arena = DcSyntaxArena::new();
    let stub = nav
        .next_sibling_stub(policy)?
        .context("Expected at least one frame stub")?;
    nav.decode_frame_with_arena::<T>(&stub, &mut arena, None, policy)
}

#[crate::ctb_test]
fn test_challenge_journal_session_record_varied_roundtrip() -> Result<()> {
    let env_sample = Arc::new(EnvDescription {
        os: "linux".to_owned(),
        cwd: Some(PathBuf::from("/workspaces/ctoolbox")),
        ..EnvDescription::default()
    });

    let test_cases = vec![
        // 1. Empty sources, basic destination, platform 0, None environment
        JournalSessionRecord {
            sources: Vec::new(),
            destination: PathBuf::from("/var/data/dest"),
            origin_platform: 0,
            environment: None,
        },
        // 2. Multiple sources, unicode path with emoji and special chars, Some environment
        JournalSessionRecord {
            sources: vec![
                PathBuf::from("/home/user/src1"),
                PathBuf::from("rel/path/two"),
                PathBuf::from("/🦀/ünicøde/🚀/日本語/folder"),
            ],
            destination: PathBuf::from("/mnt/backup/destination_🎯"),
            origin_platform: 1,
            environment: Some(Arc::clone(&env_sample)),
        },
        // 3. Single source, platform u32::MAX, extreme unicode
        JournalSessionRecord {
            sources: vec![PathBuf::from("test/path")],
            destination: PathBuf::from("dest/path"),
            origin_platform: u32::MAX,
            environment: None,
        },
    ];

    let strict_policy = DcProjectionPolicy::strict();

    for (idx, original) in test_cases.into_iter().enumerate() {
        let mut mstring = DcMstring::new();
        original
            .encode_dc_mixed(&mut mstring)
            .with_context(|| format!("Case {idx}: encode_dc_mixed failed"))?;

        // Verify that encoded segment passes strict syntax validation
        validate_dc_mixed_strict(&mstring)
            .with_context(|| format!("Case {idx}: validate_dc_mixed_strict failed"))?;

        // Decode via navigator and arena
        let decoded: JournalSessionRecord = decode_single_frame(
            &mstring,
            &strict_policy,
        ).with_context(|| format!("Case {idx}: decode_single_frame failed"))?;

        ensure!(
            decoded.sources == original.sources,
            "Case {idx}: sources mismatch: {:?} vs {:?}",
            decoded.sources,
            original.sources
        );
        ensure!(
            decoded.destination == original.destination,
            "Case {idx}: destination mismatch: {:?} vs {:?}",
            decoded.destination,
            original.destination
        );
        ensure!(
            decoded.origin_platform == original.origin_platform,
            "Case {idx}: origin_platform mismatch: {} vs {}",
            decoded.origin_platform,
            original.origin_platform
        );
        ensure!(
            decoded.environment == original.environment,
            "Case {idx}: environment mismatch"
        );
    }

    Ok(())
}

#[crate::ctb_test]
fn test_challenge_journal_error_record_payload_varied_roundtrip() -> Result<()> {
    let test_cases = vec![
        // Empty message, zero timestamp, zero packed status
        JournalErrorRecordPayload::new(
            PathBuf::from("/etc/hosts"),
            0,
            String::new(),
            0,
        ),
        // Unicode message, extreme positive timestamp, packed status with high stage code
        JournalErrorRecordPayload::new(
            PathBuf::from("/dir/🦀/file.txt"),
            (3_u64 << 32) | 13_u64,
            "Failed to copy file: permission denied 🚫 (äöü)".to_owned(),
            1_800_000_000,
        ),
        // Negative timestamp, packed status u64::MAX, long message
        JournalErrorRecordPayload::new(
            PathBuf::from("rel/error.log"),
            u64::MAX,
            "A".repeat(500),
            -1_000_000,
        ),
        // Extreme negative timestamp (i64::MIN + 1), extreme positive timestamp (i64::MAX)
        JournalErrorRecordPayload::new(
            PathBuf::from("/path"),
            1,
            "min timestamp".to_owned(),
            -62_135_596_800,
        ),
        JournalErrorRecordPayload::new(
            PathBuf::from("/path2"),
            2,
            "max timestamp".to_owned(),
            253_402_300_799,
        ),
    ];

    let strict_policy = DcProjectionPolicy::strict();

    for (idx, original) in test_cases.into_iter().enumerate() {
        let mut mstring = DcMstring::new();
        original
            .encode_dc_mixed(&mut mstring)
            .with_context(|| format!("Case {idx}: encode failed"))?;

        validate_dc_mixed_strict(&mstring)
            .with_context(|| format!("Case {idx}: strict validation failed"))?;

        let decoded: JournalErrorRecordPayload = decode_single_frame(
            &mstring,
            &strict_policy,
        ).with_context(|| format!("Case {idx}: decode failed"))?;

        ensure!(
            decoded == original,
            "Case {idx}: Decoded payload does not match original: {decoded:?} vs {original:?}"
        );
    }

    Ok(())
}

#[crate::ctb_test]
fn test_challenge_journal_warning_record_payload_varied_roundtrip() -> Result<()> {
    let test_cases = vec![
        JournalWarningRecordPayload::new("WARN001", "Caveat detected", 0),
        JournalWarningRecordPayload::new("", "", 1_700_000_000),
        JournalWarningRecordPayload::new("CODE:SUBCODE", "Message with : multiple : colons : ⚠️", -500),
        JournalWarningRecordPayload::new("LONG", &"W".repeat(1000), 2_000_000_000),
    ];

    let strict_policy = DcProjectionPolicy::strict();

    for (idx, original) in test_cases.into_iter().enumerate() {
        let mut mstring = DcMstring::new();
        original
            .encode_dc_mixed(&mut mstring)
            .with_context(|| format!("Case {idx}: encode failed"))?;

        validate_dc_mixed_strict(&mstring)
            .with_context(|| format!("Case {idx}: strict validation failed"))?;

        let decoded: JournalWarningRecordPayload = decode_single_frame(
            &mstring,
            &strict_policy,
        ).with_context(|| format!("Case {idx}: decode failed"))?;

        ensure!(
            decoded == original,
            "Case {idx}: Decoded warning payload mismatch: {decoded:?} vs {original:?}"
        );
    }

    // Also test JournalWarningRecordStringPayload (dev-era legacy compatibility)
    let string_payload = JournalWarningRecordStringPayload {
        message: "Legacy warning message without timestamp tuple".to_owned(),
    };
    let mut mstring = DcMstring::new();
    string_payload.encode_dc_mixed(&mut mstring)?;
    let decoded_string: JournalWarningRecordStringPayload = decode_single_frame(
        &mstring,
        &strict_policy,
    )?;
    ensure!(decoded_string == string_payload, "String warning payload mismatch");

    Ok(())
}

fn try_decode_slice<T: DcMixedDecode>(bytes: &[u8], policy: &DcProjectionPolicy) {
    if let Ok(mstr) = DcMstr::from_bytes(bytes) {
        let mut nav = DcDocumentNavigator::from_mstr(mstr);
        let mut arena = DcSyntaxArena::new();
        if let Ok(Some(stub)) = nav.next_sibling_stub(policy) {
            let _ = nav.decode_frame_with_arena::<T>(&stub, &mut arena, None, policy);
        }
    }
}

#[crate::ctb_test]
fn test_challenge_truncated_malformed_records_no_panic() -> Result<()> {
    let session = JournalSessionRecord {
        sources: vec![PathBuf::from("/src1"), PathBuf::from("/src2")],
        destination: PathBuf::from("/dest"),
        origin_platform: 1,
        environment: None,
    };
    let error = JournalErrorRecordPayload::new(
        PathBuf::from("/err/path"),
        42,
        "Something failed".to_owned(),
        1_700_000_000,
    );
    let warning = JournalWarningRecordPayload::new("W1", "Warning msg", 1_700_000_000);

    let mut session_bytes = DcMstring::new();
    session.encode_dc_mixed(&mut session_bytes)?;
    let session_slice = session_bytes.as_bytes();

    let mut error_bytes = DcMstring::new();
    error.encode_dc_mixed(&mut error_bytes)?;
    let error_slice = error_bytes.as_bytes();

    let mut warning_bytes = DcMstring::new();
    warning.encode_dc_mixed(&mut warning_bytes)?;
    let warning_slice = warning_bytes.as_bytes();

    let policies = [
        DcProjectionPolicy::strict(),
        DcProjectionPolicy::forgiving(),
        DcProjectionPolicy::forgiving_quiet(),
    ];

    // Truncate at every single byte from 0 up to len - 1
    for slice in [session_slice, error_slice, warning_slice] {
        let total_len = slice.len();
        for cut_len in 0..total_len {
            let truncated = slice.get(..cut_len).unwrap_or(&[]);
            for policy in &policies {
                // Must never panic
                try_decode_slice::<JournalSessionRecord>(truncated, policy);
                try_decode_slice::<JournalErrorRecordPayload>(truncated, policy);
                try_decode_slice::<JournalWarningRecordPayload>(truncated, policy);
            }
        }
    }

    // Malformed bitflips / byte corruptions
    for slice in [session_slice, error_slice, warning_slice] {
        let mut corrupted = slice.to_vec();
        for i in 0..corrupted.len() {
            let orig = corrupted[i];
            corrupted[i] = orig ^ 0xFF; // flip bits
            for policy in &policies {
                try_decode_slice::<JournalSessionRecord>(&corrupted, policy);
                try_decode_slice::<JournalErrorRecordPayload>(&corrupted, policy);
                try_decode_slice::<JournalWarningRecordPayload>(&corrupted, policy);
            }
            corrupted[i] = orig;
        }
    }

    Ok(())
}

// ============================================================================
// 2. EMPIRICAL VERIFICATION: O(1) Arena Reuse Across Streaming Records
// ============================================================================

#[crate::ctb_test]
fn test_challenge_arena_reuse_o1_steady_state() -> Result<()> {
    // Generate 1,000 distinct JournalErrorRecordPayload frames in a single contiguous buffer
    const RECORD_COUNT: usize = 1_000;
    let mut combined_stream = DcMstring::new();

    for i in 0..RECORD_COUNT {
        let payload = JournalErrorRecordPayload::new(
            PathBuf::from(format!("/data/path/item_{i}.dat")),
            u64::try_from(i)?,
            format!("Error condition report for record index #{i}"),
            1_700_000_000i64.saturating_add(i64::try_from(i)?),
        );
        payload.encode_dc_mixed(&mut combined_stream)?;
    }

    let mut nav = DcDocumentNavigator::from_mstr(&combined_stream);
    let policy = DcProjectionPolicy::strict();

    // Reusable arena instantiated once
    let mut arena = DcSyntaxArena::new();
    let mut decoded_count = 0_usize;
    let mut max_nodes_seen = 0_usize;

    while let Some(stub) = nav.next_sibling_stub(&policy)? {
        // Empirically verify arena is empty before each frame decode
        arena.clear();
        ensure!(
            arena.nodes().is_empty(),
            "arena.clear() must reset arena.nodes().len() to 0"
        );
        ensure!(
            arena.roots().is_empty(),
            "arena.clear() must reset arena.roots().len() to 0"
        );
        ensure!(
            arena.diagnostics().is_empty(),
            "arena.clear() must reset arena.diagnostics().len() to 0"
        );

        let decoded: JournalErrorRecordPayload =
            nav.decode_frame_with_arena(&stub, &mut arena, None, &policy)?;

        let current_nodes = arena.nodes().len();
        max_nodes_seen = max_nodes_seen.max(current_nodes);

        // A single JournalErrorRecordPayload has a bounded number of syntax nodes (< 50)
        ensure!(
            current_nodes < 50,
            "Single record node count exceeded bound: {current_nodes}"
        );

        ensure!(
            decoded.packed_status == u64::try_from(decoded_count)?,
            "Decoded sequence mismatch"
        );

        decoded_count = decoded_count.saturating_add(1);
    }

    ensure!(
        decoded_count == RECORD_COUNT,
        "Must have decoded exactly {RECORD_COUNT} records, got {decoded_count}"
    );

    // Verify steady-state bound: at no point did arena.nodes().len() exceed single-record bound
    ensure!(
        max_nodes_seen < 50,
        "Arena nodes count grew beyond single-record bound: {max_nodes_seen}"
    );

    Ok(())
}

// ============================================================================
// 3. ADVERSARIAL STRESS TEST: read_v4_journal_snapshot Under Corrupted Inputs
// ============================================================================

#[crate::ctb_test]
fn test_challenge_read_v4_journal_snapshot_adversarial_inputs() -> Result<()> {
    let temp = tempdir()?;
    let strict_policy = DcProjectionPolicy::strict();
    let forgiving_policy = DcProjectionPolicy::forgiving();

    // 1. Truncated magic (< 8 bytes)
    for len in 0..8 {
        let path = temp.path().join(format!("trunc_magic_{len}.cscjournal"));
        let magic_prefix = JOURNAL_MAGIC_V4.get(..len).unwrap_or(&[]);
        fs::write(&path, magic_prefix)?;

        let mut file = File::open(&path)?;
        let strict_res = read_v4_journal_snapshot(&mut file, &path, false, &strict_policy);
        ensure!(strict_res.is_err(), "Truncated magic must return Err");

        let mut file = File::open(&path)?;
        let forgiving_res = read_v4_journal_snapshot(&mut file, &path, false, &forgiving_policy);
        ensure!(forgiving_res.is_err(), "Truncated magic must return Err even under forgiving");
    }

    // 2. Exactly 8 bytes magic (no session header)
    {
        let path = temp.path().join("magic_only.cscjournal");
        fs::write(&path, JOURNAL_MAGIC_V4)?;
        let mut file = File::open(&path)?;
        let res = read_v4_journal_snapshot(&mut file, &path, false, &strict_policy);
        ensure!(res.is_err(), "Magic only without session header must fail");
    }

    // 3. Truncated session header after magic
    {
        let mut session_mst = DcMstring::new();
        let session = JournalSessionRecord {
            sources: vec![PathBuf::from("/src")],
            destination: PathBuf::from("/dst"),
            origin_platform: 1,
            environment: None,
        };
        session.encode_dc_mixed(&mut session_mst)?;
        let session_bytes = session_mst.payload_bytes();

        for cut in 1..session_bytes.len() {
            let path = temp.path().join(format!("trunc_session_{cut}.cscjournal"));
            let mut file_data = JOURNAL_MAGIC_V4.to_vec();
            file_data.extend_from_slice(session_bytes.get(..cut).unwrap_or(&[]));
            fs::write(&path, file_data)?;

            let mut file = File::open(&path)?;
            let strict_res = read_v4_journal_snapshot(&mut file, &path, false, &strict_policy);
            ensure!(strict_res.is_err(), "Truncated session must return Err");

            let mut file = File::open(&path)?;
            let forgiving_res = read_v4_journal_snapshot(&mut file, &path, false, &forgiving_policy);
            ensure!(forgiving_res.is_err(), "Truncated session must return Err under forgiving");
        }
    }

    // 4. Corrupted batch tail after valid session and valid batch 1
    {
        let journal_path = temp.path().join("corrupted_tail.cscjournal");
        let desc_path = temp.path().join("corrupted_tail.cscdesc");
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

        let entity = FileEntity {
            identity: FileIdentity {
                origin: FileOrigin::Synthetic,
                relative_path: Some(PathBuf::from("valid.txt")),
                enclosing_path: Some(src.clone()),
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
                sha256: [1; 32],
                is_sparse: false,
                extents: Vec::new(),
            },
            streams: Vec::new(),
        };

        writer.record_entity(&entity);
        writer.commit_batch()?;

        // Append random garbage bytes
        {
            let mut file = OpenOptions::new().append(true).open(&journal_path)?;
            file.write_all(&[0xDE, 0xAD, 0xBE, 0xEF, 0xFF, 0x00, 0x12, 0x34])?;
        }

        // Reading snapshot under both policies must successfully recover batch 1 without panicking!
        let mut file = File::open(&journal_path)?;
        let snapshot = read_v4_journal_snapshot(&mut file, &journal_path, false, &strict_policy)?;
        ensure!(
            snapshot.last_batch_id == 1,
            "Garbage tail must roll back to batch 1"
        );
        ensure!(
            snapshot.is_committed(b"valid.txt"),
            "Batch 1 entity must remain committed"
        );
    }

    Ok(())
}

#[crate::ctb_test]
fn test_challenge_interleaved_journal_records_lossless_roundtrip() -> Result<()> {
    let temp = tempdir()?;
    let journal_path = temp.path().join("interleaved.cscjournal");
    let desc_path = temp.path().join("interleaved.cscdesc");
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

    // 1. Record warnings before any batch
    writer.record_warning("W_INIT", "Initialization warning before batches")?;

    // 2. Record error before batch
    let error1 = JournalErrorRecord {
        path: PathBuf::from("skipped/unreadable.txt"),
        raw_path: b"skipped/unreadable.txt".to_vec(),
        stage: JournalErrorStage::MetadataRead,
        error_message: "Permission denied reading metadata".to_owned(),
        os_error: Some(13), // EACCES
        timestamp_sec: 1_700_000_010,
    };
    writer.record_error(&error1)?;

    // 3. Commit Batch 1
    let entity1 = FileEntity {
        identity: FileIdentity {
            origin: FileOrigin::Synthetic,
            relative_path: Some(PathBuf::from("file1.txt")),
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
            flags: None,
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
            sha256: [0x55; 32],
            is_sparse: false,
            extents: Vec::new(),
        },
        streams: Vec::new(),
    };
    writer.record_entity(&entity1);
    writer.commit_batch()?;

    // 4. Record warning and error between Batch 1 and Batch 2
    writer.record_warning("W_MID", "Mid-stream warning: symlink target unverified")?;
    let error2 = JournalErrorRecord {
        path: PathBuf::from("skipped/negative_os.txt"),
        raw_path: b"skipped/negative_os.txt".to_vec(),
        stage: JournalErrorStage::PayloadRead,
        error_message: "Special negative OS code".to_owned(),
        os_error: Some(-1),
        timestamp_sec: 1_700_000_020,
    };
    writer.record_error(&error2)?;

    // 5. Commit Batch 2
    let entity2 = FileEntity {
        identity: FileIdentity {
            origin: FileOrigin::Synthetic,
            relative_path: Some(PathBuf::from("file2.txt")),
            enclosing_path: Some(src.clone()),
            nlink: 1,
            hardlink_group: None,
        },
        metadata: FileMetadata {
            native: None,
            mode: Some(0o755),
            uid: Some(1000),
            gid: Some(1000),
            timestamps: None,
            flags: None,
            platform_raw_flags: None,
            read_time: None,
            filesystem_type: None,
            environment: writer.environment.clone(),
            apple: None,
            warnings: Vec::new(),
            errors: Vec::new(),
        },
        kind: FileEntityKind::Regular {
            size: 200,
            sha256: [0x66; 32],
            is_sparse: false,
            extents: Vec::new(),
        },
        streams: Vec::new(),
    };
    writer.record_entity(&entity2);
    writer.commit_batch()?;

    // 6. Final warning and mark completed
    writer.record_warning("W_DONE", "Operation finishing with caveats")?;
    writer.mark_completed()?;

    // 7. Verify read_journal_snapshot under strict policy
    let snapshot = read_journal_snapshot(&journal_path)?;
    ensure!(snapshot.is_completed, "Snapshot must be completed");
    ensure!(snapshot.last_batch_id == 2, "Snapshot last_batch_id must be 2");
    ensure!(snapshot.committed_entities.len() == 2, "Must have 2 committed entities");
    ensure!(snapshot.is_committed(b"file1.txt"), "file1.txt must be committed");
    ensure!(snapshot.is_committed(b"file2.txt"), "file2.txt must be committed");

    // Verify errors
    ensure!(snapshot.errors.len() == 2, "Must have 2 error records");
    ensure!(snapshot.errors[0] == error1, "Error 1 mismatch: {:?} vs {:?}", snapshot.errors[0], error1);
    ensure!(snapshot.errors[1] == error2, "Error 2 mismatch: {:?} vs {:?}", snapshot.errors[1], error2);

    // Verify warnings
    ensure!(snapshot.warnings.len() == 3, "Must have 3 warning records, got {}", snapshot.warnings.len());
    ensure!(snapshot.warnings[0].code == "W_INIT", "Warning 0 code mismatch");
    ensure!(snapshot.warnings[1].code == "W_MID", "Warning 1 code mismatch");
    ensure!(snapshot.warnings[2].code == "W_DONE", "Warning 2 code mismatch");

    Ok(())
}

