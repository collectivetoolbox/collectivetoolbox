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

//! Adversarial Stress Test Suite for Milestone 1:
//! - Deeply nested containers, empty containers, unclosed delimiters
//! - Frame isolation: outer containers do not bleed into inner frames
//! - Format delimiter handling: format rules match only internal payload without eagerly consuming outer 512

#[expect(unused_imports, clippy::wildcard_imports, reason = "Integration test root")]
use ctb_utilities::*;

use anyhow::{Context, Result, ensure};
use ctb_formats_checksum::sha256;
use ctb_formats_dcdata::dc::{
    DC_BEGIN_CHECKSUMMED_SECTION, DC_BEGIN_ERROR_RECORD, DC_BEGIN_FORMAT_REPRESENTATION,
    DC_BEGIN_KV_MAP, DC_BEGIN_KV_PAIR, DC_BEGIN_LEGACY_KV_MAP, DC_BEGIN_LIST,
    DC_BEGIN_NUMBER, DC_BEGIN_OPERATION_SESSION, DC_BEGIN_WARNING_RECORD,
    DC_BOOLEAN_FALSE, DC_BOOLEAN_TRUE, DC_END_CHECKSUMMED_SECTION,
    DC_END_ERROR_RECORD, DC_END_FORMAT_REPRESENTATION, DC_END_KV_MAP, DC_END_KV_PAIR,
    DC_END_LEGACY_KV_MAP, DC_END_LIST, DC_END_NUMBER, DC_END_OPERATION_SESSION,
    DC_END_WARNING_RECORD, DC_LIST_ELEMENT_SEPARATOR, DC_SECTION_DIGEST,
};
use ctb_formats_dcdata::syntax::{
    CharTarget, DatasetRuleResolver, SyntaxPattern, SyntaxRuleResolver, SyntaxTerm,
    default_syntax_resolver, parse_dc_syntax,
};
use ctb_formats_dcstring::dc_mixed_serde::encode_format_representation;
use ctb_formats_dcstring::{
    DcByteSpan, DcChar, DcDocumentNavigator, DcFrameHeaderInfo, DcFrameStub,
    DcMixedEncode, DcMstring, DcProjectionPolicy, DcStr, DcString, DcSyntaxArena,
    DcSyntaxKind, SyntaxEvalContext, validate_dc_mixed_strict, validate_dc_tokens_strict,
};

const DC_TYPE_WILDCARD: DcChar = DcChar::from_short(275);

fn sdc(id: u32) -> DcChar {
    DcChar::from_short(id)
}

fn fmt(id: u32) -> DcChar {
    DcChar::from_format(id)
}

fn create_mstring(chars: &[DcChar]) -> DcMstring {
    let mut mst = DcMstring::new();
    for &ch in chars {
        mst.push_char(ch);
    }
    mst
}

// ============================================================================
// 1. NESTED CONTAINERS AND FRAME ISOLATION
// ============================================================================

#[crate::ctb_test]
fn test_challenge_shallow_scan_depth_budget() -> Result<()> {
    let doc = create_mstring(&[
        DC_BEGIN_OPERATION_SESSION,
        DC_BEGIN_OPERATION_SESSION,
        DC_BEGIN_OPERATION_SESSION,
        DC_BOOLEAN_TRUE,
        DC_END_OPERATION_SESSION,
        DC_END_OPERATION_SESSION,
        DC_END_OPERATION_SESSION,
    ]);
    for budget in [0, 1, 2, 3] {
        let policy = DcProjectionPolicy::strict()
            .with_max_depth(budget, ctb_formats_dcstring::DiagnosticAction::Error);
        let mut nav = DcDocumentNavigator::from_mstr(&doc);
        ensure!(nav.next_sibling_stub(&policy).is_ok() == (budget == 3), "Shallow scan ignored budget {budget}");
    }
    Ok(())
}

#[crate::ctb_test]
fn test_challenge_opaque_comment_payload_framing() -> Result<()> {
    use std::io::{Seek, SeekFrom, Write};

    let policy = DcProjectionPolicy::strict();
    for payload in [vec![sdc(317)], vec![sdc(317), sdc(255), sdc(248), sdc(260), sdc(6)]] {
                for nested in [false, true] {
                    let mut chars = vec![sdc(246)];
                    chars.extend(payload.iter().copied());
                    chars.extend([sdc(248), DC_BOOLEAN_TRUE]);
            if nested {
                        chars.insert(0, DC_BEGIN_OPERATION_SESSION);
                        chars.push(DC_END_OPERATION_SESSION);
                    }
                    let doc = create_mstring(&chars);
                    let mut text = DcString::new();
                    for ch in chars {
                        text.push(ch);
                    }
                    let mut file = tempfile::tempfile()?;
                    file.write_all(doc.as_ref())?;
                    file.seek(SeekFrom::Start(0))?;
                    for mut nav in [
                        DcDocumentNavigator::from_mstr(&doc),
                        DcDocumentNavigator::from_dc_str(&text),
                        DcDocumentNavigator::from_dcmm_file(file, 0)?,
                    ] {
                        if nested {
                            let session = nav.next_sibling_stub(&policy)?.context("Expected session")?;
                            nav.enter_frame(&session)?;
                        }
                        let comment = nav.next_sibling_stub(&policy)?.context("Expected opaque comment")?;
                        ensure!(comment.head_dc == sdc(246) && comment.closer_dc == Some(sdc(248)));
                        ensure!(!comment.is_unclosed());
                        ensure!(nav.next_sibling_stub(&policy)?.context("Expected sentinel")?.head_dc == DC_BOOLEAN_TRUE);
                        ensure!(nav.next_sibling_stub(&policy)?.is_none());
            }
                }
    }
    Ok(())
}

