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

//! Resilient pattern matching primitives for evaluating Dc syntax rules
//! against character ID streams in DcText documents.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::ast::{
    CharTarget, DcSyntaxRule, MatchMode, Quantifier, SyntaxDiagnostic,
    SyntaxElement, SyntaxPattern, SyntaxTerm,
};
use super::framing::{
    DC_IDENTIFIER_BEGIN, DC_LITERAL_BEGIN, scan_identifier_frame,
    scan_literal_frame,
};
use super::resolver::SyntaxRuleResolver;
use std::collections::HashMap;
use std::sync::Arc;

/// Maximum recursion depth allowed during grammar rule expansion.
pub const MAX_SYNTAX_EXPANSION_DEPTH: usize = 32;

/// Result of evaluating a pattern match against a stream of character tokens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatchOutcome {
    /// Exact match consuming the specified number of tokens.
    Matched { consumed: usize },
    /// Resilient error-recovery match (tag-soup recovery) with diagnostic warning.
    MatchedWithRecovery { consumed: usize, warning: String },
    /// Pattern does not match current stream position.
    Mismatch,
}

impl MatchOutcome {
    /// Returns true if the outcome represents a match (exact or recovered).
    #[must_use]
    pub const fn is_matched(&self) -> bool {
        matches!(self, Self::Matched { .. } | Self::MatchedWithRecovery { .. })
    }

    /// Returns the number of tokens consumed if matched.
    #[must_use]
    pub const fn consumed_tokens(&self) -> usize {
        match self {
            Self::Matched { consumed } | Self::MatchedWithRecovery { consumed, .. } => *consumed,
            Self::Mismatch => 0,
        }
    }
}

/// Execution context for pattern matching, recording variable captures and diagnostics.
#[derive(Clone)]
pub struct MatchContext {
    /// The global ID of the defining character (for resolving `~`).
    pub self_dc: Option<u128>,
    /// Captured character token sequences keyed by variable name (e.g. `ident`, `val`).
    pub captured_vars: HashMap<String, Vec<u128>>,
    /// Diagnostic warnings collected during resilient recovery.
    pub warnings: Vec<String>,
    /// Mode governing matching: Permissive (tag-soup) vs Strict.
    pub mode: MatchMode,
    /// Detailed syntax diagnostics collected during parsing/matching.
    pub diagnostics: Vec<SyntaxDiagnostic>,
    /// True if any framing or syntax errors occurred.
    pub has_errors: bool,
    /// Current recursion depth during rule/named type expansion.
    pub depth: usize,
    /// Maximum recursion depth allowed.
    pub max_depth: usize,
    /// Call stack for cycle detection.
    pub call_stack: Vec<String>,
    /// Optional rule resolver for expanding rule references and named types.
    pub resolver: Option<Arc<dyn SyntaxRuleResolver + Send + Sync>>,
}

impl Default for MatchContext {
    fn default() -> Self {
        Self::new(None)
    }
}

impl std::fmt::Debug for MatchContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MatchContext")
            .field("self_dc", &self.self_dc)
            .field("captured_vars", &self.captured_vars)
            .field("warnings", &self.warnings)
            .field("mode", &self.mode)
            .field("diagnostics", &self.diagnostics)
            .field("has_errors", &self.has_errors)
            .field("depth", &self.depth)
            .field("max_depth", &self.max_depth)
            .field("call_stack", &self.call_stack)
            .finish()
    }
}

impl MatchContext {
    /// Creates a new match context for a specific defining short Dc in Permissive mode.
    #[must_use]
    pub fn new(self_dc: Option<u32>) -> Self {
        Self::new_with_token(self_dc.map(|id| DcChar::from_short(id).to_u128()))
    }

    /// Creates a new match context for a specific defining global character token in Permissive mode.
    #[must_use]
    pub fn new_with_token(self_dc: Option<u128>) -> Self {
        Self {
            self_dc,
            captured_vars: HashMap::new(),
            warnings: Vec::new(),
            mode: MatchMode::Permissive,
            diagnostics: Vec::new(),
            has_errors: false,
            depth: 0,
            max_depth: MAX_SYNTAX_EXPANSION_DEPTH,
            call_stack: Vec::new(),
            resolver: None,
        }
    }

    /// Sets the match mode (Strict or Permissive).
    #[must_use]
    pub fn with_mode(mut self, mode: MatchMode) -> Self {
        self.mode = mode;
        self
    }

