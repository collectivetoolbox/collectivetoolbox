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

//! Empirical Challenge Test Suite for Milestone 3 (LinkPath, StreamName, AttachedStream):
//! 1. Stress-test `LinkPath`: strict rejection of bare strings/binaries vs forgiving recovery,
//!    conflicting 561 equivalent representations, and path normalization across platforms.
//! 2. Stress-test `StreamName`: WTF-8 lone surrogates, invalid UTF-8 bytes, long stream names
//!    (up to 128 KiB), and format tag handling.
//! 3. Stress-test `AttachedStream`: missing stream data tags (Dc 385), missing StreamKind,
//!    missing FileEntity, corrupt container framing, and nameless stream handling.


#[expect(unused_imports, clippy::wildcard_imports, reason = "Integration test root")]
use ctb_utilities::*;

use ctb_formats_dcstring::dc::*;
use ctb_formats_dcstring::{
    DcChar, DcMixedDecode, DcMixedEncode, DcMstring, DcProjectionPolicy,
};
use ctb_io_file::metadata::{
    AppleMetadata, FileTimestamps, NativeMetadata, PlatformRawFlags,
};
use ctb_io_file::{
    AttachedStream, FileEntity, FileEntityKind, LinkPath, StreamKind, StreamName,
};
use ctb_utilities::format_id::FormatId;
use std::path::{Path, PathBuf};

// ============================================================================
// 1. LINKPATH EMPIRICAL STRESS TESTS
// ============================================================================

#[crate::ctb_test]
fn test_challenge_linkpath_strict_rejection_vs_forgiving_recovery() -> Result<()> {
    let strict = DcProjectionPolicy::strict();
    let forgiving = DcProjectionPolicy::forgiving();

    // A. Bare string literal: e.g. "relative/symlink/target"
    let mut bare_string = DcMstring::new();
    "relative/symlink/target".encode_dc_mixed(&mut bare_string)?;

    // 1. Strict policy MUST reject bare string
    let strict_res = bare_string.decode::<LinkPath>(&strict);
    ensure!(
        strict_res.is_err(),
        "LinkPath under strict policy must reject bare string, but succeeded"
    );

    // 2. Standalone LinkPath under forgiving policy also rejects bare string
    let forgiving_res = bare_string.decode::<LinkPath>(&forgiving);
    ensure!(
        forgiving_res.is_err(),
        "Standalone LinkPath under forgiving policy must reject bare string"
    );

    // B. Bare binary payload: e.g. raw bytes b"/usr/local/bin"
    let mut bare_binary = DcMstring::new();
    bare_binary.push_binary_with_sha256(b"/usr/local/bin")?;

    // 1. Strict policy MUST reject bare binary
    let strict_bin_res = bare_binary.decode::<LinkPath>(&strict);
    ensure!(
        strict_bin_res.is_err(),
        "LinkPath under strict policy must reject bare binary payload, but succeeded"
    );

    // 2. Standalone LinkPath under forgiving policy also rejects bare binary
    let forgiving_bin_res = bare_binary.decode::<LinkPath>(&forgiving);
    ensure!(
        forgiving_bin_res.is_err(),
        "Standalone LinkPath under forgiving policy must reject bare binary"
    );

    // C. Valid typed LinkPath (Dc 561 typed path container)
    let valid_path = LinkPath::from_bytes(b"target/dir/subfile", false);
    let mut valid_mst = DcMstring::new();
    valid_path.encode_dc_mixed(&mut valid_mst)?;

    // Both strict and forgiving MUST successfully decode typed path
    let decoded_strict = valid_mst.decode::<LinkPath>(&strict)?;
    ensure!(decoded_strict == valid_path, "Strict failed to decode valid typed LinkPath");

    let decoded_forgiving = valid_mst.decode::<LinkPath>(&forgiving)?;
    ensure!(decoded_forgiving == valid_path, "Forgiving failed to decode valid typed LinkPath");

    Ok(())
}

