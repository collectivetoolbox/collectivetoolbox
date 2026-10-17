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

//! Empirical Challenge Test Suite for Milestone 2:
//! 1. Stress-test `DcMixedDecode` projection over complex composite structures:
//!    deeply nested optionals, nested tuples, `BTreeMap`, arrays `[T; N]`, and
//!    `Duration` with short (`393..394`) and long (`395..396`) delimiters.
//! 2. Verify strict vs forgiving policy: verify that `strict()` rejects bare strings
//!    and unformatted paths, while `forgiving()` recovers them safely.
//! 3. Verify that zero base64 allocations occur during projection of raw binary payloads.

#[expect(unused_imports, clippy::wildcard_imports, reason = "Integration test root")]
use ctb_utilities::*;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

pub use ctb_formats_dcstring::*;
use ctb_formats_dcdata::dc::*;

// ============================================================================
// 1. COMPLEX COMPOSITE STRUCTURES: NESTED OPTIONALS
// ============================================================================

#[crate::ctb_test]
fn test_challenge_deeply_nested_optionals() -> Result<()> {
    let strict = DcProjectionPolicy::strict();

    // 1. None
    let opt_none: Option<Option<Option<u64>>> = None;
    let mut mst = DcMstring::new();
    opt_none.encode_dc_mixed(&mut mst)?;
    let decoded = mst.decode::<Option<Option<Option<u64>>>>(&strict)?;
    ensure!(decoded == opt_none, "None failed: got {decoded:?}");

    // 2. Some(None)
    let opt_some_none: Option<Option<Option<u64>>> = Some(None);
    let mut mst = DcMstring::new();
    opt_some_none.encode_dc_mixed(&mut mst)?;
    let decoded = mst.decode::<Option<Option<Option<u64>>>>(&strict)?;
    ensure!(decoded == opt_some_none, "Some(None) failed: got {decoded:?}");

    // 3. Some(Some(None))
    let opt_some_some_none: Option<Option<Option<u64>>> = Some(Some(None));
    let mut mst = DcMstring::new();
    opt_some_some_none.encode_dc_mixed(&mut mst)?;
    let decoded = mst.decode::<Option<Option<Option<u64>>>>(&strict)?;
    ensure!(decoded == opt_some_some_none, "Some(Some(None)) failed: got {decoded:?}");

    // 4. Some(Some(Some(42)))
    let opt_some_some_val: Option<Option<Option<u64>>> = Some(Some(Some(42)));
    let mut mst = DcMstring::new();
    opt_some_some_val.encode_dc_mixed(&mut mst)?;
    let decoded = mst.decode::<Option<Option<Option<u64>>>>(&strict)?;
    ensure!(decoded == opt_some_some_val, "Some(Some(Some(42))) failed: got {decoded:?}");

    // 5. 5-level nested optional
    type DeepOpt = Option<Option<Option<Option<Option<String>>>>>;
    let deep_val: DeepOpt = Some(Some(Some(Some(Some("deep_value".to_string())))));
    let mut mst = DcMstring::new();
    deep_val.encode_dc_mixed(&mut mst)?;
    let decoded = mst.decode::<DeepOpt>(&strict)?;
    ensure!(decoded == deep_val, "DeepOpt value mismatch: got {decoded:?}");

    let deep_none: DeepOpt = Some(Some(Some(Some(None))));
    let mut mst = DcMstring::new();
    deep_none.encode_dc_mixed(&mut mst)?;
    let decoded = mst.decode::<DeepOpt>(&strict)?;
    ensure!(decoded == deep_none, "DeepOpt None mismatch: got {decoded:?}");

    Ok(())
}

// ============================================================================
// 2. COMPLEX COMPOSITE STRUCTURES: TUPLES AND NESTED TUPLES
// ============================================================================

#[crate::ctb_test]
fn test_challenge_converted_document_preserves_consumed_roots() -> Result<()> {
    #[derive(Debug, Default, PartialEq, DcMixed)]
    #[dc(try_from = (u64, u64), try_into = (u64, u64))]
    struct ConvertedPair {
        first: u64,
        second: u64,
    }

    impl From<(u64, u64)> for ConvertedPair {
        fn from((first, second): (u64, u64)) -> Self {
            Self { first, second }
        }
    }

    impl From<&ConvertedPair> for (u64, u64) {
        fn from(value: &ConvertedPair) -> Self {
            (value.first, value.second)
        }
    }

    let strict = DcProjectionPolicy::strict();
    let mut source = DcMstring::new();
    12_u64.encode_dc_mixed(&mut source)?;
    34_u64.encode_dc_mixed(&mut source)?;
    let expected = ConvertedPair { first: 12, second: 34 };
    ensure!(source.decode::<ConvertedPair>(&strict)? == expected);
    true.encode_dc_mixed(&mut source)?;

    let mut nav = DcDocumentNavigator::from_mstr(&source);
    let mut arena = DcSyntaxArena::new();
    nav.parse_document_into_arena(&mut arena, &strict)?;
    let mut ctx = DcProjectionContext::new(&arena, &strict);
    let (decoded, consumed) = ConvertedPair::decode_dc_document(&mut ctx, arena.roots())?;
    ensure!(decoded == expected);
    ensure!(consumed == 2, "Conversion must preserve the representation's root count");
    ensure!(source.decode::<ConvertedPair>(&strict).is_err(), "Trailing roots must not be swallowed");
    Ok(())
}

#[crate::ctb_test]
fn test_challenge_field_conversion_errors_and_projection() -> Result<()> {
    #[derive(Debug, Default, PartialEq, DcMixed)]
    #[dc(try_from = i64, try_into = i64)]
    struct CheckedNumber {
        value: i64,
    }

    impl TryFrom<i64> for CheckedNumber {
        type Error = anyhow::Error;
        fn try_from(value: i64) -> Result<Self> {
            ensure!(value >= 0, "Negative representation");
            Ok(Self { value })
        }
    }

    impl TryFrom<&i64> for CheckedNumber {
        type Error = anyhow::Error;
        fn try_from(value: &i64) -> Result<Self> {
            Self::try_from(*value)
        }
    }

    impl From<&CheckedNumber> for i64 {
        fn from(value: &CheckedNumber) -> Self { value.value }
    }

    impl TryFrom<CheckedNumber> for i64 {
        type Error = anyhow::Error;
        fn try_from(value: CheckedNumber) -> Result<Self> {
            ensure!(value.value != 13, "Rejected decoded value");
            Ok(value.value)
        }
    }

    #[derive(Debug, Default, PartialEq, DcMixed)]
    #[dc(begin = 317, end = 318)]
    struct Bounded {
        #[dc(327, via = CheckedNumber, omit_default)]
        value: Option<i64>,
    }

    #[derive(Debug, Default, PartialEq, DcMixed)]
    #[dc(flatten)]
    struct Flattened {
        #[dc(327, via = CheckedNumber)]
        value: i64,
    }

    let strict = DcProjectionPolicy::strict();
    assert_dc_roundtrip(&Bounded { value: Some(42) })?;
    assert_dc_roundtrip(&Flattened { value: 42 })?;
    let mut invalid = DcMstring::new();
    ensure!(Bounded { value: Some(-1) }.encode_dc_mixed(&mut invalid).is_err());
    for value in [-1_i64, 13] {
        let mut source = DcMstring::new();
        source.push_char(DcChar::from_short(327));
        value.encode_dc_mixed(&mut source)?;
        ensure!(source.decode::<Flattened>(&strict).is_err());
    }
    let mut source = DcMstring::new();
    source.push_char(DcChar::from_short(317));
    Bounded { value: Some(42) }.encode_dc_field("value", &mut source)?;
    source.push_char(DcChar::from_short(318));
    ensure!(source.decode::<Bounded>(&strict)? == Bounded { value: Some(42) });
    Ok(())
}

