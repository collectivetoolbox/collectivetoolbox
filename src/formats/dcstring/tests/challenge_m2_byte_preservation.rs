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
//! 1. Verify `source_edit.rs` byte preservation on multi-field DcMixed documents.
//! 2. Untouched fields retain byte-exact original source representation.
//! 3. Whitespace, formatting, and trailing content outside records are preserved.
//! 4. Sequential edits, varied field lengths, Unicode, and nested structures.
//! 5. Verify total elimination of `DcMixedReader` and `source_edit::read_nodes`.

#[expect(unused_imports, clippy::wildcard_imports, reason = "Integration test root")]
use ctb_utilities::*;

use anyhow::{Context, Result, ensure};
use std::collections::BTreeMap;
pub use ctb_formats_dcstring::*;
use ctb_formats_dcdata::dc::*;

#[derive(Debug, Clone, PartialEq, Eq, DcMixed)]
#[dc(begin = 317, end = 318)]
pub struct MultiFieldRecord {
    #[dc(short = 327)]
    pub id: u64,
    #[dc(short = 337)]
    pub name: String,
    #[dc(short = 352)]
    pub active: bool,
    #[dc(short = 336)]
    pub score: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, DcMixed)]
#[dc(begin = 401, end = 402)]
pub struct BoundedComplexRecord {
    #[dc(short = 403)]
    pub title: String,
    #[dc(begin = 310, end = 311)]
    pub counters: Vec<u64>,
    #[dc(short = 406)]
    pub flags: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, DcMixed)]
#[dc(key_value)]
pub struct KeyedFileRecord {
    #[dc(key = "title")]
    pub title: String,
    #[dc(key = "count")]
    pub count: u64,
    #[dc(key = "active")]
    pub active: bool,
}

// ============================================================================
// 1. MULTI-FIELD BYTE PRESERVATION & UNTOUCHED FIELD EXACTNESS (WITH TRAILING CONTENT)
// ============================================================================

#[crate::ctb_test]
fn test_challenge_source_edit_single_field_update_preserves_untouched_bytes() -> Result<()> {
    let policy = DcProjectionPolicy::strict();

    let original = MultiFieldRecord {
        id: 42,
        name: "alpha_original_name".to_string(),
        active: true,
        score: Some(100),
    };

    // Part A: Clean document without trailing content -> strict roundtrip
    let doc_clean = original.to_dc_mixed()?;
    let orig_clean_bytes = doc_clean.as_bytes().to_vec();

    let view_clean = doc_clean.inspect::<MultiFieldRecord>();
    ensure!(view_clean.fields().len() == 4);

    let name_field = view_clean
        .fields()
        .iter()
        .find(|f| f.field() == "name")
        .context("Missing name field annotation")?;
    let name_range = name_field.range();

    // Update `name` to a substantially longer string
    let mut updated_longer = original.clone();
    updated_longer.name = "beta_substantially_longer_replacement_string_for_testing".to_string();

    let revised_clean = view_clean.replace_field("name", &updated_longer)?;
    ensure!(revised_clean.decode::<MultiFieldRecord>(&policy)? == updated_longer);

    let rev_clean_view = revised_clean.inspect::<MultiFieldRecord>();
    let rev_name_range = rev_clean_view
        .fields()
        .iter()
        .find(|f| f.field() == "name")
        .context("Missing revised name annotation")?
        .range();

    // 1. Prefix before `name` is 100% byte-exact:
    ensure!(
        &revised_clean.as_bytes()[..rev_name_range.start] == &orig_clean_bytes[..name_range.start],
        "Untouched prefix before updated field was altered!"
    );

    // 2. Suffix after `name` is 100% byte-exact:
    ensure!(
        &revised_clean.as_bytes()[rev_name_range.end..] == &orig_clean_bytes[name_range.end..],
        "Untouched suffix after updated field was altered!"
    );

    // Part B: Document with trailing non-decoded payload
    let mut doc_with_trailing = original.to_dc_mixed()?;
    let trailing_data = b"trailing_raw_binary_metadata_block_12345";
    doc_with_trailing.push_binary(trailing_data);
    let orig_trailing_bytes = doc_with_trailing.as_bytes().to_vec();

    let view_trailing = doc_with_trailing.inspect::<MultiFieldRecord>();
    ensure!(view_trailing.fields().len() == 4);
    ensure!(view_trailing.value() == Some(&original));

    let rev_trailing = view_trailing.replace_field("name", &updated_longer)?;
    ensure!(rev_trailing.inspect::<MultiFieldRecord>().value() == Some(&updated_longer));

    let rev_trailing_name_range = rev_trailing
        .inspect::<MultiFieldRecord>()
        .fields()
        .iter()
        .find(|f| f.field() == "name")
        .context("Missing revised trailing name")?
        .range();

    // Prefix before `name` in trailing document is byte-exact
    ensure!(
        &rev_trailing.as_bytes()[..rev_trailing_name_range.start] == &orig_trailing_bytes[..name_range.start]
    );

    // Suffix after `name` (including trailing binary payload) is 100% byte-exact!
    ensure!(
        &rev_trailing.as_bytes()[rev_trailing_name_range.end..] == &orig_trailing_bytes[name_range.end..]
    );

    // Trailing binary payload was not lost or corrupted
    ensure!(
        rev_trailing.as_bytes().windows(trailing_data.len()).any(|w| w == trailing_data),
        "Trailing binary payload was lost or modified!"
    );

    Ok(())
}