#[crate::ctb_test]
fn test_challenge_linkpath_conflicting_and_malformed_representations() -> Result<()> {
    let strict = DcProjectionPolicy::strict();

    // 1. Conflicting representations: segments disagree with binary payload
    // Dc 561 [ 275 segments ; binary ]
    let mut conflict_mst = DcMstring::new();
    conflict_mst.push_char(DC_PARTIALLY_EQUIVALENT_REPRESENTATIONS); // 561
    conflict_mst.push_char(DC_BEGIN_LIST); // 310
    conflict_mst.push_char(DcChar::from_short(275)); // 275
    // Segments: ["foo", "bar"]
    ctb_formats_dcstring::dc_mixed_serde::encode_filepath_segments(
        &[b"foo".to_vec(), b"bar".to_vec()],
        &mut conflict_mst,
    )?;
    conflict_mst.push_char(DC_LIST_ELEMENT_SEPARATOR); // 312
    // Binary: b"different/path" (segments: ["different", "path"])
    conflict_mst.push_binary_with_sha256(b"different/path")?;
    conflict_mst.push_char(DC_END_LIST); // 311

    let res = conflict_mst.decode::<LinkPath>(&strict);
    ensure!(
        res.is_err(),
        "Conflicting segments vs binary path MUST fail, but succeeded"
    );

    // 2. Empty 561 container: Dc 561 [ ]
    let mut empty_mst = DcMstring::new();
    empty_mst.push_char(DC_PARTIALLY_EQUIVALENT_REPRESENTATIONS);
    empty_mst.push_char(DC_BEGIN_LIST);
    empty_mst.push_char(DC_END_LIST);

    let empty_res = empty_mst.decode::<LinkPath>(&strict);
    ensure!(
        empty_res.is_err(),
        "Empty 561 container MUST fail, but succeeded"
    );

    Ok(())
}

#[crate::ctb_test]
fn test_challenge_linkpath_normalization_roundtrip() -> Result<()> {
    let cases: &[(&[u8], bool)] = &[
        (b"", false),
        (b"/", false),
        (b"///", false),
        (b"relative/path", false),
        (b"./relative//sub/../path", false),
        (b"/var/log/\xff\xfe/data", false),
        (b"C:\\Windows\\System32", true),
        (b"C:relative\\file", true),
        (b"\\\\server\\share\\path\\to\\file", true),
        (b"\\\\?\\UNC\\server\\share\\file", true),
    ];

    let strict = DcProjectionPolicy::strict();

    for &(bytes, is_windows) in cases {
        let lp = LinkPath::from_bytes(bytes, is_windows);
        ensure!(lp.to_bytes() == bytes);

        let mut mst = DcMstring::new();
        lp.encode_dc_mixed(&mut mst)?;

        let decoded = mst.decode::<LinkPath>(&strict)?;
        // Typed representation preserves segments
        ensure!(
            decoded.segments() == lp.segments(),
            "Segments mismatch for {:?}: got {:?}",
            String::from_utf8_lossy(bytes),
            decoded.segments()
        );
    }
    Ok(())
}

// ============================================================================
// 2. STREAMNAME EMPIRICAL STRESS TESTS
// ============================================================================

#[crate::ctb_test]
fn test_challenge_streamname_wtf8_and_invalid_utf8() -> Result<()> {
    let strict = DcProjectionPolicy::strict();

    // 1. WTF-8 with lone surrogates
    // U+D800 in WTF-8 is [0xed, 0xa0, 0x80]
    // U+DFFF in WTF-8 is [0xed, 0xbf, 0xbf]
    let surrogate_u16 = [0xd800u16, 0x0061u16, 0xdfffu16];
    let name_from_utf16 = StreamName::from_windows_utf16(&surrogate_u16);

    let mut mst = DcMstring::new();
    name_from_utf16.encode_dc_mixed(&mut mst)?;

    let decoded: StreamName = mst.decode(&strict)?;
    ensure!(
        decoded == name_from_utf16,
        "WTF-8 surrogate stream name failed to roundtrip"
    );

    // 2. Arbitrary invalid UTF-8 bytes: [0xff, 0xfe, 0x00, 0x80]
    let raw_invalid = b"\xff\xfe\x00\x80";
    let name_invalid = StreamName::from_bytes(raw_invalid);

    let mut mst_invalid = DcMstring::new();
    name_invalid.encode_dc_mixed(&mut mst_invalid)?;

    let decoded_invalid: StreamName = mst_invalid.decode(&strict)?;
    ensure!(
        decoded_invalid.as_bytes().as_ref() == raw_invalid,
        "Invalid UTF-8 byte stream name failed to roundtrip exact bytes"
    );

    // 3. WTF-8 format tag validation:
    // If bytes are canonical WTF-8, it decodes to WindowsUtf16
    let wtf8_bytes = vec![0xed, 0xa0, 0x80];
    let from_enc = StreamName::from_encoded(FormatId::Wtf8, wtf8_bytes.clone())?;
    match from_enc {
        StreamName::WindowsUtf16(ref units) => {
            ensure!(units == &[0xd800], "Expected single surrogate unit 0xd800");
        }
        other => bail!("Expected WindowsUtf16 for canonical WTF-8, got {other:?}"),
    }

    Ok(())
}

