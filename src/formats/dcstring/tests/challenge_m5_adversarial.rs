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

//! Milestone 5 Tier 5 Adversarial Coverage Hardening Test Suite:
//!
//! 1. Syntax expansion depth and recursion limits (`MAX_SYNTAX_EXPANSION_DEPTH = 32`)
//!    preventing stack overflows on deeply nested syntax trees.
//! 2. Complex container nesting: equivalent representations (`397`/`561`) inside
//!    optionals (`315`/`316`) inside checksummed sections (`543..544 + 545`).
//! 3. Transparent unwrapping and transparent projection of equivalent representations
//!    and formats.
//! 4. Strict syntax enforcement: strict rejection of bare strings, bare numbers,
//!    mismatched delimiters, and stray closers.
//! 5. In-place source editing (`preserve_source`, `preserve_text`): replacements,
//!    deletions, insertions over `DcSyntaxArena` without panics.

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
    DC_END_WARNING_RECORD, DC_EQUIVALENT_REPRESENTATIONS, DC_ESCAPE,
    DC_LIST_ELEMENT_SEPARATOR, DC_OPTIONAL_ABSENT, DC_OPTIONAL_PRESENT,
    DC_PARTIALLY_EQUIVALENT_REPRESENTATIONS, DC_SECTION_DIGEST, DC_SEQUENCE_IDENTIFIER,
};
use ctb_formats_dcdata::syntax::{
    CharTarget, DatasetRuleResolver, SyntaxPattern, SyntaxRuleResolver, SyntaxTerm,
    default_syntax_resolver,
};
use ctb_formats_dcstring::dc_mixed_serde::encode_format_representation;
pub use ctb_formats_dcstring::*;
pub use ctb_formats_dcdata::dc::*;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;

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
// 1. SYNTAX EXPANSION DEPTH AND RECURSION LIMITS (MAX_SYNTAX_EXPANSION_DEPTH = 32)
// ============================================================================

mod challenge1_recursion_depth_limits {
    use super::*;

    #[crate::ctb_test]
    fn test_adversarial_type_constructor_depth_budget() -> Result<()> {
        let resolver = default_syntax_resolver();
        let term = SyntaxTerm::NamedConstruct {
            name: "type".to_owned(),
            subtype: None,
            capture_var: None,
        };
        for constructor in [DC_TYPE_UNION, DC_TYPE_INTERSECTION] {
            let doc = create_mstring(&[
                constructor,
                constructor,
                DC_TYPE_WILDCARD,
                DC_TYPE_WILDCARD,
                DC_TYPE_WILDCARD,
            ]);
            for budget in [0, 1, 2] {
                let mut nav = DcDocumentNavigator::from_mstr(&doc);
                let start = nav.position();
                let mut ctx = SyntaxEvalContext::new(&**resolver, None);
                ctx.max_depth = budget;
                ensure!(nav.eval_term_single(&term, &mut ctx)? == (budget == 2), "Wrong constructor boundary at budget {budget}");
                ensure!(ctx.has_errors == (budget < 2), "Depth exhaustion must produce a diagnostic");
                ensure!(ctx.depth == 0 && ctx.self_dc.is_none());
                if budget < 2 {
                    ensure!(nav.position() == start, "Failed type matching must restore the cursor");
                } else {
                    ensure!(nav.position() == nav.active_span().end);
                }
            }
            let depth = 8192usize;
            let mut deep = DcMstring::new();
            for _ in 0..depth { deep.push_char(constructor); }
            for _ in 0..=depth { deep.push_char(DC_TYPE_WILDCARD); }
            let mut nav = DcDocumentNavigator::from_mstr(&deep);
            let mut ctx = SyntaxEvalContext::new(&**resolver, None);
            ctx.max_depth = depth;
            ensure!(nav.eval_term_single(&term, &mut ctx)?, "Raised-budget constructor chain must parse iteratively");
            ensure!(!ctx.has_errors && nav.position() == nav.active_span().end);

            for chars in [
                vec![constructor, DC_TYPE_WILDCARD],
                vec![constructor, DC_BOOLEAN_TRUE, DC_TYPE_WILDCARD],
            ] {
                let malformed = create_mstring(&chars);
                let mut nav = DcDocumentNavigator::from_mstr(&malformed);
                let start = nav.position();
                let mut ctx = SyntaxEvalContext::new(&**resolver, None);
                ensure!(!nav.eval_term_single(&term, &mut ctx)?);
                ensure!(nav.position() == start && ctx.depth == 0 && ctx.self_dc.is_none());
            }
            let list = create_mstring(&[
                constructor, DC_BEGIN_LIST, DC_TYPE_WILDCARD,
                DC_TYPE_WILDCARD, DC_LIST_ELEMENT_SEPARATOR, DC_TYPE_WILDCARD, DC_END_LIST,
            ]);
            let mut nav = DcDocumentNavigator::from_mstr(&list);
            let mut ctx = SyntaxEvalContext::new(&**resolver, None);
            ensure!(nav.eval_term_single(&term, &mut ctx)?, "Generated list alternative must remain accepted");
            ensure!(nav.position() == nav.active_span().end && !ctx.has_errors);
        }
        Ok(())
    }

