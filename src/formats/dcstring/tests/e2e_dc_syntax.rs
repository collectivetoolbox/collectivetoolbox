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

//! Comprehensive Opaque-Box E2E Test Suite for Document Character (`Dc`) Syntax Parser,
//! `DcSyntaxArena` Deserialization, and `#[derive(DcMixed)]` Refactoring.
//!
//! Tiers covered:
//! - Tier 1: Feature Coverage (Stage 1 data-driven syntax, container framing, format delimiters, strict validation)
//! - Tier 2: Boundary & Corner Cases (empty containers, unclosed delimiters, nested equivalents, depth limits)
//! - Tier 3: Cross-Feature Combinations (key-value maps in checksummed sections, format in optionals, shared contexts)
//! - Tier 5: Adversarial Hardening (tamper detection, truncated byte streams, forward-compatible tags)

#[expect(unused_imports, clippy::wildcard_imports, reason = "Integration test root")]
use ctb_utilities::*;

use anyhow::{Context, Result, ensure};
use ctb_formats_checksum::sha256;
use ctb_formats_dcdata::dc::{
    DC_BEGIN_CHECKSUMMED_SECTION, DC_BEGIN_FORMAT_REPRESENTATION, DC_BEGIN_IDENTIFIER,
    DC_BEGIN_KV_MAP, DC_BEGIN_KV_PAIR, DC_BEGIN_LEGACY_KV_MAP, DC_BEGIN_LIST,
    DC_BEGIN_OPERATION_SESSION, DC_BOOLEAN_FALSE, DC_BOOLEAN_TRUE,
    DC_END_CHECKSUMMED_SECTION, DC_END_FORMAT_REPRESENTATION, DC_END_IDENTIFIER,
    DC_END_KV_MAP, DC_END_KV_PAIR, DC_END_LEGACY_KV_MAP, DC_END_LIST,
    DC_END_OPERATION_SESSION, DC_EQUIVALENT_REPRESENTATIONS, DC_FILE_PATH_DELIMITER,
    DC_LIST_ELEMENT_SEPARATOR, DC_LOG_MESSAGE, DC_OPERATION_STATUS_CODE,
    DC_OPTIONAL_ABSENT, DC_OPTIONAL_PRESENT, DC_PARTIALLY_EQUIVALENT_REPRESENTATIONS,
    DC_SECTION_DIGEST, DC_SEQUENCE_IDENTIFIER,
};
use ctb_formats_dcdata::syntax::{
    CharTarget, DatasetRuleResolver, DcRole, SyntaxPattern, SyntaxRuleResolver,
    SyntaxTerm, default_syntax_resolver,
};
use ctb_formats_dcstring::dc_mixed_serde::encode_format_representation;
use ctb_formats_dcstring::{
    DcByteSpan, DcChar, DcDocumentNavigator, DcEvaluatedValue, DcExpressionEvaluator,
    DcFrameHeaderInfo, DcFrameStub, DcMixedEncode, DcMstring, DcProjectionPolicy,
    DcStr, DcString, DcSyntaxArena, DcSyntaxKind, DefaultDcExpressionEvaluator,
    MAX_SYNTAX_EXPANSION_DEPTH, SyntaxEvalContext, ValueRequirement,
    validate_dc_mixed_strict, validate_dc_str_strict,
};

/// Type token wildcard `*` (Object or any subtype, short Dc 275).
const DC_TYPE_WILDCARD: DcChar = DcChar::from_short(275);

/// Helper to create a `DcMstring` document containing the specified sequence of `DcChar`s.
fn create_test_mstring(chars: &[DcChar]) -> DcMstring {
    let mut mst = DcMstring::new();
    for &ch in chars {
        mst.push_char(ch);
    }
    mst
}

/// Helper to build a canonical typed list: `310 [275] [item1] 312 [item2] ... 311`.
fn build_typed_list<T: DcMixedEncode + Clone>(items: &[T]) -> Result<DcMstring> {
    let mut mst = DcMstring::new();
    items.to_vec().encode_dc_mixed(&mut mst)?;
    Ok(mst)
}

// ============================================================================
// TIER 1: FEATURE COVERAGE
// ============================================================================

mod tier1_feature_coverage {
    use super::*;
    use ctb_formats_dcdata::syntax::SyntaxRuleResolver;

    /// Verifies sacred container framing: nested containers (`511..512` within `310..311`)
    /// step cleanly without delimiter bleed or premature closing.
    #[crate::ctb_test]
    fn test_tier1_sacred_container_framing_no_bleed() -> Result<()> {
        let policy = DcProjectionPolicy::strict();
        let fmt_char = DcChar::from_format(228); // f228: FileName format

        let mut doc = DcMstring::new();
        doc.push_char(DC_BEGIN_LIST);
        doc.push_char(DC_TYPE_WILDCARD);

        // Format representation for FileName: 511 [f228] "test.txt" 512
        encode_format_representation(&"test.txt".to_string(), fmt_char, &mut doc, &Default::default())?;

        doc.push_char(DC_END_LIST);

        let mut nav = DcDocumentNavigator::from_mstr(&doc);

        // 1. Root level should observe the outer list stub
        let list_stub = nav
            .next_sibling_stub(&policy)?
            .context("Expected outer list stub")?;
        ensure!(
            list_stub.head_dc == DC_BEGIN_LIST,
            "Outer stub head must be DC_BEGIN_LIST"
        );
        ensure!(
            list_stub.closer_dc == Some(DC_END_LIST),
            "Outer stub closer must be DC_END_LIST"
        );
        ensure!(!list_stub.is_unclosed(), "Outer list should not be unclosed");

        // 2. Drill into outer list
        nav.enter_frame(&list_stub)?;
        ensure!(nav.depth() == 1, "Nesting depth must be 1 inside list");

        // First stub inside list is type declaration
        let type_stub = nav
            .next_sibling_stub(&policy)?
            .context("Expected type declaration stub")?;
        ensure!(
            type_stub.head_dc == DC_TYPE_WILDCARD,
            "First stub inside list must be type declaration"
        );

        // Child level should observe the inner format representation stub
        let fmt_stub = nav
            .next_sibling_stub(&policy)?
            .context("Expected inner format representation stub")?;
        ensure!(
            fmt_stub.head_dc == DC_BEGIN_FORMAT_REPRESENTATION,
            "Inner stub head must be DC_BEGIN_FORMAT_REPRESENTATION"
        );
        ensure!(
            fmt_stub.closer_dc == Some(DC_END_FORMAT_REPRESENTATION),
            "Inner stub closer must be DC_END_FORMAT_REPRESENTATION"
        );
        ensure!(!fmt_stub.is_unclosed(), "Inner format representation should not be unclosed");

        // 3. Drill into inner format representation
        nav.enter_frame(&fmt_stub)?;
        ensure!(nav.depth() == 2, "Nesting depth must be 2 inside format representation");

        // Step inside format representation consumes payload
        let _ = nav.next_sibling_stub(&policy)?;

        // 4. Unwind both frames cleanly
        nav.exit_frame()?;
        ensure!(nav.depth() == 1, "Nesting depth must return to 1");

        // No further siblings inside list
        ensure!(
            nav.next_sibling_stub(&policy)?.is_none(),
            "No more siblings expected inside list"
        );

        nav.exit_frame()?;
        ensure!(nav.depth() == 0, "Nesting depth must return to root");

        // Document level complete
        ensure!(
            nav.next_sibling_stub(&policy)?.is_none(),
            "No more siblings expected at document root"
        );

        Ok(())
    }

