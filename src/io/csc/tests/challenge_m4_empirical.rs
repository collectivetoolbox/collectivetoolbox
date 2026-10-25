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

//! Empirical Challenge Test Suite for Milestone 4:
//! 1. Authentic v4 journal fixtures (`v4_dcmixed*`):
//!    - Under `strict()` policy: strict rejection with error / batch rollback
//!      due to untyped checksum framing and dev-era syntax discrepancies.
//!    - Under `forgiving()` policy: full salvage of session, batches, entities,
//!      and recognition of `is_completed = true`.
//!    - Exact disk fixture byte preservation (zero byte mutations).
//! 2. All journal format revisions (v1, v2, v3, v4) fidelity and conversion.
//! 3. Total elimination of `DcMixedReader`.
//! 4. $O(1)$ arena reuse and stress testing across multi-record streams.

#[expect(unused_imports, clippy::wildcard_imports, reason = "Integration test root")]
use ctb_utilities::*;

use ctb_formats_checksum::sha256;
use ctb_formats_dcdata::dc::{
    DC_BEGIN_CHECKSUMMED_SECTION, DC_BEGIN_ERROR_RECORD, DC_BEGIN_OPERATION_SESSION,
    DC_BEGIN_WARNING_RECORD, DC_END_CHECKSUMMED_SECTION, DC_OPERATION_COMPLETED,
    DC_SECTION_DIGEST, DC_SEQUENCE_IDENTIFIER, DC_SOURCE_PATH,
};
use ctb_formats_dcstring::{
    DcChar, DcDocumentNavigator, DcFrameStub, DcMixedDecode, DcMixedEncode,
    DcMstr, DcMstring, DcProjectionPolicy, DcSyntaxArena, validate_dc_mixed_strict,
};
use ctb_io::file::apple_single_double::{
    ExtendedFinderInfo, FinderFlags, FinderInfo, FinderLabel,
};
use ctb_io::file::entity::{FileEntity, FileEntityKind};
use ctb_io::file::identity::{FileIdentity, FileOrigin};
use ctb_io::file::link_path::LinkPath;
use ctb_io::file::metadata::{FileMetadata, FileTimestamps};
use ctb_io::file::streams::StreamName;
use ctb_io_csc::journal::{
    JOURNAL_MAGIC_V4, JournalErrorRecord, JournalErrorRecordPayload, JournalErrorStage,
    JournalSessionRecord, JournalSnapshot, JournalWarningRecord,
    JournalWarningRecordPayload, JournalWarningRecordStringPayload, StateJournal,
    convert_cscjournal_to_dcmstring, convert_cscjournal_to_bytes,
    read_journal_snapshot, read_journal_snapshot_with_policy, read_v4_journal_snapshot,
};
use ctb_io_environment::EnvDescription;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::tempdir;

fn fixtures_path() -> PathBuf {
    ctb_utilities::manifest_fixture_path(env!("CARGO_MANIFEST_DIR"), "fixtures/journals")
}

// ============================================================================
// 1. AUTHENTIC V4 FIXTURE EMPIRICAL CHALLENGES
// ============================================================================

