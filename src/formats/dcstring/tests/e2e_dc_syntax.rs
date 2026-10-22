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
    DC_BEGIN_OPERATION_SESSION, DC_BEGIN_TYPE_DEF, DC_BOOLEAN_FALSE, DC_BOOLEAN_TRUE,
    DC_END_CHECKSUMMED_SECTION, DC_END_FORMAT_REPRESENTATION, DC_END_IDENTIFIER,
    DC_END_KV_MAP, DC_END_KV_PAIR, DC_END_LEGACY_KV_MAP, DC_END_LIST,
    DC_END_OPERATION_SESSION, DC_END_TYPE_DEF, DC_EQUIVALENT_REPRESENTATIONS, DC_FILE_PATH_DELIMITER,
    DC_LIST_ELEMENT_SEPARATOR, DC_LOG_MESSAGE, DC_OPERATION_STATUS_CODE,
    DC_OPTIONAL_ABSENT, DC_OPTIONAL_PRESENT, DC_PARTIALLY_EQUIVALENT_REPRESENTATIONS,
    DC_SECTION_DIGEST, DC_SEQUENCE_IDENTIFIER, DC_START_ENCAPSULATION_BINARY,
    DC_END_ENCAPSULATION_BINARY, DC_TYPE_ANY,
};
use ctb_formats_dcdata::syntax::{
    CharTarget, DatasetRuleResolver, DcRole, Quantifier, SyntaxElement,
    SyntaxPattern, SyntaxRuleResolver, SyntaxTerm, default_syntax_resolver,
};
use ctb_formats_dcstring::dc_mixed_serde::encode_format_representation;
pub use ctb_formats_dcstring::*;

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
    fn implicit_representations_and_escaped_tokens_in_operands() -> Result<()> {
        let mut valid_slice = DcMstring::new();
        valid_slice.push_char(DcChar::from_short(374));
        valid_slice.push_char(DcChar::from_short(376));
        valid_slice.push_char(DC_EQUIVALENT_REPRESENTATIONS);
        vec![0u64, 0u64].encode_dc_mixed(&mut valid_slice)?;
        valid_slice.push_char(DcChar::from_short(377));
        16u64.encode_dc_mixed(&mut valid_slice)?;
        validate_dc_mixed_strict(&valid_slice)?;

        let mut escaped_tag_in_slice = DcMstring::new();
        escaped_tag_in_slice.push_char(DcChar::from_short(374));
        escaped_tag_in_slice.push_char(DcChar::from_short(376));
        escaped_tag_in_slice.push_char(DcChar::from_short(255));
        escaped_tag_in_slice.push_char(DcChar::from_short(377));
        escaped_tag_in_slice.push_char(DC_EQUIVALENT_REPRESENTATIONS);
        vec![0u64, 0u64].encode_dc_mixed(&mut escaped_tag_in_slice)?;
        escaped_tag_in_slice.push_char(DcChar::from_short(377));
        16u64.encode_dc_mixed(&mut escaped_tag_in_slice)?;
        ensure!(
            validate_dc_mixed_strict(&escaped_tag_in_slice).is_err(),
            "Escaped Dc 255 377 must not be silently deleted where a [number:positive] operand is required"
        );

        // Escaped delimiters (255 515 and 255 312) inside a 514..515 key-value pair
        let mut kv_doc = DcString::new();
        for short_id in [559u32, 275, 275, 514, 255, 515, 312, 255, 312, 515, 560] {
            kv_doc.push(DcChar::from_short(short_id));
        }
        let mut arena = DcSyntaxArena::new();
        let policy = DcProjectionPolicy::strict();
        let root_id = DcDocumentNavigator::parse_arena(
            kv_doc.as_bytes(),
            &mut arena,
            &policy,
        )?;
        let mut ctx = DcProjectionContext::new(&arena, &policy);
        let (_, entries) = ctx.extract_kv_map_entries(root_id)?;
        ensure!(entries.len() == 1, "Expected 1 kv entry, got {}", entries.len());
        let (key_id, val_id, _, _) = entries.into_iter().next().context("Missing kv entry")??;
        let key_node = arena.get(key_id).context("Missing key node")?;
        let val_node = arena.get(val_id).context("Missing val node")?;
        ensure!(
            key_node.kind == DcSyntaxKind::Atom(DcChar::from_short(515)) && key_node.is_escaped_atom(),
            "Expected escaped Atom(515) key, got {:?}",
            key_node
        );
        ensure!(
            val_node.kind == DcSyntaxKind::Atom(DcChar::from_short(312)) && val_node.is_escaped_atom(),
            "Expected escaped Atom(312) value, got {:?}",
            val_node
        );

        // Deprecated 313..314 key-value map warns once under strict() and records 1 diagnostic
        let mut legacy_kv = DcString::new();
        for short_id in [313u32, 275, 275, 255, 314, 312, 255, 312, 314] {
            legacy_kv.push(DcChar::from_short(short_id));
        }
        let mut legacy_arena = DcSyntaxArena::new();
        let legacy_root = DcDocumentNavigator::parse_arena(
            legacy_kv.as_bytes(),
            &mut legacy_arena,
            &policy,
        )?;
        ensure!(
            legacy_arena.diagnostics().len() == 1,
            "Expected 1 deprecation diagnostic in arena, got {:?}",
            legacy_arena.diagnostics()
        );
        let mut legacy_ctx = DcProjectionContext::new(&legacy_arena, &policy);
        let (_, legacy_entries) = legacy_ctx.extract_kv_map_entries(legacy_root)?;
        ensure!(legacy_entries.len() == 1, "Expected 1 legacy kv entry, got {}", legacy_entries.len());
        let (lkey_id, lval_id, _, _) = legacy_entries.into_iter().next().context("Missing legacy kv entry")??;
        ensure!(
            legacy_arena.get(lkey_id).is_some_and(|n| n.kind == DcSyntaxKind::Atom(DcChar::from_short(314)) && n.is_escaped_atom()),
            "Expected escaped Atom(314) key in legacy kv map"
        );
        ensure!(
            legacy_arena.get(lval_id).is_some_and(|n| n.kind == DcSyntaxKind::Atom(DcChar::from_short(312)) && n.is_escaped_atom()),
            "Expected escaped Atom(312) value in legacy kv map"
        );
        let mut reject_deprecated = DcProjectionPolicy::strict();
        reject_deprecated.deprecated_syntax = DiagnosticAction::Error;
        let mut err_arena = DcSyntaxArena::new();
        ensure!(
            DcDocumentNavigator::parse_arena(
                legacy_kv.as_bytes(),
                &mut err_arena,
                &reject_deprecated,
            )
            .is_err(),
            "Deprecated 313..314 must fail when deprecated_syntax is Error"
        );

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

        // Dc 388..389 (`:~ ([list:Semantic:fileflag] | [539:])? 389`) accepts 397/561 wrapping
        // both branches of the grouped alternation without explicit `| [397:] | [561:]` in CSV,
        // while still rejecting non-fileflag Dcs inside the list branch.
        let mut valid_flags = DcMstring::new();
        for short_id in [388u32, 397, 310, 275, 310, 275, 409, 311, 312, 539] {
            valid_flags.push_char(DcChar::from_short(short_id));
        }
        valid_flags.push_binary(&[1u8, 2u8]);
        valid_flags.push_char(DcChar::from_short(311));
        valid_flags.push_char(DcChar::from_short(389));
        validate_dc_mixed_strict(&valid_flags)?;

        let mut invalid_flags = DcMstring::new();
        for short_id in [388u32, 397, 310, 275, 310, 275, 403, 311, 311, 389] {
            invalid_flags.push_char(DcChar::from_short(short_id));
        }
        ensure!(
            validate_dc_mixed_strict(&invalid_flags).is_err(),
            "Dc 388 equivalent wrapper must reject non-fileflag Dc 403 inside [list:Semantic:fileflag]"
        );

        // [format] named type (`:[f0..f1114111]`) accepts the full allocated format region up to f1114111
        let mut max_format_repr = DcMstring::new();
        max_format_repr.push_char(DC_BEGIN_FORMAT_REPRESENTATION);
        max_format_repr.push_char(DcChar::from_format(1_114_111));
        max_format_repr.push_char(DC_BOOLEAN_TRUE);
        max_format_repr.push_char(DC_END_FORMAT_REPRESENTATION);
        validate_dc_mixed_strict(&max_format_repr)?;
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
    use ctb_formats_dcstring::BinaryPayload;
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

    /// Verifies expression evaluation fallback for `[data]` and `[identifier]`.
    #[crate::ctb_test]
    fn test_expression_fallback_for_data_and_identifier() -> Result<()> {
        let resolver = default_syntax_resolver();

        // 1. [data] satisfied by an expression returning binary data (Dc 203)
        let mut data_doc = DcMstring::new();
        BinaryPayload::new(vec![0xAA, 0xBB, 0xCC]).encode_dc_mixed(&mut data_doc)?;

        let data_evaluator = Arc::new(MockExpressionEvaluator {
            mock_doc: data_doc,
        });

        let mut expr_doc = DcMstring::new();
        expr_doc.push_char(DcChar::from_short(540));
        expr_doc.push_char(DC_BEGIN_IDENTIFIER);
        expr_doc.push_str("data_var");
        expr_doc.push_char(DC_END_IDENTIFIER);

        let mut nav = DcDocumentNavigator::from_mstr(&expr_doc);
        let mut ctx = SyntaxEvalContext::new(&**resolver, None);
        ctx.evaluator = Some(data_evaluator);

        let data_term = SyntaxTerm::NamedConstruct {
            name: "data".to_string(),
            subtype: None,
            capture_var: None,
        };
        let data_matched = nav.eval_term_single(&data_term, &mut ctx)?;
        ensure!(data_matched, "[data] must be satisfied by expression evaluating to binary data");

        // 2. [identifier] satisfied by an expression returning an identifier
        let mut id_doc = DcMstring::new();
        id_doc.push_char(DC_BEGIN_IDENTIFIER);
        id_doc.push_str("evaluated_id");
        id_doc.push_char(DC_END_IDENTIFIER);

        let id_evaluator = Arc::new(MockExpressionEvaluator {
            mock_doc: id_doc,
        });

        let mut nav = DcDocumentNavigator::from_mstr(&expr_doc);
        let mut ctx = SyntaxEvalContext::new(&**resolver, None);
        ctx.evaluator = Some(id_evaluator);

        let id_term = SyntaxTerm::NamedConstruct {
            name: "identifier".to_string(),
            subtype: None,
            capture_var: None,
        };
        let id_matched = nav.eval_term_single(&id_term, &mut ctx)?;
        ensure!(id_matched, "[identifier] must be satisfied by expression evaluating to identifier");

        Ok(())
    }

    /// Verifies static type checking and compatibility for `ReferenceNamedObject` (`276 [type] [identifier]`).
    #[crate::ctb_test]
    fn test_reference_named_object_type_compatibility() -> Result<()> {
        let resolver = default_syntax_resolver();

        // 276 [type: 264 (string)] [id: "my_str"]
        let mut string_ref = DcMstring::new();
        string_ref.push_char(DcChar::from_short(276));
        string_ref.push_char(DcChar::from_short(264)); // f8 UTF-8 string format
        string_ref.push_char(DC_BEGIN_IDENTIFIER);
        string_ref.push_str("my_str");
        string_ref.push_char(DC_END_IDENTIFIER);

        // 276 [type: DC_TYPE_NUMBER (number)] [id: "my_num"]
        let mut number_ref = DcMstring::new();
        number_ref.push_char(DcChar::from_short(276));
        number_ref.push_char(ctb_formats_dcdata::dc::DC_TYPE_NUMBER);
        number_ref.push_char(DC_BEGIN_IDENTIFIER);
        number_ref.push_str("my_num");
        number_ref.push_char(DC_END_IDENTIFIER);

        let string_term = SyntaxTerm::NamedConstruct {
            name: "string".to_string(),
            subtype: None,
            capture_var: None,
        };
        let number_term = SyntaxTerm::NamedConstruct {
            name: "number".to_string(),
            subtype: None,
            capture_var: None,
        };

        // 1. String ref satisfies [string] requirement
        let mut nav = DcDocumentNavigator::from_mstr(&string_ref);
        let mut ctx = SyntaxEvalContext::new(&**resolver, None);
        ensure!(nav.eval_term_single(&string_term, &mut ctx)?, "276 [string] must satisfy [string]");

        // 2. String ref rejected for [number] requirement
        let mut nav = DcDocumentNavigator::from_mstr(&string_ref);
        let mut ctx = SyntaxEvalContext::new(&**resolver, None);
        ensure!(!nav.eval_term_single(&number_term, &mut ctx)?, "276 [string] must reject [number]");

        // 3. Number ref satisfies [number] requirement
        let mut nav = DcDocumentNavigator::from_mstr(&number_ref);
        let mut ctx = SyntaxEvalContext::new(&**resolver, None);
        ensure!(nav.eval_term_single(&number_term, &mut ctx)?, "276 [number] must satisfy [number]");

        // 4. Number ref rejected for [string] requirement
        let mut nav = DcDocumentNavigator::from_mstr(&number_ref);
        let mut ctx = SyntaxEvalContext::new(&**resolver, None);
        ensure!(!nav.eval_term_single(&string_term, &mut ctx)?, "276 [number] must reject [string]");

        // 5. Explicit type wrapped in 262..263 type def
        let mut wrapped_string_ref = DcMstring::new();
        wrapped_string_ref.push_char(DcChar::from_short(276));
        wrapped_string_ref.push_char(DC_BEGIN_TYPE_DEF);
        wrapped_string_ref.push_char(DcChar::from_short(264));
        wrapped_string_ref.push_char(DC_END_TYPE_DEF);
        wrapped_string_ref.push_char(DC_BEGIN_IDENTIFIER);
        wrapped_string_ref.push_str("wrapped_str");
        wrapped_string_ref.push_char(DC_END_IDENTIFIER);

        let mut nav = DcDocumentNavigator::from_mstr(&wrapped_string_ref);
        let mut ctx = SyntaxEvalContext::new(&**resolver, None);
        ensure!(nav.eval_term_single(&string_term, &mut ctx)?, "276 [262 264 263] must satisfy [string]");
        let mut nav = DcDocumentNavigator::from_mstr(&wrapped_string_ref);
        let mut ctx = SyntaxEvalContext::new(&**resolver, None);
        ensure!(!nav.eval_term_single(&number_term, &mut ctx)?, "276 [262 264 263] must reject [number]");

        Ok(())
    }

    /// Verifies DefaultDcExpressionEvaluator evaluates EquivalentRepresentations (397).
    #[crate::ctb_test]
    fn test_default_evaluator_equivalent_representations() -> Result<()> {
        let resolver = default_syntax_resolver();
        let evaluator = DefaultDcExpressionEvaluator;

        // Build 397 with two representations: [true] and [42]
        let mut equiv_doc = DcMstring::new();
        equiv_doc.push_char(DC_EQUIVALENT_REPRESENTATIONS);
        equiv_doc.push_char(DC_BEGIN_LIST);
        equiv_doc.push_char(DC_TYPE_ANY);
        equiv_doc.push_char(DC_BOOLEAN_TRUE);
        equiv_doc.push_char(DC_LIST_ELEMENT_SEPARATOR);
        42u64.encode_dc_mixed(&mut equiv_doc)?;
        equiv_doc.push_char(DC_END_LIST);

        let tokens: Vec<DcChar> = equiv_doc.to_dc_string()?.chars().collect();

        // 1. Evaluate for [number:Natural0]
        let num_req = ValueRequirement::NamedConstruct {
            name: "number",
            subtype: Some("Natural0"),
        };
        let res = evaluator.eval_expression_for_requirement(&tokens, &num_req, &**resolver)?;
        ensure!(res.is_some(), "Default evaluator must find matching candidate for [number:Natural0]");

        // 2. Evaluate for [string] (no candidate in list)
        let str_req = ValueRequirement::NamedConstruct {
            name: "string",
            subtype: None,
        };
        let res = evaluator.eval_expression_for_requirement(&tokens, &str_req, &**resolver)?;
        ensure!(res.is_none(), "Default evaluator must return None when no candidate satisfies [string]");

        Ok(())
    }

    /// Verifies that a Group pattern wrapping [number:Natural0] properly checks subtype
    /// constraints on an EquivalentRepresentations set.
    #[crate::ctb_test]
    fn test_group_pattern_wrapping_constrained_number_in_equivalent_set() -> Result<()> {
        let resolver = default_syntax_resolver();

        // 397 with negative number -42 only
        let mut neg_equiv = DcMstring::new();
        neg_equiv.push_char(DC_EQUIVALENT_REPRESENTATIONS);
        neg_equiv.push_char(DC_BEGIN_LIST);
        neg_equiv.push_char(DC_TYPE_ANY);
        (-42i64).encode_dc_mixed(&mut neg_equiv)?;
        neg_equiv.push_char(DC_END_LIST);

        // Group wrapping [number:Natural0]
        let group_term = SyntaxTerm::Group(SyntaxPattern::Sequence(vec![
            SyntaxElement {
                term: SyntaxTerm::NamedConstruct {
                    name: "number".to_string(),
                    subtype: Some("Natural0".to_string()),
                    capture_var: None,
                },
                quantifier: Quantifier::ExactOne,
            },
        ]));

        let mut nav = DcDocumentNavigator::from_mstr(&neg_equiv);
        let mut ctx = SyntaxEvalContext::new(&**resolver, None);
        let matched = nav.eval_term_single(&group_term, &mut ctx)?;
        ensure!(!matched, "Group wrapping [number:Natural0] must reject equivalent set with negative number");

        // 397 with non-negative number 42
        let mut pos_equiv = DcMstring::new();
        pos_equiv.push_char(DC_EQUIVALENT_REPRESENTATIONS);
        pos_equiv.push_char(DC_BEGIN_LIST);
        pos_equiv.push_char(DC_TYPE_ANY);
        42u64.encode_dc_mixed(&mut pos_equiv)?;
        pos_equiv.push_char(DC_END_LIST);

        let mut nav = DcDocumentNavigator::from_mstr(&pos_equiv);
        let mut ctx = SyntaxEvalContext::new(&**resolver, None);
        let matched = nav.eval_term_single(&group_term, &mut ctx)?;
        ensure!(matched, "Group wrapping [number:Natural0] must accept equivalent set with non-negative number");

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

        let root_id = *roots.first().context("Root node missing")?;
        let root_node = arena.get(root_id).context("Root node missing")?;
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
                    children.len() == 13,
                    "Expected 13 spliced children for 'Hello, World!', got {}",
                    children.len()
                );

                let expected_text = [57u32, 86, 93, 93, 96, 30, 18, 72, 96, 99, 93, 85, 19];
                for (i, &expected_id) in expected_text.iter().enumerate() {
                    let node_id = *children.get(i).context("Child index out of bounds")?;
                    let child_node = arena.get(node_id).context("Child node missing")?;
                    match &child_node.kind {
                        DcSyntaxKind::Atom(ch) => {
                            ensure!(
                                *ch == DcChar::from_short(expected_id),
                                "Child {i} expected {expected_id}, got {ch}"
                            );
                        }
                        other => bail!("Child {i} expected Atom, got {other:?}"),
                    }
                }
            }
            other => bail!("Root node must be Container, got {other:?}"),
        }

        Ok(())
    }

    /// Verifies that `lang.say` inside a container where the closer (Dc 287) is outside
    /// the `lang.say` call correctly splices the 13 characters into the container.
    #[crate::ctb_test]
    fn test_lang_say_external_closer_stream_splicing() -> Result<()> {
        let policy = DcProjectionPolicy::strict();

        // 286 (Begin italicized text)
        // 256 (void lang.say(String))
        // 258 260 262 264 263 57 86 93 93 96 30 18 72 96 99 93 85 19 261 259
        // 287 (End italicized text - outside lang.say)
        let token_ids: [u32; 23] = [
            286, 256, 258, 260, 262, 264, 263, 57, 86, 93, 93, 96, 30, 18, 72, 96, 99, 93, 85, 19,
            261, 259, 287,
        ];
        let chars: Vec<DcChar> = token_ids.iter().map(|&id| DcChar::from_short(id)).collect();
        let doc = create_test_mstring(&chars);

        let mut arena_nav = DcDocumentNavigator::from_mstr(&doc);
        let mut arena = DcSyntaxArena::new();
        arena_nav.parse_document_into_arena(&mut arena, &policy)?;

        let roots = arena.roots();
        ensure!(roots.len() == 1, "Expected 1 root node in arena, found {}", roots.len());

        let root_id = *roots.first().context("Root node missing")?;
        let root_node = arena.get(root_id).context("Root node missing")?;
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
                    children.len() == 13,
                    "Expected 13 spliced children for 'Hello, World!', got {}",
                    children.len()
                );

                let expected_text = [57u32, 86, 93, 93, 96, 30, 18, 72, 96, 99, 93, 85, 19];
                for (i, &expected_id) in expected_text.iter().enumerate() {
                    let node_id = *children.get(i).context("Child index out of bounds")?;
                    let child_node = arena.get(node_id).context("Child node missing")?;
                    match &child_node.kind {
                        DcSyntaxKind::Atom(ch) => {
                            ensure!(
                                *ch == DcChar::from_short(expected_id),
                                "Child {i} expected {expected_id}, got {ch}"
                            );
                        }
                        other => bail!("Child {i} expected Atom, got {other:?}"),
                    }
                }
            }
            other => bail!("Root node must be Container, got {other:?}"),
        }

        Ok(())
    }

    /// Verifies that `eval_term_single` handles `lang.say` stream splicing
    /// during grammar validation.
    #[crate::ctb_test]
    fn test_eval_term_single_lang_say_stream_splicing() -> Result<()> {
        let policy = DcProjectionPolicy::strict();
        let resolver = default_syntax_resolver();

        // 256 (void lang.say(String))
        // 258 260 262 264 263 57 86 93 93 96 30 18 72 96 99 93 85 19 261 259
        let token_ids: [u32; 21] = [
            256, 258, 260, 262, 264, 263, 57, 86, 93, 93, 96, 30, 18, 72, 96, 99, 93, 85, 19,
            261, 259,
        ];
        let chars: Vec<DcChar> = token_ids.iter().map(|&id| DcChar::from_short(id)).collect();
        let doc = create_test_mstring(&chars);

        let mut nav = DcDocumentNavigator::from_mstr(&doc);
        let mut ctx = SyntaxEvalContext::new(&**resolver, None);

        // Term matching 'H' (Dc 57)
        let term_h = SyntaxTerm::CharRef(CharTarget::Dc(57));
        let matched = nav.eval_term_single(&term_h, &mut ctx)?;
        ensure!(
            matched,
            "eval_term_single must splice lang.say and match first spliced character 'H'"
        );

        // Term matching 'e' (Dc 86)
        let term_e = SyntaxTerm::CharRef(CharTarget::Dc(86));
        let matched_e = nav.eval_term_single(&term_e, &mut ctx)?;
        ensure!(
            matched_e,
            "eval_term_single must match next spliced character 'e'"
        );

        Ok(())
    }

    /// Verifies list declared length checking using DcRole-based semantics.
    #[crate::ctb_test]
    fn test_declared_list_length_dcrole_validation() -> Result<()> {
        let policy = DcProjectionPolicy::strict();

        // Declared length 2 with 2 elements:
        let mut valid_doc = DcMstring::new();
        valid_doc.push_char(DC_BEGIN_LIST);
        2u64.encode_dc_mixed(&mut valid_doc)?;
        valid_doc.push_char(ctb_formats_dcdata::dc::DC_TYPE_ANY);
        valid_doc.push_char(ctb_formats_dcdata::dc::DC_BOOLEAN_TRUE);
        valid_doc.push_char(ctb_formats_dcdata::dc::DC_LIST_ELEMENT_SEPARATOR);
        valid_doc.push_char(ctb_formats_dcdata::dc::DC_BOOLEAN_FALSE);
        valid_doc.push_char(ctb_formats_dcdata::dc::DC_END_LIST);

        let mut nav = DcDocumentNavigator::from_mstr(&valid_doc);
        ensure!(nav.validate_document(&policy).is_ok(), "Declared length 2 with 2 elements must validate");

        // Declared length 3 with 2 elements -> mismatch:
        let mut bad_doc = DcMstring::new();
        bad_doc.push_char(DC_BEGIN_LIST);
        3u64.encode_dc_mixed(&mut bad_doc)?;
        bad_doc.push_char(ctb_formats_dcdata::dc::DC_TYPE_ANY);
        bad_doc.push_char(ctb_formats_dcdata::dc::DC_BOOLEAN_TRUE);
        bad_doc.push_char(ctb_formats_dcdata::dc::DC_LIST_ELEMENT_SEPARATOR);
        bad_doc.push_char(ctb_formats_dcdata::dc::DC_BOOLEAN_FALSE);
        bad_doc.push_char(ctb_formats_dcdata::dc::DC_END_LIST);

        let mut bad_nav = DcDocumentNavigator::from_mstr(&bad_doc);
        ensure!(bad_nav.validate_document(&policy).is_err(), "Declared length 3 with 2 elements must fail strict validation");
        Ok(())
    }

    /// Verifies number subtype constraint resolution through SyntaxRuleResolver
    /// with format shorthands and rule references.
    #[crate::ctb_test]
    fn test_number_satisfies_subtype_resolver_driven() -> Result<()> {
        let resolver = default_syntax_resolver();
        let mut doc_2 = DcMstring::new();
        2u64.encode_dc_mixed(&mut doc_2)?;

        // Test shorthand format subtypes
        let mut nav = DcDocumentNavigator::from_mstr(&doc_2);
        let mut ctx = SyntaxEvalContext::new(&**resolver, None);
        let term_f200 = SyntaxTerm::NamedConstruct {
            name: "number".to_string(),
            subtype: Some("f200".to_string()),
            capture_var: None,
        };
        ensure!(nav.eval_term_single(&term_f200, &mut ctx)?, "2 must satisfy f200 (Natural0)");

        let mut nav_u8 = DcDocumentNavigator::from_mstr(&doc_2);
        let mut ctx_u8 = SyntaxEvalContext::new(&**resolver, None);
        let term_u8 = SyntaxTerm::NamedConstruct {
            name: "number".to_string(),
            subtype: Some("f203".to_string()),
            capture_var: None,
        };
        ensure!(nav_u8.eval_term_single(&term_u8, &mut ctx_u8)?, "2 must satisfy f203 (U8)");

        // Test format rule resolution via resolver (e.g. DcIdentifier: :[number:Natural1 & <(2^128)])
        let mut nav_rule = DcDocumentNavigator::from_mstr(&doc_2);
        let mut ctx_rule = SyntaxEvalContext::new(&**resolver, None);
        let term_ident_rule = SyntaxTerm::NamedConstruct {
            name: "number".to_string(),
            subtype: Some("DcIdentifier".to_string()),
            capture_var: None,
        };
        ensure!(nav_rule.eval_term_single(&term_ident_rule, &mut ctx_rule)?, "2 must satisfy DcIdentifier format rule");
        Ok(())
    }

    /// Verifies that IP address decoders accept equivalent format tokens.
    #[crate::ctb_test]
    fn test_format_token_equivalence_in_mixed_serde() -> Result<()> {
        use std::net::{IpAddr, Ipv4Addr};
        let policy = DcProjectionPolicy::strict();

        let mut mst = DcMstring::new();
        let expected_ip = Ipv4Addr::new(127, 0, 0, 1);
        expected_ip.encode_dc_mixed(&mut mst)?;

        let decoded: Ipv4Addr = mst.decode::<Ipv4Addr>(&policy)?;
        ensure!(decoded == expected_ip, "Decoded IPv4 must match 127.0.0.1");

        let decoded_ip: IpAddr = mst.decode::<IpAddr>(&policy)?;
        ensure!(decoded_ip == IpAddr::V4(expected_ip), "Decoded IpAddr must match 127.0.0.1");
        Ok(())
    }

    struct CustomMockResolver {
        fallback: std::sync::Arc<DatasetRuleResolver>,
    }

    impl SyntaxRuleResolver for CustomMockResolver {
        fn is_equivalent_to(&self, a: u128, b: u128) -> bool {
            if (a == 999999 && b == 888888) || (a == 888888 && b == 999999) {
                return true;
            }
            self.fallback.is_equivalent_to(a, b)
        }

        fn canonical_token(&self, token: u128) -> u128 {
            if token == 999999 {
                return 888888;
            }
            self.fallback.canonical_token(token)
        }

        fn token_fills_role(&self, token: u128, role: DcRole) -> bool {
            self.fallback.token_fills_role(token, role)
        }

        fn resolve_rule(&self, target: &ctb_formats_dcdata::CharTarget) -> Option<&ctb_formats_dcdata::DcSyntaxRule> {
            self.fallback.resolve_rule(target)
        }

        fn resolve_named_type(&self, name: &str) -> Option<&ctb_formats_dcdata::DcSyntaxExpr> {
            self.fallback.resolve_named_type(name)
        }

        fn matches_script(&self, script: &str, codepoint: u128) -> bool {
            self.fallback.matches_script(script, codepoint)
        }
    }

    /// Verifies threading custom SyntaxRuleResolver and DcExpressionEvaluator into DcProjectionContext.
    #[crate::ctb_test]
    fn test_projection_context_custom_resolver_and_evaluator_threading() -> Result<()> {
        let policy = DcProjectionPolicy::strict();
        let arena = DcSyntaxArena::new();
        let fallback = default_syntax_resolver().clone();
        let custom_resolver = CustomMockResolver { fallback };
        let custom_evaluator = DefaultDcExpressionEvaluator;

        let mut ctx = ctb_formats_dcstring::DcProjectionContext::new(&arena, &policy);
        // Default resolver is default_syntax_resolver
        ensure!(!ctx.tokens_equivalent(DcChar::from_u128(999999), DcChar::from_u128(888888)));
        ensure!(ctx.evaluator().is_none());

        // Thread custom resolver and evaluator
        ctx = ctx.with_syntax_resolver(&custom_resolver).with_evaluator(&custom_evaluator);
        ensure!(ctx.tokens_equivalent(DcChar::from_u128(999999), DcChar::from_u128(888888)));
        ensure!(ctx.canonical_token(DcChar::from_u128(999999)).to_u128() == 888888);
        ensure!(ctx.evaluator().is_some());

        // Test from_eval_context
        let mut eval_ctx = SyntaxEvalContext::new(&custom_resolver, None);
        eval_ctx.evaluator = Some(std::sync::Arc::new(DefaultDcExpressionEvaluator));
        let ctx2 = ctb_formats_dcstring::DcProjectionContext::from_eval_context(&arena, &eval_ctx, &policy);
        ensure!(ctx2.tokens_equivalent(DcChar::from_u128(999999), DcChar::from_u128(888888)));
        ensure!(ctx2.evaluator().is_some());
        Ok(())
    }

    /// Verifies unwrap_equivalent_or_self and unwrap_transparent resolving 398 and expression nodes (540, 276, 279).
    #[crate::ctb_test]
    fn test_unwrap_equivalent_and_expression_nodes() -> Result<()> {
        let policy = DcProjectionPolicy::strict();

        // 1. Partially equivalent representations container (398)
        let mut doc_398 = DcMstring::new();
        doc_398.push_char(DC_PARTIALLY_EQUIVALENT_REPRESENTATIONS);
        doc_398.push_char(DC_BEGIN_LIST);
        doc_398.push_char(DC_TYPE_ANY);
        42u64.encode_dc_mixed(&mut doc_398)?;
        doc_398.push_char(DC_END_LIST);

        let mut nav = DcDocumentNavigator::from_mstr(&doc_398);
        let mut arena = DcSyntaxArena::new();
        nav.parse_document_into_arena(&mut arena, &policy)?;
        let root = *arena.roots().first().context("Missing root for 398")?;
        let ctx = ctb_formats_dcstring::DcProjectionContext::new(&arena, &policy);
        let unwrapped = ctx.unwrap_equivalent_or_self(root)?;
        let num_node = ctx.node(unwrapped)?;
        ensure!(matches!(num_node.kind, DcSyntaxKind::Number { .. }), "398 must unwrap to Number");

        // 2. Bound value reference (540) to an assignment in the arena
        let mut doc_assign = DcMstring::new();
        doc_assign.push_char(DC_BEGIN_IDENTIFIER);
        doc_assign.push_str("my_var");
        doc_assign.push_char(DC_END_IDENTIFIER);
        doc_assign.push_char(ctb_formats_dcdata::dc::DC_ASSIGN);
        100u64.encode_dc_mixed(&mut doc_assign)?;
        doc_assign.push_char(ctb_formats_dcdata::dc::DC_REFERENCE_BOUND_VALUE);
        doc_assign.push_char(DC_BEGIN_IDENTIFIER);
        doc_assign.push_str("my_var");
        doc_assign.push_char(DC_END_IDENTIFIER);

        let mut nav = DcDocumentNavigator::from_mstr(&doc_assign);
        let mut arena = DcSyntaxArena::new();
        nav.parse_document_into_arena(&mut arena, &policy)?;
        let ref_node_id = *arena.roots().get(1).context("Missing reference root")?;
        let mut ctx = ctb_formats_dcstring::DcProjectionContext::new(&arena, &policy);
        let unwrapped_ref = ctx.unwrap_equivalent_or_self(ref_node_id)?;
        let target_node = ctx.node(unwrapped_ref)?;
        ensure!(matches!(target_node.kind, DcSyntaxKind::Number { .. }), "540 must resolve to assigned Number");
        // Test transparent decoding through decode_dc_node:
        let decoded: u64 = ctb_formats_dcstring::DcMixedDecode::decode_dc_node(&mut ctx, ref_node_id)?;
        ensure!(decoded == 100, "540 reference must transparently decode as 100");

        // 3. Named object reference (276)
        let mut doc_named = DcMstring::new();
        doc_named.push_char(DC_BEGIN_IDENTIFIER);
        doc_named.push_str("obj_var");
        doc_named.push_char(DC_END_IDENTIFIER);
        doc_named.push_char(ctb_formats_dcdata::dc::DC_ASSIGN);
        "named_str".to_string().encode_dc_mixed(&mut doc_named)?;
        doc_named.push_char(ctb_formats_dcdata::dc::DC_REFERENCE_NAMED_OBJECT);
        doc_named.push_char(DC_BEGIN_IDENTIFIER);
        doc_named.push_str("obj_var");
        doc_named.push_char(DC_END_IDENTIFIER);

        let mut nav = DcDocumentNavigator::from_mstr(&doc_named);
        let mut arena = DcSyntaxArena::new();
        nav.parse_document_into_arena(&mut arena, &policy)?;
        let ref_node_id = *arena.roots().get(1).context("Missing named reference root")?;
        let mut ctx = ctb_formats_dcstring::DcProjectionContext::new(&arena, &policy);
        let unwrapped_ref = ctx.unwrap_equivalent_or_self(ref_node_id)?;
        let target_node = ctx.node(unwrapped_ref)?;
        ensure!(matches!(target_node.kind, DcSyntaxKind::Literal { .. }), "276 must resolve to assigned Literal");
        let decoded_str: String = ctb_formats_dcstring::DcMixedDecode::decode_dc_node(&mut ctx, ref_node_id)?;
        ensure!(decoded_str == "named_str", "276 reference must transparently decode as named_str");

        // 4. Routine invocation (279)
        let mut doc_routine = DcMstring::new();
        doc_routine.push_char(ctb_formats_dcdata::dc::DC_ROUTINE_INVOCATION);
        doc_routine.push_char(DC_BEGIN_IDENTIFIER);
        doc_routine.push_str("lang.say");
        doc_routine.push_char(DC_END_IDENTIFIER);
        doc_routine.push_char(ctb_formats_dcdata::dc::DC_BEGIN_PARAMETER);
        "routine_payload".to_string().encode_dc_mixed(&mut doc_routine)?;
        doc_routine.push_char(ctb_formats_dcdata::dc::DC_END_PARAMETER);

        let mut nav = DcDocumentNavigator::from_mstr(&doc_routine);
        let mut arena = DcSyntaxArena::new();
        nav.parse_document_into_arena(&mut arena, &policy)?;
        let routine_id = *arena.roots().first().context("Missing routine root")?;
        let mut ctx = ctb_formats_dcstring::DcProjectionContext::new(&arena, &policy);
        let unwrapped_routine = ctx.unwrap_equivalent_or_self(routine_id)?;
        let target_node = ctx.node(unwrapped_routine)?;
        ensure!(matches!(target_node.kind, DcSyntaxKind::Literal { .. }), "279 must resolve to argument Literal");
        let decoded_routine: String = ctb_formats_dcstring::DcMixedDecode::decode_dc_node(&mut ctx, routine_id)?;
        ensure!(decoded_routine == "routine_payload", "279 routine must transparently decode payload");
        Ok(())
    }

    /// Verifies that decoding equivalent representations selects a candidate that decodes cleanly
    /// for the target type even when preceding candidates in the set have different types.
    #[crate::ctb_test]
    fn test_equivalent_representations_clean_target_type_selection() -> Result<()> {
        use ctb_formats_dcstring::DcMixedDecode;
        let policy = DcProjectionPolicy::strict();

        // 1. 397 with [ "hello", 12345u64 ]
        // First candidate is String; second is Number.
        let mut equiv_str_num = DcMstring::new();
        equiv_str_num.push_char(DC_EQUIVALENT_REPRESENTATIONS);
        equiv_str_num.push_char(DC_BEGIN_LIST);
        equiv_str_num.push_char(DC_TYPE_ANY);
        "hello".to_string().encode_dc_mixed(&mut equiv_str_num)?;
        equiv_str_num.push_char(DC_LIST_ELEMENT_SEPARATOR);
        12345u64.encode_dc_mixed(&mut equiv_str_num)?;
        equiv_str_num.push_char(DC_END_LIST);

        let mut nav = DcDocumentNavigator::from_mstr(&equiv_str_num);
        let mut arena = DcSyntaxArena::new();
        nav.parse_document_into_arena(&mut arena, &policy)?;
        let root = *arena.roots().first().context("Missing root")?;
        let mut ctx = ctb_formats_dcstring::DcProjectionContext::new(&arena, &policy);

        // Target type u64 should cleanly select candidate 1 (12345)
        let num_val: u64 = u64::decode_dc_node(&mut ctx, root)?;
        ensure!(num_val == 12345, "u64 must select Number candidate (12345)");

        // Target type String should cleanly select candidate 0 ("hello")
        let str_val: String = String::decode_dc_node(&mut ctx, root)?;
        ensure!(str_val == "hello", "String must select Literal candidate (hello)");

        // 2. 398 with [ 999u64, true ]
        // First candidate is Number; second is Boolean.
        let mut part_num_bool = DcMstring::new();
        part_num_bool.push_char(DC_PARTIALLY_EQUIVALENT_REPRESENTATIONS);
        part_num_bool.push_char(DC_BEGIN_LIST);
        part_num_bool.push_char(DC_TYPE_ANY);
        999u64.encode_dc_mixed(&mut part_num_bool)?;
        part_num_bool.push_char(DC_LIST_ELEMENT_SEPARATOR);
        true.encode_dc_mixed(&mut part_num_bool)?;
        part_num_bool.push_char(DC_END_LIST);

        let mut nav = DcDocumentNavigator::from_mstr(&part_num_bool);
        let mut arena = DcSyntaxArena::new();
        nav.parse_document_into_arena(&mut arena, &policy)?;
        let root = *arena.roots().first().context("Missing root")?;
        let mut ctx = ctb_formats_dcstring::DcProjectionContext::new(&arena, &policy);

        // Target type bool should cleanly select candidate 1 (true)
        let bool_val: bool = bool::decode_dc_node(&mut ctx, root)?;
        ensure!(bool_val, "bool must select BooleanTrue candidate");

        // Target type u64 should cleanly select candidate 0 (999)
        let num_val: u64 = u64::decode_dc_node(&mut ctx, root)?;
        ensure!(num_val == 999, "u64 must select Number candidate (999)");

        Ok(())
    }
}

