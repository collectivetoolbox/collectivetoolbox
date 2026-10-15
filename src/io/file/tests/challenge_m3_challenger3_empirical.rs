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

//! Empirical Challenge Test Suite for Milestone 3 Remediation:
//! 1. Equivalent list decoding and strict Dc 312 separation verification.
//! 2. Standalone FileTimestamps roundtrip stress testing across varied bounds.

#[expect(unused_imports, clippy::wildcard_imports, reason = "Integration test root")]
pub use ctb_utilities::*;

use anyhow::{Context, Result, ensure};
use ctb_formats_apple_single_double::{FinderFlags, FinderInfo, FinderLabel};
use ctb_formats_dcstring::{
    DcChar, DcDocumentNavigator, DcMixedDecode, DcMixedEncode, DcMstring,
    DcProjectionContext, DcProjectionPolicy, DcSyntaxArena, assert_dc_roundtrip,
};
use ctb_io_file::metadata::FileTimestamps;

// ============================================================================
// 1. EQUIVALENT LIST DECODING & STRICT SEPARATION EMPIRICAL CHALLENGES
// ============================================================================

/// Empirically verifies that equivalent representation lists require strict
/// Dc 312 element separation under strict policy and fail when missing.
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
    eprintln!("unwrap_equivalent_or_self on 397 [ 275 ] result: {unwrap_res:?}");

    let vec_res = bare_empty_equiv.decode::<Vec<u32>>(&strict);
    eprintln!("bare_empty_equiv.decode::<Vec<u32>> result: {vec_res:?}");

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