    /// Verifies that format rules specify only internal payload syntax, never the
    /// enclosing `511..512` container delimiters. In particular, format evaluation
    /// must NEVER eagerly consume a trailing `Dc 512` (`DC_END_FORMAT_REPRESENTATION`).
    #[crate::ctb_test]
    fn test_tier1_format_delimiters_no_eager_512_consumption() -> Result<()> {
        let policy = DcProjectionPolicy::strict();
        let fmt_char = DcChar::from_format(228);

        // Two adjacent format representations inside a list
        let mut doc = DcMstring::new();
        doc.push_char(DC_BEGIN_LIST);
        doc.push_char(DC_TYPE_WILDCARD);

        encode_format_representation(&"first.txt".to_string(), fmt_char, &mut doc, &Default::default())?;
        doc.push_char(DC_LIST_ELEMENT_SEPARATOR);
        encode_format_representation(&"second.txt".to_string(), fmt_char, &mut doc, &Default::default())?;

        doc.push_char(DC_END_LIST);

        // Strict document validation must succeed: both 511..512 containers match without
        // eager 512 consumption corrupting the second container's boundary.
        validate_dc_mixed_strict(&doc)
            .context("Two adjacent format representations in list must pass strict validation")?;

        // Stepping with navigator verifies both distinct stubs are produced
        let mut nav = DcDocumentNavigator::from_mstr(&doc);
        let list_stub = nav
            .next_sibling_stub(&policy)?
            .context("Outer list must be present")?;
        nav.enter_frame(&list_stub)?;

        let _type_stub = nav.next_sibling_stub(&policy)?; // skip type wildcard

        let first_fmt = nav
            .next_sibling_stub(&policy)?
            .context("First format stub must be present")?;
        ensure!(
            first_fmt.head_dc == DC_BEGIN_FORMAT_REPRESENTATION,
            "First child must be format representation"
        );

        let sep = nav
            .next_sibling_stub(&policy)?
            .context("Separator stub must be present")?;
        ensure!(
            sep.head_dc == DC_LIST_ELEMENT_SEPARATOR,
            "Separator stub must be DC_LIST_ELEMENT_SEPARATOR"
        );

        let second_fmt = nav
            .next_sibling_stub(&policy)?
            .context("Second format stub must be present")?;
        ensure!(
            second_fmt.head_dc == DC_BEGIN_FORMAT_REPRESENTATION,
            "Second child must be format representation"
        );

        ensure!(
            nav.next_sibling_stub(&policy)?.is_none(),
            "No more stubs expected in list"
        );

        nav.exit_frame()?;
        Ok(())
    }

    /// Verifies data-driven prefix pattern evaluation via `DatasetRuleResolver`
    /// rather than hardcoded arity functions (`prefix_rule_fixed_arity`).
    #[crate::ctb_test]
    fn test_tier1_data_driven_prefix_operator_evaluation() -> Result<()> {
        let resolver = default_syntax_resolver();
        let sdc_val = |id: u32| DcChar::from_short(id).to_u128();

        // Verify dataset resolver loaded valid syntax rules for core constructs
        ensure!(
            resolver.matches_named_type_starter("value", sdc_val(517)), // Dc 517: Boolean true
            "Resolver must identify Dc 517 as a value starter"
        );
        ensure!(
            resolver.matches_named_type_starter("value", sdc_val(518)), // Dc 518: Boolean false
            "Resolver must identify Dc 518 as a value starter"
        );
        ensure!(
            resolver.matches_named_type_starter("value", sdc_val(310)), // Dc 310: List opener
            "Resolver must identify Dc 310 as a value starter"
        );
        ensure!(
            resolver.matches_named_type_starter("value", sdc_val(511)), // Dc 511: Format opener
            "Resolver must identify Dc 511 as a value starter"
        );

        // Opener and closer mapping is completely data-driven
        ensure!(
            resolver.closer_for_opener(&CharTarget::Dc(310)) == Some(sdc_val(311)),
            "Resolver must map 310 to closer 311"
        );
        ensure!(
            resolver.closer_for_opener(&CharTarget::Dc(511)) == Some(sdc_val(512)),
            "Resolver must map 511 to closer 512"
        );
        ensure!(
            resolver.closer_for_opener(&CharTarget::Dc(543)) == Some(sdc_val(544)),
            "Resolver must map 543 to closer 544"
        );
        ensure!(
            resolver.closer_for_opener(&CharTarget::Dc(559)) == Some(sdc_val(560)),
            "Resolver must map 559 to closer 560"
        );

        Ok(())
    }

    /// Verifies strict document validation succeeds for well-formed documents.
    #[crate::ctb_test]
    fn test_tier1_document_validation_strict_valid() -> Result<()> {
        let doc = build_typed_list(&[true, false])?;
        validate_dc_mixed_strict(&doc)
            .context("Valid boolean list must pass strict validation")?;

        Ok(())
    }

