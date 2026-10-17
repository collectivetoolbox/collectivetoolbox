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
use ctb_formats_apple_single_double::{FinderFlags, FinderInfo, FinderLabel};
use ctb_formats_dcstring::dc::{DC_BEGIN_LIST, DC_END_LIST, DC_TYPE_ANY};
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

    // Document projection explicitly receives the complete root sequence.
    let mut ctx = DcProjectionContext::new(&arena, &strict);
    let (decoded, consumed) = FileTimestamps::decode_dc_document(&mut ctx, roots)?;
    ensure!(consumed == roots.len(), "FileTimestamps must consume every timestamp root");
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

    // Standalone decode uses FileTimestamps::decode_dc_document to consume the roots.
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
    let encoding = DcKvMapEncoding::Legacy;
    encoding.encode_header(&mut mst);
    encoding.encode_entry_begin(&mut mst, true);
    "source_os".encode_dc_mixed(&mut mst)?;
    mst.push_char(DcChar::from_short(312));
    OsFamily::FreeBSD.encode_dc_mixed(&mut mst)?;
    encoding.encode_entry_end(&mut mst);

    encoding.encode_entry_begin(&mut mst, false);
    "values".encode_dc_mixed(&mut mst)?;
    mst.push_char(DcChar::from_short(312));
    let empty_values: BTreeMap<String, NativeMetadataValue> = BTreeMap::new();
    empty_values.encode_dc_mixed(&mut mst)?;
    encoding.encode_entry_end(&mut mst);
    encoding.encode_end(&mut mst);

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
    encoding.encode_header(&mut mst_uneven);
    "source_os".encode_dc_mixed(&mut mst_uneven)?;
    mst_uneven.push_char(DcChar::from_short(312));
    OsFamily::FreeBSD.encode_dc_mixed(&mut mst_uneven)?;
    mst_uneven.push_char(DcChar::from_short(312));
    "trailing_key_without_val".encode_dc_mixed(&mut mst_uneven)?;
    encoding.encode_end(&mut mst_uneven);

    let mut nav_uneven = DcDocumentNavigator::from_mstr(&mst_uneven);
    let mut arena_uneven = DcSyntaxArena::new();
    ensure!(
        nav_uneven.parse_document_into_arena(&mut arena_uneven, &strict).is_err(),
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
    ensure!(decoded == apple, "Nonempty AppleMetadata roundtrip mismatch");
    assert_dc_roundtrip(&apple)?;

    let empty = AppleMetadata::default().to_dc_mixed()?;
    let mut nav_empty = DcDocumentNavigator::from_mstr(&empty);
    let mut arena_empty = DcSyntaxArena::new();
    ensure!(
        nav_empty.parse_document_into_arena(&mut arena_empty, &strict).is_err(),
        "Empty AppleMetadata container must fail the nonempty record grammar"
    );

    // Test rejection of invalid container opener (e.g. 310..311 list instead of 401..402)
    let mut mst_bad = DcMstring::new();
    mst_bad.push_char(DC_BEGIN_LIST);
    mst_bad.push_char(DC_TYPE_ANY);
    mst_bad.push_char(DC_END_LIST);
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

#[crate::ctb_test]
fn test_challenge_equivalent_list_strict_312_separation() -> Result<()> {
    // A valid FinderInfo with FinderLabel::InProgress produces an equivalent
    // representations list [405, 397, 310, 275, 487, 312, ...] where 312 separates
    // the enum variant (Dc 487) from the numeric equivalent representation.
    let finfo = FinderInfo {
        label: FinderLabel::InProgress,
        ..FinderInfo::default()
    };
    let encoded = finfo.to_dc_mixed()?;
    let tokens = encoded.to_dc_tokens()?;

    // Verify valid equivalent list roundtrips strictly:
    let decoded: FinderInfo = encoded.decode(&DcProjectionPolicy::strict())?;
    ensure!(decoded == finfo, "Valid equivalent list must decode to identical FinderInfo");

    // Construct malformed equivalent lists where Dc 312 separator is omitted:
    // Pattern: 401 (record), 405 (label tag), 397 (equiv tag), 310 (list open),
    // 275 (type header), 487 (InProgress variant), 487 (duplicate variant without 312),
    // 311 (list close), 402 (record close).
    let malformed_no_sep = vec![401, 405, 397, 310, 275, 487, 487, 311, 402];
    let mut mst_malformed = DcMstring::new();
    for &tok in &malformed_no_sep {
        mst_malformed.push_char(DcChar::from_short(tok));
    }

    let strict_err = mst_malformed.decode::<FinderInfo>(&DcProjectionPolicy::strict());
    ensure!(
        strict_err.is_err(),
        "Equivalent list with missing Dc 312 must fail strict decoding"
    );
    let err_str = format!("{strict_err:?}");
    ensure!(
        err_str.contains("missing element separator (Dc 312)"),
        "Expected error message mentioning missing Dc 312, got: {err_str}"
    );

    // Verify that forgiving policy recovers the primary item even without 312:
    let forgiving_decoded = mst_malformed.decode::<FinderInfo>(&DcProjectionPolicy::forgiving())?;
    ensure!(
        forgiving_decoded.label == FinderLabel::InProgress,
        "Forgiving policy must recover primary equivalent element"
    );

    // Test a valid list with explicit Dc 312 separator:
    // [401, 405, 397, 310, 275, 487, 312, 487, 311, 402]
    let valid_with_sep = vec![401, 405, 397, 310, 275, 487, 312, 487, 311, 402];
    let mut mst_valid = DcMstring::new();
    for &tok in &valid_with_sep {
        mst_valid.push_char(DcChar::from_short(tok));
    }
    let valid_decoded = mst_valid.decode::<FinderInfo>(&DcProjectionPolicy::strict())?;
    ensure!(
        valid_decoded.label == FinderLabel::InProgress,
        "Valid equivalent list with Dc 312 must pass strict decoding"
    );

    Ok(())
}

/// Empirically challenges equivalent lists with multi-element configurations
/// and verifies that partial omission of Dc 312 is strictly rejected.

#[crate::ctb_test]
fn test_challenge_equivalent_list_multi_element_separation() -> Result<()> {
    // Three elements: el1, 312, el2, el3 (missing separator before el3)
    let missing_second_sep = vec![401, 405, 397, 310, 275, 487, 312, 486, 485, 311, 402];
    let mut mst = DcMstring::new();
    for &tok in &missing_second_sep {
        mst.push_char(DcChar::from_short(tok));
    }
    let res = mst.decode::<FinderInfo>(&DcProjectionPolicy::strict());
    ensure!(
        res.is_err(),
        "Missing second separator in 3-element list must fail strict decoding"
    );

    // Three elements: el1, 312, el2, 312, el3 (all separated)
    let all_separated = vec![401, 405, 397, 310, 275, 487, 312, 486, 312, 485, 311, 402];
    let mut mst_all = DcMstring::new();
    for &tok in &all_separated {
        mst_all.push_char(DcChar::from_short(tok));
    }
    let res_all = mst_all.decode::<FinderInfo>(&DcProjectionPolicy::strict())?;
    ensure!(
        res_all.label == FinderLabel::InProgress,
        "All-separated 3-element list must decode to primary element"
    );

    // Empty equivalent list: 397 [ 275 ] (no elements)
    let empty_list = vec![401, 405, 397, 310, 275, 311, 402];
    let mut mst_empty = DcMstring::new();
    for &tok in &empty_list {
        mst_empty.push_char(DcChar::from_short(tok));
    }
    // Strict policy must fail decoding:
    ensure!(
        mst_empty.decode::<FinderInfo>(&DcProjectionPolicy::strict()).is_err(),
        "Empty equivalent list must fail strict decoding"
    );
    // Forgiving policy handles the malformed field and falls back to default label:
    let forgiving_empty = mst_empty.decode::<FinderInfo>(&DcProjectionPolicy::forgiving())?;
    ensure!(
        forgiving_empty.label == FinderLabel::None,
        "Forgiving policy falls back to default on empty equivalent list"
    );

    // Direct unwrap_equivalent_or_self test:
    let mut bare_empty_equiv = DcMstring::new();
    bare_empty_equiv.push_char(DcChar::from_short(397));
    bare_empty_equiv.push_char(DcChar::from_short(310));
    bare_empty_equiv.push_char(DcChar::from_short(275));
    bare_empty_equiv.push_char(DcChar::from_short(311));

    let mut nav = DcDocumentNavigator::from_mstr(&bare_empty_equiv);
    let mut arena = DcSyntaxArena::new();
    let strict = DcProjectionPolicy::strict();
    nav.parse_document_into_arena(&mut arena, &strict)?;
    let ctx = DcProjectionContext::new(&arena, &strict);
    let root = arena.roots()[0];
    let unwrap_res = ctx.unwrap_equivalent_or_self(root);

    let vec_res = bare_empty_equiv.decode::<Vec<u32>>(&strict);

    Ok(())
}

/// Empirically challenges partially equivalent representations (Dc 561)
/// and verifies strict Dc 312 enforcement.

#[crate::ctb_test]
fn test_challenge_partially_equivalent_561_separation() -> Result<()> {
    // Dc 561 wrapper without 312
    let no_sep_561 = vec![401, 405, 561, 310, 275, 487, 487, 311, 402];
    let mut mst_no_sep = DcMstring::new();
    for &tok in &no_sep_561 {
        mst_no_sep.push_char(DcChar::from_short(tok));
    }
    let res = mst_no_sep.decode::<FinderInfo>(&DcProjectionPolicy::strict());
    ensure!(
        res.is_err(),
        "Dc 561 list missing Dc 312 must fail strict decoding"
    );

    // Dc 561 wrapper with 312
    let with_sep_561 = vec![401, 405, 561, 310, 275, 487, 312, 487, 311, 402];
    let mut mst_with_sep = DcMstring::new();
    for &tok in &with_sep_561 {
        mst_with_sep.push_char(DcChar::from_short(tok));
    }
    let res_ok = mst_with_sep.decode::<FinderInfo>(&DcProjectionPolicy::strict())?;
    ensure!(
        res_ok.label == FinderLabel::InProgress,
        "Dc 561 list with Dc 312 must pass strict decoding"
    );

    Ok(())
}

// ============================================================================
// 2. STANDALONE FILETIMESTAMPS ROUNDTRIP EMPIRICAL CHALLENGES
// ============================================================================

/// Tests standalone FileTimestamps across varied epoch boundary and nanosecond
/// configurations using direct `assert_dc_roundtrip(&ts)`.

#[crate::ctb_test]
fn test_challenge_standalone_file_timestamps_varied_configurations() -> Result<()> {
    // Varied timestamp test cases:
    // (atime_sec, atime_nsec, mtime_sec, mtime_nsec, ctime_sec, ctime_nsec,
    //  birthtime_sec, birthtime_nsec, resolution_nsec)
    let test_matrix = [
        // 1. Unix Epoch exact zero
        (0i64, 0u32, 0i64, 0u32, 0i64, 0u32, None, None, None),
        // 2. Epoch with resolution
        (0i64, 0u32, 0i64, 0u32, 0i64, 0u32, None, None, Some(100u32)),
        // 3. Epoch with 1ns resolution
        (0i64, 0u32, 0i64, 0u32, 0i64, 0u32, None, None, Some(1u32)),
        // 4. Positive seconds, zero nanoseconds
        (1_700_000_000i64, 0u32, 1_700_000_100i64, 0u32, 1_700_000_200i64, 0u32, None, None, None),
        // 5. Positive seconds with subsecond nanoseconds (arbitrary & max)
        (
            1_700_000_000i64, 123_456_789u32,
            1_700_000_001i64, 999_999_999u32,
            1_700_000_002i64, 1u32,
            Some(1_699_999_999i64), Some(500_000_000u32),
            Some(100u32),
        ),
        // 6. Pre-epoch negative 1 second with 0 nanos
        (-1i64, 0u32, -1i64, 0u32, -1i64, 0u32, None, None, None),
        // 7. Pre-epoch negative 1 second with fractional nanos (500_000_000)
        (-1i64, 500_000_000u32, -1i64, 500_000_000u32, -1i64, 500_000_000u32, None, None, None),
        // 8. Pre-epoch negative 1 second with max nanos (999_999_999)
        (-1i64, 999_999_999u32, -1i64, 999_999_999u32, -1i64, 999_999_999u32, None, None, None),
        // 9. Pre-epoch negative 1 second with 1 nano
        (-1i64, 1u32, -1i64, 1u32, -1i64, 1u32, None, None, None),
        // 10. Large negative seconds (1 day before epoch = -86400)
        (-86_400i64, 0u32, -86_400i64, 123_456u32, -86_400i64, 789_012u32, None, None, Some(1000u32)),
        // 11. 1 year before epoch (-31_536_000)
        (-31_536_000i64, 42u32, -31_536_000i64, 42u32, -31_536_000i64, 42u32, None, None, None),
        // 12. Windows epoch 1601-01-01 (-11_644_473_600 seconds)
        (
            -11_644_473_600i64, 100u32,
            -11_644_473_600i64, 200u32,
            -11_644_473_600i64, 300u32,
            Some(-11_644_473_600i64), Some(100u32),
            Some(100u32),
        ),
        // 13. Asymmetric positive and negative timestamps across fields
        (
            -500i64, 123_456_789u32,
            1_000i64, 987_654_321u32,
            0i64, 0u32,
            Some(-10_000i64), Some(111_222_333u32),
            Some(50u32),
        ),
        // 14. Large future seconds (year ~2286: 10_000_000_000)
        (
            10_000_000_000i64, 999_999_999u32,
            10_000_000_001i64, 0u32,
            10_000_000_002i64, 500_000_000u32,
            Some(10_000_000_000i64), Some(1u32),
            Some(1u32),
        ),
    ];

    for (atime_sec, atime_nsec, mtime_sec, mtime_nsec, ctime_sec, ctime_nsec,
         birthtime_sec, birthtime_nsec, resolution_nsec) in test_matrix {
        let ts = FileTimestamps {
            atime_sec,
            atime_nsec,
            mtime_sec,
            mtime_nsec,
            ctime_sec,
            ctime_nsec,
            birthtime_sec,
            birthtime_nsec,
            resolution_nsec,
        };

        // Ensure validate() accepts this configuration:
        ts.validate()?;

        // Perform strict standalone assert_dc_roundtrip:
        assert_dc_roundtrip(&ts)
            .with_context(|| format!("Standalone roundtrip failed for ts: {ts:?}"))?;

        // Also test direct decode via mst.decode::<FileTimestamps>(&strict):
        let mut mst = DcMstring::new();
        ts.encode_dc_mixed(&mut mst)?;
        let strict = DcProjectionPolicy::strict();
        let decoded = mst.decode::<FileTimestamps>(&strict)?;
        ensure!(
            decoded == ts,
            "Direct mst.decode::<FileTimestamps> mismatch: {decoded:?} vs {ts:?}"
        );
    }

    Ok(())
}

/// Tests that standalone FileTimestamps reject invalid timestamps and malformed input.

#[crate::ctb_test]
fn test_challenge_standalone_file_timestamps_rejections() -> Result<()> {
    // Nanoseconds >= 1_000_000_000 must fail validation and encoding:
    let bad_nanos = FileTimestamps {
        atime_sec: 0,
        atime_nsec: 1_000_000_000,
        mtime_sec: 0,
        mtime_nsec: 0,
        ctime_sec: 0,
        ctime_nsec: 0,
        birthtime_sec: None,
        birthtime_nsec: None,
        resolution_nsec: None,
    };
    ensure!(bad_nanos.validate().is_err(), "atime_nsec >= 1e9 must fail validate()");
    let mut mst = DcMstring::new();
    ensure!(bad_nanos.encode_dc_mixed(&mut mst).is_err(), "atime_nsec >= 1e9 must fail encode()");

    // Birthtime seconds present but nanoseconds missing:
    let bad_birthtime = FileTimestamps {
        atime_sec: 0,
        atime_nsec: 0,
        mtime_sec: 0,
        mtime_nsec: 0,
        ctime_sec: 0,
        ctime_nsec: 0,
        birthtime_sec: Some(10),
        birthtime_nsec: None,
        resolution_nsec: None,
    };
    ensure!(bad_birthtime.validate().is_err(), "birthtime without nanos must fail validate()");

    // Resolution = 0:
    let bad_res = FileTimestamps {
        atime_sec: 0,
        atime_nsec: 0,
        mtime_sec: 0,
        mtime_nsec: 0,
        ctime_sec: 0,
        ctime_nsec: 0,
        birthtime_sec: None,
        birthtime_nsec: None,
        resolution_nsec: Some(0),
    };
    ensure!(bad_res.validate().is_err(), "resolution = 0 must fail validate()");

    Ok(())
}

#[crate::ctb_test]
fn test_platform_raw_flags_valid_payloads_and_endianness() -> Result<()> {
    let strict = DcProjectionPolicy::strict();
    let forgiving = DcProjectionPolicy::forgiving();

    // Case 1A: 8-byte valid binary payload (little-endian representation)
    let val64 = 0x1122_3344_5566_7788u64;
    let flags64 = PlatformRawFlags::from_raw(val64);
    let mut mst64 = DcMstring::new();
    flags64.encode_dc_mixed(&mut mst64)?;

    let dec_strict64: PlatformRawFlags = mst64.decode(&strict)?;
    ensure!(
        dec_strict64.raw_value == val64,
        "8-byte PlatformRawFlags raw_value mismatch under strict policy"
    );
    let dec_forgiving64: PlatformRawFlags = mst64.decode(&forgiving)?;
    ensure!(
        dec_forgiving64.raw_value == val64,
        "8-byte PlatformRawFlags raw_value mismatch under forgiving policy"
    );
    ensure!(
        dec_strict64.as_ref() == &val64.to_le_bytes(),
        "8-byte PlatformRawFlags as_ref bytes must match little-endian bytes"
    );

    // Case 1B: Boundary values for 8-byte
    for &boundary in &[0u64, 1u64, u64::MAX, 0x8000_0000_0000_0000u64, 0xAAAA_5555_AAAA_5555u64] {
        let f = PlatformRawFlags::from_raw(boundary);
        let mut m = DcMstring::new();
        f.encode_dc_mixed(&mut m)?;
        let d: PlatformRawFlags = m.decode(&strict)?;
        ensure!(d.raw_value == boundary, "Boundary {boundary:#x} roundtrip failed");
    }

    // Case 1C: 4-byte valid binary payload (legacy BSD 32-bit chflags)
    let val32 = 0xAABB_CCDDu32;
    let mut mst32 = DcMstring::new();
    mst32.push_binary_with_sha256(&val32.to_le_bytes())?;

    let dec_strict32: PlatformRawFlags = mst32.decode(&strict)?;
    ensure!(
        dec_strict32.raw_value == u64::from(val32),
        "4-byte PlatformRawFlags did not zero-extend u32 to u64 accurately"
    );
    let dec_forgiving32: PlatformRawFlags = mst32.decode(&forgiving)?;
    ensure!(
        dec_forgiving32.raw_value == u64::from(val32),
        "4-byte PlatformRawFlags forgiving decode mismatch"
    );

    // Case 1D: [u8] list (Dc 310 .. Dc 311) must be rejected because PlatformRawFlags requires Binary node
    let mut mst_list4 = DcMstring::new();
    mst_list4.push_char(DC_BEGIN_LIST);
    for b in val32.to_le_bytes() {
        b.encode_dc_mixed(&mut mst_list4)?;
    }
    mst_list4.push_char(DC_END_LIST);
    ensure!(
        mst_list4.decode::<PlatformRawFlags>(&strict).is_err(),
        "4-byte [u8] list PlatformRawFlags decode must fail under strict policy"
    );
    ensure!(
        mst_list4.decode::<PlatformRawFlags>(&forgiving).is_err(),
        "4-byte [u8] list PlatformRawFlags decode must fail under forgiving policy"
    );

    let mut mst_list8 = DcMstring::new();
    mst_list8.push_char(DC_BEGIN_LIST);
    for b in val64.to_le_bytes() {
        b.encode_dc_mixed(&mut mst_list8)?;
    }
    mst_list8.push_char(DC_END_LIST);
    ensure!(
        mst_list8.decode::<PlatformRawFlags>(&strict).is_err(),
        "8-byte [u8] list PlatformRawFlags decode must fail under strict policy"
    );
    ensure!(
        mst_list8.decode::<PlatformRawFlags>(&forgiving).is_err(),
        "8-byte [u8] list PlatformRawFlags decode must fail under forgiving policy"
    );

    Ok(())
}

#[crate::ctb_test]
fn test_platform_raw_flags_invalid_lengths_rejection() -> Result<()> {
    let strict = DcProjectionPolicy::strict();
    let forgiving = DcProjectionPolicy::forgiving();

    // Invalid lengths: 0, 1, 2, 3, 5, 6, 7, 9, 10, 12, 16, 32, 64
    let invalid_lengths = [0usize, 1, 2, 3, 5, 6, 7, 9, 10, 12, 16, 32, 64];

    for &len in &invalid_lengths {
        // Test via Binary node
        let mut mst_bin = DcMstring::new();
        mst_bin.push_binary_with_sha256(&vec![0x5Au8; len])?;

        let res_strict = mst_bin.decode::<PlatformRawFlags>(&strict);
        ensure!(
            res_strict.is_err(),
            "PlatformRawFlags under strict policy MUST reject {len}-byte binary payload, but succeeded"
        );
        let res_forgiving = mst_bin.decode::<PlatformRawFlags>(&forgiving);
        ensure!(
            res_forgiving.is_err(),
            "PlatformRawFlags under forgiving policy MUST reject {len}-byte binary payload, but succeeded"
        );

        // Test via [u8] list
        let mut mst_list = DcMstring::new();
        mst_list.push_char(DC_BEGIN_LIST);
        for _ in 0..len {
            0x5Au8.encode_dc_mixed(&mut mst_list)?;
        }
        mst_list.push_char(DC_END_LIST);

        let res_list_strict = mst_list.decode::<PlatformRawFlags>(&strict);
        ensure!(
            res_list_strict.is_err(),
            "PlatformRawFlags under strict policy MUST reject {len}-element list, but succeeded"
        );
        let res_list_forgiving = mst_list.decode::<PlatformRawFlags>(&forgiving);
        ensure!(
            res_list_forgiving.is_err(),
            "PlatformRawFlags under forgiving policy MUST reject {len}-element list, but succeeded"
        );
    }

    // Test rejection of non-binary, non-list nodes
    let mut mst_str = DcMstring::new();
    "flags_string".encode_dc_mixed(&mut mst_str)?;
    ensure!(mst_str.decode::<PlatformRawFlags>(&strict).is_err());
    ensure!(mst_str.decode::<PlatformRawFlags>(&forgiving).is_err());

    let mut mst_int = DcMstring::new();
    12345u64.encode_dc_mixed(&mut mst_int)?;
    ensure!(mst_int.decode::<PlatformRawFlags>(&strict).is_err());
    ensure!(mst_int.decode::<PlatformRawFlags>(&forgiving).is_err());

    let mut mst_bool = DcMstring::new();
    true.encode_dc_mixed(&mut mst_bool)?;
    ensure!(mst_bool.decode::<PlatformRawFlags>(&strict).is_err());
    ensure!(mst_bool.decode::<PlatformRawFlags>(&forgiving).is_err());

    Ok(())
}

// ============================================================================
// 2. ATTACHED STREAM: SERIALIZATION & ROUNDTRIP WITHOUT AD-HOC TOKEN PATCHING
// ============================================================================

#[crate::ctb_test]
fn test_platform_raw_flags_tagged_and_malformed_nodes() -> Result<()> {
    let strict = DcProjectionPolicy::strict();

    // 1. Dc 539 tag wrapping 8-byte payload
    let val8 = 0x0102_0304_0506_0708u64;
    let mut mst_tag8 = DcMstring::new();
    mst_tag8.push_char(DcChar::from_short(539));
    mst_tag8.push_binary_with_sha256(&val8.to_le_bytes())?;
    let dec_tag8: PlatformRawFlags = mst_tag8.decode(&strict)?;
    ensure!(dec_tag8.raw_value == val8, "Tagged Dc 539 8-byte payload mismatch");

    // 2. Dc 539 tag wrapping 4-byte payload
    let val4 = 0xAABB_CCDDu32;
    let mut mst_tag4 = DcMstring::new();
    mst_tag4.push_char(DcChar::from_short(539));
    mst_tag4.push_binary_with_sha256(&val4.to_le_bytes())?;
    let dec_tag4: PlatformRawFlags = mst_tag4.decode(&strict)?;
    ensure!(dec_tag4.raw_value == u64::from(val4), "Tagged Dc 539 4-byte payload mismatch");

    // 3. Dc 539 tag wrapping invalid 3-byte payload
    let mut mst_tag_bad = DcMstring::new();
    mst_tag_bad.push_char(DcChar::from_short(539));
    mst_tag_bad.push_binary_with_sha256(&[1u8, 2, 3])?;
    ensure!(
        mst_tag_bad.decode::<PlatformRawFlags>(&strict).is_err(),
        "Tagged Dc 539 with 3-byte payload must fail"
    );

    // 4. Standalone bare Dc 539 tag without argument
    let mut mst_bare_tag = DcMstring::new();
    mst_bare_tag.push_char(DcChar::from_short(539));
    ensure!(
        mst_bare_tag.decode::<PlatformRawFlags>(&strict).is_err(),
        "Bare Dc 539 tag without argument must error safely"
    );

    // 5. Container Dc 310 with non-u8 string elements
    let mut mst_bad_list = DcMstring::new();
    mst_bad_list.push_char(DC_BEGIN_LIST);
    "not_a_u8".encode_dc_mixed(&mut mst_bad_list)?;
    mst_bad_list.push_char(DC_END_LIST);
    ensure!(
        mst_bad_list.decode::<PlatformRawFlags>(&strict).is_err(),
        "Container with string element must reject PlatformRawFlags decode"
    );

    Ok(())
}