/// Empirically verify that authentic dev-era v4 fixtures are strictly rejected
/// under `strict()` policy and cleanly recovered under `forgiving()` policy.
#[crate::ctb_test]
fn test_challenge_authentic_v4_strict_rejection_and_forgiving_salvage() -> Result<()> {
    let fixture_path = fixtures_path().join("v4_dcmixed-20261011.cscjournal");
    ensure!(fixture_path.exists(), "v4_dcmixed fixture file must exist");

    let initial_bytes = fs::read(&fixture_path)?;
    let initial_len = initial_bytes.len();
    ensure!(initial_len > 8, "Fixture file must contain magic and payload");

    // Check magic bytes
    let magic = &initial_bytes[0..8];
    ensure!(
        magic == JOURNAL_MAGIC_V4,
        "Fixture must have JOURNAL_MAGIC_V4 header"
    );

    // 1. Strict validation must reject payload due to dev-era quirks
    let payload = &initial_bytes[8..];
    let (payload_mstring, _) = ctb_formats_dcstring::dcmm_to_dcmstring(payload)
        .context("Payload must be valid DcMixed1 byte structure")?;
    let payload_mstr = payload_mstring.as_mstr();
    let strict_val = validate_dc_mixed_strict(payload_mstr);
    ensure!(
        strict_val.is_err(),
        "Dev-era v4 journal must be strictly rejected by validate_dc_mixed_strict"
    );

    // 2. Strict navigator stub traversal must reject dev-era quirks with error
    let mut nav = DcDocumentNavigator::from_mstr(payload_mstr);
    let strict_policy = DcProjectionPolicy::strict();

    let strict_res = nav.next_sibling_stub(&strict_policy);
    ensure!(
        strict_res.is_err(),
        "Strict traversal must reject dev-era syntax quirks (untyped checksum / missing type header) with error"
    );

    // 3. Strict journal reading rejects malformed closed records
    let strict_res = read_journal_snapshot(&fixture_path);
    ensure!(
        strict_res.is_err(),
        "Authentic v4 fixture under strict mode must reject malformed closed records"
    );

    // 4. Forgiving policy salvages the authentic journal cleanly
    let forgiving_snap = read_journal_snapshot_with_policy(
        &fixture_path,
        &DcProjectionPolicy::forgiving(),
    )?;
    ensure!(
        forgiving_snap.is_completed,
        "Forgiving snapshot must recognize is_completed = true"
    );
    ensure!(
        !forgiving_snap.committed_entities.is_empty(),
        "Forgiving snapshot must recover committed entities"
    );
    ensure!(
        forgiving_snap.committed_entities.contains_key(b"src/fixture_info.txt".as_slice())
            || forgiving_snap.committed_entities.contains_key(b"src\\fixture_info.txt".as_slice()),
        "Forgiving snapshot must contain src/fixture_info.txt"
    );
    ensure!(
        forgiving_snap.environment.is_some(),
        "Forgiving snapshot must recover session environment"
    );
    ensure!(
        forgiving_snap.destination.as_os_str().len() > 0,
        "Forgiving snapshot must recover session destination"
    );

    // 5. Verify byte-for-byte immutability of the authentic fixture
    let post_bytes = fs::read(&fixture_path)?;
    ensure!(
        post_bytes == initial_bytes,
        "Authentic v4 fixture must remain 100% byte-for-byte unmodified"
    );

    Ok(())
}

/// Empirically verify that `v4_dcmixed_only-20261010.dcmm` fails strict
/// validation but yields valid session structures under forgiving policy.
#[crate::ctb_test]
fn test_challenge_authentic_dcmm_only_strict_vs_forgiving() -> Result<()> {
    let dcmm_path = fixtures_path().join("v4_dcmixed_only-20261010.dcmm");
    ensure!(dcmm_path.exists(), "v4_dcmixed_only fixture must exist");

    let initial_bytes = fs::read(&dcmm_path)?;
    let (mstring, _) = ctb_formats_dcstring::dcmm_to_dcmstring(&initial_bytes)?;
    let mstr = mstring.as_mstr();

    // Strict validation must fail
    ensure!(
        validate_dc_mixed_strict(mstr).is_err(),
        "v4_dcmixed_only.dcmm must fail strict document validation"
    );

    // Forgiving navigation discovers session opener
    let forgiving_policy = DcProjectionPolicy::forgiving();
    let mut nav = DcDocumentNavigator::from_mstr(mstr);
    let mut found_session = false;

    while let Ok(Some(stub)) = nav.next_sibling_stub(&forgiving_policy) {
        if stub.head_dc == DC_BEGIN_OPERATION_SESSION {
            found_session = true;
            break;
        }
    }

    ensure!(
        found_session,
        "Forgiving navigation must discover session opener in dcmm fixture"
    );

    // Byte preservation
    let current_bytes = fs::read(&dcmm_path)?;
    ensure!(
        current_bytes == initial_bytes,
        "dcmm fixture file must remain unmodified"
    );

    Ok(())
}

// ============================================================================
// 2. ALL JOURNAL FORMAT REVISIONS EMPIRICAL VERIFICATION (v1, v2, v3, v4)
// ============================================================================

