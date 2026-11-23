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

//! Syntax pattern evaluation, construct validation, and subtype constraints.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::{
    Arc, CharTarget, Cow, DCMM_HEADER_LEN, DcByteSpan, DcCharExt,
    DcDocumentNavigator, DcFrameHeaderInfo, DcMstr, DcMstring,
    DcProjectionPolicy, DcRole, DcStr, DcString, DcSyntaxArena, DcSyntaxKind,
    Digest, MAX_NAV_CACHE_ENTRIES, NavBackend, Quantifier, Read,
    SubtypeConstraint, SyntaxDiagnostic, SyntaxElement, SyntaxPattern,
    SyntaxRuleResolver, SyntaxTerm, default_syntax_resolver,
};

/// Maximum recursion depth allowed during grammar rule expansion.
pub const MAX_SYNTAX_EXPANSION_DEPTH: usize = 32;

/// Default maximum container and wrapper nesting depth during arena building and projection.
pub const DEFAULT_MAX_PROJECTION_DEPTH: usize = 64;

/// Requirement for a value position in Document Character syntax.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueRequirement<'a> {
    /// A named construct from README-named-types.csv, e.g. `("string", None)` or `("number", Some(&SubtypeConstraint::Number(...)))`.
    NamedConstruct {
        /// Name of the construct.
        name: &'a str,
        /// Optional subtype constraint.
        subtype: Option<&'a SubtypeConstraint>,
    },
    /// A specific format or type token.
    TypeToken(DcChar),
    /// A specific container or tagged construct opener (up to `<equiv>`).
    ConstructOpener(DcChar),
}

/// Result of evaluating an expression in a value position.
#[derive(Debug, Clone)]
pub enum DcEvaluatedValue {
    /// Evaluated `DcMstring` / token sequence that can be spliced or validated against any syntax rule.
    Document(DcMstring),
    /// Direct typed shared value from `SharedValueContext`.
    SharedBinding {
        /// Name of the shared binding.
        name: String,
    },
}

/// Trait for evaluating expressions (e.g. Lisp/Scheme-like expressions or stream splicing) in Dc streams.
pub trait DcExpressionEvaluator: Send + Sync {
    /// Evaluates an expression node when encountered in a value position.
    ///
    /// # Errors
    /// Returns an error if expression evaluation fails.
    fn eval_expression_for_requirement(
        &self,
        expr_tokens: &[DcChar],
        requirement: &ValueRequirement<'_>,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<Option<DcEvaluatedValue>>;

    /// Evaluates a stream-splicing invocation and returns
    /// the `DcChar` tokens to splice into the navigator's active stream.
    ///
    /// # Errors
    /// Returns an error if evaluating the splice invocation fails.
    fn eval_stream_splice(
        &self,
        call_tokens: &[DcChar],
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<Option<Vec<DcChar>>>;
}

/// Default expression evaluator supporting equivalent representations and
/// Dc-WASM helper routines.
#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultDcExpressionEvaluator;

/// Prevents candidate expressions from executing during graph syntax recognition.
struct SyntaxOnlyDcExpressionEvaluator;

impl DcExpressionEvaluator for SyntaxOnlyDcExpressionEvaluator {
    fn eval_expression_for_requirement(
        &self,
        _expr_tokens: &[DcChar],
        _requirement: &ValueRequirement<'_>,
        _resolver: &dyn SyntaxRuleResolver,
    ) -> Result<Option<DcEvaluatedValue>> {
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

/// Returns `true` when `tokens` matches the named construct `construct_name` completely
/// under `resolver` without executing nested candidate expressions.
pub(crate) fn tokens_match_named_construct_with_resolver(
    tokens: &[DcChar],
    construct_name: &str,
    resolver: &dyn SyntaxRuleResolver,
) -> Result<bool> {
    let mut prog_mstr = DcMstring::new();
    for &token in tokens {
        prog_mstr.push_char(token);
    }
    let mut routine_navigator = DcDocumentNavigator::from_mstr(&prog_mstr);
    let mut routine_context = SyntaxEvalContext::new(resolver, None);
    routine_context.evaluator = Some(Arc::new(SyntaxOnlyDcExpressionEvaluator));
    let matched = routine_navigator.eval_term_single(
        &SyntaxTerm::NamedConstruct {
            name: construct_name.to_owned(),
            subtype: None,
            capture_var: None,
        },
        &mut routine_context,
    )?;
    Ok(
        matched
            && !routine_context.has_errors
            && routine_navigator.position() == routine_navigator.active_span().end,
    )
}

impl DefaultDcExpressionEvaluator {
    /// Creates a [`SharedQuotaDcExpressionEvaluator`] initialized with explicit [`crate::DcWasmResourceLimits`].
    #[must_use]
    pub fn with_resource_limits(
        limits: crate::DcWasmResourceLimits,
    ) -> SharedQuotaDcExpressionEvaluator {
        SharedQuotaDcExpressionEvaluator::with_resource_limits(limits)
    }

    /// Creates a [`SharedQuotaDcExpressionEvaluator`] backed by an existing [`crate::converters::wasm::SharedWasmTaskQuota`].
    #[must_use]
    pub const fn with_shared_task_quota(
        shared_quota: crate::converters::wasm::SharedWasmTaskQuota,
    ) -> SharedQuotaDcExpressionEvaluator {
        SharedQuotaDcExpressionEvaluator::with_shared_task_quota(shared_quota)
    }
}

/// Expression evaluator sharing a task-level [`crate::converters::wasm::SharedWasmTaskQuota`]
/// across all helper evaluations and nested syntax contexts.
#[derive(Debug, Clone)]
pub struct SharedQuotaDcExpressionEvaluator {
    shared_quota: crate::converters::wasm::SharedWasmTaskQuota,
}

impl Default for SharedQuotaDcExpressionEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl SharedQuotaDcExpressionEvaluator {
    /// Creates a [`SharedQuotaDcExpressionEvaluator`] with default [`crate::DcWasmResourceLimits`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            shared_quota: crate::converters::wasm::SharedWasmTaskQuota::new(
                crate::DcWasmResourceLimits::default(),
            ),
        }
    }

    /// Creates a [`SharedQuotaDcExpressionEvaluator`] initialized with explicit [`crate::DcWasmResourceLimits`].
    #[must_use]
    pub fn with_resource_limits(limits: crate::DcWasmResourceLimits) -> Self {
        Self {
            shared_quota: crate::converters::wasm::SharedWasmTaskQuota::new(
                limits,
            ),
        }
    }

    /// Creates a [`SharedQuotaDcExpressionEvaluator`] backed by an existing [`crate::converters::wasm::SharedWasmTaskQuota`].
    #[must_use]
    pub const fn with_shared_task_quota(
        shared_quota: crate::converters::wasm::SharedWasmTaskQuota,
    ) -> Self {
        Self { shared_quota }
    }

    /// Returns the shared [`crate::converters::wasm::SharedWasmTaskQuota`] for this evaluator.
    #[must_use]
    pub const fn shared_task_quota(
        &self,
    ) -> &crate::converters::wasm::SharedWasmTaskQuota {
        &self.shared_quota
    }
}

impl DcExpressionEvaluator for DefaultDcExpressionEvaluator {
    fn eval_expression_for_requirement(
        &self,
        expr_tokens: &[DcChar],
        requirement: &ValueRequirement<'_>,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<Option<DcEvaluatedValue>> {
        eval_expression_for_requirement_impl(
            expr_tokens,
            requirement,
            resolver,
            None,
        )
    }

    fn eval_stream_splice(
        &self,
        call_tokens: &[DcChar],
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<Option<Vec<DcChar>>> {
        eval_stream_splice_impl(call_tokens, resolver, None)
    }
}

impl DcExpressionEvaluator for SharedQuotaDcExpressionEvaluator {
    fn eval_expression_for_requirement(
        &self,
        expr_tokens: &[DcChar],
        requirement: &ValueRequirement<'_>,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<Option<DcEvaluatedValue>> {
        eval_expression_for_requirement_impl(
            expr_tokens,
            requirement,
            resolver,
            Some(self.shared_quota.clone()),
        )
    }