#[crate::ctb_test]
fn test_challenge_nested_tuples_framed() -> Result<()> {
    let strict = DcProjectionPolicy::strict();

    // 1. Single framed pair container: Dc 514 ... 312 ... 515
    let mut framed_pair_mst = DcMstring::new();
    framed_pair_mst.push_char(DC_BEGIN_KV_PAIR);
    100_u32.encode_dc_mixed(&mut framed_pair_mst)?;
    framed_pair_mst.push_char(DC_LIST_ELEMENT_SEPARATOR);
    "hello".to_string().encode_dc_mixed(&mut framed_pair_mst)?;
    framed_pair_mst.push_char(DC_END_KV_PAIR);

    let decoded_pair = framed_pair_mst.decode::<(u32, String)>(&strict)
        .context("Framed pair Dc 514..515 failed to decode into (u32, String)")?;
    ensure!(decoded_pair == (100, "hello".to_string()), "Framed pair mismatch");

    // 2. Manually framed nested pair of pairs: ((u32, String), (bool, u64))
    let mut nested_framed_mst = DcMstring::new();
    nested_framed_mst.push_char(DC_BEGIN_KV_PAIR);
    // Inner pair 0
    nested_framed_mst.push_char(DC_BEGIN_KV_PAIR);
    100_u32.encode_dc_mixed(&mut nested_framed_mst)?;
    nested_framed_mst.push_char(DC_LIST_ELEMENT_SEPARATOR);
    "inner_left".to_string().encode_dc_mixed(&mut nested_framed_mst)?;
    nested_framed_mst.push_char(DC_END_KV_PAIR);

    nested_framed_mst.push_char(DC_LIST_ELEMENT_SEPARATOR);

    // Inner pair 1
    nested_framed_mst.push_char(DC_BEGIN_KV_PAIR);
    true.encode_dc_mixed(&mut nested_framed_mst)?;
    nested_framed_mst.push_char(DC_LIST_ELEMENT_SEPARATOR);
    999_u64.encode_dc_mixed(&mut nested_framed_mst)?;
    nested_framed_mst.push_char(DC_END_KV_PAIR);

    nested_framed_mst.push_char(DC_END_KV_PAIR);

    let decoded_nested = nested_framed_mst.decode::<((u32, String), (bool, u64))>(&strict)
        .context("Manually framed nested pair of pairs failed to decode")?;
    ensure!(
        decoded_nested == ((100, "inner_left".to_string()), (true, 999)),
        "Decoded nested pair mismatch"
    );

    Ok(())
}

/// BUG 1 EMPIRICAL REPRODUCTION:
/// `(A, B)::encode_dc_mixed` emits uncontained flat tokens without `DC_BEGIN_KV_PAIR` (514)
/// or `DC_END_KV_PAIR` (515) framing, but `(A, B)::decode_dc_node` requires a `Container`
/// or `Tagged` node with 2 args. Consequently, any attempt to roundtrip `(A, B)` via trait
/// methods fails.
#[crate::ctb_test]
fn test_challenge_bug_tuple_encode_decode_roundtrip_asymmetry() -> Result<()> {
    let strict = DcProjectionPolicy::strict();

    let tuple_val = (100_u32, "hello".to_string());
    let mut mst = DcMstring::new();
    tuple_val.encode_dc_mixed(&mut mst)?;

    let decoded = mst.decode::<(u32, String)>(&strict)
        .context("Multi-root (A, B) tuple failed to decode")?;
    ensure!(decoded == tuple_val, "Tuple roundtrip mismatch: got {decoded:?}");

    Ok(())
}

// ============================================================================
// 3. COMPLEX COMPOSITE STRUCTURES: BTREEMAP
// ============================================================================

#[crate::ctb_test]
fn test_challenge_btreemap_nested_and_legacy() -> Result<()> {
    let strict = DcProjectionPolicy::strict();
    let forgiving = DcProjectionPolicy::forgiving();

    // 1. Standard BTreeMap with nested values
    let mut map: BTreeMap<String, Option<u64>> = BTreeMap::new();
    map.insert("alpha".to_string(), Some(10));
    map.insert("beta".to_string(), None);
    map.insert("gamma".to_string(), Some(30));

    let mut mst = DcMstring::new();
    map.encode_dc_mixed(&mut mst)?;
    let decoded = mst.decode::<BTreeMap<String, Option<u64>>>(&strict)?;
    ensure!(decoded == map, "BTreeMap<String, Option<u64>> mismatch");

    // 2. Nested BTreeMap: BTreeMap<String, BTreeMap<String, u32>>
    let mut outer: BTreeMap<String, BTreeMap<String, u32>> = BTreeMap::new();
    let mut inner1 = BTreeMap::new();
    inner1.insert("x".to_string(), 1);
    inner1.insert("y".to_string(), 2);
    let mut inner2 = BTreeMap::new();
    inner2.insert("z".to_string(), 3);
    outer.insert("group1".to_string(), inner1);
    outer.insert("group2".to_string(), inner2);

    let mut mst = DcMstring::new();
    outer.encode_dc_mixed(&mut mst)?;
    let decoded = mst.decode::<BTreeMap<String, BTreeMap<String, u32>>>(&strict)?;
    ensure!(decoded == outer, "Nested BTreeMap mismatch");

    // 3. Legacy Dc 313..314 map format under forgiving policy
    let mut legacy_mst = DcMstring::new();
    legacy_mst.push_char(DcChar::from_short(313)); // Begin legacy map
    legacy_mst.push_char(DcChar::from_short(275)); // Object type indicator
    legacy_mst.push_char(DcChar::from_short(275)); // Object type indicator
    "key1".to_string().encode_dc_mixed(&mut legacy_mst)?;
    legacy_mst.push_char(DcChar::from_short(312)); // Separator
    100_u32.encode_dc_mixed(&mut legacy_mst)?;
    legacy_mst.push_char(DcChar::from_short(312)); // Separator
    "key2".to_string().encode_dc_mixed(&mut legacy_mst)?;
    legacy_mst.push_char(DcChar::from_short(312)); // Separator
    200_u32.encode_dc_mixed(&mut legacy_mst)?;
    legacy_mst.push_char(DcChar::from_short(314)); // End legacy map

    let decoded_legacy = legacy_mst.decode::<BTreeMap<String, u32>>(&forgiving)
        .context("Legacy Dc 313 map must decode under forgiving policy")?;
    ensure!(decoded_legacy.len() == 2, "Legacy map should contain 2 entries");
    ensure!(decoded_legacy.get("key1") == Some(&100));
    ensure!(decoded_legacy.get("key2") == Some(&200));

    Ok(())
}