    /// Document depth is a caller policy, independent of grammar expansion.
    #[crate::ctb_test]
    fn test_adversarial_depth_limit_boundary_at_32_and_33() -> Result<()> {
        ensure!(
            MAX_SYNTAX_EXPANSION_DEPTH == 32,
            "MAX_SYNTAX_EXPANSION_DEPTH invariant must be 32"
        );

        // A. 31 levels of nested lists (depth 0..=30 inside outer, well within 32)
        let mut doc31 = DcMstring::new();
        for _ in 0..31 {
            doc31.push_char(DC_BEGIN_LIST);
            doc31.push_char(DC_TYPE_WILDCARD);
        }
        doc31.push_char(DC_BOOLEAN_TRUE);
        for _ in 0..31 {
            doc31.push_char(DC_END_LIST);
        }

        // Must validate cleanly under strict document validation
        validate_dc_mixed_strict(&doc31)
            .context("Depth 31 nesting should pass strict validation")?;

        // B. 32 levels of nested lists: interior depth reaches 32, triggering MAX_SYNTAX_EXPANSION_DEPTH
        let mut doc32 = DcMstring::new();
        for _ in 0..32 {
            doc32.push_char(DC_BEGIN_LIST);
            doc32.push_char(DC_TYPE_WILDCARD);
        }
        doc32.push_char(DC_BOOLEAN_TRUE);
        for _ in 0..32 {
            doc32.push_char(DC_END_LIST);
        }

        validate_dc_mixed_strict(&doc32)?;

        // C. 33 levels of nested lists: strictly rejected
        let mut doc33 = DcMstring::new();
        for _ in 0..33 {
            doc33.push_char(DC_BEGIN_LIST);
            doc33.push_char(DC_TYPE_WILDCARD);
        }
        doc33.push_char(DC_BOOLEAN_TRUE);
        for _ in 0..33 {
            doc33.push_char(DC_END_LIST);
        }

        validate_dc_mixed_strict(&doc33)?;
        let low = DcProjectionPolicy::strict().with_max_depth(32, DiagnosticAction::Error);
        let mut nav = DcDocumentNavigator::from_mstr(&doc33);
        ensure!(nav.validate_document(&low).is_err());
        let mut arena = DcSyntaxArena::new();
        ensure!(nav.parse_document_into_arena(&mut arena, &low).is_err());

        // D. Also verify validate_dc_str_strict and validate_dc_tokens_strict reject depth 33
        let mut text33 = DcString::new();
        for _ in 0..33 {
            text33.push(DC_BEGIN_LIST);
            text33.push(DC_TYPE_WILDCARD);
        }
        text33.push(DC_BOOLEAN_TRUE);
        for _ in 0..33 {
            text33.push(DC_END_LIST);
        }

        validate_dc_str_strict(&text33)?;

        let mut tokens33 = Vec::new();
        for _ in 0..33 {
            tokens33.push(DC_BEGIN_LIST.to_u128());
            tokens33.push(DC_TYPE_WILDCARD.to_u128());
        }
        tokens33.push(DC_BOOLEAN_TRUE.to_u128());
        for _ in 0..33 {
            tokens33.push(DC_END_LIST.to_u128());
        }

        validate_dc_tokens_strict(&tokens33)?;

        Ok(())
    }

    /// Verifies that extreme adversarial nesting (50, 100, 200, 500 levels deep)
    /// never causes a stack overflow or infinite recursion.
    #[crate::ctb_test]
    fn test_adversarial_extreme_nesting_no_stack_overflow() -> Result<()> {
        for &depth in &[50, 100, 200, 500] {
            let mut doc = DcMstring::new();
            for _ in 0..depth {
                doc.push_char(DC_BEGIN_LIST);
                doc.push_char(DC_TYPE_WILDCARD);
            }
            doc.push_char(DC_BOOLEAN_TRUE);
            for _ in 0..depth {
                doc.push_char(DC_END_LIST);
            }

            let res = validate_dc_mixed_strict(&doc);
            ensure!(
                res.is_ok() == (depth <= 64),
                "Default document depth budget disagrees at depth {depth}"
            );
            let raised = DcProjectionPolicy::strict().with_max_depth(600, DiagnosticAction::Error);
            let mut nav = DcDocumentNavigator::from_mstr(&doc);
            nav.validate_document(&raised)?;
            let mut arena = DcSyntaxArena::new();
            nav.parse_document_into_arena(&mut arena, &raised)?;
            ensure!(arena.diagnostics().is_empty());
        }
        Ok(())
    }

    /// Verifies that Stage 1 arena parsing and `unwrap_transparent` respect
    /// `policy.max_depth` and `policy.depth_limit_exceeded` without hanging or looping infinitely.
    #[crate::ctb_test]
    fn test_adversarial_transparent_unwrapping_depth_bound() -> Result<()> {
        let strict = DcProjectionPolicy::strict()
            .with_max_depth(32, ctb_formats_dcstring::DiagnosticAction::Error);
        let forgiving = DcProjectionPolicy::forgiving()
            .with_max_depth(32, ctb_formats_dcstring::DiagnosticAction::Warn);

        // 40 levels of nested 315 optional present containers
        let mut doc = DcMstring::new();
        for _ in 0..40 {
            doc.push_char(DC_OPTIONAL_PRESENT);
        }
        42u64.encode_dc_mixed(&mut doc)?;

        // Under strict policy, exceeding max_depth (32) in Stage 1 returns an error
        let mut nav_strict = DcDocumentNavigator::from_mstr(&doc);
        let mut arena_strict = DcSyntaxArena::new();
        ensure!(
            nav_strict.parse_document_into_arena(&mut arena_strict, &strict).is_err(),
            "Strict policy must reject transparent wrapper chains exceeding max_depth"
        );

        // Under forgiving policy, Stage 1 caps the arena at 32 levels and unwrap_transparent terminates cleanly
        let mut nav_forgiving = DcDocumentNavigator::from_mstr(&doc);
        let mut arena_forgiving = DcSyntaxArena::new();
        nav_forgiving.parse_document_into_arena(&mut arena_forgiving, &forgiving)?;

        let roots = arena_forgiving.roots();
        ensure!(!roots.is_empty(), "Arena must contain root node");
        let root = roots[0];

        let ctx_forgiving = DcProjectionContext::new(&arena_forgiving, &forgiving);
        let unwrapped = ctx_forgiving.unwrap_transparent(root)?;
        let node = ctx_forgiving.node(unwrapped)?;
        ensure!(
            matches!(&node.kind, DcSyntaxKind::Malformed { .. }),
            "Unwrapped node should stop at the depth-capped Malformed node after 32 iterations"
        );

        Ok(())
    }
}

// ============================================================================
// 2. COMPLEX CONTAINER NESTING: EQUIVALENTS IN OPTIONALS IN CHECKSUMMED SECTIONS
// ============================================================================

mod challenge2_complex_container_nesting {
    use super::*;

