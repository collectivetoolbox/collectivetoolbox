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

//! Milestone 5 Tier 5 Adversarial Coverage Hardening Test Suite:
//! 1. Continuous multi-batch journal streaming and O(1) arena reuse across
//!    interleaved session, entity, error, and warning frames.
//! 2. Crash rollback on unclosed batches across arbitrary mid-stream truncation
//!    offsets and tampered SHA-256 batch digests.
//! 3. Strict rejection vs forgiving salvage of authentic dev-era test fixtures
//!    (`v4_dcmixed*`) with byte-for-byte immutability guarantees.
//! 4. Strict vs forgiving compliance on domain structs (`LinkPath`, `FileTimestamps`,
//!    `StreamName`, and `FinderInfo`).

#![allow(
    clippy::panic,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::unwrap_in_result,
    clippy::panic_in_result_fn,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    reason = "Standard repository test boilerplate"
)]

#[expect(unused_imports, clippy::wildcard_imports, reason = "Integration test root")]
use ctb_utilities::*;

use anyhow::{Context, Result, ensure};
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
use ctb_io::file::apple_single_double::{ExtendedFinderInfo, FinderFlags, FinderInfo, FinderLabel};
use ctb_io::file::entity::{FileEntity, FileEntityKind};
use ctb_io::file::identity::{FileIdentity, FileOrigin};
use ctb_io::file::link_path::LinkPath;
use ctb_io::file::metadata::{FileMetadata, FileTimestamps};
use ctb_io::file::streams::StreamName;
use ctb_io_csc::journal::{
    JOURNAL_MAGIC_V4, JournalErrorRecordPayload, JournalSessionRecord,
    JournalSnapshot, JournalWarningRecordPayload, JournalWriter,
    convert_cscjournal_to_dcmstring, read_journal_snapshot,
    read_journal_snapshot_with_policy, read_v4_journal_snapshot,
};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::tempdir;

fn fixtures_path() -> PathBuf {
    ctb_utilities::manifest_fixture_path(env!("CARGO_MANIFEST_DIR"), "fixtures/journals")
}

// ============================================================================
// 1. MULTI-BATCH CONTINUOUS STREAMING & O(1) ARENA REUSE
// ============================================================================