// ============================================================================
// 4. COMPLEX COMPOSITE STRUCTURES: ARRAYS [T; N]
// ============================================================================

#[derive(Debug, PartialEq, Eq, DcMixed)]
#[dc(begin = 401, end = 402)]
struct StructWithArray {
    #[dc(short = 403)]
    file_type: String,
    #[dc(407, representation = f523)]
    icon_location: [i16; 2],
}

#[derive(Debug, PartialEq, Eq, DcMixed)]
#[dc(flatten)]
struct FlattenedProjectionRecord {
    #[dc(405)]
    label_color: u32,
    #[dc(400)]
    comment: String,
}

#[derive(Debug, PartialEq, Eq, DcMixed)]
#[dc(begin = 401, end = 402)]
struct TemporalProjectionRecord {
    #[dc(387)]
    inspected_at: SystemTime,
    #[dc(400)]
    comment: String,
}

#[derive(Debug, PartialEq, Eq, DcMixed)]
#[dc(flatten)]
struct AdjacentDateFields {
    #[dc(330)]
    atime: SystemTime,
    #[dc(329)]
    mtime: SystemTime,
}

#[crate::ctb_test]
fn test_adjacent_date_fields_preserve_prefix_boundaries() -> Result<()> {
    let value = AdjacentDateFields {
        atime: SystemTime::UNIX_EPOCH.checked_add(Duration::new(1_700_000_000, 500)).context("Time overflow")?,
        mtime: SystemTime::UNIX_EPOCH.checked_add(Duration::new(1_700_000_001, 600)).context("Time overflow")?,
    };
    let encoded = value.to_dc_mixed()?;
    let strict = DcProjectionPolicy::strict();
    let mut nav = DcDocumentNavigator::from_mstr(&encoded);
    let mut arena = DcSyntaxArena::new();
    nav.parse_document_into_arena(&mut arena, &strict)?;
    ensure!(arena.roots().len() == 2, "Date fields must remain sibling roots: {:?}", arena.roots());
    ensure!(encoded.decode::<AdjacentDateFields>(&strict)? == value);
    Ok(())
}

#[crate::ctb_test]
fn test_strict_decode_requires_list_type_header() -> Result<()> {
    let strict = DcProjectionPolicy::strict();
    let mut empty = DcMstring::new();
    empty.push_char(DC_BEGIN_LIST);
    empty.push_char(DC_END_LIST);
    ensure!(validate_dc_mixed_strict(&empty).is_err());
    ensure!(empty.decode::<Vec<u32>>(&strict).is_err(), "Strict Vec decode accepted a missing type header");

    let mut array = DcMstring::new();
    array.push_char(DC_BEGIN_LIST);
    42_u32.encode_dc_mixed(&mut array)?;
    array.push_char(DC_END_LIST);
    ensure!(array.decode::<[u32; 1]>(&strict).is_err(), "Strict array decode accepted a missing type header");
    Ok(())
}

#[crate::ctb_test]
fn test_challenge_array_bare_roots() -> Result<()> {
    let strict = DcProjectionPolicy::strict();

    // 1. Bare array roundtrip [u32; 3] across root boundaries
    let arr = [10_u32, 20_u32, 30_u32];
    let mut mst = DcMstring::new();
    arr.encode_dc_mixed(&mut mst)?;
    let decoded = mst.decode::<[u32; 3]>(&strict)?;
    ensure!(decoded == arr, "Bare [u32; 3] roundtrip mismatch");
    ensure!(mst.to_dc_string()?.decode::<[u32; 3]>(&strict)? == arr);
    ensure!(mst.to_dc_string()?.decode_with_context::<[u32; 3]>(&SharedValueContext::default(), &strict)? == arr);

    // 2. Bare array with N = 1
    let arr1 = [999_u64];
    let mut mst = DcMstring::new();
    arr1.encode_dc_mixed(&mut mst)?;
    let decoded = mst.decode::<[u64; 1]>(&strict)?;
    ensure!(decoded == arr1, "Bare [u64; 1] roundtrip mismatch");

    Ok(())
}

/// BUG 2 EMPIRICAL REPRODUCTION:
/// In `[T; N]::decode_dc_node` (src/formats/dcstring/dc_mixed_serde.rs:1888-1910),
/// when `node.kind` is a `Container` (`Dc 310` list or `Dc 511` format representation),
/// the method iterates over `child_slice` and populates `items: Vec<T>`, but the author
/// omitted `return items.try_into().map_err(...)`.
/// Consequently, the populated `items` vector is dropped at the end of the `if let` block,
/// and the function falls through to line 1934 and bails with:
/// `Expected container or N roots for [T; N], found Container ...`.
#[crate::ctb_test]
fn test_challenge_bug_array_container_decoding_dropped_items() -> Result<()> {
    let strict = DcProjectionPolicy::strict();

    // Build a list container: Dc 310, 11, 312, 22, 312, 33, Dc 311
    let mut list_mst = DcMstring::new();
    list_mst.push_char(DC_BEGIN_LIST);
    list_mst.push_char(DC_TYPE_ANY);
    11_u32.encode_dc_mixed(&mut list_mst)?;
    list_mst.push_char(DC_LIST_ELEMENT_SEPARATOR);
    22_u32.encode_dc_mixed(&mut list_mst)?;
    list_mst.push_char(DC_LIST_ELEMENT_SEPARATOR);
    33_u32.encode_dc_mixed(&mut list_mst)?;
    list_mst.push_char(DC_END_LIST);

    let decoded = list_mst.decode::<[u32; 3]>(&strict)
        .context("Array [T; 3] in container failed to decode")?;
    ensure!(decoded == [11, 22, 33], "Decoded array mismatch: got {decoded:?}");

    Ok(())
}