    /// Sets the syntax rule resolver.
    #[must_use]
    pub fn with_resolver(
        mut self,
        resolver: Arc<dyn SyntaxRuleResolver + Send + Sync>,
    ) -> Self {
        self.resolver = Some(resolver);
        self
    }
}

/// Helper checking if a single character token matches a `CharTarget`.
fn target_matches_token(target: &CharTarget, token: u128) -> bool {
    target.to_u128() == token
}

/// Evaluates a single `SyntaxTerm` against a character stream.
fn match_term_single(
    stream: &[u128],
    term: &SyntaxTerm,
    context: &mut MatchContext,
) -> MatchOutcome {
    let Some(&first) = stream.first() else {
        return MatchOutcome::Mismatch;
    };

    // Equivalent representations wrapper: Dc 397 (can wrap alternative representations of any value)
    let dc_equiv = ctb_storage_minimal::global_graph_layout::SHORT_DC_REGION_START
        .saturating_add(397);
    if first == dc_equiv
        && !matches!(
            term,
            SyntaxTerm::SelfChar
                | SyntaxTerm::CharRef(CharTarget::Dc(397))
                | SyntaxTerm::RuleRef {
                    target: CharTarget::Dc(397),
                }
        )
    {
        if let Some(ref resolver) = context.resolver.clone() {
            if let Some(rule) = resolver.resolve_rule(&CharTarget::Dc(397)) {
                let old_self = context.self_dc;
                context.self_dc = Some(dc_equiv);
                let alt_outcome = match_pattern(stream, &rule.pattern, context);
                context.self_dc = old_self;
                if alt_outcome.is_matched() {
                    return alt_outcome;
                }
            }
        }
    }

    let outcome = match term {
        SyntaxTerm::SelfChar => {
            if let Some(self_id) = context.self_dc {
                if first == self_id {
                    MatchOutcome::Matched { consumed: 1 }
                } else {
                    MatchOutcome::Mismatch
                }
            } else {
                MatchOutcome::Matched { consumed: 1 }
            }
        }
        SyntaxTerm::CharRef(target) => {
            if target_matches_token(target, first) {
                MatchOutcome::Matched { consumed: 1 }
            } else {
                MatchOutcome::Mismatch
            }
        }
        SyntaxTerm::CharSet { negated, members } => {
            let in_set = members.iter().any(|m| target_matches_token(m, first));
            let matches = if *negated { !in_set } else { in_set };
            if matches {
                MatchOutcome::Matched { consumed: 1 }
            } else {
                MatchOutcome::Mismatch
            }
        }
        SyntaxTerm::CharRange { start, end } => {
            let s = start.to_u128();
            let e = end.to_u128();
            if (s..=e).contains(&first) {
                MatchOutcome::Matched { consumed: 1 }
            } else {
                MatchOutcome::Mismatch
            }
        }
        SyntaxTerm::Group(group_pat) => match_pattern(stream, group_pat, context),
        SyntaxTerm::NamedConstruct {
            name,
            subtype,
            capture_var,
        } => {
            // Script classification check: [script:<name>]
            if name == "script" {
                if let (Some(sub), Some(resolver)) = (subtype, &context.resolver) {
                    if resolver.matches_script(sub, first) {
                        if let Some(var) = capture_var {
                            context
                                .captured_vars
                                .entry(var.clone())
                                .or_default()
                                .push(first);
                        }
                        return MatchOutcome::Matched { consumed: 1 };
                    }
                    return MatchOutcome::Mismatch;
                }
            }

            // Format or type category check: [format:<category>] or [type:<category>]
            if (name == "format" || name == "type") && subtype.is_some() {
                if let (Some(sub), Some(resolver)) = (subtype, &context.resolver) {
                    let script_name = format!("Formats:{sub}");
                    if resolver.matches_script(&script_name, first)
                        || resolver.matches_script(sub, first)
                    {
                        if let Some(var) = capture_var {
                            context
                                .captured_vars
                                .entry(var.clone())
                                .or_default()
                                .push(first);
                        }
                        return MatchOutcome::Matched { consumed: 1 };
                    }
                    return MatchOutcome::Mismatch;
                }
            }

            // General type check: [type] (matches EL Types, format types, or type definitions)
            if name == "type" && subtype.is_none() {
                let is_format_token = (ctb_storage_minimal::global_graph_layout::FORMAT_REGION_START
                    ..=ctb_storage_minimal::global_graph_layout::FORMAT_REGION_END)
                    .contains(&first);
                    // FIXME: This should use a named Dc const from dcdata/dc.generated.rs.
                let dc_path_delim = ctb_storage_minimal::global_graph_layout::SHORT_DC_REGION_START
                    .saturating_add(513);
                let matches = is_format_token
                    || first == dc_path_delim
                    || context.resolver.as_ref().map_or(false, |r| {
                        r.matches_script("EL Types", first) || r.matches_script("Formats", first)
                    });
                if matches {
                    if let Some(var) = capture_var {
                        context
                            .captured_vars
                            .entry(var.clone())
                            .or_default()
                            .push(first);
                    }
                    return MatchOutcome::Matched { consumed: 1 };
                }

                // Type constructor rules: Dc 516 (Type intersection) or Dc 300 (Type union)
                let dc_intersect = ctb_storage_minimal::global_graph_layout::SHORT_DC_REGION_START
                    .saturating_add(516);
                let dc_union = ctb_storage_minimal::global_graph_layout::SHORT_DC_REGION_START
                    .saturating_add(300);
                if first == dc_intersect || first == dc_union {
                    if let Some(ref resolver) = context.resolver.clone() {
                        let target = if first == dc_intersect {
                            CharTarget::Dc(516)
                        } else {
                            CharTarget::Dc(300)
                        };
                        if let Some(rule) = resolver.resolve_rule(&target) {
                            let old_self = context.self_dc;
                            context.self_dc = Some(first);
                            let outcome = match_pattern(stream, &rule.pattern, context);
                            context.self_dc = old_self;
                            if outcome.is_matched() {
                                return outcome;
                            }
                        }
                    }
                }
            }

            // Format check: [format] or [formats]
            if (name == "format" || name == "formats") && subtype.is_none() {
                let is_format_token = (ctb_storage_minimal::global_graph_layout::FORMAT_REGION_START
                    ..=ctb_storage_minimal::global_graph_layout::FORMAT_REGION_END)
                    .contains(&first);
                let matches = is_format_token
                    || context
                        .resolver
                        .as_ref()
                        .map_or(false, |r| r.matches_script("Formats", first));
                if matches {
                    if let Some(var) = capture_var {
                        context
                            .captured_vars
                            .entry(var.clone())
                            .or_default()
                            .push(first);
                    }
                    return MatchOutcome::Matched { consumed: 1 };
                }
            }

            // Typed literal framing: [string]
            if name == "string" {
                if first == DC_LITERAL_BEGIN {
                    if let Some((lit, consumed)) = scan_literal_frame(
                        stream,
                        context.mode,
                        &mut context.diagnostics,
                        0,
                    ) {
                        if let Some(var) = capture_var {
                            context
                                .captured_vars
                                .entry(var.clone())
                                .or_default()
                                .extend_from_slice(&lit.raw_tokens);
                        }
                        return MatchOutcome::Matched { consumed };
                    }
                    return MatchOutcome::Mismatch;
                }
                if context.resolver.is_none() {
                    if let Some(var) = capture_var {
                        context
                            .captured_vars
                            .entry(var.clone())
                            .or_default()
                            .push(first);
                    }
                    return MatchOutcome::Matched { consumed: 1 };
                }
                return MatchOutcome::Mismatch;
            }

            // Identifier framing: [identifier]
            if name == "identifier" {
                if first == DC_IDENTIFIER_BEGIN {
                    if let Some((ident, consumed)) = scan_identifier_frame(
                        stream,
                        context.mode,
                        &mut context.diagnostics,
                        0,
                    ) {
                        if let Some(var) = capture_var {
                            context
                                .captured_vars
                                .entry(var.clone())
                                .or_default()
                                .extend_from_slice(&ident.raw_tokens);
                        }
                        return MatchOutcome::Matched { consumed };
                    }
                    return MatchOutcome::Mismatch;
                }
                if context.resolver.is_none() {
                    if let Some(var) = capture_var {
                        context
                            .captured_vars
                            .entry(var.clone())
                            .or_default()
                            .push(first);
                    }
                    return MatchOutcome::Matched { consumed: 1 };
                }
                return MatchOutcome::Mismatch;
            }

            // Recursive expansion for registered named types
            if let Some(ref resolver) = context.resolver.clone() {
                if context.depth >= context.max_depth {
                    context.has_errors = true;
                    context.diagnostics.push(SyntaxDiagnostic {
                        message: format!(
                            "Syntax expansion depth limit reached ({}) for named construct [{name}]",
                            context.max_depth
                        ),
                        token_offset: 0,
                        is_error: true,
                    });
                    return MatchOutcome::Mismatch;
                }
                let frame_key = format!("named:{name}");
                if context.call_stack.contains(&frame_key) {
                    return MatchOutcome::Mismatch;
                }
                if let Some(pattern) = resolver.resolve_named_type_with_arg(name, subtype.as_deref()) {
                    context.call_stack.push(frame_key);
                    context.depth = context.depth.saturating_add(1);
                    let outcome = match_pattern(stream, &pattern, context);
                    context.depth = context.depth.saturating_sub(1);
                    context.call_stack.pop();
                    if outcome.is_matched() {
                        let consumed = outcome.consumed_tokens();
                        if let Some(var) = capture_var {
                            if let Some(slice) = stream.get(..consumed) {
                                context
                                    .captured_vars
                                    .entry(var.clone())
                                    .or_default()
                                    .extend_from_slice(slice);
                            }
                        }
                        return outcome;
                    }
                    return MatchOutcome::Mismatch;
                }
            }

            // Fallback for placeholder consumption when resolver is absent
            if context.resolver.is_none() {
                if let Some(var) = capture_var {
                    context
                        .captured_vars
                        .entry(var.clone())
                        .or_default()
                        .push(first);
                }
                return MatchOutcome::Matched { consumed: 1 };
            }
            MatchOutcome::Mismatch
        }
        SyntaxTerm::RuleRef { target } => {
            if let Some(ref resolver) = context.resolver.clone() {
                if context.depth >= context.max_depth {
                    context.has_errors = true;
                    context.diagnostics.push(SyntaxDiagnostic {
                        message: format!(
                            "Syntax expansion depth limit reached ({}) for rule {target}",
                            context.max_depth
                        ),
                        token_offset: 0,
                        is_error: true,
                    });
                    return MatchOutcome::Mismatch;
                }
                let frame_key = format!("rule:{target}");
                if context.call_stack.contains(&frame_key) {
                    return MatchOutcome::Mismatch;
                }
                if let Some(rule) = resolver.resolve_rule(target) {
                    let old_self = context.self_dc;
                    context.self_dc = Some(target.to_u128());
                    context.call_stack.push(frame_key);
                    context.depth = context.depth.saturating_add(1);

                    let (offset_after_fmt, has_fmt_token) = match target {
                        CharTarget::Format(_) if stream.first() == Some(&target.to_u128()) => {
                            (1usize, true)
                        }
                        _ => (0usize, false),
                    };

                    let sub_stream = stream.get(offset_after_fmt..).unwrap_or(&[]);
                    let outcome = match_pattern(sub_stream, &rule.pattern, context);

                    context.depth = context.depth.saturating_sub(1);
                    context.call_stack.pop();
                    context.self_dc = old_self;

                    if outcome.is_matched() {
                        let consumed_payload = outcome.consumed_tokens();
                        let mut total = offset_after_fmt.saturating_add(consumed_payload);
                        let dc_end_repr =
                            ctb_storage_minimal::global_graph_layout::SHORT_DC_REGION_START
                                .saturating_add(512);
                        if has_fmt_token && stream.get(total) == Some(&dc_end_repr) {
                            total = total.saturating_add(1);
                        }
                        return MatchOutcome::Matched { consumed: total };
                    }
                    return outcome;
                }
                return MatchOutcome::Mismatch;
            }

            if target_matches_token(target, first) {
                MatchOutcome::Matched { consumed: 1 }
            } else {
                MatchOutcome::Mismatch
            }
        }
    };

    outcome
}

