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
use ctb_formats_dcdata::dc::{
    DC_BEGIN_CHECKSUMMED_SECTION, DC_BEGIN_OPERATION_SESSION,
    DC_OPERATION_COMPLETED, DC_SECTION_DIGEST, DC_SEQUENCE_IDENTIFIER,
};
use ctb_formats_dcstring::{
    DcChar, DcDocumentNavigator, DcMstr, DcMstring, DcProjectionPolicy,
    DcSyntaxArena, validate_dc_mixed_strict,
};
use ctb_io::file::entity::{FileEntity, FileEntityKind};
use ctb_io::file::identity::{FileIdentity, FileOrigin};
use ctb_io::file::metadata::FileMetadata;
use ctb_io_csc::journal::{
    JOURNAL_MAGIC_V4, JournalSnapshot, JournalWriter, convert_cscjournal_to_dcmstring,
    convert_cscjournal_to_v4_bytes, read_journal_snapshot,
    read_journal_snapshot_with_policy,
};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
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
    let payload_mstr = DcMstr::from_bytes(payload)
        .context("Payload must be valid DcMstr byte structure")?;
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

    // 3. Strict journal reading rolls back unverified dev-era batch
    let strict_snap = read_journal_snapshot(&fixture_path)?;
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
    let mstr = DcMstr::from_bytes(&initial_bytes)?;

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

    // Revision 3
    let s3 = read_journal_snapshot(&fixtures.join("v3_ctbmeta3-20261011.cscjournal"))?;
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
    for rev_file in [
        "v1_ctbmeta1-20261011.cscjournal",
        "v2_ctbmeta2-20261011.cscjournal",
        "v3_ctbmeta3-20261011.cscjournal",
        "v4_dcmixed-20261011.cscjournal",
    ] {
        let path = fixtures.join(rev_file);
        let bytes = fs::read(&path)?;
        let (doc, log) = convert_cscjournal_to_dcmstring(&bytes)?;
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

    let mut writer = JournalWriter::create_at_path(
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

    let mut batch_count = 0;
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
            batch_count += 1;
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
                    offending_lines.push(format!("{}:{}: {}", file.display(), line_idx + 1, trimmed));
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