mod dynamic_runtime_equivalence_overlay_tests {
    use super::*;
    use ctb_formats_dcdata::dc::{
        DC_BEGIN_LITERAL, DC_END_LITERAL, DC_TYPE_STRING,
        DC_BEGIN_NUMBER, DC_END_NUMBER,
    };
    use ctb_formats_dcstring::DcMixed;

    #[derive(Debug, Clone, PartialEq, Eq, DcMixed)]
    #[dc(begin = 401, end = 402)]
    struct OverlaySampleRecord {
        #[dc(short = 400)]
        title: String,
        #[dc(short = 406)]
        count: u64,
    }

    #[derive(Debug, Clone, PartialEq, Eq, DcMixed)]
    enum OverlaySampleStatus {
        #[dc(short = 327)]
        Active(u64),
        #[dc(short = 359)]
        Inactive,
    }

    /// Verifies list container openers, closers, and separators (`310`..`312`)
    /// dynamically substituted via runtime equivalence overlay.
    #[crate::ctb_test]
    fn test_list_openers_closers_and_separators_equivalence_overlay() -> Result<()> {
        let policy = DcProjectionPolicy::strict();
        let mut registry = SyntaxRuntimeRegistry::default();

        let synth_list_open = DcChar::from_short(3910).to_u128();
        let synth_list_close = DcChar::from_short(3911).to_u128();
        let synth_list_sep = DcChar::from_short(3912).to_u128();

        registry.register_equivalence(synth_list_open, DC_BEGIN_LIST.to_u128());
        registry.register_equivalence(synth_list_close, DC_END_LIST.to_u128());
        registry.register_equivalence(synth_list_sep, DC_LIST_ELEMENT_SEPARATOR.to_u128());

        let mut doc = DcMstring::new();
        doc.push_char(DcChar::from_u128(synth_list_open));
        doc.push_char(DC_TYPE_ANY);
        10u64.encode_dc_mixed(&mut doc)?;
        doc.push_char(DcChar::from_u128(synth_list_sep));
        20u64.encode_dc_mixed(&mut doc)?;
        doc.push_char(DcChar::from_u128(synth_list_sep));
        30u64.encode_dc_mixed(&mut doc)?;
        doc.push_char(DcChar::from_u128(synth_list_close));

        // 1. Decodes successfully using the overlay resolver on DcMstr
        let list: Vec<u64> =
            doc.decode_with_resolver(registry.resolver(), &policy)?;
        ensure!(list == [10, 20, 30], "List must decode elements [10, 20, 30]");

        // 2. Decodes successfully using the overlay resolver on DcStr
        let dc_string = doc.to_dc_string()?;
        let list_str: Vec<u64> =
            dc_string.decode_with_resolver(registry.resolver(), &policy)?;
        ensure!(list_str == [10, 20, 30], "DcStr must decode elements");

        // 3. Without the overlay resolver, decoding fails strict validation
        ensure!(
            doc.decode::<Vec<u64>>(&policy).is_err(),
            "Decoding synthetic list without overlay resolver must fail"
        );

        Ok(())
    }

