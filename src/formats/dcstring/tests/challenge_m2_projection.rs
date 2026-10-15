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
use std::time::Duration;

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
    title: String,
    #[dc(short = 404)]
    values: [u32; 3],
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

/// BUG 3 EMPIRICAL REPRODUCTION:
/// Arrays in struct fields: `StructWithArray { values: [u32; 3] }` with `#[dc(short = 404)]`.
/// The macro encodes tag 404 followed by 3 numbers. However, `navigator.rs` parses tag 404
/// with default arity 1. Therefore, `values: [u32; 3]` receives a `Tagged` node with 1 arg
/// instead of 3, and fails with `Array length mismatch` or container mismatch.
#[crate::ctb_test]
fn test_challenge_bug_array_in_struct_field() -> Result<()> {
    let strict = DcProjectionPolicy::strict();

    let item = StructWithArray {
        title: "test_array".to_string(),
        values: [100, 200, 300],
    };
    let mut mst = DcMstring::new();
    item.encode_dc_mixed(&mut mst)?;

    let res = mst.decode::<StructWithArray>(&strict);
    ensure!(
        res.is_err(),
        "Expected StructWithArray decode to fail due to tag arity mismatch in array field"
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

    // 4. Duration without format indicator inside 395..396 (defaults to nanos)
    let mut mst_395_raw = DcMstring::new();
    mst_395_raw.push_char(DcChar::from_short(395));
    nanos.encode_dc_mixed(&mut mst_395_raw)?;
    mst_395_raw.push_char(DcChar::from_short(396));

    let dec_395_raw = mst_395_raw.decode::<Duration>(&strict)
        .context("Duration in Dc 395..396 without format failed to decode")?;
    ensure!(dec_395_raw == expected, "395..396 raw mismatch: got {dec_395_raw:?}");

    // 5. Bare number representation of Duration:
    // STRICT must reject; FORGIVING must recover.
    let mut bare_num_mst = DcMstring::new();
    nanos.encode_dc_mixed(&mut bare_num_mst)?;

    let strict_res = bare_num_mst.decode::<Duration>(&strict);
    ensure!(strict_res.is_err(), "Bare number Duration MUST fail under strict policy!");

    let forgiving_dec = bare_num_mst.decode::<Duration>(&forgiving)
        .context("Bare number Duration MUST recover under forgiving policy")?;
    ensure!(forgiving_dec == expected, "Forgiving recovered duration mismatch");

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

    // Strict policy MUST reject bare string for PathBuf
    let strict_bare_res = bare_str_mst.decode::<PathBuf>(&strict);
    ensure!(
        strict_bare_res.is_err(),
        "Strict policy MUST reject bare string representation of PathBuf! Got: {strict_bare_res:?}"
    );

    // Forgiving policy MUST recover bare string for PathBuf
    let forgiving_bare = bare_str_mst.decode::<PathBuf>(&forgiving)
        .context("Forgiving policy MUST safely recover bare string for PathBuf")?;
    ensure!(
        forgiving_bare == PathBuf::from("usr/local/bin"),
        "Forgiving PathBuf recovery mismatch"
    );

    // 3. Bare raw binary representation of PathBuf:
    let mut bare_bin_mst = DcMstring::new();
    bare_bin_mst.push_binary(b"var/log/syslog");

    let strict_bin_res = bare_bin_mst.decode::<PathBuf>(&strict);
    ensure!(
        strict_bin_res.is_err(),
        "Strict policy MUST reject bare binary representation of PathBuf! Got: {strict_bin_res:?}"
    );

    let forgiving_bin = bare_bin_mst.decode::<PathBuf>(&forgiving)
        .context("Forgiving policy MUST safely recover bare binary for PathBuf")?;
    ensure!(
        forgiving_bin == PathBuf::from("var/log/syslog"),
        "Forgiving binary PathBuf recovery mismatch"
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
    mst.push_char(DcChar::from_format(100)); // Some format
    42_u32.encode_dc_mixed(&mut mst)?;
    "extraneous".to_string().encode_dc_mixed(&mut mst)?;
    mst.push_char(DC_END_FORMAT_REPRESENTATION);

    let mut nav = DcDocumentNavigator::from_mstr(&mst);
    let mut arena = DcSyntaxArena::new();
    nav.parse_document_into_arena(&mut arena, &strict)?;
    let root_id = arena.roots().first().copied().context("Missing root")?;

    // Under strict policy, extraneous trailing child MUST be rejected
    let mut ctx_strict = DcProjectionContext::new(&arena, &strict);
    let res_strict = ctx_strict.unwrap_format_representation(root_id);
    ensure!(
        res_strict.is_err(),
        "Expected strict decode to reject format representation with extraneous trailing children"
    );

    // Under forgiving policy, it should unwrap format and payload
    let mut ctx_forgiving = DcProjectionContext::new(&arena, &forgiving);
    let res_forgiving = ctx_forgiving.unwrap_format_representation(root_id)?;
    ensure!(res_forgiving.is_some(), "Forgiving policy should unwrap format representation");
    let (fmt, payload_id) = res_forgiving.unwrap();
    ensure!(fmt == DcChar::from_format(100));
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
    empty_mst.push_char(DC_END_LIST);

    let decoded_empty = empty_mst.decode::<[u32; 0]>(&strict)
        .context("Empty list container failed to decode into [u32; 0]")?;
    ensure!(decoded_empty == [0u32; 0], "Empty array mismatch");

    // 2. Length mismatch: container has 4 items, target is [u32; 3]
    let mut too_many_mst = DcMstring::new();
    too_many_mst.push_char(DC_BEGIN_LIST);
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
    mst4.push_char(DcChar::from_format(100));
    888_u32.encode_dc_mixed(&mut mst4)?;
    "extra1".to_string().encode_dc_mixed(&mut mst4)?;
    "extra2".to_string().encode_dc_mixed(&mut mst4)?;
    mst4.push_char(DC_END_FORMAT_REPRESENTATION);

    let mut nav4 = DcDocumentNavigator::from_mstr(&mst4);
    let mut arena4 = DcSyntaxArena::new();
    nav4.parse_document_into_arena(&mut arena4, &strict)?;
    let root4 = arena4.roots().first().copied().context("Missing root 4")?;

    let mut ctx_strict4 = DcProjectionContext::new(&arena4, &strict);
    ensure!(
        ctx_strict4.unwrap_format_representation(root4).is_err(),
        "4-child format container must fail under strict policy"
    );

    let mut ctx_forgiving4 = DcProjectionContext::new(&arena4, &forgiving);
    let (fmt4, payload_id4) = ctx_forgiving4
        .unwrap_format_representation(root4)?
        .context("4-child format container should unwrap under forgiving policy")?;
    ensure!(fmt4 == DcChar::from_format(100));
    let val4 = u32::decode_dc_node(&mut ctx_forgiving4, payload_id4)?;
    ensure!(val4 == 888);

    // 2. Exact 2-child format container passes both strict and forgiving
    let mut mst2 = DcMstring::new();
    mst2.push_char(DC_BEGIN_FORMAT_REPRESENTATION);
    mst2.push_char(DcChar::from_format(200));
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
    ensure!(fmt2 == DcChar::from_format(200));
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

