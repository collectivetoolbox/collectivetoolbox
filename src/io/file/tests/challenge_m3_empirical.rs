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

//! Empirical Challenge Test Suite for Milestone 3:
//! 1. `FileTimestamps`: multi-root timestamp decodes, isolated timestamp maps,
//!    nanosecond precision, out-of-range epoch seconds.
//! 2. `PlatformRawFlags`: malformed binary payloads, 4-byte/8-byte validation,
//!    non-binary nodes.
//! 3. `NativeMetadata`: pairs vs legacy key-value maps, duplicate keys, missing
//!    fields under strict vs forgiving policies.
//! 4. `AppleMetadata`: container framing, unknown fields, FinderInfo, real name,
//!    comment, backup timestamp.

#[expect(unused_imports, clippy::wildcard_imports, reason = "Integration test root")]
pub use ctb_utilities::*;

use anyhow::{Context, Result, ensure};
use std::collections::BTreeMap;
use ctb_formats_apple_single_double::FinderInfo;
use ctb_formats_dcstring::{
    DcChar, DcDocumentNavigator, DcKvMapEncoding, DcMixedDecode, DcMixedEncode,
    DcMstring, DcProjectionContext, DcProjectionPolicy, DcSyntaxArena,
    DiagnosticAction, assert_dc_roundtrip,
};
use ctb_io_file::metadata::{
    AppleMetadata, FileMetadata, FileTimestamps, NativeMetadata,
    NativeMetadataValue, OsFamily, PlatformRawFlags,
};

// ============================================================================
// 1. FILE TIMESTAMPS: MULTI-ROOT & ISOLATED MAP DECODING
// ============================================================================

#[crate::ctb_test]
fn test_challenge_file_timestamps_multi_root_decode() -> Result<()> {
    let original = FileTimestamps {
        atime_sec: 1_700_000_010,
        atime_nsec: 123_456_789,
        mtime_sec: 1_700_000_020,
        mtime_nsec: 987_654_321,
        ctime_sec: 1_700_000_030,
        ctime_nsec: 555_555_555,
        birthtime_sec: Some(1_700_000_000),
        birthtime_nsec: Some(111_222_333),
        resolution_nsec: Some(100),
    };

    let mut mst = DcMstring::new();
    original.encode_dc_mixed(&mut mst)?;

    // Parse the encoded multi-root buffer directly into a syntax arena.
    let mut nav = DcDocumentNavigator::from_mstr(&mst);
    let mut arena = DcSyntaxArena::new();
    let strict = DcProjectionPolicy::strict();
    nav.parse_document_into_arena(&mut arena, &strict)?;

    // Notice: FileTimestamps encodes as multiple root statements (330, 329, 332, 331, 378).
    let roots = arena.roots();
    ensure!(roots.len() >= 4, "Expected at least 4 root statements for FileTimestamps");

    // Decoding from the first root must discover all sister roots in arena.roots().
    let mut ctx = DcProjectionContext::new(&arena, &strict);
    let decoded = FileTimestamps::decode_dc_node(&mut ctx, roots[0])?;
    ensure!(decoded == original, "Multi-root FileTimestamps decode mismatch");

    // Also verify decode when wrapped in FileMetadata (single root container 321..322).
    let meta = FileMetadata {
        timestamps: Some(original),
        ..FileMetadata::empty()
    };
    assert_dc_roundtrip(&meta)?;

    Ok(())
}