/// Matches a quantified `SyntaxElement` against a stream.
fn match_element(
    stream: &[u128],
    element: &SyntaxElement,
    context: &mut MatchContext,
) -> MatchOutcome {
    let mut total_consumed = 0usize;
    let mut count = 0usize;

    loop {
        let remaining = match stream.get(total_consumed..) {
            Some(s) => s,
            None => break,
        };
        if remaining.is_empty() {
            break;
        }

        let outcome = match_term_single(remaining, &element.term, context);
        match outcome {
            MatchOutcome::Matched { consumed } if consumed > 0 => {
                total_consumed = total_consumed.saturating_add(consumed);
                count = count.saturating_add(1);
                if element.quantifier.reached_max(count) {
                    break;
                }
            }
            MatchOutcome::MatchedWithRecovery { consumed, warning } => {
                total_consumed = total_consumed.saturating_add(consumed);
                count = count.saturating_add(1);
                context.warnings.push(warning);
                if element.quantifier.reached_max(count) {
                    break;
                }
            }
            _ => break,
        }
    }

    match element.quantifier {
        Quantifier::ExactOne => {
            if count == 1 {
                MatchOutcome::Matched { consumed: total_consumed }
            } else {
                MatchOutcome::Mismatch
            }
        }
        Quantifier::Optional => MatchOutcome::Matched { consumed: total_consumed },
        Quantifier::OneOrMore => {
            if count >= 1 {
                MatchOutcome::Matched { consumed: total_consumed }
            } else {
                MatchOutcome::Mismatch
            }
        }
        Quantifier::ZeroOrMore => MatchOutcome::Matched { consumed: total_consumed },
        Quantifier::Range { min, max } => {
            let within_min = count >= min;
            let within_max = match max {
                Some(m) => count <= m,
                None => true,
            };
            if within_min && within_max {
                MatchOutcome::Matched { consumed: total_consumed }
            } else {
                MatchOutcome::Mismatch
            }
        }
    }
}