#[crate::ctb_test]
fn test_challenge_streamname_long_names_stress() -> Result<()> {
    let strict = DcProjectionPolicy::strict();

    // Stress with long stream names: 4 KiB, 64 KiB, 128 KiB
    for size in [4 * 1024, 64 * 1024, 128 * 1024] {
        let pattern = b"user.long_attribute_name_chunk_1234567890_";
        let mut long_bytes = Vec::with_capacity(size);
        while long_bytes.len() < size {
            long_bytes.extend_from_slice(pattern);
        }
        long_bytes.truncate(size);

        let name = StreamName::from_bytes(&long_bytes);
        let mut mst = DcMstring::new();
        name.encode_dc_mixed(&mut mst)?;

        let decoded: StreamName = mst.decode(&strict)?;
        ensure!(
            decoded.as_bytes().as_ref() == long_bytes.as_slice(),
            "Stream name of size {size} failed roundtrip"
        );
    }
    Ok(())
}

// ============================================================================
// 3. ATTACHEDSTREAM EMPIRICAL STRESS TESTS
// ============================================================================

#[crate::ctb_test]
fn test_challenge_attached_stream_missing_data_tag() -> Result<()> {
    let strict = DcProjectionPolicy::strict();

    // 1. AttachedStream with data = None (e.g. metadata-only stream)
    let stream_no_data = AttachedStream {
        name: Some(StreamName::from_str("user.test_attr")),
        kind: StreamKind::ExtendedAttribute,
        entity: Box::new(FileEntity::from_vec(Vec::new())),
        data: None,
    };

    let mut mst = DcMstring::new();
    stream_no_data.encode_dc_mixed(&mut mst)?;

    // Verify Dc 385 (stream payload tag) is NOT present when data is None
    let dc_385 = DcChar::from_short(385);
    let dcs = mst.to_dc_string()?;
    ensure!(
        !dcs.chars().any(|c| c == dc_385),
        "Dc 385 data tag must not be emitted when data is None"
    );

    // Decode and ensure data is None
    let decoded: AttachedStream = mst.decode(&strict)?;
    ensure!(
        decoded.data.is_none(),
        "Decoded stream data must be None when Dc 385 is omitted"
    );
    ensure!(decoded.kind == StreamKind::ExtendedAttribute);
    ensure!(decoded.name == stream_no_data.name);

    // 2. AttachedStream with empty data: data = Some(vec![])
    let stream_empty_data = AttachedStream {
        name: Some(StreamName::from_str("user.empty_payload")),
        kind: StreamKind::ExtendedAttribute,
        entity: Box::new(FileEntity::from_vec(Vec::new())),
        data: Some(Vec::new()),
    };

    let mut mst_empty = DcMstring::new();
    stream_empty_data.encode_dc_mixed(&mut mst_empty)?;

    let decoded_empty: AttachedStream = mst_empty.decode(&strict)?;
    ensure!(
        decoded_empty.data == Some(Vec::new()),
        "Decoded stream data must be Some(vec![])"
    );

    // 3. AttachedStream with payload data: data = Some(vec![1, 2, 3, 4, 5])
    let payload = vec![1u8, 2, 3, 4, 5, 255, 254, 0];
    let stream_with_data = AttachedStream {
        name: Some(StreamName::from_str("user.with_payload")),
        kind: StreamKind::NtfsAlternateDataStream,
        entity: Box::new(FileEntity::from_vec(Vec::new())),
        data: Some(payload.clone()),
    };

    let mut mst_data = DcMstring::new();
    stream_with_data.encode_dc_mixed(&mut mst_data)?;

    let decoded_data: AttachedStream = mst_data.decode(&strict)?;
    ensure!(
        decoded_data.data == Some(payload),
        "Decoded stream payload mismatch"
    );
    ensure!(decoded_data.kind == StreamKind::NtfsAlternateDataStream);

    Ok(())
}