#[crate::ctb_test]
fn test_challenge_deeply_nested_containers_and_bounds() -> Result<()> {
    let policy = DcProjectionPolicy::strict();

    // Build 5-level nested containers:
    // Level 0 (root): Session 547 .. 548
    //   Level 1: KvMap 559 [275] [275] .. 560
    //     Level 2: KvPair 514 .. 515
    //       Level 3: List 310 [275] .. 311
    //         Level 4: FormatRepresentation 511 [f228] "nested.txt" 512
    let mut doc = DcMstring::new();
    doc.push_char(DC_BEGIN_OPERATION_SESSION); // 547

    doc.push_char(DC_BEGIN_KV_MAP); // 559
    doc.push_char(DC_TYPE_WILDCARD);
    doc.push_char(DC_TYPE_WILDCARD);

    doc.push_char(DC_BEGIN_KV_PAIR); // 514
    doc.push_char(sdc(107)); // key
    doc.push_char(DC_LIST_ELEMENT_SEPARATOR); // 312

    doc.push_char(DC_BEGIN_LIST); // 310
    doc.push_char(DC_TYPE_WILDCARD);

    encode_format_representation(&"nested.txt".to_string(), fmt(228), &mut doc, &Default::default())?;

    doc.push_char(DC_END_LIST); // 311
    doc.push_char(DC_END_KV_PAIR); // 515
    doc.push_char(DC_END_KV_MAP); // 560
    doc.push_char(DC_END_OPERATION_SESSION); // 548

    // Strict validation must succeed
    validate_dc_mixed_strict(&doc)?;

    let mut nav = DcDocumentNavigator::from_mstr(&doc);

    // Level 0 -> Level 1 (Session)
    let session_stub = nav.next_sibling_stub(&policy)?.context("Expected session")?;
    ensure!(session_stub.head_dc == DC_BEGIN_OPERATION_SESSION);
    ensure!(session_stub.closer_dc == Some(DC_END_OPERATION_SESSION));
    let root_span = nav.active_span();
    nav.enter_frame(&session_stub)?;
    ensure!(nav.depth() == 1);
    let session_inner = nav.active_span();

    // Verify seeking outside session_inner fails
    ensure!(nav.seek_to_offset(root_span.end).is_err(), "Must reject seeking outside active span");

    // Level 1 -> Level 2 (KvMap)
    let map_stub = nav.next_sibling_stub(&policy)?.context("Expected kv map")?;
    ensure!(map_stub.head_dc == DC_BEGIN_KV_MAP);
    nav.enter_frame(&map_stub)?;
    ensure!(nav.depth() == 2);
    let map_inner = nav.active_span();

    // Skip map type wildcards
    let _ = nav.next_sibling_stub(&policy)?;
    let _ = nav.next_sibling_stub(&policy)?;

    // Level 2 -> Level 3 (KvPair)
    let pair_stub = nav.next_sibling_stub(&policy)?.context("Expected kv pair")?;
    ensure!(pair_stub.head_dc == DC_BEGIN_KV_PAIR);
    nav.enter_frame(&pair_stub)?;
    ensure!(nav.depth() == 3);
    let pair_inner = nav.active_span();

    // Key and separator
    let _key = nav.next_sibling_stub(&policy)?;
    let _sep = nav.next_sibling_stub(&policy)?;

    // Level 3 -> Level 4 (List)
    let list_stub = nav.next_sibling_stub(&policy)?.context("Expected list")?;
    ensure!(list_stub.head_dc == DC_BEGIN_LIST);
    nav.enter_frame(&list_stub)?;
    ensure!(nav.depth() == 4);
    let list_inner = nav.active_span();

    // Skip list type wildcard
    let _ = nav.next_sibling_stub(&policy)?;

    // Level 4 -> Level 5 (FormatRepresentation)
    let fmt_stub = nav.next_sibling_stub(&policy)?.context("Expected format rep")?;
    ensure!(fmt_stub.head_dc == DC_BEGIN_FORMAT_REPRESENTATION);
    nav.enter_frame(&fmt_stub)?;
    ensure!(nav.depth() == 5);
    let fmt_inner = nav.active_span();

    // Inner frame bounds verification:
    // Format representation interior must end at its 512, NOT bleed into list 311 or session 548!
    ensure!(fmt_inner.end < list_inner.end, "fmt_inner must be strictly smaller than list_inner");
    ensure!(list_inner.end < pair_inner.end, "list_inner must be strictly smaller than pair_inner");
    ensure!(pair_inner.end < map_inner.end, "pair_inner must be strictly smaller than map_inner");
    ensure!(map_inner.end < session_inner.end, "map_inner must be strictly smaller than session_inner");

    // Reading format tag and payload inside format rep
    let fmt_token_stub = nav.next_sibling_stub(&policy)?.context("Expected format tag stub")?;
    ensure!(fmt_token_stub.head_dc == fmt(228), "First token in format rep must be fmt(228)");
    let payload_stub = nav.next_sibling_stub(&policy)?.context("Expected payload stub")?;
    ensure!(payload_stub.head_dc == sdc(260), "Payload in format rep must be string literal (260)");
    // Format rep should now have no more siblings
    ensure!(nav.next_sibling_stub(&policy)?.is_none(), "No more siblings in format rep");

    // Unwind all frames one by one and check cursor and depth at each level
    nav.exit_frame()?;
    ensure!(nav.depth() == 4);
    ensure!(nav.position() == fmt_stub.outer_span.end);
    ensure!(nav.next_sibling_stub(&policy)?.is_none(), "No more siblings in list");

    nav.exit_frame()?;
    ensure!(nav.depth() == 3);
    ensure!(nav.position() == list_stub.outer_span.end);
    ensure!(nav.next_sibling_stub(&policy)?.is_none(), "No more siblings in pair");

    nav.exit_frame()?;
    ensure!(nav.depth() == 2);
    ensure!(nav.position() == pair_stub.outer_span.end);
    ensure!(nav.next_sibling_stub(&policy)?.is_none(), "No more siblings in map");

    nav.exit_frame()?;
    ensure!(nav.depth() == 1);
    ensure!(nav.position() == map_stub.outer_span.end);
    ensure!(nav.next_sibling_stub(&policy)?.is_none(), "No more siblings in session");

    nav.exit_frame()?;
    ensure!(nav.depth() == 0);
    ensure!(nav.position() == session_stub.outer_span.end);
    ensure!(nav.next_sibling_stub(&policy)?.is_none(), "No more siblings at root");

    Ok(())
}

// ============================================================================
// 2. EMPTY CONTAINERS STRESS TESTING
// ============================================================================

#[crate::ctb_test]
fn test_challenge_empty_containers_all_types() -> Result<()> {
    let policy = DcProjectionPolicy::forgiving();

    let empty_pairs = [
        (DC_BEGIN_FORMAT_REPRESENTATION, DC_END_FORMAT_REPRESENTATION, "FormatRepresentation"),
        (DC_BEGIN_LIST, DC_END_LIST, "List"),
        (DC_BEGIN_KV_MAP, DC_END_KV_MAP, "KvMap"),
        (DC_BEGIN_LEGACY_KV_MAP, DC_END_LEGACY_KV_MAP, "LegacyKvMap"),
        (DC_BEGIN_KV_PAIR, DC_END_KV_PAIR, "KvPair"),
        (DC_BEGIN_OPERATION_SESSION, DC_END_OPERATION_SESSION, "OperationSession"),
        (DC_BEGIN_ERROR_RECORD, DC_END_ERROR_RECORD, "ErrorRecord"),
        (DC_BEGIN_WARNING_RECORD, DC_END_WARNING_RECORD, "WarningRecord"),
    ];

    for (opener, closer, name) in empty_pairs {
        // Build document with empty container followed by a sentinel boolean true
        let doc = create_mstring(&[opener, closer, DC_BOOLEAN_TRUE]);
        ensure!(validate_dc_mixed_strict(&doc).is_err(), "{name}: strict parsing must enforce required contents");
        let mut nav = DcDocumentNavigator::from_mstr(&doc);

        let stub = nav
            .next_sibling_stub(&policy)?
            .with_context(|| format!("{name}: stub expected"))?;
        ensure!(stub.head_dc == opener, "{name}: head matches");
        ensure!(stub.closer_dc == Some(closer), "{name}: closer matches");
        ensure!(!stub.is_unclosed(), "{name}: not unclosed");

        let inner = stub.inner_span.context("expected inner span")?;
        ensure!(inner.start == inner.end, "{name}: inner span length must be 0, got {:?}", inner);

        // Enter empty frame
        nav.enter_frame(&stub)?;
        ensure!(nav.depth() == 1, "{name}: depth 1");
        ensure!(nav.next_sibling_stub(&policy)?.is_none(), "{name}: empty container has 0 siblings");

        // Exit empty frame
        nav.exit_frame()?;
        ensure!(nav.depth() == 0, "{name}: depth 0");
        ensure!(nav.position() == stub.outer_span.end, "{name}: cursor after closer");

        // Read sentinel sibling
        let sentinel = nav
            .next_sibling_stub(&policy)?
            .context("Sentinel stub expected after empty container")?;
        ensure!(sentinel.head_dc == DC_BOOLEAN_TRUE, "Sentinel must be DC_BOOLEAN_TRUE");
    }

    Ok(())
}

// ============================================================================
// 3. UNCLOSED DELIMITERS AND BLEEDING STRESS TESTING
// ============================================================================

#[crate::ctb_test]
fn test_challenge_unclosed_inner_container_strict_rejection() -> Result<()> {
    // Inner container unclosed before outer closer:
    // List (310) containing KvPair (514) that is NOT closed with 515 before List closer (311)
    let doc = create_mstring(&[
        DC_BEGIN_LIST,
        DC_TYPE_WILDCARD,
        DC_BEGIN_KV_PAIR,
        sdc(107),
        DC_LIST_ELEMENT_SEPARATOR,
        sdc(118),
        // missing 515!
        DC_END_LIST,
    ]);

    // Strict validation must reject this malformed framing
    ensure!(
        validate_dc_mixed_strict(&doc).is_err(),
        "Strict validation must reject unclosed inner container before outer closer"
    );

    // Forgiving policy must synchronize at outer closer
    let forgiving = DcProjectionPolicy::forgiving();
    let mut nav = DcDocumentNavigator::from_mstr(&doc);
    let list_stub = nav
        .next_sibling_stub(&forgiving)?
        .context("Forgiving mode must recover list stub")?;
    ensure!(list_stub.head_dc == DC_BEGIN_LIST);
    ensure!(list_stub.closer_dc == Some(DC_END_LIST));

    Ok(())
}