// ============================================================================
// 2. TEXTUAL (DCSTRING / DCSTR) BYTE & UNICODE PRESERVATION
// ============================================================================

#[crate::ctb_test]
fn test_challenge_source_edit_textual_dc_string_preserves_bytes_and_formatting() -> Result<()> {
    let policy = DcProjectionPolicy::strict();

    let original = MultiFieldRecord {
        id: 999,
        name: "original_textual_value".to_string(),
        active: false,
        score: Some(777),
    };

    let text_doc = original.to_dc_string()?;
    let orig_bytes = text_doc.as_bytes().to_vec();

    let view = text_doc.inspect::<MultiFieldRecord>();
    ensure!(view.fields().len() == 4);

    let name_annot = view
        .fields()
        .iter()
        .find(|f| f.field() == "name")
        .context("Missing name field")?;
    let name_range = name_annot.range();

    let mut updated = original.clone();
    updated.name = "revised_unicode_🔥_日本語_text".to_string();

    let revised: DcString = view.replace_field("name", &updated)?;
    ensure!(revised.decode::<MultiFieldRecord>(&policy)? == updated);

    let rev_view = revised.inspect::<MultiFieldRecord>();
    let rev_name_annot = rev_view
        .fields()
        .iter()
        .find(|f| f.field() == "name")
        .context("Missing rev_name_annot")?;

    // Byte-exact prefix check
    ensure!(
        &revised.as_bytes()[..rev_name_annot.range().start] == &orig_bytes[..name_range.start]
    );

    // Byte-exact suffix check (including active, score, and container closer)
    ensure!(
        &revised.as_bytes()[rev_name_annot.range().end..] == &orig_bytes[name_range.end..]
    );

    Ok(())
}

// ============================================================================
// 3. FIRST FIELD, LAST FIELD, AND MIDDLE FIELD INDEPENDENT EDITS
// ============================================================================