/// Finder icon coordinates use the database's QuickDraw point representation.
#[crate::ctb_test]
fn test_challenge_bug_array_in_struct_field() -> Result<()> {
    let strict = DcProjectionPolicy::strict();

    let item = StructWithArray {
        file_type: "TEXT".to_string(),
        icon_location: [100, 200],
    };
    let mut mst = DcMstring::new();
    item.encode_dc_mixed(&mut mst)?;

    validate_dc_mixed_strict(&mst)?;
    ensure!(mst.decode::<StructWithArray>(&strict)? == item);

    Ok(())
}

#[crate::ctb_test]
fn test_challenge_nested_tuple_and_array_do_not_project_document_roots() -> Result<()> {
    let strict = DcProjectionPolicy::strict();

    let mut mst = DcMstring::new();
    vec![1_u16].encode_dc_mixed(&mut mst)?;
    "outside".to_string().encode_dc_mixed(&mut mst)?;

    ensure!(
        mst.decode::<Vec<(DcRawValue, DcRawValue)>>(&strict).is_err(),
        "Nested tuple projection must not escape to document roots"
    );
    ensure!(
        mst.decode::<Vec<[DcRawValue; 2]>>(&strict).is_err(),
        "Nested array projection must not escape to document roots"
    );

    Ok(())
}

#[crate::ctb_test]
fn test_challenge_tuple_reports_local_leftovers() -> Result<()> {
    let strict = DcProjectionPolicy::strict();
    let forgiving = DcProjectionPolicy::forgiving();
    let mut source = DcMstring::new();
    for value in [1_u32, 2, 3] {
        value.encode_dc_mixed(&mut source)?;
    }
    let mut nav = DcDocumentNavigator::from_mstr(&source);
    let mut arena = DcSyntaxArena::new();
    nav.parse_document_into_arena(&mut arena, &strict)?;
    let children = arena.roots().to_vec();
    let pair = arena.alloc(
        DcByteSpan::new(0, 0),
        DcSyntaxKind::Container {
            opener: DC_BEGIN_KV_PAIR,
            closer: DC_END_KV_PAIR,
            header: Vec::new(),
            children: children.clone(),
            trailing: Vec::new(),
            unclosed: false,
        },
    );
    let tagged = arena.alloc(
        DcByteSpan::new(0, 0),
        DcSyntaxKind::Tagged { tag: DC_BEGIN_KV_PAIR, args: children },
    );
    for node in [pair, tagged] {
        let mut ctx = DcProjectionContext::new(&arena, &strict);
        let error = <(u32, u32)>::decode_dc_node(&mut ctx, node)
            .expect_err("Strict tuple projection must report unused local nodes");
        ensure!(format!("{error:#}").contains("Unconsumed Dc node"));
        let mut ctx = DcProjectionContext::new(&arena, &forgiving);
        ensure!(<(u32, u32)>::decode_dc_node(&mut ctx, node)? == (1, 2));
    }
    Ok(())
}

#[crate::ctb_test]
fn test_challenge_document_sequence_leftovers() -> Result<()> {
    let strict = DcProjectionPolicy::strict();
    let forgiving = DcProjectionPolicy::forgiving();
    let mut array = [1_u32, 2].to_dc_mixed()?;
    3_u32.encode_dc_mixed(&mut array)?;
    ensure!(array.decode::<[u32; 2]>(&strict).is_err());
    ensure!(array.decode::<[u32; 2]>(&forgiving)? == [1, 2]);

    let mut pair = (1_u32, 2_u32).to_dc_mixed()?;
    "outside".to_string().encode_dc_mixed(&mut pair)?;
    ensure!(pair.decode::<(u32, u32)>(&strict).is_err());
    ensure!(pair.decode::<(u32, u32)>(&forgiving)? == (1, 2));

    let mut framed_array = vec![1_u32, 2].to_dc_mixed()?;
    "outside".to_string().encode_dc_mixed(&mut framed_array)?;
    ensure!(framed_array.decode::<[u32; 2]>(&strict).is_err());
    ensure!(framed_array.decode::<[u32; 2]>(&forgiving)? == [1, 2]);
    Ok(())
}

#[crate::ctb_test]
fn test_challenge_flattened_document_projection_boundary() -> Result<()> {
    let strict = DcProjectionPolicy::strict();
    let record = FlattenedProjectionRecord {
        label_color: 7,
        comment: "visible".to_string(),
    };

    let mut encoded = DcMstring::new();
    record.encode_dc_mixed(&mut encoded)?;
    validate_dc_mixed_strict(&encoded)?;
    let decoded = encoded.decode::<FlattenedProjectionRecord>(&strict)?;
    ensure!(decoded == record, "Top-level flattened decode mismatch");
    ensure!(encoded.to_dc_string()?.decode::<FlattenedProjectionRecord>(&strict)? == record);
    ensure!(encoded.to_dc_string()?.decode_with_context::<FlattenedProjectionRecord>(&SharedValueContext::default(), &strict)? == record);

    let mut nested_doc = DcMstring::new();
    vec![1_u16].encode_dc_mixed(&mut nested_doc)?;
    record.encode_dc_mixed(&mut nested_doc)?;

    let mut nav = DcDocumentNavigator::from_mstr(&nested_doc);
    let mut arena = DcSyntaxArena::new();
    nav.parse_document_into_arena(&mut arena, &strict)?;
    let list_root = arena.roots().first().copied().context("Missing list root")?;
    let nested_child_id = match &arena.node(list_root).context("Missing list root")?.kind {
        ctb_formats_dcstring::navigator::DcSyntaxKind::Container { children, .. } => {
            children.last().copied().context("Missing nested list child")?
        }
        other => bail!("Expected list root for nested flattened repro, found {other:?}"),
    };

    let mut ctx = DcProjectionContext::new(&arena, &strict);
    ensure!(
        FlattenedProjectionRecord::decode_dc_node(&mut ctx, nested_child_id).is_err(),
        "Nested flattened decode must not project from document roots"
    );

    Ok(())
}

// ============================================================================
// 5. DURATION WITH SHORT (393..394) AND LONG (395..396) DELIMITERS
// ============================================================================