#[crate::ctb_test]
fn test_challenge_attached_stream_missing_fields_and_corrupt_framing() -> Result<()> {
    let strict = DcProjectionPolicy::strict();

    // 1. Missing StreamKind: Container 323..324 with FileEntity (317..318) but no StreamKind
    let mut missing_kind_mst = DcMstring::new();
    missing_kind_mst.push_char(DcChar::from_short(323)); // container begin
    // FileEntity
    let entity = FileEntity::from_vec(Vec::new());
    entity.encode_dc_mixed(&mut missing_kind_mst)?;
    missing_kind_mst.push_char(DcChar::from_short(324)); // container end

    let res = missing_kind_mst.decode::<AttachedStream>(&strict);
    ensure!(
        res.is_err(),
        "Missing StreamKind MUST cause decode to error, but succeeded"
    );

    // 2. Missing FileEntity: Container 323..324 with StreamKind but no FileEntity
    let mut missing_entity_mst = DcMstring::new();
    missing_entity_mst.push_char(DcChar::from_short(323));
    StreamKind::ExtendedAttribute.encode_dc_mixed(&mut missing_entity_mst)?;
    missing_entity_mst.push_char(DcChar::from_short(324));

    let res2 = missing_entity_mst.decode::<AttachedStream>(&strict);
    ensure!(
        res2.is_err(),
        "Missing FileEntity MUST cause decode to error, but succeeded"
    );

    // 3. Corrupt container framing: not 323..324
    let mut bad_container_mst = DcMstring::new();
    bad_container_mst.push_char(DcChar::from_short(310)); // list opener, not 323
    StreamKind::ExtendedAttribute.encode_dc_mixed(&mut bad_container_mst)?;
    bad_container_mst.push_char(DcChar::from_short(311));

    let res3 = bad_container_mst.decode::<AttachedStream>(&strict);
    ensure!(
        res3.is_err(),
        "Bad container framing MUST cause decode to error, but succeeded"
    );

    Ok(())
}

#[crate::ctb_test]
fn test_challenge_attached_stream_nameless_and_wtf8() -> Result<()> {
    let strict = DcProjectionPolicy::strict();

    // 1. Nameless stream (Classic Mac OS resource fork has name: None)
    let nameless = AttachedStream {
        name: None,
        kind: StreamKind::MacOsResourceFork,
        entity: Box::new(FileEntity::from_vec(Vec::new())),
        data: Some(b"RESOURCE_FORK_RAW_DATA".to_vec()),
    };

    let mut mst = DcMstring::new();
    nameless.encode_dc_mixed(&mut mst)?;

    let decoded: AttachedStream = mst.decode(&strict)?;
    ensure!(decoded.name.is_none(), "Nameless stream must decode name as None");
    ensure!(decoded.kind == StreamKind::MacOsResourceFork);
    ensure!(decoded.data == Some(b"RESOURCE_FORK_RAW_DATA".to_vec()));

    // 2. WTF-8 stream name with lone surrogates inside AttachedStream
    let wtf8_name = StreamName::from_windows_utf16(&[0xd83d, 0x0061, 0xde00]);
    let wtf8_stream = AttachedStream {
        name: Some(wtf8_name.clone()),
        kind: StreamKind::NtfsAlternateDataStream,
        entity: Box::new(FileEntity::from_vec(Vec::new())),
        data: Some(vec![42]),
    };

    let mut mst_wtf8 = DcMstring::new();
    wtf8_stream.encode_dc_mixed(&mut mst_wtf8)?;

    let decoded_wtf8: AttachedStream = mst_wtf8.decode(&strict)?;
    ensure!(decoded_wtf8.name == Some(wtf8_name));
    ensure!(decoded_wtf8.kind == StreamKind::NtfsAlternateDataStream);
    ensure!(decoded_wtf8.data == Some(vec![42]));

    Ok(())
}

// ============================================================================
// 4. METADATA STRUCTS EMPIRICAL STRESS TESTS (FileTimestamps, PlatformRawFlags, AppleMetadata, NativeMetadata)
// ============================================================================