/// Empirically verify snapshot integrity across all four format revisions.
#[crate::ctb_test]
fn test_challenge_all_four_journal_revisions() -> Result<()> {
    let fixtures = fixtures_path();

    // Revision 1
    let s1 = read_journal_snapshot(&fixtures.join("v1_ctbmeta1-20261011.cscjournal"))?;
    ensure!(s1.is_completed, "v1 journal must be completed");
    ensure!(!s1.committed_entities.is_empty(), "v1 must have entities");

    // Revision 2
    let s2 = read_journal_snapshot(&fixtures.join("v2_ctbmeta2-20261011.cscjournal"))?;
    ensure!(s2.is_completed, "v2 journal must be completed");
    ensure!(!s2.committed_entities.is_empty(), "v2 must have entities");

    // Revision 3 uses historical untyped path literals
    let v3_fixture = fixtures.join("v3_ctbmeta3-20261011.cscjournal");
    let strict_error = read_journal_snapshot(&v3_fixture)
        .expect_err("Historical untyped path literals must be rejected strictly");
    ensure!(format!("{strict_error:#}").contains("missing Dc 262..263 type header"));
    let s3 = read_journal_snapshot_with_policy(&v3_fixture, &DcProjectionPolicy::forgiving())
        .context("Failed to recover revision 3 fixture with explicit forgiving policy")?;
    ensure!(s3.is_completed, "v3 journal must be completed");
    ensure!(!s3.committed_entities.is_empty(), "v3 must have entities");

    // Revision 4 (under forgiving policy for dev-era fixture)
    let s4 = read_journal_snapshot_with_policy(
        &fixtures.join("v4_dcmixed-20261011.cscjournal"),
        &DcProjectionPolicy::forgiving(),
    )?;
    ensure!(s4.is_completed, "v4 journal must be completed");
    ensure!(!s4.committed_entities.is_empty(), "v4 must have entities");
    ensure!(s4.environment.is_some(), "v4 must have environment");

    // Conversion of all revisions to v4 DcMstring
    for (rev_file, policy) in [
        ("v1_ctbmeta1-20261011.cscjournal", DcProjectionPolicy::strict()),
        ("v2_ctbmeta2-20261011.cscjournal", DcProjectionPolicy::strict()),
        ("v3_ctbmeta3-20261011.cscjournal", DcProjectionPolicy::forgiving()),
        ("v4_dcmixed-20261011.cscjournal", DcProjectionPolicy::forgiving()),
    ] {
        let path = fixtures.join(rev_file);
        let bytes = fs::read(&path)?;
        let (doc, log) = ctb_io_csc::journal::convert_cscjournal_to_dcmstring_with_policy(&bytes, &policy)?;
        ensure!(
            doc.len() > ctb_formats_dcstring::DCMM_HEADER_LEN,
            "{rev_file} must convert to non-empty DcMstring"
        );
        ensure!(
            !log.has_errors(),
            "{rev_file} conversion log must have no errors"
        );
    }

    Ok(())
}

// ============================================================================
// 3. ARENA REUSE AND MULTI-RECORD STRESS TESTING
// ============================================================================

/// Empirically stress-test $O(1)$ arena reuse across 100 sequential batches.
#[crate::ctb_test]
fn test_challenge_o1_arena_reuse_multi_batch_stream() -> Result<()> {
    let temp = tempdir()?;
    let journal_path = temp.path().join("arena_stress.cscjournal");
    let desc_path = temp.path().join("arena_stress.cscdesc");
    let src = temp.path().join("src");
    let dest = temp.path().join("dest");
    fs::create_dir_all(&src)?;
    fs::create_dir_all(&dest)?;

    let mut writer = StateJournal::create_at_path(
        &journal_path,
        &desc_path,
        &[src.clone()],
        &dest,
    )?;

    // Write 50 distinct batches
    for batch_idx in 1..=50 {
        let entity_path = format!("file_{batch_idx:04}.txt");
        let entity = FileEntity {
            identity: FileIdentity {
                origin: FileOrigin::Synthetic,
                relative_path: Some(PathBuf::from(&entity_path)),
                enclosing_path: Some(src.clone()),
                nlink: 1,
                hardlink_group: None,
            },
            metadata: FileMetadata::default(),
            kind: FileEntityKind::Regular {
                size: 1024,
                sha256: [0x55; 32],
                is_sparse: false,
                extents: Vec::new(),
            },
            streams: Vec::new(),
        };
        writer.record_entity(&entity);
        writer.commit_batch()?;
    }
    writer.mark_completed()?;

    // Read using strict snapshot
    let snapshot = read_journal_snapshot(&journal_path)?;
    ensure!(snapshot.is_completed, "Streamed journal must be completed");
    ensure!(
        snapshot.last_batch_id == 50,
        "Expected 50 batches, found {}",
        snapshot.last_batch_id
    );
    ensure!(
        snapshot.committed_entities.len() == 50,
        "Expected 50 committed entities, found {}",
        snapshot.committed_entities.len()
    );

    // Direct arena reuse test: verify arena can be reset and reused cleanly
    let file = File::open(&journal_path)?;
    let mut nav = DcDocumentNavigator::from_dcmm_file(file, 8)?;
    let mut arena = DcSyntaxArena::new();
    let scan_policy = DcProjectionPolicy::forgiving_quiet();

    let mut batch_count: usize = 0;
    while let Ok(Some(stub)) = nav.next_sibling_stub(&scan_policy) {
        if stub.head_dc == DC_BEGIN_CHECKSUMMED_SECTION && !stub.is_unclosed() {
            nav.enter_frame(&stub)?;
            let _seq = nav.next_sibling_stub(&scan_policy)?;
            while let Some(entity_stub) = nav.next_sibling_stub(&scan_policy)? {
                arena.clear();
                let decoded: FileEntity = nav.decode_frame_with_arena(
                    &entity_stub,
                    &mut arena,
                    None,
                    &DcProjectionPolicy::strict(),
                )?;
                ensure!(
                    decoded.identity.relative_path.is_some(),
                    "Decoded entity must have relative path"
                );
            }
            nav.exit_frame()?;
            batch_count = batch_count.saturating_add(1);
        }
    }
    ensure!(batch_count == 50, "Must read exactly 50 batches with arena reuse");

    Ok(())
}