    /// Constructs equivalent representations (`397`/`561`) inside an optional present
    /// (`315`) inside a checksummed section (`543..544 + 545`), verifying:
    /// - SHA-256 section digest verification
    /// - Framing and arena AST construction
    /// - `DcMixedDecode` transparent projection to typed values (PathBuf and String)
    /// - Detection of tamper anywhere in the nested envelope
    #[crate::ctb_test]
    fn test_adversarial_equivalent_in_optional_in_checksummed_section() -> Result<()> {
        let policy = DcProjectionPolicy::strict();
        let mut mst = DcMstring::new();

        // 1. Begin checksummed section (543)
        mst.push_char(DC_BEGIN_CHECKSUMMED_SECTION);
        let body_start = mst.len();

        // Sequence identifier (546) followed by integer 1001
        mst.push_char(DC_SEQUENCE_IDENTIFIER);
        1001u64.encode_dc_mixed(&mut mst)?;

        // 2. Optional present (315) wrapping equivalent representations (397)
        mst.push_char(DC_OPTIONAL_PRESENT);

        // 3. Equivalent representations (397): :~ [list]
        mst.push_char(DC_EQUIVALENT_REPRESENTATIONS);
        mst.push_char(DC_BEGIN_LIST);
        mst.push_char(DC_TYPE_WILDCARD);

        // Primary candidate: PathBuf (f494 format representation)
        PathBuf::from("report.pdf").encode_dc_mixed(&mut mst)?;

        // Separator (312)
        mst.push_char(DC_LIST_ELEMENT_SEPARATOR);

        // Secondary candidate: alternative PathBuf
        PathBuf::from("backup.pdf").encode_dc_mixed(&mut mst)?;

        // Close equivalent list (311)
        mst.push_char(DC_END_LIST);

        let body_end = mst.len();

        // 4. End checksummed section (544)
        mst.push_char(DC_END_CHECKSUMMED_SECTION);

        // 5. Compute SHA-256 digest over [body_start..body_end]
        let digest = {
            let mstr_bytes: &[u8] = mst.as_ref();
            let body_bytes = mstr_bytes
                .get(body_start..body_end)
                .context("Body byte slice")?;
            sha256(body_bytes)
        };

        // 6. Section digest (545) + SHA-256
        mst.push_char(DC_SECTION_DIGEST);
        mst.push_checksum(ctb_utilities::format_id::DC_SHA256, &digest)?;

        // Strict validation must pass
        validate_dc_mixed_strict(&mst)
            .context("Complex nested container must pass strict validation")?;

        // Navigator verification: verify digest validity
        let mut nav = DcDocumentNavigator::from_mstr(&mst);
        let section_stub = nav
            .next_sibling_stub(&policy)?
            .context("Expected checksummed section stub")?;

        ensure!(
            section_stub.head_dc == DC_BEGIN_CHECKSUMMED_SECTION,
            "Stub must begin with 543"
        );
        ensure!(
            section_stub.closer_dc == Some(DC_END_CHECKSUMMED_SECTION),
            "Stub must close with 544"
        );

        if let DcFrameHeaderInfo::ChecksummedSection { digest_valid, .. } =
            section_stub.header_info
        {
            ensure!(digest_valid, "Digest must be verified as valid");
        } else {
            anyhow::bail!("Expected ChecksummedSection header info");
        }

        // Expand frame into arena
        let mut arena = DcSyntaxArena::new();
        let section_node_id = nav.expand_frame(&section_stub, &mut arena, &policy)?;

        let section_node = arena.get(section_node_id).context("Section node")?;
        let children = section_node.children();
        ensure!(
            children.len() >= 2,
            "Section must have seq id and optional children"
        );

        // Decode the optional equivalent PathBuf
        let opt_child_id = children[1];
        let mut ctx = DcProjectionContext::new(&arena, &policy);

        let decoded_path: Option<PathBuf> =
            Option::<PathBuf>::decode_dc_node(&mut ctx, opt_child_id)
                .context("Projecting Option<PathBuf> through 315 -> 397 -> 511 [f494]")?;
        ensure!(
            decoded_path == Some(PathBuf::from("report.pdf")),
            "Primary candidate must project to PathBuf: got {decoded_path:?}"
        );

        // Adversarial challenge: Tamper 1 byte inside the nested equivalent filename
        let mut tampered = mst.clone().into_bytes();
        let target_byte_index = tampered
            .windows(4)
            .position(|w| w == b"repo")
            .context("Finding target byte")?;
        tampered[target_byte_index] = b'X'; // Corrupt character 'r' to 'X' without breaking UTF-8e-128 tokenization

        let tampered_mst = DcMstring::from_bytes(tampered)?;
        let mut tampered_nav = DcDocumentNavigator::from_mstr(&tampered_mst);
        ensure!(tampered_nav.next_sibling_stub(&policy).is_err());
        let mut tampered_nav = DcDocumentNavigator::from_mstr(&tampered_mst);
        let tampered_stub = tampered_nav
            .next_sibling_stub(&DcProjectionPolicy::forgiving())?
            .context("Tampered stub")?;

        if let DcFrameHeaderInfo::ChecksummedSection { digest_valid, .. } =
            tampered_stub.header_info
        {
            ensure!(
                !digest_valid,
                "Tampered byte inside nested equivalent must invalidate section digest"
            );
        } else {
            anyhow::bail!("Expected ChecksummedSection header info");
        }

        Ok(())
    }

    /// Verifies optional absent (`316`) inside a checksummed section.
    #[crate::ctb_test]
    fn test_adversarial_optional_absent_in_checksummed_section() -> Result<()> {
        let policy = DcProjectionPolicy::strict();
        let mut mst = DcMstring::new();

        mst.push_char(DC_BEGIN_CHECKSUMMED_SECTION);
        let body_start = mst.len();

        mst.push_char(DC_SEQUENCE_IDENTIFIER);
        42u64.encode_dc_mixed(&mut mst)?;

        mst.push_char(DC_OPTIONAL_ABSENT); // 316
        let body_end = mst.len();

        mst.push_char(DC_END_CHECKSUMMED_SECTION);

        let digest = {
            let mstr_bytes: &[u8] = mst.as_ref();
            let body_bytes = mstr_bytes
                .get(body_start..body_end)
                .context("Body byte slice")?;
            sha256(body_bytes)
        };
        mst.push_char(DC_SECTION_DIGEST);
        mst.push_checksum(ctb_utilities::format_id::DC_SHA256, &digest)?;

        validate_dc_mixed_strict(&mst)?;

        let mut nav = DcDocumentNavigator::from_mstr(&mst);
        let stub = nav.next_sibling_stub(&policy)?.context("Section stub")?;
        let mut arena = DcSyntaxArena::new();
        let sec_id = nav.expand_frame(&stub, &mut arena, &policy)?;

        let sec_node = arena.get(sec_id).context("Section node")?;
        let opt_child = sec_node.children()[1];

        let mut ctx = DcProjectionContext::new(&arena, &policy);
        let decoded: Option<String> = Option::<String>::decode_dc_node(&mut ctx, opt_child)?;
        ensure!(
            decoded.is_none(),
            "Optional absent must decode to None: got {decoded:?}"
        );

        Ok(())
    }
}