#[crate::ctb_test]
fn test_challenge_duration_short_and_long_delimiters() -> Result<()> {
    let strict = DcProjectionPolicy::strict();
    let forgiving = DcProjectionPolicy::forgiving();

    let expected = Duration::new(42, 123_456_789);
    let nanos: u128 = 42u128
        .checked_mul(1_000_000_000)
        .context("Mul overflow")?
        .checked_add(123_456_789)
        .context("Add overflow")?;

    // 1. Duration with short delimiters: 393 .. 394 (nanoseconds format 489)
    let mut mst_393 = DcMstring::new();
    mst_393.push_char(DcChar::from_short(393)); // Begin date magnitude
    mst_393.push_char(DcChar::from_format(489)); // Nanoseconds format
    nanos.encode_dc_mixed(&mut mst_393)?;
    mst_393.push_char(DcChar::from_short(394)); // End date magnitude

    let dec_393 = mst_393.decode::<Duration>(&strict)
        .context("Duration in Dc 393..394 container failed to decode")?;
    ensure!(dec_393 == expected, "393..394 mismatch: got {dec_393:?}");

    // 2. Duration with short delimiters: 393 .. 394 (seconds format 488)
    let expected_secs = Duration::from_secs(120);
    let mut mst_393_s = DcMstring::new();
    mst_393_s.push_char(DcChar::from_short(393));
    mst_393_s.push_char(DcChar::from_format(488));
    120_u64.encode_dc_mixed(&mut mst_393_s)?;
    mst_393_s.push_char(DcChar::from_short(394));

    let dec_393_s = mst_393_s.decode::<Duration>(&strict)
        .context("Duration in Dc 393..394 with format 488 failed to decode")?;
    ensure!(dec_393_s == expected_secs, "393..394 seconds mismatch");

    // 3. Duration with long delimiters: 395 .. 396 (nanoseconds format 489)
    let mut mst_395 = DcMstring::new();
    mst_395.push_char(DcChar::from_short(395)); // Begin date duration description
    mst_395.push_char(DcChar::from_format(489));
    nanos.encode_dc_mixed(&mut mst_395)?;
    mst_395.push_char(DcChar::from_short(396)); // End date duration description

    let dec_395 = mst_395.decode::<Duration>(&strict)
        .context("Duration in Dc 395..396 container failed to decode")?;
    ensure!(dec_395 == expected, "395..396 mismatch: got {dec_395:?}");

    // 4. Duration without a format indicator must not guess nanoseconds.
    let mut mst_395_raw = DcMstring::new();
    mst_395_raw.push_char(DcChar::from_short(395));
    nanos.encode_dc_mixed(&mut mst_395_raw)?;
    mst_395_raw.push_char(DcChar::from_short(396));

    ensure!(mst_395_raw.decode::<Duration>(&strict).is_err());
    ensure!(mst_395_raw.decode::<Duration>(&forgiving).is_err());

    // 5. Bare number representation of Duration:
    // Both strict and forgiving policies reject bare number for standalone Duration
    let mut bare_num_mst = DcMstring::new();
    nanos.encode_dc_mixed(&mut bare_num_mst)?;

    ensure!(
        bare_num_mst.decode::<Duration>(&strict).is_err(),
        "Bare number Duration MUST fail under strict policy!"
    );
    ensure!(
        bare_num_mst.decode::<Duration>(&forgiving).is_err(),
        "Standalone Duration decoder MUST reject bare number under forgiving policy too"
    );

    Ok(())
}

#[crate::ctb_test]
fn test_challenge_temporal_codecs_reject_unsupported_formats() -> Result<()> {
    let strict = DcProjectionPolicy::strict();
    let forgiving = DcProjectionPolicy::forgiving();
    let pan_date = DcChar::from_u128(
        ctb_formats_utilities::FormatId::PanDate
            .dc_id()
            .context("Missing PanDate format identifier")?,
    );
    let mut system_time_doc = DcMstring::new();
    system_time_doc.push_char(DC_BEGIN_TIME_MOMENT);
    system_time_doc.push_char(pan_date);
    123_i64.encode_dc_mixed(&mut system_time_doc)?;
    system_time_doc.push_char(DC_END_TIME_MOMENT);

    let system_time_err = system_time_doc
        .decode::<SystemTime>(&strict)
        .expect_err("Unsupported SystemTime format must fail");
    ensure!(
        format!("{system_time_err:#}").contains("Unsupported SystemTime format"),
        "SystemTime error must name unsupported format: {system_time_err:#}"
    );
    ensure!(
        system_time_doc.decode::<SystemTime>(&forgiving).is_err(),
        "Standalone SystemTime decode must reject unsupported formats under forgiving policy too"
    );

    let mut nav = DcDocumentNavigator::from_mstr(&system_time_doc);
    let mut arena = DcSyntaxArena::new();
    nav.parse_document_into_arena(&mut arena, &strict)?;
    let root = arena.roots().first().copied().context("Missing time root")?;
    let children = arena.node(root).context("Missing time node")?.children().to_vec();
    let unsupported_duration = arena.alloc(
        DcByteSpan::new(0, 0),
        DcSyntaxKind::Container {
            opener: DC_BEGIN_DATE_MAGNITUDE,
            closer: DC_END_DATE_MAGNITUDE,
            header: Vec::new(),
            children,
            trailing: Vec::new(),
            unclosed: false,
        },
    );
    for policy in [&strict, &forgiving] {
        let mut ctx = DcProjectionContext::new(&arena, policy);
        let error = Duration::decode_dc_node(&mut ctx, unsupported_duration)
            .expect_err("Unsupported Duration format must fail at projection");
        ensure!(format!("{error:#}").contains("Unsupported Duration format"));
    }

    let mut record_doc = DcMstring::new();
    record_doc.push_char(DcChar::from_short(401));
    record_doc.push_char(DcChar::from_short(387));
    record_doc.push_char(DC_BEGIN_TIME_MOMENT);
    record_doc.push_char(pan_date);
    456_i64.encode_dc_mixed(&mut record_doc)?;
    record_doc.push_char(DC_END_TIME_MOMENT);
    record_doc.push_char(DcChar::from_short(400));
    "kept".to_string().encode_dc_mixed(&mut record_doc)?;
    record_doc.push_char(DcChar::from_short(402));

    ensure!(
        record_doc.decode::<TemporalProjectionRecord>(&strict).is_err(),
        "Strict struct decode must reject unsupported nested SystemTime formats"
    );
    let recovered = record_doc.decode::<TemporalProjectionRecord>(&forgiving)?;
    ensure!(
        recovered == TemporalProjectionRecord {
            inspected_at: SystemTime::UNIX_EPOCH,
            comment: "kept".to_string(),
        },
        "Forgiving struct decode must recover with default SystemTime"
    );

    Ok(())
}