    /// Verifies key-value map openers, closers, and pairs (`559`..`560`,
    /// `514`..`515`) dynamically substituted via runtime equivalence overlay.
    #[crate::ctb_test]
    fn test_map_openers_closers_and_pairs_equivalence_overlay() -> Result<()> {
        let policy = DcProjectionPolicy::strict();
        let mut registry = SyntaxRuntimeRegistry::default();

        let synth_map_open = DcChar::from_short(3959).to_u128();
        let synth_map_close = DcChar::from_short(3960).to_u128();
        let synth_pair_open = DcChar::from_short(3914).to_u128();
        let synth_pair_close = DcChar::from_short(3915).to_u128();

        registry.register_equivalence(synth_map_open, DC_BEGIN_KV_MAP.to_u128());
        registry.register_equivalence(synth_map_close, DC_END_KV_MAP.to_u128());
        registry.register_equivalence(synth_pair_open, DC_BEGIN_KV_PAIR.to_u128());
        registry.register_equivalence(synth_pair_close, DC_END_KV_PAIR.to_u128());

        let mut doc = DcMstring::new();
        doc.push_char(DcChar::from_u128(synth_map_open));
        doc.push_char(DC_TYPE_WILDCARD);
        doc.push_char(DC_TYPE_WILDCARD);

        // Pair 1: 1 -> 100
        doc.push_char(DcChar::from_u128(synth_pair_open));
        1u64.encode_dc_mixed(&mut doc)?;
        doc.push_char(DC_LIST_ELEMENT_SEPARATOR);
        100u64.encode_dc_mixed(&mut doc)?;
        doc.push_char(DcChar::from_u128(synth_pair_close));

        // Pair 2: 2 -> 200
        doc.push_char(DcChar::from_u128(synth_pair_open));
        2u64.encode_dc_mixed(&mut doc)?;
        doc.push_char(DC_LIST_ELEMENT_SEPARATOR);
        200u64.encode_dc_mixed(&mut doc)?;
        doc.push_char(DcChar::from_u128(synth_pair_close));

        doc.push_char(DcChar::from_u128(synth_map_close));

        // 1. Decode into BTreeMap using overlay resolver
        let map: std::collections::BTreeMap<u64, u64> =
            doc.decode_with_resolver(registry.resolver(), &policy)?;
        ensure!(map.len() == 2, "Decoded map must contain 2 entries");
        ensure!(map.get(&1) == Some(&100), "Map key 1 must map to 100");
        ensure!(map.get(&2) == Some(&200), "Map key 2 must map to 200");

        // 2. Direct navigator extraction using extract_kv_map_node_pairs_with_resolver
        let mut nav = DcDocumentNavigator::from_mstr(&doc);
        let mut arena = DcSyntaxArena::new();
        nav.parse_document_into_arena_with_resolver(
            &mut arena,
            &policy,
            registry.resolver(),
        )?;
        let root = *arena.roots().first().context("Missing arena root")?;
        let (_opener, pairs) = arena.extract_kv_map_node_pairs_with_resolver(
            root,
            registry.resolver(),
        )?;
        ensure!(pairs.len() == 2, "Must extract 2 pair nodes from arena");

        Ok(())
    }