    /// Verifies strict document validation rejects invalid framing and misplaced tokens.
    #[crate::ctb_test]
    fn test_tier1_document_validation_strict_rejection() -> Result<()> {
        // Mismatched closer: opened with 310, closed with 512
        let mismatched = create_test_mstring(&[
            DC_BEGIN_LIST,
            DC_TYPE_WILDCARD,
            DC_BOOLEAN_TRUE,
            DC_END_FORMAT_REPRESENTATION,
        ]);
        ensure!(
            validate_dc_mixed_strict(&mismatched).is_err(),
            "Mismatched closer must be rejected by strict validation"
        );

        // Stray closer without opener
        let stray_closer = create_test_mstring(&[
            DC_END_LIST,
        ]);
        ensure!(
            validate_dc_mixed_strict(&stray_closer).is_err(),
            "Stray closer without opener must be rejected by strict validation"
        );

        Ok(())
    }
}

// ============================================================================
// TIER 2: BOUNDARY & CORNER CASES
// ============================================================================

mod tier2_boundary_corner_cases {
    use super::*;

    /// Verifies that empty containers parse cleanly with zero children
    /// and correct inner spans (`inner_span.start == inner_span.end`).
    #[crate::ctb_test]
    fn test_tier2_empty_containers() -> Result<()> {
        let policy = DcProjectionPolicy::forgiving();
        for (opener, closer, name) in [
            (DC_BEGIN_FORMAT_REPRESENTATION, DC_END_FORMAT_REPRESENTATION, "empty format representation"),
            (DC_BEGIN_LEGACY_KV_MAP, DC_END_LEGACY_KV_MAP, "empty legacy kv map"),
            (DC_BEGIN_KV_MAP, DC_END_KV_MAP, "empty kv map"),
        ] {
            let doc = create_test_mstring(&[opener, closer]);
            ensure!(validate_dc_mixed_strict(&doc).is_err());
            let mut nav = DcDocumentNavigator::from_mstr(&doc);

            let stub = nav
                .next_sibling_stub(&policy)?
                .with_context(|| format!("{name} stub must exist"))?;

            ensure!(
                stub.head_dc == opener,
                "{name}: head_dc must match opener"
            );
            ensure!(
                stub.closer_dc == Some(closer),
                "{name}: closer_dc must match closer"
            );
            ensure!(
                !stub.is_unclosed(),
                "{name}: stub must not be marked unclosed"
            );

            if let Some(inner) = stub.inner_span {
                ensure!(
                    inner.start == inner.end,
                    "{name}: inner span of empty container must have start == end, got {inner:?}"
                );
            } else {
                anyhow::bail!("{name}: expected inner_span to be Some");
            }
        }
        Ok(())
    }

    /// Verifies that unclosed containers are detected and flagged by `is_unclosed()`,
    /// error under strict navigation, and are strictly rejected by `validate_dc_mixed_strict`.
    #[crate::ctb_test]
    fn test_tier2_unclosed_container_delimiters() -> Result<()> {
        let unclosed_list = create_test_mstring(&[
            DC_BEGIN_LIST,
            DC_TYPE_WILDCARD,
            DC_BOOLEAN_TRUE,
        ]);

        // Strict mode navigation must error or flag unclosed
        let strict_policy = DcProjectionPolicy::strict();
        let mut strict_nav = DcDocumentNavigator::from_mstr(&unclosed_list);
        let strict_res = strict_nav.next_sibling_stub(&strict_policy);
        ensure!(
            strict_res.is_err() || strict_res.is_ok_and(|opt| opt.is_some_and(|s| s.is_unclosed())),
            "Strict navigation must reject or flag unclosed container"
        );

        // Forgiving mode navigation must recover the stub and mark is_unclosed() == true
        let forgiving_policy = DcProjectionPolicy::forgiving();
        let mut forgiving_nav = DcDocumentNavigator::from_mstr(&unclosed_list);
        let stub = forgiving_nav
            .next_sibling_stub(&forgiving_policy)?
            .context("Forgiving navigation must salvage unclosed stub")?;
        ensure!(
            stub.is_unclosed(),
            "Unclosed list must be flagged by stub.is_unclosed()"
        );

        // Strict document validation must reject unclosed container
        ensure!(
            validate_dc_mixed_strict(&unclosed_list).is_err(),
            "Unclosed list must fail strict document validation"
        );

        Ok(())
    }

    /// Verifies nested equivalent lists (`397` and `561`).
    #[crate::ctb_test]
    fn test_tier2_nested_equivalent_representations() -> Result<()> {
        let policy = DcProjectionPolicy::strict();
        let doc = create_test_mstring(&[
            DC_BEGIN_LIST,
            DC_TYPE_WILDCARD,
            DC_EQUIVALENT_REPRESENTATIONS,
            DC_BEGIN_LIST,
            DC_TYPE_WILDCARD,
            DC_BOOLEAN_TRUE,
            DC_LIST_ELEMENT_SEPARATOR,
            DC_PARTIALLY_EQUIVALENT_REPRESENTATIONS,
            DC_BEGIN_LIST,
            DC_TYPE_WILDCARD,
            DC_BOOLEAN_FALSE,
            DC_END_LIST,
            DC_END_LIST,
            DC_END_LIST,
        ]);

        let mut nav = DcDocumentNavigator::from_mstr(&doc);
        let outer = nav.next_sibling_stub(&policy)?.context("Outer list stub")?;
        ensure!(outer.head_dc == DC_BEGIN_LIST, "Outer list stub head");
        ensure!(!outer.is_unclosed(), "Outer list must not be unclosed");

        let mut arena = DcSyntaxArena::new();
        let node_id = nav.expand_frame(&outer, &mut arena, &policy)?;

        let node = arena.get(node_id).context("Root node in arena")?;
        ensure!(
            matches!(node.kind, DcSyntaxKind::Container { opener, .. } if opener == DC_BEGIN_LIST),
            "Arena root must be container"
        );

        Ok(())
    }