// ============================================================================
// 3. TRANSPARENT UNWRAPPING AND PROJECTION OF EQUIVALENT REPRESENTATIONS & FORMATS
// ============================================================================

mod challenge3_transparent_unwrapping {
    use super::*;

    /// Verifies deep transparent unwrapping across multiple layers:
    /// `397` [list] -> `561` [list] -> `315` (Optional)
    /// wrapping various target types.
    #[crate::ctb_test]
    fn test_adversarial_multi_layer_transparent_projection() -> Result<()> {
        let policy = DcProjectionPolicy::strict();

        // 1. Primitive integer u64 through 397 [list] -> 561 [list] -> 315
        let mut mst_u64 = DcMstring::new();
        mst_u64.push_char(DC_EQUIVALENT_REPRESENTATIONS);
        mst_u64.push_char(DC_BEGIN_LIST);
        mst_u64.push_char(DC_TYPE_WILDCARD);

        mst_u64.push_char(DC_PARTIALLY_EQUIVALENT_REPRESENTATIONS);
        mst_u64.push_char(DC_BEGIN_LIST);
        mst_u64.push_char(DC_TYPE_WILDCARD);

        mst_u64.push_char(DC_OPTIONAL_PRESENT);
        9876543210u64.encode_dc_mixed(&mut mst_u64)?;

        mst_u64.push_char(DC_END_LIST);
        mst_u64.push_char(DC_END_LIST);

        let mut nav = DcDocumentNavigator::from_mstr(&mst_u64);
        let mut arena = DcSyntaxArena::new();
        nav.parse_document_into_arena(&mut arena, &policy)?;
        let root = arena.roots()[0];

        let mut ctx = DcProjectionContext::new(&arena, &policy);
        let val_u64 = u64::decode_dc_node(&mut ctx, root)?;
        ensure!(val_u64 == 9876543210, "u64 unwrapping mismatch");

        // 2. String through 397 [list] -> 561 [list] -> 315
        let mut mst_str = DcMstring::new();
        mst_str.push_char(DC_EQUIVALENT_REPRESENTATIONS);
        mst_str.push_char(DC_BEGIN_LIST);
        mst_str.push_char(DC_TYPE_WILDCARD);

        mst_str.push_char(DC_PARTIALLY_EQUIVALENT_REPRESENTATIONS);
        mst_str.push_char(DC_BEGIN_LIST);
        mst_str.push_char(DC_TYPE_WILDCARD);

        mst_str.push_char(DC_OPTIONAL_PRESENT);
        "multi_layer_string".to_string().encode_dc_mixed(&mut mst_str)?;

        mst_str.push_char(DC_END_LIST);
        mst_str.push_char(DC_END_LIST);

        let mut nav_str = DcDocumentNavigator::from_mstr(&mst_str);
        let mut arena_str = DcSyntaxArena::new();
        nav_str.parse_document_into_arena(&mut arena_str, &policy)?;
        let root_str = arena_str.roots()[0];

        let mut ctx_str = DcProjectionContext::new(&arena_str, &policy);
        let val_str = String::decode_dc_node(&mut ctx_str, root_str)?;
        ensure!(val_str == "multi_layer_string", "String unwrapping mismatch");

        // 3. Composite framed map BTreeMap<String, u32> through 397 [list] -> 315
        let mut map_data = BTreeMap::new();
        map_data.insert("key1".to_string(), 100u32);
        map_data.insert("key2".to_string(), 200u32);
        let mut mst_map = DcMstring::new();
        mst_map.push_char(DC_EQUIVALENT_REPRESENTATIONS);
        mst_map.push_char(DC_BEGIN_LIST);
        mst_map.push_char(DC_TYPE_WILDCARD);

        mst_map.push_char(DC_OPTIONAL_PRESENT);
        map_data.encode_dc_mixed(&mut mst_map)?;

        mst_map.push_char(DC_END_LIST);

        let mut nav_map = DcDocumentNavigator::from_mstr(&mst_map);
        let mut arena_map = DcSyntaxArena::new();
        nav_map.parse_document_into_arena(&mut arena_map, &policy)?;
        let root_map = arena_map.roots()[0];

        let mut ctx_map = DcProjectionContext::new(&arena_map, &policy);
        let val_map = Option::<BTreeMap<String, u32>>::decode_dc_node(&mut ctx_map, root_map)?;
        ensure!(val_map == Some(map_data), "Map unwrapping mismatch");

        // 4. Collection Vec<u32> through 397 [list] -> 315
        let vec_data = vec![10u32, 20u32, 30u32, 40u32];
        let mut mst_vec = DcMstring::new();
        mst_vec.push_char(DC_EQUIVALENT_REPRESENTATIONS);
        mst_vec.push_char(DC_BEGIN_LIST);
        mst_vec.push_char(DC_TYPE_WILDCARD);

        mst_vec.push_char(DC_OPTIONAL_PRESENT);
        vec_data.encode_dc_mixed(&mut mst_vec)?;

        mst_vec.push_char(DC_END_LIST);

        let mut nav_vec = DcDocumentNavigator::from_mstr(&mst_vec);
        let mut arena_vec = DcSyntaxArena::new();
        nav_vec.parse_document_into_arena(&mut arena_vec, &policy)?;
        let root_vec = arena_vec.roots()[0];

        let mut ctx_vec = DcProjectionContext::new(&arena_vec, &policy);
        let val_vec = Option::<Vec<u32>>::decode_dc_node(&mut ctx_vec, root_vec)?;
        ensure!(val_vec == Some(vec_data.clone()), "Option<Vec> unwrapping mismatch");
        let direct_vec = Vec::<u32>::decode_dc_node(&mut ctx_vec, root_vec)?;
        ensure!(direct_vec == vec_data, "Direct Vec unwrapping mismatch");

        // 5. Composite framed pair (u32, bool) = (777, true) through 397 [list] -> 315
        let pair: (u32, bool) = (777, true);
        let mut mst_pair = DcMstring::new();
        mst_pair.push_char(DC_EQUIVALENT_REPRESENTATIONS);
        mst_pair.push_char(DC_BEGIN_LIST);
        mst_pair.push_char(DC_TYPE_WILDCARD);
        mst_pair.push_char(DC_OPTIONAL_PRESENT);
        pair.encode_dc_mixed(&mut mst_pair)?;
        mst_pair.push_char(DC_END_LIST);

        let mut nav_pair = DcDocumentNavigator::from_mstr(&mst_pair);
        let mut arena_pair = DcSyntaxArena::new();
        nav_pair.parse_document_into_arena(&mut arena_pair, &policy)?;
        let root_pair = arena_pair.roots()[0];
        let mut ctx_pair = DcProjectionContext::new(&arena_pair, &policy);
        let decoded_pair = <(u32, bool)>::decode_dc_node(&mut ctx_pair, root_pair)?;
        ensure!(decoded_pair == pair, "Composite pair (777, true) unwrapping mismatch");

        // 6. #[derive(DcMixed)] struct with framed and tagged (u32, bool) + Vec<u32> fields
        // wrapped in 397 [list] -> 315
        #[derive(Debug, Clone, PartialEq, Eq, DcMixed)]
        #[dc(begin = 401, end = 402)]
        struct FramedPairAndVecRecord {
            #[dc(begin = 490, end = 491)]
            bounded_pair: (u32, bool),
            #[dc(begin = 340, end = 341)]
            bounded_vec: Vec<u32>,
            #[dc(short = 406)]
            tagged_pair: (u32, bool),
            #[dc(short = 408)]
            tagged_vec: Vec<u32>,
        }

        let mut mst_rec = DcMstring::new();
        mst_rec.push_char(DcChar::from_short(401));
        // bounded_pair wrapped inside 490..491 via 397 [list] -> 315
        mst_rec.push_char(DcChar::from_short(490));
        mst_rec.push_char(DC_EQUIVALENT_REPRESENTATIONS);
        mst_rec.push_char(DC_BEGIN_LIST);
        mst_rec.push_char(DC_TYPE_WILDCARD);
        mst_rec.push_char(DC_OPTIONAL_PRESENT);
        pair.encode_dc_mixed(&mut mst_rec)?;
        mst_rec.push_char(DC_END_LIST);
        mst_rec.push_char(DcChar::from_short(491));
        // bounded_vec wrapped inside 340..341 via 397 [list] -> 315
        mst_rec.push_char(DcChar::from_short(340));
        mst_rec.push_char(DC_EQUIVALENT_REPRESENTATIONS);
        mst_rec.push_char(DC_BEGIN_LIST);
        mst_rec.push_char(DC_TYPE_WILDCARD);
        mst_rec.push_char(DC_OPTIONAL_PRESENT);
        vec_data.encode_dc_mixed(&mut mst_rec)?;
        mst_rec.push_char(DC_END_LIST);
        mst_rec.push_char(DcChar::from_short(341));
        // tagged_pair wrapped via 397 [list] -> 315
        mst_rec.push_char(DcChar::from_short(406));
        mst_rec.push_char(DC_EQUIVALENT_REPRESENTATIONS);
        mst_rec.push_char(DC_BEGIN_LIST);
        mst_rec.push_char(DC_TYPE_WILDCARD);
        mst_rec.push_char(DC_OPTIONAL_PRESENT);
        pair.encode_dc_mixed(&mut mst_rec)?;
        mst_rec.push_char(DC_END_LIST);
        // tagged_vec wrapped via 397 [list] -> 315
        for &item in &vec_data {
            mst_rec.push_char(DcChar::from_short(408));
            mst_rec.push_char(DC_EQUIVALENT_REPRESENTATIONS);
            mst_rec.push_char(DC_BEGIN_LIST);
            mst_rec.push_char(DC_TYPE_WILDCARD);
            mst_rec.push_char(DC_OPTIONAL_PRESENT);
            item.encode_dc_mixed(&mut mst_rec)?;
            mst_rec.push_char(DC_END_LIST);
        }
        mst_rec.push_char(DcChar::from_short(402));

        let decoded_rec: FramedPairAndVecRecord = mst_rec.decode(&policy)?;
        ensure!(
            decoded_rec
                == FramedPairAndVecRecord {
                    bounded_pair: pair,
                    bounded_vec: vec_data.clone(),
                    tagged_pair: pair,
                    tagged_vec: vec_data,
                },
            "Derive macro FramedPairAndVecRecord through 397 [list] -> 315 mismatch: {decoded_rec:?}"
        );

        Ok(())
    }