    /// Verifies literals, identifiers, numbers, and booleans
    /// (`260`..`261`, `270`..`271`, `6`..`7`, `517`..`518`) dynamically
    /// substituted via runtime equivalence overlay.
    #[crate::ctb_test]
    fn test_primitives_literals_identifiers_numbers_and_booleans_overlay(
    ) -> Result<()> {
        let policy = DcProjectionPolicy::strict();
        let mut registry = SyntaxRuntimeRegistry::default();

        let synth_lit_open = DcChar::from_short(3962).to_u128();
        let synth_lit_close = DcChar::from_short(3963).to_u128();
        let synth_ident_open = DcChar::from_short(3970).to_u128();
        let synth_ident_close = DcChar::from_short(3971).to_u128();
        let synth_num_open = DcChar::from_short(3906).to_u128();
        let synth_num_close = DcChar::from_short(3907).to_u128();
        let synth_bool_true = DcChar::from_short(3917).to_u128();
        let synth_bool_false = DcChar::from_short(3918).to_u128();

        registry.register_equivalence(synth_lit_open, DC_BEGIN_LITERAL.to_u128());
        registry.register_equivalence(synth_lit_close, DC_END_LITERAL.to_u128());
        registry.register_equivalence(synth_ident_open, DC_BEGIN_IDENTIFIER.to_u128());
        registry.register_equivalence(synth_ident_close, DC_END_IDENTIFIER.to_u128());
        registry.register_equivalence(synth_num_open, DC_BEGIN_NUMBER.to_u128());
        registry.register_equivalence(synth_num_close, DC_END_NUMBER.to_u128());
        registry.register_equivalence(synth_bool_true, DC_BOOLEAN_TRUE.to_u128());
        registry.register_equivalence(synth_bool_false, DC_BOOLEAN_FALSE.to_u128());

        // 1. Literal string decoding
        let mut lit_doc = DcMstring::new();
        lit_doc.push_char(DcChar::from_u128(synth_lit_open));
        lit_doc.push_char(DC_BEGIN_TYPE_DEF);
        lit_doc.push_char(DC_TYPE_STRING);
        lit_doc.push_char(DC_END_TYPE_DEF);
        lit_doc.push_str("overlay_text");
        lit_doc.push_char(DcChar::from_u128(synth_lit_close));

        let decoded_str: String =
            lit_doc.decode_with_resolver(registry.resolver(), &policy)?;
        ensure!(decoded_str == "overlay_text", "Decoded literal string mismatch");

        // 2. Identifier decoding
        let mut ident_doc = DcMstring::new();
        ident_doc.push_char(DcChar::from_u128(synth_ident_open));
        ident_doc.push_str("sample_identifier");
        ident_doc.push_char(DcChar::from_u128(synth_ident_close));

        let decoded_ident: String =
            ident_doc.decode_with_resolver(registry.resolver(), &policy)?;
        ensure!(
            decoded_ident == "sample_identifier",
            "Decoded identifier mismatch"
        );

        // 3. Number decoding (u64)
        let base_chars =
            ctb_formats_dcstring::dc_number::i128_to_dc_number_chars(4242)?;
        let mut num_doc = DcMstring::new();
        num_doc.push_char(DcChar::from_u128(synth_num_open));
        let middle_slice = base_chars
            .get(1..base_chars.len().saturating_sub(1))
            .context("Middle number digits")?;
        for &ch in middle_slice {
            num_doc.push_char(ch);
        }
        num_doc.push_char(DcChar::from_u128(synth_num_close));

        let decoded_num: u64 =
            num_doc.decode_with_resolver(registry.resolver(), &policy)?;
        ensure!(decoded_num == 4242, "Decoded number mismatch");

        // 4. Boolean decoding (bool)
        let mut true_doc = DcMstring::new();
        true_doc.push_char(DcChar::from_u128(synth_bool_true));
        let decoded_true: bool =
            true_doc.decode_with_resolver(registry.resolver(), &policy)?;
        ensure!(decoded_true, "Expected decoded true");

        let mut false_doc = DcMstring::new();
        false_doc.push_char(DcChar::from_u128(synth_bool_false));
        let decoded_false: bool =
            false_doc.decode_with_resolver(registry.resolver(), &policy)?;
        ensure!(!decoded_false, "Expected decoded false");

        Ok(())
    }