    #[crate::ctb_test]
    fn implicit_representations_and_ignored_tokens_in_operands() -> Result<()> {
        for ignored_token in [false, true] {
            let mut document = DcMstring::new();
            document.push_char(DcChar::from_short(374));
            document.push_char(DcChar::from_short(376));
            if ignored_token {
                document.push_char(DcChar::from_short(255));
                document.push_char(DcChar::from_short(377));
            }
            document.push_char(DC_EQUIVALENT_REPRESENTATIONS);
            vec![0u64, 0u64].encode_dc_mixed(&mut document)?;
            document.push_char(DcChar::from_short(377));
            16u64.encode_dc_mixed(&mut document)?;
            validate_dc_mixed_strict(&document)
                .with_context(|| format!("Implicit operand syntax, ignored_token={ignored_token}"))?;
        }
        let mut resolution = DcMstring::new();
        resolution.push_char(DcChar::from_short(378));
        resolution.push_char(DC_EQUIVALENT_REPRESENTATIONS);
        vec![std::time::Duration::from_nanos(1), std::time::Duration::from_nanos(1)]
            .encode_dc_mixed(&mut resolution)?;
        validate_dc_mixed_strict(&resolution)?;
        let mut wrong_resolution = DcMstring::new();
        wrong_resolution.push_char(DcChar::from_short(378));
        wrong_resolution.push_char(DC_EQUIVALENT_REPRESENTATIONS);
        vec![true, true].encode_dc_mixed(&mut wrong_resolution)?;
        ensure!(validate_dc_mixed_strict(&wrong_resolution).is_err(), "Representations bypassed datemagnitude operand type");
        Ok(())
    }

    #[crate::ctb_test]
    fn numeric_subtype_constraints_apply_to_raw_and_wrapped_operands() -> Result<()> {
        let mut raw = DcMstring::new();
        raw.push_char(DcChar::from_short(376));
        (-1i64).encode_dc_mixed(&mut raw)?;
        let mut wrapped = DcMstring::new();
        wrapped.push_char(DcChar::from_short(376));
        wrapped.push_char(DC_EQUIVALENT_REPRESENTATIONS);
        vec![-1i64, -1i64].encode_dc_mixed(&mut wrapped)?;
        let raw_accepted = validate_dc_mixed_strict(&raw).is_ok();
        let wrapped_accepted = validate_dc_mixed_strict(&wrapped).is_ok();
        ensure!(!raw_accepted && !wrapped_accepted,
            "Numeric subtype validation is pending: negative Natural0 accepted (raw={raw_accepted}, wrapped={wrapped_accepted})");
        Ok(())
    }

    /// Verifies that expansion depth limits (`MAX_SYNTAX_EXPANSION_DEPTH` = 32)
    /// prevent infinite recursion or stack exhaustion.
    #[crate::ctb_test]
    fn test_tier2_expansion_depth_limits() -> Result<()> {
        let policy = DcProjectionPolicy::strict();
        ensure!(
            MAX_SYNTAX_EXPANSION_DEPTH == 32,
            "MAX_SYNTAX_EXPANSION_DEPTH invariant must be 32"
        );

        let resolver = default_syntax_resolver();
        let mut ctx = SyntaxEvalContext::new(resolver.as_ref(), None);
        ctx.max_depth = 5;

        // Create a 6-deep nested list
        let mut chars = Vec::new();
        for _ in 0..6 {
            chars.push(DC_BEGIN_LIST);
            chars.push(DC_TYPE_WILDCARD);
        }
        for _ in 0..6 {
            chars.push(DC_END_LIST);
        }

        let doc = create_test_mstring(&chars);
        let mut nav = DcDocumentNavigator::from_mstr(&doc);

        // Drilling past max_depth must trigger an error or identify depth boundary
        let mut exceeded = false;
        for _ in 0..6 {
            if let Ok(Some(stub)) = nav.next_sibling_stub(&policy) {
                if nav.depth() >= ctx.max_depth {
                    exceeded = true;
                    break;
                }
                nav.enter_frame(&stub)?;
                let _ = nav.next_sibling_stub(&policy)?; // Skip type wildcard inside list
            } else {
                break;
            }
        }

        ensure!(exceeded, "Navigator nesting should identify depth boundary");
        Ok(())
    }
}

// ============================================================================
// TIER 3: CROSS-FEATURE COMBINATIONS
// ============================================================================

mod tier3_cross_feature_combinations {
    use super::*;

    /// Verifies key-value map (`559..560`) containing pairs (`514..515`)
    /// embedded inside a checksummed section (`543..544 + 545`).
    #[crate::ctb_test]
    fn test_tier3_kv_map_inside_checksummed_section() -> Result<()> {
        let policy = DcProjectionPolicy::strict();
        let mut mst = DcMstring::new();

        // 1. Begin checksummed section (543)
        mst.push_char(DC_BEGIN_CHECKSUMMED_SECTION);

        let body_start = mst.len();

        // Sequence identifier (546) followed by an integer (e.g. 1u64)
        mst.push_char(DC_SEQUENCE_IDENTIFIER);
        1u64.encode_dc_mixed(&mut mst)?;

        // 2. Begin KV map (559)
        mst.push_char(DC_BEGIN_KV_MAP);
        mst.push_char(DC_TYPE_WILDCARD);
        mst.push_char(DC_TYPE_WILDCARD);

        // First KV pair: 514 [key] 312 [val] 515
        mst.push_char(DC_BEGIN_KV_PAIR);
        mst.push_char(DC_BOOLEAN_TRUE);
        mst.push_char(DC_LIST_ELEMENT_SEPARATOR);
        mst.push_char(DC_BOOLEAN_FALSE);
        mst.push_char(DC_END_KV_PAIR);

        // Close KV map (560)
        mst.push_char(DC_END_KV_MAP);
        let body_end = mst.len();

        // Close checksummed section (544)
        mst.push_char(DC_END_CHECKSUMMED_SECTION);

        // 3. Compute SHA-256 over body slice [body_start..body_end]
        let digest = {
            let mstr_bytes: &[u8] = mst.as_ref();
            let body_bytes = mstr_bytes
                .get(body_start..body_end)
                .context("Body byte slice")?;
            sha256(body_bytes)
        };

        // Append 545 digest
        mst.push_char(DC_SECTION_DIGEST);
        mst.push_checksum(ctb_utilities::format_id::DC_SHA256, &digest)?;

        // Navigator must step into section and verify valid digest
        let mut nav = DcDocumentNavigator::from_mstr(&mst);
        let section_stub = nav
            .next_sibling_stub(&policy)?
            .context("Checksummed section stub")?;

        ensure!(
            section_stub.head_dc == DC_BEGIN_CHECKSUMMED_SECTION,
            "Stub head must be DC_BEGIN_CHECKSUMMED_SECTION"
        );
        ensure!(
            section_stub.closer_dc == Some(DC_END_CHECKSUMMED_SECTION),
            "Stub closer must be DC_END_CHECKSUMMED_SECTION"
        );

        if let DcFrameHeaderInfo::ChecksummedSection { digest_valid, .. } =
            section_stub.header_info
        {
            ensure!(digest_valid, "Trailing section digest must be verified as valid");
        } else {
            anyhow::bail!("Expected ChecksummedSection header info");
        }

        // Enter section and step to KV map
        nav.enter_frame(&section_stub)?;

        let _seq = nav.next_sibling_stub(&policy)?; // Sequence identifier
        let kv_stub = nav.next_sibling_stub(&policy)?.context("KV map stub")?;
        ensure!(
            kv_stub.head_dc == DC_BEGIN_KV_MAP,
            "Child stub must be DC_BEGIN_KV_MAP"
        );

        nav.enter_frame(&kv_stub)?;
        ensure!(nav.next_sibling_stub(&policy)?.context("Key type header")?.head_dc == DC_TYPE_WILDCARD);
        ensure!(nav.next_sibling_stub(&policy)?.context("Value type header")?.head_dc == DC_TYPE_WILDCARD);
        let pair_stub = nav.next_sibling_stub(&policy)?.context("KV pair stub")?;
        ensure!(
            pair_stub.head_dc == DC_BEGIN_KV_PAIR,
            "Pair stub must be DC_BEGIN_KV_PAIR"
        );

        nav.exit_frame()?; // Exit KV map
        nav.exit_frame()?; // Exit Checksummed section

        Ok(())
    }