#[crate::ctb_test]
fn test_challenge_system_time_supported_formats_and_signed_nanos() -> Result<()> {
    use ctb_formats_utilities::FormatId;
    let strict = DcProjectionPolicy::strict();
    for format in [FormatId::SecondsSinceEpoch, FormatId::NsSinceEpoch] {
        for value in [-1_000_000_001_i64, -999_999_999, -1, 0, 1, 999_999_999, 1_000_000_001] {
            let mut source = DcMstring::new();
            source.push_char(DC_BEGIN_TIME_MOMENT);
            source.push_char(DcChar::from_u128(format.dc_id().context("Missing date format")?));
            value.encode_dc_mixed(&mut source)?;
            source.push_char(DC_END_TIME_MOMENT);
            validate_dc_mixed_strict(&source)?;
            let magnitude = match format {
                FormatId::SecondsSinceEpoch => Duration::from_secs(value.unsigned_abs()),
                FormatId::NsSinceEpoch => Duration::from_nanos(value.unsigned_abs()),
                _ => bail!("Unexpected test format"),
            };
            let expected = if value < 0 {
                SystemTime::UNIX_EPOCH.checked_sub(magnitude)
            } else {
                SystemTime::UNIX_EPOCH.checked_add(magnitude)
            }.context("Test timestamp out of range")?;
            ensure!(source.decode::<SystemTime>(&strict)? == expected);
            assert_dc_roundtrip(&expected)?;
        }
    }
    for format in [FormatId::PanDate, FormatId::PanTime, FormatId::PanSuperDate] {
        let mut source = DcMstring::new();
        source.push_char(DC_BEGIN_TIME_MOMENT);
        source.push_char(DcChar::from_u128(format.dc_id().context("Missing date format")?));
        1_i64.encode_dc_mixed(&mut source)?;
        source.push_char(DC_END_TIME_MOMENT);
        validate_dc_mixed_strict(&source)?;
        for policy in [DcProjectionPolicy::strict(), DcProjectionPolicy::forgiving()] {
            let error = source.decode::<SystemTime>(&policy).expect_err("Unsupported date format");
            ensure!(format!("{error:#}").contains("Unsupported SystemTime format"));
        }
    }
    Ok(())
}

// ============================================================================
// 6. STRICT VS FORGIVING POLICY: PATHBUF AND UNFORMATTED PATHS
// ============================================================================

#[crate::ctb_test]
fn test_challenge_strict_vs_forgiving_pathbuf() -> Result<()> {
    let strict = DcProjectionPolicy::strict();
    let forgiving = DcProjectionPolicy::forgiving();

    // 1. Valid formatted PathBuf:
    // Format representation: Dc 511 + f494 + Dc 310 + segments + Dc 311 + Dc 512
    let valid_path = PathBuf::from("usr/local/bin");
    let mut valid_mst = DcMstring::new();
    valid_path.encode_dc_mixed(&mut valid_mst)?;

    let strict_decoded = valid_mst.decode::<PathBuf>(&strict)
        .context("Valid formatted PathBuf must decode under strict policy")?;
    ensure!(strict_decoded == valid_path, "Strict PathBuf decode mismatch");

    // 2. Bare string representation of PathBuf:
    // E.g. "usr/local/bin" as a standard literal string (Dc 260 .. Dc 261)
    let mut bare_str_mst = DcMstring::new();
    "usr/local/bin".to_string().encode_dc_mixed(&mut bare_str_mst)?;

    // Both strict and forgiving policies reject bare string for standalone PathBuf
    ensure!(
        bare_str_mst.decode::<PathBuf>(&strict).is_err(),
        "Strict policy MUST reject bare string representation of PathBuf"
    );
    ensure!(
        bare_str_mst.decode::<PathBuf>(&forgiving).is_err(),
        "Standalone PathBuf decoder MUST require canonical Dc 511 + f494 framing"
    );

    // 3. Bare raw binary representation of PathBuf:
    let mut bare_bin_mst = DcMstring::new();
    bare_bin_mst.push_binary(b"var/log/syslog");

    ensure!(
        bare_bin_mst.decode::<PathBuf>(&strict).is_err(),
        "Strict policy MUST reject bare binary representation of PathBuf"
    );
    ensure!(
        bare_bin_mst.decode::<PathBuf>(&forgiving).is_err(),
        "Standalone PathBuf decoder MUST reject bare binary representation"
    );

    Ok(())
}

// ============================================================================
// 7. EMPIRICAL VERIFICATION: ZERO BASE64 ALLOCATION DURING PROJECTION
// ============================================================================

#[crate::ctb_test]
fn test_challenge_zero_base64_allocation_projection() -> Result<()> {
    let strict = DcProjectionPolicy::strict();

    // 1. Build a 5 MiB binary payload
    let payload_size = 5usize.saturating_mul(1024).saturating_mul(1024);
    let mut payload = Vec::with_capacity(payload_size);
    for i in 0..payload_size {
        payload.push(u8::try_from(i.wrapping_rem(251)).unwrap_or(0));
    }

    // 2. Push raw binary with SHA-256 encapsulation
    let mut mst = DcMstring::new();
    let bp = BinaryPayload::new(payload.clone());
    bp.encode_dc_mixed(&mut mst)?;

    // 3. Project directly into BinaryPayload using DcSyntaxArena
    let t0 = std::time::Instant::now();
    let projected: BinaryPayload = mst.decode::<BinaryPayload>(&strict)
        .context("Failed to project large BinaryPayload")?;
    let elapsed = t0.elapsed();

    // If projection were converting the 5 MiB payload to Base64 text and then parsing,
    // it would allocate ~6.67 MiB of Base64 chars and take substantial CPU time.
    // Zero-allocation projection slices directly into the arena's byte span.
    println!("Projected 5 MiB BinaryPayload in {:?}", elapsed);
    ensure!(
        elapsed < Duration::from_millis(100),
        "Projection of 5 MiB BinaryPayload took too long ({:?}); suspected Base64 conversion or extraneous copies!",
        elapsed
    );

    ensure!(
        projected.as_ref() == payload.as_slice(),
        "Projected binary payload content does not match original bytes!"
    );

    Ok(())
}

#[crate::ctb_test]
fn test_challenge_tuple_with_tagged_first_element_multi_root() -> Result<()> {
    let strict = DcProjectionPolicy::strict();

    let tuple_val: (Option<u32>, String) = (Some(777_u32), "world".to_string());
    let mut mst = DcMstring::new();
    tuple_val.encode_dc_mixed(&mut mst)?;

    let decoded = mst.decode::<(Option<u32>, String)>(&strict)
        .context("Multi-root (Option<u32>, String) failed to decode")?;
    ensure!(decoded == tuple_val, "Tuple with Option element mismatch: got {decoded:?}");

    Ok(())
}

#[crate::ctb_test]
fn test_challenge_array_container_with_type_indicator() -> Result<()> {
    let strict = DcProjectionPolicy::strict();

    // Container with leading Dc 275 type token
    let mut list_mst = DcMstring::new();
    list_mst.push_char(DC_BEGIN_LIST);
    list_mst.push_char(DcChar::from_short(275)); // Type indicator
    10_u32.encode_dc_mixed(&mut list_mst)?;
    list_mst.push_char(DC_LIST_ELEMENT_SEPARATOR);
    20_u32.encode_dc_mixed(&mut list_mst)?;
    list_mst.push_char(DC_LIST_ELEMENT_SEPARATOR);
    30_u32.encode_dc_mixed(&mut list_mst)?;
    list_mst.push_char(DC_END_LIST);

    let decoded = list_mst.decode::<[u32; 3]>(&strict)
        .context("Array in typed list container failed to decode")?;
    ensure!(decoded == [10, 20, 30], "Array mismatch: got {decoded:?}");

    Ok(())
}