#[crate::ctb_test]
fn test_challenge_dangling_closers_rejected() -> Result<()> {
    let closers = [
        DC_END_FORMAT_REPRESENTATION, // 512
        DC_END_LIST,                  // 311
        DC_END_KV_MAP,               // 560
        DC_END_LEGACY_KV_MAP,        // 314
        DC_END_KV_PAIR,              // 515
        DC_END_OPERATION_SESSION,     // 548
        DC_END_CHECKSUMMED_SECTION,   // 544
        DC_END_ERROR_RECORD,         // 553
        DC_END_WARNING_RECORD,       // 555
        DC_END_NUMBER,               // 7
    ];

    for closer in closers {
        let doc = create_mstring(&[closer]);
        ensure!(
            validate_dc_mixed_strict(&doc).is_err(),
            "Dangling closer {:?} must be rejected by strict validation",
            closer
        );

        let policy = DcProjectionPolicy::strict();
        let mut nav = DcDocumentNavigator::from_mstr(&doc);
        ensure!(
            nav.next_sibling_stub(&policy).is_err(),
            "Dangling closer {:?} must cause error in strict navigator",
            closer
        );
    }

    Ok(())
}

// ============================================================================
// 4. FORMAT DELIMITER HANDLING: NO EAGER 512 CONSUMPTION
// ============================================================================

#[crate::ctb_test]
fn test_challenge_format_delimiters_followed_by_various_tokens() -> Result<()> {
    let policy = DcProjectionPolicy::strict();
    let fmt_228 = fmt(228);

    // Case 1: Format representation followed by boolean true
    let mut doc1 = DcMstring::new();
    encode_format_representation(&"file_one.txt".to_string(), fmt_228, &mut doc1, &Default::default())?;
    doc1.push_char(DC_BOOLEAN_TRUE);

    validate_dc_mixed_strict(&doc1)?;

    let mut nav1 = DcDocumentNavigator::from_mstr(&doc1);
    let fmt_stub1 = nav1.next_sibling_stub(&policy)?.context("Expected format rep")?;
    ensure!(fmt_stub1.head_dc == DC_BEGIN_FORMAT_REPRESENTATION);
    ensure!(fmt_stub1.closer_dc == Some(DC_END_FORMAT_REPRESENTATION));

    let bool_stub1 = nav1.next_sibling_stub(&policy)?.context("Expected bool after format rep")?;
    ensure!(bool_stub1.head_dc == DC_BOOLEAN_TRUE);
    ensure!(nav1.next_sibling_stub(&policy)?.is_none());

    // Case 2: Three consecutive format representations
    let mut doc2 = DcMstring::new();
    for name in ["alpha.bin", "beta.bin", "gamma.bin"] {
        encode_format_representation(&name.to_string(), fmt_228, &mut doc2, &Default::default())?;
    }

    validate_dc_mixed_strict(&doc2)?;

    let mut nav2 = DcDocumentNavigator::from_mstr(&doc2);
    for expected_name in ["alpha.bin", "beta.bin", "gamma.bin"] {
        let stub = nav2.next_sibling_stub(&policy)?.context("Expected format rep")?;
        ensure!(stub.head_dc == DC_BEGIN_FORMAT_REPRESENTATION);
        ensure!(stub.closer_dc == Some(DC_END_FORMAT_REPRESENTATION));
        // Verify format header info
        if let DcFrameHeaderInfo::FormatRepresentation { format } = stub.header_info {
            ensure!(format == fmt_228);
        } else {
            anyhow::bail!("Header info must be FormatRepresentation");
        }
    }
    ensure!(nav2.next_sibling_stub(&policy)?.is_none());

    // Case 3: Format representation followed by a container (List)
    let mut doc3 = DcMstring::new();
    encode_format_representation(&"container_lead.txt".to_string(), fmt_228, &mut doc3, &Default::default())?;
    doc3.push_char(DC_BEGIN_LIST);
    doc3.push_char(DC_TYPE_WILDCARD);
    doc3.push_char(DC_BOOLEAN_FALSE);
    doc3.push_char(DC_END_LIST);

    validate_dc_mixed_strict(&doc3)?;

    let mut nav3 = DcDocumentNavigator::from_mstr(&doc3);
    let fstub = nav3.next_sibling_stub(&policy)?.context("Expected format stub")?;
    ensure!(fstub.head_dc == DC_BEGIN_FORMAT_REPRESENTATION);
    let lstub = nav3.next_sibling_stub(&policy)?.context("Expected list stub")?;
    ensure!(lstub.head_dc == DC_BEGIN_LIST);
    ensure!(nav3.next_sibling_stub(&policy)?.is_none());

    Ok(())
}

#[crate::ctb_test]
fn test_challenge_format_rule_eval_does_not_consume_trailing_512() -> Result<()> {
    let resolver = default_syntax_resolver();
    let target = CharTarget::Format(FormatId::FileName);
    let _rule = resolver.resolve_rule(&target).context("Rule for f228 must exist")?;

    // Create a stream with: format payload followed by 512 followed by 517 (true)
    let mut doc = DcMstring::new();
    // Encode string literal payload for FileName: 260 262 264 263 ... 261
    let literal_chars = [
        sdc(260),
        sdc(262),
        sdc(264),
        sdc(263),
        sdc(65),
        sdc(66),
        sdc(261),
    ];
    for ch in literal_chars {
        doc.push_char(ch);
    }
    let payload_end_pos = doc.as_bytes().len();

    // Now append 512 (DC_END_FORMAT_REPRESENTATION) and 517 (DC_BOOLEAN_TRUE)
    doc.push_char(DC_END_FORMAT_REPRESENTATION);
    doc.push_char(DC_BOOLEAN_TRUE);

    // Evaluate RuleRef { target: Format(228) } on this stream
    let term = SyntaxTerm::RuleRef { target: target.clone() };
    let mut nav = DcDocumentNavigator::from_mstr(&doc);
    let mut ctx = SyntaxEvalContext::new(&**resolver, None);

    let matched = nav.eval_term_single(&term, &mut ctx)?;
    ensure!(matched, "Format rule must match payload");
    ensure!(!ctx.has_errors, "Context must have no errors");

    // CRITICAL ASSERTION: The navigator cursor MUST be positioned exactly at payload_end_pos!
    // It must NOT have consumed DC_END_FORMAT_REPRESENTATION (512)!
    ensure!(
        nav.position() == u64::try_from(payload_end_pos)?,
        "eval_term_single for format rule must NOT eagerly consume trailing Dc 512! Expected cursor at {payload_end_pos}, got {}",
        nav.position()
    );

    // Now the next token read by the navigator MUST be DC_END_FORMAT_REPRESENTATION!
    let forgiving = DcProjectionPolicy::forgiving();
    let next_stub = nav
        .next_sibling_stub(&forgiving)?
        .context("Expected next stub at cursor")?;
    ensure!(
        next_stub.head_dc == DC_END_FORMAT_REPRESENTATION,
        "Next token after format rule must be DC_END_FORMAT_REPRESENTATION (512), got {:?}",
        next_stub.head_dc
    );

    Ok(())
}

// ============================================================================
// 5. SYNTAX ARENA AST INTEGRITY ON FRAMES
// ============================================================================

#[crate::ctb_test]
fn test_challenge_syntax_arena_nested_ast_integrity() -> Result<()> {
    let policy = DcProjectionPolicy::strict();

    // Build List of two elements: a format representation and a boolean
    let mut doc = DcMstring::new();
    doc.push_char(DC_BEGIN_LIST);
    doc.push_char(DC_TYPE_WILDCARD);

    encode_format_representation(&"sample.txt".to_string(), fmt(228), &mut doc, &Default::default())?;
    doc.push_char(DC_LIST_ELEMENT_SEPARATOR);
    doc.push_char(DC_BOOLEAN_TRUE);
    doc.push_char(DC_END_LIST);

    let mut nav = DcDocumentNavigator::from_mstr(&doc);
    let mut arena = DcSyntaxArena::new();
    nav.parse_document_into_arena(&mut arena, &policy)?;

    // Verify arena roots
    let roots = arena.roots();
    ensure!(roots.len() == 1, "Expected 1 root node in arena");
    let root_id = roots[0];
    let root = arena.node(root_id).context("Root node must exist")?;

    ensure!(
        matches!(&root.kind, DcSyntaxKind::Container { opener, closer, .. } if *opener == DC_BEGIN_LIST && *closer == DC_END_LIST),
        "Root node must be List container"
    );

    // List should have children
    let children = root.children();
    ensure!(children.len() >= 3, "List should have type, format rep, and boolean children");

    // Clear arena for reuse (validating O(1) clear lifecycle)
    arena.clear();
    ensure!(arena.nodes().is_empty());
    ensure!(arena.roots().is_empty());

    Ok(())
}