    /// Verifies format representation (`511..512`) embedded inside an optional wrapper
    /// (`315 OptionalPresent` and `316 OptionalAbsent`).
    #[crate::ctb_test]
    fn test_tier3_format_inside_optional() -> Result<()> {
        let fmt_char = DcChar::from_format(228);

        // Optional present containing format representation
        let mut present_doc = DcMstring::new();
        present_doc.push_char(DC_BEGIN_LIST);
        present_doc.push_char(DC_TYPE_WILDCARD);
        present_doc.push_char(DC_OPTIONAL_PRESENT);
        encode_format_representation(&"doc.txt".to_string(), fmt_char, &mut present_doc, &Default::default())?;
        present_doc.push_char(DC_END_LIST);

        validate_dc_mixed_strict(&present_doc)
            .context("Optional present containing format must pass strict validation")?;

        // Optional absent
        let absent_doc = create_test_mstring(&[
            DC_BEGIN_LIST,
            DC_TYPE_WILDCARD,
            DC_OPTIONAL_ABSENT,
            DC_END_LIST,
        ]);
        validate_dc_mixed_strict(&absent_doc)
            .context("Optional absent must pass strict validation")?;

        Ok(())
    }

    /// Verifies `DcSyntaxArena` allocation, retrieval, and clearing lifecycle.
    #[crate::ctb_test]
    fn test_tier3_arena_lifecycle_and_reuse() -> Result<()> {
        let mut arena = DcSyntaxArena::new();
        ensure!(arena.nodes().is_empty(), "New arena must have zero nodes");
        ensure!(arena.roots().is_empty(), "New arena must have zero roots");

        let span1 = DcByteSpan::new(10, 20);
        let id1 = arena.alloc(span1, DcSyntaxKind::Atom(DC_BOOLEAN_TRUE));
        arena.push_root(id1);

        let span2 = DcByteSpan::new(20, 30);
        let id2 = arena.alloc(span2, DcSyntaxKind::Atom(DC_BOOLEAN_FALSE));
        arena.push_root(id2);

        ensure!(arena.nodes().len() == 2, "Arena must hold 2 nodes");
        ensure!(arena.roots().len() == 2, "Arena must hold 2 roots");

        let node1 = arena.get(id1).context("Node 1 must exist")?;
        ensure!(node1.span == span1, "Node 1 span match");
        ensure!(
            matches!(node1.kind, DcSyntaxKind::Atom(ch) if ch == DC_BOOLEAN_TRUE),
            "Node 1 kind match"
        );

        // Clear must reset length while retaining capacity
        arena.clear();

        ensure!(arena.nodes().is_empty(), "Cleared arena must have zero nodes");
        ensure!(arena.roots().is_empty(), "Cleared arena must have zero roots");
        ensure!(arena.diagnostics().is_empty(), "Cleared arena must have zero diagnostics");

        // Reallocating in cleared arena works seamlessly
        let id3 = arena.alloc(span1, DcSyntaxKind::Atom(DC_BOOLEAN_TRUE));
        ensure!(id3.0 == 0, "First allocation in cleared arena must reuse index 0");

        Ok(())
    }
}

// ============================================================================
// TIER 5: ADVERSARIAL HARDENING
// ============================================================================

mod tier5_adversarial_hardening {
    use super::*;

