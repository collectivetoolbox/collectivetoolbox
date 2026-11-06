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
    Arc, CachedLeafKind, CharTarget, Cow, DCMM_HEADER_LEN, DcByteSpan,
    DcCharExt, DcDocumentNavigator, DcFrameHeaderInfo, DcMstr, DcMstring,
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

    /// Evaluates a stream-splicing invocation (such as `256 lang.say(String)`) and returns
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

/// Default built-in expression evaluator supporting `256 lang.say` and equivalent representations.
#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultDcExpressionEvaluator;

impl DcExpressionEvaluator for DefaultDcExpressionEvaluator {
    fn eval_expression_for_requirement(
        &self,
        expr_tokens: &[DcChar],
        requirement: &ValueRequirement<'_>,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<Option<DcEvaluatedValue>> {
        let Some(first) = expr_tokens.first() else {
            return Ok(None);
        };
        if resolver.is_representation_set(first.to_u128()) {
            let mut mstr = DcMstring::new();
            for &tok in expr_tokens {
                mstr.push_char(tok);
            }
            let mut nav = DcDocumentNavigator::from_mstr(&mstr);
            let mut ctx = SyntaxEvalContext::new(resolver, None);
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
            let Some((list_open, after_list_open)) =
                nav.read_token_or_binary_at(after_wrapper)?
            else {
                return Ok(None);
            };
            if !list_open.fills_role_with(DcRole::BeginList, resolver) {
                return Ok(None);
            }
            // Reason for fallback: default closing delimiter for BeginList is DC_END_LIST.
            let list_closer = CharTarget::from_u128(list_open.to_u128())
                .and_then(|t| resolver.closer_for_opener(&t))
                .map_or(ctb_formats_dcdata::dc::DC_END_LIST, DcChar::from_u128);
            let close_len = nav.char_encoded_len(list_closer);
            let Ok(mstr_len) = u64::try_from(mstr.len()) else {
                return Ok(None);
            };
            let list_end = mstr_len.saturating_sub(close_len);
            if list_end < after_list_open {
                return Ok(None);
            }
            nav.active_span = DcByteSpan::new(after_list_open, list_end);
            nav.cursor = after_list_open;

            // Skip optional [number] and [type] header
            let num_term = SyntaxTerm::NamedConstruct {
                name: "number".to_owned(),
                subtype: None,
                capture_var: None,
            };
            let num_pos = nav.cursor;
            if !nav.eval_term_single(&num_term, &mut ctx)? {
                nav.cursor = num_pos;
            }
            let type_term = SyntaxTerm::NamedConstruct {
                name: "type".to_owned(),
                subtype: None,
                capture_var: None,
            };
            if !nav.eval_term_single(&type_term, &mut ctx)? {
                return Ok(None);
            }

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
                if let Ok(Some(stub)) = nav.next_sibling_stub(&ctx.policy) {
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

    fn eval_stream_splice(
        &self,
        call_tokens: &[DcChar],
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<Option<Vec<DcChar>>> {
        let Some(first) = call_tokens.first() else {
            return Ok(None);
        };
        if first.fills_role_with(DcRole::LangSay, resolver) {
            let mut in_str = false;
            let mut expect_hdr = false;
            let mut in_hdr = false;
            let mut chars = Vec::new();
            let mut iter = call_tokens.iter().skip(1).peekable();
            while let Some(&tok) = iter.next() {
                if !in_str {
                    if tok.fills_role_with(DcRole::BeginLiteral, resolver) {
                        in_str = true;
                        expect_hdr = true;
                    }
                    continue;
                }
                if expect_hdr {
                    expect_hdr = false;
                    if tok.fills_role_with(DcRole::BeginTypeDef, resolver) {
                        in_hdr = true;
                        continue;
                    }
                }
                if in_hdr {
                    if tok.fills_role_with(DcRole::EndTypeDef, resolver) {
                        in_hdr = false;
                    }
                    continue;
                }
                if tok.fills_role_with(DcRole::Escape, resolver) {
                    if let Some(&escaped) = iter.next() {
                        chars.push(escaped);
                        continue;
                    }
                }
                if tok.fills_role_with(DcRole::EndLiteral, resolver) {
                    in_str = false;
                    continue;
                }
                chars.push(tok);
            }
            return Ok(Some(chars));
        }
        Ok(None)
    }
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
    /// Returns true if this checkpoint has active spliced tokens remaining.
    #[must_use]
    pub const fn has_spliced(&self) -> bool {
        self.spliced_head < self.spliced_end
    }

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
            evaluator: Some(Arc::new(DefaultDcExpressionEvaluator)),
            captured_numbers: std::collections::HashMap::new(),
            unescaped_literal_payload: false,
            policy: DcProjectionPolicy::strict(),
        }
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
        enum Frame<'p> {
            Pattern(&'p SyntaxPattern),
            Sequence(&'p [SyntaxElement], usize, NavigatorCheckpoint),
            Alternation {
                branches: &'p [SyntaxPattern],
                index: usize,
                branch_start: NavigatorCheckpoint,
                saved_ctx: EvalContextCheckpoint,
                zero_match:
                    Option<(EvalContextCheckpoint, NavigatorCheckpoint)>,
            },
            Element(
                &'p SyntaxElement,
                usize,
                NavigatorCheckpoint,
                NavigatorCheckpoint,
            ),
            Term(&'p SyntaxTerm),
            Restore(Option<u128>, usize, NavigatorCheckpoint),
        }

        let start = self.save_checkpoint();
        let old_depth = ctx.depth;
        let old_self = ctx.self_dc;
        let result = (|| -> Result<bool> {
            let mut frames = vec![Frame::Pattern(pattern)];
            let mut matched = true;
            while let Some(frame) = frames.pop() {
                match frame {
                    Frame::Term(term) => {
                        let token = self.peek_next_token()?;
                        let constructor = matches!(
                            term,
                            SyntaxTerm::NamedConstruct { name, subtype: None, .. } if name == "type"
                        ) && token.is_some_and(|ch| {
                            ch.fills_role_with(
                                DcRole::BeginTypeDef,
                                ctx.resolver,
                            ) || ch.fills_role_with(
                                DcRole::TypeUnion,
                                ctx.resolver,
                            ) || ch.fills_role_with(
                                DcRole::TypeIntersection,
                                ctx.resolver,
                            ) || ch.fills_role_with(
                                DcRole::FormatParamApplication,
                                ctx.resolver,
                            )
                        });
                        let has_term_wrapper = !ctx.unescaped_literal_payload
                            && token.is_some_and(|ch| {
                                ctx.resolver
                                    .is_special_wrapper_token(ch.to_u128())
                            });
                        if constructor {
                            let Some(rule) = token
                                .and_then(|ch| {
                                    CharTarget::from_u128(ch.to_u128())
                                })
                                .and_then(|target| {
                                    ctx.resolver.resolve_rule(&target)
                                })
                            else {
                                matched = false;
                                continue;
                            };
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
                            frames.push(Frame::Restore(
                                ctx.self_dc,
                                ctx.depth,
                                self.save_checkpoint(),
                            ));
                            ctx.depth = ctx.depth.saturating_add(1);
                            ctx.self_dc = token.map(DcChar::to_u128);
                            frames.push(Frame::Pattern(&rule.pattern));
                        } else if !has_term_wrapper
                            && let SyntaxTerm::Group(group_pat) = term
                        {
                            frames.push(Frame::Pattern(group_pat));
                        } else {
                            matched = self.eval_term_single(term, ctx)?;
                        }
                    }
                    Frame::Pattern(pat) => match pat {
                        SyntaxPattern::Sequence(elements) => {
                            frames.push(Frame::Sequence(
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
                            frames.push(Frame::Alternation {
                                branches,
                                index: 1,
                                branch_start: self.save_checkpoint(),
                                saved_ctx: ctx.save_checkpoint(),
                                zero_match: None,
                            });
                            frames.push(Frame::Pattern(branch));
                        }
                    },
                    Frame::Sequence(elements, index, sequence_start) => {
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
                            frames.push(Frame::Sequence(
                                elements,
                                next_idx.saturating_add(1),
                                sequence_start,
                            ));
                            frames.push(Frame::Element(
                                elem,
                                0,
                                cp,
                                cp,
                            ));
                            frames.push(Frame::Term(&elem.term));
                            break;
                        }
                    }
                    Frame::Alternation {
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
                        if let Some(branch) = branches.get(index) {
                            if branches
                                .first()
                                .is_some_and(SyntaxPattern::starts_with_self)
                                && !branch.starts_with_self()
                                && !self.eval_term_single(
                                    &SyntaxTerm::SelfChar,
                                    ctx,
                                )?
                            {
                                matched = false;
                                continue;
                            }
                            frames.push(Frame::Alternation {
                                branches,
                                index: index.saturating_add(1),
                                branch_start,
                                saved_ctx,
                                zero_match,
                            });
                            frames.push(Frame::Pattern(branch));
                        } else if let Some((zm_ctx, zm_cp)) = zero_match {
                            self.restore_checkpoint(zm_cp);
                            ctx.restore_checkpoint(&zm_ctx);
                            matched = true;
                        } else {
                            matched = false;
                        }
                    }
                    Frame::Element(
                        element,
                        count,
                        element_start,
                        iteration_start,
                    ) => {
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
                            && !element.quantifier.reached_max(count)
                            && more_input
                        {
                            let cp = self.save_checkpoint();
                            frames.push(Frame::Element(
                                element,
                                count,
                                element_start,
                                cp,
                            ));
                            frames.push(Frame::Term(&element.term));
                            continue;
                        }
                        matched = match element.quantifier {
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
                    Frame::Restore(self_dc, depth, rule_start) => {
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

    /// Evaluates a quantified [`SyntaxElement`] starting at the current cursor.
    pub fn eval_element(
        &mut self,
        element: &SyntaxElement,
        ctx: &mut SyntaxEvalContext<'_>,
    ) -> Result<bool> {
        self.eval_pattern(&SyntaxPattern::Sequence(vec![element.clone()]), ctx)
    }

    /// Evaluates a fixed-arity sequence of `ExactOne` operands starting at the current cursor
    /// without allocating the general `eval_pattern` backtracking frame stack.
    pub(super) fn eval_fixed_prefix_operands(
        &mut self,
        operands: &[SyntaxElement],
        ctx: &mut SyntaxEvalContext<'_>,
    ) -> Result<bool> {
        let start = self.save_checkpoint();
        for elem in operands {
            if self.cursor >= self.active_span.end && !self.has_spliced_tokens()
            {
                self.restore_checkpoint(start);
                return Ok(false);
            }
            let matched = match &elem.term {
                SyntaxTerm::Group(group_pat) => {
                    let has_wrapper = !ctx.unescaped_literal_payload
                        && self.peek_next_token()?.is_some_and(|ch| {
                            ctx.resolver.is_special_wrapper_token(ch.to_u128())
                        });
                    if has_wrapper {
                        self.eval_term_single(&elem.term, ctx)?
                    } else {
                        self.eval_pattern(group_pat, ctx)?
                    }
                }
                SyntaxTerm::NamedConstruct {
                    name,
                    subtype: None,
                    ..
                } if name == "type" => {
                    let token = self.peek_next_token()?;
                    let is_constructor = token.is_some_and(|ch| {
                        ch.fills_role_with(DcRole::BeginTypeDef, ctx.resolver)
                            || ch.fills_role_with(
                                DcRole::TypeUnion,
                                ctx.resolver,
                            )
                            || ch.fills_role_with(
                                DcRole::TypeIntersection,
                                ctx.resolver,
                            )
                            || ch.fills_role_with(
                                DcRole::FormatParamApplication,
                                ctx.resolver,
                            )
                    });
                    if is_constructor {
                        let Some(rule) = token
                            .and_then(|ch| CharTarget::from_u128(ch.to_u128()))
                            .and_then(|target| {
                                ctx.resolver.resolve_rule(&target)
                            })
                        else {
                            self.restore_checkpoint(start);
                            return Ok(false);
                        };
                        let old_self = ctx.self_dc;
                        let old_depth = ctx.depth;
                        ctx.self_dc = token.map(DcChar::to_u128);
                        ctx.depth = ctx.depth.saturating_add(1);
                        let res = self.eval_pattern(&rule.pattern, ctx);
                        ctx.self_dc = old_self;
                        ctx.depth = old_depth;
                        res?
                    } else {
                        self.eval_term_single(&elem.term, ctx)?
                    }
                }
                term => self.eval_term_single(term, ctx)?,
            };
            if !matched || ctx.has_errors {
                self.restore_checkpoint(start);
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Evaluates a single [`SyntaxTerm`] occurrence starting at the current cursor.
    pub fn eval_term_single(
        &mut self,
        term: &SyntaxTerm,
        ctx: &mut SyntaxEvalContext<'_>,
    ) -> Result<bool> {
        let term_checkpoint = self.save_checkpoint();
        let mut pre_read = if !self.has_spliced_tokens()
            && self.cursor < self.active_span.end
        {
            self.read_token_or_binary_at(self.cursor)?
        } else {
            None
        };
        if !ctx.unescaped_literal_payload
            && let Some((ch, _)) = pre_read
            && ctx.resolver.is_splice_trigger_token(ch.to_u128())
        {
            let term_targets_say = match term {
                SyntaxTerm::SelfChar => ctx
                    .self_dc
                    .is_some_and(|t| ctx.token_fills_role(t, DcRole::LangSay)),
                SyntaxTerm::CharRef(target)
                | SyntaxTerm::RuleRef { target } => {
                    ctx.token_fills_role(target.to_u128(), DcRole::LangSay)
                }
                SyntaxTerm::CharSet {
                    negated: false,
                    members,
                } => members.iter().any(|target| {
                    ctx.token_fills_role(target.to_u128(), DcRole::LangSay)
                }),
                _ => false,
            };
            if !term_targets_say {
                let spliced = if let Some(evaluator) = &ctx.evaluator {
                    self.splice_stream_invocation(
                        evaluator.as_ref(),
                        ctx.resolver,
                        &ctx.policy,
                    )?
                } else {
                    self.splice_stream_invocation(
                        &DefaultDcExpressionEvaluator,
                        ctx.resolver,
                        &ctx.policy,
                    )?
                };
                if spliced {
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

        if !ctx.unescaped_literal_payload
            && ctx.resolver.is_special_wrapper_token(first)
        {
            // 2. Optional value wrappers: Dc 315 (present) and Dc 316 (absent)
            let term_targets_opt_present = match term {
                SyntaxTerm::SelfChar | SyntaxTerm::CharSet { .. } => true,
                SyntaxTerm::Group(pat) => pat.starts_with_self_or_charset(),
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
                SyntaxTerm::Group(pat) => pat.starts_with_self_or_charset(),
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
                    if !ctx.unescaped_literal_payload
                        && first_dc
                            .fills_role_with(DcRole::Escape, ctx.resolver)
                    {
                        let _ = self.pop_next_token()?;
                        let Some(_) = self.pop_next_token()? else {
                            ctx.has_errors = true;
                            ctx.diagnostics.push(SyntaxDiagnostic {
                                message: format!("Dangling escape Dc 255 at offset {start_offset}"),
                                token_offset: start_offset,
                                is_error: true,
                            });
                            self.restore_checkpoint(term_checkpoint);
                            return Ok(false);
                        };
                        return Ok(true);
                    }
                    let in_opaque = ctx.unescaped_literal_payload
                        || (!from_spliced
                            && ctx.self_dc.is_some_and(|s| {
                                ctx.resolver.is_opaque_container(s)
                            }));
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
                        if first_dc
                            .fills_role_with(DcRole::BeginLiteral, ctx.resolver)
                        {
                            return self.eval_literal_frame(None, None, ctx);
                        }
                        if first_dc.fills_role_with(
                            DcRole::BeginIdentifier,
                            ctx.resolver,
                        ) {
                            return self.eval_identifier_frame(None, ctx);
                        }
                        if first_dc
                            .fills_role_with(DcRole::BeginNumber, ctx.resolver)
                        {
                            return self.eval_number_frame(None, None, ctx);
                        }
                        if let Some(target) = CharTarget::from_u128(first)
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
                let direct_matched = if name == "data" {
                    self.eval_data_construct(subtype.as_ref(), ctx)?
                } else if name == "typed_data" {
                    self.eval_typed_data_construct(subtype.as_ref(), ctx)?
                } else if matches!(name.as_str(), "script" | "format" | "type")
                {
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
                            let _ = self.pop_next_token()?;
                            let matched = if let Some(inner_dc) =
                                self.pop_next_token()?
                                && let Some(close_dc) = self.pop_next_token()?
                                && close_dc.fills_role_with(
                                    DcRole::EndTypeDef,
                                    ctx.resolver,
                                ) {
                                ctx.resolver.token_satisfies_type(
                                    inner_dc.to_u128(),
                                    "type",
                                    subtype.as_ref(),
                                )
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
                        let _ = self.pop_next_token()?;
                        true
                    } else {
                        false
                    }
                } else if name == "string" || name == "String" {
                    self.eval_literal_frame(
                        Some(DcRole::TypeString),
                        subtype.as_ref(),
                        ctx,
                    )?
                } else if name == "Routine" {
                    self.eval_literal_frame(
                        Some(DcRole::TypeRoutine),
                        subtype.as_ref(),
                        ctx,
                    )?
                } else if name == "value"
                    && subtype.is_none()
                    && first_dc
                        .fills_role_with(DcRole::BeginLiteral, ctx.resolver)
                {
                    self.eval_literal_frame(None, None, ctx)?
                } else if name == "value"
                    && subtype.is_none()
                    && first_dc
                        .fills_role_with(DcRole::BeginNumber, ctx.resolver)
                {
                    self.eval_number_frame(None, None, ctx)?
                } else if name == "value"
                    && subtype.is_none()
                    && !from_spliced
                    && first_dc.fills_role_with(
                        DcRole::StartEncapsulationBinary,
                        ctx.resolver,
                    )
                    && let Ok((after_bin, DcFrameHeaderInfo::Binary { .. })) =
                        self.scan_binary_at(
                            start_offset,
                            &ctx.policy,
                            ctx.resolver,
                        )
                {
                    self.cursor = after_bin;
                    true
                } else if name == "identifier" {
                    self.eval_identifier_frame(subtype.as_ref(), ctx)?
                } else if name == "number" || name == "Number" {
                    self.eval_number_frame(
                        subtype.as_ref(),
                        capture_var.as_deref(),
                        ctx,
                    )?
                } else if name == "atom_dc"
                    && subtype.is_none()
                    && ctx.resolver.is_plain_atom_dc(first)
                {
                    let _ = self.pop_next_token()?;
                    true
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
                    let mut matched = false;
                    let resolved_pat = if let Some(sub) = subtype.as_ref() {
                        ctx.resolver
                            .resolve_named_type_with_arg(lookup_name, Some(sub))
                            .map(Cow::Owned)
                    } else {
                        ctx.resolver
                            .resolve_named_type(lookup_name)
                            .map(Cow::Borrowed)
                            .or_else(|| {
                                ctx.resolver
                                    .resolve_named_type_with_arg(
                                        lookup_name,
                                        None,
                                    )
                                    .map(Cow::Owned)
                            })
                    };
                    if let Some(pattern) = resolved_pat {
                        let skip_value_pattern = lookup_name == "value"
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
                            matched = self.eval_pattern(&pattern, ctx)?;
                            ctx.depth = ctx.depth.saturating_sub(1);
                            ctx.call_stack.pop();
                        }
                        if matched
                            && lookup_name == "list"
                            && (subtype.is_some() || list_len_arg.is_some())
                        {
                            if from_spliced {
                                let consumed = self
                                    .consumed_tokens_since(&term_checkpoint)?;
                                let mut mstr = DcMstring::new();
                                for ch in consumed {
                                    mstr.push_char(ch.to_u128());
                                }
                                let mut sub_nav =
                                    DcDocumentNavigator::from_mstr(&mstr);
                                // Reason for fallback: length overflow yields 0 for an empty span.
                                let list_end =
                                    u64::try_from(mstr.len()).unwrap_or(0);
                                matched = sub_nav.eval_list_constraints(
                                    0,
                                    list_end,
                                    subtype.as_ref(),
                                    list_len_arg,
                                    ctx,
                                )?;
                            } else {
                                let list_end = self.cursor;
                                matched = self.eval_list_constraints(
                                    start_offset,
                                    list_end,
                                    subtype.as_ref(),
                                    list_len_arg,
                                    ctx,
                                )?;
                                if matched {
                                    self.cursor = list_end;
                                }
                            }
                        }
                        if !matched
                            && lookup_name == "value"
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
                    && first_dc.fills_role_with(
                        DcRole::EvaluableExpressionStarter,
                        ctx.resolver,
                    )
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
                                let mut evaluated = false;
                                if let Some(evaluator) = &ctx.evaluator {
                                    if let Ok(Some(
                                        DcEvaluatedValue::Document(doc),
                                    )) = evaluator
                                        .eval_expression_for_requirement(
                                            &expr_dc_chars,
                                            &req,
                                            ctx.resolver,
                                        )
                                    {
                                        evaluated = true;
                                        let mut sub_nav =
                                            DcDocumentNavigator::from_mstr(
                                                &doc,
                                            );
                                        let mut sub_ctx = ctx.clone();
                                        if let Ok(true) = sub_nav
                                            .eval_term_single(
                                                term,
                                                &mut sub_ctx,
                                            )
                                        {
                                            self.cursor = stub.outer_span.end;
                                            return Ok(true);
                                        }
                                    }
                                }
                                if !evaluated {
                                    if subtype.as_ref().is_none_or(|sub| {
                                        sub.as_target_str().is_some_and(|s| {
                                            Self::type_tokens_satisfy(
                                                type_tokens,
                                                s,
                                                None,
                                                ctx.resolver,
                                            )
                                        })
                                    }) {
                                        self.cursor = stub.outer_span.end;
                                        return Ok(true);
                                    }
                                }
                                self.restore_checkpoint(checkpoint);
                                return Ok(false);
                            }
                        }

                        if let Some(evaluator) = &ctx.evaluator {
                            if let Ok(Some(DcEvaluatedValue::Document(doc))) =
                                evaluator.eval_expression_for_requirement(
                                    &expr_dc_chars,
                                    &req,
                                    ctx.resolver,
                                )
                            {
                                let mut sub_nav =
                                    DcDocumentNavigator::from_mstr(&doc);
                                let mut sub_ctx = ctx.clone();
                                if let Ok(true) =
                                    sub_nav.eval_term_single(term, &mut sub_ctx)
                                {
                                    self.cursor = stub.outer_span.end;
                                    return Ok(true);
                                }
                            }
                        }
                    }
                    self.restore_checkpoint(checkpoint);
                }
                self.restore_checkpoint(term_checkpoint);
                Ok(false)
            }
            SyntaxTerm::RuleRef { target } => {
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

                    let matched = self.eval_pattern(&rule.pattern, ctx)?;

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
        if !self.eval_pattern(
            &SyntaxPattern::Sequence(
                ctx.resolver
                    .container_header_elements(&list_target)
                    .to_vec(),
            ),
            ctx,
        )? || ctx.has_errors
        {
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

    pub(super) fn eval_data_construct(
        &mut self,
        subtype: Option<&SubtypeConstraint>,
        ctx: &mut SyntaxEvalContext<'_>,
    ) -> Result<bool> {
        let start_offset = self.cursor;
        let Some((first_dc, _)) = self.read_token_or_binary_at(start_offset)?
        else {
            return Ok(false);
        };
        if !first_dc
            .fills_role_with(DcRole::StartEncapsulationBinary, ctx.resolver)
        {
            return Ok(false);
        }
        let constraint: Option<(
            Option<FormatId>,
            Option<&str>,
            Option<usize>,
        )> = match subtype {
            None => None,
            Some(SubtypeConstraint::Data {
                format,
                format_id,
                bytes,
            }) => Some((*format_id, format.as_deref(), *bytes)),
            Some(SubtypeConstraint::Target(fmt)) => Some((
                subtype.and_then(SubtypeConstraint::resolved_format_id),
                Some(fmt.as_str()),
                None,
            )),
            Some(other) => {
                ctx.has_errors = true;
                ctx.diagnostics.push(SyntaxDiagnostic {
                    message: format!(
                        "Invalid data constraint: {}",
                        other.format_dsl()
                    ),
                    token_offset: start_offset,
                    is_error: true,
                });
                return Ok(false);
            }
        };

        let matches_expected_fmt = |expected_fmt_id: Option<FormatId>,
                                    actual_id: Option<u128>|
         -> bool {
            let expected_id = expected_fmt_id.and_then(|f| f.dc_id());
            matches!((expected_id, actual_id), (Some(exp), Some(act)) if ctx.resolver.is_equivalent_to(act, exp))
        };

        if let Some((
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
        )? {
            if ctx.validate_interiors
                && matches!(self.backend, NavBackend::SeekableFile { .. })
            {
                self.parse_binary_arena_node(
                    DcByteSpan::new(start_offset, end_offset),
                    &ctx.policy,
                    ctx.resolver,
                )?;
            }
            if let Some((expected_fmt_id, expected_fmt, expected_bytes)) =
                constraint
            {
                if expected_fmt.is_some() {
                    let actual_id = data_type.as_ref().and_then(|dt| {
                        let mut chars = dt.chars();
                        let first_ch = chars.next()?;
                        if first_ch
                            .fills_role_with(DcRole::BeginTypeDef, ctx.resolver)
                        {
                            chars.next().map(DcChar::to_u128)
                        } else {
                            Some(first_ch.to_u128())
                        }
                    });
                    if !matches_expected_fmt(expected_fmt_id, actual_id) {
                        return Ok(false);
                    }
                }
                if let Some(req_bytes) = expected_bytes {
                    if payload_len != u64::try_from(req_bytes)? {
                        return Ok(false);
                    }
                }
            }
            self.cursor = end_offset;
            return Ok(true);
        }

        let Some(target_203) = CharTarget::from_u128(first_dc.to_u128()) else {
            return Ok(false);
        };
        let Some(rule_203) = ctx.resolver.resolve_rule(&target_203) else {
            return Ok(false);
        };
        let old_self = ctx.self_dc;
        ctx.self_dc = Some(first_dc.to_u128());
        let matched = self.eval_pattern(&rule_203.pattern, ctx)?;
        ctx.self_dc = old_self;
        if !matched {
            self.cursor = start_offset;
            return Ok(false);
        }
        let end_offset = self.cursor;

        if let Some((expected_fmt_id, expected_fmt, expected_bytes)) =
            constraint
        {
            let tokens = self.read_tokens_in_span(DcByteSpan::new(
                start_offset,
                end_offset,
            ))?;
            if expected_fmt.is_some() {
                let actual_id = if tokens.get(1).is_some_and(|&t| {
                    ctx.token_fills_role(t, DcRole::BeginTypeDef)
                }) {
                    tokens.get(2).copied()
                } else {
                    tokens.get(1).copied()
                };
                if !matches_expected_fmt(expected_fmt_id, actual_id) {
                    self.cursor = start_offset;
                    return Ok(false);
                }
            }
            if let Some(req_bytes) = expected_bytes {
                match encapsulated_byte_length(&tokens, ctx.resolver) {
                    Ok(len) if len == req_bytes => {}
                    Ok(_) => {
                        self.cursor = start_offset;
                        return Ok(false);
                    }
                    Err(err) => {
                        ctx.has_errors = true;
                        ctx.diagnostics.push(SyntaxDiagnostic {
                            message: format!(
                                "Invalid constrained binary payload: {err}"
                            ),
                            token_offset: start_offset,
                            is_error: true,
                        });
                        self.cursor = start_offset;
                        return Ok(false);
                    }
                }
            }
        }
        Ok(true)
    }

    pub(super) fn eval_typed_data_construct(
        &mut self,
        subtype: Option<&SubtypeConstraint>,
        ctx: &mut SyntaxEvalContext<'_>,
    ) -> Result<bool> {
        let start_offset = self.cursor;
        let Some((first_dc, _)) = self.read_token_or_binary_at(start_offset)?
        else {
            return Ok(false);
        };
        if !first_dc
            .fills_role_with(DcRole::StartEncapsulationBinary, ctx.resolver)
        {
            return Ok(false);
        }
        if let Some((end_offset, DcFrameHeaderInfo::Binary { data_type, .. })) =
            self.try_scan_raw_binary_at(
                start_offset,
                &ctx.policy,
                ctx.resolver,
            )?
        {
            let Some(ref dt) = data_type else {
                return Ok(false);
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
                return Ok(true);
            }
            return Ok(false);
        }
        let Some(pattern) = ctx
            .resolver
            .resolve_named_type_with_arg("typed_data", subtype)
        else {
            return Ok(false);
        };
        self.eval_pattern(&pattern, ctx)
    }

    pub(super) fn eval_literal_frame(
        &mut self,
        expected_type_role: Option<DcRole>,
        subtype: Option<&SubtypeConstraint>,
        ctx: &mut SyntaxEvalContext<'_>,
    ) -> Result<bool> {
        let start_offset = self.cursor;
        self.scan_cache.ensure_resolver(ctx.resolver);
        if subtype.is_none()
            && !self.has_spliced_tokens()
            && let Some(&(cached_end, CachedLeafKind::Literal(type_dc))) =
                self.scan_cache.validated_leaf_frames.get(&start_offset)
            && cached_end <= self.active_span.end
        {
            if let Some(role) = expected_type_role
                && !type_dc.fills_role_with(role, ctx.resolver)
                && !type_dc.fills_role_with(DcRole::TypeAny, ctx.resolver)
            {
                return Ok(false);
            }
            self.cursor = cached_end;
            return Ok(true);
        }
        let term_checkpoint = self.save_checkpoint();
        let Some(first_dc) = self.pop_next_token()? else {
            return Ok(false);
        };
        if !first_dc.fills_role_with(DcRole::BeginLiteral, ctx.resolver) {
            self.restore_checkpoint(term_checkpoint);
            return Ok(false);
        }
        let Some(hdr_open) = self.pop_next_token()? else {
            ctx.has_errors = true;
            ctx.diagnostics.push(SyntaxDiagnostic {
                message: format!("Unclosed literal at offset {start_offset}: missing Dc 262..263"),
                token_offset: start_offset,
                is_error: true,
            });
            self.restore_checkpoint(term_checkpoint);
            return Ok(false);
        };
        if !hdr_open.fills_role_with(DcRole::BeginTypeDef, ctx.resolver) {
            ctx.has_errors = true;
            ctx.diagnostics.push(SyntaxDiagnostic {
                message: format!("Literal at offset {start_offset} is missing Dc 262..263 type header"),
                token_offset: start_offset,
                is_error: true,
            });
            self.restore_checkpoint(term_checkpoint);
            return Ok(false);
        }
        let Some(type_dc) = self.pop_next_token()? else {
            ctx.has_errors = true;
            self.restore_checkpoint(term_checkpoint);
            return Ok(false);
        };
        if !ctx.resolver.matches_script(".EL Types", type_dc.to_u128()) {
            ctx.has_errors = true;
            self.restore_checkpoint(term_checkpoint);
            return Ok(false);
        }
        let Some(hdr_close) = self.pop_next_token()? else {
            ctx.has_errors = true;
            self.restore_checkpoint(term_checkpoint);
            return Ok(false);
        };
        if !hdr_close.fills_role_with(DcRole::EndTypeDef, ctx.resolver) {
            ctx.has_errors = true;
            self.restore_checkpoint(term_checkpoint);
            return Ok(false);
        }
        if let Some(role) = expected_type_role
            && !type_dc.fills_role_with(role, ctx.resolver)
            && !type_dc.fills_role_with(DcRole::TypeAny, ctx.resolver)
        {
            self.restore_checkpoint(term_checkpoint);
            return Ok(false);
        }

        let mut decoded_payload = Vec::new();
        while let Some(ch) = self.pop_next_token()? {
            if ch.fills_role_with(DcRole::Escape, ctx.resolver) {
                let Some(escaped) = self.pop_next_token()? else {
                    ctx.has_errors = true;
                    ctx.diagnostics.push(SyntaxDiagnostic {
                        message: format!(
                            "Dangling escape Dc 255 in literal at offset {}",
                            self.cursor
                        ),
                        token_offset: self.cursor,
                        is_error: true,
                    });
                    self.restore_checkpoint(term_checkpoint);
                    return Ok(false);
                };
                if subtype.is_some() {
                    decoded_payload.push(escaped);
                }
            } else if ch.fills_role_with(DcRole::EndLiteral, ctx.resolver) {
                if let Some(sub) = subtype {
                    if ctx.depth >= ctx.max_depth {
                        ctx.has_errors = true;
                        ctx.diagnostics.push(SyntaxDiagnostic {
                            message: "String constraint expansion depth limit reached".to_owned(),
                            token_offset: start_offset,
                            is_error: true,
                        });
                        self.restore_checkpoint(term_checkpoint);
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
                        self.restore_checkpoint(term_checkpoint);
                        return Ok(false);
                    };
                    let mut sub_nav =
                        DcDocumentNavigator::from_dc_str(<&DcStr>::from(""));
                    sub_nav.push_spliced_tokens(decoded_payload);
                    let mut sub_ctx =
                        SyntaxEvalContext::new(ctx.resolver, None);
                    sub_ctx.policy.clone_from(&ctx.policy);
                    sub_ctx.depth = ctx.depth.saturating_add(1);
                    sub_ctx.max_depth = ctx.max_depth;
                    sub_ctx.call_stack.clone_from(&ctx.call_stack);
                    sub_ctx.unescaped_literal_payload = true;
                    let sub_ok =
                        sub_nav.eval_pattern(sub_pattern, &mut sub_ctx)?;
                    ctx.has_errors |= sub_ctx.has_errors;
                    ctx.diagnostics.extend(sub_ctx.diagnostics);
                    if !sub_ok
                        || sub_ctx.has_errors
                        || sub_nav.has_spliced_tokens()
                    {
                        self.restore_checkpoint(term_checkpoint);
                        return Ok(false);
                    }
                }
                if !term_checkpoint.has_spliced()
                    && !self.has_spliced_tokens()
                {
                    if self.scan_cache.validated_leaf_frames.len()
                        >= MAX_NAV_CACHE_ENTRIES
                    {
                        self.scan_cache.validated_leaf_frames.clear();
                    }
                    self.scan_cache.validated_leaf_frames.insert(
                        start_offset,
                        (self.cursor, CachedLeafKind::Literal(type_dc)),
                    );
                }
                return Ok(true);
            } else if subtype.is_some() {
                decoded_payload.push(ch);
            }
        }
        ctx.has_errors = true;
        ctx.diagnostics.push(SyntaxDiagnostic {
            message: format!(
                "Unclosed literal frame starting at offset {start_offset}"
            ),
            token_offset: start_offset,
            is_error: true,
        });
        self.restore_checkpoint(term_checkpoint);
        Ok(false)
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

    pub(super) fn eval_identifier_frame(
        &mut self,
        subtype: Option<&SubtypeConstraint>,
        ctx: &mut SyntaxEvalContext<'_>,
    ) -> Result<bool> {
        let start_offset = self.cursor;
        self.scan_cache.ensure_resolver(ctx.resolver);
        if subtype.is_none()
            && !self.has_spliced_tokens()
            && let Some(&(cached_end, CachedLeafKind::Identifier)) =
                self.scan_cache.validated_leaf_frames.get(&start_offset)
            && cached_end <= self.active_span.end
        {
            self.cursor = cached_end;
            return Ok(true);
        }
        let term_checkpoint = self.save_checkpoint();
        let Some(first_dc) = self.pop_next_token()? else {
            return Ok(false);
        };
        if !first_dc.fills_role_with(DcRole::BeginIdentifier, ctx.resolver) {
            self.restore_checkpoint(term_checkpoint);
            return Ok(false);
        }
        // Reason for fallback: opener defaults to CharTarget::Dc(270) and closer defaults to DC_END_IDENTIFIER.
        let closer = CharTarget::from_u128(first_dc.to_u128())
            .and_then(|t| ctx.resolver.closer_for_opener(&t))
            .map_or(
                ctb_formats_dcdata::dc::DC_END_IDENTIFIER,
                DcChar::from_u128,
            );
        let leading_type_tokens =
            self.try_consume_optional_leading_type(closer, ctx)?;
        if ctx.has_errors {
            self.restore_checkpoint(term_checkpoint);
            return Ok(false);
        }
        let mut char_count = 0usize;
        let mut name_mstr = subtype.is_some().then(DcMstring::new);
        while let Some(ch) = self.pop_next_token()? {
            if ch.fills_role_with(DcRole::Escape, ctx.resolver) {
                let Some(escaped) = self.pop_next_token()? else {
                    ctx.has_errors = true;
                    ctx.diagnostics.push(SyntaxDiagnostic {
                        message: format!(
                            "Dangling escape Dc 255 in identifier at offset {}",
                            self.cursor
                        ),
                        token_offset: self.cursor,
                        is_error: true,
                    });
                    self.restore_checkpoint(term_checkpoint);
                    return Ok(false);
                };
                if let Some(ref mut mstr) = name_mstr {
                    mstr.push_char(escaped);
                }
                char_count = char_count.saturating_add(1);
            } else if ch.fills_role_with(DcRole::EndIdentifier, ctx.resolver) {
                if char_count == 0 {
                    ctx.has_errors = true;
                    ctx.diagnostics.push(SyntaxDiagnostic {
                        message: format!("Empty identifier frame (Dc 270..271) at offset {start_offset}"),
                        token_offset: start_offset,
                        is_error: true,
                    });
                    self.restore_checkpoint(term_checkpoint);
                    return Ok(false);
                }
                if let Some(sub) = subtype {
                    let Some(sub_str) = sub.as_target_str() else {
                        self.restore_checkpoint(term_checkpoint);
                        return Ok(false);
                    };
                    let type_ok =
                        if let Some(ref type_tokens) = leading_type_tokens {
                            Self::type_tokens_satisfy(
                                type_tokens,
                                sub_str,
                                None,
                                ctx.resolver,
                            )
                        } else if let Some(ref mstr) = name_mstr {
                            self.identifier_bound_to_type_in_scope(
                                mstr,
                                start_offset,
                                sub_str,
                                ctx,
                            )?
                        } else {
                            false
                        };
                    if !type_ok {
                        self.restore_checkpoint(term_checkpoint);
                        return Ok(false);
                    }
                }
                if !term_checkpoint.has_spliced()
                    && !self.has_spliced_tokens()
                {
                    if self.scan_cache.validated_leaf_frames.len()
                        >= MAX_NAV_CACHE_ENTRIES
                    {
                        self.scan_cache.validated_leaf_frames.clear();
                    }
                    self.scan_cache.validated_leaf_frames.insert(
                        start_offset,
                        (self.cursor, CachedLeafKind::Identifier),
                    );
                }
                return Ok(true);
            } else {
                if let Some(ref mut mstr) = name_mstr {
                    mstr.push_char(ch);
                }
                char_count = char_count.saturating_add(1);
            }
        }
        ctx.has_errors = true;
        ctx.diagnostics.push(SyntaxDiagnostic {
            message: format!(
                "Unclosed identifier frame starting at offset {start_offset}"
            ),
            token_offset: start_offset,
            is_error: true,
        });
        self.restore_checkpoint(term_checkpoint);
        Ok(false)
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
        if let Some(evaluator) = &ctx.evaluator {
            let mut expr_dcs = vec![
                ctb_formats_dcdata::dc::DC_REFERENCE_BOUND_VALUE,
                ctb_formats_dcdata::dc::DC_BEGIN_IDENTIFIER,
            ];
            if let Ok(dc_str) = ident_name.to_dc_string() {
                for ch in dc_str.chars() {
                    if ch.fills_role_with(DcRole::Escape, ctx.resolver)
                        || ch.fills_role_with(
                            DcRole::EndIdentifier,
                            ctx.resolver,
                        )
                    {
                        expr_dcs.push(ctb_formats_dcdata::dc::DC_ESCAPE);
                    }
                    expr_dcs.push(ch);
                }
                expr_dcs.push(ctb_formats_dcdata::dc::DC_END_IDENTIFIER);
                if let Ok(Some(DcEvaluatedValue::Document(doc))) = evaluator
                    .eval_expression_for_requirement(
                        &expr_dcs,
                        &req,
                        ctx.resolver,
                    )
                {
                    let mut sub_nav = DcDocumentNavigator::from_mstr(&doc);
                    let mut sub_ctx = ctx.clone();
                    let term = SyntaxTerm::NamedConstruct {
                        name: expected_type.to_owned(),
                        subtype: None,
                        capture_var: None,
                    };
                    if let Ok(true) =
                        sub_nav.eval_term_single(&term, &mut sub_ctx)
                    {
                        return Ok(true);
                    }
                }
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
                        && let Some(DcSyntaxKind::Assignment { ident, value }) =
                            temp_arena.get(root_id).map(|n| &n.kind)
                        && let Some(val_node) = temp_arena.get(*value)
                    {
                        if self.scan_cache.top_level_assignments.len()
                            >= MAX_NAV_CACHE_ENTRIES
                        {
                            self.scan_cache.top_level_assignments.clear();
                        }
                        self.scan_cache.top_level_assignments.push((
                            stub.outer_span.end,
                            ident.clone(),
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
        let term = SyntaxTerm::NamedConstruct {
            name: expected_type.to_owned(),
            subtype: None,
            capture_var: None,
        };
        // Reason for fallback: term evaluation errors or mismatches evaluate to non-match (false).
        let ok = self.eval_term_single(&term, &mut sub_ctx).unwrap_or(false)
            && !sub_ctx.has_errors
            && self.cursor == val_span.end;
        self.diagnostics.truncate(saved_diag_len);
        self.restore_checkpoint(saved_for_eval);
        Ok(ok)
    }

    pub(super) fn eval_number_frame(
        &mut self,
        subtype: Option<&SubtypeConstraint>,
        capture_var: Option<&str>,
        ctx: &mut SyntaxEvalContext<'_>,
    ) -> Result<bool> {
        let start_offset = self.cursor;
        let need_digits = subtype.is_some() || capture_var.is_some();
        self.scan_cache.ensure_resolver(ctx.resolver);
        if !need_digits
            && !self.has_spliced_tokens()
            && let Some(&(cached_end, CachedLeafKind::Number(_))) =
                self.scan_cache.validated_leaf_frames.get(&start_offset)
            && cached_end <= self.active_span.end
        {
            self.cursor = cached_end;
            return Ok(true);
        }
        let term_checkpoint = self.save_checkpoint();
        let Some(first_dc) = self.pop_next_token()? else {
            return Ok(false);
        };
        if !first_dc.fills_role_with(DcRole::BeginNumber, ctx.resolver) {
            self.restore_checkpoint(term_checkpoint);
            return Ok(false);
        }
        let Some(fmt_dc) = self.pop_next_token()? else {
            self.restore_checkpoint(term_checkpoint);
            return Ok(false);
        };
        if !ctx
            .resolver
            .matches_script("Formats:number", fmt_dc.to_u128())
            && !ctx.resolver.matches_script("number", fmt_dc.to_u128())
        {
            self.restore_checkpoint(term_checkpoint);
            return Ok(false);
        }
        let mut digits = Vec::new();
        let mut digit_count = 0usize;
        while let Some(ch) = self.pop_next_token()? {
            if ch.fills_role_with(DcRole::EndNumber, ctx.resolver) {
                if digit_count == 0 {
                    self.restore_checkpoint(term_checkpoint);
                    return Ok(false);
                }
                if need_digits {
                    let Ok(val) =
                        crate::dc_number::parse_dc_number_digits_with_resolver(
                            &digits,
                            ctx.resolver,
                        )
                    else {
                        self.restore_checkpoint(term_checkpoint);
                        return Ok(false);
                    };
                    if let Some(sub) = subtype {
                        if !number_satisfies_subtype(&val, sub, ctx.resolver) {
                            self.restore_checkpoint(term_checkpoint);
                            return Ok(false);
                        }
                    }
                    if let Some(var) = capture_var {
                        ctx.captured_numbers.insert(
                            var.trim_start_matches('$').to_owned(),
                            val,
                        );
                    }
                }
                if !term_checkpoint.has_spliced()
                    && !self.has_spliced_tokens()
                {
                    if self.scan_cache.validated_leaf_frames.len()
                        >= MAX_NAV_CACHE_ENTRIES
                    {
                        self.scan_cache.validated_leaf_frames.clear();
                    }
                    self.scan_cache.validated_leaf_frames.insert(
                        start_offset,
                        (self.cursor, CachedLeafKind::Number(fmt_dc)),
                    );
                }
                return Ok(true);
            }
            digit_count = digit_count.saturating_add(1);
            if need_digits {
                digits.push(ch);
            }
        }
        self.restore_checkpoint(term_checkpoint);
        Ok(false)
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
        self.detect_leading_document_most_recent();
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
                } else if resolver
                    .token_fills_role(target.to_u128(), DcRole::BeginLiteral)
                {
                    self.eval_literal_frame(None, None, &mut ctx)?
                } else if resolver
                    .token_fills_role(target.to_u128(), DcRole::BeginIdentifier)
                {
                    self.eval_identifier_frame(None, &mut ctx)?
                } else if resolver
                    .token_fills_role(target.to_u128(), DcRole::BeginNumber)
                {
                    self.eval_number_frame(None, None, &mut ctx)?
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
            let header = resolver.container_header_elements(target);
            let mut declared_list_len = None;
            if !header.is_empty() {
                let mut ctx =
                    SyntaxEvalContext::new(resolver, Some(target.to_u128()));
                ctx.policy.clone_from(policy);
                if !self.eval_pattern(
                    &SyntaxPattern::Sequence(header.to_vec()),
                    &mut ctx,
                )? || ctx.has_errors
                {
                    self.syntax_issue(
                        policy,
                        inner.start,
                        format!("Container header mismatch inside {target} at offset {}", inner.start),
                    )?;
                }
                declared_list_len = ctx.captured_numbers.remove("length");
            }
            if let Some(length) = declared_list_len
                && resolver
                    .token_fills_role(target.to_u128(), DcRole::BeginList)
            {
                let body_start = self.cursor;
                self.check_declared_list_length_with_resolver(
                    target, inner, body_start, &length, policy, resolver,
                )?;
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
        let saved_span = self.active_span;
        let saved_cursor = self.cursor;
        let mut span_stack: Vec<(DcByteSpan, u64, usize)> =
            vec![(initial_span, initial_span.start, initial_depth)];

        let res = (|| -> Result<()> {
            while let Some((span, cursor, depth)) = span_stack.pop() {
                if cursor >= span.end {
                    continue;
                }
                self.active_span = span;
                self.cursor = cursor;
                let sub_spans = self.step_validate_construct_with_resolver(
                    depth, policy, resolver,
                )?;
                let next_cursor = self.cursor;
                ensure!(
                    next_cursor > cursor,
                    "Parser made no progress at offset {cursor}"
                );
                if next_cursor < span.end {
                    span_stack.push((span, next_cursor, depth));
                }
                for (sub_span, sub_depth) in sub_spans.into_iter().rev() {
                    if !sub_span.is_empty() {
                        span_stack.push((sub_span, sub_span.start, sub_depth));
                    }
                }
            }
            Ok(())
        })();

        self.active_span = saved_span;
        self.cursor = saved_cursor.min(saved_span.end);
        res
    }

    pub(super) fn step_validate_construct_with_resolver(
        &mut self,
        depth: usize,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<Vec<(DcByteSpan, usize)>> {
        let Some(stub) =
            self.next_sibling_stub_with_resolver(policy, resolver)?
        else {
            return Ok(Vec::new());
        };
        if stub
            .head_dc
            .fills_role_with(DcRole::StartEncapsulationBinary, resolver)
            && matches!(stub.header_info, DcFrameHeaderInfo::Binary { .. })
        {
            self.parse_binary_arena_node(stub.outer_span, policy, resolver)?;
        }
        if let Some(inner) = stub.inner_span {
            if !resolver.is_opaque_container(stub.head_dc.to_u128()) {
                if depth >= policy.max_depth {
                    self.record_diagnostic(
                        policy,
                        policy.depth_limit_exceeded,
                        stub.outer_span.start,
                        format!(
                            "Document nesting depth exceeds maximum limit ({}) at offset {}",
                            policy.max_depth, stub.outer_span.start,
                        ),
                    )?;
                } else {
                    return Ok(vec![(inner, depth.saturating_add(1))]);
                }
            }
        }
        Ok(Vec::new())
    }
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
