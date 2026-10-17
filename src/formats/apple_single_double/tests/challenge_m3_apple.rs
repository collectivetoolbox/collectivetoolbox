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

//! Empirical Challenge Test Suite for Milestone 3 (AppleSingle/AppleDouble):
//! 1. Stress-test `read_apple_single_double` with truncated, corrupt headers,
//!    overflowing entry descriptors, out-of-bound slices, and corrupt xattrs.
//! 2. Stress-test `FinderInfo` and `ExtendedFinderInfo` DcMixed projection under
//!    strict and forgiving policies with malformed Dc structures, missing fields,
//!    untyped equivalent lists, and invalid label indices.
//! 3. Generative fuzzing harness to ensure error propagation and zero panics.



#[expect(unused_imports, clippy::wildcard_imports, reason = "Integration test root")]
use ctb_utilities::*;

use ctb_formats_apple_single_double::{
    read_apple_single_double, AppleArchive, AppleFormat, ExtendedFinderInfo,
    ExtendedFlags, FinderFlags, FinderInfo, FinderLabel, APPLEDOUBLE_MAGIC_BE,
    APPLEDOUBLE_MAGIC_LE, APPLESINGLE_MAGIC_BE, APPLESINGLE_MAGIC_LE,
};
use ctb_formats_dcdata::dc::*;
use ctb_formats_dcstring::{
    DcChar, DcMixedDecode, DcMixedEncode, DcMstring, DcProjectionPolicy,
};

// ============================================================================
// 1. BINARY PARSER STRESS TESTING: CORRUPT HEADERS & OVERFLOWING DESCRIPTORS
// ============================================================================

#[crate::ctb_test]
fn test_challenge_binary_truncated_headers() -> Result<()> {
    // Empty buffer
    ensure!(read_apple_single_double(&[]).is_err(), "Empty buffer must error");

    // Sub-header sizes (1..26 bytes)
    for len in 1..26 {
        let buf = vec![0u8; len];
        ensure!(
            read_apple_single_double(&buf).is_err(),
            "Buffer of length {len} must error"
        );
    }
    Ok(())
}

#[crate::ctb_test]
fn test_challenge_binary_corrupt_magic() -> Result<()> {
    // 26 bytes with invalid magic
    let mut buf = vec![0u8; 26];
    for magic in [0x00000000u32, 0xdeadbeef, 0x00051601, 0x00051608, 0xffffffff] {
        buf[0..4].copy_from_slice(&magic.to_be_bytes());
        ensure!(
            read_apple_single_double(&buf).is_err(),
            "Magic {magic:#010x} must be rejected"
        );
    }
    Ok(())
}

#[crate::ctb_test]
fn test_challenge_binary_descriptor_boundary_and_overflow() -> Result<()> {
    // Create valid 26-byte header with APPLESINGLE_MAGIC_BE, version 2.0
    let mut header = vec![0u8; 26];
    header[0..4].copy_from_slice(&APPLESINGLE_MAGIC_BE.to_be_bytes());
    header[4..8].copy_from_slice(&0x00020000u32.to_be_bytes());

    // 1. Declared 1 entry, but buffer is only 26 bytes (descriptor needs 12 bytes: total 38)
    header[24..26].copy_from_slice(&1u16.to_be_bytes());
    ensure!(
        read_apple_single_double(&header).is_err(),
        "Truncated descriptor must error"
    );

    // 2. Buffer has 38 bytes, but entry descriptor has offset beyond EOF
    let mut with_desc = header.clone();
    with_desc.extend_from_slice(&1u32.to_be_bytes()); // entry ID 1 (data fork)
    with_desc.extend_from_slice(&1000u32.to_be_bytes()); // offset 1000
    with_desc.extend_from_slice(&50u32.to_be_bytes()); // length 50
    ensure!(
        read_apple_single_double(&with_desc).is_err(),
        "Offset out of bounds must error"
    );

    // 3. Entry descriptor offset + length arithmetic overflow
    let mut overflow_desc = header.clone();
    overflow_desc.extend_from_slice(&1u32.to_be_bytes()); // entry ID 1
    overflow_desc.extend_from_slice(&u32::MAX.to_be_bytes()); // offset u32::MAX
    overflow_desc.extend_from_slice(&100u32.to_be_bytes()); // length 100
    ensure!(
        read_apple_single_double(&overflow_desc).is_err(),
        "Offset + length overflow must error without panic"
    );

    // 4. Declared 0xffff entries on small buffer
    let mut huge_entries = header.clone();
    huge_entries[24..26].copy_from_slice(&0xffffu16.to_be_bytes());
    huge_entries.extend_from_slice(&[0u8; 100]);
    ensure!(
        read_apple_single_double(&huge_entries).is_err(),
        "Huge entry count must error"
    );

    Ok(())
}

