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

//! Adversarial Empirical Challenge Suite for Milestone 3 Remediation:
//! 1. PlatformRawFlags: 4-byte and 8-byte valid payloads, endianness, boundary
//!    values, and exhaustive rejection of invalid payload lengths and node types.
//! 2. AttachedStream: serialization roundtrip without ad-hoc token patching,
//!    nameless and named streams, WTF-8 names, data payload variations, and
//!    integration in FileEntity.
//! 3. PathBuf and Option<PathBuf>: strict rejection of unformatted bare strings
//!    and bare binary payloads vs forgiving recovery across standalone decoders
//!    and derived struct fields.

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
use ctb_formats_dcstring::dc::*;
use ctb_formats_dcstring::{
    DcChar, DcDocumentNavigator, DcMixedDecode, DcMixedEncode, DcMstring,
    DcProjectionContext, DcProjectionPolicy, DcSyntaxArena, assert_dc_roundtrip,
};
use ctb_io_file::metadata::PlatformRawFlags;
use ctb_io_file::streams::{AttachedStream, StreamKind, StreamName};
use ctb_io_file::{FileEntity, FileIdentity, FileOrigin};
use std::path::PathBuf;

// ============================================================================
// 1. PLATFORM RAW FLAGS: EMPIRICAL CHALLENGES
// ============================================================================

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

    // Case 1D: 4-byte and 8-byte payloads represented via [u8] list (Dc 310 .. Dc 311)
    let mut mst_list4 = DcMstring::new();
    mst_list4.push_char(DC_BEGIN_LIST);
    for b in val32.to_le_bytes() {
        b.encode_dc_mixed(&mut mst_list4)?;
    }
    mst_list4.push_char(DC_END_LIST);
    let dec_list4: PlatformRawFlags = mst_list4.decode(&strict)?;
    ensure!(
        dec_list4.raw_value == u64::from(val32),
        "4-byte [u8] list PlatformRawFlags decode mismatch"
    );

    let mut mst_list8 = DcMstring::new();
    mst_list8.push_char(DC_BEGIN_LIST);
    for b in val64.to_le_bytes() {
        b.encode_dc_mixed(&mut mst_list8)?;
    }
    mst_list8.push_char(DC_END_LIST);
    let dec_list8: PlatformRawFlags = mst_list8.decode(&strict)?;
    ensure!(
        dec_list8.raw_value == val64,
        "8-byte [u8] list PlatformRawFlags decode mismatch"
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
fn test_attached_stream_serialization_clean_roundtrip() -> Result<()> {
    let strict = DcProjectionPolicy::strict();

    // Verify all stream kinds and data combinations
    let test_cases = vec![
        // Nameless stream with no data
        AttachedStream {
            name: None,
            kind: StreamKind::MacOsResourceFork,
            entity: Box::new(FileEntity::from_vec(Vec::new())),
            data: None,
        },
        // Nameless stream with empty data
        AttachedStream {
            name: None,
            kind: StreamKind::MacOsResourceFork,
            entity: Box::new(FileEntity::from_vec(Vec::new())),
            data: Some(Vec::new()),
        },
        // Nameless stream with binary data
        AttachedStream {
            name: None,
            kind: StreamKind::ExtendedAttribute,
            entity: Box::new(FileEntity::from_vec(Vec::new())),
            data: Some(vec![0xDE, 0xAD, 0xBE, 0xEF]),
        },
        // Named stream with ASCII name and data
        AttachedStream {
            name: Some(StreamName::from_str("user.mime_type")),
            kind: StreamKind::ExtendedAttribute,
            entity: Box::new(FileEntity::from_vec(Vec::new())),
            data: Some(b"text/plain".to_vec()),
        },
        // Named stream with Unicode name
        AttachedStream {
            name: Some(StreamName::from_str("com.apple.metadata:kMDItemWhereFroms_üñîçødë")),
            kind: StreamKind::ExtendedAttribute,
            entity: Box::new(FileEntity::from_vec(Vec::new())),
            data: Some(b"https://example.com/test".to_vec()),
        },
        // Named NTFS alternate data stream with substantial binary data
        AttachedStream {
            name: Some(StreamName::from_str("Zone.Identifier")),
            kind: StreamKind::NtfsAlternateDataStream,
            entity: Box::new(FileEntity::from_vec(Vec::new())),
            data: Some(vec![42u8; 1024]),
        },
        // Stream with PosixAclAccess kind and no data
        AttachedStream {
            name: Some(StreamName::from_str("system.posix_acl_access")),
            kind: StreamKind::PosixAclAccess,
            entity: Box::new(FileEntity::from_vec(Vec::new())),
            data: None,
        },
        // Stream with SecurityLabel kind and data
        AttachedStream {
            name: Some(StreamName::from_str("security.selinux")),
            kind: StreamKind::SecurityLabel,
            entity: Box::new(FileEntity::from_vec(Vec::new())),
            data: Some(b"system_u:object_r:bin_t:s0".to_vec()),
        },
    ];

    for stream in test_cases {
        let mut mst = DcMstring::new();
        stream.encode_dc_mixed(&mut mst)?;

        // Verify structural framing:
        // Container must start with Dc 323 and end with Dc 324
        let dcs = mst.to_dc_string()?;
        let chars: Vec<DcChar> = dcs.chars().collect();
        ensure!(
            chars.first() == Some(&DcChar::from_short(323)),
            "AttachedStream must begin with Dc 323"
        );
        ensure!(
            chars.last() == Some(&DcChar::from_short(324)),
            "AttachedStream must end with Dc 324"
        );

        // Verify Dc 385 tag is present if and only if stream.data.is_some()
        let has_dc_385 = chars.iter().any(|&c| c == DcChar::from_short(385));
        if stream.data.is_some() {
            ensure!(has_dc_385, "Expected Dc 385 tag when stream data is present");
        } else {
            ensure!(!has_dc_385, "Dc 385 tag must not be present when stream data is None");
        }

        // Verify Dc 338 tag is present if and only if stream.name.is_some()
        let has_dc_338 = chars.iter().any(|&c| c == DcChar::from_short(338));
        if stream.name.is_some() {
            ensure!(has_dc_338, "Expected Dc 338 tag when stream name is present");
        } else {
            ensure!(!has_dc_338, "Dc 338 tag must not be present when stream name is None");
        }

        // Decode under strict policy and verify lossless roundtrip
        let decoded: AttachedStream = mst.decode(&strict)?;
        ensure!(decoded == stream, "AttachedStream roundtrip mismatch for {stream:?}");
    }

    Ok(())
}

#[crate::ctb_test]
fn test_attached_stream_in_file_entity_roundtrip() -> Result<()> {
    let strict = DcProjectionPolicy::strict();

    let mut entity = FileEntity::from_vec(b"main payload content".to_vec());
    entity.streams = vec![
        AttachedStream {
            name: Some(StreamName::from_str("user.stream_1")),
            kind: StreamKind::ExtendedAttribute,
            entity: Box::new(FileEntity::from_vec(Vec::new())),
            data: Some(b"attr1_value".to_vec()),
        },
        AttachedStream {
            name: None,
            kind: StreamKind::MacOsResourceFork,
            entity: Box::new(FileEntity::from_vec(Vec::new())),
            data: Some(vec![1, 2, 3, 4]),
        },
        AttachedStream {
            name: Some(StreamName::from_str("SummaryInformation")),
            kind: StreamKind::NtfsAlternateDataStream,
            entity: Box::new(FileEntity::from_vec(Vec::new())),
            data: None,
        },
    ];

    let mut mst = DcMstring::new();
    entity.encode_dc_mixed(&mut mst)?;

    let decoded: FileEntity = mst.decode(&strict)?;
    ensure!(decoded.streams.len() == 3, "Decoded stream count mismatch");
    ensure!(decoded.streams == entity.streams, "Attached streams in FileEntity roundtrip mismatch");

    Ok(())
}

// ============================================================================
// 3. PATHBUF & OPTION<PATHBUF>: STRICT REJECTION VS FORGIVING RECOVERY
// ============================================================================

#[crate::ctb_test]
fn test_pathbuf_standalone_strict_rejection_vs_forgiving_recovery() -> Result<()> {
    let strict = DcProjectionPolicy::strict();
    let forgiving = DcProjectionPolicy::forgiving();

    // 1. Valid formatted PathBuf (Dc 511 + f494)
    let valid_path = PathBuf::from("var/log/app.log");
    let mut valid_mst = DcMstring::new();
    valid_path.encode_dc_mixed(&mut valid_mst)?;

    let decoded_strict: PathBuf = valid_mst.decode(&strict)?;
    ensure!(decoded_strict == valid_path, "Strict failed to decode formatted PathBuf");
    let decoded_forgiving: PathBuf = valid_mst.decode(&forgiving)?;
    ensure!(decoded_forgiving == valid_path, "Forgiving failed to decode formatted PathBuf");

    // 2. Bare string representation of PathBuf
    let mut bare_str_mst = DcMstring::new();
    "var/log/app.log".encode_dc_mixed(&mut bare_str_mst)?;

    // Strict policy MUST reject bare string
    let strict_bare_res = bare_str_mst.decode::<PathBuf>(&strict);
    ensure!(
        strict_bare_res.is_err(),
        "Strict policy MUST reject bare string for PathBuf, but succeeded"
    );

    // Forgiving policy MUST recover bare string
    let forgiving_bare: PathBuf = bare_str_mst.decode(&forgiving)?;
    ensure!(
        forgiving_bare == valid_path,
        "Forgiving policy failed to recover bare string for PathBuf"
    );

    // 3. Bare binary representation of PathBuf
    let mut bare_bin_mst = DcMstring::new();
    bare_bin_mst.push_binary_with_sha256(b"var/log/app.log")?;

    // Strict policy MUST reject bare binary
    let strict_bin_res = bare_bin_mst.decode::<PathBuf>(&strict);
    ensure!(
        strict_bin_res.is_err(),
        "Strict policy MUST reject bare binary for PathBuf, but succeeded"
    );

    // Forgiving policy MUST recover bare binary
    let forgiving_bin: PathBuf = bare_bin_mst.decode(&forgiving)?;
    ensure!(
        forgiving_bin == valid_path,
        "Forgiving policy failed to recover bare binary for PathBuf"
    );

    Ok(())
}

#[crate::ctb_test]
fn test_option_pathbuf_standalone_strict_vs_forgiving() -> Result<()> {
    let strict = DcProjectionPolicy::strict();
    let forgiving = DcProjectionPolicy::forgiving();

    // 1. None (Dc 316)
    let mut none_mst = DcMstring::new();
    none_mst.push_char(DC_OPTIONAL_ABSENT); // 316
    let dec_none_strict: Option<PathBuf> = none_mst.decode(&strict)?;
    ensure!(dec_none_strict.is_none(), "Strict decode of Dc 316 must be None");
    let dec_none_forgiving: Option<PathBuf> = none_mst.decode(&forgiving)?;
    ensure!(dec_none_forgiving.is_none(), "Forgiving decode of Dc 316 must be None");

    // 2. Some(formatted PathBuf) (Dc 315 + format 511 + f494)
    let opt_some = Some(PathBuf::from("usr/bin/python"));
    let mut some_mst = DcMstring::new();
    opt_some.encode_dc_mixed(&mut some_mst)?;

    let dec_some_strict: Option<PathBuf> = some_mst.decode(&strict)?;
    ensure!(dec_some_strict == opt_some, "Strict decode of formatted Option<PathBuf> mismatch");
    let dec_some_forgiving: Option<PathBuf> = some_mst.decode(&forgiving)?;
    ensure!(dec_some_forgiving == opt_some, "Forgiving decode of formatted Option<PathBuf> mismatch");

    // 3. Bare string inside Dc 315: 315 "usr/bin/python"
    let mut bare_some_mst = DcMstring::new();
    bare_some_mst.push_char(DC_OPTIONAL_PRESENT);
    "usr/bin/python".encode_dc_mixed(&mut bare_some_mst)?;

    // Strict MUST reject bare string inside Option
    let res_strict = bare_some_mst.decode::<Option<PathBuf>>(&strict);
    ensure!(
        res_strict.is_err(),
        "Strict decode of Option<PathBuf> with bare string MUST error, but succeeded"
    );

    // Forgiving MUST recover bare string inside Option
    let res_forgiving: Option<PathBuf> = bare_some_mst.decode(&forgiving)?;
    ensure!(
        res_forgiving == opt_some,
        "Forgiving decode of Option<PathBuf> with bare string failed to recover"
    );

    // 4. Bare binary inside Dc 315: 315 binary(b"usr/bin/python")
    let mut bare_bin_some_mst = DcMstring::new();
    bare_bin_some_mst.push_char(DC_OPTIONAL_PRESENT);
    bare_bin_some_mst.push_binary_with_sha256(b"usr/bin/python")?;

    // Strict MUST reject bare binary inside Option
    let res_bin_strict = bare_bin_some_mst.decode::<Option<PathBuf>>(&strict);
    ensure!(
        res_bin_strict.is_err(),
        "Strict decode of Option<PathBuf> with bare binary MUST error, but succeeded"
    );

    // Forgiving MUST recover bare binary inside Option
    let res_bin_forgiving: Option<PathBuf> = bare_bin_some_mst.decode(&forgiving)?;
    ensure!(
        res_bin_forgiving == opt_some,
        "Forgiving decode of Option<PathBuf> with bare binary failed to recover"
    );

    Ok(())
}

#[crate::ctb_test]
fn test_derived_struct_option_pathbuf_strict_vs_forgiving() -> Result<()> {
    let strict = DcProjectionPolicy::strict();
    let forgiving = DcProjectionPolicy::forgiving();

    // Case 3A: Valid formatted Option<PathBuf> in FileIdentity
    let valid_id = FileIdentity {
        origin: FileOrigin::Synthetic,
        relative_path: Some(PathBuf::from("docs/report.pdf")),
        enclosing_path: Some(PathBuf::from("/var/data")),
        nlink: 1,
        hardlink_group: None,
    };
    let mut valid_mst = DcMstring::new();
    valid_id.encode_dc_mixed(&mut valid_mst)?;

    let dec_strict: FileIdentity = valid_mst.decode(&strict)?;
    ensure!(dec_strict == valid_id, "Valid FileIdentity strict roundtrip mismatch");
    let dec_forgiving: FileIdentity = valid_mst.decode(&forgiving)?;
    ensure!(dec_forgiving == valid_id, "Valid FileIdentity forgiving roundtrip mismatch");

    // Case 3B: FileIdentity with unformatted bare string in relative_path
    // Framing: 319 (begin FileIdentity), origin, 337 (relative_path), "docs/report.pdf", 355 (nlink), 1, 320 (end)
    let mut bare_str_id_mst = DcMstring::new();
    bare_str_id_mst.push_char(DcChar::from_short(319));
    FileOrigin::Synthetic.encode_dc_mixed(&mut bare_str_id_mst)?;
    bare_str_id_mst.push_char(DcChar::from_short(337));
    "docs/report.pdf".encode_dc_mixed(&mut bare_str_id_mst)?;
    bare_str_id_mst.push_char(DcChar::from_short(355));
    1u64.encode_dc_mixed(&mut bare_str_id_mst)?;
    bare_str_id_mst.push_char(DcChar::from_short(320));

    // Strict policy MUST reject unformatted bare string in struct field
    let res_strict = bare_str_id_mst.decode::<FileIdentity>(&strict);
    ensure!(
        res_strict.is_err(),
        "Strict policy MUST reject FileIdentity with bare string relative_path, but succeeded"
    );

    // Forgiving policy MUST recover bare string in struct field
    let res_forgiving: FileIdentity = bare_str_id_mst.decode(&forgiving)?;
    ensure!(
        res_forgiving.relative_path == Some(PathBuf::from("docs/report.pdf")),
        "Forgiving policy failed to recover bare string relative_path in FileIdentity"
    );

    // Case 3C: FileIdentity with unformatted bare binary in relative_path
    let mut bare_bin_id_mst = DcMstring::new();
    bare_bin_id_mst.push_char(DcChar::from_short(319));
    FileOrigin::Synthetic.encode_dc_mixed(&mut bare_bin_id_mst)?;
    bare_bin_id_mst.push_char(DcChar::from_short(337));
    bare_bin_id_mst.push_binary_with_sha256(b"docs/report.pdf")?;
    bare_bin_id_mst.push_char(DcChar::from_short(355));
    1u64.encode_dc_mixed(&mut bare_bin_id_mst)?;
    bare_bin_id_mst.push_char(DcChar::from_short(320));

    let res_bin_strict = bare_bin_id_mst.decode::<FileIdentity>(&strict);
    ensure!(
        res_bin_strict.is_err(),
        "Strict policy MUST reject FileIdentity with bare binary relative_path, but succeeded"
    );

    let res_bin_forgiving: FileIdentity = bare_bin_id_mst.decode(&forgiving)?;
    ensure!(
        res_bin_forgiving.relative_path == Some(PathBuf::from("docs/report.pdf")),
        "Forgiving policy failed to recover bare binary relative_path in FileIdentity"
    );

    // Case 3D: FileIdentity with legacy v4 equivalent list in relative_path:
    // 337 (relative_path) -> 315 [ 397 [ 310 "docs/report.pdf" binary 311 ] ]
    let mut v4_id_mst = DcMstring::new();
    v4_id_mst.push_char(DcChar::from_short(319));
    FileOrigin::Synthetic.encode_dc_mixed(&mut v4_id_mst)?;
    v4_id_mst.push_char(DcChar::from_short(337));
    v4_id_mst.push_char(DC_OPTIONAL_PRESENT); // 315
    v4_id_mst.push_char(DC_EQUIVALENT_REPRESENTATIONS); // 397
    v4_id_mst.push_char(DC_BEGIN_LIST); // 310
    "docs/report.pdf".encode_dc_mixed(&mut v4_id_mst)?;
    v4_id_mst.push_binary_with_sha256(b"docs/report.pdf")?;
    v4_id_mst.push_char(DC_END_LIST); // 311
    v4_id_mst.push_char(DcChar::from_short(355));
    1u64.encode_dc_mixed(&mut v4_id_mst)?;
    v4_id_mst.push_char(DcChar::from_short(320));

    let res_v4_strict = v4_id_mst.decode::<FileIdentity>(&strict);
    ensure!(
        res_v4_strict.is_err(),
        "Strict policy MUST reject FileIdentity with legacy v4 equivalent list, but succeeded"
    );

    let res_v4_forgiving: FileIdentity = v4_id_mst.decode(&forgiving)?;
    ensure!(
        res_v4_forgiving.relative_path == Some(PathBuf::from("docs/report.pdf")),
        "Forgiving policy failed to recover legacy v4 equivalent list relative_path"
    );

    Ok(())
}

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

#[crate::ctb_test]
fn test_attached_stream_adversarial_framing_and_assert_roundtrip() -> Result<()> {
    let strict = DcProjectionPolicy::strict();

    // 1. assert_dc_roundtrip on standard AttachedStream
    let stream = AttachedStream {
        name: Some(StreamName::from_str("user.checksum")),
        kind: StreamKind::ExtendedAttribute,
        entity: Box::new(FileEntity::from_vec(b"test_entity".to_vec())),
        data: Some(vec![0xAA, 0xBB, 0xCC, 0xDD]),
    };
    assert_dc_roundtrip(&stream)?;

    // 2. assert_dc_roundtrip on nameless stream
    let nameless = AttachedStream {
        name: None,
        kind: StreamKind::MacOsResourceFork,
        entity: Box::new(FileEntity::from_vec(Vec::new())),
        data: None,
    };
    assert_dc_roundtrip(&nameless)?;

    // 3. Malformed data payload: Tagged 385 wrapping non-binary/non-list literal node
    let mut arena = DcSyntaxArena::new();
    let span = ctb_formats_dcstring::navigator::DcByteSpan::new(0, 10);
    let str_node = arena.alloc(
        span,
        ctb_formats_dcstring::navigator::DcSyntaxKind::Literal {
            type_header: Vec::new(),
            decoded: "hello".to_string(),
        },
    );
    let tag_385 = arena.alloc(
        span,
        ctb_formats_dcstring::navigator::DcSyntaxKind::Tagged {
            tag: DcChar::from_short(385),
            args: vec![str_node],
        },
    );
    let kind_atom = arena.alloc(
        span,
        ctb_formats_dcstring::navigator::DcSyntaxKind::Atom(DcChar::from_short(379)),
    );
    let entity_node = arena.alloc(
        span,
        ctb_formats_dcstring::navigator::DcSyntaxKind::Container {
            opener: DcChar::from_short(317),
            closer: DcChar::from_short(318),
            children: Vec::new(),
            header: Vec::new(),
            trailing: Vec::new(),
            unclosed: false,
        },
    );
    let root = arena.alloc(
        span,
        ctb_formats_dcstring::navigator::DcSyntaxKind::Container {
            opener: DcChar::from_short(323),
            closer: DcChar::from_short(324),
            children: vec![kind_atom, entity_node, tag_385],
            header: Vec::new(),
            trailing: Vec::new(),
            unclosed: false,
        },
    );
    let mut ctx = DcProjectionContext::new(&arena, &strict);
    let res = AttachedStream::decode_dc_node(&mut ctx, root);
    ensure!(
        res.is_err(),
        "AttachedStream must error when Tagged 385 contains a non-binary/list arg"
    );

    // 4. Invalid stream kind atom (Dc 999) inside container 323..324
    let mut mst_bad_kind = DcMstring::new();
    mst_bad_kind.push_char(DcChar::from_short(323));
    mst_bad_kind.push_char(DcChar::from_short(999)); // not a StreamKind atom
    FileEntity::from_vec(Vec::new()).encode_dc_mixed(&mut mst_bad_kind)?;
    mst_bad_kind.push_char(DcChar::from_short(324));

    ensure!(
        mst_bad_kind.decode::<AttachedStream>(&strict).is_err(),
        "AttachedStream without valid StreamKind atom must error"
    );

    // 5. Container 323..324 with unknown extra tokens (should be safely ignored)
    let mut mst_extra = DcMstring::new();
    mst_extra.push_char(DcChar::from_short(323));
    // Unknown atom
    mst_extra.push_char(DcChar::from_short(299));
    // Valid StreamKind
    StreamKind::NtfsAlternateDataStream.encode_dc_mixed(&mut mst_extra)?;
    // Unknown integer
    99999u64.encode_dc_mixed(&mut mst_extra)?;
    // Valid FileEntity
    FileEntity::from_vec(Vec::new()).encode_dc_mixed(&mut mst_extra)?;
    mst_extra.push_char(DcChar::from_short(324));

    let dec_extra: AttachedStream = mst_extra.decode(&strict)?;
    ensure!(dec_extra.kind == StreamKind::NtfsAlternateDataStream);
    ensure!(dec_extra.name.is_none());
    ensure!(dec_extra.data.is_none());

    Ok(())
}

#[crate::ctb_test]
fn test_option_pathbuf_equivalent_list_separation() -> Result<()> {
    let strict = DcProjectionPolicy::strict();
    let forgiving = DcProjectionPolicy::forgiving();

    let path = PathBuf::from("usr/include/stdio.h");
    let mut path_mst = DcMstring::new();
    path.encode_dc_mixed(&mut path_mst)?;

    // 1. Well-formed equivalent list: 397 [ 275 formatted_path ; formatted_path 311 ]
    let mut well_formed = DcMstring::new();
    well_formed.push_char(DC_EQUIVALENT_REPRESENTATIONS); // 397
    well_formed.push_char(DC_BEGIN_LIST); // 310
    well_formed.push_char(DcChar::from_short(275)); // 275
    path.encode_dc_mixed(&mut well_formed)?;
    well_formed.push_char(DC_LIST_ELEMENT_SEPARATOR); // 312
    path.encode_dc_mixed(&mut well_formed)?;
    well_formed.push_char(DC_END_LIST); // 311

    let dec_wf_strict: PathBuf = well_formed.decode(&strict)?;
    ensure!(dec_wf_strict == path, "Well-formed equivalent list decode mismatch");

    // 2. Malformed equivalent list: missing separator Dc 312 between elements
    // 397 [ 275 formatted_path formatted_path 311 ]
    let mut malformed = DcMstring::new();
    malformed.push_char(DC_EQUIVALENT_REPRESENTATIONS); // 397
    malformed.push_char(DC_BEGIN_LIST); // 310
    malformed.push_char(DcChar::from_short(275)); // 275
    path.encode_dc_mixed(&mut malformed)?;
    // Missing DC_LIST_ELEMENT_SEPARATOR (312)!
    path.encode_dc_mixed(&mut malformed)?;
    malformed.push_char(DC_END_LIST); // 311

    // Under strict policy, missing separator MUST error!
    let res_strict = malformed.decode::<PathBuf>(&strict);
    ensure!(
        res_strict.is_err(),
        "Strict policy MUST reject equivalent list with missing element separator (Dc 312), but succeeded"
    );

    // Under forgiving policy, it should recover the first representation
    let dec_forgiving: PathBuf = malformed.decode(&forgiving)?;
    ensure!(
        dec_forgiving == path,
        "Forgiving policy failed to recover path from malformed equivalent list"
    );

    Ok(())
}