#[crate::ctb_test]
fn test_challenge_file_timestamps_isolated_map_decoding() -> Result<()> {
    // Construct a document where FileTimestamps is nested inside an isolated container
    // rather than at document root level.
    let ts = FileTimestamps {
        atime_sec: 100,
        atime_nsec: 200,
        mtime_sec: 300,
        mtime_nsec: 400,
        ctime_sec: 500,
        ctime_nsec: 600,
        birthtime_sec: None,
        birthtime_nsec: None,
        resolution_nsec: None,
    };

    // Container [ 321 ... 322 ] containing timestamps statements.
    let mut mst = DcMstring::new();
    mst.push_char(DcChar::from_short(321)); // BEGIN_FILE_RECORD
    ts.encode_dc_mixed(&mut mst)?;
    mst.push_char(DcChar::from_short(322)); // END_FILE_RECORD

    let mut nav = DcDocumentNavigator::from_mstr(&mst);
    let mut arena = DcSyntaxArena::new();
    let strict = DcProjectionPolicy::strict();
    nav.parse_document_into_arena(&mut arena, &strict)?;

    let roots = arena.roots();
    ensure!(roots.len() == 1, "Expected single container root");

    // Projecting FileTimestamps by passing the container node ID:
    let mut ctx = DcProjectionContext::new(&arena, &strict);
    let decoded = FileTimestamps::decode_dc_node(&mut ctx, roots[0])?;
    ensure!(decoded == ts, "Isolated container FileTimestamps decode mismatch");

    Ok(())
}

#[crate::ctb_test]
fn test_challenge_file_timestamps_standalone_decode_behavior() -> Result<()> {
    let ts = FileTimestamps {
        atime_sec: 1_700_000_010,
        atime_nsec: 123_456_789,
        mtime_sec: 1_700_000_020,
        mtime_nsec: 987_654_321,
        ctime_sec: 1_700_000_030,
        ctime_nsec: 555_555_555,
        birthtime_sec: Some(1_700_000_000),
        birthtime_nsec: Some(111_222_333),
        resolution_nsec: Some(100),
    };

    let mut mst = DcMstring::new();
    ts.encode_dc_mixed(&mut mst)?;

    // Standalone decode via mst.decode::<FileTimestamps> under strict policy
    // succeeds because FileTimestamps::decode_dc_node calls ctx.mark_all_roots_consumed().
    let strict = DcProjectionPolicy::strict();
    let decoded_strict = mst.decode::<FileTimestamps>(&strict)?;
    ensure!(decoded_strict == ts, "Strict standalone decode mismatch");

    // Under forgiving policy, it succeeds as well:
    let forgiving = DcProjectionPolicy::forgiving();
    let decoded_forgiving = mst.decode::<FileTimestamps>(&forgiving)?;
    ensure!(decoded_forgiving == ts, "Forgiving standalone decode mismatch");

    Ok(())
}

// ============================================================================
// 2. FILE TIMESTAMPS: NANOSECOND PRECISION & OUT-OF-RANGE EPOCH SECONDS
// ============================================================================

#[crate::ctb_test]
fn test_challenge_file_timestamps_nanosecond_precision() -> Result<()> {
    // Nanosecond boundaries: 0, 1, 999_999_999.
    let test_cases = [
        (0i64, 0u32),
        (1i64, 1u32),
        (1_700_000_000i64, 999_999_999u32),
        (-1i64, 500_000_000u32),
        (-100i64, 999_999_999u32),
    ];

    for (sec, nsec) in test_cases {
        let ts = FileTimestamps {
            atime_sec: sec,
            atime_nsec: nsec,
            mtime_sec: sec,
            mtime_nsec: nsec,
            ctime_sec: sec,
            ctime_nsec: nsec,
            birthtime_sec: Some(sec),
            birthtime_nsec: Some(nsec),
            resolution_nsec: Some(1),
        };
        ts.validate()?;

        let meta = FileMetadata {
            timestamps: Some(ts),
            ..FileMetadata::empty()
        };
        assert_dc_roundtrip(&meta)?;
    }

    Ok(())
}

#[crate::ctb_test]
fn test_challenge_file_timestamps_invalid_nanoseconds_rejected() -> Result<()> {
    // Nanoseconds >= 1_000_000_000 must be rejected by validate().
    let invalid_ts = FileTimestamps {
        atime_sec: 10,
        atime_nsec: 1_000_000_000,
        mtime_sec: 10,
        mtime_nsec: 0,
        ctime_sec: 10,
        ctime_nsec: 0,
        birthtime_sec: None,
        birthtime_nsec: None,
        resolution_nsec: None,
    };
    ensure!(invalid_ts.validate().is_err(), "validate() must reject atime_nsec >= 1e9");

    let mut mst = DcMstring::new();
    ensure!(invalid_ts.encode_dc_mixed(&mut mst).is_err(), "encode must reject invalid nanoseconds");

    Ok(())
}