#[crate::ctb_test]
fn test_challenge_binary_corrupt_extended_attributes() -> Result<()> {
    // Build an archive with entry ID 9 (FinderInfo / ATTR)
    let mut buf = vec![0u8; 26];
    buf[0..4].copy_from_slice(&APPLEDOUBLE_MAGIC_BE.to_be_bytes());
    buf[4..8].copy_from_slice(&0x00020000u32.to_be_bytes());
    buf[24..26].copy_from_slice(&1u16.to_be_bytes()); // 1 entry

    let entry_offset = 38u32;
    let entry_len = 50u32;
    buf.extend_from_slice(&9u32.to_be_bytes()); // ID 9: Finder info / xattrs
    buf.extend_from_slice(&entry_offset.to_be_bytes());
    buf.extend_from_slice(&entry_len.to_be_bytes());

    // 1. ATTR header truncated (< 36 bytes)
    let mut short_attr = buf.clone();
    short_attr.extend_from_slice(&[0u8; 20]);
    // It should not panic when reading xattrs; either errors or ignores corrupt xattrs
    let _ = read_apple_single_double(&short_attr);

    // 2. ATTR header with corrupt magic (not b"ATTR")
    let mut bad_magic_attr = buf.clone();
    bad_magic_attr.extend_from_slice(b"NOPE");
    bad_magic_attr.extend_from_slice(&[0u8; 46]);
    let _ = read_apple_single_double(&bad_magic_attr);

    // 3. ATTR header with b"ATTR" magic but invalid count/offsets
    let mut corrupt_offsets = buf.clone();
    corrupt_offsets.extend_from_slice(b"ATTR");
    corrupt_offsets.extend_from_slice(&[0u8; 10]);
    corrupt_offsets.extend_from_slice(&0xffffu16.to_be_bytes()); // num_attrs = 65535
    corrupt_offsets.extend_from_slice(&[0xffu8; 34]);
    let _ = read_apple_single_double(&corrupt_offsets);

    Ok(())
}

#[crate::ctb_test]
fn test_challenge_binary_generative_fuzzing() -> Result<()> {
    // Valid minimal AppleDouble binary structure:
    let mut valid = vec![0u8; 26];
    valid[0..4].copy_from_slice(&APPLEDOUBLE_MAGIC_BE.to_be_bytes());
    valid[4..8].copy_from_slice(&0x00020000u32.to_be_bytes());
    valid[24..26].copy_from_slice(&1u16.to_be_bytes()); // 1 entry
    // Entry 1: RealName at offset 38, length 4 ("test")
    valid.extend_from_slice(&3u32.to_be_bytes()); // ID 3 = RealName
    valid.extend_from_slice(&38u32.to_be_bytes());
    valid.extend_from_slice(&4u32.to_be_bytes());
    valid.extend_from_slice(b"test");

    ensure!(read_apple_single_double(&valid).is_ok());

    // Deterministic mutation fuzzing: 2000 variations
    let mut seed = 0x12345678u64;
    let lcg = |s: &mut u64| -> u64 {
        *s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
        *s
    };

    for _ in 0..2000 {
        let mut mutated = valid.clone();
        let r = lcg(&mut seed);
        let mutation_type = r.checked_rem(4).unwrap_or(0);
        let len_u64 = u64::try_from(mutated.len()).unwrap_or(1);

        match mutation_type {
            0 => {
                // Truncation
                let cut = usize::try_from(r.checked_rem(len_u64).unwrap_or(0)).unwrap_or(0);
                mutated.truncate(cut);
            }
            1 => {
                // Bit flips
                let pos = usize::try_from(r.checked_rem(len_u64).unwrap_or(0)).unwrap_or(0);
                let bit = (r >> 8).checked_rem(8).unwrap_or(0);
                if let Some(byte) = mutated.get_mut(pos) {
                    *byte ^= 1 << bit;
                }
            }
            2 => {
                // Byte corruption
                let pos = usize::try_from(r.checked_rem(len_u64).unwrap_or(0)).unwrap_or(0);
                let val = u8::try_from((r >> 16) & 0xff).unwrap_or(0);
                if let Some(byte) = mutated.get_mut(pos) {
                    *byte = val;
                }
            }
            _ => {
                // Random buffer append / expansion
                let extra_len = usize::try_from((r >> 24).checked_rem(100).unwrap_or(0)).unwrap_or(0);
                mutated.extend(vec![0xaa; extra_len]);
            }
        }

        // Must never panic on any input
        let res = read_apple_single_double(&mutated);
        let _ = res;
    }
    Ok(())
}