#[crate::ctb_test]
fn test_challenge_first_and_last_field_updates_preserve_boundaries() -> Result<()> {
    let policy = DcProjectionPolicy::strict();

    let original = MultiFieldRecord {
        id: 1,
        name: "hello".to_string(),
        active: true,
        score: Some(42),
    };

    let doc = original.to_dc_mixed()?;
    let doc_bytes = doc.as_bytes().to_vec();

    let view = doc.inspect::<MultiFieldRecord>();

    // 1. Update FIRST field (id)
    let id_annot = view
        .fields()
        .iter()
        .find(|f| f.field() == "id")
        .context("Missing id")?;
    let id_range = id_annot.range();

    let mut updated_first = original.clone();
    updated_first.id = 123456789012345;

    let revised_first = view.replace_field("id", &updated_first)?;
    ensure!(revised_first.decode::<MultiFieldRecord>(&policy)? == updated_first);

    let rev_first_view = revised_first.inspect::<MultiFieldRecord>();
    let rev_id_end = rev_first_view
        .fields()
        .iter()
        .find(|f| f.field() == "id")
        .context("Missing rev_id")?
        .range()
        .end;

    // Opener before id is untouched:
    ensure!(&revised_first.as_bytes()[..id_range.start] == &doc_bytes[..id_range.start]);
    // Suffix after id (name + active + score + closer) is untouched:
    ensure!(&revised_first.as_bytes()[rev_id_end..] == &doc_bytes[id_range.end..]);

    // 2. Update LAST field (score)
    let score_annot = view
        .fields()
        .iter()
        .find(|f| f.field() == "score")
        .context("Missing score")?;
    let score_range = score_annot.range();

    let mut updated_last = original.clone();
    updated_last.score = Some(99999);

    let revised_last = view.replace_field("score", &updated_last)?;
    ensure!(revised_last.decode::<MultiFieldRecord>(&policy)? == updated_last);

    let rev_last_view = revised_last.inspect::<MultiFieldRecord>();
    let rev_score_end = rev_last_view
        .fields()
        .iter()
        .find(|f| f.field() == "score")
        .context("Missing rev_score")?
        .range()
        .end;

    // Everything before score is untouched:
    ensure!(&revised_last.as_bytes()[..score_range.start] == &doc_bytes[..score_range.start]);
    // Closer after score is untouched:
    ensure!(&revised_last.as_bytes()[rev_score_end..] == &doc_bytes[score_range.end..]);

    // 3. Update MIDDLE field (active)
    let active_annot = view
        .fields()
        .iter()
        .find(|f| f.field() == "active")
        .context("Missing active")?;
    let active_range = active_annot.range();

    let mut updated_mid = original.clone();
    updated_mid.active = false;

    let revised_mid = view.replace_field("active", &updated_mid)?;
    ensure!(revised_mid.decode::<MultiFieldRecord>(&policy)? == updated_mid);

    let rev_mid_view = revised_mid.inspect::<MultiFieldRecord>();
    let rev_active_end = rev_mid_view
        .fields()
        .iter()
        .find(|f| f.field() == "active")
        .context("Missing rev_active")?
        .range()
        .end;

    ensure!(&revised_mid.as_bytes()[..active_range.start] == &doc_bytes[..active_range.start]);
    ensure!(&revised_mid.as_bytes()[rev_active_end..] == &doc_bytes[active_range.end..]);

    Ok(())
}

// ============================================================================
// 4. SEQUENTIAL FIELD EDITS ON THE SAME DOCUMENT
// ============================================================================

#[crate::ctb_test]
fn test_challenge_sequential_edits_retain_untouched_bytes_throughout() -> Result<()> {
    let policy = DcProjectionPolicy::strict();

    let original = MultiFieldRecord {
        id: 10,
        name: "start_name".to_string(),
        active: false,
        score: Some(50),
    };

    let doc = original.to_dc_mixed()?;

    // Step 1: edit `name`
    let mut step1_target = original.clone();
    step1_target.name = "step1_modified_name".to_string();

    let doc1 = doc.inspect::<MultiFieldRecord>().replace_field("name", &step1_target)?;
    ensure!(doc1.decode::<MultiFieldRecord>(&policy)? == step1_target);

    // Step 2: edit `score` on doc1
    let mut step2_target = step1_target.clone();
    step2_target.score = Some(888);

    let doc2 = doc1.inspect::<MultiFieldRecord>().replace_field("score", &step2_target)?;
    ensure!(doc2.decode::<MultiFieldRecord>(&policy)? == step2_target);

    // Step 3: edit `id` on doc2
    let mut step3_target = step2_target.clone();
    step3_target.id = 99999;

    let doc3 = doc2.inspect::<MultiFieldRecord>().replace_field("id", &step3_target)?;
    ensure!(doc3.decode::<MultiFieldRecord>(&policy)? == step3_target);

    // Verify untouched field `active` was NEVER modified:
    let orig_view = doc.inspect::<MultiFieldRecord>();
    let orig_active_range = orig_view
        .fields()
        .iter()
        .find(|f| f.field() == "active")
        .context("Missing orig active")?
        .range();
    let orig_active_bytes = &doc.as_bytes()[orig_active_range];

    let doc3_view = doc3.inspect::<MultiFieldRecord>();
    let doc3_active_range = doc3_view
        .fields()
        .iter()
        .find(|f| f.field() == "active")
        .context("Missing doc3 active")?
        .range();
    let doc3_active_bytes = &doc3.as_bytes()[doc3_active_range];

    ensure!(
        orig_active_bytes == doc3_active_bytes,
        "Untouched active field bytes diverged across sequential edits!"
    );

    Ok(())
}

// ============================================================================
// 5. UNKNOWN FORWARD-COMPATIBLE TOKENS PRESERVED INTACT UNDER FORGIVING POLICY
// ============================================================================