#[crate::ctb_test]
fn test_challenge_file_timestamps_out_of_range_epoch() -> Result<()> {
    // Out-of-range epoch values: i128 nanoseconds exceeding i64 range.
    // Construct a Dc 390..391 node with an enormous integer exceeding i64::MAX * 1e9.
    let mut mst = DcMstring::new();
    mst.push_char(DcChar::from_short(321));
    mst.push_char(DcChar::from_short(330));
    mst.push_char(DcChar::from_short(390));
    mst.push_char(DcChar::from_format(487));
    // Number larger than i64::MAX * 1_000_000_000:
    let huge_nanos: i128 = i128::from(i64::MAX).checked_mul(2_000_000_000).context("overflow")?;
    huge_nanos.encode_dc_mixed(&mut mst)?;
    mst.push_char(DcChar::from_short(391));
    mst.push_char(DcChar::from_short(322));

    let mut nav = DcDocumentNavigator::from_mstr(&mst);
    let mut arena = DcSyntaxArena::new();
    let strict = DcProjectionPolicy::strict();
    nav.parse_document_into_arena(&mut arena, &strict)?;

    let mut ctx = DcProjectionContext::new(&arena, &strict);
    let res = FileTimestamps::decode_dc_node(&mut ctx, arena.roots()[0]);
    ensure!(res.is_err(), "Out-of-range epoch seconds must be rejected");

    Ok(())
}

// ============================================================================
// 3. PLATFORM RAW FLAGS: MALFORMED BINARY PAYLOADS & WIDTH VALIDATION
// ============================================================================

#[crate::ctb_test]
fn test_challenge_platform_raw_flags_payload_widths() -> Result<()> {
    let strict = DcProjectionPolicy::strict();

    // Valid: 8 bytes
    let raw8 = PlatformRawFlags::from_raw(0x1234_5678_9ABC_DEF0);
    let mut mst8 = DcMstring::new();
    raw8.encode_dc_mixed(&mut mst8)?;
    let mut nav8 = DcDocumentNavigator::from_mstr(&mst8);
    let mut arena8 = DcSyntaxArena::new();
    nav8.parse_document_into_arena(&mut arena8, &strict)?;
    let mut ctx8 = DcProjectionContext::new(&arena8, &strict);
    let decoded8 = PlatformRawFlags::decode_dc_node(&mut ctx8, arena8.roots()[0])?;
    ensure!(decoded8.raw_value() == 0x1234_5678_9ABC_DEF0, "8-byte raw flags mismatch");

    // Valid: 4 bytes via binary payload
    let mut mst4 = DcMstring::new();
    mst4.push_binary_with_sha256(&[0x78u8, 0x56, 0x34, 0x12])?;
    let mut nav4 = DcDocumentNavigator::from_mstr(&mst4);
    let mut arena4 = DcSyntaxArena::new();
    nav4.parse_document_into_arena(&mut arena4, &strict)?;
    let mut ctx4 = DcProjectionContext::new(&arena4, &strict);
    let decoded4 = PlatformRawFlags::decode_dc_node(&mut ctx4, arena4.roots()[0])?;
    ensure!(decoded4.raw_value() == 0x1234_5678, "4-byte raw flags mismatch");

    // Invalid widths: 0, 1, 2, 3, 5, 6, 7, 9 bytes
    for bad_len in [0, 1, 2, 3, 5, 6, 7, 9, 16] {
        let mut mst_bad = DcMstring::new();
        let payload = vec![0u8; bad_len];
        mst_bad.push_binary_with_sha256(&payload)?;

        let mut nav_bad = DcDocumentNavigator::from_mstr(&mst_bad);
        let mut arena_bad = DcSyntaxArena::new();
        nav_bad.parse_document_into_arena(&mut arena_bad, &strict)?;
        let mut ctx_bad = DcProjectionContext::new(&arena_bad, &strict);
        let res = PlatformRawFlags::decode_dc_node(&mut ctx_bad, arena_bad.roots()[0]);
        ensure!(res.is_err(), "PlatformRawFlags must reject invalid payload width {bad_len}");
    }

    // Invalid node type: String node instead of binary
    let mut mst_str = DcMstring::new();
    "not_binary".encode_dc_mixed(&mut mst_str)?;
    let mut nav_str = DcDocumentNavigator::from_mstr(&mst_str);
    let mut arena_str = DcSyntaxArena::new();
    nav_str.parse_document_into_arena(&mut arena_str, &strict)?;
    let mut ctx_str = DcProjectionContext::new(&arena_str, &strict);
    let res = PlatformRawFlags::decode_dc_node(&mut ctx_str, arena_str.roots()[0]);
    ensure!(res.is_err(), "PlatformRawFlags must reject non-binary/list string node");

    Ok(())
}