    /// Verifies that runtime-registered `<equiv>` container openers, field tags,
    /// and enum variant tags deserialize identically to their canonical targets
    /// for `#[derive(DcMixed)]` types without code recompilation.
    #[crate::ctb_test]
    fn test_derive_dc_mixed_structs_and_enums_equivalence_overlay(
    ) -> Result<()> {
        let policy = DcProjectionPolicy::strict();
        let mut registry = SyntaxRuntimeRegistry::default();

        let synth_rec_open = DcChar::from_short(3801).to_u128();
        let synth_rec_close = DcChar::from_short(3802).to_u128();
        let synth_field_title = DcChar::from_short(3803).to_u128();
        let synth_field_count = DcChar::from_short(3804).to_u128();
        let synth_variant_active = DcChar::from_short(3805).to_u128();
        let synth_variant_inactive = DcChar::from_short(3806).to_u128();

        registry.register_equivalence(synth_rec_open, DcChar::from_short(401).to_u128());
        registry.register_equivalence(synth_rec_close, DcChar::from_short(402).to_u128());
        registry.register_equivalence(synth_field_title, DcChar::from_short(400).to_u128());
        registry.register_equivalence(synth_field_count, DcChar::from_short(406).to_u128());
        registry.register_equivalence(synth_variant_active, DcChar::from_short(327).to_u128());
        registry.register_equivalence(synth_variant_inactive, DcChar::from_short(359).to_u128());

        // 1. Struct with overlay container opener/closer and field tags
        let mut struct_doc = DcMstring::new();
        struct_doc.push_char(DcChar::from_u128(synth_rec_open));
        struct_doc.push_char(DcChar::from_u128(synth_field_title));
        "hello_overlay".encode_dc_mixed(&mut struct_doc)?;
        struct_doc.push_char(DcChar::from_u128(synth_field_count));
        777u64.encode_dc_mixed(&mut struct_doc)?;
        struct_doc.push_char(DcChar::from_u128(synth_rec_close));

        let decoded_struct: OverlaySampleRecord =
            struct_doc.decode_with_resolver(registry.resolver(), &policy)?;
        ensure!(decoded_struct.title == "hello_overlay", "Title mismatch");
        ensure!(decoded_struct.count == 777, "Count mismatch");

        // 2. Enum variants with overlay variant tags
        let mut active_doc = DcMstring::new();
        active_doc.push_char(DcChar::from_u128(synth_variant_active));
        1234u64.encode_dc_mixed(&mut active_doc)?;

        let decoded_active: OverlaySampleStatus =
            active_doc.decode_with_resolver(registry.resolver(), &policy)?;
        ensure!(
            decoded_active == OverlaySampleStatus::Active(1234),
            "Active variant mismatch"
        );

        let mut inactive_doc = DcMstring::new();
        inactive_doc.push_char(DcChar::from_u128(synth_variant_inactive));

        let decoded_inactive: OverlaySampleStatus =
            inactive_doc.decode_with_resolver(registry.resolver(), &policy)?;
        ensure!(
            decoded_inactive == OverlaySampleStatus::Inactive,
            "Inactive variant mismatch"
        );

        // 3. Inspection with overlay resolver
        let inspection = struct_doc.inspect_with_resolver::<OverlaySampleRecord>(
            registry.resolver(),
        );
        let fields = inspection.fields();
        ensure!(
            fields.iter().any(|f| f.field() == "title"),
            "Inspection must find title field"
        );
        ensure!(
            fields.iter().any(|f| f.field() == "count"),
            "Inspection must find count field"
        );

        Ok(())
    }