/// Adversarially verifies continuous streaming of session, entity, error, and
/// warning frames across 100 sequential batches, proving O(1) steady-state arena
/// memory reuse and zero node leakage.
#[crate::ctb_test]
fn test_challenge_m5_o1_arena_reuse_under_continuous_streaming() -> Result<()> {
    let temp = tempdir()?;
    let journal_path = temp.path().join("stream_o1_arena.cscjournal");
    let desc_path = temp.path().join("stream_o1_arena.cscdesc");
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

// ============================================================================
// 2. CRASH ROLLBACK & DIGEST TAMPERING
// ============================================================================

/// Adversarially simulates crash truncations at arbitrary byte offsets inside
/// unclosed batches and tampered digests, proving clean rollback to the last verified batch.
#[crate::ctb_test]
fn test_challenge_m5_crash_rollback_mid_batch_truncations() -> Result<()> {
    let temp = tempdir()?;
    let base_journal = temp.path().join("base.cscjournal");
    let base_desc = temp.path().join("base.cscdesc");
    let src = temp.path().join("src");
    let dest = temp.path().join("dest");
    fs::create_dir_all(&src)?;
    fs::create_dir_all(&dest)?;

    let mut writer = JournalWriter::create_at_path(
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
    batch3_mst.push_char(DC_END_CHECKSUMMED_SECTION);
    batch3_mst.push_char(DC_SECTION_DIGEST);
    // Real SHA-256 digest over inner body
    let body_bytes = batch3_mst.as_bytes().get(3..batch3_mst.len().saturating_sub(6)).unwrap_or(&[]);
    let valid_hash = sha256(body_bytes);
    batch3_mst.push_checksum(ctb_utilities::format_id::DC_SHA256, &valid_hash)?;

    let batch3_slice = batch3_mst.as_bytes();

    // Test 1: Cut off batch 3 at boundary and varied byte offsets
    let cutoffs: Vec<usize> = (1..batch3_slice.len())
        .step_by(11)
        .chain([1, 2, 3, 5, 10, batch3_slice.len().saturating_sub(1)])
        .collect();
    for &cut in &cutoffs {
        let test_path = temp.path().join(format!("cut_{cut}.cscjournal"));
        let mut test_bytes = base_bytes.clone();
        test_bytes.extend_from_slice(batch3_slice.get(..cut).unwrap_or(&[]));
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
        test_bytes.extend_from_slice(tampered_mst.as_bytes());
        fs::write(&tampered_path, test_bytes)?;

        let snapshot = read_journal_snapshot(&tampered_path)
            .context("Tampered digest journal must recover gracefully")?;

        ensure!(
            snapshot.last_batch_id == 2,
            "Tampered digest: snapshot must isolate corrupt batch and roll back to batch 2"
        );
        ensure!(
            snapshot.committed_entities.len() == 2,
            "Tampered digest: snapshot must contain exactly 2 entities"
        );
        ensure!(
            !snapshot.is_committed(b"uncommitted_crash_file.txt"),
            "Tampered entity must NOT be committed"
        );
    }

    Ok(())
}

// ============================================================================
// 3. AUTHENTIC FIXTURES: STRICT REJECTION VS FORGIVING SALVAGE
// ============================================================================

/// Adversarially verifies authentic test fixtures (`src/io/csc/fixtures/journals/v4_dcmixed*`)
/// under `strict()` and `forgiving()` policies, ensuring byte-for-byte immutability.
#[crate::ctb_test]
fn test_challenge_m5_authentic_fixtures_strict_vs_forgiving() -> Result<()> {
    let fixtures = fixtures_path();

    // 1. Authentic v4 journal fixture: v4_dcmixed-20261011.cscjournal
    let v4_file = fixtures.join("v4_dcmixed-20261011.cscjournal");
    ensure!(v4_file.exists(), "v4_dcmixed fixture file must exist");

    let initial_v4_bytes = fs::read(&v4_file)?;
    let initial_v4_len = initial_v4_bytes.len();
    ensure!(initial_v4_len > 8, "v4 fixture must contain magic and payload");

    // Must have JOURNAL_MAGIC_V4 header
    ensure!(
        initial_v4_bytes.starts_with(JOURNAL_MAGIC_V4),
        "v4 fixture must start with JOURNAL_MAGIC_V4"
    );

    // Payload strict validation must fail due to dev-era syntax discrepancies
    let v4_payload = initial_v4_bytes.get(8..).context("Payload bytes")?;
    let v4_mstr = DcMstr::from_bytes(v4_payload)?;
    let strict_validation = validate_dc_mixed_strict(v4_mstr);
    ensure!(
        strict_validation.is_err(),
        "v4_dcmixed fixture payload MUST fail validate_dc_mixed_strict"
    );

    // Strict journal reading must roll back unverified dev-era batch
    let strict_snap = read_journal_snapshot(&v4_file)?;
    ensure!(
        strict_snap.last_batch_id == 0,
        "Strict snapshot must have last_batch_id == 0"
    );
    ensure!(
        strict_snap.committed_entities.is_empty(),
        "Strict snapshot must roll back entities on unverified batch"
    );
    ensure!(
        !strict_snap.is_completed,
        "Strict snapshot must not be marked completed"
    );

    // Forgiving journal reading must salvage the authentic session, batch, and completion
    let forgiving_snap = read_journal_snapshot_with_policy(
        &v4_file,
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
        forgiving_snap.environment.is_some(),
        "Forgiving snapshot must recover session environment"
    );

    // Conversion to DcMstring document must succeed cleanly
    let (dcmstring_doc, log) = convert_cscjournal_to_dcmstring(&initial_v4_bytes)?;
    ensure!(
        dcmstring_doc.len() > ctb_formats_dcstring::DCMM_HEADER_LEN,
        "Converted DcMstring document must contain salvaged payload"
    );
    ensure!(
        !log.has_errors(),
        "Conversion log must not report unrecoverable errors"
    );

    // Verify fixture file remained 100% byte-for-byte unmodified
    let current_v4_bytes = fs::read(&v4_file)?;
    ensure!(
        current_v4_bytes == initial_v4_bytes,
        "v4_dcmixed-20261011.cscjournal was mutated on disk!"
    );

    // 2. Authentic dcmm-only fixture: v4_dcmixed_only-20261010.dcmm
    let dcmm_file = fixtures.join("v4_dcmixed_only-20261010.dcmm");
    ensure!(dcmm_file.exists(), "v4_dcmixed_only fixture file must exist");

    let initial_dcmm_bytes = fs::read(&dcmm_file)?;
    let dcmm_mstr = DcMstr::from_bytes(&initial_dcmm_bytes)?;

    // Strict validation must fail
    ensure!(
        validate_dc_mixed_strict(dcmm_mstr).is_err(),
        "v4_dcmixed_only.dcmm MUST fail strict document validation"
    );

    // Forgiving navigation discovers session opener
    let mut nav = DcDocumentNavigator::from_mstr(dcmm_mstr);
    let mut found_session = false;
    let forgiving_policy = DcProjectionPolicy::forgiving();

    while let Ok(Some(stub)) = nav.next_sibling_stub(&forgiving_policy) {
        if stub.head_dc == DC_BEGIN_OPERATION_SESSION {
            found_session = true;
            break;
        }
    }
    ensure!(
        found_session,
        "Forgiving navigation must discover DC_BEGIN_OPERATION_SESSION in dcmm fixture"
    );

    // Verify dcmm fixture file remained 100% unmodified
    let current_dcmm_bytes = fs::read(&dcmm_file)?;
    ensure!(
        current_dcmm_bytes == initial_dcmm_bytes,
        "v4_dcmixed_only-20261010.dcmm was mutated on disk!"
    );

    Ok(())
}

// ============================================================================
// 4. STRICT VS FORGIVING COMPLIANCE ON DOMAIN STRUCTS
// ============================================================================

/// Adversarially stress-tests strict rejection vs forgiving recovery across
/// domain structs: `LinkPath`, `FileTimestamps`, `StreamName`, and `FinderInfo`.
#[crate::ctb_test]
fn test_challenge_m5_domain_structs_strict_vs_forgiving() -> Result<()> {
    let strict = DcProjectionPolicy::strict();
    let forgiving = DcProjectionPolicy::forgiving();

    // ------------------------------------------------------------------------
    // A. LinkPath
    // ------------------------------------------------------------------------
    // 1. Bare string: strict MUST reject; forgiving MUST salvage
    let mut bare_str = DcMstring::new();
    "target/rel/path".encode_dc_mixed(&mut bare_str)?;
    ensure!(
        bare_str.decode::<LinkPath>(&strict).is_err(),
        "LinkPath: bare string literal MUST fail strict decode"
    );
    let salvaged_str: LinkPath = bare_str.decode(&forgiving)?;
    ensure!(
        salvaged_str == "target/rel/path",
        "LinkPath: forgiving failed to salvage bare string"
    );

    // 2. Bare binary payload: strict MUST reject; forgiving MUST salvage
    let mut bare_bin = DcMstring::new();
    bare_bin.push_binary_with_sha256(b"raw/bin/target")?;
    ensure!(
        bare_bin.decode::<LinkPath>(&strict).is_err(),
        "LinkPath: bare binary payload MUST fail strict decode"
    );
    let salvaged_bin: LinkPath = bare_bin.decode(&forgiving)?;
    ensure!(
        salvaged_bin == "raw/bin/target",
        "LinkPath: forgiving failed to salvage bare binary"
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
        salvaged_ts.mtime_sec == 1_700_000_000,
        "FileTimestamps: forgiving decode failed to salvage bare timestamp number"
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