    /// Verifies that `Vec<T>` and `(A, B)` call `unwrap_transparent` and properly
    /// unwrap transparent optional (`315`) wrappers.
    #[crate::ctb_test]
    fn test_adversarial_bare_vec_and_tuple_unwrap_transparent_optionals() -> Result<()> {
        let policy = DcProjectionPolicy::strict();

        // 315 wrapping a Vec<u32>
        let mut doc_vec = DcMstring::new();
        doc_vec.push_char(DC_OPTIONAL_PRESENT);
        vec![1u32, 2u32].encode_dc_mixed(&mut doc_vec)?;

        let mut nav = DcDocumentNavigator::from_mstr(&doc_vec);
        let mut arena = DcSyntaxArena::new();
        nav.parse_document_into_arena(&mut arena, &policy)?;
        let root = arena.roots()[0];

        let mut ctx = DcProjectionContext::new(&arena, &policy);
        let res_vec = Vec::<u32>::decode_dc_node(&mut ctx, root)?;
        ensure!(
            res_vec == vec![1u32, 2u32],
            "Vec<T>::decode_dc_node must unwrap transparent 315 wrapper"
        );

        // 315 wrapping a framed pair (u32, u32)
        let mut doc_pair = DcMstring::new();
        doc_pair.push_char(DC_OPTIONAL_PRESENT);
        doc_pair.push_char(DC_BEGIN_KV_PAIR);
        10u32.encode_dc_mixed(&mut doc_pair)?;
        doc_pair.push_char(DC_LIST_ELEMENT_SEPARATOR);
        20u32.encode_dc_mixed(&mut doc_pair)?;
        doc_pair.push_char(DC_END_KV_PAIR);

        let mut nav_p = DcDocumentNavigator::from_mstr(&doc_pair);
        let mut arena_p = DcSyntaxArena::new();
        nav_p.parse_document_into_arena(&mut arena_p, &policy)?;
        let root_p = arena_p.roots()[0];

        let mut ctx_p = DcProjectionContext::new(&arena_p, &policy);
        let res_pair = <(u32, u32)>::decode_dc_node(&mut ctx_p, root_p)?;
        ensure!(
            res_pair == (10u32, 20u32),
            "(A, B)::decode_dc_node must unwrap transparent 315 wrapper"
        );

        Ok(())
    }