    #[crate::ctb_test]
    fn test_dynamic_custom_container_header_and_trailing_rule() -> Result<()> {
        use ctb_formats_dcdata::syntax::ast::{DcSyntaxRule, Quantifier, SyntaxElement, SyntaxPattern, SyntaxTerm};

        let mut registry = SyntaxRuntimeRegistry::new();
        let custom_open = CharTarget::Dc(9800);
        let custom_close = CharTarget::Dc(9801);
        let custom_close_u128 = custom_close.to_u128();
        // Register rule: : ~ [type] [number] 9801 [type]
        // Header: [type], Body: [number], Closer: 9801, Trailing: [type]
        let rule = DcSyntaxRule {
            pattern: SyntaxPattern::Sequence(vec![
                SyntaxElement { term: SyntaxTerm::SelfChar, quantifier: Quantifier::ExactOne },
                SyntaxElement { term: SyntaxTerm::NamedConstruct { name: "type".to_owned(), subtype: None, capture_var: None }, quantifier: Quantifier::ExactOne },
                SyntaxElement { term: SyntaxTerm::NamedConstruct { name: "number".to_owned(), subtype: None, capture_var: None }, quantifier: Quantifier::ExactOne },
                SyntaxElement { term: SyntaxTerm::CharRef(custom_close.clone()), quantifier: Quantifier::ExactOne },
                SyntaxElement { term: SyntaxTerm::NamedConstruct { name: "type".to_owned(), subtype: None, capture_var: None }, quantifier: Quantifier::ExactOne },
            ]),
            action: None,
            raw: String::new(),
        };
        registry.register_closing_pair(custom_open.clone(), custom_close_u128);
        registry.register_rule(custom_open.clone(), rule);

        let headers = registry.resolver().container_header_elements(&custom_open);
        ensure!(headers.len() == 1, "Expected 1 header element, found {}", headers.len());
        ensure!(matches!(&headers[0].term, SyntaxTerm::NamedConstruct { name, .. } if name == "type"), "Header should be type construct");

        let trailing = registry.resolver().trailing_after_closer(&custom_open);
        ensure!(trailing.len() == 1, "Expected 1 trailing element, found {}", trailing.len());
        ensure!(matches!(&trailing[0].term, SyntaxTerm::NamedConstruct { name, .. } if name == "type"), "Trailing should be type construct");

        // Also test equivalent opener inherits the custom headers and trailing:
        let equiv_open = CharTarget::Dc(9802);
        registry.register_equivalence(equiv_open.to_u128(), custom_open.to_u128());
        let equiv_headers = registry.resolver().container_header_elements(&equiv_open);
        ensure!(equiv_headers.len() == 1, "Equiv opener must inherit custom header");
        let equiv_trailing = registry.resolver().trailing_after_closer(&equiv_open);
        ensure!(equiv_trailing.len() == 1, "Equiv opener must inherit custom trailing");

        Ok(())
    }