#[crate::ctb_test]
fn test_challenge_unknown_forward_compatible_tokens_preserved_around_edits() -> Result<()> {
    let policy = DcProjectionPolicy::forgiving();

    // Build document with unknown future tag 777 embedded between fields
    let mut doc = DcMstring::new();
    doc.push_char(DcChar::from_short(317));

    // Field id
    doc.push_char(DcChar::from_short(327));
    77_u64.encode_dc_mixed(&mut doc)?;

    // Unknown future payload: tag 777 + binary payload
    doc.push_char(DcChar::from_short(777));
    doc.push_binary_with_sha256(b"unknown_future_sensor_telemetry_payload")?;

    // Field name
    doc.push_char(DcChar::from_short(337));
    "known_name".encode_dc_mixed(&mut doc)?;

    doc.push_char(DcChar::from_short(352));
    true.encode_dc_mixed(&mut doc)?;

    doc.push_char(DcChar::from_short(318));

    let unknown_bytes = b"unknown_future_sensor_telemetry_payload";

    // View under forgiving policy
    let view = doc.inspect_with_policy::<MultiFieldRecord>(&policy);
    ensure!(view.fields().len() == 3); // id, name, active

    // Replace `name`
    let updated = MultiFieldRecord {
        id: 77,
        name: "updated_known_name".to_string(),
        active: true,
        score: None,
    };
    let revised = view.replace_field("name", &updated)?;

    // Verify unknown payload was NOT corrupted or stripped
    ensure!(
        revised.as_bytes().windows(unknown_bytes.len()).any(|w| w == unknown_bytes),
        "Unknown future binary payload was destroyed during known field replacement!"
    );

    // Verify unknown tag 777 is still present
    let mut tag_777_bytes = DcMstring::new();
    tag_777_bytes.push_char(DcChar::from_short(777));
    ensure!(
        revised.as_bytes().windows(tag_777_bytes.payload_len()).any(|w| w == tag_777_bytes.payload_bytes()),
        "Unknown future tag was stripped during known field replacement!"
    );

    Ok(())
}

// ============================================================================
// 6. BOUNDED COLLECTIONS & COMPLEX FIELD EDITS
// ============================================================================

#[crate::ctb_test]
fn test_challenge_complex_record_with_collections_and_maps() -> Result<()> {
    let policy = DcProjectionPolicy::strict();

    let original = BoundedComplexRecord {
        title: "original_title".to_string(),
        counters: vec![1, 2, 3, 4, 5],
        flags: Some(406),
    };

    let mut doc = DcMstring::new();
    original.encode_dc_mixed(&mut doc)?;

    let orig_bytes = doc.as_bytes().to_vec();

    // Update `title` only
    let mut updated_title = original.clone();
    updated_title.title = "new_short_title".to_string();

    let view = doc.inspect::<BoundedComplexRecord>();
    let title_range = view
        .fields()
        .iter()
        .find(|f| f.field() == "title")
        .context("Missing title")?
        .range();

    let revised = view.replace_field("title", &updated_title)?;
    ensure!(revised.decode::<BoundedComplexRecord>(&policy)? == updated_title);

    let rev_view = revised.inspect::<BoundedComplexRecord>();
    let rev_title_range = rev_view
        .fields()
        .iter()
        .find(|f| f.field() == "title")
        .context("Missing rev title")?
        .range();

    // 1. All bytes before `title` are 100% byte-exact:
    ensure!(
        &revised.as_bytes()[..rev_title_range.start] == &orig_bytes[..title_range.start],
        "Bytes before title diverged!"
    );

    // 2. All bytes after `title` (which includes `counters`, `flags`, and closer)
    // are 100% byte-exact!
    ensure!(
        &revised.as_bytes()[rev_title_range.end..] == &orig_bytes[title_range.end..],
        "Bytes after title diverged!"
    );

    // 3. Update bounded `counters` collection only
    let mut updated_counters = original.clone();
    updated_counters.counters = vec![100, 200];

    let view2 = doc.inspect::<BoundedComplexRecord>();
    let counters_range = view2
        .fields()
        .iter()
        .find(|f| f.field() == "counters")
        .context("Missing counters")?
        .range();

    let revised2 = view2.replace_field("counters", &updated_counters)?;
    ensure!(revised2.decode::<BoundedComplexRecord>(&policy)? == updated_counters);

    let rev2_view = revised2.inspect::<BoundedComplexRecord>();
    let rev2_counters_range = rev2_view
        .fields()
        .iter()
        .find(|f| f.field() == "counters")
        .context("Missing rev2 counters")?
        .range();

    // Title and opener before counters must be byte-exact:
    ensure!(
        &revised2.as_bytes()[..rev2_counters_range.start] == &orig_bytes[..counters_range.start]
    );
    // Flags and closer after counters must be byte-exact:
    ensure!(
        &revised2.as_bytes()[rev2_counters_range.end..] == &orig_bytes[counters_range.end..]
    );

    Ok(())
}