/// Evaluates a `SyntaxPattern` against a character token stream.
pub fn match_pattern(
    stream: &[u128],
    pattern: &SyntaxPattern,
    context: &mut MatchContext,
) -> MatchOutcome {
    match pattern {
        SyntaxPattern::Alternation(branches) => {
            for branch in branches {
                let mut trial_ctx = context.clone();
                let outcome = match_pattern(stream, branch, &mut trial_ctx);
                if outcome.is_matched() {
                    *context = trial_ctx;
                    return outcome;
                }
            }
            MatchOutcome::Mismatch
        }
        SyntaxPattern::Sequence(elements) => {
            let mut total_consumed = 0usize;

            for (idx, elem) in elements.iter().enumerate() {
                let remaining = match stream.get(total_consumed..) {
                    Some(s) => s,
                    None => &[],
                };

                // Handling end of stream
                if remaining.is_empty() {
                    if elem.quantifier.allows_zero() {
                        continue;
                    }
                    if context.mode == MatchMode::Strict {
                        context.has_errors = true;
                        context.diagnostics.push(SyntaxDiagnostic {
                            message: format!(
                                "Unclosed syntax structure at end-of-stream (element index {idx} in sequence)"
                            ),
                            token_offset: total_consumed,
                            is_error: true,
                        });
                        return MatchOutcome::Mismatch;
                    }
                    let warning = format!(
                        "Unclosed syntax structure at end-of-stream (element index {idx} in sequence)"
                    );
                    context.has_errors = true;
                    context.diagnostics.push(SyntaxDiagnostic {
                        message: warning.clone(),
                        token_offset: total_consumed,
                        is_error: true,
                    });
                    context.warnings.push(warning.clone());
                    return MatchOutcome::MatchedWithRecovery {
                        consumed: total_consumed,
                        warning,
                    };
                }

                let outcome = match_element(remaining, elem, context);
                match outcome {
                    MatchOutcome::Matched { consumed } => {
                        total_consumed = total_consumed.saturating_add(consumed);
                    }
                    MatchOutcome::MatchedWithRecovery { consumed, warning } => {
                        total_consumed = total_consumed.saturating_add(consumed);
                        context.warnings.push(warning);
                    }
                    MatchOutcome::Mismatch => return MatchOutcome::Mismatch,
                }
            }

            MatchOutcome::Matched { consumed: total_consumed }
        }
    }
}