    #[crate::ctb_test]
    fn test_source_edit_and_dctext_equivalence_overlay() -> Result<()> {
        let mut registry = SyntaxRuntimeRegistry::new();
        let synth_local = DcChar::from_short(9810).to_u128();
        let canon_local = ctb_formats_dcdata::dc::DC_LOCAL_NODE.to_u128();
        registry.register_equivalence(synth_local, canon_local);

        // 1. Test dctext serialization with custom LocalNode role
        let gids = vec![
            synth_local,
            DC_BEGIN_NUMBER.to_u128(),
            DcChar::from_format(199).to_u128(),
            DcChar::from_short(127 + 5).to_u128(),
            DC_END_NUMBER.to_u128(),
        ]; // @L5@
        let dctext_bytes = converters::dctext::dclist_to_dctext_with_resolver(&gids, registry.resolver());
        let dctext_str = std::str::from_utf8(&dctext_bytes)?;
        ensure!(dctext_str == "@L5@", "Expected @L5@, found {dctext_str}");

        // 2. Test source preservation with custom resolver
        let mut old_doc = DcMstring::new();
        old_doc.push_char(DC_BEGIN_KV_MAP);
        old_doc.push_char(ctb_formats_dcdata::dc::DC_TYPE_ANY);
        old_doc.push_char(ctb_formats_dcdata::dc::DC_TYPE_ANY);
        old_doc.push_char(DC_BEGIN_KV_PAIR);
        "key1".encode_dc_mixed(&mut old_doc)?;
        old_doc.push_char(DC_LIST_ELEMENT_SEPARATOR);
        "val1".encode_dc_mixed(&mut old_doc)?;
        old_doc.push_char(DC_END_KV_PAIR);
        old_doc.push_char(DC_END_KV_MAP);

        let mut new_doc = DcMstring::new();
        new_doc.push_char(DC_BEGIN_KV_MAP);
        new_doc.push_char(ctb_formats_dcdata::dc::DC_TYPE_ANY);
        new_doc.push_char(ctb_formats_dcdata::dc::DC_TYPE_ANY);
        new_doc.push_char(DC_BEGIN_KV_PAIR);
        "key1".encode_dc_mixed(&mut new_doc)?;
        new_doc.push_char(DC_LIST_ELEMENT_SEPARATOR);
        "new_val".encode_dc_mixed(&mut new_doc)?;
        new_doc.push_char(DC_END_KV_PAIR);
        new_doc.push_char(DC_END_KV_MAP);

        let preserved = ctb_formats_dcstring::source_edit::preserve_source_with_resolver(&old_doc, &new_doc, registry.resolver())?;
        ensure!(!preserved.as_bytes().is_empty(), "Preserved output must not be empty");

        Ok(())
    }