// ============================================================================
// 6. NAMED TYPES AND FORMAT EVALUATION (:511 [fXXX:] 512)
// ============================================================================

#[crate::ctb_test]
fn test_challenge_named_types_with_format_rules() -> Result<()> {
    let resolver = default_syntax_resolver();

    // Verify named types are registered
    let filename_pat = resolver.resolve_named_type("filename").context("filename named type")?;
    let filepath_pat = resolver.resolve_named_type("filepath").context("filepath named type")?;
    let ipaddr_pat = resolver.resolve_named_type("ipaddr").context("ipaddr named type")?;

    // 1. filename: 511 [f228:] 512
    let mut doc_fn = DcMstring::new();
    encode_format_representation(&"document.pdf".to_string(), fmt(228), &mut doc_fn, &Default::default())?;
    let mut nav_fn = DcDocumentNavigator::from_mstr(&doc_fn);
    let mut ctx_fn = SyntaxEvalContext::new(&**resolver, None);
    let matched_fn = nav_fn.eval_pattern(&filename_pat, &mut ctx_fn)?;
    ensure!(matched_fn, "filename rule must match 511 [f228: ...] 512");
    ensure!(!ctx_fn.has_errors, "no errors in eval");
    ensure!(nav_fn.position() == u64::try_from(doc_fn.as_bytes().len())?, "must consume entire representation");

    // 2. Missing 512 on filename must FAIL
    let mut doc_fn_truncated = DcMstring::new();
    doc_fn_truncated.push_char(DC_BEGIN_FORMAT_REPRESENTATION);
    doc_fn_truncated.push_char(fmt(228));
    "document.pdf".to_string().encode_dc_mixed(&mut doc_fn_truncated)?;
    // Omitting DC_END_FORMAT_REPRESENTATION
    let mut nav_fn_trunc = DcDocumentNavigator::from_mstr(&doc_fn_truncated);
    let mut ctx_fn_trunc = SyntaxEvalContext::new(&**resolver, None);
    let matched_fn_trunc = nav_fn_trunc.eval_pattern(&filename_pat, &mut ctx_fn_trunc)?;
    ensure!(!matched_fn_trunc, "filename rule must FAIL when 512 is missing");

    // 3. filename followed by another token: 511 [f228: ...] 512 DC_BOOLEAN_TRUE
    // Format rule must match 511..512 and leave cursor at DC_BOOLEAN_TRUE
    let mut doc_fn_seq = doc_fn.clone();
    doc_fn_seq.push_char(DC_BOOLEAN_TRUE);
    let mut nav_fn_seq = DcDocumentNavigator::from_mstr(&doc_fn_seq);
    let mut ctx_fn_seq = SyntaxEvalContext::new(&**resolver, None);
    let matched_fn_seq = nav_fn_seq.eval_pattern(&filename_pat, &mut ctx_fn_seq)?;
    ensure!(matched_fn_seq, "filename rule matches prefix");
    ensure!(nav_fn_seq.position() == u64::try_from(doc_fn.as_bytes().len())?, "must stop before boolean true");

    // 4. ipaddr: :511 [f226:] 512 where f226 is ([f490:] | [f491:]) and f490 is [number:f207]
    // A valid ipaddr token sequence: 511, fmt(490), 6, fmt(199), 127, 7, 512
    let doc_ip = create_mstring(&[
        DC_BEGIN_FORMAT_REPRESENTATION, // 511
        fmt(490),                      // f490 (matches [f490:] branch of f226)
        DC_BEGIN_NUMBER,                // 6
        fmt(199),
        sdc(127),
        DC_END_NUMBER,                  // 7
        DC_END_FORMAT_REPRESENTATION,   // 512
    ]);
    let mut nav_ip = DcDocumentNavigator::from_mstr(&doc_ip);
    let mut ctx_ip = SyntaxEvalContext::new(&**resolver, None);
    let matched_ip = nav_ip.eval_pattern(&ipaddr_pat, &mut ctx_ip)?;
    ensure!(matched_ip, "ipaddr rule must match 511 [f490: 6 [f199:] 7] 512");
    ensure!(nav_ip.position() == u64::try_from(doc_ip.as_bytes().len())?);

    Ok(())
}

// ============================================================================
// 7. UNCLOSED INNER SUB-STRUCTURES INSIDE CONTAINERS
// ============================================================================

#[crate::ctb_test]
fn test_challenge_unclosed_substructures_inside_containers() -> Result<()> {
    // A: Container containing unclosed literal 260 .. (missing 261) before 311
    let doc_unclosed_lit = create_mstring(&[
        DC_BEGIN_LIST,
        DC_TYPE_WILDCARD,
        sdc(260), // open literal
        sdc(262),
        sdc(264),
        sdc(263),
        sdc(65),
        // missing 261!
        DC_END_LIST,
    ]);
    ensure!(
        validate_dc_mixed_strict(&doc_unclosed_lit).is_err(),
        "Unclosed literal in container must fail strict validation"
    );

    // B: Container containing unclosed number 6 .. (missing 7) before 311
    let doc_unclosed_num = create_mstring(&[
        DC_BEGIN_LIST,
        DC_TYPE_WILDCARD,
        DC_BEGIN_NUMBER, // 6
        fmt(199),
        sdc(127),
        // missing 7!
        DC_END_LIST,
    ]);
    ensure!(
        validate_dc_mixed_strict(&doc_unclosed_num).is_err(),
        "Unclosed number in container must fail strict validation"
    );

    Ok(())
}

// ============================================================================
// 8. MULTI-LEVEL UNCLOSED SYNCHRONIZATION AND RECOVERY
// ============================================================================

#[crate::ctb_test]
fn test_challenge_multi_level_unclosed_synchronization() -> Result<()> {
    // 3 levels of containers all unclosed before outermost session closer (548):
    // Session (547) -> List (310) -> KvMap (559) -> KvPair (514) ... Session closer (548)
    // 514, 559, and 310 are all unclosed!
    let doc = create_mstring(&[
        DC_BEGIN_OPERATION_SESSION, // 547
        DC_BEGIN_LIST,              // 310
        DC_TYPE_WILDCARD,
        DC_BEGIN_KV_MAP,           // 559
        DC_TYPE_WILDCARD,
        DC_TYPE_WILDCARD,
        DC_BEGIN_KV_PAIR,          // 514
        sdc(107),
        DC_LIST_ELEMENT_SEPARATOR,
        sdc(118),
        // All closers (515, 560, 311) are missing!
        DC_END_OPERATION_SESSION,   // 548
    ]);

    // Strict validation must reject this
    ensure!(
        validate_dc_mixed_strict(&doc).is_err(),
        "Multi-level unclosed containers must be rejected by strict validation"
    );

    // Forgiving navigation must pop all unclosed levels and synchronize at 548
    let forgiving = DcProjectionPolicy::forgiving();
    let mut nav = DcDocumentNavigator::from_mstr(&doc);
    let session_stub = nav
        .next_sibling_stub(&forgiving)?
        .context("Forgiving mode must recover session stub")?;
    ensure!(session_stub.head_dc == DC_BEGIN_OPERATION_SESSION);
    ensure!(session_stub.closer_dc == Some(DC_END_OPERATION_SESSION));
    ensure!(nav.position() == session_stub.outer_span.end);
    ensure!(nav.next_sibling_stub(&forgiving)?.is_none(), "No more siblings after session");

    Ok(())
}

// ============================================================================
// 9. BINARY ENCAPSULATION FRAMING BOUNDARIES
// ============================================================================