#[crate::ctb_test]
fn test_challenge_file_timestamps_strict_rejection_and_resolution() -> Result<()> {
    let strict = DcProjectionPolicy::strict();
    let forgiving = DcProjectionPolicy::forgiving();

    // 1. Valid FileTimestamps with high-precision nanoseconds and birthtime inside FileMetadata
    let ts = FileTimestamps {
        atime_sec: 1700000000,
        atime_nsec: 123456789,
        mtime_sec: 1700000100,
        mtime_nsec: 987654321,
        ctime_sec: 1700000200,
        ctime_nsec: 555555555,
        birthtime_sec: Some(1699999999),
        birthtime_nsec: Some(111222333),
        resolution_nsec: Some(1),
    };

    let meta = ctb_io_file::FileMetadata {
        timestamps: Some(ts),
        ..ctb_io_file::FileMetadata::empty()
    };

    let mut mst = DcMstring::new();
    meta.encode_dc_mixed(&mut mst)?;

    let decoded_strict: ctb_io_file::FileMetadata = mst.decode(&strict)?;
    ensure!(decoded_strict.timestamps == Some(ts), "Strict roundtrip mismatch for FileTimestamps");

    let decoded_forgiving: ctb_io_file::FileMetadata = mst.decode(&forgiving)?;
    ensure!(decoded_forgiving.timestamps == Some(ts), "Forgiving roundtrip mismatch for FileTimestamps");

    // 2. Negative timestamps (before Unix epoch, e.g. 1950-01-01)
    let neg_ts = FileTimestamps {
        atime_sec: -631152000,
        atime_nsec: 500,
        mtime_sec: -631152000,
        mtime_nsec: 500,
        ctime_sec: -631152000,
        ctime_nsec: 500,
        birthtime_sec: Some(-631152000),
        birthtime_nsec: Some(500),
        resolution_nsec: None,
    };
    let neg_meta = ctb_io_file::FileMetadata {
        timestamps: Some(neg_ts),
        ..ctb_io_file::FileMetadata::empty()
    };
    let mut neg_mst = DcMstring::new();
    neg_meta.encode_dc_mixed(&mut neg_mst)?;
    let decoded_neg: ctb_io_file::FileMetadata = neg_mst.decode(&strict)?;
    ensure!(decoded_neg.timestamps == Some(neg_ts), "Negative timestamp roundtrip mismatch");

    // 3. Strict rejection of bare timestamp number without Dc 390..391 time moment framing
    let mut bare_ts = DcMstring::new();
    bare_ts.push_char(DcChar::from_short(330)); // atime tag
    let raw_nanos = 1700000000i128.checked_mul(1_000_000_000).unwrap();
    raw_nanos.encode_dc_mixed(&mut bare_ts)?; // bare number, no 390..391

    let strict_bare_res = bare_ts.decode::<FileTimestamps>(&strict);
    ensure!(
        strict_bare_res.is_err(),
        "Strict decode MUST reject bare timestamp number without Dc 390..391, but succeeded"
    );

    // 4. Forgiving fallback for bare/incomplete timestamp
    let forgiving_bare_res = bare_ts.decode::<FileTimestamps>(&forgiving)?;
    ensure!(
        forgiving_bare_res.atime_sec == 0,
        "Forgiving decode should fall back to default (got {})",
        forgiving_bare_res.atime_sec
    );

    Ok(())
}

#[crate::ctb_test]
fn test_challenge_platform_raw_flags_safety_and_width() -> Result<()> {
    let strict = DcProjectionPolicy::strict();

    // 1. Valid 8-byte raw flags (64-bit)
    let flags8 = PlatformRawFlags::from_raw(0x0102030405060708);
    let mut mst8 = DcMstring::new();
    flags8.encode_dc_mixed(&mut mst8)?;

    let decoded8: PlatformRawFlags = mst8.decode(&strict)?;
    ensure!(decoded8 == flags8, "8-byte PlatformRawFlags roundtrip mismatch");
    ensure!(decoded8.raw_value == 0x0102030405060708);

    // 2. Valid 4-byte raw flags (32-bit legacy BSD chflags)
    let mut mst4 = DcMstring::new();
    mst4.push_binary_with_sha256(&0x01020304u32.to_le_bytes())?;
    let decoded4: PlatformRawFlags = mst4.decode(&strict)?;
    ensure!(decoded4.raw_value == 0x01020304);

    // 3. Reject invalid width binary payloads (not 4 or 8 bytes: e.g. 1, 3, 5, 7, 9 bytes)
    for bad_len in [1, 3, 5, 7, 9, 16] {
        let mut bad_mst = DcMstring::new();
        bad_mst.push_binary_with_sha256(&vec![0x42u8; bad_len])?;
        ensure!(
            bad_mst.decode::<PlatformRawFlags>(&strict).is_err(),
            "{bad_len}-byte payload MUST be rejected for PlatformRawFlags"
        );
    }

    Ok(())
}

#[crate::ctb_test]
fn test_challenge_apple_and_native_metadata_framing() -> Result<()> {
    let strict = DcProjectionPolicy::strict();

    // 1. AppleMetadata framing: Container must be 401..402
    let mut bad_apple = DcMstring::new();
    bad_apple.push_char(DcChar::from_short(310)); // list opener, not 401
    bad_apple.push_char(DcChar::from_short(311));
    ensure!(
        bad_apple.decode::<AppleMetadata>(&strict).is_err(),
        "AppleMetadata without 401..402 framing MUST fail strict decode"
    );

    // 2. NativeMetadata framing: Container must be 559..560 (or legacy 313..314)
    let mut bad_native = DcMstring::new();
    bad_native.push_char(DcChar::from_short(310)); // list opener, not 559
    bad_native.push_char(DcChar::from_short(311));
    ensure!(
        bad_native.decode::<NativeMetadata>(&strict).is_err(),
        "NativeMetadata without map container framing MUST fail strict decode"
    );

    Ok(())
}