// ============================================================================
// 4. NATIVE METADATA: PAIRS VS LEGACY MAPS & STRICT REJECTION
// ============================================================================

#[crate::ctb_test]
fn test_challenge_native_metadata_pairs_roundtrip() -> Result<()> {
    let mut values = BTreeMap::new();
    values.insert("inode".to_string(), NativeMetadataValue::Unsigned(12345));
    values.insert("offset".to_string(), NativeMetadataValue::Signed(-99));
    values.insert("acl_bytes".to_string(), NativeMetadataValue::Bytes(vec![0xDE, 0xAD, 0xBE, 0xEF]));

    let native = NativeMetadata {
        source_os: OsFamily::Linux,
        values,
    };

    let mut mst = DcMstring::new();
    native.encode_dc_mixed(&mut mst)?;

    // Strict roundtrip verification
    let mut nav = DcDocumentNavigator::from_mstr(&mst);
    let mut arena = DcSyntaxArena::new();
    let strict = DcProjectionPolicy::strict();
    nav.parse_document_into_arena(&mut arena, &strict)?;
    let mut ctx = DcProjectionContext::new(&arena, &strict);
    let decoded = NativeMetadata::decode_dc_node(&mut ctx, arena.roots()[0])?;

    ensure!(decoded == native, "NativeMetadata pairs roundtrip mismatch");
    Ok(())
}

#[crate::ctb_test]
fn test_challenge_native_metadata_legacy_map_and_diagnostics() -> Result<()> {
    // Construct a legacy Dc 313..314 map:
    let mut mst = DcMstring::new();
    mst.push_char(DcChar::from_short(313));
    "source_os".encode_dc_mixed(&mut mst)?;
    mst.push_char(DcChar::from_short(312));
    OsFamily::FreeBSD.encode_dc_mixed(&mut mst)?;

    "values".encode_dc_mixed(&mut mst)?;
    mst.push_char(DcChar::from_short(312));
    let empty_values: BTreeMap<String, NativeMetadataValue> = BTreeMap::new();
    empty_values.encode_dc_mixed(&mut mst)?;
    mst.push_char(DcChar::from_short(314));

    // Under standard strict policy (where deprecated_syntax is Warn), it succeeds with warning:
    let mut nav = DcDocumentNavigator::from_mstr(&mst);
    let mut arena = DcSyntaxArena::new();
    let strict = DcProjectionPolicy::strict();
    nav.parse_document_into_arena(&mut arena, &strict)?;
    let mut ctx_strict = DcProjectionContext::new(&arena, &strict);
    let decoded = NativeMetadata::decode_dc_node(&mut ctx_strict, arena.roots()[0])?;
    ensure!(decoded.source_os == OsFamily::FreeBSD, "Source OS mismatch in legacy map");
    ensure!(decoded.values.is_empty(), "Values should be empty");

    // Under policy with deprecated_syntax = DiagnosticAction::Error, it must reject:
    let deprecated_error_policy = DcProjectionPolicy {
        deprecated_syntax: DiagnosticAction::Error,
        ..DcProjectionPolicy::strict()
    };
    let mut nav_err = DcDocumentNavigator::from_mstr(&mst);
    let mut arena_err = DcSyntaxArena::new();
    let res_err = nav_err.parse_document_into_arena(&mut arena_err, &deprecated_error_policy);
    ensure!(res_err.is_err(), "Policy with deprecated_syntax=Error must reject Dc 313 map");

    // Malformed legacy map with uneven number of elements must be rejected:
    let mut mst_uneven = DcMstring::new();
    mst_uneven.push_char(DcChar::from_short(313));
    "source_os".encode_dc_mixed(&mut mst_uneven)?;
    mst_uneven.push_char(DcChar::from_short(312));
    OsFamily::FreeBSD.encode_dc_mixed(&mut mst_uneven)?;
    "trailing_key_without_val".encode_dc_mixed(&mut mst_uneven)?;
    mst_uneven.push_char(DcChar::from_short(314));

    let mut nav_uneven = DcDocumentNavigator::from_mstr(&mst_uneven);
    let mut arena_uneven = DcSyntaxArena::new();
    nav_uneven.parse_document_into_arena(&mut arena_uneven, &strict)?;
    let mut ctx_uneven = DcProjectionContext::new(&arena_uneven, &strict);
    ensure!(
        NativeMetadata::decode_dc_node(&mut ctx_uneven, arena_uneven.roots()[0]).is_err(),
        "Legacy map with uneven item count must be rejected"
    );

    Ok(())
}