#[crate::ctb_test]
fn test_challenge_binary_encapsulation_inside_containers() -> Result<()> {
    use ctb_formats_dcstring::dc_mixed::{DC_203_BYTES, TYPE_BINARY};

    // Valid raw binary encapsulation inside a list
    let mut doc = DcMstring::new();
    doc.push_char(DC_BEGIN_LIST);
    doc.push_char(DC_TYPE_WILDCARD);
    doc.push_binary(b"TEST");
    doc.push_char(DC_END_LIST);

    // Strict validation must succeed
    validate_dc_mixed_strict(&doc)?;

    // Navigator must frame binary as a stub with Binary header_info
    let policy = DcProjectionPolicy::strict();
    let mut nav = DcDocumentNavigator::from_mstr(&doc);
    let list_stub = nav.next_sibling_stub(&policy)?.context("list stub")?;
    nav.enter_frame(&list_stub)?;

    let _type_stub = nav.next_sibling_stub(&policy)?; // skip wildcard
    let bin_stub = nav.next_sibling_stub(&policy)?.context("bin stub")?;
    ensure!(bin_stub.head_dc == ctb_formats_dcdata::dc::DC_START_ENCAPSULATION_BINARY);
    ensure!(bin_stub.closer_dc == Some(ctb_formats_dcdata::dc::DC_END_ENCAPSULATION_BINARY));
    if let DcFrameHeaderInfo::Binary { payload_len, .. } = bin_stub.header_info {
        ensure!(payload_len == 4, "Payload length must be 4");
    } else {
        anyhow::bail!("Header info must be Binary");
    }

    ensure!(nav.next_sibling_stub(&policy)?.is_none(), "No more siblings in list");
    nav.exit_frame()?;
    ensure!(nav.position() == list_stub.outer_span.end);

    // Truncated binary inside list: length claims 100 bytes, but stream ends with list closer (311)
    let mut doc_trunc_mst = DcMstring::new();
    doc_trunc_mst.push_char(DC_BEGIN_LIST);
    doc_trunc_mst.push_char(DC_TYPE_WILDCARD);
    let mut doc_trunc_bytes = doc_trunc_mst.into_bytes();

    // Truncated binary: 203, TYPE_BINARY, length 100, but only 5 bytes payload
    doc_trunc_bytes.extend_from_slice(&DC_203_BYTES);
    doc_trunc_bytes.push(TYPE_BINARY);
    doc_trunc_bytes.extend_from_slice(&100u128.to_be_bytes());
    doc_trunc_bytes.extend_from_slice(b"SHORT");
    // Followed by list closer (311)
    doc_trunc_bytes.extend_from_slice(&ctb_formats_utf_8e_128::encode_utf_8e_128(DC_END_LIST.to_u128()));

    // Truncated binary: DcMstring::from_bytes or validate_dc_mixed_strict must reject it!
    let from_bytes_res = ctb_formats_dcstring::DcMstring::from_bytes(doc_trunc_bytes);
    ensure!(
        from_bytes_res.is_err() || from_bytes_res.as_ref().is_ok_and(|doc| validate_dc_mixed_strict(doc).is_err()),
        "Truncated binary encapsulation extending past container must be rejected"
    );

    Ok(())
}




// ============================================================================
// CONSOLIDATED CHALLENGER M1 TESTS
// ============================================================================