/// Evaluates a full `DcSyntaxRule` against a character token stream using an
/// explicit `MatchContext`.
#[must_use]
pub fn match_syntax_rule_with_context(
    stream: &[u128],
    rule: &DcSyntaxRule,
    context: &mut MatchContext,
) -> MatchOutcome {
    match_pattern(stream, &rule.pattern, context)
}

/// Evaluates a full `DcSyntaxRule` against a character token stream in default
/// permissive mode with an optional defining short Dc ID.
#[must_use]
pub fn match_syntax_rule(
    stream: &[u128],
    rule: &DcSyntaxRule,
    self_dc: Option<u32>,
) -> (MatchOutcome, MatchContext) {
    let mut context = MatchContext::new(self_dc);
    let outcome = match_pattern(stream, &rule.pattern, &mut context);
    (outcome, context)
}

/// Evaluates a full `DcSyntaxRule` against a character token stream in default
/// permissive mode with an optional defining global character token.
#[must_use]
pub fn match_syntax_rule_with_token(
    stream: &[u128],
    rule: &DcSyntaxRule,
    self_token: Option<u128>,
) -> (MatchOutcome, MatchContext) {
    let mut context = MatchContext::new_with_token(self_token);
    let outcome = match_pattern(stream, &rule.pattern, &mut context);
    (outcome, context)
}