#[crate::ctb_test]
fn test_challenge_format_container_extraneous_trailing_children() -> Result<()> {
    let strict = DcProjectionPolicy::strict();
    let forgiving = DcProjectionPolicy::forgiving();

    // Build format representation container Dc 511 ... 512 with 3 children: [format, payload, extraneous]
    let mut mst = DcMstring::new();
    mst.push_char(DC_BEGIN_FORMAT_REPRESENTATION);
    mst.push_char(DcChar::from_format(490)); // IPv4 numeric representation
    42_u32.encode_dc_mixed(&mut mst)?;
    "extraneous".to_string().encode_dc_mixed(&mut mst)?;
    mst.push_char(DC_END_FORMAT_REPRESENTATION);

    let mut nav_strict = DcDocumentNavigator::from_mstr(&mst);
    let mut arena_strict = DcSyntaxArena::new();
    ensure!(
        nav_strict.parse_document_into_arena(&mut arena_strict, &strict).is_err(),
        "Expected strict decode to reject format representation with extraneous trailing children"
    );

    let mut nav = DcDocumentNavigator::from_mstr(&mst);
    let mut arena = DcSyntaxArena::new();
    nav.parse_document_into_arena(&mut arena, &forgiving)?;
    let root_id = arena.roots().first().copied().context("Missing root")?;

    // Under forgiving policy, it should unwrap format and payload
    let mut ctx_forgiving = DcProjectionContext::new(&arena, &forgiving);
    let res_forgiving = ctx_forgiving.unwrap_format_representation(root_id)?;
    ensure!(res_forgiving.is_some(), "Forgiving policy should unwrap format representation");
    let (fmt, payload_id) = res_forgiving.unwrap();
    ensure!(fmt == DcChar::from_format(490));
    let val = u32::decode_dc_node(&mut ctx_forgiving, payload_id)?;
    ensure!(val == 42);

    Ok(())
}

#[crate::ctb_test]
fn test_challenge_array_container_adversarial_boundaries() -> Result<()> {
    let strict = DcProjectionPolicy::strict();

    // 1. Empty array [u32; 0] from empty list container
    let mut empty_mst = DcMstring::new();
    empty_mst.push_char(DC_BEGIN_LIST);
    empty_mst.push_char(DC_TYPE_ANY);
    empty_mst.push_char(DC_END_LIST);

    let decoded_empty = empty_mst.decode::<[u32; 0]>(&strict)
        .context("Empty list container failed to decode into [u32; 0]")?;
    ensure!(decoded_empty == [0u32; 0], "Empty array mismatch");

    // 2. Length mismatch: container has 4 items, target is [u32; 3]
    let mut too_many_mst = DcMstring::new();
    too_many_mst.push_char(DC_BEGIN_LIST);
    too_many_mst.push_char(DC_TYPE_ANY);
    1_u32.encode_dc_mixed(&mut too_many_mst)?;
    too_many_mst.push_char(DC_LIST_ELEMENT_SEPARATOR);
    2_u32.encode_dc_mixed(&mut too_many_mst)?;
    too_many_mst.push_char(DC_LIST_ELEMENT_SEPARATOR);
    3_u32.encode_dc_mixed(&mut too_many_mst)?;
    too_many_mst.push_char(DC_LIST_ELEMENT_SEPARATOR);
    4_u32.encode_dc_mixed(&mut too_many_mst)?;
    too_many_mst.push_char(DC_END_LIST);

    let res_too_many = too_many_mst.decode::<[u32; 3]>(&strict);
    ensure!(
        res_too_many.is_err(),
        "Expected array decode to fail when container has more items than N"
    );

    // 3. Length mismatch: container has 2 items, target is [u32; 3]
    let mut too_few_mst = DcMstring::new();
    too_few_mst.push_char(DC_BEGIN_LIST);
    too_few_mst.push_char(DC_TYPE_ANY);
    1_u32.encode_dc_mixed(&mut too_few_mst)?;
    too_few_mst.push_char(DC_LIST_ELEMENT_SEPARATOR);
    2_u32.encode_dc_mixed(&mut too_few_mst)?;
    too_few_mst.push_char(DC_END_LIST);

    let res_too_few = too_few_mst.decode::<[u32; 3]>(&strict);
    ensure!(
        res_too_few.is_err(),
        "Expected array decode to fail when container has fewer items than N"
    );

    // 4. Array of Strings [String; 3]
    let string_arr = ["alpha".to_string(), "beta".to_string(), "gamma".to_string()];
    let mut str_mst = DcMstring::new();
    str_mst.push_char(DC_BEGIN_LIST);
    str_mst.push_char(DC_TYPE_ANY);
    string_arr.get(0).context("Missing elem 0")?.encode_dc_mixed(&mut str_mst)?;
    str_mst.push_char(DC_LIST_ELEMENT_SEPARATOR);
    string_arr.get(1).context("Missing elem 1")?.encode_dc_mixed(&mut str_mst)?;
    str_mst.push_char(DC_LIST_ELEMENT_SEPARATOR);
    string_arr.get(2).context("Missing elem 2")?.encode_dc_mixed(&mut str_mst)?;
    str_mst.push_char(DC_END_LIST);

    let decoded_str_arr = str_mst.decode::<[String; 3]>(&strict)
        .context("List container failed to decode into [String; 3]")?;
    ensure!(decoded_str_arr == string_arr, "String array mismatch");

    // 5. Array of Option values [Option<u32>; 3]
    let opt_arr = [Some(10_u32), None, Some(30_u32)];
    let mut opt_mst = DcMstring::new();
    opt_mst.push_char(DC_BEGIN_LIST);
    opt_mst.push_char(DC_TYPE_ANY);
    opt_arr.get(0).context("Missing opt 0")?.encode_dc_mixed(&mut opt_mst)?;
    opt_mst.push_char(DC_LIST_ELEMENT_SEPARATOR);
    opt_arr.get(1).context("Missing opt 1")?.encode_dc_mixed(&mut opt_mst)?;
    opt_mst.push_char(DC_LIST_ELEMENT_SEPARATOR);
    opt_arr.get(2).context("Missing opt 2")?.encode_dc_mixed(&mut opt_mst)?;
    opt_mst.push_char(DC_END_LIST);

    let decoded_opt_arr = opt_mst.decode::<[Option<u32>; 3]>(&strict)
        .context("List container failed to decode into [Option<u32>; 3]")?;
    ensure!(decoded_opt_arr == opt_arr, "Option array mismatch");

    Ok(())
}