#[crate::ctb_test]
fn test_challenge_native_metadata_missing_fields_policy() -> Result<()> {
    // Pairs map containing only "source_os", missing "values"
    let mut mst = DcMstring::new();
    let enc = DcKvMapEncoding::Pairs;
    enc.encode_header(&mut mst);
    enc.encode_entry_begin(&mut mst, true);
    "source_os".encode_dc_mixed(&mut mst)?;
    mst.push_char(DcChar::from_short(312));
    OsFamily::Windows.encode_dc_mixed(&mut mst)?;
    enc.encode_entry_end(&mut mst);
    enc.encode_end(&mut mst);

    let mut nav = DcDocumentNavigator::from_mstr(&mst);
    let mut arena = DcSyntaxArena::new();
    let strict = DcProjectionPolicy::strict();
    nav.parse_document_into_arena(&mut arena, &strict)?;

    // Strict must error on missing required field
    let mut ctx_strict = DcProjectionContext::new(&arena, &strict);
    ensure!(
        NativeMetadata::decode_dc_node(&mut ctx_strict, arena.roots()[0]).is_err(),
        "Strict policy must reject missing 'values' field"
    );

    // Forgiving should supply forgiving default (empty map)
    let forgiving = DcProjectionPolicy::forgiving();
    let mut ctx_forgiving = DcProjectionContext::new(&arena, &forgiving);
    let decoded = NativeMetadata::decode_dc_node(&mut ctx_forgiving, arena.roots()[0])?;
    ensure!(decoded.source_os == OsFamily::Windows);
    ensure!(decoded.values.is_empty());

    // Duplicate map key in pairs map must error:
    let mut mst_dup = DcMstring::new();
    enc.encode_header(&mut mst_dup);
    enc.encode_entry_begin(&mut mst_dup, true);
    "source_os".encode_dc_mixed(&mut mst_dup)?;
    mst_dup.push_char(DcChar::from_short(312));
    OsFamily::Linux.encode_dc_mixed(&mut mst_dup)?;
    enc.encode_entry_end(&mut mst_dup);

    enc.encode_entry_begin(&mut mst_dup, true);
    "source_os".encode_dc_mixed(&mut mst_dup)?;
    mst_dup.push_char(DcChar::from_short(312));
    OsFamily::Windows.encode_dc_mixed(&mut mst_dup)?;
    enc.encode_entry_end(&mut mst_dup);
    enc.encode_end(&mut mst_dup);

    let mut nav_dup = DcDocumentNavigator::from_mstr(&mst_dup);
    let mut arena_dup = DcSyntaxArena::new();
    nav_dup.parse_document_into_arena(&mut arena_dup, &strict)?;
    let mut ctx_dup = DcProjectionContext::new(&arena_dup, &strict);
    ensure!(
        NativeMetadata::decode_dc_node(&mut ctx_dup, arena_dup.roots()[0]).is_err(),
        "Duplicate map keys must be rejected"
    );

    Ok(())
}