    /// Verifies strict enforcement of separator `312` between equivalent representation items.
    #[crate::ctb_test]
    fn test_adversarial_equivalent_list_separator_enforcement() -> Result<()> {
        let strict = DcProjectionPolicy::strict();

        // 397 list with 2 items BUT missing the 312 separator!
        let mut doc = DcMstring::new();
        doc.push_char(DC_EQUIVALENT_REPRESENTATIONS);
        doc.push_char(DC_BEGIN_LIST);
        doc.push_char(DC_TYPE_WILDCARD);
        100u64.encode_dc_mixed(&mut doc)?;
        // Missing DC_LIST_ELEMENT_SEPARATOR (312)!
        200u64.encode_dc_mixed(&mut doc)?;
        doc.push_char(DC_END_LIST);

        ensure!(validate_dc_mixed_strict(&doc).is_err(), "Standalone validation must enforce equivalent separators too");
        let mut nav = DcDocumentNavigator::from_mstr(&doc);
        let mut arena = DcSyntaxArena::new();
        let res = nav.parse_document_into_arena(&mut arena, &strict);
        ensure!(
            res.is_err(),
            "Equivalent list missing 312 separator must be rejected in strict mode"
        );
        let err_msg = format!("{:#}", res.unwrap_err());
        ensure!(
            err_msg.contains("missing element separator"),
            "Error must mention missing separator: got {err_msg}"
        );

        Ok(())
    }

    /// Verifies strict rejection of empty equivalent representation containers and empty optionals.
    #[crate::ctb_test]
    fn test_adversarial_empty_equivalent_and_empty_optional_rejection() -> Result<()> {
        let strict = DcProjectionPolicy::strict();

        // A. Empty 397 tagged construct
        let mut doc_empty_eq = DcMstring::new();
        doc_empty_eq.push_char(DC_EQUIVALENT_REPRESENTATIONS);

        let mut nav_eq = DcDocumentNavigator::from_mstr(&doc_empty_eq);
        let mut arena_eq = DcSyntaxArena::new();
        let _ = nav_eq.parse_document_into_arena(&mut arena_eq, &strict);
        if let Some(&root) = arena_eq.roots().first() {
            let ctx = DcProjectionContext::new(&arena_eq, &strict);
            let res = ctx.unwrap_transparent(root);
            ensure!(
                res.is_err(),
                "Empty equivalent representation must error under strict policy"
            );
        }

        // B. Empty 315 tagged construct
        let mut doc_empty_opt = DcMstring::new();
        doc_empty_opt.push_char(DC_OPTIONAL_PRESENT);

        let mut nav_opt = DcDocumentNavigator::from_mstr(&doc_empty_opt);
        let mut arena_opt = DcSyntaxArena::new();
        let _ = nav_opt.parse_document_into_arena(&mut arena_opt, &strict);
        if let Some(&root) = arena_opt.roots().first() {
            let ctx = DcProjectionContext::new(&arena_opt, &strict);
            let res = ctx.unwrap_transparent(root);
            ensure!(
                res.is_err(),
                "Empty optional construct must error under strict policy"
            );
        }

        Ok(())
    }
}

// ============================================================================
// 4. STRICT SYNTAX ENFORCEMENT
// ============================================================================

mod challenge4_strict_syntax_enforcement {
    use super::*;

    /// Verifies that bare strings and bare binaries are rejected for `PathBuf`.
    #[crate::ctb_test]
    fn test_adversarial_strict_rejection_bare_strings_and_binary() -> Result<()> {
        let strict = DcProjectionPolicy::strict();
        let forgiving = DcProjectionPolicy::forgiving();

        // 1. Bare string representation for PathBuf
        let mut mst_bare_str = DcMstring::new();
        "folder/file.txt".to_string().encode_dc_mixed(&mut mst_bare_str)?;

        let mut nav = DcDocumentNavigator::from_mstr(&mst_bare_str);
        let mut arena = DcSyntaxArena::new();
        nav.parse_document_into_arena(&mut arena, &strict)?;
        let root = arena.roots()[0];

        let mut ctx_strict = DcProjectionContext::new(&arena, &strict);
        let res_strict = PathBuf::decode_dc_node(&mut ctx_strict, root);
        ensure!(
            res_strict.is_err(),
            "Bare string for PathBuf must fail under strict policy"
        );
        let err_msg = format!("{:#}", res_strict.unwrap_err());
        ensure!(
            err_msg.contains("Expected format representation (Dc 511 + f494) for PathBuf"),
            "Error message must specify canonical format representation requirement: got {err_msg}"
        );

        let mut ctx_forgiving = DcProjectionContext::new(&arena, &forgiving);
        ensure!(
            PathBuf::decode_dc_node(&mut ctx_forgiving, root).is_err(),
            "Standalone PathBuf decoder must reject bare string under forgiving policy too"
        );

        // 2. Bare binary payload for PathBuf
        let mut mst_bare_bin = DcMstring::new();
        mst_bare_bin.push_binary(b"raw/binary/path.dat");

        let mut nav_bin = DcDocumentNavigator::from_mstr(&mst_bare_bin);
        let mut arena_bin = DcSyntaxArena::new();
        nav_bin.parse_document_into_arena(&mut arena_bin, &strict)?;
        let root_bin = arena_bin.roots()[0];

        let mut ctx_bin_strict = DcProjectionContext::new(&arena_bin, &strict);
        let res_bin_strict = PathBuf::decode_dc_node(&mut ctx_bin_strict, root_bin);
        ensure!(
            res_bin_strict.is_err(),
            "Bare binary for PathBuf must fail under strict policy"
        );

        let mut ctx_bin_forgiving = DcProjectionContext::new(&arena_bin, &forgiving);
        ensure!(
            PathBuf::decode_dc_node(&mut ctx_bin_forgiving, root_bin).is_err(),
            "Standalone PathBuf decoder must reject bare binary under forgiving policy too"
        );

        Ok(())
    }

