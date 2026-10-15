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
    let policy = DcProjectionPolicy::strict();

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