// ============================================================================
// 5. APPLE METADATA: CONTAINER FRAMING, UNKNOWN FIELDS, FULL ROUNDTRIP
// ============================================================================

#[crate::ctb_test]
fn test_challenge_apple_metadata_container_and_fields() -> Result<()> {
    let mut fi = FinderInfo::default();
    fi.file_type = "TEXT".to_string();
    fi.file_creator = "ttxt".to_string();

    let apple = AppleMetadata {
        finder_info: Some(fi),
        real_name: Some("test_document.txt".to_string()),
        comment: Some("Test comment".to_string()),
        backup_timestamp_sec: Some(1_700_000_500),
        unrecognized_entries: Vec::new(),
    };

    let mut mst = DcMstring::new();
    apple.encode_dc_mixed(&mut mst)?;

    // Strict roundtrip
    let mut nav = DcDocumentNavigator::from_mstr(&mst);
    let mut arena = DcSyntaxArena::new();
    let strict = DcProjectionPolicy::strict();
    nav.parse_document_into_arena(&mut arena, &strict)?;
    let mut ctx = DcProjectionContext::new(&arena, &strict);
    let decoded = AppleMetadata::decode_dc_node(&mut ctx, arena.roots()[0])?;

    ensure!(decoded.finder_info == apple.finder_info, "FinderInfo mismatch");
    ensure!(decoded.real_name == apple.real_name, "Real name mismatch");
    ensure!(decoded.comment == apple.comment, "Comment mismatch");
    ensure!(decoded.backup_timestamp_sec == apple.backup_timestamp_sec, "Backup timestamp mismatch");

    // Test rejection of invalid container opener (e.g. 310..311 list instead of 401..402)
    let mut mst_bad = DcMstring::new();
    mst_bad.push_char(DcChar::from_short(310));
    mst_bad.push_char(DcChar::from_short(311));
    let mut nav_bad = DcDocumentNavigator::from_mstr(&mst_bad);
    let mut arena_bad = DcSyntaxArena::new();
    nav_bad.parse_document_into_arena(&mut arena_bad, &strict)?;
    let mut ctx_bad = DcProjectionContext::new(&arena_bad, &strict);
    ensure!(
        AppleMetadata::decode_dc_node(&mut ctx_bad, arena_bad.roots()[0]).is_err(),
        "AppleMetadata must reject non-401 container"
    );

    // Test rejection of unknown field tag inside AppleMetadata in strict mode
    let mut mst_unknown = DcMstring::new();
    mst_unknown.push_char(DcChar::from_short(401));
    mst_unknown.push_char(DcChar::from_short(999)); // unknown tag 999
    42u32.encode_dc_mixed(&mut mst_unknown)?;
    mst_unknown.push_char(DcChar::from_short(402));

    let mut nav_unknown = DcDocumentNavigator::from_mstr(&mst_unknown);
    let mut arena_unknown = DcSyntaxArena::new();
    nav_unknown.parse_document_into_arena(&mut arena_unknown, &strict)?;
    let mut ctx_unknown = DcProjectionContext::new(&arena_unknown, &strict);
    ensure!(
        AppleMetadata::decode_dc_node(&mut ctx_unknown, arena_unknown.roots()[0]).is_err(),
        "AppleMetadata must reject unknown tag under strict policy"
    );

    // Under forgiving policy, unknown tag should be ignored
    let forgiving = DcProjectionPolicy::forgiving();
    let mut ctx_forgiving = DcProjectionContext::new(&arena_unknown, &forgiving);
    let decoded_forgiving = AppleMetadata::decode_dc_node(&mut ctx_forgiving, arena_unknown.roots()[0])?;
    ensure!(decoded_forgiving.is_empty(), "AppleMetadata with unknown tags must decode as default/empty under forgiving policy");

    Ok(())
}