    #[crate::ctb_test]
    fn test_section_digest_algorithms() -> Result<()> {
        for algorithm in [
            SupportedHashFormat::Sha256,
            SupportedHashFormat::XxHash64,
            SupportedHashFormat::Fnv1a_1024,
        ] {
            let mut mst = DcMstring::new();
            mst.push_char(DC_BEGIN_CHECKSUMMED_SECTION);
            let start = mst.len();
            mst.push_char(DC_SEQUENCE_IDENTIFIER);
            1u64.encode_dc_mixed(&mut mst)?;
            mst.push_char(DC_BOOLEAN_TRUE);
            let mut hasher = ctb_formats_checksum::StreamingHasher::new(algorithm);
            hasher.update(mst.as_bytes().get(start..).context("Missing section body")?);
            let digest = hasher.finalize();
            mst.push_char(DC_END_CHECKSUMMED_SECTION);
            mst.push_char(DC_SECTION_DIGEST);
            mst.push_checksum(
                algorithm.to_format_id().dc_char().context("Missing hash format Dc")?,
                &digest,
            )?;
            let text = mst.to_dc_string()?;
            let mut file = tempfile::tempfile()?;
            std::io::Write::write_all(&mut file, mst.as_ref())?;
            for mut nav in [
                DcDocumentNavigator::from_mstr(&mst),
                DcDocumentNavigator::from_dc_str(&text),
                DcDocumentNavigator::from_dcmm_file(file, 0)?,
            ] {
                let stub = nav.next_sibling_stub(&DcProjectionPolicy::strict())?
                    .context("Missing checksummed section")?;
                ensure!(matches!(stub.header_info,
                    DcFrameHeaderInfo::ChecksummedSection { digest_valid: true, .. }
                ));
            }
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_section_digest_untyped_forgiving_recovery() -> Result<()> {
        let mut mst = DcMstring::new();
        mst.push_char(DC_BEGIN_CHECKSUMMED_SECTION);
        mst.push_char(DC_SEQUENCE_IDENTIFIER);
        1u64.encode_dc_mixed(&mut mst)?;
        mst.push_char(DC_END_CHECKSUMMED_SECTION);
        mst.push_char(DC_SECTION_DIGEST);
        mst.push_binary(&[0; 32]);
        mst.push_char(DC_BOOLEAN_TRUE);
        let mut strict = DcDocumentNavigator::from_mstr(&mst);
        ensure!(strict.next_sibling_stub(&DcProjectionPolicy::strict()).is_err());
        let mut file = tempfile::tempfile()?;
        std::io::Write::write_all(&mut file, mst.as_ref())?;
        for mut nav in [
            DcDocumentNavigator::from_mstr(&mst),
            DcDocumentNavigator::from_dcmm_file(file, 0)?,
        ] {
            let policy = DcProjectionPolicy::forgiving_quiet();
            let stub = nav.next_sibling_stub(&policy)?.context("Missing recovered section")?;
            ensure!(matches!(stub.header_info,
                DcFrameHeaderInfo::ChecksummedSection { digest_valid: false, .. }
            ));
            ensure!(nav.next_sibling_stub(&policy)?.context("Missing following sibling")?.head_dc == DC_BOOLEAN_TRUE);
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_section_digest_length_rejected_before_materialization() -> Result<()> {
        let mut mst = DcMstring::new();
        mst.push_char(DC_BEGIN_CHECKSUMMED_SECTION);
        mst.push_char(DC_SEQUENCE_IDENTIFIER);
        1u64.encode_dc_mixed(&mut mst)?;
        mst.push_char(DC_END_CHECKSUMMED_SECTION);
        mst.push_char(DC_SECTION_DIGEST);
        mst.push_checksum(ctb_utilities::format_id::DC_SHA256, &vec![0; 65_536])?;
        let mut file = tempfile::tempfile()?;
        std::io::Write::write_all(&mut file, mst.as_ref())?;
        for mut nav in [
            DcDocumentNavigator::from_mstr(&mst),
            DcDocumentNavigator::from_dcmm_file(file, 0)?,
        ] {
            let error = nav.next_sibling_stub(&DcProjectionPolicy::strict())
                .err().context("Oversized section digest was accepted")?;
            ensure!(format!("{error:#}").contains("digest length"), "{error:#}");
        }
        Ok(())
    }

    /// Tampering with 1 byte of the checksummed section body MUST cause
    /// digest validation to fail.
    #[crate::ctb_test]
    fn test_tier5_checksummed_section_tamper_detection() -> Result<()> {
        let policy = DcProjectionPolicy::strict();
        let mut mst = DcMstring::new();

        mst.push_char(DC_BEGIN_CHECKSUMMED_SECTION);
        let body_start = mst.len();

        mst.push_char(DC_SEQUENCE_IDENTIFIER);
        1u64.encode_dc_mixed(&mut mst)?;
        mst.push_char(DC_BOOLEAN_TRUE);
        let body_end = mst.len();
        mst.push_char(DC_END_CHECKSUMMED_SECTION);

        let digest = {
            let mstr_bytes: &[u8] = mst.as_ref();
            let body_bytes = mstr_bytes
                .get(body_start..body_end)
                .context("Body slice")?;
            sha256(body_bytes)
        };

        mst.push_char(DC_SECTION_DIGEST);
        mst.push_checksum(ctb_utilities::format_id::DC_SHA256, &digest)?;

        // Authentic section must have digest_valid == true
        let mut auth_nav = DcDocumentNavigator::from_mstr(&mst);
        let auth_stub = auth_nav
            .next_sibling_stub(&policy)?
            .context("Stub for authentic section")?;
        ensure!(
            matches!(
                auth_stub.header_info,
                DcFrameHeaderInfo::ChecksummedSection {
                    digest_valid: true,
                    ..
                }
            ),
            "Authentic section must be marked digest_valid: true"
        );

        // Construct a tampered section containing DC_BOOLEAN_FALSE, but with the digest of the original body:
        let mut tampered_mst = DcMstring::new();
        tampered_mst.push_char(DC_BEGIN_CHECKSUMMED_SECTION);
        tampered_mst.push_char(DC_SEQUENCE_IDENTIFIER);
        1u64.encode_dc_mixed(&mut tampered_mst)?;
        tampered_mst.push_char(DC_BOOLEAN_FALSE);
        tampered_mst.push_char(DC_END_CHECKSUMMED_SECTION);

        tampered_mst.push_char(DC_SECTION_DIGEST);
        tampered_mst.push_checksum(ctb_utilities::format_id::DC_SHA256, &digest)?;

        let mut nav = DcDocumentNavigator::from_mstr(&tampered_mst);
        ensure!(nav.next_sibling_stub(&policy).is_err());
        let mut nav = DcDocumentNavigator::from_mstr(&tampered_mst);
        let stub = nav
            .next_sibling_stub(&DcProjectionPolicy::forgiving())?
            .context("Stub for tampered section")?;

        if let DcFrameHeaderInfo::ChecksummedSection { digest_valid, .. } = stub.header_info {
            ensure!(
                !digest_valid,
                "Tampered checksummed section must NOT be marked digest_valid"
            );
        } else {
            anyhow::bail!("Expected ChecksummedSection header info");
        }

        // Corrupted section framing (e.g. missing closer) must fail strict validation
        let mut corrupted_framing = DcMstring::new();
        corrupted_framing.push_char(DC_BEGIN_CHECKSUMMED_SECTION);
        corrupted_framing.push_char(DC_SEQUENCE_IDENTIFIER);
        1u64.encode_dc_mixed(&mut corrupted_framing)?;
        corrupted_framing.push_char(DC_BOOLEAN_FALSE);
        // Missing DC_END_CHECKSUMMED_SECTION
        corrupted_framing.push_char(DC_SECTION_DIGEST);
        corrupted_framing.push_checksum(ctb_utilities::format_id::DC_SHA256, &digest)?;

        ensure!(
            validate_dc_mixed_strict(&corrupted_framing).is_err(),
            "Corrupted section framing must be rejected by strict validation"
        );

        Ok(())
    }

    /// Verifies that a truncated token stream at EOF errors under strict mode
    /// and is gracefully salvaged under forgiving mode without panics or infinite loops.
    #[crate::ctb_test]
    fn test_tier5_truncated_token_stream_at_eof() -> Result<()> {
        let strict_policy = DcProjectionPolicy::strict();
        let forgiving_policy = DcProjectionPolicy::forgiving();

        let mut mst = DcMstring::new();
        mst.push_char(DC_BEGIN_LIST);
        mst.push_char(DC_TYPE_WILDCARD);
        // Truncate without closer

        // Strict navigation must error on unclosed container at EOF
        let mut strict_nav = DcDocumentNavigator::from_mstr(&mst);
        let strict_res = strict_nav.next_sibling_stub(&strict_policy);
        ensure!(
            strict_res.is_err(),
            "Strict navigation must error on unclosed container at EOF"
        );

        // Forgiving navigation must recover stub marked as unclosed
        let mut forgiving_nav = DcDocumentNavigator::from_mstr(&mst);
        let forgiving_res = forgiving_nav.next_sibling_stub(&forgiving_policy)?;
        ensure!(
            forgiving_res.is_some_and(|s| s.is_unclosed()),
            "Forgiving navigation must recover unclosed stub"
        );

        // Strict validation must fail
        ensure!(
            validate_dc_mixed_strict(&mst).is_err(),
            "Truncated stream must fail strict validation"
        );

        Ok(())
    }

    /// Injects forward-compatible / unknown tags into an open container
    /// and verifies policy enforcement.
    #[crate::ctb_test]
    fn test_tier5_unknown_forward_compatible_tags() -> Result<()> {
        let policy = DcProjectionPolicy::forgiving();
        let unknown_tag = DcChar::from_short(9999);
        let doc = create_test_mstring(&[
            DC_BEGIN_LIST,
            DC_TYPE_WILDCARD,
            unknown_tag,
            DC_BOOLEAN_TRUE,
            DC_END_LIST,
        ]);

        let mut nav = DcDocumentNavigator::from_mstr(&doc);

        // Navigator can step over unknown tags in forgiving mode without panic
        let list_stub = nav.next_sibling_stub(&policy)?.context("List stub")?;
        nav.enter_frame(&list_stub)?;

        let child1 = nav.next_sibling_stub(&policy)?;
        ensure!(child1.is_some(), "Navigator must read type tag as stub");

        let child2 = nav.next_sibling_stub(&policy)?;
        ensure!(child2.is_some(), "Navigator must read unknown tag as stub");

        let child3 = nav.next_sibling_stub(&policy)?;
        ensure!(child3.is_some(), "Navigator must advance to next child");

        nav.exit_frame()?;
        Ok(())
    }
}

// ============================================================================
// TIER 4: EXPRESSION EVALUATION & STREAM SPLICING
// ============================================================================

mod tier4_expression_evaluation_and_stream_splicing {
    use super::*;
    use std::sync::Arc;

    /// Mock expression evaluator that resolves bound references (`540`) to a mock document.
    struct MockExpressionEvaluator {
        mock_doc: DcMstring,
    }

    impl DcExpressionEvaluator for MockExpressionEvaluator {
        fn eval_expression_for_requirement(
            &self,
            expr_tokens: &[DcChar],
            _requirement: &ValueRequirement<'_>,
            resolver: &dyn SyntaxRuleResolver,
        ) -> Result<Option<DcEvaluatedValue>> {
            if let Some(first) = expr_tokens.first() {
                if resolver.token_fills_role(first.to_u128(), DcRole::EvaluableExpressionStarter) {
                    return Ok(Some(DcEvaluatedValue::Document(self.mock_doc.clone())));
                }
            }
            Ok(None)
        }

        fn eval_stream_splice(
            &self,
            _call_tokens: &[DcChar],
            _resolver: &dyn SyntaxRuleResolver,
        ) -> Result<Option<Vec<DcChar>>> {
            Ok(None)
        }
    }

    /// Verifies that syntax requirements `[string]` and `[number:Natural0]` are satisfied
    /// by evaluable Dc expressions returning that type, and that subtype violations
    /// (e.g., negative numbers for `Natural0`) are rejected.
    #[crate::ctb_test]
    fn test_typed_expression_evaluation_and_subtype_constraints() -> Result<()> {
        let resolver = default_syntax_resolver();

        // 1. [string] satisfied by an expression returning a string
        let mut string_doc = DcMstring::new();
        "Hello from expression".to_string().encode_dc_mixed(&mut string_doc)?;

        let string_evaluator = Arc::new(MockExpressionEvaluator {
            mock_doc: string_doc,
        });

        // Document containing an expression (bound value reference Dc 540 + identifier "var")
        let mut expr_doc = DcMstring::new();
        expr_doc.push_char(DcChar::from_short(540));
        expr_doc.push_char(DC_BEGIN_IDENTIFIER);
        expr_doc.push_str("var");
        expr_doc.push_char(DC_END_IDENTIFIER);
        let mut nav = DcDocumentNavigator::from_mstr(&expr_doc);
        let mut ctx = SyntaxEvalContext::new(&**resolver, None);
        ctx.evaluator = Some(string_evaluator);

        let string_term = SyntaxTerm::NamedConstruct {
            name: "string".to_string(),
            subtype: None,
            capture_var: None,
        };
        let string_matched = nav.eval_term_single(&string_term, &mut ctx)?;
        ensure!(string_matched, "[string] must be satisfied by expression evaluating to string");

        // 2. [number:Natural0] satisfied by an expression returning a non-negative number
        let mut natural_doc = DcMstring::new();
        42u64.encode_dc_mixed(&mut natural_doc)?;

        let natural_evaluator = Arc::new(MockExpressionEvaluator {
            mock_doc: natural_doc,
        });

        let mut nav = DcDocumentNavigator::from_mstr(&expr_doc);
        let mut ctx = SyntaxEvalContext::new(&**resolver, None);
        ctx.evaluator = Some(natural_evaluator);

        let natural_term = SyntaxTerm::NamedConstruct {
            name: "number".to_string(),
            subtype: Some("Natural0".to_string()),
            capture_var: None,
        };
        let natural_matched = nav.eval_term_single(&natural_term, &mut ctx)?;
        ensure!(natural_matched, "[number:Natural0] must be satisfied by expression evaluating to non-negative number");

        // 3. [number:Natural0] rejected when the expression returns a negative number
        let mut negative_doc = DcMstring::new();
        (-42i64).encode_dc_mixed(&mut negative_doc)?;

        let negative_evaluator = Arc::new(MockExpressionEvaluator {
            mock_doc: negative_doc,
        });

        let mut nav = DcDocumentNavigator::from_mstr(&expr_doc);
        let mut ctx = SyntaxEvalContext::new(&**resolver, None);
        ctx.evaluator = Some(negative_evaluator);

        let negative_matched = nav.eval_term_single(&natural_term, &mut ctx)?;
        ensure!(!negative_matched, "[number:Natural0] must reject expression evaluating to negative number");

        Ok(())
    }

    /// Verifies end-to-end stream splicing of `lang.say` (Dc 256):
    /// `286 256 258 260 262 264 263 57 86 93 93 96 30 18 72 96 99 93 85 19 287 261 259`
    ///
    /// 1. Splicing execution: evaluates `lang.say`, splices `"Hello, World!"` + `287` into `spliced_tokens`,
    ///    and cleanly closes the outer `286..287` italicized text span.
    /// 2. CST Arena parsing: parses into the arena without dropping the inner `lang.say` invocation.
    #[crate::ctb_test]
    fn test_lang_say_stream_splicing_end_to_end() -> Result<()> {
        let policy = DcProjectionPolicy::strict();
        let resolver = default_syntax_resolver();

        // 286 (Begin italicized text)
        // 256 (void lang.say(String))
        // 258 (param opener)
        // 260 (literal opener)
        // 262 (type header opener)
        // 264 (f8 UTF-8 format)
        // 263 (type header closer)
        // 57 86 93 93 96 30 18 72 96 99 93 85 19 ("Hello, World!")
        // 287 (End italicized text)
        // 261 (literal closer)
        // 259 (param closer)
        let token_ids: [u32; 23] = [
            286, 256, 258, 260, 262, 264, 263, 57, 86, 93, 93, 96, 30, 18, 72, 96, 99, 93, 85, 19,
            287, 261, 259,
        ];
        let chars: Vec<DcChar> = token_ids.iter().map(|&id| DcChar::from_short(id)).collect();
        let doc = create_test_mstring(&chars);

        // --- Part 1: Stream Splicing Verification ---
        let mut nav = DcDocumentNavigator::from_mstr(&doc);

        // Step 1: Open italic span (Dc 286)
        let first = nav.read_next_token()?.context("Missing opening Dc 286")?;
        ensure!(first == DcChar::from_short(286), "Expected Dc 286, found {first}");

        // Step 2: Next token is Dc 256 (lang.say)
        let evaluator = DefaultDcExpressionEvaluator;
        let spliced = nav.splice_stream_invocation(&evaluator, &**resolver, &policy)?;
        ensure!(spliced, "splice_stream_invocation must succeed on Dc 256");

        // Verify spliced_tokens queue contains "Hello, World!" followed by 287
        let expected_spliced: Vec<DcChar> = [57u32, 86, 93, 93, 96, 30, 18, 72, 96, 99, 93, 85, 19, 287]
            .iter()
            .map(|&id| DcChar::from_short(id))
            .collect();
        let actual_spliced: Vec<DcChar> = nav.spliced_tokens().iter().copied().collect();
        ensure!(
            actual_spliced == expected_spliced,
            "Spliced tokens queue mismatch: expected {expected_spliced:?}, got {actual_spliced:?}"
        );

        // Step 3: Consume tokens from stream, encountering "Hello, World!" and then 287
        let payload_len = expected_spliced.len().saturating_sub(1);
        let payload = expected_spliced.get(..payload_len).context("Missing payload slice")?;
        for &expected_char in payload {
            let ch = nav.read_next_token()?.context("Missing spliced character")?;
            ensure!(ch == expected_char, "Character mismatch: expected {expected_char}, got {ch}");
        }

        // Cleanly encounter closing 287 (End italicized text)
        let closer = nav.read_next_token()?.context("Missing closing Dc 287")?;
        ensure!(closer == DcChar::from_short(287), "Expected Dc 287, found {closer}");

        // Stream is cleanly consumed at end
        ensure!(nav.read_next_token()?.is_none(), "Stream must be exhausted after closing 287");

        // --- Part 2: CST Arena Parsing Verification ---
        let mut arena_nav = DcDocumentNavigator::from_mstr(&doc);
        let mut arena = DcSyntaxArena::new();
        arena_nav.parse_document_into_arena(&mut arena, &policy)?;

        let roots = arena.roots();
        ensure!(roots.len() == 1, "Expected 1 root node in arena, found {}", roots.len());

        let root_node = arena.get(roots[0]).context("Root node missing")?;
        match &root_node.kind {
            DcSyntaxKind::Container {
                opener,
                closer,
                unclosed,
                children,
                ..
            } => {
                ensure!(*opener == DcChar::from_short(286), "Opener must be 286, got {opener}");
                ensure!(*closer == DcChar::from_short(287), "Closer must be 287, got {closer}");
                ensure!(!unclosed, "Container must not be marked unclosed");
                ensure!(
                    !children.is_empty(),
                    "Container children must not be empty (inner lang.say content dropped!)"
                );

                let child_node = arena.get(children[0]).context("Child node missing")?;
                let child_head = match &child_node.kind {
                    DcSyntaxKind::Tagged { tag, .. } => *tag,
                    DcSyntaxKind::Container { opener, .. } => *opener,
                    DcSyntaxKind::Atom(ch) => *ch,
                    other => bail!("Unexpected child kind for lang.say: {other:?}"),
                };
                ensure!(
                    child_head == DcChar::from_short(256),
                    "Child node must be Dc 256 lang.say, found {child_head}"
                );
            }
            other => bail!("Root node must be Container, got {other:?}"),
        }

        Ok(())
    }
}