    fn eval_stream_splice(
        &self,
        call_tokens: &[DcChar],
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<Option<Vec<DcChar>>> {
        eval_stream_splice_impl(
            call_tokens,
            resolver,
            Some(self.shared_quota.clone()),
        )
    }
}

pub(crate) fn resolve_pinned_syntax_resolver(
    base_resolver: &dyn SyntaxRuleResolver,
    pin_node_id: u128,
) -> Result<Arc<ctb_formats_dcdata::syntax::DatasetRuleResolver>> {
    use ctb_formats_dcdata::global_graph_layout::{
        SYSTEM_REGION_END, SYSTEM_REGION_START,
        unsupported_graph_block_description,
    };
    use ctb_formats_dcdata::seed_graph::SeedGraphIndex;
    use ctb_formats_dcdata::syntax::DatasetRuleResolver;

    if let Some(block_desc) = unsupported_graph_block_description(pin_node_id) {
        bail!(
            "Unsupported graph block pin in Dc 641: node ID l{pin_node_id} targets {block_desc}"
        );
    }
    ensure!(
        (SYSTEM_REGION_START..=SYSTEM_REGION_END).contains(&pin_node_id),
        "Invalid System-block most recent node ID {pin_node_id} for Dc 641: expected System block range {SYSTEM_REGION_START}..={SYSTEM_REGION_END}"
    );
    if let Some(sg) = base_resolver.seed_graph() {
        let head_id = sg
            .system_block_most_recent_node_id()
            .max(sg.nodes().iter().map(|n| n.id).max())
            .context(
                "Cannot resolve Dc 641 pin against an empty System graph",
            )?;
        ensure!(
            pin_node_id <= head_id,
            "Document pin Dc 641 requires a newer System graph snapshot than the local graph (l{pin_node_id} > l{head_id})"
        );
        if !sg.is_from_asset_bundle() {
            let pinned_sg = Arc::new(SeedGraphIndex::from_nodes_at_most_recent(
                sg.nodes().to_vec(),
                pin_node_id,
            ));
            return Ok(Arc::new(DatasetRuleResolver::from_seed_graph_arc(
                pinned_sg,
            )?));
        }
    }
    ctb_formats_dcdata::syntax::syntax_resolver_at_most_recent(pin_node_id)
}

pub(crate) fn extract_pin_node_id_from_pin_description(
    pin_tokens: &[DcChar],
    resolver: &dyn SyntaxRuleResolver,
    shared_quota: Option<crate::converters::wasm::SharedWasmTaskQuota>,
) -> Result<u128> {
    let first = pin_tokens
        .first()
        .copied()
        .context("Empty pin_description in ResolveToPin")?;
    ensure!(
        first.fills_role_with(
            DcRole::GlobalGraphSystemBlockMostRecentNodeId,
            resolver
        ),
        "Unsupported pin_description opener {first} in ResolveToPin (expected Dc 641)"
    );
    // Reason for fallback: missing operand after Dc 641 produces an empty slice rejected below.
    let operand_slice = pin_tokens.get(1..).unwrap_or(&[]);
    let second = operand_slice
        .first()
        .copied()
        .context("Missing System-block node ID operand after Dc 641")?;
    if second.fills_role_with(DcRole::Escape, resolver)
        || second.fills_role_with(DcRole::QuoteFollowingDcAsAtom, resolver)
    {
        ensure!(
            operand_slice.len() == 2,
            "Trailing tokens in escaped/quoted Dc 641 atom_dc operand"
        );
        let escaped = operand_slice
            .get(1)
            .copied()
            .context("Dangling escape/quote in Dc 641 atom_dc operand")?;
        return Ok(escaped.to_u128());
    }
    if second.fills_role_with(DcRole::EvaluateValueAsAtom, resolver) {
        // Reason for fallback: missing operand after Dc 616 produces an empty slice rejected below.
        let inner_expr = operand_slice.get(1..).unwrap_or(&[]);
        let eval_doc = eval_expression_for_requirement_impl(
            inner_expr,
            &ValueRequirement::NamedConstruct {
                name: "value",
                subtype: None,
            },
            resolver,
            shared_quota,
        )?
        .context("Dc 616 inside Dc 641 pin_description failed to evaluate")?;
        let DcEvaluatedValue::Document(mstr) = eval_doc else {
            bail!(
                "Dc 616 inside Dc 641 pin_description must evaluate to a Document value"
            );
        };
        let chars: Vec<DcChar> = mstr.to_dc_string()?.chars().collect();
        ensure!(
            chars.len() == 1,
            "Dc 616 inside Dc 641 pin_description must evaluate to 1 atom Dc, got {} tokens",
            chars.len()
        );
        let pin_ch = chars
            .first()
            .copied()
            .context("Empty evaluated Dc 641 pin")?;
        return Ok(pin_ch.to_u128());
    }
    ensure!(
        operand_slice.len() == 1,
        "Unexpected trailing tokens in Dc 641 pin_description"
    );
    Ok(second.to_u128())
}

fn split_resolve_to_pin_operands(
    tokens: &[DcChar],
    resolver: &dyn SyntaxRuleResolver,
    shared_quota: Option<crate::converters::wasm::SharedWasmTaskQuota>,
) -> Result<(
    Arc<ctb_formats_dcdata::syntax::DatasetRuleResolver>,
    Vec<DcChar>,
)> {
    let mut rest_mstr = DcMstring::new();
    for &tok in tokens.iter().skip(1) {
        rest_mstr.push_char(tok);
    }
    let mut nav = DcDocumentNavigator::from_mstr(&rest_mstr);
    let start_pos = nav.position();
    let mut ctx = SyntaxEvalContext::new(resolver, None);
    if let Some(ref shared) = shared_quota {
        ctx.evaluator = Some(Arc::new(
            SharedQuotaDcExpressionEvaluator::with_shared_task_quota(
                shared.clone(),
            ),
        ));
    }
    let matched_pin = nav.eval_term_single(
        &SyntaxTerm::NamedConstruct {
            name: "pin_description".to_owned(),
            subtype: None,
            capture_var: None,
        },
        &mut ctx,
    )?;
    ensure!(
        matched_pin && !ctx.has_errors && nav.position() > start_pos,
        "ResolveToPin requires a valid leading [pin_description]"
    );
    let pin_end = nav.position();
    let pin_u128s =
        nav.read_tokens_in_span(DcByteSpan::new(start_pos, pin_end))?;
    let pin_chars: Vec<DcChar> =
        pin_u128s.into_iter().map(DcChar::from_u128).collect();
    let pin_node_id = extract_pin_node_id_from_pin_description(
        &pin_chars,
        resolver,
        shared_quota,
    )?;
    let pinned_res = resolve_pinned_syntax_resolver(resolver, pin_node_id)?;
    let total_len = u64::try_from(rest_mstr.len())?;
    ensure!(
        pin_end < total_len,
        "ResolveToPin is missing its target ([value] | [atom_dc]) operand"
    );
    let operand_u128s =
        nav.read_tokens_in_span(DcByteSpan::new(pin_end, total_len))?;
    let operand_chars: Vec<DcChar> =
        operand_u128s.into_iter().map(DcChar::from_u128).collect();
    Ok((pinned_res, operand_chars))
}

fn strip_leading_dc_comment_frames<'a>(
    mut chars: &'a [DcChar],
    resolver: &dyn SyntaxRuleResolver,
) -> Result<&'a [DcChar]> {
    while let Some(&first) = chars.first() {
        let end_role = if first
            .fills_role_with(DcRole::BeginSingleLineComment, resolver)
            || first.fills_role_with(
                DcRole::BeginSingleLineCommentNewLine,
                resolver,
            ) {
            Some(DcRole::EndSingleLineComment)
        } else if first.fills_role_with(DcRole::BeginBlockComment, resolver)
            || first.fills_role_with(DcRole::BeginBlockCommentNewLine, resolver)
        {
            Some(DcRole::EndBlockComment)
        } else {
            None
        };
        let Some(closer_role) = end_role else {
            break;
        };
        let mut idx = 1usize;
        let mut closed = false;
        while let Some(&ch) = chars.get(idx) {
            if ch.fills_role_with(DcRole::Escape, resolver) {
                idx = idx.saturating_add(2);
                continue;
            }
            idx = idx.saturating_add(1);
            if ch.fills_role_with(closer_role, resolver) {
                closed = true;
                break;
            }
        }
        ensure!(closed, "Unterminated comment frame in target Dc document");
        // Reason for fallback: idx == chars.len() yields an empty trailing slice.
        chars = chars.get(idx..).unwrap_or(&[]);
    }
    Ok(chars)
}

/// Evaluates `Dc 2608 "Convert Dc string to WASM"` (`.EM Miscellaneous`) via the
/// System-graph StageR 3 implementation (`impls/std/convert_dc_string_to_wasm.sr3`)
/// when active in the pinned snapshot, falling back to this Rust implementation
/// strictly for pre-migration historical `Dc 641` pins lacking an active
/// StageR decomposition on `Dc 2608`.
fn eval_convert_dc_string_to_wasm(
    call_tokens: &[DcChar],
    resolver: &dyn SyntaxRuleResolver,
    shared_quota: Option<crate::converters::wasm::SharedWasmTaskQuota>,
) -> Result<DcMstring> {
    use ctb_storage_minimal::shorthand::DcShorthand;

    let head_dc = call_tokens
        .first()
        .copied()
        .context("Empty ConvertDcStringToWasm invocation")?;
    // Reason for fallback: missing argument slice yields empty slice rejected below.
    let raw_arg = call_tokens.get(1..).unwrap_or(&[]);
    ensure!(
        !raw_arg.is_empty(),
        "ConvertDcStringToWasm requires a [string] argument"
    );

    let unwrapped_arg = if raw_arg
        .first()
        .is_some_and(|c| c.fills_role_with(DcRole::BeginParameter, resolver))
    {
        ensure!(
            raw_arg.len() >= 2
                && raw_arg.last().is_some_and(|c| {
                    c.fills_role_with(DcRole::EndParameter, resolver)
                }),
            "Unclosed parameter container in ConvertDcStringToWasm"
        );
        raw_arg
            .get(1..raw_arg.len().saturating_sub(1))
            .context("Parameter container slice out of bounds")?
    } else {
        raw_arg
    };
    ensure!(
        !unwrapped_arg.is_empty(),
        "ConvertDcStringToWasm parameter is empty"
    );

    let normalized_call: Vec<DcChar> = if unwrapped_arg
        .first()
        .is_some_and(|c| c.fills_role_with(DcRole::BeginLiteral, resolver))
    {
        call_tokens.to_vec()
    } else {
        let evaluated = eval_expression_for_requirement_impl(
            unwrapped_arg,
            &ValueRequirement::NamedConstruct {
                name: "string",
                subtype: None,
            },
            resolver,
            shared_quota.clone(),
        )?
        .context(
            "ConvertDcStringToWasm operand did not evaluate to a [string]",
        )?;
        let DcEvaluatedValue::Document(str_doc) = evaluated else {
            bail!(
                "ConvertDcStringToWasm operand must evaluate to a Document [string]"
            );
        };
        let mut rebuilt = vec![head_dc];
        for ch in str_doc.to_dc_string()?.chars() {
            rebuilt.push(ch);
        }
        rebuilt
    };

    let string_slice = if normalized_call
        .get(1)
        .is_some_and(|c| c.fills_role_with(DcRole::BeginParameter, resolver))
    {
        // Reason for fallback: parameter wrapper bounds were checked above.
        normalized_call
            .get(2..normalized_call.len().saturating_sub(1))
            .unwrap_or(&[])
    } else {
        // Reason for fallback: normalized_call always starts with head_dc followed by string tokens.
        normalized_call.get(1..).unwrap_or(&[])
    };
    ensure!(
        tokens_match_named_construct_with_resolver(
            string_slice,
            "string",
            resolver,
        )?,
        "ConvertDcStringToWasm requires a valid [string] literal argument"
    );

    let mut canonical_call =
        Vec::with_capacity(string_slice.len().saturating_add(1));
    canonical_call.push(head_dc);
    canonical_call.extend_from_slice(string_slice);
    if let Some(wasm_doc) =
        crate::converters::wasm::run_graph_stager_dc_2608_with_quota(
            &canonical_call,
            resolver,
            shared_quota.clone(),
        )?
    {
        return Ok(wasm_doc);
    }

    let unescaped_doc = if let Some(wasm_shorthands) =
        resolver.resolve_wasm_definition_tokens(head_dc.to_u128())
    {
        crate::converters::wasm::run_graph_wasm_tokens_on_dc_slice_with_quota(
            wasm_shorthands,
            &normalized_call,
            resolver,
            shared_quota.clone(),
        )?
    } else {
        unquote_validated_dc_string_literal(string_slice, resolver)?
    };

    let raw_target_chars: Vec<DcChar> =
        unescaped_doc.to_dc_string()?.chars().collect();
    let mut body_chars =
        strip_leading_dc_comment_frames(&raw_target_chars, resolver)?;
    ensure!(
        !body_chars.is_empty(),
        "ConvertDcStringToWasm target document contains no executable tokens"
    );

    if body_chars.first().is_some_and(|c| {
        c.fills_role_with(
            DcRole::GlobalGraphSystemBlockMostRecentNodeId,
            resolver,
        )
    }) {
        let pin_tokens: Vec<DcChar> =
            if resolver.resolve_named_type("pin_description").is_some() {
                let mut nav_mstr = DcMstring::new();
                for &tok in body_chars {
                    nav_mstr.push_char(tok);
                }
                let mut nav = DcDocumentNavigator::from_mstr(&nav_mstr);
                let start_pos = nav.position();
                let mut ctx = SyntaxEvalContext::new(resolver, None);
                let matched_pin = nav.eval_term_single(
                    &SyntaxTerm::NamedConstruct {
                        name: "pin_description".to_owned(),
                        subtype: None,
                        capture_var: None,
                    },
                    &mut ctx,
                )?;
                ensure!(
                    matched_pin && !ctx.has_errors && nav.position() > start_pos,
                    "Invalid leading [pin_description] inside target Dc document"
                );
                let pin_end = nav.position();
                let pin_u128s = nav
                    .read_tokens_in_span(DcByteSpan::new(start_pos, pin_end))?;
                pin_u128s.into_iter().map(DcChar::from_u128).collect()
            } else {
                let second = body_chars.get(1).copied().context(
                    "Missing System-block node ID operand after Dc 641",
                )?;
                let pin_len = if second.fills_role_with(DcRole::Escape, resolver)
                    || second.fills_role_with(
                        DcRole::QuoteFollowingDcAsAtom,
                        resolver,
                    )
                {
                    3usize
                } else {
                    2usize
                };
                body_chars
                    .get(..pin_len)
                    .context("Truncated leading Dc 641 pin inside target Dc document")?
                    .to_vec()
            };
        let inner_pin_id = extract_pin_node_id_from_pin_description(
            &pin_tokens,
            resolver,
            shared_quota.clone(),
        )?;
        if let Some(sg) = resolver.seed_graph()
            && let Some(wm) = sg.system_block_most_recent_node_id()
        {
            ensure!(
                inner_pin_id <= wm,
                "Inner document Dc 641 pin l{inner_pin_id} exceeds active pinned System graph snapshot l{wm}"
            );
        }
        // Reason for fallback: exhausted slice after pin_tokens.len() is rejected immediately below.
        let after_pin = body_chars.get(pin_tokens.len()..).unwrap_or(&[]);
        body_chars = strip_leading_dc_comment_frames(after_pin, resolver)?;
        ensure!(
            !body_chars.is_empty(),
            "Target Dc document contains only a pin_description and no body"
        );
    }

    let first_body = body_chars
        .first()
        .copied()
        .context("Empty target Dc document body")?;

    if first_body.fills_role_with(DcRole::BeginWasmProgram, resolver)
        || (!first_body.fills_role_with(DcRole::BeginRoutine, resolver)
            && tokens_match_named_construct_with_resolver(
                body_chars, "Routine", resolver,
            )?)
    {
        let body_shorthands: Vec<DcShorthand> = body_chars
            .iter()
            .map(|c| DcShorthand::from_global_id(c.to_u128()))
            .collect();
        return crate::converters::wasm::expand_graph_wasm_shorthands_to_dcmstring(
            &body_shorthands,
            resolver,
        );
    }

    if body_chars.len() == 1 {
        let single_sh = DcShorthand::from_global_id(first_body.to_u128());
        if let Some(sg) = resolver.seed_graph() {
            if let Some((_, wasm_tokens)) =
                sg.resolve_wasm_helper_shorthand(single_sh)?
            {
                return crate::converters::wasm::expand_graph_wasm_shorthands_to_dcmstring(
                    wasm_tokens,
                    resolver,
                );
            }
        } else if let Some(wasm_tokens) =
            resolver.resolve_wasm_definition_tokens(first_body.to_u128())
        {
            return crate::converters::wasm::expand_graph_wasm_shorthands_to_dcmstring(
                wasm_tokens,
                resolver,
            );
        }
    }

    let Some(sg) = resolver.seed_graph() else {
        bail!(
            "ConvertDcStringToWasm requires a SeedGraphIndex snapshot to resolve converter routines for {first_body}"
        );
    };
    let src_sh = DcShorthand::from_global_id(first_body.to_u128());
    let dst_sh = DcShorthand::from_global_id(
        ctb_formats_dcdata::dc::DC_BEGIN_WASM_PROGRAM.to_u128(),
    );
    let Some(chain) = sg.resolve_active_converter_chain(src_sh, dst_sh)? else {
        if is_uncompiled_code_token(first_body, resolver) {
            bail!(
                "No active converter routine or chain ([Dc 302 {src_sh} ... {dst_sh}]) found in System graph snapshot"
            );
        }
        return eval_historical_declarative_dc_document_to_wasm(
            body_chars,
            resolver,
            shared_quota,
        );
    };

    let raw_wasm_doc =
        crate::converters::wasm::run_graph_converter_chain_on_dc_slice_with_quota(
            &chain,
            body_chars,
            sg,
            resolver,
            shared_quota,
        )?;
    let out_chars: Vec<DcChar> = raw_wasm_doc.to_dc_string()?.chars().collect();
    let is_wasm_prog = out_chars
        .first()
        .is_some_and(|c| c.fills_role_with(DcRole::BeginWasmProgram, resolver))
        && out_chars.last().is_some_and(|c| {
            c.fills_role_with(DcRole::EndWasmProgram, resolver)
        });
    let is_wasm_routine = is_wasm_prog
        || (out_chars
            .first()
            .is_some_and(|c| !c.fills_role_with(DcRole::BeginRoutine, resolver))
            && tokens_match_named_construct_with_resolver(
                &out_chars, "Routine", resolver,
            )?);
    ensure!(
        is_wasm_routine,
        "Converter chain for {src_sh} -> {dst_sh} did not emit a valid Dc-WASM program (Dc 1927..Dc 1928)"
    );
    let out_shorthands: Vec<DcShorthand> = out_chars
        .iter()
        .map(|c| DcShorthand::from_global_id(c.to_u128()))
        .collect();
    crate::converters::wasm::expand_graph_wasm_shorthands_to_dcmstring(
        &out_shorthands,
        resolver,
    )
}

fn unquote_validated_dc_string_literal(
    tokens: &[DcChar],
    resolver: &dyn SyntaxRuleResolver,
) -> Result<DcMstring> {
    ensure!(
        tokens.len() >= 2
            && tokens
                .first()
                .is_some_and(|c| c.fills_role_with(DcRole::BeginLiteral, resolver))
            && tokens
                .last()
                .is_some_and(|c| c.fills_role_with(DcRole::EndLiteral, resolver)),
        "Invalid Dc 260..Dc 261 string literal slice"
    );
    let end = tokens.len().saturating_sub(1);
    let mut i = 1usize;
    if tokens
        .get(i)
        .is_some_and(|c| c.fills_role_with(DcRole::BeginTypeDef, resolver))
    {
        let mut depth = 1usize;
        i = i.saturating_add(1);
        while i < end && depth > 0 {
            let tok = tokens
                .get(i)
                .copied()
                .context("String literal type header out of bounds")?;
            if tok.fills_role_with(DcRole::Escape, resolver) {
                i = i.saturating_add(2);
                continue;
            }
            if tok.fills_role_with(DcRole::BeginTypeDef, resolver) {
                depth = depth.saturating_add(1);
            } else if tok.fills_role_with(DcRole::EndTypeDef, resolver) {
                depth = depth.saturating_sub(1);
            }
            i = i.saturating_add(1);
        }
        ensure!(depth == 0, "Unclosed Dc 262..Dc 263 type header in string literal");
    } else if tokens
        .get(i)
        .is_some_and(|c| c.fills_role_with(DcRole::TypeString, resolver))
    {
        i = i.saturating_add(1);
    }

    let mut out = DcMstring::new();
    while i < end {
        let tok = tokens
            .get(i)
            .copied()
            .context("String literal payload index out of bounds")?;
        if tok.fills_role_with(DcRole::Escape, resolver) {
            let nx = tokens
                .get(i.saturating_add(1))
                .copied()
                .context("Dangling Dc 255 escape at end of string literal")?;
            ensure!(
                i.saturating_add(1) < end,
                "Unclosed string literal after trailing Dc 255 escape"
            );
            out.push_char(nx);
            i = i.saturating_add(2);
        } else {
            out.push_char(tok);
            i = i.saturating_add(1);
        }
    }
    Ok(out)
}

fn is_uncompiled_code_token(
    tok: DcChar,
    resolver: &dyn SyntaxRuleResolver,
) -> bool {
    let gid = tok.to_u128();
    tok.fills_role_with(DcRole::BeginModule, resolver)
        || tok.fills_role_with(DcRole::BeginStagerModule, resolver)
        || tok.fills_role_with(DcRole::BeginRoutineDecl, resolver)
        || tok.fills_role_with(DcRole::BeginRoutine, resolver)
        || tok.fills_role_with(DcRole::DeclareLet, resolver)
        || tok.fills_role_with(DcRole::DeclareConst, resolver)
        || tok.fills_role_with(DcRole::DeclarePub, resolver)
        || tok.fills_role_with(DcRole::DeclareBrandType, resolver)
        || tok.fills_role_with(DcRole::BeginPositionalStructDecl, resolver)
        || tok.fills_role_with(DcRole::BeginNamedStructDecl, resolver)
        || tok.fills_role_with(DcRole::BeginEnumDecl, resolver)
        || tok.fills_role_with(DcRole::BeginCodeBlock, resolver)
        || tok.fills_role_with(DcRole::BeginRoutineCall, resolver)
        || resolver.matches_script(".EM Control Flow", gid)
        || resolver.matches_script(".EM Operators", gid)
        || resolver.matches_script(".EM Builtins", gid)
}

/// Evaluates a pre-migration historical-pin declarative/prose/`lang.say` Dc document
/// slice (`body_chars`) under `resolver` and compiles its evaluated output stream into
/// a static-output `Dc 1927`..`Dc 1928` Dc-WASM program.
///
/// Invoked strictly when `run_graph_stager_dc_2608_with_quota` returns `None` because
/// `resolver`'s pinned System-graph snapshot predates `Dc 2608` having an active
/// StageR 3 decomposition.
fn eval_historical_declarative_dc_document_to_wasm(
    body_chars: &[DcChar],
    resolver: &dyn SyntaxRuleResolver,
    shared_quota: Option<crate::converters::wasm::SharedWasmTaskQuota>,
) -> Result<DcMstring> {
    let mut doc_mstr = DcMstring::new();
    for &tok in body_chars {
        doc_mstr.push_char(tok);
    }
    let strict = DcProjectionPolicy::strict();
    let mut nav = DcDocumentNavigator::from_mstr(&doc_mstr);
    let evaluator: Box<dyn DcExpressionEvaluator> = match shared_quota {
        Some(q) => Box::new(
            SharedQuotaDcExpressionEvaluator::with_shared_task_quota(q),
        ),
        None => Box::new(DefaultDcExpressionEvaluator),
    };

    let mut out_chars: Vec<DcChar> = Vec::with_capacity(body_chars.len());
    let mut doc_depth = 0usize;

    loop {
        if let Some(spliced) = nav.pop_spliced_token() {
            out_chars.push(spliced);
            continue;
        }
        if nav.cursor >= nav.active_span.end {
            break;
        }
        let Some((ch, next_offset)) = nav.read_token_or_binary_at(nav.cursor)?
        else {
            break;
        };
        if resolver.is_splice_trigger_token(ch.to_u128())
            && nav.splice_stream_invocation(evaluator.as_ref(), resolver, &strict)?
        {
            continue;
        }
        nav.cursor = next_offset;

        if ch.fills_role_with(DcRole::Escape, resolver)
            || ch.fills_role_with(DcRole::QuoteFollowingDcAsAtom, resolver)
        {
            let Some((nx, nx_after)) = nav.read_token_or_binary_at(nav.cursor)?
            else {
                bail!("Dangling escape token in Dc document");
            };
            nav.cursor = nx_after;
            if ch.fills_role_with(DcRole::QuoteFollowingDcAsAtom, resolver)
                && nx.fills_role_with(DcRole::Escape, resolver)
            {
                let Some((nx2, nx2_after)) =
                    nav.read_token_or_binary_at(nav.cursor)?
                else {
                    bail!("Dangling Dc 255 after Dc 2239 in Dc document");
                };
                nav.cursor = nx2_after;
                out_chars.push(nx2);
            } else {
                out_chars.push(nx);
            }
            continue;
        }

        if ch.fills_role_with(DcRole::BeginDocument, resolver) {
            doc_depth = doc_depth.saturating_add(1);
            continue;
        }
        if ch.fills_role_with(DcRole::EndDocument, resolver) {
            ensure!(
                doc_depth > 0,
                "Unexpected Dc 236 (End document) without matching Dc 235"
            );
            doc_depth = doc_depth.saturating_sub(1);
            continue;
        }

        out_chars.push(ch);
    }

    ensure!(
        doc_depth == 0,
        "Unclosed Dc 235 (Begin document) container in Dc document"
    );

    build_static_output_dc_wasm_program(&out_chars, resolver)
}

fn build_static_output_dc_wasm_program(
    out_chars: &[DcChar],
    resolver: &dyn SyntaxRuleResolver,
) -> Result<DcMstring> {
    use std::fmt::Write as _;

    if out_chars.is_empty() {
        return crate::converters::wat::wat_to_dcmstring_with_resolver(
            "(module (func (export \"step\") (result i32) (i32.const 0)))",
            resolver,
        );
    }

    let mut out_mstr = DcMstring::new();
    {
        let mut writer = out_mstr.writer();
        for &ch in out_chars {
            writer.push_char(ch)?;
        }
    }
    let out_bytes = out_mstr
        .as_bytes()
        .get(DCMM_HEADER_LEN..)
        .context("DcMstring missing header prefix")?;
    let out_len = out_bytes.len();
    let mut escaped_wat_bytes = String::with_capacity(out_bytes.len().saturating_mul(3));
    for &b in out_bytes {
        let _ = write!(escaped_wat_bytes, "\\{b:02x}");
    }
    let wat_src = format!(
        "(module (memory 1) (data (i32.const 32) \"{escaped_wat_bytes}\") (func (export \"step\") (result i32) (i32.store (i32.const 8) (i32.const 32)) (i32.store (i32.const 12) (i32.const {out_len})) (i32.const 0)))"
    );
    crate::converters::wat::wat_to_dcmstring_with_resolver(&wat_src, resolver)
}

/// Applies a System-graph `.Formats:transformation` converter (`format_id`, such as
/// `FormatId::StageL3ToWasm` `f1680` or `FormatId::StageR3ToStageL3` `f1681`) to `doc`.
///
/// Strips optional leading comment frames and honors an optional leading
/// `pin_description` (`Dc 641 "Global graph system block most recent node ID at time of document creation"`).
pub(crate) fn apply_graph_dc_transformation_format(
    doc: &DcMstr,
    format_id: ctb_utilities::format_id::FormatId,
) -> Result<DcMstring> {
    use crate::converters::wasm::{DcWasmResourceLimits, SharedWasmTaskQuota};
    use ctb_storage_minimal::shorthand::DcShorthand;

    let fmt_dc_id = format_id.dc_id().with_context(|| {
        format!("Format {format_id:?} has no assigned Format Dc ID")
    })?;
    let fmt_sh = DcShorthand::from_global_id(fmt_dc_id);

    let base_resolver = default_syntax_resolver();
    let raw_chars: Vec<DcChar> = doc.to_dc_string()?.chars().collect();
    let mut body_chars =
        strip_leading_dc_comment_frames(&raw_chars, base_resolver.as_ref())?;
    ensure!(
        !body_chars.is_empty(),
        "Input document for transformation {} ({}) contains no tokens",
        format_id.ident(),
        format_id.shorthand()
    );

    let mut pinned_holder: Option<
        Arc<ctb_formats_dcdata::syntax::DatasetRuleResolver>,
    > = None;
    if body_chars.first().is_some_and(|c| {
        c.fills_role_with(
            DcRole::GlobalGraphSystemBlockMostRecentNodeId,
            base_resolver.as_ref(),
        )
    }) {
        let mut nav_mstr = DcMstring::new();
        for &tok in body_chars {
            nav_mstr.push_char(tok);
        }
        let mut nav = DcDocumentNavigator::from_mstr(&nav_mstr);
        let start_pos = nav.position();
        let mut ctx = SyntaxEvalContext::new(base_resolver.as_ref(), None);
        let matched_pin = nav.eval_term_single(
            &SyntaxTerm::NamedConstruct {
                name: "pin_description".to_owned(),
                subtype: None,
                capture_var: None,
            },
            &mut ctx,
        )?;
        ensure!(
            matched_pin && !ctx.has_errors && nav.position() > start_pos,
            "Invalid leading [pin_description] in transformation input document"
        );
        let pin_end = nav.position();
        let pin_u128s =
            nav.read_tokens_in_span(DcByteSpan::new(start_pos, pin_end))?;
        let pin_tokens: Vec<DcChar> =
            pin_u128s.into_iter().map(DcChar::from_u128).collect();
        let pin_id = extract_pin_node_id_from_pin_description(
            &pin_tokens,
            base_resolver.as_ref(),
            None,
        )?;
        pinned_holder = Some(resolve_pinned_syntax_resolver(
            base_resolver.as_ref(),
            pin_id,
        )?);
        // Reason for fallback: empty slice after pin_tokens is rejected immediately below.
        let after_pin = body_chars.get(pin_tokens.len()..).unwrap_or(&[]);
        body_chars = strip_leading_dc_comment_frames(
            after_pin,
            base_resolver.as_ref(),
        )?;
        ensure!(
            !body_chars.is_empty(),
            "Transformation input document contains only a pin_description and no body"
        );
    }

    let active_resolver: &dyn SyntaxRuleResolver =
        if let Some(ref pinned) = pinned_holder {
            pinned.as_ref()
        } else {
            base_resolver.as_ref()
        };
    let sg = active_resolver.seed_graph().context(
        "Active syntax resolver does not expose a SeedGraphIndex snapshot",
    )?;

    let steps = sg.list_all_active_converter_steps()?;
    let step = steps
        .into_iter()
        .find(|s| s.subject == fmt_sh)
        .with_context(|| {
            format!(
                "No active converter implementation found in System graph snapshot for transformation {} ({})",
                format_id.ident(),
                format_id.shorthand()
            )
        })?;

    let first_body = body_chars
        .first()
        .copied()
        .context("Empty transformation body")?;
    let first_sh = DcShorthand::from_global_id(first_body.to_u128());
    ensure!(
        first_sh == step.src,
        "Transformation {} ({}) expects input starting with {}, found {}",
        format_id.ident(),
        format_id.shorthand(),
        step.src,
        first_sh
    );

    let host_quota = SharedWasmTaskQuota::new(DcWasmResourceLimits {
        max_instructions_per_step: 500_000_000,
        max_memory_pages: 512,
        max_step_output_bytes: 16 * 1024 * 1024,
        max_session_output_bytes: 32 * 1024 * 1024,
        ..DcWasmResourceLimits::default()
    });
    let raw_out =
        crate::converters::wasm::run_active_converter_step_on_dc_slice_with_quota(
            &step,
            body_chars,
            sg,
            active_resolver,
            Some(host_quota),
        )?;

    let wasm_dst_sh = DcShorthand::from_global_id(
        ctb_formats_dcdata::dc::DC_BEGIN_WASM_PROGRAM.to_u128(),
    );
    if step.dst == wasm_dst_sh {
        let out_chars: Vec<DcChar> = raw_out.to_dc_string()?.chars().collect();
        let out_shorthands: Vec<DcShorthand> = out_chars
            .iter()
            .map(|c| DcShorthand::from_global_id(c.to_u128()))
            .collect();
        return crate::converters::wasm::expand_graph_wasm_shorthands_to_dcmstring(
            &out_shorthands,
            active_resolver,
        );
    }

    Ok(raw_out)
}

fn tokens_satisfy_value_requirement(
    tokens: &[DcChar],
    requirement: &ValueRequirement<'_>,
    resolver: &dyn SyntaxRuleResolver,
    shared_quota: Option<crate::converters::wasm::SharedWasmTaskQuota>,
) -> Result<bool> {
    let mut mstr = DcMstring::new();
    for &tok in tokens {
        mstr.push_char(tok);
    }
    let mut nav = DcDocumentNavigator::from_mstr(&mstr);
    let mut ctx = SyntaxEvalContext::new(resolver, None);
    if let Some(shared) = shared_quota {
        ctx.evaluator = Some(Arc::new(
            SharedQuotaDcExpressionEvaluator::with_shared_task_quota(shared),
        ));
    }
    let target_term = match requirement {
        ValueRequirement::NamedConstruct { name, subtype } => {
            SyntaxTerm::NamedConstruct {
                name: (*name).to_owned(),
                subtype: subtype.cloned(),
                capture_var: None,
            }
        }
        ValueRequirement::TypeToken(tok) => {
            let Some(target) = CharTarget::from_u128(tok.to_u128()) else {
                return Ok(false);
            };
            SyntaxTerm::CharRef(target)
        }
        ValueRequirement::ConstructOpener(op) => {
            let Some(target) = CharTarget::from_u128(op.to_u128()) else {
                return Ok(false);
            };
            SyntaxTerm::CharRef(target)
        }
    };
    let ok = nav.eval_term_single(&target_term, &mut ctx)?;
    Ok(ok && !ctx.has_errors && nav.position() == u64::try_from(mstr.len())?)
}

fn eval_expression_for_requirement_impl(
    expr_tokens: &[DcChar],
    requirement: &ValueRequirement<'_>,
    resolver: &dyn SyntaxRuleResolver,
    shared_quota: Option<crate::converters::wasm::SharedWasmTaskQuota>,
) -> Result<Option<DcEvaluatedValue>> {
        let Some(first) = expr_tokens.first() else {
            return Ok(None);
        };
        if first.fills_role_with(DcRole::ResolveToPin, resolver) {
            let (pinned_res, operand_tokens) = split_resolve_to_pin_operands(
                expr_tokens,
                resolver,
                shared_quota.clone(),
            )?;
            if let Some(val) = eval_expression_for_requirement_impl(
                &operand_tokens,
                requirement,
                pinned_res.as_ref(),
                shared_quota.clone(),
            )? {
                return Ok(Some(val));
            }
            if tokens_satisfy_value_requirement(
                &operand_tokens,
                requirement,
                pinned_res.as_ref(),
                shared_quota,
            )? {
                let mut doc = DcMstring::new();
                for &tok in &operand_tokens {
                    doc.push_char(tok);
                }
                return Ok(Some(DcEvaluatedValue::Document(doc)));
            }
            return Ok(None);
        }
        if first.fills_role_with(DcRole::ConvertDcStringToWasm, resolver) {
            let wasm_doc = eval_convert_dc_string_to_wasm(
                expr_tokens,
                resolver,
                shared_quota,
            )?;
            return Ok(Some(DcEvaluatedValue::Document(wasm_doc)));
        }
        if first.fills_role_with(DcRole::LangSay, resolver) {
            let output = eval_stream_splice_impl(expr_tokens, resolver, shared_quota)?
                .context("lang.say did not produce an evaluated document")?;
            let mut document = DcMstring::new();
            for token in output {
                document.push_char(token);
            }
            return Ok(Some(DcEvaluatedValue::Document(document)));
        }
        if resolver.is_splice_trigger_token(first.to_u128())
            && let Some(wasm_shorthands) =
                resolver.resolve_wasm_definition_tokens(first.to_u128())
        {
            let doc =
                crate::converters::wasm::run_graph_wasm_tokens_on_dc_slice_with_quota(
                    wasm_shorthands,
                    expr_tokens,
                    resolver,
                    shared_quota,
                )?;
            return Ok(Some(DcEvaluatedValue::Document(doc)));
        }
        let is_direct_wasm = first
            .fills_role_with(DcRole::BeginWasmProgram, resolver)
            || (!first.fills_role_with(DcRole::BeginRoutine, resolver)
                && tokens_match_named_construct_with_resolver(
                    expr_tokens,
                    "Routine",
                    resolver,
                )?);
        if is_direct_wasm {
            let mut prog_mstr = DcMstring::new();
            for &token in expr_tokens {
                prog_mstr.push_char(token);
            }
            let empty_input = DcMstring::new();
            let doc =
                crate::converters::wasm::run_dc_wasm_pure_conversion_with_resolver_and_quota(
                    &prog_mstr,
                    &empty_input,
                    resolver,
                    shared_quota,
                )?;
            return Ok(Some(DcEvaluatedValue::Document(doc)));
        }
        if resolver.is_representation_set(first.to_u128()) {
            let mut mstr = DcMstring::new();
            for &tok in expr_tokens {
                mstr.push_char(tok);
            }
            let mut nav = DcDocumentNavigator::from_mstr(&mstr);
            let mut ctx = SyntaxEvalContext::new(resolver, None);
            if let Some(shared) = shared_quota {
                ctx.evaluator = Some(Arc::new(
                    SharedQuotaDcExpressionEvaluator::with_shared_task_quota(
                        shared,
                    ),
                ));
            }
            let target_term = match requirement {
                ValueRequirement::NamedConstruct { name, subtype } => {
                    SyntaxTerm::NamedConstruct {
                        name: (*name).to_owned(),
                        subtype: subtype.cloned(),
                        capture_var: None,
                    }
                }
                ValueRequirement::TypeToken(tok) => {
                    let Some(target) = CharTarget::from_u128(tok.to_u128())
                    else {
                        return Ok(None);
                    };
                    SyntaxTerm::CharRef(target)
                }
                ValueRequirement::ConstructOpener(op) => {
                    let Some(target) = CharTarget::from_u128(op.to_u128())
                    else {
                        return Ok(None);
                    };
                    SyntaxTerm::CharRef(target)
                }
            };
            let start = nav.cursor;
            let Some((_wrapper_tok, after_wrapper)) =
                nav.read_token_or_binary_at(start)?
            else {
                return Ok(None);
            };
            let Ok(mstr_len) = u64::try_from(mstr.len()) else {
                return Ok(None);
            };
            let Some((_, list_end)) = nav.enter_list_body_after_header(
                after_wrapper,
                mstr_len,
                &mut ctx,
            )?
            else {
                return Ok(None);
            };

            while nav.cursor < list_end {
                let elem_start = nav.cursor;
                let Some((tok, after_tok)) =
                    nav.read_token_or_binary_at(elem_start)?
                else {
                    break;
                };
                if tok.fills_role_with(DcRole::ListElementSeparator, resolver) {
                    nav.cursor = after_tok;
                    continue;
                }
                let mut cand_ctx = ctx.clone();
                let cand_ok =
                    nav.eval_term_single(&target_term, &mut cand_ctx)?;
                if cand_ok && !cand_ctx.has_errors && nav.cursor > elem_start {
                    let cand_span = DcByteSpan::new(elem_start, nav.cursor);
                    let cand_tokens = nav.read_tokens_in_span(cand_span)?;
                    let mut cand_mstr = DcMstring::new();
                    for t in cand_tokens {
                        cand_mstr.push_char(DcChar::from_u128(t));
                    }
                    return Ok(Some(DcEvaluatedValue::Document(cand_mstr)));
                }
                nav.cursor = elem_start;
                if let Ok(Some(stub)) =
                    nav.next_sibling_stub_with_resolver(&ctx.policy, resolver)
                {
                    if stub.outer_span.end > elem_start {
                        nav.cursor = stub.outer_span.end;
                    } else {
                        break;
                    }
                } else {
                    let val_term = SyntaxTerm::NamedConstruct {
                        name: "value".to_owned(),
                        subtype: None,
                        capture_var: None,
                    };
                    if !nav.eval_term_single(&val_term, &mut ctx)?
                        || nav.cursor <= elem_start
                    {
                        break;
                    }
                }
            }
        }
        Ok(None)
    }

fn eval_stream_splice_impl(
    call_tokens: &[DcChar],
    resolver: &dyn SyntaxRuleResolver,
    shared_quota: Option<crate::converters::wasm::SharedWasmTaskQuota>,
) -> Result<Option<Vec<DcChar>>> {
        let Some(first) = call_tokens.first() else {
            return Ok(None);
        };
        if first.fills_role_with(DcRole::ResolveToPin, resolver) {
            let (pinned_res, operand_tokens) = split_resolve_to_pin_operands(
                call_tokens,
                resolver,
                shared_quota.clone(),
            )?;
            if let Some(spliced) = eval_stream_splice_impl(
                &operand_tokens,
                pinned_res.as_ref(),
                shared_quota.clone(),
            )? {
                return Ok(Some(spliced));
            }
            if let Some(DcEvaluatedValue::Document(doc)) =
                eval_expression_for_requirement_impl(
                    &operand_tokens,
                    &ValueRequirement::NamedConstruct {
                        name: "value",
                        subtype: None,
                    },
                    pinned_res.as_ref(),
                    shared_quota,
                )?
            {
                return Ok(Some(doc.to_dc_string()?.chars().collect()));
            }
            if tokens_match_named_construct_with_resolver(
                &operand_tokens,
                "value",
                pinned_res.as_ref(),
            )? || tokens_match_named_construct_with_resolver(
                &operand_tokens,
                "atom_dc",
                pinned_res.as_ref(),
            )? {
                return Ok(Some(operand_tokens));
            }
            bail!(
                "ResolveToPin target operand did not match [value] or [atom_dc] under pinned System graph snapshot"
            );
        }
        if first.fills_role_with(DcRole::ConvertDcStringToWasm, resolver) {
            let wasm_doc = eval_convert_dc_string_to_wasm(
                call_tokens,
                resolver,
                shared_quota,
            )?;
            let dc_str = wasm_doc.to_dc_string()?;
            return Ok(Some(dc_str.chars().collect()));
        }
        let wasm_definition = resolver.resolve_wasm_definition_tokens(first.to_u128());
        if first.fills_role_with(DcRole::LangSay, resolver)
            && wasm_definition.is_none()
        {
            let parameter = call_tokens.get(1)
                .context("lang.say is missing its parameter")?;
            ensure!(parameter.fills_role_with(DcRole::BeginParameter, resolver),
                "lang.say requires a parameter container");
            let mut argument = DcMstring::new();
            for token in call_tokens.iter().skip(2) {
                argument.push_char(*token);
            }
            let policy = DcProjectionPolicy::strict();
            let mut navigator = DcDocumentNavigator::from_mstr(&argument);
            let stub = navigator.next_sibling_stub_with_resolver(&policy, resolver)?
                .context("lang.say is missing its string argument")?;
            let literal = navigator.read_span_as_mstring(stub.outer_span)?;
            ensure!(navigator.read_next_token()?.is_some_and(|token|
                token.fills_role_with(DcRole::EndParameter, resolver)),
                "lang.say requires exactly one string argument");
            ensure!(navigator.read_next_token()?.is_none(),
                "Unexpected tokens after lang.say parameter");
            let mut literal_navigator = DcDocumentNavigator::from_mstr(&literal);
            let checkpoint = literal_navigator.save_checkpoint();
            let mut context = SyntaxEvalContext::new(resolver, None);
            ensure!(literal_navigator.eval_term_single(
                &SyntaxTerm::NamedConstruct {
                    name: "string".to_owned(), subtype: None, capture_var: None,
                }, &mut context,
            )? && !context.has_errors
                && literal_navigator.position() == u64::try_from(literal.len())?,
                "lang.say requires a string literal");
            literal_navigator.restore_checkpoint(checkpoint);
            let mut arena = crate::DcSyntaxArena::new();
            literal_navigator.parse_document_into_arena_with_resolver(
                &mut arena, &policy, resolver,
            )?;
            let root = *arena.roots().first().context("Missing lang.say literal")?;
            let (_, output) = arena.decode_literal_with_resolver(root, resolver)
                .context("lang.say argument is not a literal string")?;
            let mut chars: Vec<DcChar> = output.to_dc_string()?.chars().collect();
            let line_feed = DcChar::from('\n');
            if !chars.last().is_some_and(|token|
                resolver.is_equivalent_to(token.to_u128(), line_feed.to_u128()))
            {
                chars.push(line_feed);
            }
            return Ok(Some(chars));
        }
        if let Some(wasm_shorthands) = wasm_definition {
            let doc =
                crate::converters::wasm::run_graph_wasm_tokens_on_dc_slice_with_quota(
                    wasm_shorthands,
                    call_tokens,
                    resolver,
                    shared_quota,
                )?;
            let dc_str = doc.to_dc_string()?;
            return Ok(Some(dc_str.chars().collect()));
        }
        Ok(None)
}

/// Checkpoint holding cursor and spliced token slice state for backtracking in [`DcDocumentNavigator`].
#[derive(Debug, Clone, Copy)]
pub struct NavigatorCheckpoint {
    /// Byte offset cursor in the backend stream.
    pub cursor: u64,
    /// Active span for the current container level.
    pub active_span: DcByteSpan,
    /// Start index of the active spliced token slice in `spliced_buf`.
    pub spliced_head: usize,
    /// End index (exclusive) of the active spliced token slice in `spliced_buf`.
    pub spliced_end: usize,
}

impl NavigatorCheckpoint {
    /// Returns the number of active spliced tokens in this checkpoint.
    #[must_use]
    pub const fn spliced_len(&self) -> usize {
        self.spliced_end.saturating_sub(self.spliced_head)
    }
}

/// Execution context for evaluating [`SyntaxPattern`] rules directly over a [`DcDocumentNavigator`].
#[derive(Clone)]
pub struct SyntaxEvalContext<'r> {
    /// Syntax rule and named-type resolver.
    pub resolver: &'r dyn SyntaxRuleResolver,
    /// Defining character token (`~`) when evaluating a character's own syntax rule.
    pub self_dc: Option<u128>,
    /// Current recursion depth.
    pub depth: usize,
    /// Maximum allowed recursion depth.
    pub max_depth: usize,
    /// Call stack for cycle detection.
    pub call_stack: Vec<String>,
    /// When true, matching a container via `RuleRef` recursively validates its interior children.
    pub validate_interiors: bool,
    /// Syntax diagnostics recorded during evaluation.
    pub diagnostics: Vec<SyntaxDiagnostic>,
    /// True if any framing or constraint error occurred during evaluation.
    pub has_errors: bool,
    /// Optional expression evaluator for dynamic/Scheme/Lisp-like expression resolution.
    pub evaluator: Option<Arc<dyn DcExpressionEvaluator>>,
    /// Captured integer variables bound via `-> $var` during rule evaluation.
    pub captured_numbers: std::collections::HashMap<String, malachite::Integer>,
    /// True when evaluating a subtype pattern over an already-unescaped literal payload.
    pub unescaped_literal_payload: bool,
    pub(super) policy: DcProjectionPolicy,
}