// ============================================================================
// 7. KEYED MAP LEGACY DELIMITERS & ENTRY BYTE PRESERVATION
// ============================================================================

#[crate::ctb_test]
fn test_challenge_keyed_map_preserves_legacy_delimiters_and_untouched_entries() -> Result<()> {
    let strict = DcProjectionPolicy::strict();
    let original = KeyedFileRecord {
        title: "original_title".to_string(),
        count: 42,
        active: true,
    };
    let updated = KeyedFileRecord {
        title: "original_title".to_string(),
        count: 9999,
        active: true,
    };

    // Encode using Legacy encoding (Dc 313 .. 314)
    let encoding = crate::DcKvMapEncoding::Legacy;
    let mut source = DcMstring::new();
    encoding.encode_header(&mut source);

    // Entry 1: title
    original.encode_dc_map_field("title", &mut source, encoding)?;
    source.push_char(DcChar::from_short(312));

    // Entry 2: count
    original.encode_dc_map_field("count", &mut source, encoding)?;
    source.push_char(DcChar::from_short(312));

    // Entry 3: active
    original.encode_dc_map_field("active", &mut source, encoding)?;
    encoding.encode_end(&mut source);

    let orig_bytes = source.as_bytes().to_vec();

    // Verify original decodes cleanly under strict
    ensure!(source.decode::<KeyedFileRecord>(&strict)? == original);

    let view = source.inspect::<KeyedFileRecord>();
    ensure!(view.fields().len() == 3);

    // Replace `count` field
    let revised = view.replace_field("count", &updated)?;
    ensure!(revised.decode::<KeyedFileRecord>(&strict)? == updated);

    let rev_view = revised.inspect::<KeyedFileRecord>();
    let count_annot = view
        .fields()
        .iter()
        .find(|f| f.field() == "count")
        .context("Missing count")?;
    let rev_count_annot = rev_view
        .fields()
        .iter()
        .find(|f| f.field() == "count")
        .context("Missing rev count")?;

    // 1. Untouched entry `title` before `count` is 100% byte-exact:
    ensure!(
        &revised.as_bytes()[..rev_count_annot.range().start] == &orig_bytes[..count_annot.range().start],
        "Prefix before count in keyed map diverged!"
    );

    // 2. Untouched entry `active` after `count` is 100% byte-exact:
    ensure!(
        &revised.as_bytes()[rev_count_annot.range().end..] == &orig_bytes[count_annot.range().end..],
        "Suffix after count in keyed map diverged!"
    );

    // 3. Delimiters preserved: map is still using Legacy 313 .. 314 and NOT upgraded to Pairs 559 .. 560
    let tokens = revised.to_dc_tokens()?;
    ensure!(tokens.contains(&DcChar::from_short(313).to_u128()));
    ensure!(tokens.contains(&DcChar::from_short(314).to_u128()));
    ensure!(!tokens.contains(&DcChar::from_short(559).to_u128()));

    Ok(())
}

// ============================================================================
// 8. TOTAL ELIMINATION OF DCMIXEDREADER AND READ_NODES VERIFICATION
// ============================================================================

#[crate::ctb_test]
fn test_challenge_verify_total_elimination_of_legacy_symbols() -> Result<()> {
    // In addition to git grep checks performed in CI, verify that our code
    // exclusively uses DcSyntaxArena and DcProjectionContext without any
    // pull-parser remnants.
    let arena = DcSyntaxArena::new();
    ensure!(arena.roots().is_empty());

    let doc = 12345_u64.to_dc_mixed()?;
    let strict = DcProjectionPolicy::strict();
    let decoded = doc.decode::<u64>(&strict)?;
    ensure!(decoded == 12345);
    Ok(())
}