mod challenger_framing_and_operators {
    use super::*;
use anyhow::{Context, Result, ensure};
use ctb_formats_dcdata::dc::*;
use ctb_formats_dcdata::syntax::{
    CharTarget, DatasetRuleResolver, SyntaxPattern, SyntaxRuleResolver,
    default_syntax_resolver,
};
use ctb_formats_dcstring::{
    DcByteSpan, DcChar, DcDocumentNavigator, DcFrameHeaderInfo, DcFrameStub,
    DcMstr, DcMstring, DcProjectionPolicy, DcStr, DcString, SyntaxEvalContext,
    dc_mixed, validate_dc_mixed_strict, validate_dc_str_strict,
    validate_dc_tokens_strict,
};

fn sdc(id: u32) -> u128 {
    DcChar::from_short(id).to_u128()
}

fn fmt(id: u32) -> u128 {
    DcChar::from_format(id).to_u128()
}

fn dc_string_from_tokens(tokens: &[u128]) -> DcString {
    let mut s = DcString::with_capacity(tokens.len().saturating_mul(6));
    for &tok in tokens {
        s.push(DcChar::from_u128(tok));
    }
    s
}

/// Helper: creates a `DcMstring` from raw `u128` tokens.
fn mstring_from_tokens(tokens: &[u128]) -> Result<DcMstring> {
    let s = dc_string_from_tokens(tokens);
    DcMstring::from_dc_str(&s)
}

// ============================================================================
// 1. EMPIRICAL VERIFICATION: ZERO BASE64 ALLOCATION IN VALIDATION
// ============================================================================

#[crate::ctb_test]
fn test_challenge_validate_dc_mixed_zero_base64_allocation() -> Result<()> {
    // 1. Build a DcMstring with a large raw binary payload (1 MiB).
    // If validation were converting this to base64, it would format ~1.33 MiB of
    // base64 characters. With raw O(1) header scanning, it must finish
    // virtually instantaneously without heap allocation for base64.
    let payload_size = 1024usize.saturating_mul(1024);
    let mut large_payload = Vec::with_capacity(payload_size);
    for idx in 0..payload_size {
        let byte_val = u8::try_from(idx.wrapping_rem(256)).unwrap_or(0);
        large_payload.push(byte_val);
    }

    // Warm up the static CSV syntax resolver so we benchmark validation,
    // not one-time CSV disk loading.
    let _ = default_syntax_resolver();

    // 1. Bare binary in DcMstring
    let mut bare_mst = DcMstring::new();
    bare_mst.push_binary(&large_payload);

    let t_bare = std::time::Instant::now();
    validate_dc_mixed_strict(&bare_mst)
        .context("Bare raw binary payload in DcMstring must validate strictly")?;
    let elapsed_bare = t_bare.elapsed();
    println!("validate_dc_mixed_strict (bare 1 MiB binary): {:?}", elapsed_bare);

    ensure!(
        elapsed_bare < std::time::Duration::from_millis(50),
        "validate_dc_mixed_strict on bare 1 MiB raw binary took too long ({:?})",
        elapsed_bare
    );

    // 2. Binary in list
    let mut mst = DcMstring::new();
    mst.push_char(DC_BEGIN_LIST);
    mst.push_char(DcChar::from_short(275)); // Type wildcard

    let t0 = std::time::Instant::now();
    mst.push_binary(&large_payload);
    mst.push_char(DC_END_LIST);
    let t_push = t0.elapsed();

    // Validate using validate_dc_mixed_strict
    let t1 = std::time::Instant::now();
    validate_dc_mixed_strict(&mst)
        .context("Large raw binary payload in DcMstring must validate strictly")?;
    let t_validate = t1.elapsed();

    println!("push_binary: {:?}, validate_dc_mixed_strict (in list): {:?}", t_push, t_validate);

    ensure!(
        t_validate < std::time::Duration::from_millis(50),
        "validate_dc_mixed_strict on 1 MiB raw binary took too long ({:?})",
        t_validate
    );

    // 2. Verify with binary with metadata (e.g. SHA-256)
    let mut mst_with_meta = DcMstring::new();
    mst_with_meta.push_char(DC_BEGIN_LIST);
    mst_with_meta.push_char(DcChar::from_short(275));
    mst_with_meta.push_binary_with_sha256(&large_payload)?;
    mst_with_meta.push_char(DC_END_LIST);

    let t2 = std::time::Instant::now();
    validate_dc_mixed_strict(&mst_with_meta)
        .context("Binary with SHA-256 in DcMstring must validate strictly")?;
    let t_validate_meta = t2.elapsed();

    println!("validate_dc_mixed_strict with metadata: {:?}", t_validate_meta);
    ensure!(
        t_validate_meta < std::time::Duration::from_millis(50),
        "validate_dc_mixed_strict with metadata took too long ({:?})",
        t_validate_meta
    );
    Ok(())
}

// ============================================================================
// 2. EMPIRICAL VERIFICATION: STRICT FRAMING REJECTION ACROSS ALL THREE MODES
// ============================================================================

#[crate::ctb_test]
fn test_challenge_framing_rejection_across_all_modes() -> Result<()> {
    // List of invalid token sequences that violate paired container framing:
    let invalid_framing_cases: Vec<(&'static str, Vec<u128>)> = vec![
        ("dangling list closer", vec![sdc(311)]),
        ("dangling format closer", vec![sdc(512)]),
        ("dangling checksummed section closer", vec![sdc(544)]),
        ("dangling session closer", vec![sdc(548)]),
        ("dangling kv map closer", vec![sdc(560)]),
        ("dangling number closer", vec![sdc(7)]),
        ("dangling literal closer", vec![sdc(261)]),
        ("dangling identifier closer", vec![sdc(271)]),
        ("dangling kv pair closer", vec![sdc(515)]),
        // Mismatched container opener / closer
        (
            "mismatched list opener with format closer",
            vec![sdc(310), sdc(275), sdc(512)],
        ),
        (
            "mismatched format opener with list closer",
            vec![sdc(511), fmt(228), 116, 101, 115, 116, sdc(311)],
        ),
        (
            "mismatched session opener with checksummed section closer",
            vec![sdc(547), sdc(270), 99, 115, 99, sdc(271), sdc(544)],
        ),
        (
            "mismatched kv map opener with pair closer",
            vec![sdc(559), sdc(275), sdc(275), sdc(515)],
        ),
        // Unclosed containers at EOF
        (
            "unclosed list at EOF",
            vec![sdc(310), sdc(275), sdc(517)],
        ),
        (
            "unclosed format representation at EOF",
            vec![sdc(511), fmt(228), 116, 101, 115, 116],
        ),
        (
            "unclosed session at EOF",
            vec![sdc(547), sdc(270), 99, 115, 99, sdc(271)],
        ),
        (
            "unclosed checksummed section at EOF",
            vec![sdc(543), sdc(517)],
        ),
        (
            "unclosed kv map at EOF",
            vec![sdc(559), sdc(275), sdc(275)],
        ),
        // Truncated escape token at EOF
        (
            "truncated escape Dc 255 at EOF",
            vec![sdc(310), sdc(275), sdc(255)],
        ),
        // Invalid interior constraint: raw number instead of framed IP address
        (
            "host local IP without format framing",
            vec![sdc(346), sdc(6), fmt(199), sdc(127), sdc(7)],
        ),
        // Invalid interior constraint: raw number instead of date framing
        (
            "date timestamp without 390..391 framing",
            vec![sdc(330), sdc(6), fmt(199), sdc(127), sdc(7)],
        ),
    ];

    for (desc, tokens) in invalid_framing_cases {
        // 1. Must fail validate_dc_tokens_strict
        ensure!(
            validate_dc_tokens_strict(&tokens).is_err(),
            "{desc} must be rejected by validate_dc_tokens_strict"
        );

        // 2. Must fail validate_dc_str_strict
        let dc_str = dc_string_from_tokens(&tokens);
        ensure!(
            validate_dc_str_strict(&dc_str).is_err(),
            "{desc} must be rejected by validate_dc_str_strict"
        );

        // 3. Must fail validate_dc_mixed_strict
        if let Ok(mst) = mstring_from_tokens(&tokens) {
            ensure!(
                validate_dc_mixed_strict(&mst).is_err(),
                "{desc} must be rejected by validate_dc_mixed_strict"
            );
        }
    }

    Ok(())
}

// ============================================================================
// 3. EMPIRICAL VERIFICATION: ACCEPTANCE OF VALID DOCUMENTS IN ALL THREE MODES
// ============================================================================

#[crate::ctb_test]
fn test_challenge_acceptance_across_all_modes() -> Result<()> {
    let valid_documents: Vec<(&'static str, Vec<u128>)> = vec![
        (
            "atomic booleans",
            vec![sdc(517), sdc(518)],
        ),
        (
            "number construct Dc 6..7",
            vec![sdc(6), fmt(199), 49, 50, 51, sdc(7)],
        ),
        (
            "identifier construct Dc 270..271",
            vec![sdc(270), 102, 111, 111, sdc(271)],
        ),
        (
            "literal string construct Dc 260..261",
            vec![sdc(260), sdc(262), sdc(264), sdc(263), 104, 101, 108, 108, 111, sdc(261)],
        ),
        (
            "valid typed list Dc 310..311",
            vec![sdc(310), sdc(275), sdc(517), sdc(312), sdc(518), sdc(311)],
        ),
        (
            "valid format representation Dc 511..512",
            vec![
                sdc(511),
                fmt(228),
                sdc(260),
                sdc(262),
                sdc(264),
                sdc(263),
                102,
                105,
                108,
                101,
                sdc(261),
                sdc(512),
            ],
        ),
        (
            "valid operation session Dc 547..548",
            vec![sdc(547), sdc(270), 99, 115, 99, sdc(271), sdc(548)],
        ),
        (
            "valid kv pair Dc 514..515",
            vec![sdc(514), 107, sdc(312), 118, sdc(515)],
        ),
        (
            "valid kv map Dc 559..560",
            vec![
                sdc(559),
                sdc(275),
                sdc(275),
                sdc(514),
                107,
                sdc(312),
                118,
                sdc(515),
                sdc(560),
            ],
        ),
        (
            "valid host IP Dc 346 with f490 format",
            vec![
                sdc(346),
                sdc(511),
                fmt(490),
                sdc(6),
                fmt(199),
                sdc(127),
                sdc(7),
                sdc(512),
            ],
        ),
        (
            "valid date timestamp Dc 330 with Dc 390..391",
            vec![
                sdc(330),
                sdc(390),
                fmt(487),
                sdc(6),
                fmt(199),
                sdc(127),
                sdc(7),
                sdc(391),
            ],
        ),
    ];

    for (desc, tokens) in valid_documents {
        // 1. Must pass validate_dc_tokens_strict
        validate_dc_tokens_strict(&tokens)
            .with_context(|| format!("{desc} failed validate_dc_tokens_strict"))?;

        // 2. Must pass validate_dc_str_strict
        let dc_str = dc_string_from_tokens(&tokens);
        validate_dc_str_strict(&dc_str)
            .with_context(|| format!("{desc} failed validate_dc_str_strict"))?;

        // 3. Must pass validate_dc_mixed_strict
        let mst = mstring_from_tokens(&tokens)
            .with_context(|| format!("{desc} failed mstring conversion"))?;
        validate_dc_mixed_strict(&mst)
            .with_context(|| format!("{desc} failed validate_dc_mixed_strict"))?;
    }

    Ok(())
}

// ============================================================================
// 4. EMPIRICAL VERIFICATION: PREFIX OPERATOR EVALUATION WITHOUT FIXED ARITY
// ============================================================================

#[crate::ctb_test]
fn test_challenge_prefix_operator_evaluation_data_driven() -> Result<()> {
    let resolver = default_syntax_resolver();
    let policy = DcProjectionPolicy::strict();

    // A. Prefix operator Dc 330 (Date timestamp): rule defines internal syntax
    // Pattern: :~ [date $date]
    let target_330 = CharTarget::Dc(330);
    let rule_330 = resolver
        .resolve_rule(&target_330)
        .context("Resolver must find rule for Dc 330")?;
    ensure!(
        rule_330.pattern.starts_with_self(),
        "Rule 330 must start with self"
    );

    // Build tokens for 330 with valid date payload:
    let date_tokens = vec![
        sdc(330),
        sdc(390),
        fmt(487),
        sdc(6),
        fmt(199),
        sdc(127),
        sdc(7),
        sdc(391),
    ];
    let date_str = dc_string_from_tokens(&date_tokens);
    let mut nav = DcDocumentNavigator::from_dc_str(&date_str);
    let stub = nav
        .next_sibling_stub(&policy)?
        .context("Navigator must produce stub for prefix operator 330")?;
    ensure!(stub.head_dc == DcChar::from_short(330), "Stub head must be 330");
    ensure!(
        nav.position() == u64::try_from(date_str.len())?,
        "Navigator must advance exactly to end of prefix rule construct"
    );

    // B. Prefix operator followed by another token: verify NO greediness/overconsumption
    let mut multi_tokens = date_tokens.clone();
    let trailing_bool = sdc(517); // Boolean true
    multi_tokens.push(trailing_bool);

    let multi_str = dc_string_from_tokens(&multi_tokens);
    let mut multi_nav = DcDocumentNavigator::from_dc_str(&multi_str);

    let stub1 = multi_nav
        .next_sibling_stub(&policy)?
        .context("First stub must be prefix operator 330")?;
    ensure!(
        stub1.head_dc == DcChar::from_short(330),
        "First stub head must be 330"
    );
    ensure!(
        multi_nav.position() == u64::try_from(date_str.len())?,
        "Position after prefix stub must NOT consume trailing sibling token"
    );

    let stub2 = multi_nav
        .next_sibling_stub(&policy)?
        .context("Second stub must be trailing boolean token")?;
    ensure!(
        stub2.head_dc == DC_BOOLEAN_TRUE,
        "Second stub must be DC_BOOLEAN_TRUE"
    );

    // C. Host IP address prefix operator Dc 346: :~ [ipaddr]
    let target_346 = CharTarget::Dc(346);
    let rule_346 = resolver
        .resolve_rule(&target_346)
        .context("Resolver must find rule for Dc 346")?;
    ensure!(
        rule_346.pattern.starts_with_self(),
        "Rule 346 must start with self"
    );

    let ip_tokens = vec![
        sdc(346),
        sdc(511),
        fmt(490),
        sdc(6),
        fmt(199),
        sdc(127),
        sdc(7),
        sdc(512),
        sdc(518), // Followed by boolean false
    ];
    let ip_str = dc_string_from_tokens(&ip_tokens);
    let mut ip_nav = DcDocumentNavigator::from_dc_str(&ip_str);

    let ip_stub = ip_nav
        .next_sibling_stub(&policy)?
        .context("Navigator must produce stub for prefix 346")?;
    ensure!(
        ip_stub.head_dc == DcChar::from_short(346),
        "IP stub head must be 346"
    );

    let next_stub = ip_nav
        .next_sibling_stub(&policy)?
        .context("Following stub must be boolean false")?;
    ensure!(
        next_stub.head_dc == DC_BOOLEAN_FALSE,
        "Next stub must be DC_BOOLEAN_FALSE"
    );

    // D. Prefix operator with missing arguments: must fail evaluation gracefully
    let incomplete_ip = vec![sdc(346)];
    let incomplete_ip_str = dc_string_from_tokens(&incomplete_ip);
    let mut incomplete_nav = DcDocumentNavigator::from_dc_str(&incomplete_ip_str);
    ensure!(
        incomplete_nav.next_sibling_stub(&policy).is_err(),
        "Prefix operator with missing arguments must error under strict policy"
    );

    Ok(())
}

// ============================================================================
// 5. EMPIRICAL VERIFICATION: ADJACENT UNFRAMED & FRAMED CONSTRUCTS (NO 512 HACK)
// ============================================================================

#[crate::ctb_test]
fn test_challenge_adjacent_format_representations_no_512_consumption() -> Result<()> {
    let policy = DcProjectionPolicy::strict();

    // Construct: [310] [*] [511] [f228] "f1" [512] [312] [511] [f228] "f2" [512] [311]
    let tokens = vec![
        sdc(310),
        sdc(275), // Wildcard
        sdc(511),
        fmt(228),
        sdc(260),
        sdc(262),
        sdc(264),
        sdc(263),
        102,
        49,
        sdc(261),
        sdc(512),
        sdc(312), // Separator
        sdc(511),
        fmt(228),
        sdc(260),
        sdc(262),
        sdc(264),
        sdc(263),
        102,
        50,
        sdc(261),
        sdc(512),
        sdc(311), // End list
    ];

    validate_dc_tokens_strict(&tokens)
        .context("Adjacent format representations in list must pass validate_dc_tokens_strict")?;

    let doc_str = dc_string_from_tokens(&tokens);
    validate_dc_str_strict(&doc_str)
        .context("Adjacent format representations in list must pass validate_dc_str_strict")?;

    let mst = mstring_from_tokens(&tokens)?;
    validate_dc_mixed_strict(&mst)
        .context("Adjacent format representations in list must pass validate_dc_mixed_strict")?;

    // Step through with navigator
    let mut nav = DcDocumentNavigator::from_dc_str(&doc_str);
    let list_stub = nav
        .next_sibling_stub(&policy)?
        .context("Outer list stub must be present")?;
    nav.enter_frame(&list_stub)?;

    let _type_stub = nav.next_sibling_stub(&policy)?; // skip type wildcard

    let fmt1 = nav
        .next_sibling_stub(&policy)?
        .context("First format stub must be present")?;
    ensure!(
        fmt1.head_dc == DC_BEGIN_FORMAT_REPRESENTATION,
        "First child must be 511"
    );
    ensure!(
        fmt1.closer_dc == Some(DC_END_FORMAT_REPRESENTATION),
        "First child closer must be 512"
    );

    let sep = nav
        .next_sibling_stub(&policy)?
        .context("Separator stub must be present")?;
    ensure!(
        sep.head_dc == DC_LIST_ELEMENT_SEPARATOR,
        "Separator must be 312"
    );

    let fmt2 = nav
        .next_sibling_stub(&policy)?
        .context("Second format stub must be present")?;
    ensure!(
        fmt2.head_dc == DC_BEGIN_FORMAT_REPRESENTATION,
        "Second child must be 511"
    );
    ensure!(
        fmt2.closer_dc == Some(DC_END_FORMAT_REPRESENTATION),
        "Second child closer must be 512"
    );

    ensure!(
        nav.next_sibling_stub(&policy)?.is_none(),
        "No further siblings in list"
    );

    nav.exit_frame()?;
    ensure!(
        nav.next_sibling_stub(&policy)?.is_none(),
        "No further siblings at root"
    );

    Ok(())
}

// ============================================================================
// 6. ADVERSARIAL: CORRUPTED BINARY CHUNK BOUNDS AND METADATA TAMPERING
// ============================================================================

#[crate::ctb_test]
fn test_challenge_adversarial_corrupted_binary_chunk_bounds() -> Result<()> {
    // A. Binary chunk header claims size larger than actual byte stream length
    let mut mst = DcMstring::new();
    mst.push_char(DC_BEGIN_LIST);
    mst.push_char(DcChar::from_short(275)); // Type wildcard
    mst.push_binary(&[1, 2, 3, 4]); // 4 bytes
    mst.push_char(DC_END_LIST);

    // Tamper with the binary chunk size field (bytes 7..23) to claim 0xFFFFFFFF bytes
    let mut tampered_bytes = mst.as_bytes().to_vec();
    // Locate the DC_203_BYTES
    let dc203_pos = tampered_bytes
        .windows(dc_mixed::DC_203_BYTES.len())
        .position(|window| window == dc_mixed::DC_203_BYTES)
        .context("Missing DC_203_BYTES in serialized DcMstring")?;

    let size_start = dc203_pos
        .checked_add(dc_mixed::DC_203_BYTES.len())
        .and_then(|p| p.checked_add(1)) // Skip type byte
        .context("Size offset overflow")?;

    // Corrupt size to 99999999 (extends far past stream end)
    let bad_size = 99_999_999u128.to_be_bytes();
    for (i, &b) in bad_size.iter().enumerate() {
        if let Some(target) = tampered_bytes.get_mut(size_start.saturating_add(i)) {
            *target = b;
        }
    }

    let res = match DcMstr::from_bytes(&tampered_bytes) {
        Ok(mstr) => validate_dc_mixed_strict(mstr),
        Err(e) => Err(anyhow::anyhow!(e)),
    };
    ensure!(
        res.is_err(),
        "Binary chunk claiming length exceeding stream length must fail validation"
    );

    // B. Binary chunk claiming length that extends past its enclosing container
    let mut mst2 = DcMstring::new();
    mst2.push_char(DC_BEGIN_LIST);
    mst2.push_char(DcChar::from_short(275));
    mst2.push_binary(&[10, 20, 30, 40]);
    mst2.push_char(DC_END_LIST);
    mst2.push_char(DC_BOOLEAN_TRUE);

    let mut tampered_bytes2 = mst2.as_bytes().to_vec();
    let dc203_pos2 = tampered_bytes2
        .windows(dc_mixed::DC_203_BYTES.len())
        .position(|window| window == dc_mixed::DC_203_BYTES)
        .context("Missing DC_203_BYTES")?;
    let size_start2 = dc203_pos2
        .checked_add(dc_mixed::DC_203_BYTES.len())
        .and_then(|p| p.checked_add(1))
        .context("Size offset overflow")?;

    // Corrupt size to 50 bytes (extends past DC_END_LIST)
    let bad_size2 = 50u128.to_be_bytes();
    for (i, &b) in bad_size2.iter().enumerate() {
        if let Some(target) = tampered_bytes2.get_mut(size_start2.saturating_add(i)) {
            *target = b;
        }
    }

    let res2 = match DcMstr::from_bytes(&tampered_bytes2) {
        Ok(mstr) => validate_dc_mixed_strict(mstr),
        Err(e) => Err(anyhow::anyhow!(e)),
    };
    ensure!(
        res2.is_err(),
        "Binary chunk extending past container must fail validation"
    );

    Ok(())
}

// ============================================================================
// 7. ADVERSARIAL: INTERLEAVED AND INVERTED FRAMING REJECTION
// ============================================================================

#[crate::ctb_test]
fn test_challenge_interleaved_and_inverted_framing_rejection() -> Result<()> {
    // A. Inverted delimiters: closer appears before opener
    let inverted_cases: Vec<(&'static str, Vec<u128>)> = vec![
        ("inverted format 512..511", vec![sdc(512), fmt(228), sdc(511)]),
        ("inverted list 311..310", vec![sdc(311), sdc(275), sdc(310)]),
        ("inverted session 548..547", vec![sdc(548), sdc(547)]),
        ("inverted map 560..559", vec![sdc(560), sdc(559)]),
        ("inverted number 7..6", vec![sdc(7), sdc(6)]),
    ];

    for (desc, tokens) in inverted_cases {
        ensure!(
            validate_dc_tokens_strict(&tokens).is_err(),
            "{desc} must be rejected by validate_dc_tokens_strict"
        );
        let dc_str = dc_string_from_tokens(&tokens);
        ensure!(
            validate_dc_str_strict(&dc_str).is_err(),
            "{desc} must be rejected by validate_dc_str_strict"
        );
        if let Ok(mst) = mstring_from_tokens(&tokens) {
            ensure!(
                validate_dc_mixed_strict(&mst).is_err(),
                "{desc} must be rejected by validate_dc_mixed_strict"
            );
        }
    }

    // B. Interleaved containers (cross-framing):
    // List opened (310), Format opened (511), List closed (311), Format closed (512)
    let interleaved = vec![
        sdc(310),
        sdc(275),
        sdc(511),
        fmt(228),
        sdc(260),
        sdc(262),
        sdc(264),
        sdc(263),
        102,
        sdc(261),
        sdc(311), // premature list closer before format closer!
        sdc(512),
    ];
    ensure!(
        validate_dc_tokens_strict(&interleaved).is_err(),
        "Interleaved list and format containers must fail validate_dc_tokens_strict"
    );
    let interleaved_str = dc_string_from_tokens(&interleaved);
    ensure!(
        validate_dc_str_strict(&interleaved_str).is_err(),
        "Interleaved containers must fail validate_dc_str_strict"
    );
    if let Ok(mst) = mstring_from_tokens(&interleaved) {
        ensure!(
            validate_dc_mixed_strict(&mst).is_err(),
            "Interleaved containers must fail validate_dc_mixed_strict"
        );
    }

    // C. Interleaved session and checksummed section
    let interleaved_session = vec![
        sdc(547), // Begin session
        sdc(543), // Begin checksummed
        sdc(548), // End session (crosses closer!)
        sdc(544), // End checksummed
    ];
    ensure!(
        validate_dc_tokens_strict(&interleaved_session).is_err(),
        "Interleaved session and checksummed section must fail validate_dc_tokens_strict"
    );

    Ok(())
}

// ============================================================================
// 8. ADVERSARIAL: NESTED FORMAT REPRESENTATIONS NO PREMATURE 512 CLOSING
// ============================================================================

#[crate::ctb_test]
fn test_challenge_nested_format_representations_no_premature_512() -> Result<()> {
    let policy = DcProjectionPolicy::strict();

    // Nested: :511 [f228] (outer) containing a typed list with an inner :511 [f228]
    // Outer: 511 f228 [string] 512
    // Inside a list: 310 [275] 511 [f228] 260 262 264 263 97 261 512 312 511 [f228] 260 262 264 263 98 261 512 311
    let tokens = vec![
        sdc(310),
        sdc(275),
        // Item 1
        sdc(511),
        fmt(228),
        sdc(260),
        sdc(262),
        sdc(264),
        sdc(263),
        97,
        sdc(261),
        sdc(512),
        sdc(312), // Separator
        // Item 2
        sdc(511),
        fmt(228),
        sdc(260),
        sdc(262),
        sdc(264),
        sdc(263),
        98,
        sdc(261),
        sdc(512),
        sdc(311), // End list
    ];

    validate_dc_tokens_strict(&tokens)?;
    let dc_str = dc_string_from_tokens(&tokens);
    validate_dc_str_strict(&dc_str)?;
    let mst = mstring_from_tokens(&tokens)?;
    validate_dc_mixed_strict(&mst)?;

    // Step navigator through list
    let mut nav = DcDocumentNavigator::from_dc_str(&dc_str);
    let list_stub = nav.next_sibling_stub(&policy)?.context("List stub")?;
    nav.enter_frame(&list_stub)?;

    let _type = nav.next_sibling_stub(&policy)?; // Type wildcard
    let item1 = nav.next_sibling_stub(&policy)?.context("Item 1")?;
    ensure!(item1.head_dc == DC_BEGIN_FORMAT_REPRESENTATION);
    ensure!(item1.closer_dc == Some(DC_END_FORMAT_REPRESENTATION));

    let sep = nav.next_sibling_stub(&policy)?.context("Sep")?;
    ensure!(sep.head_dc == DC_LIST_ELEMENT_SEPARATOR);

    let item2 = nav.next_sibling_stub(&policy)?.context("Item 2")?;
    ensure!(item2.head_dc == DC_BEGIN_FORMAT_REPRESENTATION);
    ensure!(item2.closer_dc == Some(DC_END_FORMAT_REPRESENTATION));

    ensure!(nav.next_sibling_stub(&policy)?.is_none());
    nav.exit_frame()?;
    ensure!(nav.next_sibling_stub(&policy)?.is_none());

    Ok(())
}

// ============================================================================
// 9. ADVERSARIAL: CHAINED PREFIX OPERATORS WITHOUT FIXED ARITY
// ============================================================================

#[crate::ctb_test]
fn test_challenge_chained_prefix_operators() -> Result<()> {
    let policy = DcProjectionPolicy::strict();

    // Sequence of distinct prefix operators followed by an atomic value:
    // 1. Dc 330 (Date prefix) + payload
    // 2. Dc 346 (Host IP prefix) + payload
    // 3. Dc 330 (Date prefix) + payload
    // 4. Atomic boolean Dc 517
    let tokens = vec![
        // 1. Date prefix 330
        sdc(330),
        sdc(390),
        fmt(487),
        sdc(6),
        fmt(199),
        sdc(127),
        sdc(7),
        sdc(391),
        // 2. Host IP prefix 346
        sdc(346),
        sdc(511),
        fmt(490),
        sdc(6),
        fmt(199),
        sdc(127),
        sdc(7),
        sdc(512),
        // 3. Date prefix 330
        sdc(330),
        sdc(390),
        fmt(487),
        sdc(6),
        fmt(199),
        sdc(127),
        sdc(7),
        sdc(391),
        // 4. Atomic boolean true
        sdc(517),
    ];

    validate_dc_tokens_strict(&tokens)?;
    let dc_str = dc_string_from_tokens(&tokens);
    validate_dc_str_strict(&dc_str)?;
    let mst = mstring_from_tokens(&tokens)?;
    validate_dc_mixed_strict(&mst)?;

    // Step navigator through each sibling
    let mut nav = DcDocumentNavigator::from_dc_str(&dc_str);

    let s1 = nav.next_sibling_stub(&policy)?.context("Prefix 1 (date)")?;
    ensure!(s1.head_dc == DcChar::from_short(330), "Stub 1 must be 330");

    let s2 = nav.next_sibling_stub(&policy)?.context("Prefix 2 (ip)")?;
    ensure!(s2.head_dc == DcChar::from_short(346), "Stub 2 must be 346");

    let s3 = nav.next_sibling_stub(&policy)?.context("Prefix 3 (date)")?;
    ensure!(s3.head_dc == DcChar::from_short(330), "Stub 3 must be 330");

    let s4 = nav.next_sibling_stub(&policy)?.context("Stub 4 (bool)")?;
    ensure!(s4.head_dc == DC_BOOLEAN_TRUE, "Stub 4 must be DC_BOOLEAN_TRUE");

    ensure!(
        nav.next_sibling_stub(&policy)?.is_none(),
        "No further stubs expected"
    );

    Ok(())
}


}