// ============================================================================
// 2. FINDERINFO & EXTENDEDFINDERINFO DCMIXED PROJECTION STRESS
// ============================================================================

#[crate::ctb_test]
fn test_challenge_finder_info_strict_vs_forgiving() -> Result<()> {
    let strict = DcProjectionPolicy::strict();
    let forgiving = DcProjectionPolicy::forgiving();

    // 1. Valid default FinderInfo roundtrip
    let finfo = FinderInfo {
        file_type: "TEXT".to_string(),
        file_creator: "ttxt".to_string(),
        label: FinderLabel::Hot,
        flags: FinderFlags {
            is_invisible: true,
            has_bundle: true,
            ..FinderFlags::default()
        },
        location: (120, 240),
        folder_id: 10,
        extended: Some(ExtendedFinderInfo {
            icon_id: 128,
            script: 1,
            xflags: ExtendedFlags {
                custom_badge: true,
                routing_info: false,
                extended_flags_invalid: false,
            },
            comment: 42,
            put_away: 1001,
            scroll_position: None,
        }),
        window_bounds: Some([10, 20, 300, 400]),
    };

    let mut mst = DcMstring::new();
    finfo.encode_dc_mixed(&mut mst)?;

    let decoded_strict: FinderInfo = mst.decode(&strict)?;
    ensure!(decoded_strict == finfo, "Strict decode failed to match original");

    let decoded_forgiving: FinderInfo = mst.decode(&forgiving)?;
    ensure!(decoded_forgiving == finfo, "Forgiving decode failed to match original");

    // 2. Corrupt: missing container closer (401 without 402)
    let mut unclosed = DcMstring::new();
    unclosed.push_char(DcChar::from_short(401));
    unclosed.push_char(DcChar::from_short(403));
    "TEXT".encode_dc_mixed(&mut unclosed)?;
    ensure!(
        unclosed.decode::<FinderInfo>(&strict).is_err(),
        "Unclosed 401 container must fail strict decoding"
    );

    // 3. Corrupt: bare closer without opener (402)
    let mut stray_closer = DcMstring::new();
    stray_closer.push_char(DcChar::from_short(402));
    ensure!(
        stray_closer.decode::<FinderInfo>(&strict).is_err(),
        "Stray closer must fail strict decoding"
    );

    // 4. Corrupt: invalid label index (> 7)
    let mut bad_label = DcMstring::new();
    bad_label.push_char(DcChar::from_short(401));
    bad_label.push_char(DcChar::from_short(405));
    99u8.encode_dc_mixed(&mut bad_label)?;
    bad_label.push_char(DcChar::from_short(402));
    ensure!(
        bad_label.decode::<FinderInfo>(&strict).is_err(),
        "Label index 99 must fail strict decoding"
    );

    // 5. Corrupt: type mismatch (string where flags number expected)
    let mut type_mismatch = DcMstring::new();
    type_mismatch.push_char(DcChar::from_short(401));
    type_mismatch.push_char(DcChar::from_short(406));
    "not_a_number".encode_dc_mixed(&mut type_mismatch)?;
    type_mismatch.push_char(DcChar::from_short(402));
    ensure!(
        type_mismatch.decode::<FinderInfo>(&strict).is_err(),
        "String in numeric flags must fail strict decoding"
    );

    Ok(())
}

#[crate::ctb_test]
fn test_challenge_finder_info_boundary_values() -> Result<()> {
    let strict = DcProjectionPolicy::strict();

    // Extreme location coordinates (i16::MIN, i16::MAX)
    let finfo = FinderInfo {
        location: (i16::MIN, i16::MAX),
        folder_id: i16::MIN,
        window_bounds: Some([i16::MIN, i16::MIN, i16::MAX, i16::MAX]),
        ..FinderInfo::default()
    };
    let mut mst = DcMstring::new();
    finfo.encode_dc_mixed(&mut mst)?;
    let decoded: FinderInfo = mst.decode(&strict)?;
    ensure!(decoded == finfo, "Extreme coordinates roundtrip mismatch");

    // All FinderLabel variants (0..=7)
    for idx in 0..=7 {
        let label = FinderLabel::try_from(idx)?;
        let mut mst = DcMstring::new();
        label.encode_dc_mixed(&mut mst)?;
        let decoded: FinderLabel = mst.decode(&strict)?;
        ensure!(decoded == label, "FinderLabel {idx} roundtrip mismatch");
    }

    Ok(())
}