    /// Verifies that bare numbers are rejected for `Duration` and `SystemTime`.
    #[crate::ctb_test]
    fn test_adversarial_strict_rejection_bare_numbers() -> Result<()> {
        let strict = DcProjectionPolicy::strict();
        let forgiving = DcProjectionPolicy::forgiving();

        // 1. Bare number for Duration
        let mut mst_num = DcMstring::new();
        5_000_000_000u64.encode_dc_mixed(&mut mst_num)?;

        let mut nav = DcDocumentNavigator::from_mstr(&mst_num);
        let mut arena = DcSyntaxArena::new();
        nav.parse_document_into_arena(&mut arena, &strict)?;
        let root = arena.roots()[0];

        let mut ctx_dur_strict = DcProjectionContext::new(&arena, &strict);
        let res_dur_strict = Duration::decode_dc_node(&mut ctx_dur_strict, root);
        ensure!(
            res_dur_strict.is_err(),
            "Bare number for Duration must fail under strict policy"
        );
        let err_dur = format!("{:#}", res_dur_strict.unwrap_err());
        ensure!(
            err_dur.contains("Expected duration container (Dc 393..394 or 395..396) for Duration"),
            "Error must specify duration container requirement: got {err_dur}"
        );

        let mut ctx_dur_forgiving = DcProjectionContext::new(&arena, &forgiving);
        ensure!(
            Duration::decode_dc_node(&mut ctx_dur_forgiving, root).is_err(),
            "Standalone Duration decoder must reject bare number under forgiving policy too"
        );

        // 2. Bare number for SystemTime
        let mut ctx_time_strict = DcProjectionContext::new(&arena, &strict);
        let res_time_strict = std::time::SystemTime::decode_dc_node(&mut ctx_time_strict, root);
        ensure!(
            res_time_strict.is_err(),
            "Bare number for SystemTime must fail under strict policy"
        );

        let mut ctx_time_forgiving = DcProjectionContext::new(&arena, &forgiving);
        ensure!(
            std::time::SystemTime::decode_dc_node(&mut ctx_time_forgiving, root).is_err(),
            "Standalone SystemTime decoder must reject bare number under forgiving policy too"
        );

        Ok(())
    }

    /// Verifies strict rejection of mismatched delimiters.
    #[crate::ctb_test]
    fn test_adversarial_strict_rejection_mismatched_delimiters() -> Result<()> {
        let mismatched_pairs: &[(DcChar, DcChar, &str)] = &[
            (DC_BEGIN_LIST, DC_END_FORMAT_REPRESENTATION, "List opened with 310, closed with 512"),
            (DC_BEGIN_KV_MAP, DC_END_LIST, "KV Map opened with 559, closed with 311"),
            (DC_BEGIN_CHECKSUMMED_SECTION, DC_END_OPERATION_SESSION, "Section opened with 543, closed with 548"),
            (DC_BEGIN_KV_PAIR, DC_END_KV_MAP, "KV Pair opened with 514, closed with 560"),
            (DC_BEGIN_OPERATION_SESSION, DC_END_LIST, "Session opened with 547, closed with 311"),
            (DC_BEGIN_FORMAT_REPRESENTATION, DC_END_LEGACY_KV_MAP, "Format opened with 511, closed with 516"),
        ];

        for &(opener, closer, desc) in mismatched_pairs {
            let doc = create_mstring(&[opener, DC_TYPE_WILDCARD, closer]);
            let res = validate_dc_mixed_strict(&doc);
            ensure!(
                res.is_err(),
                "Mismatched pair '{desc}' must be rejected by validate_dc_mixed_strict"
            );
        }

        Ok(())
    }

    /// Verifies strict rejection of stray closing delimiters at statement level.
    #[crate::ctb_test]
    fn test_adversarial_strict_rejection_stray_closers() -> Result<()> {
        let stray_closers = [
            (DC_END_LIST, "311 (End List)"),
            (DC_END_FORMAT_REPRESENTATION, "512 (End Format)"),
            (DC_END_CHECKSUMMED_SECTION, "544 (End Checksummed Section)"),
            (DC_END_OPERATION_SESSION, "548 (End Operation Session)"),
            (DC_END_KV_MAP, "560 (End KV Map)"),
            (DC_END_KV_PAIR, "515 (End KV Pair)"),
            (DC_END_ERROR_RECORD, "553 (End Error Record)"),
            (DC_END_WARNING_RECORD, "555 (End Warning Record)"),
            (DC_END_NUMBER, "7 (End Number)"),
        ];

        for (closer, name) in stray_closers {
            let doc = create_mstring(&[closer]);
            let res = validate_dc_mixed_strict(&doc);
            ensure!(
                res.is_err(),
                "Stray closer {name} at document root must fail strict validation"
            );
            let err_msg = format!("{:#}", res.unwrap_err());
            ensure!(
                err_msg.contains("Unexpected closing delimiter"),
                "Error message for {name} must mention unexpected closing delimiter: got {err_msg}"
            );
        }

        Ok(())
    }

    /// Verifies that dangling escape character (Dc 255) at EOF is strictly rejected.
    #[crate::ctb_test]
    fn test_adversarial_dangling_escape_rejection() -> Result<()> {
        let doc = create_mstring(&[DC_ESCAPE]);
        let res = validate_dc_mixed_strict(&doc);
        ensure!(
            res.is_err(),
            "Dangling escape Dc 255 at EOF must fail strict validation"
        );
        let err_msg = format!("{:#}", res.unwrap_err());
        ensure!(
            err_msg.contains("Dangling escape"),
            "Error message must specify dangling escape: got {err_msg}"
        );

        Ok(())
    }
}

// ============================================================================
// 5. IN-PLACE SOURCE EDITING OVER DCSYNTAXARENA
// ============================================================================

mod challenge5_source_editing_over_arena {
    use super::*;