impl<'r> SyntaxEvalContext<'r> {
    /// Creates a new syntax evaluation context with default recursion bounds.
    #[must_use]
    pub fn new(
        resolver: &'r dyn SyntaxRuleResolver,
        self_dc: Option<u128>,
    ) -> Self {
        Self {
            resolver,
            self_dc,
            depth: 0,
            max_depth: MAX_SYNTAX_EXPANSION_DEPTH,
            call_stack: Vec::new(),
            validate_interiors: false,
            diagnostics: Vec::new(),
            has_errors: false,
            evaluator: None,
            captured_numbers: std::collections::HashMap::new(),
            unescaped_literal_payload: false,
            policy: DcProjectionPolicy::strict(),
        }
    }

    /// Configures the context's [`SharedQuotaDcExpressionEvaluator`] with explicit
    /// [`crate::DcWasmResourceLimits`] shared across all WASM helper evaluations in this context.
    #[must_use]
    pub fn with_wasm_resource_limits(
        mut self,
        limits: crate::DcWasmResourceLimits,
    ) -> Self {
        self.evaluator = Some(Arc::new(
            SharedQuotaDcExpressionEvaluator::with_resource_limits(limits),
        ));
        self
    }

    /// Returns true if `token` fills `role` under the context resolver.
    #[must_use]
    pub fn token_fills_role(&self, token: u128, role: DcRole) -> bool {
        self.resolver.token_fills_role(token, role)
    }

    fn save_checkpoint(&self) -> EvalContextCheckpoint {
        EvalContextCheckpoint {
            self_dc: self.self_dc,
            depth: self.depth,
            call_stack_len: self.call_stack.len(),
            diagnostics_len: self.diagnostics.len(),
            has_errors: self.has_errors,
            unescaped_literal_payload: self.unescaped_literal_payload,
            captured_numbers: (!self.captured_numbers.is_empty())
                .then(|| self.captured_numbers.clone()),
        }
    }

    fn restore_checkpoint(&mut self, saved: &EvalContextCheckpoint) {
        self.self_dc = saved.self_dc;
        self.depth = saved.depth;
        self.call_stack.truncate(saved.call_stack_len);
        self.diagnostics.truncate(saved.diagnostics_len);
        self.has_errors = saved.has_errors;
        self.unescaped_literal_payload = saved.unescaped_literal_payload;
        if let Some(ref nums) = saved.captured_numbers {
            self.captured_numbers.clone_from(nums);
        } else if !self.captured_numbers.is_empty() {
            self.captured_numbers.clear();
        }
    }
}