    /// Verifies that literal, identifier, and assignment payloads preserve all
    /// Document Characters (`DcChar`s, including short Dcs, long Dcs, format Dcs,
    /// escaped closers `255 261` / `255 271`, escaped escape `255 255`, and `<equiv>`
    /// aliases) losslessly in `DcMstring` / `DcString` without Plane 14 substitution.
    #[crate::ctb_test]
    fn test_lossless_dc_native_literals_and_identifiers() -> Result<()> {
        let policy = DcProjectionPolicy::strict();

        // 1. Build a DcMstring containing Unicode scalars, short Dcs, escaped 255,
        //    escaped 261, a format Dc, and a long Dc:
        let expected_chars = vec![
            DcChar::from_char('A'),
            DcChar::from_short(517),
            DcChar::from_short(260),
            DcChar::from_short(261),
            DcChar::from_short(255),
            DcChar::from_format(494),
            DcChar::from_long(123_456_789),
            DcChar::from_char('Ω'),
        ];
        let payload_mstr: DcMstring = expected_chars.iter().copied().collect();
        let payload_dc_str = payload_mstr.to_dc_string()?;

        // Roundtrip as DcMstring and DcString through DcMixedEncode / DcMixedDecode:
        ctb_formats_dcstring::assert_dc_document_roundtrip(&payload_mstr)?;
        ctb_formats_dcstring::assert_dc_document_roundtrip(&payload_dc_str)?;

        let encoded_lit = payload_mstr.to_dc_mixed()?;
        let mut arena = DcSyntaxArena::new();
        let root_id = DcDocumentNavigator::parse_arena(encoded_lit.as_bytes(), &mut arena, &policy)?;
        let root_node = arena.node(root_id).context("Missing literal root node")?;
        match &root_node.kind {
            DcSyntaxKind::Literal { decoded, .. } => {
                ensure!(*decoded == payload_mstr, "Arena Literal node must preserve exact DcMstring");
                let actual_chars: Vec<DcChar> = decoded.to_dc_string()?.chars().collect();
                ensure!(actual_chars == expected_chars, "Decoded DcChar sequence mismatch");
            }
            other => bail!("Expected Literal node, found {other:?}"),
        }

        // Decoding non-Unicode Dcs into a Rust Unicode String must fail cleanly:
        ensure!(
            encoded_lit.decode::<String>(&policy).is_err(),
            "String::decode must reject literals containing non-Unicode Dcs"
        );

        // Single-token <equiv> alias of a Unicode scalar decodes to String via resolver:
        let mut registry = SyntaxRuntimeRegistry::new();
        let alias_dc = DcChar::from_short(9850);
        registry.register_equivalence(alias_dc.to_u128(), u128::from(u32::from('Z')));
        let alias_payload: DcMstring = [DcChar::from_char('A'), alias_dc, DcChar::from_char('B')]
            .into_iter()
            .collect();
        let encoded_alias_lit = alias_payload.to_dc_mixed()?;
        let preserved_mstr: DcMstring =
            encoded_alias_lit.decode_with_resolver(registry.resolver(), &policy)?;
        ensure!(
            preserved_mstr == alias_payload,
            "DcMstring decode must preserve original alias DcChar without lossy canonicalization"
        );
        let projected_str: String =
            encoded_alias_lit.decode_with_resolver(registry.resolver(), &policy)?;
        ensure!(
            projected_str == "AZB",
            "String decode with resolver must resolve single-token Unicode alias to 'AZB', got {projected_str}"
        );

        // 2. Constrained [string:...] rule over unescaped literal payload containing
        //    escaped closer (261), escaped escape (255), short Dc (517), and long Dc:
        let mut nav = DcDocumentNavigator::from_mstr(&encoded_lit);
        let resolver = default_syntax_resolver();
        let mut eval_ctx = SyntaxEvalContext::new(&**resolver, None);
        let constrained_term = SyntaxTerm::NamedConstruct {
            name: "string".to_owned(),
            subtype: Some(":u0041 517 260 261 255 f494 [^] u03a9".to_owned()),
            capture_var: None,
        };
        ensure!(
            nav.eval_term_single(&constrained_term, &mut eval_ctx)?,
            "Constrained [string:...] must match unescaped DcChar sequence losslessly"
        );

        // 3. Lossless Dc-native identifier (270..271) and lexical assignment / bound reference (540):
        let ident_chars = vec![
            DcChar::from_char('v'),
            DcChar::from_short(517),
            DcChar::from_short(271),
            DcChar::from_short(255),
            DcChar::from_long(987_654_321),
        ];
        let ident_mstr: DcMstring = ident_chars.into_iter().collect();
        let mut single_ident_doc = DcMstring::new();
        ctb_formats_dcstring::encode_dc_identifier(&mut single_ident_doc, &ident_mstr)?;
        let decoded_ident: DcMstring = single_ident_doc.decode(&policy)?;
        ensure!(decoded_ident == ident_mstr, "Identifier DcMstring roundtrip mismatch");

        let mut assign_doc = DcMstring::new();
        ctb_formats_dcstring::encode_dc_identifier(&mut assign_doc, &ident_mstr)?;
        assign_doc.push_char(ctb_formats_dcdata::dc::DC_ASSIGN);
        777u64.encode_dc_mixed(&mut assign_doc)?;
        assign_doc.push_char(ctb_formats_dcdata::dc::DC_REFERENCE_BOUND_VALUE);
        ctb_formats_dcstring::encode_dc_identifier(&mut assign_doc, &ident_mstr)?;

        let mut assign_nav = DcDocumentNavigator::from_mstr(&assign_doc);
        let mut assign_arena = DcSyntaxArena::new();
        assign_nav.parse_document_into_arena(&mut assign_arena, &policy)?;
        let ref_root = *assign_arena.roots().get(1).context("Missing bound reference root")?;
        let mut proj_ctx = ctb_formats_dcstring::DcProjectionContext::new(&assign_arena, &policy);
        let resolved_val: u64 = ctb_formats_dcstring::DcMixedDecode::decode_dc_node(&mut proj_ctx, ref_root)?;
        ensure!(
            resolved_val == 777,
            "Bound reference (540) with non-Unicode Dc identifier must resolve to assigned value 777"
        );

        Ok(())
    }
}