    fn build_kv_map(entries: &[(&str, u64)]) -> Result<DcMstring> {
        let mut mst = DcMstring::new();
        mst.push_char(DC_BEGIN_KV_MAP);
        mst.push_char(DC_TYPE_WILDCARD);
        mst.push_char(DC_TYPE_WILDCARD);
        for &(key, val) in entries {
            mst.push_char(DC_BEGIN_KV_PAIR);
            key.to_string().encode_dc_mixed(&mut mst)?;
            mst.push_char(DC_LIST_ELEMENT_SEPARATOR);
            val.encode_dc_mixed(&mut mst)?;
            mst.push_char(DC_END_KV_PAIR);
        }
        mst.push_char(DC_END_KV_MAP);
        Ok(mst)
    }

    fn do_preserve_source(old: &DcMstring, new: &DcMstring) -> Result<DcMstring> {
        let range = ctb_formats_dcstring::DCMM_HEADER_LEN..old.len();
        old.replace_typed_source_field(range, new)
    }

    #[derive(Debug, Clone, PartialEq, Eq, ctb_formats_dcstring::DcMixed)]
    #[dc(key_value)]
    struct KeyedEditRecord {
        #[dc(key = "title")]
        pub title: String,
        #[dc(key = "count")]
        pub count: u64,
        #[dc(key = "active")]
        pub active: bool,
    }

    /// Tests replacements, deletions, and insertions over `DcSyntaxArena` in `preserve_source`.
    #[crate::ctb_test]
    fn test_adversarial_source_edit_replacements_insertions_deletions() -> Result<()> {
        let strict = DcProjectionPolicy::strict();

        // Original document: Map with keys "alpha": 10, "beta": 20, "gamma": 30
        let original = build_kv_map(&[("alpha", 10), ("beta", 20), ("gamma", 30)])?;

        // Replacement:
        // - "alpha" is deleted
        // - "beta" value replaced with 999
        // - "gamma" value unchanged (30)
        // - "delta" is newly inserted (40)
        let modified = build_kv_map(&[("beta", 999), ("gamma", 30), ("delta", 40)])?;

        let revised = do_preserve_source(&original, &modified)
            .context("do_preserve_source replacement/insertion/deletion")?;

        // Verify that the resulting document parses and contains the expected entries
        let mut nav = DcDocumentNavigator::from_mstr(&revised);
        let mut arena = DcSyntaxArena::new();
        nav.parse_document_into_arena(&mut arena, &strict)?;
        let root = arena.roots()[0];

        let mut ctx = DcProjectionContext::new(&arena, &strict);
        let map: BTreeMap<String, u64> = BTreeMap::<String, u64>::decode_dc_node(&mut ctx, root)?;

        ensure!(!map.contains_key("alpha"), "Deleted key 'alpha' must not be in result");
        ensure!(map.get("beta") == Some(&999), "Updated key 'beta' must have new value 999");
        ensure!(map.get("gamma") == Some(&30), "Unchanged key 'gamma' must retain value 30");
        ensure!(map.get("delta") == Some(&40), "Inserted key 'delta' must be present with 40");

        Ok(())
    }

    /// Tests typed field editing through `DcInspection` without panics.
    #[crate::ctb_test]
    fn test_adversarial_inspection_field_editing() -> Result<()> {
        let strict = DcProjectionPolicy::strict();

        let rec = KeyedEditRecord {
            title: "initial_title".to_string(),
            count: 42,
            active: true,
        };

        let mut doc = DcMstring::new();
        rec.encode_dc_mixed(&mut doc)?;

        let view = doc.inspect::<KeyedEditRecord>();
        ensure!(view.fields().len() == 3, "Expected 3 field annotations");

        let mut updated = rec.clone();
        updated.title = "adversarially_updated_title_with_extra_length".to_string();

        let revised: DcMstring = view.replace_field("title", &updated)?;
        let decoded = revised.decode::<KeyedEditRecord>(&strict)?;
        ensure!(decoded == updated, "Decoded revised record mismatch");

        Ok(())
    }

    /// Verifies that empty, boundary, and malformed inputs never panic in source editing.
    #[crate::ctb_test]
    fn test_adversarial_source_edit_boundary_and_malformed_inputs() -> Result<()> {
        let empty_doc = DcMstring::new();
        let valid_doc = build_kv_map(&[("entry", 100)])?;

        // 1. Both empty -> returns empty
        let both_empty = do_preserve_source(&empty_doc, &empty_doc)?;
        ensure!(both_empty.as_bytes().len() == ctb_formats_dcstring::DCMM_HEADER_LEN);

        // 2. Empty old, valid new -> returns valid new
        let empty_old = do_preserve_source(&empty_doc, &valid_doc)?;
        ensure!(empty_old == valid_doc);

        // 3. Valid old, empty new -> returns empty payload
        let empty_new = do_preserve_source(&valid_doc, &empty_doc)?;
        ensure!(empty_new.payload_bytes().is_empty());

        // 4. Malformed old (unclosed container) -> returns error, NO panic
        let malformed_old = create_mstring(&[DC_BEGIN_LIST, DC_TYPE_WILDCARD]);
        let res_malformed_old = do_preserve_source(&malformed_old, &valid_doc);
        ensure!(
            res_malformed_old.is_err(),
            "Malformed old source must return error, not panic"
        );

        // 5. Malformed new (unclosed container) -> returns error, NO panic
        let res_malformed_new = do_preserve_source(&valid_doc, &malformed_old);
        ensure!(
            res_malformed_new.is_err(),
            "Malformed new source must return error, not panic"
        );

        // 6. Ambiguous duplicate keys in map -> returns error, NO panic
        let mut dup_map = DcMstring::new();
        dup_map.push_char(DC_BEGIN_KV_MAP);
        dup_map.push_char(DC_TYPE_WILDCARD);
        dup_map.push_char(DC_TYPE_WILDCARD);
        for _ in 0..2 {
            dup_map.push_char(DC_BEGIN_KV_PAIR);
            "duplicate_key".to_string().encode_dc_mixed(&mut dup_map)?;
            dup_map.push_char(DC_LIST_ELEMENT_SEPARATOR);
            10u64.encode_dc_mixed(&mut dup_map)?;
            dup_map.push_char(DC_END_KV_PAIR);
        }
        dup_map.push_char(DC_END_KV_MAP);

        let res_dup = do_preserve_source(&dup_map, &valid_doc);
        ensure!(
            res_dup.is_err(),
            "Duplicate map keys must return error, not panic"
        );

        Ok(())
    }
}