#[crate::ctb_test]
fn test_challenge_extended_finder_info_and_scroll_roundtrip() -> Result<()> {
    let strict = DcProjectionPolicy::strict();

    // 1. Default ExtendedFinderInfo roundtrip (Dc 490 491)
    let default_ext = ExtendedFinderInfo::default();
    ctb_formats_dcstring::assert_dc_document_roundtrip(&default_ext)?;
    let default_finfo = FinderInfo {
        extended: Some(default_ext),
        ..FinderInfo::default()
    };
    ctb_formats_dcstring::assert_dc_document_roundtrip(&default_finfo)?;

    // 2. ExtendedFinderInfo with custom xflags (Dc 562) roundtrip
    let mut ext_with_flags = ExtendedFinderInfo::default();
    ext_with_flags.xflags.custom_badge = true;
    ext_with_flags.xflags.routing_info = true;
    ctb_formats_dcstring::assert_dc_document_roundtrip(&ext_with_flags)?;

    let make_archive = |finfo: FinderInfo| AppleArchive {
        format: AppleFormat::AppleDouble,
        version: 0x00020000,
        real_name: None,
        comment: None,
        timestamps: None,
        backup_timestamp_sec: None,
        finder_info: Some(finfo),
        extended_attributes: vec![],
        unrecognized_entries: vec![],
        data_fork: None,
        resource_fork: None,
        data_fork_size: None,
        resource_fork_size: None,
        entries: vec![],
    };

    // 3. AppleSingle binary roundtrip with folder frScroll (including 0, 0)
    let mut finfo_scroll = FinderInfo {
        window_bounds: Some([50, 50, 400, 600]),
        ..FinderInfo::default()
    };
    finfo_scroll.extended = Some(ExtendedFinderInfo {
        icon_id: 0,
        scroll_position: Some((150, 300)),
        ..ExtendedFinderInfo::default()
    });
    let archive_scroll = make_archive(finfo_scroll);
    let written = ctb_formats_apple_single_double::write_apple_single_double(&archive_scroll)?;
    let parsed = ctb_formats_apple_single_double::read_apple_single_double(&written)?;
    let parsed_ext = parsed.finder_info.as_ref().and_then(|f| f.extended).context("Missing parsed extended info")?;
    ensure!(parsed_ext.scroll_position == Some((150, 300)));
    ensure!(parsed_ext.icon_id == 0);

    // 3b. Folder with scroll_position (0, 0) - must preserve (0, 0) and not drop to None
    let mut finfo_scroll_zero = FinderInfo {
        window_bounds: Some([50, 50, 400, 600]),
        ..FinderInfo::default()
    };
    finfo_scroll_zero.extended = Some(ExtendedFinderInfo {
        icon_id: 0,
        scroll_position: Some((0, 0)),
        ..ExtendedFinderInfo::default()
    });
    let written_zero = ctb_formats_apple_single_double::write_apple_single_double(&make_archive(finfo_scroll_zero))?;
    let parsed_zero = ctb_formats_apple_single_double::read_apple_single_double(&written_zero)?;
    let parsed_ext_zero = parsed_zero.finder_info.as_ref().and_then(|f| f.extended).context("Missing zero scroll extended info")?;
    ensure!(parsed_ext_zero.scroll_position == Some((0, 0)), "Folder scroll_position (0, 0) must be preserved");
    ensure!(parsed_ext_zero.icon_id == 0);

    // 4. AppleSingle binary roundtrip with custom icon_id (file)
    let mut finfo_icon = FinderInfo {
        file_type: "TEXT".to_string(),
        file_creator: "ttxt".to_string(),
        ..FinderInfo::default()
    };
    finfo_icon.extended = Some(ExtendedFinderInfo {
        icon_id: -16455,
        scroll_position: None,
        ..ExtendedFinderInfo::default()
    });
    let archive_icon = make_archive(finfo_icon);
    let mut written_icon = ctb_formats_apple_single_double::write_apple_single_double(&archive_icon)?;
    // Inject dirty non-zero bytes in file's reserved bytes 18..19 (bytes 18..20 of FinderInfo)
    // to test that a file with non-zero reserved bytes is NOT misclassified as having scroll_position
    if let Some(pos) = written_icon.windows(4).position(|w| w == b"TEXT") {
        if let Some(slot) = written_icon.get_mut(pos + 18..pos + 20) {
            slot.copy_from_slice(&[0x12, 0x34]);
        }
    }
    let parsed_icon = ctb_formats_apple_single_double::read_apple_single_double(&written_icon)?;
    let parsed_ext_icon = parsed_icon.finder_info.as_ref().and_then(|f| f.extended).context("Missing parsed extended info")?;
    ensure!(parsed_ext_icon.icon_id == -16455);
    ensure!(parsed_ext_icon.scroll_position.is_none(), "File with non-zero reserved bytes must not emit scroll_position");

    Ok(())
}