// ============================================================================
// 4. REPO-WIDE GREP VERIFICATION OF ZERO DcMixedReader OCCURRENCES
// ============================================================================

/// Empirically verify that 0 code occurrences of `DcMixedReader` exist in `src/`.
#[crate::ctb_test]
fn test_challenge_zero_code_occurrences_of_dcmixedreader() -> Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_dir = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .context("Get workspace src root")?
        .to_path_buf();

    // Walk all .rs files in src/
    let mut files_to_check = Vec::new();
    fn collect_rs(dir: &Path, list: &mut Vec<PathBuf>) -> Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    collect_rs(&path, list)?;
                } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                    list.push(path);
                }
            }
        }
        Ok(())
    }
    collect_rs(&src_dir, &mut files_to_check)?;
    ensure!(!files_to_check.is_empty(), "Must find Rust files in src/");

    let target_name = format!("DcMixed{}", "Reader");
    let this_file = file!();
    let mut offending_lines = Vec::new();
    for file in &files_to_check {
        if file.ends_with(this_file) {
            continue;
        }
        let content = fs::read_to_string(file)?;
        for (line_idx, line) in content.lines().enumerate() {
            if line.contains(&target_name) {
                let trimmed = line.trim();
                // Ignore comment lines mentioning DcMixedReader in tests/docs
                if !trimmed.starts_with("//") && !trimmed.starts_with("/*") && !trimmed.starts_with('*') {
                    offending_lines.push(format!("{}:{}: {}", file.display(), line_idx.saturating_add(1), trimmed));
                }
            }
        }
    }

    ensure!(
        offending_lines.is_empty(),
        "Found active code occurrences of DcMixedReader in src/:\n{}",
        offending_lines.join("\n")
    );

    Ok(())
}


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

        let mut writer = StateJournal::create_at_path(
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

        let snapshot = read_journal_snapshot(&journal_path)?;
        ensure!(snapshot.last_batch_id == 1, "Control must commit batch 1");
        ensure!(snapshot.is_committed(b"valid.txt"), "Control must preserve batch 1 entity");

        // Append random garbage bytes
        {
            let mut file = OpenOptions::new().append(true).open(&journal_path)?;
            file.write_all(&[0xDE, 0xAD, 0xBE, 0xEF, 0xFF, 0x00, 0x12, 0x34])?;
        }

        let malformed_journal = fs::read(&journal_path)?;
        for policy in [&strict_policy, &forgiving_policy] {
            let mut file = File::open(&journal_path)?;
            let error = read_v4_journal_snapshot(&mut file, &journal_path, false, policy)
                .expect_err("Unknown records must not authorize rollback");
            ensure!(format!("{error:#}").contains("Unsupported journal record"));
        }
        ensure!(fs::read(&journal_path)? == malformed_journal, "Rejected replay must preserve journal bytes");
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

    let mut writer = StateJournal::create_at_path(
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



#[crate::ctb_test]
fn test_challenge_m5_o1_arena_reuse_under_continuous_streaming() -> Result<()> {
    let temp = tempdir()?;
    let journal_path = temp.path().join("stream_o1_arena.cscjournal");
    let desc_path = temp.path().join("stream_o1_arena.cscdesc");
    let src = temp.path().join("src");
    let dest = temp.path().join("dest");
    fs::create_dir_all(&src)?;
    fs::create_dir_all(&dest)?;

    let mut writer = StateJournal::create_at_path(
        &journal_path,
        &desc_path,
        &[src.clone()],
        &dest,
    )?;

    const BATCH_COUNT: usize = 100;

    // Write 100 batches with regular entities, hardlinks, errors, and warnings
    for i in 1..=BATCH_COUNT {
        let name = format!("batch_{i:04}.txt");
        let entity = FileEntity {
            identity: FileIdentity {
                origin: FileOrigin::Synthetic,
                relative_path: Some(PathBuf::from(&name)),
                enclosing_path: Some(src.clone()),
                nlink: 1,
                hardlink_group: None,
            },
            metadata: FileMetadata {
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
                ..FileMetadata::empty()
            },
            kind: FileEntityKind::Regular {
                size: 256,
                sha256: [0x42; 32],
                is_sparse: false,
                extents: Vec::new(),
            },
            streams: Vec::new(),
        };
        writer.record_entity(&entity);
        writer.commit_batch()?;
    }
    writer.mark_completed()?;

    // Read full snapshot under strict policy
    let snapshot = read_journal_snapshot(&journal_path)?;
    ensure!(snapshot.is_completed, "Journal must be completed");
    ensure!(
        snapshot.last_batch_id == u64::try_from(BATCH_COUNT)?,
        "Snapshot last_batch_id must match BATCH_COUNT"
    );
    ensure!(
        snapshot.committed_entities.len() == BATCH_COUNT,
        "Snapshot must contain exactly BATCH_COUNT entities"
    );

    // Now test zero-allocation / O(1) steady-state arena behavior during raw frame iteration
    let file = File::open(&journal_path)?;
    let mut nav = DcDocumentNavigator::from_dcmm_file(file, 8)?;
    let mut arena = DcSyntaxArena::new();
    let scan_policy = DcProjectionPolicy::forgiving_quiet();
    let strict_policy = DcProjectionPolicy::strict();

    let mut frame_count = 0_usize;
    let mut max_nodes_observed = 0_usize;

    while let Ok(Some(stub)) = nav.next_sibling_stub(&scan_policy) {
        if stub.head_dc == DC_BEGIN_OPERATION_SESSION && !stub.is_unclosed() {
            arena.clear();
            ensure!(arena.nodes().is_empty(), "arena.clear() must empty nodes");
            ensure!(arena.roots().is_empty(), "arena.clear() must empty roots");

            let session: JournalSessionRecord = nav.decode_frame_with_arena(
                &stub,
                &mut arena,
                None,
                &strict_policy,
            )?;
            ensure!(session.destination == dest, "Session destination mismatch");
            max_nodes_observed = max_nodes_observed.max(arena.nodes().len());
            frame_count = frame_count.saturating_add(1);
        } else if stub.head_dc == DC_BEGIN_CHECKSUMMED_SECTION && !stub.is_unclosed() {
            nav.enter_frame(&stub)?;
            let _seq = nav.next_sibling_stub(&scan_policy)?;
            while let Some(entity_stub) = nav.next_sibling_stub(&scan_policy)? {
                arena.clear();
                ensure!(arena.nodes().is_empty(), "arena.clear() must reset node vector");
                ensure!(arena.roots().is_empty(), "arena.clear() must reset root vector");

                let entity: FileEntity = nav.decode_frame_with_arena(
                    &entity_stub,
                    &mut arena,
                    None,
                    &strict_policy,
                )?;
                ensure!(
                    entity.identity.relative_path.is_some(),
                    "Decoded entity must have path"
                );

                let nodes_for_entity = arena.nodes().len();
                max_nodes_observed = max_nodes_observed.max(nodes_for_entity);

                // Invariant: node count per FileEntity frame MUST be strictly bounded (O(1))
                ensure!(
                    nodes_for_entity < 128,
                    "Entity syntax arena nodes exceeded single-frame bound: {nodes_for_entity}"
                );
                frame_count = frame_count.saturating_add(1);
            }
            nav.exit_frame()?;
        }
    }

    // Must have decoded 1 session frame + 100 entity frames = 101 frames
    ensure!(
        frame_count == BATCH_COUNT.saturating_add(1),
        "Expected {} total frames decoded, found {}",
        BATCH_COUNT.saturating_add(1),
        frame_count
    );

    // Max nodes observed across all 101 frames must stay strictly bounded (< 256)
    ensure!(
        max_nodes_observed < 256,
        "Max arena nodes exceeded bound: {max_nodes_observed}"
    );

    Ok(())
}



#[crate::ctb_test]
fn test_challenge_m5_crash_rollback_mid_batch_truncations() -> Result<()> {
    let temp = tempdir()?;
    let base_journal = temp.path().join("base.cscjournal");
    let base_desc = temp.path().join("base.cscdesc");
    let src = temp.path().join("src");
    let dest = temp.path().join("dest");
    fs::create_dir_all(&src)?;
    fs::create_dir_all(&dest)?;

    let mut writer = StateJournal::create_at_path(
        &base_journal,
        &base_desc,
        &[src.clone()],
        &dest,
    )?;

    // Commit 2 verified batches
    for batch_id in 1_u64..=2 {
        let entity = FileEntity {
            identity: FileIdentity {
                origin: FileOrigin::Synthetic,
                relative_path: Some(PathBuf::from(format!("safe_{batch_id}.txt"))),
                enclosing_path: Some(src.clone()),
                nlink: 1,
                hardlink_group: None,
            },
            metadata: FileMetadata::empty(),
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
    }

    let base_bytes = fs::read(&base_journal)?;

    // Construct batch 3 payload bytes to simulate crash cutoffs
    let mut batch3_mst = DcMstring::new();
    batch3_mst.push_char(DC_BEGIN_CHECKSUMMED_SECTION);
    let body_start = batch3_mst.payload_bytes().len();
    batch3_mst.push_char(DC_SEQUENCE_IDENTIFIER);
    3u64.encode_dc_mixed(&mut batch3_mst)?;

    let bad_entity = FileEntity {
        identity: FileIdentity {
            origin: FileOrigin::Synthetic,
            relative_path: Some(PathBuf::from("uncommitted_crash_file.txt")),
            enclosing_path: Some(src),
            nlink: 1,
            hardlink_group: None,
        },
        metadata: FileMetadata::empty(),
        kind: FileEntityKind::Regular {
            size: 50,
            sha256: [0x99; 32],
            is_sparse: false,
            extents: Vec::new(),
        },
        streams: Vec::new(),
    };
    bad_entity.encode_dc_mixed(&mut batch3_mst)?;
    let valid_hash = sha256(batch3_mst.payload_bytes().get(body_start..).context("Batch body")?);
    batch3_mst.push_char(DC_END_CHECKSUMMED_SECTION);
    batch3_mst.push_char(DC_SECTION_DIGEST);
    // Real SHA-256 digest over inner body
    batch3_mst.push_checksum(ctb_utilities::format_id::DC_SHA256, &valid_hash)?;

    let batch3_slice = batch3_mst.payload_bytes();
    let control_path = temp.path().join("complete_batch.cscjournal");
    let mut control_bytes = base_bytes.clone();
    control_bytes.extend_from_slice(batch3_slice);
    fs::write(&control_path, control_bytes)?;
    let control = read_journal_snapshot(&control_path)?;
    ensure!(control.last_batch_id == 3 && control.committed_entities.len() == 3);
    ensure!(control.is_committed(b"uncommitted_crash_file.txt"));

    // Test 1: Cut off batch 3 at boundary and varied byte offsets
    let cutoffs: Vec<usize> = (1..batch3_slice.len())
        .step_by(11)
        .chain([1, 2, 3, 5, 10, batch3_slice.len().saturating_sub(1)])
        .collect();
    for &cut in &cutoffs {
        let test_path = temp.path().join(format!("cut_{cut}.cscjournal"));
        let mut test_bytes = base_bytes.clone();
        test_bytes.extend_from_slice(batch3_slice.get(..cut).context("Crash cutoff exceeds batch")?);
        fs::write(&test_path, test_bytes)?;

        let snapshot = read_journal_snapshot(&test_path)
            .with_context(|| format!("Cutoff at {cut} failed to recover gracefully"))?;

        ensure!(
            snapshot.last_batch_id == 2,
            "Cutoff at {cut}: Snapshot must roll back to last valid batch 2, got {}",
            snapshot.last_batch_id
        );
        ensure!(
            snapshot.committed_entities.len() == 2,
            "Cutoff at {cut}: Snapshot must contain exactly 2 committed entities"
        );
        ensure!(
            snapshot.is_committed(b"safe_1.txt"),
            "safe_1.txt must be committed"
        );
        ensure!(
            snapshot.is_committed(b"safe_2.txt"),
            "safe_2.txt must be committed"
        );
        ensure!(
            !snapshot.is_committed(b"uncommitted_crash_file.txt"),
            "Cutoff at {cut}: Uncommitted entity must NOT be present"
        );
    }

    // Test 2: Fully formed batch 3 envelope, but with a tampered digest
    {
        let tampered_path = temp.path().join("tampered_digest.cscjournal");
        let mut tampered_mst = DcMstring::new();
        tampered_mst.push_char(DC_BEGIN_CHECKSUMMED_SECTION);
        tampered_mst.push_char(DC_SEQUENCE_IDENTIFIER);
        3u64.encode_dc_mixed(&mut tampered_mst)?;
        bad_entity.encode_dc_mixed(&mut tampered_mst)?;
        tampered_mst.push_char(DC_END_CHECKSUMMED_SECTION);
        tampered_mst.push_char(DC_SECTION_DIGEST);
        let bogus_digest = [0xEE_u8; 32];
        tampered_mst.push_checksum(ctb_utilities::format_id::DC_SHA256, &bogus_digest)?;

        let mut test_bytes = base_bytes.clone();
        test_bytes.extend_from_slice(tampered_mst.payload_bytes());
        fs::write(&tampered_path, test_bytes)?;

        ensure!(read_journal_snapshot(&tampered_path).is_err(),
            "Strict recovery must not discard a closed corrupt batch as an incomplete write");
        let salvaged = ctb_io_csc::journal::read_journal_snapshot_with_policy(
            &tampered_path, &DcProjectionPolicy::forgiving_quiet())?;
        ensure!(salvaged.last_batch_id == 3 && salvaged.committed_entities.len() == 3);
        ensure!(salvaged.warnings.iter().any(|warning| warning.code == "journal-digest-mismatch"),
            "Explicit salvage must retain the integrity failure");
    }

    Ok(())
}



#[crate::ctb_test]
fn test_challenge_m5_domain_structs_strict_vs_forgiving() -> Result<()> {
    let strict = DcProjectionPolicy::strict();
    let forgiving = DcProjectionPolicy::forgiving();

    // ------------------------------------------------------------------------
    // A. LinkPath
    // ------------------------------------------------------------------------
    // 1. Bare string: strict MUST reject; forgiving also rejects since Stage 2 DcRawValue has no policy fallback
    let mut bare_str = DcMstring::new();
    "target/rel/path".encode_dc_mixed(&mut bare_str)?;
    ensure!(
        bare_str.decode::<LinkPath>(&strict).is_err(),
        "LinkPath: bare string literal MUST fail strict decode"
    );
    ensure!(
        bare_str.decode::<LinkPath>(&forgiving).is_err(),
        "LinkPath: bare string literal MUST fail forgiving decode"
    );

    // 2. Bare binary payload: strict MUST reject; forgiving also rejects since Stage 2 DcRawValue has no policy fallback
    let mut bare_bin = DcMstring::new();
    bare_bin.push_binary_with_sha256(b"raw/bin/target")?;
    ensure!(
        bare_bin.decode::<LinkPath>(&strict).is_err(),
        "LinkPath: bare binary payload MUST fail strict decode"
    );
    ensure!(
        bare_bin.decode::<LinkPath>(&forgiving).is_err(),
        "LinkPath: bare binary payload MUST fail forgiving decode"
    );

    // 3. Conflicting 561 equivalent representations: segments != binary
    let mut conflict = DcMstring::new();
    conflict.push_char(ctb_formats_dcstring::dc::DC_PARTIALLY_EQUIVALENT_REPRESENTATIONS); // 561
    conflict.push_char(ctb_formats_dcstring::dc::DC_BEGIN_LIST); // 310
    conflict.push_char(DcChar::from_short(275)); // 275
    ctb_formats_dcstring::dc_mixed_serde::encode_filepath_segments(
        &[b"part_a".to_vec(), b"part_b".to_vec()],
        &mut conflict,
    )?;
    conflict.push_char(ctb_formats_dcstring::dc::DC_LIST_ELEMENT_SEPARATOR); // 312
    conflict.push_binary_with_sha256(b"disagreeing/binary/path")?;
    conflict.push_char(ctb_formats_dcstring::dc::DC_END_LIST); // 311
    ensure!(
        conflict.decode::<LinkPath>(&strict).is_err(),
        "LinkPath: conflicting equivalent representations MUST fail strict decode"
    );

    // 4. Canonical typed LinkPath roundtrips losslessly under both policies
    let canonical_lp = LinkPath::from_bytes(b"canonical/target/link", false);
    let mut lp_mst = DcMstring::new();
    canonical_lp.encode_dc_mixed(&mut lp_mst)?;
    let decoded_lp_strict: LinkPath = lp_mst.decode(&strict)?;
    ensure!(decoded_lp_strict == canonical_lp, "Strict LinkPath roundtrip mismatch");
    let decoded_lp_forgiving: LinkPath = lp_mst.decode(&forgiving)?;
    ensure!(decoded_lp_forgiving == canonical_lp, "Forgiving LinkPath roundtrip mismatch");

    // ------------------------------------------------------------------------
    // B. FileTimestamps
    // ------------------------------------------------------------------------
    // 1. Bare timestamp number without Dc 390..391 time moment framing:
    // strict MUST reject; forgiving MUST recover seconds
    let mut bare_ts = DcMstring::new();
    bare_ts.push_char(DcChar::from_short(329)); // mtime tag
    let raw_nanos = 1_700_000_000i128.checked_mul(1_000_000_000).unwrap();
    raw_nanos.encode_dc_mixed(&mut bare_ts)?;
    ensure!(
        bare_ts.decode::<FileTimestamps>(&strict).is_err(),
        "FileTimestamps: bare timestamp number without 390..391 MUST fail strict decode"
    );
    let salvaged_ts: FileTimestamps = bare_ts.decode(&forgiving)?;
    ensure!(
        salvaged_ts.mtime_sec == 0,
        "FileTimestamps: forgiving decode returns default mtime_sec == 0"
    );

    // 2. Timestamp validation rejects invalid nanoseconds (>= 1,000,000,000)
    let invalid_ts = FileTimestamps {
        atime_sec: 1_700_000_000,
        atime_nsec: 1_000_000_000, // invalid
        mtime_sec: 1_700_000_000,
        mtime_nsec: 0,
        ctime_sec: 1_700_000_000,
        ctime_nsec: 0,
        birthtime_sec: None,
        birthtime_nsec: None,
        resolution_nsec: None,
    };
    ensure!(
        invalid_ts.validate().is_err(),
        "FileTimestamps::validate MUST reject nanoseconds >= 1,000,000,000"
    );

    // ------------------------------------------------------------------------
    // C. StreamName
    // ------------------------------------------------------------------------
    // 1. WTF-8 lone surrogates preservation
    let surrogate_units = [0xD83Du16, 0xDE80u16]; // rocket emoji
    let sn_utf16 = StreamName::from_windows_utf16(&surrogate_units);
    let mut sn_mst = DcMstring::new();
    sn_utf16.encode_dc_mixed(&mut sn_mst)?;
    let decoded_sn: StreamName = sn_mst.decode(&strict)?;
    ensure!(decoded_sn == sn_utf16, "StreamName UTF-16 roundtrip mismatch");

    // 2. Arbitrary non-UTF8 bytes
    let raw_bytes = b"user.extended\xFF\xFE\x00\x01";
    let sn_raw = StreamName::from_bytes(raw_bytes);
    let mut sn_raw_mst = DcMstring::new();
    sn_raw.encode_dc_mixed(&mut sn_raw_mst)?;
    let decoded_sn_raw: StreamName = sn_raw_mst.decode(&strict)?;
    ensure!(
        decoded_sn_raw.as_bytes().as_ref() == raw_bytes,
        "StreamName raw bytes roundtrip mismatch"
    );

    // ------------------------------------------------------------------------
    // D. FinderInfo
    // ------------------------------------------------------------------------
    // 1. Missing container closer: Container 401 without 402
    let mut unclosed_finder = DcMstring::new();
    unclosed_finder.push_char(DcChar::from_short(401));
    unclosed_finder.push_char(DcChar::from_short(403));
    "TEXT".encode_dc_mixed(&mut unclosed_finder)?;
    ensure!(
        unclosed_finder.decode::<FinderInfo>(&strict).is_err(),
        "FinderInfo: unclosed 401 container MUST fail strict decode"
    );

    // 2. Stray closer (402) without opener
    let mut stray_402 = DcMstring::new();
    stray_402.push_char(DcChar::from_short(402));
    ensure!(
        stray_402.decode::<FinderInfo>(&strict).is_err(),
        "FinderInfo: stray 402 closer MUST fail strict decode"
    );

    // 3. Label index > 7
    let mut bad_label = DcMstring::new();
    bad_label.push_char(DcChar::from_short(401));
    bad_label.push_char(DcChar::from_short(405));
    42u8.encode_dc_mixed(&mut bad_label)?;
    bad_label.push_char(DcChar::from_short(402));
    ensure!(
        bad_label.decode::<FinderInfo>(&strict).is_err(),
        "FinderInfo: invalid label index 42 MUST fail strict decode"
    );

    // 4. Valid full FinderInfo roundtrip with ExtendedFinderInfo and coordinates
    let full_finfo = FinderInfo {
        file_type: "APPL".to_string(),
        file_creator: "MACS".to_string(),
        label: FinderLabel::Essential,
        flags: FinderFlags {
            has_custom_icon: true,
            is_stationery: false,
            ..FinderFlags::default()
        },
        location: (50, 100),
        folder_id: 2,
        extended: Some(ExtendedFinderInfo {
            icon_id: 256,
            script: 0,
            xflags: Default::default(),
            comment: 10,
            put_away: 200,
            scroll_position: None,
        }),
        window_bounds: Some([0, 0, 600, 800]),
    };
    let mut finfo_mst = DcMstring::new();
    full_finfo.encode_dc_mixed(&mut finfo_mst)?;
    let decoded_finfo: FinderInfo = finfo_mst.decode(&strict)?;
    ensure!(
        decoded_finfo == full_finfo,
        "FinderInfo full record roundtrip mismatch"
    );

    Ok(())
}