#[crate::ctb_test]
fn test_challenge_tuple_all_option_combinations_roundtrip() -> Result<()> {
    let strict = DcProjectionPolicy::strict();

    // 1. (Some, Some)
    let pair1: (Option<u32>, Option<String>) = (Some(42), Some("yes".to_string()));
    let mut mst1 = DcMstring::new();
    pair1.encode_dc_mixed(&mut mst1)?;
    let dec1 = mst1.decode::<(Option<u32>, Option<String>)>(&strict)
        .context("(Some, Some) failed to roundtrip")?;
    ensure!(dec1 == pair1, "(Some, Some) mismatch: {dec1:?}");

    // 2. (Some, None)
    let pair2: (Option<u32>, Option<String>) = (Some(42), None);
    let mut mst2 = DcMstring::new();
    pair2.encode_dc_mixed(&mut mst2)?;
    let dec2 = mst2.decode::<(Option<u32>, Option<String>)>(&strict)
        .context("(Some, None) failed to roundtrip")?;
    ensure!(dec2 == pair2, "(Some, None) mismatch: {dec2:?}");

    // 3. (None, Some)
    let pair3: (Option<u32>, Option<String>) = (None, Some("no".to_string()));
    let mut mst3 = DcMstring::new();
    pair3.encode_dc_mixed(&mut mst3)?;
    let dec3 = mst3.decode::<(Option<u32>, Option<String>)>(&strict)
        .context("(None, Some) failed to roundtrip")?;
    ensure!(dec3 == pair3, "(None, Some) mismatch: {dec3:?}");

    // 4. (None, None)
    let pair4: (Option<u32>, Option<String>) = (None, None);
    let mut mst4 = DcMstring::new();
    pair4.encode_dc_mixed(&mut mst4)?;
    let dec4 = mst4.decode::<(Option<u32>, Option<String>)>(&strict)
        .context("(None, None) failed to roundtrip")?;
    ensure!(dec4 == pair4, "(None, None) mismatch: {dec4:?}");

    // 5. Boundary value tuple (u128::MAX, 0_u8)
    let pair_boundary: (u128, u8) = (u128::MAX, 0);
    let mut mst_boundary = DcMstring::new();
    pair_boundary.encode_dc_mixed(&mut mst_boundary)?;
    let dec_boundary = mst_boundary.decode::<(u128, u8)>(&strict)
        .context("Boundary tuple failed to roundtrip")?;
    ensure!(dec_boundary == pair_boundary, "Boundary tuple mismatch");

    Ok(())
}

#[crate::ctb_test]
fn test_challenge_format_container_adversarial_stress() -> Result<()> {
    let strict = DcProjectionPolicy::strict();
    let forgiving = DcProjectionPolicy::forgiving();

    // 1. Format container with 4 children: [format, payload, child2, child3]
    let mut mst4 = DcMstring::new();
    mst4.push_char(DC_BEGIN_FORMAT_REPRESENTATION);
    mst4.push_char(DcChar::from_format(490));
    888_u32.encode_dc_mixed(&mut mst4)?;
    "extra1".to_string().encode_dc_mixed(&mut mst4)?;
    "extra2".to_string().encode_dc_mixed(&mut mst4)?;
    mst4.push_char(DC_END_FORMAT_REPRESENTATION);

    let mut nav4_strict = DcDocumentNavigator::from_mstr(&mst4);
    let mut arena4_strict = DcSyntaxArena::new();
    ensure!(
        nav4_strict.parse_document_into_arena(&mut arena4_strict, &strict).is_err(),
        "4-child format container must fail in Stage 1 under strict policy"
    );

    let mut nav4 = DcDocumentNavigator::from_mstr(&mst4);
    let mut arena4 = DcSyntaxArena::new();
    nav4.parse_document_into_arena(&mut arena4, &forgiving)?;
    let root4 = arena4.roots().first().copied().context("Missing root 4")?;

    let mut ctx_forgiving4 = DcProjectionContext::new(&arena4, &forgiving);
    let (fmt4, payload_id4) = ctx_forgiving4
        .unwrap_format_representation(root4)?
        .context("4-child format container should unwrap under forgiving policy")?;
    ensure!(fmt4 == DcChar::from_format(490));
    let val4 = u32::decode_dc_node(&mut ctx_forgiving4, payload_id4)?;
    ensure!(val4 == 888);

    // 2. Exact 2-child format container passes both strict and forgiving
    let mut mst2 = DcMstring::new();
    mst2.push_char(DC_BEGIN_FORMAT_REPRESENTATION);
    mst2.push_char(DcChar::from_format(490));
    777_u32.encode_dc_mixed(&mut mst2)?;
    mst2.push_char(DC_END_FORMAT_REPRESENTATION);

    let mut nav2 = DcDocumentNavigator::from_mstr(&mst2);
    let mut arena2 = DcSyntaxArena::new();
    nav2.parse_document_into_arena(&mut arena2, &strict)?;
    let root2 = arena2.roots().first().copied().context("Missing root 2")?;

    let mut ctx_strict2 = DcProjectionContext::new(&arena2, &strict);
    let (fmt2, payload_id2) = ctx_strict2
        .unwrap_format_representation(root2)?
        .context("2-child format container should unwrap under strict policy")?;
    ensure!(fmt2 == DcChar::from_format(490));
    let val2 = u32::decode_dc_node(&mut ctx_strict2, payload_id2)?;
    ensure!(val2 == 777);

    // 3. End-to-end typed projection: Ipv4Addr wrapped in 511 container with an extra child
    use std::net::Ipv4Addr;
    let valid_ip = Ipv4Addr::new(192, 168, 1, 1);
    let mut mst_ip_extra = DcMstring::new();
    mst_ip_extra.push_char(DC_BEGIN_FORMAT_REPRESENTATION);
    mst_ip_extra.push_char(DcChar::from_format(490)); // IPv4 format
    u32::from_be_bytes(valid_ip.octets()).encode_dc_mixed(&mut mst_ip_extra)?;
    "extraneous_child".to_string().encode_dc_mixed(&mut mst_ip_extra)?;
    mst_ip_extra.push_char(DC_END_FORMAT_REPRESENTATION);

    let res_strict_ip = mst_ip_extra.decode::<Ipv4Addr>(&strict);
    ensure!(
        res_strict_ip.is_err(),
        "End-to-end Ipv4Addr with extraneous child in 511 container MUST fail under strict policy!"
    );

    let res_forgiving_ip = mst_ip_extra.decode::<Ipv4Addr>(&forgiving)
        .context("End-to-end Ipv4Addr with extraneous child in 511 container MUST recover under forgiving policy")?;
    ensure!(res_forgiving_ip == valid_ip, "Forgiving Ipv4Addr mismatch: got {res_forgiving_ip:?}");

    Ok(())
}