#[derive(Clone)]
struct EvalContextCheckpoint {
    self_dc: Option<u128>,
    depth: usize,
    call_stack_len: usize,
    diagnostics_len: usize,
    has_errors: bool,
    unescaped_literal_payload: bool,
    captured_numbers:
        Option<std::collections::HashMap<String, malachite::Integer>>,
}

enum EvalInit<'p> {
    Pattern(&'p SyntaxPattern),
    Tokens(&'p [ctb_storage_minimal::shorthand::DcShorthand]),
    Sequence(&'p [SyntaxElement]),
}

impl DcDocumentNavigator<'_> {
    /// Evaluates a [`SyntaxPattern`] starting at the navigator's current cursor.
    ///
    /// If the pattern matches, advances `self.cursor` past the matched span and
    /// returns `Ok(true)`. If the pattern does not match, rewinds `self.cursor`
    /// to its starting offset and returns `Ok(false)`.
    pub fn eval_pattern<'p, 'r: 'p>(
        &mut self,
        pattern: &'p SyntaxPattern,
        ctx: &mut SyntaxEvalContext<'r>,
    ) -> Result<bool> {
        self.run_eval_frames(EvalInit::Pattern(pattern), ctx)
    }

    /// Evaluates a `.Semantic:Pattern` `&[DcShorthand]` token slice directly from the seed graph
    /// without constructing a recursive `SyntaxPattern` AST.
    pub fn eval_pattern_tokens<'p, 'r: 'p>(
        &mut self,
        raw_tokens: &'p [ctb_storage_minimal::shorthand::DcShorthand],
        ctx: &mut SyntaxEvalContext<'r>,
    ) -> Result<bool> {
        self.run_eval_frames(EvalInit::Tokens(raw_tokens), ctx)
    }

    /// Evaluates a fixed-arity sequence of `ExactOne` operands starting at the current cursor.
    pub(super) fn eval_fixed_prefix_operands<'p, 'r: 'p>(
        &mut self,
        operands: &'p [SyntaxElement],
        ctx: &mut SyntaxEvalContext<'r>,
    ) -> Result<bool> {
        self.run_eval_frames(EvalInit::Sequence(operands), ctx)
    }

    fn run_eval_frames<'p, 'r: 'p>(
        &mut self,
        init: EvalInit<'p>,
        ctx: &mut SyntaxEvalContext<'r>,
    ) -> Result<bool> {
        use ctb_formats_dcdata::syntax::PatternTokenStep;
        use ctb_storage_minimal::shorthand::DcShorthand;

        if ctx.evaluator.is_none() {
            ctx.evaluator = Some(Arc::new(self.default_expression_evaluator()));
        }

        #[derive(Clone)]
        enum EvalStep<'p> {
            Term(Cow<'p, SyntaxTerm>),
            TokenGroup(&'p [DcShorthand]),
        }

        enum EvalBranches<'p> {
            Ast(&'p [SyntaxPattern]),
            Tokens(Vec<&'p [DcShorthand]>),
        }

        enum EvalFrame<'p> {
            Pattern(&'p SyntaxPattern),
            Tokens(&'p [DcShorthand]),
            Sequence(&'p [SyntaxElement], usize, NavigatorCheckpoint),
            TokenSequence(&'p [DcShorthand], usize, NavigatorCheckpoint),
            Alternation {
                branches: EvalBranches<'p>,
                index: usize,
                branch_start: NavigatorCheckpoint,
                saved_ctx: EvalContextCheckpoint,
                zero_match:
                    Option<(EvalContextCheckpoint, NavigatorCheckpoint)>,
            },
            Element {
                step: EvalStep<'p>,
                quantifier: Quantifier,
                count: usize,
                element_start: NavigatorCheckpoint,
                iteration_start: NavigatorCheckpoint,
            },
            Step(EvalStep<'p>),
            Restore(Option<u128>, usize, NavigatorCheckpoint),
        }

        let sg_opt = ctx.resolver.seed_graph();
        let start = self.save_checkpoint();
        let old_depth = ctx.depth;
        let old_self = ctx.self_dc;
        let first_frame = match init {
            EvalInit::Pattern(p) => EvalFrame::Pattern(p),
            EvalInit::Tokens(t) => {
                if sg_opt.is_none() {
                    return Ok(false);
                }
                EvalFrame::Tokens(t)
            }
            EvalInit::Sequence(s) => EvalFrame::Sequence(s, 0, start),
        };
        let result = (|| -> Result<bool> {
            let mut frames = vec![first_frame];
            let mut matched = true;
            while let Some(frame) = frames.pop() {
                match frame {
                    EvalFrame::Step(step) => match step {
                        EvalStep::TokenGroup(group_tokens) => {
                            let Some(sg) = sg_opt else {
                                matched = false;
                                continue;
                            };
                            let token = self.peek_next_token()?;
                            let has_wrapper = !ctx.unescaped_literal_payload
                                && !sg.pattern_tokens_start_with_self_or_charset(
                                    group_tokens,
                                )
                                && token.is_some_and(|ch| {
                                    ctx.resolver
                                        .is_special_wrapper_token(ch.to_u128())
                                });
                            if has_wrapper
                                && let Some(first_dc) = token
                            {
                                if first_dc.fills_role_with(
                                    DcRole::OptionalPresent,
                                    ctx.resolver,
                                ) && !sg.pattern_tokens_can_start_with_token(
                                    group_tokens,
                                    &mut |gid| {
                                        ctx.token_fills_role(
                                            gid,
                                            DcRole::OptionalPresent,
                                        )
                                    },
                                ) {
                                    let opt_cp = self.save_checkpoint();
                                    while self.peek_next_token()?.is_some_and(
                                        |ch| {
                                            ch.fills_role_with(
                                                DcRole::OptionalPresent,
                                                ctx.resolver,
                                            )
                                        },
                                    ) {
                                        let _ = self.pop_next_token()?;
                                    }
                                    if self
                                        .eval_pattern_tokens(group_tokens, ctx)?
                                    {
                                        matched = true;
                                        continue;
                                    }
                                    ctx.has_errors = false;
                                    self.restore_checkpoint(opt_cp);
                                } else if first_dc.fills_role_with(
                                    DcRole::OptionalAbsent,
                                    ctx.resolver,
                                ) && !sg.pattern_tokens_can_start_with_token(
                                    group_tokens,
                                    &mut |gid| {
                                        ctx.token_fills_role(
                                            gid,
                                            DcRole::OptionalAbsent,
                                        )
                                    },
                                ) {
                                    let _ = self.pop_next_token()?;
                                    matched = true;
                                    continue;
                                }
                            }
                            frames.push(EvalFrame::Tokens(group_tokens));
                        }
                        EvalStep::Term(term) => {
                            let token = self.peek_next_token()?;
                            let constructor = matches!(
                                term.as_ref(),
                                SyntaxTerm::NamedConstruct { name, subtype: None, .. } if name == "type"
                            ) && token.is_some_and(|ch| {
                                is_compound_type_starter(ch, ctx.resolver)
                            });
                            let has_term_wrapper = !ctx.unescaped_literal_payload
                                && token.is_some_and(|ch| {
                                    ctx.resolver
                                        .is_special_wrapper_token(ch.to_u128())
                                        || ctx
                                            .resolver
                                            .is_splice_trigger_token(ch.to_u128())
                                });
                            if constructor {
                                let target_opt = token.and_then(|ch| {
                                    CharTarget::from_u128(ch.to_u128())
                                });
                                let rule_tokens_opt = target_opt
                                    .as_ref()
                                    .and_then(|target| {
                                        ctx.resolver.resolve_rule_tokens(target)
                                    });
                                let rule_opt = if rule_tokens_opt.is_none() {
                                    target_opt.as_ref().and_then(|target| {
                                        ctx.resolver.resolve_rule(target)
                                    })
                                } else {
                                    None
                                };
                                if rule_tokens_opt.is_none()
                                    && rule_opt.is_none()
                                {
                                    matched = false;
                                    continue;
                                }
                                if ctx.depth >= ctx.max_depth {
                                    ctx.has_errors = true;
                                    ctx.diagnostics.push(SyntaxDiagnostic {
                                        message: format!(
                                            "Syntax expansion depth limit reached ({}) for type constructor",
                                            ctx.max_depth
                                        ),
                                        token_offset: self.cursor,
                                        is_error: true,
                                    });
                                    return Ok(false);
                                }
                                frames.push(EvalFrame::Restore(
                                    ctx.self_dc,
                                    ctx.depth,
                                    self.save_checkpoint(),
                                ));
                                ctx.depth = ctx.depth.saturating_add(1);
                                ctx.self_dc = token.map(DcChar::to_u128);
                                if let Some(rule_tokens) = rule_tokens_opt {
                                    frames.push(EvalFrame::Tokens(rule_tokens));
                                } else if let Some(rule) = rule_opt {
                                    frames.push(EvalFrame::Pattern(
                                        &rule.pattern,
                                    ));
                                }
                            } else if !has_term_wrapper
                                && let Cow::Borrowed(SyntaxTerm::Group(
                                    group_pat,
                                )) = term
                            {
                                frames.push(EvalFrame::Pattern(group_pat));
                            } else {
                                matched =
                                    self.eval_term_single(term.as_ref(), ctx)?;
                            }
                        }
                    },
                    EvalFrame::Pattern(pat) => match pat {
                        SyntaxPattern::Sequence(elements) => {
                            frames.push(EvalFrame::Sequence(
                                elements,
                                0,
                                self.save_checkpoint(),
                            ));
                            matched = true;
                        }
                        SyntaxPattern::Alternation(branches) => {
                            let Some(branch) = branches.first() else {
                                matched = false;
                                continue;
                            };
                            frames.push(EvalFrame::Alternation {
                                branches: EvalBranches::Ast(branches),
                                index: 1,
                                branch_start: self.save_checkpoint(),
                                saved_ctx: ctx.save_checkpoint(),
                                zero_match: None,
                            });
                            frames.push(EvalFrame::Pattern(branch));
                        }
                    },
                    EvalFrame::Tokens(raw_slice) => {
                        let Some(sg) = sg_opt else {
                            matched = false;
                            continue;
                        };
                        let tokens =
                            sg.strip_pattern_action_tokens(raw_slice)?;
                        if let Some(branches) =
                            sg.split_alternation_branches(tokens)?
                        {
                            let Some(&first_branch) = branches.first() else {
                                matched = false;
                                continue;
                            };
                            frames.push(EvalFrame::Alternation {
                                branches: EvalBranches::Tokens(branches),
                                index: 1,
                                branch_start: self.save_checkpoint(),
                                saved_ctx: ctx.save_checkpoint(),
                                zero_match: None,
                            });
                            frames.push(EvalFrame::Tokens(first_branch));
                        } else {
                            frames.push(EvalFrame::TokenSequence(
                                tokens,
                                0,
                                self.save_checkpoint(),
                            ));
                            matched = true;
                        }
                    }
                    EvalFrame::Sequence(elements, index, sequence_start) => {
                        if !matched {
                            self.restore_checkpoint(sequence_start);
                            continue;
                        }
                        let mut next_idx = index;
                        while let Some(elem) = elements.get(next_idx) {
                            let at_end = self.cursor >= self.active_span.end
                                && !self.has_spliced_tokens();
                            if at_end && elem.quantifier.allows_zero() {
                                next_idx = next_idx.saturating_add(1);
                                continue;
                            }
                            if at_end {
                                matched = false;
                                self.restore_checkpoint(sequence_start);
                                break;
                            }
                            let cp = self.save_checkpoint();
                            let step =
                                EvalStep::Term(Cow::Borrowed(&elem.term));
                            frames.push(EvalFrame::Sequence(
                                elements,
                                next_idx.saturating_add(1),
                                sequence_start,
                            ));
                            frames.push(EvalFrame::Element {
                                step: step.clone(),
                                quantifier: elem.quantifier,
                                count: 0,
                                element_start: cp,
                                iteration_start: cp,
                            });
                            frames.push(EvalFrame::Step(step));
                            break;
                        }
                    }
                    EvalFrame::TokenSequence(tokens, index, sequence_start) => {
                        if !matched {
                            self.restore_checkpoint(sequence_start);
                            continue;
                        }
                        let Some(sg) = sg_opt else {
                            matched = false;
                            continue;
                        };
                        let mut next_idx = index;
                        while next_idx < tokens.len() {
                            let (raw_step, quantifier, after_idx) =
                                sg.step_pattern_token_slice(tokens, next_idx)?;
                            let at_end = self.cursor >= self.active_span.end
                                && !self.has_spliced_tokens();
                            if at_end && quantifier.allows_zero() {
                                next_idx = after_idx;
                                continue;
                            }
                            if at_end {
                                matched = false;
                                self.restore_checkpoint(sequence_start);
                                break;
                            }
                            let cp = self.save_checkpoint();
                            let step = match raw_step {
                                PatternTokenStep::Group(g) => {
                                    EvalStep::TokenGroup(g)
                                }
                                PatternTokenStep::Term(t) => {
                                    EvalStep::Term(Cow::Owned(t))
                                }
                            };
                            frames.push(EvalFrame::TokenSequence(
                                tokens,
                                after_idx,
                                sequence_start,
                            ));
                            frames.push(EvalFrame::Element {
                                step: step.clone(),
                                quantifier,
                                count: 0,
                                element_start: cp,
                                iteration_start: cp,
                            });
                            frames.push(EvalFrame::Step(step));
                            break;
                        }
                    }
                    EvalFrame::Alternation {
                        branches,
                        index,
                        branch_start,
                        saved_ctx,
                        mut zero_match,
                    } => {
                        if matched && !ctx.has_errors {
                            let consumed = self.cursor > branch_start.cursor
                                || self.spliced_len()
                                    < branch_start.spliced_len();
                            if consumed {
                                continue;
                            }
                            if zero_match.is_none() {
                                zero_match = Some((
                                    ctx.save_checkpoint(),
                                    self.save_checkpoint(),
                                ));
                            }
                        }
                        self.restore_checkpoint(branch_start);
                        ctx.restore_checkpoint(&saved_ctx);
                        let next_branch = match &branches {
                            EvalBranches::Ast(b) => b.get(index).map(|pat| {
                                let need_self = b
                                    .first()
                                    .is_some_and(SyntaxPattern::starts_with_self)
                                    && !pat.starts_with_self();
                                (need_self, EvalFrame::Pattern(pat))
                            }),
                            EvalBranches::Tokens(b) => {
                                b.get(index).and_then(|&toks| {
                                    let sg = sg_opt?;
                                    let need_self = b.first().is_some_and(
                                        |first| {
                                            sg.pattern_tokens_start_with_self(
                                                first,
                                            )
                                        },
                                    ) && !sg
                                        .pattern_tokens_start_with_self(toks);
                                    Some((need_self, EvalFrame::Tokens(toks)))
                                })
                            }
                        };
                        if let Some((need_self, branch_frame)) = next_branch {
                            if need_self
                                && !self.eval_term_single(
                                    &SyntaxTerm::SelfChar,
                                    ctx,
                                )?
                            {
                                matched = false;
                                continue;
                            }
                            frames.push(EvalFrame::Alternation {
                                branches,
                                index: index.saturating_add(1),
                                branch_start,
                                saved_ctx,
                                zero_match,
                            });
                            frames.push(branch_frame);
                        } else if let Some((zm_ctx, zm_cp)) = zero_match {
                            self.restore_checkpoint(zm_cp);
                            ctx.restore_checkpoint(&zm_ctx);
                            matched = true;
                        } else {
                            matched = false;
                        }
                    }
                    EvalFrame::Element {
                        step,
                        quantifier,
                        count,
                        element_start,
                        iteration_start,
                    } => {
                        if ctx.has_errors {
                            self.restore_checkpoint(element_start);
                            return Ok(false);
                        }
                        let count = if matched {
                            count.saturating_add(1)
                        } else {
                            self.restore_checkpoint(iteration_start);
                            count
                        };
                        let progressed = self.cursor > iteration_start.cursor
                            || self.spliced_len()
                                < iteration_start.spliced_len();
                        let more_input = self.cursor < self.active_span.end
                            || self.has_spliced_tokens();
                        if matched
                            && progressed
                            && !quantifier.reached_max(count)
                            && more_input
                        {
                            let cp = self.save_checkpoint();
                            frames.push(EvalFrame::Element {
                                step: step.clone(),
                                quantifier,
                                count,
                                element_start,
                                iteration_start: cp,
                            });
                            frames.push(EvalFrame::Step(step));
                            continue;
                        }
                        matched = match quantifier {
                            Quantifier::ExactOne => count == 1,
                            Quantifier::Optional | Quantifier::ZeroOrMore => {
                                true
                            }
                            Quantifier::OneOrMore => count >= 1,
                            Quantifier::Range { min, max } => {
                                count >= min
                                    && max.is_none_or(|limit| count <= limit)
                            }
                        };
                        if !matched {
                            self.restore_checkpoint(element_start);
                        }
                    }
                    EvalFrame::Restore(self_dc, depth, rule_start) => {
                        ctx.self_dc = self_dc;
                        ctx.depth = depth;
                        if !matched {
                            self.restore_checkpoint(rule_start);
                        }
                    }
                }
            }
            Ok(matched)
        })();
        ctx.depth = old_depth;
        ctx.self_dc = old_self;
        if !matches!(result, Ok(true)) {
            self.restore_checkpoint(start);
        }
        result
    }

    /// Evaluates a single [`SyntaxTerm`] occurrence starting at the current cursor.
    pub fn eval_term_single(
        &mut self,
        term: &SyntaxTerm,
        ctx: &mut SyntaxEvalContext<'_>,
    ) -> Result<bool> {
        if ctx.evaluator.is_none() {
            ctx.evaluator = Some(Arc::new(self.default_expression_evaluator()));
        }
        let term_checkpoint = self.save_checkpoint();
        let mut pre_read = if !self.has_spliced_tokens()
            && self.cursor < self.active_span.end
        {
            self.read_token_or_binary_at(self.cursor)?
        } else {
            None
        };
        let mut active_checkpoint = term_checkpoint;
        if !ctx.unescaped_literal_payload
            && let Some((ch, _)) = pre_read
            && ctx.resolver.is_splice_trigger_token(ch.to_u128())
        {
            let ch_id = ch.to_u128();
            let term_targets_splice_self = match term {
                SyntaxTerm::SelfChar => ctx.self_dc.is_some_and(|t| {
                    t == ch_id
                        || ctx.resolver.is_equivalent_to(t, ch_id)
                }),
                SyntaxTerm::CharRef(target) => {
                    let tid = target.to_u128();
                    tid == ch_id
                        || ctx.resolver.is_equivalent_to(tid, ch_id)
                }
                SyntaxTerm::RuleRef { target } => {
                    ctx.resolver.is_equivalent_to(target.to_u128(), ch_id)
                }
                SyntaxTerm::CharSet {
                    negated: false,
                    members,
                } => members.iter().any(|target| {
                    let tid = target.to_u128();
                    tid == ch_id
                        || ctx.resolver.is_equivalent_to(tid, ch_id)
                }),
                _ => false,
            };
            if !term_targets_splice_self {
                let spliced = if let Some(evaluator) = &ctx.evaluator {
                    self.splice_stream_invocation(
                        evaluator.as_ref(),
                        ctx.resolver,
                        &ctx.policy,
                    )?
                } else {
                    let evaluator = self.default_expression_evaluator();
                    self.splice_stream_invocation(
                        &evaluator,
                        ctx.resolver,
                        &ctx.policy,
                    )?
                };
                if spliced {
                    active_checkpoint = self.save_checkpoint();
                    pre_read = if !self.has_spliced_tokens()
                        && self.cursor < self.active_span.end
                    {
                        self.read_token_or_binary_at(self.cursor)?
                    } else {
                        None
                    };
                }
            }
        }

        let from_spliced = self.has_spliced_tokens();
        let (first_dc, after_first) = if from_spliced {
            let Some(ch) = self.peek_spliced_token() else {
                return Ok(false);
            };
            (ch, self.cursor)
        } else {
            let Some((dc, after)) = pre_read else {
                return Ok(false);
            };
            (dc, after)
        };

        let start_offset = self.cursor;
        let first = first_dc.to_u128();

        if let Some(block_desc) =
            ctb_formats_dcdata::global_graph_layout::unsupported_graph_block_description(
                first,
            )
        {
            ctx.has_errors = true;
            ctx.diagnostics.push(SyntaxDiagnostic {
                message: format!(
                    "Reference to unsupported graph block at offset {start_offset}: token l{first} targets {block_desc}"
                ),
                token_offset: start_offset,
                is_error: true,
            });
            self.restore_checkpoint(term_checkpoint);
            return Ok(false);
        }

        if !ctx.unescaped_literal_payload
            && ctx.resolver.is_special_wrapper_token(first)
        {
            // 2. Optional value wrappers: Dc 315 (present) and Dc 316 (absent)
            let term_targets_opt_present = match term {
                SyntaxTerm::SelfChar | SyntaxTerm::CharSet { .. } => true,
                SyntaxTerm::Group(pat) => {
                    pat.starts_with_self_or_charset()
                        || pat.can_start_with_token(&mut |gid| {
                            ctx.token_fills_role(gid, DcRole::OptionalPresent)
                        })
                }
                SyntaxTerm::CharRef(t) | SyntaxTerm::RuleRef { target: t } => {
                    ctx.token_fills_role(t.to_u128(), DcRole::OptionalPresent)
                }
                _ => false,
            };
            if first_dc.fills_role_with(DcRole::OptionalPresent, ctx.resolver)
                && !term_targets_opt_present
            {
                let opt_checkpoint = self.save_checkpoint();
                let _ = self.pop_next_token()?;
                while let Some(next_tok) = self.peek_next_token()? {
                    if next_tok
                        .fills_role_with(DcRole::OptionalPresent, ctx.resolver)
                    {
                        let _ = self.pop_next_token()?;
                    } else {
                        break;
                    }
                }
                if self.eval_term_single(term, ctx)? {
                    return Ok(true);
                }
                ctx.has_errors = false;
                self.restore_checkpoint(opt_checkpoint);
            }
            let term_targets_opt_absent = match term {
                SyntaxTerm::SelfChar | SyntaxTerm::CharSet { .. } => true,
                SyntaxTerm::Group(pat) => {
                    pat.starts_with_self_or_charset()
                        || pat.can_start_with_token(&mut |gid| {
                            ctx.token_fills_role(gid, DcRole::OptionalAbsent)
                        })
                }
                SyntaxTerm::CharRef(t) | SyntaxTerm::RuleRef { target: t } => {
                    ctx.token_fills_role(t.to_u128(), DcRole::OptionalAbsent)
                }
                _ => false,
            };
            if first_dc.fills_role_with(DcRole::OptionalAbsent, ctx.resolver)
                && !term_targets_opt_absent
            {
                let _ = self.pop_next_token()?;
                return Ok(true);
            }

            // 1. Equivalent (Dc 397) and partially equivalent (Dc 561) representations wrapper
            let is_equiv_wrapper = ctx.resolver.is_representation_set(first);
            let term_targets_equiv = match term {
                SyntaxTerm::SelfChar | SyntaxTerm::CharSet { .. } => true,
                SyntaxTerm::Group(pat) => pat.starts_with_self_or_charset(),
                SyntaxTerm::CharRef(t) | SyntaxTerm::RuleRef { target: t } => {
                    ctx.resolver.is_representation_set(t.to_u128())
                }
                _ => false,
            };
            if is_equiv_wrapper && !term_targets_equiv {
                let equiv_checkpoint = self.save_checkpoint();
                if let Some((list_open, after_list_open)) =
                    self.read_token_or_binary_at(after_first)?
                {
                    if list_open
                        .fills_role_with(DcRole::BeginList, ctx.resolver)
                    {
                        // Reason for fallback: default closer for BeginList is DC_END_LIST.
                        let list_closer =
                            CharTarget::from_u128(list_open.to_u128())
                                .and_then(|t| {
                                    ctx.resolver.closer_for_opener(&t)
                                })
                                .map_or(
                                    ctb_formats_dcdata::dc::DC_END_LIST,
                                    DcChar::from_u128,
                                );
                        if let Ok((_, wrapper_end, _)) = self
                            .scan_balanced_container(
                                after_list_open,
                                list_open,
                                list_closer,
                                &ctx.policy,
                                ctx.resolver,
                            )
                        {
                            let old_self = ctx.self_dc;
                            ctx.self_dc = Some(first);
                            let ok = self.eval_constrained_equivalent_list(
                                after_first,
                                wrapper_end,
                                term,
                                ctx,
                            )?;
                            ctx.self_dc = old_self;
                            if ok {
                                self.cursor = wrapper_end;
                                return Ok(true);
                            }
                            ctx.has_errors = false;
                            self.restore_checkpoint(equiv_checkpoint);
                            return Ok(false);
                        }
                    }
                }
                ctx.has_errors = false;
                self.restore_checkpoint(equiv_checkpoint);
            }
        }

        match term {
            SyntaxTerm::SelfChar => {
                if ctx.self_dc.is_none_or(|self_id| {
                    first == self_id
                        || ctx.resolver.is_equivalent_to(first, self_id)
                }) {
                    let _ = self.pop_next_token()?;
                    Ok(true)
                } else {
                    self.restore_checkpoint(term_checkpoint);
                    Ok(false)
                }
            }
            SyntaxTerm::CharRef(target) => {
                if first == target.to_u128()
                    || ctx.resolver.is_equivalent_to(first, target.to_u128())
                {
                    let _ = self.pop_next_token()?;
                    Ok(true)
                } else {
                    self.restore_checkpoint(term_checkpoint);
                    Ok(false)
                }
            }
            SyntaxTerm::CharRange { start, end } => {
                if (start.to_u128()..=end.to_u128()).contains(&first) {
                    let _ = self.pop_next_token()?;
                    Ok(true)
                } else {
                    self.restore_checkpoint(term_checkpoint);
                    Ok(false)
                }
            }
            SyntaxTerm::CharSet { negated, members } => {
                let in_set = members.iter().any(|m| {
                    m.to_u128() == first
                        || ctx.resolver.is_equivalent_to(first, m.to_u128())
                });
                if *negated {
                    let is_atom_dc_rule = !ctx.unescaped_literal_payload
                        && ctx.self_dc.is_some_and(|s| {
                            ctx.token_fills_role(s, DcRole::TypeAtomDc)
                                || ctx.token_fills_role(
                                    s,
                                    DcRole::QuoteFollowingDcAsAtom,
                                )
                        });
                    if !ctx.unescaped_literal_payload
                        && !is_atom_dc_rule
                        && (first_dc
                            .fills_role_with(DcRole::Escape, ctx.resolver)
                            || first_dc.fills_role_with(
                                DcRole::QuoteFollowingDcAsAtom,
                                ctx.resolver,
                            ))
                    {
                        let _ = self.pop_next_token()?;
                        let Some(_) = self.pop_next_token()? else {
                            ctx.has_errors = true;
                            ctx.diagnostics.push(SyntaxDiagnostic {
                                message: format!("Dangling escape or atom-quote Dc at offset {start_offset}"),
                                token_offset: start_offset,
                                is_error: true,
                            });
                            self.restore_checkpoint(term_checkpoint);
                            return Ok(false);
                        };
                        return Ok(true);
                    }
                    let in_opaque = ctx.unescaped_literal_payload
                        || is_atom_dc_rule
                        || ctx.self_dc.is_some_and(|s| {
                            ctx.resolver.is_opaque_container(s)
                        });
                    let in_graph_node_def = !ctx.unescaped_literal_payload
                        && ctx.self_dc.is_some_and(|s| {
                            ctx.token_fills_role(
                                s,
                                DcRole::BeginGraphNodeDefinition,
                            )
                        });
                    if !in_opaque {
                        if !from_spliced
                            && first_dc.fills_role_with(
                                DcRole::StartEncapsulationBinary,
                                ctx.resolver,
                            )
                        {
                            if let Ok((
                                after_bin,
                                DcFrameHeaderInfo::Binary { .. },
                            )) = self.scan_binary_at(
                                start_offset,
                                &ctx.policy,
                                ctx.resolver,
                            ) {
                                self.cursor = after_bin;
                                return Ok(true);
                            }
                        }
                        if !in_graph_node_def
                            && let Some(target) = CharTarget::from_u128(first)
                            && let Some(closer_u128) =
                                ctx.resolver.closer_for_opener(&target)
                        {
                            let closer_dc = DcChar::from_u128(closer_u128);
                            if from_spliced {
                                let _ = self.pop_next_token()?;
                                let mut depth = 1usize;
                                while let Some(tok) = self.pop_next_token()? {
                                    if tok.fills_role_with(
                                        DcRole::Escape,
                                        ctx.resolver,
                                    ) || tok.fills_role_with(
                                        DcRole::QuoteFollowingDcAsAtom,
                                        ctx.resolver,
                                    ) {
                                        let _ = self.pop_next_token()?;
                                        continue;
                                    }
                                    if tok == closer_dc
                                        || ctx.resolver.is_equivalent_to(
                                            tok.to_u128(),
                                            closer_dc.to_u128(),
                                        )
                                    {
                                        depth = depth.saturating_sub(1);
                                        if depth == 0 {
                                            return Ok(true);
                                        }
                                    } else if tok == first_dc
                                        || ctx.resolver.is_equivalent_to(
                                            tok.to_u128(),
                                            first,
                                        )
                                    {
                                        depth = depth.saturating_add(1);
                                    }
                                }
                                self.restore_checkpoint(term_checkpoint);
                                return Ok(false);
                            }
                            if let Ok((_, after_closer, _)) = self
                                .scan_balanced_container(
                                    after_first,
                                    first_dc,
                                    closer_dc,
                                    &ctx.policy,
                                    ctx.resolver,
                                )
                            {
                                self.cursor = after_closer;
                                return Ok(true);
                            }
                            return Ok(false);
                        }
                    } else if !ctx.unescaped_literal_payload
                        && ctx.self_dc.is_some_and(|s| {
                            ctx.token_fills_role(s, DcRole::BeginIdentifier)
                        })
                        && is_compound_type_starter(first_dc, ctx.resolver)
                    {
                        self.restore_checkpoint(term_checkpoint);
                        return Ok(false);
                    }
                    if in_set {
                        self.restore_checkpoint(term_checkpoint);
                        return Ok(false);
                    }
                    let _ = self.pop_next_token()?;
                    Ok(true)
                } else if in_set {
                    let _ = self.pop_next_token()?;
                    Ok(true)
                } else {
                    self.restore_checkpoint(term_checkpoint);
                    Ok(false)
                }
            }
            SyntaxTerm::Group(group_pat) => self.eval_pattern(group_pat, ctx),
            SyntaxTerm::NamedConstruct {
                name,
                subtype,
                capture_var,
            } => {
                let direct_matched = if matches!(
                    name.as_str(),
                    "script" | "format" | "type"
                ) {
                    if name == "type"
                        && is_compound_type_starter(first_dc, ctx.resolver)
                        && let Some(target) = CharTarget::from_u128(first)
                        && ctx.resolver.resolve_rule(&target).is_some()
                    {
                        if subtype.is_none() {
                            self.eval_term_single(
                                &SyntaxTerm::RuleRef { target },
                                ctx,
                            )?
                        } else if first_dc
                            .fills_role_with(DcRole::BeginTypeDef, ctx.resolver)
                        {
                            let type_cp = self.save_checkpoint();
                            let matched = if self.eval_term_single(
                                &SyntaxTerm::RuleRef { target },
                                ctx,
                            )? {
                                let consumed =
                                    self.consumed_tokens_since(&type_cp)?;
                                consumed.get(1).is_some_and(|inner_dc| {
                                    ctx.resolver.token_satisfies_type(
                                        inner_dc.to_u128(),
                                        "type",
                                        subtype.as_ref(),
                                    )
                                })
                            } else {
                                false
                            };
                            if !matched {
                                self.restore_checkpoint(type_cp);
                            }
                            matched
                        } else {
                            false
                        }
                    } else if ctx.resolver.token_satisfies_type(
                        first,
                        name,
                        subtype.as_ref(),
                    ) {
                        if name == "script"
                            && let Some(target) = CharTarget::from_u128(first)
                            && ctx.resolver.resolve_rule(&target).is_some()
                        {
                            self.eval_term_single(
                                &SyntaxTerm::RuleRef { target },
                                ctx,
                            )?
                        } else {
                            let _ = self.pop_next_token()?;
                            true
                        }
                    } else {
                        false
                    }
                } else if name == "atom_dc"
                    && subtype.is_none()
                    && ctx.resolver.is_plain_atom_dc(first)
                {
                    let _ = self.pop_next_token()?;
                    true
                } else if !from_spliced
                    && first_dc.fills_role_with(
                        DcRole::StartEncapsulationBinary,
                        ctx.resolver,
                    )
                    && matches!(name.as_str(), "data" | "typed_data" | "value")
                    && let Some(raw_ok) = self.try_eval_raw_binary_construct(
                        name.as_str(),
                        subtype.as_ref(),
                        start_offset,
                        ctx,
                    )?
                {
                    raw_ok
                } else {
                    if ctx.depth >= ctx.max_depth {
                        ctx.has_errors = true;
                        ctx.diagnostics.push(SyntaxDiagnostic {
                            message: format!(
                                "Syntax expansion depth limit reached ({}) for named construct [{name}]",
                                ctx.max_depth
                            ),
                            token_offset: start_offset,
                            is_error: true,
                        });
                        return Ok(false);
                    }
                    let frame_key = format!(
                        "named:{name}@{start_offset}:{}",
                        self.spliced_len()
                    );
                    if ctx.call_stack.contains(&frame_key) {
                        return Ok(false);
                    }
                    let (lookup_name, list_len_arg) = if let Some(len_arg) =
                        name.strip_prefix("list(")
                            .and_then(|s| s.strip_suffix(')'))
                    {
                        ("list", Some(len_arg))
                    } else {
                        (name.as_str(), None)
                    };
                    let lookup_key = match lookup_name {
                        "String" => "string",
                        "Number" => "number",
                        other => other,
                    };
                    let mut matched = false;
                    let raw_tokens_opt = if subtype.is_none() {
                        ctx.resolver.resolve_named_type_tokens(lookup_key)
                    } else {
                        None
                    };
                    let resolved_pat = if raw_tokens_opt.is_some() {
                        None
                    } else if let Some(sub) = subtype.as_ref() {
                        ctx.resolver
                            .resolve_named_type_with_arg(lookup_key, Some(sub))
                            .map(Cow::Owned)
                    } else {
                        ctx.resolver
                            .resolve_named_type(lookup_key)
                            .map(Cow::Borrowed)
                            .or_else(|| {
                                ctx.resolver
                                    .resolve_named_type_with_arg(
                                        lookup_key,
                                        None,
                                    )
                                    .map(Cow::Owned)
                            })
                    };
                    if raw_tokens_opt.is_some() || resolved_pat.is_some() {
                        let skip_value_pattern = lookup_key == "value"
                            && subtype.is_none()
                            && !ctx
                                .resolver
                                .matches_named_type_starter("value", first)
                            && !first_dc.fills_role_with(
                                DcRole::EvaluableExpressionStarter,
                                ctx.resolver,
                            );
                        if !skip_value_pattern {
                            ctx.call_stack.push(frame_key);
                            ctx.depth = ctx.depth.saturating_add(1);
                            matched = if let Some(raw_tokens) = raw_tokens_opt {
                                self.eval_pattern_tokens(raw_tokens, ctx)?
                            } else if let Some(pattern) = &resolved_pat {
                                self.eval_pattern(pattern, ctx)?
                            } else {
                                false
                            };
                            ctx.depth = ctx.depth.saturating_sub(1);
                            ctx.call_stack.pop();
                        }
                        if matched {
                            matched = self.validate_matched_construct(
                                lookup_key,
                                subtype.as_ref(),
                                capture_var.as_deref(),
                                list_len_arg,
                                &active_checkpoint,
                                start_offset,
                                from_spliced,
                                ctx,
                            )?;
                        }
                        if !matched
                            && lookup_key == "value"
                            && subtype.is_none()
                        {
                            if let Some(target) = CharTarget::from_u128(first)
                                && !first_dc.fills_role_with(
                                    DcRole::BeginIdentifier,
                                    ctx.resolver,
                                )
                                && (ctx
                                    .resolver
                                    .closer_for_opener(&target)
                                    .is_some()
                                    || ctx
                                        .resolver
                                        .resolve_rule(&target)
                                        .is_some_and(|rule| {
                                            rule.pattern.starts_with_self()
                                        }))
                            {
                                let rule_term = SyntaxTerm::RuleRef { target };
                                matched =
                                    self.eval_term_single(&rule_term, ctx)?;
                            } else if Self::is_value_atom_with(
                                first_dc,
                                ctx.resolver,
                            ) || (ctx
                                .resolver
                                .matches_script(".Semantic", first)
                                && ctx.resolver.framing_shape(first)
                                    == ctb_formats_dcdata::syntax::FramingShape::Atom)
                            {
                                let _ = self.pop_next_token()?;
                                matched = true;
                            }
                        }
                        if !matched {
                            self.restore_checkpoint(term_checkpoint);
                        }
                    }
                    matched
                };
                if direct_matched {
                    return Ok(true);
                }

                if name != "value"
                    && (first_dc.fills_role_with(
                        DcRole::EvaluableExpressionStarter,
                        ctx.resolver,
                    ) || ctx.resolver.is_splice_trigger_token(first)
                        || first_dc.fills_role_with(
                            DcRole::BeginWasmProgram,
                            ctx.resolver,
                        ))
                {
                    let checkpoint = self.save_checkpoint();
                    if let Ok(Some(stub)) = self
                        .next_sibling_stub_with_resolver(
                            &ctx.policy,
                            ctx.resolver,
                        )
                    {
                        // Reason for fallback: unreadable token spans yield empty token vector.
                        let expr_tokens = self
                            .read_tokens_in_span(stub.outer_span)
                            .unwrap_or_default();
                        let expr_dc_chars: Vec<DcChar> = expr_tokens
                            .into_iter()
                            .map(DcChar::from_u128)
                            .collect();
                        let req = ValueRequirement::NamedConstruct {
                            name: name.as_str(),
                            subtype: subtype.as_ref(),
                        };

                        if first_dc.fills_role_with(
                            DcRole::ReferenceNamedObject,
                            ctx.resolver,
                        ) {
                            let id_pos = expr_dc_chars
                                .iter()
                                .enumerate()
                                .skip(1)
                                .position(|(_, ch)| {
                                    ch.fills_role_with(
                                        DcRole::BeginIdentifier,
                                        ctx.resolver,
                                    )
                                })
                                .map(|p| p.saturating_add(1));
                            let explicit_type = if let Some(pos) = id_pos {
                                if pos > 1 {
                                    expr_dc_chars.get(1..pos)
                                } else {
                                    None
                                }
                            } else {
                                None
                            };

                            if let Some(type_tokens) = explicit_type {
                                if !Self::type_tokens_satisfy(
                                    type_tokens,
                                    name,
                                    subtype.as_ref(),
                                    ctx.resolver,
                                ) {
                                    self.restore_checkpoint(checkpoint);
                                    return Ok(false);
                                }
                                match self.eval_expr_and_check_term(
                                    &expr_dc_chars,
                                    &req,
                                    term,
                                    ctx,
                                ) {
                                    Some(true) => {
                                        self.cursor = stub.outer_span.end;
                                        return Ok(true);
                                    }
                                    None if subtype.as_ref().is_none_or(|sub| {
                                        sub.as_target_str().is_some_and(|s| {
                                            Self::type_tokens_satisfy(
                                                type_tokens,
                                                s,
                                                None,
                                                ctx.resolver,
                                            )
                                        })
                                    }) =>
                                    {
                                        self.cursor = stub.outer_span.end;
                                        return Ok(true);
                                    }
                                    _ => {
                                        self.restore_checkpoint(checkpoint);
                                        return Ok(false);
                                    }
                                }
                            }
                        }

                        if self.eval_expr_and_check_term(
                            &expr_dc_chars,
                            &req,
                            term,
                            ctx,
                        ) == Some(true)
                        {
                            self.cursor = stub.outer_span.end;
                            return Ok(true);
                        }
                    }
                    self.restore_checkpoint(checkpoint);
                }
                self.restore_checkpoint(term_checkpoint);
                Ok(false)
            }
            SyntaxTerm::RuleRef { target } => {
                if ctx.token_fills_role(target.to_u128(), DcRole::TypeAtomDc)
                    && ctx.resolver.is_plain_atom_dc(first)
                {
                    let _ = self.pop_next_token()?;
                    return Ok(true);
                }
                if ctx.depth >= ctx.max_depth {
                    ctx.has_errors = true;
                    ctx.diagnostics.push(SyntaxDiagnostic {
                        message: format!(
                            "Syntax expansion depth limit reached ({}) for rule {target}",
                            ctx.max_depth
                        ),
                        token_offset: start_offset,
                        is_error: true,
                    });
                    self.restore_checkpoint(term_checkpoint);
                    return Ok(false);
                }
                let frame_key = format!(
                    "rule:{target}@{start_offset}:{}",
                    self.spliced_len()
                );
                if ctx.call_stack.contains(&frame_key) {
                    self.restore_checkpoint(term_checkpoint);
                    return Ok(false);
                }
                if let Some(rule) = ctx.resolver.resolve_rule(target) {
                    if !from_spliced
                        && target.to_u128()
                            == ctb_formats_dcdata::dc::DC_START_ENCAPSULATION_BINARY
                                .to_u128()
                        && first_dc.fills_role_with(
                            DcRole::StartEncapsulationBinary,
                            ctx.resolver,
                        )
                        && let Some((
                            end_offset,
                            DcFrameHeaderInfo::Binary { .. },
                        )) = self.try_scan_raw_binary_at(
                            start_offset,
                            &ctx.policy,
                            ctx.resolver,
                        )?
                    {
                        if ctx.validate_interiors
                            && matches!(
                                self.backend,
                                NavBackend::SeekableFile { .. }
                            )
                        {
                            self.parse_binary_arena_node(
                                DcByteSpan::new(start_offset, end_offset),
                                &ctx.policy,
                                ctx.resolver,
                            )?;
                        }
                        self.cursor = end_offset;
                        return Ok(true);
                    }
                    self.scan_cache.ensure_resolver(ctx.resolver);
                    if !from_spliced
                        && !ctx.validate_interiors
                        && let Some(&(cached_end, self_delimited)) = self
                            .scan_cache
                            .validated_rule_spans
                            .get(&(target.to_u128(), start_offset))
                        && (self_delimited
                            && cached_end <= self.active_span.end
                            || cached_end == self.active_span.end)
                    {
                        self.cursor = cached_end;
                        return Ok(true);
                    }

                    let old_self = ctx.self_dc;
                    ctx.self_dc = Some(target.to_u128());
                    ctx.call_stack.push(frame_key);
                    ctx.depth = ctx.depth.saturating_add(1);

                    let has_fmt_token = matches!(target, CharTarget::Format(_))
                        && (first == target.to_u128()
                            || ctx
                                .resolver
                                .is_equivalent_to(first, target.to_u128()));
                    if has_fmt_token && !rule.pattern.starts_with_self() {
                        let _ = self.pop_next_token()?;
                    }

                    let matched = if let Some(raw_tokens) =
                        ctx.resolver.resolve_rule_tokens(target)
                    {
                        self.eval_pattern_tokens(raw_tokens, ctx)?
                    } else {
                        self.eval_pattern(&rule.pattern, ctx)?
                    };

                    ctx.depth = ctx.depth.saturating_sub(1);
                    ctx.call_stack.pop();
                    ctx.self_dc = old_self;

                    if matched {
                        let matched_end = self.cursor;
                        if !from_spliced
                            && !ctx.has_errors
                            && !self.has_spliced_tokens()
                        {
                            if self.scan_cache.validated_rule_spans.len()
                                >= MAX_NAV_CACHE_ENTRIES
                            {
                                self.scan_cache.validated_rule_spans.clear();
                            }
                            let self_delimited = ctx
                                .resolver
                                .closer_for_opener(target)
                                .is_some()
                                || matched_end < self.active_span.end;
                            self.scan_cache.validated_rule_spans.insert(
                                (target.to_u128(), start_offset),
                                (matched_end, self_delimited),
                            );
                        }
                        if ctx.validate_interiors {
                            if let Some(closer_u128) =
                                ctx.resolver.closer_for_opener(target)
                            {
                                if !ctx.resolver.is_opaque_container(first) {
                                    let inner_end = matched_end.saturating_sub(
                                        self.char_encoded_len(
                                            DcChar::from_u128(closer_u128),
                                        ),
                                    );
                                    if self
                                        .validate_span_with_resolver(
                                            DcByteSpan::new(
                                                after_first,
                                                inner_end,
                                            ),
                                            0,
                                            &ctx.policy,
                                            ctx.resolver,
                                        )
                                        .is_err()
                                    {
                                        ctx.has_errors = true;
                                        self.restore_checkpoint(
                                            term_checkpoint,
                                        );
                                        return Ok(false);
                                    }
                                    self.cursor = matched_end;
                                }
                            }
                        }
                        return Ok(true);
                    }
                    self.restore_checkpoint(term_checkpoint);
                    return Ok(false);
                }
                if first == target.to_u128()
                    || ctx.resolver.is_equivalent_to(first, target.to_u128())
                {
                    let _ = self.pop_next_token()?;
                    Ok(true)
                } else {
                    self.restore_checkpoint(term_checkpoint);
                    Ok(false)
                }
            }
        }
    }

    pub(super) fn eval_expr_and_check_term(
        &self,
        expr_dc_chars: &[DcChar],
        req: &ValueRequirement<'_>,
        term: &SyntaxTerm,
        ctx: &SyntaxEvalContext<'_>,
    ) -> Option<bool> {
        let evaluator = ctx.evaluator.as_ref()?;
        let Ok(Some(DcEvaluatedValue::Document(doc))) = evaluator
            .eval_expression_for_requirement(expr_dc_chars, req, ctx.resolver)
        else {
            return None;
        };
        let mut sub_nav = DcDocumentNavigator::from_mstr(&doc);
        let mut sub_ctx = ctx.clone();
        Some(matches!(sub_nav.eval_term_single(term, &mut sub_ctx), Ok(true)))
    }

    pub(super) fn enter_list_body_after_header(
        &mut self,
        list_start: u64,
        list_end: u64,
        ctx: &mut SyntaxEvalContext<'_>,
    ) -> Result<Option<(u64, u64)>> {
        let Some((list_open, after_list_open)) =
            self.read_token_or_binary_at(list_start)?
        else {
            return Ok(None);
        };
        if !list_open.fills_role_with(DcRole::BeginList, ctx.resolver) {
            return Ok(None);
        }
        let list_target = CharTarget::from_u128(list_open.to_u128())
            .context("Missing list target")?;
        // Reason for fallback: default closer for BeginList is DC_END_LIST.
        let list_closer = ctx
            .resolver
            .closer_for_opener(&list_target)
            .map_or(ctb_formats_dcdata::dc::DC_END_LIST, DcChar::from_u128);
        let close_len = self.char_encoded_len(list_closer);
        if list_end < after_list_open.saturating_add(close_len) {
            return Ok(None);
        }
        let list_body_end = list_end.saturating_sub(close_len);
        self.active_span = DcByteSpan::new(after_list_open, list_body_end);
        self.cursor = after_list_open;
        let header = ctx.resolver.container_header_elements(&list_target);
        if !self.eval_fixed_prefix_operands(header, ctx)? || ctx.has_errors {
            return Ok(None);
        }
        Ok(Some((after_list_open, list_body_end)))
    }

    pub(super) fn eval_constrained_equivalent_list(
        &mut self,
        after_wrapper_tag: u64,
        wrapper_end: u64,
        term: &SyntaxTerm,
        ctx: &mut SyntaxEvalContext<'_>,
    ) -> Result<bool> {
        if ctx.depth >= ctx.max_depth {
            ctx.has_errors = true;
            ctx.diagnostics.push(SyntaxDiagnostic {
                message:
                    "Constrained representation expansion depth limit reached"
                        .to_owned(),
                token_offset: after_wrapper_tag,
                is_error: true,
            });
            return Ok(false);
        }
        let saved_span = self.active_span;
        let mut sub_ctx = ctx.clone();
        sub_ctx.depth = sub_ctx.depth.saturating_add(1);
        let Some((after_list_open, list_body_end)) = self
            .enter_list_body_after_header(
                after_wrapper_tag,
                wrapper_end,
                &mut sub_ctx,
            )?
        else {
            self.active_span = saved_span;
            return Ok(false);
        };

        let value_term = SyntaxTerm::NamedConstruct {
            name: "value".to_owned(),
            subtype: None,
            capture_var: None,
        };
        let mut matched_any = false;
        while self.cursor < list_body_end {
            let elem_start = self.cursor;
            let Some((tok, after_tok)) =
                self.read_token_or_binary_at(elem_start)?
            else {
                break;
            };
            if tok.fills_role_with(DcRole::ListElementSeparator, ctx.resolver) {
                self.cursor = after_tok;
                continue;
            }
            self.cursor = elem_start;
            self.active_span = saved_span;
            let mut cand_ctx = sub_ctx.clone();
            let cand_ok = self.eval_term_single(term, &mut cand_ctx)?;
            self.active_span = DcByteSpan::new(after_list_open, list_body_end);
            if cand_ok
                && !cand_ctx.has_errors
                && self.cursor > elem_start
                && self.cursor <= list_body_end
            {
                matched_any = true;
                continue;
            }
            self.cursor = elem_start;
            if let Some(unconstrained) = term.unconstrained() {
                self.active_span = saved_span;
                let mut trial_ctx = sub_ctx.clone();
                let trial_ok =
                    self.eval_term_single(&unconstrained, &mut trial_ctx)?;
                self.active_span =
                    DcByteSpan::new(after_list_open, list_body_end);
                if trial_ok
                    && !trial_ctx.has_errors
                    && self.cursor <= list_body_end
                {
                    self.active_span = saved_span;
                    return Ok(false);
                }
                self.cursor = elem_start;
            }
            self.active_span = saved_span;
            let mut val_ctx = sub_ctx.clone();
            let val_ok = self.eval_term_single(&value_term, &mut val_ctx)?;
            self.active_span = DcByteSpan::new(after_list_open, list_body_end);
            if val_ok
                && !val_ctx.has_errors
                && self.cursor > elem_start
                && self.cursor <= list_body_end
            {
                continue;
            }
            self.active_span = saved_span;
            return Ok(false);
        }
        self.active_span = saved_span;
        Ok(matched_any)
    }

    pub(super) fn try_eval_raw_binary_construct(
        &mut self,
        construct_name: &str,
        subtype: Option<&SubtypeConstraint>,
        start_offset: u64,
        ctx: &mut SyntaxEvalContext<'_>,
    ) -> Result<Option<bool>> {
        let Some((
            end_offset,
            DcFrameHeaderInfo::Binary {
                data_type,
                payload_len,
                ..
            },
        )) = self.try_scan_raw_binary_at(
            start_offset,
            &ctx.policy,
            ctx.resolver,
        )?
        else {
            return Ok(None);
        };
        if ctx.validate_interiors
            && matches!(self.backend, NavBackend::SeekableFile { .. })
        {
            self.parse_binary_arena_node(
                DcByteSpan::new(start_offset, end_offset),
                &ctx.policy,
                ctx.resolver,
            )?;
        }
        match construct_name {
            "value" if subtype.is_none() => {
                self.cursor = end_offset;
                Ok(Some(true))
            }
            "typed_data" => {
                let Some(ref dt) = data_type else {
                    return Ok(Some(false));
                };
                let mut sub_nav = DcDocumentNavigator::from_dc_str(dt);
                let mut sub_ctx = SyntaxEvalContext::new(ctx.resolver, None);
                sub_ctx.policy.clone_from(&ctx.policy);
                let type_term = SyntaxTerm::NamedConstruct {
                    name: "type".to_owned(),
                    subtype: subtype.cloned(),
                    capture_var: None,
                };
                if sub_nav.eval_term_single(&type_term, &mut sub_ctx)?
                    && sub_nav.position() == sub_nav.active_span().end
                {
                    self.cursor = end_offset;
                    Ok(Some(true))
                } else {
                    Ok(Some(false))
                }
            }
            "data" => {
                if let Some(sub) = subtype {
                    let Some((expected_fmt_id, expected_fmt, expected_bytes)) =
                        parse_data_subtype_constraint(sub, start_offset, ctx)
                    else {
                        return Ok(Some(false));
                    };
                    if expected_fmt.is_some() {
                        let actual_id = data_type.as_ref().and_then(|dt| {
                            let mut chars = dt.chars();
                            let first_ch = chars.next()?;
                            if first_ch.fills_role_with(
                                DcRole::BeginTypeDef,
                                ctx.resolver,
                            ) {
                                chars.next().map(DcChar::to_u128)
                            } else {
                                Some(first_ch.to_u128())
                            }
                        });
                        if !data_format_id_matches(
                            expected_fmt_id,
                            actual_id,
                            ctx.resolver,
                        ) {
                            return Ok(Some(false));
                        }
                    }
                    if let Some(req_bytes) = expected_bytes
                        && payload_len != u64::try_from(req_bytes)?
                    {
                        return Ok(Some(false));
                    }
                }
                self.cursor = end_offset;
                Ok(Some(true))
            }
            _ => Ok(None),
        }
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "Post-match constraint validation checks subtype, capture, and span parameters together"
    )]
    pub(super) fn validate_matched_construct(
        &mut self,
        construct_name: &str,
        subtype: Option<&SubtypeConstraint>,
        capture_var: Option<&str>,
        list_len_arg: Option<&str>,
        term_checkpoint: &NavigatorCheckpoint,
        start_offset: u64,
        from_spliced: bool,
        ctx: &mut SyntaxEvalContext<'_>,
    ) -> Result<bool> {
        match construct_name {
            "list" if subtype.is_some() || list_len_arg.is_some() => {
                if from_spliced {
                    let consumed =
                        self.consumed_tokens_since(term_checkpoint)?;
                    let mut mstr = DcMstring::new();
                    for ch in consumed {
                        mstr.push_char(ch.to_u128());
                    }
                    let mut sub_nav = DcDocumentNavigator::from_mstr(&mstr);
                    // Reason for fallback: length overflow yields 0 for an empty span.
                    let list_end = u64::try_from(mstr.len()).unwrap_or(0);
                    sub_nav.eval_list_constraints(
                        0,
                        list_end,
                        subtype,
                        list_len_arg,
                        ctx,
                    )
                } else {
                    let list_end = self.cursor;
                    let ok = self.eval_list_constraints(
                        start_offset,
                        list_end,
                        subtype,
                        list_len_arg,
                        ctx,
                    )?;
                    if ok {
                        self.cursor = list_end;
                    }
                    Ok(ok)
                }
            }
            "string" | "Routine" => {
                let consumed = self.consumed_tokens_since(term_checkpoint)?;
                if construct_name == "string" {
                    let Some(&type_dc) = consumed.get(2) else {
                        return Ok(false);
                    };
                    if !type_dc.fills_role_with(DcRole::TypeString, ctx.resolver)
                        && !type_dc.fills_role_with(DcRole::TypeAny, ctx.resolver)
                    {
                        return Ok(false);
                    }
                }
                let Some(sub) = subtype else {
                    return Ok(true);
                };
                if construct_name == "Routine"
                    && consumed.first().is_some_and(|ch| {
                        ch.fills_role_with(DcRole::BeginRoutine, ctx.resolver)
                    })
                {
                    return Ok(false);
                }
                if ctx.depth >= ctx.max_depth {
                    ctx.has_errors = true;
                    ctx.diagnostics.push(SyntaxDiagnostic {
                        message:
                            "String constraint expansion depth limit reached"
                                .to_owned(),
                        token_offset: start_offset,
                        is_error: true,
                    });
                    return Ok(false);
                }
                let SubtypeConstraint::Pattern(sub_pattern) = sub else {
                    ctx.has_errors = true;
                    ctx.diagnostics.push(SyntaxDiagnostic {
                        message: format!(
                            "Invalid string constraint: {}",
                            sub.format_dsl()
                        ),
                        token_offset: start_offset,
                        is_error: true,
                    });
                    return Ok(false);
                };
                let mut payload_start = 4usize;
                if consumed.get(1).is_some_and(|ch| {
                    ch.fills_role_with(DcRole::BeginTypeDef, ctx.resolver)
                }) {
                    let mut depth = 0usize;
                    for (idx, ch) in consumed.iter().enumerate().skip(1) {
                        if ch.fills_role_with(
                            DcRole::BeginTypeDef,
                            ctx.resolver,
                        ) {
                            depth = depth.saturating_add(1);
                        } else if ch
                            .fills_role_with(DcRole::EndTypeDef, ctx.resolver)
                        {
                            depth = depth.saturating_sub(1);
                            if depth == 0 {
                                payload_start = idx.saturating_add(1);
                                break;
                            }
                        }
                    }
                }
                let Some(payload_slice) = consumed
                    .get(payload_start..consumed.len().saturating_sub(1))
                else {
                    return Ok(false);
                };
                let mut decoded_payload =
                    Vec::with_capacity(payload_slice.len());
                let mut iter = payload_slice.iter().copied();
                while let Some(ch) = iter.next() {
                    if ch.fills_role_with(DcRole::Escape, ctx.resolver) {
                        if let Some(esc) = iter.next() {
                            decoded_payload.push(esc);
                        }
                    } else {
                        decoded_payload.push(ch);
                    }
                }
                let mut sub_nav =
                    DcDocumentNavigator::from_dc_str(<&DcStr>::from(""));
                sub_nav.push_spliced_tokens(decoded_payload);
                let mut sub_ctx = SyntaxEvalContext::new(ctx.resolver, None);
                sub_ctx.policy.clone_from(&ctx.policy);
                sub_ctx.depth = ctx.depth.saturating_add(1);
                sub_ctx.max_depth = ctx.max_depth;
                sub_ctx.call_stack.clone_from(&ctx.call_stack);
                sub_ctx.unescaped_literal_payload = true;
                let sub_ok =
                    sub_nav.eval_pattern(sub_pattern, &mut sub_ctx)?;
                ctx.has_errors |= sub_ctx.has_errors;
                ctx.diagnostics.extend(sub_ctx.diagnostics);
                Ok(sub_ok
                    && !sub_ctx.has_errors
                    && !sub_nav.has_spliced_tokens())
            }
            "identifier" if subtype.is_some() => {
                let Some(sub_str) =
                    subtype.and_then(SubtypeConstraint::as_target_str)
                else {
                    return Ok(false);
                };
                let consumed = self.consumed_tokens_since(term_checkpoint)?;
                let mut mstr = DcMstring::new();
                for ch in consumed {
                    mstr.push_char(ch);
                }
                let mut id_nav = DcDocumentNavigator::from_mstr(&mstr);
                let _ = id_nav.pop_next_token()?;
                let closer = ctb_formats_dcdata::dc::DC_END_IDENTIFIER;
                let mut type_ctx = SyntaxEvalContext::new(ctx.resolver, None);
                type_ctx.policy.clone_from(&ctx.policy);
                let leading_type_tokens = id_nav
                    .try_consume_optional_leading_type(closer, &mut type_ctx)?;
                if let Some(ref type_tokens) = leading_type_tokens {
                    Ok(Self::type_tokens_satisfy(
                        type_tokens,
                        sub_str,
                        None,
                        ctx.resolver,
                    ))
                } else {
                    let mut name_mstr = DcMstring::new();
                    while let Some(ch) = id_nav.pop_next_token()? {
                        if ch.fills_role_with(DcRole::Escape, ctx.resolver) {
                            if let Some(esc) = id_nav.pop_next_token()? {
                                name_mstr.push_char(esc);
                            }
                        } else if ch
                            .fills_role_with(DcRole::EndIdentifier, ctx.resolver)
                        {
                            break;
                        } else {
                            name_mstr.push_char(ch);
                        }
                    }
                    self.identifier_bound_to_type_in_scope(
                        &name_mstr,
                        start_offset,
                        sub_str,
                        ctx,
                    )
                }
            }
            "number" if subtype.is_some() || capture_var.is_some() => {
                let consumed = self.consumed_tokens_since(term_checkpoint)?;
                let Some(digits) =
                    consumed.get(2..consumed.len().saturating_sub(1))
                else {
                    return Ok(false);
                };
                let Ok(val) =
                    crate::dc_number::parse_dc_number_digits_with_resolver(
                        digits,
                        ctx.resolver,
                    )
                else {
                    return Ok(false);
                };
                if let Some(sub) = subtype
                    && !number_satisfies_subtype(&val, sub, ctx.resolver)
                {
                    return Ok(false);
                }
                if let Some(var) = capture_var {
                    ctx.captured_numbers.insert(
                        var.trim_start_matches('$').to_owned(),
                        val,
                    );
                }
                Ok(true)
            }
            "data" if subtype.is_some() => {
                let Some(sub) = subtype else {
                    return Ok(true);
                };
                let Some((expected_fmt_id, expected_fmt, expected_bytes)) =
                    parse_data_subtype_constraint(sub, start_offset, ctx)
                else {
                    return Ok(false);
                };
                let consumed = self.consumed_tokens_since(term_checkpoint)?;
                let tokens: Vec<u128> =
                    consumed.into_iter().map(DcChar::to_u128).collect();
                if expected_fmt.is_some() {
                    let actual_id = if tokens.get(1).is_some_and(|&t| {
                        ctx.token_fills_role(t, DcRole::BeginTypeDef)
                    }) {
                        tokens.get(2).copied()
                    } else {
                        tokens.get(1).copied()
                    };
                    if !data_format_id_matches(
                        expected_fmt_id,
                        actual_id,
                        ctx.resolver,
                    ) {
                        return Ok(false);
                    }
                }
                if let Some(req_bytes) = expected_bytes {
                    match encapsulated_byte_length(&tokens, ctx.resolver) {
                        Ok(len) if len == req_bytes => {}
                        Ok(_) => return Ok(false),
                        Err(err) => {
                            ctx.has_errors = true;
                            ctx.diagnostics.push(SyntaxDiagnostic {
                                message: format!(
                                    "Invalid constrained binary payload: {err}"
                                ),
                                token_offset: start_offset,
                                is_error: true,
                            });
                            return Ok(false);
                        }
                    }
                }
                Ok(true)
            }
            _ => Ok(true),
        }
    }

    pub(super) fn try_consume_optional_leading_type(
        &mut self,
        closer: DcChar,
        ctx: &mut SyntaxEvalContext<'_>,
    ) -> Result<Option<Vec<DcChar>>> {
        let Some(first_inner) = self.peek_next_token()? else {
            return Ok(None);
        };
        if first_inner.fills_role_with(DcRole::Escape, ctx.resolver)
            || first_inner == closer
            || ctx
                .resolver
                .is_equivalent_to(first_inner.to_u128(), closer.to_u128())
        {
            return Ok(None);
        }
        if is_compound_type_starter(first_inner, ctx.resolver) {
            let type_checkpoint = self.save_checkpoint();
            let mut type_ctx = ctx.clone();
            let type_term = SyntaxTerm::NamedConstruct {
                name: "type".to_owned(),
                subtype: None,
                capture_var: None,
            };
            let type_ok = self.eval_term_single(&type_term, &mut type_ctx)?;
            let type_toks = self.consumed_tokens_since(&type_checkpoint)?;
            if !type_ok || type_ctx.has_errors || type_toks.is_empty() {
                ctx.has_errors = true;
                ctx.diagnostics.push(SyntaxDiagnostic {
                    message: format!(
                        "Invalid leading [type] in identifier at offset {}",
                        type_checkpoint.cursor
                    ),
                    token_offset: type_checkpoint.cursor,
                    is_error: true,
                });
                return Ok(None);
            }
            Ok(Some(type_toks))
        } else if is_single_type_token(first_inner, ctx.resolver) {
            let _ = self.pop_next_token()?;
            Ok(Some(vec![first_inner]))
        } else {
            Ok(None)
        }
    }

    pub(super) fn identifier_bound_to_type_in_scope(
        &mut self,
        ident_name: &DcMstr,
        before_offset: u64,
        expected_type: &str,
        ctx: &SyntaxEvalContext<'_>,
    ) -> Result<bool> {
        let req = ValueRequirement::NamedConstruct {
            name: expected_type,
            subtype: None,
        };
        let term = SyntaxTerm::NamedConstruct {
            name: expected_type.to_owned(),
            subtype: None,
            capture_var: None,
        };
        if ctx.evaluator.is_some()
            && let Ok(dc_str) = ident_name.to_dc_string()
        {
            let mut expr_dcs = vec![
                ctb_formats_dcdata::dc::DC_REFERENCE_BOUND_VALUE,
                ctb_formats_dcdata::dc::DC_BEGIN_IDENTIFIER,
            ];
            for ch in dc_str.chars() {
                if ch.fills_role_with(DcRole::Escape, ctx.resolver)
                    || ch.fills_role_with(DcRole::EndIdentifier, ctx.resolver)
                {
                    expr_dcs.push(ctb_formats_dcdata::dc::DC_ESCAPE);
                }
                expr_dcs.push(ch);
            }
            expr_dcs.push(ctb_formats_dcdata::dc::DC_END_IDENTIFIER);
            if self.eval_expr_and_check_term(&expr_dcs, &req, &term, ctx)
                == Some(true)
            {
                return Ok(true);
            }
        }
        if ctx.depth > 8 {
            return Ok(false);
        }
        let doc_start = match &self.backend {
            NavBackend::MemoryMstr(_) => u64::try_from(DCMM_HEADER_LEN)?,
            NavBackend::MemoryText(_) => 0,
            NavBackend::SeekableFile { base_offset, .. } => {
                base_offset.saturating_add(u64::try_from(DCMM_HEADER_LEN)?)
            }
        };
        if before_offset <= doc_start {
            return Ok(false);
        }
        self.scan_cache.ensure_resolver(ctx.resolver);
        let saved_checkpoint = self.save_checkpoint();
        let saved_diag_len = self.diagnostics.len();
        let quiet = DcProjectionPolicy::forgiving_quiet();
        // Reason for fallback: when no prior assignment scan offset is cached, start scanning from doc_start.
        let scan_from = self
            .scan_cache
            .assignments_scanned_up_to
            .filter(|&pos| pos >= doc_start)
            .unwrap_or(doc_start);
        if scan_from < before_offset {
            self.active_span = DcByteSpan::new(scan_from, before_offset);
            self.cursor = scan_from;
            self.clear_spliced_tokens();
            while self.cursor < before_offset {
                let prev_cursor = self.cursor;
                let Ok(Some(stub)) =
                    self.next_sibling_stub_with_resolver(&quiet, ctx.resolver)
                else {
                    break;
                };
                if self.cursor <= prev_cursor {
                    break;
                }
                if ctx.resolver.is_assignment_construct(stub.head_dc.to_u128())
                {
                    let mut temp_arena = DcSyntaxArena::new();
                    if let Ok(root_id) = self
                        .build_arena_node_from_stub_with_resolver(
                            &stub,
                            &mut temp_arena,
                            &quiet,
                            ctx.resolver,
                        )
                        && let Some((ident, value)) = temp_arena
                            .decode_assignment_with_resolver(
                                root_id,
                                ctx.resolver,
                            )
                        && let Some(val_node) = temp_arena.get(value)
                    {
                        if self.scan_cache.top_level_assignments.len()
                            >= MAX_NAV_CACHE_ENTRIES
                        {
                            self.scan_cache.top_level_assignments.clear();
                        }
                        self.scan_cache.top_level_assignments.push((
                            stub.outer_span.end,
                            ident,
                            val_node.span,
                        ));
                    }
                }
            }
            self.scan_cache.assignments_scanned_up_to = Some(self.cursor);
            self.diagnostics.truncate(saved_diag_len);
            self.restore_checkpoint(saved_checkpoint);
        }
        let bound_value_span = self
            .scan_cache
            .top_level_assignments
            .iter()
            .rev()
            .find(|(end_pos, ident, _)| {
                *end_pos <= before_offset && ident.as_mstr() == ident_name
            })
            .map(|(_, _, span)| *span);
        let Some(val_span) = bound_value_span else {
            return Ok(false);
        };
        let saved_for_eval = self.save_checkpoint();
        self.active_span = val_span;
        self.cursor = val_span.start;
        self.clear_spliced_tokens();
        let mut sub_ctx = ctx.clone();
        sub_ctx.depth = sub_ctx.depth.saturating_add(1);
        // Reason for fallback: term evaluation errors or mismatches evaluate to non-match (false).
        let ok = self.eval_term_single(&term, &mut sub_ctx).unwrap_or(false)
            && !sub_ctx.has_errors
            && self.cursor == val_span.end;
        self.diagnostics.truncate(saved_diag_len);
        self.restore_checkpoint(saved_for_eval);
        Ok(ok)
    }

    pub(super) fn eval_list_constraints(
        &mut self,
        list_start: u64,
        list_end: u64,
        subtype: Option<&SubtypeConstraint>,
        len_arg: Option<&str>,
        ctx: &mut SyntaxEvalContext<'_>,
    ) -> Result<bool> {
        let expected_len: Option<malachite::Integer> =
            if let Some(raw_len) = len_arg {
                let trimmed = raw_len.trim();
                let val = if let Some(var_name) = trimmed.strip_prefix('$') {
                    ctx.captured_numbers.get(var_name).cloned()
                } else {
                    parse_bound(trimmed)
                };
                let Some(val) = val else {
                    return Ok(false);
                };
                if val < 0 {
                    return Ok(false);
                }
                Some(val)
            } else {
                None
            };

        let Some(elem_term) = ctx.resolver.resolve_list_element_term(subtype)
        else {
            return Ok(false);
        };

        let saved_span = self.active_span;
        let mut sub_ctx = ctx.clone();
        sub_ctx.captured_numbers.remove("length");
        let Some((_, list_body_end)) = self.enter_list_body_after_header(
            list_start,
            list_end,
            &mut sub_ctx,
        )?
        else {
            self.active_span = saved_span;
            return Ok(false);
        };
        let header_count = sub_ctx.captured_numbers.remove("length");

        // 3. Elements separated by Dc 312
        let mut elem_count: usize = 0;
        let mut expect_elem = false;
        while self.cursor < list_body_end {
            let elem_start = self.cursor;
            if !self.eval_term_single(&elem_term, &mut sub_ctx)?
                || sub_ctx.has_errors
                || self.cursor <= elem_start
            {
                self.active_span = saved_span;
                return Ok(false);
            }
            elem_count = elem_count.saturating_add(1);
            expect_elem = false;
            if self.cursor < list_body_end {
                let Some((sep_tok, after_sep)) =
                    self.read_token_or_binary_at(self.cursor)?
                else {
                    self.active_span = saved_span;
                    return Ok(false);
                };
                if sep_tok
                    .fills_role_with(DcRole::ListElementSeparator, ctx.resolver)
                {
                    self.cursor = after_sep;
                    expect_elem = true;
                } else if !matches!(elem_term, SyntaxTerm::Group(_)) {
                    self.active_span = saved_span;
                    return Ok(false);
                }
            }
        }
        self.active_span = saved_span;
        if expect_elem {
            return Ok(false);
        }
        let total_count = malachite::Integer::from(elem_count);
        if let Some(hdr) = header_count {
            if hdr != total_count {
                return Ok(false);
            }
        }
        if let Some(exp) = expected_len {
            if exp != total_count {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Strictly validates all constructs in the navigator's active span against
    /// the Document Character syntax rules and named-type definitions.
    pub fn validate_document_strict(&mut self) -> Result<()> {
        self.validate_document(&DcProjectionPolicy::strict())
    }

    /// Validates the active span with the same grammar and resource policy used
    /// by arena parsing, without retaining document nodes.
    pub fn validate_document(
        &mut self,
        policy: &DcProjectionPolicy,
    ) -> Result<()> {
        self.ensure_root_pin_or_fallback(policy, None)?;
        let resolver = self.effective_resolver();
        self.validate_document_with_resolver(policy, resolver.as_ref())
    }

    /// Validates the active span with the same grammar and resource policy used
    /// by arena parsing and the provided resolver, without retaining document nodes.
    pub fn validate_document_with_resolver(
        &mut self,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<()> {
        self.ensure_root_pin_or_fallback(policy, Some(resolver))?;
        self.validate_span_with_resolver(self.active_span, 0, policy, resolver)
    }

    pub(super) fn check_container_grammar_with_resolver(
        &mut self,
        target: &CharTarget,
        outer: DcByteSpan,
        inner: DcByteSpan,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<()> {
        self.scan_cache.ensure_resolver(resolver);
        if !self.has_spliced_tokens()
            && self
                .scan_cache
                .validated_containers
                .contains(&(outer.start, outer.end))
        {
            return Ok(());
        }
        let diag_before = self.diagnostics.len();
        let saved_checkpoint = self.save_checkpoint();
        let result = (|| -> Result<()> {
            self.active_span = outer;
            self.cursor = outer.start;
            self.clear_spliced_tokens();
            if let Some(rule) = resolver.resolve_rule(target) {
                let mut ctx =
                    SyntaxEvalContext::new(resolver, Some(target.to_u128()));
                ctx.policy.clone_from(policy);
                let matched = if self
                    .scan_cache
                    .validated_rule_spans
                    .get(&(target.to_u128(), outer.start))
                    .is_some_and(|&(end, _)| end == outer.end)
                {
                    self.cursor = outer.end;
                    true
                } else if let Some(raw_tokens) =
                    resolver.resolve_rule_tokens(target)
                {
                    self.eval_pattern_tokens(raw_tokens, &mut ctx)?
                } else {
                    self.eval_pattern(&rule.pattern, &mut ctx)?
                };
                for diag in ctx.diagnostics {
                    self.syntax_issue(policy, diag.token_offset, diag.message)?;
                }
                if !matched || ctx.has_errors || self.cursor != outer.end {
                    self.syntax_issue(
                        policy,
                        outer.start,
                        format!(
                            "Syntax rule mismatch for {target} at offset {}",
                            outer.start
                        ),
                    )?;
                } else if !self.has_spliced_tokens() {
                    if self.scan_cache.validated_rule_spans.len()
                        >= MAX_NAV_CACHE_ENTRIES
                    {
                        self.scan_cache.validated_rule_spans.clear();
                    }
                    self.scan_cache.validated_rule_spans.insert(
                        (target.to_u128(), outer.start),
                        (outer.end, true),
                    );
                }
            }
            if resolver.is_opaque_container(target.to_u128()) {
                return Ok(());
            }
            self.active_span = inner;
            self.cursor = inner.start;
            self.clear_spliced_tokens();
            let is_list =
                resolver.token_fills_role(target.to_u128(), DcRole::BeginList);
            if resolver.resolve_rule(target).is_none() || is_list {
                let header = resolver.container_header_elements(target);
                let mut declared_list_len = None;
                if !header.is_empty() {
                    let mut ctx = SyntaxEvalContext::new(
                        resolver,
                        Some(target.to_u128()),
                    );
                    ctx.policy.clone_from(policy);
                    if !self.eval_fixed_prefix_operands(header, &mut ctx)?
                        || ctx.has_errors
                    {
                        self.syntax_issue(
                            policy,
                            inner.start,
                            format!(
                                "Container header mismatch inside {target} at offset {}",
                                inner.start
                            ),
                        )?;
                    }
                    declared_list_len = ctx.captured_numbers.remove("length");
                }
                if let Some(length) = declared_list_len
                    && is_list
                {
                    let body_start = self.cursor;
                    self.check_declared_list_length_with_resolver(
                        target, inner, body_start, &length, policy, resolver,
                    )?;
                }
            }
            if resolver.token_fills_role(
                target.to_u128(),
                DcRole::BeginFormatRepresentation,
            ) {
                if let Some((format, _)) =
                    self.read_token_or_binary_at(inner.start)?
                {
                    let canon_format =
                        resolver.canonical_token(format.to_u128());
                    if let Some(format_target @ CharTarget::Format(_)) =
                        CharTarget::from_u128(canon_format)
                    {
                        if resolver.resolve_rule(&format_target).is_some() {
                            self.cursor = inner.start;
                            let mut ctx =
                                SyntaxEvalContext::new(resolver, None);
                            ctx.policy.clone_from(policy);
                            if !self.eval_term_single(
                                &SyntaxTerm::RuleRef {
                                    target: format_target.clone(),
                                },
                                &mut ctx,
                            )? || ctx.has_errors
                                || self.cursor != inner.end
                            {
                                self.syntax_issue(
                                    policy,
                                    inner.start,
                                    format!("Format representation interior does not match syntax rule for {format_target} at offset {}", inner.start),
                                )?;
                            }
                        }
                    }
                }
            }
            Ok(())
        })();
        self.restore_checkpoint(saved_checkpoint);
        if result.is_ok()
            && self.diagnostics.len() == diag_before
            && !self.has_spliced_tokens()
        {
            if self.scan_cache.validated_containers.len()
                >= MAX_NAV_CACHE_ENTRIES
            {
                self.scan_cache.validated_containers.clear();
            }
            self.scan_cache
                .validated_containers
                .insert((outer.start, outer.end));
        }
        result
    }

    pub(super) fn check_declared_list_length_with_resolver(
        &mut self,
        list_target: &CharTarget,
        inner: DcByteSpan,
        body_start: u64,
        length: &malachite::Integer,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<()> {
        let Ok(expected) = usize::try_from(length) else {
            return self.syntax_issue(
                policy,
                inner.start,
                "Invalid declared list length".to_owned(),
            );
        };
        let mut ctx =
            SyntaxEvalContext::new(resolver, Some(list_target.to_u128()));
        ctx.policy.clone_from(policy);
        // Reason for fallback: ListElementSeparator role defaults to Dc 312.
        let sep_u128 = DcRole::ListElementSeparator
            .defining_short()
            .map_or(312u128, |s| DcChar::from_short(s).to_u128());
        let operand = SyntaxTerm::CharSet {
            negated: true,
            members: vec![
                CharTarget::from_u128(sep_u128)
                    .context("Missing separator target")?,
            ],
        };
        let mut actual = 0_usize;
        let mut in_element = false;
        self.cursor = body_start;
        while self.cursor < inner.end {
            let start = self.cursor;
            if let Some((tok, after)) = self.read_token_or_binary_at(start)? {
                if resolver.token_fills_role(
                    tok.to_u128(),
                    DcRole::ListElementSeparator,
                ) {
                    in_element = false;
                    self.cursor = after;
                    continue;
                }
            }
            if !in_element {
                actual = actual.saturating_add(1);
                in_element = true;
            }
            if !self.eval_term_single(&operand, &mut ctx)?
                || self.cursor <= start
            {
                break;
            }
        }
        if actual != expected {
            self.syntax_issue(policy, inner.start, format!(
                "List length mismatch: header declared {expected} elements, found {actual}",
            ))?;
        }
        Ok(())
    }

    pub(super) fn check_equivalent_list_grammar_with_resolver(
        &mut self,
        span: DcByteSpan,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<()> {
        let saved_span = self.active_span;
        let saved_cursor = self.cursor;
        let result = (|| -> Result<()> {
            let mut ctx = SyntaxEvalContext::new(resolver, None);
            ctx.policy.clone_from(policy);
            let Some((_, body_end)) = self
                .enter_list_body_after_header(span.start, span.end, &mut ctx)?
            else {
                return self.syntax_issue(
                    policy,
                    span.start,
                    "Equivalent representations require a valid list header"
                        .to_owned(),
                );
            };
            let value = SyntaxTerm::NamedConstruct {
                name: "value".to_owned(),
                subtype: None,
                capture_var: None,
            };
            let mut previous_was_separator = true;
            while self.cursor < body_end {
                let start = self.cursor;
                let Some((head, after)) =
                    self.read_token_or_binary_at(start)?
                else {
                    break;
                };
                if head.fills_role_with(DcRole::ListElementSeparator, resolver)
                {
                    previous_was_separator = true;
                    self.cursor = after;
                    continue;
                }
                if !previous_was_separator {
                    self.syntax_issue(policy, start, "Malformed equivalent representations list: missing element separator (Dc 312)".to_owned())?;
                }
                previous_was_separator = false;
                let mut trial = ctx.clone();
                if self.eval_term_single(&value, &mut trial)?
                    && !trial.has_errors
                    && self.cursor > start
                {
                    continue;
                }
                self.cursor = start;
                if self
                    .next_sibling_stub_with_resolver(policy, resolver)?
                    .is_none()
                    || self.cursor <= start
                {
                    self.cursor = after;
                }
            }
            Ok(())
        })();
        self.active_span = saved_span;
        self.cursor = saved_cursor;
        result
    }

    pub(super) fn scan_trailing_parameter_frames_with_resolver(
        &mut self,
        mut offset: u64,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<u64> {
        while let Some((next_dc, after_open)) =
            self.read_token_or_binary_at(offset)?
        {
            if !next_dc.fills_role_with(DcRole::BeginParameter, resolver) {
                break;
            }
            // Reason for fallback: default closer for BeginParameter is DC_END_PARAMETER.
            let closer = CharTarget::from_u128(next_dc.to_u128())
                .and_then(|t| resolver.closer_for_opener(&t))
                .map_or(
                    ctb_formats_dcdata::dc::DC_END_PARAMETER,
                    DcChar::from_u128,
                );
            let (_, after_close, _) = self.scan_balanced_container(
                after_open, next_dc, closer, policy, resolver,
            )?;
            offset = after_close;
        }
        Ok(offset)
    }

    pub(super) fn validate_span_with_resolver(
        &mut self,
        initial_span: DcByteSpan,
        initial_depth: usize,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<()> {
        enum ValidateWorkItem {
            Step {
                span: DcByteSpan,
                cursor: u64,
                depth: usize,
            },
            ExitFrame,
        }

        let base_level_depth = self.level_stack.len();
        let saved_span = self.active_span;
        let saved_cursor = self.cursor;
        let mut span_stack: Vec<ValidateWorkItem> =
            vec![ValidateWorkItem::Step {
                span: initial_span,
                cursor: initial_span.start,
                depth: initial_depth,
            }];

        let res = (|| -> Result<()> {
            while let Some(item) = span_stack.pop() {
                match item {
                    ValidateWorkItem::ExitFrame => {
                        self.exit_frame()?;
                    }
                    ValidateWorkItem::Step {
                        span,
                        cursor,
                        depth,
                    } => {
                        if cursor >= span.end && !self.has_spliced_tokens() {
                            continue;
                        }
                        self.active_span = span;
                        self.cursor = cursor;
                        let spliced_before = self.spliced_len();
                        let pinned_arc = self.pinned_resolver.clone();
                        let active_res: &dyn SyntaxRuleResolver =
                            if let Some(ref r) = pinned_arc {
                                r.as_ref()
                            } else {
                                resolver
                            };
                        let Some(stub) = self.next_sibling_stub_with_resolver(
                            policy, active_res,
                        )?
                        else {
                            continue;
                        };
                        if stub.head_dc.fills_role_with(
                            DcRole::StartEncapsulationBinary,
                            active_res,
                        ) && matches!(
                            stub.header_info,
                            DcFrameHeaderInfo::Binary { .. }
                        ) {
                            self.parse_binary_arena_node(
                                stub.outer_span,
                                policy,
                                active_res,
                            )?;
                        }
                        let next_cursor = self.cursor;
                        ensure!(
                            next_cursor > cursor
                                || self.spliced_len() < spliced_before,
                            "Parser made no progress at offset {cursor}"
                        );
                        if next_cursor < span.end || self.has_spliced_tokens() {
                            span_stack.push(ValidateWorkItem::Step {
                                span,
                                cursor: next_cursor,
                                depth,
                            });
                        }
                        if let Some(inner) = stub.inner_span {
                            if !active_res
                                .is_opaque_container(stub.head_dc.to_u128())
                            {
                                if depth >= policy.max_depth {
                                    self.record_diagnostic(
                                        policy,
                                        policy.depth_limit_exceeded,
                                        stub.outer_span.start,
                                        format!(
                                            "Document nesting depth exceeds maximum limit ({}) at offset {}",
                                            policy.max_depth,
                                            stub.outer_span.start,
                                        ),
                                    )?;
                                } else if !inner.is_empty() {
                                    let post_stub_pinned =
                                        self.pinned_resolver.clone();
                                    let enter_res: &dyn SyntaxRuleResolver =
                                        if let Some(ref r) = post_stub_pinned {
                                            r.as_ref()
                                        } else {
                                            resolver
                                        };
                                    self.enter_frame_with_policy(
                                        &stub, policy, enter_res,
                                    )?;
                                    span_stack
                                        .push(ValidateWorkItem::ExitFrame);
                                    span_stack.push(ValidateWorkItem::Step {
                                        span: inner,
                                        cursor: inner.start,
                                        depth: depth.saturating_add(1),
                                    });
                                }
                            }
                        }
                    }
                }
            }
            Ok(())
        })();

        while self.level_stack.len() > base_level_depth {
            let _ = self.exit_frame();
        }
        self.active_span = saved_span;
        self.cursor = saved_cursor.min(saved_span.end);
        res
    }
}

pub(super) fn parse_data_subtype_constraint<'a>(
    sub: &'a SubtypeConstraint,
    start_offset: u64,
    ctx: &mut SyntaxEvalContext<'_>,
) -> Option<(Option<FormatId>, Option<&'a str>, Option<usize>)> {
    match sub {
        SubtypeConstraint::Data {
            format,
            format_id,
            bytes,
        } => Some((*format_id, format.as_deref(), *bytes)),
        SubtypeConstraint::Target(fmt) => {
            Some((sub.resolved_format_id(), Some(fmt.as_str()), None))
        }
        other => {
            ctx.has_errors = true;
            ctx.diagnostics.push(SyntaxDiagnostic {
                message: format!(
                    "Invalid data constraint: {}",
                    other.format_dsl()
                ),
                token_offset: start_offset,
                is_error: true,
            });
            None
        }
    }
}

pub(super) fn data_format_id_matches(
    expected_fmt_id: Option<FormatId>,
    actual_id: Option<u128>,
    resolver: &dyn SyntaxRuleResolver,
) -> bool {
    matches!(
        (expected_fmt_id.and_then(|f| f.dc_id()), actual_id),
        (Some(exp), Some(act)) if resolver.is_equivalent_to(act, exp)
    )
}

pub(super) fn numeric_bound_to_integer(
    bound: ctb_formats_dcdata::syntax::NumericBoundValue,
) -> malachite::Integer {
    use ctb_formats_dcdata::syntax::NumericBoundValue;
    use std::ops::{Neg, Shl};

    match bound {
        NumericBoundValue::Unsigned(v) => malachite::Integer::from(v),
        NumericBoundValue::Negative(abs) => malachite::Integer::from(abs).neg(),
        NumericBoundValue::TwoPow128 => {
            malachite::Integer::from(1u128).shl(128u32)
        }
    }
}

pub(super) fn parse_bound(expr: &str) -> Option<malachite::Integer> {
    let bound =
        ctb_formats_dcdata::syntax::NumericBoundValue::parse_dsl_bound(expr)
            .ok()?;
    Some(numeric_bound_to_integer(bound))
}

pub(super) fn number_satisfies_subtype(
    val: &malachite::Integer,
    subtype: &SubtypeConstraint,
    resolver: &dyn SyntaxRuleResolver,
) -> bool {
    let check_target = |part: &str, pre_fmt: Option<FormatId>| -> bool {
        let fmt = pre_fmt.or_else(|| {
            FormatId::from_shorthand(part)
                .or_else(|| FormatId::from_ident(part))
        });
        if let Some(fmt) = fmt {
            if let Some(rule) = resolver.resolve_rule(&CharTarget::Format(fmt))
                && let Some(ok) = check_pattern_number_constraint(
                    &rule.pattern,
                    val,
                    resolver,
                )
            {
                ok
            } else {
                fmt.dc_id().is_some_and(|dc_id| {
                    resolver.matches_script("Formats:number", dc_id)
                        || resolver
                            .matches_script("Formats:number_abstract", dc_id)
                })
            }
        } else if let Some(rule) = resolver.resolve_named_type(part) {
            // Reason for fallback: pattern evaluation errors or mismatches evaluate to non-match (false).
            check_pattern_number_constraint(rule, val, resolver)
                .unwrap_or(false)
        } else {
            false
        }
    };

    match subtype {
        SubtypeConstraint::Number(num) => {
            if let Some(part) = &num.format
                && !check_target(part, num.format_id)
            {
                return false;
            }
            if let Some((exclusive, bound)) = num.lower_bound {
                let b = numeric_bound_to_integer(bound);
                let ok = if exclusive { val > &b } else { val >= &b };
                if !ok {
                    return false;
                }
            }
            if let Some((exclusive, bound)) = num.upper_bound {
                let b = numeric_bound_to_integer(bound);
                let ok = if exclusive { val < &b } else { val <= &b };
                if !ok {
                    return false;
                }
            }
            true
        }
        SubtypeConstraint::Target(part) => {
            check_target(part, subtype.resolved_format_id())
        }
        _ => false,
    }
}

pub(super) fn check_pattern_number_constraint(
    pattern: &SyntaxPattern,
    val: &malachite::Integer,
    resolver: &dyn SyntaxRuleResolver,
) -> Option<bool> {
    match pattern {
        SyntaxPattern::Sequence(elements) => {
            for element in elements {
                if let Some(res) =
                    check_term_number_constraint(&element.term, val, resolver)
                {
                    return Some(res);
                }
            }
            None
        }
        SyntaxPattern::Alternation(branches) => {
            let mut any_found = false;
            for branch in branches {
                if let Some(res) =
                    check_pattern_number_constraint(branch, val, resolver)
                {
                    any_found = true;
                    if res {
                        return Some(true);
                    }
                }
            }
            if any_found { Some(false) } else { None }
        }
    }
}

pub(super) fn check_term_number_constraint(
    term: &SyntaxTerm,
    val: &malachite::Integer,
    resolver: &dyn SyntaxRuleResolver,
) -> Option<bool> {
    match term {
        SyntaxTerm::NamedConstruct { name, subtype, .. }
            if name == "number" =>
        {
            if let Some(sub) = subtype {
                Some(number_satisfies_subtype(val, sub, resolver))
            } else {
                Some(true)
            }
        }
        SyntaxTerm::RuleRef { target } => {
            if let Some(rule) = resolver.resolve_rule(target) {
                check_pattern_number_constraint(&rule.pattern, val, resolver)
            } else {
                None
            }
        }
        SyntaxTerm::Group(pattern) => {
            check_pattern_number_constraint(pattern, val, resolver)
        }
        _ => None,
    }
}

pub(crate) fn is_compound_type_starter(
    tok: DcChar,
    resolver: &dyn SyntaxRuleResolver,
) -> bool {
    tok.fills_role_with(DcRole::BeginTypeDef, resolver)
        || tok.fills_role_with(DcRole::TypeIntersection, resolver)
        || tok.fills_role_with(DcRole::TypeUnion, resolver)
        || tok.fills_role_with(DcRole::FormatParamApplication, resolver)
        || tok == ctb_formats_dcdata::dc::DC_VALUE_RANGE
        || resolver.is_equivalent_to(
            tok.to_u128(),
            ctb_formats_dcdata::dc::DC_VALUE_RANGE.to_u128(),
        )
}

pub(crate) fn is_single_type_token(
    tok: DcChar,
    resolver: &dyn SyntaxRuleResolver,
) -> bool {
    resolver.token_satisfies_type(tok.to_u128(), "type", None)
}

pub(crate) fn is_identifier_leading_type_starter(
    tok: DcChar,
    resolver: &dyn SyntaxRuleResolver,
) -> bool {
    is_compound_type_starter(tok, resolver)
        || is_single_type_token(tok, resolver)
}

impl DcDocumentNavigator<'_> {
    pub(super) fn type_tokens_satisfy(
        type_tokens: &[DcChar],
        req_name: &str,
        req_subtype: Option<&SubtypeConstraint>,
        resolver: &dyn SyntaxRuleResolver,
    ) -> bool {
        let tokens = if type_tokens.len() >= 2
            && type_tokens.first().is_some_and(|c| {
                c.fills_role_with(DcRole::BeginTypeDef, resolver)
            })
            && type_tokens.last().is_some_and(|c| {
                c.fills_role_with(DcRole::EndTypeDef, resolver)
            }) {
            let end_idx = type_tokens.len().saturating_sub(1);
            // Reason for fallback: slice range out of bounds defaults to empty slice.
            type_tokens.get(1..end_idx).unwrap_or(&[])
        } else {
            type_tokens
        };

        if let [tok] = tokens {
            resolver.token_satisfies_type(tok.to_u128(), req_name, req_subtype)
        } else if let Some(&first_tok) = tokens.first()
            && first_tok.fills_role_with(DcRole::TypeRoutine, resolver)
            && resolver.token_satisfies_type(
                first_tok.to_u128(),
                req_name,
                req_subtype,
            )
        {
            true
        } else if !tokens.is_empty() && req_name == "type" {
            let mut mstr = DcMstring::new();
            for &t in tokens {
                mstr.push_char(t);
            }
            let mut nav = DcDocumentNavigator::from_mstr(&mstr);
            let mut ctx = SyntaxEvalContext::new(resolver, None);
            let type_term = SyntaxTerm::NamedConstruct {
                name: "type".to_owned(),
                subtype: req_subtype.cloned(),
                capture_var: None,
            };
            // Reason for fallback: evaluation errors treat requirement as unsatisfied (false).
            nav.eval_term_single(&type_term, &mut ctx).unwrap_or(false)
                && u64::try_from(mstr.len()).is_ok_and(|len| nav.cursor == len)
        } else {
            false
        }
    }
}

pub(super) fn encapsulated_byte_length(
    stream: &[u128],
    resolver: &dyn SyntaxRuleResolver,
) -> Result<usize> {
    let payload_end = stream
        .len()
        .checked_sub(1)
        .context("Missing binary terminator")?;
    let mut payload_start = payload_end;
    while let Some(index) = payload_start.checked_sub(1) {
        let token = *stream.get(index).context("Missing binary token")?;
        if !resolver.token_fills_role(token, DcRole::Base64Digit)
            && !resolver.token_fills_role(token, DcRole::Base64Padding)
        {
            break;
        }
        payload_start = index;
    }
    let payload = stream
        .get(payload_start..payload_end)
        .context("Invalid binary bounds")?;
    ensure!(
        payload.len().is_multiple_of(4),
        "Invalid Base64 payload length"
    );
    let digits = payload
        .iter()
        .map(|&token| {
            if let Some(digit) = resolver.base64_digit_value(token) {
                Ok(digit)
            } else if resolver.token_fills_role(token, DcRole::Base64Padding) {
                Ok(64u8)
            } else {
                bail!("Invalid Base64 token in binary encapsulation: {token}")
            }
        })
        .collect::<Result<Vec<_>>>()?;
    let padding = digits
        .iter()
        .rev()
        .take_while(|digit| **digit == 64)
        .count();
    ensure!(padding <= 2, "Invalid Base64 padding");
    let unpadded = digits
        .len()
        .checked_sub(padding)
        .context("Invalid Base64 padding")?;
    ensure!(
        digits
            .get(..unpadded)
            .context("Invalid Base64 bounds")?
            .iter()
            .all(|digit| *digit < 64),
        "Embedded Base64 padding"
    );
    if padding > 0 {
        let last = digits
            .get(unpadded.checked_sub(1).context("Missing Base64 digit")?)
            .context("Missing Base64 digit")?;
        let mask = if padding == 2 { 15 } else { 3 };
        ensure!(last & mask == 0, "Noncanonical Base64 trailing bits");
    }
    digits
        .len()
        .checked_div(4)
        .and_then(|groups| groups.checked_mul(3))
        .and_then(|bytes| bytes.checked_sub(padding))
        .context("Binary byte count overflow")
}

/// Strictly validates a [`DcMstr`] buffer against all Document Character syntax
/// rules, paired container delimiters, and constraints.
pub fn validate_dc_mixed_strict(mstr: &DcMstr) -> Result<()> {
    let mut nav = DcDocumentNavigator::from_mstr(mstr);
    nav.validate_document_strict()
}

/// Strictly validates a [`DcStr`] buffer against all Document Character syntax
/// rules, paired container delimiters, and constraints.
pub fn validate_dc_str_strict(text: &DcStr) -> Result<()> {
    let mut nav = DcDocumentNavigator::from_dc_str(text);
    nav.validate_document_strict()
}

/// Strictly validates a raw `u128` Document Character token slice against all
/// Document Character syntax rules, paired container delimiters, and constraints.
pub fn validate_dc_tokens_strict(tokens: &[u128]) -> Result<()> {
    let mut dc_str = DcString::with_capacity(tokens.len().saturating_mul(6));
    for &tok in tokens {
        dc_str.push(DcChar::from_u128(tok));
    }
    validate_dc_str_strict(&dc_str)
}
